mod absolute;
mod keyboard;
mod relative;

pub use absolute::AbsoluteMouse;
pub use keyboard::Keyboard;
pub use relative::RelativeMouse;

/// Mouse controls.
///
/// Use `rel` for relative movement and `abs` for absolute positioning.
pub struct Mouse {
    /// Relative mouse (delta movement).
    pub rel: RelativeMouse,
    /// Absolute mouse (positioning).
    pub abs: AbsoluteMouse,
}
