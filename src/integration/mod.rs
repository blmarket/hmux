//! Prototype consumers of the native runtime observability contracts.
//!
//! A single background [`AgentObserver`] polls pane observability and classifies
//! each pane's agent lifecycle state. Detection is split into a generic harness
//! (this module) and per-agent [`AgentDetector`]s (e.g. [`codex`], [`claude`],
//! and [`pi`]),
//! so adding an agent is a matter of adding a detector rather than another
//! poller. Running one observer with a detector registry — instead of one thread
//! per agent — avoids two observers fighting over the same pane, and matches how
//! Herdr identifies a pane's agent once and then reads only that agent's UI.

use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use tracing::{info, warn};

use crate::observability::v1::{PaneId, PaneObservability, ServerObservability};
use crate::platform::{CurrentPlatform, Platform};

pub mod claude;
pub mod codex;
pub mod pi;
pub mod status;

use status::{AgentStatus, StatusHub};

#[cfg(test)]
mod tests;

const POLL_INTERVAL: Duration = Duration::from_millis(200);
const SCREEN_LINES: usize = 64;

/// Screen/title-derived lifecycle state reported for an agent pane.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentState {
    /// The pane does not currently contain enough agent UI evidence.
    Unknown,
    /// The agent is showing its live input prompt.
    Idle,
    /// The agent is actively processing or running a tool.
    Working,
    /// The agent is waiting for a confirmation or answer.
    Blocked,
    /// The pane's child process has exited.
    Exited,
}

impl AgentState {
    /// The wire/format spelling of this state, shared by the `#{pane_agent_state}`
    /// format variable and the push protocol's status records. [`Unknown`] maps
    /// to `"none"` — a pane with no definite agent lifecycle signal.
    ///
    /// [`Unknown`]: AgentState::Unknown
    pub fn wire_str(self) -> &'static str {
        match self {
            AgentState::Unknown => "none",
            AgentState::Idle => "idle",
            AgentState::Working => "working",
            AgentState::Blocked => "blocked",
            AgentState::Exited => "exited",
        }
    }

    /// Compact status-bar representation. Unknown or absent status expands to
    /// an empty string so format conditionals can fall back to ordinary pane or
    /// window labels.
    pub fn emoji(self) -> &'static str {
        match self {
            AgentState::Unknown => "",
            AgentState::Idle => "💤",
            AgentState::Working => "🔄",
            AgentState::Blocked => "✋",
            AgentState::Exited => "🏁",
        }
    }
}

/// The outcome of running a detector over one screen sample.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Detection {
    /// A classified state.
    State(AgentState),
    /// Transient UI (e.g. a scrollback/transcript viewer) that carries no
    /// lifecycle signal; the previously published state should be preserved.
    KeepPrevious,
}

/// Terminal cursor evidence captured with the same screen snapshot used by an
/// agent detector. Cursor state is only meaningful after process attribution;
/// an ordinary shell may expose the same shapes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CursorEvidence {
    pub(crate) visible: bool,
    pub(crate) shape: u8,
}

/// Stable source used to attribute an agent process to its session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SessionIdSource {
    /// Inspect open files across the attributed agent process tree.
    ProcessTreeOpenFiles,
    /// Resolve the session from the attributed agent's working directory: the
    /// detector maps the cwd to a per-project session directory, and the newest
    /// file in it names the live session.
    AgentCwdTranscript,
}

/// Whether `text` is a canonical 8-4-4-4-12 hex UUID (case-insensitive).
pub(crate) fn is_uuid(text: &str) -> bool {
    text.len() == 36
        && text.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
}

/// A per-agent recognizer: how to spot the agent's process and how to read its
/// terminal UI. Implementors are stateless — the harness owns all per-pane
/// state — so a single shared instance serves every pane.
pub(crate) trait AgentDetector: Send + Sync {
    /// Stable label used in logs and reports (e.g. `"codex"`, `"claude"`,
    /// `"pi"`).
    fn label(&self) -> &'static str;

    /// Whether `program` — a process image name or a runtime-wrapped script
    /// name — belongs to this agent.
    fn matches_program(&self, program: &OsStr) -> bool;

    /// Whether a process argv identifies this agent. By default, check argv[0]
    /// and argv[1]: argv[0] covers ordinary direct execution, while argv[1]
    /// covers runtime wrappers such as `node path/to/claude.js` without making
    /// the platform layer understand language runtimes.
    fn matches_invocation(&self, arguments: &[OsString]) -> bool {
        arguments
            .iter()
            .take(2)
            .any(|argument| self.matches_program(argument))
    }

    /// Return a lifecycle state implied by the agent's invocation, if any.
    /// Most agents communicate state through their terminal UI. Non-interactive
    /// modes may instead have one state for the lifetime of the process.
    fn invocation_state(&self, _arguments: &[OsString]) -> Option<AgentState> {
        None
    }

    /// Where this agent exposes its stable session id. `None` means the agent
    /// has no discoverable session id.
    fn session_id_source(&self) -> Option<SessionIdSource> {
        None
    }

    /// Extract a session id from an open file owned by the agent process.
    fn session_id_from_open_file(&self, _path: &Path) -> Option<String> {
        None
    }

    /// Map the attributed agent's working directory to the directory that holds
    /// its per-project session files, if the agent keeps any.
    fn session_dir_for_cwd(&self, _cwd: &Path) -> Option<PathBuf> {
        None
    }

    /// Extract a session id from a session file's name (not its full path).
    fn session_id_from_file_name(&self, _name: &OsStr) -> Option<String> {
        None
    }

    /// Classify a plain-text screen tail plus the optional window title.
    fn detect(&self, screen: &str, title: Option<&str>) -> Detection;

    /// Classify with cursor evidence when the process tree has already
    /// attributed the pane to this detector. Most agents do not use cursor
    /// state, so their default remains screen/title-only.
    fn detect_with_cursor(
        &self,
        screen: &str,
        title: Option<&str>,
        _cursor: CursorEvidence,
    ) -> Detection {
        self.detect(screen, title)
    }
}

/// The built-in agent detectors, in dispatch priority order.
pub(crate) fn default_detectors() -> Vec<Box<dyn AgentDetector>> {
    vec![
        Box::new(codex::CodexDetector),
        Box::new(claude::ClaudeDetector),
        Box::new(pi::PiDetector),
    ]
}

/// A single braille cell (U+2800–U+28FF). Codex and Claude both animate their
/// "working" spinner with these in the window title.
pub(crate) fn is_braille(c: char) -> bool {
    ('\u{2800}'..='\u{28FF}').contains(&c)
}

/// Whether a window title begins with a braille spinner cell followed by a space
/// (`^[⠀-⣿] `) — the shared "working" title signal.
pub(crate) fn title_working_spinner(title: &str) -> bool {
    let mut chars = title.chars();
    matches!(chars.next(), Some(first) if is_braille(first)) && chars.next() == Some(' ')
}

/// Background observer which logs agent state transitions for native panes.
pub struct AgentObserver {
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl AgentObserver {
    /// Start polling an observable server with the built-in detector registry,
    /// attributing panes to agents via the real OS process table and
    /// publishing every classified state to `hub` for readers (format variables
    /// and the push handler).
    pub fn start<O>(observability: O, hub: StatusHub) -> io::Result<Self>
    where
        O: ServerObservability + 'static,
    {
        Self::start_with(
            observability,
            default_detectors(),
            Arc::new(SystemProcesses),
            Some(hub),
        )
    }

    /// Start polling with an explicit detector registry, process source, and
    /// optional status hub (test seam). A `None` hub logs state transitions but
    /// publishes nowhere.
    pub(crate) fn start_with<O>(
        observability: O,
        detectors: Vec<Box<dyn AgentDetector>>,
        source: Arc<dyn ProcessSource>,
        hub: Option<StatusHub>,
    ) -> io::Result<Self>
    where
        O: ServerObservability + 'static,
    {
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker = thread::Builder::new()
            .name("hmux-agent-observer".to_string())
            .spawn(move || run(observability, detectors, source, hub, worker_stop))?;
        Ok(Self {
            stop,
            worker: Some(worker),
        })
    }
}

impl Drop for AgentObserver {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            worker.thread().unpark();
            let _ = worker.join();
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReportedStatus {
    agent: Option<&'static str>,
    state: AgentState,
    pid: Option<u32>,
    session_id: Option<String>,
}

struct TrackedPane {
    pane: Arc<dyn PaneObservability>,
    revision: Option<u64>,
    /// Last `(agent label, state, pid, session id)` tuple reported for this pane.
    /// Dedup keys on the whole tuple — not just the state — so that identifying
    /// the agent, replacing the attributed process, or discovering a late
    /// session id is published even when the classified state is unchanged.
    reported: Option<ReportedStatus>,
    /// Index into the detector registry of the last agent identified in this
    /// pane's process tree, if any.
    agent: Option<usize>,
    /// Last matched agent process id, if process attribution found one.
    agent_pid: Option<u32>,
    /// Session id resolved from the matched agent. Lookup retains the last known
    /// id when a poll's inspection is temporarily unavailable and replaces it
    /// when the agent's live session changes (a new Codex rollout, or a newer
    /// Claude transcript in the same project directory).
    agent_session_id: Option<String>,
}

fn run<O: ServerObservability>(
    observability: O,
    detectors: Vec<Box<dyn AgentDetector>>,
    source: Arc<dyn ProcessSource>,
    hub: Option<StatusHub>,
    stop: Arc<AtomicBool>,
) {
    let mut panes = HashMap::<PaneId, TrackedPane>::new();
    while !stop.load(Ordering::Acquire) {
        poll(
            &observability,
            &detectors,
            source.as_ref(),
            hub.as_ref(),
            &mut panes,
        );
        thread::park_timeout(POLL_INTERVAL);
    }
}

fn poll<O: ServerObservability>(
    observability: &O,
    detectors: &[Box<dyn AgentDetector>],
    source: &dyn ProcessSource,
    hub: Option<&StatusHub>,
    panes: &mut HashMap<PaneId, TrackedPane>,
) {
    let ids = match observability.pane_ids() {
        Ok(ids) => ids,
        Err(error) => {
            warn!(target: "hmux::integration", %error, "could not list observable panes");
            return;
        }
    };
    // Scan the whole process table once per poll and share the cached snapshot
    // (read-only) with every pane inspected this tick. Rebuilding the parent→child
    // index requires reading every visible process, which dominates
    // attribution cost; doing it here — the dedicated poller — rather than inside
    // each pane's tree walk means a busy server with many streaming agents scans
    // the process table once per poll cadence instead of once per pane per poll.
    let snapshot = ProcessSnapshot::capture(source);

    let current = ids.iter().copied().collect::<HashSet<_>>();
    panes.retain(|id, tracked| {
        if current.contains(id) {
            true
        } else {
            info!(
                target: "hmux::integration",
                pane_id = id.0,
                previous = ?tracked.reported,
                state = "removed",
                "agent integration state changed"
            );
            if let Some(hub) = hub {
                hub.remove(*id);
            }
            false
        }
    });

    for id in ids {
        if let std::collections::hash_map::Entry::Vacant(entry) = panes.entry(id) {
            match observability.resolve_pane(id) {
                Ok(Some(pane)) => {
                    entry.insert(TrackedPane {
                        pane,
                        revision: None,
                        reported: None,
                        agent: None,
                        agent_pid: None,
                        agent_session_id: None,
                    });
                }
                Ok(None) => continue,
                Err(error) => {
                    warn!(
                        target: "hmux::integration",
                        pane_id = id.0,
                        %error,
                        "could not resolve observable pane"
                    );
                    continue;
                }
            }
        }

        if let Some(tracked) = panes.get_mut(&id) {
            inspect(id, tracked, detectors, &snapshot, hub);
        }
    }
}

/// The result of scanning a pane's process tree for a known agent.
#[derive(Debug)]
enum TreeScan {
    /// The process table is unavailable; the agent cannot be identified by process.
    NoProcessTable,
    /// A known agent was found; the value indexes the detector registry.
    Found {
        detector: usize,
        pid: u32,
        invocation_state: Option<AgentState>,
    },
    /// The process table is available and no known agent runs in the tree.
    NotFound,
}

fn inspect(
    id: PaneId,
    tracked: &mut TrackedPane,
    detectors: &[Box<dyn AgentDetector>],
    snapshot: &ProcessSnapshot,
    hub: Option<&StatusHub>,
) {
    let process = match tracked.pane.process() {
        Ok(process) => process,
        Err(error) => {
            warn!(
                target: "hmux::integration",
                pane_id = id.0,
                %error,
                "could not inspect pane process"
            );
            return;
        }
    };

    if process.exited {
        tracked.agent_pid = None;
        tracked.agent_session_id = None;
        publish(
            id,
            tracked,
            AgentState::Exited,
            tracked.revision,
            process.child_pid,
            None,
            tracked.agent.map(|i| detectors[i].label()),
            hub,
        );
        return;
    }

    // Identify which agent (if any) owns this pane from its process tree.
    let scan = match process.child_pid {
        Some(pid) => find_agent_in_tree(snapshot, pid, detectors),
        // A process-less fixture pane can't be attributed by process; fall back
        // to screen-only detection across the whole registry.
        None => TreeScan::NoProcessTable,
    };

    let revision = match tracked.pane.output_revision() {
        Ok(revision) => revision,
        Err(error) => {
            warn!(
                target: "hmux::integration",
                pane_id = id.0,
                %error,
                "could not inspect pane output revision"
            );
            return;
        }
    };

    // The process tree says no known agent runs here: this is not an agent pane.
    if let TreeScan::NotFound = scan {
        tracked.agent = None;
        tracked.agent_pid = None;
        tracked.agent_session_id = None;
        tracked.revision = Some(revision);
        publish(
            id,
            tracked,
            AgentState::Unknown,
            Some(revision),
            process.child_pid,
            None,
            None,
            hub,
        );
        return;
    }

    // A newly identified agent or replacement pid forces a re-scan even if output
    // hasn't advanced.
    let agent_started =
        matches!(scan, TreeScan::Found { detector, .. } if tracked.agent != Some(detector));
    let agent_pid_changed = match scan {
        TreeScan::Found { pid, .. } => tracked.agent_pid != Some(pid),
        TreeScan::NoProcessTable => tracked.agent_pid.is_some(),
        TreeScan::NotFound => false,
    };
    let previous_session_id = tracked.agent_session_id.clone();
    match scan {
        TreeScan::Found { detector, pid, .. } => {
            let agent_changed = tracked.agent != Some(detector) || tracked.agent_pid != Some(pid);
            if agent_changed {
                tracked.agent_session_id = None;
            }
            if let Some(source) = detectors[detector].session_id_source() {
                // Both sources are rechecked every poll so switching threads or
                // sessions within one agent process replaces the cached id. Each
                // only overwrites on a positive read, so a transient inspection
                // failure retains the last known id rather than clearing it.
                let resolved = match source {
                    SessionIdSource::ProcessTreeOpenFiles => {
                        find_open_file_session_in_tree(snapshot, pid, detectors[detector].as_ref())
                    }
                    SessionIdSource::AgentCwdTranscript => {
                        find_cwd_transcript_session(snapshot, pid, detectors[detector].as_ref())
                    }
                };
                if let Some(session_id) = resolved {
                    tracked.agent_session_id = Some(session_id);
                }
            }
            tracked.agent = Some(detector);
            tracked.agent_pid = Some(pid);
        }
        TreeScan::NoProcessTable => {
            tracked.agent_pid = None;
            tracked.agent_session_id = None;
        }
        TreeScan::NotFound => {}
    }
    let session_id_changed = tracked.agent_session_id != previous_session_id;
    if !agent_started
        && !agent_pid_changed
        && !session_id_changed
        && tracked.revision == Some(revision)
    {
        return;
    }

    if let TreeScan::Found {
        detector,
        pid,
        invocation_state: Some(state),
    } = scan
    {
        tracked.revision = Some(revision);
        publish(
            id,
            tracked,
            state,
            Some(revision),
            process.child_pid,
            Some(pid),
            Some(detectors[detector].label()),
            hub,
        );
        return;
    }

    // The window title is a strong status channel for some agents, but
    // best-effort: a read error must not suppress screen-based detection.
    let title = match tracked.pane.title() {
        Ok(title) => title,
        Err(error) => {
            warn!(
                target: "hmux::integration",
                pane_id = id.0,
                %error,
                "could not read pane title"
            );
            None
        }
    };

    let screen = match tracked.pane.last_lines(SCREEN_LINES) {
        Ok(screen) => screen,
        Err(error) => {
            warn!(
                target: "hmux::integration",
                pane_id = id.0,
                %error,
                "could not read pane screen"
            );
            return;
        }
    };
    tracked.revision = Some(screen.revision);

    let (detection, label, agent_pid) = match scan {
        TreeScan::Found { detector, pid, .. } => (
            detectors[detector].detect_with_cursor(
                &screen.text,
                title.as_deref(),
                CursorEvidence {
                    visible: screen.cursor_visible,
                    shape: screen.cursor_shape,
                },
            ),
            Some(detectors[detector].label()),
            Some(pid),
        ),
        // Process attribution unavailable: run the whole registry and take the
        // first definite classification. Do not pass the terminal title here:
        // Codex uses any non-empty OSC title as a last-resort idle signal once
        // the process tree has already identified Codex, but ordinary shells on
        // macOS often have a static hostname title.
        TreeScan::NoProcessTable => {
            let (detection, label) = run_all(detectors, &screen.text, None);
            (detection, label, None)
        }
        TreeScan::NotFound => unreachable!("handled above"),
    };

    match detection {
        Detection::State(state) => publish(
            id,
            tracked,
            state,
            Some(screen.revision),
            process.child_pid,
            agent_pid,
            label,
            hub,
        ),
        Detection::KeepPrevious => {}
    }
}

/// Run every detector and return the first definite (non-`Unknown`)
/// classification, falling back to any `KeepPrevious`, then `Unknown`.
fn run_all(
    detectors: &[Box<dyn AgentDetector>],
    screen: &str,
    title: Option<&str>,
) -> (Detection, Option<&'static str>) {
    let mut keep_previous = None;
    for detector in detectors {
        match detector.detect(screen, title) {
            Detection::State(AgentState::Unknown) => {}
            state @ Detection::State(_) => return (state, Some(detector.label())),
            Detection::KeepPrevious => {
                keep_previous.get_or_insert((Detection::KeepPrevious, Some(detector.label())));
            }
        }
    }
    keep_previous.unwrap_or((Detection::State(AgentState::Unknown), None))
}

fn publish(
    id: PaneId,
    tracked: &mut TrackedPane,
    state: AgentState,
    revision: Option<u64>,
    child_pid: Option<u32>,
    agent_pid: Option<u32>,
    agent: Option<&'static str>,
    hub: Option<&StatusHub>,
) {
    let reported = ReportedStatus {
        agent,
        state,
        pid: agent_pid,
        session_id: tracked.agent_session_id.clone(),
    };
    if tracked.reported.as_ref() == Some(&reported) {
        return;
    }
    let previous = tracked.reported.clone();
    tracked.reported = Some(reported);
    info!(
        target: "hmux::integration",
        pane_id = id.0,
        agent = ?agent,
        ?child_pid,
        ?agent_pid,
        ?revision,
        ?previous,
        ?state,
        "agent integration state changed"
    );
    // Mirror the transition into the shared hub for readers (format variables +
    // push handler). Only real transitions reach here (the dedup above), so this
    // bumps the hub revision exactly once per observable change.
    if let Some(hub) = hub {
        hub.publish(
            id,
            AgentStatus {
                agent: agent.unwrap_or(""),
                pid: agent_pid,
                session_id: tracked.agent_session_id.clone(),
                state,
            },
        );
    }
}

/// Read-only access to the OS process table, used to attribute a pane's process
/// subtree to a known agent.
///
/// This is abstracted behind a trait so the attribution logic can be exercised
/// against a synthetic tree in tests, and so the descent does not depend on any
/// single OS interface. In particular the real implementation reconstructs the
/// tree from each process's parent pid rather than relying on a kernel-provided
/// children list.
pub(crate) trait ProcessSource: Send + Sync {
    /// A snapshot of `(pid, ppid)` for every visible process, or `None` when the
    /// process table is unavailable on this platform.
    fn process_table(&self) -> Option<Vec<(u32, u32)>>;

    /// Candidate OS-visible program names for `pid`, most specific first.
    /// Empty when the process cannot be read.
    fn programs(&self, pid: u32) -> Vec<OsString>;

    /// Full process argument vector, including argv[0]. Empty when unreadable.
    fn arguments(&self, _pid: u32) -> Vec<OsString> {
        Vec::new()
    }

    /// The working directory of `pid`, when readable — used to locate an agent's
    /// per-project session state. `None` when unsupported or unreadable.
    fn cwd(&self, _pid: u32) -> Option<PathBuf> {
        None
    }

    /// The regular files in `dir`, most recently modified first. Empty when the
    /// directory is unreadable or the platform does not support inspection.
    fn files_newest_first(&self, _dir: &Path) -> Vec<PathBuf> {
        Vec::new()
    }

    /// Paths currently open by `pid`. Empty when unsupported or unreadable.
    fn open_files(&self, _pid: u32) -> Vec<PathBuf> {
        Vec::new()
    }
}

/// The production [`ProcessSource`], backed by the host OS process table.
pub(crate) struct SystemProcesses;

impl ProcessSource for SystemProcesses {
    fn process_table(&self) -> Option<Vec<(u32, u32)>> {
        CurrentPlatform::process_table().map(|table| {
            table
                .into_iter()
                .map(|process| (process.pid, process.ppid))
                .collect()
        })
    }

    fn programs(&self, pid: u32) -> Vec<OsString> {
        CurrentPlatform::process_programs(pid)
    }

    fn arguments(&self, pid: u32) -> Vec<OsString> {
        CurrentPlatform::process_arguments(pid)
    }

    fn cwd(&self, pid: u32) -> Option<PathBuf> {
        CurrentPlatform::process_cwd(pid)
    }

    fn files_newest_first(&self, dir: &Path) -> Vec<PathBuf> {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        let mut files = entries
            .flatten()
            .filter_map(|entry| {
                let modified = entry.metadata().ok()?.modified().ok()?;
                Some((modified, entry.path()))
            })
            .collect::<Vec<_>>();
        files.sort_unstable_by_key(|(modified, _)| std::cmp::Reverse(*modified));
        files.into_iter().map(|(_, path)| path).collect()
    }

    fn open_files(&self, pid: u32) -> Vec<PathBuf> {
        CurrentPlatform::process_open_files(pid)
    }
}

/// A single poll tick's view of the process table, captured once by the observer
/// poller and shared read-only across every pane inspected that tick.
///
/// Building the parent→child index requires reading every visible process,
/// which dominates attribution cost. Capturing it once per poll — rather than
/// re-scanning inside each pane's tree walk — is what keeps a busy server (many
/// agents all streaming output) scanning the table once per poll cadence instead
/// of once per pane per poll. The poller is the sole writer: it constructs the
/// snapshot and hands out only shared references, so readers cannot mutate the
/// cached table and cannot trigger a fresh scan on output churn.
pub(crate) struct ProcessSnapshot<'a> {
    source: &'a dyn ProcessSource,
    /// `ppid -> child pids`, reconstructed from the parent-pid edges, or `None`
    /// when the process table was unavailable this tick.
    children: Option<HashMap<u32, Vec<u32>>>,
}

impl<'a> ProcessSnapshot<'a> {
    /// Scan the process table once and index it by parent pid; all per-pane
    /// attribution reads from the returned snapshot.
    pub(crate) fn capture(source: &'a dyn ProcessSource) -> Self {
        let children = source.process_table().map(|table| {
            let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
            for (pid, ppid) in table {
                children.entry(ppid).or_default().push(pid);
            }
            children
        });
        Self { source, children }
    }

    /// Whether the process table was available when this snapshot was captured.
    fn has_process_table(&self) -> bool {
        self.children.is_some()
    }

    /// The direct children of `pid` in the captured table.
    fn children_of(&self, pid: u32) -> &[u32] {
        self.children
            .as_ref()
            .and_then(|children| children.get(&pid))
            .map_or(&[][..], Vec::as_slice)
    }
}

/// Find a detector-recognized session file on an agent process or descendant.
/// Launchers named `codex` commonly keep the actual Codex runtime as a child,
/// and that runtime owns the open rollout descriptor.
fn find_open_file_session_in_tree(
    snapshot: &ProcessSnapshot,
    root: u32,
    detector: &dyn AgentDetector,
) -> Option<String> {
    let mut pending = vec![root];
    let mut visited = HashSet::new();
    while let Some(pid) = pending.pop() {
        if !visited.insert(pid) {
            continue;
        }
        if let Some(session_id) = snapshot
            .source
            .open_files(pid)
            .iter()
            .find_map(|path| detector.session_id_from_open_file(path))
        {
            return Some(session_id);
        }
        pending.extend(snapshot.children_of(pid).iter().copied());
    }
    None
}

/// Resolve a session id from the agent's working directory: the detector maps
/// its cwd to a per-project session directory, and the newest file whose name
/// the detector recognizes names the live session. Reading the cwd once from the
/// matched process suffices — an agent and any wrapper below it share the working
/// directory they were launched in.
fn find_cwd_transcript_session(
    snapshot: &ProcessSnapshot,
    pid: u32,
    detector: &dyn AgentDetector,
) -> Option<String> {
    let cwd = snapshot.source.cwd(pid)?;
    let dir = detector.session_dir_for_cwd(&cwd)?;
    snapshot
        .source
        .files_newest_first(&dir)
        .iter()
        .find_map(|path| {
            path.file_name()
                .and_then(|name| detector.session_id_from_file_name(name))
        })
}

/// Return the registry index of the first known agent found at `root` or one of
/// its descendants, or the appropriate [`TreeScan`] outcome when none is.
///
/// The subtree is reconstructed from the parent-pid edges captured in `snapshot`,
/// so descent works regardless of whether the kernel exposes a per-process
/// children list — and, because the table was scanned once by the poller, this
/// walk does no full process-table scan of its own. Per-pid `programs` and
/// `arguments` reads are bounded to the pids on the pane's own subtree path.
fn find_agent_in_tree(
    snapshot: &ProcessSnapshot,
    root: u32,
    detectors: &[Box<dyn AgentDetector>],
) -> TreeScan {
    if !snapshot.has_process_table() {
        return TreeScan::NoProcessTable;
    }

    let mut pending = vec![root];
    let mut visited = HashSet::new();
    while let Some(pid) = pending.pop() {
        if !visited.insert(pid) {
            continue;
        }
        for program in snapshot.source.programs(pid) {
            if let Some(index) = program_matches_any(&program, detectors) {
                let arguments = snapshot.source.arguments(pid);
                return TreeScan::Found {
                    detector: index,
                    pid,
                    invocation_state: detectors[index].invocation_state(&arguments),
                };
            }
        }
        let arguments = snapshot.source.arguments(pid);
        if let Some(index) = invocation_matches_any(&arguments, detectors) {
            return TreeScan::Found {
                detector: index,
                pid,
                invocation_state: detectors[index].invocation_state(&arguments),
            };
        }
        pending.extend(snapshot.children_of(pid).iter().copied());
    }
    TreeScan::NotFound
}

fn program_matches_any(program: &OsStr, detectors: &[Box<dyn AgentDetector>]) -> Option<usize> {
    detectors.iter().position(|d| d.matches_program(program))
}

fn invocation_matches_any(
    arguments: &[OsString],
    detectors: &[Box<dyn AgentDetector>],
) -> Option<usize> {
    detectors
        .iter()
        .position(|d| d.matches_invocation(arguments))
}
