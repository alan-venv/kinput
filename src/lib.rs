mod core;
mod types;

use crate::core::{AbsoluteMouse, Device, DeviceType, Keyboard, RelativeMouse};
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

pub struct Mouse {
    pub rel: RelativeMouse,
    pub abs: AbsoluteMouse,
}

impl InputDevice {
    /// Creates a new `InputDevice`.
    pub fn new() -> Self {
        let keyboard_device = Device::new(DeviceType::Keyboard);
        let relative_mouse_device = Device::new(DeviceType::RelativeMouse);
        let absolute_mouse_device = Device::new(DeviceType::AbsoluteMouse);

        let keyboard = Keyboard::new(Rc::new(keyboard_device));
        let relative_mouse = RelativeMouse::new(Rc::new(relative_mouse_device));
        let absolute_mouse = AbsoluteMouse::new(Rc::new(absolute_mouse_device));

        Self {
            mouse: Mouse {
                rel: relative_mouse,
                abs: absolute_mouse,
            },
            keyboard: keyboard,
        }
    }
}
