use crate::src::cmd::cmd_mouse_at;
use crate::src::ffi::libc::{strchr, strlen};
use crate::src::format::bytes::{xformat, xformat_with};
use crate::src::key_string::key_string_format;
use crate::src::log::{log_cstr, log_cstr_n, log_debug, log_get_level};
use crate::src::options::options_get_number;
use crate::src::reactor::bufferevent_write;
use crate::src::shared::abi::*;
use crate::src::shared::control_character::{C0_CR, C0_ESC, C0_HT};
use crate::src::shared::event::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::mouse::{
    mouse_event, MOUSE_MASK_BUTTONS, MOUSE_MASK_DRAG, MOUSE_PARAM_BTN_OFF, MOUSE_PARAM_MAX,
    MOUSE_PARAM_POS_OFF, MOUSE_PARAM_UTF8_MAX,
};
use crate::src::shared::pane::window_pane;
use crate::src::shared::screen::{
    screen, ALL_MOUSE_MODES, EXTENDED_KEY_MODES, MODE_BRACKETPASTE, MODE_KCURSOR,
    MODE_KEYS_EXTENDED, MODE_KEYS_EXTENDED_2, MODE_KKEYPAD, MODE_MOUSE_ALL, MODE_MOUSE_BUTTON,
    MODE_MOUSE_SGR, MODE_MOUSE_UTF8,
};
use crate::src::shared::utf8::wchar_t;
use crate::src::shared::utf8::*;
use crate::src::text::utf8::{utf8_to_data, utf8_towc};
use crate::src::tmux::global_options;
use crate::src::window::window_pane_is_visible;
use std::ffi::{CStr, CString};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_key_entry {
    pub key: key_code,
    pub data: *const ::core::ffi::c_char,
}
#[derive(Default)]
pub struct input_key_tree {
    entries: std::collections::BTreeMap<key_code, *mut input_key_entry>,
    generated: Vec<Box<InputKeyGenerated>>,
}

// The public entry borrows the CString beside it. The tree keeps these boxes
// for its lifetime, so growing the owner vector does not move entry pointers.
struct InputKeyGenerated {
    entry: input_key_entry,
    _data: CString,
}
pub const MOTION_MOUSE_MODES: ::core::ffi::c_int = MODE_MOUSE_BUTTON | MODE_MOUSE_ALL;

// The old RB comparator ordered entries by their unsigned key value. The map
// therefore preserves exact lookup, duplicate insertion, and in-order
// traversal semantics without storing links in each input_key_entry.
fn input_key_tree_find(head: &input_key_tree, key: key_code) -> *mut input_key_entry {
    head.entries
        .get(&key)
        .copied()
        .unwrap_or(::core::ptr::null_mut::<input_key_entry>())
}

unsafe fn input_key_tree_insert(
    head: &mut input_key_tree,
    elm: &mut input_key_entry,
) -> *mut input_key_entry {
    match head.entries.entry(elm.key) {
        std::collections::btree_map::Entry::Occupied(entry) => *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(&raw mut *elm);
            ::core::ptr::null_mut::<input_key_entry>()
        }
    }
}

unsafe fn input_key_tree_insert_generated(
    head: &mut input_key_tree,
    mut generated: Box<InputKeyGenerated>,
) {
    let entry = &raw mut generated.entry;
    if input_key_tree_insert(head, &mut *entry).is_null() {
        head.generated.push(generated);
    }
}

fn input_key_generated(template: &CStr, key: key_code, j: u_int) -> Box<InputKeyGenerated> {
    let mut bytes = template.to_bytes().to_vec();
    let modifier = bytes
        .iter()
        .position(|byte| *byte == b'_')
        .expect("modified key template has no placeholder");
    bytes[modifier] = b'0' + j as u8;
    let data = CString::new(bytes).expect("modified key template contains an interior NUL");
    Box::new(InputKeyGenerated {
        entry: input_key_entry {
            key,
            data: data.as_ptr(),
        },
        _data: data,
    })
}

unsafe fn input_key_tree_minmax(head: &input_key_tree) -> *mut input_key_entry {
    let entry = head.entries.iter().next();
    entry
        .map(|(_, entry)| *entry)
        .unwrap_or(::core::ptr::null_mut::<input_key_entry>())
}

unsafe fn input_key_tree_next(
    head: &input_key_tree,
    elm: &input_key_entry,
) -> *mut input_key_entry {
    head.entries
        .range((
            std::ops::Bound::Excluded(elm.key),
            std::ops::Bound::Unbounded,
        ))
        .next()
        .map(|(_, entry)| *entry)
        .unwrap_or(::core::ptr::null_mut::<input_key_entry>())
}
pub static mut input_key_tree: input_key_tree = input_key_tree {
    entries: std::collections::BTreeMap::new(),
    generated: Vec::new(),
};

static mut input_key_defaults: [input_key_entry; 85] = [
    input_key_entry {
        key: KEYC_PASTE_START as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[200~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_PASTE_START as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
        data: b"\x1B[200~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_PASTE_END as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[201~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_PASTE_END as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
        data: b"\x1B[201~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code,
        data: b"\x1BOP\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code,
        data: b"\x1BOQ\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code,
        data: b"\x1BOR\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code,
        data: b"\x1BOS\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[15~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[17~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[18~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[19~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[20~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[21~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[23~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[24~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_IC as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[2~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_DC as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[3~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[1~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_END as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[4~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[6~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[5~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_BTAB as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[Z\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
        data: b"\x1BOA\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
        data: b"\x1BOB\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
        data: b"\x1BOC\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
        data: b"\x1BOD\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_UP as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[A\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[B\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[C\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[D\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_SLASH as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOo\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_STAR as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOj\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_MINUS as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOm\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_SEVEN as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOw\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_EIGHT as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOx\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_NINE as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOy\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_PLUS as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOk\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_FOUR as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOt\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_FIVE as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOu\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_SIX as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOv\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_ONE as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOq\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_TWO as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOr\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_THREE as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOs\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_ENTER as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOM\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_ZERO as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOp\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_PERIOD as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOn\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_SLASH as ::core::ffi::c_ulong as key_code,
        data: b"/\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_STAR as ::core::ffi::c_ulong as key_code,
        data: b"*\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_MINUS as ::core::ffi::c_ulong as key_code,
        data: b"-\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_SEVEN as ::core::ffi::c_ulong as key_code,
        data: b"7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_EIGHT as ::core::ffi::c_ulong as key_code,
        data: b"8\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_NINE as ::core::ffi::c_ulong as key_code,
        data: b"9\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_PLUS as ::core::ffi::c_ulong as key_code,
        data: b"+\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_FOUR as ::core::ffi::c_ulong as key_code,
        data: b"4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_FIVE as ::core::ffi::c_ulong as key_code,
        data: b"5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_SIX as ::core::ffi::c_ulong as key_code,
        data: b"6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_ONE as ::core::ffi::c_ulong as key_code,
        data: b"1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_TWO as ::core::ffi::c_ulong as key_code,
        data: b"2\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_THREE as ::core::ffi::c_ulong as key_code,
        data: b"3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_ENTER as ::core::ffi::c_ulong as key_code,
        data: b"\n\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_ZERO as ::core::ffi::c_ulong as key_code,
        data: b"0\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_KP_PERIOD as ::core::ffi::c_ulong as key_code,
        data: b".\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[1;_P\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[1;_Q\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[1;_R\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[1;_S\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[15;_~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[17;_~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[18;_~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[19;_~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[20;_~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[21;_~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[23;_~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[24;_~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[1;_A\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[1;_B\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[1;_C\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[1;_D\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[1;_H\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_END as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[1;_F\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[5;_~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[6;_~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_IC as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[2;_~\0" as *const u8 as *const ::core::ffi::c_char,
    },
    input_key_entry {
        key: KEYC_DC as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[3;_~\0" as *const u8 as *const ::core::ffi::c_char,
    },
];
static mut input_key_modifiers: [key_code; 9] = [
    0 as ::core::ffi::c_int as key_code,
    0 as ::core::ffi::c_int as key_code,
    KEYC_SHIFT,
    KEYC_META | KEYC_IMPLIED_META,
    KEYC_SHIFT | KEYC_META | KEYC_IMPLIED_META,
    KEYC_CTRL,
    KEYC_SHIFT | KEYC_CTRL,
    KEYC_META | KEYC_IMPLIED_META | KEYC_CTRL,
    KEYC_SHIFT | KEYC_META | KEYC_IMPLIED_META | KEYC_CTRL,
];
unsafe fn input_key_get(mut key: key_code) -> *mut input_key_entry {
    return input_key_tree_find(&*(&raw const input_key_tree), key);
}
unsafe fn input_key_split2(mut c: u_int, mut dst: *mut u_char) -> size_t {
    if c > 0x7f as u_int {
        *dst.offset(0 as ::core::ffi::c_int as isize) =
            (c >> 6 as ::core::ffi::c_int | 0xc0 as u_int) as u_char;
        *dst.offset(1 as ::core::ffi::c_int as isize) =
            (c & 0x3f as u_int | 0x80 as u_int) as u_char;
        return 2 as size_t;
    }
    *dst.offset(0 as ::core::ffi::c_int as isize) = c as u_char;
    return 1 as size_t;
}
pub unsafe fn input_key_build() {
    let mut ike: *mut input_key_entry = ::core::ptr::null_mut::<input_key_entry>();
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut key: key_code = 0;
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[input_key_entry; 85]>() as usize)
            .wrapping_div(::core::mem::size_of::<input_key_entry>() as usize)
    {
        ike = (&raw mut input_key_defaults as *mut input_key_entry).offset(i as isize)
            as *mut input_key_entry;
        if !((*ike).key as ::core::ffi::c_ulonglong) & KEYC_BUILD_MODIFIERS != 0 {
            input_key_tree_insert(&mut *(&raw mut input_key_tree), &mut *ike);
        } else {
            j = 2 as u_int;
            while (j as usize)
                < (::core::mem::size_of::<[key_code; 9]>() as usize)
                    .wrapping_div(::core::mem::size_of::<key_code>() as usize)
            {
                key = ((*ike).key as ::core::ffi::c_ulonglong & !KEYC_BUILD_MODIFIERS) as key_code;
                let generated = input_key_generated(
                    CStr::from_ptr((*ike).data),
                    key | input_key_modifiers[j as usize],
                    j,
                );
                input_key_tree_insert_generated(&mut *(&raw mut input_key_tree), generated);
                j = j.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
    ike = input_key_tree_minmax(&*(&raw const input_key_tree));
    while !ike.is_null() {
        let key_string = key_string_format((*ike).key, true);
        log_debug(format_args!(
            "{}: 0x{:x} ({}) is {}",
            "input_key_build",
            ((*ike).key) as u64,
            log_cstr((key_string.as_ptr()) as *const _),
            log_cstr(((*ike).data) as *const _)
        ));
        ike = input_key_tree_next(&*(&raw const input_key_tree), &*ike);
    }
}
pub unsafe fn input_key_pane(
    mut wp: *mut window_pane,
    mut key: key_code,
    mut m: *mut mouse_event,
) -> ::core::ffi::c_int {
    if log_get_level() != 0 as ::core::ffi::c_int {
        let key_string = key_string_format(key, true);
        log_debug(format_args!(
            "writing key 0x{:x} ({}) to %{}",
            key,
            log_cstr((key_string.as_ptr()) as *const _),
            ((*wp).id) as u32
        ));
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
    {
        if !m.is_null() && (*m).wp != -(1 as ::core::ffi::c_int) && (*m).wp as u_int == (*wp).id {
            input_key_mouse(wp, m);
        }
        return 0 as ::core::ffi::c_int;
    }
    return input_key((*wp).screen, (*wp).event, key);
}
unsafe fn input_key_write(
    mut from: *const ::core::ffi::c_char,
    mut bev: *mut bufferevent,
    mut data: *const ::core::ffi::c_char,
    mut size: size_t,
) {
    log_debug(format_args!(
        "{}: {}",
        log_cstr((from) as *const _),
        log_cstr_n((data) as *const _, size as ::core::ffi::c_int)
    ));
    bufferevent_write(bev, data as *const ::core::ffi::c_void, size);
}
unsafe fn input_key_extended(mut bev: *mut bufferevent, mut key: key_code) -> ::core::ffi::c_int {
    let mut tmp: [::core::ffi::c_char; 64] = [0; 64];
    let mut modifier: ::core::ffi::c_char = 0;
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut wc: wchar_t = 0;
    match key as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS {
        KEYC_SHIFT => {
            modifier = '2' as i32 as ::core::ffi::c_char;
        }
        KEYC_META => {
            modifier = '3' as i32 as ::core::ffi::c_char;
        }
        87960930222080 => {
            modifier = '4' as i32 as ::core::ffi::c_char;
        }
        KEYC_CTRL => {
            modifier = '5' as i32 as ::core::ffi::c_char;
        }
        105553116266496 => {
            modifier = '6' as i32 as ::core::ffi::c_char;
        }
        52776558133248 => {
            modifier = '7' as i32 as ::core::ffi::c_char;
        }
        123145302310912 => {
            modifier = '8' as i32 as ::core::ffi::c_char;
        }
        _ => return -(1 as ::core::ffi::c_int),
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
        == (KEYC_TYPE_UNICODE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
            << 32 as ::core::ffi::c_int
        && key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY > 0x7f as ::core::ffi::c_ulonglong
    {
        utf8_to_data(
            (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY) as utf8_char,
            &mut ud,
        );
        if utf8_towc(&raw mut ud, &raw mut wc) as ::core::ffi::c_uint
            == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            key = wc as key_code;
        } else {
            return -(1 as ::core::ffi::c_int);
        }
    } else {
        key &= KEYC_MASK_KEY;
    }
    if options_get_number(
        global_options,
        b"extended-keys-format\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 1 as ::core::ffi::c_longlong
    {
        xformat_with(&mut tmp, |out| {
            out.write_all(b"\x1B[27;")?;
            out.write_all(&[modifier as u8])?;
            write!(out, ";{}~", key)
        });
    } else {
        xformat_with(&mut tmp, |out| {
            write!(out, "\x1B[{};", key)?;
            out.write_all(&[modifier as u8])?;
            out.write_all(b"u")
        });
    }
    input_key_write(
        b"input_key_extended\0" as *const u8 as *const ::core::ffi::c_char,
        bev,
        &raw mut tmp as *mut ::core::ffi::c_char,
        strlen(&raw mut tmp as *mut ::core::ffi::c_char),
    );
    return 0 as ::core::ffi::c_int;
}
unsafe fn input_key_vt10x(mut bev: *mut bufferevent, mut key: key_code) -> ::core::ffi::c_int {
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut onlykey: key_code = 0;
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    static mut standard_map: [*const ::core::ffi::c_char; 2] = [
        b"1!9(0)=+;:'\",<.>/-8? 2\0" as *const u8 as *const ::core::ffi::c_char,
        b"119900=+;;'',,..\x1F\x1F\x7F\x7F\0\0\0" as *const u8 as *const ::core::ffi::c_char,
    ];
    log_debug(format_args!("{}: key in {:x}", "input_key_vt10x", key));
    if key as ::core::ffi::c_ulonglong & KEYC_META != 0 {
        input_key_write(
            b"input_key_vt10x\0" as *const u8 as *const ::core::ffi::c_char,
            bev,
            b"\x1B\0" as *const u8 as *const ::core::ffi::c_char,
            1 as size_t,
        );
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
        == (KEYC_TYPE_UNICODE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
            << 32 as ::core::ffi::c_int
        && key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY > 0x7f as ::core::ffi::c_ulonglong
    {
        utf8_to_data(key as utf8_char, &mut ud);
        input_key_write(
            b"input_key_vt10x\0" as *const u8 as *const ::core::ffi::c_char,
            bev,
            &raw mut ud.data as *mut u_char as *const ::core::ffi::c_char,
            ud.size as size_t,
        );
        return 0 as ::core::ffi::c_int;
    }
    onlykey = (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY) as key_code;
    if onlykey == '\r' as i32 as key_code
        || onlykey == '\n' as i32 as key_code
        || onlykey == '\t' as i32 as key_code
    {
        key &= !KEYC_CTRL;
    }
    if key as ::core::ffi::c_ulonglong & KEYC_CTRL != 0 {
        p = strchr(
            standard_map[0 as ::core::ffi::c_int as usize],
            onlykey as ::core::ffi::c_int,
        );
        if !p.is_null() {
            key = *standard_map[1 as ::core::ffi::c_int as usize].offset(
                p.offset_from(standard_map[0 as ::core::ffi::c_int as usize]) as ::core::ffi::c_long
                    as isize,
            ) as key_code;
        } else if onlykey >= '3' as i32 as key_code && onlykey <= '7' as i32 as key_code {
            key = onlykey.wrapping_sub('\u{18}' as i32 as key_code);
        } else if onlykey >= '@' as i32 as key_code && onlykey <= '~' as i32 as key_code {
            key = onlykey & 0x1f as key_code;
        } else {
            return -(1 as ::core::ffi::c_int);
        }
    }
    log_debug(format_args!("{}: key out {:x}", "input_key_vt10x", key));
    ud.data[0 as ::core::ffi::c_int as usize] = (key & 0x7f as key_code) as u_char;
    input_key_write(
        b"input_key_vt10x\0" as *const u8 as *const ::core::ffi::c_char,
        bev,
        (&raw mut ud.data as *mut u_char).offset(0 as ::core::ffi::c_int as isize) as *mut u_char
            as *const ::core::ffi::c_char,
        1 as size_t,
    );
    return 0 as ::core::ffi::c_int;
}
unsafe fn input_key_mode1(mut bev: *mut bufferevent, mut key: key_code) -> ::core::ffi::c_int {
    let mut onlykey: key_code = 0;
    log_debug(format_args!("{}: key in {:x}", "input_key_mode1", key));
    if key as ::core::ffi::c_ulonglong & (KEYC_CTRL | KEYC_META) == KEYC_META {
        return input_key_vt10x(bev, key);
    }
    onlykey = (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY) as key_code;
    if key as ::core::ffi::c_ulonglong & KEYC_CTRL != 0
        && (onlykey == ' ' as i32 as key_code
            || onlykey == '/' as i32 as key_code
            || onlykey == '@' as i32 as key_code
            || onlykey == '^' as i32 as key_code
            || onlykey >= '2' as i32 as key_code && onlykey <= '8' as i32 as key_code
            || onlykey >= '@' as i32 as key_code && onlykey <= '~' as i32 as key_code)
    {
        return input_key_vt10x(bev, key);
    }
    return -(1 as ::core::ffi::c_int);
}
pub unsafe fn input_key(
    mut s: *mut screen,
    mut bev: *mut bufferevent,
    mut key: key_code,
) -> ::core::ffi::c_int {
    let mut ike: *mut input_key_entry = ::core::ptr::null_mut::<input_key_entry>();
    let mut newkey: key_code = 0;
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if key as ::core::ffi::c_ulonglong & KEYC_LITERAL != 0 {
        ud.data[0 as ::core::ffi::c_int as usize] = key as u_char;
        input_key_write(
            b"input_key\0" as *const u8 as *const ::core::ffi::c_char,
            bev,
            (&raw mut ud.data as *mut u_char).offset(0 as ::core::ffi::c_int as isize)
                as *mut u_char as *const ::core::ffi::c_char,
            1 as size_t,
        );
        return 0 as ::core::ffi::c_int;
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_BSPACE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
    {
        newkey = options_get_number(
            global_options,
            b"backspace\0" as *const u8 as *const ::core::ffi::c_char,
        ) as key_code;
        log_debug(format_args!(
            "{}: key 0x{:x} is backspace -> 0x{:x}",
            "input_key",
            key,
            (newkey) as u64
        ));
        if key as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS == 0 as ::core::ffi::c_ulonglong {
            ud.data[0 as ::core::ffi::c_int as usize] = 255 as u_char;
            if newkey as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS
                == 0 as ::core::ffi::c_ulonglong
            {
                ud.data[0 as ::core::ffi::c_int as usize] = newkey as u_char;
            } else if newkey as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS == KEYC_CTRL {
                newkey &= KEYC_MASK_KEY;
                if newkey == '?' as i32 as key_code {
                    ud.data[0 as ::core::ffi::c_int as usize] = 0x7f as u_char;
                } else if newkey >= '@' as i32 as key_code && newkey <= '_' as i32 as key_code {
                    ud.data[0 as ::core::ffi::c_int as usize] =
                        newkey.wrapping_sub(0x40 as key_code) as u_char;
                } else if newkey >= 'a' as i32 as key_code && newkey <= 'z' as i32 as key_code {
                    ud.data[0 as ::core::ffi::c_int as usize] =
                        newkey.wrapping_sub(0x60 as key_code) as u_char;
                }
            }
            if ud.data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                != 255 as ::core::ffi::c_int
            {
                input_key_write(
                    b"input_key\0" as *const u8 as *const ::core::ffi::c_char,
                    bev,
                    (&raw mut ud.data as *mut u_char).offset(0 as ::core::ffi::c_int as isize)
                        as *mut u_char as *const ::core::ffi::c_char,
                    1 as size_t,
                );
            }
            return 0 as ::core::ffi::c_int;
        }
        key = (newkey as ::core::ffi::c_ulonglong
            | key as ::core::ffi::c_ulonglong & (KEYC_MASK_FLAGS | KEYC_MASK_MODIFIERS))
            as key_code;
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_BTAB as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
    {
        if (*s).mode & MODE_KEYS_EXTENDED_2 != 0 {
            key = ('\t' as i32 as ::core::ffi::c_ulonglong
                | key as ::core::ffi::c_ulonglong & !KEYC_MASK_KEY
                | KEYC_SHIFT) as key_code;
        } else {
            key &= !KEYC_MASK_MODIFIERS;
        }
    }
    if key as ::core::ffi::c_ulonglong & !KEYC_MASK_KEY == 0 {
        if key == C0_HT as ::core::ffi::c_int as key_code
            || key == C0_CR as ::core::ffi::c_int as key_code
            || key == C0_ESC as ::core::ffi::c_int as key_code
            || key >= 0x20 as key_code && key <= 0x7f as key_code
        {
            ud.data[0 as ::core::ffi::c_int as usize] = key as u_char;
            input_key_write(
                b"input_key\0" as *const u8 as *const ::core::ffi::c_char,
                bev,
                (&raw mut ud.data as *mut u_char).offset(0 as ::core::ffi::c_int as isize)
                    as *mut u_char as *const ::core::ffi::c_char,
                1 as size_t,
            );
            return 0 as ::core::ffi::c_int;
        }
        if key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            == (KEYC_TYPE_UNICODE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY > 0x7f as ::core::ffi::c_ulonglong
        {
            utf8_to_data(key as utf8_char, &mut ud);
            input_key_write(
                b"input_key\0" as *const u8 as *const ::core::ffi::c_char,
                bev,
                &raw mut ud.data as *mut u_char as *const ::core::ffi::c_char,
                ud.size as size_t,
            );
            return 0 as ::core::ffi::c_int;
        }
    }
    if !(*s).mode & MODE_KKEYPAD != 0 {
        key &= !KEYC_KEYPAD;
    }
    if !(*s).mode & MODE_KCURSOR != 0 {
        key &= !KEYC_CURSOR;
    }
    if ike.is_null() {
        ike = input_key_get(key);
    }
    if ike.is_null()
        && key as ::core::ffi::c_ulonglong & KEYC_META != 0
        && !(key as ::core::ffi::c_ulonglong) & KEYC_IMPLIED_META != 0
    {
        ike = input_key_get(key & !KEYC_META);
    }
    if ike.is_null() && key as ::core::ffi::c_ulonglong & KEYC_CURSOR != 0 {
        ike = input_key_get(key & !KEYC_CURSOR);
    }
    if ike.is_null() && key as ::core::ffi::c_ulonglong & KEYC_KEYPAD != 0 {
        ike = input_key_get(key & !KEYC_KEYPAD);
    }
    if !ike.is_null() {
        log_debug(format_args!(
            "{}: found key 0x{:x}: \"{}\"",
            "input_key",
            key,
            log_cstr(((*ike).data) as *const _)
        ));
        if key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            == (KEYC_TYPE_FUNCTION as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                == KEYC_PASTE_START as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                || key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                    == KEYC_PASTE_END as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong)
            && !(*s).mode & MODE_BRACKETPASTE != 0
        {
            return 0 as ::core::ffi::c_int;
        }
        if key as ::core::ffi::c_ulonglong & KEYC_META != 0
            && !(key as ::core::ffi::c_ulonglong) & KEYC_IMPLIED_META != 0
        {
            input_key_write(
                b"input_key\0" as *const u8 as *const ::core::ffi::c_char,
                bev,
                b"\x1B\0" as *const u8 as *const ::core::ffi::c_char,
                1 as size_t,
            );
        }
        input_key_write(
            b"input_key\0" as *const u8 as *const ::core::ffi::c_char,
            bev,
            (*ike).data,
            strlen((*ike).data),
        );
        return 0 as ::core::ffi::c_int;
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
        == (KEYC_TYPE_USER as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
            << 32 as ::core::ffi::c_int
        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            == (KEYC_TYPE_FUNCTION as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
        || (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
            == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
            || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                    <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                        << 32 as ::core::ffi::c_int)
    {
        log_debug(format_args!("{}: ignoring key 0x{:x}", "input_key", key));
        return 0 as ::core::ffi::c_int;
    }
    match (*s).mode & EXTENDED_KEY_MODES {
        MODE_KEYS_EXTENDED_2 => return input_key_extended(bev, key),
        MODE_KEYS_EXTENDED => {
            if input_key_mode1(bev, key) == -(1 as ::core::ffi::c_int) {
                return input_key_extended(bev, key);
            }
            return 0 as ::core::ffi::c_int;
        }
        _ => return input_key_vt10x(bev, key),
    };
}
/// Encode into caller-owned storage. Only the returned byte count is sent.
pub unsafe fn input_key_get_mouse(
    mut s: *mut screen,
    mut m: *mut mouse_event,
    mut x: u_int,
    mut y: u_int,
    buf: &mut [::core::ffi::c_char; 40],
) -> Option<size_t> {
    let mut len: size_t = 0;
    if (*m).b & MOUSE_MASK_DRAG as u_int != 0
        && (*s).mode & MOTION_MOUSE_MODES == 0 as ::core::ffi::c_int
    {
        return None;
    }
    if (*s).mode & ALL_MOUSE_MODES == 0 as ::core::ffi::c_int {
        return None;
    }
    if (*m).sgr_type != ' ' as i32 as u_int {
        if (*m).sgr_b & MOUSE_MASK_DRAG as u_int != 0
            && (*m).sgr_b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
            && !(*s).mode & MODE_MOUSE_ALL != 0
        {
            return None;
        }
    } else if (*m).b & MOUSE_MASK_DRAG as u_int != 0
        && (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
        && (*m).lb & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
        && !(*s).mode & MODE_MOUSE_ALL != 0
    {
        return None;
    }
    if (*m).sgr_type != ' ' as i32 as u_int && (*s).mode & MODE_MOUSE_SGR != 0 {
        len = xformat_with(buf, |out| {
            write!(
                out,
                "\x1B[<{};{};{}",
                ((*m).sgr_b) as u32,
                (x.wrapping_add(1 as u_int)) as u32,
                (y.wrapping_add(1 as u_int)) as u32
            )?;
            out.write_all(&[((*m).sgr_type) as u8])
        }) as size_t;
    } else if (*s).mode & MODE_MOUSE_UTF8 != 0 {
        if (*m).b > (MOUSE_PARAM_UTF8_MAX - MOUSE_PARAM_BTN_OFF) as u_int
            || x > (MOUSE_PARAM_UTF8_MAX - MOUSE_PARAM_POS_OFF) as u_int
            || y > (MOUSE_PARAM_UTF8_MAX - MOUSE_PARAM_POS_OFF) as u_int
        {
            return None;
        }
        len = xformat(buf, format_args!("\x1B[M")) as size_t;
        len = len.wrapping_add(input_key_split2(
            (*m).b.wrapping_add(MOUSE_PARAM_BTN_OFF as u_int),
            buf.as_mut_ptr().offset(len as isize) as *mut ::core::ffi::c_char as *mut u_char,
        ));
        len = len.wrapping_add(input_key_split2(
            x.wrapping_add(MOUSE_PARAM_POS_OFF as u_int),
            buf.as_mut_ptr().offset(len as isize) as *mut ::core::ffi::c_char as *mut u_char,
        ));
        len = len.wrapping_add(input_key_split2(
            y.wrapping_add(MOUSE_PARAM_POS_OFF as u_int),
            buf.as_mut_ptr().offset(len as isize) as *mut ::core::ffi::c_char as *mut u_char,
        ));
    } else {
        if (*m).b.wrapping_add(MOUSE_PARAM_BTN_OFF as u_int) > MOUSE_PARAM_MAX as u_int {
            return None;
        }
        len = xformat(buf, format_args!("\x1B[M")) as size_t;
        let fresh0 = len;
        len = len.wrapping_add(1);
        buf[fresh0 as usize] =
            (*m).b.wrapping_add(MOUSE_PARAM_BTN_OFF as u_int) as ::core::ffi::c_char;
        if x.wrapping_add(MOUSE_PARAM_POS_OFF as u_int) > MOUSE_PARAM_MAX as u_int {
            let fresh1 = len;
            len = len.wrapping_add(1);
            buf[fresh1 as usize] = MOUSE_PARAM_MAX as ::core::ffi::c_char;
        } else {
            let fresh2 = len;
            len = len.wrapping_add(1);
            buf[fresh2 as usize] =
                x.wrapping_add(MOUSE_PARAM_POS_OFF as u_int) as ::core::ffi::c_char;
        }
        if y.wrapping_add(MOUSE_PARAM_POS_OFF as u_int) > MOUSE_PARAM_MAX as u_int {
            let fresh3 = len;
            len = len.wrapping_add(1);
            buf[fresh3 as usize] = MOUSE_PARAM_MAX as ::core::ffi::c_char;
        } else {
            let fresh4 = len;
            len = len.wrapping_add(1);
            buf[fresh4 as usize] =
                y.wrapping_add(MOUSE_PARAM_POS_OFF as u_int) as ::core::ffi::c_char;
        }
    }
    Some(len)
}
unsafe fn input_key_mouse(mut wp: *mut window_pane, mut m: *mut mouse_event) {
    let mut s: *mut screen = (*wp).screen;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut buf = [0; 40];
    if (*m).ignore != 0 || (*s).mode & ALL_MOUSE_MODES == 0 as ::core::ffi::c_int {
        return;
    }
    if cmd_mouse_at(wp, m, &raw mut x, &raw mut y, 0 as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        return;
    }
    if window_pane_is_visible(wp) == 0 {
        return;
    }
    let Some(len) = input_key_get_mouse(s, m, x, y, &mut buf) else {
        return;
    };
    log_debug(format_args!(
        "writing mouse {} to %{}",
        log_cstr_n(buf.as_ptr(), len as ::core::ffi::c_int),
        ((*wp).id) as u32
    ));
    input_key_write(
        b"input_key_mouse\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).event,
        buf.as_ptr(),
        len,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_entries_keep_stable_data_and_first_duplicate() {
        unsafe {
            let mut tree = input_key_tree::default();
            input_key_tree_insert_generated(&mut tree, input_key_generated(c"\x1b[1;_A", 7, 2));
            let first = input_key_tree_find(&tree, 7);
            assert_eq!(CStr::from_ptr((*first).data), c"\x1b[1;2A");

            for key in 10..110 {
                input_key_tree_insert_generated(
                    &mut tree,
                    input_key_generated(c"\x1b[1;_B", key, 3),
                );
            }
            input_key_tree_insert_generated(&mut tree, input_key_generated(c"\x1b[1;_C", 7, 4));
            assert_eq!(tree.generated.len(), 101);
            assert_eq!(input_key_tree_find(&tree, 7), first);
            assert_eq!(CStr::from_ptr((*first).data), c"\x1b[1;2A");
        }
    }
}
