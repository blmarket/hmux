//! Task-based server event sources on the hmux-rt runtime.

pub(crate) mod job;
pub(crate) mod listener;
pub(crate) mod pane;
pub(crate) mod process;
pub(crate) mod protocol;
pub(crate) mod suspend;
pub(crate) mod term_signal;

/// Drive one command queue to completion on a runtime of its own.
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
    use hmux_rt::{completion_pair, Completion, WakeFn};

    use std::future::Future;

    use super::suspend::EventCommandRuntime;
    use hmux_rt::TaskHandle;
    use hmux_rt::TaskRuntime;

    /// How long one command queue may take before the test is declared stuck.
    const DEADLINE: Duration = Duration::from_secs(30);
    const TURN_TIMEOUT: Duration = Duration::from_millis(10);
    const DISPATCH_BUDGET: usize = 64;

    /// Run one async suspension to completion on a runtime of its own.
    ///
    /// The daemon spawns these on the runtime it already has; a test has none,
    /// so this builds one, spawns the task on it and drives turns until the
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
        runtime: TaskRuntime,
    }

    impl LoopCommandDriver {
        pub(crate) fn new() -> io::Result<Self> {
            Ok(Self {
                runtime: TaskRuntime::new()?,
            })
        }

        fn command_runtime(&self) -> Rc<dyn CommandRuntime> {
            Rc::new(EventCommandRuntime::new(self.runtime.handle()))
        }

        /// One dispatch-then-poll turn, polling only when nothing is queued.
        fn run_turn(&mut self, timeout: Duration) -> io::Result<()> {
            self.runtime.dispatch(DISPATCH_BUDGET)?;
            if self.runtime.pending() == 0 {
                self.runtime.poll(Some(timeout))?;
            }
            Ok(())
        }

        /// Spawn one future on this runtime and drive turns until it finishes.
        pub(crate) fn drive_task_future<T, F>(&mut self, spawn: impl FnOnce(TaskHandle) -> F) -> T
        where
            T: 'static,
            F: Future<Output = T> + 'static,
        {
            let tasks = self.runtime.handle();
            let (completion, sender) = completion_pair().expect("completion pair");
            let future = spawn(tasks.clone());
            tasks.spawn(async move {
                sender.complete(future.await);
            });
            let mut completion = completion;
            self.drive_completion(&mut completion).expect("task result")
        }

        pub(crate) fn run_queue(
            &mut self,
            queue: ResumableCommandQueue,
            state: &SharedState,
        ) -> CommandResult {
            let mut queued = match self
                .command_runtime()
                .spawn_queue(queue, Rc::clone(state), DISPATCH_BUDGET)
            {
                Ok(queued) => queued,
                Err(error) => return CommandResult::err(format!("{error}\n")),
            };
            let deadline = Instant::now() + DEADLINE;
            let woken = Rc::new(Cell::new(false));
            let flag = Rc::clone(&woken);
            let wake: WakeFn = Rc::new(move || flag.set(true));
            while !queued.poll() {
                assert!(Instant::now() < deadline, "command queue never completed");
                queued.set_wake(&wake);
                let timeout = if woken.replace(false) {
                    Duration::ZERO
                } else {
                    TURN_TIMEOUT
                };
                self.run_turn(timeout).expect("event loop turn");
            }
            match queued.take_output() {
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

        /// Drive runtime turns until `completion` has its value.
        fn drive_completion<T>(&mut self, completion: &mut Completion<T>) -> io::Result<T> {
            let deadline = Instant::now() + DEADLINE;
            // This task belongs to the test, not to the runtime, so the
            // runtime cannot wake it. Its own wake sets this flag instead,
            // which turns the next turn into a poll that does not block:
            // without it, every suspension resolved inside a dispatch would
            // cost a full `TURN_TIMEOUT` before the test noticed.
            let woken = Rc::new(Cell::new(false));
            let flag = Rc::clone(&woken);
            let wake: WakeFn = Rc::new(move || flag.set(true));
            loop {
                if let Some(value) = completion.take() {
                    return value;
                }
                assert!(Instant::now() < deadline, "job never completed");
                completion.set_wake(&wake);
                let timeout = if woken.replace(false) {
                    Duration::ZERO
                } else {
                    TURN_TIMEOUT
                };
                self.run_turn(timeout).expect("event loop turn");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::os::fd::AsRawFd as _;
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::PathBuf;
    use std::rc::Rc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, Instant};

    use crate::server::command::{
        ClientContext, ClientFileWrite, CommandSuspension, CommandSuspensionResult,
    };
    use hmux_rt::TaskRuntime;

    const POLL_TIMEOUT: Duration = Duration::from_secs(1);
    const DISPATCH_BUDGET: usize = 64;
    static NEXT_LISTENER_PATH: AtomicU64 = AtomicU64::new(0);

    struct ListenerPath(PathBuf);

    impl ListenerPath {
        fn new() -> Self {
            let sequence = NEXT_LISTENER_PATH.fetch_add(1, Ordering::Relaxed);
            Self(PathBuf::from(format!(
                "/tmp/hmux-event-loop-test-{}-{sequence}.sock",
                std::process::id()
            )))
        }
    }

    impl Drop for ListenerPath {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    /// One dispatch-then-poll turn, polling only when nothing is queued.
    fn run_turn(runtime: &mut TaskRuntime, timeout: Duration) {
        runtime.dispatch(DISPATCH_BUDGET).unwrap();
        if runtime.pending() == 0 {
            runtime.poll(Some(timeout)).unwrap();
        }
    }

    fn dispatch_all(runtime: &mut TaskRuntime) {
        let dispatched = runtime.dispatch(4096).unwrap();
        assert!(dispatched < 4096, "event queue did not quiesce");
    }

    #[test]
    fn existing_imsg_descriptor_is_readiness_driven() {
        use crate::tmux::codec::{ImsgReader, ImsgWriter};
        use crate::tmux::message::{Frame, Message};
        use hmux_rt::{Interest, MioReactor, Reactor as _};
        use std::os::fd::AsFd as _;

        let mut reactor = MioReactor::new().expect("reactor");
        let (sender, receiver) = UnixStream::pair().expect("socket pair");
        sender.set_nonblocking(true).expect("nonblocking sender");
        receiver
            .set_nonblocking(true)
            .expect("nonblocking receiver");
        let mut reader = ImsgReader::new(receiver.into());
        let mut writer = ImsgWriter::new(sender.into());
        let token = reactor
            .register(reader.as_fd(), Interest::READABLE, 42u32)
            .expect("register imsg reader");
        writer
            .send(Frame::new(Message::Command(vec!["list-sessions".into()])))
            .expect("send frame");

        let mut ready = Vec::new();
        reactor
            .poll(Some(POLL_TIMEOUT), &mut ready)
            .expect("poll");

        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].token(), token);
        let frame = reader.try_recv().expect("receive ready frame");
        assert_eq!(frame.msg, Message::Command(vec!["list-sessions".into()]));
    }

    #[test]
    fn listener_readiness_accepts_a_waiting_client() {
        let path = ListenerPath::new();
        let listener = UnixListener::bind(&path.0).unwrap();
        let mut runtime = TaskRuntime::new().unwrap();
        let listener = super::listener::spawn(&runtime.handle(), listener, 64).unwrap();
        dispatch_all(&mut runtime);

        let client = UnixStream::connect(&path.0).unwrap();
        assert_eq!(listener.accepted_len(), 0);

        let deadline = Instant::now() + Duration::from_secs(5);
        while listener.accepted_len() == 0 {
            assert!(Instant::now() < deadline, "client was never accepted");
            run_turn(&mut runtime, POLL_TIMEOUT);
        }

        let accepted = listener.pop_accepted().expect("accepted client");
        assert!(listener.pop_accepted().is_none());
        drop((accepted, client));

        listener.shutdown();
        dispatch_all(&mut runtime);
        assert!(!listener.is_alive());
    }

    #[test]
    fn listener_budget_queues_one_accept_continuation() {
        let path = ListenerPath::new();
        let listener = UnixListener::bind(&path.0).unwrap();
        let mut runtime = TaskRuntime::new().unwrap();
        let listener = super::listener::spawn(&runtime.handle(), listener, 2).unwrap();
        dispatch_all(&mut runtime);

        let clients = (0..3)
            .map(|_| UnixStream::connect(&path.0).unwrap())
            .collect::<Vec<_>>();
        runtime.poll(Some(POLL_TIMEOUT)).unwrap();

        assert_eq!(runtime.dispatch(1).unwrap(), 1);
        assert_eq!(listener.accepted_len(), 2);
        assert_eq!(runtime.pending(), 1);

        assert_eq!(runtime.dispatch(1).unwrap(), 1);
        assert_eq!(listener.accepted_len(), 3);
        assert_eq!(runtime.pending(), 0);

        while listener.pop_accepted().is_some() {}
        drop(clients);
        listener.shutdown();
        dispatch_all(&mut runtime);
        assert!(!listener.is_alive());
    }

    /// Run runtime turns until a submitted suspension reports its result.
    fn resolve_on_loop(
        runtime: &mut TaskRuntime,
        suspension: CommandSuspension,
    ) -> CommandSuspensionResult {
        let command_runtime = super::suspend::EventCommandRuntime::new(runtime.handle());
        let completion =
            crate::server::command::CommandRuntime::submit(&command_runtime, suspension)
                .expect("completion pair");
        let mut completion = completion;
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(result) = completion.take() {
                return result.expect("suspension result");
            }
            assert!(Instant::now() < deadline, "suspension never completed");
            // The completion belongs to the caller, not to a task, so nothing
            // here wakes it: keep turns short instead of blocking the reactor.
            run_turn(runtime, Duration::from_millis(10));
        }
    }

    fn run_shell(args: &[&str]) -> CommandSuspension {
        CommandSuspension::RunShell {
            args: std::iter::once("run-shell")
                .chain(args.iter().copied())
                .map(str::to_string)
                .collect(),
            context: ClientContext::default(),
        }
    }

    #[test]
    fn executor_resolves_a_shell_suspension_without_a_thread() {
        let mut runtime = TaskRuntime::new().unwrap();

        let result = resolve_on_loop(&mut runtime, run_shell(&["echo hello; exit 3"]));

        let CommandSuspensionResult::RunShell(completion) = result else {
            panic!("run-shell suspension resolved as another variant");
        };
        assert_eq!(completion.result().exit, 3);
        assert_eq!(
            completion.result().stdout,
            "hello\n'echo hello; exit 3' returned 3\n"
        );
    }

    #[test]
    fn executor_arms_a_loop_timer_for_a_delayed_shell_suspension() {
        let mut runtime = TaskRuntime::new().unwrap();
        let started = Instant::now();

        let result = resolve_on_loop(&mut runtime, run_shell(&["-d", "0.2", "echo late"]));

        let CommandSuspensionResult::RunShell(completion) = result else {
            panic!("run-shell suspension resolved as another variant");
        };
        assert!(started.elapsed() >= Duration::from_millis(150));
        assert_eq!(completion.result().stdout, "late\n");
        assert_eq!(
            runtime.armed_timers(),
            0,
            "the job's timer outlived the job"
        );
    }

    #[test]
    fn runtime_timer_wakes_a_spawned_sleeper() {
        let mut runtime = TaskRuntime::new().unwrap();
        let fired = Rc::new(Cell::new(false));
        let fired_by_timer = Rc::clone(&fired);
        let started = Instant::now();
        let tasks = runtime.handle();
        let sleeper = tasks.clone();
        tasks.spawn(async move {
            hmux_rt::sleep_until(&sleeper, started + Duration::from_millis(20)).await;
            fired_by_timer.set(true);
        });

        let test_deadline = started + Duration::from_secs(1);
        while !fired.get() {
            assert!(Instant::now() < test_deadline, "timer task never completed");
            run_turn(&mut runtime, Duration::from_millis(10));
        }

        assert!(started.elapsed() >= Duration::from_millis(20));
    }

    #[test]
    fn executor_reads_a_fifo_source_file_on_the_loop() {
        let mut runtime = TaskRuntime::new().unwrap();
        let path = ListenerPath::new();
        let raw = std::ffi::CString::new(path.0.as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(raw.as_ptr(), 0o600) }, 0);
        let writer = std::thread::spawn({
            let path = path.0.clone();
            move || {
                std::thread::sleep(Duration::from_millis(50));
                std::fs::write(&path, b"set-buffer -b sourced yes\n").unwrap();
            }
        });

        let result = resolve_on_loop(
            &mut runtime,
            CommandSuspension::SourceFile {
                paths: vec![path.0.display().to_string()],
            },
        );

        writer.join().unwrap();
        let CommandSuspensionResult::SourceFile(reads) = result else {
            panic!("source-file suspension resolved as another variant");
        };
        assert_eq!(
            reads[0].contents().expect("FIFO contents"),
            "set-buffer -b sourced yes\n"
        );
    }

    #[test]
    fn executor_writes_a_fifo_save_buffer_on_the_loop() {
        let mut runtime = TaskRuntime::new().unwrap();
        let path = ListenerPath::new();
        let raw = std::ffi::CString::new(path.0.as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(raw.as_ptr(), 0o600) }, 0);
        // More than a pipe buffer, so the write only completes as the reader
        // drains it — the loop has to watch for writability, not just once.
        let payload = vec![b'x'; 512 * 1024];
        let reader = std::thread::spawn({
            let path = path.0.clone();
            move || {
                std::thread::sleep(Duration::from_millis(50));
                std::fs::read(&path).unwrap()
            }
        });

        let result = resolve_on_loop(
            &mut runtime,
            CommandSuspension::SaveBuffer {
                request: ClientFileWrite {
                    path: path.0.clone(),
                    display_path: path.0.display().to_string(),
                    flags: libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
                    data: payload.clone(),
                },
            },
        );

        let CommandSuspensionResult::SaveBuffer(result) = result else {
            panic!("save-buffer suspension resolved as another variant");
        };
        assert_eq!(result.exit, 0, "{}", result.stderr);
        assert_eq!(reader.join().unwrap(), payload);
    }

    #[test]
    fn executor_writes_a_fifo_save_buffer_a_reader_drains_late() {
        use std::os::unix::fs::OpenOptionsExt;

        let mut runtime = TaskRuntime::new().unwrap();
        let path = ListenerPath::new();
        let raw = std::ffi::CString::new(path.0.as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(raw.as_ptr(), 0o600) }, 0);
        // Many pipe buffers' worth, with the reader attached from the start but
        // idle: the job's very first write fills the pipe, so everything after
        // it depends on the loop rearming writable interest.
        let payload = vec![b'x'; 8 * 1024 * 1024];
        let reader = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(&path.0)
            .unwrap();
        let drained = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            let flags = unsafe { libc::fcntl(reader.as_raw_fd(), libc::F_GETFL) };
            assert_eq!(
                unsafe {
                    libc::fcntl(reader.as_raw_fd(), libc::F_SETFL, flags & !libc::O_NONBLOCK)
                },
                0
            );
            let mut contents = Vec::new();
            let mut reader = reader;
            std::io::Read::read_to_end(&mut reader, &mut contents).unwrap();
            contents
        });

        let result = resolve_on_loop(
            &mut runtime,
            CommandSuspension::SaveBuffer {
                request: ClientFileWrite {
                    path: path.0.clone(),
                    display_path: path.0.display().to_string(),
                    flags: libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
                    data: payload.clone(),
                },
            },
        );

        let CommandSuspensionResult::SaveBuffer(result) = result else {
            panic!("save-buffer suspension resolved as another variant");
        };
        assert_eq!(result.exit, 0, "{}", result.stderr);
        assert_eq!(drained.join().unwrap(), payload);
    }

    #[test]
    fn a_finished_suspension_leaves_no_registration_behind() {
        let mut runtime = TaskRuntime::new().unwrap();

        resolve_on_loop(&mut runtime, run_shell(&["true"]));
        // The descriptors go with the task's leaves, which the next sync
        // releases.
        runtime.dispatch(1).unwrap();

        assert_eq!(runtime.handle().registered_io(), 0);
    }

    #[test]
    fn dispatch_budget_leaves_queued_work_pending() {
        let mut runtime = TaskRuntime::new().unwrap();
        let tasks = runtime.handle();
        tasks.spawn(async {});
        tasks.spawn(async {});

        assert_eq!(runtime.dispatch(1).unwrap(), 1);
        assert_eq!(runtime.pending(), 1);

        dispatch_all(&mut runtime);
        assert_eq!(runtime.pending(), 0);
    }
}
