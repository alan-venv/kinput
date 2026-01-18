use crate::types::constants::{EV_KEY, EV_REL, EV_SYN, REL_X, REL_Y, SYN_REPORT};
use crate::types::structs::InputEvent;
use nix::ioctl_none;
use std::os::unix::io::RawFd;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::thread::sleep;
use std::time::{Duration, Instant};

ioctl_none!(ui_dev_destroy, b'U', 2);

const KEYBOARD_ACTION_DELAY: Duration = Duration::from_millis(2);

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
                            Self::emit(self.fd, EV_KEY, key, 1);
                            Self::emit(self.fd, EV_SYN, SYN_REPORT, 0);
                        }
                        KeyboardAction::Release(key) => {
                            Self::emit(self.fd, EV_KEY, key, 0);
                            Self::emit(self.fd, EV_SYN, SYN_REPORT, 0);
                        }
                    }

                    sleep(KEYBOARD_ACTION_DELAY);
                }
                KeyboardMsg::Shutdown => break,
            }
        }

        unsafe {
            let _ = ui_dev_destroy(self.fd);
            let _ = libc::close(self.fd);
        }
    }

    fn emit(fd: RawFd, type_: u16, code: u16, value: i32) {
        emit(fd, type_, code, value)
    }
}

// RELATIVE MOUSE

#[derive(Debug, Copy, Clone)]
pub enum RelativeMouseAction {
    Move(i32, i32),
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
        // Fixed coalescing window equal to the action delay.
        // For each window we emit at most one REL_X/REL_Y pair.
        let mut queued_dx: i32 = 0;
        let mut queued_dy: i32 = 0;

        while let Ok(msg) = self.rx.recv() {
            match msg {
                RelativeMouseMsg::Action(RelativeMouseAction::Move(dx, dy)) => {
                    queued_dx += dx;
                    queued_dy += dy;

                    let deadline = Instant::now() + KEYBOARD_ACTION_DELAY;

                    // Coalesce everything that arrives within the window.
                    loop {
                        let now = Instant::now();
                        if now >= deadline {
                            break;
                        }

                        match self.rx.recv_timeout(deadline - now) {
                            Ok(RelativeMouseMsg::Action(RelativeMouseAction::Move(x, y))) => {
                                queued_dx += x;
                                queued_dy += y;
                            }
                            Ok(RelativeMouseMsg::Shutdown) => {
                                // Flush what we have and then exit.
                                break;
                            }
                            Err(RecvTimeoutError::Timeout) => break,
                            Err(RecvTimeoutError::Disconnected) => break,
                        }
                    }

                    if queued_dx != 0 {
                        emit(self.fd, EV_REL, REL_X, queued_dx);
                    }
                    if queued_dy != 0 {
                        emit(self.fd, EV_REL, REL_Y, queued_dy);
                    }
                    emit(self.fd, EV_SYN, SYN_REPORT, 0);

                    queued_dx = 0;
                    queued_dy = 0;
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

fn emit(fd: RawFd, type_: u16, code: u16, value: i32) {
    let ev = InputEvent {
        time: libc::timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        type_,
        code,
        value,
    };

    let size = std::mem::size_of::<InputEvent>();
    let ret = unsafe { libc::write(fd, &ev as *const _ as *const libc::c_void, size) };
    if ret != size as isize {
        panic!(
            "write failed or partial write: {}",
            std::io::Error::last_os_error()
        );
    }
}
