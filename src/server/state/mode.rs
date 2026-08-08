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
