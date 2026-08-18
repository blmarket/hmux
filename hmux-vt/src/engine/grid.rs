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

use super::cell::{Cell, CellData, colour, flag};

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
///
/// `time` is the history stamp of the logical line's first row and lands on the
/// first row it becomes; the rows a split adds behind it carry none, as the
/// ones tmux's `grid_reflow_add` allocates do not.
fn lay_out(out: &mut Vec<Line>, cells: Vec<Cell>, sx: usize, flags: u8, time: u64) {
    let mut row = Line {
        flags,
        time,
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
        row.push_cell(&cell);
    }
    out.push(row);
}

/// tmux's `grid_need_extended_cell`: whether a cell is too rich for the compact
/// entry and has to be stored in the line's extended array.
///
/// This decides storage, and it is also what tmux lets show: the line flag it
/// sets is sticky and `capture-pane -F` prints it, and the entries it allocates
/// are what `#{history_all_bytes}` prices.
///
/// Everything a cell that answers `false` carries fits the compact entry
/// exactly — one byte of text one column wide, an attribute byte, two palette
/// colours, the default underline colour and no link — so a compact round trip
/// gives the cell back unchanged.
pub fn needs_extended(cell: &Cell) -> bool {
    cell.attr > 0xff
        || cell.data.len() != 1
        || cell.data.width != 1
        || cell.fg & colour::FLAG_RGB != 0
        || cell.bg & colour::FLAG_RGB != 0
        || cell.us != colour::DEFAULT
        || cell.link != 0
        || cell.flags & flag::TAB != 0
}

/// tmux's `GRID_FLAG_*` bits that exist only in a stored entry. They say how
/// the entry is encoded, so they are set on the way in and stripped on the way
/// out; a [`Cell`] never carries one.
mod entry_flag {
    /// tmux's `GRID_FLAG_FG256`: the stored foreground byte is a palette index.
    pub const FG256: u8 = 0x1;
    /// tmux's `GRID_FLAG_BG256`.
    pub const BG256: u8 = 0x2;
    /// tmux's `GRID_FLAG_EXTENDED`: the payload is an index into the line's
    /// extended entries rather than the cell itself.
    pub const EXTENDED: u8 = 0x8;

    pub const ALL: u8 = FG256 | BG256 | EXTENDED;
}

/// tmux's `grid_cell_entry`: the five bytes a cell needing nothing special
/// occupies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Entry {
    /// tmux's union. With [`entry_flag::EXTENDED`] set these bytes are the
    /// index of the line's extended entry; otherwise they are the attribute
    /// byte, the foreground, the background and the cell's one byte of text.
    payload: [u8; 4],
    flags: u8,
}

impl Entry {
    /// tmux's `grid_cleared_entry`: what an erase leaves behind before the
    /// background is painted back over it.
    const CLEARED: Entry = Entry {
        payload: [0, 8, 8, b' '],
        flags: flag::CLEARED,
    };

    fn is_extended(self) -> bool {
        self.flags & entry_flag::EXTENDED != 0
    }

    fn index(self) -> usize {
        u32::from_ne_bytes(self.payload) as usize
    }

    /// tmux's `grid_store_cell`, for a cell [`needs_extended`] has cleared.
    fn store(cell: &Cell) -> Entry {
        let mut flags = cell.flags;
        if cell.fg & colour::FLAG_256 != 0 {
            flags |= entry_flag::FG256;
        }
        if cell.bg & colour::FLAG_256 != 0 {
            flags |= entry_flag::BG256;
        }
        Entry {
            payload: [
                cell.attr as u8,
                (cell.fg & 0xff) as u8,
                (cell.bg & 0xff) as u8,
                cell.data.bytes().first().copied().unwrap_or(b' '),
            ],
            flags,
        }
    }

    /// The compact half of tmux's `grid_get_cell1`.
    fn read(self) -> Cell {
        let mut fg = i32::from(self.payload[1]);
        if self.flags & entry_flag::FG256 != 0 {
            fg |= colour::FLAG_256;
        }
        let mut bg = i32::from(self.payload[2]);
        if self.flags & entry_flag::BG256 != 0 {
            bg |= colour::FLAG_256;
        }
        Cell {
            data: CellData::from_byte(self.payload[3]),
            attr: u16::from(self.payload[0]),
            flags: self.flags & !entry_flag::ALL,
            fg,
            bg,
            us: colour::DEFAULT,
            link: 0,
        }
    }
}

/// tmux's `grid_extd_entry`: a cell the compact entry cannot hold. The cluster
/// itself lives in the line's byte arena, which is what keeps this small.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Extended {
    cluster: u32,
    len: u8,
    width: u8,
    flags: u8,
    attr: u16,
    fg: i32,
    bg: i32,
    us: i32,
    link: u32,
}

/// tmux's `COLOUR_DEFAULT`: neither of the two spellings of "unset".
pub fn colour_is_default(value: i32) -> bool {
    value == 8 || value == 9
}

/// One cell as it sits in a row, for a reader that wants what is in it rather
/// than how it is drawn.
///
/// The text borrows the row's storage — the compact entry's own byte, or the
/// cluster arena — so reading a cell this way copies nothing.
pub struct CellView<'a> {
    pub text: &'a [u8],
    pub width: u8,
    pub flags: u8,
    pub link: u32,
}

impl CellView<'_> {
    pub fn is_padding(&self) -> bool {
        self.flags & flag::PADDING != 0
    }

    /// The cluster as text. Stored cluster bytes are always valid UTF-8,
    /// because they only ever arrive as an encoded `char`.
    pub fn text(&self) -> &str {
        std::str::from_utf8(self.text).unwrap_or("")
    }
}

/// One row, stored as tmux stores it: a compact entry per column, with the
/// cells too rich for one spilled into a second array.
///
/// The split is the whole reason this type owns its storage rather than holding
/// a `Vec<Cell>`. A [`Cell`] is 56 bytes because it carries an inline cluster
/// and full-width colours; an [`Entry`] is five. A screen of ordinary text is
/// the case that matters, and there every cell takes the compact form.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Line {
    /// The allocated cells. tmux's `cellsize` is this length.
    cells: Vec<Entry>,
    /// tmux's `extddata`. Monotonic while the line lives — tmux never reclaims
    /// an entry when a rich cell is overwritten with a simple one — and reset
    /// with the line.
    extd: Vec<Extended>,
    /// The cluster bytes the extended entries index. Written in place when a
    /// cell's new cluster fits where its old one was, and appended otherwise,
    /// so rewriting a cell repeatedly does not grow this without bound.
    clusters: Vec<u8>,
    /// tmux's `cellused`: how far a program has written into the row.
    used: usize,
    pub flags: u8,
    /// tmux's `gl->time`: the wall-clock second this row entered the history,
    /// or zero while it is still on screen. Only a scroll stamps it, so a
    /// resize that hands rows to the history leaves them unstamped, as tmux's
    /// `screen_resize_y` does.
    pub time: u64,
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
        self.extd.len()
    }

    pub fn is_wrapped(&self) -> bool {
        self.flags & line_flag::WRAPPED != 0
    }

    /// The cell at `px` read in place, or `None` past the allocated extent.
    ///
    /// This is what a reader that wants the text and how the cell is classified
    /// takes, rather than [`Line::cell`]: a snapshot of a deep history is
    /// millions of cells, and rebuilding a whole [`Cell`] for each one — with
    /// its inline cluster — costs more than reading the entry does.
    pub fn view(&self, px: usize) -> Option<CellView<'_>> {
        let entry = self.cells.get(px)?;
        if !entry.is_extended() {
            return Some(CellView {
                text: &entry.payload[3..4],
                width: 1,
                flags: entry.flags & !entry_flag::ALL,
                link: 0,
            });
        }
        let Some(extended) = self.extd.get(entry.index()) else {
            return Some(CellView {
                text: b" ",
                width: 1,
                flags: 0,
                link: 0,
            });
        };
        let start = extended.cluster as usize;
        let end = start + usize::from(extended.len);
        Some(CellView {
            text: self.clusters.get(start..end).unwrap_or_default(),
            width: extended.width,
            flags: extended.flags,
            link: extended.link,
        })
    }

    /// How the cell at `px` is shaped — its column width and its flags — or
    /// `None` past the allocated extent.
    ///
    /// This is the smallest read there is, and the one the write path makes:
    /// the padding scans around a collected run ask nothing else of a cell,
    /// once per character.
    fn shape(&self, px: usize) -> Option<(u8, u8)> {
        let entry = *self.cells.get(px)?;
        if !entry.is_extended() {
            return Some((1, entry.flags & !entry_flag::ALL));
        }
        Some(
            self.extd
                .get(entry.index())
                .map_or((1, 0), |extended| (extended.width, extended.flags)),
        )
    }

    /// The cell stored at `px`, or `None` past the allocated extent.
    ///
    /// A reader walking a row takes this directly, so that the row is looked up
    /// once rather than once per column.
    pub fn cell(&self, px: usize) -> Option<Cell> {
        let entry = *self.cells.get(px)?;
        if !entry.is_extended() {
            return Some(entry.read());
        }
        // tmux answers an index past the array with a default cell rather than
        // trusting it, and so does this.
        let Some(extended) = self.extd.get(entry.index()) else {
            return Some(Cell::DEFAULT);
        };
        let start = extended.cluster as usize;
        let end = start + usize::from(extended.len);
        let bytes = self.clusters.get(start..end).unwrap_or_default();
        Some(Cell {
            data: CellData::from_bytes(bytes, extended.width),
            attr: extended.attr,
            flags: extended.flags,
            fg: extended.fg,
            bg: extended.bg,
            us: extended.us,
            link: extended.link,
        })
    }

    /// Whether storing `cell` at `px` would leave the row exactly as it is, or
    /// `None` past the allocated extent.
    ///
    /// The comparison is between stored entries, which is both what tmux
    /// compares and the cheap way to ask: this runs once per printed
    /// character, and rebuilding the stored cell to compare fields would cost
    /// more than the write it is trying to avoid. An entry already extended
    /// never matches, as in tmux, because the flag is sticky.
    fn matches(&self, px: usize, cell: &Cell) -> Option<bool> {
        let entry = *self.cells.get(px)?;
        if entry.is_extended() || needs_extended(cell) {
            return Some(false);
        }
        Some(entry == Entry::store(cell))
    }

    /// tmux's `grid_set_cell` past its expansion: store `cell` at `px`, which
    /// must already be inside the allocated extent.
    ///
    /// `extended` is [`needs_extended`] for this cell, computed by the caller
    /// because it needs the answer too. It runs once per printed character, so
    /// asking twice is worth avoiding.
    fn set_cell(&mut self, px: usize, cell: &Cell, extended: bool) {
        let Some(slot) = self.cells.get_mut(px) else {
            return;
        };
        if px + 1 > self.used {
            self.used = px + 1;
        }
        if !extended && !slot.is_extended() {
            *slot = Entry::store(cell);
            return;
        }
        self.set_extended(px, cell);
    }

    /// tmux's `grid_extended_cell`: put `cell` in the line's extended array,
    /// reusing the entry this column already owns rather than allocating a
    /// second one.
    fn set_extended(&mut self, px: usize, cell: &Cell) {
        let entry = self.cells[px];
        let index = if entry.is_extended() {
            entry.index()
        } else {
            self.extd.push(Extended::default());
            self.extd.len() - 1
        };
        self.cells[px] = Entry {
            payload: (index as u32).to_ne_bytes(),
            flags: cell.flags | entry_flag::EXTENDED,
        };
        let (cluster, len) = self.store_cluster(index, cell.data.bytes());
        self.extd[index] = Extended {
            cluster,
            len,
            width: cell.data.width,
            flags: cell.flags,
            attr: cell.attr,
            fg: cell.fg,
            bg: cell.bg,
            us: cell.us,
            link: cell.link,
        };
    }

    /// Put a cluster in the arena, over the bytes the entry already holds when
    /// they are room enough.
    fn store_cluster(&mut self, index: usize, bytes: &[u8]) -> (u32, u8) {
        let existing = self.extd[index];
        let len = u8::try_from(bytes.len()).unwrap_or(u8::MAX);
        if bytes.len() <= usize::from(existing.len) {
            let start = existing.cluster as usize;
            self.clusters[start..start + bytes.len()].copy_from_slice(bytes);
            return (existing.cluster, len);
        }
        let start = u32::try_from(self.clusters.len()).unwrap_or(u32::MAX);
        self.clusters.extend_from_slice(bytes);
        (start, len)
    }

    /// tmux's `grid_clear_cell`: the cleared entry, with `bg` painted back into
    /// it. A direct-colour background does not fit the compact entry, which is
    /// why an erase to one allocates.
    fn clear_cell(&mut self, px: usize, bg: i32) {
        if px >= self.cells.len() {
            return;
        }
        self.cells[px] = Entry::CLEARED;
        if bg == colour::DEFAULT {
            return;
        }
        if bg & colour::FLAG_RGB != 0 {
            self.set_extended(px, &Cell::cleared(bg));
            return;
        }
        if bg & colour::FLAG_256 != 0 {
            self.cells[px].flags |= entry_flag::BG256;
        }
        self.cells[px].payload[2] = (bg & 0xff) as u8;
    }

    /// Grow the allocated extent, clearing what that brings into existence.
    fn grow(&mut self, sx: usize, bg: i32) {
        let from = self.cells.len();
        self.cells.resize(sx, Entry::CLEARED);
        if bg == colour::DEFAULT {
            return;
        }
        for px in from..sx {
            self.clear_cell(px, bg);
        }
    }

    /// The entry half of tmux's `grid_move_cells`: shift stored entries within
    /// the row.
    ///
    /// The entries move as they stand, extended ones included. An extended
    /// entry indexes this same line, so the index it carries stays right where
    /// it lands — which is why a shift allocates nothing, as tmux's `memmove`
    /// allocates nothing.
    fn move_cells(&mut self, dx: usize, px: usize, nx: usize) {
        if px + nx > self.cells.len() || dx + nx > self.cells.len() {
            return;
        }
        self.cells.copy_within(px..px + nx, dx);
        if dx + nx > self.used {
            self.used = dx + nx;
        }
    }

    /// Append a cell, as a reflow laying a logical line out again does.
    fn push_cell(&mut self, cell: &Cell) {
        self.cells.push(Entry::CLEARED);
        let px = self.cells.len() - 1;
        self.set_cell(px, cell, needs_extended(cell));
    }

    /// The cells the row holds, in column order.
    fn iter(&self) -> impl Iterator<Item = Cell> + '_ {
        (0..self.cells.len()).filter_map(|px| self.cell(px))
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
    /// The cell is rebuilt from the row's storage rather than borrowed from it,
    /// because a compact entry is not a whole cell.
    pub fn get(&self, px: usize, py: usize) -> Cell {
        self.peek(px, py).unwrap_or(Cell::DEFAULT)
    }

    /// The cell as it is actually stored, or `None` past the row's allocated
    /// extent. Unlike [`Grid::get`] this tells "a cell holding a default" apart
    /// from "no cell at all", which is a distinction `screen_write_cell` makes.
    pub fn peek(&self, px: usize, py: usize) -> Option<Cell> {
        self.lines.get(py).and_then(|line| line.cell(px))
    }

    /// Whether writing `cell` at `px`/`py` would change nothing, or `None`
    /// past the row's allocated extent; see [`Line::matches`].
    pub fn matches(&self, px: usize, py: usize, cell: &Cell) -> Option<bool> {
        self.lines.get(py).and_then(|line| line.matches(px, cell))
    }

    /// Whether the stored cell is the blank right half of a wide character.
    /// Read in place, since the scans that ask walk a row a column at a time.
    pub fn is_padding(&self, px: usize, py: usize) -> bool {
        self.lines
            .get(py)
            .and_then(|line| line.shape(px))
            .is_some_and(|(_, flags)| flags & flag::PADDING != 0)
    }

    /// How many columns the stored cell occupies, read in place. A read past
    /// the allocated extent answers one, as the default cell is one column.
    pub fn cell_width(&self, px: usize, py: usize) -> u8 {
        self.lines
            .get(py)
            .and_then(|line| line.shape(px))
            .map_or(1, |(width, _)| width)
    }

    /// The stored cell's flags, read in place.
    pub fn cell_flags(&self, px: usize, py: usize) -> u8 {
        self.lines
            .get(py)
            .and_then(|line| line.shape(px))
            .map_or(0, |(_, flags)| flags)
    }

    /// tmux's `grid_set_cell`.
    pub fn set(&mut self, px: usize, py: usize, cell: &Cell) {
        if py >= self.lines.len() {
            return;
        }
        self.expand_line(py, px + 1, colour::DEFAULT);
        let extended = needs_extended(cell);
        let line = &mut self.lines[py];
        if cell.link != 0 {
            line.flags |= line_flag::HYPERLINK;
        }
        if extended {
            line.flags |= line_flag::EXTENDED;
        }
        line.set_cell(px, cell, extended);
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
        if sx <= line.size() {
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
        if sx <= line.size() {
            return;
        }
        line.grow(sx, bg);
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
            let sx = self.sx.min(self.lines[yy].size());
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
            line.clear_cell(px, bg);
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
        self.lines[py].move_cells(dx, px, nx);
        for xx in px..px + nx {
            if xx < dx || xx >= dx + nx {
                self.clear_cell(xx, py, bg);
            }
        }
    }

    /// tmux's `grid_scroll_history`: the top visible row becomes history and a
    /// fresh row appears at the bottom.
    ///
    /// `now` is the second the promoted row is stamped with, tmux's
    /// `current_time`.
    pub fn scroll_history(&mut self, bg: i32, now: u64) {
        self.lines.push(Line::default());
        let last = self.lines.len() - 1;
        self.empty_line(last, bg);
        self.hscrolled += 1;
        self.lines[self.hsize].time = now;
        self.hsize += 1;
    }

    /// tmux's `grid_scroll_history_region`: scrolling a region whose top is the
    /// top of the screen still feeds the history, unlike one further down.
    pub fn scroll_history_region(&mut self, upper: usize, lower: usize, bg: i32, now: u64) {
        self.lines.insert(self.hsize, Line::default());
        // The region moved down by one with the insert; take its first row into
        // the history slot and close the gap behind it.
        let upper = self.hsize + upper + 1;
        let lower = self.hsize + lower + 1;
        let promoted = self.lines.remove(upper);
        self.lines[self.hsize] = promoted;
        self.lines[self.hsize].time = now;
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
    pub fn view_clear_history(&mut self, bg: i32, now: u64) {
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
            self.scroll_history(bg, now);
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
        let mut px = line.size().min(self.sx);
        while px > 0 {
            let Some(cell) = line.cell(px - 1) else { break };
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
        // The history stamp belongs to the logical line, so it is the one its
        // first row carried; the rows a wrap split it into are the same line.
        let mut carried_time = None;

        let lines = std::mem::take(&mut self.lines);
        for line in lines {
            // The flags that belong to the logical line, not to the row the
            // old width happened to split it into.
            carried_flags |= line.flags & !line_flag::WRAPPED;
            carried_time.get_or_insert(line.time);
            logical.extend(line.iter().take(line.used()));
            if line.flags & line_flag::WRAPPED != 0 {
                continue;
            }
            lay_out(
                &mut rewrapped,
                std::mem::take(&mut logical),
                sx,
                carried_flags,
                carried_time.take().unwrap_or(0),
            );
            carried_flags = 0;
        }
        // A trailing run that never met an unwrapped row still has to land.
        if !logical.is_empty() {
            lay_out(
                &mut rewrapped,
                logical,
                sx,
                carried_flags,
                carried_time.unwrap_or(0),
            );
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
    fn a_rewrap_keeps_the_logical_lines_history_stamp() {
        let mut grid = Grid::new(4, 2, 100);
        for (px, ch) in "abcd".chars().enumerate() {
            grid.set(
                px,
                0,
                &Cell {
                    data: CellData::from_char(ch, 1),
                    ..Cell::default()
                },
            );
        }
        grid.scroll_history(colour::DEFAULT, 1_700_000_000);
        grid.reflow(2);
        assert_eq!(written(&grid, 0), "ab");
        assert_eq!(written(&grid, 1), "cd");
        assert_eq!(
            grid.line(0).map(|line| line.time),
            Some(1_700_000_000),
            "the row the split leaves the line starting on keeps the stamp"
        );
        assert_eq!(
            grid.line(1).map(|line| line.time),
            Some(0),
            "the row the split added carries none"
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
        grid.scroll_history(colour::DEFAULT, 1_700_000_000);
        assert_eq!(grid.hsize, 1);
        assert_eq!(grid.total(), 3);
        assert_eq!(written(&grid, 0), "a", "the row is now history row zero");
        assert_eq!(
            grid.line(0).map(|line| line.time),
            Some(1_700_000_000),
            "the promoted row is stamped"
        );
        assert_eq!(
            grid.line(1).map(|line| line.time),
            Some(0),
            "a row still on screen carries no stamp"
        );
    }

    #[test]
    fn history_is_collected_a_tenth_at_a_time_once_over_the_limit() {
        let mut grid = Grid::new(10, 1, 10);
        for _ in 0..10 {
            grid.scroll_history(colour::DEFAULT, 0);
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
        grid.scroll_history_region(0, 2, colour::DEFAULT, 1_700_000_000);
        assert_eq!(grid.hsize, 1);
        assert_eq!(written(&grid, 0), "a", "the region's top row is history");
        assert_eq!(
            grid.line(0).map(|line| line.time),
            Some(1_700_000_000),
            "the promoted row is stamped"
        );
        assert_eq!(written(&grid, 1), "b");
        assert_eq!(written(&grid, 2), "c");
        assert_eq!(written(&grid, 3), "", "the region's bottom row is blank");
        assert_eq!(written(&grid, 4), "d", "outside the region, untouched");
    }
}
