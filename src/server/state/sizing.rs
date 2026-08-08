//! Window sizing — how a window's dimensions are derived from the clients
//! that can see it, and the scrollbar geometry that sizing has to leave room
//! for.
//!
//! [`WindowSizePolicy`] is tmux's `window-size` option; [`ClientViewport`] is
//! the window-relative rectangle one client's terminal shows when the window
//! is bigger than the terminal.

use super::layout::{WINDOW_MAXIMUM, WINDOW_MINIMUM};
use super::PaneNode;

/// How a window derives its size from the clients that can see it — the
/// `window-size` option.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowSizePolicy {
    Largest,
    Smallest,
    Manual,
    Latest,
}

impl WindowSizePolicy {
    pub(super) fn parse(value: Option<&str>) -> Self {
        match value {
            Some("largest") => Self::Largest,
            Some("smallest") => Self::Smallest,
            Some("manual") => Self::Manual,
            _ => Self::Latest,
        }
    }
}

/// Which edge `resize-window` moves.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowResizeAdjust {
    Left,
    Right,
    Up,
    Down,
}

/// One `resize-window` invocation, already parsed.
#[derive(Clone, Copy, Debug, Default)]
pub struct WindowResizeRequest {
    /// `-x`/`-y`, replacing that axis before any adjustment.
    pub cols: Option<u16>,
    pub rows: Option<u16>,
    /// `-L/-R/-U/-D`, moved by `adjustment` cells.
    pub adjust: Option<WindowResizeAdjust>,
    /// The trailing adjustment argument; tmux defaults it to 1.
    pub adjustment: u16,
    /// `-a` (smallest) or `-A` (largest), which overwrite everything else.
    pub snap: Option<WindowSizePolicy>,
}

/// Where one client's terminal sits over the window it is showing — tmux's
/// `tty->oox`/`ooy`/`osx`/`osy` and the `oflag` that reports whether the window
/// is bigger than the terminal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClientViewport {
    /// Whether the window is bigger than the client — `#{window_bigger}`.
    pub bigger: bool,
    /// The viewport's top-left corner in window coordinates.
    pub ox: u16,
    pub oy: u16,
    /// The viewport's own size.
    pub sx: u16,
    pub sy: u16,
}

/// A client whose terminal size counts toward window sizing, with its pane area
/// (terminal minus status line) already measured.
pub(super) struct SizingClient {
    pub(super) session_id: u32,
    pub(super) cols: u16,
    pub(super) rows: u16,
    pub(super) size_seq: u64,
}

/// Fold a client set into one size under `policy`, or `None` when it is empty.
pub(super) fn fold_client_sizes<'a>(
    clients: impl Iterator<Item = &'a SizingClient>,
    policy: WindowSizePolicy,
) -> Option<(u16, u16)> {
    clients.fold(None, |best, client| {
        Some(match best {
            None => (client.cols, client.rows),
            Some((cols, rows)) if policy == WindowSizePolicy::Largest => {
                (cols.max(client.cols), rows.max(client.rows))
            }
            Some((cols, rows)) => (cols.min(client.cols), rows.min(client.rows)),
        })
    })
}

pub(super) fn clamp_window_size((cols, rows): (u16, u16)) -> (u16, u16) {
    (
        cols.clamp(WINDOW_MINIMUM, WINDOW_MAXIMUM),
        rows.clamp(WINDOW_MINIMUM, WINDOW_MAXIMUM),
    )
}

/// Parse a `WxH` option value such as `default-size`.
pub(super) fn parse_size_pair(value: &str) -> Option<(u16, u16)> {
    let (cols, rows) = value.split_once('x')?;
    Some((cols.parse().ok()?, rows.parse().ok()?))
}

/// Where a pane's scrollbar slider sits and how tall it is, in rows from the
/// top of a `height`-row bar — tmux's `screen_redraw_draw_pane_scrollbar`.
///
/// Out of a mode the slider rests at the bottom, sized by how much of the
/// pane's whole history the viewport covers; in one it tracks the offset the
/// mode is showing.
pub(crate) fn pane_slider(pane: &PaneNode, height: u16) -> (u16, u16) {
    let bar = f64::from(height.max(1));
    let size = match pane.copy.as_ref() {
        Some(copy) => copy.grid.scrollback_rows,
        None => pane.pane.scrollback_rows().unwrap_or(0),
    } as f64;
    let total = size + bar;
    let slider_height = (bar * (bar / total)) as u16;
    let slider_top = match pane.copy.as_ref() {
        // The offset the mode is showing, counted from the top of history.
        Some(copy) => {
            let offset = size - copy.scroll.min(copy.grid.scrollback_rows) as f64;
            ((bar + 1.0) * (offset / total)) as u16
        }
        None => height.saturating_sub(slider_height),
    };
    (
        slider_top.min(height.saturating_sub(1)),
        slider_height.max(1),
    )
}

/// The columns `pane-scrollbars-style` asks for: its `width` plus its `pad`,
/// with tmux's floors of one and zero.
pub(super) fn scrollbar_style_columns(style: Option<&str>) -> u16 {
    let field = |name: &str, default: u16| {
        style
            .unwrap_or_default()
            .split(',')
            .filter_map(|entry| entry.trim().strip_prefix(name))
            .filter_map(|value| value.strip_prefix('='))
            .filter_map(|value| value.parse::<u16>().ok())
            .next_back()
            .unwrap_or(default)
    };
    field("width", 1).max(1) + field("pad", 0)
}
