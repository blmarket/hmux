//! Text conversion for command arguments.

use core::ffi::c_longlong;
use std::ffi::{CStr, CString};

/// Quoting and numeric conversion for command argument text.
pub trait ArgumentTextCodec {
    /// Quotes one argument for command text.
    fn escape(&self, value: &CStr) -> CString;

    /// Parses a number or a percentage of `current` within the allowed range.
    fn percentage(
        &self,
        value: &CStr,
        minimum: c_longlong,
        maximum: c_longlong,
        current: c_longlong,
        cause: &mut Option<CString>,
    ) -> c_longlong;
}

/// Argument text conversion implemented by hmux.
#[derive(Clone, Copy, Debug, Default)]
pub struct RustArgumentTextCodec;

impl ArgumentTextCodec for RustArgumentTextCodec {
    fn escape(&self, value: &CStr) -> CString {
        crate::arguments::args_escape_impl(value)
    }

    fn percentage(
        &self,
        value: &CStr,
        minimum: c_longlong,
        maximum: c_longlong,
        current: c_longlong,
        cause: &mut Option<CString>,
    ) -> c_longlong {
        crate::arguments::args_string_percentage_impl(value, minimum, maximum, current, cause)
    }
}
