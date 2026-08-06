//! Reading the screen back out: snapshots, plain text, and VT.
//!
//! Three readers, three consumers. The snapshot is what `capture-pane` and
//! copy mode walk cell by cell; the plain text is what a reader of pane output
//! wants; the VT serialization is what the compositor writes to a client's tty
//! on attach and repaint.
//!
//! The VT form is modelled on tmux's redraw path rather than on any one
//! formatter: each row is positioned absolutely and drawn with the styles its
//! cells carry, and every style and hyperlink a row opens it also closes. The
//! compositor splits a dump on rows and paints each one where it belongs, so a
//! style left open would bleed into whatever is drawn next — including another
//! pane.

use std::fmt::Write as _;

use super::cell::{attr, colour, flag, Cell};
use super::grid::Grid as EngineGrid;
use super::screen::Screen;
use crate::vt::screen::mode;
use crate::vt::screen::{
    CaptureExtent, CellSemantic, CellWidth, Grid, GridCell, GridRow, RowFlags,
};

use super::grid::line_flag;

/// Snapshot physical rows `[start, start + count)`, clamped to the grid.
pub(crate) fn snapshot(screen: &Screen, start: usize, count: usize) -> Grid {
    snapshot_grid(screen, &screen.grid, start, count)
}

/// As [`snapshot`], but of a grid other than the screen's own — the one the
/// alternate-screen switch displaced, which `capture-pane -a` reads.
///
/// The screen is still what resolves a cell's link, as in tmux: `-a` swaps the
/// grid it walks but keeps reading `wp->base`'s hyperlink table.
pub(crate) fn snapshot_grid(
    screen: &Screen,
    grid: &EngineGrid,
    start: usize,
    count: usize,
) -> Grid {
    let total = grid.total();
    let start = start.min(total);
    let end = start.saturating_add(count).min(total);
    let mut rows = Vec::with_capacity(end - start);
    for py in start..end {
        let line = grid.line(py);
        let wrapped = line.is_some_and(super::grid::Line::is_wrapped);
        let semantic = row_semantic(grid, py);
        // Past the row's written extent nothing has been put there at all,
        // whatever the cell store happens to hold.
        let used = line.map_or(0, super::grid::Line::used);
        let cells = (0..grid.sx)
            .map(|px| snapshot_cell(screen, &grid.get(px, py), semantic, px < used))
            .collect();
        rows.push(GridRow {
            cells,
            wrapped,
            used,
            size: line.map_or(0, super::grid::Line::size),
            extd: line.map_or(0, super::grid::Line::extd),
            flags: row_flags(line),
        });
    }
    Grid {
        cols: u16::try_from(grid.sx).unwrap_or(u16::MAX),
        viewport_rows: u16::try_from(grid.sy).unwrap_or(u16::MAX),
        scrollback_rows: grid.hsize,
        rows,
    }
}

/// The row's tmux line flags, minus the wrap flag the snapshot reports on its
/// own.
fn row_flags(line: Option<&super::grid::Line>) -> RowFlags {
    let flags = line.map_or(0, |line| line.flags);
    RowFlags {
        hyperlink: flags & line_flag::HYPERLINK != 0,
        start_output: flags & line_flag::START_OUTPUT != 0,
        start_prompt: flags & line_flag::START_PROMPT != 0,
        extended: flags & line_flag::EXTENDED != 0,
    }
}

/// tmux classifies a whole row, not a cell: `GRID_LINE_START_PROMPT` and
/// `GRID_LINE_START_OUTPUT` are line flags, and everything on the line inherits
/// them. A row that starts output and one that starts neither are both output,
/// which leaves the prompt flag as the only thing to look for.
fn row_semantic(grid: &EngineGrid, py: usize) -> CellSemantic {
    let Some(line) = grid.line(py) else {
        return CellSemantic::Output;
    };
    if line.flags & line_flag::START_PROMPT != 0 {
        CellSemantic::Prompt
    } else {
        CellSemantic::Output
    }
}

fn snapshot_cell(screen: &Screen, cell: &Cell, semantic: CellSemantic, written: bool) -> GridCell {
    let width = if cell.is_padding() {
        CellWidth::SpacerTail
    } else if cell.data.width > 1 {
        CellWidth::Wide
    } else {
        CellWidth::Narrow
    };
    // A cell nothing has been written into reports as empty rather than as a
    // literal space, which is how a consumer tells a gap from typed blanks.
    // An erase counts as nothing written, even though it touched the cell.
    let empty = !written || cell.is_padding() || cell.flags & flag::CLEARED != 0;
    let text = if empty {
        String::new()
    } else {
        cell.data.text().to_string()
    };
    // A cell's `link` indexes the screen's hyperlink table. The `id=` comes
    // back only when the sequence carried one: an anonymous link has a URI and
    // no id, which is the distinction `capture-pane -e` re-emits.
    let (hyperlink, hyperlink_id) = match screen.hyperlinks.get(cell.link) {
        Some((uri, id)) => (Some(uri.to_string()), id.map(str::to_string)),
        None => (None, None),
    };
    GridCell {
        text,
        width,
        semantic,
        hyperlink,
        hyperlink_id,
        tab: cell.flags & flag::TAB != 0,
    }
}

/// Plain text for physical rows `[start, start + count)`.
///
/// Trailing blanks go, and so do trailing blank *rows*: a plain read of a
/// mostly-empty screen is the text on it, not the text plus a run of newlines.
/// When `unwrap` is set, a row that soft-wraps is rejoined with the row it
/// wrapped into, so a logical line that the margin split reads as one line.
pub(crate) fn plain(screen: &Screen, start: usize, count: usize, unwrap: bool) -> String {
    let grid = &screen.grid;
    let total = grid.total();
    let start = start.min(total);
    let end = start.saturating_add(count).min(total);

    let mut lines: Vec<String> = Vec::new();
    let mut joining = false;
    for py in start..end {
        let wrapped = unwrap && grid.line(py).is_some_and(super::grid::Line::is_wrapped);
        if !joining {
            lines.push(String::new());
        }
        let line = lines.last_mut().expect("a line was just started");
        // A row being rejoined keeps its full width; only the last row of a
        // wrapped run has its trailing blanks trimmed.
        let width = if wrapped {
            grid.sx
        } else {
            grid.line_length(py)
        };
        for px in 0..width {
            let cell = grid.get(px, py);
            if cell.is_padding() {
                continue;
            }
            line.push_str(cell.data.text());
        }
        joining = wrapped;
    }
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    lines.join("\n")
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
pub(crate) enum RowExtent {
    /// Up to the last non-blank cell; the caller erases the rest.
    Redraw,
    /// A capture, running to the extent `capture-pane`'s flags selected.
    Capture(CaptureExtent),
}

/// VT bytes for physical rows `[start, start + count)`, positioned absolutely
/// and with the cursor left where the screen has it.
pub(crate) fn vt(screen: &Screen, start: usize, count: usize, extent: RowExtent) -> Vec<u8> {
    vt_grid(screen, &screen.grid, start, count, extent)
}

/// As [`vt`], but of a grid other than the screen's own; see [`snapshot_grid`].
pub(crate) fn vt_grid(
    screen: &Screen,
    grid: &EngineGrid,
    start: usize,
    count: usize,
    extent: RowExtent,
) -> Vec<u8> {
    let total = grid.total();
    let start = start.min(total);
    let end = start.saturating_add(count).min(total);

    let mut out = String::new();
    // A capture is one stream. tmux carries `grid_string_cells`'s last cell
    // across the rows of a single `capture-pane`, so a style a row opens is
    // closed by the next row that differs from it — and never at all if nothing
    // differs, which is why a capture does not end in a reset. A redraw cannot
    // work that way: its rows are painted in different places, so each one
    // starts and ends at the default cell.
    let mut pen = Cell::default();
    for (index, py) in (start..end).enumerate() {
        if index != 0 {
            out.push_str("\r\n");
        }
        if extent == RowExtent::Redraw {
            pen = Cell::default();
        }
        pen = vt_row(screen, grid, py, extent, pen, &mut out);
    }
    // Put the cursor back where the screen has it, in one-based coordinates.
    let _ = write!(out, "\x1b[{};{}H", screen.cy + 1, screen.cx + 1);
    out.into_bytes()
}

/// The OSC 8 that ends whichever hyperlink is open.
const CLOSE_HYPERLINK: &str = "\x1b]8;;\x1b\\";

/// One row, with each run of like-styled cells written under one SGR. `pen` is
/// the style already in effect; the style this row leaves in effect comes back.
fn vt_row(
    screen: &Screen,
    grid: &EngineGrid,
    py: usize,
    extent: RowExtent,
    mut pen: Cell,
    out: &mut String,
) -> Cell {
    let mut charset = false;
    let mut link = 0u32;
    let width = match extent {
        RowExtent::Redraw => grid.line_length(py),
        RowExtent::Capture(CaptureExtent::Allocated) => grid
            .line(py)
            .map_or(0, super::grid::Line::size)
            .min(grid.sx),
        RowExtent::Capture(CaptureExtent::Written) => grid
            .line(py)
            .map_or(0, super::grid::Line::used)
            .min(grid.sx),
    };
    for px in 0..width {
        let cell = grid.get(px, py);
        if cell.is_padding() {
            continue;
        }
        if !cell.looks_equal(&pen) {
            out.push_str(&sgr(&pen, &cell));
            pen = cell.clone();
        }
        // tmux writes the hyperlink after the style codes, and only when the
        // cell's link differs from the one already open.
        if cell.link != link {
            match screen.hyperlinks.get(cell.link) {
                Some((uri, Some(id))) => {
                    let _ = write!(out, "\x1b]8;id={id};{uri}\x1b\\");
                }
                Some((uri, None)) => {
                    let _ = write!(out, "\x1b]8;;{uri}\x1b\\");
                }
                // The cell is not in a link; close the one that is open.
                None => out.push_str(CLOSE_HYPERLINK),
            }
            link = cell.link;
        }
        let wants_charset = cell.attr & attr::CHARSET != 0;
        if wants_charset != charset {
            out.push_str(if wants_charset { "\x1b(0" } else { "\x1b(B" });
            charset = wants_charset;
        }
        // A capture puts the tab back: tmux's `grid_string_cells` writes one
        // `\t` for a tab cell whatever blanks it holds, so the captured row is
        // narrower than the columns the tab covered. A redraw keeps the blanks
        // — the client's own tab stops are not the pane's, and a `\t` there
        // would land the rest of the row somewhere else.
        if extent != RowExtent::Redraw && cell.flags & flag::TAB != 0 {
            out.push('\t');
        } else {
            out.push_str(cell.data.text());
        }
    }
    if charset {
        out.push_str("\x1b(B");
    }
    if link != 0 {
        out.push_str(CLOSE_HYPERLINK);
    }
    // A redraw closes whatever the row opened, because the next row is painted
    // somewhere else. A capture leaves it open and hands it to the next row,
    // which is what tmux's carried `lastgc` does.
    if extent == RowExtent::Redraw && !pen.looks_equal(&Cell::default()) {
        out.push_str("\x1b[0m");
        pen = Cell::default();
    }
    // Nothing is erased here. A pane is not always the full width of the
    // client's line — inside a popup, or beside another pane, the columns to
    // the right belong to something else — and erasing to end of line takes
    // them with it, a popup's own right border included. The compositor
    // already blanks exactly the pane's width with ECH before painting each
    // row, so the row only has to carry its own cells.
    pen
}

/// The SGR that takes the terminal from `from` to `to`.
///
/// A full reset first keeps this simple and correct: the alternative is
/// tracking which of a dozen attributes have to be turned off individually,
/// and the compositor already coalesces the bytes.
fn sgr(from: &Cell, to: &Cell) -> String {
    let mut out = String::new();
    // The hyperlink is deliberately not part of this decision. tmux's
    // `grid_string_cells_code` decides the leading reset from the attributes
    // and the underline colour alone and writes the link separately, so a cell
    // that only leaves a link must not emit an SGR reset between the linked
    // text and the sequence that closes the link.
    let mut unlinked = from.clone();
    unlinked.link = 0;
    if !unlinked.looks_equal(&Cell::default()) {
        out.push_str("\x1b[0m");
    }
    let mut params: Vec<String> = Vec::new();
    let attrs = [
        (attr::BRIGHT, "1"),
        (attr::DIM, "2"),
        (attr::ITALICS, "3"),
        (attr::UNDERSCORE, "4"),
        (attr::BLINK, "5"),
        (attr::REVERSE, "7"),
        (attr::HIDDEN, "8"),
        (attr::STRIKETHROUGH, "9"),
        (attr::OVERLINE, "53"),
    ];
    for (bit, code) in attrs {
        if to.attr & bit != 0 {
            params.push(code.to_string());
        }
    }
    for (style, code) in [
        (attr::UNDERSCORE_2, "4:2"),
        (attr::UNDERSCORE_3, "4:3"),
        (attr::UNDERSCORE_4, "4:4"),
        (attr::UNDERSCORE_5, "4:5"),
    ] {
        if to.attr & style != 0 {
            params.push(code.to_string());
        }
    }
    if !params.is_empty() {
        let _ = write!(out, "\x1b[{}m", params.join(";"));
    }

    // Each colour gets its own sequence. tmux's `grid_string_cells_code` calls
    // `grid_string_cells_add_code` once per colour and each call writes a
    // complete SGR, so the attributes arrive joined and the colours separately.
    if let Some(code) = sgr_colour(to.fg, 30, 38) {
        let _ = write!(out, "\x1b[{code}m");
    }
    if let Some(code) = sgr_colour(to.bg, 40, 48) {
        let _ = write!(out, "\x1b[{code}m");
    }
    // The underline colour has no eight-colour spelling: tmux's
    // `grid_string_cells_us` only ever writes the 5 and 2 forms.
    if !super::grid::colour_is_default(to.us) {
        if let Some((red, green, blue)) = colour::as_rgb(to.us) {
            let _ = write!(out, "\x1b[58;2;{red};{green};{blue}m");
        } else if to.us & colour::FLAG_256 != 0 {
            let _ = write!(out, "\x1b[58;5;{}m", to.us & 0xff);
        }
    }
    out
}

/// One colour as SGR parameters. `base` is the eight-colour offset (30 or 40)
/// and `extended` the 38/48 introducer for the richer forms.
fn sgr_colour(value: i32, base: i32, extended: i32) -> Option<String> {
    if super::grid::colour_is_default(value) {
        return None;
    }
    if let Some((red, green, blue)) = colour::as_rgb(value) {
        return Some(format!("{extended};2;{red};{green};{blue}"));
    }
    if value & colour::FLAG_256 != 0 {
        return Some(format!("{extended};5;{}", value & 0xff));
    }
    match value {
        0..=7 => Some((base + value).to_string()),
        // The aixterm brights have their own codes, and the background ones sit
        // ten above the foreground ones. tmux stores both halves as 90-97 and
        // adds the ten back on the way out, which is why `base` carries it.
        90..=97 => Some((value + base - 30).to_string()),
        _ => None,
    }
}

/// Whether the cursor should be shown, tmux's `MODE_CURSOR`.
pub(crate) fn cursor_visible(screen: &Screen) -> bool {
    screen.mode & mode::CURSOR != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vt::engine::dispatch::Engine;
    use crate::vt::parser::tokenize;

    fn engine(sx: usize, sy: usize, input: &[u8]) -> Engine {
        let mut engine = Engine::new(sx, sy, 100);
        for token in tokenize(input) {
            engine.apply(&token.kind);
        }
        engine
    }

    fn all_plain(engine: &Engine, unwrap: bool) -> String {
        let total = engine.screen.grid.total();
        plain(&engine.screen, 0, total, unwrap)
    }

    #[test]
    fn plain_text_is_the_rows_with_their_tails_trimmed() {
        let engine = engine(10, 3, b"one\r\ntwo");
        assert_eq!(all_plain(&engine, false), "one\ntwo");
    }

    #[test]
    fn plain_text_can_rejoin_a_soft_wrap() {
        let engine = engine(4, 3, b"abcdefg");
        assert_eq!(all_plain(&engine, false), "abcd\nefg");
        assert_eq!(all_plain(&engine, true), "abcdefg");
    }

    #[test]
    fn a_hard_newline_is_never_rejoined() {
        let engine = engine(10, 3, b"ab\r\ncd");
        assert_eq!(all_plain(&engine, true), "ab\ncd");
    }

    #[test]
    fn plain_text_covers_history_as_well_as_the_viewport() {
        let engine = engine(10, 2, b"a\r\nb\r\nc\r\nd");
        assert_eq!(engine.screen.grid.hsize, 2);
        assert_eq!(all_plain(&engine, false), "a\nb\nc\nd");
    }

    #[test]
    fn a_snapshot_distinguishes_written_spaces_from_untouched_cells() {
        let engine = engine(6, 1, b"a c");
        let grid = snapshot(&engine.screen, 0, 1);
        assert_eq!(grid.rows[0].cells[0].text, "a");
        assert_eq!(grid.rows[0].cells[1].text, " ", "a space was written here");
        assert_eq!(grid.rows[0].cells[3].text, "", "nothing was written here");
    }

    #[test]
    fn a_snapshot_marks_wide_characters_and_their_spacers() {
        let engine = engine(6, 1, "界x".as_bytes());
        let grid = snapshot(&engine.screen, 0, 1);
        assert_eq!(grid.rows[0].cells[0].text, "界");
        assert_eq!(grid.rows[0].cells[0].width, CellWidth::Wide);
        assert_eq!(grid.rows[0].cells[1].width, CellWidth::SpacerTail);
        assert_eq!(grid.rows[0].cells[2].text, "x");
    }

    #[test]
    fn a_snapshot_reports_the_soft_wrap_flag() {
        let engine = engine(4, 3, b"abcdef");
        let grid = snapshot(&engine.screen, 0, 3);
        assert!(grid.rows[0].wrapped);
        assert!(!grid.rows[1].wrapped);
    }

    #[test]
    fn vt_carries_the_text_its_colours_and_the_cursor() {
        let engine = engine(10, 2, b"\x1b[31mRED\x1b[0m");
        let dump = String::from_utf8(vt(&engine.screen, 0, 2, RowExtent::Redraw)).expect("utf8");
        assert!(dump.contains("RED"), "got {dump:?}");
        assert!(dump.contains("\x1b[31m"), "got {dump:?}");
        assert!(dump.ends_with("\x1b[1;4H"), "cursor last, got {dump:?}");
    }

    #[test]
    fn every_row_closes_the_style_it_opened() {
        let engine = engine(10, 2, b"\x1b[41ma\r\nb");
        let dump = String::from_utf8(vt(&engine.screen, 0, 2, RowExtent::Redraw)).expect("utf8");
        for row in dump.split("\r\n") {
            let opens = row.matches("\x1b[41m").count();
            if opens > 0 {
                assert!(
                    row.contains("\x1b[0m"),
                    "a row that opens a style must close it, got {row:?}"
                );
            }
        }
    }

    #[test]
    fn vt_reopens_the_line_drawing_set_only_where_it_is_used() {
        let engine = engine(10, 2, b"\x1b(0q\x1b(Bx");
        let dump = String::from_utf8(vt(&engine.screen, 0, 1, RowExtent::Redraw)).expect("utf8");
        assert!(dump.contains("\x1b(0q"), "got {dump:?}");
        assert!(dump.contains("\x1b(Bx"), "got {dump:?}");
    }

    #[test]
    fn vt_writes_direct_colours_as_direct_colours() {
        let engine = engine(10, 1, b"\x1b[38;2;1;2;3mx");
        let dump = String::from_utf8(vt(&engine.screen, 0, 1, RowExtent::Redraw)).expect("utf8");
        assert!(dump.contains("\x1b[38;2;1;2;3m"), "got {dump:?}");
    }
}
