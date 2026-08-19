//! Driving a pane's mode: entering and leaving copy mode and the mode-tree
//! views, and dispatching the commands bound in `copy-mode` and `choose-*`.
//!
//! The mode's own data lives in `state::copy` and `state::mode`; what is here
//! is the `ServerState` side — which pane is in which mode, and how a key or
//! mouse event turns into a mutation of that mode's state.

use std::io;

use hmux_vt::ScreenSnapshot;

use super::copy::*;
use super::mode::update_mode_edit_item;
use super::sizing::pane_slider;
use super::target::pane_not_found;
use crate::server::options::PaneHook;
/// tmux's `MODE_TREE_HELP_DEFAULT_WIDTH`.
const MODE_TREE_HELP_DEFAULT_WIDTH: u16 = 39;

use super::{
    ModeBindingUpdate, ModeEdit, ModeKind, ModePrompt, ModeTarget, ModeView, ModeViewKeyResult,
    PopupRequest, RenderInvalidation, ServerState,
};

impl ServerState {
    pub(crate) fn set_pane_mode(&mut self, target: &str, mode: Option<&str>) -> io::Result<()> {
        self.set_pane_mode_with_scroll_exit(target, mode, false)
    }

    pub(crate) fn set_pane_mode_with_scroll_exit(
        &mut self,
        target: &str,
        mode: Option<&str>,
        scroll_exit: bool,
    ) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let (pane_id, was_in_mode) = {
            let node = &self.window(resolved.session, resolved.window).panes[resolved.pane];
            (node.id, node.mode.is_some())
        };
        {
            let node = &mut self.window_mut(resolved.session, resolved.window).panes[resolved.pane];
            node.mode_view = None;
            if mode.is_some() && node.copy.is_some() {
                // tmux keeps the existing window-mode entry when `copy-mode` is
                // invoked again. In particular, do not silently refresh the
                // frozen backing screen or discard its cursor and selection.
                node.mode = mode.map(str::to_string);
            } else {
                let copy = if mode.is_some() {
                    let (ScreenSnapshot { grid, vt }, (col, row)) = node.pane.copy_snapshot();
                    let vt_rows = copy_vt_row_ranges(&vt);
                    Some(CopyState {
                        backing: CopyBacking::PaneSnapshot,
                        cursor: CopyCursor {
                            row: grid.scrollback_rows.saturating_add(row as usize),
                            col: col as usize,
                        },
                        desired_col: col as usize,
                        desired_line_end: 0,
                        selection: None,
                        rectangle: false,
                        selection_mode: CopySelectionMode::Character,
                        mark: None,
                        jump: None,
                        hide_position: false,
                        search: node.search_string.as_ref().map(|pattern| CopySearch {
                            pattern: pattern.clone(),
                            regex: node.search_regex,
                            direction: CopySearchDirection::Backward,
                            last_direction: CopySearchDirection::Backward,
                            matches: Vec::new(),
                        }),
                        search_marks: true,
                        incremental_search_origin: None,
                        prefix: 1,
                        scroll_exit,
                        recentre: CopyRecentre {
                            state: CopyRecentreState::Middle,
                            line: 0,
                        },
                        grid,
                        vt,
                        vt_rows,
                        scroll: 0,
                    })
                } else {
                    None
                };
                node.mode = mode.map(str::to_string);
                node.copy = copy;
            }
        }
        self.invalidate_session(session_id, RenderInvalidation::MODE);
        // tmux notifies from `window_pane_set_mode`/`window_pane_reset_mode`,
        // that is only when the pane actually enters or leaves a mode.
        if was_in_mode != mode.is_some() {
            self.notify_pane(PaneHook::ModeChanged, pane_id);
        }
        Ok(())
    }

    pub(crate) fn enter_mode_view(&mut self, target: &str, view: ModeView) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let (pane_id, was_in_mode) = {
            let node = &self.window(resolved.session, resolved.window).panes[resolved.pane];
            (node.id, node.mode.is_some())
        };
        let node = &mut self.window_mut(resolved.session, resolved.window).panes[resolved.pane];
        node.mode = Some(
            match view.kind {
                ModeKind::Tree => "tree-mode",
                ModeKind::Client => "client-mode",
                ModeKind::Buffer => "buffer-mode",
                ModeKind::Customize => "options-mode",
                ModeKind::Clock => "clock-mode",
            }
            .to_string(),
        );
        node.copy = None;
        node.mode_view = Some(view);
        self.invalidate_session(session_id, RenderInvalidation::MODE);
        if !was_in_mode {
            self.notify_pane(PaneHook::ModeChanged, pane_id);
        }
        Ok(())
    }

    pub(crate) fn mode_view_active(&self, target: &str) -> bool {
        self.resolve(target)
            .and_then(|resolved| {
                self.window(resolved.session, resolved.window)
                    .panes
                    .get(resolved.pane)
            })
            .is_some_and(|pane| pane.mode_view.is_some())
    }

    pub(crate) fn copy_mode_active(&self, target: &str) -> bool {
        self.resolve(target)
            .and_then(|resolved| {
                self.window(resolved.session, resolved.window)
                    .panes
                    .get(resolved.pane)
            })
            .is_some_and(|pane| pane.copy.is_some())
    }

    pub(crate) fn set_copy_mode_prefix(&mut self, target: &str, prefix: u32) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let copy = self.window_mut(resolved.session, resolved.window).panes[resolved.pane]
            .copy
            .as_mut()
            .ok_or_else(|| io::Error::other("not in a mode"))?;
        copy.prefix = prefix;
        Ok(())
    }

    pub(crate) fn take_copy_mode_prefix(&mut self, target: &str) -> io::Result<u32> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let copy = self.window_mut(resolved.session, resolved.window).panes[resolved.pane]
            .copy
            .as_mut()
            .ok_or_else(|| io::Error::other("not in a mode"))?;
        Ok(std::mem::replace(&mut copy.prefix, 1))
    }

    /// Handle one key in a generic window mode. A returned argv is the selected
    /// item's command and is executed by the attached-client command context.
    /// The popup tmux's `popup_editor` opens for a buffer: the buffer's bytes
    /// in a temporary file, the `editor` option run on it, and the load that
    /// reads it back once the editor exits.
    fn buffer_editor_popup(&self, name: &str) -> Option<PopupRequest> {
        let editor = self
            .server_options()
            .get("editor")
            .unwrap_or("vi")
            .to_owned();
        if editor.is_empty() {
            return None;
        }
        let data = self
            .buffers()
            .iter()
            .find(|(buffer, _)| buffer == name)
            .map(|(_, data)| data.clone())?;
        // One file per buffer: a second edit of the same buffer rewrites it
        // with the buffer's current bytes.
        let safe = name
            .chars()
            .map(|glyph| if glyph.is_alphanumeric() { glyph } else { '_' })
            .collect::<String>();
        let path = std::env::temp_dir().join(format!("hmux-editor-{}-{safe}", std::process::id()));
        std::fs::write(&path, &data).ok()?;
        // tmux hands `editor path` to the shell, so a configured editor with
        // its own arguments works.
        let argv = vec![
            "/bin/sh".to_owned(),
            "-c".to_owned(),
            format!("{editor} {}", path.to_string_lossy()),
        ];
        Some(PopupRequest {
            title: String::new(),
            pane: None,
            mouse: None,
            content: None,
            argv,
            environment: self.job_environment(None).as_ref().clone(),
            cwd: None,
            width: Some("90%".to_owned()),
            height: Some("90%".to_owned()),
            x: None,
            y: None,
            close_on_exit: true,
            close_on_success: false,
            close_on_key: false,
            border: true,
            on_close: vec![
                "load-buffer".to_owned(),
                "-b".to_owned(),
                name.to_owned(),
                path.to_string_lossy().into_owned(),
            ],
            on_close_remove: Some(path),
        })
    }

    /// tmux's `mode_tree_display_help`: the key list a mode-tree shows for
    /// `F1`, in a popup that closes on any key. The box is the mode's own
    /// width and one row per line, centred on the client — a client too small
    /// for it gets nothing, as tmux gives it nothing.
    fn mode_tree_help_popup(kind: &ModeKind, cols: u16, rows: u16) -> Option<PopupRequest> {
        const COMMON: &[(&str, &str)] = &[
            ("Up, k", "Move cursor up"),
            ("Down, j", "Move cursor down"),
            ("g", "Go to top"),
            ("G", "Go to bottom"),
            ("PPage, C-b", "Page up"),
            ("NPage, C-f", "Page down"),
            ("Left, h", "Collapse %1"),
            ("Right, l", "Expand %1"),
            ("M--", "Collapse all %1s"),
            ("M-+", "Expand all %1s"),
            ("t", "Toggle %1 tag"),
            ("T", "Untag all %1s"),
            ("C-t", "Tag all %1s"),
            ("C-s", "Search forward"),
            ("n", "Repeat search forward"),
            ("N", "Repeat search backward"),
            ("f", "Filter %1s"),
            ("O", "Change sort order"),
            ("r", "Reverse sort order"),
            ("v", "Toggle preview"),
        ];
        const TREE: &[(&str, &str)] = &[
            ("Enter", "Choose selected item"),
            ("S-Up", "Swap current and previous window"),
            ("S-Down", "Swap current and next window"),
            ("x", "Kill selected item"),
            ("X", "Kill tagged items"),
            ("<", "Scroll previews left"),
            (">", "Scroll previews right"),
            ("m", "Set the marked pane"),
            ("M", "Clear the marked pane"),
            (":", "Run a command for each tagged item"),
            ("f", "Enter a format"),
            ("H", "Jump to the starting pane"),
        ];
        const CLIENT: &[(&str, &str)] = &[
            ("Enter", "Choose selected %1"),
            ("d", "Detach selected %1"),
            ("D", "Detach tagged %1s"),
            ("x", "Detach selected %1"),
            ("X", "Detach tagged %1s"),
            ("z", "Suspend selected %1"),
            ("Z", "Suspend tagged %1s"),
            ("f", "Enter a filter"),
        ];
        const BUFFER: &[(&str, &str)] = &[
            ("Enter", "Paste selected %1"),
            ("p", "Paste selected %1"),
            ("P", "Paste tagged %1s"),
            ("d", "Delete selected %1"),
            ("D", "Delete tagged %1s"),
            ("e", "Open %1 in editor"),
            ("f", "Enter a filter"),
        ];
        // tmux's per-mode help width, and the default the tree's own overrides.
        let (own, item, own_width) = match kind {
            ModeKind::Tree => (TREE, "item", 51),
            ModeKind::Client => (CLIENT, "client", 0),
            ModeKind::Buffer => (BUFFER, "buffer", 0),
            _ => return None,
        };
        const END: &[(&str, &str)] = &[("q, Escape", "Exit mode")];
        let lines = COMMON
            .iter()
            .chain(own)
            .chain(END)
            .map(|(key, description)| {
                format!("{key:>11} \u{2502} {}", description.replace("%1", item))
            })
            .collect::<Vec<_>>();
        // tmux sizes the whole box — borders included — at the mode's width
        // and one row per line, so the two lines the borders take are the ones
        // the text scrolls past.
        let width = own_width.max(MODE_TREE_HELP_DEFAULT_WIDTH);
        let height = lines.len() as u16;
        if cols < width || rows < height {
            return None;
        }
        let content = lines.join("\r\n").into_bytes();
        Some(PopupRequest {
            title: String::new(),
            pane: None,
            mouse: None,
            content: Some(content),
            argv: Vec::new(),
            environment: Vec::new(),
            cwd: None,
            width: Some(width.to_string()),
            height: Some(height.to_string()),
            x: Some(((cols - width) / 2).to_string()),
            // A popup's `-y` names the row below its last line.
            y: Some(((rows - height) / 2 + height).to_string()),
            close_on_exit: false,
            close_on_success: false,
            close_on_key: true,
            border: true,
            on_close: Vec::new(),
            on_close_remove: None,
        })
    }

    /// The size of the client the mode is drawn on, which is what a popup is
    /// centred in. The largest client showing the session stands in for the
    /// one that pressed the key, which the state does not see from here.
    fn client_size_for_pane(&self, session: usize) -> (u16, u16) {
        let session_id = self.sessions[session].id;
        self.client_snapshots()
            .iter()
            .filter(|client| !client.control_mode && client.session_id == session_id)
            .map(|client| (client.cols, client.rows))
            .max()
            .unwrap_or((80, 24))
    }

    pub(crate) fn mode_view_key(
        &mut self,
        target: &str,
        key: &str,
        visible_rows: usize,
    ) -> io::Result<ModeViewKeyResult> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let node = &mut self.window_mut(resolved.session, resolved.window).panes[resolved.pane];
        let pane_id = node.id;
        let Some(view) = node.mode_view.as_mut() else {
            return Ok(ModeViewKeyResult::None);
        };
        if view.kind == ModeKind::Clock {
            node.mode = None;
            node.mode_view = None;
            self.invalidate_session(session_id, RenderInvalidation::MODE);
            // tmux reports the leave from `window_pane_reset_mode`, wherever
            // the key that ended the mode was handled.
            self.notify_pane(PaneHook::ModeChanged, pane_id);
            return Ok(ModeViewKeyResult::None);
        }

        let prompt = match key {
            "/" | "C-s" => Some(ModePrompt::Search),
            "f" => Some(ModePrompt::Filter {
                initial: view.filter.clone(),
            }),
            ":" if view.kind == ModeKind::Tree => view
                .items
                .get(view.selected)
                .and_then(|item| item.prompt_target.clone())
                .map(|item_target| ModePrompt::Command { item_target }),
            "s" if view.kind == ModeKind::Customize => view
                .items
                .get(view.selected)
                .and_then(|item| item.edit.clone())
                .map(ModePrompt::Edit),
            _ => None,
        };
        if let Some(prompt) = prompt {
            return Ok(ModeViewKeyResult::Prompt(prompt));
        }

        // tmux's `mode_tree_display_help`, which the mode-tree answers before
        // it looks at its own action keys.
        if matches!(key, "F1" | "C-h") {
            let kind = view.kind.clone();
            let (cols, rows) = self.client_size_for_pane(resolved.session);
            self.invalidate_session(session_id, RenderInvalidation::MODE);
            return Ok(match Self::mode_tree_help_popup(&kind, cols, rows) {
                Some(request) => ModeViewKeyResult::Popup(Box::new(request)),
                None => ModeViewKeyResult::None,
            });
        }
        if let Some(result) = mode_view_action(view, key) {
            let ModeAction {
                command,
                removed,
                exit,
                confirm,
            } = result;
            for index in removed.iter().rev() {
                view.remove(*index);
            }
            let emptied = view.items.is_empty() && !removed.is_empty();
            let left = exit || emptied;
            if left {
                node.mode = None;
                node.mode_view = None;
            }
            self.invalidate_session(session_id, RenderInvalidation::MODE);
            if left {
                self.notify_pane(PaneHook::ModeChanged, pane_id);
            }
            return Ok(match (confirm, command) {
                (Some(prompt), command) if !command.is_empty() => {
                    ModeViewKeyResult::Confirm { prompt, command }
                }
                (_, command) if !command.is_empty() => ModeViewKeyResult::Command(command),
                _ => ModeViewKeyResult::None,
            });
        }

        let last = view.items.len().saturating_sub(1);
        let mut left = false;
        match key {
            "q" | "Escape" | "C-c" => {
                node.mode = None;
                node.mode_view = None;
                left = true;
            }
            "Up" | "k" | "C-p" => view.selected = view.selected.saturating_sub(1),
            "Down" | "j" | "C-n" => view.selected = (view.selected + 1).min(last),
            // tmux's mode-tree expansion: `+` and `-` on this row, `M-+` and
            // `M--` on every row.
            "+" | "-" => {
                let expand = key == "+";
                if let Some(item) = view.items.get_mut(view.selected) {
                    if item.expanded.is_some() {
                        item.expanded = Some(expand);
                    }
                }
            }
            "M-+" => view.expand_all(true),
            "M--" => view.expand_all(false),
            // tmux's mode-tree tagging: `t` toggles this row, `T` clears
            // every tag and `C-t` sets them all.
            "t" if view.kind != ModeKind::Customize => {
                if let Some(item) = view.items.get_mut(view.selected) {
                    item.tagged = !item.tagged;
                }
                // tmux's `mode_tree_key` moves on after tagging whenever it has
                // a mouse event to look at, and the key path always hands it
                // the client's, so a tagging key from the keyboard advances too.
                view.selected = (view.selected + 1).min(view.items.len().saturating_sub(1));
            }
            "T" => {
                for item in &mut view.items {
                    item.tagged = false;
                }
            }
            "C-t" => {
                for item in &mut view.items {
                    item.tagged = true;
                }
            }
            // tmux's buffer mode edits the selected buffer in a popup running
            // the `editor` option, and loads the file back when it closes.
            "e" if view.kind == ModeKind::Buffer => {
                let name = view
                    .items
                    .get(view.selected)
                    .and_then(|item| item.prompt_target.clone());
                if let Some(name) = name {
                    if let Some(request) = self.buffer_editor_popup(&name) {
                        return Ok(ModeViewKeyResult::Popup(Box::new(request)));
                    }
                }
                return Ok(ModeViewKeyResult::None);
            }
            "Home" | "g" => view.selected = 0,
            "End" | "G" => view.selected = last,
            "PageUp" | "C-b" => view.selected = view.selected.saturating_sub(visible_rows.max(1)),
            "PageDown" | "C-f" => view.selected = (view.selected + visible_rows.max(1)).min(last),
            "Enter" => {
                let command = view
                    .items
                    .get(view.selected)
                    .map(|item| item.command.clone());
                node.mode = None;
                node.mode_view = None;
                self.invalidate_session(session_id, RenderInvalidation::MODE);
                self.notify_pane(PaneHook::ModeChanged, pane_id);
                return Ok(command
                    .map(ModeViewKeyResult::Command)
                    .unwrap_or(ModeViewKeyResult::None));
            }
            _ => {}
        }
        if let Some(view) = node.mode_view.as_mut() {
            let height = visible_rows.max(1);
            if view.selected < view.scroll {
                view.scroll = view.selected;
            } else if view.selected >= view.scroll + height {
                view.scroll = view.selected + 1 - height;
            }
        }
        self.invalidate_session(session_id, RenderInvalidation::MODE);
        if left {
            self.notify_pane(PaneHook::ModeChanged, pane_id);
        }
        Ok(ModeViewKeyResult::None)
    }

    pub(crate) fn mode_view_search(&mut self, target: &str, search: &str) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let view = self.window_mut(resolved.session, resolved.window).panes[resolved.pane]
            .mode_view
            .as_mut()
            .ok_or_else(|| io::Error::other("not in a mode"))?;
        if !search.is_empty() && !view.items.is_empty() {
            let folded = search.to_lowercase();
            let start = (view.selected + 1) % view.items.len();
            if let Some(offset) = (0..view.items.len()).find(|offset| {
                view.items[(start + offset) % view.items.len()]
                    .label
                    .to_lowercase()
                    .contains(&folded)
            }) {
                let selected = (start + offset) % view.items.len();
                // Search can land on a child beneath collapsed rows. Open all
                // preceding branches so the flat item index remains a visible
                // row for rendering and the next navigation key.
                for item in view.items.iter_mut().take(selected + 1) {
                    if item.expanded.is_some() {
                        item.expanded = Some(true);
                    }
                }
                view.selected = selected;
                view.scroll = view.selected;
            }
        }
        self.invalidate_session(session_id, RenderInvalidation::MODE);
        Ok(())
    }

    pub(crate) fn mode_view_filter(&mut self, target: &str, filter: &str) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let view = self.window_mut(resolved.session, resolved.window).panes[resolved.pane]
            .mode_view
            .as_mut()
            .ok_or_else(|| io::Error::other("not in a mode"))?;
        view.filter = filter.to_string();
        if filter.is_empty() {
            view.items.clone_from(&view.all_items);
        } else {
            let folded = filter.to_lowercase();
            view.items = view
                .all_items
                .iter()
                .filter(|item| item.label.to_lowercase().contains(&folded))
                .cloned()
                .collect();
        }
        view.selected = 0;
        view.scroll = 0;
        self.invalidate_session(session_id, RenderInvalidation::MODE);
        Ok(())
    }

    pub(crate) fn mode_view_update_edit(
        &mut self,
        target: &str,
        edit: &ModeEdit,
        value: &str,
    ) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let view = self.window_mut(resolved.session, resolved.window).panes[resolved.pane]
            .mode_view
            .as_mut()
            .ok_or_else(|| io::Error::other("not in a mode"))?;
        for item in view.all_items.iter_mut().chain(view.items.iter_mut()) {
            update_mode_edit_item(item, edit, value);
        }
        self.invalidate_session(session_id, RenderInvalidation::MODE);
        Ok(())
    }

    pub(crate) fn mode_view_update_binding(
        &mut self,
        target: &str,
        update: ModeBindingUpdate,
    ) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let view = self.window_mut(resolved.session, resolved.window).panes[resolved.pane]
            .mode_view
            .as_mut()
            .ok_or_else(|| io::Error::other("not in a mode"))?;
        for item in view.all_items.iter_mut().chain(view.items.iter_mut()) {
            match item.edit.as_mut() {
                Some(ModeEdit::BindingCommand {
                    table: item_table,
                    key: item_key,
                    value,
                    note: item_note,
                    repeat: item_repeat,
                }) if item_table == &update.table && item_key == &update.key => {
                    value.clone_from(&update.command_text);
                    item_note.clone_from(&update.note);
                    *item_repeat = update.repeat;
                    item.label = format!(
                        "key {} {} command {}",
                        update.table, update.key, update.command_text
                    );
                }
                Some(ModeEdit::BindingNote {
                    table: item_table,
                    key: item_key,
                    value,
                    command: item_command,
                    repeat: item_repeat,
                }) if item_table == &update.table && item_key == &update.key => {
                    *value = update.note.as_deref().unwrap_or_default().to_string();
                    item_command.clear();
                    item_command.extend_from_slice(&update.command);
                    *item_repeat = update.repeat;
                    item.label = format!(
                        "key {} {} note {}",
                        update.table,
                        update.key,
                        update.note.as_deref().unwrap_or_default()
                    );
                }
                _ => {}
            }
        }
        self.invalidate_session(session_id, RenderInvalidation::MODE);
        Ok(())
    }

    pub(crate) fn set_copy_mode(
        &mut self,
        target: &str,
        source: Option<&str>,
        scroll_exit: bool,
        hide_position: bool,
    ) -> io::Result<()> {
        let target_resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let source_resolved = match source {
            Some(source) => self.resolve(source).ok_or_else(|| pane_not_found(source))?,
            None => target_resolved,
        };
        let snapshot = {
            let source = &self
                .window(source_resolved.session, source_resolved.window)
                .panes[source_resolved.pane];
            source.pane.copy_snapshot()
        };
        let session_id = self.sessions[target_resolved.session].id;
        let node = &mut self
            .window_mut(target_resolved.session, target_resolved.window)
            .panes[target_resolved.pane];
        if node.copy.is_none() {
            let (ScreenSnapshot { grid, vt }, (col, row)) = snapshot;
            let vt_rows = copy_vt_row_ranges(&vt);
            node.copy = Some(CopyState {
                backing: CopyBacking::PaneSnapshot,
                cursor: CopyCursor {
                    row: grid.scrollback_rows.saturating_add(row as usize),
                    col: col as usize,
                },
                desired_col: col as usize,
                desired_line_end: 0,
                selection: None,
                rectangle: false,
                selection_mode: CopySelectionMode::Character,
                mark: None,
                jump: None,
                hide_position,
                search: node.search_string.as_ref().map(|pattern| CopySearch {
                    pattern: pattern.clone(),
                    regex: node.search_regex,
                    direction: CopySearchDirection::Backward,
                    last_direction: CopySearchDirection::Backward,
                    matches: Vec::new(),
                }),
                search_marks: true,
                incremental_search_origin: None,
                prefix: 1,
                scroll_exit,
                recentre: CopyRecentre {
                    state: CopyRecentreState::Middle,
                    line: 0,
                },
                grid,
                vt,
                vt_rows,
                scroll: 0,
            });
        } else if let Some(copy) = node.copy.as_mut() {
            copy.hide_position = hide_position;
        }
        let pane_id = node.id;
        let entered = node.mode.is_none();
        node.mode = Some("copy-mode".to_string());
        self.invalidate_session(session_id, RenderInvalidation::MODE);
        if entered {
            self.notify_pane(PaneHook::ModeChanged, pane_id);
        }
        Ok(())
    }

    pub(crate) fn copy_mode_command(
        &mut self,
        target: &str,
        command: &str,
        vi: bool,
        separators: &str,
    ) -> io::Result<Option<String>> {
        self.copy_mode_command_with_argument(target, command, None, vi, separators)
    }

    pub(crate) fn position_copy_cursor_from_mouse(
        &mut self,
        target: &str,
        x: u16,
        y: u16,
        vi: bool,
    ) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let state = self.window_mut(resolved.session, resolved.window).panes[resolved.pane]
            .copy
            .as_mut()
            .ok_or_else(|| io::Error::other("not in a mode"))?;
        let view_top = state.grid.scrollback_rows.saturating_sub(state.scroll);
        position_copy_cursor(
            state,
            view_top.saturating_add(y as usize),
            (x as usize).min(copy_cursor_limit(
                &state.grid,
                view_top
                    .saturating_add(y as usize)
                    .min(state.grid.rows.len().saturating_sub(1)),
                vi,
            )),
        );
        Ok(())
    }

    /// Drag the active selection's far end to the pointer — tmux's
    /// `window_copy_drag_update`. `false` when the pane has no drag to
    /// continue, which leaves the report to the ordinary key tables.
    pub(crate) fn drag_copy_selection_to_mouse(
        &mut self,
        target: &str,
        x: u16,
        y: u16,
        vi: bool,
    ) -> bool {
        let Some(resolved) = self.resolve(target) else {
            return false;
        };
        let dragging = self.window(resolved.session, resolved.window).panes[resolved.pane]
            .copy
            .as_ref()
            .and_then(|copy| copy.selection.as_ref())
            .is_some_and(|selection| selection.active);
        if !dragging {
            return false;
        }
        if self
            .position_copy_cursor_from_mouse(target, x, y, vi)
            .is_err()
        {
            return false;
        }
        let state = self.window_mut(resolved.session, resolved.window).panes[resolved.pane]
            .copy
            .as_mut();
        if let Some(state) = state {
            synchronize_copy_selection(state);
        }
        true
    }

    pub(crate) fn set_copy_scroll_from_mouse(
        &mut self,
        target: &str,
        y: u16,
        height: u16,
        vi: bool,
    ) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let state = self.window_mut(resolved.session, resolved.window).panes[resolved.pane]
            .copy
            .as_mut()
            .ok_or_else(|| io::Error::other("not in a mode"))?;
        let denominator = height.saturating_sub(1).max(1) as usize;
        let from_top = state.grid.scrollback_rows.saturating_mul(y as usize) / denominator;
        state.scroll = state.grid.scrollback_rows.saturating_sub(from_top);
        let view_top = state.grid.scrollback_rows.saturating_sub(state.scroll);
        state.cursor.row = view_top.min(state.grid.rows.len().saturating_sub(1));
        state.cursor.col =
            state
                .cursor
                .col
                .min(copy_cursor_limit(&state.grid, state.cursor.row, vi));
        Ok(())
    }

    /// Drag a scrollbar's slider — tmux's `window_copy_scroll1`.
    ///
    /// `screen_row` is where the pointer is now and `grab` where inside the
    /// slider it took hold, so the grabbed row stays under the pointer; the
    /// view then moves by the inverse of the formula that drew the slider.
    fn slide_copy_scroll(&mut self, target: &str, screen_row: u16, grab: u16) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let window = self.window(resolved.session, resolved.window);
        let node = &window.panes[resolved.pane];
        let rect = window
            .pane_rect(node.id)
            .ok_or_else(|| io::Error::other("no pane rect"))?;
        let (_, slider_height) = pane_slider(node, rect.height);
        let bar = rect.height.max(1);
        let travel = bar.saturating_sub(slider_height);
        let row = i32::from(screen_row) - i32::from(rect.top) - i32::from(grab);
        let slider_top = u16::try_from(row.clamp(0, i32::from(travel))).unwrap_or(0);

        let copy = self.window_mut(resolved.session, resolved.window).panes[resolved.pane]
            .copy
            .as_mut()
            .ok_or_else(|| io::Error::other("not in a mode"))?;
        let size = copy.grid.scrollback_rows;
        let new_offset =
            (f64::from(slider_top) * ((size as f64 + f64::from(bar)) / f64::from(bar))) as usize;
        let offset = size - copy.scroll.min(size);
        let scrolled = copy.scroll as i64 + offset as i64 - new_offset as i64;
        let scroll = scrolled.clamp(0, size as i64) as usize;
        // The cursor keeps its row on screen, so its place in the grid moves
        // with the view.
        let moved = scroll as i64 - copy.scroll as i64;
        copy.scroll = scroll;
        copy.cursor.row = (copy.cursor.row as i64 - moved)
            .clamp(0, copy.grid.rows.len().saturating_sub(1) as i64)
            as usize;
        Ok(())
    }

    pub(crate) fn scroll_copy_to_mouse(
        &mut self,
        target: &str,
        y: u16,
        grab: Option<u16>,
        vi: bool,
        scroll_exit: bool,
    ) -> io::Result<bool> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let height = self.window(resolved.session, resolved.window).panes[resolved.pane]
            .copy
            .as_ref()
            .ok_or_else(|| io::Error::other("not in a mode"))?
            .grid
            .viewport_rows;
        match grab {
            Some(grab) => self.slide_copy_scroll(target, y, grab)?,
            None => self.set_copy_scroll_from_mouse(target, y, height, vi)?,
        }
        let exited = scroll_exit
            && self.window(resolved.session, resolved.window).panes[resolved.pane]
                .copy
                .as_ref()
                .is_some_and(|copy| copy.scroll == 0);
        if exited {
            let pane = &mut self.window_mut(resolved.session, resolved.window).panes[resolved.pane];
            let pane_id = pane.id;
            pane.mode = None;
            pane.copy = None;
            pane.mode_view = None;
            self.invalidate_session(session_id, RenderInvalidation::MODE);
            self.notify_pane(PaneHook::ModeChanged, pane_id);
        }
        Ok(exited)
    }

    pub(crate) fn copy_mode_command_with_argument(
        &mut self,
        target: &str,
        command: &str,
        argument: Option<&str>,
        vi: bool,
        separators: &str,
    ) -> io::Result<Option<String>> {
        self.copy_mode_command_with_prefix(target, command, argument, 1, vi, separators)
    }

    pub(crate) fn copy_mode_command_with_prefix(
        &mut self,
        target: &str,
        command: &str,
        argument: Option<&str>,
        prefix: u32,
        vi: bool,
        separators: &str,
    ) -> io::Result<Option<String>> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let wrap_search = self.option_for_target(target, "wrap-search") != Some("off");
        let absolute_line_numbers = matches!(
            self.option_for_target(target, "copy-mode-line-numbers"),
            Some("absolute" | "relative" | "hybrid")
        );
        let session_id = self.sessions[resolved.session].id;
        let mut left_mode = false;
        let result = {
            let node = &mut self.window_mut(resolved.session, resolved.window).panes[resolved.pane];
            if node.copy.is_none() {
                return Err(io::Error::other("not in a mode"));
            }
            if command == "refresh-from-pane" {
                let (ScreenSnapshot { grid, vt }, _) = node.pane.copy_snapshot();
                let state = node.copy.as_mut().expect("copy mode checked above");
                state.backing = CopyBacking::PaneSnapshot;
                state.grid = grid;
                state.replace_vt(vt);
                clamp_copy_state(state, vi);
                ensure_copy_cursor_visible(state);
                if let Some(search) = state.search.as_mut() {
                    search.matches.clear();
                }
                state.search_marks = false;
                state.incremental_search_origin = Some(CopySearchOrigin {
                    cursor: state.cursor.clone(),
                    desired_col: state.desired_col,
                    scroll: state.scroll,
                });
                None
            } else {
                let mut end_mode = false;
                let output = {
                    let state = node.copy.as_mut().expect("copy mode checked above");
                    clamp_copy_state(state, vi);

                    let output = match command {
                        "history-top" => {
                            state.cursor.row = 0;
                            state.cursor.col = 0;
                            state.scroll = state.grid.scrollback_rows;
                            None
                        }
                        "history-bottom" => {
                            state.cursor.row = state.grid.rows.len().saturating_sub(1);
                            state.cursor.col = copy_cursor_limit(&state.grid, state.cursor.row, vi);
                            state.scroll = 0;
                            None
                        }
                        "page-up" => {
                            for _ in 0..prefix {
                                move_copy_page(state, true, vi);
                            }
                            None
                        }
                        "page-down" => {
                            for _ in 0..prefix {
                                move_copy_page(state, false, vi);
                                if state.scroll_exit && state.scroll == 0 {
                                    end_mode = true;
                                    break;
                                }
                            }
                            None
                        }
                        "page-down-and-cancel" => {
                            for _ in 0..prefix {
                                move_copy_page(state, false, vi);
                                if state.scroll == 0 {
                                    end_mode = true;
                                    break;
                                }
                            }
                            None
                        }
                        "halfpage-up" => {
                            for _ in 0..prefix {
                                move_copy_rows(
                                    state,
                                    true,
                                    (state.grid.viewport_rows as usize / 2).max(1),
                                    vi,
                                );
                            }
                            None
                        }
                        "halfpage-down" => {
                            for _ in 0..prefix {
                                move_copy_rows(
                                    state,
                                    false,
                                    (state.grid.viewport_rows as usize / 2).max(1),
                                    vi,
                                );
                                if state.scroll_exit && state.scroll == 0 {
                                    end_mode = true;
                                    break;
                                }
                            }
                            None
                        }
                        "halfpage-down-and-cancel" => {
                            for _ in 0..prefix {
                                move_copy_rows(
                                    state,
                                    false,
                                    (state.grid.viewport_rows as usize / 2).max(1),
                                    vi,
                                );
                                if state.scroll == 0 {
                                    end_mode = true;
                                    break;
                                }
                            }
                            None
                        }
                        "cursor-up" => {
                            for _ in 0..prefix {
                                move_copy_row(state, true, vi);
                            }
                            None
                        }
                        "cursor-down" => {
                            for _ in 0..prefix {
                                move_copy_row(state, false, vi);
                            }
                            None
                        }
                        "cursor-down-and-cancel" => {
                            let old_y = copy_cursor_view_y(state);
                            for _ in 0..prefix {
                                move_copy_row(state, false, vi);
                            }
                            if old_y == copy_cursor_view_y(state) && state.scroll == 0 {
                                end_mode = true;
                            }
                            None
                        }
                        "cursor-centre-vertical" => {
                            centre_copy_cursor_vertical(state, vi);
                            None
                        }
                        "cursor-centre-horizontal" => {
                            centre_copy_cursor_horizontal(state, vi);
                            None
                        }
                        "start-of-line" => {
                            copy_reader_start_of_line(&mut state.cursor, &state.grid);
                            ensure_copy_cursor_visible(state);
                            None
                        }
                        "cursor-left" => {
                            for _ in 0..prefix {
                                copy_reader_cursor_left(&mut state.cursor, &state.grid, false);
                            }
                            ensure_copy_cursor_visible(state);
                            None
                        }
                        "cursor-right" => {
                            // A rectangle selection frees the cursor from the
                            // text, as tmux's `all` argument does.
                            let all = state.selection.is_some() && state.rectangle;
                            for _ in 0..prefix {
                                copy_reader_cursor_right(
                                    &mut state.cursor,
                                    &state.grid,
                                    true,
                                    all,
                                    !vi,
                                );
                            }
                            ensure_copy_cursor_visible(state);
                            None
                        }
                        "back-to-indentation" => {
                            copy_reader_back_to_indentation(&mut state.cursor, &state.grid);
                            ensure_copy_cursor_visible(state);
                            None
                        }
                        "end-of-line" => {
                            copy_reader_end_of_line(&mut state.cursor, &state.grid, vi);
                            ensure_copy_cursor_visible(state);
                            None
                        }
                        "top-line" => {
                            move_copy_view_line(state, CopyViewLine::Top);
                            None
                        }
                        "middle-line" => {
                            move_copy_view_line(state, CopyViewLine::Middle);
                            None
                        }
                        "bottom-line" => {
                            move_copy_view_line(state, CopyViewLine::Bottom);
                            None
                        }
                        "goto-line" => {
                            if let Some(line) = argument {
                                goto_copy_line(state, line, absolute_line_numbers);
                            }
                            None
                        }
                        "next-paragraph" => {
                            for _ in 0..prefix {
                                move_copy_paragraph(state, false);
                            }
                            None
                        }
                        "previous-paragraph" => {
                            for _ in 0..prefix {
                                move_copy_paragraph(state, true);
                            }
                            None
                        }
                        "next-matching-bracket" => {
                            for _ in 0..prefix {
                                move_copy_matching_bracket(state, false, vi);
                            }
                            None
                        }
                        "previous-matching-bracket" => {
                            for _ in 0..prefix {
                                move_copy_matching_bracket(state, true, vi);
                            }
                            None
                        }
                        "scroll-top" => {
                            align_copy_cursor_in_view(state, 0);
                            None
                        }
                        "scroll-middle" => {
                            let target = state.grid.viewport_rows.saturating_sub(1) as usize / 2;
                            align_copy_cursor_in_view(state, target);
                            None
                        }
                        "scroll-bottom" => {
                            let target = state.grid.viewport_rows.saturating_sub(1) as usize;
                            align_copy_cursor_in_view(state, target);
                            None
                        }
                        "scroll-up" => {
                            for _ in 0..prefix {
                                scroll_copy_content(state, true, vi);
                            }
                            None
                        }
                        "scroll-down" => {
                            for _ in 0..prefix {
                                scroll_copy_content(state, false, vi);
                            }
                            if state.scroll_exit && state.scroll == 0 {
                                end_mode = true;
                            }
                            None
                        }
                        "scroll-down-and-cancel" => {
                            for _ in 0..prefix {
                                scroll_copy_content(state, false, vi);
                            }
                            if state.scroll == 0 {
                                end_mode = true;
                            }
                            None
                        }
                        "scroll-exit-on" => {
                            state.scroll_exit = true;
                            None
                        }
                        "scroll-exit-off" => {
                            state.scroll_exit = false;
                            None
                        }
                        "scroll-exit-toggle" => {
                            state.scroll_exit = !state.scroll_exit;
                            None
                        }
                        "recentre-top-bottom" => {
                            recentre_copy_cursor(state);
                            None
                        }
                        "begin-selection" => {
                            let point = (state.cursor.row, state.cursor.col);
                            state.selection = Some(CopySelection {
                                anchor: point,
                                end: point,
                                active: true,
                                drag_anchor: false,
                            });
                            None
                        }
                        "set-mark" => {
                            state.mark = Some((state.cursor.row, state.cursor.col));
                            None
                        }
                        "jump-to-mark" => {
                            if let Some((row, col)) = state.mark {
                                position_copy_cursor(state, row, col);
                            }
                            None
                        }
                        "jump-forward" | "jump-backward" | "jump-to-forward"
                        | "jump-to-backward" => {
                            if let Some(text) = argument.filter(|text| !text.is_empty()) {
                                let kind = match command {
                                    "jump-forward" => CopyJumpKind::Forward,
                                    "jump-backward" => CopyJumpKind::Backward,
                                    "jump-to-forward" => CopyJumpKind::ToForward,
                                    _ => CopyJumpKind::ToBackward,
                                };
                                state.jump = Some(CopyJump {
                                    text: text.to_string(),
                                    kind,
                                });
                                for _ in 0..prefix {
                                    repeat_copy_jump(state, false);
                                }
                            }
                            None
                        }
                        "jump-again" | "jump-reverse" => {
                            for _ in 0..prefix {
                                repeat_copy_jump(state, command == "jump-reverse");
                            }
                            None
                        }
                        "next-prompt" => {
                            move_copy_prompt(state, true, argument == Some("-o"));
                            None
                        }
                        "previous-prompt" => {
                            move_copy_prompt(state, false, argument == Some("-o"));
                            None
                        }
                        "stop-selection" => {
                            if let Some(selection) = state.selection.as_mut() {
                                selection.active = false;
                            }
                            None
                        }
                        "clear-selection" => {
                            state.selection = None;
                            None
                        }
                        "other-end" => {
                            if !prefix.is_multiple_of(2) {
                                if let Some(selection) = state.selection.as_mut() {
                                    // tmux flips which end the cursor drags and
                                    // leaves both where they are, so the
                                    // geometry formats keep reporting the same
                                    // pair.
                                    selection.drag_anchor = !selection.drag_anchor;
                                    let target = if selection.drag_anchor {
                                        selection.anchor
                                    } else {
                                        selection.end
                                    };
                                    state.cursor.row = target.0;
                                    state.cursor.col = target.1;
                                    selection.active = true;
                                    ensure_copy_cursor_visible(state);
                                }
                            }
                            None
                        }
                        "rectangle-on" => {
                            state.rectangle = true;
                            None
                        }
                        "rectangle-off" => {
                            state.rectangle = false;
                            None
                        }
                        "rectangle-toggle" => {
                            state.rectangle = !state.rectangle;
                            None
                        }
                        "select-line" => {
                            state.selection_mode = CopySelectionMode::Line;
                            select_copy_line(state, vi);
                            for _ in 1..prefix {
                                move_copy_row(state, false, vi);
                                copy_reader_end_of_line(&mut state.cursor, &state.grid, vi);
                            }
                            None
                        }
                        "select-word" => {
                            state.selection_mode = CopySelectionMode::Word;
                            select_copy_word(state, vi, separators);
                            None
                        }
                        "selection-mode" => {
                            state.selection_mode =
                                match argument.unwrap_or("char").to_ascii_lowercase().as_str() {
                                    "word" | "w" => CopySelectionMode::Word,
                                    "line" | "l" => CopySelectionMode::Line,
                                    _ => CopySelectionMode::Character,
                                };
                            None
                        }
                        "toggle-position" => {
                            state.hide_position = !state.hide_position;
                            None
                        }
                        "search-forward-text" => {
                            if let Some(pattern) = argument.filter(|pattern| !pattern.is_empty()) {
                                start_copy_search(
                                    state,
                                    pattern,
                                    CopySearchDirection::Forward,
                                    false,
                                    vi,
                                    wrap_search,
                                );
                                for _ in 1..prefix {
                                    repeat_copy_search(state, false, vi, wrap_search);
                                }
                            }
                            None
                        }
                        "search-backward-text" => {
                            if let Some(pattern) = argument.filter(|pattern| !pattern.is_empty()) {
                                start_copy_search(
                                    state,
                                    pattern,
                                    CopySearchDirection::Backward,
                                    false,
                                    vi,
                                    wrap_search,
                                );
                                for _ in 1..prefix {
                                    repeat_copy_search(state, false, vi, wrap_search);
                                }
                            }
                            None
                        }
                        "search-forward" => {
                            if let Some(pattern) = argument.filter(|pattern| !pattern.is_empty()) {
                                start_copy_search(
                                    state,
                                    pattern,
                                    CopySearchDirection::Forward,
                                    true,
                                    vi,
                                    wrap_search,
                                );
                                for _ in 1..prefix {
                                    repeat_copy_search(state, false, vi, wrap_search);
                                }
                            }
                            None
                        }
                        "search-backward" => {
                            if let Some(pattern) = argument.filter(|pattern| !pattern.is_empty()) {
                                start_copy_search(
                                    state,
                                    pattern,
                                    CopySearchDirection::Backward,
                                    true,
                                    vi,
                                    wrap_search,
                                );
                                for _ in 1..prefix {
                                    repeat_copy_search(state, false, vi, wrap_search);
                                }
                            }
                            None
                        }
                        "search-forward-incremental" => {
                            if let Some(argument) = argument {
                                incremental_copy_search(state, argument, true, vi, wrap_search);
                            }
                            None
                        }
                        "search-backward-incremental" => {
                            if let Some(argument) = argument {
                                incremental_copy_search(state, argument, false, vi, wrap_search);
                            }
                            None
                        }
                        "search-again" => {
                            for _ in 0..prefix {
                                repeat_copy_search(state, false, vi, wrap_search);
                            }
                            None
                        }
                        "search-reverse" => {
                            for _ in 0..prefix {
                                repeat_copy_search(state, true, vi, wrap_search);
                            }
                            None
                        }
                        "previous-word" => {
                            for _ in 0..prefix {
                                move_previous(&mut state.cursor, &state.grid, vi, separators, true);
                            }
                            ensure_copy_cursor_visible(state);
                            None
                        }
                        "previous-space" => {
                            for _ in 0..prefix {
                                move_previous(&mut state.cursor, &state.grid, vi, "", true);
                            }
                            ensure_copy_cursor_visible(state);
                            None
                        }
                        "next-word" => {
                            for _ in 0..prefix {
                                move_next_start(&mut state.cursor, &state.grid, vi, separators);
                            }
                            ensure_copy_cursor_visible(state);
                            None
                        }
                        "next-space" => {
                            for _ in 0..prefix {
                                move_next_start(&mut state.cursor, &state.grid, vi, "");
                            }
                            ensure_copy_cursor_visible(state);
                            None
                        }
                        "next-word-end" => {
                            for _ in 0..prefix {
                                move_next_end(&mut state.cursor, &state.grid, vi, separators);
                            }
                            ensure_copy_cursor_visible(state);
                            None
                        }
                        "next-space-end" => {
                            for _ in 0..prefix {
                                move_next_end(&mut state.cursor, &state.grid, vi, "");
                            }
                            ensure_copy_cursor_visible(state);
                            None
                        }
                        "copy-selection"
                        | "copy-selection-no-clear"
                        | "copy-selection-and-cancel"
                        | "copy-pipe"
                        | "copy-pipe-no-clear"
                        | "copy-pipe-and-cancel"
                        | "pipe"
                        | "pipe-no-clear"
                        | "pipe-and-cancel"
                        | "append-selection"
                        | "append-selection-and-cancel" => {
                            // With nothing selected tmux copies the marked
                            // search match under the cursor, and copies nothing
                            // at all when there is none.
                            let data = if state.selection.is_some() {
                                Some(copy_selection(state, vi))
                            } else {
                                copy_match_at_cursor(state)
                            };
                            if !command.ends_with("no-clear") {
                                clear_copy_selection(state);
                            }
                            end_mode = command.ends_with("and-cancel");
                            data
                        }
                        "copy-end-of-line"
                        | "copy-end-of-line-and-cancel"
                        | "copy-pipe-end-of-line"
                        | "copy-pipe-end-of-line-and-cancel" => {
                            let data = copy_from_cursor_to_line_end(state, vi);
                            clear_copy_selection(state);
                            end_mode = command.ends_with("and-cancel");
                            Some(data)
                        }
                        "copy-line"
                        | "copy-line-and-cancel"
                        | "copy-pipe-line"
                        | "copy-pipe-line-and-cancel" => {
                            let data = copy_current_line(state, vi);
                            clear_copy_selection(state);
                            end_mode = command.ends_with("and-cancel");
                            Some(data)
                        }
                        "cancel" => {
                            end_mode = true;
                            None
                        }
                        _ => None,
                    };
                    if matches!(
                        command,
                        "history-top"
                            | "history-bottom"
                            | "start-of-line"
                            | "cursor-left"
                            | "cursor-right"
                            | "cursor-centre-vertical"
                            | "cursor-centre-horizontal"
                            | "back-to-indentation"
                            | "end-of-line"
                            | "top-line"
                            | "middle-line"
                            | "bottom-line"
                            | "next-paragraph"
                            | "previous-paragraph"
                            | "next-matching-bracket"
                            | "previous-matching-bracket"
                            | "other-end"
                            | "select-line"
                            | "select-word"
                            | "previous-word"
                            | "previous-space"
                            | "next-word"
                            | "next-space"
                            | "next-word-end"
                            | "next-space-end"
                    ) {
                        state.desired_col = state.cursor.col;
                    }
                    synchronize_copy_selection(state);
                    clear_copy_search_marks_after_command(state, command, vi);
                    output
                };
                if matches!(
                    command,
                    "search-forward"
                        | "search-backward"
                        | "search-forward-text"
                        | "search-backward-text"
                ) {
                    if let Some(pattern) = argument.filter(|pattern| !pattern.is_empty()) {
                        node.search_string = Some(pattern.to_string());
                        node.search_regex = matches!(command, "search-forward" | "search-backward")
                            && search_uses_posix_regex(pattern);
                    }
                } else if matches!(
                    command,
                    "search-forward-incremental" | "search-backward-incremental"
                ) {
                    if let Some((prefix, pattern)) = argument.and_then(incremental_search_argument)
                    {
                        if !pattern.is_empty() && matches!(prefix, '=' | '+' | '-') {
                            node.search_string = Some(pattern.to_string());
                            node.search_regex = false;
                        }
                    }
                }
                if end_mode {
                    node.mode = None;
                    node.copy = None;
                    node.mode_view = None;
                    left_mode = true;
                }
                output
            }
        };
        self.invalidate_session(session_id, RenderInvalidation::MODE);
        if left_mode {
            // tmux's `window_pane_reset_mode`: the pane reports the leave
            // however the command that ended the mode reached it.
            let pane_id = self.window(resolved.session, resolved.window).panes[resolved.pane].id;
            self.notify_pane(PaneHook::ModeChanged, pane_id);
        }
        // A copy that produced data writes the terminal selection unless
        // `set-clipboard` is `off`, and tmux notifies the pane whenever it
        // does — including under `external`, where the write is the client's.
        if let Some(copied) = result.as_ref().filter(|_| {
            self.option_for_target(target, "set-clipboard")
                .is_none_or(|value| value != "off")
        }) {
            let window = self.window(resolved.session, resolved.window);
            let (window_id, pane_id) = (window.id, window.panes[resolved.pane].id);
            // tmux's `screen_write_setselection` with an empty selection name,
            // written through the same `Ms` capability and to the same clients
            // as any other pane output: those with the window on screen.
            self.write_window_selection(window_id, copied.clone().into_bytes());
            self.notify_pane(PaneHook::SetClipboard, pane_id);
        }
        Ok(result)
    }

    pub(crate) fn active_copy_state(&self, session_name: &str) -> Option<&CopyState> {
        let (window, active) = self.active_window_panes(session_name).ok()?;
        window.panes.get(active)?.copy.as_ref()
    }

    pub(crate) fn active_mode_view(&self, session_name: &str) -> Option<&ModeView> {
        let (window, active) = self.active_window_panes(session_name).ok()?;
        window.panes.get(active)?.mode_view.as_ref()
    }
}

/// What one of a mode's own action keys does: the command it runs, the rows it
/// destroyed, whether the mode is over, and the question asked first.
struct ModeAction {
    command: Vec<String>,
    removed: Vec<usize>,
    exit: bool,
    confirm: Option<String>,
}

impl ModeAction {
    fn command(command: Vec<String>) -> Self {
        Self {
            command,
            removed: Vec::new(),
            exit: false,
            confirm: None,
        }
    }

    fn nothing() -> Self {
        Self::command(Vec::new())
    }

    fn exiting(mut self) -> Self {
        self.exit = true;
        self
    }

    fn removing(mut self, removed: Vec<usize>) -> Self {
        self.removed = removed;
        self
    }

    fn confirmed(mut self, prompt: String) -> Self {
        self.confirm = Some(prompt);
        self
    }
}

/// Chain several commands into the one argv the queue compiles, the way a
/// mode-tree row's own template already carries a `;` between two commands.
fn chain(commands: Vec<Vec<String>>) -> Vec<String> {
    let mut argv = Vec::new();
    for command in commands {
        if command.is_empty() {
            continue;
        }
        if !argv.is_empty() {
            argv.push(";".to_owned());
        }
        argv.extend(command);
    }
    argv
}

/// The `-t` a row's target spells.
fn mode_target_argument(target: &ModeTarget) -> String {
    match target {
        ModeTarget::Session { name } => format!("={name}:"),
        ModeTarget::Window { session, index } => format!("={session}:{index}"),
        ModeTarget::Client { name } | ModeTarget::Buffer { name } => name.clone(),
    }
}

/// The `-t` a row's target spells for the commands that take a session or a
/// window rather than a pane.
fn mode_kill_argument(target: &ModeTarget) -> String {
    match target {
        ModeTarget::Session { name } => name.clone(),
        ModeTarget::Window { session, index } => format!("{session}:{index}"),
        _ => mode_target_argument(target),
    }
}

/// The kill this row's `x` runs, and the question it asks first.
fn mode_kill_command(target: &ModeTarget) -> Option<(String, Vec<String>)> {
    let argument = mode_kill_argument(target);
    match target {
        ModeTarget::Session { name } => Some((
            format!("Kill session {name}? "),
            vec!["kill-session".to_owned(), "-t".to_owned(), argument],
        )),
        ModeTarget::Window { index, .. } => Some((
            format!("Kill window {index}? "),
            vec!["kill-window".to_owned(), "-t".to_owned(), argument],
        )),
        ModeTarget::Client { .. } | ModeTarget::Buffer { .. } => None,
    }
}

/// The rows `x`, `d` and their tagged forms act on: every tagged row, or —
/// where tmux passes `mode_tree_each_tagged` a current of its own — the
/// selected one when nothing is tagged.
fn action_rows(view: &ModeView, tagged_only: bool) -> Vec<usize> {
    let tagged: Vec<usize> = view
        .items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| item.tagged.then_some(index))
        .collect();
    if !tagged.is_empty() || tagged_only {
        return tagged;
    }
    vec![view.selected]
}

/// The per-mode action keys tmux keeps in `window_tree_key`,
/// `window_client_key` and `window_buffer_key`, which its mode-tree dispatches
/// to after its own common keys.
fn mode_view_action(view: &mut ModeView, key: &str) -> Option<ModeAction> {
    match (&view.kind, key) {
        // Mode-tree's own reordering and sorting, which the tree mode backs
        // with a window swap.
        (_, "r") => {
            view.reverse_sort();
            Some(ModeAction::nothing())
        }
        (ModeKind::Tree, "K" | "J" | "S-Up" | "S-Down") => {
            let forward = matches!(key, "J" | "S-Down");
            let other = if forward {
                view.selected.checked_add(1)
            } else {
                view.selected.checked_sub(1)
            }
            .filter(|index| *index < view.items.len())?;
            let (
                Some(ModeTarget::Window {
                    session,
                    index: current,
                }),
                Some(ModeTarget::Window {
                    session: other_session,
                    index: swap_with,
                }),
            ) = (
                view.items.get(view.selected).and_then(|item| item.target.clone()),
                view.items.get(other).and_then(|item| item.target.clone()),
            )
            else {
                return Some(ModeAction::nothing());
            };
            if session != other_session {
                return Some(ModeAction::nothing());
            }
            view.swap(view.selected, other);
            Some(ModeAction::command(vec![
                "swap-window".to_owned(),
                "-d".to_owned(),
                "-s".to_owned(),
                format!("={session}:{current}"),
                "-t".to_owned(),
                format!("={other_session}:{swap_with}"),
            ]))
        }
        (ModeKind::Tree, "m") => {
            let target = view
                .items
                .get(view.selected)
                .and_then(|item| item.target.as_ref())
                .map(mode_target_argument)?;
            Some(ModeAction::command(vec![
                "select-pane".to_owned(),
                "-m".to_owned(),
                "-t".to_owned(),
                target,
            ]))
        }
        (ModeKind::Tree, "M") => Some(ModeAction::command(vec![
            "select-pane".to_owned(),
            "-M".to_owned(),
        ])),
        (ModeKind::Tree, "x" | "X") => {
            let tagged_only = key == "X";
            let rows = action_rows(view, tagged_only);
            let kills: Vec<(String, Vec<String>)> = rows
                .iter()
                .filter_map(|index| view.items.get(*index))
                .filter_map(|item| item.target.as_ref())
                .filter_map(mode_kill_command)
                .collect();
            if kills.is_empty() {
                return Some(ModeAction::nothing());
            }
            let prompt = if tagged_only || rows.len() > 1 {
                format!("Kill {} tagged? ", kills.len())
            } else {
                kills[0].0.clone()
            };
            let commands = kills.into_iter().map(|(_, command)| command).collect();
            Some(ModeAction::command(chain(commands)).confirmed(prompt))
        }
        (ModeKind::Client, "d" | "x" | "D" | "X") => {
            let tagged_only = matches!(key, "D" | "X");
            let hangup = matches!(key, "x" | "X");
            let rows = action_rows(view, tagged_only);
            let commands: Vec<Vec<String>> = rows
                .iter()
                .filter_map(|index| view.items.get(*index))
                .filter_map(|item| match item.target.as_ref() {
                    Some(ModeTarget::Client { name }) => {
                        let mut command = vec!["detach-client".to_owned()];
                        if hangup {
                            command.push("-P".to_owned());
                        }
                        command.push("-t".to_owned());
                        command.push(name.clone());
                        Some(command)
                    }
                    _ => None,
                })
                .collect();
            if commands.is_empty() {
                return Some(ModeAction::nothing());
            }
            Some(ModeAction::command(chain(commands)).removing(rows))
        }
        (ModeKind::Client, "z" | "Z") => {
            let rows = action_rows(view, key == "Z");
            let commands: Vec<Vec<String>> = rows
                .iter()
                .filter_map(|index| view.items.get(*index))
                .filter_map(|item| match item.target.as_ref() {
                    Some(ModeTarget::Client { name }) => Some(vec![
                        "suspend-client".to_owned(),
                        "-t".to_owned(),
                        name.clone(),
                    ]),
                    _ => None,
                })
                .collect();
            if commands.is_empty() {
                return Some(ModeAction::nothing());
            }
            Some(ModeAction::command(chain(commands)).removing(rows))
        }
        (ModeKind::Buffer, "d" | "D") => {
            let rows = action_rows(view, key == "D");
            let commands: Vec<Vec<String>> = rows
                .iter()
                .filter_map(|index| view.items.get(*index))
                .filter_map(|item| match item.target.as_ref() {
                    Some(ModeTarget::Buffer { name }) => Some(vec![
                        "delete-buffer".to_owned(),
                        "-b".to_owned(),
                        name.clone(),
                    ]),
                    _ => None,
                })
                .collect();
            if commands.is_empty() {
                return Some(ModeAction::nothing());
            }
            Some(ModeAction::command(chain(commands)).removing(rows))
        }
        (ModeKind::Buffer, "p" | "P") => {
            let rows = action_rows(view, key == "P");
            let commands: Vec<Vec<String>> = rows
                .iter()
                .filter_map(|index| view.items.get(*index))
                .map(|item| item.command.clone())
                .collect();
            Some(ModeAction::command(chain(commands)).exiting())
        }
        _ => None,
    }
}
