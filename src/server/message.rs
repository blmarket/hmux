use super::client::{server_client_remove_pane, server_client_set_session};
use super::run::client_walk;
use super::run::marked_pane;
use crate::ffi::{close, getpid, gettimeofday, kill, strlen, utempter_remove_record};
use crate::fmt_args;
use crate::format::format_draw;
use crate::format::format_single;
use crate::grid::grid_default_cell;
use crate::layout::layout_close_pane;
use crate::notify::{notify_pane, notify_session_window};
use crate::options::{options_get_number, options_get_string, options_ptr};
use crate::proc::{peer_ptr, proc_send};
use crate::resize::recalculate_sizes;
use crate::screen::screen_grid_ptr;
use crate::screen::{
    screen_write_cursormove, screen_write_linefeed, screen_write_scrollregion,
    screen_write_start_pane, screen_write_stop,
};
use crate::session::{group_walk, session_activity_time, session_attached, session_options};
use crate::session::{
    session_attach, session_destroy, session_detach, session_group_contains, session_group_count,
    session_has, session_next_session, session_previous_session, session_renumber_windows,
    session_select, sessions_after, sessions_first,
};
use crate::session::{session_get_curw, session_set_curw};
use crate::terminfo::{tty_term_of, tty_term_string};
use crate::tty::{tty_raw, tty_stop_tty};
pub use crate::types::*;
use crate::window::{
    window_count_panes, window_panes_first, window_remove_pane, window_unzoom,
    winlink_find_by_index, winlink_find_by_window, winlink_remove, winlink_stack_remove,
};
use crate::xmalloc::xasprintf;
use ::std::ffi::CString;
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
pub const SORT_END: sort_order = 8;
pub const SORT_Z: sort_order = 7;
pub const SORT_SIZE: sort_order = 6;
pub const SORT_ORDER: sort_order = 5;
pub const SORT_NAME: sort_order = 4;
pub const SORT_MODIFIER: sort_order = 3;
pub const SORT_INDEX: sort_order = 2;
pub const SORT_CREATION: sort_order = 1;
pub const SORT_ACTIVITY: sort_order = 0;
pub const SIGCHLD: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const IMSG_HEADER_SIZE: usize = ::core::mem::size_of::<imsg_hdr>();
pub const MAX_IMSGSIZE: ::core::ffi::c_int = 16384 as ::core::ffi::c_int;
pub const MODE_CURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PANE_REDRAW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PANE_STATUSREADY: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const PANE_STATUSDRAWN: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const WINLINK_BELL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const WINLINK_ACTIVITY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const WINLINK_SILENCE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const WINLINK_ALERTFLAGS: ::core::ffi::c_int =
    WINLINK_BELL | WINLINK_ACTIVITY | WINLINK_SILENCE;
pub const CLIENT_EXIT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CLIENT_REDRAWWINDOW: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CLIENT_REDRAWSTATUS: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CLIENT_SUSPENDED: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CLIENT_REDRAWBORDERS: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWSTATUSALWAYS: ::core::ffi::c_int = 0x1000000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWOVERLAY: ::core::ffi::c_int = 0x2000000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWPANES: ::core::ffi::c_int = 0x20000000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWSCROLLBARS: ::core::ffi::c_ulonglong =
    0x4000000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_NO_DETACH_ON_DESTROY: ::core::ffi::c_ulonglong =
    0x8000000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_ALLREDRAWFLAGS: ::core::ffi::c_ulonglong = (CLIENT_REDRAWWINDOW
    | CLIENT_REDRAWSTATUS
    | CLIENT_REDRAWSTATUSALWAYS
    | CLIENT_REDRAWBORDERS
    | CLIENT_REDRAWOVERLAY
    | CLIENT_REDRAWPANES)
    as ::core::ffi::c_ulonglong
    | CLIENT_REDRAWSCROLLBARS;
pub unsafe fn server_redraw_client(mut c: *mut client) {
    unsafe {
        (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong | CLIENT_ALLREDRAWFLAGS) as uint64_t;
    }
}
pub unsafe fn server_status_client(mut c: *mut client) {
    unsafe {
        (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
    }
}
pub unsafe fn server_redraw_session(mut s: *mut session) {
    unsafe {
        for c in client_walk() {
            if (*c).session == s {
                server_redraw_client(c);
            }
        }
    }
}
pub unsafe fn server_redraw_session_group(mut s: *mut session) {
    unsafe {
        let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
        sg = session_group_contains(s);
        if sg.is_null() {
            server_redraw_session(s);
        } else {
            for s in group_walk(sg) {
                server_redraw_session(s);
            }
        };
    }
}
pub unsafe fn server_status_session(mut s: *mut session) {
    unsafe {
        for c in client_walk() {
            if (*c).session == s {
                server_status_client(c);
            }
        }
    }
}
pub unsafe fn server_status_session_group(mut s: *mut session) {
    unsafe {
        let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
        sg = session_group_contains(s);
        if sg.is_null() {
            server_status_session(s);
        } else {
            for s in group_walk(sg) {
                server_status_session(s);
            }
        };
    }
}
pub unsafe fn server_redraw_window(mut w: *mut window) {
    unsafe {
        for c in client_walk() {
            if !(*c).session.is_null()
                && !session_get_curw((*c).session).is_null()
                && (*session_get_curw((*c).session)).window() == w
            {
                server_redraw_client(c);
            }
        }
    }
}
pub unsafe fn server_redraw_window_borders(mut w: *mut window) {
    unsafe {
        for c in client_walk() {
            if !(*c).session.is_null()
                && !session_get_curw((*c).session).is_null()
                && (*session_get_curw((*c).session)).window() == w
            {
                (*c).flags |= CLIENT_REDRAWBORDERS as uint64_t;
            }
        }
    }
}
pub unsafe fn server_status_window(mut w: *mut window) {
    unsafe {
        let mut s: *mut session = ::core::ptr::null_mut::<session>();
        s = sessions_first();
        while !s.is_null() {
            if session_has(s, w) != 0 {
                server_status_session(s);
            }
            s = sessions_after(s);
        }
    }
}
pub fn server_lock() {
    unsafe {
        for c in client_walk() {
            if !(*c).session.is_null() {
                server_lock_client(c);
            }
        }
    }
}
pub unsafe fn server_lock_session(mut s: *mut session) {
    unsafe {
        for c in client_walk() {
            if (*c).session == s {
                server_lock_client(c);
            }
        }
    }
}
pub unsafe fn server_lock_client(mut c: *mut client) {
    unsafe {
        let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            return;
        }
        if (*c).flags & CLIENT_SUSPENDED as uint64_t != 0 {
            return;
        }
        cmd = options_get_string(session_options((*c).session), c"lock-command".as_ptr());
        if *cmd as ::core::ffi::c_int == '\0' as i32
            || strlen(cmd).wrapping_add(1 as size_t)
                > (MAX_IMSGSIZE as usize).wrapping_sub(IMSG_HEADER_SIZE)
        {
            return;
        }
        tty_stop_tty(&raw mut (*c).tty);
        tty_raw(
            &raw mut (*c).tty,
            tty_term_string(tty_term_of(&(*c).tty), TTYC_SMCUP),
        );
        tty_raw(
            &raw mut (*c).tty,
            tty_term_string(tty_term_of(&(*c).tty), TTYC_CLEAR),
        );
        tty_raw(
            &raw mut (*c).tty,
            tty_term_string(tty_term_of(&(*c).tty), TTYC_E3),
        );
        (*c).flags |= CLIENT_SUSPENDED as uint64_t;
        proc_send(
            peer_ptr(&(*c).peer),
            MSG_LOCK,
            -(1 as ::core::ffi::c_int),
            cmd as *const u8,
            strlen(cmd).wrapping_add(1 as size_t),
        );
    }
}
pub unsafe fn server_kill_pane(mut wp: *mut window_pane) {
    unsafe {
        let mut w: *mut window = (*wp).window;
        if window_count_panes(w, 1 as ::core::ffi::c_int) == 1 as u_int {
            server_kill_window(w, 1 as ::core::ffi::c_int);
            recalculate_sizes();
        } else {
            server_unzoom_window(w);
            server_client_remove_pane(wp);
            layout_close_pane(wp);
            window_remove_pane(w, wp);
            server_redraw_window(w);
        };
    }
}
pub unsafe fn server_kill_window(mut w: *mut window, mut renumber: ::core::ffi::c_int) {
    unsafe {
        let mut s: *mut session = ::core::ptr::null_mut::<session>();
        let mut s1: *mut session = ::core::ptr::null_mut::<session>();
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        s = sessions_first();
        while !s.is_null() && {
            s1 = sessions_after(s);
            1 as ::core::ffi::c_int != 0
        } {
            if !(session_has(s, w) == 0) {
                server_unzoom_window(w);
                loop {
                    wl = winlink_find_by_window(&raw mut (*s).windows, w);
                    if wl.is_null() {
                        break;
                    }
                    if session_detach(s, wl) != 0 {
                        server_destroy_session_group(s);
                        break;
                    } else {
                        server_redraw_session_group(s);
                    }
                }
                if renumber != 0 {
                    server_renumber_session(s);
                }
            }
            s = s1;
        }
        recalculate_sizes();
    }
}
pub unsafe fn server_renumber_session(mut s: *mut session) {
    unsafe {
        let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
        if options_get_number(session_options(s), c"renumber-windows".as_ptr()) != 0 {
            sg = session_group_contains(s);
            if !sg.is_null() {
                for s in group_walk(sg) {
                    session_renumber_windows(s);
                }
            } else {
                session_renumber_windows(s);
            }
        }
    }
}
pub fn server_renumber_all() {
    unsafe {
        let mut s: *mut session = ::core::ptr::null_mut::<session>();
        s = sessions_first();
        while !s.is_null() {
            server_renumber_session(s);
            s = sessions_after(s);
        }
    }
}
pub unsafe fn server_link_window(
    mut src: *mut session,
    mut srcwl: *mut winlink,
    mut dst: *mut session,
    mut dstidx: ::core::ffi::c_int,
    mut killflag: ::core::ffi::c_int,
    mut selectflag: ::core::ffi::c_int,
    cause: &mut Option<CString>,
) -> ::core::ffi::c_int {
    unsafe {
        let mut dstwl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        let mut srcsg: *mut session_group = ::core::ptr::null_mut::<session_group>();
        let mut dstsg: *mut session_group = ::core::ptr::null_mut::<session_group>();
        srcsg = session_group_contains(src);
        dstsg = session_group_contains(dst);
        if src != dst && !srcsg.is_null() && !dstsg.is_null() && srcsg == dstsg {
            *cause = Some(xasprintf(c"sessions are grouped".as_ptr(), fmt_args![]));
            return -(1 as ::core::ffi::c_int);
        }
        dstwl = ::core::ptr::null_mut::<winlink>();
        if dstidx != -(1 as ::core::ffi::c_int) {
            dstwl = winlink_find_by_index(&raw mut (*dst).windows, dstidx);
        }
        if !dstwl.is_null() {
            if (*dstwl).window() == (*srcwl).window() {
                *cause = Some(xasprintf(c"same index: %d".as_ptr(), fmt_args![dstidx]));
                return -(1 as ::core::ffi::c_int);
            }
            if killflag != 0 {
                notify_session_window(c"window-unlinked".as_ptr(), dst, (*dstwl).window());
                (*dstwl).flags &= !WINLINK_ALERTFLAGS;
                winlink_stack_remove(&raw mut (*dst).lastw, dstwl);
                winlink_remove(&raw mut (*dst).windows, dstwl);
                if dstwl == session_get_curw(dst) {
                    selectflag = 1 as ::core::ffi::c_int;
                    session_set_curw(dst, ::core::ptr::null_mut::<winlink>());
                }
            }
        }
        if dstidx == -(1 as ::core::ffi::c_int) {
            dstidx = (-(1 as ::core::ffi::c_int) as ::core::ffi::c_longlong
                - options_get_number(session_options(dst), c"base-index".as_ptr()))
                as ::core::ffi::c_int;
        }
        dstwl = session_attach(dst, (*srcwl).window(), dstidx, cause);
        if dstwl.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        if marked_pane.winlink() == srcwl {
            marked_pane.set_winlink(dstwl);
        }
        if selectflag != 0 {
            session_select(dst, (*dstwl).idx);
        }
        server_redraw_session_group(dst);
        0 as ::core::ffi::c_int
    }
}
pub unsafe fn server_unlink_window(mut s: *mut session, mut wl: *mut winlink) {
    unsafe {
        if session_detach(s, wl) != 0 {
            server_destroy_session_group(s);
        } else {
            server_redraw_session_group(s);
        };
    }
}
pub unsafe fn server_destroy_pane(mut wp: *mut window_pane, mut notify: ::core::ffi::c_int) {
    unsafe {
        let mut w: *mut window = (*wp).window;
        let mut ctx = screen_write_ctx::default();
        let mut gc = grid_default_cell;
        let mut remain_on_exit: ::core::ffi::c_int = 0;
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut sx: u_int = (*screen_grid_ptr(&raw mut (*wp).base)).sx;
        let mut sy: u_int = (*screen_grid_ptr(&raw mut (*wp).base)).sy;
        if (*wp).fd != -(1 as ::core::ffi::c_int) {
            utempter_remove_record((*wp).fd);
            kill(getpid(), SIGCHLD);
            (*wp).event.free();
            (*wp).event = Stream::NONE;
            close((*wp).fd);
            (*wp).fd = -(1 as ::core::ffi::c_int);
        }
        remain_on_exit = options_get_number(options_ptr(&(*wp).options), c"remain-on-exit".as_ptr())
            as ::core::ffi::c_int;
        if remain_on_exit != 0 as ::core::ffi::c_int && !(*wp).flags & PANE_STATUSREADY != 0 {
            return;
        }
        let mut current_block_31: u64;
        match remain_on_exit {
            2 => {
                if (*wp).status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                    && ((*wp).status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                {
                    current_block_31 = 13550086250199790493;
                } else {
                    current_block_31 = 622960851218599991;
                }
            }
            1 | 3 => {
                current_block_31 = 622960851218599991;
            }
            _ => {
                current_block_31 = 13550086250199790493;
            }
        }
        match current_block_31 {
            13550086250199790493 => {}
            _ => {
                if (*wp).flags & PANE_STATUSDRAWN != 0 {
                    return;
                }
                (*wp).flags |= PANE_STATUSDRAWN;
                gettimeofday(&raw mut (*wp).dead_time, ::core::ptr::null_mut());
                if notify != 0 {
                    notify_pane(c"pane-died".as_ptr(), wp);
                }
                s = options_get_string(
                    options_ptr(&(*wp).options),
                    c"remain-on-exit-format".as_ptr(),
                );
                if *s as ::core::ffi::c_int != '\0' as i32 {
                    screen_write_start_pane(&mut ctx, wp, &raw mut (*wp).base);
                    screen_write_scrollregion(&mut ctx, 0 as u_int, sy.wrapping_sub(1 as u_int));
                    screen_write_cursormove(
                        &mut ctx,
                        0 as ::core::ffi::c_int,
                        sy.wrapping_sub(1 as u_int) as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    screen_write_linefeed(&mut ctx, 1 as ::core::ffi::c_int, 8 as u_int);
                    gc = grid_default_cell;
                    let expanded = format_single(
                        ::core::ptr::null_mut::<cmdq_item>(),
                        ::core::ffi::CStr::from_ptr(s),
                        ::core::ptr::null_mut::<client>(),
                        ::core::ptr::null_mut::<session>(),
                        ::core::ptr::null_mut::<winlink>(),
                        wp,
                    );
                    format_draw(
                        &mut ctx,
                        &gc,
                        sx,
                        expanded.as_bytes(),
                        None,
                        0 as ::core::ffi::c_int,
                    );
                    screen_write_stop(&mut ctx);
                }
                (*wp).base.mode &= !MODE_CURSOR;
                (*wp).flags |= PANE_REDRAW;
                return;
            }
        }
        if notify != 0 {
            notify_pane(c"pane-exited".as_ptr(), wp);
        }
        server_unzoom_window(w);
        server_client_remove_pane(wp);
        layout_close_pane(wp);
        window_remove_pane(w, wp);
        if window_panes_first(w).is_null() {
            server_kill_window(w, 1 as ::core::ffi::c_int);
        } else {
            server_redraw_window(w);
        };
    }
}
unsafe fn server_destroy_session_group(mut s: *mut session) {
    unsafe {
        let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
        sg = session_group_contains(s);
        if sg.is_null() {
            server_destroy_session(s);
            session_destroy(
                s,
                1 as ::core::ffi::c_int,
                c"server_destroy_session_group".as_ptr(),
            );
        } else {
            for s in group_walk(sg) {
                server_destroy_session(s);
                session_destroy(
                    s,
                    1 as ::core::ffi::c_int,
                    c"server_destroy_session_group".as_ptr(),
                );
            }
        };
    }
}
unsafe fn server_find_session(
    mut s: *mut session,
    mut f: Option<unsafe fn(*mut session, *mut session) -> ::core::ffi::c_int>,
) -> *mut session {
    unsafe {
        let mut s_loop: *mut session = ::core::ptr::null_mut::<session>();
        let mut s_out: *mut session = ::core::ptr::null_mut::<session>();
        s_loop = sessions_first();
        while !s_loop.is_null() {
            if s_loop != s && f.expect("non-null function pointer")(s_loop, s_out) != 0 {
                s_out = s_loop;
            }
            s_loop = sessions_after(s_loop);
        }
        s_out
    }
}
unsafe fn server_newer_session(
    mut s_loop: *mut session,
    mut s_out: *mut session,
) -> ::core::ffi::c_int {
    unsafe {
        if s_out.is_null() {
            return 1 as ::core::ffi::c_int;
        }
        if session_activity_time(s_loop).tv_sec == session_activity_time(s_out).tv_sec {
            (session_activity_time(s_loop).tv_usec > session_activity_time(s_out).tv_usec)
                as ::core::ffi::c_int
        } else {
            (session_activity_time(s_loop).tv_sec > session_activity_time(s_out).tv_sec)
                as ::core::ffi::c_int
        }
    }
}
unsafe fn server_newer_detached_session(
    mut s_loop: *mut session,
    mut s_out: *mut session,
) -> ::core::ffi::c_int {
    unsafe {
        if session_attached(s_loop) != 0 {
            return 0 as ::core::ffi::c_int;
        }
        server_newer_session(s_loop, s_out)
    }
}
pub unsafe fn server_destroy_session(mut s: *mut session) {
    unsafe {
        let mut s_new: *mut session = ::core::ptr::null_mut::<session>();
        let mut cs_new: *mut session = ::core::ptr::null_mut::<session>();
        let mut use_s: *mut session = ::core::ptr::null_mut::<session>();
        let mut sort_crit: sort_criteria_t = {
            sort_criteria_t {
                order: SORT_NAME,
                reversed: 0 as ::core::ffi::c_int,
                order_seq: None,
            }
        };
        let mut detach_on_destroy: ::core::ffi::c_int = 0;
        detach_on_destroy = options_get_number(session_options(s), c"detach-on-destroy".as_ptr())
            as ::core::ffi::c_int;
        if detach_on_destroy == 0 as ::core::ffi::c_int {
            s_new = server_find_session(s, Some(server_newer_session));
        } else if detach_on_destroy == 2 as ::core::ffi::c_int {
            s_new = server_find_session(s, Some(server_newer_detached_session));
        } else if detach_on_destroy == 3 as ::core::ffi::c_int {
            s_new = session_previous_session(s, &sort_crit);
        } else if detach_on_destroy == 4 as ::core::ffi::c_int {
            s_new = session_next_session(s, &sort_crit);
        }
        if s_new == s {
            s_new = ::core::ptr::null_mut::<session>();
        }
        if s_new.is_null()
            && (detach_on_destroy == 1 as ::core::ffi::c_int
                || detach_on_destroy == 2 as ::core::ffi::c_int)
        {
            cs_new = server_find_session(s, Some(server_newer_session));
        }
        for c in client_walk() {
            if !((*c).session != s) {
                use_s = s_new;
                if use_s.is_null()
                    && (*c).flags as ::core::ffi::c_ulonglong & CLIENT_NO_DETACH_ON_DESTROY != 0
                {
                    use_s = cs_new;
                }
                (*c).session = ::core::ptr::null_mut::<session>();
                (*c).last_session = None;
                server_client_set_session(c, use_s);
                if use_s.is_null() {
                    (*c).flags |= CLIENT_EXIT as uint64_t;
                }
            }
        }
        recalculate_sizes();
    }
}
pub fn server_check_unattached() {
    unsafe {
        let mut s: *mut session = ::core::ptr::null_mut::<session>();
        let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
        let mut current_block_3: u64;
        s = sessions_first();
        while !s.is_null() {
            if !(session_attached(s) != 0 as u_int) {
                match options_get_number(session_options(s), c"destroy-unattached".as_ptr()) {
                    0 => {}
                    2 => {
                        current_block_3 = 10880549826457868132;
                        match current_block_3 {
                            6317823858094538023 => {
                                sg = session_group_contains(s);
                                if !sg.is_null() && session_group_count(sg) == 1 as u_int {
                                    current_block_3 = 16668937799742929182;
                                } else {
                                    current_block_3 = 13109137661213826276;
                                }
                            }
                            10880549826457868132 => {
                                sg = session_group_contains(s);
                                if sg.is_null() || session_group_count(sg) <= 1 as u_int {
                                    current_block_3 = 16668937799742929182;
                                } else {
                                    current_block_3 = 13109137661213826276;
                                }
                            }
                            _ => {}
                        }
                        match current_block_3 {
                            16668937799742929182 => {}
                            _ => {
                                server_destroy_session(s);
                                session_destroy(
                                    s,
                                    1 as ::core::ffi::c_int,
                                    c"server_check_unattached".as_ptr(),
                                );
                            }
                        }
                    }
                    3 => {
                        current_block_3 = 6317823858094538023;
                        match current_block_3 {
                            6317823858094538023 => {
                                sg = session_group_contains(s);
                                if !sg.is_null() && session_group_count(sg) == 1 as u_int {
                                    current_block_3 = 16668937799742929182;
                                } else {
                                    current_block_3 = 13109137661213826276;
                                }
                            }
                            10880549826457868132 => {
                                sg = session_group_contains(s);
                                if sg.is_null() || session_group_count(sg) <= 1 as u_int {
                                    current_block_3 = 16668937799742929182;
                                } else {
                                    current_block_3 = 13109137661213826276;
                                }
                            }
                            _ => {}
                        }
                        match current_block_3 {
                            16668937799742929182 => {}
                            _ => {
                                server_destroy_session(s);
                                session_destroy(
                                    s,
                                    1 as ::core::ffi::c_int,
                                    c"server_check_unattached".as_ptr(),
                                );
                            }
                        }
                    }
                    _ => {
                        current_block_3 = 13109137661213826276;
                        match current_block_3 {
                            6317823858094538023 => {
                                sg = session_group_contains(s);
                                if !sg.is_null() && session_group_count(sg) == 1 as u_int {
                                    current_block_3 = 16668937799742929182;
                                } else {
                                    current_block_3 = 13109137661213826276;
                                }
                            }
                            10880549826457868132 => {
                                sg = session_group_contains(s);
                                if sg.is_null() || session_group_count(sg) <= 1 as u_int {
                                    current_block_3 = 16668937799742929182;
                                } else {
                                    current_block_3 = 13109137661213826276;
                                }
                            }
                            _ => {}
                        }
                        match current_block_3 {
                            16668937799742929182 => {}
                            _ => {
                                server_destroy_session(s);
                                session_destroy(
                                    s,
                                    1 as ::core::ffi::c_int,
                                    c"server_check_unattached".as_ptr(),
                                );
                            }
                        }
                    }
                }
            }
            s = sessions_after(s);
        }
    }
}
pub unsafe fn server_unzoom_window(mut w: *mut window) {
    unsafe {
        if window_unzoom(w, 1 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int {
            server_redraw_window(w);
        }
    }
}
