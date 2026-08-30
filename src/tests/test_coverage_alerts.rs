//! Unit tests for the alert engine in [`crate::alerts`]: the queue that
//! batches windows waiting for an alert check ([`alerts_queue`]), the pass
//! over one session's winlinks ([`alerts_check_session`]) and the reset sweep
//! over every registered window ([`alerts_reset_all`]).
//!
//! The queue itself, the `fired` latch and the two event callbacks are
//! private to the module and driven from the event loop, so they stay
//! unexercised; so do the client-visible message and terminal-bell branches,
//! which want a live client behind the session. What is tested is the
//! observable state: the window and winlink alert flags, queue membership,
//! the session's alert mark, and whether the silence
//! timer stands armed — with the monitor options and the `-action` choices
//! deciding each outcome.
//!
//! A window left queued cannot be taken back out again from here, because
//! the only thing that drains runs on the event loop. Freeing such a fixture
//! would leave a dangling pointer on the private queue for every later test
//! that drains it, so tests that end with windows still queued give those
//! fixtures up without running their destructors.

use crate::alerts::{
    ALERT_ANY, ALERT_CURRENT, ALERT_OTHER, WINDOW_ACTIVITY, WINDOW_ALERTFLAGS, WINDOW_BELL,
    WINDOW_SILENCE, WINLINK_ACTIVITY, WINLINK_BELL, WINLINK_SILENCE, alerts_check_session,
    alerts_queue, alerts_reset_all, queued_windows,
};
use crate::options::options_set_number;
use crate::reactor::Timer;
use crate::session::{session_add_attached, session_alerted};
use crate::tests::test_fixtures::{
    Registry, Session, Window, ensure_reactor, globals, link, unlink,
};
use crate::types::*;
use ::core::ffi::c_longlong;

/// Whether the window's silence timer is pending in the event base.
unsafe fn timer_armed(w: *mut window) -> bool {
    unsafe { (*w).alerts_timer.is_armed() }
}

/// Takes a window's silence timer out of the event base, so the fixture can
/// be dropped without leaving a pending event pointing at freed memory.
unsafe fn disarm_timer(w: *mut window) {
    unsafe {
        (*w).alerts_timer.disarm();
    }
}

/// Gives up a fixture whose window is still linked in the module's private
/// queue, keeping every pointer the queue holds valid for the life of the
/// process. See the file header.
fn leave_queued(w: Window) {
    drop(w);
}

#[test]
fn queueing_a_monitored_family_sets_the_flags_and_links_the_window() {
    let _guard = globals();
    ensure_reactor();
    let mut w = Window::new(1, "bell", 80, 24);
    unsafe {
        assert_eq!((*w.ptr()).alerts_queued, 0);
        assert!(!queued_windows().contains(&w.ptr()));

        alerts_queue(w.ptr(), WINDOW_BELL);

        assert_eq!((*w.ptr()).flags & WINDOW_BELL, WINDOW_BELL);
        assert_eq!((*w.ptr()).alerts_queued, 1);
        assert_eq!(queued_windows().last().copied(), Some(w.ptr()));
        assert!(!timer_armed(w.ptr()));
    }
    leave_queued(w);
}

#[test]
fn queueing_a_family_nobody_monitors_sets_the_flags_but_never_queues() {
    let _guard = globals();
    ensure_reactor();
    let mut w = Window::new(2, "quiet", 80, 24);
    unsafe {
        options_set_number(w.options(), c"monitor-bell".as_ptr(), 0);

        alerts_queue(w.ptr(), WINDOW_BELL);
        assert_eq!((*w.ptr()).flags & WINDOW_BELL, WINDOW_BELL);
        assert_eq!((*w.ptr()).alerts_queued, 0);
        assert!(!queued_windows().contains(&w.ptr()));

        alerts_queue(w.ptr(), WINDOW_ACTIVITY);
        assert_eq!((*w.ptr()).flags & WINDOW_ACTIVITY, WINDOW_ACTIVITY);
        assert_eq!((*w.ptr()).alerts_queued, 0);

        alerts_queue(w.ptr(), WINDOW_SILENCE);
        assert_eq!((*w.ptr()).flags & WINDOW_SILENCE, WINDOW_SILENCE);
        assert_eq!((*w.ptr()).alerts_queued, 0);
        assert!(!queued_windows().contains(&w.ptr()));
    }
}

#[test]
fn queueing_a_mixed_request_queues_when_any_one_family_is_monitored() {
    let _guard = globals();
    ensure_reactor();
    let mut w = Window::new(3, "mixed", 80, 24);
    unsafe {
        options_set_number(w.options(), c"monitor-bell".as_ptr(), 0);
        options_set_number(w.options(), c"monitor-activity".as_ptr(), 1);

        alerts_queue(w.ptr(), WINDOW_BELL | WINDOW_ACTIVITY);

        assert_eq!(
            (*w.ptr()).flags & (WINDOW_BELL | WINDOW_ACTIVITY),
            WINDOW_BELL | WINDOW_ACTIVITY
        );
        assert_eq!((*w.ptr()).alerts_queued, 1);
        assert_eq!(queued_windows().last().copied(), Some(w.ptr()));
    }
    leave_queued(w);
}

#[test]
fn queueing_twice_leaves_one_entry_and_one_flag_set() {
    let _guard = globals();
    ensure_reactor();
    let mut w = Window::new(4, "twice", 80, 24);
    unsafe {
        options_set_number(w.options(), c"monitor-activity".as_ptr(), 1);
        alerts_queue(w.ptr(), WINDOW_ACTIVITY);
        let queued = queued_windows();
        alerts_queue(w.ptr(), WINDOW_ACTIVITY);

        assert_eq!((*w.ptr()).alerts_queued, 1);
        assert_eq!(queued_windows(), queued);
        assert_eq!((*w.ptr()).flags & WINDOW_ACTIVITY, WINDOW_ACTIVITY);
    }
    leave_queued(w);
}

#[test]
fn two_queued_windows_are_chained_in_arrival_order() {
    let _guard = globals();
    ensure_reactor();
    let mut first = Window::new(5, "first", 80, 24);
    let mut second = Window::new(6, "second", 80, 24);
    unsafe {
        options_set_number(first.options(), c"monitor-activity".as_ptr(), 1);
        options_set_number(second.options(), c"monitor-activity".as_ptr(), 1);

        alerts_queue(first.ptr(), WINDOW_ACTIVITY);
        alerts_queue(second.ptr(), WINDOW_ACTIVITY);

        let queued = queued_windows();
        assert_eq!(queued[queued.len() - 2..], [first.ptr(), second.ptr()]);
        assert_eq!((*first.ptr()).alerts_queued, 1);
        assert_eq!((*second.ptr()).alerts_queued, 1);
    }
    leave_queued(first);
    leave_queued(second);
}

#[test]
fn queueing_resets_the_silence_state_before_rearming_what_is_still_monitored() {
    let _guard = globals();
    ensure_reactor();
    let mut w = Window::new(7, "silence", 80, 24);
    unsafe {
        options_set_number(w.options(), c"monitor-silence".as_ptr(), 2);

        alerts_queue(w.ptr(), WINDOW_SILENCE);
        assert!(timer_armed(w.ptr()));
        assert_eq!((*w.ptr()).flags & WINDOW_SILENCE, WINDOW_SILENCE);
        assert_eq!((*w.ptr()).alerts_queued, 1);

        options_set_number(w.options(), c"monitor-silence".as_ptr(), 0);
        options_set_number(w.options(), c"monitor-bell".as_ptr(), 1);
        alerts_queue(w.ptr(), WINDOW_BELL);
        assert!(!timer_armed(w.ptr()));
        assert_eq!(
            (*w.ptr()).flags & WINDOW_ALERTFLAGS,
            WINDOW_BELL,
            "the silence flag is reset away and only the queued family comes back"
        );
        assert_eq!((*w.ptr()).alerts_queued, 1);
    }
    leave_queued(w);
}

#[test]
fn queueing_no_flags_at_all_still_runs_the_reset() {
    let _guard = globals();
    ensure_reactor();
    let mut w = Window::new(8, "noflags", 80, 24);
    unsafe {
        (*w.ptr()).flags |= WINDOW_SILENCE;

        alerts_queue(w.ptr(), 0);

        assert_eq!((*w.ptr()).flags & WINDOW_ALERTFLAGS, 0);
        assert_eq!((*w.ptr()).alerts_queued, 0);
        assert!(!timer_armed(w.ptr()));
    }
}

#[test]
fn alerts_reset_all_clears_the_silence_flag_of_every_registered_window_only() {
    let _guard = globals();
    ensure_reactor();

    {
        alerts_reset_all();
    }

    let mut registry = Registry::new();
    let mut w1 = Window::new(9, "one", 80, 24);
    let mut w2 = Window::new(10, "two", 80, 24);
    let mut stray = Window::new(11, "stray", 80, 24);
    registry.add_window(&mut w1);
    registry.add_window(&mut w2);
    unsafe {
        (*w1.ptr()).flags |= WINDOW_SILENCE;
        (*w2.ptr()).flags |= WINDOW_SILENCE;
        (*stray.ptr()).flags |= WINDOW_SILENCE;
        assert!(!(*w1.ptr()).alerts_timer.is_set());

        alerts_reset_all();

        assert_eq!((*w1.ptr()).flags & WINDOW_SILENCE, 0);
        assert_eq!((*w2.ptr()).flags & WINDOW_SILENCE, 0);
        assert_eq!((*stray.ptr()).flags & WINDOW_SILENCE, WINDOW_SILENCE);
        assert!((*w1.ptr()).alerts_timer.is_set());
        assert!((*w2.ptr()).alerts_timer.is_set());
        assert!(!(*stray.ptr()).alerts_timer.is_set());
        assert!(!timer_armed(w1.ptr()));
        assert!(!timer_armed(w2.ptr()));
    }
}

#[test]
fn alerts_reset_all_arms_the_timer_exactly_where_silence_is_monitored() {
    let _guard = globals();
    ensure_reactor();
    let mut registry = Registry::new();
    let mut watched = Window::new(12, "watched", 80, 24);
    let mut ignored = Window::new(13, "ignored", 80, 24);
    registry.add_window(&mut watched);
    registry.add_window(&mut ignored);
    unsafe {
        (*watched.ptr()).flags |= WINDOW_SILENCE;
        (*ignored.ptr()).flags |= WINDOW_SILENCE;
        options_set_number(watched.options(), c"monitor-silence".as_ptr(), 5);

        alerts_reset_all();
        assert!(timer_armed(watched.ptr()));
        assert!(!timer_armed(ignored.ptr()));

        options_set_number(watched.options(), c"monitor-silence".as_ptr(), 0);
        options_set_number(ignored.options(), c"monitor-silence".as_ptr(), 9);
        alerts_reset_all();
        assert!(!timer_armed(watched.ptr()));
        assert!(timer_armed(ignored.ptr()));

        disarm_timer(watched.ptr());
        disarm_timer(ignored.ptr());
        assert!(!timer_armed(watched.ptr()));
        assert!(!timer_armed(ignored.ptr()));
    }
}

#[test]
fn alerts_check_session_checks_every_monitored_family_of_every_winlink() {
    let _guard = globals();
    let mut s = Session::new(1, "checked");
    let mut loud = Window::new(14, "loud", 80, 24);
    let mut quiet = Window::new(15, "quiet", 80, 24);
    let loud_wl = link(&mut s, &mut loud, 0);
    let quiet_wl = link(&mut s, &mut quiet, 1);
    unsafe {
        options_set_number(loud.options(), c"monitor-activity".as_ptr(), 1);
        options_set_number(loud.options(), c"monitor-silence".as_ptr(), 1);
        options_set_number(
            s.options(),
            c"activity-action".as_ptr(),
            ALERT_ANY as c_longlong,
        );
        options_set_number(
            s.options(),
            c"silence-action".as_ptr(),
            ALERT_ANY as c_longlong,
        );
        (*loud.ptr()).flags |= WINDOW_BELL | WINDOW_ACTIVITY | WINDOW_SILENCE;

        alerts_check_session(s.ptr());

        assert_eq!(
            (*loud_wl).flags,
            WINLINK_BELL | WINLINK_ACTIVITY | WINLINK_SILENCE
        );
        assert_eq!((*quiet_wl).flags, 0);
        assert!(session_alerted(s.ptr()));
        assert_eq!(
            (*loud.ptr()).flags & WINDOW_ALERTFLAGS,
            WINDOW_BELL | WINDOW_ACTIVITY | WINDOW_SILENCE,
            "checking never clears the window's own flags"
        );
        assert_eq!((*loud.ptr()).alerts_queued, 0);

        unlink(&mut s, loud_wl);
        unlink(&mut s, quiet_wl);
    }
}

#[test]
fn alerts_check_session_leaves_unmonitored_or_unflagged_families_alone() {
    let _guard = globals();
    let mut s = Session::new(2, "partial");
    let mut w = Window::new(16, "half", 80, 24);
    let wl = link(&mut s, &mut w, 0);
    unsafe {
        (*w.ptr()).flags |= WINDOW_ACTIVITY;

        alerts_check_session(s.ptr());
        assert_eq!((*wl).flags, 0);
        assert!(!session_alerted(s.ptr()));

        options_set_number(w.options(), c"monitor-activity".as_ptr(), 1);
        options_set_number(
            s.options(),
            c"activity-action".as_ptr(),
            ALERT_ANY as c_longlong,
        );
        alerts_check_session(s.ptr());
        assert_eq!((*wl).flags, WINLINK_ACTIVITY);
        assert!(session_alerted(s.ptr()));

        unlink(&mut s, wl);
    }
}

#[test]
fn activity_and_silence_notify_once_while_bell_notifies_on_every_pass() {
    let _guard = globals();
    let mut s = Session::new(3, "guarded");
    let mut w = Window::new(17, "guarded-win", 80, 24);
    let wl = link(&mut s, &mut w, 0);
    unsafe {
        options_set_number(w.options(), c"monitor-activity".as_ptr(), 1);
        options_set_number(w.options(), c"monitor-silence".as_ptr(), 1);
        options_set_number(
            s.options(),
            c"activity-action".as_ptr(),
            ALERT_ANY as c_longlong,
        );
        options_set_number(
            s.options(),
            c"silence-action".as_ptr(),
            ALERT_ANY as c_longlong,
        );
        (*w.ptr()).flags |= WINDOW_ACTIVITY | WINDOW_SILENCE;

        alerts_check_session(s.ptr());
        assert_eq!((*wl).flags, WINLINK_ACTIVITY | WINLINK_SILENCE);
        assert!(session_alerted(s.ptr()));

        alerts_check_session(s.ptr());
        assert_eq!((*wl).flags, WINLINK_ACTIVITY | WINLINK_SILENCE);
        assert!(
            !session_alerted(s.ptr()),
            "the once-only families clear the mark and then decline to raise it again"
        );

        (*w.ptr()).flags |= WINDOW_BELL;
        alerts_check_session(s.ptr());
        assert_eq!(
            (*wl).flags,
            WINLINK_BELL | WINLINK_ACTIVITY | WINLINK_SILENCE
        );
        assert!(
            !session_alerted(s.ptr()),
            "bell runs first and raises the mark, but the skipped guarded families clear it again on their way past"
        );

        (*w.ptr()).flags &= !(WINDOW_ACTIVITY | WINDOW_SILENCE);
        alerts_check_session(s.ptr());
        assert!(
            session_alerted(s.ptr()),
            "with nothing checked after it, bell's own raise survives"
        );

        alerts_check_session(s.ptr());
        assert!(
            session_alerted(s.ptr()),
            "bell carries no once-only mark and notifies again"
        );

        unlink(&mut s, wl);
    }
}

#[test]
fn the_bell_action_decides_whose_notification_outlives_the_pass() {
    let _guard = globals();
    let mut s = Session::new(4, "actions");
    let mut first = Window::new(18, "first", 80, 24);
    let mut second = Window::new(19, "second", 80, 24);
    let first_wl = link(&mut s, &mut first, 0);
    let second_wl = link(&mut s, &mut second, 1);
    unsafe {
        (*first.ptr()).flags |= WINDOW_BELL;
        (*second.ptr()).flags |= WINDOW_BELL;

        alerts_check_session(s.ptr());
        assert_eq!((*first_wl).flags, WINLINK_BELL);
        assert_eq!((*second_wl).flags, WINLINK_BELL);
        assert!(session_alerted(s.ptr()));

        options_set_number(
            s.options(),
            c"bell-action".as_ptr(),
            ALERT_CURRENT as c_longlong,
        );
        alerts_check_session(s.ptr());
        assert!(
            !session_alerted(s.ptr()),
            "the second window's check clears the mark the first raised"
        );

        options_set_number(
            s.options(),
            c"bell-action".as_ptr(),
            ALERT_OTHER as c_longlong,
        );
        alerts_check_session(s.ptr());
        assert!(session_alerted(s.ptr()));

        options_set_number(s.options(), c"bell-action".as_ptr(), 0);
        alerts_check_session(s.ptr());
        assert!(!session_alerted(s.ptr()));

        unlink(&mut s, first_wl);
        unlink(&mut s, second_wl);
    }
}

#[test]
fn a_shared_window_is_checked_per_session_with_that_sessions_own_action() {
    let _guard = globals();
    let mut first = Session::new(5, "current");
    let mut second = Session::new(6, "other");
    let mut w = Window::new(20, "shared", 80, 24);
    let first_wl = link(&mut first, &mut w, 0);
    let second_wl = link(&mut second, &mut w, 5);
    unsafe {
        session_add_attached(first.ptr());
        session_add_attached(second.ptr());
        options_set_number(
            first.options(),
            c"bell-action".as_ptr(),
            ALERT_CURRENT as c_longlong,
        );
        options_set_number(
            second.options(),
            c"bell-action".as_ptr(),
            ALERT_OTHER as c_longlong,
        );
        (*w.ptr()).flags |= WINDOW_BELL;

        alerts_check_session(first.ptr());
        assert_eq!(
            (*first_wl).flags & WINLINK_BELL,
            0,
            "an attached session's current winlink is never marked"
        );
        assert_eq!((*second_wl).flags & WINLINK_BELL, 0);
        assert!(session_alerted(first.ptr()));
        assert!(!session_alerted(second.ptr()));

        options_set_number(
            second.options(),
            c"bell-action".as_ptr(),
            ALERT_ANY as c_longlong,
        );
        alerts_check_session(first.ptr());
        assert_eq!((*second_wl).flags & WINLINK_BELL, 0);
        assert!(session_alerted(second.ptr()));
        assert!(session_alerted(first.ptr()));

        unlink(&mut first, first_wl);
        unlink(&mut second, second_wl);
    }
}

#[test]
fn alerts_check_session_on_a_session_without_windows_does_nothing() {
    let _guard = globals();
    let mut s = Session::new(7, "empty");
    unsafe {
        alerts_check_session(s.ptr());
        assert!(!session_alerted(s.ptr()));
    }
}

#[test]
fn every_alert_choice_reads_back_through_the_option_it_is_set_through() {
    let _guard = globals();
    let mut s = Session::new(8, "choices");
    unsafe {
        for (name, value) in [
            (c"bell-action".as_ptr(), ALERT_ANY as c_longlong),
            (c"bell-action".as_ptr(), ALERT_CURRENT as c_longlong),
            (c"bell-action".as_ptr(), ALERT_OTHER as c_longlong),
            (c"activity-action".as_ptr(), ALERT_CURRENT as c_longlong),
            (c"silence-action".as_ptr(), ALERT_OTHER as c_longlong),
        ] {
            options_set_number(s.options(), name, value);
            assert_eq!(crate::options::options_get_number(s.options(), name), value);
        }
    }
}
