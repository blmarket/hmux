use crate::ffi::{ioctl, tcgetpgrp};
use crate::reactor;
pub use crate::types::*;
use ::std::ffi::CString;
use ::std::fs;
use ::std::os::unix::ffi::OsStringExt;
pub const TIOCGSID: ::core::ffi::c_int = 0x5429 as ::core::ffi::c_int;
/// The command the foreground process group on `fd` was started with, read
/// out of `/proc`, or nothing when there is no such group or it has no
/// command line.
pub unsafe fn osdep_get_name(
    fd: ::core::ffi::c_int,
    _tty: *mut ::core::ffi::c_char,
) -> Option<CString> {
    unsafe {
        let pgrp: pid_t = tcgetpgrp(fd) as pid_t;
        if pgrp == -(1 as ::core::ffi::c_int) {
            return None;
        }
        let cmdline = fs::read(format!("/proc/{pgrp}/cmdline")).ok()?;
        let argv0: Vec<u8> = cmdline.into_iter().take_while(|&byte| byte != 0).collect();
        if argv0.is_empty() {
            return None;
        }
        Some(CString::from_vec_unchecked(argv0))
    }
}
pub fn osdep_get_cwd(mut fd: ::core::ffi::c_int) -> Option<CString> {
    unsafe {
        let pgrp: pid_t = tcgetpgrp(fd) as pid_t;
        if pgrp == -(1 as ::core::ffi::c_int) {
            return None;
        }
        let mut target = fs::read_link(format!("/proc/{pgrp}/cwd")).ok();
        if target.is_none() {
            let mut sid: pid_t = 0;
            if ioctl(fd, TIOCGSID as ::core::ffi::c_ulong, &raw mut sid)
                != -(1 as ::core::ffi::c_int)
            {
                target = fs::read_link(format!("/proc/{sid}/cwd")).ok();
            }
        }
        let target = target?;
        let bytes = target.into_os_string().into_vec();
        if bytes.is_empty() {
            return None;
        }
        CString::new(bytes).ok()
    }
}
pub fn osdep_event_init() -> reactor::Base {
    reactor::current()
}

#[cfg(test)]
#[path = "tests/test_osdep_linux.rs"]
mod tests;
