//! The terminal a client is attached to: the driver that writes to it, the
//! line drawing it does, and the keys it reads back.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod draw;
mod driver;
mod keys;

pub use draw::tty_draw_line;
pub use driver::{
    TTY_OPENED, tty_attributes, tty_cell, tty_check_overlay_range, tty_client, tty_clipboard_query,
    tty_close, tty_cmd_alignmenttest, tty_cmd_cell, tty_cmd_cells, tty_cmd_clearcharacter,
    tty_cmd_clearendofscreen, tty_cmd_clearscreen, tty_cmd_clearstartofscreen,
    tty_cmd_deletecharacter, tty_cmd_deleteline, tty_cmd_insertcharacter, tty_cmd_insertline,
    tty_cmd_rawstring, tty_cmd_redrawline, tty_cmd_reverseindex, tty_cmd_scrolldown,
    tty_cmd_scrollup, tty_cmd_setselection, tty_cmd_syncstart, tty_create_log, tty_cursor,
    tty_default_colours, tty_free, tty_init, tty_margin_off, tty_open, tty_putcode, tty_putcode_ss,
    tty_putn, tty_puts, tty_raw, tty_region_off, tty_repeat_requests, tty_reset, tty_resize,
    tty_send_requests, tty_set_path, tty_set_progress_bar, tty_set_selection, tty_set_size,
    tty_set_title, tty_start_tty, tty_stop_tty, tty_sync_end, tty_sync_start,
    tty_update_client_offset, tty_update_mode, tty_update_window_offset, tty_window_bigger,
    tty_window_offset, tty_write,
};
pub use keys::{tty_key, tty_keys_build, tty_keys_colours};

#[cfg(test)]
pub(crate) use draw::{
    GRID_ATTR_CHARSET, GRID_FLAG_CLEARED, GRID_FLAG_PADDING, GRID_FLAG_SELECTED, GRID_FLAG_TAB,
    GRID_LINE_WRAPPED, MSG_COMMAND, MSG_FLAGS, MSG_READ_CANCEL, MSG_READ_OPEN, MSG_VERSION,
    TTY_DRAW_LINE_DONE, TTY_DRAW_LINE_EMPTY, TTY_DRAW_LINE_FIRST, TTY_DRAW_LINE_FLUSH,
    TTY_DRAW_LINE_NEW1, TTY_DRAW_LINE_NEW2, TTY_DRAW_LINE_SAME, TTY_NOCURSOR, TTYC_ACSC, TTYC_BCE,
    TTYC_ECH, TTYC_EL, TTYC_EL1, TTYC_XT,
};
#[cfg(test)]
pub(crate) use driver::{
    CLIENT_REDRAWSTATUS, CLIENT_REDRAWWINDOW, CLIENT_TERMINAL, MODE_CURSOR, MODE_MOUSE_ALL,
    MODE_MOUSE_BUTTON, MODE_MOUSE_STANDARD, TERM_DECFRA, TERM_DECSLRM, TERM_NOAM, TERM_RGBCOLOURS,
    TERM_VT100LIKE, TTY_BLOCK, TTY_FREEZE, TTY_STARTED, TTYC_CLEAR, TTYC_CUP, TTYC_KMOUS,
    tty_fake_bce,
};
#[cfg(test)]
pub(crate) use keys::{
    KEYC_CTRL, KEYC_CURSOR, KEYC_DC, KEYC_DOUBLECLICK_PANE, KEYC_DOWN, KEYC_F1, KEYC_F5,
    KEYC_IMPLIED_META, KEYC_KEYPAD, KEYC_KP_ZERO, KEYC_LEFT, KEYC_MASK_KEY, KEYC_META,
    KEYC_PASTE_END, KEYC_PASTE_START, KEYC_RIGHT, KEYC_SECONDCLICK_PANE, KEYC_SHIFT,
    KEYC_TRIPLECLICK_PANE, KEYC_TYPE_DOUBLECLICK, KEYC_TYPE_FUNCTION, KEYC_TYPE_MOUSEDOWN,
    KEYC_TYPE_MOUSEDRAG, KEYC_TYPE_MOUSEDRAGEND, KEYC_TYPE_MOUSEMOVE, KEYC_TYPE_MOUSEUP,
    KEYC_TYPE_NOTYPE, KEYC_TYPE_SECONDCLICK, KEYC_TYPE_TRIPLECLICK, KEYC_TYPE_UNICODE,
    KEYC_TYPE_USER, KEYC_TYPE_WHEELDOWN, KEYC_TYPE_WHEELUP, KEYC_UNKNOWN, KEYC_UP, KEYC_USER,
    TTY_BRACKETPASTE, TTY_WAITBG, TTY_WAITFG, TTY_WINSIZEQUERY, tty_keys_free,
};
