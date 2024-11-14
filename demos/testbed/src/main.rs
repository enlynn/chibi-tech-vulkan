extern crate chibi_engine;

use chibi_engine::core::engine;

struct Testbed{}

impl engine::Game for Testbed {
    fn on_init(&mut self)     -> bool { return true; }
    fn on_update(&mut self)   -> bool { return true; }
    fn on_render(&mut self)   -> bool { return true; }
    fn on_shutdown(&mut self) -> bool { return true; }
}

fn main() {
    let testbed = Box::new(Testbed{});

    let chibi_engine = chibi_engine::new_engine();
    chibi_engine.register_game(testbed);

    chibi_engine.run();
}
