//! The alert engine: how a bell, a burst of activity or a stretch of silence
//! in a window becomes a marked winlink, a hook and a message on the status
//! line of every client watching it.
//!
//! Nothing here checks anything the moment it is told. [`WindowRef::raise_alerts`] only
//! records the family on the window, puts the window on a queue and — the
//! first time round — asks ensure_reactor for one deferred callback; that callback
//! is what runs the checks, empties the queue and puts the latch down again,
//! so a window that rings a hundred times before the event loop next turns is
//! checked once. The queue owns a strong handle for each window for as long as
//! it is on the queue, and the window itself stays discoverable through the
//! server's weak registry.
//!
//! The three families differ only in which flags they use, which options
//! decide whether anyone is watching and what the message says, so they are
//! one [`Family`] table and one check. Bell is the one that reads differently:
//! it has no "already alerted" guard, because a bell is allowed even where a
//! bell is already pending.
//!
//! Silence is the family with a timer. [`alerts_reset`] arms the window's
//! `alerts_timer` for `monitor-silence` seconds on every queueing and on every
//! option change, and the timer expiring queues a silence check.
//!
//! Quirks kept. Every check takes the alerted mark off each session showing
//! the window before it starts, so of the three families that run for one
//! window the last one to look decides whether the session ends up marked —
//! a family that finds nothing to do still clears the mark the family before
//! it raised. `alerts_reset` runs before the flags are added, so queueing any
//! family drops a silence flag that was already standing, and queueing no
//! family at all still resets the silence state and re-arms the timer.
//!
//! Coverage exemptions: none.
use crate::fmt_args;
use crate::log::log_debug;
use crate::notify::notify_winlink;

use crate::reactor;
use crate::reactor::Reactor;
use crate::server::client_walk;
use crate::server::server_status_session;

use crate::status::status_message_set;
use crate::tree::GlobalQueue;
use crate::tty::tty_putcode;
pub use crate::types::*;
use crate::window::WINDOWS;
use ::core::ffi::{CStr, c_int};

pub use crate::consts::{
    ALERT_ANY, ALERT_OTHER, CLIENT_CONTROL, EV_TIMEOUT, RB_NEGINF, TTYC_BEL, VISUAL_OFF,
    WINDOW_ACTIVITY, WINDOW_ALERTFLAGS, WINDOW_BELL, WINDOW_SILENCE, WINLINK_ACTIVITY,
    WINLINK_BELL, WINLINK_SILENCE,
};

pub const ALERT_CURRENT: c_int = 2;

pub const VISUAL_BOTH: c_int = 2;

/// Whether a deferred check is already asked for, so that a burst of alerts
/// asks ensure_reactor for one callback and not one each.
static mut alerts_fired: c_int = 0;

/// The windows waiting for that check, in the order they were queued. Strong
/// handles keep them alive until the deferred callback drains the queue.
static alerts_list: GlobalQueue<WindowRef> = GlobalQueue::new();

/// One alert family: the window flag that records it and the winlink flag that
/// marks a session's copy of the window, the options that say whether anyone
/// is watching and what an alert applies to, and the names the hook and the
/// message carry.
struct Family {
    window_flag: c_int,
    winlink_flag: c_int,
    monitor: &'static CStr,
    action: &'static CStr,
    hook: &'static CStr,
    label: &'static CStr,
    visual: &'static CStr,
    /// Whether the family alerts again where its winlink flag already stands.
    /// Only bell does.
    again: bool,
}

static BELL: Family = Family {
    window_flag: WINDOW_BELL,
    winlink_flag: WINLINK_BELL,
    monitor: c"monitor-bell",
    action: c"bell-action",
    hook: c"alert-bell",
    label: c"Bell",
    visual: c"visual-bell",
    again: true,
};

static ACTIVITY: Family = Family {
    window_flag: WINDOW_ACTIVITY,
    winlink_flag: WINLINK_ACTIVITY,
    monitor: c"monitor-activity",
    action: c"activity-action",
    hook: c"alert-activity",
    label: c"Activity",
    visual: c"visual-activity",
    again: false,
};

static SILENCE: Family = Family {
    window_flag: WINDOW_SILENCE,
    winlink_flag: WINLINK_SILENCE,
    monitor: c"monitor-silence",
    action: c"silence-action",
    hook: c"alert-silence",
    label: c"Silence",
    visual: c"visual-silence",
    again: false,
};

/// The winlinks that show `w`, in the order the window's own list holds them.
fn showing(w: &WindowRef) -> impl Iterator<Item = crate::window::WinlinkRef> {
    w.winlinks()
}

/// The deferred check ensure_reactor runs once per batch: every queued window is
/// checked, unlinked and cleared of its alert flags, and the latch goes down so
/// the next alert asks for a fresh callback. Dropping each queued handle at the
/// end of the callback releases the queue's ownership.
unsafe fn alerts_callback() {
    unsafe {
        let mut queued = core::mem::take(&mut *alerts_list.queue());
        while let Some(w_ref) = queued.pop_front() {
            let alerts = w_ref.check_all_alerts();
            log_debug(
                c"@%u alerts check, alerts %#x",
                fmt_args![w_ref.window_id(), alerts],
            );
            w_ref.finish_alerts();
        }
        alerts_fired = 0;
    }
}

/// Whether `wl` is one the session's `{bell,activity,silence}-action` asks to
/// be told about: none means nothing happens, current means only the current
/// window and other means only windows that are not it.
unsafe fn alerts_action_applies(wl: &winlink, name: &CStr) -> bool {
    unsafe {
        let session = wl.session().expect("a link has a session");
        let action = session.options().number(name) as c_int;
        let current = session
            .curw()
            .is_some_and(|current| current.index() == wl.idx);
        match action {
            ALERT_ANY => true,
            ALERT_CURRENT => current,
            ALERT_OTHER => !current,
            _ => false,
        }
    }
}

/// Checks every window `s` shows, without waiting for the event loop.
pub unsafe fn alerts_check_session(s: &mut session) {
    unsafe {
        let windows: Vec<_> = s
            .windows
            .values()
            .filter_map(|wl| wl.window_handle().cloned())
            .collect();
        for window in windows {
            window.check_all_alerts();
        }
    }
}

/// Re-arms every window's silence timer, which is what an option change asks
/// for.
pub fn alerts_reset_all() {
    WINDOWS.with(|windows| unsafe {
        for w_ref in windows.iter().filter_map(|(_, window)| window.upgrade()) {
            w_ref.reset_alerts();
        }
    });
}

/// Passes an alert on to the user. Every client of the winlink's session that
/// is not a control client hears it: `visual-{bell,activity,silence}` off
/// means the terminal bell alone, on means the message alone and both means
/// both.
unsafe fn alerts_set_message(wl: &winlink, label: &CStr, option: &CStr) {
    unsafe {
        let session = wl.session().expect("a link has a session");
        let visual = (session.options()).number(option) as c_int;
        for mut c in client_walk() {
            if !c.attached_session().is_some_and(|s| session.ptr_eq(&s))
                || c.flags() & CLIENT_CONTROL as uint64_t != 0
            {
                continue;
            }

            if visual == VISUAL_OFF || visual == VISUAL_BOTH {
                tty_putcode(c.as_tty_mut(), TTYC_BEL);
            }
            if visual == VISUAL_OFF {
                continue;
            }
            if session
                .curw()
                .is_some_and(|current| current.index() == wl.idx)
            {
                status_message_set(
                    Some(c.as_client_mut()),
                    -1,
                    1,
                    0,
                    0,
                    c"%s in current window",
                    fmt_args![label],
                );
            } else {
                status_message_set(
                    Some(c.as_client_mut()),
                    -1,
                    1,
                    0,
                    0,
                    c"%s in window %d",
                    fmt_args![label, wl.idx],
                );
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/test_alerts.rs"]
mod tests;

#[cfg(test)]
pub(crate) use tests::queued_window_ids;

impl WindowRef {
    /// The silence timer expiring: nothing has been written to the window for
    /// `monitor-silence` seconds, so it is queued for a silence check.
    unsafe fn on_alert_timer(&self) {
        let w_ref = self;

        unsafe {
            log_debug(c"@%u alerts timer expired", fmt_args![w_ref.window_id()]);
            w_ref.raise_alerts(WINDOW_SILENCE);
        }
    }
    /// Checks every family against `w` and answers the window flags that applied.
    unsafe fn check_all_alerts(&self) -> c_int {
        let w = self;

        unsafe { w.check_alert(&BELL) | w.check_alert(&ACTIVITY) | w.check_alert(&SILENCE) }
    }
    /// One family's check: with the flag standing and the option watched, every
    /// winlink showing the window is marked and its hook raised, and the first of
    /// each session to get that far also puts a message on that session's clients.
    /// Answers the family's window flag, or zero if there was nothing to do.
    unsafe fn check_alert(&self, family: &Family) -> c_int {
        let w_ref = self;

        unsafe {
            if w_ref.alert_flags() & family.window_flag == 0 {
                return 0;
            }
            if w_ref.options().number(family.monitor) == 0 {
                return 0;
            }

            for held in showing(w_ref) {
                if held.get().is_some() {
                    held.session().set_alerted(false);
                }
            }

            for mut held in showing(w_ref) {
                let Some(flags) = held.get().map(|wl| wl.flags) else {
                    continue;
                };
                if !family.again && flags & family.winlink_flag != 0 {
                    continue;
                }
                let mut session = held.session().clone();
                let active = session
                    .curw()
                    .is_some_and(|current| current.index() == held.index());
                if !active || session.attached() == 0 {
                    if let Some(wl) = held.get_mut() {
                        wl.flags |= family.winlink_flag;
                    }
                    server_status_session(session.as_session_mut());
                }
                let Some(wl) = held.get() else {
                    continue;
                };
                if !alerts_action_applies(wl, family.action) {
                    continue;
                }
                notify_winlink(family.hook, wl);
                if session.alerted() {
                    continue;
                }
                session.set_alerted(true);
                if let Some(wl) = held.get() {
                    alerts_set_message(wl, family.label, family.visual);
                }
            }

            family.window_flag
        }
    }
    /// Whether any of the families in `flags` is watched on the window.
    unsafe fn alerts_enabled(&self, flags: c_int) -> bool {
        let w = self;

        unsafe {
            for family in [&BELL, &ACTIVITY, &SILENCE] {
                if flags & family.window_flag != 0 && w.options().number(family.monitor) != 0 {
                    return true;
                }
            }
            false
        }
    }
    /// Drops the window's silence flag and arms its silence timer afresh, for
    /// as many seconds as `monitor-silence` asks; zero seconds leaves it unarmed.
    unsafe fn reset_alerts(&self) {
        let w = self;

        unsafe {
            let seconds = w.options().number(c"monitor-silence");
            let weak = w.downgrade();
            w.reset_alert_timer(seconds as __time_t, move || {
                if let Some(window) = weak.upgrade() {
                    window.on_alert_timer();
                }
            });
            log_debug(
                c"@%u alerts timer reset %u",
                fmt_args![w.window_id(), seconds as u_int],
            );
        }
    }
    /// Records `flags` on `w` and, if anyone is watching any of them, puts the
    /// window on the queue the deferred check drains.
    pub unsafe fn raise_alerts(&self, flags: c_int) {
        let w = self;

        unsafe {
            w.reset_alerts();

            if w.add_alert_flags(flags) {
                log_debug(
                    c"@%u alerts flags added %#x",
                    fmt_args![w.window_id(), flags],
                );
            }

            if w.alerts_enabled(flags) {
                if w.queue_alerts() {
                    alerts_list.queue().push_back(w.clone());
                }

                if alerts_fired == 0 {
                    log_debug(c"alerts check queued (by @%u)", fmt_args![w.window_id()]);
                    reactor::current().defer(|| alerts_callback());
                    alerts_fired = 1;
                }
            }
        }
    }
}
