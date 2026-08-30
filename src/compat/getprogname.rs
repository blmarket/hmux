use crate::ffi::program_invocation_short_name;
use ::core::ffi::CStr;

/// The short (basename) form of the name this process was invoked with, as
/// published by glibc in `program_invocation_short_name`.
fn progname() -> &'static CStr {
    unsafe { CStr::from_ptr(program_invocation_short_name) }
}

pub fn getprogname() -> *const ::core::ffi::c_char {
    progname().as_ptr()
}

#[cfg(test)]
#[path = "../tests/test_compat_getprogname.rs"]
mod tests;
