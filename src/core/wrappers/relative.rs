use std::rc::Rc;

use crate::core::RelativeMouseDevice;
use crate::types::constants::{BTN_LEFT, BTN_MIDDLE, BTN_RIGHT};

/// Relative mouse for movement and clicks.
pub struct RelativeMouse {
    device: Rc<RelativeMouseDevice>,
}

impl RelativeMouse {
    /// Creates a `RelativeMouse`.
    pub fn new(device: Rc<RelativeMouseDevice>) -> Self {
        Self { device }
    }

    /// Left click.
    pub fn left_click(&self) {
        self.device.press(BTN_LEFT);
        self.device.release(BTN_LEFT);
    }

    /// Right click.
    pub fn right_click(&self) {
        self.device.press(BTN_RIGHT);
        self.device.release(BTN_RIGHT);
    }

    /// Middle click.
    pub fn middle_click(&self) {
        self.device.press(BTN_MIDDLE);
        self.device.release(BTN_MIDDLE);
    }

    /// Moves the cursor to the top-left corner.
    pub fn reset_axis(&self) {
        self.device.move_relative(-10000, -10000);
    }

    /// Moves the mouse by a relative delta.
    pub fn move_xy(&self, x: i32, y: i32) {
        self.device.move_relative(x, y);
    }
}
