//! The pane's grid and the types it is read back as.
//!
//! The screen itself is [`PaneScreen`], re-exported here; the rest of the
//! module holds the values its reads produce and the options it consults. They are shaped by what the
//! daemon consumes: the bytes come from the tokenizer ([`super::observer`]),
//! and the grid, the scrollback and the serializations of both come from the
//! screen.
//!
//! Stability: this is deliberately `pub`. It is an implementation surface, not
//! a compatibility contract, and the daemon's contract is its tmux-compatible
//! command line and wire protocol.

use std::sync::Arc;

use super::sixel::SixelImage;

pub use crate::engine::backend::PaneScreen;

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
    pub(crate) const CRLF: u32 = 0x4000;
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
    pub(crate) const ALL_MOUSE: u32 = MOUSE_STANDARD | MOUSE_BUTTON | MOUSE_ALL;
    /// tmux's `EXTENDED_KEY_MODES`.
    pub(crate) const ALL_KEYS_EXTENDED: u32 = KEYS_EXTENDED | KEYS_EXTENDED_2;
}

/// How many display cells a cell's content occupies.
///
/// A wide character occupies its own [`Wide`](CellWidth::Wide) cell plus a
/// following [`SpacerTail`](CellWidth::SpacerTail).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellWidth {
    Narrow,
    Wide,
    SpacerTail,
}

/// The shell-integration classification of a cell's content.
///
/// tmux classifies rows with `GRID_LINE_START_PROMPT` and
/// `GRID_LINE_START_OUTPUT` alone, so every cell is one of these two.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellSemantic {
    Output,
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
    /// `capture-pane` output can be narrower than the columns it covers.
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
/// [`OutputPolicy`](super::observer::OutputPolicy) has for the tokenizer, and
/// it is deliberately a separate one: those options decide how a pane's bytes
/// are *parsed*, these decide what an operation does to the grid.
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

/// A screen materialized in one walk: the structured grid and its
/// `capture-pane -e` serialization together.
///
/// Two screens come back this way — the one the alternate-screen switch
/// displaced ([`PaneScreen::inactive_snapshot`]) and the one copy mode
/// freezes. Neither is the live grid, and their consumers want both forms of
/// the same walk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenSnapshot {
    pub grid: Grid,
    pub vt: Vec<u8>,
}

/// How far along a row a VT serialization runs.
///
/// tmux keeps these apart and so do we: `grid_string_cells`, which is what
/// `capture-pane -e` uses, runs to one of the row's two extents, while the tty
/// redraw paints the row and erases what follows. Forcing both through one
/// extent loses either a written trailing space (capture) or everything past
/// the last non-blank cell on a row whose tail is not blank — a popup's closing
/// border, for instance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RowExtent {
    /// Up to the last non-blank cell; the caller erases the rest.
    Redraw,
    /// A capture, running to the extent `capture-pane`'s flags selected.
    Capture(CaptureExtent),
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
    /// which `#{history_all_bytes}` counts.
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
    /// for its own terminal. The codec's representation does not leave the
    /// crate; [`ScreenImage::sixel_for_client`] is what a compositor reads.
    pub(crate) data: Arc<SixelImage>,
    /// The text a client that cannot draw sixel gets instead, already laid out
    /// as tmux lays it out — one `\r\n`-terminated line per image row.
    pub fallback: Arc<str>,
}

impl ScreenImage {
    /// tmux's `tty_cmd_sixelimage`: the sixel bytes a client draws for the part
    /// of this image it can see.
    ///
    /// `origin` and `size` are in *image* cells — the crop `tty_clamp_area`
    /// works out — and `cell_pixels` is the client terminal's own cell size.
    /// The crop is resampled onto that geometry and re-serialized against the
    /// original's palette, which is why the rescale itself carries no colour
    /// registers. `None` when the crop lands outside the image or the image
    /// never selected a colour, both of which are tmux's NULL.
    ///
    /// A client that cannot draw sixel writes [`Self::fallback`] instead.
    #[must_use]
    pub fn sixel_for_client(
        &self,
        cell_pixels: (u16, u16),
        origin: (u16, u16),
        size: (u16, u16),
    ) -> Option<Vec<u8>> {
        self.data
            .scale(
                u32::from(cell_pixels.0),
                u32::from(cell_pixels.1),
                u32::from(origin.0),
                u32::from(origin.1),
                u32::from(size.0),
                u32::from(size.1),
                false,
            )?
            .print(Some(&self.data))
    }
}

/// Row geometry of the active screen, read without walking any cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridDims {
    pub cols: u16,
    pub viewport_rows: u16,
    /// How many rows of history sit above the viewport. This is where a caller
    /// that only wants the history count gets it.
    pub scrollback_rows: usize,
    pub total_rows: usize,
}
