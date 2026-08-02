use super::*;

pub(crate) fn copy_mode_active(state: &Arc<Mutex<ServerState>>, target: &str) -> bool {
    state
        .lock()
        .ok()
        .is_some_and(|st| st.active_copy_state(target).is_some())
}

pub(super) fn set_copy_mode_state(
    state: &Arc<Mutex<ServerState>>,
    target: &str,
    active: bool,
    page_up: bool,
) {
    if let Ok(mut st) = state.lock() {
        let _ = st.set_pane_mode(target, active.then_some("copy-mode"));
        if active && page_up {
            let vi = match st.option_for_target(target, "mode-keys") {
                Some(mode) => mode == "vi",
                None => crate::server::options::mode_keys_default() == "vi",
            };
            let separators = st
                .option_for_target(target, "word-separators")
                .unwrap_or(" !\"#$%&'()*+,-./:;<=>?@[\\]^`{|}~")
                .to_string();
            let _ = st.copy_mode_command(target, "page-up", vi, &separators);
        }
    }
}

pub(super) fn copy_table_name(state: &Arc<Mutex<ServerState>>, target: &str) -> &'static str {
    let mode = state.lock().ok().and_then(|st| {
        st.option_for_target(target, "mode-keys")
            .map(str::to_string)
    });
    let vi = match mode.as_deref() {
        Some(mode) => mode == "vi",
        None => crate::server::options::mode_keys_default() == "vi",
    };
    if vi {
        "copy-mode-vi"
    } else {
        "copy-mode"
    }
}

pub(super) fn copy_mode_uses_vi_keys(st: &ServerState, target: &str) -> bool {
    match st.option_for_target(target, "mode-keys") {
        Some(mode) => mode == "vi",
        None => crate::server::options::mode_keys_default() == "vi",
    }
}

pub(super) fn copy_style_escape(
    st: &ServerState,
    target: &str,
    option: &str,
    fallback: &str,
    terminal: &dyn TerminalCapabilities,
) -> Vec<u8> {
    status::option_style_escape_for(st, target, option, fallback, terminal)
}

pub(super) fn render_copy_mark_and_position(
    out: &mut Vec<u8>,
    st: &ServerState,
    target: &str,
    copy: &CopyState,
    screen_top: u16,
    screen_left: u16,
    height: u16,
    width: u16,
    terminal: &dyn TerminalCapabilities,
) {
    let view_top = copy.grid.scrollback_rows.saturating_sub(copy.scroll);
    if let Some((row, _)) = copy.mark {
        if row >= view_top && row < view_top.saturating_add(height as usize) {
            let style = copy_style_escape(
                st,
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
            out.extend_from_slice(&style);
            for cell in copy.grid.rows[row].cells.iter().take(width as usize) {
                if !matches!(
                    cell.width,
                    ghostty_sys::GridCellWidth::SpacerTail | ghostty_sys::GridCellWidth::SpacerHead
                ) {
                    if cell.text.is_empty() {
                        out.push(b' ');
                    } else {
                        out.extend_from_slice(cell.text.as_bytes());
                    }
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
            copy.search_count
                .map(|count| count.to_string())
                .unwrap_or_default(),
        )
        .set("copy_cursor_x", copy.cursor.col.to_string())
        .set("copy_cursor_y", copy.cursor.row.to_string());
    let configured = st
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
    out.extend_from_slice(&copy_style_escape(
        st,
        target,
        "copy-mode-position-style",
        "bg=yellow,fg=black",
        terminal,
    ));
    out.extend_from_slice(text.as_bytes());
    append_terminal_style_reset(out, terminal);
}

pub(super) fn copy_line_number_width(st: &ServerState, target: &str, copy: &CopyState) -> usize {
    if st
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

pub(super) fn render_copy_line_number(
    out: &mut Vec<u8>,
    st: &ServerState,
    target: &str,
    copy: &CopyState,
    physical_row: usize,
    current: bool,
    width: usize,
    terminal: &dyn TerminalCapabilities,
) {
    if width == 0 {
        return;
    }
    let mode = st
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
        copy_style_escape(
            st,
            target,
            "copy-mode-current-line-number-style",
            "fg=yellow",
            terminal,
        )
    } else {
        copy_style_escape(
            st,
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

pub(super) fn render_copy_selection(
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
        for cell in &copy.grid.rows[row].cells[from..to] {
            if matches!(
                cell.width,
                ghostty_sys::GridCellWidth::SpacerTail | ghostty_sys::GridCellWidth::SpacerHead
            ) {
                continue;
            }
            if cell.text.is_empty() {
                out.push(b' ');
            } else {
                out.extend_from_slice(cell.text.as_bytes());
            }
        }
        append_terminal_style_reset(out, terminal);
    }
}

pub(super) fn render_copy_search(
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
        for cell in &copy.grid.rows[row].cells[from..to] {
            if matches!(
                cell.width,
                ghostty_sys::GridCellWidth::SpacerTail | ghostty_sys::GridCellWidth::SpacerHead
            ) {
                continue;
            }
            if cell.text.is_empty() {
                out.push(b' ');
            } else {
                out.extend_from_slice(cell.text.as_bytes());
            }
        }
        append_terminal_style_reset(out, terminal);
    }
}
