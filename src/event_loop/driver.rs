//! Central FIFO dispatch and deferred reactor effects.

use std::collections::VecDeque;
use std::io;
use std::os::fd::AsRawFd;
use std::time::{Duration, Instant};

use crate::server::pane::PaneIo;
use crate::server::task::{FdDirection, WaitToken};
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
use super::suspend::{
    ExecutorEvent, JobRegistration, SuspensionExecutor, SuspensionExecutorHandle,
};
use super::term_signal::{TermSignal, TermSignalEvent};
use super::timer::{ExpiredTimer, TimerQueue};
use crate::server::pane::PanePipeIo;
use crate::server::status::FormatJob;

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
    TermSignal {
        target: ActorRef<TermSignal>,
        event: TermSignalEvent,
    },
    Protocol {
        target: ActorRef<ProtocolClient>,
        event: ProtocolEvent,
    },
    Background {
        target: ActorRef<BackgroundCommands>,
        event: JobEvent,
    },
    Executor {
        target: ActorRef<SuspensionExecutor>,
        event: ExecutorEvent,
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
            Envelope::TermSignal { target, event } => {
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
            // The executor never changes its own registrations; the loop
            // reconciles those once per dispatch instead. It does address the
            // background-command actor, which owns the queues it finishes.
            Envelope::Executor { target, event } => {
                target.with_mut(|executor| executor.handle(event, outbox));
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
    SetTermSignalInterest {
        target: ActorRef<TermSignal>,
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
    SetPaneTimer {
        target: ActorRef<EventPane>,
        deadline: Instant,
    },
    CancelPaneTimer {
        target: ActorRef<EventPane>,
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

    pub(crate) fn set_term_signal_interest(&mut self, target: ActorRef<TermSignal>, enabled: bool) {
        self.effects
            .push(Effect::SetTermSignalInterest { target, enabled });
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

    /// Arm the pane's ground timer, tmux's five seconds of patience with a
    /// string sequence whose terminator has not arrived.
    pub(crate) fn set_pane_timer(&mut self, target: ActorRef<EventPane>, deadline: Instant) {
        self.effects.push(Effect::SetPaneTimer { target, deadline });
    }

    pub(crate) fn cancel_pane_timer(&mut self, target: ActorRef<EventPane>) {
        self.effects.push(Effect::CancelPaneTimer { target });
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
    TermSignal {
        target: WeakActorRef<TermSignal>,
    },
    Protocol {
        target: WeakActorRef<ProtocolClient>,
        side: ProtocolIoSide,
    },
    Executor {
        target: WeakActorRef<SuspensionExecutor>,
        job: u64,
        source: WaitToken,
    },
}

/// References returned when a Unix listener is added to the loop.
pub(crate) struct ListenerHandle {
    listener: ActorRef<Listener>,
    accepted: AcceptedClients,
}

pub(crate) struct PaneHandle {
    pane: ActorRef<EventPane>,
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
    pub(crate) fn is_alive(&self) -> bool {
        self.pane.is_alive()
    }
}

impl ChildSignalHandle {
    pub(crate) fn is_alive(&self) -> bool {
        self.signal.is_alive()
    }
}

/// Metadata for one event-loop turn. Test scaffolding for `run_turn`.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct TurnResult {
    dispatched: usize,
    poll: Option<PollResult>,
}

#[cfg(test)]
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
    executor: ActorRef<SuspensionExecutor>,
    executor_handle: SuspensionExecutorHandle,
    term_signal: Option<ActorRef<TermSignal>>,
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
        let (executor, executor_handle) = SuspensionExecutor::new();
        Self {
            reactor,
            events: VecDeque::new(),
            ready: Vec::new(),
            timers: TimerQueue::new(),
            expired_timers: Vec::new(),
            background_commands: None,
            executor: ActorRef::new(executor),
            executor_handle,
            term_signal: None,
        }
    }

    pub(crate) fn add_protocol(
        &mut self,
        reader: ImsgReader,
        writer: NonblockingImsgWriter,
        server: Server,
        peer_uid: Option<u32>,
    ) -> ProtocolHandle {
        let executor_handle = self.executor_handle.clone();
        let background_commands = self
            .background_commands
            .get_or_insert_with(|| {
                ActorRef::new(BackgroundCommands::new(
                    server.state(),
                    server.status_hub(),
                    self.executor.clone(),
                    executor_handle.clone(),
                ))
            })
            .clone();
        let (protocol, status) = ProtocolClient::new(
            reader,
            writer,
            server,
            background_commands,
            executor_handle,
            peer_uid,
        );
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

    pub(crate) fn add_pane(&mut self, io: PaneIo) -> PaneHandle {
        let pane = ActorRef::new(EventPane::new(io));
        self.events.push_back(Envelope::Pane {
            target: pane.clone(),
            event: PaneEvent::Start,
        });
        PaneHandle { pane }
    }

    /// Watch for `SIGINT`/`SIGTERM` on the loop. The source lives as long as
    /// the loop does; the first signal ends the process.
    pub(crate) fn add_term_signal(&mut self) -> io::Result<()> {
        let signal = ActorRef::new(TermSignal::new()?);
        self.events.push_back(Envelope::TermSignal {
            target: signal.clone(),
            event: TermSignalEvent::Start,
        });
        self.term_signal = Some(signal);
        Ok(())
    }

    pub(crate) fn add_child_signal(&mut self, server: Server) -> io::Result<ChildSignalHandle> {
        let signal = ActorRef::new(ChildSignal::new(server)?);
        self.events.push_back(Envelope::ChildSignal {
            target: signal.clone(),
            event: ChildSignalEvent::Start,
        });
        Ok(ChildSignalHandle { signal })
    }

    /// Adopt every `#()` job launched since the last pass. Format expansion
    /// runs deep inside rendering, so the jobs are collected there and handed
    /// to the loop here.
    /// Hand detached command work to the loop. The loop's own passes raise
    /// hook bodies with no client behind them; they run as background queues,
    /// the same way a `-b` command does.
    pub(crate) fn adopt_background_commands(
        &mut self,
        server: &Server,
        requests: Vec<crate::server::command::BackgroundCommandRequest>,
    ) -> io::Result<()> {
        if requests.is_empty() {
            return Ok(());
        }
        let executor_handle = self.executor_handle.clone();
        let target = self
            .background_commands
            .get_or_insert_with(|| {
                ActorRef::new(BackgroundCommands::new(
                    server.state(),
                    server.status_hub(),
                    self.executor.clone(),
                    executor_handle,
                ))
            })
            .clone();
        for request in requests {
            self.events.push_back(Envelope::Background {
                target: target.clone(),
                event: JobEvent::Start(request),
            });
        }
        Ok(())
    }

    pub(crate) fn adopt_format_jobs(&mut self, jobs: Vec<FormatJob>) -> io::Result<()> {
        if jobs.is_empty() {
            return Ok(());
        }
        let executor = self.executor.clone();
        let mut outbox = Outbox::new();
        executor.with_mut(|executor| {
            for job in jobs {
                executor.adopt_format_job(job, &mut outbox);
            }
        });
        for effect in outbox.effects {
            self.apply(effect)?;
        }
        Ok(())
    }

    /// Adopt every `pipe-pane` child opened since the last pass.
    pub(crate) fn adopt_pane_pipes(&mut self, pipes: Vec<PanePipeIo>) -> io::Result<()> {
        if pipes.is_empty() {
            return Ok(());
        }
        let executor = self.executor.clone();
        let mut outbox = Outbox::new();
        executor.with_mut(|executor| {
            for pipe in pipes {
                executor.adopt_pane_pipe(pipe, &mut outbox);
            }
        });
        for effect in outbox.effects {
            self.apply(effect)?;
        }
        Ok(())
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

    pub(crate) fn pending_events(&self) -> usize {
        self.events.len()
    }

    #[cfg(test)]
    pub(crate) fn executor_handle(&self) -> SuspensionExecutorHandle {
        self.executor_handle.clone()
    }

    pub(crate) fn dispatch_one(&mut self) -> io::Result<bool> {
        self.sync_executor()?;
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
        self.sync_executor()?;
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
        Ok(result)
    }

    /// Adopt newly submitted suspension jobs, then make the reactor and the
    /// timer queue describe exactly what every live job is waiting for.
    fn sync_executor(&mut self) -> io::Result<()> {
        let executor = self.executor.clone();
        // A job adopted here can finish on its very first turn and address the
        // background-command actor, so collect effects and apply them once the
        // executor's borrow is released.
        let mut outbox = Outbox::new();
        let result = executor
            .with_mut(|executor| -> io::Result<()> {
                executor.adopt_submitted(&mut outbox);
                for id in executor.job_ids() {
                    let Some(state) = executor.job_mut(id) else {
                        continue;
                    };
                    if state.is_finished() {
                        // The job already reported its result to the suspended
                        // command queue; only its registrations are left.
                        for registration in std::mem::take(&mut state.registrations) {
                            self.reactor.deregister(registration.token)?;
                        }
                        if let Some((_, timer)) = state.timer.take() {
                            self.timers.cancel(timer);
                        }
                        executor.remove_job(id);
                        continue;
                    }
                    let Some(state) = executor.job_mut(id) else {
                        continue;
                    };
                    let Some(wait) = state.task.wait() else {
                        continue;
                    };
                    let sources = wait
                        .sources()
                        .iter()
                        .map(|source| (source.token(), source.fd().as_raw_fd(), source.direction()))
                        .collect::<Vec<_>>();
                    if !state.is_registered_for(&sources) {
                        for registration in std::mem::take(&mut state.registrations) {
                            self.reactor.deregister(registration.token)?;
                        }
                        for source in wait.sources() {
                            let recipient = IoRecipient {
                                target: IoTarget::Executor {
                                    target: self.executor.downgrade(),
                                    job: id,
                                    source: source.token(),
                                },
                            };
                            let interest = match source.direction() {
                                FdDirection::Read => Interest::READABLE,
                                FdDirection::Write => Interest::WRITABLE,
                            };
                            let token = self.reactor.register(source.fd(), interest, recipient)?;
                            state.registrations.push(JobRegistration {
                                source: source.token(),
                                fd: source.fd().as_raw_fd(),
                                direction: source.direction(),
                                token,
                            });
                        }
                    }
                    match (wait.deadline(), state.timer) {
                        (Some(deadline), Some((armed, _))) if armed == deadline => {}
                        (Some(deadline), previous) => {
                            if let Some((_, timer)) = previous {
                                self.timers.cancel(timer);
                            }
                            let timer = self.timers.set(
                                deadline,
                                Envelope::Executor {
                                    target: self.executor.clone(),
                                    event: ExecutorEvent::Timeout { job: id },
                                },
                            );
                            state.timer = Some((deadline, timer));
                        }
                        (None, Some((_, timer))) => {
                            self.timers.cancel(timer);
                            state.timer = None;
                        }
                        (None, None) => {}
                    }
                }
                Ok(())
            })
            .unwrap_or(Ok(()));
        for effect in outbox.effects {
            self.apply(effect)?;
        }
        result
    }

    /// Drive one dispatch-then-poll turn. Test scaffolding; the server runs
    /// the loop through [`EventLoop::run`].
    #[cfg(test)]
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
            IoTarget::TermSignal { target: recipient } => {
                let Some(target) = recipient.upgrade() else {
                    return;
                };
                let should_enqueue = target
                    .with_mut(TermSignal::mark_work_queued)
                    .unwrap_or(false);
                if should_enqueue {
                    self.events.push_back(Envelope::TermSignal {
                        target,
                        event: TermSignalEvent::Readable,
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
            IoTarget::Executor {
                target: recipient,
                job,
                source,
            } => {
                let Some(target) = recipient.upgrade() else {
                    return;
                };
                self.events.push_back(Envelope::Executor {
                    target,
                    event: ExecutorEvent::Ready {
                        job: *job,
                        source: *source,
                    },
                });
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
            Effect::SetTermSignalInterest { target, enabled } => {
                self.set_term_signal_interest(&target, enabled)?;
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
            Effect::SetPaneTimer { target, deadline } => {
                self.cancel_pane_timer(&target);
                let timer = self.timers.set(
                    deadline,
                    Envelope::Pane {
                        target: target.clone(),
                        event: PaneEvent::GroundTimer,
                    },
                );
                target.with_mut(|pane| pane.set_timer(Some(timer)));
            }
            Effect::CancelPaneTimer { target } => {
                self.cancel_pane_timer(&target);
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

    fn set_term_signal_interest(
        &mut self,
        target: &ActorRef<TermSignal>,
        enabled: bool,
    ) -> io::Result<()> {
        let token = target.with(TermSignal::token).flatten();
        match (enabled, token) {
            (true, None) => {
                let recipient = IoRecipient {
                    target: IoTarget::TermSignal {
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
                    match result {
                        Ok(()) => {}
                        // A descriptor with no poll operation — a client that
                        // redirected its output to `/dev/null` is the usual
                        // case — is rejected by `epoll_ctl` with EPERM. Such a
                        // descriptor is never *not* ready, so serve it directly
                        // instead of taking the server down with the client.
                        Err(error) if error.raw_os_error() == Some(libc::EPERM) => {
                            self.enqueue_protocol_ready(target, side);
                        }
                        Err(error) => return Err(error),
                    }
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

    /// Deliver a readiness event for a protocol side that cannot be polled.
    fn enqueue_protocol_ready(&mut self, target: &ActorRef<ProtocolClient>, side: ProtocolIoSide) {
        if !target
            .with_mut(|client| client.mark_work_queued(side))
            .unwrap_or(false)
        {
            return;
        }
        let event = match side {
            ProtocolIoSide::Read => ProtocolEvent::Readable,
            ProtocolIoSide::Write => ProtocolEvent::Writable,
            ProtocolIoSide::Command => ProtocolEvent::CommandCompleted,
            ProtocolIoSide::Control(source) => ProtocolEvent::ControlReady(source),
            ProtocolIoSide::Attach(source) => ProtocolEvent::AttachReady(source),
        };
        self.events.push_back(Envelope::Protocol {
            target: target.clone(),
            event,
        });
    }

    fn cancel_protocol_timer(&mut self, target: &ActorRef<ProtocolClient>) {
        let timer = target.with(ProtocolClient::timer).flatten();
        if let Some(timer) = timer {
            self.timers.cancel(timer);
            target.with_mut(|client| client.set_timer(None));
        }
    }

    fn cancel_pane_timer(&mut self, target: &ActorRef<EventPane>) {
        let timer = target.with(EventPane::timer).flatten();
        if let Some(timer) = timer {
            self.timers.cancel(timer);
            target.with_mut(|pane| pane.set_timer(None));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use crate::server::command::{
        ClientContext, ClientFileWrite, CommandSuspension, CommandSuspensionResult,
    };
    use crate::server::task::TaskState;

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

    /// Run loop turns until a submitted suspension reports its result.
    fn resolve_on_loop(
        loop_: &mut EventLoop<MioReactor<IoRecipient>>,
        suspension: CommandSuspension,
    ) -> CommandSuspensionResult {
        let completion = loop_
            .executor_handle()
            .submit(suspension)
            .expect("completion pair");
        let mut task = TaskState::new(completion);
        let deadline = Instant::now() + Duration::from_secs(10);
        while !task.poll_after_single_source_wakeup(true, Instant::now()) {
            assert!(Instant::now() < deadline, "suspension never completed");
            // The completion descriptor belongs to the caller, not to this
            // loop, so keep turns short instead of blocking on the reactor.
            loop_.run_turn(Some(Duration::from_millis(10)), 64).unwrap();
        }
        task.take_output()
            .expect("completed suspension")
            .expect("suspension result")
    }

    fn run_shell(args: &[&str]) -> CommandSuspension {
        CommandSuspension::RunShell {
            args: std::iter::once("run-shell")
                .chain(args.iter().copied())
                .map(str::to_string)
                .collect(),
            context: ClientContext::default(),
        }
    }

    #[test]
    fn executor_resolves_a_shell_suspension_without_a_thread() {
        let mut loop_ = EventLoop::new().unwrap();

        let result = resolve_on_loop(&mut loop_, run_shell(&["echo hello; exit 3"]));

        let CommandSuspensionResult::RunShell(completion) = result else {
            panic!("run-shell suspension resolved as another variant");
        };
        assert_eq!(completion.result().exit, 3);
        assert_eq!(
            completion.result().stdout,
            "hello\n'echo hello; exit 3' returned 3\n"
        );
    }

    #[test]
    fn executor_arms_a_loop_timer_for_a_delayed_shell_suspension() {
        let mut loop_ = EventLoop::new().unwrap();
        let started = Instant::now();

        let result = resolve_on_loop(&mut loop_, run_shell(&["-d", "0.2", "echo late"]));

        let CommandSuspensionResult::RunShell(completion) = result else {
            panic!("run-shell suspension resolved as another variant");
        };
        assert!(started.elapsed() >= Duration::from_millis(150));
        assert_eq!(completion.result().stdout, "late\n");
        assert!(loop_.timers.is_empty(), "the job's timer outlived the job");
    }

    #[test]
    fn executor_reads_a_fifo_source_file_on_the_loop() {
        let mut loop_ = EventLoop::new().unwrap();
        let path = ListenerPath::new();
        let raw = std::ffi::CString::new(path.0.as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(raw.as_ptr(), 0o600) }, 0);
        let writer = std::thread::spawn({
            let path = path.0.clone();
            move || {
                std::thread::sleep(Duration::from_millis(50));
                std::fs::write(&path, b"set-buffer -b sourced yes\n").unwrap();
            }
        });

        let result = resolve_on_loop(
            &mut loop_,
            CommandSuspension::SourceFile {
                paths: vec![path.0.display().to_string()],
            },
        );

        writer.join().unwrap();
        let CommandSuspensionResult::SourceFile(reads) = result else {
            panic!("source-file suspension resolved as another variant");
        };
        assert_eq!(
            reads[0].contents().expect("FIFO contents"),
            "set-buffer -b sourced yes\n"
        );
    }

    #[test]
    fn executor_writes_a_fifo_save_buffer_on_the_loop() {
        let mut loop_ = EventLoop::new().unwrap();
        let path = ListenerPath::new();
        let raw = std::ffi::CString::new(path.0.as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(raw.as_ptr(), 0o600) }, 0);
        // More than a pipe buffer, so the write only completes as the reader
        // drains it — the loop has to watch for writability, not just once.
        let payload = vec![b'x'; 512 * 1024];
        let reader = std::thread::spawn({
            let path = path.0.clone();
            move || {
                std::thread::sleep(Duration::from_millis(50));
                std::fs::read(&path).unwrap()
            }
        });

        let result = resolve_on_loop(
            &mut loop_,
            CommandSuspension::SaveBuffer {
                request: ClientFileWrite {
                    path: path.0.clone(),
                    display_path: path.0.display().to_string(),
                    flags: libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
                    data: payload.clone(),
                },
            },
        );

        let CommandSuspensionResult::SaveBuffer(result) = result else {
            panic!("save-buffer suspension resolved as another variant");
        };
        assert_eq!(result.exit, 0, "{}", result.stderr);
        assert_eq!(reader.join().unwrap(), payload);
    }

    #[test]
    fn executor_writes_a_fifo_save_buffer_a_reader_drains_late() {
        use std::os::unix::fs::OpenOptionsExt;

        let mut loop_ = EventLoop::new().unwrap();
        let path = ListenerPath::new();
        let raw = std::ffi::CString::new(path.0.as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(raw.as_ptr(), 0o600) }, 0);
        // Many pipe buffers' worth, with the reader attached from the start but
        // idle: the job's very first write fills the pipe, so everything after
        // it depends on the loop rearming writable interest.
        let payload = vec![b'x'; 8 * 1024 * 1024];
        let reader = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(&path.0)
            .unwrap();
        let drained = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            let flags = unsafe { libc::fcntl(reader.as_raw_fd(), libc::F_GETFL) };
            assert_eq!(
                unsafe {
                    libc::fcntl(reader.as_raw_fd(), libc::F_SETFL, flags & !libc::O_NONBLOCK)
                },
                0
            );
            let mut contents = Vec::new();
            let mut reader = reader;
            std::io::Read::read_to_end(&mut reader, &mut contents).unwrap();
            contents
        });

        let result = resolve_on_loop(
            &mut loop_,
            CommandSuspension::SaveBuffer {
                request: ClientFileWrite {
                    path: path.0.clone(),
                    display_path: path.0.display().to_string(),
                    flags: libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
                    data: payload.clone(),
                },
            },
        );

        let CommandSuspensionResult::SaveBuffer(result) = result else {
            panic!("save-buffer suspension resolved as another variant");
        };
        assert_eq!(result.exit, 0, "{}", result.stderr);
        assert_eq!(drained.join().unwrap(), payload);
    }

    #[test]
    fn executor_releases_every_registration_once_a_job_finishes() {
        let mut loop_ = EventLoop::new().unwrap();

        resolve_on_loop(&mut loop_, run_shell(&["true"]));
        // The completed job is retired by the next sync, not by the poll that
        // produced its result.
        loop_.dispatch_one().unwrap();

        assert_eq!(
            loop_
                .executor
                .with(|executor| executor.job_ids().len())
                .unwrap(),
            0
        );
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
