//! The core engine framework.

use config::Config;
use ecs::{Component, World, Dispatcher, DispatcherBuilder};
use ecs::systems::SystemExt;
use error::{Error, Result};
use event::{EventReceiver, EventSender};
use rayon::ThreadPool;
use state::{State, StateMachine};
use std::sync::Arc;
use std::time::Duration;
use timing::{Stopwatch, Time};

#[cfg(feature = "profiler")]
use thread_profiler::{register_thread_with_profiler, write_profile};

/// User-facing engine handle.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Engine<'a> {
    config: Config,
    #[derivative(Debug = "ignore")]
    dispatcher: Dispatcher<'a, 'a>,
    #[derivative(Debug = "ignore")]
    world: World,
    #[derivative(Debug = "ignore")]
    pool: Arc<ThreadPool>,
    time: Time,
}

impl<'a> Engine<'a> {
    /// Spawns a parallel task in the engine threadpool.
    pub fn spawn_task<R, T: FnOnce() -> R + Send> (&self, task: T) -> R {
        self.pool.install(task)
    }

    /// Sets the fixed time step duration for `fixed_update()`.
    pub fn set_fixed_step<D: Into<Duration>>(&mut self, fixed_delta: D) {
        self.time.fixed_step = fixed_delta.into();
    }

    /// Gets current timing information from the engine.
    pub fn time(&self) -> Time {
        self.time
    }

    /// Returns a mutable reference to the world.
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}

/// User-friendly facade for building games. Manages main loop.
#[derive(Debug)]
pub struct Application<'a> {
    engine: Engine<'a>,
    events: EventReceiver,
    states: StateMachine<'a>,
    timer: Stopwatch,
}

impl<'a> Application<'a> {
    /// Creates a new Application with the given initial game state.
    pub fn new<S: State + 'a>(initial_state: S) -> Result<Application<'a>> {
        use ecs::systems::TransformSystem;
        ApplicationBuilder::new(initial_state, Config::default())
            .with_components(|w| TransformSystem::register(w))
            .with_systems(|p| p.add(TransformSystem::default(), "trans", &[]))
            .finish()
    }

    /// Builds a new application using builder pattern.
    pub fn build<S>(initial_state: S, cfg: Config) -> ApplicationBuilder<'a, S>
        where S: State + 'a
    {
        ApplicationBuilder::new(initial_state, cfg)
    }

    /// Starts the application and manages the game loop.
    pub fn run(&mut self) {
        self.initialize();

        while self.states.is_running() {
            self.timer.restart();
            self.advance_frame();
            self.timer.stop();
            self.engine.time.delta_time = self.timer.elapsed();
        }
    }

    /// Sets up the application.
    fn initialize(&mut self) {
        #[cfg(feature = "profiler")]
        profile_scope!("initialize");

        let time = self.engine.time.clone();
        self.engine.world.add_resource(time);

        self.states.start(&mut self.engine);
    }

    /// Advances the game world by one tick.
    fn advance_frame(&mut self) {
        let ref mut engine = self.engine;

        let mut time = Time {
            delta_time: engine.time.delta_time,
            fixed_step: engine.time.fixed_step,
            last_fixed_update: engine.time.last_fixed_update,
        };

        #[cfg(feature = "profiler")]
        profile_scope!("handle_event");
        for e in self.events.try_iter() {
            self.states.handle_event(engine, e);
        }

        #[cfg(feature = "profiler")]
        profile_scope!("fixed_update");
        if time.last_fixed_update.elapsed() >= time.fixed_step {
            self.states.fixed_update(engine);
            time.last_fixed_update += time.fixed_step;
        }
        *engine.world.write_resource::<Time>() = time;

        #[cfg(feature = "profiler")]
        profile_scope!("update");
        self.states.update(engine);

        #[cfg(feature = "profiler")]
        profile_scope!("dispatch");
        engine.dispatcher.dispatch(&mut engine.world.res);
    }
}

#[cfg(feature = "profiler")]
impl<'a> Drop for Application<'a> {
    fn drop(&mut self) {
        // TODO: Specify filename in config.
        let path = format!("{}/thread_profile.json", env!("CARGO_MANIFEST_DIR"));
        write_profile(path.as_str());
    }
}

/// Helper builder for Applications.
pub struct ApplicationBuilder<'a, T: State>{
    add_comps: Box<Fn(&mut World)>,
    add_systems: Box<Fn(DispatcherBuilder<'a, 'a>) -> DispatcherBuilder<'a, 'a>>,
    config: Config,
    event_recv: EventReceiver,
    event_send: EventSender,
    initial_state: T,
    world: World,
}

impl<'a, T: State + 'a> ApplicationBuilder<'a, T> {
    /// Creates a new ApplicationBuilder with the given initial game state and
    /// display configuration.
    pub fn new(initial_state: T, cfg: Config) -> Self {
        use std::sync::mpsc::channel;

        let (send, recv) = channel();

        ApplicationBuilder {
            add_comps: Box::new(|w| {}),
            add_systems: Box::new(|p| p),
            config: cfg,
            event_recv: recv,
            event_send: send,
            initial_state: initial_state,
            world: World::new(),
        }
    }

    /// Registers a set of component types to be used in the game.
    pub fn with_components<F>(mut self, f: F) -> Self
        where F: Fn(&mut World) + 'static
    {
        self.add_comps = Box::new(f);
        self
    }

    /// Adds a given system `sys`, assigns it the string identifier `name`,
    /// and marks it with the runtime priority `pri`.
    pub fn with_systems<F>(mut self, f: F) -> Self
        where F: Fn(DispatcherBuilder<'a, 'a>) -> DispatcherBuilder<'a, 'a> + 'static
    {
        self.add_systems = Box::new(f);
        self
    }

    /// Builds the Application and returns the result.
    pub fn finish(self) -> Result<Application<'a>> {
        use num_cpus;
        use rayon::Configuration;

        #[cfg(feature = "profiler")]
        register_thread_with_profiler("Main".into());
        #[cfg(feature = "profiler")]
        profile_scope!("new");

        let num_cores = num_cpus::get();
        let pool_cfg = Configuration::new().num_threads(num_cores);
        let pool = ThreadPool::new(pool_cfg).map(|p| Arc::new(p)).unwrap();

        let mut world = self.world;
        (*self.add_comps)(&mut world);
        let dispatcher_builder = DispatcherBuilder::new().with_pool(pool.clone());
        let dispatcher_builder = (*self.add_systems)(dispatcher_builder);

        let engine = Engine {
            config: self.config,
            dispatcher: dispatcher_builder.build(),
            world: world,
            pool: pool,
            time: Time::default(),
        };

        Ok(Application {
            engine: engine,
            events: self.event_recv,
            states: StateMachine::new(self.initial_state),
            timer: Stopwatch::new(),
        })
    }
}
