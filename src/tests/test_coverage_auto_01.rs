//! Unit tests for [`crate::job`], the job wrapper around `fork` / `fdforkpty`.
//!
//! Covers lifecycle constants, callback snapshots, registered job streams,
//! and process startup, transfer, and cleanup through the production job API.

use crate::job::{
    JOB_CLOSED, JOB_DEAD, JOB_DEFAULTSHELL, JOB_KEEPWRITE, JOB_NOWAIT, JOB_PTY, JOB_RUNNING,
    JOB_SHOWSTDERR, job_check_died, job_free, job_kill_all, job_print_summary, job_resize, job_run,
    job_still_running, job_transfer,
};
use crate::tests::test_fixtures::{Item, ensure_reactor, globals, zeroed};
use crate::types::job;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// `JOB_RUNNING`, `JOB_DEAD` and `JOB_CLOSED` number the lifecycle
/// consecutively from zero, which pins both the values and the absence of
/// collisions. The order matters: the callbacks compare `state == JOB_DEAD`
/// vs `state == JOB_CLOSED` to decide whether to complete or close.
#[test]
fn the_job_state_constants_are_consecutive_from_zero() {
    assert_eq!(JOB_RUNNING, 0);
    assert_eq!(JOB_DEAD, 1);
    assert_eq!(JOB_CLOSED, 2);
    assert!(JOB_RUNNING < JOB_DEAD);
    assert!(JOB_DEAD < JOB_CLOSED);
}

/// The `JOB_*` flag bits are distinct powers of two at the upstream values,
/// so combining them with `|` never collides.
#[test]
fn the_job_flag_constants_are_distinct_bits() {
    assert_eq!(JOB_NOWAIT, 0x1);
    assert_eq!(JOB_KEEPWRITE, 0x2);
    assert_eq!(JOB_PTY, 0x4);
    assert_eq!(JOB_DEFAULTSHELL, 0x8);
    assert_eq!(JOB_SHOWSTDERR, 0x10);
    // pairwise disjoint
    let flags = [
        JOB_NOWAIT,
        JOB_KEEPWRITE,
        JOB_PTY,
        JOB_DEFAULTSHELL,
        JOB_SHOWSTDERR,
    ];
    for i in 0..flags.len() {
        for j in (i + 1)..flags.len() {
            assert_eq!(
                flags[i] & flags[j],
                0,
                "flags 0x{:x} and 0x{:x} overlap",
                flags[i],
                flags[j]
            );
        }
    }
    // all together is the expected mask
    assert_eq!(
        JOB_NOWAIT | JOB_KEEPWRITE | JOB_PTY | JOB_DEFAULTSHELL | JOB_SHOWSTDERR,
        0x1f
    );
}

// ---------------------------------------------------------------------------
// Pure accessors over a minimal job
// ---------------------------------------------------------------------------

/// Event snapshots retain the job status at the time they were captured.
#[test]
fn job_event_snapshot_retains_the_status() {
    let mut j = zeroed::<job>();
    j.status = 42;
    let snapshot = j.event_state();
    assert_eq!(snapshot.status, 42);
    j.status = -7;
    assert_eq!(j.event_state().status, -7);
    assert_eq!(snapshot.status, 42);
}

// ---------------------------------------------------------------------------
// Global-list helpers when no jobs exist
// ---------------------------------------------------------------------------

/// With no registered job `job_still_running` has nothing to wait for and
/// answers 0.
#[test]
fn still_running_is_zero_when_no_jobs_exist() {
    let _guard = globals();
    let v = job_still_running();
    assert_eq!(v, 0);
}

/// `job_kill_all` walks the list and sends `SIGTERM` to every entry. With
/// an empty list it does nothing and does not crash; `job_still_running`
/// stays 0 afterwards.
#[test]
fn kill_all_is_a_noop_when_no_jobs_exist() {
    let _guard = globals();
    job_kill_all();
    let v = job_still_running();
    assert_eq!(v, 0);
}

/// `job_check_died` for a pid that is not in the list simply returns, with
/// no effect on `job_still_running`.
#[test]
fn check_died_with_unknown_pid_does_nothing() {
    let _guard = globals();
    job_check_died(999999, 0);
    assert_eq!(job_still_running(), 0);
}

/// `job_check_died` ignores a `WIFSTOPPED` status for `SIGTTIN`/`SIGTTOU`
/// (the shell stopping itself for job control). Even with a stopped status
/// and an unknown pid the call must not panic or treat the job as dead.
#[test]
fn check_died_ignores_stopped_sigttin_and_sigttou() {
    let _guard = globals();
    // WIFSTOPPED is (status & 0xff == 0x7f) and WSTOPSIG is (status>>8)&0xff.
    let stopped_sigttin: core::ffi::c_int = (21 << 8) | 0x7f; // SIGTTIN = 21
    let stopped_sigttou: core::ffi::c_int = (22 << 8) | 0x7f; // SIGTTOU = 22
    {
        job_check_died(999998, stopped_sigttin);
        job_check_died(999997, stopped_sigttou);
    }
    assert_eq!(job_still_running(), 0);
}

#[test]
fn test_job_run_and_lifecycle() {
    let _guard = globals();
    ensure_reactor();
    let proc = crate::proc::ProcessRef::default();
    let prev_proc = unsafe { crate::server::server_proc.take() };
    unsafe { crate::server::server_proc = Some(proc) };

    unsafe {
        let j = job_run(Some(c"true"), &[], None, None, None, None, None, 0, 80, 24);
        let id = j.expect("the job starts");
        assert!(crate::job::job_event_by_id(id).is_some_and(|event| !event.is_none()));
        assert_eq!(job_still_running(), 1);

        let mut item = Item::new();
        job_print_summary(&item.read(), 1);

        job_resize(id, 100, 30);
        job_kill_all();
        job_free(id);
        assert_eq!(crate::job::job_event_by_id(id), None);

        assert_eq!(job_still_running(), 0);
        crate::server::server_proc = prev_proc;
    }
}

#[test]
fn test_job_transfer_and_check_died() {
    let _guard = globals();
    ensure_reactor();
    let proc = crate::proc::ProcessRef::default();
    let prev_proc = unsafe { crate::server::server_proc.take() };
    unsafe { crate::server::server_proc = Some(proc) };

    unsafe {
        let j = job_run(
            Some(c"true"),
            &[],
            None,
            None,
            None,
            None,
            None,
            JOB_NOWAIT,
            80,
            24,
        );
        let id = j.expect("the job starts");
        assert!(crate::job::job_event_by_id(id).is_some());

        let mut tty = [0u8; 32];
        let (fd, pid) = job_transfer(id, Some(&mut tty)).expect("a registered job transfers");
        assert!(pid > 0);
        assert_eq!(crate::job::job_event_by_id(id), None);
        if fd >= 0 {
            libc::close(fd);
        }

        crate::server::server_proc = prev_proc;
    }
}
