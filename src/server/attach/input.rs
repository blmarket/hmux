use super::*;

/// The terminal reports hmux acts on itself rather than forwarding.
const FOCUS_IN_REPORT: &[u8] = b"\x1b[I";
const FOCUS_OUT_REPORT: &[u8] = b"\x1b[O";
const DARK_THEME_REPORT: &[u8] = b"\x1b[?997;1n";
const LIGHT_THEME_REPORT: &[u8] = b"\x1b[?997;2n";

impl AttachSession {
    /// Strip the focus and theme reports out of one tty read and apply them.
    ///
    /// tmux decodes these in `tty-keys.c` as `KEYC_FOCUS_IN`/`KEYC_FOCUS_OUT`
    /// and `KEYC_REPORT_*_THEME`, which never reach a pane as keystrokes; a
    /// pane that asked for mode 1004 is sent its own copy instead.
    fn take_terminal_reports(
        &mut self,
        data: &[u8],
        state: &Arc<Mutex<ServerState>>,
        target: &str,
    ) -> Vec<u8> {
        const REPORTS: [&[u8]; 4] = [
            FOCUS_IN_REPORT,
            FOCUS_OUT_REPORT,
            DARK_THEME_REPORT,
            LIGHT_THEME_REPORT,
        ];
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

    fn apply_terminal_report(
        &mut self,
        report: &[u8],
        state: &Arc<Mutex<ServerState>>,
        target: &str,
    ) {
        let client = self.attachments.render_attachment.client_name();
        let Ok(mut st) = state.lock() else {
            return;
        };
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
    fn publish_key_table(&mut self, state: &Arc<Mutex<ServerState>>) {
        let table = self.compositor.input.keys.table();
        if self.compositor.input.published_key_table == table {
            return;
        }
        self.compositor.input.published_key_table = table.to_string();
        let client = self.attachments.render_attachment.client_name();
        if let Ok(mut st) = state.lock() {
            st.set_client_key_table(&client, &self.compositor.input.published_key_table);
        }
        self.status
            .status_cache
            .update_client_key_table(self.compositor.input.published_key_table.clone());
    }

    /// tmux's `server_client_repeat_timer`: a `bind -r` chain lapses on its own
    /// once `repeat-time` passes, with no further input.
    pub(super) fn expire_repeat_chain(
        &mut self,
        state: &Arc<Mutex<ServerState>>,
        target: &str,
        now: Instant,
    ) {
        let default_table = client_key_table(state, target);
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

    pub(super) fn drive_input(
        &mut self,
        state: &Arc<Mutex<ServerState>>,
        hub: &StatusHub,
    ) -> io::Result<Option<AttachDrive>> {
        let stable_target = self.compositor.target.stable_target.clone();
        let target = stable_target.as_str();
        // 2. Relay terminal queries which Ghostty consumed from pane output.
        //    The outer terminal's reply is read immediately below and forwarded
        //    through the ordinary pane-input path. In particular, Neovim sends
        //    an OSC 11 default-background request followed by a CSI 5n status
        //    request; it needs both the RGB and CSI 0n replies.
        let terminal_queries = state
            .lock()
            .ok()
            .and_then(|st| st.take_active_pane_terminal_queries(target).ok())
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
                            let deadline =
                                self.compositor.input.key_prompt.deadline_or_insert(
                                    Instant::now() + prompt_escape_delay(state),
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
                            let request = handle_command_prompt_key(
                                &mut self.compositor.ui.command_prompt,
                                &key,
                                state,
                                hub,
                                &self.compositor.target.context,
                            );
                            self.compositor.input.key_prompt.clear();
                            force_render = true;
                            if let Some(request) = request {
                                if !tail.is_empty() {
                                    self.compositor.input.injected.push_front(ClientKey {
                                        bytes: tail,
                                        forward_unbound,
                                    });
                                }
                                self.commands.pending = Some(request);
                                break;
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
            if let Ok(mut st) = state.lock() {
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
                    if let Some(request) = handle_command_prompt_key(
                        &mut self.compositor.ui.command_prompt,
                        &key,
                        state,
                        hub,
                        &self.compositor.target.context,
                    ) {
                        self.compositor.input.key_prompt.clear();
                        if !tail.is_empty() {
                            self.compositor.input.injected.push_front(ClientKey {
                                bytes: tail,
                                forward_unbound,
                            });
                        }
                        self.commands.pending = Some(request);
                        force_render = true;
                        break;
                    }
                    prompt_tail = Some(tail);
                    self.compositor.input.key_prompt.clear();
                    force_render = true;
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
                    self.compositor
                        .input
                        .key_prompt
                        .set_deadline_if_none(Instant::now() + prompt_escape_delay(state));
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
                        .handle_key(&decoded.name, &data[start..i], state, target);
                    let close = outcome.close;
                    let close_exit = outcome.exit;
                    let selected_command = outcome.command;
                    let inserted = selected_command
                        .as_ref()
                        .is_some_and(|command| !command.is_empty());
                    if let Some(command) = selected_command
                        .as_ref()
                        .filter(|command| !command.is_empty())
                        .filter(|_| self.compositor.target.context.defer_attach_commands)
                    {
                        let overlay = self
                            .compositor
                            .ui
                            .active_overlay
                            .take()
                            .expect("overlay checked");
                        self.commands.pending = Some(AttachCommandRequest {
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
                    let result = if let Some(command) =
                        selected_command.filter(|command| !command.is_empty())
                    {
                        let agents = hub.snapshot().panes;
                        Some(command::run_with_context(
                            &command,
                            state,
                            &agents,
                            &self.compositor.target.context,
                        ))
                    } else if close {
                        Some(if close_exit == 0 {
                            command::CommandResult::ok("")
                        } else {
                            let mut result = command::CommandResult::err("");
                            result.exit = close_exit;
                            result
                        })
                    } else {
                        None
                    };
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
                                self.commands.pending = Some(AttachCommandRequest {
                                    source,
                                    context: self.compositor.target.context.clone(),
                                    continuation: AttachCommandContinuation::Prompt {
                                        prompt: Box::new(prompt),
                                    },
                                });
                                break;
                            }
                            prompt.complete(&result, state, &self.compositor.target.context);
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
                        hub,
                        &self.compositor.target.context,
                    ) {
                        self.commands.pending = Some(AttachCommandRequest {
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
                if state
                    .lock()
                    .ok()
                    .is_some_and(|st| st.mode_view_active(target))
                {
                    let (decoded, consumed) = decode_tty_key(&data[i..]).unwrap_or_else(|| {
                        (
                            DecodedTtyKey {
                                name: plain_prompt_key(data[i]),
                                code: Some(key_from_byte(data[i])),
                                mouse: None,
                            },
                            1,
                        )
                    });
                    i += consumed;
                    let outcome = state
                        .lock()
                        .ok()
                        .and_then(|mut st| {
                            st.mode_view_key(
                                target,
                                &decoded.name,
                                self.viewport.pane_rows as usize,
                            )
                            .ok()
                        })
                        .unwrap_or(ModeViewKeyResult::None);
                    match outcome {
                        ModeViewKeyResult::Command(command) if !command.is_empty() => {
                            if self.compositor.target.context.defer_attach_commands {
                                self.commands.pending = Some(AttachCommandRequest {
                                    source: command::DeferredCommand::Args(command),
                                    context: self.compositor.target.context.clone(),
                                    continuation: AttachCommandContinuation::Ignore,
                                });
                                break;
                            }
                            let agents = hub.snapshot().panes;
                            let _ = command::run_with_context(
                                &command,
                                state,
                                &agents,
                                &self.compositor.target.context,
                            );
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
                let (key, mouse, consumed) = match decode_tty_key(&data[i..]) {
                    Some((mut decoded, consumed)) => {
                        resolve_mouse_key(
                            &mut decoded,
                            &mut self.compositor.input.mouse,
                            state,
                            target,
                            self.viewport.cols,
                            self.viewport.rows,
                            &mut self.status.status_cache,
                        );
                        (decoded.code, decoded.mouse, consumed)
                    }
                    None => (Some(key_from_byte(data[i])), None, 1),
                };
                i += consumed;
                let Some(key) = key else {
                    continue;
                };
                let tables = ServerKeyTables::new(state, target);
                let resolution = self
                    .compositor
                    .input
                    .keys
                    .resolve(key, Instant::now(), &tables);
                // The walk can move the client between tables even when the key
                // ends up in the pane — an expired prefix, a chain that just
                // ended — so republish before acting on the outcome.
                self.publish_key_table(state);
                if matches!(resolution, KeyResolution::Forward) {
                    if forward_unbound {
                        forward_buf.extend_from_slice(&data[start..i]);
                    }
                    continue;
                }
                // The key machinery claimed this key, so flush what preceded
                // it: a `send-prefix` byte has to stay in typing order.
                if !forward_buf.is_empty() {
                    first_forward_at.get_or_insert_with(Instant::now);
                    if let Ok(stats) = forward_input(state, target, &forward_buf) {
                        add_input_stats(&mut forwarded, stats);
                    }
                    forward_buf.clear();
                }
                let KeyResolution::Binding { table } = resolution else {
                    continue;
                };
                // Re-entering copy mode from inside it only honors `-u`.
                let reentering_copy_mode = copy_mode::is_active(state, target);
                match dispatch_key_binding(
                    &table,
                    key,
                    state,
                    target,
                    self.viewport.cols,
                    self.viewport.pane_rows,
                    hub,
                    &self.compositor.target.context,
                    mouse,
                ) {
                    PrefixOutcome::Detach => {
                        self.compositor.transition =
                            Some(AttachTransition::Finish(AttachFinishReason::Detached));
                        break;
                    }
                    PrefixOutcome::SendPrefix(bytes) => forward_buf.extend(bytes),
                    PrefixOutcome::CopyMode(action) => {
                        if reentering_copy_mode {
                            action.reactivate(state, target);
                        } else {
                            action.apply(state, target, self.viewport.pane_rows);
                        }
                        force_render = true;
                    }
                    PrefixOutcome::Confirm { prompt, action } => {
                        self.compositor.ui.confirm = Some(ActiveConfirm {
                            prompt,
                            action,
                            confirm_key: b'y',
                            default_yes: false,
                            reply: None,
                        });
                        force_render = true;
                    }
                    PrefixOutcome::Prompt { args } => {
                        if let Ok(mut prompt) = CommandPrompt::new(
                            args,
                            None,
                            state,
                            hub,
                            &self.compositor.target.context,
                        ) {
                            if prompt.should_freeze() {
                                prompt.freeze(self.compositor.render.last_render.clone());
                            }
                            prompt.initial_incremental(state, hub, &self.compositor.target.context);
                            self.compositor.ui.command_prompt = Some(prompt);
                        }
                        force_render = true;
                    }
                    PrefixOutcome::Message { text, duration } => {
                        self.compositor.ui.confirm = None;
                        self.compositor.ui.status_message = Some(StatusMessage {
                            text,
                            deadline: Instant::now()
                                .checked_add(duration)
                                .unwrap_or_else(Instant::now),
                        });
                        force_render = true;
                    }
                    PrefixOutcome::ViewOutput(bytes) => {
                        append_view_output(state, target, &bytes);
                        force_render = true;
                    }
                    PrefixOutcome::DeferredCommand { args, context } => {
                        self.commands.pending = Some(AttachCommandRequest {
                            source: command::DeferredCommand::Args(args),
                            context,
                            continuation: AttachCommandContinuation::PrefixBinding {
                                target: target.to_string(),
                                cols: self.viewport.cols,
                                pane_rows: self.viewport.pane_rows,
                            },
                        });
                        break;
                    }
                    PrefixOutcome::DeferredMessage {
                        args,
                        context,
                        target,
                        escape_hashes,
                        explicit_duration,
                    } => {
                        self.commands.pending = Some(AttachCommandRequest {
                            source: command::DeferredCommand::Args(args),
                            context,
                            continuation: AttachCommandContinuation::Message {
                                target,
                                escape_hashes,
                                explicit_duration,
                            },
                        });
                        break;
                    }
                    PrefixOutcome::Handled { changed } => {
                        if changed {
                            force_render = true;
                        }
                    }
                }
            }
            if matches!(
                self.compositor.transition,
                Some(AttachTransition::Finish(_))
            ) {
                break;
            }
        }
        if !forward_buf.is_empty() {
            first_forward_at.get_or_insert_with(Instant::now);
            if let Ok(stats) = forward_input(state, target, &forward_buf) {
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
            return Ok(Some(self.begin_finish(reason)));
        }

        // A prefix command changed the window/pane layout: drop the cached frame
        // and force a full clear so the (possibly smaller) new active pane can't
        // leave the previous pane's cells behind.
        if force_render {
            self.compositor.render.last_render.clear();
            self.compositor.render.force_clear = true;
            self.status.status_cache.invalidate();
            let st = state
                .lock()
                .map_err(|_| io::Error::other("state poisoned"))?;
            match active_window_output_subscription(&st, target) {
                Ok(subscription) => {
                    (
                        self.attachments.subscribed_window,
                        self.attachments.output_subscription,
                    ) = subscription;
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    return Ok(Some(self.begin_finish(AttachFinishReason::SessionEnded)));
                }
                Err(error) => return Err(error),
            }
            self.attachments.output_generation = self.attachments.output_generation.wrapping_add(1);
        }

        Ok(None)
    }
}
