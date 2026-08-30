//! The alert engine: how a bell, a burst of activity or a stretch of silence
//! in a window becomes a marked winlink, a hook and a message on the status
//! line of every client watching it.
//!
//! Nothing here checks anything the moment it is told. [`alerts_queue`] only
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
use crate::options::{options_get_number, options_ptr};
use crate::reactor;
use crate::reactor::{Reactor, Timer};
use crate::server::client_walk;
use crate::server::server_status_session;
use crate::session::{
    session_alerted, session_attached, session_get_curw, session_options, session_set_alerted,
};
use crate::status::status_message_set;
use crate::tree::GlobalQueue;
use crate::tty::tty_putcode;
pub use crate::types::*;
use crate::window::winlinks_into;
use crate::window::{
    window_find_by_id_ref, window_ref_from_ptr, windows, winlinks_after, winlinks_first,
};
use ::core::ffi::{CStr, c_int};
use ::core::ptr::null_mut;

pub const RB_NEGINF: c_int = -1;
pub const EV_TIMEOUT: c_int = 0x1;
pub const ALERT_ANY: c_int = 1;
pub const ALERT_CURRENT: c_int = 2;
pub const ALERT_OTHER: c_int = 3;
pub const VISUAL_OFF: c_int = 0;
pub const VISUAL_BOTH: c_int = 2;
pub const WINDOW_BELL: c_int = 0x1;
pub const WINDOW_ACTIVITY: c_int = 0x2;
pub const WINDOW_SILENCE: c_int = 0x4;
pub const WINDOW_ALERTFLAGS: c_int = WINDOW_BELL | WINDOW_ACTIVITY | WINDOW_SILENCE;
pub const WINLINK_BELL: c_int = 0x1;
pub const WINLINK_ACTIVITY: c_int = 0x2;
pub const WINLINK_SILENCE: c_int = 0x4;
pub const CLIENT_CONTROL: c_int = 0x2000;
pub const TTYC_BEL: tty_code_code = 4;

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
fn showing(w: *mut window) -> impl Iterator<Item = *mut winlink> {
    unsafe { winlinks_into(w) }
}

/// The winlinks of `s`, in index order.
fn windows_of(s: *mut session) -> impl Iterator<Item = *mut winlink> {
    let mut current = null_mut::<winlink>();
    let mut started = false;
    ::core::iter::from_fn(move || unsafe {
        current = if started {
            winlinks_after(current)
        } else {
            started = true;
            winlinks_first(&raw mut (*s).windows)
        };
        (!current.is_null()).then_some(current)
    })
}

/// Every window the server knows, in id order.
fn each_window() -> impl Iterator<Item = WindowRef> {
    let ids: Vec<u_int> = windows.map().keys().copied().collect();
    ids.into_iter().filter_map(window_find_by_id_ref)
}

/// The silence timer expiring: nothing has been written to the window for
/// `monitor-silence` seconds, so it is queued for a silence check.
unsafe fn alerts_timer(w: *mut window) {
    unsafe {
        let Some(w_ref) = window_ref_from_ptr(w) else {
            return;
        };
        let w = w_ref.as_ptr();
        log_debug(c"@%u alerts timer expired".as_ptr(), fmt_args![(*w).id]);
        alerts_queue(w, WINDOW_SILENCE);
    }
}

/// The deferred check ensure_reactor runs once per batch: every queued window is
/// checked, unlinked and cleared of its alert flags, and the latch goes down so
/// the next alert asks for a fresh callback. Dropping each queued handle at the
/// end of the callback releases the queue's ownership.
unsafe fn alerts_callback() {
    unsafe {
        let mut queued = ::core::mem::take(alerts_list.queue());
        while let Some(w_ref) = queued.pop_front() {
            let w = w_ref.as_ptr();
            let alerts = alerts_check_all(w);
            log_debug(
                c"@%u alerts check, alerts %#x".as_ptr(),
                fmt_args![(*w).id, alerts],
            );
            (*w).alerts_queued = 0;
            (*w).flags &= !WINDOW_ALERTFLAGS;
        }
        alerts_fired = 0;
    }
}

/// Whether `wl` is one the session's `{bell,activity,silence}-action` asks to
/// be told about: none means nothing happens, current means only the current
/// window and other means only windows that are not it.
unsafe fn alerts_action_applies(wl: *mut winlink, name: &CStr) -> bool {
    unsafe {
        let action = options_get_number(session_options((*wl).session()), name.as_ptr()) as c_int;
        if action == ALERT_ANY {
            return true;
        }
        if action == ALERT_CURRENT {
            return wl == session_get_curw((*wl).session());
        }
        if action == ALERT_OTHER {
            return wl != session_get_curw((*wl).session());
        }
        false
    }
}

/// Checks every family against `w` and answers the window flags that applied.
unsafe fn alerts_check_all(w: *mut window) -> c_int {
    unsafe { alerts_check(w, &BELL) | alerts_check(w, &ACTIVITY) | alerts_check(w, &SILENCE) }
}

/// One family's check: with the flag standing and the option watched, every
/// winlink showing `w` is marked and its hook raised, and the first one of
/// each session to get that far also puts a message on that session's clients.
/// Answers the family's window flag, or zero if there was nothing to do.
unsafe fn alerts_check(w: *mut window, family: &Family) -> c_int {
    unsafe {
        if (*w).flags & family.window_flag == 0 {
            return 0;
        }
        if options_get_number(options_ptr(&(*w).options), family.monitor.as_ptr()) == 0 {
            return 0;
        }

        for wl in showing(w) {
            session_set_alerted((*wl).session(), false);
        }

        for wl in showing(w) {
            if !family.again && (*wl).flags & family.winlink_flag != 0 {
                continue;
            }
            let s = (*wl).session();
            if session_get_curw(s) != wl || session_attached(s) == 0 {
                (*wl).flags |= family.winlink_flag;
                server_status_session(s);
            }
            if !alerts_action_applies(wl, family.action) {
                continue;
            }
            notify_winlink(family.hook.as_ptr(), wl);

            if session_alerted(s) {
                continue;
            }
            session_set_alerted(s, true);

            alerts_set_message(wl, family.label, family.visual);
        }

        family.window_flag
    }
}

/// Checks every window `s` shows, without waiting for the event loop.
pub unsafe fn alerts_check_session(s: *mut session) {
    unsafe {
        for wl in windows_of(s) {
            alerts_check_all((*wl).window());
        }
    }
}

/// Whether any of the families in `flags` is watched on `w`.
unsafe fn alerts_enabled(w: *mut window, flags: c_int) -> bool {
    unsafe {
        for family in [&BELL, &ACTIVITY, &SILENCE] {
            if flags & family.window_flag != 0
                && options_get_number(options_ptr(&(*w).options), family.monitor.as_ptr()) != 0
            {
                return true;
            }
        }
        false
    }
}

/// Re-arms every window's silence timer, which is what an option change asks
/// for.
pub fn alerts_reset_all() {
    unsafe {
        for w_ref in each_window() {
            alerts_reset(w_ref.as_ptr());
        }
    }
}

/// Drops `w`'s silence flag and arms its silence timer afresh, for as many
/// seconds as `monitor-silence` asks; zero seconds leaves it unarmed.
unsafe fn alerts_reset(w: *mut window) {
    unsafe {
        if !(*w).alerts_timer.is_set() {
            let w_weak = window_ref_from_ptr(w).map(|w_ref| w_ref.downgrade());
            (*w).alerts_timer.set_callback(move || {
                let Some(w_ref) = w_weak.as_ref().and_then(WindowWeak::upgrade) else {
                    return;
                };
                alerts_timer(w_ref.as_ptr());
            });
        }

        (*w).flags &= !WINDOW_SILENCE;
        (*w).alerts_timer.disarm();

        let tv = timeval {
            tv_sec: options_get_number(options_ptr(&(*w).options), c"monitor-silence".as_ptr())
                as __time_t,
            tv_usec: 0,
        };

        log_debug(
            c"@%u alerts timer reset %u".as_ptr(),
            fmt_args![(*w).id, tv.tv_sec as u_int],
        );
        if tv.tv_sec != 0 {
            (*w).alerts_timer.arm(tv);
        }
    }
}

/// The queue's contents, in arrival order, for the tests: membership used to
/// be readable through the links the `window` struct carried, and is not any
/// more.
#[cfg(test)]
pub(crate) fn queued_windows() -> Vec<*mut window> {
    alerts_list.queue().iter().map(WindowRef::as_ptr).collect()
}

/// Records `flags` on `w` and, if anyone is watching any of them, puts the
/// window on the queue the deferred check drains.
pub unsafe fn alerts_queue(w: *mut window, flags: c_int) {
    unsafe {
        alerts_reset(w);

        if (*w).flags & flags != flags {
            (*w).flags |= flags;
            log_debug(
                c"@%u alerts flags added %#x".as_ptr(),
                fmt_args![(*w).id, flags],
            );
        }

        if alerts_enabled(w, flags) {
            if (*w).alerts_queued == 0 {
                let Some(w_ref) = window_ref_from_ptr(w) else {
                    return;
                };
                (*w).alerts_queued = 1;
                alerts_list.queue().push_back(w_ref);
            }

            if alerts_fired == 0 {
                log_debug(c"alerts check queued (by @%u)".as_ptr(), fmt_args![(*w).id]);
                reactor::current().defer(|| alerts_callback());
                alerts_fired = 1;
            }
        }
    }
}

/// Passes an alert on to the user. Every client of the winlink's session that
/// is not a control client hears it: `visual-{bell,activity,silence}` off
/// means the terminal bell alone, on means the message alone and both means
/// both.
unsafe fn alerts_set_message(wl: *mut winlink, label: &CStr, option: &CStr) {
    unsafe {
        let visual = options_get_number(session_options((*wl).session()), option.as_ptr()) as c_int;
        for c in client_walk() {
            if (*c).session != (*wl).session() || (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                continue;
            }

            if visual == VISUAL_OFF || visual == VISUAL_BOTH {
                tty_putcode(&raw mut (*c).tty, TTYC_BEL);
            }
            if visual == VISUAL_OFF {
                continue;
            }
            if session_get_curw((*c).session) == wl {
                status_message_set(
                    c,
                    -1,
                    1,
                    0,
                    0,
                    c"%s in current window".as_ptr(),
                    fmt_args![label.as_ptr()],
                );
            } else {
                status_message_set(
                    c,
                    -1,
                    1,
                    0,
                    0,
                    c"%s in window %d".as_ptr(),
                    fmt_args![label.as_ptr(), (*wl).idx],
                );
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/test_alerts.rs"]
mod tests;
