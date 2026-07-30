//! Central FIFO dispatch and deferred reactor effects.

use std::collections::VecDeque;
use std::io;
use std::time::Duration;

use crate::tmux::codec::{ImsgReader, NonblockingImsgWriter};
use crate::tmux::message::Frame;

use super::actor::{ActorRef, WeakActorRef};
use super::client::{dispatch_inbox, ClientInbox, ClientInboxEvent, ClientIo, ClientIoEvent};
use super::reactor::{Interest, MioReactor, PollResult, Reactor, Ready};

/// One queued event with a direct reference to its destination.
pub(crate) enum Envelope {
    ClientIo {
        target: ActorRef<ClientIo>,
        event: ClientIoEvent,
    },
    ClientInbox {
        target: ActorRef<ClientInbox>,
        event: ClientInboxEvent,
    },
}

impl Envelope {
    fn dispatch(self, outbox: &mut Outbox) {
        match self {
            Envelope::ClientIo { target, event } => {
                let dispatch_target = target.clone();
                target.with_mut(|client| client.handle(&dispatch_target, event, outbox));
            }
            Envelope::ClientInbox { target, event } => {
                dispatch_inbox(&target, event);
            }
        }
    }
}

enum Effect {
    Enqueue(Envelope),
    SetReadInterest {
        target: ActorRef<ClientIo>,
        enabled: bool,
    },
    SetWriteInterest {
        target: ActorRef<ClientIo>,
        enabled: bool,
    },
    StopClient(ActorRef<ClientIo>),
}

/// Effects emitted by one handler and applied only after it returns.
pub(crate) struct Outbox {
    effects: Vec<Effect>,
}

impl Outbox {
    fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    pub(crate) fn enqueue(&mut self, envelope: Envelope) {
        self.effects.push(Effect::Enqueue(envelope));
    }

    pub(crate) fn set_read_interest(&mut self, target: ActorRef<ClientIo>, enabled: bool) {
        self.effects
            .push(Effect::SetReadInterest { target, enabled });
    }

    pub(crate) fn set_write_interest(&mut self, target: ActorRef<ClientIo>, enabled: bool) {
        self.effects
            .push(Effect::SetWriteInterest { target, enabled });
    }

    pub(crate) fn stop_client(&mut self, target: ActorRef<ClientIo>) {
        self.effects.push(Effect::StopClient(target));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IoSide {
    Read,
    Write,
}

#[derive(Clone, Debug)]
pub(crate) struct IoRecipient {
    target: WeakActorRef<ClientIo>,
    side: IoSide,
}

/// References returned when a connection is added to the loop.
pub(crate) struct ClientHandle {
    io: ActorRef<ClientIo>,
    inbox: ActorRef<ClientInbox>,
}

impl ClientHandle {
    pub(crate) fn io(&self) -> &ActorRef<ClientIo> {
        &self.io
    }

    pub(crate) fn inbox(&self) -> &ActorRef<ClientInbox> {
        &self.inbox
    }
}

/// Metadata for one event-loop turn.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct TurnResult {
    dispatched: usize,
    poll: Option<PollResult>,
}

impl TurnResult {
    pub(crate) fn dispatched(self) -> usize {
        self.dispatched
    }

    pub(crate) fn poll_result(self) -> Option<PollResult> {
        self.poll
    }
}

/// Single-threaded event loop with a central FIFO.
pub(crate) struct EventLoop<R>
where
    R: Reactor<IoRecipient>,
{
    reactor: R,
    events: VecDeque<Envelope>,
    ready: Vec<Ready<IoRecipient>>,
}

impl EventLoop<MioReactor<IoRecipient>> {
    pub(crate) fn new() -> io::Result<Self> {
        Ok(Self::with_reactor(MioReactor::new()?))
    }
}

impl<R> EventLoop<R>
where
    R: Reactor<IoRecipient>,
{
    fn with_reactor(reactor: R) -> Self {
        Self {
            reactor,
            events: VecDeque::new(),
            ready: Vec::new(),
        }
    }

    pub(crate) fn add_client(
        &mut self,
        reader: ImsgReader,
        writer: NonblockingImsgWriter,
    ) -> ClientHandle {
        let inbox = ActorRef::new(ClientInbox::default());
        let io = ActorRef::new(ClientIo::new(reader, writer, inbox.clone()));
        self.events.push_back(Envelope::ClientIo {
            target: io.clone(),
            event: ClientIoEvent::Start,
        });
        ClientHandle { io, inbox }
    }

    pub(crate) fn try_send(
        &mut self,
        target: &ActorRef<ClientIo>,
        frame: Frame,
    ) -> Result<(), Frame> {
        let accepted = target
            .with_mut(ClientIo::mark_send_work_queued)
            .unwrap_or(false);
        if !accepted {
            return Err(frame);
        }

        self.events.push_back(Envelope::ClientIo {
            target: target.clone(),
            event: ClientIoEvent::Send(frame),
        });
        Ok(())
    }

    pub(crate) fn shutdown(&mut self, target: &ActorRef<ClientIo>) {
        self.events.push_back(Envelope::ClientIo {
            target: target.clone(),
            event: ClientIoEvent::Shutdown,
        });
    }

    pub(crate) fn pending_events(&self) -> usize {
        self.events.len()
    }

    pub(crate) fn dispatch_one(&mut self) -> io::Result<bool> {
        let Some(envelope) = self.events.pop_front() else {
            return Ok(false);
        };

        let mut outbox = Outbox::new();
        envelope.dispatch(&mut outbox);
        for effect in outbox.effects {
            self.apply(effect)?;
        }
        Ok(true)
    }

    pub(crate) fn dispatch_with_budget(&mut self, budget: usize) -> io::Result<usize> {
        let mut dispatched = 0;
        while dispatched < budget && self.dispatch_one()? {
            dispatched += 1;
        }
        Ok(dispatched)
    }

    pub(crate) fn poll(&mut self, timeout: Option<Duration>) -> io::Result<PollResult> {
        let result = self.reactor.poll(timeout, &mut self.ready)?;
        let mut ready = std::mem::take(&mut self.ready);
        for notification in ready.drain(..) {
            self.enqueue_readiness(notification);
        }
        self.ready = ready;
        Ok(result)
    }

    pub(crate) fn run_turn(
        &mut self,
        timeout: Option<Duration>,
        dispatch_budget: usize,
    ) -> io::Result<TurnResult> {
        let dispatched = self.dispatch_with_budget(dispatch_budget)?;
        if !self.events.is_empty() {
            return Ok(TurnResult {
                dispatched,
                poll: None,
            });
        }

        let poll = self.poll(timeout)?;
        Ok(TurnResult {
            dispatched,
            poll: Some(poll),
        })
    }

    fn enqueue_readiness(&mut self, notification: Ready<IoRecipient>) {
        let recipient = notification.recipient();
        let Some(target) = recipient.target.upgrade() else {
            return;
        };

        let should_enqueue = match recipient.side {
            IoSide::Read => target
                .with_mut(ClientIo::mark_read_work_queued)
                .unwrap_or(false),
            IoSide::Write => target
                .with_mut(ClientIo::mark_write_work_queued)
                .unwrap_or(false),
        };
        if !should_enqueue {
            return;
        }

        let event = match recipient.side {
            IoSide::Read => ClientIoEvent::Readable,
            IoSide::Write => ClientIoEvent::Writable,
        };
        self.events.push_back(Envelope::ClientIo { target, event });
    }

    fn apply(&mut self, effect: Effect) -> io::Result<()> {
        match effect {
            Effect::Enqueue(envelope) => self.events.push_back(envelope),
            Effect::SetReadInterest { target, enabled } => {
                self.set_read_interest(&target, enabled)?;
            }
            Effect::SetWriteInterest { target, enabled } => {
                self.set_write_interest(&target, enabled)?;
            }
            Effect::StopClient(target) => {
                target.stop();
            }
        }
        Ok(())
    }

    fn set_read_interest(&mut self, target: &ActorRef<ClientIo>, enabled: bool) -> io::Result<()> {
        let token = target.with(ClientIo::read_token).flatten();
        match (enabled, token) {
            (true, None) => {
                let recipient = IoRecipient {
                    target: target.downgrade(),
                    side: IoSide::Read,
                };
                if let Some(result) = target.with_mut(|client| {
                    let token =
                        self.reactor
                            .register(client.read_fd(), Interest::READABLE, recipient)?;
                    client.set_read_token(Some(token));
                    Ok::<(), io::Error>(())
                }) {
                    result?;
                }
            }
            (false, Some(token)) => {
                self.reactor.deregister(token)?;
                target.with_mut(|client| client.set_read_token(None));
            }
            _ => {}
        }
        Ok(())
    }

    fn set_write_interest(&mut self, target: &ActorRef<ClientIo>, enabled: bool) -> io::Result<()> {
        let token = target.with(ClientIo::write_token).flatten();
        match (enabled, token) {
            (true, None) => {
                let recipient = IoRecipient {
                    target: target.downgrade(),
                    side: IoSide::Write,
                };
                if let Some(result) = target.with_mut(|client| {
                    let token =
                        self.reactor
                            .register(client.write_fd(), Interest::WRITABLE, recipient)?;
                    client.set_write_token(Some(token));
                    Ok::<(), io::Error>(())
                }) {
                    result?;
                }
            }
            (false, Some(token)) => {
                self.reactor.deregister(token)?;
                target.with_mut(|client| client.set_write_token(None));
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::os::fd::{AsFd as _, OwnedFd};
    use std::os::unix::net::UnixStream;

    use super::*;
    use crate::event_loop::client::{CloseReason, READ_FRAME_BUDGET};
    use crate::tmux::codec::{dup_fd, encode_bytes, split_nonblocking_stream, split_stream};
    use crate::tmux::message::Message;

    const POLL_TIMEOUT: Duration = Duration::from_secs(1);

    fn dispatch_all(loop_: &mut EventLoop<MioReactor<IoRecipient>>) {
        let dispatched = loop_.dispatch_with_budget(4096).unwrap();
        assert!(dispatched < 4096, "event queue did not quiesce");
    }

    fn nonblocking_pair_with_limit(
        stream: UnixStream,
        limit: usize,
    ) -> io::Result<(ImsgReader, NonblockingImsgWriter)> {
        let read_fd: OwnedFd = stream.into();
        let write_fd = dup_fd(read_fd.as_fd())?;
        Ok((
            ImsgReader::new(read_fd),
            NonblockingImsgWriter::with_queue_limit(write_fd, limit),
        ))
    }

    #[test]
    fn read_budget_uses_deferred_fifo_continuation_for_buffered_frames() {
        let (peer, server) = UnixStream::pair().unwrap();
        let (_peer_reader, mut peer_writer) = split_stream(peer).unwrap();
        let (server_reader, server_writer) = split_nonblocking_stream(server).unwrap();
        let mut loop_ = EventLoop::new().unwrap();
        let client = loop_.add_client(server_reader, server_writer);
        dispatch_all(&mut loop_);

        for index in 0..READ_FRAME_BUDGET + 2 {
            peer_writer
                .send(Frame::new(Message::Command(vec![index.to_string()])))
                .unwrap();
        }

        loop_.poll(Some(POLL_TIMEOUT)).unwrap();
        assert_eq!(loop_.pending_events(), 1);
        assert_eq!(
            loop_.dispatch_with_budget(READ_FRAME_BUDGET + 1).unwrap(),
            READ_FRAME_BUDGET + 1
        );
        assert_eq!(
            client.inbox().with(ClientInbox::len),
            Some(READ_FRAME_BUDGET)
        );
        assert_eq!(loop_.pending_events(), 1);

        dispatch_all(&mut loop_);
        let frames = client
            .inbox()
            .with_mut(|inbox| std::iter::from_fn(|| inbox.pop_frame()).collect::<Vec<_>>())
            .unwrap();
        assert_eq!(frames.len(), READ_FRAME_BUDGET + 2);
        for (index, frame) in frames.into_iter().enumerate() {
            assert_eq!(frame.msg, Message::Command(vec![index.to_string()]));
        }
    }

    #[test]
    fn writable_interest_exists_only_while_output_is_pending() {
        let (peer, server) = UnixStream::pair().unwrap();
        let (mut peer_reader, _peer_writer) = split_stream(peer).unwrap();
        let (server_reader, server_writer) = split_nonblocking_stream(server).unwrap();
        let mut loop_ = EventLoop::new().unwrap();
        let client = loop_.add_client(server_reader, server_writer);
        dispatch_all(&mut loop_);

        loop_
            .try_send(client.io(), Frame::new(Message::Ready))
            .unwrap();
        dispatch_all(&mut loop_);
        loop_
            .try_send(client.io(), Frame::new(Message::Exited))
            .unwrap();
        dispatch_all(&mut loop_);
        assert!(client.io().with(ClientIo::write_token).flatten().is_some());

        loop_.poll(Some(POLL_TIMEOUT)).unwrap();
        dispatch_all(&mut loop_);
        assert_eq!(client.io().with(ClientIo::write_token).flatten(), None);
        assert_eq!(peer_reader.recv().unwrap().msg, Message::Ready);
        assert_eq!(peer_reader.recv().unwrap().msg, Message::Exited);
    }

    #[test]
    fn deferred_send_slot_returns_a_second_unconsumed_frame() {
        let (_peer, server) = UnixStream::pair().unwrap();
        let (server_reader, server_writer) = split_nonblocking_stream(server).unwrap();
        let mut loop_ = EventLoop::new().unwrap();
        let client = loop_.add_client(server_reader, server_writer);
        let rejected_message = Message::Exited;

        loop_
            .try_send(client.io(), Frame::new(Message::Ready))
            .unwrap();
        let rejected = loop_
            .try_send(client.io(), Frame::new(rejected_message.clone()))
            .expect_err("one send event is already deferred");

        assert_eq!(rejected.msg, rejected_message);
        assert_eq!(loop_.pending_events(), 2);
        dispatch_all(&mut loop_);
        loop_.try_send(client.io(), rejected).unwrap();
    }

    #[test]
    fn queue_full_pauses_reads_and_retries_frame_in_order() {
        let (peer, server) = UnixStream::pair().unwrap();
        let (mut peer_reader, _peer_writer) = split_stream(peer).unwrap();
        let first = Message::Command(vec!["one".into()]);
        let second = Message::Command(vec!["two".into()]);
        let queue_limit = encode_bytes(&Frame::new(first.clone())).len();
        let (server_reader, server_writer) =
            nonblocking_pair_with_limit(server, queue_limit).unwrap();
        let mut loop_ = EventLoop::new().unwrap();
        let client = loop_.add_client(server_reader, server_writer);
        dispatch_all(&mut loop_);

        loop_
            .try_send(client.io(), Frame::new(first.clone()))
            .unwrap();
        dispatch_all(&mut loop_);
        loop_
            .try_send(client.io(), Frame::new(second.clone()))
            .unwrap();
        dispatch_all(&mut loop_);

        assert_eq!(client.io().with(ClientIo::reads_paused), Some(true));
        assert_eq!(client.io().with(ClientIo::has_retry), Some(true));
        assert_eq!(client.io().with(ClientIo::read_token).flatten(), None);
        assert!(client.io().with(ClientIo::write_token).flatten().is_some());

        loop_.poll(Some(POLL_TIMEOUT)).unwrap();
        dispatch_all(&mut loop_);
        assert_eq!(client.io().with(ClientIo::reads_paused), Some(false));
        assert_eq!(client.io().with(ClientIo::has_retry), Some(false));
        assert!(client.io().with(ClientIo::read_token).flatten().is_some());

        assert_eq!(client.io().with(ClientIo::write_token).flatten(), None);
        assert_eq!(peer_reader.recv().unwrap().msg, first);
        assert_eq!(peer_reader.recv().unwrap().msg, second);
    }

    #[test]
    fn shutdown_deregisters_sources_before_stopping_actor() {
        let (_peer, server) = UnixStream::pair().unwrap();
        let (server_reader, server_writer) = split_nonblocking_stream(server).unwrap();
        let mut loop_ = EventLoop::new().unwrap();
        let client = loop_.add_client(server_reader, server_writer);
        dispatch_all(&mut loop_);

        loop_.shutdown(client.io());
        dispatch_all(&mut loop_);

        assert!(!client.io().is_alive());
        assert_eq!(
            client.inbox().with(ClientInbox::close_reason),
            Some(Some(CloseReason::Shutdown))
        );
    }

    #[test]
    fn turn_skips_poll_while_dispatch_budget_leaves_queued_work() {
        let (_peer, server) = UnixStream::pair().unwrap();
        let (server_reader, server_writer) = split_nonblocking_stream(server).unwrap();
        let mut loop_ = EventLoop::new().unwrap();
        let client = loop_.add_client(server_reader, server_writer);
        loop_
            .try_send(client.io(), Frame::new(Message::Ready))
            .unwrap();

        let result = loop_.run_turn(Some(POLL_TIMEOUT), 1).unwrap();

        assert_eq!(result.dispatched(), 1);
        assert_eq!(result.poll_result(), None);
        assert_eq!(loop_.pending_events(), 1);
    }
}
