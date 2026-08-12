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
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::future::Future;
use std::io;
use std::os::fd::{BorrowedFd, OwnedFd};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, ContextBuilder, LocalWake, LocalWaker, Poll, Waker};
use std::time::{Duration, Instant};

use crate::reactor::{Interest, Readiness, Token};

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
    /// The deadline each task is sleeping until, at most one apiece.
    deadlines: RefCell<BTreeMap<TaskId, Instant>>,
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
        self.shared
            .spawned
            .borrow_mut()
            .push((id, Box::pin(future), true));
        id
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
        if poll.is_pending() {
            // Already polled: whatever it parked on is what will wake it.
            self.shared.spawned.borrow_mut().push((id, future, false));
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
}

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

    /// Adopt every task spawned since the last sync, reporting the ids that
    /// still owe a first poll.
    pub fn take_spawned(&mut self) -> Vec<TaskId> {
        let spawned = std::mem::take(&mut *self.shared.spawned.borrow_mut());
        spawned
            .into_iter()
            .filter_map(|(id, future, needs_poll)| {
                self.tasks.insert(id, Some(future));
                needs_poll.then_some(id)
            })
            .collect()
    }

    /// Poll one task, dropping it if it finishes.
    pub fn poll(&mut self, id: TaskId) {
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
                self.tasks.remove(&id);
                // The future's leaves went with it; their entries are marked
                // dropped and the loop releases them on the next sync.
                self.shared.deadlines.borrow_mut().remove(&id);
            }
            Poll::Pending => {
                self.tasks.insert(id, Some(future));
            }
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

    /// What every sleeping task is waiting for, for the loop to arm timers
    /// against.
    pub fn deadlines(&self) -> BTreeMap<TaskId, Instant> {
        self.shared.deadlines.borrow().clone()
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
    pub fn readiness(&self) -> ReadinessFuture<'_> {
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

pub struct ReadinessFuture<'a> {
    fd: &'a AsyncFd,
}

impl Future for ReadinessFuture<'_> {
    type Output = Readiness;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Readiness> {
        let mut slot = self.fd.slot.borrow_mut();
        if let Some(readiness) = slot.pending.take() {
            return Poll::Ready(readiness);
        }
        slot.waker = Some(context.local_waker().clone());
        Poll::Pending
    }
}

/// Sleep until `deadline`, on the loop's timer queue.
pub fn sleep_until(handle: &TaskHandle, deadline: Instant) -> Sleep {
    Sleep {
        shared: Rc::clone(&handle.shared),
        deadline,
    }
}

pub fn sleep(handle: &TaskHandle, duration: Duration) -> Sleep {
    sleep_until(handle, Instant::now() + duration)
}

pub struct Sleep {
    shared: Rc<TaskShared>,
    deadline: Instant,
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<()> {
        let Some(task) = self.shared.polling.get() else {
            return Poll::Pending;
        };
        if Instant::now() >= self.deadline {
            self.shared.deadlines.borrow_mut().remove(&task);
            return Poll::Ready(());
        }
        self.shared
            .deadlines
            .borrow_mut()
            .insert(task, self.deadline);
        Poll::Pending
    }
}

/// Give the loop a turn before continuing.
///
/// A task that has more work but has used its budget parks itself behind
/// everything the loop already has queued, rather than running the loop's other
/// work late.
pub fn yield_now() -> YieldNow {
    YieldNow { yielded: false }
}

pub struct YieldNow {
    yielded: bool,
}

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<()> {
        if self.yielded {
            return Poll::Ready(());
        }
        self.yielded = true;
        context.local_waker().wake_by_ref();
        Poll::Pending
    }
}

/// Run two futures on one task, finishing when both have.
///
/// The two pipes of a child process are the reason this exists: draining them
/// one after the other deadlocks as soon as the one not being read fills up.
pub fn join<A: Future, B: Future>(first: A, second: B) -> Join<A, B> {
    Join {
        first: Pending::Running(first),
        second: Pending::Running(second),
    }
}

enum Pending<F: Future> {
    Running(F),
    Done(Option<F::Output>),
}

impl<F: Future> Pending<F> {
    /// Poll unless already finished, and report the output once both halves
    /// can be taken.
    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> bool {
        // Safety: the projection is structural and nothing here is moved out
        // of the pinned future while it is `Running`.
        let this = unsafe { self.get_unchecked_mut() };
        match this {
            Self::Running(future) => {
                let pinned = unsafe { Pin::new_unchecked(future) };
                match pinned.poll(context) {
                    Poll::Ready(output) => {
                        *this = Self::Done(Some(output));
                        true
                    }
                    Poll::Pending => false,
                }
            }
            Self::Done(_) => true,
        }
    }

    fn take(&mut self) -> F::Output {
        match self {
            Self::Done(output) => output.take().expect("a joined future reported twice"),
            Self::Running(_) => unreachable!("a joined future that has not finished"),
        }
    }
}

pub struct Join<A: Future, B: Future> {
    first: Pending<A>,
    second: Pending<B>,
}

impl<A: Future, B: Future> Future for Join<A, B> {
    type Output = (A::Output, B::Output);

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        // Safety: `Join` is only ever pinned as a whole, and the projections
        // below are the standard structural ones.
        let this = unsafe { self.get_unchecked_mut() };
        // Both halves are polled every time: whichever woke us is not knowable
        // here, and a poll of an already-finished half is free.
        let first = unsafe { Pin::new_unchecked(&mut this.first) }.poll(context);
        let second = unsafe { Pin::new_unchecked(&mut this.second) }.poll(context);
        if first && second {
            return Poll::Ready((this.first.take(), this.second.take()));
        }
        Poll::Pending
    }
}
