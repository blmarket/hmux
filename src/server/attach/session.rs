use super::*;
use crate::server::state::SharedState;

impl AttachSession {
    /// Whether this client could be attached at all, without side effects.
    ///
    /// tmux's `cmd_new_session_exec` opens the terminal *before* it creates
    /// anything, so a client with no controlling terminal fails without
    /// leaving a session behind; running the same test up front is what keeps
    /// the interactive create path from leaking one.
    pub(crate) fn check_terminal(client_tty: &ClientTty) -> Result<(), AttachStartFailure> {
        let failure =
            || AttachStartFailure::Client("open terminal failed: not a terminal\n".to_string());
        let render = client_tty.render_fd().or_else(|| client_tty.input_fd());
        let input = client_tty.input_fd().or_else(|| client_tty.render_fd());
        let (Some(render), Some(input)) = (render, input) else {
            return Err(failure());
        };
        if !is_tty(render.as_raw_fd()) || !is_tty(input.as_raw_fd()) {
            return Err(failure());
        }
        Ok(())
    }

    /// Whether the server wants terminal focus reporting turned on.
    fn focus_events(state: &ServerState) -> bool {
        state.server_options().get("focus-events") == Some("on")
    }

    pub(crate) fn start_in_mode<W>(
        target: &str,
        client_tty: ClientTty,
        client_flags: super::super::state::ClientFlagState,
        state: &SharedState,
        hub: &StatusHub,
        context: &command::ClientContext,
        writer: &mut W,
    ) -> Result<Self, AttachStartFailure>
    where
        W: FrameSink + ?Sized,
    {
        let render_fd_borrowed = client_tty.render_fd();
        let input_fd_borrowed = client_tty.input_fd();
        let (render_raw, input_raw) = match (render_fd_borrowed, input_fd_borrowed) {
            (Some(render), Some(input)) => (render.as_raw_fd(), input.as_raw_fd()),
            (Some(render), None) => (render.as_raw_fd(), render.as_raw_fd()),
            (None, Some(input)) => (input.as_raw_fd(), input.as_raw_fd()),
            (None, None) => {
                return Err(AttachStartFailure::Client(
                    "open terminal failed: not a terminal\n".to_string(),
                ));
            }
        };

        let render_fd = unsafe {
            let duplicated = libc::dup(render_raw);
            if duplicated < 0 {
                return Err(AttachStartFailure::Client(
                    "open terminal failed: not a terminal\n".to_string(),
                ));
            }
            OwnedFd::from_raw_fd(duplicated)
        };
        let input_fd = unsafe {
            let duplicated = libc::dup(input_raw);
            if duplicated < 0 {
                return Err(AttachStartFailure::Client(
                    "open terminal failed: not a terminal\n".to_string(),
                ));
            }
            OwnedFd::from_raw_fd(duplicated)
        };
        if !is_tty(render_fd.as_raw_fd()) || !is_tty(input_fd.as_raw_fd()) {
            return Err(AttachStartFailure::Client(
                "open terminal failed: not a terminal\n".to_string(),
            ));
        }

        let winsize = get_winsize(render_fd.as_raw_fd()).unwrap_or(ClientWinsize {
            cols: 80,
            rows: 24,
            xpixel: 0,
            ypixel: 0,
        });
        let (cols, rows) = (winsize.cols, winsize.rows);
        let (prompt_registry, render_registry, session_id) = {
            let mut st = state.borrow_mut();
            let session_id = st.session_id(target).ok_or_else(|| {
                AttachStartFailure::Client(format!("can't find session: {target}\n"))
            })?;
            st.touch_session_activity(session_id, true);
            (
                st.client_prompt_registry(),
                st.client_render_registry(),
                session_id,
            )
        };
        let prompt_attachment = prompt_registry.attach(
            client_tty.tty_name.clone().unwrap_or_default(),
            client_tty.client_pid,
            session_id,
        )?;
        let render_name = client_tty
            .tty_name
            .clone()
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| format!("client-{}", client_tty.client_pid.unwrap_or_default()));
        let render_attachment = render_registry.attach_with_details(
            session_id,
            render_name,
            client_tty.term.clone().unwrap_or_default(),
            client_tty.client_pid,
            cols,
            rows,
            client_tty.flags,
            client_flags,
            false,
        )?;
        let peer_uid = context.peer_uid;
        let peer_user = peer_uid
            .and_then(super::super::format::username)
            .unwrap_or_default();
        render_attachment.set_peer_identity(peer_uid, peer_user.clone());
        render_attachment.set_environment(context.environment.clone());
        // The cell's pixel size arrives with the terminal size and feeds the
        // window's own, so it is published alongside — a client that reports
        // none publishes zero, as tmux's `tty->xpixel` does.
        render_attachment.update_cell_pixels(winsize.xpixel, winsize.ypixel);

        let stable_target = format!("${session_id}");
        let mut attached_context = context.clone();
        attached_context.current_session_id = Some(session_id);
        attached_context.kind = command::ClientKind::Attached;
        let mut compositor =
            AttachCompositorState::new(session_id, attached_context, stable_target);
        let target = compositor.target.stable_target.as_str();
        let terminal_identity = TerminalIdentity::new(
            client_tty.term.clone().unwrap_or_default(),
            client_tty.terminfo.clone(),
            client_tty.features,
            context.env("COLORTERM").map(str::to_string),
        )
        .with_utf8(client_tty.flags & 0x10000 != 0);
        let (status_h, status_interval, terminal) = {
            let st = state.borrow_mut();
            (
                status::height(&st, target),
                status::interval(&st, target),
                ResolvedTerm::resolve(terminal_identity, st.server_options().iter_effective()),
            )
        };
        if let Some(cause) = terminal.validation_error() {
            return Err(AttachStartFailure::Client(format!("{cause}\n")));
        }
        render_attachment.update_terminal(&terminal);
        writer.send(Frame::new(Message::Ready))?;

        let status_timer = StatusTimer::new(status_interval, Instant::now());
        let mut status_cache = status::RenderCache::for_client(
            status::RenderClientContext {
                term: (!terminal.name().is_empty()).then(|| terminal.name().to_string()),
                tty: client_tty.tty_name.clone(),
                pid: client_tty.client_pid,
                uid: peer_uid,
                user: peer_user,
                cwd: context.cwd.clone(),
                ..status::RenderClientContext::default()
            },
            render_attachment.format_jobs(),
        );
        let agent_status_subscription = hub.subscribe()?;
        status_cache.update_agents(hub.snapshot());
        let pane_rows = rows.saturating_sub(status_h).max(1);
        {
            let mut st = state.borrow_mut();
            let _ = st.resize_session(target, cols, pane_rows);
        }

        let saved_termios = make_raw(input_fd.as_raw_fd()).ok();
        let verase = saved_termios
            .as_ref()
            .map(|termios| termios.c_cc[libc::VERASE])
            .filter(|byte| *byte != 0);
        let termios_guard = TermiosGuard {
            fd: input_fd.as_raw_fd(),
            saved: saved_termios,
        };
        set_nonblock(input_fd.as_raw_fd())?;
        set_nonblock(render_fd.as_raw_fd())?;

        let mut tty_output = TtyOutput::new();
        let focus_events = Self::focus_events(&state.borrow_mut());
        let tty_start = tty_start_sequence(&terminal, focus_events);
        // The capability queries ride out with the start sequence, so their
        // answers are recognised from the first read of this client's input.
        if terminal.is_vt100_like() {
            compositor.input.awaiting = CapabilityAnswers::asked();
        }
        let _ = tty_output.queue(render_fd.as_raw_fd(), &tty_start);
        // The mouse mode is not set here: `sync_tty_mouse_mode` does it on the
        // first pass of the loop, from the pane's modes and the options as
        // they stand then, and keeps doing it as either changes.
        let (subscribed_window, output_subscription) = {
            let st = state.borrow_mut();
            active_window_output_subscription(&st, target)?
        };
        let latmon = LatMon::new(format!("sess={target}"));

        Ok(Self {
            tty: AttachTty {
                termios_guard,
                verase,
                input_fd,
                render_fd,
                terminal,
                output: tty_output,
            },
            attachments: AttachAttachments {
                prompt_attachment,
                render_attachment,
                agent_status_subscription,
                output_subscription,
                subscribed_window,
                output_generation: 0,
            },
            viewport: AttachViewport {
                cols,
                rows,
                pane_rows,
                status_height: status_h,
                xpixel: winsize.xpixel,
                ypixel: winsize.ypixel,
            },
            status: AttachStatus {
                status_timer,
                status_cache,
                output_refresh: OutputStatusRefresh::default(),
            },
            pane_io: AttachPaneIo { latmon },
            commands: AttachCommands {
                pending: VecDeque::new(),
                deferred_prompts: VecDeque::new(),
            },
            compositor,
            pending_exec: None,
            pending_hangup: false,
        })
    }

    pub(crate) fn take_command_request(&mut self) -> Option<AttachCommandRequest> {
        self.commands
            .pending
            .pop_front()
            .or_else(|| self.commands.deferred_prompts.pop_front())
            .or_else(|| {
                let source = self
                    .compositor
                    .ui
                    .command_prompt
                    .as_mut()?
                    .take_deferred_incremental()?;
                Some(AttachCommandRequest {
                    source,
                    context: self.compositor.target.context.clone(),
                    continuation: AttachCommandContinuation::Ignore,
                })
            })
    }

    pub(crate) fn complete_command(
        &mut self,
        continuation: AttachCommandContinuation,
        result: command::CommandResult,
        state: &SharedState,
    ) {
        match continuation {
            AttachCommandContinuation::PrefixBinding {
                target,
                cols,
                pane_rows,
            } => {
                if result.exit == 0 && !result.stdout_data().is_empty() {
                    append_view_output(state, &target, result.stdout_data());
                }
                if result.exit == 0 {
                    {
                        let mut state = state.borrow_mut();
                        let _ = state.resize_session(&target, cols, pane_rows);
                    }
                } else {
                    self.show_command_error(state, &result.stderr);
                }
                self.compositor.render.force_clear = true;
            }
            AttachCommandContinuation::Overlay {
                mut overlay,
                inserted,
            } => {
                overlay.complete(result, inserted);
                self.compositor.render.force_clear = true;
            }
            AttachCommandContinuation::Confirm { reply, inserted } => {
                if let Some(reply) = reply {
                    reply.send(Some(super::super::state::PromptCompletion {
                        stdout: result.stdout,
                        stderr: result.stderr,
                        exit: result.exit,
                        inserted,
                    }));
                }
                self.compositor.render.force_clear = true;
            }
            AttachCommandContinuation::Prompt { mut prompt } => {
                prompt.apply_deferred_side_effect(&result, state);
                if let Some(error) =
                    prompt.complete(&result, state, &self.compositor.target.context)
                {
                    self.show_command_error(state, &error);
                }
                self.compositor.render.force_clear = true;
            }
            AttachCommandContinuation::Message {
                target,
                escape_hashes,
                explicit_duration,
            } => {
                if result.exit == 0 {
                    let mut text = result
                        .stdout
                        .strip_suffix('\n')
                        .unwrap_or(&result.stdout)
                        .to_string();
                    if escape_hashes {
                        text = text.replace('#', "##");
                    }
                    let milliseconds = explicit_duration
                        .or_else(|| {
                            state
                                .borrow_mut()
                                .option_for_target(&target, "display-time")
                                .and_then(|value| value.parse().ok())
                        })
                        .unwrap_or(750);
                    self.compositor.ui.status_message = Some(StatusMessage {
                        text,
                        deadline: Instant::now()
                            .checked_add(Duration::from_millis(milliseconds))
                            .unwrap_or_else(Instant::now),
                    });
                } else {
                    self.show_command_error(state, &result.stderr);
                }
                self.compositor.render.force_clear = true;
            }
            AttachCommandContinuation::CloseHook { remove } => {
                if let Some(path) = remove {
                    let _ = std::fs::remove_file(path);
                }
                self.compositor.render.force_clear = true;
            }
            AttachCommandContinuation::Ignore => {
                self.compositor.render.force_clear = true;
            }
        }
        self.compositor.render.last_render.clear();
    }

    /// A failed command's report on the status line, as tmux's `cmdq_error`
    /// shows it for an attached client: first letter uppercased, displayed
    /// for `display-time`.
    pub(super) fn show_command_error(&mut self, state: &SharedState, stderr: &str) {
        let text = stderr.strip_suffix('\n').unwrap_or(stderr);
        let mut chars = text.chars();
        let Some(first) = chars.next() else {
            return;
        };
        let milliseconds = state
            .borrow_mut()
            .option_for_target(
                self.compositor.target.stable_target.as_str(),
                "display-time",
            )
            .and_then(|value| value.parse().ok())
            .unwrap_or(750);
        self.compositor.ui.status_message = Some(StatusMessage {
            text: format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
            deadline: Instant::now()
                .checked_add(Duration::from_millis(milliseconds))
                .unwrap_or_else(Instant::now),
        });
    }

    /// Give the terminal back what this attach took from it. Only the sequence
    /// is queued here; pushing it out belongs to the caller, which owns the
    /// waiting.
    pub(crate) fn stop_terminal(&mut self) {
        let tty_stop = tty_stop_sequence(&self.tty.terminal, self.viewport.rows);
        let _ = self
            .tty
            .output
            .queue(self.tty.render_fd.as_raw_fd(), &tty_stop);
    }

    /// The descriptor the terminal is rendered to, for a caller waiting on it
    /// to take the rest.
    pub(crate) fn tty_render_fd(&self) -> BorrowedFd<'_> {
        self.tty.render_fd.as_fd()
    }

    pub(crate) fn tty_output_pending(&self) -> bool {
        self.tty.output.has_pending()
    }

    pub(crate) fn flush_tty_output(&mut self) -> io::Result<()> {
        self.tty.output.flush(self.tty.render_fd.as_raw_fd())
    }

    /// Hand the terminal back the way it was found. Nothing more may be queued
    /// for it after this.
    pub(crate) fn restore_tty(&mut self) {
        let _ = set_blocking(self.tty.input_fd.as_raw_fd());
        let _ = set_blocking(self.tty.render_fd.as_raw_fd());
        self.tty.termios_guard.restore_and_disarm();
    }

    /// What the client is told this attach ended as, or nothing when the client
    /// is the one that went away.
    pub(crate) fn finish_message(
        &mut self,
        reason: AttachFinishReason,
        state: &SharedState,
    ) -> Option<Message> {
        match reason {
            // `-E` replaces the detach message entirely: the client execs
            // rather than reporting a detach.
            AttachFinishReason::Detached if self.pending_exec.is_some() => {
                let (command, shell) = self.pending_exec.take().expect("exec checked above");
                Some(Message::Exec { command, shell })
            }
            AttachFinishReason::Detached => {
                let session_name = state
                    .borrow_mut()
                    .sessions()
                    .iter()
                    .find(|candidate| candidate.id == self.compositor.target.session_id)
                    .map(|candidate| candidate.name.clone())
                    .unwrap_or_else(|| self.compositor.target.stable_target.clone());
                // `detach-client -P` and `attach-session -x` ask the client to
                // hang itself up, which it reports as `[detached and SIGHUP]`.
                if std::mem::take(&mut self.pending_hangup) {
                    Some(Message::DetachKill(Some(session_name)))
                } else {
                    Some(Message::Detach(Some(session_name)))
                }
            }
            AttachFinishReason::SessionEnded => Some(Message::Exit(Some(0))),
            AttachFinishReason::ConnectionClosed => None,
        }
    }

    pub(crate) fn prepare_wait(
        &mut self,
        state: &SharedState,
        control_buffered: bool,
    ) -> io::Result<AttachPrepared> {
        if let Some(transition) = self.compositor.transition.take() {
            match transition {
                AttachTransition::SwitchSession(session_id) => {
                    self.compositor.target.switch_session(session_id);
                    let target = self.compositor.target.stable_target.as_str();
                    {
                        let mut st = state.borrow_mut();
                        self.viewport.status_height = status::height(&st, target);
                        self.viewport.pane_rows = self
                            .viewport
                            .rows
                            .saturating_sub(self.viewport.status_height)
                            .max(1);
                        let _ =
                            st.resize_session(target, self.viewport.cols, self.viewport.pane_rows);
                        self.status
                            .status_timer
                            .configure(status::interval(&st, target), Instant::now());
                    }
                    self.status.status_cache.invalidate();
                    self.compositor.render.last_render.clear();
                    self.compositor.render.force_clear = true;
                }
                AttachTransition::Finish(reason) => return Ok(AttachPrepared::Finish(reason)),
            }
        }
        let stable_target = self.compositor.target.stable_target.clone();
        let target = stable_target.as_str();

        let target_exists = {
            let mut st = state.borrow_mut();
            if st.reap_exited_panes() {
                let _ = st.resize_session(target, self.viewport.cols, self.viewport.pane_rows);
                self.compositor.render.last_render.clear();
                self.compositor.render.force_clear = true;
                self.status.status_cache.invalidate();
            }
            st.find(target).is_some()
        };
        if !target_exists {
            // The session may be gone *because* this client was told to move:
            // `kill-session` under `detach-on-destroy off` publishes the
            // switch and destroys the session in one step, and which of the
            // two notifications is seen first is a scheduling accident. Honor
            // the pending switch before concluding the session ended.
            if let Some((session_id, _)) = self.attachments.render_attachment.take_switch() {
                self.compositor.transition = Some(AttachTransition::SwitchSession(session_id));
                return self.prepare_wait(state, control_buffered);
            }
            return Ok(AttachPrepared::Finish(AttachFinishReason::SessionEnded));
        }

        match refresh_active_window_output_subscription(
            &state.borrow_mut(),
            target,
            &mut self.attachments.subscribed_window,
            &mut self.attachments.output_subscription,
        ) {
            Ok(true) => {
                self.attachments.output_generation =
                    self.attachments.output_generation.wrapping_add(1);
                self.compositor.render.last_render.clear();
                self.compositor.render.force_clear = true;
                self.status.status_cache.invalidate();
            }
            Ok(false) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                if let Some((session_id, _)) = self.attachments.render_attachment.take_switch() {
                    self.compositor.transition = Some(AttachTransition::SwitchSession(session_id));
                    return self.prepare_wait(state, control_buffered);
                }
                return Ok(AttachPrepared::Finish(AttachFinishReason::SessionEnded));
            }
            Err(error) => return Err(error),
        }

        let (popup_read, popup_write) = self
            .compositor
            .ui
            .active_overlay
            .as_ref()
            .map(ActiveOverlay::popup_sources)
            .unwrap_or((-1, -1));
        if self
            .compositor
            .ui
            .active_overlay
            .as_ref()
            .is_some_and(ActiveOverlay::popup_read_continuation)
        {
            return Ok(AttachPrepared::Ready(AttachWaitReady {
                popup_read: true,
                ..AttachWaitReady::default()
            }));
        }

        // Keys held back from an earlier pass — the tail of a coalesced read
        // whose first key deferred a command — are already in hand, so the
        // client must not wait on the tty before replaying them.
        if !self.compositor.input.injected.is_empty() && self.commands.pending.is_empty() {
            return Ok(AttachPrepared::Ready(AttachWaitReady::default()));
        }

        let now = Instant::now();
        // The overlay owns its cadence; without one, a clock window ticks on
        // its own so the minute can advance with no other activity.
        let overlay_deadline = match self.compositor.ui.active_overlay.as_ref() {
            Some(overlay) => overlay.deadline(now),
            None => state
                .borrow_mut()
                .active_mode_view(target)
                .is_some_and(|view| view.kind == ModeKind::Clock)
                .then(|| now + Duration::from_secs(1)),
        };
        // Every timed concern of this session, as a plain list: the earliest
        // entry bounds the wait, and each one is re-checked from state on the
        // pass that wakes us. A concern missing here sleeps through its
        // deadline — extend the list, don't fold outside it.
        let deadline = [
            self.status.status_timer.deadline(),
            self.compositor
                .ui
                .status_message
                .as_ref()
                .map(|message| message.deadline),
            self.status.output_refresh.due(),
            overlay_deadline,
            self.compositor.input.key_prompt.deadline(),
            self.repeat_deadline(),
            self.click_deadline(),
            self.compositor
                .input
                .terminal_reply
                .as_ref()
                .map(|reply| reply.deadline),
        ]
        .into_iter()
        .flatten()
        .min();
        let tty_backpressured = self.tty.output.has_pending();
        if !tty_backpressured && control_buffered {
            return Ok(AttachPrepared::Ready(AttachWaitReady {
                control: true,
                ..AttachWaitReady::default()
            }));
        }

        Ok(AttachPrepared::Wait {
            sources: AttachWaitSources {
                input: if tty_backpressured || self.compositor.io_state != ClientIoState::Active {
                    -1
                } else {
                    self.tty.input_fd.as_raw_fd()
                },
                tty_output: if tty_backpressured {
                    self.tty.render_fd.as_raw_fd()
                } else {
                    -1
                },
                output: if tty_backpressured || self.compositor.io_state != ClientIoState::Active {
                    -1
                } else {
                    self.attachments.output_subscription.as_raw_fd()
                },
                output_generation: self.attachments.output_generation,
                prompt: if tty_backpressured || self.compositor.io_state != ClientIoState::Active {
                    -1
                } else {
                    self.attachments.prompt_attachment.as_raw_fd()
                },
                render: if tty_backpressured {
                    -1
                } else {
                    self.attachments.render_attachment.as_raw_fd()
                },
                status: if tty_backpressured {
                    -1
                } else {
                    self.attachments.agent_status_subscription.as_raw_fd()
                },
                popup_read,
                popup_write,
            },
            deadline: if tty_backpressured { None } else { deadline },
        })
    }

    pub(crate) fn drive_ready<R, W>(
        &mut self,
        state: &SharedState,
        hub: &StatusHub,
        ready: AttachWaitReady,
        reader: &mut R,
        writer: &mut W,
    ) -> io::Result<AttachDrive>
    where
        R: NonblockingFrameReader,
        W: FrameSink,
    {
        if ready.popup_read || ready.popup_write {
            if let Some(overlay) = self.compositor.ui.active_overlay.as_mut() {
                overlay.drive_popup_io(ready.popup_read, ready.popup_write)?;
            }
        }
        if ready.tty_output {
            self.tty.output.flush(self.tty.render_fd.as_raw_fd())?;
        }
        self.attachments
            .render_attachment
            .publish_written(self.tty.output.total_written());
        if self.tty.output.has_pending() {
            return Ok(AttachDrive::Continue);
        }
        let control_ready = ready.control;
        let triggers = match self.handle_notifications(state, hub, &ready, writer)? {
            AttachNotificationOutcome::Continue(triggers) => triggers,
            AttachNotificationOutcome::Return(drive) => return Ok(drive),
        };

        // Handle imsg control messages only when reading cannot block.
        if control_ready {
            if let Some(drive) = self.handle_control_message(state, reader, writer)? {
                return Ok(drive);
            }
        }

        if self.compositor.io_state != ClientIoState::Active {
            return Ok(AttachDrive::Continue);
        }

        if let Some(drive) = self.drive_input(state, hub)? {
            return Ok(drive);
        }
        self.render_turn(state, triggers)?;
        self.sync_tty_mouse_mode(state);
        Ok(AttachDrive::Continue)
    }

    fn handle_notifications<W>(
        &mut self,
        state: &SharedState,
        hub: &StatusHub,
        ready: &AttachWaitReady,
        writer: &mut W,
    ) -> io::Result<AttachNotificationOutcome>
    where
        W: FrameSink,
    {
        let stable_target = self.compositor.target.stable_target.clone();
        let target = stable_target.as_str();
        let mut output_ready = ready.output;
        let prompt_ready = ready.prompt;
        let render_ready = ready.render;
        let agent_status_ready = ready.status;
        let now = Instant::now();
        let agent_status_changed = if agent_status_ready {
            self.attachments.agent_status_subscription.drain();
            self.status.status_cache.update_agents(hub.snapshot())
        } else {
            false
        };
        self.expire_repeat_chain(state, target, now);
        self.expire_click_timer(state, target, hub, now);
        let status_timer_ready = self.status.status_timer.take_expired(now)
            | self.status.output_refresh.take_expired(now);
        let overlay_tick = self.compositor.ui.active_overlay.is_some();
        let overlay_exit = self
            .compositor
            .ui
            .active_overlay
            .as_mut()
            .and_then(|overlay| overlay.tick(now));
        if let Some(overlay_exit) = overlay_exit {
            if let Some(mut overlay) = self.compositor.ui.active_overlay.take() {
                let mut result = command::CommandResult::ok("");
                result.exit = overlay_exit;
                // A popup that was opened to edit something reads the result
                // back before it is forgotten; the file goes with it.
                if let Some((command, remove)) = overlay.take_on_close() {
                    if overlay_exit == 0 {
                        // Queued rather than run here: the close hook is an
                        // ordinary command line and may suspend. The file it
                        // reads is removed by the continuation, once it has.
                        self.commands.pending.push_back(AttachCommandRequest {
                            source: command::DeferredCommand::Argv(command),
                            context: self.compositor.target.context.clone(),
                            continuation: AttachCommandContinuation::CloseHook { remove },
                        });
                    } else if let Some(path) = remove {
                        let _ = std::fs::remove_file(path);
                    }
                }
                overlay.complete(result, false);
            }
            self.compositor.render.last_render.clear();
            self.compositor.render.force_clear = true;
        }
        let message_expired = self
            .compositor
            .ui
            .status_message
            .as_ref()
            .is_some_and(|message| message.deadline <= now);
        if message_expired {
            self.compositor.ui.status_message = None;
            self.compositor.render.last_render.clear();
        }
        if status_timer_ready {
            self.status.status_cache.invalidate();
        }
        let render_invalidation = if render_ready {
            self.attachments.render_attachment.take()
        } else {
            super::super::state::RenderInvalidation::default()
        };
        if render_ready {
            // `refresh-client -f` aimed at this client from elsewhere: the
            // registry entry is already updated, so pull its flag view into
            // the copies this client renders and runs commands with.
            if !self
                .attachments
                .render_attachment
                .take_flag_updates()
                .is_empty()
            {
                let (flags, read_only) = self.attachments.render_attachment.client_flags_view();
                self.status
                    .status_cache
                    .update_client_flags(flags, read_only);
                self.compositor.target.context.read_only = read_only;
            }
            for message in self.attachments.render_attachment.take_messages() {
                if message.bell {
                    let _ = self
                        .tty
                        .output
                        .queue(self.tty.render_fd.as_raw_fd(), b"\x07");
                }
                if message.text.is_empty() {
                    continue;
                }
                self.compositor.ui.status_message = Some(StatusMessage {
                    text: message.text,
                    deadline: Instant::now() + Duration::from_millis(message.duration_ms),
                });
                self.compositor.ui.confirm = None;
                self.compositor.render.last_render.clear();
                self.compositor.render.force_clear = true;
            }
            for payload in self.attachments.render_attachment.take_client_output() {
                let _ = self
                    .tty
                    .output
                    .queue(self.tty.render_fd.as_raw_fd(), &payload);
                // tmux's `tty_invalidate`: once an application has written to
                // the terminal itself, nothing cached about its state holds, so
                // the next frame is painted in full rather than as a delta.
                self.compositor.render.last_render.clear();
            }
            if let Some(action) = self.attachments.render_attachment.take_action() {
                match action {
                    ClientAction::Lock(command)
                        if self.compositor.io_state == ClientIoState::Active =>
                    {
                        let stop = tty_stop_sequence(&self.tty.terminal, self.viewport.rows);
                        let _ = self.tty.output.queue(self.tty.render_fd.as_raw_fd(), &stop);
                        self.compositor.render.output_cursor_visible = None;
                        self.tty.termios_guard.restore();
                        writer.send(Frame::new(Message::Lock(command)))?;
                        self.compositor.io_state = ClientIoState::Locked;
                        self.compositor.render.last_render.clear();
                        self.compositor.render.force_clear = true;
                    }
                    ClientAction::Suspend if self.compositor.io_state == ClientIoState::Active => {
                        let stop = tty_stop_sequence(&self.tty.terminal, self.viewport.rows);
                        let _ = self.tty.output.queue(self.tty.render_fd.as_raw_fd(), &stop);
                        self.compositor.render.output_cursor_visible = None;
                        self.tty.termios_guard.restore();
                        writer.send(Frame::new(Message::Suspend))?;
                        self.attachments.render_attachment.set_suspended(true);
                        self.compositor.io_state = ClientIoState::Suspended;
                        self.compositor.render.last_render.clear();
                        self.compositor.render.force_clear = true;
                    }
                    ClientAction::Detach { exec, hangup } => {
                        self.pending_hangup = hangup;
                        // `detach-client -E` hands the client a command to exec
                        // in place of detaching, the way tmux's
                        // `server_client_exec` does. The client replaces itself
                        // with it, so nothing is drawn afterwards; the tty is
                        // handed back first, exactly as for a lock.
                        if let Some(command) = exec.filter(|command| !command.is_empty()) {
                            let shell = {
                                let state = state.borrow_mut();
                                crate::server::command::default_shell(
                                    &state,
                                    Some(&format!("${}", self.compositor.target.session_id)),
                                )
                            };
                            self.pending_exec = Some((command, shell));
                        }
                        return Ok(AttachNotificationOutcome::Return(AttachDrive::Finish(
                            AttachFinishReason::Detached,
                        )));
                    }
                    ClientAction::Switch { session_id, .. } => {
                        self.compositor.transition =
                            Some(AttachTransition::SwitchSession(session_id));
                        return Ok(AttachNotificationOutcome::Return(AttachDrive::Continue));
                    }
                    ClientAction::Keys(keys)
                        if self.compositor.io_state == ClientIoState::Active =>
                    {
                        self.compositor.input.injected.extend(keys);
                    }
                    ClientAction::SetSelection(data) => {
                        // `None` is a query: tmux sends the same capability
                        // with `?` where the payload would be.
                        let encoded = data
                            .as_deref()
                            .map(base64_encode)
                            .unwrap_or_else(|| "?".to_string());
                        if let Some(sequence) = term::expand_capability(
                            &self.tty.terminal,
                            Capability::Ms,
                            &[
                                term::CapabilityParameter::String(""),
                                term::CapabilityParameter::String(&encoded),
                            ],
                        ) {
                            let _ = self
                                .tty
                                .output
                                .queue(self.tty.render_fd.as_raw_fd(), &sequence);
                        }
                    }
                    ClientAction::Overlay { request, reply } => {
                        if matches!(request, OverlayRequest::Clear) {
                            if let Some(mut overlay) = self.compositor.ui.active_overlay.take() {
                                overlay.complete(command::CommandResult::ok(""), false);
                            }
                            if let Some(reply) = reply {
                                reply.send(Some(super::super::state::PromptCompletion {
                                    stdout: String::new(),
                                    stderr: String::new(),
                                    exit: 0,
                                    inserted: false,
                                }));
                            }
                        } else if self.compositor.ui.active_overlay.is_some() {
                            if let Some(reply) = reply {
                                reply.send(Some(super::super::state::PromptCompletion {
                                    stdout: String::new(),
                                    stderr: String::new(),
                                    exit: 0,
                                    inserted: false,
                                }));
                            }
                        } else {
                            let anchor = overlay::OverlayAnchor::capture(
                                state,
                                target,
                                &request,
                                self.viewport.status_height,
                                &self.status.status_cache,
                            );
                            self.compositor.ui.active_overlay = ActiveOverlay::from_request(
                                request,
                                reply,
                                self.viewport.cols,
                                self.viewport.rows,
                                anchor,
                            )
                            .ok()
                            .flatten();
                        }
                        self.compositor.render.last_render.clear();
                        self.compositor.render.force_clear = true;
                    }
                    ClientAction::Confirm {
                        prompt,
                        command,
                        confirm_key,
                        default_yes,
                        reply,
                    } => {
                        self.compositor.ui.confirm = Some(ActiveConfirm {
                            prompt,
                            action: ConfirmAction::Command(command),
                            confirm_key,
                            default_yes,
                            reply,
                        });
                        self.compositor.render.last_render.clear();
                        self.compositor.render.force_clear = true;
                    }
                    ClientAction::Lock(_) => {}
                    ClientAction::Suspend => {}
                    ClientAction::Keys(_) => {}
                }
            }
        }
        if render_invalidation.contains(super::super::state::RenderInvalidation::SESSION_GONE) {
            return Ok(AttachNotificationOutcome::Return(AttachDrive::Finish(
                AttachFinishReason::SessionEnded,
            )));
        }
        if !render_invalidation.is_empty() {
            self.status.status_cache.invalidate();
        }
        if render_invalidation.contains(super::super::state::RenderInvalidation::RESET_MODE)
            || render_invalidation.contains(super::super::state::RenderInvalidation::MODE)
        {
            self.compositor.render.last_render.clear();
        }
        if render_invalidation.contains(super::super::state::RenderInvalidation::STATUS) {
            let mut st = state.borrow_mut();
            if render_invalidation.contains(super::super::state::RenderInvalidation::TERMINAL) {
                self.tty
                    .terminal
                    .refresh(st.server_options().iter_effective());
                self.attachments
                    .render_attachment
                    .update_terminal(&self.tty.terminal);
            }
            self.status
                .status_timer
                .configure(status::interval(&st, target), Instant::now());
            let new_status_h = status::height(&st, target);
            if new_status_h != self.viewport.status_height {
                self.viewport.status_height = new_status_h;
                self.viewport.pane_rows = self
                    .viewport
                    .rows
                    .saturating_sub(self.viewport.status_height)
                    .max(1);
                let _ = st.resize_session(target, self.viewport.cols, self.viewport.pane_rows);
                self.compositor.render.last_render.clear();
                self.compositor.render.force_clear = true;
            }
        }
        if prompt_ready && self.compositor.ui.command_prompt.is_none() {
            if let Some(external) = self.attachments.prompt_attachment.take_command_prompt() {
                let args = external.args().to_vec();
                if let Ok(mut prompt) = CommandPrompt::new(
                    args,
                    Some(external),
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
                self.compositor.render.last_render.clear();
            }
        }
        // An external command connection may switch windows while this thread
        // is blocked in poll. Tty input or an old-pane notification wakes us;
        // replace the stale subscription before attributing output or sending
        // that input to the newly active pane.
        match refresh_active_window_output_subscription(
            &state.borrow_mut(),
            target,
            &mut self.attachments.subscribed_window,
            &mut self.attachments.output_subscription,
        ) {
            Ok(true) => {
                self.attachments.output_generation =
                    self.attachments.output_generation.wrapping_add(1);
                output_ready = false;
                self.compositor.render.last_render.clear();
                self.compositor.render.force_clear = true;
                self.status.status_cache.invalidate();
            }
            Ok(false) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                // A wake from a non-render source can observe the session's
                // destruction before the render notification that carries the
                // client's reassignment is processed; honor the pending
                // switch before concluding the session ended.
                if let Some((session_id, _)) = self.attachments.render_attachment.take_switch() {
                    self.compositor.transition = Some(AttachTransition::SwitchSession(session_id));
                    return Ok(AttachNotificationOutcome::Return(AttachDrive::Continue));
                }
                return Ok(AttachNotificationOutcome::Return(AttachDrive::Finish(
                    AttachFinishReason::SessionEnded,
                )));
            }
            Err(error) => return Err(error),
        }
        if output_ready {
            self.attachments.output_subscription.drain();
            // Output reaches the status only through slow-moving derived
            // content, so its invalidation is throttled; a deferred refresh
            // fires through `take_expired` above. Alerts, renames, and option
            // changes invalidate immediately via `RenderInvalidation::STATUS`.
            if self.status.output_refresh.request(now) {
                self.status.status_cache.invalidate();
            }
            // If this wake came from the active pane, mark its latest output so
            // the upcoming compose is timed against the keystroke that caused
            // it. Background-pane wakes have no newer active timestamp.
            self.pane_io
                .latmon
                .on_output(self.attachments.output_subscription.last_output_at());
        }
        Ok(AttachNotificationOutcome::Continue(AttachRenderTriggers {
            output_ready,
            status_timer_ready,
            agent_status_changed,
            overlay_tick,
            message_expired,
            render_invalidation,
        }))
    }

    fn handle_control_message<R, W>(
        &mut self,
        state: &SharedState,
        reader: &mut R,
        writer: &mut W,
    ) -> io::Result<Option<AttachDrive>>
    where
        R: NonblockingFrameReader,
        W: FrameSink,
    {
        let target = self.compositor.target.stable_target.as_str();
        match reader.try_recv() {
            Ok(frame) => {
                if frame.version != PROTOCOL_VERSION {
                    let _ = writer.send(Frame::new(Message::Version));
                    return Ok(Some(AttachDrive::Finish(
                        AttachFinishReason::ConnectionClosed,
                    )));
                }
                match frame.msg {
                    Message::Resize => {
                        if let Ok(winsize) = get_winsize(self.tty.render_fd.as_raw_fd()) {
                            let (new_cols, new_rows) = (winsize.cols, winsize.rows);
                            // A font change can move the cell's pixel size
                            // without moving the cell count, and an image
                            // already on screen has to be rescaled for it.
                            if (winsize.xpixel, winsize.ypixel)
                                != (self.viewport.xpixel, self.viewport.ypixel)
                            {
                                self.viewport.xpixel = winsize.xpixel;
                                self.viewport.ypixel = winsize.ypixel;
                                self.attachments
                                    .render_attachment
                                    .update_cell_pixels(winsize.xpixel, winsize.ypixel);
                                self.compositor.render.force_clear = true;
                            }
                            if new_cols != self.viewport.cols || new_rows != self.viewport.rows {
                                self.viewport.cols = new_cols;
                                self.viewport.rows = new_rows;
                                if let Some(overlay) = self.compositor.ui.active_overlay.as_mut() {
                                    overlay.resize(self.viewport.cols, self.viewport.rows);
                                }
                                self.attachments
                                    .render_attachment
                                    .update_size(self.viewport.cols, self.viewport.rows);
                                {
                                    let mut st = state.borrow_mut();
                                    // tmux's `MSG_RESIZE` promotes the resizing
                                    // client before recalculating sizes.
                                    st.update_latest_client(
                                        &self.attachments.render_attachment.client_name(),
                                        self.compositor.target.session_id,
                                    );
                                    self.viewport.status_height = status::height(&st, target);
                                    self.viewport.pane_rows = self
                                        .viewport
                                        .rows
                                        .saturating_sub(self.viewport.status_height)
                                        .max(1);
                                    let _ = st.resize_session(
                                        target,
                                        self.viewport.cols,
                                        self.viewport.pane_rows,
                                    );
                                }
                                // Force a full re-render on resize: dimensions
                                // changed, so clear once to drop any stale cells.
                                self.compositor.render.last_render.clear();
                                self.compositor.render.force_clear = true;
                                self.status.status_cache.invalidate();
                            }
                        }
                    }
                    Message::Unlock if self.compositor.io_state == ClientIoState::Locked => {
                        let _ = make_raw(self.tty.input_fd.as_raw_fd());
                        let start = tty_start_sequence(
                            &self.tty.terminal,
                            Self::focus_events(&state.borrow_mut()),
                        );
                        let _ = self
                            .tty
                            .output
                            .queue(self.tty.render_fd.as_raw_fd(), &start);
                        self.compositor.render.output_cursor_visible = None;
                        // The start sequence just cleared the mouse modes, so
                        // the next pass re-applies whatever is wanted now.
                        self.compositor.render.tty_mouse_mode = TtyMouseMode::Off;
                        // tmux stamps session activity when a client comes back
                        // from MSG_UNLOCK/MSG_WAKEUP, which re-arms the lock
                        // timer instead of leaving a resumed client unlocked.
                        {
                            let mut st = state.borrow_mut();
                            st.touch_session_activity(self.compositor.target.session_id, false);
                        }
                        self.compositor.io_state = ClientIoState::Active;
                        self.compositor.render.last_render.clear();
                        self.compositor.render.force_clear = true;
                        self.status.status_cache.invalidate();
                    }
                    Message::Wakeup if self.compositor.io_state == ClientIoState::Suspended => {
                        let _ = make_raw(self.tty.input_fd.as_raw_fd());
                        let start = tty_start_sequence(
                            &self.tty.terminal,
                            Self::focus_events(&state.borrow_mut()),
                        );
                        let _ = self
                            .tty
                            .output
                            .queue(self.tty.render_fd.as_raw_fd(), &start);
                        self.compositor.render.output_cursor_visible = None;
                        // The start sequence just cleared the mouse modes, so
                        // the next pass re-applies whatever is wanted now.
                        self.compositor.render.tty_mouse_mode = TtyMouseMode::Off;
                        // tmux stamps session activity when a client comes back
                        // from MSG_UNLOCK/MSG_WAKEUP, which re-arms the lock
                        // timer instead of leaving a resumed client unlocked.
                        {
                            let mut st = state.borrow_mut();
                            st.touch_session_activity(self.compositor.target.session_id, false);
                        }
                        self.attachments.render_attachment.set_suspended(false);
                        self.compositor.io_state = ClientIoState::Active;
                        self.compositor.render.last_render.clear();
                        self.compositor.render.force_clear = true;
                        self.status.status_cache.invalidate();
                    }
                    Message::Detach(_) | Message::DetachKill(_) => {
                        // A server-driven detach (rare on the inbound path): run
                        // the graceful handshake below, like a `C-b d` detach.
                        return Ok(Some(AttachDrive::Finish(AttachFinishReason::Detached)));
                    }
                    Message::Exit(_) | Message::Shutdown => {
                        return Ok(Some(AttachDrive::Finish(
                            AttachFinishReason::ConnectionClosed,
                        )));
                    }
                    _ => {
                        // Ignore other control frames while attached.
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                return Ok(Some(AttachDrive::Finish(
                    AttachFinishReason::ConnectionClosed,
                )));
            }
            Err(_) => {
                // Treat as detach on error.
                return Ok(Some(AttachDrive::Finish(
                    AttachFinishReason::ConnectionClosed,
                )));
            }
        }
        Ok(None)
    }
}
