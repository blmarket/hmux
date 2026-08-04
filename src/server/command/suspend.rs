//! Runtime-neutral coroutines for the shell-backed command suspensions.
//!
//! `run-shell` and `if-shell` both run `sh -c` and need the child's output and
//! exit status before the command queue can continue. Expressing them as
//! [`Coroutine`]s instead of a blocking `Command::output()` gives the suspension
//! an explicit readiness description — an optional pre-spawn delay, then the
//! child's stdout and stderr pipes — so the server loop can drive the job
//! between its other work, and a blocking test driver can run the very same
//! job on the calling thread.

use std::io::{self, Read};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};

use crate::server::task::{Coroutine, FdInterest, ReadySet, TaskPoll, WaitRequest, WaitToken};

use super::{
    flag_value, has_flag, job_delay, positionals, shell_command, ClientContext, CommandResult,
    RunShellCompletion,
};

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
    use super::{IfShellJob, RunShellJob};
    use crate::server::command::ClientContext;
    use crate::server::task::run_blocking;
    use std::time::{Duration, Instant};

    fn args(values: &[&str]) -> Vec<String> {
        std::iter::once("run-shell")
            .chain(values.iter().copied())
            .map(str::to_string)
            .collect()
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
            &ClientContext::default(),
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
}
