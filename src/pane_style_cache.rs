//! Cached normal and active styles for a pane.

use crate::types::grid_cell;

/// The normal and active cached cells for a pane.
#[derive(Copy, Clone, Default)]
pub struct PaneStyleCells {
    pub cached_gc: grid_cell,
    pub cached_active_gc: grid_cell,
}

/// Storage for a pane's cached normal and active styles.
pub trait PaneStyleCache {
    /// Returns both cached style cells.
    fn styles(&self) -> PaneStyleCells;

    /// Refreshes both cached styles if invalid, clearing the invalidation before
    /// evaluating formats so recursive observations retain the existing ordering.
    /// # Safety
    /// Exclude conflicting pane access while format callbacks execute.
    unsafe fn refresh_styles(&mut self) -> PaneStyleCells;
}
