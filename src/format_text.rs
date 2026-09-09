//! Width and trimming operations for expanded format text.

use crate::types::u_int;
use std::ffi::CString;

/// Display-column operations that preserve embedded style directives.
pub trait FormatText {
    /// Returns the display width with style directives excluded.
    fn width(&self, expanded: &[u8]) -> u_int;

    /// Returns the first `limit` display columns with styles preserved.
    fn trim_left(&self, expanded: &[u8], limit: u_int) -> CString;

    /// Returns the last `limit` display columns with styles preserved.
    fn trim_right(&self, expanded: &[u8], limit: u_int) -> CString;
}

/// Expanded-format text operations implemented by hmux.
#[derive(Clone, Copy, Debug, Default)]
pub struct RustFormatText;

impl FormatText for RustFormatText {
    fn width(&self, expanded: &[u8]) -> u_int {
        crate::screen::format_width_impl(expanded)
    }

    fn trim_left(&self, expanded: &[u8], limit: u_int) -> CString {
        crate::screen::format_trim_left_impl(expanded, limit)
    }

    fn trim_right(&self, expanded: &[u8], limit: u_int) -> CString {
        crate::screen::format_trim_right_impl(expanded, limit)
    }
}
