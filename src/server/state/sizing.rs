//! Window sizing — how a window's dimensions are derived from the clients
//! that can see it, and the scrollbar geometry that sizing has to leave room
//! for.
//!
//! [`WindowSizePolicy`] is tmux's `window-size` option; [`ClientViewport`] is
//! the window-relative rectangle one client's terminal shows when the window
//! is bigger than the terminal.

use std::collections::BTreeSet;
use std::io;

use super::layout::{resize_panes_to_layout, WINDOW_MAXIMUM, WINDOW_MINIMUM};
use crate::server::options::WindowHook;
use super::{
    ClientActionResult, PaneBorderStatus, PaneNode, PaneScrollbar, RenderInvalidation, ServerState,
    Session, ViewportClient,
};

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
pub(crate) struct ClientViewport {
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
    /// The client terminal's cell size in pixels, zero when it reports none.
    pub(super) xpixel: u16,
    pub(super) ypixel: u16,
    pub(super) size_seq: u64,
}

/// tmux's pixel fold in `clients_calculate_size`: the largest cell size any
/// candidate reports, and only from a client that is larger in *both*
/// directions — tmux compares them together, so a client with a wider cell but
/// a shorter one does not contribute either.
fn fold_client_pixels<'a>(clients: impl Iterator<Item = &'a SizingClient>) -> (u16, u16) {
    clients.fold((0, 0), |(xpixel, ypixel), client| {
        if client.xpixel > xpixel && client.ypixel > ypixel {
            (client.xpixel, client.ypixel)
        } else {
            (xpixel, ypixel)
        }
    })
}

/// tmux's `DEFAULT_XPIXEL` and `DEFAULT_YPIXEL`: the cell size a window assumes
/// when no attached client reports one.
pub(crate) const DEFAULT_XPIXEL: u16 = 16;
pub(crate) const DEFAULT_YPIXEL: u16 = 32;

/// A window size as `clients_calculate_size` derives it: the cell count and the
/// cell's pixel size, which move together.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WindowSize {
    pub(crate) size: (u16, u16),
    /// Zero in either direction when no candidate client reported a pixel size.
    pub(crate) pixels: (u16, u16),
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
        None => pane.pane.scrollback_rows(),
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

impl ServerState {
    /// Recompute how many columns each pane gives up to a scrollbar, resizing
    /// the panes of any window whose answer moved.
    ///
    /// tmux's `window_pane_show_scrollbar`: `on` shows one for every pane,
    /// `modal` only for a pane in a mode, and neither shows one for a pane on
    /// its alternate screen. `layout_fix_panes` then takes the style's width
    /// and padding off the pane.
    pub(crate) fn refresh_pane_scrollbars(&mut self) -> io::Result<()> {
        let mut changed = Vec::new();
        for (window_id, window) in &mut self.windows {
            let (mode, reserved, on_left) = {
                let options = window.options(&self.global_options);
                let mode = options.get("pane-scrollbars").unwrap_or("off").to_owned();
                let reserved = if mode == "off" {
                    0
                } else {
                    scrollbar_style_columns(options.get("pane-scrollbars-style"))
                };
                let on_left = options.get("pane-scrollbars-position") == Some("left");
                (mode, reserved, on_left)
            };
            window.scrollbars_on_left = on_left;
            let modal = mode == "modal";
            let border_status = match window
                .options(&self.global_options)
                .get("pane-border-status")
                .unwrap_or("off")
            {
                "top" => Some(PaneBorderStatus::Top),
                "bottom" => Some(PaneBorderStatus::Bottom),
                _ => None,
            };
            let window_rows = window.rows;
            let rects = window
                .panes
                .iter()
                .map(|node| window.layout.pane_rect(node.id))
                .collect::<Vec<_>>();
            let mut moved = false;
            for (node, rect) in window.panes.iter_mut().zip(rects) {
                let shown = reserved != 0
                    && (!modal || node.mode.is_some())
                    && !node.pane.alternate_screen().0;
                let columns = if shown { reserved } else { 0 };
                moved |= node.scrollbar_columns != columns;
                node.scrollbar_columns = columns;
                // Only the pane against the window's own top (or bottom) edge
                // gives up a row; the rest write on a border they already have.
                let side = rect.and_then(|rect| match border_status {
                    Some(PaneBorderStatus::Top) if rect.top == 0 => Some(PaneBorderStatus::Top),
                    Some(PaneBorderStatus::Bottom) if rect.top + rect.height >= window_rows => {
                        Some(PaneBorderStatus::Bottom)
                    }
                    _ => None,
                });
                moved |= node.border_status != side;
                node.border_status = side;
            }
            if moved {
                changed.push(*window_id);
            }
        }
        for window_id in changed {
            if let Some(window) = self.windows.get_mut(&window_id) {
                resize_panes_to_layout(window)?;
            }
            let sessions = self
                .sessions
                .iter()
                .filter(|session| session.windows.iter().any(|link| link.id == window_id))
                .map(|session| session.id)
                .collect::<Vec<_>>();
            for session_id in sessions {
                self.invalidate_session(session_id, RenderInvalidation::LAYOUT);
            }
        }
        Ok(())
    }

    /// `resize-window`: pin the target window at a manual size.
    ///
    /// tmux's `cmd_resize_window_exec` — every form starts from the window's
    /// current size, applies `-x`/`-y` then one `-L/-R/-U/-D` adjustment, lets
    /// `-a`/`-A` overwrite the result with the smallest/largest client, and
    /// finally switches the window's own `window-size` option to `manual` so the
    /// pinned size survives later client resizes. A shrink wider than the
    /// current size is a no-op rather than a clamp.
    pub fn resize_window(&mut self, target: &str, request: WindowResizeRequest) -> io::Result<()> {
        let t = self.resolve_window_target(target)?;
        let window_id = self.sessions[t.session].windows[t.window].id;
        let window = self.window(t.session, t.window);
        let mut size = (
            request.cols.unwrap_or(window.cols),
            request.rows.unwrap_or(window.rows),
        );
        let adjust = request.adjustment;
        // A shrink wider than the axis is a no-op, not a clamp to the minimum.
        match request.adjust {
            Some(WindowResizeAdjust::Left) if size.0 >= adjust => size.0 -= adjust,
            Some(WindowResizeAdjust::Right) => size.0 = size.0.saturating_add(adjust),
            Some(WindowResizeAdjust::Up) if size.1 >= adjust => size.1 -= adjust,
            Some(WindowResizeAdjust::Down) => size.1 = size.1.saturating_add(adjust),
            _ => {}
        }
        // `-a`/`-A` ignore everything computed above and take the extreme of the
        // clients that can see the window, falling back to `default-size`.
        if let Some(policy) = request.snap {
            size = self.default_window_size(t.session, Some(window_id), Some(policy));
        }
        self.window_mut(t.session, t.window)
            .option_overrides_mut()
            .set("window-size", "manual");
        self.window_mut(t.session, t.window).manual_size = size;
        // tmux resizes just this window, and immediately: `recalculate_size(w, 1)`.
        self.recalculate_window_size_now(window_id)
    }

    pub(crate) fn resize_linked_window(
        &mut self,
        target: &str,
        cols: u16,
        rows: u16,
    ) -> io::Result<()> {
        let resolved = self.resolve_window_target(target)?;
        let window_id = self.sessions[resolved.session].windows[resolved.window].id;
        let window = self.windows.get_mut(&window_id).expect("window present");
        window.manual_size = (cols, rows);
        window.option_overrides_mut().set("window-size", "manual");
        self.recalculate_sizes()
    }

    /// tmux's `default_window_size`: the size to create a window at, or the size
    /// `resize-window -a`/`-A` snaps to.
    ///
    /// `window` scopes the client set — `Some(id)` counts every client whose
    /// session links that window (the `-a`/`-A` case), `None` counts the
    /// clients of `session` itself (the window-creation case). With no
    /// qualifying client, or under `manual`, the session's `default-size`
    /// decides.
    pub(crate) fn default_window_size(
        &self,
        session: usize,
        window: Option<u32>,
        policy: Option<WindowSizePolicy>,
    ) -> (u16, u16) {
        let policy = policy.unwrap_or_else(|| {
            WindowSizePolicy::parse(self.global_options.window().get("window-size"))
        });
        let session_id = self.sessions[session].id;
        if policy != WindowSizePolicy::Manual {
            let clients = self.sizing_clients();
            let visible = clients.iter().filter(|client| match window {
                Some(id) => self.session_links_window(client.session_id, id),
                None => client.session_id == session_id,
            });
            // A window-less `latest` has no `w->latest` to prefer, so tmux lets
            // every candidate through and folds them as for `smallest`.
            if let Some(size) = fold_client_sizes(visible, policy) {
                return clamp_window_size(size);
            }
        }
        let size = self.sessions[session]
            .options(&self.global_options)
            .get("default-size")
            .and_then(parse_size_pair)
            .unwrap_or((80, 24));
        clamp_window_size(size)
    }

    /// tmux's `recalculate_sizes`: re-derive every window's size from the
    /// clients that can see it.
    ///
    /// Windows are global, so one window has one size however many sessions link
    /// it. Each window's own `window-size` picks the policy and
    /// `aggressive-resize` narrows the client set to those actually showing it.
    /// A window with no qualifying client keeps the size it has — only window
    /// *creation* falls back to `default-size`.
    pub(crate) fn recalculate_sizes(&mut self) -> io::Result<()> {
        self.recalculate_sizes_now(false)
    }

    /// [`ServerState::recalculate_sizes`], with tmux's `now` flag.
    ///
    /// Automatic sizing is deferred for a window no attached session is
    /// currently showing: the new size is recorded and applied by
    /// [`ServerState::flush_pending_window_sizes`] once the window becomes
    /// current, so a background window's panes are not resized under a program
    /// that cannot see it happen. A manually sized window, or `now`, resizes
    /// straight away.
    pub(crate) fn recalculate_sizes_now(&mut self, now: bool) -> io::Result<()> {
        let clients = self.sizing_clients();
        let window_ids = self.windows.keys().copied().collect::<Vec<_>>();
        // tmux's `server_client_update_latest`: a client only ever becomes the
        // latest client of the window it is currently showing.
        for window_id in &window_ids {
            let latest = clients
                .iter()
                .filter(|client| self.session_shows_window(client.session_id, *window_id))
                .max_by_key(|client| client.size_seq)
                .map(|client| client.size_seq);
            if let Some(latest) = latest {
                if let Some(window) = self.windows.get_mut(window_id) {
                    window.latest_client = Some(latest);
                }
            }
        }
        let mut result = Ok(());
        for window_id in window_ids {
            if let Err(error) = self.recalculate_window_size(window_id, &clients, now) {
                result = Err(error);
            }
        }
        if let Err(error) = self.flush_pending_window_sizes() {
            result = Err(error);
        }
        result
    }

    /// tmux's `recalculate_size` for a single window.
    fn recalculate_window_size(
        &mut self,
        window_id: u32,
        clients: &[SizingClient],
        now: bool,
    ) -> io::Result<()> {
        let Some(window) = self.windows.get(&window_id) else {
            return Ok(());
        };
        let options = window.options(&self.global_options);
        let policy = WindowSizePolicy::parse(options.get("window-size"));
        let aggressive = matches!(options.get("aggressive-resize"), Some("on" | "1"));
        let Some(size) = self.calculate_window_size(&window_id, policy, aggressive, clients) else {
            return Ok(());
        };
        // tmux drops a recalculation that arrived at the size the window
        // already has — but only when it is free to defer it. `now` resizes
        // straight through, which is what makes `resize-window` to the size the
        // window already has still raise `window-resized`.
        if !now && self.window_size_is_settled(window_id, size) {
            return Ok(());
        }
        if now || policy == WindowSizePolicy::Manual {
            if let Some(window) = self.windows.get_mut(&window_id) {
                window.pending_size = None;
            }
            self.apply_window_size(window_id, size)
        } else {
            if let Some(window) = self.windows.get_mut(&window_id) {
                window.pending_size = Some(size);
            }
            Ok(())
        }
    }

    /// tmux's `recalculate_size(w, 1)`: resize one window immediately, whether
    /// or not the size it arrives at is the one it already has.
    fn recalculate_window_size_now(&mut self, window_id: u32) -> io::Result<()> {
        let clients = self.sizing_clients();
        self.recalculate_window_size(window_id, &clients, true)
    }

    /// tmux's `server_client_check_window_resize`: apply a deferred size once
    /// some attached session is showing the window.
    fn flush_pending_window_sizes(&mut self) -> io::Result<()> {
        let visible = self.client_renders.with_entries(|entries| {
            entries
                .filter_map(|entry| self.current_window_of_session(entry.session_id))
                .collect::<BTreeSet<_>>()
        });
        let mut result = Ok(());
        for window_id in visible {
            let Some(size) = self
                .windows
                .get_mut(&window_id)
                .and_then(|window| window.pending_size.take())
            else {
                continue;
            };
            if let Err(error) = self.apply_window_size(window_id, size) {
                result = Err(error);
            }
        }
        result
    }

    /// tmux's `clients_calculate_size` for one window, or `None` when no client
    /// qualifies and the window should keep its current size.
    fn calculate_window_size(
        &self,
        window_id: &u32,
        policy: WindowSizePolicy,
        aggressive: bool,
        clients: &[SizingClient],
    ) -> Option<WindowSize> {
        let window = self.windows.get(window_id)?;
        if policy == WindowSizePolicy::Manual {
            // tmux skips the client loop entirely under `manual`, so no client
            // contributes a pixel size and the window falls back to the
            // default one.
            return Some(WindowSize {
                size: window.manual_size,
                pixels: (0, 0),
            });
        }
        // `latest` narrows to `w->latest` only once more than one client can see
        // the window; a lone client is folded as for `smallest`.
        let linked = clients
            .iter()
            .filter(|client| self.session_links_window(client.session_id, *window_id))
            .count();
        let candidates = clients.iter().filter(|client| {
            let visible = if aggressive {
                self.session_shows_window(client.session_id, *window_id)
            } else {
                self.session_links_window(client.session_id, *window_id)
            };
            visible
                && !(policy == WindowSizePolicy::Latest
                    && linked > 1
                    && window.latest_client != Some(client.size_seq))
        });
        // tmux folds the size and the pixel size in one pass over the same
        // candidates, so they are collected once and folded twice.
        let candidates = candidates.collect::<Vec<_>>();
        let size = fold_client_sizes(candidates.iter().copied(), policy)?;
        Some(WindowSize {
            size,
            pixels: fold_client_pixels(candidates.into_iter()),
        })
    }

    /// Whether a window already has the size a recalculation arrived at.
    ///
    /// A window with a deferred size is compared against *that* rather than
    /// against the size it currently shows, so a size scheduled and then
    /// recalculated to the same value is not rescheduled — tmux's
    /// `WINDOW_RESIZE` branch of `recalculate_size`.
    fn window_size_is_settled(&self, window_id: u32, size: WindowSize) -> bool {
        let Some(window) = self.windows.get(&window_id) else {
            return true;
        };
        let scheduled = window
            .pending_size
            .map_or((window.cols, window.rows), |pending| {
                clamp_window_size(pending.size)
            });
        scheduled == clamp_window_size(size.size)
    }

    /// Resize one window and everything laid out inside it.
    ///
    /// Unconditional, like tmux's `resize_window`: the caller decides whether a
    /// size that did not move is worth applying, because that is what settles
    /// whether `window-resized` is raised.
    fn apply_window_size(&mut self, window_id: u32, size: WindowSize) -> io::Result<()> {
        let (cols, rows) = clamp_window_size(size.size);
        let Some(window) = self.windows.get_mut(&window_id) else {
            return Ok(());
        };
        window.cols = cols;
        window.rows = rows;
        // tmux only reaches `resize_window` when the cell count moved, so the
        // pixel size rides along with it rather than being applied on its own.
        // `window_resize` substitutes the defaults for a size no client knew.
        window.xpixel = if size.pixels.0 == 0 {
            DEFAULT_XPIXEL
        } else {
            size.pixels.0
        };
        window.ypixel = if size.pixels.1 == 0 {
            DEFAULT_YPIXEL
        } else {
            size.pixels.1
        };
        window.layout.resize(cols, rows);
        let result = resize_panes_to_layout(window);
        for session_id in self.sessions_linking_window(window_id) {
            self.invalidate_session(
                session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        }
        self.notify_window(WindowHook::LayoutChanged, window_id);
        self.notify_window(WindowHook::Resized, window_id);
        result
    }

    /// The clients whose terminal size counts, with their pane area already
    /// measured — tmux's `ignore_client_size` plus `status_line_size`.
    fn sizing_clients(&self) -> Vec<SizingClient> {
        self.client_renders.with_entries(|entries| {
            // An `ignore-size` client counts only while every client is
            // flagged.
            let entries = entries.collect::<Vec<_>>();
            let any_unflagged = entries
                .iter()
                .any(|entry| entry.counts_for_sizing() && !entry.ignore_size());
            entries
                .iter()
                .filter(|entry| {
                    entry.counts_for_sizing() && !(entry.ignore_size() && any_unflagged)
                })
                .filter_map(|entry| {
                    let session = self
                        .sessions
                        .iter()
                        .find(|session| session.id == entry.session_id)?;
                    // A control client has no status line to pay for, and tmux
                    // turns the status line off rather than shrink a window to
                    // nothing on a terminal too short to hold both
                    // (`CLIENT_STATUSOFF`).
                    let status = if entry.control_mode {
                        0
                    } else {
                        self.status_lines(session)
                    };
                    let rows = if entry.rows > status {
                        entry.rows - status
                    } else {
                        entry.rows
                    };
                    Some(SizingClient {
                        session_id: entry.session_id,
                        cols: entry.cols,
                        rows,
                        xpixel: entry.xpixel,
                        ypixel: entry.ypixel,
                        size_seq: entry.size_seq,
                    })
                })
                .collect()
        })
    }

    /// The rows a session's status line occupies — tmux's `status_line_size`.
    fn status_lines(&self, session: &Session) -> u16 {
        match session.options(&self.global_options).get("status") {
            Some("off" | "0") => 0,
            Some("2") => 2,
            Some("3") => 3,
            Some("4") => 4,
            Some("5") => 5,
            _ => 1,
        }
    }

    /// The scrollbar the target's pane shows: the columns it occupies, whether
    /// they sit to the left of the pane, and the slider's row range within the
    /// pane — tmux's `screen_redraw_draw_pane_scrollbar`.
    pub(crate) fn active_pane_scrollbar(&self, target: &str) -> Option<PaneScrollbar> {
        let resolved = self.resolve(target)?;
        let window = self.window(resolved.session, resolved.window);
        let node = &window.panes[resolved.pane];
        if node.scrollbar_columns == 0 {
            return None;
        }
        let rect = window.pane_rect(node.id)?;
        if rect.height == 0 {
            return None;
        }
        let (slider_top, slider_height) = pane_slider(node, rect.height);
        Some(PaneScrollbar {
            columns: node.scrollbar_columns,
            on_left: window.scrollbars_on_left,
            left: if window.scrollbars_on_left {
                rect.left.saturating_sub(node.scrollbar_columns)
            } else {
                rect.left + rect.width
            },
            top: rect.top,
            height: rect.height,
            slider_top,
            slider_height,
        })
    }

    /// Which side `pane-border-status` puts its row on for the target's pane,
    /// when that pane is the one that gave up a row for it.
    pub(crate) fn active_pane_border_status(&self, target: &str) -> Option<PaneBorderStatus> {
        let resolved = self.resolve(target)?;
        self.window(resolved.session, resolved.window).panes[resolved.pane].border_status
    }

    /// The size of the window a target is showing, and the character
    /// `fill-character` asks for the client area outside it — tmux's
    /// `w->fill_character`, which it paints over every `CELL_OUTSIDE` cell.
    /// `None` for an unset, empty, or wider-than-one-column value, all of
    /// which `window_set_fill_character` refuses.
    pub(crate) fn window_fill(&self, target: &str) -> Option<((u16, u16), String)> {
        let resolved = self.resolve(target)?;
        let window = self.window(resolved.session, resolved.window);
        let fill = window.options(&self.global_options).get("fill-character")?;
        (crate::server::format::display_width(fill) == 1)
            .then(|| ((window.cols, window.rows), fill.to_owned()))
    }

    pub(crate) fn client_window_offset(&self, client: &ViewportClient) -> Option<ClientViewport> {
        let window_id = self.current_window_of_session(client.session_id)?;
        let window = self.windows.get(&window_id)?;
        let session = self
            .sessions
            .iter()
            .find(|session| session.id == client.session_id)?;
        let status = if client.control_mode {
            0
        } else {
            self.status_lines(session)
        };
        let (view_cols, view_rows) = (client.cols, client.rows.saturating_sub(status));
        if view_cols >= window.cols && view_rows >= window.rows {
            return Some(ClientViewport {
                bigger: false,
                ox: 0,
                oy: 0,
                sx: window.cols,
                sy: window.rows,
            });
        }
        let (sx, sy) = (view_cols, view_rows);
        // A pan survives only while it belongs to the window on screen.
        if client.pan_window == Some(window_id) {
            return Some(ClientViewport {
                bigger: true,
                ox: if sx >= window.cols {
                    0
                } else {
                    client.pan_ox.min(window.cols - sx)
                },
                oy: if sy >= window.rows {
                    0
                } else {
                    client.pan_oy.min(window.rows - sy)
                },
                sx,
                sy,
            });
        }
        // Otherwise centre the viewport on the active pane's cursor, as tmux
        // does for a window the client has never panned.
        let (mut ox, mut oy) = (0, 0);
        if let Some(pane) = window.panes.get(window.active) {
            if let Some(rect) = window.pane_rect(pane.id) {
                let (cursor_x, cursor_y) = pane.pane.cursor_position();
                let cx = rect.left.saturating_add(cursor_x);
                let cy = rect.top.saturating_add(cursor_y);
                ox = if cx < sx {
                    0
                } else if cx > window.cols.saturating_sub(sx) {
                    window.cols.saturating_sub(sx)
                } else {
                    cx - sx / 2
                };
                oy = if cy < sy {
                    0
                } else if cy > window.rows.saturating_sub(sy) {
                    window.rows.saturating_sub(sy)
                } else {
                    cy.saturating_sub(sy) + 1
                };
            }
        }
        Some(ClientViewport {
            bigger: true,
            ox,
            oy,
            sx,
            sy,
        })
    }

    /// The viewport of the client with this name, for the render path — which
    /// knows the client it is painting for by name, not by handle.
    pub(crate) fn client_viewport(&self, client_name: &str) -> Option<ClientViewport> {
        let client = self.client_renders.viewport_client(client_name)?;
        self.client_window_offset(&client)
    }

    /// `refresh-client -L/-R/-U/-D`, and `-c` to drop the pan.
    ///
    /// tmux seeds a new pan from the offset the client is already showing, so
    /// panning away from a cursor-following viewport continues from where it
    /// was rather than jumping to the window origin.
    pub(crate) fn pan_client(
        &mut self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        adjust: Option<WindowResizeAdjust>,
        adjustment: u16,
    ) -> ClientActionResult {
        let selected = target
            .and_then(|name| {
                let client = self.client_renders.viewport_client(name)?;
                Some((name.to_string(), client))
            })
            .or_else(|| {
                let tty = invoking_tty?;
                let client = self.client_renders.viewport_client(tty)?;
                Some((tty.to_string(), client))
            });
        let Some((name, client)) = selected else {
            return if target.is_some() {
                ClientActionResult::TargetNotFound
            } else {
                ClientActionResult::NoCurrentClient
            };
        };
        let Some(adjust) = adjust else {
            self.client_renders.set_client_pan(&name, None);
            self.invalidate_session(client.session_id, RenderInvalidation::LAYOUT);
            return ClientActionResult::Queued;
        };
        let Some(window_id) = self.current_window_of_session(client.session_id) else {
            return ClientActionResult::NoCurrentClient;
        };
        let Some(view) = self.client_window_offset(&client) else {
            return ClientActionResult::NoCurrentClient;
        };
        let (mut ox, mut oy) = if client.pan_window == Some(window_id) {
            (client.pan_ox, client.pan_oy)
        } else {
            (view.ox, view.oy)
        };
        let window = self.windows.get(&window_id);
        let (limit_x, limit_y) = window.map_or((0, 0), |window| {
            (
                window.cols.saturating_sub(view.sx),
                window.rows.saturating_sub(view.sy),
            )
        });
        match adjust {
            WindowResizeAdjust::Left => ox = ox.saturating_sub(adjustment),
            WindowResizeAdjust::Right => ox = ox.saturating_add(adjustment).min(limit_x),
            WindowResizeAdjust::Up => oy = oy.saturating_sub(adjustment),
            WindowResizeAdjust::Down => oy = oy.saturating_add(adjustment).min(limit_y),
        }
        self.client_renders
            .set_client_pan(&name, Some((window_id, ox, oy)));
        self.invalidate_session(client.session_id, RenderInvalidation::LAYOUT);
        ClientActionResult::Queued
    }

    /// Record the viewport a client is showing `session_name` in, then re-derive
    /// every window's size from the clients that can see it.
    ///
    /// `cols`×`rows` is the client's *pane* area (its terminal minus the status
    /// line). tmux has no session size at all — this keeps hmux's, because pane
    /// spawns, popups and overlays are still laid out against it — but the
    /// window sizes it used to set directly now come from
    /// [`ServerState::recalculate_sizes`].
    pub fn resize_session(&mut self, session_name: &str, cols: u16, rows: u16) -> io::Result<()> {
        let session = self.session_index(session_name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {session_name}"),
            )
        })?;
        self.sessions[session].cols = cols;
        self.sessions[session].rows = rows;
        self.recalculate_sizes()
    }

    /// `new-session -x/-y`: pin the session's own `default-size`.
    ///
    /// tmux writes the option onto the new session's option set before creating
    /// it, so the session's first window *and every later one* are created at
    /// that size rather than at the global default. hmux creates the session
    /// first, so the windows already made are resized to match.
    pub(crate) fn set_session_default_size(
        &mut self,
        session_name: &str,
        cols: u16,
        rows: u16,
    ) -> io::Result<()> {
        let Some(session) = self.session_index(session_name) else {
            return Ok(());
        };
        let (cols, rows) = clamp_window_size((cols, rows));
        self.sessions[session]
            .option_overrides_mut()
            .set("default-size", &format!("{cols}x{rows}"));
        self.sessions[session].cols = cols;
        self.sessions[session].rows = rows;
        for window_id in self.sessions[session]
            .windows
            .iter()
            .map(|link| link.id)
            .collect::<Vec<_>>()
        {
            if let Some(window) = self.windows.get_mut(&window_id) {
                window.manual_size = (cols, rows);
            }
            // `resize-session` pins the cell count; the pixel size is the
            // clients' to report, so it is left as it stands.
            let pixels = self
                .windows
                .get(&window_id)
                .map_or((0, 0), |window| (window.xpixel, window.ypixel));
            let size = WindowSize {
                size: (cols, rows),
                pixels,
            };
            // tmux pins `default-size` before the session's windows exist, so
            // they are simply created at this size and never resized. Standing
            // in for that after the fact must not announce a resize that tmux
            // never performed.
            if self.window_size_is_settled(window_id, size) {
                continue;
            }
            self.apply_window_size(window_id, size)?;
        }
        self.recalculate_sizes()
    }
}
