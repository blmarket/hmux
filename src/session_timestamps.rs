//! Copyable timestamps retained by a session.

use crate::types::timeval;

/// All timestamps retained by a session.
#[derive(Copy, Clone, Default)]
pub struct SessionTimestamps {
    pub creation: timeval,
    pub last_attached: timeval,
    pub activity: timeval,
    pub last_activity: timeval,
}

/// Storage for a session's retained timestamps.
pub trait SessionTimestampState {
    /// Returns all retained timestamps.
    fn session_timestamps(&self) -> SessionTimestamps;

    /// Replaces the creation time.
    fn set_session_creation_time(&mut self, time: timeval);

    /// Replaces the last attachment time.
    fn set_session_last_attached_time(&mut self, time: timeval);

    /// Replaces the current activity time.
    fn set_session_activity_time(&mut self, time: timeval);

    /// Replaces the preceding activity time.
    fn set_session_last_activity_time(&mut self, time: timeval);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamps_are_independently_replaceable() {
        let mut state = crate::session::session::default();
        state.set_session_creation_time(timeval {
            tv_sec: 1,
            tv_usec: 2,
        });
        state.set_session_last_attached_time(timeval {
            tv_sec: 3,
            tv_usec: 4,
        });
        state.set_session_activity_time(timeval {
            tv_sec: 5,
            tv_usec: 6,
        });
        state.set_session_last_activity_time(timeval {
            tv_sec: 7,
            tv_usec: 8,
        });
        let times = state.session_timestamps();
        assert_eq!((times.creation.tv_sec, times.creation.tv_usec), (1, 2));
        assert_eq!(
            (times.last_attached.tv_sec, times.last_attached.tv_usec),
            (3, 4)
        );
        assert_eq!((times.activity.tv_sec, times.activity.tv_usec), (5, 6));
        assert_eq!(
            (times.last_activity.tv_sec, times.last_activity.tv_usec),
            (7, 8)
        );
    }
}
