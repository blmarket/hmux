//! Authoritative posix_terminal declarations from the translated Linux C ABI.

pub const VTIME: ::core::ffi::c_int = 5 as ::core::ffi::c_int;

pub const VMIN: ::core::ffi::c_int = 6 as ::core::ffi::c_int;

pub const ICRNL: ::core::ffi::c_int = 0o400 as ::core::ffi::c_int;

pub const OPOST: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;

pub const ONLCR: ::core::ffi::c_int = 0o4 as ::core::ffi::c_int;

pub const TCSANOW: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct winsize {
    pub ws_row: ::core::ffi::c_ushort,
    pub ws_col: ::core::ffi::c_ushort,
    pub ws_xpixel: ::core::ffi::c_ushort,
    pub ws_ypixel: ::core::ffi::c_ushort,
}

pub const TIOCSWINSZ: ::core::ffi::c_int = 0x5414 as ::core::ffi::c_int;

pub const VERASE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
