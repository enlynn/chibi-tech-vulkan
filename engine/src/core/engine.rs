
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::path::PathBuf;
use std::ptr;

use assetlib::mesh::Vertex;

use crate::window;
use crate::renderer::{
    command_buffer::*,
    system::{RenderSystem, RendererCreateInfo},
    thread::*,
};

use super::asset_system::*;

pub trait Game {
    fn on_init(&mut self)                                                         -> bool;
    fn on_update(&mut self, frame_time_ms: f64, commands: &[RenderCommandBuffer]) -> bool;
    fn on_render(&mut self)                                                       -> bool; //Will this function be necessary?
    fn on_shutdown(&mut self)                                                     -> bool;
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
    fn on_init(&mut self)                                                        -> bool { return false; }
    fn on_update(&mut self, _frame_time: f64, _commands: &[RenderCommandBuffer]) -> bool { return false; }
    fn on_render(&mut self)                                                      -> bool { return false; }
    fn on_shutdown(&mut self)                                                    -> bool { return false; }
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

        let refresh_rate = 1.0 / 60.0; //todo: choose based on monitor/settings

        use std::time::{Duration, Instant};
        let mut frame_timer = Instant::now();
        let mut last_frame_time_ms = 0.0f64;

        loop {
            frame_timer = Instant::now();

            if !self.window_system.pump_window_message() {
                break;
            }

            if self.client_window.borrow().should_window_close() {
                break;
            }

            // Process any waiting messages from the renderer
            //
            let mut last_frame_rendered = false;
            let mut render_commands: Vec<RenderCommandBuffer> = Vec::new();
            while let Some(msg) = self.render_thread.recieve_message(false) {
                match msg {
                    RenderThreadResponse::RenderFrameDone              => last_frame_rendered = true,
                    RenderThreadResponse::RendererShutdown             => todo!(),
                    RenderThreadResponse::SubmitCommandList(response)  => { render_commands.push(response.cmd_buffer); },
                }
            }

            game_res = game.on_update(last_frame_time_ms, &render_commands);
            if !game_res {
                break;
            }

            game_res = game.on_render();
            if !game_res {
                break;
            }

            self.render_thread.render_frame(frame_index);

            let elapsed_time = Instant::now();
            let sec_elapsed = (elapsed_time - frame_timer).as_secs_f64();
            if (sec_elapsed < refresh_rate)
            {
                let sleep_ms = (1000.0 * (refresh_rate - sec_elapsed)) as u64;
                std::thread::sleep(Duration::from_millis(sleep_ms));
            }

            last_frame_time_ms = (Instant::now() - frame_timer).as_millis_f64();
            //println!("\tFinished rendering. {}", last_frame_time_ms);

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
