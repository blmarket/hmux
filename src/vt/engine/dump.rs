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
use super::screen::{mode, Screen};
use crate::vt::screen::{CellSemantic, CellWidth, Grid, GridCell, GridRow};

use super::grid::line_flag;

/// Snapshot physical rows `[start, start + count)`, clamped to the grid.
pub(crate) fn snapshot(screen: &Screen, start: usize, count: usize) -> Grid {
    let grid = &screen.grid;
    let total = grid.total();
    let start = start.min(total);
    let end = start.saturating_add(count).min(total);
    let mut rows = Vec::with_capacity(end - start);
    for py in start..end {
        let wrapped = grid.line(py).is_some_and(super::grid::Line::is_wrapped);
        let semantic = row_semantic(grid, py);
        // Past the row's written extent nothing has been put there at all,
        // whatever the cell store happens to hold.
        let used = grid.line(py).map_or(0, super::grid::Line::used);
        let cells = (0..grid.sx)
            .map(|px| snapshot_cell(&grid.get(px, py), semantic, px < used))
            .collect();
        rows.push(GridRow { cells, wrapped });
    }
    Grid {
        cols: u16::try_from(grid.sx).unwrap_or(u16::MAX),
        viewport_rows: u16::try_from(grid.sy).unwrap_or(u16::MAX),
        scrollback_rows: grid.hsize,
        rows,
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

fn snapshot_cell(cell: &Cell, semantic: CellSemantic, written: bool) -> GridCell {
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
    GridCell {
        text,
        width,
        semantic,
        hyperlink: None,
        hyperlink_id: None,
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

/// VT bytes for physical rows `[start, start + count)`, positioned absolutely
/// and with the cursor left where the screen has it.
pub(crate) fn vt(screen: &Screen, start: usize, count: usize) -> Vec<u8> {
    let grid = &screen.grid;
    let total = grid.total();
    let start = start.min(total);
    let end = start.saturating_add(count).min(total);

    let mut out = String::new();
    for (index, py) in (start..end).enumerate() {
        if index != 0 {
            out.push_str("\r\n");
        }
        vt_row(screen, py, &mut out);
    }
    // Put the cursor back where the screen has it, in one-based coordinates.
    let _ = write!(out, "\x1b[{};{}H", screen.cy + 1, screen.cx + 1);
    out.into_bytes()
}

/// One row, with each run of like-styled cells written under one SGR.
fn vt_row(screen: &Screen, py: usize, out: &mut String) {
    let grid = &screen.grid;
    let mut pen = Cell::default();
    let mut charset = false;
    let width = grid.line_length(py);
    for px in 0..width {
        let cell = grid.get(px, py);
        if cell.is_padding() {
            continue;
        }
        if !cell.looks_equal(&pen) {
            out.push_str(&sgr(&pen, &cell));
            pen = cell.clone();
        }
        let wants_charset = cell.attr & attr::CHARSET != 0;
        if wants_charset != charset {
            out.push_str(if wants_charset { "\x1b(0" } else { "\x1b(B" });
            charset = wants_charset;
        }
        out.push_str(cell.data.text());
    }
    if charset {
        out.push_str("\x1b(B");
    }
    // Close whatever the row opened: the next row is painted somewhere else.
    if !pen.looks_equal(&Cell::default()) {
        out.push_str("\x1b[0m");
    }
    // Erase the rest of the row so a shorter row does not leave the previous
    // frame's tail behind it.
    out.push_str("\x1b[K");
}

/// The SGR that takes the terminal from `from` to `to`.
///
/// A full reset first keeps this simple and correct: the alternative is
/// tracking which of a dozen attributes have to be turned off individually,
/// and the compositor already coalesces the bytes.
fn sgr(from: &Cell, to: &Cell) -> String {
    let mut out = String::new();
    if !from.looks_equal(&Cell::default()) {
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
    if let Some(code) = sgr_colour(to.fg, 30, 38) {
        params.push(code);
    }
    if let Some(code) = sgr_colour(to.bg, 40, 48) {
        params.push(code);
    }
    if params.is_empty() {
        return out;
    }
    let _ = write!(out, "\x1b[{}m", params.join(";"));
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
        // The aixterm brights have their own codes.
        90..=97 => Some(value.to_string()),
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
        let dump = String::from_utf8(vt(&engine.screen, 0, 2)).expect("utf8");
        assert!(dump.contains("RED"), "got {dump:?}");
        assert!(dump.contains("\x1b[31m"), "got {dump:?}");
        assert!(dump.ends_with("\x1b[1;4H"), "cursor last, got {dump:?}");
    }

    #[test]
    fn every_row_closes_the_style_it_opened() {
        let engine = engine(10, 2, b"\x1b[41ma\r\nb");
        let dump = String::from_utf8(vt(&engine.screen, 0, 2)).expect("utf8");
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
        let dump = String::from_utf8(vt(&engine.screen, 0, 1)).expect("utf8");
        assert!(dump.contains("\x1b(0q"), "got {dump:?}");
        assert!(dump.contains("\x1b(Bx"), "got {dump:?}");
    }

    #[test]
    fn vt_writes_direct_colours_as_direct_colours() {
        let engine = engine(10, 1, b"\x1b[38;2;1;2;3mx");
        let dump = String::from_utf8(vt(&engine.screen, 0, 1)).expect("utf8");
        assert!(dump.contains("\x1b[38;2;1;2;3m"), "got {dump:?}");
    }
}
