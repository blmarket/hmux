//! Loop-owned execution of command suspensions.
//!
//! A suspended command queue waits for something the loop can poll: a child
//! process, a pipe, a deadline. The executor owns those jobs directly instead
//! of handing each one to a thread — the loop registers a job's descriptors
//! and deadline with its reactor, resumes the job when they are ready, and
//! reports the result through the same fd-backed [`Completion`] the protocol
//! drivers already poll for a suspended command.
//!
//! Suspension variants the executor does not own yet are handed back to
//! [`EventCommandRuntime`](super::task::EventCommandRuntime), which still runs
//! them on a worker thread.

use std::collections::BTreeMap;
use std::io;
use std::os::fd::RawFd;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::server::command::suspend::{
    FileWriteJob, IfShellJob, LoadBufferJob, RunShellJob, SourceFileJob, WaitForJob,
};
use crate::server::command::{CommandSuspension, CommandSuspensionResult};
use crate::server::task::{
    completion_pair, Completion, CompletionSender, Coroutine, FdDirection, ReadySet, TaskPoll,
    TaskState, WaitRequest, WaitToken,
};

use super::reactor::{Token, WakeHandle};
use super::timer::TimerId;

/// Work addressed to one job the executor owns.
pub(crate) enum ExecutorEvent {
    /// One of a job's registered descriptors reported readiness.
    Ready { job: u64, source: WaitToken },
    /// A job's deadline elapsed.
    Timeout { job: u64 },
}

/// One suspension the loop drives to its [`CommandSuspensionResult`].
pub(super) enum ExecutorJob {
    RunShell(RunShellJob),
    IfShell(IfShellJob),
    SourceFile(SourceFileJob),
    LoadBuffer(LoadBufferJob),
    SaveBuffer(FileWriteJob),
    WaitFor(WaitForJob),
}

impl Coroutine for ExecutorJob {
    type Output = CommandSuspensionResult;

    fn wait(&self) -> WaitRequest<'_> {
        match self {
            Self::RunShell(job) => job.wait(),
            Self::IfShell(job) => job.wait(),
            Self::SourceFile(job) => job.wait(),
            Self::LoadBuffer(job) => job.wait(),
            Self::SaveBuffer(job) => job.wait(),
            Self::WaitFor(job) => job.wait(),
        }
    }

    fn resume(&mut self, ready: &ReadySet) -> TaskPoll<Self::Output> {
        match self {
            Self::RunShell(job) => match job.resume(ready) {
                TaskPoll::Ready(completion) => {
                    TaskPoll::Ready(CommandSuspensionResult::RunShell(completion))
                }
                TaskPoll::Pending => TaskPoll::Pending,
            },
            Self::IfShell(job) => match job.resume(ready) {
                TaskPoll::Ready(status) => {
                    TaskPoll::Ready(CommandSuspensionResult::IfShell(status))
                }
                TaskPoll::Pending => TaskPoll::Pending,
            },
            Self::SourceFile(job) => match job.resume(ready) {
                TaskPoll::Ready(reads) => {
                    TaskPoll::Ready(CommandSuspensionResult::SourceFile(reads))
                }
                TaskPoll::Pending => TaskPoll::Pending,
            },
            Self::LoadBuffer(job) => match job.resume(ready) {
                TaskPoll::Ready(contents) => {
                    TaskPoll::Ready(CommandSuspensionResult::LoadBuffer(contents))
                }
                TaskPoll::Pending => TaskPoll::Pending,
            },
            Self::SaveBuffer(job) => match job.resume(ready) {
                TaskPoll::Ready(result) => {
                    TaskPoll::Ready(CommandSuspensionResult::SaveBuffer(result))
                }
                TaskPoll::Pending => TaskPoll::Pending,
            },
            Self::WaitFor(job) => match job.resume(ready) {
                TaskPoll::Ready(result) => {
                    TaskPoll::Ready(CommandSuspensionResult::Completed(result))
                }
                TaskPoll::Pending => TaskPoll::Pending,
            },
        }
    }
}

/// A job handed to the executor but not yet adopted by the loop.
struct SubmittedJob {
    job: ExecutorJob,
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
    sender: Option<CompletionSender<CommandSuspensionResult>>,
}

impl ExecutorJobState {
    /// Whether the job has already reported its result and is only waiting to
    /// have its registrations released.
    pub(super) fn is_finished(&self) -> bool {
        self.sender.is_none()
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
    wake: WakeHandle,
}

impl SuspensionExecutorHandle {
    /// Hand one suspension to the loop. A variant the executor does not own is
    /// returned unchanged so the caller can fall back to a worker thread.
    pub(crate) fn submit(
        &self,
        suspension: CommandSuspension,
    ) -> Result<io::Result<Completion<CommandSuspensionResult>>, CommandSuspension> {
        let job = match suspension {
            CommandSuspension::RunShell { args, context } => {
                ExecutorJob::RunShell(RunShellJob::new(&args, &context))
            }
            CommandSuspension::IfShell { condition, context } => {
                ExecutorJob::IfShell(IfShellJob::new(&condition, &context))
            }
            CommandSuspension::SourceFile { paths } => {
                ExecutorJob::SourceFile(SourceFileJob::new(paths))
            }
            CommandSuspension::LoadBuffer { path } => {
                ExecutorJob::LoadBuffer(LoadBufferJob::new(path))
            }
            CommandSuspension::SaveBuffer { request } => {
                ExecutorJob::SaveBuffer(FileWriteJob::new(request))
            }
            CommandSuspension::WaitFor { args, registry } => {
                ExecutorJob::WaitFor(WaitForJob::new(&args, &registry))
            }
            suspension => return Err(suspension),
        };
        Ok(self.enqueue(job))
    }

    fn enqueue(&self, job: ExecutorJob) -> io::Result<Completion<CommandSuspensionResult>> {
        let (completion, sender) = completion_pair()?;
        self.submitted
            .lock()
            .map_err(|_| io::Error::other("suspension executor inbox poisoned"))?
            .push(SubmittedJob { job, sender });
        // A client command suspends while the loop is already running its own
        // dispatch; only a background command queue, which still runs on a
        // worker thread, can submit while the loop sits in `poll`.
        let _ = self.wake.wake();
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
    pub(crate) fn new(wake: WakeHandle) -> (Self, SuspensionExecutorHandle) {
        let submitted = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                submitted: Arc::clone(&submitted),
                jobs: BTreeMap::new(),
                next_id: 1,
            },
            SuspensionExecutorHandle { submitted, wake },
        )
    }

    pub(crate) fn handle(&mut self, event: ExecutorEvent) {
        match event {
            ExecutorEvent::Ready { job, source } => {
                self.poll_job(job, &ReadySet::from_sources(vec![source], false));
            }
            ExecutorEvent::Timeout { job } => {
                // The timer that produced this event is gone; forget it before
                // the job asks for its next deadline.
                if let Some(state) = self.jobs.get_mut(&job) {
                    state.timer = None;
                }
                self.poll_job(job, &ReadySet::from_sources(Vec::new(), true));
            }
        }
    }

    /// Take ownership of everything submitted since the last sync and give each
    /// new job its first turn.
    pub(super) fn adopt_submitted(&mut self) {
        let submitted = match self.submitted.lock() {
            Ok(mut submitted) => std::mem::take(&mut *submitted),
            Err(_) => return,
        };
        for SubmittedJob { job, sender } in submitted {
            let id = self.next_id;
            self.next_id = self.next_id.wrapping_add(1).max(1);
            self.jobs.insert(
                id,
                ExecutorJobState {
                    task: TaskState::new(job),
                    registrations: Vec::new(),
                    timer: None,
                    sender: Some(sender),
                },
            );
            self.poll_job(id, &ReadySet::default());
        }
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

    fn poll_job(&mut self, id: u64, ready: &ReadySet) {
        let Some(state) = self.jobs.get_mut(&id) else {
            return;
        };
        // Readiness queued before the job finished still names a live job id.
        if state.is_finished() || !state.task.poll(ready) {
            return;
        }
        let Some(sender) = state.sender.take() else {
            return;
        };
        match state.task.take_output() {
            Some(result) => sender.complete(result),
            None => drop(sender),
        }
    }
}
