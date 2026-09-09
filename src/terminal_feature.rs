//! Stable access to terminal-feature table records.

use core::ffi::{CStr, c_int};

/// One named terminal feature and the capabilities and flags it enables.
pub trait TerminalFeature {
    /// Builds a terminal feature.
    fn from_terminal_feature(
        name: &'static CStr,
        capabilities: &'static [&'static CStr],
        flags: c_int,
    ) -> Self
    where
        Self: Sized;

    /// Returns the feature name.
    fn terminal_feature_name(&self) -> &CStr;

    /// Returns the number of terminfo capabilities supplied by the feature.
    fn terminal_feature_capability_count(&self) -> usize;

    /// Returns one terminfo capability, or `None` when `index` is out of range.
    fn terminal_feature_capability(&self, index: usize) -> Option<&CStr>;

    /// Returns the terminal flags enabled by the feature.
    fn terminal_feature_flags(&self) -> c_int;
}

impl TerminalFeature for crate::terminfo::tty_feature {
    fn from_terminal_feature(
        name: &'static CStr,
        capabilities: &'static [&'static CStr],
        flags: c_int,
    ) -> Self {
        Self {
            name,
            capabilities,
            flags,
        }
    }
    fn terminal_feature_name(&self) -> &CStr {
        self.name
    }
    fn terminal_feature_capability_count(&self) -> usize {
        self.capabilities.len()
    }
    fn terminal_feature_capability(&self, index: usize) -> Option<&CStr> {
        self.capabilities.get(index).copied()
    }
    fn terminal_feature_flags(&self) -> c_int {
        self.flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminfo::tty_feature;

    #[test]
    fn terminal_feature_exposes_every_field() {
        let feature = tty_feature::from_terminal_feature(c"sample", &[c"AX", c"Rect"], 0x18);
        assert_eq!(feature.terminal_feature_name(), c"sample");
        assert_eq!(feature.terminal_feature_capability_count(), 2);
        assert_eq!(feature.terminal_feature_capability(0), Some(c"AX"));
        assert_eq!(feature.terminal_feature_capability(1), Some(c"Rect"));
        assert_eq!(feature.terminal_feature_capability(2), None);
        assert_eq!(feature.terminal_feature_flags(), 0x18);
    }
}
