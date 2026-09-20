//! Authoritative prompt domains.

pub type prompt_type = ::core::ffi::c_uint;
pub const PROMPT_TYPE_COMMAND: prompt_type = 0;
pub const PROMPT_TYPE_INVALID: prompt_type = 255;
pub const PROMPT_TYPE_SEARCH: prompt_type = 1;

pub type prompt_key_result = ::core::ffi::c_uint;
pub const PROMPT_KEY_NOT_HANDLED: prompt_key_result = 0;
pub const PROMPT_KEY_HANDLED: prompt_key_result = 1;
pub const PROMPT_KEY_CLOSE: prompt_key_result = 2;
pub const PROMPT_KEY_MOVE: prompt_key_result = 3;

#[cfg(test)]
mod tests {
    use super::*;
    use ::core::mem::{align_of, size_of};

    #[test]
    fn prompt_domains_match_translated_c_baseline() {
        assert_eq!(size_of::<prompt_type>(), 4);
        assert_eq!(align_of::<prompt_type>(), 4);
        assert_eq!(size_of::<prompt_key_result>(), 4);
        assert_eq!(align_of::<prompt_key_result>(), 4);
        assert_eq!(PROMPT_TYPE_COMMAND, 0);
        assert_eq!(PROMPT_TYPE_SEARCH, 1);
        assert_eq!(PROMPT_TYPE_INVALID, 255);
        assert_eq!(PROMPT_KEY_NOT_HANDLED, 0);
        assert_eq!(PROMPT_KEY_MOVE, 3);
    }
}
