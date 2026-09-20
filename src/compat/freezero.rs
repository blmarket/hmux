use crate::src::ffi::libc::{free, memset};
use crate::src::shared::abi::*;

#[no_mangle]
pub unsafe extern "C" fn freezero(mut ptr: *mut ::core::ffi::c_void, mut size: size_t) {
    if !ptr.is_null() {
        memset(ptr, 0 as ::core::ffi::c_int, size);
        free(ptr);
    }
}
