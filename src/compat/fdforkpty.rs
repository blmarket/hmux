use crate::src::ffi::libc::forkpty;
use crate::src::shared::abi::*;
use crate::src::shared::posix_terminal::winsize;
use crate::src::shared::terminal::*;

#[no_mangle]
pub unsafe extern "C" fn getptmfd() -> ::core::ffi::c_int {
    return 2147483647 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn fdforkpty(
    _ptmfd: ::core::ffi::c_int,
    mut master: *mut ::core::ffi::c_int,
    mut name: *mut ::core::ffi::c_char,
    mut tio: *mut termios,
    mut ws: *mut winsize,
) -> pid_t {
    return forkpty(master, name, tio, ws) as pid_t;
}
