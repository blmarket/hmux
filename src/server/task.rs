//! Runtime-neutral resumable tasks and any-of wait descriptions.

use std::cell::RefCell;
use std::io::{self, Read, Write};
use std::os::fd::{AsFd, BorrowedFd};
use std::os::unix::net::UnixStream;
use std::rc::Rc;
use std::time::Instant;

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
}

impl<'a> WaitRequest<'a> {
    pub(crate) fn new(sources: Vec<FdInterest<'a>>, deadline: Option<Instant>) -> Self {
        Self { sources, deadline }
    }

    pub(crate) fn sources(&self) -> &[FdInterest<'a>] {
        &self.sources
    }

    pub(crate) fn deadline(&self) -> Option<Instant> {
        self.deadline
    }
}

#[derive(Default)]
pub(crate) struct ReadySet {
    sources: Vec<WaitToken>,
    timed_out: bool,
}

impl ReadySet {
    #[cfg(test)]
    pub(crate) fn source(token: WaitToken) -> Self {
        Self {
            sources: vec![token],
            timed_out: false,
        }
    }

    pub(crate) fn contains(&self, token: WaitToken) -> bool {
        self.sources.contains(&token)
    }

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

/// Readable completion descriptor returned by a job the loop owns.
///
/// The descriptor is what makes the result pollable; the slot beside it only
/// carries the value between the two ends, both of which live on the loop.
pub(crate) struct Completion<T> {
    reader: UnixStream,
    result: Rc<RefCell<Option<T>>>,
}

/// The producing side of a [`Completion`].
pub(crate) struct CompletionSender<T> {
    writer: UnixStream,
    result: Rc<RefCell<Option<T>>>,
}

pub(crate) fn completion_pair<T>() -> io::Result<(Completion<T>, CompletionSender<T>)> {
    let (reader, writer) = UnixStream::pair()?;
    reader.set_nonblocking(true)?;
    writer.set_nonblocking(true)?;
    let result = Rc::new(RefCell::new(None));
    Ok((
        Completion {
            reader,
            result: Rc::clone(&result),
        },
        CompletionSender { writer, result },
    ))
}

impl<T> CompletionSender<T> {
    pub(crate) fn complete(mut self, value: T) {
        *self.result.borrow_mut() = Some(value);
        let _ = self.writer.write_all(&[1]);
    }
}

impl<T> Completion<T> {
    const COMPLETED: WaitToken = WaitToken::new(0);
}

impl<T> Coroutine for Completion<T> {
    type Output = io::Result<T>;

    fn wait(&self) -> WaitRequest<'_> {
        WaitRequest::new(
            vec![FdInterest::readable(Self::COMPLETED, self.reader.as_fd())],
            None,
        )
    }

    fn resume(&mut self, ready: &ReadySet) -> TaskPoll<Self::Output> {
        if !ready.contains(Self::COMPLETED) {
            return TaskPoll::Pending;
        }
        let mut byte = [0u8; 1];
        match self.reader.read(&mut byte) {
            Ok(0) => TaskPoll::Ready(Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "task worker stopped without a result",
            ))),
            Ok(_) => TaskPoll::Ready(
                self.result
                    .borrow_mut()
                    .take()
                    .ok_or_else(|| io::Error::other("task completed without a result")),
            ),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => TaskPoll::Pending,
            Err(error) => TaskPoll::Ready(Err(error)),
        }
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
        completion_pair, Coroutine, FdInterest, ReadySet, TaskPoll, WaitRequest, WaitToken,
    };
    use std::os::fd::AsFd;
    use std::os::unix::net::UnixStream;
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
    fn completion_is_nonblocking_until_its_sender_finishes() {
        let (mut completion, sender) = completion_pair().expect("completion pair");
        assert!(matches!(
            completion.resume(&ReadySet::default()),
            TaskPoll::Pending
        ));

        sender.complete(42);
        assert!(matches!(
            completion.resume(&ReadySet::source(super::Completion::<i32>::COMPLETED)),
            TaskPoll::Ready(Ok(42))
        ));
    }
}
