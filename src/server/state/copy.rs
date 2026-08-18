//! Copy mode — the frozen grid a pane shows while in it, and every cursor
//! motion, selection, and search that operates on that grid.
//!
//! [`CopyState`] is tmux's `window_copy_mode_data`: a snapshot of the pane's
//! screen plus scrollback, the cursor and selection over it, and the search
//! state. Everything here works on that snapshot, never on `ServerState`.

use std::io;

use super::PaneScreen;
use hmux_vt::{CellSemantic, CellWidth, Grid, GridCell, GridRow, RowExtent};

#[derive(Clone, Debug)]
pub(crate) struct CopyState {
    pub(crate) backing: CopyBacking,
    pub(crate) cursor: CopyCursor,
    /// Horizontal goal retained while vertical movement crosses short rows.
    pub(crate) desired_col: usize,
    pub(crate) selection: Option<CopySelection>,
    pub(crate) rectangle: bool,
    pub(crate) selection_mode: CopySelectionMode,
    pub(crate) mark: Option<(usize, usize)>,
    pub(crate) jump: Option<CopyJump>,
    pub(crate) hide_position: bool,
    pub(crate) search: Option<CopySearch>,
    /// Whether the matches are currently marked up — tmux's
    /// `data->searchmark != NULL`, which is what gates `#{search_count}`. A
    /// search that is cleared or re-run against a fresh grid drops the marks
    /// without dropping the pattern.
    pub(crate) search_marks: bool,
    /// Cursor and viewport saved when an incremental search begins.
    pub(crate) incremental_search_origin: Option<CopySearchOrigin>,
    /// Numeric prefix retained on the mode entry until its next command.
    pub(crate) prefix: u32,
    pub(crate) scroll_exit: bool,
    pub(crate) recentre: CopyRecentre,
    pub(crate) grid: Grid,
    /// Frozen styled VT serialization captured with `grid`.
    pub(crate) vt: Vec<u8>,
    /// Byte ranges for the content of each row in `vt`, excluding CR and the
    /// trailing cursor-position sequence. Copy-mode movement reuses these
    /// offsets instead of rescanning the entire history on every repaint.
    pub(super) vt_rows: Vec<std::ops::Range<usize>>,
    /// Rows above the live-bottom viewport, matching tmux's copy-mode `oy`.
    pub(crate) scroll: usize,
}

#[derive(Clone, Debug)]
pub(crate) enum CopyBacking {
    PaneSnapshot,
    ViewOutput(Vec<u8>),
}

impl CopyState {
    /// `#{search_count}`: how many matches are marked up, or `None` while
    /// nothing is — tmux reports the count only alongside a live searchmark.
    pub(crate) fn search_count(&self) -> Option<usize> {
        self.search_marks.then(|| {
            self.search
                .as_ref()
                .map_or(0, |search| search.matches.len())
        })
    }

    /// `#{top_line_time}`: the second the row at the top of the view scrolled
    /// into the history, tmux's `grid_get_line(backing, hsize - oy)->time`. A
    /// view still at the bottom tops out on a row that is on screen rather than
    /// in the history, and one of those was never stamped, so it reports zero.
    pub(crate) fn top_line_time(&self) -> u64 {
        let view_top = self.grid.scrollback_rows.saturating_sub(self.scroll);
        self.grid.rows.get(view_top).map_or(0, |row| row.time)
    }

    pub(crate) fn vt_rows(&self) -> impl Iterator<Item = &[u8]> {
        self.vt_rows.iter().map(|range| &self.vt[range.clone()])
    }

    pub(super) fn replace_vt(&mut self, vt: Vec<u8>) {
        self.vt_rows = copy_vt_row_ranges(&vt);
        self.vt = vt;
    }
}

pub(super) fn copy_vt_row_ranges(vt: &[u8]) -> Vec<std::ops::Range<usize>> {
    let content_end = if vt.last() == Some(&b'H') {
        let mut start = vt.len() - 1;
        while start > 0 && matches!(vt[start - 1], b';' | b'0'..=b'9') {
            start -= 1;
        }
        if start >= 2 && vt[start - 2..start] == [0x1b, b'['] {
            start - 2
        } else {
            vt.len()
        }
    } else {
        vt.len()
    };
    let content = &vt[..content_end];
    let mut rows = Vec::new();
    let mut start = 0;
    for (offset, byte) in content.iter().enumerate() {
        if *byte == b'\n' {
            let end = if offset > start && content[offset - 1] == b'\r' {
                offset - 1
            } else {
                offset
            };
            rows.push(start..end);
            start = offset + 1;
        }
    }
    let end = if content.len() > start && content.last() == Some(&b'\r') {
        content.len() - 1
    } else {
        content.len()
    };
    rows.push(start..end);
    rows
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CopyJumpKind {
    Forward,
    Backward,
    ToForward,
    ToBackward,
}

#[derive(Clone, Debug)]
pub(crate) struct CopyJump {
    pub(crate) text: String,
    pub(crate) kind: CopyJumpKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CopySelectionMode {
    Character,
    Word,
    Line,
}

#[derive(Clone, Debug)]
pub(crate) struct CopyCursor {
    pub(crate) row: usize,
    pub(crate) col: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct CopySearchOrigin {
    pub(crate) cursor: CopyCursor,
    pub(crate) desired_col: usize,
    pub(crate) scroll: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CopyRecentreState {
    Middle,
    Top,
    Bottom,
}

#[derive(Clone, Debug)]
pub(crate) struct CopyRecentre {
    pub(crate) state: CopyRecentreState,
    pub(crate) line: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct CopySelection {
    pub(crate) anchor: (usize, usize),
    pub(crate) end: (usize, usize),
    pub(crate) active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CopySearchDirection {
    Backward,
    Forward,
}

#[derive(Clone, Debug)]
pub(crate) struct CopySearch {
    pub(crate) pattern: String,
    pub(crate) regex: bool,
    pub(crate) direction: CopySearchDirection,
    pub(crate) last_direction: CopySearchDirection,
    pub(crate) matches: Vec<CopySearchMatch>,
}

#[derive(Clone, Debug)]
pub(crate) struct CopySearchMatch {
    pub(crate) start: (usize, usize),
    pub(crate) end_after: (usize, usize),
    pub(crate) segments: Vec<(usize, usize, usize)>,
}

pub(super) fn reflow_copy_snapshot(state: &mut CopyState, cols: u16, rows: u16) -> io::Result<()> {
    if state.grid.cols == cols && state.grid.viewport_rows == rows {
        return Ok(());
    }
    let cursor_offset = copy_point_offset(&state.grid, (state.cursor.row, state.cursor.col));
    let mark_offset = state
        .mark
        .map(|point| copy_point_offset(&state.grid, point));
    // A logical line's history stamp survives a rewrap on the row the rewrap
    // leaves it starting on, as `Grid::reflow` keeps it. The replay below
    // rebuilds the snapshot from VT, which carries no stamps, so they are
    // re-attached afterwards by the offset the logical line starts at.
    let mut stamps = Vec::new();
    for (row, line) in state.grid.rows.iter().enumerate() {
        let starts_logical_line = row == 0 || !state.grid.rows[row - 1].wrapped;
        if line.time != 0 && starts_logical_line {
            stamps.push((copy_point_offset(&state.grid, (row, 0)), line.time));
        }
    }
    let mut metadata = Vec::new();
    for (row, line) in state.grid.rows.iter().enumerate() {
        for (col, cell) in line.cells.iter().enumerate() {
            if cell.semantic != CellSemantic::Output || cell.hyperlink.is_some() {
                metadata.push((
                    copy_point_offset(&state.grid, (row, col)),
                    cell.semantic,
                    cell.hyperlink.clone(),
                ));
            }
        }
    }
    let mut terminal = PaneScreen::new(state.grid.cols, state.grid.viewport_rows);
    terminal.apply_vt(&copy_reflow_vt(state));
    terminal.resize(cols, rows);
    state.grid = terminal.grid_snapshot_range(0, terminal.grid_dims().total_rows);
    for (offset, semantic, hyperlink) in metadata {
        let (row, col) = copy_point_at_offset(&state.grid, offset);
        if let Some(cell) = state
            .grid
            .rows
            .get_mut(row)
            .and_then(|line| line.cells.get_mut(col))
        {
            cell.semantic = semantic;
            cell.hyperlink = hyperlink;
        }
    }
    for (offset, time) in stamps {
        let (row, _) = copy_point_at_offset(&state.grid, offset);
        if let Some(line) = state.grid.rows.get_mut(row) {
            line.time = time;
        }
    }
    let vt = terminal.dump_vt_rows(0, terminal.grid_dims().total_rows, RowExtent::Redraw);
    state.replace_vt(vt);
    let (cursor_row, cursor_col) = copy_point_at_offset(&state.grid, cursor_offset);
    state.cursor = CopyCursor {
        row: cursor_row,
        col: cursor_col,
    };
    state.desired_col = state.cursor.col;
    state.selection = None;
    state.mark = mark_offset.map(|offset| copy_point_at_offset(&state.grid, offset));
    if let Some(search) = state.search.as_mut() {
        search.matches = copy_search_matches(&state.grid, &search.pattern, search.regex);
    }
    state.scroll = state.scroll.min(state.grid.scrollback_rows);
    Ok(())
}

fn view_output_vt(output: &[u8]) -> Vec<u8> {
    let mut vt =
        Vec::with_capacity(output.len() + output.iter().filter(|&&byte| byte == b'\n').count());
    let mut previous = None;
    for &byte in output {
        if byte == b'\n' && previous != Some(b'\r') {
            vt.push(b'\r');
        }
        vt.push(byte);
        previous = Some(byte);
    }
    vt
}

pub(super) fn view_copy_state(output: Vec<u8>, cols: u16, rows: u16) -> io::Result<CopyState> {
    let mut terminal = PaneScreen::new(cols.max(1), rows.max(1));
    terminal.apply_vt(&view_output_vt(&output));
    let total = terminal.grid_dims().total_rows;
    let grid = terminal.grid_snapshot_range(0, total);
    let vt = terminal.dump_vt_rows(0, total, RowExtent::Redraw);
    let vt_rows = copy_vt_row_ranges(&vt);
    let (col, row) = terminal.cursor_position();
    let scroll = grid.scrollback_rows;
    Ok(CopyState {
        backing: CopyBacking::ViewOutput(output),
        cursor: CopyCursor {
            row: grid.scrollback_rows.saturating_add(row as usize),
            col: col as usize,
        },
        desired_col: col as usize,
        selection: None,
        rectangle: false,
        selection_mode: CopySelectionMode::Character,
        mark: None,
        jump: None,
        hide_position: false,
        search: None,
        search_marks: true,
        incremental_search_origin: None,
        prefix: 1,
        scroll_exit: false,
        recentre: CopyRecentre {
            state: CopyRecentreState::Middle,
            line: 0,
        },
        grid,
        vt,
        vt_rows,
        scroll,
    })
}

fn copy_point_offset(grid: &Grid, point: (usize, usize)) -> usize {
    let mut offset = 0;
    for row in 0..point.0.min(grid.rows.len()) {
        offset += copy_line_length(grid, row);
        if !grid.rows[row].wrapped {
            offset += 1;
        }
    }
    offset + point.1.min(copy_line_length(grid, point.0))
}

fn copy_point_at_offset(grid: &Grid, mut offset: usize) -> (usize, usize) {
    for row in 0..grid.rows.len() {
        let length = copy_line_length(grid, row);
        if offset <= length {
            return (row, offset.min(grid.cols.saturating_sub(1) as usize));
        }
        offset -= length;
        if !grid.rows[row].wrapped {
            if offset == 0 {
                return (row, length.min(grid.cols.saturating_sub(1) as usize));
            }
            offset -= 1;
        }
    }
    let row = grid.rows.len().saturating_sub(1);
    (
        row,
        copy_line_length(grid, row).min(grid.cols.saturating_sub(1) as usize),
    )
}

fn copy_reflow_vt(state: &CopyState) -> Vec<u8> {
    let cursor_start = state
        .vt
        .iter()
        .rposition(|&byte| byte == 0x1b)
        .filter(|&start| state.vt.get(start + 1) == Some(&b'[') && state.vt.last() == Some(&b'H'))
        .unwrap_or(state.vt.len());
    let (content, cursor) = state.vt.split_at(cursor_start);
    let rows = content.split(|&byte| byte == b'\n').collect::<Vec<_>>();
    let mut out = Vec::with_capacity(state.vt.len());
    for (index, row) in rows.iter().enumerate() {
        out.extend_from_slice(row.strip_suffix(b"\r").unwrap_or(row));
        if index + 1 < rows.len() && !state.grid.rows.get(index).is_some_and(|row| row.wrapped) {
            out.extend_from_slice(b"\r\n");
        }
    }
    out.extend_from_slice(cursor);
    out
}

fn copy_cell_is_padding(cell: &GridCell) -> bool {
    matches!(cell.width, CellWidth::SpacerTail)
}

fn copy_line_length(grid: &Grid, row: usize) -> usize {
    let Some(row) = grid.rows.get(row) else {
        return 0;
    };
    let mut length = 0;
    for (col, cell) in row.cells.iter().enumerate() {
        if copy_cell_is_padding(cell) || (!cell.text.is_empty() && cell.text != " ") {
            length = col + 1;
        }
    }
    length.min(grid.cols as usize)
}

/// tmux's `grid_in_set`: whether the cell under the cursor is one of `set`, and
/// how many columns of it are.
///
/// The answer is a column count rather than a flag because of tabs. A set that
/// contains `\t` matches a tab cell whole — and matches a cell inside the run
/// the tab painted, counting the columns that are left — so a caller stepping
/// over whitespace crosses the tab in one move instead of landing inside it.
/// Everything else is one column, and nothing is zero.
fn copy_cell_set_width(grid: &Grid, cursor: &CopyCursor, set: &str) -> usize {
    let Some(row) = grid.rows.get(cursor.row) else {
        return usize::from(set.contains(' '));
    };
    let Some(cell) = row.cells.get(cursor.col) else {
        return usize::from(set.contains(' '));
    };
    if set.contains('\t') {
        if copy_cell_is_padding(cell) {
            // Walk back to whatever owns this padding; only a tab's counts.
            let start = row.cells[..cursor.col]
                .iter()
                .rposition(|cell| !copy_cell_is_padding(cell));
            if let Some(start) = start {
                if row.cells[start].tab {
                    let width = copy_cell_columns(row, start);
                    return width.saturating_sub(cursor.col - start);
                }
            }
        } else if cell.tab {
            return copy_cell_columns(row, cursor.col);
        }
    }
    if copy_cell_is_padding(cell) {
        return 0;
    }
    if cell.text.is_empty() {
        return usize::from(set.contains(' '));
    }
    let mut chars = cell.text.chars();
    let Some(ch) = chars.next() else {
        return usize::from(set.contains(' '));
    };
    usize::from(chars.next().is_none() && set.chars().any(|candidate| candidate == ch))
}

/// How many columns the cell at `col` covers: itself plus the padding that
/// follows it. tmux reads this off `gc.data.width`.
fn copy_cell_columns(row: &GridRow, col: usize) -> usize {
    1 + row.cells[col + 1..]
        .iter()
        .take_while(|cell| copy_cell_is_padding(cell))
        .count()
}

fn copy_cell_in_set(grid: &Grid, cursor: &CopyCursor, set: &str) -> bool {
    copy_cell_set_width(grid, cursor, set) != 0
}

pub(super) fn copy_cursor_limit(grid: &Grid, row: usize, vi: bool) -> usize {
    let length = copy_line_length(grid, row);
    if vi && length != 0 {
        length - 1
    } else {
        length
    }
}

fn clamp_copy_cursor(cursor: &mut CopyCursor, grid: &Grid, vi: bool) {
    cursor.row = cursor.row.min(grid.rows.len().saturating_sub(1));
    cursor.col = cursor.col.min(copy_cursor_limit(grid, cursor.row, vi));
}

pub(super) fn clamp_copy_state(state: &mut CopyState, vi: bool) {
    // A rectangle selection is not bound to the text. tmux lets the cursor and
    // both corners sit anywhere within the screen — that is what its `all`
    // argument to `grid_reader_cursor_right` is for — so clamping them to a
    // row's length here would drag the rectangle back to the shortest row it
    // covers, and the cursor would never get past the end of the row it is on.
    let rectangle = state.selection.is_some() && state.rectangle;
    let limit = |grid: &Grid, row: usize| {
        if rectangle {
            grid.cols as usize
        } else {
            copy_cursor_limit(grid, row, vi)
        }
    };
    state.cursor.row = state
        .cursor
        .row
        .min(state.grid.rows.len().saturating_sub(1));
    state.cursor.col = state.cursor.col.min(limit(&state.grid, state.cursor.row));
    state.desired_col = state.desired_col.min(state.grid.cols as usize);
    if let Some(selection) = state.selection.as_mut() {
        for point in [&mut selection.anchor, &mut selection.end] {
            point.0 = point.0.min(state.grid.rows.len().saturating_sub(1));
            point.1 = point.1.min(limit(&state.grid, point.0));
        }
    }
}

fn copy_view_top(state: &CopyState) -> usize {
    state.grid.scrollback_rows.saturating_sub(state.scroll)
}

/// tmux's `window_copy_clear_selection`: dropping a selection also puts the
/// selection mode back to characters, which is what `#{selection_mode}`
/// reports once a copy has consumed a line or word selection.
pub(super) fn clear_copy_selection(state: &mut CopyState) {
    state.selection = None;
    state.selection_mode = CopySelectionMode::Character;
}

pub(super) fn ensure_copy_cursor_visible(state: &mut CopyState) {
    let rows = state.grid.viewport_rows.max(1) as usize;
    let top = copy_view_top(state);
    if state.cursor.row < top {
        state.scroll = state.grid.scrollback_rows.saturating_sub(state.cursor.row);
    } else if state.cursor.row >= top.saturating_add(rows) {
        let new_top = state.cursor.row.saturating_add(1).saturating_sub(rows);
        state.scroll = state.grid.scrollback_rows.saturating_sub(new_top);
    }
    state.scroll = state.scroll.min(state.grid.scrollback_rows);
}

pub(super) fn move_copy_row(state: &mut CopyState, up: bool, vi: bool) {
    if up {
        state.cursor.row = state.cursor.row.saturating_sub(1);
    } else {
        state.cursor.row = state
            .cursor
            .row
            .saturating_add(1)
            .min(state.grid.rows.len().saturating_sub(1));
    }
    state.cursor.col = state
        .desired_col
        .min(copy_cursor_limit(&state.grid, state.cursor.row, vi));
    ensure_copy_cursor_visible(state);
}

pub(super) fn move_copy_page(state: &mut CopyState, up: bool, vi: bool) {
    let amount = (state.grid.viewport_rows as usize).saturating_sub(2).max(1);
    move_copy_rows(state, up, amount, vi);
}

pub(super) fn move_copy_rows(state: &mut CopyState, up: bool, amount: usize, vi: bool) {
    if up {
        state.cursor.row = state.cursor.row.saturating_sub(amount);
        state.scroll = state
            .scroll
            .saturating_add(amount)
            .min(state.grid.scrollback_rows);
    } else {
        state.cursor.row = state
            .cursor
            .row
            .saturating_add(amount)
            .min(state.grid.rows.len().saturating_sub(1));
        state.scroll = state.scroll.saturating_sub(amount);
    }
    state.cursor.col = state
        .desired_col
        .min(copy_cursor_limit(&state.grid, state.cursor.row, vi));
    ensure_copy_cursor_visible(state);
}

#[derive(Clone, Copy)]
pub(super) enum CopyViewLine {
    Top,
    Middle,
    Bottom,
}

pub(super) fn move_copy_view_line(state: &mut CopyState, line: CopyViewLine) {
    let top = copy_view_top(state);
    let height = state.grid.viewport_rows.max(1) as usize;
    let offset = match line {
        CopyViewLine::Top => 0,
        CopyViewLine::Middle => (height - 1) / 2,
        CopyViewLine::Bottom => height - 1,
    };
    state.cursor.row = top
        .saturating_add(offset)
        .min(state.grid.rows.len().saturating_sub(1));
    state.cursor.col = 0;
}

pub(super) fn copy_cursor_view_y(state: &CopyState) -> usize {
    state.cursor.row.saturating_sub(copy_view_top(state))
}

pub(super) fn centre_copy_cursor_vertical(state: &mut CopyState, vi: bool) {
    let row = copy_view_top(state)
        .saturating_add(state.grid.viewport_rows as usize / 2)
        .min(state.grid.rows.len().saturating_sub(1));
    state.cursor.row = row;
    if !state.rectangle {
        state.cursor.col = state
            .cursor
            .col
            .min(copy_cursor_limit(&state.grid, row, vi));
    }
}

pub(super) fn centre_copy_cursor_horizontal(state: &mut CopyState, vi: bool) {
    let centre = state.grid.cols as usize / 2;
    state.cursor.col = if state.rectangle {
        centre
    } else {
        centre.min(copy_cursor_limit(&state.grid, state.cursor.row, vi))
    };
}

pub(super) fn position_copy_cursor(state: &mut CopyState, row: usize, col: usize) {
    let row = row.min(state.grid.rows.len().saturating_sub(1));
    let top = copy_view_top(state);
    let height = state.grid.viewport_rows.max(1) as usize;
    if row < top || row >= top.saturating_add(height) {
        let history = state.grid.scrollback_rows;
        let gap = height / 4;
        let new_top = if row < height {
            0
        } else if row > history.saturating_add(height).saturating_sub(gap) {
            history
        } else {
            row.saturating_add(gap).saturating_sub(height)
        };
        state.scroll = history.saturating_sub(new_top.min(history));
    }
    state.cursor.row = row;
    state.cursor.col = col;
    state.desired_col = col;
}

pub(super) fn move_copy_paragraph(state: &mut CopyState, backward: bool) {
    if state.grid.rows.is_empty() {
        return;
    }
    let mut row = state.cursor.row;
    if backward {
        while row > 0 && copy_line_length(&state.grid, row) == 0 {
            row -= 1;
        }
        while row > 0 && copy_line_length(&state.grid, row) > 0 {
            row -= 1;
        }
        position_copy_cursor(state, row, 0);
    } else {
        let last = state.grid.rows.len() - 1;
        while row < last && copy_line_length(&state.grid, row) == 0 {
            row += 1;
        }
        while row < last && copy_line_length(&state.grid, row) > 0 {
            row += 1;
        }
        let col = copy_line_length(&state.grid, row);
        position_copy_cursor(state, row, col);
    }
}

pub(super) fn align_copy_cursor_in_view(state: &mut CopyState, target: usize) {
    let height = state.grid.viewport_rows.max(1) as usize;
    let target = target.min(height - 1);
    let current = copy_cursor_view_y(state);
    if current > target {
        let delta = current - target;
        if state.scroll >= delta {
            state.scroll -= delta;
        }
    } else {
        let delta = target - current;
        if state.grid.scrollback_rows.saturating_sub(state.scroll) >= delta {
            state.scroll += delta;
        }
    }
}

pub(super) fn scroll_copy_content(state: &mut CopyState, up: bool, vi: bool) {
    if up {
        if state.scroll == state.grid.scrollback_rows {
            return;
        }
        state.scroll += 1;
        state.cursor.row = state.cursor.row.saturating_sub(1);
    } else {
        if state.scroll == 0 {
            return;
        }
        state.scroll -= 1;
        state.cursor.row = state
            .cursor
            .row
            .saturating_add(1)
            .min(state.grid.rows.len().saturating_sub(1));
    }
    if state.selection.is_none() || !state.rectangle {
        state.cursor.col =
            state
                .desired_col
                .min(copy_cursor_limit(&state.grid, state.cursor.row, vi));
    }
}

pub(super) fn goto_copy_line(state: &mut CopyState, argument: &str, absolute: bool) {
    let Ok(mut line) = argument.parse::<i32>() else {
        return;
    };
    if line < -1 {
        return;
    }
    let history = state.grid.scrollback_rows;
    let cursor_y = copy_cursor_view_y(state);
    state.scroll = if absolute {
        if line <= 0 {
            line = 1;
        }
        let line = (line as usize).min(history.saturating_add(1));
        history.saturating_sub(line.saturating_sub(1))
    } else if line < 0 || line as usize > history {
        history
    } else {
        line as usize
    };
    state.cursor.row = copy_view_top(state)
        .saturating_add(cursor_y)
        .min(state.grid.rows.len().saturating_sub(1));
}

pub(super) fn recentre_copy_cursor(state: &mut CopyState) {
    if state.recentre.line != state.cursor.row {
        state.recentre.state = CopyRecentreState::Middle;
        state.recentre.line = state.cursor.row;
    }
    let height = state.grid.viewport_rows.max(1) as usize;
    let target = match state.recentre.state {
        CopyRecentreState::Middle => {
            state.recentre.state = CopyRecentreState::Top;
            (height - 1) / 2
        }
        CopyRecentreState::Top => {
            state.recentre.state = CopyRecentreState::Bottom;
            0
        }
        CopyRecentreState::Bottom => {
            state.recentre.state = CopyRecentreState::Middle;
            height - 1
        }
    };
    align_copy_cursor_in_view(state, target);
}

fn copy_ascii_cell(grid: &Grid, row: usize, col: usize) -> Option<u8> {
    let cell = grid.rows.get(row)?.cells.get(col)?;
    if copy_cell_is_padding(cell) || cell.text.len() != 1 {
        return None;
    }
    cell.text.as_bytes().first().copied()
}

fn matching_open(close: u8) -> Option<u8> {
    match close {
        b'}' => Some(b'{'),
        b']' => Some(b'['),
        b')' => Some(b'('),
        _ => None,
    }
}

fn matching_close(open: u8) -> Option<u8> {
    match open {
        b'{' => Some(b'}'),
        b'[' => Some(b']'),
        b'(' => Some(b')'),
        _ => None,
    }
}

fn find_previous_matching_bracket(
    grid: &Grid,
    row: usize,
    col: usize,
    close: u8,
) -> Option<(usize, usize)> {
    let open = matching_open(close)?;
    let mut row = row;
    let mut col = col;
    let mut depth = 1usize;
    loop {
        if col == 0 {
            if row == 0 {
                return None;
            }
            row -= 1;
            let length = copy_line_length(grid, row);
            if length == 0 {
                continue;
            }
            col = length - 1;
        } else {
            col -= 1;
        }
        match copy_ascii_cell(grid, row, col) {
            Some(ch) if ch == close => depth += 1,
            Some(ch) if ch == open => {
                depth -= 1;
                if depth == 0 {
                    return Some((row, col));
                }
            }
            _ => {}
        }
    }
}

fn find_next_matching_bracket(
    grid: &Grid,
    row: usize,
    col: usize,
    open: u8,
) -> Option<(usize, usize)> {
    let close = matching_close(open)?;
    let mut row = row;
    let mut col = col;
    let mut depth = 1usize;
    loop {
        let length = copy_line_length(grid, row);
        if col + 1 < length {
            col += 1;
        } else {
            row += 1;
            if row >= grid.rows.len() {
                return None;
            }
            col = 0;
            if copy_line_length(grid, row) == 0 {
                continue;
            }
        }
        match copy_ascii_cell(grid, row, col) {
            Some(ch) if ch == open => depth += 1,
            Some(ch) if ch == close => {
                depth -= 1;
                if depth == 0 {
                    return Some((row, col));
                }
            }
            _ => {}
        }
    }
}

pub(super) fn move_copy_matching_bracket(state: &mut CopyState, backward: bool, vi: bool) {
    if state.grid.rows.is_empty() {
        return;
    }
    let original = state.cursor.clone();
    if backward {
        let mut candidate = original.clone();
        let mut close = copy_ascii_cell(&state.grid, candidate.row, candidate.col)
            .filter(|&ch| matching_open(ch).is_some());
        if close.is_none() && !vi && candidate.col > 0 {
            candidate.col -= 1;
            close = copy_ascii_cell(&state.grid, candidate.row, candidate.col)
                .filter(|&ch| matching_open(ch).is_some());
        }
        if let Some(close) = close {
            if let Some((row, col)) =
                find_previous_matching_bracket(&state.grid, candidate.row, candidate.col, close)
            {
                position_copy_cursor(state, row, col);
            }
        } else if !vi {
            let mut cursor = original;
            move_previous(&mut cursor, &state.grid, false, "}])", true);
            position_copy_cursor(state, cursor.row, cursor.col);
        }
        return;
    }

    if vi {
        let mut candidate = original;
        loop {
            if let Some(ch) = copy_ascii_cell(&state.grid, candidate.row, candidate.col) {
                if matching_open(ch).is_some() {
                    if let Some((row, col)) = find_previous_matching_bracket(
                        &state.grid,
                        candidate.row,
                        candidate.col,
                        ch,
                    ) {
                        position_copy_cursor(state, row, col);
                    }
                    return;
                }
                if matching_close(ch).is_some() {
                    if let Some((row, col)) =
                        find_next_matching_bracket(&state.grid, candidate.row, candidate.col, ch)
                    {
                        position_copy_cursor(state, row, col);
                    }
                    return;
                }
            }
            let length = copy_line_length(&state.grid, candidate.row);
            if candidate.col < length {
                candidate.col += 1;
            } else if state.grid.rows[candidate.row].wrapped
                && candidate.row + 1 < state.grid.rows.len()
            {
                candidate.row += 1;
                candidate.col = 0;
            } else {
                return;
            }
        }
    }

    let mut candidate = original.clone();
    let mut open = copy_ascii_cell(&state.grid, candidate.row, candidate.col)
        .filter(|&ch| matching_close(ch).is_some());
    if open.is_none() {
        candidate.col += 1;
        open = copy_ascii_cell(&state.grid, candidate.row, candidate.col)
            .filter(|&ch| matching_close(ch).is_some());
    }
    if let Some(open) = open {
        if let Some((row, col)) =
            find_next_matching_bracket(&state.grid, candidate.row, candidate.col, open)
        {
            position_copy_cursor(state, row, col);
        }
    } else {
        let mut cursor = original;
        move_next_end(&mut cursor, &state.grid, false, "{[(");
        position_copy_cursor(state, cursor.row, cursor.col);
    }
}

pub(super) fn copy_first_nonblank(grid: &Grid, row: usize) -> usize {
    let limit = copy_line_length(grid, row);
    (0..limit)
        .find(|&col| {
            let cursor = CopyCursor { row, col };
            !copy_cell_in_set(grid, &cursor, " \t")
        })
        .unwrap_or(limit)
}

pub(super) fn move_copy_prompt(state: &mut CopyState, forward: bool, output: bool) {
    let is_prompt_row = |row: usize| {
        state.grid.rows[row]
            .cells
            .iter()
            .any(|cell| cell.semantic == CellSemantic::Prompt)
    };
    let mut candidates = Vec::new();
    for row in 0..state.grid.rows.len() {
        if !is_prompt_row(row) {
            continue;
        }
        let target = if output {
            let mut target = row;
            while target < state.grid.rows.len() && is_prompt_row(target) {
                target += 1;
            }
            target.min(state.grid.rows.len().saturating_sub(1))
        } else {
            row
        };
        if candidates.last() != Some(&target) {
            candidates.push(target);
        }
    }
    let target = if forward {
        candidates.into_iter().find(|&row| row > state.cursor.row)
    } else {
        candidates
            .into_iter()
            .rev()
            .find(|&row| row < state.cursor.row)
    };
    if let Some(row) = target {
        position_copy_cursor(state, row, copy_first_nonblank(&state.grid, row));
    }
}

pub(super) fn repeat_copy_jump(state: &mut CopyState, reverse: bool) {
    let Some(jump) = state.jump.clone() else {
        return;
    };
    let kind = if reverse {
        match jump.kind {
            CopyJumpKind::Forward => CopyJumpKind::Backward,
            CopyJumpKind::Backward => CopyJumpKind::Forward,
            CopyJumpKind::ToForward => CopyJumpKind::ToBackward,
            CopyJumpKind::ToBackward => CopyJumpKind::ToForward,
        }
    } else {
        jump.kind
    };
    let row = state.cursor.row;
    let cells = &state.grid.rows[row].cells;
    let found = match kind {
        CopyJumpKind::Forward | CopyJumpKind::ToForward => {
            ((state.cursor.col + 1)..cells.len()).find(|&col| cells[col].text == jump.text)
        }
        CopyJumpKind::Backward | CopyJumpKind::ToBackward => (0..state.cursor.col)
            .rev()
            .find(|&col| cells[col].text == jump.text),
    };
    let Some(mut col) = found else {
        return;
    };
    match kind {
        CopyJumpKind::ToForward => col = col.saturating_sub(1),
        CopyJumpKind::ToBackward => col = (col + 1).min(cells.len().saturating_sub(1)),
        _ => {}
    }
    position_copy_cursor(state, row, col);
}

pub(super) fn synchronize_copy_selection(state: &mut CopyState) {
    if let Some(selection) = state
        .selection
        .as_mut()
        .filter(|selection| selection.active)
    {
        selection.end = (state.cursor.row, state.cursor.col);
    }
}

pub(super) fn select_copy_line(state: &mut CopyState, vi: bool) {
    copy_reader_start_of_line(&mut state.cursor, &state.grid);
    let anchor = (state.cursor.row, state.cursor.col);
    copy_reader_end_of_line(&mut state.cursor, &state.grid, vi);
    state.selection = Some(CopySelection {
        anchor,
        end: (state.cursor.row, state.cursor.col),
        active: true,
    });
}

/// tmux's `window_copy_cmd_select_word`, built out of the same wrap-following
/// word motions as `previous-word` and `next-word-end` so that a word straddling
/// the right margin is selected whole.
pub(super) fn select_copy_word(state: &mut CopyState, vi: bool, separators: &str) {
    if state.grid.rows.is_empty() {
        return;
    }
    move_previous(&mut state.cursor, &state.grid, vi, separators, false);
    let anchor = (state.cursor.row, state.cursor.col);

    // tmux's "handle single character words": the word ends where it starts
    // unless the cell after it holds text, and that cell is on the next row
    // when the word starts in the last column of a wrapped one.
    let mut next = CopyCursor {
        row: anchor.0,
        col: anchor.1 + 1,
    };
    if next.col > state.grid.cols.saturating_sub(1) as usize && state.grid.rows[anchor.0].wrapped {
        next = CopyCursor {
            row: anchor.0 + 1,
            col: 0,
        };
    }
    if anchor.1 >= copy_line_length(&state.grid, anchor.0)
        || !copy_cell_in_set(&state.grid, &next, " \t")
    {
        move_next_end(&mut state.cursor, &state.grid, vi, separators);
    }

    state.selection = Some(CopySelection {
        anchor,
        end: (state.cursor.row, state.cursor.col),
        active: true,
    });
}

#[derive(Clone, Debug)]
struct CopySearchCell {
    row: usize,
    col: usize,
    byte_start: usize,
    byte_end: usize,
    col_end: usize,
}

#[repr(C)]
// POSIX specifies `regmatch_t` as two `regoff_t` fields. libc omits the
// declaration on glibc targets but exports it on Darwin, so keep the shared ABI
// shape here and call the same libc regex functions tmux uses.
struct PosixRegMatch {
    rm_so: libc::regoff_t,
    rm_eo: libc::regoff_t,
}

unsafe extern "C" {
    #[link_name = "regcomp"]
    fn posix_regcomp(
        regex: *mut libc::regex_t,
        pattern: *const libc::c_char,
        flags: libc::c_int,
    ) -> libc::c_int;
    #[link_name = "regexec"]
    fn posix_regexec(
        regex: *const libc::regex_t,
        text: *const libc::c_char,
        count: libc::size_t,
        matches: *mut PosixRegMatch,
        flags: libc::c_int,
    ) -> libc::c_int;
    #[link_name = "regfree"]
    fn posix_regfree(regex: *mut libc::regex_t);
}

struct PosixRegex {
    raw: libc::regex_t,
}

impl PosixRegex {
    fn compile(pattern: &str, case_insensitive: bool) -> Option<Self> {
        const REG_EXTENDED: libc::c_int = 1;
        const REG_ICASE: libc::c_int = 2;

        let pattern = std::ffi::CString::new(pattern).ok()?;
        let mut raw = std::mem::MaybeUninit::<libc::regex_t>::uninit();
        let flags = REG_EXTENDED | if case_insensitive { REG_ICASE } else { 0 };
        if unsafe { posix_regcomp(raw.as_mut_ptr(), pattern.as_ptr(), flags) } != 0 {
            return None;
        }
        Some(Self {
            raw: unsafe { raw.assume_init() },
        })
    }

    fn find(&self, text: &[u8], not_bol: bool) -> Option<(usize, usize)> {
        const REG_NOTBOL: libc::c_int = 1;

        let text = std::ffi::CString::new(text).ok()?;
        let mut matched = PosixRegMatch {
            rm_so: -1,
            rm_eo: -1,
        };
        let result = unsafe {
            posix_regexec(
                &self.raw,
                text.as_ptr(),
                1,
                &mut matched,
                if not_bol { REG_NOTBOL } else { 0 },
            )
        };
        if result != 0 || matched.rm_so < 0 || matched.rm_eo <= matched.rm_so {
            return None;
        }
        Some((matched.rm_so as usize, matched.rm_eo as usize))
    }
}

impl Drop for PosixRegex {
    fn drop(&mut self) {
        unsafe { posix_regfree(&mut self.raw) };
    }
}

pub(super) fn search_uses_posix_regex(pattern: &str) -> bool {
    pattern.bytes().any(|byte| b"^$*+()?[].\\".contains(&byte))
}

fn copy_search_matches(grid: &Grid, pattern: &str, regex: bool) -> Vec<CopySearchMatch> {
    if pattern.is_empty() || grid.rows.is_empty() {
        return Vec::new();
    }
    let case_insensitive = pattern
        .bytes()
        .all(|byte| byte == byte.to_ascii_lowercase());
    let regex = if regex {
        let Some(regex) = PosixRegex::compile(pattern, case_insensitive) else {
            return Vec::new();
        };
        Some(regex)
    } else {
        None
    };
    let mut matches = Vec::new();
    let mut first_row = 0;

    while first_row < grid.rows.len() {
        let mut text = String::new();
        let mut cells = Vec::new();
        let mut row = first_row;
        loop {
            for col in 0..grid.cols as usize {
                let Some(cell) = grid.rows[row].cells.get(col) else {
                    let byte_start = text.len();
                    text.push(' ');
                    cells.push(CopySearchCell {
                        row,
                        col,
                        byte_start,
                        byte_end: text.len(),
                        col_end: col + 1,
                    });
                    continue;
                };
                if copy_cell_is_padding(cell) {
                    continue;
                }
                let byte_start = text.len();
                if cell.text.is_empty() {
                    text.push(' ');
                } else {
                    text.push_str(&cell.text);
                }
                let width = usize::from(matches!(cell.width, CellWidth::Wide)) + 1;
                cells.push(CopySearchCell {
                    row,
                    col,
                    byte_start,
                    byte_end: text.len(),
                    col_end: col.saturating_add(width).min(grid.cols as usize),
                });
            }
            if !grid.rows[row].wrapped || row + 1 == grid.rows.len() {
                break;
            }
            row += 1;
        }

        let ranges = if let Some(regex) = regex.as_ref() {
            posix_regex_match_ranges(&text, regex)
        } else {
            literal_match_ranges(&text, pattern, case_insensitive)
        };
        for (byte_start, byte_end) in ranges {
            let Some(first) = cells.iter().position(|cell| cell.byte_start == byte_start) else {
                continue;
            };
            let Some(last) = cells
                .iter()
                .rposition(|cell| cell.byte_end == byte_end)
                .filter(|&last| last >= first)
            else {
                continue;
            };
            let mut segments: Vec<(usize, usize, usize)> = Vec::new();
            for cell in &cells[first..=last] {
                if let Some((_, _, to)) = segments
                    .last_mut()
                    .filter(|segment| segment.0 == cell.row && segment.2 == cell.col)
                {
                    *to = cell.col_end;
                } else {
                    segments.push((cell.row, cell.col, cell.col_end));
                }
            }
            let last_cell = &cells[last];
            let mut end_after = (last_cell.row, last_cell.col_end);
            if end_after.1 == grid.cols as usize
                && grid.rows[last_cell.row].wrapped
                && last_cell.row + 1 < grid.rows.len()
            {
                end_after = (last_cell.row + 1, 0);
            }
            matches.push(CopySearchMatch {
                start: (cells[first].row, cells[first].col),
                end_after,
                segments,
            });
        }
        first_row = row + 1;
    }
    matches.sort_by_key(|found| found.start);
    matches
}

fn posix_regex_match_ranges(text: &str, regex: &PosixRegex) -> Vec<(usize, usize)> {
    let text = text.as_bytes();
    let mut ranges = Vec::new();
    let mut offset = 0;
    while offset <= text.len() {
        let Some((start, end)) = regex.find(&text[offset..], offset != 0) else {
            break;
        };
        ranges.push((offset + start, offset + end));
        offset += end;
    }
    ranges
}

fn literal_match_ranges(text: &str, pattern: &str, case_insensitive: bool) -> Vec<(usize, usize)> {
    let text_bytes = text.as_bytes();
    let pattern = pattern.as_bytes();
    if pattern.is_empty() || pattern.len() > text_bytes.len() {
        return Vec::new();
    }
    let mut ranges = Vec::new();
    let mut at = 0;
    while at + pattern.len() <= text_bytes.len() {
        let candidate = &text_bytes[at..at + pattern.len()];
        let equal = if case_insensitive {
            candidate
                .iter()
                .zip(pattern)
                .all(|(&left, &right)| left.to_ascii_lowercase() == right)
        } else {
            candidate == pattern
        };
        if equal && text.is_char_boundary(at) && text.is_char_boundary(at + pattern.len()) {
            ranges.push((at, at + pattern.len()));
            at += pattern.len();
        } else {
            at += 1;
        }
    }
    ranges
}

pub(super) fn start_copy_search(
    state: &mut CopyState,
    pattern: &str,
    direction: CopySearchDirection,
    regex: bool,
    vi: bool,
    wrap: bool,
) {
    let regex = regex && search_uses_posix_regex(pattern);
    state.search = Some(CopySearch {
        pattern: pattern.to_string(),
        regex,
        direction,
        last_direction: direction,
        matches: copy_search_matches(&state.grid, pattern, regex),
    });
    move_to_copy_search_match(state, direction, vi, wrap, false);
    state.search_marks = true;
}

pub(super) fn incremental_search_argument(argument: &str) -> Option<(char, &str)> {
    let mut chars = argument.char_indices();
    let (_, prefix) = chars.next()?;
    let pattern_at = chars.next().map(|(at, _)| at).unwrap_or(argument.len());
    Some((prefix, &argument[pattern_at..]))
}

pub(super) fn incremental_copy_search(
    state: &mut CopyState,
    argument: &str,
    forward_command: bool,
    vi: bool,
    wrap: bool,
) {
    let Some((prefix, pattern)) = incremental_search_argument(argument) else {
        return;
    };
    if state.incremental_search_origin.is_none() {
        state.incremental_search_origin = Some(CopySearchOrigin {
            cursor: state.cursor.clone(),
            desired_col: state.desired_col,
            scroll: state.scroll,
        });
    } else if state
        .search
        .as_ref()
        .is_some_and(|search| search.pattern != pattern)
    {
        let Some(origin) = state.incremental_search_origin.as_ref() else {
            return;
        };
        state.cursor = origin.cursor.clone();
        state.desired_col = origin.desired_col;
        state.scroll = origin.scroll;
        // tmux passes a false rectangle flag to window_copy_cursor_limit here;
        // that still addresses the final character rather than an emacs-style
        // insertion point after it.
        state.cursor.col = copy_cursor_limit(&state.grid, state.cursor.row, true);
        state.desired_col = state.cursor.col;
    }

    if pattern.is_empty() {
        if let Some(search) = state.search.as_mut() {
            search.matches.clear();
        }
        state.search_marks = false;
        return;
    }

    let direction = match (forward_command, prefix) {
        (true, '=' | '+') | (false, '+') => Some(CopySearchDirection::Forward),
        (true, '-') | (false, '=' | '-') => Some(CopySearchDirection::Backward),
        _ => None,
    };
    let Some(direction) = direction else {
        return;
    };
    start_copy_search(state, pattern, direction, false, vi, wrap);
    if state
        .search
        .as_ref()
        .is_none_or(|search| search.matches.is_empty())
    {
        state.search_marks = false;
    }
}

pub(super) fn repeat_copy_search(state: &mut CopyState, reverse: bool, vi: bool, wrap: bool) {
    let Some(search) = state.search.as_mut() else {
        return;
    };
    let preserve_marks = !search.matches.is_empty();
    if search.matches.is_empty() {
        search.matches = copy_search_matches(&state.grid, &search.pattern, search.regex);
    }
    let direction = if reverse {
        match search.direction {
            CopySearchDirection::Backward => CopySearchDirection::Forward,
            CopySearchDirection::Forward => CopySearchDirection::Backward,
        }
    } else {
        search.direction
    };
    move_to_copy_search_match(state, direction, vi, wrap, preserve_marks);
    state.search_marks = true;
}

fn move_to_copy_search_match(
    state: &mut CopyState,
    direction: CopySearchDirection,
    vi: bool,
    wrap: bool,
    preserve_marks: bool,
) {
    let cursor = (state.cursor.row, state.cursor.col);
    let Some(search) = state.search.as_mut() else {
        return;
    };
    search.last_direction = direction;
    let found = match direction {
        CopySearchDirection::Forward => search
            .matches
            .iter()
            .position(|found| found.start > cursor || (!vi && found.start == cursor))
            .or_else(|| wrap.then_some(0).filter(|_| !search.matches.is_empty())),
        CopySearchDirection::Backward => search
            .matches
            .iter()
            .rposition(|found| {
                found.start < cursor || (!wrap && cursor == (0, 0) && found.start == cursor)
            })
            .or_else(|| wrap.then(|| search.matches.len().checked_sub(1)).flatten()),
    };
    let Some(found) = found else {
        if !preserve_marks {
            search.matches.clear();
        }
        return;
    };
    let matched = &search.matches[found];
    let point = if direction == CopySearchDirection::Forward && !vi {
        matched.end_after
    } else {
        matched.start
    };
    state.cursor.row = point.0;
    state.cursor.col = point.1;
    state.desired_col = point.1;
    ensure_copy_cursor_visible(state);
}

pub(super) fn clear_copy_search_marks_after_command(
    state: &mut CopyState,
    command: &str,
    vi: bool,
) {
    if command.starts_with("search-") {
        return;
    }
    let vi_keeps_marks = vi
        && matches!(
            command,
            "history-top"
                | "history-bottom"
                | "page-up"
                | "page-down"
                | "halfpage-up"
                | "halfpage-down"
                | "cursor-up"
                | "cursor-down"
                | "cursor-left"
                | "cursor-right"
                | "cursor-centre-vertical"
                | "cursor-centre-horizontal"
                | "start-of-line"
                | "end-of-line"
                | "goto-line"
                | "top-line"
                | "middle-line"
                | "bottom-line"
                | "next-paragraph"
                | "previous-paragraph"
                | "scroll-up"
                | "scroll-down"
                | "previous-word"
                | "previous-space"
                | "next-word"
                | "next-space"
                | "next-word-end"
                | "next-space-end"
                | "other-end"
        );
    if vi_keeps_marks {
        return;
    }
    let had_marks = state
        .search
        .as_ref()
        .is_some_and(|search| !search.matches.is_empty());
    if !had_marks {
        return;
    }
    if let Some(search) = state.search.as_mut() {
        search.matches.clear();
    }
    state.search_marks = false;
    state.incremental_search_origin = None;
}

pub(crate) fn copy_search_segments(
    state: &CopyState,
    vi: bool,
) -> Vec<(usize, usize, usize, bool)> {
    let Some(search) = state.search.as_ref() else {
        return Vec::new();
    };
    let cursor = (state.cursor.row, state.cursor.col);
    let current = search.matches.iter().position(|found| {
        found
            .segments
            .iter()
            .any(|&(row, from, to)| row == cursor.0 && cursor.1 >= from && cursor.1 < to)
            || (!vi
                && search.last_direction == CopySearchDirection::Forward
                && found.end_after == cursor)
    });
    search
        .matches
        .iter()
        .enumerate()
        .flat_map(|(index, found)| {
            found
                .segments
                .iter()
                .map(move |&(row, from, to)| (row, from, to, current == Some(index)))
        })
        .collect()
}

fn copy_reader_handle_wrap(cursor: &mut CopyCursor, grid: &Grid, line_end: &mut usize) -> bool {
    let last_row = grid.rows.len().saturating_sub(1);
    while cursor.col > *line_end {
        if cursor.row == last_row {
            return false;
        }
        cursor.col = 0;
        cursor.row += 1;
        *line_end = if grid.rows[cursor.row].wrapped {
            grid.cols.saturating_sub(1) as usize
        } else {
            copy_line_length(grid, cursor.row)
        };
    }
    true
}

/// tmux's `grid_reader_cursor_right`.
///
/// How far right the cursor may go depends on the caller: `all` is a rectangle
/// selection, which can put the cursor anywhere on the screen rather than on
/// the text; `onemore` is emacs keys, which allow the column just past the last
/// cell where vi keys stop on it. With `wrap`, a cursor already at that limit
/// moves to the start of the next line instead of staying put — which is what
/// makes `cursor-right` walk off the end of a line, and why the word-end
/// helpers ask for the un-wrapped form.
pub(super) fn copy_reader_cursor_right(
    cursor: &mut CopyCursor,
    grid: &Grid,
    wrap: bool,
    all: bool,
    onemore: bool,
) {
    let end = if all {
        grid.cols as usize
    } else {
        let length = copy_line_length(grid, cursor.row);
        if onemore {
            length
        } else {
            length.saturating_sub(1)
        }
    };
    if wrap && cursor.col >= end {
        // The last row of the grid has nowhere to wrap to.
        if cursor.row + 1 < grid.rows.len() {
            cursor.row += 1;
            cursor.col = 0;
        }
        return;
    }
    if cursor.col < end {
        cursor.col += 1;
        while cursor.col < end {
            let Some(cell) = grid.rows[cursor.row].cells.get(cursor.col) else {
                break;
            };
            if !copy_cell_is_padding(cell) {
                break;
            }
            cursor.col += 1;
        }
    }
}

pub(super) fn copy_reader_cursor_left(cursor: &mut CopyCursor, grid: &Grid, wrap: bool) {
    while cursor.col > 0
        && grid.rows[cursor.row]
            .cells
            .get(cursor.col)
            .is_some_and(copy_cell_is_padding)
    {
        cursor.col -= 1;
    }
    if cursor.col == 0 && cursor.row > 0 && (wrap || grid.rows[cursor.row - 1].wrapped) {
        cursor.row -= 1;
        cursor.col = copy_line_length(grid, cursor.row);
    } else if cursor.col > 0 {
        cursor.col -= 1;
    }
}

/// tmux's `grid_reader_cursor_start_of_line` with wrapping, as
/// `window_copy_cursor_start_of_line` always asks for it.
///
/// A soft-wrapped row is a continuation, not a line of its own, so the logical
/// line starts on the first row above that nothing wrapped into.
pub(super) fn copy_reader_start_of_line(cursor: &mut CopyCursor, grid: &Grid) {
    while cursor.row > 0 && grid.rows[cursor.row - 1].wrapped {
        cursor.row -= 1;
    }
    cursor.col = 0;
}

/// tmux's `grid_reader_cursor_end_of_line` with wrapping, followed by its
/// `window_copy_cursor_limit`: walk down while the row wraps into the next one,
/// then stop at that row's own limit.
pub(super) fn copy_reader_end_of_line(cursor: &mut CopyCursor, grid: &Grid, vi: bool) {
    let last_row = grid.rows.len().saturating_sub(1);
    while cursor.row < last_row && grid.rows[cursor.row].wrapped {
        cursor.row += 1;
    }
    cursor.col = copy_cursor_limit(grid, cursor.row, vi);
}

/// tmux's `grid_reader_cursor_back_to_indentation`: the first non-blank cell of
/// the logical line, which can begin above a soft-wrapped row and run on below
/// it. A line with nothing on it leaves the cursor where it was.
pub(super) fn copy_reader_back_to_indentation(cursor: &mut CopyCursor, grid: &Grid) {
    let original = cursor.clone();
    copy_reader_start_of_line(cursor, grid);
    let last_row = grid.rows.len().saturating_sub(1);
    loop {
        let row = cursor.row;
        let found = (0..copy_line_length(grid, row))
            .find(|&col| !copy_cell_in_set(grid, &CopyCursor { row, col }, " \t"));
        if let Some(col) = found {
            cursor.col = col;
            return;
        }
        if row == last_row || !grid.rows[row].wrapped {
            break;
        }
        cursor.row += 1;
    }
    *cursor = original;
}

/// tmux's `grid_reader_cursor_previous_word`; `already` is its argument of the
/// same name, set when the caller wants the word *before* the cursor rather
/// than the start of the word the cursor is already inside.
pub(super) fn move_previous(
    cursor: &mut CopyCursor,
    grid: &Grid,
    vi: bool,
    separators: &str,
    already: bool,
) {
    if grid.rows.is_empty() {
        return;
    }
    let stop_at_eol = !vi;
    let word_is_letters = if already || copy_cell_in_set(grid, cursor, " \t") {
        loop {
            if cursor.col > 0 {
                cursor.col -= 1;
                if !copy_cell_in_set(grid, cursor, " \t") {
                    break !copy_cell_in_set(grid, cursor, separators);
                }
            } else {
                if cursor.row == 0 {
                    return;
                }
                cursor.row -= 1;
                cursor.col = copy_line_length(grid, cursor.row);
                if stop_at_eol && cursor.col > 0 {
                    cursor.col -= 1;
                    let at_eol = copy_cell_in_set(grid, cursor, " \t");
                    cursor.col += 1;
                    if at_eol {
                        break false;
                    }
                }
            }
        }
    } else {
        !copy_cell_in_set(grid, cursor, separators)
    };

    loop {
        let old = cursor.clone();
        if cursor.col == 0 {
            if cursor.row == 0 || !grid.rows[cursor.row - 1].wrapped {
                *cursor = old;
                break;
            }
            cursor.row -= 1;
            cursor.col = grid.cols as usize;
        }
        if cursor.col > 0 {
            cursor.col -= 1;
        }
        if copy_cell_in_set(grid, cursor, " \t")
            || word_is_letters == copy_cell_in_set(grid, cursor, separators)
        {
            *cursor = old;
            break;
        }
    }
    clamp_copy_cursor(cursor, grid, vi);
}

pub(super) fn move_next_start(cursor: &mut CopyCursor, grid: &Grid, vi: bool, separators: &str) {
    if grid.rows.is_empty() {
        return;
    }
    let mut line_end = if grid.rows[cursor.row].wrapped {
        grid.cols.saturating_sub(1) as usize
    } else {
        copy_line_length(grid, cursor.row)
    };
    if !copy_reader_handle_wrap(cursor, grid, &mut line_end) {
        return;
    }
    if !copy_cell_in_set(grid, cursor, " \t") {
        if copy_cell_in_set(grid, cursor, separators) {
            loop {
                cursor.col += 1;
                if !copy_reader_handle_wrap(cursor, grid, &mut line_end)
                    || !copy_cell_in_set(grid, cursor, separators)
                    || copy_cell_in_set(grid, cursor, " \t")
                {
                    break;
                }
            }
        } else {
            loop {
                cursor.col += 1;
                if !copy_reader_handle_wrap(cursor, grid, &mut line_end)
                    || copy_cell_in_set(grid, cursor, separators)
                    || copy_cell_in_set(grid, cursor, " \t")
                {
                    break;
                }
            }
        }
    }
    // A tab is crossed in one step: its whole run is whitespace, and stopping
    // inside it would leave the cursor on padding.
    while copy_reader_handle_wrap(cursor, grid, &mut line_end) {
        let width = copy_cell_set_width(grid, cursor, " \t");
        if width == 0 {
            break;
        }
        cursor.col += width;
    }
    clamp_copy_cursor(cursor, grid, vi);
}

fn copy_reader_next_word_end(cursor: &mut CopyCursor, grid: &Grid, separators: &str) {
    let mut line_end = if grid.rows[cursor.row].wrapped {
        grid.cols.saturating_sub(1) as usize
    } else {
        copy_line_length(grid, cursor.row)
    };
    while copy_reader_handle_wrap(cursor, grid, &mut line_end) {
        if copy_cell_in_set(grid, cursor, " \t") {
            cursor.col += 1;
        } else if copy_cell_in_set(grid, cursor, separators) {
            loop {
                cursor.col += 1;
                if !copy_reader_handle_wrap(cursor, grid, &mut line_end)
                    || !copy_cell_in_set(grid, cursor, separators)
                    || copy_cell_in_set(grid, cursor, " \t")
                {
                    return;
                }
            }
        } else {
            loop {
                cursor.col += 1;
                if !copy_reader_handle_wrap(cursor, grid, &mut line_end)
                    || copy_cell_in_set(grid, cursor, " \t")
                    || copy_cell_in_set(grid, cursor, separators)
                {
                    return;
                }
            }
        }
    }
}

pub(super) fn move_next_end(cursor: &mut CopyCursor, grid: &Grid, vi: bool, separators: &str) {
    if grid.rows.is_empty() {
        return;
    }
    if vi {
        if !copy_cell_in_set(grid, cursor, " \t") {
            copy_reader_cursor_right(cursor, grid, false, false, false);
        }
        copy_reader_next_word_end(cursor, grid, separators);
        copy_reader_cursor_left(cursor, grid, true);
    } else {
        copy_reader_next_word_end(cursor, grid, separators);
    }
    clamp_copy_cursor(cursor, grid, vi);
}

pub(crate) fn copy_selection_segments(state: &CopyState, vi: bool) -> Vec<(usize, usize, usize)> {
    let Some(selection) = state.selection.as_ref() else {
        return Vec::new();
    };
    let grid = &state.grid;
    if grid.rows.is_empty() {
        return Vec::new();
    }
    if state.rectangle {
        let first_row = selection.anchor.0.min(selection.end.0);
        let last_row = selection.anchor.0.max(selection.end.0);
        let first_col = selection.anchor.1.min(selection.end.1);
        let last_col = selection
            .anchor
            .1
            .max(selection.end.1)
            .saturating_add(usize::from(vi));
        return (first_row..=last_row)
            .map(|row| {
                let length = copy_line_length(grid, row);
                (row, first_col.min(length), last_col.min(length))
            })
            .collect();
    }

    let (start, end) = if selection.anchor <= selection.end {
        (selection.anchor, selection.end)
    } else {
        (selection.end, selection.anchor)
    };
    (start.0..=end.0)
        .map(|row| {
            let row_end = if grid.rows[row].wrapped {
                grid.cols as usize
            } else {
                copy_line_length(grid, row)
            };
            let from = if row == start.0 { start.1 } else { 0 };
            let requested_end = if row == end.0 {
                end.1.saturating_add(usize::from(vi))
            } else {
                grid.cols as usize
            };
            (row, from.min(row_end), requested_end.min(row_end))
        })
        .collect()
}

fn append_copy_cells(output: &mut String, grid: &Grid, row: usize, from: usize, to: usize) {
    if from >= to {
        return;
    }
    for cell in &grid.rows[row].cells[from..to] {
        if cell.width == CellWidth::SpacerTail {
            continue;
        }
        // tmux's `window_copy_copy_line` puts the tab back rather than copying
        // the blanks it painted, so what is yanked is what was typed.
        if cell.tab {
            output.push('\t');
        } else if cell.text.is_empty() {
            output.push(' ');
        } else {
            output.push_str(&cell.text);
        }
    }
}

/// tmux's `window_copy_match_at_cursor`: the marked search match the cursor sits
/// in — or the one that ends immediately before it — which a copy command with no
/// selection yanks instead of nothing.
pub(super) fn copy_match_at_cursor(state: &CopyState) -> Option<String> {
    if !state.search_marks {
        return None;
    }
    let matches = &state.search.as_ref()?.matches;
    let covers = |candidate: &CopySearchMatch, row: usize, col: usize| {
        candidate
            .segments
            .iter()
            .any(|&(at_row, from, to)| at_row == row && col >= from && col < to)
    };
    let (row, col) = (state.cursor.row, state.cursor.col);
    let previous = if col == 0 {
        row.checked_sub(1)
            .map(|row| (row, usize::from(state.grid.cols).saturating_sub(1)))
    } else {
        Some((row, col - 1))
    };
    let matched = matches
        .iter()
        .find(|candidate| covers(candidate, row, col))
        .or_else(|| {
            let (row, col) = previous?;
            matches.iter().find(|candidate| covers(candidate, row, col))
        })?;
    let mut output = String::new();
    for &(row, from, to) in &matched.segments {
        append_copy_cells(&mut output, &state.grid, row, from, to);
    }
    (!output.is_empty()).then_some(output)
}

pub(super) fn copy_selection(state: &CopyState, vi: bool) -> String {
    let Some(selection) = state.selection.as_ref() else {
        return String::new();
    };
    let grid = &state.grid;
    if grid.rows.is_empty() {
        return String::new();
    }
    let end = if selection.anchor <= selection.end {
        selection.end
    } else {
        selection.anchor
    };

    let segments = copy_selection_segments(state, vi);
    if state.rectangle {
        // A rectangle asks every row for the same column range, so tmux's
        // per-line rule decides the newlines exactly as it does for a linear
        // selection: `window_copy_copy_line` ends each row, and the final one
        // is taken back only when the range stopped inside the row's text.
        // That is what makes a rectangle wider than its last row keep a
        // trailing newline while one cutting through it does not.
        let last_col = selection
            .anchor
            .1
            .max(selection.end.1)
            .saturating_add(usize::from(vi));
        let mut output = String::new();
        for (row_index, from, to) in segments {
            let row = &grid.rows[row_index];
            let row_end = if row.wrapped {
                grid.cols as usize
            } else {
                copy_line_length(grid, row_index)
            };
            append_copy_cells(&mut output, grid, row_index, from, to);
            if !row.wrapped || last_col.min(row_end) != row_end {
                output.push('\n');
            }
        }
        let end_line_length = copy_line_length(grid, end.0);
        if (!vi || last_col <= end_line_length)
            && (!grid.rows[end.0].wrapped || last_col != end_line_length)
            && output.ends_with('\n')
        {
            output.pop();
        }
        return output;
    }

    let end_line_length = copy_line_length(grid, end.0);
    let end_col = end.1.min(end_line_length);
    let last_end = end_col.saturating_add(usize::from(vi));
    let mut output = String::new();

    for (row_index, from, to) in segments {
        let row = &grid.rows[row_index];
        let row_end = if row.wrapped {
            grid.cols as usize
        } else {
            copy_line_length(grid, row_index)
        };
        let requested_end = if row_index == end.0 {
            last_end
        } else {
            grid.cols as usize
        };
        append_copy_cells(&mut output, grid, row_index, from, to);
        if !row.wrapped || requested_end != row_end {
            output.push('\n');
        }
    }

    if state.selection_mode != CopySelectionMode::Line
        && (!vi || last_end <= end_line_length)
        && (!grid.rows[end.0].wrapped || last_end != end_line_length)
        && output.ends_with('\n')
    {
        output.pop();
    }
    output
}

pub(super) fn copy_from_cursor_to_line_end(state: &CopyState, vi: bool) -> String {
    let mut temporary = state.clone();
    let anchor = (temporary.cursor.row, temporary.cursor.col);
    copy_reader_end_of_line(&mut temporary.cursor, &temporary.grid, vi);
    temporary.selection = Some(CopySelection {
        anchor,
        end: (temporary.cursor.row, temporary.cursor.col),
        active: false,
    });
    copy_selection(&temporary, vi)
}

pub(super) fn copy_current_line(state: &CopyState, vi: bool) -> String {
    let mut temporary = state.clone();
    copy_reader_start_of_line(&mut temporary.cursor, &temporary.grid);
    let anchor = (temporary.cursor.row, temporary.cursor.col);
    copy_reader_end_of_line(&mut temporary.cursor, &temporary.grid, vi);
    temporary.selection = Some(CopySelection {
        anchor,
        end: (temporary.cursor.row, temporary.cursor.col),
        active: false,
    });
    copy_selection(&temporary, vi)
}
