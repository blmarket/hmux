use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::io;
use std::os::fd::{BorrowedFd, OwnedFd};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, ContextBuilder, LocalWake, LocalWaker, Poll, Waker};
use std::time::{Duration, Instant};

use crate::handoff::{Handoff, handoff};
use crate::reactor::{Interest, Readiness, Token};
use crate::timer::{TimerId, TimerQueue};

pub type TaskId = u64;

/// Readiness mailbox shared between one [`AsyncFd`] and the loop.
#[derive(Default)]
pub(crate) struct IoState {
    pub(crate) pending: Option<Readiness>,
    pub(crate) always_ready: bool,
    waker: Option<LocalWaker>,
}

pub(crate) type IoSlot = Rc<RefCell<IoState>>;

/// One [`AsyncFd`]'s registration, from the request to the deregistration.
pub(crate) struct IoEntry {
    /// The descriptor to register, taken by the loop when it does so. The
    /// reactor duplicates it, so it is not kept afterwards.
    pub(crate) fd: Option<OwnedFd>,
    pub(crate) interest: Interest,
    pub(crate) task: TaskId,
    pub(crate) slot: IoSlot,
    pub(crate) token: Option<Token>,
    pub(crate) dropped: bool,
}

/// The state a running task may touch, shared with the loop that drives it.
///
/// Separate from the runtime's task table because a task runs *inside* the
/// borrow of that table: spawning, registering a descriptor and asking for a
/// deadline all happen while the table is borrowed.
#[derive(Default)]
pub(crate) struct TaskShared {
    /// Tasks waiting to be adopted, with whether they still owe a first poll.
    pub(crate) spawned: RefCell<Vec<(TaskId, Pin<Box<dyn Future<Output = ()>>>, bool)>>,
    pub(crate) io: RefCell<BTreeMap<u64, IoEntry>>,
    /// Ids in `io` the loop still owes work on: descriptors created since the
    /// last sync, and those whose `AsyncFd` is gone.
    ///
    /// The leaf records what it did at the moment it does it, so a sync costs
    /// what was actually asked for rather than a walk of every live
    /// descriptor. An id can sit on both lists — created and dropped inside
    /// one window — which is why the loop releases before it registers.
    pub(crate) io_new: RefCell<Vec<u64>>,
    pub(crate) io_dropped: RefCell<Vec<u64>>,
    /// Every armed deadline, one entry per parked [`Sleep`] and naming the
    /// task to poll when it lands.
    ///
    /// Unlike the reactor this is plain data — no syscall, no exclusive
    /// handle, no callback that could re-enter — so a sleep arms and cancels
    /// its own entry directly from inside its poll. A descriptor cannot do
    /// that, which is why [`IoEntry`] above still has to be a request the loop
    /// picks up later.
    pub(crate) timers: RefCell<TimerQueue<TaskId>>,
    /// Every task that has been spawned and has not finished or been dropped.
    pub(crate) live: RefCell<BTreeSet<TaskId>>,
    /// Cancellation requests waiting for the loop to drop their futures.
    pub(crate) cancelled: RefCell<BTreeSet<TaskId>>,
    /// The run queue: every wake is "poll this task", in fire order. It lives
    /// here rather than in the runtime because a waker holds nothing else.
    pub(crate) woken: RefCell<VecDeque<TaskId>>,
    next_task: Cell<TaskId>,
    next_io: Cell<u64>,
    /// The task currently being polled, which is who a descriptor created now
    /// belongs to.
    pub(crate) polling: Cell<Option<TaskId>>,
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

    /// Poll `future` as task `id`: give it a waker that queues that id, and
    /// name it as the owner of any descriptor it creates while it runs.
    pub(crate) fn poll_future(
        self: &Rc<Self>,
        id: TaskId,
        mut future: Pin<&mut dyn Future<Output = ()>>,
    ) -> Poll<()> {
        let waker = LocalWaker::from(Rc::new(TaskWaker {
            task: id,
            shared: Rc::clone(self),
        }));
        // The `Waker` half is inert on purpose: a `Send` wake path would mean
        // an `Arc<Mutex<..>>` behind wakes that never cross a thread.
        let mut context = ContextBuilder::from_waker(Waker::noop())
            .local_waker(&waker)
            .build();
        let previous = self.polling.replace(Some(id));
        let poll = future.as_mut().poll(&mut context);
        self.polling.set(previous);
        poll
    }

    // Sleeps need nothing here: dropping the task's future drops its leaves,
    // and each one hands its own entry back on the way out.
    pub(crate) fn finish(&self, task: TaskId) {
        self.cancelled.borrow_mut().remove(&task);
        self.live.borrow_mut().remove(&task);
    }
}

/// Cloneable capability to spawn tasks and build the leaves they wait on.
#[derive(Clone)]
pub struct TaskHandle {
    shared: Rc<TaskShared>,
}

impl TaskHandle {
    pub(crate) fn new(shared: Rc<TaskShared>) -> Self {
        Self { shared }
    }

    /// Remember, everything is single-threaded
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
    /// Cancellation is idempotent and wakes the loop even when the task is
    /// parked on I/O or a distant deadline. Returns `false` if the task has
    /// already finished or cancellation was already requested.
    pub fn cancel(&self, task: TaskId) -> bool {
        if !self.shared.live.borrow().contains(&task) {
            return false;
        }
        if !self.shared.cancelled.borrow_mut().insert(task) {
            return false;
        }
        self.shared.woken.borrow_mut().push_back(task);
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
        let poll = self.shared.poll_future(id, future.as_mut());
        if poll.is_pending() && !self.shared.cancelled.borrow().contains(&id) {
            // Already polled: whatever it parked on is what will wake it.
            self.shared.spawned.borrow_mut().push((id, future, false));
        } else {
            self.shared.finish(id);
        }
    }

    /// Descriptors the loop has live registrations for.
    pub fn registered_io(&self) -> usize {
        self.shared
            .io
            .borrow()
            .values()
            .filter(|entry| !entry.dropped)
            .count()
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

/// Wakes are just "poll this task"; the run queue is the whole mechanism.
///
/// [`LocalWake`] keeps the payload an `Rc` — the `Send + Sync` bound that would
/// force an `Arc` belongs to `Waker`, not `LocalWaker`.
struct TaskWaker {
    task: TaskId,
    shared: Rc<TaskShared>,
}

impl LocalWake for TaskWaker {
    fn wake(self: Rc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Rc<Self>) {
        self.shared.woken.borrow_mut().push_back(self.task);
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
        handle.shared.io_new.borrow_mut().push(io);
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
            // An `AsyncFd` owns its entry alone and drops once, so this names
            // the id exactly once.
            self.shared.io_dropped.borrow_mut().push(self.io);
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
pub fn sleep_until(handle: &TaskHandle, deadline: Instant) -> impl Future<Output = ()> + use<> {
    Sleep {
        shared: Rc::clone(&handle.shared),
        deadline,
        armed: Cell::new(None),
    }
}

pub fn sleep(handle: &TaskHandle, duration: Duration) -> impl Future<Output = ()> + use<> {
    sleep_until(handle, Instant::now() + duration)
}

struct Sleep {
    shared: Rc<TaskShared>,
    deadline: Instant,
    /// The timer this sleep armed for itself, held so that dropping it —
    /// losing a [`select`] is the usual way — cancels that one and no other.
    ///
    /// Identity lives here rather than in a table the loop reconciles, so two
    /// sleeps parked on one task, the shape a merged pair of timed sources
    /// produces, neither collide nor cancel each other.
    armed: Cell<Option<TimerId>>,
}

impl Sleep {
    /// Give up the deadline this sleep armed, if it still holds one.
    fn disarm(&self) {
        if let Some(timer) = self.armed.replace(None) {
            // A timer already drained by the loop is gone from the queue, so
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
            // from inside this poll, so there is no window where the loop has
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
