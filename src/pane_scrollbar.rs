//! The last scrollbar slider calculated for a pane.

use crate::types::u_int;

/// A scrollbar slider's vertical position and height.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PaneScrollbarSlider {
    /// The slider's row relative to the pane.
    pub y: u_int,
    /// The slider's height in rows.
    pub height: u_int,
}

/// Storage for a pane's last calculated scrollbar slider.
pub trait PaneScrollbar {
    /// Returns a copy of the current slider.
    fn slider(&self) -> PaneScrollbarSlider;

    /// Replaces both slider properties together.
    fn set_slider(&mut self, slider: PaneScrollbarSlider);

    /// Restores the empty slider.
    fn clear(&mut self);
}

/// The pane scrollbar storage used by hmux.
#[derive(Default)]
pub struct RustPaneScrollbar {
    slider: PaneScrollbarSlider,
}

impl PaneScrollbar for RustPaneScrollbar {
    fn slider(&self) -> PaneScrollbarSlider {
        self.slider
    }

    fn set_slider(&mut self, slider: PaneScrollbarSlider) {
        self.slider = slider;
    }

    fn clear(&mut self) {
        self.slider = PaneScrollbarSlider::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_slider_is_replaced_and_cleared_as_one_value() {
        let mut scrollbar = RustPaneScrollbar::default();
        assert_eq!(scrollbar.slider(), PaneScrollbarSlider::default());
        scrollbar.set_slider(PaneScrollbarSlider { y: 3, height: 7 });
        assert_eq!(scrollbar.slider(), PaneScrollbarSlider { y: 3, height: 7 });
        scrollbar.clear();
        assert_eq!(scrollbar.slider(), PaneScrollbarSlider::default());
    }
}
