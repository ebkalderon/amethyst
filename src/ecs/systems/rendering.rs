//! Rendering system.

use config::Config;
use ecs::{RunArg, System, World};
use event::EventSender;
use error::Result;
use renderer::prelude::*;
use super::SystemExt;
use glutin::EventsLoop;

/// Rendering system.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct RenderingSystem {
    events: EventSender,
    // pipe: Pipeline,
    // renderer: Renderer,
    scene: Scene,
    #[derivative(Debug = "ignore")]
    win_events: EventsLoop,
}

impl SystemExt for RenderingSystem {
    fn build(_: &Config, send: EventSender) -> Result<RenderingSystem> {
        let events = EventsLoop::new();
        // let mut renderer = Renderer::new(&events).unwrap();
        // let pipe = renderer.create_pipe(Pipeline::forward()).unwrap();
        Ok(RenderingSystem {
            events: send,
            // pipe: pipe,
            // renderer: renderer,
            scene: Scene::default(),
            win_events: events,
        })
    }

    fn register(_world: &mut World) {}
}

impl System<()> for RenderingSystem {
    fn run(&mut self, arg: RunArg, _: ()) {
        use std::time::Duration;

        let ents = arg.fetch(|w| {
            w.entities()
        });


        // self.win_events.poll_events(|e| {
        //     self.events.send(e.into()).expect("Broken channel");
        // });
    }
}
