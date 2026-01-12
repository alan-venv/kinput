use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use crate::core::device::Device;
use crate::types::enums::Key;

pub struct Keyboard {
    device: Rc<Device>,
}

impl Keyboard {
    pub fn new(device: Rc<Device>) -> Keyboard {
        return Keyboard { device: device };
    }

    pub fn text<T: IntoIterator<Item = Key>>(&self, keys: T) {
        for key in keys {
            self.click(key);
        }
    }

    pub fn click(&self, key: Key) {
        self.press(key);
        sleep(Duration::from_millis(5));
        self.release(key);
        sleep(Duration::from_millis(10));
    }

    pub fn press(&self, key: Key) {
        self.device.press(key.value());
    }

    pub fn release(&self, key: Key) {
        self.device.release(key.value());
    }
}
