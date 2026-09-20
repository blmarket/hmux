pub use crate::src::shared::limits::{__INT_MAX__, INT_MAX};
pub use crate::src::shared::posix_terminal::winsize;
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
