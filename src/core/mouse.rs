use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use crate::core::device::Device;
use crate::types::constants::{BTN_LEFT, BTN_RIGHT};

/// Mouse for movement and clicks.
pub struct Mouse {
    device: Rc<Device>,
}

impl Mouse {
    /// Creates a `Mouse`.
    pub fn new(device: Rc<Device>) -> Mouse {
        return Mouse { device: device };
    }

    /// Left click.
    pub fn left_click(&self) {
        self.device.press(BTN_LEFT);
        sleep(Duration::from_millis(10));
        self.device.release(BTN_LEFT);
        sleep(Duration::from_millis(30));
    }

    /// Right click.
    pub fn right_click(&self) {
        self.device.press(BTN_RIGHT);
        sleep(Duration::from_millis(10));
        self.device.release(BTN_RIGHT);
        sleep(Duration::from_millis(30));
    }

    /// Moves cursor to top-left-ish.
    pub fn reset_axis(&self) {
        self.device.move_relative(-10000, -10000);
    }

    /// Moves the mouse by a relative delta.
    pub fn move_relative(&self, x: i32, y: i32) {
        self.device.move_relative(x, y);
    }

    /// Moves the mouse by a absolute delta.
    pub fn move_absolute(&self, x: i32, y: i32) {
        self.device.move_absolute(x, y);
    }
}
