//! Authoritative option-table value domains.

pub type options_table_type = ::core::ffi::c_uint;
pub const OPTIONS_TABLE_NUMBER: options_table_type = 1;
pub const OPTIONS_TABLE_KEY: options_table_type = 2;
pub const OPTIONS_TABLE_COLOUR: options_table_type = 3;
pub const OPTIONS_TABLE_FLAG: options_table_type = 4;
pub const OPTIONS_TABLE_CHOICE: options_table_type = 5;
pub const OPTIONS_TABLE_COMMAND: options_table_type = 6;
pub const OPTIONS_TABLE_STRING: options_table_type = 0;

#[cfg(test)]
mod tests {
    use super::*;
    use ::core::mem::{align_of, size_of};

    #[test]
    fn option_table_domain_matches_translated_c_baseline() {
        assert_eq!(size_of::<options_table_type>(), 4);
        assert_eq!(align_of::<options_table_type>(), 4);
        assert_eq!(OPTIONS_TABLE_STRING, 0);
        assert_eq!(OPTIONS_TABLE_COMMAND, 6);
    }
}
