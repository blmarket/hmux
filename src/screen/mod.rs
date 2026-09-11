//! The screen a pane is drawn on: the state it holds, the writer that changes
//! it, and the redraw that puts it on a client's terminal.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod draw;
mod redraw;
mod state;
pub(crate) mod write;
mod writer;

pub use draw::{format_trim_left, format_trim_right, format_width};
pub(crate) use draw::{format_trim_left_impl, format_trim_right_impl, format_width_impl};
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
    screen_redraw_cell_border, screen_redraw_check_is, screen_redraw_pane_border,
    screen_redraw_two_panes, screen_redraw_type_of_cell,
};
pub(crate) use redraw::{screen_redraw_is_visible, screen_redraw_pane, screen_redraw_screen};
#[cfg(test)]
pub(crate) use state::{GRID_HISTORY, MODE_WRAP, SCREEN_CURSOR_DEFAULT};
pub use state::{
    MODE_CURSOR, MODE_CURSOR_BLINKING, MODE_CURSOR_BLINKING_SET, SCREEN_CURSOR_BAR,
    SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_UNDERLINE, Screen, ScreenModeState,
};
pub(crate) use state::{MODE_SYNC, screen_mode_to_string};
pub(crate) use write::screen_write_init_ctx;
pub use write::{CItem, screen_write_citem, screen_write_cline};
#[cfg(test)]
pub(crate) use write::{GRID_LINE_WRAPPED, TTY_CTX_PANE_OBSCURED, citem_snapshot};
pub(crate) use writer::{
    RustScreenWriteCtx, ScreenWriteCtx, screen_write_ctx_on_pane, screen_write_ctx_on_pane_base,
    screen_write_ctx_on_screen, screen_write_strlen,
};

use state::screen;

/// The Rust screen implementation used by hmux.
///
/// ```compile_fail
/// use tmux_c2rs::{RustScreen, Screen};
/// let mut screen = RustScreen::new(80, 24, 100);
/// screen.grid_mut();
/// ```
#[derive(Default)]
pub struct RustScreen(screen);
