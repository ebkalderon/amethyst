//! Resource management.

use std::path::PathBuf;
use walkdir::WalkDir;

use cfg::Configs;

/// Logical representation of the game's resource files.
pub struct Resources {
    /// Path to local `resources` folder.
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

        let cfg = try!(Configs::parse());

        let res = Resources {
            root: Some(root.clone()),
            entities: Vec::new(),
            configs: cfg,
        };

        Ok(res)
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

        let cfg = try!(Configs::parse());

        let res = Resources {
            root: None,
            entities: Vec::new(),
            configs: cfg,
        };

        Ok(res)
    }

    /// Signal the engine to close all open resources.
    pub fn close(&mut self) {
        unimplemented!();
    }
}
