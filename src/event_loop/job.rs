//! Event-loop-owned detached command queues.
//!
//! A `-b` command has no client waiting on it, so nothing outside the loop
//! needs to observe its progress: the queue is handed to the suspension
//! executor as a job of its own and reports back here when it finishes.

use std::collections::BTreeMap;
use std::io;
use std::rc::Rc;

use crate::integration::status::StatusHub;
use crate::server::command::{
    self, BackgroundCommand, BackgroundCommandRequest, ClientContext, PendingBackground,
};
use crate::server::command::CommandResult;
use crate::server::state::SharedState;
use crate::server::task::{completion_pair, Completion};

use super::actor::ActorRef;
use super::driver::{Outbox, WakeQueue};
use super::suspend::EventCommandRuntime;
use super::tasks::TaskHandle;

const COMMAND_QUEUE_BUDGET: usize = 64;

pub(crate) enum JobEvent {
    Start(BackgroundCommandRequest),
    /// One of this actor's detached jobs has a result waiting.
    Ready { id: u64 },
}

pub(crate) struct BackgroundCommands {
    state: SharedState,
    tasks: TaskHandle,
    hub: StatusHub,
    runtime: Rc<EventCommandRuntime>,
    wakes: WakeQueue,
    next_id: u64,
    jobs: BTreeMap<u64, JobState>,
}

/// One piece of detached work and what this actor owes it when it finishes.
///
/// Each holds the completion its task reports through. Nothing polls them: the
/// wake installed alongside is what says a value has arrived.
enum JobState {
    /// An `if-shell -b` condition is running; its branches wait on the answer.
    ResolvingCondition {
        completion: Completion<command::CommandSuspensionResult>,
        then_command: Option<String>,
        else_command: Option<String>,
        context: ClientContext,
    },
    /// A `run-shell -b` child, whose output goes to a pane's view mode.
    RunningShell {
        completion: Completion<command::CommandSuspensionResult>,
    },
    /// A detached command queue, which may detach more work in turn.
    RunningQueue {
        completion: Completion<io::Result<CommandResult>>,
    },
}

impl BackgroundCommands {
    pub(crate) fn new(
        state: SharedState,
        hub: StatusHub,
        wakes: WakeQueue,
        tasks: TaskHandle,
    ) -> Self {
        Self {
            state,
            hub,
            runtime: Rc::new(EventCommandRuntime::new(tasks.clone())),
            tasks,
            wakes,
            next_id: 1,
            jobs: BTreeMap::new(),
        }
    }

    /// Nothing here addresses another actor, so the outbox goes unused: a job
    /// reports through its completion and this actor only reads it.
    pub(crate) fn handle(&mut self, target: &ActorRef<Self>, event: JobEvent, _outbox: &mut Outbox) {
        match event {
            JobEvent::Start(request) => self.start(target, request),
            JobEvent::Ready { id } => self.finish(target, id),
        }
    }

    /// Take one finished job's result, if it has one yet.
    ///
    /// A wake can arrive before the value — the task set fires one whenever a
    /// completion is installed onto — so a job that is not done goes back.
    fn finish(&mut self, target: &ActorRef<Self>, id: u64) {
        let Some(mut job) = self.jobs.remove(&id) else {
            return;
        };
        match &mut job {
            JobState::ResolvingCondition { completion, .. } => {
                let Some(answer) = completion.take() else {
                    self.jobs.insert(id, job);
                    return;
                };
                let JobState::ResolvingCondition {
                    then_command,
                    else_command,
                    context,
                    ..
                } = job
                else {
                    return;
                };
                let matched = matches!(
                    answer,
                    Ok(command::CommandSuspensionResult::IfShell(true))
                );
                let command = if matched { then_command } else { else_command };
                self.start_command(target, BackgroundCommand::Line(command), context);
            }
            JobState::RunningShell { completion } => {
                let Some(answer) = completion.take() else {
                    self.jobs.insert(id, job);
                    return;
                };
                // The child's output goes to a pane's view mode, which needs
                // the state handle the job itself does not hold.
                if let Ok(command::CommandSuspensionResult::RunShell(output)) = answer {
                    let mut state = self.state.borrow_mut();
                    output.deliver_detached(&mut state);
                }
            }
            JobState::RunningQueue { completion } => {
                let Some(answer) = completion.take() else {
                    self.jobs.insert(id, job);
                    return;
                };
                if let Ok(Ok(mut result)) = answer {
                    for request in result.background_commands.drain(..) {
                        self.start(target, request);
                    }
                }
            }
        }
    }

    /// Store one job and install the wake that reports it.
    fn watch(&mut self, target: &ActorRef<Self>, job: JobState) {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        let wake = self.wakes.background_wake(target.downgrade(), id);
        let mut job = job;
        match &mut job {
            JobState::ResolvingCondition { completion, .. }
            | JobState::RunningShell { completion } => completion.set_wake(&wake),
            JobState::RunningQueue { completion } => completion.set_wake(&wake),
        }
        self.jobs.insert(id, job);
    }

    fn start(&mut self, target: &ActorRef<Self>, request: BackgroundCommandRequest) {
        match request.into_pending() {
            PendingBackground::Ready(command, context) => {
                self.start_command(target, command, context)
            }
            PendingBackground::Condition {
                condition,
                then_command,
                else_command,
                context,
            } => {
                let Ok((completion, sender)) = completion_pair() else {
                    return;
                };
                let job_context = self.job_context(&context);
                let tasks = self.tasks.clone();
                self.tasks.spawn(async move {
                    let matched = command::suspend::if_shell(&tasks, condition, job_context).await;
                    sender.complete(command::CommandSuspensionResult::IfShell(matched));
                });
                self.watch(
                    target,
                    JobState::ResolvingCondition {
                        completion,
                        then_command,
                        else_command,
                        context,
                    },
                );
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
                let Ok((completion, sender)) = completion_pair() else {
                    return;
                };
                let job_context = self.job_context(&context);
                let tasks = self.tasks.clone();
                self.tasks.spawn(async move {
                    let output =
                        command::suspend::background_shell(&tasks, args, job_context, jobs).await;
                    sender.complete(command::CommandSuspensionResult::RunShell(output));
                });
                self.watch(target, JobState::RunningShell { completion });
                return;
            }
        };
        let Ok(queue) = queue else { return };
        // A detached queue has no client polling it, so the loop owns the whole
        // thing: the executor holds only the completion its task reports to.
        let Ok(completion) = self
            .runtime
            .spawn_detached_queue(queue, Rc::clone(&self.state), COMMAND_QUEUE_BUDGET)
        else {
            return;
        };
        self.watch(target, JobState::RunningQueue { completion });
    }

}
