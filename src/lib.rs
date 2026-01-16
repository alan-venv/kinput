mod core;
mod types;

use crate::core::{Device, Keyboard, Mouse};
use std::rc::Rc;

/// Keyboard keys supported by `kinput`.
pub use crate::types::enums::Key;

/// Virtual input device with keyboard and mouse.
pub struct InputDevice {
    /// Mouse actions (move/click).
    pub mouse: Mouse,
    /// Keyboard actions (press/type).
    pub keyboard: Keyboard,
}

impl InputDevice {
    /// Creates a new `InputDevice`.
    pub fn new() -> Self {
        let device = Rc::new(Device::new());
        Self {
            mouse: Mouse::new(Rc::clone(&device)),
            keyboard: Keyboard::new(Rc::clone(&device)),
        }
    }
}
