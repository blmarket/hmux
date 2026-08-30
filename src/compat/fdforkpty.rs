use crate::ffi::forkpty;
pub use crate::types::*;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub fn getptmfd() -> ::core::ffi::c_int {
    2147483647 as ::core::ffi::c_int
}
pub unsafe fn fdforkpty(
    _ptmfd: ::core::ffi::c_int,
    mut master: *mut ::core::ffi::c_int,
    mut name: *mut ::core::ffi::c_char,
    mut tio: *mut termios,
    mut ws: *mut winsize,
) -> pid_t {
    unsafe { forkpty(master, name, tio, ws) as pid_t }
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

#[cfg(test)]
#[path = "../tests/test_compat_fdforkpty.rs"]
mod tests;
