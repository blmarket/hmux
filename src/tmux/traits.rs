//! The swappable server abstraction — **definitions only** (per design.md and
//! prompt.md). Message types and codecs live beside this in sibling modules.
//!
//! The connection is split into independent read and write halves so the
//! two-thread-per-pairing model (client→server and server→client) can hold one
//! handle each. A single `&mut self` connection, or one behind a mutex, would
//! deadlock: a blocking `recv` on the read side would hold the lock the write
//! side needs.
//!
//! Tests in the `hmux-conformance` workspace crate are generic over
//! [`TmuxServer`] so the same suite can validate native hmux against a direct
//! stock tmux reference.

use std::io;

use super::message::Frame;

/// A tmux control-plane server: something a client can connect to and exchange
/// [`Frame`]s with.
pub trait TmuxServer: Send + Sync {
    type Reader: FrameReader;
    type Writer: FrameWriter;

    /// Open a fresh client connection, returning its read and write halves.
    fn connect(&self) -> io::Result<(Self::Reader, Self::Writer)>;
}

/// The receiving half of a connection.
pub trait FrameReader: Send {
    /// Block until the next frame arrives. Returns an `UnexpectedEof` error when
    /// the peer closes at a frame boundary.
    fn recv(&mut self) -> io::Result<Frame>;
}

/// A receiving half that can be driven by an I/O readiness loop.
///
/// This trait is intentionally independent of [`FrameReader`]. Event-driven
/// users should not need to implement or depend on the legacy blocking
/// operation.
pub trait NonblockingFrameReader: Send {
    /// Return the next complete frame without blocking.
    ///
    /// Returns [`io::ErrorKind::WouldBlock`] when no complete frame is
    /// currently available. Any partial frame bytes and received descriptors
    /// are retained for a later call.
    fn try_recv(&mut self) -> io::Result<Frame>;
}

/// The sending half of a connection.
pub trait FrameWriter: Send {
    /// Send a frame, forwarding any attached `SCM_RIGHTS` fd.
    fn send(&mut self, frame: Frame) -> io::Result<()>;
}
