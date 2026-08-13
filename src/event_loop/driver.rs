//! Central FIFO dispatch and deferred reactor effects.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::server::pane::PaneIo;
use hmux_rt::WakeFn;
use crate::server::Server;
use crate::tmux::codec::{ImsgReader, NonblockingImsgWriter};

use super::actor::{ActorRef, WeakActorRef};
use super::job::{BackgroundCommands, JobEvent};
use super::listener::{AcceptedClients, Listener, ListenerEvent};
use super::pane::{EventPane, PaneEvent, PaneInterest};
use super::protocol::{
    ProtocolClient, ProtocolCloseReason, ProtocolEvent, ProtocolIoSide, ProtocolStatus,
};
use hmux_rt::{Interest, MioReactor, PollResult, Reactor, Ready};
use hmux_rt::{TaskEvent, TaskHandle, TaskId, TaskLoop};
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
    Protocol {
        target: ActorRef<ProtocolClient>,
        event: ProtocolEvent,
    },
    Background {
        target: ActorRef<BackgroundCommands>,
        event: JobEvent,
    },
    /// Work for the loop's own task set, which is not an actor: the loop
    /// dispatches it directly.
    Task {
        event: TaskEvent,
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
            Envelope::Protocol { target, event } => {
                let dispatch_target = target.clone();
                target.with_mut(|client| client.handle(&dispatch_target, event, outbox));
            }
            Envelope::Background { target, event } => {
                let dispatch_target = target.clone();
                target.with_mut(|jobs| jobs.handle(&dispatch_target, event, outbox));
            }
            // A task reaches the reactor and the timer queue only through the
            // inboxes the loop reconciles, so the loop dispatches it before
            // this is ever reached.
            Envelope::Task { .. } => {
                unreachable!("task events are dispatched by the loop, not the envelope")
            }
        }
    }
}

/// A wake that has fired but has not yet become an event.
///
/// Only the destination's address is recorded. Resolving it — upgrading that
/// address and deduplicating against work already queued — touches the
/// destination actor, and a wake fires from inside its *producer's* dispatch,
/// where the producer may be the very actor being woken: a client that answers
/// its own `command-prompt -w` does exactly that. Deferring the resolution to
/// the drain keeps the wake itself from reaching into any actor.
enum PendingWake {
    Protocol {
        target: WeakActorRef<ProtocolClient>,
        side: ProtocolIoSide,
    },
    /// A task parked on something that is not a descriptor — the loop's own
    /// task set is the destination, so no address is needed.
    Task {
        task: TaskId,
    },
    ProtocolTimer {
        target: WeakActorRef<ProtocolClient>,
        event: ProtocolEvent,
    },
    PaneTimer {
        target: WeakActorRef<EventPane>,
        generation: u64,
    },
    Background {
        target: WeakActorRef<BackgroundCommands>,
        id: u64,
    },
}

/// Where a fired [`WakeFn`] leaves its event.
///
/// Dispatch order is observable behavior, so a wake never resumes its consumer
/// inline; it queues here and the loop picks it up like anything else. Draining
/// into the main FIFO before each dispatch keeps one order over everything the
/// loop does, rather than a second, competing one.
#[derive(Clone, Default)]
pub(crate) struct WakeQueue {
    fired: Rc<RefCell<VecDeque<PendingWake>>>,
}

impl WakeQueue {
    /// The wake for a client whose command queue is parked on a suspension.
    fn protocol_wake(&self, target: WeakActorRef<ProtocolClient>, side: ProtocolIoSide) -> WakeFn {
        let fired = Rc::clone(&self.fired);
        Rc::new(move || {
            fired.borrow_mut().push_back(PendingWake::Protocol {
                target: target.clone(),
                side,
            });
        })
    }

    /// The wake for one piece of detached work the background-command actor is
    /// waiting on.
    pub(super) fn background_wake(
        &self,
        target: WeakActorRef<BackgroundCommands>,
        id: u64,
    ) -> WakeFn {
        let fired = Rc::clone(&self.fired);
        Rc::new(move || {
            fired.borrow_mut().push_back(PendingWake::Background {
                target: target.clone(),
                id,
            });
        })
    }

    /// The wake a task's [`LocalWaker`](std::task::LocalWaker) fires.
    pub(super) fn wake_task(&self, task: TaskId) {
        self.fired
            .borrow_mut()
            .push_back(PendingWake::Task { task });
    }

    fn fire_protocol_timer(&self, target: WeakActorRef<ProtocolClient>, event: ProtocolEvent) {
        self.fired
            .borrow_mut()
            .push_back(PendingWake::ProtocolTimer { target, event });
    }

    fn fire_pane_timer(&self, target: WeakActorRef<EventPane>, generation: u64) {
        self.fired
            .borrow_mut()
            .push_back(PendingWake::PaneTimer { target, generation });
    }

    fn len(&self) -> usize {
        self.fired.borrow().len()
    }

    /// Turn every fired wake into an event. Runs between dispatches, so
    /// reaching into a destination actor here is safe.
    fn drain_into(&self, events: &mut VecDeque<Envelope>) {
        // The borrow ends before any actor is touched: resolving one wake can
        // fire another, and that wake has to find this queue free.
        let fired = std::mem::take(&mut *self.fired.borrow_mut());
        for wake in fired {
            match wake {
                PendingWake::Protocol { target, side } => {
                    let Some(target) = target.upgrade() else {
                        continue;
                    };
                    // Same dedup a readiness notification gets: a client woken
                    // twice before it runs is one turn's work.
                    let should_enqueue = target
                        .with_mut(|client| client.mark_work_queued(side))
                        .unwrap_or(false);
                    if !should_enqueue {
                        continue;
                    }
                    events.push_back(Envelope::Protocol {
                        target,
                        event: protocol_ready_event(side),
                    });
                }
                PendingWake::Background { target, id } => {
                    let Some(target) = target.upgrade() else {
                        continue;
                    };
                    events.push_back(Envelope::Background {
                        target,
                        event: JobEvent::Ready { id },
                    });
                }
                PendingWake::Task { task } => {
                    events.push_back(Envelope::Task {
                        event: TaskEvent::Poll(task),
                    });
                }
                PendingWake::ProtocolTimer { target, event } => {
                    let Some(target) = target.upgrade() else {
                        continue;
                    };
                    events.push_back(Envelope::Protocol { target, event });
                }
                PendingWake::PaneTimer { target, generation } => {
                    let Some(target) = target.upgrade() else {
                        continue;
                    };
                    events.push_back(Envelope::Pane {
                        target,
                        event: PaneEvent::GroundTimer(generation),
                    });
                }
            }
        }
    }
}

/// The event a protocol client gets when `side` has work.
fn protocol_ready_event(side: ProtocolIoSide) -> ProtocolEvent {
    match side {
        ProtocolIoSide::Read => ProtocolEvent::Readable,
        ProtocolIoSide::Write => ProtocolEvent::Writable,
        ProtocolIoSide::Command => ProtocolEvent::CommandCompleted,
        ProtocolIoSide::Control(source) => ProtocolEvent::ControlReady(source),
        ProtocolIoSide::Attach(source) => ProtocolEvent::AttachReady(source),
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
    SetPaneTimer {
        target: ActorRef<EventPane>,
        deadline: Instant,
        generation: u64,
    },
    StopListener(ActorRef<Listener>),
    StopPane(ActorRef<EventPane>),
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

    /// Arm the pane's ground timer, tmux's five seconds of patience with a
    /// string sequence whose terminator has not arrived.
    pub(crate) fn set_pane_timer(
        &mut self,
        target: ActorRef<EventPane>,
        deadline: Instant,
        generation: u64,
    ) {
        self.effects.push(Effect::SetPaneTimer {
            target,
            deadline,
            generation,
        });
    }

    pub(crate) fn stop_listener(&mut self, target: ActorRef<Listener>) {
        self.effects.push(Effect::StopListener(target));
    }

    pub(crate) fn stop_pane(&mut self, target: ActorRef<EventPane>) {
        self.effects.push(Effect::StopPane(target));
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
    Protocol {
        target: WeakActorRef<ProtocolClient>,
        side: ProtocolIoSide,
    },
    /// One `AsyncFd`. The registration names the descriptor, not the task, so
    /// consecutive suspensions never share one and there is nothing to
    /// re-point.
    Task {
        io: u64,
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
    wakes: WakeQueue,
    ready: Vec<Ready<IoRecipient>>,
    background_commands: Option<ActorRef<BackgroundCommands>>,
    task_loop: TaskLoop,
    task_handle: TaskHandle,
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
        let wakes = WakeQueue::default();
        let task_wakes = wakes.clone();
        let task_loop = TaskLoop::new(Rc::new(move |task| task_wakes.wake_task(task)));
        let task_handle = task_loop.handle();
        Self {
            reactor,
            events: VecDeque::new(),
            wakes,
            ready: Vec::new(),
            background_commands: None,
            task_loop,
            task_handle,
        }
    }

    pub(crate) fn add_protocol(
        &mut self,
        reader: ImsgReader,
        writer: NonblockingImsgWriter,
        server: Server,
        peer_uid: Option<u32>,
    ) -> ProtocolHandle {
        let background_commands = self
            .background_commands
            .get_or_insert_with(|| {
                ActorRef::new(BackgroundCommands::new(
                    server.state(),
                    server.status_hub(),
                    self.wakes.clone(),
                    self.task_handle.clone(),
                ))
            })
            .clone();
        let (protocol, status) = ProtocolClient::new(
            reader,
            writer,
            server,
            background_commands,
            self.task_handle.clone(),
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

    /// Watch for `SIGINT`/`SIGTERM` on the loop. The task lives as long as
    /// the loop does; the first signal ends the process.
    pub(crate) fn add_term_signal(&mut self) -> io::Result<()> {
        super::term_signal::spawn(&self.task_handle)
    }

    pub(crate) fn add_child_signal(
        &mut self,
        server: Server,
    ) -> io::Result<super::process::ChildSignalHandle> {
        super::process::spawn(&self.task_handle, server)
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
        let target = self
            .background_commands
            .get_or_insert_with(|| {
                ActorRef::new(BackgroundCommands::new(
                    server.state(),
                    server.status_hub(),
                    self.wakes.clone(),
                    self.task_handle.clone(),
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
        for job in jobs {
            let tasks = self.task_handle.clone();
            self.task_handle.spawn(async move { job.run(&tasks).await });
        }
        Ok(())
    }

    /// Adopt every `pipe-pane` child opened since the last pass.
    pub(crate) fn adopt_pane_pipes(&mut self, pipes: Vec<PanePipeIo>) -> io::Result<()> {
        for pipe in pipes {
            let tasks = self.task_handle.clone();
            self.task_handle.spawn(async move { pipe.run(&tasks).await });
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

    /// Work the loop can do without waiting for the kernel. A fired wake counts
    /// even before it is drained, so the server never blocks in the reactor
    /// with a resumable task in hand.
    pub(crate) fn pending_events(&self) -> usize {
        self.events.len() + self.wakes.len()
    }

    /// Test scaffolding: the daemon's own actors get the handle at
    /// construction, so only a test ever asks the loop for it.
    #[cfg(test)]
    pub(crate) fn task_handle(&self) -> TaskHandle {
        self.task_handle.clone()
    }

    pub(crate) fn dispatch_one(&mut self) -> io::Result<bool> {
        self.sync_tasks()?;
        // After the syncs, which are what install the wakes that may fire.
        self.wakes.drain_into(&mut self.events);
        let Some(envelope) = self.events.pop_front() else {
            return Ok(false);
        };

        // The task set is the loop's own, not an actor: its events carry no
        // target and go straight to it.
        if let Envelope::Task { event } = envelope {
            self.task_loop.dispatch(event);
            return Ok(true);
        }

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
        self.sync_tasks()?;
        self.wakes.drain_into(&mut self.events);
        // A wake that fired during the sync is work in hand; blocking on the
        // kernel now would hold it until something unrelated happened.
        let timeout = if self.events.is_empty() {
            timeout
        } else {
            Some(Duration::ZERO)
        };
        let timer_timeout = self.task_loop.time_until_next_deadline(Instant::now());
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
        let events = &mut self.events;
        self.task_loop.drain_expired(Instant::now(), |event| {
            events.push_back(Envelope::Task { event });
        });
        Ok(result)
    }

    /// Reconcile the task set with the reactor and the loop's event queue.
    fn sync_tasks(&mut self) -> io::Result<()> {
        let events = &mut self.events;
        self.task_loop.sync(
            &mut self.reactor,
            |io| IoRecipient {
                target: IoTarget::Task { io },
            },
            |event| events.push_back(Envelope::Task { event }),
        )
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
            IoTarget::Task { io } => {
                // The readiness goes to the leaf that asked for it; the task is
                // enqueued here rather than through the leaf's parked waker, so
                // it keeps its place among the rest of this poll's batch.
                let Some(task) = self.task_loop.deliver_io(*io, notification.readiness()) else {
                    return;
                };
                self.events.push_back(Envelope::Task {
                    event: TaskEvent::Poll(task),
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
                let target = target.downgrade();
                let wakes = self.wakes.clone();
                self.spawn_timer(deadline, move || {
                    wakes.fire_protocol_timer(target, event);
                });
            }
            Effect::SetPaneTimer {
                target,
                deadline,
                generation,
            } => {
                let target = target.downgrade();
                let wakes = self.wakes.clone();
                self.spawn_timer(deadline, move || {
                    wakes.fire_pane_timer(target, generation);
                });
            }
            Effect::StopListener(target) => {
                target.stop();
            }
            Effect::StopPane(target) => {
                target.stop();
            }
            Effect::StopProtocol(target) => {
                target.stop();
            }
        }
        Ok(())
    }

    fn spawn_timer(&self, deadline: Instant, fire: impl FnOnce() + 'static) {
        let tasks = self.task_handle.clone();
        self.task_handle.spawn(async move {
            hmux_rt::sleep_until(&tasks, deadline).await;
            fire();
        });
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

    fn set_protocol_interest(
        &mut self,
        target: &ActorRef<ProtocolClient>,
        side: ProtocolIoSide,
        enabled: bool,
    ) -> io::Result<()> {
        if side.is_command() {
            return self.set_command_wake(target, side, enabled);
        }
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

    /// Interest in a suspended command queue, which has no descriptor to
    /// register: what the client gets instead is the wake its completion fires.
    ///
    /// Nothing here is keyed by which suspension is current. The client routes
    /// the wake to whatever it is parked on now, and a completion that already
    /// has its value fires the wake as it is installed — so unlike a
    /// registration that had to be re-pointed at each new descriptor, this
    /// cannot be left aimed at a suspension that is over.
    fn set_command_wake(
        &mut self,
        target: &ActorRef<ProtocolClient>,
        side: ProtocolIoSide,
        enabled: bool,
    ) -> io::Result<()> {
        let installed = target
            .with(|client| client.command_wake_installed(side))
            .unwrap_or(false);
        match (enabled, installed) {
            (true, _) => {
                let wake = self.wakes.protocol_wake(target.downgrade(), side);
                let installed = target
                    .with_mut(|client| client.install_command_wake(side, &wake))
                    .unwrap_or(false);
                if !installed {
                    // The queue this side named is gone — the same nothing a
                    // vanished descriptor leaves behind.
                    return Ok(());
                }
            }
            (false, true) => {
                target.with_mut(|client| client.clear_command_wake(side));
            }
            (false, false) => {}
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
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use crate::server::command::{
        ClientContext, ClientFileWrite, CommandSuspension, CommandSuspensionResult,
    };
    use std::os::fd::{AsFd as _, AsRawFd as _};

    use super::*;

    const POLL_TIMEOUT: Duration = Duration::from_secs(1);
    static NEXT_LISTENER_PATH: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn existing_imsg_descriptor_is_readiness_driven() {
        use crate::tmux::codec::{ImsgReader, ImsgWriter};
        use crate::tmux::message::{Frame, Message};

        let mut reactor = MioReactor::new().expect("reactor");
        let (sender, receiver) = UnixStream::pair().expect("socket pair");
        sender.set_nonblocking(true).expect("nonblocking sender");
        receiver
            .set_nonblocking(true)
            .expect("nonblocking receiver");
        let mut reader = ImsgReader::new(receiver.into());
        let mut writer = ImsgWriter::new(sender.into());
        let token = reactor
            .register(reader.as_fd(), Interest::READABLE, 42u32)
            .expect("register imsg reader");
        writer
            .send(Frame::new(Message::Command(vec!["list-sessions".into()])))
            .expect("send frame");

        let mut ready = Vec::new();
        reactor
            .poll(Some(POLL_TIMEOUT), &mut ready)
            .expect("poll");

        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].token(), token);
        let frame = reader.try_recv().expect("receive ready frame");
        assert_eq!(frame.msg, Message::Command(vec!["list-sessions".into()]));
    }

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
        let runtime = super::super::suspend::EventCommandRuntime::new(loop_.task_handle());
        let completion = crate::server::command::CommandRuntime::submit(&runtime, suspension)
            .expect("completion pair");
        let mut completion = completion;
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(result) = completion.take() {
                return result.expect("suspension result");
            }
            assert!(Instant::now() < deadline, "suspension never completed");
            // The completion belongs to the caller, not to a task, so nothing
            // here wakes it: keep turns short instead of blocking the reactor.
            loop_.run_turn(Some(Duration::from_millis(10)), 64).unwrap();
        }
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
        assert_eq!(
            loop_.task_loop.armed_timers(),
            0,
            "the job's timer outlived the job"
        );
    }

    #[test]
    fn event_loop_timer_uses_runtime_sleep() {
        let mut loop_ = EventLoop::new().unwrap();
        let fired = Rc::new(Cell::new(false));
        let fired_by_timer = Rc::clone(&fired);
        let started = Instant::now();
        loop_.spawn_timer(
            started + Duration::from_millis(20),
            move || fired_by_timer.set(true),
        );

        let test_deadline = started + Duration::from_secs(1);
        while !fired.get() {
            assert!(Instant::now() < test_deadline, "timer task never completed");
            loop_
                .run_turn(Some(Duration::from_millis(10)), 64)
                .unwrap();
        }

        assert!(started.elapsed() >= Duration::from_millis(20));
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
    fn a_finished_suspension_leaves_no_registration_behind() {
        let mut loop_ = EventLoop::new().unwrap();

        resolve_on_loop(&mut loop_, run_shell(&["true"]));
        // The descriptors go with the task's leaves, which the next sync
        // releases.
        loop_.dispatch_one().unwrap();

        assert_eq!(loop_.task_handle().registered_io(), 0);
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
