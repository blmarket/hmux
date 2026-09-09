use crate::fmt_engine::{FmtArg, format_alloc};
pub use crate::types::*;
use ::std::ffi::{CStr, CString};

pub fn xasprintf(fmt: &CStr, args: &[FmtArg]) -> CString {
    format_alloc(fmt, args)
}

#[cfg(test)]
#[path = "tests/test_xmalloc.rs"]
mod tests;
