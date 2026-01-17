mod core;
mod types;

use crate::core::{AbsoluteMouse, Device, DeviceType, Keyboard, Mouse, RelativeMouse};
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
        let keyboard_device = Device::new(DeviceType::Keyboard);
        let relative_mouse_device = Device::new(DeviceType::RelativeMouse);
        let absolute_mouse_device = Device::new(DeviceType::AbsoluteMouse);

        let keyboard = Keyboard::new(Rc::new(keyboard_device));
        let relative_mouse = RelativeMouse::new(Rc::new(relative_mouse_device));
        let absolute_mouse = AbsoluteMouse::new(Rc::new(absolute_mouse_device), width, height);

        Self {
            mouse: Mouse {
                rel: relative_mouse,
                abs: absolute_mouse,
            },
            keyboard: keyboard,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main() {
        let device = InputDevice::new();

        device.keyboard.press(Key::LeftShift);
        device.keyboard.press(Key::Num1);
        device.keyboard.release(Key::Num1);
        device.keyboard.release(Key::LeftShift);
    }
}
