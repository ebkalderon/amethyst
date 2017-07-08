//! Types for constructing render passes.

#![allow(missing_docs)]

use error::Result;
use pipe::{Effect, EffectBuilder, Target, Targets};
use scene::{Model, Scene};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::Arc;
use types::{Encoder, Factory};

pub type PrepFn = Arc<Fn(&mut Encoder, &Target) + Send + Sync>;
pub type MainFn = Arc<Fn(&mut Encoder, &Target, &Effect, &Scene, &Model) + Send + Sync>;
pub type PostFn = Arc<Fn(&mut Encoder, &Target, &Effect, &Scene) + Send + Sync>;

pub enum Pass {
    Prep(PrepPass),
    Main(MainPass),
    Post(PostPass),
}

/// Simple prepparation pass
#[derive(Clone)]
pub struct PrepPass(PrepFn);

/// Main pass renders each model
#[derive(Clone)]
pub struct MainPass(MainFn, Effect);

/// Post-processing pass
#[derive(Clone)]
pub struct PostPass(PostFn, Effect);

impl PrepPass {
    /// Applies the rendering pass using the given `Encoder` and `Target`.
    pub fn apply(&self, enc: &mut Encoder, out: &Target) {
        (self.0)(enc, out)
    }
}

impl Debug for PrepPass {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_tuple("PrepPass")
            .field(&"[closure]")
            .finish()
    }
}

impl MainPass {
    /// Applies the rendering pass using the given `Encoder`, `Target`, `Scene` and `Model`.
    pub fn apply(&self, enc: &mut Encoder, out: &Target, scene: &Scene, model: &Model) {
        (self.0)(enc, out, &self.1, scene, model)
    }
}

impl Debug for MainPass {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_tuple("MainPass")
            .field(&"[closure]")
            .field(&self.1)
            .finish()
    }
}

impl PostPass {
    /// Applies the rendering pass using the given `Encoder`, `Target` and `Scene`.
    pub fn apply(&self, enc: &mut Encoder, out: &Target, scene: &Scene) {
        (self.0)(enc, out, &self.1, scene)
    }
}

impl Debug for PostPass {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_tuple("PostPass")
            .field(&"[closure]")
            .field(&self.1)
            .finish()
    }
}

#[derive(Clone)]
pub enum PassBuilder<'a> {
    Prep(PrepFn),
    Main(MainFn, EffectBuilder<'a>),
    Post(PostFn, EffectBuilder<'a>),
}

impl<'a> PassBuilder<'a> {
    pub fn prep<F>(func: F) -> Self
        where F: Fn(&mut Encoder, &Target) + Send + Sync + 'static
    {
        PassBuilder::Prep(Arc::new(func))
    }
    pub fn main<F>(eb: EffectBuilder<'a>, func: F) -> Self
        where F: Fn(&mut Encoder, &Target, &Effect, &Scene, &Model) + Send + Sync + 'static
    {
        PassBuilder::Main(Arc::new(func), eb)
    }

    pub fn post<F>(eb: EffectBuilder<'a>, func: F) -> Self
        where F: Fn(&mut Encoder, &Target, &Effect, &Scene) + Send + Sync + 'static
    {
        PassBuilder::Post(Arc::new(func), eb)
    }

    pub(crate) fn finish(self, fac: &mut Factory, t: &Targets, out: &Target) -> Result<Pass> {
        match self {
            PassBuilder::Prep(f) => Ok(Pass::Prep(PrepPass(f))),
            PassBuilder::Main(f, e) => Ok(Pass::Main(MainPass(f, e.finish(fac, out)?))),
            PassBuilder::Post(f, e) => Ok(Pass::Post(PostPass(f, e.finish(fac, out)?))),
        }
    }
}

impl<'a> Debug for PassBuilder<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            PassBuilder::Prep(_) => {
                fmt.debug_tuple("Prep")
                    .field(&"[closure]")
                    .finish()
            }
            PassBuilder::Main(_, ref e) => {
                fmt.debug_tuple("Main")
                    .field(&"[closure]")
                    .field(e)
                    .finish()
            }
            PassBuilder::Post(_, ref e) => {
                fmt.debug_tuple("Post")
                    .field(&"[closure]")
                    .field(e)
                    .finish()
            }
        }
    }
}
