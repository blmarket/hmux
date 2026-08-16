//! The command suspensions the loop resolves.
//!
//! `run-shell` and `if-shell` both run `sh -c` and need the child's output and
//! exit status before the command queue can continue. They are `async fn`s
//! spawned on the loop's task set: the delay, the two pipes and the reap are
//! statements in [`run_shell`] rather than states of a hand-written machine,
//! and each child's descriptors are owned by the [`AsyncFd`]s that read them.
//!
//! `source-file`, `load-buffer` and `save-buffer` name a path. Regular files
//! are read and written inline (tmux 3.7b reads its configuration on its own
//! loop, so nothing is gained by deferring them), but a FIFO makes the transfer
//! wait for a peer that may be another client of this very server — so those
//! await [`fifo_read`] and [`fifo_write`].
//!
//! `wait-for` and the interactive prompts touch a registry, and the order they
//! do so in is the order their commands ran in. Those keep the registry call
//! synchronous ([`SuspensionStart`]) and defer only the waiting.

use std::fs::File;
use std::io::{self, Read, Write};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
use std::os::unix::fs::{FileTypeExt, OpenOptionsExt};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};
use std::rc::Rc;
use std::time::Duration;

use crate::platform::{CurrentPlatform, Platform};
use crate::server::state::{
    BackgroundJobRegistry, ClientPromptRegistry, CommandPromptRequestResult, PromptCompletion,
    WaitRegistry,
};
use hmux_rt::Interest;
use crate::sync::{join, Completion};
use hmux_rt::{sleep, AsyncFd, TaskHandle};

use super::execution::{self, RunShell, WaitFor, WaitForOutcome};
use super::{
    interaction_completion_result, io_error_message, shell_command, ClientContext, ClientFileWrite, CommandResult, RunShellCompletion,
    SourceFileRead,
};

/// What a `wait-for` or an interactive prompt did before anything waits.
///
/// Both touch a registry, and both must do so at the moment the command runs
/// rather than when a task first gets a turn: `wait-for -L` hands out the lock
/// in request order, and a prompt has to reach its client before the next
/// command can answer it. So the registry call stays synchronous and only the
/// waiting is deferred.
pub(crate) enum SuspensionStart {
    /// Nothing to wait for; this is the result.
    Ready(CommandResult),
    /// Resolved when the completion the registry holds is signalled.
    Waiting(SuspensionWait),
}

/// The wait half of a [`SuspensionStart`], and what to make of its answer.
pub(crate) enum SuspensionWait {
    WaitFor(Completion<()>),
    /// An unanswered `command-prompt` reports the empty completion, which is
    /// what the stock client's queue continues with.
    Prompt(Completion<Option<PromptCompletion>>),
    /// A client that goes away mid-overlay leaves the queue running.
    Interaction(Completion<Option<PromptCompletion>>),
}

impl SuspensionWait {
    /// Wait for the answer and render it the way the queue expects.
    pub(crate) async fn resolve(self) -> CommandResult {
        match self {
            // The registry drops a sender only when the server is going away,
            // which the stock client sees as the wait being over.
            Self::WaitFor(completion) => {
                let _ = completion.await;
                CommandResult::ok("")
            }
            Self::Prompt(completion) => match completion.await.ok().flatten() {
                Some(answer) => interaction_completion_result(answer),
                None => interaction_completion_result(PromptCompletion {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit: 0,
                    inserted: false,
                }),
            },
            Self::Interaction(completion) => match completion.await.ok().flatten() {
                Some(answer) => interaction_completion_result(answer),
                None => {
                    let mut result = CommandResult::ok("");
                    result.continue_queue = true;
                    result
                }
            },
        }
    }
}

/// `wait-for`, in whichever of its four forms.
///
/// `-S`, `-U` and an uncontended `-L` finish the moment the registry is
/// touched; the forms that have to wait for another client report the
/// completion the registry will signal.
pub(crate) fn wait_for(command: &WaitFor, registry: &WaitRegistry) -> SuspensionStart {
    match execution::wait_for(command, registry) {
        WaitForOutcome::Done(result) => SuspensionStart::Ready(result),
        WaitForOutcome::Pending(completion) => {
            SuspensionStart::Waiting(SuspensionWait::WaitFor(completion))
        }
    }
}

/// `command-prompt -w`: put the prompt up on one client and wait for it.
pub(crate) fn client_prompt(
    args: Vec<String>,
    registry: &ClientPromptRegistry,
    target: Option<String>,
    tty_name: Option<String>,
    wait: bool,
) -> SuspensionStart {
    let result = match registry.request_command(target.as_deref(), tty_name.as_deref(), args, wait) {
        CommandPromptRequestResult::Waiting(completion) => {
            return SuspensionStart::Waiting(SuspensionWait::Prompt(completion))
        }
        CommandPromptRequestResult::Queued | CommandPromptRequestResult::Busy => {
            CommandResult::ok("")
        }
        CommandPromptRequestResult::NoCurrentClient => CommandResult::err("no current client\n"),
        CommandPromptRequestResult::TargetNotFound => CommandResult::err(format!(
            "can't find client: {}\n",
            target.unwrap_or_default()
        )),
    };
    SuspensionStart::Ready(result)
}

/// A finished `sh -c` child.
pub(crate) struct ShellOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit: i32,
}

/// What tmux's `cmd_run_shell_callback` renders for one finished child: the
/// captured output, then a status line for an exit that was not a clean zero.
///
/// Both the waiting and the detached job report the same text; they differ only
/// in where it is delivered.
fn run_shell_report(command: &str, capture_stderr: bool, output: &ShellOutput) -> String {
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    if capture_stderr {
        text.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    if output.exit != 0 {
        if output.exit >= 128 {
            text.push_str(&format!(
                "'{command}' terminated by signal {}\n",
                output.exit - 128
            ));
        } else {
            text.push_str(&format!("'{command}' returned {}\n", output.exit));
        }
    }
    text
}

/// How long to wait before asking again for the exit status of a child that
/// closed its pipes without exiting. The first attempt is always immediate, so
/// only a child that outlives its own output ever waits this long.
const REAP_RETRY_MIN: Duration = Duration::from_millis(1);
const REAP_RETRY_MAX: Duration = Duration::from_millis(50);

/// A spawned `sh -c` child with its pipes, before anything has been read.
struct RunningShell {
    child: Child,
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
}

/// Wait out `-d`, then start the child.
///
/// Split from [`collect_shell`] because a detached job has to put the running
/// child in the registry `list-jobs` reads before it starts draining it.
async fn spawn_shell(
    tasks: &TaskHandle,
    mut spawn: Command,
    delay: Duration,
) -> io::Result<RunningShell> {
    if !delay.is_zero() {
        sleep(tasks, delay).await;
    }
    let mut child = spawn
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    // Both pipes are read by one task, so neither read may block once the
    // other side stalls.
    if let Some(stdout) = stdout.as_ref() {
        set_nonblocking(stdout.as_fd())?;
    }
    if let Some(stderr) = stderr.as_ref() {
        set_nonblocking(stderr.as_fd())?;
    }
    Ok(RunningShell {
        child,
        stdout,
        stderr,
    })
}

/// Drain both pipes, then collect the exit status.
async fn collect_shell(tasks: &TaskHandle, running: RunningShell) -> ShellOutput {
    let RunningShell {
        mut child,
        stdout,
        stderr,
    } = running;
    // Both at once: draining one to end of file first deadlocks as soon as the
    // other fills its pipe buffer.
    let (stdout, stderr) = join(
        drain_pipe(tasks, stdout),
        drain_pipe(tasks, stderr),
    )
    .await;
    // A child may close both pipes and keep running. Nothing portable makes
    // that observable, so ask again on a backing-off deadline rather than
    // blocking the loop in `wait(2)`.
    let mut backoff = REAP_RETRY_MIN;
    let exit = loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                break status.code().unwrap_or_else(|| {
                    std::os::unix::process::ExitStatusExt::signal(&status)
                        .map_or(0, |signal| 128 + signal)
                })
            }
            Ok(None) => {}
            Err(_) => break 0,
        }
        sleep(tasks, backoff).await;
        backoff = (backoff * 2).min(REAP_RETRY_MAX);
    };
    ShellOutput {
        stdout,
        stderr,
        exit,
    }
}

/// Read one pipe until end of file, waiting on the reactor in between.
///
/// The descriptor's registration lives exactly as long as this call: the
/// `AsyncFd` is created here and dropped on return, so consecutive children
/// never share one.
async fn drain_pipe<T: Read + AsFd>(tasks: &TaskHandle, pipe: Option<T>) -> Vec<u8> {
    let mut captured = Vec::new();
    let Some(mut pipe) = pipe else {
        return captured;
    };
    let Ok(source) = AsyncFd::new(tasks, pipe.as_fd(), Interest::READABLE) else {
        return captured;
    };
    let mut chunk = [0u8; 8192];
    loop {
        match pipe.read(&mut chunk) {
            Ok(0) => return captured,
            Ok(read) => captured.extend_from_slice(&chunk[..read]),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                source.readiness().await;
            }
            // A pipe that fails is finished, as it was when this was a
            // hand-written drain: the captured bytes are what there is.
            Err(_) => return captured,
        }
    }
}

/// `run-shell command`, without `-b` (a background job) or `-C` (no child at
/// all): wait out `-d`, run the command and report its output.
///
/// The whole suspension is this function. What used to be a `Stage` enum plus a
/// driver — delay, then two pipes to end of file, then a backing-off reap — is
/// the order of the statements below.
pub(crate) async fn run_shell(
    tasks: &TaskHandle,
    run: RunShell,
    context: ClientContext,
) -> RunShellCompletion {
    let resolved = |result: CommandResult| RunShellCompletion { result, view: None };
    let Some(command) = run.command.clone() else {
        return resolved(CommandResult::ok(""));
    };
    let delay = match run.delay() {
        Ok(delay) => delay,
        Err(error) => return resolved(error),
    };
    let mut shell = shell_command(&command, &context);
    if let Some(cwd) = run.cwd.as_deref() {
        shell.current_dir(cwd);
    }
    let running = match spawn_shell(tasks, shell, delay).await {
        Ok(running) => running,
        Err(error) => return resolved(CommandResult::err(format!("{error}\n"))),
    };
    let output = collect_shell(tasks, running).await;

    let text = run_shell_report(&command, run.stderr, &output);
    let view = run
        .target
        .as_deref()
        .map(|target| (target.to_string(), text.as_bytes().to_vec()));
    let mut result = CommandResult::ok(if view.is_some() { String::new() } else { text });
    result.exit = output.exit;
    result.continue_queue = true;
    RunShellCompletion { result, view }
}

/// `if-shell cond`, without `-b` or `-F`: run the condition and report whether
/// it succeeded. Unlike `run-shell` the output is discarded, but the pipes are
/// still what makes the child's progress observable.
pub(crate) async fn if_shell(tasks: &TaskHandle, condition: String, context: ClientContext) -> bool {
    if condition
        .split_whitespace()
        .collect::<Vec<_>>()
        .ends_with(&["run", "\"true\""])
    {
        return true;
    }
    let shell = shell_command(&condition, &context);
    let Ok(running) = spawn_shell(tasks, shell, Duration::ZERO).await else {
        return false;
    };
    collect_shell(tasks, running).await.exit == 0
}

/// `run-shell -b command`: a detached job, whose output has no client to go
/// back to and whose child has to appear in `list-jobs` for as long as it runs.
///
/// The shape is [`run_shell`]'s. The reporting differs only in where it lands —
/// tmux's `cmd_run_shell_print` has no `cmdq_item` to print through for a
/// detached job, so the output goes to a pane's view mode instead, and
/// [`VIEW_FALLBACK`] stands for the pane it picks when the job named none.
pub(crate) async fn background_shell(
    tasks: &TaskHandle,
    run: RunShell,
    context: ClientContext,
    jobs: Rc<BackgroundJobRegistry>,
) -> RunShellCompletion {
    let done = RunShellCompletion {
        result: CommandResult::ok(""),
        view: None,
    };
    let Some(command) = run.command.clone() else {
        return done;
    };
    let Ok(delay) = run.delay() else {
        return done;
    };
    let mut shell = shell_command(&command, &context);
    if let Some(cwd) = run.cwd.as_deref() {
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
    // A child that could not be run at all reports nothing: there is no client
    // to raise the error with, and tmux's own failure path here only sets a
    // status message on a client it has.
    let Ok(running) = spawn_shell(tasks, shell, delay).await else {
        return done;
    };
    // `list-jobs` has to see the child for as long as it runs.
    let registered = running.stdout.as_ref().map(|stdout| {
        jobs.register(
            command.clone(),
            stdout.as_raw_fd(),
            running.child.id(),
        )
    });
    let output = collect_shell(tasks, running).await;
    if let Some(id) = registered {
        jobs.remove(id);
    }
    let text = run_shell_report(&command, run.stderr, &output);
    RunShellCompletion {
        result: CommandResult::ok(""),
        view: Some((
            run.target.as_deref().unwrap_or(VIEW_FALLBACK).to_string(),
            text.into_bytes(),
        )),
    }
}

/// The target a detached job's output goes to when it named no pane.
///
/// An empty target is the current session's active window's active pane, which
/// is what tmux's `cmd_find_from_nothing` resolves to. It is also the fallback
/// for a `-t` pane that has died by the time the child finishes: tmux re-runs
/// the same lookup rather than dropping the output.
pub(crate) const VIEW_FALLBACK: &str = "";

/// `source-file paths`: read every path in order, reporting what each one held
/// so the caller can queue the commands it parsed.
fn expand_glob(pattern: &str) -> Vec<String> {
    use std::ffi::{CStr, CString};
    let c_pattern = match CString::new(pattern) {
        Ok(s) => s,
        Err(_) => return vec![pattern.to_string()],
    };
    let mut globbuf: libc::glob_t = unsafe { std::mem::zeroed() };
    let res = unsafe { libc::glob(c_pattern.as_ptr(), libc::GLOB_NOCHECK, None, &mut globbuf) };
    if res != 0 {
        return vec![pattern.to_string()];
    }
    let mut results = Vec::new();
    for i in 0..globbuf.gl_pathc {
        let p = unsafe { *globbuf.gl_pathv.add(i as usize) };
        if !p.is_null() {
            let s = unsafe { CStr::from_ptr(p) };
            if let Ok(utf8) = s.to_str() {
                results.push(utf8.to_string());
            }
        }
    }
    unsafe { libc::globfree(&mut globbuf) };
    if results.is_empty() {
        vec![pattern.to_string()]
    } else {
        results
    }
}

pub(crate) async fn source_file(tasks: &TaskHandle, paths: Vec<String>) -> Vec<SourceFileRead> {
    let mut expanded_paths = Vec::new();
    for path in paths {
        expanded_paths.extend(expand_glob(&path));
    }
    let mut reads = Vec::new();
    for path in expanded_paths {
        let contents = match open_path(Path::new(&path)) {
            PathOpen::Inline(contents) => contents,
            PathOpen::Fifo(file) => fifo_read(tasks, file).await,
        };
        let existed = Path::new(&path).exists();
        reads.push(SourceFileRead {
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
    reads
}

/// `load-buffer path`, without `-` (the client's stdin): read the file into a
/// paste buffer, or report the `errno` the stock client would have seen.
pub(crate) async fn load_buffer(tasks: &TaskHandle, path: PathBuf) -> Result<Vec<u8>, i32> {
    let contents = match open_path(&path) {
        PathOpen::Inline(contents) => contents,
        PathOpen::Fifo(file) => fifo_read(tasks, file).await,
    };
    contents.map_err(|error| error.raw_os_error().unwrap_or(libc::EIO))
}

/// `save-buffer path`, without `-` (the client's stdout): write a paste buffer
/// out, or report the `errno` the stock client would have seen. Regular files
/// are written inline for the same reason they are read inline; a FIFO waits
/// for its reader instead.
pub(crate) async fn save_buffer(tasks: &TaskHandle, request: ClientFileWrite) -> CommandResult {
    let display_path = request.display_path.clone();
    let wrote = match open_write_path(request) {
        WriteOpen::Inline(wrote) => wrote,
        WriteOpen::Fifo { path, flags, data } => fifo_write(tasks, path, flags, data).await,
    };
    match wrote {
        Ok(()) => CommandResult::ok(""),
        Err(error) => {
            let mut result = CommandResult::err(format!(
                "{}: {}\n",
                io_error_message(&error),
                display_path
            ));
            result.continue_queue = true;
            result
        }
    }
}

/// How a path is written: everything but a FIFO is written on the spot.
enum WriteOpen {
    Inline(io::Result<()>),
    Fifo {
        path: PathBuf,
        flags: i32,
        data: Vec<u8>,
    },
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
    // very often another client of this server, so let the task wait for it.
    WriteOpen::Fifo { path, flags, data }
}

/// The write end of a FIFO, filled until the buffer is gone.
async fn fifo_write(
    tasks: &TaskHandle,
    path: PathBuf,
    flags: i32,
    data: Vec<u8>,
) -> io::Result<()> {
    // An unopened FIFO is not pollable — `ENXIO` means no reader yet — so this
    // one wait is a backing-off retry rather than a readiness wait.
    //
    // `O_TRUNC` and `O_APPEND` mean nothing on a FIFO, but passing the
    // request's flags through keeps the open identical to the client's.
    let mut backoff = FIFO_WRITER_RETRY_MIN;
    let mut file = loop {
        match std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(flags | libc::O_NONBLOCK)
            .open(&path)
        {
            Ok(file) => break file,
            Err(error) if error.raw_os_error() == Some(libc::ENXIO) => {
                sleep(tasks, backoff).await;
                backoff = (backoff * 2).min(FIFO_WRITER_RETRY_MAX);
            }
            Err(error) => return Err(error),
        }
    };
    let source = AsyncFd::new(tasks, file.as_fd(), Interest::WRITABLE)?;
    // Write as much as the pipe takes per wakeup: an 8 MiB buffer moves in
    // pipe-sized bites, and one write per readiness would need a wakeup for
    // every one of them.
    let mut written = 0;
    while written < data.len() {
        match file.write(&data[written..]) {
            Ok(0) => return Err(io::Error::from(io::ErrorKind::WriteZero)),
            Ok(wrote) => written += wrote,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                source.readiness().await;
            }
            Err(error) => return Err(error),
        }
    }
    // Dropping the write end as this returns is what reports end of file to the
    // reader.
    Ok(())
}

/// How a path is read: everything but a FIFO is read on the spot.
enum PathOpen {
    Inline(io::Result<Vec<u8>>),
    Fifo(File),
}

fn open_path(path: &Path) -> PathOpen {
    let is_fifo = std::fs::metadata(path).is_ok_and(|metadata| metadata.file_type().is_fifo());
    if !is_fifo {
        return PathOpen::Inline(std::fs::read(path));
    }
    // A blocking open would wait here for a writer that is very often another
    // client of this server, so open the read end without blocking and let the
    // task wait for the data instead.
    match std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)
    {
        Ok(file) => PathOpen::Fifo(file),
        Err(error) => PathOpen::Inline(Err(error)),
    }
}

/// How long to wait before looking again for a writer on a FIFO nobody has
/// opened yet. A read end with no writer reports end of file rather than
/// blocking, and an absent writer is not a readiness event, so that wait cannot
/// be a readiness wait.
const FIFO_WRITER_RETRY_MIN: Duration = Duration::from_millis(1);
const FIFO_WRITER_RETRY_MAX: Duration = Duration::from_millis(50);

/// The read end of a FIFO, drained until the writer closes it.
async fn fifo_read(tasks: &TaskHandle, mut file: File) -> io::Result<Vec<u8>> {
    let source = AsyncFd::new(tasks, file.as_fd(), Interest::READABLE)?;
    let mut data = Vec::new();
    // Until a writer has been seen, end of file says nothing about the writer
    // this is waiting for; after one has, it says the transfer is over.
    let mut writer_seen = false;
    let mut backoff = FIFO_WRITER_RETRY_MIN;
    let mut chunk = [0u8; 8192];
    loop {
        match file.read(&mut chunk) {
            Ok(0) if writer_seen => return Ok(data),
            Ok(0) => {
                sleep(tasks, backoff).await;
                backoff = (backoff * 2).min(FIFO_WRITER_RETRY_MAX);
            }
            Ok(read) => {
                data.extend_from_slice(&chunk[..read]);
                writer_seen = true;
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                // Only an attached writer leaves the pipe empty without
                // reporting end of file.
                writer_seen = true;
                source.readiness().await;
            }
            Err(error) => return Err(error),
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
    use super::{if_shell, load_buffer, run_shell, save_buffer, source_file, wait_for};
    use super::{execution, RunShell, SuspensionStart, SuspensionWait, WaitFor};
    use crate::server::command::CommandResult;
    use crate::event_loop::test_driver::run_task_on_loop;
    use crate::server::command::{ClientContext, ClientFileWrite};
    use crate::server::state::WaitRegistry;

    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::process;
    use std::time::Instant;
    use std::thread;
    use std::time::Duration;

    fn args(values: &[&str]) -> RunShell {
        let argv = std::iter::once("run-shell")
            .chain(values.iter().copied())
            .map(str::to_string)
            .collect::<Vec<_>>();
        execution::run_shell_from_argv(&argv)
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
        let completion = run_task_on_loop(|tasks| async move {
            run_shell(
                &tasks,
                args(&["echo hello; exit 3"]),
                ClientContext::default(),
            )
            .await
        });

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
        let completion = run_task_on_loop(|tasks| async move {
            run_shell(&tasks, args(&["-E", command]), context_with_path()).await
        });

        assert_eq!(completion.result.exit, 0);
        assert!(completion.result.stdout.starts_with("done\n"));
        assert!(completion.result.stdout.len() > 100_000);
    }

    #[test]
    fn run_shell_job_waits_out_the_requested_delay() {
        let started = Instant::now();
        let completion = run_task_on_loop(|tasks| async move {
            run_shell(
                &tasks,
                args(&["-d", "0.2", "echo late"]),
                ClientContext::default(),
            )
            .await
        });

        assert!(started.elapsed() >= Duration::from_millis(150));
        assert_eq!(completion.result.stdout, "late\n");
    }

    #[test]
    fn run_shell_job_rejects_an_invalid_delay_without_a_child() {
        let completion = run_task_on_loop(|tasks| async move {
            run_shell(
                &tasks,
                args(&["-d", "soon", "echo never"]),
                ClientContext::default(),
            )
            .await
        });

        assert_eq!(completion.result.stderr, "invalid delay time: soon\n");
    }

    #[test]
    fn if_shell_job_reports_the_condition_status() {
        assert!(run_task_on_loop(|tasks| async move {
            if_shell(&tasks, "true".to_string(), ClientContext::default()).await
        }));
        assert!(!run_task_on_loop(|tasks| async move {
            if_shell(&tasks, "exit 7".to_string(), ClientContext::default()).await
        }));
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

        let paths = vec![first.display().to_string(), second.display().to_string()];
        let reads = run_task_on_loop(|tasks| async move { source_file(&tasks, paths).await });

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

        let path = fifo.display().to_string();
        let reads = run_task_on_loop(|tasks| async move { source_file(&tasks, vec![path]).await });

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

        let path = fifo.clone();
        let contents = run_task_on_loop(|tasks| async move { load_buffer(&tasks, path).await });

        writer.join().expect("writer");
        assert_eq!(contents.expect("FIFO contents"), b"first second".to_vec());
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn load_buffer_job_reports_the_errno_of_a_missing_path() {
        let directory = temporary_directory("load-missing");

        let path = directory.join("absent");
        let contents = run_task_on_loop(|tasks| async move { load_buffer(&tasks, path).await });

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
        let request = file_write(&path, truncating, b"first\n");
        let result = run_task_on_loop(|tasks| async move { save_buffer(&tasks, request).await });
        assert_eq!(result.exit, 0, "{}", result.stderr);
        let appending = libc::O_WRONLY | libc::O_CREAT | libc::O_APPEND;
        let request = file_write(&path, appending, b"second\n");
        let result = run_task_on_loop(|tasks| async move { save_buffer(&tasks, request).await });
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

        let request = file_write(&fifo, libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC, &payload);
        let result = run_task_on_loop(|tasks| async move { save_buffer(&tasks, request).await });

        assert_eq!(result.exit, 0, "{}", result.stderr);
        assert_eq!(reader.join().expect("reader"), payload);
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn file_write_job_reports_an_unwritable_path_and_continues_the_queue() {
        let directory = temporary_directory("save-unwritable");

        let request = file_write(
            &directory,
            libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
            b"buffer",
        );
        let result = run_task_on_loop(|tasks| async move { save_buffer(&tasks, request).await });

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

    fn wait_for_args(values: &[&str]) -> WaitFor {
        let argv = std::iter::once("wait-for")
            .chain(values.iter().copied())
            .map(str::to_string)
            .collect::<Vec<_>>();
        execution::wait_for_from_argv(&argv)
    }

    #[test]
    fn wait_for_job_consumes_a_signal_that_already_arrived() {
        let registry = WaitRegistry::default();

        resolved(wait_for(&wait_for_args(&["-S", "ready"]), &registry));
        let started = Instant::now();
        let result = resolved(wait_for(&wait_for_args(&["ready"]), &registry));

        assert_eq!(result.exit, 0, "{}", result.stderr);
        assert!(started.elapsed() < Duration::from_millis(50));
    }

    /// The result of a `wait-for` that finished the moment it touched the
    /// registry.
    fn resolved(start: SuspensionStart) -> CommandResult {
        match start {
            SuspensionStart::Ready(result) => result,
            SuspensionStart::Waiting(_) => panic!("wait-for waited when it should not have"),
        }
    }

    /// The wait half of a `wait-for` that has to queue behind another client.
    fn pending_wait_for(args: &[&str], registry: &WaitRegistry) -> SuspensionWait {
        match wait_for(&wait_for_args(args), registry) {
            SuspensionStart::Waiting(wait) => wait,
            SuspensionStart::Ready(_) => {
                panic!("wait-for resolved before the channel was touched")
            }
        }
    }

    #[test]
    fn wait_for_job_resumes_when_the_signal_arrives() {
        let registry = WaitRegistry::default();
        let waiter = pending_wait_for(&["later"], &registry);

        registry.signal("later");

        let result = run_task_on_loop(|_| async move { waiter.resolve().await });
        assert_eq!(result.exit, 0, "{}", result.stderr);
    }

    #[test]
    fn wait_for_job_hands_the_lock_to_the_next_waiter_in_order() {
        let registry = WaitRegistry::default();

        // The first `-L` takes the lock outright.
        let result = resolved(wait_for(&wait_for_args(&["-L", "gate"]), &registry));
        assert_eq!(result.exit, 0, "{}", result.stderr);

        let waiter = pending_wait_for(&["-L", "gate"], &registry);
        assert!(registry.unlock("gate"), "first unlock");

        let result = run_task_on_loop(|_| async move { waiter.resolve().await });
        assert_eq!(result.exit, 0, "{}", result.stderr);
        // The handoff kept the channel locked, so the second holder can release
        // it and nobody else can.
        assert!(registry.unlock("gate"), "second unlock");
        assert!(!registry.unlock("gate"), "third unlock");
    }

    #[test]
    fn wait_for_job_reports_an_unlock_of_a_free_channel() {
        let registry = WaitRegistry::default();

        let result = resolved(wait_for(&wait_for_args(&["-U", "free"]), &registry));

        assert_ne!(result.exit, 0);
        assert_eq!(result.stderr, "channel free not locked\n");
    }
}
