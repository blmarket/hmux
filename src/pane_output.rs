//! Absolute cursors into a pane's retained output stream.

/// A copyable position in a pane's output coordinate space.
pub trait PaneOutputOffset: Copy + Default {
    /// Makes an offset at `position`.
    fn at(position: usize) -> Self;

    /// Returns the absolute byte position.
    fn position(&self) -> usize;

    /// Moves the cursor to an absolute byte position.
    fn set_position(&mut self, position: usize);

    /// Advances the cursor with wrapping arithmetic.
    fn advance(&mut self, amount: usize);

    /// Subtracts a discarded base with wrapping arithmetic.
    fn rebase(&mut self, old_base: usize);
}

/// The pane output offset used by hmux.
#[derive(Clone, Copy, Default)]
pub struct RustPaneOutputOffset {
    used: usize,
}

impl PaneOutputOffset for RustPaneOutputOffset {
    fn at(position: usize) -> Self {
        Self { used: position }
    }

    fn position(&self) -> usize {
        self.used
    }

    fn set_position(&mut self, position: usize) {
        self.used = position;
    }

    fn advance(&mut self, amount: usize) {
        self.used = self.used.wrapping_add(amount);
    }

    fn rebase(&mut self, old_base: usize) {
        self.used = self.used.wrapping_sub(old_base);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets_move_in_the_wrapping_coordinate_space() {
        let mut offset = RustPaneOutputOffset::at(20);
        assert_eq!(offset.position(), 20);
        offset.advance(7);
        assert_eq!(offset.position(), 27);
        offset.rebase(10);
        assert_eq!(offset.position(), 17);
        offset.set_position(usize::MAX);
        offset.advance(2);
        assert_eq!(offset.position(), 1);
        offset.rebase(3);
        assert_eq!(offset.position(), usize::MAX - 1);
    }
}
