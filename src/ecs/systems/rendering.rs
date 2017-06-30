//! Rendering system.

use config::Config;
use ecs::{WriteStorage, ReadStorage, Entities, System, World};
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

impl<'a> SystemExt<'a> for RenderingSystem {
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

impl<'a> System<'a> for RenderingSystem {
    type SystemData = Entities<'a>;
    fn run(&mut self, entities: Entities<'a>) {
        use std::time::Duration;
        
        // self.win_events.poll_events(|e| {
        //     self.events.send(e.into()).expect("Broken channel");
        // });
    }
}
