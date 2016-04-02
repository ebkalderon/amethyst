//! Resource management.

use std::path::PathBuf;
use walkdir::WalkDir;

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
    pub fn load_disk(root: PathBuf) -> Result<Resources, &'static str> {
        if !root.as_path().exists() {
            return Err("Resources path is inaccessible or nonexistent!");
        } else if !root.as_path().is_dir() {
            return Err("Resources path is not a directory!");
        }

        let r = Resources {
            root: Some(root.clone()),
            entities: Vec::new(),
            configs: Configs::init(&root),
        };

        Ok(r)
    }

    /// Load from a `resources.zip` file placed in the current directory.
    ///
    /// TODO: Should we support loading `resources.zip` from any directory? Is
    /// such a feature necessary?
    pub fn load_zip() -> Result<Resources, &'static str> {
        use std::env::current_dir;

        let zip = current_dir().unwrap().join("resources.zip");
        if !zip.exists() || !zip.is_file() {
            return Err("File `resources.zip` not found in current directory!");
        }

        let r = Resources {
            root: None,
            entities: Vec::new(),
            configs: Configs::init(&zip),
        };

        Ok(r)
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
    pub fn init(root: &PathBuf) -> Configs {
        if !root.join("config.yml").exists() {
            panic!("`config.yml` not found in directory!");
        }

        // Load `config.yml` here, build paths for each config field.
        let d = root.join("display.yml");
        let i = root.join("input.yml");
        let l = root.join("logging.yml");

        Configs {
            display: d,
            input: i,
            logging: l,
        }
    }
}
