//! Authoritative borders declarations.

pub const CELL_UD: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

pub const CELL_LR: ::core::ffi::c_int = 2 as ::core::ffi::c_int;

pub const CELL_RD: ::core::ffi::c_int = 3 as ::core::ffi::c_int;

pub const CELL_LD: ::core::ffi::c_int = 4 as ::core::ffi::c_int;

pub const CELL_RU: ::core::ffi::c_int = 5 as ::core::ffi::c_int;

pub const CELL_LU: ::core::ffi::c_int = 6 as ::core::ffi::c_int;

pub const CELL_LRD: ::core::ffi::c_int = 7 as ::core::ffi::c_int;

pub const CELL_LRU: ::core::ffi::c_int = 8 as ::core::ffi::c_int;

pub const CELL_URD: ::core::ffi::c_int = 9 as ::core::ffi::c_int;

pub const CELL_ULD: ::core::ffi::c_int = 10 as ::core::ffi::c_int;

pub const CELL_LRUD: ::core::ffi::c_int = 11 as ::core::ffi::c_int;

pub const CELL_NONE: ::core::ffi::c_int = 12 as ::core::ffi::c_int;

pub const CELL_BORDERS: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b" xqlkmjwvtun~\0") };

pub const SIMPLE_BORDERS: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b" |-+++++++++.\0") };
