use std::sync::atomic::{AtomicBool, Ordering};

use crate::Error;

static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

extern "C" fn request_shutdown(_: std::os::raw::c_int) {
    SHUTDOWN_REQUESTED.store(true, Ordering::Relaxed);
}

pub fn install_shutdown_handlers() -> Result<(), Error> {
    use nix::sys::signal::{self, SaFlags, SigAction, SigHandler, SigSet, Signal};

    let action = SigAction::new(
        SigHandler::Handler(request_shutdown),
        SaFlags::empty(),
        SigSet::empty(),
    );

    unsafe {
        signal::sigaction(Signal::SIGINT, &action)?;
        signal::sigaction(Signal::SIGTERM, &action)?;
    }

    Ok(())
}

pub fn shutdown_requested() -> bool {
    SHUTDOWN_REQUESTED.load(Ordering::Relaxed)
}
