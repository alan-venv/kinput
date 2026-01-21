use libc::input_event;

#[allow(dead_code)]
pub struct Device {
    pub fd: i32,
    pub path: [u8; 256],
    pub name: [u8; 256],
    pub pending_dx: i32,
    pub pending_dy: i32,
    pub pending_wheel: i32,
}

impl Default for Device {
    fn default() -> Self {
        Self {
            fd: -1,
            path: [0; 256],
            name: [0; 256],
            pending_dx: 0,
            pending_dy: 0,
            pending_wheel: 0,
        }
    }
}

pub struct EventBatch {
    pub events: [input_event; 64],
    pub count: usize,
}

impl Default for EventBatch {
    fn default() -> Self {
        // input_event is plain old data; zeroed is fine for buffer storage.
        Self {
            events: unsafe { core::mem::zeroed() },
            count: 0,
        }
    }
}

impl EventBatch {
    pub fn as_slice(&self) -> &[input_event] {
        let end = self.count.min(self.events.len());
        &self.events[..end]
    }
}
