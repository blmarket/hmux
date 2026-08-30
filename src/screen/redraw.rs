use super::state::{screen_free, screen_grid, screen_grid_ptr};
use super::write::{
    screen_write_cell, screen_write_cursormove, screen_write_start, screen_write_stop,
    screen_write_stop_sync,
};
use crate::fmt_args;
use crate::format::format_draw;
use crate::format::{format_create, format_create_defaults, format_defaults, format_expand_time};
use crate::grid::{grid_compare, grid_default_cell};
use crate::log::log_debug;
use crate::modes::window_copy_get_current_offset;
use crate::options::{options_get_number, options_get_string, options_ptr};
use crate::server::server_client_get_pane;
use crate::server::{marked_pane, server_is_marked};
use crate::server::{server_client_ensure_ranges, server_client_ranges_is_empty};
use crate::session::{session_get_curw, session_options};
use crate::status::{status_line_size, status_message_redraw, status_prompt_redraw, status_redraw};
use crate::style::style_ranges_free;
use crate::style::{style_add, style_apply};
use crate::terminfo::{tty_acs_double_borders, tty_acs_heavy_borders};
use crate::terminfo::{tty_term_has, tty_term_of};
use crate::text::{utf8_copy, utf8_set};
use crate::tty::tty_draw_line;
use crate::tty::{
    tty_cell, tty_check_overlay_range, tty_cursor, tty_default_colours, tty_puts, tty_reset,
    tty_sync_start, tty_update_mode, tty_window_offset,
};
pub use crate::types::*;
use crate::window::PaneStack;
use crate::window::{
    window_pane_index, window_pane_is_floating, window_pane_mode, window_pane_show_scrollbar,
    window_pane_stack_first, window_pane_stack_last, window_pane_stack_next,
    window_pane_stack_prev, window_pane_visible, window_panes_first, window_panes_next,
};
pub const MSG_READ_CANCEL: msgtype = 307;
pub const MSG_WRITE_CLOSE: msgtype = 306;
pub const MSG_WRITE_READY: msgtype = 305;
pub const MSG_WRITE: msgtype = 304;
pub const MSG_WRITE_OPEN: msgtype = 303;
pub const MSG_READ_DONE: msgtype = 302;
pub const MSG_READ: msgtype = 301;
pub const MSG_READ_OPEN: msgtype = 300;
pub const MSG_FLAGS: msgtype = 218;
pub const MSG_EXEC: msgtype = 217;
pub const MSG_WAKEUP: msgtype = 216;
pub const MSG_UNLOCK: msgtype = 215;
pub const MSG_SUSPEND: msgtype = 214;
pub const MSG_OLDSTDOUT: msgtype = 213;
pub const MSG_OLDSTDIN: msgtype = 212;
pub const MSG_OLDSTDERR: msgtype = 211;
pub const MSG_SHUTDOWN: msgtype = 210;
pub const MSG_SHELL: msgtype = 209;
pub const MSG_RESIZE: msgtype = 208;
pub const MSG_READY: msgtype = 207;
pub const MSG_LOCK: msgtype = 206;
pub const MSG_EXITING: msgtype = 205;
pub const MSG_EXITED: msgtype = 204;
pub const MSG_EXIT: msgtype = 203;
pub const MSG_DETACHKILL: msgtype = 202;
pub const MSG_DETACH: msgtype = 201;
pub const MSG_COMMAND: msgtype = 200;
pub const MSG_IDENTIFY_TERMINFO: msgtype = 112;
pub const MSG_IDENTIFY_LONGFLAGS: msgtype = 111;
pub const MSG_IDENTIFY_STDOUT: msgtype = 110;
pub const MSG_IDENTIFY_FEATURES: msgtype = 109;
pub const MSG_IDENTIFY_CWD: msgtype = 108;
pub const MSG_IDENTIFY_CLIENTPID: msgtype = 107;
pub const MSG_IDENTIFY_DONE: msgtype = 106;
pub const MSG_IDENTIFY_ENVIRON: msgtype = 105;
pub const MSG_IDENTIFY_STDIN: msgtype = 104;
pub const MSG_IDENTIFY_OLDCWD: msgtype = 103;
pub const MSG_IDENTIFY_TTYNAME: msgtype = 102;
pub const MSG_IDENTIFY_TERM: msgtype = 101;
pub const MSG_IDENTIFY_FLAGS: msgtype = 100;
pub const MSG_VERSION: msgtype = 12;
pub const PANE_LINES_SPACES: pane_lines = 5;
pub const PANE_LINES_NUMBER: pane_lines = 4;
pub const PANE_LINES_SIMPLE: pane_lines = 3;
pub const PANE_LINES_HEAVY: pane_lines = 2;
pub const PANE_LINES_DOUBLE: pane_lines = 1;
pub const PANE_LINES_SINGLE: pane_lines = 0;
pub const PROGRESS_BAR_PAUSED: progress_bar_state = 4;
pub const PROGRESS_BAR_INDETERMINATE: progress_bar_state = 3;
pub const PROGRESS_BAR_ERROR: progress_bar_state = 2;
pub const PROGRESS_BAR_NORMAL: progress_bar_state = 1;
pub const PROGRESS_BAR_HIDDEN: progress_bar_state = 0;
pub const SCREEN_CURSOR_BAR: screen_cursor_style = 3;
pub const SCREEN_CURSOR_UNDERLINE: screen_cursor_style = 2;
pub const SCREEN_CURSOR_BLOCK: screen_cursor_style = 1;
pub const SCREEN_CURSOR_DEFAULT: screen_cursor_style = 0;
pub const STYLE_DEFAULT_SET: style_default_type = 3;
pub const STYLE_DEFAULT_POP: style_default_type = 2;
pub const STYLE_DEFAULT_PUSH: style_default_type = 1;
pub const STYLE_DEFAULT_BASE: style_default_type = 0;
pub const STYLE_RANGE_CONTROL: style_range_type = 7;
pub const STYLE_RANGE_USER: style_range_type = 6;
pub const STYLE_RANGE_SESSION: style_range_type = 5;
pub const STYLE_RANGE_WINDOW: style_range_type = 4;
pub const STYLE_RANGE_PANE: style_range_type = 3;
pub const STYLE_RANGE_RIGHT: style_range_type = 2;
pub const STYLE_RANGE_LEFT: style_range_type = 1;
pub const STYLE_RANGE_NONE: style_range_type = 0;
pub const STYLE_LIST_RIGHT_MARKER: style_list = 4;
pub const STYLE_LIST_LEFT_MARKER: style_list = 3;
pub const STYLE_LIST_FOCUS: style_list = 2;
pub const STYLE_LIST_ON: style_list = 1;
pub const STYLE_LIST_OFF: style_list = 0;
pub const STYLE_ALIGN_ABSOLUTE_CENTRE: style_align = 4;
pub const STYLE_ALIGN_RIGHT: style_align = 3;
pub const STYLE_ALIGN_CENTRE: style_align = 2;
pub const STYLE_ALIGN_LEFT: style_align = 1;
pub const STYLE_ALIGN_DEFAULT: style_align = 0;
pub const THEME_DARK: client_theme = 2;
pub const THEME_LIGHT: client_theme = 1;
pub const THEME_UNKNOWN: client_theme = 0;
pub const LAYOUT_WINDOWPANE: layout_type = 2;
pub const LAYOUT_TOPBOTTOM: layout_type = 1;
pub const LAYOUT_LEFTRIGHT: layout_type = 0;
pub const PROMPT_TYPE_INVALID: prompt_type = 255;
pub const PROMPT_TYPE_WINDOW_TARGET: prompt_type = 3;
pub const PROMPT_TYPE_TARGET: prompt_type = 2;
pub const PROMPT_TYPE_SEARCH: prompt_type = 1;
pub const PROMPT_TYPE_COMMAND: prompt_type = 0;
pub const PROMPT_COMMAND: client_prompt_mode = 1;
pub const PROMPT_ENTRY: client_prompt_mode = 0;
pub const CLIENT_EXIT_DETACH: client_exit_type = 2;
pub const CLIENT_EXIT_SHUTDOWN: client_exit_type = 1;
pub const CLIENT_EXIT_RETURN: client_exit_type = 0;
pub const TTYC_XT: tty_code_code = 232;
pub const TTYC_VPA: tty_code_code = 231;
pub const TTYC_U8: tty_code_code = 230;
pub const TTYC_TSL: tty_code_code = 229;
pub const TTYC_TC: tty_code_code = 228;
pub const TTYC_SYNC: tty_code_code = 227;
pub const TTYC_SWD: tty_code_code = 226;
pub const TTYC_SS: tty_code_code = 225;
pub const TTYC_SXL: tty_code_code = 224;
pub const TTYC_SPB: tty_code_code = 223;
pub const TTYC_SMXX: tty_code_code = 222;
pub const TTYC_SMULX: tty_code_code = 221;
pub const TTYC_SMUL: tty_code_code = 220;
pub const TTYC_SMSO: tty_code_code = 219;
pub const TTYC_SMOL: tty_code_code = 218;
pub const TTYC_SMKX: tty_code_code = 217;
pub const TTYC_SMCUP: tty_code_code = 216;
pub const TTYC_SMACS: tty_code_code = 215;
pub const TTYC_SITM: tty_code_code = 214;
pub const TTYC_SGR0: tty_code_code = 213;
pub const TTYC_SETULC1: tty_code_code = 212;
pub const TTYC_SETULC: tty_code_code = 211;
pub const TTYC_SETRGBF: tty_code_code = 210;
pub const TTYC_SETRGBB: tty_code_code = 209;
pub const TTYC_SETAL: tty_code_code = 208;
pub const TTYC_SETAF: tty_code_code = 207;
pub const TTYC_SETAB: tty_code_code = 206;
pub const TTYC_SE: tty_code_code = 205;
pub const TTYC_RMKX: tty_code_code = 204;
pub const TTYC_RMCUP: tty_code_code = 203;
pub const TTYC_RMACS: tty_code_code = 202;
pub const TTYC_RIN: tty_code_code = 201;
pub const TTYC_RI: tty_code_code = 200;
pub const TTYC_RGB: tty_code_code = 199;
pub const TTYC_REV: tty_code_code = 198;
pub const TTYC_RECT: tty_code_code = 197;
pub const TTYC_OP: tty_code_code = 196;
pub const TTYC_OL: tty_code_code = 195;
pub const TTYC_NOBR: tty_code_code = 194;
pub const TTYC_MS: tty_code_code = 193;
pub const TTYC_KUP7: tty_code_code = 192;
pub const TTYC_KUP6: tty_code_code = 191;
pub const TTYC_KUP5: tty_code_code = 190;
pub const TTYC_KUP4: tty_code_code = 189;
pub const TTYC_KUP3: tty_code_code = 188;
pub const TTYC_KUP2: tty_code_code = 187;
pub const TTYC_KRIT7: tty_code_code = 186;
pub const TTYC_KRIT6: tty_code_code = 185;
pub const TTYC_KRIT5: tty_code_code = 184;
pub const TTYC_KRIT4: tty_code_code = 183;
pub const TTYC_KRIT3: tty_code_code = 182;
pub const TTYC_KRIT2: tty_code_code = 181;
pub const TTYC_KRI: tty_code_code = 180;
pub const TTYC_KPRV7: tty_code_code = 179;
pub const TTYC_KPRV6: tty_code_code = 178;
pub const TTYC_KPRV5: tty_code_code = 177;
pub const TTYC_KPRV4: tty_code_code = 176;
pub const TTYC_KPRV3: tty_code_code = 175;
pub const TTYC_KPRV2: tty_code_code = 174;
pub const TTYC_KPP: tty_code_code = 173;
pub const TTYC_KNXT7: tty_code_code = 172;
pub const TTYC_KNXT6: tty_code_code = 171;
pub const TTYC_KNXT5: tty_code_code = 170;
pub const TTYC_KNXT4: tty_code_code = 169;
pub const TTYC_KNXT3: tty_code_code = 168;
pub const TTYC_KNXT2: tty_code_code = 167;
pub const TTYC_KNP: tty_code_code = 166;
pub const TTYC_KMOUS: tty_code_code = 165;
pub const TTYC_KLFT7: tty_code_code = 164;
pub const TTYC_KLFT6: tty_code_code = 163;
pub const TTYC_KLFT5: tty_code_code = 162;
pub const TTYC_KLFT4: tty_code_code = 161;
pub const TTYC_KLFT3: tty_code_code = 160;
pub const TTYC_KLFT2: tty_code_code = 159;
pub const TTYC_KIND: tty_code_code = 158;
pub const TTYC_KICH1: tty_code_code = 157;
pub const TTYC_KIC7: tty_code_code = 156;
pub const TTYC_KIC6: tty_code_code = 155;
pub const TTYC_KIC5: tty_code_code = 154;
pub const TTYC_KIC4: tty_code_code = 153;
pub const TTYC_KIC3: tty_code_code = 152;
pub const TTYC_KIC2: tty_code_code = 151;
pub const TTYC_KHOME: tty_code_code = 150;
pub const TTYC_KHOM7: tty_code_code = 149;
pub const TTYC_KHOM6: tty_code_code = 148;
pub const TTYC_KHOM5: tty_code_code = 147;
pub const TTYC_KHOM4: tty_code_code = 146;
pub const TTYC_KHOM3: tty_code_code = 145;
pub const TTYC_KHOM2: tty_code_code = 144;
pub const TTYC_KF9: tty_code_code = 143;
pub const TTYC_KF8: tty_code_code = 142;
pub const TTYC_KF7: tty_code_code = 141;
pub const TTYC_KF63: tty_code_code = 140;
pub const TTYC_KF62: tty_code_code = 139;
pub const TTYC_KF61: tty_code_code = 138;
pub const TTYC_KF60: tty_code_code = 137;
pub const TTYC_KF6: tty_code_code = 136;
pub const TTYC_KF59: tty_code_code = 135;
pub const TTYC_KF58: tty_code_code = 134;
pub const TTYC_KF57: tty_code_code = 133;
pub const TTYC_KF56: tty_code_code = 132;
pub const TTYC_KF55: tty_code_code = 131;
pub const TTYC_KF54: tty_code_code = 130;
pub const TTYC_KF53: tty_code_code = 129;
pub const TTYC_KF52: tty_code_code = 128;
pub const TTYC_KF51: tty_code_code = 127;
pub const TTYC_KF50: tty_code_code = 126;
pub const TTYC_KF5: tty_code_code = 125;
pub const TTYC_KF49: tty_code_code = 124;
pub const TTYC_KF48: tty_code_code = 123;
pub const TTYC_KF47: tty_code_code = 122;
pub const TTYC_KF46: tty_code_code = 121;
pub const TTYC_KF45: tty_code_code = 120;
pub const TTYC_KF44: tty_code_code = 119;
pub const TTYC_KF43: tty_code_code = 118;
pub const TTYC_KF42: tty_code_code = 117;
pub const TTYC_KF41: tty_code_code = 116;
pub const TTYC_KF40: tty_code_code = 115;
pub const TTYC_KF4: tty_code_code = 114;
pub const TTYC_KF39: tty_code_code = 113;
pub const TTYC_KF38: tty_code_code = 112;
pub const TTYC_KF37: tty_code_code = 111;
pub const TTYC_KF36: tty_code_code = 110;
pub const TTYC_KF35: tty_code_code = 109;
pub const TTYC_KF34: tty_code_code = 108;
pub const TTYC_KF33: tty_code_code = 107;
pub const TTYC_KF32: tty_code_code = 106;
pub const TTYC_KF31: tty_code_code = 105;
pub const TTYC_KF30: tty_code_code = 104;
pub const TTYC_KF3: tty_code_code = 103;
pub const TTYC_KF29: tty_code_code = 102;
pub const TTYC_KF28: tty_code_code = 101;
pub const TTYC_KF27: tty_code_code = 100;
pub const TTYC_KF26: tty_code_code = 99;
pub const TTYC_KF25: tty_code_code = 98;
pub const TTYC_KF24: tty_code_code = 97;
pub const TTYC_KF23: tty_code_code = 96;
pub const TTYC_KF22: tty_code_code = 95;
pub const TTYC_KF21: tty_code_code = 94;
pub const TTYC_KF20: tty_code_code = 93;
pub const TTYC_KF2: tty_code_code = 92;
pub const TTYC_KF19: tty_code_code = 91;
pub const TTYC_KF18: tty_code_code = 90;
pub const TTYC_KF17: tty_code_code = 89;
pub const TTYC_KF16: tty_code_code = 88;
pub const TTYC_KF15: tty_code_code = 87;
pub const TTYC_KF14: tty_code_code = 86;
pub const TTYC_KF13: tty_code_code = 85;
pub const TTYC_KF12: tty_code_code = 84;
pub const TTYC_KF11: tty_code_code = 83;
pub const TTYC_KF10: tty_code_code = 82;
pub const TTYC_KF1: tty_code_code = 81;
pub const TTYC_KEND7: tty_code_code = 80;
pub const TTYC_KEND6: tty_code_code = 79;
pub const TTYC_KEND5: tty_code_code = 78;
pub const TTYC_KEND4: tty_code_code = 77;
pub const TTYC_KEND3: tty_code_code = 76;
pub const TTYC_KEND2: tty_code_code = 75;
pub const TTYC_KEND: tty_code_code = 74;
pub const TTYC_KDN7: tty_code_code = 73;
pub const TTYC_KDN6: tty_code_code = 72;
pub const TTYC_KDN5: tty_code_code = 71;
pub const TTYC_KDN4: tty_code_code = 70;
pub const TTYC_KDN3: tty_code_code = 69;
pub const TTYC_KDN2: tty_code_code = 68;
pub const TTYC_KDCH1: tty_code_code = 67;
pub const TTYC_KDC7: tty_code_code = 66;
pub const TTYC_KDC6: tty_code_code = 65;
pub const TTYC_KDC5: tty_code_code = 64;
pub const TTYC_KDC4: tty_code_code = 63;
pub const TTYC_KDC3: tty_code_code = 62;
pub const TTYC_KDC2: tty_code_code = 61;
pub const TTYC_KCUU1: tty_code_code = 60;
pub const TTYC_KCUF1: tty_code_code = 59;
pub const TTYC_KCUD1: tty_code_code = 58;
pub const TTYC_KCUB1: tty_code_code = 57;
pub const TTYC_KCBT: tty_code_code = 56;
pub const TTYC_INVIS: tty_code_code = 55;
pub const TTYC_INDN: tty_code_code = 54;
pub const TTYC_IL1: tty_code_code = 53;
pub const TTYC_IL: tty_code_code = 52;
pub const TTYC_ICH1: tty_code_code = 51;
pub const TTYC_ICH: tty_code_code = 50;
pub const TTYC_HPA: tty_code_code = 49;
pub const TTYC_HOME: tty_code_code = 48;
pub const TTYC_HLS: tty_code_code = 47;
pub const TTYC_FSL: tty_code_code = 46;
pub const TTYC_ENMG: tty_code_code = 45;
pub const TTYC_ENFCS: tty_code_code = 44;
pub const TTYC_ENEKS: tty_code_code = 43;
pub const TTYC_ENBP: tty_code_code = 42;
pub const TTYC_ENACS: tty_code_code = 41;
pub const TTYC_EL1: tty_code_code = 40;
pub const TTYC_EL: tty_code_code = 39;
pub const TTYC_ED: tty_code_code = 38;
pub const TTYC_ECH: tty_code_code = 37;
pub const TTYC_E3: tty_code_code = 36;
pub const TTYC_DSMG: tty_code_code = 35;
pub const TTYC_DSFCS: tty_code_code = 34;
pub const TTYC_DSEKS: tty_code_code = 33;
pub const TTYC_DSBP: tty_code_code = 32;
pub const TTYC_DL1: tty_code_code = 31;
pub const TTYC_DL: tty_code_code = 30;
pub const TTYC_DIM: tty_code_code = 29;
pub const TTYC_DCH1: tty_code_code = 28;
pub const TTYC_DCH: tty_code_code = 27;
pub const TTYC_CVVIS: tty_code_code = 26;
pub const TTYC_CUU1: tty_code_code = 25;
pub const TTYC_CUU: tty_code_code = 24;
pub const TTYC_CUP: tty_code_code = 23;
pub const TTYC_CUF1: tty_code_code = 22;
pub const TTYC_CUF: tty_code_code = 21;
pub const TTYC_CUD1: tty_code_code = 20;
pub const TTYC_CUD: tty_code_code = 19;
pub const TTYC_CUB1: tty_code_code = 18;
pub const TTYC_CUB: tty_code_code = 17;
pub const TTYC_CSR: tty_code_code = 16;
pub const TTYC_CS: tty_code_code = 15;
pub const TTYC_CR: tty_code_code = 14;
pub const TTYC_COLORS: tty_code_code = 13;
pub const TTYC_CNORM: tty_code_code = 12;
pub const TTYC_CMG: tty_code_code = 11;
pub const TTYC_CLMG: tty_code_code = 10;
pub const TTYC_CLEAR: tty_code_code = 9;
pub const TTYC_CIVIS: tty_code_code = 8;
pub const TTYC_BOLD: tty_code_code = 7;
pub const TTYC_BLINK: tty_code_code = 6;
pub const TTYC_BIDI: tty_code_code = 5;
pub const TTYC_BEL: tty_code_code = 4;
pub const TTYC_BCE: tty_code_code = 3;
pub const TTYC_AX: tty_code_code = 2;
pub const TTYC_AM: tty_code_code = 1;
pub const TTYC_ACSC: tty_code_code = 0;
pub const SCREEN_REDRAW_BORDER_LEFT: screen_redraw_border_type = 2;
pub const SCREEN_REDRAW_BORDER_RIGHT: screen_redraw_border_type = 3;
pub const SCREEN_REDRAW_OUTSIDE: screen_redraw_border_type = 0;
pub const SCREEN_REDRAW_BORDER_TOP: screen_redraw_border_type = 4;
pub const SCREEN_REDRAW_BORDER_BOTTOM: screen_redraw_border_type = 5;
pub const SCREEN_REDRAW_INSIDE: screen_redraw_border_type = 1;
pub type screen_redraw_border_type = ::core::ffi::c_uint;
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const MODE_SYNC: ::core::ffi::c_int = 0x100000 as ::core::ffi::c_int;
pub const GRID_ATTR_REVERSE: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const GRID_ATTR_CHARSET: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const CELL_INSIDE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CELL_TOPBOTTOM: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CELL_LEFTRIGHT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CELL_TOPLEFT: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const CELL_TOPRIGHT: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const CELL_BOTTOMLEFT: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const CELL_BOTTOMRIGHT: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const CELL_TOPJOIN: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const CELL_BOTTOMJOIN: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const CELL_LEFTJOIN: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const CELL_RIGHTJOIN: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const CELL_JOIN: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const CELL_OUTSIDE: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const CELL_SCROLLBAR: ::core::ffi::c_int = 13 as ::core::ffi::c_int;
pub const CELL_BORDERS: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b" xqlkmjwvtun~\0") };
pub const SIMPLE_BORDERS: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b" |-+++++++++.\0") };
pub const PANE_BORDER_COLOUR: ::core::ffi::c_longlong = 1 as ::core::ffi::c_longlong;
pub const PANE_BORDER_ARROWS: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const PANE_BORDER_BOTH: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const WINDOW_PANE_NO_MODE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_STATUS_OFF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_STATUS_TOP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_STATUS_BOTTOM: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_OFF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_MODAL: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_RIGHT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_LEFT: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CLIENT_REDRAWWINDOW: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CLIENT_REDRAWSTATUS: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CLIENT_SUSPENDED: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CLIENT_REDRAWBORDERS: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const CLIENT_UTF8: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWSTATUSALWAYS: ::core::ffi::c_int = 0x1000000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWOVERLAY: ::core::ffi::c_int = 0x2000000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWPANES: ::core::ffi::c_int = 0x20000000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWSCROLLBARS: ::core::ffi::c_ulonglong =
    0x4000000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_ALLREDRAWFLAGS: ::core::ffi::c_ulonglong = (CLIENT_REDRAWWINDOW
    | CLIENT_REDRAWSTATUS
    | CLIENT_REDRAWSTATUSALWAYS
    | CLIENT_REDRAWBORDERS
    | CLIENT_REDRAWOVERLAY
    | CLIENT_REDRAWPANES)
    as ::core::ffi::c_ulonglong
    | CLIENT_REDRAWSCROLLBARS;
pub const FORMAT_STATUS: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const FORMAT_PANE: ::core::ffi::c_uint = 0x80000000 as ::core::ffi::c_uint;
pub const START_ISOLATE: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"\xE2\x81\xA6\0") };
pub const END_ISOLATE: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"\xE2\x81\xA9\0") };
pub const BORDER_MARKERS: [::core::ffi::c_char; 7] =
    unsafe { ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"  +,.-\0") };
pub(crate) unsafe fn screen_redraw_border_set(
    mut w: *mut window,
    mut wp: *mut window_pane,
    mut pane_lines: pane_lines,
    mut cell_type: ::core::ffi::c_int,
    mut gc: *mut grid_cell,
) {
    unsafe {
        let mut idx: u_int = 0;
        if cell_type == CELL_OUTSIDE
            && let Some(fill) = (*w).fill_character.as_deref()
        {
            utf8_copy(&mut (*gc).data, fill);
            return;
        }
        match pane_lines {
            PANE_LINES_NUMBER => {
                if cell_type == CELL_OUTSIDE {
                    (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
                    utf8_set(
                        &mut (*gc).data,
                        CELL_BORDERS[CELL_OUTSIDE as usize] as u_char,
                    );
                } else {
                    (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
                    if !wp.is_null() && {
                        let found;
                        (found, idx) = window_pane_index(wp);
                        found == 0
                    } {
                        utf8_set(
                            &mut (*gc).data,
                            ('0' as i32 as u_int).wrapping_add(idx.wrapping_rem(10 as u_int))
                                as u_char,
                        );
                    } else {
                        utf8_set(&mut (*gc).data, '*' as i32 as u_char);
                    }
                }
            }
            PANE_LINES_DOUBLE => {
                (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
                utf8_copy(&mut (*gc).data, &*tty_acs_double_borders(cell_type));
            }
            PANE_LINES_HEAVY => {
                (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
                utf8_copy(&mut (*gc).data, &*tty_acs_heavy_borders(cell_type));
            }
            PANE_LINES_SIMPLE => {
                (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
                utf8_set(
                    &mut (*gc).data,
                    SIMPLE_BORDERS[cell_type as usize] as u_char,
                );
            }
            PANE_LINES_SPACES => {
                (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
                utf8_set(&mut (*gc).data, ' ' as i32 as u_char);
            }
            _ => {
                (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
                utf8_set(&mut (*gc).data, CELL_BORDERS[cell_type as usize] as u_char);
            }
        };
    }
}
pub(crate) unsafe fn screen_redraw_two_panes(
    mut w: *mut window,
    mut type_0: *mut layout_type,
) -> ::core::ffi::c_int {
    unsafe {
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut count: u_int = 0 as u_int;
        wp = window_panes_first(w);
        while !wp.is_null() {
            if !(window_pane_is_floating(wp) != 0 || (*wp).layout_cell.is_null()) {
                count = count.wrapping_add(1);
                if count > 2 as u_int || (*(*wp).layout_cell).parent.is_null() {
                    return 0 as ::core::ffi::c_int;
                }
                *type_0 = (*(*(*wp).layout_cell).parent).type_0;
            }
            wp = window_panes_next(w, wp);
        }
        if count <= 1 as u_int {
            return 0 as ::core::ffi::c_int;
        }
        1 as ::core::ffi::c_int
    }
}
pub(crate) unsafe fn screen_redraw_pane_border(
    ctx: &mut screen_redraw_ctx,
    mut wp: *mut window_pane,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
) -> screen_redraw_border_type {
    unsafe {
        let mut w: *mut window = (*wp).window;
        let mut oo: *mut options = options_ptr(&(*w).options);
        let mut ex: ::core::ffi::c_int =
            ((*wp).xoff as u_int).wrapping_add((*wp).sx) as ::core::ffi::c_int;
        let mut ey: ::core::ffi::c_int =
            ((*wp).yoff as u_int).wrapping_add((*wp).sy) as ::core::ffi::c_int;
        let mut hsplit: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut vsplit: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut pane_status: ::core::ffi::c_int = ctx.pane_status;
        let mut sb_w: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut sx: ::core::ffi::c_int = (*wp).sx as ::core::ffi::c_int;
        let mut sy: ::core::ffi::c_int = (*wp).sy as ::core::ffi::c_int;
        let mut left: ::core::ffi::c_int = 0;
        let mut right: ::core::ffi::c_int = 0;
        let mut split_type: layout_type = LAYOUT_LEFTRIGHT;
        if px >= (*wp).xoff && px < ex && py >= (*wp).yoff && py < ey {
            return SCREEN_REDRAW_INSIDE;
        }
        if window_pane_show_scrollbar(wp) != 0 {
            sb_w = (*wp).scrollbar_style.width + (*wp).scrollbar_style.pad;
        }
        if window_pane_is_floating(wp) != 0 {
            left = (*wp).xoff - 1 as ::core::ffi::c_int;
            right = (*wp).xoff + sx;
            if (*w).sb != PANE_SCROLLBARS_OFF && (*w).sb_pos == PANE_SCROLLBARS_LEFT {
                left -= sb_w;
            } else {
                right += sb_w;
            }
            if py >= (*wp).yoff - 1 as ::core::ffi::c_int && py <= (*wp).yoff + sy {
                if px == left {
                    return SCREEN_REDRAW_BORDER_LEFT;
                }
                if px == right {
                    return SCREEN_REDRAW_BORDER_RIGHT;
                }
            }
            if px > left && px <= right {
                if py == (*wp).yoff - 1 as ::core::ffi::c_int {
                    return SCREEN_REDRAW_BORDER_TOP;
                }
                if py == (*wp).yoff + sy {
                    return SCREEN_REDRAW_BORDER_BOTTOM;
                }
            }
            return SCREEN_REDRAW_OUTSIDE;
        }
        match options_get_number(oo, c"pane-border-indicators".as_ptr()) {
            1 | 3 if screen_redraw_two_panes((*wp).window, &raw mut split_type) != 0 => {
                hsplit = (split_type as ::core::ffi::c_uint
                    == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint)
                    as ::core::ffi::c_int;
                vsplit = (split_type as ::core::ffi::c_uint
                    == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint)
                    as ::core::ffi::c_int;
            }
            _ => {}
        }
        if ((*wp).yoff == 0 as ::core::ffi::c_int || py >= (*wp).yoff - 1 as ::core::ffi::c_int)
            && py <= ey
        {
            if (*w).sb != PANE_SCROLLBARS_OFF && (*w).sb_pos == PANE_SCROLLBARS_LEFT {
                if (*wp).xoff - sb_w == 0 as ::core::ffi::c_int
                    && px == sx + sb_w
                    && (hsplit == 0 || hsplit != 0 && py <= sy / 2 as ::core::ffi::c_int)
                {
                    return SCREEN_REDRAW_BORDER_RIGHT;
                }
                if (*wp).xoff - sb_w != 0 as ::core::ffi::c_int {
                    if px == (*wp).xoff - sb_w - 1 as ::core::ffi::c_int
                        && (hsplit == 0 || hsplit != 0 && py > sy / 2 as ::core::ffi::c_int)
                    {
                        return SCREEN_REDRAW_BORDER_LEFT;
                    }
                    if px == (*wp).xoff + sx + sb_w - 1 as ::core::ffi::c_int {
                        return SCREEN_REDRAW_BORDER_RIGHT;
                    }
                }
            } else {
                if (*wp).xoff == 0 as ::core::ffi::c_int
                    && px == sx + sb_w
                    && (hsplit == 0 || hsplit != 0 && py <= sy / 2 as ::core::ffi::c_int)
                {
                    return SCREEN_REDRAW_BORDER_RIGHT;
                }
                if (*wp).xoff != 0 as ::core::ffi::c_int {
                    if px == (*wp).xoff - 1 as ::core::ffi::c_int
                        && (hsplit == 0 || hsplit != 0 && py > sy / 2 as ::core::ffi::c_int)
                    {
                        return SCREEN_REDRAW_BORDER_LEFT;
                    }
                    if px == (*wp).xoff + sx + sb_w {
                        return SCREEN_REDRAW_BORDER_RIGHT;
                    }
                }
            }
        }
        if vsplit != 0 && pane_status == PANE_STATUS_OFF {
            if (*wp).yoff == 0 as ::core::ffi::c_int
                && py == sy
                && px <= sx / 2 as ::core::ffi::c_int
            {
                return SCREEN_REDRAW_BORDER_BOTTOM;
            }
            if (*wp).yoff != 0 as ::core::ffi::c_int
                && py == (*wp).yoff - 1 as ::core::ffi::c_int
                && px > sx / 2 as ::core::ffi::c_int
            {
                return SCREEN_REDRAW_BORDER_TOP;
            }
        } else if (*w).sb != PANE_SCROLLBARS_OFF && (*w).sb_pos == PANE_SCROLLBARS_LEFT {
            if ((*wp).xoff - sb_w == 0 as ::core::ffi::c_int || px >= (*wp).xoff - sb_w)
                && (px <= ex || sb_w != 0 as ::core::ffi::c_int && px < ex + sb_w)
            {
                if pane_status != PANE_STATUS_BOTTOM
                    && (*wp).yoff != 0 as ::core::ffi::c_int
                    && py == (*wp).yoff - 1 as ::core::ffi::c_int
                {
                    return SCREEN_REDRAW_BORDER_TOP;
                }
                if pane_status != PANE_STATUS_TOP && py == ey {
                    return SCREEN_REDRAW_BORDER_BOTTOM;
                }
            }
        } else if ((*wp).xoff == 0 as ::core::ffi::c_int || px >= (*wp).xoff)
            && (px <= ex || sb_w != 0 as ::core::ffi::c_int && px < ex + sb_w)
        {
            if pane_status != PANE_STATUS_BOTTOM
                && (*wp).yoff != 0 as ::core::ffi::c_int
                && py == (*wp).yoff - 1 as ::core::ffi::c_int
            {
                return SCREEN_REDRAW_BORDER_TOP;
            }
            if pane_status != PANE_STATUS_TOP && py == ey {
                return SCREEN_REDRAW_BORDER_BOTTOM;
            }
        }
        SCREEN_REDRAW_OUTSIDE
    }
}
unsafe fn screen_redraw_cell_border1(
    ctx: &mut screen_redraw_ctx,
    mut sb_pos: ::core::ffi::c_int,
    mut sb_w: ::core::ffi::c_int,
    mut wp: *mut window_pane,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if sb_pos == PANE_SCROLLBARS_LEFT {
            if (px < (*wp).xoff - 1 as ::core::ffi::c_int - sb_w
                || px > (*wp).xoff + (*wp).sx as ::core::ffi::c_int)
                && (py < (*wp).yoff - 1 as ::core::ffi::c_int
                    || py > (*wp).yoff + (*wp).sy as ::core::ffi::c_int)
            {
                return -(1 as ::core::ffi::c_int);
            }
        } else if (px < (*wp).xoff - 1 as ::core::ffi::c_int
            || px > (*wp).xoff + (*wp).sx as ::core::ffi::c_int + sb_w)
            && (py < (*wp).yoff - 1 as ::core::ffi::c_int
                || py > (*wp).yoff + (*wp).sy as ::core::ffi::c_int)
        {
            return -(1 as ::core::ffi::c_int);
        }
        match screen_redraw_pane_border(ctx, wp, px, py) {
            SCREEN_REDRAW_INSIDE => 0 as ::core::ffi::c_int,
            SCREEN_REDRAW_OUTSIDE => -(1 as ::core::ffi::c_int),
            _ => 1 as ::core::ffi::c_int,
        }
    }
}
pub(crate) unsafe fn screen_redraw_cell_border(
    ctx: &mut screen_redraw_ctx,
    mut wp: *mut window_pane,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut c: *mut client = ctx.c;
        let mut w: *mut window = (*session_get_curw((*c).session)).window();
        let mut wp2: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut sx: ::core::ffi::c_int = (*w).sx as ::core::ffi::c_int;
        let mut sy: ::core::ffi::c_int = (*w).sy as ::core::ffi::c_int;
        let mut sb_w: ::core::ffi::c_int = 0;
        let mut n: ::core::ffi::c_int = 0;
        sb_w = (*wp).scrollbar_style.width + (*wp).scrollbar_style.pad;
        if window_pane_is_floating(wp) != 0 {
            n = screen_redraw_cell_border1(
                &mut *ctx,
                if (*w).sb != PANE_SCROLLBARS_OFF {
                    (*w).sb_pos
                } else {
                    0 as ::core::ffi::c_int
                },
                sb_w,
                wp,
                px,
                py,
            );
            if n == -(1 as ::core::ffi::c_int) {
                return 0 as ::core::ffi::c_int;
            }
            return n;
        }
        if ctx.pane_status == PANE_STATUS_BOTTOM {
            sy -= 1;
        }
        if px > sx || py > sy {
            return 0 as ::core::ffi::c_int;
        }
        if px == sx || py == sy {
            return 1 as ::core::ffi::c_int;
        }
        wp2 = window_pane_stack_first(w, PaneStack::ZIndex);
        while !wp2.is_null() {
            if !(window_pane_visible(wp2) == 0 || window_pane_is_floating(wp2) != 0) {
                n = screen_redraw_cell_border1(
                    &mut *ctx,
                    if (*w).sb != PANE_SCROLLBARS_OFF {
                        (*w).sb_pos
                    } else {
                        0 as ::core::ffi::c_int
                    },
                    sb_w,
                    wp2,
                    px,
                    py,
                );
                if n != -(1 as ::core::ffi::c_int) {
                    return n;
                }
            }
            wp2 = window_pane_stack_next(w, PaneStack::ZIndex, wp2);
        }
        0 as ::core::ffi::c_int
    }
}
pub(crate) unsafe fn screen_redraw_type_of_cell(
    ctx: &mut screen_redraw_ctx,
    mut wp: *mut window_pane,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut c: *mut client = ctx.c;
        let mut w: *mut window = (*session_get_curw((*c).session)).window();
        let mut pane_status: ::core::ffi::c_int = ctx.pane_status;
        let mut borders: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut sx: ::core::ffi::c_int = (*w).sx as ::core::ffi::c_int;
        let mut sy: ::core::ffi::c_int = (*w).sy as ::core::ffi::c_int;
        if pane_status == PANE_STATUS_BOTTOM {
            sy -= 1;
        }
        if px > sx || py > sy {
            return 12 as ::core::ffi::c_int;
        }
        if window_pane_is_floating(wp) == 0 {
            if px == 0 as ::core::ffi::c_int
                || screen_redraw_cell_border(ctx, wp, px - 1 as ::core::ffi::c_int, py) != 0
            {
                borders |= 8 as ::core::ffi::c_int;
            }
            if px <= sx && screen_redraw_cell_border(ctx, wp, px + 1 as ::core::ffi::c_int, py) != 0
            {
                borders |= 4 as ::core::ffi::c_int;
            }
            if pane_status == PANE_STATUS_TOP {
                if py != 0 as ::core::ffi::c_int
                    && screen_redraw_cell_border(ctx, wp, px, py - 1 as ::core::ffi::c_int) != 0
                {
                    borders |= 2 as ::core::ffi::c_int;
                }
                if screen_redraw_cell_border(ctx, wp, px, py + 1 as ::core::ffi::c_int) != 0 {
                    borders |= 1 as ::core::ffi::c_int;
                }
            } else if pane_status == PANE_STATUS_BOTTOM {
                if py == 0 as ::core::ffi::c_int
                    || screen_redraw_cell_border(ctx, wp, px, py - 1 as ::core::ffi::c_int) != 0
                {
                    borders |= 2 as ::core::ffi::c_int;
                }
                if py != sy
                    && screen_redraw_cell_border(ctx, wp, px, py + 1 as ::core::ffi::c_int) != 0
                {
                    borders |= 1 as ::core::ffi::c_int;
                }
            } else {
                if py == 0 as ::core::ffi::c_int
                    || screen_redraw_cell_border(ctx, wp, px, py - 1 as ::core::ffi::c_int) != 0
                {
                    borders |= 2 as ::core::ffi::c_int;
                }
                if screen_redraw_cell_border(ctx, wp, px, py + 1 as ::core::ffi::c_int) != 0 {
                    borders |= 1 as ::core::ffi::c_int;
                }
            }
        } else {
            if screen_redraw_cell_border(ctx, wp, px - 1 as ::core::ffi::c_int, py) != 0 {
                borders |= 8 as ::core::ffi::c_int;
            }
            if px <= sx && screen_redraw_cell_border(ctx, wp, px + 1 as ::core::ffi::c_int, py) != 0
            {
                borders |= 4 as ::core::ffi::c_int;
            }
            if pane_status == PANE_STATUS_TOP {
                if py != 0 as ::core::ffi::c_int
                    && screen_redraw_cell_border(ctx, wp, px, py - 1 as ::core::ffi::c_int) != 0
                {
                    borders |= 2 as ::core::ffi::c_int;
                }
                if screen_redraw_cell_border(ctx, wp, px, py + 1 as ::core::ffi::c_int) != 0 {
                    borders |= 1 as ::core::ffi::c_int;
                }
            } else if pane_status == PANE_STATUS_BOTTOM {
                if screen_redraw_cell_border(ctx, wp, px, py - 1 as ::core::ffi::c_int) != 0 {
                    borders |= 2 as ::core::ffi::c_int;
                }
                if py != sy
                    && screen_redraw_cell_border(ctx, wp, px, py + 1 as ::core::ffi::c_int) != 0
                {
                    borders |= 1 as ::core::ffi::c_int;
                }
            } else {
                if screen_redraw_cell_border(ctx, wp, px, py - 1 as ::core::ffi::c_int) != 0 {
                    borders |= 2 as ::core::ffi::c_int;
                }
                if screen_redraw_cell_border(ctx, wp, px, py + 1 as ::core::ffi::c_int) != 0 {
                    borders |= 1 as ::core::ffi::c_int;
                }
            }
        }
        match borders {
            15 => return 11 as ::core::ffi::c_int,
            14 => return 8 as ::core::ffi::c_int,
            13 => return 7 as ::core::ffi::c_int,
            12 => return 2 as ::core::ffi::c_int,
            11 => return 10 as ::core::ffi::c_int,
            10 => return 6 as ::core::ffi::c_int,
            9 => return 4 as ::core::ffi::c_int,
            7 => return 9 as ::core::ffi::c_int,
            6 => return 5 as ::core::ffi::c_int,
            5 => return 3 as ::core::ffi::c_int,
            3 => return 1 as ::core::ffi::c_int,
            _ => {}
        }
        12 as ::core::ffi::c_int
    }
}
/// What is drawn at `px`,`py`, and the pane the answer belongs to.
unsafe fn screen_redraw_check_cell(
    ctx: &mut screen_redraw_ctx,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
    wpp: &mut *mut window_pane,
) -> ::core::ffi::c_int {
    unsafe {
        let mut current_block: u64;
        let mut c: *mut client = ctx.c;
        let mut w: *mut window = (*session_get_curw((*c).session)).window();
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut start: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut sx: ::core::ffi::c_int = (*w).sx as ::core::ffi::c_int;
        let mut sy: ::core::ffi::c_int = (*w).sy as ::core::ffi::c_int;
        let mut pane_status: ::core::ffi::c_int = ctx.pane_status;
        let mut border: ::core::ffi::c_int = 0;
        let mut pane_status_line: ::core::ffi::c_int = 0;
        let mut tiled_only: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut left: ::core::ffi::c_int = 0;
        let mut right: ::core::ffi::c_int = 0;
        let mut sb_w: ::core::ffi::c_int = 0;
        *wpp = ::core::ptr::null_mut::<window_pane>();
        if px > sx || py > sy {
            return 12 as ::core::ffi::c_int;
        }
        wp = window_pane_stack_first(w, PaneStack::ZIndex);
        while !wp.is_null() {
            if !(window_pane_is_floating(wp) != 0 && (px >= sx || py >= sy)) {
                sb_w = (*wp).scrollbar_style.width + (*wp).scrollbar_style.pad;
                if (*w).sb != PANE_SCROLLBARS_OFF && (*w).sb_pos == PANE_SCROLLBARS_LEFT {
                    if px >= (*wp).xoff - 1 as ::core::ffi::c_int - sb_w
                        && px <= (*wp).xoff + (*wp).sx as ::core::ffi::c_int
                        && (py >= (*wp).yoff - 1 as ::core::ffi::c_int
                            && py <= (*wp).yoff + (*wp).sy as ::core::ffi::c_int)
                    {
                        break;
                    }
                } else if px >= (*wp).xoff - 1 as ::core::ffi::c_int
                    && px <= (*wp).xoff + (*wp).sx as ::core::ffi::c_int + sb_w
                    && (py >= (*wp).yoff - 1 as ::core::ffi::c_int
                        && py <= (*wp).yoff + (*wp).sy as ::core::ffi::c_int)
                {
                    break;
                }
            }
            wp = window_pane_stack_next(w, PaneStack::ZIndex, wp);
        }
        if !wp.is_null() {
            start = wp;
        } else {
            wp = server_client_get_pane(c);
            start = wp;
        }
        if wp.is_null() {
            return 12 as ::core::ffi::c_int;
        }
        if px == sx || py == sy {
            return screen_redraw_type_of_cell(ctx, wp, px, py);
        }
        if window_pane_is_floating(wp) == 0 {
            tiled_only = 1 as ::core::ffi::c_int;
        }
        loop {
            if !(window_pane_visible(wp) == 0)
                && !(tiled_only != 0 && window_pane_is_floating(wp) != 0)
            {
                *wpp = wp;
                sb_w = (*wp).scrollbar_style.width + (*wp).scrollbar_style.pad;
                if (*w).sb != PANE_SCROLLBARS_OFF && (*w).sb_pos == PANE_SCROLLBARS_LEFT {
                    if (px < (*wp).xoff - 1 as ::core::ffi::c_int - sb_w
                        || px > (*wp).xoff + (*wp).sx as ::core::ffi::c_int)
                        && (py < (*wp).yoff - 1 as ::core::ffi::c_int
                            || py > (*wp).yoff + (*wp).sy as ::core::ffi::c_int)
                    {
                        current_block = 13503835911103092327;
                    } else {
                        current_block = 16924917904204750491;
                    }
                } else if (px < (*wp).xoff - 1 as ::core::ffi::c_int
                    || px > (*wp).xoff + (*wp).sx as ::core::ffi::c_int + sb_w)
                    && (py < (*wp).yoff - 1 as ::core::ffi::c_int
                        || py > (*wp).yoff + (*wp).sy as ::core::ffi::c_int)
                {
                    current_block = 13503835911103092327;
                } else {
                    current_block = 16924917904204750491;
                }
                match current_block {
                    13503835911103092327 => {}
                    _ => {
                        if pane_status != PANE_STATUS_OFF {
                            if pane_status == PANE_STATUS_TOP {
                                pane_status_line = (*wp).yoff - 1 as ::core::ffi::c_int;
                            } else {
                                pane_status_line = (*wp).yoff + (*wp).sy as ::core::ffi::c_int;
                            }
                            left = (*wp).xoff + 2 as ::core::ffi::c_int;
                            right = (*wp).xoff
                                + 2 as ::core::ffi::c_int
                                + (*wp).status_size as ::core::ffi::c_int
                                - 1 as ::core::ffi::c_int;
                            if py == pane_status_line + ctx.oy && px >= left && px <= right {
                                return 0 as ::core::ffi::c_int;
                            }
                        }
                        if window_pane_show_scrollbar(wp) != 0 {
                            sb_w = (*wp).scrollbar_style.width + (*wp).scrollbar_style.pad;
                            if ((*wp).yoff == 0 as ::core::ffi::c_int
                                && py < (*wp).sy as ::core::ffi::c_int
                                || py >= (*wp).yoff
                                    && py < (*wp).yoff + (*wp).sy as ::core::ffi::c_int)
                                && ((*w).sb_pos == PANE_SCROLLBARS_RIGHT
                                    && (px >= (*wp).xoff + (*wp).sx as ::core::ffi::c_int
                                        && px < (*wp).xoff + (*wp).sx as ::core::ffi::c_int + sb_w)
                                    || (*w).sb_pos == PANE_SCROLLBARS_LEFT
                                        && (px >= (*wp).xoff - sb_w && px < (*wp).xoff))
                            {
                                return 13 as ::core::ffi::c_int;
                            }
                        }
                        border = screen_redraw_pane_border(ctx, wp, px, py) as ::core::ffi::c_int;
                        if border == SCREEN_REDRAW_INSIDE as ::core::ffi::c_int {
                            return 0 as ::core::ffi::c_int;
                        }
                        if !(border == SCREEN_REDRAW_OUTSIDE as ::core::ffi::c_int) {
                            return screen_redraw_type_of_cell(ctx, wp, px, py);
                        }
                    }
                }
            }
            wp = window_pane_stack_next(w, PaneStack::ZIndex, wp);
            if wp.is_null() {
                wp = window_pane_stack_first(w, PaneStack::ZIndex);
            }
            if !(wp != start) {
                break;
            }
        }
        12 as ::core::ffi::c_int
    }
}
pub(crate) unsafe fn screen_redraw_check_is(
    ctx: &mut screen_redraw_ctx,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    unsafe {
        let mut border: screen_redraw_border_type = SCREEN_REDRAW_OUTSIDE;
        if wp.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        border = screen_redraw_pane_border(ctx, wp, px, py);
        if border as ::core::ffi::c_uint
            != SCREEN_REDRAW_INSIDE as ::core::ffi::c_int as ::core::ffi::c_uint
            && border as ::core::ffi::c_uint
                != SCREEN_REDRAW_OUTSIDE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return 1 as ::core::ffi::c_int;
        }
        0 as ::core::ffi::c_int
    }
}
unsafe fn screen_redraw_make_pane_status(
    mut c: *mut client,
    mut wp: *mut window_pane,
    mut rctx: &mut screen_redraw_ctx,
    mut pane_lines: pane_lines,
) -> ::core::ffi::c_int {
    unsafe {
        let mut w: *mut window = (*wp).window;
        let mut gc = grid_default_cell;
        let mut fmt: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut border_option: *const ::core::ffi::c_char =
            ::core::ptr::null::<::core::ffi::c_char>();
        let mut sle: *mut style_line_entry = &raw mut (*wp).border_status_line;
        let mut pane_status: ::core::ffi::c_int = rctx.pane_status;
        let mut sb_w: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut max_width: ::core::ffi::c_int = 0;
        let mut width: u_int = 0;
        let mut i: u_int = 0;
        let mut cell_type: u_int = 0;
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut ctx = screen_write_ctx::default();
        if window_pane_show_scrollbar(wp) != 0 {
            sb_w = (*wp).scrollbar_style.width + (*wp).scrollbar_style.pad;
        }
        let mut ft = format_create(
            c,
            ::core::ptr::null_mut::<cmdq_item>(),
            (FORMAT_PANE | (*wp).id) as ::core::ffi::c_int,
            FORMAT_STATUS,
        );
        format_defaults(&mut ft, c, (*c).session, session_get_curw((*c).session), wp);
        if wp == server_client_get_pane(c) {
            border_option = c"pane-active-border-style".as_ptr();
        } else {
            border_option = c"pane-border-style".as_ptr();
        }
        style_apply(
            &raw mut gc,
            options_ptr(&(*wp).options),
            border_option,
            Some(&mut ft),
        );
        fmt = options_get_string(options_ptr(&(*wp).options), c"pane-border-format".as_ptr());
        let expanded = format_expand_time(&mut ft, ::core::ffi::CStr::from_ptr(fmt));
        if (*wp).sx < 4 as u_int {
            width = 0 as u_int;
        } else {
            width = (*wp)
                .sx
                .wrapping_add(sb_w as u_int)
                .wrapping_sub(2 as u_int);
        }
        max_width = (*w).sx as ::core::ffi::c_int - ((*wp).xoff + 2 as ::core::ffi::c_int);
        if max_width < 0 as ::core::ffi::c_int {
            max_width = 0 as ::core::ffi::c_int;
        }
        if width > max_width as u_int {
            width = max_width as u_int;
        }
        (*wp).status_size = width as size_t;
        let mut old = ::core::mem::replace(
            &mut (*wp).status_screen,
            screen::new(width, 1 as u_int, 0 as u_int),
        );
        (*wp).status_screen.mode = 0 as ::core::ffi::c_int;
        screen_write_start(&mut ctx, &raw mut (*wp).status_screen);
        i = 0 as u_int;
        while i < width {
            px = (((*wp).xoff + 2 as ::core::ffi::c_int) as u_int).wrapping_add(i);
            if pane_status == PANE_STATUS_TOP {
                py = ((*wp).yoff - 1 as ::core::ffi::c_int) as u_int;
            } else {
                py = ((*wp).yoff as u_int).wrapping_add((*wp).sy);
            }
            cell_type = screen_redraw_type_of_cell(
                rctx,
                wp,
                px as ::core::ffi::c_int,
                py as ::core::ffi::c_int,
            ) as u_int;
            screen_redraw_border_set(
                w,
                wp,
                pane_lines,
                cell_type as ::core::ffi::c_int,
                &raw mut gc,
            );
            screen_write_cell(&mut ctx, &raw mut gc);
            i = i.wrapping_add(1);
        }
        gc.attr = (gc.attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
        screen_write_cursormove(
            &mut ctx,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        style_ranges_free(&raw mut (*sle).ranges);
        format_draw(
            &mut ctx,
            &gc,
            width,
            expanded.as_bytes(),
            Some(&mut (*sle).ranges),
            0 as ::core::ffi::c_int,
        );
        screen_write_stop(&mut ctx);
        (*sle).expanded = Some(expanded);
        if grid_compare(screen_grid(&(*wp).status_screen), screen_grid(&old))
            == 0 as ::core::ffi::c_int
        {
            screen_free(&raw mut old);
            return 0 as ::core::ffi::c_int;
        }
        screen_free(&raw mut old);
        1 as ::core::ffi::c_int
    }
}
unsafe fn screen_redraw_draw_pane_status(ctx: &mut screen_redraw_ctx) {
    unsafe {
        let mut c: *mut client = ctx.c;
        let mut w: *mut window = (*session_get_curw((*c).session)).window();
        let mut tty: *mut tty = &raw mut (*c).tty;
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
        let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
        let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
        let mut i: u_int = 0;
        let mut l: u_int = 0;
        let mut x: u_int = 0;
        let mut width: u_int = 0;
        let mut size: u_int = 0;
        let mut xoff: ::core::ffi::c_int = 0;
        let mut yoff: ::core::ffi::c_int = 0;
        log_debug(
            c"%s: %s @%u".as_ptr(),
            fmt_args![
                c"screen_redraw_draw_pane_status".as_ptr(),
                cstr_ptr(&(*c).name),
                (*w).id
            ],
        );
        wp = window_panes_first(w);
        while !wp.is_null() {
            if !(window_pane_visible(wp) == 0) {
                s = &raw mut (*wp).status_screen;
                size = (*wp).status_size as u_int;
                if ctx.pane_status == PANE_STATUS_TOP {
                    yoff = (*wp).yoff - 1 as ::core::ffi::c_int;
                } else {
                    yoff = ((*wp).yoff as u_int).wrapping_add((*wp).sy) as ::core::ffi::c_int;
                }
                xoff = (*wp).xoff + 2 as ::core::ffi::c_int;
                if !(xoff + size as ::core::ffi::c_int <= ctx.ox
                    || xoff >= ctx.ox + ctx.sx as ::core::ffi::c_int
                    || yoff < ctx.oy
                    || yoff >= ctx.oy + ctx.sy as ::core::ffi::c_int)
                {
                    if xoff >= ctx.ox
                        && (xoff as u_int).wrapping_add(size)
                            <= (ctx.ox as u_int).wrapping_add(ctx.sx)
                    {
                        l = 0 as u_int;
                        x = (xoff - ctx.ox) as u_int;
                        width = size;
                    } else if xoff < ctx.ox
                        && (xoff as u_int).wrapping_add(size)
                            > (ctx.ox as u_int).wrapping_add(ctx.sx)
                    {
                        l = (ctx.ox - xoff) as u_int;
                        x = 0 as u_int;
                        width = ctx.sx;
                    } else if xoff < ctx.ox {
                        l = (ctx.ox - xoff) as u_int;
                        x = 0 as u_int;
                        width = size.wrapping_sub(l);
                    } else {
                        l = 0 as u_int;
                        x = (xoff - ctx.ox) as u_int;
                        width = size.wrapping_sub(x);
                    }
                    r = tty_check_overlay_range(tty, x, yoff as u_int, width);
                    screen_redraw_clip_visible_ranges(
                        wp,
                        x as ::core::ffi::c_int,
                        yoff,
                        width,
                        &mut *r,
                    );
                    if ctx.statustop != 0 {
                        yoff = (yoff as u_int).wrapping_add(ctx.statuslines) as ::core::ffi::c_int
                            as ::core::ffi::c_int;
                    }
                    i = 0 as u_int;
                    while i < (*r).used {
                        ri = (*r).ranges.as_mut_ptr().offset(i as isize);
                        if !((*ri).nx == 0 as u_int) {
                            tty_draw_line(
                                tty,
                                s,
                                l.wrapping_add((*ri).px.wrapping_sub(x)),
                                0 as u_int,
                                (*ri).nx,
                                (*ri).px,
                                (yoff - ctx.oy) as u_int,
                                &raw const grid_default_cell,
                                ::core::ptr::null_mut::<colour_palette>(),
                            );
                        }
                        i = i.wrapping_add(1);
                    }
                }
            }
            wp = window_panes_next(w, wp);
        }
        tty_cursor(tty, 0 as u_int, 0 as u_int);
    }
}
unsafe fn screen_redraw_update(ctx: &mut screen_redraw_ctx, mut flags: uint64_t) -> uint64_t {
    unsafe {
        let mut c: *mut client = ctx.c;
        let mut w: *mut window = (*session_get_curw((*c).session)).window();
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut redraw: ::core::ffi::c_int = 0;
        let mut lines: pane_lines = PANE_LINES_SINGLE;
        if (*c).message_string.is_some() {
            redraw = status_message_redraw(c);
        } else if (*c).prompt_string.is_some() {
            redraw = status_prompt_redraw(c);
        } else {
            redraw = status_redraw(c);
        }
        if redraw == 0 && !flags & CLIENT_REDRAWSTATUSALWAYS as uint64_t != 0 {
            flags &= !CLIENT_REDRAWSTATUS as uint64_t;
        }
        if (*c).overlay().is_some() {
            flags |= CLIENT_REDRAWOVERLAY as uint64_t;
        }
        if ctx.pane_status != PANE_STATUS_OFF {
            lines = ctx.pane_lines;
            redraw = 0 as ::core::ffi::c_int;
            wp = window_panes_first(w);
            while !wp.is_null() {
                if screen_redraw_make_pane_status(c, wp, ctx, lines) != 0 {
                    redraw = 1 as ::core::ffi::c_int;
                }
                wp = window_panes_next(w, wp);
            }
            if redraw != 0 {
                flags |= CLIENT_REDRAWBORDERS as uint64_t;
            }
        }
        flags
    }
}
unsafe fn screen_redraw_set_context(mut c: *mut client, ctx: &mut screen_redraw_ctx) {
    unsafe {
        let mut s: *mut session = (*c).session;
        let mut oo: *mut options = session_options(s);
        let mut w: *mut window = (*session_get_curw(s)).window();
        let mut wo: *mut options = options_ptr(&(*w).options);
        let mut lines: u_int = 0;
        *ctx = screen_redraw_ctx::default();
        ctx.c = c;
        lines = status_line_size(c);
        if (*c).message_string.is_some() || (*c).prompt_string.is_some() {
            lines = if lines == 0 as u_int {
                1 as u_int
            } else {
                lines
            };
        }
        if lines != 0 as u_int
            && options_get_number(oo, c"status-position".as_ptr()) == 0 as ::core::ffi::c_longlong
        {
            ctx.statustop = 1 as ::core::ffi::c_int;
        }
        ctx.statuslines = lines;
        ctx.pane_status =
            options_get_number(wo, c"pane-border-status".as_ptr()) as ::core::ffi::c_int;
        ctx.pane_lines = options_get_number(wo, c"pane-border-lines".as_ptr()) as pane_lines;
        {
            let (_bigger, off_x, off_y, off_sx, off_sy) = tty_window_offset(&raw mut (*c).tty);
            (ctx.ox, ctx.oy) = (off_x as ::core::ffi::c_int, off_y as ::core::ffi::c_int);
            (ctx.sx, ctx.sy) = (off_sx, off_sy);
        }
        log_debug(
            c"%s: %s @%u ox=%u oy=%u sx=%u sy=%u %u/%d".as_ptr(),
            fmt_args![
                c"screen_redraw_set_context".as_ptr(),
                cstr_ptr(&(*c).name),
                (*w).id,
                ctx.ox,
                ctx.oy,
                ctx.sx,
                ctx.sy,
                ctx.statuslines,
                ctx.statustop
            ],
        );
    }
}
pub unsafe fn screen_redraw_screen(mut c: *mut client) {
    unsafe {
        let mut ctx = screen_redraw_ctx::default();
        let mut flags: uint64_t = 0;
        if (*c).flags & CLIENT_SUSPENDED as uint64_t != 0 {
            return;
        }
        screen_redraw_set_context(c, &mut ctx);
        flags = screen_redraw_update(&mut ctx, (*c).flags);
        if flags as ::core::ffi::c_ulonglong & CLIENT_ALLREDRAWFLAGS
            == 0 as ::core::ffi::c_ulonglong
        {
            return;
        }
        tty_sync_start(&raw mut (*c).tty);
        tty_update_mode(
            &raw mut (*c).tty,
            (*c).tty.mode,
            ::core::ptr::null_mut::<screen>(),
        );
        if flags & (CLIENT_REDRAWWINDOW | CLIENT_REDRAWBORDERS) as uint64_t != 0 {
            log_debug(
                c"%s: redrawing borders".as_ptr(),
                fmt_args![cstr_ptr(&(*c).name)],
            );
            screen_redraw_draw_borders(&mut ctx);
            if ctx.pane_status != PANE_STATUS_OFF {
                screen_redraw_draw_pane_status(&mut ctx);
            }
            screen_redraw_draw_pane_scrollbars(&mut ctx);
        }
        if flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
            log_debug(
                c"%s: redrawing panes".as_ptr(),
                fmt_args![cstr_ptr(&(*c).name)],
            );
            screen_redraw_draw_panes(&mut ctx);
            screen_redraw_draw_pane_scrollbars(&mut ctx);
        }
        if ctx.statuslines != 0 as u_int
            && flags & (CLIENT_REDRAWSTATUS | CLIENT_REDRAWSTATUSALWAYS) as uint64_t != 0
        {
            log_debug(
                c"%s: redrawing status".as_ptr(),
                fmt_args![cstr_ptr(&(*c).name)],
            );
            screen_redraw_draw_status(&mut ctx);
        }
        if (*c).overlay().is_some() && flags & CLIENT_REDRAWOVERLAY as uint64_t != 0 {
            log_debug(
                c"%s: redrawing overlay".as_ptr(),
                fmt_args![cstr_ptr(&(*c).name)],
            );
            (*c).overlay()
                .draw(c, (*c).current_overlay_data(), &mut ctx);
        }
        tty_reset(&raw mut (*c).tty);
    }
}
pub unsafe fn screen_redraw_pane(
    mut c: *mut client,
    mut wp: *mut window_pane,
    mut redraw_scrollbar_only: ::core::ffi::c_int,
) {
    unsafe {
        let mut ctx = screen_redraw_ctx::default();
        if window_pane_visible(wp) == 0 {
            return;
        }
        screen_redraw_set_context(c, &mut ctx);
        tty_sync_start(&raw mut (*c).tty);
        tty_update_mode(
            &raw mut (*c).tty,
            (*c).tty.mode,
            ::core::ptr::null_mut::<screen>(),
        );
        if redraw_scrollbar_only == 0 {
            screen_redraw_draw_pane(&mut ctx, wp);
        }
        if window_pane_show_scrollbar(wp) != 0 {
            screen_redraw_draw_pane_scrollbar(&mut ctx, wp);
        }
        tty_reset(&raw mut (*c).tty);
    }
}
unsafe fn screen_redraw_draw_borders_style(
    ctx: &mut screen_redraw_ctx,
    mut x: u_int,
    mut y: u_int,
    mut wp: *mut window_pane,
    mut ngc: *mut grid_cell,
) {
    unsafe {
        let mut c: *mut client = ctx.c;
        let mut s: *mut session = (*c).session;
        let mut active: *mut window_pane = server_client_get_pane(c);
        let mut gc: *mut grid_cell = ::core::ptr::null_mut::<grid_cell>();
        let mut border_option: *const ::core::ffi::c_char =
            ::core::ptr::null::<::core::ffi::c_char>();
        let mut flag: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
        if window_pane_is_floating(wp) != 0 && wp == active
            || window_pane_is_floating(wp) == 0
                && screen_redraw_check_is(
                    ctx,
                    x as ::core::ffi::c_int,
                    y as ::core::ffi::c_int,
                    active,
                ) != 0
        {
            flag = &raw mut (*wp).active_border_gc_set;
            gc = &raw mut (*wp).active_border_gc;
            border_option = c"pane-active-border-style".as_ptr();
        } else {
            flag = &raw mut (*wp).border_gc_set;
            gc = &raw mut (*wp).border_gc;
            border_option = c"pane-border-style".as_ptr();
        }
        if *flag == 0 {
            let mut ft = format_create_defaults(
                ::core::ptr::null_mut::<cmdq_item>(),
                c,
                s,
                session_get_curw(s),
                wp,
            );
            style_apply(
                gc,
                options_ptr(&(*wp).options),
                border_option,
                Some(&mut ft),
            );
            *flag = 1 as ::core::ffi::c_int;
        }
        *ngc = *gc;
    }
}
unsafe fn screen_redraw_draw_border_arrows(
    ctx: &mut screen_redraw_ctx,
    mut i: ::core::ffi::c_int,
    mut j: ::core::ffi::c_int,
    mut cell_type: u_int,
    mut wp: *mut window_pane,
    mut active: *mut window_pane,
    mut gc: *mut grid_cell,
) {
    unsafe {
        let mut c: *mut client = ctx.c;
        let mut s: *mut session = (*c).session;
        let mut w: *mut window = (*session_get_curw(s)).window();
        let mut oo: *mut options = options_ptr(&(*w).options);
        let mut x: u_int = (ctx.ox + i) as u_int;
        let mut y: u_int = (ctx.oy + j) as u_int;
        let mut value: ::core::ffi::c_int = 0;
        let mut arrows: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut border: ::core::ffi::c_int = 0;
        let mut type_0: layout_type = LAYOUT_LEFTRIGHT;
        if wp.is_null() {
            return;
        }
        if i != (*wp).xoff + 1 as ::core::ffi::c_int && j != (*wp).yoff + 1 as ::core::ffi::c_int {
            return;
        }
        if wp != active {
            if window_pane_is_floating(active) != 0 {
                return;
            }
            if window_pane_is_floating(wp) != 0 {
                return;
            }
        }
        value = options_get_number(oo, c"pane-border-indicators".as_ptr()) as ::core::ffi::c_int;
        if value != PANE_BORDER_ARROWS && value != PANE_BORDER_BOTH {
            return;
        }
        border = screen_redraw_pane_border(
            ctx,
            active,
            x as ::core::ffi::c_int,
            y as ::core::ffi::c_int,
        ) as ::core::ffi::c_int;
        if border == SCREEN_REDRAW_INSIDE as ::core::ffi::c_int {
            return;
        }
        if i == (*wp).xoff + 1 as ::core::ffi::c_int {
            if border == SCREEN_REDRAW_OUTSIDE as ::core::ffi::c_int {
                if screen_redraw_two_panes((*wp).window, &raw mut type_0) != 0 {
                    if active == window_panes_first(w) {
                        border = SCREEN_REDRAW_BORDER_BOTTOM as ::core::ffi::c_int;
                    } else {
                        border = SCREEN_REDRAW_BORDER_TOP as ::core::ffi::c_int;
                    }
                    arrows = 1 as ::core::ffi::c_int;
                }
            } else if cell_type == CELL_LEFTRIGHT as u_int {
                arrows = 1 as ::core::ffi::c_int;
            } else if cell_type == CELL_TOPJOIN as u_int
                && border == SCREEN_REDRAW_BORDER_BOTTOM as ::core::ffi::c_int
            {
                arrows = 1 as ::core::ffi::c_int;
            } else if cell_type == CELL_BOTTOMJOIN as u_int
                && border == SCREEN_REDRAW_BORDER_TOP as ::core::ffi::c_int
            {
                arrows = 1 as ::core::ffi::c_int;
            }
        }
        if j == (*wp).yoff + 1 as ::core::ffi::c_int {
            if border == SCREEN_REDRAW_OUTSIDE as ::core::ffi::c_int {
                if screen_redraw_two_panes((*wp).window, &raw mut type_0) != 0 {
                    if active == window_panes_first(w) {
                        border = SCREEN_REDRAW_BORDER_RIGHT as ::core::ffi::c_int;
                    } else {
                        border = SCREEN_REDRAW_BORDER_LEFT as ::core::ffi::c_int;
                    }
                    arrows = 1 as ::core::ffi::c_int;
                }
            } else if cell_type == CELL_TOPBOTTOM as u_int {
                arrows = 1 as ::core::ffi::c_int;
            } else if cell_type == CELL_LEFTJOIN as u_int
                && border == SCREEN_REDRAW_BORDER_RIGHT as ::core::ffi::c_int
            {
                arrows = 1 as ::core::ffi::c_int;
            } else if cell_type == CELL_RIGHTJOIN as u_int
                && border == SCREEN_REDRAW_BORDER_LEFT as ::core::ffi::c_int
            {
                arrows = 1 as ::core::ffi::c_int;
            }
        }
        if arrows != 0 {
            (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
            utf8_set(&mut (*gc).data, BORDER_MARKERS[border as usize] as u_char);
        }
    }
}
unsafe fn screen_redraw_draw_borders_cell(ctx: &mut screen_redraw_ctx, mut i: u_int, mut j: u_int) {
    unsafe {
        let mut c: *mut client = ctx.c;
        let mut s: *mut session = (*c).session;
        let mut w: *mut window = (*session_get_curw(s)).window();
        let mut oo: *mut options = options_ptr(&(*w).options);
        let mut tty: *mut tty = &raw mut (*c).tty;
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut active: *mut window_pane = server_client_get_pane(c);
        let mut gc = grid_default_cell;
        let mut cell_type: u_int = 0;
        let mut x: u_int = (ctx.ox as u_int).wrapping_add(i);
        let mut y: u_int = (ctx.oy as u_int).wrapping_add(j);
        let mut isolates: ::core::ffi::c_int = 0;
        let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
        if (*c).overlay_check().is_some() {
            r = (*c)
                .overlay_check()
                .call(c, (*c).current_overlay_data(), x, y, 1 as u_int);
            if server_client_ranges_is_empty(r) != 0 {
                return;
            }
        }
        cell_type = screen_redraw_check_cell(
            &mut *ctx,
            x as ::core::ffi::c_int,
            y as ::core::ffi::c_int,
            &mut wp,
        ) as u_int;
        if cell_type == CELL_INSIDE as u_int || cell_type == CELL_SCROLLBAR as u_int {
            return;
        }
        if wp.is_null() || cell_type == CELL_OUTSIDE as u_int {
            if ctx.no_pane_gc_set == 0 {
                let mut ft = format_create_defaults(
                    ::core::ptr::null_mut::<cmdq_item>(),
                    c,
                    s,
                    session_get_curw(s),
                    ::core::ptr::null_mut::<window_pane>(),
                );
                ctx.no_pane_gc = grid_default_cell;
                style_add(
                    &raw mut ctx.no_pane_gc,
                    oo,
                    c"pane-border-style".as_ptr(),
                    Some(&mut ft),
                );
                ctx.no_pane_gc_set = 1 as ::core::ffi::c_int;
            }
            gc = ctx.no_pane_gc;
        } else {
            screen_redraw_draw_borders_style(&mut *ctx, x, y, wp, &raw mut gc);
            if server_is_marked(s, session_get_curw(s), marked_pane.pane()) != 0
                && screen_redraw_check_is(
                    ctx,
                    x as ::core::ffi::c_int,
                    y as ::core::ffi::c_int,
                    marked_pane.pane(),
                ) != 0
            {
                gc.attr = (gc.attr as ::core::ffi::c_int ^ GRID_ATTR_REVERSE) as u_short;
            }
        }
        screen_redraw_border_set(
            w,
            wp,
            ctx.pane_lines,
            cell_type as ::core::ffi::c_int,
            &raw mut gc,
        );
        if cell_type == CELL_TOPBOTTOM as u_int
            && (*c).flags & CLIENT_UTF8 as uint64_t != 0
            && tty_term_has(tty_term_of(&*tty), TTYC_BIDI) != 0
        {
            isolates = 1 as ::core::ffi::c_int;
        } else {
            isolates = 0 as ::core::ffi::c_int;
        }
        if ctx.statustop != 0 {
            tty_cursor(tty, i, ctx.statuslines.wrapping_add(j));
        } else {
            tty_cursor(tty, i, j);
        }
        if isolates != 0 {
            tty_puts(tty, END_ISOLATE.as_ptr());
        }
        screen_redraw_draw_border_arrows(
            &mut *ctx,
            i as ::core::ffi::c_int,
            j as ::core::ffi::c_int,
            cell_type,
            wp,
            active,
            &raw mut gc,
        );
        tty_cell(
            tty,
            &raw mut gc,
            &raw const grid_default_cell,
            ::core::ptr::null_mut::<colour_palette>(),
            ::core::ptr::null_mut::<hyperlinks>(),
        );
        if isolates != 0 {
            tty_puts(tty, START_ISOLATE.as_ptr());
        }
    }
}
unsafe fn screen_redraw_draw_borders(ctx: &mut screen_redraw_ctx) {
    unsafe {
        let mut c: *mut client = ctx.c;
        let mut s: *mut session = (*c).session;
        let mut w: *mut window = (*session_get_curw(s)).window();
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut i: u_int = 0;
        let mut j: u_int = 0;
        log_debug(
            c"%s: %s @%u".as_ptr(),
            fmt_args![
                c"screen_redraw_draw_borders".as_ptr(),
                cstr_ptr(&(*c).name),
                (*w).id
            ],
        );
        wp = window_panes_first(w);
        while !wp.is_null() {
            (*wp).border_gc_set = 0 as ::core::ffi::c_int;
            (*wp).active_border_gc_set = 0 as ::core::ffi::c_int;
            wp = window_panes_next(w, wp);
        }
        j = 0 as u_int;
        while j < (*c).tty.sy.wrapping_sub(ctx.statuslines) {
            i = 0 as u_int;
            while i < (*c).tty.sx {
                screen_redraw_draw_borders_cell(&mut *ctx, i, j);
                i = i.wrapping_add(1);
            }
            j = j.wrapping_add(1);
        }
    }
}
unsafe fn screen_redraw_draw_panes(ctx: &mut screen_redraw_ctx) {
    unsafe {
        let mut c: *mut client = ctx.c;
        let mut w: *mut window = (*session_get_curw((*c).session)).window();
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        log_debug(
            c"%s: %s @%u".as_ptr(),
            fmt_args![
                c"screen_redraw_draw_panes".as_ptr(),
                cstr_ptr(&(*c).name),
                (*w).id
            ],
        );
        wp = window_panes_first(w);
        while !wp.is_null() {
            if window_pane_visible(wp) != 0 {
                screen_redraw_draw_pane(&mut *ctx, wp);
            }
            wp = window_panes_next(w, wp);
        }
    }
}
unsafe fn screen_redraw_draw_status(ctx: &mut screen_redraw_ctx) {
    unsafe {
        let mut c: *mut client = ctx.c;
        let mut w: *mut window = (*session_get_curw((*c).session)).window();
        let mut tty: *mut tty = &raw mut (*c).tty;
        let mut s: *mut screen = (*c).status.active();
        let mut i: u_int = 0;
        let mut y: u_int = 0;
        log_debug(
            c"%s: %s @%u".as_ptr(),
            fmt_args![
                c"screen_redraw_draw_status".as_ptr(),
                cstr_ptr(&(*c).name),
                (*w).id
            ],
        );
        if ctx.statustop != 0 {
            y = 0 as u_int;
        } else {
            y = (*c).tty.sy.wrapping_sub(ctx.statuslines);
        }
        i = 0 as u_int;
        while i < ctx.statuslines {
            tty_draw_line(
                tty,
                s,
                0 as u_int,
                i,
                UINT_MAX,
                0 as u_int,
                y.wrapping_add(i),
                &raw const grid_default_cell,
                ::core::ptr::null_mut::<colour_palette>(),
            );
            i = i.wrapping_add(1);
        }
    }
}
pub unsafe fn screen_redraw_is_visible(
    mut r: *mut visible_ranges,
    mut px: u_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut i: u_int = 0;
        let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
        if r.is_null() {
            return 1 as ::core::ffi::c_int;
        }
        i = 0 as u_int;
        while i < (*r).used {
            ri = (*r).ranges.as_mut_ptr().offset(i as isize);
            if (*ri).nx != 0 as u_int && px >= (*ri).px && px < (*ri).px.wrapping_add((*ri).nx) {
                return 1 as ::core::ffi::c_int;
            }
            i = i.wrapping_add(1);
        }
        0 as ::core::ffi::c_int
    }
}
/// The ranges of the `width` cells at `px`,`py` of `base_wp` that no pane
/// in front of it covers, written into `r`.
pub unsafe fn screen_redraw_get_visible_ranges(
    base_wp: *mut window_pane,
    px: ::core::ffi::c_int,
    py: ::core::ffi::c_int,
    width: u_int,
    r: &mut visible_ranges,
) {
    unsafe { screen_redraw_visible_ranges(base_wp, px, py, width, r, true) }
}

/// Narrows the ranges `r` already holds to what no pane in front of
/// `base_wp` covers. The caller has already worked out which cells of the
/// `width` at `px`,`py` an overlay leaves it.
pub unsafe fn screen_redraw_clip_visible_ranges(
    base_wp: *mut window_pane,
    px: ::core::ffi::c_int,
    py: ::core::ffi::c_int,
    width: u_int,
    r: &mut visible_ranges,
) {
    unsafe { screen_redraw_visible_ranges(base_wp, px, py, width, r, false) }
}

/// The two of them: `seed` starts from the whole span asked for, and
/// otherwise the ranges the caller handed in are the ones to narrow.
unsafe fn screen_redraw_visible_ranges(
    mut base_wp: *mut window_pane,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
    mut width: u_int,
    out: &mut visible_ranges,
    seed: bool,
) {
    unsafe {
        let r: *mut visible_ranges = out;
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
        let mut found_self: ::core::ffi::c_int = 0;
        let mut sb_w: ::core::ffi::c_int = 0;
        let mut lb: ::core::ffi::c_int = 0;
        let mut rb: ::core::ffi::c_int = 0;
        let mut tb: ::core::ffi::c_int = 0;
        let mut bb: ::core::ffi::c_int = 0;
        let mut sx: ::core::ffi::c_int = 0;
        let mut ex: ::core::ffi::c_int = 0;
        let mut i: u_int = 0;
        let mut s: u_int = 0;
        if py < 0 as ::core::ffi::c_int || width == 0 as u_int {
            (*r).used = 0 as u_int;
            return;
        }
        if px < 0 as ::core::ffi::c_int {
            if -px as u_int >= width {
                (*r).used = 0 as u_int;
                return;
            }
            width = width.wrapping_sub(-px as u_int);
            px = 0 as ::core::ffi::c_int;
        }
        if base_wp.is_null() {
            if seed {
                server_client_ensure_ranges(r, 1 as u_int);
                (*r).ranges[0_usize].px = px as u_int;
                (*r).ranges[0_usize].nx = width;
                (*r).used = 1 as u_int;
            }
            return;
        }
        w = (*base_wp).window;
        if py as u_int >= (*w).sy {
            (*r).used = 0 as u_int;
            return;
        }
        if (px as u_int).wrapping_add(width) > (*w).sx {
            width = (*w).sx.wrapping_sub(px as u_int);
        }
        if seed {
            server_client_ensure_ranges(r, 1 as u_int);
            (*r).ranges[0_usize].px = px as u_int;
            (*r).ranges[0_usize].nx = width;
            (*r).used = 1 as u_int;
        }
        found_self = 0 as ::core::ffi::c_int;
        wp = window_pane_stack_last(w, PaneStack::ZIndex);
        while !wp.is_null() {
            if wp == base_wp {
                found_self = 1 as ::core::ffi::c_int;
            } else {
                tb = if (*wp).yoff > 0 as ::core::ffi::c_int {
                    (*wp).yoff - 1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
                bb = ((*wp).yoff as u_int).wrapping_add((*wp).sy) as ::core::ffi::c_int;
                if !(found_self == 0 || window_pane_visible(wp) == 0 || py < tb || py > bb)
                    && !(window_pane_is_floating(wp) == 0 && (py == tb || py == bb))
                {
                    sb_w = (*wp).scrollbar_style.width + (*wp).scrollbar_style.pad;
                    if window_pane_show_scrollbar(wp) == 0 {
                        sb_w = 0 as ::core::ffi::c_int;
                    }
                    i = 0 as u_int;
                    while i < (*r).used {
                        ri = (*r).ranges.as_mut_ptr().offset(i as isize);
                        if (*w).sb_pos == PANE_SCROLLBARS_LEFT {
                            if (*wp).xoff > sb_w {
                                lb = (*wp).xoff - 1 as ::core::ffi::c_int - sb_w;
                            } else {
                                lb = 0 as ::core::ffi::c_int;
                            }
                        } else if (*wp).xoff > 0 as ::core::ffi::c_int {
                            lb = (*wp).xoff - 1 as ::core::ffi::c_int;
                        } else {
                            lb = 0 as ::core::ffi::c_int;
                        }
                        if (*w).sb_pos == PANE_SCROLLBARS_LEFT {
                            rb = ((*wp).xoff as u_int).wrapping_add((*wp).sx) as ::core::ffi::c_int;
                        } else {
                            rb = ((*wp).xoff as u_int)
                                .wrapping_add((*wp).sx)
                                .wrapping_add(sb_w as u_int)
                                as ::core::ffi::c_int;
                        }
                        if rb > (*w).sx as ::core::ffi::c_int {
                            rb = (*w).sx.wrapping_sub(1 as u_int) as ::core::ffi::c_int;
                        }
                        sx = (*ri).px as ::core::ffi::c_int;
                        ex = (sx as u_int)
                            .wrapping_add((*ri).nx)
                            .wrapping_sub(1 as u_int)
                            as ::core::ffi::c_int;
                        if lb > sx && lb <= ex && rb > ex {
                            (*ri).nx = (lb - sx) as u_int;
                        } else if rb >= sx && rb <= ex && lb <= sx {
                            (*ri).nx = (ex - rb) as u_int;
                            (*ri).px = (rb + 1 as ::core::ffi::c_int) as u_int;
                        } else if lb > sx && rb <= ex {
                            server_client_ensure_ranges(r, (*r).used.wrapping_add(1 as u_int));
                            s = (*r).used;
                            while s > i {
                                *(*r).ranges.as_mut_ptr().offset(s as isize) = *(*r)
                                    .ranges
                                    .as_mut_ptr()
                                    .offset(s.wrapping_sub(1 as u_int) as isize);
                                s = s.wrapping_sub(1);
                            }
                            ri = (*r).ranges.as_mut_ptr().offset(i as isize);
                            (*(*r)
                                .ranges
                                .as_mut_ptr()
                                .offset(i.wrapping_add(1 as u_int) as isize))
                            .px = (rb + 1 as ::core::ffi::c_int) as u_int;
                            (*(*r)
                                .ranges
                                .as_mut_ptr()
                                .offset(i.wrapping_add(1 as u_int) as isize))
                            .nx = (ex - rb) as u_int;
                            (*ri).nx = (lb - sx) as u_int;
                            (*r).used = (*r).used.wrapping_add(1);
                        } else if lb <= sx && rb > ex {
                            (*ri).nx = 0 as u_int;
                        }
                        i = i.wrapping_add(1);
                    }
                }
            }
            wp = window_pane_stack_prev(w, PaneStack::ZIndex, wp);
        }
    }
}
unsafe fn screen_redraw_draw_pane(ctx: &mut screen_redraw_ctx, mut wp: *mut window_pane) {
    unsafe {
        let mut c: *mut client = ctx.c;
        let mut w: *mut window = (*session_get_curw((*c).session)).window();
        let mut tty: *mut tty = &raw mut (*c).tty;
        let mut s: *mut screen = (*wp).screen();
        let mut palette: *mut colour_palette = &raw mut (*wp).palette;
        let mut defaults = grid_default_cell;
        let mut j: u_int = 0;
        let mut k: u_int = 0;
        let mut woy: u_int = 0;
        let mut wx: u_int = 0;
        let mut wy: u_int = 0;
        let mut py: u_int = 0;
        let mut width: u_int = 0;
        let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
        let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
        if (*wp).base.mode & MODE_SYNC != 0 {
            screen_write_stop_sync(wp);
        }
        log_debug(
            c"%s: %s @%u %%%u".as_ptr(),
            fmt_args![
                c"screen_redraw_draw_pane".as_ptr(),
                cstr_ptr(&(*c).name),
                (*w).id,
                (*wp).id
            ],
        );
        if (*wp).xoff + (*wp).sx as ::core::ffi::c_int <= ctx.ox
            || (*wp).xoff >= ctx.ox + ctx.sx as ::core::ffi::c_int
        {
            return;
        }
        if ctx.statustop != 0 {
            woy = ctx.statuslines;
        } else {
            woy = 0 as u_int;
        }
        j = 0 as u_int;
        while j < (*wp).sy {
            if !(((*wp).yoff + j as ::core::ffi::c_int) < ctx.oy
                || (*wp).yoff + j as ::core::ffi::c_int >= ctx.oy + ctx.sy as ::core::ffi::c_int)
            {
                wy = ((*wp).yoff as u_int).wrapping_add(j);
                py = woy.wrapping_add(wy).wrapping_sub(ctx.oy as u_int);
                if !(py > (*tty).sy) {
                    if (*wp).xoff >= ctx.ox
                        && (*wp).xoff + (*wp).sx as ::core::ffi::c_int
                            <= ctx.ox + ctx.sx as ::core::ffi::c_int
                    {
                        wx = ((*wp).xoff - ctx.ox) as u_int;
                        width = (*wp).sx;
                    } else if (*wp).xoff < ctx.ox
                        && (*wp).xoff + (*wp).sx as ::core::ffi::c_int
                            > ctx.ox + ctx.sx as ::core::ffi::c_int
                    {
                        wx = 0 as u_int;
                        width = ctx.sx;
                    } else if (*wp).xoff < ctx.ox {
                        wx = 0 as u_int;
                        width = (*wp).sx.wrapping_sub((ctx.ox - (*wp).xoff) as u_int);
                    } else {
                        wx = ((*wp).xoff - ctx.ox) as u_int;
                        width = ctx.sx.wrapping_sub(wx);
                    }
                    r = tty_check_overlay_range(tty, wx, wy, width);
                    screen_redraw_clip_visible_ranges(
                        wp,
                        wx as ::core::ffi::c_int,
                        wy as ::core::ffi::c_int,
                        width,
                        &mut *r,
                    );
                    tty_default_colours(&raw mut defaults, wp);
                    k = 0 as u_int;
                    while k < (*r).used {
                        ri = (*r).ranges.as_mut_ptr().offset(k as isize);
                        if !((*ri).nx == 0 as u_int) {
                            log_debug(
                                c"%s: %s %%%u range %u (%u,%u) width %u, tty (%u,%u) width %u"
                                    .as_ptr(),
                                fmt_args![
                                    c"screen_redraw_draw_pane".as_ptr(),
                                    cstr_ptr(&(*c).name),
                                    (*wp).id,
                                    k,
                                    (*ri)
                                        .px
                                        .wrapping_add(ctx.ox as u_int)
                                        .wrapping_sub((*wp).xoff as u_int),
                                    j,
                                    (*ri).nx,
                                    (*ri).px,
                                    py,
                                    (*ri).nx
                                ],
                            );
                            tty_draw_line(
                                tty,
                                s,
                                (*ri)
                                    .px
                                    .wrapping_add(ctx.ox as u_int)
                                    .wrapping_sub((*wp).xoff as u_int),
                                j,
                                (*ri).nx,
                                (*ri).px,
                                py,
                                &raw mut defaults,
                                palette,
                            );
                        }
                        k = k.wrapping_add(1);
                    }
                }
            }
            j = j.wrapping_add(1);
        }
    }
}
unsafe fn screen_redraw_draw_pane_scrollbars(ctx: &mut screen_redraw_ctx) {
    unsafe {
        let mut c: *mut client = ctx.c;
        let mut w: *mut window = (*session_get_curw((*c).session)).window();
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        log_debug(
            c"%s: %s @%u".as_ptr(),
            fmt_args![
                c"screen_redraw_draw_pane_scrollbars".as_ptr(),
                cstr_ptr(&(*c).name),
                (*w).id
            ],
        );
        wp = window_panes_first(w);
        while !wp.is_null() {
            if window_pane_show_scrollbar(wp) != 0 && window_pane_visible(wp) != 0 {
                screen_redraw_draw_pane_scrollbar(&mut *ctx, wp);
            }
            wp = window_panes_next(w, wp);
        }
    }
}
unsafe fn screen_redraw_draw_pane_scrollbar(ctx: &mut screen_redraw_ctx, mut wp: *mut window_pane) {
    unsafe {
        let mut s: *mut screen = (*wp).screen();
        let mut percent_view: ::core::ffi::c_double = 0.;
        let mut sb: u_int = (*(*wp).window).sb as u_int;
        let mut total_height: u_int = 0;
        let mut sb_h: u_int = (*wp).sy;
        let mut sb_pos: u_int = (*(*wp).window).sb_pos as u_int;
        let mut slider_h: u_int = 0;
        let mut slider_y: u_int = 0;
        let mut sb_w: ::core::ffi::c_int = (*wp).scrollbar_style.width;
        let mut sb_pad: ::core::ffi::c_int = (*wp).scrollbar_style.pad;
        let mut cm_y: ::core::ffi::c_int = 0;
        let mut cm_size: ::core::ffi::c_int = 0;
        let mut xoff: ::core::ffi::c_int = (*wp).xoff;
        let mut sb_x: ::core::ffi::c_int = 0;
        let mut sb_y: ::core::ffi::c_int = (*wp).yoff;
        if window_pane_mode(wp) == WINDOW_PANE_NO_MODE {
            if sb == PANE_SCROLLBARS_MODAL as u_int {
                return;
            }
            total_height = (*screen_grid_ptr(s))
                .sy
                .wrapping_add((*screen_grid_ptr(s)).hsize);
            percent_view = sb_h as ::core::ffi::c_double / total_height as ::core::ffi::c_double;
            slider_h = (sb_h as ::core::ffi::c_double * percent_view) as u_int;
            slider_y = sb_h.wrapping_sub(slider_h);
        } else {
            if (*wp).modes.is_empty() {
                return;
            }
            let Some((offset, size)) = window_copy_get_current_offset(wp) else {
                return;
            };
            (cm_y, cm_size) = (offset as ::core::ffi::c_int, size as ::core::ffi::c_int);
            total_height = (cm_size as u_int).wrapping_add(sb_h);
            percent_view = sb_h as ::core::ffi::c_double
                / (cm_size as u_int).wrapping_add(sb_h) as ::core::ffi::c_double;
            slider_h = (sb_h as ::core::ffi::c_double * percent_view) as u_int;
            slider_y = (sb_h.wrapping_add(1 as u_int) as ::core::ffi::c_double
                * (cm_y as ::core::ffi::c_double / total_height as ::core::ffi::c_double))
                as u_int;
        }
        if sb_pos == PANE_SCROLLBARS_LEFT as u_int {
            sb_x = xoff - sb_w - sb_pad;
        } else {
            sb_x = (xoff as u_int).wrapping_add((*wp).sx) as ::core::ffi::c_int;
        }
        if slider_h < 1 as u_int {
            slider_h = 1 as u_int;
        }
        if slider_y >= sb_h {
            slider_y = sb_h.wrapping_sub(1 as u_int);
        }
        screen_redraw_draw_scrollbar(
            &mut *ctx,
            wp,
            sb_pos as ::core::ffi::c_int,
            sb_x,
            sb_y,
            sb_h,
            slider_h,
            slider_y,
        );
        (*wp).sb_slider_y = slider_y;
        (*wp).sb_slider_h = slider_h;
    }
}
unsafe fn screen_redraw_draw_scrollbar(
    ctx: &mut screen_redraw_ctx,
    mut wp: *mut window_pane,
    mut sb_pos: ::core::ffi::c_int,
    mut sb_x: ::core::ffi::c_int,
    mut sb_y: ::core::ffi::c_int,
    mut sb_h: u_int,
    mut slider_h: u_int,
    mut slider_y: u_int,
) {
    unsafe {
        let mut c: *mut client = ctx.c;
        let mut tty: *mut tty = &raw mut (*c).tty;
        let mut gc = grid_default_cell;
        let mut slgc = grid_default_cell;
        let mut gcp: *mut grid_cell = ::core::ptr::null_mut::<grid_cell>();
        let mut sb_style: *mut style = &raw mut (*wp).scrollbar_style;
        let mut i: u_int = 0;
        let mut j: u_int = 0;
        let mut imin: u_int = 0 as u_int;
        let mut jmin: u_int = 0 as u_int;
        let mut imax: u_int = 0;
        let mut jmax: u_int = 0;
        let mut sb_w: u_int = (*sb_style).width as u_int;
        let mut sb_pad: u_int = (*sb_style).pad as u_int;
        let mut px: ::core::ffi::c_int = 0;
        let mut py: ::core::ffi::c_int = 0;
        let mut wx: ::core::ffi::c_int = 0;
        let mut wy: ::core::ffi::c_int = 0;
        let mut ox: ::core::ffi::c_int = 0;
        let mut oy: ::core::ffi::c_int = 0;
        let mut sx: ::core::ffi::c_int = 0;
        let mut sy: ::core::ffi::c_int = 0;
        let mut sb_tty_y: ::core::ffi::c_int = 0;
        let mut xoff: ::core::ffi::c_int = (*wp).xoff;
        let mut yoff: ::core::ffi::c_int = (*wp).yoff;
        let mut sb_wy: ::core::ffi::c_int = sb_y;
        let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
        sx = ctx.sx as ::core::ffi::c_int;
        sy = (*tty).sy.wrapping_sub(ctx.statuslines) as ::core::ffi::c_int;
        ox = ctx.ox;
        oy = ctx.oy;
        if ctx.statustop != 0 {
            sb_y = (sb_y as u_int).wrapping_add(ctx.statuslines) as ::core::ffi::c_int
                as ::core::ffi::c_int;
            sy = (sy as u_int).wrapping_add(ctx.statuslines) as ::core::ffi::c_int
                as ::core::ffi::c_int;
        }
        gc = (*sb_style).gc;
        slgc = gc;
        slgc.fg = gc.bg;
        slgc.bg = gc.fg;
        if (sb_x + sb_w as ::core::ffi::c_int) < 0 as ::core::ffi::c_int || sb_x >= sx || sb_y >= sy
        {
            return;
        }
        if sb_x < 0 as ::core::ffi::c_int {
            imin = -sb_x as u_int;
        }
        imax = sb_w.wrapping_add(sb_pad);
        if imax as ::core::ffi::c_int + sb_x > sx {
            if sb_x > sx {
                return;
            }
            imax = (sx - sb_x) as u_int;
        }
        jmax = sb_h;
        if jmax as ::core::ffi::c_int + sb_y > sy {
            if sb_y >= sy {
                return;
            }
            jmax = (sy - sb_y) as u_int;
        }
        sb_tty_y = sb_y - oy;
        if sb_tty_y > sy {
            return;
        }
        if sb_tty_y < 0 as ::core::ffi::c_int {
            jmin = -sb_tty_y as u_int;
        }
        if sb_tty_y + sb_h as ::core::ffi::c_int <= 0 as ::core::ffi::c_int {
            return;
        }
        jmax = sb_h;
        if sb_tty_y + jmax as ::core::ffi::c_int > sy {
            jmax = (sy - sb_tty_y) as u_int;
        }
        j = jmin;
        while j < jmax {
            wy = (sb_wy as u_int).wrapping_add(j) as ::core::ffi::c_int;
            py = (sb_tty_y as u_int).wrapping_add(j) as ::core::ffi::c_int;
            r = tty_check_overlay_range(tty, sb_x as u_int, wy as u_int, imax);
            screen_redraw_clip_visible_ranges(wp, sb_x, wy, imax, &mut *r);
            i = imin;
            while i < imax {
                px = ((sb_x + ox) as u_int).wrapping_add(i) as ::core::ffi::c_int;
                wx = (sb_x as u_int).wrapping_add(i) as ::core::ffi::c_int;
                if !(wx < xoff - sb_w as ::core::ffi::c_int - sb_pad as ::core::ffi::c_int
                    || px >= sx
                    || px < 0 as ::core::ffi::c_int
                    || wy < yoff - 1 as ::core::ffi::c_int
                    || py >= sy
                    || py < 0 as ::core::ffi::c_int
                    || screen_redraw_is_visible(r, wx as u_int) == 0)
                {
                    tty_cursor(tty, px as u_int, py as u_int);
                    if sb_pos == PANE_SCROLLBARS_LEFT && i >= sb_w && i < sb_w.wrapping_add(sb_pad)
                        || sb_pos == PANE_SCROLLBARS_RIGHT && i < sb_pad
                    {
                        tty_cell(
                            tty,
                            &raw const grid_default_cell,
                            &raw const grid_default_cell,
                            ::core::ptr::null_mut::<colour_palette>(),
                            ::core::ptr::null_mut::<hyperlinks>(),
                        );
                    } else {
                        if j >= slider_y && j < slider_y.wrapping_add(slider_h) {
                            gcp = &raw mut slgc;
                        } else {
                            gcp = &raw mut gc;
                        }
                        tty_cell(
                            tty,
                            gcp,
                            &raw const grid_default_cell,
                            ::core::ptr::null_mut::<colour_palette>(),
                            ::core::ptr::null_mut::<hyperlinks>(),
                        );
                    }
                }
                i = i.wrapping_add(1);
            }
            j = j.wrapping_add(1);
        }
    }
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
