//! Runtime-neutral resumable tasks and any-of wait descriptions.

use std::io::{self, Read, Write};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct WaitToken(u32);

impl WaitToken {
    pub(crate) const fn new(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FdInterest<'a> {
    token: WaitToken,
    fd: BorrowedFd<'a>,
}

impl<'a> FdInterest<'a> {
    pub(crate) fn readable(token: WaitToken, fd: BorrowedFd<'a>) -> Self {
        Self { token, fd }
    }

    pub(crate) fn fd(self) -> BorrowedFd<'a> {
        self.fd
    }

    pub(crate) fn token(self) -> WaitToken {
        self.token
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

/// Readable completion descriptor returned by a runtime-owned worker.
pub(crate) struct Completion<T> {
    reader: UnixStream,
    result: Arc<Mutex<Option<T>>>,
}

/// The worker side of a [`Completion`].
pub(crate) struct CompletionSender<T> {
    writer: UnixStream,
    result: Arc<Mutex<Option<T>>>,
}

pub(crate) fn completion_pair<T>() -> io::Result<(Completion<T>, CompletionSender<T>)> {
    let (reader, writer) = UnixStream::pair()?;
    reader.set_nonblocking(true)?;
    writer.set_nonblocking(true)?;
    let result = Arc::new(Mutex::new(None));
    Ok((
        Completion {
            reader,
            result: Arc::clone(&result),
        },
        CompletionSender { writer, result },
    ))
}

impl<T> CompletionSender<T> {
    pub(crate) fn complete(mut self, value: T) {
        if let Ok(mut result) = self.result.lock() {
            *result = Some(value);
        }
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
            Ok(_) => match self.result.lock() {
                Ok(mut result) => TaskPoll::Ready(
                    result
                        .take()
                        .ok_or_else(|| io::Error::other("task completed without a result")),
                ),
                Err(_) => TaskPoll::Ready(Err(io::Error::other("task result poisoned"))),
            },
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => TaskPoll::Pending,
            Err(error) => TaskPoll::Ready(Err(error)),
        }
    }
}

pub(crate) enum TaskPoll<T> {
    Ready(T),
    Pending,
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

    pub(crate) fn take_output(&mut self) -> Option<T::Output> {
        self.output.take()
    }
}

/// Drive a coroutine to completion on the calling thread using `poll(2)`.
///
/// This is the blocking counterpart of the server loop's reactor: worker
/// threads and unit-test drivers run the very same coroutines the loop polls,
/// so a job only ever has one implementation.
pub(crate) fn drive_blocking<T: Coroutine>(state: &mut TaskState<T>) {
    state.poll(&ReadySet::default());
    loop {
        let ready = {
            let Some(request) = state.wait() else { break };
            poll_wait_request(&request)
        };
        state.poll(&ready);
    }
}

/// Run one coroutine to completion on the calling thread and take its output.
pub(crate) fn run_blocking<T: Coroutine>(task: T) -> T::Output {
    let mut state = TaskState::new(task);
    drive_blocking(&mut state);
    state
        .take_output()
        .expect("completed coroutine has a result")
}

/// Block until one of `request`'s sources is ready or its deadline elapses,
/// reporting the exact tokens that woke the wait.
fn poll_wait_request(request: &WaitRequest<'_>) -> ReadySet {
    let mut descriptors: Vec<libc::pollfd> = request
        .sources()
        .iter()
        .map(|source| libc::pollfd {
            fd: source.fd().as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        })
        .collect();
    let count = libc::nfds_t::try_from(descriptors.len()).expect("wait request fits in nfds_t");
    let (pointer, count) = if descriptors.is_empty() {
        (std::ptr::null_mut(), 0)
    } else {
        (descriptors.as_mut_ptr(), count)
    };
    loop {
        let timeout = request.deadline().map_or(-1, |deadline| {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let millis = remaining.as_nanos().saturating_add(999_999) / 1_000_000;
            i32::try_from(millis).unwrap_or(i32::MAX)
        });
        let woken = unsafe { libc::poll(pointer, count, timeout) };
        if woken < 0 && io::Error::last_os_error().kind() == io::ErrorKind::Interrupted {
            continue;
        }
        let sources = if woken > 0 {
            descriptors
                .iter()
                .zip(request.sources())
                .filter(|(descriptor, _)| descriptor.revents != 0)
                .map(|(_, source)| source.token())
                .collect()
        } else {
            Vec::new()
        };
        return ReadySet::from_sources(
            sources,
            request
                .deadline()
                .is_some_and(|deadline| Instant::now() >= deadline),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{
        completion_pair, poll_wait_request, Coroutine, FdInterest, ReadySet, TaskPoll, WaitRequest,
        WaitToken,
    };
    use std::io::Write;
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
    fn multi_source_wait_reports_only_the_ready_descriptor() {
        let (idle, _idle_writer) = UnixStream::pair().expect("socket pair");
        let (ready, mut ready_writer) = UnixStream::pair().expect("socket pair");
        ready_writer.write_all(b"x").expect("write");
        let idle_token = WaitToken::new(1);
        let ready_token = WaitToken::new(2);
        let request = WaitRequest::new(
            vec![
                FdInterest::readable(idle_token, idle.as_fd()),
                FdInterest::readable(ready_token, ready.as_fd()),
            ],
            None,
        );

        let woken = poll_wait_request(&request);
        assert!(woken.contains(ready_token));
        assert!(!woken.contains(idle_token));
        assert!(!woken.timed_out());
    }

    #[test]
    fn multi_source_wait_reports_a_timeout_with_no_ready_descriptor() {
        let (idle, _idle_writer) = UnixStream::pair().expect("socket pair");
        let request = WaitRequest::new(
            vec![FdInterest::readable(WaitToken::new(1), idle.as_fd())],
            Some(Instant::now() + Duration::from_millis(5)),
        );

        let woken = poll_wait_request(&request);
        assert!(woken.timed_out());
        assert!(!woken.contains(WaitToken::new(1)));
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
