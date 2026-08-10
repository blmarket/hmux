//! The pane's screen: the grid, its scrollback, and the ways the daemon reads
//! them back out.
//!
//! Row indices are *physical*: zero is the oldest scrollback row and
//! [`GridDims::scrollback_rows`] is the first visible one. Cursor coordinates
//! are zero-based and viewport-relative.

use super::dispatch::Engine;
use super::dump;
use super::keys;
use super::screen::DEFAULT_HISTORY_LIMIT;
use crate::input::MouseEvent;
use crate::parser::Token;
use crate::screen::{CaptureExtent, Grid, GridDims, RowExtent, ScreenOptions, ScreenSnapshot};

/// hmux's own screen, and the only one: the engine is the chosen
/// implementation rather than one of several a caller picks between.
pub struct PaneScreen {
    engine: Engine,
}

impl PaneScreen {
    pub fn new(cols: u16, rows: u16) -> PaneScreen {
        PaneScreen {
            engine: Engine::new(
                usize::from(cols.max(1)),
                usize::from(rows.max(1)),
                DEFAULT_HISTORY_LIMIT,
            ),
        }
    }

    /// Apply one parsed token.
    ///
    /// The screen takes tokens rather than bytes because a live pane's stream
    /// is parsed once, upstream. This never fails: malformed input has already
    /// been resolved by the parser, and a sequence the screen does not
    /// implement is ignored, not an error.
    pub(crate) fn apply(&mut self, token: &Token) {
        self.engine.apply(&token.kind);
    }

    /// Apply a complete VT byte sequence, tokenized here. For rebuilding a
    /// grid from previously dumped output; a live pane's stream belongs to
    /// [`crate::Terminal`], whose tokenizer holds state across reads.
    pub fn apply_vt(&mut self, vt: &[u8]) {
        for token in crate::parser::tokenize(vt) {
            self.apply(&token);
        }
    }

    /// Resize the grid. Both dimensions are clamped to at least one.
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.engine
            .screen
            .resize(usize::from(cols.max(1)), usize::from(rows.max(1)));
    }

    /// Apply the pane options the screen consults; see [`ScreenOptions`].
    pub fn set_options(&mut self, options: ScreenOptions) {
        self.engine.screen.options = options;
    }

    /// The options as last pushed, for the answers that read them back — the
    /// XTWINOPS pixel reports are answered from the cell size they carry.
    pub fn options(&self) -> ScreenOptions {
        self.engine.screen.options
    }

    /// Apply a pane's history limit to the retained primary-screen scrollback.
    ///
    /// The server owns this option, while the screen owns the rows it trims;
    /// keeping the operation here avoids reconstructing a grid at the command
    /// layer.
    pub fn set_history_limit(&mut self, limit: usize) {
        self.engine.screen.set_history_limit(limit);
    }

    /// The pane's VT modes, as [`crate::screen::mode`]'s bits.
    ///
    /// Every mode a pane can set lives here, not only the ones that steer the
    /// grid, because one sequence writes a mode and one word should hold it.
    /// The server reads this rather than counting DECSETs beside the screen.
    pub fn modes(&self) -> u32 {
        self.engine.screen.mode
    }

    /// `resize-pane -T`: drop the rows below the cursor and pull the same
    /// number of rows out of the history into the viewport, so the cursor's
    /// line ends up at the bottom of the screen with its content intact.
    ///
    /// This is tmux's one operation on history that is neither a write nor a
    /// read.
    pub fn trim_history_below_cursor(&mut self) {
        let screen = &mut self.engine.screen;
        // The rows below the cursor are what goes, and there is only as much
        // history as there is to pull up in their place.
        let adjust = (screen.sy() - 1 - screen.cy).min(screen.grid.hsize);
        screen.grid.remove_history(adjust);
        screen.cy += adjust;
    }

    /// The cursor, zero-based and relative to the viewport.
    pub fn cursor_position(&self) -> (u16, u16) {
        let screen = &self.engine.screen;
        (
            u16::try_from(screen.cx).unwrap_or(u16::MAX),
            u16::try_from(screen.cy).unwrap_or(u16::MAX),
        )
    }

    /// Whether the cursor is visible (DECTCEM).
    ///
    /// A full-screen application typically hides the hardware cursor and paints
    /// its own as a styled cell. [`Self::dump_vt_rows`] carries the cursor's
    /// *position* but not this, so the compositor has to ask separately and
    /// mirror the answer onto the client tty — otherwise the client's real
    /// cursor stays lit on top of the painted one.
    pub fn cursor_visible(&self) -> bool {
        dump::cursor_visible(&self.engine.screen)
    }

    /// The DECSCUSR parameter as the pane last sent it (0 default, 1..=6
    /// block/underline/bar with blinking on the odd values), for `#{cursor_shape}`
    /// and for mirroring onto the attached tty. tmux's `s->cstyle`, kept as the
    /// raw parameter. Like [`Self::cursor_visible`], [`Self::dump_vt_rows`]
    /// does not carry it, so a compositor restoring a screen asks separately.
    pub fn cursor_style(&self) -> u8 {
        self.engine.screen.cursor_style
    }

    /// Whether the alternate screen is up (DECSET 47/1047/1049), tmux's
    /// `SCREEN_IS_ALTERNATE`.
    pub fn alternate_active(&self) -> bool {
        self.engine.screen.alternate_active()
    }

    /// The title the pane last announced (OSC 0/2 or the APC form), tmux's
    /// `screen->title` — or `None` while the pane has never set one, which is
    /// where the daemon's host-name fallback applies.
    pub fn title(&self) -> Option<String> {
        self.engine.screen.title.clone()
    }

    /// The tab stops, ascending, as `#{pane_tabs}` lists them: the default
    /// every-eight layout until the pane edits it with HTS/TBC, laid out
    /// afresh when a resize changes the width (tmux's `screen_reset_tabs`).
    pub fn tab_stops(&self) -> Vec<u16> {
        self.engine
            .screen
            .tabs
            .iter()
            .map(|stop| u16::try_from(*stop).unwrap_or(u16::MAX))
            .collect()
    }

    /// Row geometry, without walking any cells. This is also where the history
    /// count comes from: [`GridDims::scrollback_rows`].
    pub fn grid_dims(&self) -> GridDims {
        let grid = &self.engine.screen.grid;
        GridDims {
            cols: u16::try_from(grid.sx).unwrap_or(u16::MAX),
            viewport_rows: u16::try_from(grid.sy).unwrap_or(u16::MAX),
            scrollback_rows: grid.hsize,
            total_rows: grid.total(),
        }
    }

    /// The *inactive* screen — the one the alternate-screen switch displaced —
    /// materialized, or `None` when no alternate screen is in use.
    ///
    /// tmux keeps it as `saved_grid` and `capture-pane -a` is the only thing
    /// that reads it. Both forms come back together because a capture picks one
    /// of them and the grid is walked either way; splitting them would mean
    /// walking it twice or holding a cursor into a screen that has none.
    pub fn inactive_snapshot(&self) -> Option<ScreenSnapshot> {
        let screen = &self.engine.screen;
        screen.saved_grid().map(|grid| {
            let total = grid.total();
            ScreenSnapshot {
                grid: dump::snapshot_grid(screen, grid, 0, total),
                // `-a` reads the whole displaced screen; which extent the
                // capture wants is decided when its rows are serialized.
                vt: dump::vt_grid(
                    screen,
                    grid,
                    0,
                    total,
                    RowExtent::Capture(CaptureExtent::Allocated),
                ),
            }
        })
    }

    /// Snapshot physical rows `[start, start + count)`, clamped to the grid.
    /// The per-cell walk dominates the cost of a snapshot, so a consumer with a
    /// known row range pays for that range alone. The returned dimensions still
    /// describe the whole grid; `rows[0]` is physical row `start`.
    ///
    /// A whole-grid snapshot is this call over `0..grid_dims().total_rows`.
    pub fn grid_snapshot_range(&self, start: usize, count: usize) -> Grid {
        dump::snapshot(&self.engine.screen, start, count)
    }

    /// Physical rows `[start, start + rows)` as plain text, trailing whitespace
    /// trimmed. With `join_wraps`, rows split only by a right-margin soft wrap
    /// come back as one logical line.
    pub fn dump_plain_rows(&self, start: usize, rows: usize, join_wraps: bool) -> String {
        if rows == 0 {
            return String::new();
        }
        dump::plain(&self.engine.screen, start, rows, join_wraps)
    }

    /// Physical rows `[start, start + rows)` as VT escape sequences: text, SGR
    /// styles, hyperlinks and a final cursor position.
    ///
    /// The extent picks which of the two reads this is — they are not the
    /// same. A [`Capture`](RowExtent::Capture) runs to one of the row's two
    /// extents, so a space a program wrote — perhaps carrying a background
    /// colour — is part of the captured row. A [`Redraw`](RowExtent::Redraw)
    /// stops at the last non-blank cell and erases the rest, which is cheaper
    /// and is what a client tty wants. tmux keeps the same two paths apart.
    pub fn dump_vt_rows(&self, start: usize, rows: usize, extent: RowExtent) -> Vec<u8> {
        if rows == 0 {
            return Vec::new();
        }
        dump::vt(&self.engine.screen, start, rows, extent)
    }

    /// The bytes this mouse event produces, or none when the program has asked
    /// for no mouse reports.
    ///
    /// Encoding lives with the screen because it reads the modes the screen
    /// tracks — which mouse protocol the program asked for. Keys are not here:
    /// the pane's key path is the server's port of `input-keys.c`, and the
    /// keyless caller has [`crate::input::encode_key_default_modes`].
    pub fn encode_mouse(&self, mouse: MouseEvent) -> Vec<u8> {
        keys::encode_mouse(&self.engine.screen, mouse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tokenize;

    fn screen(cols: u16, rows: u16, input: &[u8]) -> PaneScreen {
        let mut screen = PaneScreen::new(cols, rows);
        for token in tokenize(input) {
            screen.apply(&token);
        }
        screen
    }

    /// The whole grid, which every caller spells as the full row range.
    fn all_rows(screen: &PaneScreen) -> usize {
        screen.grid_dims().total_rows
    }

    #[test]
    fn the_engine_answers_the_whole_seam() {
        let screen = screen(10, 3, b"hello\r\nworld");
        assert_eq!(screen.cursor_position(), (5, 1));
        assert!(screen.cursor_visible());
        let dims = screen.grid_dims();
        assert_eq!(dims.scrollback_rows, 0);
        assert_eq!((dims.cols, dims.viewport_rows, dims.total_rows), (10, 3, 3));
        let total = all_rows(&screen);
        assert!(screen.dump_plain_rows(0, total, false).contains("world"));
        assert!(!screen.dump_vt_rows(0, total, RowExtent::Redraw).is_empty());
        assert_eq!(screen.grid_snapshot_range(0, total).rows.len(), 3);
    }

    #[test]
    fn a_hidden_cursor_is_reported_hidden() {
        let screen = screen(10, 2, b"\x1b[?25l");
        assert!(!screen.cursor_visible());
    }

    #[test]
    fn a_range_snapshot_starts_at_the_row_it_was_asked_for() {
        let screen = screen(10, 2, b"a\r\nb\r\nc\r\nd");
        assert_eq!(screen.grid_dims().scrollback_rows, 2);
        let grid = screen.grid_snapshot_range(2, 2);
        assert_eq!(grid.rows.len(), 2);
        assert_eq!(grid.rows[0].cells[0].text, "c");
        assert_eq!(
            grid.scrollback_rows, 2,
            "the dimensions still describe the whole grid"
        );
    }

    #[test]
    fn resizing_rewraps_the_content_and_keeps_the_cursor_with_it() {
        let mut screen = screen(10, 3, b"abcdefgh");
        screen.resize(4, 2);
        let dims = screen.grid_dims();
        assert_eq!((dims.cols, dims.viewport_rows), (4, 2));
        assert!(screen
            .dump_plain_rows(0, dims.total_rows, false)
            .contains("abcd\nefgh"));
        // Four is the pending-wrap column of a four-column screen, which is
        // where tmux parks a cursor that has just filled the last one.
        assert_eq!(screen.cursor_position(), (4, 0));
    }

    #[test]
    fn joining_wraps_puts_a_soft_wrapped_line_back_together() {
        let screen = screen(4, 3, b"abcdefgh");
        let total = all_rows(&screen);
        assert!(screen
            .dump_plain_rows(0, total, false)
            .contains("abcd\nefgh"));
        assert!(screen.dump_plain_rows(0, total, true).contains("abcdefgh"));
    }
}
