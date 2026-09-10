//! Log output and fatal process exits are checked in isolated child processes.

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
/// [`log_with_level`]. The child process keeps unrelated messages and level
/// changes out of the assertions. It is the same test binary with one test
/// selected and one test thread.
fn in_a_child_process(test: &str) -> bool {
    if std::env::var_os(CHILD).is_some() {
        return false;
    }
    let exe = std::env::current_exe().expect("the test binary");
    let out = std::process::Command::new(exe)
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
    fn path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(format!("tmux-unit-test-{}.log", std::process::id()))
    }

    /// What has been written to the log so far, with the timestamp in
    /// front of each line taken off.
    fn lines(&self) -> Vec<String> {
        std::fs::read_to_string(self.path())
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
        let _ = std::fs::remove_file(self.path());
    }

    fn open(&self) {
        log_open(c"unit-test");
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
    log_debug(c"nothing", fmt_args![]);
    assert!(!log.path().exists());
    assert_eq!(log.lines(), Vec::<String>::new());
});

log_test!(a_log_that_is_open_takes_what_is_written_to_it, {
    let log = Log::new();
    {
        log_add_level();
        log.open();
        log_debug(c"one %d", fmt_args![1 as c_int]);
        log_debug(c"two %s", fmt_args![c"here".as_ptr()]);
    }
    assert_eq!(log.lines(), ["one 1", "two here"]);
});

log_test!(what_is_written_is_escaped, {
    let log = Log::new();
    {
        log_add_level();
        log.open();
        log_debug(c"a\nb\tc\x07d\x80e", fmt_args![]);
    }
    assert_eq!(log.lines(), ["a\\nb\\tc\\ad\\200e"]);
});

log_test!(a_log_that_is_closed_takes_nothing_more, {
    let log = Log::new();
    {
        log_add_level();
        log.open();
        log_debug(c"before", fmt_args![]);
        log_close();
        log_debug(c"after", fmt_args![]);
        log_close();
    }
    assert_eq!(log.lines(), ["before"]);
});

log_test!(opening_a_log_twice_carries_on_where_the_first_left_off, {
    let log = Log::new();
    {
        log_add_level();
        log.open();
        log_debug(c"first", fmt_args![]);
        log.open();
        log_debug(c"second", fmt_args![]);
    }
    assert_eq!(log.lines(), ["first", "second"]);
});

log_test!(toggling_opens_the_log_and_toggling_again_closes_it, {
    let log = Log::new();
    {
        log_toggle(c"unit-test");
        assert_eq!(log_get_level(), 1);
        log_debug(c"between", fmt_args![]);
        log_add_level();
        log_toggle(c"unit-test");
        assert_eq!(log_get_level(), 0);
        log_debug(c"after", fmt_args![]);
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
    {
        log_add_level();
        log_open(c"no/such/place");
        log_debug(c"nowhere", fmt_args![]);
        assert_eq!(log_get_level(), 1);
    }
    assert!(!log.path().exists());
});

log_test!(a_log_is_named_after_what_it_was_opened_with, {
    let log = Log::new();
    let other = std::path::PathBuf::from(format!("tmux-other-name-{}.log", std::process::id()));
    let _ = std::fs::remove_file(&other);
    {
        log_add_level();
        log_open(c"other-name");
        log_debug(c"in the other one", fmt_args![]);
        log_close();
    }
    assert!(!log.path().exists());
    assert!(
        std::fs::read_to_string(&other)
            .expect("the other log")
            .contains("in the other one")
    );
    let _ = std::fs::remove_file(&other);
});

log_test!(concurrent_writes_and_reopens_keep_each_record_intact, {
    let log = Log::new();
    log_add_level();
    log.open();
    std::thread::scope(|scope| {
        for worker in 0..4 {
            scope.spawn(move || {
                for record in 0..100 {
                    log_debug(c"worker %d record %d", fmt_args![worker, record]);
                }
            });
        }
        scope.spawn(|| {
            for _ in 0..50 {
                log_close();
                log_open(c"unit-test");
            }
        });
    });
    log.open();
    {
        log_debug(c"final", fmt_args![]);
    }
    log_close();
    let lines = log.lines();
    assert_eq!(lines.last().map(String::as_str), Some("final"));
    for line in &lines[..lines.len() - 1] {
        let fields: Vec<_> = line.split_whitespace().collect();
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], "worker");
        assert_eq!(fields[2], "record");
        assert!(fields[1].parse::<u32>().unwrap() < 4);
        assert!(fields[3].parse::<u32>().unwrap() < 100);
    }
    assert_eq!(
        lines.iter().collect::<std::collections::HashSet<_>>().len(),
        lines.len()
    );
});

#[test]
fn fatal_preserves_errno_and_exits_with_or_without_an_open_log() {
    let test = "log::tests::fatal_preserves_errno_and_exits_with_or_without_an_open_log";
    if let Ok(mode) = std::env::var(CHILD) {
        unsafe {
            if mode == "fatal-open" {
                log_add_level();
                log_open(c"unit-test");
            }
            *__errno_location() = libc::EACCES;
            fatal(c"operation %s", fmt_args![c"failed\nretry"]);
        }
    }
    for mode in ["fatal-open", "fatal-closed"] {
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", test, "--test-threads=1", "--nocapture"])
            .env(CHILD, mode)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let path = format!("tmux-unit-test-{}.log", child.id());
        let out = child.wait_with_output().unwrap();
        let logged = std::fs::read_to_string(&path);
        let _ = std::fs::remove_file(&path);
        assert_eq!(out.status.code(), Some(1), "{out:?}");
        if mode == "fatal-open" {
            let logged = logged.unwrap();
            let error = error_message(libc::EACCES);
            assert_eq!(logged.lines().count(), 1);
            assert_eq!(
                logged.split_once(' ').unwrap().1,
                format!(
                    "fatal: {}: operation failed\\nretry\n",
                    error.to_string_lossy()
                )
            );
        } else {
            assert_eq!(logged.unwrap_err().kind(), std::io::ErrorKind::NotFound);
        }
    }
}

/// Puts the debug level back where a test found it. What the level changes is
/// the guards in front of the calls that build a message first; whether
/// anything is written out as well wants a log that has been opened, which
/// only this module's own tests do.
pub(crate) fn log_with_level<T>(level: c_int, body: impl FnOnce() -> T) -> T {
    let was = log_level.swap(level, Ordering::Relaxed);
    let answer = body();
    log_level.store(was, Ordering::Relaxed);
    answer
}
