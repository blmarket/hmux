//! The swappable server abstraction — **definitions only** (per design.md and
//! prompt.md). Message types and codecs live beside this in sibling modules.
//!
//! The connection is split into independent read and write halves so the
//! two-thread-per-connection model (client→server and server→client) can hold one
//! handle each. A single `&mut self` connection, or one behind a mutex, would
//! deadlock: a blocking `recv` on the read side would hold the lock the write
//! side needs.
//!
//! Tests in the `hmux-conformance` workspace crate are generic over
//! [`TmuxServer`] so the same suite can validate native hmux against a direct
//! stock tmux reference.

use std::{error, fmt, io};

use super::message::Frame;

/// A nonblocking writer rejected a frame because its private queue is full.
pub struct WriteQueueFull<F> {
    frame: F,
}

impl<F> WriteQueueFull<F> {
    /// Construct a queue-full error that returns ownership of the rejected
    /// frame.
    pub fn new(frame: F) -> Self {
        Self { frame }
    }

    /// Borrow the rejected frame.
    pub fn frame(&self) -> &F {
        &self.frame
    }

    /// Recover ownership of the rejected frame.
    pub fn into_frame(self) -> F {
        self.frame
    }
}

impl<F> fmt::Debug for WriteQueueFull<F> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("WriteQueueFull")
    }
}

impl<F> fmt::Display for WriteQueueFull<F> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("nonblocking writer queue is full")
    }
}

impl<F> error::Error for WriteQueueFull<F> {}

/// A tmux control-plane server: something a client can connect to and exchange
/// [`Frame`]s with.
///
/// A server and the halves it hands out belong to one thread: the connection
/// they describe is driven by a single readiness loop. Callers that do need to
/// move one across threads bound it themselves.
pub trait TmuxServer {
    type Reader: FrameReader;
    type Writer: FrameWriter;

    /// Open a fresh client connection, returning its read and write halves.
    fn connect(&self) -> io::Result<(Self::Reader, Self::Writer)>;
}

/// The receiving half of a connection.
pub trait FrameReader {
    /// Block until the next frame arrives. Returns an `UnexpectedEof` error when
    /// the peer closes at a frame boundary.
    fn recv(&mut self) -> io::Result<Frame>;
}

/// A receiving half that can be driven by an I/O readiness loop.
///
/// This trait is intentionally independent of [`FrameReader`]. Event-driven
/// users should not need to implement or depend on the legacy blocking
/// operation.
pub trait NonblockingFrameReader {
    /// Return the next complete frame without blocking.
    ///
    /// Returns [`io::ErrorKind::WouldBlock`] when no complete frame is
    /// currently available. Any partial frame bytes and received descriptors
    /// are retained for a later call.
    fn try_recv(&mut self) -> io::Result<Frame>;
}

/// A sending half that can be driven by an I/O readiness loop.
///
/// Implementations choose their input frame type and privately own all queued
/// and partial-write state.
pub trait NonblockingFrameWriter {
    /// One owned logical value accepted by the writer.
    type Frame;

    /// Attempt to append one frame to the implementation's private output
    /// buffer.
    ///
    /// This operation must not perform I/O or block. On queue exhaustion, the
    /// error returns ownership of the unconsumed frame so the caller can apply
    /// backpressure and retry it after flushing.
    fn try_queue(&mut self, frame: Self::Frame) -> Result<(), WriteQueueFull<Self::Frame>>;

    /// Advance queued output without blocking.
    ///
    /// `Ok(())` means all queued output was sent. `WouldBlock` means the writer
    /// retained the unwritten suffix and should be retried after writable
    /// readiness.
    fn try_flush(&mut self) -> io::Result<()>;

    /// Whether any output remains queued.
    fn has_pending(&self) -> bool;
}

/// The sending half of a connection.
pub trait FrameWriter {
    /// Send a frame, forwarding any attached `SCM_RIGHTS` fd.
    fn send(&mut self, frame: Frame) -> io::Result<()>;
}
