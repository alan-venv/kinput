use std::io;
use std::sync::mpsc::{self, Receiver, RecvError, SyncSender};
use std::thread;

use crate::reader::devices::{close_device, discover_event_devices, open_device, poll_once};
use crate::reader::events::{normalize_event, read_events};
use crate::reader::signals;
use crate::reader::structs::{Device, EventBatch};
use crate::types::enums::Key;

fn bytes_to_string(buf: &[u8]) -> String {
    let end = buf.iter().position(|b| *b == 0).unwrap_or(buf.len());
    String::from_utf8_lossy(&buf[..end]).into_owned()
}

pub struct InputReader {
    tx: Option<SyncSender<Key>>,
    rx: Receiver<Key>,
}

impl InputReader {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::sync_channel::<Key>(4096);
        Self { tx: Some(tx), rx }
    }

    pub fn start(&mut self) -> io::Result<()> {
        if let Err(err) = signals::install_signal_handlers() {
            eprintln!("signal setup failed: {}", err);
            return Err(err);
        }

        let paths = match discover_event_devices() {
            Ok(paths) => paths,
            Err(err) => {
                eprintln!("opendir(/dev/input) failed: {}", err);
                return Err(err);
            }
        };

        let mut devices: Vec<Device> = Vec::new();
        for path in paths {
            if let Ok(dev) = open_device(&path) {
                devices.push(dev);
            }
        }

        if devices.is_empty() {
            eprintln!("No /dev/input/event* devices opened.");
            eprintln!("Tip: ensure you have permission (root or group 'input').");
            return Err(io::Error::new(io::ErrorKind::NotFound, "no input devices"));
        }

        let tx = match self.tx.take() {
            Some(tx) => tx,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "reader already started",
                ));
            }
        };
        thread::spawn(move || {
            if let Err(err) = capture_loop(&mut devices, &tx) {
                eprintln!("capture loop failed: {}", err);
            }
            for dev in &mut devices {
                close_device(dev);
            }
        });

        Ok(())
    }

    pub fn receive(&self) -> Result<Key, RecvError> {
        self.rx.recv()
    }
}

fn capture_loop(devices: &mut Vec<Device>, tx: &SyncSender<Key>) -> io::Result<()> {
    let mut batch = EventBatch::default();

    while signals::is_running() {
        let ready = match poll_once(devices) {
            Ok(ready) => ready,
            Err(err) => {
                if err.kind() == io::ErrorKind::Interrupted {
                    if signals::is_running() {
                        continue;
                    }
                    break;
                }
                eprintln!("poll() failed: {}", err);
                return Err(err);
            }
        };

        if !signals::is_running() {
            break;
        }

        let mut idx = 0;
        while idx < devices.len() {
            let flags = ready
                .get(idx)
                .copied()
                .unwrap_or_else(nix::poll::PollFlags::empty);
            let has_input = flags.contains(nix::poll::PollFlags::POLLIN);
            let has_error = flags.intersects(
                nix::poll::PollFlags::POLLERR
                    | nix::poll::PollFlags::POLLHUP
                    | nix::poll::PollFlags::POLLNVAL,
            );

            if !has_input && !has_error {
                idx += 1;
                continue;
            }

            let dev = &mut devices[idx];
            match read_events(dev, &mut batch) {
                Ok(_) => {
                    for ev in batch.as_slice() {
                        if let Some(key) = normalize_event(ev) {
                            if tx.send(key).is_err() {
                                return Ok(());
                            }
                        }
                    }
                }
                Err(err) => {
                    let path = bytes_to_string(&dev.path);
                    let name = bytes_to_string(&dev.name);
                    if err.kind() == io::ErrorKind::UnexpectedEof {
                        eprintln!("device closed: {} ({})", path, name);
                    } else if has_error {
                        // Device removal/error can surface as poll error; treat as quiet removal.
                    } else {
                        eprintln!("read({}) failed: {}", path, err);
                    }
                    close_device(dev);
                    devices.swap_remove(idx);
                    continue;
                }
            }
            idx += 1;
        }
    }

    Ok(())
}
