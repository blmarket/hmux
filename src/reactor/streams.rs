use super::{descriptor, evbuffer, handle};
use crate::src::control::CONTROL_BUFFER_LOW;
use crate::src::shared::event::{bufferevent, bufferevent_data_cb, bufferevent_event_cb};
use hmux_buffer::{Buf, BufMut};
use hmux_rt::Handle as _;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ffi::{c_int, c_short, c_void};
use std::future::{poll_fn, Future};
use std::pin::pin;
use std::rc::{Rc, Weak};
use std::task::{LocalWaker, Poll};
struct StreamState {
    stream: RefCell<Option<Box<bufferevent>>>,
    fd: c_int,
    original_flags: c_int,
    pid: u32,
    live: Cell<bool>,
    generation: Cell<u64>,
    write_requested: Cell<bool>,
    wake: RefCell<Option<LocalWaker>>,
    task: RefCell<Option<hmux_rt::mio::Task>>,
}

/// Observe a runtime-owned stream without extending its allocation lifetime.
#[derive(Clone, Default)]
pub struct StreamHandle(Weak<StreamState>);

impl StreamHandle {
    /// The pointer must be null or refer to a stream registered by bufferevent_new.
    pub unsafe fn from_ptr(stream: *mut bufferevent) -> Self {
        if stream.is_null() {
            return Self::default();
        }
        STREAMS.with(|streams| {
            let streams = streams.borrow();
            let owner = streams.get(&(stream as usize)).expect("registered stream");
            Self(Rc::downgrade(owner))
        })
    }

    /// Keep the stream's allocation slot borrowed for one synchronous operation.
    pub fn with_ptr<R>(&self, access: impl FnOnce(*mut bufferevent) -> R) -> Option<R> {
        let owner = self.0.upgrade()?;
        if !owner.live.get() {
            return None;
        }
        let slot = owner.stream.borrow();
        let stream = slot.as_ref()?;
        Some(access((&**stream as *const bufferevent).cast_mut()))
    }

    pub fn is_alive(&self) -> bool {
        self.with_ptr(|_| ()).is_some()
    }

    /// Legacy pointer view. The caller must keep the stream registered through use.
    pub fn ptr(&self) -> *mut bufferevent {
        self.with_ptr(|stream| stream).unwrap_or(std::ptr::null_mut())
    }

    /// Free this registered stream at the existing teardown point.
    pub fn free(&mut self) {
        let stream = self.ptr();
        *self = Self::default();
        unsafe { bufferevent_free(stream) };
    }
}
thread_local! {
    static STREAMS: RefCell<HashMap<usize, Rc<StreamState>>> = RefCell::new(HashMap::new());
    static BUFFERS: RefCell<HashMap<usize, Weak<StreamState>>> = RefCell::new(HashMap::new());
}
impl StreamState {
    fn wake(&self) {
        self.generation.set(
            self.generation
                .get()
                .checked_add(1)
                .expect("stream generation exhausted"),
        );
        let wake = self.wake.borrow().clone();
        if let Some(wake) = wake {
            wake.wake();
        }
    }
}
pub(super) fn wake_buffer(buffer: &evbuffer) {
    let state = BUFFERS.with(|b| {
        b.borrow()
            .get(&(buffer as *const evbuffer as usize))
            .and_then(Weak::upgrade)
    });
    if let Some(state) = state {
        state.wake();
    }
}
fn state(stream: &bufferevent) -> Rc<StreamState> {
    STREAMS.with(|s| {
        s.borrow()
            .get(&(stream as *const bufferevent as usize))
            .expect("live stream")
            .clone()
    })
}
pub(super) fn stop_tasks() {
    let states = STREAMS.with(|s| s.borrow().values().cloned().collect::<Vec<_>>());
    for s in states {
        let task = s.task.borrow_mut().take();
        drop(task);
        s.wake.borrow_mut().take();
    }
}
pub(super) fn restart() {
    let states = STREAMS.with(|s| s.borrow().values().cloned().collect::<Vec<_>>());
    for s in states {
        start(&s).expect("rebuild stream");
    }
}
pub(super) fn clear() {
    let streams = STREAMS.with(|s| s.borrow().keys().copied().collect::<Vec<_>>());
    for stream in streams {
        unsafe {
            bufferevent_free(stream as *mut bufferevent);
        }
    }
}
fn start(state: &Rc<StreamState>) -> std::io::Result<()> {
    // Empty panes keep stream buffers and an input parser without a PTY.
    if state.fd == -1 {
        return Ok(());
    }
    let source = descriptor(state.fd)?;
    let s = state.clone();
    let task = handle().spawn(async move {
        // Each readiness delivery performs at most one 64 KiB read/write, then
        // yields. Input/output mutation wakes this same task; no callback queue.
        loop {
            if !s.live.get() {
                break;
            }
            let stream = s
                .stream
                .borrow_mut()
                .as_deref_mut()
                .map_or(std::ptr::null_mut(), |stream| stream as *mut bufferevent);
            let (read, write, generation) = unsafe {
                let b = &*stream;
                (
                    b.enabled & 2 != 0
                        && (b.wm_read.high == 0 || (*b.input).remaining() < b.wm_read.high),
                    b.enabled & 4 != 0 && ((*b.output).has_remaining() || s.write_requested.get()),
                    s.generation.get(),
                )
            };
            let readiness = {
                let wait = async {
                    if read || write {
                        source.wait(read, write).await
                    } else {
                        std::future::pending().await
                    }
                };
                let mut wait = pin!(wait);
                poll_fn(|cx| {
                    *s.wake.borrow_mut() = Some(cx.local_waker().clone());
                    if s.generation.get() != generation {
                        return Poll::Ready(None);
                    }
                    wait.as_mut().poll(cx).map(Some)
                })
                .await
            };
            if !s.live.get() {
                break;
            }
            let Some(readiness) = readiness else {
                continue;
            };
            let (readable, writable) = readiness.expect("stream readiness");
            unsafe {
                if readable && (*stream).enabled & 2 != 0 {
                    let high = (*stream).wm_read.high;
                    let count = if high == 0 {
                        65536
                    } else {
                        high.saturating_sub((*(*stream).input).remaining())
                            .min(65536)
                    };
                    if count > 0 {
                        let n = super::evbuffer_read(&mut *(*stream).input, s.fd, count as c_int);
                        if n > 0 {
                            if (*(*stream).input).remaining() >= (*stream).wm_read.low {
                                let cb = (*stream).readcb.clone();
                                if let Some(cb) = cb {
                                    let mut cb = cb.borrow_mut();
                                    (*cb)(std::ptr::NonNull::new(stream).expect("live stream"));
                                }
                            }
                        } else if n == 0
                            || (libc::EAGAIN != *libc::__errno_location()
                                && libc::EINTR != *libc::__errno_location())
                        {
                            (*stream).enabled &= !2;
                            let cb = (*stream).errorcb.clone();
                            if let Some(cb) = cb {
                                let mut cb = cb.borrow_mut();
                                (*cb)(
                                    std::ptr::NonNull::new(stream).expect("live stream"),
                                    1 | if n == 0 { 0x10 } else { 0x20 },
                                );
                            }
                        }
                    }
                }
                if !s.live.get() {
                    break;
                }
                if writable && (*stream).enabled & 4 != 0 {
                    let requested = s.write_requested.replace(false);
                    let empty = !(*(*stream).output).has_remaining();
                    let n = if empty {
                        0
                    } else {
                        super::evbuffer_write(&mut *(*stream).output, s.fd)
                    };
                    if n > 0 || (empty && requested) {
                        if (*(*stream).output).remaining() <= (*stream).wm_write.low {
                            let cb = (*stream).writecb.clone();
                            if let Some(cb) = cb {
                                let mut cb = cb.borrow_mut();
                                (*cb)(std::ptr::NonNull::new(stream).expect("live stream"));
                            }
                        }
                    } else if n < 0
                        && libc::EAGAIN != *libc::__errno_location()
                        && libc::EINTR != *libc::__errno_location()
                    {
                        (*stream).enabled &= !4;
                        let cb = (*stream).errorcb.clone();
                        if let Some(cb) = cb {
                            let mut cb = cb.borrow_mut();
                            (*cb)(
                                std::ptr::NonNull::new(stream).expect("live stream"),
                                2 | 0x20,
                            );
                        }
                    }
                }
            }
            if !s.live.get() {
                break;
            }
            super::yield_now().await;
        }
    })?;
    *state.task.borrow_mut() = Some(task);
    Ok(())
}
pub unsafe fn bufferevent_new(
    fd: c_int,
    readcb: bufferevent_data_cb,
    writecb: bufferevent_data_cb,
    errorcb: bufferevent_event_cb,
) -> *mut bufferevent {
    super::ensure_runtime();
    let original_flags = if fd == -1 {
        0
    } else {
        let flags = libc::fcntl(fd, libc::F_GETFL);
        if flags < 0 || libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) < 0 {
            return std::ptr::null_mut();
        }
        flags
    };
    let mut owner = Box::new(bufferevent {
        readcb,
        writecb,
        errorcb,
        enabled: 4,
        ..Default::default()
    });
    let stream = &raw mut *owner;
    let s = Rc::new(StreamState {
        stream: RefCell::new(Some(owner)),
        fd,
        original_flags,
        pid: std::process::id(),
        live: Cell::new(true),
        generation: Cell::new(0),
        write_requested: Cell::new(false),
        wake: RefCell::new(None),
        task: RefCell::new(None),
    });
    STREAMS.with(|r| r.borrow_mut().insert(stream as usize, s.clone()));
    BUFFERS.with(|b| {
        let mut b = b.borrow_mut();
        b.insert(
            (&*(*stream).input as *const evbuffer) as usize,
            Rc::downgrade(&s),
        );
        b.insert(
            (&*(*stream).output as *const evbuffer) as usize,
            Rc::downgrade(&s),
        );
    });
    if let Err(error) = start(&s) {
        bufferevent_free(stream);
        *libc::__errno_location() = error.raw_os_error().unwrap_or(libc::EIO);
        return std::ptr::null_mut();
    }
    stream
}
pub unsafe fn bufferevent_free(stream: *mut bufferevent) {
    if stream.is_null() {
        return;
    }
    let s = STREAMS.with(|r| r.borrow_mut().remove(&(stream as usize)));
    if let Some(s) = s {
        s.live.set(false);
        super::FDS.with(|f| f.borrow_mut().remove(&s.fd));
        let task = s.task.borrow_mut().take();
        drop(task);
        BUFFERS.with(|b| {
            let mut b = b.borrow_mut();
            b.remove(&((&*(*stream).input as *const evbuffer) as usize));
            b.remove(&((&*(*stream).output as *const evbuffer) as usize));
        });
        if s.fd != -1 && s.pid == std::process::id() && s.original_flags & libc::O_NONBLOCK == 0 {
            libc::fcntl(s.fd, libc::F_SETFL, s.original_flags);
        }
        // Task state may still be retained by the callback that called free.
        // Detach the allocation now and release the slot borrow before capture Drop.
        let owner = s.stream.borrow_mut().take();
        drop(owner);
    }
}

pub fn bufferevent_get_output(stream: &mut bufferevent) -> &mut evbuffer {
    &mut stream.output
}
pub unsafe fn bufferevent_enable(stream: *mut bufferevent, flags: c_short) -> c_int {
    let previous = (*stream).enabled;
    (*stream).enabled |= flags;
    let s = state(&*stream);
    if flags & 4 != 0 {
        s.write_requested.set(true);
    }
    if previous != (*stream).enabled || flags & 4 != 0 {
        s.wake();
    }
    0
}
pub unsafe fn bufferevent_disable(stream: *mut bufferevent, flags: c_short) -> c_int {
    let previous = (*stream).enabled;
    (*stream).enabled &= !flags;
    if previous != (*stream).enabled {
        state(&*stream).wake();
    }
    0
}
pub unsafe fn bufferevent_write(
    stream: *mut bufferevent,
    data: *const c_void,
    size: usize,
) -> c_int {
    super::evbuffer_add(&mut *(*stream).output, data, size)
}
pub unsafe fn bufferevent_write_buffer(stream: *mut bufferevent, buffer: &mut evbuffer) -> c_int {
    if std::ptr::eq(buffer, &raw const *(*stream).output) {
        return -1;
    }
    (*(*stream).output).put(&mut *buffer);
    wake_buffer(buffer);
    state(&*stream).wake();
    0
}
pub unsafe fn bufferevent_setwatermark(stream: *mut bufferevent) {
    (*stream).wm_write.low = CONTROL_BUFFER_LOW as usize;
    (*stream).wm_write.high = 0;

    state(&*stream).wake();
}
