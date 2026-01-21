use std::io;
use std::sync::atomic::{AtomicBool, Ordering};

use nix::sys::signal::{self, SaFlags, SigAction, SigHandler, SigSet, Signal};

static RUNNING: AtomicBool = AtomicBool::new(true);

extern "C" fn handle_signal(_signo: libc::c_int) {
    RUNNING.store(false, Ordering::Relaxed);
}

fn nix_to_io(err: nix::Error) -> io::Error {
    io::Error::from(err)
}

pub fn install_signal_handlers() -> io::Result<()> {
    let action = SigAction::new(
        SigHandler::Handler(handle_signal),
        SaFlags::empty(),
        SigSet::empty(),
    );
    unsafe {
        signal::sigaction(Signal::SIGINT, &action).map_err(nix_to_io)?;
        signal::sigaction(Signal::SIGTERM, &action).map_err(nix_to_io)?;
    }
    Ok(())
}

pub fn is_running() -> bool {
    RUNNING.load(Ordering::Relaxed)
}
