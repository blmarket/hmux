//! Cached normal and active styles for a pane.

use crate::types::grid_cell;

/// The normal and active cached cells for a pane.
#[derive(Copy, Clone, Default)]
pub struct PaneStyleCells {
    pub normal: grid_cell,
    pub active: grid_cell,
}

/// Storage for a pane's cached normal and active styles.
pub trait PaneStyleCache {
    /// Returns both cached style cells.
    fn styles(&self) -> PaneStyleCells;

    /// Replaces both cached style cells.
    fn set_styles(&mut self, styles: PaneStyleCells);
}

/// The pane style cache used by hmux.
#[derive(Default)]
pub struct RustPaneStyleCache {
    styles: PaneStyleCells,
}

impl PaneStyleCache for RustPaneStyleCache {
    fn styles(&self) -> PaneStyleCells {
        self.styles
    }

    fn set_styles(&mut self, styles: PaneStyleCells) {
        self.styles = styles;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_is_replaced_together() {
        let mut cache = RustPaneStyleCache::default();
        assert_eq!(cache.styles().normal.data.data[0], 0);
        assert_eq!(cache.styles().active.data.data[0], 0);
        cache.set_styles(PaneStyleCells {
            normal: grid_cell {
                fg: 3,
                ..Default::default()
            },
            active: grid_cell {
                bg: 4,
                ..Default::default()
            },
        });
        assert_eq!(cache.styles().normal.fg, 3);
        assert_eq!(cache.styles().active.bg, 4);
    }
}
