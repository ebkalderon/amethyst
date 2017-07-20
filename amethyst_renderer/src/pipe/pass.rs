//! Types for constructing render passes.

#![allow(missing_docs)]

use error::Result;
use pipe::{Effect, EffectBuilder, Target, Targets};
use scene::{Model, Scene};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::Arc;
use types::{Encoder, Factory};

pub type BasicFn = Arc<Fn(&mut Encoder, &Target) + Send + Sync>;
pub type SimpleFn = Arc<Fn(&mut Encoder, &Target, &Effect, &Scene) + Send + Sync>;
pub type ModelFn = Arc<Fn(&mut Encoder, &Target, &Effect, &Scene, &Model) + Send + Sync>;

#[derive(Clone, Debug)]
pub enum Pass {
    Basic(BasicPass),
    Simple(SimplePass),
    Model(ModelPass),
}

/// Simple prepparation pass
#[derive(Clone)]
pub struct BasicPass(BasicFn);

/// Simple-processing pass
#[derive(Clone)]
pub struct SimplePass(SimpleFn, Effect);

/// Model pass renders each model
#[derive(Clone)]
pub struct ModelPass(ModelFn, Effect);

impl BasicPass {
    /// Applies the rendering pass using the given `Encoder` and `Target`.
    pub fn apply(&self, enc: &mut Encoder, out: &Target) {
        (self.0)(enc, out)
    }
}

impl Debug for BasicPass {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_tuple("BasicPass")
            .field(&"[closure]")
            .finish()
    }
}

impl SimplePass {
    /// Applies the rendering pass using the given `Encoder`, `Target` and `Scene`.
    pub fn apply(&self, enc: &mut Encoder, out: &Target, scene: &Scene) {
        (self.0)(enc, out, &self.1, scene)
    }
}

impl Debug for SimplePass {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_tuple("SimplePass")
            .field(&"[closure]")
            .field(&self.1)
            .finish()
    }
}

impl ModelPass {
    /// Applies the rendering pass using the given `Encoder`, `Target`, `Scene` and `Model`.
    pub fn apply(&self, enc: &mut Encoder, out: &Target, scene: &Scene, model: &Model) {
        (self.0)(enc, out, &self.1, scene, model)
    }
}

impl Debug for ModelPass {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_tuple("ModelPass")
            .field(&"[closure]")
            .field(&self.1)
            .finish()
    }
}

#[derive(Clone)]
pub enum PassBuilder<'a> {
    Basic(BasicFn),
    Model(ModelFn, EffectBuilder<'a>),
    Simple(SimpleFn, EffectBuilder<'a>),
}

impl<'a> PassBuilder<'a> {
    pub fn prep<F>(func: F) -> Self
        where F: Fn(&mut Encoder, &Target) + Send + Sync + 'static
    {
        PassBuilder::Basic(Arc::new(func))
    }
    pub fn main<F>(eb: EffectBuilder<'a>, func: F) -> Self
        where F: Fn(&mut Encoder, &Target, &Effect, &Scene, &Model) + Send + Sync + 'static
    {
        PassBuilder::Model(Arc::new(func), eb)
    }

    pub fn post<F>(eb: EffectBuilder<'a>, func: F) -> Self
        where F: Fn(&mut Encoder, &Target, &Effect, &Scene) + Send + Sync + 'static
    {
        PassBuilder::Simple(Arc::new(func), eb)
    }

    pub(crate) fn finish(self, fac: &mut Factory, t: &Targets, out: &Target) -> Result<Pass> {
        match self {
            PassBuilder::Basic(f) => Ok(Pass::Basic(BasicPass(f))),
            PassBuilder::Model(f, e) => Ok(Pass::Model(ModelPass(f, e.finish(fac, out)?))),
            PassBuilder::Simple(f, e) => Ok(Pass::Simple(SimplePass(f, e.finish(fac, out)?))),
        }
    }
}

impl<'a> Debug for PassBuilder<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            PassBuilder::Basic(_) => {
                fmt.debug_tuple("Basic")
                    .field(&"[closure]")
                    .finish()
            }
            PassBuilder::Model(_, ref e) => {
                fmt.debug_tuple("Model")
                    .field(&"[closure]")
                    .field(e)
                    .finish()
            }
            PassBuilder::Simple(_, ref e) => {
                fmt.debug_tuple("Simple")
                    .field(&"[closure]")
                    .field(e)
                    .finish()
            }
        }
    }
}
