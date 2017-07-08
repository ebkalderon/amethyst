//! A stage in the rendering pipeline.

use error::{Error, Result};
use pipe::{Target, Targets};
use pipe::pass::{MainPass, Pass, PassBuilder, PostPass, PrepPass};
use scene::{Model, Scene};
use std::sync::Arc;
use types::{Encoder, Factory};

/// A stage in the rendering pipeline.
#[derive(Clone, Debug)]
pub struct Stage {
    enabled: bool,
    prep_passes: Vec<PrepPass>,
    main_passes: Vec<MainPass>,
    post_passes: Vec<PostPass>,
    target: Arc<Target>,
}

impl Stage {
    /// Creates a new stage using the Target with the given name.
    pub fn with_target<'a, T: Into<String>>(target_name: T) -> StageBuilder<'a> {
        StageBuilder::new(target_name.into())
    }

    /// Creates a new layer which draws straight into the backbuffer.
    pub fn with_backbuffer<'a>() -> StageBuilder<'a> {
        StageBuilder::new("")
    }

    /// Sets whether this layer should execute.
    pub fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Checks whether this layer is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Applies all passes in this stage to the given `Scene` and outputs the
    /// result to the proper target.
    pub fn apply_prep(&self, enc: &mut Encoder) {
        if self.enabled {
            for pass in self.prep_passes.iter() {
                pass.apply(enc, &self.target);
            }
        }
    }

    /// Applies all passes in this stage to the given `Scene` and outputs the
    /// result to the proper target.
    pub fn apply_main(&self, enc: &mut Encoder, scene: &Scene, model: &Model) {
        if self.enabled {
            for pass in self.main_passes.iter() {
                pass.apply(enc, &self.target, scene, model);
            }
        }
    }

    /// Applies all passes in this stage to the given `Scene` and outputs the
    /// result to the proper target.
    pub fn apply_post(&self, enc: &mut Encoder, scene: &Scene) {
        if self.enabled {
            for pass in self.post_passes.iter() {
                pass.apply(enc, &self.target, scene);
            }
        }
    }
}

/// Constructs a new rendering stage.
#[derive(Clone, Debug)]
pub struct StageBuilder<'a> {
    enabled: bool,
    passes: Vec<PassBuilder<'a>>,
    target_name: String,
}

impl<'a> StageBuilder<'a> {
    /// Creates a new `StageBuilder` using the given target.
    pub fn new<T: Into<String>>(target_name: T) -> Self {
        StageBuilder {
            enabled: true,
            passes: Vec::new(),
            target_name: target_name.into(),
        }
    }

    /// Appends another `Pass` to the stage.
    pub fn with_pass<P: Into<PassBuilder<'a>>>(mut self, pass: P) -> Self {
        self.passes.push(pass.into());
        self
    }

    /// Sets whether the `Stage` is turned on by default.
    pub fn enabled(mut self, val: bool) -> Self {
        self.enabled = val;
        self
    }

    /// Builds and returns the stage.
    #[doc(hidden)]
    pub(crate) fn finish(mut self, fac: &mut Factory, targets: &Targets) -> Result<Stage> {
        let name = self.target_name;
        let out = targets
            .get(&name)
            .cloned()
            .ok_or(Error::NoSuchTarget(name))?;

        let mut prep_passes = Vec::new();
        let mut main_passes = Vec::new();
        let mut post_passes = Vec::new();

        for pass in self.passes.into_iter().map(|pb| pb.finish(fac, targets, &out)) {
            match pass? {
                Pass::Prep(pass) => prep_passes.push(pass),
                Pass::Main(pass) => main_passes.push(pass),
                Pass::Post(pass) => post_passes.push(pass),
            }
        }

        Ok(Stage {
            enabled: self.enabled,
            prep_passes: prep_passes,
            main_passes: main_passes,
            post_passes: post_passes,
            target: out,
        })
    }
}
