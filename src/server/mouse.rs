//! Normalized mouse input and geometry-based target resolution.

use std::time::{Duration, Instant};

use super::key::{KeyBase, KeyCode, Modifiers, MouseKey, MouseKind, MouseLocation};
use super::state::{PaneNode, PaneRect, ServerState, Session, Window};
use super::status::{self, RenderedStatus, StatusRangeKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MouseProtocol {
    Legacy,
    Sgr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MouseEventKind {
    Move,
    Down,
    Up,
    Drag,
    DragEnd,
    WheelUp,
    WheelDown,
    SecondClick,
    DoubleClick,
    TripleClick,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MouseButton {
    One,
    Two,
    Three,
    WheelUp,
    WheelDown,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Eleven,
    Unknown(u16),
}

impl MouseButton {
    fn binding_number(self) -> Option<u8> {
        match self {
            Self::One => Some(1),
            Self::Two => Some(2),
            Self::Three => Some(3),
            Self::Six => Some(6),
            Self::Seven => Some(7),
            Self::Eight => Some(8),
            Self::Nine => Some(9),
            Self::Ten => Some(10),
            Self::Eleven => Some(11),
            Self::WheelUp | Self::WheelDown | Self::Unknown(_) => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct MouseModifiers {
    pub(crate) shift: bool,
    pub(crate) meta: bool,
    pub(crate) control: bool,
}

impl MouseModifiers {
    fn key_modifiers(self) -> Modifiers {
        Modifiers::new(self.meta, self.control, self.shift)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct MousePosition {
    pub(crate) x: u16,
    pub(crate) y: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MouseStatusRange {
    Left,
    Right,
    Window(u32),
    Pane(u32),
    Session(u32),
    User(String),
    Control(u8),
}

/// Which of a pane's four borders an event landed on. `resize-pane -M` needs
/// this to know which way the border it grabbed moves the pane.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BorderSide {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MouseTarget {
    pub(crate) session_id: u32,
    pub(crate) window_id: Option<u32>,
    pub(crate) pane_id: Option<u32>,
    pub(crate) location: MouseLocation,
    pub(crate) local_position: Option<MousePosition>,
    pub(crate) status_line: Option<u16>,
    pub(crate) status_range: Option<MouseStatusRange>,
    pub(crate) border_side: Option<BorderSide>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MouseEvent {
    pub(crate) kind: MouseEventKind,
    pub(crate) button: Option<MouseButton>,
    pub(crate) modifiers: MouseModifiers,
    pub(crate) position: MousePosition,
    pub(crate) last_position: Option<MousePosition>,
    pub(crate) last_button: Option<MouseButton>,
    pub(crate) protocol: MouseProtocol,
    pub(crate) raw_button: u16,
    pub(crate) target: Option<MouseTarget>,
    /// tmux's `m->ignore`: set on the click timer's replayed `DoubleClick`,
    /// which never reaches a pane because the terminal already sent the
    /// presses this event is derived from.
    pub(crate) ignore: bool,
}

impl MouseEvent {
    pub(crate) fn from_terminal_report(
        protocol: MouseProtocol,
        raw_button: u16,
        release: bool,
        position: MousePosition,
    ) -> Self {
        let code = raw_button & 0xc3;
        let modifiers = MouseModifiers {
            shift: raw_button & 4 != 0,
            meta: raw_button & 8 != 0,
            control: raw_button & 16 != 0,
        };
        let button = match code {
            0 => Some(MouseButton::One),
            1 => Some(MouseButton::Two),
            2 => Some(MouseButton::Three),
            3 => None,
            64 => Some(MouseButton::WheelUp),
            65 => Some(MouseButton::WheelDown),
            66 => Some(MouseButton::Six),
            67 => Some(MouseButton::Seven),
            128 => Some(MouseButton::Eight),
            129 => Some(MouseButton::Nine),
            130 => Some(MouseButton::Ten),
            131 => Some(MouseButton::Eleven),
            other => Some(MouseButton::Unknown(other)),
        };
        let kind = if code == 64 {
            MouseEventKind::WheelUp
        } else if code == 65 {
            MouseEventKind::WheelDown
        } else if release {
            MouseEventKind::Up
        } else if code == 3 && raw_button & 32 != 0 {
            MouseEventKind::Move
        } else if raw_button & 32 != 0 {
            MouseEventKind::Drag
        } else {
            MouseEventKind::Down
        };
        Self {
            kind,
            button,
            modifiers,
            position,
            last_position: None,
            last_button: None,
            protocol,
            raw_button,
            target: None,
            ignore: false,
        }
    }

    pub(crate) fn key_code_for(&self, location: MouseLocation) -> Option<KeyCode> {
        let (kind, button) = match self.kind {
            MouseEventKind::Move => (MouseKind::Move, 0),
            MouseEventKind::Down => (MouseKind::Down, self.button?.binding_number()?),
            MouseEventKind::Up => (MouseKind::Up, self.button?.binding_number()?),
            MouseEventKind::Drag => (MouseKind::Drag, self.button?.binding_number()?),
            MouseEventKind::DragEnd => (MouseKind::DragEnd, self.button?.binding_number()?),
            MouseEventKind::WheelUp => (MouseKind::WheelUp, 0),
            MouseEventKind::WheelDown => (MouseKind::WheelDown, 0),
            MouseEventKind::SecondClick => (MouseKind::SecondClick, self.button?.binding_number()?),
            MouseEventKind::DoubleClick => (MouseKind::DoubleClick, self.button?.binding_number()?),
            MouseEventKind::TripleClick => (MouseKind::TripleClick, self.button?.binding_number()?),
        };
        Some(KeyCode::new(
            KeyBase::Mouse(MouseKey {
                kind,
                button,
                location,
            }),
            self.modifiers.key_modifiers(),
        ))
    }

    pub(crate) fn key_code(&self) -> Option<KeyCode> {
        self.key_code_for(self.target.as_ref()?.location)
    }

    pub(crate) fn pane_position(&self) -> MousePosition {
        self.target
            .as_ref()
            .and_then(|target| target.local_position)
            .unwrap_or(self.position)
    }

    pub(crate) fn pane_input_event(&self) -> Option<(u32, ghostty_sys::MouseEvent)> {
        if self.ignore {
            return None;
        }
        let target = self.target.as_ref()?;
        let pane_id = target.pane_id?;
        let position = target.local_position?;
        let action = match self.kind {
            MouseEventKind::Up | MouseEventKind::DragEnd => ghostty_sys::MouseAction::Release,
            MouseEventKind::Move | MouseEventKind::Drag => ghostty_sys::MouseAction::Motion,
            MouseEventKind::Down
            | MouseEventKind::WheelUp
            | MouseEventKind::WheelDown
            | MouseEventKind::SecondClick
            | MouseEventKind::DoubleClick
            | MouseEventKind::TripleClick => ghostty_sys::MouseAction::Press,
        };
        let button = self.button.and_then(|button| match button {
            MouseButton::One => Some(ghostty_sys::MouseButton::Left),
            MouseButton::Two => Some(ghostty_sys::MouseButton::Middle),
            MouseButton::Three => Some(ghostty_sys::MouseButton::Right),
            MouseButton::WheelUp => Some(ghostty_sys::MouseButton::WheelUp),
            MouseButton::WheelDown => Some(ghostty_sys::MouseButton::WheelDown),
            MouseButton::Six => Some(ghostty_sys::MouseButton::Six),
            MouseButton::Seven => Some(ghostty_sys::MouseButton::Seven),
            MouseButton::Eight => Some(ghostty_sys::MouseButton::Eight),
            MouseButton::Nine => Some(ghostty_sys::MouseButton::Nine),
            MouseButton::Ten => Some(ghostty_sys::MouseButton::Ten),
            MouseButton::Eleven => Some(ghostty_sys::MouseButton::Eleven),
            MouseButton::Unknown(_) => None,
        });
        let any_button_pressed = action != ghostty_sys::MouseAction::Release
            && button.is_some_and(|button| {
                !matches!(
                    button,
                    ghostty_sys::MouseButton::WheelUp | ghostty_sys::MouseButton::WheelDown
                )
            });
        Some((
            pane_id,
            ghostty_sys::MouseEvent {
                action,
                button,
                shift: self.modifiers.shift,
                control: self.modifiers.control,
                alt: self.modifiers.meta,
                column: position.x,
                row: position.y,
                any_button_pressed,
            },
        ))
    }
}

/// The `#{mouse_status_range}` spelling of a status range.
pub(crate) fn range_name(range: &MouseStatusRange) -> String {
    match range {
        MouseStatusRange::Left => "left".to_string(),
        MouseStatusRange::Right => "right".to_string(),
        MouseStatusRange::Window(_) => "window".to_string(),
        MouseStatusRange::Pane(_) => "pane".to_string(),
        MouseStatusRange::Session(_) => "session".to_string(),
        // A user range reports its own argument, not a fixed word.
        MouseStatusRange::User(value) => value.clone(),
        MouseStatusRange::Control(_) => "control".to_string(),
    }
}

/// `#{mouse_word}` and `#{mouse_line}` for a pane-local position.
///
/// tmux reads these straight off the grid (`format_grid_word`/
/// `format_grid_line`) rather than through copy mode, and stops a word at the
/// first `word-separators` character in either direction. Unlike tmux this does
/// not follow a wrapped line into its neighbour, which only shows up for a word
/// straddling the right margin.
pub(crate) fn grid_word_and_line(
    pane: &super::pane::Pane,
    position: MousePosition,
    separators: &str,
) -> (String, String) {
    let Ok(dump) = pane.dump() else {
        return (String::new(), String::new());
    };
    let history = pane.scrollback_rows().unwrap_or(0);
    let Some(line) = dump.lines().nth(history + usize::from(position.y)) else {
        return (String::new(), String::new());
    };
    let line = line.trim_end_matches(' ');
    let cells: Vec<char> = line.chars().collect();
    let index = usize::from(position.x);
    let is_separator = |ch: char| ch == ' ' || separators.contains(ch);
    let word = match cells.get(index) {
        Some(&ch) if !is_separator(ch) => {
            let start = cells[..index]
                .iter()
                .rposition(|&ch| is_separator(ch))
                .map_or(0, |at| at + 1);
            let end = cells[index..]
                .iter()
                .position(|&ch| is_separator(ch))
                .map_or(cells.len(), |at| index + at);
            cells[start..end].iter().collect()
        }
        _ => String::new(),
    };
    (word, line.to_string())
}

/// tmux's `KEYC_CLICK_TIMEOUT`.
const CLICK_TIMEOUT: Duration = Duration::from_millis(300);

/// The press the click timer replays if no further click arrives.
#[derive(Clone, Debug)]
struct ClickState {
    event: MouseEvent,
    button: Option<MouseButton>,
    identity: Option<(u32, Option<u32>, Option<u32>, MouseLocation)>,
    deadline: Instant,
}

/// The per-client mouse state a report is interpreted against: the previous
/// position and button, whether a drag is in flight, and tmux's two-flag repeat
/// click machine.
///
/// The click shape is not the obvious one. A second press is `SecondClick`, a
/// third is `TripleClick`, and `DoubleClick` is delivered *afterwards* by the
/// timer when no third press arrives — so a binding on `DoubleClick1Pane` runs
/// 300ms after the second release, not on it.
#[derive(Default)]
pub(crate) struct MouseInputState {
    last_position: Option<MousePosition>,
    last_button: Option<MouseButton>,
    drag_button: Option<MouseButton>,
    /// tmux's `CLIENT_DOUBLECLICK`: one press has landed, so the next is a
    /// `SecondClick`.
    expect_second: bool,
    /// tmux's `CLIENT_TRIPLECLICK`: two presses have landed, so the next is a
    /// `TripleClick` — and if none comes, the timer replays a `DoubleClick`.
    expect_third: bool,
    click: Option<ClickState>,
    /// What the in-flight drag started on, tmux's `mouse_last_pane` plus the
    /// drag callback it installed.
    drag_origin: Option<MouseTarget>,
}

impl MouseInputState {
    /// Fold one freshly resolved report into the client's mouse state,
    /// rewriting its kind to the key tmux would dispatch.
    pub(crate) fn observe(&mut self, event: &mut MouseEvent, now: Instant) {
        event.last_position = self.last_position;
        event.last_button = self.last_button;

        if event.kind == MouseEventKind::Up && event.button.is_none() {
            event.button = self.last_button;
        }
        if event.kind == MouseEventKind::Up && self.drag_button == event.button {
            event.kind = MouseEventKind::DragEnd;
        }
        match event.kind {
            MouseEventKind::Drag => {
                // A drag belongs to whatever it started on for as long as it
                // lasts. tmux gets this from the drag callback the opening
                // command installed, which later reports bypass the key tables
                // to reach; keeping the origin's location and pane on the event
                // is the same thing expressed through the ordinary dispatch, so
                // a border drag keeps producing `MouseDrag1Border` after the
                // pointer has left the border.
                match self.drag_origin.as_ref() {
                    Some(origin) => {
                        if let Some(target) = event.target.as_mut() {
                            target.location = origin.location;
                            target.window_id = origin.window_id;
                            target.pane_id = origin.pane_id;
                        } else {
                            event.target = Some(origin.clone());
                        }
                    }
                    None => self.drag_origin = event.target.clone(),
                }
                self.drag_button = event.button;
            }
            MouseEventKind::Up | MouseEventKind::DragEnd => {
                self.drag_button = None;
                self.drag_origin = None;
            }
            _ => {}
        }

        if event.kind == MouseEventKind::Down {
            if self.expect_second {
                self.expect_second = false;
                self.expect_third = true;
                event.kind = MouseEventKind::SecondClick;
            } else if self.expect_third {
                self.expect_third = false;
                event.kind = MouseEventKind::TripleClick;
            } else {
                self.expect_second = true;
            }
        }

        if matches!(
            event.kind,
            MouseEventKind::Down | MouseEventKind::SecondClick | MouseEventKind::TripleClick
        ) {
            let identity = click_identity(event.target.as_ref());
            // A repeat click that moved, changed button, or landed somewhere
            // else is not a repeat at all: the sequence restarts here.
            if event.kind != MouseEventKind::Down
                && self.click.as_ref().is_none_or(|click| {
                    click.button != event.button || click.identity != identity
                })
            {
                event.kind = MouseEventKind::Down;
                self.expect_third = false;
                self.expect_second = true;
            }
            if event.kind != MouseEventKind::TripleClick {
                self.click = Some(ClickState {
                    event: event.clone(),
                    button: event.button,
                    identity,
                    deadline: now + CLICK_TIMEOUT,
                });
            }
        }

        self.last_position = Some(event.position);
        self.last_button = event.button;
    }

    /// Where the opening report of a drag resolves.
    ///
    /// tmux starts a drag at `m->lx/m->ly` — the position the button went down
    /// — so a press on a border followed by motion away from it still resolves
    /// as a border drag.
    pub(crate) fn drag_start_position(&self, event: &MouseEvent) -> Option<MousePosition> {
        (event.kind == MouseEventKind::Drag && self.drag_button.is_none())
            .then_some(self.last_position)
            .flatten()
    }

    /// When the client next has to wake up for the repeat-click timer.
    pub(crate) fn click_deadline(&self) -> Option<Instant> {
        (self.expect_second || self.expect_third)
            .then(|| self.click.as_ref().map(|click| click.deadline))
            .flatten()
    }

    /// tmux's `server_client_click_timer`. Two presses with no third mean the
    /// gesture was a double click after all, so replay the stored press as one;
    /// a single press just ends the sequence.
    pub(crate) fn expire_click(&mut self, now: Instant) -> Option<MouseEvent> {
        let click = self.click.as_ref()?;
        if now < click.deadline || !(self.expect_second || self.expect_third) {
            return None;
        }
        let replay = self.expect_third.then(|| {
            let mut event = click.event.clone();
            event.kind = MouseEventKind::DoubleClick;
            // The pane already saw the presses this is derived from.
            event.ignore = true;
            event
        });
        self.expect_second = false;
        self.expect_third = false;
        self.click = None;
        replay
    }
}

fn click_identity(
    target: Option<&MouseTarget>,
) -> Option<(u32, Option<u32>, Option<u32>, MouseLocation)> {
    target.map(|target| {
        (
            target.session_id,
            target.window_id,
            target.pane_id,
            target.location,
        )
    })
}

pub(crate) fn resolve_event(
    state: &ServerState,
    session_name: &str,
    terminal_rows: u16,
    rendered_status: &RenderedStatus,
    event: &mut MouseEvent,
) {
    event.target = None;
    let Some(session) = state.find(session_name) else {
        return;
    };
    let Ok((window, active)) = state.active_window_panes(session_name) else {
        return;
    };
    let status_height = status::height(state, session_name);
    let status_top = status::at_top(state, session_name);
    let status_start = if status_top {
        0
    } else {
        terminal_rows.saturating_sub(status_height)
    };
    if status_height > 0
        && event.position.y >= status_start
        && event.position.y < status_start.saturating_add(status_height)
    {
        let line = event.position.y - status_start;
        let range = rendered_status
            .range_at(usize::from(line), event.position.x)
            .map(|range| range.kind.clone());
        event.target = Some(resolve_status_target(session, window, line, range));
        return;
    }

    let pane_y = if status_top {
        event.position.y.saturating_sub(status_height)
    } else {
        event.position.y
    };
    let point = MousePosition {
        x: event.position.x,
        y: pane_y,
    };
    event.target = resolve_pane_target(state, session.id, session_name, window, active, point);
}

fn resolve_status_target(
    session: &Session,
    window: &Window,
    line: u16,
    range: Option<StatusRangeKind>,
) -> MouseTarget {
    let mut target = MouseTarget {
        session_id: session.id,
        window_id: None,
        pane_id: None,
        location: MouseLocation::StatusDefault,
        local_position: None,
        status_line: Some(line),
        status_range: None,
        border_side: None,
    };
    match range {
        None => {}
        Some(StatusRangeKind::Left) => {
            target.location = MouseLocation::StatusLeft;
            target.status_range = Some(MouseStatusRange::Left);
        }
        Some(StatusRangeKind::Right) => {
            target.location = MouseLocation::StatusRight;
            target.status_range = Some(MouseStatusRange::Right);
        }
        Some(StatusRangeKind::Window(index)) => {
            target.location = MouseLocation::Status;
            target.window_id = session
                .windows
                .iter()
                .find(|link| link.index == index)
                .map(|link| link.id);
            target.status_range = Some(MouseStatusRange::Window(index));
        }
        Some(StatusRangeKind::Pane(id)) => {
            target.location = MouseLocation::Status;
            target.window_id = Some(window.id);
            target.pane_id = Some(id);
            target.status_range = Some(MouseStatusRange::Pane(id));
        }
        Some(StatusRangeKind::Session(id)) => {
            target.location = MouseLocation::Status;
            target.session_id = id;
            target.status_range = Some(MouseStatusRange::Session(id));
        }
        Some(StatusRangeKind::User(value)) => {
            target.location = MouseLocation::Status;
            target.status_range = Some(MouseStatusRange::User(value));
        }
        Some(StatusRangeKind::Control(control)) => {
            let control = u8::try_from(control).unwrap_or(9).min(9);
            target.location = MouseLocation::Control(control);
            target.status_range = Some(MouseStatusRange::Control(control));
        }
    }
    target
}

fn resolve_pane_target(
    state: &ServerState,
    session_id: u32,
    session_name: &str,
    window: &Window,
    active: usize,
    point: MousePosition,
) -> Option<MouseTarget> {
    let mut order = (0..window.panes.len()).collect::<Vec<_>>();
    order.sort_by_key(|&index| {
        (
            window.panes[index].floating.is_some(),
            index == active,
            index,
        )
    });
    order.reverse();

    for index in order.iter().copied() {
        let pane = &window.panes[index];
        let rect = window.pane_rect(pane.id)?;
        if contains(rect, point) {
            let local = MousePosition {
                x: point.x - rect.left,
                y: point.y - rect.top,
            };
            let location = scrollbar_location(state, session_name, pane, rect, local)
                .unwrap_or(MouseLocation::Pane);
            return Some(MouseTarget {
                session_id,
                window_id: Some(window.id),
                pane_id: Some(pane.id),
                location,
                local_position: Some(local),
                status_line: None,
                status_range: None,
                border_side: None,
            });
        }
    }
    for index in order {
        let pane = &window.panes[index];
        let rect = window.pane_rect(pane.id)?;
        if let Some(side) = on_border(rect, point) {
            return Some(MouseTarget {
                session_id,
                window_id: Some(window.id),
                pane_id: Some(pane.id),
                location: MouseLocation::Border,
                local_position: None,
                status_line: None,
                status_range: None,
                border_side: Some(side),
            });
        }
    }
    None
}

fn contains(rect: PaneRect, point: MousePosition) -> bool {
    point.x >= rect.left
        && point.x < rect.left.saturating_add(rect.width)
        && point.y >= rect.top
        && point.y < rect.top.saturating_add(rect.height)
}

fn on_border(rect: PaneRect, point: MousePosition) -> Option<BorderSide> {
    let right = rect.left.saturating_add(rect.width);
    let bottom = rect.top.saturating_add(rect.height);
    if point.y >= rect.top && point.y <= bottom {
        if point.x == right {
            return Some(BorderSide::Right);
        }
        if rect.left.checked_sub(1) == Some(point.x) {
            return Some(BorderSide::Left);
        }
    }
    if point.x >= rect.left && point.x <= right {
        if point.y == bottom {
            return Some(BorderSide::Bottom);
        }
        if rect.top.checked_sub(1) == Some(point.y) {
            return Some(BorderSide::Top);
        }
    }
    None
}

fn scrollbar_location(
    state: &ServerState,
    session_name: &str,
    pane: &PaneNode,
    rect: PaneRect,
    local: MousePosition,
) -> Option<MouseLocation> {
    let pane_target = format!("%{}", pane.id);
    match state
        .option_for_target(&pane_target, "pane-scrollbars")
        .or_else(|| state.option_for_target(session_name, "pane-scrollbars"))
        .unwrap_or("off")
    {
        "on" => {}
        "modal" if pane.copy.is_some() => {}
        _ => return None,
    }
    let style = state
        .option_for_target(&pane_target, "pane-scrollbars-style")
        .or_else(|| state.option_for_target(session_name, "pane-scrollbars-style"))
        .unwrap_or("bg=black,fg=white,width=1,pad=0");
    let width = style
        .split(',')
        .find_map(|part| part.strip_prefix("width=")?.parse::<u16>().ok())
        .unwrap_or(1)
        .clamp(1, rect.width.max(1));
    let left = state
        .option_for_target(&pane_target, "pane-scrollbars-position")
        .or_else(|| state.option_for_target(session_name, "pane-scrollbars-position"))
        == Some("left");
    let in_bar = if left {
        local.x < width
    } else {
        local.x >= rect.width.saturating_sub(width)
    };
    if !in_bar {
        return None;
    }

    let height = usize::from(rect.height.max(1));
    let (history, viewport, scroll) = if let Some(copy) = pane.copy.as_ref() {
        (
            copy.grid.scrollback_rows,
            usize::from(copy.grid.viewport_rows),
            copy.scroll,
        )
    } else {
        (
            pane.pane.scrollback_rows().unwrap_or(0),
            usize::from(rect.height.max(1)),
            0,
        )
    };
    let total = history.saturating_add(viewport).max(1);
    let slider_height = (height.saturating_mul(viewport) / total).clamp(1, height);
    let available = height.saturating_sub(slider_height);
    let slider_top = history
        .saturating_sub(scroll.min(history))
        .saturating_mul(available)
        .checked_div(history)
        .unwrap_or(0);
    let row = usize::from(local.y).min(height - 1);
    if row < slider_top {
        Some(MouseLocation::ScrollbarUp)
    } else if row < slider_top.saturating_add(slider_height) {
        Some(MouseLocation::ScrollbarSlider)
    } else {
        Some(MouseLocation::ScrollbarDown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::state::{PaneSpec, SplitDirection};
    use crate::server::status::RenderCache;

    fn event(x: u16, y: u16) -> MouseEvent {
        MouseEvent::from_terminal_report(MouseProtocol::Sgr, 0, false, MousePosition { x, y })
    }

    #[test]
    fn resolves_split_panes_and_local_coordinates() {
        let mut state = ServerState::with_test_session().expect("state");
        state.resize_session("0", 80, 23).expect("resize");
        state
            .split_window_direction_with_spec(
                "0",
                true,
                false,
                SplitDirection::LeftRight,
                PaneSpec::Inert,
            )
            .expect("split");
        let mut cache = RenderCache::default();
        let rendered = cache.render(&state, "0", 80, 24);
        let mut mouse = event(60, 4);
        resolve_event(&state, "0", 24, rendered, &mut mouse);
        let target = mouse.target.expect("target");
        assert_eq!(target.pane_id, Some(1));
        assert_eq!(target.location, MouseLocation::Pane);
        assert_eq!(target.local_position, Some(MousePosition { x: 19, y: 4 }));

        let mut border_mouse = event(40, 4);
        resolve_event(&state, "0", 24, rendered, &mut border_mouse);
        let border = border_mouse.target.expect("border target");
        assert!(border.pane_id.is_some());
        assert_eq!(border.location, MouseLocation::Border);
    }

    #[test]
    fn pane_input_event_maps_local_release_and_modifiers() {
        let mut mouse = MouseEvent::from_terminal_report(
            MouseProtocol::Sgr,
            1 | 4 | 8 | 16,
            true,
            MousePosition { x: 40, y: 12 },
        );
        mouse.target = Some(MouseTarget {
            session_id: 0,
            window_id: Some(0),
            pane_id: Some(7),
            location: MouseLocation::Pane,
            local_position: Some(MousePosition { x: 3, y: 2 }),
            status_line: None,
            status_range: None,
            border_side: None,
        });

        let (pane_id, event) = mouse.pane_input_event().expect("pane input event");
        assert_eq!(pane_id, 7);
        assert_eq!(
            event,
            ghostty_sys::MouseEvent {
                action: ghostty_sys::MouseAction::Release,
                button: Some(ghostty_sys::MouseButton::Middle),
                shift: true,
                control: true,
                alt: true,
                column: 3,
                row: 2,
                any_button_pressed: false,
            }
        );
    }

    #[test]
    fn resolves_scrollbar_slider_geometry() {
        let mut state = ServerState::with_test_session().expect("state");
        state.resize_session("0", 80, 23).expect("resize");
        state.set_global_option("pane-scrollbars", "on");
        let (window, active) = state.active_window_panes("0").expect("window");
        window.panes[active]
            .pane
            .feed(b"1\r\n2\r\n3\r\n4\r\n5\r\n6\r\n7\r\n8\r\n9\r\n10\r\n11\r\n12\r\n13\r\n14\r\n15\r\n16\r\n17\r\n18\r\n19\r\n20\r\n21\r\n22\r\n23\r\n24\r\n25\r\n26\r\n27\r\n28\r\n29\r\n30");
        let mut cache = RenderCache::default();
        let rendered = cache.render(&state, "0", 80, 24);

        let mut above = event(79, 0);
        resolve_event(&state, "0", 24, rendered, &mut above);
        assert_eq!(
            above.target.as_ref().map(|target| target.location),
            Some(MouseLocation::ScrollbarUp)
        );
        let mut slider = event(79, 22);
        resolve_event(&state, "0", 24, rendered, &mut slider);
        assert_eq!(
            slider.target.as_ref().map(|target| target.location),
            Some(MouseLocation::ScrollbarSlider)
        );
    }

    #[test]
    fn resolves_status_ranges_and_top_status_offset() {
        let mut state = ServerState::with_test_session().expect("state");
        state.set_global_option("status-position", "top");
        state.set_global_option("status-format[0]", "#[range=control|8]x#[norange]");
        let mut cache = RenderCache::default();
        let rendered = cache.render(&state, "0", 80, 24);

        let mut status_mouse = event(0, 0);
        resolve_event(&state, "0", 24, rendered, &mut status_mouse);
        assert_eq!(
            status_mouse.target.as_ref().map(|target| target.location),
            Some(MouseLocation::Control(8))
        );

        let mut pane_mouse = event(3, 1);
        resolve_event(&state, "0", 24, rendered, &mut pane_mouse);
        assert_eq!(
            pane_mouse
                .target
                .as_ref()
                .and_then(|target| target.local_position),
            Some(MousePosition { x: 3, y: 0 })
        );
    }

    #[test]
    fn tracks_previous_state_drag_end_and_click_count() {
        let mut state = MouseInputState::default();
        let now = Instant::now();
        let mut down = event(2, 3);
        state.observe(&mut down, now);
        let mut drag = MouseEvent::from_terminal_report(
            MouseProtocol::Sgr,
            32,
            false,
            MousePosition { x: 4, y: 5 },
        );
        state.observe(&mut drag, now);
        assert_eq!(drag.last_position, Some(MousePosition { x: 2, y: 3 }));
        let mut up = MouseEvent::from_terminal_report(
            MouseProtocol::Sgr,
            0,
            true,
            MousePosition { x: 4, y: 5 },
        );
        state.observe(&mut up, now);
        assert_eq!(up.kind, MouseEventKind::DragEnd);

        let mut legacy_release = MouseEvent::from_terminal_report(
            MouseProtocol::Legacy,
            3,
            true,
            MousePosition { x: 4, y: 5 },
        );
        state.last_button = Some(MouseButton::Two);
        state.observe(&mut legacy_release, now);
        assert_eq!(legacy_release.button, Some(MouseButton::Two));
        assert_eq!(legacy_release.kind, MouseEventKind::Up);

        let mut second = event(2, 3);
        state.observe(&mut second, now + Duration::from_millis(10));
        assert_eq!(second.kind, MouseEventKind::SecondClick);
    }

    #[test]
    fn repeat_clicks_follow_tmuxs_second_then_timer_double_shape() {
        let mut state = MouseInputState::default();
        let start = Instant::now();

        let mut first = event(2, 3);
        state.observe(&mut first, start);
        assert_eq!(first.kind, MouseEventKind::Down);
        // The timer is armed but a lone press replays nothing.
        assert_eq!(state.click_deadline(), Some(start + CLICK_TIMEOUT));

        let mut second = event(2, 3);
        state.observe(&mut second, start + Duration::from_millis(50));
        assert_eq!(second.kind, MouseEventKind::SecondClick);

        // No third press: the timer delivers the DoubleClick, and it must not
        // be written into the pane a second time.
        let replayed = state
            .expire_click(start + Duration::from_millis(400))
            .expect("timer replays a double click");
        assert_eq!(replayed.kind, MouseEventKind::DoubleClick);
        assert!(replayed.ignore);
        assert_eq!(state.click_deadline(), None);
    }

    #[test]
    fn a_third_press_is_a_triple_click_and_cancels_the_timer() {
        let mut state = MouseInputState::default();
        let start = Instant::now();
        for expected in [
            MouseEventKind::Down,
            MouseEventKind::SecondClick,
            MouseEventKind::TripleClick,
        ] {
            let mut press = event(2, 3);
            state.observe(&mut press, start);
            assert_eq!(press.kind, expected);
        }
        assert_eq!(state.click_deadline(), None);
        assert_eq!(state.expire_click(start + Duration::from_secs(1)), None);
    }

    #[test]
    fn a_repeat_click_somewhere_else_restarts_the_sequence() {
        let mut state = MouseInputState::default();
        let now = Instant::now();
        let mut first = event(2, 3);
        first.target = Some(pane_target(1));
        state.observe(&mut first, now);

        let mut elsewhere = event(2, 3);
        elsewhere.target = Some(pane_target(2));
        state.observe(&mut elsewhere, now);
        assert_eq!(elsewhere.kind, MouseEventKind::Down);
    }

    fn pane_target(pane_id: u32) -> MouseTarget {
        MouseTarget {
            session_id: 0,
            window_id: Some(0),
            pane_id: Some(pane_id),
            location: MouseLocation::Pane,
            local_position: Some(MousePosition { x: 0, y: 0 }),
            status_line: None,
            status_range: None,
            border_side: None,
        }
    }
}
