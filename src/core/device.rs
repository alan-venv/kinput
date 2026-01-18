use crate::core::uinput::{
    open_uinput, setup_absolute_mouse, setup_keyboard, setup_relative_mouse, wait_device_ready,
};
use crate::core::worker::{
    AbsoluteMouseAction, AbsoluteMouseMsg, AbsoluteMouseWorker, KeyboardAction, KeyboardMsg,
    KeyboardWorker, RelativeMouseAction, RelativeMouseMsg, RelativeMouseWorker,
};

use std::sync::mpsc::{SyncSender, sync_channel};
use std::thread::{self, JoinHandle};

const QUEUE_CAPACITY: usize = 1024;

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
