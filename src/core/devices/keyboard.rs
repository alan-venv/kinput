use crate::core::devices::QUEUE_CAPACITY;
use crate::core::uinput::{open_uinput, setup_keyboard, wait_device_ready};
use crate::core::workers::{KeyboardAction, KeyboardMsg, KeyboardWorker};

use std::sync::mpsc::{SyncSender, sync_channel};
use std::thread::{self, JoinHandle};

pub struct KeyboardDevice {
    tx: Option<SyncSender<KeyboardMsg>>,
    worker: Option<JoinHandle<()>>,
}

impl KeyboardDevice {
    pub fn new() -> Self {
        let fd = open_uinput();
        setup_keyboard(fd);
        wait_device_ready();

        let (tx, rx) = sync_channel::<KeyboardMsg>(QUEUE_CAPACITY);
        let worker = Some(thread::spawn(move || KeyboardWorker::run(fd, rx)));

        Self {
            tx: Some(tx),
            worker,
        }
    }

    pub fn press(&self, key: u16) {
        self.tx
            .as_ref()
            .expect("keyboard sender missing")
            .send(KeyboardMsg::Action(KeyboardAction::Press(key)))
            .expect("keyboard worker stopped");
    }

    pub fn release(&self, key: u16) {
        self.tx
            .as_ref()
            .expect("keyboard sender missing")
            .send(KeyboardMsg::Action(KeyboardAction::Release(key)))
            .expect("keyboard worker stopped");
    }
}

impl Drop for KeyboardDevice {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(KeyboardMsg::Shutdown);
        }
        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
    }
}
