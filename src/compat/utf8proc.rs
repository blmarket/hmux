use crate::src::shared::abi::*;
extern "C" {
    fn utf8proc_iterate(
        str: *const utf8proc_uint8_t,
        strlen: utf8proc_ssize_t,
        codepoint_ref: *mut utf8proc_int32_t,
    ) -> utf8proc_ssize_t;
    fn utf8proc_codepoint_valid(codepoint: utf8proc_int32_t) -> utf8proc_bool;
    fn utf8proc_encode_char(
        codepoint: utf8proc_int32_t,
        dst: *mut utf8proc_uint8_t,
    ) -> utf8proc_ssize_t;
    fn utf8proc_charwidth(codepoint: utf8proc_int32_t) -> ::core::ffi::c_int;
    fn utf8proc_category(codepoint: utf8proc_int32_t) -> utf8proc_category_t;
}
pub type __int32_t = i32;
pub type int32_t = __int32_t;
pub type wchar_t = ::libc::wchar_t;
pub type ptrdiff_t = isize;
pub type utf8proc_uint8_t = uint8_t;
pub type utf8proc_int32_t = int32_t;
pub type utf8proc_ssize_t = ptrdiff_t;
pub type utf8proc_bool = bool;
pub type utf8proc_category_t = ::core::ffi::c_uint;
pub const UTF8PROC_CATEGORY_CO: utf8proc_category_t = 29;
pub const UTF8PROC_CATEGORY_CS: utf8proc_category_t = 28;
pub const UTF8PROC_CATEGORY_CF: utf8proc_category_t = 27;
pub const UTF8PROC_CATEGORY_CC: utf8proc_category_t = 26;
pub const UTF8PROC_CATEGORY_ZP: utf8proc_category_t = 25;
pub const UTF8PROC_CATEGORY_ZL: utf8proc_category_t = 24;
pub const UTF8PROC_CATEGORY_ZS: utf8proc_category_t = 23;
pub const UTF8PROC_CATEGORY_SO: utf8proc_category_t = 22;
pub const UTF8PROC_CATEGORY_SK: utf8proc_category_t = 21;
pub const UTF8PROC_CATEGORY_SC: utf8proc_category_t = 20;
pub const UTF8PROC_CATEGORY_SM: utf8proc_category_t = 19;
pub const UTF8PROC_CATEGORY_PO: utf8proc_category_t = 18;
pub const UTF8PROC_CATEGORY_PF: utf8proc_category_t = 17;
pub const UTF8PROC_CATEGORY_PI: utf8proc_category_t = 16;
pub const UTF8PROC_CATEGORY_PE: utf8proc_category_t = 15;
pub const UTF8PROC_CATEGORY_PS: utf8proc_category_t = 14;
pub const UTF8PROC_CATEGORY_PD: utf8proc_category_t = 13;
pub const UTF8PROC_CATEGORY_PC: utf8proc_category_t = 12;
pub const UTF8PROC_CATEGORY_NO: utf8proc_category_t = 11;
pub const UTF8PROC_CATEGORY_NL: utf8proc_category_t = 10;
pub const UTF8PROC_CATEGORY_ND: utf8proc_category_t = 9;
pub const UTF8PROC_CATEGORY_ME: utf8proc_category_t = 8;
pub const UTF8PROC_CATEGORY_MC: utf8proc_category_t = 7;
pub const UTF8PROC_CATEGORY_MN: utf8proc_category_t = 6;
pub const UTF8PROC_CATEGORY_LO: utf8proc_category_t = 5;
pub const UTF8PROC_CATEGORY_LM: utf8proc_category_t = 4;
pub const UTF8PROC_CATEGORY_LT: utf8proc_category_t = 3;
pub const UTF8PROC_CATEGORY_LL: utf8proc_category_t = 2;
pub const UTF8PROC_CATEGORY_LU: utf8proc_category_t = 1;
pub const UTF8PROC_CATEGORY_CN: utf8proc_category_t = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[no_mangle]
pub unsafe extern "C" fn utf8proc_wcwidth(mut wc: wchar_t) -> ::core::ffi::c_int {
    let mut cat: ::core::ffi::c_int = 0;
    cat = utf8proc_category(wc as utf8proc_int32_t) as ::core::ffi::c_int;
    if cat == UTF8PROC_CATEGORY_CO as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    return utf8proc_charwidth(wc as utf8proc_int32_t);
}
#[no_mangle]
pub unsafe extern "C" fn utf8proc_mbtowc(
    mut pwc: *mut wchar_t,
    mut s: *const ::core::ffi::c_char,
    mut n: size_t,
) -> ::core::ffi::c_int {
    let mut slen: utf8proc_ssize_t = 0;
    if s.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    slen = utf8proc_iterate(
        s as *const utf8proc_uint8_t,
        n as utf8proc_ssize_t,
        pwc as *mut utf8proc_int32_t,
    );
    if *pwc == -(1 as ::core::ffi::c_int) as wchar_t || slen < 0 as utf8proc_ssize_t {
        return -(1 as ::core::ffi::c_int);
    }
    return slen as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn utf8proc_wctomb(
    mut s: *mut ::core::ffi::c_char,
    mut wc: wchar_t,
) -> ::core::ffi::c_int {
    if s.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if !utf8proc_codepoint_valid(wc as utf8proc_int32_t) {
        return -(1 as ::core::ffi::c_int);
    }
    return utf8proc_encode_char(wc as utf8proc_int32_t, s as *mut utf8proc_uint8_t)
        as ::core::ffi::c_int;
}
