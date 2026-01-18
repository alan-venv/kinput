mod absolute_mouse;
mod keyboard;
mod relative_mouse;

pub use absolute_mouse::AbsoluteMouseDevice;
pub use keyboard::KeyboardDevice;
pub use relative_mouse::RelativeMouseDevice;

const QUEUE_CAPACITY: usize = 1024;
