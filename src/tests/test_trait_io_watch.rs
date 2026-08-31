//! The [`IoWatch`] contract, exercised only through the trait.
//!
//! A handle is instantiated by the concrete type and given its descriptor,
//! interest, mode and callback, which the trait has no method for. Everything
//! after that is [`IoWatch::enable`] and [`IoWatch::disable`], so what the
//! trait leaves observable is exactly whether the callback runs.

use ::std::io::Write as _;
use ::std::os::fd::AsRawFd as _;
use ::std::os::unix::net::UnixStream;
use ::std::rc::Rc;
use ::std::sync::atomic::{AtomicUsize, Ordering};

use crate::reactor::registry::IoHandle;
use crate::reactor::{Interest, IoWatch, Reactor, WatchMode, current};

/// How many turns of the loop a readiness is given to arrive before a test
/// gives up on it.
const TURNS: usize = 8;

/// How many turns are enough to show something does *not* happen. Readiness a
/// test has already written is queued by then, so this need not be the whole
/// budget above — and each turn that finds nothing ready costs a poll
/// timeout, which the suite pays in wall clock.
const SETTLE: usize = 2;

/// A connected pair, both ends non-blocking: the one a watch is put on, and
/// the one a test makes readable.
fn pair() -> (UnixStream, UnixStream) {
    let (source, peer) = UnixStream::pair().expect("socket pair");
    source.set_nonblocking(true).expect("nonblocking source");
    peer.set_nonblocking(true).expect("nonblocking peer");
    (source, peer)
}

/// A watch on `source` becoming readable whose callback drains one byte and
/// counts, together with the count.
fn counting(source: &UnixStream, mode: WatchMode) -> (IoHandle, Rc<AtomicUsize>) {
    let calls = Rc::new(AtomicUsize::new(0));
    let counted = Rc::clone(&calls);
    let mut watch = IoHandle::ZERO;
    watch.set_callback(source.as_raw_fd(), Interest::Read, mode, move |fd, _events| {
        let mut byte = 0u8;
        unsafe {
            libc::read(fd, (&raw mut byte).cast(), 1);
        }
        counted.fetch_add(1, Ordering::SeqCst);
    });
    (watch, calls)
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
    assert_eq!(<IoHandle as IoWatch>::ZERO, IoHandle::default());
}

/// Enabling and disabling a handle that has no descriptor and callback are
/// both allowed and do nothing.
#[test]
fn enabling_a_handle_without_a_callback_does_nothing() {
    fn exercise(watch: &mut impl IoWatch) {
        watch.enable();
        watch.disable();
        watch.enable();
    }

    let mut zero = <IoHandle as IoWatch>::ZERO;
    exercise(&mut zero);
    turn();
}

/// A watch runs its callback only once it is enabled: readiness that arrives
/// beforehand is picked up when it goes on the loop, not lost, but nothing
/// runs while it is off.
#[test]
fn a_watch_runs_nothing_until_it_is_enabled() {
    let (source, mut peer) = pair();
    let (mut watch, calls) = counting(&source, WatchMode::Persistent);

    peer.write_all(b"a").expect("write readiness");
    turn();
    assert_eq!(calls.load(Ordering::SeqCst), 0, "the watch is off the loop");

    fn enable(watch: &mut impl IoWatch) {
        watch.enable();
    }
    enable(&mut watch);
    turn_until(&calls, 1);
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    watch.disable();
}

/// A `Once` watch comes off the loop as its callback runs, so later readiness
/// does not reach it again.
#[test]
fn a_one_shot_watch_runs_once() {
    let (source, mut peer) = pair();
    let (mut watch, calls) = counting(&source, WatchMode::Once);

    fn enable(watch: &mut impl IoWatch) {
        watch.enable();
    }
    enable(&mut watch);

    peer.write_all(b"a").expect("first byte");
    turn_until(&calls, 1);
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    peer.write_all(b"b").expect("second byte");
    turn();
    assert_eq!(calls.load(Ordering::SeqCst), 1, "the watch is spent");
}

/// A `Persistent` watch stays on the loop, running again each time the
/// descriptor becomes readable.
#[test]
fn a_persistent_watch_runs_every_time() {
    let (source, mut peer) = pair();
    let (mut watch, calls) = counting(&source, WatchMode::Persistent);

    fn enable(watch: &mut impl IoWatch) {
        watch.enable();
    }
    enable(&mut watch);

    peer.write_all(b"a").expect("first byte");
    turn_until(&calls, 1);
    peer.write_all(b"b").expect("second byte");
    turn_until(&calls, 2);
    assert_eq!(calls.load(Ordering::SeqCst), 2);

    watch.disable();
}

/// Disabling takes a persistent watch off the loop, and readiness after that
/// does not reach it.
#[test]
fn disabling_stops_a_persistent_watch() {
    let (source, mut peer) = pair();
    let (mut watch, calls) = counting(&source, WatchMode::Persistent);

    fn exercise(watch: &mut impl IoWatch) {
        watch.enable();
    }
    exercise(&mut watch);

    peer.write_all(b"a").expect("first byte");
    turn_until(&calls, 1);
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    watch.disable();
    peer.write_all(b"b").expect("second byte");
    turn();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

/// Enabling an already-enabled watch and disabling an already-disabled one
/// both do nothing, which is what lets a caller reach for either without
/// asking first.
#[test]
fn enabling_twice_and_disabling_twice_do_nothing() {
    let (source, mut peer) = pair();
    let (mut watch, calls) = counting(&source, WatchMode::Persistent);

    fn enable_twice(watch: &mut impl IoWatch) {
        watch.enable();
        watch.enable();
    }
    enable_twice(&mut watch);

    peer.write_all(b"a").expect("one byte");
    turn_until(&calls, 1);
    assert_eq!(calls.load(Ordering::SeqCst), 1, "enabled once over");

    fn disable_twice(watch: &mut impl IoWatch) {
        watch.disable();
        watch.disable();
    }
    disable_twice(&mut watch);

    peer.write_all(b"b").expect("another byte");
    turn();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

/// A handle is `Copy`, and a copy names the same watch: disabling through one
/// takes the other off the loop too.
#[test]
fn a_copy_of_a_handle_names_the_same_watch() {
    let (source, mut peer) = pair();
    let (mut watch, calls) = counting(&source, WatchMode::Persistent);
    let mut copy = watch;

    fn enable(watch: &mut impl IoWatch) {
        watch.enable();
    }
    enable(&mut watch);

    peer.write_all(b"a").expect("first byte");
    turn_until(&calls, 1);
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    copy.disable();
    peer.write_all(b"b").expect("second byte");
    turn();
    assert_eq!(calls.load(Ordering::SeqCst), 1, "disabled through the copy");
}
