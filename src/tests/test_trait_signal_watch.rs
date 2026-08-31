//! The [`SignalWatch`] contract, exercised only through the trait.
//!
//! A handle is instantiated by the concrete type and given its signal and
//! callback, which the trait has no method for. After that the only thing the
//! trait offers is [`SignalWatch::unwatch`], so what is asserted here is that
//! a watch stays on until it is taken off, and is off the moment it is.
//!
//! Signal dispositions are process-wide rather than per-thread, so every test
//! here takes a real-time signal of its own and none of them share one.

use ::std::rc::Rc;
use ::std::sync::atomic::{AtomicUsize, Ordering};

use crate::reactor::registry::SignalHandle;
use crate::reactor::{Reactor, SignalWatch, current};

/// How many turns of the loop a raised signal is given to arrive.
const TURNS: usize = 8;

/// How many turns are enough to show a signal does *not* arrive. A raised
/// signal is already pending by then, and each turn that finds nothing ready
/// costs a poll timeout the suite pays in wall clock.
const SETTLE: usize = 2;

/// A real-time signal nothing else in the process uses. Each test passes an
/// offset of its own so two running at once never share a disposition.
fn signal(offset: ::core::ffi::c_int) -> ::core::ffi::c_int {
    libc::SIGRTMIN() + offset
}

/// A watch on `signo` whose callback counts the times it runs, and the count.
fn counting(signo: ::core::ffi::c_int) -> (SignalHandle, Rc<AtomicUsize>) {
    let calls = Rc::new(AtomicUsize::new(0));
    let counted = Rc::clone(&calls);
    let mut watch = SignalHandle::ZERO;
    watch.set_callback(signo, move |_signo, _events| {
        counted.fetch_add(1, Ordering::SeqCst);
    });
    (watch, calls)
}

/// Raises `signo` on this thread, which is where the watch is.
fn raise(signo: ::core::ffi::c_int) {
    assert_eq!(unsafe { libc::raise(signo) }, 0, "raise");
}

/// Turns the loop until `calls` reaches `want`, or until the budget runs out.
fn turn_until(calls: &AtomicUsize, want: usize) {
    for _ in 0..TURNS {
        if calls.load(Ordering::SeqCst) >= want {
            return;
        }
        current().run_once();
    }
}

/// Turns the loop a fixed number of times, for asserting something does *not*
/// happen.
fn turn() {
    for _ in 0..SETTLE {
        current().run_once();
    }
}

/// `ZERO` is the all-zero state a handle embedded in `xcalloc`ed memory reads
/// as, and it is what `Default` answers with.
#[test]
fn the_zero_handle_is_the_default_one() {
    assert_eq!(<SignalHandle as SignalWatch>::ZERO, SignalHandle::default());
}

/// Unwatching a handle that was never given a signal is allowed and does
/// nothing.
#[test]
fn unwatching_a_handle_without_a_callback_does_nothing() {
    fn exercise(watch: &mut impl SignalWatch) {
        watch.unwatch();
        watch.unwatch();
    }

    let mut zero = <SignalHandle as SignalWatch>::ZERO;
    exercise(&mut zero);
    turn();
}

/// A watch stays on until it is taken off, so the same signal reaches it
/// every time it is raised.
#[test]
fn a_watch_stays_on_until_it_is_taken_off() {
    let signo = signal(8);
    let (mut watch, calls) = counting(signo);
    turn();

    raise(signo);
    turn_until(&calls, 1);
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    raise(signo);
    turn_until(&calls, 2);
    assert_eq!(calls.load(Ordering::SeqCst), 2);

    watch.unwatch();
}

/// Unwatching is immediate: a signal raised afterwards does not reach the
/// callback.
#[test]
fn unwatching_takes_the_watch_off_at_once() {
    let signo = signal(9);
    let (mut watch, calls) = counting(signo);
    turn();

    raise(signo);
    turn_until(&calls, 1);
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    fn take_off(watch: &mut impl SignalWatch) {
        watch.unwatch();
    }
    take_off(&mut watch);

    raise(signo);
    turn();
    assert_eq!(calls.load(Ordering::SeqCst), 1, "the watch is off");
}

/// Unwatching a watch that is already off is allowed and does nothing.
#[test]
fn unwatching_twice_does_nothing() {
    let signo = signal(10);
    let (mut watch, calls) = counting(signo);
    turn();

    fn take_off_twice(watch: &mut impl SignalWatch) {
        watch.unwatch();
        watch.unwatch();
    }
    take_off_twice(&mut watch);

    raise(signo);
    turn();
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

/// A handle is `Copy`, and a copy names the same watch: taking it off through
/// one takes it off for the other.
#[test]
fn a_copy_of_a_handle_names_the_same_watch() {
    let signo = signal(11);
    let (watch, calls) = counting(signo);
    let mut copy = watch;
    turn();

    raise(signo);
    turn_until(&calls, 1);
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    fn take_off(watch: &mut impl SignalWatch) {
        watch.unwatch();
    }
    take_off(&mut copy);

    raise(signo);
    turn();
    assert_eq!(calls.load(Ordering::SeqCst), 1, "off through the copy");
}
