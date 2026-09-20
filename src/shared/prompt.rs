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

pub const PROMPT_CLOSE: prompt_result = 1;
pub const PROMPT_CONTINUE: prompt_result = 0;
pub const PROMPT_SINGLE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PROMPT_NUMERIC: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const PROMPT_INCREMENTAL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const PROMPT_KEY: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const PROMPT_BSPACE_EXIT: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const PROMPT_NOFREEZE: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const PROMPT_ISPANE: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const PROMPT_NTYPES: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const PROMPT_NOFORMAT: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const PROMPT_ACCEPT: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const PROMPT_ISMODE: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const PROMPT_QUOTENEXT: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const PROMPT_COMMANDMODE: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const PROMPT_EDITARROWS: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;

pub type prompt_result = ::core::ffi::c_uint;

pub type prompt_free_cb = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
pub type prompt_input_cb = Option<
    unsafe extern "C" fn(
        *mut ::core::ffi::c_void,
        *const ::core::ffi::c_char,
        prompt_key_result,
    ) -> prompt_result,
>;

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
