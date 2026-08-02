use super::*;

pub(crate) struct ActiveOverlay {
    state: OverlayState,
    reply: Option<std::sync::mpsc::Sender<super::super::state::PromptCompletion>>,
}

enum OverlayState {
    Menu(MenuOverlay),
    Popup(PopupOverlay),
    DisplayPanes(DisplayPanesOverlay),
}

struct MenuOverlay {
    request: MenuRequest,
    selected: usize,
}

struct PopupOverlay {
    request: Box<PopupRequest>,
    pane: Box<Pane>,
    io: Option<Box<PaneIo>>,
    read_continuation: bool,
    exit_status: Option<i32>,
}

struct DisplayPanesOverlay {
    deadline: Instant,
    command: Vec<String>,
    accept_input: bool,
}

pub(super) struct OverlayInputOutcome {
    pub(super) close: bool,
    pub(super) exit: i32,
    pub(super) command: Option<Vec<String>>,
}

impl OverlayInputOutcome {
    fn stay() -> Self {
        Self {
            close: false,
            exit: 0,
            command: None,
        }
    }

    fn close(exit: i32, command: Option<Vec<String>>) -> Self {
        Self {
            close: true,
            exit,
            command,
        }
    }
}

impl ActiveOverlay {
    pub(super) fn menu(request: MenuRequest, selected: usize) -> Self {
        Self::menu_with_reply(request, selected, None)
    }

    fn menu_with_reply(
        request: MenuRequest,
        selected: usize,
        reply: Option<std::sync::mpsc::Sender<super::super::state::PromptCompletion>>,
    ) -> Self {
        let selected = selected.min(request.items.len().saturating_sub(1));
        Self {
            state: OverlayState::Menu(MenuOverlay { request, selected }),
            reply,
        }
    }

    pub(super) fn from_request(
        request: OverlayRequest,
        reply: Option<std::sync::mpsc::Sender<super::super::state::PromptCompletion>>,
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
            let _ = reply.send(super::super::state::PromptCompletion {
                stdout: result.stdout,
                stderr: result.stderr,
                exit: result.exit,
                inserted,
            });
        }
    }

    pub(super) fn poll_timeout(&self, now: Instant) -> i32 {
        match &self.state {
            OverlayState::DisplayPanes(overlay) => {
                deadline_poll_timeout(Some(overlay.deadline), now)
            }
            OverlayState::Popup(_) => 50,
            OverlayState::Menu(_) => -1,
        }
    }

    pub(super) fn tick(&mut self, now: Instant) -> Option<i32> {
        match &mut self.state {
            OverlayState::DisplayPanes(overlay) => (overlay.deadline <= now).then_some(0),
            OverlayState::Popup(overlay) => overlay.tick(),
            OverlayState::Menu(_) => None,
        }
    }

    pub(super) fn resize(&mut self, cols: u16, rows: u16) {
        if let OverlayState::Popup(overlay) = &mut self.state {
            overlay.resize(cols, rows);
        }
    }

    pub(super) fn popup_sources(&self) -> (RawFd, RawFd) {
        match &self.state {
            OverlayState::Popup(overlay) => overlay.sources(),
            _ => (-1, -1),
        }
    }

    pub(super) fn popup_read_continuation(&self) -> bool {
        matches!(
            &self.state,
            OverlayState::Popup(PopupOverlay {
                read_continuation: true,
                ..
            })
        )
    }

    pub(super) fn drive_popup_io(&mut self, readable: bool, writable: bool) -> io::Result<()> {
        match &mut self.state {
            OverlayState::Popup(overlay) => overlay.drive_io(readable, writable),
            _ => Ok(()),
        }
    }

    pub(super) fn handle_key(
        &mut self,
        key: &str,
        raw: &[u8],
        state: &Arc<Mutex<ServerState>>,
        target: &str,
    ) -> OverlayInputOutcome {
        match &mut self.state {
            OverlayState::Menu(overlay) => overlay.handle_key(key),
            OverlayState::Popup(overlay) => overlay.handle_key(key, raw),
            OverlayState::DisplayPanes(overlay) => overlay.handle_key(key, state, target),
        }
    }

    pub(super) fn render(
        &self,
        state: &ServerState,
        target: &str,
        cols: u16,
        rows: u16,
        terminal: &dyn TerminalCapabilities,
    ) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"\x1b[?25l");
        append_terminal_style_reset(&mut out, terminal);
        match &self.state {
            OverlayState::Menu(overlay) => overlay.render(&mut out, cols, rows, terminal),
            OverlayState::Popup(overlay) => overlay.render(&mut out, cols, rows),
            OverlayState::DisplayPanes(overlay) => {
                overlay.render(&mut out, state, target, terminal)
            }
        }
        out
    }
}

impl MenuOverlay {
    fn handle_key(&mut self, key: &str) -> OverlayInputOutcome {
        match key {
            "q" | "Escape" | "C-c" => OverlayInputOutcome::close(0, None),
            "Up" | "k" => {
                self.selected = self.selected.saturating_sub(1);
                OverlayInputOutcome::stay()
            }
            "Down" | "j" => {
                self.selected = (self.selected + 1).min(self.request.items.len().saturating_sub(1));
                OverlayInputOutcome::stay()
            }
            "Enter" => OverlayInputOutcome::close(
                0,
                self.request
                    .items
                    .get(self.selected)
                    .map(|item| item.command.clone()),
            ),
            key => self
                .request
                .items
                .iter()
                .find(|item| item.key == key)
                .map(|item| OverlayInputOutcome::close(0, Some(item.command.clone())))
                .unwrap_or_else(OverlayInputOutcome::stay),
        }
    }

    fn render(&self, out: &mut Vec<u8>, cols: u16, rows: u16, terminal: &dyn TerminalCapabilities) {
        let content_width = self
            .request
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
            .max(format::display_width(&self.request.title));
        let width = (content_width + 4).min(cols as usize).max(3) as u16;
        let height = (self.request.items.len() + 2).min(rows as usize).max(3) as u16;
        let left = overlay_position(self.request.x.as_deref(), cols, width);
        let top = overlay_position(self.request.y.as_deref(), rows, height);
        draw_overlay_box(out, top, left, width, height, &self.request.title);
        for (index, item) in self
            .request
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
            if index == self.selected {
                out.extend_from_slice(b"\x1b[7m");
            }
            out.extend_from_slice(text.as_bytes());
            if index == self.selected {
                append_terminal_style_reset(out, terminal);
            }
        }
    }
}

impl PopupOverlay {
    fn tick(&mut self) -> Option<i32> {
        if !self.pane.has_exited() {
            return None;
        }
        if self.exit_status.is_none() {
            self.exit_status = self.pane.try_wait();
        }
        self.exit_status.filter(|exit| {
            self.request.close_on_exit || (self.request.close_on_success && *exit == 0)
        })
    }

    fn resize(&mut self, cols: u16, rows: u16) {
        let width = overlay_dimension(self.request.width.as_deref(), cols, 50)
            .max(3)
            .min(cols.max(3));
        let height = overlay_dimension(self.request.height.as_deref(), rows, 50)
            .max(3)
            .min(rows.max(3));
        let inset = u16::from(self.request.border) * 2;
        let _ = self.pane.resize(
            width.saturating_sub(inset).max(1),
            height.saturating_sub(inset).max(1),
        );
    }

    fn sources(&self) -> (RawFd, RawFd) {
        let Some(io) = self.io.as_ref() else {
            return (-1, -1);
        };
        let fd = io.as_fd().as_raw_fd();
        (fd, if io.wants_write() { fd } else { -1 })
    }

    fn drive_io(&mut self, readable: bool, writable: bool) -> io::Result<()> {
        let Some(io) = self.io.as_mut() else {
            return Ok(());
        };
        if writable {
            io.drive_writable();
        }
        if readable {
            self.read_continuation = io.drive_readable()?.continuation;
        }
        Ok(())
    }

    fn handle_key(&mut self, key: &str, raw: &[u8]) -> OverlayInputOutcome {
        if self.exit_status.is_some()
            || self.request.close_on_key
            || (matches!(key, "Escape" | "C-c")
                && !self.request.close_on_exit
                && !self.request.close_on_success)
        {
            return OverlayInputOutcome::close(self.exit_status.unwrap_or(129), None);
        }
        let _ = self.pane.input(raw);
        OverlayInputOutcome::stay()
    }

    fn render(&self, out: &mut Vec<u8>, cols: u16, rows: u16) {
        let width = overlay_dimension(self.request.width.as_deref(), cols, 50)
            .max(3)
            .min(cols.max(3));
        let height = overlay_dimension(self.request.height.as_deref(), rows, 50)
            .max(3)
            .min(rows.max(3));
        let left = overlay_position(self.request.x.as_deref(), cols, width);
        let top = overlay_position(self.request.y.as_deref(), rows, height);
        let inset = u16::from(self.request.border);
        if self.request.border {
            draw_overlay_box(out, top, left, width, height, &self.request.title);
        }
        if let Ok(vt) = self.pane.dump_vt() {
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
            if !self.pane.has_exited() {
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
}

impl DisplayPanesOverlay {
    fn handle_key(
        &self,
        key: &str,
        state: &Arc<Mutex<ServerState>>,
        target: &str,
    ) -> OverlayInputOutcome {
        if !self.accept_input || matches!(key, "Escape" | "q" | "C-c") {
            return OverlayInputOutcome::close(0, None);
        }
        let Some(index) = key
            .chars()
            .next()
            .filter(|_| key.chars().count() == 1)
            .and_then(|value| value.to_digit(10))
        else {
            return OverlayInputOutcome::stay();
        };
        let pane_id = state.lock().ok().and_then(|st| {
            st.active_window_panes(target)
                .ok()
                .and_then(|(window, _)| window.panes.get(index as usize))
                .map(|pane| pane.id)
        });
        let Some(pane_id) = pane_id else {
            return OverlayInputOutcome::stay();
        };
        let command = if self.command.is_empty() {
            vec![
                "select-pane".to_string(),
                "-t".to_string(),
                format!("%{pane_id}"),
            ]
        } else {
            self.command
                .iter()
                .map(|word| word.replace("%%", &format!("%{pane_id}")))
                .collect()
        };
        OverlayInputOutcome::close(0, Some(command))
    }

    fn render(
        &self,
        out: &mut Vec<u8>,
        state: &ServerState,
        target: &str,
        terminal: &dyn TerminalCapabilities,
    ) {
        if let Ok((window, _)) = state.active_window_panes(target) {
            for (index, pane) in window.panes.iter().enumerate() {
                let rect = window.pane_rect(pane.id).unwrap_or_default();
                let label = index.to_string();
                let row = rect.top + rect.height / 2 + 1;
                let col = rect.left + rect.width.saturating_sub(label.len() as u16) / 2 + 1;
                out.extend_from_slice(format!("\x1b[{row};{col}H\x1b[30;43m{label}").as_bytes());
                append_terminal_style_reset(out, terminal);
            }
        }
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
