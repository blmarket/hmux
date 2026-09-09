//! Identity of a window link within its session.

use core::ffi::c_int;

/// Storage for the index a session assigns to a window link.
pub trait WinlinkIdentity {
    /// Returns the link's index within its session.
    fn winlink_index(&self) -> c_int;

    /// Replaces the link's index.
    fn set_winlink_index(&mut self, index: c_int);
}

/// The window-link identity storage used by hmux.
#[derive(Default)]
pub struct RustWinlinkIdentity {
    index: c_int,
}

impl WinlinkIdentity for RustWinlinkIdentity {
    fn winlink_index(&self) -> c_int {
        self.index
    }

    fn set_winlink_index(&mut self, index: c_int) {
        self.index = index;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_can_be_replaced() {
        let mut identity = RustWinlinkIdentity::default();
        assert_eq!(identity.winlink_index(), 0);
        identity.set_winlink_index(-1);
        assert_eq!(identity.winlink_index(), -1);
        identity.set_winlink_index(c_int::MAX);
        assert_eq!(identity.winlink_index(), c_int::MAX);
    }
}
