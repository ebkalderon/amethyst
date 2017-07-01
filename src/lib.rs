//! Amethyst is a free and open source game engine written in idiomatic
//! [Rust][rs] for building video games and interactive multimedia applications.
//! The source code is available for download on [GitHub][gh]. See the
//! [online book][bk] for a complete guide to using Amethyst.
//!
//! [rs]: https://www.rust-lang.org/
//! [gh]: https://github.com/amethyst/amethyst
//! [bk]: https://www.amethyst.rs/book/
//!
//! This project is a work in progress and is very incomplete. Pardon the dust!
//!
//! # Example
//!
//! ```no_run
//! extern crate amethyst;
//!
//! use amethyst::prelude::*;
//!
//! struct GameState;
//!
//! impl State for GameState {
//!     fn on_start(&mut self, _: &mut Engine) {
//!         println!("Starting game!");
//!     }
//!
//!     fn handle_event(&mut self, _: &mut Engine, event: &Event) -> Trans {
//!         match event {
//!             Event::Window(e) => match e {
//!                 WindowEvent::KeyboardInput(_, _, Some(Key::Escape), _) |
//!                 WindowEvent::Closed => Trans::Quit,
//!                 _ => Trans::None,
//!             }
//!             _ => Trans::None,
//!         }
//!     }
//!
//!     fn update(&mut self, _: &mut Engine) -> Trans {
//!         println!("Computing some more whoop-ass...");
//!     }
//! }
//!
//! fn main() {
//!     let mut game = Application::new(GameState).expect("Fatal error");
//!     game.run();
//! }
//! ```

#![deny(missing_docs)]
#![doc(html_logo_url = "https://tinyurl.com/jtmm43a")]

pub extern crate amethyst_renderer as renderer;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_yaml;
extern crate toml;

extern crate cgmath;
extern crate dds;
#[macro_use]
extern crate derivative;
extern crate fnv;
extern crate genmesh;
extern crate gfx;
extern crate imagefmt;
extern crate num_cpus;
extern crate rayon;
extern crate specs;
extern crate wavefront_obj;
extern crate winit;

#[cfg(feature="profiler")]
#[macro_use]
extern crate thread_profiler;

pub use self::app::{Application, ApplicationBuilder, Engine};
pub use self::error::{Error, Result};
pub use self::state::{State, StateMachine, Trans};

pub mod assets;
pub mod ecs;
pub mod event;
pub mod prelude;
#[macro_use]
pub mod project;
pub mod timing;

mod app;
mod state;
mod error;
