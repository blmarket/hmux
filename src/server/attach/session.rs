use super::*;

impl AttachSession {
    pub(crate) fn start<W>(
        target: &str,
        client_tty: ClientTty,
        state: &Arc<Mutex<ServerState>>,
        hub: &StatusHub,
        context: &command::ClientContext,
        writer: &mut W,
    ) -> Result<Self, AttachStartFailure>
    where
        W: FrameWriter + ?Sized,
    {
        Self::start_in_mode(
            target,
            client_tty,
            state,
            hub,
            context,
            writer,
            PaneIoMode::Threaded(crate::native::pane::spawn_reader),
        )
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
            let st = state
                .lock()
                .map_err(|_| io::Error::other("state poisoned"))?;
            let session_id = st.session_id(target).ok_or_else(|| {
                AttachStartFailure::Client(format!("can't find session: {target}\n"))
            })?;
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
            String::new(),
            false,
            false,
        )?;

        let stable_target = format!("${session_id}");
        let mut attached_context = context.clone();
        attached_context.current_session_id = Some(session_id);
        attached_context.wait_for_interactions = false;
        let compositor = AttachCompositorState::new(session_id, attached_context, stable_target);
        let target = compositor.stable_target.as_str();
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

        let termios_guard = TermiosGuard {
            fd: input_fd.as_raw_fd(),
            saved: make_raw(input_fd.as_raw_fd()).ok(),
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
                latmon: LatMon::new(format!("sess={target}")),
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
                    .command_prompt
                    .as_mut()?
                    .take_deferred_incremental()?;
                Some(AttachCommandRequest {
                    source,
                    context: self.compositor.context.clone(),
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
                self.compositor.force_clear = true;
            }
            AttachCommandContinuation::Overlay {
                mut overlay,
                inserted,
            } => {
                overlay.complete(result, inserted);
                self.compositor.force_clear = true;
            }
            AttachCommandContinuation::Confirm { reply, inserted } => {
                if let Some(reply) = reply {
                    let _ = reply.send(crate::server::state::PromptCompletion {
                        stdout: result.stdout,
                        stderr: result.stderr,
                        exit: result.exit,
                        inserted,
                    });
                }
                self.compositor.force_clear = true;
            }
            AttachCommandContinuation::Prompt { mut prompt } => {
                prompt.apply_deferred_side_effect(&result, state);
                prompt.complete(&result, state, &self.compositor.context);
                self.compositor.force_clear = true;
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
                    self.compositor.status_message = Some((
                        text,
                        Instant::now()
                            .checked_add(Duration::from_millis(milliseconds))
                            .unwrap_or_else(Instant::now),
                    ));
                }
                self.compositor.force_clear = true;
            }
            AttachCommandContinuation::Ignore => {
                self.compositor.force_clear = true;
            }
        }
        self.compositor.last_render.clear();
    }

    fn begin_finish(&mut self) -> AttachDrive {
        if self.finish == AttachFinishState::Running {
            let tty_stop = tty_stop_sequence(&self.tty.terminal, self.viewport.rows);
            let _ = self
                .tty
                .output
                .queue(self.tty.render_fd.as_raw_fd(), &tty_stop);
            self.finish = AttachFinishState::DrainingTty;
        }
        AttachDrive::Continue
    }

    fn prepare_finish(&self, control_fd: RawFd, control_buffered: bool) -> AttachPrepared {
        match self.finish {
            AttachFinishState::Running => unreachable!("finish preparation while running"),
            AttachFinishState::DrainingTty => {
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
            AttachFinishState::DrainingTty => {
                if ready.tty_output {
                    self.tty.output.flush(self.tty.render_fd.as_raw_fd())?;
                }
                if self.tty.output.has_pending() {
                    return Ok(AttachDrive::Continue);
                }

                let _ = set_blocking(self.tty.input_fd.as_raw_fd());
                let _ = set_blocking(self.tty.render_fd.as_raw_fd());
                self.tty.termios_guard.restore_and_disarm();

                if self.compositor.detach_requested {
                    let session_name = state
                        .lock()
                        .ok()
                        .and_then(|st| {
                            st.sessions()
                                .iter()
                                .find(|candidate| candidate.id == self.compositor.session_id)
                                .map(|candidate| candidate.name.clone())
                        })
                        .unwrap_or_else(|| self.compositor.stable_target.clone());
                    writer.send(Frame::new(Message::Detach(Some(session_name))))?;
                    self.finish = AttachFinishState::WaitingForAck {
                        deadline: Instant::now() + Duration::from_secs(2),
                    };
                    return Ok(AttachDrive::Continue);
                }
                if self.compositor.session_ended {
                    writer.send(Frame::new(Message::Exit(Some(0))))?;
                    self.finish = AttachFinishState::WaitingForAck {
                        deadline: Instant::now() + Duration::from_secs(2),
                    };
                    return Ok(AttachDrive::Continue);
                }
                self.finish = AttachFinishState::Done;
                Ok(AttachDrive::Finished)
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
        if let Some(new_session_id) = self.compositor.switch_to.take() {
            self.compositor.session_id = new_session_id;
            self.compositor.stable_target = format!("${}", self.compositor.session_id);
            self.compositor.context.current_session_id = Some(self.compositor.session_id);
            let target = self.compositor.stable_target.as_str();
            if let Ok(mut st) = state.lock() {
                self.viewport.status_height = status::height(&st, target);
                self.viewport.pane_rows = self
                    .viewport
                    .rows
                    .saturating_sub(self.viewport.status_height)
                    .max(1);
                let _ = st.resize_session(target, self.viewport.cols, self.viewport.pane_rows);
                self.status
                    .status_timer
                    .configure(status::interval(&st, target), Instant::now());
            }
            self.status.status_cache.invalidate();
            self.compositor.last_render.clear();
            self.compositor.force_clear = true;
        }
        let target = self.compositor.stable_target.as_str();
        if self.compositor.should_exit {
            self.begin_finish();
            return Ok(self.prepare_finish(control_fd, control_buffered));
        }

        let target_exists = match state.lock() {
            Ok(mut st) => {
                if st.reap_exited_panes() {
                    let _ = st.resize_session(target, self.viewport.cols, self.viewport.pane_rows);
                    self.compositor.last_render.clear();
                    self.compositor.force_clear = true;
                    self.status.status_cache.invalidate();
                }
                st.find(target).is_some()
            }
            Err(_) => false,
        };
        if !target_exists {
            self.compositor.session_ended = true;
            self.begin_finish();
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
                self.compositor.last_render.clear();
                self.compositor.force_clear = true;
                self.status.status_cache.invalidate();
            }
            Ok(false) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                self.compositor.session_ended = true;
                self.begin_finish();
                return Ok(self.prepare_finish(control_fd, control_buffered));
            }
            Err(error) => return Err(error),
        }

        let (popup_read, popup_write) = self
            .compositor
            .active_overlay
            .as_ref()
            .map(ActiveOverlay::sources)
            .unwrap_or((-1, -1));
        if self
            .compositor
            .active_overlay
            .as_ref()
            .is_some_and(ActiveOverlay::read_continuation)
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
                    .status_message
                    .as_ref()
                    .map(|(_, deadline)| *deadline),
                now,
            ),
        );
        let timeout = minimum_poll_timeout(
            timeout,
            self.compositor
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
            deadline_poll_timeout(self.compositor.key_prompt_deadline, now),
        );
        let timeout = minimum_poll_timeout(
            timeout,
            deadline_poll_timeout(self.compositor.terminal_reply_deadline, now),
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
                input: if tty_backpressured || self.compositor.locked || self.compositor.suspended {
                    -1
                } else {
                    self.tty.input_fd.as_raw_fd()
                },
                tty_output: if tty_backpressured {
                    self.tty.render_fd.as_raw_fd()
                } else {
                    -1
                },
                output: if tty_backpressured || self.compositor.locked || self.compositor.suspended
                {
                    -1
                } else {
                    self.attachments.output_subscription.as_raw_fd()
                },
                output_generation: self.attachments.output_generation,
                prompt: if tty_backpressured || self.compositor.locked || self.compositor.suspended
                {
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
        let target = self.compositor.stable_target.as_str();
        if ready.popup_read || ready.popup_write {
            if let Some(overlay) = self.compositor.active_overlay.as_mut() {
                overlay.drive_io(ready.popup_read, ready.popup_write)?;
            }
        }
        if ready.tty_output {
            self.tty.output.flush(self.tty.render_fd.as_raw_fd())?;
        }
        if self.tty.output.has_pending() {
            return Ok(AttachDrive::Continue);
        }
        let control_ready = ready.control;
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
        let overlay_tick = self.compositor.active_overlay.is_some();
        let mut overlay_exit = 0;
        let overlay_expired = match self.compositor.active_overlay.as_mut() {
            Some(ActiveOverlay {
                state: OverlayState::DisplayPanes(display),
                ..
            }) => display.deadline <= now,
            Some(ActiveOverlay {
                state:
                    OverlayState::Popup(PopupOverlay {
                        request,
                        pane,
                        exit_status,
                        ..
                    }),
                ..
            }) if pane.has_exited() => {
                if exit_status.is_none() {
                    *exit_status = pane.try_wait();
                }
                if let Some(exit) = *exit_status {
                    overlay_exit = exit;
                    request.close_on_exit || (request.close_on_success && exit == 0)
                } else {
                    false
                }
            }
            _ => false,
        };
        if overlay_expired {
            if let Some(mut overlay) = self.compositor.active_overlay.take() {
                let mut result = command::CommandResult::ok("");
                result.exit = overlay_exit;
                overlay.complete(result, false);
            }
            self.compositor.last_render.clear();
            self.compositor.force_clear = true;
        }
        let message_expired = self
            .compositor
            .status_message
            .as_ref()
            .is_some_and(|(_, deadline)| *deadline <= now);
        if message_expired {
            self.compositor.status_message = None;
            self.compositor.last_render.clear();
        }
        if status_timer_ready {
            self.status.status_cache.invalidate();
        }
        let render_invalidation = if render_ready {
            self.attachments.render_attachment.take()
        } else {
            crate::server::state::RenderInvalidation::default()
        };
        if render_ready {
            for message in self.attachments.render_attachment.take_messages() {
                self.compositor.status_message = Some((
                    message.text,
                    Instant::now() + Duration::from_millis(message.duration_ms),
                ));
                self.compositor.confirm = None;
                self.compositor.last_render.clear();
                self.compositor.force_clear = true;
            }
            if let Some(action) = self.attachments.render_attachment.take_action() {
                match action {
                    ClientAction::Lock(command) if !self.compositor.locked => {
                        let stop = tty_stop_sequence(&self.tty.terminal, self.viewport.rows);
                        let _ = self.tty.output.queue(self.tty.render_fd.as_raw_fd(), &stop);
                        self.compositor.output_cursor_visible = None;
                        self.tty.termios_guard.restore();
                        writer.send(Frame::new(Message::Lock(command)))?;
                        self.compositor.locked = true;
                        self.compositor.last_render.clear();
                        self.compositor.force_clear = true;
                    }
                    ClientAction::Suspend if !self.compositor.suspended => {
                        let stop = tty_stop_sequence(&self.tty.terminal, self.viewport.rows);
                        let _ = self.tty.output.queue(self.tty.render_fd.as_raw_fd(), &stop);
                        self.compositor.output_cursor_visible = None;
                        self.tty.termios_guard.restore();
                        writer.send(Frame::new(Message::Suspend))?;
                        self.compositor.suspended = true;
                        self.compositor.last_render.clear();
                        self.compositor.force_clear = true;
                    }
                    ClientAction::Detach => {
                        self.compositor.detach_requested = true;
                        return Ok(self.begin_finish());
                    }
                    ClientAction::Switch(new_session_id) => {
                        self.compositor.switch_to = Some(new_session_id);
                        return Ok(AttachDrive::Continue);
                    }
                    ClientAction::Keys(keys)
                        if !self.compositor.locked && !self.compositor.suspended =>
                    {
                        self.compositor.injected_input.extend(keys);
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
                            if let Some(mut overlay) = self.compositor.active_overlay.take() {
                                overlay.complete(command::CommandResult::ok(""), false);
                            }
                            if let Some(reply) = reply {
                                let _ = reply.send(crate::server::state::PromptCompletion {
                                    stdout: String::new(),
                                    stderr: String::new(),
                                    exit: 0,
                                    inserted: false,
                                });
                            }
                        } else if self.compositor.active_overlay.is_some() {
                            if let Some(reply) = reply {
                                let _ = reply.send(crate::server::state::PromptCompletion {
                                    stdout: String::new(),
                                    stderr: String::new(),
                                    exit: 0,
                                    inserted: false,
                                });
                            }
                        } else {
                            self.compositor.active_overlay = ActiveOverlay::from_request(
                                request,
                                reply,
                                self.viewport.cols,
                                self.viewport.rows,
                                self.pane_io.mode,
                            )
                            .ok()
                            .flatten();
                        }
                        self.compositor.last_render.clear();
                        self.compositor.force_clear = true;
                    }
                    ClientAction::Confirm {
                        prompt,
                        command,
                        confirm_key,
                        default_yes,
                        reply,
                    } => {
                        self.compositor.confirm = Some(ActiveConfirm {
                            prompt,
                            action: ConfirmAction::Command(command),
                            confirm_key,
                            default_yes,
                            reply,
                        });
                        self.compositor.last_render.clear();
                        self.compositor.force_clear = true;
                    }
                    ClientAction::Lock(_) => {}
                    ClientAction::Suspend => {}
                    ClientAction::Keys(_) => {}
                }
            }
        }
        if render_invalidation.contains(crate::server::state::RenderInvalidation::SESSION_GONE) {
            self.compositor.session_ended = true;
            return Ok(self.begin_finish());
        }
        if !render_invalidation.is_empty() {
            self.status.status_cache.invalidate();
        }
        if render_invalidation.contains(crate::server::state::RenderInvalidation::RESET_MODE)
            || render_invalidation.contains(crate::server::state::RenderInvalidation::MODE)
        {
            self.compositor.last_render.clear();
        }
        if render_invalidation.contains(crate::server::state::RenderInvalidation::STATUS) {
            let mut st = state
                .lock()
                .map_err(|_| io::Error::other("state poisoned"))?;
            if render_invalidation.contains(crate::server::state::RenderInvalidation::TERMINAL) {
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
                self.compositor.last_render.clear();
                self.compositor.force_clear = true;
            }
        }
        if prompt_ready && self.compositor.command_prompt.is_none() {
            if let Some(external) = self.attachments.prompt_attachment.take_command_prompt() {
                let args = external.args().to_vec();
                if let Ok(mut prompt) =
                    CommandPrompt::new(args, Some(external), state, hub, &self.compositor.context)
                {
                    if !prompt.request.spec.no_freeze {
                        prompt.presentation.frozen_frame =
                            Some(self.compositor.last_render.clone());
                    }
                    prompt.initial_incremental(state, hub, &self.compositor.context);
                    self.compositor.command_prompt = Some(prompt);
                }
                self.compositor.last_render.clear();
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
                self.compositor.last_render.clear();
                self.compositor.force_clear = true;
                self.status.status_cache.invalidate();
            }
            Ok(false) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                self.compositor.session_ended = true;
                return Ok(self.begin_finish());
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

        // 1. Handle imsg control messages (resize, detach) when poll says that
        // reading cannot block.
        if control_ready {
            match reader.try_recv() {
                Ok(frame) => {
                    if frame.version != PROTOCOL_VERSION {
                        let _ = writer.send(Frame::new(Message::Version));
                        return Ok(self.begin_finish());
                    }
                    match frame.msg {
                        Message::Resize => {
                            if let Ok((new_cols, new_rows)) =
                                get_winsize(self.tty.render_fd.as_raw_fd())
                            {
                                if new_cols != self.viewport.cols || new_rows != self.viewport.rows
                                {
                                    self.viewport.cols = new_cols;
                                    self.viewport.rows = new_rows;
                                    if let Some(overlay) = self.compositor.active_overlay.as_mut() {
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
                                    self.compositor.last_render.clear();
                                    self.compositor.force_clear = true;
                                    self.status.status_cache.invalidate();
                                }
                            }
                        }
                        Message::Unlock if self.compositor.locked => {
                            let _ = make_raw(self.tty.input_fd.as_raw_fd());
                            let start = tty_start_sequence(&self.tty.terminal);
                            let _ = self
                                .tty
                                .output
                                .queue(self.tty.render_fd.as_raw_fd(), &start);
                            self.compositor.output_cursor_visible = None;
                            if state.lock().ok().is_some_and(|st| {
                                st.option_for_target(target, "mouse") == Some("on")
                            }) {
                                let _ = self.tty.output.queue(
                                    self.tty.render_fd.as_raw_fd(),
                                    b"\x1b[?1000h\x1b[?1002h\x1b[?1006h",
                                );
                            }
                            self.compositor.locked = false;
                            self.compositor.last_render.clear();
                            self.compositor.force_clear = true;
                            self.status.status_cache.invalidate();
                        }
                        Message::Wakeup if self.compositor.suspended => {
                            let _ = make_raw(self.tty.input_fd.as_raw_fd());
                            let start = tty_start_sequence(&self.tty.terminal);
                            let _ = self
                                .tty
                                .output
                                .queue(self.tty.render_fd.as_raw_fd(), &start);
                            self.compositor.output_cursor_visible = None;
                            if state.lock().ok().is_some_and(|st| {
                                st.option_for_target(target, "mouse") == Some("on")
                            }) {
                                let _ = self.tty.output.queue(
                                    self.tty.render_fd.as_raw_fd(),
                                    b"\x1b[?1000h\x1b[?1002h\x1b[?1006h",
                                );
                            }
                            self.compositor.suspended = false;
                            self.compositor.last_render.clear();
                            self.compositor.force_clear = true;
                            self.status.status_cache.invalidate();
                        }
                        Message::Detach(_) | Message::DetachKill(_) => {
                            // A server-driven detach (rare on the inbound path): run
                            // the graceful handshake below, like a `C-b d` detach.
                            self.compositor.detach_requested = true;
                            return Ok(self.begin_finish());
                        }
                        Message::Exit(_) | Message::Shutdown => {
                            return Ok(self.begin_finish());
                        }
                        _ => {
                            // Ignore other control frames while attached.
                        }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                    return Ok(self.begin_finish());
                }
                Err(_) => {
                    // Treat as detach on error.
                    return Ok(self.begin_finish());
                }
            }
        }

        if self.compositor.locked || self.compositor.suspended {
            return Ok(AttachDrive::Continue);
        }

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
        let waiting_for_terminal_reply = !self.compositor.terminal_reply_buf.is_empty();
        let mut forward_buf = std::mem::take(&mut self.compositor.terminal_reply_buf);
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
                } else if let Some(key) = self.compositor.injected_input.pop_front() {
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
                        .command_prompt
                        .as_ref()
                        .is_some_and(|prompt| prompt.request.spec.key)
                        && !self.compositor.key_prompt_buf.is_empty()
                    {
                        let decoded = decode_prompt_key(&self.compositor.key_prompt_buf);
                        let could_be_terminal_key = decoded.is_none()
                            && self.compositor.key_prompt_buf.starts_with(b"\x1b")
                            && matches!(self.compositor.key_prompt_buf.get(1), Some(b'[' | b'O'));
                        if could_be_terminal_key {
                            let deadline = *self
                                .compositor
                                .key_prompt_deadline
                                .get_or_insert_with(|| Instant::now() + prompt_escape_delay(state));
                            if Instant::now() < deadline {
                                break;
                            }
                        }
                        let decoded = decoded.or_else(|| {
                            (self.compositor.key_prompt_buf.len() >= 2
                                && self.compositor.key_prompt_buf[0] == 0x1b)
                                .then(|| (meta_prompt_key(self.compositor.key_prompt_buf[1]), 2))
                        });
                        if let Some((key, consumed)) = decoded {
                            let tail = self.compositor.key_prompt_buf[consumed..].to_vec();
                            let request = handle_command_prompt_key(
                                &mut self.compositor.command_prompt,
                                &key,
                                state,
                                hub,
                                &self.compositor.context,
                            );
                            self.compositor.key_prompt_buf.clear();
                            self.compositor.key_prompt_deadline = None;
                            force_render = true;
                            if let Some(request) = request {
                                if !tail.is_empty() {
                                    self.compositor.injected_input.push_front(ClientKey {
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
                        let deadline = *self
                            .compositor
                            .terminal_reply_deadline
                            .get_or_insert_with(|| Instant::now() + Duration::from_millis(2));
                        if Instant::now() < deadline {
                            self.compositor.terminal_reply_buf = std::mem::take(&mut forward_buf);
                        }
                    }
                    break;
                } else {
                    self.compositor.should_exit = true;
                    break;
                }
            } else if n == 0 {
                // EOF on tty: client closed.
                self.compositor.should_exit = true;
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
            let mut prompt_tail = None;
            if self
                .compositor
                .command_prompt
                .as_ref()
                .is_some_and(|prompt| prompt.request.spec.key)
            {
                self.compositor.key_prompt_buf.extend_from_slice(read_data);
                if let Some((key, consumed)) = decode_prompt_key(&self.compositor.key_prompt_buf) {
                    let tail = self.compositor.key_prompt_buf[consumed..].to_vec();
                    if let Some(request) = handle_command_prompt_key(
                        &mut self.compositor.command_prompt,
                        &key,
                        state,
                        hub,
                        &self.compositor.context,
                    ) {
                        self.compositor.key_prompt_buf.clear();
                        self.compositor.key_prompt_deadline = None;
                        if !tail.is_empty() {
                            self.compositor.injected_input.push_front(ClientKey {
                                bytes: tail,
                                forward_unbound,
                            });
                        }
                        self.commands.pending = Some(request);
                        force_render = true;
                        break;
                    }
                    prompt_tail = Some(tail);
                    self.compositor.key_prompt_buf.clear();
                    self.compositor.key_prompt_deadline = None;
                    force_render = true;
                } else if self.compositor.key_prompt_buf.starts_with(b"\x1b")
                    && matches!(self.compositor.key_prompt_buf.get(1), Some(b'[' | b'O'))
                    && self.compositor.key_prompt_deadline.is_none()
                {
                    self.compositor.key_prompt_deadline =
                        Some(Instant::now() + prompt_escape_delay(state));
                }
                if prompt_tail.as_ref().is_none_or(Vec::is_empty) {
                    continue;
                }
            }
            let data = prompt_tail.as_deref().unwrap_or(read_data);
            self.compositor.key_prompt_buf.clear();
            self.compositor.key_prompt_deadline = None;
            let mut i = 0;
            while i < data.len() {
                if self.compositor.active_overlay.is_some() {
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
                    let mut close = false;
                    let mut close_exit = 0;
                    let mut selected_command = None;
                    match &mut self
                        .compositor
                        .active_overlay
                        .as_mut()
                        .expect("overlay checked")
                        .state
                    {
                        OverlayState::Menu(MenuOverlay {
                            request, selected, ..
                        }) => match decoded.name.as_str() {
                            "q" | "Escape" | "C-c" => close = true,
                            "Up" | "k" => *selected = selected.saturating_sub(1),
                            "Down" | "j" => {
                                *selected =
                                    (*selected + 1).min(request.items.len().saturating_sub(1))
                            }
                            "Enter" => {
                                selected_command = request
                                    .items
                                    .get(*selected)
                                    .map(|item| item.command.clone());
                                close = true;
                            }
                            key => {
                                if let Some(item) =
                                    request.items.iter().find(|item| item.key == key)
                                {
                                    selected_command = Some(item.command.clone());
                                    close = true;
                                }
                            }
                        },
                        OverlayState::Popup(PopupOverlay {
                            request,
                            pane,
                            exit_status,
                            ..
                        }) => {
                            if exit_status.is_some()
                                || request.close_on_key
                                || ((decoded.name == "Escape" || decoded.name == "C-c")
                                    && !request.close_on_exit
                                    && !request.close_on_success)
                            {
                                close = true;
                                close_exit = (*exit_status).unwrap_or(129);
                            } else {
                                let _ = pane.input(&data[start..i]);
                            }
                        }
                        OverlayState::DisplayPanes(DisplayPanesOverlay {
                            command,
                            accept_input,
                            ..
                        }) => {
                            if !*accept_input
                                || matches!(decoded.name.as_str(), "Escape" | "q" | "C-c")
                            {
                                close = true;
                            } else if let Some(index) = decoded
                                .name
                                .chars()
                                .next()
                                .filter(|_| decoded.name.chars().count() == 1)
                                .and_then(|value| value.to_digit(10))
                            {
                                let pane_id = state.lock().ok().and_then(|st| {
                                    st.active_window_panes(target)
                                        .ok()
                                        .and_then(|(window, _)| window.panes.get(index as usize))
                                        .map(|pane| pane.id)
                                });
                                if let Some(pane_id) = pane_id {
                                    let source = if command.is_empty() {
                                        vec![
                                            "select-pane".to_string(),
                                            "-t".to_string(),
                                            format!("%{pane_id}"),
                                        ]
                                    } else {
                                        command
                                            .iter()
                                            .map(|word| word.replace("%%", &format!("%{pane_id}")))
                                            .collect()
                                    };
                                    selected_command = Some(source);
                                    close = true;
                                }
                            }
                        }
                    }
                    let inserted = selected_command
                        .as_ref()
                        .is_some_and(|command| !command.is_empty());
                    if let Some(command) = selected_command
                        .as_ref()
                        .filter(|command| !command.is_empty())
                        .filter(|_| self.compositor.context.defer_attach_commands)
                    {
                        let overlay = self
                            .compositor
                            .active_overlay
                            .take()
                            .expect("overlay checked");
                        self.commands.pending = Some(AttachCommandRequest {
                            source: command::DeferredCommand::Args(command.clone()),
                            context: self.compositor.context.clone(),
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
                            &self.compositor.context,
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
                        if let Some(mut overlay) = self.compositor.active_overlay.take() {
                            overlay.complete(
                                result.unwrap_or_else(|| command::CommandResult::ok("")),
                                inserted,
                            );
                        }
                    }
                    force_render = true;
                    continue;
                }
                if let Some(prompt) = self.compositor.command_prompt.as_mut() {
                    let (decoded, consumed) = decode_tty_key(&data[i..])
                        .map(|(key, consumed)| (key.name, consumed))
                        .unwrap_or_else(|| (plain_prompt_key(data[i]), 1));
                    i += consumed;
                    let mut incremental = None;
                    match prompt.handle_key(&decoded, state, hub, &self.compositor.context) {
                        CommandPromptInput::Continue => {
                            incremental = prompt.take_deferred_incremental();
                        }
                        CommandPromptInput::Finish(mut result) => {
                            let mut prompt = self
                                .compositor
                                .command_prompt
                                .take()
                                .expect("command prompt checked");
                            if let Some(source) = take_deferred_attach_command(&mut result) {
                                self.commands.pending = Some(AttachCommandRequest {
                                    source,
                                    context: self.compositor.context.clone(),
                                    continuation: AttachCommandContinuation::Prompt {
                                        prompt: Box::new(prompt),
                                    },
                                });
                                break;
                            }
                            prompt.complete(&result, state, &self.compositor.context);
                        }
                        CommandPromptInput::Cancel => {
                            let mut prompt = self
                                .compositor
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
                                context: self.compositor.context.clone(),
                                continuation: AttachCommandContinuation::Ignore,
                            });
                    }
                    force_render = true;
                    continue;
                }
                if let Some(active) = self.compositor.confirm.take() {
                    // A confirm-before prompt is up: this key answers it and is
                    // consumed whole (so a multi-byte escape can't leak to the
                    // pane). `y`/`Y` runs the guarded command; every other key
                    // cancels, exactly like tmux's client-confirm callback.
                    let (key, consumed) = read_key(&data[i..]);
                    i += consumed;
                    force_render = true;
                    let accepted = matches!(key, Key::Byte(value) if value == active.confirm_key)
                        || (key == Key::Enter && active.default_yes);
                    let result = if accepted {
                        match active.action {
                            ConfirmAction::Command(command) => {
                                if self.compositor.context.defer_attach_commands {
                                    self.commands.pending = Some(AttachCommandRequest {
                                        source: command::DeferredCommand::Args(command),
                                        context: self.compositor.context.clone(),
                                        continuation: AttachCommandContinuation::Confirm {
                                            reply: active.reply,
                                            inserted: true,
                                        },
                                    });
                                    break;
                                }
                                let agents = hub.snapshot().panes;
                                command::run_with_context(
                                    &command,
                                    state,
                                    &agents,
                                    &self.compositor.context,
                                )
                            }
                            action @ (ConfirmAction::KillPane | ConfirmAction::KillWindow) => {
                                let killed = if let Ok(mut st) = state.lock() {
                                    let killed = match action {
                                        ConfirmAction::KillPane => st.kill_pane(target).is_ok(),
                                        ConfirmAction::KillWindow => st.kill_window(target).is_ok(),
                                        ConfirmAction::Command(_) => unreachable!(),
                                    };
                                    // A survivor window/pane inherits the client viewport,
                                    // just like a layout-changing prefix key.
                                    if killed && st.find(target).is_some() {
                                        let _ = st.resize_session(
                                            target,
                                            self.viewport.cols,
                                            self.viewport.pane_rows,
                                        );
                                    }
                                    killed
                                } else {
                                    false
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
                    if let Some(reply) = active.reply {
                        let _ = reply.send(crate::server::state::PromptCompletion {
                            stdout: result.stdout,
                            stderr: result.stderr,
                            exit: result.exit,
                            inserted: accepted,
                        });
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
                            if self.compositor.context.defer_attach_commands {
                                self.commands.pending = Some(AttachCommandRequest {
                                    source: command::DeferredCommand::Args(command),
                                    context: self.compositor.context.clone(),
                                    continuation: AttachCommandContinuation::Ignore,
                                });
                                break;
                            }
                            let agents = hub.snapshot().panes;
                            let _ = command::run_with_context(
                                &command,
                                state,
                                &agents,
                                &self.compositor.context,
                            );
                        }
                        ModeViewKeyResult::Prompt(request) => {
                            if let Ok(mut prompt) = CommandPrompt::for_mode(
                                request,
                                target,
                                state,
                                hub,
                                &self.compositor.context,
                            ) {
                                if !prompt.request.spec.no_freeze {
                                    prompt.presentation.frozen_frame =
                                        Some(self.compositor.last_render.clone());
                                }
                                prompt.initial_incremental(state, hub, &self.compositor.context);
                                self.compositor.command_prompt = Some(prompt);
                            }
                        }
                        ModeViewKeyResult::None | ModeViewKeyResult::Command(_) => {}
                    }
                    force_render = true;
                    continue;
                }
                if self.compositor.prefix_pending {
                    self.compositor.prefix_pending = false;
                    // Flush any keystrokes typed before this command so the pane
                    // sees them in order relative to a possible send-prefix byte.
                    if !forward_buf.is_empty() {
                        first_forward_at.get_or_insert_with(Instant::now);
                        if let Ok(stats) = forward_input(state, target, &forward_buf) {
                            add_input_stats(&mut forwarded, stats);
                        }
                        forward_buf.clear();
                    }
                    // The command key can be a multi-byte escape (e.g. PgUp), so
                    // parse a logical key rather than taking one raw byte.
                    let (key, mouse, consumed) = match decode_tty_key(&data[i..]) {
                        Some((mut decoded, consumed)) => {
                            resolve_mouse_key(
                                &mut decoded,
                                &mut self.compositor.mouse_input,
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
                    match dispatch_key_binding(
                        "prefix",
                        key,
                        state,
                        target,
                        self.viewport.cols,
                        self.viewport.pane_rows,
                        hub,
                        &self.compositor.context,
                        mouse,
                    ) {
                        PrefixOutcome::Detach => {
                            self.compositor.detach_requested = true;
                            self.compositor.should_exit = true;
                            break;
                        }
                        PrefixOutcome::SendPrefix(bytes) => forward_buf.extend(bytes),
                        PrefixOutcome::CopyMode {
                            page_up,
                            page_down,
                            slider,
                            mouse,
                            begin_selection,
                        } => {
                            set_copy_mode_state(state, target, true, page_up);
                            if let Some(mouse) = mouse {
                                if let Ok(mut st) = state.lock() {
                                    let vi = copy_mode_uses_vi_keys(&st, target);
                                    let position = mouse.pane_position();
                                    let _ = st.position_copy_cursor_from_mouse(
                                        target, position.x, position.y, vi,
                                    );
                                    if slider {
                                        let _ = st.set_copy_scroll_from_mouse(
                                            target,
                                            position.y,
                                            self.viewport.pane_rows,
                                            vi,
                                        );
                                    }
                                    if begin_selection {
                                        let separators = st
                                            .option_for_target(target, "word-separators")
                                            .unwrap_or("")
                                            .to_string();
                                        let _ = st.copy_mode_command(
                                            target,
                                            "begin-selection",
                                            vi,
                                            &separators,
                                        );
                                    }
                                }
                            }
                            if page_down {
                                if let Ok(mut st) = state.lock() {
                                    let vi = copy_mode_uses_vi_keys(&st, target);
                                    let separators = st
                                        .option_for_target(target, "word-separators")
                                        .unwrap_or("")
                                        .to_string();
                                    let _ =
                                        st.copy_mode_command(target, "page-down", vi, &separators);
                                }
                            }
                            force_render = true;
                        }
                        PrefixOutcome::Confirm { prompt, action } => {
                            self.compositor.confirm = Some(ActiveConfirm {
                                prompt,
                                action,
                                confirm_key: b'y',
                                default_yes: false,
                                reply: None,
                            });
                            force_render = true;
                        }
                        PrefixOutcome::Prompt { args } => {
                            if let Ok(mut prompt) =
                                CommandPrompt::new(args, None, state, hub, &self.compositor.context)
                            {
                                if !prompt.request.spec.no_freeze {
                                    prompt.presentation.frozen_frame =
                                        Some(self.compositor.last_render.clone());
                                }
                                prompt.initial_incremental(state, hub, &self.compositor.context);
                                self.compositor.command_prompt = Some(prompt);
                            }
                            force_render = true;
                        }
                        PrefixOutcome::Message { text, duration } => {
                            self.compositor.confirm = None;
                            self.compositor.status_message = Some((
                                text,
                                Instant::now()
                                    .checked_add(duration)
                                    .unwrap_or_else(Instant::now),
                            ));
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
                    continue;
                }
                if copy_mode_active(state, target) {
                    let (key, mouse, consumed) = match decode_tty_key(&data[i..]) {
                        Some((mut decoded, consumed)) => {
                            resolve_mouse_key(
                                &mut decoded,
                                &mut self.compositor.mouse_input,
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
                    if is_configured_prefix(state, target, key) {
                        self.compositor.prefix_pending = true;
                        continue;
                    }
                    let copy_table = copy_table_name(state, target);
                    let table = state
                        .lock()
                        .ok()
                        .filter(|st| st.key_binding(copy_table, key).is_none())
                        .and_then(|st| st.key_binding("root", key).map(|_| "root"))
                        .unwrap_or(copy_table);
                    match dispatch_key_binding(
                        table,
                        key,
                        state,
                        target,
                        self.viewport.cols,
                        self.viewport.pane_rows,
                        hub,
                        &self.compositor.context,
                        mouse,
                    ) {
                        PrefixOutcome::Detach => {
                            self.compositor.detach_requested = true;
                            self.compositor.should_exit = true;
                            break;
                        }
                        PrefixOutcome::SendPrefix(bytes) => forward_buf.extend(bytes),
                        PrefixOutcome::CopyMode {
                            page_up,
                            page_down: _,
                            slider: _,
                            mouse: _,
                            begin_selection: _,
                        } => {
                            set_copy_mode_state(state, target, true, page_up);
                            force_render = true;
                        }
                        PrefixOutcome::Confirm { prompt, action } => {
                            self.compositor.confirm = Some(ActiveConfirm {
                                prompt,
                                action,
                                confirm_key: b'y',
                                default_yes: false,
                                reply: None,
                            });
                            force_render = true;
                        }
                        PrefixOutcome::Prompt { args } => {
                            if let Ok(mut prompt) =
                                CommandPrompt::new(args, None, state, hub, &self.compositor.context)
                            {
                                if !prompt.request.spec.no_freeze {
                                    prompt.presentation.frozen_frame =
                                        Some(self.compositor.last_render.clone());
                                }
                                prompt.initial_incremental(state, hub, &self.compositor.context);
                                self.compositor.command_prompt = Some(prompt);
                            }
                            force_render = true;
                        }
                        PrefixOutcome::Message { text, duration } => {
                            self.compositor.confirm = None;
                            self.compositor.status_message = Some((
                                text,
                                Instant::now()
                                    .checked_add(duration)
                                    .unwrap_or_else(Instant::now),
                            ));
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
                    continue;
                }
                // Normal passthrough: forward bytes verbatim (arrow keys, UTF-8,
                // pastes, …), intercepting only the prefix key.
                let start = i;
                let (key, mouse, consumed) = match decode_tty_key(&data[i..]) {
                    Some((mut decoded, consumed)) => {
                        resolve_mouse_key(
                            &mut decoded,
                            &mut self.compositor.mouse_input,
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
                if key.is_some_and(|key| is_configured_prefix(state, target, key)) {
                    // Flush what preceded the prefix, then await the command key.
                    if !forward_buf.is_empty() {
                        first_forward_at.get_or_insert_with(Instant::now);
                        if let Ok(stats) = forward_input(state, target, &forward_buf) {
                            add_input_stats(&mut forwarded, stats);
                        }
                        forward_buf.clear();
                    }
                    self.compositor.prefix_pending = true;
                } else if key.is_some_and(|key| {
                    let table = client_key_table(state, target);
                    state
                        .lock()
                        .ok()
                        .is_some_and(|st| st.key_binding(&table, key).is_some())
                }) {
                    if !forward_buf.is_empty() {
                        first_forward_at.get_or_insert_with(Instant::now);
                        if let Ok(stats) = forward_input(state, target, &forward_buf) {
                            add_input_stats(&mut forwarded, stats);
                        }
                        forward_buf.clear();
                    }
                    let table = client_key_table(state, target);
                    match dispatch_key_binding(
                        &table,
                        key.expect("checked root binding"),
                        state,
                        target,
                        self.viewport.cols,
                        self.viewport.pane_rows,
                        hub,
                        &self.compositor.context,
                        mouse,
                    ) {
                        PrefixOutcome::Detach => {
                            self.compositor.detach_requested = true;
                            self.compositor.should_exit = true;
                            break;
                        }
                        PrefixOutcome::SendPrefix(bytes) => forward_buf.extend(bytes),
                        PrefixOutcome::CopyMode {
                            page_up,
                            page_down,
                            slider,
                            mouse,
                            begin_selection,
                        } => {
                            set_copy_mode_state(state, target, true, page_up);
                            if let Some(mouse) = mouse {
                                if let Ok(mut st) = state.lock() {
                                    let vi = copy_mode_uses_vi_keys(&st, target);
                                    let position = mouse.pane_position();
                                    let _ = st.position_copy_cursor_from_mouse(
                                        target, position.x, position.y, vi,
                                    );
                                    if slider {
                                        let _ = st.set_copy_scroll_from_mouse(
                                            target,
                                            position.y,
                                            self.viewport.pane_rows,
                                            vi,
                                        );
                                    }
                                    if begin_selection {
                                        let separators = st
                                            .option_for_target(target, "word-separators")
                                            .unwrap_or("")
                                            .to_string();
                                        let _ = st.copy_mode_command(
                                            target,
                                            "begin-selection",
                                            vi,
                                            &separators,
                                        );
                                    }
                                }
                            }
                            if page_down {
                                if let Ok(mut st) = state.lock() {
                                    let vi = copy_mode_uses_vi_keys(&st, target);
                                    let separators = st
                                        .option_for_target(target, "word-separators")
                                        .unwrap_or("")
                                        .to_string();
                                    let _ =
                                        st.copy_mode_command(target, "page-down", vi, &separators);
                                }
                            }
                            force_render = true;
                        }
                        PrefixOutcome::Confirm { prompt, action } => {
                            self.compositor.confirm = Some(ActiveConfirm {
                                prompt,
                                action,
                                confirm_key: b'y',
                                default_yes: false,
                                reply: None,
                            });
                            force_render = true;
                        }
                        PrefixOutcome::Prompt { args } => {
                            if let Ok(mut prompt) =
                                CommandPrompt::new(args, None, state, hub, &self.compositor.context)
                            {
                                if !prompt.request.spec.no_freeze {
                                    prompt.presentation.frozen_frame =
                                        Some(self.compositor.last_render.clone());
                                }
                                prompt.initial_incremental(state, hub, &self.compositor.context);
                                self.compositor.command_prompt = Some(prompt);
                            }
                            force_render = true;
                        }
                        PrefixOutcome::Message { text, duration } => {
                            self.compositor.confirm = None;
                            self.compositor.status_message = Some((
                                text,
                                Instant::now()
                                    .checked_add(duration)
                                    .unwrap_or_else(Instant::now),
                            ));
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
                } else if forward_unbound {
                    forward_buf.extend_from_slice(&data[start..i]);
                }
            }
            if self.compositor.should_exit {
                break;
            }
        }
        if self.compositor.terminal_reply_buf.is_empty() {
            self.compositor.terminal_reply_deadline = None;
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
        if self.compositor.should_exit {
            return Ok(self.begin_finish());
        }

        // A prefix command changed the window/pane layout: drop the cached frame
        // and force a full clear so the (possibly smaller) new active pane can't
        // leave the previous pane's cells behind.
        if force_render {
            self.compositor.last_render.clear();
            self.compositor.force_clear = true;
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
                    self.compositor.session_ended = true;
                    return Ok(self.begin_finish());
                }
                Err(error) => return Err(error),
            }
            self.attachments.output_generation = self.attachments.output_generation.wrapping_add(1);
        }

        // 4. Render only when pane output or a layout/input action requests it.
        let should_render = output_ready
            || status_timer_ready
            || agent_status_changed
            || overlay_tick
            || message_expired
            || !render_invalidation.is_empty()
            || self.compositor.last_render.is_empty();

        if should_render {
            let mut wrote_frame = false;
            let mut large_scroll_repaint = false;
            if let Ok(st) = state.lock() {
                let title = terminal_title_update(
                    &st,
                    target,
                    self.viewport.cols,
                    self.viewport.rows,
                    &mut self.status.status_cache,
                    &self.tty.terminal,
                    &mut self.compositor.last_title,
                );
                if !title.is_empty() {
                    let _ = self
                        .tty
                        .output
                        .queue(self.tty.render_fd.as_raw_fd(), &title);
                }
                large_scroll_repaint = take_large_scroll_repaint(
                    &st,
                    target,
                    self.viewport.cols,
                    &self.tty.terminal,
                    &mut self.compositor.seen_large_scroll,
                );
            }
            let frame = self
                .compositor
                .command_prompt
                .as_ref()
                .and_then(|prompt| prompt.presentation.frozen_frame.clone())
                .map(Ok)
                .unwrap_or_else(|| {
                    let st = state.lock();
                    match st {
                        Ok(g) => compose_frame_cached(
                            &g,
                            target,
                            self.viewport.cols,
                            self.viewport.rows,
                            self.viewport.status_height,
                            0,
                            &mut self.status.status_cache,
                            &self.tty.terminal,
                        ),
                        Err(_) => Err(io::Error::other("state poisoned")),
                    }
                });
            if let Ok(mut frame) = frame {
                if let Some(overlay) = self.compositor.active_overlay.as_ref() {
                    if let Ok(st) = state.lock() {
                        frame.extend_from_slice(&render_active_overlay(
                            overlay,
                            &st,
                            target,
                            self.viewport.cols,
                            self.viewport.rows,
                            &self.tty.terminal,
                        ));
                    }
                }
                // Overlay a client prompt on the message line (tmux's last
                // row). It is appended to the frame so the diff below repaints
                // it, and its absence after completion redraws the status bar
                // underneath.
                if let Some(prompt) = self.compositor.command_prompt.as_ref() {
                    if prompt.editor.completion.is_some() {
                        if let Ok(state) = state.lock() {
                            frame.extend_from_slice(&render_prompt_completion(
                                prompt,
                                &state,
                                target,
                                self.viewport.cols,
                                self.viewport.rows,
                                self.viewport.status_height,
                                &self.tty.terminal,
                            ));
                        }
                    }
                    let (display, cursor, row, style, fill) = state
                        .lock()
                        .ok()
                        .map(|st| {
                            let (display, cursor) = prompt.formatted_display(
                                &st,
                                target,
                                usize::from(self.viewport.cols),
                            );
                            let line = st
                                .option_for_target(target, "message-line")
                                .and_then(|value| value.parse::<u16>().ok())
                                .unwrap_or(0)
                                .min(self.viewport.status_height.saturating_sub(1));
                            let row =
                                if st.option_for_target(target, "status-position") == Some("top") {
                                    line + 1
                                } else {
                                    self.viewport
                                        .rows
                                        .saturating_sub(self.viewport.status_height)
                                        .saturating_add(line)
                                        + 1
                                };
                            let (style_option, style_fallback) =
                                if prompt.editor.mode == PromptInputMode::ViCommand {
                                    ("message-command-style", "bg=black,fg=yellow,fill=black")
                                } else {
                                    ("message-style", "bg=yellow,fg=black,fill=yellow")
                                };
                            let style_value = st
                                .option_for_target(target, style_option)
                                .unwrap_or(style_fallback);
                            (
                                display,
                                cursor,
                                row,
                                style_value.to_string(),
                                style_value
                                    .split(',')
                                    .any(|part| part.trim().starts_with("fill=")),
                            )
                        })
                        .unwrap_or_else(|| {
                            (
                                prompt.display(),
                                prompt.display_cursor(),
                                self.viewport.rows,
                                "bg=yellow,fg=black,fill=yellow".to_string(),
                                true,
                            )
                        });
                    let writable_cols = term::writable_width(
                        &self.tty.terminal,
                        row,
                        self.viewport.cols,
                        self.viewport.rows,
                    ) as u16;
                    frame.extend_from_slice(&render_status_prompt_styled_at_row(
                        &display,
                        cursor,
                        self.viewport.cols,
                        writable_cols,
                        row,
                        &style,
                        fill,
                        &self.tty.terminal,
                    ));
                } else if let Some(active) = &self.compositor.confirm {
                    let prompt = &active.prompt;
                    let (row, style, fill) = state
                        .lock()
                        .ok()
                        .map(|st| {
                            let visible_lines = self.viewport.status_height.max(1);
                            let line = st
                                .option_for_target(target, "message-line")
                                .and_then(|value| value.parse::<u16>().ok())
                                .unwrap_or(0)
                                .min(visible_lines.saturating_sub(1));
                            let row = if self.viewport.status_height == 0 {
                                self.viewport.rows
                            } else if status::at_top(&st, target) {
                                line + 1
                            } else {
                                self.viewport
                                    .rows
                                    .saturating_sub(self.viewport.status_height)
                                    .saturating_add(line)
                                    + 1
                            };
                            let value = st
                                .option_for_target(target, "message-style")
                                .unwrap_or("bg=yellow,fg=black,fill=yellow");
                            (
                                row,
                                value.to_string(),
                                value
                                    .split(',')
                                    .any(|part| part.trim().starts_with("fill=")),
                            )
                        })
                        .unwrap_or_else(|| {
                            (
                                self.viewport.rows,
                                "bg=yellow,fg=black,fill=yellow".to_string(),
                                true,
                            )
                        });
                    let writable_cols = term::writable_width(
                        &self.tty.terminal,
                        row,
                        self.viewport.cols,
                        self.viewport.rows,
                    ) as u16;
                    frame.extend_from_slice(&render_status_prompt_styled_at_row(
                        prompt,
                        prompt.chars().count(),
                        self.viewport.cols,
                        writable_cols,
                        row,
                        &style,
                        fill,
                        &self.tty.terminal,
                    ));
                } else if let Some((message, _)) = self.compositor.status_message.as_ref() {
                    let (row, rendered) = state
                        .lock()
                        .ok()
                        .map(|st| {
                            let visible_lines = self.viewport.status_height.max(1);
                            let line = st
                                .option_for_target(target, "message-line")
                                .and_then(|value| value.parse::<u16>().ok())
                                .unwrap_or(0)
                                .min(visible_lines.saturating_sub(1));
                            let row = if self.viewport.status_height == 0 {
                                self.viewport.rows
                            } else if status::at_top(&st, target) {
                                line + 1
                            } else {
                                self.viewport
                                    .rows
                                    .saturating_sub(self.viewport.status_height)
                                    .saturating_add(line)
                                    + 1
                            };
                            let writable = term::writable_width(
                                &self.tty.terminal,
                                row,
                                self.viewport.cols,
                                self.viewport.rows,
                            );
                            (
                                row,
                                self.status.status_cache.message_row(
                                    &st,
                                    target,
                                    message,
                                    self.viewport.cols,
                                    self.viewport.rows,
                                    writable,
                                    &self.tty.terminal,
                                ),
                            )
                        })
                        .unwrap_or_else(|| (self.viewport.rows, Vec::new()));
                    frame.extend_from_slice(&render_status_message_row_at(
                        row,
                        &rendered,
                        &self.tty.terminal,
                    ));
                }
                if frame != self.compositor.last_render || large_scroll_repaint {
                    let (mut repaint, direct_cursor_safe) =
                        if self.compositor.last_render.is_empty() || large_scroll_repaint {
                            (frame.clone(), false)
                        } else {
                            let delta = diff_rendered_frame(&self.compositor.last_render, &frame);
                            let direct_cursor_safe = delta.direct_cursor_safe();
                            (delta.into_frame(), direct_cursor_safe)
                        };
                    // A first paint, resize, or layout change still needs one
                    // full clear. Keep that sequence out of the cached canonical frame
                    // so subsequent frames compare canonical compositor output.
                    if self.compositor.force_clear {
                        let mut cleared = Vec::with_capacity(repaint.len() + 8);
                        cleared.extend_from_slice(b"\x1b[H\x1b[2J");
                        cleared.extend_from_slice(&repaint);
                        repaint = cleared;
                    }

                    // Commit multi-row repaint deltas atomically when possible.
                    // Cursor-only changes and a bounded update ending on the
                    // cursor row can be sent directly: they never march the
                    // hardware cursor across unrelated rows. Larger
                    // unsynchronized deltas get an
                    // immediate hide/restore pair around only the dirty rows.
                    let sync_start = term::expand_capability(
                        &self.tty.terminal,
                        "Sync",
                        &[term::CapabilityParameter::Number(1)],
                    );
                    let sync_end = term::expand_capability(
                        &self.tty.terminal,
                        "Sync",
                        &[term::CapabilityParameter::Number(2)],
                    );
                    if let (Some(sync_start), Some(sync_end)) = (sync_start, sync_end) {
                        let output = suppress_redundant_cursor_visibility(
                            &repaint,
                            &mut self.compositor.output_cursor_visible,
                        );
                        let mut atomic_output =
                            Vec::with_capacity(sync_start.len() + output.len() + sync_end.len());
                        atomic_output.extend_from_slice(&sync_start);
                        atomic_output.extend_from_slice(&output);
                        atomic_output.extend_from_slice(&sync_end);
                        let _ = self
                            .tty
                            .output
                            .queue(self.tty.render_fd.as_raw_fd(), &atomic_output);
                    } else if direct_cursor_safe && !self.compositor.force_clear {
                        let output = suppress_redundant_cursor_visibility(
                            &repaint,
                            &mut self.compositor.output_cursor_visible,
                        );
                        let _ = self
                            .tty
                            .output
                            .queue(self.tty.render_fd.as_raw_fd(), &output);
                    } else {
                        let output = guard_cursor_during_repaint(
                            &repaint,
                            &mut self.compositor.output_cursor_visible,
                        );
                        let _ = self
                            .tty
                            .output
                            .queue(self.tty.render_fd.as_raw_fd(), &output);
                    }
                    self.compositor.last_render = frame;
                    self.compositor.force_clear = false;
                    wrote_frame = true;
                }
            }
            // Close the latency sample: a written frame is the keystroke's echo
            // reaching the screen; an unchanged frame means this input drew
            // nothing, so drop it rather than blame a later frame.
            if wrote_frame {
                self.pane_io.latmon.on_render();
            } else {
                self.pane_io.latmon.discard();
            }
        }
        Ok(AttachDrive::Continue)
    }
}
