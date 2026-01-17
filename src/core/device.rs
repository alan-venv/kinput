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
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread::{self, JoinHandle, sleep};
use std::time::Duration;

const KEYBOARD_QUEUE_CAPACITY: usize = 1024;
const KEYBOARD_ACTION_DELAY: Duration = Duration::from_millis(2);

/// Low-level wrapper around a `/dev/uinput` device.
///
/// For `DeviceType::Keyboard`, key actions are enqueued into a bounded
/// `sync_channel` and written by a dedicated worker thread (backpressure).
///
/// For other device types, events are written directly to the uinput FD.
pub struct Device {
    inner: DeviceInner,
}

enum DeviceInner {
    Direct {
        fd: RawFd,
    },
    Keyboard {
        tx: SyncSender<KeyboardMsg>,
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

                // Give the kernel/userspace a moment to register the new device.
                sleep(Duration::from_millis(500));

                let (tx, rx) = sync_channel::<KeyboardMsg>(KEYBOARD_QUEUE_CAPACITY);
                let worker = Some(thread::spawn(move || KeyboardWorker::run(fd, rx)));

                Self {
                    inner: DeviceInner::Keyboard { tx, worker },
                }
            }
            DeviceType::RelativeMouse => {
                Self::setup_relative_mouse(fd);
                Self {
                    inner: DeviceInner::Direct { fd },
                }
            }
            DeviceType::AbsoluteMouse => {
                Self::setup_absolute_mouse(fd);
                Self {
                    inner: DeviceInner::Direct { fd },
                }
            }
        }
    }

    pub fn press(&self, key: u16) {
        match &self.inner {
            DeviceInner::Keyboard { tx, .. } => {
                tx.send(KeyboardMsg::Action(KeyboardAction::Press(key)))
                    .expect("keyboard worker stopped");
            }
            DeviceInner::Direct { fd } => {
                Self::emit_fd(*fd, EV_KEY, key, 1);
                Self::emit_fd(*fd, EV_SYN, SYN_REPORT, 0);
            }
        }
    }

    pub fn release(&self, key: u16) {
        match &self.inner {
            DeviceInner::Keyboard { tx, .. } => {
                tx.send(KeyboardMsg::Action(KeyboardAction::Release(key)))
                    .expect("keyboard worker stopped");
            }
            DeviceInner::Direct { fd } => {
                Self::emit_fd(*fd, EV_KEY, key, 0);
                Self::emit_fd(*fd, EV_SYN, SYN_REPORT, 0);
            }
        }
    }

    pub fn move_relative(&self, x: i32, y: i32) {
        match &self.inner {
            DeviceInner::Keyboard { .. } => panic!("move_relative called on keyboard device"),
            DeviceInner::Direct { fd } => {
                Self::emit_fd(*fd, EV_REL, REL_X, x);
                Self::emit_fd(*fd, EV_REL, REL_Y, y);
                Self::emit_fd(*fd, EV_SYN, SYN_REPORT, 0);
            }
        }
    }

    pub fn move_absolute(&self, x: i32, y: i32) {
        match &self.inner {
            DeviceInner::Keyboard { .. } => panic!("move_absolute called on keyboard device"),
            DeviceInner::Direct { fd } => {
                Self::emit_fd(*fd, EV_ABS, ABS_X, x);
                Self::emit_fd(*fd, EV_ABS, ABS_Y, y);
                Self::emit_fd(*fd, EV_SYN, SYN_REPORT, 0);
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
        sleep(Duration::from_millis(500));
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
        sleep(Duration::from_millis(500));
    }

    fn emit_fd(fd: RawFd, type_: u16, code: u16, value: i32) {
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
}

impl Drop for Device {
    fn drop(&mut self) {
        match &mut self.inner {
            DeviceInner::Direct { fd } => unsafe {
                let _ = ui_dev_destroy(*fd);
                let _ = libc::close(*fd);
            },
            DeviceInner::Keyboard { tx, worker } => {
                let _ = tx.send(KeyboardMsg::Shutdown);
                if let Some(handle) = worker.take() {
                    let _ = handle.join();
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum KeyboardAction {
    Press(u16),
    Release(u16),
}

enum KeyboardMsg {
    Action(KeyboardAction),
    Shutdown,
}

struct KeyboardWorker {
    fd: RawFd,
    rx: Receiver<KeyboardMsg>,
}

impl KeyboardWorker {
    fn run(fd: RawFd, rx: Receiver<KeyboardMsg>) {
        let worker = Self { fd, rx };
        worker.event_loop();
    }

    fn event_loop(self) {
        while let Ok(msg) = self.rx.recv() {
            match msg {
                KeyboardMsg::Action(action) => {
                    match action {
                        KeyboardAction::Press(key) => {
                            Device::emit_fd(self.fd, EV_KEY, key, 1);
                            Device::emit_fd(self.fd, EV_SYN, SYN_REPORT, 0);
                        }
                        KeyboardAction::Release(key) => {
                            Device::emit_fd(self.fd, EV_KEY, key, 0);
                            Device::emit_fd(self.fd, EV_SYN, SYN_REPORT, 0);
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
}
