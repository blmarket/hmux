//! Stable access to one parsed argument flag entry.

use crate::types::{ArgsValue, u_char, u_int};
use core::ffi::c_int;

/// One flag, its occurrences, and the values attached to those occurrences.
pub trait ArgumentEntry {
    /// Builds an entry for `flag` with the parser metadata in `flags`.
    fn from_argument_flag(flag: u_char, flags: c_int) -> Self
    where
        Self: Sized;

    /// Returns the flag represented by this entry.
    fn argument_entry_flag(&self) -> u_char;

    /// Returns how many times the flag occurred.
    fn argument_entry_count(&self) -> u_int;

    /// Returns the parser metadata attached when the entry was created.
    fn argument_entry_flags(&self) -> c_int;

    /// Returns the number of values retained for the flag.
    fn argument_entry_value_count(&self) -> usize;

    /// Returns one retained value in insertion order.
    fn argument_entry_value(&self, index: usize) -> Option<&ArgsValue>;

    /// Records one occurrence and retains its value when it is not `NONE`.
    fn add_argument_occurrence(&mut self, value: Option<Box<ArgsValue>>);
}

impl ArgumentEntry for crate::types::args_entry {
    fn from_argument_flag(flag: u_char, flags: c_int) -> Self {
        Self {
            flag,
            values: Vec::new(),
            count: 0,
            flags,
        }
    }
    fn argument_entry_flag(&self) -> u_char {
        self.flag
    }
    fn argument_entry_count(&self) -> u_int {
        self.count
    }
    fn argument_entry_flags(&self) -> c_int {
        self.flags
    }
    fn argument_entry_value_count(&self) -> usize {
        self.values.len()
    }
    fn argument_entry_value(&self, index: usize) -> Option<&ArgsValue> {
        self.values.get(index).map(Box::as_ref)
    }
    fn add_argument_occurrence(&mut self, value: Option<Box<ArgsValue>>) {
        self.count += 1;
        if let Some(value) = value.filter(|value| !matches!(**value, ArgsValue::None)) {
            self.values.push(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::argument_value::ArgumentValue;
    use crate::types::args_entry;
    use std::ffi::CString;

    #[test]
    fn entry_tracks_occurrences_and_nonempty_values() {
        let mut entry = args_entry::from_argument_flag(b'x', 1);
        entry.add_argument_occurrence(None);
        entry.add_argument_occurrence(Some(Box::default()));
        entry.add_argument_occurrence(Some(Box::new(ArgumentValue::from_argument_string(
            CString::new("value").unwrap(),
        ))));

        assert_eq!(entry.argument_entry_flag(), b'x');
        assert_eq!(entry.argument_entry_flags(), 1);
        assert_eq!(entry.argument_entry_count(), 3);
        assert_eq!(entry.argument_entry_value_count(), 1);
        assert_eq!(
            entry
                .argument_entry_value(0)
                .and_then(ArgumentValue::argument_string_value),
            Some(c"value")
        );
        assert!(entry.argument_entry_value(1).is_none());
    }
}
