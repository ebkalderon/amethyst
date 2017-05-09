//! Rendering system.

use config::Config;
use ecs::{RunArg, System, World};
use event::EventSender;
use error::Result;
use renderer::prelude::*;
use super::SystemExt;
use winit::EventsLoop;

/// Rendering system.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct RenderingSystem {
    events: EventSender,
    // renderer: Renderer,
    scene: Scene,
    #[derivative(Debug = "ignore")]
    win_events: EventsLoop,
}

impl SystemExt for RenderingSystem {
    fn build(_: &Config, send: EventSender) -> Result<RenderingSystem> {
        let events = EventsLoop::new();
        // let renderer = Renderer::new(&events)?;
        Ok(RenderingSystem {
            events: send,
            // renderer: Mutex::new(renderer),
            scene: Scene::default(),
            win_events: events,
        })
    }

    fn register(_world: &mut World) {}
}

impl System<()> for RenderingSystem {
    fn run(&mut self, arg: RunArg, _: ()) {
        self.win_events.poll_events(|e| {
            self.events.send(e.into()).expect("Broken channel");
        });
    }
}
