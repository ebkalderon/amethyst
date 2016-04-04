#![crate_name = "amethyst_engine"]
#![crate_type = "lib"]
#![doc(html_logo_url = "http://tinyurl.com/hgsb45k")]

//! Game engine sitting atop the core libraries.

extern crate time;
extern crate walkdir;
extern crate yaml_rust;

mod app;
mod cfg;
mod res;
mod state;
mod timing;

pub use self::app::Application;
pub use self::state::{State, StateMachine, Trans};
pub use self::timing::{Duration, SteadyTime, Stopwatch};

mod input;
pub use self::input::{InputBinds};
