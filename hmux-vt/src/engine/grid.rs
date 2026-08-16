//! The cell store: tmux's `grid.c`.
//!
//! A grid is `hsize` history lines followed by `sy` visible lines, addressed
//! together: row zero is the oldest history row and row `hsize` is the top of
//! the viewport. That is the same addressing `grid.c` uses and the same one
//! `capture-pane -S/-E` speaks, so nothing has to be translated between them.
//!
//! Two per-line lengths matter and are not the same. `cells.len()` is the
//! allocated extent, which decides what an out-of-range read returns; `used` is
//! how far a program has actually written, which is what `capture-pane -N`
//! reports. tmux keeps both (`cellsize` and `cellused`) and so does this.

use super::cell::{colour, flag, Cell, CellData};

/// tmux's `GRID_LINE_*`.
pub mod line_flag {
    /// The line soft-wraps into the next one.
    pub const WRAPPED: u8 = 0x1;
    /// Some cell on this line has needed tmux's extended cell representation.
    /// tmux sets this in `grid_extended_cell`, where it is an allocation
    /// decision, and never clears it; `capture-pane -F` reports it as `X`.
    pub const EXTENDED: u8 = 0x2;
    /// A shell-integration prompt starts on this line (OSC 133 A).
    pub const START_PROMPT: u8 = 0x8;
    /// Command output starts on this line (OSC 133 C).
    pub const START_OUTPUT: u8 = 0x10;
    /// Some cell on this line belongs to an OSC 8 hyperlink. `capture-pane -e`
    /// checks this before it walks a row looking for links.
    pub const HYPERLINK: u8 = 0x20;
}

/// Lay one logical line out at `sx` columns, appending the rows it becomes.
///
/// An empty logical line still produces one row: it was a line, and dropping it
/// would shift everything below it up.
fn lay_out(out: &mut Vec<Line>, cells: Vec<Cell>, sx: usize, flags: u8) {
    let mut row = Line {
        flags,
        ..Line::default()
    };
    let mut width = 0;
    for cell in cells {
        let cell_width = usize::from(cell.data.width).max(1);
        if width + cell_width > sx {
            row.flags |= line_flag::WRAPPED;
            out.push(std::mem::replace(
                &mut row,
                Line {
                    flags,
                    ..Line::default()
                },
            ));
            width = 0;
        }
        width += cell_width;
        if needs_extended(&cell) {
            row.extd += 1;
        }
        row.cells.push(cell);
        row.used = row.cells.len();
    }
    out.push(row);
}

/// tmux's `grid_need_extended_cell`: whether a cell is too rich for the compact
/// entry and has to be stored in the line's extended array.
///
/// This engine stores every cell the same way, so the answer changes no storage
/// here. It is still worth computing, because tmux lets the answer show: the
/// line flag it sets is sticky and `capture-pane -F` prints it.
pub fn needs_extended(cell: &Cell) -> bool {
    cell.attr > 0xff
        || cell.data.len() > 1
        || cell.data.width > 1
        || cell.fg & colour::FLAG_RGB != 0
        || cell.bg & colour::FLAG_RGB != 0
        || cell.us != colour::DEFAULT
        || cell.link != 0
        || cell.flags & flag::TAB != 0
}

/// tmux's `COLOUR_DEFAULT`: neither of the two spellings of "unset".
pub fn colour_is_default(value: i32) -> bool {
    value == 8 || value == 9
}

/// One row.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Line {
    /// The allocated cells. tmux's `cellsize` is this length.
    cells: Vec<Cell>,
    /// tmux's `cellused`: how far a program has written into the row.
    used: usize,
    /// tmux's `extdsize`: how many extended entries the line has allocated.
    /// Monotonic while the line lives — tmux never reclaims an entry when a
    /// rich cell is overwritten with a simple one — and reset with the line.
    extd: usize,
    pub flags: u8,
}

impl Line {
    /// The allocated extent, tmux's `cellsize`, which is where a capture that
    /// keeps its empty cells stops.
    pub fn size(&self) -> usize {
        self.cells.len()
    }

    /// The written extent, tmux's `cellused`.
    pub fn used(&self) -> usize {
        self.used
    }

    /// The allocated extended entries, tmux's `extdsize`.
    pub fn extd(&self) -> usize {
        self.extd
    }

    pub fn is_wrapped(&self) -> bool {
        self.flags & line_flag::WRAPPED != 0
    }
}

/// The cell store for one screen.
#[derive(Clone, Debug)]
pub struct Grid {
    /// Visible width and height.
    pub sx: usize,
    pub sy: usize,
    /// How many history rows are stored, tmux's `hsize`.
    pub hsize: usize,
    /// The history cap. Zero means this grid keeps no history at all.
    pub hlimit: usize,
    /// tmux's `GRID_HISTORY`. Cleared while the alternate screen is up, so
    /// scrolling there adds nothing to the scrollback the primary screen built.
    pub history: bool,
    /// How far the history has scrolled, tmux's `hscrolled`. Only the
    /// `history_bytes`-style accounting reads it.
    pub hscrolled: usize,
    lines: Vec<Line>,
}

impl Grid {
    /// tmux's `grid_create`. `hlimit` of zero means no history, which is also
    /// what leaves `GRID_HISTORY` clear.
    pub fn new(sx: usize, sy: usize, hlimit: usize) -> Grid {
        Grid {
            sx,
            sy,
            hsize: 0,
            hlimit,
            history: hlimit != 0,
            hscrolled: 0,
            lines: vec![Line::default(); sy],
        }
    }

    /// tmux's `grid_duplicate_lines`: copy `ny` rows out of `src` at row `sy`
    /// over this grid's rows from `dy`, clipped to what either grid holds.
    pub fn duplicate_lines(&mut self, dy: usize, src: &Grid, sy: usize, ny: usize) {
        let ny = ny
            .min(self.total().saturating_sub(dy))
            .min(src.total().saturating_sub(sy));
        for offset in 0..ny {
            self.lines[dy + offset] = src.lines[sy + offset].clone();
        }
    }

    /// Total stored rows: history plus viewport.
    pub fn total(&self) -> usize {
        self.hsize + self.sy
    }

    pub fn line(&self, py: usize) -> Option<&Line> {
        self.lines.get(py)
    }

    /// tmux's `grid_get_cell`: a read past the allocated extent of a row is
    /// the *default* cell, not a cleared one — a distinction `capture-pane -e`
    /// can see, because a cleared cell may carry a background colour.
    pub fn get(&self, px: usize, py: usize) -> Cell {
        match self.lines.get(py) {
            Some(line) => line.cells.get(px).cloned().unwrap_or_default(),
            None => Cell::default(),
        }
    }

    /// The cell as it is actually stored, or `None` past the row's allocated
    /// extent. Unlike [`Grid::get`] this tells "a cell holding a default" apart
    /// from "no cell at all", which is a distinction `screen_write_cell` makes.
    pub fn peek(&self, px: usize, py: usize) -> Option<&Cell> {
        self.lines.get(py).and_then(|line| line.cells.get(px))
    }

    /// tmux's `grid_set_cell`.
    pub fn set(&mut self, px: usize, py: usize, cell: &Cell) {
        if py >= self.lines.len() {
            return;
        }
        self.expand_line(py, px + 1, colour::DEFAULT);
        let line = &mut self.lines[py];
        if px + 1 > line.used {
            line.used = px + 1;
        }
        if cell.link != 0 {
            line.flags |= line_flag::HYPERLINK;
        }
        if needs_extended(cell) {
            line.flags |= line_flag::EXTENDED;
            // tmux allocates a new extended entry only when the cell was not
            // already extended; overwriting extended with simple leaks the
            // entry, which is why the count never goes down.
            if !needs_extended(&line.cells[px]) {
                line.extd += 1;
            }
        }
        line.cells[px] = *cell;
    }

    /// tmux's `grid_set_padding`: the blank right half of a wide character.
    pub fn set_padding(&mut self, px: usize, py: usize) {
        self.set(px, py, &Cell::padding());
    }

    /// tmux's `grid_expand_line`, rounding included.
    ///
    /// The rounding is not just an allocation detail: the cells it brings into
    /// existence are *cleared* with `bg`, so how far a row grows decides what
    /// colour the space past a program's last write has.
    pub fn expand_line(&mut self, py: usize, sx: usize, bg: i32) {
        let Some(line) = self.lines.get(py) else {
            return;
        };
        if sx <= line.cells.len() {
            return;
        }
        self.grow_line(py, self.rounded_extent(sx), bg);
    }

    /// The extent `grid_expand_line` rounds a request for `sx` cells up to: a
    /// quarter of the screen width, then a half, then the width itself.
    ///
    /// The steps are why the size of a *write* is observable. tmux expands a
    /// row once for the whole run it is about to put there, so a run that ends
    /// on a step rounds past it; the same characters written a cell at a time
    /// would stop at the first step and never allocate the rest.
    pub fn rounded_extent(&self, sx: usize) -> usize {
        if sx < self.sx / 4 {
            self.sx / 4
        } else if sx < self.sx / 2 {
            self.sx / 2
        } else if self.sx > sx {
            self.sx
        } else {
            sx
        }
    }

    /// Grow a row to `sx` allocated cells, clearing the new ones with `bg`.
    /// The rounding is the caller's, so this is the half of `grid_expand_line`
    /// a run can apply once it knows how far it reached.
    pub fn grow_line(&mut self, py: usize, sx: usize, bg: i32) {
        let Some(line) = self.lines.get_mut(py) else {
            return;
        };
        if sx <= line.cells.len() {
            return;
        }
        // An RGB background makes the fill cell itself extended, and tmux's
        // `grid_clear_cell` allocates an entry for each one it writes.
        let grown = sx - line.cells.len();
        if needs_extended(&Cell::cleared(bg)) {
            line.extd += grown;
        }
        line.cells.resize(sx, Cell::cleared(bg));
    }

    /// tmux's `grid_empty_line`: forget the row entirely. A non-default
    /// background is painted straight back over the full width, because an
    /// erase to a colour has to be visible where nothing was written.
    pub fn empty_line(&mut self, py: usize, bg: i32) {
        if py >= self.lines.len() {
            return;
        }
        self.lines[py] = Line::default();
        if !colour_is_default(bg) {
            self.expand_line(py, self.sx, bg);
        }
    }

    /// tmux's `grid_clear_lines`.
    pub fn clear_lines(&mut self, py: usize, ny: usize, bg: i32) {
        if ny == 0 || py + ny > self.lines.len() {
            return;
        }
        for yy in py..py + ny {
            self.empty_line(yy, bg);
        }
        if py != 0 {
            self.lines[py - 1].flags &= !line_flag::WRAPPED;
        }
    }

    /// tmux's `grid_clear`: erase a rectangle.
    pub fn clear(&mut self, px: usize, py: usize, nx: usize, ny: usize, bg: i32) {
        if nx == 0 || ny == 0 {
            return;
        }
        if px == 0 && nx == self.sx {
            self.clear_lines(py, ny, bg);
            return;
        }
        if py + ny > self.lines.len() {
            return;
        }
        for yy in py..py + ny {
            let sx = self.sx.min(self.lines[yy].cells.len());
            let mut ox = nx;
            if colour_is_default(bg) {
                // With the default background there is nothing to paint past
                // what the row already holds.
                if px > sx {
                    continue;
                }
                if px + nx > sx {
                    ox = sx - px;
                }
            }
            self.expand_line(yy, px + ox, colour::DEFAULT);
            for xx in px..px + ox {
                self.clear_cell(xx, yy, bg);
            }
        }
    }

    /// tmux's `grid_clear_cell`: put a cleared cell in place, keeping `bg`.
    pub fn clear_cell(&mut self, px: usize, py: usize, bg: i32) {
        if let Some(line) = self.lines.get_mut(py) {
            if let Some(cell) = line.cells.get_mut(px) {
                *cell = Cell::cleared(bg);
            }
        }
    }

    /// tmux's `grid_move_lines`: move `ny` rows from `py` to `dy` within the
    /// grid, blanking whatever the move vacated.
    pub fn move_lines(&mut self, dy: usize, py: usize, ny: usize, bg: i32) {
        if ny == 0 || py == dy {
            return;
        }
        if py + ny > self.lines.len() || dy + ny > self.lines.len() {
            return;
        }
        if dy != 0 {
            self.lines[dy - 1].flags &= !line_flag::WRAPPED;
        }
        let moved: Vec<Line> = self.lines[py..py + ny].to_vec();
        self.lines[dy..dy + ny].clone_from_slice(&moved);
        for yy in py..py + ny {
            if yy < dy || yy >= dy + ny {
                self.empty_line(yy, bg);
            }
        }
        if py != 0 && (py < dy || py >= dy + ny) {
            self.lines[py - 1].flags &= !line_flag::WRAPPED;
        }
    }

    /// tmux's `grid_move_cells`: shift cells within one row, as insert and
    /// delete character do.
    pub fn move_cells(&mut self, dx: usize, px: usize, py: usize, nx: usize, bg: i32) {
        if nx == 0 || px == dx || py >= self.lines.len() {
            return;
        }
        self.expand_line(py, px + nx, colour::DEFAULT);
        self.expand_line(py, dx + nx, colour::DEFAULT);
        let moved: Vec<Cell> = self.lines[py].cells[px..px + nx].to_vec();
        self.lines[py].cells[dx..dx + nx].clone_from_slice(&moved);
        if dx + nx > self.lines[py].used {
            self.lines[py].used = dx + nx;
        }
        for xx in px..px + nx {
            if xx < dx || xx >= dx + nx {
                self.clear_cell(xx, py, bg);
            }
        }
    }

    /// tmux's `grid_scroll_history`: the top visible row becomes history and a
    /// fresh row appears at the bottom.
    pub fn scroll_history(&mut self, bg: i32) {
        self.lines.push(Line::default());
        let last = self.lines.len() - 1;
        self.empty_line(last, bg);
        self.hscrolled += 1;
        self.hsize += 1;
    }

    /// tmux's `grid_scroll_history_region`: scrolling a region whose top is the
    /// top of the screen still feeds the history, unlike one further down.
    pub fn scroll_history_region(&mut self, upper: usize, lower: usize, bg: i32) {
        self.lines.insert(self.hsize, Line::default());
        // The region moved down by one with the insert; take its first row into
        // the history slot and close the gap behind it.
        let upper = self.hsize + upper + 1;
        let lower = self.hsize + lower + 1;
        let promoted = self.lines.remove(upper);
        self.lines[self.hsize] = promoted;
        self.lines.insert(lower, Line::default());
        self.empty_line(lower, bg);
        self.hscrolled += 1;
        self.hsize += 1;
    }

    /// tmux's `grid_collect_history`: drop the oldest tenth once the history is
    /// over its limit, so trimming is amortized rather than per-line.
    pub fn collect_history(&mut self) {
        if self.hsize == 0 || self.hsize < self.hlimit {
            return;
        }
        let mut ny = (self.hlimit / 10).max(1);
        if ny > self.hsize {
            ny = self.hsize;
        }
        self.trim_history(ny);
    }

    /// Change the history cap and immediately discard rows beyond it.
    ///
    /// tmux applies a lowered `history-limit` to panes that already exist, so
    /// this is the `all` form of `grid_collect_history`, unlike the amortized
    /// one-tenth collection used while output is scrolling.
    pub fn set_history_limit(&mut self, hlimit: usize) {
        self.hlimit = hlimit;
        if self.hsize > hlimit {
            self.trim_history(self.hsize - hlimit);
        }
    }

    fn trim_history(&mut self, ny: usize) {
        if ny == 0 || ny > self.hsize {
            return;
        }
        self.lines.drain(..ny);
        self.hsize -= ny;
        if self.hscrolled > self.hsize {
            self.hscrolled = self.hsize;
        }
    }

    /// tmux's `grid_clear_history`.
    /// tmux's `grid_view_clear_history`: push the viewport into the history
    /// instead of blanking it where it stands.
    ///
    /// Only the rows a program actually wrote go. A screen whose last two rows
    /// were never touched scrolls the rows above them and blanks the rest, so
    /// clearing does not fill the scrollback with the blank tail of a
    /// half-drawn screen — and a screen nothing was written to scrolls nothing
    /// at all.
    pub fn view_clear_history(&mut self, bg: i32) {
        let last = (0..self.sy)
            .filter(|yy| {
                self.line(self.hsize + yy)
                    .is_some_and(|line| line.used() != 0)
            })
            .map(|yy| yy + 1)
            .next_back()
            .unwrap_or(0);
        if last == 0 {
            let py = self.hsize;
            self.clear(0, py, self.sx, self.sy, bg);
            return;
        }
        for _ in 0..last {
            self.collect_history();
            self.scroll_history(bg);
        }
        if last < self.sy {
            let py = self.hsize;
            self.clear(0, py, self.sx, self.sy - last, bg);
        }
        // The view is back at the bottom: everything that was on it is history.
        self.hscrolled = 0;
    }

    /// tmux's `grid_remove_history`: drop the last `ny` rows and count `ny`
    /// fewer of them as history.
    ///
    /// Nothing moves. The viewport is addressed from `hsize`, so lowering it
    /// slides the window down over rows that were history a moment ago, and the
    /// rows that fall off the bottom are the ones the viewport no longer
    /// reaches. Asking to remove more history than there is does nothing at
    /// all, as in tmux.
    pub fn remove_history(&mut self, ny: usize) {
        if ny > self.hsize {
            return;
        }
        self.lines.truncate(self.hsize + self.sy - ny);
        self.hsize -= ny;
    }

    pub fn clear_history(&mut self) {
        self.lines.drain(..self.hsize);
        self.hsize = 0;
        self.hscrolled = 0;
    }

    /// tmux's `grid_line_length`: the row's extent with trailing spaces
    /// trimmed, which is what a plain capture of the row shows.
    pub fn line_length(&self, py: usize) -> usize {
        let Some(line) = self.lines.get(py) else {
            return 0;
        };
        let mut px = line.cells.len().min(self.sx);
        while px > 0 {
            let cell = &line.cells[px - 1];
            if cell.is_padding() || !cell.data.is_space() {
                break;
            }
            px -= 1;
        }
        px
    }

    /// Set or clear a line flag on a stored row.
    pub fn set_line_flags(&mut self, py: usize, flags: u8, on: bool) {
        if let Some(line) = self.lines.get_mut(py) {
            if on {
                line.flags |= flags;
            } else {
                line.flags &= !flags;
            }
        }
    }

    /// Grow or shrink the viewport height: tmux's `screen_resize_y`.
    ///
    /// The cursor is carried through as an *absolute* row and handed back the
    /// same way, because that is the coordinate a resize does not move. How far
    /// the cursor travels in viewport terms falls out of what happened to the
    /// history, which is exactly how tmux keeps the two in step.
    pub fn resize_y(&mut self, sy: usize, cursor: usize, bg: i32) -> usize {
        if sy == self.sy {
            return cursor;
        }
        if sy < self.sy {
            let before = self.sy;
            let cy = cursor.saturating_sub(self.hsize);
            let mut needed = before - sy;

            // Delete as many rows as possible from below the cursor before
            // touching the history. Skipping this is not a small difference:
            // a pane merely made shorter would accumulate scrollback it never
            // scrolled, and every row index the server reports — copy mode's
            // cursor, `capture-pane`'s ranges, `history_size` — would shift.
            let eaten = needed.min((before - 1).saturating_sub(cy));
            needed -= eaten;

            let mut cursor = cursor;
            if self.hlimit != 0 {
                // The rows left over become history, which leaves the absolute
                // cursor row where it was.
                self.hsize += needed;
                self.hscrolled += needed;
            } else if needed > 0 {
                // With no history to give them to, they come off the top and
                // the cursor comes with them.
                let available = cy.min(needed);
                if available > 0 {
                    self.move_lines(self.hsize, self.hsize + available, before - available, bg);
                    cursor -= available;
                }
            }
            self.lines.truncate(self.hsize + sy);
            self.sy = sy;
            return cursor;
        }

        let gained = sy - self.sy;
        // Pull rows back out of the history where there are any, and otherwise
        // add blank ones at the bottom.
        let from_history = gained.min(self.hsize);
        self.hsize -= from_history;
        for _ in 0..(gained - from_history) {
            self.lines.push(Line::default());
            let last = self.lines.len() - 1;
            self.empty_line(last, bg);
        }
        self.sy = sy;
        cursor
    }

    /// Rewrap every stored row to a new width: tmux's `grid_reflow`.
    ///
    /// tmux walks the rows one at a time, moving, splitting or joining each
    /// against the new width. The same result falls out of doing it the other
    /// way round — gather each *logical* line by following the wrap flags, then
    /// lay it out again at the new width — and that is what this does, because
    /// the joins and splits then cannot disagree with each other.
    ///
    /// A wide character is never split across the margin: a row ends early
    /// rather than leave half of one behind.
    pub fn reflow(&mut self, sx: usize) {
        if sx == 0 || sx == self.sx {
            self.sx = sx.max(1);
            return;
        }
        let mut rewrapped: Vec<Line> = Vec::new();
        let mut logical: Vec<Cell> = Vec::new();
        let mut carried_flags = 0u8;

        let lines = std::mem::take(&mut self.lines);
        for line in lines {
            // The flags that belong to the logical line, not to the row the
            // old width happened to split it into.
            carried_flags |= line.flags & !line_flag::WRAPPED;
            logical.extend(line.cells.into_iter().take(line.used));
            if line.flags & line_flag::WRAPPED != 0 {
                continue;
            }
            lay_out(
                &mut rewrapped,
                std::mem::take(&mut logical),
                sx,
                carried_flags,
            );
            carried_flags = 0;
        }
        // A trailing run that never met an unwrapped row still has to land.
        if !logical.is_empty() {
            lay_out(&mut rewrapped, logical, sx, carried_flags);
        }

        self.sx = sx;
        // The viewport keeps its height; whatever the rewrap left above it is
        // history.
        while rewrapped.len() < self.sy {
            rewrapped.push(Line::default());
        }
        self.hsize = rewrapped.len() - self.sy;
        if self.hscrolled > self.hsize {
            self.hscrolled = self.hsize;
        }
        self.lines = rewrapped;
    }

    /// Where a cursor sits within its *logical* line, so it can be found again
    /// after a rewrap. tmux's `grid_wrap_position`.
    pub fn wrap_position(&self, px: usize, py: usize) -> (Option<usize>, usize) {
        let mut ax = 0;
        let mut ay = 0;
        for yy in 0..py.min(self.lines.len()) {
            if self.lines[yy].is_wrapped() {
                ax += self.lines[yy].used;
            } else {
                ax = 0;
                ay += 1;
            }
        }
        let used = self.lines.get(py).map_or(0, |line| line.used);
        // Past the row's own content the column is not a position in the
        // logical line at all; it means "the end", wherever that lands.
        if px >= used {
            (None, ay)
        } else {
            (Some(ax + px), ay)
        }
    }

    /// The inverse: turn a logical position back into a row and column.
    /// tmux's `grid_unwrap_position`.
    pub fn unwrap_position(&self, wx: Option<usize>, wy: usize) -> (usize, usize) {
        let mut yy = 0;
        let mut ay = 0;
        while yy + 1 < self.lines.len() {
            if ay == wy {
                break;
            }
            if !self.lines[yy].is_wrapped() {
                ay += 1;
            }
            yy += 1;
        }
        let Some(mut wx) = wx else {
            while self.lines.get(yy).is_some_and(Line::is_wrapped) {
                yy += 1;
            }
            return (self.lines.get(yy).map_or(0, |line| line.used), yy);
        };
        while self.lines.get(yy).is_some_and(Line::is_wrapped) {
            let used = self.lines[yy].used;
            if wx < used {
                break;
            }
            wx -= used;
            yy += 1;
        }
        (wx, yy)
    }

    /// tmux's `grid_set_tab`: one cell standing for a run of `width` columns a
    /// horizontal tab created, rather than for spaces someone wrote.
    pub fn tab_cell(template: &Cell, width: usize) -> Cell {
        Cell {
            data: CellData::blanks(width),
            flags: (template.flags | flag::TAB) & !flag::PADDING,
            ..*template
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn written(grid: &Grid, py: usize) -> String {
        (0..grid.line_length(py))
            .map(|px| grid.get(px, py).data.text().to_string())
            .collect()
    }

    #[test]
    fn a_written_row_reports_both_its_extents() {
        let mut grid = Grid::new(80, 4, 100);
        grid.set(
            0,
            0,
            &Cell {
                data: CellData::from_char('a', 1),
                ..Cell::default()
            },
        );
        let line = grid.line(0).expect("row");
        assert_eq!(line.used(), 1, "one cell has been written");
        assert!(
            line.size() >= 20,
            "but the row is allocated in chunks, got {}",
            line.size()
        );
    }

    #[test]
    fn a_read_past_the_allocated_extent_is_the_default_cell() {
        let grid = Grid::new(80, 4, 100);
        let cell = grid.get(79, 0);
        assert_eq!(cell.bg, colour::DEFAULT);
        assert_eq!(cell.flags, 0, "not a cleared cell, an absent one");
    }

    #[test]
    fn clearing_to_a_colour_paints_the_whole_width() {
        let mut grid = Grid::new(10, 2, 0);
        grid.clear_lines(0, 1, colour::indexed(4));
        assert_eq!(grid.get(9, 0).bg, colour::indexed(4));
        assert_eq!(grid.line(0).expect("row").size(), 10);
    }

    #[test]
    fn clearing_to_the_default_leaves_untouched_cells_alone() {
        let mut grid = Grid::new(10, 2, 0);
        grid.clear_lines(0, 1, colour::DEFAULT);
        assert_eq!(
            grid.line(0).expect("row").size(),
            0,
            "nothing has to be allocated to erase to the default"
        );
    }

    #[test]
    fn line_length_trims_trailing_spaces_but_not_written_text() {
        let mut grid = Grid::new(10, 2, 0);
        for (px, ch) in "hi".chars().enumerate() {
            grid.set(
                px,
                0,
                &Cell {
                    data: CellData::from_char(ch, 1),
                    ..Cell::default()
                },
            );
        }
        grid.set(4, 0, &Cell::default()); // a written space
        assert_eq!(grid.line_length(0), 2);
        assert_eq!(written(&grid, 0), "hi");
    }

    #[test]
    fn scrolling_moves_the_top_row_into_history() {
        let mut grid = Grid::new(10, 2, 100);
        grid.set(
            0,
            0,
            &Cell {
                data: CellData::from_char('a', 1),
                ..Cell::default()
            },
        );
        grid.scroll_history(colour::DEFAULT);
        assert_eq!(grid.hsize, 1);
        assert_eq!(grid.total(), 3);
        assert_eq!(written(&grid, 0), "a", "the row is now history row zero");
    }

    #[test]
    fn history_is_collected_a_tenth_at_a_time_once_over_the_limit() {
        let mut grid = Grid::new(10, 1, 10);
        for _ in 0..10 {
            grid.scroll_history(colour::DEFAULT);
        }
        assert_eq!(grid.hsize, 10);
        grid.collect_history();
        assert_eq!(grid.hsize, 9, "the oldest tenth goes");
    }

    #[test]
    fn moving_lines_blanks_what_it_vacated() {
        let mut grid = Grid::new(10, 4, 0);
        grid.set(
            0,
            2,
            &Cell {
                data: CellData::from_char('x', 1),
                ..Cell::default()
            },
        );
        grid.move_lines(0, 2, 1, colour::DEFAULT);
        assert_eq!(written(&grid, 0), "x");
        assert_eq!(written(&grid, 2), "");
    }

    #[test]
    fn moving_cells_shifts_within_the_row() {
        let mut grid = Grid::new(10, 1, 0);
        for (px, ch) in "abc".chars().enumerate() {
            grid.set(
                px,
                0,
                &Cell {
                    data: CellData::from_char(ch, 1),
                    ..Cell::default()
                },
            );
        }
        grid.move_cells(1, 0, 0, 3, colour::DEFAULT);
        assert_eq!(written(&grid, 0), " abc");
    }

    #[test]
    fn a_region_scroll_at_the_top_of_the_screen_still_feeds_history() {
        let mut grid = Grid::new(10, 4, 100);
        for row in 0..4 {
            grid.set(
                0,
                row,
                &Cell {
                    data: CellData::from_char((b'a' + row as u8) as char, 1),
                    ..Cell::default()
                },
            );
        }
        grid.scroll_history_region(0, 2, colour::DEFAULT);
        assert_eq!(grid.hsize, 1);
        assert_eq!(written(&grid, 0), "a", "the region's top row is history");
        assert_eq!(written(&grid, 1), "b");
        assert_eq!(written(&grid, 2), "c");
        assert_eq!(written(&grid, 3), "", "the region's bottom row is blank");
        assert_eq!(written(&grid, 4), "d", "outside the region, untouched");
    }
}
