//! Stable value-state capabilities of a window link.

use crate::{WinlinkFlagsState, WinlinkIdentity};

/// The stable, independently implementable state of a window link.
///
/// Session and window membership remain graph resources managed by the
/// window engine.
pub trait Winlink: WinlinkIdentity + WinlinkFlagsState {}

impl<T> Winlink for T where T: WinlinkIdentity + WinlinkFlagsState {}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_winlink<T: Winlink>() {}

    #[test]
    fn server_winlink_implements_the_aggregate_contract() {
        assert_winlink::<crate::types::winlink>();
    }
}
