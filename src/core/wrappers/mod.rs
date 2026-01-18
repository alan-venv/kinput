mod absolute_mouse;
mod keyboard;
mod relative_mouse;

pub use absolute_mouse::AbsoluteMouse;
pub use keyboard::Keyboard;
pub use relative_mouse::RelativeMouse;

/// Mouse controls.
///
/// Use `rel` for relative movement and `abs` for absolute positioning.
pub struct Mouse {
    /// Relative mouse (delta movement).
    pub rel: RelativeMouse,
    /// Absolute mouse (positioning).
    pub abs: AbsoluteMouse,
}
