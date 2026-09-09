//! How something is drawn: the colours, the cell attributes, and the style
//! strings that name them.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod attributes;
mod colour;
mod parse;

pub use colour::{
    ColourEngine, RustColourEngine, THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, colour_palette,
};
pub use parse::{
    RustStyleCodec, StyleCodec, StyleParseError, pane_scrollbar_style_from_option, style_add,
    style_apply, style_copy, style_default, style_parse, style_ranges_free, style_ranges_get_range,
    style_set, style_set_scrollbar_style_from_option, style_tostring,
};

pub use attributes::{AttributeCodec, RustAttributeCodec};
#[cfg(test)]
pub(crate) use attributes::{GRID_ATTR_BRIGHT, GRID_ATTR_DIM};
#[cfg(test)]
pub(crate) use colour::{COLOUR_FLAG_256, COLOUR_FLAG_RGB};
#[cfg(test)]
pub(crate) use parse::{
    GRID_ATTR_NOATTR, STYLE_ALIGN_CENTRE, STYLE_LIST_ON, STYLE_LIST_RIGHT_MARKER,
    STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_RIGHT,
};
