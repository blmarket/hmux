//! Attach-client actions and key-binding resolution.

use crate::integration::status::StatusHub;

use super::super::command;
use super::super::key::{KeyCode, basic_key_bytes, key_from_byte, parse_key_name};
use super::super::mouse::MouseEvent;
use super::super::state::{ClientKey, PromptCompletion, PromptReply, SharedState};
use super::copy_mode::{self, CopyModeAction};
use super::{client_key_table, decode_tty_key, is_configured_prefix};

/// A destructive binding that tmux guards behind a `confirm-before` `(y/n)`
/// prompt. The attach loop shows the prompt and only runs the action once the
/// user answers `y`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ConfirmAction {
    /// `C-b x` -> `confirm-before ... kill-pane`.
    KillPane,
    /// `C-b &` -> `confirm-before ... kill-window`.
    KillWindow,
    Command(Vec<String>),
}

pub(super) struct ActiveConfirm {
    pub(super) prompt: String,
    pub(super) action: ConfirmAction,
    pub(super) confirm_key: u8,
    pub(super) default_yes: bool,
    pub(super) reply: Option<PromptReply>,
}

pub(super) enum ConfirmResolution {
    Complete,
    Deferred {
        command: Vec<String>,
        reply: Option<PromptReply>,
    },
}

impl ActiveConfirm {
    pub(super) fn resolve(
        self,
        accepted: bool,
        state: &SharedState,
        target: &str,
        cols: u16,
        pane_rows: u16,
    ) -> ConfirmResolution {
        let result = if accepted {
            match self.action {
                ConfirmAction::Command(command) => {
                    return ConfirmResolution::Deferred {
                        command,
                        reply: self.reply,
                    };
                }
                action @ (ConfirmAction::KillPane | ConfirmAction::KillWindow) => {
                    let killed = {
                        let mut state = state.borrow_mut();
                        let killed = match action {
                            ConfirmAction::KillPane => state.kill_pane(target).is_ok(),
                            ConfirmAction::KillWindow => state.kill_window(target).is_ok(),
                            ConfirmAction::Command(_) => unreachable!(),
                        };
                        if killed && state.find(target).is_some() {
                            let _ = state.resize_session(target, cols, pane_rows);
                        }
                        killed
                    };
                    if killed {
                        command::CommandResult::ok("")
                    } else {
                        command::CommandResult::err("")
                    }
                }
            }
        } else {
            command::CommandResult::err("")
        };
        if let Some(reply) = self.reply {
            reply.send(Some(PromptCompletion {
                stdout: result.stdout,
                stderr: result.stderr,
                exit: result.exit,
                inserted: accepted,
            }));
        }
        ConfirmResolution::Complete
    }
}

/// What a resolved key binding tells the attach loop to do.
pub(super) enum PrefixOutcome {
    /// Detach the client (`C-b d`), ending the attach loop.
    Detach,
    /// Send a literal prefix byte to the pane (`C-b C-b`, i.e. `send-prefix`).
    SendPrefix(Vec<u8>),
    /// Enter the attached client's copy-mode view, optionally paging up.
    CopyMode(CopyModeAction),
    /// Raise a `confirm-before` prompt. The loop shows `prompt` in the status
    /// line and waits for the answer before running `action`.
    Confirm {
        prompt: String,
        action: ConfirmAction,
    },
    Prompt {
        args: Vec<String>,
    },
    DeferredCommand {
        command: command::LazyCommand,
        context: command::ClientContext,
    },
    DeferredMessage {
        command: command::LazyCommand,
        context: command::ClientContext,
        target: String,
        escape_hashes: bool,
        explicit_duration: Option<u64>,
    },
    /// The binding was refused before it ran. The loop shows the reason the
    /// way it shows a command's own error.
    Error(String),
    /// The binding ran (or was a no-op / unknown key). `changed` is true when it
    /// altered the window/pane layout, so the compositor must redraw.
    Handled {
        changed: bool,
    },
}

/// Resolve and execute one key from an attached client's active table.
///
/// Ordinary commands run through the command interpreter in the attached
/// client's context. Client-local operations return an outcome for the attach
/// loop to apply.
pub(super) fn dispatch_key_binding(
    table: &str,
    key: KeyCode,
    state: &SharedState,
    target: &str,
    hub: &StatusHub,
    context: &command::ClientContext,
    mouse: Option<MouseEvent>,
) -> PrefixOutcome {
    let binding = {
        let st = state.borrow_mut();
        st.key_binding(table, key).cloned()
    };
    let Some(binding) = binding else {
        return PrefixOutcome::Handled { changed: false };
    };
    // tmux's `key_bindings_dispatch` decides this before the binding is looked
    // at any further, so a read-only client cannot reach even the bindings the
    // attached client answers itself.
    if context.read_only && !binding.command.all_read_only() {
        return PrefixOutcome::Error("client is read-only".to_string());
    }
    // The attached client answers a few bindings itself rather than queueing
    // them, and decides from the words; the compiled body prints back as the
    // argv the command path would have re-parsed; what dispatch defers is the
    // binding as it was bound.
    let mut words = binding.command.argv();
    let mut deferred = command::LazyCommand::Compiled(binding.command);
    // The default mouse bindings guard their real command behind `if-shell -F`,
    // so resolve that here: the branch may be a client-local outcome the
    // command interpreter has no way to express.
    if matches!(words.first().map(String::as_str), Some("if-shell" | "if")) {
        let mut binding_context = context.clone();
        binding_context.key_event = Some(key);
        binding_context.mouse = mouse.clone();
        let agents = hub.snapshot().panes;
        let branch = {
            let mut st = state.borrow_mut();
            command::resolve_conditional_binding(&words, &mut st, &agents, &binding_context)
        };
        if let Some(branch) = branch {
            words = command::binding_words(&branch);
            deferred = command::LazyCommand::Line(branch);
        }
    }
    let Some(command_name) = words.first().map(String::as_str) else {
        return PrefixOutcome::Handled { changed: false };
    };

    match command_name {
        "detach-client" | "detach" => return PrefixOutcome::Detach,
        "send-prefix" => {
            let option = if words.iter().any(|word| word == "-2") {
                "prefix2"
            } else {
                "prefix"
            };
            let bytes = {
                let st = state.borrow_mut();
                st.option_for_target(target, option)
                    .or_else(|| super::super::options::option_default(option))
                    .and_then(parse_key_name)
            }
            .and_then(basic_key_bytes)
            .unwrap_or_default();
            return PrefixOutcome::SendPrefix(bytes);
        }
        "copy-mode" if !words.iter().any(|word| word == ";") => {
            return PrefixOutcome::CopyMode(CopyModeAction::new(
                words.iter().any(|word| word == "-u"),
                words.iter().any(|word| word == "-d"),
                words.iter().any(|word| word == "-S"),
                mouse,
                words.iter().any(|word| word == "-M"),
                words.iter().any(|word| word == "-e"),
            ));
        }
        "command-prompt" => {
            return PrefixOutcome::Prompt {
                args: words.clone(),
            };
        }
        "display-message"
            if !words.iter().any(|word| word == ";") && !words.iter().any(|word| word == "-p") =>
        {
            let mut argv = words.clone();
            argv.insert(1, "-p".to_string());
            let explicit_duration = words
                .windows(2)
                .find(|words| words[0] == "-d")
                .and_then(|words| words[1].parse::<u64>().ok());
            let aliases = {
                let st = state.borrow_mut();
                st.command_aliases()
            };
            let Ok(compiled) = command::ExecutableCommand::compile_argv(&argv, &aliases) else {
                return PrefixOutcome::Handled { changed: false };
            };
            let mut binding_context = context.clone();
            binding_context.key_event = Some(key);
            binding_context.mouse = mouse;
            return PrefixOutcome::DeferredMessage {
                command: command::LazyCommand::Compiled(compiled),
                context: binding_context,
                target: target.to_string(),
                escape_hashes: words.iter().any(|word| word == "-N"),
                explicit_duration,
            };
        }
        "confirm-before" | "confirm" if words.last().is_some_and(|word| word == "kill-window") => {
            let name = state
                .borrow_mut()
                .active_window_name(target)
                .unwrap_or_default();
            return PrefixOutcome::Confirm {
                prompt: format!("kill-window {name}? (y/n)"),
                action: ConfirmAction::KillWindow,
            };
        }
        "confirm-before" | "confirm" if words.last().is_some_and(|word| word == "kill-pane") => {
            let idx = state.borrow_mut().active_pane_index(target).unwrap_or(0);
            return PrefixOutcome::Confirm {
                prompt: format!("kill-pane {idx}? (y/n)"),
                action: ConfirmAction::KillPane,
            };
        }
        _ => {}
    }

    let mut binding_context = context.clone();
    binding_context.key_event = Some(key);
    binding_context.mouse = mouse;
    PrefixOutcome::DeferredCommand {
        command: deferred,
        context: binding_context,
    }
}

pub(in crate::server) fn dispatch_control_client_keys(
    keys: &[ClientKey],
    prefix_pending: &mut bool,
    state: &SharedState,
    target: &str,
    hub: &StatusHub,
    context: &command::ClientContext,
    deferred: &mut Vec<command::BackgroundCommandRequest>,
) -> bool {
    for injected in keys {
        let bytes = &injected.bytes;
        let mut index = 0;
        while index < bytes.len() {
            let start = index;
            let (key, consumed) = decode_tty_key(&bytes[index..])
                .map(|(decoded, consumed)| (decoded.code, consumed))
                .unwrap_or_else(|| (Some(key_from_byte(bytes[index])), 1));
            index += consumed;
            let Some(key) = key else {
                continue;
            };
            if is_configured_prefix(&state.borrow_mut(), target, key) {
                *prefix_pending = true;
                continue;
            }

            let from_prefix = std::mem::take(prefix_pending);
            let table = if from_prefix {
                "prefix".to_string()
            } else if state.borrow_mut().copy_mode_active(target) {
                copy_mode::key_table(&state.borrow_mut(), target).to_string()
            } else {
                client_key_table(&state.borrow_mut(), target)
            };
            let bound = state.borrow_mut().key_binding(&table, key).is_some();
            if !bound {
                if !from_prefix && injected.forward_unbound {
                    {
                        let state = state.borrow_mut();
                        let _ = state.input_to_active_pane(target, &bytes[start..index]);
                    }
                }
                continue;
            }

            match dispatch_key_binding(&table, key, state, target, hub, context, None) {
                PrefixOutcome::Detach => return true,
                PrefixOutcome::SendPrefix(bytes) => {
                    let state = state.borrow_mut();
                    let _ = state.input_to_active_pane(target, &bytes);
                }
                PrefixOutcome::CopyMode(action) => action.apply(state, target),
                PrefixOutcome::DeferredCommand { command, context }
                | PrefixOutcome::DeferredMessage {
                    command, context, ..
                } => {
                    deferred.push(command::BackgroundCommandRequest::Command { command, context });
                }
                // A read-only control client never reaches the refusal here:
                // the `send-keys` that injected these keys checked the target
                // client's own read-only flag first.
                PrefixOutcome::Confirm { .. }
                | PrefixOutcome::Prompt { .. }
                | PrefixOutcome::Error(_)
                | PrefixOutcome::Handled { .. } => {}
            }
        }
    }
    false
}

/// Resolve one prefix key and run whatever binding it deferred, standing in
/// for the attach loop's continuation. Test scaffolding.
#[cfg(test)]
pub(super) fn dispatch_prefix_key(
    key: u8,
    state: &SharedState,
    target: &str,
    cols: u16,
    pane_rows: u16,
) -> PrefixOutcome {
    let current_session_id = state.borrow_mut().session_id(target);
    let context = command::ClientContext {
        current_session_id,
        ..command::ClientContext::default()
    };
    let hub = StatusHub::new();
    let outcome = dispatch_key_binding(
        "prefix",
        key_from_byte(key),
        state,
        target,
        &hub,
        &context,
        None,
    );
    let PrefixOutcome::DeferredCommand { command, context } = outcome else {
        return outcome;
    };
    let result = command::run_lazy_with_context(command, state, &hub.snapshot().panes, &context);
    let changed = result.exit == 0;
    if changed {
        let mut st = state.borrow_mut();
        let _ = st.resize_session(target, cols, pane_rows);
    }
    PrefixOutcome::Handled { changed }
}
