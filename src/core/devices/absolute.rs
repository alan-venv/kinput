use crate::core::devices::QUEUE_CAPACITY;
use crate::core::uinput::{open_uinput, setup_absolute_mouse, wait_device_ready};
use crate::core::workers::{AbsoluteMouseAction, AbsoluteMouseMsg, AbsoluteMouseWorker};

use std::sync::mpsc::{SyncSender, sync_channel};
use std::thread::{self, JoinHandle};

pub struct AbsoluteMouseDevice {
    tx: Option<SyncSender<AbsoluteMouseMsg>>,
    worker: Option<JoinHandle<()>>,
}

impl AbsoluteMouseDevice {
    pub fn new() -> Self {
        let fd = open_uinput();
        setup_absolute_mouse(fd);
        wait_device_ready();

        let (tx, rx) = sync_channel::<AbsoluteMouseMsg>(QUEUE_CAPACITY);
        let worker = Some(thread::spawn(move || AbsoluteMouseWorker::run(fd, rx)));

        Self {
            tx: Some(tx),
            worker,
        }
    }

    pub fn move_absolute(&self, x: i32, y: i32) {
        self.tx
            .as_ref()
            .expect("absolute mouse sender missing")
            .send(AbsoluteMouseMsg::Action(AbsoluteMouseAction::Move(x, y)))
            .expect("absolute mouse worker stopped");
    }

    pub fn press(&self, btn: u16) {
        self.tx
            .as_ref()
            .expect("absolute mouse sender missing")
            .send(AbsoluteMouseMsg::Action(AbsoluteMouseAction::Press(btn)))
            .expect("absolute mouse worker stopped");
    }

    pub fn release(&self, btn: u16) {
        self.tx
            .as_ref()
            .expect("absolute mouse sender missing")
            .send(AbsoluteMouseMsg::Action(AbsoluteMouseAction::Release(btn)))
            .expect("absolute mouse worker stopped");
    }
}

impl Drop for AbsoluteMouseDevice {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(AbsoluteMouseMsg::Shutdown);
        }
        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
    }
}
