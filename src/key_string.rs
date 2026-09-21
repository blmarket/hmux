use crate::src::ffi::libc::{__ctype_tolower_loc, free, sscanf, strcasecmp, strlen, wctomb};
pub use crate::src::shared::abi::__int32_t;
use crate::src::shared::abi::*;
pub use crate::src::shared::control_character::{
    control_character_code, C0_ASC, C0_BEL, C0_BS, C0_CAN, C0_CR, C0_DC1, C0_DC2, C0_DC3, C0_DC4,
    C0_DLE, C0_EM, C0_ENQ, C0_EOT, C0_ESC, C0_ETB, C0_ETX, C0_FF, C0_FS, C0_GS, C0_HT, C0_LF,
    C0_NAK, C0_NUL, C0_RS, C0_SI, C0_SO, C0_SOH, C0_STX, C0_SUB, C0_SYN, C0_US, C0_VT,
};
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
pub use crate::src::shared::utf8::wchar_t;
use crate::src::shared::utf8::*;
use crate::src::utf8::{utf8_append, utf8_from_data, utf8_fromcstr, utf8_open, utf8_to_data};
use std::ffi::{CStr, CString};

pub use crate::src::shared::key::key_code_enum as C2RustUnnamed_0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_1 {
    pub string: *const ::core::ffi::c_char,
    pub key: key_code,
}
#[inline]
unsafe extern "C" fn tolower(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_tolower_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}
pub const MB_LEN_MAX: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
static mut key_string_table: [C2RustUnnamed_1; 1379] = [
    C2RustUnnamed_1 {
        string: b"F1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"F2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"F3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"F4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"F5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"F6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"F7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"F8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"F9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"F10\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"F11\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"F12\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"IC\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_IC as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"Insert\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_IC as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"DC\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DC as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"Delete\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DC as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"Home\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"End\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_END as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"NPage\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"PageDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"PgDn\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"PPage\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"PageUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"PgUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"BTab\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_BTAB as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"Space\0" as *const u8 as *const ::core::ffi::c_char,
        key: ' ' as i32 as key_code,
    },
    C2RustUnnamed_1 {
        string: b"BSpace\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_BSPACE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[NUL]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_NUL as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[SOH]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_SOH as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[STX]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_STX as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[ETX]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_ETX as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[EOT]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_EOT as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[ENQ]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_ENQ as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[ASC]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_ASC as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[BEL]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_BEL as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[BS]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_BS as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"Tab\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_HT as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[LF]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_LF as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[VT]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_VT as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[FF]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_FF as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"Enter\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_CR as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[SO]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_SO as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[SI]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_SI as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[DLE]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_DLE as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[DC1]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_DC1 as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[DC2]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_DC2 as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[DC3]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_DC3 as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[DC4]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_DC4 as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[NAK]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_NAK as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[SYN]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_SYN as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[ETB]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_ETB as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[CAN]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_CAN as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[EM]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_EM as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[SUB]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_SUB as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"Escape\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_ESC as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[FS]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_FS as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[GS]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_GS as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[RS]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_RS as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"[US]\0" as *const u8 as *const ::core::ffi::c_char,
        key: C0_US as ::core::ffi::c_int as key_code,
    },
    C2RustUnnamed_1 {
        string: b"Up\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"Down\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"Left\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"Right\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_IMPLIED_META,
    },
    C2RustUnnamed_1 {
        string: b"KP/\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_SLASH as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    C2RustUnnamed_1 {
        string: b"KP*\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_STAR as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    C2RustUnnamed_1 {
        string: b"KP-\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_MINUS as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    C2RustUnnamed_1 {
        string: b"KP7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_SEVEN as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    C2RustUnnamed_1 {
        string: b"KP8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_EIGHT as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    C2RustUnnamed_1 {
        string: b"KP9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_NINE as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    C2RustUnnamed_1 {
        string: b"KP+\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_PLUS as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    C2RustUnnamed_1 {
        string: b"KP4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_FOUR as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    C2RustUnnamed_1 {
        string: b"KP5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_FIVE as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    C2RustUnnamed_1 {
        string: b"KP6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_SIX as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    C2RustUnnamed_1 {
        string: b"KP1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_ONE as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    C2RustUnnamed_1 {
        string: b"KP2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_TWO as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    C2RustUnnamed_1 {
        string: b"KP3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_THREE as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    C2RustUnnamed_1 {
        string: b"KPEnter\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_ENTER as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    C2RustUnnamed_1 {
        string: b"KP0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_ZERO as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    C2RustUnnamed_1 {
        string: b"KP.\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_PERIOD as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown1Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN1_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown1Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN1_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown1StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN1_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown1StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN1_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown1StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN1_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown1ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN1_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown1ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN1_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown1ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN1_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown1Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN1_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown1Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN1_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown1Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN1_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown1Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN1_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown1Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN1_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown1Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN1_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown1Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN1_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown1Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN1_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown1Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN1_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown1Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN1_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown1Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN1_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown1Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN1_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown2Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN2_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown2Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN2_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown2StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN2_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown2StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN2_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown2StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN2_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown2ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN2_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown2ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN2_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown2ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN2_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown2Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN2_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown2Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN2_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown2Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN2_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown2Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN2_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown2Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN2_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown2Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN2_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown2Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN2_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown2Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN2_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown2Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN2_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown2Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN2_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown2Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN2_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown2Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN2_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown3Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN3_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown3Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN3_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown3StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN3_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown3StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN3_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown3StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN3_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown3ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN3_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown3ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN3_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown3ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN3_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown3Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN3_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown3Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN3_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown3Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN3_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown3Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN3_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown3Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN3_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown3Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN3_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown3Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN3_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown3Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN3_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown3Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN3_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown3Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN3_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown3Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN3_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown3Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN3_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown6Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN6_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown6Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN6_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown6StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN6_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown6StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN6_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown6StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN6_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown6ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN6_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown6ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN6_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown6ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN6_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown6Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN6_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown6Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN6_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown6Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN6_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown6Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN6_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown6Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN6_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown6Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN6_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown6Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN6_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown6Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN6_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown6Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN6_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown6Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN6_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown6Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN6_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown6Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN6_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown7Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN7_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown7Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN7_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown7StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN7_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown7StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN7_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown7StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN7_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown7ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN7_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown7ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN7_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown7ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN7_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown7Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN7_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown7Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN7_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown7Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN7_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown7Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN7_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown7Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN7_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown7Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN7_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown7Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN7_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown7Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN7_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown7Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN7_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown7Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN7_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown7Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN7_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown7Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN7_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown8Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN8_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown8Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN8_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown8StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN8_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown8StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN8_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown8StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN8_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown8ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN8_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown8ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN8_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown8ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN8_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown8Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN8_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown8Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN8_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown8Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN8_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown8Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN8_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown8Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN8_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown8Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN8_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown8Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN8_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown8Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN8_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown8Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN8_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown8Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN8_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown8Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN8_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown8Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN8_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown9Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN9_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown9Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN9_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown9StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN9_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown9StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN9_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown9StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN9_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown9ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN9_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown9ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN9_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown9ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN9_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown9Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN9_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown9Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN9_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown9Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN9_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown9Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN9_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown9Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN9_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown9Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN9_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown9Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN9_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown9Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN9_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown9Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN9_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown9Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN9_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown9Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN9_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown9Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN9_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown10Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN10_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown10Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN10_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown10StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN10_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown10StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN10_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown10StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN10_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown10ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN10_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown10ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN10_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown10ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN10_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown10Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN10_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown10Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN10_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown10Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN10_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown10Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN10_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown10Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN10_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown10Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN10_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown10Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN10_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown10Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN10_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown10Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN10_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown10Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN10_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown10Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN10_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown10Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN10_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown11Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN11_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown11Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN11_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown11StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN11_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown11StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN11_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown11StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN11_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown11ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN11_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown11ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN11_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown11ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN11_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown11Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN11_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown11Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN11_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown11Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN11_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown11Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN11_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown11Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN11_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown11Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN11_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown11Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN11_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown11Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN11_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown11Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN11_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown11Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN11_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown11Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN11_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDown11Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDOWN11_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp1Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP1_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp1Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP1_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp1StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP1_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp1StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP1_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp1StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP1_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp1ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP1_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp1ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP1_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp1ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP1_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp1Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP1_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp1Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP1_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp1Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP1_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp1Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP1_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp1Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP1_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp1Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP1_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp1Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP1_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp1Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP1_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp1Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP1_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp1Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP1_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp1Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP1_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp1Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP1_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp2Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP2_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp2Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP2_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp2StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP2_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp2StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP2_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp2StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP2_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp2ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP2_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp2ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP2_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp2ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP2_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp2Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP2_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp2Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP2_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp2Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP2_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp2Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP2_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp2Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP2_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp2Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP2_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp2Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP2_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp2Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP2_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp2Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP2_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp2Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP2_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp2Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP2_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp2Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP2_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp3Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP3_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp3Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP3_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp3StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP3_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp3StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP3_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp3StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP3_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp3ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP3_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp3ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP3_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp3ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP3_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp3Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP3_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp3Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP3_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp3Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP3_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp3Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP3_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp3Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP3_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp3Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP3_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp3Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP3_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp3Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP3_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp3Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP3_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp3Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP3_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp3Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP3_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp3Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP3_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp6Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP6_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp6Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP6_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp6StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP6_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp6StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP6_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp6StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP6_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp6ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP6_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp6ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP6_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp6ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP6_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp6Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP6_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp6Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP6_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp6Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP6_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp6Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP6_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp6Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP6_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp6Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP6_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp6Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP6_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp6Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP6_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp6Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP6_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp6Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP6_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp6Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP6_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp6Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP6_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp7Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP7_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp7Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP7_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp7StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP7_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp7StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP7_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp7StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP7_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp7ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP7_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp7ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP7_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp7ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP7_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp7Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP7_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp7Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP7_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp7Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP7_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp7Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP7_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp7Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP7_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp7Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP7_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp7Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP7_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp7Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP7_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp7Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP7_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp7Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP7_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp7Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP7_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp7Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP7_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp8Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP8_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp8Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP8_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp8StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP8_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp8StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP8_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp8StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP8_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp8ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP8_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp8ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP8_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp8ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP8_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp8Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP8_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp8Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP8_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp8Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP8_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp8Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP8_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp8Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP8_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp8Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP8_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp8Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP8_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp8Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP8_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp8Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP8_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp8Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP8_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp8Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP8_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp8Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP8_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp9Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP9_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp9Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP9_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp9StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP9_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp9StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP9_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp9StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP9_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp9ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP9_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp9ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP9_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp9ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP9_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp9Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP9_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp9Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP9_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp9Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP9_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp9Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP9_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp9Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP9_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp9Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP9_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp9Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP9_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp9Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP9_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp9Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP9_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp9Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP9_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp9Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP9_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp9Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP9_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp10Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP10_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp10Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP10_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp10StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP10_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp10StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP10_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp10StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP10_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp10ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP10_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp10ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP10_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp10ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP10_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp10Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP10_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp10Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP10_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp10Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP10_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp10Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP10_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp10Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP10_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp10Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP10_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp10Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP10_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp10Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP10_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp10Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP10_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp10Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP10_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp10Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP10_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp10Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP10_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp11Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP11_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp11Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP11_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp11StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP11_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp11StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP11_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp11StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP11_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp11ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP11_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp11ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP11_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp11ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP11_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp11Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP11_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp11Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP11_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp11Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP11_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp11Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP11_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp11Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP11_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp11Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP11_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp11Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP11_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp11Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP11_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp11Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP11_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp11Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP11_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp11Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP11_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseUp11Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEUP11_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag1Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG1_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag1Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG1_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag1StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG1_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag1StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG1_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag1StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG1_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag1ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG1_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag1ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG1_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag1ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG1_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag1Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG1_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag1Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG1_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag1Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG1_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag1Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG1_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag1Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG1_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag1Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG1_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag1Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG1_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag1Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG1_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag1Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG1_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag1Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG1_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag1Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG1_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag1Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG1_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag2Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG2_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag2Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG2_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag2StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG2_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag2StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG2_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag2StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG2_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag2ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG2_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag2ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG2_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag2ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG2_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag2Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG2_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag2Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG2_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag2Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG2_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag2Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG2_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag2Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG2_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag2Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG2_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag2Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG2_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag2Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG2_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag2Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG2_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag2Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG2_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag2Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG2_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag2Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG2_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag3Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG3_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag3Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG3_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag3StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG3_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag3StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG3_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag3StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG3_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag3ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG3_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag3ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG3_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag3ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG3_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag3Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG3_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag3Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG3_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag3Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG3_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag3Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG3_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag3Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG3_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag3Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG3_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag3Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG3_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag3Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG3_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag3Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG3_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag3Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG3_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag3Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG3_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag3Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG3_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag6Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG6_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag6Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG6_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag6StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG6_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag6StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG6_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag6StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG6_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag6ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG6_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag6ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG6_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag6ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG6_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag6Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG6_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag6Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG6_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag6Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG6_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag6Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG6_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag6Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG6_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag6Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG6_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag6Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG6_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag6Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG6_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag6Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG6_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag6Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG6_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag6Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG6_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag6Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG6_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag7Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG7_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag7Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG7_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag7StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG7_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag7StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG7_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag7StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG7_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag7ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG7_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag7ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG7_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag7ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG7_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag7Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG7_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag7Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG7_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag7Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG7_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag7Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG7_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag7Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG7_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag7Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG7_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag7Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG7_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag7Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG7_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag7Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG7_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag7Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG7_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag7Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG7_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag7Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG7_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag8Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG8_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag8Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG8_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag8StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG8_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag8StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG8_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag8StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG8_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag8ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG8_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag8ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG8_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag8ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG8_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag8Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG8_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag8Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG8_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag8Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG8_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag8Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG8_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag8Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG8_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag8Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG8_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag8Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG8_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag8Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG8_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag8Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG8_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag8Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG8_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag8Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG8_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag8Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG8_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag9Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG9_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag9Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG9_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag9StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG9_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag9StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG9_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag9StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG9_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag9ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG9_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag9ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG9_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag9ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG9_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag9Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG9_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag9Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG9_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag9Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG9_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag9Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG9_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag9Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG9_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag9Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG9_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag9Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG9_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag9Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG9_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag9Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG9_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag9Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG9_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag9Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG9_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag9Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG9_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag10Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG10_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag10Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG10_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag10StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG10_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag10StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG10_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag10StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG10_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag10ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG10_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag10ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG10_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag10ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG10_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag10Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG10_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag10Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG10_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag10Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG10_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag10Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG10_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag10Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG10_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag10Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG10_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag10Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG10_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag10Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG10_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag10Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG10_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag10Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG10_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag10Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG10_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag10Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG10_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag11Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG11_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag11Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG11_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag11StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG11_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag11StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG11_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag11StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG11_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag11ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG11_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag11ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG11_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag11ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG11_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag11Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG11_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag11Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG11_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag11Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG11_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag11Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG11_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag11Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG11_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag11Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG11_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag11Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG11_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag11Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG11_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag11Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG11_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag11Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG11_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag11Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG11_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDrag11Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAG11_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd1Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND1_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd1Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND1_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd1StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND1_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd1StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND1_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd1StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND1_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd1ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND1_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd1ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND1_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd1ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND1_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd1Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND1_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd1Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND1_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd1Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND1_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd1Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND1_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd1Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND1_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd1Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND1_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd1Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND1_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd1Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND1_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd1Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND1_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd1Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND1_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd1Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND1_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd1Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND1_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd2Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND2_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd2Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND2_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd2StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND2_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd2StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND2_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd2StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND2_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd2ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND2_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd2ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND2_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd2ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND2_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd2Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND2_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd2Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND2_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd2Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND2_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd2Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND2_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd2Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND2_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd2Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND2_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd2Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND2_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd2Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND2_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd2Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND2_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd2Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND2_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd2Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND2_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd2Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND2_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd3Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND3_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd3Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND3_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd3StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND3_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd3StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND3_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd3StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND3_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd3ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND3_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd3ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND3_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd3ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND3_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd3Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND3_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd3Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND3_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd3Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND3_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd3Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND3_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd3Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND3_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd3Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND3_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd3Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND3_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd3Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND3_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd3Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND3_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd3Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND3_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd3Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND3_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd3Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND3_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd6Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND6_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd6Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND6_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd6StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND6_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd6StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND6_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd6StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND6_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd6ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND6_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd6ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND6_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd6ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND6_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd6Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND6_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd6Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND6_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd6Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND6_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd6Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND6_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd6Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND6_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd6Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND6_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd6Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND6_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd6Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND6_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd6Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND6_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd6Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND6_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd6Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND6_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd6Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND6_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd7Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND7_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd7Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND7_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd7StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND7_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd7StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND7_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd7StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND7_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd7ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND7_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd7ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND7_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd7ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND7_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd7Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND7_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd7Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND7_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd7Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND7_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd7Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND7_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd7Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND7_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd7Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND7_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd7Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND7_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd7Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND7_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd7Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND7_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd7Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND7_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd7Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND7_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd7Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND7_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd8Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND8_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd8Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND8_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd8StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND8_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd8StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND8_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd8StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND8_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd8ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND8_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd8ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND8_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd8ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND8_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd8Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND8_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd8Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND8_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd8Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND8_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd8Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND8_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd8Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND8_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd8Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND8_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd8Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND8_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd8Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND8_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd8Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND8_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd8Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND8_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd8Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND8_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd8Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND8_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd9Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND9_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd9Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND9_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd9StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND9_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd9StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND9_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd9StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND9_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd9ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND9_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd9ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND9_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd9ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND9_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd9Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND9_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd9Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND9_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd9Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND9_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd9Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND9_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd9Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND9_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd9Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND9_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd9Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND9_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd9Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND9_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd9Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND9_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd9Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND9_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd9Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND9_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd9Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND9_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd10Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND10_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd10Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND10_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd10StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND10_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd10StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND10_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd10StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND10_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd10ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND10_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd10ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND10_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd10ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND10_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd10Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND10_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd10Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND10_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd10Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND10_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd10Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND10_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd10Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND10_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd10Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND10_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd10Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND10_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd10Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND10_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd10Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND10_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd10Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND10_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd10Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND10_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd10Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND10_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd11Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND11_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd11Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND11_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd11StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND11_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd11StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND11_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd11StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND11_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd11ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND11_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd11ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND11_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd11ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND11_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd11Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND11_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd11Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND11_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd11Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND11_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd11Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND11_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd11Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND11_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd11Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND11_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd11Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND11_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd11Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND11_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd11Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND11_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd11Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND11_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd11Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND11_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"MouseDragEnd11Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_MOUSEDRAGEND11_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelUpPane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELUP_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelUpStatus\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELUP_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelUpStatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELUP_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelUpStatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELUP_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelUpStatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELUP_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelUpScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELUP_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelUpScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELUP_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelUpScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELUP_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelUpEmpty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELUP_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelUpBorder\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELUP_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelUpControl0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELUP_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelUpControl1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELUP_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelUpControl2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELUP_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelUpControl3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELUP_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelUpControl4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELUP_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelUpControl5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELUP_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelUpControl6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELUP_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelUpControl7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELUP_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelUpControl8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELUP_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelUpControl9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELUP_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelDownPane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELDOWN_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelDownStatus\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELDOWN_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelDownStatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELDOWN_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelDownStatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELDOWN_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelDownStatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELDOWN_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelDownScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELDOWN_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelDownScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELDOWN_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelDownScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELDOWN_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelDownEmpty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELDOWN_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelDownBorder\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELDOWN_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelDownControl0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELDOWN_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelDownControl1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELDOWN_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelDownControl2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELDOWN_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelDownControl3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELDOWN_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelDownControl4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELDOWN_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelDownControl5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELDOWN_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelDownControl6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELDOWN_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelDownControl7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELDOWN_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelDownControl8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELDOWN_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"WheelDownControl9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_WHEELDOWN_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick1Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK1_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick1Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK1_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick1StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK1_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick1StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK1_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick1StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK1_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick1ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK1_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick1ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK1_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick1ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK1_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick1Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK1_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick1Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK1_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick1Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK1_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick1Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK1_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick1Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK1_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick1Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK1_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick1Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK1_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick1Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK1_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick1Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK1_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick1Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK1_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick1Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK1_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick1Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK1_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick2Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK2_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick2Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK2_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick2StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK2_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick2StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK2_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick2StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK2_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick2ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK2_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick2ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK2_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick2ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK2_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick2Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK2_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick2Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK2_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick2Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK2_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick2Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK2_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick2Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK2_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick2Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK2_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick2Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK2_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick2Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK2_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick2Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK2_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick2Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK2_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick2Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK2_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick2Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK2_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick3Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK3_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick3Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK3_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick3StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK3_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick3StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK3_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick3StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK3_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick3ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK3_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick3ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK3_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick3ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK3_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick3Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK3_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick3Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK3_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick3Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK3_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick3Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK3_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick3Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK3_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick3Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK3_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick3Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK3_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick3Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK3_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick3Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK3_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick3Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK3_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick3Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK3_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick3Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK3_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick6Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK6_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick6Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK6_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick6StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK6_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick6StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK6_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick6StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK6_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick6ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK6_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick6ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK6_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick6ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK6_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick6Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK6_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick6Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK6_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick6Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK6_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick6Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK6_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick6Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK6_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick6Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK6_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick6Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK6_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick6Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK6_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick6Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK6_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick6Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK6_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick6Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK6_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick6Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK6_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick7Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK7_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick7Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK7_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick7StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK7_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick7StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK7_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick7StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK7_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick7ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK7_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick7ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK7_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick7ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK7_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick7Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK7_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick7Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK7_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick7Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK7_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick7Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK7_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick7Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK7_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick7Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK7_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick7Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK7_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick7Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK7_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick7Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK7_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick7Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK7_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick7Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK7_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick7Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK7_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick8Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK8_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick8Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK8_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick8StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK8_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick8StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK8_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick8StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK8_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick8ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK8_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick8ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK8_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick8ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK8_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick8Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK8_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick8Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK8_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick8Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK8_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick8Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK8_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick8Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK8_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick8Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK8_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick8Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK8_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick8Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK8_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick8Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK8_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick8Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK8_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick8Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK8_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick8Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK8_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick9Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK9_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick9Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK9_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick9StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK9_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick9StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK9_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick9StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK9_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick9ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK9_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick9ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK9_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick9ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK9_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick9Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK9_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick9Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK9_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick9Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK9_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick9Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK9_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick9Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK9_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick9Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK9_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick9Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK9_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick9Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK9_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick9Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK9_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick9Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK9_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick9Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK9_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick9Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK9_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick10Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK10_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick10Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK10_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick10StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK10_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick10StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK10_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick10StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK10_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick10ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK10_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick10ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK10_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick10ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK10_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick10Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK10_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick10Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK10_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick10Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK10_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick10Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK10_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick10Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK10_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick10Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK10_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick10Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK10_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick10Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK10_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick10Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK10_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick10Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK10_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick10Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK10_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick10Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK10_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick11Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK11_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick11Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK11_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick11StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK11_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick11StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK11_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick11StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK11_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick11ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK11_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick11ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK11_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick11ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK11_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick11Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK11_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick11Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK11_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick11Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK11_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick11Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK11_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick11Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK11_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick11Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK11_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick11Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK11_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick11Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK11_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick11Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK11_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick11Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK11_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick11Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK11_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"SecondClick11Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_SECONDCLICK11_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick1Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK1_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick1Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK1_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick1StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK1_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick1StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK1_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick1StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK1_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick1ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK1_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick1ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK1_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick1ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK1_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick1Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK1_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick1Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK1_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick1Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK1_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick1Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK1_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick1Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK1_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick1Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK1_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick1Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK1_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick1Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK1_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick1Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK1_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick1Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK1_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick1Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK1_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick1Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK1_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick2Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK2_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick2Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK2_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick2StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK2_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick2StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK2_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick2StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK2_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick2ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK2_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick2ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK2_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick2ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK2_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick2Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK2_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick2Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK2_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick2Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK2_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick2Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK2_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick2Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK2_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick2Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK2_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick2Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK2_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick2Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK2_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick2Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK2_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick2Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK2_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick2Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK2_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick2Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK2_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick3Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK3_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick3Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK3_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick3StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK3_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick3StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK3_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick3StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK3_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick3ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK3_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick3ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK3_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick3ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK3_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick3Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK3_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick3Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK3_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick3Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK3_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick3Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK3_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick3Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK3_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick3Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK3_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick3Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK3_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick3Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK3_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick3Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK3_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick3Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK3_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick3Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK3_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick3Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK3_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick6Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK6_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick6Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK6_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick6StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK6_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick6StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK6_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick6StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK6_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick6ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK6_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick6ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK6_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick6ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK6_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick6Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK6_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick6Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK6_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick6Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK6_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick6Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK6_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick6Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK6_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick6Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK6_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick6Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK6_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick6Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK6_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick6Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK6_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick6Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK6_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick6Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK6_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick6Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK6_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick7Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK7_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick7Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK7_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick7StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK7_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick7StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK7_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick7StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK7_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick7ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK7_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick7ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK7_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick7ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK7_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick7Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK7_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick7Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK7_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick7Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK7_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick7Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK7_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick7Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK7_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick7Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK7_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick7Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK7_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick7Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK7_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick7Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK7_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick7Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK7_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick7Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK7_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick7Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK7_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick8Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK8_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick8Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK8_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick8StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK8_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick8StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK8_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick8StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK8_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick8ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK8_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick8ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK8_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick8ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK8_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick8Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK8_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick8Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK8_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick8Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK8_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick8Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK8_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick8Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK8_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick8Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK8_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick8Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK8_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick8Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK8_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick8Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK8_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick8Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK8_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick8Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK8_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick8Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK8_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick9Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK9_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick9Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK9_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick9StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK9_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick9StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK9_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick9StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK9_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick9ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK9_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick9ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK9_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick9ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK9_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick9Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK9_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick9Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK9_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick9Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK9_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick9Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK9_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick9Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK9_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick9Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK9_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick9Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK9_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick9Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK9_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick9Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK9_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick9Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK9_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick9Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK9_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick9Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK9_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick10Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK10_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick10Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK10_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick10StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK10_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick10StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK10_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick10StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK10_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick10ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK10_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick10ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK10_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick10ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK10_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick10Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK10_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick10Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK10_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick10Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK10_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick10Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK10_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick10Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK10_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick10Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK10_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick10Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK10_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick10Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK10_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick10Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK10_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick10Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK10_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick10Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK10_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick10Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK10_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick11Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK11_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick11Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK11_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick11StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK11_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick11StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK11_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick11StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK11_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick11ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK11_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick11ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK11_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick11ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK11_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick11Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK11_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick11Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK11_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick11Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK11_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick11Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK11_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick11Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK11_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick11Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK11_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick11Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK11_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick11Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK11_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick11Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK11_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick11Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK11_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick11Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK11_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"DoubleClick11Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOUBLECLICK11_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick1Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK1_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick1Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK1_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick1StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK1_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick1StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK1_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick1StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK1_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick1ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK1_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick1ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK1_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick1ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK1_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick1Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK1_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick1Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK1_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick1Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK1_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick1Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK1_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick1Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK1_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick1Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK1_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick1Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK1_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick1Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK1_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick1Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK1_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick1Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK1_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick1Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK1_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick1Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK1_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick2Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK2_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick2Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK2_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick2StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK2_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick2StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK2_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick2StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK2_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick2ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK2_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick2ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK2_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick2ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK2_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick2Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK2_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick2Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK2_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick2Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK2_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick2Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK2_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick2Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK2_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick2Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK2_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick2Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK2_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick2Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK2_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick2Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK2_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick2Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK2_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick2Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK2_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick2Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK2_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick3Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK3_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick3Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK3_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick3StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK3_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick3StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK3_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick3StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK3_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick3ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK3_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick3ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK3_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick3ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK3_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick3Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK3_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick3Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK3_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick3Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK3_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick3Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK3_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick3Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK3_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick3Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK3_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick3Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK3_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick3Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK3_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick3Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK3_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick3Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK3_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick3Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK3_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick3Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK3_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick6Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK6_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick6Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK6_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick6StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK6_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick6StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK6_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick6StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK6_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick6ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK6_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick6ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK6_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick6ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK6_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick6Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK6_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick6Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK6_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick6Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK6_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick6Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK6_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick6Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK6_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick6Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK6_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick6Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK6_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick6Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK6_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick6Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK6_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick6Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK6_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick6Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK6_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick6Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK6_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick7Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK7_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick7Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK7_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick7StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK7_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick7StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK7_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick7StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK7_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick7ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK7_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick7ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK7_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick7ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK7_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick7Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK7_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick7Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK7_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick7Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK7_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick7Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK7_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick7Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK7_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick7Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK7_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick7Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK7_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick7Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK7_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick7Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK7_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick7Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK7_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick7Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK7_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick7Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK7_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick8Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK8_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick8Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK8_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick8StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK8_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick8StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK8_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick8StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK8_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick8ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK8_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick8ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK8_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick8ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK8_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick8Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK8_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick8Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK8_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick8Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK8_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick8Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK8_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick8Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK8_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick8Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK8_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick8Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK8_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick8Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK8_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick8Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK8_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick8Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK8_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick8Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK8_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick8Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK8_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick9Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK9_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick9Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK9_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick9StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK9_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick9StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK9_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick9StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK9_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick9ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK9_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick9ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK9_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick9ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK9_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick9Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK9_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick9Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK9_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick9Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK9_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick9Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK9_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick9Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK9_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick9Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK9_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick9Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK9_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick9Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK9_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick9Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK9_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick9Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK9_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick9Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK9_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick9Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK9_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick10Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK10_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick10Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK10_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick10StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK10_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick10StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK10_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick10StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK10_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick10ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK10_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick10ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK10_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick10ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK10_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick10Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK10_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick10Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK10_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick10Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK10_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick10Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK10_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick10Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK10_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick10Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK10_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick10Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK10_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick10Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK10_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick10Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK10_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick10Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK10_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick10Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK10_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick10Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK10_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick11Pane\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK11_PANE as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick11Status\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK11_STATUS as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick11StatusLeft\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK11_STATUS_LEFT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick11StatusRight\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK11_STATUS_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick11StatusDefault\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK11_STATUS_DEFAULT as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick11ScrollbarUp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK11_SCROLLBAR_UP as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick11ScrollbarSlider\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK11_SCROLLBAR_SLIDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick11ScrollbarDown\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK11_SCROLLBAR_DOWN as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick11Empty\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK11_EMPTY as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick11Border\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK11_BORDER as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick11Control0\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK11_CONTROL0 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick11Control1\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK11_CONTROL1 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick11Control2\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK11_CONTROL2 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick11Control3\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK11_CONTROL3 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick11Control4\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK11_CONTROL4 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick11Control5\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK11_CONTROL5 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick11Control6\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK11_CONTROL6 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick11Control7\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK11_CONTROL7 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick11Control8\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK11_CONTROL8 as ::core::ffi::c_ulong as key_code,
    },
    C2RustUnnamed_1 {
        string: b"TripleClick11Control9\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_TRIPLECLICK11_CONTROL9 as ::core::ffi::c_ulong as key_code,
    },
];
unsafe extern "C" fn key_string_search_table(mut string: *const ::core::ffi::c_char) -> key_code {
    let mut i: u_int = 0;
    let mut user: u_int = 0;
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_1; 1379]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_1>() as usize)
    {
        if strcasecmp(string, key_string_table[i as usize].string) == 0 as ::core::ffi::c_int {
            return key_string_table[i as usize].key;
        }
        i = i.wrapping_add(1);
    }
    if sscanf(
        string,
        b"User%u\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut user,
    ) == 1 as ::core::ffi::c_int
        && user <= KEYC_NUSER as u_int
    {
        return (KEYC_USER as ::core::ffi::c_ulong).wrapping_add(user as ::core::ffi::c_ulong)
            as key_code;
    }
    return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
}
unsafe extern "C" fn key_string_get_modifiers(
    mut string: *mut *const ::core::ffi::c_char,
) -> key_code {
    let mut modifiers: key_code = 0;
    modifiers = 0 as key_code;
    while *(*string).offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
        && *(*string).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '-' as i32
    {
        match *(*string).offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            67 | 99 => {
                modifiers |= KEYC_CTRL;
            }
            77 | 109 => {
                modifiers |= KEYC_META;
            }
            83 | 115 => {
                modifiers |= KEYC_SHIFT;
            }
            _ => {
                *string = ::core::ptr::null::<::core::ffi::c_char>();
                return 0 as key_code;
            }
        }
        *string = (*string).offset(2 as ::core::ffi::c_int as isize);
    }
    return modifiers;
}
unsafe fn key_string_lookup_string_impl(mut string: *const ::core::ffi::c_char) -> key_code {
    let mut key: key_code = 0;
    let mut modifiers: key_code = 0 as key_code;
    let mut u: u_int = 0;
    let mut i: u_int = 0;
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut udp: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    let mut more: utf8_state = UTF8_MORE;
    let mut uc: utf8_char = 0;
    let mut m: [::core::ffi::c_char; 17] = [0; 17];
    let mut mlen: ::core::ffi::c_int = 0;
    if strcasecmp(string, b"None\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        return KEYC_NONE as ::core::ffi::c_ulong as key_code;
    }
    if strcasecmp(string, b"Any\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        return KEYC_ANY as ::core::ffi::c_ulong as key_code;
    }
    if *string.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '0' as i32
        && *string.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'x' as i32
    {
        if sscanf(
            string.offset(2 as ::core::ffi::c_int as isize),
            b"%x\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut u,
        ) != 1 as ::core::ffi::c_int
        {
            return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
        }
        if u < 32 as u_int {
            return u as key_code;
        }
        mlen = wctomb(&raw mut m as *mut ::core::ffi::c_char, u as wchar_t);
        if mlen <= 0 as ::core::ffi::c_int || mlen > MB_LEN_MAX {
            return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
        }
        m[mlen as usize] = '\0' as i32 as ::core::ffi::c_char;
        udp = utf8_fromcstr(&raw mut m as *mut ::core::ffi::c_char);
        if udp.is_null()
            || (*udp.offset(0 as ::core::ffi::c_int as isize)).size as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            || (*udp.offset(1 as ::core::ffi::c_int as isize)).size as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            || utf8_from_data(
                udp.offset(0 as ::core::ffi::c_int as isize) as *mut utf8_data,
                &raw mut uc,
            ) as ::core::ffi::c_uint
                != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            free(udp as *mut ::core::ffi::c_void);
            return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
        }
        free(udp as *mut ::core::ffi::c_void);
        return uc as key_code;
    }
    if *string.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '^' as i32
        && *string.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
    {
        if *string.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\0' as i32 {
            return ({
                let mut __res: ::core::ffi::c_int = 0;
                if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                    if 0 != 0 {
                        let mut __c: ::core::ffi::c_int =
                            *string.offset(1 as ::core::ffi::c_int as isize) as u_char
                                as ::core::ffi::c_int;
                        __res = (if __c < -(128 as ::core::ffi::c_int)
                            || __c > 255 as ::core::ffi::c_int
                        {
                            __c as __int32_t
                        } else {
                            *(*__ctype_tolower_loc()).offset(__c as isize)
                        }) as ::core::ffi::c_int;
                    } else {
                        __res = tolower(*string.offset(1 as ::core::ffi::c_int as isize) as u_char
                            as ::core::ffi::c_int);
                    }
                } else {
                    __res = *(*__ctype_tolower_loc())
                        .offset(*string.offset(1 as ::core::ffi::c_int as isize) as u_char
                            as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int;
                }
                __res
            }) as key_code
                | KEYC_CTRL;
        }
        modifiers |= KEYC_CTRL;
        string = string.offset(1);
    }
    modifiers |= key_string_get_modifiers(&raw mut string);
    if string.is_null()
        || *string.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\0' as i32
    {
        return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
    }
    if *string.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\0' as i32
        && *string.offset(0 as ::core::ffi::c_int as isize) as u_char as ::core::ffi::c_int
            <= 127 as ::core::ffi::c_int
    {
        key = *string.offset(0 as ::core::ffi::c_int as isize) as u_char as key_code;
        if key < 32 as key_code {
            return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
        }
    } else {
        more = utf8_open(&raw mut ud, *string as u_char);
        if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint {
            if strlen(string) != ud.size as size_t {
                return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
            }
            i = 1 as u_int;
            while i < ud.size as u_int {
                more = utf8_append(&raw mut ud, *string.offset(i as isize) as u_char);
                i = i.wrapping_add(1);
            }
            if more as ::core::ffi::c_uint != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
            }
            if utf8_from_data(&raw mut ud, &raw mut uc) as ::core::ffi::c_uint
                != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
            }
            return uc as key_code | modifiers;
        }
        key = key_string_search_table(string);
        if key == KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code {
            return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
        }
        if !(modifiers as ::core::ffi::c_ulonglong) & KEYC_META != 0 {
            key &= !KEYC_IMPLIED_META;
        }
    }
    return key | modifiers;
}
/// Parse a complete key name from bytes without converting it to UTF-8.
/// Embedded NUL bytes and invalid key names return `None`.
pub fn key_string_parse(input: &[u8]) -> Option<key_code> {
    let input = CString::new(input).ok()?;
    key_string_parse_cstr(&input)
}

/// Parse a NUL-terminated key name without retaining the input pointer.
pub fn key_string_parse_cstr(input: &CStr) -> Option<key_code> {
    if input.to_bytes().is_empty() {
        return None;
    }
    let key = unsafe { key_string_lookup_string_impl(input.as_ptr()) };
    (key != KEYC_UNKNOWN).then_some(key)
}

/// C ABI compatibility shim for the historical sentinel-returning parser.
///
/// # Safety
/// `string` must point to a readable NUL-terminated string for this call.
#[no_mangle]
pub unsafe extern "C" fn key_string_lookup_string(string: *const ::core::ffi::c_char) -> key_code {
    key_string_parse_cstr(CStr::from_ptr(string)).unwrap_or(KEYC_UNKNOWN)
}

/// Format canonical key text into a caller-owned NUL-terminated buffer.
///
/// The returned length excludes the trailing NUL. If `output` is too small,
/// this function returns `None` without modifying it. The output matches
/// [`key_string_format`].
pub fn key_string_format_into(key: key_code, with_flags: bool, output: &mut [u8]) -> Option<usize> {
    let formatted = key_string_format_bytes(key, with_flags);
    if output.len() <= formatted.len() {
        return None;
    }
    output[..formatted.len()].copy_from_slice(&formatted);
    output[formatted.len()] = 0;
    Some(formatted.len())
}

/// Format a canonical key name into an owned NUL-terminated byte string.
///
/// The result is independent of all later formatting calls.
pub fn key_string_format(key: key_code, with_flags: bool) -> CString {
    CString::new(key_string_format_bytes(key, with_flags))
        .expect("key names contain no embedded NUL")
}

fn key_string_named_name(key: key_code) -> Option<&'static [u8]> {
    match key {
        KEYC_NONE => Some(b"None"),
        KEYC_UNKNOWN => Some(b"Unknown"),
        KEYC_ANY => Some(b"Any"),
        KEYC_FOCUS_IN => Some(b"FocusIn"),
        KEYC_FOCUS_OUT => Some(b"FocusOut"),
        KEYC_PASTE_START => Some(b"PasteStart"),
        KEYC_PASTE_END => Some(b"PasteEnd"),
        KEYC_REPORT_DARK_THEME => Some(b"ReportDarkTheme"),
        KEYC_REPORT_LIGHT_THEME => Some(b"ReportLightTheme"),
        KEYC_MOUSE => Some(b"Mouse"),
        KEYC_DRAGGING => Some(b"Dragging"),
        KEYC_MOUSEMOVE_PANE => Some(b"MouseMovePane"),
        KEYC_MOUSEMOVE_STATUS => Some(b"MouseMoveStatus"),
        KEYC_MOUSEMOVE_STATUS_LEFT => Some(b"MouseMoveStatusLeft"),
        KEYC_MOUSEMOVE_STATUS_RIGHT => Some(b"MouseMoveStatusRight"),
        KEYC_MOUSEMOVE_BORDER => Some(b"MouseMoveBorder"),
        _ => None,
    }
}

fn key_string_table_name(key: key_code) -> Option<Vec<u8>> {
    unsafe {
        let table = ::core::ptr::addr_of!(key_string_table) as *const C2RustUnnamed_1;
        for index in 0..1379 {
            let entry = &*table.add(index);
            if entry.key & KEYC_MASK_KEY == key {
                return Some(CStr::from_ptr(entry.string).to_bytes().to_vec());
            }
        }
    }
    None
}

fn key_string_format_bytes(saved: key_code, with_flags: bool) -> Vec<u8> {
    let mut output = Vec::new();

    if saved & KEYC_LITERAL != 0 {
        let literal = (saved & 0xff) as u8;
        if literal != 0 {
            output.push(literal);
        }
    } else {
        if saved & KEYC_CTRL != 0 {
            output.extend_from_slice(b"C-");
        }
        if saved & KEYC_META != 0 {
            output.extend_from_slice(b"M-");
        }
        if saved & KEYC_SHIFT != 0 {
            output.extend_from_slice(b"S-");
        }

        let key = saved & KEYC_MASK_KEY;
        if let Some(name) = key_string_named_name(key) {
            output.extend_from_slice(name);
        } else if key & KEYC_MASK_TYPE == (KEYC_TYPE_USER as key_code) << 32 {
            output.extend_from_slice(format!("User{}", key.wrapping_sub(KEYC_USER)).as_bytes());
        } else if let Some(name) = key_string_table_name(key) {
            output.extend_from_slice(&name);
        } else if key & KEYC_MASK_TYPE == 0 && key > 0x7f {
            let mut data = utf8_data {
                data: [0; 32],
                have: 0,
                size: 0,
                width: 0,
            };
            unsafe {
                utf8_to_data(key as utf8_char, &raw mut data);
            }
            output.extend_from_slice(&data.data[..data.size as usize]);
        } else if key > 255 {
            // The historical snprintf branch replaced the modifier prefix
            // when it reported an invalid key. Preserve that canonical text.
            output.clear();
            output.extend_from_slice(format!("Invalid#{saved:x}").as_bytes());
        } else if key > 32 && key <= 126 {
            output.push(key as u8);
        } else if key == 127 {
            output.extend_from_slice(b"C-?");
        } else if key >= 128 {
            output.extend_from_slice(format!(r"\{:o}", key).as_bytes());
        }
    }

    // The legacy formatter wrote into a C string buffer. Invalid packed
    // Unicode can therefore place a NUL in the output and make the visible
    // result end there; normalize that terminator before the legacy flag
    // append, which would overwrite it with the suffix.
    if let Some(nul) = output.iter().position(|&byte| byte == 0) {
        output.truncate(nul);
    }

    if with_flags && saved & KEYC_MASK_FLAGS != 0 {
        output.push(b'[');
        if saved & KEYC_LITERAL != 0 {
            output.push(b'L');
        }
        if saved & KEYC_KEYPAD != 0 {
            output.push(b'K');
        }
        if saved & KEYC_CURSOR != 0 {
            output.push(b'C');
        }
        if saved & KEYC_IMPLIED_META != 0 {
            output.push(b'I');
        }
        if saved & KEYC_BUILD_MODIFIERS != 0 {
            output.push(b'B');
        }
        if saved & KEYC_SENT != 0 {
            output.push(b'S');
        }
        output.push(b']');
    }
    output
}

/// C ABI compatibility shim for callers that still require the historical symbol.
///
/// The legacy pointer remains valid until the next call to this shim on the same
/// thread or thread exit. Rust callers should retain key_string_format.
///
/// # Safety
/// The returned pointer must only be read before the next call to this shim on
/// the same thread, and must not be freed by the caller.
#[no_mangle]
pub unsafe extern "C" fn key_string_lookup_key(
    key: key_code,
    with_flags: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    thread_local! {
        static BUFFER: std::cell::RefCell<CString> = std::cell::RefCell::new(CString::default());
    }
    BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        *buffer = key_string_format(key, with_flags != 0);
        buffer.as_ptr()
    })
}
