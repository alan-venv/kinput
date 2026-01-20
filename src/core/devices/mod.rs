mod absolute;
mod keyboard;
mod relative;

pub use absolute::AbsoluteMouseDevice;
pub use keyboard::KeyboardDevice;
pub use relative::RelativeMouseDevice;

const QUEUE_CAPACITY: usize = 1024;
