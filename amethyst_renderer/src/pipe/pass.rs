//! Types for constructing render passes.

#![allow(missing_docs)]

use error::Result;
use pipe::{Effect, EffectBuilder, Target, Targets};
use scene::{Model, Scene};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::Arc;
use types::{Encoder, Factory};

pub type FunctionFn = Arc<Fn(&mut Encoder, &Target) + Send + Sync>;
pub type MainFn = Arc<Fn(&mut Encoder, &Target, &Model, &Effect, &Scene) + Send + Sync>;
pub type PostFn = Arc<Fn(&mut Encoder, &Target, &Effect, &Scene) + Send + Sync>;

/// Discrete rendering pass.
#[derive(Clone)]
pub enum Pass {
    Function(FunctionFn),
    Main(MainFn, Effect),
    Post(PostFn, Effect),
}

impl Pass {
    /// Applies the rendering pass using the given `Encoder` and `Target`.
    pub fn apply(&self, enc: &mut Encoder, model: &Model, scene: &Scene, out: &Target) {
        match *self {
            Pass::Function(ref func) => func(enc, out),
            Pass::Main(ref func, ref e) => func(enc, out, model, e, scene),
            Pass::Post(ref func, ref e) => func(enc, out, e, scene),
        }
    }
}

impl Debug for Pass {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            Pass::Function(_) => {
                fmt.debug_tuple("Function")
                    .field(&"[closure]")
                    .finish()
            }
            Pass::Main(_, ref e) => {
                fmt.debug_tuple("Main")
                    .field(&"[closure]")
                    .field(e)
                    .finish()
            }
            Pass::Post(_, ref e) => {
                fmt.debug_tuple("Post")
                    .field(&"[closure]")
                    .field(e)
                    .finish()
            }
        }
    }
}

#[derive(Clone)]
pub enum PassBuilder<'a> {
    Function(FunctionFn),
    Main(MainFn, EffectBuilder<'a>),
    Post(PostFn, EffectBuilder<'a>),
}

impl<'a> PassBuilder<'a> {
    pub fn function<F>(func: F) -> Self
        where F: Fn(&mut Encoder, &Target) + Send + Sync + 'static
    {
        PassBuilder::Function(Arc::new(func))
    }
    pub fn main<F>(eb: EffectBuilder<'a>, func: F) -> Self
        where F: Fn(&mut Encoder, &Target, &Model, &Effect, &Scene) + Send + Sync + 'static
    {
        PassBuilder::Main(Arc::new(func), eb)
    }

    pub fn postproc<F>(eb: EffectBuilder<'a>, func: F) -> Self
        where F: Fn(&mut Encoder, &Target, &Effect, &Scene) + Send + Sync + 'static
    {
        PassBuilder::Post(Arc::new(func), eb)
    }

    pub(crate) fn finish(self, fac: &mut Factory, t: &Targets, out: &Target) -> Result<Pass> {
        match self {
            PassBuilder::Function(f) => Ok(Pass::Function(f)),
            PassBuilder::Main(f, e) => Ok(Pass::Main(f, e.finish(fac, out)?)),
            PassBuilder::Post(f, e) => Ok(Pass::Post(f, e.finish(fac, out)?)),
        }
    }
}

impl<'a> Debug for PassBuilder<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            PassBuilder::Function(_) => {
                fmt.debug_tuple("Function")
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
