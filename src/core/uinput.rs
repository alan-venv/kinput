use crate::types::constants::*;
use crate::types::structs::{UInputAbsSetup, UInputSetup};

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

const DEVICE_READY_DELAY: Duration = Duration::from_millis(500);

pub fn open_uinput() -> RawFd {
    let path = std::ffi::CString::new("/dev/uinput").unwrap();
    // Open in blocking mode: the worker thread can block on write, and the
    // bounded queue provides backpressure to callers.
    let fd = unsafe { libc::open(path.as_ptr(), libc::O_WRONLY) };
    if fd < 0 {
        panic!(
            "open /dev/uinput failed: {}",
            std::io::Error::last_os_error()
        );
    }
    fd
}

pub fn wait_device_ready() {
    // uinput device creation is async; give the system time to register it.
    sleep(DEVICE_READY_DELAY);
}

pub fn setup_keyboard(fd: RawFd) {
    unsafe {
        ui_set_evbit(fd, EV_KEY as u64).unwrap();

        for key in 1..=119 {
            ui_set_keybit(fd, key as u64).unwrap();
        }

        let mut setup: UInputSetup = std::mem::zeroed();
        setup.id.bustype = BUS_USB;
        setup.id.vendor = 0x1234;
        setup.id.product = 0x5678;
        setup.id.version = 0;
        setup.ff_effects_max = 0;

        let name = b"Keyboard device\0";
        setup.name[..name.len()].copy_from_slice(name);

        ui_dev_setup(fd, &setup).unwrap();
        ui_dev_create(fd).unwrap();
    }
}

pub fn setup_relative_mouse(fd: RawFd) {
    unsafe {
        ui_set_evbit(fd, EV_KEY as u64).unwrap();
        ui_set_keybit(fd, BTN_LEFT as u64).unwrap();
        ui_set_keybit(fd, BTN_RIGHT as u64).unwrap();

        ui_set_evbit(fd, EV_REL as u64).unwrap();
        ui_set_relbit(fd, REL_X as u64).unwrap();
        ui_set_relbit(fd, REL_Y as u64).unwrap();

        let mut setup: UInputSetup = std::mem::zeroed();
        setup.id.bustype = BUS_USB;
        setup.id.vendor = 0x1234;
        setup.id.product = 0x5678;
        setup.id.version = 0;
        setup.ff_effects_max = 0;

        let name = b"Relative mouse device\0";
        setup.name[..name.len()].copy_from_slice(name);

        ui_dev_setup(fd, &setup).unwrap();
        ui_dev_create(fd).unwrap();
    }
}

pub fn setup_absolute_mouse(fd: RawFd) {
    unsafe {
        ui_set_evbit(fd, EV_KEY as u64).unwrap();
        ui_set_keybit(fd, BTN_LEFT as u64).unwrap();
        ui_set_keybit(fd, BTN_RIGHT as u64).unwrap();

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

        let name = b"Absolute mouse device\0";
        setup.name[..name.len()].copy_from_slice(name);

        ui_dev_setup(fd, &setup).unwrap();
        ui_dev_create(fd).unwrap();
    }
}
