
use std::cell::RefCell;

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
    game: RefCell<Box<dyn Game>>, //TODO: not sure how I feel about using an option here.
}

impl Engine {
    pub fn new() -> Engine {
        return Engine{
            game: RefCell::new(Box::new(DefaultGame{})),
        }
    }

    pub fn register_game(&self, game: Box<dyn Game>) {
        self.game.replace(game);
    }

    pub fn run(&self) {
        let mut game = self.game.borrow_mut();

        // run post-game setup

        // initialize the game
        let mut game_res = game.on_init();
        if !game_res {
            return;
        }

        loop {
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
