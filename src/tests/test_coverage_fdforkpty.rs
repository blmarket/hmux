//! Unit tests for [`crate::compat`].
//!
//! This compat module carries the Linux translation of OpenBSD's
//! pseudo-terminal-master plumbing: [`getptmfd`](getptmfd)
//! stands in for the system call that hands back a descriptor for the
//! ptm device, and `fdforkpty`
//! ([`fdforkpty`](crate::compat::fdforkpty)) wraps libc's
//! `forkpty(3)` behind an extra leading `_ptmfd` argument the Linux build has
//! no use for. Alongside them live the C limits `INT_MAX` and `__INT_MAX__`.
//!
//! Everything here is safe and deterministic to exercise except one branch:
//! calling `fdforkpty` itself would drive the real `forkpty(3)` and split the
//! process into parent and child. No unit test may take that step, so the
//! wrapper is pinned instead at compile time by a function-pointer coercion.
//!
//! `getptmfd` reads no state and touches none of the process-wide statics the
//! server keeps, so — like the pure-arithmetic suites — these tests hold no
//! turn at the [`crate::tests::test_fixtures::globals`] mutex.

use crate::compat::{__INT_MAX__, FdForkptyResult, INT_MAX, fdforkpty, getptmfd};
use crate::types::*;
use ::core::ffi::c_int;

type ForkptyWrapper = unsafe fn(c_int, Option<&termios>, Option<&winsize>) -> FdForkptyResult;

#[test]
fn int_max_constants_hold_the_c_limit() {
    assert_eq!(INT_MAX, 2147483647);
    assert_eq!(__INT_MAX__, 2147483647);
    assert_eq!(INT_MAX, __INT_MAX__);
    assert_eq!(INT_MAX, c_int::MAX);
    assert_eq!(INT_MAX.checked_add(1), None);
}

#[test]
fn getptmfd_answers_the_stub_descriptor() {
    {
        assert_eq!(getptmfd(), INT_MAX);
        assert_eq!(getptmfd(), c_int::MAX);
        assert_ne!(getptmfd(), -(1 as c_int));
    }
}

#[test]
fn getptmfd_is_deterministic_across_calls() {
    {
        let first = getptmfd();
        for _ in 0..16 {
            assert_eq!(getptmfd(), first);
        }
    }
}

#[test]
fn fdforkpty_matches_the_forkpty_wrapper_shape() {
    let _wrapper_must_compile: ForkptyWrapper = fdforkpty;
}
