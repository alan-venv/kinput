use crate::core::workers::{ACTION_DELAY, emit};
use crate::types::constants::{EV_KEY, EV_REL, EV_SYN, REL_X, REL_Y, SYN_REPORT};

use nix::ioctl_none;
use std::os::unix::io::RawFd;
use std::sync::mpsc::Receiver;
use std::thread::sleep;

ioctl_none!(ui_dev_destroy, b'U', 2);

#[derive(Debug, Copy, Clone)]
pub enum RelativeMouseAction {
    Move(i32, i32),
    Press(u16),
    Release(u16),
}

pub enum RelativeMouseMsg {
    Action(RelativeMouseAction),
    Shutdown,
}

pub struct RelativeMouseWorker {
    fd: RawFd,
    rx: Receiver<RelativeMouseMsg>,
}

impl RelativeMouseWorker {
    pub fn run(fd: RawFd, rx: Receiver<RelativeMouseMsg>) {
        let worker = Self { fd, rx };
        worker.event_loop();
    }

    fn event_loop(self) {
        // todo: implement coalescing for move events.
        while let Ok(msg) = self.rx.recv() {
            match msg {
                RelativeMouseMsg::Action(action) => {
                    match action {
                        RelativeMouseAction::Move(dx, dy) => {
                            if dx != 0 {
                                emit(self.fd, EV_REL, REL_X, dx);
                            }
                            if dy != 0 {
                                emit(self.fd, EV_REL, REL_Y, dy);
                            }
                            emit(self.fd, EV_SYN, SYN_REPORT, 0);
                        }
                        RelativeMouseAction::Press(btn) => {
                            emit(self.fd, EV_KEY, btn, 1);
                            emit(self.fd, EV_SYN, SYN_REPORT, 0);
                        }
                        RelativeMouseAction::Release(btn) => {
                            emit(self.fd, EV_KEY, btn, 0);
                            emit(self.fd, EV_SYN, SYN_REPORT, 0);
                        }
                    }

                    sleep(ACTION_DELAY);
                }
                RelativeMouseMsg::Shutdown => break,
            }
        }

        unsafe {
            let _ = ui_dev_destroy(self.fd);
            let _ = libc::close(self.fd);
        }
    }
}
