//! Each process initializes its own reactor after the fork.
#![cfg(unix)]

use hmux2::src::reactor::{poll_runtime, shutdown_runtime, Task, Timer};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

fn run_callbacks() {
    let calls = Rc::new(Cell::new(0));
    let observed = calls.clone();
    let timer = Timer::new(Duration::from_millis(5), move || {
        observed.set(observed.get() + 1)
    })
    .unwrap();
    let deferred = Rc::new(Cell::new(false));
    let observed = deferred.clone();
    let mut task = Task::new();
    task.start(move || Ok(async move { observed.set(true) }))
        .unwrap();
    while timer.is_pending() || !deferred.get() {
        poll_runtime();
    }
    assert_eq!(calls.get(), 1);
    drop(timer);
    shutdown_runtime();
}

#[test]
fn runtimes_initialized_after_fork_run_independently() {
    let pid = unsafe { libc::fork() };
    assert!(pid >= 0, "fork: {}", std::io::Error::last_os_error());
    if pid == 0 {
        unsafe { libc::alarm(5) };
        let succeeded = std::panic::catch_unwind(run_callbacks).is_ok();
        unsafe { libc::_exit(if succeeded { 0 } else { 1 }) };
    }
    run_callbacks();
    let mut status = 0;
    assert_eq!(unsafe { libc::waitpid(pid, &mut status, 0) }, pid);
    assert!(libc::WIFEXITED(status), "child status {status}");
    assert_eq!(libc::WEXITSTATUS(status), 0);
}
