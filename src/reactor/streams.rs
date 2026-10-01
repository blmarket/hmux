mod api;
use super::{handle, io};
use crate::src::control::CONTROL_BUFFER_LOW;
use crate::src::shared::event::{bufferevent, bufferevent_data_cb, bufferevent_event_cb};
pub use api::*;
use hmux_buffer::{Buf, BufMut, SegmentedBuf};
use hmux_rt::AsyncFd as _;
use hmux_rt::Handle as _;
use std::cell::{Cell, RefCell};
use std::ffi::{c_int, c_short, c_void};
use std::future::{poll_fn, Future};
use std::pin::pin;
use std::rc::{Rc, Weak};
use std::task::{LocalWaker, Poll};
pub(crate) struct StreamState {
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
        Self(Rc::downgrade(
            (*stream).state.as_ref().expect("registered stream"),
        ))
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

    /// Legacy pointer view. The caller must keep the stream registered through use.
    pub fn ptr(&self) -> *mut bufferevent {
        self.with_ptr(|stream| stream)
            .unwrap_or(std::ptr::null_mut())
    }
}
thread_local! {
    // Enumeration only: I/O and teardown use the state carried by each object.
    static LIVE_STREAMS: RefCell<Vec<Weak<StreamState>>> = const { RefCell::new(Vec::new()) };
}
fn live_states() -> Vec<Rc<StreamState>> {
    LIVE_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        streams.retain(|stream| stream.strong_count() != 0);
        streams
            .iter()
            .filter_map(Weak::upgrade)
            .filter(|s| s.live.get())
            .collect()
    })
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
fn state(stream: &bufferevent) -> Rc<StreamState> {
    stream.state.as_ref().expect("live stream").clone()
}
pub(super) fn clear() {
    for state in live_states() {
        let stream = state
            .stream
            .borrow()
            .as_ref()
            .map(|s| (&**s as *const bufferevent).cast_mut());
        if let Some(stream) = stream {
            unsafe { bufferevent_free(stream) };
        }
    }
    LIVE_STREAMS.with(|streams| streams.borrow_mut().clear());
}

fn start(state: &Rc<StreamState>) -> std::io::Result<()> {
    // Empty panes keep stream buffers and an input parser without a PTY.
    if state.fd == -1 {
        return Ok(());
    }
    // SAFETY: the stream's model owner keeps this descriptor open through registration.
    let source = io(unsafe { std::os::fd::BorrowedFd::borrow_raw(state.fd) })?;
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
                        source.ready(read, write).await
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
                        let n = super::evbuffer_read(&mut (*stream).input, s.fd, count as c_int);
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
                        super::evbuffer_write(&mut (*stream).output, s.fd)
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
    // Explicit free breaks this ownership link, including streams without a task.
    (*stream).state = Some(s.clone());
    LIVE_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        streams.retain(|stream| stream.strong_count() != 0);
        streams.push(Rc::downgrade(&s));
    });
    if let Err(error) = start(&s) {
        bufferevent_free(stream);
        *libc::__errno_location() = if error.kind() == std::io::ErrorKind::Unsupported {
            libc::EOPNOTSUPP
        } else {
            error.raw_os_error().unwrap_or(libc::EIO)
        };
        return std::ptr::null_mut();
    }
    stream
}
pub unsafe fn bufferevent_free(stream: *mut bufferevent) {
    if stream.is_null() {
        return;
    }
    let s = (*stream).state.take();
    if let Some(s) = s {
        s.live.set(false);
        let task = s.task.borrow_mut().take();
        drop(task);
        if s.fd != -1 && s.pid == std::process::id() && s.original_flags & libc::O_NONBLOCK == 0 {
            libc::fcntl(s.fd, libc::F_SETFL, s.original_flags);
        }
        // Task state may still be retained by the callback that called free.
        // Detach the allocation now and release the slot borrow before capture Drop.
        let owner = s.stream.borrow_mut().take();
        drop(owner);
    }
}

/// Schedule an I/O recheck before lending input for synchronous consumption.
/// The local task runs after the caller returns to the reactor, so it observes
/// the updated length and can resume reads paused at the high watermark.
pub fn bufferevent_get_input(stream: &mut bufferevent) -> &mut SegmentedBuf {
    state(stream).wake();
    &mut stream.input
}
/// Schedule an I/O recheck before lending output for synchronous mutation.
/// Callers must finish modifying the buffer before returning to the reactor.
pub fn bufferevent_get_output(stream: &mut bufferevent) -> &mut SegmentedBuf {
    state(stream).wake();
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
    super::evbuffer_add(bufferevent_get_output(&mut *stream), data, size)
}
/// Move bytes into output. If the source belongs to another stream, obtain it
/// through bufferevent_get_input/output so that stream also rechecks its I/O.
pub unsafe fn bufferevent_write_buffer(
    stream: *mut bufferevent,
    buffer: &mut SegmentedBuf,
) -> c_int {
    if std::ptr::eq(buffer, &raw const *(*stream).output) {
        return -1;
    }
    (*(*stream).output).put(buffer);
    state(&*stream).wake();
    0
}
pub unsafe fn bufferevent_setwatermark(stream: *mut bufferevent) {
    (*stream).wm_write.low = CONTROL_BUFFER_LOW as usize;
    (*stream).wm_write.high = 0;

    state(&*stream).wake();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_buffer_access_wakes_and_explicit_free_invalidates_handles() {
        unsafe {
            let stream = bufferevent_new(-1, None, None, None);
            let mut handle = StreamHandle::from_ptr(stream);
            let observer = handle.clone();
            let state = state(&*stream);
            let generation = state.generation.get();
            bufferevent_write(stream, b"abc".as_ptr().cast(), 3);
            assert!(state.generation.get() > generation);
            assert_eq!((*stream).output.remaining(), 3);
            let generation = state.generation.get();
            super::super::evbuffer_add(&mut (*stream).input, b"x".as_ptr().cast(), 1);
            assert_eq!(state.generation.get(), generation);
            super::super::evbuffer_drain(bufferevent_get_input(&mut *stream), 1);
            assert_eq!(state.generation.get(), generation + 1);
            handle.free();
            assert!(!observer.is_alive());
            assert!(state.stream.borrow().is_none());
            assert!(!state.live.get());
            drop(state);
            assert!(observer.0.upgrade().is_none());
        }
    }

    pub(super) fn poll_until(mut ready: impl FnMut() -> bool) {
        use std::time::{Duration, Instant};

        let deadline = Instant::now() + Duration::from_secs(2);
        while !ready() {
            assert!(Instant::now() < deadline, "stream did not make progress");
            super::super::poll_runtime_with_timeout(Some(Duration::from_millis(1)));
        }
    }

    #[test]
    fn output_mutations_restart_idle_writes() {
        use std::io::Read;
        use std::os::fd::AsRawFd;
        use std::os::unix::net::UnixStream;

        let (socket, mut peer) = UnixStream::pair().unwrap();
        peer.set_nonblocking(true).unwrap();
        unsafe {
            let stream = bufferevent_new(socket.as_raw_fd(), None, None, None);
            assert!(!stream.is_null());
            let mut owner = StreamHandle::from_ptr(stream);
            // Default read is disabled and output is empty: no fd readiness
            // interest can drive this task until a buffer mutation wakes it.
            poll_until(|| state(&*stream).wake.borrow().is_some());
            for mode in 0..3 {
                match mode {
                    0 => {
                        bufferevent_write(stream, b"abc".as_ptr().cast(), 3);
                    }
                    1 => {
                        super::super::evbuffer_add_formatted(
                            bufferevent_get_output(&mut *stream),
                            |out| out.write_all(b"abc"),
                        );
                    }
                    _ => {
                        let mut source = SegmentedBuf::from(b"abc".to_vec());
                        bufferevent_write_buffer(stream, &mut source);
                        assert!(!source.has_remaining());
                    }
                }
                poll_until(|| !(*stream).output.has_remaining());
                let mut received = [0; 3];
                peer.read_exact(&mut received).unwrap();
                assert_eq!(&received, b"abc");
            }
            owner.free();
            super::super::shutdown_runtime();
        }
    }

    #[test]
    fn consuming_input_resumes_reads_at_high_watermark() {
        use std::io::Write;
        use std::os::fd::AsRawFd;
        use std::os::unix::net::UnixStream;

        let (socket, mut peer) = UnixStream::pair().unwrap();
        unsafe {
            let stream = bufferevent_new(socket.as_raw_fd(), None, None, None);
            assert!(!stream.is_null());
            let mut owner = StreamHandle::from_ptr(stream);
            (*stream).wm_read.high = 3;
            bufferevent_enable(stream, 2);
            peer.write_all(b"abcdef").unwrap();
            poll_until(|| (*stream).input.remaining() == 3);
            assert_eq!((*stream).input.chunk(), b"abc");
            // Let the task settle with reads paused before consuming input.
            let generation = state(&*stream).generation.get();
            poll_until(|| state(&*stream).wake.borrow().is_some());
            super::super::evbuffer_drain(bufferevent_get_input(&mut *stream), 3);
            assert!(state(&*stream).generation.get() > generation);
            poll_until(|| (*stream).input.remaining() == 3);
            assert_eq!((*stream).input.chunk(), b"def");
            owner.free();
            super::super::shutdown_runtime();
        }
    }

    #[test]
    fn runtime_clear_frees_streams_without_tasks() {
        unsafe {
            let first = StreamHandle::from_ptr(bufferevent_new(-1, None, None, None));
            let second = StreamHandle::from_ptr(bufferevent_new(-1, None, None, None));
            assert!(first.is_alive());
            assert!(second.is_alive());
            clear();
            assert!(!first.is_alive());
            assert!(!second.is_alive());
            assert!(first.0.upgrade().is_none());
            assert!(second.0.upgrade().is_none());
        }
    }
}
