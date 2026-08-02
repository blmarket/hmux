//! Event-loop-owned detached command queues.

use std::collections::{BTreeMap, VecDeque};
use std::io;
use std::sync::{Arc, Mutex};

use crate::integration::status::StatusHub;
use crate::server::command::{
    self, BackgroundCommand, BackgroundCommandRequest, ClientContext, CommandResult,
};
use crate::server::state::ServerState;
use crate::server::task::TaskState;

use super::actor::ActorRef;
use super::driver::Outbox;
use super::reactor::WakeHandle;

const COMMAND_QUEUE_BUDGET: usize = 64;

pub(crate) enum JobEvent {
    Start(BackgroundCommandRequest),
    ConditionResolved {
        id: u64,
        command: Option<String>,
        context: ClientContext,
    },
    CommandCompleted {
        id: u64,
        result: io::Result<CommandResult>,
    },
}

enum WorkerCompletion {
    Condition {
        id: u64,
        command: Option<String>,
        context: ClientContext,
    },
    Command {
        id: u64,
        result: io::Result<CommandResult>,
    },
}

#[derive(Clone)]
struct CompletionSender {
    pending: Arc<Mutex<VecDeque<WorkerCompletion>>>,
    wake: WakeHandle,
}

impl CompletionSender {
    fn send(&self, completion: WorkerCompletion) {
        if let Ok(mut pending) = self.pending.lock() {
            pending.push_back(completion);
        }
        let _ = self.wake.wake();
    }
}

pub(crate) struct BackgroundCommands {
    state: Arc<Mutex<ServerState>>,
    hub: StatusHub,
    completions: CompletionSender,
    next_id: u64,
    jobs: BTreeMap<u64, JobState>,
}

enum JobState {
    ResolvingCondition,
    Running,
}

impl BackgroundCommands {
    pub(crate) fn new(state: Arc<Mutex<ServerState>>, hub: StatusHub, wake: WakeHandle) -> Self {
        Self {
            state,
            hub,
            completions: CompletionSender {
                pending: Arc::new(Mutex::new(VecDeque::new())),
                wake,
            },
            next_id: 1,
            jobs: BTreeMap::new(),
        }
    }

    pub(crate) fn take_completions(&self) -> Vec<JobEvent> {
        let Ok(mut pending) = self.completions.pending.lock() else {
            return Vec::new();
        };
        pending
            .drain(..)
            .map(|completion| match completion {
                WorkerCompletion::Condition {
                    id,
                    command,
                    context,
                } => JobEvent::ConditionResolved {
                    id,
                    command,
                    context,
                },
                WorkerCompletion::Command { id, result } => {
                    JobEvent::CommandCompleted { id, result }
                }
            })
            .collect()
    }

    pub(crate) fn handle(&mut self, target: &ActorRef<Self>, event: JobEvent, outbox: &mut Outbox) {
        match event {
            JobEvent::Start(request) if request.is_ready() => {
                let (command, context) = request.resolve();
                self.start_command(None, command, context);
            }
            JobEvent::Start(request) => {
                let id = self.allocate_id();
                self.jobs.insert(id, JobState::ResolvingCondition);
                let completed = self.completions.clone();
                if super::task::EventCommandRuntime::spawn_blocking(
                    move || request.resolve(),
                    move |(command, context)| {
                        let BackgroundCommand::Line(command) = command else {
                            unreachable!("argv background requests are ready immediately")
                        };
                        completed.send(WorkerCompletion::Condition {
                            id,
                            command,
                            context,
                        });
                    },
                )
                .is_err()
                {
                    self.jobs.remove(&id);
                }
            }
            JobEvent::ConditionResolved {
                id,
                command,
                context,
            } => {
                if matches!(self.jobs.get(&id), Some(JobState::ResolvingCondition)) {
                    self.jobs.remove(&id);
                    self.start_command(Some(id), BackgroundCommand::Line(command), context);
                }
            }
            JobEvent::CommandCompleted { id, result } => {
                if !matches!(self.jobs.remove(&id), Some(JobState::Running)) {
                    return;
                }
                if let Ok(mut result) = result {
                    for request in result.background_commands.drain(..) {
                        outbox.enqueue_background(target.clone(), JobEvent::Start(request));
                    }
                }
            }
        }
    }

    fn start_command(
        &mut self,
        id: Option<u64>,
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
                let id = id.unwrap_or_else(|| self.allocate_id());
                self.jobs.insert(id, JobState::Running);
                let completed = self.completions.clone();
                if super::task::EventCommandRuntime::spawn_blocking(
                    move || command::run_background_shell(&args, &context, jobs),
                    move |()| {
                        completed.send(WorkerCompletion::Command {
                            id,
                            result: Ok(CommandResult::ok("")),
                        });
                    },
                )
                .is_err()
                {
                    self.jobs.remove(&id);
                }
                return;
            }
        };
        let Ok(queue) = queue else { return };
        let id = id.unwrap_or_else(|| self.allocate_id());
        self.jobs.insert(id, JobState::Running);
        let task = TaskState::new(command::CommandCoroutine::new(
            queue,
            Arc::clone(&self.state),
            Arc::new(super::task::EventCommandRuntime),
            COMMAND_QUEUE_BUDGET,
        ));
        let completed = self.completions.clone();
        if super::task::EventCommandRuntime::spawn_coroutine(task, move |result| {
            completed.send(WorkerCompletion::Command { id, result });
        })
        .is_err()
        {
            self.jobs.remove(&id);
        }
    }

    fn allocate_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        id
    }
}
