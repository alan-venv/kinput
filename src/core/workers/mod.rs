mod absolute_mouse;
mod keyboard;
mod relative_mouse;

pub use absolute_mouse::{AbsoluteMouseAction, AbsoluteMouseMsg, AbsoluteMouseWorker};
pub use keyboard::{KeyboardAction, KeyboardMsg, KeyboardWorker};
pub use relative_mouse::{RelativeMouseAction, RelativeMouseMsg, RelativeMouseWorker};

use crate::types::structs::InputEvent;
use std::os::unix::io::RawFd;
use std::time::Duration;

const ACTION_DELAY: Duration = Duration::from_micros(500);

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
