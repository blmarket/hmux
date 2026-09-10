//! The last scrollbar slider calculated for a pane.

use crate::types::u_int;

/// A scrollbar slider's vertical position and height.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PaneScrollbarSlider {
    /// The slider's row relative to the pane.
    pub sb_slider_y: u_int,
    /// The slider's height in rows.
    pub sb_slider_h: u_int,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_slider_is_replaced_and_cleared_as_one_value() {
        let mut scrollbar = crate::tests::test_fixtures::PaneAllocation::default();
        assert_eq!(scrollbar.slider(), PaneScrollbarSlider::default());
        scrollbar.set_slider(PaneScrollbarSlider { sb_slider_y: 3, sb_slider_h: 7 });
        assert_eq!(scrollbar.slider(), PaneScrollbarSlider { sb_slider_y: 3, sb_slider_h: 7 });
        PaneScrollbar::clear(&mut *scrollbar);
        assert_eq!(scrollbar.slider(), PaneScrollbarSlider::default());
    }
}
