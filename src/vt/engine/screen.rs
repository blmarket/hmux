//! The screen and the operations on it: tmux's `screen.c` and
//! `screen-write.c`.
//!
//! A screen is a [`Grid`] plus everything that decides where the next cell
//! lands: the cursor, the scrolling region, the modes, the tab stops. The
//! operations below are the ones `input.c` calls, ported one for one.
//!
//! tmux's `screen_write_collect_*` batching is deliberately not ported. It
//! exists to coalesce writes to a real tty; it changes *when* tmux draws, never
//! what the grid holds, and hmux composites from the grid rather than from the
//! write stream. Dropping it removes a whole class of ordering subtleties that
//! would have no observable effect here.

use std::collections::BTreeSet;

use super::cell::{colour, Cell, CellData, UTF8_SIZE};
use super::combine;
use super::grid::{colour_is_default, line_flag, Grid};
use super::hyperlinks::Hyperlinks;
use crate::vt::screen::{mode, ScreenOptions};
use crate::vt::width;

/// What [`Screen::combine`] decided about a character, which is tmux's return
/// value from `screen_write_combine` given a name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Combined {
    /// The character was folded into the cell to its left, or dropped. Either
    /// way it does not get a cell of its own.
    Consumed,
    /// The character stands alone and should be placed normally.
    NotCombined,
}

/// tmux's default history limit, the `history-limit` option's default.
pub(crate) const DEFAULT_HISTORY_LIMIT: usize = 2000;

/// The cursor and character state a DECSC saves and a DECRC restores.
#[derive(Clone, Debug)]
struct SavedState {
    cx: usize,
    cy: usize,
    cell: Cell,
    mode: u32,
}

/// One pane's screen.
pub(crate) struct Screen {
    pub(crate) grid: Grid,
    /// The alternate screen's grid while it is in use, and the primary's while
    /// it is not. tmux keeps the displaced grid in `saved_grid`.
    saved_grid: Option<Grid>,
    saved_state: Option<SavedState>,
    /// DECSC's own save, which is separate from the alternate screen's.
    saved_cursor: Option<SavedState>,
    pub(crate) cx: usize,
    pub(crate) cy: usize,
    /// The scrolling region, inclusive, in viewport rows.
    pub(crate) rupper: usize,
    pub(crate) rlower: usize,
    pub(crate) mode: u32,
    pub(crate) tabs: BTreeSet<usize>,
    /// The cell attributes the next character is written with.
    pub(crate) cell: Cell,
    /// The OSC 8 links this screen's cells point into, tmux's
    /// `screen->hyperlinks`.
    pub(crate) hyperlinks: Hyperlinks,
    /// The pane options this screen consults, as the server last resolved them.
    pub(crate) options: ScreenOptions,
}

impl Screen {
    /// tmux's `screen_init`.
    pub(crate) fn new(sx: usize, sy: usize, hlimit: usize) -> Screen {
        let sx = sx.max(1);
        let sy = sy.max(1);
        Screen {
            grid: Grid::new(sx, sy, hlimit),
            saved_grid: None,
            saved_state: None,
            saved_cursor: None,
            cx: 0,
            cy: 0,
            rupper: 0,
            rlower: sy - 1,
            mode: mode::CURSOR | mode::WRAP,
            tabs: default_tabs(sx),
            cell: Cell::default(),
            hyperlinks: Hyperlinks::default(),
            options: ScreenOptions::default(),
        }
    }

    pub(crate) fn sx(&self) -> usize {
        self.grid.sx
    }

    pub(crate) fn sy(&self) -> usize {
        self.grid.sy
    }

    /// The absolute row of viewport row `py`, tmux's `grid_view_y`.
    fn view_y(&self, py: usize) -> usize {
        self.grid.hsize + py
    }

    /// Whether this screen keeps history. The alternate screen does not.
    fn has_history(&self) -> bool {
        self.grid.hlimit != 0
    }

    // ---- cursor ----------------------------------------------------------

    /// tmux's `screen_write_set_cursor`.
    ///
    /// The column may sit one past the last one: that is the pending-wrap
    /// position a character written in the final column leaves behind, and
    /// clamping it away would lose the wrap.
    pub(crate) fn set_cursor(&mut self, cx: Option<usize>, cy: Option<usize>) {
        if let Some(cx) = cx {
            self.cx = if cx > self.sx() { self.sx() - 1 } else { cx };
        }
        if let Some(cy) = cy {
            self.cy = cy.min(self.sy() - 1);
        }
    }

    /// Apply a session's history limit to the primary grid. While the
    /// alternate screen is active, its zero-history grid is untouched and the
    /// displaced primary grid is trimmed instead.
    pub(crate) fn set_history_limit(&mut self, limit: usize) {
        if let Some(saved) = self.saved_grid.as_mut() {
            saved.set_history_limit(limit);
        } else {
            self.grid.set_history_limit(limit);
        }
    }

    /// tmux's `screen_write_cursormove`: absolute addressing, which origin mode
    /// makes relative to the scrolling region.
    pub(crate) fn cursor_move(&mut self, px: Option<usize>, py: Option<usize>, origin: bool) {
        let py = py.map(|py| {
            if origin && self.mode & mode::ORIGIN != 0 {
                if py > self.rlower - self.rupper {
                    self.rlower
                } else {
                    self.rupper + py
                }
            } else {
                py
            }
        });
        // Unlike `set_cursor`, addressing clamps to the last real column:
        // there is no way to *ask* for the pending-wrap position.
        self.set_cursor(px.map(|px| px.min(self.sx() - 1)), py);
    }

    pub(crate) fn cursor_up(&mut self, n: usize) {
        // Above the region the cursor stops at the top of the screen; inside
        // it, at the region's own top.
        let top = if self.cy < self.rupper {
            0
        } else {
            self.rupper
        };
        self.cy = self.cy.saturating_sub(n).max(top);
    }

    pub(crate) fn cursor_down(&mut self, n: usize) {
        let bottom = if self.cy > self.rlower {
            self.sy() - 1
        } else {
            self.rlower
        };
        self.cy = (self.cy + n).min(bottom);
    }

    pub(crate) fn cursor_left(&mut self, n: usize) {
        self.cx = self.cx.saturating_sub(n);
    }

    pub(crate) fn cursor_right(&mut self, n: usize) {
        self.cx = (self.cx + n).min(self.sx() - 1);
    }

    /// tmux's `screen_write_carriagereturn`.
    pub(crate) fn carriage_return(&mut self) {
        self.set_cursor(Some(0), None);
    }

    /// tmux's `screen_write_backspace`: at column zero it steps back onto the
    /// end of the previous row, but only if that row soft-wrapped into this
    /// one.
    pub(crate) fn backspace(&mut self) {
        let (mut cx, mut cy) = (self.cx, self.cy);
        if cx == 0 {
            if cy == 0 {
                return;
            }
            let wrapped = self
                .grid
                .line(self.view_y(cy - 1))
                .is_some_and(super::grid::Line::is_wrapped);
            if !wrapped {
                return;
            }
            cy -= 1;
            cx = self.sx() - 1;
        } else {
            cx -= 1;
        }
        self.set_cursor(Some(cx), Some(cy));
    }

    // ---- scrolling -------------------------------------------------------

    /// tmux's `grid_view_scroll_region_up`: rows leave the top of the region.
    /// A region that covers the whole screen feeds the history; one that does
    /// not, on a screen that has history, still promotes its top row.
    fn scroll_region_up(&mut self, bg: i32) {
        let (rupper, rlower) = (self.rupper, self.rlower);
        if self.has_history() {
            self.grid.collect_history();
            if rupper == 0 && rlower == self.sy() - 1 {
                self.grid.scroll_history(bg);
            } else {
                self.grid.scroll_history_region(rupper, rlower, bg);
            }
            return;
        }
        let upper = self.view_y(rupper);
        self.grid.move_lines(upper, upper + 1, rlower - rupper, bg);
    }

    /// tmux's `grid_view_scroll_region_down`: rows leave the bottom.
    fn scroll_region_down(&mut self, bg: i32) {
        let upper = self.view_y(self.rupper);
        let lower = self.view_y(self.rlower);
        self.grid.move_lines(upper + 1, upper, lower - upper, bg);
    }

    /// tmux's `screen_write_linefeed`. `wrapped` marks the row it leaves as
    /// soft-wrapped, which is how `capture-pane -J` rejoins it.
    pub(crate) fn linefeed(&mut self, wrapped: bool, bg: i32) {
        if wrapped {
            let row = self.view_y(self.cy);
            self.grid.set_line_flags(row, line_flag::WRAPPED, true);
        }
        if self.cy == self.rlower {
            self.scroll_region_up(bg);
        } else if self.cy < self.sy() - 1 {
            self.cy += 1;
        }
    }

    /// tmux's `screen_write_reverseindex`.
    pub(crate) fn reverse_index(&mut self, bg: i32) {
        if self.cy == self.rupper {
            self.scroll_region_down(bg);
        } else if self.cy > 0 {
            self.cy -= 1;
        }
    }

    /// tmux's `screen_write_scrollup`.
    pub(crate) fn scroll_up(&mut self, lines: usize, bg: i32) {
        let limit = self.rlower - self.rupper + 1;
        let lines = lines.max(1).min(limit);
        for _ in 0..lines {
            self.scroll_region_up(bg);
        }
    }

    /// tmux's `screen_write_scrolldown`.
    pub(crate) fn scroll_down(&mut self, lines: usize, bg: i32) {
        let limit = self.rlower - self.rupper + 1;
        let lines = lines.max(1).min(limit);
        for _ in 0..lines {
            self.scroll_region_down(bg);
        }
    }

    // ---- insert and delete -----------------------------------------------

    /// tmux's `screen_write_insertcharacter`.
    pub(crate) fn insert_character(&mut self, nx: usize, bg: i32) {
        let sx = self.sx();
        if self.cx > sx - 1 {
            return;
        }
        let nx = nx.max(1).min(sx - self.cx);
        let (px, py) = (self.cx, self.view_y(self.cy));
        if px >= sx - 1 {
            self.grid.clear(px, py, 1, 1, bg);
        } else {
            self.grid.move_cells(px + nx, px, py, sx - px - nx, bg);
        }
    }

    /// tmux's `screen_write_deletecharacter`.
    pub(crate) fn delete_character(&mut self, nx: usize, bg: i32) {
        let sx = self.sx();
        if self.cx > sx - 1 {
            return;
        }
        let nx = nx.max(1).min(sx - self.cx);
        let (px, py) = (self.cx, self.view_y(self.cy));
        self.grid.move_cells(px, px + nx, py, sx - px - nx, bg);
        self.grid.clear(sx - nx, py, nx, 1, bg);
    }

    /// tmux's `screen_write_clearcharacter`: erase in place, no shifting.
    pub(crate) fn clear_character(&mut self, nx: usize, bg: i32) {
        let sx = self.sx();
        if self.cx > sx - 1 {
            return;
        }
        let nx = nx.max(1).min(sx - self.cx);
        let py = self.view_y(self.cy);
        self.grid.clear(self.cx, py, nx, 1, bg);
    }

    /// tmux's `screen_write_insertline`. Outside the scrolling region the
    /// insert runs to the bottom of the screen instead.
    pub(crate) fn insert_line(&mut self, ny: usize, bg: i32) {
        let sy = self.sy();
        if self.cy < self.rupper || self.cy > self.rlower {
            let ny = ny.max(1).min(sy - self.cy);
            let py = self.view_y(self.cy);
            let sy_abs = self.view_y(sy);
            self.grid.move_lines(py + ny, py, sy_abs - py - ny, bg);
            return;
        }
        let ny = ny.max(1).min(self.rlower + 1 - self.cy);
        let rlower = self.view_y(self.rlower);
        let py = self.view_y(self.cy);
        // The move blanks whatever it vacated, so the clear only has work to
        // do when more lines were asked for than the region has left.
        let ny2 = (rlower + 1).saturating_sub(py + ny);
        self.grid.move_lines(rlower + 1 - ny2, py, ny2, bg);
        self.grid
            .clear(0, py + ny2, self.grid.sx, ny.saturating_sub(ny2), bg);
    }

    /// tmux's `screen_write_deleteline`.
    pub(crate) fn delete_line(&mut self, ny: usize, bg: i32) {
        let sy = self.sy();
        if self.cy < self.rupper || self.cy > self.rlower {
            let ny = ny.max(1).min(sy - self.cy);
            let py = self.view_y(self.cy);
            let sy_abs = self.view_y(sy);
            self.grid.move_lines(py, py + ny, sy_abs - py - ny, bg);
            self.grid.clear(0, sy_abs - ny, self.grid.sx, ny, bg);
            return;
        }
        let ny = ny.max(1).min(self.rlower + 1 - self.cy);
        let rlower = self.view_y(self.rlower);
        let py = self.view_y(self.cy);
        let ny2 = (rlower + 1).saturating_sub(py + ny);
        self.grid.move_lines(py, py + ny, ny2, bg);
        self.grid
            .clear(0, py + ny2, self.grid.sx, ny.saturating_sub(ny2), bg);
    }

    // ---- clearing --------------------------------------------------------

    /// tmux's `screen_write_clearline`.
    pub(crate) fn clear_line(&mut self, bg: i32) {
        let py = self.view_y(self.cy);
        self.grid.clear(0, py, self.grid.sx, 1, bg);
    }

    /// tmux's `screen_write_clearendofline`.
    pub(crate) fn clear_end_of_line(&mut self, bg: i32) {
        let py = self.view_y(self.cy);
        let sx = self.grid.sx;
        self.grid.clear(self.cx, py, sx - self.cx, 1, bg);
    }

    /// tmux's `screen_write_clearstartofline`.
    pub(crate) fn clear_start_of_line(&mut self, bg: i32) {
        let py = self.view_y(self.cy);
        let sx = self.grid.sx;
        self.grid.clear(0, py, (self.cx + 1).min(sx), 1, bg);
    }

    /// tmux's `screen_write_clearscreen`.
    pub(crate) fn clear_screen(&mut self, bg: i32) {
        if self.scrolls_on_clear() {
            self.grid.view_clear_history(bg);
            return;
        }
        let py = self.view_y(0);
        let (sx, sy) = (self.grid.sx, self.sy());
        self.grid.clear(0, py, sx, sy, bg);
    }

    /// Whether a clear should push the screen into the history rather than
    /// blank it. The alternate screen keeps no history, so there is nowhere for
    /// it to go and the option does not apply there.
    fn scrolls_on_clear(&self) -> bool {
        self.options.scroll_on_clear && self.has_history()
    }

    /// tmux's `screen_write_clearendofscreen`.
    pub(crate) fn clear_end_of_screen(&mut self, bg: i32) {
        // From the home position this erases the whole screen, so it takes the
        // same route as `clear_screen`. From anywhere else it does not, and
        // tmux checks the cursor rather than the extent.
        if self.cx == 0 && self.cy == 0 && self.scrolls_on_clear() {
            self.grid.view_clear_history(bg);
            return;
        }
        let py = self.view_y(self.cy);
        let (sx, sy) = (self.grid.sx, self.sy());
        self.grid.clear(self.cx, py, sx - self.cx, 1, bg);
        if self.cy + 1 < sy {
            self.grid.clear(0, py + 1, sx, sy - self.cy - 1, bg);
        }
    }

    /// tmux's `screen_write_clearstartofscreen`.
    pub(crate) fn clear_start_of_screen(&mut self, bg: i32) {
        let top = self.view_y(0);
        let py = self.view_y(self.cy);
        let sx = self.grid.sx;
        if self.cy > 0 {
            self.grid.clear(0, top, sx, self.cy, bg);
        }
        self.grid.clear(0, py, (self.cx + 1).min(sx), 1, bg);
    }

    /// tmux's `screen_write_clearhistory`.
    pub(crate) fn clear_history(&mut self) {
        self.grid.clear_history();
    }

    /// tmux's `screen_write_alignmenttest` (DECALN).
    pub(crate) fn alignment_test(&mut self) {
        let (sx, sy) = (self.grid.sx, self.sy());
        let cell = Cell {
            data: CellData::from_char('E', 1),
            ..Cell::default()
        };
        for py in 0..sy {
            let row = self.view_y(py);
            for px in 0..sx {
                self.grid.set(px, row, &cell);
            }
        }
        self.set_cursor(Some(0), Some(0));
        self.rupper = 0;
        self.rlower = sy - 1;
    }

    // ---- writing characters ----------------------------------------------

    /// tmux's `screen_write_cell`: place one character, wrapping, padding and
    /// overwriting as needed.
    pub(crate) fn put_cell(&mut self, cell: &Cell) {
        if cell.is_padding() {
            return;
        }
        // Every character is offered to the combining rules first, not just the
        // zero-width ones: a skin tone, a flag's second half and a Hangul jamo
        // all have a width of their own and still belong in the cell to their
        // left.
        if self.combine(cell) == Combined::Consumed {
            return;
        }
        let width = usize::from(cell.data.width);
        let sx = self.sx();
        let wrap = self.mode & mode::WRAP != 0;

        // A wide character that cannot fit and cannot wrap is dropped whole.
        if !wrap && width > 1 && (width > sx || (self.cx != sx && self.cx > sx - width)) {
            return;
        }
        if self.mode & mode::INSERT != 0 {
            self.insert_character(width, colour::DEFAULT);
        }
        if wrap && self.cx > sx - width {
            self.linefeed(true, colour::DEFAULT);
            self.set_cursor(Some(0), None);
        }
        if self.cx > sx - width || self.cy > self.sy() - 1 {
            return;
        }

        // Overwriting the left half of a wide character has to erase its
        // right half too, and vice versa, or a stale padding cell is left.
        self.overwrite(width);

        let row = self.view_y(self.cy);
        self.grid.set(self.cx, row, cell);
        for xx in self.cx + 1..self.cx + width {
            self.grid.set_padding(xx, row, cell);
        }

        // Without wrapping the cursor sticks on the last column and the next
        // character replaces this one.
        let not_wrap = usize::from(!wrap);
        if self.cx + width + not_wrap <= sx {
            self.set_cursor(Some(self.cx + width), None);
        } else {
            self.cx = sx - not_wrap;
        }
    }

    /// tmux's `screen_write_combine`: decide whether this character joins the
    /// cluster in the cell to its left, and fold it in if so.
    ///
    /// The return value is tmux's: [`Combined::Consumed`] means the character
    /// has been dealt with — folded in, or deliberately dropped — and
    /// [`Combined::NotCombined`] means it should be placed as a cell of its own.
    fn combine(&mut self, cell: &Cell) -> Combined {
        let incoming = cell.data.text();

        // tmux discards the Hangul filler outright, wherever it appears.
        if combine::is_hangul_filler(incoming) {
            return Combined::Consumed;
        }

        // A character that makes no sense on its own is flagged here and
        // dropped below if there turns out to be nothing to combine it with.
        let mut force_wide = false;
        let zero_width = if combine::is_zwj(incoming) {
            true
        } else if combine::is_vs(incoming) {
            force_wide = width::variation_selector_always_wide();
            true
        } else {
            cell.data.width == 0
        };
        let alone = if zero_width {
            Combined::Consumed
        } else {
            Combined::NotCombined
        };

        // Nothing to combine a single-byte character with, and nothing to the
        // left of column zero.
        if cell.data.bytes.len() < 2 || self.cx == 0 {
            return alone;
        }

        // Find the cell to combine with, stepping back over a padding cell onto
        // the wide character it belongs to.
        let row = self.view_y(self.cy);
        let mut n = 1;
        let mut last = self.grid.get(self.cx - n, row);
        if self.cx != 1 && last.is_padding() {
            n = 2;
            last = self.grid.get(self.cx - n, row);
        }
        if usize::from(last.data.width) != n || last.is_padding() {
            return alone;
        }

        // A character with a width of its own only combines if one of the
        // scripts' rules says it should.
        if !zero_width {
            match combine::jamo_state(last.data.text(), incoming) {
                combine::JamoState::NotComposable => return Combined::Consumed,
                combine::JamoState::Choseong => return Combined::NotCombined,
                combine::JamoState::Composable => {}
                combine::JamoState::NotHangulJamo => {
                    if combine::should_combine(last.data.text(), incoming)
                        || combine::should_combine(incoming, last.data.text())
                    {
                        force_wide = true;
                    } else if !combine::has_zwj(last.data.text()) {
                        return Combined::NotCombined;
                    }
                }
            }
        }

        // A cluster that would outgrow the cell is left to start its own.
        if last.data.bytes.len() + cell.data.bytes.len() > UTF8_SIZE {
            return Combined::NotCombined;
        }
        last.data.bytes.extend_from_slice(&cell.data.bytes);

        // A modifier or variation selector widens the cell it joined, which
        // costs the column to the right of it and moves the cursor on.
        let mut cx = self.cx;
        if last.data.width == 1 && force_wide {
            last.data.width = 2;
            n = 2;
            cx += 1;
        } else {
            force_wide = false;
        }

        self.grid.set(cx - n, row, &last);
        if force_wide {
            self.grid.set_padding(cx - 1, row, &last);
        }
        self.set_cursor(Some(cx), None);
        Combined::Consumed
    }

    /// tmux's `screen_write_overwrite`: clear the padding around a wide
    /// character the incoming one partly covers.
    fn overwrite(&mut self, width: usize) {
        let row = self.view_y(self.cy);
        let sx = self.sx();

        // Landing on the right half of a wide character: erase its left half.
        if self.grid.get(self.cx, row).is_padding() {
            let mut xx = self.cx;
            while xx > 0 {
                xx -= 1;
                if !self.grid.get(xx, row).is_padding() {
                    break;
                }
            }
            self.grid.clear(xx, row, self.cx - xx, 1, colour::DEFAULT);
        }

        // Covering the left half of a wide character: erase the padding that
        // followed it.
        for xx in self.cx..(self.cx + width).min(sx) {
            let cell = self.grid.get(xx, row);
            if xx > self.cx && !cell.is_padding() {
                break;
            }
            if usize::from(cell.data.width) > 1 || (xx > self.cx && cell.is_padding()) {
                let mut end = xx + 1;
                while end < sx && self.grid.get(end, row).is_padding() {
                    end += 1;
                }
                self.grid.clear(xx, row, end - xx, 1, colour::DEFAULT);
            }
        }
    }

    // ---- tabs ------------------------------------------------------------

    /// tmux's `screen_reset_tabs`.
    pub(crate) fn reset_tabs(&mut self) {
        self.tabs = default_tabs(self.sx());
    }

    /// tmux's HT handling in `input_c0_dispatch`: advance to the next stop,
    /// and where the run is blank record it as one tab cell rather than as
    /// spaces, so `capture-pane -e` can tell them apart.
    pub(crate) fn tab(&mut self) {
        let sx = self.sx();
        if self.cx >= sx - 1 {
            return;
        }
        let row = self.view_y(self.cy);
        let first = self.grid.get(self.cx, row);
        let mut cx = self.cx;
        let mut has_content = false;
        loop {
            if !has_content {
                let cell = self.grid.get(cx, row);
                if !cell.data.is_space() || !cell.looks_equal(&first) {
                    has_content = true;
                }
            }
            cx += 1;
            if self.tabs.contains(&cx) || cx >= sx - 1 {
                break;
            }
        }
        let width = cx - self.cx;
        if has_content || width > super::cell::UTF8_SIZE {
            self.cx = cx;
            return;
        }
        let template = self.grid.get(self.cx, row);
        let cell = Grid::tab_cell(&template, width);
        self.put_cell(&cell);
    }

    // ---- modes and regions -----------------------------------------------

    /// DECSTBM. A region that is empty or off the screen is refused, and a
    /// valid one homes the cursor, as tmux's `screen_write_scrollregion` does.
    pub(crate) fn set_scroll_region(&mut self, upper: usize, lower: usize) {
        let sy = self.sy();
        if upper >= lower || lower > sy - 1 {
            return;
        }
        self.rupper = upper;
        self.rlower = lower;
    }

    /// tmux's `screen_write_mode_set`.
    pub(crate) fn mode_set(&mut self, bits: u32) {
        self.mode |= bits;
    }

    /// tmux's `screen_write_mode_clear`.
    pub(crate) fn mode_clear(&mut self, bits: u32) {
        self.mode &= !bits;
    }

    /// DECSC: remember the cursor and the character attributes.
    pub(crate) fn save_cursor(&mut self) {
        self.saved_cursor = Some(SavedState {
            cx: self.cx,
            cy: self.cy,
            cell: self.cell.clone(),
            mode: self.mode,
        });
    }

    /// DECRC. With nothing saved the cursor goes home, as tmux's
    /// `input_restore_state` leaves it.
    pub(crate) fn restore_cursor(&mut self) {
        match self.saved_cursor.clone() {
            Some(state) => {
                self.cell = state.cell;
                self.mode = (self.mode & !mode::ORIGIN) | (state.mode & mode::ORIGIN);
                self.set_cursor(Some(state.cx), Some(state.cy));
            }
            None => {
                self.cell = Cell::default();
                self.mode &= !mode::ORIGIN;
                self.set_cursor(Some(0), Some(0));
            }
        }
    }

    /// tmux's `screen_alternate_on`. The alternate screen keeps no history, and
    /// the primary's grid is put aside untouched.
    pub(crate) fn alternate_on(&mut self, save_cursor: bool) {
        if self.saved_grid.is_some() {
            return;
        }
        if save_cursor {
            self.saved_state = Some(SavedState {
                cx: self.cx,
                cy: self.cy,
                cell: self.cell.clone(),
                mode: self.mode,
            });
        }
        let (sx, sy) = (self.grid.sx, self.grid.sy);
        self.saved_grid = Some(std::mem::replace(&mut self.grid, Grid::new(sx, sy, 0)));
        self.clear_screen(colour::DEFAULT);
    }

    /// tmux's `screen_alternate_off`.
    pub(crate) fn alternate_off(&mut self, restore_cursor: bool) {
        let Some(mut grid) = self.saved_grid.take() else {
            return;
        };
        // The pane may have been resized while the alternate screen was up.
        grid.reflow(self.grid.sx);
        let cursor = grid.hsize + self.cy.min(grid.sy - 1);
        grid.resize_y(self.grid.sy, cursor, colour::DEFAULT);
        self.grid = grid;
        if restore_cursor {
            if let Some(state) = self.saved_state.take() {
                self.cell = state.cell;
                self.set_cursor(Some(state.cx), Some(state.cy));
                return;
            }
        }
        self.saved_state = None;
        self.set_cursor(Some(self.cx), Some(self.cy));
    }

    /// The grid the alternate-screen switch displaced, or `None` when no
    /// alternate screen is in use. tmux's `saved_grid`, which `capture-pane -a`
    /// reads; its presence is also what "the alternate screen is up" means.
    pub(crate) fn saved_grid(&self) -> Option<&Grid> {
        self.saved_grid.as_ref()
    }

    /// RIS: back to a screen that has been sent nothing.
    pub(crate) fn reset(&mut self) {
        let (sx, sy) = (self.grid.sx, self.grid.sy);
        let hlimit = self.grid.hlimit;
        self.saved_grid = None;
        self.saved_state = None;
        self.saved_cursor = None;
        self.grid = Grid::new(sx, sy, hlimit);
        self.cx = 0;
        self.cy = 0;
        self.rupper = 0;
        self.rlower = sy - 1;
        self.mode = mode::CURSOR | mode::WRAP;
        self.cell = Cell::default();
        self.reset_tabs();
    }

    /// Resize the screen, as tmux's `screen_resize` does: the width first, then
    /// the height, with the tab stops laid out again against the new width.
    pub(crate) fn resize(&mut self, sx: usize, sy: usize) {
        let sx = sx.max(1);
        let sy = sy.max(1);
        if sx != self.grid.sx {
            // Rewrapping moves the cursor's row, so remember where it sits in
            // its *logical* line and find that place again afterwards.
            let py = self.view_y(self.cy);
            let (wx, wy) = self.grid.wrap_position(self.cx, py);
            self.grid.reflow(sx);
            let (px, py) = self.grid.unwrap_position(wx, wy);
            // `set_cursor`'s clamp, not addressing's: a cursor that was parked
            // past the last column stays parked past the new one.
            self.cx = if px > sx { sx - 1 } else { px };
            self.cy = py.saturating_sub(self.grid.hsize).min(self.grid.sy - 1);
            self.reset_tabs();
            if let Some(grid) = self.saved_grid.as_mut() {
                grid.reflow(sx);
            }
        }
        if sy != self.grid.sy {
            // tmux carries the cursor through a resize as an absolute row and
            // converts back at the end; a cursor that ended up inside the
            // history has nowhere to be but the top left.
            let cursor = self
                .grid
                .resize_y(sy, self.view_y(self.cy), colour::DEFAULT);
            if cursor >= self.grid.hsize {
                self.cy = cursor - self.grid.hsize;
            } else {
                self.cx = 0;
                self.cy = 0;
            }
        }
        self.rupper = 0;
        self.rlower = sy - 1;
        // `set_cursor`'s clamp again: a resize must not quietly cancel a
        // pending wrap the rewrap just carried across.
        if self.cx > sx {
            self.cx = sx - 1;
        }
        self.cy = self.cy.min(sy - 1);
    }
}

/// tmux's `screen_reset_tabs`: a stop every eight columns, skipping column 0.
fn default_tabs(sx: usize) -> BTreeSet<usize> {
    (1..)
        .map(|multiple| multiple * 8)
        .take_while(|stop| *stop < sx)
        .collect()
}

/// The background an erase uses: tmux erases to the current cell's background
/// unless that background is the default.
pub(crate) fn erase_background(cell: &Cell) -> i32 {
    if colour_is_default(cell.bg) {
        colour::DEFAULT
    } else {
        cell.bg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(screen: &Screen, py: usize) -> String {
        let row = screen.grid.hsize + py;
        (0..screen.grid.line_length(row))
            .map(|px| screen.grid.get(px, row).data.text().to_string())
            .collect()
    }

    /// One history row, counted from the oldest.
    fn history(screen: &Screen, py: usize) -> String {
        (0..screen.grid.line_length(py))
            .map(|px| screen.grid.get(px, py).data.text().to_string())
            .collect()
    }

    fn put(screen: &mut Screen, s: &str) {
        for character in s.chars() {
            let width = if character == '界' { 2 } else { 1 };
            screen.put_cell(&Cell {
                data: CellData::from_char(character, width),
                ..Cell::default()
            });
        }
    }

    #[test]
    fn text_lands_where_the_cursor_is_and_moves_it_on() {
        let mut screen = Screen::new(10, 3, 100);
        put(&mut screen, "hi");
        assert_eq!(text(&screen, 0), "hi");
        assert_eq!((screen.cx, screen.cy), (2, 0));
    }

    #[test]
    fn text_wraps_at_the_margin_and_marks_the_row() {
        let mut screen = Screen::new(4, 3, 100);
        put(&mut screen, "abcdef");
        assert_eq!(text(&screen, 0), "abcd");
        assert_eq!(text(&screen, 1), "ef");
        assert!(
            screen.grid.line(0).expect("row").is_wrapped(),
            "the first row soft-wrapped into the second"
        );
    }

    #[test]
    fn without_wrapping_the_cursor_sticks_on_the_last_column() {
        let mut screen = Screen::new(4, 3, 100);
        screen.mode_clear(mode::WRAP);
        put(&mut screen, "abcdef");
        assert_eq!(text(&screen, 0), "abcf", "each character replaces the last");
        assert_eq!(text(&screen, 1), "");
    }

    #[test]
    fn a_wide_character_takes_a_padding_cell_with_it() {
        let mut screen = Screen::new(6, 2, 100);
        put(&mut screen, "a界b");
        assert_eq!(screen.grid.get(1, 0).data.text(), "界");
        assert!(screen.grid.get(2, 0).is_padding());
        assert_eq!(screen.grid.get(3, 0).data.text(), "b");
        assert_eq!(screen.cx, 4);
    }

    #[test]
    fn overwriting_half_a_wide_character_erases_the_other_half() {
        let mut screen = Screen::new(6, 2, 100);
        put(&mut screen, "界");
        screen.set_cursor(Some(1), Some(0));
        put(&mut screen, "x");
        assert_eq!(screen.grid.get(0, 0).data.text(), " ", "left half erased");
        assert_eq!(screen.grid.get(1, 0).data.text(), "x");
    }

    #[test]
    fn a_wide_character_wraps_rather_than_splitting() {
        let mut screen = Screen::new(3, 3, 100);
        put(&mut screen, "ab界");
        assert_eq!(text(&screen, 0), "ab");
        assert_eq!(screen.grid.get(0, 1).data.text(), "界");
    }

    #[test]
    fn a_combining_mark_joins_the_cell_to_its_left() {
        let mut screen = Screen::new(6, 2, 100);
        put(&mut screen, "e");
        screen.put_cell(&Cell {
            data: CellData::from_char('\u{301}', 0),
            ..Cell::default()
        });
        assert_eq!(screen.grid.get(0, 0).data.text(), "e\u{301}");
        assert_eq!(screen.cx, 1, "a zero-width character does not advance");
    }

    #[test]
    fn a_linefeed_at_the_bottom_scrolls_into_history() {
        let mut screen = Screen::new(10, 2, 100);
        put(&mut screen, "one");
        screen.linefeed(false, colour::DEFAULT);
        screen.carriage_return();
        put(&mut screen, "two");
        screen.linefeed(false, colour::DEFAULT);
        assert_eq!(screen.grid.hsize, 1);
        assert_eq!(screen.cy, 1);
        assert_eq!(text(&screen, 0), "two");
    }

    #[test]
    fn a_scrolling_region_keeps_rows_outside_it_still() {
        let mut screen = Screen::new(10, 4, 100);
        for row in 0..4 {
            screen.set_cursor(Some(0), Some(row));
            put(&mut screen, &format!("r{row}"));
        }
        screen.set_scroll_region(1, 2);
        screen.set_cursor(Some(0), Some(2));
        screen.linefeed(false, colour::DEFAULT);
        assert_eq!(text(&screen, 0), "r0", "above the region");
        assert_eq!(text(&screen, 1), "r2", "the region scrolled");
        assert_eq!(text(&screen, 2), "");
        assert_eq!(text(&screen, 3), "r3", "below the region");
    }

    #[test]
    fn reverse_index_at_the_top_of_the_region_scrolls_it_down() {
        let mut screen = Screen::new(10, 3, 100);
        put(&mut screen, "top");
        screen.set_cursor(Some(0), Some(0));
        screen.reverse_index(colour::DEFAULT);
        assert_eq!(text(&screen, 0), "");
        assert_eq!(text(&screen, 1), "top");
    }

    #[test]
    fn backspace_steps_onto_the_previous_row_only_across_a_soft_wrap() {
        let mut screen = Screen::new(4, 3, 100);
        put(&mut screen, "abcde");
        screen.set_cursor(Some(0), Some(1));
        screen.backspace();
        assert_eq!((screen.cx, screen.cy), (3, 0), "the rows are joined");

        let mut screen = Screen::new(4, 3, 100);
        put(&mut screen, "ab");
        screen.linefeed(false, colour::DEFAULT);
        screen.carriage_return();
        screen.backspace();
        assert_eq!((screen.cx, screen.cy), (0, 1), "a hard newline is a wall");
    }

    #[test]
    fn insert_and_delete_character_shift_within_the_row() {
        let mut screen = Screen::new(8, 2, 100);
        put(&mut screen, "abcd");
        screen.set_cursor(Some(1), Some(0));
        screen.insert_character(2, colour::DEFAULT);
        assert_eq!(text(&screen, 0), "a  bcd");
        screen.delete_character(2, colour::DEFAULT);
        assert_eq!(text(&screen, 0), "abcd");
    }

    #[test]
    fn insert_and_delete_line_respect_the_region() {
        let mut screen = Screen::new(6, 4, 100);
        for row in 0..4 {
            screen.set_cursor(Some(0), Some(row));
            put(&mut screen, &format!("r{row}"));
        }
        screen.set_scroll_region(0, 2);
        screen.set_cursor(Some(0), Some(0));
        screen.delete_line(1, colour::DEFAULT);
        assert_eq!(text(&screen, 0), "r1");
        assert_eq!(text(&screen, 1), "r2");
        assert_eq!(text(&screen, 2), "");
        assert_eq!(text(&screen, 3), "r3", "outside the region");
    }

    #[test]
    fn clearing_reaches_exactly_where_the_cursor_says() {
        let mut screen = Screen::new(8, 2, 100);
        put(&mut screen, "abcdef");
        screen.set_cursor(Some(3), Some(0));
        screen.clear_end_of_line(colour::DEFAULT);
        assert_eq!(text(&screen, 0), "abc");

        let mut screen = Screen::new(8, 2, 100);
        put(&mut screen, "abcdef");
        screen.set_cursor(Some(3), Some(0));
        screen.clear_start_of_line(colour::DEFAULT);
        assert_eq!(text(&screen, 0), "    ef");
    }

    #[test]
    fn origin_mode_addresses_relative_to_the_region() {
        let mut screen = Screen::new(8, 6, 100);
        screen.set_scroll_region(2, 4);
        screen.mode_set(mode::ORIGIN);
        screen.cursor_move(Some(0), Some(0), true);
        assert_eq!(screen.cy, 2);
        screen.cursor_move(Some(0), Some(9), true);
        assert_eq!(screen.cy, 4, "clamped to the region's bottom");
    }

    #[test]
    fn clearing_the_screen_can_push_it_into_the_history() {
        let mut screen = Screen::new(10, 4, 100);
        for row in 0..3 {
            screen.cursor_move(Some(0), Some(row), false);
            put(&mut screen, &format!("row{row}"));
        }
        assert_eq!(screen.grid.hsize, 0);

        screen.clear_screen(colour::DEFAULT);
        // The three written rows move into the history; the fourth was never
        // touched, so it is blanked where it stands rather than stored.
        assert_eq!(screen.grid.hsize, 3);
        assert_eq!(history(&screen, 0), "row0");
        assert_eq!(history(&screen, 2), "row2");
        assert_eq!(text(&screen, 0), "", "the viewport is blank afterwards");
    }

    #[test]
    fn scroll_on_clear_off_blanks_the_screen_where_it_stands() {
        let mut screen = Screen::new(10, 4, 100);
        screen.options.scroll_on_clear = false;
        put(&mut screen, "kept");
        screen.clear_screen(colour::DEFAULT);
        assert_eq!(screen.grid.hsize, 0, "nothing was stored");
        assert_eq!(text(&screen, 0), "");
    }

    #[test]
    fn a_clear_from_home_takes_the_same_route_as_a_whole_screen_erase() {
        let mut screen = Screen::new(10, 4, 100);
        put(&mut screen, "here");
        screen.cursor_move(Some(0), Some(0), false);
        screen.clear_end_of_screen(colour::DEFAULT);
        assert_eq!(screen.grid.hsize, 1);
        assert_eq!(history(&screen, 0), "here");
    }

    #[test]
    fn a_clear_from_anywhere_else_stores_nothing() {
        let mut screen = Screen::new(10, 4, 100);
        put(&mut screen, "here");
        // Column one, not the home position: this erases part of the screen,
        // and tmux checks where the cursor is rather than what it covers.
        screen.cursor_move(Some(1), Some(0), false);
        screen.clear_end_of_screen(colour::DEFAULT);
        assert_eq!(screen.grid.hsize, 0);
        assert_eq!(text(&screen, 0), "h");
    }

    #[test]
    fn the_alternate_screen_is_blank_and_keeps_no_history() {
        let mut screen = Screen::new(8, 2, 100);
        put(&mut screen, "primary");
        screen.alternate_on(true);
        assert!(screen.saved_grid().is_some());
        assert_eq!(text(&screen, 0), "");
        put(&mut screen, "alt");
        screen.linefeed(false, colour::DEFAULT);
        screen.linefeed(false, colour::DEFAULT);
        assert_eq!(screen.grid.hsize, 0, "the alternate screen scrolls away");
        screen.alternate_off(true);
        assert_eq!(text(&screen, 0), "primary");
        assert_eq!((screen.cx, screen.cy), (7, 0), "the cursor came back");
    }

    #[test]
    fn a_tab_run_over_blank_cells_is_one_tab_cell() {
        let mut screen = Screen::new(20, 2, 100);
        screen.tab();
        assert_eq!(screen.cx, 8);
        let cell = screen.grid.get(0, 0);
        assert_eq!(
            cell.flags & super::super::cell::flag::TAB,
            super::super::cell::flag::TAB,
            "the run is recorded as a tab, not as spaces"
        );
    }

    #[test]
    fn a_tab_over_written_text_only_moves_the_cursor() {
        let mut screen = Screen::new(20, 2, 100);
        put(&mut screen, "abc");
        screen.set_cursor(Some(0), Some(0));
        screen.tab();
        assert_eq!(screen.cx, 8);
        assert_eq!(text(&screen, 0), "abc", "the text is not overwritten");
    }

    #[test]
    fn an_alignment_test_fills_the_screen_and_homes_the_cursor() {
        let mut screen = Screen::new(4, 2, 100);
        screen.set_cursor(Some(2), Some(1));
        screen.alignment_test();
        assert_eq!(text(&screen, 0), "EEEE");
        assert_eq!(text(&screen, 1), "EEEE");
        assert_eq!((screen.cx, screen.cy), (0, 0));
    }

    /// Checked against the oracle: tmux leaves one row in the history, shows
    /// `efgh`, and parks the cursor on the pending-wrap column.
    #[test]
    fn a_narrower_screen_rewraps_the_lines_it_split() {
        let mut screen = Screen::new(10, 4, 100);
        put(&mut screen, "abcdefgh");
        screen.resize(4, 4);
        assert_eq!(screen.grid.hsize, 1);
        assert_eq!(history(&screen, 0), "abcd");
        assert_eq!(text(&screen, 0), "efgh");
        assert!(screen.grid.line(0).expect("row").is_wrapped());
        assert_eq!((screen.cx, screen.cy), (4, 0));
    }

    #[test]
    fn a_wider_screen_rejoins_the_lines_it_had_wrapped() {
        let mut screen = Screen::new(4, 4, 100);
        put(&mut screen, "abcdefgh");
        assert_eq!(text(&screen, 0), "abcd");
        screen.resize(10, 4);
        assert_eq!(text(&screen, 0), "abcdefgh");
        assert_eq!(text(&screen, 1), "");
    }

    #[test]
    fn a_hard_newline_is_never_rejoined_by_a_rewrap() {
        let mut screen = Screen::new(4, 4, 100);
        put(&mut screen, "ab");
        screen.linefeed(false, colour::DEFAULT);
        screen.carriage_return();
        put(&mut screen, "cd");
        screen.resize(10, 4);
        assert_eq!(text(&screen, 0), "ab");
        assert_eq!(text(&screen, 1), "cd");
    }

    /// Checked against the oracle: tmux puts two rows in the history and shows
    /// `b`, with the cursor at column two.
    #[test]
    fn a_rewrap_never_splits_a_wide_character() {
        let mut screen = Screen::new(10, 3, 100);
        put(&mut screen, "a界b");
        screen.resize(2, 3);
        assert_eq!(screen.grid.hsize, 2);
        assert_eq!(history(&screen, 0), "a", "the wide character would not fit");
        assert_eq!(history(&screen, 1), "界");
        assert_eq!(text(&screen, 0), "b");
        assert_eq!((screen.cx, screen.cy), (2, 0));
    }

    /// Checked against the oracle for the widening case: tmux keeps no history
    /// and leaves the cursor at column six.
    #[test]
    fn the_cursor_follows_its_own_text_through_a_rewrap() {
        let mut screen = Screen::new(4, 4, 100);
        put(&mut screen, "abcdef");
        // The cursor sits just after `f`, the sixth character of the logical
        // line, wherever the margin happens to fall.
        screen.resize(10, 4);
        assert_eq!(screen.grid.hsize, 0);
        assert_eq!(text(&screen, 0), "abcdef");
        assert_eq!((screen.cx, screen.cy), (6, 0));
        screen.resize(3, 4);
        assert_eq!(text(&screen, 0), "def");
        assert_eq!((screen.cx, screen.cy), (3, 0));
    }

    #[test]
    fn erasing_to_a_colour_survives_where_nothing_was_written() {
        let mut screen = Screen::new(6, 2, 100);
        screen.clear_line(colour::indexed(4));
        assert_eq!(screen.grid.get(5, 0).bg, colour::indexed(4));
    }
}
