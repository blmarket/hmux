//! Central FIFO dispatch and deferred reactor effects.

use std::collections::VecDeque;
use std::io;
use std::time::{Duration, Instant};

use crate::server::pane::PaneIo;
use crate::server::Server;
use crate::tmux::codec::{ImsgReader, NonblockingImsgWriter};

use super::actor::{ActorRef, WeakActorRef};
use super::job::{BackgroundCommands, JobEvent};
use super::listener::{AcceptedClients, Listener, ListenerEvent};
use super::pane::{EventPane, PaneEvent, PaneInterest};
use super::process::{ChildSignal, ChildSignalEvent};
use super::protocol::{
    ProtocolClient, ProtocolCloseReason, ProtocolEvent, ProtocolIoSide, ProtocolStatus,
};
use super::reactor::{Interest, MioReactor, PollResult, Reactor, Ready};
use super::timer::{ExpiredTimer, TimerQueue};

/// One queued event with a direct reference to its destination.
pub(crate) enum Envelope {
    Listener {
        target: ActorRef<Listener>,
        event: ListenerEvent,
    },
    Pane {
        target: ActorRef<EventPane>,
        event: PaneEvent,
    },
    ChildSignal {
        target: ActorRef<ChildSignal>,
        event: ChildSignalEvent,
    },
    Protocol {
        target: ActorRef<ProtocolClient>,
        event: ProtocolEvent,
    },
    Background {
        target: ActorRef<BackgroundCommands>,
        event: JobEvent,
    },
}

impl Envelope {
    fn dispatch(self, outbox: &mut Outbox) {
        match self {
            Envelope::Listener { target, event } => {
                let dispatch_target = target.clone();
                target.with_mut(|listener| listener.handle(&dispatch_target, event, outbox));
            }
            Envelope::Pane { target, event } => {
                let dispatch_target = target.clone();
                target.with_mut(|pane| pane.handle(&dispatch_target, event, outbox));
            }
            Envelope::ChildSignal { target, event } => {
                let dispatch_target = target.clone();
                target.with_mut(|signal| signal.handle(&dispatch_target, event, outbox));
            }
            Envelope::Protocol { target, event } => {
                let dispatch_target = target.clone();
                target.with_mut(|client| client.handle(&dispatch_target, event, outbox));
            }
            Envelope::Background { target, event } => {
                let dispatch_target = target.clone();
                target.with_mut(|jobs| jobs.handle(&dispatch_target, event, outbox));
            }
        }
    }
}

enum Effect {
    Enqueue(Envelope),
    SetListenerInterest {
        target: ActorRef<Listener>,
        enabled: bool,
    },
    SetPaneInterest {
        target: ActorRef<EventPane>,
        interest: PaneInterest,
    },
    SetChildSignalInterest {
        target: ActorRef<ChildSignal>,
        enabled: bool,
    },
    SetProtocolInterest {
        target: ActorRef<ProtocolClient>,
        side: ProtocolIoSide,
        enabled: bool,
    },
    SetProtocolTimer {
        target: ActorRef<ProtocolClient>,
        deadline: Instant,
        event: ProtocolEvent,
    },
    CancelProtocolTimer {
        target: ActorRef<ProtocolClient>,
    },
    StopListener(ActorRef<Listener>),
    StopPane(ActorRef<EventPane>),
    StopChildSignal(ActorRef<ChildSignal>),
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

    pub(crate) fn enqueue_listener(&mut self, target: ActorRef<Listener>, event: ListenerEvent) {
        self.enqueue(Envelope::Listener { target, event });
    }

    pub(crate) fn enqueue_pane(&mut self, target: ActorRef<EventPane>, event: PaneEvent) {
        self.enqueue(Envelope::Pane { target, event });
    }

    pub(crate) fn enqueue_child_signal(
        &mut self,
        target: ActorRef<ChildSignal>,
        event: ChildSignalEvent,
    ) {
        self.enqueue(Envelope::ChildSignal { target, event });
    }

    pub(crate) fn enqueue_protocol(
        &mut self,
        target: ActorRef<ProtocolClient>,
        event: ProtocolEvent,
    ) {
        self.enqueue(Envelope::Protocol { target, event });
    }

    pub(crate) fn enqueue_background(
        &mut self,
        target: ActorRef<BackgroundCommands>,
        event: JobEvent,
    ) {
        self.enqueue(Envelope::Background { target, event });
    }

    pub(crate) fn set_listener_interest(&mut self, target: ActorRef<Listener>, enabled: bool) {
        self.effects
            .push(Effect::SetListenerInterest { target, enabled });
    }

    pub(crate) fn set_pane_interest(
        &mut self,
        target: ActorRef<EventPane>,
        interest: PaneInterest,
    ) {
        self.effects
            .push(Effect::SetPaneInterest { target, interest });
    }

    pub(crate) fn set_child_signal_interest(
        &mut self,
        target: ActorRef<ChildSignal>,
        enabled: bool,
    ) {
        self.effects
            .push(Effect::SetChildSignalInterest { target, enabled });
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

    pub(crate) fn set_protocol_timer_event(
        &mut self,
        target: ActorRef<ProtocolClient>,
        deadline: Instant,
        event: ProtocolEvent,
    ) {
        self.effects.push(Effect::SetProtocolTimer {
            target,
            deadline,
            event,
        });
    }

    pub(crate) fn cancel_protocol_timer(&mut self, target: ActorRef<ProtocolClient>) {
        self.effects.push(Effect::CancelProtocolTimer { target });
    }

    pub(crate) fn stop_listener(&mut self, target: ActorRef<Listener>) {
        self.effects.push(Effect::StopListener(target));
    }

    pub(crate) fn stop_pane(&mut self, target: ActorRef<EventPane>) {
        self.effects.push(Effect::StopPane(target));
    }

    pub(crate) fn stop_child_signal(&mut self, target: ActorRef<ChildSignal>) {
        self.effects.push(Effect::StopChildSignal(target));
    }

    pub(crate) fn stop_protocol(&mut self, target: ActorRef<ProtocolClient>) {
        self.effects.push(Effect::StopProtocol(target));
    }
}

#[derive(Clone, Debug)]
pub(crate) struct IoRecipient {
    target: IoTarget,
}

#[derive(Clone, Debug)]
enum IoTarget {
    Listener {
        target: WeakActorRef<Listener>,
    },
    Pane {
        target: WeakActorRef<EventPane>,
    },
    ChildSignal {
        target: WeakActorRef<ChildSignal>,
    },
    Protocol {
        target: WeakActorRef<ProtocolClient>,
        side: ProtocolIoSide,
    },
}

/// References returned when a Unix listener is added to the loop.
pub(crate) struct ListenerHandle {
    listener: ActorRef<Listener>,
    accepted: AcceptedClients,
}

pub(crate) struct PaneHandle {
    pane: ActorRef<EventPane>,
    runtime_id: u64,
}

pub(crate) struct ChildSignalHandle {
    signal: ActorRef<ChildSignal>,
}

/// References returned when a protocol client is added to the loop.
pub(crate) struct ProtocolHandle {
    protocol: ActorRef<ProtocolClient>,
    status: ProtocolStatus,
}

impl ProtocolHandle {
    pub(crate) fn is_alive(&self) -> bool {
        self.protocol.is_alive()
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
    pub(crate) fn is_control(&self) -> bool {
        self.protocol
            .with(ProtocolClient::is_control)
            .unwrap_or(false)
    }

    #[cfg(test)]
    pub(crate) fn is_attach(&self) -> bool {
        self.protocol
            .with(ProtocolClient::is_attach)
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

impl PaneHandle {
    pub(crate) fn runtime_id(&self) -> u64 {
        self.runtime_id
    }

    pub(crate) fn is_alive(&self) -> bool {
        self.pane.is_alive()
    }
}

impl ChildSignalHandle {
    pub(crate) fn is_alive(&self) -> bool {
        self.signal.is_alive()
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
    timers: TimerQueue<Envelope>,
    expired_timers: Vec<ExpiredTimer<Envelope>>,
    background_commands: Option<ActorRef<BackgroundCommands>>,
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
            timers: TimerQueue::new(),
            expired_timers: Vec::new(),
            background_commands: None,
        }
    }

    pub(crate) fn add_protocol(
        &mut self,
        reader: ImsgReader,
        writer: NonblockingImsgWriter,
        server: Server,
    ) -> ProtocolHandle {
        let background_commands = self
            .background_commands
            .get_or_insert_with(|| {
                ActorRef::new(BackgroundCommands::new(
                    server.state(),
                    server.status_hub(),
                    self.reactor.wake_handle(),
                ))
            })
            .clone();
        let (protocol, status) = ProtocolClient::new(reader, writer, server, background_commands);
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

    pub(crate) fn add_pane(&mut self, runtime_id: u64, io: PaneIo) -> PaneHandle {
        let pane = ActorRef::new(EventPane::new(io));
        self.events.push_back(Envelope::Pane {
            target: pane.clone(),
            event: PaneEvent::Start,
        });
        PaneHandle { pane, runtime_id }
    }

    pub(crate) fn add_child_signal(&mut self, server: Server) -> io::Result<ChildSignalHandle> {
        let signal = ActorRef::new(ChildSignal::new(server)?);
        self.events.push_back(Envelope::ChildSignal {
            target: signal.clone(),
            event: ChildSignalEvent::Start,
        });
        Ok(ChildSignalHandle { signal })
    }

    pub(crate) fn sync_pane(&mut self, target: &PaneHandle) -> io::Result<()> {
        if let Some(interest) = target
            .pane
            .with_mut(EventPane::take_interest_change)
            .flatten()
        {
            self.set_pane_interest(&target.pane, interest)?;
        }
        Ok(())
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

    pub(crate) fn shutdown_pane(&mut self, target: &PaneHandle) {
        let should_enqueue = target
            .pane
            .with_mut(EventPane::request_shutdown)
            .unwrap_or(false);
        if should_enqueue {
            self.events.push_back(Envelope::Pane {
                target: target.pane.clone(),
                event: PaneEvent::Shutdown,
            });
        }
    }

    pub(crate) fn shutdown_child_signal(&mut self, target: &ChildSignalHandle) {
        let should_enqueue = target
            .signal
            .with_mut(ChildSignal::request_shutdown)
            .unwrap_or(false);
        if should_enqueue {
            self.events.push_back(Envelope::ChildSignal {
                target: target.signal.clone(),
                event: ChildSignalEvent::Shutdown,
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
        self.enqueue_background_completions();
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
        let timer_timeout = self.timers.time_until_next(Instant::now());
        let timeout = match (timeout, timer_timeout) {
            (Some(requested), Some(timer)) => Some(requested.min(timer)),
            (requested, None) => requested,
            (None, timer) => timer,
        };
        let result = self.reactor.poll(timeout, &mut self.ready)?;
        let mut ready = std::mem::take(&mut self.ready);
        for notification in ready.drain(..) {
            self.enqueue_readiness(notification);
        }
        self.ready = ready;
        self.timers
            .drain_expired(Instant::now(), &mut self.expired_timers);
        self.events
            .extend(self.expired_timers.drain(..).map(ExpiredTimer::into_value));
        self.enqueue_background_completions();
        Ok(result)
    }

    fn enqueue_background_completions(&mut self) {
        let Some(target) = self.background_commands.as_ref() else {
            return;
        };
        let events = target
            .with(BackgroundCommands::take_completions)
            .unwrap_or_default();
        self.events
            .extend(events.into_iter().map(|event| Envelope::Background {
                target: target.clone(),
                event,
            }));
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
            IoTarget::Pane { target: recipient } => {
                let Some(target) = recipient.upgrade() else {
                    return;
                };
                let should_enqueue = target
                    .with_mut(EventPane::mark_work_queued)
                    .unwrap_or(false);
                if should_enqueue {
                    self.events.push_back(Envelope::Pane {
                        target,
                        event: PaneEvent::Ready(notification.readiness()),
                    });
                }
            }
            IoTarget::ChildSignal { target: recipient } => {
                let Some(target) = recipient.upgrade() else {
                    return;
                };
                let should_enqueue = target
                    .with_mut(ChildSignal::mark_work_queued)
                    .unwrap_or(false);
                if should_enqueue {
                    self.events.push_back(Envelope::ChildSignal {
                        target,
                        event: ChildSignalEvent::Readable,
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
                    ProtocolIoSide::Control(source) => ProtocolEvent::ControlReady(*source),
                    ProtocolIoSide::Attach(source) => ProtocolEvent::AttachReady(*source),
                };
                self.events.push_back(Envelope::Protocol { target, event });
            }
        }
    }

    fn apply(&mut self, effect: Effect) -> io::Result<()> {
        match effect {
            Effect::Enqueue(envelope) => self.events.push_back(envelope),
            Effect::SetListenerInterest { target, enabled } => {
                self.set_listener_interest(&target, enabled)?;
            }
            Effect::SetPaneInterest { target, interest } => {
                self.set_pane_interest(&target, interest)?;
            }
            Effect::SetChildSignalInterest { target, enabled } => {
                self.set_child_signal_interest(&target, enabled)?;
            }
            Effect::SetProtocolInterest {
                target,
                side,
                enabled,
            } => {
                self.set_protocol_interest(&target, side, enabled)?;
            }
            Effect::SetProtocolTimer {
                target,
                deadline,
                event,
            } => {
                self.cancel_protocol_timer(&target);
                let timer = self.timers.set(
                    deadline,
                    Envelope::Protocol {
                        target: target.clone(),
                        event,
                    },
                );
                target.with_mut(|client| client.set_timer(Some(timer)));
            }
            Effect::CancelProtocolTimer { target } => {
                self.cancel_protocol_timer(&target);
            }
            Effect::StopListener(target) => {
                target.stop();
            }
            Effect::StopPane(target) => {
                target.stop();
            }
            Effect::StopChildSignal(target) => {
                target.stop();
            }
            Effect::StopProtocol(target) => {
                target.stop();
            }
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

    fn set_pane_interest(
        &mut self,
        target: &ActorRef<EventPane>,
        interest: PaneInterest,
    ) -> io::Result<()> {
        let token = target.with(EventPane::token).flatten();
        let reactor_interest = match interest {
            PaneInterest::Disabled => {
                if let Some(token) = token {
                    self.reactor.deregister(token)?;
                    target.with_mut(|pane| pane.set_token(None));
                }
                return Ok(());
            }
            PaneInterest::Readable => Interest::READABLE,
            PaneInterest::ReadableWritable => Interest::READABLE | Interest::WRITABLE,
        };

        match token {
            None => {
                let recipient = IoRecipient {
                    target: IoTarget::Pane {
                        target: target.downgrade(),
                    },
                };
                if let Some(result) = target.with_mut(|pane| {
                    let token = self
                        .reactor
                        .register(pane.fd(), reactor_interest, recipient)?;
                    pane.set_token(Some(token));
                    Ok::<(), io::Error>(())
                }) {
                    result?;
                }
            }
            Some(token) => {
                self.reactor.reregister(token, reactor_interest)?;
            }
        }
        Ok(())
    }

    fn set_child_signal_interest(
        &mut self,
        target: &ActorRef<ChildSignal>,
        enabled: bool,
    ) -> io::Result<()> {
        let token = target.with(ChildSignal::token).flatten();
        match (enabled, token) {
            (true, None) => {
                let recipient = IoRecipient {
                    target: IoTarget::ChildSignal {
                        target: target.downgrade(),
                    },
                };
                if let Some(result) = target.with_mut(|signal| {
                    let token =
                        self.reactor
                            .register(signal.fd(), Interest::READABLE, recipient)?;
                    signal.set_token(Some(token));
                    Ok::<(), io::Error>(())
                }) {
                    result?;
                }
            }
            (false, Some(token)) => {
                self.reactor.deregister(token)?;
                target.with_mut(|signal| signal.set_token(None));
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
                            ProtocolIoSide::Control(source) => {
                                if ProtocolClient::control_source_is_writable(source) {
                                    Interest::WRITABLE
                                } else {
                                    Interest::READABLE
                                }
                            }
                            ProtocolIoSide::Attach(source) => {
                                if ProtocolClient::attach_source_is_writable(source) {
                                    Interest::WRITABLE
                                } else {
                                    Interest::READABLE
                                }
                            }
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

    fn cancel_protocol_timer(&mut self, target: &ActorRef<ProtocolClient>) {
        let timer = target.with(ProtocolClient::timer).flatten();
        if let Some(timer) = timer {
            self.timers.cancel(timer);
            target.with_mut(|client| client.set_timer(None));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

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
    fn turn_skips_poll_while_dispatch_budget_leaves_queued_work() {
        let path = ListenerPath::new();
        let listener = UnixListener::bind(&path.0).unwrap();
        let mut loop_ = EventLoop::new().unwrap();
        let listener = loop_.add_listener(listener, 1).unwrap();
        loop_.shutdown_listener(&listener);

        let result = loop_.run_turn(Some(POLL_TIMEOUT), 1).unwrap();

        assert_eq!(result.dispatched(), 1);
        assert_eq!(result.poll_result(), None);
        assert_eq!(loop_.pending_events(), 1);

        dispatch_all(&mut loop_);
        assert!(!listener.is_alive());
    }
}
