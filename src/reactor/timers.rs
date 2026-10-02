//! Timer work is a future owned by the caller's task handle.
use super::ensure_runtime;
use hmux_rt::Handle as _;
use std::io;
use std::time::{Duration, Instant};

/// Owns one scheduled future. Dropping it cancels the work.
/// Owners that track active work clear their timer from the callback.
pub struct Timer {
    _task: hmux_rt::mio::Task,
}

impl Timer {
    /// Zero duration defers execution to a runtime turn.
    pub fn new(delay: Duration, callback: impl FnOnce() + 'static) -> io::Result<Self> {
        ensure_runtime();
        let now = Instant::now();
        // Preserve the existing behavior: an overflowing deadline is expired.
        let deadline = now.checked_add(delay).unwrap_or(now);
        let handle = hmux_rt::mio::Handle::current();
        let sleep = hmux_rt::mio::Sleep::new(deadline);
        let task = handle.spawn(async move {
            sleep.await.expect("timer wait failed");
            callback();
        })?;
        Ok(Self { _task: task })
    }
}
