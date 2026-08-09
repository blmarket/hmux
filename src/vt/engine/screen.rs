//! The screen and the operations on it: tmux's `screen.c` and
//! `screen-write.c`.
//!
//! A screen is a [`Grid`] plus everything that decides where the next cell
//! lands: the cursor, the scrolling region, the modes, the tab stops. The
//! operations below are the ones `input.c` calls, ported one for one.
//!
//! tmux's `screen_write_collect_*` batching is not ported as batching. It
//! exists to coalesce writes to a real tty, and hmux composites from the grid
//! rather than from the write stream, so the *ordering* half of it has no
//! observable effect here and dropping it removes a class of subtleties with
//! it. Cells therefore go down one at a time.
//!
//! One half of it does have to be here, because it is not about drawing at
//! all: a collected run reaches the grid through `grid_set_cells`, which
//! expands the row once for the whole run, and `grid_expand_line` rounds that
//! request up. A row written a cell at a time stops at the first rounding step
//! and never allocates the rest — and `capture-pane` reads a row to its
//! allocated extent, so the difference is visible in the captured bytes. See
//! [`Run`], which carries just enough of the batching to get the extent right.

use std::collections::BTreeSet;

use super::cell::{attr, colour, flag, Cell, CellData, UTF8_SIZE};
use super::combine;
use super::grid::{colour_is_default, line_flag, needs_extended, Grid, Line};
use super::hyperlinks::Hyperlinks;
use super::images::Images;
use crate::vt::screen::{mode, ScreenOptions};
use crate::vt::sixel::SixelImage;
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

/// The run of plain characters currently being written, tmux's
/// `screen_write_citem`.
///
/// tmux collects consecutive plain characters and puts them on the grid in one
/// `grid_set_cells`, which expands the row once for the whole run. That is the
/// one part of `screen_write_collect_*` a grid-only port cannot skip: the
/// rounding `grid_expand_line` applies is computed from the length of the whole
/// run, so a row written a cell at a time stops short of the step the run
/// would have reached, and `capture-pane -e` reads to exactly that extent.
///
/// Only the extent is deferred here. The cells themselves still go down one at
/// a time, because nothing between them can observe the difference.
#[derive(Clone, Copy)]
struct Run {
    /// The grid row, so a wrap or a cursor move starts a new run.
    row: usize,
    /// The column the run has reached, tmux's `ci->x + ci->used`.
    end: usize,
    /// The row's allocated extent when the run opened. `grid_expand_line`
    /// compares against the extent it finds, and for a run it finds this one.
    opened_at: usize,
}

/// One pane's screen.
pub(crate) struct Screen {
    pub(crate) grid: Grid,
    /// The primary screen's visible rows while the alternate screen is up, and
    /// `None` while it is not. tmux's `saved_grid`, which holds those rows only
    /// — the grid itself never changes hands, so the history stays under the
    /// alternate screen where a copy-mode scroll can still reach it.
    saved_grid: Option<Grid>,
    /// tmux's `saved_flags`: whether the grid was keeping history before the
    /// alternate screen took it away.
    saved_history: bool,
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
    /// The sixel images anchored to this screen's cells, tmux's `s->images`.
    pub(crate) images: Images,
    /// The primary screen's images while the alternate screen is up, tmux's
    /// `s->saved_images`. Unlike the grid, they are handed over rather than
    /// copied: an image is not made of cells and has nothing to duplicate.
    saved_images: Images,
    /// The pane options this screen consults, as the server last resolved them.
    pub(crate) options: ScreenOptions,
    /// The run of plain characters in progress; see [`Run`].
    run: Option<Run>,
}

impl Screen {
    /// tmux's `screen_init`.
    pub(crate) fn new(sx: usize, sy: usize, hlimit: usize) -> Screen {
        let sx = sx.max(1);
        let sy = sy.max(1);
        Screen {
            grid: Grid::new(sx, sy, hlimit),
            saved_grid: None,
            saved_history: hlimit != 0,
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
            images: Images::default(),
            saved_images: Images::default(),
            options: ScreenOptions::default(),
            run: None,
        }
    }

    /// End the run of plain characters, tmux's `screen_write_collect_end`.
    ///
    /// Anything that is not another plain character in the next column closes
    /// the run, because tmux writes it to the grid at that point and the next
    /// run's expansion starts from the extent this one left.
    pub(crate) fn end_run(&mut self) {
        self.run = None;
    }

    /// Record that a run of plain characters has reached column `end` on `row`,
    /// and give the row the allocated extent `grid_expand_line` would have
    /// given it for the run so far.
    ///
    /// Applying this on every character rather than once at the end reaches the
    /// same extent — the rounding never shrinks as the run grows — and saves
    /// having to find a moment to flush.
    fn extend_run(&mut self, row: usize, end: usize) {
        let opened_at = match self.run {
            Some(run) if run.row == row && run.end + 1 == end => run.opened_at,
            _ => self.grid.line(row).map_or(0, Line::size),
        };
        self.run = Some(Run {
            row,
            end,
            opened_at,
        });
        if end > opened_at {
            let extent = self.grid.rounded_extent(end);
            self.grid.grow_line(row, extent, colour::DEFAULT);
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

    /// Whether this screen keeps history, tmux's `GRID_HISTORY`. The alternate
    /// screen does not.
    fn has_history(&self) -> bool {
        self.grid.history && self.grid.hlimit != 0
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

    /// Apply a session's history limit. There is one grid to apply it to: the
    /// alternate screen borrows the primary's rather than bringing its own.
    pub(crate) fn set_history_limit(&mut self, limit: usize) {
        self.grid.set_history_limit(limit);
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
        self.leave_pending_wrap();
    }

    pub(crate) fn cursor_down(&mut self, n: usize) {
        let bottom = if self.cy > self.rlower {
            self.sy() - 1
        } else {
            self.rlower
        };
        self.cy = (self.cy + n).min(bottom);
        self.leave_pending_wrap();
    }

    /// Step off the pending-wrap column, which is one past the last real one.
    ///
    /// Writing into the last column leaves the cursor there rather than on the
    /// next row, so the next character wraps. tmux's `screen_write_cursorup`
    /// and `screen_write_cursordown` are the two moves that resolve that
    /// position instead of carrying it: they pull the cursor back onto the last
    /// real column, which is what `#{cursor_x}` then reports. Every other move
    /// either lands on a column of its own or clamps in `set_cursor`.
    fn leave_pending_wrap(&mut self) {
        if self.cx == self.sx() {
            self.cx -= 1;
        }
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
        self.scroll_bounds_up(rupper, rlower, bg);
    }

    /// The same, against a region given rather than the screen's own.
    ///
    /// Writing an image scrolls the *whole* screen to make room for it, which
    /// tmux does by calling `grid_view_scroll_region_up` with the screen's
    /// bounds instead of the scrolling region's.
    fn scroll_bounds_up(&mut self, rupper: usize, rlower: usize, bg: i32) {
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
            // A region that reaches the bottom of the screen carries the
            // images up with it; one that does not cannot, so tmux throws away
            // every image the region crosses instead.
            if self.rlower == self.sy() - 1 {
                self.images.scroll_up(1);
            } else {
                let (rupper, rlower) = (self.rupper, self.rlower);
                self.images.check_line(rupper, rlower - rupper);
            }
            self.scroll_region_up(bg);
        } else if self.cy < self.sy() - 1 {
            self.cy += 1;
        }
    }

    /// tmux's `screen_write_reverseindex`.
    pub(crate) fn reverse_index(&mut self, bg: i32) {
        if self.cy == self.rupper {
            self.images.free_all();
            self.scroll_region_down(bg);
        } else if self.cy > 0 {
            self.cy -= 1;
        }
    }

    /// tmux's `screen_write_scrollup`.
    pub(crate) fn scroll_up(&mut self, lines: usize, bg: i32) {
        let limit = self.rlower - self.rupper + 1;
        let lines = lines.max(1).min(limit);
        self.images.scroll_up(lines);
        for _ in 0..lines {
            self.scroll_region_up(bg);
        }
    }

    /// tmux's `screen_write_scrolldown`.
    pub(crate) fn scroll_down(&mut self, lines: usize, bg: i32) {
        let limit = self.rlower - self.rupper + 1;
        let lines = lines.max(1).min(limit);
        // Nothing moves an image *down*, so every one of them goes.
        self.images.free_all();
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
        let cy = self.cy;
        self.images.check_line(cy, 1);
        self.insert_cells(nx, bg);
    }

    /// tmux's `grid_view_insert_cells`, without the image check its
    /// `screen_write_insertcharacter` caller makes.
    ///
    /// Insert mode's own shift reaches the grid through here rather than
    /// through [`Screen::insert_character`], as tmux's `screen_write_cell`
    /// calls `grid_view_insert_cells` itself; see the note at that call site.
    fn insert_cells(&mut self, nx: usize, bg: i32) {
        let sx = self.sx();
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
        let cy = self.cy;
        self.images.check_line(cy, 1);
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
        let cy = self.cy;
        self.images.check_line(cy, 1);
        let py = self.view_y(self.cy);
        self.grid.clear(self.cx, py, nx, 1, bg);
    }

    /// tmux's `screen_write_insertline`. Outside the scrolling region the
    /// insert runs to the bottom of the screen instead.
    pub(crate) fn insert_line(&mut self, ny: usize, bg: i32) {
        let sy = self.sy();
        let cy = self.cy;
        self.images.check_line(cy, sy - cy);
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
        let cy = self.cy;
        self.images.check_line(cy, sy - cy);
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

    /// The row's allocated extent, which the clearing operations test before
    /// they decide an image is in the way.
    fn line_size(&self) -> usize {
        let row = self.view_y(self.cy);
        self.grid.line(row).map_or(0, Line::size)
    }

    /// tmux's `screen_write_clearline`.
    pub(crate) fn clear_line(&mut self, bg: i32) {
        // tmux leaves an untouched row alone entirely — including the images
        // over it — when the erase has no colour to leave behind either.
        if self.line_size() != 0 || !colour_is_default(bg) {
            let cy = self.cy;
            self.images.check_line(cy, 1);
        }
        let py = self.view_y(self.cy);
        self.grid.clear(0, py, self.grid.sx, 1, bg);
    }

    /// tmux's `screen_write_clearendofline`.
    pub(crate) fn clear_end_of_line(&mut self, bg: i32) {
        let sx = self.grid.sx;
        if self.cx == 0 {
            self.clear_line(bg);
            return;
        }
        // tmux's `s->cx > sx - 1 || (s->cx >= gl->cellsize && COLOUR_DEFAULT(bg))`
        // return, read the other way round.
        if self.cx < sx && (self.cx < self.line_size() || !colour_is_default(bg)) {
            let cy = self.cy;
            self.images.check_line(cy, 1);
        }
        let py = self.view_y(self.cy);
        self.grid.clear(self.cx, py, sx - self.cx, 1, bg);
    }

    /// tmux's `screen_write_clearstartofline`.
    pub(crate) fn clear_start_of_line(&mut self, bg: i32) {
        let sx = self.grid.sx;
        if self.cx >= sx - 1 {
            self.clear_line(bg);
            return;
        }
        let cy = self.cy;
        self.images.check_line(cy, 1);
        let py = self.view_y(self.cy);
        self.grid.clear(0, py, (self.cx + 1).min(sx), 1, bg);
    }

    /// tmux's `screen_write_clearscreen`.
    pub(crate) fn clear_screen(&mut self, bg: i32) {
        self.images.free_all();
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
        let (cy, sy) = (self.cy, self.sy());
        self.images.check_line(cy, sy - cy);
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
        // `s->cy - 1` in tmux, which on the first row underflows to the whole
        // unsigned range and takes every image on the screen with it. The
        // wrapping is the behaviour, not an accident to be tidied away.
        let cy = self.cy;
        self.images.check_line(0, cy.wrapping_sub(1));
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
        self.images.free_all();
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

    // ---- images ----------------------------------------------------------

    /// tmux's `screen_write_sixelimage`: put an image on the screen at the
    /// cursor, making room for it first.
    ///
    /// An image that does not fit is cropped from the *top*, which is what a
    /// terminal does when something too tall arrives: the bottom of the image
    /// is the part that stays. An image that fits but has no room below the
    /// cursor scrolls the whole screen up instead — the whole screen, not the
    /// scrolling region, because an image is not part of the region's contents.
    pub(crate) fn sixel_image(&mut self, image: SixelImage, bg: i32) {
        // Nowhere to put an image on a one-row screen: the cursor's row is the
        // only row, and the image would have to start below it.
        if self.sy() == 1 {
            return;
        }

        let mut image = image;
        let (x, mut y) = cells(&image);
        let (cx, cy) = (self.cx, self.cy);
        if x > self.sx() || y > self.sy() - 1 {
            let sx = if x > self.sx().saturating_sub(cx) {
                self.sx().saturating_sub(cx)
            } else {
                x
            };
            let sy = if y > self.sy() - 1 { self.sy() - 1 } else { y };
            let Some(cropped) = image.scale(0, 0, 0, (y - sy) as u32, sx as u32, sy as u32, true)
            else {
                return;
            };
            image = cropped;
            // tmux re-measures both dimensions; only the height is read again,
            // to decide how far the screen has to scroll.
            y = cells(&image).1;
        }

        let below = self.sy() - cy;
        if below <= y {
            let lines = y - below + 1;
            self.images.scroll_up(lines);
            let bottom = self.sy() - 1;
            for _ in 0..lines {
                self.scroll_bounds_up(0, bottom, bg);
            }
            let row = cy.saturating_sub(lines);
            self.set_cursor(None, Some(row));
        }

        let (px, py) = (self.cx, self.cy);
        self.images.store(image, px, py);

        // tmux moves the cursor using the row the cursor was on *before* the
        // scroll, which lands past the last row and is clamped onto it — the
        // same row `cy - lines + y` names.
        self.cursor_move(Some(0), Some(cy + y), false);
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

        // tmux's `screen_write_collect_add` takes only plain characters into a
        // run; anything else it hands straight to `screen_write_cell`, which
        // expands the row for that one cell.
        let collected = self.collects(cell);
        if !collected {
            self.end_run();
        }

        // A wide character that cannot fit and cannot wrap is dropped whole.
        if !wrap && width > 1 && (width > sx || (self.cx != sx && self.cx > sx - width)) {
            return;
        }
        if self.mode & mode::INSERT != 0 {
            // tmux shifts the row with `grid_view_insert_cells` here rather
            // than calling `screen_write_insertcharacter`, so insert mode skips
            // that function's image check: a character written in insert mode
            // lands on top of an image without taking it away.
            self.insert_cells(width, colour::DEFAULT);
        }
        if wrap && self.cx > sx - width {
            // The run is written out before the wrap, so the row it lands on is
            // the one it was collected against.
            self.end_run();
            self.linefeed(true, colour::DEFAULT);
            self.set_cursor(Some(0), None);
        }
        if self.cx > sx - width || self.cy > self.sy() - 1 {
            return;
        }

        // Overwriting the left half of a wide character has to erase its
        // right half too, and vice versa, or a stale padding cell is left.
        // The two write paths clean up the padding around them differently, so
        // each gets its own: a run through `screen_write_collect_end`, a single
        // character through `screen_write_overwrite`.
        let erased = if collected {
            self.collect_padding_before();
            false
        } else {
            self.overwrite(width)
        };

        // tmux's `skip`. A write that would leave the cell exactly as it found
        // it does not reach the grid at all — and only the uncollected path
        // asks: a run goes down through `grid_set_cells`, which always writes.
        let row = self.view_y(self.cy);
        let skip = !collected
            && !erased
            && width == 1
            && self.mode & mode::INSERT == 0
            && self.writing_changes_nothing(row, cell);
        if !skip {
            // The run claims the row's extent before the cells go down, as
            // tmux's `grid_set_cells` expands ahead of the write it will make.
            if collected {
                // tmux checks the images once for the whole collected run,
                // which is the same set of images a per-cell check over that
                // run frees: the run is contiguous and stays on one row. Only
                // collected cells check at all — a wide character or one
                // written in insert mode goes down through `screen_write_cell`,
                // which has no image hook.
                let (cx, cy) = (self.cx, self.cy);
                self.images.check_area(cx, cy, width, 1);
                self.extend_run(row, self.cx + width);
            }
            self.grid.set(self.cx, row, cell);
            for xx in self.cx + 1..self.cx + width {
                self.grid.set_padding(xx, row);
            }
        }

        // Without wrapping the cursor sticks on the last column and the next
        // character replaces this one.
        let not_wrap = usize::from(!wrap);
        if self.cx + width + not_wrap <= sx {
            self.set_cursor(Some(self.cx + width), None);
        } else {
            self.cx = sx - not_wrap;
        }

        // A run cleans up after itself: `screen_write_collect_end` clears the
        // padding left beyond the cells it has just written, so a wide
        // character the run half-covered does not leave its right half behind.
        // tmux does this once when the run flushes; doing it as each character
        // lands reaches the same grid, because the run is contiguous and only
        // ever grows to the right.
        if collected {
            let row = self.view_y(self.cy);
            let mut xx = self.cx;
            while xx < sx && self.grid.get(xx, row).is_padding() {
                self.grid.clear(xx, row, 1, 1, colour::DEFAULT);
                xx += 1;
            }
        }
    }

    /// tmux's "if no change, do not draw" test in `screen_write_cell`.
    ///
    /// Past the row's allocated extent there is no cell to compare against, so
    /// the question becomes whether writing one would show anything at all.
    /// That is what leaves a row of plain spaces unallocated, and it is
    /// observable: `capture-pane` reads a row to its allocated extent.
    ///
    /// Inside the extent tmux compares against the *packed* cell, so anything
    /// rich enough to be stored out of line never matches. The cleared flag is
    /// part of the comparison, which makes a space written over a cell an
    /// erase produced a change even though the two look identical.
    fn writing_changes_nothing(&self, row: usize, cell: &Cell) -> bool {
        let Some(existing) = self.grid.peek(self.cx, row) else {
            return cell.equals(&Cell::default());
        };
        !needs_extended(existing)
            && cell.flags == existing.flags
            && cell.attr == existing.attr
            && cell.fg == existing.fg
            && cell.bg == existing.bg
            && cell.data.width == 1
            && cell.data.bytes.len() == 1
            && existing.data.bytes == cell.data.bytes
    }

    /// Whether this character joins a run, tmux's test at the top of
    /// `screen_write_collect_add`.
    ///
    /// A run is plain single-byte ASCII in the ordinary writing mode. tmux also
    /// refuses to collect while a selection is up, which has no counterpart
    /// here: hmux's selection lives in the server's copy mode, not in the
    /// screen, and never reaches this path.
    fn collects(&self, cell: &Cell) -> bool {
        cell.data.width == 1
            && cell.data.bytes.len() == 1
            && cell.data.bytes[0] < 0x7f
            && cell.flags & flag::TAB == 0
            && cell.attr & attr::CHARSET == 0
            && self.mode & mode::WRAP != 0
            && self.mode & mode::INSERT == 0
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
            self.grid.set_padding(cx - 1, row);
        }
        self.set_cursor(Some(cx), None);
        Combined::Consumed
    }

    /// tmux's padding cleanup on the left of a collected run, from
    /// `screen_write_collect_end`.
    ///
    /// It is deliberately not [`Screen::overwrite`]'s. Both walk back over the
    /// padding to the character it belongs to, but this one clears that
    /// character only when it is itself wide or padding — so a run landing
    /// immediately after an ordinary single-column character leaves it standing.
    fn collect_padding_before(&mut self) {
        if self.cx == 0 {
            return;
        }
        let row = self.view_y(self.cy);
        let mut xx = self.cx;
        let mut cell = self.grid.get(xx, row);
        while xx > 0 {
            cell = self.grid.get(xx, row);
            if !cell.is_padding() {
                break;
            }
            self.grid.clear(xx, row, 1, 1, colour::DEFAULT);
            xx -= 1;
        }
        if xx == self.cx {
            return;
        }
        // tmux re-reads the cell when the walk ran off the left of the row,
        // because the loop left `gc` holding column one.
        if xx == 0 {
            cell = self.grid.get(0, row);
        }
        if usize::from(cell.data.width) > 1 || cell.is_padding() {
            self.grid.clear(xx, row, 1, 1, colour::DEFAULT);
        }
    }

    /// tmux's `screen_write_overwrite`: clear the padding around a wide
    /// character the incoming one partly covers.
    ///
    /// This is the single-character path only; a run of plain characters is
    /// cleaned up by [`Screen::collect_padding_before`] instead, which is where
    /// tmux puts it and which decides differently.
    ///
    /// The answer is whether anything was erased, which is what tells the
    /// caller the write is not a no-op after all.
    fn overwrite(&mut self, width: usize) -> bool {
        let row = self.view_y(self.cy);
        let sx = self.sx();
        // The cell as it was found. Both decisions below are made about it, so
        // it is read once, before the first of them writes over it.
        let now = self.grid.get(self.cx, row);
        let mut erased = false;

        // Landing on the right half of a wide character: erase the padding
        // back to the character it belongs to, and that character with it.
        if now.is_padding() {
            let mut xx = self.cx;
            while xx > 0 && self.grid.get(xx, row).is_padding() {
                self.grid.clear(xx, row, 1, 1, colour::DEFAULT);
                xx -= 1;
            }
            self.grid.clear(xx, row, 1, 1, colour::DEFAULT);
            erased = true;
        }

        // Covering the left half of a wide character: erase the padding that
        // followed it.
        if width != 1 || usize::from(now.data.width) != 1 || now.is_padding() {
            let mut xx = self.cx + width;
            while xx < sx && self.grid.get(xx, row).is_padding() {
                if now.flags & flag::TAB != 0 {
                    // A tab is not erased by writing over its first column:
                    // tmux turns each column it still covers into a tab cell
                    // of its own, so what is left of the run is still captured
                    // as tabs rather than as blanks.
                    let mut cell = now.clone();
                    cell.data = CellData {
                        bytes: b" ".to_vec(),
                        width: 1,
                    };
                    self.grid.set(xx, row, &cell);
                } else {
                    self.grid.clear(xx, row, 1, 1, colour::DEFAULT);
                }
                erased = true;
                xx += 1;
            }
        }
        erased
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

    /// DECSTBM, tmux's `screen_write_scrollregion`. Both edges are pulled onto
    /// the screen first, so a bottom past the last row keeps the region rather
    /// than refusing it; only a region that has collapsed to one line or
    /// inverted is refused. The answer is whether it took, which is what tells
    /// the caller to home the cursor.
    pub(crate) fn set_scroll_region(&mut self, upper: usize, lower: usize) -> bool {
        let last = self.sy() - 1;
        let (upper, lower) = (upper.min(last), lower.min(last));
        if upper >= lower {
            return false;
        }
        self.rupper = upper;
        self.rlower = lower;
        true
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
            // tmux restores through `screen_write_cursormove`, which clamps to
            // the last real column — so a cursor saved in the pending-wrap
            // position comes back one column short of where it was.
            Some(state) => {
                self.cell = state.cell;
                self.mode = (self.mode & !mode::ORIGIN) | (state.mode & mode::ORIGIN);
                self.cursor_move(Some(state.cx), Some(state.cy), false);
            }
            None => {
                self.cell = Cell::default();
                self.mode &= !mode::ORIGIN;
                self.cursor_move(Some(0), Some(0), false);
            }
        }
    }

    /// tmux's `screen_alternate_on`.
    ///
    /// The grid does not change hands. The visible rows are copied aside and
    /// then blanked where they stand, so the history the primary screen built
    /// up stays under the alternate screen — `history_size` still counts it and
    /// a copy-mode scroll still reaches it. What the alternate screen loses is
    /// the ability to *add* to that history, which is `GRID_HISTORY` going out.
    pub(crate) fn alternate_on(&mut self, save_cursor: bool) {
        if self.saved_grid.is_some() {
            return;
        }
        let (sx, sy) = (self.grid.sx, self.grid.sy);
        let mut saved = Grid::new(sx, sy, 0);
        saved.duplicate_lines(0, &self.grid, self.grid.hsize, sy);
        self.saved_grid = Some(saved);
        if save_cursor {
            self.saved_state = Some(SavedState {
                cx: self.cx,
                cy: self.cy,
                cell: self.cell.clone(),
                mode: self.mode,
            });
        }
        // `grid_view_clear`, not the clear an `ED 2` performs: the rows are
        // blanked where they stand rather than scrolled anywhere.
        let py = self.grid.hsize;
        self.grid.clear(0, py, sx, sy, colour::DEFAULT);
        // The images go with the primary screen's rows rather than being freed,
        // so a vim session underneath does not cost the pane its images.
        self.saved_images.take_from(&mut self.images);
        self.saved_history = self.grid.history;
        self.grid.history = false;
    }

    /// tmux's `screen_alternate_off`.
    pub(crate) fn alternate_off(&mut self, restore_cursor: bool) {
        // The cursor and the cell come back whether or not an alternate screen
        // is up, and the position is clamped onto the screen either way — which
        // is how a pane sitting in the pending-wrap column loses it here.
        if restore_cursor {
            if let Some(state) = self.saved_state.clone() {
                self.cell = state.cell;
                self.cx = state.cx;
                self.cy = state.cy;
            }
        }
        let Some(saved) = self.saved_grid.take() else {
            self.cx = self.cx.min(self.sx() - 1);
            self.cy = self.cy.min(self.sy() - 1);
            return;
        };
        let (sx, sy) = (self.grid.sx, self.grid.sy);
        // The pane may have been resized while the alternate screen was up, and
        // the copy back goes row for row: the screen returns to the size the
        // rows were saved at, takes them, and comes back to the current one.
        if (saved.sx, saved.sy) != (sx, sy) {
            self.resize(saved.sx, saved.sy);
        }
        self.saved_state = None;
        let py = self.grid.hsize;
        let ny = saved.sy;
        self.grid.duplicate_lines(py, &saved, 0, ny);
        // History comes back on before the resize, so the resize may use it.
        self.grid.history = self.saved_history;
        if (saved.sx, saved.sy) != (sx, sy) {
            self.resize(sx, sy);
        }
        // Whatever the alternate screen drew goes; the primary screen's images
        // come back with its rows.
        self.images.free_all();
        self.images.take_from(&mut self.saved_images);
        self.cx = self.cx.min(self.sx() - 1);
        self.cy = self.cy.min(self.sy() - 1);
    }

    /// The primary screen's visible rows, put aside when the alternate screen
    /// went up, or `None` when no alternate screen is in use. tmux's
    /// `saved_grid`, which `capture-pane -a` reads; its presence is also what
    /// "the alternate screen is up" means.
    pub(crate) fn saved_grid(&self) -> Option<&Grid> {
        self.saved_grid.as_ref()
    }

    /// RIS, as tmux's `screen_write_reset` performs it.
    ///
    /// The grid is not replaced: the reset ends in the same clear an `ED 2`
    /// ends in, so the visible screen goes into the history rather than being
    /// discarded with it. `MODE_CRLF` is the one mode a reset leaves alone.
    pub(crate) fn reset(&mut self) {
        self.reset_tabs();
        let sy = self.sy();
        self.set_scroll_region(0, sy - 1);
        self.mode = mode::CURSOR | mode::WRAP | (self.mode & mode::CRLF);
        // The alternate screen survives a reset: `screen_write_reset` never
        // looks at `saved_grid`. That is observable, because the alternate
        // screen is also what has the grid's history turned off — so the clear
        // below blanks the rows where they stand instead of scrolling them into
        // a history the pane is not adding to.
        self.clear_screen(colour::DEFAULT);
        self.set_cursor(Some(0), Some(0));
        // `input_reset_cell`: the cell the pane draws with, and the one DECSC
        // would restore, both go back to the default.
        self.cell = Cell::default();
        self.saved_state = None;
        self.saved_cursor = None;
    }

    /// Resize the screen, as tmux's `screen_resize` does: the width first, then
    /// the height, with the tab stops laid out again against the new width.
    pub(crate) fn resize(&mut self, sx: usize, sy: usize) {
        let sx = sx.max(1);
        let sy = sy.max(1);
        // tmux's `screen_resize` drops the images outright rather than trying
        // to place them again against a grid that has just been rewrapped.
        self.images.free_all();
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

/// An image's size in cells, as the screen counts cells.
fn cells(image: &SixelImage) -> (usize, usize) {
    let (x, y) = image.size_in_cells();
    (x as usize, y as usize)
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

    /// tmux's `screen_write_cursorup` and `screen_write_cursordown` are the
    /// only moves that resolve the pending-wrap column, so a pane that filled
    /// its last column and then moved a row reports the last real column
    /// rather than the one past it.
    #[test]
    fn a_vertical_move_steps_off_the_pending_wrap_column() {
        for down in [false, true] {
            let mut screen = Screen::new(4, 3, 100);
            put(&mut screen, "abcd");
            assert_eq!(screen.cx, 4, "the last column leaves the wrap pending");
            if down {
                screen.cursor_down(1);
            } else {
                screen.cursor_up(1);
            }
            assert_eq!(screen.cx, 3, "the move lands on the last real column");
        }
    }

    /// The horizontal moves have a column of their own to land on, so they
    /// clamp the way they always did rather than through the wrap.
    #[test]
    fn a_horizontal_move_from_the_pending_wrap_column_still_clamps() {
        let mut screen = Screen::new(4, 3, 100);
        put(&mut screen, "abcd");
        screen.cursor_right(1);
        assert_eq!(screen.cx, 3);
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
    fn the_alternate_screen_leaves_the_history_it_found() {
        let mut screen = Screen::new(8, 2, 100);
        put(&mut screen, "one");
        screen.linefeed(false, colour::DEFAULT);
        screen.linefeed(false, colour::DEFAULT);
        assert_eq!(screen.grid.hsize, 1, "the primary screen built history");
        screen.alternate_on(false);
        assert_eq!(
            screen.grid.hsize, 1,
            "which the alternate screen does not take with it"
        );
        assert_eq!(history(&screen, 0), "one");
        screen.alternate_off(false);
        assert_eq!(screen.grid.hsize, 1);
        assert_eq!(history(&screen, 0), "one");
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

    // ---- images ----------------------------------------------------------

    /// A solid image `cells_x` by `cells_y` cells, measured against the default
    /// sixteen by thirty-two pixel cell a screen assumes.
    fn image(cells_x: usize, cells_y: usize) -> SixelImage {
        let mut payload = b"#0;2;100;0;0#0".to_vec();
        // Each band of six pixel rows takes one `-`, and the height wanted is
        // the smallest that still rounds up to `cells_y` cells.
        let bands = ((cells_y - 1) * 32) / 6 + 1;
        for band in 0..bands {
            if band != 0 {
                payload.push(b'-');
            }
            payload.extend_from_slice(format!("!{}~", cells_x * 16).as_bytes());
        }
        crate::vt::sixel::parse(&payload, 0, 16, 32).expect("the payload parses")
    }

    fn placed(screen: &Screen) -> Vec<(usize, usize, usize, usize)> {
        screen
            .images
            .live()
            .map(|image| (image.px, image.py, image.sx, image.sy))
            .collect()
    }

    #[test]
    fn an_image_lands_at_the_cursor_and_the_cursor_ends_below_it() {
        let mut screen = Screen::new(10, 5, 100);
        screen.set_cursor(Some(3), Some(1));
        screen.sixel_image(image(2, 2), colour::DEFAULT);
        assert_eq!(placed(&screen), vec![(3, 1, 2, 2)]);
        assert_eq!(
            (screen.cx, screen.cy),
            (0, 3),
            "the cursor returns to column zero, below the image"
        );
    }

    #[test]
    fn a_one_row_screen_has_nowhere_to_put_an_image() {
        let mut screen = Screen::new(10, 1, 100);
        screen.sixel_image(image(1, 1), colour::DEFAULT);
        assert!(placed(&screen).is_empty());
    }

    #[test]
    fn an_image_with_no_room_below_the_cursor_scrolls_the_screen() {
        let mut screen = Screen::new(10, 5, 100);
        for row in 0..5 {
            screen.set_cursor(Some(0), Some(row));
            put(&mut screen, &format!("r{row}"));
        }
        screen.set_cursor(Some(0), Some(4));
        screen.sixel_image(image(1, 2), colour::DEFAULT);
        // Two rows are needed and one was left, so the screen scrolled by two.
        assert_eq!(text(&screen, 0), "r2");
        assert_eq!(screen.grid.hsize, 2, "the rows went into the history");
        assert_eq!(placed(&screen), vec![(0, 2, 1, 2)]);
        assert_eq!((screen.cx, screen.cy), (0, 4));
    }

    /// The scroll is of the whole screen, not of the scrolling region: an image
    /// is not part of the region's contents.
    #[test]
    fn making_room_for_an_image_ignores_the_scrolling_region() {
        let mut screen = Screen::new(10, 5, 100);
        for row in 0..5 {
            screen.set_cursor(Some(0), Some(row));
            put(&mut screen, &format!("r{row}"));
        }
        screen.set_scroll_region(1, 3);
        screen.set_cursor(Some(0), Some(4));
        screen.sixel_image(image(1, 1), colour::DEFAULT);
        // One row was needed and the cursor was on the last one, so everything
        // moved up by one — including `r0`, which the region does not cover.
        assert_eq!(text(&screen, 0), "r1");
        assert_eq!(text(&screen, 3), "r4", "the row below the region moved too");
        assert_eq!(placed(&screen), vec![(0, 3, 1, 1)]);
    }

    #[test]
    fn an_image_too_wide_for_the_screen_is_cropped_to_it() {
        let mut screen = Screen::new(4, 5, 100);
        screen.sixel_image(image(6, 1), colour::DEFAULT);
        assert_eq!(placed(&screen), vec![(0, 0, 4, 1)]);
    }

    /// Too tall, the *top* is what goes: the bottom of the image is the part a
    /// terminal keeps.
    #[test]
    fn an_image_too_tall_for_the_screen_keeps_its_bottom() {
        let mut screen = Screen::new(10, 4, 100);
        screen.sixel_image(image(1, 5), colour::DEFAULT);
        assert_eq!(placed(&screen), vec![(0, 0, 1, 3)]);
    }

    #[test]
    fn writing_over_an_image_takes_it_away() {
        let mut screen = Screen::new(10, 5, 100);
        screen.set_cursor(Some(2), Some(1));
        screen.sixel_image(image(2, 1), colour::DEFAULT);
        screen.set_cursor(Some(0), Some(1));
        put(&mut screen, "x");
        assert_eq!(placed(&screen).len(), 1, "column zero is beside it");
        screen.set_cursor(Some(3), Some(1));
        put(&mut screen, "x");
        assert!(placed(&screen).is_empty(), "column three is under it");
    }

    #[test]
    fn clearing_the_screen_takes_every_image_with_it() {
        let mut screen = Screen::new(10, 5, 100);
        screen.sixel_image(image(1, 1), colour::DEFAULT);
        screen.clear_screen(colour::DEFAULT);
        assert!(placed(&screen).is_empty());
    }

    /// tmux's `screen_write_clearstartofscreen` passes `s->cy - 1`, which on the
    /// first row underflows and frees every image rather than the rows above.
    #[test]
    fn clearing_to_the_start_of_the_screen_from_the_first_row_frees_them_all() {
        let mut screen = Screen::new(10, 5, 100);
        screen.set_cursor(Some(0), Some(3));
        screen.sixel_image(image(1, 1), colour::DEFAULT);
        screen.set_cursor(Some(1), Some(0));
        screen.clear_start_of_screen(colour::DEFAULT);
        assert!(placed(&screen).is_empty());
    }

    #[test]
    fn a_linefeed_at_the_bottom_carries_the_images_up_with_it() {
        let mut screen = Screen::new(10, 5, 100);
        screen.set_cursor(Some(0), Some(1));
        screen.sixel_image(image(1, 1), colour::DEFAULT);
        screen.set_cursor(Some(0), Some(4));
        screen.linefeed(false, colour::DEFAULT);
        assert_eq!(placed(&screen), vec![(0, 0, 1, 1)]);
        screen.linefeed(false, colour::DEFAULT);
        assert!(placed(&screen).is_empty(), "it left the top of the screen");
    }

    #[test]
    fn the_alternate_screen_gives_the_images_back_on_the_way_out() {
        let mut screen = Screen::new(10, 5, 100);
        screen.sixel_image(image(1, 1), colour::DEFAULT);
        screen.alternate_on(true);
        assert!(placed(&screen).is_empty(), "the alternate screen is bare");
        screen.sixel_image(image(2, 1), colour::DEFAULT);
        screen.alternate_off(true);
        assert_eq!(
            placed(&screen),
            vec![(0, 0, 1, 1)],
            "the primary screen's image came back and the other one did not"
        );
    }

    #[test]
    fn a_resize_drops_the_images() {
        let mut screen = Screen::new(10, 5, 100);
        screen.sixel_image(image(1, 1), colour::DEFAULT);
        screen.resize(10, 6);
        assert!(placed(&screen).is_empty());
    }
}
