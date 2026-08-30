//! The pane layout: the tree of cells a window's panes sit in, the layouts
//! that arrange them, and the string a layout is written as.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod cells;
mod custom;
mod set;

pub use cells::{
    layout_assign_pane, layout_cell_set_pane, layout_close_pane, layout_fix_panes, layout_free,
    layout_free_cell, layout_get_floating_cell, layout_get_tiled_cell, layout_init, layout_resize,
    layout_resize_layout, layout_resize_pane, layout_resize_pane_to, layout_root_ptr,
    layout_search_by_border, layout_set_size, layout_split_pane, layout_spread_out,
};
pub use custom::{layout_dump, layout_parse};
pub use set::{layout_set_lookup, layout_set_next, layout_set_previous, layout_set_select};

#[cfg(test)]
pub(crate) use cells::{
    LAYOUT_CELL_FLOATING, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, SPAWN_BEFORE,
    layout_cell_pane, layout_count_cells, layout_create_cell, layout_fix_offsets,
    layout_fix_zindexes, layout_floating_pane, layout_make_leaf, layout_make_node,
};
