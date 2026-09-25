use super::{descriptor, ensure_runtime, handle};
use crate::src::shared::abi::timeval;
use crate::src::shared::event::{event, event_base, EventCallback};
use hmux_rt::{Handle as _, Runtime as _, Signals as _};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_short, c_void};
use std::future::{poll_fn, Future};
use std::pin::pin;
use std::rc::Rc;
use std::task::Poll;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
type Callback = EventCallback;
struct EventState {
    key: usize,
    fd: c_int,
    flags: c_short,
    callback: Callback,
    arg: *mut c_void,
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
    let states = EVENTS.with(|e| e.borrow_mut().drain().map(|(_, s)| s).collect::<Vec<_>>());
    for state in states {
        state.live.set(false);
        let task = state.task.borrow_mut().take();
        let activation = state.activation.borrow_mut().take();
        drop(task);
        drop(activation);
    }
}
fn fire(state: &Rc<EventState>, flags: c_short) {
    if !state.live.get() {
        return;
    }
    if state.flags & 0x10 == 0 {
        remove(state.key);
    }
    if let Some(cb) = state.callback {
        unsafe {
            cb(state.fd, flags, state.arg);
        }
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
                s.deadline.set(s.interval.map(|d| Instant::now() + d));
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
fn configure(ev: *mut event, interval: Option<Duration>) -> Rc<EventState> {
    unsafe {
        Rc::new(EventState {
            key: ev as usize,
            fd: (*ev).fd,
            flags: (*ev).flags,
            callback: (*ev).callback,
            arg: (*ev).arg,
            deadline: Cell::new(interval.map(|d| Instant::now() + d)),
            interval,
            active: Cell::new(0),
            live: Cell::new(true),
            task: RefCell::new(None),
            activation: RefCell::new(None),
            owned: RefCell::new(None),
        })
    }
}
fn delay(tv: &timeval) -> Duration {
    Duration::from_secs(tv.tv_sec.max(0) as u64)
        .saturating_add(Duration::from_micros(tv.tv_usec.max(0) as u64))
}
pub unsafe fn event_init() -> *mut event_base {
    ensure_runtime();
    std::ptr::NonNull::<event_base>::dangling().as_ptr()
}
pub unsafe fn event_reinit(_: *mut event_base) -> c_int {
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
pub unsafe fn event_loop(flags: c_int) -> c_int {
    super::run_once(flags & 2 != 0);
    0
}
pub fn event_initialized(ev: &event) -> c_int {
    ev.initialized as c_int
}
pub unsafe fn event_del(ev: *mut event) -> c_int {
    remove(ev as usize);
    0
}
pub unsafe fn event_set(
    ev: *mut event,
    fd: c_int,
    flags: c_short,
    callback: Callback,
    arg: *mut c_void,
) {
    event_del(ev);
    ev.write(event {
        initialized: true,
        fd,
        flags,
        callback,
        arg,
    });
}
pub unsafe fn event_add(ev: *mut event, timeout: *const timeval) -> c_int {
    ensure_runtime();
    event_del(ev);
    let state = configure(ev, timeout.as_ref().map(delay));
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
pub unsafe fn event_active(ev: *mut event, flags: c_int, _: c_short) {
    ensure_runtime();
    let existing = EVENTS.with(|e| e.borrow().get(&(ev as usize)).cloned());
    let state = existing.unwrap_or_else(|| {
        let s = configure(ev, None);
        EVENTS.with(|e| e.borrow_mut().insert(ev as usize, s.clone()));
        s
    });
    state.active.set(state.active.get() | flags as c_short);
    activate(&state);
}
pub unsafe fn event_once(
    fd: c_int,
    flags: c_short,
    cb: Callback,
    arg: *mut c_void,
    timeout: *const timeval,
) -> c_int {
    let mut ev: Box<event> = Box::default();
    event_set(&mut *ev, fd, flags & !0x10, cb, arg);
    let zero = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let timeout = if timeout.is_null() && flags & 6 == 0 {
        &zero
    } else {
        timeout
    };
    let result = event_add(&mut *ev, timeout);
    if result == 0 {
        let key = &*ev as *const event as usize;
        EVENTS.with(|e| *e.borrow().get(&key).unwrap().owned.borrow_mut() = Some(ev));
    }
    result
}
pub unsafe fn event_pending(ev: *const event, flags: c_short, out: *mut timeval) -> c_int {
    EVENTS.with(|e| {
        let e = e.borrow();
        let Some(s) = e.get(&(ev as usize)) else {
            return 0;
        };
        let mut pending = s.flags & 14;
        if let Some(deadline) = s.deadline.get() {
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
pub type event_log_cb = Option<unsafe extern "C" fn(c_int, *const c_char)>;
pub unsafe fn event_set_log_callback(_: event_log_cb) {}
