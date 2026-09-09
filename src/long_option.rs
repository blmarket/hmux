//! Stable access to one long-command-line option descriptor.

use core::ffi::CStr;

/// One entry in a `getopt_long` option table.
///
/// A descriptor cannot outlive its flag storage.
///
/// ```compile_fail
/// use tmux_c2rs::{LongOption, compat::option_t};
/// fn escaped() -> option_t<'static> {
///     let mut flag = 0;
///     option_t::from_long_option(Some(c"quiet"), 0, Some(&mut flag), 1)
/// }
/// ```
///
/// Reading a flag and updating it require separate borrows.
///
/// ```compile_fail
/// use tmux_c2rs::{LongOption, compat::option_t};
/// let mut flag = 0;
/// let mut entry = option_t::from_long_option(Some(c"quiet"), 0, Some(&mut flag), 1);
/// let before = entry.long_option_flag().unwrap();
/// *entry.long_option_flag_mut().unwrap() = 1;
/// assert_eq!(*before, 0);
/// ```
pub trait LongOption<'a> {
    /// Builds a long-option descriptor from its complete portable state.
    fn from_long_option(
        name: Option<&'static CStr>,
        argument_kind: i32,
        flag: Option<&'a mut i32>,
        value: i32,
    ) -> Self;

    /// Returns the long option name, or `None` for the table terminator.
    fn long_option_name(&self) -> Option<&CStr>;

    /// Returns whether and how the option accepts an argument.
    fn long_option_argument_kind(&self) -> i32;

    /// Borrows the flag written instead of returning the option value.
    fn long_option_flag(&self) -> Option<&i32>;

    /// Exclusively borrows the flag for an update.
    fn long_option_flag_mut(&mut self) -> Option<&mut i32>;

    /// Returns the value written or returned when the option is found.
    fn long_option_value(&self) -> i32;
}

impl<'a> LongOption<'a> for crate::compat::option_t<'a> {
    fn from_long_option(
        name: Option<&'static CStr>,
        argument_kind: i32,
        flag: Option<&'a mut i32>,
        value: i32,
    ) -> Self {
        Self {
            name,
            has_arg: argument_kind,
            flag,
            val: value,
        }
    }
    fn long_option_name(&self) -> Option<&CStr> {
        self.name
    }
    fn long_option_argument_kind(&self) -> i32 {
        self.has_arg
    }
    fn long_option_flag(&self) -> Option<&i32> {
        self.flag.as_deref()
    }
    fn long_option_flag_mut(&mut self) -> Option<&mut i32> {
        self.flag.as_deref_mut()
    }
    fn long_option_value(&self) -> i32 {
        self.val
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compat::option_t;

    #[test]
    fn descriptor_and_terminator_round_trip() {
        let mut flag = 0;
        let mut entry = option_t::from_long_option(Some(c"verbose"), 1, Some(&mut flag), 7);
        assert_eq!(entry.long_option_name(), Some(c"verbose"));
        assert_eq!(entry.long_option_argument_kind(), 1);
        assert_eq!(entry.long_option_flag(), Some(&0));
        *entry.long_option_flag_mut().unwrap() = 7;
        assert_eq!(entry.long_option_flag(), Some(&7));
        assert_eq!(entry.long_option_value(), 7);

        let end = option_t::from_long_option(None, 0, None, 0);
        assert_eq!(end.long_option_name(), None);
        assert!(end.long_option_flag().is_none());
    }
}
