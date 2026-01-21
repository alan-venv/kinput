use std::ffi::CString;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::BorrowedFd;
use std::path::{Path, PathBuf};

use nix::ioctl_read_buf;
use nix::ioctl_write_int;
use nix::poll::{PollFd, PollFlags, PollTimeout, poll};
use nix::request_code_read;
use nix::sys::ioctl::ioctl_num_type;

use crate::reader::structs::Device;
use crate::types::enums::Key;

ioctl_read_buf!(eviocgname, b'E', 0x06, u8);
ioctl_write_int!(eviocgrab, b'E', 0x90);

const EV_KEY: u8 = 0x01;

const BTN_LEFT: u16 = 272;
const BTN_RIGHT: u16 = 273;
const BTN_MIDDLE: u16 = 274;

const EV_BITS_BYTES: usize = 8;
const KEY_BITS_BYTES: usize = 96;

pub fn discover_event_devices() -> io::Result<Vec<PathBuf>> {
    let dir = Path::new("/dev/input");
    let mut paths = Vec::new();

    for entry in dir.read_dir()? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("event") {
            continue;
        }
        paths.push(entry.path());
    }

    Ok(paths)
}

fn nix_to_io(err: nix::Error) -> io::Error {
    io::Error::from(err)
}

fn write_c_string(dest: &mut [u8], src: &[u8]) {
    if dest.is_empty() {
        return;
    }
    let max = dest.len() - 1;
    let count = src.len().min(max);
    dest[..count].copy_from_slice(&src[..count]);
    dest[count] = 0;
    for byte in &mut dest[count + 1..] {
        *byte = 0;
    }
}

pub fn open_device(path: &Path) -> io::Result<Device> {
    let c_path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "path contains NUL"))?;

    let fd = unsafe {
        libc::open(
            c_path.as_ptr(),
            libc::O_RDONLY | libc::O_NONBLOCK | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }

    let mut dev = Device::default();
    dev.fd = fd;
    write_c_string(&mut dev.path, path.as_os_str().as_bytes());

    let mut name = [0u8; 256];
    unsafe {
        let _ = eviocgname(fd, &mut name);
    }
    dev.name = name;

    if !device_has_basic_keys(fd)? {
        close_device(&mut dev);
        return Err(io::Error::new(io::ErrorKind::Other, "device filtered"));
    }

    Ok(dev)
}

pub fn close_device(dev: &mut Device) {
    if dev.fd < 0 {
        return;
    }
    unsafe {
        libc::close(dev.fd);
    }
    dev.fd = -1;
}

pub fn poll_once(devices: &[Device]) -> io::Result<Vec<PollFlags>> {
    let mut pfds: Vec<PollFd> = devices
        .iter()
        .map(|dev| {
            let fd = unsafe { BorrowedFd::borrow_raw(dev.fd) };
            PollFd::new(fd, PollFlags::POLLIN)
        })
        .collect();

    poll(&mut pfds, PollTimeout::NONE).map_err(nix_to_io)?;

    let mut ready = Vec::with_capacity(pfds.len());
    for pfd in pfds {
        let flags = pfd.revents().unwrap_or_else(PollFlags::empty);
        ready.push(flags);
    }

    Ok(ready)
}

fn ioctl_read_bits(fd: i32, ev: u8, buf: &mut [u8]) -> io::Result<()> {
    let req = request_code_read!(b'E', 0x20 + ev, buf.len()) as ioctl_num_type;
    let res = unsafe { libc::ioctl(fd, req, buf.as_mut_ptr()) };
    if res < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn bit_is_set(buf: &[u8], code: u16) -> bool {
    let idx = (code / 8) as usize;
    if idx >= buf.len() {
        return false;
    }
    let mask = 1u8 << (code % 8);
    (buf[idx] & mask) != 0
}

fn device_has_basic_keys(fd: i32) -> io::Result<bool> {
    let mut ev_bits = [0u8; EV_BITS_BYTES];
    ioctl_read_bits(fd, 0, &mut ev_bits)?;
    if !bit_is_set(&ev_bits, EV_KEY as u16) {
        return Ok(false);
    }

    let mut key_bits = [0u8; KEY_BITS_BYTES];
    ioctl_read_bits(fd, EV_KEY, &mut key_bits)?;

    let has_keyboard_key = bit_is_set(&key_bits, Key::A.value())
        || bit_is_set(&key_bits, Key::Q.value())
        || bit_is_set(&key_bits, Key::Z.value())
        || bit_is_set(&key_bits, Key::Num1.value())
        || bit_is_set(&key_bits, Key::Enter.value())
        || bit_is_set(&key_bits, Key::Space.value());

    let has_mouse_button = bit_is_set(&key_bits, BTN_LEFT)
        || bit_is_set(&key_bits, BTN_RIGHT)
        || bit_is_set(&key_bits, BTN_MIDDLE);

    Ok(has_keyboard_key || has_mouse_button)
}

#[allow(dead_code)]
pub fn try_grab_device(fd: i32) -> io::Result<()> {
    let rc = unsafe { eviocgrab(fd, 1) };
    match rc {
        Ok(_) => Ok(()),
        Err(err) => Err(io::Error::from(err)),
    }
}

#[allow(dead_code)]
pub fn ungrab_device(fd: i32) -> io::Result<()> {
    let rc = unsafe { eviocgrab(fd, 0) };
    match rc {
        Ok(_) => Ok(()),
        Err(err) => Err(io::Error::from(err)),
    }
}
