//! A pane's position and size within its window.

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

    /// Moves the pane without changing its size.
    fn set_position(&mut self, x: c_int, y: c_int);


}

