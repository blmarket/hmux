//! Source location retained by a command.

use crate::types::u_int;
use core::ffi::CStr;

/// A command's optional source file and its line number.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CommandSource<'a> {
    pub file: Option<&'a CStr>,
    pub line: u_int,
}
