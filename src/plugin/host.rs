//! The server side of the plugin contract: pane observation over
//! `window_pane`, and the redraw a plugin asks for when its values change.
//!
//! Panes are named by id throughout. A handle resolves its pane on every call
//! and answers as if the pane were gone once it is, so plugin state can never
//! reach a destroyed pane through a pointer it kept.

use crate::WindowPane;
use crate::screen::Screen;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ffi::CString;
use std::io;
use std::rc::Rc;

use hmux_agent::observability::v1::{
    PaneId, PaneObservability, PaneProcess, ScreenSource, ScreenTail, ServerObservability,
};

use crate::grid::{GRID_LINE_WRAPPED, GRID_STRING_EMPTY_CELLS, GRID_STRING_TRIM_SPACES, Grid};
use crate::screen::{
    MODE_CURSOR, MODE_CURSOR_BLINKING, MODE_CURSOR_BLINKING_SET, RustScreen, SCREEN_CURSOR_BAR,
    SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_UNDERLINE,
};

use crate::types::{grid, u_int};
use crate::window::{RustWindowPaneWeak, pane_walk, window_pane_find_by_id};

use super::{Host, PaneId as HostPaneId};

const REVISIONS: crate::server_state::LocalField<RefCell<HashMap<u_int, u64>>> =
    crate::server_state::LocalField::new(|state| &state.plugin_revisions);

/// Record that a pane parsed some output, which is what makes its screen worth
/// reading again.
pub(crate) fn note_output(id: u_int) {
    REVISIONS.with(|revisions| {
        *revisions.borrow_mut().entry(id).or_default() += 1;
    });
}

/// Drop the revisions of panes the server no longer holds, so a long-lived
/// server does not accumulate one entry per pane it has ever run.
pub(crate) fn prune_revisions() {
    let live: HashSet<u_int> = pane_walk().map(|pane| pane.id()).collect();
    REVISIONS.with(|revisions| {
        revisions.borrow_mut().retain(|id, _| live.contains(id));
    });
}

fn revision(id: u_int) -> u64 {
    REVISIONS.with(|revisions| revisions.borrow().get(&id).copied().unwrap_or(0))
}

/// The whole server, as a plugin sees it.
#[derive(Clone, Copy)]
pub struct ServerHost;

impl ServerObservability for ServerHost {
    fn pane_ids(&self) -> io::Result<Vec<PaneId>> {
        Ok(pane_walk().map(|pane| PaneId(pane.id())).collect())
    }

    fn resolve_pane(&self, id: PaneId) -> io::Result<Option<Rc<dyn PaneObservability>>> {
        if window_pane_find_by_id(id.0).is_none() {
            return Ok(None);
        }
        Ok(Some(Rc::new(PaneView { id: id.0 })))
    }
}

impl Host for ServerHost {
    fn invalidate(&self, pane: HostPaneId) {
        unsafe {
            if let Some(pane) = window_pane_find_by_id(pane.0)
                && let Some(window) = pane.window()
            {
                window.redraw_status();
            }
        }
    }
}

/// One pane, named by id and resolved on every call.
struct PaneView {
    id: u_int,
}

impl PaneView {
    /// The pane this handle names, or an error once the server has given it
    /// up — which is the same answer a consumer would get for a pane it never
    /// knew about.
    fn pane(&self) -> io::Result<RustWindowPaneWeak> {
        window_pane_find_by_id(self.id).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "pane is no longer in the server")
        })
    }
}

impl PaneObservability for PaneView {
    fn process(&self) -> io::Result<PaneProcess> {
        let pane = self.pane()?;
        unsafe {
            let wp = pane.get().expect("resolved pane");
            Ok(PaneProcess {
                child_pid: (wp.process_id() > 0).then_some(wp.process_id() as u32),
                exited: !wp.process_active(),
            })
        }
    }

    fn output_revision(&self) -> io::Result<u64> {
        self.pane()?;
        Ok(revision(self.id))
    }

    fn screen(&self, source: ScreenSource, lines: usize) -> io::Result<ScreenTail> {
        let pane = self.pane()?;
        unsafe {
            let wp = pane.get().expect("resolved pane");
            let s = wp.base();
            let gd = RustScreen::grid(s);
            Ok(ScreenTail {
                revision: revision(self.id),
                text: screen_text(gd, source, lines),
                cursor_visible: s.mode() & MODE_CURSOR != 0,
                cursor_shape: cursor_shape(s),
            })
        }
    }

    fn scrollback_rows(&self) -> io::Result<usize> {
        let pane = self.pane()?;
        unsafe { Ok(pane.get().expect("resolved pane").base().grid().history_size() as usize) }
    }

    fn title(&self) -> io::Result<Option<CString>> {
        let pane = self.pane()?;
        unsafe {
            Ok(pane
                .get()
                .expect("resolved pane")
                .base()
                .title()
                .map(|title| title.to_owned()))
        }
    }
}

/// The DECSCUSR parameter a screen's cursor state corresponds to: `0` for the
/// terminal default, then the blinking/steady pairs for block, underline and
/// bar. The screen keeps the shape and the blink apart, which is how tmux
/// answers a DECRQM for the same thing.
fn cursor_shape(s: &RustScreen) -> u8 {
    let mode = s.mode();
    let blinking = mode & MODE_CURSOR_BLINKING != 0;
    match s.cursor_style() {
        SCREEN_CURSOR_BLOCK => {
            if blinking {
                1
            } else {
                2
            }
        }
        SCREEN_CURSOR_UNDERLINE => {
            if blinking {
                3
            } else {
                4
            }
        }
        SCREEN_CURSOR_BAR => {
            if blinking {
                5
            } else {
                6
            }
        }
        _ if mode & MODE_CURSOR_BLINKING_SET != 0 => {
            if blinking {
                1
            } else {
                2
            }
        }
        _ => 0,
    }
}

/// One physical row as plain text, with the trailing blanks a row carries
/// dropped — the same reading `capture-pane` takes of a row.
fn row_text(gd: &grid, py: u_int, trim: bool) -> CString {
    let flags = if trim {
        GRID_STRING_EMPTY_CELLS | GRID_STRING_TRIM_SPACES
    } else {
        GRID_STRING_EMPTY_CELLS
    };
    gd.string_cells(0, py, gd.width(), None, flags, None)
}

/// Whether a row runs on into the next one because the text reached the right
/// margin rather than ending there.
fn wrapped(gd: &grid, py: u_int) -> bool {
    crate::grid::grid_peek_info(gd, py).is_some_and(|gl| gl.flags & GRID_LINE_WRAPPED != 0)
}

/// The last `lines` rows of the requested slice of a pane's buffer, rendered
/// the way a reader of the pane's text expects: blank rows at the very bottom
/// are not rows anybody wrote, so they are dropped before the tail is taken.
fn screen_text(gd: &grid, source: ScreenSource, lines: usize) -> CString {
    if lines == 0 {
        return CString::default();
    }
    let total = gd.history_size().wrapping_add(gd.height());
    let floor = match source {
        ScreenSource::Visible => gd.history_size(),
        _ => 0,
    };
    let mut end = total;
    while end > floor && row_text(gd, end - 1, true).is_empty() {
        end -= 1;
    }
    if end == floor {
        return CString::default();
    }
    match source {
        ScreenSource::RecentUnwrapped => unwrapped_tail(gd, floor, end, lines),
        _ => {
            let start = end.saturating_sub(lines as u_int).max(floor);
            join_cstrings((start..end).map(|py| row_text(gd, py, true)), b'\n')
        }
    }
}

/// The last `lines` *logical* lines of `[floor, end)`, with rows split only by
/// a right-margin wrap rejoined. A rejoined row keeps its full width; only the
/// last row of a run has its trailing blanks dropped.
fn unwrapped_tail(gd: &grid, floor: u_int, end: u_int, lines: usize) -> CString {
    let mut start = end;
    let mut counted = 0usize;
    while start > floor && counted < lines {
        start -= 1;
        if start == floor || !wrapped(gd, start - 1) {
            counted += 1;
        }
    }
    let mut out = Vec::new();
    let mut current = Vec::new();
    for py in start..end {
        let continues = wrapped(gd, py) && py + 1 < end;
        current.extend_from_slice(row_text(gd, py, !continues).to_bytes());
        if !continues {
            out.push(CString::new(core::mem::take(&mut current)).expect("screen row"));
        }
    }
    if !current.is_empty() {
        out.push(CString::new(current).expect("screen row"));
    }
    join_cstrings(out, b'\n')
}

fn join_cstrings(values: impl IntoIterator<Item = CString>, separator: u8) -> CString {
    let mut out = Vec::new();
    for value in values {
        if !out.is_empty() {
            out.push(separator);
        }
        out.extend_from_slice(value.to_bytes());
    }
    CString::new(out).expect("screen text has no interior NUL")
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::PaneIdentity;
    use crate::fmt_args;

    use crate::screen::{ScreenWriteCtx, screen_write_ctx_on_screen};
    use crate::tests::test_fixtures::{Target, globals};
    use crate::types::grid_cell;

    /// A screen with some rows written into it, freed when the test ends.
    struct Written {
        screen: Box<RustScreen>,
    }

    impl Written {
        /// Write `rows` at the top of an `sx` by `sy` screen, scrolling the
        /// earlier ones into history once there are more rows than fit.
        fn new(sx: u_int, sy: u_int, rows: &[&str]) -> Written {
            let mut screen = Box::new(RustScreen::new_with_server_options(sx, sy, 100));
            let mut ctx = screen_write_ctx_on_screen(&mut screen);
            let gc = grid_cell::default();
            for (at, row) in rows.iter().enumerate() {
                if at > 0 {
                    ctx.linefeed(0, 8);
                    ctx.cursormove(0, -1, 0);
                }
                ctx.puts(
                    &gc,
                    c"%s",
                    fmt_args![std::ffi::CString::new(*row).expect("row").as_ptr()],
                );
            }
            ctx.stop();
            drop(ctx);
            Written { screen }
        }

        fn text(&self, source: ScreenSource, lines: usize) -> CString {
            screen_text(RustScreen::grid(&self.screen), source, lines)
        }
    }

    /// The tail is the last rows anybody wrote: the blank rows below the
    /// bottom of the text are not rows, so they are dropped before the tail is
    /// taken. Without that, a screen with three lines of output at the top of
    /// a 24-row terminal would read back as nothing but blanks.
    #[test]
    fn the_recent_tail_ends_at_the_last_row_with_text() {
        let _guard = globals();
        let written = Written::new(20, 6, &["first", "second", "third"]);

        assert_eq!(
            written.text(ScreenSource::Recent, 64),
            c"first\nsecond\nthird"
        );
        assert_eq!(written.text(ScreenSource::Recent, 2), c"second\nthird");
        assert_eq!(written.text(ScreenSource::Recent, 0), c"");
    }

    /// Trailing blanks on a row are padding, not text, and a row of nothing
    /// but padding still separates the rows around it.
    #[test]
    fn a_row_keeps_its_text_and_loses_its_padding() {
        let _guard = globals();
        let written = Written::new(20, 6, &["left   ", "", "after the gap"]);

        assert_eq!(
            written.text(ScreenSource::Recent, 64),
            c"left\n\nafter the gap"
        );
    }

    /// A screen nobody has written to has no tail at all, whichever slice is
    /// asked for.
    #[test]
    fn an_untouched_screen_reads_as_nothing() {
        let _guard = globals();
        let written = Written::new(20, 6, &[]);

        assert_eq!(written.text(ScreenSource::Recent, 64), c"");
        assert_eq!(written.text(ScreenSource::Visible, 64), c"");
    }

    /// Rows that scrolled off the top are still part of the recent slice, and
    /// are exactly what the visible slice leaves out.
    #[test]
    fn history_belongs_to_the_recent_slice_and_not_the_visible_one() {
        let _guard = globals();
        let rows = ["one", "two", "three", "four", "five", "six", "seven"];
        let written = Written::new(20, 3, &rows);

        assert_eq!(
            written.text(ScreenSource::Recent, 64),
            CString::new(rows.join("\n")).expect("rows")
        );
        assert_eq!(
            written.text(ScreenSource::Visible, 64),
            c"five\nsix\nseven",
            "the viewport is the last three rows"
        );
    }

    /// The cursor shape is reported as the DECSCUSR parameter that would set
    /// it, which is what a detector reading the screen compares against.
    #[test]
    fn the_cursor_shape_is_the_parameter_that_would_set_it() {
        let _guard = globals();
        let mut written = Written::new(20, 3, &["x"]);
        let s = &mut *written.screen;

        assert_eq!(cursor_shape(s), 0, "the terminal default");
        for (style, expected) in [(1, 1u8), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)] {
            s.set_mode(s.mode() & !MODE_CURSOR_BLINKING);
            s.set_cursor_style(style);
            assert_eq!(cursor_shape(s), expected, "DECSCUSR {style}");
        }
    }

    /// A pane's revision moves only when the pane produces output, and stops
    /// existing along with the pane.
    #[test]
    fn output_advances_a_panes_revision() {
        let _guard = globals();
        prune_revisions();
        let mut target = Target::new(20, 3);
        let id = unsafe { (*target.pane(0)).pane_id() };
        assert_eq!(revision(id), 0);
        note_output(id);
        note_output(id);
        prune_revisions();
        assert_eq!(revision(id), 2, "live panes survive the sweep");
        note_output(4242);
        prune_revisions();
        assert_eq!(revision(4242), 0, "unregistered panes are pruned");
        drop(target);
        prune_revisions();
        assert_eq!(revision(id), 0);
    }
}
