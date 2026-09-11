//! Retained process-exit result for a pane.

use crate::types::timeval;
use core::ffi::c_int;

/// Observations of a pane's retained process-exit result.
pub trait PaneExitState {
    /// Returns the raw wait status.
    fn exit_status(&self) -> c_int;

    /// Returns the recorded death time.
    fn death_time(&self) -> timeval;

}
