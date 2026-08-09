//! The screen seam: what the server needs a terminal emulator to be.
//!
//! Everything above this module addresses the pane's grid through
//! [`VtScreen`], never through a particular emulator. The bytes come from the
//! tokenizer ([`super::observer`]); the grid, the scrollback and the
//! serializations of both come from here.
//!
//! The trait is shaped by what the daemon consumes, not by what any one
//! emulator exposes: a backend whose library hides some of this owes the
//! reconstruction, rather than leaving the server to recover it. The types
//! below belong to hmux for the same reason — a backend converts into them.
//!
//! Stability: this is deliberately `pub`. It is an implementation seam
//! being carved, not a compatibility contract, and the daemon's contract is its
//! tmux-compatible command line and wire protocol.

use std::io;
use std::sync::Arc;

use super::parser::Token;
use super::sixel::SixelImage;

/// tmux's `MODE_*`, the bits `screen->mode` carries.
///
/// This is the whole word, not the part that steers the grid. Modes nothing in
/// the engine reads — bracketed paste, focus reporting — still belong here: they
/// are the pane's state, one sequence sets and clears them, and a second copy
/// beside this one is a second implementation of the same DECSET semantics with
/// its own chances to disagree.
pub mod mode {
    pub const CURSOR: u32 = 0x1;
    pub const INSERT: u32 = 0x2;
    pub const KCURSOR: u32 = 0x4;
    pub const KKEYPAD: u32 = 0x8;
    pub const WRAP: u32 = 0x10;
    /// DECSET 1000: presses and releases.
    pub const MOUSE_STANDARD: u32 = 0x20;
    /// DECSET 1002: adds motion while a button is held.
    pub const MOUSE_BUTTON: u32 = 0x40;
    /// DECSET 12, and a side effect of every DECSCUSR style but the default.
    pub const CURSOR_BLINKING: u32 = 0x80;
    /// DECSET 1005: UTF-8 coordinate encoding.
    pub const MOUSE_UTF8: u32 = 0x100;
    /// DECSET 1006: SGR encoding.
    pub const MOUSE_SGR: u32 = 0x200;
    /// DECSET 2004: the pane wants the paste markers.
    pub const BRACKETPASTE: u32 = 0x400;
    /// DECSET 1004: the pane asked to be told when focus moves.
    pub const FOCUSON: u32 = 0x800;
    /// DECSET 1003: adds button-less motion.
    pub const MOUSE_ALL: u32 = 0x1000;
    pub const ORIGIN: u32 = 0x2000;
    pub const CRLF: u32 = 0x4000;
    /// `CSI > 4 ; 1 m`: the `modifyOtherKeys` level the pane asked for. What it
    /// gets also depends on `extended-keys`, which hmux applies where the key
    /// is encoded rather than here — so these two bits are the pane's request,
    /// where tmux's are the request already mixed with the option.
    pub const KEYS_EXTENDED: u32 = 0x8000;
    /// `CSI > 4 ; 2 m`: the same request, at the level that reports every key.
    pub const KEYS_EXTENDED_2: u32 = 0x4_0000;
    /// DECSET 2031: the pane asked to be told when the theme changes.
    pub const THEME_UPDATES: u32 = 0x8_0000;
    /// DECSET 2026: the pane asked for its output to be held back until it says
    /// the frame is done.
    pub const SYNC: u32 = 0x10_0000;
    /// The pane has spoken about the cursor blink itself, so a query is
    /// answered from [`CURSOR_BLINKING`] rather than from `cursor-style`.
    pub const CURSOR_BLINKING_SET: u32 = 0x2_0000;

    /// tmux's `ALL_MOUSE_MODES`: the program asked for reports at all.
    pub const ALL_MOUSE: u32 = MOUSE_STANDARD | MOUSE_BUTTON | MOUSE_ALL;
    /// tmux's `EXTENDED_KEY_MODES`.
    pub const ALL_KEYS_EXTENDED: u32 = KEYS_EXTENDED | KEYS_EXTENDED_2;
}

/// Ghostty-style display-cell width classification.
///
/// A wide character occupies its own [`Wide`](CellWidth::Wide) cell plus a
/// following [`SpacerTail`](CellWidth::SpacerTail); a
/// [`SpacerHead`](CellWidth::SpacerHead) is the blank left at a right margin
/// too narrow for the wide character that had to wrap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellWidth {
    Narrow,
    Wide,
    SpacerTail,
    /// A backend may report this; tmux's grid has no equivalent, so the
    /// engine never produces one.
    SpacerHead,
}

/// The shell-integration classification of a cell's content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellSemantic {
    Output,
    /// A backend may report this. tmux classifies rows with
    /// `GRID_LINE_START_PROMPT` and `GRID_LINE_START_OUTPUT` alone, so under
    /// the engine every row is one of the other two.
    Input,
    Prompt,
}

/// An immutable copy of one terminal cell.
///
/// `text` holds the complete grapheme cluster in the cell. An empty cell has an
/// empty string; that intentionally distinguishes it from a literal U+0020 cell
/// without claiming how the empty cell was produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GridCell {
    pub text: String,
    pub width: CellWidth,
    pub semantic: CellSemantic,
    pub hyperlink: Option<String>,
    /// The OSC 8 `id=` the program set explicitly, if any. Implicit ids the
    /// emulator invents for its own bookkeeping are not reported.
    pub hyperlink_id: Option<String>,
    /// Which link this cell belongs to, as the screen's own identity for it —
    /// tmux's *inner* id, the number a `grid_cell` carries in `link`. Zero
    /// means the cell is not in a link.
    ///
    /// Identity is not the URI: every anonymous OSC 8 opens a fresh link even
    /// when it names an address already on screen, because two mentions of one
    /// address are two links. A caller that has to count or de-duplicate links
    /// — `capture-pane -H` does both — compares this, not
    /// [`Self::hyperlink`]. The number is meaningful only within one screen and
    /// only until it is reset.
    pub hyperlink_slot: u32,
    /// Whether this cell stands for the run of columns a horizontal tab
    /// created, tmux's `GRID_FLAG_TAB`.
    ///
    /// The cell holds blanks so that the grid renders as the program intended,
    /// but a text read of the row puts the tab back: tmux's
    /// `grid_string_cells` emits a single `\t` for such a cell, which is why
    /// `capture-pane` output can be narrower than the columns it covers. A
    /// backend that does not keep the tab's origin reports `false` and the tab
    /// reads back as the spaces it painted.
    pub tab: bool,
}

/// The tmux `GRID_LINE_*` flags of one row, which `capture-pane -F` reports.
///
/// [`GridRow::wrapped`] is the sixth and is kept separate because everything
/// reads it. `GRID_LINE_DEAD` has no field: it marks a placeholder left behind
/// mid-reflow, and no grid a caller can observe holds one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RowFlags {
    /// Some cell on the row belongs to an OSC 8 hyperlink.
    pub hyperlink: bool,
    /// Command output starts on this row (OSC 133 C).
    pub start_output: bool,
    /// A shell-integration prompt starts on this row (OSC 133 A).
    pub start_prompt: bool,
    /// The row has held a cell needing tmux's extended representation — a wide
    /// or multi-byte character, an RGB colour, an underline colour or a link.
    /// It is sticky, as in tmux: erasing the cell does not clear the flag.
    pub extended: bool,
}

/// The pane options the screen itself has to consult.
///
/// tmux reads these out of `wp->options` inside the operation that needs them,
/// with the whole server in reach. hmux's screen runs away from server state,
/// so the resolved values are pushed to it instead and re-pushed whenever they
/// can have changed. That is the same shape
/// [`OutputPolicy`](super::observer::OutputPolicy) has for the
/// tokenizer, and it is deliberately a separate one: those options decide how a
/// pane's bytes are *parsed*, these decide what an operation does to the grid,
/// and a backend that implements one need not implement the other.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScreenOptions {
    /// `scroll-on-clear`: whether clearing the screen moves what was on it into
    /// the history instead of blanking it where it stands.
    pub scroll_on_clear: bool,
    /// The window's cell size in pixels, tmux's `w->xpixel`/`w->ypixel`.
    ///
    /// Not an option — it comes from the attached clients' terminals, which
    /// `recalculate_sizes` aggregates onto the window — but it reaches the
    /// screen the same way an option does and for the same reason: a sixel
    /// payload is measured in pixels and has to be converted into cells at the
    /// moment it is parsed, with no server state in reach.
    pub xpixel: u32,
    pub ypixel: u32,
}

impl Default for ScreenOptions {
    /// tmux's defaults, which is what a screen uses until the server says
    /// otherwise.
    fn default() -> Self {
        ScreenOptions {
            scroll_on_clear: true,
            // tmux's `DEFAULT_XPIXEL` and `DEFAULT_YPIXEL`, which stand in
            // whenever no attached client reports a pixel size.
            xpixel: 16,
            ypixel: 32,
        }
    }
}

/// How far along a row `capture-pane` reads.
///
/// tmux's `grid_string_cells` takes this as a flag bit, and the two extents are
/// genuinely different boundaries rather than one with a tidier spelling. The
/// choice is visible in `-e` output: a capture that runs to the allocated
/// extent crosses the blank cells past what a program wrote, and the style
/// transition into those blanks outlives the trailing-space trim that removes
/// the blanks themselves.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureExtent {
    /// tmux's `cellsize`, which is what a capture reads unless asked otherwise.
    Allocated,
    /// tmux's `cellused`, the written extent, which `-J` and `-T` ask for.
    Written,
}

/// One physical row of a grid snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GridRow {
    pub cells: Vec<GridCell>,
    /// Whether this physical row soft-wraps into the following one.
    pub wrapped: bool,
    /// tmux's `cellused`: how far into the row a program has written.
    ///
    /// This cannot be recovered from the cells. A cell erased inside the
    /// written extent is indistinguishable from one never touched, so a reader
    /// scanning back from the right for content finds a different boundary than
    /// tmux does — which is exactly what `capture-pane -J`/`-T` stop at.
    pub used: usize,
    /// tmux's `cellsize`: the row's allocated extent, where a capture that
    /// keeps empty cells (the default, and `-N`) stops.
    pub size: usize,
    /// tmux's `extdsize`: how many extended entries the row has allocated,
    /// which `#{history_all_bytes}` counts. A backend with no allocation
    /// model behind its rows reports zero.
    pub extd: usize,
    pub flags: RowFlags,
}

/// An immutable copy of the active screen, scrollback first, viewport last.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid {
    pub cols: u16,
    pub viewport_rows: u16,
    pub scrollback_rows: usize,
    pub rows: Vec<GridRow>,
}

/// One image the screen holds, as a client's frame needs it.
///
/// An image is not made of cells, so it cannot come back inside [`Grid`]: it is
/// anchored to a cell and drawn over whatever is there. The compositor reads
/// these after it has painted a pane's text, which is where tmux's
/// `tty_draw_images` sits.
#[derive(Clone, Debug)]
pub struct ScreenImage {
    /// The anchor cell, in viewport coordinates.
    pub px: u16,
    pub py: u16,
    /// The size in cells.
    pub sx: u16,
    pub sy: u16,
    /// The screen's image-list revision when this was read. It changes whenever
    /// the set of images does, so a renderer can skip re-deriving bytes it has
    /// already sent without comparing the images themselves.
    pub revision: u64,
    /// The decoded image, shared: every attached client re-scales the same one
    /// for its own terminal.
    pub data: Arc<SixelImage>,
    /// The text a client that cannot draw sixel gets instead, already laid out
    /// as tmux lays it out — one `\r\n`-terminated line per image row.
    pub fallback: Arc<str>,
}

/// Row geometry of the active screen, read without walking any cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridDims {
    pub cols: u16,
    pub viewport_rows: u16,
    pub scrollback_rows: usize,
    pub total_rows: usize,
}

/// The pane's screen: the grid, its scrollback, and the ways the daemon reads
/// them back out.
///
/// Row indices are *physical*: zero is the oldest scrollback row and
/// [`GridDims::scrollback_rows`] is the first visible one. Cursor coordinates
/// are zero-based and viewport-relative.
///
/// # Stability
///
/// This trait is open — anyone may implement it — and it is a versioned
/// compatibility contract: it does not change without a major version bump of
/// `hmux-vt`, which while the crate is `0.x` means the minor. Adding a required
/// method breaks every implementor, so new capability lands as a new trait or
/// as inherent methods on the backends; a method with a default body is the one
/// semver-minor escape hatch, and is used sparingly.
pub trait VtScreen {
    /// Apply one parsed token.
    ///
    /// The seam carries tokens rather than bytes because the pane's stream is
    /// parsed once, upstream. A backend that wraps another emulator hands
    /// [`Token::raw`] straight on; the in-house engine reads the token itself
    /// and never looks at the bytes again.
    ///
    /// This never fails: malformed input has already been resolved by the
    /// parser, and a sequence the screen does not implement is ignored, not an
    /// error.
    fn apply(&mut self, token: &Token);

    /// Resize the grid. Both dimensions are clamped to at least one.
    fn resize(&mut self, cols: u16, rows: u16) -> io::Result<()>;

    /// Apply the pane options the screen consults; see [`ScreenOptions`].
    fn set_options(&mut self, options: ScreenOptions);

    /// Apply a pane's history limit to the retained primary-screen scrollback.
    ///
    /// The server owns this option, while the screen owns the rows it trims;
    /// keeping the operation here avoids reconstructing a grid at the command
    /// layer. A backend without row-level history control may leave this as a
    /// no-op and document that limitation.
    fn set_history_limit(&mut self, limit: usize);

    /// The pane's VT modes, as [`mode`]'s bits.
    ///
    /// Every mode a pane can set lives here, not only the ones that steer the
    /// grid, because one sequence writes a mode and one word should hold it.
    /// The server reads this rather than counting DECSETs beside the screen.
    fn modes(&self) -> u32;

    /// `resize-pane -T`: drop the rows below the cursor and pull the same
    /// number of rows out of the history into the viewport, so the cursor's
    /// line ends up at the bottom of the screen with its content intact.
    ///
    /// This is tmux's one operation on history that is neither a write nor a
    /// read, and a backend that does not own its scrollback cannot offer it.
    fn trim_history_below_cursor(&mut self) -> io::Result<()>;

    /// The cursor, zero-based and relative to the viewport.
    fn cursor_position(&self) -> io::Result<(u16, u16)>;

    /// Whether the cursor is visible (DECTCEM).
    ///
    /// A full-screen application typically hides the hardware cursor and paints
    /// its own as a styled cell. [`Self::dump_vt`] carries the cursor's
    /// *position* but not this, so the compositor has to ask separately and
    /// mirror the answer onto the client tty — otherwise the client's real
    /// cursor stays lit on top of the painted one.
    fn cursor_visible(&self) -> io::Result<bool>;

    /// How many rows of history sit above the viewport.
    fn scrollback_rows(&self) -> io::Result<usize>;

    /// Row geometry, without walking any cells.
    fn grid_dims(&self) -> io::Result<GridDims>;

    /// Snapshot every physical cell and row.
    fn grid_snapshot(&self) -> io::Result<Grid>;

    /// The images anchored to the *visible* screen, oldest first.
    ///
    /// Separate from [`Self::grid_snapshot`] because an image is not cells: it
    /// covers them rather than being one of them, and a caller that only wants
    /// the text should not pay to carry pixels around. A backend with no image
    /// support answers with an empty list.
    fn images(&self) -> Vec<ScreenImage>;

    /// The *inactive* screen — the one the alternate-screen switch displaced —
    /// as its snapshot and its `capture-pane -e` serialization, or `None` when
    /// no alternate screen is in use.
    ///
    /// tmux keeps it as `saved_grid` and `capture-pane -a` is the only thing
    /// that reads it. Both forms come back together because a capture picks one
    /// of them and the grid is walked either way; splitting them would mean
    /// walking it twice or asking a backend to hold a cursor into a screen that
    /// has none.
    fn inactive_snapshot(&self) -> io::Result<Option<(Grid, Vec<u8>)>>;

    /// Snapshot only physical rows `[start, start + count)`, clamped to the
    /// grid. The per-cell walk dominates the cost of a snapshot, so a consumer
    /// with a known row range pays for that range alone. The returned
    /// dimensions still describe the whole grid; `rows[0]` is physical row
    /// `start`.
    fn grid_snapshot_range(&self, start: usize, count: usize) -> io::Result<Grid>;

    /// The whole screen as plain text, trailing whitespace trimmed.
    fn dump_plain(&self) -> io::Result<String>;

    /// As [`Self::dump_plain`], but with rows split only by a right-margin soft
    /// wrap rejoined into one logical line.
    fn dump_plain_unwrapped(&self) -> io::Result<String>;

    /// Plain text for physical rows `[start, start + rows)` alone.
    fn dump_plain_rows(&self, start: usize, rows: usize, cols: u16) -> io::Result<String>;

    /// The whole screen as VT escape sequences, ready to write to a client tty:
    /// text, SGR styles, hyperlinks and a final cursor position.
    fn dump_vt(&self) -> io::Result<Vec<u8>>;

    /// VT bytes for physical rows `[start, start + rows)` alone.
    fn dump_vt_rows(&self, start: usize, rows: usize, cols: u16) -> io::Result<Vec<u8>>;

    /// As [`Self::dump_vt_rows`], but for `capture-pane -e` rather than for a
    /// client's tty.
    ///
    /// The two are not the same read. A capture runs to one of the row's two
    /// extents, so a space a program wrote — perhaps carrying a background
    /// colour — is part of the captured row. A redraw stops at the last
    /// non-blank cell and erases the rest, which is cheaper and is what the
    /// compositor wants. tmux keeps the same two paths apart.
    fn dump_vt_capture_rows(
        &self,
        start: usize,
        rows: usize,
        cols: u16,
        extent: CaptureExtent,
    ) -> io::Result<Vec<u8>>;
}
