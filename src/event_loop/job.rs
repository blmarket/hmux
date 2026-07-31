//! Event-loop-owned detached command queues.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::common::reactor::WakeHandle;
use crate::integration::status::StatusHub;
use crate::native::command::{
    self, BackgroundCommand, BackgroundCommandRequest, ClientContext, CommandSuspensionResult,
    ResumableCommandQueue, ResumableCommandTurn,
};
use crate::native::state::ServerState;

use super::actor::ActorRef;
use super::driver::Outbox;

const COMMAND_QUEUE_BUDGET: usize = 64;

pub(crate) enum JobEvent {
    Start(BackgroundCommandRequest),
    ConditionResolved {
        id: u64,
        command: Option<String>,
        context: ClientContext,
    },
    SuspensionResolved {
        id: u64,
        result: CommandSuspensionResult,
    },
    Continue(u64),
}

enum WorkerCompletion {
    Condition {
        id: u64,
        command: Option<String>,
        context: ClientContext,
    },
    Suspension {
        id: u64,
        result: CommandSuspensionResult,
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
    waiting_conditions: BTreeSet<u64>,
    active: BTreeMap<u64, ResumableCommandQueue>,
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
            waiting_conditions: BTreeSet::new(),
            active: BTreeMap::new(),
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
                WorkerCompletion::Suspension { id, result } => {
                    JobEvent::SuspensionResolved { id, result }
                }
            })
            .collect()
    }

    pub(crate) fn handle(&mut self, target: &ActorRef<Self>, event: JobEvent, outbox: &mut Outbox) {
        match event {
            JobEvent::Start(request) if request.is_ready() => {
                let (command, context) = request.resolve();
                self.start_command(target, command, context, outbox);
            }
            JobEvent::Start(request) => {
                let id = self.allocate_id();
                self.waiting_conditions.insert(id);
                let completed = self.completions.clone();
                thread::spawn(move || {
                    let (command, context) = request.resolve();
                    let BackgroundCommand::Line(command) = command else {
                        unreachable!("argv background requests are ready immediately")
                    };
                    completed.send(WorkerCompletion::Condition {
                        id,
                        command,
                        context,
                    });
                });
            }
            JobEvent::ConditionResolved {
                id,
                command,
                context,
            } => {
                if self.waiting_conditions.remove(&id) {
                    self.start_command(target, BackgroundCommand::Line(command), context, outbox);
                }
            }
            JobEvent::SuspensionResolved { id, result } => {
                let Some(queue) = self.active.get_mut(&id) else {
                    return;
                };
                queue.resume(result, &self.state);
                outbox.enqueue_background(target.clone(), JobEvent::Continue(id));
            }
            JobEvent::Continue(id) => self.drive_command(target, id, outbox),
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
        };
        let Ok(queue) = queue else { return };
        let id = self.allocate_id();
        self.active.insert(id, queue);
        outbox.enqueue_background(target.clone(), JobEvent::Continue(id));
    }

    fn drive_command(&mut self, target: &ActorRef<Self>, id: u64, outbox: &mut Outbox) {
        let Some(queue) = self.active.get_mut(&id) else {
            return;
        };
        match queue.drive(&self.state, COMMAND_QUEUE_BUDGET) {
            ResumableCommandTurn::Pending => {
                outbox.enqueue_background(target.clone(), JobEvent::Continue(id));
            }
            ResumableCommandTurn::Suspended(suspension) => {
                let completed = self.completions.clone();
                thread::spawn(move || {
                    completed.send(WorkerCompletion::Suspension {
                        id,
                        result: suspension.resolve(),
                    });
                });
            }
            ResumableCommandTurn::Complete(mut result) => {
                self.active.remove(&id);
                for request in result.background_commands.drain(..) {
                    outbox.enqueue_background(target.clone(), JobEvent::Start(request));
                }
            }
        }
    }

    fn allocate_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        id
    }
}
