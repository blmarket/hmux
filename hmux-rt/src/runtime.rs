//! The runtime host: the event loop the daemon actually runs.
//!
//! Every turn is the same shape — sync, dispatch queued polls, poll the
//! reactor — driven by the daemon through [`TaskRuntime::dispatch`] and
//! [`TaskRuntime::poll`]. The daemon owns the turn cadence and its dispatch
//! order; a handoff-driven `block_on` sits in front for tests and for the
//! demo in `examples/tasks.rs`. Every event source is a task: there is no
//! side channel to the reactor.
//!
//! The runtime owns what a running task must not touch — the reactor and the
//! task table — while the state a task reaches from inside its own poll lives
//! in `TaskShared` beside the leaves that use it.

use std::cell::Cell;
use std::collections::{BTreeSet, HashMap};
use std::future::Future;
use std::io;
use std::os::fd::AsFd as _;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Poll;
use std::time::{Duration, Instant};

use crate::handoff::handoff;
use crate::reactor::{MioReactor, Reactor as _, Readiness, Ready};
use crate::tasks::{TaskHandle, TaskId, TaskShared};
use crate::timer::{ExpiredTimer, TimerQueue};

/// How much the runtime dispatches before it polls, and how long it blocks in
/// the reactor when nothing has woken it.
const DISPATCH_BUDGET: usize = 64;
const TURN_TIMEOUT: Duration = Duration::from_millis(10);

/// An event loop that exists to run tasks and nothing else.
pub struct TaskRuntime {
    reactor: MioReactor<u64>,
    /// The slot is `None` while its future is being polled, which lets a
    /// running task spawn and register without re-borrowing.
    tasks: HashMap<TaskId, Option<Pin<Box<dyn Future<Output = ()>>>>>,
    /// Everything a running task may reach, including the run queue every wake
    /// lands on.
    shared: Rc<TaskShared>,
    /// Scratch for [`TaskRuntime::drain_expired`], reused so a turn full of
    /// deadlines does not allocate. The deadlines themselves live in `shared`,
    /// where the sleeps that own them can reach them.
    expired: Vec<ExpiredTimer<TaskId>>,
    ready: Vec<Ready<u64>>,
}

impl TaskRuntime {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            reactor: MioReactor::new()?,
            tasks: HashMap::new(),
            shared: Rc::new(TaskShared::default()),
            expired: Vec::new(),
            ready: Vec::new(),
        })
    }

    pub fn handle(&self) -> TaskHandle {
        TaskHandle::new(Rc::clone(&self.shared))
    }

    /// Work in hand: wakes and readiness already queued, plus spawns not yet
    /// adopted. While this is nonzero the daemon should keep dispatching rather
    /// than block in [`TaskRuntime::poll`].
    pub fn pending(&self) -> usize {
        self.shared.woken.borrow().len() + self.shared.spawned.borrow().len()
    }

    /// Number of timers - for testing purpose
    pub fn armed_timers(&self) -> usize {
        self.shared.timers.borrow().len()
    }

    /// Make the reactor and the run queue describe the task set: adopt
    /// spawns, make the registrations that were asked for, release the ones
    /// whose `AsyncFd` is gone.
    ///
    /// Unlike effect-driven interest updates there is nothing to diff, and
    /// nothing to search for either: a descriptor is owned by the leaf that
    /// created it, and that leaf names its id on the way in and on the way
    /// out. Each half here drains the ids it was handed, so a sync costs what
    /// was asked for rather than a walk of every live descriptor.
    ///
    /// Release runs first because one id can be on both lists — a descriptor
    /// created and dropped inside a single window — and releasing it leaves
    /// nothing for the registration pass to find.
    ///
    /// Deadlines need no sync at all: a sleep arms and cancels its own, inside
    /// the poll that parks or drops it.
    fn sync(&mut self) -> io::Result<()> {
        self.release_dropped_io()?;
        self.register_new_io()?;
        self.adopt_spawned();
        Ok(())
    }

    /// Release every registration whose `AsyncFd` is gone.
    fn release_dropped_io(&mut self) -> io::Result<()> {
        let dropped = std::mem::take(&mut *self.shared.io_dropped.borrow_mut());
        let mut entries = self.shared.io.borrow_mut();
        let mut failure = None;
        for io in dropped {
            let Some(entry) = entries.remove(&io) else {
                continue;
            };
            let Some(token) = entry.token else {
                continue;
            };
            // Every id is released, so one bad deregister costs its own
            // registration rather than the tail of the list behind it.
            if let Err(error) = self.reactor.deregister(token) {
                failure.get_or_insert(error);
            }
        }
        failure.map_or(Ok(()), Err)
    }

    /// Register every descriptor asked for since the last sync. The reactor
    /// addresses readiness by the entry's own id, which is what comes back to
    /// [`TaskRuntime::deliver_io`].
    fn register_new_io(&mut self) -> io::Result<()> {
        let created = std::mem::take(&mut *self.shared.io_new.borrow_mut());
        let mut entries = self.shared.io.borrow_mut();
        let mut failure = None;
        for io in created {
            // Gone already: the pass above released a descriptor that did not
            // outlive the window it was created in.
            let Some(entry) = entries.get_mut(&io) else {
                continue;
            };
            let Some(fd) = entry.fd.take() else {
                continue;
            };
            match self.reactor.register(fd.as_fd(), entry.interest, io) {
                Ok(token) => entry.token = Some(token),
                // A descriptor with no poll operation — a regular file, or a
                // client that redirected its output to `/dev/null` — is
                // rejected by `epoll_ctl` with EPERM. Such a descriptor is
                // never *not* ready, so serve it directly instead of taking
                // the loop down with it.
                Err(error) if error.raw_os_error() == Some(libc::EPERM) => {
                    entry.slot.borrow_mut().always_ready = true;
                    self.shared.woken.borrow_mut().push_back(entry.task);
                }
                // As above: finish the list rather than stranding the requests
                // behind this one, which the drain has already taken.
                Err(error) => {
                    failure.get_or_insert(error);
                }
            }
            // The reactor took its own duplicate.
            drop(fd);
        }
        failure.map_or(Ok(()), Err)
    }

    /// Adopt every task spawned since the last sync, queueing the first poll
    /// of each that still owes one.
    fn adopt_spawned(&mut self) {
        let spawned = std::mem::take(&mut *self.shared.spawned.borrow_mut());
        for (id, future, needs_poll) in spawned {
            if self.shared.cancelled.borrow().contains(&id) {
                drop(future);
                self.finish(id);
                continue;
            }
            self.tasks.insert(id, Some(future));
            // A task's first turn is an event like any other, so a spawn never
            // runs ahead of work already queued.
            if needs_poll {
                self.shared.woken.borrow_mut().push_back(id);
            }
        }
    }

    /// Run up to `budget` queued task polls.
    pub fn dispatch(&mut self, budget: usize) -> io::Result<usize> {
        let mut dispatched = 0;
        while dispatched < budget {
            // Before every poll, so a registration or spawn made by the last
            // task is honored before the next runs.
            self.sync()?;
            let task = self.shared.woken.borrow_mut().pop_front();
            let Some(task) = task else {
                return Ok(dispatched);
            };
            self.poll_task(task);
            dispatched += 1;
        }
        self.sync()?;
        Ok(dispatched)
    }

    /// Poll one task, dropping it if it finishes.
    fn poll_task(&mut self, id: TaskId) {
        if self.shared.cancelled.borrow().contains(&id) {
            self.finish(id);
            return;
        }
        let Some(Some(mut future)) = self.tasks.get_mut(&id).map(Option::take) else {
            // Finished task, or a second wake for one already being polled.
            // Ids are never reused, so a stale wake can only miss.
            return;
        };
        match self.shared.poll_future(id, future.as_mut()) {
            Poll::Ready(()) => {
                // The future's leaves went with it; their entries are marked
                // dropped and the loop releases them on the next sync.
                self.finish(id);
            }
            Poll::Pending if !self.shared.cancelled.borrow().contains(&id) => {
                self.tasks.insert(id, Some(future));
            }
            Poll::Pending => self.finish(id),
        }
    }

    // Removing the future drops the sleeps it was holding, and each cancels
    // its own deadline on the way out.
    fn finish(&mut self, task: TaskId) {
        self.tasks.remove(&task);
        self.shared.finish(task);
    }

    /// Wait for readiness or the nearest deadline, queueing what arrives.
    pub fn poll(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        // Work in hand caps the wait at zero; so does the nearest deadline.
        // A spawn that has not been adopted yet is work too: its task has never
        // run, so it is on no run queue and holds no registration, and blocking
        // here would strand it until something unrelated wakes the loop.
        let timeout = if self.pending() == 0 {
            timeout
        } else {
            Some(Duration::ZERO)
        };
        // Nothing has to be synced first: a sleep is in the queue from the
        // moment it parks, so this reads what is armed right now.
        let deadline_timeout = self
            .shared
            .timers
            .borrow_mut()
            .time_until_next(Instant::now());
        let timeout = match (timeout, deadline_timeout) {
            (Some(requested), Some(deadline)) => Some(requested.min(deadline)),
            (requested, None) => requested,
            (None, deadline) => deadline,
        };
        let mut ready = std::mem::take(&mut self.ready);
        self.reactor.poll(timeout, &mut ready)?;
        for notification in ready.drain(..) {
            self.deliver_io(*notification.recipient(), notification.readiness());
        }
        self.ready = ready;
        self.drain_expired(Instant::now());
        Ok(())
    }

    /// Hand readiness to the descriptor it was registered for and queue the
    /// task waiting on it. Nothing to deliver once the `AsyncFd` is gone.
    fn deliver_io(&mut self, io: u64, readiness: Readiness) {
        let entries = self.shared.io.borrow();
        let Some(entry) = entries.get(&io) else {
            return;
        };
        if entry.dropped {
            return;
        }
        entry.slot.borrow_mut().pending = Some(readiness);
        self.shared.woken.borrow_mut().push_back(entry.task);
    }

    /// Queue a poll for every task whose deadline has elapsed.
    fn drain_expired(&mut self, now: Instant) {
        self.shared
            .timers
            .borrow_mut()
            .drain_expired(now, &mut self.expired);
        let mut woken = BTreeSet::new();
        for timer in self.expired.drain(..) {
            let task = timer.into_value();
            // Several of one task's sleeps coming due together is still one
            // turn's work: the poll re-checks every leaf the task holds, and a
            // second event for it would only find nothing left to do.
            if woken.insert(task) {
                self.shared.woken.borrow_mut().push_back(task);
            }
        }
    }

    /// Spawn `future` and run turns until it produces a value.
    ///
    /// The value comes back through a handoff, the same way a [`JoinHandle`]
    /// collects one. Nothing inside the loop can wake this waiter, so a turn is
    /// bounded rather than blocking outright.
    ///
    /// [`JoinHandle`]: crate::JoinHandle
    pub fn block_on<T: 'static>(&mut self, future: impl Future<Output = T> + 'static) -> T {
        let (mut result, sender) = handoff();
        self.handle().spawn(async move {
            sender.complete(future.await);
        });
        let woken = Rc::new(Cell::new(false));
        let flag = Rc::clone(&woken);
        let wake: Rc<dyn Fn()> = Rc::new(move || flag.set(true));
        loop {
            if let Some(value) = result.take() {
                return value.expect("the spawned task reported a value");
            }
            result.set_wake(&wake);
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

impl Drop for TaskRuntime {
    fn drop(&mut self) {
        // Drop adopted futures first, then tasks that never received their
        // first poll. Their handoff senders close join handles and their
        // leaves mark registrations for release in the usual way.
        self.tasks.clear();
        let spawned = std::mem::take(&mut *self.shared.spawned.borrow_mut());
        drop(spawned);
        self.shared.io.borrow_mut().clear();
        self.shared.io_new.borrow_mut().clear();
        self.shared.io_dropped.borrow_mut().clear();
        *self.shared.timers.borrow_mut() = TimerQueue::default();
        self.shared.cancelled.borrow_mut().clear();
        self.shared.live.borrow_mut().clear();
        self.shared.woken.borrow_mut().clear();
        self.shared.polling.set(None);
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read as _;
    use std::os::fd::{AsRawFd as _, BorrowedFd};
    use std::process::{Command, Stdio};

    use crate::reactor::Interest;
    use crate::tasks::{AsyncFd, JoinError, sleep};

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
    fn a_handoff_wakes_a_task_without_any_descriptor() {
        let mut runtime = TaskRuntime::new().expect("runtime");
        let handle = runtime.handle();
        let probe = runtime.handle();
        let (value, sender) = handoff::<u32>();
        let result = runtime.block_on(async move {
            // Spawned second in program order but the receiver parks first, so
            // the send happens while the waiter is suspended.
            handle.spawn(async move {
                sender.complete(41);
            });
            value.await
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
        // through a handoff (userland), while the command task waits on the
        // kernel (child stdout). Both waits are the same thing to the loop: a
        // task that is not on the event queue.
        let mut runtime = TaskRuntime::new().expect("runtime");
        let handle = runtime.handle();
        let result = runtime.block_on(async move {
            let (output, sender) = handoff();
            let shell = handle.clone();
            handle.spawn(async move {
                let output = run_shell(&shell, "echo done").await;
                sender.complete(output.expect("shell output"));
            });
            output.await.expect("command task completed")
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

    /// A descriptor created and dropped inside one poll names its id on both
    /// of the loop's lists before the loop looks at either. Releasing first is
    /// what leaves the registration pass nothing to find; the other order
    /// would register a descriptor no task is waiting on.
    #[test]
    fn a_descriptor_dropped_before_its_first_sync_is_never_registered() {
        let mut runtime = TaskRuntime::new().expect("runtime");
        let handle = runtime.handle();
        let probe = runtime.handle();
        let (reader, _writer) = std::os::unix::net::UnixStream::pair().expect("socket pair");
        let output = runtime.block_on(async move {
            drop(AsyncFd::new(&handle, reader.as_fd(), Interest::READABLE).expect("async fd"));
            // The loop is whole afterwards: a descriptor created next still
            // registers and delivers, so nothing was stranded on either list.
            run_shell(&handle, "echo after")
                .await
                .expect("shell output")
        });
        assert_eq!(output, b"after\n");
        assert_eq!(probe.registered_io(), 0);
    }

    /// A spawn the loop has not adopted yet is work in hand. The task has
    /// never been polled, so it holds no registration and no deadline, and
    /// nothing in the reactor can wake the loop on its behalf: blocking for the
    /// requested timeout would strand it for that whole interval. The daemon
    /// spawns a client's actor exactly this way, after its own dispatch.
    #[test]
    fn a_spawn_awaiting_adoption_caps_the_wait_at_zero() {
        let mut runtime = TaskRuntime::new().expect("runtime");
        runtime.handle().spawn(async {});
        assert_eq!(runtime.pending(), 1);

        let start = Instant::now();
        runtime.poll(Some(Duration::from_secs(5))).expect("poll");
        assert!(
            start.elapsed() < Duration::from_secs(1),
            "poll blocked on a pending spawn for {:?}",
            start.elapsed()
        );
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
        let (started, child_started) = handoff::<()>();
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
            let (started, child_started) = handoff::<()>();
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
