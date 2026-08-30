//! The screen a pane is drawn on: the state it holds, the writer that changes
//! it, and the redraw that puts it on a client's terminal.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod redraw;
mod state;
mod write;

pub use redraw::{
    screen_redraw_get_visible_ranges, screen_redraw_is_visible, screen_redraw_pane,
    screen_redraw_screen,
};
pub use state::{
    MODE_CURSOR, MODE_CURSOR_BLINKING, MODE_CURSOR_BLINKING_SET, SCREEN_CURSOR_BAR,
    SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_UNDERLINE, screen_clear_selection, screen_free,
    screen_hide_selection, screen_init, screen_mode_to_string, screen_pop_title, screen_push_title,
    screen_reinit, screen_reset_hyperlinks, screen_resize, screen_resize_cursor, screen_sel,
    screen_select_cell, screen_set_cursor_colour, screen_set_cursor_style,
    screen_set_default_cursor, screen_set_path, screen_set_progress_bar, screen_set_selection,
    screen_set_title, screen_titles,
};
pub use write::{
    CItem, screen_write_alignmenttest, screen_write_alternateoff, screen_write_alternateon,
    screen_write_backspace, screen_write_box, screen_write_carriagereturn, screen_write_cell,
    screen_write_citem, screen_write_clearcharacter, screen_write_clearendofline,
    screen_write_clearendofscreen, screen_write_clearhistory, screen_write_clearline,
    screen_write_clearscreen, screen_write_clearstartofline, screen_write_clearstartofscreen,
    screen_write_cline, screen_write_collect_add, screen_write_collect_end,
    screen_write_cursordown, screen_write_cursorleft, screen_write_cursormove,
    screen_write_cursorright, screen_write_cursorup, screen_write_deletecharacter,
    screen_write_deleteline, screen_write_fast_copy, screen_write_fullredraw, screen_write_hline,
    screen_write_insertcharacter, screen_write_insertline, screen_write_linefeed,
    screen_write_menu, screen_write_mode_clear, screen_write_mode_set, screen_write_nputs,
    screen_write_preview, screen_write_putc, screen_write_puts, screen_write_rawstring,
    screen_write_reset, screen_write_reverseindex, screen_write_scrolldown,
    screen_write_scrollregion, screen_write_scrollup, screen_write_setselection,
    screen_write_start, screen_write_start_callback, screen_write_start_pane,
    screen_write_start_sync, screen_write_stop, screen_write_stop_sync, screen_write_strlen,
    screen_write_text, screen_write_vline, screen_write_vnputs,
};

#[cfg(test)]
pub(crate) use redraw::{
    BORDER_MARKERS, CELL_BORDERS, CELL_BOTTOMJOIN, CELL_BOTTOMLEFT, CELL_BOTTOMRIGHT, CELL_INSIDE,
    CELL_JOIN, CELL_LEFTJOIN, CELL_LEFTRIGHT, CELL_OUTSIDE, CELL_RIGHTJOIN, CELL_SCROLLBAR,
    CELL_TOPBOTTOM, CELL_TOPJOIN, CELL_TOPLEFT, CELL_TOPRIGHT, CLIENT_ALLREDRAWFLAGS,
    CLIENT_REDRAWBORDERS, CLIENT_REDRAWOVERLAY, CLIENT_REDRAWPANES, CLIENT_REDRAWSCROLLBARS,
    CLIENT_REDRAWSTATUS, CLIENT_REDRAWSTATUSALWAYS, CLIENT_REDRAWWINDOW, CLIENT_SUSPENDED,
    CLIENT_UTF8, END_ISOLATE, GRID_ATTR_CHARSET, GRID_ATTR_REVERSE, LAYOUT_LEFTRIGHT,
    LAYOUT_TOPBOTTOM, PANE_LINES_DOUBLE, PANE_LINES_HEAVY, PANE_LINES_NUMBER, PANE_LINES_SIMPLE,
    PANE_LINES_SINGLE, PANE_LINES_SPACES, PANE_SCROLLBARS_LEFT, PANE_SCROLLBARS_MODAL,
    PANE_SCROLLBARS_OFF, PANE_SCROLLBARS_RIGHT, PANE_STATUS_BOTTOM, PANE_STATUS_OFF,
    PANE_STATUS_TOP, SCREEN_REDRAW_BORDER_BOTTOM, SCREEN_REDRAW_BORDER_LEFT,
    SCREEN_REDRAW_BORDER_RIGHT, SCREEN_REDRAW_BORDER_TOP, SCREEN_REDRAW_INSIDE,
    SCREEN_REDRAW_OUTSIDE, SIMPLE_BORDERS, START_ISOLATE, screen_redraw_border_set,
    screen_redraw_cell_border, screen_redraw_check_is, screen_redraw_clip_visible_ranges,
    screen_redraw_pane_border, screen_redraw_two_panes, screen_redraw_type_of_cell,
};
#[cfg(test)]
pub(crate) use state::{
    GRID_HISTORY, MODE_MOUSE_ALL, MODE_MOUSE_BUTTON, MODE_WRAP, SCREEN_CURSOR_DEFAULT,
};
pub(crate) use state::{screen_grid, screen_grid_mut, screen_grid_ptr, screen_saved_grid_ptr};
#[cfg(test)]
pub(crate) use write::{
    GRID_FLAG_PADDING, GRID_FLAG_SELECTED, GRID_LINE_WRAPPED, MODE_SYNC, PANE_REDRAW,
    PANE_REDRAWSCROLLBAR, SCREEN_WRITE_CHECKED_IF_OBSCURED, TTY_CTX_INVISIBLE_PANES,
    TTY_CTX_PANE_OBSCURED, TTY_CTX_SYNC, TTY_CTX_WINDOW_BIGGER, citem, screen_write_ctx,
    test_hooks,
};
