//! `send-keys` and `send-prefix`.
//!
//! This follows tmux's `cmd-send-keys.c` organization: parse the repeat prefix
//! once, route `-X` to the active mode, and otherwise encode each semantic key
//! before falling back to literal text. Client (`-K`) and mouse (`-M`) injection
//! remain separate routing concerns rather than being folded into pane byte
//! encoding.

use std::ffi::CString;
use std::process::{Command, Stdio};

use crate::integration::status::PaneAgents;

use super::command::args::ParsedArgs;
use super::command::{ClientContext, CommandResult};
use super::format;
use super::input_keys::{PaneKey, PaneKeyEncoding};
use super::key::{parse_key_name, KeyBase, KeyCode, Modifiers, SpecialKey};
use super::options;
use super::state::{ClientKey, KeyBinding, ServerState};
use hmux_vt::{encode_key_default_modes, Key, KeyEvent};

/// `send-keys [-FHKlMRX] [-c target-client] [-N repeat-count] [-t target-pane] [key ...]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct SendKeys {
    /// `-t`: the pane the keys reach.
    target: Option<String>,
    /// `-c`: the client `-K` sends to, and whose read-only flag is consulted.
    client: Option<String>,
    /// `-N`: the repeat count, expanded as a format when the command runs.
    repeat: Option<String>,
    /// `-l`: send the operands as literal text rather than key names.
    literal: bool,
    /// `-H`: the operands are hexadecimal byte values.
    hex: bool,
    /// `-X`: the operands name a copy-mode command instead of keys.
    copy_mode: bool,
    /// `-M`: forward the triggering mouse event to its pane.
    mouse: bool,
    /// `-R`: reset the pane's terminal state first.
    reset: bool,
    /// `-K`: send to the client rather than into the pane.
    to_client: bool,
    /// `-F`: expand the copy-mode search argument as a format.
    expand: bool,
    /// The keys, or — under `-X` — the copy-mode command and its arguments.
    operands: Vec<String>,
}

impl SendKeys {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            target: args.value('t').map(str::to_string),
            client: args.value('c').map(str::to_string),
            repeat: args.value('N').map(str::to_string),
            literal: args.has('l'),
            hex: args.has('H'),
            copy_mode: args.has('X'),
            mouse: args.has('M'),
            reset: args.has('R'),
            to_client: args.has('K'),
            expand: args.has('F'),
            operands: args.positionals().to_vec(),
        })
    }
}

/// `send-prefix [-2] [-t target-pane]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct SendPrefix {
    /// `-2`: send the secondary prefix key.
    secondary: bool,
    /// `-t`: the pane the prefix key reaches.
    target: Option<String>,
}

impl SendPrefix {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            secondary: args.has('2'),
            target: args.value('t').map(str::to_string),
        })
    }
}

/// Execute `send-keys`.
pub(in crate::server) fn exec(
    command: &SendKeys,
    state: &mut ServerState,
    agents: &PaneAgents,
    context: &ClientContext,
) -> CommandResult {
    let target = command
        .target
        .clone()
        .or_else(|| current_target(state));
    let target = match target {
        Some(target) => target,
        None => return CommandResult::err("no current target\n"),
    };
    if state.resolve(&target).is_none() {
        return CommandResult::err(format!("{}\n", state.pane_target_error(&target)));
    }
    let target_client_read_only = command
        .client
        .as_deref()
        .and_then(|client| state.client_read_only(Some(client), context.tty_name.as_deref()))
        .unwrap_or(false);
    if (context.read_only || target_client_read_only) && !command.copy_mode {
        return CommandResult::err("client is read-only\n");
    }

    let repeat = match repeat_count(command, state, &target, agents) {
        Ok(repeat) => repeat,
        Err(error) => return CommandResult::err(error),
    };
    let operands = &command.operands;
    if command.repeat.is_some() && (command.copy_mode || operands.is_empty()) {
        let _ = state.set_copy_mode_prefix(&target, repeat);
    }

    if command.copy_mode {
        return send_copy_mode_command(
            command,
            state,
            agents,
            context,
            &target,
            context.read_only || target_client_read_only,
        );
    }
    if command.mouse {
        let Some((pane_id, event)) = context
            .mouse
            .as_ref()
            .and_then(super::mouse::MouseEvent::pane_input_event)
        else {
            return CommandResult::err("no mouse target\n");
        };
        let mouse_target = format!("%{pane_id}");
        return match state.input_mouse_to_pane(&mouse_target, event) {
            Ok(()) => CommandResult::ok(""),
            Err(_) => CommandResult::err("no mouse target\n"),
        };
    }
    if command.reset {
        let _ = state.reset_pane_terminal(&target);
    }

    if command.to_client {
        return send_client_keys(command, state, context, repeat);
    }
    if operands.is_empty() {
        if command.repeat.is_some() || command.reset {
            return CommandResult::ok("");
        }
        let Some(key) = context.key_event else {
            return CommandResult::ok("");
        };
        let mut bytes = Vec::new();
        let mut mode_bindings = Vec::new();
        inject_parsed_key(state, &target, key, &mut bytes, &mut mode_bindings);
        if !bytes.is_empty() {
            let result = write_pane_input(state, &target, &bytes);
            if result.exit != 0 {
                return result;
            }
        }
        return run_mode_bindings(&target, mode_bindings);
    }

    let mut bytes = Vec::new();
    let mut mode_bindings = Vec::new();
    for _ in 0..repeat {
        for operand in operands {
            inject_string(
                state,
                &target,
                operand,
                command.literal,
                command.hex,
                &mut bytes,
                &mut mode_bindings,
            );
        }
    }
    if !bytes.is_empty() {
        let result = write_pane_input(state, &target, &bytes);
        if result.exit != 0 {
            return result;
        }
    }
    run_mode_bindings(&target, mode_bindings)
}

fn send_client_keys(
    command: &SendKeys,
    state: &ServerState,
    context: &ClientContext,
    repeat: u32,
) -> CommandResult {
    let operands = &command.operands;
    if operands.is_empty() && (command.repeat.is_some() || command.reset) {
        return CommandResult::ok("");
    }

    let mut keys = Vec::new();
    if operands.is_empty() {
        if let Some(key) = context.key_event {
            if let Some(key) = encode_parsed_key_for_client(key) {
                keys.push(key);
            }
        }
    } else {
        for _ in 0..repeat {
            for operand in operands {
                inject_client_string(operand, command.literal, command.hex, &mut keys);
            }
        }
    }
    if !keys.is_empty() {
        let _ = state.send_client_keys(
            command.client.as_deref(),
            context.tty_name.as_deref(),
            keys,
        );
    }
    CommandResult::ok("")
}

/// Execute `send-prefix`, which shares tmux's semantic key injection path with
/// `send-keys`.
pub(in crate::server) fn exec_prefix(
    command: &SendPrefix,
    state: &mut ServerState,
    context: &ClientContext,
) -> CommandResult {
    let target = command.target.clone().or_else(|| current_target(state));
    let target = match target {
        Some(target) => target,
        None => return CommandResult::err("no current target\n"),
    };
    if context.read_only {
        return CommandResult::err("client is read-only\n");
    }
    if state.resolve(&target).is_none() {
        return CommandResult::err(format!("{}\n", state.pane_target_error(&target)));
    }

    let option = if command.secondary { "prefix2" } else { "prefix" };
    let key = state
        .option_for_target(&target, option)
        .or_else(|| options::option_default(option))
        .unwrap_or("None");
    if key == "None" {
        return CommandResult::ok("");
    }
    let Some(bytes) = encode_key_for_pane(state, &target, key) else {
        return CommandResult::ok("");
    };
    write_pane_input(state, &target, &bytes)
}

fn send_copy_mode_command(
    send: &SendKeys,
    state: &mut ServerState,
    agents: &PaneAgents,
    context: &ClientContext,
    target: &str,
    read_only: bool,
) -> CommandResult {
    let copy_args = send
        .operands
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let command = copy_args.first().copied().unwrap_or("");
    let copy_positionals = copy_args
        .iter()
        .skip(1)
        .copied()
        .filter(|value| !matches!(*value, "-C" | "-P" | "--"))
        .collect::<Vec<_>>();
    let set_paste = !copy_args.contains(&"-P");
    let set_clipboard = !copy_args.contains(&"-C");
    if !state.copy_mode_active(target) {
        return CommandResult::err("not in a mode\n");
    }
    if read_only && !copy_mode_command_is_read_only(command) {
        return CommandResult::ok("");
    }
    if command.is_empty() {
        return CommandResult::ok("");
    }
    let repeat = match state.take_copy_mode_prefix(target) {
        Ok(repeat) => repeat,
        Err(error) => return CommandResult::err(format!("{error}\n")),
    };
    let argument = match copy_args.get(1).copied() {
        Some("--") => copy_args.get(2).copied(),
        argument => argument,
    };
    let expanded_argument = if send.expand
        && matches!(
            command,
            "search-forward" | "search-backward" | "search-forward-text" | "search-backward-text"
        ) {
        argument.map(|argument| {
            let resolved = state
                .resolve(target)
                .expect("send-keys target was resolved before format expansion");
            let mut vars = super::command::vars_full(
                state,
                &state.sessions()[resolved.session],
                resolved.window,
                resolved.pane,
                agents,
                state.marked_pane(),
            );
            for (name, value) in state.env_iter() {
                vars.set(name.to_string(), value);
            }
            if let Ok(entries) = state.format_option_entries(target) {
                for (name, value) in entries {
                    vars.set(name.to_string(), value);
                }
            }
            format::expand(argument, &vars)
        })
    } else {
        argument.map(str::to_string)
    };
    let vi = match state.option_for_target(target, "mode-keys") {
        Some(mode) => mode == "vi",
        None => options::mode_keys_default() == "vi",
    };
    let separators = state
        .option_for_target(target, "word-separators")
        .unwrap_or(" !\"#$%&'()*+,-./:;<=>?@[\\]^`{|}~")
        .to_string();
    if let Some(mouse) = context.mouse.as_ref() {
        let position = mouse.pane_position();
        if !matches!(
            mouse.kind,
            super::mouse::MouseEventKind::WheelUp | super::mouse::MouseEventKind::WheelDown
        ) {
            let _ = state.position_copy_cursor_from_mouse(target, position.x, position.y, vi);
        }
        if command == "scroll-to-mouse"
            && state
                .scroll_copy_to_mouse(
                    target,
                    // The slider drag works in screen rows, since the grab
                    // offset it holds on to is measured against the pane.
                    mouse.position.y,
                    mouse
                        .target
                        .as_ref()
                        .and_then(|target| target.slider_offset),
                    vi,
                    copy_args.contains(&"-e"),
                )
                .unwrap_or(false)
        {
            return CommandResult::ok("");
        }
    }

    let mut client_output = String::new();
    let pipe_environment = context.with_job_environment(state).environment;
    match state.copy_mode_command_with_prefix(
        target,
        command,
        expanded_argument.as_deref(),
        repeat,
        vi,
        &separators,
    ) {
        Ok(Some(selection)) => {
            let is_pipe = command.contains("pipe")
                || matches!(command, "pipe" | "pipe-no-clear" | "pipe-and-cancel");
            if is_pipe {
                let pipe_command = copy_positionals
                    .first()
                    .copied()
                    .or_else(|| state.server_options().get("copy-command"))
                    .unwrap_or("");
                if !pipe_command.is_empty() {
                    // tmux's `cmd_copy_pipe` runs the command as a job, so it
                    // sees the same environment a pane would.
                    let mut shell = Command::new("sh");
                    shell
                        .arg("-c")
                        .arg(pipe_command)
                        .stdin(Stdio::piped())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .env_clear();
                    for entry in &pipe_environment {
                        if let Some((name, value)) = entry.split_once('=') {
                            shell.env(name, value);
                        }
                    }
                    if let Ok(mut child) = shell.spawn() {
                        if let Some(stdin) = child.stdin.take() {
                            // The loop writes the selection as the filter
                            // takes it; a big selection and a slow reader
                            // would otherwise stall every client.
                            if let Ok(job) = super::pane::PanePipeIo::for_write(
                                child,
                                stdin,
                                selection.as_bytes(),
                            ) {
                                state.adopt_pipe(job);
                            }
                        }
                    }
                }
            }
            let sets_buffer =
                set_paste && !matches!(command, "pipe" | "pipe-no-clear" | "pipe-and-cancel");
            if sets_buffer {
                let buffer_name = if is_pipe {
                    copy_positionals.get(1).copied()
                } else {
                    copy_positionals.first().copied()
                };
                if command.starts_with("append-selection") {
                    state.append_buffer(buffer_name, selection.as_bytes());
                } else {
                    state.add_buffer_with_prefix(buffer_name, selection.as_bytes());
                }
            }
            if set_clipboard
                && !matches!(command, "pipe" | "pipe-no-clear" | "pipe-and-cancel")
                && context.key_event.is_some()
                && state.server_options().get("set-clipboard") != Some("off")
            {
                client_output = format!("\x1b]52;c;{}\x07", base64_encode(selection.as_bytes()));
            }
        }
        Ok(None) => {}
        Err(error) => return CommandResult::err(format!("{error}\n")),
    }
    CommandResult::ok(client_output)
}

fn copy_mode_command_is_read_only(command: &str) -> bool {
    matches!(
        command,
        "back-to-indentation"
            | "bottom-line"
            | "cancel"
            | "cursor-down"
            | "cursor-down-and-cancel"
            | "cursor-left"
            | "cursor-right"
            | "cursor-up"
            | "cursor-centre-vertical"
            | "cursor-centre-horizontal"
            | "end-of-line"
            | "goto-line"
            | "halfpage-down"
            | "halfpage-down-and-cancel"
            | "halfpage-up"
            | "history-bottom"
            | "history-top"
            | "jump-to-mark"
            | "next-prompt"
            | "previous-prompt"
            | "middle-line"
            | "next-matching-bracket"
            | "next-paragraph"
            | "next-space"
            | "next-space-end"
            | "next-word"
            | "next-word-end"
            | "page-down"
            | "page-down-and-cancel"
            | "page-up"
            | "previous-matching-bracket"
            | "previous-paragraph"
            | "previous-space"
            | "previous-word"
            | "recentre-top-bottom"
            | "refresh-from-pane"
            | "scroll-bottom"
            | "scroll-down"
            | "scroll-down-and-cancel"
            | "scroll-middle"
            | "scroll-to-mouse"
            | "scroll-top"
            | "scroll-up"
            | "set-mark"
            | "start-of-line"
            | "toggle-position"
            | "top-line"
    )
}

pub(crate) fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let a = chunk[0];
        let b = chunk.get(1).copied().unwrap_or(0);
        let c = chunk.get(2).copied().unwrap_or(0);
        out.push(TABLE[(a >> 2) as usize] as char);
        out.push(TABLE[(((a & 3) << 4) | (b >> 4)) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(((b & 15) << 2) | (c >> 6)) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(c & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn inject_client_string(
    operand: &str,
    literal: bool,
    hex_literal: bool,
    output: &mut Vec<ClientKey>,
) {
    if hex_literal {
        if let Some(bytes) = encode_hex_literal(operand) {
            output.push(ClientKey {
                bytes,
                forward_unbound: true,
            });
        }
        return;
    }
    if !literal {
        if let Some(key) = parse_key_name(operand) {
            if matches!(key.base, KeyBase::None) {
                push_client_literal(operand, output);
            } else if let Some(key) = encode_parsed_key_for_client(key) {
                output.push(key);
            }
            return;
        }
    }
    push_client_literal(operand, output);
}

fn push_client_literal(text: &str, output: &mut Vec<ClientKey>) {
    output.extend(text.chars().map(|ch| ClientKey {
        bytes: ch.to_string().into_bytes(),
        forward_unbound: true,
    }));
}

/// tmux's `cmd_send_keys_inject_string`: recognize a named key first, then
/// treat an unrecognized operand as literal UTF-8 text.
fn inject_string(
    state: &ServerState,
    target: &str,
    operand: &str,
    literal: bool,
    hex_literal: bool,
    output: &mut Vec<u8>,
    mode_bindings: &mut Vec<KeyBinding>,
) {
    if hex_literal {
        if let Some(bytes) = encode_hex_literal(operand) {
            let key = literal_key(char::from(bytes[0]));
            if !route_mode_key(state, target, key, mode_bindings) {
                output.extend_from_slice(&bytes);
            }
        }
        return;
    }
    if !literal {
        if operand.starts_with("0x") {
            if let Some(bytes) = encode_hex_key(operand) {
                let key = if bytes.len() == 1 {
                    literal_key(char::from(bytes[0]))
                } else {
                    let ch = std::str::from_utf8(&bytes).ok().and_then(|text| {
                        let mut chars = text.chars();
                        let ch = chars.next()?;
                        chars.next().is_none().then_some(ch)
                    });
                    match ch {
                        Some(ch) => literal_key(ch),
                        None => return,
                    }
                };
                if !route_mode_key(state, target, key, mode_bindings) {
                    output.extend_from_slice(&bytes);
                }
                return;
            }
        }
        if let Some(key) = parse_key_name(operand) {
            match key.base {
                // tmux deliberately excludes KEYC_NONE from the recognized-key
                // path, so the spelling "None" falls back to literal text.
                KeyBase::None | KeyBase::Mouse(_) => {}
                KeyBase::Any | KeyBase::User(_) => {
                    // Internal keys are valid names but have no pane encoding.
                    // An active mode still gets the opportunity to bind them.
                    let _ = route_mode_key(state, target, key, mode_bindings);
                    return;
                }
                _ => {
                    if inject_parsed_key(state, target, key, output, mode_bindings) {
                        return;
                    }
                    // tmux falls through to the literal spelling for a key its
                    // pane encoder had no form for, so `send-keys C-Escape`
                    // types those nine characters rather than doing nothing.
                }
            }
        }
    }
    for ch in operand.chars() {
        if !route_mode_key(state, target, literal_key(ch), mode_bindings) {
            let mut encoded = [0; 4];
            output.extend_from_slice(ch.encode_utf8(&mut encoded).as_bytes());
        }
    }
}

/// Returns whether the key was fully accounted for. A false leaves the caller
/// to fall back to tmux's literal spelling of the key name.
fn inject_parsed_key(
    state: &ServerState,
    target: &str,
    key: KeyCode,
    output: &mut Vec<u8>,
    mode_bindings: &mut Vec<KeyBinding>,
) -> bool {
    if matches!(key.base, KeyBase::Any | KeyBase::User(_)) {
        let _ = route_mode_key(state, target, key, mode_bindings);
        return true;
    }
    if matches!(key.base, KeyBase::None | KeyBase::Mouse(_)) {
        return true;
    }
    if route_mode_key(state, target, key, mode_bindings) {
        return true;
    }
    let encoding = encode_parsed_key_for_pane(state, target, key);
    // Whatever the encoder managed before giving up still goes out, exactly as
    // tmux's writes to the pane are not unwound by a later failure.
    output.extend_from_slice(&encoding.bytes);
    encoding.complete
}

fn literal_key(ch: char) -> KeyCode {
    KeyCode {
        base: KeyBase::Char(ch),
        modifiers: Modifiers::default(),
    }
}

fn route_mode_key(
    state: &ServerState,
    target: &str,
    key: KeyCode,
    mode_bindings: &mut Vec<KeyBinding>,
) -> bool {
    let vi = state
        .option_for_target(target, "mode-keys")
        .map_or_else(|| options::mode_keys_default() == "vi", |mode| mode == "vi");
    match state.pane_mode_key_binding(target, key, vi) {
        Ok(Some(binding)) => {
            if let Some(binding) = binding {
                mode_bindings.push(binding);
            }
            true
        }
        Ok(None) | Err(_) => false,
    }
}

fn run_mode_bindings(target: &str, bindings: Vec<KeyBinding>) -> CommandResult {
    let mut output = CommandResult::ok("");
    for binding in bindings {
        let mut command = binding.command;
        if command
            .first()
            .is_some_and(|name| matches!(name.as_str(), "send-keys" | "send"))
            && !command.iter().any(|word| word == "-t")
        {
            command.splice(1..1, ["-t".to_string(), target.to_string()]);
        }
        // Handed back to the queue that owns this command rather than run
        // here: a mode binding is an ordinary command line, and running one
        // under the caller's state borrow is what forced the blocking paths.
        output
            .deferred_commands
            .push(super::command::DeferredCommand::Args(command));
    }
    output
}

fn write_pane_input(state: &mut ServerState, target: &str, bytes: &[u8]) -> CommandResult {
    match state.input_to_pane(target, bytes) {
        Ok(()) => CommandResult::ok(""),
        Err(_) => CommandResult::err(format!("{}\n", state.pane_target_error(target))),
    }
}

fn repeat_count(
    command: &SendKeys,
    state: &ServerState,
    target: &str,
    agents: &PaneAgents,
) -> Result<u32, String> {
    let Some(value) = command.repeat.as_deref() else {
        return Ok(1);
    };
    let resolved = state
        .resolve(target)
        .expect("send-keys target was resolved before repeat expansion");
    let vars = super::command::vars_full(
        state,
        &state.sessions()[resolved.session],
        resolved.window,
        resolved.pane,
        agents,
        state.marked_pane(),
    );
    let value = format::expand(value, &vars);
    // tmux's strtonum path accepts leading C whitespace and a leading plus,
    // rejects trailing whitespace, and classifies numeric under/overflow
    // separately from malformed input.
    let value = value.trim_start_matches(|ch: char| ch.is_ascii_whitespace());
    match value.parse::<i128>() {
        Ok(value) if value < 1 => Err("repeat count too small\n".to_string()),
        Ok(value) if value > i128::from(u32::MAX) => Err("repeat count too large\n".to_string()),
        Ok(value) => Ok(value as u32),
        Err(_) => {
            let (negative, digits) = value
                .strip_prefix('-')
                .map_or((false, value), |digits| (true, digits));
            let digits = digits.strip_prefix('+').unwrap_or(digits);
            if !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()) {
                if negative {
                    Err("repeat count too small\n".to_string())
                } else {
                    Err("repeat count too large\n".to_string())
                }
            } else {
                Err("repeat count invalid\n".to_string())
            }
        }
    }
}

fn encode_key_for_pane(state: &ServerState, target: &str, name: &str) -> Option<Vec<u8>> {
    if name.starts_with("0x") {
        return encode_hex_key(name);
    }
    let code = parse_key_name(name)?;
    let encoding = encode_parsed_key_for_pane(state, target, code);
    encoding.complete.then_some(encoding.bytes)
}

/// Encode a key named on a `send-keys` command line for its pane.
///
/// The name carries the application cursor/keypad flags of tmux's key-string
/// table, so `Up` and `KP1` mean "whichever form this pane is asking for".
fn encode_parsed_key_for_pane(state: &ServerState, target: &str, code: KeyCode) -> PaneKeyEncoding {
    state
        .encode_pane_key(target, PaneKey::from_name(code))
        .unwrap_or_default()
}

fn encode_parsed_key_for_client(code: KeyCode) -> Option<ClientKey> {
    let bytes = with_key_event(code, encode_key_default_modes)?;
    Some(ClientKey {
        bytes,
        forward_unbound: matches!(code.base, KeyBase::Char(_)),
    })
}

fn with_key_event<T>(code: KeyCode, encode: impl FnOnce(KeyEvent<'_>) -> T) -> Option<T> {
    let mut shift = code.modifiers.shift();
    let (key, text, unshifted) = match code.base {
        KeyBase::Char('\u{1b}') => (Key::ESCAPE, None, None),
        KeyBase::Char('\t') => (Key::TAB, None, None),
        KeyBase::Char('\r' | '\n') => (Key::ENTER, None, None),
        KeyBase::Char(ch) => {
            let text = if shift && ch.is_ascii_alphabetic() {
                ch.to_ascii_uppercase().to_string()
            } else {
                ch.to_string()
            };
            (
                Key::from_ascii(ch),
                Some(text),
                Some(ch.to_ascii_lowercase()),
            )
        }
        KeyBase::Special(SpecialKey::F(number)) => (Key::function(number)?, None, None),
        KeyBase::Special(SpecialKey::Insert) => (Key::INSERT, None, None),
        KeyBase::Special(SpecialKey::Delete) => (Key::DELETE, None, None),
        KeyBase::Special(SpecialKey::Home) => (Key::HOME, None, None),
        KeyBase::Special(SpecialKey::End) => (Key::END, None, None),
        KeyBase::Special(SpecialKey::PageDown) => (Key::PAGE_DOWN, None, None),
        KeyBase::Special(SpecialKey::PageUp) => (Key::PAGE_UP, None, None),
        KeyBase::Special(SpecialKey::BackTab) => {
            shift = true;
            (Key::TAB, None, None)
        }
        KeyBase::Special(SpecialKey::Backspace) => (Key::BACKSPACE, None, None),
        KeyBase::Special(SpecialKey::Up) => (Key::ARROW_UP, None, None),
        KeyBase::Special(SpecialKey::Down) => (Key::ARROW_DOWN, None, None),
        KeyBase::Special(SpecialKey::Left) => (Key::ARROW_LEFT, None, None),
        KeyBase::Special(SpecialKey::Right) => (Key::ARROW_RIGHT, None, None),
        KeyBase::Special(SpecialKey::Keypad(ch)) if ch.is_ascii_digit() => {
            (Key::numpad_digit(ch)?, None, None)
        }
        KeyBase::Special(SpecialKey::Keypad('/')) => (Key::NUMPAD_DIVIDE, None, None),
        KeyBase::Special(SpecialKey::Keypad('*')) => (Key::NUMPAD_MULTIPLY, None, None),
        KeyBase::Special(SpecialKey::Keypad('-')) => (Key::NUMPAD_SUBTRACT, None, None),
        KeyBase::Special(SpecialKey::Keypad('+')) => (Key::NUMPAD_ADD, None, None),
        KeyBase::Special(SpecialKey::Keypad('.')) => (Key::NUMPAD_DECIMAL, None, None),
        KeyBase::Special(SpecialKey::Keypad(_)) => return None,
        KeyBase::Special(SpecialKey::KeypadEnter) => (Key::NUMPAD_ENTER, None, None),
        // The paste markers are bracketing, not characters: nothing in a copy
        // mode or a mode tree has a key for them.
        KeyBase::Special(SpecialKey::PasteStart | SpecialKey::PasteEnd) => return None,
        KeyBase::User(_) | KeyBase::Mouse(_) | KeyBase::Any | KeyBase::None => return None,
    };
    Some(encode(KeyEvent {
        key,
        shift,
        control: code.modifiers.ctrl(),
        alt: code.modifiers.meta(),
        text: text.as_deref(),
        unshifted_codepoint: unshifted,
    }))
}

fn encode_hex_key(key: &str) -> Option<Vec<u8>> {
    let input = CString::new(key.strip_prefix("0x")?).ok()?;
    let mut value: libc::c_uint = 0;
    // SAFETY: `input` and the format are NUL-terminated C strings, and `%x`
    // expects exactly the `unsigned int` pointer supplied here.
    let converted = unsafe { libc::sscanf(input.as_ptr(), c"%x".as_ptr(), &mut value) };
    if converted != 1 {
        return None;
    }
    if value < 32 {
        return Some(vec![value as u8]);
    }
    let scalar = char::from_u32(value)?;
    let mut encoded = [0; 4];
    Some(scalar.encode_utf8(&mut encoded).as_bytes().to_vec())
}

fn encode_hex_literal(key: &str) -> Option<Vec<u8>> {
    let input = CString::new(key).ok()?;
    let mut end = std::ptr::null_mut();
    // SAFETY: `input` is NUL-terminated and `end` points to storage for the
    // parser's end pointer.
    let value = unsafe { libc::strtol(input.as_ptr(), &mut end, 16) };
    let consumed_all = end.cast_const() == unsafe { input.as_ptr().add(key.len()) };
    if key.is_empty() || !consumed_all || !(0..=0xff).contains(&value) {
        return None;
    }
    Some(vec![value as u8])
}

fn current_session(state: &ServerState) -> Option<String> {
    state
        .command_session_name()
        .or_else(|| state.sessions().last().map(|session| session.name.clone()))
}

/// The target `send-keys` uses without a `-t`: the current session's current
/// window and active pane. The `:` keeps the session name a session — a bare
/// `0` is pane 0 of the current window in tmux's target grammar.
fn current_target(state: &ServerState) -> Option<String> {
    current_session(state).map(|session| format!("{session}:"))
}

#[cfg(test)]
mod tests {
    use super::{encode_hex_key, encode_hex_literal, inject_client_string, repeat_count, SendKeys};
    use crate::integration::status::PaneAgents;
    use crate::server::command::args::ParsedArgs;
    use crate::server::state::ServerState;

    fn send_keys(value: &str) -> SendKeys {
        let argv = ["send-keys", "-N", value, "x"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        SendKeys::parse(&ParsedArgs::lex("send-keys", &argv)).expect("send-keys arguments")
    }

    #[test]
    fn repeat_count_matches_tmux_bounds() {
        let state = ServerState::with_test_session().expect("state");
        let agents = PaneAgents::new();
        let repeat = |value| repeat_count(&send_keys(value), &state, "0", &agents);
        let maximum = u32::MAX.to_string();
        assert_eq!(repeat("1"), Ok(1));
        assert_eq!(repeat(&maximum), Ok(u32::MAX));
        assert_eq!(repeat("0"), Err("repeat count too small\n".to_string()));
        assert_eq!(
            repeat("4294967296"),
            Err("repeat count too large\n".to_string())
        );
        assert_eq!(repeat("nope"), Err("repeat count invalid\n".to_string()));
        assert_eq!(repeat("-1"), Err("repeat count too small\n".to_string()));
        assert_eq!(repeat(" +1"), Ok(1));
        assert_eq!(repeat("1 "), Err("repeat count invalid\n".to_string()));
        assert_eq!(repeat("#{window_width}"), Ok(80));
    }

    #[test]
    fn hexadecimal_key_names_follow_tmux_scanf_rules() {
        assert_eq!(encode_hex_key("0x0"), Some(vec![0x00]));
        assert_eq!(encode_hex_key("0x1f"), Some(vec![0x1f]));
        assert_eq!(encode_hex_key("0x20"), Some(vec![0x20]));
        assert_eq!(encode_hex_key("0x80"), Some("\u{80}".as_bytes().to_vec()));
        assert_eq!(encode_hex_key("0x414"), Some("Д".as_bytes().to_vec()));
        assert_eq!(
            encode_hex_key("0x10ffff"),
            Some("\u{10ffff}".as_bytes().to_vec())
        );

        // `%x` stops at a non-hex suffix, but consumes suffix characters that
        // are themselves hexadecimal digits.
        assert_eq!(encode_hex_key("0x41g"), Some(vec![b'A']));
        assert_eq!(
            encode_hex_key("0x41bad"),
            Some("\u{41bad}".as_bytes().to_vec())
        );

        // These spellings are unusual but are part of `%x`'s accepted grammar.
        assert_eq!(encode_hex_key("0x+41"), Some(vec![b'A']));
        assert_eq!(encode_hex_key("0x 41"), Some(vec![b'A']));
        assert_eq!(encode_hex_key("0x0x41"), Some(vec![b'A']));
        assert_eq!(encode_hex_key("0x-0"), Some(vec![0x00]));
    }

    #[test]
    fn invalid_hexadecimal_key_names_fall_back_to_literal_input() {
        for key in ["0x", "0xGG", "0xd800", "0x110000", "0x-1"] {
            assert_eq!(encode_hex_key(key), None, "{key}");
        }
        assert_eq!(encode_hex_key("0X41"), None);
    }

    #[test]
    fn send_keys_hex_literal_requires_one_complete_byte() {
        assert_eq!(encode_hex_literal("41"), Some(vec![b'A']));
        assert_eq!(encode_hex_literal("0x42"), Some(vec![b'B']));
        assert_eq!(encode_hex_literal("+43"), Some(vec![b'C']));
        assert_eq!(encode_hex_literal(" 44"), Some(vec![b'D']));

        for key in ["", "100", "gg", "41junk", "-1"] {
            assert_eq!(encode_hex_literal(key), None, "{key}");
        }
    }

    #[test]
    fn client_injection_preserves_semantic_and_literal_keys() {
        let mut keys = Vec::new();
        inject_client_string("C-b", false, false, &mut keys);
        inject_client_string("F1", false, false, &mut keys);
        inject_client_string("None", false, false, &mut keys);
        inject_client_string("ff", false, true, &mut keys);
        let bytes = keys
            .iter()
            .flat_map(|key| key.bytes.iter().copied())
            .collect::<Vec<_>>();
        assert_eq!(bytes, b"\x02\x1bOPNone\xff");
        assert_eq!(
            keys.iter()
                .map(|key| key.forward_unbound)
                .collect::<Vec<_>>(),
            [true, false, true, true, true, true, true]
        );
    }
}
