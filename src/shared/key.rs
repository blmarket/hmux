//! Authoritative key-code scalar domains.

pub type key_code = ::core::ffi::c_ulonglong;

pub type key_code_type = ::core::ffi::c_uint;
pub const KEYC_TYPE_NOTYPE: key_code_type = 13;
pub const KEYC_TYPE_TRIPLECLICK: key_code_type = 12;
pub const KEYC_TYPE_DOUBLECLICK: key_code_type = 11;
pub const KEYC_TYPE_SECONDCLICK: key_code_type = 10;
pub const KEYC_TYPE_WHEELUP: key_code_type = 9;
pub const KEYC_TYPE_WHEELDOWN: key_code_type = 8;
pub const KEYC_TYPE_MOUSEDRAGEND: key_code_type = 7;
pub const KEYC_TYPE_MOUSEDRAG: key_code_type = 6;
pub const KEYC_TYPE_MOUSEUP: key_code_type = 5;
pub const KEYC_TYPE_MOUSEDOWN: key_code_type = 4;
pub const KEYC_TYPE_MOUSEMOVE: key_code_type = 3;
pub const KEYC_TYPE_FUNCTION: key_code_type = 2;
pub const KEYC_TYPE_USER: key_code_type = 1;
pub const KEYC_TYPE_UNICODE: key_code_type = 0;

#[cfg(test)]
mod tests {
    use super::*;
    use ::core::mem::{align_of, size_of};

    #[test]
    fn key_domains_match_translated_c_baseline() {
        assert_eq!(size_of::<key_code>(), 8);
        assert_eq!(align_of::<key_code>(), 8);
        assert_eq!(size_of::<key_code_type>(), 4);
        assert_eq!(align_of::<key_code_type>(), 4);
        assert_eq!(KEYC_TYPE_UNICODE, 0);
        assert_eq!(KEYC_TYPE_NOTYPE, 13);
    }
}
