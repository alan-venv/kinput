use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use crate::core::device::Device;
use crate::types::constants::{BTN_LEFT, BTN_RIGHT};

/// Mouse controls.
///
/// Use `rel` for relative movement and `abs` for absolute positioning.
pub struct Mouse {
    /// Relative mouse (delta movement).
    pub rel: RelativeMouse,
    /// Absolute mouse (positioning).
    pub abs: AbsoluteMouse,
}

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
    }
}

/// Absolute mouse for movement and clicks.
pub struct AbsoluteMouse {
    device: Rc<Device>,
    width: i32,
    height: i32,
}

impl AbsoluteMouse {
    /// Creates an `AbsoluteMouse`.
    pub fn new(device: Rc<Device>, width: i32, height: i32) -> Self {
        Self {
            device,
            width,
            height,
        }
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
        self.device.move_absolute(self.abs_x(x), self.abs_y(y));
        sleep(Duration::from_millis(30));
    }

    fn abs_x(&self, pixel: i32) -> i32 {
        Self::abs_from_px(pixel, self.width)
    }

    fn abs_y(&self, pixel: i32) -> i32 {
        Self::abs_from_px(pixel, self.height)
    }

    fn abs_from_px(mut px: i32, size_px: i32) -> i32 {
        if size_px <= 1 {
            return 0;
        }
        if px < 0 {
            px = 0;
        } else if px > size_px - 1 {
            px = size_px - 1;
        }
        let px = px as i64;
        let size_px = size_px as i64;
        ((px * 65535 + (size_px - 2) / 2) / (size_px - 1)) as i32
    }
}
