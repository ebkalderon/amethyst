//! Configuration parsing and storage.

use std::path::Path;

/// Stores engine configuration data.
pub struct Configs {
    /// Resolution, vsync, window title options.
    display: Display,
    /// Input bindings for keyboards, gamepads, and touch screens.
    input: Input,
    /// Developer options for logging verbosity.
    logging: Logging,
}

impl Configs {
    /// Loads configuration data from given YAML strings.
    pub fn parse() -> Result<Configs, &'static str> {
        // Parse the strings, build config fields.
        
        let cfg = Configs {
            display: Display(1.0, false, [1024, 768], "Amethyst".to_string()),
            input: Input,
            logging: Logging("log.log".to_string(), Verbosity::Debug, Verbosity::Debug),
        };

        Ok(cfg)
    }
}

/// Display configuration data.
/// Format: (brightness, fullscreen, [width, height], title)
struct Display(f32, bool, [i32; 2], String);

/// Input configuration data.
/// TODO: Missing fields; no key/gamepad/touch input representation defined yet.
struct Input;

/// Logging configuration data.
/// Format: (log file path, stdout verbosity, log file verbosity)
struct Logging(String, Verbosity, Verbosity);

enum Verbosity {
    None,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
