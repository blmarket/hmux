//! Unit tests for [`crate::job`], the job wrapper around `fork` / `fdforkpty`.
//!
//! The live `job_run` path forks, touches descriptors and chdirs the process
//! and is not exercised here. What is covered is the metadata the module
//! carries and the branches that are answered from memory alone: the job
//! state ordering, the flag bits, the three pure accessors built over a
//! zeroed `job`, and the global-list helpers when no job is registered
//! (`job_still_running`, `job_kill_all`, `job_check_died`).

use crate::job::{
    JOB_CLOSED, JOB_DEAD, JOB_DEFAULTSHELL, JOB_KEEPWRITE, JOB_NOWAIT, JOB_PTY, JOB_RUNNING,
    JOB_SHOWSTDERR, job_check_died, job_free, job_get_data, job_get_event, job_get_status,
    job_kill_all, job_print_summary, job_resize, job_run, job_still_running, job_transfer,
};
use crate::tests::test_fixtures::{Item, ensure_reactor, globals, zeroed};
use crate::types::{JobData, job, tmuxproc};
use ::core::ptr::{null, null_mut};

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

/// `job_get_status` returns whatever `status` the job was given, without
/// touching any other field.
#[test]
fn job_get_status_returns_the_status_field() {
    let mut j = zeroed::<job>();
    j.status = 42;
    let got = unsafe { job_get_status(&raw mut *j) };
    assert_eq!(got, 42);
    j.status = -7;
    let got = unsafe { job_get_status(&raw mut *j) };
    assert_eq!(got, -7);
}

/// `job_get_data` returns the exact `data` pointer the job carries.
#[test]
fn job_get_data_returns_the_data_pointer() {
    let mut j = zeroed::<job>();
    let mut sentinel: u32 = 0xdeadbeef;
    j.data = JobData::Format((&raw mut sentinel).cast());
    let got = unsafe { job_get_data(&raw mut *j) };
    assert!(matches!(
        got,
        JobData::Format(data) if *data == (&raw mut sentinel).cast()
    ));
    // null when nothing was stored
    j.data = JobData::None;
    let got = unsafe { job_get_data(&raw mut *j) };
    assert!(matches!(got, JobData::None));
}

/// `job_get_event` returns the exact `Stream` the job was given. A zeroed
/// job has `Stream::NONE`; a fabricated non-null value round-trips unchanged.
#[test]
fn job_get_event_returns_the_event_field() {
    let mut j = zeroed::<job>();
    // zeroed memory is Stream::NONE (null ctx)
    let ev = unsafe { job_get_event(&raw mut *j) };
    assert!(ev.is_none(), "zeroed job should carry Stream::NONE");
    // store a fake non-null pointer and check it comes back
    let fake = 0x1234 as *mut ::core::ffi::c_void;
    // Stream is transparent over *mut StreamCtx; transmute via raw pointer
    let fake_stream: crate::reactor::Stream = unsafe { ::core::mem::transmute(fake) };
    j.event = fake_stream;
    let ev = unsafe { job_get_event(&raw mut *j) };
    let ev_ptr: *mut ::core::ffi::c_void = unsafe { ::core::mem::transmute(ev) };
    assert_eq!(ev_ptr, fake);
}

/// `job_get_status` and `job_get_data` are orthogonal: writing one does not
/// disturb the other.
#[test]
fn status_and_data_are_independent() {
    let mut j = zeroed::<job>();
    j.status = 99;
    let mut x: i32 = 5;
    j.data = JobData::Format((&raw mut x).cast());
    assert_eq!(unsafe { job_get_status(&raw mut *j) }, 99);
    assert!(matches!(
        unsafe { job_get_data(&raw mut *j) },
        JobData::Format(data) if *data == (&raw mut x).cast()
    ));
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
    let stopped_sigttin: ::core::ffi::c_int = (21 << 8) | 0x7f; // SIGTTIN = 21
    let stopped_sigttou: ::core::ffi::c_int = (22 << 8) | 0x7f; // SIGTTOU = 22
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
    let mut proc = Box::new(tmuxproc::default());
    let prev_proc = unsafe { crate::server::server_proc };
    unsafe { crate::server::server_proc = &raw mut *proc };

    unsafe {
        let j = job_run(
            c"true".as_ptr(),
            &[],
            null_mut(),
            null_mut(),
            null(),
            None,
            None,
            None,
            JobData::None,
            0,
            80,
            24,
        );
        assert!(!j.is_null());
        assert_eq!(job_still_running(), 1);

        let mut item = Item::new();
        job_print_summary(item.ptr(), 1);

        job_resize(j, 100, 30);
        job_kill_all();
        job_free(j);

        assert_eq!(job_still_running(), 0);
        crate::server::server_proc = prev_proc;
    }
}

#[test]
fn test_job_transfer_and_check_died() {
    let _guard = globals();
    ensure_reactor();
    let mut proc = Box::new(tmuxproc::default());
    let prev_proc = unsafe { crate::server::server_proc };
    unsafe { crate::server::server_proc = &raw mut *proc };

    unsafe {
        let j = job_run(
            c"true".as_ptr(),
            &[],
            null_mut(),
            null_mut(),
            null(),
            None,
            None,
            None,
            JobData::None,
            JOB_NOWAIT,
            80,
            24,
        );
        assert!(!j.is_null());

        let mut tty = [0 as ::core::ffi::c_char; 32];
        let pid_val = (*j).pid;
        let (fd, pid) = job_transfer(j, tty.as_mut_ptr(), 32);
        assert_eq!(pid, pid_val);
        if fd >= 0 {
            ::libc::close(fd);
        }

        crate::server::server_proc = prev_proc;
    }
}
