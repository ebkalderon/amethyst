//! The core engine framework.

use config::Config;
use ecs::{Gate, Planner, Priority, World};
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
    /// Creates a new `Engine` handle from the given inputs.
    #[doc(hidden)]
    pub fn new(cfg: Config, plan: Planner<()>, pool: Arc<ThreadPool>) -> Engine {
        Engine {
            config: cfg,
            planner: plan,
            pool: pool,
            time: Time::default(),
        }
    }

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
pub struct Application<'a> {
    engine: Engine,
    events: EventReceiver,
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
        for e in self.events.iter() {
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
impl<'a> Drop for Application<'a> {
    fn drop(&mut self) {
        // TODO: Specify filename in config.
        let path = format!("{}/thread_profile.json", env!("CARGO_MANIFEST_DIR"));
        write_profile(path.as_str());
    }
}

/// Helper builder for Applications.
pub struct ApplicationBuilder<T: State>{
    config: Config,
    errors: Vec<Error>,
    initial_state: T,
    planner: Planner<()>,
    pool: Arc<ThreadPool>,
    recv: EventReceiver,
    send: EventSender,
}

impl<'a, T: State + 'a> ApplicationBuilder<T> {
    /// Creates a new ApplicationBuilder with the given initial game state and
    /// display configuration.
    pub fn new(initial_state: T, cfg: Config) -> Self {
        use num_cpus;
        use rayon::Configuration;
        use std::sync::mpsc;

        let num_cores = num_cpus::get();
        let pool_cfg = Configuration::new().num_threads(num_cores);
        let pool = ThreadPool::new(pool_cfg).map(|p| Arc::new(p)).unwrap();
        let (send, recv) = mpsc::channel();

        ApplicationBuilder {
            config: cfg,
            errors: Vec::new(),
            initial_state: initial_state,
            planner: Planner::from_pool(World::new(), pool.clone()),
            pool: pool,
            recv: recv,
            send: send,
        }
    }

    /// Adds a given system `sys`, assigns it the string identifier `name`,
    /// and marks it with the runtime priority `pri`.
    pub fn with_system<S>(mut self, name: &str, pri: Priority) -> Self
        where S: SystemExt + 'static
    {
        S::register(self.planner.mut_world());
        match S::build(&self.config, self.send.clone()) {
            Ok(sys) => self.planner.add_system(sys, name.into(), pri),
            Err(e) => self.errors.push(e),
        }

        self
    }

    /// Builds the Application and returns the result.
    pub fn finish(self) -> Result<Application<'a>> {
        #[cfg(feature = "profiler")]
        register_thread_with_profiler("Main".into());
        #[cfg(feature = "profiler")]
        profile_scope!("new");

        match self.errors.last() {
            Some(_) => Err(Error::Application),
            None => Ok(Application {
                engine: Engine::new(self.config, self.planner, self.pool),
                events: self.recv,
                states: StateMachine::new(self.initial_state),
                timer: Stopwatch::new(),
            })
        }
    }
}
