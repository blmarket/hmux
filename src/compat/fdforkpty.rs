use crate::src::shared::terminal::*;
use crate::src::shared::abi::*;
extern "C" {
    fn forkpty(
        __amaster: *mut ::core::ffi::c_int,
        __name: *mut ::core::ffi::c_char,
        __termp: *const termios,
        __winp: *const winsize,
    ) -> ::core::ffi::c_int;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct winsize {
    pub ws_row: ::core::ffi::c_ushort,
    pub ws_col: ::core::ffi::c_ushort,
    pub ws_xpixel: ::core::ffi::c_ushort,
    pub ws_ypixel: ::core::ffi::c_ushort,
}
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
#[no_mangle]
pub unsafe extern "C" fn getptmfd() -> ::core::ffi::c_int {
    return 2147483647 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn fdforkpty(
    mut ptmfd: ::core::ffi::c_int,
    mut master: *mut ::core::ffi::c_int,
    mut name: *mut ::core::ffi::c_char,
    mut tio: *mut termios,
    mut ws: *mut winsize,
) -> pid_t {
    return forkpty(master, name, tio, ws) as pid_t;
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
