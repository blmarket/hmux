//! Readiness-driven frame forwarding for one client/server pairing.

use std::cell::Cell;
use std::io;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
use std::rc::Rc;

use crate::common::reactor::Token;
use crate::tmux::codec::{ImsgReader, NonblockingImsgWriter};
use crate::tmux::introspect::{log_frame, Direction};
use crate::tmux::message::Frame;
use crate::tmux::traits::NonblockingFrameWriter;

use super::actor::ActorRef;
use super::client::READ_FRAME_BUDGET;
use super::driver::Outbox;

/// One physical endpoint of a proxied connection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PairEndpoint {
    Client,
    Server,
}

impl PairEndpoint {
    pub(crate) const ALL: [Self; 2] = [Self::Client, Self::Server];

    fn index(self) -> usize {
        match self {
            Self::Client => 0,
            Self::Server => 1,
        }
    }

    fn other(self) -> Self {
        match self {
            Self::Client => Self::Server,
            Self::Server => Self::Client,
        }
    }

    fn direction(self) -> Direction {
        match self {
            Self::Client => Direction::ClientToServer,
            Self::Server => Direction::ServerToClient,
        }
    }
}

/// Read or write registration on one pairing endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PairIoSide {
    Read,
    Write,
}

/// Why a pairing stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PairingCloseReason {
    PeerClosed,
    Error(io::ErrorKind),
    Shutdown,
    FrameExceedsQueueLimit,
}

/// Events delivered to a pairing actor.
pub(crate) enum PairingEvent {
    Start,
    Readable(PairEndpoint),
    ReadContinuation(PairEndpoint),
    Writable(PairEndpoint),
    Shutdown,
}

/// Shared completion state retained after the actor drops its descriptors.
#[derive(Clone)]
pub(crate) struct PairingStatus {
    close_reason: Rc<Cell<Option<PairingCloseReason>>>,
}

impl PairingStatus {
    fn new() -> Self {
        Self {
            close_reason: Rc::new(Cell::new(None)),
        }
    }

    pub(crate) fn close_reason(&self) -> Option<PairingCloseReason> {
        self.close_reason.get()
    }
}

/// Both endpoints and all bounded forwarding state for one pairing.
pub(crate) struct Pairing {
    readers: [ImsgReader; 2],
    writers: [NonblockingImsgWriter; 2],
    retries: [Option<Frame>; 2],
    read_tokens: [Option<Token>; 2],
    write_tokens: [Option<Token>; 2],
    read_work_queued: [bool; 2],
    write_work_queued: [bool; 2],
    read_eof: [bool; 2],
    write_shutdown: [bool; 2],
    status: PairingStatus,
    closed: bool,
}

impl Pairing {
    pub(crate) fn new(
        client_reader: ImsgReader,
        client_writer: NonblockingImsgWriter,
        server_reader: ImsgReader,
        server_writer: NonblockingImsgWriter,
    ) -> (Self, PairingStatus) {
        let status = PairingStatus::new();
        (
            Self {
                readers: [client_reader, server_reader],
                writers: [client_writer, server_writer],
                retries: [None, None],
                read_tokens: [None, None],
                write_tokens: [None, None],
                read_work_queued: [false, false],
                write_work_queued: [false, false],
                read_eof: [false, false],
                write_shutdown: [false, false],
                status: status.clone(),
                closed: false,
            },
            status,
        )
    }

    pub(crate) fn fd(&self, endpoint: PairEndpoint, side: PairIoSide) -> BorrowedFd<'_> {
        match side {
            PairIoSide::Read => self.readers[endpoint.index()].as_fd(),
            PairIoSide::Write => self.writers[endpoint.index()].as_fd(),
        }
    }

    pub(crate) fn token(&self, endpoint: PairEndpoint, side: PairIoSide) -> Option<Token> {
        match side {
            PairIoSide::Read => self.read_tokens[endpoint.index()],
            PairIoSide::Write => self.write_tokens[endpoint.index()],
        }
    }

    pub(crate) fn set_token(
        &mut self,
        endpoint: PairEndpoint,
        side: PairIoSide,
        token: Option<Token>,
    ) {
        match side {
            PairIoSide::Read => self.read_tokens[endpoint.index()] = token,
            PairIoSide::Write => self.write_tokens[endpoint.index()] = token,
        }
    }

    pub(crate) fn mark_work_queued(&mut self, endpoint: PairEndpoint, side: PairIoSide) -> bool {
        let index = endpoint.index();
        let queued = match side {
            PairIoSide::Read => &mut self.read_work_queued[index],
            PairIoSide::Write => &mut self.write_work_queued[index],
        };
        if self.closed
            || *queued
            || (side == PairIoSide::Read && self.read_eof[index])
            || (side == PairIoSide::Read && self.retries[index].is_some())
        {
            return false;
        }
        *queued = true;
        true
    }

    pub(crate) fn handle(
        &mut self,
        target: &ActorRef<Self>,
        event: PairingEvent,
        outbox: &mut Outbox,
    ) {
        if self.closed {
            return;
        }

        match event {
            PairingEvent::Start => {
                for endpoint in PairEndpoint::ALL {
                    outbox.set_pairing_interest(target.clone(), endpoint, PairIoSide::Read, true);
                    if self.writers[endpoint.index()].has_pending() {
                        outbox.set_pairing_interest(
                            target.clone(),
                            endpoint,
                            PairIoSide::Write,
                            true,
                        );
                    }
                }
            }
            PairingEvent::Readable(endpoint) | PairingEvent::ReadContinuation(endpoint) => {
                self.read_work_queued[endpoint.index()] = false;
                self.handle_readable(target, endpoint, outbox);
            }
            PairingEvent::Writable(endpoint) => {
                self.write_work_queued[endpoint.index()] = false;
                self.handle_writable(target, endpoint, outbox);
            }
            PairingEvent::Shutdown => {
                self.close(target, PairingCloseReason::Shutdown, outbox);
            }
        }
    }

    fn handle_readable(
        &mut self,
        target: &ActorRef<Self>,
        source: PairEndpoint,
        outbox: &mut Outbox,
    ) {
        if self.read_eof[source.index()] || self.retries[source.index()].is_some() {
            return;
        }

        let destination = source.other();
        for _ in 0..READ_FRAME_BUDGET {
            let frame = match self.readers[source.index()].try_recv() {
                Ok(frame) => frame,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => return,
                Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
                    self.handle_eof(target, source, outbox);
                    return;
                }
                Err(error) => {
                    self.close(target, PairingCloseReason::Error(error.kind()), outbox);
                    return;
                }
            };
            log_frame(source.direction(), &frame);

            match self.writers[destination.index()].try_queue(frame) {
                Ok(()) => {
                    outbox.set_pairing_interest(
                        target.clone(),
                        destination,
                        PairIoSide::Write,
                        true,
                    );
                }
                Err(error) if self.writers[destination.index()].has_pending() => {
                    self.retries[source.index()] = Some(error.into_frame());
                    outbox.set_pairing_interest(target.clone(), source, PairIoSide::Read, false);
                    outbox.set_pairing_interest(
                        target.clone(),
                        destination,
                        PairIoSide::Write,
                        true,
                    );
                    return;
                }
                Err(_) => {
                    self.close(target, PairingCloseReason::FrameExceedsQueueLimit, outbox);
                    return;
                }
            }
        }

        self.schedule_read_continuation(target, source, outbox);
    }

    fn handle_writable(
        &mut self,
        target: &ActorRef<Self>,
        destination: PairEndpoint,
        outbox: &mut Outbox,
    ) {
        match self.writers[destination.index()].try_flush() {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            Err(error) => {
                self.close(target, PairingCloseReason::Error(error.kind()), outbox);
                return;
            }
        }

        let source = destination.other();
        let mut accepted_retry = false;
        if let Some(frame) = self.retries[source.index()].take() {
            match self.writers[destination.index()].try_queue(frame) {
                Ok(()) => {
                    accepted_retry = true;
                    if !self.read_eof[source.index()] {
                        outbox.set_pairing_interest(target.clone(), source, PairIoSide::Read, true);
                        self.schedule_read_continuation(target, source, outbox);
                    }
                }
                Err(error) if self.writers[destination.index()].has_pending() => {
                    self.retries[source.index()] = Some(error.into_frame());
                }
                Err(_) => {
                    self.close(target, PairingCloseReason::FrameExceedsQueueLimit, outbox);
                    return;
                }
            }
        }

        // Writable readiness is edge-triggered. A retry accepted after the
        // existing queue drains must be advanced before returning.
        if accepted_retry {
            match self.writers[destination.index()].try_flush() {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => {
                    self.close(target, PairingCloseReason::Error(error.kind()), outbox);
                    return;
                }
            }
        }

        outbox.set_pairing_interest(
            target.clone(),
            destination,
            PairIoSide::Write,
            self.writers[destination.index()].has_pending(),
        );
        self.advance_half_closes(target, outbox);
    }

    fn schedule_read_continuation(
        &mut self,
        target: &ActorRef<Self>,
        source: PairEndpoint,
        outbox: &mut Outbox,
    ) {
        if self.mark_work_queued(source, PairIoSide::Read) {
            outbox.enqueue_pairing(target.clone(), PairingEvent::ReadContinuation(source));
        }
    }

    fn handle_eof(&mut self, target: &ActorRef<Self>, source: PairEndpoint, outbox: &mut Outbox) {
        if self.read_eof[source.index()] {
            return;
        }
        self.read_eof[source.index()] = true;
        outbox.set_pairing_interest(target.clone(), source, PairIoSide::Read, false);
        self.advance_half_closes(target, outbox);
    }

    fn advance_half_closes(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        for source in PairEndpoint::ALL {
            let destination = source.other();
            if self.read_eof[source.index()]
                && self.retries[source.index()].is_none()
                && !self.writers[destination.index()].has_pending()
                && !self.write_shutdown[destination.index()]
            {
                // A successful SHUT_WR guarantees queued Unix-stream bytes are
                // delivered before EOF. The opposite read half remains alive
                // so a final client acknowledgement cannot turn the close into
                // a reset that discards those bytes.
                unsafe {
                    libc::shutdown(
                        self.writers[destination.index()].as_fd().as_raw_fd(),
                        libc::SHUT_WR,
                    );
                }
                self.write_shutdown[destination.index()] = true;
                outbox.set_pairing_interest(target.clone(), destination, PairIoSide::Write, false);
            }
        }

        if self.read_eof.iter().all(|eof| *eof)
            && PairEndpoint::ALL.iter().all(|source| {
                let destination = source.other();
                self.retries[source.index()].is_none()
                    && !self.writers[destination.index()].has_pending()
            })
        {
            self.close(target, PairingCloseReason::PeerClosed, outbox);
        }
    }

    fn close(&mut self, target: &ActorRef<Self>, reason: PairingCloseReason, outbox: &mut Outbox) {
        self.closed = true;
        self.retries = [None, None];
        self.status.close_reason.set(Some(reason));
        for endpoint in PairEndpoint::ALL {
            for side in [PairIoSide::Read, PairIoSide::Write] {
                outbox.set_pairing_interest(target.clone(), endpoint, side, false);
            }
        }
        outbox.stop_pairing(target.clone());
    }
}
