//! Resource management.

use std::path::PathBuf;

/// Logical representation of the game's resource files.
pub struct Resources {
    /// Path to local `resources` folder.
    ///
    /// If `Some`, load from disk relative to that path. If `None`, load from
    /// `resources.zip` file found relative to `std::env::current_dir()`.
    root: Option<PathBuf>,
    /// Relative paths to entity YAML files.
    entities: Vec<PathBuf>,
    /// Configuration file data.
    configs: Configs,
}

impl Resources {
    /// Load files relative to the path provided.
    pub fn load_disk(root: PathBuf) -> Result<Resources, String> {
                
    }

    /// Load from a `resources.zip` file placed in the current directory.
    pub fn load_zip() -> Result<Resources, String> {

    }

    /// Signal the engine to close all open resources.
    pub fn close(&mut self) {
        unimplemented!();
    }
}

/// Contains relative paths to files where the corresponding fields (e.g.
/// `display`, `input`, `logging`, etc) can be found.
pub struct Configs {
    /// Resolution, vsync, window title options.
    display: PathBuf,
    /// Input bindings for keyboards, gamepads, and touch screens.
    input: PathBuf,
    /// Developer options for logging verbosity.
    logging: PathBuf,
}

impl Configs {
    
}
