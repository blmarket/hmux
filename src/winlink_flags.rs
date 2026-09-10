//! Flags carried by a window link.

use core::ffi::c_int;

/// Storage and mask operations for window-link flags.
pub trait WinlinkFlagsState {
    /// Returns every flag currently set on the link.
    fn winlink_flags(&self) -> c_int;

    /// Replaces every flag on the link.
    fn set_winlink_flags(&mut self, flags: c_int);

    /// Reports whether any bit in `flags` is set.
    fn has_winlink_flags(&self, flags: c_int) -> bool {
        self.winlink_flags() & flags != 0
    }

    /// Adds every bit in `flags`.
    fn add_winlink_flags(&mut self, flags: c_int) {
        self.set_winlink_flags(self.winlink_flags() | flags);
    }

    /// Removes every bit in `flags`.
    fn remove_winlink_flags(&mut self, flags: c_int) {
        self.set_winlink_flags(self.winlink_flags() & !flags);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_can_be_added_and_removed() {
        let mut state = crate::types::winlink {
            idx: 0,
            session_ref: None,
            window_ref: None,
            flags: 0,
        };
        assert_eq!(state.winlink_flags(), 0);
        state.add_winlink_flags(0x3);
        assert!(state.has_winlink_flags(0x1));
        state.remove_winlink_flags(0x2);
        assert_eq!(state.winlink_flags(), 0x1);
        state.set_winlink_flags(c_int::MAX);
        assert_eq!(state.winlink_flags(), c_int::MAX);
    }
}
