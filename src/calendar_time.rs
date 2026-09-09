//! Stable access to a broken-down calendar time.

use core::ffi::CStr;
use std::rc::Rc;

/// A broken-down local calendar time in the operating-system shape.
pub trait CalendarTime: Default {
    /// Builds a calendar time retaining an owned copy of the optional timezone.
    #[allow(clippy::too_many_arguments)]
    fn from_calendar_time(
        seconds: i32,
        minutes: i32,
        hour: i32,
        month_day: i32,
        month: i32,
        year: i32,
        week_day: i32,
        year_day: i32,
        is_dst: i32,
        gmt_offset: i64,
        zone: Option<&CStr>,
    ) -> Self
    where
        Self: Sized;

    fn calendar_seconds(&self) -> i32;
    fn calendar_minutes(&self) -> i32;
    fn calendar_hour(&self) -> i32;
    fn calendar_month_day(&self) -> i32;
    fn calendar_month(&self) -> i32;
    fn calendar_year(&self) -> i32;
    fn calendar_week_day(&self) -> i32;
    fn calendar_year_day(&self) -> i32;
    fn calendar_is_dst(&self) -> i32;
    fn calendar_gmt_offset(&self) -> i64;
    fn calendar_zone(&self) -> Option<&CStr>;
}

impl CalendarTime for crate::types::tm {
    fn from_calendar_time(
        seconds: i32,
        minutes: i32,
        hour: i32,
        month_day: i32,
        month: i32,
        year: i32,
        week_day: i32,
        year_day: i32,
        is_dst: i32,
        gmt_offset: i64,
        zone: Option<&CStr>,
    ) -> Self {
        Self {
            tm_sec: seconds,
            tm_min: minutes,
            tm_hour: hour,
            tm_mday: month_day,
            tm_mon: month,
            tm_year: year,
            tm_wday: week_day,
            tm_yday: year_day,
            tm_isdst: is_dst,
            tm_gmtoff: gmt_offset,
            tm_zone: zone.map(Rc::from),
        }
    }

    fn calendar_seconds(&self) -> i32 {
        self.tm_sec
    }
    fn calendar_minutes(&self) -> i32 {
        self.tm_min
    }
    fn calendar_hour(&self) -> i32 {
        self.tm_hour
    }
    fn calendar_month_day(&self) -> i32 {
        self.tm_mday
    }
    fn calendar_month(&self) -> i32 {
        self.tm_mon
    }
    fn calendar_year(&self) -> i32 {
        self.tm_year
    }
    fn calendar_week_day(&self) -> i32 {
        self.tm_wday
    }
    fn calendar_year_day(&self) -> i32 {
        self.tm_yday
    }
    fn calendar_is_dst(&self) -> i32 {
        self.tm_isdst
    }
    fn calendar_gmt_offset(&self) -> i64 {
        self.tm_gmtoff
    }
    fn calendar_zone(&self) -> Option<&CStr> {
        self.tm_zone.as_deref()
    }
}

impl crate::types::tm {
    /// Converts a timestamp to local calendar components and retains its timezone.
    pub fn local(timestamp: crate::types::time_t) -> Option<Self> {
        let mut native: libc::tm = unsafe { core::mem::zeroed() };
        if unsafe { libc::localtime_r(&timestamp, &mut native) }.is_null() {
            return None;
        }
        Some(Self::from_calendar_time(
            native.tm_sec,
            native.tm_min,
            native.tm_hour,
            native.tm_mday,
            native.tm_mon,
            native.tm_year,
            native.tm_wday,
            native.tm_yday,
            native.tm_isdst,
            native.tm_gmtoff,
            (!native.tm_zone.is_null()).then(|| unsafe { CStr::from_ptr(native.tm_zone) }),
        ))
    }

    /// Formats the calendar into a bounded buffer using the retained timezone.
    ///
    /// # Safety
    ///
    /// Calendar components must satisfy libc's requirements for the conversions
    /// requested by `format`.
    pub(crate) unsafe fn format_into(&self, buffer: &mut [u8], format: &CStr) -> usize {
        let native = libc::tm {
            tm_sec: self.tm_sec,
            tm_min: self.tm_min,
            tm_hour: self.tm_hour,
            tm_mday: self.tm_mday,
            tm_mon: self.tm_mon,
            tm_year: self.tm_year,
            tm_wday: self.tm_wday,
            tm_yday: self.tm_yday,
            tm_isdst: self.tm_isdst,
            tm_gmtoff: self.tm_gmtoff,
            tm_zone: self
                .tm_zone
                .as_deref()
                .map_or(core::ptr::null(), CStr::as_ptr),
        };
        unsafe {
            libc::strftime(
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                format.as_ptr(),
                &native,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::tm;

    #[test]
    fn timezone_survives_its_source_and_calendar_clone() {
        let zone = c"PST".to_owned();
        let value = tm::from_calendar_time(1, 2, 3, 4, 5, 126, 6, 7, 0, -28_800, Some(&zone));
        drop(zone);
        let clone = value.clone();
        assert!(Rc::ptr_eq(
            value.tm_zone.as_ref().unwrap(),
            clone.tm_zone.as_ref().unwrap()
        ));
        drop(value);
        assert_eq!(clone.calendar_zone(), Some(c"PST"));
        let mut bytes = [0; 64];
        let length = unsafe { clone.format_into(&mut bytes, c"%Z %z") };
        assert_eq!(&bytes[..length], b"PST -0800");
    }

    #[test]
    fn local_calendar_can_be_formatted_after_another_conversion() {
        let value = tm::local(1_700_000_000).unwrap();
        let mut before = [0; 128];
        let before_length = unsafe { value.format_into(&mut before, c"%Y-%m-%d %H:%M:%S %Z %z") };
        let _other = tm::local(0).unwrap();
        let mut after = [0; 128];
        let after_length = unsafe { value.format_into(&mut after, c"%Y-%m-%d %H:%M:%S %Z %z") };
        assert_ne!(before_length, 0);
        assert_eq!(&before[..before_length], &after[..after_length]);
        assert_eq!(unsafe { value.format_into(&mut [], c"%Y") }, 0);
    }

    #[test]
    fn every_component_round_trips() {
        let value = tm::from_calendar_time(1, 2, 3, 4, 5, 126, 6, 7, 1, -28_800, Some(c"PST"));
        assert_eq!(value.calendar_seconds(), 1);
        assert_eq!(value.calendar_minutes(), 2);
        assert_eq!(value.calendar_hour(), 3);
        assert_eq!(value.calendar_month_day(), 4);
        assert_eq!(value.calendar_month(), 5);
        assert_eq!(value.calendar_year(), 126);
        assert_eq!(value.calendar_week_day(), 6);
        assert_eq!(value.calendar_year_day(), 7);
        assert_eq!(value.calendar_is_dst(), 1);
        assert_eq!(value.calendar_gmt_offset(), -28_800);
        assert_eq!(value.calendar_zone(), Some(c"PST"));
    }
}
