use crate::cmd::cmd_mouse_at;
use crate::fmt_args;
use crate::log::{log_debug, log_get_level};
use crate::options::options_get_number;
use crate::text::key_string_lookup_key;
use crate::text::{utf8_to_data, utf8_towc};
use crate::tmux::global_options;
pub use crate::types::*;
use crate::window::window_pane_visible;
use crate::xmalloc::xasprintf;
use ::core::ffi::{CStr, c_char, c_int};
use ::std::collections::BTreeMap;
use ::std::ffi::CString;


pub const C0_ESC: key_code = 27;
pub const C0_CR: key_code = 13;
pub const C0_HT: key_code = 9;

pub const KEYC_BSPACE: key_code = 8589934599;
pub const KEYC_BTAB: key_code = 8589934618;
pub const KEYC_DC: key_code = 8589934613;
pub const KEYC_DOWN: key_code = 8589934620;
pub const KEYC_END: key_code = 8589934615;
pub const KEYC_F1: key_code = 8589934600;
pub const KEYC_F10: key_code = 8589934609;
pub const KEYC_F11: key_code = 8589934610;
pub const KEYC_F12: key_code = 8589934611;
pub const KEYC_F2: key_code = 8589934601;
pub const KEYC_F3: key_code = 8589934602;
pub const KEYC_F4: key_code = 8589934603;
pub const KEYC_F5: key_code = 8589934604;
pub const KEYC_F6: key_code = 8589934605;
pub const KEYC_F7: key_code = 8589934606;
pub const KEYC_F8: key_code = 8589934607;
pub const KEYC_F9: key_code = 8589934608;
pub const KEYC_HOME: key_code = 8589934614;
pub const KEYC_IC: key_code = 8589934612;
pub const KEYC_KP_EIGHT: key_code = 8589934627;
pub const KEYC_KP_ENTER: key_code = 8589934636;
pub const KEYC_KP_FIVE: key_code = 8589934631;
pub const KEYC_KP_FOUR: key_code = 8589934630;
pub const KEYC_KP_MINUS: key_code = 8589934625;
pub const KEYC_KP_NINE: key_code = 8589934628;
pub const KEYC_KP_ONE: key_code = 8589934633;
pub const KEYC_KP_PERIOD: key_code = 8589934638;
pub const KEYC_KP_PLUS: key_code = 8589934629;
pub const KEYC_KP_SEVEN: key_code = 8589934626;
pub const KEYC_KP_SIX: key_code = 8589934632;
pub const KEYC_KP_SLASH: key_code = 8589934623;
pub const KEYC_KP_STAR: key_code = 8589934624;
pub const KEYC_KP_THREE: key_code = 8589934635;
pub const KEYC_KP_TWO: key_code = 8589934634;
pub const KEYC_KP_ZERO: key_code = 8589934637;
pub const KEYC_LEFT: key_code = 8589934621;
pub const KEYC_MOUSE: key_code = 8589934641;
pub const KEYC_NPAGE: key_code = 8589934616;
pub const KEYC_PASTE_END: key_code = 8589934598;
pub const KEYC_PASTE_START: key_code = 8589934597;
pub const KEYC_PPAGE: key_code = 8589934617;
pub const KEYC_RIGHT: key_code = 8589934622;
pub const KEYC_UNKNOWN: key_code = 8589934593;
pub const KEYC_UP: key_code = 8589934619;

pub const KEYC_TYPE_UNICODE: key_code_type = 0;
pub const KEYC_TYPE_USER: key_code_type = 1;
pub const KEYC_TYPE_FUNCTION: key_code_type = 2;
pub const KEYC_TYPE_MOUSEMOVE: key_code_type = 3;
pub const KEYC_TYPE_TRIPLECLICK: key_code_type = 12;

pub const KEYC_META: key_code = 0x100000000000;
pub const KEYC_CTRL: key_code = 0x200000000000;
pub const KEYC_SHIFT: key_code = 0x400000000000;
pub const KEYC_LITERAL: key_code = 0x1000000000000;
pub const KEYC_KEYPAD: key_code = 0x2000000000000;
pub const KEYC_CURSOR: key_code = 0x4000000000000;
pub const KEYC_IMPLIED_META: key_code = 0x8000000000000;
pub const KEYC_BUILD_MODIFIERS: key_code = 0x10000000000000;
pub const KEYC_MASK_TYPE: key_code = 0xff00000000;
pub const KEYC_MASK_MODIFIERS: key_code = 0xff0000000000;
pub const KEYC_MASK_FLAGS: key_code = 0xff000000000000;
pub const KEYC_MASK_KEY: key_code = 0xffffffffff;

pub const MODE_KCURSOR: c_int = 0x4;
pub const MODE_KKEYPAD: c_int = 0x8;
pub const MODE_MOUSE_STANDARD: c_int = 0x20;
pub const MODE_MOUSE_BUTTON: c_int = 0x40;
pub const MODE_MOUSE_UTF8: c_int = 0x100;
pub const MODE_MOUSE_SGR: c_int = 0x200;
pub const MODE_BRACKETPASTE: c_int = 0x400;
pub const MODE_MOUSE_ALL: c_int = 0x1000;
pub const MODE_KEYS_EXTENDED: c_int = 32768;
pub const MODE_KEYS_EXTENDED_2: c_int = 262144;
pub const ALL_MOUSE_MODES: c_int = MODE_MOUSE_STANDARD | MODE_MOUSE_BUTTON | MODE_MOUSE_ALL;
pub const MOTION_MOUSE_MODES: c_int = MODE_MOUSE_BUTTON | MODE_MOUSE_ALL;
pub const EXTENDED_KEY_MODES: c_int = MODE_KEYS_EXTENDED | MODE_KEYS_EXTENDED_2;
pub const MOUSE_PARAM_MAX: c_int = 0xff;
pub const MOUSE_PARAM_UTF8_MAX: c_int = 0x7ff;
pub const MOUSE_PARAM_BTN_OFF: c_int = 0x20;
pub const MOUSE_PARAM_POS_OFF: c_int = 0x21;
pub const MOUSE_MASK_BUTTONS: c_int = 195;
pub const MOUSE_MASK_DRAG: c_int = 32;

/// The modifiers each digit of an extended key names, in the order the
/// `_` in a default's escape sequence is filled in with: the digit is the
/// index, so index two is `2`.
static input_key_modifiers: [key_code; 9] = [
    0,
    0,
    KEYC_SHIFT,
    KEYC_META | KEYC_IMPLIED_META,
    KEYC_SHIFT | KEYC_META | KEYC_IMPLIED_META,
    KEYC_CTRL,
    KEYC_SHIFT | KEYC_CTRL,
    KEYC_META | KEYC_IMPLIED_META | KEYC_CTRL,
    KEYC_SHIFT | KEYC_META | KEYC_IMPLIED_META | KEYC_CTRL,
];

/// The escape sequence each key is written as. An entry carrying
/// `KEYC_BUILD_MODIFIERS` stands for seven of them, one per modifier
/// combination, with the `_` in its sequence filled in with the digit that
/// names them.
static input_key_defaults: [(key_code, &CStr); 85] = [
    (KEYC_PASTE_START, c"\x1b[200~"),
    (KEYC_PASTE_START | KEYC_IMPLIED_META, c"\x1b[200~"),
    (KEYC_PASTE_END, c"\x1b[201~"),
    (KEYC_PASTE_END | KEYC_IMPLIED_META, c"\x1b[201~"),
    (KEYC_F1, c"\x1bOP"),
    (KEYC_F2, c"\x1bOQ"),
    (KEYC_F3, c"\x1bOR"),
    (KEYC_F4, c"\x1bOS"),
    (KEYC_F5, c"\x1b[15~"),
    (KEYC_F6, c"\x1b[17~"),
    (KEYC_F7, c"\x1b[18~"),
    (KEYC_F8, c"\x1b[19~"),
    (KEYC_F9, c"\x1b[20~"),
    (KEYC_F10, c"\x1b[21~"),
    (KEYC_F11, c"\x1b[23~"),
    (KEYC_F12, c"\x1b[24~"),
    (KEYC_IC, c"\x1b[2~"),
    (KEYC_DC, c"\x1b[3~"),
    (KEYC_HOME, c"\x1b[1~"),
    (KEYC_END, c"\x1b[4~"),
    (KEYC_NPAGE, c"\x1b[6~"),
    (KEYC_PPAGE, c"\x1b[5~"),
    (KEYC_BTAB, c"\x1b[Z"),
    (KEYC_UP | KEYC_CURSOR, c"\x1bOA"),
    (KEYC_DOWN | KEYC_CURSOR, c"\x1bOB"),
    (KEYC_RIGHT | KEYC_CURSOR, c"\x1bOC"),
    (KEYC_LEFT | KEYC_CURSOR, c"\x1bOD"),
    (KEYC_UP, c"\x1b[A"),
    (KEYC_DOWN, c"\x1b[B"),
    (KEYC_RIGHT, c"\x1b[C"),
    (KEYC_LEFT, c"\x1b[D"),
    (KEYC_KP_SLASH | KEYC_KEYPAD, c"\x1bOo"),
    (KEYC_KP_STAR | KEYC_KEYPAD, c"\x1bOj"),
    (KEYC_KP_MINUS | KEYC_KEYPAD, c"\x1bOm"),
    (KEYC_KP_SEVEN | KEYC_KEYPAD, c"\x1bOw"),
    (KEYC_KP_EIGHT | KEYC_KEYPAD, c"\x1bOx"),
    (KEYC_KP_NINE | KEYC_KEYPAD, c"\x1bOy"),
    (KEYC_KP_PLUS | KEYC_KEYPAD, c"\x1bOk"),
    (KEYC_KP_FOUR | KEYC_KEYPAD, c"\x1bOt"),
    (KEYC_KP_FIVE | KEYC_KEYPAD, c"\x1bOu"),
    (KEYC_KP_SIX | KEYC_KEYPAD, c"\x1bOv"),
    (KEYC_KP_ONE | KEYC_KEYPAD, c"\x1bOq"),
    (KEYC_KP_TWO | KEYC_KEYPAD, c"\x1bOr"),
    (KEYC_KP_THREE | KEYC_KEYPAD, c"\x1bOs"),
    (KEYC_KP_ENTER | KEYC_KEYPAD, c"\x1bOM"),
    (KEYC_KP_ZERO | KEYC_KEYPAD, c"\x1bOp"),
    (KEYC_KP_PERIOD | KEYC_KEYPAD, c"\x1bOn"),
    (KEYC_KP_SLASH, c"/"),
    (KEYC_KP_STAR, c"*"),
    (KEYC_KP_MINUS, c"-"),
    (KEYC_KP_SEVEN, c"7"),
    (KEYC_KP_EIGHT, c"8"),
    (KEYC_KP_NINE, c"9"),
    (KEYC_KP_PLUS, c"+"),
    (KEYC_KP_FOUR, c"4"),
    (KEYC_KP_FIVE, c"5"),
    (KEYC_KP_SIX, c"6"),
    (KEYC_KP_ONE, c"1"),
    (KEYC_KP_TWO, c"2"),
    (KEYC_KP_THREE, c"3"),
    (KEYC_KP_ENTER, c"\n"),
    (KEYC_KP_ZERO, c"0"),
    (KEYC_KP_PERIOD, c"."),
    (KEYC_F1 | KEYC_BUILD_MODIFIERS, c"\x1b[1;_P"),
    (KEYC_F2 | KEYC_BUILD_MODIFIERS, c"\x1b[1;_Q"),
    (KEYC_F3 | KEYC_BUILD_MODIFIERS, c"\x1b[1;_R"),
    (KEYC_F4 | KEYC_BUILD_MODIFIERS, c"\x1b[1;_S"),
    (KEYC_F5 | KEYC_BUILD_MODIFIERS, c"\x1b[15;_~"),
    (KEYC_F6 | KEYC_BUILD_MODIFIERS, c"\x1b[17;_~"),
    (KEYC_F7 | KEYC_BUILD_MODIFIERS, c"\x1b[18;_~"),
    (KEYC_F8 | KEYC_BUILD_MODIFIERS, c"\x1b[19;_~"),
    (KEYC_F9 | KEYC_BUILD_MODIFIERS, c"\x1b[20;_~"),
    (KEYC_F10 | KEYC_BUILD_MODIFIERS, c"\x1b[21;_~"),
    (KEYC_F11 | KEYC_BUILD_MODIFIERS, c"\x1b[23;_~"),
    (KEYC_F12 | KEYC_BUILD_MODIFIERS, c"\x1b[24;_~"),
    (KEYC_UP | KEYC_BUILD_MODIFIERS, c"\x1b[1;_A"),
    (KEYC_DOWN | KEYC_BUILD_MODIFIERS, c"\x1b[1;_B"),
    (KEYC_RIGHT | KEYC_BUILD_MODIFIERS, c"\x1b[1;_C"),
    (KEYC_LEFT | KEYC_BUILD_MODIFIERS, c"\x1b[1;_D"),
    (KEYC_HOME | KEYC_BUILD_MODIFIERS, c"\x1b[1;_H"),
    (KEYC_END | KEYC_BUILD_MODIFIERS, c"\x1b[1;_F"),
    (KEYC_PPAGE | KEYC_BUILD_MODIFIERS, c"\x1b[5;_~"),
    (KEYC_NPAGE | KEYC_BUILD_MODIFIERS, c"\x1b[6;_~"),
    (KEYC_IC | KEYC_BUILD_MODIFIERS, c"\x1b[2;_~"),
    (KEYC_DC | KEYC_BUILD_MODIFIERS, c"\x1b[3;_~"),
];

/// Every key the terminal is told about, in key order. tmux runs on a single
/// thread, which is what makes handing out the global safe.
static mut INPUT_KEYS: BTreeMap<key_code, CString> = BTreeMap::new();

fn keys() -> &'static mut BTreeMap<key_code, CString> {
    unsafe { &mut INPUT_KEYS }
}

/// What the terminal is sent for `key`, if anything.
fn input_key_get(key: key_code) -> Option<&'static CStr> {
    keys().get(&key).map(|data| data.as_c_str())
}

/// Works out the key table, filling in each `_` of the defaults that stand for
/// a whole family with the digit naming its modifiers. A key already there
/// keeps what it had, the way the tree's insert did.
pub fn input_key_build() {
    unsafe {
        for &(key, data) in &input_key_defaults {
            if key & KEYC_BUILD_MODIFIERS == 0 {
                keys().entry(key).or_insert_with(|| data.to_owned());
                continue;
            }
            for (j, modifiers) in input_key_modifiers.iter().enumerate().skip(2) {
                let mut bytes = data.to_bytes().to_vec();
                let at = bytes
                    .iter()
                    .position(|&byte| byte == b'_')
                    .expect("every family names where its modifiers go");
                bytes[at] = b'0' + j as u8;
                keys()
                    .entry((key & !KEYC_BUILD_MODIFIERS) | modifiers)
                    .or_insert_with(|| CString::new(bytes).expect("no NUL inside"));
            }
        }
        for (key, data) in keys().iter() {
            log_debug(
                c"%s: 0x%llx (%s) is %s".as_ptr(),
                fmt_args![
                    c"input_key_build".as_ptr(),
                    *key,
                    key_string_lookup_key(*key, 1),
                    data.as_ptr()
                ],
            );
        }
    }
}

/// Whether `key` is a mouse key rather than one the terminal is sent bytes for.
fn is_mouse_key(key: key_code) -> bool {
    key & KEYC_MASK_KEY == KEYC_MOUSE
        || (key & KEYC_MASK_TYPE >= (KEYC_TYPE_MOUSEMOVE as key_code) << 32
            && key & KEYC_MASK_TYPE <= (KEYC_TYPE_TRIPLECLICK as key_code) << 32)
}

/// Tells the pane about a key: a mouse key becomes a mouse report if it landed
/// in this pane, and anything else is written to it as bytes.
pub unsafe fn input_key_pane(
    wp: *mut window_pane,
    key: key_code,
    m: Option<&mouse_event>,
) -> c_int {
    unsafe {
        if log_get_level() != 0 {
            log_debug(
                c"writing key 0x%llx (%s) to %%%u".as_ptr(),
                fmt_args![key, key_string_lookup_key(key, 1), (*wp).id],
            );
        }
        if is_mouse_key(key) {
            if let Some(m) = m
                && m.wp != -1
                && m.wp as u_int == (*wp).id
            {
                input_key_mouse(wp, m);
            }
            return 0;
        }
        input_key((*wp).screen(), (*wp).event, key)
    }
}

/// Hands `data` to the terminal.
fn input_key_write(from: &CStr, bev: Stream, data: &[u8]) {
    unsafe {
        log_debug(
            c"%s: %.*s".as_ptr(),
            fmt_args![from.as_ptr(), data.len() as c_int, data.as_ptr()],
        );
        bev.write(data.as_ptr(), data.len() as size_t);
    }
}

/// Writes `key` in the form that can name any modifier, which a terminal has
/// to ask for. Answers -1 for a key it cannot name: one with no modifiers at
/// all, or one whose character cannot be read back.
fn input_key_extended(bev: Stream, mut key: key_code) -> c_int {
    unsafe {
        let modifier = match key & KEYC_MASK_MODIFIERS {
            m if m == KEYC_SHIFT => b'2',
            m if m == KEYC_META => b'3',
            m if m == KEYC_SHIFT | KEYC_META => b'4',
            m if m == KEYC_CTRL => b'5',
            m if m == KEYC_SHIFT | KEYC_CTRL => b'6',
            m if m == KEYC_META | KEYC_CTRL => b'7',
            m if m == KEYC_SHIFT | KEYC_META | KEYC_CTRL => b'8',
            _ => return -1,
        };

        if key & KEYC_MASK_TYPE == (KEYC_TYPE_UNICODE as key_code) << 32
            && key & KEYC_MASK_KEY > 0x7f
        {
            let mut ud = utf8_data::default();
            utf8_to_data((key & KEYC_MASK_KEY) as utf8_char, &mut ud);
            let Some(wc) = utf8_towc(&ud) else {
                return -1;
            };
            key = wc as key_code;
        } else {
            key &= KEYC_MASK_KEY;
        }

        let tmp = if options_get_number(global_options, c"extended-keys-format".as_ptr()) == 1 {
            xasprintf(
                c"\x1b[27;%c;%llu~".as_ptr(),
                fmt_args![modifier as c_int, key],
            )
        } else {
            xasprintf(c"\x1b[%llu;%cu".as_ptr(), fmt_args![key, modifier as c_int])
        };
        input_key_write(c"input_key_extended", bev, tmp.as_bytes());
        0
    }
}

/// The one-byte forms a terminal with no extended keys understands: what a
/// control key stands for, with an escape in front of it for meta.
///
/// The two rows below are read together — a control key found in the first is
/// written as the byte below it in the second — and the first is searched
/// through its terminating NUL, the way the C's `strchr` was.
static standard_map: [&[u8]; 2] = [
    b"1!9(0)=+;:'\",<.>/-8? 2\0",
    b"119900=+;;'',,..\x1f\x1f\x7f\x7f\0\0\0",
];

fn input_key_vt10x(bev: Stream, mut key: key_code) -> c_int {
    unsafe {
        log_debug(
            c"%s: key in %llx".as_ptr(),
            fmt_args![c"input_key_vt10x".as_ptr(), key],
        );

        if key & KEYC_META != 0 {
            input_key_write(c"input_key_vt10x", bev, b"\x1b");
        }
        if key & KEYC_MASK_TYPE == (KEYC_TYPE_UNICODE as key_code) << 32
            && key & KEYC_MASK_KEY > 0x7f
        {
            let mut ud = utf8_data::default();
            utf8_to_data(key as utf8_char, &mut ud);
            input_key_write(c"input_key_vt10x", bev, &ud.data[..ud.size as usize]);
            return 0;
        }

        let onlykey = key & KEYC_MASK_KEY;
        if onlykey == b'\r' as key_code
            || onlykey == b'\n' as key_code
            || onlykey == b'\t' as key_code
        {
            key &= !KEYC_CTRL;
        }
        if key & KEYC_CTRL != 0 {
            if let Some(i) = standard_map[0].iter().position(|&b| b == onlykey as u8) {
                key = standard_map[1][i] as key_code;
            } else if (b'3' as key_code..=b'7' as key_code).contains(&onlykey) {
                key = onlykey - 0x18;
            } else if (b'@' as key_code..=b'~' as key_code).contains(&onlykey) {
                key = onlykey & 0x1f;
            } else {
                return -1;
            }
        }

        log_debug(
            c"%s: key out %llx".as_ptr(),
            fmt_args![c"input_key_vt10x".as_ptr(), key],
        );
        input_key_write(c"input_key_vt10x", bev, &[(key & 0x7f) as u8]);
        0
    }
}

/// The first extended-key mode: only the keys a one-byte form cannot carry are
/// written in the extended form, and the rest as themselves.
fn input_key_mode1(bev: Stream, key: key_code) -> c_int {
    unsafe {
        log_debug(
            c"%s: key in %llx".as_ptr(),
            fmt_args![c"input_key_mode1".as_ptr(), key],
        );

        if key & (KEYC_CTRL | KEYC_META) == KEYC_META {
            return input_key_vt10x(bev, key);
        }
        let onlykey = key & KEYC_MASK_KEY;
        if key & KEYC_CTRL != 0
            && (onlykey == b' ' as key_code
                || onlykey == b'/' as key_code
                || onlykey == b'@' as key_code
                || onlykey == b'^' as key_code
                || (b'2' as key_code..=b'8' as key_code).contains(&onlykey)
                || (b'@' as key_code..=b'~' as key_code).contains(&onlykey))
        {
            return input_key_vt10x(bev, key);
        }
        -1
    }
}

/// Writes `key` to the terminal behind `bev`, in whichever form the modes of
/// `s` say it understands.
pub unsafe fn input_key(s: *mut screen, bev: Stream, mut key: key_code) -> c_int {
    unsafe {
        if is_mouse_key(key) {
            return 0;
        }

        /* A literal key is written as the one byte it stands for. */
        if key & KEYC_LITERAL != 0 {
            input_key_write(c"input_key", bev, &[key as u8]);
            return 0;
        }

        /* Backspace is whatever the option says it is. */
        if key & KEYC_MASK_KEY == KEYC_BSPACE {
            let mut newkey = options_get_number(global_options, c"backspace".as_ptr()) as key_code;
            log_debug(
                c"%s: key 0x%llx is backspace -> 0x%llx".as_ptr(),
                fmt_args![c"input_key".as_ptr(), key, newkey],
            );
            if key & KEYC_MASK_MODIFIERS == 0 {
                let mut byte = 255;
                if newkey & KEYC_MASK_MODIFIERS == 0 {
                    byte = newkey as u8;
                } else if newkey & KEYC_MASK_MODIFIERS == KEYC_CTRL {
                    newkey &= KEYC_MASK_KEY;
                    if newkey == b'?' as key_code {
                        byte = 0x7f;
                    } else if (b'@' as key_code..=b'_' as key_code).contains(&newkey) {
                        byte = (newkey - 0x40) as u8;
                    } else if (b'a' as key_code..=b'z' as key_code).contains(&newkey) {
                        byte = (newkey - 0x60) as u8;
                    }
                }
                if byte != 255 {
                    input_key_write(c"input_key", bev, &[byte]);
                }
                return 0;
            }
            key = newkey | (key & (KEYC_MASK_FLAGS | KEYC_MASK_MODIFIERS));
        }

        /*
         * A back tab is a shifted tab to a terminal that asked for every key
         * in the extended form, and a key of its own to any other.
         */
        if key & KEYC_MASK_KEY == KEYC_BTAB {
            if (*s).mode & MODE_KEYS_EXTENDED_2 != 0 {
                key = b'\t' as key_code | (key & !KEYC_MASK_KEY) | KEYC_SHIFT;
            } else {
                key &= !KEYC_MASK_MODIFIERS;
            }
        }

        /* A key with nothing on it is written as itself. */
        if key & !KEYC_MASK_KEY == 0 {
            if key == C0_HT as key_code
                || key == C0_CR as key_code
                || key == C0_ESC as key_code
                || (0x20..=0x7f).contains(&key)
            {
                input_key_write(c"input_key", bev, &[key as u8]);
                return 0;
            }
            if key & KEYC_MASK_TYPE == (KEYC_TYPE_UNICODE as key_code) << 32
                && key & KEYC_MASK_KEY > 0x7f
            {
                let mut ud = utf8_data::default();
                utf8_to_data(key as utf8_char, &mut ud);
                input_key_write(c"input_key", bev, &ud.data[..ud.size as usize]);
                return 0;
            }
        }

        /*
         * The keypad and cursor forms of a key only stand for themselves while
         * the terminal has asked for them.
         */
        if (*s).mode & MODE_KKEYPAD == 0 {
            key &= !KEYC_KEYPAD;
        }
        if (*s).mode & MODE_KCURSOR == 0 {
            key &= !KEYC_CURSOR;
        }

        let mut found = input_key_get(key);
        if found.is_none() && key & KEYC_META != 0 && key & KEYC_IMPLIED_META == 0 {
            found = input_key_get(key & !KEYC_META);
        }
        if found.is_none() && key & KEYC_CURSOR != 0 {
            found = input_key_get(key & !KEYC_CURSOR);
        }
        if found.is_none() && key & KEYC_KEYPAD != 0 {
            found = input_key_get(key & !KEYC_KEYPAD);
        }
        if let Some(data) = found {
            log_debug(
                c"%s: found key 0x%llx: \"%s\"".as_ptr(),
                fmt_args![c"input_key".as_ptr(), key, data.as_ptr()],
            );
            if key & KEYC_MASK_TYPE == (KEYC_TYPE_FUNCTION as key_code) << 32
                && (key & KEYC_MASK_KEY == KEYC_PASTE_START
                    || key & KEYC_MASK_KEY == KEYC_PASTE_END)
                && (*s).mode & MODE_BRACKETPASTE == 0
            {
                return 0;
            }
            if key & KEYC_META != 0 && key & KEYC_IMPLIED_META == 0 {
                input_key_write(c"input_key", bev, b"\x1b");
            }
            input_key_write(c"input_key", bev, data.to_bytes());
            return 0;
        }

        /* No terminal has a form for these. */
        if key & KEYC_MASK_TYPE == (KEYC_TYPE_USER as key_code) << 32
            || key & KEYC_MASK_TYPE == (KEYC_TYPE_FUNCTION as key_code) << 32
            || is_mouse_key(key)
        {
            log_debug(
                c"%s: ignoring key 0x%llx".as_ptr(),
                fmt_args![c"input_key".as_ptr(), key],
            );
            return 0;
        }

        match (*s).mode & EXTENDED_KEY_MODES {
            MODE_KEYS_EXTENDED_2 => input_key_extended(bev, key),
            MODE_KEYS_EXTENDED => {
                if input_key_mode1(bev, key) == -1 {
                    return input_key_extended(bev, key);
                }
                0
            }
            _ => input_key_vt10x(bev, key),
        }
    }
}

/// One mouse parameter, as one byte or as the two a UTF-8 terminal reads.
fn input_key_split2(c: u_int, out: &mut Vec<u8>) {
    if c > 0x7f {
        out.push((c >> 6 | 0xc0) as u8);
        out.push((c & 0x3f | 0x80) as u8);
    } else {
        out.push(c as u8);
    }
}

/// The mouse report `m` at (`x`, `y`) is written as, in whichever form the
/// modes of `s` asked for, as the caller's own bytes. Answers nothing for a
/// report the terminal did not ask for or that the form cannot carry.
pub unsafe fn input_key_get_mouse(
    s: *mut screen,
    m: &mouse_event,
    x: u_int,
    y: u_int,
) -> Option<Vec<u8>> {
    unsafe {
        /* A drag needs the terminal to be following them. */
        if m.b & MOUSE_MASK_DRAG as u_int != 0 && (*s).mode & MOTION_MOUSE_MODES == 0 {
            return None;
        }
        if (*s).mode & ALL_MOUSE_MODES == 0 {
            return None;
        }

        /* A drag with no button down is only for a terminal wanting them all. */
        if m.sgr_type != b' ' as u_int {
            if m.sgr_b & MOUSE_MASK_DRAG as u_int != 0
                && m.sgr_b & MOUSE_MASK_BUTTONS as u_int == 3
                && (*s).mode & MODE_MOUSE_ALL == 0
            {
                return None;
            }
        } else if m.b & MOUSE_MASK_DRAG as u_int != 0
            && m.b & MOUSE_MASK_BUTTONS as u_int == 3
            && m.lb & MOUSE_MASK_BUTTONS as u_int == 3
            && (*s).mode & MODE_MOUSE_ALL == 0
        {
            return None;
        }

        let mut out = Vec::<u8>::new();
        if m.sgr_type != b' ' as u_int && (*s).mode & MODE_MOUSE_SGR != 0 {
            let tmp = xasprintf(
                c"\x1b[<%u;%u;%u%c".as_ptr(),
                fmt_args![m.sgr_b, x.wrapping_add(1), y.wrapping_add(1), m.sgr_type],
            );
            out.extend_from_slice(tmp.as_bytes());
        } else if (*s).mode & MODE_MOUSE_UTF8 != 0 {
            if m.b > (MOUSE_PARAM_UTF8_MAX - MOUSE_PARAM_BTN_OFF) as u_int
                || x > (MOUSE_PARAM_UTF8_MAX - MOUSE_PARAM_POS_OFF) as u_int
                || y > (MOUSE_PARAM_UTF8_MAX - MOUSE_PARAM_POS_OFF) as u_int
            {
                return None;
            }
            out.extend_from_slice(b"\x1b[M");
            input_key_split2(m.b.wrapping_add(MOUSE_PARAM_BTN_OFF as u_int), &mut out);
            input_key_split2(x.wrapping_add(MOUSE_PARAM_POS_OFF as u_int), &mut out);
            input_key_split2(y.wrapping_add(MOUSE_PARAM_POS_OFF as u_int), &mut out);
        } else {
            if m.b.wrapping_add(MOUSE_PARAM_BTN_OFF as u_int) > MOUSE_PARAM_MAX as u_int {
                return None;
            }
            out.extend_from_slice(b"\x1b[M");
            out.push(m.b.wrapping_add(MOUSE_PARAM_BTN_OFF as u_int) as u8);
            for place in [x, y] {
                let param = place.wrapping_add(MOUSE_PARAM_POS_OFF as u_int);
                if param > MOUSE_PARAM_MAX as u_int {
                    out.push(MOUSE_PARAM_MAX as u8);
                } else {
                    out.push(param as u8);
                }
            }
        }

        Some(out)
    }
}

/// Tells the pane's terminal where the mouse is, if it asked and the mouse is
/// over a part of it that can be seen.
unsafe fn input_key_mouse(wp: *mut window_pane, m: &mouse_event) {
    unsafe {
        let s = (*wp).screen();
        if m.ignore != 0 || (*s).mode & ALL_MOUSE_MODES == 0 {
            return;
        }

        let Some((x, y)) = cmd_mouse_at(wp, m, 0) else {
            return;
        };
        if window_pane_visible(wp) == 0 {
            return;
        }

        let Some(report) = input_key_get_mouse(s, m, x, y) else {
            return;
        };
        log_debug(
            c"writing mouse %.*s to %%%u".as_ptr(),
            fmt_args![
                report.len() as c_int,
                report.as_ptr() as *const c_char,
                (*wp).id
            ],
        );
        input_key_write(c"input_key_mouse", (*wp).event, &report);
    }
}
#[cfg(test)]
#[path = "../tests/test_input_keys.rs"]
mod tests;
