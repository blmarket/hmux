//! Stable access to one parsed format modifier.

use core::ffi::CStr;
use std::ffi::CString;

/// A format modifier and the arguments parsed for it.
pub trait FormatModifier {
    /// Builds a parsed modifier, copying its one- or two-byte name.
    fn from_format_modifier(name: &CStr, arguments: Vec<CString>) -> Self
    where
        Self: Sized;

    /// Returns the modifier name.
    fn format_modifier_name(&self) -> &CStr;

    /// Returns the number of bytes in the modifier name.
    fn format_modifier_size(&self) -> u32;

    /// Returns the arguments supplied to the modifier.
    fn format_modifier_arguments(&self) -> &[CString];
}

impl FormatModifier for crate::types::format_modifier {
    fn from_format_modifier(name: &CStr, arguments: Vec<CString>) -> Self {
        assert!(name.to_bytes().len() <= 2);
        let mut modifier = [0; 3];
        modifier[..name.count_bytes()].copy_from_slice(name.to_bytes());
        Self {
            modifier,
            size: name.to_bytes().len() as u32,
            argv: arguments,
        }
    }

    fn format_modifier_name(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.modifier).expect("format modifier name is NUL terminated")
    }

    fn format_modifier_size(&self) -> u32 {
        self.size
    }
    fn format_modifier_arguments(&self) -> &[CString] {
        &self.argv
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::format_modifier;

    #[test]
    fn modifier_and_arguments_round_trip() {
        let value = format_modifier::from_format_modifier(
            c"||",
            vec![c"left".to_owned(), c"right".to_owned()],
        );
        assert_eq!(value.format_modifier_name(), c"||");
        assert_eq!(value.format_modifier_size(), 2);
        assert_eq!(
            value.format_modifier_arguments(),
            &[c"left".to_owned(), c"right".to_owned()]
        );
    }
}
