//! Authoritative argument-domain values.
//!
//! The recursive `args` model and its callback remain private to the
//! translation units until their mutually recursive model family is migrated.

pub type args_type = ::core::ffi::c_uint;
pub const ARGS_COMMANDS: args_type = 2;
pub const ARGS_STRING: args_type = 1;
pub const ARGS_NONE: args_type = 0;

pub type args_parse_type = ::core::ffi::c_uint;
pub const ARGS_PARSE_COMMANDS: args_parse_type = 3;
pub const ARGS_PARSE_COMMANDS_OR_STRING: args_parse_type = 2;
pub const ARGS_PARSE_STRING: args_parse_type = 1;
pub const ARGS_PARSE_INVALID: args_parse_type = 0;

#[cfg(test)]
mod tests {
    use super::*;
    use ::core::mem::{align_of, size_of};

    #[test]
    fn argument_domains_match_translated_c_baseline() {
        assert_eq!(size_of::<args_type>(), 4);
        assert_eq!(align_of::<args_type>(), 4);
        assert_eq!(size_of::<args_parse_type>(), 4);
        assert_eq!(align_of::<args_parse_type>(), 4);
        assert_eq!(ARGS_NONE, 0);
        assert_eq!(ARGS_COMMANDS, 2);
        assert_eq!(ARGS_PARSE_INVALID, 0);
        assert_eq!(ARGS_PARSE_COMMANDS, 3);
    }
}
