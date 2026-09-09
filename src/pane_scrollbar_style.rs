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

/// Storage for a pane's cached scrollbar style.
pub trait PaneScrollbarStyleState {
    /// Returns the cached scrollbar style.
    fn scrollbar_style(&self) -> PaneScrollbarStyle;

    /// Replaces the cached scrollbar style.
    fn set_scrollbar_style(&mut self, style: PaneScrollbarStyle);
}

/// The pane scrollbar-style storage used by hmux.
#[derive(Default)]
pub struct RustPaneScrollbarStyleState {
    style: PaneScrollbarStyle,
}

impl PaneScrollbarStyleState for RustPaneScrollbarStyleState {
    fn scrollbar_style(&self) -> PaneScrollbarStyle {
        self.style
    }

    fn set_scrollbar_style(&mut self, style: PaneScrollbarStyle) {
        self.style = style;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_is_replaced_as_one_snapshot() {
        let mut state = RustPaneScrollbarStyleState::default();
        let style = PaneScrollbarStyle {
            cell: grid_cell {
                fg: 3,
                ..Default::default()
            },
            width: 4,
            padding: 2,
        };
        state.set_scrollbar_style(style);
        assert_eq!(state.scrollbar_style().cell.fg, 3);
        assert_eq!(state.scrollbar_style().width, 4);
        assert_eq!(state.scrollbar_style().padding, 2);
    }
}
