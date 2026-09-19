extern "C" {
    static mut program_invocation_short_name: *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn getprogname() -> *const ::core::ffi::c_char {
    return program_invocation_short_name;
}
