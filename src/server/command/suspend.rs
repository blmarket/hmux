//! Runtime-neutral coroutines for the command suspensions the loop can drive.
//!
//! `run-shell` and `if-shell` both run `sh -c` and need the child's output and
//! exit status before the command queue can continue. Expressing them as
//! [`Coroutine`]s instead of a blocking `Command::output()` gives the suspension
//! an explicit readiness description — an optional pre-spawn delay, then the
//! child's stdout and stderr pipes — so the server loop can drive the job
//! between its other work, and a blocking test driver can run the very same
//! job on the calling thread.
//!
//! `source-file`, `load-buffer` and `save-buffer` name a path. Regular files
//! are read and written inline (tmux 3.7b reads its configuration on its own
//! loop, so nothing is gained by deferring them), but a FIFO makes the transfer
//! wait for a peer that may be another client of this very server — so those
//! become [`FifoRead`] and [`FifoWrite`] jobs.

use std::fs::File;
use std::io::{self, Read, Write};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
use std::os::unix::fs::{FileTypeExt, OpenOptionsExt};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::platform::{CurrentPlatform, Platform};
use crate::server::state::{
    BackgroundJobRegistry, ClientPromptRegistry, CommandPromptRequestResult, PromptCompletion,
    WaitRegistry,
};
use crate::server::task::{
    Completion, Coroutine, FdInterest, ReadySet, TaskPoll, WaitRequest, WaitToken,
};

use super::execution::{self, WaitForOutcome};
use super::{
    flag_value, has_flag, interaction_completion_result, io_error_message, job_delay, positionals,
    shell_command, ClientContext, ClientFileWrite, CommandResult, CommandSuspension,
    CommandSuspensionResult, PaneOutputSuspension, RunShellCompletion, SourceFileRead,
};

/// One suspension expressed as the coroutine that resolves it.
///
/// Both drivers build this from the same [`CommandSuspension`]: the server
/// loop's executor registers its descriptors and deadlines with the reactor,
/// and the blocking test driver runs it on the calling thread. There is only
/// ever one implementation of what a suspension does.
pub(crate) enum SuspensionJob {
    BackgroundShell(BackgroundShellJob),
    RunShell(RunShellJob),
    IfShell(IfShellJob),
    SourceFile(SourceFileJob),
    LoadBuffer(LoadBufferJob),
    SaveBuffer(FileWriteJob),
    WaitFor(WaitForJob),
    ClientPrompt(ClientPromptJob),
    PaneOutput(PaneOutputSuspension),
}

impl SuspensionJob {
    /// An `if-shell -b` condition, whose branch the caller picks.
    pub(crate) fn if_shell(condition: &str, context: &ClientContext) -> Self {
        Self::IfShell(IfShellJob::new(condition, context))
    }

    /// A `run-shell -b` job, which reports nothing but has to be reaped.
    pub(crate) fn background_shell(
        args: &[String],
        context: &ClientContext,
        jobs: Arc<BackgroundJobRegistry>,
    ) -> Self {
        Self::BackgroundShell(BackgroundShellJob::new(args, context, jobs))
    }

    pub(crate) fn new(suspension: CommandSuspension) -> Self {
        match suspension {
            CommandSuspension::RunShell { args, context } => {
                Self::RunShell(RunShellJob::new(&args, &context))
            }
            CommandSuspension::IfShell { condition, context } => {
                Self::IfShell(IfShellJob::new(&condition, &context))
            }
            CommandSuspension::SourceFile { paths } => Self::SourceFile(SourceFileJob::new(paths)),
            CommandSuspension::LoadBuffer { path } => Self::LoadBuffer(LoadBufferJob::new(path)),
            CommandSuspension::SaveBuffer { request } => {
                Self::SaveBuffer(FileWriteJob::new(request))
            }
            CommandSuspension::WaitFor { args, registry } => {
                Self::WaitFor(WaitForJob::new(&args, &registry))
            }
            CommandSuspension::CommandPrompt {
                args,
                registry,
                target,
                tty_name,
                wait,
            } => Self::ClientPrompt(ClientPromptJob::prompt(
                args, &registry, target, tty_name, wait,
            )),
            CommandSuspension::ClientInteraction { completed } => {
                Self::ClientPrompt(ClientPromptJob::interaction(completed))
            }
            CommandSuspension::PaneOutput(wait) => Self::PaneOutput(wait),
        }
    }
}

impl Coroutine for SuspensionJob {
    type Output = CommandSuspensionResult;

    fn wait(&self) -> WaitRequest<'_> {
        match self {
            Self::BackgroundShell(job) => job.wait(),
            Self::RunShell(job) => job.wait(),
            Self::IfShell(job) => job.wait(),
            Self::SourceFile(job) => job.wait(),
            Self::LoadBuffer(job) => job.wait(),
            Self::SaveBuffer(job) => job.wait(),
            Self::WaitFor(job) => job.wait(),
            Self::ClientPrompt(job) => job.wait(),
            Self::PaneOutput(job) => job.wait(),
        }
    }

    fn resume(&mut self, ready: &ReadySet) -> TaskPoll<Self::Output> {
        match self {
            Self::BackgroundShell(job) => {
                job.resume(ready).map(CommandSuspensionResult::Completed)
            }
            Self::RunShell(job) => job
                .resume(ready)
                .map(CommandSuspensionResult::RunShell),
            Self::IfShell(job) => job.resume(ready).map(CommandSuspensionResult::IfShell),
            Self::SourceFile(job) => job.resume(ready).map(CommandSuspensionResult::SourceFile),
            Self::LoadBuffer(job) => job.resume(ready).map(CommandSuspensionResult::LoadBuffer),
            Self::SaveBuffer(job) => job.resume(ready).map(CommandSuspensionResult::SaveBuffer),
            Self::WaitFor(job) => job.resume(ready).map(CommandSuspensionResult::Completed),
            Self::ClientPrompt(job) => job.resume(ready).map(CommandSuspensionResult::Completed),
            Self::PaneOutput(job) => job.resume(ready),
        }
    }
}

/// `run-shell -b command`: a detached job, whose output goes nowhere but whose
/// child has to appear in `list-jobs` for as long as it runs.
///
/// The shape is [`RunShellJob`]'s, minus the reporting: wait out `-d`, run the
/// command, and hold the registry entry until the child is reaped.
pub(crate) struct BackgroundShellJob {
    process: Option<ShellProcess>,
    /// `(registry, command)` until the child exists to register.
    pending: Option<(Arc<BackgroundJobRegistry>, String)>,
    registered: Option<(Arc<BackgroundJobRegistry>, u64)>,
}

impl BackgroundShellJob {
    pub(crate) fn new(
        args: &[String],
        context: &ClientContext,
        jobs: Arc<BackgroundJobRegistry>,
    ) -> Self {
        let done = Self {
            process: None,
            pending: None,
            registered: None,
        };
        let Some(command) = positionals(args, &["-t", "-c", "-d"])
            .into_iter()
            .next()
            .map(str::to_string)
        else {
            return done;
        };
        let Ok(delay) = job_delay(args) else {
            return done;
        };
        let mut shell = shell_command(&command, context);
        if let Some(cwd) = flag_value(args, "-c") {
            shell.current_dir(cwd);
        }
        // Identified client streams are open in the daemon as descriptors above
        // stderr. A background job retaining one would make the command client
        // wait for EOF until the job exits, defeating `-b`.
        unsafe {
            shell.pre_exec(|| {
                CurrentPlatform::close_fds_from(3);
                Ok(())
            });
        }
        Self {
            process: Some(ShellProcess::new(shell, delay)),
            pending: Some((jobs, command)),
            registered: None,
        }
    }
}

impl Coroutine for BackgroundShellJob {
    type Output = CommandResult;

    fn wait(&self) -> WaitRequest<'_> {
        match &self.process {
            Some(process) => process.wait(),
            None => WaitRequest::new(Vec::new(), Some(Instant::now())),
        }
    }

    fn resume(&mut self, ready: &ReadySet) -> TaskPoll<Self::Output> {
        let Some(process) = self.process.as_mut() else {
            return TaskPoll::Ready(CommandResult::ok(""));
        };
        let poll = process.resume(ready);
        // The child exists from the first resume that gets past `-d`, and
        // `list-jobs` has to see it for as long as it runs.
        if self.registered.is_none() {
            if let Some((jobs, command)) = self
                .pending
                .take_if(|_| process.child_stdout().is_some())
            {
                let fd = process.child_stdout().map_or(-1, AsRawFd::as_raw_fd);
                let pid = process.child_pid().unwrap_or(0);
                let id = jobs.register(command, fd, pid);
                self.registered = Some((jobs, id));
            }
        }
        match poll {
            TaskPoll::Ready(_) => {
                self.process = None;
                self.pending = None;
                if let Some((jobs, id)) = self.registered.take() {
                    jobs.remove(id);
                }
                TaskPoll::Ready(CommandResult::ok(""))
            }
            TaskPoll::Pending => TaskPoll::Pending,
        }
    }
}

/// A finished `sh -c` child.
struct ShellOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit: i32,
}

/// `run-shell command`, without `-b` (a background job) or `-C` (no child at
/// all): wait out `-d`, run the command and report its output.
pub(crate) struct RunShellJob {
    /// Set when the arguments resolved before any child was needed.
    resolved: Option<RunShellCompletion>,
    process: Option<ShellProcess>,
    command: String,
    capture_stderr: bool,
    view_target: Option<String>,
}

impl RunShellJob {
    pub(crate) fn new(args: &[String], context: &ClientContext) -> Self {
        debug_assert!(!has_flag(args, "-b"));
        let resolve = |result: CommandResult| Self {
            resolved: Some(RunShellCompletion { result, view: None }),
            process: None,
            command: String::new(),
            capture_stderr: false,
            view_target: None,
        };
        let Some(command) = positionals(args, &["-t", "-c", "-d"])
            .into_iter()
            .next()
            .map(str::to_string)
        else {
            return resolve(CommandResult::ok(""));
        };
        let delay = match job_delay(args) {
            Ok(delay) => delay,
            Err(error) => return resolve(error),
        };
        let mut shell = shell_command(&command, context);
        if let Some(cwd) = flag_value(args, "-c") {
            shell.current_dir(cwd);
        }
        Self {
            resolved: None,
            process: Some(ShellProcess::new(shell, delay)),
            command,
            capture_stderr: has_flag(args, "-E"),
            view_target: flag_value(args, "-t").map(str::to_string),
        }
    }

    /// Render one finished child the way tmux reports `run-shell`.
    fn finish(&self, output: io::Result<ShellOutput>) -> RunShellCompletion {
        let output = match output {
            Ok(output) => output,
            Err(error) => {
                return RunShellCompletion {
                    result: CommandResult::err(format!("{error}\n")),
                    view: None,
                };
            }
        };
        let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
        if self.capture_stderr {
            text.push_str(&String::from_utf8_lossy(&output.stderr));
        }
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        if output.exit != 0 {
            if output.exit >= 128 {
                text.push_str(&format!(
                    "'{}' terminated by signal {}\n",
                    self.command,
                    output.exit - 128
                ));
            } else {
                text.push_str(&format!("'{}' returned {}\n", self.command, output.exit));
            }
        }
        let view = self
            .view_target
            .as_ref()
            .map(|target| (target.clone(), text.as_bytes().to_vec()));
        let mut result = CommandResult::ok(if view.is_some() { String::new() } else { text });
        result.exit = output.exit;
        result.continue_queue = true;
        RunShellCompletion { result, view }
    }
}

impl Coroutine for RunShellJob {
    type Output = RunShellCompletion;

    fn wait(&self) -> WaitRequest<'_> {
        match &self.process {
            Some(process) => process.wait(),
            None => WaitRequest::new(Vec::new(), Some(Instant::now())),
        }
    }

    fn resume(&mut self, ready: &ReadySet) -> TaskPoll<Self::Output> {
        if let Some(resolved) = self.resolved.take() {
            return TaskPoll::Ready(resolved);
        }
        let Some(process) = self.process.as_mut() else {
            return TaskPoll::Pending;
        };
        match process.resume(ready) {
            TaskPoll::Ready(output) => {
                self.process = None;
                TaskPoll::Ready(self.finish(output))
            }
            TaskPoll::Pending => TaskPoll::Pending,
        }
    }
}

/// `if-shell cond`, without `-b` or `-F`: run the condition and report whether
/// it succeeded. Unlike `run-shell` the output is discarded, but the pipes are
/// still what makes the child's progress observable.
pub(crate) struct IfShellJob {
    resolved: Option<bool>,
    process: Option<ShellProcess>,
}

impl IfShellJob {
    pub(crate) fn new(condition: &str, context: &ClientContext) -> Self {
        if condition
            .split_whitespace()
            .collect::<Vec<_>>()
            .ends_with(&["run", "\"true\""])
        {
            return Self {
                resolved: Some(true),
                process: None,
            };
        }
        Self {
            resolved: None,
            process: Some(ShellProcess::new(
                shell_command(condition, context),
                Duration::ZERO,
            )),
        }
    }
}

impl Coroutine for IfShellJob {
    type Output = bool;

    fn wait(&self) -> WaitRequest<'_> {
        match &self.process {
            Some(process) => process.wait(),
            None => WaitRequest::new(Vec::new(), Some(Instant::now())),
        }
    }

    fn resume(&mut self, ready: &ReadySet) -> TaskPoll<Self::Output> {
        if let Some(resolved) = self.resolved.take() {
            return TaskPoll::Ready(resolved);
        }
        let Some(process) = self.process.as_mut() else {
            return TaskPoll::Pending;
        };
        match process.resume(ready) {
            TaskPoll::Ready(output) => {
                self.process = None;
                TaskPoll::Ready(output.is_ok_and(|output| output.exit == 0))
            }
            TaskPoll::Pending => TaskPoll::Pending,
        }
    }
}

/// How long to wait before asking again for the exit status of a child that
/// closed its pipes without exiting. The first attempt is always immediate, so
/// only a child that outlives its own output ever waits this long.
const REAP_RETRY_MIN: Duration = Duration::from_millis(1);
const REAP_RETRY_MAX: Duration = Duration::from_millis(50);

enum Stage {
    /// Waiting out `run-shell -d` before the child is spawned.
    Delay { deadline: Instant, spawn: Command },
    /// Draining the child's pipes until both report end of file.
    Running(RunningShell),
    /// Both pipes reported end of file; collecting the exit status.
    Reaping {
        shell: RunningShell,
        retry: Instant,
        backoff: Duration,
    },
    /// The child was reaped; the job has produced its output.
    Done,
}

/// The `sh -c` child shared by every shell-backed suspension.
struct ShellProcess {
    stage: Stage,
}

impl ShellProcess {
    fn new(spawn: Command, delay: Duration) -> Self {
        Self {
            stage: Stage::Delay {
                deadline: Instant::now() + delay,
                spawn,
            },
        }
    }

    /// The running child's stdout, while it is still open.
    fn child_stdout(&self) -> Option<&ChildStdout> {
        match &self.stage {
            Stage::Running(running) => running.stdout.as_ref(),
            _ => None,
        }
    }

    fn child_pid(&self) -> Option<u32> {
        match &self.stage {
            Stage::Running(running) => Some(running.child.id()),
            _ => None,
        }
    }
}

impl Coroutine for ShellProcess {
    type Output = io::Result<ShellOutput>;

    fn wait(&self) -> WaitRequest<'_> {
        match &self.stage {
            Stage::Delay { deadline, .. } => WaitRequest::new(Vec::new(), Some(*deadline)),
            Stage::Running(running) => WaitRequest::new(running.sources(), None),
            Stage::Reaping { retry, .. } => WaitRequest::new(Vec::new(), Some(*retry)),
            Stage::Done => WaitRequest::new(Vec::new(), Some(Instant::now())),
        }
    }

    fn resume(&mut self, ready: &ReadySet) -> TaskPoll<Self::Output> {
        if let Stage::Delay { deadline, .. } = &self.stage {
            if Instant::now() < *deadline && !ready.timed_out() {
                return TaskPoll::Pending;
            }
            let Stage::Delay { spawn, .. } = std::mem::replace(&mut self.stage, Stage::Done) else {
                unreachable!("the delay stage was just observed");
            };
            match RunningShell::spawn(spawn) {
                Ok(running) => self.stage = Stage::Running(running),
                Err(error) => return TaskPoll::Ready(Err(error)),
            }
        }
        if let Stage::Running(running) = &mut self.stage {
            running.drain(ready);
            if !running.is_drained() {
                return TaskPoll::Pending;
            }
            let Stage::Running(shell) = std::mem::replace(&mut self.stage, Stage::Done) else {
                unreachable!("the running stage was just observed");
            };
            self.stage = Stage::Reaping {
                shell,
                retry: Instant::now(),
                backoff: REAP_RETRY_MIN,
            };
        }
        let Stage::Reaping {
            shell,
            retry,
            backoff,
        } = &mut self.stage
        else {
            return TaskPoll::Pending;
        };
        let Some(output) = shell.try_reap() else {
            // A child may close both pipes and keep running. Nothing portable
            // makes that observable, so ask again on a backing-off deadline
            // rather than blocking the driver in `wait(2)`.
            *retry = Instant::now() + *backoff;
            *backoff = (*backoff * 2).min(REAP_RETRY_MAX);
            return TaskPoll::Pending;
        };
        self.stage = Stage::Done;
        TaskPoll::Ready(Ok(output))
    }
}

struct RunningShell {
    child: Child,
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
    captured_stdout: Vec<u8>,
    captured_stderr: Vec<u8>,
}

impl RunningShell {
    const STDOUT: WaitToken = WaitToken::new(0);
    const STDERR: WaitToken = WaitToken::new(1);

    fn spawn(mut spawn: Command) -> io::Result<Self> {
        let mut child = spawn
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        // Both pipes are read on whichever thread drives the job, so neither
        // read may block once the other side stalls.
        if let Some(stdout) = stdout.as_ref() {
            set_nonblocking(stdout.as_fd())?;
        }
        if let Some(stderr) = stderr.as_ref() {
            set_nonblocking(stderr.as_fd())?;
        }
        Ok(Self {
            child,
            stdout,
            stderr,
            captured_stdout: Vec::new(),
            captured_stderr: Vec::new(),
        })
    }

    fn sources(&self) -> Vec<FdInterest<'_>> {
        let mut sources = Vec::new();
        if let Some(stdout) = self.stdout.as_ref() {
            sources.push(FdInterest::readable(Self::STDOUT, stdout.as_fd()));
        }
        if let Some(stderr) = self.stderr.as_ref() {
            sources.push(FdInterest::readable(Self::STDERR, stderr.as_fd()));
        }
        sources
    }

    fn drain(&mut self, ready: &ReadySet) {
        if ready.contains(Self::STDOUT) {
            drain_pipe(&mut self.stdout, &mut self.captured_stdout);
        }
        if ready.contains(Self::STDERR) {
            drain_pipe(&mut self.stderr, &mut self.captured_stderr);
        }
    }

    fn is_drained(&self) -> bool {
        self.stdout.is_none() && self.stderr.is_none()
    }

    /// Collect the exit status if the child has already exited. Both pipes are
    /// closed by the time this is called, so the usual answer is immediate —
    /// but unlike `Command::output()` a driver that shares its thread with
    /// other work must not block here.
    fn try_reap(&mut self) -> Option<ShellOutput> {
        let exit = match self.child.try_wait() {
            Ok(Some(status)) => status.code().unwrap_or_else(|| {
                std::os::unix::process::ExitStatusExt::signal(&status)
                    .map_or(0, |signal| 128 + signal)
            }),
            Ok(None) => return None,
            Err(_) => 0,
        };
        Some(ShellOutput {
            stdout: std::mem::take(&mut self.captured_stdout),
            stderr: std::mem::take(&mut self.captured_stderr),
            exit,
        })
    }
}

/// Read a ready pipe until it would block; clear it once it reports end of
/// file or fails, which is what retires the descriptor from the wait set.
fn drain_pipe<T: Read>(pipe: &mut Option<T>, captured: &mut Vec<u8>) {
    let Some(reader) = pipe.as_mut() else { return };
    let mut chunk = [0u8; 8192];
    loop {
        match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(read) => captured.extend_from_slice(&chunk[..read]),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return,
            Err(_) => break,
        }
    }
    *pipe = None;
}

/// `source-file paths`: read every path in order, reporting what each one
/// held so the caller can queue the commands it parsed.
pub(crate) struct SourceFileJob {
    remaining: std::vec::IntoIter<String>,
    reads: Vec<SourceFileRead>,
    current: Option<(String, FifoRead)>,
}

impl SourceFileJob {
    pub(crate) fn new(paths: Vec<String>) -> Self {
        Self {
            remaining: paths.into_iter(),
            reads: Vec::new(),
            current: None,
        }
    }

    fn record(&mut self, path: String, contents: io::Result<Vec<u8>>) {
        let existed = Path::new(&path).exists();
        self.reads.push(SourceFileRead {
            path,
            contents: contents.and_then(|bytes| {
                String::from_utf8(bytes).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "stream did not contain valid UTF-8",
                    )
                })
            }),
            existed,
        });
    }
}

impl Coroutine for SourceFileJob {
    type Output = Vec<SourceFileRead>;

    fn wait(&self) -> WaitRequest<'_> {
        match &self.current {
            Some((_, read)) => read.wait(),
            None => WaitRequest::new(Vec::new(), Some(Instant::now())),
        }
    }

    fn resume(&mut self, ready: &ReadySet) -> TaskPoll<Self::Output> {
        loop {
            if let Some((_, read)) = self.current.as_mut() {
                let TaskPoll::Ready(contents) = read.resume(ready) else {
                    return TaskPoll::Pending;
                };
                let (path, _) = self.current.take().expect("the read was just observed");
                self.record(path, contents);
            }
            let Some(path) = self.remaining.next() else {
                return TaskPoll::Ready(std::mem::take(&mut self.reads));
            };
            match open_path(Path::new(&path)) {
                PathOpen::Inline(contents) => self.record(path, contents),
                PathOpen::Fifo(read) => self.current = Some((path, read)),
            }
        }
    }
}

/// `load-buffer path`, without `-` (the client's stdin): read the file into a
/// paste buffer, or report the `errno` the stock client would have seen.
pub(crate) struct LoadBufferJob {
    open: Option<PathBuf>,
    read: Option<FifoRead>,
}

impl LoadBufferJob {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self {
            open: Some(path),
            read: None,
        }
    }

    fn finish(contents: io::Result<Vec<u8>>) -> Result<Vec<u8>, i32> {
        contents.map_err(|error| error.raw_os_error().unwrap_or(libc::EIO))
    }
}

impl Coroutine for LoadBufferJob {
    type Output = Result<Vec<u8>, i32>;

    fn wait(&self) -> WaitRequest<'_> {
        match &self.read {
            Some(read) => read.wait(),
            None => WaitRequest::new(Vec::new(), Some(Instant::now())),
        }
    }

    fn resume(&mut self, ready: &ReadySet) -> TaskPoll<Self::Output> {
        if let Some(path) = self.open.take() {
            match open_path(&path) {
                PathOpen::Inline(contents) => return TaskPoll::Ready(Self::finish(contents)),
                PathOpen::Fifo(read) => self.read = Some(read),
            }
        }
        let Some(read) = self.read.as_mut() else {
            return TaskPoll::Pending;
        };
        match read.resume(ready) {
            TaskPoll::Ready(contents) => {
                self.read = None;
                TaskPoll::Ready(Self::finish(contents))
            }
            TaskPoll::Pending => TaskPoll::Pending,
        }
    }
}

/// `save-buffer path`, without `-` (the client's stdout): write a paste buffer
/// out, or report the `errno` the stock client would have seen. Regular files
/// are written inline for the same reason they are read inline; a FIFO waits
/// for its reader instead.
pub(crate) struct FileWriteJob {
    /// Set until the first resume classifies the path.
    request: Option<ClientFileWrite>,
    display_path: String,
    write: Option<FifoWrite>,
}

impl FileWriteJob {
    pub(crate) fn new(request: ClientFileWrite) -> Self {
        Self {
            display_path: request.display_path.clone(),
            request: Some(request),
            write: None,
        }
    }

    fn finish(&self, wrote: io::Result<()>) -> CommandResult {
        match wrote {
            Ok(()) => CommandResult::ok(""),
            Err(error) => {
                let mut result = CommandResult::err(format!(
                    "{}: {}\n",
                    io_error_message(&error),
                    self.display_path
                ));
                result.continue_queue = true;
                result
            }
        }
    }
}

impl Coroutine for FileWriteJob {
    type Output = CommandResult;

    fn wait(&self) -> WaitRequest<'_> {
        match &self.write {
            Some(write) => write.wait(),
            None => WaitRequest::new(Vec::new(), Some(Instant::now())),
        }
    }

    fn resume(&mut self, ready: &ReadySet) -> TaskPoll<Self::Output> {
        if let Some(request) = self.request.take() {
            match open_write_path(request) {
                WriteOpen::Inline(wrote) => return TaskPoll::Ready(self.finish(wrote)),
                WriteOpen::Fifo(write) => self.write = Some(write),
            }
        }
        let Some(write) = self.write.as_mut() else {
            return TaskPoll::Pending;
        };
        match write.resume(ready) {
            TaskPoll::Ready(wrote) => {
                self.write = None;
                TaskPoll::Ready(self.finish(wrote))
            }
            TaskPoll::Pending => TaskPoll::Pending,
        }
    }
}

/// `wait-for`, in whichever of its four forms.
///
/// `-S`, `-U` and an uncontended `-L` finish the moment the registry is
/// touched; the forms that have to wait for another client hold the completion
/// the registry signals, so the waiting queue is resumed by its own driver
/// rather than by parking a thread inside the registry.
pub(crate) enum WaitForJob {
    Done(Option<CommandResult>),
    Waiting(Completion<()>),
}

impl WaitForJob {
    pub(crate) fn new(args: &[String], registry: &WaitRegistry) -> Self {
        match execution::wait_for(args, registry) {
            WaitForOutcome::Done(result) => Self::Done(Some(result)),
            WaitForOutcome::Pending(completion) => Self::Waiting(completion),
        }
    }
}

impl Coroutine for WaitForJob {
    type Output = CommandResult;

    fn wait(&self) -> WaitRequest<'_> {
        match self {
            // Already resolved: `resume` reports it before anyone waits.
            Self::Done(_) => WaitRequest::new(Vec::new(), Some(Instant::now())),
            Self::Waiting(completion) => completion.wait(),
        }
    }

    fn resume(&mut self, ready: &ReadySet) -> TaskPoll<Self::Output> {
        match self {
            Self::Done(result) => TaskPoll::Ready(
                result
                    .take()
                    .expect("resolved wait-for reported its result twice"),
            ),
            Self::Waiting(completion) => match completion.resume(ready) {
                // The registry drops a sender only when the server is going
                // away, which the stock client sees as the wait being over.
                TaskPoll::Ready(_) => TaskPoll::Ready(CommandResult::ok("")),
                TaskPoll::Pending => TaskPoll::Pending,
            },
        }
    }
}

/// A command queue waiting for a client to answer something.
///
/// `command-prompt -w` waits for the prompt it put up on one client;
/// `confirm-before`, `display-menu` and `display-popup` wait for the overlay
/// they opened. Both hold the completion the answering side signals, so the
/// queue is resumed by its own driver instead of blocking on a receive.
pub(crate) enum ClientPromptJob {
    /// Nothing to wait for: no client took the request, or `-w` was absent.
    Done(Option<CommandResult>),
    /// An unanswered `command-prompt` reports the empty completion, which is
    /// what the stock client's queue continues with.
    Prompt(Completion<Option<PromptCompletion>>),
    /// A client that goes away mid-overlay leaves the queue running.
    Interaction(Completion<Option<PromptCompletion>>),
}

impl ClientPromptJob {
    pub(crate) fn prompt(
        args: Vec<String>,
        registry: &ClientPromptRegistry,
        target: Option<String>,
        tty_name: Option<String>,
        wait: bool,
    ) -> Self {
        let result = match registry.request_command(
            target.as_deref(),
            tty_name.as_deref(),
            args,
            wait,
        ) {
            CommandPromptRequestResult::Waiting(completion) => return Self::Prompt(completion),
            CommandPromptRequestResult::Queued | CommandPromptRequestResult::Busy => {
                CommandResult::ok("")
            }
            CommandPromptRequestResult::NoCurrentClient => CommandResult::err("no current client\n"),
            CommandPromptRequestResult::TargetNotFound => CommandResult::err(format!(
                "can't find client: {}\n",
                target.unwrap_or_default()
            )),
        };
        Self::Done(Some(result))
    }

    pub(crate) fn interaction(completed: Completion<Option<PromptCompletion>>) -> Self {
        Self::Interaction(completed)
    }
}

impl Coroutine for ClientPromptJob {
    type Output = CommandResult;

    fn wait(&self) -> WaitRequest<'_> {
        match self {
            // Already resolved: `resume` reports it before anyone waits.
            Self::Done(_) => WaitRequest::new(Vec::new(), Some(Instant::now())),
            Self::Prompt(completion) | Self::Interaction(completion) => completion.wait(),
        }
    }

    fn resume(&mut self, ready: &ReadySet) -> TaskPoll<Self::Output> {
        let answered = match self {
            Self::Done(result) => {
                return TaskPoll::Ready(
                    result
                        .take()
                        .expect("resolved client prompt reported its result twice"),
                )
            }
            Self::Prompt(completion) | Self::Interaction(completion) => {
                match completion.resume(ready) {
                    TaskPoll::Ready(answered) => answered.ok().flatten(),
                    TaskPoll::Pending => return TaskPoll::Pending,
                }
            }
        };
        TaskPoll::Ready(match (&self, answered) {
            (_, Some(completion)) => interaction_completion_result(completion),
            (Self::Prompt(_), None) => interaction_completion_result(PromptCompletion {
                stdout: String::new(),
                stderr: String::new(),
                exit: 0,
                inserted: false,
            }),
            (_, None) => {
                let mut result = CommandResult::ok("");
                result.continue_queue = true;
                result
            }
        })
    }
}

/// How a path is written: everything but a FIFO is written on the spot.
enum WriteOpen {
    Inline(io::Result<()>),
    Fifo(FifoWrite),
}

fn open_write_path(request: ClientFileWrite) -> WriteOpen {
    let ClientFileWrite {
        path, flags, data, ..
    } = request;
    let is_fifo = std::fs::metadata(&path).is_ok_and(|metadata| metadata.file_type().is_fifo());
    if !is_fifo {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true);
        if flags & libc::O_APPEND != 0 {
            options.append(true);
        } else {
            options.truncate(true);
        }
        return WriteOpen::Inline(
            options
                .open(&path)
                .and_then(|mut file| file.write_all(&data)),
        );
    }
    // Opening the write end blocks until a reader attaches, and the reader is
    // very often another client of this server, so let the job wait for it.
    WriteOpen::Fifo(FifoWrite::new(path, flags, data))
}

enum FifoWriteStage {
    /// No reader has attached yet, so the open still reports `ENXIO`; retrying
    /// on a backing-off deadline, since an unopened FIFO is not pollable.
    Opening { retry: Instant, backoff: Duration },
    /// The write end is open: drain the buffer as the reader consumes it.
    Writing { file: File, written: usize },
    /// The buffer was written, or the write failed.
    Done,
}

/// The write end of a FIFO, filled until the buffer is gone.
struct FifoWrite {
    path: PathBuf,
    flags: i32,
    data: Vec<u8>,
    stage: FifoWriteStage,
}

impl FifoWrite {
    const WRITABLE: WaitToken = WaitToken::new(0);

    fn new(path: PathBuf, flags: i32, data: Vec<u8>) -> Self {
        Self {
            path,
            flags,
            data,
            stage: FifoWriteStage::Opening {
                retry: Instant::now(),
                backoff: FIFO_WRITER_RETRY_MIN,
            },
        }
    }

    /// Try the non-blocking open once. `ENXIO` — no reader yet — is the only
    /// outcome worth retrying; everything else is the error tmux would report.
    fn open(&mut self) -> TaskPoll<io::Result<()>> {
        // `O_TRUNC` and `O_APPEND` mean nothing on a FIFO, but passing the
        // request's flags through keeps the open identical to the client's.
        match std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(self.flags | libc::O_NONBLOCK)
            .open(&self.path)
        {
            Ok(file) => {
                self.stage = FifoWriteStage::Writing { file, written: 0 };
                TaskPoll::Pending
            }
            Err(error) if error.raw_os_error() == Some(libc::ENXIO) => {
                let FifoWriteStage::Opening { retry, backoff } = &mut self.stage else {
                    unreachable!("only the opening stage opens the FIFO");
                };
                *retry = Instant::now() + *backoff;
                *backoff = (*backoff * 2).min(FIFO_WRITER_RETRY_MAX);
                TaskPoll::Pending
            }
            Err(error) => {
                self.stage = FifoWriteStage::Done;
                TaskPoll::Ready(Err(error))
            }
        }
    }
}

impl Coroutine for FifoWrite {
    type Output = io::Result<()>;

    fn wait(&self) -> WaitRequest<'_> {
        match &self.stage {
            FifoWriteStage::Opening { retry, .. } => WaitRequest::new(Vec::new(), Some(*retry)),
            FifoWriteStage::Writing { file, .. } => WaitRequest::new(
                vec![FdInterest::writable(Self::WRITABLE, file.as_fd())],
                None,
            ),
            FifoWriteStage::Done => WaitRequest::new(Vec::new(), Some(Instant::now())),
        }
    }

    fn resume(&mut self, _ready: &ReadySet) -> TaskPoll<Self::Output> {
        // Like the read side, every stage only attempts non-blocking work, so a
        // spurious wakeup costs one syscall and no driver has to describe which
        // source woke it.
        if matches!(self.stage, FifoWriteStage::Opening { .. }) {
            if let TaskPoll::Ready(result) = self.open() {
                return TaskPoll::Ready(result);
            }
        }
        let FifoWriteStage::Writing { file, written } = &mut self.stage else {
            return TaskPoll::Pending;
        };
        // Write as much as the pipe takes per wakeup: an 8 MiB buffer moves in
        // pipe-sized bites, and one write per readiness would need a wakeup for
        // every one of them.
        while *written < self.data.len() {
            match file.write(&self.data[*written..]) {
                Ok(0) => {
                    self.stage = FifoWriteStage::Done;
                    return TaskPoll::Ready(Err(io::Error::from(io::ErrorKind::WriteZero)));
                }
                Ok(wrote) => *written += wrote,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    return TaskPoll::Pending
                }
                Err(error) => {
                    self.stage = FifoWriteStage::Done;
                    return TaskPoll::Ready(Err(error));
                }
            }
        }
        // Dropping the write end here, rather than when the executor retires
        // the job, is what reports end of file to the reader.
        self.stage = FifoWriteStage::Done;
        TaskPoll::Ready(Ok(()))
    }
}

/// How a path is read: everything but a FIFO is read on the spot.
enum PathOpen {
    Inline(io::Result<Vec<u8>>),
    Fifo(FifoRead),
}

fn open_path(path: &Path) -> PathOpen {
    let is_fifo = std::fs::metadata(path).is_ok_and(|metadata| metadata.file_type().is_fifo());
    if !is_fifo {
        return PathOpen::Inline(std::fs::read(path));
    }
    // A blocking open would wait here for a writer that is very often another
    // client of this server, so open the read end without blocking and let the
    // job wait for the data instead.
    match std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)
    {
        Ok(file) => PathOpen::Fifo(FifoRead::new(file)),
        Err(error) => PathOpen::Inline(Err(error)),
    }
}

/// How long to wait before looking again for a writer on a FIFO nobody has
/// opened yet. A read end with no writer reports end of file rather than
/// blocking, and an absent writer is not a readiness event, so this stage
/// cannot be described to a poller.
const FIFO_WRITER_RETRY_MIN: Duration = Duration::from_millis(1);
const FIFO_WRITER_RETRY_MAX: Duration = Duration::from_millis(50);

enum FifoStage {
    /// No writer has appeared yet; retrying the read on a backing-off deadline.
    Waiting { retry: Instant, backoff: Duration },
    /// A writer is attached: end of file now means the writer is done.
    Reading,
    /// The contents were reported.
    Done,
}

/// The read end of a FIFO, drained until the writer closes it.
struct FifoRead {
    file: File,
    data: Vec<u8>,
    stage: FifoStage,
}

impl FifoRead {
    const READABLE: WaitToken = WaitToken::new(0);

    fn new(file: File) -> Self {
        Self {
            file,
            data: Vec::new(),
            stage: FifoStage::Waiting {
                retry: Instant::now(),
                backoff: FIFO_WRITER_RETRY_MIN,
            },
        }
    }
}

impl Coroutine for FifoRead {
    type Output = io::Result<Vec<u8>>;

    fn wait(&self) -> WaitRequest<'_> {
        match &self.stage {
            FifoStage::Waiting { retry, .. } => WaitRequest::new(Vec::new(), Some(*retry)),
            FifoStage::Reading => WaitRequest::new(
                vec![FdInterest::readable(Self::READABLE, self.file.as_fd())],
                None,
            ),
            FifoStage::Done => WaitRequest::new(Vec::new(), Some(Instant::now())),
        }
    }

    fn resume(&mut self, _ready: &ReadySet) -> TaskPoll<Self::Output> {
        // Every stage only ever attempts a non-blocking read, so a spurious
        // wakeup costs one syscall and no driver has to describe which source
        // woke it.
        let mut chunk = [0u8; 8192];
        loop {
            let read = match self.file.read(&mut chunk) {
                Ok(read) => read,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    // Only an attached writer leaves the pipe empty without
                    // reporting end of file.
                    self.stage = FifoStage::Reading;
                    return TaskPoll::Pending;
                }
                Err(error) => {
                    self.stage = FifoStage::Done;
                    return TaskPoll::Ready(Err(error));
                }
            };
            if read > 0 {
                self.data.extend_from_slice(&chunk[..read]);
                self.stage = FifoStage::Reading;
                continue;
            }
            return match &mut self.stage {
                // End of file before any writer appeared says nothing about the
                // writer that `source-file` is waiting for; ask again shortly.
                FifoStage::Waiting { retry, backoff } => {
                    *retry = Instant::now() + *backoff;
                    *backoff = (*backoff * 2).min(FIFO_WRITER_RETRY_MAX);
                    TaskPoll::Pending
                }
                _ => {
                    self.stage = FifoStage::Done;
                    TaskPoll::Ready(Ok(std::mem::take(&mut self.data)))
                }
            };
        }
    }
}

fn set_nonblocking(fd: BorrowedFd<'_>) -> io::Result<()> {
    let flags = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{FileWriteJob, IfShellJob, LoadBufferJob, RunShellJob, SourceFileJob, WaitForJob};
    use crate::server::command::{ClientContext, ClientFileWrite};
    use crate::server::state::WaitRegistry;
    use crate::server::task::{drive_blocking, run_blocking, ReadySet, TaskState};
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::process;
    use std::thread;
    use std::time::{Duration, Instant};

    fn args(values: &[&str]) -> Vec<String> {
        std::iter::once("run-shell")
            .chain(values.iter().copied())
            .map(str::to_string)
            .collect()
    }

    /// A client whose environment carries the test runner's `PATH`. The child
    /// is spawned with `environ_push` semantics — the context's environment is
    /// the whole of it — so a command that runs anything but a shell builtin
    /// has no way to find it otherwise.
    fn context_with_path() -> ClientContext {
        let mut context = ClientContext::default();
        if let Some(path) = std::env::var_os("PATH") {
            context.environment = vec![format!("PATH={}", path.to_string_lossy())];
        }
        context
    }

    #[test]
    fn run_shell_job_reports_stdout_and_the_exit_status() {
        let completion = run_blocking(RunShellJob::new(
            &args(&["echo hello; exit 3"]),
            &ClientContext::default(),
        ));

        assert_eq!(completion.result.exit, 3);
        assert_eq!(
            completion.result.stdout,
            "hello\n'echo hello; exit 3' returned 3\n"
        );
    }

    #[test]
    fn run_shell_job_drains_stderr_while_stdout_stays_open() {
        // The writer holds stdout open past the stderr burst, so a driver that
        // read the pipes in order would deadlock on a full stderr buffer.
        let command = "sh -c 'yes error >&2 & sleep 0.2; kill %1' 2>&1 >/dev/null | head -c 200000 >&2; echo done";
        let completion = run_blocking(RunShellJob::new(
            &args(&["-E", command]),
            &context_with_path(),
        ));

        assert_eq!(completion.result.exit, 0);
        assert!(completion.result.stdout.starts_with("done\n"));
        assert!(completion.result.stdout.len() > 100_000);
    }

    #[test]
    fn run_shell_job_waits_out_the_requested_delay() {
        let started = Instant::now();
        let completion = run_blocking(RunShellJob::new(
            &args(&["-d", "0.2", "echo late"]),
            &ClientContext::default(),
        ));

        assert!(started.elapsed() >= Duration::from_millis(150));
        assert_eq!(completion.result.stdout, "late\n");
    }

    #[test]
    fn run_shell_job_rejects_an_invalid_delay_without_a_child() {
        let completion = run_blocking(RunShellJob::new(
            &args(&["-d", "soon", "echo never"]),
            &ClientContext::default(),
        ));

        assert_eq!(completion.result.stderr, "invalid delay time: soon\n");
    }

    #[test]
    fn if_shell_job_reports_the_condition_status() {
        let context = ClientContext::default();
        assert!(run_blocking(IfShellJob::new("true", &context)));
        assert!(!run_blocking(IfShellJob::new("exit 7", &context)));
    }

    fn temporary_directory(name: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!("hmux-suspend-{name}-{}", process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("temporary directory");
        directory
    }

    fn mkfifo(path: &Path) -> PathBuf {
        let raw = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("FIFO path");
        assert_eq!(
            unsafe { libc::mkfifo(raw.as_ptr(), 0o600) },
            0,
            "mkfifo: {}",
            io::Error::last_os_error()
        );
        path.to_path_buf()
    }

    #[test]
    fn source_file_job_reads_regular_files_without_waiting() {
        let directory = temporary_directory("source-regular");
        let first = directory.join("first.conf");
        let second = directory.join("missing.conf");
        fs::write(&first, "set-buffer -b one yes\n").expect("write config");

        let reads = run_blocking(SourceFileJob::new(vec![
            first.display().to_string(),
            second.display().to_string(),
        ]));

        assert_eq!(reads.len(), 2);
        assert_eq!(
            reads[0].contents.as_deref().expect("first file"),
            "set-buffer -b one yes\n"
        );
        assert!(reads[0].existed);
        assert!(reads[1].contents.is_err());
        assert!(!reads[1].existed);
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn source_file_job_waits_for_the_fifo_writer() {
        let directory = temporary_directory("source-fifo");
        let fifo = mkfifo(&directory.join("commands"));
        let writer = thread::spawn({
            let fifo = fifo.clone();
            move || {
                thread::sleep(Duration::from_millis(100));
                fs::write(&fifo, b"set-buffer -b sourced yes\n").expect("write FIFO");
            }
        });

        let reads = run_blocking(SourceFileJob::new(vec![fifo.display().to_string()]));

        writer.join().expect("writer");
        assert_eq!(
            reads[0].contents.as_deref().expect("FIFO contents"),
            "set-buffer -b sourced yes\n"
        );
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn load_buffer_job_reads_a_fifo_written_in_pieces() {
        let directory = temporary_directory("load-fifo");
        let fifo = mkfifo(&directory.join("buffer"));
        let writer = thread::spawn({
            let fifo = fifo.clone();
            move || {
                let mut file = fs::OpenOptions::new()
                    .write(true)
                    .open(&fifo)
                    .expect("open FIFO");
                std::io::Write::write_all(&mut file, b"first ").expect("first write");
                thread::sleep(Duration::from_millis(50));
                std::io::Write::write_all(&mut file, b"second").expect("second write");
            }
        });

        let contents = run_blocking(LoadBufferJob::new(fifo.clone()));

        writer.join().expect("writer");
        assert_eq!(contents.expect("FIFO contents"), b"first second".to_vec());
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn load_buffer_job_reports_the_errno_of_a_missing_path() {
        let directory = temporary_directory("load-missing");

        let contents = run_blocking(LoadBufferJob::new(directory.join("absent")));

        assert_eq!(contents, Err(libc::ENOENT));
        let _ = fs::remove_dir_all(&directory);
    }

    fn file_write(path: &Path, flags: i32, data: &[u8]) -> ClientFileWrite {
        ClientFileWrite {
            path: path.to_path_buf(),
            display_path: path.display().to_string(),
            flags,
            data: data.to_vec(),
        }
    }

    #[test]
    fn file_write_job_writes_regular_files_without_waiting() {
        let directory = temporary_directory("save-regular");
        let path = directory.join("buffer");

        let truncating = libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC;
        let result = run_blocking(FileWriteJob::new(file_write(&path, truncating, b"first\n")));
        assert_eq!(result.exit, 0, "{}", result.stderr);
        let appending = libc::O_WRONLY | libc::O_CREAT | libc::O_APPEND;
        let result = run_blocking(FileWriteJob::new(file_write(&path, appending, b"second\n")));
        assert_eq!(result.exit, 0, "{}", result.stderr);

        assert_eq!(fs::read(&path).expect("saved file"), b"first\nsecond\n");
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn file_write_job_waits_for_the_fifo_reader() {
        let directory = temporary_directory("save-fifo");
        let fifo = mkfifo(&directory.join("buffer"));
        // Comfortably more than a pipe buffer, so the write only finishes as
        // the reader drains it.
        let payload = vec![b'x'; 512 * 1024];
        let reader = thread::spawn({
            let fifo = fifo.clone();
            move || {
                thread::sleep(Duration::from_millis(100));
                fs::read(&fifo).expect("read FIFO")
            }
        });

        let result = run_blocking(FileWriteJob::new(file_write(
            &fifo,
            libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
            &payload,
        )));

        assert_eq!(result.exit, 0, "{}", result.stderr);
        assert_eq!(reader.join().expect("reader"), payload);
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn file_write_job_reports_an_unwritable_path_and_continues_the_queue() {
        let directory = temporary_directory("save-unwritable");

        let result = run_blocking(FileWriteJob::new(file_write(
            &directory,
            libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
            b"buffer",
        )));

        assert_ne!(result.exit, 0);
        assert!(
            result
                .stderr
                .ends_with(&format!("{}\n", directory.display())),
            "{}",
            result.stderr
        );
        assert!(result.continue_queue);
        let _ = fs::remove_dir_all(&directory);
    }

    fn wait_for_args(values: &[&str]) -> Vec<String> {
        std::iter::once("wait-for")
            .chain(values.iter().copied())
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn wait_for_job_consumes_a_signal_that_already_arrived() {
        let registry = WaitRegistry::default();

        run_blocking(WaitForJob::new(&wait_for_args(&["-S", "ready"]), &registry));
        let started = Instant::now();
        let result = run_blocking(WaitForJob::new(&wait_for_args(&["ready"]), &registry));

        assert_eq!(result.exit, 0, "{}", result.stderr);
        assert!(started.elapsed() < Duration::from_millis(50));
    }

    /// Start a `wait-for` job and check it has not resolved yet. The registry
    /// and the waiter share a thread, so the job is driven one poll at a time
    /// instead of parked on a helper thread.
    fn pending_wait_for(args: &[&str], registry: &WaitRegistry) -> TaskState<WaitForJob> {
        let mut task = TaskState::new(WaitForJob::new(&wait_for_args(args), registry));
        assert!(
            !task.poll(&ReadySet::default()),
            "wait-for resolved before the channel was touched"
        );
        task
    }

    #[test]
    fn wait_for_job_resumes_when_the_signal_arrives() {
        let registry = WaitRegistry::default();
        let mut waiter = pending_wait_for(&["later"], &registry);

        registry.signal("later");

        drive_blocking(&mut waiter);
        let result = waiter.take_output().expect("signalled wait-for");
        assert_eq!(result.exit, 0, "{}", result.stderr);
    }

    #[test]
    fn wait_for_job_hands_the_lock_to_the_next_waiter_in_order() {
        let registry = WaitRegistry::default();

        // The first `-L` takes the lock outright.
        let result = run_blocking(WaitForJob::new(&wait_for_args(&["-L", "gate"]), &registry));
        assert_eq!(result.exit, 0, "{}", result.stderr);

        let mut waiter = pending_wait_for(&["-L", "gate"], &registry);
        assert!(registry.unlock("gate"), "first unlock");

        drive_blocking(&mut waiter);
        let result = waiter.take_output().expect("handed-off wait-for");
        assert_eq!(result.exit, 0, "{}", result.stderr);
        // The handoff kept the channel locked, so the second holder can release
        // it and nobody else can.
        assert!(registry.unlock("gate"), "second unlock");
        assert!(!registry.unlock("gate"), "third unlock");
    }

    #[test]
    fn wait_for_job_reports_an_unlock_of_a_free_channel() {
        let registry = WaitRegistry::default();

        let result = run_blocking(WaitForJob::new(&wait_for_args(&["-U", "free"]), &registry));

        assert_ne!(result.exit, 0);
        assert_eq!(result.stderr, "channel free not locked\n");
    }
}
