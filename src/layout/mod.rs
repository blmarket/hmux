//! The pane layout: the tree of cells a window's panes sit in, the layouts
//! that arrange them, and the string a layout is written as.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod cells;
mod custom;
mod set;

pub(crate) use cells::layout_fix_panes_on_drop;
pub use cells::{
    LayoutCellPath, layout_cell_set_pane, layout_free_cell, layout_search_by_border,
    layout_set_size,
};

pub use set::layout_set_lookup;

pub(crate) use cells::{LAYOUT_CELL_FLOATING, layout_bind_pane, layout_cell_for_pane};

#[cfg(test)]
pub(crate) use cells::{
    LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, SPAWN_BEFORE, layout_cell_pane,
    layout_count_cells, layout_create_cell, layout_make_leaf, layout_make_node,
};

mod drag;
