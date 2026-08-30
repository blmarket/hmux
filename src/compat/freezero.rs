use crate::ffi::free;
pub use crate::types::*;
pub unsafe fn freezero(mut ptr: *mut u8, mut size: size_t) {
    unsafe {
        if !ptr.is_null() {
            ::core::ptr::write_bytes(ptr, 0, size);
            free(ptr as *mut ::core::ffi::c_void);
        }
    }
}
