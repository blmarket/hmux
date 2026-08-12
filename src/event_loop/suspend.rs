//! Loop-owned execution of command work that has to wait.
//!
//! A suspended command queue waits for something the loop can poll: a child
//! process, a pipe, a deadline. The executor owns that work directly instead of
//! handing each piece to a thread — the loop registers a job's descriptors and
//! deadline with its reactor and resumes the job when they are ready.
//!
//! Two kinds of work arrive here. A client's suspension is submitted by its
//! command runtime and reported back through a [`Completion`], which wakes the
//! client rather than making a descriptor readable — both ends of it live on
//! this loop. A detached (`-b`) command queue has no client waiting at all, so
//! the loop owns the whole queue and hands its result straight to the
//! background-command actor.
//!
//! The suspension jobs themselves live with the commands
//! ([`SuspensionJob`](crate::server::command::suspend::SuspensionJob)) so the
//! blocking test driver resolves a suspension exactly the way the loop does.

use std::collections::BTreeMap;
use std::io;
use std::os::fd::RawFd;
use std::time::Instant;

use std::rc::Rc;

use crate::server::command::suspend::{self, SuspensionStart, SuspensionWait};
use crate::server::command::{
    run_command_queue, CommandResult, CommandRuntime, CommandSuspension, CommandSuspensionResult,
    QueueStatus, QueuedCommand, ResumableCommandQueue,
};
use crate::server::pane::PanePipeIo;
use crate::server::status::FormatJob;
use crate::server::task::{
    completion_pair, Completion, CompletionSender, Coroutine, FdDirection, ReadySet, TaskPoll,
    TaskState, WaitRequest, WaitToken, WakeFn,
};

use super::actor::ActorRef;
use super::driver::{Envelope, Outbox};
use super::job::{BackgroundCommands, JobEvent};
use super::reactor::Token;
use super::tasks::TaskHandle;
use super::timer::TimerId;

/// Command runtime for clients served by the event loop.
///
/// Every suspension is resolved by the loop's [`SuspensionExecutor`], which
/// drives it as one of its own jobs.
#[derive(Clone)]
pub(crate) struct EventCommandRuntime {
    tasks: TaskHandle,
}

impl EventCommandRuntime {
    pub(crate) fn new(tasks: TaskHandle) -> Self {
        Self { tasks }
    }
}

impl CommandRuntime for EventCommandRuntime {
    /// Every suspension is a task on the loop's task set, reporting through the
    /// completion the queue is parked on.
    ///
    /// The two registry-backed suspensions do their registry work here rather
    /// than in their task: the order they touch a registry in is the order the
    /// commands ran in, which a deferred first turn would not preserve.
    fn spawn_queue(
        &self,
        queue: ResumableCommandQueue,
        state: crate::server::state::SharedState,
        budget: usize,
    ) -> io::Result<QueuedCommand> {
        self.spawn_command_queue(queue, state, budget)
    }

    fn submit(
        &self,
        suspension: CommandSuspension,
    ) -> io::Result<Completion<CommandSuspensionResult>> {
        let (completion, sender) = completion_pair()?;
        let tasks = self.tasks.clone();
        match suspension {
            CommandSuspension::RunShell { args, context } => {
                self.tasks.spawn(async move {
                    let output = suspend::run_shell(&tasks, args, context).await;
                    sender.complete(CommandSuspensionResult::RunShell(output));
                });
            }
            CommandSuspension::IfShell { condition, context } => {
                self.tasks.spawn(async move {
                    let matched = suspend::if_shell(&tasks, condition, context).await;
                    sender.complete(CommandSuspensionResult::IfShell(matched));
                });
            }
            CommandSuspension::SourceFile { paths } => {
                self.tasks.spawn(async move {
                    let reads = suspend::source_file(&tasks, paths).await;
                    sender.complete(CommandSuspensionResult::SourceFile(reads));
                });
            }
            CommandSuspension::LoadBuffer { path } => {
                self.tasks.spawn(async move {
                    let contents = suspend::load_buffer(&tasks, path).await;
                    sender.complete(CommandSuspensionResult::LoadBuffer(contents));
                });
            }
            CommandSuspension::SaveBuffer { request } => {
                self.tasks.spawn(async move {
                    let result = suspend::save_buffer(&tasks, request).await;
                    sender.complete(CommandSuspensionResult::SaveBuffer(result));
                });
            }
            CommandSuspension::WaitFor { args, registry } => {
                self.start(suspend::wait_for(&args, &registry), sender);
            }
            CommandSuspension::CommandPrompt {
                args,
                registry,
                target,
                tty_name,
                wait,
            } => {
                self.start(
                    suspend::client_prompt(args, &registry, target, tty_name, wait),
                    sender,
                );
            }
            CommandSuspension::ClientInteraction { completed } => {
                self.start(
                    SuspensionStart::Waiting(SuspensionWait::Interaction(completed)),
                    sender,
                );
            }
        }
        Ok(completion)
    }
}

impl EventCommandRuntime {
    /// Report a registry-backed suspension: at once if it already finished,
    /// otherwise from a task that waits for the answer.
    fn start(&self, start: SuspensionStart, sender: CompletionSender<CommandSuspensionResult>) {
        match start {
            SuspensionStart::Ready(result) => {
                sender.complete(CommandSuspensionResult::Completed(result));
            }
            SuspensionStart::Waiting(wait) => {
                self.tasks.spawn(async move {
                    sender.complete(CommandSuspensionResult::Completed(wait.resolve().await));
                });
            }
        }
    }
}

impl EventCommandRuntime {
    fn spawn_command_queue(
        &self,
        queue: ResumableCommandQueue,
        state: crate::server::state::SharedState,
        budget: usize,
    ) -> io::Result<QueuedCommand> {
        let (completion, sender) = completion_pair()?;
        let status = Rc::new(QueueStatus::default());
        let runtime: Rc<dyn CommandRuntime> = Rc::new(self.clone());
        let queue_status = Rc::clone(&status);
        self.tasks.spawn_now(async move {
            let result = run_command_queue(queue, state, runtime, budget, queue_status).await;
            sender.complete(result);
        });
        Ok(QueuedCommand::new(completion, status))
    }
}

/// Work addressed to one job the executor owns.
pub(crate) enum ExecutorEvent {
    /// One of a job's registered descriptors reported readiness.
    Ready { job: u64, source: WaitToken },
    /// A job's deadline elapsed.
    Timeout { job: u64 },
    /// A completion the job was parked on has a value.
    Wake { job: u64 },
}

/// One piece of waiting work the loop drives to completion.
pub(crate) enum ExecutorJob {
    /// A detached command queue running as a task; the executor holds only the
    /// handle its result arrives through.
    Queue(QueuedCommand),
    /// A suspension running as a task on the loop's task set. The executor
    /// contributes nothing but the wait: it holds the completion so the result
    /// still reaches the background-command actor through the usual sink.
    Awaiting(Completion<CommandSuspensionResult>),
    /// A `#()` format job, which publishes to its registry as it reads.
    Format(FormatJob),
    /// An open `pipe-pane` child, in both directions.
    PanePipe(PanePipeIo),
}

pub(crate) enum ExecutorJobOutput {
    Suspension(CommandSuspensionResult),
    /// The work stopped without a result, which only happens as the server
    /// goes away.
    Cancelled,
    Queue(io::Result<CommandResult>),
    Format,
    PanePipe,
}

impl Coroutine for ExecutorJob {
    type Output = ExecutorJobOutput;

    fn wait(&self) -> WaitRequest<'_> {
        match self {
            Self::Queue(queue) => queue.wait(),
            Self::Awaiting(completion) => completion.wait(),
            Self::Format(job) => job.wait(),
            Self::PanePipe(pipe) => pipe.wait(),
        }
    }

    fn resume(&mut self, ready: &ReadySet) -> TaskPoll<Self::Output> {
        match self {
            Self::Queue(queue) => queue.resume(ready).map(ExecutorJobOutput::Queue),
            Self::Awaiting(completion) => completion.resume(ready).map(|result| match result {
                Ok(result) => ExecutorJobOutput::Suspension(result),
                Err(_) => ExecutorJobOutput::Cancelled,
            }),
            Self::Format(job) => job.resume(ready).map(|()| ExecutorJobOutput::Format),
            Self::PanePipe(pipe) => pipe.resume(ready).map(|()| ExecutorJobOutput::PanePipe),
        }
    }

    /// A format job and a `pipe-pane` child only ever wait on their own
    /// descriptors; the other two can be parked on a completion.
    fn set_wake(&mut self, wake: &WakeFn) {
        match self {
            Self::Queue(queue) => queue.set_wake(wake),
            Self::Awaiting(completion) => completion.set_wake(wake),
            Self::Format(_) | Self::PanePipe(_) => {}
        }
    }
}

/// Where a finished job reports its result.
enum JobSink {
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
    /// The wake the loop hands a parked job, built once and re-installed on
    /// every suspension this job goes through.
    pub(super) wake: Option<WakeFn>,
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

/// Loop-owned driver for every adopted suspension job.
pub(crate) struct SuspensionExecutor {
    jobs: BTreeMap<u64, ExecutorJobState>,
    next_id: u64,
}

impl SuspensionExecutor {
    pub(crate) fn new() -> Self {
        Self {
            jobs: BTreeMap::new(),
            next_id: 1,
        }
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
            // A parked job reads its value out of the completion itself, so
            // the wake carries no readiness of its own.
            ExecutorEvent::Wake { job } => {
                self.poll_job(job, &ReadySet::default(), outbox);
            }
        }
    }

    /// Take on one detached command queue's handle, so its result reaches the
    /// background-command actor.
    pub(crate) fn adopt_background_queue(
        &mut self,
        queue: QueuedCommand,
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

    /// Take on one suspension the background-command actor needs resolved
    /// before it can decide what to run, and which runs as a task.
    pub(crate) fn adopt_background_awaiting(
        &mut self,
        completion: Completion<CommandSuspensionResult>,
        target: ActorRef<BackgroundCommands>,
        id: u64,
        outbox: &mut Outbox,
    ) {
        self.adopt(
            ExecutorJob::Awaiting(completion),
            JobSink::Background { target, id },
            outbox,
        );
    }

    /// Take on one `#()` job the format registry has already spawned.
    pub(super) fn adopt_format_job(&mut self, job: FormatJob, outbox: &mut Outbox) {
        self.adopt(ExecutorJob::Format(job), JobSink::Detached, outbox);
    }

    /// Take on one `pipe-pane` child the pane has already spawned.
    pub(super) fn adopt_pane_pipe(&mut self, pipe: PanePipeIo, outbox: &mut Outbox) {
        self.adopt(ExecutorJob::PanePipe(pipe), JobSink::Detached, outbox);
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
                wake: None,
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
