//! Opens an empty window.

extern crate amethyst;

use amethyst::prelude::*;
use amethyst::ecs::systems::TransformSystem;

struct Example;

impl State for Example {
    fn handle_event(&mut self, _: &mut Engine, event: Event) -> Trans {
        match event {
            Event::Window(e) => match e {
                WindowEvent::KeyboardInput(_, _, Some(Key::Escape), _) |
                WindowEvent::Closed => Trans::Quit,
                _ => Trans::None,
            },
            _ => Trans::None,
        }
    }
}

fn main() {
    let path = format!("{}/examples/01_window/resources/config.yml",
                       env!("CARGO_MANIFEST_DIR"));
    let cfg = Config::from_file(path).unwrap();
    let mut game = Application::build(Example, cfg)
        .with_system::<TransformSystem>("trans", 0)
        .finish()
        .expect("Fatal error");

    game.run();
}
