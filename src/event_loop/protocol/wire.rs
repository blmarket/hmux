//! The socket one protocol client speaks over: frames in, frames out.
//!
//! Both directions are registered for the life of the connection and waited on
//! as futures, so the pauses a client used to describe to a central loop —
//! "stop reading, I am above high water", "tell me when the socket takes more"
//! — are now just where its own code awaits. Reading always services the write
//! side too: a peer that is waiting on our answer before it says anything else
//! would otherwise never speak.

use std::io;
use std::os::fd::AsFd;

use crate::sync::{select, Either};
use crate::tmux::codec::{ImsgReader, NonblockingImsgWriter};
use crate::tmux::introspect::{log_frame, Direction};
use crate::tmux::message::Frame;
use crate::tmux::traits::NonblockingFrameWriter;
use hmux_rt::{AsyncFd, Interest, TaskHandle};

use super::ProtocolCloseReason;

/// One connection's read and write halves with their readiness registrations.
pub(super) struct Wire {
    reader: ImsgReader,
    writer: NonblockingImsgWriter,
    readable: AsyncFd,
    writable: AsyncFd,
}

impl Wire {
    /// Register both halves. Must be called from inside the task that owns the
    /// connection, which is what a descriptor registration belongs to.
    pub(super) fn new(
        tasks: &TaskHandle,
        reader: ImsgReader,
        writer: NonblockingImsgWriter,
    ) -> io::Result<Self> {
        let readable = AsyncFd::new(tasks, reader.as_fd(), Interest::READABLE)?;
        let writable = AsyncFd::new(tasks, writer.as_fd(), Interest::WRITABLE)?;
        Ok(Self {
            reader,
            writer,
            readable,
            writable,
        })
    }

    /// The next frame from the peer, waiting for one.
    pub(super) async fn recv(&mut self) -> Result<Frame, ProtocolCloseReason> {
        loop {
            match self.try_recv() {
                Ok(Some(frame)) => return Ok(frame),
                Ok(None) => {}
                Err(reason) => return Err(reason),
            }
            self.wait_readable().await?;
        }
    }

    /// The next frame if one is already buffered or on the socket.
    pub(super) fn try_recv(&mut self) -> Result<Option<Frame>, ProtocolCloseReason> {
        match self.reader.try_recv() {
            Ok(frame) => {
                log_frame(Direction::ClientToServer, &frame);
                Ok(Some(frame))
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
                Err(ProtocolCloseReason::PeerClosed)
            }
            Err(error) => Err(ProtocolCloseReason::Error(error.kind())),
        }
    }

    /// Wait for the socket to have more to read, flushing queued output while
    /// waiting.
    pub(super) async fn wait_readable(&mut self) -> Result<(), ProtocolCloseReason> {
        let Self {
            writer,
            readable,
            writable,
            ..
        } = self;
        loop {
            if !writer.has_pending() {
                readable.readiness().await;
                return Ok(());
            }
            match select(readable.readiness(), writable.readiness()).await {
                Either::First(_) => return Ok(()),
                Either::Second(_) => flush(writer)?,
            }
        }
    }

    /// The readiness leaf for the read side, for a client that waits on the
    /// socket alongside sources of its own.
    ///
    /// A caller racing this owes the write side the same service `recv` gives
    /// it, which is what [`Wire::writable`] is for.
    pub(super) fn readable(&self) -> impl std::future::Future<Output = ()> + '_ {
        let readable = &self.readable;
        async move {
            readable.readiness().await;
        }
    }

    /// The readiness leaf for the write side, armed only while output is
    /// queued: a socket with nothing to send never becomes interesting.
    pub(super) fn writable(&self) -> impl std::future::Future<Output = ()> + '_ {
        let armed = self.writer.has_pending();
        let writable = &self.writable;
        async move {
            if !armed {
                std::future::pending::<()>().await;
            }
            writable.readiness().await;
        }
    }

    /// Hand the socket whatever it will take right now.
    pub(super) fn flush(&mut self) -> Result<(), ProtocolCloseReason> {
        flush(&mut self.writer)
    }

    /// Queue one frame for the peer.
    ///
    /// A frame that does not fit the connection's private queue is fatal to it:
    /// there is no larger queue to wait for, so the client is closed rather
    /// than left with a hole in its output.
    pub(super) fn queue(&mut self, frame: Frame) -> Result<(), ProtocolCloseReason> {
        log_frame(Direction::ServerToClient, &frame);
        self.writer
            .try_queue(frame)
            .map_err(|_| ProtocolCloseReason::FrameExceedsQueueLimit)
    }

    /// Queue one frame, waiting first for room below the high-water mark, and
    /// hand it to the socket.
    ///
    /// This is the backpressure that used to be a paused read interest: a
    /// producer that outruns the peer parks here instead of queueing without
    /// bound, and parks before the frame rather than after it.
    pub(super) async fn send(&mut self, frame: Frame) -> Result<(), ProtocolCloseReason> {
        self.wait_below_high_water().await?;
        self.queue(frame)?;
        self.flush()
    }

    /// Wait until one more frame may be queued.
    pub(super) async fn wait_below_high_water(&mut self) -> Result<(), ProtocolCloseReason> {
        loop {
            flush(&mut self.writer)?;
            if self.writer.is_below_high_water() {
                return Ok(());
            }
            self.writable.readiness().await;
        }
    }

    /// Push everything queued out, waiting as long as the peer takes.
    pub(super) async fn drain(&mut self) -> Result<(), ProtocolCloseReason> {
        loop {
            flush(&mut self.writer)?;
            if !self.writer.has_pending() {
                return Ok(());
            }
            self.writable.readiness().await;
        }
    }

    pub(super) fn is_below_high_water(&self) -> bool {
        self.writer.is_below_high_water()
    }
}

/// A flush that treats a full socket as ordinary: the bytes stay queued and
/// the write side stays armed.
fn flush(writer: &mut NonblockingImsgWriter) -> Result<(), ProtocolCloseReason> {
    match writer.try_flush() {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => Ok(()),
        Err(error) => Err(ProtocolCloseReason::Error(error.kind())),
    }
}
