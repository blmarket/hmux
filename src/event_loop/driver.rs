//! Central FIFO dispatch and deferred reactor effects.

use std::collections::VecDeque;
use std::io;
use std::time::Duration;

use crate::tmux::codec::{ImsgReader, NonblockingImsgWriter};
use crate::tmux::message::Frame;
use crate::tmux::native::NativeServer;

use super::actor::{ActorRef, WeakActorRef};
use super::client::{dispatch_inbox, ClientInbox, ClientInboxEvent, ClientIo, ClientIoEvent};
use super::listener::{AcceptedClients, Listener, ListenerEvent};
use super::pairing::{PairEndpoint, PairIoSide, Pairing, PairingEvent, PairingStatus};
use super::protocol::{
    ProtocolClient, ProtocolCloseReason, ProtocolEvent, ProtocolIoSide, ProtocolStatus,
};
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
    Pairing {
        target: ActorRef<Pairing>,
        event: PairingEvent,
    },
    Listener {
        target: ActorRef<Listener>,
        event: ListenerEvent,
    },
    Protocol {
        target: ActorRef<ProtocolClient>,
        event: ProtocolEvent,
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
            Envelope::Pairing { target, event } => {
                let dispatch_target = target.clone();
                target.with_mut(|pairing| pairing.handle(&dispatch_target, event, outbox));
            }
            Envelope::Listener { target, event } => {
                let dispatch_target = target.clone();
                target.with_mut(|listener| listener.handle(&dispatch_target, event, outbox));
            }
            Envelope::Protocol { target, event } => {
                let dispatch_target = target.clone();
                target.with_mut(|client| client.handle(&dispatch_target, event, outbox));
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
    SetPairingInterest {
        target: ActorRef<Pairing>,
        endpoint: PairEndpoint,
        side: PairIoSide,
        enabled: bool,
    },
    SetListenerInterest {
        target: ActorRef<Listener>,
        enabled: bool,
    },
    SetProtocolInterest {
        target: ActorRef<ProtocolClient>,
        side: ProtocolIoSide,
        enabled: bool,
    },
    HandoffProtocol {
        target: ActorRef<ProtocolClient>,
        client_reader: ImsgReader,
        client_writer: NonblockingImsgWriter,
        server_reader: ImsgReader,
        server_writer: NonblockingImsgWriter,
    },
    StopClient(ActorRef<ClientIo>),
    StopPairing(ActorRef<Pairing>),
    StopListener(ActorRef<Listener>),
    StopProtocol(ActorRef<ProtocolClient>),
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

    pub(crate) fn enqueue_pairing(&mut self, target: ActorRef<Pairing>, event: PairingEvent) {
        self.enqueue(Envelope::Pairing { target, event });
    }

    pub(crate) fn enqueue_listener(&mut self, target: ActorRef<Listener>, event: ListenerEvent) {
        self.enqueue(Envelope::Listener { target, event });
    }

    pub(crate) fn enqueue_protocol(
        &mut self,
        target: ActorRef<ProtocolClient>,
        event: ProtocolEvent,
    ) {
        self.enqueue(Envelope::Protocol { target, event });
    }

    pub(crate) fn set_read_interest(&mut self, target: ActorRef<ClientIo>, enabled: bool) {
        self.effects
            .push(Effect::SetReadInterest { target, enabled });
    }

    pub(crate) fn set_write_interest(&mut self, target: ActorRef<ClientIo>, enabled: bool) {
        self.effects
            .push(Effect::SetWriteInterest { target, enabled });
    }

    pub(crate) fn set_pairing_interest(
        &mut self,
        target: ActorRef<Pairing>,
        endpoint: PairEndpoint,
        side: PairIoSide,
        enabled: bool,
    ) {
        self.effects.push(Effect::SetPairingInterest {
            target,
            endpoint,
            side,
            enabled,
        });
    }

    pub(crate) fn set_listener_interest(&mut self, target: ActorRef<Listener>, enabled: bool) {
        self.effects
            .push(Effect::SetListenerInterest { target, enabled });
    }

    pub(crate) fn set_protocol_interest(
        &mut self,
        target: ActorRef<ProtocolClient>,
        side: ProtocolIoSide,
        enabled: bool,
    ) {
        self.effects.push(Effect::SetProtocolInterest {
            target,
            side,
            enabled,
        });
    }

    pub(crate) fn handoff_protocol(
        &mut self,
        target: ActorRef<ProtocolClient>,
        client_reader: ImsgReader,
        client_writer: NonblockingImsgWriter,
        server_reader: ImsgReader,
        server_writer: NonblockingImsgWriter,
    ) {
        self.effects.push(Effect::HandoffProtocol {
            target,
            client_reader,
            client_writer,
            server_reader,
            server_writer,
        });
    }

    pub(crate) fn stop_client(&mut self, target: ActorRef<ClientIo>) {
        self.effects.push(Effect::StopClient(target));
    }

    pub(crate) fn stop_pairing(&mut self, target: ActorRef<Pairing>) {
        self.effects.push(Effect::StopPairing(target));
    }

    pub(crate) fn stop_listener(&mut self, target: ActorRef<Listener>) {
        self.effects.push(Effect::StopListener(target));
    }

    pub(crate) fn stop_protocol(&mut self, target: ActorRef<ProtocolClient>) {
        self.effects.push(Effect::StopProtocol(target));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IoSide {
    Read,
    Write,
}

#[derive(Clone, Debug)]
pub(crate) struct IoRecipient {
    target: IoTarget,
}

#[derive(Clone, Debug)]
enum IoTarget {
    Client {
        target: WeakActorRef<ClientIo>,
        side: IoSide,
    },
    Pairing {
        target: WeakActorRef<Pairing>,
        endpoint: PairEndpoint,
        side: PairIoSide,
    },
    Listener {
        target: WeakActorRef<Listener>,
    },
    Protocol {
        target: WeakActorRef<ProtocolClient>,
        side: ProtocolIoSide,
    },
}

/// References returned when a connection is added to the loop.
pub(crate) struct ClientHandle {
    io: ActorRef<ClientIo>,
    inbox: ActorRef<ClientInbox>,
}

/// References returned when a bidirectional pairing is added to the loop.
pub(crate) struct PairingHandle {
    pairing: ActorRef<Pairing>,
    status: PairingStatus,
}

/// References returned when a Unix listener is added to the loop.
pub(crate) struct ListenerHandle {
    listener: ActorRef<Listener>,
    accepted: AcceptedClients,
}

/// References returned when a protocol client is added to the loop.
pub(crate) struct ProtocolHandle {
    protocol: ActorRef<ProtocolClient>,
    status: ProtocolStatus,
}

impl ProtocolHandle {
    pub(crate) fn is_alive(&self) -> bool {
        self.protocol
            .with(ProtocolClient::is_active)
            .unwrap_or(false)
    }

    pub(crate) fn close_reason(&self) -> Option<ProtocolCloseReason> {
        self.protocol
            .with(ProtocolClient::close_reason)
            .flatten()
            .or_else(|| self.status.close_reason())
    }

    #[cfg(test)]
    pub(crate) fn is_direct(&self) -> bool {
        self.protocol
            .with(ProtocolClient::is_direct)
            .unwrap_or(false)
    }

    #[cfg(test)]
    pub(crate) fn is_fallback(&self) -> bool {
        self.protocol
            .with(ProtocolClient::is_fallback)
            .unwrap_or(false)
    }
}

impl ListenerHandle {
    pub(crate) fn pop_accepted(&self) -> Option<std::os::unix::net::UnixStream> {
        self.accepted.pop_front()
    }

    #[cfg(test)]
    fn accepted_len(&self) -> usize {
        self.accepted.len()
    }

    pub(crate) fn is_alive(&self) -> bool {
        self.listener.is_alive()
    }
}

impl PairingHandle {
    pub(crate) fn is_alive(&self) -> bool {
        self.pairing.is_alive()
    }

    pub(crate) fn status(&self) -> &PairingStatus {
        &self.status
    }
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

    pub(crate) fn add_pairing(
        &mut self,
        client_reader: ImsgReader,
        client_writer: NonblockingImsgWriter,
        server_reader: ImsgReader,
        server_writer: NonblockingImsgWriter,
    ) -> PairingHandle {
        let (pairing, status) =
            Pairing::new(client_reader, client_writer, server_reader, server_writer);
        let pairing = ActorRef::new(pairing);
        self.events.push_back(Envelope::Pairing {
            target: pairing.clone(),
            event: PairingEvent::Start,
        });
        PairingHandle { pairing, status }
    }

    pub(crate) fn add_protocol(
        &mut self,
        reader: ImsgReader,
        writer: NonblockingImsgWriter,
        server: NativeServer,
    ) -> ProtocolHandle {
        let (protocol, status) = ProtocolClient::new(reader, writer, server);
        let protocol = ActorRef::new(protocol);
        self.events.push_back(Envelope::Protocol {
            target: protocol.clone(),
            event: ProtocolEvent::Start,
        });
        ProtocolHandle { protocol, status }
    }

    pub(crate) fn add_listener(
        &mut self,
        listener: std::os::unix::net::UnixListener,
        accept_budget: usize,
    ) -> io::Result<ListenerHandle> {
        listener.set_nonblocking(true)?;
        let (listener, accepted) = Listener::new(listener, accept_budget);
        let listener = ActorRef::new(listener);
        self.events.push_back(Envelope::Listener {
            target: listener.clone(),
            event: ListenerEvent::Start,
        });
        Ok(ListenerHandle { listener, accepted })
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

    pub(crate) fn shutdown_pairing(&mut self, target: &PairingHandle) {
        self.events.push_back(Envelope::Pairing {
            target: target.pairing.clone(),
            event: PairingEvent::Shutdown,
        });
    }

    pub(crate) fn shutdown_listener(&mut self, target: &ListenerHandle) {
        let should_enqueue = target
            .listener
            .with_mut(Listener::request_shutdown)
            .unwrap_or(false);
        if should_enqueue {
            self.events.push_back(Envelope::Listener {
                target: target.listener.clone(),
                event: ListenerEvent::Shutdown,
            });
        }
    }

    pub(crate) fn shutdown_protocol(&mut self, target: &ProtocolHandle) {
        self.events.push_back(Envelope::Protocol {
            target: target.protocol.clone(),
            event: ProtocolEvent::Shutdown,
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
        match &notification.recipient().target {
            IoTarget::Client {
                target: recipient,
                side,
            } => {
                let Some(target) = recipient.upgrade() else {
                    return;
                };
                let should_enqueue = match side {
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

                let event = match side {
                    IoSide::Read => ClientIoEvent::Readable,
                    IoSide::Write => ClientIoEvent::Writable,
                };
                self.events.push_back(Envelope::ClientIo { target, event });
            }
            IoTarget::Pairing {
                target: recipient,
                endpoint,
                side,
            } => {
                let Some(target) = recipient.upgrade() else {
                    return;
                };
                let should_enqueue = target
                    .with_mut(|pairing| pairing.mark_work_queued(*endpoint, *side))
                    .unwrap_or(false);
                if !should_enqueue {
                    return;
                }

                let event = match side {
                    PairIoSide::Read => PairingEvent::Readable(*endpoint),
                    PairIoSide::Write => PairingEvent::Writable(*endpoint),
                };
                self.events.push_back(Envelope::Pairing { target, event });
            }
            IoTarget::Listener { target: recipient } => {
                let Some(target) = recipient.upgrade() else {
                    return;
                };
                let should_enqueue = target
                    .with_mut(Listener::mark_accept_work_queued)
                    .unwrap_or(false);
                if should_enqueue {
                    self.events.push_back(Envelope::Listener {
                        target,
                        event: ListenerEvent::Readable,
                    });
                }
            }
            IoTarget::Protocol {
                target: recipient,
                side,
            } => {
                let Some(target) = recipient.upgrade() else {
                    return;
                };
                let should_enqueue = target
                    .with_mut(|client| client.mark_work_queued(*side))
                    .unwrap_or(false);
                if !should_enqueue {
                    return;
                }
                let event = match side {
                    ProtocolIoSide::Read => ProtocolEvent::Readable,
                    ProtocolIoSide::Write => ProtocolEvent::Writable,
                    ProtocolIoSide::Command => ProtocolEvent::CommandCompleted,
                };
                self.events.push_back(Envelope::Protocol { target, event });
            }
        }
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
            Effect::SetPairingInterest {
                target,
                endpoint,
                side,
                enabled,
            } => {
                self.set_pairing_interest(&target, endpoint, side, enabled)?;
            }
            Effect::SetListenerInterest { target, enabled } => {
                self.set_listener_interest(&target, enabled)?;
            }
            Effect::SetProtocolInterest {
                target,
                side,
                enabled,
            } => {
                self.set_protocol_interest(&target, side, enabled)?;
            }
            Effect::HandoffProtocol {
                target,
                client_reader,
                client_writer,
                server_reader,
                server_writer,
            } => {
                let pairing =
                    self.add_pairing(client_reader, client_writer, server_reader, server_writer);
                target.with_mut(|client| client.install_fallback(pairing));
            }
            Effect::StopClient(target) => {
                target.stop();
            }
            Effect::StopPairing(target) => {
                target.stop();
            }
            Effect::StopListener(target) => {
                target.stop();
            }
            Effect::StopProtocol(target) => {
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
                    target: IoTarget::Client {
                        target: target.downgrade(),
                        side: IoSide::Read,
                    },
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
                    target: IoTarget::Client {
                        target: target.downgrade(),
                        side: IoSide::Write,
                    },
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

    fn set_pairing_interest(
        &mut self,
        target: &ActorRef<Pairing>,
        endpoint: PairEndpoint,
        side: PairIoSide,
        enabled: bool,
    ) -> io::Result<()> {
        let token = target
            .with(|pairing| pairing.token(endpoint, side))
            .flatten();
        match (enabled, token) {
            (true, None) => {
                let recipient = IoRecipient {
                    target: IoTarget::Pairing {
                        target: target.downgrade(),
                        endpoint,
                        side,
                    },
                };
                if let Some(result) = target.with_mut(|pairing| {
                    let token = self.reactor.register(
                        pairing.fd(endpoint, side),
                        match side {
                            PairIoSide::Read => Interest::READABLE,
                            PairIoSide::Write => Interest::WRITABLE,
                        },
                        recipient,
                    )?;
                    pairing.set_token(endpoint, side, Some(token));
                    Ok::<(), io::Error>(())
                }) {
                    result?;
                }
            }
            (false, Some(token)) => {
                self.reactor.deregister(token)?;
                target.with_mut(|pairing| pairing.set_token(endpoint, side, None));
            }
            _ => {}
        }
        Ok(())
    }

    fn set_listener_interest(
        &mut self,
        target: &ActorRef<Listener>,
        enabled: bool,
    ) -> io::Result<()> {
        let token = target.with(Listener::token).flatten();
        match (enabled, token) {
            (true, None) => {
                let recipient = IoRecipient {
                    target: IoTarget::Listener {
                        target: target.downgrade(),
                    },
                };
                if let Some(result) = target.with_mut(|listener| {
                    let token =
                        self.reactor
                            .register(listener.fd(), Interest::READABLE, recipient)?;
                    listener.set_token(Some(token));
                    Ok::<(), io::Error>(())
                }) {
                    result?;
                }
            }
            (false, Some(token)) => {
                self.reactor.deregister(token)?;
                target.with_mut(|listener| listener.set_token(None));
            }
            _ => {}
        }
        Ok(())
    }

    fn set_protocol_interest(
        &mut self,
        target: &ActorRef<ProtocolClient>,
        side: ProtocolIoSide,
        enabled: bool,
    ) -> io::Result<()> {
        let token = target.with(|client| client.token(side)).flatten();
        match (enabled, token) {
            (true, None) => {
                let recipient = IoRecipient {
                    target: IoTarget::Protocol {
                        target: target.downgrade(),
                        side,
                    },
                };
                if let Some(result) = target.with_mut(|client| {
                    let source = client.fd(side).ok_or_else(|| {
                        io::Error::other("protocol readiness source is unavailable")
                    })?;
                    let token = self.reactor.register(
                        source,
                        match side {
                            ProtocolIoSide::Read | ProtocolIoSide::Command => Interest::READABLE,
                            ProtocolIoSide::Write => Interest::WRITABLE,
                        },
                        recipient,
                    )?;
                    client.set_token(side, Some(token));
                    Ok::<(), io::Error>(())
                }) {
                    result?;
                }
            }
            (false, Some(token)) => {
                self.reactor.deregister(token)?;
                target.with_mut(|client| client.set_token(side, None));
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::os::fd::{AsFd as _, OwnedFd};
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;
    use crate::event_loop::client::{CloseReason, READ_FRAME_BUDGET};
    use crate::event_loop::pairing::PairingCloseReason;
    use crate::tmux::codec::{dup_fd, encode_bytes, split_nonblocking_stream, split_stream};
    use crate::tmux::message::Message;

    const POLL_TIMEOUT: Duration = Duration::from_secs(1);
    static NEXT_LISTENER_PATH: AtomicU64 = AtomicU64::new(0);

    struct ListenerPath(PathBuf);

    impl ListenerPath {
        fn new() -> Self {
            let sequence = NEXT_LISTENER_PATH.fetch_add(1, Ordering::Relaxed);
            Self(PathBuf::from(format!(
                "/tmp/hmux-event-loop-test-{}-{sequence}.sock",
                std::process::id()
            )))
        }
    }

    impl Drop for ListenerPath {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

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
    fn listener_readiness_accepts_a_waiting_client() {
        let path = ListenerPath::new();
        let listener = UnixListener::bind(&path.0).unwrap();
        let mut loop_ = EventLoop::new().unwrap();
        let listener = loop_.add_listener(listener, 64).unwrap();
        dispatch_all(&mut loop_);

        let client = UnixStream::connect(&path.0).unwrap();
        assert_eq!(listener.accepted_len(), 0);

        let poll = loop_.poll(Some(POLL_TIMEOUT)).unwrap();
        assert_eq!(poll.ready_count(), 1);
        assert_eq!(listener.accepted_len(), 0);
        assert!(loop_.dispatch_one().unwrap());

        let accepted = listener.pop_accepted().expect("accepted client");
        assert!(listener.pop_accepted().is_none());
        drop((accepted, client));

        loop_.shutdown_listener(&listener);
        dispatch_all(&mut loop_);
        assert!(!listener.is_alive());
    }

    #[test]
    fn listener_budget_queues_one_accept_continuation() {
        let path = ListenerPath::new();
        let listener = UnixListener::bind(&path.0).unwrap();
        let mut loop_ = EventLoop::new().unwrap();
        let listener = loop_.add_listener(listener, 2).unwrap();
        dispatch_all(&mut loop_);

        let clients = (0..3)
            .map(|_| UnixStream::connect(&path.0).unwrap())
            .collect::<Vec<_>>();
        loop_.poll(Some(POLL_TIMEOUT)).unwrap();

        assert!(loop_.dispatch_one().unwrap());
        assert_eq!(listener.accepted_len(), 2);
        assert_eq!(loop_.pending_events(), 1);

        assert!(loop_.dispatch_one().unwrap());
        assert_eq!(listener.accepted_len(), 3);
        assert_eq!(loop_.pending_events(), 0);

        while listener.pop_accepted().is_some() {}
        drop(clients);
        loop_.shutdown_listener(&listener);
        dispatch_all(&mut loop_);
        assert!(!listener.is_alive());
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
    fn exact_read_budget_continues_to_probe_edge_triggered_source() {
        let (peer, server) = UnixStream::pair().unwrap();
        let (_peer_reader, mut peer_writer) = split_stream(peer).unwrap();
        let (server_reader, server_writer) = split_nonblocking_stream(server).unwrap();
        let mut loop_ = EventLoop::new().unwrap();
        let client = loop_.add_client(server_reader, server_writer);
        dispatch_all(&mut loop_);

        for index in 0..READ_FRAME_BUDGET {
            peer_writer
                .send(Frame::new(Message::Command(vec![index.to_string()])))
                .unwrap();
        }

        loop_.poll(Some(POLL_TIMEOUT)).unwrap();
        assert_eq!(
            loop_.dispatch_with_budget(READ_FRAME_BUDGET + 1).unwrap(),
            READ_FRAME_BUDGET + 1
        );
        assert_eq!(
            client.inbox().with(ClientInbox::len),
            Some(READ_FRAME_BUDGET)
        );
        assert_eq!(
            loop_.pending_events(),
            1,
            "budget exhaustion must leave one read continuation"
        );

        dispatch_all(&mut loop_);
        assert_eq!(loop_.pending_events(), 0);
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

    #[test]
    fn pairing_forwards_frames_in_both_directions() {
        use std::fs::OpenOptions;

        let (client_peer, client_endpoint) = UnixStream::pair().unwrap();
        let (server_peer, server_endpoint) = UnixStream::pair().unwrap();
        let (mut client_peer_reader, mut client_peer_writer) = split_stream(client_peer).unwrap();
        let (mut server_peer_reader, mut server_peer_writer) = split_stream(server_peer).unwrap();
        let (client_reader, client_writer) = split_nonblocking_stream(client_endpoint).unwrap();
        let (server_reader, server_writer) = split_nonblocking_stream(server_endpoint).unwrap();
        let mut loop_ = EventLoop::new().unwrap();
        let pairing = loop_.add_pairing(client_reader, client_writer, server_reader, server_writer);
        dispatch_all(&mut loop_);

        let passed_fd = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/null")
            .unwrap()
            .into();
        client_peer_writer
            .send(Frame::with_fd(
                Message::Command(vec!["client".into()]),
                passed_fd,
            ))
            .unwrap();
        server_peer_writer
            .send(Frame::new(Message::Command(vec!["server".into()])))
            .unwrap();

        loop_.poll(Some(POLL_TIMEOUT)).unwrap();
        dispatch_all(&mut loop_);
        loop_.poll(Some(POLL_TIMEOUT)).unwrap();
        dispatch_all(&mut loop_);

        let client_frame = server_peer_reader.recv().unwrap();
        assert_eq!(client_frame.msg, Message::Command(vec!["client".into()]));
        assert!(client_frame.fd.is_some());
        assert_eq!(
            client_peer_reader.recv().unwrap().msg,
            Message::Command(vec!["server".into()])
        );
        assert!(pairing.is_alive());
        assert_eq!(pairing.status().close_reason(), None);
    }

    #[test]
    fn pairing_preserves_large_stream_under_backpressure() {
        const FRAME_COUNT: usize = 1024;
        const CHUNK_SIZE: usize = 8 * 1024;

        let (client_peer, client_endpoint) = UnixStream::pair().unwrap();
        let (server_peer, server_endpoint) = UnixStream::pair().unwrap();
        let (mut client_peer_reader, _client_peer_writer) = split_stream(client_peer).unwrap();
        let (server_peer_reader, mut server_peer_writer) = split_stream(server_peer).unwrap();
        drop(server_peer_reader);
        let (client_reader, client_writer) = split_nonblocking_stream(client_endpoint).unwrap();
        let (server_reader, server_writer) = split_nonblocking_stream(server_endpoint).unwrap();
        let mut loop_ = EventLoop::new().unwrap();
        let pairing = loop_.add_pairing(client_reader, client_writer, server_reader, server_writer);
        dispatch_all(&mut loop_);

        let sender = std::thread::spawn(move || {
            for index in 0..FRAME_COUNT {
                let mut data = vec![b'x'; CHUNK_SIZE];
                data[..std::mem::size_of::<usize>()].copy_from_slice(&index.to_ne_bytes());
                server_peer_writer
                    .send(Frame::new(Message::Write { stream: 3, data }))
                    .unwrap();
            }
            server_peer_writer
                .send(Frame::new(Message::WriteClose { stream: 3 }))
                .unwrap();
        });

        let initial_backpressure_deadline = std::time::Instant::now() + Duration::from_millis(100);
        while std::time::Instant::now() < initial_backpressure_deadline {
            loop_.run_turn(Some(Duration::from_millis(1)), 256).unwrap();
        }

        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        let mut next_index = 0;
        let mut saw_close = false;
        while std::time::Instant::now() < deadline && !saw_close {
            loop_.run_turn(Some(Duration::from_millis(1)), 256).unwrap();
            loop {
                match client_peer_reader.try_recv() {
                    Ok(Frame {
                        msg: Message::Write { stream: 3, data },
                        ..
                    }) => {
                        assert_eq!(data.len(), CHUNK_SIZE);
                        let index = usize::from_ne_bytes(
                            data[..std::mem::size_of::<usize>()].try_into().unwrap(),
                        );
                        assert_eq!(index, next_index);
                        next_index += 1;
                    }
                    Ok(Frame {
                        msg: Message::WriteClose { stream: 3 },
                        ..
                    }) => saw_close = true,
                    Ok(frame) => panic!("unexpected frame: {:?}", frame.msg),
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                    Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => break,
                    Err(error) => panic!("failed to receive forwarded frame: {error}"),
                }
            }
        }

        sender.join().unwrap();
        assert_eq!(next_index, FRAME_COUNT);
        assert!(saw_close);
        assert!(
            pairing.is_alive()
                || pairing.status().close_reason() == Some(PairingCloseReason::PeerClosed)
        );
    }

    #[test]
    fn pairing_queue_full_pauses_only_the_upstream_reader() {
        let (client_peer, client_endpoint) = UnixStream::pair().unwrap();
        let (server_peer, server_endpoint) = UnixStream::pair().unwrap();
        let (_client_peer_reader, mut client_peer_writer) = split_stream(client_peer).unwrap();
        let (mut server_peer_reader, _server_peer_writer) = split_stream(server_peer).unwrap();
        let first = Message::Command(vec!["one".into()]);
        let second = Message::Command(vec!["two".into()]);
        let queue_limit = encode_bytes(&Frame::new(first.clone())).len();
        let (client_reader, client_writer) = split_nonblocking_stream(client_endpoint).unwrap();
        let (server_reader, server_writer) =
            nonblocking_pair_with_limit(server_endpoint, queue_limit).unwrap();
        let mut loop_ = EventLoop::new().unwrap();
        let pairing = loop_.add_pairing(client_reader, client_writer, server_reader, server_writer);
        dispatch_all(&mut loop_);

        client_peer_writer.send(Frame::new(first.clone())).unwrap();
        client_peer_writer.send(Frame::new(second.clone())).unwrap();
        loop_.poll(Some(POLL_TIMEOUT)).unwrap();
        dispatch_all(&mut loop_);

        assert_eq!(
            pairing
                .pairing
                .with(|pairing| pairing.token(PairEndpoint::Client, PairIoSide::Read))
                .flatten(),
            None
        );
        assert!(pairing
            .pairing
            .with(|pairing| pairing.token(PairEndpoint::Server, PairIoSide::Read))
            .flatten()
            .is_some());

        loop_.poll(Some(POLL_TIMEOUT)).unwrap();
        dispatch_all(&mut loop_);

        assert!(pairing
            .pairing
            .with(|pairing| pairing.token(PairEndpoint::Client, PairIoSide::Read))
            .flatten()
            .is_some());
        assert_eq!(server_peer_reader.recv().unwrap().msg, first);
        assert_eq!(server_peer_reader.recv().unwrap().msg, second);
    }

    #[test]
    fn pairing_shutdown_deregisters_all_sources_before_stopping() {
        let (_client_peer, client_endpoint) = UnixStream::pair().unwrap();
        let (_server_peer, server_endpoint) = UnixStream::pair().unwrap();
        let (client_reader, client_writer) = split_nonblocking_stream(client_endpoint).unwrap();
        let (server_reader, server_writer) = split_nonblocking_stream(server_endpoint).unwrap();
        let mut loop_ = EventLoop::new().unwrap();
        let pairing = loop_.add_pairing(client_reader, client_writer, server_reader, server_writer);
        dispatch_all(&mut loop_);

        loop_.shutdown_pairing(&pairing);
        dispatch_all(&mut loop_);

        assert!(!pairing.is_alive());
        assert_eq!(
            pairing.status().close_reason(),
            Some(PairingCloseReason::Shutdown)
        );
    }
}
