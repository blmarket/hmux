//! Calendar-time ABI declarations and conversions for Rust timestamps.
use super::abi::{__syscall_slong_t, __time_t};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct tm {
    pub tm_sec: ::core::ffi::c_int,
    pub tm_min: ::core::ffi::c_int,
    pub tm_hour: ::core::ffi::c_int,
    pub tm_mday: ::core::ffi::c_int,
    pub tm_mon: ::core::ffi::c_int,
    pub tm_year: ::core::ffi::c_int,
    pub tm_wday: ::core::ffi::c_int,
    pub tm_yday: ::core::ffi::c_int,
    pub tm_isdst: ::core::ffi::c_int,
    pub tm_gmtoff: ::core::ffi::c_long,
    pub tm_zone: *const ::core::ffi::c_char,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct timespec {
    pub tv_sec: __time_t,
    pub tv_nsec: __syscall_slong_t,
}

pub const CLOCK_REALTIME: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

/// Whole Unix seconds for the remaining calendar-time formatting interfaces.
/// Fractional times before the epoch round down, like the former C timestamp field.
pub fn unix_seconds(time: SystemTime) -> super::abi::time_t {
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() as super::abi::time_t,
        Err(error) => {
            let duration = error.duration();
            -(duration.as_secs() as super::abi::time_t) - i64::from(duration.subsec_nanos() != 0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn calendar_seconds_round_down_on_both_sides_of_epoch() {
        assert_eq!(unix_seconds(UNIX_EPOCH), 0);
        assert_eq!(
            unix_seconds(UNIX_EPOCH + Duration::from_micros(1_999_999)),
            1
        );
        assert_eq!(unix_seconds(UNIX_EPOCH - Duration::from_micros(1)), -1);
        assert_eq!(unix_seconds(UNIX_EPOCH - Duration::from_secs(1)), -1);
        assert_eq!(
            unix_seconds(UNIX_EPOCH - Duration::from_micros(1_000_001)),
            -2
        );
    }
}
