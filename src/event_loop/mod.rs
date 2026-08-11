//! Single-threaded server event-loop infrastructure.

pub(crate) mod actor;
pub(crate) mod driver;
/// Prototype `Future` executor on the same reactor; example code only,
/// re-exported at the crate root for `examples/future_rt.rs`.
pub mod future_rt;
pub(crate) mod job;
pub(crate) mod listener;
pub(crate) mod pane;
pub(crate) mod process;
pub(crate) mod protocol;
pub(crate) mod reactor;
pub(crate) mod suspend;
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
    use std::io;
    use std::rc::Rc;
    use std::time::{Duration, Instant};

    use crate::server::command::suspend::IfShellJob;
    use crate::server::command::{
        BackgroundCommand, BackgroundCommandRequest, CommandCoroutine, CommandResult,
        CommandRuntime, PendingBackground, ResumableCommandQueue,
    };
    use crate::server::state::SharedState;
    use crate::server::task::{Coroutine, TaskState};

    use super::driver::{EventLoop, IoRecipient};
    use super::reactor::MioReactor;
    use super::suspend::EventCommandRuntime;

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

    /// Drive an already-started job on a loop of its own until it finishes.
    pub(crate) fn finish_on_loop<T: Coroutine>(task: &mut TaskState<T>) {
        LoopCommandDriver::new()
            .expect("job test loop")
            .drive_task(task);
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
            Rc::new(EventCommandRuntime::new(self.loop_.executor_handle()))
        }

        pub(crate) fn run_queue(
            &mut self,
            queue: ResumableCommandQueue,
            state: &SharedState,
        ) -> CommandResult {
            let mut task = TaskState::new(CommandCoroutine::new(
                queue,
                Rc::clone(state),
                self.runtime(),
                DISPATCH_BUDGET,
            ));
            let deadline = Instant::now() + DEADLINE;
            // The queue's completion descriptor belongs to this task, not to
            // the loop, so turns stay short instead of blocking the reactor.
            while !task.poll_optimistically(Instant::now()) {
                assert!(Instant::now() < deadline, "command queue never completed");
                self.loop_
                    .run_turn(Some(TURN_TIMEOUT), DISPATCH_BUDGET)
                    .expect("event loop turn");
            }
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
                    let matched = self.drive_detached(IfShellJob::new(&condition, &context));
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
                    let job = crate::server::command::suspend::BackgroundShellJob::new(
                        &args, &context, jobs,
                    );
                    self.drive_detached(job);
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
            while !task.poll_optimistically(Instant::now()) {
                assert!(Instant::now() < deadline, "job never completed");
                self.loop_
                    .run_turn(Some(TURN_TIMEOUT), DISPATCH_BUDGET)
                    .expect("event loop turn");
            }
        }
    }
}
