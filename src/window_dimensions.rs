//! Cell and pixel dimensions tracked for a window.

use crate::pane_resize::PaneSize;
use crate::types::u_int;

/// A width and height measured in pixels.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct WindowPixelSize {
    pub width: u_int,
    pub height: u_int,
}

/// A cell position within a window.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct WindowCellPosition {
    pub x: u_int,
    pub y: u_int,
}

/// A snapshot of every dimension tracked for a window.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct WindowDimensions {
    pub size: PaneSize,
    pub manual_size: PaneSize,
    pub pixels: WindowPixelSize,
    pub pending_size: PaneSize,
    pub pending_pixels: WindowPixelSize,
    pub last_new_pane: WindowCellPosition,
}

impl Default for WindowDimensions {
    fn default() -> Self {
        Self {
            size: PaneSize {
                width: 0,
                height: 0,
            },
            manual_size: PaneSize {
                width: 0,
                height: 0,
            },
            pixels: WindowPixelSize::default(),
            pending_size: PaneSize {
                width: 0,
                height: 0,
            },
            pending_pixels: WindowPixelSize::default(),
            last_new_pane: WindowCellPosition::default(),
        }
    }
}

/// Storage for the dimensions tracked for a window.
pub trait WindowDimensionsState: Default {
    /// Returns all current dimension values.
    fn dimensions(&self) -> WindowDimensions;

    /// Replaces all dimension values.
    fn set_dimensions(&mut self, dimensions: WindowDimensions);

    /// Replaces the current cell size.
    fn set_size(&mut self, size: PaneSize);

    /// Replaces the manual cell-size override.
    fn set_manual_size(&mut self, size: PaneSize);

    /// Replaces the current pixel size.
    fn set_pixels(&mut self, pixels: WindowPixelSize);

    /// Replaces the pending cell size.
    fn set_pending_size(&mut self, size: PaneSize);

    /// Replaces the pending pixel size.
    fn set_pending_pixels(&mut self, pixels: WindowPixelSize);

    /// Replaces the last new-pane cell position.
    fn set_last_new_pane(&mut self, position: WindowCellPosition);
}

/// The window dimension storage used by hmux.
#[derive(Default)]
pub struct RustWindowDimensionsState {
    dimensions: WindowDimensions,
}

impl WindowDimensionsState for RustWindowDimensionsState {
    fn dimensions(&self) -> WindowDimensions {
        self.dimensions
    }

    fn set_dimensions(&mut self, dimensions: WindowDimensions) {
        self.dimensions = dimensions;
    }

    fn set_size(&mut self, size: PaneSize) {
        self.dimensions.size = size;
    }

    fn set_manual_size(&mut self, size: PaneSize) {
        self.dimensions.manual_size = size;
    }

    fn set_pixels(&mut self, pixels: WindowPixelSize) {
        self.dimensions.pixels = pixels;
    }

    fn set_pending_size(&mut self, size: PaneSize) {
        self.dimensions.pending_size = size;
    }

    fn set_pending_pixels(&mut self, pixels: WindowPixelSize) {
        self.dimensions.pending_pixels = pixels;
    }

    fn set_last_new_pane(&mut self, position: WindowCellPosition) {
        self.dimensions.last_new_pane = position;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_updates_preserve_the_other_dimensions() {
        let mut state = RustWindowDimensionsState::default();
        state.set_size(PaneSize {
            width: 80,
            height: 24,
        });
        state.set_pixels(WindowPixelSize {
            width: 8,
            height: 16,
        });
        state.set_last_new_pane(WindowCellPosition { x: 10, y: 12 });
        let dimensions = state.dimensions();
        assert_eq!(dimensions.size.width, 80);
        assert_eq!(dimensions.size.height, 24);
        assert_eq!(dimensions.pixels.width, 8);
        assert_eq!(dimensions.pixels.height, 16);
        assert_eq!(dimensions.last_new_pane.x, 10);
        assert_eq!(dimensions.last_new_pane.y, 12);
        assert_eq!(dimensions.manual_size.width, 0);
        assert_eq!(dimensions.pending_size.width, 0);
    }
}
