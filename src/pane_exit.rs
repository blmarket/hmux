//! Retained process-exit result for a pane.

use crate::types::timeval;
use core::ffi::c_int;

/// Storage for a pane's raw wait status and death time.
pub trait PaneExitState {
    /// Returns the raw wait status.
    fn exit_status(&self) -> c_int;

    /// Replaces the raw wait status.
    fn set_exit_status(&mut self, status: c_int);

    /// Returns the recorded death time.
    fn death_time(&self) -> timeval;

    /// Replaces the recorded death time.
    fn set_death_time(&mut self, time: timeval);
}

/// The pane exit-state storage used by hmux.
#[derive(Default)]
pub struct RustPaneExitState {
    status: c_int,
    time: timeval,
}

impl PaneExitState for RustPaneExitState {
    fn exit_status(&self) -> c_int {
        self.status
    }

    fn set_exit_status(&mut self, status: c_int) {
        self.status = status;
    }

    fn death_time(&self) -> timeval {
        self.time
    }

    fn set_death_time(&mut self, time: timeval) {
        self.time = time;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_and_time_are_independently_replaceable() {
        let mut state = RustPaneExitState::default();
        assert_eq!(state.exit_status(), 0);
        assert_eq!(state.death_time().tv_sec, 0);
        assert_eq!(state.death_time().tv_usec, 0);
        state.set_exit_status(0x700);
        state.set_death_time(timeval {
            tv_sec: 42,
            tv_usec: 17,
        });
        assert_eq!(state.exit_status(), 0x700);
        assert_eq!(state.death_time().tv_sec, 42);
        assert_eq!(state.death_time().tv_usec, 17);
    }
}
