//! Call a Rust-defined C variadic function directly through its C function type.
use core::ffi::{c_char, c_int, c_long};
use std::ffi::CStr;

#[test]
fn direct_variadic_import_preserves_register_classes_and_va_list_forwarding() {
    let format: unsafe extern "C" fn(*mut *mut c_char, *const c_char, ...) -> c_int =
        hmux2::src::xmalloc::xasprintf;
    let mut output = std::ptr::null_mut();
    unsafe {
        // xasprintf forwards its VaList to xvasprintf and then libc vasprintf.
        // Mix pointer, integer, and floating-point varargs to exercise that ABI.
        let len = format(
            &mut output,
            c"%s:%ld:%.1f".as_ptr(),
            c"direct".as_ptr(),
            42 as c_long,
            2.5_f64,
        );
        assert!(!output.is_null());
        let actual = CStr::from_ptr(output).to_bytes().to_vec();
        hmux2::src::ffi::libc::free(output.cast());
        assert_eq!(actual, b"direct:42:2.5");
        assert_eq!(len as usize, actual.len());
    }
}
