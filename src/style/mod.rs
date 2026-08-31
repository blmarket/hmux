//! How something is drawn: the colours, the cell attributes, and the style
//! strings that name them.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod attributes;
mod colour;
mod parse;

pub use colour::{
    colour_256to16, colour_find_rgb, colour_force_rgb, colour_fromstring, colour_join_rgb,
    colour_palette, colour_palette_clear, colour_palette_free, colour_palette_from_defaults,
    colour_palette_get, colour_palette_init, colour_palette_set, colour_parseX11, colour_split_rgb,
    colour_tostring, colour_totheme,
};
pub use parse::{
    style_add, style_apply, style_copy, style_default, style_parse, style_ranges_free,
    style_ranges_get_range, style_ranges_init, style_set, style_set_scrollbar_style_from_option,
    style_tostring,
};

#[cfg(test)]
pub(crate) use attributes::{
    GRID_ATTR_BRIGHT, GRID_ATTR_DIM, attributes_fromstring, attributes_tostring,
};
#[cfg(test)]
pub(crate) use colour::{COLOUR_FLAG_256, COLOUR_FLAG_RGB, colour_byname};
#[cfg(test)]
pub(crate) use parse::{
    GRID_ATTR_NOATTR, STYLE_ALIGN_CENTRE, STYLE_LIST_ON, STYLE_LIST_RIGHT_MARKER,
    STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_RIGHT,
};
