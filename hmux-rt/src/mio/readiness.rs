use std::cell::{Cell, RefCell};
use std::future::{Future, poll_fn};
use std::io;
use std::os::fd::{AsRawFd, OwnedFd, RawFd};
use std::pin::{Pin, pin};
use std::rc::{Rc, Weak};
use std::task::{Context, LocalWaker, Poll};

use mio::unix::SourceFd;

use super::runtime::{Core, invalid};
#[derive(Clone, Copy)]
pub(super) enum Direction {
    Read,
    Write,
}

#[derive(Clone, Copy, Default)]
pub(super) struct Readiness {
    closed: bool,
    error: bool,
}

#[derive(Default)]
struct DirectionState {
    cached: Cell<Option<Readiness>>,
    generation: Cell<usize>,
    waiter: RefCell<Option<(usize, LocalWaker)>>,
}

pub(crate) struct IoState {
    core: Weak<Core>,
    id: usize,
    raw: RawFd,
    fd: RefCell<Option<OwnedFd>>,
    registered: bool,
    read: DirectionState,
    write: DirectionState,
}

impl IoState {
    pub(super) fn check(&self) -> io::Result<()> {
        self.core.upgrade().ok_or_else(invalid)?.check()
    }

    fn direction(&self, direction: Direction) -> &DirectionState {
        match direction {
            Direction::Read => &self.read,
            Direction::Write => &self.write,
        }
    }

    pub(crate) fn raw(&self) -> RawFd {
        self.raw
    }

    pub(crate) fn observe(&self, event: &mio::event::Event) -> io::Result<()> {
        let core = self.core.upgrade().ok_or_else(invalid)?;
        for (direction, ready, closed) in [
            (Direction::Read, event.is_readable(), event.is_read_closed()),
            (
                Direction::Write,
                event.is_writable(),
                event.is_write_closed(),
            ),
        ] {
            if !ready && !closed && !event.is_error() {
                continue;
            }
            let state = self.direction(direction);
            let generation = state
                .generation
                .get()
                .checked_add(1)
                .ok_or_else(|| io::Error::other("readiness generation exhausted"))?;
            state.generation.set(generation);
            let previous = state.cached.get().unwrap_or_default();
            state.cached.set(Some(Readiness {
                closed: closed || previous.closed,
                error: event.is_error() || previous.error,
            }));
            let wake = state.waiter.borrow().as_ref().map(|(_, wake)| wake.clone());
            if let Some(wake) = wake {
                core.queue_wake(wake);
            }
        }
        Ok(())
    }

    pub(super) fn poll_ready(
        &self,
        direction: Direction,
        id: &mut Option<usize>,
        context: &mut Context<'_>,
    ) -> Poll<io::Result<(usize, Readiness)>> {
        let core = self.core.upgrade().ok_or_else(invalid)?;
        core.check()?;
        if self.fd.borrow().is_none() {
            return Poll::Ready(Err(invalid()));
        }
        let state = self.direction(direction);
        if state
            .waiter
            .borrow()
            .as_ref()
            .is_some_and(|(existing, _)| Some(*existing) != *id)
        {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "direction already has a waiter",
            )));
        }
        let id = match *id {
            Some(id) => id,
            None => {
                let next = core.allocate()?;
                *id = Some(next);
                next
            }
        };
        *state.waiter.borrow_mut() = Some((id, context.local_waker().clone()));
        if !self.registered {
            return Poll::Ready(Ok((state.generation.get(), Readiness::default())));
        }
        if let Some(ready) = state.cached.get() {
            return Poll::Ready(Ok((state.generation.get(), ready)));
        }
        Poll::Pending
    }

    pub(super) fn remove_waiter(&self, direction: Direction, id: Option<usize>) {
        let state = self.direction(direction);
        let mut waiter = state.waiter.borrow_mut();
        if waiter
            .as_ref()
            .is_some_and(|(existing, _)| Some(*existing) == id)
        {
            waiter.take();
        }
    }

    pub(super) fn clear(&self, direction: Direction, generation: usize) {
        let state = self.direction(direction);
        if state.generation.get() == generation {
            state.cached.set(None);
        }
    }

    pub(crate) fn close(&self, normal: bool) {
        // Retain ownership until deregistration and waiter cleanup are complete.
        let fd = self.fd.borrow_mut().take();
        if fd.is_none() {
            return;
        }
        if let Some(core) = self.core.upgrade() {
            if normal && self.registered {
                let error = core
                    .registry
                    .borrow()
                    .as_ref()
                    .and_then(|registry| registry.deregister(&mut SourceFd(&self.raw)).err());
                if let Some(error) = error {
                    core.record(error);
                }
            }
            core.io.borrow_mut().remove(&self.id);
            if core.fds.borrow().get(&self.raw) == Some(&self.id) {
                core.fds.borrow_mut().remove(&self.raw);
            }
        }
        self.read.waiter.borrow_mut().take();
        self.write.waiter.borrow_mut().take();
        drop(fd);
    }
}

impl Drop for IoState {
    fn drop(&mut self) {
        let normal = self
            .core
            .upgrade()
            .is_some_and(|core| core.pid == std::process::id());
        self.close(normal);
    }
}

/// An owned descriptor with [`crate::AsyncFd`] readiness
/// waits and [`crate::AsyncRead`]/[`crate::AsyncWrite`] byte-stream operations.
/// Use readiness waits with your own bounded, nonblocking syscalls for listeners,
/// datagrams, or terminals. Regular files bypass the readiness poller and perform
/// synchronous I/O on the runtime thread. Devices rejected by the poller can
/// also perform immediate I/O, but cannot wait for a later readiness event.
///
/// Custom operations are not part of the public byte-stream API:
/// ```compile_fail
/// use hmux_rt::mio::Io;
/// fn custom(io: &Io) {
///     let _ = io.read_with(|| Ok(()));
/// }
/// ```
/// ```compile_fail
/// use hmux_rt::mio::Io;
/// fn custom(io: &Io) {
///     let _ = io.write_with(|| Ok(()));
/// }
/// ```
pub struct Io {
    pub(crate) state: Rc<IoState>,
}

impl Io {
    pub(crate) fn new(core: &Rc<Core>, fd: OwnedFd) -> io::Result<Self> {
        core.check()?;
        let raw = fd.as_raw_fd();
        if core.fds.borrow().contains_key(&raw) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "descriptor already registered",
            ));
        }
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: fd is owned and stat provides valid output storage.
        if unsafe { libc::fstat(raw, stat.as_mut_ptr()) } < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: successful fstat initialized stat.
        let regular = unsafe { stat.assume_init() }.st_mode & libc::S_IFMT == libc::S_IFREG;
        // SAFETY: fcntl only queries flags on a live, owned descriptor.
        let flags = unsafe { libc::fcntl(raw, libc::F_GETFL) };
        if flags < 0 {
            return Err(io::Error::last_os_error());
        }
        if !regular && flags & libc::O_NONBLOCK == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "descriptor must be nonblocking",
            ));
        }
        let id = core.allocate()?;
        let registered = if regular {
            false
        } else {
            let result = core
                .registry
                .borrow()
                .as_ref()
                .ok_or_else(invalid)?
                .register(
                    &mut SourceFd(&raw),
                    mio::Token(id),
                    mio::Interest::READABLE | mio::Interest::WRITABLE,
                );
            match result {
                Ok(()) => true,
                // epoll rejects immediate-I/O devices such as /dev/null.
                Err(error) if error.raw_os_error() == Some(libc::EPERM) => false,
                Err(error) => return Err(error),
            }
        };
        let state = Rc::new(IoState {
            core: Rc::downgrade(core),
            id,
            raw,
            fd: RefCell::new(Some(fd)),
            registered,
            read: DirectionState::default(),
            write: DirectionState::default(),
        });
        core.io.borrow_mut().insert(id, Rc::downgrade(&state));
        core.fds.borrow_mut().insert(raw, id);
        Ok(Self { state })
    }

    fn probe(&self, read: bool, write: bool) -> io::Result<(bool, bool)> {
        self.state.check()?;
        let mut pfd = libc::pollfd {
            fd: self.state.raw(),
            events: (if read { libc::POLLIN } else { 0 }) | (if write { libc::POLLOUT } else { 0 }),
            revents: 0,
        };
        // SAFETY: one initialized pollfd; zero timeout never blocks.
        let result = unsafe { libc::poll(&mut pfd, 1, 0) };
        if result < 0 {
            return Err(io::Error::last_os_error());
        }
        if pfd.revents & libc::POLLNVAL != 0 {
            return Err(io::Error::from_raw_os_error(libc::EBADF));
        }
        let terminal = pfd.revents & (libc::POLLERR | libc::POLLHUP) != 0;
        let ready = (
            read && (terminal || pfd.revents & libc::POLLIN != 0),
            write && (terminal || pfd.revents & libc::POLLOUT != 0),
        );
        if ready.0 || ready.1 {
            Ok(ready)
        } else {
            Err(io::ErrorKind::WouldBlock.into())
        }
    }
}

impl crate::AsyncFd for Io {
    /// Wait for requested directions. The returned pair is (readable, writable).
    /// Cancellation consumes no bytes. One waiter per direction is permitted.
    async fn ready(&self, read: bool, write: bool) -> io::Result<(bool, bool)> {
        if !read && !write {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        match self.probe(read, write) {
            Err(e)
                if e.kind() == io::ErrorKind::WouldBlock
                    || e.kind() == io::ErrorKind::Interrupted => {}
            result => return result,
        }
        let mut reader = pin!(self.read_with(|| self.probe(true, false)));
        let mut writer = pin!(self.write_with(|| self.probe(false, true)));
        poll_fn(|cx| {
            if read && let Poll::Ready(result) = reader.as_mut().poll(cx) {
                return Poll::Ready(result);
            }
            if write && let Poll::Ready(result) = writer.as_mut().poll(cx) {
                return Poll::Ready(result);
            }
            Poll::Pending
        })
        .await
    }
}

impl crate::AsyncRead for Io {
    async fn read(&self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        self.read_with(|| {
            // SAFETY: readiness validation ensures the descriptor lease is live;
            // buffer is exclusively borrowed writable storage.
            let count = unsafe {
                libc::read(
                    self.state.raw(),
                    buffer.as_mut_ptr().cast(),
                    buffer.len().min(isize::MAX as usize),
                )
            };
            syscall_result(count)
        })
        .await
    }
}

impl crate::AsyncWrite for Io {
    async fn write(&self, buffer: &[u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        self.write_with(|| {
            // SAFETY: readiness validation ensures the descriptor lease is live;
            // buffer remains readable throughout the syscall.
            let count = unsafe {
                libc::write(
                    self.state.raw(),
                    buffer.as_ptr().cast(),
                    buffer.len().min(isize::MAX as usize),
                )
            };
            syscall_result(count)
        })
        .await
    }
}

impl Io {
    // Internal adapter hook. The closure performs one bounded operation,
    // nonblocking except for the temporary regular-file path, and reports
    // WouldBlock only if no progress was made.
    // Invoke business callbacks after awaiting, never inside the retry closure.
    pub(crate) fn read_with<T, F>(&self, operation: F) -> impl Future<Output = io::Result<T>>
    where
        F: FnMut() -> io::Result<T>,
    {
        self.operation(Direction::Read, operation)
    }

    pub(crate) fn write_with<T, F>(&self, operation: F) -> impl Future<Output = io::Result<T>>
    where
        F: FnMut() -> io::Result<T>,
    {
        self.operation(Direction::Write, operation)
    }
}

fn syscall_result(count: isize) -> io::Result<usize> {
    if count < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(count as usize)
    }
}

impl Io {
    fn operation<F>(&self, direction: Direction, operation: F) -> Operation<'_, F> {
        Operation {
            source: self,
            direction,
            id: None,
            operation,
        }
    }
}

struct Operation<'a, F> {
    source: &'a Io,
    direction: Direction,
    id: Option<usize>,
    operation: F,
}

// The closure is never structurally pinned or exposed through a pinned reference.
impl<F> Unpin for Operation<'_, F> {}

impl<T, F: FnMut() -> io::Result<T>> Future for Operation<'_, F> {
    type Output = io::Result<T>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let generation = match this
            .source
            .state
            .poll_ready(this.direction, &mut this.id, context)
        {
            Poll::Ready(result) => result?.0,
            Poll::Pending => return Poll::Pending,
        };
        // No runtime/state borrow crosses the caller's syscall closure.
        match (this.operation)() {
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                if !this.source.state.registered {
                    this.source
                        .state
                        .remove_waiter(this.direction, this.id.take());
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "descriptor cannot wait for readiness",
                    )));
                }
                this.source.state.clear(this.direction, generation);
                // Register before returning Pending. A newer event may have
                // survived the clear; schedule another poll in that case.
                match this
                    .source
                    .state
                    .poll_ready(this.direction, &mut this.id, context)
                {
                    Poll::Ready(Err(error)) => return Poll::Ready(Err(error)),
                    Poll::Ready(Ok(_)) => context.local_waker().wake_by_ref(),
                    Poll::Pending => {}
                }
                Poll::Pending
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {
                // Bound retries to one operation per poll, retaining readiness.
                context.local_waker().wake_by_ref();
                Poll::Pending
            }
            result => {
                this.source
                    .state
                    .remove_waiter(this.direction, this.id.take());
                Poll::Ready(result)
            }
        }
    }
}

impl<F> Drop for Operation<'_, F> {
    fn drop(&mut self) {
        self.source
            .state
            .remove_waiter(self.direction, self.id.take());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Handle as _, Runtime as _};
    use std::io::Write;
    use std::os::unix::net::UnixStream;
    use std::task::Waker;
    use std::time::Duration;

    #[test]
    fn unregistered_operation_cannot_wait_and_releases_its_waiter() {
        let mut runtime = super::super::Runtime::new().unwrap();
        let file = std::fs::File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).unwrap();
        let source = runtime.handle().io(file.into()).unwrap();
        let mut operation = source.read_with(|| Err::<(), _>(io::ErrorKind::WouldBlock.into()));
        assert!(
            matches!(poll(&mut operation), Poll::Ready(Err(e)) if e.kind() == io::ErrorKind::Unsupported)
        );
        assert!(matches!(
            poll(&mut source.read_with(|| Ok(42))),
            Poll::Ready(Ok(42))
        ));
        drop(operation);
        drop(source);
        // Bypassed descriptors must never be deregistered from mio.
        tick(&mut runtime);
    }

    #[test]
    fn acknowledging_an_old_generation_preserves_a_new_notification() {
        let mut runtime = super::super::Runtime::new().unwrap();
        let (mut writer, reader) = UnixStream::pair().unwrap();
        reader.set_nonblocking(true).unwrap();
        let source = runtime.handle().io(reader.into()).unwrap();
        writer.write_all(b"first").unwrap();
        runtime.poll(Some(Duration::ZERO)).unwrap();
        let old = source.state.read.generation.get();
        let mut bytes = [0; 16];
        // SAFETY: source leases the descriptor and bytes is writable storage.
        assert_eq!(
            unsafe { libc::read(source.state.raw(), bytes.as_mut_ptr().cast(), bytes.len()) },
            5
        );
        writer.write_all(b"new").unwrap();
        runtime.poll(Some(Duration::ZERO)).unwrap();
        assert!(source.state.read.generation.get() > old);
        source.state.clear(Direction::Read, old);
        let mut operation = source.read_with(|| Ok(42));
        assert!(matches!(
            Pin::new(&mut operation).poll(&mut Context::from_waker(Waker::noop())),
            Poll::Ready(Ok(42))
        ));
    }

    fn tick(runtime: &mut super::super::Runtime) {
        runtime.poll(Some(Duration::ZERO)).unwrap();
    }
    fn poll<F: Future + Unpin>(future: &mut F) -> Poll<F::Output> {
        Pin::new(future).poll(&mut Context::from_waker(Waker::noop()))
    }
    fn pair() -> (UnixStream, OwnedFd) {
        let (writer, reader) = UnixStream::pair().unwrap();
        reader.set_nonblocking(true).unwrap();
        (writer, reader.into())
    }
    #[test]
    fn interrupted_operations_yield_keep_the_waiter_and_return_errors() {
        let mut runtime = super::super::Runtime::new().unwrap();
        let (_writer, fd) = pair();
        let source = runtime.handle().io(fd).unwrap();
        tick(&mut runtime);
        let calls = Cell::new(0);
        let mut operation = source.write_with(|| {
            calls.set(calls.get() + 1);
            match calls.get() {
                1 => Err(io::Error::from(io::ErrorKind::Interrupted)),
                _ => Err::<(), _>(io::Error::from(io::ErrorKind::PermissionDenied)),
            }
        });
        assert_eq!(calls.get(), 0);
        assert!(poll(&mut operation).is_pending());
        assert_eq!(calls.get(), 1);
        assert!(
            matches!(poll(&mut source.write_with(|| Ok(()))), Poll::Ready(Err(e)) if e.kind() == io::ErrorKind::AlreadyExists)
        );
        assert!(
            matches!(poll(&mut operation), Poll::Ready(Err(e)) if e.kind() == io::ErrorKind::PermissionDenied)
        );
        assert_eq!(calls.get(), 2);
        assert!(matches!(
            poll(&mut source.write_with(|| Ok(42))),
            Poll::Ready(Ok(42))
        ));
    }

    #[test]
    fn accepts_connections_through_the_same_operation_api() {
        use std::os::unix::net::UnixListener;
        let mut runtime = super::super::Runtime::new().unwrap();
        let socket_path =
            std::env::temp_dir().join(format!("hmux-rt-accept-{}", std::process::id()));
        let listener = UnixListener::bind(&socket_path).unwrap();
        // Unix sockets remain connectable through the bound pathname until unlink.
        listener.set_nonblocking(true).unwrap();
        let lease = OwnedFd::from(listener.try_clone().unwrap());
        let source = runtime.handle().io(lease).unwrap();
        let mut accept = source.read_with(|| listener.accept());
        assert!(poll(&mut accept).is_pending());
        let client = UnixStream::connect(&socket_path).unwrap();
        std::fs::remove_file(&socket_path).unwrap();
        tick(&mut runtime);
        assert!(matches!(
            poll(&mut accept),
            Poll::Ready(Ok((_connection, _address)))
        ));
        drop(client);
    }
}
