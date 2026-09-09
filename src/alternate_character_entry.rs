//! Stable access to tmux alternate-character-set table records.

use core::ffi::CStr;

/// One mapping between an ACS byte and its UTF-8 rendering.
pub trait AlternateCharacterEntry {
    /// Builds an ACS mapping.
    fn from_alternate_character_entry(key: u8, string: &'static CStr) -> Self
    where
        Self: Sized;

    /// Returns the ACS byte.
    fn alternate_character_key(&self) -> u8;

    /// Returns the UTF-8 rendering.
    fn alternate_character_string(&self) -> &CStr;
}

impl AlternateCharacterEntry for crate::terminfo::tty_acs_entry {
    fn from_alternate_character_entry(key: u8, string: &'static CStr) -> Self {
        Self { key, string }
    }
    fn alternate_character_key(&self) -> u8 {
        self.key
    }
    fn alternate_character_string(&self) -> &CStr {
        self.string
    }
}

impl AlternateCharacterEntry for crate::terminfo::tty_acs_reverse_entry {
    fn from_alternate_character_entry(key: u8, string: &'static CStr) -> Self {
        Self { string, key }
    }
    fn alternate_character_key(&self) -> u8 {
        self.key
    }
    fn alternate_character_string(&self) -> &CStr {
        self.string
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminfo::{tty_acs_entry, tty_acs_reverse_entry};

    fn assert_mapping<E: AlternateCharacterEntry>() {
        let entry = E::from_alternate_character_entry(b'q', c"─");
        assert_eq!(entry.alternate_character_key(), b'q');
        assert_eq!(entry.alternate_character_string(), c"─");
    }

    #[test]
    fn forward_and_reverse_records_expose_the_same_mapping() {
        assert_mapping::<tty_acs_entry>();
        assert_mapping::<tty_acs_reverse_entry>();
    }
}
