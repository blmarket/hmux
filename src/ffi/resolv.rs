//! Foreign declarations supplied by resolv.

use crate::src::shared::abi::size_t;

unsafe extern "C" {
    pub fn __b64_ntop(
        _: *const ::core::ffi::c_uchar,
        _: size_t,
        _: *mut ::core::ffi::c_char,
        _: size_t,
    ) -> ::core::ffi::c_int;
    pub fn __b64_pton(
        _: *const ::core::ffi::c_char,
        _: *mut ::core::ffi::c_uchar,
        _: size_t,
    ) -> ::core::ffi::c_int;
}
