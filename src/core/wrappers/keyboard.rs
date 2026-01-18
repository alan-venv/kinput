use std::rc::Rc;

use crate::core::KeyboardDevice;
use crate::types::enums::Key;

/// Keyboard for sending key events.
pub struct Keyboard {
    device: Rc<KeyboardDevice>,
}

impl Keyboard {
    /// Creates a `Keyboard`.
    pub fn new(device: Rc<KeyboardDevice>) -> Keyboard {
        Keyboard { device }
    }

    /// Types a sequence of keys.
    pub fn text<T: IntoIterator<Item = Key>>(&self, keys: T) {
        for key in keys {
            self.click(key);
        }
    }

    /// Presses and releases a key.
    pub fn click(&self, key: Key) {
        self.press(key);
        self.release(key);
    }

    /// Presses a key.
    pub fn press(&self, key: Key) {
        self.device.press(key.value());
    }

    /// Releases a key.
    pub fn release(&self, key: Key) {
        self.device.release(key.value());
    }
}
