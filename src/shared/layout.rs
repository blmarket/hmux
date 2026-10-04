//! Authoritative border line kinds, the pane rectangle and width preference.

use super::abi::u_int;

pub type box_lines = ::core::ffi::c_int;
pub const BOX_LINES_DEFAULT: box_lines = -1;
pub const BOX_LINES_NONE: box_lines = 6;
pub const BOX_LINES_SINGLE: box_lines = 0;

pub type pane_lines = ::core::ffi::c_uint;
pub const PANE_LINES_NONE: pane_lines = 6;
pub const PANE_LINES_ROUNDED: pane_lines = 7;
pub const PANE_LINES_SINGLE: pane_lines = 0;

/// A pane's rectangle in window coordinates, before pane border status and
/// scrollbar adjustments.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct layout_geometry {
    pub sx: u_int,
    pub sy: u_int,
    pub xoff: ::core::ffi::c_int,
    pub yoff: ::core::ffi::c_int,
}

/// A pane's share of the visible window width in the strip.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum PaneWidth {
    #[default]
    Half,
    Full,
}

impl PaneWidth {
    pub fn toggled(self) -> Self {
        match self {
            Self::Half => Self::Full,
            Self::Full => Self::Half,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::core::mem::{align_of, size_of};

    #[test]
    fn line_kinds_match_translated_c_baseline() {
        assert_eq!(size_of::<box_lines>(), 4);
        assert_eq!(align_of::<box_lines>(), 4);
        assert_eq!(size_of::<pane_lines>(), 4);
        assert_eq!(align_of::<pane_lines>(), 4);
        assert_eq!(BOX_LINES_DEFAULT, -1);
        assert_eq!(PANE_LINES_ROUNDED, 7);
    }
}
