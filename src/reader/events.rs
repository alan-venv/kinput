use std::io;
use std::mem;

use crate::reader::structs::Device;
use crate::reader::structs::EventBatch;
use crate::types::enums::Key;

const EV_KEY: u16 = 0x01;

pub fn read_events(dev: &Device, batch: &mut EventBatch) -> io::Result<usize> {
    let event_size = mem::size_of::<libc::input_event>();
    let buf_size = mem::size_of_val(&batch.events);

    let n = unsafe { libc::read(dev.fd, batch.events.as_mut_ptr().cast(), buf_size) };
    if n < 0 {
        let err = io::Error::last_os_error();
        if err.kind() == io::ErrorKind::WouldBlock {
            batch.count = 0;
            return Ok(0);
        }
        return Err(err);
    }

    if n == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "device closed",
        ));
    }

    let count = (n as usize) / event_size;
    batch.count = count;
    Ok(count)
}

pub fn normalize_event(ev: &libc::input_event) -> Option<Key> {
    if ev.type_ != EV_KEY {
        return None;
    }

    // 0 released // 1 press // autorepeat
    if ev.value != 0 {
        return Key::from_code(ev.code);
    }

    None
}
