use super::*;
use crate::server::state::SharedState;

pub(crate) struct ActiveOverlay {
    state: OverlayState,
    reply: Option<super::super::state::PromptReply>,
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
    /// Where a drag has put the popup, once one has: tmux keeps the dragged
    /// position and size on the popup rather than recomputing them from the
    /// command's own `-x`/`-y`/`-w`/`-h`.
    placement: Option<PopupPlacement>,
    dragging: Option<PopupDrag>,
    /// The pointer's previous position, which the overlay path has to keep for
    /// itself — tmux's `pd->lx`/`pd->ly`. A drag takes hold of the border the
    /// button went down on, not the cell the pointer has already reached.
    last_pointer: Option<(u16, u16)>,
    /// The popup's own menu, opened by a right-click on its border. It is
    /// drawn over the popup and takes its keys, as tmux's `pd->md` does.
    menu: Option<MenuOverlay>,
    /// tmux's `pd->close`, set by a menu item that ends the popup rather than
    /// changing it — the pane conversions, which leave nothing behind to draw.
    closing: bool,
}

/// The item list tmux's `popup_menu_items` offers, keyed as it keys them. The
/// commands are markers the popup itself reads; they never reach the command
/// layer.
fn popup_menu_items() -> Vec<super::super::state::MenuItem> {
    [
        ("Close", "q"),
        ("", ""),
        ("Fill Space", "F"),
        ("Centre", "C"),
        ("", ""),
        ("To Horizontal Pane", "h"),
        ("To Vertical Pane", "v"),
    ]
    .into_iter()
    .map(|(label, key)| super::super::state::MenuItem {
        label: label.to_owned(),
        key: key.to_owned(),
        command: if key.is_empty() {
            Vec::new()
        } else {
            vec![POPUP_MENU_MARKER.to_owned(), key.to_owned()]
        },
    })
    .collect()
}

/// The first word of a popup menu item's command, which marks it as one the
/// popup handles itself rather than a tmux command.
const POPUP_MENU_MARKER: &str = "\u{0}popup-menu";

#[derive(Clone, Copy)]
struct PopupPlacement {
    left: u16,
    top: u16,
    width: u16,
    height: u16,
}

/// A drag in progress on a popup's border: button 1 moves it, button 3 resizes
/// it, and the offsets hold the grabbed point under the pointer.
#[derive(Clone, Copy)]
struct PopupDrag {
    resize: bool,
    dx: u16,
    dy: u16,
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
        reply: Option<super::super::state::PromptReply>,
    ) -> Self {
        let selected = selected.min(request.items.len().saturating_sub(1));
        Self {
            state: OverlayState::Menu(MenuOverlay { request, selected }),
            reply,
        }
    }

    pub(super) fn from_request(
        request: OverlayRequest,
        reply: Option<super::super::state::PromptReply>,
        cols: u16,
        rows: u16,
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
                // The popup is a spawn like any other, so it runs with the
                // environment `environ_for_session` built for it rather than
                // whatever the daemon happens to hold.
                let argv = if request.environment.is_empty() {
                    argv
                } else {
                    let mut wrapped = vec!["/usr/bin/env".to_string(), "-i".to_string()];
                    wrapped.extend(request.environment.iter().cloned());
                    wrapped.extend(argv);
                    wrapped
                };
                let refs = argv.iter().map(String::as_str).collect::<Vec<_>>();
                let mut pane = Pane::spawn(
                    &refs,
                    request.cwd.as_deref(),
                    inner_width.max(1),
                    inner_height.max(1),
                )?;
                let io = pane.take_event_io().map(Box::new);
                Some(Self {
                    state: OverlayState::Popup(PopupOverlay {
                        request: Box::new(request),
                        pane: Box::new(pane),
                        io,
                        read_continuation: false,
                        exit_status: None,
                        placement: None,
                        dragging: None,
                        last_pointer: None,
                        menu: None,
                        closing: false,
                    }),
                    reply,
                })
            }
        })
    }

    pub(super) fn complete(&mut self, result: command::CommandResult, inserted: bool) {
        if let Some(reply) = self.reply.take() {
            reply.send(Some(super::super::state::PromptCompletion {
                stdout: result.stdout,
                stderr: result.stderr,
                exit: result.exit,
                inserted,
            }));
        }
    }

    /// When this overlay next needs a pass without other activity: the
    /// display-panes expiry, or a popup's 50ms output poll. A menu waits on
    /// input alone.
    pub(super) fn deadline(&self, now: Instant) -> Option<Instant> {
        match &self.state {
            OverlayState::DisplayPanes(overlay) => Some(overlay.deadline),
            OverlayState::Popup(_) => Some(now + Duration::from_millis(50)),
            OverlayState::Menu(_) => None,
        }
    }

    /// The command a closing popup leaves behind, with the file it edited —
    /// tmux's `popup_editor_close_cb`, which reads the file back and unlinks
    /// it.
    pub(super) fn take_on_close(&mut self) -> Option<(Vec<String>, Option<std::path::PathBuf>)> {
        let OverlayState::Popup(overlay) = &mut self.state else {
            return None;
        };
        if overlay.request.on_close.is_empty() {
            return None;
        }
        Some((
            std::mem::take(&mut overlay.request.on_close),
            overlay.request.on_close_remove.take(),
        ))
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
        mouse: Option<&MouseEvent>,
        cols: u16,
        rows: u16,
        state: &SharedState,
        target: &str,
    ) -> OverlayInputOutcome {
        match &mut self.state {
            OverlayState::Menu(overlay) => match mouse {
                Some(mouse) => overlay.handle_mouse(mouse, cols, rows),
                None => overlay.handle_key(key),
            },
            OverlayState::Popup(overlay) => {
                overlay.handle_key(key, raw, mouse, cols, rows, state, target)
            }
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
            OverlayState::Menu(overlay) => {
                overlay.render(&mut out, state, target, cols, rows, terminal)
            }
            OverlayState::Popup(overlay) => {
                overlay.render(&mut out, state, target, cols, rows, terminal)
            }
            OverlayState::DisplayPanes(overlay) => {
                overlay.render(&mut out, state, target, terminal)
            }
        }
        out
    }
}

impl MenuOverlay {
    /// Where the menu's box sits and how big it is, so the mouse and the
    /// renderer agree on which row is which item.
    fn geometry(&self, cols: u16, rows: u16) -> (u16, u16, u16, u16) {
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
        let left = overlay_position(self.request.x.as_deref(), cols, width, false);
        let top = overlay_position(self.request.y.as_deref(), rows, height, true);
        (left, top, width, height)
    }

    /// tmux's `menu_key_cb` mouse half: the pointer picks the item under it,
    /// a release on one runs it, and a release outside closes the menu.
    fn handle_mouse(&mut self, mouse: &MouseEvent, cols: u16, rows: u16) -> OverlayInputOutcome {
        let (left, top, width, _) = self.geometry(cols, rows);
        let released = matches!(mouse.kind, MouseEventKind::Up | MouseEventKind::DragEnd);
        let last = self.request.items.len().saturating_sub(1);
        let inside = mouse.position.x >= left
            && mouse.position.x <= left.saturating_add(width)
            && mouse.position.y > top
            && mouse.position.y <= top.saturating_add(1).saturating_add(last as u16);
        if !inside {
            return if released {
                OverlayInputOutcome::close(0, None)
            } else {
                OverlayInputOutcome::stay()
            };
        }
        self.selected = usize::from(mouse.position.y.saturating_sub(top).saturating_sub(1));
        if released {
            return OverlayInputOutcome::close(
                0,
                self.request
                    .items
                    .get(self.selected)
                    .map(|item| item.command.clone()),
            );
        }
        OverlayInputOutcome::stay()
    }

    /// Move the selection one item, skipping the separators and disabled
    /// names tmux's own `menu_key_cb` walks past.
    fn step(&mut self, delta: isize) {
        let count = self.request.items.len();
        if count == 0 {
            return;
        }
        let mut index = self.selected;
        for _ in 0..count {
            index = ((index as isize + delta).rem_euclid(count as isize)) as usize;
            let item = &self.request.items[index];
            if !item.label.is_empty() && !item.label.starts_with('-') {
                self.selected = index;
                return;
            }
        }
    }

    fn handle_key(&mut self, key: &str) -> OverlayInputOutcome {
        if let Some(item) = self.request.items.iter().find(|item| {
            !item.key.is_empty()
                && item.key == key
                && !item.label.is_empty()
                && !item.label.starts_with('-')
        }) {
            return OverlayInputOutcome::close(0, Some(item.command.clone()));
        }
        match key {
            "q" | "Escape" | "C-c" | "C-g" | "C-[" => OverlayInputOutcome::close(0, None),
            "Up" | "k" | "BTab" => {
                self.step(-1);
                OverlayInputOutcome::stay()
            }
            "Down" | "j" => {
                self.step(1);
                OverlayInputOutcome::stay()
            }
            "PageUp" | "C-b" => {
                let mut count = 5;
                let mut index = self.selected;
                while count > 0 && index > 0 {
                    index -= 1;
                    let item = &self.request.items[index];
                    if !item.label.is_empty() && !item.label.starts_with('-') {
                        count -= 1;
                    }
                }
                while index < self.request.items.len() {
                    let item = &self.request.items[index];
                    if !item.label.is_empty() && !item.label.starts_with('-') {
                        self.selected = index;
                        break;
                    }
                    index += 1;
                }
                OverlayInputOutcome::stay()
            }
            "PageDown" => {
                let total = self.request.items.len();
                if total > 0 {
                    let mut count = 5;
                    let mut index = self.selected;
                    while count > 0 && index + 1 < total {
                        index += 1;
                        let item = &self.request.items[index];
                        if !item.label.is_empty() && !item.label.starts_with('-') {
                            count -= 1;
                        }
                    }
                    while index < total {
                        let item = &self.request.items[index];
                        if !item.label.is_empty() && !item.label.starts_with('-') {
                            self.selected = index;
                            break;
                        }
                        if index == 0 {
                            break;
                        }
                        index -= 1;
                    }
                }
                OverlayInputOutcome::stay()
            }
            "g" | "Home" => {
                if let Some(pos) = self
                    .request
                    .items
                    .iter()
                    .position(|item| !item.label.is_empty() && !item.label.starts_with('-'))
                {
                    self.selected = pos;
                }
                OverlayInputOutcome::stay()
            }
            "G" | "End" => {
                if let Some(pos) = self
                    .request
                    .items
                    .iter()
                    .rposition(|item| !item.label.is_empty() && !item.label.starts_with('-'))
                {
                    self.selected = pos;
                }
                OverlayInputOutcome::stay()
            }
            "Enter" | "C-m" => {
                let cmd = self.request.items.get(self.selected).and_then(|item| {
                    if !item.label.is_empty() && !item.label.starts_with('-') {
                        Some(item.command.clone())
                    } else {
                        None
                    }
                });
                OverlayInputOutcome::close(0, cmd)
            }
            _ => OverlayInputOutcome::stay(),
        }
    }

    fn render(
        &self,
        out: &mut Vec<u8>,
        state: &ServerState,
        target: &str,
        cols: u16,
        rows: u16,
        terminal: &dyn TerminalCapabilities,
    ) {
        let style = |option: &str, fallback: &str| {
            super::super::status::option_style_escape_for(state, target, option, fallback, terminal)
        };
        let menu_style = style("menu-style", "default");
        let selected_style = style("menu-selected-style", "bg=yellow,fg=black");
        let border_style = style("menu-border-style", "default");
        let glyphs = overlay_border_glyphs(
            state
                .option_for_target(target, "menu-border-lines")
                .unwrap_or("single"),
        );
        let (left, top, width, height) = self.geometry(cols, rows);
        draw_overlay_box_with(
            out,
            top,
            left,
            width,
            height,
            &self.request.title,
            glyphs,
            &border_style,
        );
        for (index, item) in self
            .request
            .items
            .iter()
            .take(height.saturating_sub(2) as usize)
            .enumerate()
        {
            if item.label.is_empty() {
                // tmux's `screen_write_hline` joins a separator to the box.
                out.extend_from_slice(
                    format!(
                        "\x1b[{};{}H├{}┤",
                        top + index as u16 + 2,
                        left + 1,
                        "─".repeat(width.saturating_sub(2) as usize)
                    )
                    .as_bytes(),
                );
                continue;
            }
            // A name starting with `-` is disabled: the dash is not shown, the
            // row is dimmed, and it never takes the selected style.
            let disabled = item.label.starts_with('-');
            let label = if disabled {
                &item.label[1..]
            } else {
                item.label.as_str()
            };
            let room = width.saturating_sub(4) as usize;
            let text = if item.key.is_empty() {
                clip_mode_line(label, room)
            } else {
                // tmux right-aligns the key inside the item's own width.
                let key = format!("({})", item.key);
                let label = clip_mode_line(label, room.saturating_sub(key.len() + 1));
                let gap = room
                    .saturating_sub(format::display_width(&label))
                    .saturating_sub(key.len());
                format!("{label}{}{key}", " ".repeat(gap))
            };
            out.extend_from_slice(
                format!("\x1b[{};{}H", top + index as u16 + 2, left + 3).as_bytes(),
            );
            out.extend_from_slice(if index == self.selected && !disabled {
                &selected_style
            } else {
                &menu_style
            });
            if disabled {
                out.extend_from_slice(b"\x1b[2m");
            }
            out.extend_from_slice(text.as_bytes());
            append_terminal_style_reset(out, terminal);
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

    /// Where the popup's box is: what a drag left behind, or what the command
    /// asked for.
    fn geometry(&self, cols: u16, rows: u16) -> PopupPlacement {
        if let Some(placement) = self.placement {
            return placement;
        }
        let width = overlay_dimension(self.request.width.as_deref(), cols, 50)
            .max(3)
            .min(cols.max(3));
        let height = overlay_dimension(self.request.height.as_deref(), rows, 50)
            .max(3)
            .min(rows.max(3));
        PopupPlacement {
            left: overlay_position(self.request.x.as_deref(), cols, width, false),
            top: overlay_position(self.request.y.as_deref(), rows, height, true),
            width,
            height,
        }
    }

    /// tmux's `popup_key_cb` mouse half and `popup_handle_drag`: a drag that
    /// starts on the border moves the popup with button 1 and resizes it with
    /// button 3. Everything else is the popup program's own.
    fn handle_mouse(
        &mut self,
        mouse: &MouseEvent,
        cols: u16,
        rows: u16,
        state: &SharedState,
        target: &str,
    ) -> bool {
        let place = self.geometry(cols, rows);
        let (x, y) = (mouse.position.x, mouse.position.y);
        let dragging = mouse.kind == MouseEventKind::Drag;
        let grabbed = self.last_pointer.unwrap_or((x, y));
        self.last_pointer = Some((x, y));
        if let Some(menu) = self.menu.as_mut() {
            let outcome = menu.handle_mouse(mouse, cols, rows);
            if outcome.close {
                let command = outcome.command.unwrap_or_default();
                self.menu = None;
                self.run_menu_item(
                    command.get(1).map(String::as_str),
                    cols,
                    rows,
                    state,
                    target,
                );
            }
            return true;
        }
        if let Some(drag) = self.dragging {
            if !dragging {
                self.dragging = None;
                return true;
            }
            let mut place = place;
            if drag.resize {
                let minimum = if self.request.border { 3 } else { 1 };
                if x < place.left + minimum || y < place.top + minimum {
                    return true;
                }
                place.width = x - place.left;
                place.height = y - place.top;
                let inset = u16::from(self.request.border) * 2;
                let _ = self.pane.resize(
                    place.width.saturating_sub(inset).max(1),
                    place.height.saturating_sub(inset).max(1),
                );
            } else {
                place.left = if x < drag.dx {
                    0
                } else {
                    (x - drag.dx).min(cols.saturating_sub(place.width))
                };
                place.top = if y < drag.dy {
                    0
                } else {
                    (y - drag.dy).min(rows.saturating_sub(place.height))
                };
                self.dragging = Some(PopupDrag {
                    dx: x.saturating_sub(place.left),
                    dy: y.saturating_sub(place.top),
                    ..drag
                });
            }
            self.placement = Some(place);
            return true;
        }
        let right_button = mouse.button == Some(super::super::mouse::MouseButton::Three);
        let outside = x < place.left
            || x >= place.left.saturating_add(place.width)
            || y < place.top
            || y >= place.top.saturating_add(place.height);
        if outside {
            // tmux opens the popup's menu on a right-click outside it.
            if right_button && mouse.kind == MouseEventKind::Down {
                self.open_menu(x, y, cols, rows);
            }
            return true;
        }
        let on_border = self.request.border
            && (x == place.left
                || x == place.left + place.width - 1
                || y == place.top
                || y == place.top + place.height - 1);
        // A right-click on the top or left border opens the menu rather than
        // starting a resize.
        if right_button
            && mouse.kind == MouseEventKind::Down
            && self.request.border
            && (x == place.left || y == place.top)
        {
            self.open_menu(x, y, cols, rows);
            return true;
        }
        if on_border && dragging {
            self.dragging = Some(PopupDrag {
                resize: mouse.button == Some(super::super::mouse::MouseButton::Three),
                dx: grabbed.0.saturating_sub(place.left),
                dy: grabbed.1.saturating_sub(place.top),
            });
            self.placement = Some(place);
            return true;
        }
        on_border
    }

    /// Open the popup's own menu, centred on the pointer as tmux centres it.
    fn open_menu(&mut self, x: u16, y: u16, cols: u16, rows: u16) {
        let mut menu = MenuOverlay {
            request: MenuRequest {
                title: String::new(),
                items: popup_menu_items(),
                selected: 0,
                x: None,
                y: None,
            },
            selected: 0,
        };
        // The menu is centred on the pointer, so its own width has to be
        // measured the way it will be drawn.
        let (_, _, width, height) = menu.geometry(cols, rows);
        menu.request.x = Some(x.saturating_sub(width / 2).to_string());
        // A menu's `-y` names the row below its last line.
        menu.request.y = Some(y.saturating_add(height).min(rows).to_string());
        self.menu = Some(menu);
    }

    /// Run a popup menu item, keyed as tmux keys it.
    fn run_menu_item(
        &mut self,
        key: Option<&str>,
        cols: u16,
        rows: u16,
        state: &SharedState,
        target: &str,
    ) {
        let place = self.geometry(cols, rows);
        match key {
            Some("q") => self.exit_status = Some(0),
            // tmux's `popup_make_pane`: the popup's running child moves into a
            // new pane split from the window's active one, and the popup is
            // done — there is nothing left in it to draw.
            Some(key @ ("h" | "v")) => {
                let direction = if key == "h" {
                    super::super::state::SplitDirection::LeftRight
                } else {
                    super::super::state::SplitDirection::TopBottom
                };
                let Ok(emptied) = Pane::inert(1, 1) else {
                    return;
                };
                let mut pane = std::mem::replace(&mut *self.pane, emptied);
                // The overlay drove this pane's I/O itself; the loop takes it
                // back over as soon as the pane is in a window.
                if let Some(io) = self.io.take() {
                    pane.restore_event_io(*io);
                }
                let inserted = state.borrow_mut().split_window_direction_with_spec(
                    target,
                    true,
                    false,
                    false,
                    direction,
                    super::super::state::PaneSpec::Existing(Box::new(pane)),
                    None,
                );
                if inserted.is_ok() {
                    self.closing = true;
                    self.exit_status = Some(0);
                }
            }
            Some("F") => {
                self.placement = Some(PopupPlacement {
                    left: 0,
                    top: 0,
                    width: cols,
                    height: rows,
                });
                let inset = u16::from(self.request.border) * 2;
                let _ = self.pane.resize(
                    cols.saturating_sub(inset).max(1),
                    rows.saturating_sub(inset).max(1),
                );
            }
            Some("C") => {
                self.placement = Some(PopupPlacement {
                    left: cols / 2 - place.width / 2,
                    top: rows / 2 - place.height / 2,
                    ..place
                });
            }
            _ => {}
        }
    }

    fn resize(&mut self, cols: u16, rows: u16) {
        if self.placement.is_some() {
            return;
        }
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

    #[allow(clippy::too_many_arguments)]
    fn handle_key(
        &mut self,
        key: &str,
        raw: &[u8],
        mouse: Option<&MouseEvent>,
        cols: u16,
        rows: u16,
        state: &SharedState,
        target: &str,
    ) -> OverlayInputOutcome {
        if let Some(mouse) = mouse {
            if self.handle_mouse(mouse, cols, rows, state, target) {
                if self.closing {
                    return OverlayInputOutcome::close(0, None);
                }
                return OverlayInputOutcome::stay();
            }
        }
        if let Some(menu) = self.menu.as_mut() {
            let outcome = menu.handle_key(key);
            if outcome.close {
                let command = outcome.command.unwrap_or_default();
                self.menu = None;
                self.run_menu_item(
                    command.get(1).map(String::as_str),
                    cols,
                    rows,
                    state,
                    target,
                );
            }
            if self.closing {
                return OverlayInputOutcome::close(0, None);
            }
            return OverlayInputOutcome::stay();
        }
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

    fn render(
        &self,
        out: &mut Vec<u8>,
        state: &ServerState,
        target: &str,
        cols: u16,
        rows: u16,
        terminal: &dyn TerminalCapabilities,
    ) {
        let PopupPlacement {
            left,
            top,
            width,
            height,
        } = self.geometry(cols, rows);
        let inset = u16::from(self.request.border);
        if self.request.border {
            let border_style = super::super::status::option_style_escape_for(
                state,
                target,
                "popup-border-style",
                "default",
                terminal,
            );
            draw_overlay_box_with(
                out,
                top,
                left,
                width,
                height,
                &self.request.title,
                overlay_border_glyphs(
                    state
                        .option_for_target(target, "popup-border-lines")
                        .unwrap_or("single"),
                ),
                &border_style,
            );
        }
        // `popup-style` is the popup's own default cell, applied before its
        // content so an unstyled program inherits it.
        out.extend_from_slice(&super::super::status::option_style_escape_for(
            state,
            target,
            "popup-style",
            "default",
            terminal,
        ));
        {
            let vt = self.pane.dump_vt();
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
        // The popup's own menu is drawn over it.
        if let Some(menu) = self.menu.as_ref() {
            menu.render(out, state, target, cols, rows, terminal);
        }
    }
}

impl DisplayPanesOverlay {
    fn handle_key(&self, key: &str, state: &SharedState, target: &str) -> OverlayInputOutcome {
        if !self.accept_input || matches!(key, "Escape" | "C-c") {
            return OverlayInputOutcome::close(0, None);
        }
        let index = if key.len() == 1 {
            let b = key.as_bytes()[0];
            match b {
                b'0'..=b'9' => Some((b - b'0') as usize),
                b'a'..=b'z' => Some((10 + b - b'a') as usize),
                _ => None,
            }
        } else {
            None
        };
        let Some(index) = index else {
            return OverlayInputOutcome::close(0, None);
        };
        let pane_id = state
            .borrow_mut()
            .active_window_panes(target)
            .ok()
            .and_then(|(window, _)| window.panes.get(index))
            .map(|pane| pane.id);
        let Some(pane_id) = pane_id else {
            return OverlayInputOutcome::close(0, None);
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
        let colour = state
            .option_for_target(target, "display-panes-colour")
            .unwrap_or("blue")
            .to_owned();
        let active_colour = state
            .option_for_target(target, "display-panes-active-colour")
            .unwrap_or("red")
            .to_owned();
        let Ok((window, active)) = state.active_window_panes(target) else {
            return;
        };
        for (index, pane) in window.panes.iter().enumerate() {
            let rect = window.pane_rect(pane.id).unwrap_or_default();
            let label = index.to_string();
            let colour = if index == active {
                active_colour.as_str()
            } else {
                colour.as_str()
            };
            // tmux draws the index in the clock's own 5x5 font when the pane is
            // big enough for it, with the colour as the *background* so the
            // glyph is a solid block; a smaller pane gets the plain number in
            // the same colour as its foreground.
            if usize::from(rect.width) < label.len() * 6 || rect.height < 5 {
                let row = rect.top + rect.height / 2 + 1;
                let col = rect.left + rect.width.saturating_sub(label.len() as u16) / 2 + 1;
                out.extend_from_slice(format!("\x1b[{row};{col}H").as_bytes());
                out.extend_from_slice(&super::super::status::style_escape_value(
                    &format!("fg={colour}"),
                    terminal,
                ));
                out.extend_from_slice(label.as_bytes());
                append_terminal_style_reset(out, terminal);
                continue;
            }
            let block = super::super::status::style_escape_value(
                &format!("fg={colour},bg={colour}"),
                terminal,
            );
            let left = rect.left + rect.width / 2 - label.len() as u16 * 3;
            let top = rect.top + rect.height / 2 - 2;
            for (digit, character) in label.chars().enumerate() {
                let Some(glyph) = super::clock_glyph(character) else {
                    continue;
                };
                for (row, line) in glyph.iter().enumerate() {
                    for (column, cell) in line.iter().enumerate() {
                        if *cell != 1 {
                            continue;
                        }
                        let x = left + digit as u16 * 6 + column as u16 + 1;
                        let y = top + row as u16 + 1;
                        out.extend_from_slice(format!("\x1b[{y};{x}H").as_bytes());
                        out.extend_from_slice(&block);
                        out.push(b' ');
                    }
                }
            }
            // The pane's size goes in its top-right corner, in the same colour
            // as a foreground — which is what makes the overlay visible at all
            // when the digits themselves are blocks of blank cells.
            if rect.height > 6 {
                let size = format!("{}x{}", rect.width, rect.height);
                if usize::from(rect.width) >= size.len() {
                    let x = rect.left + rect.width - size.len() as u16 + 1;
                    out.extend_from_slice(format!("\x1b[{};{x}H", rect.top + 1).as_bytes());
                    out.extend_from_slice(&super::super::status::style_escape_value(
                        &format!("fg={colour}"),
                        terminal,
                    ));
                    out.extend_from_slice(size.as_bytes());
                }
            }
            append_terminal_style_reset(out, terminal);
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

/// Where an overlay's box starts on one axis.
///
/// A vertical position names the row *below* the box's last line, as tmux's
/// `cmd_display_menu_get_pos` does — it subtracts the height before clamping,
/// so `-y 10` puts a four-row menu on rows 6 to 9.
fn overlay_position(value: Option<&str>, available: u16, size: u16, vertical: bool) -> u16 {
    let limit = available.saturating_sub(size);
    match value {
        Some("C" | "M" | "P" | "W" | "S") | None => limit / 2,
        Some(value) if value.ends_with('%') => value
            .trim_end_matches('%')
            .parse::<u32>()
            .ok()
            .map(|percent| (u32::from(limit) * percent / 100) as u16)
            .unwrap_or(0),
        Some(value) => {
            let position = value.parse::<u16>().unwrap_or(0);
            let position = if vertical {
                position.checked_sub(size).unwrap_or(0)
            } else {
                position
            };
            position.min(limit)
        }
    }
}

/// The four corner and two line glyphs a `*-border-lines` value draws with, in
/// tmux's `tty_acs_*_borders` order: top-left, top-right, bottom-left,
/// bottom-right, horizontal, vertical.
pub(super) fn overlay_border_glyphs(lines: &str) -> [&'static str; 6] {
    match lines {
        "double" => ["╔", "╗", "╚", "╝", "═", "║"],
        "heavy" => ["┏", "┓", "┗", "┛", "━", "┃"],
        "rounded" => ["╭", "╮", "╰", "╯", "─", "│"],
        "simple" => ["+", "+", "+", "+", "-", "|"],
        "padded" => [" ", " ", " ", " ", " ", " "],
        "none" => ["", "", "", "", "", ""],
        _ => ["┌", "┐", "└", "┘", "─", "│"],
    }
}

fn draw_overlay_box_with(
    out: &mut Vec<u8>,
    top: u16,
    left: u16,
    width: u16,
    height: u16,
    title: &str,
    glyphs: [&str; 6],
    style: &[u8],
) {
    if width < 2 || height < 2 {
        return;
    }
    let [top_left, top_right, bottom_left, bottom_right, horizontal, vertical] = glyphs;
    if horizontal.is_empty() {
        return;
    }
    let inner = width.saturating_sub(2) as usize;
    out.extend_from_slice(style);
    let mut top_line = format!("{top_left}{}{top_right}", horizontal.repeat(inner));
    // tmux draws the whole line and then writes the title over it two cells in,
    // with no padding of its own (`screen_write_box`).
    if !title.is_empty() && inner > 2 {
        let shown = clip_mode_line(title, inner.saturating_sub(1));
        let mut chars = top_line.chars().collect::<Vec<_>>();
        for (index, character) in shown.chars().enumerate() {
            if index + 2 < chars.len().saturating_sub(1) {
                chars[index + 2] = character;
            }
        }
        top_line = chars.into_iter().collect();
    }
    out.extend_from_slice(format!("\x1b[{};{}H{}", top + 1, left + 1, top_line).as_bytes());
    for row in 1..height.saturating_sub(1) {
        out.extend_from_slice(
            format!(
                "\x1b[{};{}H{vertical}\x1b[{};{}H{vertical}",
                top + row + 1,
                left + 1,
                top + row + 1,
                left + width
            )
            .as_bytes(),
        );
    }
    out.extend_from_slice(
        format!(
            "\x1b[{};{}H{bottom_left}{}{bottom_right}",
            top + height,
            left + 1,
            horizontal.repeat(inner)
        )
        .as_bytes(),
    );
}
