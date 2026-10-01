use crate::src::ffi::libc::program_invocation_short_name;
pub unsafe fn getprogname() -> *const ::core::ffi::c_char {
    program_invocation_short_name
}
