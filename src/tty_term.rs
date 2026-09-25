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
use crate::src::log::{fatalx, log_debug};
use crate::src::options::{
    options_array_first, options_array_item_value, options_array_next, options_get_only,
};
use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::environment::environ_entry;
use crate::src::shared::limits::INT_MAX;
use crate::src::shared::options::{options_array_item, options_entry, options_value};
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
use crate::src::xmalloc::xsnprintf;
use std::ffi::{CStr, CString};

unsafe fn tty_term_replace_string(term: &mut tty_term, index: usize, value: Option<CString>) {
    let owner = term;
    assert!(index < owner.strings.len());
    let code = owner.codes.add(index);
    if owner.strings[index].is_some() {
        (*code).value.string = ::core::ptr::null_mut();
    }
    owner.strings[index] = value;
    if let Some(value) = owner.strings[index].as_ref() {
        (*code).value.string = value.as_ptr().cast_mut();
    }
}

pub const TTYCODE_FLAG: tty_code_type = 3;
pub const TTYCODE_NUMBER: tty_code_type = 2;
pub const TTYCODE_STRING: tty_code_type = 1;
pub const TTYCODE_NONE: tty_code_type = 0;

#[derive(Copy, Clone)]
pub struct tty_term_code_entry {
    pub type_0: tty_code_type,
    pub name: &'static ::std::ffi::CStr,
}
pub const OK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

#[no_mangle]
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
#[no_mangle]
pub unsafe extern "C" fn tty_term_ncodes() -> u_int {
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
unsafe extern "C" fn tty_term_override_next(
    mut s: *const ::core::ffi::c_char,
    mut offset: *mut size_t,
) -> *mut ::core::ffi::c_char {
    static mut value: [::core::ffi::c_char; 8192] = [0; 8192];
    let mut n: size_t = 0 as size_t;
    let mut at: size_t = *offset;
    if *s.offset(at as isize) as ::core::ffi::c_int == '\0' as i32 {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    while *s.offset(at as isize) as ::core::ffi::c_int != '\0' as i32 {
        if *s.offset(at as isize) as ::core::ffi::c_int == ':' as i32 {
            if !(*s.offset(at.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                == ':' as i32)
            {
                break;
            }
            let fresh1 = n;
            n = n.wrapping_add(1);
            value[fresh1 as usize] = ':' as i32 as ::core::ffi::c_char;
            at = at.wrapping_add(2 as size_t);
        } else {
            let fresh2 = n;
            n = n.wrapping_add(1);
            value[fresh2 as usize] = *s.offset(at as isize);
            at = at.wrapping_add(1);
        }
        if n == (::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as usize)
            .wrapping_sub(1 as usize)
        {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
    }
    if *s.offset(at as isize) as ::core::ffi::c_int != '\0' as i32 {
        *offset = at.wrapping_add(1 as size_t);
    } else {
        *offset = at;
    }
    value[n as usize] = '\0' as i32 as ::core::ffi::c_char;
    return &raw mut value as *mut ::core::ffi::c_char;
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
#[no_mangle]
pub unsafe extern "C" fn tty_term_apply(
    mut term: *mut tty_term,
    mut capabilities: *const ::core::ffi::c_char,
    mut quiet: ::core::ffi::c_int,
) {
    let mut ent: *const tty_term_code_entry = ::core::ptr::null::<tty_term_code_entry>();
    let mut code: *mut tty_code = ::core::ptr::null_mut::<tty_code>();
    let mut offset: size_t = 0 as size_t;
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut name: *const ::core::ffi::c_char = ((*term).name).as_ptr().cast_mut();
    let mut i: u_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut remove: ::core::ffi::c_int = 0;
    loop {
        s = tty_term_override_next(capabilities, &raw mut offset);
        if s.is_null() {
            break;
        }
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
                log_debug(
                    b"%s override: %s@\0" as *const u8 as *const ::core::ffi::c_char,
                    name,
                    s,
                );
            } else if *value as ::core::ffi::c_int == '\0' as i32 {
                log_debug(
                    b"%s override: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    name,
                    s,
                );
            } else {
                log_debug(
                    b"%s override: %s=%s\0" as *const u8 as *const ::core::ffi::c_char,
                    name,
                    s,
                    value,
                );
            }
        }
        i = 0 as u_int;
        while i < tty_term_ncodes() {
            ent = (&raw const tty_term_codes as *const tty_term_code_entry).offset(i as isize)
                as *const tty_term_code_entry;
            if !(strcmp(s, (*ent).name.as_ptr()) != 0 as ::core::ffi::c_int) {
                code = (*term).codes.offset(i as isize) as *mut tty_code;
                if remove != 0 {
                    tty_term_replace_string(&mut *term, i as usize, None);
                    (*code).type_0 = TTYCODE_NONE;
                } else {
                    match (*ent).type_0 as ::core::ffi::c_uint {
                        1 => {
                            tty_term_replace_string(
                                &mut *term,
                                i as usize,
                                Some(CStr::from_ptr(value).to_owned()),
                            );
                            (*code).type_0 = (*ent).type_0;
                        }
                        2 => {
                            n = strtonum(
                                value,
                                0 as ::core::ffi::c_longlong,
                                INT_MAX as ::core::ffi::c_longlong,
                                &raw mut errstr,
                            ) as ::core::ffi::c_int;
                            if errstr.is_null() {
                                tty_term_replace_string(&mut *term, i as usize, None);
                                (*code).value.number = n;
                                (*code).type_0 = (*ent).type_0;
                            }
                        }
                        3 => {
                            tty_term_replace_string(&mut *term, i as usize, None);
                            (*code).value.flag = 1 as ::core::ffi::c_int;
                            (*code).type_0 = (*ent).type_0;
                        }
                        0 | _ => {}
                    }
                }
            }
            i = i.wrapping_add(1);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_apply_overrides(mut term: *mut tty_term) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut acs: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut offset: size_t = 0;
    let mut first: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    o = options_get_only(
        global_options,
        b"terminal-overrides\0" as *const u8 as *const ::core::ffi::c_char,
    );
    a = options_array_first(o);
    while !a.is_null() {
        ov = options_array_item_value(a);
        s = (*ov).string_ptr();
        offset = 0 as size_t;
        first = tty_term_override_next(s, &raw mut offset);
        if !first.is_null()
            && fnmatch(
                first,
                ((*term).name).as_ptr().cast_mut(),
                0 as ::core::ffi::c_int,
            ) == 0 as ::core::ffi::c_int
        {
            tty_term_apply(term, s.offset(offset as isize), 0 as ::core::ffi::c_int);
        }
        a = options_array_next(a);
    }
    log_debug(
        b"SIXEL flag is %d\0" as *const u8 as *const ::core::ffi::c_char,
        ((*term).flags & TERM_SIXEL != 0) as ::core::ffi::c_int,
    );
    if tty_term_has(term, TTYC_SETRGBF) != 0 && tty_term_has(term, TTYC_SETRGBB) != 0 {
        (*term).flags |= TERM_RGBCOLOURS;
    } else {
        (*term).flags &= !TERM_RGBCOLOURS;
    }
    log_debug(
        b"RGBCOLOURS flag is %d\0" as *const u8 as *const ::core::ffi::c_char,
        ((*term).flags & TERM_RGBCOLOURS != 0) as ::core::ffi::c_int,
    );
    if tty_term_has(term, TTYC_CMG) != 0 && tty_term_has(term, TTYC_CLMG) != 0 {
        (*term).flags |= TERM_DECSLRM;
    } else {
        (*term).flags &= !TERM_DECSLRM;
    }
    log_debug(
        b"DECSLRM flag is %d\0" as *const u8 as *const ::core::ffi::c_char,
        ((*term).flags & TERM_DECSLRM != 0) as ::core::ffi::c_int,
    );
    if tty_term_has(term, TTYC_RECT) != 0 {
        (*term).flags |= TERM_DECFRA;
    } else {
        (*term).flags &= !TERM_DECFRA;
    }
    log_debug(
        b"DECFRA flag is %d\0" as *const u8 as *const ::core::ffi::c_char,
        ((*term).flags & TERM_DECFRA != 0) as ::core::ffi::c_int,
    );
    if tty_term_flag(term, TTYC_AM) == 0 {
        (*term).flags |= TERM_NOAM;
    } else {
        (*term).flags &= !TERM_NOAM;
    }
    log_debug(
        b"NOAM flag is %d\0" as *const u8 as *const ::core::ffi::c_char,
        ((*term).flags & TERM_NOAM != 0) as ::core::ffi::c_int,
    );
    memset(
        &raw mut (*term).acs as *mut [::core::ffi::c_char; 2] as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[[::core::ffi::c_char; 2]; 256]>() as size_t,
    );
    if tty_term_has(term, TTYC_ACSC) != 0 {
        acs = tty_term_string(term, TTYC_ACSC);
    } else {
        acs =
            b"a#j+k+l+m+n+o-p-q-r-s-t+u+v+w+x|y<z>~.\0" as *const u8 as *const ::core::ffi::c_char;
    }
    while *acs.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
        && *acs.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
    {
        (*term).acs[*acs.offset(0 as ::core::ffi::c_int as isize) as u_char as usize]
            [0 as ::core::ffi::c_int as usize] = *acs.offset(1 as ::core::ffi::c_int as isize);
        acs = acs.offset(2 as ::core::ffi::c_int as isize);
    }
    tty_term_validate(term);
}
unsafe extern "C" fn tty_term_validate(mut term: *mut tty_term) {
    let mut code: *mut tty_code =
        (*term).codes.offset(TTYC_MS as ::core::ffi::c_int as isize) as *mut tty_code;
    if (*code).type_0 as ::core::ffi::c_uint
        != TTYCODE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
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
    log_debug(b"removing invalid Ms capability\0" as *const u8 as *const ::core::ffi::c_char);
    (*term).flags |= TERM_INVALIDMS;
    tty_term_replace_string(&mut *term, TTYC_MS as usize, None);
    (*code).type_0 = TTYCODE_NONE;
}
pub unsafe fn tty_term_create(
    mut tty: *mut tty,
    mut name: *mut ::core::ffi::c_char,
    mut caps: *mut *mut ::core::ffi::c_char,
    mut ncaps: u_int,
) -> Result<*mut tty_term, CString> {
    let mut c: *mut client = (*tty).client;
    let mut term: *mut tty_term = ::core::ptr::null_mut::<tty_term>();
    let mut ent: *const tty_term_code_entry = ::core::ptr::null::<tty_term_code_entry>();
    let mut code: *mut tty_code = ::core::ptr::null_mut::<tty_code>();
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
    let mut first: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut n: ::core::ffi::c_int = 0;
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    log_debug(
        b"adding term %s\0" as *const u8 as *const ::core::ffi::c_char,
        name,
    );
    // The global list and tty keep this address until tty_term_free.
    let mut owner = Box::new(tty_term {
        name: CStr::from_ptr(name).to_owned(),
        tty: ::core::ptr::null_mut(),
        applied_features: 0,
        acs: [[0; 2]; 256],
        codes: ::core::ptr::null_mut(),
        flags: 0,
        entry: tty_term_entry {
            le_next: ::core::ptr::null_mut(),
            le_prev: ::core::ptr::null_mut(),
        },
        strings: vec![None; tty_term_ncodes() as usize],
    });

    term = &raw mut *owner;
    let _ = Box::into_raw(owner);
    (*term).tty = tty as *mut tty;
    let codes =
        vec![::core::mem::zeroed::<tty_code>(); tty_term_ncodes() as usize].into_boxed_slice();
    (*term).codes = Box::into_raw(codes) as *mut tty_code;
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
                        code = (*term).codes.offset(j as isize) as *mut tty_code;
                        tty_term_replace_string(&mut *term, j as usize, None);
                        (*code).type_0 = TTYCODE_NONE;
                        match (*ent).type_0 as ::core::ffi::c_uint {
                            1 => {
                                (*code).type_0 = TTYCODE_STRING;
                                tty_term_replace_string(
                                    &mut *term,
                                    j as usize,
                                    Some(tty_term_strip(CStr::from_ptr(value))),
                                );
                            }
                            2 => {
                                n = strtonum(
                                    value,
                                    0 as ::core::ffi::c_longlong,
                                    INT_MAX as ::core::ffi::c_longlong,
                                    &raw mut errstr,
                                ) as ::core::ffi::c_int;
                                if !errstr.is_null() {
                                    log_debug(
                                        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                                        (*ent).name,
                                        errstr,
                                    );
                                } else {
                                    (*code).type_0 = TTYCODE_NUMBER;
                                    (*code).value.number = n;
                                }
                            }
                            3 => {
                                (*code).type_0 = TTYCODE_FLAG;
                                (*code).value.flag = (*value as ::core::ffi::c_int == '1' as i32)
                                    as ::core::ffi::c_int;
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
        first = tty_term_override_next(s, &raw mut offset);
        if !first.is_null()
            && fnmatch(
                first,
                ((*term).name).as_ptr().cast_mut(),
                0 as ::core::ffi::c_int,
            ) == 0 as ::core::ffi::c_int
        {
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
        (*c).environ,
        b"COLORTERM\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if !envent.is_null() {
        log_debug(
            b"%s COLORTERM=%s\0" as *const u8 as *const ::core::ffi::c_char,
            ((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            ((*envent).value)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        );
        if strcasecmp(
            ((*envent).value)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            b"truecolor\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
            || strcasecmp(
                ((*envent).value)
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
            ((*envent).value)
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
            log_debug(
                b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                name,
                tty_term_describe(term, i as tty_code_code),
            );
            i = i.wrapping_add(1);
        }
        return Ok(term);
    };
    tty_term_free(term);
    Err(error.expect("unsupported terminal has an error message"))
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_free(mut term: *mut tty_term) {
    let mut i: u_int = 0;
    log_debug(
        b"removing term %s\0" as *const u8 as *const ::core::ffi::c_char,
        ((*term).name).as_ptr().cast_mut(),
    );
    i = 0 as u_int;
    while i < tty_term_ncodes() {
        tty_term_replace_string(&mut *term, i as usize, None);
        i = i.wrapping_add(1);
    }
    drop(Box::from_raw(::core::ptr::slice_from_raw_parts_mut(
        (*term).codes,
        tty_term_ncodes() as usize,
    )));
    if !(*term).entry.le_next.is_null() {
        (*(*term).entry.le_next).entry.le_prev = (*term).entry.le_prev;
    }
    *(*term).entry.le_prev = (*term).entry.le_next;
    drop(Box::from_raw(term));
}
pub(crate) unsafe fn tty_term_read_list(
    name: &CStr,
    fd: ::core::ffi::c_int,
) -> Result<Vec<CString>, CString> {
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
                    xsnprintf(
                        &raw mut tmp as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 11]>() as size_t,
                        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
                        n,
                    );
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
                fatalx(b"unknown capability type\0" as *const u8 as *const ::core::ffi::c_char);
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
#[no_mangle]
pub unsafe extern "C" fn tty_term_has(
    mut term: *mut tty_term,
    mut code: tty_code_code,
) -> ::core::ffi::c_int {
    return ((*(*term).codes.offset(code as isize)).type_0 as ::core::ffi::c_uint
        != TTYCODE_NONE as ::core::ffi::c_int as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_has_name(
    mut term: *mut tty_term,
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
#[no_mangle]
pub unsafe extern "C" fn tty_term_string(
    mut term: *mut tty_term,
    mut code: tty_code_code,
) -> *const ::core::ffi::c_char {
    if tty_term_has(term, code) == 0 {
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*(*term).codes.offset(code as isize)).type_0 as ::core::ffi::c_uint
        != TTYCODE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        fatalx(
            b"not a string: %d\0" as *const u8 as *const ::core::ffi::c_char,
            code as ::core::ffi::c_uint,
        );
    }
    return (*(*term).codes.offset(code as isize)).value.string;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_string_i(
    mut term: *mut tty_term,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = tiparm_s(1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int, x, a);
    if s.is_null() {
        log_debug(
            b"could not expand %s\0" as *const u8 as *const ::core::ffi::c_char,
            tty_term_codes[code as usize].name.as_ptr(),
        );
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_string_ii(
    mut term: *mut tty_term,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = tiparm_s(2 as ::core::ffi::c_int, 0 as ::core::ffi::c_int, x, a, b);
    if s.is_null() {
        log_debug(
            b"could not expand %s\0" as *const u8 as *const ::core::ffi::c_char,
            tty_term_codes[code as usize].name.as_ptr(),
        );
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_string_iii(
    mut term: *mut tty_term,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = tiparm_s(3 as ::core::ffi::c_int, 0 as ::core::ffi::c_int, x, a, b, c);
    if s.is_null() {
        log_debug(
            b"could not expand %s\0" as *const u8 as *const ::core::ffi::c_char,
            tty_term_codes[code as usize].name.as_ptr(),
        );
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_string_s(
    mut term: *mut tty_term,
    mut code: tty_code_code,
    mut a: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = tiparm_s(1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int, x, a);
    if s.is_null() {
        log_debug(
            b"could not expand %s\0" as *const u8 as *const ::core::ffi::c_char,
            tty_term_codes[code as usize].name.as_ptr(),
        );
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_string_ss(
    mut term: *mut tty_term,
    mut code: tty_code_code,
    mut a: *const ::core::ffi::c_char,
    mut b: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = tiparm_s(2 as ::core::ffi::c_int, 3 as ::core::ffi::c_int, x, a, b);
    if s.is_null() {
        log_debug(
            b"could not expand %s\0" as *const u8 as *const ::core::ffi::c_char,
            tty_term_codes[code as usize].name.as_ptr(),
        );
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_number(
    mut term: *mut tty_term,
    mut code: tty_code_code,
) -> ::core::ffi::c_int {
    if tty_term_has(term, code) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*(*term).codes.offset(code as isize)).type_0 as ::core::ffi::c_uint
        != TTYCODE_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        fatalx(
            b"not a number: %d\0" as *const u8 as *const ::core::ffi::c_char,
            code as ::core::ffi::c_uint,
        );
    }
    return (*(*term).codes.offset(code as isize)).value.number;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_flag(
    mut term: *mut tty_term,
    mut code: tty_code_code,
) -> ::core::ffi::c_int {
    if tty_term_has(term, code) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*(*term).codes.offset(code as isize)).type_0 as ::core::ffi::c_uint
        != TTYCODE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        fatalx(
            b"not a flag: %d\0" as *const u8 as *const ::core::ffi::c_char,
            code as ::core::ffi::c_uint,
        );
    }
    return (*(*term).codes.offset(code as isize)).value.flag;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_describe(
    mut term: *mut tty_term,
    mut code: tty_code_code,
) -> *const ::core::ffi::c_char {
    static mut s: [::core::ffi::c_char; 256] = [0; 256];
    let mut out: [::core::ffi::c_char; 128] = [0; 128];
    match (*(*term).codes.offset(code as isize)).type_0 as ::core::ffi::c_uint {
        0 => {
            xsnprintf(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
                b"%4u: %s: [missing]\0" as *const u8 as *const ::core::ffi::c_char,
                code as ::core::ffi::c_uint,
                tty_term_codes[code as usize].name.as_ptr(),
            );
        }
        1 => {
            strnvis(
                &raw mut out as *mut ::core::ffi::c_char,
                (*(*term).codes.offset(code as isize)).value.string,
                ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
                VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL,
            );
            xsnprintf(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
                b"%4u: %s: (string) %s\0" as *const u8 as *const ::core::ffi::c_char,
                code as ::core::ffi::c_uint,
                tty_term_codes[code as usize].name.as_ptr(),
                &raw mut out as *mut ::core::ffi::c_char,
            );
        }
        2 => {
            xsnprintf(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
                b"%4u: %s: (number) %d\0" as *const u8 as *const ::core::ffi::c_char,
                code as ::core::ffi::c_uint,
                tty_term_codes[code as usize].name.as_ptr(),
                (*(*term).codes.offset(code as isize)).value.number,
            );
        }
        3 => {
            xsnprintf(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
                b"%4u: %s: (flag) %s\0" as *const u8 as *const ::core::ffi::c_char,
                code as ::core::ffi::c_uint,
                tty_term_codes[code as usize].name.as_ptr(),
                if (*(*term).codes.offset(code as isize)).value.flag != 0 {
                    b"true\0" as *const u8 as *const ::core::ffi::c_char
                } else {
                    b"false\0" as *const u8 as *const ::core::ffi::c_char
                },
            );
        }
        _ => {}
    }
    return &raw mut s as *mut ::core::ffi::c_char;
}

#[cfg(test)]
mod term_string_owner_tests {
    use super::*;

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
