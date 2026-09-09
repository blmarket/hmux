//! Stable access to accepted aliases for option names.

use core::ffi::CStr;

/// An accepted option spelling and the canonical name it maps to.
pub trait OptionNameMap {
    /// Builds an option-name mapping.
    fn from_option_name_map(from: &'static CStr, to: &'static CStr) -> Self
    where
        Self: Sized;

    /// Returns the accepted alias.
    fn option_name_from(&self) -> &'static CStr;

    /// Returns the canonical option name.
    fn option_name_to(&self) -> &'static CStr;
}

impl OptionNameMap for crate::types::options_name_map {
    fn from_option_name_map(from: &'static CStr, to: &'static CStr) -> Self {
        Self { from, to }
    }
    fn option_name_from(&self) -> &'static CStr {
        self.from
    }
    fn option_name_to(&self) -> &'static CStr {
        self.to
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::options_name_map;

    #[test]
    fn alias_and_canonical_name_round_trip() {
        let value = options_name_map::from_option_name_map(c"color", c"colour");
        assert_eq!(value.option_name_from(), c"color");
        assert_eq!(value.option_name_to(), c"colour");
    }
}
