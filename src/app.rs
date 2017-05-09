//! The core engine framework.

// use asset_manager::AssetManager;
use config::Config;
use ecs::{Gate, Planner, Priority, World};
use ecs::systems::SystemExt;
use error::{Error, Result};
use rayon::ThreadPool;
use state::{State, StateMachine};
use std::sync::Arc;
use std::time::Duration;
use timing::{Stopwatch, Time};

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
        self.time.clone()
    }

    /// Returns a mutable reference to the world.
    pub fn world_mut(&mut self) -> &mut World {
        self.planner.mut_world()
    }
}

/// User-friendly facade for building games. Manages main loop.
pub struct Application<'a> {
    engine: Engine,
    states: StateMachine<'a>,
    timer: Stopwatch,
}

impl<'a> Application<'a> {
    /// Creates a new Application with the given initial game state.
    pub fn new<S: State + 'a>(initial_state: S) -> Result<Application<'a>> {
        use ecs::systems::TransformSystem;
        ApplicationBuilder::new(initial_state, Config::default())
            .with_system::<TransformSystem>("trans", 0)
            .finish()
    }

    /// Builds a new application using builder pattern.
    pub fn build<S>(initial_state: S, cfg: Config) -> ApplicationBuilder<S>
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

        // let mut events = self.planner.systems.iter()
        //     .map(|s| s.poll_events())
        //     .collect();

        let mut events: Vec<::event::Event> = Vec::new();
        while let Some(e) = events.pop() {
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

    /// Writes thread_profiler profile.
    #[cfg(feature = "profiler")]
    fn write_profile(&self) {
        // TODO: Specify filename in config.
        let path = format!("{}/thread_profile.json", env!("CARGO_MANIFEST_DIR"));
        thread_profiler::write_profile(path.as_str());
    }
}

impl<'a> Drop for Application<'a> {
    fn drop(&mut self) {
        #[cfg(feature = "profiler")]
        self.write_profile();
    }
}

/// Helper builder for Applications.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct ApplicationBuilder<T: State>{
    config: Config,
    errors: Vec<Error>,
    #[derivative(Debug = "ignore")]
    initial_state: T,
    #[derivative(Debug = "ignore")]
    planner: Planner<()>,
    #[derivative(Debug = "ignore")]
    pool: Arc<ThreadPool>,
}

impl<'a, T: State + 'a> ApplicationBuilder<T> {
    /// Creates a new ApplicationBuilder with the given initial game state and
    /// display configuration.
    pub fn new(initial_state: T, cfg: Config) -> Self {
        use num_cpus;
        use rayon::Configuration;

        let num_cores = num_cpus::get();
        let pool_cfg = Configuration::new().num_threads(num_cores);
        let pool = ThreadPool::new(pool_cfg).map(|p| Arc::new(p)).unwrap();

        ApplicationBuilder {
            config: cfg,
            errors: Vec::new(),
            initial_state: initial_state,
            planner: Planner::from_pool(World::new(), pool.clone()),
            pool: pool,
        }
    }

    /// Adds a given system `sys`, assigns it the string identifier `name`,
    /// and marks it with the runtime priority `pri`.
    pub fn with_system<S>(mut self, name: &str, pri: Priority) -> Self
        where S: SystemExt + 'static
    {
        S::register(self.planner.mut_world());
        match S::build(&self.config) {
            Ok(sys) => self.planner.add_system(sys, name.into(), pri),
            Err(e) => self.errors.push(e),
        }

        self
    }

    /// Builds the Application and returns the result.
    pub fn finish(self) -> Result<Application<'a>> {
        #[cfg(feature = "profiler")]
        thread_profile::register_thread_with_profiler("Main".into());
        #[cfg(feature = "profiler")]
        profile_scope!("new");

        match self.errors.last() {
            Some(_) => Err(Error::Application),
            None => Ok(Application {
                engine: Engine {
                    config: self.config,
                    planner: self.planner,
                    time: Time::default(),
                    pool: self.pool,
                },
                states: StateMachine::new(self.initial_state),
                timer: Stopwatch::new(),
            })
        }
    }
}
