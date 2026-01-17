use crate::types::constants::*;
use crate::types::structs::{InputEvent, UInputAbsSetup, UInputSetup};

use nix::ioctl_none;
use nix::ioctl_write_int;
use nix::ioctl_write_ptr;

ioctl_write_int!(ui_set_evbit, b'U', 100);
ioctl_write_int!(ui_set_keybit, b'U', 101);
ioctl_write_int!(ui_set_relbit, b'U', 102);
ioctl_write_int!(ui_set_absbit, b'U', 103);
ioctl_write_ptr!(ui_dev_setup, b'U', 3, UInputSetup);
ioctl_write_ptr!(ui_abs_setup, b'U', 4, UInputAbsSetup);
ioctl_none!(ui_dev_create, b'U', 1);
ioctl_none!(ui_dev_destroy, b'U', 2);

use std::os::unix::io::RawFd;
use std::thread::sleep;
use std::time::Duration;

/// Low-level wrapper around a `/dev/uinput` file descriptor.
///
/// `Device` performs the uinput ioctl sequence to register a virtual input
/// device with the kernel and then emits events by writing `InputEvent`
/// structures to the uinput FD.
#[derive(Clone)]
pub struct Device {
    fd: RawFd,
}

impl Device {
    pub fn new() -> Self {
        let fd = Self::open_uinput();
        Self::setup_device(fd);
        Self { fd: fd }
    }

    pub fn press(&self, key: u16) {
        self.emit(EV_KEY, key, 1);
        self.emit(EV_SYN, SYN_REPORT, 0);
    }

    pub fn release(&self, key: u16) {
        self.emit(EV_KEY, key, 0);
        self.emit(EV_SYN, SYN_REPORT, 0);
    }

    pub fn move_relative(&self, x: i32, y: i32) {
        self.emit(EV_REL, REL_X, x);
        self.emit(EV_REL, REL_Y, y);
        self.emit(EV_SYN, SYN_REPORT, 0);
    }

    pub fn move_absolute(&self, x: i32, y: i32) {
        self.emit(EV_ABS, ABS_X, x);
        self.emit(EV_ABS, ABS_Y, y);
        self.emit(EV_SYN, SYN_REPORT, 0);
    }

    fn open_uinput() -> RawFd {
        let path = std::ffi::CString::new("/dev/uinput").unwrap();
        let fd = unsafe { libc::open(path.as_ptr(), libc::O_WRONLY | libc::O_NONBLOCK) };
        if fd < 0 {
            panic!(
                "open /dev/uinput failed: {}",
                std::io::Error::last_os_error()
            );
        }
        return fd;
    }

    fn setup_device(fd: RawFd) {
        unsafe {
            ui_set_evbit(fd, EV_KEY as u64).unwrap();

            for key in 1..=119 {
                ui_set_keybit(fd, key as u64).unwrap();
            }

            ui_set_keybit(fd, BTN_LEFT as u64).unwrap();
            ui_set_keybit(fd, BTN_RIGHT as u64).unwrap();

            ui_set_evbit(fd, EV_REL as u64).unwrap();
            ui_set_relbit(fd, REL_X as u64).unwrap();
            ui_set_relbit(fd, REL_Y as u64).unwrap();

            ui_set_evbit(fd, EV_ABS as u64).unwrap();
            ui_set_absbit(fd, ABS_X as u64).unwrap();
            ui_set_absbit(fd, ABS_Y as u64).unwrap();

            let mut abs_x: UInputAbsSetup = std::mem::zeroed();
            abs_x.code = ABS_X;
            abs_x.absinfo.minimum = 0;
            abs_x.absinfo.maximum = 65535;
            ui_abs_setup(fd, &abs_x).unwrap();
            let mut abs_y: UInputAbsSetup = std::mem::zeroed();
            abs_y.code = ABS_Y;
            abs_y.absinfo.minimum = 0;
            abs_y.absinfo.maximum = 65535;
            ui_abs_setup(fd, &abs_y).unwrap();

            let mut setup: UInputSetup = std::mem::zeroed();
            setup.id.bustype = BUS_USB;
            setup.id.vendor = 0x1234;
            setup.id.product = 0x5678;
            setup.id.version = 0;
            setup.ff_effects_max = 0;

            let name = b"Example device\0";
            setup.name[..name.len()].copy_from_slice(name);

            ui_dev_setup(fd, &setup).unwrap();
            ui_dev_create(fd).unwrap();
        }
        sleep(Duration::from_millis(500));
    }

    fn emit(&self, type_: u16, code: u16, value: i32) {
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
        let ret = unsafe { libc::write(self.fd, &ev as *const _ as *const libc::c_void, size) };
        if ret != size as isize {
            panic!(
                "write failed or partial write: {}",
                std::io::Error::last_os_error()
            );
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            let _ = ui_dev_destroy(self.fd);
            let _ = libc::close(self.fd);
        }
    }
}
