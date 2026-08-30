//! The debug log: the file `log_debug` writes to, the level that decides
//! whether there is one at all, and the two `fatal` calls that put one last
//! line in it before ending the process.
//!
//! A log is opened only above level zero, is named after what opened it and
//! the process it belongs to, and is line-buffered so that a crash loses
//! nothing already written. Every message is escaped before it goes in, so
//! that a pane's own bytes cannot break the line or the file, and is stamped
//! with the time it was written. Runtime messages are written by the daemon
//! paths that produce them.
//!
//! `log_debug`, `fatal` and `fatalx` take their arguments as a `FmtArg` slice
//! and expand the format string with the crate's own printf engine, so nothing
//! here needs a C calling convention.
//!
//! Coverage exemptions: `fatal` and `fatalx`, which end the process — a test
//! that entered one would take the whole run with it. They are also the only
//! callers that reach `log_vwrite` with no log open, so its first guard is
//! exempt with them. In the test module, the two lines that build the panic
//! message for a child process are only read once a test has already failed.
use crate::compat::stravis;
use crate::ffi::{
    __errno_location, exit, fclose, fflush, fopen, fprintf, getpid, gettimeofday, setvbuf,
    snprintf, strerror,
};
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc};
pub use crate::types::*;
use ::core::ffi::{CStr, c_char, c_int, c_long, c_longlong};
use ::core::ptr::null_mut;

pub const _IOLBF: c_int = 1;
pub const VIS_OCTAL: c_int = 0x1;
pub const VIS_CSTYLE: c_int = 0x2;
pub const VIS_TAB: c_int = 0x8;
pub const VIS_NL: c_int = 0x10;

/// How a message is escaped on its way into the log: control bytes as C
/// escapes where there is one and octal where there is not, tabs and newlines
/// included, so that one message stays one line.
const ESCAPING: c_int = VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL;

/// The log that is open, or nothing.
static mut log_file: *mut FILE = null_mut();

/// How much is logged: nothing at all at zero, everything above it. Only the
/// guards in front of the calls that build a message read it past that, so the
/// levels above one are the callers' to tell apart.
static mut log_level: c_int = 0;

pub fn log_add_level() {
    unsafe {
        log_level += 1;
    }
}

pub fn log_get_level() -> c_int {
    unsafe { log_level }
}

/// Puts the debug level back where a test found it. What the level changes is
/// the guards in front of the calls that build a message first; whether
/// anything is written out as well wants a log that has been opened, which
/// only this module's own tests do.
#[cfg(test)]
pub(crate) fn log_with_level<T>(level: c_int, body: impl FnOnce() -> T) -> T {
    unsafe {
        let was = log_level;
        log_level = level;
        let answer = body();
        log_level = was;
        answer
    }
}

/// Opens the log for `name`, in a file named after it and this process. A level
/// of zero opens nothing, and a file that would not open leaves the log closed
/// without saying so.
pub unsafe fn log_open(name: *const c_char) {
    unsafe {
        if log_level == 0 {
            return;
        }
        log_close();
        let mut path = b"tmux-".to_vec();
        path.extend_from_slice(CStr::from_ptr(name).to_bytes());
        path.extend_from_slice(format!("-{}.log\0", getpid() as c_long).as_bytes());
        log_file = fopen(path.as_ptr() as *const c_char, c"a".as_ptr());
        if log_file.is_null() {
            return;
        }
        setvbuf(log_file, null_mut::<c_char>(), _IOLBF, 0);
    }
}

/// Turns the log on if it is off and off if it is on, writing the change into
/// the log itself on either side of it.
pub unsafe fn log_toggle(name: *const c_char) {
    unsafe {
        if log_level == 0 {
            log_level = 1;
            log_open(name);
            log_debug(c"log opened".as_ptr(), fmt_args![]);
        } else {
            log_debug(c"log closed".as_ptr(), fmt_args![]);
            log_level = 0;
            log_close();
        }
    }
}

pub fn log_close() {
    unsafe {
        if !log_file.is_null() {
            fclose(log_file);
        }
        log_file = null_mut();
    }
}

/// Writes one line to the log: the time, `prefix`, and `msg` filled in from
/// `ap` and escaped. Nothing is written if there is no log open, if the
/// message could not be built or if it could not be escaped.
unsafe fn log_vwrite(msg: *const c_char, args: &[FmtArg], prefix: &CStr) {
    unsafe {
        if log_file.is_null() {
            return;
        }
        let built = format_alloc(msg, args);
        let escaped = stravis(built.as_ptr(), ESCAPING);
        let mut tv = timeval {
            tv_sec: 0,
            tv_usec: 0,
        };
        gettimeofday(&raw mut tv, null_mut());
        if fprintf(
            log_file,
            c"%lld.%06d %s%s\n".as_ptr(),
            tv.tv_sec as c_longlong,
            tv.tv_usec as c_int,
            prefix.as_ptr(),
            escaped.as_ptr(),
        ) != -1
        {
            fflush(log_file);
        }
    }
}

pub unsafe fn log_debug(msg: *const c_char, args: &[FmtArg]) {
    unsafe {
        if log_file.is_null() {
            return;
        }
        log_vwrite(msg, args, c"");
    }
}

pub unsafe fn fatal(msg: *const c_char, args: &[FmtArg]) -> ! {
    unsafe {
        let mut prefix = [0 as c_char; 256];
        if snprintf(
            prefix.as_mut_ptr(),
            256,
            c"fatal: %s: ".as_ptr(),
            strerror(*__errno_location()),
        ) < 0
        {
            exit(1);
        }
        log_vwrite(msg, args, CStr::from_ptr(prefix.as_ptr()));
        exit(1);
    }
}

pub unsafe fn fatalx(msg: *const c_char, args: &[FmtArg]) -> ! {
    unsafe {
        log_vwrite(msg, args, c"fatal: ");
        exit(1);
    }
}

#[cfg(test)]
#[path = "tests/test_log.rs"]
mod tests;
