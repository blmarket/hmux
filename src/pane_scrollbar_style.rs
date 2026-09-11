//! Cached drawing style for a pane scrollbar.

use crate::types::grid_cell;
use core::ffi::c_int;

/// The values used to size and draw a pane scrollbar.
#[derive(Copy, Clone, Default)]
pub struct PaneScrollbarStyle {
    pub cell: grid_cell,
    pub width: c_int,
    pub padding: c_int,
}
