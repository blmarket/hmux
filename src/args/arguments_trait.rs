//! Stable access to a parsed command's arguments.

use crate::types::{ArgsValue, u_char, u_int};
use core::ffi::{CStr, c_int};

/// The observable flag and positional-value state of parsed arguments.
pub trait Arguments {
    /// Returns how many times `flag` was given.
    fn argument_flag_count(&self, flag: u_char) -> c_int;

    /// Adds one occurrence of `flag`, optionally carrying a value.
    fn set_argument_flag(&mut self, flag: u_char, value: Option<ArgsValue>, flags: c_int);

    /// Returns the last string value of `flag` when it has one.
    fn argument_flag_string(&self, flag: u_char) -> Option<&CStr>;

    /// Returns the flags in flag order.
    fn argument_flags(&self) -> Vec<u_char>;

    /// Returns the number of positional values.
    fn argument_count(&self) -> u_int;

    /// Returns all positional values in command-line order.
    fn argument_values(&self) -> &[ArgsValue];

    /// Returns one positional value.
    fn argument_value(&self, index: u_int) -> Option<&ArgsValue> {
        self.argument_values().get(index as usize)
    }

    /// Replaces one positional value with a string.
    fn set_argument_string(&mut self, index: u_int, value: &CStr) -> bool;

    /// Returns one positional value in its string form.
    fn argument_string(&self, index: u_int) -> Option<&CStr>;

    /// Returns every value attached to `flag`, in insertion order.
    fn argument_flag_values(&self, flag: u_char) -> Vec<&ArgsValue>;
}

#[cfg(test)]
mod tests {
    use crate::args::RustArguments;
    use crate::types::ArgsValue;
    use std::ffi::CString;

    #[test]
    fn parsed_arguments_implement_the_contract() {
        let mut arguments = RustArguments::default();
        arguments.set_argument_flag(b'a', None, 0);
        arguments.set_argument_flag(
            b'b',
            Some(ArgsValue::String(CString::new("value").unwrap())),
            0,
        );
        assert_eq!(arguments.argument_flags(), [b'a', b'b']);
        assert_eq!(arguments.argument_flag_count(b'a'), 1);
        assert_eq!(arguments.argument_flag_string(b'b'), Some(c"value"));
    }
}
