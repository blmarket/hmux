//! Blocking worker adapter for shared command coroutines.

use std::io;
use std::os::fd::AsRawFd;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use crate::integration::status::PaneAgents;
use crate::server::command::{
    self, BackgroundCommand, BackgroundCommandRequest, CommandCoroutine, CommandResult,
    CommandRuntime, CommandSuspension, CommandSuspensionResult, ResumableCommandQueue,
};
use crate::server::state::ServerState;
use crate::server::task::{completion_pair, Completion, ReadySet, TaskState};

#[derive(Clone, Default)]
pub(crate) struct NativeCommandRuntime;

impl NativeCommandRuntime {
    pub(crate) fn run(
        queue: ResumableCommandQueue,
        state: &Arc<Mutex<ServerState>>,
    ) -> CommandResult {
        let mut task = TaskState::new(CommandCoroutine::new(
            queue,
            Arc::clone(state),
            Arc::new(Self),
            64,
        ));
        task.poll(&ReadySet::default());
        while task.wait().is_some() {
            let ready = {
                let wait = task.wait().expect("pending command has a wait request");
                assert!(
                    wait.sources().len() <= 1,
                    "native command driver supports one descriptor per coroutine"
                );
                let mut descriptor = wait.sources().first().map(|source| libc::pollfd {
                    fd: source.fd().as_raw_fd(),
                    events: libc::POLLIN,
                    revents: 0,
                });
                let timeout = wait.deadline().map_or(-1, |deadline| {
                    let duration = deadline.saturating_duration_since(Instant::now());
                    let millis = duration.as_nanos().saturating_add(999_999) / 1_000_000;
                    i32::try_from(millis).unwrap_or(i32::MAX)
                });
                let (pointer, count) = descriptor
                    .as_mut()
                    .map_or((std::ptr::null_mut(), 0), |descriptor| {
                        (descriptor as *mut libc::pollfd, 1)
                    });
                let source_ready = loop {
                    let result = unsafe { libc::poll(pointer, count, timeout) };
                    if result >= 0
                        || io::Error::last_os_error().kind() != io::ErrorKind::Interrupted
                    {
                        break result > 0;
                    }
                };
                ReadySet::after_single_source_wakeup(&wait, source_ready, Instant::now())
            };
            task.poll(&ready);
        }
        match task.take_output() {
            Some(Ok(result)) => result,
            Some(Err(error)) => CommandResult::err(format!("{error}\n")),
            None => CommandResult::err("command stopped without a result\n"),
        }
    }

    pub(crate) fn spawn_background(
        request: BackgroundCommandRequest,
        state: Arc<Mutex<ServerState>>,
        agents: PaneAgents,
    ) -> io::Result<()> {
        thread::Builder::new()
            .name("hmux-native-background".to_string())
            .spawn(move || {
                let (command, context) = request.resolve();
                let queue = match command {
                    BackgroundCommand::Line(command) => {
                        let Some(command) = command.filter(|line| !line.trim().is_empty()) else {
                            return;
                        };
                        command::start_resumable_command_string(&command, &state, &agents, &context)
                    }
                    BackgroundCommand::Args(args) => {
                        if args.is_empty() {
                            return;
                        }
                        command::start_resumable_command(&args, &state, &agents, &context)
                    }
                    BackgroundCommand::RunShell { args, jobs } => {
                        command::run_background_shell(&args, &context, jobs);
                        return;
                    }
                };
                let Ok(queue) = queue else { return };
                let mut result = Self::run(queue, &state);
                for request in result.background_commands.drain(..) {
                    let _ = Self::spawn_background(request, Arc::clone(&state), agents.clone());
                }
            })?;
        Ok(())
    }
}

impl CommandRuntime for NativeCommandRuntime {
    fn submit(
        &self,
        suspension: CommandSuspension,
    ) -> io::Result<Completion<CommandSuspensionResult>> {
        let (completion, sender) = completion_pair()?;
        thread::Builder::new()
            .name("hmux-native-command".to_string())
            .spawn(move || sender.complete(suspension.resolve_blocking()))?;
        Ok(completion)
    }
}
