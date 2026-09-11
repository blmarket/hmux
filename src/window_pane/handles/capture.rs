use super::*;
pub use crate::consts::{
    GRID_LINE_DEAD, GRID_LINE_EXTENDED, GRID_LINE_HYPERLINK, GRID_LINE_START_OUTPUT,
    GRID_LINE_START_PROMPT, GRID_LINE_WRAPPED, GRID_STRING_EMPTY_CELLS,
    GRID_STRING_ESCAPE_SEQUENCES, GRID_STRING_TRIM_SPACES, GRID_STRING_WITH_SEQUENCES,
};
use crate::grid::{Grid, Hyperlinks, grid_default_cell, grid_peek_info};
use crate::screen::RustScreen as screen;
use core::ffi::{c_char, c_int};

/// The hyperlink URIs line `py` carries that `links` has not seen yet, joined
/// with spaces, which is what `-H` captures in place of the line's text. Each
/// URI is listed once over the whole capture, so `links` carries the link ids
/// already written from line to line and grows to at most one screen width of
/// them; the line that would take it past that stops there.
fn cmd_capture_pane_hyperlinks(
    gd: &grid,
    s: &screen,
    py: u_int,
    links: &mut Vec<u_int>,
) -> Vec<u8> {
    let mut line = Vec::new();
    let hyperlinks = s.hyperlinks();
    let Some(gl) = grid_peek_info(gd, py) else {
        return line;
    };
    if gl.flags & GRID_LINE_HYPERLINK == 0 {
        return line;
    }
    let cellused = gl.cellused;
    let mut gc;
    for i in 0..cellused {
        gc = gd.cell(i, py);
        if gc.link == 0 || links.contains(&gc.link) {
            continue;
        }
        let Some((uri, _, _)) = hyperlinks.get(gc.link) else {
            continue;
        };
        if links.len() as u_int == gd.width() {
            break;
        }
        links.push(gc.link);
        if !line.is_empty() {
            line.push(b' ');
        }
        line.extend_from_slice(uri.to_bytes());
    }
    line
}

pub(crate) enum CapturePaneEdge {
    Dash,
    Default,
    Line(c_int),
}

/// Resolves an expanded capture edge against the selected grid. A literal
/// dash uses `dash`, invalid values use `fallback`, and negative offsets count
/// back into history. Wrapping negation preserves the `INT_MIN` boundary.
fn cmd_capture_pane_line(edge: CapturePaneEdge, gd: &grid, dash: u_int, fallback: u_int) -> u_int {
    let line = match edge {
        CapturePaneEdge::Dash => return dash,
        CapturePaneEdge::Default => fallback,
        CapturePaneEdge::Line(n) if n < 0 && n.wrapping_neg() as u_int > gd.history_size() => 0,
        CapturePaneEdge::Line(n) => gd.history_size().wrapping_add(n as u_int),
    };
    line.min(gd.history_size().wrapping_add(gd.height()).wrapping_sub(1))
}

pub(crate) struct PaneCapture {
    pub start: CapturePaneEdge,
    pub end: CapturePaneEdge,
    pub alternate: bool,
    pub mode: bool,
    pub join_lines: bool,
    pub sequences: bool,
    pub escape: bool,
    pub empty_cells: bool,
    pub trim_spaces: bool,
    pub number_lines: bool,
    pub show_flags: bool,
    pub hyperlinks: bool,
}

impl RustWindowPaneWeak {
    /// # Safety
    /// Exclude mutable access to the base screen during the query.
    pub(crate) unsafe fn has_saved_screen(&self) -> Option<bool> {
        let owner = self.upgrade()?;
        Some(unsafe { owner.as_pane().base().saved_grid().is_some() })
    }

    /// # Safety
    /// Exclude conflicting pane, screen, and mode access during the capture.
    pub(crate) unsafe fn capture_history(
        &self,
        capture: PaneCapture,
    ) -> Option<Result<Vec<u8>, ()>> {
        {
            let owner = self.upgrade()?;
            let wp = unsafe { owner.as_pane() };
            let sx = screen::grid(wp.base()).width();
            let (s, gd) = if capture.alternate {
                let Some(gd) = wp.base().saved_grid() else {
                    return Some(Err(()));
                };
                (wp.base(), gd)
            } else if capture.mode {
                if let Some(wme) = (wp).active_mode()
                    && let Some(s) = wme.mode().get_screen(wme)
                {
                    (s, screen::grid(s))
                } else {
                    (wp.base(), screen::grid(wp.base()))
                }
            } else {
                (wp.base(), screen::grid(wp.base()))
            };

            let last = gd.history_size().wrapping_add(gd.height()).wrapping_sub(1);
            let mut top = cmd_capture_pane_line(capture.start, gd, 0, gd.history_size());
            let mut bottom = cmd_capture_pane_line(capture.end, gd, last, last);
            if bottom < top {
                core::mem::swap(&mut top, &mut bottom);
            }

            let join_lines = capture.join_lines;
            let mut flags: c_int = 0;
            if capture.sequences {
                flags |= GRID_STRING_WITH_SEQUENCES;
            }
            if capture.escape {
                flags |= GRID_STRING_ESCAPE_SEQUENCES;
            }
            if !join_lines && capture.empty_cells {
                flags |= GRID_STRING_EMPTY_CELLS;
            }
            if !join_lines && capture.trim_spaces {
                flags |= GRID_STRING_TRIM_SPACES;
            }
            let number_lines = capture.number_lines;
            let show_flags = capture.show_flags;
            let hyperlinks = capture.hyperlinks;

            let mut links: Vec<u_int> = Vec::new();
            if hyperlinks {
                links.reserve(gd.width() as usize);
            }
            let mut lastgc: grid_cell = grid_default_cell;
            let mut buf: Vec<u8> = Vec::new();
            for i in top..=bottom {
                let line = if hyperlinks {
                    let line = cmd_capture_pane_hyperlinks(gd, s, i, &mut links);
                    if line.is_empty() {
                        continue;
                    }
                    line
                } else {
                    gd.string_cells(0, i, sx, Some(&mut lastgc), flags, Some(s))
                        .into_bytes()
                };
                if number_lines {
                    let n = if i >= gd.history_size() {
                        i.wrapping_sub(gd.history_size()) as c_int
                    } else {
                        i as c_int - gd.history_size() as c_int
                    };
                    buf.extend_from_slice(format!("{n} ").as_bytes());
                }
                if show_flags {
                    let gl = grid_peek_info(gd, i);
                    let mut letters: Vec<u8> = Vec::new();
                    for (bit, letter) in [
                        (GRID_LINE_DEAD, b'D'),
                        (GRID_LINE_HYPERLINK, b'H'),
                        (GRID_LINE_START_OUTPUT, b'O'),
                        (GRID_LINE_START_PROMPT, b'P'),
                        (GRID_LINE_WRAPPED, b'W'),
                        (GRID_LINE_EXTENDED, b'X'),
                    ] {
                        if gl.is_some_and(|gl| gl.flags & bit != 0) {
                            letters.push(letter);
                        }
                    }
                    if letters.is_empty() {
                        letters.push(b'-');
                    }
                    letters.push(b' ');
                    buf.extend_from_slice(&letters);
                }
                buf.extend_from_slice(&line);
                let gl = grid_peek_info(gd, i);
                if !join_lines || !gl.is_some_and(|gl| gl.flags & GRID_LINE_WRAPPED != 0) {
                    buf.push(b'\n');
                }
            }
            Some(Ok(buf))
        }
    }

    /// # Safety
    /// Exclude conflicting pane and screen access while clearing history.
    pub(crate) unsafe fn clear_history(&self, hyperlinks: bool) -> bool {
        unsafe { self.reset_modes() };
        let Some(mut owner) = self.upgrade() else {
            return false;
        };
        let pane = unsafe { owner.as_pane_mut() };
        pane.base_mut().clear_history(hyperlinks);
        true
    }
}
