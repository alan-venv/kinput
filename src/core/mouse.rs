use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use crate::core::device::Device;
use crate::types::constants::{BTN_LEFT, BTN_RIGHT};

/// Relative mouse for movement and clicks.
pub struct RelativeMouse {
    device: Rc<Device>,
}

impl RelativeMouse {
    /// Creates a `RelativeMouse`.
    pub fn new(device: Rc<Device>) -> Self {
        Self { device }
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

    /// Moves the cursor to the top-left corner.
    pub fn reset_axis(&self) {
        self.device.move_relative(-10000, -10000);
        sleep(Duration::from_millis(30));
    }

    /// Moves the mouse by a relative delta.
    pub fn move_xy(&self, x: i32, y: i32) {
        self.device.move_relative(x, y);
        sleep(Duration::from_millis(30));
    }
}

/// Absolute mouse for movement and clicks.
pub struct AbsoluteMouse {
    device: Rc<Device>,
}

impl AbsoluteMouse {
    /// Creates an `AbsoluteMouse`.
    pub fn new(device: Rc<Device>) -> Self {
        Self { device }
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

    /// Moves the cursor to (0, 0).
    pub fn reset_axis(&self) {
        self.device.move_absolute(0, 0);
        sleep(Duration::from_millis(30));
    }

    /// Moves the mouse to an absolute position.
    pub fn move_xy(&self, x: i32, y: i32) {
        self.device.move_absolute(x, y);
        sleep(Duration::from_millis(30));
    }
}
