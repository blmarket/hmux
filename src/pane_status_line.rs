//! Cached width of a pane's rendered status line.

/// Storage for a pane's cached status-line width.
pub trait PaneStatusLineState {
    /// Returns the cached status-line width.
    fn status_line_width(&self) -> usize;

    /// Replaces the cached status-line width.
    fn set_status_line_width(&mut self, width: usize);
}

/// The pane status-line storage used by hmux.
#[derive(Default)]
pub struct RustPaneStatusLineState {
    width: usize,
}

impl PaneStatusLineState for RustPaneStatusLineState {
    fn status_line_width(&self) -> usize {
        self.width
    }

    fn set_status_line_width(&mut self, width: usize) {
        self.width = width;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_is_zero_then_replaceable() {
        let mut state = RustPaneStatusLineState::default();
        assert_eq!(state.status_line_width(), 0);
        state.set_status_line_width(42);
        assert_eq!(state.status_line_width(), 42);
        state.set_status_line_width(usize::MAX);
        assert_eq!(state.status_line_width(), usize::MAX);
    }
}
