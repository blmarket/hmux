//! `setproctitle`, which BSD has and Linux does not: what a process calls
//! itself where `ps` will show it.
//!
//! Linux has nothing that rewrites the command line, so tmux settles for the
//! nearest thing `prctl` offers — the name of the calling thread, which holds
//! sixteen bytes including the terminator. The title is formatted first, into
//! sixteen bytes of its own, and the program's own name is put in front of it
//! afterwards, so a long title is cut down twice.
//!
//! Coverage exemptions: none in the module. The one line the test module
//! leaves uncovered is the arm that takes a poisoned mutex back, which is only
//! read once another test has already failed.
use crate::compat::getprogname::getprogname;
use crate::ffi::prctl;
use crate::fmt_engine::{FmtArg, format_into};
use ::core::ffi::{CStr, c_char, c_int};

pub const PR_SET_NAME: c_int = 15;

/// How many bytes a thread name holds, the terminator that ends it included.
const NAME: usize = 16;

/// The thread name for a program called `program` showing `title`: the two
/// with a colon between them, cut down to the bytes a name holds.
///
/// What did not fit is cut back a second time, to the last space still inside
/// those bytes — so a title of several words keeps as many whole ones as fit,
/// and a title of one word leaves nothing behind but the program's name and
/// its colon. A program whose own name fills the name has no space left to cut
/// back to, and keeps the first fifteen bytes as they stand.
fn thread_name(program: &[u8], title: &[u8]) -> [c_char; NAME] {
    let mut whole = program.to_vec();
    whole.extend_from_slice(b": ");
    whole.extend_from_slice(title);
    let wanted = whole.len();
    whole.truncate(NAME - 1);
    if wanted >= NAME
        && let Some(space) = whole.iter().rposition(|byte| *byte == b' ')
    {
        whole.truncate(space);
    }
    let mut name = [0 as c_char; NAME];
    for (to, byte) in name.iter_mut().zip(&whole) {
        *to = *byte as c_char;
    }
    name
}

pub unsafe fn setproctitle(fmt: *const c_char, args: &[FmtArg]) {
    unsafe {
        let mut title = [0 as c_char; NAME];
        format_into(title.as_mut_ptr(), NAME, fmt, args);
        let name = thread_name(
            CStr::from_ptr(getprogname()).to_bytes(),
            CStr::from_ptr(title.as_ptr()).to_bytes(),
        );
        prctl(PR_SET_NAME, name.as_ptr());
    }
}

#[cfg(test)]
#[path = "../tests/test_compat_setproctitle.rs"]
mod tests;
