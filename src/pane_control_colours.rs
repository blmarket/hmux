//! Foreground and background colours reported for a pane by control mode.

use core::ffi::c_int;

/// The optional foreground and background reported for a pane.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PaneControlColourPair {
    /// The reported foreground, if known.
    pub control_fg: Option<c_int>,
    /// The reported background, if known.
    pub control_bg: Option<c_int>,
}
