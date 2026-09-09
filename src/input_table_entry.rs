//! Stable access to an input sequence table entry.

use core::ffi::{CStr, c_int};

/// A final byte, intermediate bytes, and the parser action they select.
pub trait InputTableEntry {
    /// Builds an input sequence table entry.
    fn from_input_table_entry(
        character: c_int,
        intermediates: &'static CStr,
        input_type: c_int,
    ) -> Self
    where
        Self: Sized;

    /// Returns the sequence's final byte.
    fn input_table_character(&self) -> c_int;

    /// Returns the intermediate bytes before the final byte.
    fn input_table_intermediates(&self) -> &CStr;

    /// Returns the parser action selected by the sequence.
    fn input_table_type(&self) -> c_int;
}

impl InputTableEntry for crate::types::input_table_entry {
    fn from_input_table_entry(
        character: c_int,
        intermediates: &'static CStr,
        input_type: c_int,
    ) -> Self {
        Self {
            ch: character,
            interm: intermediates,
            type_0: input_type,
        }
    }
    fn input_table_character(&self) -> c_int {
        self.ch
    }
    fn input_table_intermediates(&self) -> &CStr {
        self.interm
    }
    fn input_table_type(&self) -> c_int {
        self.type_0
    }
}

impl crate::types::input_table_entry {
    /// Builds an entry usable in the input parser's static dispatch tables.
    pub const fn new(character: c_int, intermediates: &'static CStr, input_type: c_int) -> Self {
        Self {
            ch: character,
            interm: intermediates,
            type_0: input_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::input_table_entry;

    #[test]
    fn const_entry_exposes_its_sequence() {
        const ENTRY: input_table_entry = input_table_entry::new(b'Z' as c_int, c"?", 17);
        assert_eq!(ENTRY.input_table_character(), b'Z' as c_int);
        assert_eq!(ENTRY.input_table_intermediates(), c"?");
        assert_eq!(ENTRY.input_table_type(), 17);
    }
}
