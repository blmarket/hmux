//! A pane's position and size within its window.

use crate::pane_resize::PaneSize;
use crate::types::u_int;
use core::ffi::c_int;

/// A snapshot of a pane rectangle.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct PaneGeometry {
    pub xoff: c_int,
    pub yoff: c_int,
    pub sx: u_int,
    pub sy: u_int,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_and_resize_preserve_the_other_half() {
        let mut state = crate::tests::test_fixtures::PaneAllocation::default();
        state.set_geometry(PaneGeometry {
            xoff: -3,
            yoff: 4,
            sx: 80,
            sy: 24,
        });
        state.set_position(7, -2);
        assert_eq!(
            state.geometry(),
            PaneGeometry {
                xoff: 7,
                yoff: -2,
                sx: 80,
                sy: 24,
            }
        );
        state.set_size(PaneSize {
            width: 132,
            height: 43,
        });
        assert_eq!(
            state.geometry(),
            PaneGeometry {
                xoff: 7,
                yoff: -2,
                sx: 132,
                sy: 43,
            }
        );
    }
}
