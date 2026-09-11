//! The last scrollbar slider calculated for a pane.

use crate::types::u_int;

/// A scrollbar slider's vertical position and height.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PaneScrollbarSlider {
    /// The slider's row relative to the pane.
    pub sb_slider_y: u_int,
    /// The slider's height in rows.
    pub sb_slider_h: u_int,
}
