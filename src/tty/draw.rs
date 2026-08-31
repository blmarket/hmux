use super::driver::tty_client;
use super::driver::{
    tty_attributes, tty_check_codeset, tty_cursor, tty_default_attributes, tty_fake_bce,
    tty_margin_off, tty_putc, tty_putcode, tty_putcode_i, tty_putn, tty_region_off,
    tty_repeat_space, tty_update_mode,
};
use crate::fmt_args;
use crate::grid::grid_cells_look_equal;
use crate::grid::grid_view_get_cell;
use crate::grid::{grid_default_cell, grid_get_line};
use crate::log::{fatalx, log_debug, log_get_level};
use crate::screen::{screen_grid_ptr, screen_select_cell};
use crate::terminfo::{tty_term_has, tty_term_of};
pub use crate::types::*;
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
pub type tty_draw_line_state = ::core::ffi::c_uint;
pub const TTY_DRAW_LINE_DONE: tty_draw_line_state = 6;
pub const TTY_DRAW_LINE_SAME: tty_draw_line_state = 5;
pub const TTY_DRAW_LINE_EMPTY: tty_draw_line_state = 4;
pub const TTY_DRAW_LINE_NEW2: tty_draw_line_state = 3;
pub const TTY_DRAW_LINE_NEW1: tty_draw_line_state = 2;
pub const TTY_DRAW_LINE_FLUSH: tty_draw_line_state = 1;
pub const TTY_DRAW_LINE_FIRST: tty_draw_line_state = 0;
pub const GRID_ATTR_CHARSET: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const GRID_FLAG_PADDING: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const GRID_FLAG_SELECTED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const GRID_FLAG_CLEARED: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const GRID_FLAG_TAB: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const GRID_LINE_WRAPPED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const TTY_NOCURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
static tty_draw_line_states: ReadOnly<[*const ::core::ffi::c_char; 7]> = ReadOnly::new([
    c"FIRST".as_ptr(),
    c"FLUSH".as_ptr(),
    c"NEW1".as_ptr(),
    c"NEW2".as_ptr(),
    c"EMPTY".as_ptr(),
    c"SAME".as_ptr(),
    c"DONE".as_ptr(),
]);
unsafe fn tty_draw_line_clear(
    tty: &mut tty,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    defaults: &grid_cell,
    mut bg: u_int,
    mut wrapped: ::core::ffi::c_int,
) {
    unsafe {
        if nx == 0 as u_int {
            return;
        }
        if (*tty_client(tty)).overlay_check().is_none()
            && wrapped == 0
            && nx >= 10 as u_int
            && tty_fake_bce(tty, defaults, bg) == 0
        {
            if px.wrapping_add(nx) >= (*tty).sx && tty_term_has(tty_term_of(tty), TTYC_EL) != 0 {
                tty_cursor(tty, px, py);
                tty_putcode(tty, TTYC_EL);
                return;
            }
            if px == 0 as u_int && tty_term_has(tty_term_of(tty), TTYC_EL1) != 0 {
                tty_cursor(tty, px.wrapping_add(nx).wrapping_sub(1 as u_int), py);
                tty_putcode(tty, TTYC_EL1);
                return;
            }
            if tty_term_has(tty_term_of(tty), TTYC_ECH) != 0 {
                tty_cursor(tty, px, py);
                tty_putcode_i(tty, TTYC_ECH, nx as ::core::ffi::c_int);
                return;
            }
        }
        if px != 0 as u_int || wrapped == 0 {
            tty_cursor(tty, px, py);
        }
        if nx == 1 as u_int {
            tty_putc(tty, ' ' as i32 as u_char);
        } else if nx == 2 as u_int {
            tty_putn(tty, b"  ", 2 as u_int);
        } else {
            tty_repeat_space(tty, nx);
        };
    }
}
pub unsafe fn tty_draw_line(
    tty: &mut tty,
    mut s: *mut screen,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut atx: u_int,
    mut aty: u_int,
    defaults: &grid_cell,
    mut palette: *mut colour_palette,
) {
    unsafe {
        let mut current_block: u64;
        let mut gd: *mut grid = screen_grid_ptr(&mut *s);
        let mut gcp: *const grid_cell = ::core::ptr::null::<grid_cell>();
        let mut gc = grid_default_cell;
        let mut ngc = grid_default_cell;
        let mut last = grid_default_cell;
        let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
        let mut i: u_int = 0;
        let mut j: u_int = 0;
        let mut last_i: u_int = 0;
        let mut cx: u_int = 0;
        let mut ex: u_int = 0;
        let mut width: u_int = 0;
        let mut cellsize: u_int = 0;
        let mut bg: u_int = 0;
        let mut flags: ::core::ffi::c_int = 0;
        let mut empty: ::core::ffi::c_int = 0;
        let mut wrapped: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut buf: [u8; 1000] = [0; 1000];
        let mut len: size_t = 0;
        let mut current_state: tty_draw_line_state = TTY_DRAW_LINE_FIRST;
        let mut next_state: tty_draw_line_state = TTY_DRAW_LINE_FIRST;
        log_debug(
            c"%s: px=%u py=%u nx=%u atx=%u aty=%u".as_ptr(),
            fmt_args![c"tty_draw_line".as_ptr(), px, py, nx, atx, aty],
        );
        if atx >= (*tty).sx {
            return;
        }
        if atx.wrapping_add(nx) >= (*tty).sx {
            nx = (*tty).sx.wrapping_sub(atx);
        }
        if nx == 0 as u_int {
            return;
        }
        cellsize = (*grid_get_line(&mut *gd, (*gd).hsize.wrapping_add(py))).cellsize();
        if (*screen_grid_ptr(&mut *s)).sx > cellsize {
            ex = cellsize;
        } else {
            ex = (*screen_grid_ptr(&mut *s)).sx;
        }
        log_debug(
            c"%s: drawing %u-%u,%u (end %u) at %u,%u; defaults: fg=%d, bg=%d".as_ptr(),
            fmt_args![
                c"tty_draw_line".as_ptr(),
                px,
                px.wrapping_add(nx),
                py,
                ex,
                atx,
                aty,
                (*defaults).fg,
                (*defaults).bg
            ],
        );
        flags = (*tty).flags & TTY_NOCURSOR;
        (*tty).flags |= TTY_NOCURSOR;
        tty_update_mode(tty, (*tty).mode, s);
        tty_region_off(tty);
        tty_margin_off(tty);
        last = grid_default_cell;
        last.bg = (*defaults).bg;
        tty_default_attributes(tty, defaults, palette, 8 as u_int, (*s).hyperlinks_ptr());
        cx = 0 as u_int;
        i = px;
        while i < px.wrapping_add(nx) {
            gc = grid_view_get_cell(&*gd, i, py);
            if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
                break;
            }
            cx = cx.wrapping_add(1);
            i = i.wrapping_add(1);
        }
        if cx != 0 as u_int {
            i = px.wrapping_add(1 as u_int);
            while i > 0 as u_int {
                gc = grid_view_get_cell(&*gd, i.wrapping_sub(1 as u_int), py);
                if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
                    break;
                }
                i = i.wrapping_sub(1);
            }
            if i == 0 as u_int {
                bg = (*defaults).bg as u_int;
            } else {
                bg = gc.bg as u_int;
                if gc.flags as ::core::ffi::c_int & GRID_FLAG_SELECTED != 0 {
                    ngc = gc;
                    if screen_select_cell(s, &mut ngc, &mut gc) != 0 {
                        bg = ngc.bg as u_int;
                    }
                }
            }
            tty_attributes(tty, &last, defaults, palette, (*s).hyperlinks_ptr());
            log_debug(
                c"%s: clearing %u padding cells".as_ptr(),
                fmt_args![c"tty_draw_line".as_ptr(), cx],
            );
            tty_draw_line_clear(tty, atx, aty, cx, defaults, bg, 0 as ::core::ffi::c_int);
            if cx == ex {
                current_block = 15793177467813045649;
            } else {
                atx = atx.wrapping_add(cx);
                px = px.wrapping_add(cx);
                nx = nx.wrapping_sub(cx);
                current_block = 7226443171521532240;
            }
        } else {
            current_block = 7226443171521532240;
        }
        if current_block == 7226443171521532240 {
            if py != 0 as u_int && atx == 0 as u_int && (*tty).cx >= (*tty).sx && nx == (*tty).sx {
                gl = grid_get_line(
                    &mut *gd,
                    (*gd).hsize.wrapping_add(py).wrapping_sub(1 as u_int),
                );
                if (*gl).flags & GRID_LINE_WRAPPED != 0 {
                    wrapped = 1 as ::core::ffi::c_int;
                }
            }
            i = 0 as u_int;
            last_i = i;
            len = 0 as size_t;
            width = 0 as u_int;
            current_state = TTY_DRAW_LINE_FIRST;
            loop {
                if i == nx {
                    empty = 0 as ::core::ffi::c_int;
                    next_state = TTY_DRAW_LINE_DONE;
                    gcp = &grid_default_cell;
                } else {
                    if i > nx {
                        fatalx(c"position %u > width %u".as_ptr(), fmt_args![i, nx]);
                    }
                    gc = grid_view_get_cell(&*gd, px.wrapping_add(i), py);
                    gc = tty_check_codeset(tty, &mut gc);
                    gcp = &gc;
                    if (*gcp).flags as ::core::ffi::c_int & GRID_FLAG_SELECTED != 0 {
                        ngc = *gcp;
                        if screen_select_cell(s, &mut ngc, &*gcp) != 0 {
                            gcp = &mut ngc;
                        }
                    }
                    empty = 0 as ::core::ffi::c_int;
                    if px >= ex || i >= ex.wrapping_sub(px) {
                        empty = 1 as ::core::ffi::c_int;
                    } else if (*gcp).data.width as u_int > nx.wrapping_sub(i) {
                        empty = nx.wrapping_sub(i) as ::core::ffi::c_int;
                    } else if (*gcp).flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
                        empty = 1 as ::core::ffi::c_int;
                    } else if (*gcp).bg == last.bg
                        && (*gcp).attr as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                        && (*gcp).link == 0 as u_int
                    {
                        if (*gcp).flags as ::core::ffi::c_int & GRID_FLAG_CLEARED != 0 {
                            empty = 1 as ::core::ffi::c_int;
                        } else if (*gcp).flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                            empty = (*gcp).data.width as ::core::ffi::c_int;
                        } else if (*gcp).data.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                            && *(&raw const (*gcp).data.data as *const u_char) as ::core::ffi::c_int
                                == ' ' as i32
                        {
                            empty = 1 as ::core::ffi::c_int;
                        }
                    }
                    if empty != 0 as ::core::ffi::c_int {
                        next_state = TTY_DRAW_LINE_EMPTY;
                    } else if current_state as ::core::ffi::c_uint
                        == TTY_DRAW_LINE_FIRST as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        next_state = TTY_DRAW_LINE_SAME;
                    } else if grid_cells_look_equal(&*gcp, &last) != 0 {
                        if (*gcp).data.size as usize > buf.len().wrapping_sub(len) {
                            next_state = TTY_DRAW_LINE_FLUSH;
                        } else {
                            next_state = TTY_DRAW_LINE_SAME;
                        }
                    } else if current_state as ::core::ffi::c_uint
                        == TTY_DRAW_LINE_NEW1 as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        next_state = TTY_DRAW_LINE_NEW2;
                    } else {
                        next_state = TTY_DRAW_LINE_NEW1;
                    }
                }
                if log_get_level() != 0 as ::core::ffi::c_int {
                    log_debug(
                        c"%s: cell %u empty %u, bg %u; state: current %s, next %s".as_ptr(),
                        fmt_args![
                            c"tty_draw_line".as_ptr(),
                            px.wrapping_add(i),
                            empty,
                            (*gcp).bg,
                            tty_draw_line_states[current_state as usize],
                            tty_draw_line_states[next_state as usize]
                        ],
                    );
                }
                if next_state as ::core::ffi::c_uint != current_state as ::core::ffi::c_uint {
                    if current_state as ::core::ffi::c_uint
                        == TTY_DRAW_LINE_EMPTY as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        tty_attributes(tty, &last, defaults, palette, (*s).hyperlinks_ptr());
                        tty_draw_line_clear(
                            tty,
                            atx.wrapping_add(last_i),
                            aty,
                            i.wrapping_sub(last_i),
                            defaults,
                            last.bg as u_int,
                            wrapped,
                        );
                        wrapped = 0 as ::core::ffi::c_int;
                    } else if next_state as ::core::ffi::c_uint
                        != TTY_DRAW_LINE_SAME as ::core::ffi::c_int as ::core::ffi::c_uint
                        && len != 0 as size_t
                    {
                        tty_attributes(tty, &last, defaults, palette, (*s).hyperlinks_ptr());
                        if atx.wrapping_add(i).wrapping_sub(width) != 0 as u_int || wrapped == 0 {
                            tty_cursor(tty, atx.wrapping_add(i).wrapping_sub(width), aty);
                        }
                        if !(last.attr as ::core::ffi::c_int) & GRID_ATTR_CHARSET != 0 {
                            tty_putn(tty, &buf[..len], width);
                        } else {
                            j = 0 as u_int;
                            while (j as size_t) < len {
                                tty_putc(tty, buf[j as usize]);
                                j = j.wrapping_add(1);
                            }
                        }
                        len = 0 as size_t;
                        width = 0 as u_int;
                        wrapped = 0 as ::core::ffi::c_int;
                    }
                    last_i = i;
                }
                if next_state as ::core::ffi::c_uint
                    != TTY_DRAW_LINE_EMPTY as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let size = (*gcp).data.size as usize;
                    buf[len..len + size].copy_from_slice(&(*gcp).data.data[..size]);
                    len = len.wrapping_add(size);
                    width = width.wrapping_add((*gcp).data.width as u_int);
                }
                if next_state as ::core::ffi::c_uint
                    == TTY_DRAW_LINE_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    break;
                }
                current_state = next_state;
                last = *gcp;
                if empty != 0 as ::core::ffi::c_int {
                    i = i.wrapping_add(empty as u_int);
                } else {
                    i = i.wrapping_add((*gcp).data.width as u_int);
                }
            }
        }
        (*tty).flags = (*tty).flags & !TTY_NOCURSOR | flags;
        tty_update_mode(tty, (*tty).mode, s);
    }
}
