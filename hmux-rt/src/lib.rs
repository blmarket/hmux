//! library entrypoint
#![feature(local_waker)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod mio;
pub mod stream;

use std::ffi::c_int;
use std::future::Future;
use std::io;
use std::os::fd::OwnedFd;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Owns and drives the runtime. Polling borrows the owner through callback
/// execution, so the owner cannot be dropped until polling returns.
pub trait Runtime: Sized + 'static {
    /// The capability for creating work on this runtime instance.
    type Handle: Handle;

    /// Create an independent executor and poller on the calling thread.
    fn new() -> io::Result<Self>;

    /// Obtain a capability bound to this instance and process generation.
    fn handle(&self) -> Self::Handle;

    /// Poll tasks, waiting at most max_wait for readiness. Callbacks schedule
    /// through handles, without borrowing the runtime again. The owner remains
    /// exclusively borrowed until all callbacks in this turn have returned.
    fn poll(&mut self, max_wait: Option<Duration>) -> io::Result<()>;
}

/// A cloneable, thread-local capability to schedule work.
/// A handle does not keep its runtime owner alive.
pub trait Handle: Clone + 'static {
    /// Owns a spawned future; dropping it cancels the work.
    type Task: 'static;

    /// Leased descriptor supporting async reads and writes.
    type Io: AsyncFd + AsyncRead + AsyncWrite + 'static;

    /// Readiness bridge for nonblocking consumers, including descriptors that
    /// the OS poller cannot register (such as regular files).
    type Descriptor: AsyncFd + 'static;

    /// Signal subscription.
    type Signals: Signals + 'static;

    /// Monotonic deadline wait.
    type Sleep: Future<Output = io::Result<()>> + 'static;

    /// Whether a task is unfinished and its runtime is still usable.
    fn task_is_pending(task: &Self::Task) -> bool;

    /// Schedule a local-waker future, without polling it inline. Ordinary Waker
    /// notifications are inert. The returned task
    /// owns the future and must be retained until completion or cancellation.
    fn spawn<F>(&self, future: F) -> io::Result<Self::Task>
    where
        F: Future<Output = ()> + 'static;

    /// Lease a nonblocking byte-stream descriptor.
    fn io(&self, fd: Rc<OwnedFd>) -> io::Result<Self::Io>;

    /// Lease a descriptor for non-consuming readiness waits. Regular files
    /// bypass the poller; their actual disk I/O can still block the thread.
    fn descriptor(&self, fd: Rc<OwnedFd>) -> io::Result<Self::Descriptor>;

    /// Subscribe to a nonempty set of valid, catchable signal numbers.
    fn signals(&self, set: &[c_int]) -> io::Result<Self::Signals>;

    /// Wait for one absolute monotonic deadline; drop cancels the wait.
    fn sleep_until(&self, deadline: Instant) -> Self::Sleep;
}

/// A registered nonblocking descriptor with local-waker readiness waits.
///
/// Created through [`Handle::io`]. The descriptor is bound to its runtime, so
/// waiting needs no runtime handle. Dropping the registration deregisters it;
/// dropping the runtime invalidates subsequent waits.
///
/// Readiness consumes no data. The caller must perform nonblocking I/O and
/// handle `WouldBlock` by waiting again. Only one pending waiter per direction
/// is supported, shared with byte reads/writes on the same registration.
///
/// ```
/// use hmux_rt::{AsyncFd, Handle, Runtime, mio};
/// use std::io::{self, Read, Write};
/// use std::os::fd::OwnedFd;
/// use std::os::unix::net::UnixStream;
/// use std::rc::Rc;
/// use std::time::Duration;
///
/// let mut runtime = mio::Runtime::new()?;
/// let (mut reader, mut writer) = UnixStream::pair()?;
/// reader.set_nonblocking(true)?;
/// let lease = Rc::new(OwnedFd::from(reader.try_clone()?));
/// let fd = runtime.handle().io(lease)?;
/// let task = runtime.handle().spawn(async move {
///     let mut byte = [0];
///     loop {
///         fd.readable().await.unwrap();
///         match reader.read(&mut byte) {
///             Err(error) if error.kind() == io::ErrorKind::WouldBlock => continue,
///             result => { assert_eq!(result.unwrap(), 1); break; }
///         }
///     }
/// })?;
/// writer.write_all(b"x")?;
/// runtime.poll(Some(Duration::ZERO))?;
/// drop(task);
/// # Ok::<(), io::Error>(())
/// ```
pub trait AsyncFd {
    /// Wait for either requested direction, returning (readable, writable).
    /// Requesting neither direction returns `InvalidInput`. Dropping a pending
    /// wait unregisters its waiter without consuming input or output capacity.
    fn ready(&self, read: bool, write: bool)
    -> impl Future<Output = io::Result<(bool, bool)>> + '_;

    /// Wait until a nonblocking read-side operation may make progress.
    fn readable(&self) -> impl Future<Output = io::Result<()>> + '_ {
        async move { self.ready(true, false).await.map(|_| ()) }
    }

    /// Wait until a nonblocking write-side operation may make progress.
    fn writable(&self) -> impl Future<Output = io::Result<()>> + '_ {
        async move { self.ready(false, true).await.map(|_| ()) }
    }
}

/// A local async byte reader.
pub trait AsyncRead {
    /// Read available bytes to buffer, return read bytes.
    fn read<'a>(&'a self, buffer: &'a mut [u8]) -> impl Future<Output = io::Result<usize>> + 'a;
}

/// A local async byte writer.
pub trait AsyncWrite {
    /// Write bytes, returning the number written.
    fn write<'a>(&'a self, buffer: &'a [u8]) -> impl Future<Output = io::Result<usize>> + 'a;
}

/// A local signal subscription
pub trait Signals {
    /// Cancellation-safe wait: dropping it cannot consume an undelivered signal.
    type Recv<'a>: Future<Output = io::Result<c_int>> + 'a
    where
        Self: 'a;

    /// Wait for a subscribed signal. The mutable borrow permits one receiver.
    fn recv(&mut self) -> Self::Recv<'_>;
}
