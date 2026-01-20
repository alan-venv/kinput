use crate::core::workers::{ACTION_DELAY, emit};
use crate::types::constants::{ABS_X, ABS_Y, EV_ABS, EV_KEY, EV_SYN, SYN_REPORT};

use nix::ioctl_none;
use std::os::unix::io::RawFd;
use std::sync::mpsc::Receiver;
use std::thread::sleep;

ioctl_none!(ui_dev_destroy, b'U', 2);

#[derive(Debug, Copy, Clone)]
pub enum AbsoluteMouseAction {
    Move(i32, i32),
    Press(u16),
    Release(u16),
}

pub enum AbsoluteMouseMsg {
    Action(AbsoluteMouseAction),
    Shutdown,
}

pub struct AbsoluteMouseWorker {
    fd: RawFd,
    rx: Receiver<AbsoluteMouseMsg>,
}

impl AbsoluteMouseWorker {
    pub fn run(fd: RawFd, rx: Receiver<AbsoluteMouseMsg>) {
        let worker = Self { fd, rx };
        worker.event_loop();
    }

    fn event_loop(self) {
        while let Ok(msg) = self.rx.recv() {
            match msg {
                AbsoluteMouseMsg::Action(action) => {
                    match action {
                        AbsoluteMouseAction::Move(x, y) => {
                            emit(self.fd, EV_ABS, ABS_X, x);
                            emit(self.fd, EV_ABS, ABS_Y, y);
                            emit(self.fd, EV_SYN, SYN_REPORT, 0);
                        }
                        AbsoluteMouseAction::Press(btn) => {
                            emit(self.fd, EV_KEY, btn, 1);
                            emit(self.fd, EV_SYN, SYN_REPORT, 0);
                        }
                        AbsoluteMouseAction::Release(btn) => {
                            emit(self.fd, EV_KEY, btn, 0);
                            emit(self.fd, EV_SYN, SYN_REPORT, 0);
                        }
                    }

                    sleep(ACTION_DELAY);
                }
                AbsoluteMouseMsg::Shutdown => break,
            }
        }

        unsafe {
            let _ = ui_dev_destroy(self.fd);
            let _ = libc::close(self.fd);
        }
    }
}
