use super::*;
use crate::options::options_set_number;
use crate::status::{status_init, status_message_clear};
use crate::tests::test_fixtures::zeroed_term;
use crate::tests::test_fixtures::{
    Clients, Session, Window, ensure_reactor, globals, link, seen, unlink,
};
use ::core::ffi::c_longlong;

/// The `visual-*` choice that asks for a message and no terminal bell.
const VISUAL_ON: c_longlong = 1;

/// Whether the window's silence timer stands armed in the event base.
unsafe fn timer_armed(w: *mut window) -> bool {
    unsafe { (*w).alerts_timer.is_armed() }
}

/// Runs the deferred check the way the event loop would, which empties the
/// module's queue of everything any earlier test left in it.
fn drain() {
    unsafe { alerts_callback() };
}

/// A client on the server's list carrying a status line and a terminal
/// whose capability table is empty, so that the bell `alerts_set_message`
/// writes reaches a real `tty_term` and turns into nothing.
unsafe fn attached(list: &mut Clients, name: &str, s: *mut session) -> *mut client {
    unsafe {
        let c = list.add(name, 80, 24);
        status_init(c);
        (*c).session = s;
        (*c).tty.term = Some(zeroed_term());
        c
    }
}

/// The silence check declines a flagged window whose `monitor-silence` is
/// off, the same way the bell and activity checks decline theirs.
#[test]
fn the_silence_check_declines_a_flagged_window_nobody_watches() {
    let _guard = globals();
    let mut s = Session::new(30, "unwatched");
    let mut w = Window::new(50, "silent", 80, 24);
    let wl = link(&mut s, &mut w, 0);
    unsafe {
        (*w.ptr()).flags |= WINDOW_SILENCE;

        alerts_check_session(s.ptr());

        assert_eq!((*wl).flags, 0);
        assert!(!session_alerted(s.ptr()));
        assert_eq!((*w.ptr()).flags & WINDOW_SILENCE, WINDOW_SILENCE);

        unlink(&mut s, wl);
    }
}

/// The silence timer expiring asks for a silence check on the window it
/// was armed for, which re-arms it on the way past.
#[test]
fn the_silence_timer_queues_the_window_it_was_armed_for() {
    let _guard = globals();
    ensure_reactor();
    let mut w = Window::new(51, "expired", 80, 24);
    unsafe {
        drain();
        options_set_number(w.options(), c"monitor-silence".as_ptr(), 3);

        alerts_timer(&w.reference());

        assert_eq!((*w.ptr()).flags & WINDOW_SILENCE, WINDOW_SILENCE);
        assert_eq!((*w.ptr()).alerts_queued, 1);
        assert!(timer_armed(w.ptr()));

        drain();
        assert_eq!((*w.ptr()).alerts_queued, 0);
        (*w.ptr()).alerts_timer.disarm();
    }
}

/// The deferred check runs over the queue in arrival order, clears each
/// window's alert flags and unlinks it — and puts the latch back so the
/// next queueing arms a fresh check.
#[test]
fn the_deferred_check_releases_every_queued_window_and_resets_the_latch() {
    let _guard = globals();
    ensure_reactor();
    let mut first = Window::new(52, "first", 80, 24);
    let mut second = Window::new(53, "second", 80, 24);
    unsafe {
        drain();
        options_set_number(first.options(), c"monitor-activity".as_ptr(), 1);
        options_set_number(second.options(), c"monitor-activity".as_ptr(), 1);

        alerts_queue(first.ptr(), WINDOW_ACTIVITY);
        alerts_queue(second.ptr(), WINDOW_ACTIVITY);
        assert_eq!(queued_windows(), [first.ptr(), second.ptr()]);
        let fired = alerts_fired;
        assert_eq!(fired, 1);

        drain();

        for w in [first.ptr(), second.ptr()] {
            assert_eq!((*w).alerts_queued, 0);
            assert_eq!((*w).flags & WINDOW_ALERTFLAGS, 0);
        }
        assert!(queued_windows().is_empty());
        let fired = alerts_fired;
        assert_eq!(fired, 0);

        let mut third = Window::new(54, "third", 80, 24);
        options_set_number(third.options(), c"monitor-activity".as_ptr(), 1);
        alerts_queue(third.ptr(), WINDOW_ACTIVITY);
        assert_eq!(
            queued_windows(),
            [third.ptr()],
            "the emptied queue takes the next window on its own"
        );
        drain();
        assert!(queued_windows().is_empty());
    }
}

/// Only a client of the alerting session that is not a control client
/// hears anything, and with the visual option off what it hears is the
/// terminal bell alone.
#[test]
fn only_an_ordinary_client_of_the_session_hears_the_alert() {
    let _guard = globals();
    ensure_reactor();
    let mut s = Session::new(31, "belled");
    let mut other = Session::new(32, "elsewhere");
    let mut w = Window::new(55, "ringing", 80, 24);
    let wl = link(&mut s, &mut w, 0);
    let mut list = Clients::new();
    unsafe {
        let watcher = attached(&mut list, "watcher", s.ptr());
        let stranger = attached(&mut list, "stranger", other.ptr());
        let control = attached(&mut list, "control", s.ptr());
        (*control).flags |= CLIENT_CONTROL as uint64_t;
        (*w.ptr()).flags |= WINDOW_BELL;

        alerts_check_session(s.ptr());

        assert!(
            (*watcher).message_string.is_none(),
            "a bell alone leaves the status line alone"
        );
        assert!((*stranger).message_string.is_none());
        assert!((*control).message_string.is_none());
        assert_eq!((*wl).flags & WINLINK_BELL, WINLINK_BELL);

        unlink(&mut s, wl);
    }
}

/// A window linked into one session twice marks both of its winlinks, but
/// the session's mark stops the second one raising a message of its own.
#[test]
fn a_window_linked_twice_into_a_session_is_only_said_once() {
    let _guard = globals();
    ensure_reactor();
    let mut s = Session::new(34, "doubled");
    let mut w = Window::new(58, "twice", 80, 24);
    let first = link(&mut s, &mut w, 0);
    let second = link(&mut s, &mut w, 1);
    let mut list = Clients::new();
    unsafe {
        let c = attached(&mut list, "watcher", s.ptr());
        options_set_number(s.options(), c"visual-bell".as_ptr(), VISUAL_ON);
        (*w.ptr()).flags |= WINDOW_BELL;

        alerts_check_session(s.ptr());

        assert_eq!((*first).flags & WINLINK_BELL, WINLINK_BELL);
        assert_eq!((*second).flags & WINLINK_BELL, WINLINK_BELL);
        assert_eq!(
            seen(cstr_ptr(&(*c).message_string)),
            "Bell in current window",
            "the first winlink is the current one and the second says nothing"
        );
        status_message_clear(c);

        unlink(&mut s, first);
        unlink(&mut s, second);
    }
}

/// A visual alert names the window it came from, unless that is the
/// client's own current window; asking for both sends the bell as well.
#[test]
fn a_visual_alert_names_the_window_unless_the_client_is_looking_at_it() {
    let _guard = globals();
    ensure_reactor();
    let mut s = Session::new(33, "visual");
    let mut here = Window::new(56, "here", 80, 24);
    let mut there = Window::new(57, "there", 80, 24);
    let here_wl = link(&mut s, &mut here, 0);
    let there_wl = link(&mut s, &mut there, 7);
    let mut list = Clients::new();
    unsafe {
        let c = attached(&mut list, "watcher", s.ptr());
        options_set_number(s.options(), c"visual-bell".as_ptr(), VISUAL_ON);
        (*here.ptr()).flags |= WINDOW_BELL;

        alerts_check_session(s.ptr());
        assert_eq!(
            seen(cstr_ptr(&(*c).message_string)),
            "Bell in current window"
        );
        status_message_clear(c);

        (*here.ptr()).flags &= !WINDOW_BELL;
        (*there.ptr()).flags |= WINDOW_BELL;
        options_set_number(
            s.options(),
            c"visual-bell".as_ptr(),
            VISUAL_BOTH as c_longlong,
        );

        alerts_check_session(s.ptr());
        assert_eq!(seen(cstr_ptr(&(*c).message_string)), "Bell in window 7");
        status_message_clear(c);

        assert_eq!((*here_wl).flags & WINLINK_BELL, WINLINK_BELL);
        assert_eq!((*there_wl).flags & WINLINK_BELL, WINLINK_BELL);

        unlink(&mut s, here_wl);
        unlink(&mut s, there_wl);
    }
}
