//! Single-threaded server event-loop infrastructure.

pub(crate) mod actor;
pub(crate) mod driver;
pub(crate) mod job;
pub(crate) mod listener;
pub(crate) mod pane;
pub(crate) mod process;
pub(crate) mod protocol;
pub(crate) mod reactor;
pub(crate) mod suspend;
pub(crate) mod tasks;
pub(crate) mod term_signal;
pub(crate) mod timer;

/// Drive one command queue to completion on a loop of its own.
///
/// Unit-test scaffolding: the tests that exercise the command language have no
/// server behind them, but the queue still suspends on jobs only a loop can
/// resolve. Running them here keeps one implementation of that resolution — the
/// same executor the daemon uses — instead of a second, blocking one.
#[cfg(test)]
pub(crate) mod test_driver {
    use std::cell::Cell;
    use std::io;
    use std::rc::Rc;
    use std::time::{Duration, Instant};

    use crate::server::command::{
        BackgroundCommand, BackgroundCommandRequest, CommandResult,
        CommandRuntime, PendingBackground, ResumableCommandQueue,
    };
    use crate::server::state::SharedState;
    use crate::server::task::{completion_pair, Coroutine, TaskState, WakeFn};

    use std::future::Future;

    use super::driver::{EventLoop, IoRecipient};
    use super::reactor::MioReactor;
    use super::suspend::EventCommandRuntime;
    use super::tasks::TaskHandle;

    /// How long one command queue may take before the test is declared stuck.
    const DEADLINE: Duration = Duration::from_secs(30);
    const TURN_TIMEOUT: Duration = Duration::from_millis(10);
    const DISPATCH_BUDGET: usize = 64;

    /// Run one job to completion on a loop of its own and take its result.
    pub(crate) fn run_on_loop<T: Coroutine>(job: T) -> T::Output {
        LoopCommandDriver::new()
            .expect("job test loop")
            .drive_detached(job)
    }

    /// Run one async suspension to completion on a loop of its own.
    ///
    /// The daemon spawns these on the loop it already has; a test has none, so
    /// this builds one, spawns the task on it and drives turns until the
    /// task's completion carries its value.
    pub(crate) fn run_task_on_loop<T, F>(spawn: impl FnOnce(TaskHandle) -> F) -> T
    where
        T: 'static,
        F: Future<Output = T> + 'static,
    {
        LoopCommandDriver::new()
            .expect("job test loop")
            .drive_task_future(spawn)
    }

    pub(crate) struct LoopCommandDriver {
        loop_: EventLoop<MioReactor<IoRecipient>>,
    }

    impl LoopCommandDriver {
        pub(crate) fn new() -> io::Result<Self> {
            Ok(Self {
                loop_: EventLoop::new()?,
            })
        }

        fn runtime(&self) -> Rc<dyn CommandRuntime> {
            Rc::new(EventCommandRuntime::new(self.loop_.task_handle()))
        }

        /// Spawn one future on this loop and drive turns until it finishes.
        pub(crate) fn drive_task_future<T, F>(&mut self, spawn: impl FnOnce(TaskHandle) -> F) -> T
        where
            T: 'static,
            F: Future<Output = T> + 'static,
        {
            let tasks = self.loop_.task_handle();
            let (completion, sender) = completion_pair().expect("completion pair");
            let future = spawn(tasks.clone());
            tasks.spawn(async move {
                sender.complete(future.await);
            });
            let mut task = TaskState::new(completion);
            self.drive_task(&mut task);
            task.take_output()
                .expect("completed task")
                .expect("task result")
        }

        pub(crate) fn run_queue(
            &mut self,
            queue: ResumableCommandQueue,
            state: &SharedState,
        ) -> CommandResult {
            let queued = match self
                .runtime()
                .spawn_queue(queue, Rc::clone(state), DISPATCH_BUDGET)
            {
                Ok(queued) => queued,
                Err(error) => return CommandResult::err(format!("{error}\n")),
            };
            let mut task = TaskState::new(queued);
            self.drive_task(&mut task);
            match task.take_output() {
                Some(Ok(result)) => result,
                Some(Err(error)) => CommandResult::err(format!("{error}\n")),
                None => CommandResult::err("command stopped without a result\n"),
            }
        }

        /// Run one detached (`-b`) request, and anything it detaches in turn.
        pub(crate) fn run_background(
            &mut self,
            request: BackgroundCommandRequest,
            state: &SharedState,
            agents: &crate::integration::status::PaneAgents,
        ) {
            let (command, context) = match request.into_pending() {
                PendingBackground::Ready(command, context) => (command, context),
                // `if-shell -b`: the condition is a job like any other.
                PendingBackground::Condition {
                    condition,
                    then_command,
                    else_command,
                    context,
                } => {
                    let job_context = context.clone();
                    let matched = self.drive_task_future(|tasks| async move {
                        crate::server::command::suspend::if_shell(&tasks, condition, job_context)
                            .await
                    });
                    let command = if matched { then_command } else { else_command };
                    (BackgroundCommand::Line(command), context)
                }
            };
            let queue = match command {
                BackgroundCommand::Line(command) => {
                    let Some(command) = command.filter(|line| !line.trim().is_empty()) else {
                        return;
                    };
                    crate::server::command::start_resumable_command_string(
                        &command, state, agents, &context,
                    )
                }
                BackgroundCommand::Args(args) => {
                    if args.is_empty() {
                        return;
                    }
                    crate::server::command::start_resumable_command(&args, state, agents, &context)
                }
                BackgroundCommand::RunShell { args, jobs } => {
                    self.drive_task_future(|tasks| async move {
                        crate::server::command::suspend::background_shell(
                            &tasks, args, context, jobs,
                        )
                        .await
                    });
                    return;
                }
            };
            let Ok(queue) = queue else { return };
            let mut result = self.run_queue(queue, state);
            for request in result.background_commands.drain(..) {
                self.run_background(request, state, agents);
            }
        }

        /// Drive a job the loop would otherwise own outright.
        fn drive_detached<T: Coroutine>(&mut self, job: T) -> T::Output {
            let mut task = TaskState::new(job);
            self.drive_task(&mut task);
            task.take_output().expect("completed job has a result")
        }

        fn drive_task<T: Coroutine>(&mut self, task: &mut TaskState<T>) {
            let deadline = Instant::now() + DEADLINE;
            // This task belongs to the test, not to the loop, so the loop
            // cannot wake it. Its own wake sets this flag instead, which turns
            // the next turn into a poll that does not block: without it, every
            // suspension resolved inside a dispatch would cost a full
            // `TURN_TIMEOUT` before the test noticed.
            let woken = Rc::new(Cell::new(false));
            let flag = Rc::clone(&woken);
            let wake: WakeFn = Rc::new(move || flag.set(true));
            while !task.poll_optimistically(Instant::now()) {
                assert!(Instant::now() < deadline, "job never completed");
                task.set_wake(&wake);
                let timeout = if woken.replace(false) {
                    Duration::ZERO
                } else {
                    TURN_TIMEOUT
                };
                self.loop_
                    .run_turn(Some(timeout), DISPATCH_BUDGET)
                    .expect("event loop turn");
            }
        }
    }
}
