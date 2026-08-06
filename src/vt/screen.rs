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
//! Stability: this is deliberately `pub(crate)`. It is an implementation seam
//! being carved, not a compatibility contract, and the daemon's contract is its
//! tmux-compatible command line and wire protocol.

use std::io;

use super::parser::Token;

/// Ghostty-style display-cell width classification.
///
/// A wide character occupies its own [`Wide`](CellWidth::Wide) cell plus a
/// following [`SpacerTail`](CellWidth::SpacerTail); a
/// [`SpacerHead`](CellWidth::SpacerHead) is the blank left at a right margin
/// too narrow for the wide character that had to wrap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CellWidth {
    Narrow,
    Wide,
    SpacerTail,
    SpacerHead,
}

/// The shell-integration classification of a cell's content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CellSemantic {
    Output,
    Input,
    Prompt,
}

/// An immutable copy of one terminal cell.
///
/// `text` holds the complete grapheme cluster in the cell. An empty cell has an
/// empty string; that intentionally distinguishes it from a literal U+0020 cell
/// without claiming how the empty cell was produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GridCell {
    pub(crate) text: String,
    pub(crate) width: CellWidth,
    pub(crate) semantic: CellSemantic,
    pub(crate) hyperlink: Option<String>,
    /// The OSC 8 `id=` the program set explicitly, if any. Implicit ids the
    /// emulator invents for its own bookkeeping are not reported.
    pub(crate) hyperlink_id: Option<String>,
}

/// One physical row of a grid snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GridRow {
    pub(crate) cells: Vec<GridCell>,
    /// Whether this physical row soft-wraps into the following one.
    pub(crate) wrapped: bool,
}

/// An immutable copy of the active screen, scrollback first, viewport last.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Grid {
    pub(crate) cols: u16,
    pub(crate) viewport_rows: u16,
    pub(crate) scrollback_rows: usize,
    pub(crate) rows: Vec<GridRow>,
}

/// Row geometry of the active screen, read without walking any cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GridDims {
    pub(crate) cols: u16,
    pub(crate) viewport_rows: u16,
    pub(crate) scrollback_rows: usize,
    pub(crate) total_rows: usize,
}

/// The pane's screen: the grid, its scrollback, and the ways the daemon reads
/// them back out.
///
/// Row indices are *physical*: zero is the oldest scrollback row and
/// [`GridDims::scrollback_rows`] is the first visible one. Cursor coordinates
/// are zero-based and viewport-relative.
pub(crate) trait VtScreen {
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
}
