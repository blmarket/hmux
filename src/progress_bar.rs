//! Stable access to terminal progress-bar state.

use crate::types::progress_bar_state;
use core::ffi::c_int;

/// The state and percentage sent through the terminal progress-bar protocol.
pub trait ProgressBar {
    /// Builds a progress bar from its state and percentage.
    fn from_progress_bar(state: progress_bar_state, progress: c_int) -> Self
    where
        Self: Sized;

    /// Returns the progress-bar state.
    fn progress_bar_state(&self) -> progress_bar_state;

    /// Returns the progress percentage.
    fn progress_bar_progress(&self) -> c_int;

    /// Replaces the progress-bar state and percentage.
    fn set_progress_bar(&mut self, state: progress_bar_state, progress: c_int);
}

impl ProgressBar for crate::types::progress_bar {
    fn from_progress_bar(state: progress_bar_state, progress: c_int) -> Self {
        Self { state, progress }
    }

    fn progress_bar_state(&self) -> progress_bar_state {
        self.state
    }
    fn progress_bar_progress(&self) -> c_int {
        self.progress
    }
    fn set_progress_bar(&mut self, state: progress_bar_state, progress: c_int) {
        self.state = state;
        self.progress = progress;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::progress_bar;

    #[test]
    fn state_and_progress_round_trip_together() {
        let mut progress = progress_bar::from_progress_bar(2, 25);
        assert_eq!(progress.progress_bar_state(), 2);
        assert_eq!(progress.progress_bar_progress(), 25);
        progress.set_progress_bar(0, -1);
        assert_eq!(
            (
                progress.progress_bar_state(),
                progress.progress_bar_progress()
            ),
            (0, -1)
        );
    }
}
