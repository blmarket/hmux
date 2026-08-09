//! A tiny tmux command interpreter.
//!
//! Only the handful of commands the prototype needs are implemented; anything
//! else returns a nonzero exit with an error line, like tmux does for an unknown
//! command. Output is returned as text + streams; the protocol layer
//! ([`super::protocol`]) delivers it over the imsg file protocol.
//!
//! Behaviors here are pinned against real tmux by the differential conformance
//! suite (`hmux_conformance::behaviors`), which runs the identical command
//! sequence against this interpreter and against stock tmux and
//! asserts the observable results (exit code, stdout, stderr) match. When a gap
//! is found there, it's closed here.

pub(in crate::server) mod buffers;
pub(in crate::server) mod clients;
pub(in crate::server) mod configuration;
pub(in crate::server) mod execution;
mod identity;
pub(in crate::server) mod keys;
pub(in crate::server) mod panes;
pub(in crate::server) mod queue;
pub(in crate::server) mod server;
pub(in crate::server) mod sessions;
pub(crate) mod suspend;
pub(in crate::server) mod windows;

#[cfg(test)]
pub(in crate::server) use identity::all as all_commands;
pub(in crate::server) use identity::Command;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::integration::status::PaneAgents;
use crate::observability::v1::PaneId;

use super::format::{self, Vars};
use super::key::{format_key_name, parse_key_name, KeyBase, KeyCode, SpecialKey};
use super::mouse::MouseEvent;
use super::options::{self, OptionScope, OptionSet, OptionsView};
use super::pane::PaneClass;
use super::registry::{self, CommandSpec, Resolution, SpecResolution};
use super::state::{
    BackgroundJobRegistry, ClientActionResult, ClientMessage, ClientMessageResult, MenuItem,
    MenuRequest, ModeEdit, ModeItem, ModeKind, ModeView, OverlayRequest, PaneSpec, PopupRequest,
    PromptCompletion, PromptReply, ServerState, Session, SharedState, SpawnSession, SplitDirection,
    Target, WaitOutcome, WaitRegistry, WindowResizeAdjust, WindowResizeRequest, WindowSizePolicy,
};
use super::style::{CaptureStyleWriter, CellPresentation, Hyperlink, SgrDecoder};
use super::task::{
    Completion, Coroutine, FdInterest, ReadySet, TaskPoll, TaskState, WaitRequest, WaitToken,
};
use crate::vt::screen::{CaptureExtent, CellWidth, Grid, GridRow};

/// tmux's `NEW_SESSION_TEMPLATE` (cmd-new-session.c): what `new-session -P`
/// prints when no `-F` is given.
const NEW_SESSION_TEMPLATE: &str = "#{session_name}:";
/// tmux's `NEW_WINDOW_TEMPLATE` (cmd-new-window.c): what `new-window -P` prints.
const NEW_WINDOW_TEMPLATE: &str = "#{session_name}:#{window_index}.#{pane_index}";
const DISPLAY_MESSAGE_TEMPLATE: &str = "[#{session_name}] #{window_index}:#{window_name}, current pane #{pane_index} - (%H:%M %d-%b-%y)";
const NEW_SESSION_VALUE_FLAGS: &[&str] = &["-c", "-e", "-F", "-f", "-n", "-s", "-t", "-x", "-y"];
const NEW_SESSION_MAX_SIZE: u16 = 10_000;

pub(crate) enum DeferredCommand {
    Args(Vec<String>),
    Line { line: String, tail: Vec<String> },
}

/// The result of running a command: what to write to the client's stdout/stderr
/// and the process exit code.
pub struct CommandResult {
    pub stdout: String,
    pub stdout_bytes: Vec<u8>,
    pub stderr: String,
    pub exit: i32,
    pub(crate) pane_output_wait: Option<(Rc<super::pane::NativePaneObservation>, u64)>,
    pub(crate) deferred_commands: Vec<DeferredCommand>,
    pub(crate) background_commands: Vec<BackgroundCommandRequest>,
    /// Continue the containing command list despite a nonzero client status.
    /// Interactive queue items use this when cancellation sets the command
    /// client's exit status without removing later items from the queue.
    pub(crate) continue_queue: bool,
    /// Results of commands inserted immediately after this queue item.
    pub(crate) inserted_results: Vec<CommandResult>,
    /// Final control-mode marker field for this inserted queue item.
    pub(crate) control_flags: u8,
}

/// What kind of endpoint the commands run under a context come from. Fixed
/// per connection: set at identify time and replaced on the clone a control
/// or attached client keeps; every later clone inherits it.
#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) enum ClientKind {
    /// No client: server-internal work such as deferred notification hooks.
    #[default]
    Detached,
    /// An unattached command-line client.
    Command,
    /// A control-mode client. When the client's active-pane flag is on, it
    /// carries the client's own view of each window's active pane, shared
    /// with every context cloned for its commands.
    Control {
        active_panes: Option<Rc<RefCell<BTreeMap<u32, u32>>>>,
    },
    /// A client attached to a session.
    Attached,
}

/// The hook-body scope a queued command runs in.
#[derive(Clone)]
pub(crate) struct HookScope {
    /// The `hook*` format variables; installed into the server state around
    /// the command's execution.
    pub(crate) vars: Rc<Vec<(String, String)>>,
    /// The hook's own target, which is what a command in its body resolves
    /// against when it names no target of its own.
    pub(crate) target: Option<Rc<str>>,
}

/// Per-command-client process context collected from tmux identify frames.
///
/// Besides the process facts, this carries two kinds of execution state: who
/// is asking ([`ClientKind`], fixed per connection) and how the command came
/// to run — the [`HookScope`] plus the `suppress_*`/`nested_granularity`
/// latches, stamped onto the clone a hook-body or nested queue runs with and
/// inherited by everything queued beneath it.
#[derive(Clone, Default)]
pub struct ClientContext {
    pub cwd: Option<PathBuf>,
    pub environment: Vec<String>,
    pub tty_name: Option<String>,
    pub client_pid: Option<i32>,
    /// The uid the kernel reports for the far end of this client's socket —
    /// what tmux's `#{client_uid}` and `#{client_user}` answer from. `None`
    /// when the platform did not report one.
    pub peer_uid: Option<u32>,
    pub(crate) input_file: Option<Result<Vec<u8>, i32>>,
    pub(crate) current_session_id: Option<u32>,
    pub(crate) read_only: bool,
    pub(crate) key_event: Option<super::key::KeyCode>,
    pub(crate) mouse: Option<MouseEvent>,
    pub(crate) interaction_reply: Option<PromptReply>,
    pub(crate) kind: ClientKind,
    /// Set on child queues (hook bodies and inserted nested command lines) so
    /// they hand each inserted item's result to the parent queue intact; the
    /// top-level queue decides whether to flatten them.
    pub(crate) nested_granularity: bool,
    /// Set for commands run from a hook body. tmux's `CMDQ_STATE_NOHOOKS`
    /// suppresses only the `after-*`/`command-error` hooks a command would
    /// raise; the event notifications its mutations raise still fire, because
    /// those are queued as fresh items rather than inheriting this state.
    pub(crate) suppress_after_hooks: bool,
    /// Set for commands run from an *event* hook body, where tmux's global
    /// queue carries `CMDQ_STATE_NOHOOKS` and `notify_add` therefore drops
    /// anything the body raises. This is what stops an event hook that mutates
    /// its own subject from re-triggering itself.
    pub(crate) suppress_notifications: bool,
    /// The hook body this command runs in, if any. `set-hook -R` shows the
    /// scope is independent of the `suppress_*` latches: its hook body gets a
    /// scope but still runs its commands' own `after-*` hooks.
    pub(crate) hook: Option<HookScope>,
}

impl ClientContext {
    /// Whether an interactive command (`command-prompt`, `display-menu`, ...)
    /// should keep this client blocked until the user responds. tmux blocks
    /// command and control clients; an attached client's own commands run the
    /// interaction inline in its UI instead.
    pub(crate) fn wait_for_interactions(&self) -> bool {
        matches!(self.kind, ClientKind::Command | ClientKind::Control { .. })
    }

    /// Whether the results of commands inserted behind a queue item (hook
    /// bodies, nested lines) stay separate `CommandResult`s instead of being
    /// flattened into the item's own result. Control mode needs the separation
    /// to give every queue item its own `%begin`/`%end` block; child queues
    /// need it so the parent queue gets to make that choice.
    pub(crate) fn preserve_queue_insertions(&self) -> bool {
        matches!(self.kind, ClientKind::Control { .. }) || self.nested_granularity
    }

    /// A copy whose environment is what a process this client starts should
    /// see: tmux's `environ_for_session` against the client's own session,
    /// which is what `job_run` builds for `run-shell`, `if-shell` and
    /// `copy-pipe`.
    pub(crate) fn with_job_environment(&self, st: &ServerState) -> ClientContext {
        let session = self
            .current_session_id
            .and_then(|id| st.session_by_id(id))
            .map(|session| session.name.clone());
        let mut context = self.clone();
        context.environment = st.job_environment(session.as_deref()).as_ref().clone();
        context
    }

    pub(crate) fn env(&self, name: &str) -> Option<&str> {
        self.environment
            .iter()
            .filter_map(|entry| entry.split_once('='))
            .find_map(|(key, value)| (key == name).then_some(value))
    }

    /// The control client's shared active-pane map, if this context belongs to
    /// a control client tracking one.
    pub(crate) fn control_active_panes(&self) -> Option<&Rc<RefCell<BTreeMap<u32, u32>>>> {
        match &self.kind {
            ClientKind::Control { active_panes } => active_panes.as_ref(),
            _ => None,
        }
    }

    fn active_panes(&self) -> Option<BTreeMap<u32, u32>> {
        self.control_active_panes()
            .map(|panes| panes.borrow().clone())
    }

    fn set_active_pane(&self, window_id: u32, pane_id: u32) {
        if let Some(panes) = self.control_active_panes() {
            panes.borrow_mut().insert(window_id, pane_id);
        }
    }
}

impl CommandResult {
    pub(crate) fn ok(stdout: impl Into<String>) -> Self {
        CommandResult {
            stdout: stdout.into(),
            stdout_bytes: Vec::new(),
            stderr: String::new(),
            exit: 0,
            pane_output_wait: None,
            deferred_commands: Vec::new(),
            background_commands: Vec::new(),
            continue_queue: false,
            inserted_results: Vec::new(),
            control_flags: 1,
        }
    }

    pub(crate) fn err(stderr: impl Into<String>) -> Self {
        CommandResult {
            stdout: String::new(),
            stdout_bytes: Vec::new(),
            stderr: stderr.into(),
            exit: 1,
            pane_output_wait: None,
            deferred_commands: Vec::new(),
            background_commands: Vec::new(),
            continue_queue: false,
            inserted_results: Vec::new(),
            control_flags: 1,
        }
    }

    pub(crate) fn ok_bytes(stdout: Vec<u8>) -> Self {
        match String::from_utf8(stdout) {
            Ok(stdout) => Self::ok(stdout),
            Err(error) => Self {
                stdout: String::new(),
                stdout_bytes: error.into_bytes(),
                stderr: String::new(),
                exit: 0,
                pane_output_wait: None,
                deferred_commands: Vec::new(),
                background_commands: Vec::new(),
                continue_queue: false,
                inserted_results: Vec::new(),
                control_flags: 1,
            },
        }
    }

    pub(crate) fn stdout_data(&self) -> &[u8] {
        if self.stdout_bytes.is_empty() {
            self.stdout.as_bytes()
        } else {
            &self.stdout_bytes
        }
    }

    /// Append stdout without losing ordering when either result contains bytes
    /// that are not valid UTF-8.
    pub(crate) fn append_stdout(&mut self, other: &Self) {
        if self.stdout_bytes.is_empty() && other.stdout_bytes.is_empty() {
            self.stdout.push_str(&other.stdout);
            return;
        }
        if self.stdout_bytes.is_empty() {
            self.stdout_bytes = std::mem::take(&mut self.stdout).into_bytes();
        }
        self.stdout_bytes.extend_from_slice(other.stdout_data());
    }
}

pub(crate) fn command_prompt_target(args: &[String]) -> Option<String> {
    let normalized = normalize_argv("command-prompt", args);
    flag_value(&normalized, "-t").map(str::to_string)
}

pub(crate) fn command_prompt_waits(args: &[String]) -> bool {
    let normalized = normalize_argv("command-prompt", args);
    !has_flag(&normalized, "-b") && !has_flag(&normalized, "-i")
}

#[derive(Clone, Debug)]
pub(crate) struct CommandPromptPage {
    pub(crate) label: String,
    pub(crate) initial: String,
}

#[derive(Clone, Debug)]
pub(crate) struct CommandPromptSpec {
    pub(crate) pages: Vec<CommandPromptPage>,
    pub(crate) single: bool,
    pub(crate) numeric: bool,
    pub(crate) incremental: bool,
    pub(crate) key: bool,
    pub(crate) backspace_exit: bool,
    pub(crate) no_freeze: bool,
    pub(crate) prompt_type: String,
}

pub(crate) fn command_prompt_spec(args: &[String]) -> Result<CommandPromptSpec, String> {
    if let Err(error) = parse_command_groups(vec![args]) {
        return Err(error.stderr);
    }
    let normalized = normalize_argv("command-prompt", args);
    let prompt_type = flag_value(&normalized, "-T").unwrap_or("command");
    if !matches!(
        prompt_type,
        "command" | "search" | "target" | "window-target"
    ) {
        return Err(format!("unknown type: {prompt_type}\n"));
    }
    let literal = has_flag(&normalized, "-l");
    let raw_prompt = flag_value(&normalized, "-p");
    let template = trailing_command(&normalized, &["-I", "-p", "-t", "-T"])
        .into_iter()
        .next();
    let (prompt_text, add_space) = match raw_prompt {
        Some(prompt) => (prompt.to_string(), true),
        None => match template {
            Some(template) => {
                let command = template
                    .split(|character: char| character.is_whitespace() || character == ',')
                    .next()
                    .unwrap_or_default();
                (format!("({command})"), true)
            }
            None => (":".to_string(), false),
        },
    };
    let raw_inputs = flag_value(&normalized, "-I").unwrap_or("");
    let labels = if literal {
        vec![prompt_text]
    } else {
        prompt_text.split(',').map(str::to_string).collect()
    };
    let mut inputs = if literal {
        vec![raw_inputs.to_string()]
    } else {
        raw_inputs
            .split(',')
            .map(str::to_string)
            .collect::<Vec<_>>()
    };
    inputs.resize(labels.len(), String::new());
    let pages = labels
        .into_iter()
        .zip(inputs)
        .map(|(label, initial)| CommandPromptPage {
            label: if add_space && !literal {
                format!("{label} ")
            } else {
                label
            },
            initial,
        })
        .collect();
    let single = has_flag(&normalized, "-1");
    let numeric = !single && has_flag(&normalized, "-N");
    let incremental = !single && !numeric && has_flag(&normalized, "-i");
    let key = !single && !numeric && !incremental && has_flag(&normalized, "-k");
    let backspace_exit = !single && !numeric && !incremental && !key && has_flag(&normalized, "-e");
    Ok(CommandPromptSpec {
        pages,
        single,
        numeric,
        incremental,
        key,
        backspace_exit,
        no_freeze: has_flag(&normalized, "-C"),
        prompt_type: prompt_type.to_string(),
    })
}

pub(crate) fn expand_command_prompt_format(
    source: &str,
    st: &mut ServerState,
    agents: &PaneAgents,
    context: &ClientContext,
) -> String {
    let previous_session = st.replace_command_session_id(context.current_session_id);
    let target = current_session(&st);
    let expanded = target
        .as_deref()
        .and_then(|target| st.resolve(target))
        .map(|resolved| {
            let mut vars = vars_full(
                &st,
                &st.sessions()[resolved.session],
                resolved.window,
                resolved.pane,
                agents,
                st.marked_pane(),
            );
            for (name, value) in st.env_iter() {
                vars.set(name, value);
            }
            if let Ok(entries) = st.format_option_entries(target.as_deref().unwrap_or_default()) {
                for (name, value) in entries {
                    vars.set(name, value);
                }
            }
            format::expand(source, &vars)
        })
        .unwrap_or_else(|| source.to_string());
    st.replace_command_session_id(previous_session);
    expanded
}

pub(crate) fn command_prompt_template(
    args: &[String],
    values: &[String],
    state: &SharedState,
    agents: &PaneAgents,
    context: &ClientContext,
) -> String {
    let normalized = normalize_argv("command-prompt", args);
    let mut template = trailing_command(&normalized, &["-I", "-p", "-t", "-T"])
        .into_iter()
        .next()
        .unwrap_or("%1")
        .to_string();
    if has_flag(&normalized, "-F") {
        template =
            expand_command_prompt_format(&template, &mut state.borrow_mut(), agents, context);
    }
    for (index, value) in values.iter().enumerate() {
        template = replace_prompt_template(&template, value, (index + 1) as u8);
    }
    template
}

/// Run a command line against the shared state.
///
/// A command client's argv can carry several commands separated by a standalone
/// `;` token (`new-session -d ; list-sessions`). tmux parses the whole line
/// first — an unknown/ambiguous command name is a *parse* error that aborts the
/// entire line before anything runs — then executes the commands in order,
/// stopping at the first one that fails. We reproduce both phases here.
#[cfg(test)]
pub fn run(args: &[String], state: &SharedState, agents: &PaneAgents) -> CommandResult {
    let context = ClientContext::default();
    let mut driver =
        crate::event_loop::test_driver::LoopCommandDriver::new().expect("command test loop");
    let mut result = match start_resumable_command(args, state, agents, &context) {
        Ok(queue) => driver.run_queue(queue, state),
        Err(result) => result,
    };
    for request in result.background_commands.drain(..) {
        driver.run_background(request, state, agents);
    }
    result
}

/// Parse and normalize every command before client-side file operations begin.
pub(crate) fn command_line_groups(
    args: &[String],
    aliases: &[(String, String)],
) -> Result<Vec<Vec<String>>, CommandResult> {
    let expanded = expand_attached_separators(args);
    parse_command_groups_with_aliases(split_commands(&expanded), aliases)
        .map(|commands| commands.into_iter().map(|command| command.args).collect())
}

/// Parse one tmux command string into normalized command argv groups.
///
/// Control mode and configuration files receive command strings rather than an
/// already split argv. Keep string tokenization and command validation in this
/// module so every caller gets the same quoting, separator, alias, getopt, and
/// arity behavior. The complete line is validated before any group is returned.
pub(crate) fn command_string_groups(line: &str) -> Result<Vec<Vec<String>>, CommandResult> {
    let tokens = tokenize_line(line);
    let owned_groups = tokenized_command_groups(&tokens);
    let groups = owned_groups.iter().map(Vec::as_slice).collect::<Vec<_>>();
    parse_command_groups(groups)
        .map(|commands| commands.into_iter().map(|command| command.args).collect())
}

pub(crate) fn command_string_groups_with_aliases(
    line: &str,
    aliases: &[(String, String)],
) -> Result<Vec<Vec<String>>, CommandResult> {
    let tokens = tokenize_line(line);
    let owned_groups = tokenized_command_groups(&tokens);
    let groups = owned_groups.iter().map(Vec::as_slice).collect::<Vec<_>>();
    parse_command_groups_with_aliases(groups, aliases)
        .map(|commands| commands.into_iter().map(|command| command.args).collect())
}

/// Whether a command needs the stock client's read/write file handshake.
pub(crate) fn uses_client_file_protocol(args: &[String]) -> bool {
    let Some(name) = args
        .first()
        .and_then(|name| match registry::resolve_spec(name) {
            SpecResolution::Spec(spec) => Some(spec.name),
            _ => None,
        })
    else {
        return false;
    };
    match name {
        "load-buffer" => true,
        "display-message" | "split-window" => has_flag(args, "-I"),
        "save-buffer" => {
            let normalized = normalize_argv(name, args);
            save_buffer_path(&normalized).is_some_and(|path| path != "-")
        }
        _ => false,
    }
}

/// Run a command line to completion on the calling thread. Test scaffolding;
/// the server drives the same queue through the loop.
#[cfg(test)]
pub fn run_with_context(
    args: &[String],
    state: &SharedState,
    agents: &PaneAgents,
    context: &ClientContext,
) -> CommandResult {
    let queue = match start_resumable_command(args, state, agents, context) {
        Ok(queue) => queue,
        Err(error) => return error,
    };
    crate::event_loop::test_driver::LoopCommandDriver::new()
        .expect("command test loop")
        .run_queue(queue, state)
}

pub(crate) fn start_resumable_command(
    args: &[String],
    state: &SharedState,
    agents: &PaneAgents,
    context: &ClientContext,
) -> Result<ResumableCommandQueue, CommandResult> {
    let expanded_args = expand_attached_separators(args);
    let groups = split_commands(&expanded_args);
    let aliases = {
        let state = state.borrow_mut();
        state.command_aliases()
    };
    let parsed = parse_command_groups_with_aliases(groups, &aliases)?;
    Ok(ResumableCommandQueue::new(parsed, agents, context))
}

pub(crate) fn start_resumable_command_string(
    line: &str,
    state: &SharedState,
    agents: &PaneAgents,
    context: &ClientContext,
) -> Result<ResumableCommandQueue, CommandResult> {
    let tokens = tokenize_line(line);
    let owned_groups = tokenized_command_groups(&tokens);
    let groups = owned_groups.iter().map(Vec::as_slice).collect::<Vec<_>>();
    let aliases = {
        let state = state.borrow_mut();
        state.command_aliases()
    };
    let parsed = parse_command_groups_with_aliases(groups, &aliases)?;
    Ok(ResumableCommandQueue::new(parsed, agents, context))
}

pub(crate) fn start_resumable_command_string_with_tail(
    line: &str,
    tail: &[String],
    state: &SharedState,
    agents: &PaneAgents,
    context: &ClientContext,
) -> Result<ResumableCommandQueue, CommandResult> {
    let tokens = tokenize_line(line);
    let owned_groups = tokenized_command_groups(&tokens);
    let groups = owned_groups.iter().map(Vec::as_slice).collect::<Vec<_>>();
    let aliases = {
        let state = state.borrow_mut();
        state.command_aliases()
    };
    let mut parsed = parse_command_groups_with_aliases(groups, &aliases)?;
    if !tail.is_empty() {
        let expanded_tail = expand_attached_separators(tail);
        parsed.extend(parse_command_groups_with_aliases(
            split_commands(&expanded_tail),
            &aliases,
        )?);
    }
    Ok(ResumableCommandQueue::new(parsed, agents, context))
}

struct PreviousCommandTargetContext {
    session_id: Option<u32>,
    window_id: Option<u32>,
    active_panes: Option<BTreeMap<u32, u32>>,
    hook_vars: Option<Vec<(String, String)>>,
    mouse: Option<MouseEvent>,
    format_jobs: Option<Rc<CommandJobs>>,
}

fn install_command_target_context(
    state: &mut ServerState,
    context: &ClientContext,
) -> PreviousCommandTargetContext {
    // Inside a hook body the hook's own target replaces the client's current
    // one, so an untargeted command in the body acts on what the hook is about.
    // A mouse binding does the same with what the event hit (tmux's
    // `cmd_find_from_mouse`), which is how `MouseDown1Status` acts on the
    // window that was clicked rather than the current one.
    let mouse_target = context
        .mouse
        .as_ref()
        .and_then(|mouse| mouse.target.as_ref())
        .map(|_| "=");
    let default_target = context
        .hook
        .as_ref()
        .and_then(|hook| hook.target.as_deref())
        .or(mouse_target)
        .and_then(|target| {
            let previous = state.replace_command_mouse(context.mouse.clone());
            let resolved = state.resolve(target);
            state.replace_command_mouse(previous);
            resolved
        });
    let session_id = default_target
        .map(|resolved| state.sessions()[resolved.session].id)
        .or(context.current_session_id);
    let window_id = default_target
        .map(|resolved| state.sessions()[resolved.session].windows[resolved.window].id);
    PreviousCommandTargetContext {
        session_id: state.replace_command_session_id(session_id),
        window_id: state.replace_command_window_id(window_id),
        active_panes: state.replace_command_active_panes(context.active_panes()),
        hook_vars: context
            .hook
            .as_ref()
            .map(|hook| state.replace_hook_format_vars(hook.vars.as_ref().clone())),
        mouse: state.replace_command_mouse(context.mouse.clone()),
        format_jobs: {
            let jobs = Rc::new(CommandJobs::new(state, context));
            state.replace_command_format_jobs(Some(jobs))
        },
    }
}

fn restore_command_target_context(state: &mut ServerState, previous: PreviousCommandTargetContext) {
    state.replace_command_session_id(previous.session_id);
    state.replace_command_window_id(previous.window_id);
    state.replace_command_active_panes(previous.active_panes);
    if let Some(vars) = previous.hook_vars {
        state.replace_hook_format_vars(vars);
    }
    state.replace_command_mouse(previous.mouse);
    state.replace_command_format_jobs(previous.format_jobs);
}

/// Expand one of a command's format templates, running `#()` jobs in the tree
/// [`install_command_target_context`] put in place for the running command.
fn expand_command_format(
    st: &ServerState,
    template: &str,
    vars: &Vars,
    loops: Option<&dyn format::LoopSource>,
) -> String {
    format::expand_with_jobs(
        template,
        vars,
        loops,
        command_jobs(st),
        Some(&ServerFormatTree(st)),
    )
}

fn command_jobs(st: &ServerState) -> Option<&dyn format::FormatJobs> {
    st.command_format_jobs()
        .map(|jobs| jobs.as_ref() as &dyn format::FormatJobs)
}

/// The server as a format's `#{C:…}` and `#{N:…}` see it: the pane and session
/// a format tree is being expanded for come from its own variables, so one
/// implementation serves every expansion point.
pub(crate) struct ServerFormatTree<'a>(pub(crate) &'a ServerState);

impl format::FormatTree for ServerFormatTree<'_> {
    fn search_pane(&self, vars: &Vars, term: &str, regex: bool, ignore_case: bool) -> u32 {
        let Some(pane_id) = vars
            .lookup("pane_id")
            .and_then(|id| id.strip_prefix('%'))
            .and_then(|id| id.parse::<u32>().ok())
        else {
            return 0;
        };
        self.0.search_pane_screen(pane_id, term, regex, ignore_case)
    }

    fn name_exists(&self, vars: &Vars, scope: format::NameScope, name: &str) -> bool {
        match scope {
            format::NameScope::Session => {
                self.0.sessions().iter().any(|session| session.name == name)
            }
            // tmux looks only in the session the format belongs to.
            format::NameScope::Window => vars
                .lookup("session_name")
                .and_then(|session| self.0.resolve_session(session))
                .is_some_and(|session| {
                    session
                        .windows
                        .iter()
                        .any(|link| self.0.window_for_link(link).name == name)
                }),
        }
    }
}

struct SourceLocation {
    path: String,
    line: usize,
}

enum SharedQueueItem {
    Command {
        command: ParsedCommand,
        source: Option<SourceLocation>,
        source_depth: u8,
        contributes_status: bool,
    },
    FinalizeSource {
        args: Vec<String>,
    },
    FinalizeHooks {
        command: &'static str,
        args: Vec<String>,
    },
    NestedCommand {
        queue: Box<ResumableCommandQueue>,
        capture: NestedCapture,
    },
    CapturedResult(CommandResult),
    EndHook {
        name: String,
    },
}

struct SharedCommandExecution {
    result: CommandResult,
    insert_next: Vec<Vec<SharedQueueItem>>,
    defer_success_hooks: bool,
}

impl SharedCommandExecution {
    fn completed(result: CommandResult) -> Self {
        Self {
            result,
            insert_next: Vec::new(),
            defer_success_hooks: false,
        }
    }
}

pub(crate) struct ResumableCommandQueue {
    queue: queue::CommandQueue<SharedQueueItem>,
    out: CommandResult,
    agents: PaneAgents,
    context: ClientContext,
    suspended: Option<SuspendedCommand>,
    nested: Option<ActiveNestedCommand>,
}

pub(crate) enum ResumableCommandTurn {
    Pending,
    Suspended(CommandSuspension),
    Complete(CommandResult),
}

pub(crate) enum BackgroundCommandRequest {
    Ready {
        command: Option<String>,
        context: ClientContext,
    },
    ReadyArgs {
        args: Vec<String>,
        context: ClientContext,
    },
    IfShell {
        condition: String,
        then_command: Option<String>,
        else_command: Option<String>,
        context: ClientContext,
    },
    RunShell {
        args: Vec<String>,
        context: ClientContext,
        jobs: Rc<BackgroundJobRegistry>,
    },
}

/// What a background request runs, once it is known.
pub(crate) enum PendingBackground {
    Ready(BackgroundCommand, ClientContext),
    /// `if-shell -b`: the condition has to run before either branch is picked.
    Condition {
        condition: String,
        then_command: Option<String>,
        else_command: Option<String>,
        context: ClientContext,
    },
}

impl BackgroundCommandRequest {
    pub(crate) fn into_pending(self) -> PendingBackground {
        match self {
            Self::Ready { command, context } => {
                PendingBackground::Ready(BackgroundCommand::Line(command), context)
            }
            Self::ReadyArgs { args, context } => {
                PendingBackground::Ready(BackgroundCommand::Args(args), context)
            }
            Self::IfShell {
                condition,
                then_command,
                else_command,
                context,
            } => PendingBackground::Condition {
                condition,
                then_command,
                else_command,
                context,
            },
            Self::RunShell {
                args,
                context,
                jobs,
            } => PendingBackground::Ready(BackgroundCommand::RunShell { args, jobs }, context),
        }
    }
}

pub(crate) enum BackgroundCommand {
    Line(Option<String>),
    Args(Vec<String>),
    RunShell {
        args: Vec<String>,
        jobs: Rc<BackgroundJobRegistry>,
    },
}

struct SuspendedCommand {
    ticket: queue::QueueTicket,
    command: ParsedCommand,
    source: Option<SourceLocation>,
    source_depth: u8,
    contributes_status: bool,
}

struct ActiveNestedCommand {
    ticket: queue::QueueTicket,
    queue: Box<ResumableCommandQueue>,
    capture: NestedCapture,
}

#[derive(Clone, Copy)]
enum NestedCapture {
    Hook,
    Inserted,
    Discard,
}

pub(crate) enum CommandSuspension {
    RunShell {
        args: Vec<String>,
        context: ClientContext,
    },
    IfShell {
        condition: String,
        context: ClientContext,
    },
    SourceFile {
        paths: Vec<String>,
    },
    LoadBuffer {
        path: PathBuf,
    },
    SaveBuffer {
        request: ClientFileWrite,
    },
    WaitFor {
        args: Vec<String>,
        registry: Rc<WaitRegistry>,
    },
    CommandPrompt {
        args: Vec<String>,
        registry: Rc<super::state::ClientPromptRegistry>,
        target: Option<String>,
        tty_name: Option<String>,
        wait: bool,
    },
    ClientInteraction {
        completed: Completion<Option<PromptCompletion>>,
    },
    PaneOutput(PaneOutputSuspension),
}

pub(crate) enum CommandSuspensionResult {
    RunShell(RunShellCompletion),
    IfShell(bool),
    SourceFile(Vec<SourceFileRead>),
    LoadBuffer(Result<Vec<u8>, i32>),
    SaveBuffer(CommandResult),
    Completed(CommandResult),
}

pub(crate) struct SourceFileRead {
    path: String,
    contents: io::Result<String>,
    existed: bool,
}

impl SourceFileRead {
    /// What the suspended `source-file` will parse for this path.
    #[cfg(test)]
    pub(crate) fn contents(&self) -> Option<&str> {
        self.contents.as_deref().ok()
    }
}

pub(crate) struct PaneOutputSuspension {
    observation: Rc<super::pane::NativePaneObservation>,
    before: u64,
    subscription: super::pane::OutputSubscription,
    deadline: Instant,
    result: Option<CommandResult>,
}

impl PaneOutputSuspension {
    const OUTPUT_READY: WaitToken = WaitToken::new(0);

    fn new(
        observation: Rc<super::pane::NativePaneObservation>,
        before: u64,
        result: CommandResult,
    ) -> Result<Self, CommandResult> {
        let subscription = match observation.subscribe_output() {
            Ok(subscription) => subscription,
            Err(_) => return Err(result),
        };
        subscription.drain();
        Ok(Self {
            observation,
            before,
            subscription,
            deadline: Instant::now() + Duration::from_millis(10),
            result: Some(result),
        })
    }

    fn complete(&mut self) -> CommandSuspensionResult {
        self.subscription.drain();
        CommandSuspensionResult::Completed(
            self.result
                .take()
                .expect("pane-output suspension completed twice"),
        )
    }
}

impl Coroutine for PaneOutputSuspension {
    type Output = CommandSuspensionResult;

    fn wait(&self) -> WaitRequest<'_> {
        WaitRequest::new(
            vec![FdInterest::readable(
                Self::OUTPUT_READY,
                self.subscription.as_fd(),
            )],
            Some(self.deadline),
        )
    }

    fn resume(&mut self, ready: &ReadySet) -> TaskPoll<Self::Output> {
        let output_changed = self.observation.contract_revision() != self.before;
        let deadline_elapsed = Instant::now() >= self.deadline;
        if output_changed
            || deadline_elapsed
            || ready.contains(Self::OUTPUT_READY)
            || ready.timed_out()
        {
            TaskPoll::Ready(self.complete())
        } else {
            TaskPoll::Pending
        }
    }
}

/// Runtime operation used only for work that cannot be polled directly.
///
/// Implementations must return promptly. The returned completion descriptor
/// becomes readable once the runtime has resolved the suspension.
pub(crate) trait CommandRuntime {
    fn submit(
        &self,
        suspension: CommandSuspension,
    ) -> io::Result<Completion<CommandSuspensionResult>>;
}

/// One nonblocking command suspension, regardless of how it is implemented.
pub(crate) enum CommandTask {
    Readiness(PaneOutputSuspension),
    Runtime(Completion<CommandSuspensionResult>),
}

impl CommandTask {
    pub(crate) fn start(
        suspension: CommandSuspension,
        runtime: &dyn CommandRuntime,
    ) -> io::Result<Self> {
        match suspension {
            CommandSuspension::PaneOutput(wait) => Ok(Self::Readiness(wait)),
            suspension => runtime.submit(suspension).map(Self::Runtime),
        }
    }
}

impl Coroutine for CommandTask {
    type Output = io::Result<CommandSuspensionResult>;

    fn wait(&self) -> WaitRequest<'_> {
        match self {
            Self::Readiness(task) => task.wait(),
            Self::Runtime(task) => task.wait(),
        }
    }

    fn resume(&mut self, ready: &ReadySet) -> TaskPoll<Self::Output> {
        match self {
            Self::Readiness(task) => match task.resume(ready) {
                TaskPoll::Ready(result) => TaskPoll::Ready(Ok(result)),
                TaskPoll::Pending => TaskPoll::Pending,
            },
            Self::Runtime(task) => task.resume(ready),
        }
    }
}

/// Runtime-facing state for a command suspension.
pub(crate) struct CommandTaskState(TaskState<CommandTask>);

impl CommandTaskState {
    pub(crate) fn start(
        suspension: CommandSuspension,
        runtime: &dyn CommandRuntime,
    ) -> io::Result<Self> {
        Ok(Self(TaskState::new(CommandTask::start(
            suspension, runtime,
        )?)))
    }

    fn wait(&self) -> WaitRequest<'_> {
        self.0
            .wait()
            .expect("completed command task must not remain registered")
    }

    fn is_complete(&mut self) -> bool {
        self.0.poll(&ReadySet::default())
    }

    fn poll(&mut self, ready: &ReadySet) -> bool {
        self.0.poll(ready)
    }

    fn take_ready_result(&mut self) -> io::Result<CommandSuspensionResult> {
        self.0
            .take_output()
            .ok_or_else(|| io::Error::other("command task has not completed"))?
    }
}

/// A complete command queue represented as one runtime-neutral coroutine.
pub(crate) struct CommandCoroutine {
    queue: ResumableCommandQueue,
    state: SharedState,
    runtime: Rc<dyn CommandRuntime>,
    pending: Option<CommandTaskState>,
    pending_allows_attach_io: bool,
    budget: usize,
}

impl CommandCoroutine {
    pub(crate) fn new(
        queue: ResumableCommandQueue,
        state: SharedState,
        runtime: Rc<dyn CommandRuntime>,
        budget: usize,
    ) -> Self {
        Self {
            queue,
            state,
            runtime,
            pending: None,
            pending_allows_attach_io: false,
            budget,
        }
    }

    pub(crate) fn allows_attach_io(&self) -> bool {
        self.pending.is_some() && self.pending_allows_attach_io
    }
}

impl Coroutine for CommandCoroutine {
    type Output = io::Result<CommandResult>;

    fn wait(&self) -> WaitRequest<'_> {
        match &self.pending {
            Some(pending) => pending.wait(),
            None => WaitRequest::new(Vec::new(), Some(Instant::now())),
        }
    }

    fn resume(&mut self, ready: &ReadySet) -> TaskPoll<Self::Output> {
        if let Some(pending) = self.pending.as_mut() {
            if !pending.poll(ready) {
                return TaskPoll::Pending;
            }
            let result = match pending.take_ready_result() {
                Ok(result) => result,
                Err(error) => return TaskPoll::Ready(Err(error)),
            };
            self.pending = None;
            self.pending_allows_attach_io = false;
            self.queue.resume(result, &self.state);
        }

        loop {
            match self.queue.drive(&self.state, self.budget) {
                ResumableCommandTurn::Pending => return TaskPoll::Pending,
                ResumableCommandTurn::Suspended(suspension) => {
                    let allows_attach_io = suspension.allows_attach_io();
                    let mut pending =
                        match CommandTaskState::start(suspension, self.runtime.as_ref()) {
                            Ok(pending) => pending,
                            Err(error) => return TaskPoll::Ready(Err(error)),
                        };
                    if !pending.is_complete() {
                        self.pending = Some(pending);
                        self.pending_allows_attach_io = allows_attach_io;
                        return TaskPoll::Pending;
                    }
                    let result = match pending.take_ready_result() {
                        Ok(result) => result,
                        Err(error) => return TaskPoll::Ready(Err(error)),
                    };
                    self.queue.resume(result, &self.state);
                }
                ResumableCommandTurn::Complete(result) => {
                    return TaskPoll::Ready(Ok(result));
                }
            }
        }
    }
}

impl CommandSuspension {
    pub(crate) fn allows_attach_io(&self) -> bool {
        matches!(
            self,
            Self::CommandPrompt { .. } | Self::ClientInteraction { .. }
        )
    }
}

impl ResumableCommandQueue {
    fn new(parsed: Vec<ParsedCommand>, agents: &PaneAgents, context: &ClientContext) -> Self {
        let mut queue = queue::CommandQueue::new();
        queue.push_back_group(parsed.into_iter().map(|command| SharedQueueItem::Command {
            command,
            source: None,
            source_depth: 0,
            contributes_status: true,
        }));
        Self {
            queue,
            out: CommandResult::ok(""),
            agents: agents.clone(),
            context: context.clone(),
            suspended: None,
            nested: None,
        }
    }

    pub(crate) fn drive(&mut self, state: &SharedState, budget: usize) -> ResumableCommandTurn {
        for _ in 0..budget {
            if let Some(nested) = self.nested.as_mut() {
                match nested.queue.drive(state, 1) {
                    ResumableCommandTurn::Pending => continue,
                    ResumableCommandTurn::Suspended(suspension) => {
                        return ResumableCommandTurn::Suspended(suspension);
                    }
                    ResumableCommandTurn::Complete(result) => {
                        let active = self
                            .nested
                            .take()
                            .expect("completed nested command disappeared");
                        let stops_group = result.exit != 0 && !result.continue_queue;
                        self.capture_nested_result(result, active.capture);
                        self.queue
                            .complete(
                                active.ticket,
                                queue::QueueCompletion {
                                    discard_group_tail: stops_group,
                                    insert_next: Vec::new(),
                                },
                            )
                            .expect("nested command owns current queue ticket");
                        continue;
                    }
                }
            }
            let Some(started) = self.queue.start_next() else {
                return ResumableCommandTurn::Complete(std::mem::replace(
                    &mut self.out,
                    CommandResult::ok(""),
                ));
            };
            let ticket = started.ticket;
            let (command, source, source_depth, contributes_status) = match started.value {
                SharedQueueItem::Command {
                    command,
                    source,
                    source_depth,
                    contributes_status,
                } => (command, source, source_depth, contributes_status),
                SharedQueueItem::FinalizeSource { args } => {
                    let insert_next = self.plan_command_hooks("source-file", &args, state);
                    self.queue
                        .complete(
                            ticket,
                            queue::QueueCompletion {
                                discard_group_tail: false,
                                insert_next,
                            },
                        )
                        .expect("source finalizer owns current queue ticket");
                    continue;
                }
                SharedQueueItem::FinalizeHooks { command, args } => {
                    let insert_next = self.plan_command_hooks(command, &args, state);
                    self.queue
                        .complete(
                            ticket,
                            queue::QueueCompletion {
                                discard_group_tail: false,
                                insert_next,
                            },
                        )
                        .expect("hook finalizer owns current queue ticket");
                    continue;
                }
                SharedQueueItem::NestedCommand { queue, capture } => {
                    self.nested = Some(ActiveNestedCommand {
                        ticket,
                        queue,
                        capture,
                    });
                    continue;
                }
                SharedQueueItem::CapturedResult(result) => {
                    let stops_group = result.exit != 0 && !result.continue_queue;
                    self.capture_nested_result(result, NestedCapture::Hook);
                    self.queue
                        .complete(
                            ticket,
                            queue::QueueCompletion {
                                discard_group_tail: stops_group,
                                insert_next: Vec::new(),
                            },
                        )
                        .expect("captured result owns current queue ticket");
                    continue;
                }
                SharedQueueItem::EndHook { name } => {
                    {
                        let mut state = state.borrow_mut();
                        state.end_hook(&name);
                        state.record_control_checkpoint();
                    }
                    self.queue
                        .complete(ticket, queue::QueueCompletion::done())
                        .expect("hook finalizer owns current queue ticket");
                    continue;
                }
            };

            {
                let mut state = {
                    let state = state.borrow_mut();
                    state
                };
                state.add_message(format!(
                    "{} command: {}",
                    self.context
                        .client_pid
                        .map(|pid| format!("client-{pid}"))
                        .unwrap_or_else(|| "client-unknown".to_string()),
                    display_command(&command.args)
                ));
            }

            let inflight = SuspendedCommand {
                ticket,
                command,
                source,
                source_depth,
                contributes_status,
            };
            if inflight.command.spec.name == "run-shell" && has_flag(&inflight.command.args, "-b") {
                let jobs = {
                    let state = state.borrow_mut();
                    state.background_job_registry()
                };
                let mut result = CommandResult::ok("");
                result
                    .background_commands
                    .push(BackgroundCommandRequest::RunShell {
                        args: {
                            let state = state.borrow_mut();
                            pin_run_shell_view_target(&inflight.command.args, &state)
                        },
                        context: self.context.clone(),
                        jobs,
                    });
                self.finish_execution(inflight, SharedCommandExecution::completed(result), state);
                continue;
            }
            if inflight.command.spec.name == "run-shell"
                && !has_flag(&inflight.command.args, "-b")
                && !has_flag(&inflight.command.args, "-C")
            {
                let suspension = CommandSuspension::RunShell {
                    args: {
                        let state = state.borrow_mut();
                        pin_run_shell_view_target(&inflight.command.args, &state)
                    },
                    context: {
                        let state = state.borrow_mut();
                        self.context.with_job_environment(&state)
                    },
                };
                self.suspended = Some(inflight);
                return ResumableCommandTurn::Suspended(suspension);
            }
            if inflight.command.spec.name == "if-shell" && has_flag(&inflight.command.args, "-b") {
                let positionals = positionals(&inflight.command.args, &["-t"]);
                let Some(condition) = positionals.first().copied() else {
                    self.finish_execution(
                        inflight,
                        SharedCommandExecution::completed(CommandResult::err(
                            "if-shell: too few arguments\n",
                        )),
                        state,
                    );
                    continue;
                };
                let then_command = positionals.get(1).map(|command| (*command).to_string());
                let else_command = positionals.get(2).map(|command| (*command).to_string());
                let request = if has_flag(&inflight.command.args, "-F") {
                    let matched = {
                        let mut state = state.borrow_mut();
                        let previous = install_command_target_context(&mut state, &self.context);
                        let expanded =
                            expand_if_cond(condition, &inflight.command.args, &state, &self.agents);
                        restore_command_target_context(&mut state, previous);
                        !expanded.is_empty() && expanded != "0"
                    };
                    BackgroundCommandRequest::Ready {
                        command: if matched { then_command } else { else_command },
                        context: self.context.clone(),
                    }
                } else {
                    BackgroundCommandRequest::IfShell {
                        condition: condition.to_string(),
                        then_command,
                        else_command,
                        context: self.context.clone(),
                    }
                };
                let mut result = CommandResult::ok("");
                result.background_commands.push(request);
                self.finish_execution(inflight, SharedCommandExecution::completed(result), state);
                continue;
            }
            if inflight.command.spec.name == "run-shell"
                && !has_flag(&inflight.command.args, "-b")
                && has_flag(&inflight.command.args, "-C")
            {
                let command = positionals(&inflight.command.args, &["-t", "-c", "-d"])
                    .first()
                    .copied();
                let execution = match command {
                    Some(command) => {
                        match self.plan_nested_command_line(command, state, NestedCapture::Inserted)
                        {
                            Ok(commands) => {
                                let mut insert_next = Vec::new();
                                if !commands.is_empty() {
                                    insert_next.push(commands);
                                }
                                insert_next.push(vec![SharedQueueItem::FinalizeHooks {
                                    command: "run-shell",
                                    args: inflight.command.args.clone(),
                                }]);
                                SharedCommandExecution {
                                    result: CommandResult::ok(""),
                                    insert_next,
                                    defer_success_hooks: true,
                                }
                            }
                            Err(result) => SharedCommandExecution::completed(result),
                        }
                    }
                    None => SharedCommandExecution::completed(CommandResult::ok("")),
                };
                self.finish_execution(inflight, execution, state);
                continue;
            }
            if inflight.command.spec.name == "if-shell" && !has_flag(&inflight.command.args, "-b") {
                let positionals = positionals(&inflight.command.args, &["-t"]);
                let Some(condition) = positionals.first().copied() else {
                    self.finish_execution(
                        inflight,
                        SharedCommandExecution::completed(CommandResult::err(
                            "if-shell: too few arguments\n",
                        )),
                        state,
                    );
                    continue;
                };
                if has_flag(&inflight.command.args, "-F") {
                    let matched = {
                        let mut state = state.borrow_mut();
                        let previous = install_command_target_context(&mut state, &self.context);
                        let expanded =
                            expand_if_cond(condition, &inflight.command.args, &state, &self.agents);
                        restore_command_target_context(&mut state, previous);
                        !expanded.is_empty() && expanded != "0"
                    };
                    let execution =
                        self.plan_if_shell_branch(&inflight.command.args, matched, state);
                    self.finish_execution(inflight, execution, state);
                    continue;
                }
                let suspension = CommandSuspension::IfShell {
                    condition: condition.to_string(),
                    context: {
                        let state = state.borrow_mut();
                        self.context.with_job_environment(&state)
                    },
                };
                self.suspended = Some(inflight);
                return ResumableCommandTurn::Suspended(suspension);
            }
            if inflight.command.spec.name == "source-file" {
                if source_depth >= 50 {
                    self.finish_execution(
                        inflight,
                        SharedCommandExecution::completed(CommandResult::err(
                            "too many nested files\n",
                        )),
                        state,
                    );
                    continue;
                }
                let paths = match prepare_source_file_paths(
                    &inflight.command.args,
                    state,
                    &self.agents,
                    &self.context,
                ) {
                    Ok(paths) => paths,
                    Err(result) => {
                        self.finish_execution(
                            inflight,
                            SharedCommandExecution::completed(result),
                            state,
                        );
                        continue;
                    }
                };
                self.suspended = Some(inflight);
                return ResumableCommandTurn::Suspended(CommandSuspension::SourceFile { paths });
            }
            if inflight.command.spec.name == "load-buffer" && self.context.input_file.is_none() {
                if let Some(path) = load_buffer_client_path(&inflight.command.args, &self.context) {
                    self.suspended = Some(inflight);
                    return ResumableCommandTurn::Suspended(CommandSuspension::LoadBuffer { path });
                }
            }
            if inflight.command.spec.name == "save-buffer" {
                let request = {
                    let state = state.borrow_mut();
                    save_buffer_client_request(&inflight.command.args, &state, &self.context)
                };
                if let Some(request) = request {
                    match request {
                        Ok(request) => {
                            self.suspended = Some(inflight);
                            return ResumableCommandTurn::Suspended(
                                CommandSuspension::SaveBuffer { request },
                            );
                        }
                        Err(result) => {
                            self.finish_execution(
                                inflight,
                                SharedCommandExecution::completed(result),
                                state,
                            );
                            continue;
                        }
                    }
                }
            }
            if inflight.command.spec.name == "wait-for" {
                let registry = {
                    let state = state.borrow_mut();
                    state.wait_registry()
                };
                let args = inflight.command.args.clone();
                self.suspended = Some(inflight);
                return ResumableCommandTurn::Suspended(CommandSuspension::WaitFor {
                    args,
                    registry,
                });
            }
            if inflight.command.spec.name == "command-prompt"
                && self.context.wait_for_interactions()
            {
                if let Err(error) = command_prompt_spec(&inflight.command.args) {
                    self.finish_execution(
                        inflight,
                        SharedCommandExecution::completed(CommandResult::err(error)),
                        state,
                    );
                    continue;
                }
                let registry = {
                    let state = state.borrow_mut();
                    state.client_prompt_registry()
                };
                let suspension = CommandSuspension::CommandPrompt {
                    target: command_prompt_target(&inflight.command.args),
                    tty_name: self.context.tty_name.clone(),
                    wait: command_prompt_waits(&inflight.command.args),
                    args: inflight.command.args.clone(),
                    registry,
                };
                self.suspended = Some(inflight);
                return ResumableCommandTurn::Suspended(suspension);
            }
            if self.context.wait_for_interactions()
                && client_interaction_waits(inflight.command.spec.name, &inflight.command.args)
            {
                let (reply, completed) = match PromptReply::new() {
                    Ok(reply) => reply,
                    Err(error) => {
                        self.finish_execution(
                            inflight,
                            SharedCommandExecution::completed(CommandResult::err(format!(
                                "{error}\n"
                            ))),
                            state,
                        );
                        continue;
                    }
                };
                let mut interaction_context = self.context.clone();
                interaction_context.interaction_reply = Some(reply);
                let initial =
                    run_single_shared(&inflight.command, state, &self.agents, &interaction_context);
                if initial.exit != 0 {
                    self.finish_execution(
                        inflight,
                        SharedCommandExecution::completed(initial),
                        state,
                    );
                    continue;
                }
                self.suspended = Some(inflight);
                return ResumableCommandTurn::Suspended(CommandSuspension::ClientInteraction {
                    completed,
                });
            }
            if inflight.command.spec.name == "set-hook" && has_flag(&inflight.command.args, "-R") {
                let hook = positionals(&inflight.command.args, &["-t"])
                    .first()
                    .copied();
                let execution = match hook {
                    None => SharedCommandExecution::completed(CommandResult::err(
                        "set-hook: missing hook\n",
                    )),
                    Some(hook) if !options::is_hook(hook) => SharedCommandExecution::completed(
                        CommandResult::err(format!("invalid option: {hook}\n")),
                    ),
                    Some(hook) => {
                        let mut insert_next = self.plan_hook_with_capture(
                            hook,
                            flag_value(&inflight.command.args, "-t"),
                            vec![("hook".to_string(), hook.to_string())],
                            state,
                            NestedCapture::Discard,
                            HookOrigin::Command,
                        );
                        insert_next.push(vec![SharedQueueItem::FinalizeHooks {
                            command: "set-hook",
                            args: inflight.command.args.clone(),
                        }]);
                        SharedCommandExecution {
                            result: CommandResult::ok(""),
                            insert_next,
                            defer_success_hooks: true,
                        }
                    }
                };
                self.finish_execution(inflight, execution, state);
                continue;
            }
            let command = &inflight.command;

            let mut execution = match command.spec.name {
                "load-buffer" => SharedCommandExecution::completed(run_load_buffer_shared(
                    &command,
                    state,
                    &self.agents,
                    &self.context,
                )),
                "save-buffer" => SharedCommandExecution::completed(run_save_buffer_shared(
                    &command,
                    state,
                    &self.agents,
                    &self.context,
                )),
                _ => SharedCommandExecution::completed(run_single_shared(
                    &command,
                    state,
                    &self.agents,
                    &self.context,
                )),
            };
            if !execution.result.deferred_commands.is_empty() {
                let commands = std::mem::take(&mut execution.result.deferred_commands);
                match self.plan_deferred_commands(commands, state) {
                    Ok(commands) => {
                        if !commands.is_empty() {
                            execution.insert_next.push(commands);
                        }
                        execution
                            .insert_next
                            .push(vec![SharedQueueItem::FinalizeHooks {
                                command: inflight.command.spec.name,
                                args: inflight.command.args.clone(),
                            }]);
                        execution.defer_success_hooks = true;
                    }
                    Err(result) => execution.result = result,
                }
            }
            if let Some((observation, before)) = execution.result.pane_output_wait.take() {
                match PaneOutputSuspension::new(observation, before, execution.result) {
                    Ok(wait) => {
                        self.suspended = Some(inflight);
                        return ResumableCommandTurn::Suspended(CommandSuspension::PaneOutput(
                            wait,
                        ));
                    }
                    Err(result) => {
                        execution.result = result;
                    }
                }
            }

            self.finish_execution(inflight, execution, state);
        }
        ResumableCommandTurn::Pending
    }

    fn plan_if_shell_branch(
        &self,
        args: &[String],
        matched: bool,
        state: &SharedState,
    ) -> SharedCommandExecution {
        let positionals = positionals(args, &["-t"]);
        let branch = if matched {
            positionals.get(1)
        } else {
            positionals.get(2)
        };
        let Some(branch) = branch else {
            return SharedCommandExecution::completed(CommandResult::ok(""));
        };
        let commands = match self.plan_nested_command_line(branch, state, NestedCapture::Inserted) {
            Ok(commands) => commands,
            Err(error) => return SharedCommandExecution::completed(error),
        };
        let mut insert_next = Vec::new();
        if !commands.is_empty() {
            insert_next.push(commands);
        }
        insert_next.push(vec![SharedQueueItem::FinalizeHooks {
            command: "if-shell",
            args: args.to_vec(),
        }]);
        SharedCommandExecution {
            result: CommandResult::ok(""),
            insert_next,
            defer_success_hooks: true,
        }
    }

    fn plan_nested_command_line(
        &self,
        line: &str,
        state: &SharedState,
        capture: NestedCapture,
    ) -> Result<Vec<SharedQueueItem>, CommandResult> {
        let tokens = tokenize_line(line);
        let owned_groups = tokenized_command_groups(&tokens);
        let groups = owned_groups.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let aliases = {
            let state = state.borrow_mut();
            state.command_aliases()
        };
        let parsed =
            parse_command_groups_with_aliases(groups, &aliases).map_err(|mut result| {
                result.continue_queue = true;
                result
            })?;
        let mut nested_context = self.context.clone();
        if matches!(capture, NestedCapture::Inserted | NestedCapture::Hook) {
            nested_context.nested_granularity = true;
        }
        if matches!(capture, NestedCapture::Hook) {
            nested_context.suppress_after_hooks = true;
        }
        Ok(parsed
            .into_iter()
            .map(|command| SharedQueueItem::NestedCommand {
                queue: Box::new(ResumableCommandQueue::new(
                    vec![command],
                    &self.agents,
                    &nested_context,
                )),
                capture,
            })
            .collect())
    }

    fn plan_deferred_commands(
        &self,
        commands: Vec<DeferredCommand>,
        state: &SharedState,
    ) -> Result<Vec<SharedQueueItem>, CommandResult> {
        let aliases = {
            let state = state.borrow_mut();
            state.command_aliases()
        };
        let mut planned = Vec::new();
        for command in commands {
            let parsed = match command {
                DeferredCommand::Args(args) => {
                    parse_command_groups_with_aliases(vec![args.as_slice()], &aliases)?
                }
                DeferredCommand::Line { line, tail } => {
                    let tokens = tokenize_line(&line);
                    let owned_groups = tokenized_command_groups(&tokens);
                    let groups = owned_groups.iter().map(Vec::as_slice).collect::<Vec<_>>();
                    let mut parsed = parse_command_groups_with_aliases(groups, &aliases)?;
                    if !tail.is_empty() {
                        let expanded_tail = expand_attached_separators(&tail);
                        parsed.extend(parse_command_groups_with_aliases(
                            split_commands(&expanded_tail),
                            &aliases,
                        )?);
                    }
                    parsed
                }
            };
            planned.extend(parsed.into_iter().map(|command| SharedQueueItem::Command {
                command,
                source: None,
                source_depth: 0,
                contributes_status: true,
            }));
        }
        Ok(planned)
    }

    pub(crate) fn resume(&mut self, result: CommandSuspensionResult, state: &SharedState) {
        if let Some(nested) = self.nested.as_mut() {
            nested.queue.resume(result, state);
            return;
        }
        let inflight = self
            .suspended
            .take()
            .expect("command suspension completed without an active queue item");
        let execution = match result {
            CommandSuspensionResult::RunShell(completion) => {
                let result = {
                    let mut state = state.borrow_mut();
                    let previous = install_command_target_context(&mut state, &self.context);
                    let result = finish_run_shell(completion, &mut state);
                    state.record_control_checkpoint();
                    restore_command_target_context(&mut state, previous);
                    result
                };
                SharedCommandExecution::completed(result)
            }
            CommandSuspensionResult::IfShell(matched) => {
                self.plan_if_shell_branch(&inflight.command.args, matched, state)
            }
            CommandSuspensionResult::SourceFile(reads) => plan_source_file_completion(
                &inflight.command.args,
                inflight.source_depth,
                reads,
                state,
            ),
            CommandSuspensionResult::LoadBuffer(input_file) => {
                let deferred_error = input_file.is_err();
                let mut context = self.context.clone();
                context.input_file = Some(input_file);
                let mut result =
                    run_single_shared(&inflight.command, state, &self.agents, &context);
                result.continue_queue |= deferred_error && result.exit != 0;
                SharedCommandExecution::completed(result)
            }
            CommandSuspensionResult::SaveBuffer(result) => {
                SharedCommandExecution::completed(result)
            }
            CommandSuspensionResult::Completed(result) => SharedCommandExecution::completed(result),
        };
        self.finish_execution(inflight, execution, state);
    }

    fn finish_execution(
        &mut self,
        inflight: SuspendedCommand,
        mut execution: SharedCommandExecution,
        state: &SharedState,
    ) {
        let command = &inflight.command;
        let exit = execution.result.exit;
        if exit != 0 {
            if let Some(source) = &inflight.source {
                let diagnostic = execution.result.stderr.trim_end();
                let location = format!("{}:{}", source.path, source.line);
                let diagnostic = if matches!(command.spec.name, "if" | "if-shell") {
                    format!("{location}: {location}: {diagnostic}")
                } else {
                    format!("{location}: {diagnostic}")
                };
                {
                    let mut state = state.borrow_mut();
                    state.push_config_error(diagnostic);
                }
            }
        }

        let inserted = std::mem::take(&mut execution.result.inserted_results);
        self.out
            .background_commands
            .append(&mut execution.result.background_commands);
        let stops_group = exit != 0 && !execution.result.continue_queue;
        if !self.context.suppress_after_hooks {
            if stops_group {
                execution.insert_next.extend(self.plan_hook(
                    "command-error",
                    flag_value(&command.args, "-t"),
                    hook_command_vars("command-error", command.spec.name, &command.args),
                    state,
                ));
            } else if !execution.defer_success_hooks {
                execution.insert_next.extend(self.plan_command_hooks(
                    command.spec.name,
                    &command.args,
                    state,
                ));
            }
        }
        // tmux inserts a command's after-hook directly behind it but *appends*
        // the notifications its mutations raised, so the after-hook runs first
        // — and a hook body's own mutations still notify.
        execution.insert_next.extend(self.plan_notifications(state));
        if inflight.contributes_status {
            self.out.continue_queue |= execution.result.continue_queue;
        }
        self.out.append_stdout(&execution.result);
        self.out.stderr.push_str(&execution.result.stderr);
        if inflight.contributes_status && (self.out.exit == 0 || exit != 0) {
            self.out.exit = exit;
        }
        if self.context.preserve_queue_insertions() {
            self.out.inserted_results.extend(inserted);
        } else {
            for inserted in inserted {
                self.out.append_stdout(&inserted);
                self.out.stderr.push_str(&inserted.stderr);
                if inserted.exit != 0 {
                    self.out.exit = inserted.exit;
                }
            }
        }

        self.queue
            .complete(
                inflight.ticket,
                queue::QueueCompletion {
                    discard_group_tail: stops_group,
                    insert_next: execution.insert_next,
                },
            )
            .expect("command owns current queue ticket");
    }

    fn plan_command_hooks(
        &self,
        command: &str,
        args: &[String],
        state: &SharedState,
    ) -> Vec<Vec<SharedQueueItem>> {
        if self.context.suppress_after_hooks {
            return Vec::new();
        }
        let after = format!("after-{command}");
        let vars = hook_command_vars(&after, command, args);
        self.plan_hook(&after, flag_value(args, "-t"), vars, state)
    }

    /// Turn every notification raised while the last command ran into hook
    /// bodies, in the order the mutations happened.
    fn plan_notifications(&self, state: &SharedState) -> Vec<Vec<SharedQueueItem>> {
        if self.context.suppress_notifications {
            return Vec::new();
        }
        let notifications = {
            let mut state = state.borrow_mut();
            state.take_notifications()
        };
        notifications
            .into_iter()
            .flat_map(|notification| {
                self.plan_hook_with_capture(
                    &notification.name,
                    notification.target.as_deref(),
                    notification.vars,
                    state,
                    NestedCapture::Hook,
                    HookOrigin::Event,
                )
            })
            .collect()
    }

    fn plan_hook(
        &self,
        hook: &str,
        requested_target: Option<&str>,
        vars: Vec<(String, String)>,
        state: &SharedState,
    ) -> Vec<Vec<SharedQueueItem>> {
        self.plan_hook_with_capture(
            hook,
            requested_target,
            vars,
            state,
            NestedCapture::Hook,
            HookOrigin::Command,
        )
    }

    fn plan_hook_with_capture(
        &self,
        hook: &str,
        requested_target: Option<&str>,
        vars: Vec<(String, String)>,
        state: &SharedState,
        capture: NestedCapture,
        origin: HookOrigin,
    ) -> Vec<Vec<SharedQueueItem>> {
        let (commands, aliases) = {
            let mut state = {
                let state = state.borrow_mut();
                state
            };
            let previous = install_command_target_context(&mut state, &self.context);
            let commands = hook_commands(hook, requested_target, &mut state, origin);
            restore_command_target_context(&mut state, previous);
            let Some(commands) = commands else {
                return Vec::new();
            };
            (commands, state.command_aliases())
        };

        let mut hook_context = self.context.clone();
        // A hook body resolves an untargeted command against the hook's own
        // target, not the server's current one; a hook without a target of its
        // own stays in the enclosing hook's scope.
        hook_context.hook = Some(HookScope {
            vars: Rc::new(vars),
            target: requested_target.map(Rc::from).or_else(|| {
                self.context
                    .hook
                    .as_ref()
                    .and_then(|hook| hook.target.clone())
            }),
        });
        if matches!(origin, HookOrigin::Event) {
            // tmux runs an event hook's body on the global queue with
            // `CMDQ_STATE_NOHOOKS`, which is exactly what makes `notify_add`
            // drop anything the body itself raises.
            hook_context.suppress_notifications = true;
        }
        if matches!(capture, NestedCapture::Hook) {
            hook_context.suppress_after_hooks = true;
            hook_context.nested_granularity = true;
        }
        let mut groups = Vec::new();
        for line in commands {
            let tokens = tokenize_line(&line);
            let owned_groups = tokenized_command_groups(&tokens);
            let command_groups = owned_groups.iter().map(Vec::as_slice).collect::<Vec<_>>();
            match parse_command_groups_with_aliases(command_groups, &aliases) {
                Ok(parsed) if !parsed.is_empty() => {
                    groups.push(
                        parsed
                            .into_iter()
                            .map(|command| SharedQueueItem::NestedCommand {
                                queue: Box::new(ResumableCommandQueue::new(
                                    vec![command],
                                    &self.agents,
                                    &hook_context,
                                )),
                                capture,
                            })
                            .collect(),
                    );
                }
                Ok(_) => {}
                Err(mut result) => {
                    result.continue_queue = true;
                    groups.push(vec![SharedQueueItem::CapturedResult(result)]);
                }
            }
        }
        if matches!(origin, HookOrigin::Command) {
            groups.push(vec![SharedQueueItem::EndHook {
                name: hook.to_string(),
            }]);
        }
        groups
    }

    fn capture_nested_result(&mut self, mut result: CommandResult, capture: NestedCapture) {
        self.out
            .background_commands
            .append(&mut result.background_commands);
        match capture {
            NestedCapture::Hook => {
                if result.inserted_results.is_empty()
                    && (result.exit != 0
                        || !result.stdout_data().is_empty()
                        || !result.stderr.is_empty())
                {
                    result.control_flags = 0;
                    self.merge_inserted_result(result);
                    return;
                }
                for mut inserted in std::mem::take(&mut result.inserted_results) {
                    inserted.control_flags = 0;
                    self.merge_inserted_result(inserted);
                }
            }
            NestedCapture::Inserted => {
                let nested = std::mem::take(&mut result.inserted_results);
                self.merge_inserted_result(result);
                for inserted in nested {
                    self.merge_inserted_result(inserted);
                }
            }
            NestedCapture::Discard => {}
        }
    }

    fn merge_inserted_result(&mut self, result: CommandResult) {
        if self.context.preserve_queue_insertions() {
            self.out.inserted_results.push(result);
        } else {
            self.out.append_stdout(&result);
            self.out.stderr.push_str(&result.stderr);
            if result.exit != 0 {
                self.out.exit = result.exit;
            }
        }
    }
}

/// Execute a command list containing a blocking command without holding the
/// server state mutex while it waits. Other commands still run one at a time
/// with the same client/session context and hook behavior.
fn client_interaction_waits(name: &str, args: &[String]) -> bool {
    match name {
        "confirm-before" | "display-panes" => !has_flag(args, "-b"),
        "display-menu" => true,
        "display-popup" => !has_flag(args, "-C"),
        _ => false,
    }
}

fn interaction_completion_result(completion: PromptCompletion) -> CommandResult {
    let mut completed = CommandResult {
        stdout: completion.stdout,
        stdout_bytes: Vec::new(),
        stderr: completion.stderr,
        exit: completion.exit,
        pane_output_wait: None,
        deferred_commands: Vec::new(),
        background_commands: Vec::new(),
        continue_queue: true,
        inserted_results: Vec::new(),
        control_flags: 1,
    };
    if completion.inserted {
        let mut original = CommandResult::ok("");
        original.inserted_results.push(completed);
        original
    } else {
        completed.continue_queue = true;
        completed
    }
}

fn run_single_shared(
    command: &ParsedCommand,
    state: &SharedState,
    agents: &PaneAgents,
    context: &ClientContext,
) -> CommandResult {
    let mut state = {
        let state = state.borrow_mut();
        state
    };
    let previous = install_command_target_context(&mut state, context);
    let result = run_single(
        command.spec.command,
        &command.args,
        &mut state,
        agents,
        context,
    );
    state.record_control_checkpoint();
    restore_command_target_context(&mut state, previous);
    result
}

fn prepare_source_file_paths(
    args: &[String],
    state: &SharedState,
    agents: &PaneAgents,
    context: &ClientContext,
) -> Result<Vec<String>, CommandResult> {
    positionals(args, &["-t"])
        .into_iter()
        .map(|raw_path| {
            if !has_bool_flag(args, 'F') {
                return Ok(raw_path.to_string());
            }
            let mut state = state.borrow_mut();
            let previous = install_command_target_context(&mut state, context);
            let path = expand_if_cond(raw_path, args, &state, agents);
            restore_command_target_context(&mut state, previous);
            Ok(path)
        })
        .collect()
}

fn plan_source_file_completion(
    args: &[String],
    source_depth: u8,
    reads: Vec<SourceFileRead>,
    state: &SharedState,
) -> SharedCommandExecution {
    let quiet = has_flag(args, "-q");
    let parse_only = has_flag(args, "-n");
    let verbose = has_flag(args, "-v");
    let mut out = CommandResult::ok("");
    let mut insert_next = Vec::new();
    for SourceFileRead {
        path,
        contents,
        existed,
    } in reads
    {
        match contents {
            Ok(contents) => {
                let mut file_insertions = Vec::new();
                let mut file_parse_error = false;
                let environment = {
                    let state = state.borrow_mut();
                    state
                        .env_iter()
                        .map(|(name, value)| (name.clone(), value.clone()))
                        .collect::<BTreeMap<_, _>>()
                };
                let parsed = match source_lines(&contents, &environment) {
                    Ok(parsed) => parsed,
                    Err((line, message)) => {
                        out.stderr.push_str(&format!("{path}:{line}: {message}\n"));
                        out.exit = 1;
                        out.continue_queue = true;
                        continue;
                    }
                };
                if !parse_only {
                    {
                        let mut state = state.borrow_mut();
                        for (name, value, hidden) in &parsed.assignments {
                            if *hidden {
                                state.set_hidden_env(name, value);
                            } else {
                                state.set_env(name, value);
                            }
                        }
                    }
                }
                for (line_number, line) in parsed.lines {
                    if verbose {
                        out.stdout.push_str(&format!(
                            "{path}:{line_number}: {}\n",
                            source_verbose_line(&line)
                        ));
                    }
                    let owned_groups = tokenized_command_groups(&line);
                    let groups = owned_groups.iter().map(Vec::as_slice).collect::<Vec<_>>();
                    let parsed = if parse_only {
                        parse_command_groups(groups)
                    } else {
                        let aliases = {
                            let state = state.borrow_mut();
                            state.command_aliases()
                        };
                        parse_command_groups_with_aliases(groups, &aliases)
                    };
                    match parsed {
                        Ok(parsed) if !parse_only && !parsed.is_empty() => {
                            let source_line = line_number;
                            file_insertions.push(
                                parsed
                                    .into_iter()
                                    .map(|command| SharedQueueItem::Command {
                                        command,
                                        source: Some(SourceLocation {
                                            path: path.clone(),
                                            line: source_line,
                                        }),
                                        source_depth: source_depth + 1,
                                        contributes_status: true,
                                    })
                                    .collect(),
                            );
                        }
                        Ok(_) => {}
                        Err(result) => {
                            file_parse_error = true;
                            let diagnostic = result.stderr.trim_end();
                            let location = format!("{path}:{line_number}");
                            let diagnostic = if matches!(
                                line.iter().find_map(LineToken::word),
                                Some("if" | "if-shell")
                            ) {
                                format!("{location}: {location}: {diagnostic}")
                            } else {
                                format!("{location}: {diagnostic}")
                            };
                            if parse_only {
                                out.stdout.push_str(&diagnostic);
                                out.stdout.push('\n');
                                out.exit = 1;
                                out.continue_queue = true;
                            } else {
                                out.append_stdout(&result);
                                out.stderr.push_str(&result.stderr);
                                out.exit = 1;
                                out.continue_queue = true;
                                {
                                    let mut state = state.borrow_mut();
                                    state.push_config_error(diagnostic);
                                }
                            }
                        }
                    }
                }
                if !parse_only && !file_parse_error {
                    insert_next.extend(file_insertions);
                }
            }
            Err(_) if quiet => {}
            Err(error) => {
                out.stderr
                    .push_str(&format!("{}: {}\n", io_error_message(&error), path));
                out.exit = 1;
                // tmux has already returned WAIT once glob expansion found the
                // path. A later read failure updates the client status, then
                // resumes the original queue group.
                out.continue_queue |= existed;
            }
        }
    }
    let defer_success_hooks = !parse_only && (out.exit == 0 || out.continue_queue);
    if defer_success_hooks {
        insert_next.push(vec![SharedQueueItem::FinalizeSource {
            args: args.to_vec(),
        }]);
    }
    SharedCommandExecution {
        result: out,
        insert_next,
        defer_success_hooks,
    }
}

fn run_load_buffer_shared(
    command: &ParsedCommand,
    state: &SharedState,
    agents: &PaneAgents,
    context: &ClientContext,
) -> CommandResult {
    let deferred_error = context.input_file.as_ref().is_some_and(Result::is_err);
    let mut result = run_single_shared(command, state, agents, context);
    result.continue_queue |= deferred_error && result.exit != 0;
    result
}

fn run_save_buffer_shared(
    command: &ParsedCommand,
    state: &SharedState,
    agents: &PaneAgents,
    context: &ClientContext,
) -> CommandResult {
    run_single_shared(command, state, agents, context)
}

fn tokenized_command_groups(tokens: &[LineToken]) -> Vec<Vec<String>> {
    let mut owned_groups: Vec<Vec<String>> = Vec::new();
    let mut group = Vec::new();
    for token in tokens {
        match token {
            LineToken::Word(word) => group.push(word.clone()),
            LineToken::Separator => {
                if !group.is_empty() {
                    owned_groups.push(std::mem::take(&mut group));
                }
            }
        }
    }
    if !group.is_empty() {
        owned_groups.push(group);
    }
    owned_groups
}

struct ParsedCommand {
    spec: &'static CommandSpec,
    args: Vec<String>,
}

pub(crate) fn display_command(args: &[String]) -> String {
    args.iter()
        .map(|argument| {
            if argument.is_empty()
                || argument
                    .chars()
                    .any(|character| character.is_whitespace() || "'\";{}[]$".contains(character))
            {
                format!("\"{}\"", argument.replace('"', "\\\""))
            } else {
                argument.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The `hook*` format variables a command hook body sees: the hook's name and
/// the triggering command's arguments and flags, as tmux publishes them.
fn hook_command_vars(hook: &str, command: &str, args: &[String]) -> Vec<(String, String)> {
    let mut vars = vec![("hook".to_string(), hook.to_string())];
    let arguments = args.get(1..).unwrap_or_default().join(" ");
    vars.push(("hook_arguments".to_string(), arguments));
    let spec = registry::getopt(command).unwrap_or("");
    let mut argument_index = 0;
    let mut in_flags = true;
    let mut i = 1;
    while i < args.len() {
        let argument = args[i].as_str();
        if in_flags && argument == "--" {
            in_flags = false;
            i += 1;
            continue;
        }
        let flag = argument
            .strip_prefix('-')
            .and_then(|rest| {
                let mut rest = rest.chars();
                match (rest.next(), rest.next()) {
                    (Some(letter), None) if letter.is_ascii_alphanumeric() => Some(letter),
                    _ => None,
                }
            })
            .filter(|_| in_flags);
        match flag {
            Some(letter) => {
                if registry::flag_kind(spec, letter) == Some(true) && i + 1 < args.len() {
                    vars.push((format!("hook_flag_{letter}"), args[i + 1].clone()));
                    i += 2;
                } else {
                    vars.push((format!("hook_flag_{letter}"), "1".to_string()));
                    i += 1;
                }
            }
            None => {
                in_flags = false;
                vars.push((
                    format!("hook_argument_{argument_index}"),
                    argument.to_string(),
                ));
                argument_index += 1;
                i += 1;
            }
        }
    }
    vars
}

/// The `after-*` hook of a command the client file protocol completed outside
/// the command queue (`save-buffer` to a client-side path), plus whatever it
/// raised, for the loop to run as detached queues.
pub(crate) fn take_client_file_after_hooks(
    args: &[String],
    st: &mut ServerState,
    context: &ClientContext,
) -> Vec<BackgroundCommandRequest> {
    let Some(name) = args.first() else {
        return Vec::new();
    };
    let Resolution::Name(name) = registry::resolve(name) else {
        return Vec::new();
    };
    let normalized = normalize_argv(name, args);
    let hook = format!("after-{name}");
    let vars = hook_command_vars(&hook, name, &normalized);
    let mut requests = Vec::new();
    push_event_hook(
        &hook,
        flag_value(&normalized, "-t"),
        vars,
        st,
        context,
        &mut requests,
    );
    if !context.suppress_notifications {
        for notification in st.take_notifications() {
            push_event_hook(
                &notification.name,
                notification.target.as_deref(),
                notification.vars,
                st,
                context,
                &mut requests,
            );
        }
    }
    requests
}

/// The bodies of notifications raised outside the command queue — a pane
/// exiting, an alert firing. tmux dispatches these from its global command
/// queue; hmux hands them to the loop as detached queues of their own, so a
/// body that has to wait for a shell waits there rather than on the loop.
pub(crate) fn take_deferred_notification_hooks(
    st: &mut ServerState,
) -> Vec<BackgroundCommandRequest> {
    let context = ClientContext::default();
    let mut requests = Vec::new();
    for notification in st.take_deferred_notifications() {
        push_event_hook(
            &notification.name,
            notification.target.as_deref(),
            notification.vars,
            st,
            &context,
            &mut requests,
        );
    }
    requests
}

fn push_event_hook(
    hook: &str,
    requested_target: Option<&str>,
    vars: Vec<(String, String)>,
    st: &mut ServerState,
    context: &ClientContext,
    requests: &mut Vec<BackgroundCommandRequest>,
) {
    let Some(commands) = hook_commands(hook, requested_target, st, HookOrigin::Event) else {
        return;
    };
    let mut context = context.clone();
    context.hook = Some(HookScope {
        vars: Rc::new(vars),
        target: requested_target.map(Rc::from),
    });
    context.suppress_after_hooks = true;
    context.suppress_notifications = true;
    for command in commands {
        if command.trim().is_empty() {
            continue;
        }
        requests.push(BackgroundCommandRequest::Ready {
            command: Some(command),
            context: context.clone(),
        });
    }
}

/// Whether a hook body comes from a command's `after-*`/`command-error` hook
/// or from an event notification. The two differ in re-entrancy: a command
/// hook is guarded by name for the length of its body, while several
/// notifications for the same event can be raised by one command and all run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HookOrigin {
    Command,
    Event,
}

fn hook_commands(
    hook: &str,
    requested_target: Option<&str>,
    st: &mut ServerState,
    origin: HookOrigin,
) -> Option<Vec<String>> {
    if !options::is_hook(hook) {
        return None;
    }
    if matches!(origin, HookOrigin::Command) && !st.begin_hook(hook) {
        return None;
    }
    let target = requested_target
        .filter(|target| st.resolve(target).is_some())
        .map(str::to_string)
        .or_else(|| current_target(st));
    let commands = target
        .as_deref()
        .and_then(|target| match options::option_scope(hook) {
            Some(OptionScope::Session) => st.session_options(target).ok(),
            Some(OptionScope::Window) => st.window_options(target).ok(),
            Some(OptionScope::WindowPane) => st.pane_options(target).ok(),
            _ => None,
        })
        .into_iter()
        .flat_map(|view| view.iter_effective())
        .filter(|(name, _)| {
            options::parse_option_name(name)
                .is_some_and(|(base, index)| base == hook && index.is_some())
        })
        .map(|(_, value)| value.to_string())
        .collect::<Vec<_>>();
    Some(commands)
}

fn parse_command_groups(groups: Vec<&[String]>) -> Result<Vec<ParsedCommand>, CommandResult> {
    // Parse phase: resolve every command's name up front. A bad name aborts the
    // whole line with no output, exactly like tmux's cmd_parse.
    let mut resolved: Vec<(&'static CommandSpec, &[String])> = Vec::with_capacity(groups.len());
    for group in &groups {
        let word = match group.first() {
            Some(w) => w.as_str(),
            None => continue, // empty group (e.g. trailing ';'): tmux ignores it
        };
        match registry::resolve_spec(word) {
            SpecResolution::Spec(spec) => resolved.push((spec, group)),
            SpecResolution::Ambiguous { error } | SpecResolution::Unknown { error } => {
                return Err(CommandResult::err(error));
            }
        }
    }

    // Still in the parse phase: validate each command's flags against tmux's
    // getopt spec. An unknown flag is a *parse* error too, so (like a bad command
    // name) it aborts the entire line with no output before anything runs.
    for (command, group) in &resolved {
        if let Some(getopt) = registry::getopt(command.name) {
            let bad = if command.name == "bind-key" {
                unknown_bind_key_flag(group, getopt)
            } else {
                unknown_flag(group, getopt)
            };
            if let Some(bad) = bad {
                return Err(CommandResult::err(format!(
                    "command {}: unknown flag -{bad}\n",
                    command.name
                )));
            }
            if let Some(flag) = missing_flag_value(group, getopt) {
                return Err(CommandResult::err(format!(
                    "command {}: -{flag} expects an argument\n",
                    command.name
                )));
            }
        }
    }

    let mut parsed = Vec::with_capacity(resolved.len());
    for (spec, group) in resolved {
        let args = normalize_argv(spec.name, group);
        if let Some((minimum, maximum)) = registry::argument_limits(spec.name) {
            let count = positional_argument_count(spec.name, &args);
            if count < minimum {
                return Err(CommandResult::err(format!(
                    "command {}: too few arguments (need at least {minimum})\n",
                    spec.name
                )));
            }
            if let Some(maximum) = maximum.filter(|maximum| count > *maximum) {
                return Err(CommandResult::err(format!(
                    "command {}: too many arguments (need at most {maximum})\n",
                    spec.name
                )));
            }
        }
        parsed.push(ParsedCommand { spec, args });
    }
    Ok(parsed)
}

fn parse_command_groups_with_aliases(
    groups: Vec<&[String]>,
    aliases: &[(String, String)],
) -> Result<Vec<ParsedCommand>, CommandResult> {
    let mut expanded = Vec::new();
    for group in groups {
        let Some(name) = group.first() else {
            continue;
        };
        let Some((_, replacement)) = aliases.iter().find(|(alias, _)| alias == name) else {
            expanded.push(group.to_vec());
            continue;
        };
        let tokens = tokenize_line(replacement);
        let mut replacements = tokenized_command_groups(&tokens);
        if replacements.is_empty() {
            // A matched alias whose value holds no commands expands to an empty
            // command list, so the invocation succeeds and drops its arguments.
            // Falling back to the alias name would report it as unknown.
            continue;
        }
        replacements
            .last_mut()
            .expect("nonempty replacement")
            .extend(group.iter().skip(1).cloned());
        expanded.extend(replacements);
    }
    let groups = expanded.iter().map(Vec::as_slice).collect::<Vec<_>>();
    parse_command_groups(groups)
}

fn positional_argument_count(name: &str, args: &[String]) -> usize {
    let Some(spec) = registry::getopt(name) else {
        return args.len().saturating_sub(1);
    };
    let mut index = 1;
    while index < args.len() {
        let argument = args[index].as_str();
        if argument == "--" {
            return args.len().saturating_sub(index + 1);
        }
        let flag = argument
            .strip_prefix('-')
            .filter(|value| value.chars().count() == 1)
            .and_then(|value| value.chars().next());
        let Some(flag) = flag else {
            return args.len() - index;
        };
        if registry::flag_kind(spec, flag) == Some(true) {
            index += 1;
        }
        index += 1;
    }
    0
}

/// Expand tmux's legacy trailing-semicolon argv form into the standalone token
/// consumed by [`split_commands`].
fn expand_attached_separators(args: &[String]) -> Vec<String> {
    let mut expanded = Vec::with_capacity(args.len());
    for arg in args {
        if arg.ends_with(r"\;") {
            expanded.push(arg.clone());
            continue;
        }
        if arg != ";" {
            if let Some(word) = arg.strip_suffix(';') {
                if !word.is_empty() {
                    expanded.push(word.to_string());
                }
                expanded.push(";".to_string());
                continue;
            }
        }
        expanded.push(arg.clone());
    }
    expanded
}

/// Split an argv into `;`-separated command groups.
fn split_commands(args: &[String]) -> Vec<&[String]> {
    let mut groups = Vec::new();
    let mut start = 0;
    for (i, a) in args.iter().enumerate() {
        if a == ";" {
            if start < i {
                groups.push(&args[start..i]);
            }
            start = i + 1;
        }
    }
    if start < args.len() {
        groups.push(&args[start..]);
    }
    groups
}

/// Dispatch one already-resolved command against the locked state. The catalog
/// identity has already replaced any alias or prefix used in the original argv.
struct CommandContext<'a> {
    state: &'a mut ServerState,
    client: &'a ClientContext,
    agents: &'a PaneAgents,
}

fn run_single(
    command: Command,
    args: &[String],
    state: &mut ServerState,
    agents: &PaneAgents,
    client: &ClientContext,
) -> CommandResult {
    command.execute(
        args,
        &mut CommandContext {
            state,
            client,
            agents,
        },
    )
}

// ---- individual commands ---------------------------------------------------

/// `list-commands [command]`. Prints the usage line for every command in tmux's
/// table, or — given a command argument — just that one. The argument resolves
/// through the same command resolver the interpreter uses everywhere else
/// (`cmd-list-commands.c` calls `cmd_find`), so an alias or unambiguous prefix
/// hits the right command while an ambiguous or unknown one reports the
/// resolver's own diagnostic (exit 1). `-F` expands once per command with
/// tmux's three command-list variables.
fn list_commands(args: &[String]) -> CommandResult {
    let template = flag_value(args, "-F");
    let render = |name: &'static str| {
        if let Some(template) = template {
            let spec = registry::COMMAND_SPECS
                .iter()
                .find(|spec| spec.name == name)
                .expect("resolved command is in the table");
            let mut vars = Vars::new();
            vars.set("command_list_name", name)
                .set("command_list_alias", spec.alias.unwrap_or(""))
                .set(
                    "command_list_usage",
                    registry::usage(name).expect("table command has a usage line"),
                );
            format::expand(template, &vars)
        } else {
            registry::command_line(name).expect("table command has a usage line")
        }
    };

    match positionals(args, &["-F"]).into_iter().next() {
        Some(word) => match registry::resolve(word) {
            Resolution::Name(name) => CommandResult::ok(format!("{}\n", render(name))),
            Resolution::Ambiguous { error } | Resolution::Unknown { error } => {
                CommandResult::err(error)
            }
        },
        None => {
            let mut out = String::new();
            for spec in registry::COMMAND_SPECS {
                out.push_str(&render(spec.name));
                out.push('\n');
            }
            CommandResult::ok(out)
        }
    }
}

/// `list-sessions [-F format]`. Sessions are listed sorted by name (tmux keys
/// its session tree by name), one per line; `-F` overrides the default summary.
/// tmux's `sort.c` orders behind the `list-*` `-O` flag.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ListSortOrder {
    Activity,
    Creation,
    Index,
    Modifier,
    Name,
    Order,
    Size,
    Z,
}

/// Parse `-O`/`-r` the way tmux's `sort_order_from_string` is consumed: no
/// `-O` means no sorting at all (a bare `-r` is inert), and an unknown order
/// is an error.
fn list_sort_criteria(args: &[String]) -> Result<(Option<ListSortOrder>, bool), CommandResult> {
    let reversed = has_flag(args, "-r");
    let Some(order) = flag_value(args, "-O") else {
        return Ok((None, reversed));
    };
    let order = match order.to_ascii_lowercase().as_str() {
        "activity" => ListSortOrder::Activity,
        "creation" => ListSortOrder::Creation,
        "index" | "key" => ListSortOrder::Index,
        "modifier" => ListSortOrder::Modifier,
        "name" | "title" => ListSortOrder::Name,
        "order" => ListSortOrder::Order,
        "size" => ListSortOrder::Size,
        "z" => ListSortOrder::Z,
        _ => return Err(CommandResult::err("invalid sort order\n")),
    };
    Ok((Some(order), reversed))
}

/// Sort one `list-*` table tmux's way: the `-O` comparison first, ties broken
/// by the name column, and `-r` flipping the combined result. `order` is the
/// natural enumeration, where `-r` just reverses the slice.
fn apply_list_sort<T>(
    items: &mut [T],
    order: Option<ListSortOrder>,
    reversed: bool,
    compare: impl Fn(ListSortOrder, &T, &T) -> std::cmp::Ordering,
    name: impl Fn(&T) -> String,
) {
    let Some(order) = order else {
        return;
    };
    if order == ListSortOrder::Order {
        if reversed {
            items.reverse();
        }
        return;
    }
    items.sort_by(|a, b| {
        let result = compare(order, a, b).then_with(|| name(a).cmp(&name(b)));
        if reversed {
            result.reverse()
        } else {
            result
        }
    });
}

fn list_sessions(args: &[String], st: &ServerState, agents: &PaneAgents) -> CommandResult {
    let template = flag_value(args, "-F");
    let filter = flag_value(args, "-f");
    let (sort_order, reversed) = match list_sort_criteria(args) {
        Ok(criteria) => criteria,
        Err(error) => return error,
    };
    let mut order: Vec<&Session> = st.sessions().iter().collect();
    order.sort_by(|a, b| a.name.cmp(&b.name));
    apply_list_sort(
        &mut order,
        sort_order,
        reversed,
        |key, a, b| match key {
            ListSortOrder::Index => a.id.cmp(&b.id),
            ListSortOrder::Creation => a.created_epoch.cmp(&b.created_epoch),
            // Most recent activity first, as tmux inverts this comparison.
            ListSortOrder::Activity => b.activity_micros.cmp(&a.activity_micros),
            ListSortOrder::Name => a.name.cmp(&b.name),
            _ => std::cmp::Ordering::Equal,
        },
        |session| session.name.clone(),
    );

    let marked = st.marked_pane();
    let mut out = String::new();
    for s in order {
        let mut vars = vars_for(st, s, s.active, agents, marked);
        // tmux's `FORMAT_TYPE_SESSION` marker for this list context.
        vars.set("session_format", "1");
        if let Some(f) = filter {
            if !format::is_true(&expand_command_format(st, f, &vars, None)) {
                continue;
            }
        }
        let line = match template {
            Some(t) => expand_command_format(st, t, &vars, None),
            None => match st.session_group_name(s) {
                Some(group) => format!("{} (group {group})", s.summary()),
                None => s.summary(),
            },
        };
        out.push_str(&line);
        out.push('\n');
    }
    CommandResult::ok(out)
}

/// `has-session [-t target]`. With `-t`, resolves the named session (exit 1 with
/// "can't find session" if missing). Without `-t`, tmux resolves the *current*
/// session, which always exists for a running server → exit 0.
fn has_session(args: &[String], st: &ServerState) -> CommandResult {
    match flag_value(args, "-t") {
        Some(name) if st.resolve_session(name).is_some() => CommandResult::ok(""),
        Some(name) => CommandResult::err(format!("can't find session: {name}\n")),
        None => match current_session(st) {
            Some(_) => CommandResult::ok(""),
            None => CommandResult::err("can't establish current session\n"),
        },
    }
}

/// `new-session [-d] [-s name] [-P] [-F format]`. Creates a detached session
/// with a shell pane. With `-P`, prints the new session via `-F` (or the default
/// `NEW_SESSION_TEMPLATE`) and a trailing newline, like real tmux.
fn new_session(args: &[String], st: &mut ServerState, context: &ClientContext) -> CommandResult {
    let name = flag_value(args, "-s")
        .map(str::to_string)
        .unwrap_or_else(|| st.next_session_name());
    // `-A` (attach-or-create): if the named session already exists, tmux
    // attaches to it instead of failing with "duplicate session". Over the
    // command path with no real tty that attach fails the same way
    // attach-session does ("open terminal failed"), which is what real tmux
    // reports here.
    if has_flag(args, "-A") && st.find(&name).is_some() {
        return CommandResult::err("open terminal failed: not a terminal\n");
    }
    if flag_value(args, "-t").is_some()
        && (flag_value(args, "-n").is_some()
            || !trailing_command(args, NEW_SESSION_VALUE_FLAGS).is_empty())
    {
        return CommandResult::err("command or window name given with target\n");
    }
    let dimensions = match new_session_dimensions(args) {
        Ok(dimensions) => dimensions,
        Err(error) => return CommandResult::err(error),
    };
    let spec = new_session_pane_spec(args, st, context);
    let result = match flag_value(args, "-t") {
        Some(target) => st.create_grouped_session(&name, target, spec),
        None => st.create_session(&name, spec),
    };
    match result {
        Ok(_) => {
            apply_new_session_opts(args, &name, st, dimensions, context);
            if has_flag(args, "-P") {
                let sess = st.find(&name).expect("session just created");
                let template = flag_value(args, "-F").unwrap_or(NEW_SESSION_TEMPLATE);
                let marked = st.marked_pane();
                let line = expand_command_format(
                    st,
                    template,
                    &vars_for(st, sess, sess.active, &PaneAgents::new(), marked),
                    None,
                );
                CommandResult::ok(format!("{line}\n"))
            } else {
                CommandResult::ok("")
            }
        }
        Err(e) => CommandResult::err(format!("{e}\n")),
    }
}

/// bind-key [-nr] [-T table] [-N note] key command [argument ...].
///
/// Defaults and user replacements share the same semantic tables consumed by
/// attached clients.
fn bind_key(args: &[String], st: &mut ServerState) -> CommandResult {
    let table = if has_flag(args, "-n") {
        "root".to_string()
    } else {
        flag_value(args, "-T").unwrap_or("prefix").to_string()
    };
    let mut key_index = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "-T" | "-N" => index += 2,
            "-n" | "-r" => index += 1,
            arg if !arg.starts_with('-') => {
                key_index = Some(index);
                break;
            }
            _ => index += 1,
        }
    }
    let Some(key_index) = key_index else {
        return CommandResult::err("missing key\n");
    };
    let key_name = &args[key_index];
    let Some(key) = parse_key_name(key_name).filter(|key| key.is_bindable()) else {
        return CommandResult::err(format!("unknown key: {key_name}\n"));
    };
    let command_start = key_index + 1;
    // tmux parses the binding's command when the binding is made, so a
    // `command-alias` is resolved here and the binding keeps what it expanded
    // to — redefining the alias afterwards leaves it alone.
    let command = expand_command_aliases(&args[command_start..], st);
    st.bind_key(
        &table,
        key,
        command,
        has_flag(args, "-r"),
        flag_value(args, "-N").map(str::to_string),
    );
    CommandResult::ok("")
}

/// One command line with its leading `command-alias` resolved, as tmux's
/// `cmd_parse` does before anything stores it. Only the first word can be an
/// alias, and its replacement keeps the arguments that followed.
fn expand_command_aliases(command: &[String], st: &ServerState) -> Vec<String> {
    let Some(name) = command.first() else {
        return command.to_vec();
    };
    let aliases = st.command_aliases();
    let Some((_, replacement)) = aliases.iter().find(|(alias, _)| alias == name) else {
        return command.to_vec();
    };
    // Only the first of the alias's own commands can take the arguments; a
    // multi-command alias in a binding is out of reach of this shape, so the
    // groups are flattened back with their separators.
    let mut expanded = Vec::new();
    for (index, group) in tokenized_command_groups(&tokenize_line(replacement))
        .into_iter()
        .enumerate()
    {
        if index != 0 {
            expanded.push(";".to_string());
        }
        expanded.extend(group);
    }
    expanded.extend(command.iter().skip(1).cloned());
    expanded
}

/// unbind-key [-anq] [-T table] key.
fn unbind_key(args: &[String], st: &mut ServerState) -> CommandResult {
    let quiet = has_flag(args, "-q");
    let error = |message| {
        if quiet {
            CommandResult::ok("")
        } else {
            CommandResult::err(message)
        }
    };
    let key_name = positionals(args, &["-T"]).into_iter().next();
    let table = flag_value(args, "-T").unwrap_or_else(|| {
        if has_flag(args, "-n") {
            "root"
        } else {
            "prefix"
        }
    });

    if has_flag(args, "-a") {
        if key_name.is_some() {
            return error("key given with -a\n".to_string());
        }
        if !st.key_table_exists(table) {
            return error(format!("table {table} doesn't exist\n"));
        }
        st.unbind_key(table, None, true);
        return CommandResult::ok("");
    }

    let Some(key_name) = key_name else {
        return error("missing key\n".to_string());
    };
    let Some(key) = parse_key_name(key_name).filter(|key| key.is_bindable()) else {
        return error(format!("unknown key: {key_name}\n"));
    };
    if flag_value(args, "-T").is_some() && !st.key_table_exists(table) {
        return error(format!("table {table} doesn't exist\n"));
    }

    // tmux treats an unbound key in an existing table as a successful no-op.
    st.unbind_key(table, Some(key), false);
    CommandResult::ok("")
}

/// list-keys [-T table] [key] for the shared mutable key tables.
fn list_keys(args: &[String], st: &ServerState) -> CommandResult {
    let table = flag_value(args, "-T");
    if let Some(table) = table {
        if !st.key_table_exists(table) {
            return CommandResult::err(format!("table {table} doesn't exist\n"));
        }
    }
    let requested_name = positionals(args, &["-F", "-O", "-P", "-T"])
        .into_iter()
        .next();
    let requested = match requested_name {
        Some(name) => match parse_key_name(name) {
            Some(key) => Some(key),
            None => return CommandResult::err(format!("invalid key: {name}\n")),
        },
        None => None,
    };
    let bindings = st.key_bindings(table);
    // `-F` expands a template per binding with tmux's `key_*` variables;
    // `-N` selects only bindings with notes unless `-a` is also given. `-r`
    // selects only repeatable bindings.
    if let Some(template) = flag_value(args, "-F") {
        let notes_filter = has_flag(args, "-N") && !has_flag(args, "-a");
        let filtered: Vec<_> = bindings
            .iter()
            .filter(|(_, key, binding)| {
                !requested.is_some_and(|wanted| wanted != *key)
                    && (!notes_filter || binding.note.is_some())
                    && (!has_flag(args, "-r") || binding.repeat)
            })
            .collect();
        let mut filtered = filtered;
        filtered.sort_by_key(|(_, key, _)| list_key_order(*key));
        let key_has_repeat = filtered.iter().any(|(_, _, binding)| binding.repeat);
        let key_string_width = filtered
            .iter()
            .map(|(_, key, _)| format_key_name(*key).chars().count())
            .max()
            .unwrap_or(0);
        let key_table_width = filtered
            .iter()
            .map(|(table_name, _, _)| table_name.chars().count())
            .max()
            .unwrap_or(0);
        let mut out = String::new();
        for (table_name, key, binding) in &filtered {
            let mut vars = format::Vars::new();
            vars.set("notes_only", if has_flag(args, "-N") { "1" } else { "0" })
                .set("key_has_repeat", if key_has_repeat { "1" } else { "0" })
                .set("key_string_width", key_string_width.to_string())
                .set("key_table_width", key_table_width.to_string())
                .set("key_repeat", if binding.repeat { "1" } else { "0" })
                .set("key_note", binding.note.clone().unwrap_or_default())
                .set("key_table", *table_name)
                .set("key_string", format_key_name(*key))
                .set("key_command", binding.command.join(" "));
            let line = format::expand(template, &vars);
            if !line.is_empty() {
                out.push_str(&line);
                out.push('\n');
            }
        }
        // A sole match goes through the target client's status message, which
        // a detached command client sees as a successful empty result.
        if filtered.len() <= 1 {
            return CommandResult::ok("");
        }
        return CommandResult::ok(out);
    }
    let mut out = String::new();
    let mut matched = 0;
    for (table_name, key, binding) in bindings {
        if requested.is_some_and(|wanted| wanted != key) {
            continue;
        }
        out.push_str("bind-key");
        if binding.repeat {
            out.push_str(" -r");
        }
        out.push_str(" -T ");
        out.push_str(table_name);
        out.push(' ');
        out.push_str(&format_key_name(key));
        for word in &binding.command {
            out.push(' ');
            out.push_str(word);
        }
        out.push('\n');
        matched += 1;
    }
    // tmux 3.7b routes a sole match through the target client's status message
    // instead of command stdout. A detached command client therefore sees a
    // successful empty result for either zero or one matching binding.
    if matched <= 1 {
        return CommandResult::ok("");
    }
    CommandResult::ok(out)
}

/// The order emitted by tmux's key table catalog: plain character keys first,
/// then named keys, followed by meta, control, and shift variants.
fn list_key_order(key: KeyCode) -> (u8, u8, u32) {
    let modifier_group = match (
        key.modifiers.meta(),
        key.modifiers.ctrl(),
        key.modifiers.shift(),
    ) {
        (false, false, false) => 0,
        (true, false, false) => 2,
        (false, true, false) => 3,
        (false, false, true) => 4,
        _ => 5,
    };
    match key.base {
        KeyBase::Char(value) => (modifier_group, 0, value as u32),
        KeyBase::Special(value) => (
            modifier_group,
            1,
            match value {
                SpecialKey::Delete => 0,
                SpecialKey::PageUp => 1,
                SpecialKey::Up => 2,
                SpecialKey::Down => 3,
                SpecialKey::Left => 4,
                SpecialKey::Right => 5,
                SpecialKey::F(number) => 10 + u32::from(number),
                SpecialKey::Insert => 30,
                SpecialKey::Home => 31,
                SpecialKey::End => 32,
                SpecialKey::PageDown => 33,
                SpecialKey::BackTab => 34,
                SpecialKey::Backspace => 35,
                SpecialKey::Keypad(value) => 40 + value as u32,
                SpecialKey::KeypadEnter => 50,
                SpecialKey::PasteStart => 51,
                SpecialKey::PasteEnd => 52,
            },
        ),
        KeyBase::User(value) => (modifier_group, 2, u32::from(value)),
        KeyBase::Mouse(_) => (modifier_group, 3, 0),
        KeyBase::Any => (modifier_group, 4, 0),
        KeyBase::None => (modifier_group, 5, 0),
    }
}

/// Apply the `new-session` options that shape a freshly-created session —
/// `-x`/`-y` (client size), `-e` (environment), `-n` (first window name). Shared
/// by the command-path [`new_session`] and the interactive
/// [`new_session_for_attach`] so both create identical sessions. Assumes `args`
/// is already normalized (see [`normalize_argv`]).
fn apply_new_session_opts(
    args: &[String],
    name: &str,
    st: &mut ServerState,
    dimensions: (Option<u16>, Option<u16>),
    context: &ClientContext,
) {
    // tmux's `s->cwd`: the `-c` directory, else where the creating client was.
    // It is what the session's `#()` jobs run in.
    if let Some(session_id) = st.session_id(name) {
        let cwd = command_option_value(args, "-c", NEW_SESSION_VALUE_FLAGS)
            .map(PathBuf::from)
            .or_else(|| context.cwd.clone());
        st.set_session_cwd(session_id, cwd);
    }
    // `-x W -y H` sets the new session's client size.
    let (x, y) = dimensions;
    let joined_existing_group = st
        .find(name)
        .is_some_and(|session| st.is_grouped(session) && st.session_group_size(session) > 1);
    if !joined_existing_group {
        if let Some(sess) = st.find(name) {
            let (session_cols, session_rows) = (sess.cols, sess.rows);
            if x.is_some() || y.is_some() {
                // tmux writes `-x`/`-y` onto the new session's own `default-size`,
                // which is what shadows the global option for every window later
                // created in it. An unspecified axis keeps the session's size.
                let _ = st.set_session_default_size(
                    name,
                    x.unwrap_or(session_cols),
                    y.unwrap_or(session_rows),
                );
            }
        }
    }
    // The `update-environment` copy-in, which `-E` skips. tmux reads the
    // option from the *global* session table here, since the session being
    // created has no options of its own yet.
    if !has_bool_flag(args, 'E') {
        st.update_session_environment(name, &context.environment);
    }
    // `-e VAR=value` seeds environment variables (repeatable), after the
    // copy-in so an explicit assignment wins.
    for kv in flag_values(args, "-e") {
        if let Some((k, v)) = kv.split_once('=') {
            let _ = st.set_session_env(name, k, v, false);
        }
    }
    // `-n name` names the session's first window.
    if let Some(win_name) = flag_value(args, "-n") {
        let _ = st.rename_window(&format!("{name}:"), win_name);
    } else if flag_value(args, "-t").is_none() {
        let command = trailing_command(args, NEW_SESSION_VALUE_FLAGS);
        apply_initial_window_name(st, name, 0, command.as_slice());
    }
}

fn new_session_dimensions(args: &[String]) -> Result<(Option<u16>, Option<u16>), String> {
    Ok((
        new_session_dimension(args, "-x", "width")?,
        new_session_dimension(args, "-y", "height")?,
    ))
}

fn new_session_dimension(args: &[String], flag: &str, label: &str) -> Result<Option<u16>, String> {
    let Some(value) = flag_value(args, flag) else {
        return Ok(None);
    };
    // tmux uses "-" to request the invoking client's size. Detached command
    // clients have no usable dimensions, so preserve the existing fallback.
    if value == "-" {
        return Ok(None);
    }
    match value.parse::<i128>() {
        Ok(value) if value < 1 => Err(format!("{label} too small\n")),
        Ok(value) if value > i128::from(u16::MAX) => Err(format!("{label} too large\n")),
        Ok(value) => Ok(Some((value as u16).min(NEW_SESSION_MAX_SIZE))),
        Err(_) if numeric_with_sign(value, '-') => Err(format!("{label} too small\n")),
        Err(_)
            if numeric_with_sign(value, '+') || value.bytes().all(|byte| byte.is_ascii_digit()) =>
        {
            Err(format!("{label} too large\n"))
        }
        Err(_) => Err(format!("{label} invalid\n")),
    }
}

fn numeric_with_sign(value: &str, sign: char) -> bool {
    value.strip_prefix(sign).is_some_and(|digits| {
        !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
    })
}

fn apply_initial_window_name(
    st: &mut ServerState,
    session: &str,
    window_position: usize,
    command: &[&str],
) {
    let Some(link) = st
        .find(session)
        .and_then(|session| session.windows.get(window_position))
        .copied()
    else {
        return;
    };
    let target = format!("{session}:{}", link.index);
    let default_shell = st
        .option_for_target(&target, "default-shell")
        .unwrap_or("/bin/sh");
    let default_command = st
        .option_for_target(&target, "default-command")
        .unwrap_or("");
    let current = match command {
        [] if !default_command.is_empty() => default_command.split_whitespace().next(),
        [] => Some(default_shell),
        [command] => command.split_whitespace().next(),
        command => command.first().copied(),
    }
    .and_then(|command| Path::new(command).file_name())
    .and_then(|command| command.to_str())
    .unwrap_or("")
    .to_string();
    let name = if st.option_for_target(&target, "automatic-rename") == Some("on") {
        let source = st
            .option_for_target(&target, "automatic-rename-format")
            .unwrap_or("#{pane_current_command}");
        let mut vars = format::Vars::new();
        vars.set("pane_current_command", current.clone())
            .set("pane_in_mode", "0")
            .set("pane_dead", "0");
        format::expand(source, &vars)
    } else {
        current
    };
    if !name.is_empty() {
        let _ = st.rename_window_automatically(&target, &name);
    }
}

/// What a client's command line means for a client that carries a real tty: it
/// determines whether the connection ends as an attached interactive client or
/// as a one-shot command client that runs and exits.
///
/// Mirrors real tmux: a command like `attach-session`, or a `new-session`
/// *without* `-d`, converts the client into an attached client (terminal I/O
/// takes over the tty); everything else runs and exits over the imsg file
/// protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    /// `attach-session` / `attach`: attach to an existing session.
    Attach,
    /// `new-session` without `-d` (and the bare-`tmux` empty command line):
    /// create (or, with `-A`, find-or-create) a session, then attach to it.
    NewAttach,
    /// Any other command line: run it and exit (the command-client path).
    Command,
}

/// tmux's `server_client_default_command`: a client that sent no command line
/// at all runs whatever `default-client-command` holds, parsed as a command
/// list. The stock default is `new-session`, which is why a bare `tmux` creates
/// and attaches.
pub fn default_client_command(st: &ServerState) -> Vec<String> {
    let value = st
        .server_options()
        .get("default-client-command")
        .unwrap_or("new-session")
        .to_owned();
    tokenize_line(&value)
        .into_iter()
        .map(|token| match token {
            LineToken::Word(word) => word,
            LineToken::Separator => ";".to_string(),
        })
        .collect()
}

/// The shell a `tmux -c` client is told to exec, from `default-shell`.
///
/// tmux resolves it against the client's session when it has one — `tmux -c`
/// does not, `detach-client -E` does — and falls back to the global session
/// option, then to `_PATH_BSHELL` when neither names a usable shell.
pub fn default_shell(st: &ServerState, session: Option<&str>) -> String {
    session
        .and_then(|session| st.option_for_target(session, "default-shell"))
        .or_else(|| st.global_options().session().get("default-shell"))
        .filter(|shell| shell.starts_with('/'))
        .unwrap_or("/bin/sh")
        .to_owned()
}

/// Classify a client's raw command argv into an [`Intent`]. `attach`/`new` name
/// resolution goes through the same [`registry`] the command dispatcher uses, so
/// aliases (`new`, `attach`) and unambiguous prefixes resolve here too.
///
/// A multi-command line (`new-session \; split-window`) is left to the
/// command-client path: interactive attach of a command *list* would need tmux's
/// full "run the list, then attach" flow, which is out of scope; the common
/// single-command interactive forms are what this closes.
pub fn classify(args: &[String]) -> Intent {
    // Bare `tmux` sends an empty command line, which tmux treats as the default
    // `new-session` — an interactive create-and-attach.
    let first = match args.first() {
        None => return Intent::NewAttach,
        Some(w) => w.as_str(),
    };
    // A command list is handled by the command path (see the doc comment).
    if args.iter().any(|a| a == ";" || a.ends_with(';')) {
        return Intent::Command;
    }
    let canonical = match registry::resolve(first) {
        Resolution::Name(name) => name,
        // Unknown/ambiguous: let the command path produce tmux's diagnostic.
        _ => return Intent::Command,
    };
    match canonical {
        "attach-session" => Intent::Attach,
        // `new-session -d` is a detached create → command path; otherwise attach.
        "new-session" if !has_bool_flag(args, 'd') => Intent::NewAttach,
        _ => Intent::Command,
    }
}

/// Create — or, with `-A`, find-or-create — the session an interactive
/// `new-session` (or a bare `tmux`) should attach to, applying the same options
/// [`new_session`] does. Returns the session name to attach to, or an
/// already-newline-terminated error line to report to the client (e.g.
/// `duplicate session: 0`).
///
/// This is the interactive twin of [`new_session`]: it does the create but not
/// the `-P` print (an attached client shows the session, it doesn't print it)
/// and not the command-path `-A`/no-tty error (attaching is exactly what we go
/// on to do).
pub fn new_session_for_attach(
    raw: &[String],
    st: &mut ServerState,
    context: &ClientContext,
) -> Result<String, String> {
    let args = normalize_argv("new-session", raw);
    let requested = flag_value(&args, "-s").map(str::to_string);
    // `-A` (attach-or-create): an existing named session is attached as-is.
    if has_bool_flag(&args, 'A') {
        if let Some(ref n) = requested {
            if st.find(n).is_some() {
                return Ok(n.clone());
            }
        }
    }
    let name = requested.unwrap_or_else(|| st.next_session_name());
    if flag_value(&args, "-t").is_some()
        && (flag_value(&args, "-n").is_some()
            || !trailing_command(&args, NEW_SESSION_VALUE_FLAGS).is_empty())
    {
        return Err("command or window name given with target\n".to_string());
    }
    let dimensions = new_session_dimensions(&args)?;
    let spec = new_session_pane_spec(&args, st, context);
    let result = match flag_value(&args, "-t") {
        Some(target) => st.create_grouped_session(&name, target, spec),
        None => st.create_session(&name, spec),
    };
    match result {
        Ok(_) => {
            apply_new_session_opts(&args, &name, st, dimensions, context);
            Ok(name)
        }
        // create_session already yields tmux's "duplicate session: <name>".
        Err(e) => Err(format!("{e}\n")),
    }
}

/// Build the first pane exactly once for detached and interactive
/// `new-session`. Both paths receive the same command tail, identify
/// environment, terminal name, and working-directory semantics.
fn new_session_pane_spec(args: &[String], st: &ServerState, context: &ClientContext) -> PaneSpec {
    let command = trailing_command(args, NEW_SESSION_VALUE_FLAGS);
    let argv = pane_command_argv(command.as_slice(), st, None);
    let spawn_environment = flag_values(args, "-e");
    let argv = pane_argv(argv, context, &spawn_environment, st, SpawnSession::Pending);
    match command_option_value(args, "-c", NEW_SESSION_VALUE_FLAGS)
        .map(PathBuf::from)
        .or_else(|| context.cwd.clone())
    {
        Some(cwd) => PaneSpec::CommandIn(argv, cwd),
        None => PaneSpec::Command(argv),
    }
}

/// Resolve the session spawn defaults used when a pane command is omitted.
/// `target` is absent only while creating the first session, before a target
/// exists; in that case the global session option table is authoritative.
fn pane_command_argv(command: &[&str], st: &ServerState, target: Option<&str>) -> Vec<String> {
    let option = |name| {
        target
            .and_then(|target| st.option_for_target(target, name))
            .or_else(|| st.global_options().session().get(name))
    };
    // The same shell the pane's `SHELL` names, so the two cannot drift.
    let shell = st.default_shell(target).to_string();
    match command {
        [] => match option("default-command").filter(|command| !command.is_empty()) {
            Some(command) => vec![shell, "-c".to_string(), command.to_string()],
            None => vec![shell],
        },
        [command] => vec![shell, "-c".to_string(), (*command).to_string()],
        command => command
            .iter()
            .map(|argument| (*argument).to_string())
            .collect(),
    }
}

/// `rename-session [-t target] new-name`. With `-t` omitted, renames the current
/// (newest) session.
fn rename_session(args: &[String], st: &mut ServerState) -> CommandResult {
    let from = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let to = positionals(args, &["-t"]).into_iter().next();
    match (from, to) {
        (Some(from), Some(to)) => match st.rename_session(&from, to) {
            Ok(()) => CommandResult::ok(""),
            Err(e) => CommandResult::err(format!("{e}\n")),
        },
        (None, _) => CommandResult::err("can't establish current session\n"),
        (_, None) => {
            CommandResult::err("command rename-session: too few arguments (need at least 1)\n")
        }
    }
}

/// `new-window [-t target] [-P] [-F format]`. Opens a window in the target's
/// session (or the current session). With `-P`, prints the new window via `-F`
/// (or `NEW_WINDOW_TEMPLATE`).
fn new_window(args: &[String], st: &mut ServerState, context: &ClientContext) -> CommandResult {
    const VALUE_FLAGS: &[&str] = &["-c", "-e", "-F", "-n", "-t"];
    let requested_target = flag_value(args, "-t");
    let (session, explicit) = match parse_window_target(requested_target, st) {
        Some(x) => x,
        None => return CommandResult::err("can't establish current session\n"),
    };
    // `-S` with a name and no explicit window index selects an existing window
    // of that name instead of creating one. tmux errors on an ambiguous name,
    // skips the selection under `-d`, and returns before the `-P` print either
    // way.
    if has_flag(args, "-S") && explicit.is_none() {
        if let Some(name) = flag_value(args, "-n") {
            let sess = st.find(&session).expect("session present");
            let matches: Vec<u32> = sess
                .windows
                .iter()
                .filter(|link| st.window_for_link(link).name == name)
                .map(|link| link.index)
                .collect();
            if matches.len() > 1 {
                return CommandResult::err(format!("multiple windows named {name}\n"));
            }
            if let Some(index) = matches.first() {
                if !has_flag(args, "-d") {
                    if let Err(error) = st.select_window(&format!("{session}:{index}")) {
                        return CommandResult::err(format!("{error}\n"));
                    }
                }
                return CommandResult::ok("");
            }
        }
    }
    // `-a` (after) / `-b` (before) insert *relative* to an anchor window rather
    // than at an explicit index. `-b` takes precedence over `-a` (tmux keys the
    // shuffle on whether `-b` is present). The anchor is the `-t` window part when
    // it names an existing window, else the session's active window. When `-t`
    // names an index that does *not* exist, tmux ignores the relative flag and
    // creates at that index — the same explicit path as without `-a`/`-b`.
    let relative = has_flag(args, "-a") || has_flag(args, "-b");
    // `-d` creates the window in the background: it is not selected, so the
    // session's current window stays put (tmux's default is to follow).
    let select = !has_flag(args, "-d");
    let explicit_cwd = command_option_value(args, "-c", VALUE_FLAGS).map(PathBuf::from);
    let cwd = explicit_cwd.as_deref().or(context.cwd.as_deref());
    let command = trailing_command(args, VALUE_FLAGS);
    let argv = pane_command_argv(command.as_slice(), st, Some(&session));
    let spawn_environment = flag_values(args, "-e");
    let argv = pane_argv(argv, context, &spawn_environment, st, SpawnSession::Existing(&session));
    let result = if relative {
        match anchor_window_index(&session, explicit, st) {
            Some(anchor) => st.new_window_relative_with_spawn(
                &session,
                anchor,
                !has_flag(args, "-b"),
                select,
                &argv,
                cwd,
            ),
            None if has_flag(args, "-k") => {
                st.new_window_replacing_with_spawn(&session, explicit, select, &argv, cwd)
            }
            None => st.new_window_with_spawn(&session, explicit, select, &argv, cwd),
        }
    } else if has_flag(args, "-k") {
        st.new_window_replacing_with_spawn(&session, explicit, select, &argv, cwd)
    } else {
        st.new_window_with_spawn(&session, explicit, select, &argv, cwd)
    };
    match result {
        Ok(win_idx) => {
            // `-n name` sets the new window's name (otherwise it stays the
            // model's default, empty).
            if let Some(name) = flag_value(args, "-n") {
                let index = st.find(&session).expect("session present").windows[win_idx].index;
                let _ = st.rename_window(&format!("{session}:{index}"), name);
            } else {
                apply_initial_window_name(st, &session, win_idx, command.as_slice());
            }
            if has_flag(args, "-P") {
                let sess = st.find(&session).expect("session present");
                let template = flag_value(args, "-F").unwrap_or(NEW_WINDOW_TEMPLATE);
                let marked = st.marked_pane();
                let line = expand_command_format(
                    st,
                    template,
                    &vars_for(st, sess, win_idx, &PaneAgents::new(), marked),
                    None,
                );
                CommandResult::ok(format!("{line}\n"))
            } else {
                CommandResult::ok("")
            }
        }
        Err(error) => match requested_target {
            Some(target) => command_target_error(error, target, "window"),
            None => CommandResult::err(format!("{error}\n")),
        },
    }
}

/// `rename-window [-t target] new-name`. Renames a window (the target's, or the
/// current session's active window).
fn rename_window(args: &[String], st: &mut ServerState) -> CommandResult {
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_session(st).map(|session| format!("{session}:")));
    let name = positionals(args, &["-t"]).into_iter().next();
    match (target, name) {
        (Some(target), Some(name)) => match st.rename_window(&target, name) {
            Ok(()) => CommandResult::ok(""),
            Err(e) => CommandResult::err(format!("{e}\n")),
        },
        (None, _) => CommandResult::err("can't establish current session\n"),
        (_, None) => {
            CommandResult::err("command rename-window: too few arguments (need at least 1)\n")
        }
    }
}

/// `list-windows [-t target] [-F format]`. Lists a session's windows in index
/// order. `-F` overrides the (structural) default line.
fn list_windows(args: &[String], st: &ServerState, agents: &PaneAgents) -> CommandResult {
    let template = flag_value(args, "-F");
    let filter = flag_value(args, "-f");
    let default_line = "#{window_index}: #{window_name} (#{window_panes} panes)";

    // `-a` lists every window across all sessions (sorted by session name), like
    // tmux; otherwise just the target session's windows.
    let sessions: Vec<&Session> = if has_flag(args, "-a") {
        let mut all: Vec<&Session> = st.sessions().iter().collect();
        all.sort_by(|a, b| a.name.cmp(&b.name));
        all
    } else {
        let session = flag_value(args, "-t")
            .and_then(|t| t.split(':').next())
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or_else(|| current_session(st));
        let session = match session {
            Some(s) => s,
            None => return CommandResult::err("can't establish current session\n"),
        };
        match st.resolve_session(&session) {
            Some(s) => vec![s],
            None => return CommandResult::err(format!("can't find session: {session}\n")),
        }
    };

    let (sort_order, reversed) = match list_sort_criteria(args) {
        Ok(criteria) => criteria,
        Err(error) => return error,
    };
    let mut links: Vec<(&Session, usize)> = sessions
        .iter()
        .flat_map(|sess| (0..sess.windows.len()).map(move |idx| (*sess, idx)))
        .collect();
    apply_list_sort(
        &mut links,
        sort_order,
        reversed,
        |key, (sess_a, idx_a), (sess_b, idx_b)| {
            let (link_a, link_b) = (&sess_a.windows[*idx_a], &sess_b.windows[*idx_b]);
            let (win_a, win_b) = (st.window_for_link(link_a), st.window_for_link(link_b));
            match key {
                ListSortOrder::Index => link_a.index.cmp(&link_b.index),
                // `activity_epoch` is written once at creation (see `Window`),
                // so it is the creation key; tmux's activity sort is inverted.
                ListSortOrder::Creation => win_a.activity_epoch.cmp(&win_b.activity_epoch),
                ListSortOrder::Activity => win_b.activity_epoch.cmp(&win_a.activity_epoch),
                ListSortOrder::Name => win_a.name.cmp(&win_b.name),
                ListSortOrder::Size => (u32::from(win_a.cols) * u32::from(win_a.rows))
                    .cmp(&(u32::from(win_b.cols) * u32::from(win_b.rows))),
                _ => std::cmp::Ordering::Equal,
            }
        },
        |(sess, idx)| st.window_for_link(&sess.windows[*idx]).name.clone(),
    );

    let marked = st.marked_pane();
    let mut out = String::new();
    for (sess, idx) in links {
        let mut vars = vars_for(st, sess, idx, agents, marked);
        // tmux's `FORMAT_TYPE_WINDOW` marker for this list context.
        vars.set("window_format", "1");
        if let Some(f) = filter {
            if !format::is_true(&expand_command_format(st, f, &vars, None)) {
                continue;
            }
        }
        // Structural default (tmux's includes a volatile layout id + @id, so
        // this isn't byte-identical; the suite pins `list-windows` via -F).
        let line = expand_command_format(st, template.unwrap_or(default_line), &vars, None);
        out.push_str(&line);
        out.push('\n');
    }
    CommandResult::ok(out)
}

/// `list-panes [-t target] [-F format]`. Lists the panes of the target window
/// (the target's `session[:window]`, defaulting to the current session's active
/// window) in index order. `-F` overrides the (structural) default line.
fn list_panes(args: &[String], st: &ServerState, agents: &PaneAgents) -> CommandResult {
    let template = flag_value(args, "-F");
    let filter = flag_value(args, "-f");
    // Structural default (tmux's includes volatile history/byte counts + %id, so
    // it isn't byte-identical; the suite pins list-panes via -F).
    let default_line = "#{pane_index}: [#{pane_active}]";

    // Each entry is (session, window Vec-position) to enumerate panes of.
    let windows: Vec<(&Session, usize)> = if has_flag(args, "-a") {
        let mut all: Vec<&Session> = st.sessions().iter().collect();
        all.sort_by(|a, b| a.name.cmp(&b.name));
        all.into_iter()
            .flat_map(|s| (0..s.windows.len()).map(move |w| (s, w)))
            .collect()
    } else if has_flag(args, "-s") {
        let target = flag_value(args, "-t")
            .and_then(|t| t.split(':').next())
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or_else(|| current_session(st));
        let target = match target {
            Some(t) => t,
            None => return CommandResult::err("can't establish current session\n"),
        };
        let session = match st.resolve_session(&target) {
            Some(s) => s,
            None => return CommandResult::err(format!("can't find session: {target}\n")),
        };
        (0..session.windows.len()).map(|w| (session, w)).collect()
    } else {
        let target = flag_value(args, "-t")
            .map(str::to_string)
            .or_else(|| current_target(st));
        let target = match target {
            Some(t) => t,
            None => return CommandResult::err("can't establish current session\n"),
        };
        // `list-panes -t` is a *window* target, so tmux names the window (or the
        // session) part that went missing rather than the whole target.
        let resolved = match st.resolve_window_target(&target) {
            Ok(resolved) => resolved,
            Err(error) => return CommandResult::err(format!("{error}\n")),
        };
        vec![(&st.sessions()[resolved.session], resolved.window)]
    };

    let (sort_order, reversed) = match list_sort_criteria(args) {
        Ok(criteria) => criteria,
        Err(error) => return error,
    };
    let mut panes: Vec<(&Session, usize, usize)> = windows
        .iter()
        .flat_map(|(sess, win_pos)| {
            (0..st.session_window(sess, *win_pos).panes.len())
                .map(move |pane_idx| (*sess, *win_pos, pane_idx))
        })
        .collect();
    let pane_title = |sess: &Session, win_pos: usize, pane_idx: usize| {
        st.session_window(sess, win_pos)
            .panes
            .get(pane_idx)
            .and_then(|pane| st.pane_title(pane))
            .unwrap_or_else(format::hostname)
    };
    apply_list_sort(
        &mut panes,
        sort_order,
        reversed,
        |key, (sess_a, win_a, idx_a), (sess_b, win_b, idx_b)| {
            let win_a = st.session_window(sess_a, *win_a);
            let win_b = st.session_window(sess_b, *win_b);
            let (pane_a, pane_b) = (&win_a.panes[*idx_a], &win_b.panes[*idx_b]);
            match key {
                ListSortOrder::Index => idx_a.cmp(idx_b),
                // Pane ids are allocated in creation order, as tmux's are.
                ListSortOrder::Creation => pane_a.id.cmp(&pane_b.id),
                ListSortOrder::Size => {
                    let area = |win: &super::state::Window, id: u32| {
                        win.pane_rect(id)
                            .map(|rect| u32::from(rect.width) * u32::from(rect.height))
                            .unwrap_or(0)
                    };
                    area(win_a, pane_a.id).cmp(&area(win_b, pane_b.id))
                }
                ListSortOrder::Name => std::cmp::Ordering::Equal,
                // tmux's activity key is the pane's `active_point`: least
                // recently active first, with ties falling to the title.
                ListSortOrder::Activity => pane_a.active_point.cmp(&pane_b.active_point),
                _ => std::cmp::Ordering::Equal,
            }
        },
        |(sess, win_pos, pane_idx)| pane_title(sess, *win_pos, *pane_idx),
    );

    let marked = st.marked_pane();
    let mut out = String::new();
    for (sess, win_pos, pane_idx) in panes {
        let vars = vars_full(st, sess, win_pos, pane_idx, agents, marked);
        if let Some(f) = filter {
            if !format::is_true(&expand_command_format(st, f, &vars, None)) {
                continue;
            }
        }
        let line = expand_command_format(st, template.unwrap_or(default_line), &vars, None);
        out.push_str(&line);
        out.push('\n');
    }
    CommandResult::ok(out)
}

fn prompt_history_type_arg(args: &[String]) -> Result<Option<&str>, CommandResult> {
    let prompt_type = flag_value(args, "-T");
    if prompt_type
        .is_some_and(|value| !matches!(value, "command" | "search" | "target" | "window-target"))
    {
        return Err(CommandResult::err(format!(
            "invalid type: {}\n",
            prompt_type.unwrap_or_default()
        )));
    }
    Ok(prompt_type)
}

fn show_prompt_history(args: &[String], st: &ServerState) -> CommandResult {
    let prompt_type = match prompt_history_type_arg(args) {
        Ok(prompt_type) => prompt_type,
        Err(error) => return error,
    };
    let types: Vec<&str> = prompt_type
        .map(|prompt_type| vec![prompt_type])
        .unwrap_or_else(|| vec!["command", "search", "target", "window-target"]);
    let mut output = String::new();
    for prompt_type in types {
        output.push_str(&format!("History for {prompt_type}:\n"));
        output.push('\n');
        let history = st.prompt_history(prompt_type);
        if !history.is_empty() {
            for (index, value) in history.iter().enumerate() {
                output.push_str(&format!("{}: {value}\n", index + 1));
            }
        }
        output.push('\n');
    }
    CommandResult::ok(output)
}

fn clear_prompt_history(args: &[String], st: &mut ServerState) -> CommandResult {
    let prompt_type = match prompt_history_type_arg(args) {
        Ok(prompt_type) => prompt_type,
        Err(error) => return error,
    };
    st.clear_prompt_history(prompt_type);
    CommandResult::ok("")
}

/// `display-message [-p] [-c client] [-t target] [message]`.
fn display_message(
    args: &[String],
    st: &mut ServerState,
    agents: &PaneAgents,
    context: &ClientContext,
) -> CommandResult {
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    if target.is_none() && (!has_flag(args, "-p") || has_flag(args, "-I")) {
        return CommandResult::err("can't establish current session\n");
    }
    // display-message uses tmux's can-fail target lookup: an unresolvable target
    // does not fail the command, it formats against however far the lookup got
    // (the named session's current window, say) or against nothing at all.
    let resolved = target
        .as_deref()
        .and_then(|target| st.resolve_or_residual(target));
    if has_flag(args, "-I") {
        let resolved = match resolved {
            Some(resolved) => resolved,
            None => {
                let target = target.as_deref().unwrap_or_default();
                return CommandResult::err(format!("can't find session: {target}\n"));
            }
        };
        let pane = &mut st.window_mut(resolved.session, resolved.window).panes[resolved.pane].pane;
        if !pane.is_empty() {
            return CommandResult::err("pane is not empty\n");
        }
        return match context.input_file.as_ref() {
            Some(Ok(data)) => {
                pane.feed(data);
                CommandResult::ok("")
            }
            Some(Err(error)) => CommandResult::err(format!(
                "{}: -\n",
                io_error_message(&io::Error::from_raw_os_error(*error))
            )),
            None => CommandResult::ok(""),
        };
    }
    let positional_message = positionals(args, &["-t", "-c", "-d", "-F"])
        .into_iter()
        .next();
    if positional_message.is_some() && flag_value(args, "-F").is_some() {
        return CommandResult::err("only one of -F or argument must be given\n");
    }
    let message = positional_message
        .or_else(|| flag_value(args, "-F"))
        .unwrap_or(DISPLAY_MESSAGE_TEMPLATE);
    // Honor the *resolved* pane (e.g. `-t sess:win.{top}`), not just the window's
    // active pane, so pane-scoped variables reflect the target.
    let mut vars = match resolved {
        Some(resolved) => vars_full(
            st,
            &st.sessions()[resolved.session],
            resolved.window,
            resolved.pane,
            agents,
            st.marked_pane(),
        ),
        None => Vars::new(),
    };
    set_current_client_vars(
        st,
        context,
        resolved.map(|resolved| st.sessions()[resolved.session].id),
        flag_value(args, "-c"),
        &mut vars,
    );
    for (name, value) in st.env_iter() {
        vars.set(name, value);
    }
    if let Some(target) = target.as_deref() {
        if let Ok(entries) = st.format_option_entries(target) {
            for (name, value) in entries {
                vars.set(name, value);
            }
        }
    }
    let loops = resolved.map(|resolved| TreeLoops {
        st,
        session: resolved.session,
        window: resolved.window,
        agents,
    });
    let expanded = if has_flag(args, "-l") {
        message.to_string()
    } else {
        format::expand_time_with_jobs(
            message,
            &vars,
            loops.as_ref().map(|loops| loops as &dyn format::LoopSource),
            command_jobs(st),
            Some(&ServerFormatTree(st)),
        )
    };
    let mut out = String::new();
    // `-v` asks tmux's format engine to write its expansion trace to the
    // command client. Literal mode bypasses the format engine, so `-l -v`
    // produces no trace (and only prints the literal when `-p` is present).
    if has_flag(args, "-v") && !has_flag(args, "-l") {
        out.push_str(&format!("# expanding format: {message}\n"));
        out.push_str(&format!("# result is: {expanded}\n"));
    }
    if has_flag(args, "-p") {
        out.push_str(&expanded);
        out.push('\n');
    } else {
        let duration_ms = match flag_value(args, "-d") {
            Some(value) => match value.parse::<u32>() {
                Ok(value) => u64::from(value),
                Err(_) => return CommandResult::err(format!("delay {value}: invalid number\n")),
            },
            None => st
                .option_for_target(target.as_deref().unwrap_or_default(), "display-time")
                .and_then(|value| value.parse().ok())
                .unwrap_or(750),
        };
        let Some(resolved) = resolved else {
            return CommandResult::ok(out);
        };
        let session_id = st.sessions()[resolved.session].id;
        match st.send_client_message(
            flag_value(args, "-c"),
            context.tty_name.as_deref(),
            session_id,
            ClientMessage {
                text: expanded.clone(),
                duration_ms,
                bell: false,
            },
        ) {
            ClientMessageResult::CurrentControl => {
                out.push_str(&format!("%message {expanded}\n"));
            }
            ClientMessageResult::Queued | ClientMessageResult::NoClient => {}
            ClientMessageResult::TargetNotFound => {
                let target = flag_value(args, "-c").unwrap_or_default();
                return CommandResult::err(format!("can't find client: {target}\n"));
            }
        }
    }
    CommandResult::ok(out)
}

// ---- format context --------------------------------------------------------

/// A [`format::LoopSource`] over the live session tree, anchored at a resolved
/// target so `#{W:…}`/`#{P:…}` iterate the right session/window. Backs the
/// `#{S:…}`/`#{W:…}`/`#{P:…}` loop modifiers in `display-message`.
struct TreeLoops<'a> {
    st: &'a ServerState,
    session: usize,
    window: usize,
    agents: &'a PaneAgents,
}

/// The `#()` jobs of one command's format expansions. tmux runs jobs from every
/// expansion — `display-message`, `if-shell -F`, a status redraw — caching each
/// in the tree of the client the format belongs to.
pub(crate) struct CommandJobs {
    registry: Rc<super::status::FormatJobRegistry>,
    session_id: u32,
    cwd: Option<PathBuf>,
    environment: Rc<Vec<String>>,
}

impl CommandJobs {
    /// The runner for a command run by `context`.
    fn new(st: &ServerState, context: &ClientContext) -> Self {
        let (registry, client_session_id) = st.format_jobs_for_client(context.tty_name.as_deref());
        // A client contributes its own directory only while it has no session;
        // otherwise the job runs in the session that client is attached to.
        let cwd = match client_session_id.and_then(|id| st.session_by_id(id)) {
            Some(session) => super::status::job_cwd(Some(session), context.cwd.as_deref()),
            None => context.cwd.clone(),
        };
        let session = client_session_id
            .and_then(|id| st.session_by_id(id))
            .map(|session| session.name.clone());
        Self {
            registry,
            session_id: client_session_id.unwrap_or_default(),
            cwd,
            environment: st.job_environment(session.as_deref()),
        }
    }
}

impl format::FormatJobs for CommandJobs {
    fn run(&self, command: &str, expanded: String, vars: &Vars) -> String {
        self.registry.output_for(
            command,
            expanded,
            vars,
            self.session_id,
            self.cwd.clone(),
            Rc::clone(&self.environment),
            // Not a status redraw, so finishing must not invalidate one.
            false,
        )
    }
}

impl format::LoopSource for TreeLoops<'_> {
    fn items(&self, kind: format::LoopKind) -> Vec<Vars> {
        match kind {
            format::LoopKind::Session => {
                let marked = self.st.marked_pane();
                let mut order: Vec<&Session> = self.st.sessions().iter().collect();
                order.sort_by(|a, b| a.name.cmp(&b.name));
                order
                    .iter()
                    .map(|s| vars_for(self.st, s, s.active, self.agents, marked))
                    .collect()
            }
            format::LoopKind::Window => {
                let marked = self.st.marked_pane();
                let sess = &self.st.sessions()[self.session];
                (0..sess.windows.len())
                    .map(|w| vars_for(self.st, sess, w, self.agents, marked))
                    .collect()
            }
            format::LoopKind::Pane => {
                let marked = self.st.marked_pane();
                let sess = &self.st.sessions()[self.session];
                let panes = sess
                    .windows
                    .get(self.window)
                    .map(|_| self.st.session_window(sess, self.window).panes.len())
                    .unwrap_or(0);
                (0..panes)
                    .map(|p| vars_full(self.st, sess, self.window, p, self.agents, marked))
                    .collect()
            }
        }
    }
}

/// Build the format variables for a session and one of its windows, using that
/// window's *active* pane for the pane variables.
pub(super) fn vars_for(
    st: &ServerState,
    sess: &Session,
    win_idx: usize,
    agents: &PaneAgents,
    marked: Option<u32>,
) -> Vars {
    let pane_idx = sess
        .windows
        .get(win_idx)
        .map(|_| st.session_window(sess, win_idx).active)
        .unwrap_or(0);
    vars_full(st, sess, win_idx, pane_idx, agents, marked)
}

/// Build the format variables for a specific session/window/pane. Mirrors the
/// `#{...}` names real tmux exposes for the variables the suite exercises, plus
/// the hmux-private `#{pane_agent*}` variables (looked up in `agents` by pane
/// id; absent status expands to `"none"` and absent metadata expands empty).
/// tmux's `cmd_find_client` with no `-c`: a command client has no session of
/// its own, so client-scoped formats borrow the *current* client — the client
/// attached to the target session with the most recent activity.
///
/// Publishing them is what makes `#{client_key_table}`/`#{client_prefix}`
/// observable from a plain `display-message -p`, which is the only direct view
/// of the prefix and repeat-chain state.
fn set_current_client_vars(
    st: &ServerState,
    context: &ClientContext,
    session_id: Option<u32>,
    named: Option<&str>,
    vars: &mut Vars,
) {
    let clients = st.client_snapshots();
    // An explicit `-c` names the client the format is evaluated for; otherwise
    // the invoking client wins when it is one of the attached clients.
    let client = named
        .and_then(|name| clients.iter().find(|client| client.name == name))
        .or_else(|| {
            context
                .tty_name
                .as_deref()
                .and_then(|tty| clients.iter().find(|client| client.name == tty))
        })
        .or_else(|| {
            clients
                .iter()
                .filter(|client| session_id.is_none_or(|id| client.session_id == id))
                .max_by_key(|client| client.activity_micros)
        });
    let Some(client) = client else {
        return;
    };
    // The per-client viewport onto an oversized window. tmux leaves the offsets
    // *unset* rather than zero when the window fits, so a format testing
    // `#{window_bigger}` can tell "flush" from "panned to the origin".
    if let Some(view) = st.client_window_offset(&client.viewport()) {
        vars.set("window_bigger", if view.bigger { "1" } else { "0" });
        if view.bigger {
            vars.set("window_offset_x", view.ox.to_string())
                .set("window_offset_y", view.oy.to_string());
        }
    }
    clients::set_client_entry_vars(st, client, vars);
    // Whether the format's session is the one this client is attached to —
    // tmux's `format_cb_session_active`, which needs both in context.
    if let Some(session_id) = session_id {
        vars.set(
            "session_active",
            if client.session_id == session_id {
                "1"
            } else {
                "0"
            },
        );
    }
    let default_table = st
        .sessions()
        .iter()
        .find(|session| session.id == client.session_id)
        .map(|session| st.session_key_table(&session.name))
        .unwrap_or_else(|| super::state::DEFAULT_KEY_TABLE.to_string());
    vars.set("client_key_table", client.key_table.clone())
        // tmux reports a prefix as "the client left its default table", so a
        // live `bind -r` repeat chain reads as a held prefix too.
        .set(
            "client_prefix",
            if client.key_table == default_table {
                "0"
            } else {
                "1"
            },
        );
}

// The `*_mode_format` variables report each choose mode's default line
// format; tmux exposes them as constant server-wide strings.
const BUFFER_MODE_DEFAULT_FORMAT: &str = "#{t/p:buffer_created}: #{buffer_sample}";
const CLIENT_MODE_DEFAULT_FORMAT: &str = "#{t/p:client_activity}: session #{session_name}";
const TREE_MODE_DEFAULT_FORMAT: &str = concat!(
    "#{?pane_format,",
    "#{?pane_marked,#[reverse],}#{?pane_floating_flag,#[italics],}",
    "#{pane_current_command}#{pane_flags}",
    "#{?#{&&:#{pane_title},#{!=:#{pane_title},#{host_short}}},: \"#{pane_title}\",}",
    ",window_format,",
    "#{?window_marked_flag,#[reverse],}",
    "#{window_name}#{window_flags}",
    "#{?#{&&:#{==:#{window_panes},1},#{&&:#{pane_title},#{!=:#{pane_title},#{host_short}}}},: \"#{pane_title}\",}",
    ",",
    "#{session_windows} windows",
    "#{?session_grouped, ",
    "(group #{session_group}: ",
    "#{session_group_list}),",
    "}",
    "#{?session_attached, (attached),}",
    "}",
);

pub(super) fn vars_full(
    st: &ServerState,
    sess: &Session,
    win_idx: usize,
    pane_idx: usize,
    agents: &PaneAgents,
    marked: Option<u32>,
) -> Vars {
    let mut v = Vars::new();
    let mut group = st
        .sessions()
        .iter()
        .filter(|candidate| candidate.link_set_id == sess.link_set_id)
        .collect::<Vec<_>>();
    group.sort_by_key(|candidate| candidate.id);
    let grouped = st.is_grouped(sess);
    let group_name = st.session_group_name(sess).unwrap_or("").to_string();
    let group_list = if grouped {
        group
            .iter()
            .map(|candidate| candidate.name.as_str())
            .collect::<Vec<_>>()
            .join(",")
    } else {
        String::new()
    };
    let attached_clients = st.session_attached_client_names(sess);
    let group_attached_clients = st.session_group_attached_client_names(sess);
    // Server-global variables (same for every target). Values are volatile
    // (`pid` differs per process); conformance pins `host`/`host_short` (set by
    // `Vars::new`) by exact value — same machine as the tmux reference — and
    // `pid` only by truthiness.
    v.set("pid", server_pid().to_string())
        .set("socket_path", st.socket_path().to_string_lossy())
        .set("version", super::TMUX_VERSION)
        .set("server_sessions", st.sessions().len().to_string())
        .set("next_session_id", format!("${}", st.next_session_id()))
        .set("start_time", st.started_epoch().to_string())
        // The pinned tmux 3.7b oracle is built `--enable-sixel`, and so is
        // this: images are parsed, stored against the screen, and drawn to a
        // client that can take them. The daemon still loads no config file
        // (see README.md on server capability gaps).
        .set("sixel_support", "1")
        .set("config_files", "")
        .set("buffer_mode_format", BUFFER_MODE_DEFAULT_FORMAT)
        .set("client_mode_format", CLIENT_MODE_DEFAULT_FORMAT)
        .set("tree_mode_format", TREE_MODE_DEFAULT_FORMAT)
        // Context-type markers; the list commands override theirs to 1.
        .set("session_format", "0")
        .set("window_format", "0")
        .set("pane_format", "0");
    v.set("session_name", sess.name.clone())
        .set("session_id", format!("${}", sess.id))
        .set("session_windows", sess.windows.len().to_string())
        .set("session_created", sess.created_epoch.to_string())
        .set(
            "session_activity",
            (sess.activity_micros / 1_000_000).to_string(),
        )
        .set(
            "session_last_attached",
            // tmux leaves the variable empty until the session has been
            // attached at least once.
            if sess.last_attached_micros == 0 {
                String::new()
            } else {
                (sess.last_attached_micros / 1_000_000).to_string()
            },
        )
        .set("session_attached", attached_clients.len().to_string())
        .set("session_attached_list", attached_clients.join(","))
        .set(
            "session_many_attached",
            if attached_clients.len() > 1 { "1" } else { "0" },
        )
        .set("session_grouped", if grouped { "1" } else { "0" })
        .set("session_group", group_name)
        .set(
            "session_group_size",
            if grouped {
                group.len().to_string()
            } else {
                String::new()
            },
        )
        .set("session_group_list", group_list)
        .set(
            "session_group_attached",
            if grouped {
                group_attached_clients.len().to_string()
            } else {
                String::new()
            },
        )
        .set(
            "session_group_many_attached",
            if grouped {
                if group_attached_clients.len() > 1 {
                    "1"
                } else {
                    "0"
                }
            } else {
                ""
            },
        )
        .set(
            "session_group_attached_list",
            if grouped {
                group_attached_clients.join(",")
            } else {
                String::new()
            },
        )
        .set("session_alert", st.session_alert(sess))
        .set("session_alerts", st.session_alerts(sess))
        // 1 when this session holds the server's marked pane (`select-pane -m`).
        // The session working directory is the server process's cwd (same as the
        // stock tmux reference).
        .set(
            "session_marked",
            if marked.is_some_and(|m| {
                sess.windows
                    .iter()
                    .any(|link| st.window_for_link(link).panes.iter().any(|p| p.id == m))
            }) {
                "1"
            } else {
                "0"
            },
        )
        .set("session_path", current_dir())
        // MRU window-index stack (current window first). tmux keys `#{session_stack}`
        // on recency; the current window sits at the top.
        .set("session_stack", session_stack(sess));
    // tmux keeps a session's windows in an index-keyed tree, so the "first" and
    // "last" window are the lowest and highest index, not the ends of the list.
    let lowest_window_index = sess.windows.iter().map(|link| link.index).min();
    let highest_window_index = sess.windows.iter().map(|link| link.index).max();
    if let Some(index) = sess.windows.get(sess.active).map(|link| link.index) {
        v.set("active_window_index", index.to_string());
    }
    if let Some(index) = highest_window_index {
        v.set("last_window_index", index.to_string());
    }
    if let Some(link) = sess.windows.get(win_idx) {
        let win = st.window_for_link(link);
        // Neither `automatic-rename` nor `allow-rename` is consulted here. Both
        // are applied to the window when the thing that renames it happens — a
        // pane's `ESC k`, or the automatic-rename pass — so the stored name is
        // already what the options made of it, and, as in tmux, does not move
        // again until something wakes those paths.
        let is_active = win_idx == sess.active;
        let is_last = Some(win_idx) == sess.last_active;
        let active_sessions = st.window_active_session_list(win.id);
        let window_flags = st.printable_window_flags(sess, win_idx, true);
        let window_raw_flags = st.printable_window_flags(sess, win_idx, false);
        v.set("window_name", win.name.clone());
        v.set("window_index", link.index.to_string())
            .set("window_id", format!("@{}", win.id))
            .set("window_panes", win.panes.len().to_string())
            .set(
                "window_linked",
                if st.window_reference_count(win.id) > 1 {
                    "1"
                } else {
                    "0"
                },
            )
            .set(
                "window_linked_sessions",
                st.window_linked_session_count(win.id).to_string(),
            )
            .set(
                "window_linked_sessions_list",
                st.window_linked_session_list(win.id),
            )
            .set("window_active_sessions", active_sessions.len().to_string())
            .set("window_active_sessions_list", active_sessions.join(","))
            // A window has one size however many sessions link it: the size
            // `recalculate_sizes` derived from the clients that can see it.
            .set("window_width", win.cols.to_string())
            .set("window_height", win.rows.to_string())
            .set("window_active", if is_active { "1" } else { "0" })
            .set("window_flags", window_flags)
            .set("window_last_flag", if is_last { "1" } else { "0" })
            .set(
                "window_start_flag",
                if Some(link.index) == lowest_window_index {
                    "1"
                } else {
                    "0"
                },
            )
            .set(
                "window_end_flag",
                if Some(link.index) == highest_window_index {
                    "1"
                } else {
                    "0"
                },
            )
            .set(
                "window_bell_flag",
                if link.alert_flags & super::state::ALERT_BELL != 0 {
                    "1"
                } else {
                    "0"
                },
            )
            .set("window_activity", win.activity_epoch.to_string())
            .set(
                "window_activity_flag",
                if link.alert_flags & super::state::ALERT_ACTIVITY != 0 {
                    "1"
                } else {
                    "0"
                },
            )
            // tmux's session alert flags read the *context* window's alerts
            // (`format_cb_session_activity_flag` checks `ft->wl`), so they
            // mirror the window flags rather than OR over the session.
            .set(
                "session_bell_flag",
                if link.alert_flags & super::state::ALERT_BELL != 0 {
                    "1"
                } else {
                    "0"
                },
            )
            .set(
                "session_activity_flag",
                if link.alert_flags & super::state::ALERT_ACTIVITY != 0 {
                    "1"
                } else {
                    "0"
                },
            )
            .set(
                "session_silence_flag",
                if link.alert_flags & super::state::ALERT_SILENCE != 0 {
                    "1"
                } else {
                    "0"
                },
            )
            .set(
                "window_active_clients",
                st.window_active_client_names(win.id).len().to_string(),
            )
            .set(
                "window_active_clients_list",
                st.window_active_client_names(win.id).join(","),
            )
            .set(
                "window_silence_flag",
                if link.alert_flags & super::state::ALERT_SILENCE != 0 {
                    "1"
                } else {
                    "0"
                },
            )
            .set("window_zoomed_flag", if win.zoomed { "1" } else { "0" })
            .set("window_raw_flags", window_raw_flags)
            // 1 when this window holds the server's marked pane.
            .set(
                "window_marked_flag",
                if marked.is_some_and(|m| win.panes.iter().any(|p| p.id == m)) {
                    "1"
                } else {
                    "0"
                },
            )
            // Position of this window in the session's MRU stack (0 = current).
            .set(
                "window_stack_index",
                window_stack_index(sess, win_idx).to_string(),
            )
            // Default per-cell pixel size tmux reports for a client-less window.
            .set("window_cell_width", "16")
            .set("window_cell_height", "32")
            // The window's pane layout string (with tmux's checksum prefix).
            // Zoom leaves the saved layout in `window_layout` while the
            // visible layout is the single full-window cell tmux swaps in.
            .set("window_layout", window_layout(win))
            .set(
                "window_visible_layout",
                if win.zoomed {
                    window_zoomed_layout(win)
                } else {
                    window_layout(win)
                },
            );
        if let Some(p) = win.panes.get(pane_idx) {
            let (window_width, window_height) = (win.cols, win.rows);
            // A zoomed pane lives in tmux's single full-window layout cell, so
            // its geometry formats grow to the window; the saved layout only
            // answers for the panes the zoom hid.
            let zoomed_full = win.zoomed && pane_idx == win.active;
            let pane_rect = if zoomed_full {
                super::state::PaneRect {
                    top: 0,
                    left: 0,
                    height: window_height,
                    width: window_width,
                }
            } else {
                win.pane_rect(p.id).unwrap_or(super::state::PaneRect {
                    top: 0,
                    left: 0,
                    height: window_height,
                    width: window_width,
                })
            };
            let pane_base_index = win
                .options(st.global_options())
                .get("pane-base-index")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0);
            let pane_title = st.pane_title(p).unwrap_or_else(format::hostname);
            let death = p.pane.death();
            // A session option, so it is read through the session view rather
            // than the window's.
            let history_limit = sess
                .options(st.global_options())
                .get("history-limit")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(2000);
            v.set("pane_index", (pane_base_index + pane_idx).to_string())
                .set("pane_id", format!("%{}", p.id))
                .set("pane_start_command", p.start_command.clone())
                // A pane spawned without an explicit `-c` inherits the server's
                // working directory, the same one `#{session_path}` reports.
                .set(
                    "pane_start_path",
                    p.pane
                        .start_path()
                        .map(|path| path.to_string_lossy().into_owned())
                        .unwrap_or_else(current_dir),
                )
                // Only meaningful while a mode is up: the flag records output
                // the frozen grid has not caught up with, and leaving the mode
                // is what clears it.
                .set(
                    "pane_unseen_changes",
                    if p.mode.is_some() && p.unseen_changes {
                        "1"
                    } else {
                        "0"
                    },
                )
                .set("pane_width", pane_rect.width.to_string())
                .set("pane_height", pane_rect.height.to_string())
                .set("pane_left", pane_rect.left.to_string())
                .set("pane_top", pane_rect.top.to_string())
                // The last column/row the pane covers, not one past it, so a
                // single-column pane reports the same value for both edges.
                .set(
                    "pane_right",
                    pane_rect
                        .left
                        .saturating_add(pane_rect.width)
                        .saturating_sub(1)
                        .to_string(),
                )
                .set(
                    "pane_bottom",
                    pane_rect
                        .top
                        .saturating_add(pane_rect.height)
                        .saturating_sub(1)
                        .to_string(),
                )
                .set("pane_x", pane_rect.left.to_string())
                .set("pane_y", pane_rect.top.to_string())
                .set("pane_z", win.pane_z_index(pane_idx).to_string())
                .set(
                    "pane_floating_flag",
                    if p.floating.is_some() { "1" } else { "0" },
                )
                .set(
                    "pane_active",
                    if pane_idx == win.active { "1" } else { "0" },
                )
                .set(
                    "pane_last",
                    if Some(pane_idx) == win.last_pane {
                        "1"
                    } else {
                        "0"
                    },
                )
                .set("pane_flags", win.printable_pane_flags(pane_idx))
                // Zooming a pane makes it active first, so the flag rides the
                // window's zoom state and its active pane together.
                .set(
                    "pane_zoomed_flag",
                    if win.zoomed && pane_idx == win.active {
                        "1"
                    } else {
                        "0"
                    },
                )
                // The pid of the process the pane forked for its pty; empty once
                // there is no child left to report.
                .set(
                    "pane_pid",
                    p.pane
                        .child_pid()
                        .map(|pid| pid.to_string())
                        .unwrap_or_default(),
                )
                .set("pane_tty", p.pane.tty_name().unwrap_or_default())
                // State flags not derived from the terminal grid. A pane is
                // dead once its child has been waited for and the pane is
                // still here — tmux's `wp->fd == -1 && PANE_STATUSREADY`,
                // which only `remain-on-exit` leaves observable.
                .set("pane_dead", if death.is_some() { "1" } else { "0" })
                .set(
                    "pane_dead_status",
                    death
                        .and_then(|death| death.status)
                        .map(|status| status.to_string())
                        .unwrap_or_default(),
                )
                .set(
                    "pane_dead_signal",
                    death
                        .and_then(|death| death.signal)
                        .map(|signal| signal.to_string())
                        .unwrap_or_default(),
                )
                .set(
                    "pane_dead_time",
                    death
                        .map(|death| unix_seconds(death.at).to_string())
                        .unwrap_or_default(),
                )
                .set("pane_key_mode", st.pane_key_mode_name(p))
                .set("pane_in_mode", if p.mode.is_some() { "1" } else { "0" })
                .set("pane_input_off", if p.input_off { "1" } else { "0" })
                .set("pane_mode", p.mode.clone().unwrap_or_default())
                .set(
                    "pane_search_string",
                    p.search_string.as_deref().unwrap_or(""),
                )
                // tmux reads this off the *pane's* option view, so a
                // pane-scoped `set-option -p synchronize-panes` shows through
                // where the window's own value would not.
                .set(
                    "pane_synchronized",
                    if p.options(win, st.global_options())
                        .get("synchronize-panes")
                        .is_some_and(|value| value == "on" || value == "1")
                    {
                        "1"
                    } else {
                        "0"
                    },
                )
                .set("pane_at_top", if pane_rect.top == 0 { "1" } else { "0" })
                .set(
                    "pane_at_bottom",
                    if pane_rect.top.saturating_add(pane_rect.height) == window_height {
                        "1"
                    } else {
                        "0"
                    },
                )
                .set("pane_at_left", if pane_rect.left == 0 { "1" } else { "0" })
                .set(
                    "pane_at_right",
                    if pane_rect.left.saturating_add(pane_rect.width) == window_width {
                        "1"
                    } else {
                        "0"
                    },
                )
                .set(
                    "cursor_x",
                    p.pane
                        .cursor_position()
                        .map(|(x, _)| x)
                        .unwrap_or(0)
                        .to_string(),
                )
                .set(
                    "cursor_y",
                    p.pane
                        .cursor_position()
                        .map(|(_, y)| y)
                        .unwrap_or(0)
                        .to_string(),
                )
                .set("cursor_character", pane_cursor_character(&p.pane))
                // tmux drops history rows past `history-limit` as they scroll
                // off, so the size saturates there. Ghostty measures its own
                // scrollback in bytes rather than rows, so hmux applies the
                // limit where the rows are counted and read back instead.
                .set(
                    "history_size",
                    p.pane
                        .scrollback_rows()
                        .unwrap_or(0)
                        .min(history_limit)
                        .to_string(),
                )
                .set("history_limit", history_limit.to_string())
                // `pane_format` marks a pane-level format context (always 1 here,
                // since this table is built per pane).
                .set("pane_format", "1")
                .set_lazy("history_bytes", {
                    // The byte totals walk every grid row, so they stay a
                    // deferred computation until a format asks.
                    let observation = p.pane.observation_state();
                    move || {
                        observation
                            .history_byte_formats()
                            .map(|(bytes, _)| bytes)
                            .unwrap_or_default()
                    }
                })
                .set_lazy("history_all_bytes", {
                    let observation = p.pane.observation_state();
                    move || {
                        observation
                            .history_byte_formats()
                            .map(|(_, all)| all)
                            .unwrap_or_default()
                    }
                })
                // `pane_marked` is 1 only for the server's marked pane;
                // `pane_marked_set` is 1 whenever any pane is marked server-wide.
                .set("pane_marked", if marked == Some(p.id) { "1" } else { "0" })
                .set("pane_marked_set", if marked.is_some() { "1" } else { "0" })
                .set("pane_pipe", if p.pane.pipe_active() { "1" } else { "0" })
                // tmux leaves `pane_pipe_pid` unset while no pipe is open.
                .set(
                    "pane_pipe_pid",
                    p.pane
                        .pipe_pid()
                        .map(|pid| pid.to_string())
                        .unwrap_or_default(),
                )
                .set("pane_bg", p.pane.background_color())
                // Default terminal tab stops sit every 8 columns.
                .set(
                    "pane_tabs",
                    p.pane
                        .tab_stops()
                        .iter()
                        .map(u16::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                )
                // The pane title defaults to the host name, as real tmux does.
                .set("pane_title", pane_title);
            // The pane's working directory is read live from its child (as
            // real tmux does via osdep_get_cwd), so it follows a shell that
            // `cd`s. A childless (inert) pane has no live cwd and falls back
            // to the server process's cwd, matching stock tmux. The `/proc`
            // reads behind both variables run only if a format names them.
            let probe = p.pane.process_probe();
            {
                let probe = probe.clone();
                v.set_lazy("pane_current_path", move || {
                    probe
                        .as_ref()
                        .and_then(|probe| probe.current_path())
                        .unwrap_or_else(current_dir)
                });
            }
            {
                let probe = probe.clone();
                v.set_lazy("pane_current_command", move || {
                    probe
                        .as_ref()
                        .and_then(|probe| probe.current_command())
                        .unwrap_or_default()
                });
            }
            set_terminal_mode_vars(&p.pane, &mut v);
            if let Some(copy) = p.copy.as_ref() {
                let view_top = copy.grid.scrollback_rows.saturating_sub(copy.scroll);
                let search = copy.search.as_ref();
                v.set("copy_cursor_x", copy.cursor.col.to_string())
                    .set(
                        "copy_cursor_y",
                        copy.cursor.row.saturating_sub(view_top).to_string(),
                    )
                    .set("scroll_position", copy.scroll.to_string())
                    .set(
                        "pane_search_string",
                        search
                            .map(|search| search.pattern.as_str())
                            .or(p.search_string.as_deref())
                            .unwrap_or(""),
                    )
                    .set(
                        "search_present",
                        if search.is_some_and(|search| !search.matches.is_empty()) {
                            "1"
                        } else {
                            "0"
                        },
                    )
                    .set("search_timed_out", "0")
                    .set("search_count_partial", "0")
                    .set(
                        "search_match",
                        search.map(|search| search.pattern.as_str()).unwrap_or(""),
                    )
                    .set("copy_position", copy.scroll.to_string())
                    .set("copy_position_limit", copy.grid.scrollback_rows.to_string())
                    .set("rectangle_toggle", if copy.rectangle { "1" } else { "0" })
                    .set(
                        "selection_mode",
                        match copy.selection_mode {
                            super::state::CopySelectionMode::Character => "char",
                            super::state::CopySelectionMode::Word => "word",
                            super::state::CopySelectionMode::Line => "line",
                        },
                    )
                    .set(
                        "selection_active",
                        if copy
                            .selection
                            .as_ref()
                            .is_some_and(|selection| selection.active)
                        {
                            "1"
                        } else {
                            "0"
                        },
                    )
                    .set(
                        "selection_present",
                        if copy
                            .selection
                            .as_ref()
                            .is_some_and(|selection| selection.anchor != selection.end)
                        {
                            "1"
                        } else {
                            "0"
                        },
                    );
                if let Some(selection) = copy.selection.as_ref() {
                    v.set("selection_start_x", selection.anchor.1.to_string())
                        .set(
                            "selection_start_y",
                            selection.anchor.0.saturating_sub(view_top).to_string(),
                        )
                        .set("selection_end_x", selection.end.1.to_string())
                        .set(
                            "selection_end_y",
                            selection.end.0.saturating_sub(view_top).to_string(),
                        );
                }
                if let Some(count) = copy.search_count {
                    v.set("search_count", count.to_string());
                }
                if let Some(row) = copy.grid.rows.get(copy.cursor.row) {
                    let line = row
                        .cells
                        .iter()
                        .filter(|cell| {
                            !matches!(cell.width, CellWidth::SpacerTail | CellWidth::SpacerHead)
                        })
                        .map(|cell| {
                            if cell.text.is_empty() {
                                " "
                            } else {
                                cell.text.as_str()
                            }
                        })
                        .collect::<String>()
                        .trim_end()
                        .to_string();
                    v.set("copy_cursor_line", line);
                    if let Some(cell) = row.cells.get(copy.cursor.col) {
                        v.set(
                            "copy_cursor_hyperlink",
                            cell.hyperlink.as_deref().unwrap_or(""),
                        );
                        let separators = " !\"#$%&'()*+,-./:;<=>?@[\\]^`{|}~";
                        let class = |text: &str| {
                            if text.trim().is_empty() {
                                0
                            } else if text.chars().any(|ch| separators.contains(ch)) {
                                1
                            } else {
                                2
                            }
                        };
                        let wanted = class(&cell.text);
                        let mut start = copy.cursor.col;
                        while start > 0 && class(&row.cells[start - 1].text) == wanted {
                            start -= 1;
                        }
                        let mut end = copy.cursor.col.saturating_add(1);
                        while end < row.cells.len() && class(&row.cells[end].text) == wanted {
                            end += 1;
                        }
                        let word = row.cells[start..end]
                            .iter()
                            .map(|cell| cell.text.as_str())
                            .collect::<String>();
                        v.set("copy_cursor_word", word);
                    }
                }
            }
            // hmux-private agent status (see PROTOCOL.md). Absent from the hub
            // (no agent, or non-native engine) reads as empty metadata + "none".
            let (agent, state, agent_emoji, pid, session_id, model) =
                match agents.get(&PaneId(p.id)) {
                    Some(status) => (
                        status.agent,
                        status.state.wire_str(),
                        status.state.emoji(),
                        status.pid.map(|pid| pid.to_string()).unwrap_or_default(),
                        status.session_id.clone().unwrap_or_default(),
                        status.model.clone().unwrap_or_default(),
                    ),
                    None => ("", "none", "", String::new(), String::new(), String::new()),
                };
            v.set("pane_agent", agent)
                .set("pane_agent_state", state)
                .set("pane_agent_pid", pid)
                .set("pane_agent_session_id", session_id)
                .set("pane_agent_model", model);

            // The compact glyph every pane gets. An agent pane reports its
            // lifecycle state; any other pane reports what it is running, so
            // this is never empty and a format need not branch on whether an
            // agent was found. Resolved lazily because telling a command that
            // is waiting on you from one that is working costs a `/proc` read.
            //
            // The agent's *label* is what decides which half applies, not its
            // emoji. The observer reports a state for every pane it watches —
            // an ordinary shell that exits is `exited` just as an agent is —
            // and only a pane that named an agent should be labelled as one.
            let alternate_on = p.pane.alternate_screen().0;
            let dead = death.is_some();
            v.set_lazy("pane_state_emoji", move || {
                if !agent.is_empty() && !agent_emoji.is_empty() {
                    return agent_emoji.to_string();
                }
                PaneClass::classify(probe.as_ref(), alternate_on, dead)
                    .emoji()
                    .to_string()
            });
        }
    }
    // `hook*` variables of the hook body currently executing, if any.
    for (key, value) in st.hook_format_vars() {
        v.set(key.clone(), value.clone());
    }
    set_mouse_vars(st, sess, win_idx, pane_idx, &mut v);
    v
}

/// Publish the pane terminal modes tmux reads out of `screen->mode`.
///
/// `cursor_very_visible` is a constant 0: tmux only ever sets
/// `MODE_CURSOR_VERY_VISIBLE` from a client terminal's own `Cvvis` capability,
/// never from a pane's output, so no sequence a pane can emit turns it on.
fn set_terminal_mode_vars(pane: &super::pane::Pane, v: &mut Vars) {
    let modes = pane.terminal_modes();
    let osc = pane.osc_state();
    let (upper, lower) = pane.scroll_region();
    let (alternate_on, saved_x, saved_y) = pane.alternate_screen();
    let flag = |set: bool| if set { "1" } else { "0" };
    v.set("alternate_on", flag(alternate_on))
        .set("alternate_saved_x", saved_x.to_string())
        .set("alternate_saved_y", saved_y.to_string())
        .set("scroll_region_upper", upper.to_string())
        .set("scroll_region_lower", lower.to_string())
        .set("cursor_colour", osc.cursor_colour)
        .set("pane_fg", osc.foreground)
        .set("pane_path", osc.path)
        .set("pane_pb_state", osc.progress_state)
        .set("pane_pb_progress", osc.progress_value.to_string())
        .set("insert_flag", flag(modes.insert))
        .set("origin_flag", flag(modes.origin))
        .set("wrap_flag", flag(modes.wrap))
        .set("cursor_flag", flag(modes.cursor_visible))
        .set("cursor_blinking", flag(modes.cursor_blinking))
        .set("cursor_very_visible", "0")
        .set("keypad_flag", flag(modes.keypad))
        .set("keypad_cursor_flag", flag(modes.cursor_keys))
        .set("synchronized_output_flag", flag(modes.synchronized_output))
        .set("bracket_paste_flag", flag(modes.bracketed_paste))
        .set("cursor_shape", modes.cursor_shape.name());
}

/// The pane `#{mouse_pane}` reports for a status-line click, which names a pane
/// only when the range was a pane range. Everything else resolves the way
/// tmux's `cmd_mouse_window` does and takes that window's active pane.
fn status_mouse_pane(st: &ServerState, target: &super::mouse::MouseTarget) -> Option<u32> {
    if let Some(pane_id) = target.pane_id {
        return Some(pane_id);
    }
    let session = st
        .sessions()
        .iter()
        .find(|session| session.id == target.session_id)?;
    let link = match target.window_id {
        Some(id) => session.windows.iter().find(|link| link.id == id)?,
        None => session.windows.get(session.active)?,
    };
    let window = st.window_for_link(link);
    window.panes.get(window.active).map(|pane| pane.id)
}

/// Publish `#{mouse_*}`.
///
/// The `*_flag` variables describe the *pane* (tmux reads `ft->wp->base.mode`),
/// so they are available with no mouse event in scope; everything else needs
/// the event the running command was dispatched from.
fn set_mouse_vars(st: &ServerState, sess: &Session, win_idx: usize, pane_idx: usize, v: &mut Vars) {
    let modes = sess
        .windows
        .get(win_idx)
        .map(|link| st.window_for_link(link))
        .and_then(|window| window.panes.get(pane_idx))
        .map(|node| node.pane.mouse_modes())
        .unwrap_or_default();
    let flag = |set: bool| if set { "1" } else { "0" };
    v.set("mouse_any_flag", flag(modes.any()))
        .set(
            "mouse_standard_flag",
            flag(modes.tracking == Some(super::pane::MouseTrackingMode::Standard)),
        )
        .set(
            "mouse_button_flag",
            flag(modes.tracking == Some(super::pane::MouseTrackingMode::Button)),
        )
        .set(
            "mouse_all_flag",
            flag(modes.tracking == Some(super::pane::MouseTrackingMode::All)),
        )
        .set("mouse_sgr_flag", flag(modes.sgr))
        .set("mouse_utf8_flag", flag(modes.utf8));

    let Some(mouse) = st.command_mouse() else {
        return;
    };
    let Some(target) = mouse.target.as_ref() else {
        return;
    };
    if let Some(line) = target.status_line {
        v.set("mouse_x", mouse.position.x.to_string())
            .set("mouse_y", line.to_string())
            .set("mouse_status_line", line.to_string());
        if let Some(range) = target.status_range.as_ref() {
            v.set("mouse_status_range", super::mouse::range_name(range));
        }
        // tmux's `cmd_mouse_pane` still answers for a status click: the lookup
        // falls back through the mouse window — or the session's current window
        // when the click named none — to that window's active pane.
        if let Some(pane_id) = status_mouse_pane(st, target) {
            v.set("mouse_pane", format!("%{pane_id}"));
        }
        return;
    }
    let Some(position) = target.local_position else {
        return;
    };
    v.set("mouse_x", position.x.to_string())
        .set("mouse_y", position.y.to_string());
    let Some(pane_id) = target.pane_id else {
        return;
    };
    let pane_target = format!("%{pane_id}");
    v.set("mouse_pane", &pane_target);
    let Some(resolved) = st.resolve(&pane_target) else {
        return;
    };
    let pane = &st.window(resolved.session, resolved.window).panes[resolved.pane].pane;
    let separators = st
        .option_for_target(&pane_target, "word-separators")
        .unwrap_or(" !\"#$%&'()*+,-./:;<=>?@[\\]^`{|}~")
        .to_string();
    let (word, line) = super::mouse::grid_word_and_line(pane, position, &separators);
    v.set("mouse_word", word).set("mouse_line", line).set(
        "mouse_hyperlink",
        super::mouse::grid_hyperlink(pane, position),
    );
}

fn pane_cursor_character(pane: &super::pane::Pane) -> String {
    let Ok((x, y)) = pane.cursor_position() else {
        return String::new();
    };
    let history = pane.scrollback_rows().unwrap_or(0);
    pane.dump_plain_row(history + y as usize)
        .ok()
        .and_then(|row| {
            row.lines()
                .next()
                .and_then(|line| line.chars().nth(x as usize))
        })
        .map(|character| character.to_string())
        .unwrap_or_else(|| " ".to_string())
}

/// The server process's working directory. This backs `#{session_path}` and is
/// the fallback for `#{pane_current_path}` when a pane has no live child cwd to
/// read (an inert pane). It is the directory hmux was started in — the same cwd
/// stock tmux inherits, so both targets report the same path.
/// Empty on failure.
fn current_dir() -> String {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The session's MRU window-index stack (`#{session_stack}`): the current window
/// first, followed by tmux's visited-window stack.
fn session_stack(sess: &Session) -> String {
    std::iter::once(sess.windows.get(sess.active).map(|link| link.index))
        .chain(sess.last_windows.iter().map(|id| {
            sess.windows
                .iter()
                .find(|link| link.link_id == *id)
                .map(|link| link.index)
        }))
        .flatten()
        .map(|index| index.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// The window's position in its session's MRU stack (`#{window_stack_index}`):
/// 0 for the current or an unvisited window, otherwise its one-based position
/// in the visited-window stack.
fn window_stack_index(sess: &Session, win_idx: usize) -> usize {
    if win_idx == sess.active {
        return 0;
    }
    let Some(link) = sess.windows.get(win_idx) else {
        return 0;
    };
    sess.last_windows
        .iter()
        .position(|id| *id == link.link_id)
        .map_or(0, |position| position + 1)
}

/// Build the window's layout string (`#{window_layout}`), matching tmux's
/// `layout_dump` output plus its 4-hex-digit checksum prefix.
fn window_layout(win: &super::state::Window) -> String {
    let body = win.layout.dump();
    format!("{:04x},{body}", layout_checksum(&body))
}

/// The visible layout of a zoomed window: the one full-window cell holding the
/// zoomed pane, which is what tmux's live `layout_root` dumps while the real
/// layout waits in `saved_layout_root`.
fn window_zoomed_layout(win: &super::state::Window) -> String {
    let pane_id = win.panes.get(win.active).map(|pane| pane.id).unwrap_or(0);
    let body = format!("{}x{},0,0,{}", win.cols, win.rows, pane_id);
    format!("{:04x},{body}", layout_checksum(&body))
}

/// tmux's `layout_checksum` (layout-custom.c): a 16-bit rotate-and-add over the
/// layout body, rendered as the `%04x` prefix on `#{window_layout}`.
fn layout_checksum(s: &str) -> u16 {
    let mut csum: u16 = 0;
    for &b in s.as_bytes() {
        csum = (csum >> 1).wrapping_add((csum & 1) << 15);
        csum = csum.wrapping_add(b as u16);
    }
    csum
}

/// A wall-clock instant as the seconds since the epoch a `FORMAT_TABLE_TIME`
/// variable reports.
fn unix_seconds(at: SystemTime) -> u64 {
    at.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

/// The server process id (`#{pid}`).
fn server_pid() -> i32 {
    // SAFETY: `getpid` reads no memory and always succeeds.
    unsafe { libc::getpid() }
}

/// The "current" session for a command with no explicit target. Client-scoped
/// execution supplies its stable session ID; other command clients fall back to
/// the newest session as an approximation of tmux's most-recently-active one.
fn current_session(st: &ServerState) -> Option<String> {
    st.command_session_name()
        .or_else(|| st.sessions().last().map(|s| s.name.clone()))
}

/// The target a command whose `-t` is absent runs against: tmux uses the
/// client's current pane, which is the current session's current window's
/// active pane. Spelled with the `:` so the session name stays a *session* —
/// a bare `0` is pane 0 of the current window in tmux's target grammar, which
/// is not what "no target given" means.
fn current_target(st: &ServerState) -> Option<String> {
    current_session(st).map(|session| format!("{session}:"))
}

/// Parse a `new-window -t` target into `(session, explicit_index)`.
///
/// - `sess:N`  → session `sess`, explicit index `N`.
/// - `sess:`   → session `sess`, append at the next free index (`None`).
/// - `N` (numeric, no colon) → a *window* target in the current session at index
///   `N` (tmux resolves a bare number as a window index, hence `new-window -t 0`
///   → "index 0 in use").
/// - `name` (non-numeric, no colon) → session `name`, append.
/// - absent → current session, append.
///
/// Returns `None` only when no current session can be established.
fn parse_window_target(target: Option<&str>, st: &ServerState) -> Option<(String, Option<u32>)> {
    match target {
        None => current_session(st).map(|s| (s, None)),
        Some(t) => match t.split_once(':') {
            Some((sess, win)) => {
                let session = if sess.is_empty() {
                    current_session(st)?
                } else {
                    sess.to_string()
                };
                let explicit = if win.is_empty() {
                    None
                } else {
                    win.parse().ok()
                };
                Some((session, explicit))
            }
            None => match t.parse::<u32>() {
                Ok(idx) => current_session(st).map(|s| (s, Some(idx))),
                Err(_) => Some((t.to_string(), None)),
            },
        },
    }
}

/// Resolve the *anchor* window index for a relative `new-window -a`/`-b`, given
/// the session and the window-index part of `-t` (as parsed by
/// [`parse_window_target`]). `None` explicit → the session's active window's
/// index. `Some(N)` → `N` when a window at index `N` exists, else `None` to signal
/// the caller should fall back to the explicit-index create path (matching tmux,
/// which ignores `-a`/`-b` when the target index doesn't exist).
fn anchor_window_index(session: &str, explicit: Option<u32>, st: &ServerState) -> Option<u32> {
    let sess = st.find(session)?;
    match explicit {
        None => sess.windows.get(sess.active).map(|w| w.index),
        Some(n) => sess.windows.iter().any(|w| w.index == n).then_some(n),
    }
}

/// Run a window-lifecycle command that takes a `session[:window]` target
/// (`kill-window`, `select-window`). The target defaults to the current
/// session's active window. tmux's diagnostics come straight from the state
/// method; we only append the trailing newline.
fn target_command(
    args: &[String],
    st: &mut ServerState,
    f: fn(&mut ServerState, &str) -> std::io::Result<()>,
) -> CommandResult {
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_session(st).map(|session| format!("{session}:")));
    let target = match target {
        Some(t) => t,
        None => return CommandResult::err("can't establish current session\n"),
    };
    match f(st, &target) {
        Ok(()) => CommandResult::ok(""),
        Err(e) => CommandResult::err(format!("{e}\n")),
    }
}

/// Run a session-scoped navigation command (`next-window`, `previous-window`,
/// `last-window`). The `-t` target names a session (its window part, if any, is
/// ignored); it defaults to the current session.
fn session_command(
    args: &[String],
    st: &mut ServerState,
    f: fn(&mut ServerState, &str) -> std::io::Result<()>,
) -> CommandResult {
    let session = flag_value(args, "-t")
        .map(|t| t.split(':').next().unwrap_or(t).to_string())
        .or_else(|| current_session(st));
    let session = match session {
        Some(s) => s,
        None => return CommandResult::err("can't establish current session\n"),
    };
    match f(st, &session) {
        Ok(()) => CommandResult::ok(""),
        Err(e) => CommandResult::err(format!("{e}\n")),
    }
}

/// `clear-history [-H] [-t target]`. Ghostty owns the pane's scrollback, so ask
/// it to erase history while retaining the viewport, then leave any copy mode
/// associated with the pane.
fn clear_history(args: &[String], st: &mut ServerState) -> CommandResult {
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let target = match target {
        Some(target) => target,
        None => return CommandResult::err("can't establish current session\n"),
    };
    match st.clear_pane_history(&target) {
        Ok(()) => CommandResult::ok(""),
        Err(_) => CommandResult::err(format!("{}\n", st.pane_target_error(&target))),
    }
}

/// `split-window [-t target] [-P] [-F format]`. Adds a pane to the target
/// window (the target's, or the current session's active window). The new pane
/// becomes active. With `-P`, prints the new pane via `-F` (or
/// `NEW_WINDOW_TEMPLATE`).
fn split_window(args: &[String], st: &mut ServerState, context: &ClientContext) -> CommandResult {
    const VALUE_FLAGS: &[&str] = &["-c", "-e", "-F", "-l", "-m", "-p", "-t"];
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let target = match target {
        Some(t) => t,
        None => return CommandResult::err("can't establish current session\n"),
    };
    // split-window's `-t` is a *pane* target, so a resolve failure reports
    // tmux's pane diagnostic ("can't find pane: <t>"), not the session one.
    let resolved = match st.resolve(&target) {
        Some(target) => target,
        None => return CommandResult::err(format!("{}\n", st.pane_target_error(&target))),
    };
    // `-d` splits in the background: the new pane is created but the original
    // pane stays active (tmux's default is to follow to the new pane).
    let select = !has_flag(args, "-d");
    // `-b` inserts the new pane *before* the target pane rather than appending.
    let before = has_flag(args, "-b");
    let direction = if has_flag(args, "-h") {
        SplitDirection::LeftRight
    } else {
        SplitDirection::TopBottom
    };
    // `-l size` (cells, or `N%`) and `-p percentage` pin the *new* pane's size
    // on the split axis; percentages are of the target pane's current extent.
    let new_size = {
        let axis_total = {
            let sess = &st.sessions()[resolved.session];
            let win = st.window_for_link(&sess.windows[resolved.window]);
            let rect =
                win.pane_rect(win.panes[resolved.pane].id)
                    .unwrap_or(super::state::PaneRect {
                        top: 0,
                        left: 0,
                        height: win.rows,
                        width: win.cols,
                    });
            match direction {
                SplitDirection::LeftRight => rect.width,
                SplitDirection::TopBottom => rect.height,
            }
        };
        let percentage_of = |value: &str| {
            value
                .parse::<u32>()
                .ok()
                .map(|percentage| (u32::from(axis_total) * percentage / 100) as u16)
        };
        let parsed = if let Some(value) = flag_value(args, "-l") {
            Some(match value.strip_suffix('%') {
                Some(percentage) => percentage_of(percentage),
                None => value.parse::<u16>().ok(),
            })
        } else {
            flag_value(args, "-p").map(|value| percentage_of(value))
        };
        match parsed {
            Some(None) => return CommandResult::err("create pane failed: size invalid\n"),
            Some(size) => size,
            None => None,
        }
    };
    let explicit_cwd = command_option_value(args, "-c", VALUE_FLAGS).map(PathBuf::from);
    let cwd = explicit_cwd.as_deref().or(context.cwd.as_deref());
    let command = trailing_command(args, VALUE_FLAGS);
    let empty = has_flag(args, "-E") || has_flag(args, "-I");
    if empty && command.iter().any(|word| !word.is_empty()) {
        return CommandResult::err("command cannot be given for empty pane\n");
    }
    let argv = pane_command_argv(command.as_slice(), st, Some(&target));
    let spawn_environment = flag_values(args, "-e");
    let argv = pane_argv(argv, context, &spawn_environment, st, SpawnSession::Existing(&target));
    let created = if empty {
        st.split_window_direction_with_spec(
            &target,
            select,
            before,
            direction,
            PaneSpec::Inert,
            new_size,
        )
    } else {
        st.split_window_direction_with_spawn(
            &target, select, before, direction, &argv, cwd, new_size,
        )
    };
    // `-k` (and `-m format`) keep the pane in place when its command exits:
    // tmux sets the pane's remain-on-exit to `key` plus the format under `-m`.
    if let Ok(pane) = &created {
        if has_flag(args, "-k") || flag_value(args, "-m").is_some() {
            let new_target = Target {
                session: resolved.session,
                window: resolved.window,
                pane: *pane,
            };
            st.set_pane_option(new_target, "remain-on-exit", "key");
            if let Some(message) = flag_value(args, "-m") {
                st.set_pane_option(new_target, "remain-on-exit-format", message);
            }
        }
    }
    match created {
        Ok(pane) if has_flag(args, "-P") => {
            let sess = &st.sessions()[resolved.session];
            // `-P` prints the *newly created* pane, at whichever index the split
            // placed it (end by default, the target's index under `-b`) — the
            // print target even under `-d`, where the active pane is the original.
            let template = flag_value(args, "-F").unwrap_or(NEW_WINDOW_TEMPLATE);
            let marked = st.marked_pane();
            let line = expand_command_format(
                st,
                template,
                &vars_full(st, sess, resolved.window, pane, &PaneAgents::new(), marked),
                None,
            );
            if has_flag(args, "-I") {
                match context.input_file.as_ref() {
                    Some(Ok(data)) => st.window(resolved.session, resolved.window).panes[pane]
                        .pane
                        .feed(data),
                    Some(Err(error)) => {
                        return CommandResult::err(format!(
                            "{}: -\n",
                            io_error_message(&io::Error::from_raw_os_error(*error))
                        ))
                    }
                    None => {}
                }
            }
            CommandResult::ok(format!("{line}\n"))
        }
        Ok(pane) => {
            if has_flag(args, "-I") {
                match context.input_file.as_ref() {
                    Some(Ok(data)) => st.window(resolved.session, resolved.window).panes[pane]
                        .pane
                        .feed(data),
                    Some(Err(error)) => {
                        return CommandResult::err(format!(
                            "{}: -\n",
                            io_error_message(&io::Error::from_raw_os_error(*error))
                        ))
                    }
                    None => {}
                }
            }
            CommandResult::ok("")
        }
        Err(e) => CommandResult::err(format!("{e}\n")),
    }
}

fn new_pane(args: &[String], st: &mut ServerState, context: &ClientContext) -> CommandResult {
    const VALUE_FLAGS: &[&str] = &[
        "-c", "-e", "-F", "-l", "-m", "-p", "-R", "-s", "-S", "-t", "-x", "-X", "-y", "-Y",
    ];
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let Some(target) = target else {
        return CommandResult::err("can't establish current session\n");
    };
    let resolved = match st.resolve(&target) {
        Some(target) => target,
        None => return CommandResult::err(format!("{}\n", st.pane_target_error(&target))),
    };
    let window = st.window(resolved.session, resolved.window);
    let (window_width, window_height) = (window.cols, window.rows);
    let geometry = |flag, total| -> Result<Option<u16>, CommandResult> {
        flag_value(args, flag)
            .map(|value| parse_pane_size(value, total))
            .transpose()
            .map_err(|_| CommandResult::err("size or position invalid floating geometry\n"))
    };
    let width = match geometry("-x", window_width) {
        Ok(value) => value,
        Err(error) => return error,
    };
    let height = match geometry("-y", window_height) {
        Ok(value) => value,
        Err(error) => return error,
    };
    let position = |flag, total| -> Result<Option<i32>, CommandResult> {
        flag_value(args, flag)
            .map(|value| parse_pane_position(value, total))
            .transpose()
            .map_err(|_| CommandResult::err("size or position invalid floating geometry\n"))
    };
    let left = match position("-X", window_width) {
        Ok(value) => value,
        Err(error) => return error,
    };
    let top = match position("-Y", window_height) {
        Ok(value) => value,
        Err(error) => return error,
    };
    let explicit_cwd = command_option_value(args, "-c", VALUE_FLAGS).map(PathBuf::from);
    let cwd = explicit_cwd.as_deref().or(context.cwd.as_deref());
    let command = trailing_command(args, VALUE_FLAGS);
    let shell = context
        .env("SHELL")
        .map(str::to_string)
        .or_else(|| std::env::var("SHELL").ok())
        .unwrap_or_else(|| "/bin/sh".into());
    let argv = match command.as_slice() {
        [] => vec![shell],
        [command] => vec![shell, "-c".into(), (*command).into()],
        command => command
            .iter()
            .map(|argument| (*argument).to_string())
            .collect(),
    };
    let spawn_environment = flag_values(args, "-e");
    let argv = pane_argv(argv, context, &spawn_environment, st, SpawnSession::Existing(&target));
    let select = !has_flag(args, "-d");
    let created = if has_flag(args, "-L") {
        let direction = if has_flag(args, "-h") {
            SplitDirection::LeftRight
        } else {
            SplitDirection::TopBottom
        };
        st.split_window_direction_with_spawn(
            &target,
            select,
            has_flag(args, "-b"),
            direction,
            &argv,
            cwd,
            None,
        )
    } else {
        st.new_floating_pane_with_spawn(&target, select, width, height, left, top, &argv, cwd)
    };
    let pane = match created {
        Ok(pane) => pane,
        Err(error) => return CommandResult::err(format!("{error}\n")),
    };
    let pane_target = Target {
        session: resolved.session,
        window: resolved.window,
        pane,
    };
    if let Some(style) = flag_value(args, "-s") {
        st.set_pane_option(pane_target, "window-style", style);
        st.set_pane_option(pane_target, "window-active-style", style);
    }
    if let Some(style) = flag_value(args, "-S") {
        st.set_pane_option(pane_target, "pane-active-border-style", style);
    }
    if let Some(style) = flag_value(args, "-R") {
        st.set_pane_option(pane_target, "pane-border-style", style);
    }
    if has_flag(args, "-k") || has_flag(args, "-m") {
        st.set_pane_option(pane_target, "remain-on-exit", "on");
    }
    if let Some(message) = flag_value(args, "-m") {
        st.set_pane_option(pane_target, "remain-on-exit-format", message);
    }
    if has_flag(args, "-P") {
        let session = &st.sessions()[resolved.session];
        let template = flag_value(args, "-F").unwrap_or(NEW_WINDOW_TEMPLATE);
        let line = expand_command_format(
            st,
            template,
            &vars_full(
                st,
                session,
                resolved.window,
                pane,
                &PaneAgents::new(),
                st.marked_pane(),
            ),
            None,
        );
        CommandResult::ok(format!("{line}\n"))
    } else {
        CommandResult::ok("")
    }
}

fn parse_pane_size(value: &str, total: u16) -> Result<u16, ()> {
    let size = if let Some(percent) = value.strip_suffix('%') {
        u32::from(total) * percent.parse::<u32>().map_err(|_| ())? / 100
    } else {
        value.parse::<u32>().map_err(|_| ())?
    };
    u16::try_from(size)
        .ok()
        .filter(|size| *size < total)
        .ok_or(())
}

fn parse_pane_position(value: &str, total: u16) -> Result<i32, ()> {
    if let Some(percent) = value.strip_suffix('%') {
        let percent = percent.parse::<i32>().map_err(|_| ())?;
        Ok(i32::from(total) * percent / 100)
    } else {
        value.parse::<i32>().map_err(|_| ())
    }
}

/// `select-pane [-t target]`. Makes the target pane active.
fn select_pane(args: &[String], st: &mut ServerState, context: &ClientContext) -> CommandResult {
    // `-l` takes the whole last-pane path first, exactly as tmux routes
    // `select-pane -l` and `last-pane` through one branch — including the
    // `-d`/`-e` input toggles, which then act on the last pane, not the
    // target pane.
    if has_flag(args, "-l") {
        return last_pane_cmd(args, st);
    }
    // `-M` clears the server's marked pane and ignores everything else.
    if has_flag(args, "-M") {
        st.clear_mark();
        return CommandResult::ok("");
    }
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let target = match target {
        Some(t) => t,
        None => return CommandResult::err("can't establish current session\n"),
    };
    if let Some(title) = flag_value(args, "-T") {
        return match st.set_pane_title(&target, title) {
            Ok(()) => CommandResult::ok(""),
            Err(error) => CommandResult::err(format!("{error}\n")),
        };
    }
    if has_flag(args, "-d") || has_flag(args, "-e") {
        return match st.set_pane_input_off(&target, has_flag(args, "-d")) {
            Ok(()) => CommandResult::ok(""),
            Err(error) => CommandResult::err(format!("{error}\n")),
        };
    }
    let directional = [
        ("-U", SplitDirection::TopBottom, false),
        ("-D", SplitDirection::TopBottom, true),
        ("-L", SplitDirection::LeftRight, false),
        ("-R", SplitDirection::LeftRight, true),
    ]
    .into_iter()
    .find(|(flag, _, _)| has_flag(args, flag));
    if let Some((_, direction, forward)) = directional {
        if context.control_active_panes().is_some() {
            return match st.pane_in_direction(&target, direction, forward) {
                Ok((window_id, pane_id)) => {
                    context.set_active_pane(window_id, pane_id);
                    CommandResult::ok("")
                }
                Err(error) => CommandResult::err(format!("{error}\n")),
            };
        }
        return match st.select_pane_direction(&target, direction, forward) {
            Ok(()) => CommandResult::ok(""),
            Err(error) => CommandResult::err(format!("{error}\n")),
        };
    }
    // `-m` toggles the mark on the target without changing the active pane; the
    // plain form selects (activates) it.
    let result = if has_flag(args, "-m") {
        st.mark_pane(&target)
    } else if context.control_active_panes().is_some() {
        match st.resolve(&target) {
            Some(target) => {
                let (window_id, pane_id) = st.target_pane_ids(target);
                context.set_active_pane(window_id, pane_id);
                Ok(())
            }
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find pane: {target}"),
            )),
        }
    } else {
        st.select_pane(&target)
    };
    match result {
        Ok(()) => CommandResult::ok(""),
        Err(e) => CommandResult::err(format!("{e}\n")),
    }
}

/// `pipe-pane [-IOo] [-t target] [command]`. By default pane output is written
/// to the command. `-I` additionally connects command output to pane input;
/// specifying only `-I` suppresses the default output direction.
fn pipe_pane(args: &[String], st: &mut ServerState) -> CommandResult {
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let target = match target {
        Some(t) => t,
        None => return CommandResult::err("can't establish current session\n"),
    };
    // `pipe-pane` has a pane target, so tmux interprets an unqualified value
    // as a pane before falling back to a window or session. Keep the existing
    // fallback for targets that resolve, but report the target-type-specific
    // diagnostic when a bare value cannot resolve at all.
    if !target.contains([':', '.'])
        && !target.starts_with(['$', '@', '%'])
        && st.resolve(&target).is_none()
    {
        return CommandResult::err(format!("can't find pane: {target}\n"));
    }
    let command = trailing_command(args, &["-t"]).join(" ");
    let only_toggle = has_flag(args, "-o");
    let input = has_flag(args, "-I");
    let output = has_flag(args, "-O") || !input;
    match st.pipe_pane(
        &target,
        (!command.is_empty()).then_some(command.as_str()),
        only_toggle,
        input,
        output,
    ) {
        Ok(()) => CommandResult::ok(""),
        Err(e) => CommandResult::err(format!("{e}\n")),
    }
}

/// `kill-pane [-t target]`. Removes the target pane (destroying its window, and
/// the session, if it was the last).
fn kill_pane(args: &[String], st: &mut ServerState) -> CommandResult {
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let target = match target {
        Some(t) => t,
        None => return CommandResult::err("can't establish current session\n"),
    };
    let result = if has_flag(args, "-a") {
        st.kill_other_panes(&target)
    } else {
        st.kill_pane(&target)
    };
    match result {
        Ok(()) => CommandResult::ok(""),
        Err(e) => CommandResult::err(format!("{e}\n")),
    }
}

/// `respawn-pane [-k] [-t target]`. Real tmux refuses to respawn a pane whose
/// child is still running unless `-k` (kill first) is given, failing with
/// `respawn pane failed: pane <t> still active` (exit 1). Native models that
/// guard: the target must resolve (else tmux's `can't find pane`), and a pane
/// that has not exited is "still active". With `-k`, or against an exited pane,
/// the command succeeds silently — the actual re-exec of the pane's command is
/// interactive byte work outside this control-plane layer.
fn respawn_pane(args: &[String], st: &mut ServerState, context: &ClientContext) -> CommandResult {
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let target = match target {
        Some(t) => t,
        None => return CommandResult::err("can't establish current session\n"),
    };
    let resolved = match st.resolve(&target) {
        Some(t) => t,
        None => return CommandResult::err(format!("{}\n", st.pane_target_error(&target))),
    };
    let sess = &st.sessions()[resolved.session];
    let link = &sess.windows[resolved.window];
    let win = st.window_for_link(link);
    let node = &win.panes[resolved.pane];
    // `-k` bypasses the guard; a pane that has already exited may be respawned.
    if !has_flag(args, "-k") && !node.pane.has_exited() {
        return CommandResult::err(format!(
            "respawn pane failed: pane {}:{}.{} still active\n",
            sess.name, link.index, resolved.pane,
        ));
    }
    let command = trailing_command(args, &["-c", "-e", "-t"]);
    // A respawn spells its command the way a spawn does: one argument is a
    // shell command line, several are an argv (tmux's `spawn_pane`).
    let argv =
        (!command.is_empty()).then(|| pane_command_argv(command.as_slice(), st, Some(&target)));
    let mut cwd = command_option_value(args, "-c", &["-c", "-e", "-t"]).map(PathBuf::from);
    // `-e` reaches the replacement the way a spawn's environment does. With no
    // command the saved spawn spec is materialized so the wrap has an argv to
    // carry, keeping its stored working directory.
    let environment = flag_values(args, "-e");
    let argv = if environment.is_empty() {
        argv
    } else {
        let base = argv.or_else(|| {
            let saved = node.pane.spawn_spec()?;
            if cwd.is_none() {
                cwd = saved.cwd;
            }
            Some(saved.argv)
        });
        base.map(|argv| pane_argv(argv, context, &environment, st, SpawnSession::Existing(&target)))
    };
    match st.respawn_pane_process(&target, argv, cwd) {
        Ok(()) => CommandResult::ok(""),
        Err(error) => CommandResult::err(format!("{error}\n")),
    }
}

/// `respawn-window [-k] [-t target]`. The window-scoped twin of `respawn_pane`:
/// real tmux refuses to respawn a window that has any still-running pane unless
/// `-k` (kill first) is given, failing with `respawn window failed: window <t>
/// still active` (exit 1). The target must resolve (else tmux's `can't find
/// window`), and a window is "still active" while any of its panes has not
/// exited. With `-k`, or an all-exited window, the command succeeds silently —
/// the actual re-exec of the panes' commands is interactive byte work outside
/// this control-plane layer.
fn respawn_window(args: &[String], st: &mut ServerState, context: &ClientContext) -> CommandResult {
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let target = match target {
        Some(t) => t,
        None => return CommandResult::err("can't establish current session\n"),
    };
    let resolved = match st.resolve_window_target(&target) {
        Ok(resolved) => resolved,
        Err(error) => return CommandResult::err(format!("{error}\n")),
    };
    let sess = &st.sessions()[resolved.session];
    let link = &sess.windows[resolved.window];
    let win = st.window_for_link(link);
    // `-k` bypasses the guard; a window whose panes have all exited may respawn.
    let live = win.panes.iter().any(|node| !node.pane.has_exited());
    if !has_flag(args, "-k") && live {
        return CommandResult::err(format!(
            "respawn window failed: window {}:{} still active\n",
            sess.name, link.index,
        ));
    }
    let command = trailing_command(args, &["-c", "-e", "-t"]);
    let argv =
        (!command.is_empty()).then(|| pane_command_argv(command.as_slice(), st, Some(&target)));
    // `-e` reaches the replacement the way a spawn's environment does.
    let environment = flag_values(args, "-e");
    let argv = if environment.is_empty() {
        argv
    } else {
        argv.map(|argv| pane_argv(argv, context, &environment, st, SpawnSession::Existing(&target)))
    };
    let cwd = command_option_value(args, "-c", &["-c", "-e", "-t"]).map(PathBuf::from);
    match st.respawn_window_process(&target, argv, cwd) {
        Ok(()) => CommandResult::ok(""),
        Err(error) => CommandResult::err(format!("{error}\n")),
    }
}

/// `swap-window -s src -t dst`. Exchanges two windows' contents (keeping their
/// indices). Missing `-s`/`-t` default to the current session's active window.
fn swap_window(args: &[String], st: &mut ServerState) -> CommandResult {
    let src = flag_value(args, "-s")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let dst = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    // tmux inverts the usual `-d` sense for swap-window: the plain form leaves
    // the current window where it is, and `-d` selects the swapped target.
    let select = has_flag(args, "-d");
    match (src, dst) {
        (Some(s), Some(d)) => match st.swap_window(&s, &d, select) {
            Ok(()) => CommandResult::ok(""),
            Err(error) => command_target_error_candidates(error, &[(&s, "window"), (&d, "window")]),
        },
        _ => CommandResult::err("can't establish current session\n"),
    }
}

/// `move-window -s src -t dst`. Renumbers a window to the destination index.
/// With `-r`, renumbers *all* windows of the target session to close gaps
/// (ignoring `-s`), matching tmux.
fn move_window(args: &[String], st: &mut ServerState) -> CommandResult {
    if has_flag(args, "-r") {
        let session = flag_value(args, "-t")
            .map(|t| t.split(':').next().unwrap_or(t).to_string())
            .or_else(|| current_session(st));
        let session = match session {
            Some(s) => s,
            None => return CommandResult::err("can't establish current session\n"),
        };
        return match st.renumber_windows(&session) {
            Ok(()) => CommandResult::ok(""),
            Err(e) => CommandResult::err(format!("{e}\n")),
        };
    }
    let src = flag_value(args, "-s")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let dst = flag_value(args, "-t").map(str::to_string);
    // tmux selects the moved window by default; `-d` leaves the current window
    // where it is.
    let select = !has_flag(args, "-d");
    // `-a` (after) / `-b` (before) relocate the window *relative* to the `-t`
    // anchor index rather than onto it. `-b` takes precedence over `-a`.
    let relative = has_flag(args, "-a") || has_flag(args, "-b");
    match (src, dst) {
        (Some(s), Some(d)) => match if relative {
            st.move_window_relative(&s, &d, !has_flag(args, "-b"), select)
        } else if has_flag(args, "-k") {
            st.move_window_replacing(&s, &d, select)
        } else {
            st.move_window(&s, &d, select)
        } {
            Ok(()) => CommandResult::ok(""),
            Err(error) => command_target_error_candidates(error, &[(&s, "window"), (&d, "window")]),
        },
        (None, _) => CommandResult::err("can't establish current session\n"),
        (_, None) => CommandResult::err("move-window: missing destination\n"),
    }
}

/// `set-environment [-Fghru] VAR [VALUE]`.
fn set_environment(args: &[String], st: &mut ServerState) -> CommandResult {
    let pos = positionals(args, &["-t"]);
    let name = match pos.first() {
        Some(n) => *n,
        None => return CommandResult::err("set-environment: missing variable\n"),
    };
    if name.is_empty() {
        return CommandResult::err("empty variable name\n");
    }
    if name.contains('=') {
        return CommandResult::err("variable name contains =\n");
    }
    let global = has_bool_flag(args, 'g');
    let target = if global {
        None
    } else {
        flag_value(args, "-t")
            .map(str::to_string)
            .or_else(|| current_target(st))
    };
    if !global && target.is_none() {
        return CommandResult::err("no current session\n");
    }
    if !global {
        let target = target.as_deref().unwrap_or_default();
        if st.resolve_session(target).is_none() {
            return match flag_value(args, "-t") {
                Some(target) => CommandResult::err(format!("no such session: {target}\n")),
                None => CommandResult::err("no current session\n"),
            };
        }
    }
    if has_flag(args, "-u") || has_flag(args, "-r") {
        if pos.get(1).is_some() {
            let flag = if has_flag(args, "-u") { "-u" } else { "-r" };
            return CommandResult::err(format!("can't specify a value with {flag}\n"));
        }
        if global {
            if has_flag(args, "-r") {
                st.remove_env(name);
            } else {
                st.unset_env(name);
            }
        } else if let Err(error) =
            st.unset_session_env(target.as_deref().unwrap(), name, has_flag(args, "-r"))
        {
            return CommandResult::err(format!("{error}\n"));
        }
    } else {
        let Some(raw_value) = pos.get(1).copied() else {
            return CommandResult::err("no value specified\n");
        };
        let value = if has_bool_flag(args, 'F') {
            let format_target = target.clone().or_else(|| current_session(st));
            match format_target.as_deref().and_then(|name| st.find(name)) {
                Some(sess) => format::expand(
                    raw_value,
                    &vars_for(st, sess, sess.active, &PaneAgents::new(), st.marked_pane()),
                ),
                None => raw_value.to_string(),
            }
        } else {
            raw_value.to_string()
        };
        if global {
            if has_bool_flag(args, 'h') {
                st.set_hidden_env(name, &value);
            } else {
                st.set_env(name, &value);
            }
        } else if let Err(error) = st.set_session_env(
            target.as_deref().unwrap(),
            name,
            &value,
            has_bool_flag(args, 'h'),
        ) {
            return CommandResult::err(format!("{error}\n"));
        }
    }
    CommandResult::ok("")
}

#[derive(Clone, Copy)]
enum OptionTarget {
    Server,
    GlobalSession,
    Session(usize),
    GlobalWindow,
    Window(Target),
    Pane(Target),
}

impl OptionTarget {
    fn local(self, st: &ServerState) -> &OptionSet {
        match self {
            Self::Server => st.global_options().server(),
            Self::GlobalSession => st.global_options().session(),
            Self::Session(session) => st.sessions()[session].option_overrides(),
            Self::GlobalWindow => st.global_options().window(),
            Self::Window(target) => st.window(target.session, target.window).option_overrides(),
            Self::Pane(target) => {
                st.window(target.session, target.window).panes[target.pane].option_overrides()
            }
        }
    }

    fn view<'a>(self, st: &'a ServerState) -> OptionsView<'a> {
        match self {
            Self::Server => st.server_options(),
            Self::GlobalSession => OptionsView::one(st.global_options().session()),
            Self::Session(session) => st.sessions()[session].options(st.global_options()),
            Self::GlobalWindow => OptionsView::one(st.global_options().window()),
            Self::Window(target) => st
                .window(target.session, target.window)
                .options(st.global_options()),
            Self::Pane(target) => {
                let window = st.window(target.session, target.window);
                window.panes[target.pane].options(window, st.global_options())
            }
        }
    }

    fn local_mut(self, st: &mut ServerState) -> &mut OptionSet {
        match self {
            Self::Server => st.global_options_mut().for_scope_mut(OptionScope::Server),
            Self::GlobalSession => st.global_options_mut().for_scope_mut(OptionScope::Session),
            Self::Session(session) => st.session_mut(session).option_overrides_mut(),
            Self::GlobalWindow => st.global_options_mut().for_scope_mut(OptionScope::Window),
            Self::Window(target) => st
                .window_mut(target.session, target.window)
                .option_overrides_mut(),
            Self::Pane(target) => st.window_mut(target.session, target.window).panes[target.pane]
                .option_overrides_mut(),
        }
    }

    fn is_global(self) -> bool {
        matches!(
            self,
            Self::Server | Self::GlobalSession | Self::GlobalWindow
        )
    }

    fn scope(self) -> OptionScope {
        match self {
            Self::Server => OptionScope::Server,
            Self::GlobalSession | Self::Session(_) => OptionScope::Session,
            Self::GlobalWindow | Self::Window(_) => OptionScope::Window,
            Self::Pane(_) => OptionScope::WindowPane,
        }
    }
}

#[derive(Clone, Copy)]
enum OptionTargetKind {
    Session,
    Window,
    Pane,
}

impl OptionTargetKind {
    fn name(self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::Window => "window",
            Self::Pane => "pane",
        }
    }
}

fn option_command_target(
    args: &[String],
    st: &ServerState,
    kind: OptionTargetKind,
) -> Result<Target, CommandResult> {
    let explicit = flag_value(args, "-t");
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let Some(target) = target else {
        return Err(CommandResult::err(format!("no current {}\n", kind.name())));
    };
    st.resolve(&target).ok_or_else(|| {
        if explicit.is_some() {
            CommandResult::err(format!("no such {}: {target}\n", kind.name()))
        } else {
            CommandResult::err(format!("no current {}\n", kind.name()))
        }
    })
}

fn option_target_from_flags(
    args: &[String],
    st: &ServerState,
    window_command: bool,
) -> Result<OptionTarget, CommandResult> {
    if has_bool_flag(args, 's') {
        return Ok(OptionTarget::Server);
    }
    if has_bool_flag(args, 'p') {
        Ok(OptionTarget::Pane(option_command_target(
            args,
            st,
            OptionTargetKind::Pane,
        )?))
    } else if window_command || has_bool_flag(args, 'w') {
        if has_bool_flag(args, 'g') {
            Ok(OptionTarget::GlobalWindow)
        } else {
            Ok(OptionTarget::Window(option_command_target(
                args,
                st,
                OptionTargetKind::Window,
            )?))
        }
    } else if has_bool_flag(args, 'g') {
        Ok(OptionTarget::GlobalSession)
    } else {
        Ok(OptionTarget::Session(
            option_command_target(args, st, OptionTargetKind::Session)?.session,
        ))
    }
}

fn option_target_from_name(
    args: &[String],
    st: &ServerState,
    scope: OptionScope,
) -> Result<OptionTarget, CommandResult> {
    match scope {
        OptionScope::Server => Ok(OptionTarget::Server),
        OptionScope::Session if has_bool_flag(args, 'g') => Ok(OptionTarget::GlobalSession),
        OptionScope::Session => Ok(OptionTarget::Session(
            option_command_target(args, st, OptionTargetKind::Session)?.session,
        )),
        OptionScope::WindowPane if has_bool_flag(args, 'p') => Ok(OptionTarget::Pane(
            option_command_target(args, st, OptionTargetKind::Pane)?,
        )),
        OptionScope::Window | OptionScope::WindowPane if has_bool_flag(args, 'g') => {
            Ok(OptionTarget::GlobalWindow)
        }
        OptionScope::Window | OptionScope::WindowPane => Ok(OptionTarget::Window(
            option_command_target(args, st, OptionTargetKind::Window)?,
        )),
    }
}

/// Re-print a command list with each command's canonical name, the way tmux's
/// `cmd_list_print` does for a stored command option. A body that does not
/// parse is kept verbatim so a not-yet-valid hook still round-trips.
fn canonical_command_list(value: &str, st: &ServerState) -> String {
    let tokens = tokenize_line(value);
    let groups = tokenized_command_groups(&tokens);
    if groups.is_empty() {
        return value.to_string();
    }
    let aliases = st.command_aliases();
    let mut printed = Vec::with_capacity(groups.len());
    for group in &groups {
        let Some(first) = group.first() else {
            return value.to_string();
        };
        let canonical = match registry::resolve(first) {
            Resolution::Name(name) => name.to_string(),
            _ => match aliases.iter().find(|(alias, _)| alias == first) {
                // An alias expands to a whole command line; leave it alone
                // rather than half-rewriting it.
                Some(_) => return value.to_string(),
                None => return value.to_string(),
            },
        };
        let mut rewritten = group.clone();
        rewritten[0] = canonical;
        printed.push(display_command(&rewritten));
    }
    printed.join(" ; ")
}

fn resolve_option_argument(argument: &str) -> Option<(&str, Option<u32>)> {
    let (name, index) = options::parse_option_name(argument)?;
    if name.starts_with('@') {
        Some((name, index))
    } else {
        options::resolve_option_name(argument).map(|(resolved, index)| (resolved as &str, index))
    }
}

/// A numeric option value with tmux's strtonum syntax: an optional sign and
/// decimal digits. Values beyond i64 saturate so range checks still report
/// too large / too small rather than invalid.
fn parse_option_number(value: &str) -> Result<i64, ()> {
    let digits = value.strip_prefix('-').unwrap_or(value);
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(());
    }
    Ok(value.parse::<i64>().unwrap_or(if value.starts_with('-') {
        i64::MIN
    } else {
        i64::MAX
    }))
}

/// Like [`positionals`], but flag parsing ends at the first operand as in
/// tmux's getopt, so a later dash-prefixed token — most importantly a negative
/// number (`set-option repeat-time -1`) — is a value, not a flag.
fn operands<'a>(args: &'a [String], value_flags: &[&str]) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut i = 1; // skip the command name
    while i < args.len() {
        let a = args[i].as_str();
        if a == "--" {
            out.extend(args[i + 1..].iter().map(String::as_str));
            break;
        }
        if a.starts_with('-') && a != "-" {
            if value_flags.contains(&a) {
                i += 1; // also skip this flag's value
            }
            i += 1;
            continue;
        }
        out.extend(args[i..].iter().map(String::as_str));
        break;
    }
    out
}

/// Set an option in the local table selected by its catalog scope and target.
fn set_option(args: &[String], st: &mut ServerState, window_command: bool) -> CommandResult {
    let pos = operands(args, &["-t"]);
    let argument = match pos.first() {
        Some(n) => *n,
        None => return CommandResult::err("set-option: missing option\n"),
    };
    let Some((name, index)) = resolve_option_argument(argument) else {
        // `-q` suppresses the diagnostic, turning the failure into a silent
        // success (exit 0, no output) — matching real tmux.
        return if has_bool_flag(args, 'q') {
            CommandResult::ok("")
        } else {
            CommandResult::err(format!("invalid option: {argument}\n"))
        };
    };
    let user_option = name.starts_with('@');
    let kind = (!user_option).then(|| options::option_kind(name)).flatten();
    if !user_option && kind.is_none() {
        return if has_bool_flag(args, 'q') {
            CommandResult::ok("")
        } else {
            CommandResult::err(format!("invalid option: {argument}\n"))
        };
    }
    // Unlike invalid names, tmux never suppresses this diagnostic with `-q`.
    if index.is_some() && (user_option || kind != Some(options::OptionKind::Array)) {
        return CommandResult::err(format!("not an array: {argument}\n"));
    }
    let target = if user_option {
        match option_target_from_flags(args, st, window_command) {
            Ok(target) => target,
            Err(error) => return error,
        }
    } else {
        match option_target_from_name(
            args,
            st,
            options::option_scope(name).expect("known option has scope"),
        ) {
            Ok(target) => target,
            Err(error) => return error,
        }
    };
    let storage_name = match index {
        Some(index) => format!("{name}[{index}]"),
        None => name.to_string(),
    };
    let already_set = if index.is_none() && kind == Some(options::OptionKind::Array) {
        target.local(st).contains_array(name)
    } else {
        target.local(st).contains(&storage_name)
    };
    if has_bool_flag(args, 'o') && already_set {
        return if has_bool_flag(args, 'q') {
            CommandResult::ok("")
        } else {
            CommandResult::err(format!("already set: {argument}\n"))
        };
    }
    let unset = has_bool_flag(args, 'u') || has_bool_flag(args, 'U');
    let raw_value = pos.get(1).copied();
    let mut value = match raw_value {
        Some(raw) if has_bool_flag(args, 'F') => {
            match current_session(st).and_then(|name| st.find(&name)) {
                Some(sess) => {
                    let window = st.command_window_index(sess);
                    expand_command_format(
                        st,
                        raw,
                        &vars_for(st, sess, window, &PaneAgents::new(), st.marked_pane()),
                        None,
                    )
                }
                None => raw.to_string(),
            }
        }
        Some(raw) => raw.to_string(),
        None => String::new(),
    };
    // tmux parses a hook (or any command-typed option) into a command list at
    // assignment time, so what `show-hooks` prints back is the canonical
    // command name rather than whatever alias the body was written with.
    if !unset && raw_value.is_some() && options::is_hook(name) {
        value = canonical_command_list(&value, st);
    }
    if !unset {
        if let Some((min, max)) = options::option_number_range(name) {
            match parse_option_number(&value) {
                Err(()) => return CommandResult::err(format!("value is invalid: {value}\n")),
                Ok(number) if number < min => {
                    return CommandResult::err(format!("value is too small: {value}\n"))
                }
                Ok(number) if number > max => {
                    return CommandResult::err(format!("value is too large: {value}\n"))
                }
                Ok(_) => {}
            }
        } else if let Some(choices) = options::option_choices(name) {
            if raw_value.is_none() {
                // No value toggles between the first two choices; from any
                // later choice the current value is kept (tmux
                // options_from_string_choice).
                let current = target
                    .view(st)
                    .get(name)
                    .map(str::to_string)
                    .or_else(|| options::option_initial_default(name));
                let index = current
                    .as_deref()
                    .and_then(|current| choices.iter().position(|choice| *choice == current));
                value = match index {
                    Some(0) => choices[1].to_string(),
                    Some(index) if index >= 2 => choices[index].to_string(),
                    _ => choices[0].to_string(),
                };
            } else if !choices.contains(&value.as_str()) {
                return CommandResult::err(format!("unknown value: {value}\n"));
            }
        } else if options::option_is_flag(name) {
            let extension = options::flag_extension_value(name);
            value = match value.as_str() {
                "on" | "yes" | "1" => "on".to_string(),
                "off" | "no" | "0" => "off".to_string(),
                "" => match target.view(st).get(name) {
                    // No value toggles the flag, except from an extension
                    // value, which is kept the way tmux keeps a choice past
                    // the first two (options_from_string_choice).
                    Some(current) if Some(current) == extension => current.to_string(),
                    Some("on") => "off".to_string(),
                    _ => "on".to_string(),
                },
                other if Some(other) == extension => other.to_string(),
                _ => return CommandResult::err(format!("bad value: {value}\n")),
            };
        }
    }
    if has_bool_flag(args, 'U') {
        let window_target = match target {
            OptionTarget::Window(target) => Some(target),
            OptionTarget::GlobalWindow => {
                match option_command_target(args, st, OptionTargetKind::Window) {
                    Ok(target) => Some(target),
                    Err(error) => return error,
                }
            }
            _ => None,
        };
        if let Some(window_target) = window_target {
            for pane in &mut st
                .window_mut(window_target.session, window_target.window)
                .panes
            {
                if kind == Some(options::OptionKind::Array) && index.is_none() {
                    pane.option_overrides_mut().clear_array(name);
                } else {
                    pane.option_overrides_mut().remove(&storage_name);
                }
            }
        }
    }
    if kind == Some(options::OptionKind::Array) && index.is_none() {
        let global = target.is_global();
        let table = target.local_mut(st);
        if has_bool_flag(args, 'u') || has_bool_flag(args, 'U') {
            table.clear_array(name);
            if global {
                options::restore_array_default(table, name);
            }
        } else {
            if !has_bool_flag(args, 'a') {
                table.clear_array(name);
            }
            let separator = options::option_array_separator(name).unwrap_or(" ,");
            let values = if separator.is_empty() {
                (!value.is_empty())
                    .then_some(value.as_str())
                    .into_iter()
                    .collect::<Vec<_>>()
            } else {
                value
                    .split(|character| separator.contains(character))
                    .filter(|value| !value.is_empty())
                    .collect::<Vec<_>>()
            };
            for value in values {
                let index = table.next_array_index(name);
                table.set(format!("{name}[{index}]"), value);
            }
        }
        st.option_changed(name);
        return CommandResult::ok("");
    }
    if has_bool_flag(args, 'u') || has_bool_flag(args, 'U') {
        let table = target.local_mut(st);
        table.remove(&storage_name);
        if target.is_global() && index.is_none() && !user_option {
            if let Some(default) = options::option_initial_default(name) {
                table.set(name, default);
            }
        }
    } else if has_bool_flag(args, 'a') {
        let table = target.local_mut(st);
        if !table.contains(&storage_name) && !user_option && index.is_none() {
            if let Some(default) = options::option_initial_default(name) {
                table.set(&storage_name, default);
            }
        }
        table.append(&storage_name, &value);
    } else {
        target.local_mut(st).set(&storage_name, &value);
    }
    st.option_changed(name);
    if name == "history-file" {
        st.load_prompt_history();
    }
    st.enforce_unattached_options();
    CommandResult::ok("")
}

fn option_in_listing_scope(name: &str, scope: OptionScope) -> bool {
    if name.starts_with('@') {
        return true;
    }
    let base = options::parse_option_name(name)
        .map(|(base, _)| base)
        .unwrap_or(name);
    match (options::option_scope(base), scope) {
        (Some(OptionScope::WindowPane), OptionScope::Window | OptionScope::WindowPane) => true,
        (candidate, expected) => candidate == Some(expected),
    }
}

fn tmux_escape_argument(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    let quote = if value.chars().any(|ch| " #';${}%".contains(ch)) {
        Some('"')
    } else if value.chars().any(|ch| " \"".contains(ch)) {
        Some('\'')
    } else {
        None
    };
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{1b}' => escaped.push_str("\\e"),
            '\\' => escaped.push_str("\\\\"),
            '"' if quote == Some('"') => escaped.push_str("\\\""),
            character if character.is_control() => {
                escaped.push_str(&format!("\\{:03o}", character as u32));
            }
            character => escaped.push(character),
        }
    }
    match quote {
        Some(quote) => format!("{quote}{escaped}{quote}"),
        None if escaped.starts_with('~') => format!("\\{escaped}"),
        None => escaped,
    }
}

fn show_option_value(name: &str, value: &str) -> String {
    let base = options::parse_option_name(name)
        .map(|(base, _)| base)
        .unwrap_or(name);
    if options::is_hook(base) {
        value.to_string()
    } else {
        tmux_escape_argument(value)
    }
}

fn show_options(args: &[String], st: &ServerState, window_command: bool) -> CommandResult {
    let value_only = has_flag(args, "-v");
    // `-q` swallows the invalid-option diagnostic, turning any name error into a
    // silent success (exit 0, no output) — matching real tmux.
    let quiet = has_bool_flag(args, 'q');
    let pos = positionals(args, &["-t"]);
    let argument = match pos.first() {
        Some(n) => *n,
        None => {
            let target = match option_target_from_flags(args, st, window_command) {
                Ok(target) => target,
                Err(_error) if quiet => return CommandResult::ok(""),
                Err(error) => return error,
            };
            let inherited = has_bool_flag(args, 'A');
            let local = target.local(st);
            let entries: Vec<_> = if inherited {
                target.view(st).iter_effective().collect()
            } else {
                local.iter().collect()
            };
            let mut output = String::new();
            for (name, value) in entries {
                if !option_in_listing_scope(name, target.scope()) {
                    continue;
                }
                let parent = inherited && !local.contains(name);
                let empty_array = options::parse_option_name(name).is_some_and(|(base, index)| {
                    index.is_none()
                        && options::option_kind(base) == Some(options::OptionKind::Array)
                        && value.is_empty()
                });
                if value_only {
                    if empty_array {
                        continue;
                    }
                    output.push_str(value);
                } else if empty_array {
                    output.push_str(name);
                } else if parent {
                    output.push_str(&format!("{name}* {}", show_option_value(name, value)));
                } else {
                    output.push_str(&format!("{name} {}", show_option_value(name, value)));
                }
                output.push('\n');
            }
            return CommandResult::ok(output);
        }
    };

    let Some((name, index)) = resolve_option_argument(argument) else {
        return if quiet {
            CommandResult::ok("")
        } else {
            CommandResult::err(format!("invalid option: {argument}\n"))
        };
    };
    let display_name = match index {
        Some(index) => format!("{name}[{index}]"),
        None => name.to_string(),
    };

    let user_option = name.starts_with('@');
    let kind = (!user_option).then(|| options::option_kind(name)).flatten();
    if !user_option && kind.is_none() {
        return if quiet {
            CommandResult::ok("")
        } else {
            CommandResult::err(format!("invalid option: {argument}\n"))
        };
    };

    let target = if user_option {
        match option_target_from_flags(args, st, window_command) {
            Ok(target) => target,
            Err(_error) if quiet => return CommandResult::ok(""),
            Err(error) => return error,
        }
    } else {
        match option_target_from_name(
            args,
            st,
            options::option_scope(name).expect("known option has scope"),
        ) {
            Ok(target) => target,
            Err(_error) if quiet => return CommandResult::ok(""),
            Err(error) => return error,
        }
    };
    let storage_name = if index.is_some() && kind == Some(options::OptionKind::Array) {
        display_name.as_str()
    } else {
        name
    };
    let inherited = has_bool_flag(args, 'A');
    let local = target.local(st);
    if kind == Some(options::OptionKind::Array) && index.is_none() {
        let entries: Vec<_> = if inherited {
            target.view(st).iter_effective().collect()
        } else {
            local.iter().collect()
        };
        let mut entries = entries
            .into_iter()
            .filter_map(|(entry, value)| {
                let (base, index) = options::parse_option_name(entry)?;
                (base == name).then_some((index?, value))
            })
            .collect::<Vec<_>>();
        entries.sort_by_key(|(index, _)| *index);
        let parent = inherited && !local.contains_array(name);
        let mut output = String::new();
        for (index, value) in entries {
            if value_only {
                output.push_str(value);
            } else if parent {
                output.push_str(&format!(
                    "{name}[{index}]* {}",
                    show_option_value(name, value)
                ));
            } else {
                output.push_str(&format!(
                    "{name}[{index}] {}",
                    show_option_value(name, value)
                ));
            }
            output.push('\n');
        }
        if output.is_empty()
            && !value_only
            && (local.contains_array(name) || (inherited && target.view(st).contains_array(name)))
        {
            output.push_str(name);
            output.push('\n');
        }
        return CommandResult::ok(output);
    }
    let local_value = local.get(storage_name);
    let parent = inherited && local_value.is_none();
    let value = local_value
        .or_else(|| {
            inherited
                .then(|| target.view(st).get(storage_name))
                .flatten()
        })
        .map(str::to_string)
        .or_else(|| {
            (index.is_some()
                && kind == Some(options::OptionKind::Array)
                && (target.is_global()
                    || local.contains_array(name)
                    || (inherited && target.view(st).contains_array(name))))
            .then(String::new)
        });
    if user_option && value.is_none() {
        return if quiet {
            CommandResult::ok("")
        } else {
            CommandResult::err(format!("invalid option: {argument}\n"))
        };
    }
    match value {
        Some(v) if value_only => CommandResult::ok(format!("{v}\n")),
        Some(v) if parent => {
            CommandResult::ok(format!("{display_name}* {}\n", show_option_value(name, &v)))
        }
        Some(v) => CommandResult::ok(format!("{display_name} {}\n", show_option_value(name, &v))),
        None => CommandResult::ok(""),
    }
}

fn set_hook(args: &[String], st: &mut ServerState) -> CommandResult {
    let hook = positionals(args, &["-t"]).into_iter().next();
    if has_flag(args, "-R") {
        let Some(hook) = hook else {
            return CommandResult::err("set-hook: missing hook\n");
        };
        if !options::is_hook(hook) {
            return CommandResult::err(format!("invalid option: {hook}\n"));
        }
        // `-R` runs the hook's body, which the queue lifts into queue items
        // of its own before dispatch reaches here.
        return CommandResult::err("not able to wait\n");
    }
    set_option(args, st, false)
}

fn show_hooks(args: &[String], st: &ServerState) -> CommandResult {
    let requested = positionals(args, &["-t"]).into_iter().next();
    if let Some(hook) = requested {
        if !options::is_hook(
            options::parse_option_name(hook)
                .map(|(base, _)| base)
                .unwrap_or(hook),
        ) {
            return CommandResult::err(format!("invalid option: {hook}\n"));
        }
        let result = show_options(args, st, false);
        if result.exit == 0 && result.stdout.is_empty() && !hook.contains('[') {
            return CommandResult::ok(format!("{hook}\n"));
        }
        return result;
    }

    let hooks = if has_flag(args, "-p") {
        if has_flag(args, "-g") {
            Vec::new()
        } else {
            options::PANE_HOOKS.to_vec()
        }
    } else if has_flag(args, "-w") {
        options::PANE_HOOKS
            .iter()
            .chain(options::WINDOW_HOOKS)
            .copied()
            .collect()
    } else {
        options::SESSION_HOOKS.to_vec()
    };
    let mut output = String::new();
    for hook in hooks {
        let mut hook_args = args.to_vec();
        hook_args.push((*hook).to_string());
        let shown = show_options(&hook_args, st, false);
        if shown.exit != 0 {
            return shown;
        } else if !shown.stdout.is_empty() {
            output.push_str(&shown.stdout);
        } else if has_flag(args, "-g") {
            output.push_str(hook);
            output.push('\n');
        }
    }
    CommandResult::ok(output)
}

fn show_messages(args: &[String], st: &ServerState) -> CommandResult {
    let show_terminals = has_flag(args, "-T");
    let show_jobs = has_flag(args, "-J");
    if show_terminals || show_jobs {
        let mut output = String::new();
        if show_terminals {
            let target = flag_value(args, "-t");
            let mut terminal_number = 0;
            for (name, term, terminal) in st.client_terminals() {
                if target.is_some_and(|target| {
                    let target = target.strip_suffix(':').unwrap_or(target);
                    name != target && name.strip_prefix("/dev/").is_none_or(|name| name != target)
                }) {
                    continue;
                }
                output.push_str(&format!(
                    "Terminal {terminal_number}: {term} for {name}, flags=0x{:x}:\n",
                    terminal.flags()
                ));
                for description in terminal.descriptions() {
                    output.push_str(&description);
                    output.push('\n');
                }
                terminal_number += 1;
            }
        }
        if show_jobs && !output.is_empty() {
            output.push('\n');
        }
        if show_jobs {
            // tmux keeps one job list for the whole server, so a `#()` still
            // running for some format is listed beside a `run-shell -b`.
            let background = st
                .background_job_registry()
                .jobs()
                .into_iter()
                .map(|job| (job.command, job.fd, job.pid as i32));
            let format_jobs = st
                .running_format_jobs()
                .into_iter()
                .map(|job| (job.command, job.fd, job.pid));
            for (id, (command, fd, pid)) in background.chain(format_jobs).enumerate() {
                output.push_str(&format!(
                    "Job {id}: {command} [fd={fd}, pid={pid}, status=0]\n"
                ));
            }
        }
        return CommandResult::ok(output);
    }
    let mut output = String::new();
    for message in st.messages().iter().rev() {
        output.push_str(&format!(
            "{}: {}\n",
            message_time(message.time),
            message.text
        ));
    }
    CommandResult::ok(output)
}

fn message_time(epoch: i64) -> String {
    let mut vars = Vars::new();
    vars.set("message_time", epoch.to_string());
    format::expand("#{t/p:message_time}", &vars)
}

fn lock_server(st: &ServerState) -> CommandResult {
    st.lock_all_clients();
    CommandResult::ok("")
}

fn lock_session(args: &[String], st: &ServerState) -> CommandResult {
    let target = flag_value(args, "-t")
        .map(|target| target.split(':').next().unwrap_or(target).to_string())
        .or_else(|| current_session(st));
    let Some(target) = target else {
        return CommandResult::err("can't establish current session\n");
    };
    match st.lock_session_clients(&target) {
        Ok(()) => CommandResult::ok(""),
        Err(error) => CommandResult::err(format!("{error}\n")),
    }
}

fn lock_client(args: &[String], st: &ServerState, context: &ClientContext) -> CommandResult {
    let target = flag_value(args, "-t");
    match st.lock_client(target, context.tty_name.as_deref()) {
        ClientActionResult::Queued => CommandResult::ok(""),
        ClientActionResult::NoCurrentClient => CommandResult::err("no current client\n"),
        ClientActionResult::TargetNotFound => CommandResult::err(format!(
            "can't find client: {}\n",
            target.unwrap_or_default()
        )),
    }
}

/// `show-environment [-ghs] [VAR]`. `-h` selects hidden entries and `-s`
/// renders shell commands instead of the environment-file representation.
fn show_environment(args: &[String], st: &ServerState) -> CommandResult {
    let pos = positionals(args, &["-t"]);
    let hidden_only = has_bool_flag(args, 'h');
    let shell = has_bool_flag(args, 's');
    if !has_bool_flag(args, 'g') {
        let target = flag_value(args, "-t")
            .map(str::to_string)
            .or_else(|| current_target(st));
        let Some(target) = target else {
            return CommandResult::err("no current session\n");
        };
        let (environment, removed, hidden) = match st.session_env(&target) {
            Ok(environment) => environment,
            Err(_) => {
                return match flag_value(args, "-t") {
                    Some(target) => CommandResult::err(format!("no such session: {target}\n")),
                    None => CommandResult::err("no current session\n"),
                }
            }
        };
        return match pos.first() {
            Some(name) if hidden.contains(*name) != hidden_only => CommandResult::ok(""),
            Some(name) if removed.contains(*name) => show_environment_value(name, None, shell),
            Some(name) => match environment.get(*name) {
                Some(value) => show_environment_value(name, Some(value), shell),
                None => CommandResult::err(format!("unknown variable: {name}\n")),
            },
            None => {
                let mut entries = environment
                    .iter()
                    .filter(|(name, _)| hidden.contains(*name) == hidden_only)
                    .map(|(name, value)| (name.as_str(), Some(value.as_str())))
                    .collect::<Vec<_>>();
                if !hidden_only {
                    entries.extend(removed.iter().map(|name| (name.as_str(), None)));
                }
                entries.sort_by_key(|(name, _)| *name);
                show_environment_entries(entries, shell)
            }
        };
    }
    match pos.first() {
        Some(name) if st.env_is_hidden(name) != hidden_only => CommandResult::ok(""),
        Some(name) if st.env_is_removed(name) => show_environment_value(name, None, shell),
        Some(name) => match st.get_env(name) {
            Some(value) => show_environment_value(name, Some(value), shell),
            None => CommandResult::err(format!("unknown variable: {name}\n")),
        },
        None => {
            let mut entries = st
                .env_iter()
                .filter(|(name, _)| st.env_is_hidden(name) == hidden_only)
                .map(|(name, value)| (name.as_str(), Some(value.as_str())))
                .collect::<Vec<_>>();
            if !hidden_only {
                entries.extend(st.removed_env_iter().map(|name| (name.as_str(), None)));
            }
            entries.sort_by_key(|(name, _)| *name);
            show_environment_entries(entries, shell)
        }
    }
}

fn show_environment_entries(entries: Vec<(&str, Option<&str>)>, shell: bool) -> CommandResult {
    let mut output = String::new();
    for (name, value) in entries {
        output.push_str(&show_environment_line(name, value, shell));
        output.push('\n');
    }
    CommandResult::ok(output)
}

fn show_environment_value(name: &str, value: Option<&str>, shell: bool) -> CommandResult {
    CommandResult::ok(format!("{}\n", show_environment_line(name, value, shell)))
}

fn show_environment_line(name: &str, value: Option<&str>, shell: bool) -> String {
    if !shell {
        return value
            .map(|value| format!("{name}={value}"))
            .unwrap_or_else(|| format!("-{name}"));
    }
    match value {
        Some(value) => {
            let mut escaped = String::with_capacity(value.len());
            for character in value.chars() {
                if matches!(character, '$' | '`' | '"' | '\\') {
                    escaped.push('\\');
                }
                escaped.push(character);
            }
            format!("{name}=\"{escaped}\"; export {name};")
        }
        None => format!("unset {name};"),
    }
}

/// `last-pane [-de] [-t target]`. Switches to the previously-active pane;
/// with `-e` or `-d` it instead enables or disables input on that pane
/// without switching, as tmux's `cmd-select-pane.c` does (`-e` wins when
/// both are given).
fn last_pane_cmd(args: &[String], st: &mut ServerState) -> CommandResult {
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    match target {
        Some(t) => {
            let result = if has_flag(args, "-e") {
                st.set_last_pane_input_off(&t, false)
            } else if has_flag(args, "-d") {
                st.set_last_pane_input_off(&t, true)
            } else {
                st.last_pane(&t)
            };
            match result {
                Ok(()) => CommandResult::ok(""),
                Err(error) => command_target_error(error, &t, "window"),
            }
        }
        None => CommandResult::err("can't establish current session\n"),
    }
}

/// `move-pane -s src -t dst`. Moves a pane into another window.
fn move_pane(args: &[String], st: &mut ServerState) -> CommandResult {
    let src = flag_value(args, "-s")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let dst = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let before = has_flag(args, "-b");
    match (src, dst) {
        (Some(s), Some(d)) => match st.move_pane(&s, &d, before) {
            Ok(()) => CommandResult::ok(""),
            Err(e) => CommandResult::err(format!("{e}\n")),
        },
        (None, _) => CommandResult::err("can't establish current session\n"),
        (_, None) => CommandResult::err("move-pane: missing destination\n"),
    }
}

/// `unlink-window [-k] [-t target]`. Removes the target winlink without killing
/// the physical window where it remains linked elsewhere; `-k` permits removal
/// of its final logical link.
fn unlink_window(args: &[String], st: &mut ServerState) -> CommandResult {
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_session(st).map(|session| format!("{session}:")));
    let target = match target {
        Some(t) => t,
        None => return CommandResult::err("can't establish current session\n"),
    };
    match st.unlink_window(&target, has_flag(args, "-k")) {
        Ok(()) => CommandResult::ok(""),
        Err(e) => CommandResult::err(format!("{e}\n")),
    }
}

/// A resolved physical row range. Rows are zero-based from the oldest retained
/// history row, matching tmux's internal grid coordinates.
#[derive(Clone, Copy, Debug)]
struct CaptureRange {
    top: usize,
    bottom: usize,
}

/// `capture-pane [-aCeFHJLMNpPqT] [-b name] [-E end] [-S start] [-t pane]`.
///
/// The command surface and operation selection follow tmux 3.7b, and so does
/// what they read: physical rows, soft wraps, the two per-row extents, the
/// prompt/output line flags and hyperlinks all come out of the engine's port of
/// `grid.c` rather than from a tmux-shaped text dump laid over something else.
fn capture_pane(args: &[String], st: &mut ServerState, agents: &PaneAgents) -> CommandResult {
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let target = match target {
        Some(t) => t,
        None => return CommandResult::err("can't establish current session\n"),
    };

    let resolved = match st.resolve(&target) {
        Some(resolved) => resolved,
        None => return CommandResult::err(format!("{}\n", st.pane_target_error(&target))),
    };

    // `-P` returns the bytes the pane's tokenizer is part-way through, tmux's
    // `input_pending`. `-H` takes precedence and continues to the grid
    // hyperlink operation, as in tmux.
    if has_flag(args, "-P") && !has_flag(args, "-H") {
        let pending = st.window(resolved.session, resolved.window).panes[resolved.pane]
            .pane
            .pending_input();
        let pending = if has_flag(args, "-C") {
            capture_escape_pending(&pending)
        } else {
            pending
        };
        return finish_capture(args, st, pending);
    }

    // `-a` reads the screen the alternate-screen switch displaced. With no
    // alternate screen up there is nothing to read and tmux fails, which `-q`
    // turns into an empty capture.
    let inactive = if has_flag(args, "-a") {
        let snapshot = st.window(resolved.session, resolved.window).panes[resolved.pane]
            .pane
            .inactive_snapshot();
        match snapshot {
            Ok(Some(snapshot)) => Some(snapshot),
            Ok(None) if has_flag(args, "-q") => return finish_capture(args, st, Vec::new()),
            Ok(None) => return CommandResult::err("no alternate screen\n"),
            Err(error) => return CommandResult::err(format!("{error}\n")),
        }
    } else {
        None
    };

    let mut vars = vars_full(
        st,
        &st.sessions()[resolved.session],
        resolved.window,
        resolved.pane,
        agents,
        st.marked_pane(),
    );
    for (name, value) in st.env_iter() {
        vars.set(name, value);
    }
    if let Ok(entries) = st.format_option_entries(&target) {
        for (name, value) in entries {
            vars.set(name, value);
        }
    }
    let use_mode = has_flag(args, "-M");
    let history_limit = st
        .option_for_target(&target, "history-limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(2000);
    let styled = has_flag(args, "-e") && !has_flag(args, "-H");
    // The snapshot walk is priced per cell, so the range is decided first —
    // from the frozen copy-mode grid when `-M` selects one, otherwise from the
    // grid's row geometry alone — and only the rows inside it are snapshotted.
    let text = {
        let node = &st.window(resolved.session, resolved.window).panes[resolved.pane];
        // Two reads arrive already materialized as a whole grid plus its `-e`
        // bytes: the screen copy mode froze, and the one the alternate screen
        // displaced. Neither is the live grid, and both are served the same way.
        let whole = inactive
            .as_ref()
            .map(|(grid, vt)| (grid, vt))
            .or_else(|| match use_mode {
                true => node.copy.as_ref().map(|copy| (&copy.grid, &copy.vt)),
                false => None,
            });
        if let Some((grid, vt)) = whole {
            if grid.rows.is_empty() {
                String::new()
            } else {
                let range = capture_range(
                    args,
                    grid.rows.len(),
                    grid.scrollback_rows,
                    grid.viewport_rows,
                    &vars,
                    history_limit,
                );
                let styled_rows = if styled {
                    let all = capture_vt_normalize_rows(vt, grid.rows.len());
                    Some(all[range.top..=range.bottom].to_vec())
                } else {
                    None
                };
                serialize_capture(args, grid, 0, range, styled_rows.as_deref())
            }
        } else {
            let dims = match node.pane.grid_dims() {
                Ok(dims) => dims,
                Err(error) => return CommandResult::err(format!("{error}\n")),
            };
            if dims.total_rows == 0 {
                String::new()
            } else {
                let range = capture_range(
                    args,
                    dims.total_rows,
                    dims.scrollback_rows,
                    dims.viewport_rows,
                    &vars,
                    history_limit,
                );
                let rows = range.bottom - range.top + 1;
                let grid = match node.pane.grid_snapshot_range(range.top, rows) {
                    Ok(grid) => grid,
                    Err(error) => return CommandResult::err(format!("{error}\n")),
                };
                let styled_rows = if styled {
                    let bytes = match node
                        .pane
                        .dump_rows_vt(range.top, rows, capture_extent(args))
                    {
                        Ok(bytes) => bytes,
                        Err(error) => return CommandResult::err(format!("{error}\n")),
                    };
                    Some(capture_vt_normalize_rows(&bytes, rows))
                } else {
                    None
                };
                serialize_capture(args, &grid, range.top, range, styled_rows.as_deref())
            }
        }
    };
    finish_capture(args, st, text.into_bytes())
}

/// Deliver a capture, which is bytes rather than text: `-P` returns whatever
/// the pane's parser is holding, and a half-read UTF-8 character in an OSC
/// payload is not a string.
fn finish_capture(args: &[String], st: &mut ServerState, mut bytes: Vec<u8>) -> CommandResult {
    if has_flag(args, "-p") {
        // tmux always prints one terminating newline, including for an empty
        // capture. Row serializers already end in one, so do not add a second.
        if !bytes.ends_with(b"\n") {
            bytes.push(b'\n');
        }
        return CommandResult::ok_bytes(bytes);
    }
    // Buffer captures retain the row serializer's final newline. An empty
    // parser-pending or hyperlink capture stores a genuinely empty buffer.
    st.set_buffer(flag_value(args, "-b"), &bytes);
    CommandResult::ok("")
}

/// `capture-pane -PC`, which escapes by its own rule rather than the one grid
/// rows use: tmux writes a byte literally only when it is at least a space and
/// not a backslash, and everything else as a three-digit octal escape. The
/// comparison is against a signed `char`, which puts every byte with the high
/// bit set on the escaped side.
fn capture_escape_pending(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    for &byte in bytes {
        if (b' '..0x80).contains(&byte) && byte != b'\\' {
            out.push(byte);
        } else {
            out.extend_from_slice(format!("\\{byte:03o}").as_bytes());
        }
    }
    out
}

fn capture_range(
    args: &[String],
    total_rows: usize,
    scrollback_rows: usize,
    viewport_rows: u16,
    vars: &format::Vars,
    history_limit: usize,
) -> CaptureRange {
    let last = total_rows.saturating_sub(1);
    let history = scrollback_rows.min(last);
    // History past `history-limit` is gone in tmux, so it must not be readable
    // here either; Ghostty keeps its scrollback by bytes rather than rows, so
    // the limit is applied where the rows are read back.
    let floor = history.saturating_sub(history_limit);
    let default_top = history;
    let default_bottom = history
        .saturating_add(viewport_rows.saturating_sub(1) as usize)
        .min(last);

    let mut top =
        capture_endpoint(flag_value(args, "-S"), default_top, history, last, vars).max(floor);
    let mut bottom =
        capture_endpoint(flag_value(args, "-E"), default_bottom, history, last, vars).max(floor);
    if bottom < top {
        std::mem::swap(&mut top, &mut bottom);
    }
    CaptureRange { top, bottom }
}

fn capture_endpoint(
    value: Option<&str>,
    default: usize,
    history: usize,
    last: usize,
    vars: &format::Vars,
) -> usize {
    let Some(value) = value else {
        return default;
    };
    if value == "-" {
        return if default == history { 0 } else { last };
    }
    let expanded = format::expand(value, vars);
    let Ok(offset) = expanded.parse::<i32>() else {
        return default;
    };
    if offset > i16::MAX as i32 {
        return default;
    }
    if offset < 0 && offset.unsigned_abs() as usize > history {
        return 0;
    }
    history.saturating_add_signed(offset as isize).min(last)
}

/// Serialize the rows of `range`. `grid.rows[0]` is physical row `start_row`,
/// so a range-limited snapshot indexes relative to it while `-L` numbering and
/// the range itself stay in physical-row terms.
fn serialize_capture(
    args: &[String],
    grid: &Grid,
    start_row: usize,
    range: CaptureRange,
    styled_rows: Option<&[String]>,
) -> String {
    let hyperlinks_only = has_flag(args, "-H");
    let join = has_flag(args, "-J");
    let line_numbers = has_flag(args, "-L");
    let line_flags = has_flag(args, "-F");
    let escape = has_flag(args, "-C");
    let mut seen_links = std::collections::HashSet::new();
    let mut out = String::new();

    for (relative, row_index) in (range.top..=range.bottom).enumerate() {
        let row = &grid.rows[row_index - start_row];
        let mut line = if hyperlinks_only {
            // tmux checks the row's flag before walking it and stops at the
            // written extent, so a link in a cell nothing wrote into is not
            // reported.
            let mut links = Vec::new();
            if row.flags.hyperlink {
                for cell in row.cells.iter().take(row.used.min(row.cells.len())) {
                    if let Some(link) = cell.hyperlink.as_ref() {
                        if seen_links.insert(link.clone()) {
                            links.push(link.clone());
                        }
                    }
                }
            }
            if links.is_empty() {
                continue;
            }
            links.join(" ")
        } else if let Some(styled) = styled_rows {
            let mut styled = styled.get(relative).cloned().unwrap_or_default();
            capture_trim_trailing(args, &mut styled);
            styled
        } else {
            capture_plain_row(row, args)
        };
        if escape && !hyperlinks_only {
            line = capture_escape(&line);
        }
        if line_numbers {
            let number = row_index as isize - grid.scrollback_rows as isize;
            out.push_str(&format!("{number} "));
        }
        if line_flags {
            out.push_str(&capture_row_flags(row));
            out.push(' ');
        }
        out.push_str(&line);
        if !join || !row.wrapped {
            out.push('\n');
        }
    }
    out
}

/// How far along each row this capture reads, tmux's `GRID_STRING_EMPTY_CELLS`:
/// to the written extent when `-J` or `-T` asked for it, and to the allocated
/// one otherwise.
fn capture_extent(args: &[String]) -> CaptureExtent {
    if has_flag(args, "-J") || has_flag(args, "-T") {
        CaptureExtent::Written
    } else {
        CaptureExtent::Allocated
    }
}

/// One row as text, following `grid_string_cells`'s two independent decisions:
/// how far along the row to read, and whether to trim what trails.
fn capture_plain_row(row: &GridRow, args: &[String]) -> String {
    let end = match capture_extent(args) {
        CaptureExtent::Written => row.used,
        CaptureExtent::Allocated => row.size,
    }
    .min(row.cells.len());

    let mut line = String::new();
    for cell in row.cells.iter().take(end) {
        if matches!(cell.width, CellWidth::SpacerTail | CellWidth::SpacerHead) {
            continue;
        }
        if cell.tab {
            line.push('\t');
        } else if cell.text.is_empty() {
            line.push(' ');
        } else {
            line.push_str(&cell.text);
        }
    }
    capture_trim_trailing(args, &mut line);
    line
}

/// tmux's `GRID_STRING_TRIM_SPACES`: a capture drops the blanks trailing a row
/// unless `-J` or `-N` asked to keep them.
///
/// It trims spaces and nothing else, so anything written just before them
/// survives. That is how a `-e` capture can end in a style change with no text
/// left to style: the row was read into its allocated blanks, the transition
/// into those blanks was emitted, and only the blanks went.
fn capture_trim_trailing(args: &[String], line: &mut String) {
    if has_flag(args, "-J") || has_flag(args, "-N") {
        return;
    }
    line.truncate(line.trim_end_matches(' ').len());
}

/// The `-F` flags, in tmux's order. `D` is absent because no grid a capture can
/// reach holds a dead line, and `X` is an allocation decision tmux lets show.
fn capture_row_flags(row: &GridRow) -> String {
    let mut flags = String::new();
    for (present, flag) in [
        (row.flags.hyperlink, 'H'),
        (row.flags.start_output, 'O'),
        (row.flags.start_prompt, 'P'),
        (row.wrapped, 'W'),
        (row.flags.extended, 'X'),
    ] {
        if present {
            flags.push(flag);
        }
    }
    if flags.is_empty() {
        flags.push('-');
    }
    flags
}

fn capture_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push('\t'),
            ' '..='~' => out.push(character),
            character if !character.is_ascii() => out.push(character),
            character => out.push_str(&format!("\\{:03o}", character as u32)),
        }
    }
    out
}

#[derive(Clone, Copy)]
enum CaptureToken<'a> {
    Text(&'a [u8]),
    Sgr(&'a [u8]),
    Osc(&'a [u8]),
    Acs(bool),
    Other,
}

fn capture_vt_normalize_rows(bytes: &[u8], rows: usize) -> Vec<String> {
    let mut out = Vec::with_capacity(rows);
    let mut decoder = SgrDecoder::default();
    let mut writer = CaptureStyleWriter::default();
    let mut presentation = CellPresentation::default();
    let mut start = 0usize;
    for end in bytes
        .iter()
        .enumerate()
        .filter_map(|(index, byte)| (*byte == b'\n').then_some(index))
    {
        let row = bytes[start..end]
            .strip_suffix(b"\r")
            .unwrap_or(&bytes[start..end]);
        out.push(capture_vt_normalize_row(
            row,
            &mut decoder,
            &mut writer,
            &mut presentation,
        ));
        start = end + 1;
        if out.len() == rows {
            return out;
        }
    }
    if out.len() < rows && start < bytes.len() {
        let row = bytes[start..]
            .strip_suffix(b"\r")
            .unwrap_or(&bytes[start..]);
        out.push(capture_vt_normalize_row(
            row,
            &mut decoder,
            &mut writer,
            &mut presentation,
        ));
    }
    out.resize(rows, String::new());
    out
}

fn capture_vt_normalize_row(
    bytes: &[u8],
    decoder: &mut SgrDecoder,
    writer: &mut CaptureStyleWriter,
    presentation: &mut CellPresentation,
) -> String {
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != 0x1b {
            if matches!(bytes[index], 0x0e | 0x0f) {
                tokens.push(CaptureToken::Acs(bytes[index] == 0x0e));
                index += 1;
                continue;
            }
            let start = index;
            while index < bytes.len()
                && bytes[index] != 0x1b
                && !matches!(bytes[index], 0x0e | 0x0f)
            {
                index += 1;
            }
            tokens.push(CaptureToken::Text(&bytes[start..index]));
            continue;
        }
        if bytes.get(index + 1) == Some(&b'[') {
            let start = index;
            index += 2;
            while index < bytes.len() && !(0x40..=0x7e).contains(&bytes[index]) {
                index += 1;
            }
            if index < bytes.len() {
                let final_byte = bytes[index];
                index += 1;
                if final_byte == b'm' {
                    tokens.push(CaptureToken::Sgr(&bytes[start + 2..index - 1]));
                } else {
                    tokens.push(CaptureToken::Other);
                }
            }
            continue;
        }
        if bytes.get(index + 1) == Some(&b']') {
            index += 2;
            let content = index;
            while index < bytes.len() {
                if bytes[index] == 0x07 {
                    index += 1;
                    break;
                }
                if bytes[index] == 0x1b && bytes.get(index + 1) == Some(&b'\\') {
                    index += 2;
                    break;
                }
                index += 1;
            }
            let end = if bytes.get(index.wrapping_sub(1)) == Some(&0x07) {
                index - 1
            } else {
                index.saturating_sub(2)
            };
            tokens.push(CaptureToken::Osc(&bytes[content..end]));
            continue;
        }
        // The grid dump selects the line-drawing set with `ESC ( 0` and ASCII
        // back with `ESC ( B`; a capture spells the same run SO … SI, so both
        // ends of it have to be recognized — a missed `ESC ( B` would leave the
        // `B` in the captured text and the run open to the end of the row.
        if bytes.get(index + 1) == Some(&b'(') && matches!(bytes.get(index + 2), Some(b'0' | b'B')) {
            let acs = bytes[index + 2] == b'0';
            index += 3;
            tokens.push(CaptureToken::Acs(acs));
        } else {
            index = (index + 2).min(bytes.len());
            tokens.push(CaptureToken::Other);
        }
    }

    let Some(last_text) = tokens.iter().rposition(|token| {
        matches!(
            token,
            CaptureToken::Text(text)
                if text.iter().any(|byte| !matches!(byte, b'\r' | b'\n'))
        )
    }) else {
        for token in tokens {
            apply_capture_control(token, decoder, presentation);
        }
        return String::new();
    };

    let mut out = Vec::new();
    for (token_index, token) in tokens.iter().enumerate() {
        match token {
            CaptureToken::Text(text) if token_index <= last_text => {
                presentation.style = decoder.style();
                writer.transition(&mut out, presentation);
                out.extend_from_slice(text);
            }
            token => apply_capture_control(*token, decoder, presentation),
        }
    }
    writer.finish_row(&mut out);
    String::from_utf8_lossy(&out).into_owned()
}

fn apply_capture_control(
    token: CaptureToken<'_>,
    decoder: &mut SgrDecoder,
    presentation: &mut CellPresentation,
) {
    match token {
        CaptureToken::Sgr(parameters) => decoder.apply(parameters),
        CaptureToken::Osc(content) => {
            let Some(rest) = content.strip_prefix(b"8;") else {
                return;
            };
            let Some(separator) = rest.iter().position(|byte| *byte == b';') else {
                return;
            };
            let parameters = String::from_utf8_lossy(&rest[..separator]);
            let uri = String::from_utf8_lossy(&rest[separator + 1..]).into_owned();
            if uri.is_empty() {
                presentation.hyperlink = None;
                presentation.hyperlink_epoch = 0;
            } else {
                presentation.hyperlink = Some(Hyperlink {
                    id: parameters
                        .split(':')
                        .find_map(|field| field.strip_prefix("id=").map(str::to_string))
                        .unwrap_or_default(),
                    uri,
                });
                // The dump writes an OSC 8 exactly where the cell's link
                // changes, so counting them is what tells two anonymous links
                // naming one URI apart.
                presentation.hyperlink_epoch = presentation.hyperlink_epoch.wrapping_add(1);
            }
        }
        CaptureToken::Acs(value) => presentation.acs = value,
        CaptureToken::Text(_) | CaptureToken::Other => {}
    }
    presentation.style = decoder.style();
}

/// `swap-pane -s src -t dst`. Both default to the current session's active pane.
fn swap_pane(args: &[String], st: &mut ServerState) -> CommandResult {
    let cur = current_target(st);
    // `-U`/`-D` swap the target (default active) pane with its previous/next
    // neighbour; `-D` takes precedence when both are given (as in tmux).
    if has_bool_flag(args, 'D') || has_bool_flag(args, 'U') {
        let target = flag_value(args, "-t")
            .map(str::to_string)
            .or_else(|| cur.clone());
        let down = has_bool_flag(args, 'D');
        return match target {
            Some(t) => match st.swap_pane_neighbour(&t, down) {
                Ok(()) => CommandResult::ok(""),
                Err(e) => CommandResult::err(format!("{e}\n")),
            },
            None => CommandResult::err("can't establish current session\n"),
        };
    }
    let src = flag_value(args, "-s")
        .map(str::to_string)
        .or_else(|| cur.clone());
    let dst = flag_value(args, "-t").map(str::to_string).or(cur);
    match (src, dst) {
        (Some(s), Some(d)) => match st.swap_pane(&s, &d) {
            Ok(()) => CommandResult::ok(""),
            Err(e) => CommandResult::err(format!("{e}\n")),
        },
        _ => CommandResult::err("can't establish current session\n"),
    }
}

/// `rotate-window [-t target]`. Rotates the target window's panes.
fn rotate_window(args: &[String], st: &mut ServerState) -> CommandResult {
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    match target {
        Some(t) => match if has_bool_flag(args, 'D') {
            st.rotate_window_down(&t)
        } else {
            st.rotate_window(&t)
        } {
            Ok(()) => CommandResult::ok(""),
            Err(error) => command_target_error(error, &t, "window"),
        },
        None => CommandResult::err("can't establish current session\n"),
    }
}

/// `resize-pane`: toggle zoom or move one boundary in the retained layout tree.
/// `resize-pane -M`: drag the border the mouse grabbed to where it is now.
///
/// tmux installs a drag callback that runs on every later report; hmux instead
/// keeps the drag pinned to the border it started on (see
/// `MouseInputState::observe`) and re-runs this on each `MouseDrag1Border`, so
/// the border tracks the pointer the same way.
fn resize_pane_to_mouse(st: &mut ServerState) -> CommandResult {
    // tmux's `cmd_resize_pane_exec` returns quietly when there is no mouse
    // event, so a command client running `resize-pane -M` is not an error.
    let Some(mouse) = st.command_mouse() else {
        return CommandResult::ok("");
    };
    let Some((pane_id, side)) = mouse
        .target
        .as_ref()
        .filter(|target| target.location == super::key::MouseLocation::Border)
        .and_then(|target| Some((target.pane_id?, target.border_side?)))
    else {
        return CommandResult::ok("");
    };
    let position = mouse.position;
    let grabbed = mouse.last_position.unwrap_or(position);
    let target = format!("%{pane_id}");
    let Some(resolved) = st.resolve(&target) else {
        return CommandResult::ok("");
    };
    // A floating pane is not in the layout, so it is moved and resized
    // directly by whichever of its own borders the pointer grabbed.
    if st.drag_floating_pane(&target, (grabbed.x, grabbed.y), (position.x, position.y)) {
        return CommandResult::ok("");
    }
    let status_offset = if super::status::at_top(st, &target) {
        super::status::height(st, &target)
    } else {
        0
    };
    let Some(rect) = st
        .window(resolved.session, resolved.window)
        .pane_rect(pane_id)
    else {
        return CommandResult::ok("");
    };
    // Moving a pane's own top or left border grows it; moving its bottom or
    // right border shrinks or grows it from the other end. Either way the new
    // size is the distance from the edge that stayed put to the pointer.
    let pane_y = position.y.saturating_sub(status_offset);
    let (direction, size) = match side {
        super::mouse::BorderSide::Bottom => {
            (SplitDirection::TopBottom, pane_y.saturating_sub(rect.top))
        }
        super::mouse::BorderSide::Top => (
            SplitDirection::TopBottom,
            rect.top
                .saturating_add(rect.height)
                .saturating_sub(pane_y)
                .saturating_sub(1),
        ),
        super::mouse::BorderSide::Right => (
            SplitDirection::LeftRight,
            position.x.saturating_sub(rect.left),
        ),
        super::mouse::BorderSide::Left => (
            SplitDirection::LeftRight,
            rect.left
                .saturating_add(rect.width)
                .saturating_sub(position.x)
                .saturating_sub(1),
        ),
    };
    if size == 0 {
        return CommandResult::ok("");
    }
    match st.resize_pane_to(&target, direction, size) {
        Ok(()) => CommandResult::ok(""),
        Err(_) => CommandResult::ok(""),
    }
}

fn resize_pane(args: &[String], st: &mut ServerState) -> CommandResult {
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let target = match target {
        Some(t) => t,
        None => return CommandResult::err("can't establish current session\n"),
    };
    if st.resolve(&target).is_none() {
        return CommandResult::err(format!("{}\n", st.pane_target_error(&target)));
    }
    // `-T` trims the blank rows below the cursor and pulls the same number of
    // rows out of the history. tmux does nothing at all when the pane is in a
    // mode, whose screen is not the one that would be trimmed.
    if has_bool_flag(args, 'T') {
        let Some(resolved) = st.resolve(&target) else {
            return CommandResult::ok("");
        };
        let node = &st.window(resolved.session, resolved.window).panes[resolved.pane];
        if node.copy.is_some() {
            return CommandResult::ok("");
        }
        return match node.pane.trim_history_below_cursor() {
            Ok(()) => CommandResult::ok(""),
            Err(error) => CommandResult::err(format!("{error}\n")),
        };
    }
    if has_bool_flag(args, 'Z') {
        return match st.toggle_zoom(&target) {
            Ok(_) => CommandResult::ok(""),
            Err(e) => CommandResult::err(format!("{e}\n")),
        };
    }
    if has_bool_flag(args, 'M') {
        return resize_pane_to_mouse(st);
    }
    for (flag, direction, label) in [
        ("-x", SplitDirection::LeftRight, "width"),
        ("-y", SplitDirection::TopBottom, "height"),
    ] {
        if let Some(value) = flag_value(args, flag) {
            let size = match value.parse::<u16>() {
                Ok(size) => size,
                Err(_) => return CommandResult::err(format!("{label} invalid\n")),
            };
            if let Err(error) = st.resize_pane_to(&target, direction, size) {
                return CommandResult::err(format!("{error}\n"));
            }
        }
    }
    let adjustment = match positionals(args, &["-t", "-x", "-y"])
        .first()
        .copied()
        .unwrap_or("1")
        .parse::<u16>()
    {
        Ok(0) => return CommandResult::err("adjustment too small\n"),
        Ok(value) => value,
        Err(_) => return CommandResult::err("adjustment invalid\n"),
    };
    let direction = if has_flag(args, "-L") {
        Some((SplitDirection::LeftRight, false))
    } else if has_flag(args, "-R") {
        Some((SplitDirection::LeftRight, true))
    } else if has_flag(args, "-U") {
        Some((SplitDirection::TopBottom, false))
    } else if has_flag(args, "-D") {
        Some((SplitDirection::TopBottom, true))
    } else {
        None
    };
    match direction {
        Some((direction, forward)) => {
            match st.resize_pane(&target, direction, forward, adjustment) {
                Ok(()) => CommandResult::ok(""),
                Err(error) => CommandResult::err(format!("{error}\n")),
            }
        }
        None => CommandResult::ok(""),
    }
}

/// `resize-window [-aADLRU] [-x cols] [-y rows] [-t target] [adjustment]`.
///
/// Every form pins the window at a manual size: `-x`/`-y` set an axis outright,
/// `-L/-R/-U/-D` move one edge by `adjustment` cells (default 1), and `-a`/`-A`
/// snap to the smallest/largest client that can see the window. An
/// out-of-range or non-numeric `-x`/`-y` value, or a bad adjustment, is
/// rejected exactly like tmux.
fn resize_window(args: &[String], st: &mut ServerState) -> CommandResult {
    let cols = match parse_size_flag(args, "-x", "width") {
        Ok(v) => v,
        Err(e) => return CommandResult::err(e),
    };
    let rows = match parse_size_flag(args, "-y", "height") {
        Ok(v) => v,
        Err(e) => return CommandResult::err(e),
    };
    let adjustment = match positionals(args, &["-t", "-x", "-y"])
        .first()
        .copied()
        .unwrap_or("1")
        .parse::<u16>()
    {
        Ok(0) => return CommandResult::err("adjustment too small\n"),
        Ok(value) => value,
        Err(_) => return CommandResult::err("adjustment invalid\n"),
    };
    let adjust = if has_flag(args, "-L") {
        Some(WindowResizeAdjust::Left)
    } else if has_flag(args, "-R") {
        Some(WindowResizeAdjust::Right)
    } else if has_flag(args, "-U") {
        Some(WindowResizeAdjust::Up)
    } else if has_flag(args, "-D") {
        Some(WindowResizeAdjust::Down)
    } else {
        None
    };
    // `-A` wins over `-a`, as tmux's flag order does.
    let snap = if has_bool_flag(args, 'A') {
        Some(WindowSizePolicy::Largest)
    } else if has_bool_flag(args, 'a') {
        Some(WindowSizePolicy::Smallest)
    } else {
        None
    };
    if cols.is_none() && rows.is_none() && adjust.is_none() && snap.is_none() {
        return CommandResult::ok("");
    }
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let target = match target {
        Some(t) => t,
        None => return CommandResult::err("can't establish current session\n"),
    };
    let request = WindowResizeRequest {
        cols,
        rows,
        adjust,
        adjustment,
        snap,
    };
    match st.resize_window(&target, request) {
        Ok(()) => CommandResult::ok(""),
        Err(error) => command_target_error(error, &target, "window"),
    }
}

/// tmux's inclusive size bounds for `resize-window -x/-y` (`strtonum` range).
const WINDOW_SIZE_MIN: i64 = 1;
const WINDOW_SIZE_MAX: i64 = 10000;

/// Parse a `-x`/`-y` size flag value the way tmux does: a number in
/// `[1, 10000]`. `label` is the axis word ("width"/"height") tmux prints in its
/// diagnostic. Returns `Ok(None)` when the flag is absent, `Ok(Some(n))` for a
/// valid size, or tmux's `<label> invalid|too small|too large` on a bad one.
fn parse_size_flag(args: &[String], flag: &str, label: &str) -> Result<Option<u16>, String> {
    match flag_value(args, flag) {
        None => Ok(None),
        Some(v) => match v.parse::<i64>() {
            Err(_) => Err(format!("{label} invalid\n")),
            Ok(n) if n < WINDOW_SIZE_MIN => Err(format!("{label} too small\n")),
            Ok(n) if n > WINDOW_SIZE_MAX => Err(format!("{label} too large\n")),
            Ok(n) => Ok(Some(n as u16)),
        },
    }
}

/// The layout names tmux accepts by name for `select-layout` / `next-layout`.
pub(crate) const LAYOUT_NAMES: &[&str] = &[
    "even-horizontal",
    "even-vertical",
    "main-horizontal",
    "main-horizontal-mirrored",
    "main-vertical",
    "main-vertical-mirrored",
    "tiled",
];

/// `select-layout [-Enop] [-t target] [layout]`. Every invocation snapshots
/// the window's layout into tmux's `w->old_layout` slot first, which is what
/// `-o` restores; an error puts the previous snapshot back, as tmux does.
fn select_layout(args: &[String], st: &mut ServerState) -> CommandResult {
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let Some(target) = target else {
        return CommandResult::err("can't establish current session\n");
    };
    let previous_old = st.snapshot_window_layout(&target).ok().flatten();
    let result = select_layout_action(args, st, &target, previous_old.as_deref());
    if result.exit != 0 {
        st.restore_window_old_layout(&target, previous_old);
    }
    result
}

fn select_layout_action(
    args: &[String],
    st: &mut ServerState,
    target: &str,
    previous_old: Option<&str>,
) -> CommandResult {
    if has_flag(args, "-n") || has_flag(args, "-p") {
        return match st.cycle_layout(target, has_flag(args, "-n")) {
            Ok(()) => CommandResult::ok(""),
            Err(error) => command_target_error(error, target, "window"),
        };
    }
    if has_flag(args, "-E") {
        return match st.spread_window_layout(target) {
            Ok(()) => CommandResult::ok(""),
            Err(error) => command_target_error(error, target, "pane"),
        };
    }
    if let Some(layout) = positionals(args, &["-t"]).into_iter().next() {
        let known = LAYOUT_NAMES.iter().position(|name| name == &layout);
        let valid = known.is_some()
            // A custom layout dump carries a checksum + `,`-separated cells.
            || layout.contains(',');
        if !valid {
            return CommandResult::err(format!("invalid layout: {layout}\n"));
        }
        if let Some(layout) = known {
            return match st.select_named_layout(target, layout) {
                Ok(()) => CommandResult::ok(""),
                Err(error) => command_target_error(error, target, "pane"),
            };
        }
        return match st.select_custom_layout(target, layout) {
            Ok(()) => CommandResult::ok(""),
            Err(error) => CommandResult::err(format!("can't set layout: {error}\n")),
        };
    }
    if has_flag(args, "-o") {
        // `-o` re-applies the layout the *previous* command snapshot; with no
        // history it is a no-op, as in tmux.
        return match previous_old {
            Some(old) => match st.select_custom_layout(target, old) {
                Ok(()) => CommandResult::ok(""),
                Err(error) => CommandResult::err(format!("can't set layout: {error}\n")),
            },
            None => CommandResult::ok(""),
        };
    }
    // Bare `select-layout` reapplies the last preset (tmux's `w->lastlayout`).
    match st.window_last_preset_layout(target) {
        Ok(Some(preset)) => match st.select_named_layout(target, preset) {
            Ok(()) => CommandResult::ok(""),
            Err(error) => command_target_error(error, target, "pane"),
        },
        Ok(None) => CommandResult::ok(""),
        Err(error) => command_target_error(error, target, "pane"),
    }
}

fn cycle_layout(args: &[String], st: &mut ServerState, forward: bool) -> CommandResult {
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let Some(target) = target else {
        return CommandResult::err("can't establish current session\n");
    };
    // next-layout/previous-layout share select-layout's exec in tmux, so they
    // snapshot `w->old_layout` the same way.
    let previous_old = st.snapshot_window_layout(&target).ok().flatten();
    match st.cycle_layout(&target, forward) {
        Ok(()) => CommandResult::ok(""),
        Err(error) => {
            st.restore_window_old_layout(&target, previous_old);
            command_target_error(error, &target, "window")
        }
    }
}

/// `link-window -s src -t dst [-k] [-d]`. Adds the source window to the
/// destination session at the destination index. By default the linked window
/// becomes the destination session's current window; `-d` suppresses the follow.
fn link_window(args: &[String], st: &mut ServerState) -> CommandResult {
    let src = flag_value(args, "-s")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let dst = flag_value(args, "-t").map(str::to_string);
    let select = !has_flag(args, "-d");
    // `-a` (after) / `-b` (before) link the window *relative* to the `-t` anchor
    // index rather than onto it. `-b` takes precedence over `-a`.
    let relative = has_flag(args, "-a") || has_flag(args, "-b");
    match (src, dst) {
        (Some(s), Some(d)) => match if relative {
            st.link_window_relative(&s, &d, !has_flag(args, "-b"), select)
        } else {
            st.link_window(&s, &d, has_flag(args, "-k"), select)
        } {
            Ok(()) => CommandResult::ok(""),
            Err(error) => command_target_error_candidates(error, &[(&s, "window"), (&d, "window")]),
        },
        (None, _) => CommandResult::err("can't establish current session\n"),
        (_, None) => CommandResult::err("link-window: missing destination\n"),
    }
}

/// `break-pane [-s src-pane] [-t dst-window] [-n name] [-P] [-F format]`. Moves
/// a pane into a new window. A `-t` that carries a pane part is rejected exactly
/// like tmux. `-n` names the new window (default empty). With `-P`, prints the
/// new window via `-F` (or `NEW_WINDOW_TEMPLATE`).
fn break_pane(args: &[String], st: &mut ServerState) -> CommandResult {
    // `-t` names a *window*; a pane part (a `.` after the window) is an error.
    if let Some(dst) = flag_value(args, "-t") {
        if dst
            .rsplit_once('.')
            .is_some_and(|(_, p)| !p.is_empty() && p.parse::<u32>().is_ok())
        {
            return CommandResult::err("can't specify pane here\n");
        }
    }
    let src = flag_value(args, "-s")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let dst = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_session(st).map(|session| format!("{session}:")));
    let relative = (has_flag(args, "-a") || has_flag(args, "-b")).then_some(!has_flag(args, "-b"));
    match (src, dst) {
        (Some(s), Some(d)) => match st.break_pane(
            &s,
            &d,
            flag_value(args, "-n"),
            !has_flag(args, "-d"),
            relative,
        ) {
            Ok(target) if has_flag(args, "-P") => {
                let sess = &st.sessions()[target.session];
                let template = flag_value(args, "-F").unwrap_or(NEW_WINDOW_TEMPLATE);
                let marked = st.marked_pane();
                let line = expand_command_format(
                    st,
                    template,
                    &vars_full(
                        st,
                        sess,
                        target.window,
                        target.pane,
                        &PaneAgents::new(),
                        marked,
                    ),
                    None,
                );
                CommandResult::ok(format!("{line}\n"))
            }
            Ok(_) => CommandResult::ok(""),
            Err(error) => command_target_error(error, &d, "window"),
        },
        _ => CommandResult::err("can't establish current session\n"),
    }
}

fn command_target_error(error: io::Error, target: &str, target_type: &str) -> CommandResult {
    command_target_error_candidates(error, &[(target, target_type)])
}

fn command_target_error_candidates(error: io::Error, targets: &[(&str, &str)]) -> CommandResult {
    let message = error.to_string();
    for (target, target_type) in targets {
        if !target.contains([':', '.'])
            && !target.starts_with(['$', '@', '%'])
            && message == format!("can't find session: {target}")
        {
            return CommandResult::err(format!("can't find {target_type}: {target}\n"));
        }
    }
    CommandResult::err(format!("{message}\n"))
}

/// `set-buffer [-b name] [-n new-name] [data]`. Stores or renames a paste buffer.
fn set_buffer(args: &[String], st: &mut ServerState, context: &ClientContext) -> CommandResult {
    let name = flag_value(args, "-b");
    let client_target = flag_value(args, "-t");
    if let Some(new_name) = flag_value(args, "-n") {
        return match name {
            Some(name) if st.rename_buffer(name, new_name) => CommandResult::ok(""),
            Some(name) => CommandResult::err(format!("unknown buffer: {name}\n")),
            None => CommandResult::err("no buffer\n"),
        };
    }
    let data = positionals(args, &["-b", "-t", "-n"]).into_iter().next();
    match data {
        Some("") => CommandResult::ok(""),
        Some(d) => {
            if has_flag(args, "-a") {
                st.append_buffer(name, d.as_bytes());
            } else {
                st.set_buffer(name, d.as_bytes());
            }
            if has_flag(args, "-w") {
                if let Some(data) = st.buffer(name).map(<[u8]>::to_vec) {
                    let _ = st.set_client_selection(
                        client_target,
                        context.tty_name.as_deref(),
                        Some(data),
                    );
                }
            }
            CommandResult::ok("")
        }
        None => CommandResult::err("no data specified\n"),
    }
}

fn load_buffer(args: &[String], st: &mut ServerState, context: &ClientContext) -> CommandResult {
    let path = match positionals(args, &["-b", "-t"]).into_iter().next() {
        Some(path) => path,
        None => {
            return CommandResult::err("command load-buffer: too few arguments (need at least 1)\n")
        }
    };
    let path_buf = PathBuf::from(path);
    let resolved = if path_buf.is_relative() {
        context
            .cwd
            .as_deref()
            .unwrap_or_else(|| Path::new("."))
            .join(path_buf)
    } else {
        path_buf
    };
    let loaded = context
        .input_file
        .clone()
        .map(|result| result.map_err(std::io::Error::from_raw_os_error))
        .unwrap_or_else(|| std::fs::read(&resolved));
    match loaded {
        Ok(data) => {
            if !data.is_empty() {
                st.set_buffer(flag_value(args, "-b"), &data);
                // `-w` additionally writes the loaded buffer to the target
                // client's terminal selection, exactly as `set-buffer -w` does.
                if has_flag(args, "-w") {
                    let _ = st.set_client_selection(
                        flag_value(args, "-t"),
                        context.tty_name.as_deref(),
                        Some(data),
                    );
                }
            }
            CommandResult::ok("")
        }
        Err(error) => CommandResult::err(format!("{}: {path}\n", io_error_message(&error))),
    }
}

pub(crate) fn client_input_path(args: &[String], context: &ClientContext) -> Option<PathBuf> {
    if args.iter().any(|arg| arg == ";") {
        return None;
    }
    let spec = match registry::resolve_spec(args.first()?.as_str()) {
        SpecResolution::Spec(spec) => spec,
        _ => return None,
    };
    let normalized = normalize_argv(spec.name, args);
    if matches!(spec.name, "display-message" | "split-window") {
        return has_flag(&normalized, "-I").then(|| PathBuf::from("-"));
    }
    if spec.name != "load-buffer" {
        return None;
    }
    let path = positionals(&normalized, &["-b", "-t"]).into_iter().next()?;
    let path = PathBuf::from(path);
    if path.as_os_str() == "-" {
        return Some(path);
    }
    Some(if path.is_relative() {
        context
            .cwd
            .as_deref()
            .unwrap_or_else(|| Path::new("."))
            .join(path)
    } else {
        path
    })
}

pub(crate) fn load_buffer_client_path(args: &[String], context: &ClientContext) -> Option<PathBuf> {
    client_input_path(args, context).filter(|_| {
        args.first().is_some_and(|name| {
            matches!(
                registry::resolve_spec(name),
                SpecResolution::Spec(spec) if spec.name == "load-buffer"
            )
        })
    })
}

/// `paste-buffer`: transform buffer newlines and enqueue the result on the
/// target pane's nonblocking PTY input path.
fn paste_buffer(args: &[String], st: &mut ServerState) -> CommandResult {
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    let Some(target) = target else {
        return CommandResult::err("can't establish current session\n");
    };
    if st.resolve(&target).is_none() {
        return CommandResult::err(format!("{}\n", st.pane_target_error(&target)));
    }
    let requested = flag_value(args, "-b");
    let selected = match requested {
        Some(name) => st
            .buffers()
            .iter()
            .find(|(buffer_name, _)| buffer_name == name)
            .cloned(),
        None => st.buffers().first().cloned(),
    };
    let Some((name, data)) = selected else {
        return match requested {
            Some(name) => CommandResult::err(format!("no buffer {name}\n")),
            None => CommandResult::ok(""),
        };
    };
    let separator =
        flag_value(args, "-s").unwrap_or(if has_flag(args, "-r") { "\n" } else { "\r" });
    let mut bytes = Vec::with_capacity(data.len());
    for byte in data {
        if byte == b'\n' {
            bytes.extend_from_slice(separator.as_bytes());
        } else {
            bytes.push(byte);
        }
    }
    if has_flag(args, "-p") && st.pane_bracketed_paste(&target).unwrap_or(false) {
        bytes.splice(0..0, b"\x1b[200~".iter().copied());
        bytes.extend_from_slice(b"\x1b[201~");
    }
    if let Err(error) = st.input_to_pane(&target, &bytes) {
        return CommandResult::err(format!("{error}\n"));
    }
    if has_flag(args, "-d") {
        st.delete_buffer(&name);
    }
    CommandResult::ok("")
}

/// `show-buffer [-b name]`. Prints the buffer's contents (no trailing newline,
/// matching tmux), or errors if there's no such buffer.
fn show_buffer(args: &[String], st: &ServerState) -> CommandResult {
    let name = flag_value(args, "-b");
    match st.buffer(name) {
        Some(data) => CommandResult::ok_bytes(data.to_vec()),
        None => match name {
            Some(n) => CommandResult::err(format!("no buffer {n}\n")),
            None => CommandResult::err("no buffers\n"),
        },
    }
}

/// `save-buffer [-a] [-b name] path`. Writes a paste buffer's contents to `path`;
/// tmux's stdout sink is `path == "-"`, where the raw bytes are emitted with no
/// trailing newline (exactly what `show-buffer` prints). `-a` appends to an
/// existing file instead of truncating it. Buffer resolution mirrors tmux's
/// shared `cmd-save-buffer.c`: an unknown named buffer is `no buffer NAME` and no
/// buffer at all is `no buffers`. Arity (the mandatory path) is checked first, as
/// tmux does it at parse time before the command runs.
fn save_buffer(args: &[String], st: &ServerState) -> CommandResult {
    // `positionals` treats a bare `-` as a flag, so scan for the path directly.
    let path = match save_buffer_path(args) {
        Some(p) => p,
        None => {
            return CommandResult::err(
                "command save-buffer: too few arguments (need at least 1)\n",
            );
        }
    };
    let name = flag_value(args, "-b");
    let data = match st.buffer(name) {
        Some(d) => d.to_vec(),
        None => {
            return match name {
                Some(n) => CommandResult::err(format!("no buffer {n}\n")),
                None => CommandResult::err("no buffers\n"),
            };
        }
    };
    if path == "-" {
        // stdout sink: emit the raw bytes with no trailing newline, like tmux.
        return CommandResult::ok_bytes(data);
    }
    let result = if has_flag(args, "-a") {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut f| std::io::Write::write_all(&mut f, &data))
    } else {
        std::fs::write(path, &data)
    };
    match result {
        Ok(()) => CommandResult::ok(""),
        Err(e) => CommandResult::err(format!("{}: {path}\n", io_error_message(&e))),
    }
}

pub(crate) struct ClientFileWrite {
    pub(crate) path: PathBuf,
    pub(crate) display_path: String,
    pub(crate) flags: i32,
    pub(crate) data: Vec<u8>,
}

pub(crate) fn save_buffer_client_request(
    args: &[String],
    state: &ServerState,
    context: &ClientContext,
) -> Option<Result<ClientFileWrite, CommandResult>> {
    if args.iter().any(|arg| arg == ";") {
        return None;
    }
    let spec = match registry::resolve_spec(args.first()?.as_str()) {
        SpecResolution::Spec(spec) if spec.name == "save-buffer" => spec,
        _ => return None,
    };
    let normalized = normalize_argv(spec.name, args);
    let path = save_buffer_path(&normalized)?;
    if path == "-" {
        return None;
    }
    let name = flag_value(&normalized, "-b");
    let data = match state.buffer(name) {
        Some(data) => data.to_vec(),
        None => {
            return Some(Err(match name {
                Some(name) => CommandResult::err(format!("no buffer {name}\n")),
                None => CommandResult::err("no buffers\n"),
            }));
        }
    };
    let requested = PathBuf::from(path);
    let path = if requested.is_relative() {
        context
            .cwd
            .as_deref()
            .unwrap_or_else(|| Path::new("."))
            .join(requested)
    } else {
        requested
    };
    let flags = libc::O_WRONLY
        | libc::O_CREAT
        | if has_flag(&normalized, "-a") {
            libc::O_APPEND
        } else {
            libc::O_TRUNC
        };
    let display_path = path.to_string_lossy().into_owned();
    Some(Ok(ClientFileWrite {
        path,
        display_path,
        flags,
        data,
    }))
}

/// The destination path for `save-buffer`: the sole positional. Unlike the generic
/// `positionals` helper, a bare `-` (tmux's stdout sink) counts as the path rather
/// than being skipped as a flag.
fn save_buffer_path(args: &[String]) -> Option<&str> {
    let mut i = 1; // skip the command name
    while i < args.len() {
        let a = args[i].as_str();
        if a == "-" {
            return Some(a);
        }
        if a == "-b" {
            i += 2; // skip the flag and its value
            continue;
        }
        if a.starts_with('-') && a != "-" {
            i += 1; // a valueless flag (e.g. -a)
            continue;
        }
        return Some(a);
    }
    None
}

/// Render an I/O error the way tmux does — `strerror(errno)` — by trimming Rust's
/// trailing ` (os error N)` from the [`std::io::Error`] display.
fn io_error_message(e: &std::io::Error) -> String {
    let s = e.to_string();
    match s.rfind(" (os error ") {
        Some(idx) => s[..idx].to_string(),
        None => s,
    }
}

/// The format variables one paste buffer answers to, shared by `list-buffers`
/// and `choose-buffer`.
pub(super) fn buffer_vars(st: &ServerState, name: &str, data: &[u8]) -> Vars {
    let mut vars = Vars::new();
    vars.set("buffer_name", name.to_owned())
        .set("buffer_size", data.len().to_string())
        .set("buffer_sample", String::from_utf8_lossy(data).into_owned())
        // `buffer_sample` is the shortened, printable form a listing shows;
        // `buffer_full` is the buffer's whole contents.
        .set("buffer_full", String::from_utf8_lossy(data).into_owned())
        .set(
            "buffer_created",
            st.buffer_created(name)
                .map(|created| created.to_string())
                .unwrap_or_default(),
        );
    vars
}

/// `list-buffers [-f filter] [-F format]`. Lists paste buffers, newest first.
fn list_buffers(args: &[String], st: &ServerState) -> CommandResult {
    let template = flag_value(args, "-F");
    let filter = flag_value(args, "-f");
    let (sort_order, reversed) = match list_sort_criteria(args) {
        Ok(criteria) => criteria,
        Err(error) => return error,
    };
    let mut buffers: Vec<(usize, &(String, Vec<u8>))> = st.buffers().iter().enumerate().collect();
    apply_list_sort(
        &mut buffers,
        sort_order,
        reversed,
        |key, (pos_a, (_, data_a)), (pos_b, (_, data_b))| match key {
            // The stack is newest first, which is tmux's descending
            // `pb->order` comparison for the creation key.
            ListSortOrder::Creation => pos_a.cmp(pos_b),
            ListSortOrder::Size => data_a.len().cmp(&data_b.len()),
            _ => std::cmp::Ordering::Equal,
        },
        |(_, (name, _))| name.clone(),
    );
    let default_line = "#{buffer_name}: #{buffer_size} bytes: \"#{buffer_sample}\"";
    let mut out = String::new();
    for (_, (name, data)) in buffers {
        let v = buffer_vars(st, name, data);
        if let Some(f) = filter {
            if !format::is_true(&expand_command_format(st, f, &v, None)) {
                continue;
            }
        }
        out.push_str(&expand_command_format(
            st,
            template.unwrap_or(default_line),
            &v,
            None,
        ));
        out.push('\n');
    }
    CommandResult::ok(out)
}

/// `delete-buffer [-b name]`. Removes a buffer (the most recent if unnamed).
fn delete_buffer(args: &[String], st: &mut ServerState) -> CommandResult {
    let name = flag_value(args, "-b")
        .map(str::to_string)
        .or_else(|| st.buffers().first().map(|(n, _)| n.clone()));
    match name {
        Some(n) if st.delete_buffer(&n) => CommandResult::ok(""),
        Some(n) => CommandResult::err(format!("unknown buffer: {n}\n")),
        None => CommandResult::err("no buffers\n"),
    }
}

fn shell_command(command: &str, context: &ClientContext) -> std::process::Command {
    let mut shell = std::process::Command::new("sh");
    shell.arg("-c").arg(command);
    // tmux's `environ_push` replaces the child's environment outright rather
    // than adding to the server's own, so the context's environment is the
    // whole of it.
    shell.env_clear();
    for entry in &context.environment {
        if let Some((name, value)) = entry.split_once('=') {
            shell.env(name, value);
        }
    }
    shell
}

fn job_delay(args: &[String]) -> Result<std::time::Duration, CommandResult> {
    let Some(value) = flag_value(args, "-d") else {
        return Ok(std::time::Duration::ZERO);
    };
    let seconds = value
        .parse::<f64>()
        .ok()
        .filter(|seconds| seconds.is_finite() && *seconds >= 0.0)
        .ok_or_else(|| CommandResult::err(format!("invalid delay time: {value}\n")))?;
    Ok(std::time::Duration::from_secs_f64(seconds))
}

pub(crate) struct RunShellCompletion {
    result: CommandResult,
    view: Option<(String, Vec<u8>)>,
}

impl RunShellCompletion {
    /// What the suspended `run-shell` will report to its client.
    #[cfg(test)]
    pub(crate) fn result(&self) -> &CommandResult {
        &self.result
    }

    /// Deliver a *detached* job's output, which has no client to fall back on.
    ///
    /// tmux resolves the view pane when the child finishes, so a `-t` pane that
    /// has since died sends the output through the same
    /// `cmd_find_from_nothing` lookup a job that named no pane uses. With no
    /// pane at all the output is dropped: `cmd_run_shell_print` returns without
    /// an item to print through.
    pub(crate) fn deliver_detached(self, state: &mut ServerState) {
        let Some((target, output)) = self.view else {
            return;
        };
        if state.append_view_output(&target, &output).is_err()
            && target != suspend::BackgroundShellJob::VIEW_FALLBACK
        {
            let _ = state.append_view_output(suspend::BackgroundShellJob::VIEW_FALLBACK, &output);
        }
    }
}

/// Rewrite `run-shell`'s `-t` to the pane's stable `%id`.
///
/// tmux keys the output view on the pane itself, so a pane that dies mid-run
/// must not let the *name* re-resolve to a survivor: the output falls back
/// instead — to the client for a waiting job, and to the current pane for a
/// detached one.
fn pin_run_shell_view_target(args: &[String], state: &ServerState) -> Vec<String> {
    let mut args = args.to_vec();
    let Some(position) = args.iter().position(|arg| arg == "-t") else {
        return args;
    };
    let Some(resolved) = args.get(position + 1).and_then(|value| state.resolve(value)) else {
        return args;
    };
    let pane_id = state.window(resolved.session, resolved.window).panes[resolved.pane].id;
    args[position + 1] = format!("%{pane_id}");
    args
}

fn finish_run_shell(completion: RunShellCompletion, state: &mut ServerState) -> CommandResult {
    let mut result = completion.result;
    if let Some((target, output)) = completion.view {
        // tmux resolves the view pane when the child finishes; with the pane
        // gone by then the output falls back to the invoking client, the way
        // `cmdq_print` does without a pane to draw into.
        if state.append_view_output(&target, &output).is_err() {
            result.stdout.push_str(&String::from_utf8_lossy(&output));
        }
    }
    result
}

/// {send -M} {copy-mode -M}`, and the branch decides between a client-local
/// outcome (entering copy mode, resizing) and an ordinary command. Resolving
/// the condition before dispatch is what lets the attach loop keep handling
/// those outcomes itself instead of losing them inside the command interpreter.
pub(super) fn resolve_conditional_binding(
    command: Vec<String>,
    st: &mut ServerState,
    agents: &PaneAgents,
    context: &ClientContext,
) -> Vec<String> {
    // Bounded: a binding that somehow nests conditionals forever must not hang
    // the client's input loop.
    let mut command = command;
    for _ in 0..4 {
        if !matches!(command.first().map(String::as_str), Some("if-shell" | "if")) {
            break;
        }
        let args = normalize_argv("if-shell", &command);
        if !has_flag(&args, "-F") || has_flag(&args, "-b") {
            break;
        }
        let positional = positionals(&args, &["-t"]);
        let Some(condition) = positional.first() else {
            break;
        };
        let previous = st.replace_command_mouse(context.mouse.clone());
        let expanded = expand_if_cond(condition, &args, st, agents);
        st.replace_command_mouse(previous);
        let branch = if !expanded.is_empty() && expanded != "0" {
            positional.get(1)
        } else {
            positional.get(2)
        };
        command = branch.map(|line| binding_words(line)).unwrap_or_default();
    }
    command
}

/// Expand `if-shell -F`'s condition as a format, anchored at the command's
/// target (`-t`, else the current session) so `#{...}` references resolve
/// against the live tree. Falls back to an empty context when no target
/// resolves — matching real tmux, which still expands the format.

/// Expand `if-shell -F`'s condition as a format, anchored at the command's
/// target (`-t`, else the current session) so `#{...}` references resolve
/// against the live tree. Falls back to an empty context when no target
/// resolves — matching real tmux, which still expands the format.
fn expand_if_cond(cond: &str, args: &[String], st: &ServerState, agents: &PaneAgents) -> String {
    let target = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(st));
    if let Some(t) = target {
        if let Some(r) = st.resolve(&t) {
            let vars = vars_full(
                st,
                &st.sessions()[r.session],
                r.window,
                r.pane,
                agents,
                st.marked_pane(),
            );
            let loops = TreeLoops {
                st,
                session: r.session,
                window: r.window,
                agents,
            };
            return expand_command_format(st, cond, &vars, Some(&loops));
        }
    }
    expand_command_format(st, cond, &Vars::default(), None)
}

/// `source-file [-Fnqv] [-t target] path ...`. Reads each file of tmux commands
/// and runs them, exactly as `.tmux.conf` is loaded. A path that can't be opened
/// One parsed configuration file: the command lines with their file line
/// numbers, plus the parser assignments made in active branches, which the
/// caller publishes to the global environment as tmux does.
struct SourcedConfig {
    lines: Vec<(usize, Vec<LineToken>)>,
    assignments: Vec<(String, String, bool)>,
}

/// One state of the `%if`/`%elif`/`%else` conditional stack.
struct SourceCondition {
    parent_active: bool,
    /// Whether any branch of this chain has been taken yet.
    taken: bool,
    active: bool,
    seen_else: bool,
}

/// Split a sourced config file into command argv lines. This preprocessing
/// layer handles the configuration-only syntax which cannot be represented by
/// ordinary command argv: conditional directives, parser assignments, and
/// brace command blocks. `environment` is the server's global environment,
/// which seeds `$NAME` expansion; an undefined name expands to nothing.
///
/// Like tmux, the whole file is parsed before anything runs, so a structural
/// error — an unbalanced conditional or an invalid escape — rejects the file:
/// the error is `(line, diagnostic)`.

/// Split a sourced config file into command argv lines. This preprocessing
/// layer handles the configuration-only syntax which cannot be represented by
/// ordinary command argv: conditional directives, parser assignments, and
/// brace command blocks. `environment` is the server's global environment,
/// which seeds `$NAME` expansion; an undefined name expands to nothing.
///
/// Like tmux, the whole file is parsed before anything runs, so a structural
/// error — an unbalanced conditional or an invalid escape — rejects the file:
/// the error is `(line, diagnostic)`.
fn source_lines(
    contents: &str,
    environment: &BTreeMap<String, String>,
) -> Result<SourcedConfig, (usize, String)> {
    let mut lines = Vec::new();
    let mut logical = String::new();
    let mut logical_start = 1;
    let mut assignments = environment.clone();
    let mut published = Vec::new();
    let mut conditions = Vec::<SourceCondition>::new();
    let mut brace_block: Option<(usize, Vec<LineToken>, String, usize)> = None;
    let mut line_number = 0;
    for raw in contents.lines() {
        line_number += 1;
        let continued = raw.trim_end().ends_with('\\');
        let part = if continued {
            raw.trim_end().strip_suffix('\\').unwrap_or(raw)
        } else {
            raw
        };
        if logical.is_empty() {
            logical_start = line_number;
        }
        // A backslash-newline splices the lines together at the character
        // level, continuing the same word, so no separator is inserted.
        logical.push_str(part);
        if continued {
            continue;
        }
        let trimmed = logical.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            logical.clear();
            continue;
        }

        if let Some(condition) = trimmed.strip_prefix("%if ") {
            let parent_active = conditions.iter().all(|condition| condition.active);
            let expanded = format::expand(condition.trim(), &Vars::default());
            let active = parent_active && format::is_true(&expanded);
            conditions.push(SourceCondition {
                parent_active,
                taken: active,
                active,
                seen_else: false,
            });
            logical.clear();
            continue;
        }
        if let Some(condition) = trimmed.strip_prefix("%elif ") {
            let Some(top) = conditions.last_mut() else {
                return Err((logical_start, "syntax error".to_string()));
            };
            if top.seen_else {
                return Err((logical_start, "syntax error".to_string()));
            }
            let expanded = format::expand(condition.trim(), &Vars::default());
            top.active = top.parent_active && !top.taken && format::is_true(&expanded);
            top.taken |= top.active;
            logical.clear();
            continue;
        }
        if trimmed == "%else" {
            let Some(top) = conditions.last_mut() else {
                return Err((logical_start, "syntax error".to_string()));
            };
            if top.seen_else {
                return Err((logical_start, "syntax error".to_string()));
            }
            top.seen_else = true;
            top.active = top.parent_active && !top.taken;
            top.taken |= top.active;
            logical.clear();
            continue;
        }
        if trimmed == "%endif" {
            if conditions.pop().is_none() {
                return Err((logical_start, "syntax error".to_string()));
            }
            logical.clear();
            continue;
        }
        if !conditions.iter().all(|condition| condition.active) {
            logical.clear();
            continue;
        }

        if brace_block.is_none() {
            let (assignment, hidden) = match trimmed.strip_prefix("%hidden") {
                Some(rest) if rest.starts_with(char::is_whitespace) => (rest.trim_start(), true),
                _ => (trimmed, false),
            };
            if let Some((name, value)) = parse_source_assignment(assignment) {
                assignments.insert(name.to_string(), value.to_string());
                published.push((name.to_string(), value.to_string(), hidden));
                logical.clear();
                continue;
            }
        }
        let expanded = expand_source_assignments(trimmed, &assignments);

        if let Some((_line, _prefix, body, depth)) = brace_block.as_mut() {
            let opens = expanded.chars().filter(|ch| *ch == '{').count();
            let closes = expanded.chars().filter(|ch| *ch == '}').count();
            let new_depth = depth.saturating_add(opens).saturating_sub(closes);
            // Only the block's own closing brace is syntax; an inner one
            // belongs to the body verbatim.
            if !(new_depth == 0 && expanded.trim() == "}") {
                if !body.is_empty() {
                    body.push_str(" ; ");
                }
                body.push_str(expanded.trim());
            }
            *depth = new_depth;
            if *depth == 0 {
                let (line, mut prefix, body, _) = brace_block.take().expect("active brace block");
                prefix.push(LineToken::Word(body));
                lines.push((line, prefix));
            }
            logical.clear();
            continue;
        }

        if let Some(prefix) = expanded.trim_end().strip_suffix('{') {
            let tokens = tokenize_line_checked(prefix.trim_end())
                .map_err(|message| (logical_start, message))?;
            brace_block = Some((logical_start, tokens, String::new(), 1));
            logical.clear();
            continue;
        }

        let argv = tokenize_line_checked(&expanded).map_err(|message| (logical_start, message))?;
        if !argv.is_empty() {
            lines.push((logical_start, argv));
        }
        logical.clear();
    }
    if !logical.trim().is_empty() {
        let argv = tokenize_line_checked(logical.trim_start())
            .map_err(|message| (logical_start, message))?;
        if !argv.is_empty() {
            lines.push((logical_start, argv));
        }
    }
    if !conditions.is_empty() {
        return Err((line_number + 1, "syntax error".to_string()));
    }
    Ok(SourcedConfig {
        lines,
        assignments: published,
    })
}

fn parse_source_assignment(line: &str) -> Option<(&str, &str)> {
    let (name, value) = line.split_once('=')?;
    if name.is_empty()
        || !name
            .chars()
            .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
        || name.as_bytes().first().is_some_and(u8::is_ascii_digit)
    {
        return None;
    }
    Some((name, value.trim_matches('"')))
}

fn expand_source_assignments(line: &str, assignments: &BTreeMap<String, String>) -> String {
    let mut output = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    let mut single_quoted = false;
    while let Some(ch) = chars.next() {
        if ch == '\'' {
            single_quoted = !single_quoted;
            output.push(ch);
            continue;
        }
        if ch != '$' || single_quoted {
            output.push(ch);
            continue;
        }
        let braced = chars.peek() == Some(&'{');
        if braced {
            chars.next();
        }
        let mut name = String::new();
        while chars
            .peek()
            .is_some_and(|next| *next == '_' || next.is_ascii_alphanumeric())
        {
            name.push(chars.next().expect("peeked assignment name"));
        }
        if braced && chars.peek() == Some(&'}') {
            chars.next();
        }
        if let Some(value) = assignments.get(&name) {
            output.push_str(value);
        } else if name.is_empty() {
            // A bare `$` is not a variable reference; keep it.
            output.push('$');
            if braced {
                output.push('{');
            }
        }
        // An undefined `$NAME` expands to nothing, as in tmux.
    }
    output
}

fn source_verbose_line(tokens: &[LineToken]) -> String {
    let mut words = tokens
        .iter()
        .map(|token| match token {
            LineToken::Word(word) => word.clone(),
            LineToken::Separator => ";".to_string(),
        })
        .collect::<Vec<_>>();
    if let Some(first) = words.first_mut() {
        if let Resolution::Name(canonical) = registry::resolve(first) {
            *first = canonical.to_string();
        }
    }
    words.join(" ")
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum LineToken {
    Word(String),
    Separator,
}

impl LineToken {
    fn word(&self) -> Option<&str> {
        match self {
            Self::Word(word) => Some(word),
            Self::Separator => None,
        }
    }
}

/// Tokenize one command line into words, honoring `'`/`"` quoting and retaining
/// an unquoted `;` as syntax rather than flattening it into an argv
/// value. Quoted and escaped semicolons are ordinary [`LineToken::Word`] values.
///
/// Backslash escapes decode as in tmux's command lexer both bare and inside
/// double quotes: octal (`\101`), the named control characters, and `\u`/`\U`
/// unicode escapes; any other escaped character stands for itself. Single
/// quotes are fully literal. An unquoted `{` starting a word collects a brace
/// block verbatim into one word, and a word-initial `~` outside single quotes
/// expands to the home directory.
///
/// An invalid escape strips the backslash and keeps the characters; the
/// checked variant used for configuration files rejects it instead.
fn tokenize_line(line: &str) -> Vec<LineToken> {
    tokenize_line_impl(line, false).unwrap_or_default()
}

/// Strict [`tokenize_line`] for configuration files: an invalid escape is an
/// error that rejects the whole file, as in tmux's parser.
fn tokenize_line_checked(line: &str) -> Result<Vec<LineToken>, String> {
    tokenize_line_impl(line, true)
}

/// Split a command line into the word list a stored key binding holds, with
/// command separators kept as literal `;` words.
///
/// The default key table is written as tmux command lines (the same shape as
/// `key_bindings_init`'s strings) rather than hand-split argv, so the bindings
/// stay readable next to the tmux source they come from.
pub(super) fn binding_words(line: &str) -> Vec<String> {
    tokenize_line(line)
        .into_iter()
        .map(|token| match token {
            LineToken::Word(word) => word,
            LineToken::Separator => ";".to_string(),
        })
        .collect()
}

/// Decode one backslash escape whose `\` has already been consumed.
fn push_token_escape(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    cur: &mut String,
    strict: bool,
) -> Result<(), String> {
    let Some(first) = chars.next() else {
        return Ok(());
    };
    match first {
        '0'..='3' => {
            let mut consumed = String::from(first);
            for _ in 0..2 {
                match chars.peek().copied() {
                    Some(digit @ '0'..='7') => {
                        consumed.push(digit);
                        chars.next();
                    }
                    _ => {
                        if strict {
                            return Err("invalid octal escape".to_string());
                        }
                        cur.push_str(&consumed);
                        return Ok(());
                    }
                }
            }
            let value = u8::from_str_radix(&consumed, 8).expect("three octal digits");
            cur.push(char::from(value));
        }
        '4'..='9' => {
            if strict {
                return Err("invalid octal escape".to_string());
            }
            cur.push(first);
        }
        'u' | 'U' => {
            let digits = if first == 'u' { 4 } else { 8 };
            let mut consumed = String::new();
            while consumed.len() < digits && chars.peek().is_some_and(char::is_ascii_hexdigit) {
                consumed.push(chars.next().expect("peeked hex digit"));
            }
            let decoded = (consumed.len() == digits)
                .then(|| u32::from_str_radix(&consumed, 16).ok())
                .flatten()
                .and_then(char::from_u32);
            match decoded {
                Some(decoded) => cur.push(decoded),
                None if strict => return Err("invalid \\u argument".to_string()),
                None => {
                    cur.push(first);
                    cur.push_str(&consumed);
                }
            }
        }
        'a' => cur.push('\x07'),
        'b' => cur.push('\x08'),
        'e' => cur.push('\x1b'),
        'f' => cur.push('\x0c'),
        'n' => cur.push('\n'),
        'r' => cur.push('\r'),
        's' => cur.push(' '),
        't' => cur.push('\t'),
        'v' => cur.push('\x0b'),
        other => cur.push(other),
    }
    Ok(())
}

fn tokenize_line_impl(line: &str, strict: bool) -> Result<Vec<LineToken>, String> {
    let mut words = Vec::new();
    let mut cur = String::new();
    let mut in_word = false;
    let mut word_tilde = false;
    let mut quote: Option<char> = None;
    let mut quote_opened = false;
    let mut chars = line.chars().peekable();

    fn finish_word(
        words: &mut Vec<LineToken>,
        cur: &mut String,
        in_word: &mut bool,
        word_tilde: &mut bool,
    ) {
        if *in_word {
            let mut word = std::mem::take(cur);
            if *word_tilde {
                if let Ok(home) = std::env::var("HOME") {
                    if word == "~" {
                        word = home;
                    } else if let Some(rest) = word.strip_prefix("~/") {
                        word = format!("{home}/{rest}");
                    }
                }
            }
            words.push(LineToken::Word(word));
            *in_word = false;
        }
        *word_tilde = false;
    }

    while let Some(c) = chars.next() {
        let at_quote_start = quote_opened;
        quote_opened = false;
        match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                } else if q == '"' && c == '\\' {
                    push_token_escape(&mut chars, &mut cur, strict)?;
                } else {
                    if c == '~' && q == '"' && at_quote_start && cur.is_empty() {
                        word_tilde = true;
                    }
                    cur.push(c);
                }
            }
            None => match c {
                '\'' | '"' => {
                    in_word = true;
                    quote = Some(c);
                    quote_opened = true;
                }
                c if c.is_whitespace() => {
                    finish_word(&mut words, &mut cur, &mut in_word, &mut word_tilde);
                }
                ';' => {
                    finish_word(&mut words, &mut cur, &mut in_word, &mut word_tilde);
                    words.push(LineToken::Separator);
                }
                '#' if !in_word => break,
                '{' if !in_word => {
                    // A brace block is one argument holding its body verbatim.
                    let mut depth = 1usize;
                    let mut body = String::new();
                    for inner in chars.by_ref() {
                        match inner {
                            '{' => depth += 1,
                            '}' => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                        body.push(inner);
                    }
                    words.push(LineToken::Word(body));
                }
                '\\' => {
                    in_word = true;
                    push_token_escape(&mut chars, &mut cur, strict)?;
                }
                '~' if !in_word => {
                    in_word = true;
                    word_tilde = true;
                    cur.push('~');
                }
                _ => {
                    in_word = true;
                    cur.push(c);
                }
            },
        }
    }
    finish_word(&mut words, &mut cur, &mut in_word, &mut word_tilde);
    Ok(words)
}

/// Replace tmux command-template placeholders for one prompt argument.
///
/// `%1` may occur repeatedly. The legacy `%%` spelling replaces only its first
/// occurrence; `%%%` (or `%1%`) additionally quotes characters that tmux's
/// command parser would otherwise treat specially.
pub(crate) fn replace_prompt_template(template: &str, value: &str, index: u8) -> String {
    let chars: Vec<char> = template.chars().collect();
    let mut output = String::with_capacity(template.len() + value.len());
    let mut cursor = 0;
    let mut replaced_legacy = false;

    while cursor < chars.len() {
        if chars[cursor] != '%' || cursor + 1 >= chars.len() {
            output.push(chars[cursor]);
            cursor += 1;
            continue;
        }

        let next = chars[cursor + 1];
        let indexed = index <= 9 && next == char::from(b'0' + index);
        let legacy = next == '%' && !replaced_legacy;
        if !indexed && !legacy {
            output.push('%');
            cursor += 1;
            continue;
        }
        if legacy {
            replaced_legacy = true;
        }
        cursor += 2;

        let quoted = cursor < chars.len() && chars[cursor] == '%';
        if quoted {
            cursor += 1;
        }
        for character in value.chars() {
            if quoted && matches!(character, '"' | '\\' | '$' | ';' | '~') {
                output.push('\\');
            }
            output.push(character);
        }
    }
    output
}

// ---- argument helpers ------------------------------------------------------

/// Value following `flag` in `args` (e.g. the argument to `-t`).
fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

/// Every value following each occurrence of `flag` (e.g. repeatable `-e`).
fn flag_values<'a>(args: &'a [String], flag: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == flag {
            if let Some(v) = args.get(i + 1) {
                out.push(v.as_str());
            }
            i += 2;
        } else {
            i += 1;
        }
    }
    out
}

/// Whether a boolean flag (e.g. `-P`, `-d`) is present.
fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

/// Rewrite a command's argv into a canonical form the flag helpers understand:
/// every short flag becomes its own `-x` token and every flag value becomes a
/// separate following token. This lets a single scanner absorb tmux's getopt
/// surface — clustered booleans (`-ga`), attached values (`-t0` / `-F#{x}`), and
/// separate values (`-t 0`) — so the per-command handlers only ever see the
/// simple `-x [value]` shape. Tokens after `--`, positionals, and args of a
/// command with no modeled `spec` pass through unchanged. Assumes flags already
/// validated by [`unknown_flag`], so an unrecognized letter is passed through
/// rather than erroring again.
fn normalize_argv(name: &str, args: &[String]) -> Vec<String> {
    let spec = match registry::getopt(name) {
        Some(s) => s,
        None => return args.to_vec(),
    };
    let mut out: Vec<String> = Vec::with_capacity(args.len());
    if let Some(word) = args.first() {
        out.push(word.clone());
    }
    let mut i = 1;
    while i < args.len() {
        let a = args[i].as_str();
        if a == "--" {
            // End of options: the rest are operands, verbatim.
            out.extend(args[i..].iter().cloned());
            break;
        }
        let bytes = a.as_bytes();
        if bytes.first() != Some(&b'-') || a.len() < 2 {
            // tmux command options end at the first operand. Everything after
            // the pane command name belongs to that command, including
            // option-looking arguments such as `/bin/sh -c script`.
            out.extend(args[i..].iter().cloned());
            break;
        }
        let cluster = &a[1..];
        let cb = cluster.as_bytes();
        let mut j = 0;
        while j < cb.len() {
            let c = cb[j] as char;
            match registry::flag_kind(spec, c) {
                Some(true) => {
                    out.push(format!("-{c}"));
                    if j + 1 < cb.len() {
                        // Attached value: the rest of the cluster.
                        out.push(cluster[j + 1..].to_string());
                    } else if i + 1 < args.len() {
                        // Separate value: the next argument.
                        out.push(args[i + 1].clone());
                        i += 1;
                    }
                    break;
                }
                // Boolean (or, defensively, an unknown letter): keep as a lone flag.
                _ => {
                    out.push(format!("-{c}"));
                    j += 1;
                }
            }
        }
        i += 1;
    }
    out
}

/// Scan a command's argv (index 0 is the command word) for a flag letter not in
/// its getopt `spec`, returning the first offending letter — tmux's getopt does a
/// left-to-right scan and reports the first unknown flag. Mirrors getopt's
/// handling of clustered short flags (`-ga`), attached values (`-t0`), separate
/// values (`-t 0`), and the `--` end-of-options marker.
fn unknown_flag(args: &[String], spec: &str) -> Option<char> {
    let mut i = 1; // skip the command word
    while i < args.len() {
        let a = args[i].as_str();
        if a == "--" {
            break; // end of options; the rest are operands
        }
        let bytes = a.as_bytes();
        if bytes.first() != Some(&b'-') || a.len() < 2 {
            break; // first operand ends this command's option parsing
        }
        // Walk the short-flag cluster after the leading '-'.
        let cluster = &a[1..];
        let cb = cluster.as_bytes();
        let mut j = 0;
        while j < cb.len() {
            let c = cb[j] as char;
            match registry::flag_kind(spec, c) {
                None => return Some(c),
                Some(false) => j += 1, // boolean flag: keep scanning the cluster
                Some(true) => {
                    // Value flag: any remaining cluster text is its attached value;
                    // otherwise it consumes the next argument.
                    if j + 1 == cb.len() {
                        i += 1;
                    }
                    break;
                }
            }
        }
        i += 1;
    }
    None
}

/// Return the first value-taking flag that has no attached or following value.
///
/// This runs after [`unknown_flag`], so every option letter is known. Like
/// getopt, a following option-looking token is still the value; only reaching
/// the end of argv is an error.
fn missing_flag_value(args: &[String], spec: &str) -> Option<char> {
    let mut i = 1;
    while i < args.len() {
        let argument = args[i].as_str();
        if argument == "--" {
            break;
        }
        if !argument.starts_with('-') || argument == "-" {
            break;
        }
        let cluster = &argument[1..];
        let bytes = cluster.as_bytes();
        let mut j = 0;
        while j < bytes.len() {
            let flag = bytes[j] as char;
            match registry::flag_kind(spec, flag) {
                Some(true) => {
                    if j + 1 == bytes.len() {
                        if i + 1 == args.len() {
                            return Some(flag);
                        }
                        i += 1;
                    }
                    break;
                }
                _ => j += 1,
            }
        }
        i += 1;
    }
    None
}

/// `bind-key` treats everything after its key operand as the command to run,
/// so command flags such as `new-window -d` must not be parsed as bind-key
/// options.
fn unknown_bind_key_flag(args: &[String], spec: &str) -> Option<char> {
    let mut i = 1;
    while i < args.len() {
        let arg = args[i].as_str();
        if !arg.starts_with('-') || arg == "-" {
            return None;
        }
        let cluster = &arg[1..];
        let bytes = cluster.as_bytes();
        let mut j = 0;
        while j < bytes.len() {
            let ch = bytes[j] as char;
            match registry::flag_kind(spec, ch) {
                None => return Some(ch),
                Some(false) => j += 1,
                Some(true) => {
                    if j + 1 == bytes.len() {
                        i += 1;
                    }
                    break;
                }
            }
        }
        i += 1;
    }
    Some('k')
}

/// Whether a single-letter boolean flag `ch` is present, including inside a
/// combined short-flag cluster (tmux's getopt merges `-g -a` into `-ga`). Only
/// pure-letter clusters count, so a value like `-3` or a positional isn't
/// mistaken for a flag group.
pub(crate) fn has_bool_flag(args: &[String], ch: char) -> bool {
    args.iter().any(|a| {
        let bytes = a.as_bytes();
        bytes.first() == Some(&b'-')
            && a.len() >= 2
            && bytes.get(1) != Some(&b'-')
            && a[1..].chars().all(|c| c.is_ascii_alphabetic())
            && a[1..].contains(ch)
    })
}

/// Positional (non-flag) arguments, skipping the command name at index 0 and any
/// `-flag value` pairs whose flag is listed in `value_flags`. Boolean flags like
/// `-d` are dropped without consuming a following argument.
fn positionals<'a>(args: &'a [String], value_flags: &[&str]) -> Vec<&'a str> {
    let mut i = 1; // skip the command name
                   // tmux's `args_parse` stops looking for flags at the first operand, so a
                   // later word that starts with `-` — a menu item named `-disabled`, say —
                   // is an operand too rather than an unknown flag.
    while i < args.len() {
        let arg = args[i].as_str();
        if arg == "--" {
            i += 1;
            break;
        }
        if !arg.starts_with('-') || arg == "-" {
            break;
        }
        if value_flags.contains(&arg) {
            i += 1; // also skip this flag's value
        }
        i += 1;
    }
    args[i.min(args.len())..]
        .iter()
        .map(String::as_str)
        .collect()
}

/// Return a command and all of its arguments after the tmux options. Unlike
/// [`positionals`], option-looking words after the command name are preserved
/// (for example `new-window /bin/sh -c script`).
fn trailing_command<'a>(args: &'a [String], value_flags: &[&str]) -> Vec<&'a str> {
    if args.is_empty() {
        return Vec::new();
    }
    let mut i = 1;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg == "--" {
            i += 1;
            break;
        }
        if !arg.starts_with('-') || arg == "-" {
            break;
        }
        if value_flags.contains(&arg) {
            i += 1;
        }
        i += 1;
    }
    args[i..].iter().map(String::as_str).collect()
}

/// Find an option value only in the tmux-command option prefix. Pane command
/// arguments may reuse the same spelling (most commonly `/bin/sh -c ...`) and
/// must not be mistaken for tmux options.
fn command_option_value<'a>(
    args: &'a [String],
    wanted: &str,
    value_flags: &[&str],
) -> Option<&'a str> {
    let mut found = None;
    let mut i = 1;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg == "--" {
            break;
        }
        if !arg.starts_with('-') || arg == "-" {
            break;
        }
        if value_flags.contains(&arg) {
            if arg == wanted {
                found = args.get(i + 1).map(String::as_str);
            }
            i += 1;
        }
        i += 1;
    }
    found
}

/// Recreate the environment a tmux server gives a newly spawned pane. Wrapping
/// with `env -i` avoids mutating the multithreaded daemon process environment
/// while preserving the complete environment sent in identify frames.
fn pane_argv(
    argv: Vec<String>,
    context: &ClientContext,
    extra_environment: &[&str],
    st: &ServerState,
    session: SpawnSession<'_>,
) -> Vec<String> {
    if context.environment.is_empty() && extra_environment.is_empty() {
        return argv;
    }
    let environment = st.pane_environment(
        session,
        &context.environment,
        // tmux's "only unattached clients": a command client is the one with no
        // session of its own, so its `PATH` is what the pane should run with.
        // hmux's control clients carry the session they attached to, so they
        // take the attached path here as an attached client does.
        matches!(context.kind, ClientKind::Command),
        extra_environment,
    );
    let mut wrapped = Vec::with_capacity(argv.len() + environment.len() + 2);
    wrapped.push("/usr/bin/env".to_string());
    wrapped.push("-i".to_string());
    wrapped.extend(environment);
    wrapped.extend(argv);
    wrapped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> SharedState {
        crate::server::state::shared_state(ServerState::with_test_session().unwrap())
    }

    fn run_str(st: &SharedState, args: &[&str]) -> CommandResult {
        let owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        run(&owned, st, &PaneAgents::new())
    }

    fn run_str_agents(st: &SharedState, agents: &PaneAgents, args: &[&str]) -> CommandResult {
        let owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        run(&owned, st, agents)
    }

    #[test]
    fn capture_style_tracks_full_state_across_physical_rows() {
        let rows = capture_vt_normalize_rows(
            b"\x1b[1;2;3;4:3;5;7;8;9;53;38;5;17;48;2;1;2;3;58;5;4mA\nB",
            2,
        );
        let prefix = concat!(
            "\x1b[1;2;3;5;7;8;9;4:3;5:3m",
            "\x1b[38;5;17m",
            "\x1b[48;2;1;2;3m",
            "\x1b[58;5;4m"
        );
        // The style carries into the second row rather than being closed and
        // reopened, which is what tmux's `lastgc` does across the rows of one
        // capture: the second row differs from the first in nothing, so it
        // opens with nothing.
        assert_eq!(rows[0], format!("{prefix}A"));
        assert_eq!(rows[1], "B");
    }

    #[test]
    fn capture_style_normalizes_selective_resets_and_hyperlinks() {
        let rows = capture_vt_normalize_rows(
            b"\x1b[1;3;31mA\x1b[23mB\x1b]8;id=link;https://example.test\x1b\\C\x1b]8;;\x1b\\",
            1,
        );
        assert_eq!(
            rows[0],
            concat!(
                "\x1b[1;3m\x1b[31mA",
                "\x1b[0;1m\x1b[31mB",
                "\x1b]8;id=link;https://example.test\x1b\\C",
                // The row closes the link it opened, as `grid_string_cells`
                // does, and leaves the style open for whatever follows.
                "\x1b]8;;\x1b\\"
            )
        );
    }

    /// Whether a command's invalidation reached this client's wakeup.
    ///
    /// A zero timeout is exact here, not a shortcut: `run` publishes the
    /// invalidation and writes the wakeup on the calling thread, so the fd is
    /// already readable by the time `run` returns. Waiting could only mask a
    /// wakeup that arrived from somewhere else.
    fn fd_is_readable(fd: std::os::fd::RawFd) -> bool {
        let mut pollfd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        assert!(unsafe { libc::poll(&mut pollfd, 1, 0) } >= 0);
        pollfd.revents & libc::POLLIN != 0
    }

    #[test]
    fn read_only_command_does_not_invalidate_an_attached_client() {
        let st = state();
        let (pane_output, render) = {
            let guard = st.borrow_mut();
            let pane_output = guard.subscribe_active_pane_output("0").unwrap();
            let registry = guard.client_render_registry();
            let session_id = guard.session_id("0").unwrap();
            (
                pane_output,
                registry.attach(session_id, "test-client".into()).unwrap(),
            )
        };
        pane_output.drain();

        let result = run_str(&st, &["display-message", "-p", "-t", "0", "unchanged"]);
        assert_eq!(result.exit, 0);
        assert!(
            !fd_is_readable(pane_output.as_raw_fd()),
            "a read-only command must not impersonate pane output"
        );
        assert!(
            !fd_is_readable(render.as_raw_fd()),
            "a read-only command must not invalidate an attached compositor"
        );
    }

    #[test]
    fn window_status_change_invalidates_only_its_attached_session() {
        let st = state();
        let (registry, session_0, session_1) = {
            let mut guard = st.borrow_mut();
            guard.create_session("1", PaneSpec::Inert).unwrap();
            (
                guard.client_render_registry(),
                guard.session_id("0").unwrap(),
                guard.session_id("1").unwrap(),
            )
        };
        let attached_0 = registry.attach(session_0, "client-0".into()).unwrap();
        let attached_1 = registry.attach(session_1, "client-1".into()).unwrap();

        let result = run_str(&st, &["rename-window", "-t", "0:", "renamed"]);
        assert_eq!(result.exit, 0);
        assert!(
            fd_is_readable(attached_0.as_raw_fd()),
            "the affected session must redraw its status"
        );
        assert!(
            !fd_is_readable(attached_1.as_raw_fd()),
            "a mutation in another session must not wake this compositor"
        );
        assert!(attached_0
            .take()
            .contains(super::super::state::RenderInvalidation::STATUS));
    }

    #[test]
    fn status_interval_change_wakes_attached_clients_to_reschedule_timer() {
        let st = state();
        let attached = {
            let guard = st.borrow_mut();
            let registry = guard.client_render_registry();
            registry
                .attach(guard.session_id("0").unwrap(), "test-client".into())
                .unwrap()
        };

        let result = run_str(&st, &["set-option", "-g", "status-interval", "1"]);
        assert_eq!(result.exit, 0);
        assert!(
            fd_is_readable(attached.as_raw_fd()),
            "an idle attached client must wake to adopt the new interval"
        );
        assert!(attached
            .take()
            .contains(super::super::state::RenderInvalidation::STATUS));
    }

    #[test]
    fn terminal_override_change_wakes_clients_to_rebuild_their_profiles() {
        let st = state();
        let attached = {
            let guard = st.borrow_mut();
            let registry = guard.client_render_registry();
            registry
                .attach(guard.session_id("0").unwrap(), "test-client".into())
                .unwrap()
        };

        let result = run_str(
            &st,
            &["set-option", "-g", "terminal-overrides[7]", "screen*:am@"],
        );
        assert_eq!(result.exit, 0);
        assert!(fd_is_readable(attached.as_raw_fd()));
        let invalidation = attached.take();
        assert!(invalidation.contains(super::super::state::RenderInvalidation::TERMINAL));
        assert!(invalidation.contains(super::super::state::RenderInvalidation::STATUS));
    }

    #[test]
    fn command_prompt_flags_normalize_target_forms() {
        let separate = ["command-prompt", "-k", "-t", "/dev/pts/7", "show"];
        let separate: Vec<String> = separate.iter().map(|arg| (*arg).to_string()).collect();
        assert_eq!(
            command_prompt_target(&separate),
            Some("/dev/pts/7".to_string())
        );

        let clustered = ["command-prompt", "-kt/dev/pts/8", "show"];
        let clustered: Vec<String> = clustered.iter().map(|arg| (*arg).to_string()).collect();
        assert_eq!(
            command_prompt_target(&clustered),
            Some("/dev/pts/8".to_string())
        );

        let positional = ["command-prompt", "-k", "--", "-target-looking-template"];
        let positional: Vec<String> = positional.iter().map(|arg| (*arg).to_string()).collect();
        assert_eq!(command_prompt_target(&positional), None);
    }

    #[test]
    fn key_prompt_template_replacement_matches_tmux_rules() {
        assert_eq!(
            replace_prompt_template("display '%%' '%%' '%1' '%1'", "Up", 1),
            "display 'Up' '%%' 'Up' 'Up'"
        );
        assert_eq!(
            replace_prompt_template("display '%%%'", "\"\\$;~", 1),
            r#"display '\"\\\$\;\~'"#
        );
        assert_eq!(
            replace_prompt_template("display '%2'", "Up", 1),
            "display '%2'"
        );
    }

    #[test]
    fn command_line_tokenizer_distinguishes_literal_and_syntax_semicolons() {
        assert_eq!(
            tokenize_line("display-message -pl ';'; set-buffer \\;"),
            vec![
                LineToken::Word("display-message".to_string()),
                LineToken::Word("-pl".to_string()),
                LineToken::Word(";".to_string()),
                LineToken::Separator,
                LineToken::Word("set-buffer".to_string()),
                LineToken::Word(";".to_string()),
            ]
        );
    }

    #[test]
    fn list_panes_exposes_agent_format_vars() {
        use crate::integration::status::AgentStatus;
        use crate::integration::AgentState;

        let st = state();
        let mut agents = PaneAgents::new();
        agents.insert(
            PaneId(0),
            AgentStatus {
                agent: "claude",
                pid: Some(4242),
                session_id: Some("sess-4242".to_string()),
                model: Some("claude-fable-5".to_string()),
                state: AgentState::Working,
            },
        );
        let r = run_str_agents(
            &st,
            &agents,
            &[
                "list-panes",
                "-a",
                "-F",
                "#{pane_id} #{pane_agent} #{pane_agent_state} #{pane_state_emoji} #{pane_agent_pid} #{pane_agent_session_id} #{pane_agent_model}",
            ],
        );
        assert_eq!(r.exit, 0);
        assert_eq!(
            r.stdout, "%0 claude working 🔄 4242 sess-4242 claude-fable-5\n",
            "got {:?}",
            r.stdout
        );
    }

    /// Only the agent metadata empties out for a pane with no agent. The state
    /// emoji still reports one, because it falls back to classifying whatever
    /// the pane is actually running — here a fixture pane with no child, which
    /// is exactly the "no live process" case.
    #[test]
    fn list_panes_without_agent_reports_none_but_still_a_state_emoji() {
        let st = state();
        let r = run_str(
            &st,
            &[
                "list-panes",
                "-a",
                "-F",
                "#{pane_agent}|#{pane_agent_state}|#{pane_state_emoji}|#{pane_agent_pid}|#{pane_agent_model}",
            ],
        );
        assert_eq!(r.exit, 0);
        assert_eq!(r.stdout, "|none|🛑||\n", "got {:?}", r.stdout);
    }

    #[test]
    fn list_sessions_reports_default() {
        let st = state();
        let r = run_str(&st, &["list-sessions"]);
        assert_eq!(r.exit, 0);
        assert!(
            r.stdout.starts_with("0: 1 windows (created "),
            "got {:?}",
            r.stdout
        );
    }

    #[test]
    fn key_table_bind_list_and_unbind_round_trip() {
        let st = state();
        assert_eq!(
            run_str(
                &st,
                &["bind-key", "-r", "-T", "foo", "F1", "new-window", "-d"],
            )
            .exit,
            0
        );
        assert_eq!(
            run_str(
                &st,
                &["bind-key", "-T", "foo", "F2", "display-message", "two"],
            )
            .exit,
            0
        );
        let listed = run_str(&st, &["list-keys", "-T", "foo"]);
        assert_eq!(
            listed.stdout,
            "bind-key -r -T foo F1 new-window -d\nbind-key -T foo F2 display-message two\n",
            "got {:?}",
            listed.stdout.clone()
        );
        assert_eq!(run_str(&st, &["unbind-key", "-T", "foo", "F1"]).exit, 0);
        assert_eq!(
            run_str(&st, &["list-keys", "-q", "-T", "foo", "F1"]).stdout,
            ""
        );
    }

    #[test]
    fn unbind_key_validates_all_key_and_explicit_table_forms() {
        let st = state();

        let key_with_all = run_str(&st, &["unbind-key", "-a", "F1"]);
        assert_eq!(key_with_all.exit, 1);
        assert_eq!(key_with_all.stderr, "key given with -a\n");

        let missing_key = run_str(&st, &["unbind-key"]);
        assert_eq!(missing_key.exit, 1);
        assert_eq!(missing_key.stderr, "missing key\n");

        for args in [
            &["unbind-key", "-a", "-T", "absent-table"][..],
            &["unbind-key", "-T", "absent-table", "F1"][..],
        ] {
            let missing_table = run_str(&st, args);
            assert_eq!(missing_table.exit, 1);
            assert_eq!(missing_table.stderr, "table absent-table doesn't exist\n");
        }
    }

    #[test]
    fn unbind_key_quiet_suppresses_validation_failures() {
        let st = state();
        for args in [
            &["unbind-key", "-q"][..],
            &["unbind-key", "-q", "DefinitelyNotAKey"][..],
            &["unbind-key", "-q", "-a", "-T", "absent-table"][..],
            &["unbind-key", "-q", "-T", "absent-table", "F1"][..],
        ] {
            let result = run_str(&st, args);
            assert_eq!(result.exit, 0);
            assert_eq!(result.stderr, "");
        }
    }

    #[test]
    fn list_keys_rejects_a_missing_table() {
        let st = state();
        let result = run_str(&st, &["list-keys", "-T", "nope"]);
        assert_eq!(result.exit, 1);
        assert_eq!(result.stderr, "table nope doesn't exist\n");
    }

    #[test]
    fn list_keys_single_builtin_root_binding_is_silent_without_client() {
        let st = state();
        let result = run_str(&st, &["list-keys", "-T", "root", "MouseDown1Pane"]);
        assert_eq!(result.exit, 0);
        assert_eq!(result.stdout, "");
    }

    #[test]
    fn list_keys_single_prefix_binding_is_silent_without_client() {
        let st = state();
        let result = run_str(&st, &["list-keys", "-T", "prefix", "C-b"]);
        assert_eq!(result.exit, 0);
        assert_eq!(result.stdout, "");

        assert_eq!(
            run_str(&st, &["bind-key", "-T", "prefix", "C-b", "display-message"]).exit,
            0
        );
        let overridden = run_str(&st, &["list-keys", "-T", "prefix", "C-b"]);
        assert_eq!(overridden.stdout, "");
    }

    #[test]
    fn grouped_session_shares_links_with_independent_selection() {
        let st = state();
        assert_eq!(run_str(&st, &["new-window", "-t", "0:"]).exit, 0);
        let result = run_str(&st, &["new-session", "-d", "-s", "g", "-t", "0"]);
        assert_eq!(result.exit, 0, "{}", result.stderr);

        assert_eq!(run_str(&st, &["new-window", "-d", "-t", "0:"]).exit, 0);
        let grouped = run_str(&st, &["list-windows", "-t", "g", "-F", "#{window_index}"]);
        assert_eq!(grouped.stdout, "0\n1\n2\n");

        assert_eq!(run_str(&st, &["select-window", "-t", "0:2"]).exit, 0);
        // `display-message` takes a pane target, so the session has to be named
        // with a `:`; a bare `0` would be a pane index in the current session.
        let work = run_str(
            &st,
            &["display-message", "-t", "0:", "-p", "#{window_index}"],
        );
        let mirror = run_str(
            &st,
            &["display-message", "-t", "g:", "-p", "#{window_index}"],
        );
        assert_eq!(work.stdout, "2\n");
        assert_eq!(mirror.stdout, "0\n");
    }

    #[test]
    fn list_sessions_format() {
        let st = state();
        run_str(&st, &["new-session", "-d", "-s", "foo"]);
        let r = run_str(
            &st,
            &["list-sessions", "-F", "#{session_name}:#{session_windows}"],
        );
        assert_eq!(r.exit, 0);
        assert_eq!(r.stdout, "0:1\nfoo:1\n", "got {:?}", r.stdout);
    }

    #[test]
    fn capture_pane_shows_visible_screen_after_scroll() {
        // Regression for the "long output corrupts the screen" bug (see
        // report.md): `capture-pane -p` must print the visible viewport, not the
        // oldest top of scrollback history.
        let st = state();
        {
            let mut g = st.borrow_mut();
            let _ = g.resize_session("0", 80, 24);
            let mut feed = b"HEAD_OLDEST\r\n".to_vec();
            for i in 1..=60 {
                feed.extend_from_slice(format!("filler{i}\r\n").as_bytes());
            }
            feed.extend_from_slice(b"TAIL_NEWEST");
            g.active_pane("0").unwrap().feed(&feed);
            assert!(
                g.pane_scrollback_rows("0").unwrap() > 0,
                "precondition: the pane must have scrolled"
            );
        }
        let r = run_str(&st, &["capture-pane", "-p", "-t", "0"]);
        assert_eq!(r.exit, 0);
        assert!(
            r.stdout.contains("TAIL_NEWEST"),
            "visible tail must be captured, got:\n{}",
            r.stdout
        );
        assert!(
            !r.stdout.contains("HEAD_OLDEST"),
            "scrolled-off history must not be captured, got:\n{}",
            r.stdout
        );
    }

    #[test]
    fn capture_pane_without_p_stores_an_auto_named_buffer() {
        // Without -p the capture lands in a paste buffer (auto-named `buffer0`
        // on a fresh server), not on stdout.
        let st = state();
        {
            let g = st.borrow_mut();
            g.active_pane("0").unwrap().feed(b"CAPTURED_LINE");
        }
        let cap = run_str(&st, &["capture-pane", "-t", "0"]);
        assert_eq!(cap.exit, 0);
        assert_eq!(cap.stdout, "", "no -p: nothing goes to stdout");
        let list = run_str(&st, &["list-buffers", "-F", "#{buffer_name}"]);
        assert_eq!(list.stdout, "buffer0\n", "got {:?}", list.stdout);
        // The captured contents are readable back from the buffer.
        let show = run_str(&st, &["show-buffer", "-b", "buffer0"]);
        assert!(
            show.stdout.contains("CAPTURED_LINE"),
            "buffer holds the capture, got:\n{}",
            show.stdout
        );
    }

    #[test]
    fn capture_pane_b_names_the_target_buffer() {
        // -b NAME captures into a buffer with that explicit name.
        let st = state();
        let cap = run_str(&st, &["capture-pane", "-b", "cap", "-t", "0"]);
        assert_eq!(cap.exit, 0);
        assert_eq!(cap.stdout, "");
        let list = run_str(&st, &["list-buffers", "-F", "#{buffer_name}"]);
        assert_eq!(list.stdout, "cap\n", "got {:?}", list.stdout);
    }

    #[test]
    fn has_session_hit_and_miss() {
        let st = state();
        assert_eq!(run_str(&st, &["has-session", "-t", "0"]).exit, 0);
        let miss = run_str(&st, &["has-session", "-t", "nope"]);
        assert_eq!(miss.exit, 1);
        assert!(miss.stderr.contains("can't find session"));
    }

    #[test]
    fn has_session_no_target_resolves_current() {
        let st = state();
        // With no -t, tmux resolves the current session → exit 0.
        assert_eq!(run_str(&st, &["has-session"]).exit, 0);
    }

    #[test]
    fn new_session_print_default_template() {
        let st = state();
        let r = run_str(&st, &["new-session", "-d", "-s", "foo", "-P"]);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        assert_eq!(r.stdout, "foo:\n", "got {:?}", r.stdout);
    }

    #[test]
    fn new_session_print_custom_format() {
        let st = state();
        let r = run_str(
            &st,
            &[
                "new-session",
                "-d",
                "-s",
                "foo",
                "-P",
                "-F",
                "#{session_name}",
            ],
        );
        assert_eq!(r.stdout, "foo\n", "got {:?}", r.stdout);
    }

    #[test]
    fn new_session_rejects_invalid_dimensions_before_creation() {
        let st = state();
        for (name, flag, value, expected) in [
            ("bad-width", "-x", "invalid", "width invalid\n"),
            ("bad-height", "-y", "invalid", "height invalid\n"),
            ("small-width", "-x", "0", "width too small\n"),
            ("small-height", "-y", "0", "height too small\n"),
            ("large-width", "-x", "65536", "width too large\n"),
            ("large-height", "-y", "65536", "height too large\n"),
        ] {
            let result = run_str(&st, &["new-session", "-d", "-s", name, flag, value]);
            assert_eq!(result.exit, 1, "name={name}");
            assert_eq!(result.stderr, expected, "name={name}");
            assert_eq!(run_str(&st, &["has-session", "-t", name]).exit, 1);
        }
    }

    #[test]
    fn new_session_caps_large_valid_dimensions() {
        let st = state();
        let result = run_str(
            &st,
            &[
                "new-session",
                "-d",
                "-s",
                "large",
                "-x",
                "65535",
                "-y",
                "65535",
            ],
        );
        assert_eq!(result.exit, 0, "stderr={:?}", result.stderr);
        let size = run_str(
            &st,
            &[
                "display-message",
                "-p",
                "-t",
                "large",
                "#{window_width}x#{window_height}",
            ],
        );
        assert_eq!(size.stdout, "10000x10000\n");
    }

    #[test]
    fn unknown_command_errors() {
        let st = state();
        let r = run_str(&st, &["frobnicate"]);
        assert_eq!(r.exit, 1);
        assert!(r.stderr.contains("unknown command"));
    }

    #[test]
    fn rename_session_renames() {
        let st = state();
        let r = run_str(&st, &["rename-session", "-t", "0", "work"]);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        assert_eq!(run_str(&st, &["has-session", "-t", "work"]).exit, 0);
        assert_eq!(run_str(&st, &["has-session", "-t", "0"]).exit, 1);
    }

    #[test]
    fn rename_missing_session_errors() {
        let st = state();
        let r = run_str(&st, &["rename-session", "-t", "nope", "x"]);
        assert_eq!(r.exit, 1);
        assert!(
            r.stderr.contains("can't find session"),
            "got {:?}",
            r.stderr
        );
    }

    #[test]
    fn new_window_targets_session() {
        let st = state();
        let r = run_str(&st, &["new-window", "-t", "0:"]);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        let ls = run_str(&st, &["list-sessions"]);
        assert!(ls.stdout.contains("0: 2 windows"), "got {:?}", ls.stdout);
    }

    #[test]
    fn new_window_print_template() {
        let st = state();
        let r = run_str(&st, &["new-window", "-t", "0:", "-P"]);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        // Default window base index 0 → the new (second) window is index 1.
        assert_eq!(r.stdout, "0:1.0\n", "got {:?}", r.stdout);
    }

    #[test]
    fn list_windows_format() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:"]);
        let r = run_str(
            &st,
            &[
                "list-windows",
                "-t",
                "0",
                "-F",
                "#{window_index}:#{window_panes}",
            ],
        );
        assert_eq!(r.exit, 0);
        assert_eq!(r.stdout, "0:1\n1:1\n", "got {:?}", r.stdout);
    }

    #[test]
    fn list_panes_format() {
        let st = state();
        let r = run_str(
            &st,
            &[
                "list-panes",
                "-t",
                "0",
                "-F",
                "#{pane_index}:#{pane_active}",
            ],
        );
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        assert_eq!(r.stdout, "0:1\n", "got {:?}", r.stdout);
    }

    #[test]
    fn list_clients_empty_ok() {
        let st = state();
        let r = run_str(&st, &["list-clients"]);
        assert_eq!(r.exit, 0);
        assert_eq!(r.stdout, "");
    }

    #[test]
    fn display_message_expands_format() {
        let st = state();
        let r = run_str(
            &st,
            &[
                "display-message",
                "-t",
                "0",
                "-p",
                "S=#{session_name} W=#{window_index}",
            ],
        );
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        assert_eq!(r.stdout, "S=0 W=0\n", "got {:?}", r.stdout);
    }

    #[test]
    fn display_message_expands_default_template_time() {
        let st = state();
        let r = run_str(&st, &["display-message", "-t", "0", "-p"]);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        assert!(r.stdout.contains("current pane 0"), "got {:?}", r.stdout);
        assert!(!r.stdout.contains("%H"), "got {:?}", r.stdout);
        assert!(!r.stdout.contains("%d"), "got {:?}", r.stdout);
    }

    #[test]
    fn display_message_routes_to_control_clients() {
        let st = state();
        let (session_id, registry) = {
            let st = st.borrow_mut();
            (st.sessions()[0].id, st.client_render_registry())
        };
        let attachment = registry
            .attach_with_details(
                session_id,
                "/dev/pts/99".to_string(),
                String::new(),
                None,
                80,
                24,
                0,
                Default::default(),
                true,
            )
            .unwrap();
        let own_context = ClientContext {
            tty_name: Some("/dev/pts/99".to_string()),
            ..ClientContext::default()
        };
        let args = ["display-message", "own-message"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let own = run_with_context(&args, &st, &PaneAgents::new(), &own_context);
        assert_eq!(own.exit, 0, "stderr={:?}", own.stderr);
        assert_eq!(own.stdout, "%message own-message\n");

        let args = ["display-message", "-c", "/dev/pts/99", "external-message"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let external = run_with_context(&args, &st, &PaneAgents::new(), &ClientContext::default());
        assert_eq!(external.exit, 0, "stderr={:?}", external.stderr);
        assert_eq!(external.stdout, "");
        let messages = attachment.take_messages();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].text, "external-message");
        assert_eq!(messages[0].duration_ms, 750);
    }

    #[test]
    fn display_message_verbose_prints_trace_with_and_without_result() {
        let st = state();
        let printed = run_str(&st, &["display-message", "-p", "-v", "hi"]);
        assert_eq!(printed.exit, 0, "stderr={:?}", printed.stderr);
        assert_eq!(
            printed.stdout,
            "# expanding format: hi\n# result is: hi\nhi\n"
        );

        let trace_only = run_str(&st, &["display-message", "-v", "hi"]);
        assert_eq!(trace_only.exit, 0, "stderr={:?}", trace_only.stderr);
        assert_eq!(
            trace_only.stdout,
            "# expanding format: hi\n# result is: hi\n"
        );
    }

    #[test]
    fn display_message_missing_session_uses_empty_format_context() {
        let st = state();
        let r = run_str(&st, &["display-message", "-t", "nope", "-p", "x"]);
        assert_eq!(r.exit, 0);
        assert_eq!(r.stdout, "x\n");
        assert_eq!(r.stderr, "");

        let format = run_str(
            &st,
            &["display-message", "-t", "nope", "-p", "#{pane_index}"],
        );
        assert_eq!(format.exit, 0);
        assert_eq!(format.stdout, "\n");
        assert_eq!(format.stderr, "");
    }

    #[test]
    fn rename_window_renames() {
        let st = state();
        let r = run_str(&st, &["rename-window", "-t", "0", "renamed"]);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_name}"]);
        assert_eq!(lw.stdout, "renamed\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn attach_reports_not_implemented() {
        let st = state();
        let r = run_str(&st, &["attach-session", "-t", "0"]);
        assert_eq!(r.exit, 1);
        assert!(
            r.stderr.contains("not a terminal") || r.stderr.contains("open terminal"),
            "got {:?}",
            r.stderr
        );
    }

    #[test]
    fn attach_missing_session() {
        let st = state();
        let r = run_str(&st, &["attach-session", "-t", "nope"]);
        assert_eq!(r.exit, 1);
        assert!(
            r.stderr.contains("can't find session"),
            "got {:?}",
            r.stderr
        );
    }

    // ---- format engine: shorthands, conditionals, comparisons ----

    #[test]
    fn display_shorthands() {
        let st = state();
        let r = run_str(&st, &["display-message", "-t", "0", "-p", "#S:#I.#P"]);
        assert_eq!(r.stdout, "0:0.0\n", "got {:?}", r.stdout);
    }

    #[test]
    fn display_conditional_true_and_false() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:"]); // window 1 becomes active
        let active = run_str(
            &st,
            &[
                "display-message",
                "-t",
                "0:1",
                "-p",
                "#{?window_active,A,I}",
            ],
        );
        assert_eq!(active.stdout, "A\n", "got {:?}", active.stdout);
        let inactive = run_str(
            &st,
            &[
                "display-message",
                "-t",
                "0:0",
                "-p",
                "#{?window_active,A,I}",
            ],
        );
        assert_eq!(inactive.stdout, "I\n", "got {:?}", inactive.stdout);
    }

    #[test]
    fn display_conditional_bare_number_is_false() {
        let st = state();
        // tmux treats `1` as an (unset) variable name → false branch.
        let r = run_str(&st, &["display-message", "-t", "0", "-p", "#{?1,T,F}"]);
        assert_eq!(r.stdout, "F\n", "got {:?}", r.stdout);
    }

    #[test]
    fn display_comparisons() {
        let st = state();
        let eq = run_str(
            &st,
            &[
                "display-message",
                "-t",
                "0",
                "-p",
                "#{==:#{session_name},0}",
            ],
        );
        assert_eq!(eq.stdout, "1\n", "got {:?}", eq.stdout);
        let ne = run_str(&st, &["display-message", "-t", "0", "-p", "#{!=:a,b}"]);
        assert_eq!(ne.stdout, "1\n", "got {:?}", ne.stdout);
    }

    // ---- if-shell -F : condition is a format, branch on truthiness ----

    #[test]
    fn if_shell_format_truthy_runs_then() {
        let st = state();
        let r = run_str(&st, &["if-shell", "-F", "1", "new-window -t 0:"]);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(lw.stdout, "0\n1\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn if_shell_format_empty_skips_then() {
        let st = state();
        let r = run_str(&st, &["if-shell", "-F", "", "new-window -t 0:"]);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(lw.stdout, "0\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn if_shell_format_zero_runs_else() {
        let st = state();
        // "0" is falsey as a format, so the else-branch runs (creates the window).
        let r = run_str(
            &st,
            &["if-shell", "-F", "0", "kill-server", "new-window -t 0:"],
        );
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(lw.stdout, "0\n1\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn if_shell_format_expands_before_testing_truthiness() {
        let st = state();
        // `#{session_windows}` expands to "1" for the single bootstrap window — a
        // truthy value → then-branch runs. (Proves the condition is expanded as a
        // format, not run as a shell command.)
        let r = run_str(
            &st,
            &["if-shell", "-F", "#{session_windows}", "new-window -t 0:"],
        );
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(lw.stdout, "0\n1\n", "got {:?}", lw.stdout);
    }

    // ---- source-file ----

    #[test]
    fn source_file_runs_file_contents() {
        let st = state();
        let path = std::env::temp_dir().join(format!("hmux_src_{}.conf", std::process::id()));
        std::fs::write(
            &path,
            "# a comment\n\nset-option -g @sf hello\nnew-window -t 0:\n",
        )
        .expect("write temp conf");
        let r = run_str(&st, &["source-file", path.to_str().unwrap()]);
        let _ = std::fs::remove_file(&path);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        // The sourced `set-option` took effect...
        let opt = run_str(&st, &["show-options", "-g", "-v", "@sf"]);
        assert_eq!(opt.stdout, "hello\n");
        // ...and so did the sourced `new-window`.
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(lw.stdout, "0\n1\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn source_file_missing_reports_open_error() {
        let st = state();
        let r = run_str(&st, &["source-file", "/no/such/hmux/source/file"]);
        assert_eq!(r.exit, 1);
        assert_eq!(r.stdout, "");
        assert_eq!(
            r.stderr,
            "No such file or directory: /no/such/hmux/source/file\n"
        );
    }

    #[test]
    fn source_file_quiet_missing_is_silent_success() {
        let st = state();
        let r = run_str(&st, &["source-file", "-q", "/no/such/hmux/source/file"]);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        assert_eq!(r.stdout, "");
        assert_eq!(r.stderr, "");
    }

    #[test]
    fn source_file_format_expands_path_before_opening() {
        let st = state();
        let r = run_str(
            &st,
            &["source-file", "-F", "/no/such/hmux/#{session_name}/file"],
        );
        assert_eq!(r.exit, 1);
        assert_eq!(r.stdout, "");
        assert_eq!(
            r.stderr,
            "No such file or directory: /no/such/hmux/0/file\n"
        );
    }

    #[test]
    fn source_file_parse_only_validates_without_mutating() {
        let st = state();
        let path =
            std::env::temp_dir().join(format!("hmux_src_parse_only_{}.conf", std::process::id()));
        std::fs::write(
            &path,
            "set-option -g @parse_only touched\nnew-window -d -t 0:\n",
        )
        .expect("write parse-only config");
        let r = run_str(
            &st,
            &["source-file", "-n", path.to_str().expect("utf8 path")],
        );
        let _ = std::fs::remove_file(&path);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        assert_eq!(
            run_str(&st, &["show-options", "-qgv", "@parse_only"]).stdout,
            ""
        );
        assert_eq!(
            run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]).stdout,
            "0\n"
        );
    }

    #[test]
    fn source_file_parse_only_reports_arity_before_any_mutation() {
        let st = state();
        let path =
            std::env::temp_dir().join(format!("hmux_src_parse_error_{}.conf", std::process::id()));
        std::fs::write(
            &path,
            "set-option -g @before_error touched\nrename-session\n",
        )
        .expect("write invalid parse-only config");
        let r = run_str(
            &st,
            &["source-file", "-n", path.to_str().expect("utf8 path")],
        );
        assert_eq!(r.exit, 1);
        assert_eq!(
            r.stdout,
            format!(
                "{}:2: command rename-session: too few arguments (need at least 1)\n",
                path.display()
            )
        );
        assert_eq!(r.stderr, "");
        assert_eq!(
            run_str(&st, &["show-options", "-qgv", "@before_error"]).stdout,
            ""
        );
        let _ = std::fs::remove_file(&path);
    }

    // ---- window lifecycle ----

    #[test]
    fn new_window_explicit_index() {
        let st = state();
        let r = run_str(&st, &["new-window", "-t", "0:5"]);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(lw.stdout, "0\n5\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn new_window_index_in_use() {
        let st = state();
        let r = run_str(&st, &["new-window", "-t", "0:0"]);
        assert_eq!(r.exit, 1);
        assert!(r.stderr.contains("index 0 in use"), "got {:?}", r.stderr);
    }

    #[test]
    fn new_window_kill_replaces_occupied_index() {
        let st = state();
        run_str(&st, &["rename-window", "-t", "0:0", "occupied"]);
        let r = run_str(&st, &["new-window", "-k", "-t", "0:0", "-n", "replacement"]);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        let lw = run_str(
            &st,
            &[
                "list-windows",
                "-t",
                "0",
                "-F",
                "#{window_index}:#{window_name}",
            ],
        );
        assert_eq!(lw.stdout, "0:replacement\n");
    }

    #[test]
    fn new_window_after_inserts_and_shuffles() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:"]); // 0,1
        run_str(&st, &["new-window", "-t", "0:"]); // 0,1,2
                                                   // -a -t 0:0 → desired index 1, occupied 1,2 (no gap) → shift up → 0,1,2,3.
        let r = run_str(
            &st,
            &[
                "new-window",
                "-a",
                "-t",
                "0:0",
                "-P",
                "-F",
                "#{window_index}",
            ],
        );
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        assert_eq!(r.stdout, "1\n", "inserted index");
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(lw.stdout, "0\n1\n2\n3\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn new_window_after_fills_first_gap_without_shuffle() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:"]); // 0,1
        run_str(&st, &["new-window", "-t", "0:"]); // 0,1,2
        run_str(&st, &["kill-window", "-t", "0:1"]); // 0,2
                                                     // -a -t 0:0 → desired index 1 is free → no shuffle → 0,1,2.
        let r = run_str(&st, &["new-window", "-a", "-t", "0:0"]);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(lw.stdout, "0\n1\n2\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn new_window_before_inserts_at_anchor_index() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:"]); // 0,1
        run_str(&st, &["new-window", "-t", "0:"]); // 0,1,2
                                                   // -b -t 0:1 → desired index 1 (the anchor's own) → shift 1,2 up → 0,1,2,3.
        let r = run_str(
            &st,
            &[
                "new-window",
                "-b",
                "-t",
                "0:1",
                "-P",
                "-F",
                "#{window_index}",
            ],
        );
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        assert_eq!(r.stdout, "1\n", "inserted at anchor index");
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(lw.stdout, "0\n1\n2\n3\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn new_window_after_anchors_on_active_when_no_target() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:"]); // 0,1 (active 1)
        run_str(&st, &["new-window", "-t", "0:"]); // 0,1,2 (active 2)
                                                   // -a with no -t → anchor the active window (index 2) → new at 3.
        let r = run_str(&st, &["new-window", "-a"]);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(lw.stdout, "0\n1\n2\n3\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn new_window_after_becomes_active() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:"]); // 0,1
        run_str(&st, &["new-window", "-t", "0:"]); // 0,1,2
        run_str(&st, &["new-window", "-a", "-t", "0:0"]); // new at 1, becomes active
        let d = run_str(
            &st,
            &["display-message", "-t", "0", "-p", "#{window_index}"],
        );
        assert_eq!(d.stdout, "1\n", "active window after -a insert");
    }

    #[test]
    fn new_window_after_missing_index_creates_there() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:"]); // 0,1
                                                   // -a -t 0:5 where index 5 doesn't exist → tmux ignores -a, creates at 5.
        let r = run_str(
            &st,
            &[
                "new-window",
                "-a",
                "-t",
                "0:5",
                "-P",
                "-F",
                "#{window_index}",
            ],
        );
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        assert_eq!(r.stdout, "5\n", "explicit index create");
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(lw.stdout, "0\n1\n5\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn kill_window_leaves_sparse_indices() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:"]); // index 1
        run_str(&st, &["new-window", "-t", "0:"]); // index 2
        let k = run_str(&st, &["kill-window", "-t", "0:1"]);
        assert_eq!(k.exit, 0, "stderr={:?}", k.stderr);
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(lw.stdout, "0\n2\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn kill_window_missing() {
        let st = state();
        let r = run_str(&st, &["kill-window", "-t", "0:99"]);
        assert_eq!(r.exit, 1);
        assert!(
            r.stderr.contains("can't find window: 99"),
            "got {:?}",
            r.stderr
        );
        let r2 = run_str(&st, &["kill-window", "-t", "nope:0"]);
        assert!(
            r2.stderr.contains("can't find session: nope"),
            "got {:?}",
            r2.stderr
        );
    }

    #[test]
    fn select_window_changes_active() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:"]); // active = window 1
        let s = run_str(&st, &["select-window", "-t", "0:0"]);
        assert_eq!(s.exit, 0, "stderr={:?}", s.stderr);
        let lw = run_str(
            &st,
            &[
                "list-windows",
                "-t",
                "0",
                "-F",
                "#{window_index}:#{window_active}",
            ],
        );
        assert_eq!(lw.stdout, "0:1\n1:0\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn navigation_single_window_errors() {
        let st = state();
        assert!(run_str(&st, &["next-window", "-t", "0"])
            .stderr
            .contains("no next window"));
        assert!(run_str(&st, &["previous-window", "-t", "0"])
            .stderr
            .contains("no previous window"));
        assert!(run_str(&st, &["last-window", "-t", "0"])
            .stderr
            .contains("no last window"));
    }

    #[test]
    fn next_and_last_window_move_active() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:"]); // windows 0,1 active=1
        run_str(&st, &["select-window", "-t", "0:0"]); // active=0, last=1
        run_str(&st, &["next-window", "-t", "0"]); // active=1
        let cur = run_str(
            &st,
            &["display-message", "-t", "0", "-p", "#{window_index}"],
        );
        assert_eq!(cur.stdout, "1\n", "got {:?}", cur.stdout);
        run_str(&st, &["last-window", "-t", "0"]); // back to 0
        let cur = run_str(
            &st,
            &["display-message", "-t", "0", "-p", "#{window_index}"],
        );
        assert_eq!(cur.stdout, "0\n", "got {:?}", cur.stdout);
    }

    #[test]
    fn next_previous_window_alert_flag_finds_none() {
        // `-a` steps to the next/previous *alerting* window; native models no
        // alert state, so even with a window to step to, the search fails like
        // tmux with `no next/previous window` (exit 1) instead of stepping.
        let st = state();
        run_str(&st, &["new-window", "-t", "0:1"]); // windows 0,1 active=1
        run_str(&st, &["select-window", "-t", "0:0"]); // active=0

        let n = run_str(&st, &["next-window", "-a", "-t", "0"]);
        assert_eq!(n.exit, 1, "stdout={:?} stderr={:?}", n.stdout, n.stderr);
        assert_eq!(n.stderr, "no next window\n");

        let p = run_str(&st, &["previous-window", "-a", "-t", "0"]);
        assert_eq!(p.exit, 1, "stdout={:?} stderr={:?}", p.stdout, p.stderr);
        assert_eq!(p.stderr, "no previous window\n");

        // The active window did not move (no plain step happened).
        let cur = run_str(
            &st,
            &["display-message", "-t", "0", "-p", "#{window_index}"],
        );
        assert_eq!(cur.stdout, "0\n", "got {:?}", cur.stdout);
    }

    #[test]
    fn window_flags_current_last_empty() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:"]); // index 1
        run_str(&st, &["new-window", "-t", "0:"]); // index 2, active; last=1
        run_str(&st, &["select-window", "-t", "0:1"]); // active=1, last=2
        let lw = run_str(
            &st,
            &[
                "list-windows",
                "-t",
                "0",
                "-F",
                "#{window_index}:#{window_flags}",
            ],
        );
        assert_eq!(lw.stdout, "0:\n1:*\n2:-\n", "got {:?}", lw.stdout);
    }

    // ---- send-keys / kill-server ----

    #[test]
    fn send_keys_hit_and_miss() {
        let st = state();
        assert_eq!(run_str(&st, &["send-keys", "-t", "0", "Enter"]).exit, 0);
        let miss = run_str(&st, &["send-keys", "-t", "nope", "Enter"]);
        assert_eq!(miss.exit, 1);
        assert!(
            miss.stderr.contains("can't find pane: nope"),
            "got {:?}",
            miss.stderr
        );
    }

    #[test]
    fn kill_server_clears_sessions() {
        let st = state();
        let r = run_str(&st, &["kill-server"]);
        assert_eq!(r.exit, 0);
        // After kill-server the session tree is empty.
        assert_eq!(run_str(&st, &["list-sessions"]).stdout, "");
    }

    #[test]
    fn kill_session_clear_alerts_keeps_session() {
        let st = state();
        run_str(&st, &["new-session", "-d", "-s", "b"]);
        // `-C` clears alerts and must NOT destroy the target session.
        let r = run_str(&st, &["kill-session", "-C", "-t", "0"]);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        assert_eq!(r.stdout, "");
        assert_eq!(r.stderr, "");
        // Both sessions survive.
        assert_eq!(
            run_str(&st, &["list-sessions", "-F", "#{session_name}"]).stdout,
            "0\nb\n"
        );
    }

    #[test]
    fn kill_session_clear_alerts_missing_target_errors() {
        let st = state();
        let r = run_str(&st, &["kill-session", "-C", "-t", "nope"]);
        assert_eq!(r.exit, 1);
        assert_eq!(r.stderr, "can't find session: nope\n");
        // The real session is untouched.
        assert_eq!(
            run_str(&st, &["list-sessions", "-F", "#{session_name}"]).stdout,
            "0\n"
        );
    }

    // ---- command resolution: aliases, prefixes, ambiguity ----

    #[test]
    fn alias_resolves() {
        let st = state();
        // `ls` → list-sessions.
        assert!(run_str(&st, &["ls"]).stdout.starts_with("0: 1 windows"));
    }

    #[test]
    fn command_alias_with_an_empty_value_runs_nothing() {
        let st = state();
        assert_eq!(
            run_str(&st, &["set-option", "-s", "command-alias[100]", "zz="]).exit,
            0
        );
        // The entry matches, so `zz` is not an unknown command; it expands to an
        // empty command list, and its arguments go with it.
        let r = run_str(&st, &["zz", "ignored"]);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        assert_eq!(r.stdout, "");
        assert_eq!(r.stderr, "");
    }

    #[test]
    fn prefix_resolves() {
        let st = state();
        let r = run_str(&st, &["new-w", "-t", "0:", "-P", "-F", "#{window_index}"]);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        assert_eq!(r.stdout, "1\n");
    }

    #[test]
    fn ambiguous_prefix_errors() {
        let st = state();
        let r = run_str(&st, &["ne"]);
        assert_eq!(r.exit, 1);
        assert_eq!(
            r.stderr,
            "ambiguous command: ne, could be: new-pane, new-session, new-window, next-layout, next-window\n"
        );
    }

    // ---- multiple commands on one line ----

    #[test]
    fn multi_command_runs_in_order() {
        let st = state();
        let r = run_str(
            &st,
            &[
                "display-message",
                "-p",
                "a",
                ";",
                "display-message",
                "-p",
                "b",
            ],
        );
        assert_eq!(r.exit, 0);
        assert_eq!(r.stdout, "a\nb\n");
    }

    #[test]
    fn multi_command_accepts_separator_attached_to_command_word() {
        let st = state();
        let r = run_str(
            &st,
            &["display-message", "-p", "a;", "display-message", "-p", "b"],
        );
        assert_eq!(r.exit, 0);
        assert_eq!(r.stdout, "a\nb\n");
    }

    #[test]
    fn multi_command_accepts_separator_attached_to_flag_cluster() {
        let st = state();
        let r = run_str(&st, &["new-session", "-dsz;", "has-session", "-t", "z"]);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
    }

    #[test]
    fn multi_command_parse_error_aborts_line() {
        let st = state();
        // An unknown command anywhere aborts the whole line with no output.
        let r = run_str(&st, &["display-message", "-p", "a", ";", "frobnicate"]);
        assert_eq!(r.exit, 1);
        assert_eq!(r.stdout, "");
        assert_eq!(r.stderr, "unknown command: frobnicate\n");
    }

    #[test]
    fn multi_command_side_effect_visible() {
        let st = state();
        let r = run_str(
            &st,
            &[
                "new-session",
                "-d",
                "-s",
                "z",
                ";",
                "list-sessions",
                "-F",
                "#{session_name}",
            ],
        );
        assert_eq!(r.stdout, "0\nz\n", "got {:?}", r.stdout);
    }

    // ---- new-session -A ----

    #[test]
    fn new_session_attach_existing_is_tty_error() {
        let st = state();
        let r = run_str(&st, &["new-session", "-A", "-d", "-s", "0"]);
        assert_eq!(r.exit, 1);
        assert!(r.stderr.contains("not a terminal"), "got {:?}", r.stderr);
    }

    #[test]
    fn new_session_attach_creates_when_absent() {
        let st = state();
        let r = run_str(
            &st,
            &[
                "new-session",
                "-A",
                "-d",
                "-s",
                "fresh",
                "-P",
                "-F",
                "#{session_name}",
            ],
        );
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        assert_eq!(r.stdout, "fresh\n");
    }

    // ---- environment ----

    #[test]
    fn environment_set_show_unset() {
        let st = state();
        assert_eq!(
            run_str(&st, &["set-environment", "-g", "FOO", "bar"]).exit,
            0
        );
        let show = run_str(&st, &["show-environment", "-g", "FOO"]);
        assert_eq!(show.stdout, "FOO=bar\n");
        // Unknown variable → exit 1.
        let miss = run_str(&st, &["show-environment", "-g", "NOPE"]);
        assert_eq!(miss.exit, 1);
        assert_eq!(miss.stderr, "unknown variable: NOPE\n");
        // Unset removes it.
        run_str(&st, &["set-environment", "-g", "-u", "FOO"]);
        assert_eq!(run_str(&st, &["show-environment", "-g", "FOO"]).exit, 1);
    }

    #[test]
    fn set_environment_expand_format_stores_expanded_value() {
        let st = state();
        assert_eq!(
            run_str(&st, &["set-environment", "-Fg", "FOO", "#{session_name}"]).exit,
            0
        );
        assert_eq!(
            run_str(&st, &["show-environment", "-g", "FOO"]).stdout,
            "FOO=0\n"
        );
    }

    #[test]
    fn hidden_environment_requires_show_h() {
        let st = state();
        assert_eq!(
            run_str(&st, &["set-environment", "-hg", "SECRET", "v"]).exit,
            0
        );
        assert_eq!(
            run_str(&st, &["show-environment", "-g", "SECRET"]).stdout,
            ""
        );
        assert_eq!(
            run_str(&st, &["show-environment", "-hg", "SECRET"]).stdout,
            "SECRET=v\n"
        );
    }

    #[test]
    fn removed_environment_is_rendered_and_replaced_by_set() {
        let st = state();
        run_str(&st, &["set-environment", "-g", "FOO", "bar"]);
        run_str(&st, &["set-environment", "-rg", "FOO"]);
        assert_eq!(
            run_str(&st, &["show-environment", "-g", "FOO"]).stdout,
            "-FOO\n"
        );
        assert!(run_str(&st, &["show-environment", "-g"])
            .stdout
            .contains("-FOO\n"));

        run_str(&st, &["set-environment", "-g", "FOO", "again"]);
        assert_eq!(
            run_str(&st, &["show-environment", "-g", "FOO"]).stdout,
            "FOO=again\n"
        );
    }

    // ---- pane lifecycle ----

    #[test]
    fn split_window_adds_active_pane() {
        let st = state();
        assert_eq!(run_str(&st, &["split-window", "-t", "0"]).exit, 0);
        let lp = run_str(
            &st,
            &[
                "list-panes",
                "-t",
                "0",
                "-F",
                "#{pane_index}:#{pane_active}",
            ],
        );
        assert_eq!(lp.stdout, "0:0\n1:1\n", "got {:?}", lp.stdout);
    }

    #[test]
    fn split_window_prints_new_pane_with_format() {
        let st = state();
        let result = run_str(
            &st,
            &[
                "split-window",
                "-t",
                "0",
                "-P",
                "-F",
                "#{window_index}.#{pane_index}",
            ],
        );
        assert_eq!(result.exit, 0);
        assert_eq!(result.stdout, "0.1\n");
    }

    #[test]
    fn split_window_before_inserts_pane_before_target() {
        let st = state();
        assert_eq!(run_str(&st, &["split-window", "-b", "-t", "0"]).exit, 0);
        let lp = run_str(
            &st,
            &[
                "list-panes",
                "-t",
                "0",
                "-F",
                "#{pane_index}:#{pane_id}:#{pane_active}",
            ],
        );
        // The new pane %1 lands before the target %0 and becomes active.
        assert_eq!(lp.stdout, "0:%1:1\n1:%0:0\n", "got {:?}", lp.stdout);
    }

    #[test]
    fn split_window_before_detached_keeps_original_active() {
        let st = state();
        assert_eq!(
            run_str(&st, &["split-window", "-b", "-d", "-t", "0"]).exit,
            0
        );
        let lp = run_str(
            &st,
            &[
                "list-panes",
                "-t",
                "0",
                "-F",
                "#{pane_index}:#{pane_id}:#{pane_active}",
            ],
        );
        // `-d` keeps the original %0 active even though the new %1 is inserted
        // before it (so the active pane's index shifts up to 1).
        assert_eq!(lp.stdout, "0:%1:0\n1:%0:1\n", "got {:?}", lp.stdout);
    }

    #[test]
    fn select_pane_changes_active() {
        let st = state();
        run_str(&st, &["split-window", "-t", "0"]); // panes 0,1; active 1
        assert_eq!(run_str(&st, &["select-pane", "-t", "0.0"]).exit, 0);
        let lp = run_str(
            &st,
            &[
                "list-panes",
                "-t",
                "0",
                "-F",
                "#{pane_index}:#{pane_active}",
            ],
        );
        assert_eq!(lp.stdout, "0:1\n1:0\n", "got {:?}", lp.stdout);
    }

    #[test]
    fn pipe_pane_missing_pane_errors() {
        let st = state();
        let r = run_str(&st, &["pipe-pane", "-t", "0:0.9", "cat"]);
        assert_eq!(r.exit, 1);
        assert!(r.stderr.contains("can't find pane"), "got {:?}", r.stderr);
    }

    #[test]
    fn pipe_pane_missing_bare_target_reports_pane_error() {
        let st = state();
        let r = run_str(&st, &["pipe-pane", "-t", "missing"]);
        assert_eq!(r.exit, 1);
        assert_eq!(r.stderr, "can't find pane: missing\n");
    }

    #[test]
    fn select_pane_mark_sets_flags_without_selecting() {
        let st = state();
        run_str(&st, &["split-window", "-t", "0:0"]); // panes 0,1; active 1
        assert_eq!(run_str(&st, &["select-pane", "-m", "-t", "0:0.0"]).exit, 0);
        // The mark lands on pane 0; #{pane_marked_set} is set server-wide, but the
        // active pane (1) is unchanged.
        let lp = run_str(
            &st,
            &[
                "list-panes",
                "-t",
                "0:0",
                "-F",
                "#{pane_index}:#{pane_marked}:#{pane_marked_set}:#{pane_active}",
            ],
        );
        assert_eq!(lp.stdout, "0:1:1:0\n1:0:1:1\n", "got {:?}", lp.stdout);
    }

    #[test]
    fn select_pane_mark_toggles_and_clears() {
        let st = state();
        run_str(&st, &["split-window", "-t", "0:0"]);
        let probe = &[
            "display-message",
            "-p",
            "-t",
            "0:0.1",
            "#{pane_marked}#{pane_marked_set}",
        ];
        run_str(&st, &["select-pane", "-m", "-t", "0:0.1"]);
        assert_eq!(run_str(&st, probe).stdout, "11\n");
        // Re-marking the same pane toggles the mark off.
        run_str(&st, &["select-pane", "-m", "-t", "0:0.1"]);
        assert_eq!(run_str(&st, probe).stdout, "00\n");
        // -M clears an existing mark unconditionally.
        run_str(&st, &["select-pane", "-m", "-t", "0:0.1"]);
        run_str(&st, &["select-pane", "-M"]);
        assert_eq!(run_str(&st, probe).stdout, "00\n");
    }

    #[test]
    fn select_pane_mark_cleared_when_marked_pane_killed() {
        let st = state();
        run_str(&st, &["split-window", "-t", "0:0"]); // panes 0,1
        run_str(&st, &["select-pane", "-m", "-t", "0:0.1"]);
        // Killing the marked pane drops the mark: #{pane_marked_set} on the
        // survivor reads 0, matching tmux (a stale mark never lingers).
        run_str(&st, &["kill-pane", "-t", "0:0.1"]);
        let dm = run_str(
            &st,
            &["display-message", "-p", "-t", "0:0.0", "#{pane_marked_set}"],
        );
        assert_eq!(dm.stdout, "0\n", "got {:?}", dm.stdout);
    }

    #[test]
    fn kill_pane_removes_targeted_pane() {
        let st = state();
        run_str(&st, &["split-window", "-t", "0"]); // panes 0,1
        assert_eq!(run_str(&st, &["kill-pane", "-t", "0.1"]).exit, 0);
        let lp = run_str(&st, &["list-panes", "-t", "0", "-F", "#{pane_index}"]);
        assert_eq!(lp.stdout, "0\n", "got {:?}", lp.stdout);
    }

    #[test]
    fn respawn_pane_refuses_live_pane_without_k() {
        let st = state();
        // The bootstrap pane has not exited → tmux treats it as still active.
        let r = run_str(&st, &["respawn-pane", "-t", "0"]);
        assert_eq!(r.exit, 1);
        assert_eq!(r.stdout, "");
        assert_eq!(
            r.stderr, "respawn pane failed: pane 0:0.0 still active\n",
            "got {:?}",
            r.stderr
        );
    }

    #[test]
    fn respawn_pane_with_k_succeeds_silently() {
        let st = state();
        let r = run_str(&st, &["respawn-pane", "-k", "-t", "0"]);
        assert_eq!(r.exit, 0);
        assert_eq!(r.stdout, "");
        assert_eq!(r.stderr, "");
    }

    #[test]
    fn respawn_pane_missing_target_reports_pane_error() {
        let st = state();
        let r = run_str(&st, &["respawn-pane", "-t", "nope"]);
        assert_eq!(r.exit, 1);
        assert_eq!(r.stderr, "can't find pane: nope\n", "got {:?}", r.stderr);
    }

    #[test]
    fn respawn_window_refuses_live_window_without_k() {
        let st = state();
        // The bootstrap window's pane has not exited → still active.
        let r = run_str(&st, &["respawn-window", "-t", "0"]);
        assert_eq!(r.exit, 1);
        assert_eq!(r.stdout, "");
        assert_eq!(
            r.stderr, "respawn window failed: window 0:0 still active\n",
            "got {:?}",
            r.stderr
        );
    }

    #[test]
    fn respawn_window_with_k_succeeds_silently() {
        let st = state();
        let r = run_str(&st, &["respawn-window", "-k", "-t", "0"]);
        assert_eq!(r.exit, 0);
        assert_eq!(r.stdout, "");
        assert_eq!(r.stderr, "");
    }

    #[test]
    fn respawn_window_missing_target_reports_window_error() {
        let st = state();
        let r = run_str(&st, &["respawn-window", "-t", "nope"]);
        assert_eq!(r.exit, 1);
        assert_eq!(r.stderr, "can't find window: nope\n", "got {:?}", r.stderr);
    }

    // ---- window movement ----

    #[test]
    fn swap_window_exchanges_contents() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:"]);
        run_str(&st, &["rename-window", "-t", "0:0", "w0"]);
        run_str(&st, &["rename-window", "-t", "0:1", "w1"]);
        assert_eq!(
            run_str(&st, &["swap-window", "-s", "0:0", "-t", "0:1"]).exit,
            0
        );
        let lw = run_str(
            &st,
            &[
                "list-windows",
                "-t",
                "0",
                "-F",
                "#{window_index}:#{window_name}",
            ],
        );
        assert_eq!(lw.stdout, "0:w1\n1:w0\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn move_window_renumbers() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:"]); // index 1
        assert_eq!(
            run_str(&st, &["move-window", "-s", "0:1", "-t", "0:5"]).exit,
            0
        );
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(lw.stdout, "0\n5\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn move_window_selects_moved() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:1"]); // becomes current
        run_str(&st, &["select-window", "-t", "0:0"]); // move current back to 0
        assert_eq!(
            run_str(&st, &["move-window", "-s", "0:1", "-t", "0:5"]).exit,
            0
        );
        let cur = run_str(
            &st,
            &["display-message", "-p", "-t", "0", "#{window_index}"],
        );
        assert_eq!(cur.stdout, "5\n", "got {:?}", cur.stdout);
    }

    #[test]
    fn move_window_detach_keeps_current() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:1"]);
        run_str(&st, &["select-window", "-t", "0:0"]);
        assert_eq!(
            run_str(&st, &["move-window", "-d", "-s", "0:1", "-t", "0:5"]).exit,
            0
        );
        let cur = run_str(
            &st,
            &["display-message", "-p", "-t", "0", "#{window_index}"],
        );
        assert_eq!(cur.stdout, "0\n", "got {:?}", cur.stdout);
    }

    #[test]
    fn move_window_session_only_destination_uses_lowest_free_index() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:2"]);
        assert_eq!(
            run_str(&st, &["move-window", "-s", "0:2", "-t", "0:"]).exit,
            0
        );
        let windows = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(windows.stdout, "0\n1\n");
    }

    #[test]
    fn move_pane_without_destination_validates_the_source_first() {
        let st = state();
        let result = run_str(&st, &["move-pane", "-s", "missing"]);
        assert_eq!(result.exit, 1);
        assert_eq!(result.stderr, "can't find pane: missing\n");
    }

    #[test]
    fn break_pane_selects_broken_off_window() {
        let st = state();
        run_str(&st, &["split-window", "-t", "0"]);
        assert_eq!(run_str(&st, &["break-pane", "-s", "0.1"]).exit, 0);
        let cur = run_str(
            &st,
            &["display-message", "-p", "-t", "0", "#{window_index}"],
        );
        assert_eq!(cur.stdout, "1\n", "got {:?}", cur.stdout);
    }

    #[test]
    fn break_pane_name_disables_automatic_rename() {
        let st = state();
        run_str(&st, &["split-window", "-t", "0"]);
        assert_eq!(
            run_str(&st, &["break-pane", "-s", "0.1", "-n", "brk"]).exit,
            0
        );
        assert_eq!(
            run_str(
                &st,
                &["display-message", "-p", "-t", "0:1", "#{window_name}",],
            )
            .stdout,
            "brk\n"
        );
        assert_eq!(
            st.borrow_mut().option_for_target("0:1", "automatic-rename"),
            Some("off")
        );
    }

    #[test]
    fn break_pane_detach_keeps_current() {
        let st = state();
        run_str(&st, &["split-window", "-t", "0"]);
        assert_eq!(run_str(&st, &["break-pane", "-d", "-s", "0.1"]).exit, 0);
        let cur = run_str(
            &st,
            &["display-message", "-p", "-t", "0", "#{window_index}"],
        );
        assert_eq!(cur.stdout, "0\n", "got {:?}", cur.stdout);
    }

    #[test]
    fn move_window_kill_replaces_occupied_index() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:1"]);
        assert_eq!(
            run_str(&st, &["move-window", "-k", "-s", "0:0", "-t", "0:1"]).exit,
            0
        );
        let lw = run_str(
            &st,
            &[
                "list-windows",
                "-t",
                "0",
                "-F",
                "#{window_index}:#{window_active}",
            ],
        );
        assert_eq!(lw.stdout, "1:1\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn move_window_after_index_relocates_after_anchor() {
        // windows 0, 2 (index 1 free). `-a -t 0:2` moves window 0 to index 3.
        let st = state();
        run_str(&st, &["new-window", "-t", "0:2"]);
        assert_eq!(
            run_str(&st, &["move-window", "-a", "-s", "0:0", "-t", "0:2"]).exit,
            0
        );
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(lw.stdout, "2\n3\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn move_window_after_index_shuffles_run_up() {
        // windows 0,1,2,3. `-a -t 0:0` moves window 1 to just after index 0: the
        // contiguous run [1,4) shifts up (including the source, tmux-style), then
        // the source lands at 1 — leaving a gap at 2. A remove-first strategy would
        // instead close up to 0,1,2,3, so this pins the shuffle-then-move order.
        let st = state();
        run_str(&st, &["new-window", "-t", "0:1"]);
        run_str(&st, &["new-window", "-t", "0:2"]);
        run_str(&st, &["new-window", "-t", "0:3"]);
        assert_eq!(
            run_str(&st, &["move-window", "-a", "-s", "0:1", "-t", "0:0"]).exit,
            0
        );
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(lw.stdout, "0\n1\n3\n4\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn move_window_before_index_relocates_before_anchor() {
        // windows 0, 2. `-b -t 0:2` targets index 2; it is occupied, so the run
        // shifts up (2 -> 3) and the source lands at 2.
        let st = state();
        run_str(&st, &["new-window", "-t", "0:2"]);
        assert_eq!(
            run_str(&st, &["move-window", "-b", "-s", "0:0", "-t", "0:2"]).exit,
            0
        );
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(lw.stdout, "2\n3\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn move_window_after_index_selects_moved() {
        // move-window follows to the relocated window by default.
        let st = state();
        run_str(&st, &["new-window", "-t", "0:2"]);
        run_str(&st, &["select-window", "-t", "0:2"]);
        assert_eq!(
            run_str(&st, &["move-window", "-a", "-s", "0:0", "-t", "0:2"]).exit,
            0
        );
        let cur = run_str(
            &st,
            &["display-message", "-p", "-t", "0", "#{window_index}"],
        );
        assert_eq!(cur.stdout, "3\n", "got {:?}", cur.stdout);
    }

    #[test]
    fn move_window_after_index_detach_keeps_current() {
        // `-d` relocates without following: the current window stays put.
        let st = state();
        run_str(&st, &["new-window", "-t", "0:2"]);
        run_str(&st, &["select-window", "-t", "0:2"]);
        assert_eq!(
            run_str(&st, &["move-window", "-d", "-a", "-s", "0:0", "-t", "0:2"]).exit,
            0
        );
        let cur = run_str(
            &st,
            &["display-message", "-p", "-t", "0", "#{window_index}"],
        );
        assert_eq!(cur.stdout, "2\n", "got {:?}", cur.stdout);
    }

    #[test]
    fn link_window_after_index_links_after_anchor() {
        // windows 0, 2 (index 1 free). `-a -t 0:2` links window 0 in at index 3
        // (after window 2); the source keeps its own slot 0 too (link, not move).
        let st = state();
        run_str(&st, &["new-window", "-t", "0:2"]);
        assert_eq!(
            run_str(&st, &["link-window", "-a", "-s", "0:0", "-t", "0:2"]).exit,
            0
        );
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(lw.stdout, "0\n2\n3\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn link_window_session_only_destination_uses_lowest_free_index() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:2"]);
        assert_eq!(
            run_str(&st, &["link-window", "-s", "0:2", "-t", "0:"]).exit,
            0
        );
        let windows = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(windows.stdout, "0\n1\n2\n");
    }

    #[test]
    fn link_window_after_index_shuffles_run_up() {
        // windows 0,1,3 (gap at 2). `-a -t 0:0` opens the slot after index 0: only
        // the contiguous run [1,2) shifts up (1 -> 2), stopping at the free index 2,
        // and the new link lands at 1 — window 3 is untouched. Source 0 is kept.
        let st = state();
        run_str(&st, &["new-window", "-t", "0:1"]);
        run_str(&st, &["new-window", "-t", "0:3"]);
        assert_eq!(
            run_str(&st, &["link-window", "-a", "-s", "0:0", "-t", "0:0"]).exit,
            0
        );
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(lw.stdout, "0\n1\n2\n3\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn link_window_before_index_places_at_anchor() {
        // link window A (index 0) before B (index 1): `-b` targets index 1, which is
        // occupied, so B shifts up (1 -> 2) and the new link (named after the source,
        // "A") lands at 1.
        let st = state();
        run_str(&st, &["rename-window", "-t", "0:0", "A"]);
        run_str(&st, &["new-window", "-t", "0:1", "-n", "B"]);
        assert_eq!(
            run_str(&st, &["link-window", "-b", "-s", "0:0", "-t", "0:1"]).exit,
            0
        );
        let lw = run_str(
            &st,
            &[
                "list-windows",
                "-t",
                "0",
                "-F",
                "#{window_index}:#{window_name}",
            ],
        );
        assert_eq!(lw.stdout, "0:A\n1:A\n2:B\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn link_window_relative_missing_anchor_links_explicit() {
        // A `-a`/`-b` anchor that names no existing window is ignored, and the link
        // is placed at the explicit index (the plain path).
        let st = state();
        assert_eq!(
            run_str(&st, &["link-window", "-a", "-s", "0:0", "-t", "0:5"]).exit,
            0
        );
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(lw.stdout, "0\n5\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn link_window_after_index_selects_linked() {
        // link-window follows to the newly-linked window by default.
        let st = state();
        run_str(&st, &["new-window", "-t", "0:2"]);
        run_str(&st, &["select-window", "-t", "0:2"]);
        assert_eq!(
            run_str(&st, &["link-window", "-a", "-s", "0:0", "-t", "0:2"]).exit,
            0
        );
        let cur = run_str(
            &st,
            &["display-message", "-p", "-t", "0", "#{window_index}"],
        );
        assert_eq!(cur.stdout, "3\n", "got {:?}", cur.stdout);
    }

    #[test]
    fn link_window_after_index_detach_keeps_current() {
        // `-d` links without following: the current window stays put.
        let st = state();
        run_str(&st, &["new-window", "-t", "0:2"]);
        run_str(&st, &["select-window", "-t", "0:2"]);
        assert_eq!(
            run_str(&st, &["link-window", "-d", "-a", "-s", "0:0", "-t", "0:2"]).exit,
            0
        );
        let cur = run_str(
            &st,
            &["display-message", "-p", "-t", "0", "#{window_index}"],
        );
        assert_eq!(cur.stdout, "2\n", "got {:?}", cur.stdout);
    }

    // ---- size / attach format variables ----

    #[test]
    fn size_and_attach_vars() {
        let st = state();
        let r = run_str(
            &st,
            &[
                "display-message",
                "-t",
                "0",
                "-p",
                "#{session_attached}/#{window_width}x#{window_height}",
            ],
        );
        assert_eq!(r.stdout, "0/80x24\n", "got {:?}", r.stdout);
    }

    // ---- target syntax: ids and pane dot ----

    #[test]
    fn target_by_ids() {
        let st = state();
        assert_eq!(
            run_str(
                &st,
                &["display-message", "-t", "$0", "-p", "#{session_name}"]
            )
            .stdout,
            "0\n"
        );
        assert_eq!(
            run_str(
                &st,
                &["display-message", "-t", "@0", "-p", "#{window_index}"]
            )
            .stdout,
            "0\n"
        );
    }

    #[test]
    fn target_pane_dot() {
        let st = state();
        run_str(&st, &["split-window", "-t", "0"]); // pane 1
        let r = run_str(
            &st,
            &["display-message", "-t", "0.1", "-p", "#{pane_index}"],
        );
        assert_eq!(r.stdout, "1\n", "got {:?}", r.stdout);
    }

    #[test]
    fn has_session_window_target() {
        let st = state();
        assert_eq!(run_str(&st, &["has-session", "-t", "0:0"]).exit, 0);
        assert_eq!(run_str(&st, &["has-session", "-t", "$0"]).exit, 0);
    }

    // ---- -n names ----

    #[test]
    fn new_window_and_session_names() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:", "-n", "named"]);
        assert_eq!(
            run_str(
                &st,
                &["display-message", "-t", "0:1", "-p", "#{window_name}"]
            )
            .stdout,
            "named\n"
        );
        run_str(&st, &["new-session", "-d", "-s", "ns", "-n", "first"]);
        assert_eq!(
            run_str(&st, &["list-windows", "-t", "ns", "-F", "#{window_name}"]).stdout,
            "first\n"
        );
    }

    // ---- -f filter / -a all ----

    #[test]
    fn list_sessions_filter() {
        let st = state();
        run_str(&st, &["new-session", "-d", "-s", "keep"]);
        let r = run_str(
            &st,
            &[
                "list-sessions",
                "-f",
                "#{==:#{session_name},keep}",
                "-F",
                "#{session_name}",
            ],
        );
        assert_eq!(r.stdout, "keep\n", "got {:?}", r.stdout);
    }

    #[test]
    fn list_windows_and_panes_all() {
        let st = state();
        run_str(&st, &["new-session", "-d", "-s", "b"]);
        let lw = run_str(
            &st,
            &[
                "list-windows",
                "-a",
                "-F",
                "#{session_name}:#{window_index}",
            ],
        );
        assert_eq!(lw.stdout, "0:0\nb:0\n", "got {:?}", lw.stdout);
        run_str(&st, &["split-window", "-t", "0:"]);
        let lp = run_str(
            &st,
            &["list-panes", "-a", "-F", "#{session_name}:#{pane_index}"],
        );
        assert_eq!(lp.stdout, "0:0\n0:1\nb:0\n", "got {:?}", lp.stdout);
    }

    // ---- base-index ----

    #[test]
    fn base_index_shifts_new_session() {
        let st = state();
        run_str(&st, &["set-option", "-g", "base-index", "1"]);
        run_str(&st, &["new-session", "-d", "-s", "bi"]);
        let r = run_str(&st, &["list-windows", "-t", "bi", "-F", "#{window_index}"]);
        assert_eq!(r.stdout, "1\n", "got {:?}", r.stdout);
    }

    #[test]
    fn new_window_fills_lowest_gap() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:"]); // 1
        run_str(&st, &["new-window", "-t", "0:"]); // 2
        run_str(&st, &["kill-window", "-t", "0:1"]); // gap at 1
        let r = run_str(
            &st,
            &["new-window", "-t", "0:", "-P", "-F", "#{window_index}"],
        );
        assert_eq!(r.stdout, "1\n", "got {:?}", r.stdout);
    }

    #[test]
    fn rename_window_missing_name_error() {
        let st = state();
        let r = run_str(&st, &["rename-window", "-t", "0"]);
        assert_eq!(r.exit, 1);
        assert_eq!(
            r.stderr,
            "command rename-window: too few arguments (need at least 1)\n"
        );
    }

    // ---- round 5: unknown-flag validation ----

    #[test]
    fn unknown_flag_reports_getopt_error() {
        let st = state();
        let r = run_str(&st, &["list-sessions", "-Z"]);
        assert_eq!(r.exit, 1);
        assert_eq!(r.stderr, "command list-sessions: unknown flag -Z\n");
        assert!(r.stdout.is_empty());
    }

    #[test]
    fn unknown_flag_in_cluster_reports_the_bad_letter() {
        let st = state();
        // `-a` is valid for kill-window, `-Z` is not; getopt reports the bad one.
        let r = run_str(&st, &["kill-window", "-aZ", "-t", "0"]);
        assert_eq!(r.stderr, "command kill-window: unknown flag -Z\n");
    }

    #[test]
    fn value_flag_without_an_argument_reports_getopt_error() {
        let st = state();
        for (args, expected) in [
            (
                &["capture-pane", "-b"][..],
                "command capture-pane: -b expects an argument\n",
            ),
            (
                &["choose-buffer", "-F"][..],
                "command choose-buffer: -F expects an argument\n",
            ),
            (
                &["delete-buffer", "-b"][..],
                "command delete-buffer: -b expects an argument\n",
            ),
        ] {
            let result = run_str(&st, args);
            assert_eq!(result.exit, 1);
            assert_eq!(result.stderr, expected);
            assert!(result.stdout.is_empty());
        }
    }

    #[test]
    fn bad_flag_aborts_the_whole_line() {
        let st = state();
        // A bad flag on the second command is a parse error that aborts both, so
        // neither session is created.
        let r = run_str(
            &st,
            &[
                "new-session",
                "-d",
                "-s",
                "zzz",
                ";",
                "new-session",
                "-d",
                "-s",
                "aaa",
                "-Z",
            ],
        );
        assert_eq!(r.exit, 1);
        let ls = run_str(&st, &["list-sessions", "-F", "#{session_name}"]);
        assert_eq!(ls.stdout, "0\n", "no new session should exist");
    }

    #[test]
    fn valid_attached_and_separate_values_are_accepted() {
        let st = state();
        // `-F#{...}` (attached value) and `-t 0` (separate value) both parse.
        let r = run_str(&st, &["list-windows", "-t", "0", "-F#{window_index}"]);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        assert_eq!(r.stdout, "0\n");
    }

    // ---- round 5: zoom / options / buffers / layout ----

    #[test]
    fn resize_pane_z_toggles_zoom_flag() {
        let st = state();
        run_str(&st, &["split-window", "-t", "0"]);
        run_str(&st, &["resize-pane", "-Z", "-t", "0"]);
        let on = run_str(
            &st,
            &["display-message", "-t", "0", "-p", "#{window_zoomed_flag}"],
        );
        assert_eq!(on.stdout, "1\n");
        run_str(&st, &["resize-pane", "-Z", "-t", "0"]);
        let off = run_str(
            &st,
            &["display-message", "-t", "0", "-p", "#{window_zoomed_flag}"],
        );
        assert_eq!(off.stdout, "0\n");
    }

    #[test]
    fn resize_window_sets_manual_size() {
        let st = state();
        let r = run_str(&st, &["resize-window", "-t", "0:0", "-x", "40", "-y", "10"]);
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        let size = run_str(
            &st,
            &[
                "display-message",
                "-p",
                "-t",
                "0:0",
                "#{window_width}x#{window_height}",
            ],
        );
        assert_eq!(size.stdout, "40x10\n");
    }

    #[test]
    fn resize_window_single_axis_keeps_other_dimension() {
        let st = state();
        // Only -x: width changes, height keeps the default 24.
        run_str(&st, &["resize-window", "-t", "0:0", "-x", "30"]);
        let after_x = run_str(
            &st,
            &[
                "display-message",
                "-p",
                "-t",
                "0:0",
                "#{window_width}x#{window_height}",
            ],
        );
        assert_eq!(after_x.stdout, "30x24\n");
        // A later -y keeps the previously set manual width.
        run_str(&st, &["resize-window", "-t", "0:0", "-y", "12"]);
        let after_y = run_str(
            &st,
            &[
                "display-message",
                "-p",
                "-t",
                "0:0",
                "#{window_width}x#{window_height}",
            ],
        );
        assert_eq!(after_y.stdout, "30x12\n");
    }

    #[test]
    fn resize_window_rejects_out_of_range_size() {
        let st = state();
        let small = run_str(&st, &["resize-window", "-t", "0:0", "-x", "0"]);
        assert_eq!(small.exit, 1);
        assert_eq!(small.stderr, "width too small\n");

        let large = run_str(&st, &["resize-window", "-t", "0:0", "-x", "10001"]);
        assert_eq!(large.exit, 1);
        assert_eq!(large.stderr, "width too large\n");

        let bad = run_str(&st, &["resize-window", "-t", "0:0", "-y", "abc"]);
        assert_eq!(bad.exit, 1);
        assert_eq!(bad.stderr, "height invalid\n");

        // A rejected size leaves the window at its default.
        let size = run_str(
            &st,
            &[
                "display-message",
                "-p",
                "-t",
                "0:0",
                "#{window_width}x#{window_height}",
            ],
        );
        assert_eq!(size.stdout, "80x24\n");
    }

    #[test]
    fn window_commands_report_missing_bare_targets_as_windows() {
        for args in [
            &["break-pane", "-t", "missing"][..],
            &["last-pane", "-t", "missing"][..],
            &["next-layout", "-t", "missing"][..],
            &["previous-layout", "-t", "missing"][..],
            &["resize-window", "-t", "missing", "-x", "10"][..],
            &["rotate-window", "-t", "missing"][..],
        ] {
            let result = run_str(&state(), args);
            assert_eq!(result.exit, 1, "args={args:?}");
            assert_eq!(
                result.stderr, "can't find window: missing\n",
                "args={args:?}"
            );
        }
    }

    #[test]
    fn set_option_append_and_user_unset() {
        let st = state();
        run_str(&st, &["set-option", "-g", "@x", "a"]);
        run_str(&st, &["set-option", "-ga", "@x", "b"]);
        let r = run_str(&st, &["show-options", "-g", "-v", "@x"]);
        assert_eq!(r.stdout, "ab\n");
        run_str(&st, &["set-option", "-gu", "@x"]);
        let gone = run_str(&st, &["show-options", "-g", "-v", "@x"]);
        assert_eq!(gone.exit, 1);
        assert_eq!(gone.stderr, "invalid option: @x\n");
    }

    #[test]
    fn session_options_are_local_and_inherit_global_values() {
        let st = state();
        run_str(&st, &["new-session", "-d", "-s", "other"]);
        // `set-option`/`show-options` take a *pane* target, so a bare `0` is a
        // pane index in the current session, not the session named `0`. The
        // trailing `:` is what names the session.
        run_str(&st, &["set-option", "-t", "0:", "status-position", "top"]);

        assert_eq!(
            run_str(&st, &["show-options", "-t", "0:", "-v", "status-position"]).stdout,
            "top\n"
        );
        assert_eq!(
            run_str(
                &st,
                &["show-options", "-t", "other:", "-v", "status-position"]
            )
            .stdout,
            ""
        );
        assert_eq!(
            run_str(
                &st,
                &[
                    "show-options",
                    "-A",
                    "-t",
                    "other:",
                    "-v",
                    "status-position"
                ]
            )
            .stdout,
            "bottom\n"
        );
    }

    #[test]
    fn session_local_base_index_affects_only_that_session() {
        let st = state();
        run_str(&st, &["new-session", "-d", "-s", "other"]);
        run_str(&st, &["set-option", "-t", "0:", "base-index", "4"]);
        run_str(&st, &["new-window", "-d", "-t", "0:"]);
        run_str(&st, &["new-window", "-d", "-t", "other:"]);

        assert_eq!(
            run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]).stdout,
            "0\n4\n"
        );
        assert_eq!(
            run_str(
                &st,
                &["list-windows", "-t", "other", "-F", "#{window_index}"]
            )
            .stdout,
            "0\n1\n"
        );
    }

    #[test]
    fn linked_sessions_share_window_options_but_not_session_options() {
        let st = state();
        run_str(&st, &["new-session", "-d", "-s", "other"]);
        run_str(&st, &["link-window", "-s", "0:0", "-t", "other:1"]);
        run_str(&st, &["set-window-option", "-t", "0:0", "@shared", "yes"]);
        run_str(&st, &["set-option", "-t", "0:", "@session", "zero"]);

        assert_eq!(
            run_str(
                &st,
                &["show-window-options", "-t", "other:1", "-v", "@shared"]
            )
            .stdout,
            "yes\n"
        );
        let session = run_str(
            &st,
            &["show-options", "-q", "-t", "other:", "-v", "@session"],
        );
        assert_eq!(session.stdout, "");
    }

    #[test]
    fn grouped_sessions_keep_independent_session_options() {
        let st = state();
        run_str(&st, &["new-session", "-d", "-s", "mirror", "-t", "0"]);
        run_str(&st, &["set-option", "-t", "0:", "status-position", "top"]);

        assert_eq!(
            run_str(
                &st,
                &[
                    "show-options",
                    "-q",
                    "-t",
                    "mirror:",
                    "-v",
                    "status-position"
                ]
            )
            .stdout,
            ""
        );
        assert_eq!(
            run_str(
                &st,
                &[
                    "show-options",
                    "-A",
                    "-t",
                    "mirror:",
                    "-v",
                    "status-position"
                ]
            )
            .stdout,
            "bottom\n"
        );
    }

    #[test]
    fn pane_options_follow_a_moved_pane_and_reinherit_from_its_new_window() {
        let st = state();
        run_str(&st, &["new-window", "-d", "-t", "0:1"]);
        run_str(&st, &["split-window", "-d", "-t", "0:0"]);
        run_str(&st, &["set-option", "-p", "-t", "0:0.1", "@pane", "kept"]);
        run_str(&st, &["set-option", "-t", "0:0", "synchronize-panes", "on"]);
        run_str(&st, &["move-pane", "-s", "0:0.1", "-t", "0:1.0"]);

        assert_eq!(
            run_str(&st, &["show-options", "-p", "-t", "0:1.1", "-v", "@pane"]).stdout,
            "kept\n"
        );
        assert_eq!(
            run_str(
                &st,
                &[
                    "show-options",
                    "-A",
                    "-p",
                    "-t",
                    "0:1.1",
                    "-v",
                    "synchronize-panes",
                ]
            )
            .stdout,
            "off\n"
        );
    }

    #[test]
    fn whole_array_assignment_and_append_create_indexed_entries() {
        let st = state();
        run_str(&st, &["set-option", "-g", "status-format", "first"]);
        run_str(&st, &["set-option", "-ag", "status-format", "second"]);

        assert_eq!(
            run_str(&st, &["show-options", "-g", "status-format"]).stdout,
            "status-format[0] first\nstatus-format[1] second\n"
        );
        assert_eq!(
            run_str(&st, &["show-options", "-gv", "status-format[0]"]).stdout,
            "first\n"
        );
    }

    #[test]
    fn local_array_entry_shadows_the_complete_global_array() {
        let st = state();
        run_str(&st, &["set-option", "-g", "status-format[0]", "global"]);
        run_str(&st, &["set-option", "-t", "0", "status-format[1]", "local"]);

        assert_eq!(
            run_str(&st, &["show-options", "-t", "0", "-v", "status-format[0]"]).stdout,
            "\n"
        );
        assert_eq!(
            run_str(
                &st,
                &["show-options", "-A", "-t", "0", "-v", "status-format[1]"]
            )
            .stdout,
            "local\n"
        );
    }

    #[test]
    fn unset_uppercase_clears_window_and_all_pane_overrides() {
        let st = state();
        run_str(&st, &["split-window", "-d", "-t", "0:0"]);
        run_str(
            &st,
            &["set-option", "-p", "-t", "0:0.0", "synchronize-panes", "on"],
        );
        run_str(
            &st,
            &["set-option", "-p", "-t", "0:0.1", "synchronize-panes", "on"],
        );
        run_str(&st, &["set-option", "-t", "0:0", "synchronize-panes", "on"]);
        run_str(&st, &["set-option", "-U", "-t", "0:0", "synchronize-panes"]);

        for pane in ["0:0.0", "0:0.1"] {
            assert_eq!(
                run_str(
                    &st,
                    &[
                        "show-options",
                        "-A",
                        "-p",
                        "-t",
                        pane,
                        "-v",
                        "synchronize-panes"
                    ]
                )
                .stdout,
                "off\n"
            );
        }
    }

    #[test]
    fn set_option_no_overwrite_preserves_existing_value() {
        let st = state();
        assert_eq!(run_str(&st, &["set-option", "-og", "@x", "a"]).exit, 0);

        let duplicate = run_str(&st, &["set-option", "-og", "@x", "b"]);
        assert_eq!(duplicate.exit, 1);
        assert_eq!(duplicate.stderr, "already set: @x\n");
        assert_eq!(
            run_str(&st, &["show-options", "-g", "-v", "@x"]).stdout,
            "a\n"
        );

        let quiet_duplicate = run_str(&st, &["set-option", "-qog", "@x", "c"]);
        assert_eq!(quiet_duplicate.exit, 0);
        assert_eq!(quiet_duplicate.stderr, "");
        assert_eq!(
            run_str(&st, &["show-options", "-g", "-v", "@x"]).stdout,
            "a\n"
        );
    }

    #[test]
    fn set_option_expand_format_stores_expanded_value() {
        let st = state();
        assert_eq!(
            run_str(&st, &["set-option", "-Fg", "@x", "#{session_name}"]).exit,
            0
        );
        assert_eq!(run_str(&st, &["show-options", "-gv", "@x"]).stdout, "0\n");
    }

    #[test]
    fn show_options_catalog_defaults_under_global_scope() {
        let st = state();
        // A global query of an unset-but-valid option returns its catalog default,
        // in both the full-line and value-only forms, across every option table.
        assert_eq!(
            run_str(&st, &["show-options", "-g", "status"]).stdout,
            "status on\n"
        );
        assert_eq!(
            run_str(&st, &["show-options", "-g", "-v", "status"]).stdout,
            "on\n"
        );
        // synchronize-panes is a window option; monitor-bell a server one — the
        // flat catalog resolves both like tmux's cross-table lookup.
        assert_eq!(
            run_str(&st, &["show-options", "-g", "-v", "synchronize-panes"]).stdout,
            "off\n"
        );
        assert_eq!(
            run_str(&st, &["show-options", "-g", "-v", "monitor-bell"]).stdout,
            "on\n"
        );
    }

    #[test]
    fn show_options_local_scope_has_no_defaults() {
        let st = state();
        // Without -g/-s, an unset valid option is empty — defaults live only in
        // the global scope.
        let r = run_str(&st, &["show-options", "-v", "status"]);
        assert_eq!(r.exit, 0);
        assert_eq!(r.stdout, "");
    }

    #[test]
    fn invalid_option_name_errors_on_set_and_show() {
        let st = state();
        // An unknown non-user name is `invalid option: NAME` (exit 1) on both
        // set-option and every show-options form, before any side effect.
        let set = run_str(&st, &["set-option", "-g", "no-such-option-xyz", "1"]);
        assert_eq!(set.exit, 1);
        assert_eq!(set.stderr, "invalid option: no-such-option-xyz\n");

        let show = run_str(&st, &["show-options", "-g", "no-such-option-xyz"]);
        assert_eq!(show.exit, 1);
        assert_eq!(show.stderr, "invalid option: no-such-option-xyz\n");

        let show_v = run_str(&st, &["show-options", "-g", "-v", "no-such-option-xyz"]);
        assert_eq!(show_v.exit, 1);
        assert_eq!(show_v.stderr, "invalid option: no-such-option-xyz\n");
    }

    #[test]
    fn quiet_flag_suppresses_invalid_option_error() {
        let st = state();
        // `-q` swallows the invalid-option diagnostic: exit 0, no output, no
        // side effect — matching real tmux.
        let set = run_str(&st, &["set-option", "-q", "-g", "no-such-option-xyz", "1"]);
        assert_eq!(set.exit, 0);
        assert_eq!(set.stdout, "");
        assert_eq!(set.stderr, "");

        let show = run_str(&st, &["show-options", "-q", "-g", "no-such-option-xyz"]);
        assert_eq!(show.exit, 0);
        assert_eq!(show.stdout, "");
        assert_eq!(show.stderr, "");

        // A `-q` query of an unset user option is likewise silent, not
        // `invalid option: @name`.
        let user = run_str(&st, &["show-options", "-q", "-g", "@neverset"]);
        assert_eq!(user.exit, 0);
        assert_eq!(user.stdout, "");
        assert_eq!(user.stderr, "");
    }

    #[test]
    fn exit_empty_takes_hmux_after_session_beside_tmux_flag_values() {
        let st = state();
        let show =
            |st: &SharedState| run_str(st, &["show-options", "-s", "-v", "exit-empty"]).stdout;
        assert_eq!(show(&st), "after-session\n");

        // tmux's flag spellings still parse, and a valueless set still toggles
        // between them.
        assert_eq!(
            run_str(&st, &["set-option", "-s", "exit-empty", "yes"]).exit,
            0
        );
        assert_eq!(show(&st), "on\n");
        run_str(&st, &["set-option", "-s", "exit-empty"]);
        assert_eq!(show(&st), "off\n");

        // The extension value is kept by a valueless set, the way tmux keeps a
        // choice value past the first two.
        assert_eq!(
            run_str(&st, &["set-option", "-s", "exit-empty", "after-session"]).exit,
            0
        );
        run_str(&st, &["set-option", "-s", "exit-empty"]);
        assert_eq!(show(&st), "after-session\n");

        // Every other value is still tmux's flag error, and unsetting restores
        // the default rather than tmux's.
        let bad = run_str(&st, &["set-option", "-s", "exit-empty", "after-attach"]);
        assert_eq!(bad.exit, 1);
        assert_eq!(bad.stderr, "bad value: after-attach\n");
        run_str(&st, &["set-option", "-su", "exit-empty"]);
        assert_eq!(show(&st), "after-session\n");
    }

    #[test]
    fn valid_catalog_option_set_reads_back() {
        let st = state();
        // A real catalog option accepts a set and reads the stored value back,
        // shadowing its default.
        run_str(&st, &["set-option", "-g", "status", "off"]);
        assert_eq!(
            run_str(&st, &["show-options", "-g", "-v", "status"]).stdout,
            "off\n"
        );
    }

    #[test]
    fn buffer_error_texts_match_tmux() {
        let st = state();
        assert_eq!(run_str(&st, &["set-buffer"]).stderr, "no data specified\n");
        assert_eq!(
            run_str(&st, &["paste-buffer", "-b", "nope"]).stderr,
            "no buffer nope\n"
        );
        assert_eq!(
            run_str(&st, &["delete-buffer", "-b", "nope"]).stderr,
            "unknown buffer: nope\n"
        );
        assert_eq!(
            run_str(&st, &["set-buffer", "-b", "nope", "-n", "new"]).stderr,
            "unknown buffer: nope\n"
        );
    }

    #[test]
    fn set_buffer_rename_preserves_data_and_replaces_destination() {
        let st = state();
        run_str(&st, &["set-buffer", "-b", "source", "foo"]);
        run_str(&st, &["set-buffer", "-b", "destination", "bar"]);
        assert_eq!(
            run_str(&st, &["set-buffer", "-b", "source", "-n", "destination"]).exit,
            0
        );
        assert_eq!(
            run_str(&st, &["show-buffer", "-b", "destination"]).stdout,
            "foo"
        );
        assert_eq!(
            run_str(&st, &["show-buffer", "-b", "source"]).stderr,
            "no buffer source\n"
        );
    }

    #[test]
    fn save_buffer_writes_named_buffer_to_stdout() {
        let st = state();
        run_str(&st, &["set-buffer", "-b", "x", "hi"]);
        let r = run_str(&st, &["save-buffer", "-b", "x", "-"]);
        assert_eq!(r.exit, 0);
        assert_eq!(
            r.stdout, "hi",
            "stdout sink emits the raw bytes, no newline"
        );
        assert_eq!(r.stderr, "");
    }

    #[test]
    fn save_buffer_errors_mirror_show_buffer() {
        let st = state();
        // Arity: the path is mandatory and is checked before buffer resolution.
        assert_eq!(
            run_str(&st, &["save-buffer", "-b", "x"]).stderr,
            "command save-buffer: too few arguments (need at least 1)\n"
        );
        // Missing named buffer, with a path present.
        assert_eq!(
            run_str(&st, &["save-buffer", "-b", "nope", "-"]).stderr,
            "no buffer nope\n"
        );
        // No buffers at all, unnamed.
        assert_eq!(run_str(&st, &["save-buffer", "-"]).stderr, "no buffers\n");
    }

    #[test]
    fn save_buffer_writes_and_appends_to_a_file() {
        let st = state();
        run_str(&st, &["set-buffer", "-b", "x", "hi"]);
        run_str(&st, &["set-buffer", "-b", "y", "world"]);
        let path =
            std::env::temp_dir().join(format!("hmux_save_buffer_{}.txt", std::process::id()));
        let path_str = path.to_str().unwrap();

        let r = run_str(&st, &["save-buffer", "-b", "x", path_str]);
        assert_eq!(r.exit, 0);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hi");

        // `-a` appends rather than truncating.
        let r = run_str(&st, &["save-buffer", "-a", "-b", "y", path_str]);
        assert_eq!(r.exit, 0);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hiworld");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn multi_command_stdout_preserves_text_and_binary_order() {
        let st = state();
        st.borrow_mut().set_buffer(Some("raw"), b"middle\0\xff");

        let result = run_str(
            &st,
            &[
                "display-message",
                "-p",
                "before",
                ";",
                "show-buffer",
                "-b",
                "raw",
                ";",
                "display-message",
                "-p",
                "after",
            ],
        );

        assert_eq!(result.exit, 0);
        assert_eq!(result.stdout_data(), b"before\nmiddle\0\xffafter\n");
    }

    #[test]
    fn list_buffers_filter_includes_only_matching_buffers() {
        let st = state();
        run_str(&st, &["set-buffer", "-b", "x", "foo"]);
        run_str(&st, &["set-buffer", "-b", "y", "bar"]);
        assert_eq!(
            run_str(
                &st,
                &[
                    "list-buffers",
                    "-f",
                    "#{==:#{buffer_name},x}",
                    "-F",
                    "#{buffer_name}",
                ],
            )
            .stdout,
            "x\n"
        );
        assert_eq!(
            run_str(
                &st,
                &[
                    "list-buffers",
                    "-f",
                    "#{==:#{buffer_name},nope}",
                    "-F",
                    "#{buffer_name}",
                ],
            )
            .stdout,
            ""
        );
    }

    #[test]
    fn select_layout_rejects_unknown_name() {
        let st = state();
        let r = run_str(&st, &["select-layout", "-t", "0", "no-such-layout"]);
        assert_eq!(r.exit, 1);
        assert_eq!(r.stderr, "invalid layout: no-such-layout\n");
        // A known name and a custom layout string are accepted.
        assert_eq!(run_str(&st, &["select-layout", "-t", "0", "tiled"]).exit, 0);
    }

    #[test]
    fn select_layout_reports_missing_bare_target_as_a_pane() {
        let result = run_str(
            &state(),
            &["select-layout", "-t", "missing", "even-horizontal"],
        );
        assert_eq!(result.exit, 1);
        assert_eq!(result.stderr, "can't find pane: missing\n");
    }

    #[test]
    fn window_layout_is_checksummed_single_leaf() {
        let st = state();
        let r = run_str(
            &st,
            &["display-message", "-t", "0", "-p", "#{window_layout}"],
        );
        assert_eq!(r.stdout, "b25d,80x24,0,0,0\n");
    }

    #[test]
    fn join_pane_relocates_like_move_pane() {
        let st = state();
        run_str(&st, &["new-window", "-t", "0:"]);
        run_str(&st, &["join-pane", "-s", "0:1", "-t", "0:0"]);
        let r = run_str(&st, &["list-panes", "-t", "0:0", "-F", "#{pane_index}"]);
        assert_eq!(r.stdout, "0\n1\n");
    }

    #[test]
    fn join_pane_before_uses_tmux_37b_pane_list_order() {
        // tmux 3.7b positions the moved pane before the target geometrically,
        // but keeps it after the target in list-panes order.
        let st = state();
        run_str(&st, &["new-window", "-t", "0:"]);
        run_str(&st, &["join-pane", "-b", "-s", "0:1", "-t", "0:0"]);
        let r = run_str(
            &st,
            &[
                "list-panes",
                "-t",
                "0:0",
                "-F",
                "#{pane_index}:#{pane_id}:#{pane_active}",
            ],
        );
        assert_eq!(r.stdout, "0:%0:0\n1:%1:1\n");
    }

    // ---- interactive-attach routing (Intent classification) ----

    fn classify_str(args: &[&str]) -> Intent {
        let owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        classify(&owned)
    }

    #[test]
    fn bare_command_is_new_attach() {
        // Bare `tmux` sends an empty command line → default new-session → attach.
        assert_eq!(classify_str(&[]), Intent::NewAttach);
    }

    #[test]
    fn new_session_without_d_is_new_attach() {
        assert_eq!(classify_str(&["new-session"]), Intent::NewAttach);
        assert_eq!(classify_str(&["new-session", "-s", "x"]), Intent::NewAttach);
        // Alias and unambiguous prefix resolve the same way.
        assert_eq!(classify_str(&["new"]), Intent::NewAttach);
    }

    #[test]
    fn new_session_with_d_is_command() {
        // A detached create runs and exits — no attach. Also in a cluster (`-ds`).
        assert_eq!(classify_str(&["new-session", "-d"]), Intent::Command);
        assert_eq!(
            classify_str(&["new-session", "-d", "-s", "x"]),
            Intent::Command
        );
        assert_eq!(classify_str(&["new-session", "-ds", "x"]), Intent::Command);
    }

    #[test]
    fn attach_is_attach_intent() {
        assert_eq!(classify_str(&["attach-session"]), Intent::Attach);
        assert_eq!(classify_str(&["attach", "-t", "foo"]), Intent::Attach);
    }

    #[test]
    fn other_commands_are_command_intent() {
        assert_eq!(classify_str(&["list-sessions"]), Intent::Command);
        assert_eq!(classify_str(&["kill-server"]), Intent::Command);
        // A command *list* is left to the command path, even if it starts with
        // an attaching command.
        assert_eq!(
            classify_str(&["new-session", ";", "split-window"]),
            Intent::Command
        );
    }

    #[test]
    fn new_session_for_attach_creates_and_returns_name() {
        let st = state();
        let mut g = st.borrow_mut();
        // Explicit name.
        let context = ClientContext::default();
        let name = new_session_for_attach(
            &["new-session".into(), "-s".into(), "work".into()],
            &mut g,
            &context,
        )
        .expect("create");
        assert_eq!(name, "work");
        assert!(g.find("work").is_some());
        // Auto-named when no -s (the default session is "0", so the next is "1").
        let auto = new_session_for_attach(&[], &mut g, &context).expect("create auto");
        assert!(g.find(&auto).is_some());
        assert_ne!(auto, "work");
    }

    #[test]
    fn interactive_new_session_pane_spec_keeps_command_and_cwd() {
        let st = ServerState::with_test_session().expect("state");
        let args = normalize_argv(
            "new-session",
            &[
                "new-session".into(),
                "-c".into(),
                "/tmp".into(),
                "--".into(),
                "/bin/sh".into(),
                "-c".into(),
                "sleep 30".into(),
            ],
        );
        let spec = new_session_pane_spec(&args, &st, &ClientContext::default());
        match spec {
            PaneSpec::CommandIn(argv, cwd) => {
                assert_eq!(cwd, PathBuf::from("/tmp"));
                assert_eq!(argv, ["/bin/sh", "-c", "sleep 30"]);
            }
            PaneSpec::Inert | PaneSpec::Command(_) | PaneSpec::Existing(_) => {
                panic!("explicit -c must produce a cwd-aware command pane")
            }
        }
    }

    #[test]
    fn new_session_for_attach_a_finds_existing() {
        let st = state();
        let mut g = st.borrow_mut();
        // The default session "0" exists; `-A -s 0` must return it, not error.
        let name = new_session_for_attach(
            &["new-session".into(), "-A".into(), "-s".into(), "0".into()],
            &mut g,
            &ClientContext::default(),
        )
        .expect("attach-or-create");
        assert_eq!(name, "0");
    }

    #[test]
    fn new_session_for_attach_duplicate_errors_without_a() {
        let st = state();
        let mut g = st.borrow_mut();
        // "0" already exists; creating it again without -A is tmux's duplicate error.
        let err = new_session_for_attach(
            &["new-session".into(), "-s".into(), "0".into()],
            &mut g,
            &ClientContext::default(),
        )
        .unwrap_err();
        assert_eq!(err, "duplicate session: 0\n");
    }

    #[test]
    fn new_session_for_attach_creates_session_group() {
        let st = state();
        let mut g = st.borrow_mut();
        let name = new_session_for_attach(
            &[
                "new-session".into(),
                "-s".into(),
                "mirror".into(),
                "-t".into(),
                "0".into(),
            ],
            &mut g,
            &ClientContext::default(),
        )
        .expect("create grouped session");
        assert_eq!(name, "mirror");
        let original = g.find("0").expect("original");
        let mirror = g.find("mirror").expect("grouped session");
        assert_eq!(original.windows, mirror.windows);
        assert_eq!(original.link_set_id, mirror.link_set_id);
    }

    #[test]
    fn list_commands_single_prints_usage_line() {
        let st = state();
        let r = run_str(&st, &["list-commands", "has-session"]);
        assert_eq!(r.exit, 0);
        assert_eq!(r.stdout, "has-session (has) [-t target-session]\n");
        assert_eq!(r.stderr, "");
    }

    #[test]
    fn list_commands_custom_format_expands_command_fields() {
        let st = state();
        let template = "#{command_list_name}|#{command_list_alias}|#{command_list_usage}";
        let r = run_str(&st, &["list-commands", "-F", template, "has-session"]);
        assert_eq!(r.exit, 0);
        assert_eq!(r.stdout, "has-session|has|[-t target-session]\n");

        let r = run_str(&st, &["list-commands", "-F", template, "kill-server"]);
        assert_eq!(r.stdout, "kill-server||\n");
    }

    #[test]
    fn list_commands_resolves_alias_and_prefix() {
        let st = state();
        // Both an exact alias and an unambiguous prefix resolve to the command.
        for arg in ["has", "has-s"] {
            let r = run_str(&st, &["list-commands", arg]);
            assert_eq!(r.exit, 0, "arg {arg:?}");
            assert_eq!(
                r.stdout, "has-session (has) [-t target-session]\n",
                "arg {arg:?}"
            );
        }
    }

    #[test]
    fn list_commands_argless_command_has_trailing_space() {
        let st = state();
        // No alias, empty usage: tmux still emits the space, leaving a trailing one.
        let r = run_str(&st, &["list-commands", "kill-server"]);
        assert_eq!(r.exit, 0);
        assert_eq!(r.stdout, "kill-server \n");
    }

    #[test]
    fn list_commands_ambiguous_and_unknown_report_resolver_diagnostic() {
        let st = state();
        let amb = run_str(&st, &["list-commands", "list"]);
        assert_eq!(amb.exit, 1);
        assert_eq!(amb.stdout, "");
        assert_eq!(
            amb.stderr,
            "ambiguous command: list, could be: list-buffers, list-clients, list-commands, \
             list-keys, list-panes, list-sessions, list-windows\n"
        );

        let unk = run_str(&st, &["list-commands", "nosuchcmd"]);
        assert_eq!(unk.exit, 1);
        assert_eq!(unk.stdout, "");
        assert_eq!(unk.stderr, "unknown command: nosuchcmd\n");
    }

    #[test]
    fn list_commands_all_lists_the_whole_table() {
        let st = state();
        let r = run_str(&st, &["list-commands"]);
        assert_eq!(r.exit, 0);
        assert_eq!(r.stderr, "");
        // One line per command in the table, in table order.
        let lines: Vec<&str> = r.stdout.lines().collect();
        assert_eq!(lines.len(), registry::COMMAND_SPECS.len());
        assert_eq!(
            lines[0],
            "attach-session (attach) [-dErx] [-c working-directory] [-f flags] [-t target-session]"
        );
        assert_eq!(*lines.last().unwrap(), "wait-for (wait) [-L|-S|-U] channel");
    }
}
