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

/// The only owner allowed to drive local tasks and perform lifecycle cleanup.
pub trait Runtime: Sized + 'static {
    /// The capability for creating work on this runtime instance.
    type Handle: Handle;

    /// Create an independent executor and poller on the calling thread.
    fn new() -> io::Result<Self>;

    /// Obtain a capability bound to this instance and process generation.
    fn handle(&self) -> Self::Handle;

    /// poll tasks up to max duration. Will wait forever if max_wait is None.
    fn poll(&mut self, max_wait: Option<Duration>) -> io::Result<()>;

    /// Replace an inherited executor/poller in a single-threaded fork child.
    fn reset_after_fork(&mut self) -> io::Result<()>;
}

/// A cloneable capability to create local work
pub trait Handle: Clone + 'static {
    /// Owns a spawned future; dropping it cancels the work.
    type Task: 'static;

    /// Leased descriptor supporting async reads and writes.
    type Io: AsyncRead + AsyncWrite + 'static;

    /// Signal subscription.
    type Signals: Signals + 'static;

    /// Monotonic deadline wait.
    type Sleep: Future<Output = io::Result<()>> + 'static;

    /// Schedule a non-Send future, without polling it inline. The returned task
    /// owns the future and must be retained until completion or cancellation.
    fn spawn<F>(&self, future: F) -> io::Result<Self::Task>
    where
        F: Future<Output = ()> + 'static;

    /// Lease a nonblocking byte-stream descriptor.
    fn io(&self, fd: Rc<OwnedFd>) -> io::Result<Self::Io>;

    /// Subscribe to a nonempty set of valid, catchable signal numbers.
    fn signals(&self, set: &[c_int]) -> io::Result<Self::Signals>;

    /// Wait for one absolute monotonic deadline; drop cancels the wait.
    fn sleep_until(&self, deadline: Instant) -> Self::Sleep;
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
