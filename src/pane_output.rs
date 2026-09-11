//! Absolute cursors into a pane's retained output stream.

/// The pane output offset used by hmux.
#[derive(Clone, Copy, Default)]
pub struct RustPaneOutputOffset {
    used: usize,
}

impl RustPaneOutputOffset {
    pub fn at(position: usize) -> Self {
        Self { used: position }
    }

    pub fn position(&self) -> usize {
        self.used
    }

    pub fn set_position(&mut self, position: usize) {
        self.used = position;
    }

    pub fn advance(&mut self, amount: usize) {
        self.used = self.used.wrapping_add(amount);
    }

    pub fn rebase(&mut self, old_base: usize) {
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
