use std::collections::VecDeque;
use std::sync::mpsc::TryRecvError;
use std::sync::{mpsc, Arc, RwLock, Mutex, Condvar};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{hint, thread};
use std::cell::RefCell;

use super::system::*;
use super::command_buffer::*;

const MAX_QUEUED_FRAMES: usize = 3;

struct ThreadFence {
    mutex:   Mutex<bool>,
    condvar: Condvar,
}

impl ThreadFence {
    pub fn new() -> Self {
        Self{
            mutex:   Mutex::new(false),
            condvar: Condvar::new(),
        }
    }

    // acquire a frame for rendering
    fn acquire(&self) {
        // wait until the frame has finished rendering.
        let mut acquired = self.mutex.lock().expect("Failed to acquire lock.");
        while *acquired {
            acquired = self.condvar.wait(acquired).unwrap();
        }

        // we have locked this frame for rendering
        *acquired = true;
    }

    // signals that a frame has finished rendering
    fn signal(&self) {
        let mut acquired = self.mutex.lock().expect("Failed to acquire lock.");
        *acquired = false;

        // notify the main thread if it is waiting
        self.condvar.notify_all();
    }
}

pub struct RtcSubmitCommandList {
    cmd_buffer: RenderCommandBuffer,
}

enum RenderThreadCommand {
    DestroyRenderer,
    SubmitCommandList(RtcSubmitCommandList),
    RenderFrame(Arc<ThreadFence>),
    Resize(u32, u32), //(width, height)
}

pub struct RtrCommandList {}

#[derive(PartialEq)]
pub enum RenderThreadResponse {
    RenderFrameDone,
    RendererShutdown,
    SubmitCommandList,
}

struct FrameSync {
    fences:      [Arc<ThreadFence>; MAX_QUEUED_FRAMES],
    fence_index: usize,
}

pub struct RenderThread {
    handle:              thread::JoinHandle<()>,
    render_thread_queue: mpsc::Sender<RenderThreadCommand>,
    main_thread_queue:   mpsc::Receiver<RenderThreadResponse>,
    sync:                RefCell<FrameSync>,
}

fn process_render_command(render_system: &mut RenderSystem, command: RenderThreadCommand) -> Option<RenderThreadResponse> {
    match command {
        RenderThreadCommand::DestroyRenderer => {
            render_system.destroy();
            return Some(RenderThreadResponse::RendererShutdown);
        },

        RenderThreadCommand::SubmitCommandList(rtc_submit_command_list) => {
            render_system.submit_render_commands(rtc_submit_command_list.cmd_buffer);
            return None; //todo: we'll probably want to send a response buffer...
        },

        RenderThreadCommand::RenderFrame(fence) => {
            render_system.render();
            fence.signal(); // let the main thread know we have finished this frame.

            return Some(RenderThreadResponse::RenderFrameDone);
        },

        RenderThreadCommand::Resize(width, height) => {
            render_system.on_resize(width, height);
            return None;
        },
    }
}

// so this is a little silly, but c_void pointers don't implement Send! Ok, it makes sense,
// but it is not a real concern for this, so I am going to, uh, sneak this in there.
unsafe impl Send for RendererCreateInfo {}
unsafe impl Send for RenderCommand {}

pub fn create_render_thread(create_info: RendererCreateInfo) -> RenderThread {
    let (main_thread_sender,   main_thread_reciever)   = mpsc::channel();
    let (render_thread_sender, render_thread_reciever) = mpsc::channel();

    let thread_handler = thread::spawn(move || {
        // Take ownership of the needed channels
        let local_sender:   mpsc::Sender<RenderThreadResponse>  = main_thread_sender.clone();
        let local_reciever: mpsc::Receiver<RenderThreadCommand> = render_thread_reciever;

        // the render thread will own the render system.
        let mut render_system = RenderSystem::new(create_info);

        'thread_loop: loop {
            if let Ok(msg) = local_reciever.recv() {
                let process_response = process_render_command(&mut render_system, msg);
                if let Some(response) = process_response {
                    let is_shutdown = response == RenderThreadResponse::RendererShutdown;

                    match local_sender.send(response) {
                        Ok(_)  => {},
                        Err(e) => println!("Failed to send a message to the main thread from the render thread."),
                    }

                    if is_shutdown {
                        break 'thread_loop;
                    }
                }
            }
        }
    });

    RenderThread{
        handle:              thread_handler,
        render_thread_queue: render_thread_sender,
        main_thread_queue:   main_thread_reciever,
        sync:                RefCell::new(FrameSync{
            fences:      [
                Arc::new(ThreadFence::new()),
                Arc::new(ThreadFence::new()),
                Arc::new(ThreadFence::new()),
            ],
            fence_index: 0,
        }),
    }
}

impl RenderThread {
    fn send_message(&self, cmd: RenderThreadCommand) {
        match self.render_thread_queue.send(cmd) {
            Ok(_) => {},
            Err(e) => {
                panic!("Failed to send a message to the render thread: {:?}", e);
            },
        }
    }

    pub fn recieve_message(&self, blocking: bool) -> Option<RenderThreadResponse> {
        if blocking {
            match self.main_thread_queue.recv() {
                Ok(msg) => return Some(msg),
                Err(e)  => panic!("Failed to recv a message from the render thread: {:?}", e),
            }
        } else {
            match self.main_thread_queue.try_recv() {
                Ok(msg) => return Some(msg),
                Err(e)  => {
                    match e {
                        TryRecvError::Empty        => return None,
                        TryRecvError::Disconnected => panic!("Failed to recv a message from the render thread: {:?}", e),
                    }
                },
            }
        }

        None
    }

    fn get_fence(&self) -> Arc<ThreadFence> {
        let sync = self.sync.borrow();
        let fence = sync.fences[sync.fence_index].clone();
        fence.acquire();

        return fence;
    }

    fn acquire_frame(&self) -> Arc<ThreadFence> {
        let fence = self.get_fence();

        let mut sync = self.sync.borrow_mut();
        sync.fence_index = (sync.fence_index + 1) % MAX_QUEUED_FRAMES;

        return fence;
    }

    pub fn submit_command_buffer(&self, cmd_buffer: RenderCommandBuffer) {
        self.send_message(RenderThreadCommand::SubmitCommandList(RtcSubmitCommandList { cmd_buffer }));
    }

    // this will block the calling thread until the frame is ready to be rendered
    //   todo: determine if I want to hide this from the caller.
    pub fn render_frame(&self, frame: usize) {
        let fence = self.acquire_frame();
        self.send_message(RenderThreadCommand::RenderFrame(fence));
    }

    pub fn on_resize(&self, width: u32, height: u32) {
        self.send_message(RenderThreadCommand::Resize(width, height));
    }

    pub fn destroy(&self) {
        self.send_message(RenderThreadCommand::DestroyRenderer);

        'msg_loop: loop {
            // block until the renderer has fully shutdown
            if let Some(response) = self.recieve_message(true) {
                if response == RenderThreadResponse::RendererShutdown {
                    break 'msg_loop;
                }
            }
        }
    }
}
