//! Log output is checked per thread; fatal exits run in child processes.

use super::*;

/// The name of the variable a child process is told it is one by.
const CHILD: &str = "TMUX_C2RS_LOG_TEST_CHILD";

/// Owns this thread's log and a filename unique to the test thread.
struct Log {
    name: CString,
}

impl Log {
    fn new() -> Log {
        log_close();
        log_level.set(0);
        let log = Log {
            name: CString::new(format!("unit-test-{:?}", std::thread::current().id())).unwrap(),
        };
        log.forget();
        log
    }

    /// Where `log_open` puts what it writes, which is the name it is given
    /// and this process's id.
    fn path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(format!(
            "tmux-{}-{}.log",
            self.name.to_str().unwrap(),
            std::process::id()
        ))
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
        log_open(&self.name);
    }
}

impl Drop for Log {
    fn drop(&mut self) {
        log_close();
        log_level.set(0);
        self.forget();
    }
}

#[test]
fn the_level_starts_at_nothing_and_goes_up_one_at_a_time() {
    let log = Log::new();
    {
        assert_eq!(log_get_level(), 0);
        log_add_level();
        assert_eq!(log_get_level(), 1);
        log_add_level();
        assert_eq!(log_get_level(), 2);
    }
    drop(log);
}

#[test]
fn a_log_at_level_zero_is_not_opened_at_all() {
    let log = Log::new();
    log.open();
    log_debug(c"nothing", fmt_args![]);
    assert!(!log.path().exists());
    assert_eq!(log.lines(), Vec::<String>::new());
}

#[test]
fn a_log_that_is_open_takes_what_is_written_to_it() {
    let log = Log::new();
    {
        log_add_level();
        log.open();
        log_debug(c"one %d", fmt_args![1 as c_int]);
        log_debug(c"two %s", fmt_args![c"here".as_ptr()]);
    }
    assert_eq!(log.lines(), ["one 1", "two here"]);
}

#[test]
fn what_is_written_is_escaped() {
    let log = Log::new();
    {
        log_add_level();
        log.open();
        log_debug(c"a\nb\tc\x07d\x80e", fmt_args![]);
    }
    assert_eq!(log.lines(), ["a\\nb\\tc\\ad\\200e"]);
}

#[test]
fn a_log_that_is_closed_takes_nothing_more() {
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
}

#[test]
fn opening_a_log_twice_carries_on_where_the_first_left_off() {
    let log = Log::new();
    {
        log_add_level();
        log.open();
        log_debug(c"first", fmt_args![]);
        log.open();
        log_debug(c"second", fmt_args![]);
    }
    assert_eq!(log.lines(), ["first", "second"]);
}

#[test]
fn toggling_opens_the_log_and_toggling_again_closes_it() {
    let log = Log::new();
    {
        log_toggle(&log.name);
        assert_eq!(log_get_level(), 1);
        log_debug(c"between", fmt_args![]);
        log_add_level();
        log_toggle(&log.name);
        assert_eq!(log_get_level(), 0);
        log_debug(c"after", fmt_args![]);
    }
    assert_eq!(log.lines(), ["log opened", "between", "log closed"]);
}

#[test]
fn opening_a_log_does_not_consume_runtime_state() {
    let log = Log::new();
    log_add_level();
    log.open();
    assert!(log.lines().is_empty());
}

#[test]
fn the_level_can_be_borrowed_and_is_given_back() {
    let log = Log::new();
    {
        assert_eq!(log_with_level(3, log_get_level), 3);
        assert_eq!(log_get_level(), 0);
    }
    drop(log);
}

#[test]
fn a_log_that_cannot_be_opened_stays_closed() {
    let log = Log::new();
    {
        log_add_level();
        log_open(c"no/such/place");
        log_debug(c"nowhere", fmt_args![]);
        assert_eq!(log_get_level(), 1);
    }
    assert!(!log.path().exists());
}

#[test]
fn a_log_is_named_after_what_it_was_opened_with() {
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
}

#[test]
fn concurrent_logs_keep_levels_files_and_reopens_independent() {
    let log = Log::new();
    log_add_level();
    log.open();
    let barrier = std::sync::Barrier::new(5);
    std::thread::scope(|scope| {
        for worker in 0..4 {
            let barrier = &barrier;
            scope.spawn(move || {
                barrier.wait();
                assert_eq!(log_get_level(), 0);
                let log = Log::new();
                for _ in 0..worker + 2 {
                    log_add_level();
                }
                log.open();
                for record in 0..100 {
                    log_debug(c"worker %d record %d", fmt_args![worker, record]);
                    log_close();
                    log.open();
                }
                assert_eq!(log_get_level(), worker + 2);
                assert_eq!(
                    log.lines(),
                    (0..100)
                        .map(|record| format!("worker {worker} record {record}"))
                        .collect::<Vec<_>>()
                );
            });
        }
        barrier.wait();
        log_debug(c"parent", fmt_args![]);
    });
    assert_eq!(log_get_level(), 1);
    assert_eq!(log.lines(), ["parent"]);
}

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
    let was = log_level.replace(level);
    let answer = body();
    log_level.set(was);
    answer
}
