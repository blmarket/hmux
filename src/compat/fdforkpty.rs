use crate::ffi::forkpty;
pub use crate::types::*;
pub fn getptmfd() -> core::ffi::c_int {
    2147483647 as core::ffi::c_int
}

pub struct FdForkptyResult {
    pub pid: pid_t,
    pub master_fd: core::ffi::c_int,
    pub tty_name: [u8; 32],
}

pub unsafe fn fdforkpty(
    _ptmfd: core::ffi::c_int,
    tio: Option<&termios>,
    ws: Option<&winsize>,
) -> FdForkptyResult {
    let mut master_fd = 0;
    let mut tty_name = [0; 32];
    let pid = unsafe {
        forkpty(
            &raw mut master_fd,
            tty_name.as_mut_ptr().cast(),
            tio.map_or(core::ptr::null(), |tio| tio as *const termios),
            ws.map_or(core::ptr::null(), |ws| ws as *const winsize),
        ) as pid_t
    };
    FdForkptyResult {
        pid,
        master_fd,
        tty_name,
    }
}

#[cfg(test)]
#[path = "../tests/test_compat_fdforkpty.rs"]
mod tests;

#[cfg(test)]
pub use crate::consts::INT_MAX;

#[cfg(test)]
pub use crate::consts::__INT_MAX__;
