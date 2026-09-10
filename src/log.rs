//! The debug log: the file `log_debug` writes to, the level that decides
//! whether there is one at all, and the two `fatal` calls that put one last
//! line in it before ending the process.
//!
//! A log is opened only above level zero, is named after what opened it and
//! the process it belongs to, and is unbuffered so that a crash loses
//! nothing already written. Every message is escaped before it goes in, so
//! that a pane's own bytes cannot break the line or the file, and is stamped
//! with the time it was written. Runtime messages are written by the daemon
//! paths that produce them.
//!
//! `log_debug`, `fatal` and `fatalx` take their arguments as a `FmtArg` slice
//! and expand the format string with the crate's own printf engine, so nothing
//! here needs a C calling convention.
//!
//! Fatal logging is tested in child processes so its exit cannot end the
//! test runner. The panic messages for failed child processes are only read
//! once a test has already failed.
use crate::compat::error_message;
use crate::compat::stravis;
use crate::ffi::__errno_location;
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc, format_bytes};
pub use crate::types::*;
use ::core::ffi::{CStr, c_int};
use ::std::ffi::{CString, OsStr};
use ::std::fs::{File, OpenOptions};
use ::std::io::Write;
use ::std::os::unix::ffi::OsStrExt;

pub use crate::consts::{VIS_CSTYLE, VIS_NL, VIS_OCTAL, VIS_TAB};

/// How a message is escaped on its way into the log: control bytes as C
/// escapes where there is one and octal where there is not, tabs and newlines
/// included, so that one message stays one line.
const ESCAPING: c_int = VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL;

const log_file_FIELD: crate::server_state::LocalField<
    std::rc::Rc<std::cell::RefCell<Option<File>>>,
> = crate::server_state::LocalField::new(|state| &state.log_file);

const log_level: crate::server_state::LocalField<std::cell::Cell<i32>> =
    crate::server_state::LocalField::new(|state| &state.log_level);

pub fn log_add_level() {
    log_level.with(|counter| {
        let previous = counter.get();
        counter.set(previous.wrapping_add(1));
        previous
    });
}

pub fn log_get_level() -> c_int {
    log_level.try_with(std::cell::Cell::get).unwrap_or(0)
}

/// Opens the log for `name`, in a file named after it and this process. A level
/// of zero opens nothing, and a file that would not open leaves the log closed
/// without saying so.
pub fn log_open(name: &CStr) {
    let log_file = log_file_FIELD.get();

    if log_level.get() == 0 {
        return;
    }
    let mut active = log_file.borrow_mut();
    drop(active.take());
    let mut path = b"tmux-".to_vec();
    path.extend_from_slice(name.to_bytes());
    path.extend_from_slice(format!("-{}.log", std::process::id()).as_bytes());
    *active = OpenOptions::new()
        .create(true)
        .append(true)
        .open(OsStr::from_bytes(&path))
        .ok();
}

/// Turns the log on if it is off and off if it is on, writing the change into
/// the log itself on either side of it.
pub fn log_toggle(name: &CStr) {
    if log_level.get() == 0 {
        log_level.set(1);
        log_open(name);
        log_debug(c"log opened", fmt_args![]);
    } else {
        log_debug(c"log closed", fmt_args![]);
        log_level.set(0);
        log_close();
    }
}

pub fn log_close() {
    let log_file = log_file_FIELD.get();

    let mut active = log_file.borrow_mut();
    drop(active.take());
}

/// Writes one line to the log: the time, `prefix`, and `msg` filled in from
/// `ap` and escaped. Nothing is written if there is no log open, if the
/// message could not be built or if it could not be escaped.
fn log_vwrite(msg: &CStr, args: &[FmtArg], prefix: &CStr) {
    let Ok(log_file) = log_file_FIELD.try_with(std::rc::Rc::clone) else {
        return;
    };

    let mut active = log_file.borrow_mut();
    let Some(file) = active.as_mut() else {
        return;
    };
    let built = format_alloc(msg, args);
    let escaped = stravis(&built, ESCAPING);
    let tv = timeval::now();
    let mut line = format!("{}.{:06} ", tv.tv_sec, tv.tv_usec).into_bytes();
    line.extend_from_slice(prefix.to_bytes());
    line.extend_from_slice(escaped.to_bytes());
    line.push(b'\n');
    let _ = file.write_all(&line);
}

pub fn log_debug(msg: &CStr, args: &[FmtArg]) {
    log_vwrite(msg, args, c"");
}

pub fn fatal(msg: &CStr, args: &[FmtArg]) -> ! {
    let error = error_message(unsafe { *__errno_location() });
    let mut prefix = format_bytes(c"fatal: %s: ", fmt_args![error.as_c_str()]);
    prefix.truncate(255);
    let prefix = CString::new(prefix).expect("the fatal prefix contains no NUL");
    log_vwrite(msg, args, &prefix);
    std::process::exit(1);
}

pub fn fatalx(msg: &CStr, args: &[FmtArg]) -> ! {
    log_vwrite(msg, args, c"fatal: ");
    std::process::exit(1);
}

#[cfg(test)]
#[path = "tests/test_log.rs"]
mod tests;

#[cfg(test)]
pub(crate) use tests::log_with_level;
