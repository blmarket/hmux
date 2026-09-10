//! Cached cells used to draw pane borders.

use crate::types::grid_cell;

/// Which pane border style a cached cell represents.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PaneBorderKind {
    Normal,
    Active,
}

/// A cache for the normal and active pane border cells.
pub trait PaneBorderCache {
    /// Returns a cached border cell when present.
    fn get(&self, kind: PaneBorderKind) -> Option<grid_cell>;

    /// Stores a border cell.
    fn insert(&mut self, kind: PaneBorderKind, cell: grid_cell);

    /// Removes both cached border cells.
    fn clear(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_are_independent_and_clear_together() {
        let mut cache = crate::tests::test_fixtures::PaneAllocation::default();
        let normal = grid_cell {
            fg: 3,
            ..Default::default()
        };
        let active = grid_cell {
            bg: 4,
            ..Default::default()
        };
        assert!(cache.get(PaneBorderKind::Normal).is_none());
        assert!(cache.get(PaneBorderKind::Active).is_none());
        cache.insert(PaneBorderKind::Normal, normal);
        assert_eq!(cache.get(PaneBorderKind::Normal).unwrap().fg, 3);
        assert!(cache.get(PaneBorderKind::Active).is_none());
        cache.insert(PaneBorderKind::Active, active);
        assert_eq!(cache.get(PaneBorderKind::Active).unwrap().bg, 4);
        PaneBorderCache::clear(&mut *cache);
        assert!(cache.get(PaneBorderKind::Normal).is_none());
        assert!(cache.get(PaneBorderKind::Active).is_none());
    }
}
