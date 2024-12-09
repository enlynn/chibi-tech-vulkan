
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::path::PathBuf;
use std::ptr;

use crate::window;
use crate::renderer::{
    command_buffer::*,
    mesh::Vertex,
    system::{RenderSystem, RendererCreateInfo},
    thread::*,
};

use vendor::imgui::*;

use super::asset_system::*;

pub trait Game {
    fn on_init(&mut self)     -> bool;
    fn on_update(&mut self)   -> bool;
    fn on_render(&mut self)   -> bool; //Will this function be necessary?
    fn on_shutdown(&mut self) -> bool;
}

pub struct GameInfo {
    pub title:         String,
    pub game_version:  u32,
    pub window_width:  u32,
    pub window_height: u32,
    pub manifest_dir:  std::path::PathBuf,
}

pub struct DefaultGame {}
impl Game for DefaultGame {
    fn on_init(&mut self) -> bool { return false; }
    fn on_update(&mut self)   -> bool { return false; }
    fn on_render(&mut self)   -> bool { return false; }
    fn on_shutdown(&mut self) -> bool { return false; }
}

pub struct Engine {
    game:          RefCell<Box<dyn Game>>,
    window_system: window::WindowSystem,
    client_window: RefCell<window::Window>,
    render_thread: RenderThread,
    asset_system:  AssetSystem,
}

impl Engine {
    pub fn new(game_info: GameInfo) -> Engine {
        let game = Box::new(DefaultGame{});

        let window_system = window::WindowSystem::new();
        let client_window = window_system.create_window(
            game_info.title.as_str(),
            game_info.window_width  as i32,
            game_info.window_height as i32,
        );

        let render_thread = create_render_thread(RendererCreateInfo{
            surface: client_window.get_native_surface(),
        });

        let (width, height) = client_window.get_framebuffer_size();
        render_thread.on_resize(width, height);

        let ig_ctx = unsafe { igCreateContext(std::ptr::null_mut()) };

        return Engine{
            game: RefCell::new(game),
            window_system,
            client_window: RefCell::new(client_window),
            render_thread,
            asset_system: AssetSystem::new(game_info.manifest_dir),
        }
    }

    pub fn register_game(&self, game: Box<dyn Game>) {
        self.game.replace(game);
    }

    pub fn run(&self) {
        let mut game = self.game.borrow_mut();

        // run post-game engine setup
        let mut frame_index: usize = 0;

        // initialize the game
        let mut game_res = game.on_init();
        if !game_res {
            return;
        }

        //use std::time::{Duration, Instant};
        //let mut current_time = Instant::now();

        loop {
            if !self.window_system.pump_window_message() {
                break;
            }

            //let mut elapsed_time = Instant::now();
            //println!("\tFinished polling input. {}", (elapsed_time - current_time).as_millis_f64());
            //current_time = elapsed_time;

            if self.client_window.borrow().should_window_close() {
                break;
            }

            // Process any waiting messages from the renderer
            //
            let mut last_frame_rendered = false;
            while let Some(msg) = self.render_thread.recieve_message(false) {
                match msg {
                    RenderThreadResponse::RenderFrameDone   => last_frame_rendered = true,
                    RenderThreadResponse::RendererShutdown  => todo!(),
                    RenderThreadResponse::SubmitCommandList => todo!(),
                }
            }

            game_res = game.on_update();
            if !game_res {
                break;
            }

            // this should be the "editor_begin_frame()"
            if false {
                use vendor::imgui::*;
                use crate::util::ffi::*;

                ig_vulkan_new_frame();
                ig_glfw_new_frame();
                call!(igNewFrame);

                let mut is_open = true;
                call!(igShowDemoWindow, &mut is_open);

                //self.render_system.borrow_mut().on_editor_update();

                call!(igEndFrame);
            }

            //elapsed_time = Instant::now();
            //println!("\tFinished updating imgui. {}", (elapsed_time - current_time).as_millis_f64());
            //current_time = elapsed_time;

            game_res = game.on_render();
            if !game_res {
                break;
            }

            self.render_thread.render_frame(frame_index);

            // this should be the "editor_render_external_windows()" - render imgui viewports
            // unsafe {
            //     let io = { &mut *vendor::imgui::igGetIO() }; // gets a mutable reference
            //     if (io.ConfigFlags & vendor::imgui::ImGuiConfigFlags_ViewportsEnable as i32) != 0
            //     {
            //         vendor::imgui::igUpdatePlatformWindows();
            //         vendor::imgui::igRenderPlatformWindowsDefault(ptr::null_mut(), ptr::null_mut());
            //     }
            // }

            //elapsed_time = Instant
            //println!("\tFinished rendering. {}", (elapsed_time - current_time).as_millis_f64());
            //current_time = elapsed_time;

            frame_index += 1;
        }

        self.render_thread.destroy();
    }

    pub fn get_asset_dir(&self, drive: AssetDrive) -> PathBuf {
        return self.asset_system.get_dir(drive);
    }

    pub fn submit_render_command_buffer(&self, cmd: RenderCommandBuffer) {
        self.render_thread.submit_command_buffer(cmd);
    }

    pub fn register_window_event(&self, ev_type: window::WindowEventType, listener: window::EventListener) {
        self.client_window.borrow_mut().register_event(ev_type, listener);
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        println!("Shutting down the engine.");

        let mut game = self.game.borrow_mut();
        game.on_shutdown();
    }
}
