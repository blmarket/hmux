//! Process-local event compatibility. Embedded handles contain no Rust owners.
use super::{control, ensure_runtime};
use crate::src::shared::abi::timeval;
use crate::src::shared::event::{event, event_base};
use hmux_rt::adapters::{WatchInterest, WatchMode};
use std::ffi::{c_int, c_short, c_void};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub unsafe fn event_init() -> *mut event_base {
    ensure_runtime(false);
    std::ptr::NonNull::<event_base>::dangling().as_ptr()
}
pub unsafe fn event_reinit(_: *mut event_base) -> c_int {
    ensure_runtime(true);
    0
}
pub unsafe fn event_loop(_: c_int) -> c_int {
    super::run_once();
    0
}
pub unsafe fn event_initialized(ev: *const event) -> c_int {
    (*ev).initialized as c_int
}
pub unsafe fn event_del(ev: *mut event) -> c_int {
    let ctl = control();
    ctl.release_timer((*ev).timer);
    ctl.release_io((*ev).io);
    ctl.release_signal((*ev).signal);
    ctl.cancel_deferred((*ev).deferred);
    (*ev).timer = 0;
    (*ev).io = 0;
    (*ev).signal = 0;
    (*ev).deferred = 0;
    0
}
pub unsafe fn event_set(
    ev: *mut event,
    fd: c_int,
    flags: c_short,
    cb: Option<unsafe extern "C" fn(c_int, c_short, *mut c_void)>,
    arg: *mut c_void,
) {
    event_del(ev);
    *ev = event {
        fd,
        flags,
        initialized: true,
        callback: cb,
        arg,
        ..event::ZERO
    };
}
unsafe fn fire(ev: *mut event, flags: c_short) {
    let fd = (*ev).fd;
    let cb = (*ev).callback;
    let arg = (*ev).arg;
    if (*ev).flags & 0x10 == 0 {
        event_del(ev);
    }
    if let Some(cb) = cb {
        cb(fd, flags, arg);
    }
}
pub unsafe fn event_add(ev: *mut event, timeout: *const timeval) -> c_int {
    event_del(ev);
    let ctl = control();
    if (*ev).flags & 8 != 0 {
        (*ev).signal = ctl.watch_signal((*ev).fd, move |_, flags| unsafe { fire(ev, flags) });
    } else if (*ev).flags & 6 != 0 {
        let interest = match (*ev).flags & 6 {
            2 => WatchInterest::Read,
            4 => WatchInterest::Write,
            _ => WatchInterest::ReadWrite,
        };
        let mode = if (*ev).flags & 0x10 != 0 {
            WatchMode::Persistent
        } else {
            WatchMode::Once
        };
        (*ev).io = ctl.allocate_io((*ev).fd, interest, mode, move |_, flags| unsafe {
            fire(ev, flags)
        });
        ctl.enable_io((*ev).io);
    }
    if !timeout.is_null() {
        let delay = Duration::from_secs((*timeout).tv_sec.max(0) as u64)
            .saturating_add(Duration::from_micros((*timeout).tv_usec.max(0) as u64));
        if delay.is_zero() {
            (*ev).deferred = ctl.defer(move || unsafe {
                (*ev).deferred = 0;
                fire(ev, 1);
            });
        } else {
            (*ev).timer = ctl.allocate_timer(move || unsafe { fire(ev, 1) });
            ctl.arm_timer((*ev).timer, delay);
        }
    }
    0
}
pub unsafe fn event_active(ev: *mut event, flags: c_int, _: c_short) {
    control().cancel_deferred((*ev).deferred);
    (*ev).deferred = control().defer(move || unsafe {
        (*ev).deferred = 0;
        fire(ev, flags as c_short);
    });
}
pub unsafe fn event_once(
    fd: c_int,
    flags: c_short,
    cb: Option<unsafe extern "C" fn(c_int, c_short, *mut c_void)>,
    arg: *mut c_void,
    timeout: *const timeval,
) -> c_int {
    use std::cell::Cell;
    use std::rc::Rc;
    let ctl = control();
    let ids = Rc::new(Cell::new((0usize, 0usize, 0usize)));
    let live = Rc::new(Cell::new(true));
    let callback: Rc<dyn Fn(c_short)> = {
        let ids = ids.clone();
        let ctl = ctl.clone();
        Rc::new(move |what| {
            if !live.replace(false) {
                return;
            }
            let (timer, io, deferred) = ids.get();
            ctl.release_timer(timer);
            ctl.release_io(io);
            ctl.cancel_deferred(deferred);
            if let Some(cb) = cb {
                unsafe {
                    cb(fd, what, arg);
                }
            }
        })
    };
    let mut registrations = (0, 0, 0);
    if flags & 6 != 0 {
        let interest = match flags & 6 {
            2 => WatchInterest::Read,
            4 => WatchInterest::Write,
            _ => WatchInterest::ReadWrite,
        };
        let cb = callback.clone();
        registrations.1 = ctl.allocate_io(fd, interest, WatchMode::Once, move |_, what| cb(what));
        ctl.enable_io(registrations.1);
    }
    if !timeout.is_null() || flags & 6 == 0 {
        let delay = if timeout.is_null() {
            Duration::ZERO
        } else {
            Duration::from_secs((*timeout).tv_sec.max(0) as u64)
                .saturating_add(Duration::from_micros((*timeout).tv_usec.max(0) as u64))
        };
        if delay.is_zero() {
            registrations.2 = ctl.defer(move || callback(1));
        } else {
            registrations.0 = ctl.allocate_timer(move || callback(1));
            ctl.arm_timer(registrations.0, delay);
        }
    }
    ids.set(registrations);
    0
}
pub unsafe fn event_pending(ev: *const event, flags: c_short, out: *mut timeval) -> c_int {
    let mut pending = 0;
    if (*ev).io != 0 {
        pending |= (*ev).flags & 6;
    }
    if (*ev).signal != 0 {
        pending |= 8;
    }
    if let Some(deadline) = control().timer_deadline((*ev).timer) {
        pending |= 1;
        if !out.is_null() {
            let absolute = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .saturating_add(deadline.saturating_duration_since(Instant::now()));
            (*out).tv_sec = absolute.as_secs() as _;
            (*out).tv_usec = absolute.subsec_micros() as _;
        }
    }
    if (*ev).deferred != 0 {
        pending |= 1;
    }
    (pending & flags) as c_int
}

#[cfg(test)]
mod tests {
    use super::*;
    unsafe extern "C" fn count(_: c_int, _: c_short, arg: *mut c_void) {
        *(arg as *mut usize) += 1;
    }
    #[test]
    fn configured_pending_rearm_and_cancel_are_distinct() {
        unsafe {
            super::super::shutdown_runtime();
            event_init();
            let mut ev = event::ZERO;
            let mut calls = 0usize;
            event_set(
                &mut ev,
                -1,
                0,
                Some(count),
                (&mut calls as *mut usize).cast(),
            );
            assert_eq!(event_initialized(&ev), 1);
            assert_eq!(event_pending(&ev, 1, std::ptr::null_mut()), 0);
            let timeout = timeval {
                tv_sec: 30,
                tv_usec: 0,
            };
            let mut deadline = timeval {
                tv_sec: 0,
                tv_usec: 0,
            };
            event_add(&mut ev, &timeout);
            assert_eq!(event_pending(&ev, 1, &mut deadline), 1);
            assert!(deadline.tv_sec > 0);
            event_add(
                &mut ev,
                &timeval {
                    tv_sec: 0,
                    tv_usec: 0,
                },
            );
            assert_eq!(calls, 0);
            event_del(&mut ev);
            event_loop(1);
            assert_eq!(calls, 0);
            event_active(&mut ev, 1, 1);
            assert_eq!(calls, 0);
            event_loop(1);
            assert_eq!(calls, 1);
            assert_eq!(event_pending(&ev, 1, std::ptr::null_mut()), 0);
            assert_eq!(event_initialized(&ev), 1);
            super::super::shutdown_runtime();
        }
    }
    struct Owner {
        ev: event,
        calls: *mut usize,
    }
    unsafe extern "C" fn free_owner(_: c_int, _: c_short, arg: *mut c_void) {
        let owner = Box::from_raw(arg as *mut Owner);
        *owner.calls += 1;
    }
    #[test]
    fn callback_can_free_owner_after_repeated_activation() {
        unsafe {
            super::super::shutdown_runtime();
            event_init();
            let mut calls = 0usize;
            let owner = Box::into_raw(Box::new(Owner {
                ev: event::ZERO,
                calls: &mut calls,
            }));
            event_set(&mut (*owner).ev, -1, 0, Some(free_owner), owner.cast());
            event_active(&mut (*owner).ev, 1, 1);
            event_active(&mut (*owner).ev, 1, 1);
            event_loop(1);
            event_loop(1);
            assert_eq!(calls, 1);
            super::super::shutdown_runtime();
        }
    }
}
