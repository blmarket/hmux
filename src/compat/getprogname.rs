use crate::src::ffi::libc::program_invocation_short_name;
pub unsafe fn getprogname() -> &'static std::ffi::CStr {
    std::ffi::CStr::from_ptr(program_invocation_short_name)
}
