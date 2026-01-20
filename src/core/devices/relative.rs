use crate::core::devices::QUEUE_CAPACITY;
use crate::core::uinput::{open_uinput, setup_relative_mouse, wait_device_ready};
use crate::core::workers::{RelativeMouseAction, RelativeMouseMsg, RelativeMouseWorker};

use std::sync::mpsc::{SyncSender, sync_channel};
use std::thread::{self, JoinHandle};

pub struct RelativeMouseDevice {
    tx: Option<SyncSender<RelativeMouseMsg>>,
    worker: Option<JoinHandle<()>>,
}

impl RelativeMouseDevice {
    pub fn new() -> Self {
        let fd = open_uinput();
        setup_relative_mouse(fd);
        wait_device_ready();

        let (tx, rx) = sync_channel::<RelativeMouseMsg>(QUEUE_CAPACITY);
        let worker = Some(thread::spawn(move || RelativeMouseWorker::run(fd, rx)));

        Self {
            tx: Some(tx),
            worker,
        }
    }

    pub fn move_relative(&self, dx: i32, dy: i32) {
        self.tx
            .as_ref()
            .expect("relative mouse sender missing")
            .send(RelativeMouseMsg::Action(RelativeMouseAction::Move(dx, dy)))
            .expect("relative mouse worker stopped");
    }

    pub fn press(&self, btn: u16) {
        self.tx
            .as_ref()
            .expect("relative mouse sender missing")
            .send(RelativeMouseMsg::Action(RelativeMouseAction::Press(btn)))
            .expect("relative mouse worker stopped");
    }

    pub fn release(&self, btn: u16) {
        self.tx
            .as_ref()
            .expect("relative mouse sender missing")
            .send(RelativeMouseMsg::Action(RelativeMouseAction::Release(btn)))
            .expect("relative mouse worker stopped");
    }
}

impl Drop for RelativeMouseDevice {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(RelativeMouseMsg::Shutdown);
        }
        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
    }
}
