//! Copy-mode input actions and viewport rendering for attached clients.

use super::super::mouse::MouseEvent;
use super::super::state::{
    CopyState, ServerState, SharedState, copy_search_segments, copy_selection_segments,
};
use super::super::term::TerminalCapabilities;
use super::super::{format, options, status};
use super::append_terminal_style_reset;
use hmux_vt::{CellWidth, GridCell};

pub(super) struct CopyModeAction {
    page_up: bool,
    page_down: bool,
    slider: bool,
    mouse: Option<MouseEvent>,
    begin_selection: bool,
    /// `copy-mode -e`: leave the mode again once a scroll reaches the bottom,
    /// which is how a wheel-entered mode gets out of the way.
    scroll_exit: bool,
}

impl CopyModeAction {
    pub(super) fn new(
        page_up: bool,
        page_down: bool,
        slider: bool,
        mouse: Option<MouseEvent>,
        begin_selection: bool,
        scroll_exit: bool,
    ) -> Self {
        Self {
            page_up,
            page_down,
            slider,
            mouse,
            begin_selection,
            scroll_exit,
        }
    }

    pub(super) fn apply(self, state: &SharedState, target: &str) {
        enter(state, target, self.page_up, self.scroll_exit);
        let mut state = state.borrow_mut();
        let vi = uses_vi_keys(&state, target);
        if let Some(mouse) = self.mouse {
            let position = mouse.pane_position();
            // `-M` opens a drag: the selection starts where the button went
            // down, not where the pointer has already reached.
            let start = if self.begin_selection {
                mouse.pane_last_position().unwrap_or(position)
            } else {
                position
            };
            let _ = state.position_copy_cursor_from_mouse(target, start.x, start.y, vi);
            if self.slider {
                let grab = mouse
                    .target
                    .as_ref()
                    .and_then(|target| target.slider_offset);
                let _ = state.scroll_copy_to_mouse(
                    target,
                    if grab.is_some() {
                        mouse.position.y
                    } else {
                        position.y
                    },
                    grab,
                    vi,
                    self.scroll_exit,
                );
            }
            if self.begin_selection {
                run_command(&mut state, target, "begin-selection", vi, "");
                // tmux ends `window_copy_start_drag` with one drag update, so
                // the pointer's current position is already selected.
                state.drag_copy_selection_to_mouse(target, position.x, position.y, vi);
            }
        }
        if self.page_down {
            run_command(&mut state, target, "page-down", vi, "");
        }
    }

    /// Re-entering copy mode from its own key table only honors `-u`.
    pub(super) fn reactivate(self, state: &SharedState, target: &str) {
        enter(state, target, self.page_up, self.scroll_exit);
    }
}

pub(super) fn is_active(state: &ServerState, target: &str) -> bool {
    state.active_copy_state(target).is_some()
}

pub(super) fn key_table(state: &ServerState, target: &str) -> &'static str {
    if uses_vi_keys(state, target) {
        "copy-mode-vi"
    } else {
        "copy-mode"
    }
}

pub(super) fn uses_vi_keys(state: &ServerState, target: &str) -> bool {
    match state.option_for_target(target, "mode-keys") {
        Some(mode) => mode == "vi",
        None => options::mode_keys_default() == "vi",
    }
}

fn enter(state: &SharedState, target: &str, page_up: bool, scroll_exit: bool) {
    {
        let mut state = state.borrow_mut();
        let _ = state.set_pane_mode_with_scroll_exit(target, Some("copy-mode"), scroll_exit);
        if page_up {
            let vi = uses_vi_keys(&state, target);
            run_command(
                &mut state,
                target,
                "page-up",
                vi,
                " !\"#$%&'()*+,-./:;<=>?@[\\]^`{|}~",
            );
        }
    }
}

fn run_command(
    state: &mut ServerState,
    target: &str,
    command: &str,
    vi: bool,
    fallback_separators: &str,
) {
    let separators = state
        .option_for_target(target, "word-separators")
        .unwrap_or(fallback_separators)
        .to_string();
    let _ = state.copy_mode_command(target, command, vi, &separators);
}

pub(super) struct CopyModeView<'a> {
    state: &'a ServerState,
    target: &'a str,
    copy: &'a CopyState,
    terminal: &'a dyn TerminalCapabilities,
    vi: bool,
    line_number_width: usize,
}

impl<'a> CopyModeView<'a> {
    pub(super) fn new(
        state: &'a ServerState,
        target: &'a str,
        copy: &'a CopyState,
        width: u16,
        terminal: &'a dyn TerminalCapabilities,
    ) -> Self {
        let line_number_width =
            line_number_width(state, target, copy).min(width.saturating_sub(1) as usize);
        Self {
            state,
            target,
            copy,
            terminal,
            vi: uses_vi_keys(state, target),
            line_number_width,
        }
    }

    pub(super) fn serialized_len(&self) -> usize {
        self.copy.vt.len()
    }

    pub(super) fn rows(&self, height: u16) -> Vec<Vec<u8>> {
        self.copy
            .vt_rows()
            .skip(self.view_top())
            .take(height as usize)
            .map(<[u8]>::to_vec)
            .collect()
    }

    pub(super) fn cursor(&self, height: u16, width: u16) -> Vec<u8> {
        let row = self
            .copy
            .cursor
            .row
            .saturating_sub(self.view_top())
            .min(height.saturating_sub(1) as usize)
            + 1;
        let col = self
            .line_number_width
            .saturating_add(self.copy.cursor.col)
            .min(width.saturating_sub(1) as usize)
            + 1;
        format!("\x1b[{row};{col}H").into_bytes()
    }

    pub(super) fn line_number_width(&self) -> usize {
        self.line_number_width
    }

    pub(super) fn render_line_number(&self, out: &mut Vec<u8>, viewport_row: usize) {
        let physical_row = self.view_top().saturating_add(viewport_row);
        render_line_number(
            self,
            out,
            physical_row,
            physical_row == self.copy.cursor.row,
        );
    }

    pub(super) fn render_overlays(
        &self,
        out: &mut Vec<u8>,
        screen_top: u16,
        screen_left: u16,
        height: u16,
        width: u16,
    ) {
        let content_left = screen_left.saturating_add(self.line_number_width as u16);
        let content_width = width.saturating_sub(self.line_number_width as u16);
        render_search(
            out,
            self.copy,
            self.vi,
            &style_escape(
                self.state,
                self.target,
                "copy-mode-match-style",
                "bg=cyan,fg=black",
                self.terminal,
            ),
            &style_escape(
                self.state,
                self.target,
                "copy-mode-current-match-style",
                "bg=magenta,fg=black",
                self.terminal,
            ),
            screen_top,
            content_left,
            height,
            content_width,
            self.terminal,
        );
        render_selection(
            out,
            self.copy,
            self.vi,
            &style_escape(
                self.state,
                self.target,
                "copy-mode-selection-style",
                "bg=yellow,fg=black",
                self.terminal,
            ),
            screen_top,
            content_left,
            height,
            content_width,
            self.terminal,
        );
        render_mark_and_position(self, out, screen_top, content_left, height, content_width);
    }

    fn view_top(&self) -> usize {
        self.copy
            .grid
            .scrollback_rows
            .saturating_sub(self.copy.scroll)
    }
}

fn style_escape(
    state: &ServerState,
    target: &str,
    option: &str,
    fallback: &str,
    terminal: &dyn TerminalCapabilities,
) -> Vec<u8> {
    status::option_style_escape_for(state, target, option, fallback, terminal)
}

fn line_number_width(state: &ServerState, target: &str, copy: &CopyState) -> usize {
    if state
        .option_for_target(target, "copy-mode-line-numbers")
        .unwrap_or("off")
        == "off"
    {
        0
    } else {
        let lines = copy
            .grid
            .scrollback_rows
            .saturating_add(copy.grid.viewport_rows as usize)
            .saturating_add(1);
        (lines.max(1).ilog10() as usize + 2).max(4)
    }
}

fn render_line_number(
    view: &CopyModeView<'_>,
    out: &mut Vec<u8>,
    physical_row: usize,
    current: bool,
) {
    let state = view.state;
    let target = view.target;
    let copy = view.copy;
    let width = view.line_number_width;
    let terminal = view.terminal;
    if width == 0 {
        return;
    }
    let mode = state
        .option_for_target(target, "copy-mode-line-numbers")
        .unwrap_or("off");
    let absolute = physical_row + 1;
    let relative = physical_row.abs_diff(copy.cursor.row);
    let number = match mode {
        "absolute" => absolute,
        "hybrid" if current => absolute,
        "relative" | "hybrid" => relative,
        _ => copy.grid.scrollback_rows.abs_diff(physical_row),
    };
    let style = if current {
        style_escape(
            state,
            target,
            "copy-mode-current-line-number-style",
            "fg=yellow",
            terminal,
        )
    } else {
        style_escape(
            state,
            target,
            "copy-mode-line-number-style",
            "fg=white,dim",
            terminal,
        )
    };
    out.extend_from_slice(&style);
    out.extend_from_slice(format!("{number:>w$} ", w = width - 1).as_bytes());
    append_terminal_style_reset(out, terminal);
}

fn render_mark_and_position(
    view: &CopyModeView<'_>,
    out: &mut Vec<u8>,
    screen_top: u16,
    screen_left: u16,
    height: u16,
    width: u16,
) {
    let state = view.state;
    let target = view.target;
    let copy = view.copy;
    let terminal = view.terminal;
    let view_top = copy.grid.scrollback_rows.saturating_sub(copy.scroll);
    if let Some((row, mark_column)) = copy.mark {
        if row >= view_top && row < view_top.saturating_add(height as usize) {
            let style = style_escape(
                state,
                target,
                "copy-mode-mark-style",
                "bg=red,fg=black",
                terminal,
            );
            out.extend_from_slice(
                format!(
                    "\x1b[{};{}H",
                    screen_top + (row - view_top) as u16 + 1,
                    screen_left + 1
                )
                .as_bytes(),
            );
            // tmux styles the whole marked row, exchanges the two colours on
            // the marked cell itself (`window_copy_update_style`), and clears
            // the blank tail of the row — which carries the background alone.
            let variant = |variant| {
                status::option_style_escape_variant(
                    state,
                    target,
                    "copy-mode-mark-style",
                    "bg=red,fg=black",
                    terminal,
                    variant,
                )
            };
            let reversed = variant(status::StyleVariant::Reversed);
            let background_only = variant(status::StyleVariant::BackgroundOnly);
            let cells = &copy.grid.rows[row].cells;
            let used = cells
                .iter()
                .rposition(|cell| !cell.text.is_empty())
                .map_or(0, |last| last + 1);
            out.extend_from_slice(&style);
            for (column, cell) in cells.iter().take(width as usize).enumerate() {
                if matches!(cell.width, CellWidth::SpacerTail) {
                    continue;
                }
                if column == used {
                    out.extend_from_slice(&background_only);
                }
                if column == mark_column {
                    out.extend_from_slice(&reversed);
                }
                if cell.text.is_empty() {
                    out.push(b' ');
                } else {
                    out.extend_from_slice(cell.text.as_bytes());
                }
                if column == mark_column {
                    out.extend_from_slice(if column >= used {
                        &background_only
                    } else {
                        &style
                    });
                }
            }
            append_terminal_style_reset(out, terminal);
        }
    }
    if copy.hide_position || width == 0 || height == 0 {
        return;
    }
    let mut vars = format::Vars::new();
    vars.set("copy_position", copy.scroll.to_string())
        .set("copy_position_limit", copy.grid.scrollback_rows.to_string())
        .set(
            "search_count",
            copy.search_count()
                .map(|count| count.to_string())
                .unwrap_or_default(),
        )
        .set("copy_cursor_x", copy.cursor.col.to_string())
        .set("copy_cursor_y", copy.cursor.row.to_string())
        .set("top_line_time", copy.top_line_time().to_string());
    let configured = state
        .option_for_target(target, "copy-mode-position-format")
        .filter(|value| !value.is_empty());
    let source = configured.unwrap_or("[#{copy_position}/#{copy_position_limit}]");
    let align_right = configured.is_none() || source.contains("#[align=right]");
    let text = format::expand(source, &vars).replace("#[align=right]", "");
    let text = format::trim_right(&text, width as usize);
    let col = if align_right {
        screen_left + width.saturating_sub(format::display_width(&text) as u16) + 1
    } else {
        screen_left + 1
    };
    out.extend_from_slice(format!("\x1b[{};{}H", screen_top + 1, col).as_bytes());
    out.extend_from_slice(&style_escape(
        state,
        target,
        "copy-mode-position-style",
        "bg=yellow,fg=black",
        terminal,
    ));
    out.extend_from_slice(text.as_bytes());
    append_terminal_style_reset(out, terminal);
}

#[allow(clippy::too_many_arguments)]
fn render_selection(
    out: &mut Vec<u8>,
    copy: &CopyState,
    vi: bool,
    style: &[u8],
    screen_top: u16,
    screen_left: u16,
    height: u16,
    width: u16,
    terminal: &dyn TerminalCapabilities,
) {
    if height == 0 || width == 0 {
        return;
    }
    let view_top = copy.grid.scrollback_rows.saturating_sub(copy.scroll);
    let view_bottom = view_top.saturating_add(height as usize);
    for (row, from, to) in copy_selection_segments(copy, vi) {
        if row < view_top || row >= view_bottom {
            continue;
        }
        let from = from.min(width as usize);
        let to = to.min(width as usize);
        if from >= to {
            continue;
        }
        out.extend_from_slice(
            format!(
                "\x1b[{};{}H",
                screen_top + (row - view_top) as u16 + 1,
                screen_left + from as u16 + 1,
            )
            .as_bytes(),
        );
        out.extend_from_slice(style);
        render_cells(out, &copy.grid.rows[row].cells[from..to]);
        append_terminal_style_reset(out, terminal);
    }
}

#[allow(clippy::too_many_arguments)]
fn render_search(
    out: &mut Vec<u8>,
    copy: &CopyState,
    vi: bool,
    other_style: &[u8],
    current_style: &[u8],
    screen_top: u16,
    screen_left: u16,
    height: u16,
    width: u16,
    terminal: &dyn TerminalCapabilities,
) {
    if height == 0 || width == 0 {
        return;
    }
    let view_top = copy.grid.scrollback_rows.saturating_sub(copy.scroll);
    let view_bottom = view_top.saturating_add(height as usize);
    for (row, from, to, current) in copy_search_segments(copy, vi) {
        if row < view_top || row >= view_bottom {
            continue;
        }
        let from = from.min(width as usize);
        let to = to.min(width as usize);
        if from >= to {
            continue;
        }
        out.extend_from_slice(
            format!(
                "\x1b[{};{}H",
                screen_top + (row - view_top) as u16 + 1,
                screen_left + from as u16 + 1,
            )
            .as_bytes(),
        );
        out.extend_from_slice(if current { current_style } else { other_style });
        render_cells(out, &copy.grid.rows[row].cells[from..to]);
        append_terminal_style_reset(out, terminal);
    }
}

fn render_cells(out: &mut Vec<u8>, cells: &[GridCell]) {
    for cell in cells {
        if matches!(cell.width, CellWidth::SpacerTail) {
            continue;
        }
        if cell.text.is_empty() {
            out.push(b' ');
        } else {
            out.extend_from_slice(cell.text.as_bytes());
        }
    }
}
