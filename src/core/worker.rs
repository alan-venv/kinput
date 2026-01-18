use crate::types::constants::{EV_KEY, EV_REL, EV_SYN, REL_X, REL_Y, SYN_REPORT};
use crate::types::structs::InputEvent;
use nix::ioctl_none;
use std::os::unix::io::RawFd;
use std::sync::mpsc::Receiver;
use std::thread::sleep;
use std::time::{Duration, Instant};

ioctl_none!(ui_dev_destroy, b'U', 2);

const KEYBOARD_ACTION_DELAY: Duration = Duration::from_millis(2);

// Upper bound to avoid starving the emitter if producers are extremely fast.
const REL_MOUSE_DRAIN_LIMIT: usize = 100;
const REL_MOUSE_DRAIN_BUDGET: Duration = Duration::from_micros(200);

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
        let mut last_execution = Instant::now() - Duration::from_millis(10);
        let mut queued_dx: i32 = 0;
        let mut queued_dy: i32 = 0;

        while let Ok(msg) = self.rx.recv() {
            match msg {
                RelativeMouseMsg::Action(RelativeMouseAction::Move(mut dx, mut dy)) => {
                    // Coalesce moves already waiting in the channel.
                    // Guard rails prevent spending unbounded time draining.
                    let drain_deadline = Instant::now() + REL_MOUSE_DRAIN_BUDGET;
                    let mut drained = 0usize;

                    while drained < REL_MOUSE_DRAIN_LIMIT && Instant::now() < drain_deadline {
                        match self.rx.try_recv() {
                            Ok(RelativeMouseMsg::Action(RelativeMouseAction::Move(x, y))) => {
                                dx += x;
                                dy += y;
                                drained += 1;
                            }
                            Ok(RelativeMouseMsg::Shutdown) => {
                                // Execute what we have buffered before shutting down.
                                queued_dx += dx;
                                queued_dy += dy;
                                break;
                            }
                            Err(_) => break,
                        }
                    }

                    let elapsed = last_execution.elapsed();

                    if elapsed < KEYBOARD_ACTION_DELAY {
                        queued_dx += dx;
                        queued_dy += dy;
                        continue;
                    }

                    // We're allowed to execute now: include anything queued.
                    let exec_dx = queued_dx + dx;
                    let exec_dy = queued_dy + dy;
                    queued_dx = 0;
                    queued_dy = 0;

                    if exec_dx != 0 {
                        emit(self.fd, EV_REL, REL_X, exec_dx);
                    }
                    if exec_dy != 0 {
                        emit(self.fd, EV_REL, REL_Y, exec_dy);
                    }
                    emit(self.fd, EV_SYN, SYN_REPORT, 0);

                    last_execution = Instant::now();
                    sleep(KEYBOARD_ACTION_DELAY);
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
