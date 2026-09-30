use super::{descriptor, ensure_runtime, handle};
use crate::src::shared::event::EV_TIMEOUT;
use crate::src::shared::event::{event, event_base, EventCallback};
use hmux_rt::{Handle as _, Runtime as _, Signals as _};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_short};
use std::future::{poll_fn, Future};
use std::pin::pin;
use std::rc::Rc;
use std::task::Poll;
use std::time::{Duration, Instant};
type Callback = EventCallback;
struct EventState {
    key: usize,
    fd: c_int,
    flags: c_short,
    callback: Callback,
    deadline: Cell<Option<Instant>>,
    interval: Option<Duration>,
    active: Cell<c_short>,
    live: Cell<bool>,
    task: RefCell<Option<hmux_rt::mio::Task>>,
    activation: RefCell<Option<hmux_rt::mio::Task>>,
    owned: RefCell<Option<Box<event>>>,
}
thread_local! { static EVENTS: RefCell<HashMap<usize, Rc<EventState>>> = RefCell::new(HashMap::new()); }
fn remove(key: usize) {
    let state = EVENTS.with(|e| e.borrow_mut().remove(&key));
    if let Some(state) = state {
        state.live.set(false);
        super::FDS.with(|f| f.borrow_mut().remove(&state.fd));
        let task = state.task.borrow_mut().take();
        let activation = state.activation.borrow_mut().take();
        drop(task);
        drop(activation);
    }
}
pub(super) fn clear() {
    loop {
        let states = EVENTS.with(|e| e.borrow_mut().drain().map(|(_, s)| s).collect::<Vec<_>>());
        if states.is_empty() {
            break;
        }
        for state in states {
            state.live.set(false);
            let task = state.task.borrow_mut().take();
            let activation = state.activation.borrow_mut().take();
            drop(task);
            drop(activation);
        }
        // Dropping callback owners may defer their own cleanup with event_once.
        // Cancel those registrations too before the runtime is torn down.
    }
}
fn fire(state: &Rc<EventState>, flags: c_short) {
    if !state.live.get() {
        return;
    }
    if state.flags & 0x10 == 0 {
        remove(state.key);
    }
    if let Some(cb) = &state.callback {
        let mut cb = cb.borrow_mut();
        (*cb)(state.fd, flags);
    }
}
fn start(state: &Rc<EventState>) -> std::io::Result<()> {
    let h = handle();
    let source = if state.flags & 6 != 0 {
        Some(descriptor(state.fd)?)
    } else {
        None
    };
    let mut signals = if state.flags & 8 != 0 {
        Some(h.signals(&[state.fd])?)
    } else {
        None
    };
    let s = state.clone();
    let task = h.clone().spawn(async move {
        loop {
            let result = {
                let io = async {
                    if let Some(signals) = signals.as_mut() {
                        signals.recv().await.map(|_| 8)
                    } else if let Some(source) = source.as_ref() {
                        source
                            .wait(s.flags & 2 != 0, s.flags & 4 != 0)
                            .await
                            .map(|(r, w)| (if r { 2 } else { 0 }) | (if w { 4 } else { 0 }))
                    } else {
                        std::future::pending().await
                    }
                };
                let timer = async {
                    if let Some(deadline) = s.deadline.get() {
                        h.sleep_until(deadline).await.map(|_| 1)
                    } else {
                        std::future::pending().await
                    }
                };
                let mut io = pin!(io);
                let mut timer = pin!(timer);
                poll_fn(|cx| {
                    if let Poll::Ready(result) = io.as_mut().poll(cx) {
                        return Poll::Ready(result);
                    }
                    timer.as_mut().poll(cx)
                })
                .await
            };
            if !s.live.get() {
                break;
            }
            let flags = result.expect("descriptor/signal wait failed");
            if s.flags & 0x10 != 0 {
                s.deadline.set(s.interval.map(deadline));
            }
            fire(&s, flags);
            if !s.live.get() {
                break;
            }
            super::yield_now().await;
        }
    })?;
    *state.task.borrow_mut() = Some(task);
    Ok(())
}
fn deadline(interval: Duration) -> Instant {
    let now = Instant::now();
    // Treat an overflowing deadline as expired, as the pinned libevent-based
    // server does. Keep the timer armed rather than silently disabling it.
    now.checked_add(interval).unwrap_or(now)
}
fn configure(ev: &event, interval: Option<Duration>) -> Rc<EventState> {
    Rc::new(EventState {
        key: ev as *const event as usize,
        fd: ev.fd,
        flags: ev.flags,
        callback: ev.callback.clone(),
        deadline: Cell::new(interval.map(deadline)),
        interval,
        active: Cell::new(0),
        live: Cell::new(true),
        task: RefCell::new(None),
        activation: RefCell::new(None),
        owned: RefCell::new(None),
    })
}
pub unsafe fn event_init() -> *mut event_base {
    ensure_runtime();
    std::ptr::NonNull::<event_base>::dangling().as_ptr()
}
pub unsafe fn event_reinit() -> c_int {
    if super::PID.with(|p| p.get() == std::process::id()) {
        return 0;
    }
    let states = EVENTS.with(|e| e.borrow().values().cloned().collect::<Vec<_>>());
    for s in &states {
        let task = s.task.borrow_mut().take();
        let activation = s.activation.borrow_mut().take();
        drop(task);
        drop(activation);
    }
    super::streams::stop_tasks();
    super::FDS.with(|f| f.borrow_mut().clear());
    let result = super::HOST.with(|h| {
        let mut h = h.borrow_mut();
        let runtime = h.as_mut().expect("runtime initialized");
        runtime.reset_after_fork()?;
        super::HANDLE.with(|h| *h.borrow_mut() = Some(runtime.handle()));
        Ok::<_, std::io::Error>(())
    });
    if result.is_err() {
        return -1;
    }
    super::PID.with(|p| p.set(std::process::id()));
    for s in states {
        if start(&s).is_err() {
            return -1;
        }
        if s.active.get() != 0 {
            activate(&s);
        }
    }
    super::streams::restart();
    0
}
pub unsafe fn event_loop() -> c_int {
    super::run_once();
    0
}
pub fn event_initialized(ev: &event) -> c_int {
    ev.initialized as c_int
}
pub unsafe fn event_del(ev: *mut event) -> c_int {
    remove(ev as usize);
    0
}
pub unsafe fn event_set<F>(ev: *mut event, fd: c_int, flags: c_short, callback: F)
where
    F: FnMut(c_int, c_short) + 'static,
{
    event_del(ev);
    *ev = event {
        initialized: true,
        fd,
        flags,
        callback: Some(std::rc::Rc::new(RefCell::new(Box::new(callback)))),
    };
}
/// Arm an event; None waits for I/O or signals without a timer.
/// Some(Duration::ZERO) schedules an immediate timeout.
pub unsafe fn event_add(ev: *mut event, timeout: Option<Duration>) -> c_int {
    ensure_runtime();
    event_del(ev);
    let state = configure(&*ev, timeout);
    EVENTS.with(|e| e.borrow_mut().insert(ev as usize, state.clone()));
    if let Err(error) = start(&state) {
        remove(ev as usize);
        *libc::__errno_location() = error.raw_os_error().unwrap_or(libc::EIO);
        return -1;
    }
    0
}
fn activate(state: &Rc<EventState>) {
    if state.activation.borrow().is_some() {
        return;
    }
    let s = state.clone();
    let task = handle()
        .spawn(async move {
            let flags = s.active.replace(0);
            let task = s.activation.borrow_mut().take();
            drop(task);
            fire(&s, flags);
        })
        .expect("activate event");
    *state.activation.borrow_mut() = Some(task);
}
pub unsafe fn event_active(ev: *mut event) {
    let flags: c_int = EV_TIMEOUT;
    ensure_runtime();
    let existing = EVENTS.with(|e| e.borrow().get(&(ev as usize)).cloned());
    let state = existing.unwrap_or_else(|| {
        let s = configure(&*ev, None);
        EVENTS.with(|e| e.borrow_mut().insert(ev as usize, s.clone()));
        s
    });
    state.active.set(state.active.get() | flags as c_short);
    activate(&state);
}
pub unsafe fn event_once<F>(cb: F) -> c_int
where
    F: FnMut(c_int, c_short) + 'static,
{
    let mut ev: Box<event> = Box::default();
    event_set(&mut *ev, -1, EV_TIMEOUT as c_short, cb);
    let zero = Duration::ZERO;
    let result = event_add(&mut *ev, Some(zero));
    if result == 0 {
        let key = &*ev as *const event as usize;
        EVENTS.with(|e| *e.borrow().get(&key).unwrap().owned.borrow_mut() = Some(ev));
    }
    result
}
/// Schedule a one-shot callback that owns a record containing its event handle.
///
/// The record stays in its original Box allocation. Only the reactor holds the
/// callback, so the embedded event cannot keep its enclosing owner alive in a
/// cycle. A missing timeout activates the callback immediately, like event_active.
pub fn event_once_owned<T: 'static>(
    mut owner: Box<T>,
    event_handle: fn(&mut T) -> &mut event,
    timeout: Option<Duration>,
    callback: impl FnOnce(Box<T>) + 'static,
) -> c_int {
    ensure_runtime();
    let ev = event_handle(&mut owner);
    remove(ev as *mut event as usize);
    *ev = event {
        initialized: true,
        fd: -1,
        flags: 0,
        callback: None,
    };
    let mut state = configure(ev, timeout);
    let mut pending = Some((owner, callback));
    Rc::get_mut(&mut state).unwrap().callback =
        Some(Rc::new(RefCell::new(Box::new(move |_, _| {
            if let Some((owner, callback)) = pending.take() {
                callback(owner);
            }
        }))));
    EVENTS.with(|events| events.borrow_mut().insert(state.key, state.clone()));
    if timeout.is_some() {
        if let Err(error) = start(&state) {
            remove(state.key);
            unsafe { *libc::__errno_location() = error.raw_os_error().unwrap_or(libc::EIO) };
            return -1;
        }
    } else {
        state.active.set(EV_TIMEOUT as c_short);
        activate(&state);
    }
    0
}
/// Report pending flags, optionally returning the monotonic timer deadline.
pub unsafe fn event_pending(ev: *const event, flags: c_short, out: Option<&mut Instant>) -> c_int {
    EVENTS.with(|e| {
        let e = e.borrow();
        let Some(s) = e.get(&(ev as usize)) else {
            return 0;
        };
        let mut pending = s.flags & 14;
        if let Some(deadline) = s.deadline.get() {
            pending |= 1;
            if let Some(out) = out {
                *out = deadline;
            }
        }
        pending |= s.active.get();
        (pending & flags) as c_int
    })
}
pub unsafe fn event_get_method() -> *const c_char {
    c"mio".as_ptr()
}
pub unsafe fn event_get_version() -> *const c_char {
    c"hmux-rt 0.1".as_ptr()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_timer_reports_monotonic_deadline_and_cancellation() {
        unsafe {
            event_init();
            let mut ev = event::default();
            event_set(&mut ev, -1, 0, |_, _| {});
            let before = Instant::now();
            let delay = Duration::from_millis(250);
            assert_eq!(event_add(&mut ev, Some(delay)), 0);
            let after = Instant::now();
            let mut deadline = before;
            assert_eq!(
                event_pending(&ev, EV_TIMEOUT as _, Some(&mut deadline)),
                EV_TIMEOUT
            );
            assert!(deadline >= before + delay && deadline <= after + delay);
            event_del(&mut ev);
            assert_eq!(event_pending(&ev, EV_TIMEOUT as _, None), 0);
            assert_eq!(event_add(&mut ev, None), 0);
            assert_eq!(event_pending(&ev, EV_TIMEOUT as _, None), 0);
            event_del(&mut ev);
            super::super::shutdown_runtime();
        }
    }

    #[test]
    fn overflowing_timers_remain_armed_and_fire_including_repeats() {
        unsafe {
            event_init();
            for flags in [0, 0x10] {
                let mut ev = event::default();
                let calls = Rc::new(Cell::new(0));
                let observed = calls.clone();
                event_set(&mut ev, -1, flags, move |_, flags| {
                    assert_eq!(flags, EV_TIMEOUT as c_short);
                    observed.set(observed.get() + 1);
                });
                let timeout = Duration::MAX;
                assert_eq!(event_add(&mut ev, Some(timeout)), 0);
                assert_eq!(event_pending(&ev, 1, None), 1);
                for _ in 0..3 {
                    super::super::HOST.with(|host| {
                        host.borrow_mut()
                            .as_mut()
                            .unwrap()
                            .poll(Some(Duration::from_millis(10)))
                            .unwrap();
                    });
                }
                event_del(&mut ev);
                if flags == 0 {
                    assert_eq!(calls.get(), 1);
                } else {
                    assert!(calls.get() >= 2, "persistent timer must rearm");
                }
            }
            super::super::shutdown_runtime();
        }
    }
}
