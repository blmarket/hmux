use crate::ffi::program_invocation_short_name;
use ::core::ffi::CStr;
use ::std::ffi::CString;

/// The short (basename) form of the name this process was invoked with, as
/// published by glibc in `program_invocation_short_name`, copied for the caller.
pub fn getprogname() -> CString {
    unsafe { CStr::from_ptr(program_invocation_short_name).to_owned() }
}

#[cfg(test)]
#[path = "../tests/test_compat_getprogname.rs"]
mod tests;
