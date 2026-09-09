//! Stable access to a seconds-and-nanoseconds timestamp.

/// A timestamp or duration in the operating-system `timespec` shape.
pub trait TimeSpec {
    /// Builds a value from whole seconds and the nanosecond remainder.
    fn from_time_spec(seconds: i64, nanoseconds: i64) -> Self
    where
        Self: Sized;

    /// Returns the whole-second component.
    fn time_spec_seconds(&self) -> i64;

    /// Returns the nanosecond component.
    fn time_spec_nanoseconds(&self) -> i64;
}

impl TimeSpec for crate::types::timespec {
    fn from_time_spec(seconds: i64, nanoseconds: i64) -> Self {
        Self {
            tv_sec: seconds,
            tv_nsec: nanoseconds,
        }
    }

    fn time_spec_seconds(&self) -> i64 {
        self.tv_sec
    }

    fn time_spec_nanoseconds(&self) -> i64 {
        self.tv_nsec
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::timespec;

    #[test]
    fn seconds_and_nanoseconds_round_trip() {
        let time = timespec::from_time_spec(17, 250_000_000);
        assert_eq!(time.time_spec_seconds(), 17);
        assert_eq!(time.time_spec_nanoseconds(), 250_000_000);
    }
}
