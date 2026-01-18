use crate::core::worker::{
    AbsoluteMouseAction, AbsoluteMouseMsg, AbsoluteMouseWorker, KeyboardAction, KeyboardMsg,
    KeyboardWorker, RelativeMouseAction, RelativeMouseMsg, RelativeMouseWorker,
};
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
use std::sync::mpsc::{SyncSender, sync_channel};
use std::thread::{self, JoinHandle, sleep};
use std::time::Duration;

const QUEUE_CAPACITY: usize = 1024;
const DEVICE_READY_DELAY: Duration = Duration::from_millis(500);

/// Low-level wrapper around a `/dev/uinput` device.
///
/// Actions are enqueued into a bounded `sync_channel` and written by a
/// dedicated worker thread (backpressure).
pub struct Device {
    inner: DeviceInner,
}

enum DeviceInner {
    Keyboard {
        tx: Option<SyncSender<KeyboardMsg>>,
        worker: Option<JoinHandle<()>>,
    },
    RelativeMouse {
        tx: Option<SyncSender<RelativeMouseMsg>>,
        worker: Option<JoinHandle<()>>,
    },
    AbsoluteMouse {
        tx: Option<SyncSender<AbsoluteMouseMsg>>,
        worker: Option<JoinHandle<()>>,
    },
}

pub enum DeviceType {
    Keyboard,
    RelativeMouse,
    AbsoluteMouse,
}

impl Device {
    pub fn new(r#type: DeviceType) -> Self {
        let fd = Self::open_uinput();
        match r#type {
            DeviceType::Keyboard => {
                Self::setup_keyboard(fd);
                Self::wait_device_ready();

                let (tx, rx) = sync_channel::<KeyboardMsg>(QUEUE_CAPACITY);
                let worker = Some(thread::spawn(move || KeyboardWorker::run(fd, rx)));

                Self {
                    inner: DeviceInner::Keyboard {
                        tx: Some(tx),
                        worker,
                    },
                }
            }
            DeviceType::RelativeMouse => {
                Self::setup_relative_mouse(fd);
                Self::wait_device_ready();

                let (tx, rx) = sync_channel::<RelativeMouseMsg>(QUEUE_CAPACITY);
                let worker = Some(thread::spawn(move || RelativeMouseWorker::run(fd, rx)));

                Self {
                    inner: DeviceInner::RelativeMouse {
                        tx: Some(tx),
                        worker,
                    },
                }
            }
            DeviceType::AbsoluteMouse => {
                Self::setup_absolute_mouse(fd);
                Self::wait_device_ready();

                let (tx, rx) = sync_channel::<AbsoluteMouseMsg>(QUEUE_CAPACITY);
                let worker = Some(thread::spawn(move || AbsoluteMouseWorker::run(fd, rx)));

                Self {
                    inner: DeviceInner::AbsoluteMouse {
                        tx: Some(tx),
                        worker,
                    },
                }
            }
        }
    }

    pub fn press(&self, key: u16) {
        match &self.inner {
            DeviceInner::Keyboard { tx, .. } => {
                tx.as_ref()
                    .expect("keyboard sender missing")
                    .send(KeyboardMsg::Action(KeyboardAction::Press(key)))
                    .expect("keyboard worker stopped");
            }
            DeviceInner::RelativeMouse { tx, .. } => {
                tx.as_ref()
                    .expect("relative mouse sender missing")
                    .send(RelativeMouseMsg::Action(RelativeMouseAction::Press(key)))
                    .expect("relative mouse worker stopped");
            }
            DeviceInner::AbsoluteMouse { tx, .. } => {
                tx.as_ref()
                    .expect("absolute mouse sender missing")
                    .send(AbsoluteMouseMsg::Action(AbsoluteMouseAction::Press(key)))
                    .expect("absolute mouse worker stopped");
            }
        }
    }

    pub fn release(&self, key: u16) {
        match &self.inner {
            DeviceInner::Keyboard { tx, .. } => {
                tx.as_ref()
                    .expect("keyboard sender missing")
                    .send(KeyboardMsg::Action(KeyboardAction::Release(key)))
                    .expect("keyboard worker stopped");
            }
            DeviceInner::RelativeMouse { tx, .. } => {
                tx.as_ref()
                    .expect("relative mouse sender missing")
                    .send(RelativeMouseMsg::Action(RelativeMouseAction::Release(key)))
                    .expect("relative mouse worker stopped");
            }
            DeviceInner::AbsoluteMouse { tx, .. } => {
                tx.as_ref()
                    .expect("absolute mouse sender missing")
                    .send(AbsoluteMouseMsg::Action(AbsoluteMouseAction::Release(key)))
                    .expect("absolute mouse worker stopped");
            }
        }
    }

    pub fn move_relative(&self, x: i32, y: i32) {
        match &self.inner {
            DeviceInner::Keyboard { .. } => panic!("move_relative called on keyboard device"),
            DeviceInner::AbsoluteMouse { .. } => {
                panic!("move_relative called on absolute mouse device")
            }
            DeviceInner::RelativeMouse { tx, .. } => {
                tx.as_ref()
                    .expect("relative mouse sender missing")
                    .send(RelativeMouseMsg::Action(RelativeMouseAction::Move(x, y)))
                    .expect("relative mouse worker stopped");
            }
        }
    }

    pub fn move_absolute(&self, x: i32, y: i32) {
        match &self.inner {
            DeviceInner::Keyboard { .. } => panic!("move_absolute called on keyboard device"),
            DeviceInner::RelativeMouse { .. } => {
                panic!("move_absolute called on relative mouse device")
            }
            DeviceInner::AbsoluteMouse { tx, .. } => {
                tx.as_ref()
                    .expect("absolute mouse sender missing")
                    .send(AbsoluteMouseMsg::Action(AbsoluteMouseAction::Move(x, y)))
                    .expect("absolute mouse worker stopped");
            }
        }
    }

    fn open_uinput() -> RawFd {
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

    fn wait_device_ready() {
        // uinput device creation is async; give the system time to register it.
        sleep(DEVICE_READY_DELAY);
    }

    fn setup_keyboard(fd: RawFd) {
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

    fn setup_relative_mouse(fd: RawFd) {
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

    fn setup_absolute_mouse(fd: RawFd) {
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
}

impl Drop for Device {
    fn drop(&mut self) {
        match &mut self.inner {
            DeviceInner::Keyboard { tx, worker } => {
                // Drop the sender first so the worker can exit even if no one
                // is receiving the shutdown message.
                if let Some(tx) = tx.take() {
                    let _ = tx.send(KeyboardMsg::Shutdown);
                }
                if let Some(handle) = worker.take() {
                    let _ = handle.join();
                }
            }
            DeviceInner::RelativeMouse { tx, worker } => {
                if let Some(tx) = tx.take() {
                    let _ = tx.send(RelativeMouseMsg::Shutdown);
                }
                if let Some(handle) = worker.take() {
                    let _ = handle.join();
                }
            }
            DeviceInner::AbsoluteMouse { tx, worker } => {
                if let Some(tx) = tx.take() {
                    let _ = tx.send(AbsoluteMouseMsg::Shutdown);
                }
                if let Some(handle) = worker.take() {
                    let _ = handle.join();
                }
            }
        }
    }
}
