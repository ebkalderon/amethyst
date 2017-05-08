//! The core engine framework.

#[cfg(feature = "profiler")]
use thread_profiler::{register_thread_with_profiler, write_profile};

// use asset_manager::AssetManager;
use config::Config;
use ecs::{Component, Gate, Planner, Priority, World};
use ecs::systems::SystemExt;
use engine::event::Event;
use engine::state::{State, StateMachine};
use engine::timing::{Stopwatch, Time};
use error::{Error, Result};
use std::time::Duration;

/// User-facing engine handle.
pub struct Engine<'e> {
    /// Configuration.
    pub config: &'e Config,
    /// Current delta time value.
    pub delta: Duration,
    /// Mutable reference to the world.
    pub world: &'e mut World,
}

/// User-friendly facade for building games. Manages main loop.
pub struct Application {
    config: Config,
    // assets: AssetManager,
    planner: Planner<()>,
    states: StateMachine,
    time: Time,
    timer: Stopwatch,
}

impl Application {
    /// Creates a new Application with the given initial game state.
    pub fn new<S: State + 'static>(initial_state: S) -> Result<Application> {
        use ecs::systems::{RenderingSystem, TransformSystem};
        ApplicationBuilder::new(initial_state, Config::default())
            .with_system::<TransformSystem>("trans", 0)
            .with_system::<RenderingSystem>("render", 0)
            .finish()
    }

    /// Builds a new application using builder pattern.
    pub fn build<S>(initial_state: S, cfg: Config) -> ApplicationBuilder<S>
        where S: State + 'static
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
            self.time.delta_time = self.timer.elapsed();
        }
    }

    /// Sets up the application.
    fn initialize(&mut self) {
        #[cfg(feature = "profiler")]
        profile_scope!("initialize");

        let mut world = self.planner.mut_world();
        world.add_resource(self.time.clone());

        let mut engine = Engine {
            config: &self.config,
            delta: self.time.delta_time,
            world: world,
        };

        self.states.start(&mut engine);
    }

    /// Advances the game world by one tick.
    fn advance_frame(&mut self) {
        {
            let mut world = self.planner.mut_world();
            let mut time = world.write_resource::<Time>().pass();
            time.delta_time = self.time.delta_time;
            time.fixed_step = self.time.fixed_step;
            time.last_fixed_update = self.time.last_fixed_update;

            let mut engine = Engine {
                config: &self.config,
                delta: self.time.delta_time,
                world: world,
            };

            #[cfg(feature = "profiler")]
            profile_scope!("handle_event");

            // let mut events = self.planner.systems.iter()
            //     .map(|s| s.poll_events())
            //     .collect();

            let mut events: Vec<Event> = Vec::new();
            while let Some(e) = events.pop() {
                self.states.handle_event(&mut engine, e);
            }

            #[cfg(feature = "profiler")]
            profile_scope!("fixed_update");
            if self.time.last_fixed_update.elapsed() >= self.time.fixed_step {
                self.states.fixed_update(&mut engine);
                self.time.last_fixed_update += self.time.fixed_step;
            }

            #[cfg(feature = "profiler")]
            profile_scope!("update");
            self.states.update(&mut engine);
        }

        #[cfg(feature = "profiler")]
        profile_scope!("dispatch");
        self.planner.dispatch(());
        self.planner.wait();
    }

    /// Writes thread_profiler profile.
    #[cfg(feature = "profiler")]
    fn write_profile(&self) {
        // TODO: Specify filename in config.
        let path = format!("{}/thread_profile.json", env!("CARGO_MANIFEST_DIR"));
        write_profile(path.as_str());
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        #[cfg(feature = "profiler")]
        self.write_profile();
    }
}

/// Helper builder for Applications.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct ApplicationBuilder<T: State + 'static>{
    config: Config,
    errors: Vec<Error>,
    #[derivative(Debug = "ignore")]
    initial_state: T,
    #[derivative(Debug = "ignore")]
    planner: Planner<()>,
}

impl<T: State + 'static> ApplicationBuilder<T> {
    /// Creates a new ApplicationBuilder with the given initial game state and
    /// display configuration.
    pub fn new(initial_state: T, cfg: Config) -> ApplicationBuilder<T> {
        use num_cpus;

        ApplicationBuilder {
            config: cfg,
            errors: Vec::new(),
            initial_state: initial_state,
            planner: Planner::with_num_threads(World::new(), num_cpus::get()),
        }
    }

    /// Registers a given component type.
    pub fn register<C: Component>(mut self) -> Self {
        self.planner.mut_world().register::<C>();
        self
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
    pub fn finish(self) -> Result<Application> {
        #[cfg(feature = "profiler")]
        register_thread_with_profiler("Main".into());
        #[cfg(feature = "profiler")]
        profile_scope!("new");

        match self.errors.last() {
            Some(_) => Err(Error::Application),
            None => Ok(Application {
                config: self.config,
                states: StateMachine::new(self.initial_state),
                planner: self.planner,
                time: Time::default(),
                timer: Stopwatch::new(),
            })
        }
    }
}
