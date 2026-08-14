//! `Future` tasks running on a host-owned reactor.
//!
//! The task set is a tenant of its host's event loop, not an owner, and the
//! design here is all consequences of that:
//!
//! - The reactor belongs to the host, so an [`AsyncFd`] cannot register
//!   itself: it records the request and the host makes it on the next sync
//!   ([`TaskSet::take_new_io`]). Nothing is missed in between, because a
//!   registration only ever happens between dispatches and readiness that
//!   predates it is reported when the descriptor is added.
//! - Deadlines are the exception, and the asymmetry is deliberate. A timer
//!   queue is plain data, not a host-exclusive handle behind `&mut` and a
//!   syscall, so a [`Sleep`] arms and cancels its own from inside its poll.
//!   No request, no reconciliation, and no window in which the host has yet
//!   to hear about a deadline that is already due.
//! - There is no `block_on` and no run queue of its own. A task that can make
//!   progress is reported through the host's [`WakeSink`], so a task resuming
//!   is ordered against every other thing the host does — which may be
//!   observable behavior and is not ours to reorder.
//!
//! Everything is single-threaded: tasks are not `Send`, hold `Rc`/`RefCell`
//! state freely, and park [`LocalWaker`]s. The `Waker` half of the
//! [`Context`] is [`Waker::noop`], so **a leaf that parks `cx.waker()`
//! instead of `cx.local_waker()` never wakes**.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::io;
use std::os::fd::{BorrowedFd, OwnedFd};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, ContextBuilder, LocalWake, LocalWaker, Poll, Waker};
use std::time::{Duration, Instant};

use crate::handoff::{handoff, Handoff};
use crate::reactor::{Interest, Readiness, Token};
use crate::timer::{ExpiredTimer, TimerId, TimerQueue};

pub type TaskId = u64;

/// Where a task's wake lands.
///
/// The host queues "poll this task" however it orders the rest of its work;
/// nothing here resumes a task inline, because dispatch order is the host's
/// observable behavior.
pub type WakeSink = Rc<dyn Fn(TaskId)>;

/// Work addressed to one task.
pub enum TaskEvent {
    /// The task can make progress: poll it.
    Poll(TaskId),
    /// The deadline the task asked for has elapsed.
    Timeout(TaskId),
}

/// Readiness mailbox shared between one [`AsyncFd`] and the loop.
#[derive(Default)]
struct IoState {
    /// Readiness delivered and not yet consumed. A later delivery overwrites
    /// an unread one, which is fine under the "drain until `WouldBlock` before
    /// waiting again" discipline every leaf here follows.
    pending: Option<Readiness>,
    /// Set when the descriptor has no poll operation (`epoll` rejects regular
    /// files and `/dev/null` with `EPERM`). Such a descriptor is never *not*
    /// ready, so waits on it resolve immediately instead of parking forever.
    always_ready: bool,
    waker: Option<LocalWaker>,
}

type IoSlot = Rc<RefCell<IoState>>;

/// One [`AsyncFd`]'s registration, from the request to the deregistration.
struct IoEntry {
    /// The descriptor to register, taken by the loop when it does so. The
    /// reactor duplicates it, so it is not kept afterwards.
    fd: Option<OwnedFd>,
    interest: Interest,
    task: TaskId,
    slot: IoSlot,
    token: Option<Token>,
    /// Set when the `AsyncFd` is dropped; the loop releases it on the next
    /// sync, since a task cannot reach the reactor from inside its own poll.
    dropped: bool,
}

/// The parts of a task set a running task may touch.
///
/// Separate from the task table because a task runs *inside* the borrow of
/// that table: spawning, registering a descriptor and asking for a deadline
/// all happen while the task set itself is borrowed.
#[derive(Default)]
struct TaskShared {
    /// Tasks waiting to be adopted, with whether they still owe a first poll.
    spawned: RefCell<Vec<(TaskId, Pin<Box<dyn Future<Output = ()>>>, bool)>>,
    io: RefCell<BTreeMap<u64, IoEntry>>,
    /// Every armed deadline, one entry per parked [`Sleep`] and naming the
    /// task to poll when it lands.
    ///
    /// Unlike the reactor this is plain data — no syscall, no host-exclusive
    /// handle, no callback that could re-enter — so a sleep arms and cancels
    /// its own entry directly from inside its poll. A descriptor cannot do
    /// that, which is why [`IoEntry`] above still has to be a request the host
    /// picks up later.
    timers: RefCell<TimerQueue<TaskId>>,
    /// Every task that has been spawned and has not finished or been dropped.
    live: RefCell<BTreeSet<TaskId>>,
    /// Cancellation requests waiting for the task set to drop their futures.
    cancelled: RefCell<BTreeSet<TaskId>>,
    next_task: Cell<TaskId>,
    next_io: Cell<u64>,
    /// The task currently being polled, which is who a descriptor created now
    /// belongs to.
    polling: Cell<Option<TaskId>>,
}

impl TaskShared {
    fn allocate_task(&self) -> TaskId {
        let id = self.next_task.get().wrapping_add(1).max(1);
        self.next_task.set(id);
        id
    }

    fn allocate_io(&self) -> u64 {
        let id = self.next_io.get().wrapping_add(1).max(1);
        self.next_io.set(id);
        id
    }
}

/// Cloneable capability to spawn tasks and build the leaves they wait on.
#[derive(Clone)]
pub struct TaskHandle {
    shared: Rc<TaskShared>,
    wake: WakeSink,
}

impl TaskHandle {
    /// No `Send` bound: tasks live and die on this thread.
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static) -> TaskId {
        let id = self.shared.allocate_task();
        self.shared.live.borrow_mut().insert(id);
        self.shared
            .spawned
            .borrow_mut()
            .push((id, Box::pin(future), true));
        id
    }

    /// Spawn a task whose result can be awaited or whose lifetime can be
    /// cancelled through the returned handle.
    ///
    /// Dropping the [`JoinHandle`] detaches the task. Owners responsible for
    /// the child's lifetime cancel it explicitly or wrap the handle in an
    /// owner whose `Drop` implementation does so.
    pub fn spawn_join<T: 'static>(
        &self,
        future: impl Future<Output = T> + 'static,
    ) -> JoinHandle<T> {
        let (result, sender) = handoff();
        let task = self.spawn(async move {
            sender.complete(future.await);
        });
        JoinHandle {
            task,
            tasks: self.clone(),
            result,
        }
    }

    /// Request that a task be dropped before its next poll.
    ///
    /// Cancellation is idempotent and wakes the host even when the task is
    /// parked on I/O or a distant deadline. Returns `false` if the task has
    /// already finished or cancellation was already requested.
    pub fn cancel(&self, task: TaskId) -> bool {
        if !self.shared.live.borrow().contains(&task) {
            return false;
        }
        if !self.shared.cancelled.borrow_mut().insert(task) {
            return false;
        }
        (self.wake)(task);
        true
    }

    /// Whether the task has not yet finished or processed cancellation.
    pub fn is_active(&self, task: TaskId) -> bool {
        self.shared.live.borrow().contains(&task)
    }

    /// Number of spawned tasks that are still live.
    pub fn active_tasks(&self) -> usize {
        self.shared.live.borrow().len()
    }

    /// Spawn and give the task its first turn right here, in the caller's own
    /// dispatch.
    ///
    /// What a command queue does before it first has to wait is part of its
    /// client's turn: a `switch-client` that never suspends has to finish
    /// before the client's next notification goes out. Taking the first turn
    /// inline is what keeps that true; everything after the first wait is an
    /// ordinary task.
    pub fn spawn_now(&self, future: impl Future<Output = ()> + 'static) {
        let id = self.shared.allocate_task();
        self.shared.live.borrow_mut().insert(id);
        let mut future: Pin<Box<dyn Future<Output = ()>>> = Box::pin(future);
        let waker = LocalWaker::from(Rc::new(TaskWaker {
            task: id,
            wake: self.wake.clone(),
        }));
        let mut context = ContextBuilder::from_waker(Waker::noop())
            .local_waker(&waker)
            .build();
        let previous = self.shared.polling.replace(Some(id));
        let poll = future.as_mut().poll(&mut context);
        self.shared.polling.set(previous);
        if poll.is_pending() && !self.shared.cancelled.borrow().contains(&id) {
            // Already polled: whatever it parked on is what will wake it.
            self.shared.spawned.borrow_mut().push((id, future, false));
        } else {
            self.finish(id);
        }
    }

    /// Descriptors this task set has live registrations for.
    pub fn registered_io(&self) -> usize {
        self.shared
            .io
            .borrow()
            .values()
            .filter(|entry| !entry.dropped)
            .count()
    }

    // Sleeps need nothing here: dropping the task's future drops its leaves,
    // and each one hands its own entry back on the way out.
    fn finish(&self, task: TaskId) {
        self.shared.cancelled.borrow_mut().remove(&task);
        self.shared.live.borrow_mut().remove(&task);
    }
}

/// The awaitable lifetime of one spawned task.
pub struct JoinHandle<T> {
    task: TaskId,
    tasks: TaskHandle,
    result: Handoff<T>,
}

impl<T> JoinHandle<T> {
    pub fn cancel(&self) -> bool {
        self.tasks.cancel(self.task)
    }

    pub fn is_finished(&self) -> bool {
        !self.tasks.is_active(self.task)
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.result).poll(context) {
            Poll::Ready(Ok(value)) => Poll::Ready(Ok(value)),
            Poll::Ready(Err(_)) => Poll::Ready(Err(JoinError)),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A joined task stopped before producing its result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JoinError;

impl fmt::Display for JoinError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("task ended without producing its result")
    }
}

impl Error for JoinError {}

/// The loop's task table.
pub struct TaskSet {
    /// The slot is `None` while its future is being polled, which lets a
    /// running task spawn and register without re-borrowing.
    tasks: HashMap<TaskId, Option<Pin<Box<dyn Future<Output = ()>>>>>,
    shared: Rc<TaskShared>,
    wake: WakeSink,
}

impl TaskSet {
    pub fn new(wake: WakeSink) -> (Self, TaskHandle) {
        let shared = Rc::new(TaskShared::default());
        (
            Self {
                tasks: HashMap::new(),
                shared: Rc::clone(&shared),
                wake: wake.clone(),
            },
            TaskHandle { shared, wake },
        )
    }

    /// Tasks spawned since the last sync, still owed their first poll.
    pub fn pending_spawned(&self) -> usize {
        self.shared.spawned.borrow().len()
    }

    /// Adopt every task spawned since the last sync, reporting the ids that
    /// still owe a first poll.
    pub fn take_spawned(&mut self) -> Vec<TaskId> {
        let spawned = std::mem::take(&mut *self.shared.spawned.borrow_mut());
        spawned
            .into_iter()
            .filter_map(|(id, future, needs_poll)| {
                if self.shared.cancelled.borrow().contains(&id) {
                    drop(future);
                    self.finish(id);
                    return None;
                }
                self.tasks.insert(id, Some(future));
                needs_poll.then_some(id)
            })
            .collect()
    }

    /// Poll one task, dropping it if it finishes.
    pub fn poll(&mut self, id: TaskId) {
        if self.shared.cancelled.borrow().contains(&id) {
            self.finish(id);
            return;
        }
        let Some(Some(mut future)) = self.tasks.get_mut(&id).map(Option::take) else {
            // Finished task, or a second wake for one already being polled.
            // Ids are never reused, so a stale wake can only miss.
            return;
        };
        let waker = LocalWaker::from(Rc::new(TaskWaker {
            task: id,
            wake: self.wake.clone(),
        }));
        // The `Waker` half is inert on purpose: a `Send` wake path would mean
        // an `Arc<Mutex<..>>` behind wakes that never cross a thread.
        let mut context = ContextBuilder::from_waker(Waker::noop())
            .local_waker(&waker)
            .build();
        let previous = self.shared.polling.replace(Some(id));
        let poll = future.as_mut().poll(&mut context);
        self.shared.polling.set(previous);
        match poll {
            Poll::Ready(()) => {
                // The future's leaves went with it; their entries are marked
                // dropped and the loop releases them on the next sync.
                self.finish(id);
            }
            Poll::Pending if !self.shared.cancelled.borrow().contains(&id) => {
                self.tasks.insert(id, Some(future));
            }
            Poll::Pending => self.finish(id),
        }
    }

    /// Hand readiness to the descriptor it was registered for, reporting the
    /// task to poll. `None` once the `AsyncFd` is gone.
    pub fn deliver_io(&mut self, io: u64, readiness: Readiness) -> Option<TaskId> {
        let entries = self.shared.io.borrow();
        let entry = entries.get(&io)?;
        if entry.dropped {
            return None;
        }
        entry.slot.borrow_mut().pending = Some(readiness);
        Some(entry.task)
    }

    /// Descriptors to release: every entry whose `AsyncFd` is gone.
    pub fn take_released_io(&mut self) -> Vec<Token> {
        let mut entries = self.shared.io.borrow_mut();
        let released = entries
            .iter()
            .filter(|(_, entry)| entry.dropped)
            .map(|(io, _)| *io)
            .collect::<Vec<_>>();
        released
            .into_iter()
            .filter_map(|io| entries.remove(&io).and_then(|entry| entry.token))
            .collect()
    }

    /// Descriptors to register: every entry the loop has not registered yet.
    pub fn take_new_io(&mut self) -> Vec<(u64, TaskId, OwnedFd, Interest)> {
        let mut entries = self.shared.io.borrow_mut();
        entries
            .iter_mut()
            .filter(|(_, entry)| entry.token.is_none() && !entry.dropped)
            .filter_map(|(io, entry)| {
                entry
                    .fd
                    .take()
                    .map(|fd| (*io, entry.task, fd, entry.interest))
            })
            .collect()
    }

    pub fn set_io_token(&mut self, io: u64, token: Token) {
        if let Some(entry) = self.shared.io.borrow_mut().get_mut(&io) {
            entry.token = Some(token);
        }
    }

    /// Record that the descriptor cannot be polled, so waits on it resolve
    /// immediately. Reports the owning task so the host can queue its poll.
    pub fn mark_io_unpollable(&mut self, io: u64) -> Option<TaskId> {
        let entries = self.shared.io.borrow();
        let entry = entries.get(&io)?;
        entry.slot.borrow_mut().always_ready = true;
        Some(entry.task)
    }

    /// How long the host may block before the nearest armed deadline.
    ///
    /// Nothing has to be synced first: a sleep is in the queue from the moment
    /// it parks, so this reads what is armed right now.
    pub fn time_until_next_deadline(&self, now: Instant) -> Option<Duration> {
        self.shared.timers.borrow_mut().time_until_next(now)
    }

    /// Take every deadline that has elapsed, naming the task each belongs to.
    pub fn drain_expired(&self, now: Instant, output: &mut Vec<ExpiredTimer<TaskId>>) {
        self.shared.timers.borrow_mut().drain_expired(now, output);
    }

    /// Deadlines armed, one per parked sleep.
    pub fn armed_timers(&self) -> usize {
        self.shared.timers.borrow().len()
    }

    // As in `TaskShared::finish`: removing the future drops the sleeps it was
    // holding, and each cancels its own deadline on the way out.
    fn finish(&mut self, task: TaskId) {
        self.tasks.remove(&task);
        self.shared.cancelled.borrow_mut().remove(&task);
        self.shared.live.borrow_mut().remove(&task);
    }
}

impl Drop for TaskSet {
    fn drop(&mut self) {
        // Drop adopted futures first, then tasks that never received their
        // first poll. Their handoff senders close join handles and their
        // leaves mark registrations for release in the usual way.
        self.tasks.clear();
        let spawned = std::mem::take(&mut *self.shared.spawned.borrow_mut());
        drop(spawned);
        self.shared.io.borrow_mut().clear();
        *self.shared.timers.borrow_mut() = TimerQueue::default();
        self.shared.cancelled.borrow_mut().clear();
        self.shared.live.borrow_mut().clear();
        self.shared.polling.set(None);
    }
}

/// Wakes are just "poll this task"; the host's [`WakeSink`] is the whole
/// mechanism.
///
/// [`LocalWake`] keeps the payload an `Rc` — the `Send + Sync` bound that would
/// force an `Arc` belongs to `Waker`, not `LocalWaker`.
struct TaskWaker {
    task: TaskId,
    wake: WakeSink,
}

impl LocalWake for TaskWaker {
    fn wake(self: Rc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Rc<Self>) {
        (self.wake)(self.task);
    }
}

/// Owned registration of one descriptor, persistent across polls: requested on
/// creation, released on drop.
///
/// A suspension that creates a fresh descriptor simply creates a fresh
/// `AsyncFd`. Reactor tokens are never reused and the registration is not
/// shared, so there is no identity for a generation key to disambiguate.
pub struct AsyncFd {
    shared: Rc<TaskShared>,
    io: u64,
    slot: IoSlot,
}

impl AsyncFd {
    /// Take a duplicate of `fd` to register.
    ///
    /// The duplicate is what the loop hands the reactor, so the caller stays
    /// free to close the original whenever its own logic is done with it.
    pub fn new(handle: &TaskHandle, fd: BorrowedFd<'_>, interest: Interest) -> io::Result<Self> {
        let task = handle
            .shared
            .polling
            .get()
            .ok_or_else(|| io::Error::other("a descriptor was created outside any task"))?;
        let slot = IoSlot::default();
        let io = handle.shared.allocate_io();
        handle.shared.io.borrow_mut().insert(
            io,
            IoEntry {
                fd: Some(fd.try_clone_to_owned()?),
                interest,
                task,
                slot: Rc::clone(&slot),
                token: None,
                dropped: false,
            },
        );
        Ok(Self {
            shared: Rc::clone(&handle.shared),
            io,
            slot,
        })
    }

    /// Wait for the next readiness delivery. Edge-style: after it resolves,
    /// read or write until `WouldBlock` before waiting again.
    pub fn readiness(&self) -> impl Future<Output = Readiness> + '_ {
        ReadinessFuture { fd: self }
    }
}

impl Drop for AsyncFd {
    fn drop(&mut self) {
        if let Some(entry) = self.shared.io.borrow_mut().get_mut(&self.io) {
            entry.dropped = true;
        }
    }
}

struct ReadinessFuture<'a> {
    fd: &'a AsyncFd,
}

impl Future for ReadinessFuture<'_> {
    type Output = Readiness;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Readiness> {
        let mut slot = self.fd.slot.borrow_mut();
        if let Some(readiness) = slot.pending.take() {
            return Poll::Ready(readiness);
        }
        if slot.always_ready {
            return Poll::Ready(Readiness::always());
        }
        slot.waker = Some(context.local_waker().clone());
        Poll::Pending
    }
}

/// Sleep until `deadline`, on the loop's timer queue.
///
/// The returned future owns its deadline outright: dropping it — losing a
/// select is the usual way — cancels the timer it armed and no other.
pub fn sleep_until(handle: &TaskHandle, deadline: Instant) -> impl Future<Output = ()> {
    Sleep {
        shared: Rc::clone(&handle.shared),
        deadline,
        armed: Cell::new(None),
    }
}

pub fn sleep(handle: &TaskHandle, duration: Duration) -> impl Future<Output = ()> {
    sleep_until(handle, Instant::now() + duration)
}

struct Sleep {
    shared: Rc<TaskShared>,
    deadline: Instant,
    /// The timer this sleep armed for itself, held so that dropping it —
    /// losing a [`select`] is the usual way — cancels that one and no other.
    ///
    /// Identity lives here rather than in a table the host reconciles, so two
    /// sleeps parked on one task, the shape a merged pair of timed sources
    /// produces, neither collide nor cancel each other.
    armed: Cell<Option<TimerId>>,
}

impl Sleep {
    /// Give up the deadline this sleep armed, if it still holds one.
    fn disarm(&self) {
        if let Some(timer) = self.armed.replace(None) {
            // A timer already drained by the host is gone from the queue, so
            // this finds nothing — which is the same as having cancelled it.
            self.shared.timers.borrow_mut().cancel(timer);
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<()> {
        let Some(task) = self.shared.polling.get() else {
            return Poll::Pending;
        };
        if Instant::now() >= self.deadline {
            self.disarm();
            return Poll::Ready(());
        }
        if self.armed.get().is_none() {
            // Armed the moment it parks: the queue is plain data reachable
            // from inside this poll, so there is no window where the host has
            // yet to hear about the deadline. A re-poll before the deadline —
            // a sibling leaf woke the task — keeps the timer it already has.
            let timer = self.shared.timers.borrow_mut().set(self.deadline, task);
            self.armed.set(Some(timer));
        }
        Poll::Pending
    }
}

impl Drop for Sleep {
    fn drop(&mut self) {
        self.disarm();
    }
}
