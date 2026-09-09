//! Stable access to seconds-and-microseconds time values.

impl crate::types::timeval {
    /// Returns the current wall-clock time with microsecond precision.
    pub(crate) fn now() -> Self {
        Self::from_system_time(std::time::SystemTime::now())
    }

    fn from_system_time(time: std::time::SystemTime) -> Self {
        let nanos = match time.duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => duration.as_nanos() as i128,
            Err(error) => -(error.duration().as_nanos() as i128),
        };
        Self {
            tv_sec: nanos.div_euclid(1_000_000_000) as i64,
            tv_usec: (nanos.rem_euclid(1_000_000_000) / 1_000) as i64,
        }
    }
}

/// A wall-clock time or duration represented by seconds and microseconds.
pub trait TimeValue {
    /// Builds a time value from seconds and microseconds.
    fn from_time_value(seconds: i64, microseconds: i64) -> Self
    where
        Self: Sized;

    /// Returns the whole seconds component.
    fn time_value_seconds(&self) -> i64;

    /// Replaces the whole seconds component.
    fn set_time_value_seconds(&mut self, seconds: i64);

    /// Returns the microseconds component.
    fn time_value_microseconds(&self) -> i64;

    /// Replaces the microseconds component.
    fn set_time_value_microseconds(&mut self, microseconds: i64);
}

impl TimeValue for crate::types::timeval {
    fn from_time_value(seconds: i64, microseconds: i64) -> Self {
        Self {
            tv_sec: seconds,
            tv_usec: microseconds,
        }
    }
    fn time_value_seconds(&self) -> i64 {
        self.tv_sec
    }
    fn set_time_value_seconds(&mut self, seconds: i64) {
        self.tv_sec = seconds;
    }
    fn time_value_microseconds(&self) -> i64 {
        self.tv_usec
    }
    fn set_time_value_microseconds(&mut self, microseconds: i64) {
        self.tv_usec = microseconds;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::timeval;

    #[test]
    fn system_time_preserves_signed_seconds_and_normalized_microseconds() {
        use std::time::{Duration, UNIX_EPOCH};

        for (time, seconds, microseconds) in [
            (UNIX_EPOCH, 0, 0),
            (UNIX_EPOCH + Duration::new(3, 456_789_999), 3, 456_789),
            (UNIX_EPOCH - Duration::new(3, 456_789_999), -4, 543_210),
            (UNIX_EPOCH - Duration::from_secs(3), -3, 0),
            (UNIX_EPOCH - Duration::from_nanos(1), -1, 999_999),
        ] {
            let value = timeval::from_system_time(time);
            assert_eq!((value.tv_sec, value.tv_usec), (seconds, microseconds));
        }
    }

    #[test]
    fn components_round_trip_and_mutate() {
        let mut value = timeval::from_time_value(3, 400);
        assert_eq!(value.time_value_seconds(), 3);
        assert_eq!(value.time_value_microseconds(), 400);
        value.set_time_value_seconds(5);
        value.set_time_value_microseconds(600);
        assert_eq!(value.time_value_seconds(), 5);
        assert_eq!(value.time_value_microseconds(), 600);
    }
}
