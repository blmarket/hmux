use super::*;

/// The terminal reports hmux acts on itself rather than forwarding.
const FOCUS_IN_REPORT: &[u8] = b"\x1b[I";
const FOCUS_OUT_REPORT: &[u8] = b"\x1b[O";
const DARK_THEME_REPORT: &[u8] = b"\x1b[?997;1n";
const LIGHT_THEME_REPORT: &[u8] = b"\x1b[?997;2n";

/// Whether a binding's outcome ended this pass over the input buffer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BindingFlow {
    Continue,
    Break,
}

/// Whether this session acts on mouse reports at all (`mouse on`).
fn mouse_enabled(state: &ServerState, target: &str) -> bool {
    state
        .option_for_target(target, "mouse")
        .or_else(|| super::super::options::option_default("mouse"))
        .is_some_and(|value| value == "on")
}

/// Spell a typed key the way the pane it is going to expects it.
///
/// `None` is a key that pane has no form for, which tmux drops.
fn encode_key_for_pane(state: &ServerState, target: &str, key: PaneKey) -> Option<Vec<u8>> {
    let encoding = state.encode_pane_key(target, key).ok()?;
    encoding.complete.then_some(encoding.bytes)
}

/// Write a mouse report into the pane it landed on, encoded from that pane's
/// own DECSET modes.
///
/// Nothing is written when the pane asked for no mouse mode, when the event
/// hit no pane (status line, border), or when the report is one the pane's
/// mode does not carry — the encoder answers all three by producing no bytes.
fn forward_mouse_to_pane(state: &ServerState, event: &MouseEvent) {
    let Some((pane_id, input)) = event.pane_input_event() else {
        return;
    };
    let _ = state.input_mouse_to_pane(&format!("%{pane_id}"), input);
}

/// Send the plain bytes buffered so far to the active pane, keeping them ahead
/// of whatever the caller is about to do.
/// Continue a copy-mode drag: the pointer moves the selection's far end in the
/// pane the drag started in. `false` when that pane has no drag under way, so
/// the report falls through to the key tables.
fn drag_copy_selection(state: &SharedState, event: &MouseEvent) -> bool {
    let Some(target) = event.target.as_ref() else {
        return false;
    };
    let (Some(pane_id), Some(position)) = (target.pane_id, target.local_position) else {
        return false;
    };
    let mut state = state.borrow_mut();
    let pane_target = format!("%{pane_id}");
    let vi = super::copy_mode::uses_vi_keys(&state, &pane_target);
    state.drag_copy_selection_to_mouse(&pane_target, position.x, position.y, vi)
}

fn flush_forward_buf(
    state: &SharedState,
    target: &str,
    forward_buf: &mut Vec<u8>,
    forwarded: &mut PaneInputStats,
    first_forward_at: &mut Option<Instant>,
) {
    if forward_buf.is_empty() {
        return;
    }
    first_forward_at.get_or_insert_with(Instant::now);
    if let Ok(stats) = forward_input(&state.borrow_mut(), target, forward_buf) {
        add_input_stats(forwarded, stats);
    }
    forward_buf.clear();
}

impl AttachSession {
    /// Strip the focus and theme reports out of one tty read and apply them.
    ///
    /// tmux decodes these in `tty-keys.c` as `KEYC_FOCUS_IN`/`KEYC_FOCUS_OUT`
    /// and `KEYC_REPORT_*_THEME`, which never reach a pane as keystrokes; a
    /// pane that asked for mode 1004 is sent its own copy instead.
    fn take_terminal_reports(&mut self, data: &[u8], state: &SharedState, target: &str) -> Vec<u8> {
        const REPORTS: [&[u8]; 4] = [
            FOCUS_IN_REPORT,
            FOCUS_OUT_REPORT,
            DARK_THEME_REPORT,
            LIGHT_THEME_REPORT,
        ];
        let data = self.take_terminal_answers(data, state);
        let data = data.as_slice();
        if !REPORTS
            .iter()
            .any(|report| data.windows(report.len()).any(|window| window == *report))
        {
            return data.to_vec();
        }
        let mut kept = Vec::with_capacity(data.len());
        let mut index = 0;
        while index < data.len() {
            let matched = REPORTS
                .iter()
                .find(|report| data[index..].starts_with(report));
            match matched {
                Some(report) => {
                    index += report.len();
                    self.apply_terminal_report(report, state, target);
                }
                None => {
                    kept.push(data[index]);
                    index += 1;
                }
            }
        }
        kept
    }

    /// Take the answers to questions the server put to this terminal out of one
    /// tty read and route them.
    ///
    /// Unlike the focus and theme reports, these are only looked for while
    /// something is actually waiting for one: outside that window an
    /// application's own OSC 4 or OSC 52 reply is just bytes on the way to a
    /// pane, and taking it would break the pane that asked for it directly.
    fn take_terminal_answers(&mut self, data: &[u8], state: &SharedState) -> Vec<u8> {
        let client = self.attachments.render_attachment.client_name();
        let scanner = &mut self.compositor.input.terminal_answer;
        if !scanner.is_holding() && !state.borrow_mut().client_awaits_terminal_reply(&client) {
            return data.to_vec();
        }
        let (kept, answers) = scanner.feed(data);
        for reply in answers {
            state.borrow_mut().deliver_terminal_reply(&client, reply);
        }
        kept
    }

    fn apply_terminal_report(&mut self, report: &[u8], state: &SharedState, target: &str) {
        let client = self.attachments.render_attachment.client_name();
        let mut st = state.borrow_mut();
        match report {
            FOCUS_IN_REPORT | FOCUS_OUT_REPORT => {
                let focused = report == FOCUS_IN_REPORT;
                st.set_client_focus(&client, self.compositor.target.session_id, focused, target);
            }
            _ => {
                let dark = report == DARK_THEME_REPORT;
                st.set_client_theme(
                    &client,
                    self.compositor.target.session_id,
                    if dark { "dark" } else { "light" },
                );
            }
        }
    }

    /// Mirror the client's key table into the server, tmux's
    /// `server_status_client` after every table change: it is what
    /// `#{client_key_table}`/`#{client_prefix}` read and what makes a status
    /// line show a pending prefix.
    fn publish_key_table(&mut self, state: &SharedState) {
        let table = self.compositor.input.keys.table();
        if self.compositor.input.published_key_table == table {
            return;
        }
        self.compositor.input.published_key_table = table.to_string();
        let client = self.attachments.render_attachment.client_name();
        {
            let mut st = state.borrow_mut();
            st.set_client_key_table(&client, &self.compositor.input.published_key_table);
        }
        self.status
            .status_cache
            .update_client_key_table(self.compositor.input.published_key_table.clone());
    }

    /// tmux's `server_client_repeat_timer`: a `bind -r` chain lapses on its own
    /// once `repeat-time` passes, with no further input.
    pub(super) fn expire_repeat_chain(&mut self, state: &SharedState, target: &str, now: Instant) {
        let default_table = client_key_table(&state.borrow_mut(), target);
        if !self
            .compositor
            .input
            .keys
            .expire_repeat(now, &default_table)
        {
            return;
        }
        self.publish_key_table(state);
    }

    pub(super) fn repeat_deadline(&self) -> Option<Instant> {
        self.compositor.input.keys.repeat_deadline()
    }

    pub(super) fn click_deadline(&self) -> Option<Instant> {
        self.compositor.input.mouse.click_deadline()
    }

    /// Apply one key binding's client-local outcome.
    ///
    /// Shared by the tty read loop and the repeat-click timer, which delivers a
    /// synthesized `DoubleClick` through exactly the same dispatch.
    fn apply_binding_outcome(
        &mut self,
        outcome: PrefixOutcome,
        state: &SharedState,
        target: &str,
        hub: &StatusHub,
        forward_buf: &mut Vec<u8>,
        force_render: &mut bool,
    ) -> BindingFlow {
        match outcome {
            PrefixOutcome::Detach => {
                self.compositor.transition =
                    Some(AttachTransition::Finish(AttachFinishReason::Detached));
                return BindingFlow::Break;
            }
            PrefixOutcome::SendPrefix(bytes) => forward_buf.extend(bytes),
            PrefixOutcome::CopyMode(action) => {
                // Re-entering copy mode from inside it only honors `-u`.
                if copy_mode::is_active(&state.borrow_mut(), target) {
                    action.reactivate(state, target);
                } else {
                    action.apply(state, target);
                }
                *force_render = true;
            }
            PrefixOutcome::Confirm { prompt, action } => {
                self.compositor.ui.confirm = Some(ActiveConfirm {
                    prompt,
                    action,
                    confirm_key: b'y',
                    default_yes: false,
                    reply: None,
                });
                *force_render = true;
            }
            PrefixOutcome::Prompt { args } => {
                if let Ok(mut prompt) =
                    CommandPrompt::new(args, None, state, hub, &self.compositor.target.context)
                {
                    if prompt.should_freeze() {
                        prompt.freeze(self.compositor.render.last_render.clone());
                    }
                    prompt.initial_incremental(state, hub, &self.compositor.target.context);
                    self.compositor.ui.command_prompt = Some(prompt);
                }
                *force_render = true;
            }
            PrefixOutcome::DeferredCommand { args, context } => {
                self.commands.pending.push_back(AttachCommandRequest {
                    source: command::DeferredCommand::Args(args),
                    context,
                    continuation: AttachCommandContinuation::PrefixBinding {
                        target: target.to_string(),
                        cols: self.viewport.cols,
                        pane_rows: self.viewport.pane_rows,
                    },
                });
                return BindingFlow::Break;
            }
            PrefixOutcome::DeferredMessage {
                args,
                context,
                target,
                escape_hashes,
                explicit_duration,
            } => {
                self.commands.pending.push_back(AttachCommandRequest {
                    source: command::DeferredCommand::Args(args),
                    context,
                    continuation: AttachCommandContinuation::Message {
                        target,
                        escape_hashes,
                        explicit_duration,
                    },
                });
                return BindingFlow::Break;
            }
            PrefixOutcome::Handled { changed } => {
                if changed {
                    *force_render = true;
                }
            }
        }
        BindingFlow::Continue
    }

    /// tmux's `server_client_click_timer`: with two presses and no third, the
    /// gesture resolves as a `DoubleClick` delivered after the fact.
    ///
    /// The replayed event runs the ordinary key walk, so a `DoubleClick*`
    /// binding sees the same tables and target a real press would.
    pub(super) fn expire_click_timer(
        &mut self,
        state: &SharedState,
        target: &str,
        hub: &StatusHub,
        now: Instant,
    ) {
        let Some(event) = self.compositor.input.mouse.expire_click(now) else {
            return;
        };
        let Some(key) = event.key_code() else {
            return;
        };
        let tables = ServerKeyTables::new(state, target);
        let resolution = self.compositor.input.keys.resolve(key, now, &tables);
        self.publish_key_table(state);
        let KeyResolution::Binding { table } = resolution else {
            return;
        };
        let outcome = dispatch_key_binding(
            &table,
            key,
            state,
            target,
            hub,
            &self.compositor.target.context,
            Some(event),
        );
        let mut forward_buf = Vec::new();
        let mut force_render = false;
        self.apply_binding_outcome(
            outcome,
            state,
            target,
            hub,
            &mut forward_buf,
            &mut force_render,
        );
        if !forward_buf.is_empty() {
            let _ = forward_input(&state.borrow_mut(), target, &forward_buf);
        }
        if force_render {
            self.compositor.render.last_render.clear();
            self.compositor.render.force_clear = true;
            self.status.status_cache.invalidate();
        }
    }

    pub(super) fn drive_input(
        &mut self,
        state: &SharedState,
        hub: &StatusHub,
    ) -> io::Result<Option<AttachDrive>> {
        // A key binding's command must finish before the next key's runs: a
        // burst like `select-pane` then `send-keys -M` depends on the order.
        // Leaving the rest of the input unread — in the kernel buffer or in
        // `injected` — is what keeps that ordering.
        if !self.commands.pending.is_empty() {
            return Ok(None);
        }
        let stable_target = self.compositor.target.stable_target.clone();
        let target = stable_target.as_str();
        // 2. Relay terminal queries which the emulator consumed from pane output.
        //    The outer terminal's reply is read immediately below and forwarded
        //    through the ordinary pane-input path. In particular, Neovim sends
        //    an OSC 11 default-background request followed by a CSI 5n status
        //    request; it needs both the RGB and CSI 0n replies.
        let terminal_queries = state
            .borrow_mut()
            .take_active_pane_terminal_queries(target)
            .unwrap_or_default();
        for query in terminal_queries {
            let _ = self
                .tty
                .output
                .queue(self.tty.render_fd.as_raw_fd(), &query);
        }

        // 3. Read input from the client tty, interpreting tmux's prefix key table
        //    and forwarding everything else to the active pane.
        let mut input_buf = [0u8; 1024];
        let mut force_render = false;
        // Bytes forwarded to the pane's pty this iteration (real keystrokes, not
        // prefix-table navigation), used to stamp keystroke latency below.
        let mut forwarded = PaneInputStats::default();
        let mut first_forward_at = None;
        // Keep plain bytes across immediately adjacent tty reads. Besides
        // reducing PTY writes, this preserves compound terminal replies such as
        // OSC 11 followed by CSI 0n when they straddle read boundaries.
        let pending_terminal_reply = self.compositor.input.terminal_reply.take();
        let waiting_for_terminal_reply = pending_terminal_reply.is_some();
        let mut terminal_reply_deadline =
            pending_terminal_reply.as_ref().map(|reply| reply.deadline);
        let mut forward_buf = pending_terminal_reply
            .map(|reply| reply.bytes)
            .unwrap_or_default();
        forward_buf.reserve(input_buf.len());
        // A key prompt may consume only the front logical key from a tty read.
        // Replay its suffix through this same loop so prefix/copy/passthrough
        // handling remains identical to input received by a later read.
        let mut replay_input = Vec::new();
        let mut replay_forward_unbound = true;
        let mut prefer_tty_reply = waiting_for_terminal_reply;
        loop {
            let (replayed, forward_unbound) = if replay_input.is_empty() {
                if prefer_tty_reply {
                    prefer_tty_reply = false;
                    (Vec::new(), true)
                } else if let Some(key) = self.compositor.input.injected.pop_front() {
                    (key.bytes, key.forward_unbound)
                } else {
                    (Vec::new(), true)
                }
            } else {
                (std::mem::take(&mut replay_input), replay_forward_unbound)
            };
            let n = if replayed.is_empty() {
                unsafe {
                    libc::read(
                        self.tty.input_fd.as_raw_fd(),
                        input_buf.as_mut_ptr() as *mut libc::c_void,
                        input_buf.len(),
                    )
                }
            } else {
                replayed.len() as isize
            };
            if n < 0 {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::WouldBlock {
                    if self
                        .compositor
                        .ui
                        .command_prompt
                        .as_ref()
                        .is_some_and(CommandPrompt::captures_literal_key)
                        && !self.compositor.input.key_prompt.bytes().is_empty()
                    {
                        let decoded = decode_prompt_key(self.compositor.input.key_prompt.bytes());
                        let could_be_terminal_key = decoded.is_none()
                            && self
                                .compositor
                                .input
                                .key_prompt
                                .bytes()
                                .starts_with(b"\x1b")
                            && matches!(
                                self.compositor.input.key_prompt.bytes().get(1),
                                Some(b'[' | b'O')
                            );
                        if could_be_terminal_key {
                            let deadline = self.compositor.input.key_prompt.deadline_or_insert(
                                Instant::now() + prompt_escape_delay(&state.borrow_mut()),
                            );
                            if Instant::now() < deadline {
                                break;
                            }
                        }
                        let decoded = decoded.or_else(|| {
                            let bytes = self.compositor.input.key_prompt.bytes();
                            (bytes.len() >= 2 && bytes[0] == 0x1b)
                                .then(|| (meta_prompt_key(bytes[1]), 2))
                        });
                        if let Some((key, consumed)) = decoded {
                            let tail =
                                self.compositor.input.key_prompt.bytes()[consumed..].to_vec();
                            let outcome = handle_command_prompt_key(
                                &mut self.compositor.ui.command_prompt,
                                &key,
                                state,
                                hub,
                                &self.compositor.target.context,
                            );
                            self.compositor.input.key_prompt.clear();
                            force_render = true;
                            match outcome {
                                Some(PromptKeyOutcome::Request(request)) => {
                                    if !tail.is_empty() {
                                        self.compositor.input.injected.push_front(ClientKey {
                                            bytes: tail,
                                            forward_unbound,
                                        });
                                    }
                                    self.commands.pending.push_back(request);
                                    break;
                                }
                                Some(PromptKeyOutcome::ShowError(error)) => {
                                    self.show_command_error(state, &error);
                                }
                                None => {}
                            }
                            replay_input = tail;
                            replay_forward_unbound = forward_unbound;
                            if !replay_input.is_empty() {
                                continue;
                            }
                        }
                    }
                    let is_partial_terminal_reply = forward_buf.starts_with(b"\x1b]")
                        && !forward_buf.windows(4).any(|bytes| bytes == b"\x1b[0n");
                    if is_partial_terminal_reply {
                        let deadline = *terminal_reply_deadline
                            .get_or_insert_with(|| Instant::now() + Duration::from_millis(2));
                        if Instant::now() < deadline {
                            self.compositor.input.terminal_reply = Some(PendingTerminalReply {
                                bytes: std::mem::take(&mut forward_buf),
                                deadline,
                            });
                        }
                    }
                    break;
                } else {
                    self.compositor.transition = Some(AttachTransition::Finish(
                        AttachFinishReason::ConnectionClosed,
                    ));
                    break;
                }
            } else if n == 0 {
                // EOF on tty: client closed.
                self.compositor.transition = Some(AttachTransition::Finish(
                    AttachFinishReason::ConnectionClosed,
                ));
                break;
            }

            // Feed the chunk through the prefix state machine byte by byte. Plain
            // bytes are buffered and flushed to the active pane in order; a prefix
            // (`Ctrl-b`) consumes the next byte as a key-table command. The
            // The prefix-pending flag lives outside the read loop, so a prefix at
            // the end of one chunk pairs with the command key in the next one —
            // exactly how a user types `C-b` then `c`.
            let read_data = if replayed.is_empty() {
                &input_buf[..n as usize]
            } else {
                replayed.as_slice()
            };
            self.attachments.prompt_attachment.note_activity();
            // tmux stamps the client and its session on every key, which is
            // what defers the `lock-after-time` timer while somebody is typing
            // — including keys that only reach the pane.
            {
                let mut st = state.borrow_mut();
                st.touch_client_activity(
                    &self.attachments.render_attachment.client_name(),
                    self.compositor.target.session_id,
                );
            }
            let mut prompt_tail = None;
            if self
                .compositor
                .ui
                .command_prompt
                .as_ref()
                .is_some_and(CommandPrompt::captures_literal_key)
            {
                self.compositor.input.key_prompt.extend(read_data);
                if let Some((key, consumed)) =
                    decode_prompt_key(self.compositor.input.key_prompt.bytes())
                {
                    let tail = self.compositor.input.key_prompt.bytes()[consumed..].to_vec();
                    match handle_command_prompt_key(
                        &mut self.compositor.ui.command_prompt,
                        &key,
                        state,
                        hub,
                        &self.compositor.target.context,
                    ) {
                        Some(PromptKeyOutcome::Request(request)) => {
                            self.compositor.input.key_prompt.clear();
                            if !tail.is_empty() {
                                self.compositor.input.injected.push_front(ClientKey {
                                    bytes: tail,
                                    forward_unbound,
                                });
                            }
                            self.commands.pending.push_back(request);
                            force_render = true;
                            break;
                        }
                        outcome => {
                            if let Some(PromptKeyOutcome::ShowError(error)) = outcome {
                                self.show_command_error(state, &error);
                            }
                            prompt_tail = Some(tail);
                            self.compositor.input.key_prompt.clear();
                            force_render = true;
                        }
                    }
                } else if self
                    .compositor
                    .input
                    .key_prompt
                    .bytes()
                    .starts_with(b"\x1b")
                    && matches!(
                        self.compositor.input.key_prompt.bytes().get(1),
                        Some(b'[' | b'O')
                    )
                    && self.compositor.input.key_prompt.deadline().is_none()
                {
                    self.compositor.input.key_prompt.set_deadline_if_none(
                        Instant::now() + prompt_escape_delay(&state.borrow_mut()),
                    );
                }
                if prompt_tail.as_ref().is_none_or(Vec::is_empty) {
                    continue;
                }
            }
            // Focus and theme reports are keys tmux consumes itself rather
            // than forwarding, so they are taken out of the stream before the
            // prefix/pane machinery ever sees them.
            let filtered = self.take_terminal_reports(
                prompt_tail.as_deref().unwrap_or(read_data),
                state,
                target,
            );
            let data = filtered.as_slice();
            self.compositor.input.key_prompt.clear();
            let mut i = 0;
            while i < data.len() {
                if self.compositor.ui.active_overlay.is_some() {
                    let start = i;
                    let (decoded, consumed) = decode_tty_key(&data[i..]).unwrap_or_else(|| {
                        (
                            DecodedTtyKey {
                                name: plain_prompt_key(data[i]),
                                code: Some(key_from_byte(data[i])),
                                mouse: None,
                                flags: TtyKeyFlags::default(),
                            },
                            1,
                        )
                    });
                    i += consumed;
                    let outcome = self
                        .compositor
                        .ui
                        .active_overlay
                        .as_mut()
                        .expect("overlay checked")
                        .handle_key(
                            &decoded.name,
                            &data[start..i],
                            decoded.mouse.as_ref(),
                            self.viewport.cols,
                            self.viewport.rows,
                            state,
                            target,
                        );
                    let close = outcome.close;
                    let close_exit = outcome.exit;
                    let selected_command = outcome.command;
                    let inserted = selected_command
                        .as_ref()
                        .is_some_and(|command| !command.is_empty());
                    if let Some(command) = selected_command
                        .as_ref()
                        .filter(|command| !command.is_empty())
                    {
                        let overlay = self
                            .compositor
                            .ui
                            .active_overlay
                            .take()
                            .expect("overlay checked");
                        self.commands.pending.push_back(AttachCommandRequest {
                            source: command::DeferredCommand::Args(command.clone()),
                            context: self.compositor.target.context.clone(),
                            continuation: AttachCommandContinuation::Overlay {
                                overlay: Box::new(overlay),
                                inserted,
                            },
                        });
                        force_render = true;
                        break;
                    }
                    // A selected command was queued above, so only the
                    // overlay's own exit is left to report.
                    let result = close.then(|| {
                        if close_exit == 0 {
                            command::CommandResult::ok("")
                        } else {
                            let mut result = command::CommandResult::err("");
                            result.exit = close_exit;
                            result
                        }
                    });
                    if close {
                        if let Some(mut overlay) = self.compositor.ui.active_overlay.take() {
                            overlay.complete(
                                result.unwrap_or_else(|| command::CommandResult::ok("")),
                                inserted,
                            );
                        }
                    }
                    force_render = true;
                    continue;
                }
                if let Some(prompt) = self.compositor.ui.command_prompt.as_mut() {
                    let (decoded, consumed) = decode_tty_key(&data[i..])
                        .map(|(key, consumed)| (key.name, consumed))
                        .unwrap_or_else(|| (plain_prompt_key(data[i]), 1));
                    i += consumed;
                    let mut incremental = None;
                    match prompt.handle_key(&decoded, state, hub, &self.compositor.target.context) {
                        CommandPromptInput::Continue => {
                            incremental = prompt.take_deferred_incremental();
                        }
                        CommandPromptInput::Finish(mut result) => {
                            let mut prompt = self
                                .compositor
                                .ui
                                .command_prompt
                                .take()
                                .expect("command prompt checked");
                            if let Some(source) = take_deferred_attach_command(&mut result) {
                                self.commands.pending.push_back(AttachCommandRequest {
                                    source,
                                    context: self.compositor.target.context.clone(),
                                    continuation: AttachCommandContinuation::Prompt {
                                        prompt: Box::new(prompt),
                                    },
                                });
                                break;
                            }
                            if let Some(error) =
                                prompt.complete(&result, state, &self.compositor.target.context)
                            {
                                self.show_command_error(state, &error);
                            }
                        }
                        CommandPromptInput::Cancel => {
                            let mut prompt = self
                                .compositor
                                .ui
                                .command_prompt
                                .take()
                                .expect("command prompt checked");
                            prompt.cancel_external();
                        }
                    }
                    if let Some(source) = incremental {
                        self.commands
                            .deferred_prompts
                            .push_back(AttachCommandRequest {
                                source,
                                context: self.compositor.target.context.clone(),
                                continuation: AttachCommandContinuation::Ignore,
                            });
                    }
                    force_render = true;
                    continue;
                }
                if let Some(active) = self.compositor.ui.confirm.take() {
                    // A confirm-before prompt is up: this key answers it and is
                    // consumed whole (so a multi-byte escape can't leak to the
                    // pane). `y`/`Y` runs the guarded command; every other key
                    // cancels, exactly like tmux's client-confirm callback.
                    let (key, consumed) = read_key(&data[i..]);
                    i += consumed;
                    force_render = true;
                    let accepted = matches!(key, Key::Byte(value) if value == active.confirm_key)
                        || (key == Key::Enter && active.default_yes);
                    if let ConfirmResolution::Deferred { command, reply } = active.resolve(
                        accepted,
                        state,
                        target,
                        self.viewport.cols,
                        self.viewport.pane_rows,
                    ) {
                        self.commands.pending.push_back(AttachCommandRequest {
                            source: command::DeferredCommand::Args(command),
                            context: self.compositor.target.context.clone(),
                            continuation: AttachCommandContinuation::Confirm {
                                reply,
                                inserted: true,
                            },
                        });
                        break;
                    }
                    continue;
                }
                if state.borrow_mut().mode_view_active(target) {
                    let (decoded, consumed) = decode_tty_key(&data[i..]).unwrap_or_else(|| {
                        (
                            DecodedTtyKey {
                                name: plain_prompt_key(data[i]),
                                code: Some(key_from_byte(data[i])),
                                mouse: None,
                                flags: TtyKeyFlags::default(),
                            },
                            1,
                        )
                    });
                    i += consumed;
                    let outcome = state
                        .borrow_mut()
                        .mode_view_key(target, &decoded.name, self.viewport.pane_rows as usize)
                        .unwrap_or(ModeViewKeyResult::None);
                    match outcome {
                        ModeViewKeyResult::Command(command) if !command.is_empty() => {
                            self.commands.pending.push_back(AttachCommandRequest {
                                source: command::DeferredCommand::Args(command),
                                context: self.compositor.target.context.clone(),
                                continuation: AttachCommandContinuation::Ignore,
                            });
                            break;
                        }
                        ModeViewKeyResult::Prompt(request) => {
                            if let Ok(mut prompt) = CommandPrompt::for_mode(
                                request,
                                target,
                                state,
                                hub,
                                &self.compositor.target.context,
                            ) {
                                if prompt.should_freeze() {
                                    prompt.freeze(self.compositor.render.last_render.clone());
                                }
                                prompt.initial_incremental(
                                    state,
                                    hub,
                                    &self.compositor.target.context,
                                );
                                self.compositor.ui.command_prompt = Some(prompt);
                            }
                        }
                        ModeViewKeyResult::Popup(request) => {
                            if let Ok(overlay) = ActiveOverlay::from_request(
                                super::super::state::OverlayRequest::Popup(*request),
                                None,
                                self.viewport.cols,
                                self.viewport.rows,
                            ) {
                                self.compositor.ui.active_overlay = overlay;
                            }
                        }
                        ModeViewKeyResult::None | ModeViewKeyResult::Command(_) => {}
                    }
                    force_render = true;
                    continue;
                }
                // Everything else goes through the client's key tables, which
                // decide whether this key runs a binding, belongs to the key
                // machinery itself (the prefix, or a stray key after it), or is
                // the pane's. The command key can be a multi-byte escape (e.g.
                // PgUp), so parse a logical key rather than one raw byte.
                let start = i;
                // tmux's `tty_keys_user` and `tty_default_code_keys`: a
                // sequence `user-keys` names, or one the client's terminfo
                // spells, is that key whatever the fixed tables would have made
                // of it. Both are resolved here so an unbound one still reaches
                // the pane through the ordinary path below.
                // A report that continues a drag belongs to the drag the
                // opening report started, not to a key table (tmux's
                // `KEYC_DRAGGING`).
                let mut continuing_drag = false;
                let named = user_key_at(state, target, &data[i..])
                    .map(|(key, consumed)| (key, consumed, TtyKeyFlags::default()))
                    .or_else(|| terminfo_key_at(&self.tty.terminal, &data[i..]));
                let (key, mouse, consumed, flags) = match named {
                    Some((key, consumed, flags)) => (Some(key), None, consumed, Some(flags)),
                    None => match decode_tty_key(&data[i..]) {
                        Some((mut decoded, consumed)) => {
                            continuing_drag = decoded.mouse.as_ref().is_some_and(|event| {
                                self.compositor.input.mouse.continuing_drag(event)
                            });
                            resolve_mouse_key(
                                &mut decoded,
                                &mut self.compositor.input.mouse,
                                state,
                                target,
                                self.viewport.cols,
                                self.viewport.rows,
                                &mut self.status.status_cache,
                            );
                            (decoded.code, decoded.mouse, consumed, Some(decoded.flags))
                        }
                        // Bytes that decode to nothing — a truncated escape, or
                        // a byte that is not valid UTF-8 — have no key identity
                        // to re-encode from, so they reach the pane as they
                        // arrived.
                        None => (Some(key_from_byte(data[i])), None, 1, None),
                    },
                };
                i += consumed;
                let Some(key) = key else {
                    continue;
                };
                // With `mouse off` the client never consults its key tables for
                // a mouse report: tmux hands it straight to the pane under the
                // pointer (`server_client_key_callback`'s `forward_key`), which
                // encodes it — or drops it — per its own DECSET modes.
                if let Some(event) = mouse.as_ref() {
                    if !mouse_enabled(&state.borrow_mut(), target) {
                        flush_forward_buf(
                            state,
                            target,
                            &mut forward_buf,
                            &mut forwarded,
                            &mut first_forward_at,
                        );
                        forward_mouse_to_pane(&state.borrow_mut(), event);
                        continue;
                    }
                }
                if continuing_drag {
                    if let Some(event) = mouse.as_ref() {
                        if drag_copy_selection(state, event) {
                            force_render = true;
                            continue;
                        }
                    }
                }
                let tables = ServerKeyTables::new(state, target);
                let now = Instant::now();
                // tmux's `server_client_is_assume_paste`, checked before the
                // key tables: keys arriving faster than a person types are
                // pasted text, so they reach the pane instead of running a
                // binding. A mouse report is exempt, as are focus reports,
                // which never come from typing at all.
                // The client's terminal bracketing a paste is the same answer
                // without the guessing.
                let bracketed = mouse.is_none() && self.compositor.input.keys.is_bracket_paste(key);
                let pasted = bracketed
                    || mouse.is_none()
                        && self.compositor.input.keys.is_assume_paste(
                            now,
                            tables.assume_paste_time(),
                            self.tty.terminal.capability(Capability::Enbp).is_some(),
                        );
                let resolution = if pasted {
                    KeyResolution::Forward
                } else {
                    self.compositor.input.keys.resolve(key, now, &tables)
                };
                // The walk can move the client between tables even when the key
                // ends up in the pane — an expired prefix, a chain that just
                // ended — so republish before acting on the outcome.
                self.publish_key_table(state);
                if matches!(resolution, KeyResolution::Forward) {
                    // An unbound mouse key is re-encoded for the pane rather
                    // than forwarded verbatim: the pane's protocol (X10, UTF-8
                    // or SGR) is its own choice, not the client terminal's.
                    if let Some(event) = mouse.as_ref() {
                        flush_forward_buf(
                            state,
                            target,
                            &mut forward_buf,
                            &mut forwarded,
                            &mut first_forward_at,
                        );
                        forward_mouse_to_pane(&state.borrow_mut(), event);
                    } else if forward_unbound {
                        // Likewise for every other key. tmux never hands a
                        // client's bytes to a pane verbatim: it decodes them and
                        // spells the key again for the pane's own terminal type
                        // and modes, which is how a pane running under
                        // `TERM=tmux-256color` is spared the `CSI H` and
                        // `CSI 105;5u` forms a modern client terminal reports.
                        match flags {
                            Some(flags) => {
                                if let Some(bytes) = encode_key_for_pane(
                                    &state.borrow_mut(),
                                    target,
                                    flags.pane_key(key),
                                ) {
                                    forward_buf.extend_from_slice(&bytes);
                                }
                            }
                            None => forward_buf.extend_from_slice(&data[start..i]),
                        }
                    }
                    continue;
                }
                // The key machinery claimed this key, so flush what preceded
                // it: a `send-prefix` byte has to stay in typing order.
                flush_forward_buf(
                    state,
                    target,
                    &mut forward_buf,
                    &mut forwarded,
                    &mut first_forward_at,
                );
                let KeyResolution::Binding { table } = resolution else {
                    continue;
                };
                let outcome = dispatch_key_binding(
                    &table,
                    key,
                    state,
                    target,
                    hub,
                    &self.compositor.target.context,
                    mouse,
                );
                let flow = self.apply_binding_outcome(
                    outcome,
                    state,
                    target,
                    hub,
                    &mut forward_buf,
                    &mut force_render,
                );
                if flow == BindingFlow::Break {
                    // A deferred command leaves this pass, but the read may
                    // have carried more keys behind it — a moving mouse sends
                    // its whole burst in one write. Replay the tail rather than
                    // dropping it.
                    if i < data.len() {
                        self.compositor.input.injected.push_front(ClientKey {
                            bytes: data[i..].to_vec(),
                            forward_unbound,
                        });
                    }
                    break;
                }
            }
            if matches!(
                self.compositor.transition,
                Some(AttachTransition::Finish(_))
            ) {
                break;
            }
            // Only one command can be queued per pass; the replayed tail waits
            // in `injected` for the next one rather than overwriting it.
            if !self.commands.pending.is_empty() {
                break;
            }
        }
        if !forward_buf.is_empty() {
            first_forward_at.get_or_insert_with(Instant::now);
            if let Ok(stats) = forward_input(&state.borrow_mut(), target, &forward_buf) {
                add_input_stats(&mut forwarded, stats);
            }
        }
        // Start (or extend) the latency clock after offering this keystroke burst
        // to the pane. The counters retain whether bytes reached the PTY now,
        // remained queued, or were dropped; the output/render hooks close it out.
        if forwarded.accepted() > 0 || forwarded.dropped > 0 {
            self.pane_io.latmon.on_input(
                first_forward_at.unwrap_or_else(Instant::now),
                forwarded.accepted(),
                forwarded.queued,
                forwarded.dropped,
            );
        }
        let finish_reason = match self.compositor.transition {
            Some(AttachTransition::Finish(reason)) => Some(reason),
            _ => None,
        };
        if let Some(reason) = finish_reason {
            self.compositor.transition = None;
            return Ok(Some(AttachDrive::Finish(reason)));
        }

        // A prefix command changed the window/pane layout: drop the cached frame
        // and force a full clear so the (possibly smaller) new active pane can't
        // leave the previous pane's cells behind.
        if force_render {
            self.compositor.render.last_render.clear();
            self.compositor.render.force_clear = true;
            self.status.status_cache.invalidate();
            let st = state.borrow_mut();
            match active_window_output_subscription(&st, target) {
                Ok(subscription) => {
                    (
                        self.attachments.subscribed_window,
                        self.attachments.output_subscription,
                    ) = subscription;
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    return Ok(Some(AttachDrive::Finish(AttachFinishReason::SessionEnded)));
                }
                Err(error) => return Err(error),
            }
            self.attachments.output_generation = self.attachments.output_generation.wrapping_add(1);
        }

        Ok(None)
    }
}

/// The `user-keys` entry `data` starts with, as a key and the bytes it took.
///
/// tmux's `tty_keys_user` matches the option's array by index: entry *n* is the
/// key `Usern`.
fn user_key_at(
    state: &SharedState,
    target: &str,
    data: &[u8],
) -> Option<(super::super::key::KeyCode, usize)> {
    let state = state.borrow_mut();
    let sequences = state.user_key_sequences(target);
    sequences
        .iter()
        .enumerate()
        .filter(|(_, sequence)| !sequence.is_empty() && data.starts_with(sequence.as_bytes()))
        // The longest match wins, so a prefix of another user key cannot
        // swallow it.
        .max_by_key(|(_, sequence)| sequence.len())
        .map(|(index, sequence)| {
            (
                super::super::key::KeyCode::new(
                    super::super::key::KeyBase::User(index as u16),
                    super::super::key::Modifiers::default(),
                ),
                sequence.len(),
            )
        })
}

/// The key capabilities tmux's `tty_default_code_keys` reads from the client's
/// terminfo, paired with the key name each one spells.
///
/// Only the capabilities whose spelling actually varies between terminals are
/// listed: everything the fixed CSI/SS3 tables already agree on is left to
/// them, so this is a fallback for a terminal that disagrees rather than a
/// second decoder.
const TERMINFO_KEYS: &[(Capability, &str)] = &[
    (Capability::kf1, "F1"),
    (Capability::kf2, "F2"),
    (Capability::kf3, "F3"),
    (Capability::kf4, "F4"),
    (Capability::kf5, "F5"),
    (Capability::kf6, "F6"),
    (Capability::kf7, "F7"),
    (Capability::kf8, "F8"),
    (Capability::kf9, "F9"),
    (Capability::kf10, "F10"),
    (Capability::kf11, "F11"),
    (Capability::kf12, "F12"),
    (Capability::kf13, "S-F1"),
    (Capability::kf14, "S-F2"),
    (Capability::kf15, "S-F3"),
    (Capability::kf16, "S-F4"),
    (Capability::kf17, "S-F5"),
    (Capability::kf18, "S-F6"),
    (Capability::kf19, "S-F7"),
    (Capability::kf20, "S-F8"),
    (Capability::kf21, "S-F9"),
    (Capability::kf22, "S-F10"),
    (Capability::kf23, "S-F11"),
    (Capability::kf24, "S-F12"),
    (Capability::kich1, "IC"),
    (Capability::kdch1, "DC"),
    (Capability::khome, "Home"),
    (Capability::kend, "End"),
    (Capability::knp, "NPage"),
    (Capability::kpp, "PPage"),
    (Capability::kcbt, "BTab"),
    (Capability::kcuu1, "Up"),
    (Capability::kcud1, "Down"),
    (Capability::kcub1, "Left"),
    (Capability::kcuf1, "Right"),
    // The modifier capabilities: a terminal that spells `C-Left` its own way is
    // taken at its word rather than left to the fixed CSI tables.
    (Capability::kf25, "C-F1"),
    (Capability::kf26, "C-F2"),
    (Capability::kf27, "C-F3"),
    (Capability::kf28, "C-F4"),
    (Capability::kf29, "C-F5"),
    (Capability::kf30, "C-F6"),
    (Capability::kf31, "C-F7"),
    (Capability::kf32, "C-F8"),
    (Capability::kf33, "C-F9"),
    (Capability::kf34, "C-F10"),
    (Capability::kf35, "C-F11"),
    (Capability::kf36, "C-F12"),
    (Capability::kf37, "C-S-F1"),
    (Capability::kf38, "C-S-F2"),
    (Capability::kf39, "C-S-F3"),
    (Capability::kf40, "C-S-F4"),
    (Capability::kf41, "C-S-F5"),
    (Capability::kf42, "C-S-F6"),
    (Capability::kf43, "C-S-F7"),
    (Capability::kf44, "C-S-F8"),
    (Capability::kf45, "C-S-F9"),
    (Capability::kf46, "C-S-F10"),
    (Capability::kf47, "C-S-F11"),
    (Capability::kf48, "C-S-F12"),
    (Capability::kf49, "M-F1"),
    (Capability::kf50, "M-F2"),
    (Capability::kf51, "M-F3"),
    (Capability::kf52, "M-F4"),
    (Capability::kf53, "M-F5"),
    (Capability::kf54, "M-F6"),
    (Capability::kf55, "M-F7"),
    (Capability::kf56, "M-F8"),
    (Capability::kf57, "M-F9"),
    (Capability::kf58, "M-F10"),
    (Capability::kf59, "M-F11"),
    (Capability::kf60, "M-F12"),
    (Capability::kf61, "M-S-F1"),
    (Capability::kf62, "M-S-F2"),
    (Capability::kf63, "M-S-F3"),
    (Capability::kind, "S-Down"),
    (Capability::kri, "S-Up"),
    (Capability::kDC, "S-DC"),
    (Capability::kDC3, "M-DC"),
    (Capability::kDC4, "S-M-DC"),
    (Capability::kDC5, "C-DC"),
    (Capability::kDC6, "S-C-DC"),
    (Capability::kDC7, "M-C-DC"),
    (Capability::kDN, "S-Down"),
    (Capability::kDN3, "M-Down"),
    (Capability::kDN4, "S-M-Down"),
    (Capability::kDN5, "C-Down"),
    (Capability::kDN6, "S-C-Down"),
    (Capability::kDN7, "M-C-Down"),
    (Capability::kEND, "S-End"),
    (Capability::kEND3, "M-End"),
    (Capability::kEND4, "S-M-End"),
    (Capability::kEND5, "C-End"),
    (Capability::kEND6, "S-C-End"),
    (Capability::kEND7, "M-C-End"),
    (Capability::kHOM, "S-Home"),
    (Capability::kHOM3, "M-Home"),
    (Capability::kHOM4, "S-M-Home"),
    (Capability::kHOM5, "C-Home"),
    (Capability::kHOM6, "S-C-Home"),
    (Capability::kHOM7, "M-C-Home"),
    (Capability::kIC, "S-IC"),
    (Capability::kIC3, "M-IC"),
    (Capability::kIC4, "S-M-IC"),
    (Capability::kIC5, "C-IC"),
    (Capability::kIC6, "S-C-IC"),
    (Capability::kIC7, "M-C-IC"),
    (Capability::kLFT, "S-Left"),
    (Capability::kLFT3, "M-Left"),
    (Capability::kLFT4, "S-M-Left"),
    (Capability::kLFT5, "C-Left"),
    (Capability::kLFT6, "S-C-Left"),
    (Capability::kLFT7, "M-C-Left"),
    (Capability::kNXT, "S-NPage"),
    (Capability::kNXT3, "M-NPage"),
    (Capability::kNXT4, "S-M-NPage"),
    (Capability::kNXT5, "C-NPage"),
    (Capability::kNXT6, "S-C-NPage"),
    (Capability::kNXT7, "M-C-NPage"),
    (Capability::kPRV, "S-PPage"),
    (Capability::kPRV3, "M-PPage"),
    (Capability::kPRV4, "S-M-PPage"),
    (Capability::kPRV5, "C-PPage"),
    (Capability::kPRV6, "S-C-PPage"),
    (Capability::kPRV7, "M-C-PPage"),
    (Capability::kRIT, "S-Right"),
    (Capability::kRIT3, "M-Right"),
    (Capability::kRIT4, "S-M-Right"),
    (Capability::kRIT5, "C-Right"),
    (Capability::kRIT6, "S-C-Right"),
    (Capability::kRIT7, "M-C-Right"),
    (Capability::kUP, "S-Up"),
    (Capability::kUP3, "M-Up"),
    (Capability::kUP4, "S-M-Up"),
    (Capability::kUP5, "C-Up"),
    (Capability::kUP6, "S-C-Up"),
    (Capability::kUP7, "M-C-Up"),
];

/// The key the client's terminfo spells with the bytes `data` starts with.
///
/// The fixed tables are tried first by the caller; this catches a terminal
/// whose capability disagrees with them — `TERM=linux`, whose `kf1` is
/// `CSI [ A` rather than `SS3 P`.
fn terminfo_key_at(
    terminal: &dyn TerminalCapabilities,
    data: &[u8],
) -> Option<(super::super::key::KeyCode, usize, TtyKeyFlags)> {
    // A capability that is a prefix of another must not swallow it, so the
    // longest match wins.
    TERMINFO_KEYS
        .iter()
        .filter_map(|(capability, name)| {
            let super::super::term::CapabilityValue::String(value) =
                terminal.capability(*capability)?
            else {
                return None;
            };
            if value.len() <= 1 || !data.starts_with(value) {
                return None;
            }
            Some((
                parse_key_name(name)?,
                value.len(),
                TtyKeyFlags {
                    // tmux marks the arrows `KEYC_CURSOR`, so a pane in DECCKM
                    // is told the application form whichever one arrived.
                    cursor: matches!(
                        capability,
                        Capability::kcuu1
                            | Capability::kcud1
                            | Capability::kcub1
                            | Capability::kcuf1
                    ),
                    // Every meta capability is `KEYC_IMPLIED_META`: the escape
                    // is part of what the terminal sent, not something to add.
                    implied_meta: name.contains("M-"),
                    ..TtyKeyFlags::default()
                },
            ))
        })
        .max_by_key(|(_, length, _)| *length)
}
