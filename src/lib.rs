mod core;
mod types;

use crate::core::{
    AbsoluteMouse, AbsoluteMouseDevice, Keyboard, KeyboardDevice, Mouse, RelativeMouse,
    RelativeMouseDevice,
};
use std::rc::Rc;

/// Keyboard keys supported by `kinput`.
pub use crate::types::enums::Key;

/// Virtual input device with keyboard and mouse.
pub struct InputDevice {
    /// Mouse actions.
    pub mouse: Mouse,
    /// Keyboard actions.
    pub keyboard: Keyboard,
}

impl InputDevice {
    /// Creates a new `InputDevice` with a default absolute mouse area of `1920x1080`.
    pub fn new() -> Self {
        Self::from((1920, 1080))
    }
}

impl From<(i32, i32)> for InputDevice {
    /// Creates a new `InputDevice` with a custom absolute mouse area.
    fn from((width, height): (i32, i32)) -> Self {
        let keyboard_device = KeyboardDevice::new();
        let relative_mouse_device = RelativeMouseDevice::new();
        let absolute_mouse_device = AbsoluteMouseDevice::new();

        let keyboard = Keyboard::new(Rc::new(keyboard_device));
        let relative_mouse = RelativeMouse::new(Rc::new(relative_mouse_device));
        let absolute_mouse = AbsoluteMouse::new(Rc::new(absolute_mouse_device), width, height);

        Self {
            mouse: Mouse {
                rel: relative_mouse,
                abs: absolute_mouse,
            },
            keyboard,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main() {
        let device = InputDevice::new();

        for _ in 0..20 {
            device.mouse.rel.move_xy(10, 0);
        }

        device.mouse.rel.left_click();
        device.mouse.abs.reset_axis();
    }
}
