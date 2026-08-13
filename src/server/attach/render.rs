use super::*;
use crate::server::state::SharedState;

impl AttachSession {
    pub(super) fn render_turn(
        &mut self,
        state: &SharedState,
        triggers: AttachRenderTriggers,
    ) -> io::Result<()> {
        let AttachRenderTriggers {
            output_ready,
            status_timer_ready,
            agent_status_changed,
            overlay_tick,
            message_expired,
            render_invalidation,
        } = triggers;
        let target = self.compositor.target.stable_target.as_str();
        // 4. Render only when pane output or a layout/input action requests it.
        let should_render = output_ready
            || status_timer_ready
            || agent_status_changed
            || overlay_tick
            || message_expired
            || !render_invalidation.is_empty()
            || self.compositor.render.last_render.is_empty();

        if should_render {
            let mut wrote_frame = false;
            let large_scroll_repaint = {
                let st = state.borrow_mut();
                let title = terminal_title_update(
                    &st,
                    target,
                    self.viewport.cols,
                    self.viewport.rows,
                    &mut self.status.status_cache,
                    &self.tty.terminal,
                    &mut self.compositor.render.last_title,
                );
                if !title.is_empty() {
                    let _ = self
                        .tty
                        .output
                        .queue(self.tty.render_fd.as_raw_fd(), &title);
                }
                take_large_scroll_repaint(
                    &st,
                    target,
                    self.viewport.cols,
                    &self.tty.terminal,
                    &mut self.compositor.render.seen_large_scroll,
                )
            };
            let frame = self
                .compositor
                .ui
                .command_prompt
                .as_ref()
                .and_then(|prompt| prompt.frozen_frame().map(<[u8]>::to_vec))
                .map(Ok)
                .unwrap_or_else(|| {
                    let client = self.attachments.render_attachment.client_name();
                    let st = state.borrow_mut();
                    compose_frame_cached(
                        &st,
                        target,
                        Some(client.as_str()),
                        self.viewport.cols,
                        self.viewport.rows,
                        self.viewport.status_height,
                        0,
                        &mut self.status.status_cache,
                        &self.tty.terminal,
                    )
                });
            if let Ok(mut frame) = frame {
                if let Some(overlay) = self.compositor.ui.active_overlay.as_ref() {
                    {
                        let st = state.borrow_mut();
                        frame.extend_from_slice(&overlay.render(
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
                if let Some(prompt) = self.compositor.ui.command_prompt.as_ref() {
                    if prompt.has_completion() {
                        {
                            let state = state.borrow_mut();
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
                    let (display, cursor, row, style, fill) = {
                        let st = state.borrow_mut();
                        let (display, cursor) =
                            prompt.formatted_display(&st, target, usize::from(self.viewport.cols));
                        let line = st
                            .option_for_target(target, "message-line")
                            .and_then(|value| value.parse::<u16>().ok())
                            .unwrap_or(0)
                            .min(self.viewport.status_height.saturating_sub(1));
                        let row = if st.option_for_target(target, "status-position") == Some("top")
                        {
                            line + 1
                        } else {
                            self.viewport
                                .rows
                                .saturating_sub(self.viewport.status_height)
                                .saturating_add(line)
                                + 1
                        };
                        let (style_option, style_fallback) = if prompt.is_vi_command() {
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
                    };
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
                    // tmux's `status_prompt_redraw` puts the prompt's own
                    // cursor colour and shape on the client's terminal, with a
                    // separate shape for the vi command mode.
                    {
                        let st = state.borrow_mut();
                        frame.extend_from_slice(&prompt_cursor_sequence(
                            &st,
                            target,
                            prompt.is_vi_command(),
                        ));
                    }
                } else if let Some(active) = &self.compositor.ui.confirm {
                    let prompt = &active.prompt;
                    let (row, style, fill) = {
                        let st = state.borrow_mut();
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
                    };
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
                } else if let Some(message) = self.compositor.ui.status_message.as_ref() {
                    let (row, rendered) = {
                        let st = state.borrow_mut();
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
                                &message.text,
                                self.viewport.cols,
                                self.viewport.rows,
                                writable,
                                &self.tty.terminal,
                            ),
                        )
                    };
                    // The message owns its row: drop the composed status
                    // content beneath it so covered state never reaches the
                    // wire while the message is alive. With `status off` the
                    // row is a pane row that may carry the cursor tail, so
                    // the message keeps painting over it as before.
                    if self.viewport.status_height > 0 {
                        frame = frame_without_row(&frame, row);
                    }
                    frame.extend_from_slice(&render_status_message_row_at(
                        row,
                        &rendered,
                        &self.tty.terminal,
                    ));
                }
                if frame != self.compositor.render.last_render || large_scroll_repaint {
                    let (mut repaint, direct_cursor_safe) =
                        if self.compositor.render.last_render.is_empty() || large_scroll_repaint {
                            (frame.clone(), false)
                        } else {
                            let delta =
                                diff_rendered_frame(&self.compositor.render.last_render, &frame);
                            let direct_cursor_safe = delta.direct_cursor_safe();
                            (delta.into_frame(), direct_cursor_safe)
                        };
                    // A first paint, resize, or layout change still needs one
                    // full clear. Keep that sequence out of the cached canonical frame
                    // so subsequent frames compare canonical compositor output.
                    if self.compositor.render.force_clear {
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
                        Capability::Sync,
                        &[term::CapabilityParameter::Number(1)],
                    );
                    let sync_end = term::expand_capability(
                        &self.tty.terminal,
                        Capability::Sync,
                        &[term::CapabilityParameter::Number(2)],
                    );
                    if let (Some(sync_start), Some(sync_end)) = (sync_start, sync_end) {
                        let output = suppress_redundant_cursor_visibility(
                            &repaint,
                            &mut self.compositor.render.output_cursor_visible,
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
                    } else if direct_cursor_safe && !self.compositor.render.force_clear {
                        let output = suppress_redundant_cursor_visibility(
                            &repaint,
                            &mut self.compositor.render.output_cursor_visible,
                        );
                        let _ = self
                            .tty
                            .output
                            .queue(self.tty.render_fd.as_raw_fd(), &output);
                    } else {
                        let output = guard_cursor_during_repaint(
                            &repaint,
                            &mut self.compositor.render.output_cursor_visible,
                        );
                        let _ = self
                            .tty
                            .output
                            .queue(self.tty.render_fd.as_raw_fd(), &output);
                    }
                    self.compositor.render.last_render = frame;
                    self.compositor.render.force_clear = false;
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
        Ok(())
    }

    /// Put the client's terminal into the mouse mode the session now wants.
    ///
    /// This runs on every pass of the attach loop, as tmux's
    /// `server_client_reset_state` does, because the wanted mode follows the
    /// active pane's DECSETs: nothing else notices when the program in the
    /// pane starts or stops tracking the mouse. The cached word keeps the
    /// steady state to an integer compare — bytes go out only on a transition,
    /// which is what `tty_update_mode` diffs `tty->mode` for.
    pub(super) fn sync_tty_mouse_mode(&mut self, state: &SharedState) {
        // tmux gates the whole mouse path on the terminal describing a mouse
        // key, and hmux's start and stop sequences already do.
        if self.tty.terminal.capability(Capability::kmous).is_none() {
            return;
        }
        let overlay = self.compositor.ui.active_overlay.is_some()
            || self.compositor.ui.command_prompt.is_some();
        let wanted = state
            .borrow_mut()
            .client_tty_mouse_mode(self.compositor.target.stable_target.as_str(), overlay);
        if self.compositor.render.tty_mouse_mode == wanted {
            return;
        }
        let _ = self
            .tty
            .output
            .queue(self.tty.render_fd.as_raw_fd(), wanted.apply_sequence());
        self.compositor.render.tty_mouse_mode = wanted;
    }
}

/// The cursor a command prompt puts on the client's terminal: the colour from
/// `prompt-cursor-colour` and the shape from `prompt-cursor-style` — or
/// `prompt-command-cursor-style` while the prompt is in vi command mode.
///
/// tmux writes these as an ordinary OSC 12 and DECSCUSR, which is what returns
/// the terminal to its own cursor when the prompt goes away and the frame that
/// carried them is redrawn without them.
fn prompt_cursor_sequence(st: &ServerState, target: &str, vi_command: bool) -> Vec<u8> {
    let mut out = Vec::new();
    if let Some(colour) = st
        .option_for_target(target, "prompt-cursor-colour")
        .filter(|value| !value.is_empty() && *value != "default")
    {
        out.extend_from_slice(format!("\x1b]12;{colour}\x07").as_bytes());
    }
    let option = if vi_command {
        "prompt-command-cursor-style"
    } else {
        "prompt-cursor-style"
    };
    let style = st.option_for_target(target, option).unwrap_or("default");
    // tmux's `screen_set_cursor_style` numbering, which DECSCUSR takes.
    let parameter = match style {
        "blinking-block" => 1,
        "block" => 2,
        "blinking-underline" => 3,
        "underline" => 4,
        "blinking-bar" => 5,
        "bar" => 6,
        _ => 0,
    };
    out.extend_from_slice(format!("\x1b[{parameter} q").as_bytes());
    out
}
