//! Timer registrations survive the reactor's fork reset in both processes.
#![cfg(unix)]

use hmux2::src::reactor::{defer, poll_runtime, reset_after_fork, shutdown_runtime, Timer};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

#[test]
fn pending_timers_keep_their_deadline_after_fork() {
    let calls = Rc::new(Cell::new(0));
    let observed = calls.clone();
    let timer = Timer::new(Duration::from_millis(500), move || {
        observed.set(observed.get() + 1)
    })
    .unwrap();
    let deadline = timer.deadline();
    poll_runtime();
    assert_eq!(calls.get(), 0, "the first poll registers the deadline wait");
    // Include a deferred callback whose task has not yet been polled.
    let deferred = Rc::new(Cell::new(false));
    let observed = deferred.clone();
    defer(move || observed.set(true));
    let pid = unsafe { libc::fork() };
    assert!(pid >= 0, "fork: {}", std::io::Error::last_os_error());
    if pid == 0 {
        unsafe {
            // Bound failures which leave an inherited wait unable to wake.
            libc::alarm(5);
            if reset_after_fork().is_err() || timer.deadline() != deadline {
                libc::_exit(1);
            }
            while timer.is_pending() || !deferred.get() {
                poll_runtime();
            }
            shutdown_runtime();
            libc::_exit(if calls.get() == 1 { 0 } else { 2 });
        }
    }
    // Resetting and cancelling child tasks must not disturb the parent runtime.
    assert_eq!(timer.deadline(), deadline);
    while timer.is_pending() || !deferred.get() {
        poll_runtime();
    }
    assert_eq!(calls.get(), 1);
    let mut status = 0;
    assert_eq!(unsafe { libc::waitpid(pid, &mut status, 0) }, pid);
    assert!(libc::WIFEXITED(status), "child status {status}");
    assert_eq!(libc::WEXITSTATUS(status), 0);
    drop(timer);
    shutdown_runtime();
}
