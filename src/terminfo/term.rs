use super::features::tty_add_features;
use super::features::tty_apply_features;
use crate::compat::strnvis;
use crate::compat::strtonum;
use crate::compat::strunvis;
use crate::environ::{environ_entry_value, environ_find, environ_ptr};
use crate::ffi::{
    cur_term, del_curterm, fnmatch, setupterm, strcasecmp, strchr, strcmp, strcspn, strlen,
    strncmp, strstr, tigetflag, tigetnum, tigetstr, tiparm_s,
};
use crate::fmt_args;
use crate::fmt_engine::format_alloc;
use crate::log::{fatalx, log_debug};
use crate::options::options_get_only_ptr;
use crate::options::{options_array_first, options_array_item_value, options_array_next};
use crate::server::client_ref_from_ptr;
use crate::tmux::global_options;
use crate::tree::GlobalQueue;
use crate::tty::tty_client;
pub use crate::types::*;
use crate::xmalloc::xasprintf;
use ::core::ffi::CStr;
use ::std::ffi::CString;
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
/// What a terminal says one capability is: the entry is either missing or
/// carries the value its table entry calls for.
pub enum TtyCode {
    None,
    String(CString),
    Number(::core::ffi::c_int),
    Flag(::core::ffi::c_int),
}
pub const TTYCODE_FLAG: tty_code_type = 3;
pub const TTYCODE_NUMBER: tty_code_type = 2;
pub const TTYCODE_STRING: tty_code_type = 1;
pub const TTYCODE_NONE: tty_code_type = 0;
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
pub const OK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const VIS_OCTAL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VIS_CSTYLE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VIS_TAB: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const VIS_NL: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const TERM_NOAM: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const TERM_DECSLRM: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const TERM_DECFRA: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const TERM_RGBCOLOURS: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const TERM_VT100LIKE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const TERM_SIXEL: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
/// One terminal description the server holds, and the client whose tty owns
/// it.
pub struct tty_term_entry {
    pub term: *mut tty_term,
    pub(crate) client: Option<ClientWeak>,
}

/// Every terminal description the server holds, newest first. Each one is
/// owned by the tty of the client it was made for; this is the observer list
/// `show-messages -T` walks.
pub static tty_terms: GlobalQueue<tty_term_entry> = GlobalQueue::new();
static tty_term_codes: [tty_term_code_entry; 233] = [
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"acsc",
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: c"am",
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: c"AX",
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: c"bce",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"bel",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Bidi",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"blink",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"bold",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"civis",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"clear",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Clmg",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Cmg",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cnorm",
    },
    tty_term_code_entry {
        type_0: TTYCODE_NUMBER,
        name: c"colors",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Cr",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Cs",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"csr",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cub",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cub1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cud",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cud1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cuf",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cuf1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cup",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cuu",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cuu1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cvvis",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"dch",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"dch1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"dim",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"dl",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"dl1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Dsbp",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Dseks",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Dsfcs",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Dsmg",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"E3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"ech",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"ed",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"el",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"el1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"enacs",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Enbp",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Eneks",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Enfcs",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Enmg",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"fsl",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Hls",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"home",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"hpa",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"ich",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"ich1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"il",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"il1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"indn",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"invis",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kcbt",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kcub1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kcud1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kcuf1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kcuu1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDC",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDC3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDC4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDC5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDC6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDC7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kdch1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDN",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDN3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDN4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDN5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDN6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDN7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kend",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kEND",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kEND3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kEND4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kEND5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kEND6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kEND7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf10",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf11",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf12",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf13",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf14",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf15",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf16",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf17",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf18",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf19",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf2",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf20",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf21",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf22",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf23",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf24",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf25",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf26",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf27",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf28",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf29",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf30",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf31",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf32",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf33",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf34",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf35",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf36",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf37",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf38",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf39",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf40",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf41",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf42",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf43",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf44",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf45",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf46",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf47",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf48",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf49",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf50",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf51",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf52",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf53",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf54",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf55",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf56",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf57",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf58",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf59",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf60",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf61",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf62",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf63",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf8",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf9",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kHOM",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kHOM3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kHOM4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kHOM5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kHOM6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kHOM7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"khome",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kIC",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kIC3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kIC4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kIC5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kIC6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kIC7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kich1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kind",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kLFT",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kLFT3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kLFT4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kLFT5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kLFT6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kLFT7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kmous",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"knp",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kNXT",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kNXT3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kNXT4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kNXT5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kNXT6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kNXT7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kpp",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kPRV",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kPRV3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kPRV4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kPRV5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kPRV6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kPRV7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kri",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kRIT",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kRIT3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kRIT4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kRIT5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kRIT6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kRIT7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kUP",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kUP3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kUP4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kUP5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kUP6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kUP7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Ms",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Nobr",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"ol",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"op",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Rect",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"rev",
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: c"RGB",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"ri",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"rin",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"rmacs",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"rmcup",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"rmkx",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Se",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"setab",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"setaf",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"setal",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"setrgbb",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"setrgbf",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Setulc",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Setulc1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"sgr0",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"sitm",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"smacs",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"smcup",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"smkx",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Smol",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"smso",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"smul",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Smulx",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"smxx",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Spb",
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: c"Sxl",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Ss",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Swd",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Sync",
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: c"Tc",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"tsl",
    },
    tty_term_code_entry {
        type_0: TTYCODE_NUMBER,
        name: c"U8",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"vpa",
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: c"XT",
    },
];
pub fn tty_term_ncodes() -> u_int {
    (::core::mem::size_of::<[tty_term_code_entry; 233]>() as usize)
        .wrapping_div(::core::mem::size_of::<tty_term_code_entry>() as usize) as u_int
}
/// The slot the terminal keeps for capability `code`.
fn tty_term_code(term: &tty_term, code: tty_code_code) -> &TtyCode {
    &term.codes[code as usize]
}

/// The same slot, to write a capability the terminal has been told about.
fn tty_term_code_mut(term: &mut tty_term, code: tty_code_code) -> &mut TtyCode {
    &mut term.codes[code as usize]
}
unsafe fn tty_term_strip(s: *const ::core::ffi::c_char) -> CString {
    unsafe {
        let bytes = CStr::from_ptr(s).to_bytes();
        if !bytes.contains(&b'$') {
            return CStr::from_ptr(s).to_owned();
        }
        let mut stripped = Vec::new();
        let mut at = 0;
        while at < bytes.len() {
            if bytes[at] == b'$' && bytes.get(at + 1) == Some(&b'<') {
                while at < bytes.len() && bytes[at] != b'>' {
                    at += 1;
                }
                if bytes.get(at) == Some(&b'>') {
                    at += 1;
                }
                if at == bytes.len() {
                    break;
                }
            }
            stripped.push(bytes[at]);
            if stripped.len() == 8191 {
                break;
            }
            at += 1;
        }
        CString::from_vec_unchecked(stripped)
    }
}
/// The next capability in a colon-separated override list, with `::` read as
/// one colon, as the caller's own string. Answers nothing at the end of the
/// list, or for a capability longer than the list format allows.
fn tty_term_override_next(s: &CStr, offset: &mut size_t) -> Option<CString> {
    const LONGEST: usize = 8191;
    let bytes = s.to_bytes();
    let mut value = Vec::<u8>::new();
    let mut at: size_t = *offset;
    if at >= bytes.len() {
        return None;
    }
    while at < bytes.len() {
        if bytes[at] == b':' {
            if bytes.get(at + 1) != Some(&b':') {
                break;
            }
            value.push(b':');
            at += 2;
        } else {
            value.push(bytes[at]);
            at += 1;
        }
        if value.len() == LONGEST {
            return None;
        }
    }
    *offset = if at < bytes.len() { at + 1 } else { at };
    Some(CString::new(value).expect("a capability has no interior NUL"))
}
pub unsafe fn tty_term_apply(
    term: &mut tty_term,
    mut capabilities: *const ::core::ffi::c_char,
    mut quiet: ::core::ffi::c_int,
) {
    unsafe {
        let mut ent: *const tty_term_code_entry = ::core::ptr::null::<tty_term_code_entry>();
        let mut offset: size_t = 0 as size_t;
        let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut name: *const ::core::ffi::c_char = cstr_ptr(&term.name);
        let mut i: u_int = 0;
        let mut remove: ::core::ffi::c_int = 0;
        loop {
            let Some(next) = tty_term_override_next(CStr::from_ptr(capabilities), &mut offset)
            else {
                break;
            };
            // The scan below writes a NUL over the `=` or the trailing `@`,
            // so the capability is walked in a buffer of this loop's own.
            let mut next = next.into_bytes_with_nul();
            s = next.as_mut_ptr() as *mut ::core::ffi::c_char;
            if *s as ::core::ffi::c_int == '\0' as i32 {
                continue;
            }
            remove = 0 as ::core::ffi::c_int;
            cp = strchr(s, '=' as i32);
            let value = if !cp.is_null() {
                let fresh0 = cp;
                cp = cp.offset(1);
                *fresh0 = '\0' as i32 as ::core::ffi::c_char;
                let encoded = CStr::from_ptr(cp);
                Some(strunvis(encoded).unwrap_or_else(|| encoded.to_owned()))
            } else if *s.add(strlen(s).wrapping_sub(1 as size_t)) as ::core::ffi::c_int
                == '@' as i32
            {
                *s.add(strlen(s).wrapping_sub(1 as size_t)) = '\0' as i32 as ::core::ffi::c_char;
                remove = 1 as ::core::ffi::c_int;
                None
            } else {
                Some(CString::default())
            };
            if quiet == 0 {
                if remove != 0 {
                    log_debug(c"%s override: %s@".as_ptr(), fmt_args![name, s]);
                } else if value.as_ref().unwrap().as_bytes().is_empty() {
                    log_debug(c"%s override: %s".as_ptr(), fmt_args![name, s]);
                } else {
                    log_debug(
                        c"%s override: %s=%s".as_ptr(),
                        fmt_args![name, s, value.as_ref().unwrap().as_ptr()],
                    );
                }
            }
            i = 0 as u_int;
            while i < tty_term_ncodes() {
                ent = tty_term_codes.as_ptr().offset(i as isize);
                if !(strcmp(s, (*ent).name.as_ptr()) != 0 as ::core::ffi::c_int) {
                    let code = tty_term_code_mut(&mut *term, i as tty_code_code);
                    if remove != 0 {
                        *code = TtyCode::None;
                    } else {
                        match (*ent).type_0 {
                            TTYCODE_STRING => {
                                *code = TtyCode::String(value.as_ref().unwrap().clone());
                            }
                            TTYCODE_NUMBER => {
                                if let Ok(n) = strtonum(
                                    value.as_ref().unwrap().as_ptr(),
                                    0 as ::core::ffi::c_longlong,
                                    INT_MAX as ::core::ffi::c_longlong,
                                ) {
                                    *code = TtyCode::Number(n as ::core::ffi::c_int);
                                }
                            }
                            TTYCODE_FLAG => {
                                *code = TtyCode::Flag(1 as ::core::ffi::c_int);
                            }
                            _ => {}
                        }
                    }
                }
                i = i.wrapping_add(1);
            }
        }
    }
}
pub unsafe fn tty_term_apply_overrides(term: &mut tty_term) {
    unsafe {
        let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
        let mut a: *mut options_array_item_t = ::core::ptr::null_mut::<options_array_item_t>();
        let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut acs: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut offset: size_t = 0;
        o = options_get_only_ptr(global_options, c"terminal-overrides".as_ptr());
        a = options_array_first(o);
        while !a.is_null() {
            ov = options_array_item_value(a);
            s = (*ov).string().as_ptr();
            offset = 0 as size_t;
            let first = tty_term_override_next(CStr::from_ptr(s), &mut offset);
            if first
                .as_ref()
                .is_some_and(|first| fnmatch(first.as_ptr(), cstr_ptr(&term.name), 0) == 0)
            {
                tty_term_apply(term, s.add(offset), 0 as ::core::ffi::c_int);
            }
            a = options_array_next(o, a);
        }
        log_debug(
            c"SIXEL flag is %d".as_ptr(),
            fmt_args![(term.flags & TERM_SIXEL != 0) as ::core::ffi::c_int],
        );
        if tty_term_has(&*term, TTYC_SETRGBF) != 0 && tty_term_has(&*term, TTYC_SETRGBB) != 0 {
            term.flags |= TERM_RGBCOLOURS;
        } else {
            term.flags &= !TERM_RGBCOLOURS;
        }
        log_debug(
            c"RGBCOLOURS flag is %d".as_ptr(),
            fmt_args![(term.flags & TERM_RGBCOLOURS != 0) as ::core::ffi::c_int],
        );
        if tty_term_has(&*term, TTYC_CMG) != 0 && tty_term_has(&*term, TTYC_CLMG) != 0 {
            term.flags |= TERM_DECSLRM;
        } else {
            term.flags &= !TERM_DECSLRM;
        }
        log_debug(
            c"DECSLRM flag is %d".as_ptr(),
            fmt_args![(term.flags & TERM_DECSLRM != 0) as ::core::ffi::c_int],
        );
        if tty_term_has(&*term, TTYC_RECT) != 0 {
            term.flags |= TERM_DECFRA;
        } else {
            term.flags &= !TERM_DECFRA;
        }
        log_debug(
            c"DECFRA flag is %d".as_ptr(),
            fmt_args![(term.flags & TERM_DECFRA != 0) as ::core::ffi::c_int],
        );
        if tty_term_flag(&*term, TTYC_AM) == 0 {
            term.flags |= TERM_NOAM;
        } else {
            term.flags &= !TERM_NOAM;
        }
        log_debug(
            c"NOAM flag is %d".as_ptr(),
            fmt_args![(term.flags & TERM_NOAM != 0) as ::core::ffi::c_int],
        );
        term.acs = [[0; 2]; 256];
        if tty_term_has(&*term, TTYC_ACSC) != 0 {
            acs = tty_term_string(&*term, TTYC_ACSC);
        } else {
            acs = c"a#j+k+l+m+n+o-p-q-r-s-t+u+v+w+x|y<z>~.".as_ptr();
        }
        while *acs.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
            && *acs.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
        {
            term.acs[*acs.offset(0 as ::core::ffi::c_int as isize) as u_char as usize]
                [0 as ::core::ffi::c_int as usize] = *acs.offset(1 as ::core::ffi::c_int as isize);
            acs = acs.offset(2 as ::core::ffi::c_int as isize);
        }
    }
}
pub unsafe fn tty_term_create(
    mut tty: *mut tty,
    mut name: *mut ::core::ffi::c_char,
    caps: &[::std::ffi::CString],
    feat: &mut ::core::ffi::c_int,
) -> Result<Box<tty_term>, CString> {
    unsafe {
        let mut term: *mut tty_term = ::core::ptr::null_mut::<tty_term>();
        let mut ent: *const tty_term_code_entry = ::core::ptr::null::<tty_term_code_entry>();
        let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
        let mut a: *mut options_array_item_t = ::core::ptr::null_mut::<options_array_item_t>();
        let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
        let mut i: u_int = 0;
        let mut j: u_int = 0;
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut offset: size_t = 0;
        let mut namelen: size_t = 0;
        let mut cap: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        log_debug(c"adding term %s".as_ptr(), fmt_args![name]);
        let mut term_box = Box::new(tty_term {
            name: Some(CStr::from_ptr(name).to_owned()),
            features: 0,
            acs: [[0; 2]; 256],
            codes: (0..tty_term_ncodes()).map(|_| TtyCode::None).collect(),
            flags: 0,
        });
        term = &raw mut *term_box;
        tty_terms.queue().push_front(tty_term_entry {
            term,
            client: client_ref_from_ptr(tty_client(tty)).map(|c| c.downgrade()),
        });
        i = 0 as u_int;
        while i < caps.len() as u_int {
            cap = caps[i as usize].as_ptr();
            namelen = strcspn(cap, c"=".as_ptr()) as size_t;
            if !(namelen == 0 as size_t) {
                value = cap.add(namelen).offset(1 as ::core::ffi::c_int as isize);
                j = 0 as u_int;
                while j < tty_term_ncodes() {
                    ent = tty_term_codes.as_ptr().offset(j as isize);
                    if !(strncmp((*ent).name.as_ptr(), cap, namelen) != 0 as ::core::ffi::c_int)
                        && (*ent).name.to_bytes().len() == namelen
                    {
                        let code = tty_term_code_mut(&mut *term, j as tty_code_code);
                        *code = TtyCode::None;
                        match (*ent).type_0 {
                            TTYCODE_STRING => {
                                *code = TtyCode::String(tty_term_strip(value));
                            }
                            TTYCODE_NUMBER => {
                                match strtonum(
                                    value,
                                    0 as ::core::ffi::c_longlong,
                                    INT_MAX as ::core::ffi::c_longlong,
                                ) {
                                    Ok(n) => {
                                        *code = TtyCode::Number(n as ::core::ffi::c_int);
                                    }
                                    Err(errstr) => log_debug(
                                        c"%s: %s".as_ptr(),
                                        fmt_args![(*ent).name, errstr.as_ptr()],
                                    ),
                                }
                            }
                            TTYCODE_FLAG => {
                                *code = TtyCode::Flag(
                                    (*value as ::core::ffi::c_int == '1' as i32)
                                        as ::core::ffi::c_int,
                                );
                            }
                            _ => {}
                        }
                    }
                    j = j.wrapping_add(1);
                }
            }
            i = i.wrapping_add(1);
        }
        o = options_get_only_ptr(global_options, c"terminal-features".as_ptr());
        a = options_array_first(o);
        while !a.is_null() {
            ov = options_array_item_value(a);
            s = (*ov).string().as_ptr();
            offset = 0 as size_t;
            let first = tty_term_override_next(CStr::from_ptr(s), &mut offset);
            if first
                .as_ref()
                .is_some_and(|first| fnmatch(first.as_ptr(), cstr_ptr(&(*term).name), 0) == 0)
            {
                tty_add_features(feat, s.add(offset), c":".as_ptr());
            }
            a = options_array_next(o, a);
        }
        del_curterm(cur_term);
        let colorterm = environ_find(
            &*environ_ptr(&(*tty_client(tty)).environ),
            c"COLORTERM".as_ptr(),
        )
        .and_then(environ_entry_value);
        if let Some(colorterm) = colorterm {
            log_debug(
                c"%s COLORTERM=%s".as_ptr(),
                fmt_args![(*tty_client(tty)).name.as_deref(), colorterm],
            );
            if strcasecmp(colorterm.as_ptr(), c"truecolor".as_ptr()) == 0 as ::core::ffi::c_int
                || strcasecmp(colorterm.as_ptr(), c"24bit".as_ptr()) == 0 as ::core::ffi::c_int
            {
                tty_add_features(feat, c"RGB".as_ptr(), c",".as_ptr());
            } else if !strstr(colorterm.as_ptr(), c"256".as_ptr()).is_null() {
                tty_add_features(feat, c"256".as_ptr(), c",".as_ptr());
            }
        }
        tty_term_apply_overrides(&mut *term);
        let cause = if tty_term_has(&*term, TTYC_CLEAR) == 0 {
            xasprintf(c"terminal does not support clear".as_ptr(), fmt_args![])
        } else if tty_term_has(&*term, TTYC_CUP) == 0 {
            xasprintf(c"terminal does not support cup".as_ptr(), fmt_args![])
        } else {
            s = tty_term_string(&*term, TTYC_CLEAR);
            if tty_term_flag(&*term, TTYC_XT) != 0
                || strncmp(s, c"\x1B[".as_ptr(), 2 as size_t) == 0 as ::core::ffi::c_int
            {
                (*term).flags |= TERM_VT100LIKE;
                tty_add_features(feat, c"bpaste,focus,title".as_ptr(), c",".as_ptr());
            }
            if (tty_term_flag(&*term, TTYC_TC) != 0 || tty_term_has(&*term, TTYC_RGB) != 0)
                && (tty_term_has(&*term, TTYC_SETRGBF) == 0
                    || tty_term_has(&*term, TTYC_SETRGBB) == 0)
            {
                tty_add_features(feat, c"RGB".as_ptr(), c",".as_ptr());
            }
            if tty_apply_features(&mut *term, *feat) != 0 {
                tty_term_apply_overrides(&mut *term);
            }
            i = 0 as u_int;
            while i < tty_term_ncodes() {
                log_debug(
                    c"%s%s".as_ptr(),
                    fmt_args![
                        name,
                        tty_term_describe(&*term, i as tty_code_code).as_c_str()
                    ],
                );
                i = i.wrapping_add(1);
            }
            return Ok(term_box);
        };
        tty_term_free(term_box);
        Err(cause)
    }
}
/// The terminal description a [`tty`] holds, as the borrowed view the
/// capability lookups take, or null before one has been opened.
/// The terminal a tty is driving. A tty only reaches the code that asks about
/// capabilities once `tty_open` has given it one.
pub fn tty_term_of(tty: &tty) -> &tty_term {
    tty.term
        .as_deref()
        .expect("a tty being driven has a terminal")
}

/// The terminal description an owner holds, if `tty_open` has given it one.
pub fn tty_term_opt(value: &Option<Box<tty_term>>) -> Option<&tty_term> {
    value.as_deref()
}

/// The same description, to write the capabilities it has been told about.
pub fn tty_term_opt_mut(value: &mut Option<Box<tty_term>>) -> Option<&mut tty_term> {
    value.as_deref_mut()
}
pub unsafe fn tty_term_free(mut term: Box<tty_term>) {
    unsafe {
        let term_ptr = &raw mut *term;
        log_debug(
            c"removing term %s".as_ptr(),
            fmt_args![(*term_ptr).name.as_deref()],
        );
        let listed = tty_terms.queue();
        if let Some(at) = listed.iter().position(|one| one.term == term_ptr) {
            listed.remove(at);
        }
        drop(term);
    }
}
pub unsafe fn tty_term_read_list(
    mut name: *const ::core::ffi::c_char,
    mut fd: ::core::ffi::c_int,
) -> Result<Vec<CString>, CString> {
    unsafe {
        let mut ent: *const tty_term_code_entry = ::core::ptr::null::<tty_term_code_entry>();
        let mut error: ::core::ffi::c_int = 0;
        let mut n: ::core::ffi::c_int = 0;
        let mut i: u_int = 0;
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut tmp = CString::default();
        if setupterm(name as *mut ::core::ffi::c_char, fd, &raw mut error) != OK {
            let cause = match error {
                1 => format_alloc(c"can't use hardcopy terminal: %s".as_ptr(), fmt_args![name]),
                0 => format_alloc(
                    c"missing or unsuitable terminal: %s".as_ptr(),
                    fmt_args![name],
                ),
                -1 => format_alloc(c"can't find terminfo database".as_ptr(), fmt_args![]),
                _ => format_alloc(c"unknown error".as_ptr(), fmt_args![]),
            };
            return Err(cause);
        }
        let mut caps: Vec<CString> = Vec::new();
        let mut current_block_23: u64;
        i = 0 as u_int;
        while i < tty_term_ncodes() {
            ent = tty_term_codes.as_ptr().offset(i as isize);
            match (*ent).type_0 {
                TTYCODE_NONE => {
                    current_block_23 = 1856101646708284338;
                }
                TTYCODE_STRING => {
                    s = tigetstr((*ent).name.as_ptr() as *mut ::core::ffi::c_char);
                    if s.is_null()
                        || std::ptr::eq(s, -(1 as ::core::ffi::c_int) as *mut ::core::ffi::c_char)
                    {
                        current_block_23 = 1856101646708284338;
                    } else {
                        current_block_23 = 14763689060501151050;
                    }
                }
                TTYCODE_NUMBER => {
                    n = tigetnum((*ent).name.as_ptr() as *mut ::core::ffi::c_char);
                    if n == -(1 as ::core::ffi::c_int) || n == -(2 as ::core::ffi::c_int) {
                        current_block_23 = 1856101646708284338;
                    } else {
                        tmp = format_alloc(c"%d".as_ptr(), fmt_args![n]);
                        s = tmp.as_ptr();
                        current_block_23 = 14763689060501151050;
                    }
                }
                TTYCODE_FLAG => {
                    n = tigetflag((*ent).name.as_ptr() as *mut ::core::ffi::c_char);
                    if n == -(1 as ::core::ffi::c_int) {
                        current_block_23 = 1856101646708284338;
                    } else {
                        if n != 0 {
                            s = c"1".as_ptr();
                        } else {
                            s = c"0".as_ptr();
                        }
                        current_block_23 = 14763689060501151050;
                    }
                }
                _ => {
                    fatalx(c"unknown capability type".as_ptr(), fmt_args![]);
                }
            }
            if current_block_23 == 14763689060501151050 {
                caps.push(format_alloc(c"%s=%s".as_ptr(), fmt_args![(*ent).name, s]));
            }
            i = i.wrapping_add(1);
        }
        del_curterm(cur_term);
        Ok(caps)
    }
}
pub fn tty_term_has(term: &tty_term, code: tty_code_code) -> ::core::ffi::c_int {
    !matches!(tty_term_code(term, code), TtyCode::None) as ::core::ffi::c_int
}
pub unsafe fn tty_term_string(
    term: &tty_term,
    mut code: tty_code_code,
) -> *const ::core::ffi::c_char {
    unsafe {
        if tty_term_has(term, code) == 0 {
            return c"".as_ptr();
        }
        let TtyCode::String(value) = tty_term_code(term, code) else {
            fatalx(
                c"not a string: %d".as_ptr(),
                fmt_args![code as ::core::ffi::c_uint],
            );
        };
        value.as_ptr()
    }
}
pub unsafe fn tty_term_string_i(
    term: &tty_term,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
) -> CString {
    unsafe {
        let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        s = tiparm_s(1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int, x, a);
        if s.is_null() {
            log_debug(
                c"could not expand %s".as_ptr(),
                fmt_args![tty_term_codes[code as usize].name],
            );
            return CString::default();
        }
        CStr::from_ptr(s).to_owned()
    }
}
pub unsafe fn tty_term_string_ii(
    term: &tty_term,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
) -> CString {
    unsafe {
        let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        s = tiparm_s(2 as ::core::ffi::c_int, 0 as ::core::ffi::c_int, x, a, b);
        if s.is_null() {
            log_debug(
                c"could not expand %s".as_ptr(),
                fmt_args![tty_term_codes[code as usize].name],
            );
            return CString::default();
        }
        CStr::from_ptr(s).to_owned()
    }
}
pub unsafe fn tty_term_string_iii(
    term: &tty_term,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) -> CString {
    unsafe {
        let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        s = tiparm_s(3 as ::core::ffi::c_int, 0 as ::core::ffi::c_int, x, a, b, c);
        if s.is_null() {
            log_debug(
                c"could not expand %s".as_ptr(),
                fmt_args![tty_term_codes[code as usize].name],
            );
            return CString::default();
        }
        CStr::from_ptr(s).to_owned()
    }
}
pub unsafe fn tty_term_string_s(
    term: &tty_term,
    mut code: tty_code_code,
    mut a: *const ::core::ffi::c_char,
) -> CString {
    unsafe {
        let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        s = tiparm_s(1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int, x, a);
        if s.is_null() {
            log_debug(
                c"could not expand %s".as_ptr(),
                fmt_args![tty_term_codes[code as usize].name],
            );
            return CString::default();
        }
        CStr::from_ptr(s).to_owned()
    }
}
pub unsafe fn tty_term_string_ss(
    term: &tty_term,
    mut code: tty_code_code,
    mut a: *const ::core::ffi::c_char,
    mut b: *const ::core::ffi::c_char,
) -> CString {
    unsafe {
        let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        s = tiparm_s(2 as ::core::ffi::c_int, 3 as ::core::ffi::c_int, x, a, b);
        if s.is_null() {
            log_debug(
                c"could not expand %s".as_ptr(),
                fmt_args![tty_term_codes[code as usize].name],
            );
            return CString::default();
        }
        CStr::from_ptr(s).to_owned()
    }
}
pub fn tty_term_number(term: &tty_term, code: tty_code_code) -> ::core::ffi::c_int {
    if tty_term_has(term, code) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    let TtyCode::Number(value) = tty_term_code(term, code) else {
        unsafe {
            fatalx(
                c"not a number: %d".as_ptr(),
                fmt_args![code as ::core::ffi::c_uint],
            );
        }
    };
    *value
}
pub fn tty_term_flag(term: &tty_term, code: tty_code_code) -> ::core::ffi::c_int {
    if tty_term_has(term, code) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    let TtyCode::Flag(value) = tty_term_code(term, code) else {
        unsafe {
            fatalx(
                c"not a flag: %d".as_ptr(),
                fmt_args![code as ::core::ffi::c_uint],
            );
        }
    };
    *value
}
/// How a capability reads: its number, its name and the value the terminal
/// gave it, as the caller's own string.
pub unsafe fn tty_term_describe(term: &tty_term, mut code: tty_code_code) -> ::std::ffi::CString {
    unsafe {
        let name = tty_term_codes[code as usize].name;
        match tty_term_code(term, code) {
            TtyCode::None => format_alloc(
                c"%4u: %s: [missing]".as_ptr(),
                fmt_args![code as ::core::ffi::c_uint, name],
            ),
            TtyCode::String(value) => {
                let mut out: [::core::ffi::c_char; 128] = [0; 128];
                strnvis(
                    &raw mut out as *mut ::core::ffi::c_char,
                    value.as_ptr(),
                    ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
                    VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL,
                );
                format_alloc(
                    c"%4u: %s: (string) %s".as_ptr(),
                    fmt_args![
                        code as ::core::ffi::c_uint,
                        name,
                        &raw mut out as *mut ::core::ffi::c_char
                    ],
                )
            }
            TtyCode::Number(value) => format_alloc(
                c"%4u: %s: (number) %d".as_ptr(),
                fmt_args![code as ::core::ffi::c_uint, name, *value],
            ),
            TtyCode::Flag(value) => format_alloc(
                c"%4u: %s: (flag) %s".as_ptr(),
                fmt_args![
                    code as ::core::ffi::c_uint,
                    name,
                    if *value != 0 { c"true" } else { c"false" }
                ],
            ),
        }
    }
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
