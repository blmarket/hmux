use crate::src::compat::strtonum::strtonum;
use crate::src::compat::unvis::strunvis;
use crate::src::compat::vis::strnvis;
use crate::src::environ::environ_find;
use crate::src::ffi::libc::{
    fnmatch, memset, strcasecmp, strchr, strcmp, strcspn, strlen, strncmp, strstr,
};
use crate::src::ffi::ncurses::TERMINAL;
use crate::src::ffi::ncurses::{
    cur_term, del_curterm, setupterm, tigetflag, tigetnum, tigetstr, tiparm_s,
};
use crate::src::format::bytes::{format_message_with, xformat};
use crate::src::log::{fatalx, log_cstr, log_debug};
use crate::src::options::{
    options_array_first, options_array_item_value, options_array_next, options_get_only,
};
use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::environment::environ_entry;
use crate::src::shared::limits::INT_MAX;
use crate::src::shared::options::{options_array_item, options_entry, options_value};
use crate::src::shared::posix_io::STDIN_FILENO;
use crate::src::shared::tty::tty_terms;
use crate::src::shared::tty::*;
use crate::src::shared::tty::{tty, tty_code, tty_code_type, tty_term, tty_term_entry};
use crate::src::shared::tty::{
    TERM_DECFRA, TERM_DECSLRM, TERM_INVALIDMS, TERM_NOAM, TERM_RGBCOLOURS, TERM_SIXEL,
    TERM_VT100LIKE,
};
use crate::src::shared::vis::{VIS_CSTYLE, VIS_NL, VIS_OCTAL, VIS_TAB};
use crate::src::tmux::global_options;
use crate::src::tty_features::{tty_apply_features, tty_parse_client_features};
use std::ffi::{CStr, CString};

pub const TTYCODE_FLAG: tty_code_type = 3;
pub const TTYCODE_NUMBER: tty_code_type = 2;
pub const TTYCODE_STRING: tty_code_type = 1;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_term_code_entry {
    pub type_0: tty_code_type,
    pub name: &'static ::std::ffi::CStr,
}
pub const OK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub static mut tty_terms: tty_terms = tty_terms {
    lh_first: ::core::ptr::null::<tty_term>() as *mut tty_term,
};
static mut tty_term_codes: [tty_term_code_entry; 234] = [
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
        name: c"ind",
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
pub unsafe fn tty_term_ncodes() -> u_int {
    return (::core::mem::size_of::<[tty_term_code_entry; 234]>() as usize)
        .wrapping_div(::core::mem::size_of::<tty_term_code_entry>() as usize) as u_int;
}
fn tty_term_strip(s: &CStr) -> CString {
    let bytes = s.to_bytes();
    if !bytes.contains(&b'$') {
        return s.to_owned();
    }
    let mut stripped = Vec::with_capacity(8191);
    let mut offset = 0;
    while offset < bytes.len() {
        if bytes[offset] == b'$' && bytes.get(offset + 1) == Some(&b'<') {
            while offset < bytes.len() && bytes[offset] != b'>' {
                offset += 1;
            }
            if offset < bytes.len() {
                offset += 1;
            }
            if offset == bytes.len() {
                break;
            }
        }
        stripped.push(bytes[offset]);
        if stripped.len() == 8191 {
            break;
        }
        offset += 1;
    }
    CString::new(stripped).expect("terminal capability C string contains no NUL")
}
unsafe fn tty_term_override_next(
    mut s: *const ::core::ffi::c_char,
    mut offset: *mut size_t,
) -> Option<Vec<::core::ffi::c_char>> {
    let mut value = Vec::new();
    let mut n: size_t = 0 as size_t;
    let mut at: size_t = *offset;
    if *s.offset(at as isize) as ::core::ffi::c_int == '\0' as i32 {
        return None;
    }
    while *s.offset(at as isize) as ::core::ffi::c_int != '\0' as i32 {
        if *s.offset(at as isize) as ::core::ffi::c_int == ':' as i32 {
            if !(*s.offset(at.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                == ':' as i32)
            {
                break;
            }
            n = n.wrapping_add(1);
            value.push(':' as ::core::ffi::c_char);
            at = at.wrapping_add(2 as size_t);
        } else {
            n = n.wrapping_add(1);
            value.push(*s.offset(at as isize));
            at = at.wrapping_add(1);
        }
        if n == (::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as usize)
            .wrapping_sub(1 as usize)
        {
            return None;
        }
    }
    if *s.offset(at as isize) as ::core::ffi::c_int != '\0' as i32 {
        *offset = at.wrapping_add(1 as size_t);
    } else {
        *offset = at;
    }
    value.push(0);
    Some(value)
}
unsafe fn tty_term_override_value(source: &CStr) -> CString {
    let mut decoded = source.to_bytes_with_nul().to_vec();
    if strunvis(decoded.as_mut_ptr().cast(), source.as_ptr()) == -1 {
        return source.to_owned();
    }
    // strunvis may decode an escape to NUL. The old C string consumers saw
    // only the prefix through that byte, including when storing the value.
    decoded.truncate(strlen(decoded.as_ptr().cast()) + 1);
    CString::from_vec_with_nul(decoded).expect("strunvis terminates its output")
}
pub unsafe fn tty_term_apply(
    mut term: *mut tty_term,
    mut capabilities: *const ::core::ffi::c_char,
    mut quiet: ::core::ffi::c_int,
) {
    let mut ent: *const tty_term_code_entry = ::core::ptr::null::<tty_term_code_entry>();
    let mut offset: size_t = 0 as size_t;
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut name: *const ::core::ffi::c_char = ((*term).name).as_ptr().cast_mut();
    let mut i: u_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut remove: ::core::ffi::c_int = 0;
    while let Some(mut token) = tty_term_override_next(capabilities, &raw mut offset) {
        let s = token.as_mut_ptr();
        if *s as ::core::ffi::c_int == '\0' as i32 {
            continue;
        }
        remove = 0 as ::core::ffi::c_int;
        cp = strchr(s, '=' as i32);
        let value = if !cp.is_null() {
            let fresh0 = cp;
            cp = cp.offset(1);
            *fresh0 = '\0' as i32 as ::core::ffi::c_char;
            Some(tty_term_override_value(CStr::from_ptr(cp)))
        } else if *s.offset(strlen(s).wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
            == '@' as i32
        {
            *s.offset(strlen(s).wrapping_sub(1 as size_t) as isize) =
                '\0' as i32 as ::core::ffi::c_char;
            remove = 1 as ::core::ffi::c_int;
            None
        } else {
            Some(CString::default())
        };
        // The pointer only borrows the loop-local owner during logging and
        // capability updates. Removed capabilities never inspect it.
        let value = value.as_ref().map_or(::core::ptr::null(), |v| v.as_ptr());
        if quiet == 0 {
            if remove != 0 {
                log_debug(format_args!(
                    "{} override: {}@",
                    log_cstr((name) as *const _),
                    log_cstr((s) as *const _)
                ));
            } else if *value as ::core::ffi::c_int == '\0' as i32 {
                log_debug(format_args!(
                    "{} override: {}",
                    log_cstr((name) as *const _),
                    log_cstr((s) as *const _)
                ));
            } else {
                log_debug(format_args!(
                    "{} override: {}={}",
                    log_cstr((name) as *const _),
                    log_cstr((s) as *const _),
                    log_cstr((value) as *const _)
                ));
            }
        }
        i = 0 as u_int;
        while i < tty_term_ncodes() {
            ent = (&raw const tty_term_codes as *const tty_term_code_entry).offset(i as isize)
                as *const tty_term_code_entry;
            if !(strcmp(s, (*ent).name.as_ptr()) != 0 as ::core::ffi::c_int) {
                if remove != 0 {
                    (&mut (*term).codes)[i as usize] = tty_code::None;
                } else {
                    match (*ent).type_0 as ::core::ffi::c_uint {
                        1 => {
                            (&mut (*term).codes)[i as usize] =
                                tty_code::String(CStr::from_ptr(value).to_owned());
                        }
                        2 => {
                            n = strtonum(
                                value,
                                0 as ::core::ffi::c_longlong,
                                INT_MAX as ::core::ffi::c_longlong,
                                &raw mut errstr,
                            ) as ::core::ffi::c_int;
                            if errstr.is_null() {
                                (&mut (*term).codes)[i as usize] = tty_code::Number(n);
                            }
                        }
                        3 => {
                            (&mut (*term).codes)[i as usize] = tty_code::Flag(1);
                        }
                        0 | _ => {}
                    }
                }
            }
            i = i.wrapping_add(1);
        }
    }
}
pub unsafe fn tty_term_apply_overrides(mut term: *mut tty_term) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut offset: size_t = 0;
    o = options_get_only(
        global_options,
        b"terminal-overrides\0" as *const u8 as *const ::core::ffi::c_char,
    );
    a = options_array_first(o);
    while !a.is_null() {
        ov = options_array_item_value(a);
        s = (*ov).string_ptr();
        offset = 0 as size_t;
        let first = tty_term_override_next(s, &raw mut offset);
        if first.as_ref().is_some_and(|first| {
            fnmatch(
                first.as_ptr(),
                ((*term).name).as_ptr().cast_mut(),
                0 as ::core::ffi::c_int,
            ) == 0
        }) {
            tty_term_apply(term, s.offset(offset as isize), 0 as ::core::ffi::c_int);
        }
        a = options_array_next(a);
    }
    log_debug(format_args!(
        "SIXEL flag is {}",
        ((*term).flags & TERM_SIXEL != 0) as ::core::ffi::c_int
    ));
    if tty_term_has(term, TTYC_SETRGBF) != 0 && tty_term_has(term, TTYC_SETRGBB) != 0 {
        (*term).flags |= TERM_RGBCOLOURS;
    } else {
        (*term).flags &= !TERM_RGBCOLOURS;
    }
    log_debug(format_args!(
        "RGBCOLOURS flag is {}",
        ((*term).flags & TERM_RGBCOLOURS != 0) as ::core::ffi::c_int
    ));
    if tty_term_has(term, TTYC_CMG) != 0 && tty_term_has(term, TTYC_CLMG) != 0 {
        (*term).flags |= TERM_DECSLRM;
    } else {
        (*term).flags &= !TERM_DECSLRM;
    }
    log_debug(format_args!(
        "DECSLRM flag is {}",
        ((*term).flags & TERM_DECSLRM != 0) as ::core::ffi::c_int
    ));
    if tty_term_has(term, TTYC_RECT) != 0 {
        (*term).flags |= TERM_DECFRA;
    } else {
        (*term).flags &= !TERM_DECFRA;
    }
    log_debug(format_args!(
        "DECFRA flag is {}",
        ((*term).flags & TERM_DECFRA != 0) as ::core::ffi::c_int
    ));
    if tty_term_flag(term, TTYC_AM) == 0 {
        (*term).flags |= TERM_NOAM;
    } else {
        (*term).flags &= !TERM_NOAM;
    }
    log_debug(format_args!(
        "NOAM flag is {}",
        ((*term).flags & TERM_NOAM != 0) as ::core::ffi::c_int
    ));
    (*term).acs.fill([0; 2]);
    let acs = if tty_term_has(term, TTYC_ACSC) != 0 {
        CStr::from_ptr(tty_term_string(term, TTYC_ACSC))
    } else {
        c"a#j+k+l+m+n+o-p-q-r-s-t+u+v+w+x|y<z>~."
    };
    for pair in acs.to_bytes().chunks_exact(2) {
        (*term).acs[pair[0] as usize][0] = pair[1];
    }
    tty_term_validate(term);
}
unsafe fn tty_term_validate(mut term: *mut tty_term) {
    if !matches!((&(*term).codes)[TTYC_MS as usize], tty_code::String(_)) {
        return;
    }
    if *tty_term_string_ss(
        term,
        TTYC_MS,
        b"c\0" as *const u8 as *const ::core::ffi::c_char,
        b"?\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int
        != '\0' as i32
    {
        (*term).flags &= !TERM_INVALIDMS;
        return;
    }
    log_debug(format_args!("removing invalid Ms capability"));
    (*term).flags |= TERM_INVALIDMS;
    (&mut (*term).codes)[TTYC_MS as usize] = tty_code::None;
}
pub unsafe fn tty_term_create(
    mut tty: *mut tty,
    mut name: *mut ::core::ffi::c_char,
    mut caps: *mut *mut ::core::ffi::c_char,
    mut ncaps: u_int,
) -> Result<Box<tty_term>, CString> {
    let mut c: *mut client = (*tty).client;
    let mut term: *mut tty_term = ::core::ptr::null_mut::<tty_term>();
    let mut ent: *const tty_term_code_entry = ::core::ptr::null::<tty_term_code_entry>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut offset: size_t = 0;
    let mut namelen: size_t = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut envent: Option<&environ_entry> = None;
    log_debug(format_args!("adding term {}", log_cstr((name) as *const _)));
    // The global list and tty keep this address until tty_term_free.
    let mut owner = Box::new(tty_term {
        name: CStr::from_ptr(name).to_owned(),
        tty: ::core::ptr::null_mut(),
        applied_features: 0,
        acs: [[0; 2]; 256],
        codes: vec![tty_code::default(); tty_term_ncodes() as usize].into_boxed_slice(),
        flags: 0,
        entry: tty_term_entry {
            le_next: ::core::ptr::null_mut(),
            le_prev: ::core::ptr::null_mut(),
        },
    });

    term = &raw mut *owner;
    (*term).tty = tty as *mut tty;
    (*term).entry.le_next = tty_terms.lh_first;
    if !(*term).entry.le_next.is_null() {
        (*tty_terms.lh_first).entry.le_prev = &raw mut (*term).entry.le_next;
    }
    tty_terms.lh_first = term;
    (*term).entry.le_prev = &raw mut tty_terms.lh_first;
    i = 0 as u_int;
    while i < ncaps {
        namelen = strcspn(
            *caps.offset(i as isize),
            b"=\0" as *const u8 as *const ::core::ffi::c_char,
        ) as size_t;
        if !(namelen == 0 as size_t) {
            value = (*caps.offset(i as isize))
                .offset(namelen as isize)
                .offset(1 as ::core::ffi::c_int as isize);
            j = 0 as u_int;
            while j < tty_term_ncodes() {
                ent = (&raw const tty_term_codes as *const tty_term_code_entry).offset(j as isize)
                    as *const tty_term_code_entry;
                if !(strncmp((*ent).name.as_ptr(), *caps.offset(i as isize), namelen)
                    != 0 as ::core::ffi::c_int)
                {
                    if !(*(*ent).name.as_ptr().offset(namelen as isize) as ::core::ffi::c_int
                        != '\0' as i32)
                    {
                        (&mut (*term).codes)[j as usize] = tty_code::None;
                        match (*ent).type_0 as ::core::ffi::c_uint {
                            1 => {
                                (&mut (*term).codes)[j as usize] =
                                    tty_code::String(tty_term_strip(CStr::from_ptr(value)));
                            }
                            2 => {
                                n = strtonum(
                                    value,
                                    0 as ::core::ffi::c_longlong,
                                    INT_MAX as ::core::ffi::c_longlong,
                                    &raw mut errstr,
                                ) as ::core::ffi::c_int;
                                if !errstr.is_null() {
                                    log_debug(format_args!(
                                        "{}: {}",
                                        log_cstr((*ent).name.as_ptr()),
                                        log_cstr((errstr) as *const _)
                                    ));
                                } else {
                                    (&mut (*term).codes)[j as usize] = tty_code::Number(n);
                                }
                            }
                            3 => {
                                (&mut (*term).codes)[j as usize] = tty_code::Flag(
                                    (*value == b'1' as ::core::ffi::c_char) as ::core::ffi::c_int,
                                );
                            }
                            0 | _ => {}
                        }
                    }
                }
                j = j.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
    o = options_get_only(
        global_options,
        b"terminal-features\0" as *const u8 as *const ::core::ffi::c_char,
    );
    a = options_array_first(o);
    while !a.is_null() {
        ov = options_array_item_value(a);
        s = (*ov).string_ptr();
        offset = 0 as size_t;
        let first = tty_term_override_next(s, &raw mut offset);
        if first.as_ref().is_some_and(|first| {
            fnmatch(
                first.as_ptr(),
                ((*term).name).as_ptr().cast_mut(),
                0 as ::core::ffi::c_int,
            ) == 0
        }) {
            tty_parse_client_features(
                c,
                s.offset(offset as isize),
                b":\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        a = options_array_next(a);
    }
    del_curterm(cur_term);
    envent = environ_find(
        (*c).environ.as_deref().expect("environment"),
        b"COLORTERM\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if !envent.is_none() {
        log_debug(format_args!(
            "{} COLORTERM={}",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            ),
            log_cstr(
                ((envent.unwrap().value)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
        if strcasecmp(
            (envent.unwrap().value)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            b"truecolor\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
            || strcasecmp(
                (envent.unwrap().value)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                b"24bit\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            tty_parse_client_features(
                c,
                b"RGB\0" as *const u8 as *const ::core::ffi::c_char,
                b",\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else if !strstr(
            (envent.unwrap().value)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            b"256\0" as *const u8 as *const ::core::ffi::c_char,
        )
        .is_null()
        {
            tty_parse_client_features(
                c,
                b"256\0" as *const u8 as *const ::core::ffi::c_char,
                b",\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    }
    tty_term_apply_overrides(term);
    let error = if tty_term_has(term, TTYC_CLEAR) == 0 {
        Some(c"terminal does not support clear".to_owned())
    } else if tty_term_has(term, TTYC_CUP) == 0 {
        Some(c"terminal does not support cup".to_owned())
    } else {
        s = tty_term_string(term, TTYC_CLEAR);
        if tty_term_flag(term, TTYC_XT) != 0
            || strncmp(
                s,
                b"\x1B[\0" as *const u8 as *const ::core::ffi::c_char,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            (*term).flags |= TERM_VT100LIKE;
            tty_parse_client_features(
                c,
                b"bpaste,focus,title\0" as *const u8 as *const ::core::ffi::c_char,
                b",\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if (tty_term_flag(term, TTYC_TC) != 0 || tty_term_has(term, TTYC_RGB) != 0)
            && (tty_term_has(term, TTYC_SETRGBF) == 0 || tty_term_has(term, TTYC_SETRGBB) == 0)
        {
            tty_parse_client_features(
                c,
                b"RGB\0" as *const u8 as *const ::core::ffi::c_char,
                b",\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if tty_apply_features(term) != 0 {
            tty_term_apply_overrides(term);
        }
        i = 0 as u_int;
        while i < tty_term_ncodes() {
            log_debug(format_args!(
                "{}{}",
                log_cstr((name) as *const _),
                log_cstr(tty_term_describe(term, i as tty_code_code).as_ptr())
            ));
            i = i.wrapping_add(1);
        }
        return Ok(owner);
    };
    drop(owner);
    Err(error.expect("unsupported terminal has an error message"))
}
pub fn tty_term_free(term: Box<tty_term>) {
    drop(term);
}

impl Drop for tty_term {
    fn drop(&mut self) {
        // Empty/test terminals are unregistered. Constructors link the stable
        // Box before validation, so this also unlinks on every failure path.
        if self.entry.le_prev.is_null() {
            return;
        }
        unsafe {
            log_debug(format_args!("removing term {}", log_cstr(self.name.as_ptr())));
            if !self.entry.le_next.is_null() {
                (*self.entry.le_next).entry.le_prev = self.entry.le_prev;
            }
            *self.entry.le_prev = self.entry.le_next;
        }
    }
}

pub(crate) unsafe fn tty_term_read_list(name: &CStr) -> Result<Vec<CString>, CString> {
    let fd: ::core::ffi::c_int = STDIN_FILENO;
    let mut ent: *const tty_term_code_entry = ::core::ptr::null::<tty_term_code_entry>();
    let mut error: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut i: u_int = 0;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut tmp: [::core::ffi::c_char; 11] = [0; 11];
    let mut caps = Vec::new();
    if setupterm(name.as_ptr().cast_mut(), fd, &raw mut error) != OK {
        let (message, with_name): (&[u8], bool) = match error {
            1 => (b"can't use hardcopy terminal: ", true),
            0 => (b"missing or unsuitable terminal: ", true),
            -1 => (b"can't find terminfo database", false),
            _ => (b"unknown error", false),
        };
        let mut cause = message.to_vec();
        if with_name {
            cause.extend_from_slice(name.to_bytes());
        }
        return Err(CString::new(cause).expect("C strings contain no interior NUL"));
    }
    let terminal_owner = CurrentTerminalOwner(cur_term);
    let mut current_block_23: u64;
    i = 0 as u_int;
    while i < tty_term_ncodes() {
        ent = (&raw const tty_term_codes as *const tty_term_code_entry).offset(i as isize)
            as *const tty_term_code_entry;
        match (*ent).type_0 as ::core::ffi::c_uint {
            0 => {
                current_block_23 = 1856101646708284338;
            }
            1 => {
                s = tigetstr((*ent).name.as_ptr().cast_mut());
                if s.is_null()
                    || s == -(1 as ::core::ffi::c_int) as *mut ::core::ffi::c_char
                        as *const ::core::ffi::c_char
                {
                    current_block_23 = 1856101646708284338;
                } else {
                    current_block_23 = 14763689060501151050;
                }
            }
            2 => {
                n = tigetnum((*ent).name.as_ptr().cast_mut());
                if n == -(1 as ::core::ffi::c_int) || n == -(2 as ::core::ffi::c_int) {
                    current_block_23 = 1856101646708284338;
                } else {
                    xformat(&mut tmp, format_args!("{}", n as i32));
                    s = &raw mut tmp as *mut ::core::ffi::c_char;
                    current_block_23 = 14763689060501151050;
                }
            }
            3 => {
                n = tigetflag((*ent).name.as_ptr().cast_mut());
                if n == -(1 as ::core::ffi::c_int) {
                    current_block_23 = 1856101646708284338;
                } else {
                    if n != 0 {
                        s = b"1\0" as *const u8 as *const ::core::ffi::c_char;
                    } else {
                        s = b"0\0" as *const u8 as *const ::core::ffi::c_char;
                    }
                    current_block_23 = 14763689060501151050;
                }
            }
            _ => {
                fatalx(|out| out.write_all(b"unknown capability type"));
            }
        }
        match current_block_23 {
            14763689060501151050 => {
                let name = (*ent).name.to_bytes();
                let value = CStr::from_ptr(s).to_bytes();
                let mut cap = Vec::with_capacity(name.len() + 1 + value.len());
                cap.extend_from_slice(name);
                cap.push(b'=');
                cap.extend_from_slice(value);
                caps.push(CString::new(cap).expect("C strings contain no interior NUL"));
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }
    drop(terminal_owner);
    Ok(caps)
}

struct CurrentTerminalOwner(*mut TERMINAL);

impl Drop for CurrentTerminalOwner {
    fn drop(&mut self) {
        unsafe {
            del_curterm(self.0);
        }
    }
}
/// Read-only legacy projection; ownership stays with the TTY.
pub fn tty_term_owner_ptr(owner: &Option<Box<tty_term>>) -> *const tty_term {
    owner.as_deref().map_or(std::ptr::null(), |term| term)
}

pub unsafe fn tty_term_has(mut term: *const tty_term, mut code: tty_code_code) -> ::core::ffi::c_int {
    (!matches!((&(*term).codes)[code as usize], tty_code::None)) as ::core::ffi::c_int
}
pub unsafe fn tty_term_has_name(
    mut term: *const tty_term,
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < tty_term_ncodes() {
        if strcmp(tty_term_codes[i as usize].name.as_ptr(), name) == 0 as ::core::ffi::c_int {
            return tty_term_has(term, i as tty_code_code);
        }
        i = i.wrapping_add(1);
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn tty_term_string(
    term: *const tty_term,
    code: tty_code_code,
) -> *const ::core::ffi::c_char {
    match &(&(*term).codes)[code as usize] {
        tty_code::None => c"".as_ptr(),
        tty_code::String(value) => value.as_ptr(),
        _ => fatalx(|out| write!(out, "not a string: {}", (code) as i32)),
    }
}
pub unsafe fn tty_term_string_i(
    mut term: *const tty_term,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = tiparm_s(1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int, x, a);
    if s.is_null() {
        log_debug(format_args!(
            "could not expand {}",
            log_cstr((tty_term_codes[code as usize].name.as_ptr()) as *const _)
        ));
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return s;
}
pub unsafe fn tty_term_string_ii(
    mut term: *const tty_term,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = tiparm_s(2 as ::core::ffi::c_int, 0 as ::core::ffi::c_int, x, a, b);
    if s.is_null() {
        log_debug(format_args!(
            "could not expand {}",
            log_cstr((tty_term_codes[code as usize].name.as_ptr()) as *const _)
        ));
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return s;
}
pub unsafe fn tty_term_string_iii(
    mut term: *const tty_term,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = tiparm_s(3 as ::core::ffi::c_int, 0 as ::core::ffi::c_int, x, a, b, c);
    if s.is_null() {
        log_debug(format_args!(
            "could not expand {}",
            log_cstr((tty_term_codes[code as usize].name.as_ptr()) as *const _)
        ));
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return s;
}
pub unsafe fn tty_term_string_s(
    mut term: *const tty_term,
    mut code: tty_code_code,
    mut a: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = tiparm_s(1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int, x, a);
    if s.is_null() {
        log_debug(format_args!(
            "could not expand {}",
            log_cstr((tty_term_codes[code as usize].name.as_ptr()) as *const _)
        ));
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return s;
}
pub unsafe fn tty_term_string_ss(
    mut term: *const tty_term,
    mut code: tty_code_code,
    mut a: *const ::core::ffi::c_char,
    mut b: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = tiparm_s(2 as ::core::ffi::c_int, 3 as ::core::ffi::c_int, x, a, b);
    if s.is_null() {
        log_debug(format_args!(
            "could not expand {}",
            log_cstr((tty_term_codes[code as usize].name.as_ptr()) as *const _)
        ));
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return s;
}
pub unsafe fn tty_term_number(term: *const tty_term, code: tty_code_code) -> ::core::ffi::c_int {
    match &(&(*term).codes)[code as usize] {
        tty_code::None => 0,
        tty_code::Number(value) => *value,
        _ => fatalx(|out| write!(out, "not a number: {}", (code) as i32)),
    }
}
pub unsafe fn tty_term_flag(term: *const tty_term, code: tty_code_code) -> ::core::ffi::c_int {
    match &(&(*term).codes)[code as usize] {
        tty_code::None => 0,
        tty_code::Flag(value) => *value,
        _ => fatalx(|out| write!(out, "not a flag: {}", (code) as i32)),
    }
}
pub unsafe fn tty_term_describe(
    term: *const tty_term,
    code: tty_code_code,
) -> CString {
    let mut escaped: [::core::ffi::c_char; 128] = [0; 128];
    match &(&(*term).codes)[code as usize] {
        tty_code::None => {
            format_message_with(|out| {
                write!(out, "{:4}: ", code as u32)?;
                out.write_all(tty_term_codes[code as usize].name.to_bytes())?;
                out.write_all(b": [missing]")
            })
        }
        tty_code::String(value) => {
            strnvis(
                &raw mut escaped as *mut ::core::ffi::c_char,
                value.as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
                VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL,
            );
            format_message_with(|out| {
                write!(out, "{:4}: ", code as u32)?;
                out.write_all(tty_term_codes[code as usize].name.to_bytes())?;
                out.write_all(b": (string) ")?;
                out.write_all(CStr::from_ptr(escaped.as_ptr()).to_bytes())
            })
        }
        tty_code::Number(value) => {
            format_message_with(|out| {
                write!(out, "{:4}: ", code as u32)?;
                out.write_all(tty_term_codes[code as usize].name.to_bytes())?;
                write!(out, ": (number) {}", (*value) as i32)
            })
        }
        tty_code::Flag(value) => {
            format_message_with(|out| {
                write!(out, "{:4}: ", code as u32)?;
                out.write_all(tty_term_codes[code as usize].name.to_bytes())?;
                out.write_all(b": (flag) ")?;
                out.write_all(if *value != 0 { b"true" } else { b"false" })
            })
        }
    }
}

#[cfg(test)]
mod term_string_owner_tests {
    use super::*;

    #[test]
    fn terminal_owners_unlink_after_constructor_failure_and_out_of_order_drop() {
        use crate::src::options::{options_create_owned, options_empty, options_search};
        unsafe {
            let saved = global_options;
            let initial_head = tty_terms.lh_first;
            let mut options = options_create_owned(std::ptr::null_mut());
            global_options = &raw mut *options;
            for name in [c"terminal-features", c"terminal-overrides"] {
                options_empty(global_options, options_search(name.as_ptr()));
            }
            let mut client = client::empty();
            client.environ = Some(crate::src::environ::environ_create());
            let mut terminal = tty::empty();
            terminal.client = &raw mut client;
            let name = c"owner-test".as_ptr().cast_mut();

            // A failed constructor must remove its already-published address.
            let failed = tty_term_create(&mut terminal, name, std::ptr::null_mut(), 0);
            assert_eq!(failed.err().unwrap().as_c_str(), c"terminal does not support clear");
            assert_eq!(tty_terms.lh_first, initial_head);

            let mut caps = [c"clear=C".as_ptr().cast_mut(), c"cup=P".as_ptr().cast_mut()];
            let first = tty_term_create(&mut terminal, name, caps.as_mut_ptr(), 2).unwrap();
            let first_ptr = &*first as *const tty_term;
            terminal.term = Some(first);
            let second = tty_term_create(&mut terminal, name, caps.as_mut_ptr(), 2).unwrap();
            assert!(std::ptr::eq(second.entry.le_next, first_ptr));
            // Remove the tail before the head; the head's back-links must be repaired.
            drop(terminal.term.take());
            assert_eq!(second.entry.le_next, initial_head);
            drop(second);
            assert_eq!(tty_terms.lh_first, initial_head);
            global_options = saved;
        }
    }

    #[test]
    fn override_tokens_keep_escaping_limits_and_independent_storage() {
        unsafe {
            let source = c":clear=left::right:bel=\xff";
            let mut offset = 0;
            let empty = tty_term_override_next(source.as_ptr(), &mut offset).unwrap();
            let first = tty_term_override_next(source.as_ptr(), &mut offset).unwrap();
            let second = tty_term_override_next(source.as_ptr(), &mut offset).unwrap();
            assert!(tty_term_override_next(source.as_ptr(), &mut offset).is_none());
            assert_eq!(CStr::from_ptr(empty.as_ptr()), c"");
            assert_eq!(CStr::from_ptr(first.as_ptr()), c"clear=left:right");
            assert_eq!(CStr::from_ptr(second.as_ptr()), c"bel=\xff");

            // tmux rejects a decoded token at 8191 bytes without advancing.
            for (len, accepted) in [(8190, true), (8191, false), (8192, false)] {
                let source = CString::new(format!("x:{}:next", "a".repeat(len))).unwrap();
                let mut offset = 2;
                let token = tty_term_override_next(source.as_ptr(), &mut offset);
                assert_eq!(token.is_some(), accepted);
                if let Some(token) = token {
                    assert_eq!(CStr::from_ptr(token.as_ptr()).to_bytes().len(), len);
                    assert_eq!(offset, len + 3);
                } else {
                    assert_eq!(offset, 2);
                }
            }
        }
    }

    #[test]
    fn descriptions_keep_padding_escaping_and_variant_labels() {
        unsafe {
            let mut term = tty_term::empty();
            term.codes = vec![tty_code::None; tty_term_ncodes() as usize].into_boxed_slice();
            for (value, suffix) in [
                (tty_code::None, "[missing]"),
                (tty_code::Number(-12), "(number) -12"),
                (tty_code::Flag(0), "(flag) false"),
                (tty_code::Flag(1), "(flag) true"),
                (
                    tty_code::String(CString::new("a\nb").unwrap()),
                    "(string) a\\nb",
                ),
            ] {
                term.codes[TTYC_CLEAR as usize] = value;
                let expected = format!("{:4}: clear: {suffix}", TTYC_CLEAR as u32);
                assert_eq!(
                    tty_term_describe(&mut term, TTYC_CLEAR).as_bytes(),
                    expected.as_bytes(),
                );
            }
        }
    }

    #[test]
    fn overrides_replace_owned_values_and_preserve_missing_semantics() {
        unsafe {
            let mut term = tty_term::empty();
            term.codes = vec![tty_code::None; tty_term_ncodes() as usize].into_boxed_slice();
            assert_eq!(tty_term_has(&mut term, TTYC_CLEAR), 0);
            assert_eq!(CStr::from_ptr(tty_term_string(&mut term, TTYC_CLEAR)), c"");
            assert_eq!(tty_term_number(&mut term, TTYC_COLORS), 0);
            assert_eq!(tty_term_flag(&mut term, TTYC_AM), 0);

            let overrides =
                CString::new(b"clear=first:colors=256:am:bel=high\xff".as_slice()).unwrap();
            tty_term_apply(&mut term, overrides.as_ptr(), 1);
            drop(overrides);
            assert_eq!(
                CStr::from_ptr(tty_term_string(&mut term, TTYC_CLEAR)),
                c"first"
            );
            assert_eq!(
                CStr::from_ptr(tty_term_string(&mut term, TTYC_BEL)).to_bytes(),
                b"high\xff"
            );
            assert_eq!(tty_term_number(&mut term, TTYC_COLORS), 256);
            assert_eq!(tty_term_flag(&mut term, TTYC_AM), 1);

            tty_term_apply(&mut term, c"clear=second:colors=invalid".as_ptr(), 1);
            assert_eq!(
                CStr::from_ptr(tty_term_string(&mut term, TTYC_CLEAR)),
                c"second"
            );
            assert_eq!(tty_term_number(&mut term, TTYC_COLORS), 256);
            tty_term_apply(&mut term, c"clear=:colors@:am@".as_ptr(), 1);
            assert_eq!(tty_term_has(&mut term, TTYC_CLEAR), 1);
            assert_eq!(CStr::from_ptr(tty_term_string(&mut term, TTYC_CLEAR)), c"");
            assert_eq!(tty_term_has(&mut term, TTYC_COLORS), 0);
            assert_eq!(tty_term_has(&mut term, TTYC_AM), 0);
            tty_term_apply(&mut term, c"clear@".as_ptr(), 1);
            assert_eq!(tty_term_has(&mut term, TTYC_CLEAR), 0);
            tty_term_apply(&mut term, c"clear=restored".as_ptr(), 1);
            assert_eq!(
                CStr::from_ptr(tty_term_string(&mut term, TTYC_CLEAR)),
                c"restored"
            );
        }
    }

    #[test]
    fn validation_removes_empty_clipboard_capability() {
        unsafe {
            let mut term = tty_term::empty();
            term.codes = vec![tty_code::None; tty_term_ncodes() as usize].into_boxed_slice();
            term.codes[TTYC_MS as usize] = tty_code::String(CString::default());
            tty_term_validate(&mut term);
            assert_eq!(tty_term_has(&mut term, TTYC_MS), 0);
            assert_ne!(term.flags & TERM_INVALIDMS, 0);
            term.codes[TTYC_MS as usize] = tty_code::String(c"%p1%s%p2%s".to_owned());
            tty_term_validate(&mut term);
            assert_eq!(tty_term_has(&mut term, TTYC_MS), 1);
            assert_eq!(term.flags & TERM_INVALIDMS, 0);
        }
    }

    #[test]
    fn strip_preserves_bytes_and_delay_path_limit() {
        assert_eq!(tty_term_strip(c"ab$<5>cd").as_bytes(), b"abcd");
        let high = CString::new(b"a\xff$<10>b".as_slice()).unwrap();
        assert_eq!(tty_term_strip(high.as_c_str()).as_bytes(), b"a\xffb");

        let mut long = b"$<1>".to_vec();
        long.extend(std::iter::repeat_n(b'x', 9000));
        let long = CString::new(long).unwrap();
        let stripped = tty_term_strip(long.as_c_str());
        assert_eq!(stripped.as_bytes().len(), 8191);
        assert!(stripped.as_bytes().iter().all(|byte| *byte == b'x'));
    }
}
