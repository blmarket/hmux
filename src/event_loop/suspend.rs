//! Loop-owned execution of command work that has to wait.
//!
//! A suspended command queue waits for something the loop can poll: a child
//! process, a pipe, a deadline. The executor owns that work directly instead of
//! handing each piece to a thread — the loop registers a job's descriptors and
//! deadline with its reactor and resumes the job when they are ready.
//!
//! Two kinds of work arrive here. A client's suspension is submitted by its
//! command runtime and reported back through the fd-backed [`Completion`] the
//! protocol drivers already poll. A detached (`-b`) command queue has no client
//! waiting on a descriptor, so the loop owns the whole queue and hands its
//! result straight to the background-command actor.
//!
//! The suspension jobs themselves live with the commands
//! ([`SuspensionJob`](crate::server::command::suspend::SuspensionJob)) so the
//! blocking test driver resolves a suspension exactly the way the loop does.

use std::collections::BTreeMap;
use std::io;
use std::os::fd::RawFd;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::server::command::suspend::SuspensionJob;
use crate::server::command::{
    CommandCoroutine, CommandResult, CommandRuntime, CommandSuspension, CommandSuspensionResult,
};
use crate::server::status::FormatJob;
use crate::server::task::{
    completion_pair, Completion, CompletionSender, Coroutine, FdDirection, ReadySet, TaskPoll,
    TaskState, WaitRequest, WaitToken,
};

use super::actor::ActorRef;
use super::driver::{Envelope, Outbox};
use super::job::{BackgroundCommands, JobEvent};
use super::reactor::Token;
use super::timer::TimerId;

/// Command runtime for clients served by the event loop.
///
/// Every suspension is resolved by the loop's [`SuspensionExecutor`], which
/// drives it as one of its own jobs.
#[derive(Clone)]
pub(crate) struct EventCommandRuntime {
    executor: SuspensionExecutorHandle,
}

impl EventCommandRuntime {
    pub(crate) fn new(executor: SuspensionExecutorHandle) -> Self {
        Self { executor }
    }
}

impl CommandRuntime for EventCommandRuntime {
    fn submit(
        &self,
        suspension: CommandSuspension,
    ) -> io::Result<Completion<CommandSuspensionResult>> {
        self.executor.submit(suspension)
    }
}

/// Work addressed to one job the executor owns.
pub(crate) enum ExecutorEvent {
    /// One of a job's registered descriptors reported readiness.
    Ready { job: u64, source: WaitToken },
    /// A job's deadline elapsed.
    Timeout { job: u64 },
}

/// One piece of waiting work the loop drives to completion.
pub(crate) enum ExecutorJob {
    /// One suspension of a command queue someone else is driving.
    Suspension(SuspensionJob),
    /// A detached command queue the loop owns outright.
    Queue(CommandCoroutine),
    /// A `#()` format job, which publishes to its registry as it reads.
    Format(FormatJob),
}

pub(crate) enum ExecutorJobOutput {
    Suspension(CommandSuspensionResult),
    Queue(io::Result<CommandResult>),
    Format,
}

impl Coroutine for ExecutorJob {
    type Output = ExecutorJobOutput;

    fn wait(&self) -> WaitRequest<'_> {
        match self {
            Self::Suspension(job) => job.wait(),
            Self::Queue(queue) => queue.wait(),
            Self::Format(job) => job.wait(),
        }
    }

    fn resume(&mut self, ready: &ReadySet) -> TaskPoll<Self::Output> {
        match self {
            Self::Suspension(job) => job.resume(ready).map(ExecutorJobOutput::Suspension),
            Self::Queue(queue) => queue.resume(ready).map(ExecutorJobOutput::Queue),
            Self::Format(job) => job.resume(ready).map(|()| ExecutorJobOutput::Format),
        }
    }
}

/// Where a finished job reports its result.
enum JobSink {
    /// Readable to the command queue that is suspended on it.
    Completion(CompletionSender<CommandSuspensionResult>),
    /// Delivered to the background-command actor as a loop event.
    Background {
        target: ActorRef<BackgroundCommands>,
        id: u64,
    },
    /// Nothing waits on the result; the job publishes its own.
    Detached,
}

impl JobSink {
    fn deliver(self, output: Option<ExecutorJobOutput>, outbox: &mut Outbox) {
        match (self, output) {
            (Self::Completion(sender), Some(ExecutorJobOutput::Suspension(result))) => {
                sender.complete(result);
            }
            // A job that reported nothing leaves its waiter to see the dropped
            // sender, which reads as the work having stopped without a result.
            (Self::Completion(sender), _) => drop(sender),
            (Self::Background { target, id }, output) => {
                outbox.enqueue(Envelope::Background {
                    target,
                    event: JobEvent::JobFinished { id, output },
                });
            }
            (Self::Detached, _) => {}
        }
    }
}

/// A suspension handed to the executor but not yet adopted by the loop.
struct SubmittedJob {
    job: SuspensionJob,
    sender: CompletionSender<CommandSuspensionResult>,
}

/// One reactor registration made on behalf of a job's wait source.
#[derive(Clone, Copy)]
pub(super) struct JobRegistration {
    pub(super) source: WaitToken,
    pub(super) fd: RawFd,
    pub(super) direction: FdDirection,
    pub(super) token: Token,
}

/// A job the loop has adopted, with the registrations made for it.
pub(super) struct ExecutorJobState {
    pub(super) task: TaskState<ExecutorJob>,
    pub(super) registrations: Vec<JobRegistration>,
    pub(super) timer: Option<(Instant, TimerId)>,
    sink: Option<JobSink>,
}

impl ExecutorJobState {
    /// Whether the job has already reported its result and is only waiting to
    /// have its registrations released.
    pub(super) fn is_finished(&self) -> bool {
        self.sink.is_none()
    }

    /// Whether `registrations` already describes exactly `sources`.
    pub(super) fn is_registered_for(&self, sources: &[(WaitToken, RawFd, FdDirection)]) -> bool {
        self.registrations.len() == sources.len()
            && self.registrations.iter().zip(sources).all(
                |(registration, (source, fd, direction))| {
                    registration.source == *source
                        && registration.fd == *fd
                        && registration.direction == *direction
                },
            )
    }
}

/// Submission side of the executor, held by the command runtimes.
#[derive(Clone)]
pub(crate) struct SuspensionExecutorHandle {
    submitted: Arc<Mutex<Vec<SubmittedJob>>>,
}

impl SuspensionExecutorHandle {
    /// Hand one suspension to the loop, which resolves it as a job of its own.
    pub(crate) fn submit(
        &self,
        suspension: CommandSuspension,
    ) -> io::Result<Completion<CommandSuspensionResult>> {
        self.enqueue(SuspensionJob::new(suspension))
    }

    fn enqueue(&self, job: SuspensionJob) -> io::Result<Completion<CommandSuspensionResult>> {
        let (completion, sender) = completion_pair()?;
        self.submitted
            .lock()
            .map_err(|_| io::Error::other("suspension executor inbox poisoned"))?
            .push(SubmittedJob { job, sender });
        // Every submitter runs on the loop, inside a dispatch the executor may
        // itself be driving, so the job is handed over through this inbox and
        // adopted by the next sync rather than by re-entering the executor.
        Ok(completion)
    }
}

/// Loop-owned driver for every adopted suspension job.
pub(crate) struct SuspensionExecutor {
    submitted: Arc<Mutex<Vec<SubmittedJob>>>,
    jobs: BTreeMap<u64, ExecutorJobState>,
    next_id: u64,
}

impl SuspensionExecutor {
    pub(crate) fn new() -> (Self, SuspensionExecutorHandle) {
        let submitted = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                submitted: Arc::clone(&submitted),
                jobs: BTreeMap::new(),
                next_id: 1,
            },
            SuspensionExecutorHandle { submitted },
        )
    }

    pub(crate) fn handle(&mut self, event: ExecutorEvent, outbox: &mut Outbox) {
        match event {
            ExecutorEvent::Ready { job, source } => {
                self.poll_job(job, &ReadySet::from_sources(vec![source], false), outbox);
            }
            ExecutorEvent::Timeout { job } => {
                // The timer that produced this event is gone; forget it before
                // the job asks for its next deadline.
                if let Some(state) = self.jobs.get_mut(&job) {
                    state.timer = None;
                }
                self.poll_job(job, &ReadySet::from_sources(Vec::new(), true), outbox);
            }
        }
    }

    /// Take ownership of everything submitted since the last sync and give each
    /// new job its first turn.
    pub(super) fn adopt_submitted(&mut self, outbox: &mut Outbox) {
        let submitted = match self.submitted.lock() {
            Ok(mut submitted) => std::mem::take(&mut *submitted),
            Err(_) => return,
        };
        for SubmittedJob { job, sender } in submitted {
            self.adopt(
                ExecutorJob::Suspension(job),
                JobSink::Completion(sender),
                outbox,
            );
        }
    }

    /// Take on one detached command queue, which the loop owns from here.
    pub(crate) fn adopt_background_queue(
        &mut self,
        queue: CommandCoroutine,
        target: ActorRef<BackgroundCommands>,
        id: u64,
        outbox: &mut Outbox,
    ) {
        self.adopt(
            ExecutorJob::Queue(queue),
            JobSink::Background { target, id },
            outbox,
        );
    }

    /// Take on one job the background-command actor needs resolved before it
    /// can decide what to run.
    pub(crate) fn adopt_background_suspension(
        &mut self,
        job: SuspensionJob,
        target: ActorRef<BackgroundCommands>,
        id: u64,
        outbox: &mut Outbox,
    ) {
        self.adopt(
            ExecutorJob::Suspension(job),
            JobSink::Background { target, id },
            outbox,
        );
    }

    /// Take on one `#()` job the format registry has already spawned.
    pub(super) fn adopt_format_job(&mut self, job: FormatJob, outbox: &mut Outbox) {
        self.adopt(ExecutorJob::Format(job), JobSink::Detached, outbox);
    }

    fn adopt(&mut self, job: ExecutorJob, sink: JobSink, outbox: &mut Outbox) {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        self.jobs.insert(
            id,
            ExecutorJobState {
                task: TaskState::new(job),
                registrations: Vec::new(),
                timer: None,
                sink: Some(sink),
            },
        );
        self.poll_job(id, &ReadySet::default(), outbox);
    }

    pub(super) fn job_ids(&self) -> Vec<u64> {
        self.jobs.keys().copied().collect()
    }

    pub(super) fn job_mut(&mut self, id: u64) -> Option<&mut ExecutorJobState> {
        self.jobs.get_mut(&id)
    }

    /// Retire a job that has reported its result; its registrations have
    /// already been released by the loop.
    pub(super) fn remove_job(&mut self, id: u64) {
        self.jobs.remove(&id);
    }

    fn poll_job(&mut self, id: u64, ready: &ReadySet, outbox: &mut Outbox) {
        let Some(state) = self.jobs.get_mut(&id) else {
            return;
        };
        // Readiness queued before the job finished still names a live job id.
        if state.is_finished() || !state.task.poll(ready) {
            return;
        }
        let Some(sink) = state.sink.take() else {
            return;
        };
        sink.deliver(state.task.take_output(), outbox);
    }
}
