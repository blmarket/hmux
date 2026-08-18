//! Mode-tree views — the rows, edits, and prompts behind `choose-tree`,
//! `choose-client`, `choose-buffer`, `customize-mode`, and `clock-mode`.
//!
//! A [`ModeView`] is the server-side mirror of tmux's `mode_tree_data`: the
//! full item list, the filtered view drawn on screen, and where the selection
//! sits within it.

use super::PopupRequest;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ModeKind {
    Tree,
    Client,
    Buffer,
    Customize,
    Clock,
}

/// What a mode-tree row names, for the per-mode action keys that act on the
/// row itself rather than running its template — tmux's `window_tree_itemdata`,
/// `window_client_itemdata` and `window_buffer_itemdata`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ModeTarget {
    Session { name: String },
    Window { session: String, index: u32 },
    Client { name: String },
    Buffer { name: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ModeItem {
    pub(crate) label: String,
    pub(crate) command: Vec<String>,
    pub(crate) prompt_target: Option<String>,
    pub(crate) edit: Option<ModeEdit>,
    /// tmux's `mode_tree_item.tagged`, toggled by `t` and drawn as a `*`.
    pub(crate) tagged: bool,
    /// The pane the preview shows while this row is selected, when the mode
    /// has one to show.
    pub(crate) preview_target: Option<String>,
    /// How deep this row sits in the tree — tmux's `mode_tree_item.depth`.
    pub(crate) depth: u16,
    /// Whether this row has children, and whether they are currently shown.
    /// `None` for a leaf.
    pub(crate) expanded: Option<bool>,
    /// What the row names, where an action key needs the thing rather than the
    /// template the row would run.
    pub(crate) target: Option<ModeTarget>,
}

/// One option row in customize mode, with the scope text its table prints.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CustomizeOption {
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) scope: String,
    pub(crate) is_array: bool,
    pub(crate) array_has_entries: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ModeEdit {
    Option {
        name: String,
        value: String,
    },
    BindingCommand {
        table: String,
        key: String,
        value: String,
        note: Option<String>,
        repeat: bool,
    },
    BindingNote {
        table: String,
        key: String,
        value: String,
        command: Vec<String>,
        repeat: bool,
    },
}

pub(crate) struct ModeBindingUpdate {
    pub(crate) table: String,
    pub(crate) key: String,
    pub(crate) command_text: String,
    pub(crate) command: Vec<String>,
    pub(crate) note: Option<String>,
    pub(crate) repeat: bool,
}

impl ModeEdit {
    pub(crate) fn prompt(&self) -> String {
        match self {
            Self::Option { name, .. } => format!("Set option {name}:"),
            Self::BindingCommand { table, key, .. } => {
                format!("Set command for {table} {key}:")
            }
            Self::BindingNote { table, key, .. } => format!("Set note for {table} {key}:"),
        }
    }

    pub(crate) fn initial(&self) -> &str {
        match self {
            Self::Option { value, .. }
            | Self::BindingCommand { value, .. }
            | Self::BindingNote { value, .. } => value,
        }
    }
}

pub(super) fn update_mode_edit_item(item: &mut ModeItem, edited: &ModeEdit, value: &str) {
    match (item.edit.as_mut(), edited) {
        (
            Some(ModeEdit::Option {
                name,
                value: current,
            }),
            ModeEdit::Option {
                name: edited_name, ..
            },
        ) if name == edited_name => {
            *current = value.to_string();
            item.label = format!("{name} {value}");
        }
        (
            Some(ModeEdit::BindingCommand {
                table,
                key,
                value: current,
                ..
            }),
            ModeEdit::BindingCommand {
                table: edited_table,
                key: edited_key,
                ..
            },
        ) if table == edited_table && key == edited_key => {
            *current = value.to_string();
            item.label = format!("key {table} {key} command {value}");
        }
        (
            Some(ModeEdit::BindingNote {
                table,
                key,
                value: current,
                ..
            }),
            ModeEdit::BindingNote {
                table: edited_table,
                key: edited_key,
                ..
            },
        ) if table == edited_table && key == edited_key => {
            *current = value.to_string();
            item.label = format!("key {table} {key} note {value}");
        }
        _ => {}
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ModePrompt {
    Search,
    Filter { initial: String },
    Command { item_target: String },
    Edit(ModeEdit),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ModeViewKeyResult {
    None,
    Command(Vec<String>),
    Prompt(ModePrompt),
    /// An overlay the mode itself asks for — buffer mode's editor.
    Popup(Box<PopupRequest>),
    /// A single-key confirmation the mode asks for before its command runs —
    /// mode-tree's `x` and `X`.
    Confirm {
        prompt: String,
        command: Vec<String>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ModeView {
    pub(crate) kind: ModeKind,
    /// tmux's `-N`, inverted: whether the bottom of the mode shows a preview of
    /// whatever the selected row names.
    pub(crate) preview: bool,
    pub(crate) title: String,
    pub(crate) items: Vec<ModeItem>,
    pub(crate) all_items: Vec<ModeItem>,
    pub(crate) filter: String,
    pub(crate) selected: usize,
    pub(crate) scroll: usize,
}

impl ModeView {
    /// The rows actually on screen: a row whose parent is collapsed is not.
    /// tmux keeps the whole tree and draws the part `mode_tree_build` walked.
    pub(crate) fn visible(&self) -> Vec<&ModeItem> {
        let mut visible = Vec::with_capacity(self.items.len());
        let mut hidden_below: Option<u16> = None;
        for item in &self.items {
            if hidden_below.is_some_and(|depth| item.depth > depth) {
                continue;
            }
            hidden_below = None;
            visible.push(item);
            if item.expanded == Some(false) {
                hidden_below = Some(item.depth);
            }
        }
        visible
    }

    /// Drop a row the action key just destroyed, keeping the selection on the
    /// row below it as tmux's per-item delete does before it rebuilds.
    pub(crate) fn remove(&mut self, index: usize) {
        if index >= self.items.len() {
            return;
        }
        let removed = self.items.remove(index);
        self.all_items.retain(|item| item != &removed);
        let last = self.items.len().saturating_sub(1);
        self.selected = self.selected.min(last);
        self.scroll = self.scroll.min(self.selected);
    }

    /// Reverse the active sort, as `r` does: the top-level rows swap order and
    /// each one keeps its own children, themselves reversed.
    pub(crate) fn reverse_sort(&mut self) {
        self.items = reverse_rows(std::mem::take(&mut self.items));
        self.all_items = reverse_rows(std::mem::take(&mut self.all_items));
    }

    /// Swap two rows and follow the selection to where the row moved, as
    /// tmux's `mode_tree_swap` does once the mode has swapped the things
    /// themselves.
    pub(crate) fn swap(&mut self, left: usize, right: usize) {
        self.items.swap(left, right);
        self.selected = right;
    }

    /// Set every row's expansion, as `M-+` and `M--` do.
    pub(crate) fn expand_all(&mut self, expanded: bool) {
        for item in &mut self.items {
            if item.expanded.is_some() {
                item.expanded = Some(expanded);
            }
        }
    }

    pub(crate) fn list(kind: ModeKind, title: impl Into<String>, items: Vec<ModeItem>) -> Self {
        Self {
            kind,
            preview: false,
            title: title.into(),
            all_items: items.clone(),
            items,
            filter: String::new(),
            selected: 0,
            scroll: 0,
        }
    }

    pub(crate) fn clock() -> Self {
        Self::list(ModeKind::Clock, "Clock", Vec::new())
    }
}

/// One level of a mode tree in reverse, each row still carrying the rows that
/// sit under it.
fn reverse_rows(items: Vec<ModeItem>) -> Vec<ModeItem> {
    let Some(top) = items.iter().map(|item| item.depth).min() else {
        return items;
    };
    let mut groups: Vec<Vec<ModeItem>> = Vec::new();
    for item in items {
        if item.depth == top || groups.is_empty() {
            groups.push(vec![item]);
        } else {
            groups.last_mut().expect("group for a child row").push(item);
        }
    }
    groups.reverse();
    groups
        .into_iter()
        .flat_map(|group| {
            let mut group = group.into_iter();
            let parent = group.next().expect("group with its own row");
            std::iter::once(parent).chain(reverse_rows(group.collect()))
        })
        .collect()
}
