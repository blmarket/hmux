use super::*;

impl AttachSession {
    /// Whether this client could be attached at all, without side effects.
    ///
    /// tmux's `cmd_new_session_exec` opens the terminal *before* it creates
    /// anything, so a client with no controlling terminal fails without
    /// leaving a session behind; running the same test up front is what keeps
    /// the interactive create path from leaking one.
    pub(crate) fn check_terminal(client_tty: &ClientTty) -> Result<(), AttachStartFailure> {
        let failure = || {
            AttachStartFailure::Client("open terminal failed: not a terminal\n".to_string())
        };
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

    pub(crate) fn start_in_mode<W>(
        target: &str,
        client_tty: ClientTty,
        state: &Arc<Mutex<ServerState>>,
        hub: &StatusHub,
        context: &command::ClientContext,
        writer: &mut W,
        pane_io_mode: PaneIoMode,
    ) -> Result<Self, AttachStartFailure>
    where
        W: FrameWriter + ?Sized,
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

        let (cols, rows) = get_winsize(render_fd.as_raw_fd()).unwrap_or((80, 24));
        let (prompt_registry, render_registry, session_id) = {
            let mut st = state
                .lock()
                .map_err(|_| io::Error::other("state poisoned"))?;
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
            super::super::state::ClientFlagState::default()
                .display_flags_with(client_tty.flags, false),
            client_tty.flags,
            Default::default(),
            false,
        )?;

        let stable_target = format!("${session_id}");
        let mut attached_context = context.clone();
        attached_context.current_session_id = Some(session_id);
        attached_context.wait_for_interactions = false;
        let compositor = AttachCompositorState::new(session_id, attached_context, stable_target);
        let target = compositor.target.stable_target.as_str();
        let terminal_identity = TerminalIdentity::new(
            client_tty.term.clone().unwrap_or_default(),
            client_tty.terminfo.clone(),
            client_tty.features,
            context.env("COLORTERM").map(str::to_string),
        )
        .with_utf8(client_tty.flags & 0x10000 != 0);
        let (status_h, status_interval, terminal) = {
            let st = state
                .lock()
                .map_err(|_| io::Error::other("state poisoned"))?;
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
        let mut status_cache = status::RenderCache::for_client(status::ClientContext {
            term: (!terminal.name().is_empty()).then(|| terminal.name().to_string()),
            tty: client_tty.tty_name.clone(),
            pid: client_tty.client_pid,
            cwd: context.cwd.clone(),
            environment: context.environment.clone(),
            ..status::ClientContext::default()
        });
        let agent_status_subscription = hub.subscribe()?;
        status_cache.update_agents(hub.snapshot());
        let pane_rows = rows.saturating_sub(status_h).max(1);
        {
            let mut st = state
                .lock()
                .map_err(|_| io::Error::other("state poisoned"))?;
            let _ = st.resize_session(target, cols, pane_rows);
        }

        let saved_termios = make_raw(input_fd.as_raw_fd()).ok();
        let termios_guard = TermiosGuard {
            fd: input_fd.as_raw_fd(),
            saved: saved_termios,
        };
        set_nonblock(input_fd.as_raw_fd())?;
        set_nonblock(render_fd.as_raw_fd())?;

        let mut tty_output = TtyOutput::new();
        let tty_start = tty_start_sequence(&terminal);
        let _ = tty_output.queue(render_fd.as_raw_fd(), &tty_start);
        if state
            .lock()
            .ok()
            .is_some_and(|st| st.option_for_target(target, "mouse") == Some("on"))
        {
            let _ = tty_output.queue(render_fd.as_raw_fd(), b"\x1b[?1000h\x1b[?1002h\x1b[?1006h");
        }
        let (subscribed_window, output_subscription) = {
            let st = state
                .lock()
                .map_err(|_| io::Error::other("state poisoned"))?;
            active_window_output_subscription(&st, target)?
        };
        let latmon = LatMon::new(format!("sess={target}"));

        Ok(Self {
            tty: AttachTty {
                termios_guard,
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
            },
            status: AttachStatus {
                status_timer,
                status_cache,
            },
            pane_io: AttachPaneIo {
                mode: pane_io_mode,
                latmon,
            },
            commands: AttachCommands {
                pending: None,
                deferred_prompts: VecDeque::new(),
            },
            compositor,
            finish: AttachFinishState::Running,
        })
    }

    pub(crate) fn take_command_request(&mut self) -> Option<AttachCommandRequest> {
        self.commands
            .pending
            .take()
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
        state: &Arc<Mutex<ServerState>>,
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
                    if let Ok(mut state) = state.lock() {
                        let _ = state.resize_session(&target, cols, pane_rows);
                    }
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
                    let _ = reply.send(super::super::state::PromptCompletion {
                        stdout: result.stdout,
                        stderr: result.stderr,
                        exit: result.exit,
                        inserted,
                    });
                }
                self.compositor.render.force_clear = true;
            }
            AttachCommandContinuation::Prompt { mut prompt } => {
                prompt.apply_deferred_side_effect(&result, state);
                prompt.complete(&result, state, &self.compositor.target.context);
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
                            state.lock().ok().and_then(|state| {
                                state
                                    .option_for_target(&target, "display-time")
                                    .and_then(|value| value.parse().ok())
                            })
                        })
                        .unwrap_or(750);
                    self.compositor.ui.status_message = Some(StatusMessage {
                        text,
                        deadline: Instant::now()
                            .checked_add(Duration::from_millis(milliseconds))
                            .unwrap_or_else(Instant::now),
                    });
                }
                self.compositor.render.force_clear = true;
            }
            AttachCommandContinuation::Ignore => {
                self.compositor.render.force_clear = true;
            }
        }
        self.compositor.render.last_render.clear();
    }

    pub(super) fn begin_finish(&mut self, reason: AttachFinishReason) -> AttachDrive {
        if self.finish == AttachFinishState::Running {
            let tty_stop = tty_stop_sequence(&self.tty.terminal, self.viewport.rows);
            let _ = self
                .tty
                .output
                .queue(self.tty.render_fd.as_raw_fd(), &tty_stop);
            self.finish = AttachFinishState::DrainingTty { reason };
        }
        AttachDrive::Continue
    }

    fn prepare_finish(&self, control_fd: RawFd, control_buffered: bool) -> AttachPrepared {
        match self.finish {
            AttachFinishState::Running => unreachable!("finish preparation while running"),
            AttachFinishState::DrainingTty { .. } => {
                if self.tty.output.has_pending() {
                    AttachPrepared::Wait {
                        sources: AttachWaitSources {
                            control: -1,
                            input: -1,
                            tty_output: self.tty.render_fd.as_raw_fd(),
                            output: -1,
                            output_generation: self.attachments.output_generation,
                            prompt: -1,
                            render: -1,
                            status: -1,
                            popup_read: -1,
                            popup_write: -1,
                        },
                        timeout: -1,
                    }
                } else {
                    AttachPrepared::Ready(AttachWaitReady::default())
                }
            }
            AttachFinishState::WaitingForAck { deadline } => {
                if control_buffered {
                    AttachPrepared::Ready(AttachWaitReady {
                        control: true,
                        ..AttachWaitReady::default()
                    })
                } else {
                    AttachPrepared::Wait {
                        sources: AttachWaitSources {
                            control: control_fd,
                            input: -1,
                            tty_output: -1,
                            output: -1,
                            output_generation: self.attachments.output_generation,
                            prompt: -1,
                            render: -1,
                            status: -1,
                            popup_read: -1,
                            popup_write: -1,
                        },
                        timeout: deadline_poll_timeout(Some(deadline), Instant::now()),
                    }
                }
            }
            AttachFinishState::Done => AttachPrepared::Finished,
        }
    }

    fn drive_finish<R, W>(
        &mut self,
        state: &Arc<Mutex<ServerState>>,
        ready: AttachWaitReady,
        reader: &mut R,
        writer: &mut W,
    ) -> io::Result<AttachDrive>
    where
        R: AttachFrameReader,
        W: FrameWriter,
    {
        match self.finish {
            AttachFinishState::Running => unreachable!("finish drive while running"),
            AttachFinishState::DrainingTty { reason } => {
                if ready.tty_output {
                    self.tty.output.flush(self.tty.render_fd.as_raw_fd())?;
                }
                if self.tty.output.has_pending() {
                    return Ok(AttachDrive::Continue);
                }

                let _ = set_blocking(self.tty.input_fd.as_raw_fd());
                let _ = set_blocking(self.tty.render_fd.as_raw_fd());
                self.tty.termios_guard.restore_and_disarm();

                match reason {
                    AttachFinishReason::Detached => {
                        let session_name = state
                            .lock()
                            .ok()
                            .and_then(|st| {
                                st.sessions()
                                    .iter()
                                    .find(|candidate| {
                                        candidate.id == self.compositor.target.session_id
                                    })
                                    .map(|candidate| candidate.name.clone())
                            })
                            .unwrap_or_else(|| self.compositor.target.stable_target.clone());
                        writer.send(Frame::new(Message::Detach(Some(session_name))))?;
                    }
                    AttachFinishReason::SessionEnded => {
                        writer.send(Frame::new(Message::Exit(Some(0))))?;
                    }
                    AttachFinishReason::ConnectionClosed => {
                        self.finish = AttachFinishState::Done;
                        return Ok(AttachDrive::Finished);
                    }
                }
                self.finish = AttachFinishState::WaitingForAck {
                    deadline: Instant::now() + Duration::from_secs(2),
                };
                Ok(AttachDrive::Continue)
            }
            AttachFinishState::WaitingForAck { deadline } => {
                let acknowledged = if ready.control {
                    match reader.try_recv() {
                        Ok(frame) => matches!(frame.msg, Message::Exiting | Message::Exit(_)),
                        Err(error) if error.kind() == io::ErrorKind::WouldBlock => false,
                        Err(_) => true,
                    }
                } else {
                    false
                };
                if !acknowledged && Instant::now() < deadline {
                    return Ok(AttachDrive::Continue);
                }
                writer.send(Frame::new(Message::Exited))?;
                self.finish = AttachFinishState::Done;
                Ok(AttachDrive::Finished)
            }
            AttachFinishState::Done => Ok(AttachDrive::Finished),
        }
    }

    pub(crate) fn prepare_wait(
        &mut self,
        state: &Arc<Mutex<ServerState>>,
        control_fd: RawFd,
        control_buffered: bool,
    ) -> io::Result<AttachPrepared> {
        if self.finish != AttachFinishState::Running {
            return Ok(self.prepare_finish(control_fd, control_buffered));
        }
        if let Some(transition) = self.compositor.transition.take() {
            match transition {
                AttachTransition::SwitchSession(session_id) => {
                    self.compositor.target.switch_session(session_id);
                    let target = self.compositor.target.stable_target.as_str();
                    if let Ok(mut st) = state.lock() {
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
                AttachTransition::Finish(reason) => {
                    self.begin_finish(reason);
                    return Ok(self.prepare_finish(control_fd, control_buffered));
                }
            }
        }
        let stable_target = self.compositor.target.stable_target.clone();
        let target = stable_target.as_str();

        let target_exists = match state.lock() {
            Ok(mut st) => {
                if st.reap_exited_panes() {
                    let _ = st.resize_session(target, self.viewport.cols, self.viewport.pane_rows);
                    self.compositor.render.last_render.clear();
                    self.compositor.render.force_clear = true;
                    self.status.status_cache.invalidate();
                }
                st.find(target).is_some()
            }
            Err(_) => false,
        };
        if !target_exists {
            self.begin_finish(AttachFinishReason::SessionEnded);
            return Ok(self.prepare_finish(control_fd, control_buffered));
        }

        match refresh_active_window_output_subscription(
            state,
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
                self.begin_finish(AttachFinishReason::SessionEnded);
                return Ok(self.prepare_finish(control_fd, control_buffered));
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

        let now = Instant::now();
        let timeout = minimum_poll_timeout(
            self.status.status_timer.poll_timeout(now),
            deadline_poll_timeout(
                self.compositor
                    .ui
                    .status_message
                    .as_ref()
                    .map(|message| message.deadline),
                now,
            ),
        );
        let timeout = minimum_poll_timeout(
            timeout,
            self.compositor
                .ui
                .active_overlay
                .as_ref()
                .map(|overlay| overlay.poll_timeout(now))
                .unwrap_or_else(|| {
                    if state.lock().ok().is_some_and(|st| {
                        st.active_mode_view(target)
                            .is_some_and(|view| view.kind == ModeKind::Clock)
                    }) {
                        1000
                    } else {
                        -1
                    }
                }),
        );
        let timeout = minimum_poll_timeout(
            timeout,
            deadline_poll_timeout(self.compositor.input.key_prompt.deadline(), now),
        );
        let timeout = minimum_poll_timeout(
            timeout,
            deadline_poll_timeout(
                self.compositor
                    .input
                    .terminal_reply
                    .as_ref()
                    .map(|reply| reply.deadline),
                now,
            ),
        );
        let tty_backpressured = self.tty.output.has_pending();
        if !tty_backpressured && control_buffered {
            return Ok(AttachPrepared::Ready(AttachWaitReady {
                control: true,
                ..AttachWaitReady::default()
            }));
        }

        Ok(AttachPrepared::Wait {
            sources: AttachWaitSources {
                control: if tty_backpressured { -1 } else { control_fd },
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
            timeout: if tty_backpressured { -1 } else { timeout },
        })
    }

    pub(crate) fn drive_ready<R, W>(
        &mut self,
        state: &Arc<Mutex<ServerState>>,
        hub: &StatusHub,
        ready: AttachWaitReady,
        reader: &mut R,
        writer: &mut W,
    ) -> io::Result<AttachDrive>
    where
        R: AttachFrameReader,
        W: FrameWriter,
    {
        if self.finish != AttachFinishState::Running {
            return self.drive_finish(state, ready, reader, writer);
        }
        if ready.popup_read || ready.popup_write {
            if let Some(overlay) = self.compositor.ui.active_overlay.as_mut() {
                overlay.drive_popup_io(ready.popup_read, ready.popup_write)?;
            }
        }
        if ready.tty_output {
            self.tty.output.flush(self.tty.render_fd.as_raw_fd())?;
        }
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
        Ok(AttachDrive::Continue)
    }

    fn handle_notifications<W>(
        &mut self,
        state: &Arc<Mutex<ServerState>>,
        hub: &StatusHub,
        ready: &AttachWaitReady,
        writer: &mut W,
    ) -> io::Result<AttachNotificationOutcome>
    where
        W: FrameWriter,
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
        let status_timer_ready = self.status.status_timer.take_expired(now);
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
            for message in self.attachments.render_attachment.take_messages() {
                self.compositor.ui.status_message = Some(StatusMessage {
                    text: message.text,
                    deadline: Instant::now() + Duration::from_millis(message.duration_ms),
                });
                self.compositor.ui.confirm = None;
                self.compositor.render.last_render.clear();
                self.compositor.render.force_clear = true;
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
                        self.compositor.io_state = ClientIoState::Suspended;
                        self.compositor.render.last_render.clear();
                        self.compositor.render.force_clear = true;
                    }
                    ClientAction::Detach => {
                        return Ok(AttachNotificationOutcome::Return(
                            self.begin_finish(AttachFinishReason::Detached),
                        ));
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
                        let encoded = base64_encode(&data);
                        if let Some(sequence) = term::expand_capability(
                            &self.tty.terminal,
                            "Ms",
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
                                let _ = reply.send(super::super::state::PromptCompletion {
                                    stdout: String::new(),
                                    stderr: String::new(),
                                    exit: 0,
                                    inserted: false,
                                });
                            }
                        } else if self.compositor.ui.active_overlay.is_some() {
                            if let Some(reply) = reply {
                                let _ = reply.send(super::super::state::PromptCompletion {
                                    stdout: String::new(),
                                    stderr: String::new(),
                                    exit: 0,
                                    inserted: false,
                                });
                            }
                        } else {
                            self.compositor.ui.active_overlay = ActiveOverlay::from_request(
                                request,
                                reply,
                                self.viewport.cols,
                                self.viewport.rows,
                                self.pane_io.mode,
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
            return Ok(AttachNotificationOutcome::Return(
                self.begin_finish(AttachFinishReason::SessionEnded),
            ));
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
            let mut st = state
                .lock()
                .map_err(|_| io::Error::other("state poisoned"))?;
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
            state,
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
                return Ok(AttachNotificationOutcome::Return(
                    self.begin_finish(AttachFinishReason::SessionEnded),
                ));
            }
            Err(error) => return Err(error),
        }
        if output_ready {
            self.attachments.output_subscription.drain();
            self.status.status_cache.invalidate();
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
        state: &Arc<Mutex<ServerState>>,
        reader: &mut R,
        writer: &mut W,
    ) -> io::Result<Option<AttachDrive>>
    where
        R: AttachFrameReader,
        W: FrameWriter,
    {
        let target = self.compositor.target.stable_target.as_str();
        match reader.try_recv() {
            Ok(frame) => {
                if frame.version != PROTOCOL_VERSION {
                    let _ = writer.send(Frame::new(Message::Version));
                    return Ok(Some(
                        self.begin_finish(AttachFinishReason::ConnectionClosed),
                    ));
                }
                match frame.msg {
                    Message::Resize => {
                        if let Ok((new_cols, new_rows)) =
                            get_winsize(self.tty.render_fd.as_raw_fd())
                        {
                            if new_cols != self.viewport.cols || new_rows != self.viewport.rows {
                                self.viewport.cols = new_cols;
                                self.viewport.rows = new_rows;
                                if let Some(overlay) = self.compositor.ui.active_overlay.as_mut() {
                                    overlay.resize(self.viewport.cols, self.viewport.rows);
                                }
                                self.attachments
                                    .render_attachment
                                    .update_size(self.viewport.cols, self.viewport.rows);
                                if let Ok(mut st) = state.lock() {
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
                        let start = tty_start_sequence(&self.tty.terminal);
                        let _ = self
                            .tty
                            .output
                            .queue(self.tty.render_fd.as_raw_fd(), &start);
                        self.compositor.render.output_cursor_visible = None;
                        if state
                            .lock()
                            .ok()
                            .is_some_and(|st| st.option_for_target(target, "mouse") == Some("on"))
                        {
                            let _ = self.tty.output.queue(
                                self.tty.render_fd.as_raw_fd(),
                                b"\x1b[?1000h\x1b[?1002h\x1b[?1006h",
                            );
                        }
                        self.compositor.io_state = ClientIoState::Active;
                        self.compositor.render.last_render.clear();
                        self.compositor.render.force_clear = true;
                        self.status.status_cache.invalidate();
                    }
                    Message::Wakeup if self.compositor.io_state == ClientIoState::Suspended => {
                        let _ = make_raw(self.tty.input_fd.as_raw_fd());
                        let start = tty_start_sequence(&self.tty.terminal);
                        let _ = self
                            .tty
                            .output
                            .queue(self.tty.render_fd.as_raw_fd(), &start);
                        self.compositor.render.output_cursor_visible = None;
                        if state
                            .lock()
                            .ok()
                            .is_some_and(|st| st.option_for_target(target, "mouse") == Some("on"))
                        {
                            let _ = self.tty.output.queue(
                                self.tty.render_fd.as_raw_fd(),
                                b"\x1b[?1000h\x1b[?1002h\x1b[?1006h",
                            );
                        }
                        self.compositor.io_state = ClientIoState::Active;
                        self.compositor.render.last_render.clear();
                        self.compositor.render.force_clear = true;
                        self.status.status_cache.invalidate();
                    }
                    Message::Detach(_) | Message::DetachKill(_) => {
                        // A server-driven detach (rare on the inbound path): run
                        // the graceful handshake below, like a `C-b d` detach.
                        return Ok(Some(self.begin_finish(AttachFinishReason::Detached)));
                    }
                    Message::Exit(_) | Message::Shutdown => {
                        return Ok(Some(
                            self.begin_finish(AttachFinishReason::ConnectionClosed),
                        ));
                    }
                    _ => {
                        // Ignore other control frames while attached.
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                return Ok(Some(
                    self.begin_finish(AttachFinishReason::ConnectionClosed),
                ));
            }
            Err(_) => {
                // Treat as detach on error.
                return Ok(Some(
                    self.begin_finish(AttachFinishReason::ConnectionClosed),
                ));
            }
        }
        Ok(None)
    }
}
