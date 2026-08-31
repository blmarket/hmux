//! What is left uncovered here, and why. `fatal` and `fatalx` end the
//! process, so a unit test that entered one would take the whole run with
//! it; they are also the only callers that reach `log_vwrite` with no log
//! open, so its first guard goes with them. The failure arm left inside
//! `log_vwrite`, for an escaping that could not be made, is the C
//! allocator's to answer, and a test cannot make it refuse.

use super::*;

/// The name of the variable a child process is told it is one by.
const CHILD: &str = "TMUX_C2RS_LOG_TEST_CHILD";

/// Runs `test` in a child process of its own and answers whether it did —
/// which is to say whether this process is the parent.
///
/// Everything this module keeps is process-wide, and both halves of it are
/// reached from outside: every module's `log_debug` writes to whatever file
/// this one has open, and the debug level is read by the guards in front of
/// those calls, which another module's tests borrow through
/// [`log_with_level`]. So a test that opens the log or moves the level
/// cannot run beside cargo's other test threads — one of them writing to
/// the file while this one closes it is a use-after-free, which showed up
/// as a segmentation fault about one run in three. The child is this same
/// test binary with one test selected and one thread to run it on.
fn in_a_child_process(test: &str) -> bool {
    if ::std::env::var_os(CHILD).is_some() {
        return false;
    }
    let exe = ::std::env::current_exe().expect("the test binary");
    let out = ::std::process::Command::new(exe)
        .args(["--exact", test, "--test-threads=1", "--nocapture"])
        .env(CHILD, "1")
        .output()
        .expect("the child process ran");
    assert!(
        out.status.success(),
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    true
}

/// A test of the log, which is one that runs in a child process of its own.
macro_rules! log_test {
    ($name:ident, $body:block) => {
        #[test]
        fn $name() {
            if in_a_child_process(concat!("log::tests::", stringify!($name))) {
                return;
            }
            $body
        }
    };
}

/// A turn at the log — the level and open file, both this module's own
/// statics — starting from a
/// closed log at level zero and leaving one behind. The file the log is
/// written to is named after the process, so it is the same path for every
/// test and is taken away again here whether the test passed or not.
struct Log;

impl Log {
    fn new() -> Log {
        log_close();
        log_level.store(0, Ordering::Relaxed);
        let log = Log;
        log.forget();
        log
    }

    /// Where `log_open` puts what it writes, which is the name it is given
    /// and this process's id.
    fn path(&self) -> ::std::path::PathBuf {
        ::std::path::PathBuf::from(format!("tmux-unit-test-{}.log", ::std::process::id()))
    }

    /// What has been written to the log so far, with the timestamp in
    /// front of each line taken off.
    fn lines(&self) -> Vec<String> {
        ::std::fs::read_to_string(self.path())
            .unwrap_or_default()
            .lines()
            .map(|line| {
                line.split_once(' ')
                    .expect("a timestamp and a message")
                    .1
                    .to_owned()
            })
            .collect()
    }

    fn forget(&self) {
        let _ = ::std::fs::remove_file(self.path());
    }

    fn open(&self) {
        unsafe { log_open(c"unit-test".as_ptr()) };
    }
}

impl Drop for Log {
    fn drop(&mut self) {
        log_close();
        log_level.store(0, Ordering::Relaxed);
        self.forget();
    }
}

log_test!(the_level_starts_at_nothing_and_goes_up_one_at_a_time, {
    let log = Log::new();
    {
        assert_eq!(log_get_level(), 0);
        log_add_level();
        assert_eq!(log_get_level(), 1);
        log_add_level();
        assert_eq!(log_get_level(), 2);
    }
    drop(log);
});

log_test!(a_log_at_level_zero_is_not_opened_at_all, {
    let log = Log::new();
    log.open();
    unsafe { log_debug(c"nothing".as_ptr(), fmt_args![]) };
    assert!(!log.path().exists());
    assert_eq!(log.lines(), Vec::<String>::new());
});

log_test!(a_log_that_is_open_takes_what_is_written_to_it, {
    let log = Log::new();
    unsafe {
        log_add_level();
        log.open();
        log_debug(c"one %d".as_ptr(), fmt_args![1 as ::core::ffi::c_int]);
        log_debug(c"two %s".as_ptr(), fmt_args![c"here".as_ptr()]);
    }
    assert_eq!(log.lines(), ["one 1", "two here"]);
});

log_test!(what_is_written_is_escaped, {
    let log = Log::new();
    unsafe {
        log_add_level();
        log.open();
        log_debug(c"a\nb\tc\x07d\x80e".as_ptr(), fmt_args![]);
    }
    assert_eq!(log.lines(), ["a\\nb\\tc\\ad\\200e"]);
});

log_test!(a_log_that_is_closed_takes_nothing_more, {
    let log = Log::new();
    unsafe {
        log_add_level();
        log.open();
        log_debug(c"before".as_ptr(), fmt_args![]);
        log_close();
        log_debug(c"after".as_ptr(), fmt_args![]);
        log_close();
    }
    assert_eq!(log.lines(), ["before"]);
});

log_test!(opening_a_log_twice_carries_on_where_the_first_left_off, {
    let log = Log::new();
    unsafe {
        log_add_level();
        log.open();
        log_debug(c"first".as_ptr(), fmt_args![]);
        log.open();
        log_debug(c"second".as_ptr(), fmt_args![]);
    }
    assert_eq!(log.lines(), ["first", "second"]);
});

log_test!(toggling_opens_the_log_and_toggling_again_closes_it, {
    let log = Log::new();
    unsafe {
        log_toggle(c"unit-test".as_ptr());
        assert_eq!(log_get_level(), 1);
        log_debug(c"between".as_ptr(), fmt_args![]);
        log_add_level();
        log_toggle(c"unit-test".as_ptr());
        assert_eq!(log_get_level(), 0);
        log_debug(c"after".as_ptr(), fmt_args![]);
    }
    assert_eq!(log.lines(), ["log opened", "between", "log closed"]);
});

log_test!(opening_a_log_does_not_consume_runtime_state, {
    let log = Log::new();
    log_add_level();
    log.open();
    assert!(log.lines().is_empty());
});

log_test!(the_level_can_be_borrowed_and_is_given_back, {
    let log = Log::new();
    {
        assert_eq!(log_with_level(3, log_get_level), 3);
        assert_eq!(log_get_level(), 0);
    }
    drop(log);
});

log_test!(a_log_that_cannot_be_opened_stays_closed, {
    let log = Log::new();
    unsafe {
        log_add_level();
        log_open(c"no/such/place".as_ptr());
        log_debug(c"nowhere".as_ptr(), fmt_args![]);
        assert_eq!(log_get_level(), 1);
    }
    assert!(!log.path().exists());
});

log_test!(a_log_is_named_after_what_it_was_opened_with, {
    let log = Log::new();
    let other = ::std::path::PathBuf::from(format!("tmux-other-name-{}.log", ::std::process::id()));
    let _ = ::std::fs::remove_file(&other);
    unsafe {
        log_add_level();
        log_open(c"other-name".as_ptr());
        log_debug(c"in the other one".as_ptr(), fmt_args![]);
        log_close();
    }
    assert!(!log.path().exists());
    assert!(
        ::std::fs::read_to_string(&other)
            .expect("the other log")
            .contains("in the other one")
    );
    let _ = ::std::fs::remove_file(&other);
});
