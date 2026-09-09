//! A pane's position and size within its window.

use crate::pane_resize::PaneSize;
use crate::types::u_int;
use core::ffi::c_int;

/// A snapshot of a pane rectangle.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct PaneGeometry {
    pub x: c_int,
    pub y: c_int,
    pub width: u_int,
    pub height: u_int,
}

/// Storage for a pane's position and size.
pub trait PaneGeometryState {
    /// Returns the complete pane rectangle.
    fn geometry(&self) -> PaneGeometry;

    /// Replaces the complete pane rectangle.
    fn set_geometry(&mut self, geometry: PaneGeometry);

    /// Moves the pane without changing its size.
    fn set_position(&mut self, x: c_int, y: c_int);

    /// Resizes the pane without moving it.
    fn set_size(&mut self, size: PaneSize);
}

/// The pane geometry storage used by hmux.
#[derive(Default)]
pub struct RustPaneGeometryState {
    geometry: PaneGeometry,
}

impl PaneGeometryState for RustPaneGeometryState {
    fn geometry(&self) -> PaneGeometry {
        self.geometry
    }

    fn set_geometry(&mut self, geometry: PaneGeometry) {
        self.geometry = geometry;
    }

    fn set_position(&mut self, x: c_int, y: c_int) {
        self.geometry.x = x;
        self.geometry.y = y;
    }

    fn set_size(&mut self, size: PaneSize) {
        self.geometry.width = size.width;
        self.geometry.height = size.height;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_and_resize_preserve_the_other_half() {
        let mut state = RustPaneGeometryState::default();
        state.set_geometry(PaneGeometry {
            x: -3,
            y: 4,
            width: 80,
            height: 24,
        });
        state.set_position(7, -2);
        assert_eq!(
            state.geometry(),
            PaneGeometry {
                x: 7,
                y: -2,
                width: 80,
                height: 24,
            }
        );
        state.set_size(PaneSize {
            width: 132,
            height: 43,
        });
        assert_eq!(
            state.geometry(),
            PaneGeometry {
                x: 7,
                y: -2,
                width: 132,
                height: 43,
            }
        );
    }
}
