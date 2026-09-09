//! Stable access to one name-to-key entry from tmux's key-string table.

use crate::text::key_code;
use core::ffi::CStr;

/// One static key name and its encoded key value.
pub trait KeyStringEntry {
    /// Builds a key-string table entry.
    fn from_key_string_entry(name: &'static CStr, key: key_code) -> Self
    where
        Self: Sized;

    /// Returns the key name.
    fn key_string_entry_name(&self) -> &CStr;

    /// Returns the encoded key value.
    fn key_string_entry_key(&self) -> key_code;
}

impl KeyStringEntry for crate::text::key_string_table_entry {
    fn from_key_string_entry(name: &'static CStr, key: key_code) -> Self {
        Self { string: name, key }
    }

    fn key_string_entry_name(&self) -> &CStr {
        self.string
    }

    fn key_string_entry_key(&self) -> key_code {
        self.key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::key_string_table_entry;

    #[test]
    fn name_and_key_round_trip() {
        let entry = key_string_table_entry::from_key_string_entry(c"C-Up", 0x1234);
        assert_eq!(entry.key_string_entry_name(), c"C-Up");
        assert_eq!(entry.key_string_entry_key(), 0x1234);
    }
}
