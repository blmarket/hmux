//! The arm that cuts the title back to the last space in it is only
//! reached when the whole of "program: title" ran past the sixteen
//! characters a thread name holds *and* a space is still inside them,
//! which wants a program name of a dozen characters or fewer. The test
//! binary's own name is longer than that, so the test for it runs in a
//! child process called something shorter.

use super::*;

use crate::fmt_args;
use ::core::ffi::{CStr, c_char};
use ::std::sync::MutexGuard;

pub const PR_GET_NAME: ::core::ffi::c_int = 16 as ::core::ffi::c_int;

/// A turn at the name of the thread the tests run on, which is what the
/// title is written to and which is put back afterwards. Cargo runs the
/// tests on parallel threads; the name belongs to whichever thread asks,
/// so this is really a turn at asking about it.
struct Name {
    was: [c_char; 16],
    _guard: MutexGuard<'static, ()>,
}

impl Name {
    fn new() -> Name {
        static NAME: ::std::sync::Mutex<()> = ::std::sync::Mutex::new(());
        let guard = NAME.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut was = [0 as c_char; 16];
        unsafe { prctl(PR_GET_NAME, &raw mut was as *mut c_char) };
        Name { was, _guard: guard }
    }

    /// What the thread is called now.
    fn now(&self) -> String {
        let mut buf = [0 as c_char; 16];
        unsafe {
            prctl(PR_GET_NAME, &raw mut buf as *mut c_char);
            CStr::from_ptr(&raw const buf as *const c_char)
                .to_string_lossy()
                .into_owned()
        }
    }
}

impl Drop for Name {
    fn drop(&mut self) {
        unsafe { prctl(PR_SET_NAME, &raw mut self.was as *mut c_char) };
    }
}

/// The title is written behind the program's own name, and the whole of it
/// is cut down to the fifteen characters a thread name holds.
#[test]
fn a_title_is_written_behind_the_name_of_the_program() {
    let name = Name::new();
    unsafe {
        setproctitle(c"%s".as_ptr(), fmt_args![c"a-title".as_ptr()]);
    }
    let expected: String = format!("{}: a-title", getprogname().to_string_lossy())
        .chars()
        .take(15)
        .collect();
    assert_eq!(name.now(), expected);
}

/// The title itself is cut down to sixteen characters before the program's
/// name is put in front of it, so what a long one leaves behind is its
/// first characters and nothing of the rest.
#[test]
fn a_long_title_is_cut_down_twice() {
    let name = Name::new();
    unsafe {
        setproctitle(
            c"%s-%d".as_ptr(),
            fmt_args![
                c"a-very-long-title-indeed".as_ptr(),
                7 as ::core::ffi::c_int
            ],
        );
    }
    let expected: String = format!("{}: a-very-long-tit", getprogname().to_string_lossy())
        .chars()
        .take(15)
        .collect();
    assert_eq!(name.now(), expected);
}

/// The name of the variable a child process is told it is one by.
const CHILD: &str = "TMUX_C2RS_SETPROCTITLE_TEST_CHILD";

/// Runs `test` in a child process of this test binary called `called`, and
/// answers whether it did — which is to say whether this process is the
/// parent.
///
/// Where the title is cut back to depends on how long the program's own
/// name is, and the C library takes that from `argv[0]`. The test binary's
/// name is far longer than the sixteen characters a thread name holds, so
/// the arm that cuts is only reached by a program with a short name; the
/// child is this same binary started under one, with a single test
/// selected and one thread to run it on.
fn in_a_child_called(called: &str, test: &str) -> bool {
    use ::std::os::unix::process::CommandExt;
    if ::std::env::var_os(CHILD).is_some() {
        return false;
    }
    let exe = ::std::env::current_exe().expect("the test binary");
    let out = ::std::process::Command::new(exe)
        .arg0(called)
        .args(["--exact", test, "--test-threads=1", "--nocapture"])
        .env(CHILD, "1")
        .output()
        .expect("the child process ran");
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "{said}");
    true
}

/// A "program: title" that ran past the sixteen characters is cut back
/// again, to the last space still inside them — so a one-word title leaves
/// nothing but the program's name and its colon behind, and a title of
/// several words keeps as many whole ones as fit.
#[test]
fn a_name_that_did_not_fit_is_cut_back_to_its_last_space() {
    if in_a_child_called(
        "sp",
        "src::compat::setproctitle::tests::a_name_that_did_not_fit_is_cut_back_to_its_last_space",
    ) {
        return;
    }
    let name = Name::new();
    assert_eq!(
        getprogname().to_bytes(),
        b"sp",
        "the child is not called what it was started as"
    );
    unsafe { setproctitle(c"%s".as_ptr(), fmt_args![c"a-long-title".as_ptr()]) };
    assert_eq!(name.now(), "sp:");
    unsafe { setproctitle(c"%s".as_ptr(), fmt_args![c"one two three four".as_ptr()]) };
    assert_eq!(name.now(), "sp: one two");
}
