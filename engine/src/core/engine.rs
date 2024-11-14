
use std::cell::RefCell;

use crate::window;

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
}

impl Engine {
    pub fn new() -> Engine {
        return Engine{
            game:          RefCell::new(Box::new(DefaultGame{})),
            window_system: window::WindowSystem::new(),
        }
    }

    pub fn register_game(&self, game: Box<dyn Game>) {
        self.game.replace(game);
    }

    pub fn run(&self) {
        let mut game = self.game.borrow_mut();

        // run post-game setup
        let client_window = self.window_system.create_window("Chibi Vulkan", 1920, 1080);

        // initialize the game
        let mut game_res = game.on_init();
        if !game_res {
            return;
        }

        loop {
            if !self.window_system.pump_window_message() {
                break;
            }

            if client_window.should_window_close() {
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
