//! Loop-owned detached command queues.
//!
//! A `-b` command has no client waiting on it, so nothing outside the loop
//! needs to observe its progress: each request runs as a task of its own and
//! starts whatever it detaches in turn.

use std::rc::Rc;

use crate::integration::status::StatusHub;
use crate::server::command::{
    self, BackgroundCommand, BackgroundCommandRequest, ClientContext, PendingBackground,
};
use crate::server::state::SharedState;
use crate::server::Server;

use hmux_rt::TaskHandle;

const COMMAND_QUEUE_BUDGET: usize = 64;

/// Starts detached command work on the loop's task set.
#[derive(Clone)]
pub(crate) struct BackgroundRunner {
    state: SharedState,
    hub: StatusHub,
    tasks: TaskHandle,
}

impl BackgroundRunner {
    pub(crate) fn new(server: &Server, tasks: TaskHandle) -> Self {
        Self {
            state: server.state(),
            hub: server.status_hub(),
            tasks,
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
                let matched =
                    command::suspend::if_shell(&self.tasks, condition, job_context).await;
                let command = if matched { then_command } else { else_command };
                self.start_command(BackgroundCommand::Line(command), context);
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
            BackgroundCommand::RunShell { command, jobs } => {
                let job_context = self.job_context(&context);
                let tasks = self.tasks.clone();
                let state = Rc::clone(&self.state);
                self.tasks.spawn(async move {
                    let output =
                        command::suspend::background_shell(&tasks, command, job_context, jobs).await;
                    // The child's output goes to a pane's view mode, which
                    // needs the state handle the job itself does not hold.
                    let mut state = state.borrow_mut();
                    output.deliver_detached(&mut state);
                });
                return;
            }
        };
        let Ok(queue) = queue else { return };
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
