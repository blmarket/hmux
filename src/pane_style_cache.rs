//! Cached normal and active styles for a pane.

use crate::types::grid_cell;

/// The normal and active cached cells for a pane.
#[derive(Copy, Clone, Default)]
pub struct PaneStyleCells {
    pub cached_gc: grid_cell,
    pub cached_active_gc: grid_cell,
}
