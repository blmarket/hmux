//! The runtime host: the event loop the daemon actually runs.
//!
//! Every turn is the same shape — sync the task set, dispatch queued polls,
//! poll the reactor — driven by the host through [`TaskRuntime::dispatch`] and
//! [`TaskRuntime::poll`]. The daemon owns the turn cadence and its dispatch
//! order; a completion-driven `block_on` sits in front for tests and for the
//! demo in `examples/tasks.rs`. Every event source is a task: there is no
//! side channel to the reactor.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::future::Future;
use std::io;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::completion::{completion_pair, WakeFn};
use crate::reactor::{MioReactor, Reactor as _, Ready};
use crate::task_loop::TaskLoop;
use crate::tasks::{TaskEvent, TaskHandle, WakeSink};

/// How much the runtime dispatches before it polls, and how long it blocks in
/// the reactor when nothing has woken it.
const DISPATCH_BUDGET: usize = 64;
const TURN_TIMEOUT: Duration = Duration::from_millis(10);

/// An event loop that exists to run tasks and nothing else.
pub struct TaskRuntime {
    reactor: MioReactor<u64>,
    tasks: TaskLoop,
    /// The run queue: every wake is "poll this task", in fire order.
    woken: Rc<RefCell<VecDeque<TaskEvent>>>,
    ready: Vec<Ready<u64>>,
}

impl TaskRuntime {
    pub fn new() -> io::Result<Self> {
        let woken = Rc::new(RefCell::new(VecDeque::new()));
        let sink: WakeSink = {
            let woken = Rc::clone(&woken);
            Rc::new(move |task| woken.borrow_mut().push_back(TaskEvent::Poll(task)))
        };
        Ok(Self {
            reactor: MioReactor::new()?,
            tasks: TaskLoop::new(sink),
            woken,
            ready: Vec::new(),
        })
    }

    pub fn handle(&self) -> TaskHandle {
        self.tasks.handle()
    }

    /// Work in hand: wakes and readiness already queued, plus spawns not yet
    /// adopted. While this is nonzero a host should keep dispatching rather
    /// than block in [`TaskRuntime::poll`].
    pub fn pending(&self) -> usize {
        self.woken.borrow().len() + self.tasks.pending_spawned()
    }

    /// Timers currently armed, one per parked sleep rather than one per
    /// sleeping task. Zero once every sleeping task has resumed.
    pub fn armed_timers(&self) -> usize {
        self.tasks.armed_timers()
    }

    /// Make the reactor and the run queue describe the task set: adopt
    /// spawns, make the registrations that were asked for, release the ones
    /// whose `AsyncFd` is gone.
    fn sync(&mut self) -> io::Result<()> {
        let woken = Rc::clone(&self.woken);
        self.tasks.sync(
            &mut self.reactor,
            |io| io,
            |event| {
                woken.borrow_mut().push_back(event);
            },
        )
    }

    /// Run up to `budget` queued task polls.
    pub fn dispatch(&mut self, budget: usize) -> io::Result<usize> {
        let mut dispatched = 0;
        while dispatched < budget {
            // Before every poll, so a registration or spawn made by the last
            // task is honored before the next runs.
            self.sync()?;
            let event = self.woken.borrow_mut().pop_front();
            let Some(event) = event else {
                return Ok(dispatched);
            };
            self.tasks.dispatch(event);
            dispatched += 1;
        }
        self.sync()?;
        Ok(dispatched)
    }

    /// Wait for readiness or the nearest deadline, queueing what arrives.
    pub fn poll(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        // Work in hand caps the wait at zero; so does the nearest deadline.
        let timeout = if self.woken.borrow().is_empty() {
            timeout
        } else {
            Some(Duration::ZERO)
        };
        let deadline_timeout = self.tasks.time_until_next_deadline(Instant::now());
        let timeout = match (timeout, deadline_timeout) {
            (Some(requested), Some(deadline)) => Some(requested.min(deadline)),
            (requested, None) => requested,
            (None, deadline) => deadline,
        };
        let mut ready = std::mem::take(&mut self.ready);
        self.reactor.poll(timeout, &mut ready)?;
        for notification in ready.drain(..) {
            if let Some(task) = self
                .tasks
                .deliver_io(*notification.recipient(), notification.readiness())
            {
                self.woken.borrow_mut().push_back(TaskEvent::Poll(task));
            }
        }
        self.ready = ready;
        let woken = Rc::clone(&self.woken);
        self.tasks.drain_expired(Instant::now(), |event| {
            woken.borrow_mut().push_back(event);
        });
        Ok(())
    }

    /// Spawn `future` and run turns until it produces a value.
    ///
    /// The value comes back through a completion, which is also how a task's
    /// non-task owner collects one. Nothing inside the loop can wake this
    /// waiter, so a turn is bounded rather than blocking outright.
    pub fn block_on<T: 'static>(&mut self, future: impl Future<Output = T> + 'static) -> T {
        let (mut completion, sender) = completion_pair().expect("completion pair");
        self.handle().spawn(async move {
            sender.complete(future.await);
        });
        let woken = Rc::new(Cell::new(false));
        let flag = Rc::clone(&woken);
        let wake: WakeFn = Rc::new(move || flag.set(true));
        loop {
            if let Some(value) = completion.take() {
                return value.expect("the spawned task reported a value");
            }
            completion.set_wake(&wake);
            let timeout = if woken.replace(false) {
                Duration::ZERO
            } else {
                TURN_TIMEOUT
            };
            self.dispatch(DISPATCH_BUDGET)
                .expect("task runtime dispatch");
            self.poll(Some(timeout)).expect("task runtime poll");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read as _;
    use std::os::fd::{AsFd as _, AsRawFd as _, BorrowedFd};
    use std::process::{Command, Stdio};

    use crate::reactor::Interest;
    use crate::tasks::{sleep, AsyncFd, JoinError};

    use super::*;

    use std::future::Future;
    use std::task::Poll;

    /// Race two futures on one task, `Ok` if the first wins. The loser is
    /// dropped before the race resolves, taking whatever it parked with it.
    ///
    /// The daemon composes its own combinators over this crate's leaves; this
    /// test-local one exists to pin the leaf semantics a race exercises.
    async fn race<A: Future, B: Future>(first: A, second: B) -> Result<A::Output, B::Output> {
        let mut first = Box::pin(first);
        let mut second = Box::pin(second);
        std::future::poll_fn(move |context| {
            if let Poll::Ready(output) = first.as_mut().poll(context) {
                return Poll::Ready(Ok(output));
            }
            if let Poll::Ready(output) = second.as_mut().poll(context) {
                return Poll::Ready(Err(output));
            }
            Poll::Pending
        })
        .await
    }

    /// Drive two futures on one task until both finish.
    async fn both(first: impl Future<Output = ()>, second: impl Future<Output = ()>) {
        let mut first = Box::pin(first);
        let mut second = Box::pin(second);
        let mut first_done = false;
        let mut second_done = false;
        std::future::poll_fn(move |context| {
            first_done = first_done || first.as_mut().poll(context).is_ready();
            second_done = second_done || second.as_mut().poll(context).is_ready();
            if first_done && second_done {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await
    }

    fn set_nonblocking(fd: BorrowedFd<'_>) -> io::Result<()> {
        let raw = fd.as_raw_fd();
        let flags = unsafe { libc::fcntl(raw, libc::F_GETFL) };
        if flags < 0 {
            return Err(io::Error::last_os_error());
        }
        if unsafe { libc::fcntl(raw, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    /// A `run-shell` in miniature: wait for the child's output, capture, reap.
    ///
    /// `source` borrowing the pipe across the `.await` is the thing a
    /// hand-written state machine cannot express.
    async fn run_shell(tasks: &TaskHandle, command: &str) -> io::Result<Vec<u8>> {
        let mut child = Command::new("/bin/sh")
            .arg("-c")
            .arg(command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let mut stdout = child.stdout.take().expect("piped stdout");
        set_nonblocking(stdout.as_fd())?;
        let source = AsyncFd::new(tasks, stdout.as_fd(), Interest::READABLE)?;
        let mut output = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            match stdout.read(&mut chunk) {
                Ok(0) => break,
                Ok(count) => output.extend_from_slice(&chunk[..count]),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    source.readiness().await;
                }
                Err(error) => return Err(error),
            }
        }
        drop(source);
        child.wait()?;
        Ok(output)
    }

    #[test]
    fn a_completion_wakes_a_task_without_any_descriptor() {
        let mut runtime = TaskRuntime::new().expect("runtime");
        let handle = runtime.handle();
        let probe = runtime.handle();
        let (completion, sender) = completion_pair::<u32>().expect("completion pair");
        let result = runtime.block_on(async move {
            // Spawned second in program order but the receiver parks first, so
            // the send happens while the waiter is suspended.
            handle.spawn(async move {
                sender.complete(41);
            });
            completion.await
        });
        assert_eq!(result.expect("the sender completed"), 41);
        assert_eq!(
            probe.registered_io(),
            0,
            "task-to-task waiting must not touch the reactor"
        );
    }

    #[test]
    fn consecutive_suspensions_need_no_generation_bookkeeping() {
        // The pipelined-wedge shape: one task runs two suspending commands back
        // to back, each waiting on a brand-new descriptor. Each suspension is
        // its own `AsyncFd` with its own reactor token, so there is no
        // registration to re-point and nothing to version.
        let mut runtime = TaskRuntime::new().expect("runtime");
        let handle = runtime.handle();
        let (one, two) = runtime.block_on(async move {
            let one = run_shell(&handle, "echo one").await.expect("first");
            let two = run_shell(&handle, "echo two").await.expect("second");
            (one, two)
        });
        assert_eq!(one, b"one\n");
        assert_eq!(two, b"two\n");
    }

    #[test]
    fn a_task_can_wait_on_io_while_another_waits_on_it() {
        // The control-client shape: a "client" task waits on a "command" task
        // through a completion (userland), while the command task waits on the
        // kernel (child stdout). Both waits are the same thing to the loop: a
        // task that is not on the event queue.
        let mut runtime = TaskRuntime::new().expect("runtime");
        let handle = runtime.handle();
        let result = runtime.block_on(async move {
            let (completion, sender) = completion_pair().expect("completion pair");
            let shell = handle.clone();
            handle.spawn(async move {
                let output = run_shell(&shell, "echo done").await;
                sender.complete(output.expect("shell output"));
            });
            completion.await.expect("command task completed")
        });
        assert_eq!(result, b"done\n");
    }

    #[test]
    fn sleep_parks_on_the_loops_timer_queue() {
        let mut runtime = TaskRuntime::new().expect("runtime");
        let handle = runtime.handle();
        let started = Instant::now();
        runtime.block_on(async move {
            sleep(&handle, Duration::from_millis(20)).await;
        });
        assert!(started.elapsed() >= Duration::from_millis(20));
    }

    /// Timers belong to sleeps, not to tasks: one task parked on two of them
    /// arms two, and neither displaces the other. Collapsing them to the
    /// earliest would arm one here and let the later deadline go missing.
    #[test]
    fn one_task_parked_on_two_sleeps_arms_a_timer_for_each() {
        let mut runtime = TaskRuntime::new().expect("runtime");
        let handle = runtime.handle();
        let sleeper = handle.clone();
        handle.spawn(async move {
            both(
                sleep(&sleeper, Duration::from_millis(20)),
                sleep(&sleeper, Duration::from_secs(30)),
            )
            .await;
        });
        // Adopt the spawn and give it the first poll that parks both halves.
        runtime.dispatch(8).expect("dispatch");
        assert_eq!(runtime.armed_timers(), 2);

        // Dropping the task releases both, one entry apiece.
        drop(runtime);
    }

    /// The near deadline firing must not disturb the far one still parked
    /// beside it, nor wake the task a second time for a timer that has gone.
    #[test]
    fn a_fired_sleep_leaves_its_siblings_timer_alone() {
        let mut runtime = TaskRuntime::new().expect("runtime");
        let handle = runtime.handle();
        let started = Instant::now();
        runtime.block_on(async move {
            let near = sleep(&handle, Duration::from_millis(20));
            let far = sleep(&handle, Duration::from_secs(30));
            let Ok(()) = race(near, far).await else {
                panic!("the far deadline beat the near one");
            };
        });
        assert!(started.elapsed() >= Duration::from_millis(20));
        assert!(started.elapsed() < Duration::from_secs(5));
        // The loser went with the `select`, taking its timer with it.
        assert_eq!(runtime.armed_timers(), 0);
    }

    #[test]
    fn a_lost_select_sleep_disarms_its_deadline() {
        let mut runtime = TaskRuntime::new().expect("runtime");
        let handle = runtime.handle();
        let started = Instant::now();
        runtime.block_on(async move {
            // The shell finishes far before the fallback deadline; losing the
            // race must drop the deadline with the sleep, or the tail of this
            // task would stall a blocking host poll for the full two seconds.
            let raced = race(
                run_shell(&handle, "echo raced"),
                sleep(&handle, Duration::from_secs(2)),
            )
            .await;
            let Ok(output) = raced else {
                panic!("the shell lost to a two-second sleep");
            };
            assert_eq!(output.expect("shell output"), b"raced\n");
            // A fresh sleep proves the stale deadline is gone: it must wake at
            // its own time, not at the raced sleep's.
            sleep(&handle, Duration::from_millis(20)).await;
        });
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn an_unpollable_descriptor_reports_always_ready() {
        let mut runtime = TaskRuntime::new().expect("runtime");
        let handle = runtime.handle();
        let readiness = runtime.block_on(async move {
            let file = std::fs::File::open("/dev/null").expect("open /dev/null");
            let source = AsyncFd::new(&handle, file.as_fd(), Interest::READABLE).expect("register");
            source.readiness().await
        });
        assert!(readiness.is_readable());
        assert!(readiness.is_writable());
    }

    #[test]
    fn join_handle_reports_a_tasks_result() {
        let mut runtime = TaskRuntime::new().expect("runtime");
        let tasks = runtime.handle();
        let probe = runtime.handle();
        let value = runtime.block_on(async move {
            let task = tasks.spawn_join(async { 42 });
            // Two live tasks: the block_on wrapper and the joined child.
            assert_eq!(tasks.active_tasks(), 2);
            let value = task.await;
            assert_eq!(tasks.active_tasks(), 1);
            value
        });

        assert_eq!(value, Ok(42));
        assert_eq!(probe.active_tasks(), 0);
    }

    #[test]
    fn a_task_can_be_cancelled_before_its_first_poll() {
        let mut runtime = TaskRuntime::new().expect("runtime");
        let tasks = runtime.handle();
        let result = runtime.block_on(async move {
            let task = tasks.spawn_join(async { 42 });
            assert!(task.cancel());
            assert!(!task.cancel(), "cancellation is idempotent");
            task.await
        });

        assert_eq!(result, Err(JoinError));
    }

    #[test]
    fn cancelling_a_sleeping_task_resolves_its_join_handle() {
        let mut runtime = TaskRuntime::new().expect("runtime");
        let tasks = runtime.handle();
        let probe = runtime.handle();
        let (started, child_started) = completion_pair::<()>().expect("completion pair");
        runtime.block_on(async move {
            let sleeper = tasks.clone();
            let task = tasks.spawn_join(async move {
                child_started.complete(());
                sleep(&sleeper, Duration::from_secs(60)).await;
                42
            });
            started.await.expect("the child started");
            assert!(task.cancel());
            assert_eq!(task.await, Err(JoinError));
        });

        assert_eq!(probe.active_tasks(), 0);
        assert_eq!(runtime.armed_timers(), 0);
    }

    #[test]
    fn cancellation_releases_a_tasks_descriptor() {
        let mut runtime = TaskRuntime::new().expect("runtime");
        let tasks = runtime.handle();
        let probe = runtime.handle();
        let (reader, _writer) = std::os::unix::net::UnixStream::pair().expect("socket pair");
        reader.set_nonblocking(true).expect("nonblocking reader");
        runtime.block_on(async move {
            let io_tasks = tasks.clone();
            let (started, child_started) = completion_pair::<()>().expect("completion pair");
            let task = tasks.spawn_join(async move {
                let source =
                    AsyncFd::new(&io_tasks, reader.as_fd(), Interest::READABLE).expect("async fd");
                child_started.complete(());
                source.readiness().await;
            });
            started.await.expect("the child started");
            assert_eq!(probe.registered_io(), 1);
            assert!(task.cancel());
            assert_eq!(task.await, Err(JoinError));
            assert_eq!(probe.registered_io(), 0);
        });
    }

    #[test]
    fn dropping_the_runtime_finishes_join_handles() {
        let task = {
            let runtime = TaskRuntime::new().expect("runtime");
            runtime.handle().spawn_join(std::future::pending::<u32>())
        };

        assert!(task.is_finished());
        let mut runtime = TaskRuntime::new().expect("second runtime");
        assert_eq!(runtime.block_on(task), Err(JoinError));
    }
}
