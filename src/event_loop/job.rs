//! Event-loop-owned detached command queues.
//!
//! A `-b` command has no client waiting on it, so nothing outside the loop
//! needs to observe its progress: the queue is handed to the suspension
//! executor as a job of its own and reports back here when it finishes.

use std::collections::BTreeMap;
use std::rc::Rc;

use crate::integration::status::StatusHub;
use crate::server::command::{
    self, BackgroundCommand, BackgroundCommandRequest, ClientContext, PendingBackground,
};
use crate::server::state::SharedState;

use super::actor::ActorRef;
use super::driver::Outbox;
use super::suspend::{
    EventCommandRuntime, ExecutorJobOutput, SuspensionExecutor, SuspensionExecutorHandle,
};

const COMMAND_QUEUE_BUDGET: usize = 64;

pub(crate) enum JobEvent {
    Start(BackgroundCommandRequest),
    /// One of this actor's executor jobs reported its result.
    JobFinished {
        id: u64,
        output: Option<ExecutorJobOutput>,
    },
}

pub(crate) struct BackgroundCommands {
    state: SharedState,
    hub: StatusHub,
    runtime: Rc<EventCommandRuntime>,
    executor: ActorRef<SuspensionExecutor>,
    next_id: u64,
    jobs: BTreeMap<u64, JobState>,
}

enum JobState {
    /// An `if-shell -b` condition is running; its branches wait on the answer.
    ResolvingCondition {
        then_command: Option<String>,
        else_command: Option<String>,
        context: ClientContext,
    },
    Running,
}

impl BackgroundCommands {
    pub(crate) fn new(
        state: SharedState,
        hub: StatusHub,
        executor: ActorRef<SuspensionExecutor>,
        executor_handle: SuspensionExecutorHandle,
    ) -> Self {
        Self {
            state,
            hub,
            runtime: Rc::new(EventCommandRuntime::new(executor_handle)),
            executor,
            next_id: 1,
            jobs: BTreeMap::new(),
        }
    }

    pub(crate) fn handle(&mut self, target: &ActorRef<Self>, event: JobEvent, outbox: &mut Outbox) {
        match event {
            JobEvent::Start(request) => self.start(target, request, outbox),
            JobEvent::JobFinished { id, output } => match self.jobs.remove(&id) {
                Some(JobState::ResolvingCondition {
                    then_command,
                    else_command,
                    context,
                }) => {
                    let matched = matches!(
                        output,
                        Some(ExecutorJobOutput::Suspension(
                            command::CommandSuspensionResult::IfShell(true)
                        ))
                    );
                    let command = if matched { then_command } else { else_command };
                    self.start_command(
                        target,
                        BackgroundCommand::Line(command),
                        context,
                        outbox,
                    );
                }
                Some(JobState::Running) => {
                    let Some(ExecutorJobOutput::Queue(Ok(mut result))) = output else {
                        return;
                    };
                    for request in result.background_commands.drain(..) {
                        self.start(target, request, outbox);
                    }
                }
                None => {}
            },
        }
    }

    fn start(
        &mut self,
        target: &ActorRef<Self>,
        request: BackgroundCommandRequest,
        outbox: &mut Outbox,
    ) {
        match request.into_pending() {
            PendingBackground::Ready(command, context) => {
                self.start_command(target, command, context, outbox)
            }
            PendingBackground::Condition {
                condition,
                then_command,
                else_command,
                context,
            } => {
                let id = self.allocate_id();
                let job = command::suspend::SuspensionJob::if_shell(
                    &condition,
                    &self.job_context(&context),
                );
                self.jobs.insert(
                    id,
                    JobState::ResolvingCondition {
                        then_command,
                        else_command,
                        context,
                    },
                );
                self.executor.with_mut(|executor| {
                    executor.adopt_background_suspension(job, target.clone(), id, outbox);
                });
            }
        }
    }

    /// The client context a shell job runs with: the caller's, with the
    /// environment tmux's `environ_for_session` would have built for it.
    fn job_context(&self, context: &ClientContext) -> ClientContext {
        {
            let state = self.state.borrow_mut();
            context.with_job_environment(&state)
        }
    }

    fn start_command(
        &mut self,
        target: &ActorRef<Self>,
        command: BackgroundCommand,
        context: ClientContext,
        outbox: &mut Outbox,
    ) {
        let agents = self.hub.snapshot().panes;
        let queue = match command {
            BackgroundCommand::Line(command) => {
                let Some(command) = command.filter(|command| !command.trim().is_empty()) else {
                    return;
                };
                command::start_resumable_command_string(&command, &self.state, &agents, &context)
            }
            BackgroundCommand::Args(args) => {
                if args.is_empty() {
                    return;
                }
                command::start_resumable_command(&args, &self.state, &agents, &context)
            }
            BackgroundCommand::RunShell { args, jobs } => {
                let id = self.allocate_id();
                let job = command::suspend::SuspensionJob::background_shell(
                    &args,
                    &self.job_context(&context),
                    jobs,
                );
                self.jobs.insert(id, JobState::Running);
                self.executor.with_mut(|executor| {
                    executor.adopt_background_suspension(job, target.clone(), id, outbox);
                });
                return;
            }
        };
        let Ok(queue) = queue else { return };
        let id = self.allocate_id();
        let coroutine = command::CommandCoroutine::new(
            queue,
            Rc::clone(&self.state),
            Rc::clone(&self.runtime) as Rc<dyn command::CommandRuntime>,
            COMMAND_QUEUE_BUDGET,
        );
        self.jobs.insert(id, JobState::Running);
        self.executor.with_mut(|executor| {
            executor.adopt_background_queue(coroutine, target.clone(), id, outbox);
        });
    }

    fn allocate_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        id
    }
}
