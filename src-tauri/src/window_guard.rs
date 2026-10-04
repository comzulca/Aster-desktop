//! Closed admission for caption intent; no window handle, persistence or egress.
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    Minimise,
    ToggleMaximise,
    ToggleFullscreen,
    StartDragging,
    Close,
}

pub(crate) fn admit(
    caller: &str,
    action: Action,
    native_open: bool,
    closing: bool,
    active_count: usize,
    persistence_idle: bool,
) -> Result<(), &'static str> {
    if caller != "main" {
        return Err("caller_denied");
    }
    if native_open {
        return Err("native_dialog_busy");
    }
    if closing {
        return Err("window_closing");
    }
    if action == Action::Close && (active_count != 0 || !persistence_idle) {
        return Err("busy");
    }
    Ok(())
}

pub(crate) fn confirm_close(
    closing: bool,
    native_open: bool,
    active_count: usize,
    persistence_idle: bool,
) -> Result<(), &'static str> {
    if !closing {
        return Err("window_close_unconfirmed");
    }
    if native_open {
        return Err("native_dialog_busy");
    }
    if active_count != 0 || !persistence_idle {
        return Err("busy");
    }
    Ok(())
}

#[derive(Default)]
pub(crate) struct CloseState {
    closing: AtomicBool,
    authorised: AtomicBool,
}

impl CloseState {
    pub(crate) fn is_closing(&self) -> bool {
        self.closing.load(Ordering::Acquire)
    }

    // Called only while the shared native/caption admission guard is held and
    // the persistent/session guards prove no request or durable write is active.
    pub(crate) fn arm(&self) -> Result<(), &'static str> {
        self.closing
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| "window_closing")?;
        self.authorised.store(true, Ordering::Release);
        Ok(())
    }

    pub(crate) fn consume_authorisation(&self) -> bool {
        self.authorised.swap(false, Ordering::AcqRel)
    }

    pub(crate) fn abort(&self) {
        self.authorised.store(false, Ordering::Release);
        self.closing.store(false, Ordering::Release);
    }
}

#[cfg(test)]
#[path = "window_guard_tests.rs"]
mod tests;
