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

    /// Replaces both cached style cells.
    fn set_styles(&mut self, styles: PaneStyleCells);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_is_replaced_together() {
        let mut cache = crate::tests::test_fixtures::PaneAllocation::default();
        assert_eq!(cache.styles().cached_gc.data.data[0], 0);
        assert_eq!(cache.styles().cached_active_gc.data.data[0], 0);
        cache.set_styles(PaneStyleCells {
            cached_gc: grid_cell {
                fg: 3,
                ..Default::default()
            },
            cached_active_gc: grid_cell {
                bg: 4,
                ..Default::default()
            },
        });
        assert_eq!(cache.styles().cached_gc.fg, 3);
        assert_eq!(cache.styles().cached_active_gc.bg, 4);
    }
}
