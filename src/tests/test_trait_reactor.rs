//! The [`Reactor`] contract, exercised only through the trait.
//!
//! The loop is instantiated by [`current`], which is the one thing the trait
//! has no method for. Everything after that goes through [`Reactor`], with
//! [`Timer`] standing in for the work a turn is meant to dispatch.

use ::std::rc::Rc;
use ::std::sync::atomic::{AtomicUsize, Ordering};

use crate::reactor::registry::TimerHandle;
use crate::reactor::{Reactor, Timer, current};
use crate::types::timeval;

/// A timer carrying a callback that counts the times it runs, and the count.
fn counting_timer() -> (TimerHandle, Rc<AtomicUsize>) {
    let calls = Rc::new(AtomicUsize::new(0));
    let counted = Rc::clone(&calls);
    let mut timer = TimerHandle::ZERO;
    timer.set_callback(move || {
        counted.fetch_add(1, Ordering::SeqCst);
    });
    (timer, calls)
}

/// The loop says what it is, and says the same thing every time.
#[test]
fn the_loop_describes_itself() {
    fn describe(reactor: &impl Reactor) -> String {
        reactor.describe()
    }

    let first = describe(&current());
    assert!(!first.is_empty());
    assert_eq!(first, describe(&current()), "the same answer every time");
}

/// A deferred call does not run where it is asked for. It runs on a later
/// turn, and only once.
#[test]
fn a_deferred_call_waits_for_a_turn_and_then_runs_once() {
    let calls = Rc::new(AtomicUsize::new(0));

    fn defer(reactor: &mut impl Reactor, calls: &Rc<AtomicUsize>) {
        let counted = Rc::clone(calls);
        reactor.defer(move || {
            counted.fetch_add(1, Ordering::SeqCst);
        });
    }

    let mut reactor = current();
    defer(&mut reactor, &calls);
    assert_eq!(calls.load(Ordering::SeqCst), 0, "not where it was asked for");

    for _ in 0..4 {
        reactor.run_once();
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1, "run, and run only once");
}

/// Every deferred call is kept, and each runs once.
#[test]
fn every_deferred_call_runs() {
    let calls = Rc::new(AtomicUsize::new(0));
    let mut reactor = current();

    for _ in 0..8 {
        let counted = Rc::clone(&calls);
        reactor.defer(move || {
            counted.fetch_add(1, Ordering::SeqCst);
        });
    }

    for _ in 0..4 {
        reactor.run_once();
    }
    assert_eq!(calls.load(Ordering::SeqCst), 8);
}

/// A call deferred from inside a deferred call is held for a later turn
/// rather than running within this one.
#[test]
fn a_call_deferred_from_a_deferred_call_waits_its_turn() {
    let order = Rc::new(::std::cell::RefCell::new(Vec::new()));
    let mut reactor = current();

    let outer = Rc::clone(&order);
    reactor.defer(move || {
        outer.borrow_mut().push("outer");
        let inner = Rc::clone(&outer);
        current().defer(move || {
            inner.borrow_mut().push("inner");
        });
    });

    for _ in 0..4 {
        reactor.run_once();
    }
    assert_eq!(*order.borrow(), vec!["outer", "inner"]);
}

/// Turning the loop dispatches a deadline that is due.
#[test]
fn a_turn_dispatches_what_is_ready() {
    let (mut timer, calls) = counting_timer();
    timer.arm(timeval::from_secs(0));

    fn turn(reactor: &mut impl Reactor) {
        reactor.run_once();
    }

    turn(&mut current());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

/// Turning a loop with nothing to do is allowed and dispatches nothing.
#[test]
fn a_turn_with_nothing_ready_dispatches_nothing() {
    let (timer, calls) = counting_timer();
    assert!(!timer.is_armed());

    let mut reactor = current();
    for _ in 0..4 {
        reactor.run_once();
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

/// `reinit` is not "drop everything": what was armed before is armed after,
/// and still reaches its callback.
#[test]
fn reinit_keeps_what_was_already_armed() {
    let (mut timer, calls) = counting_timer();
    timer.arm(timeval::from_secs(0));

    fn reinit(reactor: &mut impl Reactor) -> bool {
        reactor.reinit()
    }

    let mut reactor = current();
    assert!(reinit(&mut reactor), "the loop was rebuilt");
    assert!(timer.is_armed(), "still armed across the rebuild");

    for _ in 0..4 {
        reactor.run_once();
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

/// A deferred call outlives a rebuild too.
#[test]
fn reinit_keeps_a_deferred_call() {
    let calls = Rc::new(AtomicUsize::new(0));
    let mut reactor = current();
    let counted = Rc::clone(&calls);
    reactor.defer(move || {
        counted.fetch_add(1, Ordering::SeqCst);
    });

    assert!(reactor.reinit());
    for _ in 0..4 {
        reactor.run_once();
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

/// The loop carries no state of its own, so a second handle on it is the same
/// loop: work asked of one is dispatched by the other.
#[test]
fn every_handle_names_the_same_loop() {
    let calls = Rc::new(AtomicUsize::new(0));
    let counted = Rc::clone(&calls);
    current().defer(move || {
        counted.fetch_add(1, Ordering::SeqCst);
    });

    let mut other = current();
    for _ in 0..4 {
        other.run_once();
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
