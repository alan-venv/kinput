use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use crate::core::KeyboardDevice;
use crate::types::enums::{Key, Layout};

/// Keyboard for sending key events.
pub struct Keyboard {
    device: Rc<KeyboardDevice>,
    layout: Layout,
}

impl Keyboard {
    /// Creates a `Keyboard`.
    pub fn new(device: Rc<KeyboardDevice>, layout: Layout) -> Keyboard {
        Keyboard { device, layout }
    }

    /// Presses a key.
    pub fn press(&self, key: Key) {
        self.device.press(key.value());
    }

    /// Releases a key.
    pub fn release(&self, key: Key) {
        self.device.release(key.value());
    }

    /// Presses and releases a key.
    pub fn click(&self, key: Key) {
        self.press(key);
        self.release(key);
    }

    /// Types the provided content
    pub fn text(&self, content: &str) {
        for char in content.chars() {
            self.layout.map(char, &mut |keys| {
                self.chord(keys);
            });
        }
    }

    fn chord(&self, keys: &[Key]) {
        keys.iter().for_each(|&k| {
            self.press(k);
            sleep(Duration::from_millis(5));
        });
        keys.iter().rev().for_each(|&k| {
            self.release(k);
            sleep(Duration::from_millis(5));
        });
    }
}
