//! Copyable timestamps retained by a window.

use crate::types::timeval;

/// All timestamps retained by a window.
#[derive(Copy, Clone, Default)]
pub struct WindowTimestamps {
    pub creation_time: timeval,
    pub activity_time: timeval,
    pub name_time: timeval,
}

/// Storage for a window's retained timestamps.
pub trait WindowTimestampState: Default {
    /// Returns all retained timestamps.
    fn timestamps(&self) -> WindowTimestamps;

    /// Replaces the creation time.
    fn set_creation_time(&mut self, time: timeval);

    /// Replaces the activity time.
    fn set_activity_time(&mut self, time: timeval);

    /// Replaces the last automatic-name update time.
    fn set_name_update_time(&mut self, time: timeval);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamps_are_independently_replaceable() {
        let mut state = crate::types::window::default();
        state.set_creation_time(timeval {
            tv_sec: 1,
            tv_usec: 2,
        });
        state.set_activity_time(timeval {
            tv_sec: 3,
            tv_usec: 4,
        });
        state.set_name_update_time(timeval {
            tv_sec: 5,
            tv_usec: 6,
        });
        let times = state.timestamps();
        assert_eq!((times.creation_time.tv_sec, times.creation_time.tv_usec), (1, 2));
        assert_eq!((times.activity_time.tv_sec, times.activity_time.tv_usec), (3, 4));
        assert_eq!(
            (times.name_time.tv_sec, times.name_time.tv_usec),
            (5, 6)
        );
    }
}
