//! Loop-owned detached command queues and the server-wide queue runner.
//!
//! A `-b` command has no client waiting on it, so nothing outside the loop
//! needs to observe its progress: each request runs as a task of its own and
//! starts whatever it detaches in turn.
//!
//! The server-wide queue is the other thing with no client behind it: tmux's
//! global command queue, where every event notification lands and where its
//! hook bodies run. One persistent drainer task owns its consumption — single
//! consumer by construction, the way tmux's one server loop owns
//! `cmdq_next(NULL)` — running items in the order the events were raised, so
//! a body outlives whatever raised it.

use std::rc::Rc;

use crate::integration::status::StatusHub;
use crate::server::Server;
use crate::server::command::{
    self, BackgroundCommand, BackgroundCommandRequest, ClientContext, LazyCommand,
    PendingBackground,
};
use crate::server::state::SharedState;
use crate::sync::Notify;

use hmux_rt::TaskHandle;

const COMMAND_QUEUE_BUDGET: usize = 64;

/// Starts detached command work on the loop's task set, and runs the
/// server-wide queue.
#[derive(Clone)]
pub(crate) struct BackgroundRunner {
    state: SharedState,
    hub: StatusHub,
    tasks: TaskHandle,
    /// Wakes the drainer task. Signals delivered while it is mid-drain
    /// collapse into one permit, which the drain's own loop-until-empty
    /// absorbs.
    signal: Notify,
}

impl BackgroundRunner {
    pub(crate) fn new(server: &Server, tasks: TaskHandle) -> Self {
        Self {
            state: server.state(),
            hub: server.status_hub(),
            tasks,
            signal: Notify::new(),
        }
    }

    /// Wake the drainer for whatever is waiting on the server-wide queue.
    ///
    /// tmux drains its global queue from the server loop, before and after the
    /// client queues; the callers here are the same places — the loop turn, and
    /// a client queue that has just finished — so a body runs after the command
    /// line that raised it and before the server takes its next client request.
    /// Infallible: only a signal, so a caller inside the state guard is fine.
    pub(crate) fn pump(&self) {
        self.signal.notify();
    }

    /// Own the server-wide queue's consumption: the one task this runs on is
    /// the queue's only consumer, so items can never interleave.
    pub(crate) async fn run_drainer(self) {
        loop {
            self.signal.notified().await;
            self.drain().await;
        }
    }

    /// Take one queued event at a time and run its bodies to completion, so the
    /// next event's bodies are queued behind them rather than beside them.
    async fn drain(&self) {
        loop {
            let requests = {
                let mut state = self.state.borrow_mut();
                command::next_notification_hooks(&mut state)
            };
            let Some(requests) = requests else {
                return;
            };
            for request in requests {
                self.run_to_completion(request).await;
            }
            self.state.borrow_mut().finish_notification();
        }
    }

    /// Run one server-wide queue item where the runner stands, rather than as a
    /// task of its own: the item after it starts only once this one is over.
    async fn run_to_completion(&self, request: BackgroundCommandRequest) {
        let BackgroundCommandRequest::Command { command, context } = request else {
            self.start(request);
            return;
        };
        let Ok(compiled) = command.compile(&self.state) else {
            return;
        };
        if compiled.is_empty() {
            return;
        }
        let agents = self.hub.snapshot().panes;
        let queue = command::start_compiled_command(compiled, &agents, &context);
        let Ok(completion) = command::spawn_detached_queue(
            &self.tasks,
            queue,
            Rc::clone(&self.state),
            COMMAND_QUEUE_BUDGET,
        ) else {
            return;
        };
        let Ok(Ok(mut result)) = completion.await else {
            return;
        };
        for request in result.background_commands.drain(..) {
            self.start(request);
        }
    }

    /// Run one detached request, and anything it detaches in turn.
    ///
    /// The work begins on the request's own task, not here: a start raised
    /// mid-dispatch — a hook taken while a client drives its output — must not
    /// run command code inside its raiser's borrows, the same deferral the
    /// queued start event used to provide.
    pub(crate) fn start(&self, request: BackgroundCommandRequest) {
        let runner = self.clone();
        self.tasks.spawn(async move {
            runner.begin(request).await;
        });
    }

    async fn begin(&self, request: BackgroundCommandRequest) {
        match request.into_pending() {
            PendingBackground::Ready(command, context) => self.start_command(command, context),
            // `if-shell -b`: the condition is a job like any other.
            PendingBackground::Condition {
                condition,
                then_command,
                else_command,
                context,
            } => {
                let job_context = self.job_context(&context);
                let matched = command::suspend::if_shell(&self.tasks, condition, job_context).await;
                let Some(branch) = (if matched { then_command } else { else_command }) else {
                    return;
                };
                self.start_command(
                    BackgroundCommand::Command(LazyCommand::Line(branch)),
                    context,
                );
            }
        }
    }

    /// The client context a shell job runs with: the caller's, with the
    /// environment tmux's `environ_for_session` would have built for it.
    fn job_context(&self, context: &ClientContext) -> ClientContext {
        let state = self.state.borrow_mut();
        context.with_job_environment(&state)
    }

    fn start_command(&self, command: BackgroundCommand, context: ClientContext) {
        let agents = self.hub.snapshot().panes;
        let queue = match command {
            BackgroundCommand::Command(command) => {
                let Ok(compiled) = command.compile(&self.state) else {
                    return;
                };
                if compiled.is_empty() {
                    return;
                }
                command::start_compiled_command(compiled, &agents, &context)
            }
            BackgroundCommand::RunShell { command, jobs } => {
                let job_context = self.job_context(&context);
                let tasks = self.tasks.clone();
                let state = Rc::clone(&self.state);
                self.tasks.spawn(async move {
                    let output =
                        command::suspend::background_shell(&tasks, command, job_context, jobs)
                            .await;
                    // The child's output goes to a pane's view mode, which
                    // needs the state handle the job itself does not hold.
                    let mut state = state.borrow_mut();
                    output.deliver_detached(&mut state);
                });
                return;
            }
        };
        // A detached queue has no client polling it, so the loop owns the
        // whole thing; its first turn runs here, inside this job's own task.
        let Ok(completion) = command::spawn_detached_queue(
            &self.tasks,
            queue,
            Rc::clone(&self.state),
            COMMAND_QUEUE_BUDGET,
        ) else {
            return;
        };
        let runner = self.clone();
        self.tasks.spawn(async move {
            let Ok(Ok(mut result)) = completion.await else {
                return;
            };
            for request in result.background_commands.drain(..) {
                runner.start(request);
            }
        });
    }
}
