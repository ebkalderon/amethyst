//! The core engine framework.

use config::Config;
use ecs::{Component, Gate, Planner, Priority, World};
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
pub struct Engine {
    config: Config,
    #[derivative(Debug = "ignore")]
    planner: Planner<()>,
    #[derivative(Debug = "ignore")]
    pool: Arc<ThreadPool>,
    time: Time,
}

impl Engine {
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
        self.planner.mut_world()
    }
}

/// User-friendly facade for building games. Manages main loop.
#[derive(Debug)]
pub struct Application<'s> {
    engine: Engine,
    events: EventReceiver,
    states: StateMachine<'s>,
    timer: Stopwatch,
}

impl<'s> Application<'s> {
    /// Creates a new Application with the given initial game state.
    pub fn new<S: State + 's>(initial_state: S) -> Result<Application<'s>> {
        use ecs::systems::TransformSystem;
        ApplicationBuilder::new(initial_state, Config::default())
            .with_components(|w| TransformSystem::register(w))
            .with_systems(|p| p.add_system(TransformSystem::default(), "trans", 0))
            .finish()
    }

    /// Builds a new application using builder pattern.
    pub fn build<S>(initial_state: S, cfg: Config) -> ApplicationBuilder<S>
        where S: State + 's
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
        self.engine.planner.mut_world().add_resource(time);

        self.states.start(&mut self.engine);
    }

    /// Advances the game world by one tick.
    fn advance_frame(&mut self) {
        let ref mut engine = self.engine;

        let mut time = engine.planner.mut_world().write_resource::<Time>().pass();
        time.delta_time = engine.time.delta_time;
        time.fixed_step = engine.time.fixed_step;
        time.last_fixed_update = engine.time.last_fixed_update;

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

        #[cfg(feature = "profiler")]
        profile_scope!("update");
        self.states.update(engine);

        #[cfg(feature = "profiler")]
        profile_scope!("dispatch");
        engine.planner.dispatch(());
        engine.planner.wait();
    }
}

#[cfg(feature = "profiler")]
impl<'s> Drop for Application<'s> {
    fn drop(&mut self) {
        // TODO: Specify filename in config.
        let path = format!("{}/thread_profile.json", env!("CARGO_MANIFEST_DIR"));
        write_profile(path.as_str());
    }
}

/// Helper builder for Applications.
pub struct ApplicationBuilder<T: State>{
    add_comps: Box<Fn(&mut World)>,
    add_systems: Box<Fn(&mut Planner<()>)>,
    config: Config,
    event_recv: EventReceiver,
    event_send: EventSender,
    initial_state: T,
    world: World,
}

impl<'s, T: State + 's> ApplicationBuilder<T> {
    /// Creates a new ApplicationBuilder with the given initial game state and
    /// display configuration.
    pub fn new(initial_state: T, cfg: Config) -> Self {
        use std::sync::mpsc::channel;

        let (send, recv) = channel();

        ApplicationBuilder {
            add_comps: Box::new(|w| {}),
            add_systems: Box::new(|p| {}),
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
        where F: Fn(&mut Planner<()>) + 'static
    {
        self.add_systems = Box::new(f);
        self
    }

    /// Builds the Application and returns the result.
    pub fn finish(self) -> Result<Application<'s>> {
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
        let mut planner = Planner::from_pool(world, pool.clone());
        (*self.add_systems)(&mut planner);

        let engine = Engine {
            config: self.config,
            planner: planner,
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
