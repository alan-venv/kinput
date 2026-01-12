mod core;
mod types;

use crate::core::{Device, Keyboard, Mouse};
use std::rc::Rc;

pub use crate::types::enums::Key;

pub struct InputDevice {
    pub mouse: Mouse,
    pub keyboard: Keyboard,
}

impl InputDevice {
    pub fn new() -> Self {
        let device = Rc::new(Device::new());
        Self {
            mouse: Mouse::new(Rc::clone(&device)),
            keyboard: Keyboard::new(Rc::clone(&device)),
        }
    }
}
