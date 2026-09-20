//! Authoritative layout direction values.

pub type layout_type = ::core::ffi::c_uint;
pub const LAYOUT_WINDOWPANE: layout_type = 2;
pub const LAYOUT_TOPBOTTOM: layout_type = 1;
pub const LAYOUT_LEFTRIGHT: layout_type = 0;
pub const LAYOUT_CELL_FLOATING: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const LAYOUT_CUSTOM_OLD_FORMAT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const LAYOUT_V1_MAX_DEPTH: ::core::ffi::c_int = 1000 as ::core::ffi::c_int;

pub type box_lines = ::core::ffi::c_int;
pub const BOX_LINES_DEFAULT: box_lines = -1;
pub const BOX_LINES_DOUBLE: box_lines = 1;
pub const BOX_LINES_HEAVY: box_lines = 2;
pub const BOX_LINES_NONE: box_lines = 6;
pub const BOX_LINES_PADDED: box_lines = 5;
pub const BOX_LINES_ROUNDED: box_lines = 4;
pub const BOX_LINES_SIMPLE: box_lines = 3;
pub const BOX_LINES_SINGLE: box_lines = 0;

pub type pane_lines = ::core::ffi::c_uint;
pub const PANE_LINES_DOUBLE: pane_lines = 1;
pub const PANE_LINES_HEAVY: pane_lines = 2;
pub const PANE_LINES_NONE: pane_lines = 6;
pub const PANE_LINES_NUMBER: pane_lines = 4;
pub const PANE_LINES_ROUNDED: pane_lines = 7;
pub const PANE_LINES_SIMPLE: pane_lines = 3;
pub const PANE_LINES_SINGLE: pane_lines = 0;
pub const PANE_LINES_SPACES: pane_lines = 5;

#[cfg(test)]
mod tests {
    use super::*;
    use ::core::mem::{align_of, size_of};

    #[test]
    fn layout_domain_matches_translated_c_baseline() {
        assert_eq!(size_of::<layout_type>(), 4);
        assert_eq!(align_of::<layout_type>(), 4);
        assert_eq!(LAYOUT_LEFTRIGHT, 0);
        assert_eq!(LAYOUT_TOPBOTTOM, 1);
        assert_eq!(LAYOUT_WINDOWPANE, 2);
        assert_eq!(size_of::<box_lines>(), 4);
        assert_eq!(align_of::<box_lines>(), 4);
        assert_eq!(size_of::<pane_lines>(), 4);
        assert_eq!(align_of::<pane_lines>(), 4);
        assert_eq!(BOX_LINES_DEFAULT, -1);
        assert_eq!(PANE_LINES_ROUNDED, 7);
    }
}
