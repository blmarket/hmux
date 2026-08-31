//! The [`Timer`] contract, exercised only through the trait.
//!
//! A handle is instantiated by the concrete type and given its callback, which
//! is the one thing the trait has no method for. Everything after that goes
//! through [`Timer`], so what is asserted here is the contract itself rather
//! than the registry that happens to be behind it.

use ::std::rc::Rc;
use ::std::sync::atomic::{AtomicUsize, Ordering};

use crate::reactor::registry::TimerHandle;
use crate::reactor::{Reactor, Timer, current};
use crate::types::timeval;

/// The times a callback has run.
type Calls = Rc<AtomicUsize>;

/// A handle carrying a callback that counts the times it runs, and the count.
fn counting() -> (TimerHandle, Calls) {
    let calls = Rc::new(AtomicUsize::new(0));
    let counted = Rc::clone(&calls);
    let mut timer = TimerHandle::ZERO;
    timer.set_callback(move || {
        counted.fetch_add(1, Ordering::SeqCst);
    });
    (timer, calls)
}

/// Turns the loop far enough for a deadline already due to be dispatched.
fn turn() {
    current().run_once();
}

/// `ZERO` is the all-zero state a handle embedded in `xcalloc`ed memory reads
/// as, and it is what `Default` answers with.
#[test]
fn the_zero_handle_is_the_default_one_and_carries_no_callback() {
    let zero = <TimerHandle as Timer>::ZERO;
    assert_eq!(zero, TimerHandle::default());
    assert!(!zero.is_set());
    assert!(!zero.is_armed());
}

/// Arming and disarming a handle that has no callback are both allowed and do
/// nothing, which is what lets the tree treat a zeroed slot as a timer.
#[test]
fn arming_a_handle_without_a_callback_does_nothing() {
    fn exercise(timer: &mut impl Timer) {
        timer.arm(timeval::from_secs(0));
        assert!(!timer.is_armed());
        timer.disarm();
        assert!(!timer.is_armed());
        assert!(!timer.is_set());
    }

    let mut zero = <TimerHandle as Timer>::ZERO;
    exercise(&mut zero);
    turn();
}

/// Giving a handle a callback is what `is_set` answers for, and it does not
/// arm anything by itself.
#[test]
fn a_handle_with_a_callback_is_set_but_not_armed() {
    let (timer, calls) = counting();
    fn exercise(timer: &impl Timer) {
        assert!(timer.is_set());
        assert!(!timer.is_armed());
    }

    exercise(&timer);
    turn();
    assert_eq!(calls.load(Ordering::SeqCst), 0, "nothing was armed");
}

/// A zero `after` means the next turn of the loop, the deadline is dropped
/// once it has been reached, and the same handle can be armed again.
#[test]
fn a_timer_fires_once_and_can_be_armed_again() {
    let (mut timer, calls) = counting();

    fn arm(timer: &mut impl Timer) {
        timer.arm(timeval::from_secs(0));
        assert!(timer.is_armed());
    }

    arm(&mut timer);
    turn();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(!timer.is_armed(), "the deadline is spent");

    arm(&mut timer);
    turn();
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

/// Taking the deadline off before it is reached keeps the callback from
/// running at all.
#[test]
fn disarming_before_the_deadline_keeps_the_callback_from_running() {
    let (mut timer, calls) = counting();

    fn exercise(timer: &mut impl Timer) {
        timer.arm(timeval::from_secs(60));
        assert!(timer.is_armed());
        timer.disarm();
        assert!(!timer.is_armed());
    }

    exercise(&mut timer);
    turn();
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

/// Disarming a timer that is not armed is allowed and does nothing, including
/// straight after the one deadline it had has been reached.
#[test]
fn disarming_an_unarmed_timer_does_nothing() {
    let (mut timer, calls) = counting();

    fn exercise(timer: &mut impl Timer) {
        timer.disarm();
        assert!(!timer.is_armed());
        timer.arm(timeval::from_secs(0));
    }

    exercise(&mut timer);
    turn();
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    timer.disarm();
    timer.disarm();
    assert!(!timer.is_armed());
    turn();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

/// A second callback retires the first: the handle names the new one, and the
/// callback it used to carry never runs again.
#[test]
fn setting_a_callback_again_retires_the_previous_one() {
    let (mut timer, first) = counting();
    let second = Rc::new(AtomicUsize::new(0));
    let counted = Rc::clone(&second);
    timer.set_callback(move || {
        counted.fetch_add(1, Ordering::SeqCst);
    });

    fn exercise(timer: &mut impl Timer) {
        assert!(timer.is_set());
        timer.arm(timeval::from_secs(0));
    }

    exercise(&mut timer);
    turn();
    assert_eq!(first.load(Ordering::SeqCst), 0, "the retired callback");
    assert_eq!(second.load(Ordering::SeqCst), 1, "the one that replaced it");
}

/// A handle is `Copy`, and a copy names the same timer: arming through one is
/// visible through the other.
#[test]
fn a_copy_of_a_handle_names_the_same_timer() {
    let (mut timer, calls) = counting();
    let mut copy = timer;

    fn arm(timer: &mut impl Timer) {
        timer.arm(timeval::from_secs(0));
    }

    arm(&mut copy);
    assert!(timer.is_armed(), "armed through the copy");

    timer.disarm();
    assert!(!copy.is_armed(), "disarmed through the original");

    turn();
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}
