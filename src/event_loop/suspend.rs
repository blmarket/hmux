//! The command runtime the loop gives its clients.
//!
//! Every command that has to wait — for a child process, a pipe, a deadline, or
//! another client's answer — becomes a task on the loop's task set, and reports
//! back through a [`Completion`] that wakes whoever is parked on it. Nothing
//! here describes a descriptor to the loop: the tasks own theirs.

use std::io;

use std::rc::Rc;

use crate::server::command::suspend::{self, SuspensionStart, SuspensionWait};
use crate::server::command::{
    run_command_queue, CommandResult, CommandRuntime, CommandSuspension, CommandSuspensionResult,
    QueueStatus, QueuedCommand, ResumableCommandQueue,
};
use hmux_rt::{completion_pair, Completion, CompletionSender};

use hmux_rt::TaskHandle;

/// Command runtime for clients served by the event loop.
///
/// Every suspension becomes a task on the loop's task set.
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
    /// Run a detached queue, whose result nobody polls for: the caller only
    /// wants to be told when it is over.
    pub(crate) fn spawn_detached_queue(
        &self,
        queue: ResumableCommandQueue,
        state: crate::server::state::SharedState,
        budget: usize,
    ) -> io::Result<Completion<io::Result<CommandResult>>> {
        let (completion, sender) = completion_pair()?;
        let status = Rc::new(QueueStatus::default());
        let runtime: Rc<dyn CommandRuntime> = Rc::new(self.clone());
        self.tasks.spawn_now(async move {
            let result = run_command_queue(queue, state, runtime, budget, status).await;
            sender.complete(result);
        });
        Ok(completion)
    }

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

