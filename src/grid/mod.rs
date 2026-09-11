//! The cell store: a grid of lines, the screen's view onto it, a cursor that
//! walks it, and the hyperlink set its extended cells point into.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod line;
pub(crate) mod links;
mod reader;
mod store;
mod view;

use line::grid_line;
pub use links::{Hyperlinks, RustHyperlinks};
pub use reader::{GridReader, RustGrid, RustGridReader};
pub use store::{
    GRID_LINE_WRAPPED, GRID_STRING_EMPTY_CELLS, GRID_STRING_TRIM_SPACES, Grid, GridLineInfo, grid,
    grid_cells_equal, grid_cells_look_equal, grid_create, grid_default_cell, grid_set_tab,
};

#[cfg(test)]
pub(crate) use store::{GRID_ATTR_BRIGHT, GRID_ATTR_CHARSET, GRID_FLAG_PADDING, GRID_HISTORY};
