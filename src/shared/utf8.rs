//! Authoritative UTF-8 state values.

pub type utf8_state = ::core::ffi::c_uint;
pub const UTF8_ERROR: utf8_state = 2;
pub const UTF8_DONE: utf8_state = 1;
pub const UTF8_MORE: utf8_state = 0;

#[cfg(test)]
mod tests {
    use super::*;
    use ::core::mem::{align_of, size_of};

    #[test]
    fn utf8_state_matches_translated_c_baseline() {
        assert_eq!(size_of::<utf8_state>(), 4);
        assert_eq!(align_of::<utf8_state>(), 4);
        assert_eq!(UTF8_MORE, 0);
        assert_eq!(UTF8_DONE, 1);
        assert_eq!(UTF8_ERROR, 2);
    }
}
