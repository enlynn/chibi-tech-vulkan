
use std::cell::RefCell;

use crate::window;
use crate::renderer;

pub trait Game {
    fn on_init(&mut self)     -> bool;
    fn on_update(&mut self)   -> bool;
    fn on_render(&mut self)   -> bool; //Will this function be necessary?
    fn on_shutdown(&mut self) -> bool;
}

pub struct DefaultGame {}
impl Game for DefaultGame {
    fn on_init(&mut self)     -> bool { return false; }
    fn on_update(&mut self)   -> bool { return false; }
    fn on_render(&mut self)   -> bool { return false; }
    fn on_shutdown(&mut self) -> bool { return false; }
}

pub struct Engine {
    game:          RefCell<Box<dyn Game>>,
    window_system: window::WindowSystem,
    client_window: window::Window,
    render_system: renderer::RenderSystem,
}

impl Engine {
    pub fn new() -> Engine {
        let game = RefCell::new(Box::new(DefaultGame{}));
        let window_system = window::WindowSystem::new();
        // todo: set window title/width/height based on user game data.
        let client_window = window_system.create_window("Chibi Vulkan", 1920, 1080);
        let render_system = renderer::RenderSystem::new(renderer::RendererCreateInfo{
            surface: client_window.get_native_surface(),
        });

        return Engine{
            game,
            window_system,
            client_window,
            render_system,
        }
    }

    pub fn register_game(&self, game: Box<dyn Game>) {
        self.game.replace(game);
    }

    pub fn run(&self) {
        let mut game = self.game.borrow_mut();

        // run post-game engine setup


        // initialize the game
        let mut game_res = game.on_init();
        if !game_res {
            return;
        }

        loop {
            if !self.window_system.pump_window_message() {
                break;
            }

            if self.client_window.should_window_close() {
                break;
            }

            game_res = game.on_update();
            if !game_res {
                break;
            }

            game_res = game.on_render();
            if !game_res {
                break;
            }
        }

        game.on_shutdown();
    }
}
