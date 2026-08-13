//! A standalone driver for the executor: `block_on` without a host behind it.
//!
//! This is not a second executor. Every turn is the same shape a host loop
//! drives — sync the task set, dispatch queued polls, poll the reactor — with
//! a completion-driven `block_on` in front, for tests and for the demo in
//! `examples/tasks.rs`. What it leaves out is everything a real host adds
//! around the task set: other event sources, actors, and their dispatch order.

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

    /// Deadlines currently armed. Zero once every sleeping task has resumed.
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
    use crate::Notify;

    use super::*;

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

    #[test]
    fn a_lost_select_sleep_disarms_its_deadline() {
        use crate::tasks::{select, Either};

        let mut runtime = TaskRuntime::new().expect("runtime");
        let handle = runtime.handle();
        let started = Instant::now();
        runtime.block_on(async move {
            // The shell finishes far before the fallback deadline; losing the
            // race must drop the deadline with the sleep, or the tail of this
            // task would stall a blocking host poll for the full two seconds.
            let raced = select(
                run_shell(&handle, "echo raced"),
                sleep(&handle, Duration::from_secs(2)),
            )
            .await;
            let Either::First(output) = raced else {
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
            let id = task.id();
            assert!(tasks.is_active(id));
            let value = task.await;
            assert!(!tasks.is_active(id));
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
        let started = Notify::new();
        let child_started = started.clone();
        runtime.block_on(async move {
            let sleeper = tasks.clone();
            let task = tasks.spawn_join(async move {
                child_started.notify();
                sleep(&sleeper, Duration::from_secs(60)).await;
                42
            });
            started.notified().await;
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
            let started = Notify::new();
            let child_started = started.clone();
            let task = tasks.spawn_join(async move {
                let source =
                    AsyncFd::new(&io_tasks, reader.as_fd(), Interest::READABLE).expect("async fd");
                child_started.notify();
                source.readiness().await;
            });
            started.notified().await;
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
