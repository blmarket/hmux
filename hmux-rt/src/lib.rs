//! library entrypoint
#![feature(local_waker)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod mio;
pub mod stream;

use std::ffi::c_int;
use std::future::Future;
use std::io;
use std::os::fd::OwnedFd;
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

    /// Owned descriptor supporting readiness waits and async byte-stream I/O.
    type Io: AsyncFd + AsyncRead + AsyncWrite + 'static;

    /// Signal subscription.
    type Signals: Signals + 'static;

    /// Monotonic deadline wait.
    type Sleep: Future<Output = io::Result<()>> + 'static;

    /// Schedule a local-waker future, without polling it inline. Ordinary Waker
    /// notifications are inert. The returned task
    /// owns the future and must be retained until completion or cancellation.
    fn spawn<F>(&self, future: F) -> io::Result<Self::Task>
    where
        F: Future<Output = ()> + 'static;

    /// Take ownership of a descriptor for readiness waits and byte-stream I/O.
    /// Non-file descriptors must be nonblocking. Regular files may perform
    /// synchronous I/O on the runtime thread; they bypass the readiness poller.
    /// Construction errors close the fd.
    fn io(&self, fd: OwnedFd) -> io::Result<Self::Io>;

    /// Subscribe to a nonempty set of valid, catchable signal numbers.
    fn signals(&self, set: &[c_int]) -> io::Result<Self::Signals>;

    /// Wait for one absolute monotonic deadline; drop cancels the wait.
    fn sleep_until(&self, deadline: Instant) -> Self::Sleep;
}

/// An owned descriptor with local-waker readiness waits.
///
/// Created through [`Handle::io`]. The descriptor is bound to its runtime, so
/// waiting needs no runtime handle. Dropping the registration deregisters and
/// closes it; dropping the runtime closes registered descriptors and invalidates
/// subsequent waits.
///
/// Readiness consumes no data. The caller must perform nonblocking I/O and
/// handle `WouldBlock` by waiting again. Only one pending waiter per direction
/// is supported, shared with byte reads/writes on the same registration.
pub trait AsyncFd {
    /// Wait for either requested direction, returning (readable, writable).
    /// Requesting neither direction returns `InvalidInput`. Dropping a pending
    /// wait unregisters its waiter without consuming input or output capacity.
    fn ready(&self, read: bool, write: bool)
    -> impl Future<Output = io::Result<(bool, bool)>> + '_;
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
