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

/// The pane border cache used by hmux.
#[derive(Default)]
pub struct RustPaneBorderCache {
    normal: Option<grid_cell>,
    active: Option<grid_cell>,
}

impl PaneBorderCache for RustPaneBorderCache {
    fn get(&self, kind: PaneBorderKind) -> Option<grid_cell> {
        match kind {
            PaneBorderKind::Normal => self.normal,
            PaneBorderKind::Active => self.active,
        }
    }

    fn insert(&mut self, kind: PaneBorderKind, cell: grid_cell) {
        match kind {
            PaneBorderKind::Normal => self.normal = Some(cell),
            PaneBorderKind::Active => self.active = Some(cell),
        }
    }

    fn clear(&mut self) {
        self.normal = None;
        self.active = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_are_independent_and_clear_together() {
        let mut cache = RustPaneBorderCache::default();
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
        cache.clear();
        assert!(cache.get(PaneBorderKind::Normal).is_none());
        assert!(cache.get(PaneBorderKind::Active).is_none());
    }
}
