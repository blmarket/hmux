use super::driver::tty_client;
use super::driver::{tty_invalidate, tty_set_size, tty_update_features};
use crate::ffi::{__b64_pton, sscanf, strtol, strtoul};
use crate::fmt_args;

use crate::log::{log_debug, log_get_level};
use crate::notify::notify_client;
use crate::options::{OptionsEngine, RustOptionsEngine};
use crate::pane_control_colours::{PaneControlColourPair, PaneControlColours};

use crate::paste::{PasteBufferStore, paste_buffer_limit, with_paste_buffers_mut};
use crate::reactor::Timer;
use crate::server::{server_client_handle_key, server_client_update_focus};

use crate::style::{ColourEngine, RustColourEngine};
use crate::terminfo::{RustTerminalFeatureSet, TerminalFeatureSet};
use crate::terminfo::{TerminalCapabilities, tty_term_of};
use crate::text::{KeyStringCodec, RustKeyStringCodec};
use crate::text::{utf8_append, utf8_from_data, utf8_fromwc, utf8_open};
use crate::tmux::global_options;
pub use crate::types::*;
#[repr(C)]
pub struct tty_key {
    pub ch: core::ffi::c_char,
    pub key: key_code,
    pub left: Option<Box<tty_key>>,
    pub right: Option<Box<tty_key>>,
    pub next: Option<Box<tty_key>>,
}
pub use crate::consts::C0_CR;
pub use crate::consts::C0_ESC;
pub use crate::consts::C0_HT;
pub use crate::consts::{
    C0_NUL, CLIENT_FOCUSED, INPUT_REQUEST_CLIPBOARD, INPUT_REQUEST_PALETTE, KEYC_BSPACE, KEYC_BTAB,
    KEYC_CTRL, KEYC_CURSOR, KEYC_DC, KEYC_DOWN, KEYC_END, KEYC_F1, KEYC_F2, KEYC_F3, KEYC_F4,
    KEYC_F5, KEYC_F6, KEYC_F7, KEYC_F8, KEYC_F9, KEYC_F10, KEYC_F11, KEYC_F12, KEYC_FOCUS_IN,
    KEYC_FOCUS_OUT, KEYC_HOME, KEYC_IC, KEYC_IMPLIED_META, KEYC_KEYPAD, KEYC_KP_EIGHT,
    KEYC_KP_ENTER, KEYC_KP_FIVE, KEYC_KP_FOUR, KEYC_KP_MINUS, KEYC_KP_NINE, KEYC_KP_ONE,
    KEYC_KP_PERIOD, KEYC_KP_PLUS, KEYC_KP_SEVEN, KEYC_KP_SIX, KEYC_KP_SLASH, KEYC_KP_STAR,
    KEYC_KP_THREE, KEYC_KP_TWO, KEYC_KP_ZERO, KEYC_LEFT, KEYC_MASK_KEY, KEYC_MASK_MODIFIERS,
    KEYC_MASK_TYPE, KEYC_META, KEYC_MOUSE, KEYC_NPAGE, KEYC_PASTE_END, KEYC_PASTE_START,
    KEYC_PPAGE, KEYC_REPORT_DARK_THEME, KEYC_REPORT_LIGHT_THEME, KEYC_RIGHT, KEYC_SHIFT,
    KEYC_TYPE_UNICODE, KEYC_UNKNOWN, KEYC_UP, KEYC_USER, MOUSE_MASK_BUTTONS, MOUSE_PARAM_BTN_OFF,
    MOUSE_PARAM_POS_OFF, MOUSE_WHEEL_DOWN, MOUSE_WHEEL_UP, TTY_ALL_REQUEST_FLAGS, TTY_HAVEDA,
    TTY_HAVEDA2, TTY_HAVEXDA, TTY_OSC52QUERY, TTY_TIMER, TTY_WAITBG, TTY_WAITFG, TTY_WINSIZEQUERY,
    UTF8_DONE, UTF8_MORE, VERASE,
};

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

pub const _POSIX_VDISABLE: core::ffi::c_int = '\0' as i32;

pub const TTY_BRACKETPASTE: core::ffi::c_int = 0x8000 as core::ffi::c_int;

static tty_default_raw_keys: [tty_default_key_raw; 102] = [
    tty_default_key_raw {
        string: c"\x1BO[",
        key: '\u{1b}' as i32 as key_code,
    },
    tty_default_key_raw {
        string: c"\x1BOo",
        key: KEYC_KP_SLASH as core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOj",
        key: KEYC_KP_STAR as core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOm",
        key: KEYC_KP_MINUS as core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOw",
        key: KEYC_KP_SEVEN as core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOx",
        key: KEYC_KP_EIGHT as core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOy",
        key: KEYC_KP_NINE as core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOk",
        key: KEYC_KP_PLUS as core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOt",
        key: KEYC_KP_FOUR as core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOu",
        key: KEYC_KP_FIVE as core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOv",
        key: KEYC_KP_SIX as core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOq",
        key: KEYC_KP_ONE as core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOr",
        key: KEYC_KP_TWO as core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOs",
        key: KEYC_KP_THREE as core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOM",
        key: KEYC_KP_ENTER as core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOp",
        key: KEYC_KP_ZERO as core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOn",
        key: KEYC_KP_PERIOD as core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOA",
        key: KEYC_UP as core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: c"\x1BOB",
        key: KEYC_DOWN as core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: c"\x1BOC",
        key: KEYC_RIGHT as core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: c"\x1BOD",
        key: KEYC_LEFT as core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: c"\x1B[A",
        key: KEYC_UP as core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: c"\x1B[B",
        key: KEYC_DOWN as core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: c"\x1B[C",
        key: KEYC_RIGHT as core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: c"\x1B[D",
        key: KEYC_LEFT as core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: c"\x1B\x1BOA",
        key: KEYC_UP as core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: c"\x1B\x1BOB",
        key: KEYC_DOWN as core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: c"\x1B\x1BOC",
        key: KEYC_RIGHT as core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: c"\x1B\x1BOD",
        key: KEYC_LEFT as core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: c"\x1B\x1B[A",
        key: KEYC_UP as core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: c"\x1B\x1B[B",
        key: KEYC_DOWN as core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: c"\x1B\x1B[C",
        key: KEYC_RIGHT as core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: c"\x1B\x1B[D",
        key: KEYC_LEFT as core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: c"\x1BOH",
        key: KEYC_HOME as core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1BOF",
        key: KEYC_END as core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B\x1BOH",
        key: KEYC_HOME as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_raw {
        string: c"\x1B\x1BOF",
        key: KEYC_END as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_raw {
        string: c"\x1B[H",
        key: KEYC_HOME as core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[F",
        key: KEYC_END as core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B\x1B[H",
        key: KEYC_HOME as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_raw {
        string: c"\x1B\x1B[F",
        key: KEYC_END as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_raw {
        string: c"\x1BOa",
        key: KEYC_UP as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1BOb",
        key: KEYC_DOWN as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1BOc",
        key: KEYC_RIGHT as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1BOd",
        key: KEYC_LEFT as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[a",
        key: KEYC_UP as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[b",
        key: KEYC_DOWN as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[c",
        key: KEYC_RIGHT as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[d",
        key: KEYC_LEFT as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[11~",
        key: KEYC_F1 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[12~",
        key: KEYC_F2 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[13~",
        key: KEYC_F3 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[14~",
        key: KEYC_F4 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[15~",
        key: KEYC_F5 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[17~",
        key: KEYC_F6 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[18~",
        key: KEYC_F7 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[19~",
        key: KEYC_F8 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[20~",
        key: KEYC_F9 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[21~",
        key: KEYC_F10 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[23~",
        key: KEYC_F1 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[24~",
        key: KEYC_F2 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[25~",
        key: KEYC_F3 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[26~",
        key: KEYC_F4 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[28~",
        key: KEYC_F5 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[29~",
        key: KEYC_F6 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[31~",
        key: KEYC_F7 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[32~",
        key: KEYC_F8 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[33~",
        key: KEYC_F9 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[34~",
        key: KEYC_F10 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[23$",
        key: KEYC_F11 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[24$",
        key: KEYC_F12 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[11^",
        key: KEYC_F1 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[12^",
        key: KEYC_F2 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[13^",
        key: KEYC_F3 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[14^",
        key: KEYC_F4 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[15^",
        key: KEYC_F5 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[17^",
        key: KEYC_F6 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[18^",
        key: KEYC_F7 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[19^",
        key: KEYC_F8 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[20^",
        key: KEYC_F9 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[21^",
        key: KEYC_F10 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[23^",
        key: KEYC_F11 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[24^",
        key: KEYC_F12 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[11@",
        key: KEYC_F1 as core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[12@",
        key: KEYC_F2 as core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[13@",
        key: KEYC_F3 as core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[14@",
        key: KEYC_F4 as core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[15@",
        key: KEYC_F5 as core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[17@",
        key: KEYC_F6 as core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[18@",
        key: KEYC_F7 as core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[19@",
        key: KEYC_F8 as core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[20@",
        key: KEYC_F9 as core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[21@",
        key: KEYC_F10 as core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[23@",
        key: KEYC_F11 as core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[24@",
        key: KEYC_F12 as core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[I",
        key: KEYC_FOCUS_IN as core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[O",
        key: KEYC_FOCUS_OUT as core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[200~",
        key: KEYC_PASTE_START as core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    tty_default_key_raw {
        string: c"\x1B[201~",
        key: KEYC_PASTE_END as core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    tty_default_key_raw {
        string: c"\x1B[1;5Z",
        key: '\t' as i32 as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[?997;1n",
        key: KEYC_REPORT_DARK_THEME as core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[?997;2n",
        key: KEYC_REPORT_LIGHT_THEME as core::ffi::c_ulong as key_code,
    },
];
static tty_default_xterm_keys: [tty_default_key_xterm; 30] = [
    tty_default_key_xterm {
        template: c"\x1B[1;_P",
        key: KEYC_F1 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1BO1;_P",
        key: KEYC_F1 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1BO_P",
        key: KEYC_F1 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[1;_Q",
        key: KEYC_F2 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1BO1;_Q",
        key: KEYC_F2 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1BO_Q",
        key: KEYC_F2 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[1;_R",
        key: KEYC_F3 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1BO1;_R",
        key: KEYC_F3 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1BO_R",
        key: KEYC_F3 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[1;_S",
        key: KEYC_F4 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1BO1;_S",
        key: KEYC_F4 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1BO_S",
        key: KEYC_F4 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[15;_~",
        key: KEYC_F5 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[17;_~",
        key: KEYC_F6 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[18;_~",
        key: KEYC_F7 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[19;_~",
        key: KEYC_F8 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[20;_~",
        key: KEYC_F9 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[21;_~",
        key: KEYC_F10 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[23;_~",
        key: KEYC_F11 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[24;_~",
        key: KEYC_F12 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[1;_A",
        key: KEYC_UP as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[1;_B",
        key: KEYC_DOWN as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[1;_C",
        key: KEYC_RIGHT as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[1;_D",
        key: KEYC_LEFT as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[1;_H",
        key: KEYC_HOME as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[1;_F",
        key: KEYC_END as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[5;_~",
        key: KEYC_PPAGE as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[6;_~",
        key: KEYC_NPAGE as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[2;_~",
        key: KEYC_IC as core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[3;_~",
        key: KEYC_DC as core::ffi::c_ulong as key_code,
    },
];
static tty_default_xterm_modifiers: [key_code; 10] = [
    0 as core::ffi::c_int as key_code,
    0 as core::ffi::c_int as key_code,
    KEYC_SHIFT,
    KEYC_META | KEYC_IMPLIED_META,
    KEYC_SHIFT | KEYC_META | KEYC_IMPLIED_META,
    KEYC_CTRL,
    KEYC_SHIFT | KEYC_CTRL,
    KEYC_META | KEYC_IMPLIED_META | KEYC_CTRL,
    KEYC_SHIFT | KEYC_META | KEYC_IMPLIED_META | KEYC_CTRL,
    KEYC_META | KEYC_IMPLIED_META,
];
static tty_default_code_keys: [tty_default_key_code; 136] = [
    tty_default_key_code {
        code: TTYC_KF1,
        key: KEYC_F1 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF2,
        key: KEYC_F2 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF3,
        key: KEYC_F3 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF4,
        key: KEYC_F4 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF5,
        key: KEYC_F5 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF6,
        key: KEYC_F6 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF7,
        key: KEYC_F7 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF8,
        key: KEYC_F8 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF9,
        key: KEYC_F9 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF10,
        key: KEYC_F10 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF11,
        key: KEYC_F11 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF12,
        key: KEYC_F12 as core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF13,
        key: KEYC_F1 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF14,
        key: KEYC_F2 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF15,
        key: KEYC_F3 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF16,
        key: KEYC_F4 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF17,
        key: KEYC_F5 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF18,
        key: KEYC_F6 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF19,
        key: KEYC_F7 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF20,
        key: KEYC_F8 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF21,
        key: KEYC_F9 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF22,
        key: KEYC_F10 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF23,
        key: KEYC_F11 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF24,
        key: KEYC_F12 as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF25,
        key: KEYC_F1 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF26,
        key: KEYC_F2 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF27,
        key: KEYC_F3 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF28,
        key: KEYC_F4 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF29,
        key: KEYC_F5 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF30,
        key: KEYC_F6 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF31,
        key: KEYC_F7 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF32,
        key: KEYC_F8 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF33,
        key: KEYC_F9 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF34,
        key: KEYC_F10 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF35,
        key: KEYC_F11 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF36,
        key: KEYC_F12 as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF37,
        key: KEYC_F1 as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF38,
        key: KEYC_F2 as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF39,
        key: KEYC_F3 as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF40,
        key: KEYC_F4 as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF41,
        key: KEYC_F5 as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF42,
        key: KEYC_F6 as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF43,
        key: KEYC_F7 as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF44,
        key: KEYC_F8 as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF45,
        key: KEYC_F9 as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF46,
        key: KEYC_F10 as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF47,
        key: KEYC_F11 as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF48,
        key: KEYC_F12 as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF49,
        key: KEYC_F1 as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF50,
        key: KEYC_F2 as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF51,
        key: KEYC_F3 as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF52,
        key: KEYC_F4 as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF53,
        key: KEYC_F5 as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF54,
        key: KEYC_F6 as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF55,
        key: KEYC_F7 as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF56,
        key: KEYC_F8 as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF57,
        key: KEYC_F9 as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF58,
        key: KEYC_F10 as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF59,
        key: KEYC_F11 as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF60,
        key: KEYC_F12 as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF61,
        key: KEYC_F1 as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF62,
        key: KEYC_F2 as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF63,
        key: KEYC_F3 as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KICH1,
        key: KEYC_IC as core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KDCH1,
        key: KEYC_DC as core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KHOME,
        key: KEYC_HOME as core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KEND,
        key: KEYC_END as core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KNP,
        key: KEYC_NPAGE as core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KPP,
        key: KEYC_PPAGE as core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KCBT,
        key: KEYC_BTAB as core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KCUU1,
        key: KEYC_UP as core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_code {
        code: TTYC_KCUD1,
        key: KEYC_DOWN as core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_code {
        code: TTYC_KCUB1,
        key: KEYC_LEFT as core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_code {
        code: TTYC_KCUF1,
        key: KEYC_RIGHT as core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_code {
        code: TTYC_KDC2,
        key: KEYC_DC as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KDC3,
        key: KEYC_DC as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KDC4,
        key: KEYC_DC as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KDC5,
        key: KEYC_DC as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KDC6,
        key: KEYC_DC as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KDC7,
        key: KEYC_DC as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KIND,
        key: KEYC_DOWN as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KDN2,
        key: KEYC_DOWN as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KDN3,
        key: KEYC_DOWN as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KDN4,
        key: KEYC_DOWN as core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KDN5,
        key: KEYC_DOWN as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KDN6,
        key: KEYC_DOWN as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KDN7,
        key: KEYC_DOWN as core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KEND2,
        key: KEYC_END as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KEND3,
        key: KEYC_END as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KEND4,
        key: KEYC_END as core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KEND5,
        key: KEYC_END as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KEND6,
        key: KEYC_END as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KEND7,
        key: KEYC_END as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KHOM2,
        key: KEYC_HOME as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KHOM3,
        key: KEYC_HOME as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KHOM4,
        key: KEYC_HOME as core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KHOM5,
        key: KEYC_HOME as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KHOM6,
        key: KEYC_HOME as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KHOM7,
        key: KEYC_HOME as core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KIC2,
        key: KEYC_IC as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KIC3,
        key: KEYC_IC as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KIC4,
        key: KEYC_IC as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KIC5,
        key: KEYC_IC as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KIC6,
        key: KEYC_IC as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KIC7,
        key: KEYC_IC as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KLFT2,
        key: KEYC_LEFT as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KLFT3,
        key: KEYC_LEFT as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KLFT4,
        key: KEYC_LEFT as core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KLFT5,
        key: KEYC_LEFT as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KLFT6,
        key: KEYC_LEFT as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KLFT7,
        key: KEYC_LEFT as core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KNXT2,
        key: KEYC_NPAGE as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KNXT3,
        key: KEYC_NPAGE as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KNXT4,
        key: KEYC_NPAGE as core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KNXT5,
        key: KEYC_NPAGE as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KNXT6,
        key: KEYC_NPAGE as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KNXT7,
        key: KEYC_NPAGE as core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KPRV2,
        key: KEYC_PPAGE as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KPRV3,
        key: KEYC_PPAGE as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KPRV4,
        key: KEYC_PPAGE as core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KPRV5,
        key: KEYC_PPAGE as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KPRV6,
        key: KEYC_PPAGE as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KPRV7,
        key: KEYC_PPAGE as core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KRIT2,
        key: KEYC_RIGHT as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KRIT3,
        key: KEYC_RIGHT as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KRIT4,
        key: KEYC_RIGHT as core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KRIT5,
        key: KEYC_RIGHT as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KRIT6,
        key: KEYC_RIGHT as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KRIT7,
        key: KEYC_RIGHT as core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KRI,
        key: KEYC_UP as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KUP2,
        key: KEYC_UP as core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KUP3,
        key: KEYC_UP as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KUP4,
        key: KEYC_UP as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KUP5,
        key: KEYC_UP as core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KUP6,
        key: KEYC_UP as core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KUP7,
        key: KEYC_UP as core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META | KEYC_CTRL,
    },
];
unsafe fn tty_keys_add(tty: &mut tty, s: &core::ffi::CStr, key: key_code) {
    unsafe {
        let keystr = RustKeyStringCodec.format_key(key, true);
        let bytes = s.to_bytes();
        match tty_keys_find(tty, bytes) {
            None => {
                log_debug(
                    c"new key %s: 0x%llx (%s)",
                    fmt_args![s, key, keystr.as_ptr()],
                );
                tty_keys_add1(&mut tty.key_tree, bytes, key);
            }
            Some((tk, _)) => {
                log_debug(
                    c"replacing key %s: 0x%llx (%s)",
                    fmt_args![s, key, keystr.as_ptr()],
                );
                tk.key = key;
            }
        }
    }
}
fn tty_keys_add1(tkp: &mut Option<Box<tty_key>>, bytes: &[u8], key: key_code) {
    if bytes.is_empty() {
        return;
    }
    let ch = bytes[0] as core::ffi::c_char;
    if tkp.is_none() {
        *tkp = Some(Box::new(tty_key {
            ch,
            key: KEYC_UNKNOWN as core::ffi::c_ulong as key_code,
            left: None,
            right: None,
            next: None,
        }));
    }
    let tk = tkp.as_mut().unwrap();
    if ch == tk.ch {
        if bytes.len() == 1 {
            tk.key = key;
            return;
        }
        tty_keys_add1(&mut tk.next, &bytes[1..], key);
    } else if (ch as core::ffi::c_int) < tk.ch as core::ffi::c_int {
        tty_keys_add1(&mut tk.left, bytes, key);
    } else if (ch as core::ffi::c_int) > tk.ch as core::ffi::c_int {
        tty_keys_add1(&mut tk.right, bytes, key);
    }
}
pub unsafe fn tty_keys_build(tty: &mut tty) {
    unsafe {
        let mut i: u_int = 0;

        if tty.key_tree.is_some() {
            tty_keys_free(tty);
        }
        for tdkx in &tty_default_xterm_keys {
            let mut copy = tdkx.template.to_bytes_with_nul().to_vec();
            let marker = copy
                .iter()
                .position(|byte| *byte == b'_')
                .expect("an xterm key template has a modifier placeholder");
            for (j, modifier) in tty_default_xterm_modifiers.iter().enumerate().skip(2) {
                copy[marker] = b'0'.wrapping_add(j as u8);
                let key = tdkx.key | *modifier;
                let sequence = core::ffi::CStr::from_bytes_with_nul(&copy)
                    .expect("the modifier leaves the key sequence terminated");
                tty_keys_add(tty, sequence, key);
            }
        }
        for tdkr in &tty_default_raw_keys {
            let s = tdkr.string;
            if !s.to_bytes().is_empty() {
                tty_keys_add(tty, s, tdkr.key);
            }
        }
        for tdkc in &tty_default_code_keys {
            let s = tty_term_of(tty).string(tdkc.code).to_owned();
            if !s.to_bytes().is_empty() {
                tty_keys_add(tty, &s, tdkc.key);
            }
        }
        global_options
            .get()
            .as_ref()
            .expect("global options are initialized")
            .with_entry(c"user-keys", false, |entry| {
                let Some(entry) = entry else { return };
                for array_index in RustOptionsEngine.array_indices(entry) {
                    i = array_index;
                    let ov = RustOptionsEngine.array_get(entry, array_index).unwrap();
                    tty_keys_add(
                        tty,
                        RustOptionsEngine.value_string(ov),
                        (KEYC_USER as core::ffi::c_ulong).wrapping_add(i as core::ffi::c_ulong)
                            as key_code,
                    );
                }
            });
    }
}
pub fn tty_keys_free(tty: &mut tty) {
    {
        drop(tty.key_tree.take());
    }
}
/// The key `buf` starts with, and how many of its bytes that key took.
unsafe fn tty_keys_find<'a>(tty: &'a mut tty, buf: &[u8]) -> Option<(&'a mut tty_key, size_t)> {
    unsafe {
        let mut size = 0 as size_t;
        let tk = tty_keys_find1(tty.key_tree.as_deref_mut(), buf, &mut size)?;
        Some((tk, size))
    }
}
/// The node `buf` walks down to, counting the bytes it took into `size`. The
/// bytes are compared as the C's `char` compared them, which is what the tree
/// was built with.
unsafe fn tty_keys_find1<'a>(
    tk: Option<&'a mut tty_key>,
    buf: &[u8],
    size: &mut size_t,
) -> Option<&'a mut tty_key> {
    unsafe {
        let (&first, rest) = buf.split_first()?;
        let tk = tk?;
        let first = first as core::ffi::c_char as core::ffi::c_int;
        if first == tk.ch as core::ffi::c_int {
            *size = (*size).wrapping_add(1);
            if rest.is_empty()
                || tk.next.is_none() && tk.key != KEYC_UNKNOWN as core::ffi::c_ulong as key_code
            {
                return Some(tk);
            }
            tty_keys_find1(tk.next.as_deref_mut(), rest, size)
        } else if first < tk.ch as core::ffi::c_int {
            tty_keys_find1(tk.left.as_deref_mut(), buf, size)
        } else {
            tty_keys_find1(tk.right.as_deref_mut(), buf, size)
        }
    }
}
/// Whether `buf` is a strict, non-empty prefix of the bracketed paste end
/// sequence, which is what asks the key reader to wait longer for the rest of
/// it. A whole sequence is not partial, so an equal or longer `buf` is not one.
fn tty_keys_partial_paste_end(buf: &[u8]) -> bool {
    const PASTE_END: &[u8] = b"\x1B[201~";
    if buf.is_empty() || buf.len() >= PASTE_END.len() {
        return false;
    }
    PASTE_END.starts_with(buf)
}
unsafe fn tty_keys_next1(
    tty: &mut tty,
    buf: &[u8],
    key: &mut key_code,
    size: &mut size_t,
    expired: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        let len = buf.len();
        let c = tty_client(tty);
        let mut ud = utf8_data::default();
        let mut more: utf8_state;
        let mut i: u_int;
        log_debug(
            c"%s: next key is %zu (%.*s) (expired=%d)",
            fmt_args![
                c.as_ref().expect("the tty has a client").name(),
                len,
                len as core::ffi::c_int,
                buf,
                expired
            ],
        );
        *size = 0 as size_t;
        let found = tty_keys_find(tty, buf)
            .filter(|(tk, _)| tk.key != KEYC_UNKNOWN as core::ffi::c_ulong as key_code);
        if let Some((tk, taken)) = found {
            *size = taken;
            let mut tk1 = Some(&*tk);
            while let Some(one) = tk1 {
                log_debug(
                    c"%s: keys in list: %#llx",
                    fmt_args![c.as_ref().expect("the tty has a client").name(), one.key],
                );
                tk1 = one.next.as_deref();
            }
            if tk.next.is_some() && expired == 0 {
                return 1 as core::ffi::c_int;
            }
            *key = tk.key;
            if *key & KEYC_MASK_KEY
                == KEYC_PASTE_START as core::ffi::c_ulong as core::ffi::c_ulonglong
            {
                tty.flags |= TTY_BRACKETPASTE;
            } else if *key & KEYC_MASK_KEY
                == KEYC_PASTE_END as core::ffi::c_ulong as core::ffi::c_ulonglong
            {
                tty.flags &= !TTY_BRACKETPASTE;
            }
            return 0 as core::ffi::c_int;
        }
        let Some(&first) = buf.first() else {
            return -1;
        };
        more = utf8_open(&mut ud, first);
        if more as core::ffi::c_uint == UTF8_MORE as core::ffi::c_int as core::ffi::c_uint {
            *size = ud.size as size_t;
            if len < ud.size as size_t {
                if expired == 0 {
                    return 1 as core::ffi::c_int;
                }
                return -(1 as core::ffi::c_int);
            }
            i = 1 as u_int;
            while i < ud.size as u_int {
                more = utf8_append(&mut ud, buf[i as usize]);
                i = i.wrapping_add(1);
            }
            if more as core::ffi::c_uint != UTF8_DONE as core::ffi::c_int as core::ffi::c_uint {
                return -(1 as core::ffi::c_int);
            }
            let (state, uc) = utf8_from_data(&ud);
            if state as core::ffi::c_uint != UTF8_DONE as core::ffi::c_int as core::ffi::c_uint {
                return -(1 as core::ffi::c_int);
            }
            *key = uc as key_code;
            log_debug(
                c"%s: UTF-8 key %.*s %#llx",
                fmt_args![
                    c.as_ref().expect("the tty has a client").name(),
                    ud.size as core::ffi::c_int,
                    ud.data.as_slice(),
                    *key
                ],
            );
            return 0 as core::ffi::c_int;
        }
        -(1 as core::ffi::c_int)
    }
}
unsafe fn tty_keys_winsz(tty: &mut tty, buf: &[u8], size: &mut size_t) -> core::ffi::c_int {
    unsafe {
        let len = buf.len();
        let c = tty_client(tty);
        let mut end: size_t;
        let mut tmp = [0u8; 64];
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        let mut xpixel: u_int = 0;
        let mut ypixel: u_int = 0;
        let char_x: u_int;
        let char_y: u_int;
        *size = 0 as size_t;
        if tty.flags & TTY_WINSIZEQUERY == 0 {
            return -(1 as core::ffi::c_int);
        }
        if buf.first() != Some(&0x1b) {
            return -(1 as core::ffi::c_int);
        }
        if len == 1 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[1] as core::ffi::c_int != '[' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 2 as size_t {
            return 1 as core::ffi::c_int;
        }
        end = 2 as size_t;
        while end < len && end != tmp.len() {
            if buf[end] as core::ffi::c_int == 't' as i32 {
                break;
            }
            if libc::isdigit((buf[end] as u_char).into()) == 0
                && buf[end] as core::ffi::c_int != ';' as i32
            {
                break;
            }
            end = end.wrapping_add(1);
        }
        if end == len {
            return 1 as core::ffi::c_int;
        }
        if end == tmp.len() || buf[end] as core::ffi::c_int != 't' as i32 {
            return -(1 as core::ffi::c_int);
        }
        tmp[..end - 2].copy_from_slice(&buf[2..end]);
        let sequence = core::ffi::CStr::from_bytes_with_nul(&tmp[..end - 1])
            .expect("the numeric reply has no embedded NUL");
        if sscanf(
            sequence.as_ptr(),
            c"8;%u;%u".as_ptr(),
            &raw mut sy,
            &raw mut sx,
        ) == 2 as core::ffi::c_int
        {
            tty_set_size(tty, sx, sy, tty.xpixel, tty.ypixel);
            *size = end.wrapping_add(1 as size_t);
            return 0 as core::ffi::c_int;
        } else if sscanf(
            sequence.as_ptr(),
            c"4;%u;%u".as_ptr(),
            &raw mut ypixel,
            &raw mut xpixel,
        ) == 2 as core::ffi::c_int
        {
            char_x = if xpixel != 0 && tty.sx != 0 {
                xpixel.wrapping_div(tty.sx)
            } else {
                0 as u_int
            };
            char_y = if ypixel != 0 && tty.sy != 0 {
                ypixel.wrapping_div(tty.sy)
            } else {
                0 as u_int
            };
            tty_set_size(tty, tty.sx, tty.sy, char_x, char_y);
            tty_invalidate(tty);
            tty.flags &= !TTY_WINSIZEQUERY;
            *size = end.wrapping_add(1 as size_t);
            return 0 as core::ffi::c_int;
        }
        log_debug(
            c"%s: unrecognized window size sequence: %s",
            fmt_args![c.as_ref().expect("the tty has a client").name(), sequence],
        );
        -(1 as core::ffi::c_int)
    }
}

unsafe fn tty_keys_extended_key(
    tty: &mut tty,
    buf: &[u8],
    size: &mut size_t,
    key: &mut key_code,
) -> core::ffi::c_int {
    unsafe {
        let len = buf.len();
        let c = tty_client(tty);
        let mut end: size_t;
        let mut number: u_int = 0;
        let mut modifiers: u_int = 0;
        let mut tmp = [0u8; 64];

        let mut nkey: key_code;

        let mut ud = utf8_data::default();
        *size = 0 as size_t;
        if buf.first() != Some(&0x1b) {
            return -(1 as core::ffi::c_int);
        }
        if len == 1 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[1] as core::ffi::c_int != '[' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 2 as size_t {
            return 1 as core::ffi::c_int;
        }
        end = 2 as size_t;
        while end < len && end != tmp.len() {
            if buf[end] as core::ffi::c_int == '~' as i32 {
                break;
            }
            if libc::isdigit((buf[end] as u_char).into()) == 0
                && buf[end] as core::ffi::c_int != ';' as i32
            {
                break;
            }
            end = end.wrapping_add(1);
        }
        if end == len {
            return 1 as core::ffi::c_int;
        }
        if end == tmp.len()
            || buf[end] as core::ffi::c_int != '~' as i32
                && buf[end] as core::ffi::c_int != 'u' as i32
        {
            return -(1 as core::ffi::c_int);
        }
        tmp[..end - 2].copy_from_slice(&buf[2..end]);
        let sequence = core::ffi::CStr::from_bytes_with_nul(&tmp[..end - 1])
            .expect("the numeric key sequence has no embedded NUL");
        if buf[end] as core::ffi::c_int == '~' as i32 {
            if sscanf(
                sequence.as_ptr(),
                c"27;%u;%u".as_ptr(),
                &raw mut modifiers,
                &raw mut number,
            ) != 2 as core::ffi::c_int
            {
                return -(1 as core::ffi::c_int);
            }
        } else if sscanf(
            sequence.as_ptr(),
            c"%u;%u".as_ptr(),
            &raw mut number,
            &raw mut modifiers,
        ) != 2 as core::ffi::c_int
        {
            return -(1 as core::ffi::c_int);
        }
        *size = end.wrapping_add(1 as size_t);
        let bspace: cc_t = tty.tio.c_cc[VERASE as usize];
        if bspace as core::ffi::c_int != _POSIX_VDISABLE && number == bspace as u_int {
            nkey = KEYC_BSPACE as core::ffi::c_ulong as key_code;
        } else {
            nkey = number as key_code;
        }
        if nkey != KEYC_BSPACE as core::ffi::c_ulong as key_code
            && nkey & !(0x7f as core::ffi::c_int) as key_code != 0
        {
            if utf8_fromwc(nkey as wchar_t, &mut ud) as core::ffi::c_uint
                == UTF8_DONE as core::ffi::c_int as core::ffi::c_uint
                && let (UTF8_DONE, uc) = utf8_from_data(&ud)
            {
                nkey = uc as key_code;
            } else {
                return -(1 as core::ffi::c_int);
            }
        }
        if modifiers > 0 as u_int {
            modifiers = modifiers.wrapping_sub(1);
            if modifiers & 1 as u_int != 0 {
                nkey |= KEYC_SHIFT;
            }
            if modifiers & 2 as u_int != 0 {
                nkey |= KEYC_META | KEYC_IMPLIED_META;
            }
            if modifiers & 4 as u_int != 0 {
                nkey |= KEYC_CTRL;
            }
            if modifiers & 8 as u_int != 0 {
                nkey |= KEYC_META | KEYC_IMPLIED_META;
            }
        }
        if nkey as core::ffi::c_ulonglong & KEYC_MASK_KEY == '\t' as i32 as core::ffi::c_ulonglong
            && nkey as core::ffi::c_ulonglong & KEYC_SHIFT != 0
        {
            nkey = (KEYC_BTAB as core::ffi::c_ulong as core::ffi::c_ulonglong
                | nkey as core::ffi::c_ulonglong & !KEYC_MASK_KEY & !KEYC_SHIFT)
                as key_code;
        }
        let onlykey: key_code = (nkey as core::ffi::c_ulonglong & KEYC_MASK_KEY) as key_code;
        if (onlykey > 0x20 as key_code && onlykey < 0x7f as key_code
            || nkey as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                == (KEYC_TYPE_UNICODE as core::ffi::c_int as core::ffi::c_ulonglong)
                    << 32 as core::ffi::c_int
                && nkey as core::ffi::c_ulonglong & KEYC_MASK_KEY > 0x7f as core::ffi::c_ulonglong)
            && nkey as core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS == KEYC_SHIFT
        {
            nkey &= !KEYC_SHIFT;
        }
        if log_get_level() != 0 as core::ffi::c_int {
            let key_name = RustKeyStringCodec.format_key(nkey, true);
            log_debug(
                c"%s: extended key %.*s is %llx (%s)",
                fmt_args![
                    c.as_ref().expect("the tty has a client").name(),
                    *size as core::ffi::c_int,
                    buf,
                    nkey,
                    key_name.as_ptr()
                ],
            );
        }
        *key = nkey;
        0 as core::ffi::c_int
    }
}
unsafe fn tty_keys_mouse(
    tty: &mut tty,
    buf: &[u8],
    size: &mut size_t,
    m: &mut mouse_event,
) -> core::ffi::c_int {
    unsafe {
        let len = buf.len();
        let c = tty_client(tty);
        let mut i: u_int;
        let mut x: u_int;
        let mut y: u_int;
        let mut b: u_int;
        let mut sgr_b: u_int;
        let mut sgr_type: u_char;
        let mut ch: u_char;
        *size = 0 as size_t;
        sgr_b = 0 as u_int;
        b = sgr_b;
        y = b;
        x = y;
        sgr_type = ' ' as i32 as u_char;
        if buf.first() != Some(&0x1b) {
            return -(1 as core::ffi::c_int);
        }
        if len == 1 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[1] as core::ffi::c_int != '[' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 2 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[2] as core::ffi::c_int == 'M' as i32 {
            *size = 3 as size_t;
            i = 0 as u_int;
            while i < 3 as u_int {
                if len <= *size {
                    return 1 as core::ffi::c_int;
                }
                let fresh0 = *size;
                *size = (*size).wrapping_add(1);
                ch = buf[fresh0];
                if i == 0 as u_int {
                    b = ch as u_int;
                } else if i == 1 as u_int {
                    x = ch as u_int;
                } else {
                    y = ch as u_int;
                }
                i = i.wrapping_add(1);
            }
            log_debug(
                c"%s: mouse input: %.*s",
                fmt_args![
                    c.as_ref().expect("the tty has a client").name(),
                    *size as core::ffi::c_int,
                    buf
                ],
            );
            if b < MOUSE_PARAM_BTN_OFF as u_int
                || x < MOUSE_PARAM_POS_OFF as u_int
                || y < MOUSE_PARAM_POS_OFF as u_int
            {
                return -(2 as core::ffi::c_int);
            }
            b = b.wrapping_sub(MOUSE_PARAM_BTN_OFF as u_int);
            x = x.wrapping_sub(MOUSE_PARAM_POS_OFF as u_int);
            y = y.wrapping_sub(MOUSE_PARAM_POS_OFF as u_int);
        } else if buf[2] as core::ffi::c_int == '<' as i32 {
            *size = 3 as size_t;
            loop {
                if len <= *size {
                    return 1 as core::ffi::c_int;
                }
                let fresh1 = *size;
                *size = (*size).wrapping_add(1);
                ch = buf[fresh1];
                if ch as core::ffi::c_int == ';' as i32 {
                    break;
                }
                if (ch as core::ffi::c_int) < '0' as i32 || ch as core::ffi::c_int > '9' as i32 {
                    return -(1 as core::ffi::c_int);
                }
                sgr_b = (10 as u_int)
                    .wrapping_mul(sgr_b)
                    .wrapping_add((ch as core::ffi::c_int - '0' as i32) as u_int);
            }
            loop {
                if len <= *size {
                    return 1 as core::ffi::c_int;
                }
                let fresh2 = *size;
                *size = (*size).wrapping_add(1);
                ch = buf[fresh2];
                if ch as core::ffi::c_int == ';' as i32 {
                    break;
                }
                if (ch as core::ffi::c_int) < '0' as i32 || ch as core::ffi::c_int > '9' as i32 {
                    return -(1 as core::ffi::c_int);
                }
                x = (10 as u_int)
                    .wrapping_mul(x)
                    .wrapping_add((ch as core::ffi::c_int - '0' as i32) as u_int);
            }
            loop {
                if len <= *size {
                    return 1 as core::ffi::c_int;
                }
                let fresh3 = *size;
                *size = (*size).wrapping_add(1);
                ch = buf[fresh3];
                if ch as core::ffi::c_int == 'M' as i32 || ch as core::ffi::c_int == 'm' as i32 {
                    break;
                }
                if (ch as core::ffi::c_int) < '0' as i32 || ch as core::ffi::c_int > '9' as i32 {
                    return -(1 as core::ffi::c_int);
                }
                y = (10 as u_int)
                    .wrapping_mul(y)
                    .wrapping_add((ch as core::ffi::c_int - '0' as i32) as u_int);
            }
            log_debug(
                c"%s: mouse input (SGR): %.*s",
                fmt_args![
                    c.as_ref().expect("the tty has a client").name(),
                    *size as core::ffi::c_int,
                    buf
                ],
            );
            if x < 1 as u_int || y < 1 as u_int {
                return -(2 as core::ffi::c_int);
            }
            x = x.wrapping_sub(1);
            y = y.wrapping_sub(1);
            b = sgr_b;
            sgr_type = ch;
            if sgr_type as core::ffi::c_int == 'm' as i32 {
                b = 3 as u_int;
            }
            if sgr_type as core::ffi::c_int == 'm' as i32
                && (sgr_b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int
                    || sgr_b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_DOWN as u_int)
            {
                return -(2 as core::ffi::c_int);
            }
        } else {
            return -(1 as core::ffi::c_int);
        }
        m.lx = tty.mouse_last_x;
        m.x = x;
        m.ly = tty.mouse_last_y;
        m.y = y;
        m.lb = tty.mouse_last_b;
        m.b = b;
        m.sgr_type = sgr_type as u_int;
        m.sgr_b = sgr_b;
        tty.mouse_last_x = x;
        tty.mouse_last_y = y;
        tty.mouse_last_b = b;
        0 as core::ffi::c_int
    }
}
unsafe fn tty_keys_clipboard(tty: &mut tty, buf: &[u8], size: &mut size_t) -> core::ffi::c_int {
    unsafe {
        let len = buf.len();
        let mut c = tty_client(tty);
        let mut end: size_t;
        let mut terminator: size_t = 0 as size_t;

        let mut clip: core::ffi::c_char = 0 as core::ffi::c_char;

        *size = 0 as size_t;
        if buf.first() != Some(&0x1b) {
            return -(1 as core::ffi::c_int);
        }
        if len == 1 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[1] as core::ffi::c_int != ']' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 2 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[2] as core::ffi::c_int != '5' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 3 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[3] as core::ffi::c_int != '2' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 4 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[4] as core::ffi::c_int != ';' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 5 as size_t {
            return 1 as core::ffi::c_int;
        }
        end = 5 as size_t;
        while end < len {
            if buf[end] as core::ffi::c_int == '\u{7}' as i32 {
                terminator = 1 as size_t;
                break;
            } else if end > 5 as size_t
                && buf[end - 1] as core::ffi::c_int == '\u{1b}' as i32
                && buf[end] as core::ffi::c_int == '\\' as i32
            {
                terminator = 2 as size_t;
                break;
            } else {
                end = end.wrapping_add(1);
            }
        }
        if end == len {
            return 1 as core::ffi::c_int;
        }
        *size = end.wrapping_add(1 as size_t);
        let payload = &buf[5..end - (terminator - 1)];
        if payload.len() >= 2 && payload[0] != b';' && payload[1] == b';' {
            clip = payload[0] as core::ffi::c_char;
        }
        let Some(separator) = payload.iter().position(|byte| *byte == b';') else {
            return 0;
        };
        let encoded = &payload[separator + 1..];
        if encoded.is_empty() {
            return 0;
        }
        end = encoded.len();
        let mut copy = encoded.to_vec();
        copy.push(b'\0');
        let needed: size_t = end
            .wrapping_add(3 as size_t)
            .wrapping_div(4 as size_t)
            .wrapping_mul(3 as size_t);
        if needed == 0 as size_t {
            return 0 as core::ffi::c_int;
        }
        let mut out: Vec<u8> = vec![0_u8; needed as usize];
        let outlen: core::ffi::c_int = __b64_pton(
            copy.as_ptr() as *const core::ffi::c_char,
            out.as_mut_ptr(),
            needed,
        );
        if outlen == -(1 as core::ffi::c_int) {
            return 0 as core::ffi::c_int;
        }
        out.truncate(outlen as usize);
        log_debug(
            c"%s: %.*s",
            fmt_args![c"tty_keys_clipboard", outlen, out.as_slice()],
        );
        (c.as_mut().expect("the tty has a client")).handle_input_reply(
            INPUT_REQUEST_CLIPBOARD,
            &InputRequestData::Clipboard {
                clip,
                buf: out.clone(),
            },
        );
        if tty.flags & TTY_OSC52QUERY != 0 {
            let limit = paste_buffer_limit();
            with_paste_buffers_mut(|buffers| buffers.add_automatic(None, out, limit));
            tty.clipboard_timer.disarm();
            tty.flags &= !TTY_OSC52QUERY;
        }
        0 as core::ffi::c_int
    }
}
fn tty_keys_attribute_values(bytes: &mut [u8]) -> ([core::ffi::c_char; 32], u_int) {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .expect("the attribute reply buffer is terminated");
    for byte in &mut bytes[..end] {
        if *byte == b';' {
            *byte = 0;
        }
    }
    let mut values = [0; 32];
    let mut count = 0;
    for (index, field) in bytes[..=end]
        .split_inclusive(|byte| *byte == 0)
        .take(values.len())
        .enumerate()
    {
        let field = core::ffi::CStr::from_bytes_with_nul(field)
            .expect("each attribute field ends at its delimiter");
        let mut endptr = core::ptr::null_mut();
        unsafe {
            values[index] = strtoul(field.as_ptr(), &mut endptr, 10) as core::ffi::c_char;
            if *endptr != 0 {
                values[index] = 0;
            }
        }
        count += 1;
    }
    (values, count)
}
unsafe fn tty_keys_device_attributes(
    tty: &mut tty,
    buf: &[u8],
    size: &mut size_t,
) -> core::ffi::c_int {
    unsafe {
        let len = buf.len();
        let mut c = tty_client(tty);
        let mut i: u_int;
        let mut tmp = [0u8; 128];
        *size = 0 as size_t;
        if tty.flags & TTY_HAVEDA != 0 {
            return -(1 as core::ffi::c_int);
        }
        if buf.first() != Some(&0x1b) {
            return -(1 as core::ffi::c_int);
        }
        if len == 1 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[1] as core::ffi::c_int != '[' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 2 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[2] as core::ffi::c_int != '?' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 3 as size_t {
            return 1 as core::ffi::c_int;
        }
        i = 0 as u_int;
        while (i as usize) < tmp.len() {
            if (3 as u_int).wrapping_add(i) as size_t == len {
                return 1 as core::ffi::c_int;
            }
            if buf[3 + i as usize] as core::ffi::c_int >= 'a' as i32
                && buf[3 + i as usize] as core::ffi::c_int <= 'z' as i32
            {
                break;
            }
            tmp[i as usize] = buf[3 + i as usize];
            i = i.wrapping_add(1);
        }
        if i as usize == tmp.len() {
            return -(1 as core::ffi::c_int);
        }
        if buf[3 + i as usize] as core::ffi::c_int != 'c' as i32 {
            return -(1 as core::ffi::c_int);
        }
        tmp[i as usize] = 0;
        *size = (4 as u_int).wrapping_add(i) as size_t;
        let (p, n) = tty_keys_attribute_values(&mut tmp);
        if let 61..=65 = p[0 as core::ffi::c_int as usize] as core::ffi::c_int {
            i = 1 as u_int;
            while i < n {
                log_debug(
                    c"%s: DA feature: %d",
                    fmt_args![
                        c.as_ref().expect("the tty has a client").name(),
                        p[i as usize] as core::ffi::c_int
                    ],
                );
                if p[i as usize] as core::ffi::c_int == 4 as core::ffi::c_int {
                    *c.as_mut()
                        .expect("the tty has a client")
                        .terminal_features_mut() = RustTerminalFeatureSet.add(
                        c.as_ref()
                            .expect("the tty has a client")
                            .terminal_features(),
                        c"sixel",
                        c",",
                    );
                }
                if p[i as usize] as core::ffi::c_int == 21 as core::ffi::c_int {
                    *c.as_mut()
                        .expect("the tty has a client")
                        .terminal_features_mut() = RustTerminalFeatureSet.add(
                        c.as_ref()
                            .expect("the tty has a client")
                            .terminal_features(),
                        c"margins",
                        c",",
                    );
                }
                if p[i as usize] as core::ffi::c_int == 28 as core::ffi::c_int {
                    *c.as_mut()
                        .expect("the tty has a client")
                        .terminal_features_mut() = RustTerminalFeatureSet.add(
                        c.as_ref()
                            .expect("the tty has a client")
                            .terminal_features(),
                        c"rectfill",
                        c",",
                    );
                }
                if p[i as usize] as core::ffi::c_int == 52 as core::ffi::c_int {
                    *c.as_mut()
                        .expect("the tty has a client")
                        .terminal_features_mut() = RustTerminalFeatureSet.add(
                        c.as_ref()
                            .expect("the tty has a client")
                            .terminal_features(),
                        c"clipboard",
                        c",",
                    );
                }
                i = i.wrapping_add(1);
            }
        }
        log_debug(
            c"%s: received primary DA %.*s",
            fmt_args![
                c.as_ref().expect("the tty has a client").name(),
                *size as core::ffi::c_int,
                buf
            ],
        );
        tty_update_features(tty);
        tty.flags |= TTY_HAVEDA;
        0 as core::ffi::c_int
    }
}
unsafe fn tty_keys_device_attributes2(
    tty: &mut tty,
    buf: &[u8],
    size: &mut size_t,
) -> core::ffi::c_int {
    unsafe {
        let len = buf.len();
        let mut c = tty_client(tty);
        let mut i: u_int;
        let mut tmp = [0u8; 128];
        *size = 0 as size_t;
        if tty.flags & TTY_HAVEDA2 != 0 {
            return -(1 as core::ffi::c_int);
        }
        if buf.first() != Some(&0x1b) {
            return -(1 as core::ffi::c_int);
        }
        if len == 1 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[1] as core::ffi::c_int != '[' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 2 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[2] as core::ffi::c_int != '>' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 3 as size_t {
            return 1 as core::ffi::c_int;
        }
        i = 0 as u_int;
        while (i as usize) < tmp.len() {
            if (3 as u_int).wrapping_add(i) as size_t == len {
                return 1 as core::ffi::c_int;
            }
            if buf[3 + i as usize] as core::ffi::c_int >= 'a' as i32
                && buf[3 + i as usize] as core::ffi::c_int <= 'z' as i32
            {
                break;
            }
            tmp[i as usize] = buf[3 + i as usize];
            i = i.wrapping_add(1);
        }
        if i as usize == tmp.len() {
            return -(1 as core::ffi::c_int);
        }
        if buf[3 + i as usize] as core::ffi::c_int != 'c' as i32 {
            return -(1 as core::ffi::c_int);
        }
        tmp[i as usize] = 0;
        *size = (4 as u_int).wrapping_add(i) as size_t;
        let (p, _) = tty_keys_attribute_values(&mut tmp);
        match p[0 as core::ffi::c_int as usize] as core::ffi::c_int {
            77 => {
                *c.as_mut()
                    .expect("the tty has a client")
                    .terminal_features_mut() = RustTerminalFeatureSet.defaults(
                    c.as_ref()
                        .expect("the tty has a client")
                        .terminal_features(),
                    c"mintty",
                    0 as u_int,
                );
            }
            84 => {
                *c.as_mut()
                    .expect("the tty has a client")
                    .terminal_features_mut() = RustTerminalFeatureSet.defaults(
                    c.as_ref()
                        .expect("the tty has a client")
                        .terminal_features(),
                    c"tmux",
                    0 as u_int,
                );
            }
            85 => {
                *c.as_mut()
                    .expect("the tty has a client")
                    .terminal_features_mut() = RustTerminalFeatureSet.defaults(
                    c.as_ref()
                        .expect("the tty has a client")
                        .terminal_features(),
                    c"rxvt-unicode",
                    0 as u_int,
                );
            }
            _ => {}
        }
        log_debug(
            c"%s: received secondary DA %.*s",
            fmt_args![
                c.as_ref().expect("the tty has a client").name(),
                *size as core::ffi::c_int,
                buf
            ],
        );
        tty_update_features(tty);
        tty.flags |= TTY_HAVEDA2;
        0 as core::ffi::c_int
    }
}
unsafe fn tty_keys_extended_device_attributes(
    tty: &mut tty,
    buf: &[u8],
    size: &mut size_t,
) -> core::ffi::c_int {
    unsafe {
        let len = buf.len();
        let mut c = tty_client(tty);
        let mut i: u_int;
        let mut tmp = [0u8; 128];
        *size = 0 as size_t;
        if tty.flags & TTY_HAVEXDA != 0 {
            return -(1 as core::ffi::c_int);
        }
        if buf.first() != Some(&0x1b) {
            return -(1 as core::ffi::c_int);
        }
        if len == 1 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[1] as core::ffi::c_int != 'P' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 2 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[2] as core::ffi::c_int != '>' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 3 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[3] as core::ffi::c_int != '|' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 4 as size_t {
            return 1 as core::ffi::c_int;
        }
        i = 0 as u_int;
        while (i as usize) < tmp.len() - 1 {
            if (4 as u_int).wrapping_add(i) as size_t == len {
                return 1 as core::ffi::c_int;
            }
            if buf[3 + i as usize] as core::ffi::c_int == '\u{1b}' as i32
                && buf[4 + i as usize] as core::ffi::c_int == '\\' as i32
            {
                break;
            }
            tmp[i as usize] = buf[4 + i as usize];
            i = i.wrapping_add(1);
        }
        if i as usize == tmp.len() - 1 {
            return -(1 as core::ffi::c_int);
        }
        *size = (5 as u_int).wrapping_add(i) as size_t;
        if i == 0 as u_int {
            return 0 as core::ffi::c_int;
        }
        tmp[i.wrapping_sub(1 as u_int) as usize] = 0;
        let name = core::ffi::CStr::from_bytes_until_nul(&tmp)
            .expect("the terminal identification buffer is terminated");
        if name.to_bytes().starts_with(b"iTerm2 ") {
            *c.as_mut()
                .expect("the tty has a client")
                .terminal_features_mut() = RustTerminalFeatureSet.defaults(
                c.as_ref()
                    .expect("the tty has a client")
                    .terminal_features(),
                c"iTerm2",
                0 as u_int,
            );
        } else if name.to_bytes().starts_with(b"tmux ") {
            *c.as_mut()
                .expect("the tty has a client")
                .terminal_features_mut() = RustTerminalFeatureSet.defaults(
                c.as_ref()
                    .expect("the tty has a client")
                    .terminal_features(),
                c"tmux",
                0 as u_int,
            );
        } else if name.to_bytes().starts_with(b"XTerm(") {
            *c.as_mut()
                .expect("the tty has a client")
                .terminal_features_mut() = RustTerminalFeatureSet.defaults(
                c.as_ref()
                    .expect("the tty has a client")
                    .terminal_features(),
                c"XTerm",
                0 as u_int,
            );
        } else if name.to_bytes().starts_with(b"mintty ") {
            *c.as_mut()
                .expect("the tty has a client")
                .terminal_features_mut() = RustTerminalFeatureSet.defaults(
                c.as_ref()
                    .expect("the tty has a client")
                    .terminal_features(),
                c"mintty",
                0 as u_int,
            );
        } else if name.to_bytes().starts_with(b"foot(") {
            *c.as_mut()
                .expect("the tty has a client")
                .terminal_features_mut() = RustTerminalFeatureSet.defaults(
                c.as_ref()
                    .expect("the tty has a client")
                    .terminal_features(),
                c"foot",
                0 as u_int,
            );
        } else if name.to_bytes().starts_with(b"WezTerm") {
            *c.as_mut()
                .expect("the tty has a client")
                .terminal_features_mut() = RustTerminalFeatureSet.defaults(
                c.as_ref()
                    .expect("the tty has a client")
                    .terminal_features(),
                c"WezTerm",
                0 as u_int,
            );
        }
        log_debug(
            c"%s: received extended DA %.*s",
            fmt_args![
                c.as_ref().expect("the tty has a client").name(),
                *size as core::ffi::c_int,
                buf
            ],
        );
        *c.as_mut()
            .expect("the tty has a client")
            .terminal_type_mut() = Some(name.to_owned());
        tty_update_features(tty);
        tty.flags |= TTY_HAVEXDA;
        0 as core::ffi::c_int
    }
}
pub unsafe fn tty_keys_colours(
    tty: &mut tty,
    buf: &[u8],
    size: &mut size_t,
    fg: &mut core::ffi::c_int,
    bg: &mut core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        let len = buf.len();
        let c = tty_client(tty);
        let mut i: u_int;
        let mut tmp = [0u8; 128];

        *size = 0 as size_t;
        if buf.first() != Some(&0x1b) {
            return -(1 as core::ffi::c_int);
        }
        if len == 1 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[1] as core::ffi::c_int != ']' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 2 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[2] as core::ffi::c_int != '1' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 3 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[3] as core::ffi::c_int != '0' as i32 && buf[3] as core::ffi::c_int != '1' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 4 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[4] as core::ffi::c_int != ';' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 5 as size_t {
            return 1 as core::ffi::c_int;
        }
        i = 0 as u_int;
        while (i as usize) < tmp.len() - 1 {
            if (5 as u_int).wrapping_add(i) as size_t == len {
                return 1 as core::ffi::c_int;
            }
            if buf[4 + i as usize] as core::ffi::c_int == '\u{1b}' as i32
                && buf[5 + i as usize] as core::ffi::c_int == '\\' as i32
            {
                break;
            }
            if buf[5 + i as usize] as core::ffi::c_int == '\u{7}' as i32 {
                break;
            }
            tmp[i as usize] = buf[5 + i as usize];
            i = i.wrapping_add(1);
        }
        if i as usize == tmp.len() - 1 {
            return -(1 as core::ffi::c_int);
        }
        *size = (6 as u_int).wrapping_add(i) as size_t;
        if i == 0 as u_int {
            return 0 as core::ffi::c_int;
        }
        if tmp[i.wrapping_sub(1 as u_int) as usize] as core::ffi::c_int == '\u{1b}' as i32 {
            tmp[i.wrapping_sub(1 as u_int) as usize] = 0;
        } else {
            tmp[i as usize] = 0;
        }
        let colour = core::ffi::CStr::from_bytes_until_nul(&tmp)
            .expect("the colour reply buffer is terminated");
        let n: core::ffi::c_int = RustColourEngine.parse_x11(colour);
        if n != -(1 as core::ffi::c_int) && buf[3] as core::ffi::c_int == '0' as i32 {
            if let Some(c) = c.as_ref() {
                log_debug(
                    c"%s fg is %s",
                    fmt_args![c.name(), RustColourEngine.to_string(n).as_c_str()],
                );
            } else {
                log_debug(
                    c"fg is %s",
                    fmt_args![RustColourEngine.to_string(n).as_c_str()],
                );
            }
            *fg = n;
            tty.flags &= !TTY_WAITFG;
        } else if n != -(1 as core::ffi::c_int) {
            if let Some(c) = c.as_ref() {
                log_debug(
                    c"%s bg is %s",
                    fmt_args![c.name(), RustColourEngine.to_string(n).as_c_str()],
                );
            } else {
                log_debug(
                    c"bg is %s",
                    fmt_args![RustColourEngine.to_string(n).as_c_str()],
                );
            }
            *bg = n;
            tty.flags &= !TTY_WAITBG;
        }
        0 as core::ffi::c_int
    }
}
unsafe fn tty_keys_palette(tty: &mut tty, buf: &[u8], size: &mut size_t) -> core::ffi::c_int {
    unsafe {
        let len = buf.len();
        let mut c = tty_client(tty);
        let mut i: u_int;
        let mut tmp = [0u8; 128];
        let mut endptr: *mut core::ffi::c_char = core::ptr::null_mut::<core::ffi::c_char>();

        *size = 0 as size_t;
        if buf.first() != Some(&0x1b) {
            return -(1 as core::ffi::c_int);
        }
        if len == 1 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[1] as core::ffi::c_int != ']' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 2 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[2] as core::ffi::c_int != '4' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 3 as size_t {
            return 1 as core::ffi::c_int;
        }
        if buf[3] as core::ffi::c_int != ';' as i32 {
            return -(1 as core::ffi::c_int);
        }
        if len == 4 as size_t {
            return 1 as core::ffi::c_int;
        }
        i = 0 as u_int;
        while (i as usize) < tmp.len() - 1 {
            if (4 as u_int).wrapping_add(i) as size_t == len {
                return 1 as core::ffi::c_int;
            }
            if buf[3 + i as usize] as core::ffi::c_int == '\u{1b}' as i32
                && buf[4 + i as usize] as core::ffi::c_int == '\\' as i32
            {
                break;
            }
            if buf[4 + i as usize] as core::ffi::c_int == '\u{7}' as i32 {
                break;
            }
            tmp[i as usize] = buf[4 + i as usize];
            i = i.wrapping_add(1);
        }
        if i as usize == tmp.len() - 1 {
            return -(1 as core::ffi::c_int);
        }
        *size = (5 as u_int).wrapping_add(i) as size_t;
        if i == 0 as u_int {
            return 0 as core::ffi::c_int;
        }
        if tmp[i.wrapping_sub(1 as u_int) as usize] as core::ffi::c_int == '\u{1b}' as i32 {
            tmp[i.wrapping_sub(1 as u_int) as usize] = 0;
        } else {
            tmp[i as usize] = 0;
        }
        let sequence = core::ffi::CStr::from_bytes_until_nul(&tmp)
            .expect("the palette reply buffer is terminated");
        let idx: core::ffi::c_int =
            strtol(sequence.as_ptr(), &raw mut endptr, 10 as core::ffi::c_int) as core::ffi::c_int;
        let consumed = endptr.offset_from(sequence.as_ptr()) as usize;
        if sequence.to_bytes().get(consumed) != Some(&b';') {
            return -(1 as core::ffi::c_int);
        }
        if idx < 0 as core::ffi::c_int || idx > 255 as core::ffi::c_int {
            return -(1 as core::ffi::c_int);
        }
        let colour_name =
            core::ffi::CStr::from_bytes_with_nul(&sequence.to_bytes_with_nul()[consumed + 1..])
                .expect("the colour suffix has the reply's terminator");
        let colour = RustColourEngine.parse_x11(colour_name);
        if colour == -(1 as core::ffi::c_int) {
            return 0 as core::ffi::c_int;
        }
        (c.as_mut().expect("the tty has a client")).handle_input_reply(
            INPUT_REQUEST_PALETTE,
            &InputRequestData::Palette { idx, c: colour },
        );
        0 as core::ffi::c_int
    }
}

#[cfg(test)]
mod focused_tests {
    use super::*;
    use crate::tests::test_fixtures::{Tty, globals};

    unsafe fn with_tty<R>(f: impl FnOnce(&mut tty) -> R) -> R {
        let _guard = globals();
        let mut fixture = Tty::new();
        f(unsafe { &mut *fixture.ptr() })
    }

    #[test]
    fn private_tree_walkers_cover_branches_prefixes_and_replacement() {
        unsafe {
            let mut t = tty::default();
            tty_keys_add1(&mut t.key_tree, b"m", 1);
            tty_keys_add1(&mut t.key_tree, b"a", 2);
            tty_keys_add1(&mut t.key_tree, b"z", 3);
            tty_keys_add1(&mut t.key_tree, b"map", 4);
            tty_keys_add1(&mut t.key_tree, b"man", 5);
            tty_keys_add1(&mut t.key_tree, b"", 9);

            for (bytes, expected, taken) in [
                (b"m".as_slice(), 1, 1),
                (b"a", 2, 1),
                (b"z", 3, 1),
                (b"map", 4, 3),
                (b"man", 5, 3),
            ] {
                let (node, size) = tty_keys_find(&mut t, bytes).expect("stored key");
                assert_eq!(node.key, expected);
                assert_eq!(size, taken);
            }
            assert!(tty_keys_find(&mut t, b"q").is_none());
            assert!(tty_keys_find(&mut t, b"").is_none());

            tty_keys_add(&mut t, c"m", 17);
            assert_eq!(tty_keys_find(&mut t, b"m").unwrap().0.key, 17);
            tty_keys_free(&mut t);
            assert!(t.key_tree.is_none());
        }
    }

    #[test]
    fn paste_end_prefix_detection_has_strict_boundaries() {
        assert!(!tty_keys_partial_paste_end(b""));
        for n in 1..b"\x1b[201~".len() {
            assert!(tty_keys_partial_paste_end(&b"\x1b[201~"[..n]));
        }
        assert!(!tty_keys_partial_paste_end(b"\x1b[201~"));
        assert!(!tty_keys_partial_paste_end(b"\x1b[202"));
        assert!(!tty_keys_partial_paste_end(b"x"));
    }

    #[test]
    fn next1_decodes_tree_keys_utf8_and_incomplete_input() {
        unsafe {
            with_tty(|t| {
                tty_keys_add1(&mut t.key_tree, b"ab", 91);
                tty_keys_add1(&mut t.key_tree, b"abc", 92);
                let mut key = 0;
                let mut size = 0;
                assert_eq!(tty_keys_next1(t, b"ab", &mut key, &mut size, 0), 1);
                assert_eq!(tty_keys_next1(t, b"ab", &mut key, &mut size, 1), 0);
                assert_eq!((key, size), (91, 2));

                assert_eq!(tty_keys_next1(t, "é".as_bytes(), &mut key, &mut size, 0), 0);
                assert_eq!(size, 2);
                assert_ne!(key, 0);
                assert_eq!(tty_keys_next1(t, &[0xc3], &mut key, &mut size, 0), 1);
                assert_eq!(tty_keys_next1(t, &[0xc3], &mut key, &mut size, 1), -1);
                assert_eq!(tty_keys_next1(t, &[0xc3, 0x28], &mut key, &mut size, 1), -1);
                assert_eq!(tty_keys_next1(t, b"x", &mut key, &mut size, 0), -1);
                assert_eq!(tty_keys_next1(t, b"", &mut key, &mut size, 0), -1);
                assert_eq!(size, 0);
            });
        }
    }

    #[test]
    fn extended_key_parser_covers_formats_modifiers_and_unicode() {
        unsafe {
            with_tty(|t| {
                let mut size = 0;
                let mut key = 0;
                assert_eq!(tty_keys_extended_key(t, b"", &mut size, &mut key), -1);
                assert_eq!(size, 0);
                for sequence in [b"\x1b[27;5;97~".as_slice(), b"\x1b[233;2u"] {
                    for end in 1..sequence.len() {
                        assert_eq!(
                            tty_keys_extended_key(t, &sequence[..end], &mut size, &mut key),
                            1
                        );
                    }
                }
                for seq in [b"x".as_slice(), b"\x1bx", b"\x1b[x", b"\x1b[1:x"] {
                    assert_eq!(tty_keys_extended_key(t, seq, &mut size, &mut key), -1);
                }
                assert_eq!(tty_keys_extended_key(t, b"\x1b", &mut size, &mut key), 1);
                assert_eq!(tty_keys_extended_key(t, b"\x1b[", &mut size, &mut key), 1);
                assert_eq!(
                    tty_keys_extended_key(t, b"\x1b[65;", &mut size, &mut key),
                    1
                );

                assert_eq!(
                    tty_keys_extended_key(t, b"\x1b[65;1u", &mut size, &mut key),
                    0
                );
                assert_eq!((key & KEYC_MASK_KEY, size), (65, 7));
                assert_eq!(
                    tty_keys_extended_key(t, b"\x1b[65;8u", &mut size, &mut key),
                    0
                );
                assert_eq!(
                    key & (KEYC_SHIFT | KEYC_META | KEYC_CTRL),
                    KEYC_SHIFT | KEYC_META | KEYC_CTRL
                );
                assert_eq!(
                    tty_keys_extended_key(t, b"\x1b[27;2;9~", &mut size, &mut key),
                    0
                );
                assert_eq!(key & KEYC_MASK_KEY, KEYC_BTAB as key_code);
                assert_eq!(
                    tty_keys_extended_key(t, b"\x1b[233;1u", &mut size, &mut key),
                    0
                );
                assert_ne!(key, 233);
                assert_eq!(
                    tty_keys_extended_key(t, b"\x1b[99999999;1u", &mut size, &mut key),
                    -1
                );
                assert_eq!(
                    tty_keys_extended_key(t, b"\x1b[broken~", &mut size, &mut key),
                    -1
                );
            });
        }
    }

    #[test]
    fn mouse_parser_covers_legacy_and_sgr_protocols() {
        unsafe {
            with_tty(|t| {
                let mut size = 0;
                let mut m = mouse_event::default();
                assert_eq!(tty_keys_mouse(t, b"", &mut size, &mut m), -1);
                assert_eq!(size, 0);
                for sequence in [b"\x1b[M !!".as_slice(), b"\x1b[<0;12;7M"] {
                    for end in 1..sequence.len() {
                        assert_eq!(tty_keys_mouse(t, &sequence[..end], &mut size, &mut m), 1);
                    }
                }
                for seq in [b"x".as_slice(), b"\x1bx", b"\x1b[x"] {
                    assert_eq!(tty_keys_mouse(t, seq, &mut size, &mut m), -1);
                }
                assert_eq!(tty_keys_mouse(t, b"\x1b", &mut size, &mut m), 1);
                assert_eq!(tty_keys_mouse(t, b"\x1b[", &mut size, &mut m), 1);
                assert_eq!(tty_keys_mouse(t, b"\x1b[M", &mut size, &mut m), 1);
                assert_eq!(tty_keys_mouse(t, b"\x1b[M !!", &mut size, &mut m), 0);
                assert_eq!((m.b, m.x, m.y, size), (0, 0, 0, 6));
                assert_eq!(tty_keys_mouse(t, b"\x1b[M\x01!!", &mut size, &mut m), -2);
                assert_eq!(tty_keys_mouse(t, b"\x1b[M \xff\xff", &mut size, &mut m), 0);
                assert_eq!((m.b, m.x, m.y, size), (0, 222, 222, 6));

                assert_eq!(tty_keys_mouse(t, b"\x1b[<0;12;7M", &mut size, &mut m), 0);
                assert_eq!((m.b, m.x, m.y, m.sgr_type), (0, 11, 6, b'M' as u_int));
                assert_eq!(tty_keys_mouse(t, b"\x1b[<0;12;7m", &mut size, &mut m), 0);
                assert_eq!(m.b, 3);
                assert_eq!(tty_keys_mouse(t, b"\x1b[<64;1;1m", &mut size, &mut m), -2);
                assert_eq!(tty_keys_mouse(t, b"\x1b[<0;0;1M", &mut size, &mut m), -2);
                assert_eq!(tty_keys_mouse(t, b"\x1b[<x", &mut size, &mut m), -1);
                assert_eq!(tty_keys_mouse(t, b"\x1b[<1;2;x", &mut size, &mut m), -1);
                assert_eq!(tty_keys_mouse(t, b"\x1b[<1;", &mut size, &mut m), 1);
            });
        }
    }

    #[test]
    fn window_size_parser_handles_queries_and_both_reply_kinds() {
        unsafe {
            with_tty(|t| {
                let mut size = 0;
                assert_eq!(tty_keys_winsz(t, b"\x1b[8;24;80t", &mut size), -1);
                t.flags |= TTY_WINSIZEQUERY;
                assert_eq!(tty_keys_winsz(t, b"", &mut size), -1);
                assert_eq!(size, 0);
                for sequence in [b"\x1b[8;24;80t".as_slice(), b"\x1b[4;480;800t"] {
                    for end in 1..sequence.len() {
                        assert_eq!(tty_keys_winsz(t, &sequence[..end], &mut size), 1);
                    }
                }
                for seq in [b"x".as_slice(), b"\x1bx", b"\x1b[x"] {
                    assert_eq!(tty_keys_winsz(t, seq, &mut size), -1);
                }
                assert_eq!(tty_keys_winsz(t, b"\x1b", &mut size), 1);
                assert_eq!(tty_keys_winsz(t, b"\x1b[", &mut size), 1);
                assert_eq!(tty_keys_winsz(t, b"\x1b[8;24;80", &mut size), 1);
                assert_eq!(tty_keys_winsz(t, b"\x1b[8;24;80x", &mut size), -1);
                assert_eq!(tty_keys_winsz(t, b"\x1b[8;24;80t", &mut size), 0);
                assert_eq!((t.sx, t.sy, size), (80, 24, 10));
                assert_eq!(tty_keys_winsz(t, b"\x1b[4;480;800t", &mut size), 0);
                assert_eq!((t.xpixel, t.ypixel), (10, 20));
                assert_eq!(t.flags & TTY_WINSIZEQUERY, 0);
            });
        }
    }

    #[test]
    fn clipboard_parser_rejects_and_buffers_without_external_effects() {
        unsafe {
            with_tty(|t| {
                let mut size = 0;
                for seq in [
                    b"x".as_slice(),
                    b"\x1bx",
                    b"\x1b]x",
                    b"\x1b]5x",
                    b"\x1b]52x",
                ] {
                    assert_eq!(tty_keys_clipboard(t, seq, &mut size), -1);
                }
                for seq in [
                    b"\x1b".as_slice(),
                    b"\x1b]",
                    b"\x1b]5",
                    b"\x1b]52",
                    b"\x1b]52;",
                ] {
                    assert_eq!(tty_keys_clipboard(t, seq, &mut size), 1);
                }
                assert_eq!(tty_keys_clipboard(t, b"", &mut size), -1);
                assert_eq!(size, 0);
                for sequence in [b"\x1b]52;c;QQ==\x07".as_slice(), b"\x1b]52;c;Qg==\x1b\\"] {
                    for end in 1..sequence.len() {
                        assert_eq!(tty_keys_clipboard(t, &sequence[..end], &mut size), 1);
                    }
                    let mut input = sequence.to_vec();
                    input.extend_from_slice(b"trailing");
                    assert_eq!(tty_keys_clipboard(t, &input, &mut size), 0);
                    assert_eq!(size, sequence.len());
                }
                let invalid = b"\x1b]52;c;%%%%\x07";
                assert_eq!(tty_keys_clipboard(t, invalid, &mut size), 0);
                assert_eq!(size, invalid.len());
                assert_eq!(tty_keys_clipboard(t, b"\x1b]52;c;\x07", &mut size), 0);
                assert_eq!(tty_keys_clipboard(t, b"\x1b]52;c;QQ==", &mut size), 1);
                assert_eq!(tty_keys_clipboard(t, b"\x1b]52;c;%%%%\x1b\\", &mut size), 0);
            });
        }
    }

    #[test]
    fn attribute_fields_preserve_empty_values_truncation_and_the_parameter_limit() {
        let mut input = b"61;;+4; -1;260;4X;8\0ignored\0".to_vec();
        let (values, count) = tty_keys_attribute_values(&mut input);
        assert_eq!(count, 7);
        assert_eq!(
            &values[..7],
            &[61, 0, 4, 255u8 as core::ffi::c_char, 4, 0, 8]
        );
        let mut input = b"1;".repeat(40);
        input.push(0);
        let (values, count) = tty_keys_attribute_values(&mut input);
        assert_eq!(count, 32);
        assert_eq!(values, [1; 32]);
    }

    #[test]
    fn attribute_parsers_cover_partial_malformed_and_already_seen_replies() {
        unsafe {
            with_tty(|t| {
                let mut size = 0;
                for seq in [b"x".as_slice(), b"\x1bx", b"\x1b[x", b"\x1b[!x"] {
                    assert_eq!(tty_keys_device_attributes(t, seq, &mut size), -1);
                }
                for seq in [b"\x1b".as_slice(), b"\x1b[", b"\x1b[?", b"\x1b[?1;2"] {
                    assert_eq!(tty_keys_device_attributes(t, seq, &mut size), 1);
                }
                assert_eq!(tty_keys_device_attributes(t, b"\x1b[?1;2x", &mut size), -1);
                assert_eq!(tty_keys_device_attributes(t, b"", &mut size), -1);
                let sequence = b"\x1b[?0;;\0;c";
                for end in 1..sequence.len() {
                    assert_eq!(
                        tty_keys_device_attributes(t, &sequence[..end], &mut size),
                        1
                    );
                }
                assert_eq!(tty_keys_device_attributes(t, sequence, &mut size), 0);
                assert_eq!(size, sequence.len());
                t.flags |= TTY_HAVEDA;
                assert_eq!(tty_keys_device_attributes(t, b"\x1b[?1c", &mut size), -1);

                t.flags &= !TTY_HAVEDA2;
                for seq in [b"x".as_slice(), b"\x1bx", b"\x1b[x", b"\x1b[?x"] {
                    assert_eq!(tty_keys_device_attributes2(t, seq, &mut size), -1);
                }
                for seq in [b"\x1b".as_slice(), b"\x1b[", b"\x1b[>", b"\x1b[>84;0"] {
                    assert_eq!(tty_keys_device_attributes2(t, seq, &mut size), 1);
                }
                assert_eq!(tty_keys_device_attributes2(t, b"\x1b[>84x", &mut size), -1);
                assert_eq!(tty_keys_device_attributes2(t, b"", &mut size), -1);
                let sequence = b"\x1b[>0;;\0;c";
                for end in 1..sequence.len() {
                    assert_eq!(
                        tty_keys_device_attributes2(t, &sequence[..end], &mut size),
                        1
                    );
                }
                assert_eq!(tty_keys_device_attributes2(t, sequence, &mut size), 0);
                assert_eq!(size, sequence.len());
                t.flags |= TTY_HAVEDA2;
                assert_eq!(tty_keys_device_attributes2(t, b"\x1b[>84c", &mut size), -1);

                for seq in [b"x".as_slice(), b"\x1bx", b"\x1bPx", b"\x1bP>x"] {
                    assert_eq!(tty_keys_extended_device_attributes(t, seq, &mut size), -1);
                }
                for seq in [
                    b"\x1b".as_slice(),
                    b"\x1bP",
                    b"\x1bP>",
                    b"\x1bP>|",
                    b"\x1bP>|tmux",
                ] {
                    assert_eq!(tty_keys_extended_device_attributes(t, seq, &mut size), 1);
                }
                assert_eq!(tty_keys_extended_device_attributes(t, b"", &mut size), -1);
                assert_eq!(size, 0);
                let sequence = b"\x1bP>|unknown\0ignored\x1b\\";
                for end in 1..sequence.len() {
                    assert_eq!(
                        tty_keys_extended_device_attributes(t, &sequence[..end], &mut size),
                        1
                    );
                }
                assert_eq!(
                    tty_keys_extended_device_attributes(t, sequence, &mut size),
                    0
                );
                assert_eq!(size, sequence.len());
                assert_eq!(
                    tty_client(t).unwrap().as_client().term_type.as_deref(),
                    Some(c"unknown")
                );
                t.flags |= TTY_HAVEXDA;
                assert_eq!(
                    tty_keys_extended_device_attributes(t, b"\x1bP>|x\x1b\\", &mut size),
                    -1
                );
            });
        }
    }

    #[test]
    fn palette_parser_exercises_prefix_payload_and_bounds_checks() {
        unsafe {
            with_tty(|t| {
                let mut size = 0;
                assert_eq!(tty_keys_palette(t, b"", &mut size), -1);
                assert_eq!(size, 0);
                for sequence in [
                    b"\x1b]4;;#abcdef\0ignored\x07".as_slice(),
                    b"\x1b]4;0;#abcdef\0ignored\x1b\\",
                ] {
                    for end in 1..sequence.len() {
                        assert_eq!(tty_keys_palette(t, &sequence[..end], &mut size), 1);
                    }
                    let mut input = sequence.to_vec();
                    input.extend_from_slice(b"trailing");
                    assert_eq!(tty_keys_palette(t, &input, &mut size), 0);
                    assert_eq!(size, sequence.len());
                }
                assert_eq!(
                    tty_keys_palette(t, b"\x1b]4;0\0;#abcdef\x07", &mut size),
                    -1
                );
                for seq in [b"x".as_slice(), b"\x1bx", b"\x1b]x", b"\x1b]4x"] {
                    assert_eq!(tty_keys_palette(t, seq, &mut size), -1);
                }
                for seq in [b"\x1b".as_slice(), b"\x1b]", b"\x1b]4", b"\x1b]4;"] {
                    assert_eq!(tty_keys_palette(t, seq, &mut size), 1);
                }
                assert_eq!(tty_keys_palette(t, b"\x1b]4;\x07", &mut size), 0);
                assert_eq!(
                    tty_keys_palette(t, b"\x1b]4;x;rgb:00/00/00\x07", &mut size),
                    -1
                );
                assert_eq!(
                    tty_keys_palette(t, b"\x1b]4;256;rgb:00/00/00\x07", &mut size),
                    -1
                );
                assert_eq!(tty_keys_palette(t, b"\x1b]4;2;nonsense\x07", &mut size), 0);
                assert_eq!(tty_keys_palette(t, b"\x1b]4;2;rgb:00/00/00", &mut size), 1);
            });
        }
    }

    #[test]
    fn default_key_build_populates_raw_xterm_and_term_entries() {
        unsafe {
            with_tty(|t| {
                tty_keys_add1(&mut t.key_tree, b"obsolete", 1);
                tty_keys_build(t);
                assert!(tty_keys_find(t, b"obsolete").is_none());
                for sequence in [b"\x1b[A".as_slice(), b"\x1b[1;2A", b"\x1b[15;5~"] {
                    let (node, consumed) = tty_keys_find(t, sequence).expect("default key");
                    assert_ne!(node.key, KEYC_UNKNOWN as key_code);
                    assert_eq!(consumed, sequence.len());
                }
                tty_keys_build(t);
                assert!(tty_keys_find(t, b"\x1b[A").is_some());
            });
        }
    }

    #[test]
    fn next1_updates_bracketed_paste_state_and_resolves_prefixes() {
        unsafe {
            with_tty(|t| {
                tty_keys_add1(&mut t.key_tree, b"s", KEYC_PASTE_START as key_code);
                tty_keys_add1(&mut t.key_tree, b"e", KEYC_PASTE_END as key_code);
                tty_keys_add1(&mut t.key_tree, b"prefix", 80);
                let mut key = 0;
                let mut size = 0;
                assert_eq!(tty_keys_next1(t, b"s", &mut key, &mut size, 0), 0);
                assert_ne!(t.flags & TTY_BRACKETPASTE, 0);
                assert_eq!(tty_keys_next1(t, b"e", &mut key, &mut size, 0), 0);
                assert_eq!(t.flags & TTY_BRACKETPASTE, 0);
                assert_eq!(tty_keys_next1(t, b"prefix-tail", &mut key, &mut size, 0), 0);
                assert_eq!((key, size), (80, 6));
            });
        }
    }

    #[test]
    fn extended_keys_cover_backspace_shift_and_invalid_forms() {
        unsafe {
            with_tty(|t| {
                let mut size = 0;
                let mut key = 0;
                t.tio.c_cc[VERASE as usize] = 127;
                assert_eq!(
                    tty_keys_extended_key(t, b"\x1b[127;1u", &mut size, &mut key),
                    0
                );
                assert_eq!(key & KEYC_MASK_KEY as key_code, KEYC_BSPACE as key_code);
                assert_eq!(
                    tty_keys_extended_key(t, b"\x1b[65;2u", &mut size, &mut key),
                    0
                );
                assert_eq!(key, b'A' as key_code);
                assert_eq!(
                    tty_keys_extended_key(t, b"\x1b[1;9u", &mut size, &mut key),
                    0
                );
                assert_ne!(key & KEYC_META as key_code, 0);
                assert_eq!(
                    tty_keys_extended_key(t, b"\x1b[27;1~", &mut size, &mut key),
                    -1
                );
                assert_eq!(
                    tty_keys_extended_key(t, b"\x1b[65u", &mut size, &mut key),
                    -1
                );
                let long = format!("\x1b[{}u", "1".repeat(70));
                assert_eq!(
                    tty_keys_extended_key(t, long.as_bytes(), &mut size, &mut key),
                    -1
                );
            });
        }
    }

    #[test]
    fn colour_replies_cover_prefixes_foreground_background_and_limits() {
        unsafe {
            with_tty(|t| {
                let mut size = 0;
                let mut fg = -1;
                let mut bg = -1;
                assert_eq!(tty_keys_colours(t, b"", &mut size, &mut fg, &mut bg), -1);
                assert_eq!(size, 0);
                for sequence in [
                    b"\x1b]10;#abcdef\0ignored\x07".as_slice(),
                    b"\x1b]10;#abcdef\0ignored\x1b\\",
                ] {
                    for end in 1..sequence.len() {
                        assert_eq!(
                            tty_keys_colours(t, &sequence[..end], &mut size, &mut fg, &mut bg),
                            1
                        );
                    }
                    let mut input = sequence.to_vec();
                    input.extend_from_slice(b"trailing");
                    assert_eq!(tty_keys_colours(t, &input, &mut size, &mut fg, &mut bg), 0);
                    assert_eq!(fg, RustColourEngine.parse_x11(c"#abcdef"));
                    assert_eq!(size, sequence.len());
                }
                for seq in [b"x".as_slice(), b"\x1bx", b"\x1b]x", b"\x1b]9", b"\x1b]10x"] {
                    assert_eq!(tty_keys_colours(t, seq, &mut size, &mut fg, &mut bg), -1);
                }
                for seq in [
                    b"\x1b".as_slice(),
                    b"\x1b]",
                    b"\x1b]1",
                    b"\x1b]10",
                    b"\x1b]10;",
                ] {
                    assert_eq!(tty_keys_colours(t, seq, &mut size, &mut fg, &mut bg), 1);
                }
                t.flags |= TTY_WAITFG | TTY_WAITBG;
                assert_eq!(
                    tty_keys_colours(t, b"\x1b]10;rgb:11/22/33\x07", &mut size, &mut fg, &mut bg),
                    0
                );
                assert_ne!(fg, -1);
                assert_eq!(t.flags & TTY_WAITFG, 0);
                assert_eq!(
                    tty_keys_colours(t, b"\x1b]11;#abcdef\x1b\\", &mut size, &mut fg, &mut bg),
                    0
                );
                assert_ne!(bg, -1);
                assert_eq!(t.flags & TTY_WAITBG, 0);
                assert_eq!(
                    tty_keys_colours(t, b"\x1b]10;bad\x07", &mut size, &mut fg, &mut bg),
                    0
                );
                assert_eq!(
                    tty_keys_colours(t, b"\x1b]10;\x07", &mut size, &mut fg, &mut bg),
                    0
                );
                let overlong = format!("\x1b]10;{}\x07", "1".repeat(130));
                assert_eq!(
                    tty_keys_colours(t, overlong.as_bytes(), &mut size, &mut fg, &mut bg),
                    -1
                );
            });
        }
    }
}

impl ClientRef {
    /// Applies an OSC 10 or 11 report to the observed pane's reported colours.
    /// The pane need not belong to the client's current session. The existing
    /// parser leaves malformed or incomplete reports unchanged and updates the
    /// named colour and terminal query wait flag when it recognizes a colour.
    /// The stored pair retains the existing conversion of -1 to unknown.
    /// No redraw, hook or command callback is requested, and no borrow escapes.
    ///
    /// # Safety
    /// Run on the server thread without conflicting access to the client's TTY
    /// or the pane payload. `pane` is weak and must remain live for this call:
    /// resolve it immediately beforehand without an intervening callback or
    /// removal. The terminal parser may read the client's identity.
    pub(crate) unsafe fn report_pane_colours(
        &mut self,
        pane: &mut RustWindowPaneWeak,
        report: &[u8],
    ) {
        unsafe {
            let current = pane.as_pane().colours();
            let mut foreground = current.control_fg.unwrap_or(-1);
            let mut background = current.control_bg.unwrap_or(-1);
            let mut size = 0;
            tty_keys_colours(
                self.as_tty_mut(),
                report,
                &mut size,
                &mut foreground,
                &mut background,
            );
            pane.as_pane_mut().set_colours(PaneControlColourPair {
                control_fg: (foreground != -1).then_some(foreground),
                control_bg: (background != -1).then_some(background),
            });
        }
    }

    pub unsafe fn next_tty_key(&mut self) -> core::ffi::c_int {
        let client = self;

        unsafe {
            let mut current_block: u64;
            let mut tv = timeval::default();
            let mut size: size_t = 0;
            let mut bspace: cc_t;
            let mut delay: core::ffi::c_int;
            let mut expired: core::ffi::c_int = 0 as core::ffi::c_int;
            let mut n: core::ffi::c_int;
            let mut key: key_code = 0;
            let mut onlykey: key_code;
            let mut m = mouse_event::default();
            let input = client.as_tty_mut().r#in.as_mut().unwrap().snapshot();
            let buf = input.as_ref();
            let len = buf.len();
            if len == 0 {
                return 0;
            }
            let keys = buf;
            log_debug(
                c"%s: keys are %zu (%.*s)",
                fmt_args![client.name(), len, len as core::ffi::c_int, buf],
            );
            match tty_keys_clipboard(client.as_tty_mut(), keys, &mut size) {
                0 => {
                    key = KEYC_UNKNOWN as core::ffi::c_ulong as key_code;
                    current_block = 10299298647514879019;
                }
                -1 => {
                    current_block = 1917311967535052937;
                }
                1 => {
                    current_block = 18394967961554547273;
                }
                _ => {
                    current_block = 1917311967535052937;
                }
            }
            if current_block == 1917311967535052937 {
                match tty_keys_device_attributes(client.as_tty_mut(), keys, &mut size) {
                    0 => {
                        key = KEYC_UNKNOWN as core::ffi::c_ulong as key_code;
                        current_block = 10299298647514879019;
                    }
                    -1 => {
                        current_block = 4166486009154926805;
                    }
                    1 => {
                        current_block = 18394967961554547273;
                    }
                    _ => {
                        current_block = 4166486009154926805;
                    }
                }
                match current_block {
                    10299298647514879019 => {}
                    18394967961554547273 => {}
                    _ => {
                        match tty_keys_device_attributes2(client.as_tty_mut(), keys, &mut size) {
                            0 => {
                                key = KEYC_UNKNOWN as core::ffi::c_ulong as key_code;
                                current_block = 10299298647514879019;
                            }
                            -1 => {
                                current_block = 15652330335145281839;
                            }
                            1 => {
                                current_block = 18394967961554547273;
                            }
                            _ => {
                                current_block = 15652330335145281839;
                            }
                        }
                        match current_block {
                            10299298647514879019 => {}
                            18394967961554547273 => {}
                            _ => {
                                match tty_keys_extended_device_attributes(
                                    client.as_tty_mut(),
                                    keys,
                                    &mut size,
                                ) {
                                    0 => {
                                        key = KEYC_UNKNOWN as core::ffi::c_ulong as key_code;
                                        current_block = 10299298647514879019;
                                    }
                                    -1 => {
                                        current_block = 224731115979188411;
                                    }
                                    1 => {
                                        current_block = 18394967961554547273;
                                    }
                                    _ => {
                                        current_block = 224731115979188411;
                                    }
                                }
                                match current_block {
                                    10299298647514879019 => {}
                                    18394967961554547273 => {}
                                    _ => {
                                        let mut fg = client.as_tty().fg;
                                        let mut bg = client.as_tty().bg;
                                        let colours = tty_keys_colours(
                                            client.as_tty_mut(),
                                            keys,
                                            &mut size,
                                            &mut fg,
                                            &mut bg,
                                        );
                                        client.as_tty_mut().fg = fg;
                                        client.as_tty_mut().bg = bg;
                                        match colours {
                                            0 => {
                                                key =
                                                    KEYC_UNKNOWN as core::ffi::c_ulong as key_code;
                                                if let Some(session) = client.attached_session() {
                                                    session.theme_changed();
                                                }
                                                current_block = 10299298647514879019;
                                            }
                                            -1 => {
                                                current_block = 6669252993407410313;
                                            }
                                            1 => {
                                                if let Some(session) = client.attached_session() {
                                                    session.theme_changed();
                                                }
                                                current_block = 18394967961554547273;
                                            }
                                            _ => {
                                                current_block = 6669252993407410313;
                                            }
                                        }
                                        match current_block {
                                            10299298647514879019 => {}
                                            18394967961554547273 => {}
                                            _ => {
                                                match tty_keys_palette(
                                                    client.as_tty_mut(),
                                                    keys,
                                                    &mut size,
                                                ) {
                                                    0 => {
                                                        key = KEYC_UNKNOWN as core::ffi::c_ulong
                                                            as key_code;
                                                        current_block = 10299298647514879019;
                                                    }
                                                    -1 => {
                                                        current_block = 4488286894823169796;
                                                    }
                                                    1 => {
                                                        current_block = 18394967961554547273;
                                                    }
                                                    _ => {
                                                        current_block = 4488286894823169796;
                                                    }
                                                }
                                                match current_block {
                                                    10299298647514879019 => {}
                                                    18394967961554547273 => {}
                                                    _ => {
                                                        match tty_keys_mouse(
                                                            client.as_tty_mut(),
                                                            keys,
                                                            &mut size,
                                                            &mut m,
                                                        ) {
                                                            0 => {
                                                                key = KEYC_MOUSE
                                                                    as core::ffi::c_ulong
                                                                    as key_code;
                                                                current_block =
                                                                    10299298647514879019;
                                                            }
                                                            -1 => {
                                                                current_block =
                                                                    11385396242402735691;
                                                            }
                                                            -2 => {
                                                                key = KEYC_MOUSE
                                                                    as core::ffi::c_ulong
                                                                    as key_code;
                                                                log_debug(
                                                                    c"%s: discard key %.*s %#llx",
                                                                    fmt_args![
                                                                        client.name(),
                                                                        size as core::ffi::c_int,
                                                                        buf,
                                                                        key
                                                                    ],
                                                                );
                                                                client
                                                                    .as_tty_mut()
                                                                    .r#in
                                                                    .as_mut()
                                                                    .unwrap()
                                                                    .drain(size);
                                                                return 1 as core::ffi::c_int;
                                                            }
                                                            1 => {
                                                                current_block =
                                                                    18394967961554547273;
                                                            }
                                                            _ => {
                                                                current_block =
                                                                    11385396242402735691;
                                                            }
                                                        }
                                                        match current_block {
                                                            10299298647514879019 => {}
                                                            18394967961554547273 => {}
                                                            _ => {
                                                                match tty_keys_extended_key(
                                                                    client.as_tty_mut(),
                                                                    keys,
                                                                    &mut size,
                                                                    &mut key,
                                                                ) {
                                                                    0 => {
                                                                        current_block =
                                                                            10299298647514879019;
                                                                    }
                                                                    -1 => {
                                                                        current_block =
                                                                            8845338526596852646;
                                                                    }
                                                                    1 => {
                                                                        current_block =
                                                                            18394967961554547273;
                                                                    }
                                                                    _ => {
                                                                        current_block =
                                                                            8845338526596852646;
                                                                    }
                                                                }
                                                                match current_block {
                                                                    10299298647514879019 => {}
                                                                    18394967961554547273 => {}
                                                                    _ => {
                                                                        match tty_keys_winsz(
                                                                            client.as_tty_mut(),
                                                                            keys,
                                                                            &mut size,
                                                                        ) {
                                                                            0 => {
                                                                                current_block = 15803801955716491872;
                                                                                match current_block {
                                                                                17233256550232664581 => {
                                                                                    current_block = 18394967961554547273;
                                                                                }
                                                                                _ => {
                                                                                    key = KEYC_UNKNOWN as core::ffi::c_ulong as key_code;
                                                                                    current_block = 10299298647514879019;
                                                                                }
                                                                            }
                                                                            }
                                                                            1 => {
                                                                                current_block = 17233256550232664581;
                                                                                match current_block {
                                                                                17233256550232664581 => {
                                                                                    current_block = 18394967961554547273;
                                                                                }
                                                                                _ => {
                                                                                    key = KEYC_UNKNOWN as core::ffi::c_ulong as key_code;
                                                                                    current_block = 10299298647514879019;
                                                                                }
                                                                            }
                                                                            }
                                                                            _ => {
                                                                                current_block = 15385464967731526806;
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            loop {
                match current_block {
                    15385464967731526806 => {
                        n = tty_keys_next1(client.as_tty_mut(), keys, &mut key, &mut size, expired);
                        if n == 0 as core::ffi::c_int {
                            current_block = 10299298647514879019;
                            continue;
                        }
                        if n == 1 as core::ffi::c_int {
                            current_block = 18394967961554547273;
                            continue;
                        }
                        if buf[0] as core::ffi::c_int == '\u{1b}' as i32 && len > 1 as size_t {
                            n = tty_keys_next1(
                                client.as_tty_mut(),
                                &keys[1..],
                                &mut key,
                                &mut size,
                                expired,
                            );
                            if n == 0 as core::ffi::c_int {
                                if key as core::ffi::c_ulonglong & KEYC_IMPLIED_META != 0 {
                                    key = '\u{1b}' as i32 as key_code;
                                    size = 1 as size_t;
                                    current_block = 10299298647514879019;
                                    continue;
                                } else {
                                    key |= KEYC_META;
                                    size = size.wrapping_add(1);
                                    current_block = 10299298647514879019;
                                    continue;
                                }
                            } else if n == 1 as core::ffi::c_int {
                                current_block = 18394967961554547273;
                                continue;
                            }
                        }
                        if buf[0] as core::ffi::c_int == '\u{1b}' as i32 && len >= 2 as size_t {
                            key = buf[1] as key_code | KEYC_META;
                            size = 2 as size_t;
                        } else {
                            key = buf[0] as key_code;
                            size = 1 as size_t;
                        }
                        if key as core::ffi::c_ulonglong & KEYC_MASK_KEY
                            == C0_NUL as core::ffi::c_int as core::ffi::c_ulonglong
                        {
                            key = (' ' as i32 as core::ffi::c_ulonglong
                                | KEYC_CTRL
                                | key as core::ffi::c_ulonglong & KEYC_META)
                                as key_code;
                        }
                        bspace = client.as_tty().tio.c_cc[VERASE as usize];
                        if bspace as core::ffi::c_int != _POSIX_VDISABLE {
                            if key == bspace as key_code {
                                log_debug(
                                    c"%s: key %#llx is BSpace",
                                    fmt_args![client.name(), key],
                                );
                                key = KEYC_BSPACE as core::ffi::c_ulong as key_code;
                            }
                            if key == bspace as core::ffi::c_ulonglong | KEYC_META {
                                log_debug(
                                    c"%s: key %#llx is M-BSpace",
                                    fmt_args![client.name(), key],
                                );
                                key = (KEYC_BSPACE as core::ffi::c_ulong as core::ffi::c_ulonglong
                                    | KEYC_META) as key_code;
                            }
                        }
                        onlykey = (key as core::ffi::c_ulonglong & KEYC_MASK_KEY) as key_code;
                        if onlykey < 0x20 as key_code
                            && onlykey != C0_HT as core::ffi::c_int as key_code
                            && onlykey != C0_CR as core::ffi::c_int as key_code
                            && onlykey != C0_ESC as core::ffi::c_int as key_code
                        {
                            onlykey |= 0x40 as key_code;
                            if onlykey >= 'A' as i32 as key_code
                                && onlykey <= 'Z' as i32 as key_code
                            {
                                onlykey |= 0x20 as key_code;
                            }
                            key = (onlykey as core::ffi::c_ulonglong
                                | KEYC_CTRL
                                | key as core::ffi::c_ulonglong & KEYC_META)
                                as key_code;
                        }
                        current_block = 10299298647514879019;
                    }
                    10299298647514879019 => {
                        log_debug(
                            c"%s: complete key %.*s %#llx",
                            fmt_args![client.name(), size as core::ffi::c_int, buf, key],
                        );
                        client.as_tty_mut().key_timer.disarm();
                        client.as_tty_mut().flags &= !TTY_TIMER;
                        if key == KEYC_FOCUS_OUT as core::ffi::c_ulong as key_code {
                            *client.flags_mut() &= !CLIENT_FOCUSED as uint64_t;
                            server_client_update_focus(client.as_client());
                            notify_client(c"client-focus-out", Some(client.as_client_mut()));
                        } else if key == KEYC_FOCUS_IN as core::ffi::c_ulong as key_code {
                            *client.flags_mut() |= CLIENT_FOCUSED as uint64_t;
                            notify_client(c"client-focus-in", Some(client.as_client_mut()));
                            server_client_update_focus(client.as_client());
                        }
                        if key != KEYC_UNKNOWN as core::ffi::c_ulong as key_code {
                            let event = Box::new(key_event {
                                key,
                                m,
                                buf: if size == 0 {
                                    Vec::new()
                                } else {
                                    buf[..size].to_vec()
                                },
                            });
                            server_client_handle_key(client.as_client_mut(), event);
                        }
                        client.as_tty_mut().r#in.as_mut().unwrap().drain(size);
                        return 1 as core::ffi::c_int;
                    }
                    _ => {
                        log_debug(
                            c"%s: partial key %.*s",
                            fmt_args![client.name(), len as core::ffi::c_int, buf],
                        );
                        if client.as_tty().flags & TTY_TIMER != 0 {
                            if client.as_tty().key_timer.is_set()
                                && !client.as_tty().key_timer.is_armed()
                            {
                                expired = 1 as core::ffi::c_int;
                                current_block = 15385464967731526806;
                            } else {
                                return 0 as core::ffi::c_int;
                            }
                        } else {
                            delay = (global_options
                                .get()
                                .as_ref()
                                .expect("global options are initialized"))
                            .number(c"escape-time")
                                as core::ffi::c_int;
                            if delay == 0 as core::ffi::c_int {
                                delay = 1 as core::ffi::c_int;
                            }
                            if client.as_tty().flags & TTY_BRACKETPASTE != 0
                                && tty_keys_partial_paste_end(keys)
                            {
                                log_debug(
                                    c"%s: increasing delay (partial paste end)",
                                    fmt_args![client.name()],
                                );
                                if delay < 500 as core::ffi::c_int {
                                    delay = 500 as core::ffi::c_int;
                                }
                            }
                            if client.as_tty().flags & (TTY_WAITFG | TTY_WAITBG) != 0
                                || client.as_tty().flags & TTY_ALL_REQUEST_FLAGS
                                    != TTY_ALL_REQUEST_FLAGS
                                || !client.as_client().input_requests.is_empty()
                            {
                                log_debug(
                                    c"%s: increasing delay (active query)",
                                    fmt_args![client.name()],
                                );
                                if delay < 500 as core::ffi::c_int {
                                    delay = 500 as core::ffi::c_int;
                                }
                            }
                            tv.tv_sec = (delay / 1000 as core::ffi::c_int) as __time_t;
                            tv.tv_usec = ((delay % 1000 as core::ffi::c_int) as core::ffi::c_long
                                * 1000 as core::ffi::c_long)
                                as __suseconds_t;
                            let owner = client.as_tty().client.clone();
                            client.as_tty_mut().key_timer.set_callback(move || {
                                if let Some(mut c) = owner.as_ref().and_then(ClientWeak::upgrade) {
                                    c.on_tty_key_timer();
                                }
                            });
                            client.as_tty_mut().key_timer.arm(tv);
                            client.as_tty_mut().flags |= TTY_TIMER;
                            return 0 as core::ffi::c_int;
                        }
                    }
                }
            }
        }
    }
    unsafe fn on_tty_key_timer(&mut self) {
        let client = self;

        unsafe {
            if client.as_tty().flags & TTY_TIMER != 0 {
                while client.next_tty_key() != 0 {}
            }
        }
    }
}

#[cfg(test)]
pub use crate::consts::{
    KEYC_DOUBLECLICK_PANE, KEYC_SECONDCLICK_PANE, KEYC_TRIPLECLICK_PANE, KEYC_TYPE_DOUBLECLICK,
    KEYC_TYPE_FUNCTION, KEYC_TYPE_MOUSEDOWN, KEYC_TYPE_MOUSEDRAG, KEYC_TYPE_MOUSEDRAGEND,
    KEYC_TYPE_MOUSEMOVE, KEYC_TYPE_MOUSEUP, KEYC_TYPE_NOTYPE, KEYC_TYPE_SECONDCLICK,
    KEYC_TYPE_TRIPLECLICK, KEYC_TYPE_USER, KEYC_TYPE_WHEELDOWN, KEYC_TYPE_WHEELUP,
};
