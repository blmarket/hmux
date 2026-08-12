//! Runtime-neutral resumable tasks and any-of wait descriptions.

use std::cell::RefCell;
use std::future::Future;
use std::io;
use std::os::fd::BorrowedFd;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use std::time::Instant;

/// Called once, by the producer, to tell whoever drives a parked task that its
/// value has arrived.
///
/// The callback belongs to the driver that installed it: it enqueues that
/// driver's own wake event and returns. Resuming the task inline from here
/// would make dispatch order depend on which producer happened to finish
/// first, which is observable behavior.
pub(crate) type WakeFn = Rc<dyn Fn()>;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct WaitToken(u32);

impl WaitToken {
    pub(crate) const fn new(value: u32) -> Self {
        Self(value)
    }
}

/// Which way a descriptor has to become ready before its task can make
/// progress. A task waits for exactly one direction per descriptor: a job that
/// both reads and writes one fd describes it twice, under two tokens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FdDirection {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FdInterest<'a> {
    token: WaitToken,
    fd: BorrowedFd<'a>,
    direction: FdDirection,
}

impl<'a> FdInterest<'a> {
    pub(crate) fn readable(token: WaitToken, fd: BorrowedFd<'a>) -> Self {
        Self {
            token,
            fd,
            direction: FdDirection::Read,
        }
    }

    pub(crate) fn writable(token: WaitToken, fd: BorrowedFd<'a>) -> Self {
        Self {
            token,
            fd,
            direction: FdDirection::Write,
        }
    }

    pub(crate) fn fd(self) -> BorrowedFd<'a> {
        self.fd
    }

    pub(crate) fn token(self) -> WaitToken {
        self.token
    }

    pub(crate) fn direction(self) -> FdDirection {
        self.direction
    }
}

pub(crate) struct WaitRequest<'a> {
    sources: Vec<FdInterest<'a>>,
    deadline: Option<Instant>,
    parked: bool,
}

impl<'a> WaitRequest<'a> {
    pub(crate) fn new(sources: Vec<FdInterest<'a>>, deadline: Option<Instant>) -> Self {
        Self {
            sources,
            deadline,
            parked: false,
        }
    }

    /// A task waiting on a value another task produces: no descriptor, no
    /// deadline, resumed by the [`WakeFn`] its driver installs.
    ///
    /// Distinct from an empty [`Self::new`], which describes a task that wants
    /// its next turn rather than one that is blocked.
    pub(crate) fn parked() -> Self {
        Self {
            sources: Vec::new(),
            deadline: None,
            parked: true,
        }
    }

    pub(crate) fn sources(&self) -> &[FdInterest<'a>] {
        &self.sources
    }

    pub(crate) fn deadline(&self) -> Option<Instant> {
        self.deadline
    }

    pub(crate) fn is_parked(&self) -> bool {
        self.parked
    }

    /// Whether the task cannot make progress until something else happens —
    /// either a descriptor it named or the wake it is parked on.
    pub(crate) fn is_blocking(&self) -> bool {
        self.parked || !self.sources.is_empty()
    }
}

#[derive(Default)]
pub(crate) struct ReadySet {
    sources: Vec<WaitToken>,
    timed_out: bool,
}

impl ReadySet {
    // Which source woke a job is no longer consulted by any of them: every
    // remaining job attempts its I/O and reads `WouldBlock` as "not yet". The
    // detail is still carried because the executor's registrations are keyed by
    // source, and goes when they do.
    #[allow(dead_code)]
    pub(crate) fn contains(&self, token: WaitToken) -> bool {
        self.sources.contains(&token)
    }

    #[allow(dead_code)]
    pub(crate) fn timed_out(&self) -> bool {
        self.timed_out
    }

    /// Build readiness from the exact source tokens a driver observed.
    pub(crate) fn from_sources(sources: Vec<WaitToken>, timed_out: bool) -> Self {
        Self { sources, timed_out }
    }

    /// Build readiness for a task with at most one FD source plus a deadline.
    /// Multi-source drivers must report exact source tokens instead.
    pub(crate) fn after_single_source_wakeup(
        request: &WaitRequest<'_>,
        source_ready: bool,
        now: Instant,
    ) -> Self {
        assert!(
            request.sources.len() <= 1,
            "single-source wake helper received multiple descriptors"
        );
        Self {
            sources: source_ready
                .then(|| request.sources.iter().map(|source| source.token).collect())
                .unwrap_or_default(),
            timed_out: request.deadline.is_some_and(|deadline| now >= deadline),
        }
    }
}

/// The value a job the loop owns will produce, and the wake that says it has.
///
/// Both ends live on the loop, so the handoff is a shared slot rather than a
/// descriptor: the producer stores the value and calls the consumer's
/// [`WakeFn`]. Nothing here is pollable, which is why waiting on one describes
/// itself as [`WaitRequest::parked`].
pub struct Completion<T> {
    slot: Rc<RefCell<CompletionSlot<T>>>,
}

/// The producing side of a [`Completion`].
///
/// Dropping it without a value closes the completion, which the consumer reads
/// as the work having stopped without a result.
pub struct CompletionSender<T> {
    slot: Rc<RefCell<CompletionSlot<T>>>,
}

struct CompletionSlot<T> {
    value: Option<T>,
    closed: bool,
    wake: Option<WakeFn>,
}

/// Fallible for its callers' sake only: nothing here can fail now that the
/// pair costs no descriptor.
pub fn completion_pair<T>() -> io::Result<(Completion<T>, CompletionSender<T>)> {
    let slot = Rc::new(RefCell::new(CompletionSlot {
        value: None,
        closed: false,
        wake: None,
    }));
    Ok((
        Completion {
            slot: Rc::clone(&slot),
        },
        CompletionSender { slot },
    ))
}

impl<T> CompletionSender<T> {
    pub fn complete(self, value: T) {
        let wake = {
            let mut slot = self.slot.borrow_mut();
            slot.value = Some(value);
            slot.wake.take()
        };
        // Outside the borrow: the wake reaches a driver that may look at this
        // very completion before returning.
        if let Some(wake) = wake {
            wake();
        }
    }
}

impl<T> Drop for CompletionSender<T> {
    fn drop(&mut self) {
        let wake = {
            let mut slot = self.slot.borrow_mut();
            if slot.value.is_some() {
                return;
            }
            slot.closed = true;
            slot.wake.take()
        };
        if let Some(wake) = wake {
            wake();
        }
    }
}

impl<T> Completion<T> {
    /// Install the wake to call when the value arrives.
    ///
    /// A value that arrived before the driver got here fires the wake at once,
    /// so a completion can never be installed onto too late. That is what lets
    /// a driver (re-)install on whatever schedule suits it instead of having to
    /// interleave with the producer.
    pub(crate) fn set_wake(&mut self, wake: &WakeFn) {
        let mut slot = self.slot.borrow_mut();
        if slot.value.is_some() || slot.closed {
            drop(slot);
            wake();
            return;
        }
        slot.wake = Some(Rc::clone(wake));
    }
}

impl<T> Future for Completion<T> {
    type Output = io::Result<T>;

    /// Awaiting a completion is the same wait a [`Coroutine`] describes as
    /// parked, with the task's own waker standing in for the driver's.
    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let mut slot = self.slot.borrow_mut();
        if let Some(value) = slot.value.take() {
            return Poll::Ready(Ok(value));
        }
        if slot.closed {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "task worker stopped without a result",
            )));
        }
        let waker = context.local_waker().clone();
        slot.wake = Some(Rc::new(move || waker.wake_by_ref()));
        Poll::Pending
    }
}

impl<T> Coroutine for Completion<T> {
    type Output = io::Result<T>;

    fn wait(&self) -> WaitRequest<'_> {
        WaitRequest::parked()
    }

    fn resume(&mut self, _ready: &ReadySet) -> TaskPoll<Self::Output> {
        let mut slot = self.slot.borrow_mut();
        if let Some(value) = slot.value.take() {
            return TaskPoll::Ready(Ok(value));
        }
        if slot.closed {
            return TaskPoll::Ready(Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "task worker stopped without a result",
            )));
        }
        TaskPoll::Pending
    }

    fn set_wake(&mut self, wake: &WakeFn) {
        Completion::set_wake(self, wake);
    }
}

pub(crate) enum TaskPoll<T> {
    Ready(T),
    Pending,
}

impl<T> TaskPoll<T> {
    /// Re-wrap a finished result, leaving `Pending` alone.
    pub(crate) fn map<U>(self, wrap: impl FnOnce(T) -> U) -> TaskPoll<U> {
        match self {
            Self::Ready(value) => TaskPoll::Ready(wrap(value)),
            Self::Pending => TaskPoll::Pending,
        }
    }
}

pub(crate) trait Coroutine {
    type Output;

    fn wait(&self) -> WaitRequest<'_>;
    fn resume(&mut self, ready: &ReadySet) -> TaskPoll<Self::Output>;

    /// Install the wake for whatever this task is parked on, if anything.
    ///
    /// Only a task that can report [`WaitRequest::is_parked`] has to implement
    /// this; a task that waits on descriptors is resumed by its driver's
    /// readiness bookkeeping instead. Installing is idempotent, so a driver may
    /// call it on every turn rather than tracking which suspension is current.
    fn set_wake(&mut self, _wake: &WakeFn) {}
}

pub(crate) struct TaskState<T: Coroutine> {
    task: T,
    output: Option<T::Output>,
}

impl<T: Coroutine> TaskState<T> {
    pub(crate) fn new(task: T) -> Self {
        Self { task, output: None }
    }

    pub(crate) fn wait(&self) -> Option<WaitRequest<'_>> {
        self.output.is_none().then(|| self.task.wait())
    }

    pub(crate) fn task(&self) -> &T {
        &self.task
    }

    /// Install the wake for a task that is still running. A finished task has
    /// nothing left to be woken for.
    pub(crate) fn set_wake(&mut self, wake: &WakeFn) {
        if self.output.is_none() {
            self.task.set_wake(wake);
        }
    }

    pub(crate) fn poll(&mut self, ready: &ReadySet) -> bool {
        if self.output.is_some() {
            return true;
        }
        if let TaskPoll::Ready(output) = self.task.resume(ready) {
            self.output = Some(output);
        }
        self.output.is_some()
    }

    pub(crate) fn poll_after_single_source_wakeup(
        &mut self,
        source_ready: bool,
        now: Instant,
    ) -> bool {
        let ready = {
            let Some(wait) = self.wait() else {
                return true;
            };
            ReadySet::after_single_source_wakeup(&wait, source_ready, now)
        };
        self.poll(&ready)
    }

    /// Offer every descriptor the task waits on as ready, plus its deadline if
    /// it has passed.
    ///
    /// Test scaffolding. A job's own I/O is nonblocking, so a source that is
    /// not in fact ready costs one `EWOULDBLOCK` and leaves the job pending —
    /// which lets a test driver step a multi-source job without reproducing
    /// the reactor's readiness bookkeeping.
    #[cfg(test)]
    pub(crate) fn poll_optimistically(&mut self, now: Instant) -> bool {
        let ready = {
            let Some(wait) = self.wait() else {
                return true;
            };
            ReadySet::from_sources(
                wait.sources().iter().map(|source| source.token()).collect(),
                wait.deadline().is_some_and(|deadline| now >= deadline),
            )
        };
        self.poll(&ready)
    }

    pub(crate) fn take_output(&mut self) -> Option<T::Output> {
        self.output.take()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        completion_pair, Coroutine, FdInterest, ReadySet, TaskPoll, WaitRequest, WaitToken, WakeFn,
    };
    use std::os::fd::AsFd;
    use std::os::unix::net::UnixStream;
    use std::rc::Rc;
    use std::time::{Duration, Instant};

    #[test]
    fn wakeup_reports_an_elapsed_deadline_without_requiring_a_source() {
        let deadline = Instant::now() - Duration::from_millis(1);
        let request = WaitRequest::new(Vec::new(), Some(deadline));

        assert!(ReadySet::after_single_source_wakeup(&request, false, Instant::now()).timed_out());
    }

    #[test]
    fn any_of_wait_distinguishes_source_readiness_from_timeout() {
        let (reader, _writer) = UnixStream::pair().expect("socket pair");
        let token = WaitToken::new(7);
        let expired = WaitRequest::new(
            vec![FdInterest::readable(token, reader.as_fd())],
            Some(Instant::now() - Duration::from_millis(1)),
        );
        let timeout = ReadySet::after_single_source_wakeup(&expired, false, Instant::now());
        assert!(timeout.timed_out());
        assert!(!timeout.contains(token));

        let future = WaitRequest::new(
            vec![FdInterest::readable(token, reader.as_fd())],
            Some(Instant::now() + Duration::from_secs(1)),
        );
        let readable = ReadySet::after_single_source_wakeup(&future, true, Instant::now());
        assert!(!readable.timed_out());
        assert!(readable.contains(token));
    }

    #[test]
    fn completion_is_pending_until_its_sender_finishes() {
        let (mut completion, sender) = completion_pair().expect("completion pair");
        assert!(completion.wait().is_parked());
        assert!(matches!(
            completion.resume(&ReadySet::default()),
            TaskPoll::Pending
        ));

        sender.complete(42);
        assert!(matches!(
            completion.resume(&ReadySet::default()),
            TaskPoll::Ready(Ok(42))
        ));
    }

    #[test]
    fn a_dropped_sender_reports_work_that_stopped() {
        let (mut completion, sender) = completion_pair::<i32>().expect("completion pair");
        drop(sender);
        let TaskPoll::Ready(Err(error)) = completion.resume(&ReadySet::default()) else {
            panic!("a closed completion resolves");
        };
        assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn a_wake_installed_after_the_value_fires_at_once() {
        // The install-order race the fd-backed completion could not have: a
        // driver that reaches the completion only after its producer finished
        // still gets told, so a late install can never wedge the waiter.
        let (mut completion, sender) = completion_pair().expect("completion pair");
        let woken = Rc::new(std::cell::Cell::new(0));
        let counter = Rc::clone(&woken);
        let wake: WakeFn = Rc::new(move || counter.set(counter.get() + 1));

        sender.complete(7);
        assert_eq!(woken.get(), 0, "no wake is installed yet");
        completion.set_wake(&wake);
        assert_eq!(woken.get(), 1);
    }

    #[test]
    fn a_wake_installed_before_the_value_fires_on_completion() {
        let (mut completion, sender) = completion_pair().expect("completion pair");
        let woken = Rc::new(std::cell::Cell::new(0));
        let counter = Rc::clone(&woken);
        let wake: WakeFn = Rc::new(move || counter.set(counter.get() + 1));

        completion.set_wake(&wake);
        assert_eq!(woken.get(), 0);
        sender.complete(7);
        assert_eq!(woken.get(), 1);
        assert!(matches!(
            completion.resume(&ReadySet::default()),
            TaskPoll::Ready(Ok(7))
        ));
    }
}
