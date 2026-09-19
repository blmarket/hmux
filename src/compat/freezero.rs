extern "C" {
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
}
pub type size_t = usize;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[no_mangle]
pub unsafe extern "C" fn freezero(mut ptr: *mut ::core::ffi::c_void, mut size: size_t) {
    if !ptr.is_null() {
        memset(ptr, 0 as ::core::ffi::c_int, size);
        free(ptr);
    }
}
