//! Foreign declarations supplied by utf8proc.

use crate::src::shared::abi::{__int32_t, uint8_t};
pub type int32_t = __int32_t;

pub type ptrdiff_t = isize;

pub type utf8proc_uint8_t = uint8_t;

pub type utf8proc_int32_t = int32_t;

pub type utf8proc_ssize_t = ptrdiff_t;

pub type utf8proc_bool = bool;

pub type utf8proc_category_t = ::core::ffi::c_uint;

extern "C" {
    pub fn utf8proc_category(codepoint: utf8proc_int32_t) -> utf8proc_category_t;
    pub fn utf8proc_charwidth(codepoint: utf8proc_int32_t) -> ::core::ffi::c_int;
    pub fn utf8proc_codepoint_valid(codepoint: utf8proc_int32_t) -> utf8proc_bool;
    pub fn utf8proc_encode_char(
        codepoint: utf8proc_int32_t,
        dst: *mut utf8proc_uint8_t,
    ) -> utf8proc_ssize_t;
    pub fn utf8proc_iterate(
        str: *const utf8proc_uint8_t,
        strlen: utf8proc_ssize_t,
        codepoint_ref: *mut utf8proc_int32_t,
    ) -> utf8proc_ssize_t;
    pub fn utf8proc_version() -> *const ::core::ffi::c_char;
}
