use crate::core::workers::{ACTION_DELAY, emit};
use crate::types::constants::{EV_KEY, EV_SYN, SYN_REPORT};

use nix::ioctl_none;
use std::os::unix::io::RawFd;
use std::sync::mpsc::Receiver;
use std::thread::sleep;

ioctl_none!(ui_dev_destroy, b'U', 2);

#[derive(Debug, Copy, Clone)]
pub enum KeyboardAction {
    Press(u16),
    Release(u16),
}

pub enum KeyboardMsg {
    Action(KeyboardAction),
    Shutdown,
}

pub struct KeyboardWorker {
    fd: RawFd,
    rx: Receiver<KeyboardMsg>,
}

impl KeyboardWorker {
    pub fn run(fd: RawFd, rx: Receiver<KeyboardMsg>) {
        let worker = Self { fd, rx };
        worker.event_loop();
    }

    fn event_loop(self) {
        while let Ok(msg) = self.rx.recv() {
            match msg {
                KeyboardMsg::Action(action) => {
                    match action {
                        KeyboardAction::Press(key) => {
                            emit(self.fd, EV_KEY, key, 1);
                            emit(self.fd, EV_SYN, SYN_REPORT, 0);
                        }
                        KeyboardAction::Release(key) => {
                            emit(self.fd, EV_KEY, key, 0);
                            emit(self.fd, EV_SYN, SYN_REPORT, 0);
                        }
                    }

                    sleep(ACTION_DELAY);
                }
                KeyboardMsg::Shutdown => break,
            }
        }

        unsafe {
            let _ = ui_dev_destroy(self.fd);
            let _ = libc::close(self.fd);
        }
    }
}
