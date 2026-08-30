use super::draw::tty_draw_line;
use super::keys::{tty_keys_build, tty_keys_free, tty_keys_next};
use crate::ffi::{
    __b64_ntop, __errno_location, abs, fcntl, getpid, ioctl, isatty, open, strcmp, strerror,
    strlen, strncmp, tcflush, tcgetattr, tcsetattr, time, usleep, write,
};
use crate::fmt_args;
use crate::format::{format_create, format_defaults};
use crate::grid::grid_cells_equal;
use crate::grid::grid_default_cell;
use crate::grid::hyperlinks_get;
use crate::log::{fatal, fatalx, log_debug, log_get_level};
use crate::options::{options_get_number, options_get_string, options_ptr};
use crate::reactor::{Interest, IoWatch, Timer, WatchMode};
use crate::screen::screen_mode_to_string;
use crate::server::client_get_pan_window;
use crate::server::client_ref_from_ptr;
use crate::server::client_walk;
use crate::server::server_redraw_client;
use crate::server::{server_client_ensure_ranges, server_client_ranges_is_empty};
use crate::server::{server_client_get_pane, server_client_lost};
use crate::session::session_get_curw;
use crate::status::status_line_size;
use crate::style::style_add;
use crate::style::{
    colour_256to16, colour_find_rgb, colour_force_rgb, colour_palette_get, colour_split_rgb,
};
use crate::terminfo::tty_apply_features;
use crate::terminfo::{tty_acs_get, tty_acs_needed, tty_acs_reverse_get};
use crate::terminfo::{
    tty_term_apply_overrides, tty_term_create, tty_term_flag, tty_term_free, tty_term_has,
    tty_term_number, tty_term_of, tty_term_opt_mut, tty_term_string, tty_term_string_i,
    tty_term_string_ii, tty_term_string_iii, tty_term_string_s, tty_term_string_ss,
};
use crate::text::utf8_set;
use crate::tmux::global_options;
use crate::tmux::setblocking;
pub use crate::types::*;
use crate::window::window_get_active;
use crate::xmalloc::xasprintf;
pub const PROGRESS_BAR_PAUSED: progress_bar_state = 4;
pub const PROGRESS_BAR_INDETERMINATE: progress_bar_state = 3;
pub const PROGRESS_BAR_ERROR: progress_bar_state = 2;
pub const PROGRESS_BAR_NORMAL: progress_bar_state = 1;
pub const PROGRESS_BAR_HIDDEN: progress_bar_state = 0;
pub const SCREEN_CURSOR_BAR: screen_cursor_style = 3;
pub const SCREEN_CURSOR_UNDERLINE: screen_cursor_style = 2;
pub const SCREEN_CURSOR_BLOCK: screen_cursor_style = 1;
pub const SCREEN_CURSOR_DEFAULT: screen_cursor_style = 0;
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
pub const TIOCGWINSZ: ::core::ffi::c_int = 0x5413 as ::core::ffi::c_int;
pub const EAGAIN: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const O_WRONLY: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const O_CREAT: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const O_TRUNC: ::core::ffi::c_int = 0o1000 as ::core::ffi::c_int;
pub const F_SETFD: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const FD_CLOEXEC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const VTIME: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const VMIN: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const IGNBRK: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const ISTRIP: ::core::ffi::c_int = 0o40 as ::core::ffi::c_int;
pub const INLCR: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const IGNCR: ::core::ffi::c_int = 0o200 as ::core::ffi::c_int;
pub const ICRNL: ::core::ffi::c_int = 0o400 as ::core::ffi::c_int;
pub const IXON: ::core::ffi::c_int = 0o2000 as ::core::ffi::c_int;
pub const IXOFF: ::core::ffi::c_int = 0o10000 as ::core::ffi::c_int;
pub const IMAXBEL: ::core::ffi::c_int = 0o20000 as ::core::ffi::c_int;
pub const OPOST: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const ONLCR: ::core::ffi::c_int = 0o4 as ::core::ffi::c_int;
pub const OCRNL: ::core::ffi::c_int = 0o10 as ::core::ffi::c_int;
pub const ONLRET: ::core::ffi::c_int = 0o40 as ::core::ffi::c_int;
pub const ISIG: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const ICANON: ::core::ffi::c_int = 0o2 as ::core::ffi::c_int;
pub const ECHO: ::core::ffi::c_int = 0o10 as ::core::ffi::c_int;
pub const ECHOE: ::core::ffi::c_int = 0o20 as ::core::ffi::c_int;
pub const ECHONL: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const ECHOCTL: ::core::ffi::c_int = 0o1000 as ::core::ffi::c_int;
pub const ECHOPRT: ::core::ffi::c_int = 0o2000 as ::core::ffi::c_int;
pub const ECHOKE: ::core::ffi::c_int = 0o4000 as ::core::ffi::c_int;
pub const IEXTEN: ::core::ffi::c_int = 0o100000 as ::core::ffi::c_int;
pub const TCSANOW: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const TCOFLUSH: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const EV_READ: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const EV_WRITE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const EV_PERSIST: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const MODE_CURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const MODE_MOUSE_STANDARD: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const MODE_MOUSE_BUTTON: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const MODE_CURSOR_BLINKING: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const MODE_MOUSE_ALL: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const MODE_CURSOR_VERY_VISIBLE: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const MODE_CURSOR_BLINKING_SET: ::core::ffi::c_int = 0x20000 as ::core::ffi::c_int;
pub const ALL_MODES: ::core::ffi::c_int = 0xffffff as ::core::ffi::c_int;
pub const ALL_MOUSE_MODES: ::core::ffi::c_int =
    MODE_MOUSE_STANDARD | MODE_MOUSE_BUTTON | MODE_MOUSE_ALL;
pub const CURSOR_MODES: ::core::ffi::c_int =
    MODE_CURSOR | MODE_CURSOR_BLINKING | MODE_CURSOR_VERY_VISIBLE;
pub const UTF8_SIZE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const COLOUR_FLAG_256: ::core::ffi::c_int = 0x1000000 as ::core::ffi::c_int;
pub const COLOUR_FLAG_RGB: ::core::ffi::c_int = 0x2000000 as ::core::ffi::c_int;
pub const GRID_ATTR_BRIGHT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const GRID_ATTR_DIM: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const GRID_ATTR_BLINK: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const GRID_ATTR_REVERSE: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const GRID_ATTR_HIDDEN: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const GRID_ATTR_ITALICS: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const GRID_ATTR_CHARSET: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const GRID_ATTR_STRIKETHROUGH: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE_2: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE_3: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE_4: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE_5: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const GRID_ATTR_OVERLINE: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const GRID_ATTR_ALL_UNDERSCORE: ::core::ffi::c_int = GRID_ATTR_UNDERSCORE
    | GRID_ATTR_UNDERSCORE_2
    | GRID_ATTR_UNDERSCORE_3
    | GRID_ATTR_UNDERSCORE_4
    | GRID_ATTR_UNDERSCORE_5;
pub const GRID_FLAG_PADDING: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const GRID_FLAG_NOPALETTE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const GRID_FLAG_TAB: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const PANE_STYLECHANGED: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const TERM_256COLOURS: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const TERM_NOAM: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const TERM_DECSLRM: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const TERM_DECFRA: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const TERM_RGBCOLOURS: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const TERM_VT100LIKE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const TTY_NOCURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const TTY_FREEZE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const TTY_TIMER: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const TTY_NOBLOCK: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const TTY_STARTED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const TTY_OPENED: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const TTY_OSC52QUERY: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const TTY_BLOCK: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const TTY_HAVEDA: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const TTY_HAVEXDA: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const TTY_SYNCING: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const TTY_HAVEDA2: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const TTY_WINSIZEQUERY: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const TTY_WAITFG: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const TTY_WAITBG: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const TTY_ALL_REQUEST_FLAGS: ::core::ffi::c_int = TTY_HAVEDA | TTY_HAVEDA2 | TTY_HAVEXDA;
pub const TTY_CTX_WRAPPED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const TTY_CTX_INVISIBLE_PANES: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const TTY_CTX_WINDOW_BIGGER: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const TTY_CTX_SYNC: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const TTY_CTX_OVERLAY_SYNC: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const TTY_CTX_CELL_INVALIDATE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const TTY_CTX_PANE_OBSCURED: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CLIENT_TERMINAL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
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
pub const FORMAT_NOJOBS: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const FORMAT_PANE: ::core::ffi::c_uint = 0x80000000 as ::core::ffi::c_uint;
static mut tty_log_fd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const TTY_BLOCK_INTERVAL: ::core::ffi::c_int = 100000 as ::core::ffi::c_int;
pub const TTY_QUERY_TIMEOUT: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const TTY_REQUEST_LIMIT: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
pub fn tty_create_log() {
    unsafe {
        let name = xasprintf(
            c"tmux-out-%ld.log".as_ptr(),
            fmt_args![getpid() as ::core::ffi::c_long],
        );
        tty_log_fd = open(
            name.as_ptr(),
            O_WRONLY | O_CREAT | O_TRUNC,
            0o644 as ::core::ffi::c_int,
        );
        if tty_log_fd != -(1 as ::core::ffi::c_int)
            && fcntl(tty_log_fd, F_SETFD, FD_CLOEXEC) == -(1 as ::core::ffi::c_int)
        {
            fatal(c"fcntl failed".as_ptr(), fmt_args![]);
        }
    }
}
/// The client whose terminal `tty` is, or null once that client has gone.
pub unsafe fn tty_client(tty: *mut tty) -> *mut client {
    unsafe {
        (*tty)
            .owner
            .as_ref()
            .and_then(ClientWeak::upgrade)
            .map_or(::core::ptr::null_mut(), |c| c.as_ptr())
    }
}

pub unsafe fn tty_init(mut tty: *mut tty, mut c: *mut client) -> ::core::ffi::c_int {
    unsafe {
        if isatty((*c).fd) == 0 {
            return -(1 as ::core::ffi::c_int);
        }
        *tty = tty::default();
        (*tty).owner = client_ref_from_ptr(c).map(|c| c.downgrade());
        (*tty).cstyle = SCREEN_CURSOR_DEFAULT;
        (*tty).ccolour = -(1 as ::core::ffi::c_int);
        (*tty).bg = -(1 as ::core::ffi::c_int);
        (*tty).fg = (*tty).bg;
        (*tty).mouse_last_pane = -(1 as ::core::ffi::c_int);
        if tcgetattr((*c).fd, &raw mut (*tty).tio) != 0 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int);
        }
        0 as ::core::ffi::c_int
    }
}
pub unsafe fn tty_resize(mut tty: *mut tty) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        let mut ws = winsize::default();
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        let mut xpixel: u_int = 0;
        let mut ypixel: u_int = 0;
        if ioctl((*c).fd, TIOCGWINSZ as ::core::ffi::c_ulong, &raw mut ws)
            != -(1 as ::core::ffi::c_int)
        {
            sx = ws.ws_col as u_int;
            if sx == 0 as u_int {
                sx = 80 as u_int;
                xpixel = 0 as u_int;
            } else {
                xpixel = (ws.ws_xpixel as u_int).wrapping_div(sx);
            }
            sy = ws.ws_row as u_int;
            if sy == 0 as u_int {
                sy = 24 as u_int;
                ypixel = 0 as u_int;
            } else {
                ypixel = (ws.ws_ypixel as u_int).wrapping_div(sy);
            }
            if (xpixel == 0 as u_int || ypixel == 0 as u_int)
                && (*tty).out.is_some()
                && (*tty).flags & TTY_WINSIZEQUERY == 0
                && tty_term_of(&*tty).flags & TERM_VT100LIKE != 0
            {
                tty_puts(tty, c"\x1B[18t\x1B[14t".as_ptr());
                (*tty).flags |= TTY_WINSIZEQUERY;
            }
        } else {
            sx = 80 as u_int;
            sy = 24 as u_int;
            xpixel = 0 as u_int;
            ypixel = 0 as u_int;
        }
        log_debug(
            c"%s: %s now %ux%u (%ux%u)".as_ptr(),
            fmt_args![
                c"tty_resize".as_ptr(),
                cstr_ptr(&(*c).name),
                sx,
                sy,
                xpixel,
                ypixel
            ],
        );
        tty_set_size(tty, sx, sy, xpixel, ypixel);
        tty_invalidate(tty);
    }
}
pub unsafe fn tty_set_size(
    mut tty: *mut tty,
    mut sx: u_int,
    mut sy: u_int,
    mut xpixel: u_int,
    mut ypixel: u_int,
) {
    unsafe {
        (*tty).sx = sx;
        (*tty).sy = sy;
        (*tty).xpixel = xpixel;
        (*tty).ypixel = ypixel;
    }
}
unsafe fn tty_read_callback(tty: *mut tty) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        let mut name: *const ::core::ffi::c_char = cstr_ptr(&(*c).name);
        let size = (*tty).in_0.as_ref().unwrap().len();
        let nread = match (*tty)
            .in_0
            .as_mut()
            .unwrap()
            .read_from_fd((*c).fd, 64 * 1024)
        {
            Ok(nread) => nread as ::core::ffi::c_int,
            Err(_) => -(1 as ::core::ffi::c_int),
        };
        if nread == 0 as ::core::ffi::c_int || nread == -(1 as ::core::ffi::c_int) {
            if nread == 0 as ::core::ffi::c_int {
                log_debug(c"%s: read closed".as_ptr(), fmt_args![name]);
            } else {
                log_debug(
                    c"%s: read error: %s".as_ptr(),
                    fmt_args![name, strerror(*__errno_location())],
                );
            }
            (*tty).event_in.disable();
            server_client_lost(tty_client(tty));
            return;
        }
        log_debug(
            c"%s: read %d bytes (already %zu)".as_ptr(),
            fmt_args![name, nread, size],
        );
        while tty_keys_next(tty) != 0 {}
    }
}
unsafe fn tty_timer_callback(tty: *mut tty) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        let mut tv = timeval::from_usecs(TTY_BLOCK_INTERVAL as __suseconds_t);
        log_debug(
            c"%s: %zu discarded".as_ptr(),
            fmt_args![cstr_ptr(&(*c).name), (*tty).discarded],
        );
        (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong | CLIENT_ALLREDRAWFLAGS) as uint64_t;
        (*c).discarded = (*c).discarded.wrapping_add((*tty).discarded);
        if (*tty).discarded
            < (1 as u_int).wrapping_add((*tty).sx.wrapping_mul((*tty).sy).wrapping_div(8 as u_int))
                as size_t
        {
            (*tty).flags &= !TTY_BLOCK;
            tty_invalidate(tty);
            return;
        }
        (*tty).discarded = 0 as size_t;
        (*tty).timer.arm(tv);
    }
}
unsafe fn tty_block_maybe(mut tty: *mut tty) -> ::core::ffi::c_int {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        let size = (*tty).out.as_ref().unwrap().len();
        let mut tv = timeval::from_usecs(TTY_BLOCK_INTERVAL as __suseconds_t);
        if size == 0 as size_t {
            (*tty).flags &= !TTY_NOBLOCK;
        } else if (*tty).flags & TTY_NOBLOCK != 0 {
            return 0 as ::core::ffi::c_int;
        }
        if size
            < (1 as u_int).wrapping_add((*tty).sx.wrapping_mul((*tty).sy).wrapping_mul(8 as u_int))
                as size_t
        {
            return 0 as ::core::ffi::c_int;
        }
        if (*tty).flags & TTY_BLOCK != 0 {
            return 1 as ::core::ffi::c_int;
        }
        (*tty).flags |= TTY_BLOCK;
        log_debug(
            c"%s: can't keep up, %zu discarded".as_ptr(),
            fmt_args![cstr_ptr(&(*c).name), size],
        );
        (*tty).out.as_mut().unwrap().drain(size);
        (*c).discarded = (*c).discarded.wrapping_add(size);
        (*tty).discarded = 0 as size_t;
        (*tty).timer.arm(tv);
        1 as ::core::ffi::c_int
    }
}
unsafe fn tty_write_callback(tty: *mut tty) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        let size = (*tty).out.as_ref().unwrap().len();
        let nwrite = match (*tty).out.as_mut().unwrap().write_to_fd((*c).fd) {
            Ok(nwrite) => nwrite as ::core::ffi::c_int,
            Err(_) => -(1 as ::core::ffi::c_int),
        };
        if nwrite < 0 {
            return;
        }
        log_debug(
            c"%s: wrote %d bytes (of %zu)".as_ptr(),
            fmt_args![cstr_ptr(&(*c).name), nwrite, size],
        );
        if (*c).redraw > 0 as size_t {
            if nwrite as size_t >= (*c).redraw {
                (*c).redraw = 0 as size_t;
            } else {
                (*c).redraw = (*c).redraw.wrapping_sub(nwrite as size_t);
            }
            log_debug(
                c"%s: waiting for redraw, %zu bytes left".as_ptr(),
                fmt_args![cstr_ptr(&(*c).name), (*c).redraw],
            );
        } else if tty_block_maybe(tty) != 0 {
            return;
        }
        if (*tty).out.as_ref().unwrap().len() != 0 as size_t {
            (*tty).event_out.enable();
        }
    }
}
pub unsafe fn tty_open(
    mut tty: *mut tty,
    cause: &mut Option<::std::ffi::CString>,
) -> ::core::ffi::c_int {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        match tty_term_create(
            tty,
            cstr_ptr(&(*c).term_name),
            &(*c).term_caps,
            &mut (*c).term_features,
        ) {
            Ok(term) => (*tty).term = Some(term),
            Err(err) => {
                *cause = Some(err);
                tty_close(tty);
                return -(1 as ::core::ffi::c_int);
            }
        }
        (*tty).flags |= TTY_OPENED;
        (*tty).flags &= !(TTY_NOCURSOR | TTY_FREEZE | TTY_BLOCK | TTY_TIMER);
        (*tty).event_in.set_callback(
            (*c).fd,
            Interest::Read,
            WatchMode::Persistent,
            move |_, _| tty_read_callback(tty),
        );
        (*tty).in_0 = Some(Box::new(Buf::new()));
        (*tty)
            .event_out
            .set_callback((*c).fd, Interest::Write, WatchMode::Once, move |_, _| {
                tty_write_callback(tty)
            });
        (*tty).out = Some(Box::new(Buf::new()));
        (*tty)
            .clipboard_timer
            .set_callback(move || (*tty).flags &= !TTY_OSC52QUERY);
        (*tty)
            .start_timer
            .set_callback(move || tty_start_timer_callback(tty));
        (*tty).timer.set_callback(move || tty_timer_callback(tty));
        tty_start_tty(tty);
        tty_keys_build(tty);
        0 as ::core::ffi::c_int
    }
}
unsafe fn tty_start_timer_callback(tty: *mut tty) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        log_debug(
            c"%s: start timer fired".as_ptr(),
            fmt_args![cstr_ptr(&(*c).name)],
        );
        if (*tty).flags & (TTY_HAVEDA | TTY_HAVEDA2 | TTY_HAVEXDA) == 0 as ::core::ffi::c_int {
            tty_update_features(tty);
        }
        (*tty).flags |= TTY_ALL_REQUEST_FLAGS;
        (*tty).flags &= !(TTY_WAITBG | TTY_WAITFG);
    }
}
unsafe fn tty_start_start_timer(mut tty: *mut tty) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        let mut tv = timeval::from_secs(TTY_QUERY_TIMEOUT as __time_t);
        log_debug(
            c"%s: start timer started".as_ptr(),
            fmt_args![cstr_ptr(&(*c).name)],
        );
        (*tty).start_timer.disarm();
        (*tty).start_timer.arm(tv);
    }
}
pub unsafe fn tty_start_tty(mut tty: *mut tty) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        let mut tio: termios = ::core::mem::zeroed();
        setblocking((*c).fd, 0 as ::core::ffi::c_int);
        (*tty).event_in.enable();
        tio = (*tty).tio;
        tio.c_iflag &= !(IXON | IXOFF | ICRNL | INLCR | IGNCR | IMAXBEL | ISTRIP) as tcflag_t;
        tio.c_iflag |= IGNBRK as tcflag_t;
        tio.c_oflag &= !(OPOST | ONLCR | OCRNL | ONLRET) as tcflag_t;
        tio.c_lflag &=
            !(IEXTEN | ICANON | ECHO | ECHOE | ECHONL | ECHOCTL | ECHOPRT | ECHOKE | ISIG)
                as tcflag_t;
        tio.c_cc[VMIN as usize] = 1 as cc_t;
        tio.c_cc[VTIME as usize] = 0 as cc_t;
        if tcsetattr((*c).fd, TCSANOW, &raw mut tio) == 0 as ::core::ffi::c_int {
            tcflush((*c).fd, TCOFLUSH);
        }
        tty_putcode(tty, TTYC_SMCUP);
        tty_putcode(tty, TTYC_SMKX);
        tty_putcode(tty, TTYC_CLEAR);
        if tty_acs_needed(tty) != 0 {
            log_debug(
                c"%s: using capabilities for ACS".as_ptr(),
                fmt_args![cstr_ptr(&(*c).name)],
            );
            tty_putcode(tty, TTYC_ENACS);
        } else {
            log_debug(
                c"%s: using UTF-8 for ACS".as_ptr(),
                fmt_args![cstr_ptr(&(*c).name)],
            );
        }
        tty_putcode(tty, TTYC_CNORM);
        if tty_term_has(tty_term_of(&*tty), TTYC_KMOUS) != 0 {
            tty_puts(tty, c"\x1B[?1000l\x1B[?1002l\x1B[?1003l".as_ptr());
            tty_puts(tty, c"\x1B[?1006l\x1B[?1005l".as_ptr());
        }
        if tty_term_has(tty_term_of(&*tty), TTYC_ENBP) != 0 {
            tty_putcode(tty, TTYC_ENBP);
        }
        if tty_term_of(&*tty).flags & TERM_VT100LIKE != 0 {
            tty_puts(tty, c"\x1B[?2031h\x1B[?996n".as_ptr());
        }
        tty_start_start_timer(tty);
        (*tty).flags |= TTY_STARTED;
        tty_invalidate(tty);
        if (*tty).ccolour != -(1 as ::core::ffi::c_int) {
            tty_force_cursor_colour(tty, -(1 as ::core::ffi::c_int));
        }
        (*tty).mouse_drag_flag = 0 as ::core::ffi::c_int;
        (*tty).mouse_drag_update = None;
        (*tty).mouse_drag_release = None;
    }
}
pub unsafe fn tty_send_requests(mut tty: *mut tty) {
    unsafe {
        if !(*tty).flags & TTY_STARTED != 0 {
            return;
        }
        if tty_term_of(&*tty).flags & TERM_VT100LIKE != 0 {
            if !(*tty).flags & TTY_HAVEDA != 0 {
                tty_puts(tty, c"\x1B[c".as_ptr());
            }
            if !(*tty).flags & TTY_HAVEDA2 != 0 {
                tty_puts(tty, c"\x1B[>c".as_ptr());
            }
            if !(*tty).flags & TTY_HAVEXDA != 0 {
                tty_puts(tty, c"\x1B[>q".as_ptr());
            }
            tty_puts(tty, c"\x1B]10;?\x1B\\\x1B]11;?\x1B\\".as_ptr());
            (*tty).flags |= TTY_WAITBG | TTY_WAITFG;
        } else {
            (*tty).flags |= TTY_ALL_REQUEST_FLAGS;
        }
        (*tty).last_requests = time(::core::ptr::null_mut::<time_t>());
    }
}
pub unsafe fn tty_repeat_requests(mut tty: *mut tty, mut force: ::core::ffi::c_int) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        let mut t: time_t = time(::core::ptr::null_mut::<time_t>());
        let mut n: u_int = (t - (*tty).last_requests) as u_int;
        if !(*tty).flags & TTY_STARTED != 0 {
            return;
        }
        if force == 0 && n <= TTY_REQUEST_LIMIT as u_int {
            log_debug(
                c"%s: not repeating requests (%u seconds)".as_ptr(),
                fmt_args![cstr_ptr(&(*c).name), n],
            );
            return;
        }
        log_debug(
            c"%s: %srepeating requests (%u seconds)".as_ptr(),
            fmt_args![
                cstr_ptr(&(*c).name),
                if force != 0 {
                    c"(force) ".as_ptr()
                } else {
                    c"".as_ptr()
                },
                n
            ],
        );
        (*tty).last_requests = t;
        if tty_term_of(&*tty).flags & TERM_VT100LIKE != 0 {
            tty_puts(tty, c"\x1B]10;?\x1B\\\x1B]11;?\x1B\\".as_ptr());
            (*tty).flags |= TTY_WAITBG | TTY_WAITFG;
        }
        tty_start_start_timer(tty);
    }
}
pub unsafe fn tty_stop_tty(mut tty: *mut tty) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        let mut ws = winsize::default();
        if (*tty).flags & TTY_STARTED == 0 {
            return;
        }
        (*tty).flags &= !TTY_STARTED;
        (*tty).start_timer.disarm();
        (*tty).clipboard_timer.disarm();
        (*tty).timer.disarm();
        (*tty).flags &= !TTY_BLOCK;
        (*tty).event_in.disable();
        (*tty).event_out.disable();
        if ioctl((*c).fd, TIOCGWINSZ as ::core::ffi::c_ulong, &raw mut ws)
            == -(1 as ::core::ffi::c_int)
        {
            return;
        }
        if tcsetattr((*c).fd, TCSANOW, &raw mut (*tty).tio) == -(1 as ::core::ffi::c_int) {
            return;
        }
        tty_raw(
            tty,
            tty_term_string_ii(
                tty_term_of(&*tty),
                TTYC_CSR,
                0 as ::core::ffi::c_int,
                ws.ws_row as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            )
            .as_ptr(),
        );
        if tty_acs_needed(tty) != 0 {
            tty_raw(tty, tty_term_string(tty_term_of(&*tty), TTYC_RMACS));
        }
        tty_raw(tty, tty_term_string(tty_term_of(&*tty), TTYC_SGR0));
        tty_raw(tty, tty_term_string(tty_term_of(&*tty), TTYC_RMKX));
        tty_raw(tty, tty_term_string(tty_term_of(&*tty), TTYC_CLEAR));
        if (*tty).cstyle as ::core::ffi::c_uint
            != SCREEN_CURSOR_DEFAULT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if tty_term_has(tty_term_of(&*tty), TTYC_SE) != 0 {
                tty_raw(tty, tty_term_string(tty_term_of(&*tty), TTYC_SE));
            } else if tty_term_has(tty_term_of(&*tty), TTYC_SS) != 0 {
                tty_raw(
                    tty,
                    tty_term_string_i(tty_term_of(&*tty), TTYC_SS, 0 as ::core::ffi::c_int)
                        .as_ptr(),
                );
            }
        }
        if (*tty).ccolour != -(1 as ::core::ffi::c_int) {
            tty_raw(tty, tty_term_string(tty_term_of(&*tty), TTYC_CR));
        }
        tty_raw(tty, tty_term_string(tty_term_of(&*tty), TTYC_CNORM));
        if tty_term_has(tty_term_of(&*tty), TTYC_KMOUS) != 0 {
            tty_raw(tty, c"\x1B[?1000l\x1B[?1002l\x1B[?1003l".as_ptr());
            tty_raw(tty, c"\x1B[?1006l\x1B[?1005l".as_ptr());
        }
        if tty_term_has(tty_term_of(&*tty), TTYC_DSBP) != 0 {
            tty_raw(tty, tty_term_string(tty_term_of(&*tty), TTYC_DSBP));
        }
        if tty_term_of(&*tty).flags & TERM_VT100LIKE != 0 {
            tty_raw(tty, c"\x1B[?7727l".as_ptr());
        }
        tty_raw(tty, tty_term_string(tty_term_of(&*tty), TTYC_DSFCS));
        tty_raw(tty, tty_term_string(tty_term_of(&*tty), TTYC_DSEKS));
        if tty_term_of(&*tty).flags & TERM_DECSLRM != 0 {
            tty_raw(tty, tty_term_string(tty_term_of(&*tty), TTYC_DSMG));
        }
        tty_raw(tty, tty_term_string(tty_term_of(&*tty), TTYC_RMCUP));
        if tty_term_of(&*tty).flags & TERM_VT100LIKE != 0 {
            tty_raw(tty, c"\x1B[?2031l".as_ptr());
        }
        setblocking((*c).fd, 1 as ::core::ffi::c_int);
    }
}
pub unsafe fn tty_close(mut tty: *mut tty) {
    unsafe {
        (*tty).key_timer.disarm();
        tty_stop_tty(tty);
        if (*tty).flags & TTY_OPENED != 0 {
            (*tty).in_0 = None;
            (*tty).event_in.disable();
            (*tty).out = None;
            (*tty).event_out.disable();
            if let Some(term) = (*tty).term.take() {
                tty_term_free(term);
            }
            tty_keys_free(tty);
            (*tty).flags &= !TTY_OPENED;
        }
    }
}
pub unsafe fn tty_free(mut tty: *mut tty) {
    unsafe {
        tty_close(tty);
        (*tty).r.ranges = Vec::new();
    }
}
pub unsafe fn tty_update_features(mut tty: *mut tty) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        if tty_apply_features(
            tty_term_opt_mut(&mut (*tty).term).expect("a tty being driven has a terminal"),
            (*c).term_features,
        ) != 0
        {
            tty_term_apply_overrides(
                tty_term_opt_mut(&mut (*tty).term).expect("a tty being driven has a terminal"),
            );
        }
        if tty_term_of(&*tty).flags & TERM_DECSLRM != 0 {
            tty_putcode(tty, TTYC_ENMG);
        }
        if options_get_number(global_options, c"extended-keys".as_ptr()) != 0 {
            tty_puts(tty, tty_term_string(tty_term_of(&*tty), TTYC_ENEKS));
        }
        if options_get_number(global_options, c"focus-events".as_ptr()) != 0 {
            tty_puts(tty, tty_term_string(tty_term_of(&*tty), TTYC_ENFCS));
        }
        if tty_term_of(&*tty).flags & TERM_VT100LIKE != 0 {
            tty_puts(tty, c"\x1B[?7727h".as_ptr());
        }
        server_redraw_client(c);
        tty_invalidate(tty);
    }
}
pub unsafe fn tty_raw(mut tty: *mut tty, mut s: *const ::core::ffi::c_char) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        let mut n: ssize_t = 0;
        let mut slen: ssize_t = 0;
        let mut i: u_int = 0;
        slen = strlen(s) as ssize_t;
        i = 0 as u_int;
        while i < 5 as u_int {
            n = write((*c).fd, s as *const ::core::ffi::c_void, slen as size_t);
            if n >= 0 as ssize_t {
                s = s.offset(n as isize);
                slen -= n;
                if slen == 0 as ssize_t {
                    break;
                }
            } else if n == -(1 as ::core::ffi::c_int) as ssize_t && *__errno_location() != EAGAIN {
                break;
            }
            usleep(100 as __useconds_t);
            i = i.wrapping_add(1);
        }
    }
}
pub unsafe fn tty_putcode(mut tty: *mut tty, mut code: tty_code_code) {
    unsafe {
        tty_puts(tty, tty_term_string(tty_term_of(&*tty), code));
    }
}
pub unsafe fn tty_putcode_i(mut tty: *mut tty, mut code: tty_code_code, mut a: ::core::ffi::c_int) {
    unsafe {
        if a < 0 as ::core::ffi::c_int {
            return;
        }
        tty_puts(tty, tty_term_string_i(tty_term_of(&*tty), code, a).as_ptr());
    }
}
pub unsafe fn tty_putcode_ii(
    mut tty: *mut tty,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
) {
    unsafe {
        if a < 0 as ::core::ffi::c_int || b < 0 as ::core::ffi::c_int {
            return;
        }
        tty_puts(
            tty,
            tty_term_string_ii(tty_term_of(&*tty), code, a, b).as_ptr(),
        );
    }
}
pub unsafe fn tty_putcode_iii(
    mut tty: *mut tty,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) {
    unsafe {
        if a < 0 as ::core::ffi::c_int || b < 0 as ::core::ffi::c_int || c < 0 as ::core::ffi::c_int
        {
            return;
        }
        tty_puts(
            tty,
            tty_term_string_iii(tty_term_of(&*tty), code, a, b, c).as_ptr(),
        );
    }
}
pub unsafe fn tty_putcode_s(
    mut tty: *mut tty,
    mut code: tty_code_code,
    mut a: *const ::core::ffi::c_char,
) {
    unsafe {
        if !a.is_null() {
            tty_puts(tty, tty_term_string_s(tty_term_of(&*tty), code, a).as_ptr());
        }
    }
}
pub unsafe fn tty_putcode_ss(
    mut tty: *mut tty,
    mut code: tty_code_code,
    mut a: *const ::core::ffi::c_char,
    mut b: *const ::core::ffi::c_char,
) {
    unsafe {
        if !a.is_null() && !b.is_null() {
            tty_puts(
                tty,
                tty_term_string_ss(tty_term_of(&*tty), code, a, b).as_ptr(),
            );
        }
    }
}
unsafe fn tty_add(mut tty: *mut tty, mut buf: *const ::core::ffi::c_char, mut len: size_t) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        if (*tty).flags & TTY_BLOCK != 0 {
            (*tty).discarded = (*tty).discarded.wrapping_add(len);
            return;
        }
        (*tty)
            .out
            .as_mut()
            .unwrap()
            .append(::core::slice::from_raw_parts(buf.cast::<u8>(), len));
        log_debug(
            c"%s: %.*s".as_ptr(),
            fmt_args![cstr_ptr(&(*c).name), len as ::core::ffi::c_int, buf],
        );
        (*c).written = (*c).written.wrapping_add(len);
        if tty_log_fd != -(1 as ::core::ffi::c_int) {
            write(tty_log_fd, buf as *const ::core::ffi::c_void, len);
        }
        if (*tty).flags & TTY_STARTED != 0 {
            (*tty).event_out.enable();
        }
    }
}
pub unsafe fn tty_puts(mut tty: *mut tty, mut s: *const ::core::ffi::c_char) {
    unsafe {
        if *s as ::core::ffi::c_int != '\0' as i32 {
            tty_add(tty, s, strlen(s));
        }
    }
}
pub unsafe fn tty_putc(mut tty: *mut tty, mut ch: u_char) {
    unsafe {
        let mut acs: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if tty_term_of(&*tty).flags & TERM_NOAM != 0
            && ch as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int
            && ch as ::core::ffi::c_int != 0x7f as ::core::ffi::c_int
            && (*tty).cy == (*tty).sy.wrapping_sub(1 as u_int)
            && (*tty).cx.wrapping_add(1 as u_int) >= (*tty).sx
        {
            return;
        }
        if (*tty).cell.attr as ::core::ffi::c_int & GRID_ATTR_CHARSET != 0 {
            acs = tty_acs_get(tty, ch);
            if !acs.is_null() {
                tty_add(tty, acs, strlen(acs));
            } else {
                tty_add(tty, &raw mut ch as *const ::core::ffi::c_char, 1 as size_t);
            }
        } else {
            tty_add(tty, &raw mut ch as *const ::core::ffi::c_char, 1 as size_t);
        }
        if ch as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int
            && ch as ::core::ffi::c_int != 0x7f as ::core::ffi::c_int
        {
            if (*tty).cx >= (*tty).sx {
                (*tty).cx = 1 as u_int;
                if (*tty).cy != (*tty).rlower {
                    (*tty).cy = (*tty).cy.wrapping_add(1);
                }
                if tty_term_of(&*tty).flags & TERM_NOAM != 0 {
                    tty_putcode_ii(
                        tty,
                        TTYC_CUP,
                        (*tty).cy as ::core::ffi::c_int,
                        (*tty).cx as ::core::ffi::c_int,
                    );
                }
            } else {
                (*tty).cx = (*tty).cx.wrapping_add(1);
            }
        }
    }
}
pub unsafe fn tty_putn(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut width: u_int,
) {
    unsafe {
        if tty_term_of(&*tty).flags & TERM_NOAM != 0
            && (*tty).cy == (*tty).sy.wrapping_sub(1 as u_int)
            && ((*tty).cx as size_t).wrapping_add(len) >= (*tty).sx as size_t
        {
            len = (*tty).sx.wrapping_sub((*tty).cx).wrapping_sub(1 as u_int) as size_t;
        }
        tty_add(tty, buf, len);
        if (*tty).cx.wrapping_add(width) > (*tty).sx {
            (*tty).cx = (*tty).cx.wrapping_add(width).wrapping_sub((*tty).sx);
            if (*tty).cx <= (*tty).sx {
                (*tty).cy = (*tty).cy.wrapping_add(1);
            } else {
                (*tty).cy = UINT_MAX as u_int;
                (*tty).cx = (*tty).cy;
            }
        } else {
            (*tty).cx = (*tty).cx.wrapping_add(width);
        };
    }
}
unsafe fn tty_set_italics(mut tty: *mut tty) {
    unsafe {
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if tty_term_has(tty_term_of(&*tty), TTYC_SITM) != 0 {
            s = options_get_string(global_options, c"default-terminal".as_ptr());
            if strcmp(s, c"screen".as_ptr()) != 0 as ::core::ffi::c_int
                && strncmp(s, c"screen-".as_ptr(), 7 as size_t) != 0 as ::core::ffi::c_int
            {
                tty_putcode(tty, TTYC_SITM);
                return;
            }
        }
        tty_putcode(tty, TTYC_SMSO);
    }
}
pub unsafe fn tty_set_title(mut tty: *mut tty, mut title: *const ::core::ffi::c_char) {
    unsafe {
        if tty_term_has(tty_term_of(&*tty), TTYC_TSL) == 0
            || tty_term_has(tty_term_of(&*tty), TTYC_FSL) == 0
        {
            return;
        }
        tty_putcode(tty, TTYC_TSL);
        tty_puts(tty, title);
        tty_putcode(tty, TTYC_FSL);
    }
}
pub unsafe fn tty_set_path(mut tty: *mut tty, mut title: *const ::core::ffi::c_char) {
    unsafe {
        if tty_term_has(tty_term_of(&*tty), TTYC_SWD) == 0
            || tty_term_has(tty_term_of(&*tty), TTYC_FSL) == 0
        {
            return;
        }
        tty_putcode(tty, TTYC_SWD);
        tty_puts(tty, title);
        tty_putcode(tty, TTYC_FSL);
    }
}
unsafe fn tty_force_cursor_colour(mut tty: *mut tty, mut c: ::core::ffi::c_int) {
    unsafe {
        let mut r: u_char = 0;
        let mut g: u_char = 0;
        let mut b: u_char = 0;
        if c != -(1 as ::core::ffi::c_int) {
            c = colour_force_rgb(c);
        }
        if c == (*tty).ccolour {
            return;
        }
        if c == -(1 as ::core::ffi::c_int) {
            tty_putcode(tty, TTYC_CR);
        } else {
            colour_split_rgb(c, &raw mut r, &raw mut g, &raw mut b);
            let s = xasprintf(
                c"rgb:%02hhx/%02hhx/%02hhx".as_ptr(),
                fmt_args![
                    r as ::core::ffi::c_int,
                    g as ::core::ffi::c_int,
                    b as ::core::ffi::c_int
                ],
            );
            tty_putcode_s(tty, TTYC_CS, s.as_ptr());
        }
        (*tty).ccolour = c;
    }
}
unsafe fn tty_update_cursor(
    mut tty: *mut tty,
    mut mode: ::core::ffi::c_int,
    mut s: *mut screen,
) -> ::core::ffi::c_int {
    unsafe {
        let mut cstyle: screen_cursor_style = SCREEN_CURSOR_DEFAULT;
        let mut ccolour: ::core::ffi::c_int = 0;
        let mut changed: ::core::ffi::c_int = 0;
        let mut cmode: ::core::ffi::c_int = mode;
        if !s.is_null() {
            ccolour = (*s).ccolour;
            if (*s).ccolour == -(1 as ::core::ffi::c_int) {
                ccolour = (*s).default_ccolour;
            }
            tty_force_cursor_colour(tty, ccolour);
        }
        if !cmode & MODE_CURSOR != 0 {
            if (*tty).mode & MODE_CURSOR != 0 {
                tty_putcode(tty, TTYC_CIVIS);
            }
            return cmode;
        }
        if s.is_null() {
            cstyle = (*tty).cstyle;
        } else {
            cstyle = (*s).cstyle;
            if cstyle as ::core::ffi::c_uint
                == SCREEN_CURSOR_DEFAULT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if !cmode & MODE_CURSOR_BLINKING_SET != 0 {
                    if (*s).default_mode & MODE_CURSOR_BLINKING != 0 {
                        cmode |= MODE_CURSOR_BLINKING;
                    } else {
                        cmode &= !MODE_CURSOR_BLINKING;
                    }
                }
                cstyle = (*s).default_cstyle;
            }
        }
        changed = cmode ^ (*tty).mode;
        if changed & CURSOR_MODES == 0 as ::core::ffi::c_int
            && cstyle as ::core::ffi::c_uint == (*tty).cstyle as ::core::ffi::c_uint
        {
            return cmode;
        }
        tty_putcode(tty, TTYC_CNORM);
        match cstyle {
            SCREEN_CURSOR_DEFAULT => {
                if (*tty).cstyle as ::core::ffi::c_uint
                    != SCREEN_CURSOR_DEFAULT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if tty_term_has(tty_term_of(&*tty), TTYC_SE) != 0 {
                        tty_putcode(tty, TTYC_SE);
                    } else {
                        tty_putcode_i(tty, TTYC_SS, 0 as ::core::ffi::c_int);
                    }
                }
                if cmode & (MODE_CURSOR_BLINKING | MODE_CURSOR_VERY_VISIBLE) != 0 {
                    tty_putcode(tty, TTYC_CVVIS);
                }
            }
            SCREEN_CURSOR_BLOCK => {
                if tty_term_has(tty_term_of(&*tty), TTYC_SS) != 0 {
                    if cmode & MODE_CURSOR_BLINKING != 0 {
                        tty_putcode_i(tty, TTYC_SS, 1 as ::core::ffi::c_int);
                    } else {
                        tty_putcode_i(tty, TTYC_SS, 2 as ::core::ffi::c_int);
                    }
                } else if cmode & MODE_CURSOR_BLINKING != 0 {
                    tty_putcode(tty, TTYC_CVVIS);
                }
            }
            SCREEN_CURSOR_UNDERLINE => {
                if tty_term_has(tty_term_of(&*tty), TTYC_SS) != 0 {
                    if cmode & MODE_CURSOR_BLINKING != 0 {
                        tty_putcode_i(tty, TTYC_SS, 3 as ::core::ffi::c_int);
                    } else {
                        tty_putcode_i(tty, TTYC_SS, 4 as ::core::ffi::c_int);
                    }
                } else if cmode & MODE_CURSOR_BLINKING != 0 {
                    tty_putcode(tty, TTYC_CVVIS);
                }
            }
            SCREEN_CURSOR_BAR => {
                if tty_term_has(tty_term_of(&*tty), TTYC_SS) != 0 {
                    if cmode & MODE_CURSOR_BLINKING != 0 {
                        tty_putcode_i(tty, TTYC_SS, 5 as ::core::ffi::c_int);
                    } else {
                        tty_putcode_i(tty, TTYC_SS, 6 as ::core::ffi::c_int);
                    }
                } else if cmode & MODE_CURSOR_BLINKING != 0 {
                    tty_putcode(tty, TTYC_CVVIS);
                }
            }
            _ => {}
        }
        (*tty).cstyle = cstyle;
        cmode
    }
}
pub unsafe fn tty_update_mode(mut tty: *mut tty, mut mode: ::core::ffi::c_int, mut s: *mut screen) {
    unsafe {
        let term: &tty_term = tty_term_of(&*tty);
        let mut c: *mut client = tty_client(tty);
        let mut changed: ::core::ffi::c_int = 0;
        if (*tty).flags & TTY_NOCURSOR != 0 {
            mode &= !MODE_CURSOR;
        }
        if tty_update_cursor(tty, mode, s) & MODE_CURSOR_BLINKING != 0 {
            mode |= MODE_CURSOR_BLINKING;
        } else {
            mode &= !MODE_CURSOR_BLINKING;
        }
        changed = mode ^ (*tty).mode;
        if log_get_level() != 0 as ::core::ffi::c_int && changed != 0 as ::core::ffi::c_int {
            log_debug(
                c"%s: current mode %s".as_ptr(),
                fmt_args![
                    cstr_ptr(&(*c).name),
                    screen_mode_to_string((*tty).mode).as_c_str()
                ],
            );
            log_debug(
                c"%s: setting mode %s".as_ptr(),
                fmt_args![cstr_ptr(&(*c).name), screen_mode_to_string(mode).as_c_str()],
            );
        }
        if changed & ALL_MOUSE_MODES != 0 && tty_term_has(term, TTYC_KMOUS) != 0 {
            tty_puts(
                tty,
                c"\x1B[?1006l\x1B[?1000l\x1B[?1002l\x1B[?1003l".as_ptr(),
            );
            if mode & ALL_MOUSE_MODES != 0 {
                tty_puts(tty, c"\x1B[?1006h".as_ptr());
            }
            if mode & MODE_MOUSE_ALL != 0 {
                tty_puts(tty, c"\x1B[?1000h\x1B[?1002h\x1B[?1003h".as_ptr());
            } else if mode & MODE_MOUSE_BUTTON != 0 {
                tty_puts(tty, c"\x1B[?1000h\x1B[?1002h".as_ptr());
            } else if mode & MODE_MOUSE_STANDARD != 0 {
                tty_puts(tty, c"\x1B[?1000h".as_ptr());
            }
        }
        (*tty).mode = mode;
    }
}
unsafe fn tty_emulate_repeat(
    mut tty: *mut tty,
    mut code: tty_code_code,
    mut code1: tty_code_code,
    mut n: u_int,
) {
    unsafe {
        if tty_term_has(tty_term_of(&*tty), code) != 0 {
            tty_putcode_i(tty, code, n as ::core::ffi::c_int);
        } else {
            loop {
                let fresh0 = n;
                n = n.wrapping_sub(1);
                if !(fresh0 > 0 as u_int) {
                    break;
                }
                tty_putcode(tty, code1);
            }
        };
    }
}
pub unsafe fn tty_repeat_space(mut tty: *mut tty, mut n: u_int) {
    unsafe {
        static mut s: [::core::ffi::c_char; 500] = [0; 500];
        if *(&raw mut s as *mut ::core::ffi::c_char) as ::core::ffi::c_int != ' ' as i32 {
            s = [' ' as ::core::ffi::c_char; 500];
        }
        while n as usize > ::core::mem::size_of::<[::core::ffi::c_char; 500]>() as usize {
            tty_putn(
                tty,
                &raw mut s as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 500]>() as size_t,
                ::core::mem::size_of::<[::core::ffi::c_char; 500]>() as u_int,
            );
            n = (n as ::core::ffi::c_ulong).wrapping_sub(::core::mem::size_of::<
                [::core::ffi::c_char; 500],
            >() as usize
                as ::core::ffi::c_ulong) as u_int as u_int;
        }
        if n != 0 as u_int {
            tty_putn(
                tty,
                &raw mut s as *const ::core::ffi::c_char,
                n as size_t,
                n,
            );
        }
    }
}
pub unsafe fn tty_window_bigger(mut tty: *mut tty) -> ::core::ffi::c_int {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        let mut w: *mut window = (*session_get_curw((*c).session)).window();
        ((*tty).sx < (*w).sx || (*tty).sy.wrapping_sub(status_line_size(c)) < (*w).sy)
            as ::core::ffi::c_int
    }
}
/// Whether the window is bigger than the terminal, and the part of it the
/// terminal shows: offset and size.
pub unsafe fn tty_window_offset(
    mut tty: *mut tty,
) -> (::core::ffi::c_int, u_int, u_int, u_int, u_int) {
    unsafe { ((*tty).oflag, (*tty).oox, (*tty).ooy, (*tty).osx, (*tty).osy) }
}
unsafe fn tty_window_offset1(
    mut tty: *mut tty,
    ox: &mut u_int,
    oy: &mut u_int,
    sx: &mut u_int,
    sy: &mut u_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        let mut w: *mut window = (*session_get_curw((*c).session)).window();
        let mut wp: *mut window_pane = server_client_get_pane(c);
        let mut cx: u_int = 0;
        let mut cy: u_int = 0;
        let mut lines: u_int = 0;
        lines = status_line_size(c);
        if (*tty).sx >= (*w).sx && (*tty).sy.wrapping_sub(lines) >= (*w).sy {
            *ox = 0 as u_int;
            *oy = 0 as u_int;
            *sx = (*w).sx;
            *sy = (*w).sy;
            (*c).pan_window = None;
            return 0 as ::core::ffi::c_int;
        }
        *sx = (*tty).sx;
        *sy = (*tty).sy.wrapping_sub(lines);
        if client_get_pan_window(c) == w {
            if *sx >= (*w).sx {
                (*c).pan_ox = 0 as u_int;
            } else if (*c).pan_ox.wrapping_add(*sx) > (*w).sx {
                (*c).pan_ox = (*w).sx.wrapping_sub(*sx);
            }
            *ox = (*c).pan_ox;
            if *sy >= (*w).sy {
                (*c).pan_oy = 0 as u_int;
            } else if (*c).pan_oy.wrapping_add(*sy) > (*w).sy {
                (*c).pan_oy = (*w).sy.wrapping_sub(*sy);
            }
            *oy = (*c).pan_oy;
            return 1 as ::core::ffi::c_int;
        }
        if !(*(*wp).screen()).mode & MODE_CURSOR != 0 {
            *ox = 0 as u_int;
            *oy = 0 as u_int;
        } else {
            cx = ((*wp).xoff as u_int).wrapping_add((*(*wp).screen()).cx);
            cy = ((*wp).yoff as u_int).wrapping_add((*(*wp).screen()).cy);
            if cx < *sx {
                *ox = 0 as u_int;
            } else if cx > (*w).sx.wrapping_sub(*sx) {
                *ox = (*w).sx.wrapping_sub(*sx);
            } else {
                *ox = cx.wrapping_sub((*sx).wrapping_div(2 as u_int));
            }
            if cy < *sy {
                *oy = 0 as u_int;
            } else if cy > (*w).sy.wrapping_sub(*sy) {
                *oy = (*w).sy.wrapping_sub(*sy);
            } else {
                *oy = cy.wrapping_sub(*sy).wrapping_add(1 as u_int);
            }
        }
        (*c).pan_window = None;
        1 as ::core::ffi::c_int
    }
}
pub unsafe fn tty_update_window_offset(mut w: *mut window) {
    unsafe {
        for c in client_walk() {
            if !(*c).session.is_null()
                && !session_get_curw((*c).session).is_null()
                && (*session_get_curw((*c).session)).window() == w
            {
                tty_update_client_offset(c);
            }
        }
    }
}
pub unsafe fn tty_update_client_offset(mut c: *mut client) {
    unsafe {
        let mut ox: u_int = 0;
        let mut oy: u_int = 0;
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        if !(*c).flags & CLIENT_TERMINAL as uint64_t != 0 {
            return;
        }
        (*c).tty.oflag = tty_window_offset1(&raw mut (*c).tty, &mut ox, &mut oy, &mut sx, &mut sy);
        if ox == (*c).tty.oox && oy == (*c).tty.ooy && sx == (*c).tty.osx && sy == (*c).tty.osy {
            return;
        }
        log_debug(
            c"%s: %s offset has changed (%u,%u %ux%u -> %u,%u %ux%u)".as_ptr(),
            fmt_args![
                c"tty_update_client_offset".as_ptr(),
                cstr_ptr(&(*c).name),
                (*c).tty.oox,
                (*c).tty.ooy,
                (*c).tty.osx,
                (*c).tty.osy,
                ox,
                oy,
                sx,
                sy
            ],
        );
        (*c).tty.oox = ox;
        (*c).tty.ooy = oy;
        (*c).tty.osx = sx;
        (*c).tty.osy = sy;
        (*c).flags |= (CLIENT_REDRAWWINDOW | CLIENT_REDRAWSTATUS) as uint64_t;
    }
}
unsafe fn tty_large_region(_tty: *mut tty, ctx: &tty_ctx) -> ::core::ffi::c_int {
    (ctx.orlower.wrapping_sub(ctx.orupper) >= ctx.sy.wrapping_div(2 as u_int)) as ::core::ffi::c_int
}
pub unsafe fn tty_fake_bce(
    mut tty: *const tty,
    mut gc: *const grid_cell,
    mut bg: u_int,
) -> ::core::ffi::c_int {
    unsafe {
        if tty_term_flag(tty_term_of(&*tty), TTYC_BCE) != 0 {
            return 0 as ::core::ffi::c_int;
        }
        if !(bg == 8 as u_int || bg == 9 as u_int)
            || !((*gc).bg == 8 as ::core::ffi::c_int || (*gc).bg == 9 as ::core::ffi::c_int)
        {
            return 1 as ::core::ffi::c_int;
        }
        0 as ::core::ffi::c_int
    }
}
unsafe fn tty_redraw_region(mut tty: *mut tty, ctx: &tty_ctx) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        let mut i: u_int = 0;
        if tty_large_region(tty, ctx) != 0 || ctx.flags & TTY_CTX_PANE_OBSCURED != 0 {
            log_debug(
                c"%s: %s large region redraw".as_ptr(),
                fmt_args![c"tty_redraw_region".as_ptr(), cstr_ptr(&(*c).name)],
            );
            ctx.redraw_cb.expect("non-null function pointer")(ctx);
            return;
        }
        log_debug(
            c"%s: %s small region redraw (%u-%u)".as_ptr(),
            fmt_args![
                c"tty_redraw_region".as_ptr(),
                cstr_ptr(&(*c).name),
                ctx.orupper,
                ctx.orlower
            ],
        );
        i = ctx.orupper;
        while i <= ctx.orlower {
            tty_draw_pane(tty, ctx, i);
            i = i.wrapping_add(1);
        }
    }
}
unsafe fn tty_is_visible(
    _tty: *mut tty,
    ctx: &tty_ctx,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut ny: u_int,
) -> ::core::ffi::c_int {
    let mut xoff: u_int = (ctx.rxoff as u_int).wrapping_add(px);
    let mut yoff: u_int = (ctx.ryoff as u_int).wrapping_add(py);
    if !ctx.flags & TTY_CTX_WINDOW_BIGGER != 0 {
        return 1 as ::core::ffi::c_int;
    }
    if xoff.wrapping_add(nx) <= ctx.wox
        || xoff >= ctx.wox.wrapping_add(ctx.wsx)
        || yoff.wrapping_add(ny) <= ctx.woy
        || yoff >= ctx.woy.wrapping_add(ctx.wsy)
    {
        return 0 as ::core::ffi::c_int;
    }
    1 as ::core::ffi::c_int
}
unsafe fn tty_clamp_line(
    mut tty: *mut tty,
    ctx: &tty_ctx,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
) -> Option<(u_int, u_int, u_int, u_int)> {
    unsafe {
        let i: u_int;
        let x: u_int;
        let rx: u_int;

        let mut xoff: ::core::ffi::c_int =
            (ctx.rxoff as u_int).wrapping_add(px) as ::core::ffi::c_int;
        if tty_is_visible(tty, ctx, px, py, nx, 1 as u_int) == 0 {
            return None;
        }
        let ry: u_int = (ctx.yoff as u_int).wrapping_add(py).wrapping_sub(ctx.woy);
        if xoff >= ctx.wox as ::core::ffi::c_int
            && (xoff as u_int).wrapping_add(nx) <= ctx.wox.wrapping_add(ctx.wsx)
        {
            i = 0 as u_int;
            x = (ctx.xoff as u_int).wrapping_add(px).wrapping_sub(ctx.wox);
            rx = nx;
        } else if xoff < ctx.wox as ::core::ffi::c_int
            && (xoff as u_int).wrapping_add(nx) > ctx.wox.wrapping_add(ctx.wsx)
        {
            i = ctx.wox;
            x = 0 as u_int;
            rx = ctx.wsx;
        } else if xoff < ctx.wox as ::core::ffi::c_int {
            i = ctx.wox.wrapping_sub((ctx.xoff as u_int).wrapping_add(px));
            x = 0 as u_int;
            rx = nx.wrapping_sub(i);
        } else {
            i = 0 as u_int;
            x = (ctx.xoff as u_int).wrapping_add(px).wrapping_sub(ctx.wox);
            rx = ctx.wsx.wrapping_sub(x);
        }
        if rx > nx {
            fatalx(
                c"%s: x too big, %u > %u".as_ptr(),
                fmt_args![c"tty_clamp_line".as_ptr(), rx, nx],
            );
        }
        Some((i, x, rx, ry))
    }
}
unsafe fn tty_clear_line(
    mut tty: *mut tty,
    mut defaults: *const grid_cell,
    mut py: u_int,
    mut px: u_int,
    mut nx: u_int,
    mut bg: u_int,
) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
        let mut rr: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
        let mut i: u_int = 0;
        log_debug(
            c"%s: %s, %u at %u,%u".as_ptr(),
            fmt_args![c"tty_clear_line".as_ptr(), cstr_ptr(&(*c).name), nx, px, py],
        );
        if nx == 0 as u_int {
            return;
        }
        if (*c).overlay_check().is_none() && tty_fake_bce(tty, defaults, bg) == 0 {
            if px.wrapping_add(nx) >= (*tty).sx && tty_term_has(tty_term_of(&*tty), TTYC_EL) != 0 {
                tty_cursor(tty, px, py);
                tty_putcode(tty, TTYC_EL);
                return;
            }
            if px == 0 as u_int && tty_term_has(tty_term_of(&*tty), TTYC_EL1) != 0 {
                tty_cursor(tty, px.wrapping_add(nx).wrapping_sub(1 as u_int), py);
                tty_putcode(tty, TTYC_EL1);
                return;
            }
            if tty_term_has(tty_term_of(&*tty), TTYC_ECH) != 0 {
                tty_cursor(tty, px, py);
                tty_putcode_i(tty, TTYC_ECH, nx as ::core::ffi::c_int);
                return;
            }
        }
        r = tty_check_overlay_range(tty, px, py, nx);
        i = 0 as u_int;
        while i < (*r).used {
            rr = (*r).ranges.as_mut_ptr().offset(i as isize);
            if (*rr).nx != 0 as u_int {
                tty_cursor(tty, (*rr).px, py);
                tty_repeat_space(tty, (*rr).nx);
            }
            i = i.wrapping_add(1);
        }
    }
}
unsafe fn tty_clear_pane_line(
    mut tty: *mut tty,
    ctx: &tty_ctx,
    mut py: u_int,
    mut px: u_int,
    mut nx: u_int,
    mut bg: u_int,
) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
        let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
        let mut i: u_int = 0;
        let mut x: u_int = 0;
        let mut rx: u_int = 0;
        let mut ry: u_int = 0;
        log_debug(
            c"%s: %s, %u at %u,%u".as_ptr(),
            fmt_args![
                c"tty_clear_pane_line".as_ptr(),
                cstr_ptr(&(*c).name),
                nx,
                px,
                py
            ],
        );
        if let Some((_clamped_l, clamped_x, clamped_rx, clamped_ry)) =
            tty_clamp_line(tty, ctx, px, py, nx)
        {
            (x, rx, ry) = (clamped_x, clamped_rx, clamped_ry);
            r = tty_check_overlay_range(tty, x, ry, rx);
            i = 0 as u_int;
            while i < (*r).used {
                ri = (*r).ranges.as_mut_ptr().offset(i as isize);
                if !((*ri).nx == 0 as u_int) {
                    tty_clear_line(tty, &raw const ctx.defaults, ry, (*ri).px, (*ri).nx, bg);
                }
                i = i.wrapping_add(1);
            }
        }
    }
}
unsafe fn tty_clamp_area(
    mut tty: *mut tty,
    ctx: &tty_ctx,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut ny: u_int,
) -> Option<(u_int, u_int, u_int, u_int, u_int, u_int)> {
    unsafe {
        let i: u_int;
        let j: u_int;
        let x: u_int;
        let y: u_int;
        let rx: u_int;
        let ry: u_int;
        let mut xoff: u_int = (ctx.rxoff as u_int).wrapping_add(px);
        let mut yoff: u_int = (ctx.ryoff as u_int).wrapping_add(py);
        if tty_is_visible(tty, ctx, px, py, nx, ny) == 0 {
            return None;
        }
        if xoff >= ctx.wox && xoff.wrapping_add(nx) <= ctx.wox.wrapping_add(ctx.wsx) {
            i = 0 as u_int;
            x = (ctx.xoff as u_int).wrapping_add(px).wrapping_sub(ctx.wox);
            rx = nx;
        } else if xoff < ctx.wox && xoff.wrapping_add(nx) > ctx.wox.wrapping_add(ctx.wsx) {
            i = ctx.wox;
            x = 0 as u_int;
            rx = ctx.wsx;
        } else if xoff < ctx.wox {
            i = ctx.wox.wrapping_sub((ctx.xoff as u_int).wrapping_add(px));
            x = 0 as u_int;
            rx = nx.wrapping_sub(i);
        } else {
            i = 0 as u_int;
            x = (ctx.xoff as u_int).wrapping_add(px).wrapping_sub(ctx.wox);
            rx = ctx.wsx.wrapping_sub(x);
        }
        if rx > nx {
            fatalx(
                c"%s: x too big, %u > %u".as_ptr(),
                fmt_args![c"tty_clamp_area".as_ptr(), rx, nx],
            );
        }
        if yoff >= ctx.woy && yoff.wrapping_add(ny) <= ctx.woy.wrapping_add(ctx.wsy) {
            j = 0 as u_int;
            y = (ctx.yoff as u_int).wrapping_add(py).wrapping_sub(ctx.woy);
            ry = ny;
        } else if yoff < ctx.woy && yoff.wrapping_add(ny) > ctx.woy.wrapping_add(ctx.wsy) {
            j = ctx.woy;
            y = 0 as u_int;
            ry = ctx.wsy;
        } else if yoff < ctx.woy {
            j = ctx.woy.wrapping_sub((ctx.yoff as u_int).wrapping_add(py));
            y = 0 as u_int;
            ry = ny.wrapping_sub(j);
        } else {
            j = 0 as u_int;
            y = (ctx.yoff as u_int).wrapping_add(py).wrapping_sub(ctx.woy);
            ry = ctx.wsy.wrapping_sub(y);
        }
        if ry > ny {
            fatalx(
                c"%s: y too big, %u > %u".as_ptr(),
                fmt_args![c"tty_clamp_area".as_ptr(), ry, ny],
            );
        }
        Some((i, j, x, y, rx, ry))
    }
}
unsafe fn tty_clear_area(
    mut tty: *mut tty,
    ctx: &tty_ctx,
    mut py: u_int,
    mut ny: u_int,
    mut px: u_int,
    mut nx: u_int,
    mut bg: u_int,
) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        let mut defaults: *const grid_cell = &raw const ctx.defaults;
        let mut yy: u_int = 0;
        log_debug(
            c"%s: %s, %u,%u at %u,%u".as_ptr(),
            fmt_args![
                c"tty_clear_area".as_ptr(),
                cstr_ptr(&(*c).name),
                nx,
                ny,
                px,
                py
            ],
        );
        if nx == 0 as u_int || ny == 0 as u_int {
            return;
        }
        if (*c).overlay_check().is_none() && tty_fake_bce(tty, defaults, bg) == 0 {
            if px == 0 as u_int
                && px.wrapping_add(nx) >= (*tty).sx
                && py.wrapping_add(ny) >= (*tty).sy
                && tty_term_has(tty_term_of(&*tty), TTYC_ED) != 0
            {
                tty_cursor(tty, 0 as u_int, py);
                tty_putcode(tty, TTYC_ED);
                return;
            }
            if tty_term_of(&*tty).flags & TERM_DECFRA != 0
                && !(bg == 8 as u_int || bg == 9 as u_int)
            {
                let tmp = xasprintf(
                    c"\x1B[32;%u;%u;%u;%u$x".as_ptr(),
                    fmt_args![
                        py.wrapping_add(1 as u_int),
                        px.wrapping_add(1 as u_int),
                        py.wrapping_add(ny),
                        px.wrapping_add(nx)
                    ],
                );
                tty_puts(tty, tmp.as_ptr());
                return;
            }
            if px == 0 as u_int
                && px.wrapping_add(nx) >= (*tty).sx
                && ny > 2 as u_int
                && tty_term_has(tty_term_of(&*tty), TTYC_CSR) != 0
                && tty_term_has(tty_term_of(&*tty), TTYC_INDN) != 0
            {
                tty_region(tty, py, py.wrapping_add(ny).wrapping_sub(1 as u_int));
                tty_margin_off(tty);
                tty_putcode_i(tty, TTYC_INDN, ny as ::core::ffi::c_int);
                return;
            }
            if nx > 2 as u_int
                && ny > 2 as u_int
                && tty_term_has(tty_term_of(&*tty), TTYC_CSR) != 0
                && tty_term_of(&*tty).flags & TERM_DECSLRM != 0
                && tty_term_has(tty_term_of(&*tty), TTYC_INDN) != 0
            {
                tty_region(tty, py, py.wrapping_add(ny).wrapping_sub(1 as u_int));
                tty_margin(tty, px, px.wrapping_add(nx).wrapping_sub(1 as u_int));
                tty_putcode_i(tty, TTYC_INDN, ny as ::core::ffi::c_int);
                return;
            }
        }
        yy = py;
        while yy < py.wrapping_add(ny) {
            tty_clear_line(tty, defaults, yy, px, nx, bg);
            yy = yy.wrapping_add(1);
        }
    }
}
unsafe fn tty_clear_pane_area(
    mut tty: *mut tty,
    ctx: &tty_ctx,
    mut py: u_int,
    mut ny: u_int,
    mut px: u_int,
    mut nx: u_int,
    mut bg: u_int,
) {
    unsafe {
        let mut x: u_int = 0;
        let mut y: u_int = 0;
        let mut rx: u_int = 0;
        let mut ry: u_int = 0;
        if let Some((_clamped_i, _clamped_j, clamped_x, clamped_y, clamped_rx, clamped_ry)) =
            tty_clamp_area(tty, ctx, px, py, nx, ny)
        {
            (x, y, rx, ry) = (clamped_x, clamped_y, clamped_rx, clamped_ry);
            tty_clear_area(tty, ctx, y, ry, x, rx, bg);
        }
    }
}
unsafe fn tty_draw_pane(mut tty: *mut tty, ctx: &tty_ctx, mut py: u_int) {
    unsafe {
        let mut s: *mut screen = ctx.s;
        let mut nx: u_int = ctx.sx;
        let mut i: u_int = 0;
        let mut x: u_int = 0;
        let mut rx: u_int = 0;
        let mut ry: u_int = 0;
        let mut j: u_int = 0;
        let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
        let mut rr: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
        log_debug(
            c"%s: %s %u".as_ptr(),
            fmt_args![
                c"tty_draw_pane".as_ptr(),
                cstr_ptr(&(*tty_client(tty)).name),
                py
            ],
        );
        if !ctx.flags & TTY_CTX_WINDOW_BIGGER != 0 {
            r = tty_check_overlay_range(
                tty,
                ctx.xoff as u_int,
                (ctx.yoff as u_int).wrapping_add(py),
                nx,
            );
            j = 0 as u_int;
            while j < (*r).used {
                rr = (*r).ranges.as_mut_ptr().offset(j as isize);
                if !((*rr).nx == 0 as u_int) {
                    tty_draw_line(
                        tty,
                        s,
                        (*rr).px.wrapping_sub(ctx.xoff as u_int),
                        py,
                        (*rr).nx,
                        (*rr).px,
                        (ctx.yoff as u_int).wrapping_add(py),
                        &raw const ctx.defaults,
                        ctx.palette,
                    );
                }
                j = j.wrapping_add(1);
            }
            return;
        }
        if let Some((clamped_i, clamped_x, clamped_rx, clamped_ry)) =
            tty_clamp_line(tty, ctx, 0 as u_int, py, nx)
        {
            (i, x, rx, ry) = (clamped_i, clamped_x, clamped_rx, clamped_ry);
            r = tty_check_overlay_range(tty, x, ry, rx);
            j = 0 as u_int;
            while j < (*r).used {
                rr = (*r).ranges.as_mut_ptr().offset(j as isize);
                if !((*rr).nx == 0 as u_int) {
                    tty_draw_line(
                        tty,
                        s,
                        i.wrapping_add((*rr).px).wrapping_sub(x),
                        py,
                        (*rr).nx,
                        (*rr).px,
                        ry,
                        &raw const ctx.defaults,
                        ctx.palette,
                    );
                }
                j = j.wrapping_add(1);
            }
        }
    }
}
pub unsafe fn tty_cmd_redrawline(mut tty: *mut tty, ctx: &tty_ctx) {
    unsafe {
        let mut i: u_int = 0;
        let mut x: u_int = 0;
        let mut rx: u_int = 0;
        let mut ry: u_int = 0;
        let mut j: u_int = 0;
        let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
        let mut rr: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
        if let Some((clamped_i, clamped_x, clamped_rx, clamped_ry)) =
            tty_clamp_line(tty, ctx, ctx.ocx, ctx.ocy, tty_ctx_num(ctx))
        {
            (i, x, rx, ry) = (clamped_i, clamped_x, clamped_rx, clamped_ry);
            r = tty_check_overlay_range(tty, x, ry, rx);
            j = 0 as u_int;
            while j < (*r).used {
                rr = (*r).ranges.as_mut_ptr().offset(j as isize);
                if !((*rr).nx == 0 as u_int) {
                    tty_draw_line(
                        tty,
                        ctx.s,
                        ctx.ocx
                            .wrapping_add(i)
                            .wrapping_add((*rr).px)
                            .wrapping_sub(x),
                        ctx.ocy,
                        (*rr).nx,
                        (*rr).px,
                        ry,
                        &raw const ctx.defaults,
                        ctx.palette,
                    );
                }
                j = j.wrapping_add(1);
            }
        }
    }
}
pub unsafe fn tty_check_codeset(mut tty: *mut tty, mut gc: *const grid_cell) -> grid_cell {
    unsafe {
        let mut new: grid_cell;
        let mut c: ::core::ffi::c_int = 0;
        if (*gc).data.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            && (*(&raw const (*gc).data.data as *const u_char) as ::core::ffi::c_int)
                < 0x7f as ::core::ffi::c_int
        {
            return *gc;
        }
        if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
            return *gc;
        }
        if (*tty_client(tty)).flags & CLIENT_UTF8 as uint64_t != 0 {
            return *gc;
        }
        new = *gc;
        c = tty_acs_reverse_get(
            tty,
            &raw const (*gc).data.data as *const u_char as *const ::core::ffi::c_char,
            (*gc).data.size as size_t,
        );
        if c != -(1 as ::core::ffi::c_int) {
            utf8_set(&mut new.data, c as u_char);
            new.attr = (new.attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
            return new;
        }
        new.data.size = (*gc).data.width;
        if new.data.size as ::core::ffi::c_int > UTF8_SIZE {
            new.data.size = UTF8_SIZE as u_char;
        }
        ::core::ptr::write_bytes(
            &raw mut new.data.data as *mut u_char,
            b'_',
            new.data.size as usize,
        );
        new
    }
}
unsafe fn tty_check_overlay(mut tty: *mut tty, mut px: u_int, mut py: u_int) -> ::core::ffi::c_int {
    unsafe {
        let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
        r = tty_check_overlay_range(tty, px, py, 1 as u_int);
        (server_client_ranges_is_empty(r) == 0) as ::core::ffi::c_int
    }
}
pub unsafe fn tty_check_overlay_range(
    mut tty: *mut tty,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
) -> *mut visible_ranges {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        if (*c).overlay_check().is_none() {
            server_client_ensure_ranges(&raw mut (*tty).r, 1 as u_int);
            (*(*tty)
                .r
                .ranges
                .as_mut_ptr()
                .offset(0 as ::core::ffi::c_int as isize))
            .px = px;
            (*(*tty)
                .r
                .ranges
                .as_mut_ptr()
                .offset(0 as ::core::ffi::c_int as isize))
            .nx = nx;
            (*tty).r.used = 1 as u_int;
            return &raw mut (*tty).r;
        }
        (*c).overlay_check()
            .call(c, (*c).current_overlay_data(), px, py, nx)
    }
}
pub unsafe fn tty_sync_start(mut tty: *mut tty) {
    unsafe {
        if (*tty).flags & TTY_BLOCK != 0 {
            return;
        }
        if (*tty).flags & TTY_SYNCING != 0 {
            return;
        }
        (*tty).flags |= TTY_SYNCING;
        if tty_term_has(tty_term_of(&*tty), TTYC_SYNC) != 0 {
            log_debug(
                c"%s sync start".as_ptr(),
                fmt_args![cstr_ptr(&(*tty_client(tty)).name)],
            );
            tty_putcode_i(tty, TTYC_SYNC, 1 as ::core::ffi::c_int);
        }
    }
}
pub unsafe fn tty_sync_end(mut tty: *mut tty) {
    unsafe {
        if (*tty).flags & TTY_BLOCK != 0 {
            return;
        }
        if !(*tty).flags & TTY_SYNCING != 0 {
            return;
        }
        (*tty).flags &= !TTY_SYNCING;
        if tty_term_has(tty_term_of(&*tty), TTYC_SYNC) != 0 {
            log_debug(
                c"%s sync end".as_ptr(),
                fmt_args![cstr_ptr(&(*tty_client(tty)).name)],
            );
            tty_putcode_i(tty, TTYC_SYNC, 2 as ::core::ffi::c_int);
        }
    }
}
unsafe fn tty_client_ready(ctx: &tty_ctx, mut c: *mut client) -> ::core::ffi::c_int {
    unsafe {
        if (*c).session.is_null() || (*c).tty.term.is_none() {
            return 0 as ::core::ffi::c_int;
        }
        if (*c).flags & CLIENT_SUSPENDED as uint64_t != 0 {
            return 0 as ::core::ffi::c_int;
        }
        if ctx.flags & TTY_CTX_INVISIBLE_PANES != 0 {
            return 1 as ::core::ffi::c_int;
        }
        if (*c).flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
            return 0 as ::core::ffi::c_int;
        }
        if (*c).tty.flags & TTY_FREEZE != 0 {
            return 0 as ::core::ffi::c_int;
        }
        1 as ::core::ffi::c_int
    }
}
pub unsafe fn tty_write(mut cmdfn: Option<unsafe fn(*mut tty, &tty_ctx) -> ()>, ctx: &mut tty_ctx) {
    unsafe {
        let mut state: ::core::ffi::c_int = 0;
        if ctx.set_client_cb.is_none() {
            return;
        }
        for c in client_walk() {
            if tty_client_ready(&*ctx, c) != 0 {
                state = ctx.set_client_cb.expect("non-null function pointer")(ctx, c);
                if state == -(1 as ::core::ffi::c_int) {
                    break;
                }
                if !(state == 0 as ::core::ffi::c_int) {
                    cmdfn.expect("non-null function pointer")(&raw mut (*c).tty, &*ctx);
                }
            }
        }
    }
}
/// The count a terminal command carries. Only asked of the commands whose
/// context was built with one.
unsafe fn tty_ctx_num(ctx: &tty_ctx) -> u_int {
    unsafe {
        match ctx.value {
            TtyCtxValue::Num(n) => n,
            _ => fatalx(c"terminal command has no count".as_ptr(), fmt_args![]),
        }
    }
}

/// The bytes a terminal command carries.
unsafe fn tty_ctx_bytes(ctx: &tty_ctx) -> tty_ctx_data {
    unsafe {
        match ctx.value {
            TtyCtxValue::Data(data) => data,
            _ => fatalx(c"terminal command has no data".as_ptr(), fmt_args![]),
        }
    }
}

/// The selection a terminal command carries.
unsafe fn tty_ctx_sel_of(ctx: &tty_ctx) -> tty_ctx_sel {
    unsafe {
        match ctx.value {
            TtyCtxValue::Sel(sel) => sel,
            _ => fatalx(c"terminal command has no selection".as_ptr(), fmt_args![]),
        }
    }
}

pub unsafe fn tty_cmd_insertcharacter(mut tty: *mut tty, ctx: &tty_ctx) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
            || !(ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
            || tty_fake_bce(tty, &raw const ctx.defaults, ctx.bg) != 0
            || tty_term_has(tty_term_of(&*tty), TTYC_ICH) == 0
                && tty_term_has(tty_term_of(&*tty), TTYC_ICH1) == 0
            || (*c).overlay_check().is_some()
        {
            tty_draw_pane(tty, ctx, ctx.ocy);
            return;
        }
        tty_default_attributes(
            tty,
            &raw const ctx.defaults,
            ctx.palette,
            ctx.bg,
            (*ctx.s).hyperlinks_ptr(),
        );
        tty_cursor_pane(tty, ctx, ctx.ocx, ctx.ocy);
        tty_emulate_repeat(tty, TTYC_ICH, TTYC_ICH1, tty_ctx_num(ctx));
    }
}
pub unsafe fn tty_cmd_deletecharacter(mut tty: *mut tty, ctx: &tty_ctx) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
            || !(ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
            || tty_fake_bce(tty, &raw const ctx.defaults, ctx.bg) != 0
            || tty_term_has(tty_term_of(&*tty), TTYC_DCH) == 0
                && tty_term_has(tty_term_of(&*tty), TTYC_DCH1) == 0
            || (*c).overlay_check().is_some()
        {
            tty_draw_pane(tty, ctx, ctx.ocy);
            return;
        }
        tty_default_attributes(
            tty,
            &raw const ctx.defaults,
            ctx.palette,
            ctx.bg,
            (*ctx.s).hyperlinks_ptr(),
        );
        tty_cursor_pane(tty, ctx, ctx.ocx, ctx.ocy);
        tty_emulate_repeat(tty, TTYC_DCH, TTYC_DCH1, tty_ctx_num(ctx));
    }
}
pub unsafe fn tty_cmd_clearcharacter(mut tty: *mut tty, ctx: &tty_ctx) {
    unsafe {
        tty_default_attributes(
            tty,
            &raw const ctx.defaults,
            ctx.palette,
            ctx.bg,
            (*ctx.s).hyperlinks_ptr(),
        );
        tty_clear_pane_line(tty, ctx, ctx.ocy, ctx.ocx, tty_ctx_num(ctx), ctx.bg);
    }
}
pub unsafe fn tty_cmd_insertline(mut tty: *mut tty, ctx: &tty_ctx) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
            || !(ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
            || tty_fake_bce(tty, &raw const ctx.defaults, ctx.bg) != 0
            || tty_term_has(tty_term_of(&*tty), TTYC_CSR) == 0
            || tty_term_has(tty_term_of(&*tty), TTYC_IL1) == 0
            || ctx.sx == 1 as u_int
            || ctx.sy == 1 as u_int
            || (*c).overlay_check().is_some()
        {
            tty_redraw_region(tty, ctx);
            return;
        }
        tty_default_attributes(
            tty,
            &raw const ctx.defaults,
            ctx.palette,
            ctx.bg,
            (*ctx.s).hyperlinks_ptr(),
        );
        tty_region_pane(tty, ctx, ctx.orupper, ctx.orlower);
        tty_margin_off(tty);
        tty_cursor_pane(tty, ctx, ctx.ocx, ctx.ocy);
        tty_emulate_repeat(tty, TTYC_IL, TTYC_IL1, tty_ctx_num(ctx));
        (*tty).cy = UINT_MAX as u_int;
        (*tty).cx = (*tty).cy;
    }
}
pub unsafe fn tty_cmd_deleteline(mut tty: *mut tty, ctx: &tty_ctx) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
            || !(ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
            || tty_fake_bce(tty, &raw const ctx.defaults, ctx.bg) != 0
            || tty_term_has(tty_term_of(&*tty), TTYC_CSR) == 0
            || tty_term_has(tty_term_of(&*tty), TTYC_DL1) == 0
            || ctx.sx == 1 as u_int
            || ctx.sy == 1 as u_int
            || (*c).overlay_check().is_some()
        {
            tty_redraw_region(tty, ctx);
            return;
        }
        tty_default_attributes(
            tty,
            &raw const ctx.defaults,
            ctx.palette,
            ctx.bg,
            (*ctx.s).hyperlinks_ptr(),
        );
        tty_region_pane(tty, ctx, ctx.orupper, ctx.orlower);
        tty_margin_off(tty);
        tty_cursor_pane(tty, ctx, ctx.ocx, ctx.ocy);
        tty_emulate_repeat(tty, TTYC_DL, TTYC_DL1, tty_ctx_num(ctx));
        (*tty).cy = UINT_MAX as u_int;
        (*tty).cx = (*tty).cy;
    }
}
pub unsafe fn tty_cmd_reverseindex(mut tty: *mut tty, ctx: &tty_ctx) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        if ctx.ocy != ctx.orupper {
            return;
        }
        if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
            || !(ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
                && tty_term_of(&*tty).flags & TERM_DECSLRM == 0
            || tty_fake_bce(tty, &raw const ctx.defaults, 8 as u_int) != 0
            || tty_term_has(tty_term_of(&*tty), TTYC_CSR) == 0
            || tty_term_has(tty_term_of(&*tty), TTYC_RI) == 0
                && tty_term_has(tty_term_of(&*tty), TTYC_RIN) == 0
            || ctx.sx == 1 as u_int
            || ctx.sy == 1 as u_int
            || (*c).overlay_check().is_some()
        {
            tty_redraw_region(tty, ctx);
            return;
        }
        tty_default_attributes(
            tty,
            &raw const ctx.defaults,
            ctx.palette,
            ctx.bg,
            (*ctx.s).hyperlinks_ptr(),
        );
        tty_region_pane(tty, ctx, ctx.orupper, ctx.orlower);
        tty_margin_pane(tty, ctx);
        tty_cursor_pane(tty, ctx, ctx.ocx, ctx.orupper);
        if tty_term_has(tty_term_of(&*tty), TTYC_RI) != 0 {
            tty_putcode(tty, TTYC_RI);
        } else {
            tty_putcode_i(tty, TTYC_RIN, 1 as ::core::ffi::c_int);
        };
    }
}
pub unsafe fn tty_cmd_scrollup(mut tty: *mut tty, ctx: &tty_ctx) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        let mut i: u_int = 0;
        if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
            || !(ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
                && tty_term_of(&*tty).flags & TERM_DECSLRM == 0
            || tty_fake_bce(tty, &raw const ctx.defaults, 8 as u_int) != 0
            || tty_term_has(tty_term_of(&*tty), TTYC_CSR) == 0
            || ctx.sx == 1 as u_int
            || ctx.sy == 1 as u_int
            || (*c).overlay_check().is_some()
        {
            tty_redraw_region(tty, ctx);
            return;
        }
        tty_default_attributes(
            tty,
            &raw const ctx.defaults,
            ctx.palette,
            ctx.bg,
            (*ctx.s).hyperlinks_ptr(),
        );
        tty_region_pane(tty, ctx, ctx.orupper, ctx.orlower);
        tty_margin_pane(tty, ctx);
        if tty_ctx_num(ctx) == 1 as u_int || tty_term_has(tty_term_of(&*tty), TTYC_INDN) == 0 {
            if tty_term_of(&*tty).flags & TERM_DECSLRM == 0 {
                tty_cursor(tty, 0 as u_int, (*tty).rlower);
            } else {
                tty_cursor(tty, (*tty).rright, (*tty).rlower);
            }
            i = 0 as u_int;
            while i < tty_ctx_num(ctx) {
                tty_putc(tty, '\n' as i32 as u_char);
                i = i.wrapping_add(1);
            }
        } else {
            if (*tty).cy == UINT_MAX {
                tty_cursor(tty, 0 as u_int, 0 as u_int);
            } else {
                tty_cursor(tty, 0 as u_int, (*tty).cy);
            }
            tty_putcode_i(tty, TTYC_INDN, tty_ctx_num(ctx) as ::core::ffi::c_int);
        };
    }
}
pub unsafe fn tty_cmd_scrolldown(mut tty: *mut tty, ctx: &tty_ctx) {
    unsafe {
        let mut i: u_int = 0;
        let mut c: *mut client = tty_client(tty);
        if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
            || !(ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
                && tty_term_of(&*tty).flags & TERM_DECSLRM == 0
            || tty_fake_bce(tty, &raw const ctx.defaults, 8 as u_int) != 0
            || tty_term_has(tty_term_of(&*tty), TTYC_CSR) == 0
            || tty_term_has(tty_term_of(&*tty), TTYC_RI) == 0
                && tty_term_has(tty_term_of(&*tty), TTYC_RIN) == 0
            || ctx.sx == 1 as u_int
            || ctx.sy == 1 as u_int
            || (*c).overlay_check().is_some()
        {
            tty_redraw_region(tty, ctx);
            return;
        }
        tty_default_attributes(
            tty,
            &raw const ctx.defaults,
            ctx.palette,
            ctx.bg,
            (*ctx.s).hyperlinks_ptr(),
        );
        tty_region_pane(tty, ctx, ctx.orupper, ctx.orlower);
        tty_margin_pane(tty, ctx);
        tty_cursor_pane(tty, ctx, ctx.ocx, ctx.orupper);
        if tty_term_has(tty_term_of(&*tty), TTYC_RIN) != 0 {
            tty_putcode_i(tty, TTYC_RIN, tty_ctx_num(ctx) as ::core::ffi::c_int);
        } else {
            i = 0 as u_int;
            while i < tty_ctx_num(ctx) {
                tty_putcode(tty, TTYC_RI);
                i = i.wrapping_add(1);
            }
        };
    }
}
pub unsafe fn tty_cmd_clearendofscreen(mut tty: *mut tty, ctx: &tty_ctx) {
    unsafe {
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut nx: u_int = 0;
        let mut ny: u_int = 0;
        tty_default_attributes(
            tty,
            &raw const ctx.defaults,
            ctx.palette,
            ctx.bg,
            (*ctx.s).hyperlinks_ptr(),
        );
        tty_region_pane(tty, ctx, 0 as u_int, ctx.sy.wrapping_sub(1 as u_int));
        tty_margin_off(tty);
        px = 0 as u_int;
        nx = ctx.sx;
        py = ctx.ocy.wrapping_add(1 as u_int);
        ny = ctx.sy.wrapping_sub(ctx.ocy).wrapping_sub(1 as u_int);
        tty_clear_pane_area(tty, ctx, py, ny, px, nx, ctx.bg);
        px = ctx.ocx;
        nx = ctx.sx.wrapping_sub(ctx.ocx);
        py = ctx.ocy;
        tty_clear_pane_line(tty, ctx, py, px, nx, ctx.bg);
    }
}
pub unsafe fn tty_cmd_clearstartofscreen(mut tty: *mut tty, ctx: &tty_ctx) {
    unsafe {
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut nx: u_int = 0;
        let mut ny: u_int = 0;
        tty_default_attributes(
            tty,
            &raw const ctx.defaults,
            ctx.palette,
            ctx.bg,
            (*ctx.s).hyperlinks_ptr(),
        );
        tty_region_pane(tty, ctx, 0 as u_int, ctx.sy.wrapping_sub(1 as u_int));
        tty_margin_off(tty);
        px = 0 as u_int;
        nx = ctx.sx;
        py = 0 as u_int;
        ny = ctx.ocy;
        tty_clear_pane_area(tty, ctx, py, ny, px, nx, ctx.bg);
        px = 0 as u_int;
        nx = ctx.ocx.wrapping_add(1 as u_int);
        py = ctx.ocy;
        tty_clear_pane_line(tty, ctx, py, px, nx, ctx.bg);
    }
}
pub unsafe fn tty_cmd_clearscreen(mut tty: *mut tty, ctx: &tty_ctx) {
    unsafe {
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut nx: u_int = 0;
        let mut ny: u_int = 0;
        tty_default_attributes(
            tty,
            &raw const ctx.defaults,
            ctx.palette,
            ctx.bg,
            (*ctx.s).hyperlinks_ptr(),
        );
        tty_region_pane(tty, ctx, 0 as u_int, ctx.sy.wrapping_sub(1 as u_int));
        tty_margin_off(tty);
        px = 0 as u_int;
        nx = ctx.sx;
        py = 0 as u_int;
        ny = ctx.sy;
        tty_clear_pane_area(tty, ctx, py, ny, px, nx, ctx.bg);
    }
}
pub unsafe fn tty_cmd_alignmenttest(mut tty: *mut tty, ctx: &tty_ctx) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        let mut i: u_int = 0;
        let mut j: u_int = 0;
        if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0 || (*c).overlay_check().is_some() {
            ctx.redraw_cb.expect("non-null function pointer")(ctx);
            return;
        }
        tty_attributes(
            tty,
            &raw const grid_default_cell,
            &raw const ctx.defaults,
            ctx.palette,
            (*ctx.s).hyperlinks_ptr(),
        );
        tty_region_pane(tty, ctx, 0 as u_int, ctx.sy.wrapping_sub(1 as u_int));
        tty_margin_off(tty);
        j = 0 as u_int;
        while j < ctx.sy {
            tty_cursor_pane(tty, ctx, 0 as u_int, j);
            i = 0 as u_int;
            while i < ctx.sx {
                tty_putc(tty, 'E' as i32 as u_char);
                i = i.wrapping_add(1);
            }
            j = j.wrapping_add(1);
        }
    }
}
pub unsafe fn tty_cmd_cell(mut tty: *mut tty, ctx: &tty_ctx) {
    unsafe {
        let mut gcp: *const grid_cell = ctx.cell;
        let mut s: *mut screen = ctx.s;
        let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut i: u_int = 0;
        let mut vis: u_int = 0 as u_int;
        px = (ctx.xoff as u_int)
            .wrapping_add(ctx.ocx)
            .wrapping_sub(ctx.wox);
        py = (ctx.yoff as u_int)
            .wrapping_add(ctx.ocy)
            .wrapping_sub(ctx.woy);
        if tty_is_visible(tty, ctx, ctx.ocx, ctx.ocy, 1 as u_int, 1 as u_int) == 0 {
            return;
        }
        if (*gcp).data.width as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            && tty_check_overlay(tty, px, py) == 0
        {
            return;
        }
        if (*gcp).data.width as ::core::ffi::c_int > 1 as ::core::ffi::c_int {
            r = tty_check_overlay_range(tty, px, py, (*gcp).data.width as u_int);
            i = 0 as u_int;
            while i < (*r).used {
                vis = vis.wrapping_add((*(*r).ranges.as_mut_ptr().offset(i as isize)).nx);
                i = i.wrapping_add(1);
            }
            if vis < (*gcp).data.width as u_int {
                tty_draw_line(
                    tty,
                    s,
                    (*s).cx,
                    (*s).cy,
                    (*gcp).data.width as u_int,
                    px,
                    py,
                    &raw const ctx.defaults,
                    ctx.palette,
                );
                return;
            }
        }
        if (ctx.xoff as u_int)
            .wrapping_add(ctx.ocx)
            .wrapping_sub(ctx.wox)
            > (*tty).sx.wrapping_sub(1 as u_int)
            && ctx.ocy == ctx.orlower
            && (ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
        {
            tty_region_pane(tty, ctx, ctx.orupper, ctx.orlower);
        }
        tty_margin_off(tty);
        if ctx.flags & TTY_CTX_CELL_INVALIDATE != 0 {
            tty_invalidate(tty);
        }
        tty_cursor_pane_unless_wrap(tty, ctx, ctx.ocx, ctx.ocy);
        tty_cell(
            tty,
            ctx.cell,
            &raw const ctx.defaults,
            ctx.palette,
            (*ctx.s).hyperlinks_ptr(),
        );
        if ctx.flags & TTY_CTX_CELL_INVALIDATE != 0 {
            tty_invalidate(tty);
        }
    }
}
pub unsafe fn tty_cmd_cells(mut tty: *mut tty, ctx: &tty_ctx) {
    unsafe {
        let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
        let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
        let mut i: u_int = 0;
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut cx: u_int = 0;
        let data = tty_ctx_bytes(ctx);
        let mut cp: *const ::core::ffi::c_char = data.data;
        let mut n: size_t = data.size;
        if tty_is_visible(tty, ctx, ctx.ocx, ctx.ocy, n as u_int, 1 as u_int) == 0 {
            return;
        }
        if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
            && ((ctx.xoff as u_int).wrapping_add(ctx.ocx) < ctx.wox
                || ((ctx.xoff as u_int).wrapping_add(ctx.ocx) as size_t).wrapping_add(n)
                    > ctx.wox.wrapping_add(ctx.wsx) as size_t)
        {
            if !ctx.flags & TTY_CTX_WRAPPED != 0
                || !(ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
                || tty_term_of(&*tty).flags & TERM_NOAM != 0
                || (ctx.xoff as u_int).wrapping_add(ctx.ocx) != 0 as u_int
                || (ctx.yoff as u_int).wrapping_add(ctx.ocy) != (*tty).cy.wrapping_add(1 as u_int)
                || (*tty).cx < (*tty).sx
                || (*tty).cy == (*tty).rlower
            {
                tty_draw_pane(tty, ctx, ctx.ocy);
            } else {
                ctx.redraw_cb.expect("non-null function pointer")(ctx);
            }
            return;
        }
        tty_margin_off(tty);
        tty_cursor_pane_unless_wrap(tty, ctx, ctx.ocx, ctx.ocy);
        tty_attributes(
            tty,
            ctx.cell,
            &raw const ctx.defaults,
            ctx.palette,
            (*ctx.s).hyperlinks_ptr(),
        );
        px = (ctx.xoff as u_int)
            .wrapping_add(ctx.ocx)
            .wrapping_sub(ctx.wox);
        py = (ctx.yoff as u_int)
            .wrapping_add(ctx.ocy)
            .wrapping_sub(ctx.woy);
        r = tty_check_overlay_range(tty, px, py, n as u_int);
        i = 0 as u_int;
        while i < (*r).used {
            ri = (*r).ranges.as_mut_ptr().offset(i as isize);
            if (*ri).nx != 0 as u_int {
                cx = (*ri)
                    .px
                    .wrapping_sub(ctx.xoff as u_int)
                    .wrapping_add(ctx.wox);
                tty_cursor_pane_unless_wrap(tty, ctx, cx, ctx.ocy);
                tty_putn(
                    tty,
                    cp.offset((*ri).px as isize).offset(-(px as isize))
                        as *const ::core::ffi::c_char,
                    (*ri).nx as size_t,
                    (*ri).nx,
                );
            }
            i = i.wrapping_add(1);
        }
    }
}
pub unsafe fn tty_cmd_setselection(mut tty: *mut tty, ctx: &tty_ctx) {
    unsafe {
        tty_set_selection(
            tty,
            tty_ctx_sel_of(ctx).clip,
            tty_ctx_sel_of(ctx).data,
            tty_ctx_sel_of(ctx).size,
        );
    }
}
pub unsafe fn tty_set_selection(
    mut tty: *mut tty,
    mut clip: *const ::core::ffi::c_char,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
) {
    unsafe {
        let mut size: size_t = 0;
        if !(*tty).flags & TTY_STARTED != 0 {
            return;
        }
        if tty_term_has(tty_term_of(&*tty), TTYC_MS) == 0 {
            return;
        }
        size = (4 as size_t)
            .wrapping_mul(len.wrapping_add(2 as size_t).wrapping_div(3 as size_t))
            .wrapping_add(1 as size_t);
        let mut encoded: Vec<u8> = vec![0_u8; size as usize];
        __b64_ntop(
            buf as *const ::core::ffi::c_uchar,
            len,
            encoded.as_mut_ptr() as *mut ::core::ffi::c_char,
            size,
        );
        (*tty).flags |= TTY_NOBLOCK;
        tty_putcode_ss(
            tty,
            TTYC_MS,
            clip,
            encoded.as_ptr() as *const ::core::ffi::c_char,
        );
    }
}
pub unsafe fn tty_cmd_rawstring(mut tty: *mut tty, ctx: &tty_ctx) {
    unsafe {
        (*tty).flags |= TTY_NOBLOCK;
        tty_add(tty, tty_ctx_bytes(ctx).data, tty_ctx_bytes(ctx).size);
        tty_invalidate(tty);
    }
}
pub unsafe fn tty_cmd_syncstart(mut tty: *mut tty, ctx: &tty_ctx) {
    unsafe {
        let mut c: *mut client = tty_client(tty);
        if ctx.flags & TTY_CTX_OVERLAY_SYNC != 0 && ctx.flags & TTY_CTX_SYNC != 0 {
            tty_sync_start(tty);
        } else if !ctx.flags & TTY_CTX_OVERLAY_SYNC != 0
            && (ctx.flags & TTY_CTX_SYNC != 0 || (*c).overlay().is_some())
        {
            tty_sync_start(tty);
        }
    }
}
pub unsafe fn tty_cell(
    mut tty: *mut tty,
    mut gc: *const grid_cell,
    mut defaults: *const grid_cell,
    mut palette: *mut colour_palette,
    mut hl: *mut hyperlinks,
) {
    unsafe {
        let mut gcp: *const grid_cell = ::core::ptr::null::<grid_cell>();
        if tty_term_of(&*tty).flags & TERM_NOAM != 0
            && (*tty).cy == (*tty).sy.wrapping_sub(1 as u_int)
            && (*tty).cx == (*tty).sx.wrapping_sub(1 as u_int)
        {
            return;
        }
        if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
            return;
        }
        if tty_check_overlay(tty, (*tty).cx, (*tty).cy) == 0 {
            return;
        }
        let checked = tty_check_codeset(tty, gc);
        gcp = &raw const checked;
        tty_attributes(tty, gcp, defaults, palette, hl);
        if (*gcp).data.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            if (*(&raw const (*gcp).data.data as *const u_char) as ::core::ffi::c_int)
                < 0x20 as ::core::ffi::c_int
                || *(&raw const (*gcp).data.data as *const u_char) as ::core::ffi::c_int
                    == 0x7f as ::core::ffi::c_int
            {
                return;
            }
            tty_putc(tty, *(&raw const (*gcp).data.data as *const u_char));
            return;
        }
        tty_putn(
            tty,
            &raw const (*gcp).data.data as *const u_char as *const ::core::ffi::c_char,
            (*gcp).data.size as size_t,
            (*gcp).data.width as u_int,
        );
    }
}
pub unsafe fn tty_reset(mut tty: *mut tty) {
    unsafe {
        let mut gc: *mut grid_cell = &raw mut (*tty).cell;
        if grid_cells_equal(gc, &raw const grid_default_cell) == 0 {
            if (*gc).link != 0 as u_int {
                tty_putcode_ss(tty, TTYC_HLS, c"".as_ptr(), c"".as_ptr());
            }
            if (*gc).attr as ::core::ffi::c_int & GRID_ATTR_CHARSET != 0 && tty_acs_needed(tty) != 0
            {
                tty_putcode(tty, TTYC_RMACS);
            }
            tty_putcode(tty, TTYC_SGR0);
            *gc = grid_default_cell;
        }
        (*tty).last_cell = grid_default_cell;
    }
}
pub unsafe fn tty_invalidate(mut tty: *mut tty) {
    unsafe {
        (*tty).cell = grid_default_cell;
        (*tty).last_cell = grid_default_cell;
        (*tty).cy = UINT_MAX as u_int;
        (*tty).cx = (*tty).cy;
        (*tty).rleft = UINT_MAX as u_int;
        (*tty).rupper = (*tty).rleft;
        (*tty).rright = UINT_MAX as u_int;
        (*tty).rlower = (*tty).rright;
        if (*tty).flags & TTY_STARTED != 0 {
            if tty_term_of(&*tty).flags & TERM_DECSLRM != 0 {
                tty_putcode(tty, TTYC_ENMG);
            }
            tty_putcode(tty, TTYC_SGR0);
            (*tty).mode = ALL_MODES;
            tty_update_mode(tty, MODE_CURSOR, ::core::ptr::null_mut::<screen>());
            tty_cursor(tty, 0 as u_int, 0 as u_int);
            tty_region_off(tty);
            tty_margin_off(tty);
        } else {
            (*tty).mode = MODE_CURSOR;
        };
    }
}
pub unsafe fn tty_region_off(mut tty: *mut tty) {
    unsafe {
        tty_region(tty, 0 as u_int, (*tty).sy.wrapping_sub(1 as u_int));
    }
}
unsafe fn tty_region_pane(mut tty: *mut tty, ctx: &tty_ctx, mut rupper: u_int, mut rlower: u_int) {
    unsafe {
        tty_region(
            tty,
            (ctx.yoff as u_int)
                .wrapping_add(rupper)
                .wrapping_sub(ctx.woy),
            (ctx.yoff as u_int)
                .wrapping_add(rlower)
                .wrapping_sub(ctx.woy),
        );
    }
}
unsafe fn tty_region(mut tty: *mut tty, mut rupper: u_int, mut rlower: u_int) {
    unsafe {
        if (*tty).rlower == rlower && (*tty).rupper == rupper {
            return;
        }
        if tty_term_has(tty_term_of(&*tty), TTYC_CSR) == 0 {
            return;
        }
        (*tty).rupper = rupper;
        (*tty).rlower = rlower;
        if (*tty).cx >= (*tty).sx {
            if (*tty).cy == UINT_MAX {
                tty_cursor(tty, 0 as u_int, 0 as u_int);
            } else {
                tty_cursor(tty, 0 as u_int, (*tty).cy);
            }
        }
        tty_putcode_ii(
            tty,
            TTYC_CSR,
            (*tty).rupper as ::core::ffi::c_int,
            (*tty).rlower as ::core::ffi::c_int,
        );
        (*tty).cy = UINT_MAX as u_int;
        (*tty).cx = (*tty).cy;
    }
}
pub unsafe fn tty_margin_off(mut tty: *mut tty) {
    unsafe {
        tty_margin(tty, 0 as u_int, (*tty).sx.wrapping_sub(1 as u_int));
    }
}
unsafe fn tty_margin_pane(mut tty: *mut tty, ctx: &tty_ctx) {
    unsafe {
        let mut l: ::core::ffi::c_int = 0;
        let mut r: ::core::ffi::c_int = 0;
        l = (ctx.xoff as u_int).wrapping_sub(ctx.wox) as ::core::ffi::c_int;
        r = (ctx.xoff as u_int)
            .wrapping_add(ctx.sx)
            .wrapping_sub(1 as u_int)
            .wrapping_sub(ctx.wox) as ::core::ffi::c_int;
        if l < 0 as ::core::ffi::c_int {
            l = 0 as ::core::ffi::c_int;
        }
        if l > ctx.wsx as ::core::ffi::c_int {
            l = ctx.wsx as ::core::ffi::c_int;
        }
        if r < 0 as ::core::ffi::c_int {
            r = 0 as ::core::ffi::c_int;
        }
        if r > ctx.wsx as ::core::ffi::c_int {
            r = ctx.wsx as ::core::ffi::c_int;
        }
        tty_margin(tty, l as u_int, r as u_int);
    }
}
unsafe fn tty_margin(mut tty: *mut tty, mut rleft: u_int, mut rright: u_int) {
    unsafe {
        if tty_term_of(&*tty).flags & TERM_DECSLRM == 0 {
            return;
        }
        if (*tty).rleft == rleft && (*tty).rright == rright {
            return;
        }
        tty_putcode_ii(
            tty,
            TTYC_CSR,
            (*tty).rupper as ::core::ffi::c_int,
            (*tty).rlower as ::core::ffi::c_int,
        );
        (*tty).rleft = rleft;
        (*tty).rright = rright;
        if rleft == 0 as u_int && rright == (*tty).sx.wrapping_sub(1 as u_int) {
            tty_putcode(tty, TTYC_CLMG);
        } else {
            tty_putcode_ii(
                tty,
                TTYC_CMG,
                rleft as ::core::ffi::c_int,
                rright as ::core::ffi::c_int,
            );
        }
        (*tty).cy = UINT_MAX as u_int;
        (*tty).cx = (*tty).cy;
    }
}
unsafe fn tty_cursor_pane_unless_wrap(
    mut tty: *mut tty,
    ctx: &tty_ctx,
    mut cx: u_int,
    mut cy: u_int,
) {
    unsafe {
        if !ctx.flags & TTY_CTX_WRAPPED != 0
            || !(ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
            || tty_term_of(&*tty).flags & TERM_NOAM != 0
            || (ctx.xoff as u_int).wrapping_add(cx) != 0 as u_int
            || (ctx.yoff as u_int).wrapping_add(cy) != (*tty).cy.wrapping_add(1 as u_int)
            || (*tty).cx < (*tty).sx
            || (*tty).cy == (*tty).rlower
        {
            tty_cursor_pane(tty, ctx, cx, cy);
        } else {
            log_debug(
                c"%s: will wrap at %u,%u".as_ptr(),
                fmt_args![
                    c"tty_cursor_pane_unless_wrap".as_ptr(),
                    (*tty).cx,
                    (*tty).cy
                ],
            );
        };
    }
}
unsafe fn tty_cursor_pane(mut tty: *mut tty, ctx: &tty_ctx, mut cx: u_int, mut cy: u_int) {
    unsafe {
        tty_cursor(
            tty,
            (ctx.xoff as u_int).wrapping_add(cx).wrapping_sub(ctx.wox),
            (ctx.yoff as u_int).wrapping_add(cy).wrapping_sub(ctx.woy),
        );
    }
}
pub unsafe fn tty_cursor(mut tty: *mut tty, mut cx: u_int, mut cy: u_int) {
    unsafe {
        let mut current_block: u64;
        let term: &tty_term = tty_term_of(&*tty);
        let mut thisx: u_int = 0;
        let mut thisy: u_int = 0;
        let mut change: ::core::ffi::c_int = 0;
        if (*tty).flags & TTY_BLOCK != 0 {
            return;
        }
        thisx = (*tty).cx;
        thisy = (*tty).cy;
        if cx == thisx && cy == thisy && cx == (*tty).sx {
            return;
        }
        if cx > (*tty).sx.wrapping_sub(1 as u_int) {
            log_debug(
                c"%s: x too big %u > %u".as_ptr(),
                fmt_args![
                    c"tty_cursor".as_ptr(),
                    cx,
                    (*tty).sx.wrapping_sub(1 as u_int)
                ],
            );
            cx = (*tty).sx.wrapping_sub(1 as u_int);
        }
        if cx == thisx && cy == thisy {
            return;
        }
        if thisx > (*tty).sx.wrapping_sub(1 as u_int) {
            current_block = 415867615225958941;
        } else if cx == 0 as u_int && cy == 0 as u_int && tty_term_has(term, TTYC_HOME) != 0 {
            tty_putcode(tty, TTYC_HOME);
            current_block = 6541453531065591362;
        } else if cx == 0 as u_int
            && cy == thisy.wrapping_add(1 as u_int)
            && thisy != (*tty).rlower
            && (tty_term_of(&*tty).flags & TERM_DECSLRM == 0 || (*tty).rleft == 0 as u_int)
        {
            tty_putc(tty, '\r' as i32 as u_char);
            tty_putc(tty, '\n' as i32 as u_char);
            current_block = 6541453531065591362;
        } else if cy == thisy {
            if cx == 0 as u_int
                && (tty_term_of(&*tty).flags & TERM_DECSLRM == 0 || (*tty).rleft == 0 as u_int)
            {
                tty_putc(tty, '\r' as i32 as u_char);
                current_block = 6541453531065591362;
            } else if cx == thisx.wrapping_sub(1 as u_int) && tty_term_has(term, TTYC_CUB1) != 0 {
                tty_putcode(tty, TTYC_CUB1);
                current_block = 6541453531065591362;
            } else if cx == thisx.wrapping_add(1 as u_int) && tty_term_has(term, TTYC_CUF1) != 0 {
                tty_putcode(tty, TTYC_CUF1);
                current_block = 6541453531065591362;
            } else {
                change = thisx.wrapping_sub(cx) as ::core::ffi::c_int;
                if abs(change) as u_int > cx && tty_term_has(term, TTYC_HPA) != 0 {
                    tty_putcode_i(tty, TTYC_HPA, cx as ::core::ffi::c_int);
                    current_block = 6541453531065591362;
                } else if change > 0 as ::core::ffi::c_int
                    && tty_term_has(term, TTYC_CUB) != 0
                    && tty_term_of(&*tty).flags & TERM_DECSLRM == 0
                {
                    if change == 2 as ::core::ffi::c_int && tty_term_has(term, TTYC_CUB1) != 0 {
                        tty_putcode(tty, TTYC_CUB1);
                        tty_putcode(tty, TTYC_CUB1);
                    } else {
                        tty_putcode_i(tty, TTYC_CUB, change);
                    }
                    current_block = 6541453531065591362;
                } else if change < 0 as ::core::ffi::c_int
                    && tty_term_has(term, TTYC_CUF) != 0
                    && tty_term_of(&*tty).flags & TERM_DECSLRM == 0
                {
                    tty_putcode_i(tty, TTYC_CUF, -change);
                    current_block = 6541453531065591362;
                } else {
                    current_block = 415867615225958941;
                }
            }
        } else if cx == thisx {
            if thisy != (*tty).rupper
                && cy == thisy.wrapping_sub(1 as u_int)
                && tty_term_has(term, TTYC_CUU1) != 0
            {
                tty_putcode(tty, TTYC_CUU1);
                current_block = 6541453531065591362;
            } else if thisy != (*tty).rlower
                && cy == thisy.wrapping_add(1 as u_int)
                && tty_term_has(term, TTYC_CUD1) != 0
            {
                tty_putcode(tty, TTYC_CUD1);
                current_block = 6541453531065591362;
            } else {
                change = thisy.wrapping_sub(cy) as ::core::ffi::c_int;
                if abs(change) as u_int > cy
                    || change < 0 as ::core::ffi::c_int
                        && cy.wrapping_sub(change as u_int) > (*tty).rlower
                    || change > 0 as ::core::ffi::c_int
                        && cy.wrapping_sub(change as u_int) < (*tty).rupper
                {
                    if tty_term_has(term, TTYC_VPA) != 0 {
                        tty_putcode_i(tty, TTYC_VPA, cy as ::core::ffi::c_int);
                        current_block = 6541453531065591362;
                    } else {
                        current_block = 415867615225958941;
                    }
                } else if change > 0 as ::core::ffi::c_int && tty_term_has(term, TTYC_CUU) != 0 {
                    tty_putcode_i(tty, TTYC_CUU, change);
                    current_block = 6541453531065591362;
                } else if change < 0 as ::core::ffi::c_int && tty_term_has(term, TTYC_CUD) != 0 {
                    tty_putcode_i(tty, TTYC_CUD, -change);
                    current_block = 6541453531065591362;
                } else {
                    current_block = 415867615225958941;
                }
            }
        } else {
            current_block = 415867615225958941;
        }
        if current_block == 415867615225958941 {
            tty_putcode_ii(
                tty,
                TTYC_CUP,
                cy as ::core::ffi::c_int,
                cx as ::core::ffi::c_int,
            );
        }
        (*tty).cx = cx;
        (*tty).cy = cy;
    }
}
unsafe fn tty_hyperlink(mut tty: *mut tty, mut gc: *const grid_cell, mut hl: *mut hyperlinks) {
    unsafe {
        if (*gc).link == (*tty).cell.link {
            return;
        }
        (*tty).cell.link = (*gc).link;
        if hl.is_null() {
            return;
        }
        let found = if (*gc).link == 0 as u_int {
            None
        } else {
            hyperlinks_get(&*hl, (*gc).link)
        };
        match found {
            Some((uri, _, external_id)) => {
                tty_putcode_ss(tty, TTYC_HLS, external_id.as_ptr(), uri.as_ptr())
            }
            None => tty_putcode_ss(tty, TTYC_HLS, c"".as_ptr(), c"".as_ptr()),
        };
    }
}
pub unsafe fn tty_attributes(
    mut tty: *mut tty,
    mut gc: *const grid_cell,
    mut defaults: *const grid_cell,
    mut palette: *mut colour_palette,
    mut hl: *mut hyperlinks,
) {
    unsafe {
        let mut tc: *mut grid_cell = &raw mut (*tty).cell;
        let mut gc2 = grid_default_cell;
        let mut changed: ::core::ffi::c_int = 0;
        gc2 = *gc;
        if !((*gc).flags as ::core::ffi::c_int) & GRID_FLAG_NOPALETTE != 0 {
            if gc2.fg == 8 as ::core::ffi::c_int {
                gc2.fg = (*defaults).fg;
            }
            if gc2.bg == 8 as ::core::ffi::c_int {
                gc2.bg = (*defaults).bg;
            }
        }
        if gc2.attr as ::core::ffi::c_int == (*tty).last_cell.attr as ::core::ffi::c_int
            && gc2.fg == (*tty).last_cell.fg
            && gc2.bg == (*tty).last_cell.bg
            && gc2.us == (*tty).last_cell.us
            && gc2.link == (*tty).last_cell.link
        {
            return;
        }
        if tty_term_has(tty_term_of(&*tty), TTYC_SETAB) == 0 {
            if gc2.attr as ::core::ffi::c_int & GRID_ATTR_REVERSE != 0 {
                if gc2.fg != 7 as ::core::ffi::c_int
                    && !(gc2.fg == 8 as ::core::ffi::c_int || gc2.fg == 9 as ::core::ffi::c_int)
                {
                    gc2.attr = (gc2.attr as ::core::ffi::c_int & !GRID_ATTR_REVERSE) as u_short;
                }
            } else if gc2.bg != 0 as ::core::ffi::c_int
                && !(gc2.bg == 8 as ::core::ffi::c_int || gc2.bg == 9 as ::core::ffi::c_int)
            {
                gc2.attr = (gc2.attr as ::core::ffi::c_int | GRID_ATTR_REVERSE) as u_short;
            }
        }
        tty_check_fg(tty, palette, &raw mut gc2);
        tty_check_bg(tty, palette, &raw mut gc2);
        tty_check_us(tty, palette, &raw mut gc2);
        if (*tc).attr as ::core::ffi::c_int & !(gc2.attr as ::core::ffi::c_int) != 0
            || (*tc).us != gc2.us && gc2.us == 0 as ::core::ffi::c_int
        {
            tty_reset(tty);
        }
        tty_colours(tty, &raw mut gc2);
        changed = gc2.attr as ::core::ffi::c_int & !((*tc).attr as ::core::ffi::c_int);
        (*tc).attr = gc2.attr;
        if changed & GRID_ATTR_BRIGHT != 0 {
            tty_putcode(tty, TTYC_BOLD);
        }
        if changed & GRID_ATTR_DIM != 0 {
            tty_putcode(tty, TTYC_DIM);
        }
        if changed & GRID_ATTR_ITALICS != 0 {
            tty_set_italics(tty);
        }
        if changed & GRID_ATTR_ALL_UNDERSCORE != 0 {
            if changed & GRID_ATTR_UNDERSCORE != 0 {
                tty_putcode(tty, TTYC_SMUL);
            } else if changed & GRID_ATTR_UNDERSCORE_2 != 0 {
                tty_putcode_i(tty, TTYC_SMULX, 2 as ::core::ffi::c_int);
            } else if changed & GRID_ATTR_UNDERSCORE_3 != 0 {
                tty_putcode_i(tty, TTYC_SMULX, 3 as ::core::ffi::c_int);
            } else if changed & GRID_ATTR_UNDERSCORE_4 != 0 {
                tty_putcode_i(tty, TTYC_SMULX, 4 as ::core::ffi::c_int);
            } else if changed & GRID_ATTR_UNDERSCORE_5 != 0 {
                tty_putcode_i(tty, TTYC_SMULX, 5 as ::core::ffi::c_int);
            }
        }
        if changed & GRID_ATTR_BLINK != 0 {
            tty_putcode(tty, TTYC_BLINK);
        }
        if changed & GRID_ATTR_REVERSE != 0 {
            if tty_term_has(tty_term_of(&*tty), TTYC_REV) != 0 {
                tty_putcode(tty, TTYC_REV);
            } else if tty_term_has(tty_term_of(&*tty), TTYC_SMSO) != 0 {
                tty_putcode(tty, TTYC_SMSO);
            }
        }
        if changed & GRID_ATTR_HIDDEN != 0 {
            tty_putcode(tty, TTYC_INVIS);
        }
        if changed & GRID_ATTR_STRIKETHROUGH != 0 {
            tty_putcode(tty, TTYC_SMXX);
        }
        if changed & GRID_ATTR_OVERLINE != 0 {
            tty_putcode(tty, TTYC_SMOL);
        }
        if changed & GRID_ATTR_CHARSET != 0 && tty_acs_needed(tty) != 0 {
            tty_putcode(tty, TTYC_SMACS);
        }
        tty_hyperlink(tty, gc, hl);
        (*tty).last_cell = gc2;
    }
}
unsafe fn tty_colours(mut tty: *mut tty, mut gc: *const grid_cell) {
    unsafe {
        let mut tc: *mut grid_cell = &raw mut (*tty).cell;
        if (*gc).fg == (*tc).fg && (*gc).bg == (*tc).bg && (*gc).us == (*tc).us {
            return;
        }
        if (*gc).fg == 8 as ::core::ffi::c_int
            || (*gc).fg == 9 as ::core::ffi::c_int
            || ((*gc).bg == 8 as ::core::ffi::c_int || (*gc).bg == 9 as ::core::ffi::c_int)
        {
            if tty_term_flag(tty_term_of(&*tty), TTYC_AX) == 0 {
                tty_reset(tty);
            } else {
                if ((*gc).fg == 8 as ::core::ffi::c_int || (*gc).fg == 9 as ::core::ffi::c_int)
                    && !((*tc).fg == 8 as ::core::ffi::c_int || (*tc).fg == 9 as ::core::ffi::c_int)
                {
                    tty_puts(tty, c"\x1B[39m".as_ptr());
                    (*tc).fg = (*gc).fg;
                }
                if ((*gc).bg == 8 as ::core::ffi::c_int || (*gc).bg == 9 as ::core::ffi::c_int)
                    && !((*tc).bg == 8 as ::core::ffi::c_int || (*tc).bg == 9 as ::core::ffi::c_int)
                {
                    tty_puts(tty, c"\x1B[49m".as_ptr());
                    (*tc).bg = (*gc).bg;
                }
            }
        }
        if !((*gc).fg == 8 as ::core::ffi::c_int || (*gc).fg == 9 as ::core::ffi::c_int)
            && (*gc).fg != (*tc).fg
        {
            tty_colours_fg(tty, gc);
        }
        if !((*gc).bg == 8 as ::core::ffi::c_int || (*gc).bg == 9 as ::core::ffi::c_int)
            && (*gc).bg != (*tc).bg
        {
            tty_colours_bg(tty, gc);
        }
        if (*gc).us != (*tc).us {
            tty_colours_us(tty, gc);
        }
    }
}
unsafe fn tty_check_fg(
    mut tty: *mut tty,
    mut palette: *mut colour_palette,
    mut gc: *mut grid_cell,
) {
    unsafe {
        let mut r: u_char = 0;
        let mut g: u_char = 0;
        let mut b: u_char = 0;
        let mut colours: u_int = 0;
        let mut c: ::core::ffi::c_int = 0;
        if !((*gc).flags as ::core::ffi::c_int) & GRID_FLAG_NOPALETTE != 0 {
            c = (*gc).fg;
            if c < 8 as ::core::ffi::c_int
                && (*gc).attr as ::core::ffi::c_int & GRID_ATTR_BRIGHT != 0
                && tty_term_has(tty_term_of(&*tty), TTYC_NOBR) == 0
            {
                c += 90 as ::core::ffi::c_int;
            }
            c = colour_palette_get(palette, c);
            if c != -(1 as ::core::ffi::c_int) {
                (*gc).fg = c;
            }
        }
        if (*gc).fg & COLOUR_FLAG_RGB != 0 {
            if tty_term_of(&*tty).flags & TERM_RGBCOLOURS != 0 {
                return;
            }
            colour_split_rgb((*gc).fg, &raw mut r, &raw mut g, &raw mut b);
            (*gc).fg = colour_find_rgb(r, g, b);
        }
        if tty_term_of(&*tty).flags & TERM_256COLOURS != 0 {
            colours = 256 as u_int;
        } else {
            colours = tty_term_number(tty_term_of(&*tty), TTYC_COLORS) as u_int;
        }
        if (*gc).fg & COLOUR_FLAG_256 != 0 {
            if colours >= 256 as u_int {
                return;
            }
            (*gc).fg = colour_256to16((*gc).fg);
            if !(*gc).fg & 8 as ::core::ffi::c_int != 0 {
                return;
            }
            (*gc).fg &= 7 as ::core::ffi::c_int;
            if colours >= 16 as u_int {
                (*gc).fg += 90 as ::core::ffi::c_int;
            } else if (*gc).fg == 0 as ::core::ffi::c_int && (*gc).bg == 0 as ::core::ffi::c_int {
                (*gc).fg = 7 as ::core::ffi::c_int;
            } else if (*gc).fg == 7 as ::core::ffi::c_int && (*gc).bg == 7 as ::core::ffi::c_int {
                (*gc).fg = 0 as ::core::ffi::c_int;
            }
            return;
        }
        if (*gc).fg >= 90 as ::core::ffi::c_int
            && (*gc).fg <= 97 as ::core::ffi::c_int
            && colours < 16 as u_int
        {
            (*gc).fg -= 90 as ::core::ffi::c_int;
            (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_BRIGHT) as u_short;
        }
    }
}
unsafe fn tty_check_bg(
    mut tty: *mut tty,
    mut palette: *mut colour_palette,
    mut gc: *mut grid_cell,
) {
    unsafe {
        let mut r: u_char = 0;
        let mut g: u_char = 0;
        let mut b: u_char = 0;
        let mut colours: u_int = 0;
        let mut c: ::core::ffi::c_int = 0;
        if !((*gc).flags as ::core::ffi::c_int) & GRID_FLAG_NOPALETTE != 0 {
            c = colour_palette_get(palette, (*gc).bg);
            if c != -(1 as ::core::ffi::c_int) {
                (*gc).bg = c;
            }
        }
        if (*gc).bg & COLOUR_FLAG_RGB != 0 {
            if tty_term_of(&*tty).flags & TERM_RGBCOLOURS != 0 {
                return;
            }
            colour_split_rgb((*gc).bg, &raw mut r, &raw mut g, &raw mut b);
            (*gc).bg = colour_find_rgb(r, g, b);
        }
        if tty_term_of(&*tty).flags & TERM_256COLOURS != 0 {
            colours = 256 as u_int;
        } else {
            colours = tty_term_number(tty_term_of(&*tty), TTYC_COLORS) as u_int;
        }
        if (*gc).bg & COLOUR_FLAG_256 != 0 {
            if colours >= 256 as u_int {
                return;
            }
            (*gc).bg = colour_256to16((*gc).bg);
            if !(*gc).bg & 8 as ::core::ffi::c_int != 0 {
                return;
            }
            (*gc).bg &= 7 as ::core::ffi::c_int;
            if colours >= 16 as u_int {
                (*gc).bg += 90 as ::core::ffi::c_int;
            }
            return;
        }
        if (*gc).bg >= 90 as ::core::ffi::c_int
            && (*gc).bg <= 97 as ::core::ffi::c_int
            && colours < 16 as u_int
        {
            (*gc).bg -= 90 as ::core::ffi::c_int;
        }
    }
}
unsafe fn tty_check_us(
    mut tty: *mut tty,
    mut palette: *mut colour_palette,
    mut gc: *mut grid_cell,
) {
    unsafe {
        let mut c: ::core::ffi::c_int = 0;
        if !((*gc).flags as ::core::ffi::c_int) & GRID_FLAG_NOPALETTE != 0 {
            c = colour_palette_get(palette, (*gc).us);
            if c != -(1 as ::core::ffi::c_int) {
                (*gc).us = c;
            }
        }
        if tty_term_has(tty_term_of(&*tty), TTYC_SETULC1) == 0 {
            c = colour_force_rgb((*gc).us);
            if c == -(1 as ::core::ffi::c_int) {
                (*gc).us = 8 as ::core::ffi::c_int;
            } else {
                (*gc).us = c;
            }
        }
    }
}
unsafe fn tty_colours_fg(mut tty: *mut tty, mut gc: *const grid_cell) {
    unsafe {
        let mut tc: *mut grid_cell = &raw mut (*tty).cell;
        if (*tty).cell.fg >= 90 as ::core::ffi::c_int
            && (*tty).cell.bg <= 97 as ::core::ffi::c_int
            && ((*gc).fg < 90 as ::core::ffi::c_int || (*gc).fg > 97 as ::core::ffi::c_int)
        {
            tty_reset(tty);
        }
        if (*gc).fg & COLOUR_FLAG_RGB != 0 || (*gc).fg & COLOUR_FLAG_256 != 0 {
            if !(tty_try_colour(tty, (*gc).fg, c"38".as_ptr()) == 0 as ::core::ffi::c_int) {
                return;
            }
        } else if (*gc).fg >= 90 as ::core::ffi::c_int && (*gc).fg <= 97 as ::core::ffi::c_int {
            if tty_term_of(&*tty).flags & TERM_256COLOURS != 0 {
                let s = xasprintf(c"\x1B[%dm".as_ptr(), fmt_args![(*gc).fg]);
                tty_puts(tty, s.as_ptr());
            } else {
                tty_putcode_i(
                    tty,
                    TTYC_SETAF,
                    (*gc).fg - 90 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
                );
            }
        } else {
            tty_putcode_i(tty, TTYC_SETAF, (*gc).fg);
        }
        (*tc).fg = (*gc).fg;
    }
}
unsafe fn tty_colours_bg(mut tty: *mut tty, mut gc: *const grid_cell) {
    unsafe {
        let mut tc: *mut grid_cell = &raw mut (*tty).cell;
        if (*gc).bg & COLOUR_FLAG_RGB != 0 || (*gc).bg & COLOUR_FLAG_256 != 0 {
            if !(tty_try_colour(tty, (*gc).bg, c"48".as_ptr()) == 0 as ::core::ffi::c_int) {
                return;
            }
        } else if (*gc).bg >= 90 as ::core::ffi::c_int && (*gc).bg <= 97 as ::core::ffi::c_int {
            if tty_term_of(&*tty).flags & TERM_256COLOURS != 0 {
                let s = xasprintf(
                    c"\x1B[%dm".as_ptr(),
                    fmt_args![(*gc).bg + 10 as ::core::ffi::c_int],
                );
                tty_puts(tty, s.as_ptr());
            } else {
                tty_putcode_i(
                    tty,
                    TTYC_SETAB,
                    (*gc).bg - 90 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
                );
            }
        } else {
            tty_putcode_i(tty, TTYC_SETAB, (*gc).bg);
        }
        (*tc).bg = (*gc).bg;
    }
}
unsafe fn tty_colours_us(mut tty: *mut tty, mut gc: *const grid_cell) {
    unsafe {
        let mut tc: *mut grid_cell = &raw mut (*tty).cell;
        let mut c: u_int = 0;
        let mut r: u_char = 0;
        let mut g: u_char = 0;
        let mut b: u_char = 0;
        if (*gc).us == 8 as ::core::ffi::c_int || (*gc).us == 9 as ::core::ffi::c_int {
            tty_putcode(tty, TTYC_OL);
        } else {
            if !(*gc).us & COLOUR_FLAG_RGB != 0 {
                c = (*gc).us as u_int;
                if !c & COLOUR_FLAG_256 as u_int != 0 && (c >= 90 as u_int && c <= 97 as u_int) {
                    c = c.wrapping_sub(82 as u_int);
                }
                tty_putcode_i(
                    tty,
                    TTYC_SETULC1,
                    (c & !COLOUR_FLAG_256 as u_int) as ::core::ffi::c_int,
                );
                return;
            }
            colour_split_rgb((*gc).us, &raw mut r, &raw mut g, &raw mut b);
            c = (65536 as ::core::ffi::c_int * r as ::core::ffi::c_int
                + 256 as ::core::ffi::c_int * g as ::core::ffi::c_int
                + b as ::core::ffi::c_int) as u_int;
            if tty_term_has(tty_term_of(&*tty), TTYC_SETULC) != 0 {
                tty_putcode_i(tty, TTYC_SETULC, c as ::core::ffi::c_int);
            } else if tty_term_has(tty_term_of(&*tty), TTYC_SETAL) != 0
                && tty_term_has(tty_term_of(&*tty), TTYC_RGB) != 0
            {
                tty_putcode_i(tty, TTYC_SETAL, c as ::core::ffi::c_int);
            }
        }
        (*tc).us = (*gc).us;
    }
}
unsafe fn tty_try_colour(
    mut tty: *mut tty,
    mut colour: ::core::ffi::c_int,
    mut type_0: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut r: u_char = 0;
        let mut g: u_char = 0;
        let mut b: u_char = 0;
        if colour & COLOUR_FLAG_256 != 0 {
            if *type_0 as ::core::ffi::c_int == '3' as i32
                && tty_term_has(tty_term_of(&*tty), TTYC_SETAF) != 0
            {
                tty_putcode_i(tty, TTYC_SETAF, colour & 0xff as ::core::ffi::c_int);
            } else if tty_term_has(tty_term_of(&*tty), TTYC_SETAB) != 0 {
                tty_putcode_i(tty, TTYC_SETAB, colour & 0xff as ::core::ffi::c_int);
            }
            return 0 as ::core::ffi::c_int;
        }
        if colour & COLOUR_FLAG_RGB != 0 {
            colour_split_rgb(
                colour & 0xffffff as ::core::ffi::c_int,
                &raw mut r,
                &raw mut g,
                &raw mut b,
            );
            if *type_0 as ::core::ffi::c_int == '3' as i32
                && tty_term_has(tty_term_of(&*tty), TTYC_SETRGBF) != 0
            {
                tty_putcode_iii(
                    tty,
                    TTYC_SETRGBF,
                    r as ::core::ffi::c_int,
                    g as ::core::ffi::c_int,
                    b as ::core::ffi::c_int,
                );
            } else if tty_term_has(tty_term_of(&*tty), TTYC_SETRGBB) != 0 {
                tty_putcode_iii(
                    tty,
                    TTYC_SETRGBB,
                    r as ::core::ffi::c_int,
                    g as ::core::ffi::c_int,
                    b as ::core::ffi::c_int,
                );
            }
            return 0 as ::core::ffi::c_int;
        }
        -(1 as ::core::ffi::c_int)
    }
}
unsafe fn tty_window_default_style(mut gc: *mut grid_cell, mut wp: *mut window_pane) {
    unsafe {
        *gc = grid_default_cell;
        (*gc).fg = (*wp).palette.fg;
        (*gc).bg = (*wp).palette.bg;
    }
}
unsafe fn tty_style_changed(mut wp: *mut window_pane) {
    unsafe {
        let mut oo: *mut options = options_ptr(&(*wp).options);
        log_debug(c"%%%u: style changed".as_ptr(), fmt_args![(*wp).id]);
        (*wp).flags &= !PANE_STYLECHANGED;
        let mut ft = format_create(
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<cmdq_item>(),
            (FORMAT_PANE | (*wp).id) as ::core::ffi::c_int,
            FORMAT_NOJOBS,
        );
        format_defaults(
            &mut ft,
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            wp,
        );
        tty_window_default_style(&raw mut (*wp).cached_active_gc, wp);
        style_add(
            &raw mut (*wp).cached_active_gc,
            oo,
            c"window-active-style".as_ptr(),
            Some(&mut ft),
        );
        tty_window_default_style(&raw mut (*wp).cached_gc, wp);
        style_add(
            &raw mut (*wp).cached_gc,
            oo,
            c"window-style".as_ptr(),
            Some(&mut ft),
        );
    }
}
pub unsafe fn tty_default_colours(mut gc: *mut grid_cell, mut wp: *mut window_pane) {
    unsafe {
        if (*wp).flags & PANE_STYLECHANGED != 0 {
            tty_style_changed(wp);
        }
        *gc = grid_default_cell;
        if wp == window_get_active((*wp).window)
            && (*wp).cached_active_gc.fg != 8 as ::core::ffi::c_int
        {
            (*gc).fg = (*wp).cached_active_gc.fg;
        } else {
            (*gc).fg = (*wp).cached_gc.fg;
        }
        if wp == window_get_active((*wp).window)
            && (*wp).cached_active_gc.bg != 8 as ::core::ffi::c_int
        {
            (*gc).bg = (*wp).cached_active_gc.bg;
        } else {
            (*gc).bg = (*wp).cached_gc.bg;
        };
    }
}
pub unsafe fn tty_default_attributes(
    mut tty: *mut tty,
    mut defaults: *const grid_cell,
    mut palette: *mut colour_palette,
    mut bg: u_int,
    mut hl: *mut hyperlinks,
) {
    unsafe {
        let mut gc = grid_default_cell;
        gc = grid_default_cell;
        gc.bg = bg as ::core::ffi::c_int;
        tty_attributes(tty, &raw mut gc, defaults, palette, hl);
    }
}
pub unsafe fn tty_clipboard_query(mut tty: *mut tty) {
    unsafe {
        let mut tv = timeval::from_secs(TTY_QUERY_TIMEOUT as __time_t);
        if (*tty).flags & TTY_STARTED != 0 && !(*tty).flags & TTY_OSC52QUERY != 0 {
            tty_putcode_ss(tty, TTYC_MS, c"".as_ptr(), c"?".as_ptr());
            (*tty).flags |= TTY_OSC52QUERY;
            (*tty).clipboard_timer.arm(tv);
        }
    }
}
pub unsafe fn tty_set_progress_bar(mut tty: *mut tty, mut pb: *mut progress_bar) {
    unsafe {
        if tty_term_has(tty_term_of(&*tty), TTYC_SPB) != 0 {
            tty_putcode_ii(
                tty,
                TTYC_SPB,
                (*pb).state as ::core::ffi::c_int,
                (*pb).progress,
            );
        }
    }
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
