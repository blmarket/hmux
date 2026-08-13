//! The task that hosts one protocol client's event-driven state machine.
//!
//! The machine itself is unchanged: `ProtocolClient::handle` consumes one
//! [`ProtocolEvent`] and records what it wants in an [`Outbox`]. What changes
//! is who interprets the effects — the client's own task, which owns the
//! readiness registrations as `AsyncFd`s, spawns timer sleeps, and queues
//! follow-up events, where the central loop used to do all three.

use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::future::Future as _;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Poll;
use std::time::Instant;

use crate::server::Server;
use crate::tmux::codec::{ImsgReader, NonblockingImsgWriter};
use hmux_rt::{yield_now, AsyncFd, Interest, Notify, TaskHandle, WakeFn};

use super::super::job::BackgroundRunner;
use super::{ProtocolClient, ProtocolCloseReason, ProtocolEvent, ProtocolIoSide, ProtocolStatus};

/// What a wake or timer left for the task to pick up.
///
/// Only the payload is recorded. Resolving it — deduplicating against work
/// already queued — touches the client, and a wake fires from inside its
/// producer's dispatch, where the producer may be this very client: one that
/// answers its own `command-prompt -w` does exactly that. Deferring the
/// resolution to the drain keeps the wake itself from reaching into the state.
enum PendingItem {
    /// A command-side wake: dedup against work already queued, then dispatch.
    Side(ProtocolIoSide),
    /// A timer fired; generation checks in the handler drop stale ones.
    Timer(ProtocolEvent),
}

/// Where wakes leave their payloads, shared with the wake closures the client
/// installs on its suspensions' completions and with its timer sleeps.
#[derive(Clone, Default)]
struct Inbox {
    items: Rc<RefCell<VecDeque<PendingItem>>>,
    notify: Notify,
}

impl Inbox {
    fn push(&self, item: PendingItem) {
        self.items.borrow_mut().push_back(item);
        self.notify.notify();
    }

    /// The wake for a suspension `side` is parked on.
    fn side_wake(&self, side: ProtocolIoSide) -> WakeFn {
        let inbox = self.clone();
        Rc::new(move || inbox.push(PendingItem::Side(side)))
    }
}

enum Effect {
    Enqueue(ProtocolEvent),
    SetInterest { side: ProtocolIoSide, enabled: bool },
    SetTimer { deadline: Instant, event: ProtocolEvent },
    Stop,
}

/// Effects emitted by one `handle` dispatch and applied only after it returns.
pub(crate) struct Outbox {
    effects: Vec<Effect>,
}

impl Outbox {
    fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    pub(crate) fn enqueue_protocol(&mut self, event: ProtocolEvent) {
        self.effects.push(Effect::Enqueue(event));
    }

    pub(crate) fn set_protocol_interest(&mut self, side: ProtocolIoSide, enabled: bool) {
        self.effects.push(Effect::SetInterest { side, enabled });
    }

    pub(crate) fn set_protocol_timer_event(&mut self, deadline: Instant, event: ProtocolEvent) {
        self.effects.push(Effect::SetTimer { deadline, event });
    }

    pub(crate) fn stop_protocol(&mut self) {
        self.effects.push(Effect::Stop);
    }
}

type SharedClient = Rc<RefCell<Option<ProtocolClient>>>;

/// References returned when a protocol client is added to the loop.
pub(crate) struct ProtocolHandle {
    client: SharedClient,
    status: ProtocolStatus,
}

impl ProtocolHandle {
    pub(crate) fn is_alive(&self) -> bool {
        self.client.borrow().is_some()
    }

    pub(crate) fn close_reason(&self) -> Option<ProtocolCloseReason> {
        self.client
            .borrow()
            .as_ref()
            .and_then(ProtocolClient::close_reason)
            .or_else(|| self.status.close_reason())
    }

    #[cfg(test)]
    pub(crate) fn is_direct(&self) -> bool {
        self.client
            .borrow()
            .as_ref()
            .is_some_and(ProtocolClient::is_direct)
    }

    #[cfg(test)]
    pub(crate) fn is_control(&self) -> bool {
        self.client
            .borrow()
            .as_ref()
            .is_some_and(ProtocolClient::is_control)
    }

    #[cfg(test)]
    pub(crate) fn is_attach(&self) -> bool {
        self.client
            .borrow()
            .as_ref()
            .is_some_and(ProtocolClient::is_attach)
    }
}

/// Serve one accepted connection on the loop.
pub(crate) fn spawn(
    tasks: &TaskHandle,
    reader: ImsgReader,
    writer: NonblockingImsgWriter,
    server: Server,
    background_commands: BackgroundRunner,
    peer_uid: Option<u32>,
) -> ProtocolHandle {
    let (client, status) =
        ProtocolClient::new(reader, writer, server, background_commands, tasks.clone(), peer_uid);
    let client: SharedClient = Rc::new(RefCell::new(Some(client)));
    let handle = ProtocolHandle {
        client: Rc::clone(&client),
        status,
    };
    let task_handle = tasks.clone();
    tasks.spawn(async move {
        run(&task_handle, client).await;
    });
    handle
}

async fn run(tasks: &TaskHandle, client: SharedClient) {
    let inbox = Inbox::default();
    let mut fds: BTreeMap<ProtocolIoSide, AsyncFd> = BTreeMap::new();
    let mut events: VecDeque<ProtocolEvent> = VecDeque::new();
    events.push_back(ProtocolEvent::Start);
    loop {
        drain_inbox(&inbox, &client, &mut events);
        let Some(event) = events.pop_front() else {
            wait(&inbox, &client, &fds, &mut events).await;
            continue;
        };
        let mut outbox = Outbox::new();
        {
            let mut slot = client.borrow_mut();
            let Some(active) = slot.as_mut() else {
                return;
            };
            active.handle(event, &mut outbox);
        }
        let mut stop = false;
        for effect in outbox.effects {
            match effect {
                Effect::Enqueue(event) => events.push_back(event),
                Effect::SetInterest { side, enabled } => {
                    apply_interest(tasks, &inbox, &client, &mut fds, side, enabled);
                }
                Effect::SetTimer { deadline, event } => {
                    let inbox = inbox.clone();
                    let sleeper = tasks.clone();
                    tasks.spawn(async move {
                        hmux_rt::sleep_until(&sleeper, deadline).await;
                        inbox.push(PendingItem::Timer(event));
                    });
                }
                Effect::Stop => stop = true,
            }
        }
        if stop {
            *client.borrow_mut() = None;
            return;
        }
        // One event per turn, the granularity one envelope per dispatch had:
        // follow-up work goes behind everything the loop already has queued.
        yield_now().await;
    }
}

/// Turn every fired wake into an event. Runs between dispatches, so reaching
/// into the client here is safe.
fn drain_inbox(inbox: &Inbox, client: &SharedClient, events: &mut VecDeque<ProtocolEvent>) {
    // The borrow ends before the client is touched: resolving one wake can
    // fire another, and that wake has to find this queue free.
    let items = std::mem::take(&mut *inbox.items.borrow_mut());
    for item in items {
        match item {
            PendingItem::Side(side) => {
                // Same dedup a readiness notification gets: a client woken
                // twice before it runs is one turn's work.
                let should_enqueue = client
                    .borrow_mut()
                    .as_mut()
                    .is_some_and(|active| active.mark_work_queued(side));
                if should_enqueue {
                    events.push_back(protocol_ready_event(side));
                }
            }
            PendingItem::Timer(event) => events.push_back(event),
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

/// Make the task's registrations describe the interest one effect declares.
///
/// A descriptor side holds an [`AsyncFd`] while interested — dropping it is
/// the deregistration, re-creating it the registration, and a fresh
/// registration reports readiness that already holds, so nothing is missed
/// across a pause. A command side has no descriptor: what the client gets
/// instead is the wake its completion fires.
fn apply_interest(
    tasks: &TaskHandle,
    inbox: &Inbox,
    client: &SharedClient,
    fds: &mut BTreeMap<ProtocolIoSide, AsyncFd>,
    side: ProtocolIoSide,
    enabled: bool,
) {
    if side.is_command() {
        // Nothing here is keyed by which suspension is current. The client
        // routes the wake to whatever it is parked on now, and a completion
        // that already has its value fires the wake as it is installed — so
        // this cannot be left aimed at a suspension that is over.
        let mut slot = client.borrow_mut();
        let Some(active) = slot.as_mut() else {
            return;
        };
        match (enabled, active.command_wake_installed(side)) {
            (true, _) => {
                let wake = inbox.side_wake(side);
                // A queue that is gone has nothing to wake — the same nothing
                // a vanished descriptor leaves behind.
                let _ = active.install_command_wake(side, &wake);
            }
            (false, true) => active.clear_command_wake(side),
            (false, false) => {}
        }
        return;
    }

    if !enabled {
        if fds.remove(&side).is_some() {
            if let Some(active) = client.borrow_mut().as_mut() {
                active.note_interest(side, false);
            }
        }
        return;
    }
    if fds.contains_key(&side) {
        return;
    }
    let created = {
        let slot = client.borrow();
        let Some(active) = slot.as_ref() else {
            return;
        };
        // A side whose descriptor is already gone — the client moved on since
        // asking — is dropped the way a vanished registration is.
        let Some(fd) = active.fd(side) else {
            return;
        };
        let interest = match side {
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
        };
        AsyncFd::new(tasks, fd, interest)
    };
    if let Ok(fd) = created {
        fds.insert(side, fd);
        if let Some(active) = client.borrow_mut().as_mut() {
            active.note_interest(side, true);
        }
    }
}

/// The outcome one wait reports: an inbox arrival, or readiness on a side.
enum Wakeup {
    Inbox,
    Ready(ProtocolIoSide),
}

/// Park until a wake lands in the inbox or an interested side is ready.
///
/// Readiness becomes an event with the same dedup a central delivery gave it;
/// inbox items are left for the caller's drain.
async fn wait(
    inbox: &Inbox,
    client: &SharedClient,
    fds: &BTreeMap<ProtocolIoSide, AsyncFd>,
    events: &mut VecDeque<ProtocolEvent>,
) {
    let woken = std::future::poll_fn(|context| {
        if !inbox.items.borrow().is_empty() {
            return Poll::Ready(Wakeup::Inbox);
        }
        let mut notified = inbox.notify.notified();
        if Pin::new(&mut notified).poll(context).is_ready() {
            return Poll::Ready(Wakeup::Inbox);
        }
        for (side, fd) in fds {
            let mut readiness = fd.readiness();
            if Pin::new(&mut readiness).poll(context).is_ready() {
                return Poll::Ready(Wakeup::Ready(*side));
            }
        }
        Poll::Pending
    })
    .await;
    if let Wakeup::Ready(side) = woken {
        let should_enqueue = client
            .borrow_mut()
            .as_mut()
            .is_some_and(|active| active.mark_work_queued(side));
        if should_enqueue {
            events.push_back(protocol_ready_event(side));
        }
    }
}

