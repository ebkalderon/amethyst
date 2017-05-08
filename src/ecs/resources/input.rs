//! World resource that handles all user input.

use engine::{ElementState, Key, Event, WindowEvent};
use fnv::FnvHashMap as HashMap;
use std::collections::hash_map::{Entry, Keys};
use std::iter::Iterator;

/// Indicates whether a given `VirtualKeyCode` has been queried or not.
#[derive(Debug, Eq, Hash, PartialEq)]
enum KeyQueryState {
    NotQueried,
    Queried,
}

/// An iterator over all currently pressed down keys.
#[derive(Debug)]
pub struct PressedKeysIter<'a> {
    iter: Keys<'a, Key, KeyQueryState>,
}

impl<'a> Iterator for PressedKeysIter<'a> {
    type Item = &'a Key;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// Processes user input events.
#[derive(Debug, Default)]
pub struct InputHandler {
    pressed_keys: HashMap<Key, KeyQueryState>,
}

impl InputHandler {
    /// Creates a new input handler.
    pub fn new() -> InputHandler {
        InputHandler { pressed_keys: HashMap::default() }
    }

    /// Updates the input handler with new engine events.
    pub fn update(&mut self, event: Event) {
        use ElementState::{Pressed, Released};

        if let Event::WindowEvent(e) = event {
            match e {
                Event::KeyboardInput(Pressed, _, Some(key_code), _) => {
                    match self.pressed_keys.entry(key_code) {
                        Entry::Occupied(_) => {
                            // nop
                            // Allows more accurate `key_once` calls,
                            // I.e `key_once(key)` is queried after
                            // second `Pressed` event.
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(KeyQueryState::NotQueried);
                        }
                    }
                },
                Event::KeyboardInput(Released, _, Some(key_code), _) => {
                    self.pressed_keys.remove(&key_code);
                },
                Event::Focused(false) => self.pressed_keys.clear(),
                _ => (),
            }
        }
    }

    /// Returns an iterator for all the pressed down keys
    pub fn pressed_keys(&self) -> PressedKeysIter {
        PressedKeysIter { iter: self.pressed_keys.keys() }
    }

    /// Checks if the given key is being pressed.
    pub fn key_down(&self, key_code: Key) -> bool {
        self.pressed_keys.contains_key(&key_code)
    }

    /// Checks if all the given keys are being pressed.
    pub fn keys_down(&self, keys: &[Key]) -> bool {
        keys.iter().all(|k| self.key_down(*k))
    }

    /// Checks if the given key is being pressed and held down.
    ///
    /// If `key` hasn't been let go since the last `key_once()` query, this
    /// function will return false.
    pub fn key_once(&mut self, key_code: Key) -> bool {
        if !self.pressed_keys.contains_key(&key_code) {
            return false;
        }

        if let Some(value) = self.pressed_keys.get_mut(&key_code) {
            // Should be safe
            if *value == KeyQueryState::NotQueried {
                *value = KeyQueryState::Queried;
                return true;
            }
        }

        false
    }

    /// Checks if the all the given keys are being pressed and held down.
    ///
    /// If the `keys` haven't been let go since the last `key_once()` query,
    /// this function will return false.
    pub fn keys_once(&mut self, keys: &[Key]) -> bool {
        keys.iter().any(|k| self.key_once(*k)) && self.keys_down(keys)
    }
}
