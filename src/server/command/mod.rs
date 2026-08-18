//! The tmux command interpreter.
//!
//! Commands that are not implemented return a nonzero exit with an error line,
//! like tmux does for an unknown command. Output is returned as text + streams;
//! the protocol layer ([`crate::event_loop::protocol`]) delivers it over the
//! imsg file protocol.
//!
//! Behaviors here are pinned against real tmux by the differential conformance
//! suite (`hmux_conformance::behaviors`), which runs the identical command
//! sequence against this interpreter and against stock tmux and
//! asserts the observable results (exit code, stdout, stderr) match. When a gap
//! is found there, it's closed here.

pub(in crate::server) mod args;
pub(in crate::server) mod buffers;
pub(in crate::server) mod clients;
pub(in crate::server) mod configuration;
mod executable;
pub(in crate::server) mod execution;
mod identity;
pub(in crate::server) mod keys;
pub(in crate::server) mod panes;
pub(in crate::server) mod queue;
pub(in crate::server) mod server;
pub(in crate::server) mod sessions;
pub(crate) mod suspend;
pub(in crate::server) mod windows;

pub(in crate::server) use identity::Command;

pub(crate) use executable::{ExecutableCommand, LazyCommand};
use executable::ParsedCommand;

/// The tests that drive a job the way a command would need to build one.
#[cfg(test)]
pub(crate) use execution::{run_shell_from_argv, RunShell};

use args::ParsedArgs;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::future::Future;
use std::io;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::integration::status::PaneAgents;
use crate::observability::v1::PaneId;

use super::format::{self, Vars};
use super::key::{format_key_name, parse_key_name, KeyBase, KeyCode, SpecialKey};
use super::mouse::MouseEvent;
use super::options::{self, OptionScope, OptionSet, OptionsView};
use super::pane::PaneClass;
use super::registry::{self, Resolution, SpecResolution};
use super::state::{
    BackgroundJobRegistry, ClientActionResult, ClientMessage, ClientMessageResult, MenuItem,
    MenuRequest, ModeEdit, ModeItem, ModeKind, ModeView, OverlayRequest, PaneSpec, PopupRequest,
    PromptCompletion, PromptReply, ServerState, Session, SharedState, SpawnSession, SplitDirection,
    Target, WaitOutcome, WaitRegistry, WindowResizeAdjust, WindowResizeRequest, WindowSizePolicy,
};
use super::style::{CaptureStyleWriter, CellPresentation, Hyperlink, SgrDecoder};
use crate::sync::{yield_now, Completion, WakeFn};
use hmux_rt::TaskHandle;
use hmux_vt::{CaptureExtent, CellWidth, Grid, GridRow};
use std::cell::Cell;
use suspend::{SuspensionStart, SuspensionWait};

/// tmux's `NEW_SESSION_TEMPLATE` (cmd-new-session.c): what `new-session -P`
/// prints when no `-F` is given.
const NEW_SESSION_TEMPLATE: &str = "#{session_name}:";
/// tmux's `NEW_WINDOW_TEMPLATE` (cmd-new-window.c): what `new-window -P` prints.
const NEW_WINDOW_TEMPLATE: &str = "#{session_name}:#{window_index}.#{pane_index}";
const DISPLAY_MESSAGE_TEMPLATE: &str = "[#{session_name}] #{window_index}:#{window_name}, current pane #{pane_index} - (%H:%M %d-%b-%y)";
const NEW_SESSION_MAX_SIZE: u16 = 10_000;

/// A command line the attached client hands back to the queue rather than
/// running where it was resolved.
pub(crate) enum DeferredCommand {
    /// A stored command line, compiled where it was written or still text (see
    /// [`LazyCommand`]).
    Command(LazyCommand),
    /// Words that arrived already split: a menu or mode item's stored argv, a
    /// popup's close hook. The argv edge, compiled when the queue starts.
    Argv(Vec<String>),
    /// A prompt's answer, plus the words the client typed after it: two lexers,
    /// compiled separately and run as one line.
    Line { line: String, tail: Vec<String> },
}

/// The result of running a command: what to write to the client's stdout/stderr
/// and the process exit code.
pub struct CommandResult {
    pub stdout: String,
    pub stdout_bytes: Vec<u8>,
    pub stderr: String,
    pub exit: i32,
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
    /// The target this command's `after-*` hook body resolves against, when the
    /// command settles one of its own. tmux's `cmdq_fire_command` uses the
    /// find-state the command left behind, which the creation commands point at
    /// the window or pane they just made rather than at their own `-t`.
    pub(crate) after_hook_target: Option<String>,
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

/// What one queue frame runs under: tmux's `cmdq_state` flags, plus the hook
/// scope they go with.
///
/// This is the whole answer to "what does a nested invocation inherit". A child
/// queue — a hook body, an inserted command line — starts from a clone of its
/// parent's frame state and stamps its own latches on that clone; everything
/// queued beneath it inherits the result. Unlike the rest of
/// [`ClientContext`], none of this belongs to the client: two commands from the
/// same client run under different frame states when one of them is a hook
/// body.
#[derive(Clone, Default)]
pub(crate) struct FrameState {
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
    /// The file the command line this queue runs was compiled from, inherited
    /// by whatever a sourced command inserts so `#{current_file}` survives the
    /// nesting the way tmux's copied queue state does.
    pub(crate) current_file: Option<Rc<str>>,
}

/// Per-command-client process context collected from tmux identify frames.
///
/// Besides the process facts, this carries two kinds of execution state: who is
/// asking ([`ClientKind`], fixed per connection) and how the command came to
/// run ([`FrameState`], stamped onto the clone a hook-body or nested queue runs
/// with).
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
    /// The queue frame this command runs in. A child queue inherits it as a
    /// unit and stamps its own latches on the clone.
    pub(crate) frame: FrameState,
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
        matches!(self.kind, ClientKind::Control { .. }) || self.frame.nested_granularity
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
            deferred_commands: Vec::new(),
            background_commands: Vec::new(),
            continue_queue: false,
            inserted_results: Vec::new(),
            control_flags: 1,
            after_hook_target: None,
        }
    }

    pub(crate) fn err(stderr: impl Into<String>) -> Self {
        CommandResult {
            stdout: String::new(),
            stdout_bytes: Vec::new(),
            stderr: stderr.into(),
            exit: 1,
            deferred_commands: Vec::new(),
            background_commands: Vec::new(),
            continue_queue: false,
            inserted_results: Vec::new(),
            control_flags: 1,
            after_hook_target: None,
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
                deferred_commands: Vec::new(),
                background_commands: Vec::new(),
                continue_queue: false,
                inserted_results: Vec::new(),
                control_flags: 1,
                after_hook_target: None,
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

/// The lexed view of a `command-prompt` argv. The prompt's words travel to the
/// client that answers it and back, so its own parser reads them wherever they
/// arrive rather than from a typed command.
fn command_prompt_args(args: &[String]) -> ParsedArgs {
    ParsedArgs::lex("command-prompt", &normalize_argv("command-prompt", args))
}

pub(crate) fn command_prompt_target(args: &[String]) -> Option<String> {
    command_prompt_args(args).value('t').map(str::to_string)
}

pub(crate) fn command_prompt_waits(args: &[String]) -> bool {
    let args = command_prompt_args(args);
    !args.has('b') && !args.has('i')
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
    if let Err(error) = ExecutableCommand::compile_groups(vec![args], &[]) {
        return Err(error);
    }
    let normalized = command_prompt_args(args);
    let prompt_type = normalized.value('T').unwrap_or("command");
    if !matches!(
        prompt_type,
        "command" | "search" | "target" | "window-target"
    ) {
        return Err(format!("unknown type: {prompt_type}\n"));
    }
    let literal = normalized.has('l');
    let raw_prompt = normalized.value('p');
    let template = normalized.positionals().first().map(String::as_str);
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
    let raw_inputs = normalized.value('I').unwrap_or("");
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
    let single = normalized.has('1');
    let numeric = !single && normalized.has('N');
    let incremental = !single && !numeric && normalized.has('i');
    let key = !single && !numeric && !incremental && normalized.has('k');
    let backspace_exit = !single && !numeric && !incremental && !key && normalized.has('e');
    Ok(CommandPromptSpec {
        pages,
        single,
        numeric,
        incremental,
        key,
        backspace_exit,
        no_freeze: normalized.has('C'),
        prompt_type: prompt_type.to_string(),
    })
}

pub(crate) fn expand_command_prompt_format(
    source: &str,
    st: &mut ServerState,
    agents: &PaneAgents,
    context: &ClientContext,
) -> String {
    if !format::reads_vars(source) {
        return source.to_string();
    }
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
            st.seed_format_environment(&mut vars, st.sessions().get(resolved.session));
            if let Ok(entries) = st.format_option_entries(target.as_deref().unwrap_or_default()) {
                for (name, value) in entries {
                    vars.set(name.to_string(), value);
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
    let normalized = command_prompt_args(args);
    let mut template = normalized
        .positionals()
        .first()
        .map(String::as_str)
        .unwrap_or("%1")
        .to_string();
    if normalized.has('F') {
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
    let mut result = match start_resumable_command_argv(args, state, agents, &context) {
        Ok(queue) => driver.run_queue(queue, state, agents),
        Err(result) => result,
    };
    for request in result.background_commands.drain(..) {
        driver.run_background(request, state, agents);
    }
    driver.drain_notifications(state, agents);
    result
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
    let lexed = ParsedArgs::lex(name, &normalize_argv(name, args));
    match name {
        "load-buffer" => true,
        "display-message" | "split-window" | "new-pane" => lexed.has('I'),
        "save-buffer" => lexed.positionals().first().is_some_and(|path| path != "-"),
        _ => false,
    }
}

/// Run a deferred command line to completion on the calling thread. Test
/// scaffolding standing in for the loop that would have started its queue.
#[cfg(test)]
pub(crate) fn run_lazy_with_context(
    command: LazyCommand,
    state: &SharedState,
    agents: &PaneAgents,
    context: &ClientContext,
) -> CommandResult {
    let queue = match start_resumable_command(command, state, agents, context) {
        Ok(queue) => queue,
        Err(error) => return error,
    };
    crate::event_loop::test_driver::LoopCommandDriver::new()
        .expect("command test loop")
        .run_queue(queue, state, agents)
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
    let queue = match start_resumable_command_argv(args, state, agents, context) {
        Ok(queue) => queue,
        Err(error) => return error,
    };
    crate::event_loop::test_driver::LoopCommandDriver::new()
        .expect("command test loop")
        .run_queue(queue, state, agents)
}

/// Start a queue on an already compiled command line.
///
/// This is the queue boundary: the client paths that compile once — control
/// mode, the command client's file-protocol split — hand the compiled value
/// straight over instead of passing an argv back through the parser.
pub(crate) fn start_compiled_command(
    command: ExecutableCommand,
    agents: &PaneAgents,
    context: &ClientContext,
) -> ResumableCommandQueue {
    ResumableCommandQueue::new(command, agents, context)
}

/// Start a queue on deferred work: a body compiled where it was stored runs as
/// it was compiled, a line still kept as text is compiled here.
pub(crate) fn start_resumable_command(
    command: LazyCommand,
    state: &SharedState,
    agents: &PaneAgents,
    context: &ClientContext,
) -> Result<ResumableCommandQueue, CommandResult> {
    let compiled = command.compile(state).map_err(CommandResult::err)?;
    Ok(ResumableCommandQueue::new(compiled, agents, context))
}

/// Start a queue on a line that arrived already split into words.
///
/// The argv edge: a command client's operands, a menu or mode item's stored
/// words, the test scaffolding. Everything that *stores* a command line keeps
/// a [`LazyCommand`] and goes through [`start_resumable_command`] instead.
pub(crate) fn start_resumable_command_argv(
    args: &[String],
    state: &SharedState,
    agents: &PaneAgents,
    context: &ClientContext,
) -> Result<ResumableCommandQueue, CommandResult> {
    let aliases = {
        let state = state.borrow_mut();
        state.command_aliases()
    };
    let compiled = ExecutableCommand::compile_argv(args, &aliases).map_err(CommandResult::err)?;
    Ok(ResumableCommandQueue::new(compiled, agents, context))
}

pub(crate) fn start_resumable_command_string_with_tail(
    line: &str,
    tail: &[String],
    state: &SharedState,
    agents: &PaneAgents,
    context: &ClientContext,
) -> Result<ResumableCommandQueue, CommandResult> {
    let aliases = {
        let state = state.borrow_mut();
        state.command_aliases()
    };
    let mut compiled = ExecutableCommand::compile(line, &aliases).map_err(CommandResult::err)?;
    if !tail.is_empty() {
        compiled.extend(ExecutableCommand::compile_argv(tail, &aliases).map_err(CommandResult::err)?);
    }
    Ok(ResumableCommandQueue::new(compiled, agents, context))
}

struct PreviousCommandTargetContext {
    session_id: Option<u32>,
    window_id: Option<u32>,
    pane_id: Option<u32>,
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
        .frame
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
    let pane_id = default_target
        .map(|resolved| state.window(resolved.session, resolved.window).panes[resolved.pane].id);
    PreviousCommandTargetContext {
        session_id: state.replace_command_session_id(session_id),
        window_id: state.replace_command_window_id(window_id),
        pane_id: state.replace_command_pane_id(pane_id),
        active_panes: state.replace_command_active_panes(context.active_panes()),
        hook_vars: context
            .frame
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
    state.replace_command_pane_id(previous.pane_id);
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
    loops: Option<&dyn format::ScopedLoopSource>,
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
}

pub(crate) enum BackgroundCommandRequest {
    /// A command line to run detached, carrying with it whether it was already
    /// compiled where it was stored or is still text to compile when it fires.
    Command {
        command: LazyCommand,
        context: ClientContext,
    },
    IfShell {
        condition: String,
        then_command: Option<String>,
        else_command: Option<String>,
        context: ClientContext,
    },
    RunShell {
        command: execution::RunShell,
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
            Self::Command { command, context } => {
                PendingBackground::Ready(BackgroundCommand::Command(command), context)
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
                command,
                context,
                jobs,
            } => PendingBackground::Ready(BackgroundCommand::RunShell { command, jobs }, context),
        }
    }
}

/// What a detached queue starts from, once an `if-shell -b` condition (if any)
/// has picked its branch.
pub(crate) enum BackgroundCommand {
    Command(LazyCommand),
    RunShell {
        command: execution::RunShell,
        jobs: Rc<BackgroundJobRegistry>,
    },
}

/// The command the queue is running, and what its result counts for.
struct InflightCommand {
    command: ParsedCommand,
    source: Option<SourceLocation>,
    source_depth: u8,
    contributes_status: bool,
}

#[derive(Clone, Copy)]
enum NestedCapture {
    Hook,
    Inserted,
    Discard,
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

/// What a queue's owner can see while the queue is parked.
///
/// A running task is opaque to the actor that started it, but an attached
/// client has to know one thing about the suspension its queue is on: whether
/// answering it is the client's own job. The queue publishes that here, and a
/// change fires the owner's wake so it looks again — a `command-prompt -w`
/// whose client had stopped reading input would never be answered.
#[derive(Default)]
pub(crate) struct QueueStatus {
    allows_attach_io: Cell<bool>,
    wake: RefCell<Option<WakeFn>>,
}

impl QueueStatus {
    pub(crate) fn allows_attach_io(&self) -> bool {
        self.allows_attach_io.get()
    }

    fn set_allows_attach_io(&self, allows: bool) {
        if self.allows_attach_io.replace(allows) == allows {
            return;
        }
        let wake = self.wake.borrow_mut().take();
        if let Some(wake) = wake {
            wake();
        }
    }

    fn set_wake(&self, wake: &WakeFn) {
        *self.wake.borrow_mut() = Some(Rc::clone(wake));
    }
}

/// One command queue running as a task, from its owner's side.
///
/// Awaiting this is awaiting the queue: it reports what the queue produced, and
/// stays pending for as long as the queue is running or parked.
pub(crate) struct QueuedCommand {
    completion: Completion<io::Result<CommandResult>>,
    status: Rc<QueueStatus>,
}

impl QueuedCommand {
    pub(crate) fn new(
        completion: Completion<io::Result<CommandResult>>,
        status: Rc<QueueStatus>,
    ) -> Self {
        Self { completion, status }
    }

    pub(crate) fn allows_attach_io(&self) -> bool {
        self.status.allows_attach_io()
    }

    /// Install the wake for a change in what the queue is parked on.
    ///
    /// The value is awaited; this is the other thing an owner can care about,
    /// and it cannot be awaited the same way because the queue is opaque while
    /// it runs. One-shot: an owner that parks again arms it again.
    pub(crate) fn set_status_wake(&mut self, wake: &WakeFn) {
        self.status.set_wake(wake);
    }
}

impl Future for QueuedCommand {
    type Output = io::Result<CommandResult>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        // The outer error is the task disappearing; the inner one is the
        // queue's own. Both read the same way to the owner.
        Pin::new(&mut self.completion)
            .poll(context)
            .map(|result| result.and_then(|result| result))
    }
}

/// Run one command queue to completion, suspending as its commands need to.
pub(crate) async fn run_command_queue(
    mut queue: ResumableCommandQueue,
    state: SharedState,
    tasks: TaskHandle,
    budget: usize,
    status: Rc<QueueStatus>,
) -> io::Result<CommandResult> {
    queue.run(&state, &tasks, budget, &status).await
}

/// Start one command queue as a task, reporting through the handle its owner
/// polls.
///
/// The first turn is taken inline, so whatever the queue does before it first
/// suspends — the registry work a `wait-for` or a prompt orders by command
/// order, and the flag saying the owner is the one to answer — has happened by
/// the time this returns.
pub(crate) fn spawn_queue(
    tasks: &TaskHandle,
    queue: ResumableCommandQueue,
    state: SharedState,
    budget: usize,
) -> io::Result<QueuedCommand> {
    let (completion, sender) = crate::sync::completion_pair()?;
    let status = Rc::new(QueueStatus::default());
    let queue_status = Rc::clone(&status);
    let handle = tasks.clone();
    tasks.spawn_now(async move {
        let result = run_command_queue(queue, state, handle, budget, queue_status).await;
        sender.complete(result);
    });
    Ok(QueuedCommand::new(completion, status))
}

/// Run a detached queue, whose result nobody polls for: the caller only wants
/// to be told when it is over.
pub(crate) fn spawn_detached_queue(
    tasks: &TaskHandle,
    queue: ResumableCommandQueue,
    state: SharedState,
    budget: usize,
) -> io::Result<Completion<io::Result<CommandResult>>> {
    let (completion, sender) = crate::sync::completion_pair()?;
    let status = Rc::new(QueueStatus::default());
    let handle = tasks.clone();
    tasks.spawn_now(async move {
        let result = run_command_queue(queue, state, handle, budget, status).await;
        sender.complete(result);
    });
    Ok(completion)
}

/// Wait for an answer that only another client can give, saying while the queue
/// is parked whether giving it is the queue owner's own job.
///
/// The owner cannot see that from the outside, and a `command-prompt -w` whose
/// client had stopped reading its input would never be answered.
///
/// Whatever had to happen before anything waits — a `wait-for` taking its lock,
/// a prompt reaching its client — has happened by the time this runs: the order
/// commands touch a registry in is the order they ran in, which waiting first
/// would not preserve.
async fn wait_for_answer(
    status: &QueueStatus,
    allows_attach_io: bool,
    start: SuspensionStart,
) -> CommandResult {
    match start {
        // Nothing waits, so nothing is parked to report.
        SuspensionStart::Ready(result) => result,
        SuspensionStart::Waiting(wait) => {
            status.set_allows_attach_io(allows_attach_io);
            let result = wait.resolve().await;
            status.set_allows_attach_io(false);
            result
        }
    }
}

impl ResumableCommandQueue {
    /// Start a queue on one compiled command line. The compiled value is the
    /// only thing that becomes queue work: nothing above this boundary hands
    /// the queue text or an argv.
    fn new(command: ExecutableCommand, agents: &PaneAgents, context: &ClientContext) -> Self {
        let mut queue = queue::CommandQueue::new();
        queue.push_back_group(command.into_commands().into_iter().map(|command| SharedQueueItem::Command {
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
        }
    }

    /// Run every command in the queue, waiting wherever one has to.
    ///
    /// A command that cannot finish on its own awaits its answer where it
    /// suspended, rather than reporting the suspension up to a driver and being
    /// resumed with the answer threaded back in; a command that runs a queue of
    /// its own runs it here, nested as deep as the commands are.
    pub(crate) async fn run(
        &mut self,
        state: &SharedState,
        tasks: &TaskHandle,
        budget: usize,
        status: &QueueStatus,
    ) -> io::Result<CommandResult> {
        let mut turn = 0usize;
        loop {
            // The queue runs in bounded runs: a long one gives the loop a turn
            // of its own rather than running everything else late.
            turn += 1;
            if turn >= budget {
                turn = 0;
                yield_now().await;
            }
            let Some(item) = self.queue.start_next() else {
                return Ok(std::mem::replace(&mut self.out, CommandResult::ok("")));
            };
            let (command, source, source_depth, contributes_status) = match item {
                SharedQueueItem::Command {
                    command,
                    source,
                    source_depth,
                    contributes_status,
                } => (command, source, source_depth, contributes_status),
                SharedQueueItem::FinalizeSource { args } => {
                    let insert_next = self.plan_command_hooks("source-file", &args, None, state);
                    self.queue.complete(queue::QueueCompletion {
                        discard_group_tail: false,
                        insert_next,
                    });
                    continue;
                }
                SharedQueueItem::FinalizeHooks { command, args } => {
                    let insert_next = self.plan_command_hooks(command, &args, None, state);
                    self.queue.complete(queue::QueueCompletion {
                        discard_group_tail: false,
                        insert_next,
                    });
                    continue;
                }
                SharedQueueItem::NestedCommand {
                    queue: mut nested,
                    capture,
                } => {
                    // Erased, so this queue's future does not contain its own.
                    let running: Pin<Box<dyn Future<Output = io::Result<CommandResult>> + '_>> =
                        Box::pin(nested.run(state, tasks, budget, status));
                    let result = running.await?;
                    let stops_group = result.exit != 0 && !result.continue_queue;
                    self.capture_nested_result(result, capture);
                    self.queue.complete(queue::QueueCompletion {
                        discard_group_tail: stops_group,
                        insert_next: Vec::new(),
                    });
                    continue;
                }
                SharedQueueItem::EndHook { name } => {
                    {
                        let mut state = state.borrow_mut();
                        state.end_hook(&name);
                        state.record_control_checkpoint();
                    }
                    self.queue.complete(queue::QueueCompletion::done());
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

            let inflight = InflightCommand {
                command,
                source,
                source_depth,
                contributes_status,
            };
            // tmux's `notify_add` looks at the *global* queue's running item,
            // and `cmdq_running` masks an item that is waiting — so a hook
            // body drops only what is raised while it is actively on the
            // stack. The latch goes on around each poll, not around the whole
            // await: what another queue raises while this body is parked is
            // kept, queued behind it.
            let suppressed = self.context.frame.suppress_notifications;
            let item_formats = format::QueueItem {
                command: Some(inflight.command.spec.name),
                current_file: inflight
                    .source
                    .as_ref()
                    .map(|source| Rc::from(source.path.as_str()))
                    .or_else(|| self.context.frame.current_file.clone()),
            };
            // The command is the one that knows whether it waits, and for what.
            let mut execution = {
                let mut context = ExecContext {
                    state,
                    client: &self.context,
                    agents: &self.agents,
                    tasks,
                    status,
                    queue: self,
                    source_depth: inflight.source_depth,
                    args: &inflight.command.args,
                };
                let mut future =
                    std::pin::pin!(inflight.command.command.clone().execute(&mut context));
                std::future::poll_fn(|poll_context| {
                    let _formats = format::enter_queue_item(item_formats.clone());
                    if suppressed {
                        state.borrow_mut().begin_notification_suppression();
                    }
                    let poll = future.as_mut().poll(poll_context);
                    if suppressed {
                        state.borrow_mut().end_notification_suppression();
                    }
                    poll
                })
                .await
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
            self.finish_execution(inflight, execution, state);
        }
    }

    fn plan_nested_command_line(
        &self,
        line: &str,
        state: &SharedState,
        capture: NestedCapture,
    ) -> Result<Vec<SharedQueueItem>, CommandResult> {
        let aliases = {
            let state = state.borrow_mut();
            state.command_aliases()
        };
        let compiled = ExecutableCommand::compile(line, &aliases).map_err(|error| {
            let mut result = CommandResult::err(error);
            result.continue_queue = true;
            result
        })?;
        let mut nested_context = self.context.clone();
        nested_context.frame.current_file = format::queue_item_file();
        if matches!(capture, NestedCapture::Inserted | NestedCapture::Hook) {
            nested_context.frame.nested_granularity = true;
        }
        if matches!(capture, NestedCapture::Hook) {
            nested_context.frame.suppress_after_hooks = true;
        }
        Ok(compiled
            .split()
            .into_iter()
            .map(|command| SharedQueueItem::NestedCommand {
                queue: Box::new(ResumableCommandQueue::new(
                    command,
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
            let compiled = match command {
                DeferredCommand::Command(LazyCommand::Compiled(compiled)) => compiled,
                DeferredCommand::Command(LazyCommand::Line(line)) => {
                    ExecutableCommand::compile(&line, &aliases).map_err(CommandResult::err)?
                }
                DeferredCommand::Argv(args) => {
                    ExecutableCommand::compile_argv(&args, &aliases).map_err(CommandResult::err)?
                }
                DeferredCommand::Line { line, tail } => {
                    let mut compiled =
                        ExecutableCommand::compile(&line, &aliases).map_err(CommandResult::err)?;
                    if !tail.is_empty() {
                        compiled.extend(
                            ExecutableCommand::compile_argv(&tail, &aliases)
                                .map_err(CommandResult::err)?,
                        );
                    }
                    compiled
                }
            };
            planned.extend(compiled.into_commands().into_iter().map(|command| SharedQueueItem::Command {
                command,
                source: None,
                source_depth: 0,
                contributes_status: true,
            }));
        }
        Ok(planned)
    }

    fn finish_execution(
        &mut self,
        inflight: InflightCommand,
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
        if !self.context.frame.suppress_after_hooks {
            if stops_group {
                let lexed = ParsedArgs::lex(command.spec.name, &command.args);
                execution.insert_next.extend(self.plan_hook(
                    "command-error",
                    lexed.value('t'),
                    hook_command_vars("command-error", &command.args, &lexed),
                    state,
                ));
            } else if !execution.defer_success_hooks {
                execution.insert_next.extend(self.plan_command_hooks(
                    command.spec.name,
                    &command.args,
                    execution.result.after_hook_target.as_deref(),
                    state,
                ));
            }
        }
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

        self.queue.complete(queue::QueueCompletion {
            discard_group_tail: stops_group,
            insert_next: execution.insert_next,
        });
    }

    fn plan_command_hooks(
        &self,
        command: &str,
        args: &[String],
        settled_target: Option<&str>,
        state: &SharedState,
    ) -> Vec<Vec<SharedQueueItem>> {
        if self.context.frame.suppress_after_hooks {
            return Vec::new();
        }
        let after = format!("after-{command}");
        let lexed = ParsedArgs::lex(command, args);
        let vars = hook_command_vars(&after, args, &lexed);
        // tmux resolves an `after-*` body against the find-state the command
        // left behind, so a command that settled a new target — the window or
        // pane it just created — overrides its own `-t`.
        self.plan_hook(&after, settled_target.or(lexed.value('t')), vars, state)
    }

    fn plan_hook(
        &self,
        hook: &str,
        requested_target: Option<&str>,
        vars: Vec<(String, String)>,
        state: &SharedState,
    ) -> Vec<Vec<SharedQueueItem>> {
        self.plan_hook_with_capture(hook, requested_target, vars, state, NestedCapture::Hook)
    }

    /// Plan a *command* hook's body as items of this queue.
    ///
    /// Only a command's own hooks are planned here: tmux inserts those behind
    /// the triggering item in the same queue (`notify_hook`), while an event's
    /// hook belongs to the server-wide queue and never reaches this path.
    fn plan_hook_with_capture(
        &self,
        hook: &str,
        requested_target: Option<&str>,
        vars: Vec<(String, String)>,
        state: &SharedState,
        capture: NestedCapture,
    ) -> Vec<Vec<SharedQueueItem>> {
        let commands = {
            let mut state = {
                let state = state.borrow_mut();
                state
            };
            let previous = install_command_target_context(&mut state, &self.context);
            let commands =
                hook_commands(hook, requested_target, &mut state, HookOrigin::Command);
            restore_command_target_context(&mut state, previous);
            let Some(commands) = commands else {
                return Vec::new();
            };
            commands
        };

        let mut hook_context = self.context.clone();
        // A hook body resolves an untargeted command against the hook's own
        // target, not the server's current one; a hook without a target of its
        // own stays in the enclosing hook's scope.
        hook_context.frame.hook = Some(HookScope {
            vars: Rc::new(vars),
            target: requested_target.map(Rc::from).or_else(|| {
                self.context
                    .frame
                    .hook
                    .as_ref()
                    .and_then(|hook| hook.target.clone())
            }),
        });
        if matches!(capture, NestedCapture::Hook) {
            hook_context.frame.suppress_after_hooks = true;
            hook_context.frame.nested_granularity = true;
        }
        let mut groups = Vec::new();
        for body in commands {
            if body.is_empty() {
                continue;
            }
            groups.push(
                body.split()
                    .into_iter()
                    .map(|command| SharedQueueItem::NestedCommand {
                        queue: Box::new(ResumableCommandQueue::new(
                            command,
                            &self.agents,
                            &hook_context,
                        )),
                        capture,
                    })
                    .collect(),
            );
        }
        groups.push(vec![SharedQueueItem::EndHook {
            name: hook.to_string(),
        }]);
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

fn interaction_completion_result(completion: PromptCompletion) -> CommandResult {
    let mut completed = CommandResult {
        stdout: completion.stdout,
        stdout_bytes: Vec::new(),
        stderr: completion.stderr,
        exit: completion.exit,
        deferred_commands: Vec::new(),
        background_commands: Vec::new(),
        continue_queue: true,
        inserted_results: Vec::new(),
        control_flags: 1,
        after_hook_target: None,
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
fn hook_command_vars(hook: &str, args: &[String], lexed: &ParsedArgs) -> Vec<(String, String)> {
    let mut vars = vec![("hook".to_string(), hook.to_string())];
    let arguments = args.get(1..).unwrap_or_default().join(" ");
    vars.push(("hook_arguments".to_string(), arguments));
    for (letter, value) in lexed.flags() {
        vars.push((
            format!("hook_flag_{letter}"),
            value.unwrap_or("1").to_string(),
        ));
    }
    for (index, argument) in lexed.positionals().iter().enumerate() {
        vars.push((format!("hook_argument_{index}"), argument.clone()));
    }
    vars
}

/// The `after-*` hook of a command the client file protocol completed outside
/// the command queue (`save-buffer` to a client-side path), for the loop to run
/// as a detached queue. Whatever the command raised is on the server-wide
/// queue, which its own runner drains.
pub(crate) fn client_file_after_hooks(
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
    let lexed = ParsedArgs::lex(name, &normalized);
    let hook = format!("after-{name}");
    let vars = hook_command_vars(&hook, &normalized, &lexed);
    let mut requests = Vec::new();
    push_event_hook(&hook, lexed.value('t'), vars, st, context, &mut requests);
    requests
}

/// The bodies of the next item on the server-wide queue.
///
/// This is tmux's global queue running one `notify_callback`: the item is taken
/// off the queue and the hook's bodies are resolved and inserted where it was,
/// which is why the runner runs them before it takes the item after this one.
/// `None` means the queue is empty; an empty vector means the item's hook has
/// no body.
pub(crate) fn next_notification_hooks(
    st: &mut ServerState,
) -> Option<Vec<BackgroundCommandRequest>> {
    let notification = st.next_notification()?;
    let context = ClientContext::default();
    let mut requests = Vec::new();
    push_event_hook(
        &notification.name,
        notification.target.as_deref(),
        notification.vars,
        st,
        &context,
        &mut requests,
    );
    Some(requests)
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
    context.frame.hook = Some(HookScope {
        vars: Rc::new(vars),
        target: requested_target.map(Rc::from),
    });
    context.frame.suppress_after_hooks = true;
    context.frame.suppress_notifications = true;
    for command in commands {
        if command.is_empty() {
            continue;
        }
        requests.push(BackgroundCommandRequest::Command {
            command: LazyCommand::Compiled(command),
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

/// The compiled bodies a hook fires, in array-index order, resolved through the
/// target's own option layers.
///
/// The bodies were compiled when they were set, so a fire hands the queue
/// clones of the stored [`ExecutableCommand`]s rather than re-parsing text.
fn hook_commands(
    hook: &str,
    requested_target: Option<&str>,
    st: &mut ServerState,
    origin: HookOrigin,
) -> Option<Vec<ExecutableCommand>> {
    // A user hook is a plain string option rather than a catalogued command
    // array, so tmux compiles its body when the hook fires and drops a body
    // that does not compile.
    if hook.starts_with('@') {
        if matches!(origin, HookOrigin::Command) && !st.begin_hook(hook) {
            return None;
        }
        let target = requested_target
            .filter(|target| st.resolve(target).is_some())
            .map(str::to_string)
            .or_else(|| current_target(st));
        let body = target
            .as_deref()
            .and_then(|target| st.session_options(target).ok())
            .or_else(|| Some(st.global_session_options()))
            .and_then(|view| view.get(hook).map(str::to_string));
        let Some(body) = body else {
            return Some(Vec::new());
        };
        let aliases = st.command_aliases();
        return Some(
            ExecutableCommand::compile(&body, &aliases)
                .map(|command| vec![command])
                .unwrap_or_default(),
        );
    }
    let name = options::AnyHook::from_name(hook)?;
    if matches!(origin, HookOrigin::Command) && !st.begin_hook(hook) {
        return None;
    }
    let target = requested_target
        .filter(|target| st.resolve(target).is_some())
        .map(str::to_string)
        .or_else(|| current_target(st));
    let view = target
        .as_deref()
        .and_then(|target| match name.scope() {
            OptionScope::Session => st.session_options(target).ok(),
            OptionScope::Window => st.window_options(target).ok(),
            OptionScope::WindowPane => st.pane_options(target).ok(),
            OptionScope::Server => None,
        })
        // Nothing to read the hook through — the subject the event is about
        // was the last one there was. tmux's empty find state falls back to the
        // global session table, and has no window or pane table to fall back
        // to.
        .or_else(|| match name.scope() {
            OptionScope::Session => Some(st.global_session_options()),
            _ => None,
        });
    let commands = view
        .map(|view| {
            view.array_commands(name.as_str())
                .into_iter()
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    Some(commands)
}

/// What one already-parsed command executes against. Its arguments are its
/// own, so the context is only what the whole server offers every command.
struct CommandContext<'a> {
    state: &'a mut ServerState,
    client: &'a ClientContext,
    agents: &'a PaneAgents,
}

/// What a command executes against once the command queue is the task running
/// it.
///
/// The state is the shared cell rather than a borrow of it: a command that
/// waits has to let go of the state while it does, so it borrows in tight
/// scopes around its own awaits. [`ExecContext::sync`] is that scope for the
/// commands that never wait.
struct ExecContext<'a> {
    state: &'a SharedState,
    client: &'a ClientContext,
    agents: &'a PaneAgents,
    tasks: &'a TaskHandle,
    status: &'a QueueStatus,
    queue: &'a ResumableCommandQueue,
    /// How deep the `source-file` nesting that reached this command is.
    source_depth: u8,
    /// The command's own normalized argv. A command that inserts queue items
    /// still needs it: the `hook_*` format variables and the item that finishes
    /// a command's hooks are keyed by the words as written.
    args: &'a [String],
}

impl<'a> ExecContext<'a> {
    fn state(&self) -> &'a SharedState {
        self.state
    }

    fn client(&self) -> &'a ClientContext {
        self.client
    }

    fn agents(&self) -> &'a PaneAgents {
        self.agents
    }

    fn tasks(&self) -> &'a TaskHandle {
        self.tasks
    }

    fn args(&self) -> &'a [String] {
        self.args
    }

    fn source_depth(&self) -> u8 {
        self.source_depth
    }

    /// Run a command body against the state, under the client's command target
    /// context. Nothing awaits inside, so the borrow is this call.
    fn run_sync(
        &self,
        client: &ClientContext,
        run: impl FnOnce(&mut CommandContext<'_>) -> CommandResult,
    ) -> CommandResult {
        let mut state = self.state.borrow_mut();
        let previous = install_command_target_context(&mut state, client);
        let result = run(&mut CommandContext {
            state: &mut state,
            client,
            agents: self.agents,
        });
        state.record_control_checkpoint();
        restore_command_target_context(&mut state, previous);
        result
    }

    /// A command that finishes on its own: run it and report it as completed.
    fn sync(
        &self,
        run: impl FnOnce(&mut CommandContext<'_>) -> CommandResult,
    ) -> SharedCommandExecution {
        SharedCommandExecution::completed(self.run_sync(self.client, run))
    }

    /// Wait for an answer only another client can give, having already touched
    /// the registry that will deliver it.
    async fn wait_for_answer(
        &self,
        allows_attach_io: bool,
        start: SuspensionStart,
    ) -> CommandResult {
        wait_for_answer(self.status, allows_attach_io, start).await
    }

    /// Plan a command line this command runs as queue items of its own.
    fn plan_nested_command_line(
        &self,
        line: &str,
        capture: NestedCapture,
    ) -> Result<Vec<SharedQueueItem>, CommandResult> {
        self.queue
            .plan_nested_command_line(line, self.state, capture)
    }

    /// Plan a hook's body as queue items, the way an event or a command does.
    fn plan_hook_with_capture(
        &self,
        hook: &str,
        requested_target: Option<&str>,
        vars: Vec<(String, String)>,
        capture: NestedCapture,
    ) -> Vec<Vec<SharedQueueItem>> {
        self.queue
            .plan_hook_with_capture(hook, requested_target, vars, self.state, capture)
    }

    /// The queue item that runs a command's `after-*` hooks once the work it
    /// inserted has finished, rather than the moment the command returns.
    fn finalize_hooks(&self, command: &'static str) -> Vec<SharedQueueItem> {
        vec![SharedQueueItem::FinalizeHooks {
            command,
            args: self.args.to_vec(),
        }]
    }
}

// ---- individual commands ---------------------------------------------------

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

/// The `-O` half of [`list_sort_criteria`], for a command that already holds its
/// flags as typed arguments.
fn list_sort_order(order: Option<&str>) -> Result<Option<ListSortOrder>, CommandResult> {
    let Some(order) = order else {
        return Ok(None);
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
    Ok(Some(order))
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

/// tmux's `default_window_name`: stringify the pane's whole argument vector and
/// reduce it with `parse_window_name`, falling back to the shell the pane would
/// run when it was given no command of its own. Reducing the stringified vector
/// — rather than its first word — is what strips an `exec ` prefix and a login
/// shell's leading dash, and what keeps a relative path such as `bin/sleep`
/// whole.
///
/// `target` scopes the option lookups; a window being created has none of its
/// own yet, so its session names the same values.
pub(super) fn initial_window_name(st: &ServerState, target: &str, command: &[String]) -> String {
    let default_shell = st
        .option_for_target(target, "default-shell")
        .unwrap_or("/bin/sh");
    let default_command = st
        .option_for_target(target, "default-command")
        .unwrap_or("");
    let source = match command {
        [] if !default_command.is_empty() => default_command.to_string(),
        [] => default_shell.to_string(),
        command => crate::server::pane::stringify_argv(command),
    };
    crate::server::pane::parse_window_name(&source)
}

fn apply_initial_window_name(
    st: &mut ServerState,
    session: &str,
    window_position: usize,
    command: &[String],
) {
    let Some(link) = st
        .find(session)
        .and_then(|session| session.windows.get(window_position))
        .copied()
    else {
        return;
    };
    let target = format!("{session}:{}", link.index);
    let current = initial_window_name(st, &target, command);
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
    // Interactive attach classification happens before the command-client
    // parser sees the argv. Validate the single command here as well, so an
    // unknown flag or an extra operand cannot be swallowed by the attach path.
    if ExecutableCommand::compile_groups(vec![args], &[]).is_err() {
        return Intent::Command;
    }
    match canonical {
        "attach-session" => Intent::Attach,
        // `new-session -d` is a detached create → command path; otherwise attach.
        "new-session" if !command_flag("new-session", args, 'd') => Intent::NewAttach,
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
    let command = sessions::NewSession::parse(&ParsedArgs::lex("new-session", &args))?;
    command.create_for_attach(st, context)
}

fn pane_command_argv(command: &[String], st: &ServerState, target: Option<&str>) -> Vec<String> {
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
        [command] => vec![shell, "-c".to_string(), command.clone()],
        command => command.to_vec(),
    }
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

impl TreeLoops<'_> {
    /// The session an enclosing `#{S:…}` rebound to, or the anchor.
    fn scoped_session(&self, vars: &Vars) -> usize {
        vars.lookup("session_id")
            .and_then(|id| id.strip_prefix('$')?.parse::<u32>().ok())
            .and_then(|id| {
                self.st
                    .sessions()
                    .iter()
                    .position(|session| session.id == id)
            })
            .unwrap_or(self.session)
    }

    /// The position in `session`'s window list that an enclosing `#{W:…}`
    /// rebound to, or the anchor.
    fn scoped_window(&self, session: usize, vars: &Vars) -> usize {
        vars.lookup("window_id")
            .and_then(|id| id.strip_prefix('@')?.parse::<u32>().ok())
            .and_then(|id| {
                self.st.sessions()[session]
                    .windows
                    .iter()
                    .position(|link| link.id == id)
            })
            .unwrap_or(self.window)
    }
}

impl format::LoopSource for TreeLoops<'_> {
    fn items(&self, kind: format::LoopKind) -> Vec<Vars> {
        format::ScopedLoopSource::items_in_scope(
            self,
            match kind {
                format::LoopKind::Session => format::FormatLoopKind::Session,
                format::LoopKind::Window => format::FormatLoopKind::Window,
                format::LoopKind::Pane => format::FormatLoopKind::Pane,
            },
            "",
            &Vars::empty(),
        )
    }
}

impl format::ScopedLoopSource for TreeLoops<'_> {
    fn items_in_scope(
        &self,
        kind: format::FormatLoopKind,
        flags: &str,
        vars: &Vars,
    ) -> Vec<Vars> {
        match kind {
            format::FormatLoopKind::Session => {
                let marked = self.st.marked_pane();
                let mut order: Vec<&Session> = self.st.sessions().iter().collect();
                sort_session_loop(&mut order, flags);
                order
                    .iter()
                    .map(|s| vars_for(self.st, s, s.active, self.agents, marked))
                    .collect()
            }
            format::FormatLoopKind::Window => {
                let marked = self.st.marked_pane();
                let session = self.scoped_session(vars);
                let sess = &self.st.sessions()[session];
                let mut order = (0..sess.windows.len()).collect::<Vec<_>>();
                sort_window_loop(self.st, sess, &mut order, flags);
                order
                    .into_iter()
                    .map(|w| {
                        let mut item = vars_for(self.st, sess, w, self.agents, marked);
                        set_window_neighbour_vars(self.st, session, w, &mut item);
                        item
                    })
                    .collect()
            }
            format::FormatLoopKind::Pane => {
                let marked = self.st.marked_pane();
                let session = self.scoped_session(vars);
                let window = self.scoped_window(session, vars);
                let sess = &self.st.sessions()[session];
                if sess.windows.get(window).is_none() {
                    return Vec::new();
                }
                let mut order =
                    (0..self.st.session_window(sess, window).panes.len()).collect::<Vec<_>>();
                sort_pane_loop(self.st.session_window(sess, window), &mut order, flags);
                order
                    .into_iter()
                    .map(|p| vars_full(self.st, sess, window, p, self.agents, marked))
                    .collect()
            }
            format::FormatLoopKind::Client => {
                let mut clients = self.st.client_snapshots();
                sort_client_loop(&mut clients, flags);
                clients
                    .iter()
                    .filter_map(|client| clients::client_vars(self.st, self.agents, client))
                    .collect()
            }
        }
    }
}

/// tmux's `#{L:…}` order: the client list's own order unless a flag names a
/// key, with the client name breaking every tie.
pub(super) fn sort_client_loop(
    order: &mut [super::state::ClientSnapshot],
    flags: &str,
) {
    if flags.contains('n') || flags.contains('t') {
        order.sort_by(|left, right| {
            let key = if flags.contains('n') {
                left.name.cmp(&right.name)
            } else {
                // Most recent first, as `sort_client_cmp` inverts this.
                right.activity_micros.cmp(&left.activity_micros)
            };
            key.then_with(|| left.name.cmp(&right.name))
        });
    }
    if flags.contains('r') {
        order.reverse();
    }
}

/// tmux's `#{S:…}` order: `SORT_INDEX` (the session id) unless a flag names
/// another key, with the session name breaking every tie and `r` reversing the
/// result.
pub(super) fn sort_session_loop(order: &mut [&Session], flags: &str) {
    order.sort_by(|left, right| {
        let key = if flags.contains('n') {
            left.name.cmp(&right.name)
        } else if flags.contains('t') {
            // Most recent first, as `sort_session_cmp` inverts this comparison.
            right.activity_micros.cmp(&left.activity_micros)
        } else {
            left.id.cmp(&right.id)
        };
        key.then_with(|| left.name.cmp(&right.name))
    });
    if flags.contains('r') {
        order.reverse();
    }
}

/// tmux's `#{W:…}` order: the session's own window order unless a flag names a
/// key, with the window name breaking every tie. `SORT_ORDER` skips the sort
/// entirely and `r` only reverses the list.
pub(super) fn sort_window_loop(
    st: &ServerState,
    session: &Session,
    order: &mut [usize],
    flags: &str,
) {
    let sorted = flags.contains('n') || flags.contains('t');
    if sorted {
        order.sort_by(|left, right| {
            let (left, right) = (
                st.window_for_link(&session.windows[*left]),
                st.window_for_link(&session.windows[*right]),
            );
            let key = if flags.contains('n') {
                left.name.cmp(&right.name)
            } else {
                right.activity_micros.cmp(&left.activity_micros)
            };
            key.then_with(|| left.name.cmp(&right.name))
        });
    }
    if flags.contains('r') {
        order.reverse();
    }
}

/// tmux's `#{P:…}` order: always `SORT_CREATION`, the pane id, with the pane
/// title breaking a tie. The `i`, `n` and `t` flags name no key here; only `r`
/// is read.
pub(super) fn sort_pane_loop(window: &super::state::Window, order: &mut [usize], flags: &str) {
    order.sort_by_key(|pane| window.panes[*pane].id);
    if flags.contains('r') {
        order.reverse();
    }
}

/// The neighbour variables tmux's `format_loop_windows` adds on top of a window
/// entry's own: whether the entry sits next to the current window, and the
/// index, active flag and user options of the entries either side of it.
pub(super) fn set_window_neighbour_vars(
    st: &ServerState,
    session: usize,
    window: usize,
    vars: &mut Vars,
) {
    let sess = &st.sessions()[session];
    let count = sess.windows.len();
    vars.set(
        "window_after_active",
        if window > 0 && window - 1 == sess.active {
            "1"
        } else {
            "0"
        },
    )
    .set(
        "window_before_active",
        if window + 1 < count && window + 1 == sess.active {
            "1"
        } else {
            "0"
        },
    );
    for (prefix, neighbour) in [
        ("next", (window + 1 < count).then_some(window + 1)),
        ("prev", window.checked_sub(1)),
    ] {
        let Some(neighbour) = neighbour else {
            continue;
        };
        vars.set(
            format!("{prefix}_window_index"),
            sess.windows[neighbour].index.to_string(),
        )
        .set(
            format!("{prefix}_window_active"),
            if neighbour == sess.active { "1" } else { "0" },
        );
        for (name, value) in st
            .session_window(sess, neighbour)
            .own_options()
            .filter(|(name, _)| name.starts_with('@'))
        {
            vars.set(format!("{prefix}_{name}"), value.to_string());
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
        .map(|_| {
            let window = st.session_window(sess, win_idx);
            st.command_pane_index(window).unwrap_or(window.active)
        })
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
        // The pinned tmux 3.7b oracle is built `--disable-sixel`, and hmux
        // implements no sixel at all. The daemon also loads no config file
        // (see README.md on server capability gaps).
        .set("sixel_support", "0")
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
        .set(
            "session_path",
            sess.cwd()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_else(current_dir),
        )
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
        let is_last = Some(win_idx) == sess.last_active();
        let active_sessions = st.window_active_session_list(win.id);
        let window_flags = st.printable_window_flags(sess, win_idx, true);
        let window_raw_flags = st.printable_window_flags(sess, win_idx, false);
        v.set("window_name", win.name.clone());
        for (name, value) in win.own_options() {
            v.set(name.to_string(), value.to_string());
        }
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
            .set(
                "window_activity",
                (win.activity_micros / 1_000_000).to_string(),
            )
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
                // The pid of the process the pane forked for its pty; zero for a
                // pane that never had a child of its own, as tmux reports.
                .set(
                    "pane_pid",
                    p.pane
                        .child_pid()
                        .map_or_else(|| "0".to_string(), |pid| pid.to_string()),
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
                .set("cursor_x", p.pane.cursor_position().0.to_string())
                .set("cursor_y", p.pane.cursor_position().1.to_string())
                .set("cursor_character", pane_cursor_character(&p.pane))
                // tmux drops history rows past `history-limit` as they scroll
                // off, so the size saturates there. The limit is applied again
                // here, where the rows are counted and read back.
                .set(
                    "history_size",
                    p.pane.scrollback_rows().min(history_limit).to_string(),
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
            // `cd`s. A pane with no cwd to read — one with no child, one whose
            // child has exited, one whose child has not taken the pty yet —
            // reports nothing, as stock tmux does: its callback returns NULL
            // when `osdep_get_cwd` fails, which leaves the variable unset
            // rather than substituting the server's own cwd. The `/proc` reads
            // behind both variables run only if a format names them.
            let probe = p.pane.process_probe();
            {
                let probe = probe.clone();
                v.set_lazy("pane_current_path", move || {
                    probe
                        .as_ref()
                        .and_then(|probe| probe.current_path())
                        .unwrap_or_default()
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
                    .set("top_line_time", copy.top_line_time().to_string())
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
                if let Some(count) = copy.search_count() {
                    v.set("search_count", count.to_string());
                }
                if let Some(row) = copy.grid.rows.get(copy.cursor.row) {
                    let line = row
                        .cells
                        .iter()
                        .filter(|cell| !matches!(cell.width, CellWidth::SpacerTail))
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
        .set("cursor_very_visible", flag(modes.cursor_very_visible))
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
    let (x, y) = pane.cursor_position();
    let history = pane.scrollback_rows();
    pane.dump_plain_row(history + y as usize)
        .lines()
        .next()
        .and_then(|line| line.chars().nth(x as usize))
        .map_or_else(|| " ".to_string(), |character| character.to_string())
}

/// The server process's working directory. This backs `#{session_path}` and the
/// `#{pane_start_path}` of a pane spawned without an explicit `-c`. It is the
/// directory hmux was started in — the same cwd stock tmux inherits, so both
/// targets report the same path. Empty on failure.
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

/// tmux's `file_get_path`: expand a leading `~/` against the home directory,
/// then resolve any remaining relative path against the command client's
/// working directory.
pub(crate) fn client_file_path(path: &str, context: &ClientContext) -> PathBuf {
    let expanded = match path.strip_prefix("~/") {
        Some(rest) => {
            let home = std::env::var("HOME").unwrap_or_default();
            PathBuf::from(format!("{home}/{rest}"))
        }
        None => PathBuf::from(path),
    };
    if expanded.is_absolute() {
        expanded
    } else {
        context
            .cwd
            .as_deref()
            .unwrap_or_else(|| Path::new("."))
            .join(expanded)
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
    let lexed = ParsedArgs::lex(spec.name, &normalize_argv(spec.name, args));
    if matches!(spec.name, "display-message" | "split-window" | "new-pane") {
        return lexed.has('I').then(|| PathBuf::from("-"));
    }
    if spec.name == "source-file" {
        // `-` is the client's own standard input; tmux keeps it verbatim and
        // reads it over the file protocol rather than opening it server-side.
        return lexed
            .positionals()
            .iter()
            .any(|path| path == "-")
            .then(|| PathBuf::from("-"));
    }
    if spec.name != "load-buffer" {
        return None;
    }
    let path = lexed.positionals().first()?;
    if path == "-" {
        return Some(PathBuf::from(path));
    }
    Some(client_file_path(path, context))
}

pub(crate) struct ClientFileWrite {
    pub(crate) path: PathBuf,
    pub(crate) display_path: String,
    pub(crate) flags: i32,
    pub(crate) data: Vec<u8>,
}

pub(crate) fn save_buffer_client_request(
    args: &[String],
    state: &mut ServerState,
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
    let command = buffers::SaveBuffer::parse(&ParsedArgs::lex(spec.name, &normalized)).ok()?;
    let previous = install_command_target_context(state, context);
    let result = command.client_request(state, context);
    restore_command_target_context(state, previous);
    result
}

/// Render an I/O error the way tmux does — `strerror(errno)` — by trimming Rust's
/// trailing ` (os error N)` from the [`std::io::Error`] display.
pub(crate) fn io_error_message(e: &std::io::Error) -> String {
    let s = e.to_string();
    match s.find(" (os error") {
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
        .set("buffer_sample", buffer_sample(data))
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

fn buffer_sample(data: &[u8]) -> String {
    let sample = &data[..data.len().min(200)];
    let mut escaped = String::new();
    for character in String::from_utf8_lossy(sample).chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{07}' => escaped.push_str("\\a"),
            '\u{0b}' => escaped.push_str("\\v"),
            '\u{0c}' => escaped.push_str("\\f"),
            character if character.is_ascii_control() => {
                escaped.push_str(&format!("\\{:03o}", character as u32));
            }
            character => escaped.push(character),
        }
    }
    if data.len() > 200 || escaped.len() > 200 {
        while !escaped.is_char_boundary(200.min(escaped.len())) {
            escaped.pop();
        }
        escaped.truncate(200.min(escaped.len()));
        escaped.push_str("...");
    }
    escaped
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
        if state.append_view_output(&target, &output).is_err() && target != suspend::VIEW_FALLBACK {
            let _ = state.append_view_output(suspend::VIEW_FALLBACK, &output);
        }
    }
}

/// Resolve a binding whose real command sits behind an `if-shell -F` guard,
/// the shape the default mouse bindings take: `if -F ... {send -M}
/// {copy-mode -M}`, where the branch decides between a client-local outcome
/// (entering copy mode, resizing) and an ordinary command. Resolving the
/// condition before dispatch is what lets the attach loop keep handling those
/// outcomes itself instead of losing them inside the command interpreter.
///
/// The chosen branch comes back as the text it was written as — tmux compiles
/// a string branch when it runs it, so a `;` inside the branch splits there and
/// nowhere earlier. `None` means the binding is not a client-side guard at all
/// and stands exactly as it was bound; a guard that chose no branch answers
/// with the empty line, which runs nothing.
pub(super) fn resolve_conditional_binding(
    command: &[String],
    st: &mut ServerState,
    agents: &PaneAgents,
    context: &ClientContext,
) -> Option<String> {
    let mut words = command.to_vec();
    let mut resolved = None;
    // Bounded: a binding that somehow nests conditionals forever must not hang
    // the client's input loop.
    for _ in 0..4 {
        if !matches!(words.first().map(String::as_str), Some("if-shell" | "if")) {
            break;
        }
        let args = ParsedArgs::lex("if-shell", &normalize_argv("if-shell", &words));
        if !args.has('F') || args.has('b') {
            break;
        }
        let positional = args.positionals();
        let Some(condition) = positional.first() else {
            break;
        };
        let previous = st.replace_command_mouse(context.mouse.clone());
        let expanded = execution::expand_if_cond(condition, args.value('t'), st, agents);
        st.replace_command_mouse(previous);
        let branch = if format::is_true_first_byte(&expanded) {
            positional.get(1)
        } else {
            positional.get(2)
        };
        let branch = branch.map(|line| (*line).to_string()).unwrap_or_default();
        words = binding_words(&branch);
        resolved = Some(branch);
    }
    resolved
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

/// Rewrite a command's argv into a canonical form the flag helpers understand:
/// every short flag becomes its own `-x` token and every flag value becomes a
/// separate following token. This lets a single scanner absorb tmux's getopt
/// surface — clustered booleans (`-ga`), attached values (`-t0` / `-F#{x}`), and
/// separate values (`-t 0`) — so the per-command handlers only ever see the
/// simple `-x [value]` shape. Tokens after `--`, positionals, and args of a
/// command with no modeled `spec` pass through unchanged. Assumes flags already
/// validated by [`unknown_flag`], so an unrecognized letter is passed through
/// rather than erroring again.
pub(crate) fn normalize_argv(name: &str, args: &[String]) -> Vec<String> {
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

/// Whether a raw client argv gives one boolean flag of the command it names.
///
/// The attach path reads a couple of flags before a command line has been
/// parsed at all, so it lexes the single command it is looking at.
pub(crate) fn command_flag(name: &str, args: &[String], flag: char) -> bool {
    ParsedArgs::lex(name, &normalize_argv(name, args)).has(flag)
}

pub(crate) fn command_value(name: &str, args: &[String], flag: char) -> Option<String> {
    ParsedArgs::lex(name, &normalize_argv(name, args))
        .value(flag)
        .map(str::to_string)
}

pub(crate) fn command_positional(name: &str, args: &[String], index: usize) -> Option<String> {
    ParsedArgs::lex(name, &normalize_argv(name, args))
        .positionals()
        .get(index)
        .cloned()
}

/// Recreate the environment a tmux server gives a newly spawned pane. Wrapping
/// with `env -i` avoids mutating the multithreaded daemon process environment
/// while preserving the complete environment sent in identify frames.
/// The command inside the wrap [`pane_argv`] built, so a respawn that reuses a
/// pane's saved argv rebuilds one environment rather than nesting a second wrap
/// around the first — where the inner assignments would still win.
pub(super) fn unwrap_pane_argv(argv: Vec<String>) -> Vec<String> {
    let Some(rest) = argv
        .strip_prefix(std::slice::from_ref(&"/usr/bin/env".to_string()))
        .and_then(|rest| rest.strip_prefix(std::slice::from_ref(&"-i".to_string())))
    else {
        return argv;
    };
    let start = rest
        .iter()
        .position(|word| !word.contains('='))
        .unwrap_or(rest.len());
    rest[start..].to_vec()
}

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

    fn queued_command() -> (
        QueuedCommand,
        Rc<QueueStatus>,
        crate::sync::CompletionSender<io::Result<CommandResult>>,
    ) {
        let (completion, sender) = crate::sync::completion_pair().expect("completion pair");
        let status = Rc::new(QueueStatus::default());
        (
            QueuedCommand::new(completion, Rc::clone(&status)),
            status,
            sender,
        )
    }

    /// A template naming no variable skips the variable table entirely, so
    /// what it prints comes from a path the expander never walks. The skip is
    /// only sound if it is invisible: these are the shapes that reach it, and
    /// each must read exactly as it would have after a full expansion.
    #[test]
    fn a_template_that_names_no_variable_expands_to_itself() {
        let st = state();

        for literal in ["bench", " ", "no vars here", "trailing-brace}", "a{b}c"] {
            let out = run_str(&st, &["display-message", "-p", literal]);
            assert_eq!(
                out.stdout,
                format!("{literal}\n"),
                "literal {literal:?} did not print itself"
            );
        }

        // The neighbouring path still expands, so the skip cannot be masking a
        // table that stopped being built at all.
        let named = run_str(&st, &["display-message", "-p", "#{session_name}"]);
        assert_eq!(named.stdout, format!("{}\n", st.borrow().sessions()[0].name));
    }

    /// A wake that counts how many times it fired.
    fn counting_wake() -> (WakeFn, Rc<Cell<u32>>) {
        let fired = Rc::new(Cell::new(0));
        let counter = Rc::clone(&fired);
        (Rc::new(move || counter.set(counter.get() + 1)), fired)
    }

    #[test]
    fn a_queued_command_wakes_its_owner_when_what_it_is_parked_on_changes() {
        // A running task is opaque to its owner, so the one thing an owner has
        // to learn while the queue is still parked — that answering it is the
        // owner's own job — reaches it as a wake rather than as a value.
        let (mut queued, status, _sender) = queued_command();
        let (wake, fired) = counting_wake();

        queued.set_status_wake(&wake);
        status.set_allows_attach_io(true);

        assert_eq!(fired.get(), 1, "the parked suspension changed");
        assert!(queued.allows_attach_io());
    }

    #[test]
    fn an_unchanged_attach_io_flag_leaves_its_owner_alone() {
        // The queue sets the flag on both sides of every suspension, so only a
        // change is worth a turn of the owner's.
        let (mut queued, status, _sender) = queued_command();
        let (wake, fired) = counting_wake();
        queued.set_status_wake(&wake);

        status.set_allows_attach_io(false);

        assert_eq!(fired.get(), 0);
    }

    #[test]
    fn a_queue_parked_on_a_prompt_says_so_before_it_finishes() {
        // A waiting `command-prompt` is answered by the very client whose queue
        // is parked on it, so that client has to learn it is the one being
        // asked while the queue is still parked. A client that had stopped
        // reading its own input would never answer.
        let state = state();
        // The prompt goes to a client, so there has to be one for the queue to
        // wait on rather than to fail against.
        let _client = {
            let state = state.borrow_mut();
            let registry = state.client_prompt_registry();
            registry
                .attach("/dev/pts/0".to_string(), Some(1), 0)
                .expect("prompt client")
        };
        let context = ClientContext {
            kind: ClientKind::Command,
            ..ClientContext::default()
        };
        let queue = match start_resumable_command_argv(
            &["command-prompt".to_string()],
            &state,
            &PaneAgents::new(),
            &context,
        ) {
            Ok(queue) => queue,
            Err(result) => panic!("command-prompt parsed: {}", result.stderr),
        };

        let runtime = hmux_rt::TaskRuntime::new().expect("task runtime");
        // The first turn is taken inline, which is what starting a queue from a
        // client's own dispatch does.
        let queued = spawn_queue(&runtime.handle(), queue, Rc::clone(&state), 64)
            .expect("spawn command queue");
        assert!(
            queued.allows_attach_io(),
            "the queue is parked on a prompt the owner is the one to answer"
        );
    }

    #[test]
    fn capture_style_tracks_full_state_across_physical_rows() {
        let rows = panes::capture_vt_normalize_rows(
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
        let rows = panes::capture_vt_normalize_rows(
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

    /// `grid_string_cells` appends the OSC 8 that closes a row's link to the
    /// buffer the last cell's sequences are still sitting in, so a row whose
    /// last cell is the one that opened the link ends with that cell's
    /// sequences a second time. A row whose last cell only continued the link
    /// does not: its buffer was rewritten empty.
    #[test]
    fn a_row_ending_in_the_cell_that_opened_its_link_repeats_that_cell() {
        let opened_last = panes::capture_vt_normalize_rows(
            b"\x1b[31mA\x1b]8;;https://example.test\x1b\\B\x1b]8;;https://example.test\x1b\\\x1b]8;;\x1b\\",
            1,
        );
        assert_eq!(
            opened_last[0],
            concat!(
                "\x1b[31mA",
                "\x1b]8;;https://example.test\x1b\\B",
                "\x1b]8;;https://example.test\x1b\\",
                "\x1b]8;;\x1b\\"
            )
        );

        let continued_last = panes::capture_vt_normalize_rows(
            b"\x1b[31mA\x1b]8;;https://example.test\x1b\\BC\x1b]8;;\x1b\\",
            1,
        );
        assert_eq!(
            continued_last[0],
            concat!(
                "\x1b[31mA",
                "\x1b]8;;https://example.test\x1b\\BC",
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
    fn if_shell_format_leading_zero_runs_else() {
        let st = state();
        // Only the first byte decides: "0abc" starts with '0', so it is falsey
        // even though `format_true` (used by `-f` filters) would accept it.
        let r = run_str(
            &st,
            &["if-shell", "-F", "0abc", "kill-server", "new-window -t 0:"],
        );
        assert_eq!(r.exit, 0, "stderr={:?}", r.stderr);
        let lw = run_str(&st, &["list-windows", "-t", "0", "-F", "#{window_index}"]);
        assert_eq!(lw.stdout, "0\n1\n", "got {:?}", lw.stdout);
    }

    #[test]
    fn if_shell_format_leading_space_zero_runs_then() {
        let st = state();
        let r = run_str(&st, &["if-shell", "-F", " 0", "new-window -t 0:"]);
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
    fn kill_pane_all_unzooms_window() {
        let st = state();
        run_str(&st, &["split-window", "-t", "0"]);
        run_str(&st, &["resize-pane", "-Z", "-t", "0.0"]);
        let dm = run_str(
            &st,
            &["display-message", "-p", "-t", "0.0", "#{window_zoomed_flag}"],
        );
        assert_eq!(dm.stdout, "1\n");
        assert_eq!(run_str(&st, &["kill-pane", "-a", "-t", "0.0"]).exit, 0);
        let dm_after = run_str(
            &st,
            &["display-message", "-p", "-t", "0.0", "#{window_zoomed_flag}"],
        );
        assert_eq!(dm_after.stdout, "0\n");
    }

    #[test]
    fn switch_client_pane_target_unzooms_without_z() {
        let st = state();
        let _attached = {
            let guard = st.borrow_mut();
            let registry = guard.client_render_registry();
            let session_id = guard.session_id("0").unwrap();
            registry.attach(session_id, "test-client".into()).unwrap()
        };
        run_str(&st, &["split-window", "-t", "0"]);
        run_str(&st, &["resize-pane", "-Z", "-t", "0.0"]);
        let dm = run_str(
            &st,
            &["display-message", "-p", "-t", "0.0", "#{window_zoomed_flag}"],
        );
        assert_eq!(dm.stdout, "1\n");
        assert_eq!(
            run_str(&st, &["switch-client", "-c", "test-client", "-t", "0.1"]).exit,
            0
        );
        let dm_after = run_str(
            &st,
            &["display-message", "-p", "-t", "0.0", "#{window_zoomed_flag}"],
        );
        assert_eq!(dm_after.stdout, "0\n");
    }

    #[test]
    fn switch_client_pane_target_preserves_zoom_with_z() {
        let st = state();
        let _attached = {
            let guard = st.borrow_mut();
            let registry = guard.client_render_registry();
            let session_id = guard.session_id("0").unwrap();
            registry.attach(session_id, "test-client".into()).unwrap()
        };
        run_str(&st, &["split-window", "-t", "0"]);
        run_str(&st, &["resize-pane", "-Z", "-t", "0.0"]);
        let dm = run_str(
            &st,
            &["display-message", "-p", "-t", "0.0", "#{window_zoomed_flag}"],
        );
        assert_eq!(dm.stdout, "1\n");
        assert_eq!(
            run_str(&st, &["switch-client", "-Z", "-c", "test-client", "-t", "0.1"]).exit,
            0
        );
        let dm_after = run_str(
            &st,
            &["display-message", "-p", "-t", "0.0", "#{window_zoomed_flag}"],
        );
        assert_eq!(dm_after.stdout, "1\n");
    }

    #[test]
    fn resize_pane_directional_unzooms_window() {
        let st = state();
        run_str(&st, &["split-window", "-t", "0"]);
        run_str(&st, &["resize-pane", "-Z", "-t", "0.0"]);
        let dm = run_str(
            &st,
            &["display-message", "-p", "-t", "0.0", "#{window_zoomed_flag}"],
        );
        assert_eq!(dm.stdout, "1\n");
        assert_eq!(run_str(&st, &["resize-pane", "-t", "0.0", "-U", "1"]).exit, 0);
        let dm_after = run_str(
            &st,
            &["display-message", "-p", "-t", "0.0", "#{window_zoomed_flag}"],
        );
        assert_eq!(dm_after.stdout, "0\n");
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
    fn save_buffer_expands_format_path() {
        let st = state();
        run_str(&st, &["new-session", "-d", "-s", "s1"]);
        run_str(&st, &["set-option", "-g", "@save_name", "my_expanded.txt"]);
        run_str(&st, &["set-buffer", "-b", "x", "payload"]);
        let dir = std::env::temp_dir();
        let path_template = format!("{}/#{{@save_name}}", dir.display());
        let expected_path = dir.join("my_expanded.txt");
        let r = run_str(&st, &["save-buffer", "-b", "x", &path_template]);
        assert_eq!(r.exit, 0);
        assert_eq!(std::fs::read_to_string(&expected_path).unwrap(), "payload");
        let _ = std::fs::remove_file(&expected_path);
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
        let spec = sessions::NewSession::parse(&ParsedArgs::lex("new-session", &args))
            .expect("new-session arguments")
            .pane_spec(&st, &ClientContext::default());
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
    fn list_commands_empty_format_prints_nothing() {
        let st = state();
        // `cmd-list-commands.c` only prints a line when the expansion is
        // non-empty, so an empty template suppresses every row.
        let r = run_str(&st, &["list-commands", "-F", "", "new-window"]);
        assert_eq!(r.exit, 0);
        assert_eq!(r.stdout, "");

        let r = run_str(&st, &["list-commands", "-F", ""]);
        assert_eq!(r.exit, 0);
        assert_eq!(r.stdout, "");

        // A template that is empty for only some commands keeps the others.
        let r = run_str(&st, &["list-commands", "-F", "#{command_list_alias}"]);
        assert_eq!(r.exit, 0);
        assert!(r.stdout.lines().all(|line| !line.is_empty()));
        assert!(r.stdout.lines().any(|line| line == "has"));
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
