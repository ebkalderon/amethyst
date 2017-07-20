//! A stage in the rendering pipeline.

use error::{Error, Result};
use pipe::{Target, Targets};
use pipe::pass::{ModelPass, Pass, PassBuilder, SimplePass, BasicPass};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use scene::{Model, Scene};
use std::sync::Arc;
use types::{Encoder, Device, Factory};

/// A stage in the rendering pipeline.
#[derive(Clone, Debug)]
pub struct Stage {
    enabled: bool,
    passes: Vec<Pass>,
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

    /// Get count of parallelable passes
    pub fn encoders_required(&self, jobs_count: usize)-> usize {
        self.passes.iter().map(|pass| match *pass {
            Pass::Basic(_) => 0,
            Pass::Simple(_) => 0,
            Pass::Model(_) => 1,
        }).sum::<usize>() * (jobs_count - 1) + 1
    }

    /// Applies all passes in this stage to the given `Scene` and outputs the
    /// result to the proper target.
    pub fn apply<'a>(&self, mut encoders: &'a mut [Encoder], jobs_count: usize, scene: &Scene) -> &'a mut [Encoder] {
        if self.enabled {
            // Numbers of encoders must be enough to run all parallellable passes
            // in specified numbers of jobs
            // Note that one encoder is reused each time
            assert!(self.encoders_required(jobs_count) <= encoders.len());

            for pass in self.passes.iter() {
                match *pass {
                    // Passes that do not requires running in parallel submit their
                    // commands into first encoder available
                    Pass::Basic(ref pass) => pass.apply(&mut encoders[0], &self.target),
                    Pass::Simple(ref pass) => pass.apply(&mut encoders[0], &self.target, scene),
                    Pass::Model(ref pass) => {
                        // Retrive models in chunks
                        let mut mod_par_iter = scene.par_chunks_models(jobs_count);

                        // Check that there is enough encoders
                        assert!(encoders.len() >= mod_par_iter.len());

                        // Split off used encoders except last one
                        encoders = {
                            let encoders = encoders;
                            let (touse, left) = encoders.split_at_mut(jobs_count - 1);
                            // Apply pass for models
                            mod_par_iter.zip(touse).for_each(|(models, enc)| {
                                for model in models {
                                    pass.apply(enc, &self.target, scene, model);
                                }
                            });
                            left
                        };
                    }
                }
            }
        }
        encoders 
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

        let passes = self.passes.into_iter().map(|pb| pb.finish(fac, targets, &out)).collect::<Result<Vec<_>>>()?;

        Ok(Stage {
            enabled: self.enabled,
            passes: passes,
            target: out,
        })
    }
}
