use super::*;

pub(crate) struct ActiveOverlay {
    pub(super) state: OverlayState,
    reply: Option<std::sync::mpsc::Sender<crate::server::state::PromptCompletion>>,
}

pub(super) enum OverlayState {
    Menu(MenuOverlay),
    Popup(PopupOverlay),
    DisplayPanes(DisplayPanesOverlay),
}

pub(super) struct MenuOverlay {
    pub(super) request: MenuRequest,
    pub(super) selected: usize,
}

pub(super) struct PopupOverlay {
    pub(super) request: Box<PopupRequest>,
    pub(super) pane: Box<Pane>,
    pub(super) io: Option<Box<PaneIo>>,
    pub(super) read_continuation: bool,
    pub(super) exit_status: Option<i32>,
}

pub(super) struct DisplayPanesOverlay {
    pub(super) deadline: Instant,
    pub(super) command: Vec<String>,
    pub(super) accept_input: bool,
}

impl ActiveOverlay {
    pub(super) fn menu(request: MenuRequest, selected: usize) -> Self {
        Self::menu_with_reply(request, selected, None)
    }

    fn menu_with_reply(
        request: MenuRequest,
        selected: usize,
        reply: Option<std::sync::mpsc::Sender<crate::server::state::PromptCompletion>>,
    ) -> Self {
        let selected = selected.min(request.items.len().saturating_sub(1));
        Self {
            state: OverlayState::Menu(MenuOverlay { request, selected }),
            reply,
        }
    }

    pub(super) fn from_request(
        request: OverlayRequest,
        reply: Option<std::sync::mpsc::Sender<crate::server::state::PromptCompletion>>,
        cols: u16,
        rows: u16,
        pane_io_mode: PaneIoMode,
    ) -> io::Result<Option<Self>> {
        Ok(match request {
            OverlayRequest::Clear => None,
            OverlayRequest::Menu(request) => {
                let selected = request.selected;
                Some(Self::menu_with_reply(request, selected, reply))
            }
            OverlayRequest::DisplayPanes {
                duration_ms,
                command,
                accept_input,
            } => Some(Self {
                state: OverlayState::DisplayPanes(DisplayPanesOverlay {
                    deadline: Instant::now()
                        .checked_add(Duration::from_millis(duration_ms))
                        .unwrap_or_else(Instant::now),
                    command,
                    accept_input,
                }),
                reply,
            }),
            OverlayRequest::Popup(request) => {
                let outer_width =
                    overlay_dimension(request.width.as_deref(), cols, 50).clamp(3, cols.max(3));
                let outer_height =
                    overlay_dimension(request.height.as_deref(), rows, 50).clamp(3, rows.max(3));
                let inner_width = outer_width.saturating_sub(if request.border { 2 } else { 0 });
                let inner_height = outer_height.saturating_sub(if request.border { 2 } else { 0 });
                let argv = if request.argv.is_empty() {
                    vec![std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())]
                } else if request.argv.len() == 1 {
                    vec![
                        "/bin/sh".to_string(),
                        "-c".to_string(),
                        request.argv[0].clone(),
                    ]
                } else {
                    request.argv.clone()
                };
                let refs = argv.iter().map(String::as_str).collect::<Vec<_>>();
                let mut pane = Pane::spawn_in_mode(
                    &refs,
                    request.cwd.as_deref(),
                    inner_width.max(1),
                    inner_height.max(1),
                    pane_io_mode,
                )?;
                let io = pane.take_event_io().map(Box::new);
                Some(Self {
                    state: OverlayState::Popup(PopupOverlay {
                        request: Box::new(request),
                        pane: Box::new(pane),
                        io,
                        read_continuation: false,
                        exit_status: None,
                    }),
                    reply,
                })
            }
        })
    }

    pub(super) fn complete(&mut self, result: command::CommandResult, inserted: bool) {
        if let Some(reply) = self.reply.take() {
            let _ = reply.send(crate::server::state::PromptCompletion {
                stdout: result.stdout,
                stderr: result.stderr,
                exit: result.exit,
                inserted,
            });
        }
    }

    pub(super) fn poll_timeout(&self, now: Instant) -> i32 {
        match &self.state {
            OverlayState::DisplayPanes(display) => {
                deadline_poll_timeout(Some(display.deadline), now)
            }
            OverlayState::Popup(_) => 50,
            OverlayState::Menu(_) => -1,
        }
    }

    pub(super) fn resize(&mut self, cols: u16, rows: u16) {
        if let OverlayState::Popup(popup) = &mut self.state {
            let width = overlay_dimension(popup.request.width.as_deref(), cols, 50)
                .max(3)
                .min(cols.max(3));
            let height = overlay_dimension(popup.request.height.as_deref(), rows, 50)
                .max(3)
                .min(rows.max(3));
            let inset = u16::from(popup.request.border) * 2;
            let _ = popup.pane.resize(
                width.saturating_sub(inset).max(1),
                height.saturating_sub(inset).max(1),
            );
        }
    }

    pub(super) fn sources(&self) -> (RawFd, RawFd) {
        let OverlayState::Popup(PopupOverlay { io: Some(io), .. }) = &self.state else {
            return (-1, -1);
        };
        let fd = io.as_fd().as_raw_fd();
        (fd, if io.wants_write() { fd } else { -1 })
    }

    pub(super) fn read_continuation(&self) -> bool {
        matches!(
            &self.state,
            OverlayState::Popup(PopupOverlay {
                read_continuation: true,
                ..
            })
        )
    }

    pub(super) fn drive_io(&mut self, readable: bool, writable: bool) -> io::Result<()> {
        let OverlayState::Popup(PopupOverlay {
            io: Some(io),
            read_continuation,
            ..
        }) = &mut self.state
        else {
            return Ok(());
        };
        if writable {
            io.drive_writable();
        }
        if readable {
            *read_continuation = io.drive_readable()?.continuation;
        }
        Ok(())
    }
}

fn overlay_dimension(value: Option<&str>, available: u16, default_percent: u16) -> u16 {
    match value {
        Some(value) if value.ends_with('%') => value
            .trim_end_matches('%')
            .parse::<u32>()
            .ok()
            .map(|percent| (u32::from(available) * percent / 100) as u16)
            .unwrap_or(available),
        Some(value) => value.parse().unwrap_or(available),
        None => (u32::from(available) * u32::from(default_percent) / 100) as u16,
    }
}

fn overlay_position(value: Option<&str>, available: u16, size: u16) -> u16 {
    match value {
        Some("C" | "M" | "P" | "W" | "S") | None => available.saturating_sub(size) / 2,
        Some(value) if value.ends_with('%') => value
            .trim_end_matches('%')
            .parse::<u32>()
            .ok()
            .map(|percent| (u32::from(available.saturating_sub(size)) * percent / 100) as u16)
            .unwrap_or(0),
        Some(value) => value
            .parse::<u16>()
            .unwrap_or(0)
            .min(available.saturating_sub(size)),
    }
}

fn draw_overlay_box(out: &mut Vec<u8>, top: u16, left: u16, width: u16, height: u16, title: &str) {
    if width < 2 || height < 2 {
        return;
    }
    let inner = width.saturating_sub(2) as usize;
    let mut top_line = format!("┌{}┐", "─".repeat(inner));
    if !title.is_empty() && inner > 2 {
        let shown = clip_mode_line(title, inner.saturating_sub(2));
        let replacement = format!(" {} ", shown);
        let mut chars = top_line.chars().collect::<Vec<_>>();
        for (index, character) in replacement.chars().enumerate() {
            if index + 1 < chars.len().saturating_sub(1) {
                chars[index + 1] = character;
            }
        }
        top_line = chars.into_iter().collect();
    }
    out.extend_from_slice(format!("\x1b[{};{}H{}", top + 1, left + 1, top_line).as_bytes());
    for row in 1..height.saturating_sub(1) {
        out.extend_from_slice(
            format!(
                "\x1b[{};{}H│\x1b[{};{}H│",
                top + row + 1,
                left + 1,
                top + row + 1,
                left + width
            )
            .as_bytes(),
        );
    }
    out.extend_from_slice(
        format!("\x1b[{};{}H└{}┘", top + height, left + 1, "─".repeat(inner)).as_bytes(),
    );
}

pub(super) fn render_active_overlay(
    overlay: &ActiveOverlay,
    st: &ServerState,
    target: &str,
    cols: u16,
    rows: u16,
    terminal: &dyn TerminalCapabilities,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"\x1b[?25l");
    append_terminal_style_reset(&mut out, terminal);
    match &overlay.state {
        OverlayState::Menu(MenuOverlay {
            request, selected, ..
        }) => {
            let content_width = request
                .items
                .iter()
                .map(|item| {
                    format::display_width(&item.label)
                        + if item.key.is_empty() {
                            0
                        } else {
                            format::display_width(&item.key) + 3
                        }
                })
                .max()
                .unwrap_or(1)
                .max(format::display_width(&request.title));
            let width = (content_width + 4).min(cols as usize).max(3) as u16;
            let height = (request.items.len() + 2).min(rows as usize).max(3) as u16;
            let left = overlay_position(request.x.as_deref(), cols, width);
            let top = overlay_position(request.y.as_deref(), rows, height);
            draw_overlay_box(&mut out, top, left, width, height, &request.title);
            for (index, item) in request
                .items
                .iter()
                .take(height.saturating_sub(2) as usize)
                .enumerate()
            {
                if item.label.is_empty() {
                    out.extend_from_slice(
                        format!(
                            "\x1b[{};{}H{}",
                            top + index as u16 + 2,
                            left + 2,
                            "─".repeat(width.saturating_sub(2) as usize)
                        )
                        .as_bytes(),
                    );
                    continue;
                }
                let key = if item.key.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", item.key)
                };
                let text = clip_mode_line(
                    &format!("{}{}", item.label, key),
                    width.saturating_sub(4) as usize,
                );
                out.extend_from_slice(
                    format!("\x1b[{};{}H", top + index as u16 + 2, left + 3).as_bytes(),
                );
                if index == *selected {
                    out.extend_from_slice(b"\x1b[7m");
                }
                out.extend_from_slice(text.as_bytes());
                if index == *selected {
                    append_terminal_style_reset(&mut out, terminal);
                }
            }
        }
        OverlayState::Popup(PopupOverlay { request, pane, .. }) => {
            let width = overlay_dimension(request.width.as_deref(), cols, 50)
                .max(3)
                .min(cols.max(3));
            let height = overlay_dimension(request.height.as_deref(), rows, 50)
                .max(3)
                .min(rows.max(3));
            let left = overlay_position(request.x.as_deref(), cols, width);
            let top = overlay_position(request.y.as_deref(), rows, height);
            let inset = u16::from(request.border);
            if request.border {
                draw_overlay_box(&mut out, top, left, width, height, &request.title);
            }
            if let Ok(vt) = pane.dump_vt() {
                let (popup_rows, cursor) = split_pane_vt(&vt);
                let visible_height = height.saturating_sub(inset * 2);
                let visible_width = width.saturating_sub(inset * 2);
                for (row, content) in popup_rows
                    .iter()
                    .rev()
                    .take(visible_height as usize)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .enumerate()
                {
                    out.extend_from_slice(
                        format!(
                            "\x1b[{};{}H\x1b[{}X\x1b[{};{}H",
                            top + inset + row as u16 + 1,
                            left + inset + 1,
                            visible_width,
                            top + inset + row as u16 + 1,
                            left + inset + 1
                        )
                        .as_bytes(),
                    );
                    out.extend_from_slice(content);
                }
                if !pane.has_exited() {
                    if let Some((cursor_row, cursor_col)) = parse_cup(cursor) {
                        out.extend_from_slice(
                            format!(
                                "\x1b[{};{}H\x1b[?25h",
                                top + inset + cursor_row,
                                left + inset + cursor_col
                            )
                            .as_bytes(),
                        );
                    }
                }
            }
        }
        OverlayState::DisplayPanes(_) => {
            if let Ok((window, _)) = st.active_window_panes(target) {
                for (index, pane) in window.panes.iter().enumerate() {
                    let rect = window.pane_rect(pane.id).unwrap_or_default();
                    let label = index.to_string();
                    let row = rect.top + rect.height / 2 + 1;
                    let col = rect.left + rect.width.saturating_sub(label.len() as u16) / 2 + 1;
                    out.extend_from_slice(
                        format!("\x1b[{row};{col}H\x1b[30;43m{label}").as_bytes(),
                    );
                    append_terminal_style_reset(&mut out, terminal);
                }
            }
        }
    }
    out
}
