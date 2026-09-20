use crate::src::ffi::libc::program_invocation_short_name;
#[no_mangle]
pub unsafe extern "C" fn getprogname() -> *const ::core::ffi::c_char {
    return program_invocation_short_name;
}
