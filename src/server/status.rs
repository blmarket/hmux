//! Client status-line rendering.
//!
//! The status renderer deliberately ends at a styled cell screen. Terminal
//! placement and capability-sensitive output remain compositor concerns. This
//! mirrors tmux's `status_redraw` -> `screen` -> tty pipeline and prevents
//! logical status content from being changed to work around a tty capability.

use super::format::{self, Vars};
use super::state::{ClientRenderRegistry, RenderInvalidation, ServerState, Session, Winlink};
#[cfg(test)]
use super::style::CaptureStyleWriter;
use super::style::{self, CellPresentation, CellStyle, Colour, TerminalStyleWriter, VisualToken};
use super::term::{terminal_acs, terminal_utf8, TerminalCapabilities};
use crate::integration::status::{PaneAgents, StatusSnapshot};
use crate::server::task::{Coroutine, FdInterest, ReadySet, TaskPoll, WaitRequest, WaitToken};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::{self, Read};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

const DEFAULT_STATUS_LEFT: &str = "[#{session_name}] ";
const DEFAULT_STATUS_RIGHT: &str =
    "#{?window_bigger,[#{window_offset_x}#,#{window_offset_y}] ,}\"#{=21:pane_title}\" %H:%M %d-%b-%y";
const DEFAULT_WINDOW_FORMAT: &str =
    "#I:#{?pane_agent_state_emoji,#{pane_agent_state_emoji} #{b:pane_current_path},#W}#{?window_flags,#{window_flags}, }";
const DEFAULT_PANE_FORMAT: &str = "#P:[#T]#{?pane_flags,#{pane_flags}, }";

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct FormatJobKey {
    scope: String,
    command: String,
}

struct FormatJobEntry {
    expanded_command: String,
    output: String,
    running: bool,
    generation: u64,
    last_started: Instant,
    /// The wall-clock second the job last started in — tmux's `fj->last`, which
    /// is compared for equality, so a finished job is left alone until the
    /// second it ran in is over.
    last_started_second: i64,
    last_notified: Instant,
    process: Option<Arc<FormatJobProcess>>,
}

struct FormatJobProcess {
    cancelled: AtomicBool,
    pgid: AtomicI32,
    /// The pipe the job's output is read from, as `show-messages -J` reports it.
    fd: AtomicI32,
}

impl FormatJobProcess {
    fn new() -> Self {
        Self {
            cancelled: AtomicBool::new(false),
            pgid: AtomicI32::new(0),
            fd: AtomicI32::new(-1),
        }
    }

    fn register(&self, pgid: i32) -> bool {
        self.pgid.store(pgid, Ordering::Release);
        if self.cancelled.load(Ordering::Acquire) {
            self.kill(pgid);
            false
        } else {
            true
        }
    }

    fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
        let pgid = self.pgid.load(Ordering::Acquire);
        if pgid > 0 {
            self.kill(pgid);
        }
    }

    fn kill(&self, pgid: i32) {
        // Each format job is placed in its own process group. Killing the group
        // also stops shell children such as `sleep`, matching tmux job_free().
        unsafe {
            libc::kill(-pgid, libc::SIGKILL);
        }
    }
}

/// One live `#()` job, as `show-messages -J` lists it.
pub(crate) struct FormatJobInfo {
    pub(crate) command: String,
    pub(crate) fd: i32,
    pub(crate) pid: i32,
}

impl Drop for FormatJobEntry {
    fn drop(&mut self) {
        if let Some(process) = &self.process {
            process.cancel();
        }
    }
}

/// One tree of `#()` jobs. tmux keeps one per client and one for formats with
/// no client, keyed by the command and the session/window/pane the expansion
/// was anchored at (`fj->tag`).
pub(crate) struct FormatJobRegistry {
    jobs: Mutex<HashMap<FormatJobKey, FormatJobEntry>>,
    /// Jobs whose child is running but which the loop has not adopted yet.
    /// Format expansion happens deep inside rendering, so a launch records the
    /// job here and the loop picks it up on its next pass.
    pending: Mutex<Vec<FormatJob>>,
    /// Weak so a per-client tree stored beside its client entry does not keep
    /// the render registry that owns the entry alive.
    renders: Weak<ClientRenderRegistry>,
    eviction_timeout: Duration,
}

impl FormatJobRegistry {
    pub(crate) fn new(renders: &Arc<ClientRenderRegistry>) -> Self {
        Self {
            jobs: Mutex::new(HashMap::new()),
            pending: Mutex::new(Vec::new()),
            renders: Arc::downgrade(renders),
            eviction_timeout: Duration::from_secs(3600),
        }
    }

    /// The cached output of `command`, starting the job when nothing has run it
    /// in this wall-clock second. `vars` supplies the scope the tree is keyed
    /// by, so the same command expanded for two panes gets two entries.
    pub(crate) fn output_for(
        self: &Arc<Self>,
        command: &str,
        expanded: String,
        vars: &Vars,
        session_id: u32,
        cwd: Option<PathBuf>,
        environment: Arc<Vec<String>>,
        status: bool,
    ) -> String {
        let scope = format!(
            "{}|{}|{}",
            vars.lookup("session_id").unwrap_or(""),
            vars.lookup("window_id").unwrap_or(""),
            vars.lookup("pane_id").unwrap_or("")
        );
        self.output(
            FormatJobKey {
                scope,
                command: command.to_string(),
            },
            expanded,
            session_id,
            cwd,
            environment,
            status,
        )
    }

    /// Jobs launched since the last call, for the loop to drive.
    pub(crate) fn take_pending(&self) -> Vec<FormatJob> {
        self.pending
            .lock()
            .map(|mut pending| std::mem::take(&mut *pending))
            .unwrap_or_default()
    }

    /// Drop finished entries nothing has re-run within `eviction_timeout`, so a
    /// tree that outlives the formats that filled it does not grow without
    /// bound. A running job is never evicted: the loop still owns the entry's
    /// generation and would update a key that had been removed.
    fn evict_expired(
        jobs: &mut HashMap<FormatJobKey, FormatJobEntry>,
        eviction_timeout: Duration,
        now: Instant,
    ) {
        jobs.retain(|_, job| {
            job.running || now.saturating_duration_since(job.last_started) < eviction_timeout
        });
    }

    /// The jobs still running, for `show-messages -J`.
    pub(crate) fn running(&self) -> Vec<FormatJobInfo> {
        let Ok(jobs) = self.jobs.lock() else {
            return Vec::new();
        };
        let mut running = jobs
            .iter()
            .filter(|(_, job)| job.running)
            .filter_map(|(_, job)| {
                let process = job.process.as_ref()?;
                Some(FormatJobInfo {
                    command: job.expanded_command.clone(),
                    fd: process.fd.load(Ordering::Acquire),
                    pid: process.pgid.load(Ordering::Acquire),
                })
            })
            .collect::<Vec<_>>();
        running.sort_by(|left, right| left.pid.cmp(&right.pid));
        running
    }

    fn output(
        self: &Arc<Self>,
        key: FormatJobKey,
        expanded_command: String,
        session_id: u32,
        cwd: Option<PathBuf>,
        environment: Arc<Vec<String>>,
        status: bool,
    ) -> String {
        let now = Instant::now();
        let second = super::state::now_epoch();
        let mut launch = None;
        let output = {
            let Ok(mut jobs) = self.jobs.lock() else {
                return String::new();
            };
            Self::evict_expired(&mut jobs, self.eviction_timeout, now);
            match jobs.get_mut(&key) {
                Some(job) => {
                    let command_changed = job.expanded_command != expanded_command;
                    // tmux compares the *second* a job last ran in, not an
                    // elapsed duration: a finished job is left alone for the
                    // remainder of that second and re-runs on the next one.
                    if command_changed || (!job.running && job.last_started_second != second) {
                        if let Some(process) = job.process.take() {
                            process.cancel();
                        }
                        let process = Arc::new(FormatJobProcess::new());
                        job.expanded_command = expanded_command.clone();
                        job.running = true;
                        job.generation = job.generation.wrapping_add(1);
                        job.last_started = now;
                        job.last_started_second = second;
                        job.last_notified = now;
                        job.process = Some(Arc::clone(&process));
                        launch = Some((job.generation, process));
                    }
                    if job.running
                        && job.output.is_empty()
                        && now.saturating_duration_since(job.last_started) > Duration::from_secs(1)
                    {
                        format!("<'{}' not ready>", key.command)
                    } else {
                        job.output.clone()
                    }
                }
                None => {
                    let process = Arc::new(FormatJobProcess::new());
                    jobs.insert(
                        key.clone(),
                        FormatJobEntry {
                            expanded_command: expanded_command.clone(),
                            output: String::new(),
                            running: true,
                            generation: 1,
                            last_started: now,
                            last_started_second: second,
                            last_notified: now,
                            process: Some(Arc::clone(&process)),
                        },
                    );
                    launch = Some((1, process));
                    String::new()
                }
            }
        };

        if let Some((generation, process)) = launch {
            let job = FormatJob::spawn(
                Arc::downgrade(self),
                key,
                generation,
                session_id,
                status,
                &expanded_command,
                cwd.as_deref(),
                &environment,
                process,
            );
            if let Ok(mut pending) = self.pending.lock() {
                pending.push(job);
            }
        }
        output
    }

    fn update(
        &self,
        key: &FormatJobKey,
        generation: u64,
        session_id: u32,
        status: bool,
        output: String,
    ) {
        let notify = {
            let Ok(mut jobs) = self.jobs.lock() else {
                return;
            };
            let Some(job) = jobs.get_mut(key) else {
                return;
            };
            if job.generation != generation {
                return;
            }
            job.output = output;
            let now = Instant::now();
            if now.saturating_duration_since(job.last_notified) >= Duration::from_secs(1) {
                job.last_notified = now;
                true
            } else {
                false
            }
        };
        if notify && status {
            if let Some(renders) = self.renders.upgrade() {
                renders.publish_session(session_id, RenderInvalidation::STATUS);
            }
        }
    }

    fn complete(
        &self,
        key: FormatJobKey,
        generation: u64,
        session_id: u32,
        status: bool,
        output: String,
    ) {
        {
            let Ok(mut jobs) = self.jobs.lock() else {
                return;
            };
            let Some(job) = jobs.get_mut(&key) else {
                return;
            };
            if job.generation != generation {
                return;
            }
            job.running = false;
            job.process = None;
            if job.output != output {
                job.output = output;
            }
        }
        if status {
            if let Some(renders) = self.renders.upgrade() {
                renders.publish_session(session_id, RenderInvalidation::STATUS);
            }
        }
    }
}

/// Where a `#()` job runs, following tmux's `server_client_get_cwd`: a client
/// contributes its own working directory only while it has no session, so a
/// format expanded by an attached client runs in that client's *session*
/// directory.
pub(crate) fn job_cwd(session: Option<&Session>, client_cwd: Option<&std::path::Path>) -> Option<PathBuf> {
    match session {
        Some(session) => session
            .cwd()
            .map(Path::to_path_buf)
            .or_else(|| client_cwd.map(Path::to_path_buf)),
        None => client_cwd.map(Path::to_path_buf),
    }
}

/// One `#()` job, driven by the server loop.
///
/// The child is spawned where the job is launched, so its process group is
/// registered for cancellation before anything can observe the entry. What is
/// left — reading its output a line at a time and reaping it — is what the loop
/// drives, publishing each complete line to the registry as tmux does.
pub(crate) struct FormatJob {
    registry: Weak<FormatJobRegistry>,
    key: FormatJobKey,
    generation: u64,
    session_id: u32,
    status: bool,
    /// Kept alive so the entry's `show-messages -J` view stays valid, and so
    /// cancellation still has the process group to kill.
    _process: Arc<FormatJobProcess>,
    stage: FormatJobStage,
    /// The last line seen, which is what the finished job reports.
    output: String,
}

enum FormatJobStage {
    Reading {
        child: Child,
        stdout: ChildStdout,
        /// Bytes of the line currently being accumulated.
        partial: Vec<u8>,
    },
    Reaping {
        child: Child,
        retry: Instant,
        backoff: Duration,
    },
    Done,
}

impl FormatJob {
    const STDOUT: WaitToken = WaitToken::new(0);

    #[allow(clippy::too_many_arguments)]
    fn spawn(
        registry: Weak<FormatJobRegistry>,
        key: FormatJobKey,
        generation: u64,
        session_id: u32,
        status: bool,
        command: &str,
        cwd: Option<&std::path::Path>,
        environment: &[String],
        process: Arc<FormatJobProcess>,
    ) -> Self {
        let done = |process| Self {
            registry: registry.clone(),
            key: key.clone(),
            generation,
            session_id,
            status,
            _process: process,
            stage: FormatJobStage::Done,
            output: String::new(),
        };
        if process.cancelled.load(Ordering::Acquire) {
            return done(process);
        }
        let mut shell = Command::new("sh");
        shell
            .arg("-c")
            .arg(command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .process_group(0);
        if let Some(cwd) = cwd {
            shell.current_dir(cwd);
        }
        // tmux's `environ_push` replaces the child's environment outright
        // rather than adding to the server's own.
        shell.env_clear();
        for entry in environment {
            if let Some((name, value)) = entry.split_once('=') {
                shell.env(name, value);
            }
        }
        let Ok(mut child) = shell.spawn() else {
            return done(process);
        };
        if !process.register(child.id() as i32) {
            let _ = child.wait();
            return done(process);
        }
        let Some(stdout) = child.stdout.take() else {
            let _ = child.wait();
            return done(process);
        };
        process.fd.store(stdout.as_raw_fd(), Ordering::Release);
        // The loop reads this pipe between its other work, so the read may
        // never block once the child stalls mid-line.
        if set_nonblocking(stdout.as_fd()).is_err() {
            let _ = child.wait();
            return done(process);
        }
        Self {
            registry,
            key,
            generation,
            session_id,
            status,
            _process: process,
            stage: FormatJobStage::Reading {
                child,
                stdout,
                partial: Vec::new(),
            },
            output: String::new(),
        }
    }

    /// Consume whatever complete lines `partial` now holds, publishing each.
    fn take_lines(&mut self) {
        let FormatJobStage::Reading { partial, .. } = &mut self.stage else {
            return;
        };
        let mut lines = Vec::new();
        while let Some(end) = partial.iter().position(|byte| *byte == b'\n') {
            let mut line: Vec<u8> = partial.drain(..=end).collect();
            line.pop();
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            lines.push(String::from_utf8_lossy(&line).into_owned());
        }
        for line in lines {
            self.output = line;
            if let Some(registry) = Weak::upgrade(&self.registry) {
                registry.update(
                    &self.key,
                    self.generation,
                    self.session_id,
                    self.status,
                    self.output.clone(),
                );
            }
        }
    }

    /// At end of file a line without its newline is still the job's output, but
    /// tmux never published it as an update.
    fn take_trailing_line(&mut self) {
        let FormatJobStage::Reading { partial, .. } = &mut self.stage else {
            return;
        };
        if partial.is_empty() {
            return;
        }
        if partial.last() == Some(&b'\r') {
            partial.pop();
        }
        self.output = String::from_utf8_lossy(partial).into_owned();
        partial.clear();
    }
}

impl Coroutine for FormatJob {
    type Output = ();

    fn wait(&self) -> WaitRequest<'_> {
        match &self.stage {
            FormatJobStage::Reading { stdout, .. } => WaitRequest::new(
                vec![FdInterest::readable(Self::STDOUT, stdout.as_fd())],
                None,
            ),
            FormatJobStage::Reaping { retry, .. } => WaitRequest::new(Vec::new(), Some(*retry)),
            FormatJobStage::Done => WaitRequest::new(Vec::new(), Some(Instant::now())),
        }
    }

    fn resume(&mut self, _ready: &ReadySet) -> TaskPoll<Self::Output> {
        if let FormatJobStage::Reading { stdout, partial, .. } = &mut self.stage {
            let mut bytes = [0u8; 4096];
            let ended = loop {
                match stdout.read(&mut bytes) {
                    Ok(0) => break true,
                    Ok(count) => partial.extend_from_slice(&bytes[..count]),
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => break false,
                    Err(_) => break true,
                }
            };
            self.take_lines();
            if !ended {
                return TaskPoll::Pending;
            }
            self.take_trailing_line();
            let FormatJobStage::Reading { child, .. } =
                std::mem::replace(&mut self.stage, FormatJobStage::Done)
            else {
                unreachable!("the reading stage was just observed");
            };
            self.stage = FormatJobStage::Reaping {
                child,
                retry: Instant::now(),
                backoff: FORMAT_REAP_RETRY_MIN,
            };
        }
        if let FormatJobStage::Reaping {
            child,
            retry,
            backoff,
        } = &mut self.stage
        {
            match child.try_wait() {
                Ok(Some(_)) | Err(_) => {}
                Ok(None) => {
                    // The child can close its pipe and keep running; ask again
                    // rather than blocking the loop in `wait(2)`.
                    *retry = Instant::now() + *backoff;
                    *backoff = (*backoff * 2).min(FORMAT_REAP_RETRY_MAX);
                    return TaskPoll::Pending;
                }
            }
            self.stage = FormatJobStage::Done;
            if let Some(registry) = Weak::upgrade(&self.registry) {
                registry.complete(
                    self.key.clone(),
                    self.generation,
                    self.session_id,
                    self.status,
                    std::mem::take(&mut self.output),
                );
            }
        }
        TaskPoll::Ready(())
    }
}

const FORMAT_REAP_RETRY_MIN: Duration = Duration::from_millis(1);
const FORMAT_REAP_RETRY_MAX: Duration = Duration::from_millis(50);

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

/// The boundary used by the attach compositor. There is intentionally one
/// implementation: this trait separates status evaluation from terminal
/// serialization; it is not an alternate rendering engine extension point.
pub(crate) trait StatusRenderer {
    fn render<'a>(
        &'a mut self,
        state: &ServerState,
        session: &str,
        cols: u16,
        rows: u16,
    ) -> &'a RenderedStatus;

    fn invalidate(&mut self);
}

/// One attached client's cached status screen and serialized rows.
#[derive(Default)]
pub(crate) struct RenderCache {
    rendered: Option<RenderedStatus>,
    valid: bool,
    client: ClientContext,
    format_jobs: Option<Arc<FormatJobRegistry>>,
    agent_revision: u64,
    agents: PaneAgents,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClientContext {
    pub(crate) term: Option<String>,
    pub(crate) tty: Option<String>,
    pub(crate) pid: Option<i32>,
    pub(crate) cwd: Option<PathBuf>,
    pub(crate) environment: Vec<String>,
    pub(crate) control_mode: bool,
    pub(crate) read_only: bool,
    pub(crate) flags: String,
    /// The client's current key table — tmux's `#{client_key_table}`.
    pub(crate) key_table: String,
}

impl Default for ClientContext {
    fn default() -> Self {
        Self {
            term: None,
            tty: None,
            pid: None,
            cwd: None,
            environment: Vec::new(),
            control_mode: false,
            read_only: false,
            flags: String::new(),
            key_table: super::state::DEFAULT_KEY_TABLE.to_string(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RenderedStatus {
    cols: u16,
    terminal_rows: u16,
    screen: StatusScreen,
    #[cfg(test)]
    rows: Vec<Vec<u8>>,
    encoded: RefCell<Option<EncodedRows>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EncodedRows {
    terminal_generation: u64,
    rows: Vec<Vec<u8>>,
    rows_without_last: Vec<Vec<u8>>,
}

impl RenderedStatus {
    #[cfg(test)]
    pub(crate) fn row(&self, row: usize) -> &[u8] {
        self.rows.get(row).map(Vec::as_slice).unwrap_or_default()
    }

    pub(crate) fn append_row_for_terminal(
        &self,
        out: &mut Vec<u8>,
        row: usize,
        avoid_last: bool,
        terminal: &dyn TerminalCapabilities,
    ) {
        self.ensure_encoded(terminal);
        let encoded = self.encoded.borrow();
        let Some(encoded) = encoded.as_ref() else {
            return;
        };
        let row = if avoid_last {
            encoded.rows_without_last.get(row)
        } else {
            encoded.rows.get(row)
        };
        if let Some(row) = row {
            out.extend_from_slice(row);
        }
    }

    fn ensure_encoded(&self, terminal: &dyn TerminalCapabilities) {
        let generation = terminal.generation();
        if self
            .encoded
            .borrow()
            .as_ref()
            .is_some_and(|encoded| encoded.terminal_generation == generation)
        {
            return;
        }
        let rows = self
            .screen
            .rows
            .iter()
            .map(|row| serialize_row_width_for_terminal(row, usize::from(self.cols), terminal))
            .collect();
        let rows_without_last = self
            .screen
            .rows
            .iter()
            .map(|row| {
                serialize_row_width_for_terminal(
                    row,
                    usize::from(self.cols.saturating_sub(1)),
                    terminal,
                )
            })
            .collect();
        *self.encoded.borrow_mut() = Some(EncodedRows {
            terminal_generation: generation,
            rows,
            rows_without_last,
        });
    }

    pub(crate) fn range_at(&self, row: usize, column: u16) -> Option<&StatusRange> {
        self.screen
            .rows
            .get(row)?
            .ranges
            .iter()
            .find(|range| column >= range.start && column < range.end)
    }

    #[cfg(test)]
    fn screen(&self) -> &StatusScreen {
        &self.screen
    }
}

impl StatusRenderer for RenderCache {
    fn render<'a>(
        &'a mut self,
        state: &ServerState,
        session: &str,
        cols: u16,
        rows: u16,
    ) -> &'a RenderedStatus {
        let needs_render = !self.valid
            || self
                .rendered
                .as_ref()
                .is_none_or(|rendered| rendered.cols != cols || rendered.terminal_rows != rows);
        if needs_render {
            let jobs = self
                .format_jobs
                .get_or_insert_with(|| {
                    Arc::new(FormatJobRegistry::new(&state.client_render_registry()))
                })
                .clone();
            self.rendered = Some(render_status(
                state,
                session,
                cols,
                rows,
                &self.client,
                Some(&jobs),
                &self.agents,
            ));
            self.valid = true;
        }
        self.rendered.as_ref().expect("status cache populated")
    }

    fn invalidate(&mut self) {
        self.valid = false;
    }
}

impl RenderCache {
    /// Render for one registered client, sharing that client's `#()` job tree
    /// so a command it runs and its status line reach one cache — tmux's
    /// per-client `c->jobs`.
    pub(crate) fn for_client(client: ClientContext, format_jobs: Arc<FormatJobRegistry>) -> Self {
        Self {
            client,
            format_jobs: Some(format_jobs),
            ..Self::default()
        }
    }

    pub(crate) fn render<'a>(
        &'a mut self,
        state: &ServerState,
        session: &str,
        cols: u16,
        rows: u16,
    ) -> &'a RenderedStatus {
        <Self as StatusRenderer>::render(self, state, session, cols, rows)
    }

    pub(crate) fn invalidate(&mut self) {
        <Self as StatusRenderer>::invalidate(self)
    }

    /// Install the newest agent snapshot and invalidate cached status content
    /// when its observable revision changes.
    pub(crate) fn update_agents(&mut self, snapshot: StatusSnapshot) -> bool {
        if self.agent_revision == snapshot.revision {
            return false;
        }
        self.agent_revision = snapshot.revision;
        self.agents = snapshot.panes;
        self.invalidate();
        true
    }

    pub(crate) fn update_client_flags(&mut self, flags: String, read_only: bool) {
        self.client.flags = flags;
        self.client.read_only = read_only;
        self.invalidate();
    }

    /// Follow the client into a new key table so `#{client_key_table}` and
    /// `#{client_prefix}` in the status line track a pending prefix.
    pub(crate) fn update_client_key_table(&mut self, table: String) {
        if self.client.key_table == table {
            return;
        }
        self.client.key_table = table;
        self.invalidate();
    }

    /// Expand a client-scoped format outside the status rows. Terminal titles
    /// use the same variables, time expansion, and asynchronous job cache as
    /// the status line in tmux.
    pub(crate) fn expand_format(
        &mut self,
        state: &ServerState,
        session: &str,
        template: &str,
        cols: u16,
        rows: u16,
    ) -> Option<String> {
        let session = state.find(session)?;
        let jobs = self
            .format_jobs
            .get_or_insert_with(|| Arc::new(FormatJobRegistry::new(&state.client_render_registry())))
            .clone();
        let context = StatusContext::new(
            state,
            session,
            &self.client,
            cols,
            rows,
            Some(&jobs),
            false,
            &self.agents,
        );
        Some(context.expand_time(template, &context.base_vars()))
    }

    pub(crate) fn expand_format_for_target(
        &mut self,
        state: &ServerState,
        session: &str,
        window_id: Option<u32>,
        pane_id: Option<u32>,
        template: &str,
        cols: u16,
        rows: u16,
    ) -> Option<String> {
        let session = state.find(session)?;
        let jobs = self
            .format_jobs
            .get_or_insert_with(|| Arc::new(FormatJobRegistry::new(&state.client_render_registry())))
            .clone();
        let context = StatusContext::new(
            state,
            session,
            &self.client,
            cols,
            rows,
            Some(&jobs),
            false,
            &self.agents,
        );
        let window_index = match (window_id, pane_id) {
            (Some(window_id), _) => session
                .windows
                .iter()
                .position(|link| link.id == window_id)?,
            (None, Some(pane_id)) => session.windows.iter().position(|link| {
                state
                    .window_for_link(link)
                    .panes
                    .iter()
                    .any(|pane| pane.id == pane_id)
            })?,
            (None, None) => session.active,
        };
        let pane_index = pane_id.and_then(|pane_id| {
            let window = state.window_for_link(session.windows.get(window_index)?);
            window.panes.iter().position(|pane| pane.id == pane_id)
        });
        Some(context.expand_time(
            template,
            &context.vars_for(session, window_index, pane_index),
        ))
    }

    pub(crate) fn message_row(
        &mut self,
        state: &ServerState,
        session: &str,
        message: &str,
        cols: u16,
        rows: u16,
        writable_width: usize,
        terminal: &dyn TerminalCapabilities,
    ) -> Vec<u8> {
        let Some(session) = state.find(session) else {
            return Vec::new();
        };
        let jobs = state.format_job_registry();
        let context = StatusContext::new(
            state,
            session,
            &self.client,
            cols,
            rows,
            Some(&jobs),
            false,
            &self.agents,
        );
        let mut vars = context.base_vars();
        vars.set("message", message).set("command_prompt", "0");
        let message_style = context.expand(context.option("message-style"), &vars, 0);
        let template = context.option("message-format");
        let expanded = context.expand_time(template, &vars);
        let base =
            parse_status_cell_style(&message_style, &CellStyle::default(), &CellStyle::default());
        let row = draw_row(&expanded, usize::from(cols), &base);
        serialize_row_width_for_terminal(&row, row.used.min(writable_width), terminal)
    }
}

/// How many rows the status area occupies. tmux accepts `off`, `on`, and 1..5.
pub fn height(state: &ServerState, target: &str) -> u16 {
    match state.option_for_target(target, "status").unwrap_or("on") {
        "off" | "0" => 0,
        "2" => 2,
        "3" => 3,
        "4" => 4,
        "5" => 5,
        _ => 1,
    }
}

pub(crate) fn at_top(state: &ServerState, target: &str) -> bool {
    state.option_for_target(target, "status-position") == Some("top")
}

/// Serialize a configured style for compositor overlays such as prompts and
/// copy-mode marks. Their lifecycle remains in the attach loop, but their
/// colours and attributes use the same state machine as status rows.
/// The SGR bytes for a style already resolved to a literal value, for the
/// callers that have expanded a style option's own format first — tmux's
/// `style_apply` runs the option through the format tree before parsing it.
pub(crate) fn style_escape_value(value: &str, terminal: &dyn TerminalCapabilities) -> Vec<u8> {
    let style = parse_status_cell_style(value, &CellStyle::default(), &CellStyle::default());
    let mut out = Vec::new();
    let mut writer = TerminalStyleWriter::new(terminal);
    writer.reset(&mut out);
    writer.transition(
        &mut out,
        &CellPresentation {
            style,
            ..CellPresentation::default()
        },
    );
    out
}

pub(crate) fn option_style_escape_for(
    state: &ServerState,
    target: &str,
    option: &str,
    fallback: &str,
    terminal: &dyn TerminalCapabilities,
) -> Vec<u8> {
    option_style_escape_inner(state, target, option, fallback, terminal, StyleVariant::Plain)
}

/// How a style option is rendered when it is not applied as written.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StyleVariant {
    Plain,
    /// The two colours exchanged, which is how tmux's
    /// `window_copy_update_style` draws the marked cell itself.
    Reversed,
    /// The background alone, which is what reaches the blank tail of a row
    /// tmux clears to the end of the line.
    BackgroundOnly,
}

pub(crate) fn option_style_escape_variant(
    state: &ServerState,
    target: &str,
    option: &str,
    fallback: &str,
    terminal: &dyn TerminalCapabilities,
    variant: StyleVariant,
) -> Vec<u8> {
    option_style_escape_inner(state, target, option, fallback, terminal, variant)
}

fn option_style_escape_inner(
    state: &ServerState,
    target: &str,
    option: &str,
    fallback: &str,
    terminal: &dyn TerminalCapabilities,
    variant: StyleVariant,
) -> Vec<u8> {
    let mut value = state.option_for_target(target, option).unwrap_or(fallback);
    if let Some(name) = value
        .strip_prefix("#{E:")
        .and_then(|value| value.strip_suffix('}'))
    {
        value = state.option_for_target(target, name).unwrap_or(fallback);
    }
    let mut style = parse_status_cell_style(value, &CellStyle::default(), &CellStyle::default());
    match variant {
        StyleVariant::Plain => {}
        StyleVariant::Reversed => std::mem::swap(&mut style.fg, &mut style.bg),
        StyleVariant::BackgroundOnly => style.fg = CellStyle::default().fg,
    }
    let mut out = Vec::new();
    let mut writer = TerminalStyleWriter::new(terminal);
    writer.reset(&mut out);
    writer.transition(
        &mut out,
        &CellPresentation {
            style,
            ..CellPresentation::default()
        },
    );
    out
}

/// Render text which may contain tmux `#[...]` style directives for an overlay
/// such as a command prompt. The directives occupy no cells and must not leak
/// into the terminal as literal text.
#[cfg(test)]
pub(crate) fn render_overlay_text(
    value: &str,
    base_style: &str,
    width: usize,
    fill: bool,
) -> Vec<u8> {
    let base = parse_status_cell_style(base_style, &CellStyle::default(), &CellStyle::default());
    let row = draw_row(value, width, &base);
    serialize_row_width(&row, if fill { width } else { row.used.min(width) })
}

pub(crate) fn render_overlay_text_for_terminal(
    value: &str,
    base_style: &str,
    width: usize,
    fill: bool,
    terminal: &dyn TerminalCapabilities,
) -> Vec<u8> {
    let base = parse_status_cell_style(base_style, &CellStyle::default(), &CellStyle::default());
    let row = draw_row(value, width, &base);
    serialize_row_width_for_terminal(
        &row,
        if fill { width } else { row.used.min(width) },
        terminal,
    )
}

pub(crate) fn interval(state: &ServerState, target: &str) -> Option<Duration> {
    if height(state, target) == 0 {
        return None;
    }
    let seconds = state
        .option_for_target(target, "status-interval")
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(15);
    (seconds != 0).then(|| Duration::from_secs(seconds))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StatusScreen {
    rows: Vec<StatusRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StatusRow {
    cells: Vec<StatusCell>,
    ranges: Vec<StatusRange>,
    used: usize,
    /// The row's width in columns. A wide cell takes two columns out of one
    /// slot, so the number of cells stops being the number of columns as soon
    /// as one is placed — every bound here is a column count.
    columns: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StatusCell {
    text: String,
    width: u8,
    style: CellStyle,
    acs: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StatusRange {
    pub(crate) start: u16,
    pub(crate) end: u16,
    pub(crate) kind: StatusRangeKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StatusRangeKind {
    Left,
    Right,
    Window(u32),
    Pane(u32),
    Session(u32),
    User(String),
    Control(u32),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum Align {
    #[default]
    Left,
    Centre,
    Right,
    AbsoluteCentre,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Section {
    Left,
    Centre,
    Right,
    AbsoluteCentre,
    List,
    ListLeft,
    ListRight,
    After,
}

#[derive(Clone, Debug)]
struct DrawState {
    style: CellStyle,
    base: CellStyle,
    current_default: CellStyle,
    acs: bool,
    base_acs: bool,
    current_default_acs: bool,
    ignore: bool,
    section: Section,
    list_align: Option<Align>,
    list_active: bool,
    focus_start: Option<usize>,
    focus_end: Option<usize>,
    range: Option<StatusRangeKind>,
    fill: Option<Colour>,
}

impl DrawState {
    fn new(base: CellStyle) -> Self {
        Self {
            style: base,
            base,
            current_default: base,
            acs: false,
            base_acs: false,
            current_default_acs: false,
            ignore: false,
            section: Section::Left,
            list_align: None,
            list_active: false,
            focus_start: None,
            focus_end: None,
            range: None,
            fill: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct Sections {
    left: Vec<StatusCell>,
    centre: Vec<StatusCell>,
    right: Vec<StatusCell>,
    absolute_centre: Vec<StatusCell>,
    list: Vec<StatusCell>,
    list_left: Vec<StatusCell>,
    list_right: Vec<StatusCell>,
    after: Vec<StatusCell>,
    ranges: Vec<PendingRange>,
}

/// A `#[range=...]` region while the format is still being drawn.
///
/// tmux closes a range in `format_draw` with `fr->end = cx + 1`, so the region
/// it hands to `style_ranges_get_range` reaches one column *past* its own text
/// — which is why a click on the separator between two window entries still
/// names the window before it. A range never closed by a later style directive
/// is dropped rather than reaching the click map at all.
#[derive(Clone, Debug)]
struct PendingRange {
    section: Section,
    start: usize,
    end: usize,
    kind: StatusRangeKind,
    open: bool,
}

impl Sections {
    fn cells(&self, section: Section) -> &Vec<StatusCell> {
        match section {
            Section::Left => &self.left,
            Section::Centre => &self.centre,
            Section::Right => &self.right,
            Section::AbsoluteCentre => &self.absolute_centre,
            Section::List => &self.list,
            Section::ListLeft => &self.list_left,
            Section::ListRight => &self.list_right,
            Section::After => &self.after,
        }
    }

    fn cells_mut(&mut self, section: Section) -> &mut Vec<StatusCell> {
        match section {
            Section::Left => &mut self.left,
            Section::Centre => &mut self.centre,
            Section::Right => &mut self.right,
            Section::AbsoluteCentre => &mut self.absolute_centre,
            Section::List => &mut self.list,
            Section::ListLeft => &mut self.list_left,
            Section::ListRight => &mut self.list_right,
            Section::After => &mut self.after,
        }
    }

    fn width(&self, section: Section) -> usize {
        cells_width(self.cells(section))
    }
}

fn render_status(
    state: &ServerState,
    session: &str,
    cols: u16,
    terminal_rows: u16,
    client: &ClientContext,
    jobs: Option<&Arc<FormatJobRegistry>>,
    agents: &PaneAgents,
) -> RenderedStatus {
    let lines = usize::from(height(state, session));
    let mut base = parse_status_cell_style(
        state
            .option_for_target(session, "status-style")
            .unwrap_or("bg=green,fg=black"),
        &CellStyle::default(),
        &CellStyle::default(),
    );
    if let Some(fg) = state
        .option_for_target(session, "status-fg")
        .and_then(style::parse_colour)
    {
        if fg != Colour::Default {
            base.fg = fg;
        }
    }
    if let Some(bg) = state
        .option_for_target(session, "status-bg")
        .and_then(style::parse_colour)
    {
        if bg != Colour::Default {
            base.bg = bg;
        }
    }
    let Some(sess) = state.find(session) else {
        let row = blank_row(usize::from(cols), &base);
        let screen = StatusScreen {
            rows: vec![row; lines],
        };
        #[cfg(test)]
        let rows = screen.rows.iter().map(serialize_row).collect();
        return RenderedStatus {
            cols,
            terminal_rows,
            screen,
            #[cfg(test)]
            rows,
            encoded: RefCell::default(),
        };
    };

    let context = StatusContext::new(state, sess, client, cols, terminal_rows, jobs, true, agents);
    let mut rows = Vec::with_capacity(lines);
    for index in 0..lines {
        let expanded = match state.option_for_target(session, &format!("status-format[{index}]")) {
            Some(template) => context.expand_time(template, &context.base_vars()),
            None if index == 0 => context.default_first_line(),
            None if index == 1 => context.default_pane_line(),
            None if index == 2 => context.default_session_line(),
            None => String::new(),
        };
        rows.push(draw_row(&expanded, usize::from(cols), &base));
    }
    let screen = StatusScreen { rows };
    #[cfg(test)]
    let rows = screen.rows.iter().map(serialize_row).collect();
    RenderedStatus {
        cols,
        terminal_rows,
        screen,
        #[cfg(test)]
        rows,
        encoded: RefCell::default(),
    }
}

fn blank_row(cols: usize, base: &CellStyle) -> StatusRow {
    StatusRow {
        cells: (0..cols)
            .map(|_| StatusCell {
                text: " ".into(),
                width: 1,
                style: *base,
                acs: false,
            })
            .collect(),
        ranges: Vec::new(),
        used: 0,
        columns: cols,
    }
}

struct StatusContext<'a> {
    state: &'a ServerState,
    session: &'a Session,
    client: &'a ClientContext,
    cols: u16,
    rows: u16,
    jobs: Option<&'a Arc<FormatJobRegistry>>,
    job_status: bool,
    agents: &'a PaneAgents,
}

impl<'a> StatusContext<'a> {
    fn new(
        state: &'a ServerState,
        session: &'a Session,
        client: &'a ClientContext,
        cols: u16,
        rows: u16,
        jobs: Option<&'a Arc<FormatJobRegistry>>,
        job_status: bool,
        agents: &'a PaneAgents,
    ) -> Self {
        Self {
            state,
            session,
            client,
            cols,
            rows,
            jobs,
            job_status,
            agents,
        }
    }

    fn option(&self, name: &str) -> &str {
        self.state
            .option_for_target(&self.session.name, name)
            .or_else(|| super::options::option_default(name))
            .unwrap_or(match name {
                "status-left" => DEFAULT_STATUS_LEFT,
                "status-right" => DEFAULT_STATUS_RIGHT,
                "status-left-length" => "10",
                "status-right-length" => "40",
                "status-left-style" | "status-right-style" => "default",
                "message-style" => "bg=yellow,fg=black,fill=yellow",
                "status-justify" => "left",
                "window-status-format" | "window-status-current-format" => DEFAULT_WINDOW_FORMAT,
                "window-status-separator" => " ",
                "window-status-style" | "window-status-current-style" => "default",
                "window-status-last-style" => "default",
                "window-status-bell-style" | "window-status-activity-style" => "reverse",
                "window-pane-status-format" | "window-pane-current-status-format" => {
                    DEFAULT_PANE_FORMAT
                }
                "pane-status-style" | "pane-status-current-style" => "default",
                "session-status-style" | "session-status-current-style" => "default",
                _ => "",
            })
    }

    fn base_vars(&self) -> Vars {
        self.vars_for(self.session, self.session.active, None)
    }

    fn vars_for(&self, session: &Session, window_index: usize, pane_index: Option<usize>) -> Vars {
        let resolved_pane = session
            .windows
            .get(window_index)
            .map(|link| self.state.window_for_link(link))
            .map(|window| pane_index.unwrap_or(window.active))
            .unwrap_or(0);
        let mut vars = super::command::vars_full(
            self.state,
            session,
            window_index,
            resolved_pane,
            self.agents,
            self.state.marked_pane(),
        );

        vars.set("client_width", self.cols.to_string())
            .set("client_height", self.rows.to_string())
            .set("client_session", session.name.clone())
            .set("client_name", self.client.tty.clone().unwrap_or_default())
            .set("client_tty", self.client.tty.clone().unwrap_or_default())
            .set(
                "client_termname",
                self.client.term.clone().unwrap_or_default(),
            )
            .set(
                "client_termtype",
                self.client.term.clone().unwrap_or_default(),
            )
            .set(
                "client_pid",
                self.client.pid.unwrap_or_default().to_string(),
            )
            .set("client_utf8", "1")
            .set(
                "client_control_mode",
                if self.client.control_mode { "1" } else { "0" },
            )
            .set(
                "client_readonly",
                if self.client.read_only { "1" } else { "0" },
            )
            .set("client_flags", self.client.flags.clone())
            .set("client_key_table", self.client.key_table.clone())
            // tmux reports a prefix as "the client left its default table",
            // which covers a live `bind -r` repeat chain as well.
            .set(
                "client_prefix",
                if self.client.key_table == self.state.session_key_table(&session.name) {
                    "0"
                } else {
                    "1"
                },
            )
            .set("window_bigger", "0")
            .set("window_offset_x", "0")
            .set("window_offset_y", "0");

        if let Some(link) = session.windows.get(window_index) {
            let window = self.state.window_for_link(link);
            vars.set(
                "pane_flags",
                if resolved_pane == window.active {
                    "*"
                } else {
                    ""
                },
            );
            let option_target = format!("${}:{}", session.id, link.index);
            if let Ok(entries) = self.state.format_option_entries(&option_target) {
                for (name, value) in entries {
                    vars.set(name, value);
                }
            }
        }
        for (name, value) in self.state.env_iter() {
            if vars.lookup(name).is_none() {
                vars.set(name, value);
            }
        }
        vars
    }

    fn default_first_line(&self) -> String {
        let vars = self.base_vars();
        let left_style = self.expand(self.option("status-left-style"), &vars, 0);
        let right_style = self.expand(self.option("status-right-style"), &vars, 0);
        let left_limit = self
            .option("status-left-length")
            .parse::<usize>()
            .unwrap_or(10);
        let right_limit = self
            .option("status-right-length")
            .parse::<usize>()
            .unwrap_or(40);
        let left = format::trim_left(
            &self.expand_time(self.option("status-left"), &vars),
            left_limit,
        );
        let right = format::trim_left(
            &self.expand_time(self.option("status-right"), &vars),
            right_limit,
        );
        let mut out = format!(
            "#[align=left range=left {left_style}]#[push-default]{left}#[pop-default]#[norange default]"
        );
        let _ = write!(
            out,
            "#[list=on align={}]#[list=left-marker]<#[list=right-marker]>#[list=on]",
            self.option("status-justify")
        );
        for (index, window) in self.session.windows.iter().enumerate() {
            let item = self.window_item(window, index);
            out.push_str(&item);
            if index + 1 != self.session.windows.len() {
                let item_vars = self.vars_for(self.session, index, None);
                out.push_str(&self.expand(self.option("window-status-separator"), &item_vars, 0));
            }
        }
        let _ = write!(
            out,
            "#[nolist align=right range=right {right_style}]#[push-default]{right}#[pop-default]#[norange default]"
        );
        out
    }

    fn window_item(&self, window: &Winlink, index: usize) -> String {
        let vars = self.vars_for(self.session, index, None);
        let active = index == self.session.active;
        let option = if active {
            "window-status-current-format"
        } else {
            "window-status-format"
        };
        let style = if active {
            self.option("window-status-current-style")
        } else {
            self.option("window-status-style")
        };
        let mut style = self.expand(style, &vars, 0);
        if self.session.last_active == Some(index) {
            let last = self.expand(self.option("window-status-last-style"), &vars, 0);
            if last != "default" && !last.is_empty() {
                if !style.is_empty() {
                    style.push(',');
                }
                style.push_str(&last);
            }
        }
        let text = self.expand_time(self.option(option), &vars);
        format!(
            "#[range=window|{} {}{}]#[push-default]{}#[pop-default]#[norange list=on default]",
            window.index,
            if active { "list=focus " } else { "" },
            style,
            text
        )
    }

    fn default_pane_line(&self) -> String {
        let width = self.session.name.chars().count();
        let mut out = format!(
            "#[align=left]{}P: #[list=on align={}]",
            " ".repeat(width),
            self.option("status-justify")
        );
        if let Some(link) = self.session.windows.get(self.session.active) {
            let window = self.state.window_for_link(link);
            for (index, pane) in window.panes.iter().enumerate() {
                let vars = self.vars_for(self.session, self.session.active, Some(index));
                let active = index == window.active;
                let style = if active {
                    self.option("pane-status-current-style")
                } else {
                    self.option("pane-status-style")
                };
                let fmt = if active {
                    self.option("window-pane-current-status-format")
                } else {
                    self.option("window-pane-status-format")
                };
                let _ = write!(
                    out,
                    "#[range=pane|%{} {}{}]{}{}",
                    pane.id,
                    if active { "list=focus " } else { "" },
                    self.expand(style, &vars, 0),
                    self.expand_time(fmt, &vars),
                    if active { " " } else { "  " }
                );
            }
        }
        out
    }

    fn default_session_line(&self) -> String {
        let width = self.session.name.chars().count();
        let mut out = format!(
            "#[align=left]{}S: #[list=on align={}]",
            " ".repeat(width),
            self.option("status-justify")
        );
        for (index, session) in self.state.sessions().iter().enumerate() {
            let active = session.id == self.session.id;
            let vars = self.vars_for(session, session.active, None);
            let style = if active {
                self.option("session-status-current-style")
            } else {
                self.option("session-status-style")
            };
            let _ = write!(
                out,
                "#[range=session|${} {}{}]{}{}",
                session.id,
                if active { "list=focus " } else { "" },
                self.expand(style, &vars, 0),
                if active {
                    format!("{}*", session.name)
                } else {
                    session.name.clone()
                },
                if active { " " } else { "  " }
            );
            if index + 1 == self.state.sessions().len() {
                out.push_str("#[norange]");
            }
        }
        out
    }

    fn expand(&self, template: &str, vars: &Vars, depth: usize) -> String {
        let _ = depth;
        format::expand_with_context(template, vars, self)
    }

    fn expand_time(&self, template: &str, vars: &Vars) -> String {
        format::expand_time_with_context(template, vars, self)
    }
}

impl format::FormatContext for StatusContext<'_> {
    fn lookup(&self, vars: &Vars, key: &str) -> Option<String> {
        vars.lookup(key).map(str::to_string).or_else(|| {
            (!key.contains([':', ';']))
                .then(|| {
                    self.state
                        .option_for_target(&self.session.name, key)
                        .or_else(|| super::options::option_default(key))
                        .map(str::to_string)
                })
                .flatten()
        })
    }

    fn loop_items(
        &self,
        kind: format::FormatLoopKind,
        flags: &str,
        _vars: &Vars,
    ) -> Option<Vec<format::FormatLoopItem>> {
        let mut items = match kind {
            format::FormatLoopKind::Session => {
                let mut sessions = self.state.sessions().iter().collect::<Vec<_>>();
                if flags.contains('n') {
                    sessions.sort_by(|left, right| left.name.cmp(&right.name));
                } else {
                    sessions.sort_by_key(|session| session.id);
                }
                sessions
                    .into_iter()
                    .map(|session| format::FormatLoopItem {
                        vars: self.vars_for(session, session.active, None),
                        active: session.id == self.session.id,
                    })
                    .collect()
            }
            format::FormatLoopKind::Window => {
                let mut indices = (0..self.session.windows.len()).collect::<Vec<_>>();
                if flags.contains('n') {
                    indices.sort_by(|left, right| {
                        self.state
                            .window_for_link(&self.session.windows[*left])
                            .name
                            .cmp(
                                &self
                                    .state
                                    .window_for_link(&self.session.windows[*right])
                                    .name,
                            )
                    });
                } else {
                    indices.sort_by_key(|index| self.session.windows[*index].index);
                }
                indices
                    .into_iter()
                    .map(|index| format::FormatLoopItem {
                        vars: self.vars_for(self.session, index, None),
                        active: index == self.session.active,
                    })
                    .collect()
            }
            format::FormatLoopKind::Pane => {
                let link = self.session.windows.get(self.session.active)?;
                let window = self.state.window_for_link(link);
                (0..window.panes.len())
                    .map(|index| format::FormatLoopItem {
                        vars: self.vars_for(self.session, self.session.active, Some(index)),
                        active: index == window.active,
                    })
                    .collect()
            }
            format::FormatLoopKind::Client => vec![format::FormatLoopItem {
                vars: self.base_vars(),
                active: true,
            }],
        };
        if flags.contains('r') {
            items.reverse();
        }
        Some(items)
    }

    fn search_pane(&self, vars: &Vars, term: &str, regex: bool, ignore_case: bool) -> u32 {
        format::FormatTree::search_pane(
            &super::command::ServerFormatTree(self.state),
            vars,
            term,
            regex,
            ignore_case,
        )
    }

    fn name_exists(&self, vars: &Vars, scope: format::NameScope, name: &str) -> bool {
        format::FormatTree::name_exists(
            &super::command::ServerFormatTree(self.state),
            vars,
            scope,
            name,
        )
    }

    fn job(&self, command: &str, expanded: String, vars: &Vars) -> String {
        let Some(jobs) = self.jobs else {
            return String::new();
        };
        jobs.output_for(
            command,
            expanded,
            vars,
            self.session.id,
            job_cwd(Some(self.session), self.client.cwd.as_deref()),
            // tmux's `job_run` builds the job's environment with
            // `environ_for_session`, which gives a job no view of the client.
            self.state.job_environment(Some(&self.session.name)),
            self.job_status,
        )
    }

    fn preserve_double_hash(&self) -> bool {
        true
    }
}

fn draw_row(expanded: &str, cols: usize, base: &CellStyle) -> StatusRow {
    let mut sections = Sections::default();
    let mut state = DrawState::new(*base);
    let mut bytes = expanded.as_bytes();
    while !bytes.is_empty() {
        if !state.ignore && bytes.starts_with(b"#[") {
            if let Some(end) = find_style_close(bytes) {
                let directive = std::str::from_utf8(&bytes[2..end]).unwrap_or_default();
                apply_status_style(directive, &mut state, &mut sections);
                bytes = &bytes[end + 1..];
                continue;
            }
        }
        if state.ignore && bytes.starts_with(b"#[") {
            push_cell("#", 1, &state, &mut sections);
            bytes = &bytes[1..];
            continue;
        }
        if bytes[0] == b'#' {
            let count = bytes.iter().take_while(|&&byte| byte == b'#').count();
            if bytes.get(count) == Some(&b'[') {
                let next = if count % 2 == 0 {
                    &bytes[count + 1..]
                } else {
                    &bytes[count - 1..]
                };
                if state.ignore {
                    bytes = next;
                    continue;
                }
                for _ in 0..count / 2 {
                    push_cell("#", 1, &state, &mut sections);
                }
                if count % 2 == 0 {
                    push_cell("[", 1, &state, &mut sections);
                }
                bytes = next;
                continue;
            }
            for _ in 0..count.div_ceil(2) {
                push_cell("#", 1, &state, &mut sections);
            }
            bytes = &bytes[count..];
            continue;
        }
        let text = std::str::from_utf8(bytes).unwrap_or_default();
        let Some(ch) = text.chars().next() else { break };
        bytes = &bytes[ch.len_utf8()..];
        if ch.is_control() {
            continue;
        }
        let width = ghostty_sys::codepoint_width(ch as u32) as u8;
        if width == 0 {
            if let Some(cell) = sections.cells_mut(state.section).last_mut() {
                cell.text.push(ch);
            }
        } else {
            let mut encoded = [0; 4];
            push_cell(ch.encode_utf8(&mut encoded), width, &state, &mut sections);
        }
    }

    // A range the format never closed is discarded, exactly as tmux frees the
    // in-flight `format_range` when it runs out of directives.
    sections.ranges.retain(|range| !range.open);

    let mut fill_style = *base;
    if let Some(fill) = state.fill {
        fill_style.bg = fill;
    }
    let mut row = blank_row(cols, &fill_style);
    if state.fill.is_some() {
        row.used = cols;
    }
    layout_sections(&sections, &state, &mut row);
    row
}

fn push_cell(text: &str, width: u8, state: &DrawState, sections: &mut Sections) {
    let section = state.section;
    let start = sections.width(section);
    sections.cells_mut(section).push(StatusCell {
        text: text.to_string(),
        width,
        style: state.style,
        acs: state.acs,
    });
    if let Some(kind) = state.range.clone() {
        let end = start + usize::from(width);
        if let Some(range) = sections.ranges.last_mut().filter(|range| {
            range.open && range.section == section && range.end == start && range.kind == kind
        }) {
            range.end = end;
        } else {
            close_open_range(sections);
            sections.ranges.push(PendingRange {
                section,
                start,
                end,
                kind,
                open: true,
            });
        }
    }
}

/// End the range currently being drawn, extending it by tmux's trailing column.
fn close_open_range(sections: &mut Sections) {
    if let Some(range) = sections.ranges.last_mut().filter(|range| range.open) {
        range.open = false;
        range.end += 1;
    }
}

fn find_style_close(bytes: &[u8]) -> Option<usize> {
    bytes.iter().position(|&byte| byte == b']')
}

fn apply_status_style(directive: &str, state: &mut DrawState, sections: &mut Sections) {
    let original_state = state.clone();
    let original_sections = sections.clone();
    let saved = state.style;
    let saved_acs = state.acs;
    let mut default_action = None;
    let mut invalid = false;
    for part in style::split_style_parts(directive) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        match style::apply_visual_token(part, &mut state.style, &state.current_default) {
            Ok(VisualToken::Applied) => {
                if part == "default" {
                    state.acs = state.current_default_acs;
                } else if part == "none" {
                    state.acs = false;
                }
                continue;
            }
            Err(()) => {
                invalid = true;
                break;
            }
            Ok(VisualToken::NotVisual) => {}
        }
        match part {
            "push-default" => default_action = Some(true),
            "pop-default" => default_action = Some(false),
            "set-default" => {
                state.base = saved;
                state.current_default = saved;
                state.base_acs = saved_acs;
                state.current_default_acs = saved_acs;
            }
            "nolist" | "list=off" => {
                if state.list_active {
                    state.list_active = false;
                    if state.focus_start.is_some() && state.focus_end.is_none() {
                        state.focus_end = Some(sections.width(Section::List));
                    }
                }
                state.section = section_for_align(parse_align_from(directive));
            }
            "list=on" => {
                state.list_active = true;
                state.section = Section::List;
                if state.list_align.is_none() {
                    state.list_align = Some(parse_align_from(directive));
                }
                if state.focus_start.is_some() && state.focus_end.is_none() {
                    state.focus_end = Some(sections.width(Section::List));
                }
            }
            "list=focus" => {
                if state.list_active && state.focus_start.is_none() {
                    state.focus_start = Some(sections.width(Section::List));
                }
            }
            "list=left-marker" => state.section = Section::ListLeft,
            "list=right-marker" => state.section = Section::ListRight,
            "norange" | "range=none" => state.range = None,
            "acs" => state.acs = true,
            "noacs" => state.acs = false,
            "ignore" => state.ignore = true,
            "noignore" => state.ignore = false,
            "noalign" => state.section = Section::Left,
            _ => {
                if let Some(value) = part.strip_prefix("fill=") {
                    if let Some(colour) = style::parse_colour(value) {
                        state.fill = Some(colour);
                    } else {
                        invalid = true;
                    }
                } else if let Some(value) = part.strip_prefix("align=") {
                    if matches!(
                        value,
                        "left"
                            | "centre"
                            | "center"
                            | "right"
                            | "absolute-centre"
                            | "absolute-center"
                    ) {
                        if !state.list_active {
                            state.section = section_for_align(parse_align(value));
                        }
                    } else {
                        invalid = true;
                    }
                } else if let Some(value) = part.strip_prefix("range=") {
                    match parse_range(value) {
                        Some(range) => state.range = Some(range),
                        None => invalid = true,
                    }
                } else if let Some(value) = part.strip_prefix("width=") {
                    invalid = !valid_style_width(value);
                } else if let Some(value) = part.strip_prefix("pad=") {
                    invalid = parse_style_uint(value).is_none();
                } else {
                    invalid = true;
                }
            }
        }
        if invalid {
            break;
        }
    }
    if invalid {
        *state = original_state;
        *sections = original_sections;
        return;
    }
    match default_action {
        Some(true) => {
            state.current_default = saved;
            state.current_default_acs = saved_acs;
        }
        Some(false) => {
            state.current_default = state.base;
            state.current_default_acs = state.base_acs;
        }
        None => {}
    }
    // tmux ends a range as soon as the style stops naming it, and that closing
    // position is what gives the range its trailing column.
    if state.range != original_state.range {
        close_open_range(sections);
    }
}

fn valid_style_width(value: &str) -> bool {
    if let Some(percentage) = value.strip_suffix('%') {
        return parse_style_uint(percentage).is_some_and(|percentage| percentage <= 100);
    }
    parse_style_uint(value).is_some()
}

fn parse_style_uint(value: &str) -> Option<u32> {
    let value = value.parse::<i128>().ok()?;
    u32::try_from(value).ok()
}

fn parse_align_from(directive: &str) -> Align {
    style::split_style_parts(directive)
        .find_map(|part| part.strip_prefix("align=").map(parse_align))
        .unwrap_or(Align::Left)
}

fn parse_align(value: &str) -> Align {
    match value {
        "centre" | "center" => Align::Centre,
        "right" => Align::Right,
        "absolute-centre" | "absolute-center" => Align::AbsoluteCentre,
        _ => Align::Left,
    }
}

fn section_for_align(align: Align) -> Section {
    match align {
        Align::Left => Section::Left,
        Align::Centre => Section::Centre,
        Align::Right => Section::Right,
        Align::AbsoluteCentre => Section::AbsoluteCentre,
    }
}

fn parse_range(value: &str) -> Option<StatusRangeKind> {
    if value == "left" {
        return Some(StatusRangeKind::Left);
    }
    if value == "right" {
        return Some(StatusRangeKind::Right);
    }
    let (kind, argument) = value.split_once('|')?;
    if kind == "user" && !argument.is_empty() {
        return Some(StatusRangeKind::User(argument.to_string()));
    }
    let argument = argument
        .trim_start_matches(['@', '%', '$'])
        .parse::<u32>()
        .ok()?;
    match kind {
        "window" => Some(StatusRangeKind::Window(argument)),
        "pane" => Some(StatusRangeKind::Pane(argument)),
        "session" => Some(StatusRangeKind::Session(argument)),
        "control" if argument <= 9 => Some(StatusRangeKind::Control(argument)),
        _ => None,
    }
}

fn layout_sections(sections: &Sections, state: &DrawState, row: &mut StatusRow) {
    let available = row.columns;
    let mut left = sections.width(Section::Left);
    let mut centre = sections.width(Section::Centre);
    let mut right = sections.width(Section::Right);
    let mut list = sections.width(Section::List);
    let mut after = sections.width(Section::After);

    match state.list_align {
        None => {
            trim_widths(available, [&mut centre, &mut right, &mut left]);
            put_section(row, sections, Section::Left, 0, 0, left);
            put_section(
                row,
                sections,
                Section::Right,
                available.saturating_sub(right),
                sections.width(Section::Right).saturating_sub(right),
                right,
            );
            let gap_start = left;
            let gap_end = available.saturating_sub(right);
            let target = gap_start
                .saturating_add(gap_end.saturating_sub(gap_start) / 2)
                .saturating_sub(centre / 2);
            put_section(
                row,
                sections,
                Section::Centre,
                target,
                sections.width(Section::Centre).saturating_sub(centre) / 2,
                centre,
            );
        }
        Some(Align::Left) => {
            trim_widths(
                available,
                [&mut centre, &mut list, &mut right, &mut after, &mut left],
            );
            put_section(row, sections, Section::Left, 0, 0, left);
            put_list(row, sections, state, left, list);
            put_section(row, sections, Section::After, left + list, 0, after);
            put_section(
                row,
                sections,
                Section::Right,
                available.saturating_sub(right),
                sections.width(Section::Right).saturating_sub(right),
                right,
            );
            let used_left = left + list + after;
            let target = used_left + available.saturating_sub(right + used_left) / 2;
            put_section(
                row,
                sections,
                Section::Centre,
                target.saturating_sub(centre / 2),
                sections.width(Section::Centre).saturating_sub(centre) / 2,
                centre,
            );
        }
        Some(Align::Right) => {
            trim_widths(
                available,
                [&mut centre, &mut list, &mut right, &mut after, &mut left],
            );
            put_section(row, sections, Section::Left, 0, 0, left);
            put_section(
                row,
                sections,
                Section::After,
                available.saturating_sub(after),
                sections.width(Section::After).saturating_sub(after),
                after,
            );
            let list_at = available.saturating_sub(after + list);
            put_list(row, sections, state, list_at, list);
            let right_at = list_at.saturating_sub(right);
            put_section(row, sections, Section::Right, right_at, 0, right);
            let target = left + right_at.saturating_sub(left) / 2;
            put_section(
                row,
                sections,
                Section::Centre,
                target.saturating_sub(centre / 2),
                sections.width(Section::Centre).saturating_sub(centre) / 2,
                centre,
            );
        }
        Some(Align::Centre) => {
            trim_widths(
                available,
                [&mut list, &mut after, &mut centre, &mut right, &mut left],
            );
            put_section(row, sections, Section::Left, 0, 0, left);
            put_section(
                row,
                sections,
                Section::Right,
                available.saturating_sub(right),
                sections.width(Section::Right).saturating_sub(right),
                right,
            );
            let middle = left + available.saturating_sub(left + right) / 2;
            let list_at = middle.saturating_sub(list / 2);
            put_section(
                row,
                sections,
                Section::Centre,
                list_at.saturating_sub(centre),
                0,
                centre,
            );
            put_list(row, sections, state, list_at, list);
            put_section(row, sections, Section::After, list_at + list, 0, after);
        }
        Some(Align::AbsoluteCentre) => {
            trim_widths(available, [&mut centre, &mut right, &mut left]);
            trim_widths(available, [&mut list, &mut after]);
            put_section(row, sections, Section::Left, 0, 0, left);
            put_section(
                row,
                sections,
                Section::Right,
                available.saturating_sub(right),
                sections.width(Section::Right).saturating_sub(right),
                right,
            );
            let middle = left + available.saturating_sub(left + right) / 2;
            put_section(
                row,
                sections,
                Section::Centre,
                middle.saturating_sub(centre),
                0,
                centre,
            );
            let list_at = available.saturating_sub(list) / 2;
            put_list(row, sections, state, list_at, list);
            put_section(row, sections, Section::After, list_at + list, 0, after);
        }
    }
    let absolute = sections.width(Section::AbsoluteCentre).min(available);
    put_section(
        row,
        sections,
        Section::AbsoluteCentre,
        (available - absolute) / 2,
        0,
        absolute,
    );
}

fn trim_widths<const N: usize>(available: usize, mut order: [&mut usize; N]) {
    while order.iter().map(|width| **width).sum::<usize>() > available {
        if let Some(width) = order.iter_mut().find(|width| ***width > 0) {
            **width -= 1;
        } else {
            break;
        }
    }
}

fn put_list(row: &mut StatusRow, sections: &Sections, state: &DrawState, at: usize, width: usize) {
    let full = sections.width(Section::List);
    if width >= full {
        put_section(row, sections, Section::List, at, 0, width);
        return;
    }
    let focus_start = state.focus_start.unwrap_or(0);
    let focus_end = state.focus_end.unwrap_or(focus_start);
    let focus = (focus_start + focus_end) / 2;
    let mut start = focus
        .saturating_sub(width / 2)
        .min(full.saturating_sub(width));
    let mut target = at;
    let mut body_width = width;
    if start > 0 && !sections.list_left.is_empty() {
        let marker = sections.width(Section::ListLeft).min(body_width);
        put_section(row, sections, Section::ListLeft, target, 0, marker);
        target += marker;
        start += marker;
        body_width -= marker;
    }
    let right_hidden = start + body_width < full;
    if right_hidden && !sections.list_right.is_empty() {
        let marker = sections.width(Section::ListRight).min(body_width);
        body_width -= marker;
        put_section(
            row,
            sections,
            Section::ListRight,
            target + body_width,
            0,
            marker,
        );
    }
    start = start.min(full.saturating_sub(body_width));
    put_section(row, sections, Section::List, target, start, body_width);
}

fn put_section(
    row: &mut StatusRow,
    sections: &Sections,
    section: Section,
    target: usize,
    source: usize,
    width: usize,
) {
    if width == 0 || target >= row.columns {
        return;
    }
    let selected = slice_cells(sections.cells(section), source, width);
    let mut column = target;
    for cell in selected {
        if column + usize::from(cell.width) > row.columns {
            break;
        }
        replace_cell_at(&mut row.cells, column, cell.clone());
        column += usize::from(cell.width);
    }
    row.used = row.used.max(column);
    for range in sections
        .ranges
        .iter()
        .filter(|range| range.section == section)
    {
        let visible_start = range.start.max(source);
        let visible_end = range.end.min(source + width);
        if visible_start < visible_end {
            row.ranges.push(StatusRange {
                start: (target + visible_start - source) as u16,
                end: (target + visible_end - source) as u16,
                kind: range.kind.clone(),
            });
        }
    }
}

fn slice_cells(cells: &[StatusCell], source: usize, width: usize) -> Vec<&StatusCell> {
    let mut result = Vec::new();
    let mut column = 0;
    for cell in cells {
        let end = column + usize::from(cell.width);
        if column >= source && end <= source + width {
            result.push(cell);
        }
        column = end;
    }
    result
}

fn replace_cell_at(cells: &mut Vec<StatusCell>, column: usize, replacement: StatusCell) {
    let mut current = 0;
    let mut index = 0;
    while index < cells.len() && current < column {
        current += usize::from(cells[index].width);
        index += 1;
    }
    if current != column || index >= cells.len() {
        return;
    }
    let width = usize::from(replacement.width);
    let mut removed = 0;
    let start = index;
    while index < cells.len() && removed < width {
        removed += usize::from(cells[index].width);
        index += 1;
    }
    cells.splice(start..index, [replacement]);
    if removed > width {
        let style = cells.get(start).map(|cell| cell.style).unwrap_or_default();
        let acs = cells.get(start).is_some_and(|cell| cell.acs);
        cells.insert(
            start + 1,
            StatusCell {
                text: " ".into(),
                width: (removed - width) as u8,
                style,
                acs,
            },
        );
    }
}

fn cells_width(cells: &[StatusCell]) -> usize {
    cells.iter().map(|cell| usize::from(cell.width)).sum()
}

fn parse_status_cell_style(
    value: &str,
    base: &CellStyle,
    current_default: &CellStyle,
) -> CellStyle {
    let mut state = DrawState::new(*base);
    state.current_default = *current_default;
    apply_status_style(value, &mut state, &mut Sections::default());
    state.style
}

#[cfg(test)]
fn serialize_row(row: &StatusRow) -> Vec<u8> {
    serialize_row_width(row, cells_width(&row.cells))
}

#[cfg(test)]
fn serialize_row_width(row: &StatusRow, width: usize) -> Vec<u8> {
    let mut out = Vec::new();
    let mut writer = CaptureStyleWriter::default();
    let mut column = 0;
    for cell in &row.cells {
        if column + usize::from(cell.width) > width {
            break;
        }
        writer.transition(
            &mut out,
            &CellPresentation {
                style: cell.style,
                acs: cell.acs,
                ..CellPresentation::default()
            },
        );
        out.extend_from_slice(cell.text.as_bytes());
        column += usize::from(cell.width);
    }
    out
}

fn write_terminal_cell_text(
    out: &mut Vec<u8>,
    cell: &StatusCell,
    terminal: &dyn TerminalCapabilities,
) {
    if !cell.acs {
        out.extend_from_slice(cell.text.as_bytes());
        return;
    }
    if terminal_utf8(terminal) {
        for byte in cell.text.bytes() {
            if let Some(mapped) = utf8_acs(byte) {
                out.extend_from_slice(mapped);
            } else {
                out.push(byte);
            }
        }
    } else {
        out.extend(
            cell.text
                .bytes()
                .map(|byte| terminal_acs(terminal, byte).unwrap_or(byte)),
        );
    }
}

fn utf8_acs(byte: u8) -> Option<&'static [u8]> {
    Some(match byte {
        b'+' => "→".as_bytes(),
        b',' => "←".as_bytes(),
        b'-' => "↑".as_bytes(),
        b'.' => "↓".as_bytes(),
        b'0' => "▮".as_bytes(),
        b'`' => "◆".as_bytes(),
        b'a' => "▒".as_bytes(),
        b'b' => "␉".as_bytes(),
        b'c' => "␌".as_bytes(),
        b'd' => "␍".as_bytes(),
        b'e' => "␊".as_bytes(),
        b'f' => "°".as_bytes(),
        b'g' => "±".as_bytes(),
        b'h' => "␤".as_bytes(),
        b'i' => "␋".as_bytes(),
        b'j' => "┘".as_bytes(),
        b'k' => "┐".as_bytes(),
        b'l' => "┌".as_bytes(),
        b'm' => "└".as_bytes(),
        b'n' => "┼".as_bytes(),
        b'o' => "⎺".as_bytes(),
        b'p' => "⎻".as_bytes(),
        b'q' => "─".as_bytes(),
        b'r' => "⎼".as_bytes(),
        b's' => "⎽".as_bytes(),
        b't' => "├".as_bytes(),
        b'u' => "┤".as_bytes(),
        b'v' => "┴".as_bytes(),
        b'w' => "┬".as_bytes(),
        b'x' => "│".as_bytes(),
        b'y' => "≤".as_bytes(),
        b'z' => "≥".as_bytes(),
        b'{' => "π".as_bytes(),
        b'|' => "≠".as_bytes(),
        b'}' => "£".as_bytes(),
        b'~' => "·".as_bytes(),
        _ => return None,
    })
}

fn serialize_row_width_for_terminal(
    row: &StatusRow,
    width: usize,
    terminal: &dyn TerminalCapabilities,
) -> Vec<u8> {
    let mut out = Vec::new();
    let mut writer = TerminalStyleWriter::new(terminal);
    let mut column = 0;
    for cell in &row.cells {
        if column + usize::from(cell.width) > width {
            break;
        }
        writer.transition(
            &mut out,
            &CellPresentation {
                style: cell.style,
                acs: cell.acs,
                ..CellPresentation::default()
            },
        );
        write_terminal_cell_text(&mut out, cell, terminal);
        column += usize::from(cell.width);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::term::{ResolvedTerm, TerminalIdentity};

    #[test]
    fn status_conditionals_resolve_bare_variables_and_escaped_commas() {
        let state = ServerState::with_test_session().expect("state");
        let session = state.find("0").expect("session");
        let client = ClientContext::default();
        let agents = PaneAgents::new();
        let context = StatusContext::new(&state, session, &client, 120, 24, None, true, &agents);
        let mut vars = context.base_vars();
        vars.set("flag", "0").set("x", "1").set("y", "2");

        let template = "#{?flag,[#{x}#,#{y}],fallback}";
        assert_eq!(context.expand(template, &vars, 0), "fallback");
        vars.set("flag", "1");
        assert_eq!(context.expand(template, &vars, 0), "[1,2]");

        let default_right = context.expand(DEFAULT_STATUS_RIGHT, &context.base_vars(), 0);
        assert!(default_right.starts_with('"'), "{default_right:?}");
        assert!(!default_right.contains("[0#"), "{default_right:?}");
    }

    #[test]
    fn status_and_command_formats_share_core_evaluator_semantics() {
        let state = ServerState::with_test_session().expect("state");
        let session = state.find("0").expect("session");
        let client = ClientContext::default();
        let agents = PaneAgents::new();
        let context = StatusContext::new(&state, session, &client, 120, 24, None, true, &agents);
        let mut vars = context.base_vars();
        vars.set("flag", "1").set("left", "abc").set("right", "abc");

        for template in [
            "#{?flag,#{left}#,#{right},no}",
            "#{==:#{left},#{right}}",
            "#{&&:#{flag},#{==:#{left},abc}}",
            "#{=2:left}",
            "#{q:left}",
        ] {
            assert_eq!(
                context.expand(template, &vars, 0),
                format::expand(template, &vars),
                "different expansion for {template:?}"
            );
        }
    }

    #[test]
    fn encoded_status_cache_follows_terminal_profile_generation() {
        let mut state = ServerState::with_test_session().expect("state");
        state.set_global_option("status-style", "fg=red,bg=default");
        state.set_global_option("status-format[0]", "x");
        let rendered = render_status(
            &state,
            "0",
            2,
            24,
            &ClientContext::default(),
            None,
            &PaneAgents::new(),
        );
        let mut terminal = ResolvedTerm::resolve(
            TerminalIdentity::new(
                "style-test",
                vec![
                    "colors=8".into(),
                    "setaf=\x1b[FG%p1%d".into(),
                    "sgr0=\x1b[RESET".into(),
                ],
                0,
                None,
            ),
            [],
        );

        let mut first = Vec::new();
        rendered.append_row_for_terminal(&mut first, 0, false, &terminal);
        assert!(first.windows(3).any(|bytes| bytes == b"FG1"));

        terminal.refresh([("terminal-overrides", "style-test:setaf@")]);
        let mut second = Vec::new();
        rendered.append_row_for_terminal(&mut second, 0, false, &terminal);
        assert!(!second.windows(3).any(|bytes| bytes == b"FG1"));
    }

    #[test]
    fn attached_renderers_own_distinct_format_job_caches() {
        // Any render creates the cache's job tree, so this needs no `#()` in
        // the status format — and must not have one: starting a real job would
        // fork a shell, which a unit test is not allowed to do.
        let state = ServerState::with_test_session().expect("state");
        let mut first = RenderCache::default();
        let mut second = RenderCache::default();

        let _ = first.render(&state, "0", 20, 24);
        let _ = second.render(&state, "0", 20, 24);

        let first_jobs = first.format_jobs.as_ref().expect("first client jobs");
        let second_jobs = second.format_jobs.as_ref().expect("second client jobs");
        assert!(!Arc::ptr_eq(first_jobs, second_jobs));
        assert!(!Arc::ptr_eq(first_jobs, &state.format_job_registry()));
    }

    #[test]
    fn no_client_formats_share_the_global_format_job_cache() {
        // That the shared tree actually caches an entry across command clients
        // needs a job to run, so it is pinned by
        // `hmux-conformance/tests/format_job_cache_scope.rs` instead.
        let state = ServerState::with_test_session().expect("state");
        let first = state.format_job_registry();
        let second = state.format_job_registry();
        assert!(Arc::ptr_eq(&first, &second));
    }

    /// A cache entry with a synthetic age, so eviction can be observed without
    /// running a job or waiting for a timeout to elapse.
    fn finished_job_entry(last_started: Instant, output: &str) -> FormatJobEntry {
        FormatJobEntry {
            expanded_command: "printf cached".to_string(),
            output: output.to_string(),
            running: false,
            generation: 1,
            last_started,
            last_started_second: 0,
            last_notified: last_started,
            process: None,
        }
    }

    fn job_key(command: &str) -> FormatJobKey {
        FormatJobKey {
            scope: "global".to_string(),
            command: command.to_string(),
        }
    }

    #[test]
    fn finished_format_jobs_are_evicted_once_their_timeout_has_passed() {
        // `output` reads its cached entry through this same pass, so an evicted
        // entry is what makes the next expansion start the job again.
        let timeout = Duration::from_secs(3600);
        let now = Instant::now();
        let mut jobs = HashMap::new();
        jobs.insert(
            job_key("printf fresh"),
            finished_job_entry(now - timeout + Duration::from_secs(1), "fresh"),
        );
        jobs.insert(
            job_key("printf stale"),
            finished_job_entry(now - timeout, "stale"),
        );

        FormatJobRegistry::evict_expired(&mut jobs, timeout, now);

        assert!(jobs.contains_key(&job_key("printf fresh")));
        assert!(
            !jobs.contains_key(&job_key("printf stale")),
            "an entry untouched for the whole timeout must start fresh"
        );
    }

    #[test]
    fn a_running_format_job_survives_its_eviction_timeout() {
        // The job's thread still holds this key's generation; evicting it would
        // leave that thread updating an entry that no longer exists.
        let timeout = Duration::from_millis(200);
        let now = Instant::now();
        let mut jobs = HashMap::new();
        let mut entry = finished_job_entry(now - timeout - Duration::from_secs(1), "");
        entry.running = true;
        jobs.insert(job_key("printf slow"), entry);

        FormatJobRegistry::evict_expired(&mut jobs, timeout, now);

        assert!(jobs.contains_key(&job_key("printf slow")));
    }

    #[test]
    fn height_and_interval_follow_status_options() {
        let mut state = ServerState::with_test_session().expect("state");
        assert_eq!(height(&state, "0"), 1);
        assert_eq!(interval(&state, "0"), Some(Duration::from_secs(15)));
        state.set_global_option("status", "3");
        assert_eq!(height(&state, "0"), 3);
        state.set_global_option("status-interval", "0");
        assert_eq!(interval(&state, "0"), None);
        state.set_global_option("status", "off");
        assert_eq!(height(&state, "0"), 0);
    }

    #[test]
    fn renderer_keeps_distinct_status_rows() {
        let mut state = ServerState::with_test_session().expect("state");
        state.set_global_option("status", "2");
        state.set_global_option("status-style", "fg=default,bg=default");
        state.set_global_option("status-format[0]", "ZERO");
        state.set_global_option("status-format[1]", "ONE");
        let mut renderer = RenderCache::default();
        let rendered = renderer.render(&state, "0", 12, 24);
        assert!(String::from_utf8_lossy(rendered.row(0)).starts_with("ZERO"));
        assert!(String::from_utf8_lossy(rendered.row(1)).starts_with("ONE"));
        assert_ne!(rendered.row(0), rendered.row(1));
    }

    #[test]
    fn explicit_status_format_expands_strftime_sequences() {
        let mut state = ServerState::with_test_session().expect("state");
        state.set_global_option("status-style", "fg=default,bg=default");
        state.set_global_option("status-format[0]", "%Y");
        let mut renderer = RenderCache::default();
        let row = String::from_utf8_lossy(renderer.render(&state, "0", 8, 24).row(0));
        assert!(
            row[..4].bytes().all(|byte| byte.is_ascii_digit()),
            "{row:?}"
        );
    }

    #[test]
    fn default_window_label_shows_agent_state_and_worktree_name() {
        use crate::integration::status::{AgentStatus, StatusSnapshot};
        use crate::integration::AgentState;
        use crate::observability::v1::PaneId;

        let mut state = ServerState::with_test_session().expect("state");
        state.rename_window("0", "shell").expect("rename window");
        let session = state.find("0").expect("session");
        let window = state.window_for_link(&session.windows[session.active]);
        let pane_id = PaneId(window.panes[window.active].id);
        let window_name = window.name.clone();
        let worktree_name = std::env::current_dir()
            .expect("current directory")
            .file_name()
            .expect("worktree basename")
            .to_string_lossy()
            .into_owned();
        let mut renderer = RenderCache::default();

        let fallback =
            String::from_utf8_lossy(renderer.render(&state, "0", 80, 24).row(0)).into_owned();
        assert!(
            fallback.contains(&format!("0:{window_name}")),
            "{fallback:?}"
        );

        for (revision, agent_state, emoji) in [
            (2, AgentState::Working, "🔄"),
            (3, AgentState::Blocked, "✋"),
            (4, AgentState::Idle, "💤"),
            (5, AgentState::Exited, "🏁"),
        ] {
            let mut panes = PaneAgents::new();
            panes.insert(
                pane_id,
                AgentStatus {
                    agent: "codex",
                    pid: Some(42),
                    session_id: None,
                    model: None,
                    state: agent_state,
                },
            );
            assert!(renderer.update_agents(StatusSnapshot { revision, panes }));
            let row =
                String::from_utf8_lossy(renderer.render(&state, "0", 80, 24).row(0)).into_owned();
            assert!(
                row.contains(&format!("0:{emoji} {worktree_name}")),
                "{row:?}"
            );
            assert!(!row.contains(&format!("0:{window_name}")), "{row:?}");
        }
    }

    #[test]
    fn configured_window_format_is_not_replaced_by_agent_label() {
        use crate::integration::status::{AgentStatus, StatusSnapshot};
        use crate::integration::AgentState;
        use crate::observability::v1::PaneId;

        let mut state = ServerState::with_test_session().expect("state");
        state.set_global_option("window-status-format", "#I:custom");
        state.set_global_option("window-status-current-format", "#I:custom");
        let session = state.find("0").expect("session");
        let window = state.window_for_link(&session.windows[session.active]);
        let pane_id = PaneId(window.panes[window.active].id);
        let mut panes = PaneAgents::new();
        panes.insert(
            pane_id,
            AgentStatus {
                agent: "codex",
                pid: Some(42),
                session_id: None,
                model: None,
                state: AgentState::Working,
            },
        );
        let mut renderer = RenderCache::default();
        renderer.update_agents(StatusSnapshot { revision: 2, panes });

        let row = String::from_utf8_lossy(renderer.render(&state, "0", 80, 24).row(0)).into_owned();
        assert!(row.contains("0:custom"), "{row:?}");
        assert!(!row.contains("🔄"), "{row:?}");
    }

    #[test]
    fn complete_style_state_serializes_colours_and_attributes() {
        let mut state = ServerState::with_test_session().expect("state");
        state.set_global_option("status-style", "fg=default,bg=default");
        state.set_global_option(
            "status-format[0]",
            "#[fg=#123456,bg=colour200,bold,italics,underscore]x#[default]",
        );
        let rendered = render_status(
            &state,
            "0",
            2,
            24,
            &ClientContext::default(),
            None,
            &PaneAgents::new(),
        );
        let row = rendered.row(0);
        assert!(row.windows(13).any(|bytes| bytes == b"38;2;18;52;86"));
        assert!(row.windows(8).any(|bytes| bytes == b"48;5;200"));
        assert_eq!(rendered.screen().rows[0].cells[0].text, "x");
    }

    #[test]
    fn alignment_and_ranges_are_cell_based() {
        let base = CellStyle::default();
        let row = draw_row("#[range=left]L#[norange align=right]R", 8, &base);
        assert_eq!(row.cells.first().unwrap().text, "L");
        assert_eq!(row.cells.last().unwrap().text, "R");
        assert_eq!(row.ranges[0].kind, StatusRangeKind::Left);
        assert_eq!((row.ranges[0].start, row.ranges[0].end), (0, 1));
    }

    #[test]
    fn rendered_status_exposes_mouse_ranges() {
        let mut state = ServerState::with_test_session().expect("state");
        state.set_global_option("status-format[0]", "#[range=user|action]go#[norange]");
        let mut renderer = RenderCache::default();
        let rendered = renderer.render(&state, "0", 10, 24);
        assert_eq!(
            rendered.range_at(0, 1).map(|range| &range.kind),
            Some(&StatusRangeKind::User("action".into()))
        );
        assert!(rendered.range_at(0, 2).is_none());
    }

    #[test]
    fn invalid_style_directive_is_atomic() {
        let base = CellStyle::default();
        let row = draw_row("#[fg=red,not-a-style]x", 1, &base);
        assert_eq!(row.cells[0].style, base);
    }

    #[test]
    fn status_metadata_is_rendered_and_validated_atomically() {
        let base = CellStyle::default();
        let row = draw_row("#[acs]q#[noacs]q", 2, &base);
        assert_eq!(serialize_row(&row), b"\x0eq\x0fq");

        let row = draw_row("#[ignore]A#[fg=red]B", 12, &base);
        assert!(String::from_utf8_lossy(&serialize_row(&row)).starts_with("A#[fg=red]B"));

        let row = draw_row("#[ignore]##[fg=red]X", 8, &base);
        assert_eq!(serialize_row(&row), b"fg=red]X");

        for valid in ["width=+1", "width=-0", "width=25%", "pad=+1"] {
            let row = draw_row(&format!("#[fg=red,{valid}]x"), 1, &base);
            assert_eq!(
                row.cells[0].style.fg,
                Colour::Palette(1),
                "rejected {valid}"
            );
        }

        for invalid in [
            "width=bogus",
            "width=101%",
            "pad=bogus",
            "link=https://example.com",
            "dim=50%",
        ] {
            let row = draw_row(&format!("#[fg=red,{invalid}]x"), 1, &base);
            assert_eq!(row.cells[0].style, base, "accepted {invalid}");
        }

        let utf8 = ResolvedTerm::resolve(
            TerminalIdentity::new("utf8", vec![], 0, None).with_utf8(true),
            [],
        );
        let row = draw_row("#[acs]q", 1, &base);
        assert_eq!(
            serialize_row_width_for_terminal(&row, 1, &utf8),
            "─".as_bytes()
        );

        let legacy = ResolvedTerm::resolve(
            TerminalIdentity::new(
                "legacy",
                vec!["smacs=<".into(), "rmacs=>".into(), "acsc=q-".into()],
                0,
                None,
            ),
            [],
        );
        assert_eq!(serialize_row_width_for_terminal(&row, 1, &legacy), b"<-");
    }

    #[test]
    fn overlay_text_consumes_style_directives_without_using_columns() {
        assert_eq!(
            render_overlay_text("#[]:#[fg=red]x", "bg=yellow,fg=black", 4, false),
            b"\x1b[30m\x1b[43m:\x1b[31mx"
        );
    }

    #[test]
    fn cache_is_width_keyed_and_explicitly_invalidated() {
        let mut state = ServerState::with_test_session().expect("state");
        let mut renderer = RenderCache::default();
        let first = renderer.render(&state, "0", 20, 24).row(0).to_vec();
        state.set_global_option("status-format[0]", "changed");
        assert_eq!(renderer.render(&state, "0", 20, 24).row(0), first);
        renderer.invalidate();
        assert_ne!(renderer.render(&state, "0", 20, 24).row(0), first);
        assert_ne!(
            renderer.render(&state, "0", 10, 24).row(0).len(),
            first.len()
        );
    }
}
