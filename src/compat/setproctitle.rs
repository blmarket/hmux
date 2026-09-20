use crate::src::shared::abi::*;
extern "C" {
    fn strrchr(
        __s: *const ::core::ffi::c_char,
        __c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn vsnprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        __arg: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;
    fn getprogname() -> *const ::core::ffi::c_char;
    fn prctl(__option: ::core::ffi::c_int, ...) -> ::core::ffi::c_int;
}
pub type __builtin_va_list = [__va_list_tag; 1];
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __va_list_tag {
    pub gp_offset: ::core::ffi::c_uint,
    pub fp_offset: ::core::ffi::c_uint,
    pub overflow_arg_area: *mut ::core::ffi::c_void,
    pub reg_save_area: *mut ::core::ffi::c_void,
}
pub type va_list = __builtin_va_list;
pub const PR_SET_NAME: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn setproctitle(mut fmt: *const ::core::ffi::c_char, mut args: ...) {
    let mut title: [::core::ffi::c_char; 16] = [0; 16];
    let mut name: [::core::ffi::c_char; 16] = [0; 16];
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ap: ::core::ffi::VaList;
    let mut used: ::core::ffi::c_int = 0;
    ap = args.clone();
    vsnprintf(
        &raw mut title as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as size_t,
        fmt,
        ap,
    );
    used = snprintf(
        &raw mut name as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as size_t,
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        getprogname(),
        &raw mut title as *mut ::core::ffi::c_char,
    );
    if used >= ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as ::core::ffi::c_int {
        cp = strrchr(&raw mut name as *mut ::core::ffi::c_char, ' ' as i32);
        if !cp.is_null() {
            *cp = '\0' as i32 as ::core::ffi::c_char;
        }
    }
    prctl(PR_SET_NAME, &raw mut name as *mut ::core::ffi::c_char);
}
