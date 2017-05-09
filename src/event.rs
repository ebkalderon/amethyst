//! This module contains the `WindowEvent` type and re-exports glutin event
//! types.

pub use winit::{ElementState, ModifiersState, MouseButton, MouseScrollDelta,
                ScanCode, Touch, TouchPhase, VirtualKeyCode as Key, WindowEvent};

use std::sync::mpsc;
use winit::Event as WinitEvent;

/// Receiver half of an event queue channel.
pub type EventReceiver = mpsc::Receiver<Event>;

/// Sender half of an event queue channel.
pub type EventSender = mpsc::Sender<Event>;

/// Generic engine event.
#[derive(Debug)]
pub enum Event {
    /// An asset event.
    Asset(String),
    /// A window event.
    Window(WindowEvent),
    /// User-defined event.
    User(String),
}

impl From<WinitEvent> for Event {
    fn from(e: WinitEvent) -> Event {
        let WinitEvent::WindowEvent { event, .. } = e;
        Event::Window(event)
    }
}

impl From<WindowEvent> for Event {
    fn from(e: WindowEvent) -> Event {
        Event::Window(e)
    }
}
