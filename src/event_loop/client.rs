//! One readiness-driven imsg connection actor.

use std::collections::VecDeque;
use std::io;
use std::os::fd::{AsFd, BorrowedFd};

use crate::tmux::codec::{ImsgReader, NonblockingImsgWriter};
use crate::tmux::message::Frame;
use crate::tmux::traits::{NonblockingFrameReader, NonblockingFrameWriter};

use super::actor::ActorRef;
use super::driver::{Envelope, Outbox};
use super::reactor::Token;

/// Maximum decoded frames handled by one read event.
pub(crate) const READ_FRAME_BUDGET: usize = 32;

/// Why the connection actor stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CloseReason {
    PeerClosed,
    Error(io::ErrorKind),
    Shutdown,
    FrameExceedsQueueLimit,
    BackpressureViolation,
}

/// Events delivered to the connection actor.
pub(crate) enum ClientIoEvent {
    Start,
    Readable,
    ReadContinuation,
    Writable,
    Send(Frame),
    Shutdown,
}

/// Events delivered by the I/O actor to its current domain placeholder.
pub(crate) enum ClientInboxEvent {
    Frame(Frame),
    Closed(CloseReason),
}

/// Minimal domain-side actor used while the real client actor is migrated.
#[derive(Default)]
pub(crate) struct ClientInbox {
    frames: VecDeque<Frame>,
    close_reason: Option<CloseReason>,
}

impl ClientInbox {
    pub(crate) fn len(&self) -> usize {
        self.frames.len()
    }

    pub(crate) fn pop_frame(&mut self) -> Option<Frame> {
        self.frames.pop_front()
    }

    pub(crate) fn close_reason(&self) -> Option<CloseReason> {
        self.close_reason
    }

    fn handle(&mut self, event: ClientInboxEvent) {
        match event {
            ClientInboxEvent::Frame(frame) => self.frames.push_back(frame),
            ClientInboxEvent::Closed(reason) => self.close_reason = Some(reason),
        }
    }
}

/// Read/write state for one client connection.
pub(crate) struct ClientIo {
    reader: ImsgReader,
    writer: NonblockingImsgWriter,
    inbox: ActorRef<ClientInbox>,
    retry: Option<Frame>,
    read_token: Option<Token>,
    write_token: Option<Token>,
    read_work_queued: bool,
    write_work_queued: bool,
    send_work_queued: bool,
    reads_paused: bool,
    closed: bool,
}

impl ClientIo {
    pub(crate) fn new(
        reader: ImsgReader,
        writer: NonblockingImsgWriter,
        inbox: ActorRef<ClientInbox>,
    ) -> Self {
        Self {
            reader,
            writer,
            inbox,
            retry: None,
            read_token: None,
            write_token: None,
            read_work_queued: false,
            write_work_queued: false,
            send_work_queued: false,
            reads_paused: false,
            closed: false,
        }
    }

    pub(crate) fn read_fd(&self) -> BorrowedFd<'_> {
        self.reader.as_fd()
    }

    pub(crate) fn write_fd(&self) -> BorrowedFd<'_> {
        self.writer.as_fd()
    }

    pub(crate) fn read_token(&self) -> Option<Token> {
        self.read_token
    }

    pub(crate) fn write_token(&self) -> Option<Token> {
        self.write_token
    }

    pub(crate) fn set_read_token(&mut self, token: Option<Token>) {
        self.read_token = token;
    }

    pub(crate) fn set_write_token(&mut self, token: Option<Token>) {
        self.write_token = token;
    }

    pub(crate) fn mark_read_work_queued(&mut self) -> bool {
        if self.closed || self.reads_paused || self.read_work_queued {
            return false;
        }
        self.read_work_queued = true;
        true
    }

    pub(crate) fn mark_write_work_queued(&mut self) -> bool {
        if self.closed || self.write_work_queued {
            return false;
        }
        self.write_work_queued = true;
        true
    }

    pub(crate) fn mark_send_work_queued(&mut self) -> bool {
        if self.closed || self.retry.is_some() || self.send_work_queued {
            return false;
        }
        self.send_work_queued = true;
        true
    }

    pub(crate) fn reads_paused(&self) -> bool {
        self.reads_paused
    }

    pub(crate) fn has_retry(&self) -> bool {
        self.retry.is_some()
    }

    pub(crate) fn handle(
        &mut self,
        target: &ActorRef<Self>,
        event: ClientIoEvent,
        outbox: &mut Outbox,
    ) {
        if self.closed {
            return;
        }

        match event {
            ClientIoEvent::Start => outbox.set_read_interest(target.clone(), true),
            ClientIoEvent::Readable => {
                self.read_work_queued = false;
                self.handle_readable(target, outbox);
            }
            ClientIoEvent::ReadContinuation => {
                self.read_work_queued = false;
                self.handle_readable(target, outbox);
            }
            ClientIoEvent::Writable => {
                self.write_work_queued = false;
                self.handle_writable(target, outbox);
            }
            ClientIoEvent::Send(frame) => {
                self.send_work_queued = false;
                self.handle_send(target, frame, outbox);
            }
            ClientIoEvent::Shutdown => self.close(target, CloseReason::Shutdown, outbox),
        }
    }

    fn handle_readable(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        if self.reads_paused {
            return;
        }

        for _ in 0..READ_FRAME_BUDGET {
            match NonblockingFrameReader::try_recv(&mut self.reader) {
                Ok(frame) => outbox.enqueue(Envelope::ClientInbox {
                    target: self.inbox.clone(),
                    event: ClientInboxEvent::Frame(frame),
                }),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => return,
                Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
                    self.close(target, CloseReason::PeerClosed, outbox);
                    return;
                }
                Err(error) => {
                    self.close(target, CloseReason::Error(error.kind()), outbox);
                    return;
                }
            }
        }

        if self.reader.has_buffered_frame() {
            self.schedule_read_continuation(target, outbox);
        }
    }

    fn handle_send(&mut self, target: &ActorRef<Self>, frame: Frame, outbox: &mut Outbox) {
        if self.retry.is_some() {
            self.close(target, CloseReason::BackpressureViolation, outbox);
            return;
        }

        match self.writer.try_queue(frame) {
            Ok(()) => outbox.set_write_interest(target.clone(), true),
            Err(error) if self.writer.has_pending() => {
                self.retry = Some(error.into_frame());
                self.reads_paused = true;
                outbox.set_read_interest(target.clone(), false);
                outbox.set_write_interest(target.clone(), true);
            }
            Err(_) => self.close(target, CloseReason::FrameExceedsQueueLimit, outbox),
        }
    }

    fn handle_writable(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        match self.writer.try_flush() {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            Err(error) => {
                self.close(target, CloseReason::Error(error.kind()), outbox);
                return;
            }
        }

        let mut accepted_retry = false;
        if let Some(frame) = self.retry.take() {
            match self.writer.try_queue(frame) {
                Ok(()) => {
                    accepted_retry = true;
                    self.reads_paused = false;
                    outbox.set_read_interest(target.clone(), true);
                    if self.reader.has_buffered_frame() {
                        self.schedule_read_continuation(target, outbox);
                    }
                }
                Err(error) if self.writer.has_pending() => {
                    self.retry = Some(error.into_frame());
                }
                Err(_) => {
                    self.close(target, CloseReason::FrameExceedsQueueLimit, outbox);
                    return;
                }
            }
        }

        // Mio readiness is edge-triggered. If the socket stayed writable while
        // the retry moved into the private queue, no second writable edge is
        // guaranteed, so advance it during the current event.
        if accepted_retry {
            match self.writer.try_flush() {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => {
                    self.close(target, CloseReason::Error(error.kind()), outbox);
                    return;
                }
            }
        }

        outbox.set_write_interest(target.clone(), self.writer.has_pending());
    }

    fn schedule_read_continuation(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        if self.mark_read_work_queued() {
            outbox.enqueue(Envelope::ClientIo {
                target: target.clone(),
                event: ClientIoEvent::ReadContinuation,
            });
        }
    }

    fn close(&mut self, target: &ActorRef<Self>, reason: CloseReason, outbox: &mut Outbox) {
        self.closed = true;
        self.retry = None;
        outbox.set_read_interest(target.clone(), false);
        outbox.set_write_interest(target.clone(), false);
        outbox.enqueue(Envelope::ClientInbox {
            target: self.inbox.clone(),
            event: ClientInboxEvent::Closed(reason),
        });
        outbox.stop_client(target.clone());
    }
}

pub(crate) fn dispatch_inbox(target: &ActorRef<ClientInbox>, event: ClientInboxEvent) {
    target.with_mut(|inbox| inbox.handle(event));
}
