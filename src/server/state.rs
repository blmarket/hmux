//! The server's session/window/pane model.
//!
//! A deliberately small mirror of tmux's `session`/`winlink`/`window`/
//! `window_pane` tree — enough to back the commands the prototype implements
//! (`list-sessions`, `new-session`, `has-session`, `kill-session`) with real
//! state. Panes hold a libghostty-backed [`Pane`], so a created session is a
//! genuinely running terminal, not a stub.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io;
use std::os::fd::{AsFd, AsRawFd, RawFd};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use super::key::{parse_key_name, KeyCode};
use super::options::{GlobalOptions, OptionSet, OptionsView};
use super::pane::{
    NativePaneObservation, Pane, PaneClipboardEvent, PaneIo, PaneIoMode, PaneSpawnSpec,
};
use super::term::ResolvedTerm;
use crate::platform::{CurrentPlatform, OutputWakeup, Platform};

#[cfg(not(test))]
fn default_pane_io_mode() -> PaneIoMode {
    PaneIoMode::EventLoop
}

#[cfg(test)]
fn default_pane_io_mode() -> PaneIoMode {
    PaneIoMode::Threaded(super::pane::spawn_reader)
}

/// How to back a new pane's screen.
pub enum PaneSpec {
    /// A screen with no child process (deterministic; used for the default
    /// session and tests).
    Inert,
    /// Spawn this argv on a pty and drain it into the screen.
    Command(Vec<String>),
    /// Spawn this argv with an explicit working directory.
    CommandIn(Vec<String>, std::path::PathBuf),
}

fn pane_start_command(spec: &PaneSpec) -> String {
    let argv = match spec {
        PaneSpec::Inert => return String::new(),
        PaneSpec::Command(argv) | PaneSpec::CommandIn(argv, _) => argv,
    };
    if let Some(command) = argv
        .iter()
        .rposition(|argument| argument == "-c")
        .and_then(|index| argv.get(index + 1))
    {
        return format!("{command:?}");
    }
    argv.iter()
        .map(|argument| {
            if argument.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b'/')
            }) {
                argument.clone()
            } else {
                format!("{argument:?}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// One pane in the tree.
pub struct PaneNode {
    pub id: u32,
    pub pane: Pane,
    /// Command recorded when this pane was created, for
    /// `#{pane_start_command}`. This is observational metadata; the running
    /// child is owned by `pane`.
    pub(crate) start_command: String,
    /// `select-pane -d` blocks input until `select-pane -e` reenables it.
    pub(crate) input_off: bool,
    /// Title set with `select-pane -T`, which overrides whatever the pane's
    /// terminal reported.
    pub(crate) title: Option<String>,
    /// Whether this pane's child exit has already been announced, so the
    /// `pane-exited`/`pane-died` notification is raised exactly once.
    pub(crate) exit_notified: bool,
    /// Active pane mode name (`copy-mode`/`view-mode`), if any.
    pub(crate) mode: Option<String>,
    pub(crate) copy: Option<CopyState>,
    /// Generic tmux window-mode state (tree/client/buffer/customize/clock).
    /// Copy mode keeps its richer engine-specific state in `copy` above.
    pub(crate) mode_view: Option<ModeView>,
    pub(crate) search_string: Option<String>,
    pub(crate) search_regex: bool,
    /// Geometry outside the tiled layout tree for a `new-pane` floating pane.
    pub(crate) floating: Option<PaneRect>,
    options: OptionSet,
}

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

fn update_mode_edit_item(item: &mut ModeItem, edited: &ModeEdit, value: &str) {
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ModeView {
    pub(crate) kind: ModeKind,
    pub(crate) title: String,
    pub(crate) items: Vec<ModeItem>,
    pub(crate) all_items: Vec<ModeItem>,
    pub(crate) filter: String,
    pub(crate) search: String,
    pub(crate) selected: usize,
    pub(crate) scroll: usize,
}

impl ModeView {
    pub(crate) fn list(kind: ModeKind, title: impl Into<String>, items: Vec<ModeItem>) -> Self {
        Self {
            kind,
            title: title.into(),
            all_items: items.clone(),
            items,
            filter: String::new(),
            search: String::new(),
            selected: 0,
            scroll: 0,
        }
    }

    pub(crate) fn clock() -> Self {
        Self::list(ModeKind::Clock, "Clock", Vec::new())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MenuItem {
    pub(crate) label: String,
    pub(crate) key: String,
    pub(crate) command: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MenuRequest {
    pub(crate) title: String,
    pub(crate) items: Vec<MenuItem>,
    pub(crate) selected: usize,
    pub(crate) x: Option<String>,
    pub(crate) y: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PopupRequest {
    pub(crate) title: String,
    pub(crate) argv: Vec<String>,
    pub(crate) cwd: Option<PathBuf>,
    pub(crate) width: Option<String>,
    pub(crate) height: Option<String>,
    pub(crate) x: Option<String>,
    pub(crate) y: Option<String>,
    pub(crate) close_on_exit: bool,
    pub(crate) close_on_success: bool,
    pub(crate) close_on_key: bool,
    pub(crate) border: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum OverlayRequest {
    Menu(MenuRequest),
    Popup(PopupRequest),
    DisplayPanes {
        duration_ms: u64,
        command: Vec<String>,
        accept_input: bool,
    },
    Clear,
}

#[derive(Clone, Debug)]
pub(crate) struct MessageLogEntry {
    pub(crate) time: i64,
    pub(crate) text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BackgroundJob {
    pub(crate) id: u64,
    pub(crate) command: String,
    pub(crate) fd: RawFd,
    pub(crate) pid: u32,
}

#[derive(Default)]
struct BackgroundJobRegistryState {
    next_id: u64,
    jobs: BTreeMap<u64, BackgroundJob>,
}

#[derive(Default)]
pub(crate) struct BackgroundJobRegistry {
    inner: Mutex<BackgroundJobRegistryState>,
}

impl BackgroundJobRegistry {
    pub(crate) fn register(&self, command: String, fd: RawFd, pid: u32) -> u64 {
        let Ok(mut inner) = self.inner.lock() else {
            return u64::MAX;
        };
        let id = inner.next_id;
        inner.next_id = inner.next_id.wrapping_add(1);
        inner.jobs.insert(
            id,
            BackgroundJob {
                id,
                command,
                fd,
                pid,
            },
        );
        id
    }

    pub(crate) fn remove(&self, id: u64) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.jobs.remove(&id);
        }
    }

    pub(crate) fn jobs(&self) -> Vec<BackgroundJob> {
        self.inner
            .lock()
            .map(|inner| inner.jobs.values().cloned().collect())
            .unwrap_or_default()
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ClientAction {
    Lock(String),
    Suspend,
    Detach,
    Switch {
        session_id: u32,
        /// Set when the move is the fallout of the client's old session being
        /// destroyed, which a control client reports differently from an
        /// ordinary `switch-client`.
        destroyed: bool,
    },
    Keys(Vec<ClientKey>),
    /// Write the client's terminal selection, or query it with `None`.
    SetSelection(Option<Vec<u8>>),
    Overlay {
        request: OverlayRequest,
        reply: Option<mpsc::Sender<PromptCompletion>>,
    },
    Confirm {
        prompt: String,
        command: Vec<String>,
        confirm_key: u8,
        default_yes: bool,
        reply: Option<mpsc::Sender<PromptCompletion>>,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct ClientKey {
    pub(crate) bytes: Vec<u8>,
    pub(crate) forward_unbound: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ClientActionResult {
    Queued,
    NoCurrentClient,
    TargetNotFound,
}

#[derive(Clone, Debug)]
pub(crate) struct ClientMessage {
    pub(crate) text: String,
    pub(crate) duration_ms: u64,
    /// Write a `BEL` to the client's terminal before showing `text`. An alert
    /// under `visual-* off` is a bell with no message at all.
    pub(crate) bell: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ClientMessageResult {
    CurrentControl,
    Queued,
    NoClient,
    TargetNotFound,
}

#[derive(Default)]
struct WaitChannel {
    locked: bool,
    woken: bool,
    waiters: usize,
}

#[derive(Default)]
pub(crate) struct WaitRegistry {
    channels: Mutex<BTreeMap<String, WaitChannel>>,
    changed: Condvar,
}

impl WaitRegistry {
    pub(crate) fn signal(&self, name: &str) {
        let Ok(mut channels) = self.channels.lock() else {
            return;
        };
        channels.entry(name.to_string()).or_default().woken = true;
        self.changed.notify_all();
    }

    pub(crate) fn wait(&self, name: &str) {
        let Ok(mut channels) = self.channels.lock() else {
            return;
        };
        let channel = channels.entry(name.to_string()).or_default();
        if channel.woken {
            if !channel.locked {
                channels.remove(name);
            } else {
                channel.woken = false;
            }
            return;
        }
        channel.waiters += 1;
        loop {
            channels = match self.changed.wait(channels) {
                Ok(channels) => channels,
                Err(_) => return,
            };
            let Some(channel) = channels.get_mut(name) else {
                return;
            };
            if channel.woken {
                channel.waiters = channel.waiters.saturating_sub(1);
                if channel.waiters == 0 {
                    if channel.locked {
                        channel.woken = false;
                    } else {
                        channels.remove(name);
                    }
                }
                return;
            }
        }
    }

    pub(crate) fn lock(&self, name: &str) {
        let Ok(mut channels) = self.channels.lock() else {
            return;
        };
        loop {
            let channel = channels.entry(name.to_string()).or_default();
            if !channel.locked {
                channel.locked = true;
                return;
            }
            channels = match self.changed.wait(channels) {
                Ok(channels) => channels,
                Err(_) => return,
            };
        }
    }

    pub(crate) fn unlock(&self, name: &str) -> bool {
        let Ok(mut channels) = self.channels.lock() else {
            return false;
        };
        let Some(channel) = channels.get_mut(name) else {
            return false;
        };
        if !channel.locked {
            return false;
        }
        channel.locked = false;
        if channel.woken && channel.waiters == 0 {
            channels.remove(name);
        }
        self.changed.notify_all();
        true
    }
}

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
    pub(crate) search_count: Option<usize>,
    /// Cursor and viewport saved when an incremental search begins.
    pub(crate) incremental_search_origin: Option<CopySearchOrigin>,
    /// Numeric prefix retained on the mode entry until its next command.
    pub(crate) prefix: u32,
    pub(crate) scroll_exit: bool,
    pub(crate) recentre: CopyRecentre,
    pub(crate) grid: ghostty_sys::GridSnapshot,
    /// Frozen styled VT serialization captured with `grid`.
    pub(crate) vt: Vec<u8>,
    /// Byte ranges for the content of each row in `vt`, excluding CR and the
    /// trailing cursor-position sequence. Copy-mode movement reuses these
    /// offsets instead of rescanning the entire history on every repaint.
    vt_rows: Vec<std::ops::Range<usize>>,
    /// Rows above the live-bottom viewport, matching tmux's copy-mode `oy`.
    pub(crate) scroll: usize,
}

#[derive(Clone, Debug)]
pub(crate) enum CopyBacking {
    PaneSnapshot,
    ViewOutput(Vec<u8>),
}

impl CopyState {
    pub(crate) fn vt_rows(&self) -> impl Iterator<Item = &[u8]> {
        self.vt_rows.iter().map(|range| &self.vt[range.clone()])
    }

    fn replace_vt(&mut self, vt: Vec<u8>) {
        self.vt_rows = copy_vt_row_ranges(&vt);
        self.vt = vt;
    }
}

fn copy_vt_row_ranges(vt: &[u8]) -> Vec<std::ops::Range<usize>> {
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

/// Direction used by the attach compositor for an evenly divided window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SplitDirection {
    TopBottom,
    LeftRight,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PaneRect {
    pub(crate) top: u16,
    pub(crate) left: u16,
    pub(crate) height: u16,
    pub(crate) width: u16,
}

impl PaneRect {
    fn axis(self, direction: SplitDirection) -> u16 {
        match direction {
            SplitDirection::TopBottom => self.height,
            SplitDirection::LeftRight => self.width,
        }
    }

    fn set_axis(&mut self, direction: SplitDirection, value: u16) {
        match direction {
            SplitDirection::TopBottom => self.height = value,
            SplitDirection::LeftRight => self.width = value,
        }
    }
}

/// Preserved tmux-style pane geometry. Leaves use stable pane IDs so pane-list
/// reordering cannot silently flatten the visible layout.
#[derive(Clone, Debug)]
pub(crate) enum LayoutCell {
    Pane {
        pane_id: u32,
        rect: PaneRect,
    },
    Split {
        direction: SplitDirection,
        rect: PaneRect,
        children: Vec<LayoutCell>,
    },
}

impl LayoutCell {
    fn pane(pane_id: u32, width: u16, height: u16) -> Self {
        Self::Pane {
            pane_id,
            rect: PaneRect {
                top: 0,
                left: 0,
                height: height.max(1),
                width: width.max(1),
            },
        }
    }

    fn even(pane_ids: &[u32], direction: SplitDirection, rect: PaneRect) -> Self {
        if pane_ids.len() == 1 {
            return Self::Pane {
                pane_id: pane_ids[0],
                rect,
            };
        }
        let total = rect
            .axis(direction)
            .saturating_sub(pane_ids.len().saturating_sub(1) as u16);
        let base = total / pane_ids.len() as u16;
        let remainder = total % pane_ids.len() as u16;
        let children = pane_ids
            .iter()
            .enumerate()
            .map(|(index, pane_id)| {
                let mut child_rect = rect;
                child_rect.set_axis(direction, base + u16::from((index as u16) < remainder));
                Self::Pane {
                    pane_id: *pane_id,
                    rect: child_rect,
                }
            })
            .collect();
        let mut layout = Self::Split {
            direction,
            rect,
            children,
        };
        layout.fix_offsets();
        layout
    }

    fn main(
        pane_ids: &[u32],
        direction: SplitDirection,
        rect: PaneRect,
        requested_main: u16,
        requested_other: u16,
        mirrored: bool,
    ) -> Self {
        let available = rect.axis(direction).saturating_sub(1);
        let mut main = requested_main;
        let other;
        if main.saturating_add(1) >= available {
            main = if available <= 2 {
                1
            } else {
                available.saturating_sub(1)
            };
            other = 1;
        } else if requested_other == 0
            || requested_other > available
            || available - requested_other < main
        {
            other = available - main;
        } else {
            other = requested_other;
            main = available - other;
        }
        let mut main_rect = rect;
        main_rect.set_axis(direction, main);
        let mut other_rect = rect;
        other_rect.set_axis(direction, other);
        let cross = match direction {
            SplitDirection::TopBottom => SplitDirection::LeftRight,
            SplitDirection::LeftRight => SplitDirection::TopBottom,
        };
        let main_cell = Self::Pane {
            pane_id: pane_ids[0],
            rect: main_rect,
        };
        let other_cell = Self::even(&pane_ids[1..], cross, other_rect);
        let mut children = if mirrored {
            vec![other_cell, main_cell]
        } else {
            vec![main_cell, other_cell]
        };
        // Ensure the requested main dimension is retained after vector moves.
        for child in &mut children {
            let axis = child.rect().axis(direction);
            child.rect_mut().set_axis(direction, axis);
        }
        let mut layout = Self::Split {
            direction,
            rect,
            children,
        };
        layout.fix_offsets();
        layout
    }

    fn tiled(pane_ids: &[u32], rect: PaneRect, max_columns: usize) -> Self {
        let count = pane_ids.len();
        let mut rows = 1usize;
        let mut columns = 1usize;
        while rows * columns < count {
            rows += 1;
            if rows * columns < count && (max_columns == 0 || columns < max_columns) {
                columns += 1;
            }
        }
        let base_height = rect.height.saturating_sub(rows.saturating_sub(1) as u16) / rows as u16;
        let mut offset = 0usize;
        let mut row_cells = Vec::new();
        for row in 0..rows {
            if offset == count {
                break;
            }
            let row_count = columns.min(count - offset);
            let mut row_rect = rect;
            row_rect.height = if row + 1 == rows {
                rect.height
                    .saturating_sub(base_height.saturating_add(1) * row as u16)
            } else {
                base_height
            };
            let row_layout = if row_count == 1 {
                Self::Pane {
                    pane_id: pane_ids[offset],
                    rect: row_rect,
                }
            } else {
                let base_width = rect
                    .width
                    .saturating_sub(row_count.saturating_sub(1) as u16)
                    / row_count as u16;
                let used = base_width * row_count as u16 + row_count as u16 - 1;
                let children = pane_ids[offset..offset + row_count]
                    .iter()
                    .enumerate()
                    .map(|(index, pane_id)| {
                        let mut child_rect = row_rect;
                        child_rect.width = base_width
                            + if index + 1 == row_count {
                                rect.width.saturating_sub(used)
                            } else {
                                0
                            };
                        Self::Pane {
                            pane_id: *pane_id,
                            rect: child_rect,
                        }
                    })
                    .collect();
                let mut row_layout = Self::Split {
                    direction: SplitDirection::LeftRight,
                    rect: row_rect,
                    children,
                };
                row_layout.fix_offsets();
                row_layout
            };
            row_cells.push(row_layout);
            offset += row_count;
        }
        let mut layout = Self::Split {
            direction: SplitDirection::TopBottom,
            rect,
            children: row_cells,
        };
        layout.fix_offsets();
        layout
    }

    fn rect(&self) -> PaneRect {
        match self {
            Self::Pane { rect, .. } | Self::Split { rect, .. } => *rect,
        }
    }

    fn rect_mut(&mut self) -> &mut PaneRect {
        match self {
            Self::Pane { rect, .. } | Self::Split { rect, .. } => rect,
        }
    }

    fn shift(&mut self, dx: i32, dy: i32) {
        let rect = self.rect_mut();
        rect.left = (rect.left as i32 + dx).max(0) as u16;
        rect.top = (rect.top as i32 + dy).max(0) as u16;
        if let Self::Split { children, .. } = self {
            for child in children {
                child.shift(dx, dy);
            }
        }
    }

    fn set_rect(&mut self, rect: PaneRect) {
        let old = self.rect();
        self.resize(rect.width, rect.height);
        self.shift(
            rect.left as i32 - old.left as i32,
            rect.top as i32 - old.top as i32,
        );
    }

    fn contains(&self, pane_id: u32) -> bool {
        match self {
            Self::Pane { pane_id: id, .. } => *id == pane_id,
            Self::Split { children, .. } => children.iter().any(|child| child.contains(pane_id)),
        }
    }

    pub(crate) fn pane_rect(&self, pane_id: u32) -> Option<PaneRect> {
        match self {
            Self::Pane { pane_id: id, rect } => (*id == pane_id).then_some(*rect),
            Self::Split { children, .. } => {
                children.iter().find_map(|child| child.pane_rect(pane_id))
            }
        }
    }

    pub(crate) fn panes(&self) -> Vec<(u32, PaneRect)> {
        let mut panes = Vec::new();
        self.collect_panes(&mut panes);
        panes
    }

    fn neighbour(&self, pane_id: u32, direction: SplitDirection, forward: bool) -> Option<u32> {
        let current = self.pane_rect(pane_id)?;
        self.panes()
            .into_iter()
            .filter(|(id, rect)| {
                if *id == pane_id {
                    return false;
                }
                let overlaps = match direction {
                    SplitDirection::LeftRight => {
                        rect.top < current.top.saturating_add(current.height)
                            && current.top < rect.top.saturating_add(rect.height)
                    }
                    SplitDirection::TopBottom => {
                        rect.left < current.left.saturating_add(current.width)
                            && current.left < rect.left.saturating_add(rect.width)
                    }
                };
                let ahead = match (direction, forward) {
                    (SplitDirection::LeftRight, true) => rect.left > current.left,
                    (SplitDirection::LeftRight, false) => rect.left < current.left,
                    (SplitDirection::TopBottom, true) => rect.top > current.top,
                    (SplitDirection::TopBottom, false) => rect.top < current.top,
                };
                overlaps && ahead
            })
            .min_by_key(|(_, rect)| match direction {
                SplitDirection::LeftRight => rect.left.abs_diff(current.left),
                SplitDirection::TopBottom => rect.top.abs_diff(current.top),
            })
            .map(|(id, _)| id)
    }

    fn collect_panes(&self, panes: &mut Vec<(u32, PaneRect)>) {
        match self {
            Self::Pane { pane_id, rect } => panes.push((*pane_id, *rect)),
            Self::Split { children, .. } => {
                for child in children {
                    child.collect_panes(panes);
                }
            }
        }
    }

    fn split(
        &mut self,
        target_id: u32,
        new_id: u32,
        direction: SplitDirection,
        before: bool,
    ) -> bool {
        if matches!(self, Self::Pane { pane_id, .. } if *pane_id == target_id) {
            let old = self.rect();
            let (first, second) = split_axis(old.axis(direction));
            let mut old_rect = old;
            old_rect.set_axis(direction, if before { second } else { first });
            let mut new_rect = old;
            new_rect.set_axis(direction, if before { first } else { second });
            let old_leaf = Self::Pane {
                pane_id: target_id,
                rect: old_rect,
            };
            let new_leaf = Self::Pane {
                pane_id: new_id,
                rect: new_rect,
            };
            *self = Self::Split {
                direction,
                rect: old,
                children: if before {
                    vec![new_leaf, old_leaf]
                } else {
                    vec![old_leaf, new_leaf]
                },
            };
            self.fix_offsets();
            return true;
        }

        let Self::Split {
            direction: parent_direction,
            children,
            ..
        } = self
        else {
            return false;
        };
        let Some(index) = children.iter().position(|child| child.contains(target_id)) else {
            return false;
        };
        if *parent_direction == direction
            && matches!(children[index], Self::Pane { pane_id, .. } if pane_id == target_id)
        {
            let old = children[index].rect();
            let (first, second) = split_axis(old.axis(direction));
            let mut old_rect = old;
            old_rect.set_axis(direction, if before { second } else { first });
            let mut new_rect = old;
            new_rect.set_axis(direction, if before { first } else { second });
            children[index] = Self::Pane {
                pane_id: target_id,
                rect: old_rect,
            };
            children.insert(
                if before { index } else { index + 1 },
                Self::Pane {
                    pane_id: new_id,
                    rect: new_rect,
                },
            );
            self.fix_offsets();
            true
        } else {
            let changed = children[index].split(target_id, new_id, direction, before);
            if changed {
                self.fix_offsets();
            }
            changed
        }
    }

    fn resize(&mut self, width: u16, height: u16) {
        let old = self.rect();
        self.resize_axis(
            SplitDirection::LeftRight,
            width.max(1) as i32 - old.width as i32,
        );
        self.resize_axis(
            SplitDirection::TopBottom,
            height.max(1) as i32 - old.height as i32,
        );
        self.fix_offsets();
    }

    fn resize_pane_toward(
        &mut self,
        pane_id: u32,
        direction: SplitDirection,
        forward: bool,
        amount: u16,
    ) -> bool {
        let Self::Split {
            direction: own,
            children,
            ..
        } = self
        else {
            return false;
        };
        let Some(index) = children.iter().position(|child| child.contains(pane_id)) else {
            return false;
        };
        if children[index].resize_pane_toward(pane_id, direction, forward, amount) {
            self.fix_offsets();
            return true;
        }
        if *own != direction {
            return false;
        }
        let donor = if forward {
            index.checked_add(1).filter(|next| *next < children.len())
        } else {
            index.checked_sub(1)
        };
        let Some(donor) = donor else {
            return false;
        };
        let amount = amount.min(children[donor].available(direction));
        if amount == 0 {
            return false;
        }
        children[index].adjust(direction, i32::from(amount));
        children[donor].adjust(direction, -i32::from(amount));
        self.fix_offsets();
        true
    }

    fn resize_pane_to(&mut self, pane_id: u32, direction: SplitDirection, requested: u16) -> bool {
        let Self::Split {
            direction: own,
            children,
            ..
        } = self
        else {
            return false;
        };
        let Some(index) = children.iter().position(|child| child.contains(pane_id)) else {
            return false;
        };
        if children[index].resize_pane_to(pane_id, direction, requested) {
            self.fix_offsets();
            return true;
        }
        if *own != direction || children.len() < 2 {
            return false;
        }
        let current = children[index].rect().axis(direction);
        let mut delta = i32::from(requested) - i32::from(current);
        let receiver = if index + 1 < children.len() {
            index + 1
        } else {
            index - 1
        };
        if delta > 0 {
            delta = delta.min(i32::from(children[receiver].available(direction)));
        } else {
            delta = delta.max(-i32::from(children[index].available(direction)));
        }
        if delta == 0 {
            return true;
        }
        children[index].adjust(direction, delta);
        children[receiver].adjust(direction, -delta);
        self.fix_offsets();
        true
    }

    fn resize_axis(&mut self, direction: SplitDirection, mut change: i32) {
        if change < 0 {
            change = change.max(-(self.available(direction) as i32));
        }
        if change != 0 {
            self.adjust(direction, change);
        }
    }

    fn available(&self, direction: SplitDirection) -> u16 {
        match self {
            Self::Pane { rect, .. } => rect.axis(direction).saturating_sub(1),
            Self::Split {
                direction: own,
                children,
                ..
            } if *own == direction => children
                .iter()
                .map(|child| child.available(direction))
                .sum(),
            Self::Split { children, .. } => children
                .iter()
                .map(|child| child.available(direction))
                .min()
                .unwrap_or(0),
        }
    }

    fn adjust(&mut self, direction: SplitDirection, change: i32) {
        let rect = self.rect_mut();
        let current = rect.axis(direction) as i32;
        rect.set_axis(direction, (current + change).max(1) as u16);
        let Self::Split {
            direction: own,
            children,
            ..
        } = self
        else {
            return;
        };
        if *own != direction {
            for child in children {
                child.adjust(direction, change);
            }
            return;
        }
        let step = change.signum();
        let mut remaining = change.unsigned_abs();
        while remaining > 0 {
            let mut progressed = false;
            for child in children.iter_mut() {
                if remaining == 0 {
                    break;
                }
                if step > 0 || child.available(direction) > 0 {
                    child.adjust(direction, step);
                    remaining -= 1;
                    progressed = true;
                }
            }
            if !progressed {
                break;
            }
        }
    }

    fn fix_offsets(&mut self) {
        let Self::Split {
            direction,
            rect,
            children,
        } = self
        else {
            return;
        };
        let mut left = rect.left;
        let mut top = rect.top;
        for child in children {
            let child_rect = child.rect_mut();
            child_rect.left = left;
            child_rect.top = top;
            match direction {
                SplitDirection::LeftRight => {
                    child_rect.height = rect.height;
                    left = left.saturating_add(child_rect.width).saturating_add(1);
                }
                SplitDirection::TopBottom => {
                    child_rect.width = rect.width;
                    top = top.saturating_add(child_rect.height).saturating_add(1);
                }
            }
            child.fix_offsets();
        }
    }

    fn remove(&mut self, pane_id: u32) -> bool {
        let Self::Split {
            direction,
            rect,
            children,
        } = self
        else {
            return false;
        };
        if let Some(index) = children
            .iter()
            .position(|child| matches!(child, Self::Pane { pane_id: id, .. } if *id == pane_id))
        {
            let removed = children.remove(index);
            if children.is_empty() {
                return true;
            }
            let receiver = if index == 0 { 0 } else { index - 1 };
            children[receiver].adjust(*direction, removed.rect().axis(*direction) as i32 + 1);
            if children.len() == 1 {
                let mut survivor = children.remove(0);
                survivor.set_rect(*rect);
                *self = survivor;
            } else {
                self.fix_offsets();
            }
            return true;
        }
        if let Some(child) = children.iter_mut().find(|child| child.contains(pane_id)) {
            let removed = child.remove(pane_id);
            if removed {
                self.fix_offsets();
            }
            return removed;
        }
        false
    }

    fn keep_only(&mut self, pane_id: u32) {
        let rect = self.rect();
        *self = Self::Pane { pane_id, rect };
    }

    fn replace_pane(&mut self, old: u32, new: u32) -> bool {
        match self {
            Self::Pane { pane_id, .. } if *pane_id == old => {
                *pane_id = new;
                true
            }
            Self::Pane { .. } => false,
            Self::Split { children, .. } => children
                .iter_mut()
                .any(|child| child.replace_pane(old, new)),
        }
    }

    fn swap_panes(&mut self, first: u32, second: u32) {
        if first == second {
            return;
        }
        self.replace_pane(first, u32::MAX);
        self.replace_pane(second, first);
        self.replace_pane(u32::MAX, second);
    }

    fn assign_panes(&mut self, pane_ids: &mut impl Iterator<Item = u32>) {
        match self {
            Self::Pane { pane_id, .. } => {
                if let Some(id) = pane_ids.next() {
                    *pane_id = id;
                }
            }
            Self::Split { children, .. } => {
                for child in children {
                    child.assign_panes(pane_ids);
                }
            }
        }
    }

    pub(crate) fn dump(&self) -> String {
        match self {
            Self::Pane { pane_id, rect } => format!(
                "{}x{},{},{},{}",
                rect.width, rect.height, rect.left, rect.top, pane_id
            ),
            Self::Split {
                direction,
                rect,
                children,
            } => {
                let (open, close) = match direction {
                    SplitDirection::TopBottom => ('[', ']'),
                    SplitDirection::LeftRight => ('{', '}'),
                };
                format!(
                    "{}x{},{},{}{}{}{}",
                    rect.width,
                    rect.height,
                    rect.left,
                    rect.top,
                    open,
                    children
                        .iter()
                        .map(Self::dump)
                        .collect::<Vec<_>>()
                        .join(","),
                    close
                )
            }
        }
    }
}

fn split_axis(size: u16) -> (u16, u16) {
    let second = ((size.saturating_add(1)) / 2)
        .saturating_sub(1)
        .clamp(1, size.saturating_sub(2).max(1));
    (size.saturating_sub(second).saturating_sub(1).max(1), second)
}

fn parse_custom_layout(value: &str) -> io::Result<LayoutCell> {
    let (checksum, body) = value
        .split_once(',')
        .ok_or_else(|| io::Error::other("invalid layout"))?;
    let checksum =
        u16::from_str_radix(checksum, 16).map_err(|_| io::Error::other("invalid layout"))?;
    let actual = body.bytes().fold(0u16, |sum, byte| {
        sum.rotate_right(1).wrapping_add(u16::from(byte))
    });
    if checksum != actual {
        return Err(io::Error::other("invalid layout"));
    }
    let mut parser = LayoutParser {
        bytes: body.as_bytes(),
        offset: 0,
    };
    let layout = parser.cell()?;
    if parser.offset != parser.bytes.len() {
        return Err(io::Error::other("invalid layout"));
    }
    Ok(layout)
}

struct LayoutParser<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl LayoutParser<'_> {
    fn number(&mut self) -> io::Result<u16> {
        let start = self.offset;
        while self.bytes.get(self.offset).is_some_and(u8::is_ascii_digit) {
            self.offset += 1;
        }
        if start == self.offset {
            return Err(io::Error::other("invalid layout"));
        }
        std::str::from_utf8(&self.bytes[start..self.offset])
            .ok()
            .and_then(|value| value.parse().ok())
            .ok_or_else(|| io::Error::other("invalid layout"))
    }

    fn expect(&mut self, byte: u8) -> io::Result<()> {
        if self.bytes.get(self.offset) != Some(&byte) {
            return Err(io::Error::other("invalid layout"));
        }
        self.offset += 1;
        Ok(())
    }

    fn cell(&mut self) -> io::Result<LayoutCell> {
        let width = self.number()?;
        self.expect(b'x')?;
        let height = self.number()?;
        self.expect(b',')?;
        let left = self.number()?;
        self.expect(b',')?;
        let top = self.number()?;
        let rect = PaneRect {
            top,
            left,
            height,
            width,
        };
        match self.bytes.get(self.offset).copied() {
            Some(b',') => {
                self.offset += 1;
                let pane_id = self.number()?;
                Ok(LayoutCell::Pane {
                    pane_id: u32::from(pane_id),
                    rect,
                })
            }
            Some(open @ (b'[' | b'{')) => {
                self.offset += 1;
                let close = if open == b'[' { b']' } else { b'}' };
                let direction = if open == b'[' {
                    SplitDirection::TopBottom
                } else {
                    SplitDirection::LeftRight
                };
                let mut children = Vec::new();
                loop {
                    children.push(self.cell()?);
                    match self.bytes.get(self.offset).copied() {
                        Some(byte) if byte == close => {
                            self.offset += 1;
                            break;
                        }
                        Some(b',') => self.offset += 1,
                        _ => return Err(io::Error::other("invalid layout")),
                    }
                }
                if children.is_empty() {
                    return Err(io::Error::other("invalid layout"));
                }
                Ok(LayoutCell::Split {
                    direction,
                    rect,
                    children,
                })
            }
            _ => Err(io::Error::other("invalid layout")),
        }
    }
}

fn resize_panes_to_layout(window: &mut Window) -> io::Result<()> {
    for pane in &mut window.panes {
        if let Some(rect) = pane.floating.or_else(|| window.layout.pane_rect(pane.id)) {
            if let Some(copy) = pane.copy.as_mut() {
                reflow_copy_snapshot(copy, rect.width.max(1), rect.height.max(1))?;
            }
            pane.pane.resize(rect.width.max(1), rect.height.max(1))?;
        }
    }
    Ok(())
}

fn reflow_copy_snapshot(state: &mut CopyState, cols: u16, rows: u16) -> io::Result<()> {
    if state.grid.cols == cols && state.grid.viewport_rows == rows {
        return Ok(());
    }
    let cursor_offset = copy_point_offset(&state.grid, (state.cursor.row, state.cursor.col));
    let mark_offset = state
        .mark
        .map(|point| copy_point_offset(&state.grid, point));
    let mut metadata = Vec::new();
    for (row, line) in state.grid.rows.iter().enumerate() {
        for (col, cell) in line.cells.iter().enumerate() {
            if cell.semantic != ghostty_sys::GridCellSemantic::Output || cell.hyperlink.is_some() {
                metadata.push((
                    copy_point_offset(&state.grid, (row, col)),
                    cell.semantic,
                    cell.hyperlink.clone(),
                ));
            }
        }
    }
    let mut terminal = ghostty_sys::Terminal::new(state.grid.cols, state.grid.viewport_rows)
        .map_err(|error| io::Error::other(format!("ghostty error: {error:?}")))?;
    terminal.write(&copy_reflow_vt(state));
    terminal
        .resize(cols, rows)
        .map_err(|error| io::Error::other(format!("ghostty error: {error:?}")))?;
    state.grid = terminal
        .grid_snapshot()
        .map_err(|error| io::Error::other(format!("ghostty error: {error:?}")))?;
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
    let vt = terminal
        .dump_vt()
        .map_err(|error| io::Error::other(format!("ghostty error: {error:?}")))?;
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

fn view_copy_state(output: Vec<u8>, cols: u16, rows: u16) -> io::Result<CopyState> {
    let mut terminal = ghostty_sys::Terminal::new(cols.max(1), rows.max(1))
        .map_err(|error| io::Error::other(format!("ghostty error: {error:?}")))?;
    terminal.write(&view_output_vt(&output));
    let grid = terminal
        .grid_snapshot()
        .map_err(|error| io::Error::other(format!("ghostty error: {error:?}")))?;
    let vt = terminal
        .dump_vt()
        .map_err(|error| io::Error::other(format!("ghostty error: {error:?}")))?;
    let vt_rows = copy_vt_row_ranges(&vt);
    let (col, row) = terminal
        .cursor_position()
        .map_err(|error| io::Error::other(format!("ghostty error: {error:?}")))?;
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
        search_count: Some(0),
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

fn copy_point_offset(grid: &ghostty_sys::GridSnapshot, point: (usize, usize)) -> usize {
    let mut offset = 0;
    for row in 0..point.0.min(grid.rows.len()) {
        offset += copy_line_length(grid, row);
        if !grid.rows[row].wrapped {
            offset += 1;
        }
    }
    offset + point.1.min(copy_line_length(grid, point.0))
}

fn copy_point_at_offset(grid: &ghostty_sys::GridSnapshot, mut offset: usize) -> (usize, usize) {
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

/// One window: an ordered set of panes with an active index.
pub struct Window {
    pub id: u32,
    pub name: String,
    pub panes: Vec<PaneNode>,
    pub active: usize,
    /// Vec position of the previously-active pane (tmux's `{last}` pane target).
    pub last_pane: Option<usize>,
    /// Whether the active pane is zoomed (`resize-pane -Z`), tmux's
    /// `#{window_zoomed_flag}`. Toggled by `resize-pane -Z`.
    pub zoomed: bool,
    /// A manual `(cols, rows)` size forced by `resize-window -x/-y`. When set it
    /// overrides the session's client size for this window's
    /// `#{window_width}`/`#{window_height}`; `None` means the window tracks the
    /// session size (tmux's default automatic sizing).
    pub manual_size: Option<(u16, u16)>,
    pub(crate) layout: LayoutCell,
    pub(crate) last_layout: Option<usize>,
    pub(crate) last_new_pane_x: u16,
    pub(crate) last_new_pane_y: u16,
    /// Conditions raised for this window and not yet turned into alerts, as
    /// `ALERT_*` bits — tmux's `WINDOW_BELL`/`WINDOW_ACTIVITY`/`WINDOW_SILENCE`
    /// window flags. A bit is cleared only when its monitor option is on and
    /// the alert is actually delivered, which is what makes a monitor enabled
    /// after the fact still see the condition that is already pending.
    pub(crate) pending_alerts: u8,
    options: OptionSet,
}

/// A session-facing link to a globally owned window.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Winlink {
    /// Stable identity of this logical link within its synchronization domain.
    pub(crate) link_id: u64,
    /// The client-visible window index (`#{window_index}`).
    pub index: u32,
    /// Stable identity of the linked [`Window`].
    pub id: u32,
    pub(crate) alert_flags: u8,
}

pub(crate) const ALERT_BELL: u8 = 0x1;
pub(crate) const ALERT_ACTIVITY: u8 = 0x2;
pub(crate) const ALERT_SILENCE: u8 = 0x4;

/// One session: an ordered winlink set plus session-local selection and size.
/// Session groups synchronize these lists explicitly, as tmux does; they are
/// not the same allocation because current/last state and transient index
/// changes remain session-local.
pub struct Session {
    pub id: u32,
    pub name: String,
    pub windows: Vec<Winlink>,
    /// Identity of the session group synchronization domain. Grouped sessions
    /// use the same value; ungrouped sessions have unique values.
    pub(crate) link_set_id: u32,
    pub active: usize,              // Vec position of the active window
    pub last_active: Option<usize>, // Vec position of the previously-active window
    /// MRU winlink identities excluding the current window, matching tmux's
    /// full `lastw` stack. `last_active` is the cached first live position.
    pub(crate) last_windows: Vec<u64>,
    pub cols: u16,
    pub rows: u16,
    /// Creation time, formatted like tmux's `(created ...)` stamp. Volatile, so
    /// conformance normalizes it away; kept so the default `list-sessions` line
    /// is structurally identical to real tmux's.
    pub created: String,
    /// Creation time as Unix epoch seconds — tmux's `#{session_created}`. Its
    /// exact value is volatile (conformance only checks its truthiness), but it
    /// is always set, so `#{?session_created,…}` behaves like real tmux.
    pub created_epoch: i64,
    /// Last activity, in microseconds since the epoch — tmux's
    /// `session_update_activity`, exposed as `#{session_activity}` (seconds).
    /// Microsecond resolution is what makes sessions created in the same second
    /// still orderable, which `detach-on-destroy` relies on.
    pub(crate) activity_micros: i64,
    /// When a client last attached to this session, in microseconds since the
    /// epoch; zero while the session has never been attached. tmux's
    /// `#{session_last_attached}` (seconds).
    pub(crate) last_attached_micros: i64,
    /// The `activity_micros` value the automatic lock last fired for. tmux's
    /// lock timer is armed by activity and fires once, so a session that stays
    /// idle is not locked again until somebody types.
    locked_at_activity_micros: Option<i64>,
    environment: BTreeMap<String, String>,
    removed_environment: BTreeSet<String>,
    hidden_environment: BTreeSet<String>,
    options: OptionSet,
}

/// One event notification waiting to become hook bodies.
///
/// tmux appends an equivalent `notify_entry` callback to the command queue at
/// each mutation point; hmux records it while the state lock is held and lets
/// the command queue drain it once the triggering command has finished.
#[derive(Clone, Debug)]
pub(crate) struct Notification {
    /// Hook name, e.g. `window-linked`.
    pub(crate) name: String,
    /// Target the hook's option lookup and its body's default target resolve
    /// against. `None` when the subject no longer exists.
    pub(crate) target: Option<String>,
    /// `hook*` format variables the body sees.
    pub(crate) vars: Vec<(String, String)>,
    /// Raised outside any command (a pane exiting, an alert firing), so the
    /// server loop must run its body rather than the command queue.
    pub(crate) deferred: bool,
}

/// A key-table entry installed by `bind-key`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyBinding {
    pub repeat: bool,
    pub note: Option<String>,
    pub command: Vec<String>,
}

/// Client-scoped `command-prompt -k` routing. This is an internal server
/// capability, deliberately kept out of the public `TmuxServer` trait.
pub(crate) struct ClientPromptRegistry {
    inner: Mutex<ClientPromptRegistryState>,
    activity: AtomicU64,
}

#[derive(Default)]
struct ClientPromptRegistryState {
    next_id: u64,
    clients: BTreeMap<u64, ClientPromptEntry>,
}

struct ClientPromptEntry {
    name: String,
    tty_name: String,
    slot: Arc<ClientPromptSlot>,
    activity: Arc<AtomicU64>,
}

struct ClientPromptSlot {
    inner: Mutex<ClientPromptSlotState>,
    wakeup: <CurrentPlatform as Platform>::OutputWakeup,
}

#[derive(Default)]
struct ClientPromptSlotState {
    queued_command: Option<QueuedCommandPrompt>,
    active: bool,
}

struct QueuedCommandPrompt {
    args: Vec<String>,
    reply: mpsc::Sender<Option<PromptCompletion>>,
}

/// Registration owned by one interactive attach loop.
pub(crate) struct ClientPromptAttachment {
    registry: Arc<ClientPromptRegistry>,
    id: u64,
    slot: Arc<ClientPromptSlot>,
    activity: Arc<AtomicU64>,
}

pub(crate) struct ActiveCommandPrompt {
    slot: Arc<ClientPromptSlot>,
    args: Vec<String>,
    reply: Option<mpsc::Sender<Option<PromptCompletion>>>,
}

#[derive(Clone, Debug)]
pub(crate) struct PromptCompletion {
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) exit: i32,
    pub(crate) inserted: bool,
}

pub(crate) enum CommandPromptRequestResult {
    Completed(PromptCompletion),
    Queued,
    NoCurrentClient,
    TargetNotFound,
    Busy,
}

/// Coalesced reasons an attached client's compositor must inspect server state.
///
/// Pane output keeps its dedicated notification path; these bits cover state
/// changes made by command clients while an attach loop may be blocked waiting
/// on a pane that is no longer active.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RenderInvalidation(u8);

impl RenderInvalidation {
    pub(crate) const STATUS: Self = Self(1 << 0);
    pub(crate) const LAYOUT: Self = Self(1 << 1);
    pub(crate) const SESSION_GONE: Self = Self(1 << 2);
    pub(crate) const RESET_MODE: Self = Self(1 << 3);
    pub(crate) const MODE: Self = Self(1 << 4);
    pub(crate) const TERMINAL: Self = Self(1 << 5);

    fn bits(self) -> u8 {
        self.0
    }

    pub(crate) fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub(crate) fn contains(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

impl std::ops::BitOr for RenderInvalidation {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

/// Per-client state-change notifications, scoped by stable session id.
pub(crate) struct ClientRenderRegistry {
    inner: Mutex<ClientRenderRegistryState>,
    /// Bumped whenever the set of clients, or which session one is on,
    /// changes. Lets the server loop skip the lifecycle sweep when nothing
    /// the policies depend on has moved.
    generation: AtomicU64,
}

#[derive(Default)]
struct ClientRenderRegistryState {
    next_id: u64,
    clients: BTreeMap<u64, ClientRenderEntry>,
}

struct ClientRenderEntry {
    session_id: u32,
    name: String,
    term: String,
    pid: Option<i32>,
    cols: u16,
    rows: u16,
    flags: String,
    read_only: bool,
    control_mode: bool,
    ignore_size: bool,
    size_changed: bool,
    /// Identity bits the client sent at handshake time (`CLIENT_UTF8`, …),
    /// needed to rebuild the display flag string server-side.
    identified: i64,
    /// Whether the client's terminal currently has focus.
    focused: bool,
    /// The theme the client's terminal last reported (`dark`/`light`), empty
    /// until it says.
    theme: String,
    /// The key table the client is currently in — tmux's `c->keytable`, which
    /// `#{client_key_table}` and `#{client_prefix}` report.
    key_table: String,
    /// When this client last sent a key, in microseconds since the epoch —
    /// tmux's `c->activity_time`, which orders `cmd_find_best_client`.
    activity_micros: i64,
    /// When a clipboard query was last sent to this client's terminal.
    clipboard_query_at: Option<Instant>,
    flag_state: ClientFlagState,
    terminal: Option<ResolvedTerm>,
    slot: Arc<ClientRenderSlot>,
}

impl ClientRenderEntry {
    /// Apply one `refresh-client -f` value and republish the derived views the
    /// rest of the server reads (`#{client_flags}`, sizing, read-only checks).
    fn apply_flag_value(&mut self, value: &str) {
        self.flag_state.apply_flags(value);
        self.ignore_size = self.flag_state.ignore_size;
        self.read_only = self.flag_state.read_only;
        self.flags = self
            .flag_state
            .display_flags_full(self.identified, self.control_mode, self.focused);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AttachedClient {
    pub(crate) session_id: u32,
    pub(crate) name: String,
    pub(crate) term: String,
    pub(crate) pid: Option<i32>,
    pub(crate) cols: u16,
    pub(crate) rows: u16,
    pub(crate) flags: String,
    pub(crate) read_only: bool,
    pub(crate) control_mode: bool,
    pub(crate) ignore_size: bool,
    pub(crate) size_changed: bool,
    pub(crate) terminal: Option<ResolvedTerm>,
    /// The theme this client's terminal reported (`dark`/`light`), empty until
    /// it says — tmux's `#{client_theme}`.
    pub(crate) theme: String,
    /// Whether the client's terminal currently has focus.
    pub(crate) focused: bool,
    /// The key table the client is in — tmux's `#{client_key_table}`.
    pub(crate) key_table: String,
    /// When this client last sent a key, in microseconds since the epoch.
    pub(crate) activity_micros: i64,
}

#[derive(Clone)]
pub(crate) struct ControlPaneSnapshot {
    pub(crate) id: u32,
    pub(crate) runtime_id: u64,
    pub(crate) observation: Arc<NativePaneObservation>,
}

#[derive(Clone)]
pub(crate) struct ControlGlobalWindowSnapshot {
    pub(crate) name: String,
    pub(crate) links: usize,
}

#[derive(Clone)]
pub(crate) struct ControlWindowSnapshot {
    pub(crate) id: u32,
    pub(crate) index: u32,
    pub(crate) name: String,
    pub(crate) layout: String,
    pub(crate) flags: String,
    pub(crate) active_pane_id: u32,
    pub(crate) panes: Vec<ControlPaneSnapshot>,
}

#[derive(Clone)]
pub(crate) struct ControlStateSnapshot {
    pub(crate) session_id: u32,
    pub(crate) session_name: String,
    pub(crate) active_window_id: u32,
    pub(crate) windows: BTreeMap<u32, ControlWindowSnapshot>,
    pub(crate) sessions: BTreeMap<u32, String>,
    pub(crate) global_windows: BTreeMap<u32, ControlGlobalWindowSnapshot>,
    pub(crate) pane_modes: BTreeMap<u32, Option<String>>,
    pub(crate) buffers: BTreeMap<String, u64>,
    pub(crate) clients: BTreeMap<String, (u32, String)>,
}

struct ControlCheckpoint {
    sequence: u64,
    snapshots: BTreeMap<u32, ControlStateSnapshot>,
}

const CONTROL_CHECKPOINT_LIMIT: usize = 1024;

/// The key table a client is in until a session's `key-table` option or the
/// prefix key moves it elsewhere — tmux's `server_client_get_key_table`.
pub(crate) const DEFAULT_KEY_TABLE: &str = "root";

const CLIENT_READONLY: i64 = 0x800;
const CLIENT_UTF8: i64 = 0x10000;
const CLIENT_IGNORESIZE: i64 = 0x20000;
const CLIENT_CONTROL_NOOUTPUT: i64 = 0x4000000;
const CLIENT_ACTIVEPANE: i64 = 0x80000000;
const CLIENT_CONTROL_PAUSEAFTER: i64 = 0x100000000;
const CLIENT_CONTROL_WAITEXIT: i64 = 0x200000000;
const CLIENT_NO_DETACH_ON_DESTROY: i64 = 0x8000000000;

/// The `refresh-client -f` flag set of one client.
///
/// Both the client that owns the terminal and the registry entry other clients
/// see keep one of these, so a `refresh-client -t other -f …` is visible to
/// `#{client_flags}` and to the destroy policy without waiting for the target
/// client to wake up.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ClientFlagState {
    pub(crate) pause_after: Option<Duration>,
    pub(crate) no_output: bool,
    pub(crate) wait_exit: bool,
    pub(crate) read_only: bool,
    pub(crate) ignore_size: bool,
    pub(crate) active_pane: bool,
    pub(crate) no_detach_on_destroy: bool,
}

impl ClientFlagState {
    pub(crate) fn apply_flags(&mut self, value: &str) {
        for flag in value.split(',') {
            let (clear, flag) = flag
                .strip_prefix('!')
                .map_or((false, flag), |flag| (true, flag));
            if flag == "pause-after" || flag.starts_with("pause-after=") {
                if clear {
                    self.pause_after = None;
                } else {
                    let seconds = flag
                        .strip_prefix("pause-after=")
                        .and_then(|value| value.parse::<u64>().ok())
                        .unwrap_or(0);
                    self.pause_after = Some(Duration::from_secs(seconds));
                }
            } else {
                let enabled = !clear;
                match flag {
                    "no-output" => self.no_output = enabled,
                    "wait-exit" => self.wait_exit = enabled,
                    // An established read-only client cannot clear itself with
                    // refresh-client; switch-client -r is the owner escape.
                    "read-only" if enabled || !self.read_only => self.read_only = enabled,
                    "ignore-size" => self.ignore_size = enabled,
                    "active-pane" => self.active_pane = enabled,
                    "no-detach-on-destroy" => self.no_detach_on_destroy = enabled,
                    _ => {}
                }
            }
        }
    }

    pub(crate) fn client_flags(&self, identified: i64) -> i64 {
        let mut flags = identified;
        for (enabled, flag) in [
            (self.no_output, CLIENT_CONTROL_NOOUTPUT),
            (self.wait_exit, CLIENT_CONTROL_WAITEXIT),
            (self.pause_after.is_some(), CLIENT_CONTROL_PAUSEAFTER),
            (self.read_only, CLIENT_READONLY),
            (self.ignore_size, CLIENT_IGNORESIZE),
            (self.active_pane, CLIENT_ACTIVEPANE),
            (self.no_detach_on_destroy, CLIENT_NO_DETACH_ON_DESTROY),
        ] {
            if enabled {
                flags |= flag;
            } else {
                flags &= !flag;
            }
        }
        flags
    }

    pub(crate) fn display_flags(&self, identified: i64) -> String {
        self.display_flags_with(identified, true)
    }

    /// tmux's `server_client_get_flags` ordering. `control_mode` selects the
    /// `control-mode` marker an interactive attach does not carry.
    pub(crate) fn display_flags_with(&self, identified: i64, control_mode: bool) -> String {
        self.display_flags_full(identified, control_mode, true)
    }

    pub(crate) fn display_flags_full(
        &self,
        identified: i64,
        control_mode: bool,
        focused: bool,
    ) -> String {
        let mut flags = vec!["attached"];
        if focused {
            flags.push("focused");
        }
        if control_mode {
            flags.push("control-mode");
        }
        if self.ignore_size {
            flags.push("ignore-size");
        }
        if self.no_detach_on_destroy {
            flags.push("no-detach-on-destroy");
        }
        if self.no_output {
            flags.push("no-output");
        }
        if self.wait_exit {
            flags.push("wait-exit");
        }
        let pause_after = self
            .pause_after
            .map(|duration| format!("pause-after={}", duration.as_secs()));
        if let Some(pause_after) = pause_after.as_deref() {
            flags.push(pause_after);
        }
        if self.read_only {
            flags.push("read-only");
        }
        if self.active_pane {
            flags.push("active-pane");
        }
        if identified & CLIENT_UTF8 != 0 {
            flags.push("UTF-8");
        }
        flags.join(",")
    }
}

struct ClientRenderSlot {
    pending: AtomicU8,
    action: Mutex<Option<ClientAction>>,
    messages: Mutex<VecDeque<ClientMessage>>,
    /// `refresh-client -f` values aimed at this client by *another* client.
    /// Kept out of `action` because flag updates must not displace a queued
    /// switch or detach, and several may arrive before the client next runs.
    flag_updates: Mutex<Vec<String>>,
    wakeup: <CurrentPlatform as Platform>::OutputWakeup,
}

/// Registration owned by one interactive attach loop.
pub(crate) struct ClientRenderAttachment {
    registry: Arc<ClientRenderRegistry>,
    id: u64,
    slot: Arc<ClientRenderSlot>,
}

impl ClientRenderRegistry {
    fn new() -> Self {
        Self {
            inner: Mutex::new(ClientRenderRegistryState::default()),
            generation: AtomicU64::new(0),
        }
    }

    fn bump_generation(&self) {
        self.generation.fetch_add(1, Ordering::Release);
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    #[cfg(test)]
    pub(crate) fn attach(
        self: &Arc<Self>,
        session_id: u32,
        name: String,
    ) -> io::Result<ClientRenderAttachment> {
        self.attach_with_details(
            session_id,
            name,
            String::new(),
            None,
            80,
            24,
            String::new(),
            0,
            ClientFlagState::default(),
            false,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn attach_with_details(
        self: &Arc<Self>,
        session_id: u32,
        name: String,
        term: String,
        pid: Option<i32>,
        cols: u16,
        rows: u16,
        flags: String,
        identified: i64,
        flag_state: ClientFlagState,
        control_mode: bool,
    ) -> io::Result<ClientRenderAttachment> {
        let read_only = flag_state.read_only;
        let wakeup = CurrentPlatform::new_output_wakeup()?;
        wakeup.clear()?;
        let slot = Arc::new(ClientRenderSlot {
            pending: AtomicU8::new(0),
            action: Mutex::new(None),
            messages: Mutex::new(VecDeque::new()),
            flag_updates: Mutex::new(Vec::new()),
            wakeup,
        });
        let ignore_size = flags.split(',').any(|flag| flag == "ignore-size");
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| io::Error::other("client render registry poisoned"))?;
        let id = inner.next_id;
        inner.next_id = inner.next_id.wrapping_add(1);
        inner.clients.insert(
            id,
            ClientRenderEntry {
                session_id,
                name,
                term,
                pid,
                cols,
                rows,
                flags,
                read_only,
                control_mode,
                ignore_size,
                size_changed: !control_mode,
                identified,
                focused: true,
                theme: String::new(),
                key_table: DEFAULT_KEY_TABLE.to_string(),
                activity_micros: now_micros(),
                clipboard_query_at: None,
                flag_state,
                terminal: None,
                slot: Arc::clone(&slot),
            },
        );
        let peers = inner
            .clients
            .iter()
            .filter(|(client_id, _)| **client_id != id)
            .map(|(_, entry)| Arc::clone(&entry.slot))
            .collect::<Vec<_>>();
        drop(inner);
        self.bump_generation();
        for peer in peers {
            peer.pending
                .fetch_or(RenderInvalidation::STATUS.bits(), Ordering::Release);
            let _ = peer.wakeup.wake();
        }
        Ok(ClientRenderAttachment {
            registry: Arc::clone(self),
            id,
            slot,
        })
    }

    fn clients(&self) -> Vec<AttachedClient> {
        let Ok(inner) = self.inner.lock() else {
            return Vec::new();
        };
        inner
            .clients
            .values()
            .map(|entry| AttachedClient {
                session_id: entry.session_id,
                name: entry.name.clone(),
                term: entry.term.clone(),
                pid: entry.pid,
                cols: entry.cols,
                rows: entry.rows,
                flags: entry.flags.clone(),
                read_only: entry.read_only,
                control_mode: entry.control_mode,
                ignore_size: entry.ignore_size,
                size_changed: entry.size_changed,
                terminal: entry.terminal.clone(),
                theme: entry.theme.clone(),
                focused: entry.focused,
                key_table: entry.key_table.clone(),
                activity_micros: entry.activity_micros,
            })
            .collect()
    }

    fn names_for_sessions(&self, session_ids: &BTreeSet<u32>) -> Vec<String> {
        let Ok(inner) = self.inner.lock() else {
            return Vec::new();
        };
        inner
            .clients
            .values()
            .filter(|entry| session_ids.contains(&entry.session_id))
            .map(|entry| entry.name.clone())
            .collect()
    }

    /// Deliver one window alert to every non-control client of `session_id`.
    fn announce_alert(&self, session_id: u32, bell: bool, text: Option<String>, duration_ms: u64) {
        let Ok(inner) = self.inner.lock() else {
            return;
        };
        for entry in inner
            .clients
            .values()
            // tmux's `alerts_set_message` skips control clients outright.
            .filter(|entry| entry.session_id == session_id && !entry.control_mode)
        {
            let Ok(mut messages) = entry.slot.messages.lock() else {
                continue;
            };
            messages.push_back(ClientMessage {
                text: text.clone().unwrap_or_default(),
                duration_ms,
                bell,
            });
            let _ = entry.slot.wakeup.wake();
        }
    }

    pub(crate) fn publish_session(&self, session_id: u32, reason: RenderInvalidation) {
        let Ok(inner) = self.inner.lock() else {
            return;
        };
        for entry in inner
            .clients
            .values()
            .filter(|entry| entry.session_id == session_id)
        {
            entry
                .slot
                .pending
                .fetch_or(reason.bits(), Ordering::Release);
            let _ = entry.slot.wakeup.wake();
        }
    }

    fn publish_all(&self, reason: RenderInvalidation) {
        let Ok(inner) = self.inner.lock() else {
            return;
        };
        for entry in inner.clients.values() {
            entry
                .slot
                .pending
                .fetch_or(reason.bits(), Ordering::Release);
            let _ = entry.slot.wakeup.wake();
        }
    }

    fn queue_lock(entry: &ClientRenderEntry, command: &str) {
        // tmux's `server_lock_client` refuses to send an empty MSG_LOCK, so an
        // empty `lock-command` leaves the client running rather than suspending
        // it on a command that would do nothing.
        if command.is_empty() {
            return;
        }
        if let Ok(mut action) = entry.slot.action.lock() {
            *action = Some(ClientAction::Lock(command.to_string()));
            let _ = entry.slot.wakeup.wake();
        }
    }

    fn lock_all(&self, commands: &BTreeMap<u32, String>) {
        let Ok(inner) = self.inner.lock() else {
            return;
        };
        for entry in inner.clients.values() {
            if let Some(command) = commands.get(&entry.session_id) {
                Self::queue_lock(entry, command);
            }
        }
    }

    fn lock_session(&self, session_id: u32, command: &str) {
        let Ok(inner) = self.inner.lock() else {
            return;
        };
        for entry in inner
            .clients
            .values()
            .filter(|entry| entry.session_id == session_id)
        {
            Self::queue_lock(entry, command);
        }
    }

    fn lock_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        commands: &BTreeMap<u32, String>,
    ) -> ClientActionResult {
        let Ok(inner) = self.inner.lock() else {
            return ClientActionResult::NoCurrentClient;
        };
        let explicit = target.map(|target| target.strip_suffix(':').unwrap_or(target));
        let selected = if let Some(target) = explicit {
            inner.clients.values().find(|entry| {
                entry.name == target
                    || entry
                        .name
                        .strip_prefix("/dev/")
                        .is_some_and(|tty| tty == target)
            })
        } else {
            invoking_tty.and_then(|tty| inner.clients.values().find(|entry| entry.name == tty))
        };
        let Some(entry) = selected else {
            return if explicit.is_some() {
                ClientActionResult::TargetNotFound
            } else {
                ClientActionResult::NoCurrentClient
            };
        };
        if let Some(command) = commands.get(&entry.session_id) {
            Self::queue_lock(entry, command);
        }
        ClientActionResult::Queued
    }

    fn client_entry<'a>(
        inner: &'a ClientRenderRegistryState,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> Result<&'a ClientRenderEntry, ClientActionResult> {
        let explicit = target.map(|target| target.strip_suffix(':').unwrap_or(target));
        let selected = if let Some(target) = explicit {
            inner.clients.values().find(|entry| {
                entry.name == target
                    || entry
                        .name
                        .strip_prefix("/dev/")
                        .is_some_and(|tty| tty == target)
            })
        } else {
            invoking_tty.and_then(|tty| inner.clients.values().find(|entry| entry.name == tty))
        };
        selected.ok_or(if explicit.is_some() {
            ClientActionResult::TargetNotFound
        } else {
            ClientActionResult::NoCurrentClient
        })
    }

    fn client_id_for(
        inner: &ClientRenderRegistryState,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> Result<u64, ClientActionResult> {
        let explicit = target.map(|target| target.strip_suffix(':').unwrap_or(target));
        let selected = if let Some(target) = explicit {
            inner.clients.iter().find(|(_, entry)| {
                entry.name == target
                    || entry
                        .name
                        .strip_prefix("/dev/")
                        .is_some_and(|tty| tty == target)
            })
        } else {
            invoking_tty
                .and_then(|tty| inner.clients.iter().find(|(_, entry)| entry.name == tty))
        };
        selected
            .map(|(id, _)| *id)
            .ok_or(if explicit.is_some() {
                ClientActionResult::TargetNotFound
            } else {
                ClientActionResult::NoCurrentClient
            })
    }

    /// Queue `refresh-client -f` values for a client other than the one running
    /// the command. The client applies them to its own flag state on its next
    /// turn; the registry copy is what `#{client_flags}` and the destroy policy
    /// read in the meantime.
    fn refresh_client_flags(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        values: &[String],
    ) -> ClientActionResult {
        let Ok(mut inner) = self.inner.lock() else {
            return ClientActionResult::NoCurrentClient;
        };
        let id = match Self::client_id_for(&inner, target, invoking_tty) {
            Ok(id) => id,
            Err(result) => return result,
        };
        let entry = inner.clients.get_mut(&id).expect("selected client present");
        for value in values {
            entry.apply_flag_value(value);
        }
        if let Ok(mut pending) = entry.slot.flag_updates.lock() {
            pending.extend(values.iter().cloned());
        }
        let _ = entry.slot.wakeup.wake();
        ClientActionResult::Queued
    }

    fn send_message(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        session_id: u32,
        message: ClientMessage,
    ) -> ClientMessageResult {
        let Ok(inner) = self.inner.lock() else {
            return ClientMessageResult::NoClient;
        };
        let explicit = target.map(|target| target.strip_suffix(':').unwrap_or(target));
        let selected = if let Some(target) = explicit {
            inner.clients.values().find(|entry| {
                entry.name == target
                    || entry
                        .name
                        .strip_prefix("/dev/")
                        .is_some_and(|tty| tty == target)
            })
        } else {
            invoking_tty
                .and_then(|tty| inner.clients.values().find(|entry| entry.name == tty))
                .or_else(|| {
                    inner
                        .clients
                        .values()
                        .find(|entry| entry.session_id == session_id)
                })
        };
        let Some(entry) = selected else {
            return if explicit.is_some() {
                ClientMessageResult::TargetNotFound
            } else {
                ClientMessageResult::NoClient
            };
        };
        if entry.control_mode && invoking_tty.is_some_and(|invoking_tty| entry.name == invoking_tty)
        {
            return ClientMessageResult::CurrentControl;
        }
        let Ok(mut messages) = entry.slot.messages.lock() else {
            return ClientMessageResult::NoClient;
        };
        messages.push_back(message);
        let _ = entry.slot.wakeup.wake();
        ClientMessageResult::Queued
    }

    fn detach_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> ClientActionResult {
        let Ok(inner) = self.inner.lock() else {
            return ClientActionResult::NoCurrentClient;
        };
        let entry = match Self::client_entry(&inner, target, invoking_tty) {
            Ok(entry) => entry,
            Err(result) => return result,
        };
        if let Ok(mut action) = entry.slot.action.lock() {
            *action = Some(ClientAction::Detach);
            let _ = entry.slot.wakeup.wake();
        }
        ClientActionResult::Queued
    }

    fn suspend_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> ClientActionResult {
        let Ok(inner) = self.inner.lock() else {
            return ClientActionResult::NoCurrentClient;
        };
        let entry = match Self::client_entry(&inner, target, invoking_tty) {
            Ok(entry) => entry,
            Err(result) => return result,
        };
        if let Ok(mut action) = entry.slot.action.lock() {
            *action = Some(ClientAction::Suspend);
            let _ = entry.slot.wakeup.wake();
        }
        ClientActionResult::Queued
    }

    fn refresh_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> ClientActionResult {
        let Ok(inner) = self.inner.lock() else {
            return ClientActionResult::NoCurrentClient;
        };
        let entry = match Self::client_entry(&inner, target, invoking_tty) {
            Ok(entry) => entry,
            Err(result) => return result,
        };
        let reason = RenderInvalidation::STATUS | RenderInvalidation::LAYOUT;
        entry
            .slot
            .pending
            .fetch_or(reason.bits(), Ordering::Release);
        let _ = entry.slot.wakeup.wake();
        ClientActionResult::Queued
    }

    fn client_read_only(&self, target: Option<&str>, invoking_tty: Option<&str>) -> Option<bool> {
        let inner = self.inner.lock().ok()?;
        Self::client_entry(&inner, target, invoking_tty)
            .ok()
            .map(|entry| entry.read_only)
    }

    fn send_client_keys(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        keys: Vec<ClientKey>,
    ) -> ClientActionResult {
        let Ok(inner) = self.inner.lock() else {
            return ClientActionResult::NoCurrentClient;
        };
        let entry = match Self::client_entry(&inner, target, invoking_tty) {
            Ok(entry) => entry,
            Err(result) => return result,
        };
        if let Ok(mut action) = entry.slot.action.lock() {
            match action.as_mut() {
                Some(ClientAction::Keys(queued)) => queued.extend(keys),
                _ => *action = Some(ClientAction::Keys(keys)),
            }
            let _ = entry.slot.wakeup.wake();
        }
        ClientActionResult::Queued
    }

    /// Write, or with `None` query, the terminal selection of one client.
    ///
    /// An untargeted request run from a client with no terminal of its own —
    /// an ordinary command client — falls back to the sole attached client,
    /// which is what tmux's `cmd_find_current_client` resolves to.
    fn set_client_selection(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        data: Option<Vec<u8>>,
    ) -> ClientActionResult {
        let Ok(inner) = self.inner.lock() else {
            return ClientActionResult::NoCurrentClient;
        };
        let entry = match Self::client_entry(&inner, target, invoking_tty) {
            Ok(entry) => entry,
            Err(ClientActionResult::NoCurrentClient) if inner.clients.len() == 1 => {
                inner.clients.values().next().expect("one client present")
            }
            Err(result) => return result,
        };
        if let Ok(mut action) = entry.slot.action.lock() {
            *action = Some(ClientAction::SetSelection(data));
            let _ = entry.slot.wakeup.wake();
        }
        ClientActionResult::Queued
    }

    /// Set a client's terminal focus, reporting whether it changed.
    fn set_client_focused(&self, client: &str, focused: bool) -> bool {
        let Ok(mut inner) = self.inner.lock() else {
            return false;
        };
        let Some(entry) = inner
            .clients
            .values_mut()
            .find(|entry| entry.name == client)
        else {
            return false;
        };
        if entry.focused == focused {
            return false;
        }
        entry.focused = focused;
        entry.flags = entry
            .flag_state
            .display_flags_full(entry.identified, entry.control_mode, entry.focused);
        true
    }

    /// Record the theme a client's terminal reported, reporting whether it
    /// changed.
    fn set_client_theme(&self, client: &str, theme: &str) -> bool {
        let Ok(mut inner) = self.inner.lock() else {
            return false;
        };
        let Some(entry) = inner
            .clients
            .values_mut()
            .find(|entry| entry.name == client)
        else {
            return false;
        };
        if entry.theme == theme {
            return false;
        }
        entry.theme = theme.to_string();
        true
    }

    /// Stamp a client's activity time, tmux's `c->activity_time`.
    fn touch_client_activity(&self, client: &str, at: i64) {
        let Ok(mut inner) = self.inner.lock() else {
            return;
        };
        if let Some(entry) = inner.clients.values_mut().find(|entry| entry.name == client) {
            entry.activity_micros = at;
        }
    }

    /// Record the key table a client moved into.
    fn set_client_key_table(&self, client: &str, table: &str) {
        let Ok(mut inner) = self.inner.lock() else {
            return;
        };
        let Some(entry) = inner.clients.values_mut().find(|entry| entry.name == client) else {
            return;
        };
        if entry.key_table != table {
            entry.key_table = table.to_string();
        }
    }

    fn begin_clipboard_query(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        timeout: Duration,
    ) -> bool {
        let Ok(mut inner) = self.inner.lock() else {
            return false;
        };
        let id = match Self::client_id_for(&inner, target, invoking_tty) {
            Ok(id) => id,
            Err(ClientActionResult::NoCurrentClient) if inner.clients.len() == 1 => {
                *inner.clients.keys().next().expect("one client present")
            }
            Err(_) => return false,
        };
        let now = Instant::now();
        let entry = inner.clients.get_mut(&id).expect("selected client present");
        if entry
            .clipboard_query_at
            .is_some_and(|at| now.saturating_duration_since(at) < timeout)
        {
            return false;
        }
        entry.clipboard_query_at = Some(now);
        true
    }

    fn confirm_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        prompt: String,
        command: Vec<String>,
        confirm_key: u8,
        default_yes: bool,
        reply: Option<mpsc::Sender<PromptCompletion>>,
    ) -> ClientActionResult {
        let Ok(inner) = self.inner.lock() else {
            return ClientActionResult::NoCurrentClient;
        };
        let entry = match Self::client_entry(&inner, target, invoking_tty) {
            Ok(entry) => entry,
            Err(result) => return result,
        };
        if let Ok(mut action) = entry.slot.action.lock() {
            *action = Some(ClientAction::Confirm {
                prompt,
                command,
                confirm_key,
                default_yes,
                reply,
            });
            let _ = entry.slot.wakeup.wake();
        }
        ClientActionResult::Queued
    }

    fn switch_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        session_id: u32,
    ) -> ClientActionResult {
        let Ok(mut inner) = self.inner.lock() else {
            return ClientActionResult::NoCurrentClient;
        };
        let explicit = target.map(|target| target.strip_suffix(':').unwrap_or(target));
        let id = if let Some(target) = explicit {
            inner.clients.iter().find_map(|(id, entry)| {
                (entry.name == target
                    || entry
                        .name
                        .strip_prefix("/dev/")
                        .is_some_and(|tty| tty == target))
                .then_some(*id)
            })
        } else {
            invoking_tty.and_then(|tty| {
                inner
                    .clients
                    .iter()
                    .find_map(|(id, entry)| (entry.name == tty).then_some(*id))
            })
        };
        let Some(id) = id else {
            return if explicit.is_some() {
                ClientActionResult::TargetNotFound
            } else {
                ClientActionResult::NoCurrentClient
            };
        };
        let entry = inner
            .clients
            .get_mut(&id)
            .expect("selected client disappeared");
        entry.session_id = session_id;
        if let Ok(mut action) = entry.slot.action.lock() {
            *action = Some(ClientAction::Switch {
                session_id,
                destroyed: false,
            });
            let _ = entry.slot.wakeup.wake();
        }
        drop(inner);
        self.bump_generation();
        ClientActionResult::Queued
    }

    /// tmux's `server_destroy_session` client fan-out: every client on
    /// `from_session_id` moves to `to`, except that a client carrying
    /// `no-detach-on-destroy` falls back to `no_detach_to` when `to` is `None`.
    /// A client with no destination is left alone and exits on its own.
    fn reassign_session(
        &self,
        from_session_id: u32,
        to: Option<u32>,
        no_detach_to: Option<u32>,
    ) {
        let Ok(mut inner) = self.inner.lock() else {
            return;
        };
        for entry in inner
            .clients
            .values_mut()
            .filter(|entry| entry.session_id == from_session_id)
        {
            let Some(target) = to.or_else(|| {
                entry
                    .flag_state
                    .no_detach_on_destroy
                    .then_some(no_detach_to)
                    .flatten()
            }) else {
                continue;
            };
            entry.session_id = target;
            if let Ok(mut action) = entry.slot.action.lock() {
                *action = Some(ClientAction::Switch {
                    session_id: target,
                    destroyed: true,
                });
                let _ = entry.slot.wakeup.wake();
            }
        }
        drop(inner);
        self.bump_generation();
    }

    fn overlay_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        request: OverlayRequest,
        reply: Option<mpsc::Sender<PromptCompletion>>,
    ) -> ClientActionResult {
        let Ok(inner) = self.inner.lock() else {
            return ClientActionResult::NoCurrentClient;
        };
        let entry = match Self::client_entry(&inner, target, invoking_tty) {
            Ok(entry) => entry,
            Err(result) => return result,
        };
        if let Ok(mut action) = entry.slot.action.lock() {
            *action = Some(ClientAction::Overlay { request, reply });
            entry
                .slot
                .pending
                .fetch_or(RenderInvalidation::LAYOUT.bits(), Ordering::Release);
            let _ = entry.slot.wakeup.wake();
        }
        ClientActionResult::Queued
    }
}

impl ClientRenderAttachment {
    pub(crate) fn as_raw_fd(&self) -> RawFd {
        self.slot.wakeup.as_fd().as_raw_fd()
    }

    /// Clear readiness and take every reason published so far.
    ///
    /// Clearing before the atomic swap cannot lose a concurrent publication:
    /// a publication after the clear leaves the fd readable (at worst causing
    /// one harmless coalesced follow-up poll).
    pub(crate) fn take(&self) -> RenderInvalidation {
        let _ = self.slot.wakeup.clear();
        RenderInvalidation(self.slot.pending.swap(0, Ordering::AcqRel))
    }

    pub(crate) fn take_action(&self) -> Option<ClientAction> {
        self.slot.action.lock().ok()?.take()
    }

    pub(crate) fn take_messages(&self) -> Vec<ClientMessage> {
        self.slot
            .messages
            .lock()
            .map(|mut messages| messages.drain(..).collect())
            .unwrap_or_default()
    }

    pub(crate) fn update_size(&self, cols: u16, rows: u16) {
        if let Ok(mut inner) = self.registry.inner.lock() {
            if let Some(entry) = inner.clients.get_mut(&self.id) {
                entry.cols = cols;
                entry.rows = rows;
                entry.size_changed = true;
            }
        }
        self.registry.bump_generation();
    }

    pub(crate) fn mark_size_changed(&self) {
        if let Ok(mut inner) = self.registry.inner.lock() {
            if let Some(entry) = inner.clients.get_mut(&self.id) {
                entry.size_changed = true;
            }
        }
    }

    pub(crate) fn update_control_flags(&self, flags: String, flag_state: &ClientFlagState) {
        if let Ok(mut inner) = self.registry.inner.lock() {
            if let Some(entry) = inner.clients.get_mut(&self.id) {
                entry.ignore_size = flag_state.ignore_size;
                entry.read_only = flag_state.read_only;
                entry.flag_state = flag_state.clone();
                entry.flags = flags;
            }
        }
    }

    /// This client's registry name, as `#{client_name}` reports it.
    pub(crate) fn client_name(&self) -> String {
        self.registry
            .inner
            .lock()
            .ok()
            .and_then(|inner| inner.clients.get(&self.id).map(|entry| entry.name.clone()))
            .unwrap_or_default()
    }

    /// `refresh-client -f` values another client aimed at this one.
    pub(crate) fn take_flag_updates(&self) -> Vec<String> {
        self.slot
            .flag_updates
            .lock()
            .map(|mut pending| std::mem::take(&mut *pending))
            .unwrap_or_default()
    }

    pub(crate) fn update_terminal(&self, terminal: &ResolvedTerm) {
        if let Ok(mut inner) = self.registry.inner.lock() {
            if let Some(entry) = inner.clients.get_mut(&self.id) {
                entry.terminal = Some(terminal.clone());
            }
        }
    }

    pub(crate) fn update_session(&self, session_id: u32) {
        if let Ok(mut inner) = self.registry.inner.lock() {
            if let Some(entry) = inner.clients.get_mut(&self.id) {
                entry.session_id = session_id;
            }
        }
        self.registry.bump_generation();
    }
}

impl Drop for ClientRenderAttachment {
    fn drop(&mut self) {
        if let Ok(mut inner) = self.registry.inner.lock() {
            inner.clients.remove(&self.id);
            let peers = inner
                .clients
                .values()
                .map(|entry| Arc::clone(&entry.slot))
                .collect::<Vec<_>>();
            drop(inner);
            self.registry.bump_generation();
            for peer in peers {
                peer.pending
                    .fetch_or(RenderInvalidation::STATUS.bits(), Ordering::Release);
                let _ = peer.wakeup.wake();
            }
        }
    }
}

impl ClientPromptRegistry {
    fn new() -> Self {
        Self {
            inner: Mutex::new(ClientPromptRegistryState::default()),
            activity: AtomicU64::new(0),
        }
    }

    pub(crate) fn attach(
        self: &Arc<Self>,
        tty_name: String,
        client_pid: Option<i32>,
        _session_id: u32,
    ) -> io::Result<ClientPromptAttachment> {
        let wakeup = CurrentPlatform::new_output_wakeup()?;
        wakeup.clear()?;
        let slot = Arc::new(ClientPromptSlot {
            inner: Mutex::new(ClientPromptSlotState::default()),
            wakeup,
        });
        let activity = Arc::new(AtomicU64::new(self.next_activity()));
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| io::Error::other("client prompt registry poisoned"))?;
        let id = inner.next_id;
        inner.next_id = inner.next_id.wrapping_add(1);
        let name = if tty_name.is_empty() {
            format!("client-{}", client_pid.unwrap_or_default())
        } else {
            tty_name.clone()
        };
        inner.clients.insert(
            id,
            ClientPromptEntry {
                name,
                tty_name,
                slot: Arc::clone(&slot),
                activity: Arc::clone(&activity),
            },
        );
        Ok(ClientPromptAttachment {
            registry: Arc::clone(self),
            id,
            slot,
            activity,
        })
    }

    fn next_activity(&self) -> u64 {
        self.activity.fetch_add(1, Ordering::Relaxed)
    }

    pub(crate) fn request_command(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        args: Vec<String>,
        wait: bool,
    ) -> CommandPromptRequestResult {
        let completed = {
            let Ok(inner) = self.inner.lock() else {
                return CommandPromptRequestResult::NoCurrentClient;
            };
            let explicit = target.map(|target| target.strip_suffix(':').unwrap_or(target));
            let selected = if let Some(target) = explicit {
                inner.clients.values().find(|entry| {
                    entry.name == target
                        || entry.tty_name == target
                        || entry
                            .tty_name
                            .strip_prefix("/dev/")
                            .is_some_and(|tty| tty == target)
                })
            } else {
                invoking_tty
                    .and_then(|tty| inner.clients.values().find(|entry| entry.tty_name == tty))
                    .or_else(|| {
                        inner
                            .clients
                            .values()
                            .max_by_key(|entry| entry.activity.load(Ordering::Relaxed))
                    })
            };
            let Some(entry) = selected else {
                return if explicit.is_some() {
                    CommandPromptRequestResult::TargetNotFound
                } else {
                    CommandPromptRequestResult::NoCurrentClient
                };
            };
            let slot = Arc::clone(&entry.slot);
            let (reply, completed) = mpsc::channel();
            let Ok(mut state) = slot.inner.lock() else {
                return CommandPromptRequestResult::NoCurrentClient;
            };
            if state.active || state.queued_command.is_some() {
                return CommandPromptRequestResult::Busy;
            }
            state.queued_command = Some(QueuedCommandPrompt { args, reply });
            if slot.wakeup.wake().is_err() {
                state.queued_command = None;
                return CommandPromptRequestResult::NoCurrentClient;
            }
            completed
        };
        if !wait {
            return CommandPromptRequestResult::Queued;
        }
        match completed.recv() {
            Ok(Some(result)) => CommandPromptRequestResult::Completed(result),
            Ok(None) | Err(_) => CommandPromptRequestResult::Completed(PromptCompletion {
                stdout: String::new(),
                stderr: String::new(),
                exit: 0,
                inserted: false,
            }),
        }
    }
}

impl ClientPromptAttachment {
    pub(crate) fn as_raw_fd(&self) -> RawFd {
        self.slot.wakeup.as_fd().as_raw_fd()
    }

    pub(crate) fn take_command_prompt(&self) -> Option<ActiveCommandPrompt> {
        let _ = self.slot.wakeup.clear();
        let mut state = self.slot.inner.lock().ok()?;
        let queued = state.queued_command.take()?;
        state.active = true;
        Some(ActiveCommandPrompt {
            slot: Arc::clone(&self.slot),
            args: queued.args,
            reply: Some(queued.reply),
        })
    }

    pub(crate) fn note_activity(&self) {
        self.activity
            .store(self.registry.next_activity(), Ordering::Relaxed);
    }
}

impl Drop for ClientPromptAttachment {
    fn drop(&mut self) {
        if let Ok(mut inner) = self.registry.inner.lock() {
            inner.clients.remove(&self.id);
        }
        if let Ok(mut state) = self.slot.inner.lock() {
            if let Some(queued) = state.queued_command.take() {
                let _ = queued.reply.send(None);
            }
        }
    }
}

impl ActiveCommandPrompt {
    pub(crate) fn args(&self) -> &[String] {
        &self.args
    }

    pub(crate) fn complete(mut self, result: PromptCompletion) {
        if let Ok(mut state) = self.slot.inner.lock() {
            state.active = false;
        }
        if let Some(reply) = self.reply.take() {
            let _ = reply.send(Some(result));
        }
    }

    pub(crate) fn cancel(mut self) {
        if let Ok(mut state) = self.slot.inner.lock() {
            state.active = false;
        }
        if let Some(reply) = self.reply.take() {
            let _ = reply.send(None);
        }
    }
}

impl Drop for ActiveCommandPrompt {
    fn drop(&mut self) {
        if let Ok(mut state) = self.slot.inner.lock() {
            state.active = false;
        }
    }
}

impl Session {
    pub(crate) fn options<'a>(&'a self, globals: &'a GlobalOptions) -> OptionsView<'a> {
        OptionsView::two(&self.options, globals.session())
    }

    pub(crate) fn option_overrides(&self) -> &OptionSet {
        &self.options
    }

    pub(crate) fn option_overrides_mut(&mut self) -> &mut OptionSet {
        &mut self.options
    }

    /// A `list-sessions`-style summary line, matching real tmux's default
    /// `#{session_name}: #{session_windows} windows (created ...)` shape. The
    /// timestamp is volatile; the conformance harness normalizes it before
    /// comparing against stock tmux.
    pub fn summary(&self) -> String {
        // tmux always prints "windows" (plural), even for a single window.
        format!(
            "{}: {} windows (created {})",
            self.name,
            self.windows.len(),
            self.created,
        )
    }
}

impl Window {
    pub(crate) fn options<'a>(&'a self, globals: &'a GlobalOptions) -> OptionsView<'a> {
        OptionsView::two(&self.options, globals.window())
    }

    pub(crate) fn option_overrides(&self) -> &OptionSet {
        &self.options
    }

    pub(crate) fn option_overrides_mut(&mut self) -> &mut OptionSet {
        &mut self.options
    }

    pub(crate) fn pane_rect(&self, pane_id: u32) -> Option<PaneRect> {
        self.panes
            .iter()
            .find(|pane| pane.id == pane_id)
            .and_then(|pane| pane.floating)
            .or_else(|| self.layout.pane_rect(pane_id))
    }
}

impl PaneNode {
    pub(crate) fn options<'a>(
        &'a self,
        window: &'a Window,
        globals: &'a GlobalOptions,
    ) -> OptionsView<'a> {
        OptionsView::three(&self.options, &window.options, globals.window())
    }

    pub(crate) fn option_overrides(&self) -> &OptionSet {
        &self.options
    }

    pub(crate) fn option_overrides_mut(&mut self) -> &mut OptionSet {
        &mut self.options
    }
}

/// A resolved `target` (`session[:window]`): indices into the session/window
/// tree plus the active pane index of that window.
#[derive(Clone, Copy)]
pub struct Target {
    pub session: usize,
    pub window: usize,
    pub pane: usize,
}

/// Format tmux's `(created ...)` timestamp for "now" (e.g.
/// `Tue Jul  7 20:57:17 2026`). Best-effort; an empty string on failure (the
/// value is normalized away in conformance comparisons regardless).
pub fn created_stamp() -> String {
    // SAFETY: standard libc time/localtime/strftime dance. `localtime` returns a
    // pointer into a static buffer, used only until the next call; we copy out
    // immediately via strftime.
    unsafe {
        let t = libc::time(std::ptr::null_mut());
        let tm = libc::localtime(&t);
        if tm.is_null() {
            return String::new();
        }
        let mut buf = [0 as libc::c_char; 64];
        let n = libc::strftime(
            buf.as_mut_ptr(),
            buf.len(),
            c"%a %b %e %H:%M:%S %Y".as_ptr(),
            tm,
        );
        if n == 0 {
            return String::new();
        }
        let bytes: Vec<u8> = buf[..n].iter().map(|&c| c as u8).collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

/// The DSR reply announcing a theme to a pane.
fn theme_report(theme: &str) -> &'static [u8] {
    if theme == "dark" {
        b"\x1b[?997;1n"
    } else {
        b"\x1b[?997;2n"
    }
}

/// Current Unix time in seconds (tmux's `#{session_created}` unit).
pub fn now_epoch() -> i64 {
    // SAFETY: `time(NULL)` returns the current time and touches no memory.
    unsafe { libc::time(std::ptr::null_mut()) as i64 }
}

/// Current Unix time in microseconds, the resolution tmux keeps session
/// activity at (`struct timeval`) and compares with `timercmp`.
pub(crate) fn now_micros() -> i64 {
    let mut tv = libc::timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    // SAFETY: `gettimeofday` fills the caller-owned `timeval` and the null
    // timezone pointer is the documented way to skip the obsolete second arg.
    unsafe { libc::gettimeofday(&mut tv, std::ptr::null_mut()) };
    // `timeval`'s fields are already `i64` on the supported platforms.
    tv.tv_sec * 1_000_000 + tv.tv_usec
}

/// The whole server's state. Guarded by a mutex at the connection layer.
pub struct ServerState {
    /// True only before this explicitly launched hmux server has created its
    /// first session. An untargeted attach may consume this state by creating
    /// session 0; becoming empty later must not repeat that bootstrap behavior.
    initial_attach_pending: bool,
    pane_io_mode: PaneIoMode,
    sessions: Vec<Session>,
    /// Windows are owned once by the server and referenced through [`Winlink`].
    windows: BTreeMap<u32, Window>,
    /// Explicit tmux session groups, keyed by their synchronization identity.
    /// Group names outlive the session they were originally named after, and a
    /// one-member group remains grouped until its final session is destroyed.
    session_groups: BTreeMap<u32, String>,
    /// Set once a non-empty server becomes empty while `exit-empty` is on.
    shutdown_requested: bool,
    /// Client-registry generation the unattached sweep last ran against.
    lifecycle_generation: u64,
    /// Sessions a client just took, whose windows the next alert pass
    /// re-examines in full (tmux's `alerts_check_session`).
    alert_check_sessions: BTreeSet<u32>,
    /// The theme last announced to each pane subscribed with DECSET 2031.
    pane_theme_pushed: BTreeMap<u32, String>,
    /// Panes currently holding focus, so the focus hooks fire only on a change.
    focused_panes: BTreeSet<u32>,
    /// Event notifications raised by mutations since the command queue last
    /// drained them, in the order they happened. tmux's `notify_add` appends
    /// to the command queue; hmux collects them here because the mutation
    /// sites hold the state lock and the queue lives above it.
    pending_notifications: Vec<Notification>,
    /// Set while a mutation runs outside the command queue, so what it raises
    /// is marked for the server loop to dispatch.
    notifications_are_deferred: bool,
    /// Client name → (session, width, height) as of the last client-layer
    /// notification sweep.
    known_clients: BTreeMap<String, (u32, u16, u16)>,
    next_session_id: u32,
    next_link_set_id: u32,
    next_winlink_id: u64,
    next_window_id: u32,
    next_pane_id: u32,
    default_cols: u16,
    default_rows: u16,
    /// The global environment (`set-environment -g` / `show-environment -g`).
    /// A `BTreeMap` so `show-environment` (no variable) lists in sorted order,
    /// matching tmux.
    environment: BTreeMap<String, String>,
    /// Names marked by `set-environment -r` for removal from child processes.
    removed_environment: BTreeSet<String>,
    /// Names hidden by `set-environment -h`; omitted unless queried with
    /// `show-environment -h`.
    hidden_environment: BTreeSet<String>,
    /// Independent server, global-session, and global-window option tables.
    global_options: GlobalOptions,
    /// The paste-buffer stack, newest first (tmux's `#{buffer_name}` order in
    /// `list-buffers`). Each entry is `(name, data)`.
    buffers: Vec<(String, Vec<u8>)>,
    /// Creation/replacement time for each paste buffer, as an epoch value for
    /// `#{buffer_created}`.
    buffer_created: BTreeMap<String, i64>,
    /// Names created by the automatic (no `-b`) path. tmux's unnamed
    /// show/save lookup skips buffers created with an explicit name.
    automatic_buffers: BTreeSet<String>,
    /// Counter for the automatic `buffer0`, `buffer1`, … names tmux assigns.
    next_buffer_id: u32,
    /// The server's single marked pane (tmux's `marked_pane`), by stable pane id.
    /// `select-pane -m` toggles it, `-M` clears it; it backs `#{pane_marked}`,
    /// `#{pane_marked_set}`, `#{window_marked_flag}`, and `#{session_marked}`.
    marked_pane_id: Option<u32>,
    /// Mutable semantic key tables containing the supported defaults and all
    /// user-installed replacements.
    key_tables: BTreeMap<String, BTreeMap<KeyCode, KeyBinding>>,
    /// Configuration diagnostics waiting for a control-mode client.
    pending_config_errors: Vec<String>,
    /// Prompt history by tmux prompt type (`command`, `search`, `target`,
    /// `window-target`), oldest first.
    prompt_history: BTreeMap<String, Vec<String>>,
    message_log: Vec<MessageLogEntry>,
    background_jobs: Arc<BackgroundJobRegistry>,
    running_hooks: BTreeSet<String>,
    /// The `hook*` format variables published to the hook body currently
    /// executing; empty outside hook bodies. Like `command_session_id`, a
    /// transient interpreter hint installed around each hook command.
    hook_format_vars: Vec<(String, String)>,
    /// Stable session selected for the command currently executing. This is a
    /// transient interpreter hint, set while a client-scoped prompt template
    /// runs and restored before releasing the server-state lock.
    command_session_id: Option<u32>,
    /// Stable window selected for the command currently executing, when the
    /// default target names one — a hook body targeting a specific window.
    command_window_id: Option<u32>,
    /// Per-window pane selections for an `active-pane` client while one of its
    /// commands is executing. Missing entries fall back to the window's global
    /// active pane.
    command_active_panes: Option<BTreeMap<u32, u32>>,
    control_checkpoints: VecDeque<ControlCheckpoint>,
    next_control_checkpoint: u64,
    pane_alert_seen: BTreeMap<u32, (u64, u64)>,
    window_last_activity: BTreeMap<u32, std::time::Instant>,
    silence_alerted: BTreeSet<u32>,
    client_prompts: Arc<ClientPromptRegistry>,
    client_renders: Arc<ClientRenderRegistry>,
    wait_registry: Arc<WaitRegistry>,
    /// No-client format jobs, corresponding to tmux's process-global job tree.
    /// Current native consumers use client-owned status caches; this remains a
    /// distinct owner for no-client format contexts as those are implemented.
    #[allow(dead_code)]
    format_jobs: Arc<super::status::FormatJobRegistry>,
}

impl ServerState {
    /// Build an empty server. It remains available for the first client;
    /// `exit-empty=on` applies only after a non-empty server becomes empty.
    pub fn empty() -> ServerState {
        let client_renders = Arc::new(ClientRenderRegistry::new());
        let mut state = ServerState {
            initial_attach_pending: true,
            pane_io_mode: default_pane_io_mode(),
            sessions: Vec::new(),
            windows: BTreeMap::new(),
            session_groups: BTreeMap::new(),
            shutdown_requested: false,
            lifecycle_generation: 0,
            alert_check_sessions: BTreeSet::new(),
            focused_panes: BTreeSet::new(),
            pane_theme_pushed: BTreeMap::new(),
            pending_notifications: Vec::new(),
            notifications_are_deferred: false,
            known_clients: BTreeMap::new(),
            next_session_id: 0,
            next_link_set_id: 0,
            next_winlink_id: 0,
            next_window_id: 0,
            next_pane_id: 0,
            default_cols: 80,
            default_rows: 24,
            environment: BTreeMap::new(),
            removed_environment: BTreeSet::new(),
            hidden_environment: BTreeSet::new(),
            global_options: GlobalOptions::new(),
            buffers: Vec::new(),
            buffer_created: BTreeMap::new(),
            automatic_buffers: BTreeSet::new(),
            next_buffer_id: 0,
            marked_pane_id: None,
            key_tables: BTreeMap::new(),
            pending_config_errors: Vec::new(),
            prompt_history: BTreeMap::new(),
            message_log: Vec::new(),
            background_jobs: Arc::new(BackgroundJobRegistry::default()),
            running_hooks: BTreeSet::new(),
            hook_format_vars: Vec::new(),
            command_session_id: None,
            command_window_id: None,
            command_active_panes: None,
            control_checkpoints: VecDeque::new(),
            next_control_checkpoint: 0,
            pane_alert_seen: BTreeMap::new(),
            window_last_activity: BTreeMap::new(),
            silence_alerted: BTreeSet::new(),
            client_prompts: Arc::new(ClientPromptRegistry::new()),
            format_jobs: Arc::new(super::status::FormatJobRegistry::new(Arc::clone(
                &client_renders,
            ))),
            client_renders,
            wait_registry: Arc::new(WaitRegistry::default()),
        };
        state.install_default_key_bindings();
        state
    }

    #[cfg(test)]
    pub(crate) fn with_test_session() -> io::Result<ServerState> {
        let mut state = ServerState::empty();
        state.create_session("0", PaneSpec::Inert)?;
        Ok(state)
    }

    pub(crate) fn initial_attach_pending(&self) -> bool {
        self.initial_attach_pending
    }

    pub(crate) fn set_pane_io_mode(&mut self, mode: PaneIoMode) {
        self.pane_io_mode = mode;
    }

    pub(crate) fn take_event_pane_ios(&mut self) -> Vec<(u64, PaneIo)> {
        self.windows
            .values_mut()
            .flat_map(|window| window.panes.iter_mut())
            .filter_map(|node| {
                node.pane
                    .take_event_io()
                    .map(|pane_io| (node.pane.runtime_id(), pane_io))
            })
            .collect()
    }

    pub(crate) fn pane_runtime_ids(&self) -> BTreeSet<u64> {
        self.windows
            .values()
            .flat_map(|window| window.panes.iter())
            .map(|node| node.pane.runtime_id())
            .collect()
    }


    fn install_default_key_bindings(&mut self) {
        const DEFAULTS: &[(&str, &str, &[&str])] = &[
            (
                "root",
                "MouseDown1Pane",
                &["select-pane", "-t", "=", ";", "send-keys", "-M"],
            ),
            ("root", "MouseDrag1Pane", &["copy-mode", "-M"]),
            ("root", "WheelUpPane", &["copy-mode", "-e"]),
            (
                "root",
                "DoubleClick1Pane",
                &[
                    "copy-mode",
                    "-H",
                    ";",
                    "send-keys",
                    "-X",
                    "select-word",
                    ";",
                    "run-shell",
                    "-d",
                    "0.3",
                    ";",
                    "send-keys",
                    "-X",
                    "copy-pipe-and-cancel",
                ],
            ),
            (
                "root",
                "TripleClick1Pane",
                &[
                    "copy-mode",
                    "-H",
                    ";",
                    "send-keys",
                    "-X",
                    "select-line",
                    ";",
                    "run-shell",
                    "-d",
                    "0.3",
                    ";",
                    "send-keys",
                    "-X",
                    "copy-pipe-and-cancel",
                ],
            ),
            ("root", "MouseDown1ScrollbarUp", &["copy-mode", "-u"]),
            ("root", "MouseDown1ScrollbarDown", &["copy-mode", "-d"]),
            ("root", "MouseDrag1ScrollbarSlider", &["copy-mode", "-S"]),
            ("prefix", "C-b", &["send-prefix"]),
            ("prefix", "C-z", &["suspend-client"]),
            ("prefix", "d", &["detach-client"]),
            ("prefix", "c", &["new-window"]),
            ("prefix", "\"", &["split-window"]),
            ("prefix", "%", &["split-window", "-h"]),
            ("prefix", "n", &["next-window"]),
            ("prefix", "p", &["previous-window"]),
            ("prefix", "l", &["last-window"]),
            ("prefix", ":", &["command-prompt"]),
            ("prefix", "o", &["select-pane", "-t", ":.+"]),
            ("prefix", "[", &["copy-mode"]),
            ("prefix", "PPage", &["copy-mode", "-u"]),
            ("prefix", "Up", &["select-pane", "-U"]),
            ("prefix", "Down", &["select-pane", "-D"]),
            ("prefix", "Left", &["select-pane", "-L"]),
            ("prefix", "Right", &["select-pane", "-R"]),
            (
                "prefix",
                "&",
                &[
                    "confirm-before",
                    "-p",
                    "kill-window #W? (y/n)",
                    "kill-window",
                ],
            ),
            (
                "prefix",
                "x",
                &["confirm-before", "-p", "kill-pane #P? (y/n)", "kill-pane"],
            ),
            ("copy-mode", "PPage", &["send-keys", "-X", "page-up"]),
            ("copy-mode", "NPage", &["send-keys", "-X", "page-down"]),
            ("copy-mode", "Up", &["send-keys", "-X", "cursor-up"]),
            ("copy-mode", "Down", &["send-keys", "-X", "cursor-down"]),
            ("copy-mode", "q", &["send-keys", "-X", "cancel"]),
            ("copy-mode", "Escape", &["send-keys", "-X", "cancel"]),
            ("copy-mode", "C-c", &["send-keys", "-X", "cancel"]),
            ("copy-mode-vi", "PPage", &["send-keys", "-X", "page-up"]),
            ("copy-mode-vi", "NPage", &["send-keys", "-X", "page-down"]),
            ("copy-mode-vi", "Up", &["send-keys", "-X", "cursor-up"]),
            ("copy-mode-vi", "Down", &["send-keys", "-X", "cursor-down"]),
            ("copy-mode-vi", "k", &["send-keys", "-X", "cursor-up"]),
            ("copy-mode-vi", "j", &["send-keys", "-X", "cursor-down"]),
            ("copy-mode-vi", "g", &["send-keys", "-X", "history-top"]),
            ("copy-mode-vi", "G", &["send-keys", "-X", "history-bottom"]),
            ("copy-mode-vi", "q", &["send-keys", "-X", "cancel"]),
            (
                "copy-mode-vi",
                "Escape",
                &["send-keys", "-X", "clear-selection"],
            ),
            (
                "copy-mode-vi",
                "Enter",
                &["send-keys", "-X", "copy-pipe-and-cancel"],
            ),
            ("copy-mode-vi", "C-c", &["send-keys", "-X", "cancel"]),
            (
                "copy-mode-vi",
                "Space",
                &["send-keys", "-X", "begin-selection"],
            ),
            ("copy-mode-vi", "$", &["send-keys", "-X", "end-of-line"]),
            ("copy-mode-vi", "0", &["send-keys", "-X", "start-of-line"]),
            ("copy-mode-vi", "h", &["send-keys", "-X", "cursor-left"]),
            ("copy-mode-vi", "l", &["send-keys", "-X", "cursor-right"]),
            ("copy-mode-vi", "w", &["send-keys", "-X", "next-word"]),
            ("copy-mode-vi", "b", &["send-keys", "-X", "previous-word"]),
            ("copy-mode-vi", "e", &["send-keys", "-X", "next-word-end"]),
            ("copy-mode-vi", "n", &["send-keys", "-X", "search-again"]),
            ("copy-mode-vi", "N", &["send-keys", "-X", "search-reverse"]),
            ("copy-mode-vi", "o", &["send-keys", "-X", "other-end"]),
            (
                "copy-mode-vi",
                "v",
                &["send-keys", "-X", "rectangle-toggle"],
            ),
            ("copy-mode-vi", "V", &["send-keys", "-X", "select-line"]),
            (
                "copy-mode-vi",
                "0x5e",
                &["send-keys", "-X", "back-to-indentation"],
            ),
            (
                "copy-mode-vi",
                "%",
                &["send-keys", "-X", "next-matching-bracket"],
            ),
            ("copy-mode-vi", "H", &["send-keys", "-X", "top-line"]),
            ("copy-mode-vi", "M", &["send-keys", "-X", "middle-line"]),
            ("copy-mode-vi", "L", &["send-keys", "-X", "bottom-line"]),
            (
                "copy-mode-vi",
                "r",
                &["send-keys", "-X", "refresh-from-pane"],
            ),
            ("copy-mode-vi", "C-d", &["send-keys", "-X", "halfpage-down"]),
            ("copy-mode-vi", "C-u", &["send-keys", "-X", "halfpage-up"]),
            ("copy-mode-vi", "C-f", &["send-keys", "-X", "page-down"]),
            ("copy-mode-vi", "C-b", &["send-keys", "-X", "page-up"]),
            ("copy-mode-vi", "C-e", &["send-keys", "-X", "scroll-down"]),
            ("copy-mode-vi", "C-y", &["send-keys", "-X", "scroll-up"]),
            (
                "copy-mode",
                "C-Space",
                &["send-keys", "-X", "begin-selection"],
            ),
            ("copy-mode", "C-a", &["send-keys", "-X", "start-of-line"]),
            ("copy-mode", "C-[", &["send-keys", "-X", "cancel"]),
            ("copy-mode", "C-e", &["send-keys", "-X", "end-of-line"]),
            ("copy-mode", "C-f", &["send-keys", "-X", "cursor-right"]),
            ("copy-mode", "C-b", &["send-keys", "-X", "cursor-left"]),
            ("copy-mode", "C-g", &["send-keys", "-X", "clear-selection"]),
            (
                "copy-mode",
                "C-l",
                &["send-keys", "-X", "recentre-top-bottom"],
            ),
            ("copy-mode", "C-n", &["send-keys", "-X", "cursor-down"]),
            ("copy-mode", "C-p", &["send-keys", "-X", "cursor-up"]),
            ("copy-mode", "C-v", &["send-keys", "-X", "page-down"]),
            ("copy-mode", "Space", &["send-keys", "-X", "page-down"]),
            ("copy-mode", "n", &["send-keys", "-X", "search-again"]),
            ("copy-mode", "N", &["send-keys", "-X", "search-reverse"]),
            ("copy-mode", "R", &["send-keys", "-X", "rectangle-toggle"]),
            ("copy-mode", "r", &["send-keys", "-X", "refresh-from-pane"]),
            ("copy-mode", "Home", &["send-keys", "-X", "start-of-line"]),
            ("copy-mode", "End", &["send-keys", "-X", "end-of-line"]),
            (
                "copy-mode",
                "MouseDrag1Pane",
                &["select-pane", ";", "send-keys", "-X", "begin-selection"],
            ),
            (
                "copy-mode",
                "MouseDragEnd1Pane",
                &["send-keys", "-X", "copy-pipe-and-cancel"],
            ),
            (
                "copy-mode",
                "WheelUpPane",
                &["select-pane", ";", "send-keys", "-N5", "-X", "scroll-up"],
            ),
            (
                "copy-mode",
                "WheelDownPane",
                &["select-pane", ";", "send-keys", "-N5", "-X", "scroll-down"],
            ),
            (
                "copy-mode",
                "C-r",
                &[
                    "command-prompt",
                    "-T",
                    "search",
                    "-i",
                    "-p",
                    "(search up)",
                    "-I",
                    "#{pane_search_string}",
                    "send-keys -X search-backward-incremental -- '%%'",
                ],
            ),
            (
                "copy-mode",
                "C-s",
                &[
                    "command-prompt",
                    "-T",
                    "search",
                    "-i",
                    "-p",
                    "(search down)",
                    "-I",
                    "#{pane_search_string}",
                    "send-keys -X search-forward-incremental -- '%%'",
                ],
            ),
            (
                "copy-mode",
                "f",
                &[
                    "command-prompt",
                    "-1",
                    "-p",
                    "(jump forward)",
                    "send-keys -X jump-forward -- '%%'",
                ],
            ),
            (
                "copy-mode",
                "F",
                &[
                    "command-prompt",
                    "-1",
                    "-p",
                    "(jump backward)",
                    "send-keys -X jump-backward -- '%%'",
                ],
            ),
            (
                "copy-mode",
                "t",
                &[
                    "command-prompt",
                    "-1",
                    "-p",
                    "(jump to forward)",
                    "send-keys -X jump-to-forward -- '%%'",
                ],
            ),
            (
                "copy-mode",
                "T",
                &[
                    "command-prompt",
                    "-1",
                    "-p",
                    "(jump to backward)",
                    "send-keys -X jump-to-backward -- '%%'",
                ],
            ),
            ("copy-mode", ",", &["send-keys", "-X", "jump-reverse"]),
            ("copy-mode", ";", &["send-keys", "-X", "jump-again"]),
            ("copy-mode", "X", &["send-keys", "-X", "set-mark"]),
            ("copy-mode", "M-x", &["send-keys", "-X", "jump-to-mark"]),
            ("copy-mode", "M-b", &["send-keys", "-X", "previous-word"]),
            ("copy-mode", "M-f", &["send-keys", "-X", "next-word-end"]),
            (
                "copy-mode",
                "M-l",
                &["send-keys", "-X", "cursor-centre-horizontal"],
            ),
            (
                "copy-mode",
                "M-m",
                &["send-keys", "-X", "back-to-indentation"],
            ),
            ("copy-mode", "M-v", &["send-keys", "-X", "page-up"]),
            ("copy-mode", "M-Up", &["send-keys", "-X", "halfpage-up"]),
            ("copy-mode", "M-Down", &["send-keys", "-X", "halfpage-down"]),
            ("copy-mode", "C-Up", &["send-keys", "-X", "scroll-up"]),
            ("copy-mode", "C-Down", &["send-keys", "-X", "scroll-down"]),
            ("copy-mode", "M-<", &["send-keys", "-X", "history-top"]),
            ("copy-mode", "M->", &["send-keys", "-X", "history-bottom"]),
            ("copy-mode", "P", &["send-keys", "-X", "toggle-position"]),
            (
                "copy-mode",
                "g",
                &[
                    "command-prompt",
                    "-p",
                    "(goto line)",
                    "send-keys -X goto-line -- '%%'",
                ],
            ),
            ("copy-mode", "MouseDown1Pane", &["select-pane"]),
            (
                "copy-mode",
                "DoubleClick1Pane",
                &[
                    "select-pane",
                    ";",
                    "send-keys",
                    "-X",
                    "select-word",
                    ";",
                    "run-shell",
                    "-d",
                    "0.3",
                    ";",
                    "send-keys",
                    "-X",
                    "copy-pipe-and-cancel",
                ],
            ),
            (
                "copy-mode",
                "TripleClick1Pane",
                &[
                    "select-pane",
                    ";",
                    "send-keys",
                    "-X",
                    "select-line",
                    ";",
                    "run-shell",
                    "-d",
                    "0.3",
                    ";",
                    "send-keys",
                    "-X",
                    "copy-pipe-and-cancel",
                ],
            ),
            (
                "copy-mode-vi",
                "/",
                &[
                    "command-prompt",
                    "-p",
                    "(search down)",
                    "send-keys -X search-forward -- '%%'",
                ],
            ),
            (
                "copy-mode-vi",
                "?",
                &[
                    "command-prompt",
                    "-p",
                    "(search up)",
                    "send-keys -X search-backward -- '%%'",
                ],
            ),
            (
                "copy-mode-vi",
                "*",
                &[
                    "send-keys",
                    "-F",
                    "-X",
                    "search-forward",
                    "--",
                    "#{copy_cursor_word}",
                ],
            ),
            (
                "copy-mode-vi",
                "#",
                &[
                    "send-keys",
                    "-F",
                    "-X",
                    "search-backward",
                    "--",
                    "#{copy_cursor_word}",
                ],
            ),
            (
                "copy-mode-vi",
                "C-[",
                &["send-keys", "-X", "clear-selection"],
            ),
            ("copy-mode-vi", "C-h", &["send-keys", "-X", "cursor-left"]),
            (
                "copy-mode-vi",
                ":",
                &[
                    "command-prompt",
                    "-p",
                    "(goto line)",
                    "send-keys -X goto-line -- '%%'",
                ],
            ),
            (
                "copy-mode-vi",
                "f",
                &[
                    "command-prompt",
                    "-1",
                    "-p",
                    "(jump forward)",
                    "send-keys -X jump-forward -- '%%'",
                ],
            ),
            (
                "copy-mode-vi",
                "F",
                &[
                    "command-prompt",
                    "-1",
                    "-p",
                    "(jump backward)",
                    "send-keys -X jump-backward -- '%%'",
                ],
            ),
            (
                "copy-mode-vi",
                "t",
                &[
                    "command-prompt",
                    "-1",
                    "-p",
                    "(jump to forward)",
                    "send-keys -X jump-to-forward -- '%%'",
                ],
            ),
            (
                "copy-mode-vi",
                "T",
                &[
                    "command-prompt",
                    "-1",
                    "-p",
                    "(jump to backward)",
                    "send-keys -X jump-to-backward -- '%%'",
                ],
            ),
            ("copy-mode-vi", ",", &["send-keys", "-X", "jump-reverse"]),
            ("copy-mode-vi", ";", &["send-keys", "-X", "jump-again"]),
            ("copy-mode-vi", "X", &["send-keys", "-X", "set-mark"]),
            ("copy-mode-vi", "M-x", &["send-keys", "-X", "jump-to-mark"]),
            ("copy-mode-vi", "P", &["send-keys", "-X", "toggle-position"]),
            ("copy-mode-vi", "z", &["send-keys", "-X", "scroll-middle"]),
            ("copy-mode-vi", "B", &["send-keys", "-X", "previous-space"]),
            ("copy-mode-vi", "W", &["send-keys", "-X", "next-space"]),
            ("copy-mode-vi", "E", &["send-keys", "-X", "next-space-end"]),
            ("copy-mode-vi", "J", &["send-keys", "-X", "scroll-down"]),
            ("copy-mode-vi", "K", &["send-keys", "-X", "scroll-up"]),
            (
                "copy-mode-vi",
                "C-v",
                &["send-keys", "-X", "rectangle-toggle"],
            ),
            ("copy-mode-vi", "Left", &["send-keys", "-X", "cursor-left"]),
            (
                "copy-mode-vi",
                "Right",
                &["send-keys", "-X", "cursor-right"],
            ),
            ("copy-mode-vi", "C-Up", &["send-keys", "-X", "scroll-up"]),
            (
                "copy-mode-vi",
                "C-Down",
                &["send-keys", "-X", "scroll-down"],
            ),
            (
                "copy-mode-vi",
                "Home",
                &["send-keys", "-X", "start-of-line"],
            ),
            ("copy-mode-vi", "End", &["send-keys", "-X", "end-of-line"]),
            (
                "copy-mode-vi",
                "BSpace",
                &["send-keys", "-X", "cursor-left"],
            ),
            (
                "copy-mode-vi",
                "{",
                &["send-keys", "-X", "previous-paragraph"],
            ),
            ("copy-mode-vi", "}", &["send-keys", "-X", "next-paragraph"]),
            (
                "copy-mode-vi",
                "C-j",
                &["send-keys", "-X", "copy-pipe-and-cancel"],
            ),
            (
                "copy-mode-vi",
                "D",
                &["send-keys", "-X", "copy-pipe-end-of-line-and-cancel"],
            ),
            (
                "copy-mode-vi",
                "A",
                &["send-keys", "-X", "append-selection-and-cancel"],
            ),
            ("copy-mode", "Left", &["send-keys", "-X", "cursor-left"]),
            ("copy-mode", "Right", &["send-keys", "-X", "cursor-right"]),
            ("copy-mode", "M-R", &["send-keys", "-X", "top-line"]),
            ("copy-mode", "M-r", &["send-keys", "-X", "middle-line"]),
            (
                "copy-mode",
                "C-M-b",
                &["send-keys", "-X", "previous-matching-bracket"],
            ),
            (
                "copy-mode",
                "C-M-f",
                &["send-keys", "-X", "next-matching-bracket"],
            ),
            (
                "copy-mode",
                "M-{",
                &["send-keys", "-X", "previous-paragraph"],
            ),
            ("copy-mode", "M-}", &["send-keys", "-X", "next-paragraph"]),
            (
                "copy-mode",
                "C-k",
                &["send-keys", "-X", "copy-pipe-end-of-line-and-cancel"],
            ),
            (
                "copy-mode",
                "C-w",
                &["send-keys", "-X", "copy-pipe-and-cancel"],
            ),
            (
                "copy-mode",
                "M-w",
                &["send-keys", "-X", "copy-pipe-and-cancel"],
            ),
            (
                "copy-mode-vi",
                "MouseDrag1Pane",
                &["select-pane", ";", "send-keys", "-X", "begin-selection"],
            ),
            ("copy-mode-vi", "MouseDown1Pane", &["select-pane"]),
            (
                "copy-mode-vi",
                "DoubleClick1Pane",
                &[
                    "select-pane",
                    ";",
                    "send-keys",
                    "-X",
                    "select-word",
                    ";",
                    "run-shell",
                    "-d",
                    "0.3",
                    ";",
                    "send-keys",
                    "-X",
                    "copy-pipe-and-cancel",
                ],
            ),
            (
                "copy-mode-vi",
                "TripleClick1Pane",
                &[
                    "select-pane",
                    ";",
                    "send-keys",
                    "-X",
                    "select-line",
                    ";",
                    "run-shell",
                    "-d",
                    "0.3",
                    ";",
                    "send-keys",
                    "-X",
                    "copy-pipe-and-cancel",
                ],
            ),
            (
                "copy-mode-vi",
                "MouseDragEnd1Pane",
                &["send-keys", "-X", "copy-pipe-and-cancel"],
            ),
            (
                "copy-mode-vi",
                "WheelUpPane",
                &["select-pane", ";", "send-keys", "-N5", "-X", "scroll-up"],
            ),
            (
                "copy-mode-vi",
                "WheelDownPane",
                &["select-pane", ";", "send-keys", "-N5", "-X", "scroll-down"],
            ),
        ];
        for &(table, name, command) in DEFAULTS {
            let key =
                parse_key_name(name).unwrap_or_else(|| panic!("invalid default key name: {name}"));
            self.bind_key(
                table,
                key,
                command.iter().map(|word| (*word).to_string()).collect(),
                false,
                None,
            );
        }
        for index in 0..=9 {
            let key = parse_key_name(&index.to_string()).expect("digit key");
            self.bind_key(
                "prefix",
                key,
                vec![
                    "select-window".to_string(),
                    "-t".to_string(),
                    format!(":{index}"),
                ],
                false,
                None,
            );
            if index != 0 {
                let repeat = vec![
                    "command-prompt".to_string(),
                    "-N".to_string(),
                    "-p".to_string(),
                    "(repeat)".to_string(),
                    "-I".to_string(),
                    index.to_string(),
                    "send-keys -N '%%'".to_string(),
                ];
                self.bind_key("copy-mode-vi", key, repeat.clone(), false, None);
                let meta =
                    parse_key_name(&format!("M-{index}")).expect("copy-mode emacs repeat key");
                self.bind_key("copy-mode", meta, repeat, false, None);
            }
        }
    }

    pub fn sessions(&self) -> &[Session] {
        &self.sessions
    }

    pub(crate) fn session_mut(&mut self, session: usize) -> &mut Session {
        &mut self.sessions[session]
    }

    pub(crate) fn window(&self, session: usize, window: usize) -> &Window {
        let id = self.sessions[session].windows[window].id;
        self.windows
            .get(&id)
            .expect("every winlink references a live window")
    }

    pub(crate) fn printable_window_flags(
        &self,
        session: &Session,
        position: usize,
        escape_activity: bool,
    ) -> String {
        let Some(link) = session.windows.get(position) else {
            return String::new();
        };
        let mut flags = String::new();
        if link.alert_flags & ALERT_ACTIVITY != 0 {
            flags.push('#');
            if escape_activity {
                flags.push('#');
            }
        }
        if link.alert_flags & ALERT_BELL != 0 {
            flags.push('!');
        }
        if link.alert_flags & ALERT_SILENCE != 0 {
            flags.push('~');
        }
        if position == session.active {
            flags.push('*');
        }
        if session.last_active == Some(position) {
            flags.push('-');
        }
        if self.marked_pane_id.is_some_and(|marked| {
            self.window_for_link(link)
                .panes
                .iter()
                .any(|pane| pane.id == marked)
        }) {
            flags.push('M');
        }
        if self.window_for_link(link).zoomed {
            flags.push('Z');
        }
        flags
    }

    pub(crate) fn session_alert(&self, session: &Session) -> String {
        let mut alerts = String::new();
        let combined = session
            .windows
            .iter()
            .fold(0u8, |flags, link| flags | link.alert_flags);
        if combined & ALERT_ACTIVITY != 0 {
            alerts.push('#');
        }
        if combined & ALERT_BELL != 0 {
            alerts.push('!');
        }
        if combined & ALERT_SILENCE != 0 {
            alerts.push('~');
        }
        alerts
    }

    pub(crate) fn session_alerts(&self, session: &Session) -> String {
        session
            .windows
            .iter()
            .filter(|link| link.alert_flags != 0)
            .map(|link| {
                let mut alert = link.index.to_string();
                if link.alert_flags & ALERT_ACTIVITY != 0 {
                    alert.push('#');
                }
                if link.alert_flags & ALERT_BELL != 0 {
                    alert.push('!');
                }
                if link.alert_flags & ALERT_SILENCE != 0 {
                    alert.push('~');
                }
                alert
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    pub(crate) fn window_mut(&mut self, session: usize, window: usize) -> &mut Window {
        let id = self.sessions[session].windows[window].id;
        self.windows
            .get_mut(&id)
            .expect("every winlink references a live window")
    }

    pub(crate) fn window_for_link(&self, link: &Winlink) -> &Window {
        self.windows
            .get(&link.id)
            .expect("every winlink references a live window")
    }

    pub(crate) fn session_window(&self, session: &Session, window: usize) -> &Window {
        self.window_for_link(&session.windows[window])
    }

    pub(crate) fn all_windows(&self) -> impl Iterator<Item = &Window> {
        self.windows.values()
    }

    pub(crate) fn window_reference_count(&self, window_id: u32) -> usize {
        self.sessions
            .iter()
            .map(|session| {
                session
                    .windows
                    .iter()
                    .filter(|link| link.id == window_id)
                    .count()
            })
            .sum()
    }

    pub(crate) fn window_linked_session_count(&self, window_id: u32) -> usize {
        self.sessions
            .iter()
            .filter(|session| session.windows.iter().any(|link| link.id == window_id))
            .map(|session| session.link_set_id)
            .collect::<BTreeSet<_>>()
            .len()
    }

    pub(crate) fn window_linked_session_list(&self, window_id: u32) -> String {
        self.sessions
            .iter()
            .flat_map(|session| {
                session
                    .windows
                    .iter()
                    .filter(move |link| link.id == window_id)
                    .map(move |_| session.name.as_str())
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    pub(crate) fn window_active_session_list(&self, window_id: u32) -> Vec<&str> {
        self.sessions
            .iter()
            .filter_map(|session| {
                session
                    .windows
                    .get(session.active)
                    .is_some_and(|link| link.id == window_id)
                    .then_some(session.name.as_str())
            })
            .collect()
    }

    pub(crate) fn is_grouped(&self, session: &Session) -> bool {
        self.session_groups.contains_key(&session.link_set_id)
    }

    pub(crate) fn session_group_name(&self, session: &Session) -> Option<&str> {
        self.session_groups
            .get(&session.link_set_id)
            .map(String::as_str)
    }

    pub(crate) fn session_group_size(&self, session: &Session) -> usize {
        self.sessions
            .iter()
            .filter(|member| member.link_set_id == session.link_set_id)
            .count()
    }

    pub(crate) fn session_attached_client_names(&self, session: &Session) -> Vec<String> {
        self.client_renders
            .names_for_sessions(&BTreeSet::from([session.id]))
    }

    pub(crate) fn session_group_attached_client_names(&self, session: &Session) -> Vec<String> {
        if !self.is_grouped(session) {
            return Vec::new();
        }
        let ids = self
            .sessions
            .iter()
            .filter_map(|member| (member.link_set_id == session.link_set_id).then_some(member.id))
            .collect();
        self.client_renders.names_for_sessions(&ids)
    }

    fn links_mut(&mut self, session: usize) -> &mut Vec<Winlink> {
        &mut self.sessions[session].windows
    }

    fn refresh_last_window(session: &mut Session) {
        let active_id = session.windows.get(session.active).map(|link| link.link_id);
        let live = session
            .windows
            .iter()
            .map(|link| link.link_id)
            .collect::<BTreeSet<_>>();
        session
            .last_windows
            .retain(|id| live.contains(id) && Some(*id) != active_id);
        let mut seen = BTreeSet::new();
        session.last_windows.retain(|id| seen.insert(*id));
        session.last_active = session
            .last_windows
            .first()
            .and_then(|id| session.windows.iter().position(|link| link.link_id == *id));
    }

    fn select_window_position(session: &mut Session, position: usize) {
        if position >= session.windows.len() {
            return;
        }
        session.windows[position].alert_flags = 0;
        if position == session.active {
            return;
        }
        let old_id = session.windows[session.active].link_id;
        let new_id = session.windows[position].link_id;
        session
            .last_windows
            .retain(|id| *id != new_id && *id != old_id);
        session.last_windows.insert(0, old_id);
        session.active = position;
        Self::refresh_last_window(session);
    }

    /// [`Self::select_window_position`] plus tmux's `session-window-changed`
    /// notification, which only fires when the active window really moved.
    fn select_session_window(&mut self, session_pos: usize, position: usize) {
        let session = &mut self.sessions[session_pos];
        let previous = session.active;
        Self::select_window_position(session, position);
        if session.active != previous {
            let session_id = session.id;
            self.notify_session("session-window-changed", session_id);
        }
    }

    fn install_links_by_index(session: &mut Session, links: Vec<Winlink>) {
        let active_index = session.windows.get(session.active).map(|link| link.index);
        let mut stack_indices = session
            .last_windows
            .iter()
            .filter_map(|id| {
                session
                    .windows
                    .iter()
                    .find(|link| link.link_id == *id)
                    .map(|link| link.index)
            })
            .collect::<Vec<_>>();
        if stack_indices.is_empty() {
            if let Some(index) = session
                .last_active
                .and_then(|position| session.windows.get(position))
                .map(|link| link.index)
            {
                stack_indices.push(index);
            }
        }
        session.windows = links;
        if session.windows.is_empty() {
            session.active = 0;
            session.last_active = None;
            session.last_windows.clear();
            return;
        }
        session.active = active_index
            .and_then(|index| session.windows.iter().position(|link| link.index == index))
            .or_else(|| {
                stack_indices
                    .iter()
                    .find_map(|index| session.windows.iter().position(|link| link.index == *index))
            })
            .or_else(|| {
                active_index.and_then(|index| {
                    session
                        .windows
                        .iter()
                        .enumerate()
                        .rfind(|(_, link)| link.index < index)
                        .map(|(position, _)| position)
                })
            })
            .unwrap_or(0);
        let active_id = session.windows[session.active].link_id;
        session.last_windows = stack_indices
            .into_iter()
            .filter_map(|index| {
                session
                    .windows
                    .iter()
                    .find(|link| link.index == index)
                    .map(|link| link.link_id)
            })
            .filter(|id| *id != active_id)
            .collect();
        Self::refresh_last_window(session);
    }

    /// Synchronize every other member from `source`, preserving each member's
    /// current and previous window by index. This mirrors tmux's
    /// `session_group_synchronize_from` rather than sharing one collection.
    fn synchronize_group_from(&mut self, source: usize) {
        if !self.is_grouped(&self.sessions[source]) {
            return;
        }
        let source_id = self.sessions[source].id;
        let link_set_id = self.sessions[source].link_set_id;
        let links = self.sessions[source].windows.clone();
        for member in self
            .sessions
            .iter_mut()
            .filter(|member| member.link_set_id == link_set_id && member.id != source_id)
        {
            Self::install_links_by_index(member, links.clone());
        }
    }

    fn replace_link_set(&mut self, session: usize, links: Vec<Winlink>) {
        Self::install_links_by_index(&mut self.sessions[session], links);
        self.synchronize_group_from(session);
    }

    fn replace_link_set_preserving_positions(&mut self, session: usize, links: Vec<Winlink>) {
        let member = &mut self.sessions[session];
        member.windows = links;
        member.active = member.active.min(member.windows.len().saturating_sub(1));
        member.last_active = member
            .last_active
            .filter(|position| *position < member.windows.len());
        Self::refresh_last_window(member);
        self.synchronize_group_from(session);
    }

    fn insert_link(&mut self, session: usize, position: usize, link: Winlink, select: bool) {
        let link_id = link.link_id;
        let had_windows = !self.sessions[session].windows.is_empty();
        let active_id = self.sessions[session]
            .windows
            .get(self.sessions[session].active)
            .map(|candidate| candidate.link_id);
        self.sessions[session].windows.insert(position, link);
        self.sessions[session].active = active_id
            .and_then(|id| {
                self.sessions[session]
                    .windows
                    .iter()
                    .position(|candidate| candidate.link_id == id)
            })
            .unwrap_or(0);
        Self::refresh_last_window(&mut self.sessions[session]);
        self.synchronize_group_from(session);
        let (session_id, window_id) = {
            let member = &self.sessions[session];
            let link = member
                .windows
                .iter()
                .find(|candidate| candidate.link_id == link_id)
                .expect("inserted winlink is present");
            (member.id, link.id)
        };
        self.notify_session_window("window-linked", session_id, window_id);
        if select || !had_windows {
            let new_active = self.sessions[session]
                .windows
                .iter()
                .position(|candidate| candidate.link_id == link_id)
                .expect("inserted winlink is present");
            self.select_session_window(session, new_active);
        }
    }

    fn remove_link(&mut self, session: usize, position: usize) -> Winlink {
        let session_id = self.sessions[session].id;
        let mut links = self.sessions[session].windows.clone();
        let removed = links.remove(position);
        Self::install_links_by_index(&mut self.sessions[session], links);
        self.synchronize_group_from(session);
        self.notify_session_window("window-unlinked", session_id, removed.id);
        removed
    }

    fn remove_unlinked_windows(&mut self) {
        let linked = self
            .sessions
            .iter()
            .flat_map(|session| session.windows.iter().map(|link| link.id))
            .collect::<BTreeSet<_>>();
        let window_count = self.windows.len();
        self.windows.retain(|id, _| linked.contains(id));
        let live_link_sets = self
            .sessions
            .iter()
            .map(|session| session.link_set_id)
            .collect::<BTreeSet<_>>();
        self.session_groups
            .retain(|link_set_id, _| live_link_sets.contains(link_set_id));
        if self.windows.len() != window_count {
        }
    }

    pub(crate) fn client_prompt_registry(&self) -> Arc<ClientPromptRegistry> {
        Arc::clone(&self.client_prompts)
    }

    pub(crate) fn client_render_registry(&self) -> Arc<ClientRenderRegistry> {
        Arc::clone(&self.client_renders)
    }

    pub(crate) fn attached_clients(&self) -> Vec<AttachedClient> {
        self.client_renders.clients()
    }

    pub(crate) fn send_client_message(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        session_id: u32,
        message: ClientMessage,
    ) -> ClientMessageResult {
        self.client_renders
            .send_message(target, invoking_tty, session_id, message)
    }

    pub(crate) fn set_client_selection(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        data: Option<Vec<u8>>,
    ) -> ClientActionResult {
        self.client_renders
            .set_client_selection(target, invoking_tty, data)
    }

    /// Record a terminal focus report. The client's `focused` flag follows it
    /// whatever `focus-events` says — the option only decides whether tmux asks
    /// the terminal for reports at all — and the active pane's focus moves with
    /// the client's.
    pub(crate) fn set_client_focus(
        &mut self,
        client: &str,
        session_id: u32,
        focused: bool,
        target: &str,
    ) {
        if !self.client_renders.set_client_focused(client, focused) {
            return;
        }
        let was_deferred = std::mem::replace(&mut self.notifications_are_deferred, true);
        self.notify_client(
            if focused {
                "client-focus-in"
            } else {
                "client-focus-out"
            },
            client,
            Some(session_id),
        );
        self.notifications_are_deferred = was_deferred;
        let _ = target;
        if let Some(window_id) = self.current_window_of_session(session_id) {
            self.update_window_focus(window_id);
        }
    }

    /// The id of a session's current window.
    fn current_window_of_session(&self, session_id: u32) -> Option<u32> {
        let session = self.sessions.iter().find(|s| s.id == session_id)?;
        session.windows.get(session.active).map(|link| link.id)
    }

    /// tmux's `window_update_focus`: recompute whether the window's active pane
    /// holds focus and announce a change. Deliberately ungated — the
    /// `focus-events` option decides whether tmux *asks* the terminal for focus
    /// reports, not whether it acts on what it knows.
    pub(crate) fn update_window_focus(&mut self, window_id: u32) {
        let Some(active) = self
            .windows
            .get(&window_id)
            .and_then(|window| window.panes.get(window.active))
            .map(|pane| pane.id)
        else {
            return;
        };
        let focused = self.window_has_focused_client(window_id);
        self.update_pane_focus(active, focused);
    }

    /// Whether some attached, focused client is currently showing this window.
    fn window_has_focused_client(&self, window_id: u32) -> bool {
        self.attached_clients().into_iter().any(|client| {
            client.focused
                && self
                    .current_window_of_session(client.session_id)
                    .is_some_and(|current| current == window_id)
        })
    }

    /// tmux's `window_pane_update_focus`: announce a pane focus change once.
    pub(crate) fn update_pane_focus(&mut self, pane_id: u32, focused: bool) {
        let changed = if focused {
            self.focused_panes.insert(pane_id)
        } else {
            self.focused_panes.remove(&pane_id)
        };
        if !changed {
            return;
        }
        // A pane that asked for focus reporting gets its own copy of the
        // escape, exactly as `window_pane_update_focus` sends it.
        let subscribed = self
            .windows
            .values()
            .flat_map(|window| window.panes.iter())
            .find(|node| node.id == pane_id)
            .is_some_and(|node| node.pane.focus_reporting_enabled());
        if subscribed {
            let _ = self.write_pane_input(pane_id, if focused { b"\x1b[I" } else { b"\x1b[O" });
        }
        let was_deferred = std::mem::replace(&mut self.notifications_are_deferred, true);
        self.notify_pane(
            if focused {
                "pane-focus-in"
            } else {
                "pane-focus-out"
            },
            pane_id,
        );
        self.notifications_are_deferred = was_deferred;
    }

    /// tmux's `server_client_get_key_table`: the table a client attached to
    /// `target` uses until the prefix key moves it elsewhere.
    pub(crate) fn session_key_table(&self, target: &str) -> String {
        self.option_for_target(target, "key-table")
            .filter(|table| !table.is_empty())
            .unwrap_or(DEFAULT_KEY_TABLE)
            .to_string()
    }

    /// Record the key table an attached client moved into, so
    /// `#{client_key_table}`/`#{client_prefix}` see it. The owning client
    /// repaints its own status line; no other client's view depends on it.
    pub(crate) fn set_client_key_table(&mut self, client: &str, table: &str) {
        self.client_renders.set_client_key_table(client, table);
    }

    /// Record the theme a client's terminal reported, firing the matching hook
    /// when it changes.
    pub(crate) fn set_client_theme(&mut self, client: &str, session_id: u32, theme: &str) {
        if !self.client_renders.set_client_theme(client, theme) {
            return;
        }
        let was_deferred = std::mem::replace(&mut self.notifications_are_deferred, true);
        self.notify_client(
            if theme == "dark" {
                "client-dark-theme"
            } else {
                "client-light-theme"
            },
            client,
            Some(session_id),
        );
        self.notifications_are_deferred = was_deferred;
    }

    /// Whether a clipboard query may be sent to this client now. tmux keeps one
    /// outstanding request per terminal for `CLIPBOARD_QUERY_TIMEOUT`, so a
    /// repeat inside the window is dropped instead of queued.
    pub(crate) fn begin_clipboard_query(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> bool {
        const CLIPBOARD_QUERY_TIMEOUT: Duration = Duration::from_secs(5);
        self.client_renders
            .begin_clipboard_query(target, invoking_tty, CLIPBOARD_QUERY_TIMEOUT)
    }

    pub(crate) fn overlay_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        request: OverlayRequest,
        reply: Option<mpsc::Sender<PromptCompletion>>,
    ) -> ClientActionResult {
        self.client_renders
            .overlay_client(target, invoking_tty, request, reply)
    }

    pub(crate) fn confirm_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        prompt: String,
        command: Vec<String>,
        confirm_key: u8,
        default_yes: bool,
        reply: Option<mpsc::Sender<PromptCompletion>>,
    ) -> ClientActionResult {
        self.client_renders.confirm_client(
            target,
            invoking_tty,
            prompt,
            command,
            confirm_key,
            default_yes,
            reply,
        )
    }

    pub(crate) fn wait_registry(&self) -> Arc<WaitRegistry> {
        Arc::clone(&self.wait_registry)
    }

    #[allow(dead_code)]
    pub(crate) fn format_job_registry(&self) -> Arc<super::status::FormatJobRegistry> {
        Arc::clone(&self.format_jobs)
    }

    pub(crate) fn add_message(&mut self, text: String) {
        let limit = self
            .server_options()
            .get("message-limit")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(1000);
        self.message_log.push(MessageLogEntry {
            time: now_epoch(),
            text,
        });
        if self.message_log.len() > limit {
            self.message_log.drain(..self.message_log.len() - limit);
        }
    }

    pub(crate) fn messages(&self) -> &[MessageLogEntry] {
        &self.message_log
    }

    pub(crate) fn background_job_registry(&self) -> Arc<BackgroundJobRegistry> {
        Arc::clone(&self.background_jobs)
    }

    pub(crate) fn begin_hook(&mut self, name: &str) -> bool {
        self.running_hooks.insert(name.to_string())
    }

    pub(crate) fn end_hook(&mut self, name: &str) {
        self.running_hooks.remove(name);
    }

    /// Swap in the `hook*` format variables for a hook body, returning the
    /// previous set so the caller can restore it.
    pub(crate) fn replace_hook_format_vars(
        &mut self,
        vars: Vec<(String, String)>,
    ) -> Vec<(String, String)> {
        std::mem::replace(&mut self.hook_format_vars, vars)
    }

    pub(crate) fn hook_format_vars(&self) -> &[(String, String)] {
        &self.hook_format_vars
    }

    fn lock_commands(&self) -> BTreeMap<u32, String> {
        self.sessions
            .iter()
            .map(|session| {
                let command = session
                    .options(&self.global_options)
                    .get("lock-command")
                    .unwrap_or("lock -np")
                    .to_string();
                (session.id, command)
            })
            .collect()
    }

    pub(crate) fn lock_all_clients(&self) {
        self.client_renders.lock_all(&self.lock_commands());
    }

    pub(crate) fn lock_session_clients(&self, target: &str) -> io::Result<()> {
        let session = self.resolve_session(target).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {target}"),
            )
        })?;
        let command = session
            .options(&self.global_options)
            .get("lock-command")
            .unwrap_or("lock -np");
        self.client_renders.lock_session(session.id, command);
        Ok(())
    }

    pub(crate) fn lock_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> ClientActionResult {
        self.client_renders
            .lock_client(target, invoking_tty, &self.lock_commands())
    }

    pub(crate) fn detach_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> ClientActionResult {
        self.client_renders.detach_client(target, invoking_tty)
    }

    pub(crate) fn suspend_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> ClientActionResult {
        self.client_renders.suspend_client(target, invoking_tty)
    }

    pub(crate) fn refresh_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> ClientActionResult {
        self.client_renders.refresh_client(target, invoking_tty)
    }

    /// Record `refresh-client -f` values on the target client and hand them to
    /// it so its own flag state follows.
    pub(crate) fn refresh_client_flags(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        values: &[String],
    ) -> ClientActionResult {
        self.client_renders
            .refresh_client_flags(target, invoking_tty, values)
    }

    pub(crate) fn client_read_only(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> Option<bool> {
        self.client_renders.client_read_only(target, invoking_tty)
    }

    pub(crate) fn send_client_keys(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        keys: Vec<ClientKey>,
    ) -> ClientActionResult {
        self.client_renders
            .send_client_keys(target, invoking_tty, keys)
    }

    pub(crate) fn switch_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        session_id: u32,
    ) -> ClientActionResult {
        self.client_renders
            .switch_client(target, invoking_tty, session_id)
    }

    pub(crate) fn session_id(&self, name: &str) -> Option<u32> {
        self.resolve_session(name).map(|session| session.id)
    }

    pub(crate) fn replace_command_session_id(&mut self, session_id: Option<u32>) -> Option<u32> {
        std::mem::replace(&mut self.command_session_id, session_id)
    }

    pub(crate) fn replace_command_window_id(&mut self, window_id: Option<u32>) -> Option<u32> {
        std::mem::replace(&mut self.command_window_id, window_id)
    }

    /// The window a command with no explicit target defaults to inside
    /// `session`: the hook target's window while a hook body runs, otherwise
    /// the session's active window.
    pub(crate) fn command_window_index(&self, session: &Session) -> usize {
        self.command_window_id
            .and_then(|id| session.windows.iter().position(|link| link.id == id))
            .unwrap_or(session.active)
    }

    pub(crate) fn replace_command_active_panes(
        &mut self,
        panes: Option<BTreeMap<u32, u32>>,
    ) -> Option<BTreeMap<u32, u32>> {
        std::mem::replace(&mut self.command_active_panes, panes)
    }

    pub(crate) fn command_session_name(&self) -> Option<String> {
        let session_id = self.command_session_id?;
        self.sessions
            .iter()
            .find(|session| session.id == session_id)
            .map(|session| session.name.clone())
    }

    pub(crate) fn add_prompt_history(&mut self, prompt_type: &str, value: &str) {
        if value.is_empty() {
            return;
        }
        let limit = self
            .server_options()
            .get("prompt-history-limit")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(100);
        let history = self
            .prompt_history
            .entry(prompt_type.to_string())
            .or_default();
        if history.last().is_some_and(|last| last == value) {
            return;
        }
        history.push(value.to_string());
        if history.len() > limit {
            history.drain(..history.len() - limit);
        }
    }

    pub(crate) fn prompt_history(&self, prompt_type: &str) -> &[String] {
        self.prompt_history
            .get(prompt_type)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub(crate) fn clear_prompt_history(&mut self, prompt_type: Option<&str>) {
        if let Some(prompt_type) = prompt_type {
            self.prompt_history.remove(prompt_type);
        } else {
            self.prompt_history.clear();
        }
    }

    fn invalidate_session(&self, session_id: u32, reason: RenderInvalidation) {
        self.client_renders.publish_session(session_id, reason);
    }

    fn invalidate_all_clients(&self, reason: RenderInvalidation) {
        self.client_renders.publish_all(reason);
    }

    pub(crate) fn option_changed(&self, name: &str) {
        if option_affects_render(name) {
            self.invalidate_all_clients(option_invalidation(name));
        }
    }

    /// Whether the outer listener should stop accepting clients.
    pub fn shutdown_requested(&self) -> bool {
        self.shutdown_requested
    }

    /// Apply pane child-exit lifecycle (`remain-on-exit`, then `exit-empty`).
    ///
    /// Returns true when at least one pane was removed. Empty windows and
    /// sessions are removed with it, matching tmux's ownership hierarchy.
    pub fn reap_exited_panes(&mut self) -> bool {
        let had_sessions = !self.sessions.is_empty();
        let mut removed = false;
        let event_loop_io = matches!(self.pane_io_mode, PaneIoMode::EventLoop);
        for window in self.windows.values_mut() {
            for pane in &mut window.panes {
                if pane.pane.has_exited() {
                    pane.pane.collect_exited_child(event_loop_io);
                }
            }
        }
        let retained = self
            .windows
            .values()
            .flat_map(|window| {
                window.panes.iter().filter_map(|pane| {
                    (pane
                        .options(window, &self.global_options)
                        .get("remain-on-exit")
                        == Some("on"))
                    .then_some(pane.id)
                })
            })
            .collect::<BTreeSet<_>>();
        // tmux's `server_destroy_pane` reports `pane-died` for a pane held open
        // by `remain-on-exit` and `pane-exited` for one that goes away. Either
        // way it is announced once, the first time the child is seen gone.
        let newly_exited = self
            .windows
            .values_mut()
            .flat_map(|window| window.panes.iter_mut())
            .filter(|pane| pane.pane.has_exited() && !pane.exit_notified)
            .map(|pane| {
                pane.exit_notified = true;
                pane.id
            })
            .collect::<Vec<_>>();
        self.deferred_notifications(|state| {
            for pane_id in newly_exited {
                let name = if retained.contains(&pane_id) {
                    "pane-died"
                } else {
                    "pane-exited"
                };
                state.notify_pane(name, pane_id);
            }
        });
        for window in self.windows.values_mut() {
            let active_id = window.panes.get(window.active).map(|pane| pane.id);
            let last_id = window
                .last_pane
                .and_then(|index| window.panes.get(index))
                .map(|pane| pane.id);
            let exited_ids = window
                .panes
                .iter()
                .filter(|pane| {
                    pane.pane.has_exited()
                        && (!event_loop_io || pane.pane.child_reaped())
                        && !retained.contains(&pane.id)
                })
                .map(|pane| pane.id)
                .collect::<Vec<_>>();
            let before = window.panes.len();
            window.panes.retain(|pane| {
                !pane.pane.has_exited()
                    || (event_loop_io && !pane.pane.child_reaped())
                    || retained.contains(&pane.id)
            });
            let panes_removed = window.panes.len() != before;
            removed |= panes_removed;
            if panes_removed && !window.panes.is_empty() {
                for pane_id in exited_ids {
                    window.layout.remove(pane_id);
                }
                let _ = resize_panes_to_layout(window);
                window.active = active_id
                    .and_then(|id| window.panes.iter().position(|pane| pane.id == id))
                    .unwrap_or_else(|| window.active.min(window.panes.len() - 1));
                window.last_pane =
                    last_id.and_then(|id| window.panes.iter().position(|pane| pane.id == id));
            }
        }

        let empty = self
            .windows
            .iter()
            .filter_map(|(id, window)| window.panes.is_empty().then_some(*id))
            .collect::<BTreeSet<_>>();
        let link_sets = self
            .sessions
            .iter()
            .map(|session| session.link_set_id)
            .collect::<BTreeSet<_>>();
        for link_set_id in link_sets {
            let Some(member) = self
                .sessions
                .iter()
                .position(|session| session.link_set_id == link_set_id)
            else {
                continue;
            };
            let old = self.sessions[member].windows.clone();
            let links = old
                .iter()
                .copied()
                .filter(|link| !empty.contains(&link.id))
                .collect::<Vec<_>>();
            if links.len() == old.len() {
                continue;
            }
            self.replace_link_set(member, links);
        }
        self.sessions.retain(|session| !session.windows.is_empty());
        self.remove_unlinked_windows();

        if had_sessions && self.sessions.is_empty() && self.server_option_is_on("exit-empty", true)
        {
            self.shutdown_requested = true;
        }
        if removed {
        }
        removed
    }

    fn server_option_is_on(&self, name: &str, default: bool) -> bool {
        match self.server_options().get(name) {
            Some("on" | "yes" | "true" | "1") => true,
            Some("off" | "no" | "false" | "0") => false,
            Some(_) | None => default,
        }
    }

    /// Apply tmux's `exit-empty` policy after an explicit tree mutation.
    fn request_shutdown_if_became_empty(&mut self, had_sessions: bool) {
        if had_sessions && self.sessions.is_empty() && self.server_option_is_on("exit-empty", true)
        {
            self.shutdown_requested = true;
        }
    }

    pub fn find(&self, name: &str) -> Option<&Session> {
        self.resolve_session(name)
    }

    /// The next unused numeric session name (tmux names sessions 0,1,2,… by
    /// default).
    pub fn next_session_name(&self) -> String {
        let mut n = 0u32;
        while self.sessions.iter().any(|s| s.name == n.to_string()) {
            n += 1;
        }
        n.to_string()
    }

    /// Create a session named `name` with a single window holding one pane.
    pub fn create_session(&mut self, name: &str, spec: PaneSpec) -> io::Result<u32> {
        if self.find(name).is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("duplicate session: {name}"),
            ));
        }
        let (cols, rows) = (self.default_cols, self.default_rows);
        let start_command = pane_start_command(&spec);
        let pane = match spec {
            PaneSpec::Inert => Pane::inert(cols, rows)?,
            PaneSpec::Command(argv) => {
                let refs: Vec<&str> = argv.iter().map(String::as_str).collect();
                Pane::spawn_in_mode(&refs, None, cols, rows, self.pane_io_mode)?
            }
            PaneSpec::CommandIn(argv, cwd) => {
                let refs: Vec<&str> = argv.iter().map(String::as_str).collect();
                Pane::spawn_in_mode(&refs, Some(&cwd), cols, rows, self.pane_io_mode)?
            }
        };

        let session_id = self.next_session_id;
        self.next_session_id += 1;
        let link_set_id = self.next_link_set_id;
        self.next_link_set_id += 1;
        let window_id = self.next_window_id;
        self.next_window_id += 1;
        let winlink_id = self.next_winlink_id;
        self.next_winlink_id += 1;
        let pane_id = self.next_pane_id;
        self.next_pane_id += 1;

        let base = self.base_index();
        self.windows.insert(
            window_id,
            Window {
                id: window_id,
                name: String::new(),
                panes: vec![PaneNode {
                    id: pane_id,
                    pane,
                    start_command,
                    input_off: false,
                    title: None,
                    exit_notified: false,
                    mode: None,
                    copy: None,
                    mode_view: None,
                    search_string: None,
                    search_regex: false,
                    floating: None,
                    options: OptionSet::default(),
                }],
                active: 0,
                last_pane: None,
                zoomed: false,
                manual_size: None,
                layout: LayoutCell::pane(pane_id, cols, rows),
                last_layout: None,
                last_new_pane_x: 0,
                last_new_pane_y: 0,
                // tmux's `window_create` raises activity, so a monitor turned on
                // later still sees the window as having been active.
                pending_alerts: ALERT_ACTIVITY,
                options: OptionSet::default(),
            },
        );
        self.sessions.push(Session {
            id: session_id,
            name: name.to_string(),
            cols,
            rows,
            active: 0,
            last_active: None,
            last_windows: Vec::new(),
            created: created_stamp(),
            created_epoch: now_epoch(),
            // tmux seeds activity from the creation time, so creation order is
            // the initial `detach-on-destroy` ordering.
            activity_micros: now_micros(),
            last_attached_micros: 0,
            locked_at_activity_micros: None,
            windows: vec![Winlink {
                link_id: winlink_id,
                index: base,
                id: window_id,
                alert_flags: 0,
            }],
            link_set_id,
            environment: BTreeMap::new(),
            removed_environment: BTreeSet::new(),
            hidden_environment: BTreeSet::new(),
            options: OptionSet::default(),
        });
        self.initial_attach_pending = false;
        self.notify_session("session-created", session_id);
        Ok(session_id)
    }

    /// Create a session in a tmux session group. `target` may name an existing
    /// group, an existing session (creating a group named after it if needed),
    /// or a new one-member group. Group members share their link set but keep
    /// independent current and previous windows.
    pub fn create_grouped_session(
        &mut self,
        name: &str,
        target: &str,
        spec: PaneSpec,
    ) -> io::Result<u32> {
        if self.find(name).is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("duplicate session: {name}"),
            ));
        }
        let target_session = self.session_index(target).or_else(|| {
            if target.starts_with('$') || target.starts_with('=') {
                return None;
            }
            let mut matches = self
                .sessions
                .iter()
                .enumerate()
                .filter(|(_, session)| session.name.starts_with(target));
            let (position, _) = matches.next()?;
            matches.next().is_none().then_some(position)
        });
        let link_set_id = if let Some(target_session) = target_session {
            let link_set_id = self.sessions[target_session].link_set_id;
            if !self.session_groups.contains_key(&link_set_id) {
                let group_name = self.sessions[target_session].name.clone();
                self.session_groups.insert(link_set_id, group_name);
            }
            Some(link_set_id)
        } else {
            self.session_groups
                .iter()
                .find_map(|(link_set_id, group_name)| {
                    (group_name == target).then_some(*link_set_id)
                })
        };

        let Some(link_set_id) = link_set_id else {
            let session_id = self.create_session(name, spec)?;
            let link_set_id = self
                .sessions
                .iter()
                .find(|session| session.id == session_id)
                .expect("new session is present")
                .link_set_id;
            self.session_groups.insert(link_set_id, target.to_string());
            return Ok(session_id);
        };

        let target = self
            .sessions
            .iter()
            .position(|session| session.link_set_id == link_set_id)
            .expect("a live session owns every live group");
        // tmux creates the new session's initial window before adding it to the
        // group, then discards that window while synchronizing. Going through
        // `create_session` preserves observable window/pane ID consumption and
        // spawn failures.
        let source_windows = self.sessions[target].windows.clone();
        let source_size = (self.sessions[target].cols, self.sessions[target].rows);
        let session_id = self.create_session(name, spec)?;
        let created = self
            .sessions
            .iter()
            .position(|session| session.id == session_id)
            .expect("new session is present");
        self.sessions[created].windows = source_windows;
        self.sessions[created].link_set_id = link_set_id;
        self.sessions[created].active = 0;
        self.sessions[created].last_active = None;
        self.sessions[created].last_windows.clear();
        self.sessions[created].cols = source_size.0;
        self.sessions[created].rows = source_size.1;
        self.remove_unlinked_windows();
        Ok(session_id)
    }

    /// Install or replace a mutable key-table binding.
    pub(crate) fn bind_key(
        &mut self,
        table: &str,
        key: KeyCode,
        command: Vec<String>,
        repeat: bool,
        note: Option<String>,
    ) {
        self.key_tables
            .entry(table.to_string())
            .or_default()
            .insert(
                key,
                KeyBinding {
                    repeat,
                    note,
                    command,
                },
            );
    }

    /// Remove one binding, or all bindings in a table when `all` is set.
    pub(crate) fn unbind_key(&mut self, table: &str, key: Option<KeyCode>, all: bool) {
        if all {
            self.key_tables.remove(table);
        } else if let Some(key) = key {
            if let Some(bindings) = self.key_tables.get_mut(table) {
                bindings.remove(&key);
                if bindings.is_empty() {
                    self.key_tables.remove(table);
                }
            }
        }
    }

    pub(crate) fn key_binding(&self, table: &str, key: KeyCode) -> Option<&KeyBinding> {
        self.key_tables.get(table)?.get(&key)
    }

    /// Resolve a key against the target pane's active mode table.
    ///
    /// The outer `Option` distinguishes a pane with no mode from an active mode
    /// with no binding for `key`; tmux consumes keys in the latter case.
    pub(crate) fn pane_mode_key_binding(
        &self,
        target: &str,
        key: KeyCode,
        vi: bool,
    ) -> io::Result<Option<Option<KeyBinding>>> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let pane = &self.window(resolved.session, resolved.window).panes[resolved.pane];
        if pane.copy.is_none() {
            return Ok(None);
        }
        let table = if vi { "copy-mode-vi" } else { "copy-mode" };
        Ok(Some(self.key_binding(table, key).cloned()))
    }

    pub(crate) fn key_table_exists(&self, table: &str) -> bool {
        self.key_tables.contains_key(table)
    }

    pub(crate) fn key_bindings(&self, table: Option<&str>) -> Vec<(&str, KeyCode, &KeyBinding)> {
        let mut out = Vec::new();
        for (table_name, bindings) in &self.key_tables {
            if table.is_some_and(|wanted| wanted != table_name) {
                continue;
            }
            for (key, binding) in bindings {
                out.push((table_name.as_str(), *key, binding));
            }
        }
        out
    }

    /// Rename session `from` to `to`, mirroring tmux's `rename-session`. Errors
    /// if `from` is missing (`can't find session`) or `to` is already taken
    /// (`duplicate session`), matching tmux's diagnostics.
    pub fn rename_session(&mut self, from: &str, to: &str) -> io::Result<()> {
        let Some(session_id) = self.find(from).map(|session| session.id) else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {from}"),
            ));
        };
        if from != to && self.find(to).is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("duplicate session: {to}"),
            ));
        }
        let mut renamed = false;
        if let Some(s) = self.sessions.iter_mut().find(|s| s.id == session_id) {
            if s.name != to {
                s.name = to.to_string();
                renamed = true;
            }
        }
        if renamed {
            self.invalidate_session(session_id, RenderInvalidation::STATUS);
            self.notify_session("session-renamed", session_id);
        }
        Ok(())
    }

    /// Open a new window (one shell-backed pane) in session `session`, as tmux's
    /// `new-window` does.
    ///
    /// `explicit` is the requested window index (`new-window -t sess:N`); `None`
    /// means append at the next free index (`new-window -t sess:`), which tmux
    /// picks as one past the highest existing index (starting from `base-index`,
    /// which defaults to 0). An `explicit` index already in use is rejected with
    /// tmux's `create window failed: index N in use`. Errors if the session is
    /// missing. Windows are kept sorted by `index`; the returned value is the
    /// `Vec` position of the new window (now the session's active window).
    pub fn new_window(
        &mut self,
        session: &str,
        explicit: Option<u32>,
        select: bool,
    ) -> io::Result<usize> {
        self.new_window_impl(session, explicit, false, select)
    }

    pub(crate) fn new_window_with_spawn(
        &mut self,
        session: &str,
        explicit: Option<u32>,
        select: bool,
        argv: &[String],
        cwd: Option<&Path>,
    ) -> io::Result<usize> {
        self.new_window_spawn_impl(session, explicit, false, select, argv, cwd)
    }

    /// Open a new window, replacing an existing explicit target for
    /// `new-window -k`.
    pub(crate) fn new_window_replacing_with_spawn(
        &mut self,
        session: &str,
        explicit: Option<u32>,
        select: bool,
        argv: &[String],
        cwd: Option<&Path>,
    ) -> io::Result<usize> {
        self.new_window_spawn_impl(session, explicit, true, select, argv, cwd)
    }

    fn new_window_impl(
        &mut self,
        session: &str,
        explicit: Option<u32>,
        replace: bool,
        select: bool,
    ) -> io::Result<usize> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        self.new_window_spawn_impl(session, explicit, replace, select, &[shell], None)
    }

    fn new_window_spawn_impl(
        &mut self,
        session: &str,
        explicit: Option<u32>,
        replace: bool,
        select: bool,
        argv: &[String],
        cwd: Option<&Path>,
    ) -> io::Result<usize> {
        let session_pos = self.session_index(session);
        let s = match session_pos.and_then(|pos| self.sessions.get(pos)) {
            Some(s) => s,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("can't find session: {session}"),
                ))
            }
        };
        let session_id = s.id;
        let had_windows = !s.windows.is_empty();
        let index = match explicit {
            Some(i) => {
                if !replace && s.windows.iter().any(|w| w.index == i) {
                    return Err(io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        format!("create window failed: index {i} in use"),
                    ));
                }
                i
            }
            None => {
                // tmux appends at the lowest unused index at or above
                // `base-index`, not simply one past the maximum.
                let mut i = s
                    .options(&self.global_options)
                    .get("base-index")
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(0);
                while s.windows.iter().any(|w| w.index == i) {
                    i += 1;
                }
                i
            }
        };

        // Match ordinary new-window: inherit the destination session's current
        // size rather than briefly spawning at the server's 80x24 defaults.
        let (cols, rows) = (s.cols, s.rows);
        // Spawn the pane before mutating counters so a spawn failure leaves state
        // untouched.
        let refs: Vec<&str> = argv.iter().map(String::as_str).collect();
        let pane = Pane::spawn_in_mode(&refs, cwd, cols, rows, self.pane_io_mode)?;

        let window_id = self.next_window_id;
        self.next_window_id += 1;
        let winlink_id = self.next_winlink_id;
        self.next_winlink_id += 1;
        let pane_id = self.next_pane_id;
        self.next_pane_id += 1;

        let session_pos = self
            .sessions
            .iter()
            .position(|s| s.id == session_id)
            .expect("session presence checked above");
        if replace {
            if let Some(pos) = self.sessions[session_pos]
                .windows
                .iter()
                .position(|w| w.index == index)
            {
                self.remove_link(session_pos, pos);
            }
        }
        // Insert keeping the window list sorted by index.
        let pos = self.sessions[session_pos]
            .windows
            .iter()
            .take_while(|w| w.index < index)
            .count();
        self.windows.insert(
            window_id,
            Window {
                id: window_id,
                name: String::new(),
                panes: vec![PaneNode {
                    id: pane_id,
                    pane,
                    start_command: String::new(),
                    input_off: false,
                    title: None,
                    exit_notified: false,
                    mode: None,
                    copy: None,
                    mode_view: None,
                    search_string: None,
                    search_regex: false,
                    floating: None,
                    options: OptionSet::default(),
                }],
                active: 0,
                last_pane: None,
                zoomed: false,
                manual_size: None,
                layout: LayoutCell::pane(pane_id, cols, rows),
                last_layout: None,
                last_new_pane_x: 0,
                last_new_pane_y: 0,
                // tmux's `window_create` raises activity, so a monitor turned on
                // later still sees the window as having been active.
                pending_alerts: ALERT_ACTIVITY,
                options: OptionSet::default(),
            },
        );
        self.insert_link(
            session_pos,
            pos,
            Winlink {
                link_id: winlink_id,
                index,
                id: window_id,
                alert_flags: 0,
            },
            select,
        );
        self.remove_unlinked_windows();
        let reason = if select || !had_windows {
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS
        } else {
            RenderInvalidation::STATUS
        };
        self.invalidate_session(session_id, reason);
        Ok(pos)
    }

    /// `new-window -a`/`-b`: open a new window *relative* to an existing anchor
    /// window (tmux's `winlink_shuffle_up`). `anchor_index` is the client-visible
    /// index of an existing window in `session`; `after` selects `-a` (insert just
    /// after the anchor) vs `-b` (insert at the anchor's own index, pushing it up).
    ///
    /// The new window's desired index is `anchor_index + 1` (`-a`) or
    /// `anchor_index` (`-b`). tmux then finds the first *free* index at or above
    /// that desired index and shifts the contiguous run of occupied windows in
    /// `[desired, free)` up by one to open the slot — windows past the first gap
    /// keep their indices. The new window becomes the session's active window and
    /// the previous active window becomes "last". Returns the new window's `Vec`
    /// position. Errors (`can't find session`/`can't find window`) mirror tmux.
    pub fn new_window_relative(
        &mut self,
        session: &str,
        anchor_index: u32,
        after: bool,
        select: bool,
    ) -> io::Result<usize> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        self.new_window_relative_with_spawn(session, anchor_index, after, select, &[shell], None)
    }

    pub(crate) fn new_window_relative_with_spawn(
        &mut self,
        session: &str,
        anchor_index: u32,
        after: bool,
        select: bool,
        argv: &[String],
        cwd: Option<&Path>,
    ) -> io::Result<usize> {
        let session_pos = self.session_index(session).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {session}"),
            )
        })?;
        let s = &self.sessions[session_pos];
        let session_id = s.id;
        if !s.windows.iter().any(|w| w.index == anchor_index) {
            return Err(window_not_found(&anchor_index.to_string()));
        }
        let desired = if after {
            anchor_index + 1
        } else {
            anchor_index
        };
        // First free index at or above `desired` (end of the contiguous run to shift).
        let mut free = desired;
        while s.windows.iter().any(|w| w.index == free) {
            free += 1;
        }

        // Match ordinary new-window: inherit the destination session's current
        // size rather than briefly spawning at the server's 80x24 defaults.
        // Spawn before mutating counters so a failure leaves state untouched.
        let (cols, rows) = (s.cols, s.rows);
        let refs: Vec<&str> = argv.iter().map(String::as_str).collect();
        let pane = Pane::spawn_in_mode(&refs, cwd, cols, rows, self.pane_io_mode)?;

        let window_id = self.next_window_id;
        self.next_window_id += 1;
        let winlink_id = self.next_winlink_id;
        self.next_winlink_id += 1;
        let pane_id = self.next_pane_id;
        self.next_pane_id += 1;

        let session_pos = self
            .sessions
            .iter()
            .position(|s| s.id == session_id)
            .expect("session presence checked above");
        // Shift the contiguous occupied run [desired, free) up by one to open the
        // `desired` slot. This preserves the sorted-by-index invariant (a monotone
        // shift of a contiguous prefix of indices keeps their relative order).
        for win in self.links_mut(session_pos).iter_mut() {
            if win.index >= desired && win.index < free {
                win.index += 1;
            }
        }
        let pos = self.sessions[session_pos]
            .windows
            .iter()
            .take_while(|w| w.index < desired)
            .count();
        self.windows.insert(
            window_id,
            Window {
                id: window_id,
                name: String::new(),
                panes: vec![PaneNode {
                    id: pane_id,
                    pane,
                    start_command: String::new(),
                    input_off: false,
                    title: None,
                    exit_notified: false,
                    mode: None,
                    copy: None,
                    mode_view: None,
                    search_string: None,
                    search_regex: false,
                    floating: None,
                    options: OptionSet::default(),
                }],
                active: 0,
                last_pane: None,
                zoomed: false,
                manual_size: None,
                layout: LayoutCell::pane(pane_id, cols, rows),
                last_layout: None,
                last_new_pane_x: 0,
                last_new_pane_y: 0,
                // tmux's `window_create` raises activity, so a monitor turned on
                // later still sees the window as having been active.
                pending_alerts: ALERT_ACTIVITY,
                options: OptionSet::default(),
            },
        );
        self.insert_link(
            session_pos,
            pos,
            Winlink {
                link_id: winlink_id,
                index: desired,
                id: window_id,
                alert_flags: 0,
            },
            select,
        );
        self.invalidate_session(
            session_id,
            if select {
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS
            } else {
                RenderInvalidation::STATUS
            },
        );
        Ok(pos)
    }

    fn set_window_name(
        &mut self,
        target: &str,
        name: &str,
        disable_automatic_rename: bool,
    ) -> io::Result<()> {
        let t = self.resolve(target).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find window: {target}"),
            )
        })?;
        let session_id = self.sessions[t.session].id;
        let window = self.window_mut(t.session, t.window);
        let window_id = window.id;
        let name_changed = window.name != name;
        let rename_changed =
            disable_automatic_rename && window.options.get("automatic-rename") != Some("off");
        if name_changed {
            window.name = name.to_string();
        }
        if disable_automatic_rename {
            window.options.set("automatic-rename", "off");
        }
        if name_changed || rename_changed {
            self.invalidate_session(session_id, RenderInvalidation::STATUS);
        }
        if name_changed {
            self.notify_window("window-renamed", window_id);
        }
        Ok(())
    }

    /// Rename a window (`rename-window`). `target` is `session[:window]`; a
    /// missing window part means the session's active window. Errors if the
    /// target can't be resolved (`can't find window`). Like tmux, an explicit
    /// name disables automatic renaming for the window.
    pub fn rename_window(&mut self, target: &str, name: &str) -> io::Result<()> {
        self.set_window_name(target, name, true)
    }

    /// Store a name produced by `automatic-rename-format` without disabling
    /// subsequent automatic updates.
    pub(crate) fn rename_window_automatically(
        &mut self,
        target: &str,
        name: &str,
    ) -> io::Result<()> {
        self.set_window_name(target, name, false)
    }

    /// Resolve a full pane target to concrete `Vec` positions. Accepts every
    /// tmux target form the control plane uses:
    /// - `%N` — a pane id (searched across all sessions/windows);
    /// - `@N` — a window id (`[.pane]` optional);
    /// - `$N:...` / `name:...` — a session (by id or name) with an optional
    ///   `:window` (index or `@id`) and `.pane` (index or `%id`) suffix.
    ///
    /// A missing window part means the session's active window; a missing pane
    /// part means that window's active pane. Returns `None` if any part can't be
    /// resolved.
    pub fn resolve(&self, target: &str) -> Option<Target> {
        // `%N` — pane id, resolved directly to its window and session.
        if let Some(id) = target.strip_prefix('%') {
            let id: u32 = id.parse().ok()?;
            for (si, sess) in self.sessions.iter().enumerate() {
                for (wi, link) in sess.windows.iter().enumerate() {
                    let win = self.window_for_link(link);
                    if let Some(pi) = win.panes.iter().position(|p| p.id == id) {
                        return Some(Target {
                            session: si,
                            window: wi,
                            pane: pi,
                        });
                    }
                }
            }
            return None;
        }

        let (win_target, pane_part) = split_pane_target(target);
        // Pane/session target: a colon-less bare number is a session name.
        let (session, window) = self.resolve_window_positions(win_target, false).ok()?;
        let pane = match pane_part {
            Some(p) => self.pane_pos(session, window, p)?,
            None => {
                let window = self.window(session, window);
                self.command_active_panes
                    .as_ref()
                    .and_then(|panes| panes.get(&window.id))
                    .and_then(|pane_id| window.panes.iter().position(|pane| pane.id == *pane_id))
                    .unwrap_or(window.active)
            }
        };
        Some(Target {
            session,
            window,
            pane,
        })
    }

    /// Resolve just the session named by a target (its part before any `:`),
    /// accepting a name or a `$id`. Used by `has-session`.
    pub fn resolve_session(&self, target: &str) -> Option<&Session> {
        let spec = target.split_once(':').map(|(s, _)| s).unwrap_or(target);
        self.session_index(spec).map(|i| &self.sessions[i])
    }

    /// Position of a session named by `spec`: a session name, or a `$id`.
    fn session_index(&self, spec: &str) -> Option<usize> {
        // A leading `=` requests an exact target-name match in tmux target
        // syntax (`-t=test:`). Session names in the model are stored without
        // that selector marker.
        let spec = spec.strip_prefix('=').unwrap_or(spec);
        if let Some(id) = spec.strip_prefix('$') {
            let id: u32 = id.parse().ok()?;
            self.sessions.iter().position(|s| s.id == id)
        } else {
            self.sessions.iter().position(|s| s.name == spec)
        }
    }

    /// Position of a session (by name or `$id`), for the navigation commands.
    fn session_pos(&self, name: &str) -> Option<usize> {
        self.session_index(name)
    }

    /// Position of the "current" session — the one a target with no (or empty)
    /// session part refers to. Real tmux picks the most-recently-active session;
    /// we approximate with the newest, matching `command::current_session`.
    fn current_session_pos(&self) -> io::Result<usize> {
        self.sessions.len().checked_sub(1).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "no current session".to_string())
        })
    }

    /// Resolve a session spec (name or `$id`) to its position, treating an empty
    /// spec as the current session. Mirrors tmux's `:win` == current-session rule.
    fn session_or_current(&self, spec: &str) -> io::Result<usize> {
        if spec.is_empty() {
            return self.current_session_pos();
        }
        self.session_index(spec).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {spec}"),
            )
        })
    }

    /// Resolve a pane index within a window from a pane spec: a numeric index
    /// (matched against `Vec` position), a `%id`, or one of tmux's special pane
    /// tokens (`{last}`, `{next}`, `{previous}`, `{top}`, `{bottom}`, `+`, `-`).
    fn pane_pos(&self, session: usize, window: usize, spec: &str) -> Option<usize> {
        let win = self.window(session, window);
        let base = win
            .options(&self.global_options)
            .get("pane-base-index")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);
        pane_pos_in(win, spec, base)
    }

    /// Resolve a `session[:window]` window-target to `(session_pos, window_pos)`.
    /// The window part may be a numeric index (matched against `Window::index`),
    /// a `@id`, or empty (the active window); the session part a name or `$id`.
    /// Also accepts a bare `@id` window target (no session part).
    ///
    /// An empty session part (`:win`) always means the current session. The
    /// colon-less bare-number case is *target-type dependent*, matching tmux's
    /// `cmd_find_target`: for a window target (`bare_number_is_window`), `1` is
    /// window index 1 in the current session; for a pane/session target it is a
    /// session name (so `display-message -t 0` resolves session `0`).
    fn resolve_window_positions(
        &self,
        target: &str,
        bare_number_is_window: bool,
    ) -> io::Result<(usize, usize)> {
        // Bare window id: `@N`.
        if let Some(id) = target.strip_prefix('@') {
            let id: u32 = id.parse().map_err(|_| window_not_found(target))?;
            for (si, sess) in self.sessions.iter().enumerate() {
                if let Some(wi) = sess.windows.iter().position(|w| w.id == id) {
                    return Ok((si, wi));
                }
            }
            return Err(window_not_found(target));
        }

        // Resolve the session part, following tmux's target grammar (the same
        // rules `parse_window_target` applies to `new-window`/`split-window`):
        // an empty session part (`:win`) means the current session, and — for a
        // window target only — a colon-less bare number (`1`) is a *window* index
        // in the current session, not a session name.
        let (session, win_part) = match target.split_once(':') {
            Some((s, w)) => (self.session_or_current(s)?, Some(w)),
            None if bare_number_is_window && target.parse::<u32>().is_ok() => {
                (self.current_session_pos()?, Some(target))
            }
            None => (self.session_or_current(target)?, None),
        };
        let sess = &self.sessions[session];
        let window = match win_part.filter(|w| !w.is_empty()) {
            Some(w) => {
                let w = w.strip_prefix('=').unwrap_or(w);
                if let Some(id) = w.strip_prefix('@') {
                    let id: u32 = id.parse().map_err(|_| window_not_found(w))?;
                    sess.windows
                        .iter()
                        .position(|win| win.id == id)
                        .ok_or_else(|| window_not_found(w))?
                } else if let Some(pos) = window_special(sess, w) {
                    pos
                } else {
                    let idx: u32 = w.parse().map_err(|_| window_not_found(w))?;
                    sess.windows
                        .iter()
                        .position(|win| win.index == idx)
                        .ok_or_else(|| {
                            io::Error::new(
                                io::ErrorKind::NotFound,
                                format!("can't find window: {idx}"),
                            )
                        })?
                }
            }
            None => sess.active,
        };
        Ok((session, window))
    }

    /// Resolve `session[:window]` for the window-lifecycle commands, returning
    /// tmux's exact diagnostics: `can't find session: <name>` if the session is
    /// missing, `can't find window: <idx>` if the window index doesn't exist.
    fn resolve_window(&self, target: &str) -> io::Result<Target> {
        // Pane/session target (split-window, select-pane, …): a colon-less bare
        // number names a session, not a window index.
        let (session, window) = self.resolve_window_positions(target, false)?;
        let pane = self.window(session, window).active;
        Ok(Target {
            session,
            window,
            pane,
        })
    }

    /// Like [`Self::resolve_window`], but for commands whose `-t` is a genuine
    /// *window* target (`select-window`, `kill-window`, `kill-window -a`): there
    /// a colon-less bare number is a window index in the current session, per
    /// tmux's `cmd_find_target`.
    fn resolve_window_arg(&self, target: &str) -> io::Result<Target> {
        let (session, window) = self
            .resolve_window_positions(target, true)
            .map_err(|error| {
                if !target.contains(':')
                    && !target.starts_with('$')
                    && error.to_string().starts_with("can't find session:")
                {
                    window_not_found(target)
                } else {
                    error
                }
            })?;
        let pane = self.window(session, window).active;
        Ok(Target {
            session,
            window,
            pane,
        })
    }

    /// Destroy a physical window and every winlink to it. tmux's `kill-window`
    /// is global to the window rather than an unlink from only the target
    /// session.
    fn destroy_window_id(&mut self, window_id: u32) {
        let had_sessions = !self.sessions.is_empty();
        let affected = self
            .sessions
            .iter()
            .filter(|session| session.windows.iter().any(|link| link.id == window_id))
            .map(|session| session.id)
            .collect::<Vec<_>>();
        let link_sets = self
            .sessions
            .iter()
            .filter(|session| session.windows.iter().any(|link| link.id == window_id))
            .map(|session| session.link_set_id)
            .collect::<BTreeSet<_>>();
        for link_set_id in link_sets {
            let Some(member) = self
                .sessions
                .iter()
                .position(|session| session.link_set_id == link_set_id)
            else {
                continue;
            };
            let links = self.sessions[member]
                .windows
                .iter()
                .copied()
                .filter(|link| link.id != window_id)
                .collect();
            self.replace_link_set(member, links);
        }
        // Named while the window is still known, but reported per session it
        // was linked into, exactly as tmux's `session_detach` does.
        for session_id in &affected {
            self.notify_session_window("window-unlinked", *session_id, window_id);
        }
        self.sessions.retain(|session| !session.windows.is_empty());
        self.windows.remove(&window_id);
        self.remove_unlinked_windows();
        self.renumber_affected_sessions(&affected);
        self.request_shutdown_if_became_empty(had_sessions);
        for session_id in affected {
            if self.sessions.iter().any(|session| session.id == session_id) {
                self.invalidate_session(
                    session_id,
                    RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
                );
            } else {
                self.invalidate_session(session_id, RenderInvalidation::SESSION_GONE);
            }
        }
    }

    /// `kill-window target`: destroy the target window everywhere it is linked.
    pub fn kill_window(&mut self, target: &str) -> io::Result<()> {
        let t = self.resolve_window_arg(target)?;
        let window_id = self.sessions[t.session].windows[t.window].id;
        self.destroy_window_id(window_id);
        Ok(())
    }

    /// `kill-window -a [-t target]`: kill every window in the target's session
    /// *except* the target itself, matching tmux. The survivor becomes the
    /// session's only (and active) window.
    pub fn kill_other_windows(&mut self, target: &str) -> io::Result<()> {
        let t = self.resolve_window_arg(target)?;
        let keep_id = self.sessions[t.session].windows[t.window].id;
        let links = &self.sessions[t.session].windows;
        let mut destroy = links
            .iter()
            .filter_map(|link| (link.id != keep_id).then_some(link.id))
            .collect::<BTreeSet<_>>();
        if links.iter().filter(|link| link.id == keep_id).count() > 1 {
            destroy.insert(keep_id);
        }
        for window_id in destroy {
            self.destroy_window_id(window_id);
        }
        Ok(())
    }

    /// `unlink-window`: remove one logical winlink from its session (and thus
    /// the synchronized position from every member of its group). `-k` only
    /// bypasses the singly-linked guard; it does not kill links in other
    /// sessions.
    pub fn unlink_window(&mut self, target: &str, kill: bool) -> io::Result<()> {
        let had_sessions = !self.sessions.is_empty();
        let t = self.resolve_window_arg(target)?;
        let link_set_id = self.sessions[t.session].link_set_id;
        let window_id = self.sessions[t.session].windows[t.window].id;
        let group_members = self
            .sessions
            .iter()
            .filter(|session| session.link_set_id == link_set_id)
            .count();
        if !kill && self.window_reference_count(window_id) == group_members {
            return Err(io::Error::other("window only linked to one session"));
        }
        let affected = self
            .sessions
            .iter()
            .filter(|session| session.link_set_id == link_set_id)
            .map(|session| session.id)
            .collect::<Vec<_>>();
        self.remove_link(t.session, t.window);
        if self
            .sessions
            .iter()
            .find(|session| session.link_set_id == link_set_id)
            .is_some_and(|session| session.windows.is_empty())
        {
            self.sessions
                .retain(|session| session.link_set_id != link_set_id);
        }
        self.remove_unlinked_windows();
        self.renumber_affected_sessions(&affected);
        self.request_shutdown_if_became_empty(had_sessions);
        for session_id in affected {
            if self.sessions.iter().any(|session| session.id == session_id) {
                self.invalidate_session(
                    session_id,
                    RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
                );
            } else {
                self.invalidate_session(session_id, RenderInvalidation::SESSION_GONE);
            }
        }
        Ok(())
    }

    /// `select-window target`: make the target window active (recording the
    /// previous active window as "last").
    pub fn select_window(&mut self, target: &str) -> io::Result<()> {
        let t = self.resolve_window_arg(target)?;
        let session_id = self.sessions[t.session].id;
        if self.sessions[t.session].active != t.window {
            self.select_session_window(t.session, t.window);
            self.resize_active_window_to_session_size(t.session)?;
            self.invalidate_session(
                session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        }
        Ok(())
    }

    /// `next-window`/`previous-window`: cycle the active window within a session.
    /// A single-window session has no next/previous window, matching tmux's
    /// `no next window` / `no previous window` errors.
    pub fn next_window(&mut self, session: &str) -> io::Result<()> {
        self.cycle_window(session, true)
    }

    pub fn previous_window(&mut self, session: &str) -> io::Result<()> {
        self.cycle_window(session, false)
    }

    /// `next-window -a` / `previous-window -a`: step to the next/previous window
    /// carrying a bell, activity, or silence alert.
    pub fn next_window_alert(&mut self, session: &str) -> io::Result<()> {
        self.cycle_window_alert(session, true)
    }

    pub fn previous_window_alert(&mut self, session: &str) -> io::Result<()> {
        self.cycle_window_alert(session, false)
    }

    fn cycle_window_alert(&mut self, session: &str, forward: bool) -> io::Result<()> {
        let session_pos = self.session_pos(session).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {session}"),
            )
        })?;
        let (active, len) = {
            let session = &self.sessions[session_pos];
            (session.active, session.windows.len())
        };
        for distance in 1..len {
            let position = if forward {
                (active + distance) % len
            } else {
                (active + len - distance) % len
            };
            if self.sessions[session_pos].windows[position].alert_flags != 0 {
                let session_id = self.sessions[session_pos].id;
                self.select_session_window(session_pos, position);
                self.resize_active_window_to_session_size(session_pos)?;
                self.invalidate_session(
                    session_id,
                    RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
                );
                return Ok(());
            }
        }
        let which = if forward { "next" } else { "previous" };
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("no {which} window"),
        ))
    }

    pub(crate) fn clear_session_alerts(&mut self, session: &str) -> io::Result<()> {
        let session_pos = self.session_pos(session).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {session}"),
            )
        })?;
        for link in &mut self.sessions[session_pos].windows {
            link.alert_flags = 0;
        }
        let session_id = self.sessions[session_pos].id;
        self.invalidate_session(session_id, RenderInvalidation::STATUS);
        Ok(())
    }

    pub(crate) fn alert_poll_timeout(&self) -> Option<std::time::Duration> {
        self.windows
            .values()
            .any(|window| {
                window
                    .options(&self.global_options)
                    .get("monitor-silence")
                    .and_then(|value| value.parse::<u64>().ok())
                    .is_some_and(|seconds| seconds != 0)
            })
            .then(|| std::time::Duration::from_millis(100))
    }

    pub(crate) fn refresh_alerts(&mut self, now: std::time::Instant) -> bool {
        struct WindowActivity {
            id: u32,
            output: bool,
            bell: bool,
            last_output: Option<std::time::Instant>,
            monitor_bell: bool,
            monitor_activity: bool,
            monitor_silence: u64,
        }

        let live_panes = self
            .windows
            .values()
            .flat_map(|window| &window.panes)
            .map(|pane| pane.id)
            .collect::<BTreeSet<_>>();
        self.pane_alert_seen
            .retain(|pane_id, _| live_panes.contains(pane_id));

        let mut activities = Vec::new();
        for window in self.windows.values() {
            let options = window.options(&self.global_options);
            let monitor_bell = options.get("monitor-bell") == Some("on");
            let monitor_activity = options.get("monitor-activity") == Some("on");
            let monitor_silence = options
                .get("monitor-silence")
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(0);
            let mut output = false;
            let mut bell = false;
            let mut last_output = None;
            for pane in &window.panes {
                let (revision, bells, at) = pane.pane.observation_state().alert_snapshot();
                let previous = self.pane_alert_seen.insert(pane.id, (revision, bells));
                let (previous_revision, previous_bells) = previous.unwrap_or((0, 0));
                output |= revision > previous_revision;
                bell |= bells > previous_bells;
                if at > last_output {
                    last_output = at;
                }
            }
            activities.push(WindowActivity {
                id: window.id,
                output,
                bell,
                last_output,
                monitor_bell,
                monitor_activity,
                monitor_silence,
            });
        }

        // Windows whose whole condition set is re-examined this pass because a
        // client took their session (tmux's `alerts_check_session`).
        let sessions_to_check = std::mem::take(&mut self.alert_check_sessions);
        let recheck = self
            .sessions
            .iter()
            .filter(|session| sessions_to_check.contains(&session.id))
            .flat_map(|session| session.windows.iter().map(|link| link.id))
            .collect::<BTreeSet<_>>();

        // Record each window's raised conditions, exactly as tmux's
        // `alerts_queue` sets `WINDOW_*` flags before its check runs.
        let mut deliveries = Vec::new();
        for activity in activities {
            if activity.output {
                self.window_last_activity
                    .insert(activity.id, activity.last_output.unwrap_or(now));
                self.silence_alerted.remove(&activity.id);
            } else {
                self.window_last_activity.entry(activity.id).or_insert(now);
            }
            let silence = activity.monitor_silence != 0
                && !self.silence_alerted.contains(&activity.id)
                && self
                    .window_last_activity
                    .get(&activity.id)
                    .is_some_and(|last| {
                        now.saturating_duration_since(*last)
                            >= std::time::Duration::from_secs(activity.monitor_silence)
                    });
            if silence {
                self.silence_alerted.insert(activity.id);
            }
            let Some(window) = self.windows.get_mut(&activity.id) else {
                continue;
            };
            if activity.bell {
                window.pending_alerts |= ALERT_BELL;
            }
            if activity.output {
                window.pending_alerts |= ALERT_ACTIVITY;
            }
            if silence {
                window.pending_alerts |= ALERT_SILENCE;
            }
            // tmux checks bell, then activity, then silence, and only clears
            // the condition whose monitor is on. A condition is examined when
            // something queues it — new output, a bell, the silence timer — or
            // when a client attaches and runs `alerts_check_session`; a
            // monitor merely being switched on does not, which is what leaves
            // creation-time activity pending until the first client arrives.
            let rechecked = recheck.contains(&activity.id);
            for (bit, monitored, raised) in [
                (ALERT_BELL, activity.monitor_bell, activity.bell),
                (ALERT_ACTIVITY, activity.monitor_activity, activity.output),
                (ALERT_SILENCE, activity.monitor_silence != 0, silence),
            ] {
                if monitored && (raised || rechecked) && window.pending_alerts & bit != 0 {
                    window.pending_alerts &= !bit;
                    deliveries.push((activity.id, bit));
                }
            }
        }

        let mut changed = false;
        for (window_id, bit) in deliveries {
            changed |= self.deliver_alert(window_id, bit);
        }
        let live_windows = self.windows.keys().copied().collect::<BTreeSet<_>>();
        self.window_last_activity
            .retain(|window_id, _| live_windows.contains(window_id));
        self.silence_alerted
            .retain(|window_id| live_windows.contains(window_id));
        changed
    }

    /// Apply the clipboard policy to the OSC 52 sequences panes emitted since
    /// the last pass, mirroring tmux's `input_osc_52`.
    ///
    /// Applications may only touch the clipboard under `set-clipboard on`; the
    /// sequence is not even parsed otherwise. A query is then answered from the
    /// newest paste buffer when `get-clipboard` is `buffer`, and forwarded to
    /// the client's terminal under `request`/`both` — which hmux does not do
    /// yet, so those requests go unanswered exactly as they do with no client.
    pub(crate) fn process_pane_clipboard(&mut self) {
        let panes = self
            .windows
            .values()
            .flat_map(|window| window.panes.iter())
            .map(|node| (node.id, node.pane.observation_state()))
            .collect::<Vec<_>>();
        let allow_applications = self.server_options().get("set-clipboard") == Some("on");
        let answer_from_buffer = self.server_options().get("get-clipboard") == Some("buffer");
        for (pane_id, observation) in panes {
            for event in observation.take_clipboard_events() {
                if !allow_applications {
                    continue;
                }
                match event {
                    PaneClipboardEvent::Set { data } => {
                        self.set_buffer(None, &data);
                        let was_deferred =
                            std::mem::replace(&mut self.notifications_are_deferred, true);
                        self.notify_pane("pane-set-clipboard", pane_id);
                        self.notifications_are_deferred = was_deferred;
                    }
                    PaneClipboardEvent::Query {
                        selection,
                        string_terminator,
                    } => {
                        if !answer_from_buffer {
                            continue;
                        }
                        let Some(data) = self.buffer(None).map(<[u8]>::to_vec) else {
                            continue;
                        };
                        let mut reply = format!(
                            "\x1b]52;{selection};{}",
                            super::cmd_send_keys::base64_encode(&data)
                        )
                        .into_bytes();
                        reply.extend_from_slice(if string_terminator {
                            b"\x1b\\"
                        } else {
                            b"\x07"
                        });
                        let _ = self.write_pane_input(pane_id, &reply);
                    }
                }
            }
        }
    }

    /// Answer pending DSR ?996 questions and push theme changes to the panes
    /// that subscribed with DECSET 2031, mirroring tmux's
    /// `window_pane_send_theme_update`.
    pub(crate) fn process_pane_themes(&mut self) {
        let panes = self
            .windows
            .values()
            .flat_map(|window| window.panes.iter())
            .map(|node| (node.id, node.pane.observation_state()))
            .collect::<Vec<_>>();
        for (pane_id, _) in &panes {
            let theme = self.pane_theme(*pane_id);
            let node = self
                .windows
                .values()
                .flat_map(|window| window.panes.iter())
                .find(|node| node.id == *pane_id);
            let Some(node) = node else { continue };
            let queried = node.pane.take_theme_query();
            let subscribed = node.pane.theme_updates_enabled();
            if queried {
                if let Some(theme) = theme {
                    let _ = self.write_pane_input(*pane_id, theme_report(theme));
                }
            }
            // A pane learns of a change only after it subscribes, so the theme
            // in force at subscription time is recorded rather than pushed.
            if !subscribed {
                self.pane_theme_pushed.remove(pane_id);
                continue;
            }
            let previous = self.pane_theme_pushed.get(pane_id).cloned();
            let current = theme.unwrap_or("unknown").to_string();
            match previous {
                None => {
                    self.pane_theme_pushed.insert(*pane_id, current);
                }
                Some(previous) if previous != current => {
                    self.pane_theme_pushed.insert(*pane_id, current);
                    if let Some(theme) = theme {
                        let _ = self.write_pane_input(*pane_id, theme_report(theme));
                    }
                }
                Some(_) => {}
            }
        }
        let live = panes.iter().map(|(id, _)| *id).collect::<BTreeSet<_>>();
        self.pane_theme_pushed.retain(|id, _| live.contains(id));
    }

    /// tmux's `window_pane_get_theme`, restricted to the half hmux can answer:
    /// the pane's own background colour. `None` is tmux's `THEME_UNKNOWN`.
    fn pane_theme(&self, pane_id: u32) -> Option<&'static str> {
        let style = self.option_for_target(&format!("%{pane_id}"), "window-style")?;
        let background = style.split(',').find_map(|part| {
            part.trim()
                .strip_prefix("bg=")
                .and_then(super::style::parse_colour)
        })?;
        super::style::colour_theme(background)
    }

    /// Write bytes into a pane's input as if its terminal had produced them.
    fn write_pane_input(&self, pane_id: u32, bytes: &[u8]) -> io::Result<()> {
        let pane = self
            .windows
            .values()
            .flat_map(|window| window.panes.iter())
            .find(|node| node.id == pane_id)
            .ok_or_else(|| io::Error::other("pane disappeared"))?;
        pane.pane.input(bytes)
    }

    /// tmux's `alerts_check_bell`/`_activity`/`_silence` body: flag every
    /// winlink of the window, then let the session's `*-action` decide whether
    /// the hook and the user-visible notification follow.
    fn deliver_alert(&mut self, window_id: u32, bit: u8) -> bool {
        let (label, action_option, visual_option, hook) = match bit {
            ALERT_BELL => ("Bell", "bell-action", "visual-bell", "alert-bell"),
            ALERT_ACTIVITY => (
                "Activity",
                "activity-action",
                "visual-activity",
                "alert-activity",
            ),
            _ => ("Silence", "silence-action", "visual-silence", "alert-silence"),
        };
        let attached = self.attached_session_ids();
        let links = self
            .sessions
            .iter()
            .enumerate()
            .flat_map(|(session_index, session)| {
                session
                    .windows
                    .iter()
                    .enumerate()
                    .filter(|(_, link)| link.id == window_id)
                    .map(move |(link_index, _)| (session_index, link_index))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut alerted_sessions = BTreeSet::new();
        let mut changed = false;
        for (session_index, link_index) in links {
            let session_id = self.sessions[session_index].id;
            let is_current = self.sessions[session_index].active == link_index;
            let is_attached = attached.contains(&session_id);
            // A bell is announced even on an already-flagged winlink; the
            // other two are not.
            if bit != ALERT_BELL
                && self.sessions[session_index].windows[link_index].alert_flags & bit != 0
            {
                continue;
            }
            if !is_current || !is_attached {
                let link = &mut self.sessions[session_index].windows[link_index];
                if link.alert_flags & bit == 0 {
                    link.alert_flags |= bit;
                    changed = true;
                    self.invalidate_session(session_id, RenderInvalidation::STATUS);
                }
            }
            let applies = match self.sessions[session_index]
                .options(&self.global_options)
                .get(action_option)
                .unwrap_or("other")
            {
                "any" => true,
                "current" => is_current,
                "other" => !is_current,
                _ => false,
            };
            if !applies {
                continue;
            }
            let was_deferred = std::mem::replace(&mut self.notifications_are_deferred, true);
            self.notify_session_window(hook, session_id, window_id);
            self.notifications_are_deferred = was_deferred;
            // One visual notification per session, however many of its
            // winlinks alerted.
            if !alerted_sessions.insert(session_id) {
                continue;
            }
            self.announce_alert(session_index, link_index, label, visual_option);
        }
        changed
    }

    /// tmux's `alerts_set_message`: send each non-control client of the session
    /// a bell, a status message, or both, per `visual-*`.
    fn announce_alert(
        &mut self,
        session_index: usize,
        link_index: usize,
        label: &str,
        visual_option: &str,
    ) {
        let session = &self.sessions[session_index];
        let session_id = session.id;
        let options = session.options(&self.global_options);
        let visual = options.get(visual_option).unwrap_or("off").to_owned();
        let duration_ms = options
            .get("display-time")
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(750);
        let bell = visual != "on";
        let text = (visual != "off").then(|| {
            if session.active == link_index {
                format!("{label} in current window")
            } else {
                format!("{label} in window {}", session.windows[link_index].index)
            }
        });
        self.client_renders
            .announce_alert(session_id, bell, text, duration_ms);
    }

    fn cycle_window(&mut self, session: &str, forward: bool) -> io::Result<()> {
        let pos = self.session_pos(session).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {session}"),
            )
        })?;
        let session_id = self.sessions[pos].id;
        let sess = &mut self.sessions[pos];
        let n = sess.windows.len();
        if n <= 1 {
            let which = if forward { "next" } else { "previous" };
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("no {which} window"),
            ));
        }
        let next = if forward {
            (sess.active + 1) % n
        } else {
            (sess.active + n - 1) % n
        };
        self.select_session_window(pos, next);
        self.resize_active_window_to_session_size(pos)?;
        self.invalidate_session(
            session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
        );
        Ok(())
    }

    /// `last-window`: swap to the previously-active window. Without one (a
    /// single-window session, or a session never switched), tmux reports
    /// `no last window`.
    pub fn last_window(&mut self, session: &str) -> io::Result<()> {
        let pos = self.session_pos(session).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {session}"),
            )
        })?;
        let session_id = self.sessions[pos].id;
        let sess = &mut self.sessions[pos];
        match sess
            .last_windows
            .first()
            .copied()
            .and_then(|id| sess.windows.iter().position(|link| link.link_id == id))
        {
            Some(last) => {
                self.select_session_window(pos, last);
                self.resize_active_window_to_session_size(pos)?;
                self.invalidate_session(
                    session_id,
                    RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
                );
                Ok(())
            }
            _ => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "no last window".to_string(),
            )),
        }
    }

    /// `split-window target`: add a pane to the target window (splitting the
    /// current one, geometry aside — the control plane only observes the pane
    /// set). The new pane becomes the window's active pane, as in tmux.
    /// `split-window`. `before` selects `-b`: the new pane is inserted *before*
    /// the target pane instead of appended after it. Returns the new pane's index
    /// within the window so the caller can render `-P`.
    pub fn split_window(&mut self, target: &str, select: bool, before: bool) -> io::Result<usize> {
        self.split_window_direction(target, select, before, SplitDirection::TopBottom)
    }

    pub(crate) fn split_window_direction(
        &mut self,
        target: &str,
        select: bool,
        before: bool,
        direction: SplitDirection,
    ) -> io::Result<usize> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        self.split_window_direction_with_spawn(target, select, before, direction, &[shell], None)
    }

    pub(crate) fn split_window_direction_with_spawn(
        &mut self,
        target: &str,
        select: bool,
        before: bool,
        direction: SplitDirection,
        argv: &[String],
        cwd: Option<&Path>,
    ) -> io::Result<usize> {
        let spec = match cwd {
            Some(cwd) => PaneSpec::CommandIn(argv.to_vec(), cwd.to_path_buf()),
            None => PaneSpec::Command(argv.to_vec()),
        };
        self.split_window_direction_with_spec(target, select, before, direction, spec)
    }

    pub(crate) fn split_window_direction_with_spec(
        &mut self,
        target: &str,
        select: bool,
        before: bool,
        direction: SplitDirection,
        spec: PaneSpec,
    ) -> io::Result<usize> {
        let (_, pane_part) = split_pane_target(target);
        let t = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[t.session].id;
        let session = &self.sessions[t.session];
        let window = self.window(t.session, t.window);
        let cols = window.manual_size.map_or(session.cols, |size| size.0);
        let rows = window.manual_size.map_or(session.rows, |size| size.1);
        let pane = match spec {
            PaneSpec::Inert => Pane::inert(cols, rows)?,
            PaneSpec::Command(argv) => {
                let refs = argv.iter().map(String::as_str).collect::<Vec<_>>();
                Pane::spawn_in_mode(&refs, None, cols, rows, self.pane_io_mode)?
            }
            PaneSpec::CommandIn(argv, cwd) => {
                let refs = argv.iter().map(String::as_str).collect::<Vec<_>>();
                Pane::spawn_in_mode(&refs, Some(&cwd), cols, rows, self.pane_io_mode)?
            }
        };
        let pane_id = self.next_pane_id;
        self.next_pane_id += 1;
        let win = self.window_mut(t.session, t.window);
        // With `-b` the new pane lands at the target pane's index, pushing it (and
        // everything after) up; otherwise it follows the target. The target
        // pane is the `-t` pane part, or the active pane when none is given.
        let insert_at = if before {
            match pane_part {
                Some(_) => t.pane,
                None => win.active,
            }
        } else {
            t.pane + 1
        };
        let old_active = win.active;
        let target_index = t.pane;
        let target_id = win.panes[target_index].id;
        if !win.layout.split(target_id, pane_id, direction, before) {
            return Err(io::Error::other("target pane is absent from layout"));
        }
        win.panes.insert(
            insert_at,
            PaneNode {
                id: pane_id,
                pane,
                start_command: String::new(),
                input_off: false,
                title: None,
                exit_notified: false,
                mode: None,
                copy: None,
                mode_view: None,
                search_string: None,
                search_regex: false,
                floating: None,
                options: OptionSet::default(),
            },
        );
        resize_panes_to_layout(win)?;
        // Inserting at/at-or-before the old active pane shifts its index up by one.
        let shifted_old = if insert_at <= old_active {
            old_active + 1
        } else {
            old_active
        };
        // `-d` splits in the background: the new pane is added but the original
        // pane stays active. tmux's default is to select the new pane.
        let window_id = win.id;
        let active_changed = if select {
            win.last_pane = Some(shifted_old);
            win.active = insert_at;
            true
        } else {
            win.active = shifted_old;
            false
        };
        self.invalidate_session(
            session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
        );
        if active_changed {
            self.notify_window("window-pane-changed", window_id);
        }
        self.notify_window("window-layout-changed", window_id);
        Ok(insert_at)
    }

    pub(crate) fn new_floating_pane_with_spawn(
        &mut self,
        target: &str,
        select: bool,
        width: Option<u16>,
        height: Option<u16>,
        left: Option<i32>,
        top: Option<i32>,
        argv: &[String],
        cwd: Option<&Path>,
    ) -> io::Result<usize> {
        let target = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[target.session].id;
        let session = &self.sessions[target.session];
        let window = self.window(target.session, target.window);
        let cols = window.manual_size.map_or(session.cols, |size| size.0);
        let rows = window.manual_size.map_or(session.rows, |size| size.1);
        let width = width
            .unwrap_or(cols / 2)
            .clamp(1, cols.saturating_sub(1).max(1));
        let height = height
            .unwrap_or(rows / 4)
            .clamp(1, rows.saturating_sub(1).max(1));
        let refs = argv.iter().map(String::as_str).collect::<Vec<_>>();
        let pane = Pane::spawn_in_mode(&refs, cwd, width, height, self.pane_io_mode)?;
        let pane_id = self.next_pane_id;
        self.next_pane_id += 1;
        let window = self.window_mut(target.session, target.window);
        let left = left.unwrap_or_else(|| {
            let next = if window.last_new_pane_x == 0 || window.last_new_pane_x > cols {
                4
            } else {
                window.last_new_pane_x.saturating_add(4)
            };
            window.last_new_pane_x = next;
            i32::from(next)
        });
        let top = top.unwrap_or_else(|| {
            let next = if window.last_new_pane_y == 0 || window.last_new_pane_y > rows {
                2
            } else {
                window.last_new_pane_y.saturating_add(2)
            };
            window.last_new_pane_y = next;
            i32::from(next)
        });
        let insert_at = target.pane + 1;
        let old_active = window.active;
        window.panes.insert(
            insert_at,
            PaneNode {
                id: pane_id,
                pane,
                start_command: String::new(),
                input_off: false,
                title: None,
                exit_notified: false,
                mode: None,
                copy: None,
                mode_view: None,
                search_string: None,
                search_regex: false,
                floating: Some(PaneRect {
                    top: top.max(0) as u16,
                    left: left.max(0) as u16,
                    height,
                    width,
                }),
                options: OptionSet::default(),
            },
        );
        let shifted_old = if insert_at <= old_active {
            old_active + 1
        } else {
            old_active
        };
        if select {
            window.last_pane = Some(shifted_old);
            window.active = insert_at;
        } else {
            window.active = shifted_old;
        }
        self.invalidate_session(
            session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
        );
        Ok(insert_at)
    }

    pub(crate) fn set_pane_option(&mut self, target: Target, name: &str, value: &str) {
        self.window_mut(target.session, target.window).panes[target.pane]
            .option_overrides_mut()
            .set(name, value);
    }

    /// `select-pane target`: make the target pane active within its window.
    /// `target` is `session[:window].pane`; a missing pane part means the active
    /// pane (a no-op). Reports tmux's `can't find pane: <part>` on a miss.
    pub fn select_pane(&mut self, target: &str) -> io::Result<()> {
        let t = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[t.session].id;
        let win = self.window_mut(t.session, t.window);
        let window_id = win.id;
        let previous_active = win.active;
        if t.pane != win.active {
            win.last_pane = Some(win.active);
            win.active = t.pane;
            self.invalidate_session(
                session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
            self.notify_window("window-pane-changed", window_id);
            // tmux only follows the active pane with focus while
            // `focus-events` is on (`window_set_active_pane`).
            if self.server_option_is_on("focus-events", false) {
                let previous_pane = self.window(t.session, t.window).panes[previous_active].id;
                self.update_pane_focus(previous_pane, false);
                self.update_window_focus(window_id);
            }
        }
        Ok(())
    }

    /// `select-pane -T title`: pin a pane's title, overriding whatever its
    /// terminal reported. Raises `pane-title-changed` when the title moves.
    pub(crate) fn set_pane_title(&mut self, target: &str, title: &str) -> io::Result<()> {
        let t = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[t.session].id;
        let node = &mut self.window_mut(t.session, t.window).panes[t.pane];
        let pane_id = node.id;
        if node.title.as_deref() == Some(title) {
            return Ok(());
        }
        node.title = Some(title.to_string());
        self.invalidate_session(session_id, RenderInvalidation::STATUS);
        self.notify_pane("pane-title-changed", pane_id);
        Ok(())
    }

    /// The title `#{pane_title}` reports: the `select-pane -T` override when
    /// one is set, else what the pane's terminal last announced.
    pub(crate) fn pane_title(&self, node: &PaneNode) -> Option<String> {
        node.title.clone().or_else(|| {
            node.pane
                .observation_state()
                .contract_title()
                .ok()
                .flatten()
        })
    }

    pub(crate) fn set_pane_input_off(&mut self, target: &str, input_off: bool) -> io::Result<()> {
        let target = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        self.window_mut(target.session, target.window).panes[target.pane].input_off = input_off;
        Ok(())
    }

    pub(crate) fn target_pane_ids(&self, target: Target) -> (u32, u32) {
        let window = self.window(target.session, target.window);
        (window.id, window.panes[target.pane].id)
    }

    pub(crate) fn report_pane_control_colour(
        &mut self,
        target: &str,
        report: &[u8],
    ) -> io::Result<()> {
        let target = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        self.window_mut(target.session, target.window).panes[target.pane]
            .pane
            .input(report)?;
        Ok(())
    }

    pub(crate) fn pane_in_direction(
        &self,
        target: &str,
        direction: SplitDirection,
        forward: bool,
    ) -> io::Result<(u32, u32)> {
        let target = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let window = self.window(target.session, target.window);
        let active_id = window.panes[target.pane].id;
        let next_id = window
            .layout
            .neighbour(active_id, direction, forward)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no pane in direction"))?;
        Ok((window.id, next_id))
    }

    /// Select the nearest pane in the requested geometric direction.
    pub(crate) fn select_pane_direction(
        &mut self,
        target: &str,
        direction: SplitDirection,
        forward: bool,
    ) -> io::Result<()> {
        let t = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[t.session].id;
        let win = self.window_mut(t.session, t.window);
        let active_id = win.panes[t.pane].id;
        let next_id = win
            .layout
            .neighbour(active_id, direction, forward)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no pane in direction"))?;
        let next = win
            .panes
            .iter()
            .position(|pane| pane.id == next_id)
            .expect("layout pane belongs to window");
        win.last_pane = Some(win.active);
        win.active = next;
        self.invalidate_session(
            session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
        );
        Ok(())
    }

    /// The server's marked pane id, or `None` if no pane is marked. Self-heals a
    /// stale mark: if the marked pane has since been destroyed the mark reads as
    /// cleared, matching tmux (which drops the mark when the pane goes away), so
    /// `#{pane_marked_set}` never reports a mark that no longer exists.
    pub fn marked_pane(&self) -> Option<u32> {
        let id = self.marked_pane_id?;
        self.windows
            .values()
            .any(|window| window.panes.iter().any(|pane| pane.id == id))
            .then_some(id)
    }

    /// `select-pane -m target`: toggle the server's marked pane. Marking the
    /// already-marked pane clears the mark (tmux's toggle); marking any other pane
    /// moves the mark to it. Does *not* change which pane is active. `target` is
    /// `session[:window].pane` (default: the active pane). Reports tmux's `can't
    /// find pane: <part>` on a miss.
    pub fn mark_pane(&mut self, target: &str) -> io::Result<()> {
        let (win_target, pane_part) = split_pane_target(target);
        let t = self.resolve_window(win_target)?;
        let idx = match pane_part {
            None => self.window(t.session, t.window).active,
            Some(p) => self
                .pane_pos(t.session, t.window, p)
                .ok_or_else(|| pane_not_found(p))?,
        };
        let win = self.window(t.session, t.window);
        let id = win.panes[idx].id;
        self.marked_pane_id = if self.marked_pane_id == Some(id) {
            None
        } else {
            Some(id)
        };
        Ok(())
    }

    /// `select-pane -M`: clear the server's marked pane unconditionally.
    pub fn clear_mark(&mut self) {
        self.marked_pane_id = None;
    }

    /// Start or stop a pipe attached to the selected pane's PTY.
    pub fn pipe_pane(
        &mut self,
        target: &str,
        command: Option<&str>,
        only_toggle: bool,
        input: bool,
        output: bool,
    ) -> io::Result<()> {
        let (win_target, pane_part) = split_pane_target(target);
        let t = self.resolve_window(win_target)?;
        let idx = match pane_part {
            None => self.window(t.session, t.window).active,
            Some(p) => self
                .pane_pos(t.session, t.window, p)
                .ok_or_else(|| pane_not_found(p))?,
        };
        let win = self.window_mut(t.session, t.window);
        let node = &mut win.panes[idx];
        let had_pipe = node.pane.pipe_active();
        node.pane.close_pipe();
        if let Some(command) = command.filter(|command| !command.is_empty()) {
            if !(only_toggle && had_pipe) {
                node.pane.open_pipe(command, input, output)?;
            }
        }
        Ok(())
    }

    /// `kill-pane target`: remove the target pane. Removing a window's last pane
    /// destroys the window (and the session if it was the last window), matching
    /// tmux. `target` is `session[:window].pane` (default: the active pane).
    pub fn kill_pane(&mut self, target: &str) -> io::Result<()> {
        let t = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[t.session].id;
        let window_id = self.sessions[t.session].windows[t.window].id;
        let pane_idx = t.pane;
        let window_empty = {
            let win = self.window_mut(t.session, t.window);
            let pane_id = win.panes[pane_idx].id;
            win.panes.remove(pane_idx);
            win.layout.remove(pane_id);
            win.panes.is_empty()
        };
        if window_empty {
            self.destroy_window_id(window_id);
            return Ok(());
        } else {
            let win = self.window_mut(t.session, t.window);
            if pane_idx < win.active {
                win.active -= 1;
            } else if win.active >= win.panes.len() {
                win.active = win.panes.len() - 1;
            }
            win.last_pane = win
                .last_pane
                .and_then(|last| (last != pane_idx).then_some(last - usize::from(last > pane_idx)));
            resize_panes_to_layout(win)?;
        }
        if self.sessions.iter().any(|session| session.id == session_id) {
            self.invalidate_session(
                session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        } else {
            self.invalidate_session(session_id, RenderInvalidation::SESSION_GONE);
        }
        self.notify_window("window-layout-changed", window_id);
        Ok(())
    }

    /// `kill-pane -a -t target`: remove every pane in the target window except
    /// the target itself. The survivor is renumbered to pane index zero.
    pub fn kill_other_panes(&mut self, target: &str) -> io::Result<()> {
        let t = self.resolve(target).ok_or_else(|| {
            let pane = split_pane_target(target).1.unwrap_or(target);
            pane_not_found(pane)
        })?;
        let session_id = self.sessions[t.session].id;
        let win = self.window_mut(t.session, t.window);
        let survivor = win.panes.remove(t.pane);
        let survivor_id = survivor.id;
        win.panes.clear();
        win.panes.push(survivor);
        win.layout.keep_only(survivor_id);
        resize_panes_to_layout(win)?;
        win.active = 0;
        win.last_pane = None;
        self.invalidate_session(
            session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
        );
        Ok(())
    }

    /// `swap-window -s src -t dst`: exchange the two windows' contents, keeping
    /// each window at its original index (tmux swaps the winlinks' windows). Both
    /// targets are `session[:window]`; a missing window part means the session's
    /// active window.
    pub fn swap_window(&mut self, src: &str, dst: &str, select: bool) -> io::Result<()> {
        let a = self.resolve_window(src)?;
        let b = self.resolve_window(dst)?;
        if a.session != b.session
            && self.sessions[a.session].link_set_id == self.sessions[b.session].link_set_id
        {
            return Err(io::Error::other("can't move window, sessions are grouped"));
        }
        let src_session_id = self.sessions[a.session].id;
        let dst_session_id = self.sessions[b.session].id;
        // tmux inverts the usual `-d` sense here: the plain form leaves every
        // session's current window untouched, and `-d` selects the destination
        // slot (and, for a cross-session swap, the source slot in its session).
        let src_index = self.sessions[a.session].windows[a.window].index;
        let dst_index = self.sessions[b.session].windows[b.window].index;
        if self.sessions[a.session].windows[a.window].id
            == self.sessions[b.session].windows[b.window].id
        {
            return Ok(());
        }
        if self.sessions[a.session].link_set_id == self.sessions[b.session].link_set_id {
            let mut links = self.sessions[a.session].windows.clone();
            let first = links[a.window].id;
            links[a.window].id = links[b.window].id;
            links[b.window].id = first;
            self.replace_link_set_preserving_positions(a.session, links);
            if select {
                Self::select_window_index(&mut self.sessions[a.session], dst_index);
            }
            self.invalidate_session(
                src_session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
            return Ok(());
        } else {
            let mut a_links = self.sessions[a.session].windows.clone();
            let mut b_links = self.sessions[b.session].windows.clone();
            std::mem::swap(&mut a_links[a.window].id, &mut b_links[b.window].id);
            self.replace_link_set_preserving_positions(a.session, a_links);
            self.replace_link_set_preserving_positions(b.session, b_links);
            if select {
                Self::select_window_index(&mut self.sessions[b.session], dst_index);
                Self::select_window_index(&mut self.sessions[a.session], src_index);
            }
        }
        self.invalidate_session(
            src_session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
        );
        self.invalidate_session(
            dst_session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
        );
        Ok(())
    }

    /// Point a session's current window at the window occupying `index`,
    /// recording the outgoing window as its last-active. A no-op if no window
    /// sits at `index` or it is already current.
    fn select_window_index(sess: &mut Session, index: u32) {
        let Some(new_pos) = sess.windows.iter().position(|w| w.index == index) else {
            return;
        };
        Self::select_window_position(sess, new_pos);
    }

    /// `move-window -s src -t dst`: renumber the src window to the dst index
    /// (optionally into another session). Errors if the destination index is
    /// already in use (tmux's `index in use: N`), matching real
    /// tmux without `-k`.
    /// Move a window to `dst`. `select` follows tmux's default of making the
    /// moved window the destination session's current window; `-d` passes
    /// `false` to leave the current window where it is.
    pub fn move_window(&mut self, src: &str, dst: &str, select: bool) -> io::Result<()> {
        self.move_window_impl(src, dst, false, select)
    }

    /// Move a window, replacing an occupied destination for `move-window -k`.
    pub(crate) fn move_window_replacing(
        &mut self,
        src: &str,
        dst: &str,
        select: bool,
    ) -> io::Result<()> {
        self.move_window_impl(src, dst, true, select)
    }

    /// `move-window -a`/`-b`: relocate the source window relative to an anchor
    /// window in the destination session — `after` places it after the anchor
    /// (`-a`), else before it (`-b`). Mirrors tmux's `winlink_shuffle_up`: the
    /// contiguous occupied run at/above the desired index (the source counted as
    /// occupied) is shifted up by one to open the slot, then the source is
    /// relocated into it. When the anchor index does not name an existing window,
    /// tmux ignores `-a`/`-b` and moves to the explicit index (the plain path).
    pub fn move_window_relative(
        &mut self,
        src: &str,
        dst: &str,
        after: bool,
        select: bool,
    ) -> io::Result<()> {
        let s = self.resolve_window(src)?;
        let src_session = s.session;
        let src_session_id = self.sessions[src_session].id;
        let moved_link_id = self.sessions[src_session].windows[s.window].link_id;
        let (dst_sess_name, dst_idx) = parse_index_target(dst);
        let mut dst_session = match dst_sess_name {
            Some(name) => self.session_pos(name).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("can't find session: {name}"),
                )
            })?,
            None => src_session,
        };
        let dst_session_id = self.sessions[dst_session].id;
        if src_session != dst_session
            && self.sessions[src_session].link_set_id == self.sessions[dst_session].link_set_id
        {
            return Err(io::Error::other("sessions are grouped"));
        }
        let anchor = dst_idx.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "move-window: missing destination index".to_string(),
            )
        })?;
        // No such anchor window → tmux ignores the relative flag and moves to the
        // explicit index.
        if !self.sessions[dst_session]
            .windows
            .iter()
            .any(|w| w.index == anchor)
        {
            return self.move_window_impl(src, dst, false, select);
        }
        let desired = if after { anchor + 1 } else { anchor };
        // First free index at or above `desired` (end of the contiguous run to
        // shift). The source counts as occupied, matching tmux.
        let mut free = desired;
        while self.sessions[dst_session]
            .windows
            .iter()
            .any(|w| w.index == free)
        {
            free += 1;
        }
        let source_set = self.sessions[src_session].link_set_id;
        let destination_set = self.sessions[dst_session].link_set_id;
        if source_set == destination_set {
            let mut links = self.sessions[src_session].windows.clone();
            for link in &mut links {
                if link.index >= desired && link.index < free {
                    link.index += 1;
                }
            }
            let source = links
                .iter()
                .position(|link| link.link_id == moved_link_id)
                .expect("source window present after the shuffle");
            let mut link = links.remove(source);
            link.index = desired;
            links.push(link);
            links.sort_by_key(|link| link.index);
            self.replace_link_set(src_session, links);
        } else {
            let mut source_links = self.sessions[src_session].windows.clone();
            let source = source_links
                .iter()
                .position(|link| link.link_id == moved_link_id)
                .expect("source window present");
            let mut link = source_links.remove(source);
            link.index = desired;
            self.replace_link_set(src_session, source_links);

            let mut destination_links = self.sessions[dst_session].windows.clone();
            for existing in &mut destination_links {
                if existing.index >= desired && existing.index < free {
                    existing.index += 1;
                }
            }
            destination_links.push(link);
            destination_links.sort_by_key(|link| link.index);
            self.replace_link_set(dst_session, destination_links);

            if self.sessions[src_session].windows.is_empty() {
                self.sessions
                    .retain(|session| session.link_set_id != source_set);
                if src_session < dst_session {
                    dst_session -= 1;
                }
            }
        }
        if select {
            let sess = &mut self.sessions[dst_session];
            let new_pos = sess
                .windows
                .iter()
                .position(|w| w.link_id == moved_link_id)
                .expect("moved window is present in the destination");
            Self::select_window_position(sess, new_pos);
        }
        self.remove_unlinked_windows();
        if self
            .sessions
            .iter()
            .any(|session| session.id == src_session_id)
        {
            self.invalidate_session(
                src_session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        } else {
            self.invalidate_session(src_session_id, RenderInvalidation::SESSION_GONE);
        }
        if dst_session_id != src_session_id {
            self.invalidate_session(
                dst_session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        }
        Ok(())
    }

    fn move_window_impl(
        &mut self,
        src: &str,
        dst: &str,
        replace: bool,
        select: bool,
    ) -> io::Result<()> {
        let s = self.resolve_window(src)?;
        let src_session_id = self.sessions[s.session].id;
        let (dst_session, new_index) = self.window_destination(dst, s.session)?;
        let dst_session_id = self.sessions[dst_session].id;
        if s.session != dst_session
            && self.sessions[s.session].link_set_id == self.sessions[dst_session].link_set_id
        {
            return Err(io::Error::other("sessions are grouped"));
        }
        // A window already at the destination index blocks the move.
        let occupied = self.sessions[dst_session]
            .windows
            .iter()
            .any(|w| w.index == new_index);
        let same_slot = dst_session == s.session
            && self.sessions[s.session].windows[s.window].index == new_index;
        let source_window_id = self.sessions[s.session].windows[s.window].id;
        if let Some(destination) = self.sessions[dst_session]
            .windows
            .iter()
            .find(|link| link.index == new_index)
        {
            if destination.id == source_window_id {
                return Err(io::Error::other(format!("same index: {new_index}")));
            }
        }
        if occupied && !replace {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("index in use: {new_index}"),
            ));
        }
        let moved_link_id = self.sessions[s.session].windows[s.window].link_id;
        let source_set = self.sessions[s.session].link_set_id;
        let destination_set = self.sessions[dst_session].link_set_id;
        if source_set == destination_set {
            let mut links = self.sessions[s.session].windows.clone();
            if occupied && !same_slot {
                links.retain(|link| link.index != new_index);
            }
            let moved = links
                .iter_mut()
                .find(|link| link.link_id == moved_link_id)
                .expect("source window remains present");
            moved.index = new_index;
            links.sort_by_key(|link| link.index);
            self.replace_link_set(s.session, links);
        } else {
            let mut source_links = self.sessions[s.session].windows.clone();
            let source_position = source_links
                .iter()
                .position(|link| link.link_id == moved_link_id)
                .expect("source window present");
            let mut moved = source_links.remove(source_position);
            moved.index = new_index;
            self.replace_link_set(s.session, source_links);

            let mut destination_links = self.sessions[dst_session].windows.clone();
            if occupied && !same_slot {
                destination_links.retain(|link| link.index != new_index);
            }
            destination_links.push(moved);
            destination_links.sort_by_key(|link| link.index);
            self.replace_link_set(dst_session, destination_links);
            if self.sessions[s.session].windows.is_empty() {
                self.sessions
                    .retain(|session| session.link_set_id != source_set);
            }
        }
        let dst_session = self
            .sessions
            .iter()
            .position(|session| session.id == dst_session_id)
            .expect("destination session remains present");
        if select {
            let sess = &mut self.sessions[dst_session];
            let new_pos = sess
                .windows
                .iter()
                .position(|w| w.link_id == moved_link_id)
                .expect("moved window is present in the destination");
            Self::select_window_position(sess, new_pos);
        }
        self.remove_unlinked_windows();
        if self
            .sessions
            .iter()
            .any(|session| session.id == src_session_id)
        {
            self.invalidate_session(
                src_session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        } else {
            self.invalidate_session(src_session_id, RenderInvalidation::SESSION_GONE);
        }
        if dst_session_id != src_session_id {
            self.invalidate_session(
                dst_session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        }
        Ok(())
    }

    /// `move-window -r`: renumber every window in `session` to consecutive
    /// indices starting at `base-index`, in their current order, closing any
    /// gaps. Errors if the session is missing (`can't find session`).
    pub fn renumber_windows(&mut self, session: &str) -> io::Result<()> {
        let pos = self.session_pos(session).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {session}"),
            )
        })?;
        self.renumber_windows_at(pos);
        Ok(())
    }

    fn renumber_affected_sessions(&mut self, session_ids: &[u32]) {
        let mut link_sets = BTreeSet::new();
        for session_id in session_ids {
            let Some(pos) = self
                .sessions
                .iter()
                .position(|session| session.id == *session_id)
            else {
                continue;
            };
            if !link_sets.insert(self.sessions[pos].link_set_id) {
                continue;
            }
            let enabled = self.sessions[pos]
                .options(&self.global_options)
                .get("renumber-windows")
                == Some("on");
            if enabled {
                self.renumber_windows_at(pos);
            }
        }
    }

    fn renumber_windows_at(&mut self, pos: usize) {
        let base = self.sessions[pos]
            .options(&self.global_options)
            .get("base-index")
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        let session_id = self.sessions[pos].id;
        let active_id = self.sessions[pos]
            .windows
            .get(self.sessions[pos].active)
            .map(|link| link.link_id);
        let mut links = self.sessions[pos].windows.clone();
        // Winlinks are kept sorted by index, so position order is index order.
        for (i, win) in links.iter_mut().enumerate() {
            win.index = base + i as u32;
        }
        self.sessions[pos].windows = links;
        self.sessions[pos].active = active_id
            .and_then(|id| {
                self.sessions[pos]
                    .windows
                    .iter()
                    .position(|link| link.link_id == id)
            })
            .unwrap_or(0);
        Self::refresh_last_window(&mut self.sessions[pos]);
        self.invalidate_session(session_id, RenderInvalidation::STATUS);
    }

    /// `last-pane [-t window]`: make the previously-active pane active. Errors
    /// with `no last pane` when the window has no recorded previous pane.
    pub fn last_pane(&mut self, target: &str) -> io::Result<()> {
        let t = self.resolve_window(target)?;
        let session_id = self.sessions[t.session].id;
        let win = self.window_mut(t.session, t.window);
        match win.last_pane.filter(|&p| p < win.panes.len()) {
            Some(lp) => {
                win.last_pane = Some(win.active);
                win.active = lp;
                self.invalidate_session(
                    session_id,
                    RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
                );
                Ok(())
            }
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "no last pane".to_string(),
            )),
        }
    }

    /// `move-pane -s src -t dst`: move the source pane into the destination
    /// window after the target in pane-list order, becoming active. In tmux
    /// 3.7b, `-b` changes layout geometry but not this observable list order.
    /// Emptying the source window destroys it. Reports `can't find pane` on a
    /// miss.
    pub fn move_pane(&mut self, src: &str, dst: &str, before: bool) -> io::Result<()> {
        let s = self.resolve(src).ok_or_else(|| pane_not_found(src))?;
        let (dst_window_target, _) = split_pane_target(dst);
        // Validate the destination window exists before mutating the source.
        let initial_dst = self
            .resolve_window(dst_window_target)
            .map_err(|_| pane_not_found(dst))?;
        let src_session_id = self.sessions[s.session].id;
        let dst_session_id = self.sessions[initial_dst.session].id;
        let source_window_id = self.sessions[s.session].windows[s.window].id;
        // Capture the target pane id before removing the source because indices
        // and even window positions can shift during the mutation.
        let target_id = self
            .resolve(dst)
            .map(|t| self.window(t.session, t.window).panes[t.pane].id);
        let source_pane_id = self.window(s.session, s.window).panes[s.pane].id;
        if target_id == Some(source_pane_id) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "source and target panes must be different",
            ));
        }
        // Pull the pane out of the source window.
        let (node, source_empty) = {
            let win = self.window_mut(s.session, s.window);
            let node = win.panes.remove(s.pane);
            win.layout.remove(node.id);
            let empty = win.panes.is_empty();
            if !empty {
                if win.active >= win.panes.len() {
                    win.active = win.panes.len() - 1;
                }
                win.last_pane = None;
                resize_panes_to_layout(win)?;
            }
            (node, empty)
        };
        if source_empty {
            self.destroy_window_id(source_window_id);
        }
        // The destination window position may have shifted if the source window
        // (before it) was removed from the same session; re-resolve by target.
        let d = self
            .resolve_window(dst_window_target)
            .map_err(|_| pane_not_found(dst))?;
        let win = self.window_mut(d.session, d.window);
        let layout_target_id =
            target_id.unwrap_or_else(|| win.panes.get(win.active).map_or(node.id, |pane| pane.id));
        // tmux 3.7b inserts after the target in its pane queue even for `-b`.
        // The relocated pane becomes active either way.
        let insert_at = match target_id {
            Some(id) => win
                .panes
                .iter()
                .position(|p| p.id == id)
                .map(|index| index + 1)
                .unwrap_or(win.panes.len()),
            None => win.panes.len(),
        };
        win.last_pane = Some(win.active);
        win.layout
            .split(layout_target_id, node.id, SplitDirection::TopBottom, before);
        win.panes.insert(insert_at, node);
        win.active = insert_at;
        resize_panes_to_layout(win)?;
        self.remove_unlinked_windows();
        if self
            .sessions
            .iter()
            .any(|session| session.id == src_session_id)
        {
            self.invalidate_session(
                src_session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        } else {
            self.invalidate_session(src_session_id, RenderInvalidation::SESSION_GONE);
        }
        if dst_session_id != src_session_id {
            self.invalidate_session(
                dst_session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        }
        Ok(())
    }

    /// Dump the plain-text screen of the pane named by `target` (`capture-pane`).
    pub fn dump_pane(&self, target: &str) -> io::Result<String> {
        let t = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        self.window(t.session, t.window).panes[t.pane].pane.dump()
    }

    pub(crate) fn dump_pane_vt_rows(
        &self,
        target: &str,
        start: usize,
        rows: usize,
    ) -> io::Result<Vec<u8>> {
        let t = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        self.window(t.session, t.window).panes[t.pane]
            .pane
            .dump_rows_vt(start, rows)
    }

    /// `swap-pane -s src -t dst`: exchange the two panes' positions. Same-window
    /// swaps are a simple positional swap; cross-window swaps exchange the pane
    /// nodes. Reports `can't find pane` on a miss.
    pub fn swap_pane(&mut self, src: &str, dst: &str) -> io::Result<()> {
        let a = self.resolve(src).ok_or_else(|| pane_not_found(src))?;
        let b = self.resolve(dst).ok_or_else(|| pane_not_found(dst))?;
        let src_session_id = self.sessions[a.session].id;
        let dst_session_id = self.sessions[b.session].id;
        let first_window_id = self.sessions[a.session].windows[a.window].id;
        let second_window_id = self.sessions[b.session].windows[b.window].id;
        let first_id = self.window(a.session, a.window).panes[a.pane].id;
        let second_id = self.window(b.session, b.window).panes[b.pane].id;
        if first_window_id == second_window_id {
            let win = self.window_mut(a.session, a.window);
            win.panes.swap(a.pane, b.pane);
            win.layout.swap_panes(first_id, second_id);
            resize_panes_to_layout(win)?;
            self.invalidate_session(src_session_id, RenderInvalidation::LAYOUT);
            return Ok(());
        }
        // Cross-window/session: exchange the two pane nodes by value. Windows
        // live in the arena, so temporarily remove both to obtain disjoint
        // mutable ownership without tying it to session placement.
        let mut first = self
            .windows
            .remove(&first_window_id)
            .expect("window present");
        let mut second = self
            .windows
            .remove(&second_window_id)
            .expect("window present");
        std::mem::swap(&mut first.panes[a.pane], &mut second.panes[b.pane]);
        first.layout.replace_pane(first_id, second_id);
        second.layout.replace_pane(second_id, first_id);
        resize_panes_to_layout(&mut first)?;
        resize_panes_to_layout(&mut second)?;
        self.windows.insert(first_window_id, first);
        self.windows.insert(second_window_id, second);
        self.invalidate_session(src_session_id, RenderInvalidation::LAYOUT);
        if dst_session_id != src_session_id {
            self.invalidate_session(dst_session_id, RenderInvalidation::LAYOUT);
        }
        Ok(())
    }

    /// `swap-pane -U`/`-D`: swap the target pane (default active) with its previous
    /// (`-U`) or next (`-D`) neighbour in the same window, wrapping at the ends.
    /// The targeted pane stays active, following its swap to the neighbour's former
    /// index (matching tmux, which keeps the active flag on the target pane).
    pub(super) fn swap_pane_neighbour(&mut self, target: &str, down: bool) -> io::Result<()> {
        let t = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[t.session].id;
        let win = self.window_mut(t.session, t.window);
        let n = win.panes.len();
        if n > 1 {
            let neighbour = if down {
                (t.pane + 1) % n
            } else {
                (t.pane + n - 1) % n
            };
            let first_id = win.panes[t.pane].id;
            let second_id = win.panes[neighbour].id;
            win.panes.swap(t.pane, neighbour);
            win.layout.swap_panes(first_id, second_id);
            resize_panes_to_layout(win)?;
            win.active = neighbour;
            self.invalidate_session(session_id, RenderInvalidation::LAYOUT);
        }
        Ok(())
    }

    /// `rotate-window [-t target]`: rotate the target window's panes upward (each
    /// pane moves to the previous position; the first wraps to the last). Real
    /// tmux keeps the active pane at the *same index* — the panes rotate under the
    /// cursor — so the pane content selected changes. Default target is the
    /// current window.
    pub fn rotate_window(&mut self, target: &str) -> io::Result<()> {
        self.rotate_window_direction(target, false)
    }

    /// `rotate-window -D [-t target]`: rotate the target window's panes downward,
    /// likewise keeping the active pane at the same index.
    pub(super) fn rotate_window_down(&mut self, target: &str) -> io::Result<()> {
        self.rotate_window_direction(target, true)
    }

    fn rotate_window_direction(&mut self, target: &str, down: bool) -> io::Result<()> {
        let t = self.resolve_window(target)?;
        let session_id = self.sessions[t.session].id;
        let win = self.window_mut(t.session, t.window);
        let n = win.panes.len();
        if n > 1 {
            // tmux rotates the pane *contents* while the active pane stays at its
            // current index (`win.active` is left unchanged): the cursor holds
            // still and the panes move beneath it.
            if down {
                win.panes.rotate_right(1);
            } else {
                win.panes.rotate_left(1);
            }
            let mut pane_ids = win.panes.iter().map(|pane| pane.id);
            win.layout.assign_panes(&mut pane_ids);
            resize_panes_to_layout(win)?;
            self.invalidate_session(session_id, RenderInvalidation::LAYOUT);
        }
        Ok(())
    }

    /// `link-window -s src -t dst`: make the source window appear in the
    /// destination session at the destination index. Both winlinks reference
    /// the same globally owned window.
    /// Link the source window into the destination session at the destination
    /// index. `select` follows tmux's default of making the linked window the
    /// destination session's current window (the previously-active window becomes
    /// its "last"); `-d` passes `false` to leave the current window where it is.
    pub fn link_window(
        &mut self,
        src: &str,
        dst: &str,
        kill: bool,
        select: bool,
    ) -> io::Result<()> {
        let s = self.resolve_window(src)?;
        let source_window_id = self.sessions[s.session].windows[s.window].id;
        let (dst_session, index) = self.window_destination(dst, s.session)?;
        if s.session != dst_session
            && self.sessions[s.session].link_set_id == self.sessions[dst_session].link_set_id
        {
            return Err(io::Error::other("sessions are grouped"));
        }
        if let Some(pos) = self.sessions[dst_session]
            .windows
            .iter()
            .position(|w| w.index == index)
        {
            if self.sessions[dst_session].windows[pos].id == source_window_id {
                return Err(io::Error::other(format!("same index: {index}")));
            }
            if kill {
                self.remove_link(dst_session, pos);
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("index in use: {index}"),
                ));
            }
        }
        self.link_window_at(dst_session, index, source_window_id, select)
    }

    /// Resolve a plain link/move destination. An omitted index selects the
    /// lowest free index at or above `base-index`, matching tmux's `session:`
    /// destination behavior.
    fn window_destination(
        &self,
        target: &str,
        fallback_session: usize,
    ) -> io::Result<(usize, u32)> {
        let (session_name, requested_index) = parse_index_target(target);
        let session = match session_name {
            Some(name) => self.session_pos(name).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("can't find session: {name}"),
                )
            })?,
            None => fallback_session,
        };
        let index = requested_index.unwrap_or_else(|| {
            let destination = &self.sessions[session];
            let mut index = destination
                .options(&self.global_options)
                .get("base-index")
                .and_then(|value| value.parse().ok())
                .unwrap_or(0);
            while destination.windows.iter().any(|link| link.index == index) {
                index += 1;
            }
            index
        });
        Ok((session, index))
    }

    /// `link-window -a`/`-b`: link the source window relative to an anchor window
    /// in the destination session — `after` links it after the anchor (`-a`), else
    /// at the anchor's own index (`-b`). Mirrors tmux's `winlink_shuffle_up`: the
    /// contiguous occupied run at/above the desired index is shifted up by one to
    /// open the slot, then the new link is added there. Unlike `move-window -a`, the
    /// source keeps its own slot (this is a link, not a move). When the anchor index
    /// does not name an existing window, tmux ignores `-a`/`-b` and links at the
    /// explicit index (the plain path).
    pub fn link_window_relative(
        &mut self,
        src: &str,
        dst: &str,
        after: bool,
        select: bool,
    ) -> io::Result<()> {
        let s = self.resolve_window(src)?;
        let source_window_id = self.sessions[s.session].windows[s.window].id;
        let (dst_sess_name, dst_idx) = parse_index_target(dst);
        let dst_session = match dst_sess_name {
            Some(name) => self.session_pos(name).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("can't find session: {name}"),
                )
            })?,
            None => s.session,
        };
        if s.session != dst_session
            && self.sessions[s.session].link_set_id == self.sessions[dst_session].link_set_id
        {
            return Err(io::Error::other("sessions are grouped"));
        }
        let anchor = dst_idx.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "link-window: missing destination index".to_string(),
            )
        })?;
        // No such anchor window → tmux ignores the relative flag and links at the
        // explicit index.
        if !self.sessions[dst_session]
            .windows
            .iter()
            .any(|w| w.index == anchor)
        {
            return self.link_window(src, dst, false, select);
        }
        let desired = if after { anchor + 1 } else { anchor };
        // First free index at or above `desired` (end of the contiguous run to
        // shift).
        let mut free = desired;
        while self.sessions[dst_session]
            .windows
            .iter()
            .any(|w| w.index == free)
        {
            free += 1;
        }
        // Shift the contiguous occupied run [desired, free) up by one to open the
        // `desired` slot, matching tmux's shuffle.
        for win in self.links_mut(dst_session) {
            if win.index >= desired && win.index < free {
                win.index += 1;
            }
        }
        self.link_window_at(dst_session, desired, source_window_id, select)
    }

    /// Add a winlink to an existing window into `dst_session` at `index`,
    /// keeping the session sorted by index. When
    /// `select`, the new link becomes the session's current window and the
    /// previously-active window becomes its "last" (tmux's default; `-d` suppresses
    /// it). The caller must have already opened the `index` slot.
    fn link_window_at(
        &mut self,
        dst_session: usize,
        index: u32,
        window_id: u32,
        select: bool,
    ) -> io::Result<()> {
        let session_id = self.sessions[dst_session].id;
        let winlink_id = self.next_winlink_id;
        self.next_winlink_id += 1;
        let position = self.sessions[dst_session]
            .windows
            .iter()
            .take_while(|link| link.index < index)
            .count();
        self.insert_link(
            dst_session,
            position,
            Winlink {
                link_id: winlink_id,
                index,
                id: window_id,
                alert_flags: 0,
            },
            select,
        );
        self.remove_unlinked_windows();
        self.invalidate_session(
            session_id,
            if select {
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS
            } else {
                RenderInvalidation::STATUS
            },
        );
        Ok(())
    }

    /// `break-pane`: move a pane into a new window in `dst`. A one-pane source
    /// moves its existing window link; a multi-pane source creates a new
    /// physical window. `relative` is `Some(true)` for `-a`, `Some(false)` for
    /// `-b`, and `None` for an explicit or automatically allocated index.
    pub fn break_pane(
        &mut self,
        src: &str,
        dst: &str,
        name: Option<&str>,
        select: bool,
        relative: Option<bool>,
    ) -> io::Result<Target> {
        let t = self.resolve(src).ok_or_else(|| pane_not_found(src))?;
        let source_session_id = self.sessions[t.session].id;
        let source_window_id = self.sessions[t.session].windows[t.window].id;
        let source_index = self.sessions[t.session].windows[t.window].index;
        let source_session_name = self.sessions[t.session].name.clone();
        let source_target = format!("{source_session_name}:{source_index}");
        let (dst_name, requested_index) = parse_index_target(dst);
        let dst_session = match dst_name {
            Some(name) => self.session_pos(name).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("can't find session: {name}"),
                )
            })?,
            None => self.current_session_pos()?,
        };
        let dst_session_id = self.sessions[dst_session].id;
        let anchor = requested_index
            .and_then(|index| {
                self.sessions[dst_session]
                    .windows
                    .iter()
                    .any(|link| link.index == index)
                    .then_some(index)
            })
            .unwrap_or_else(|| {
                self.sessions[dst_session].windows[self.sessions[dst_session].active].index
            });

        if self.window(t.session, t.window).panes.len() == 1 {
            let destination = if relative.is_some() {
                format!("{}:{anchor}", self.sessions[dst_session].name)
            } else {
                let index = match requested_index {
                    Some(index) => index,
                    None => {
                        let mut index = self.sessions[dst_session]
                            .options(&self.global_options)
                            .get("base-index")
                            .and_then(|value| value.parse().ok())
                            .unwrap_or(0);
                        while self.sessions[dst_session]
                            .windows
                            .iter()
                            .any(|link| link.index == index)
                        {
                            index += 1;
                        }
                        index
                    }
                };
                format!("{}:{index}", self.sessions[dst_session].name)
            };
            if let Some(after) = relative {
                self.move_window_relative(&source_target, &destination, after, select)?;
            } else {
                self.move_window(&source_target, &destination, select)?;
            }
            if let Some(name) = name {
                if let Some(window) = self.windows.get_mut(&source_window_id) {
                    window.name = name.to_string();
                    window.options.set("automatic-rename", "off");
                }
            }
            let session = self
                .sessions
                .iter()
                .position(|session| session.id == dst_session_id)
                .ok_or_else(|| io::Error::other("destination session was destroyed"))?;
            let window = self.sessions[session]
                .windows
                .iter()
                .position(|link| link.id == source_window_id)
                .ok_or_else(|| io::Error::other("moved window is not in destination"))?;
            return Ok(Target {
                session,
                window,
                pane: 0,
            });
        }
        let index = if let Some(after) = relative {
            let desired = if after { anchor + 1 } else { anchor };
            let mut free = desired;
            while self.sessions[dst_session]
                .windows
                .iter()
                .any(|link| link.index == free)
            {
                free += 1;
            }
            for link in self.links_mut(dst_session) {
                if link.index >= desired && link.index < free {
                    link.index += 1;
                }
            }
            desired
        } else if let Some(index) = requested_index {
            if self.sessions[dst_session]
                .windows
                .iter()
                .any(|link| link.index == index)
            {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("index in use: {index}"),
                ));
            }
            index
        } else {
            let mut index = self.sessions[dst_session]
                .options(&self.global_options)
                .get("base-index")
                .and_then(|value| value.parse().ok())
                .unwrap_or(0);
            while self.sessions[dst_session]
                .windows
                .iter()
                .any(|link| link.index == index)
            {
                index += 1;
            }
            index
        };
        let source_size = (self.sessions[t.session].cols, self.sessions[t.session].rows);
        let node = self.window_mut(t.session, t.window).panes.remove(t.pane);
        let node_id = node.id;
        self.window_mut(t.session, t.window).layout.remove(node_id);
        // Fix the source window's active pane after the removal.
        let win = self.window_mut(t.session, t.window);
        if win.active >= win.panes.len() {
            win.active = win.panes.len() - 1;
        }
        win.last_pane = None;
        let window_id = self.next_window_id;
        self.next_window_id += 1;
        let winlink_id = self.next_winlink_id;
        self.next_winlink_id += 1;
        let pos = self.sessions[dst_session]
            .windows
            .iter()
            .take_while(|w| w.index < index)
            .count();
        let mut window_options = OptionSet::default();
        if name.is_some() {
            window_options.set("automatic-rename", "off");
        }
        self.windows.insert(
            window_id,
            Window {
                id: window_id,
                name: name.unwrap_or("").to_string(),
                panes: vec![node],
                active: 0,
                last_pane: None,
                zoomed: false,
                manual_size: None,
                layout: LayoutCell::pane(node_id, source_size.0, source_size.1),
                last_layout: None,
                last_new_pane_x: 0,
                last_new_pane_y: 0,
                // tmux's `window_create` raises activity, so a monitor turned on
                // later still sees the window as having been active.
                pending_alerts: ALERT_ACTIVITY,
                options: window_options,
            },
        );
        self.insert_link(
            dst_session,
            pos,
            Winlink {
                link_id: winlink_id,
                index,
                id: window_id,
                alert_flags: 0,
            },
            select,
        );
        self.invalidate_session(
            source_session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
        );
        if dst_session_id != source_session_id {
            self.invalidate_session(
                dst_session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        }
        Ok(Target {
            session: dst_session,
            window: pos,
            pane: 0,
        })
    }

    /// `set-buffer [-b name] data`: store a paste buffer. Without a name, a fresh
    /// auto-named buffer (`buffer0`, `buffer1`, …) is pushed to the front; with a
    /// name, that buffer's data is replaced (or created).
    pub fn set_buffer(&mut self, name: Option<&str>, data: &[u8]) {
        let created = unsafe { libc::time(std::ptr::null_mut()) as i64 };
        match name {
            Some(n) => {
                self.automatic_buffers.remove(n);
                self.buffer_created.insert(n.to_string(), created);
                if let Some(entry) = self.buffers.iter_mut().find(|(bn, _)| bn == n) {
                    entry.1 = data.to_vec();
                } else {
                    self.buffers.insert(0, (n.to_string(), data.to_vec()));
                }
            }
            None => {
                let n = format!("buffer{}", self.next_buffer_id);
                self.next_buffer_id += 1;
                self.automatic_buffers.insert(n.clone());
                self.buffer_created.insert(n.clone(), created);
                self.buffers.insert(0, (n, data.to_vec()));
                self.enforce_buffer_limit();
            }
        }
    }

    /// `buffer-limit` bounds only automatically named buffers. Explicitly
    /// named buffers are retained, matching tmux's paste-buffer policy.
    fn enforce_buffer_limit(&mut self) {
        let limit = self
            .server_options()
            .get("buffer-limit")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(50);
        while self.automatic_buffers.len() > limit {
            let Some(position) = self
                .buffers
                .iter()
                .rposition(|(name, _)| self.automatic_buffers.contains(name))
            else {
                break;
            };
            let (name, _) = self.buffers.remove(position);
            self.automatic_buffers.remove(&name);
            self.buffer_created.remove(&name);
        }
    }

    pub(crate) fn add_buffer_with_prefix(&mut self, prefix: Option<&str>, data: &[u8]) {
        match prefix {
            None => self.set_buffer(None, data),
            Some(prefix) => {
                let mut index = self.next_buffer_id;
                loop {
                    let name = format!("{prefix}{index}");
                    if self.buffers.iter().all(|(existing, _)| existing != &name) {
                        self.next_buffer_id = index + 1;
                        self.set_buffer(Some(&name), data);
                        break;
                    }
                    index += 1;
                }
            }
        }
    }

    /// Append data to a paste buffer. A missing named buffer is created; with
    /// no name, append to the newest buffer or create an auto-named buffer when
    /// none exists.
    pub fn append_buffer(&mut self, name: Option<&str>, data: &[u8]) {
        let entry = match name {
            Some(n) => self.buffers.iter_mut().find(|(bn, _)| bn == n),
            None => self.buffers.first_mut(),
        };
        if let Some((_, existing)) = entry {
            existing.extend_from_slice(data);
        } else {
            self.set_buffer(name, data);
        }
    }

    /// Rename a paste buffer. If `new_name` already exists, tmux replaces it
    /// with the renamed source buffer.
    pub fn rename_buffer(&mut self, name: &str, new_name: &str) -> bool {
        if !self
            .buffers
            .iter()
            .any(|(buffer_name, _)| buffer_name == name)
        {
            return false;
        }
        if name != new_name {
            self.buffers
                .retain(|(buffer_name, _)| buffer_name != new_name);
            if let Some((buffer_name, _)) = self
                .buffers
                .iter_mut()
                .find(|(buffer_name, _)| buffer_name == name)
            {
                *buffer_name = new_name.to_string();
            }
            if let Some(created) = self.buffer_created.remove(name) {
                self.buffer_created.insert(new_name.to_string(), created);
            }
        }
        self.automatic_buffers.remove(name);
        self.automatic_buffers.remove(new_name);
        true
    }

    /// The named buffer's data, or the most recent automatically named buffer's
    /// data if `name` is `None`. Explicitly named buffers do not participate in
    /// tmux's unnamed show/save lookup.
    pub fn buffer(&self, name: Option<&str>) -> Option<&[u8]> {
        match name {
            Some(n) => self
                .buffers
                .iter()
                .find(|(bn, _)| bn == n)
                .map(|(_, d)| d.as_slice()),
            None => self
                .buffers
                .iter()
                .find(|(bn, _)| self.automatic_buffers.contains(bn))
                .map(|(_, d)| d.as_slice()),
        }
    }

    /// Iterate paste buffers, newest first (tmux's `list-buffers` order).
    pub fn buffers(&self) -> &[(String, Vec<u8>)] {
        &self.buffers
    }

    pub(crate) fn buffer_created(&self, name: &str) -> Option<i64> {
        self.buffer_created.get(name).copied()
    }

    /// Delete a paste buffer by name. Returns whether it existed.
    pub fn delete_buffer(&mut self, name: &str) -> bool {
        let before = self.buffers.len();
        self.buffers.retain(|(n, _)| n != name);
        self.automatic_buffers.remove(name);
        self.buffer_created.remove(name);
        self.buffers.len() != before
    }

    /// Set a global environment variable (`set-environment -g VAR VALUE`).
    pub fn set_env(&mut self, key: &str, value: &str) {
        let changed = self.environment.get(key).is_none_or(|old| old != value)
            || self.hidden_environment.contains(key)
            || self.removed_environment.contains(key);
        self.environment.insert(key.to_string(), value.to_string());
        self.hidden_environment.remove(key);
        self.removed_environment.remove(key);
        if changed {
            self.invalidate_all_clients(RenderInvalidation::STATUS);
        }
    }

    pub(crate) fn set_session_env(
        &mut self,
        target: &str,
        key: &str,
        value: &str,
        hidden: bool,
    ) -> io::Result<()> {
        let session = self
            .find_mut(target)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "can't find session"))?;
        session
            .environment
            .insert(key.to_string(), value.to_string());
        session.removed_environment.remove(key);
        if hidden {
            session.hidden_environment.insert(key.to_string());
        } else {
            session.hidden_environment.remove(key);
        }
        Ok(())
    }

    pub(crate) fn unset_session_env(
        &mut self,
        target: &str,
        key: &str,
        remove: bool,
    ) -> io::Result<()> {
        let session = self
            .find_mut(target)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "can't find session"))?;
        session.environment.remove(key);
        session.hidden_environment.remove(key);
        if remove {
            session.removed_environment.insert(key.to_string());
        } else {
            session.removed_environment.remove(key);
        }
        Ok(())
    }

    pub(crate) fn session_env(
        &self,
        target: &str,
    ) -> io::Result<(
        &BTreeMap<String, String>,
        &BTreeSet<String>,
        &BTreeSet<String>,
    )> {
        let session = self
            .resolve_session(target)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "can't find session"))?;
        Ok((
            &session.environment,
            &session.removed_environment,
            &session.hidden_environment,
        ))
    }

    /// Set a hidden global environment variable (`set-environment -g -h`).
    pub fn set_hidden_env(&mut self, key: &str, value: &str) {
        let changed = self.environment.get(key).is_none_or(|old| old != value)
            || !self.hidden_environment.contains(key)
            || self.removed_environment.contains(key);
        self.environment.insert(key.to_string(), value.to_string());
        self.hidden_environment.insert(key.to_string());
        self.removed_environment.remove(key);
        if changed {
            self.invalidate_all_clients(RenderInvalidation::STATUS);
        }
    }

    /// Mark a global environment variable for removal from child processes.
    pub fn remove_env(&mut self, key: &str) {
        let had_value = self.environment.remove(key).is_some();
        let was_hidden = self.hidden_environment.remove(key);
        let newly_removed = self.removed_environment.insert(key.to_string());
        let changed = had_value || was_hidden || newly_removed;
        if changed {
            self.invalidate_all_clients(RenderInvalidation::STATUS);
        }
    }

    /// Unset a global environment variable (`set-environment -g -u VAR`).
    pub fn unset_env(&mut self, key: &str) {
        let had_value = self.environment.remove(key).is_some();
        let was_hidden = self.hidden_environment.remove(key);
        let was_removed = self.removed_environment.remove(key);
        let changed = had_value || was_hidden || was_removed;
        if changed {
            self.invalidate_all_clients(RenderInvalidation::STATUS);
        }
    }

    /// Look up a global environment variable.
    pub fn get_env(&self, key: &str) -> Option<&str> {
        self.environment.get(key).map(String::as_str)
    }

    /// Whether a global environment variable is hidden.
    pub fn env_is_hidden(&self, key: &str) -> bool {
        self.hidden_environment.contains(key)
    }

    /// Whether a global environment variable is marked for removal.
    pub fn env_is_removed(&self, key: &str) -> bool {
        self.removed_environment.contains(key)
    }

    /// Iterate the global environment in sorted order.
    pub fn env_iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.environment.iter()
    }

    /// Iterate names marked for removal in sorted order.
    pub fn removed_env_iter(&self) -> impl Iterator<Item = &String> {
        self.removed_environment.iter()
    }

    pub(crate) fn global_options(&self) -> &GlobalOptions {
        &self.global_options
    }

    pub(crate) fn global_options_mut(&mut self) -> &mut GlobalOptions {
        &mut self.global_options
    }

    pub(crate) fn server_options(&self) -> OptionsView<'_> {
        OptionsView::one(self.global_options.server())
    }

    pub(crate) fn command_aliases(&self) -> Vec<(String, String)> {
        self.server_options()
            .iter_effective()
            .filter_map(|(name, value)| {
                super::options::parse_option_name(name)
                    .is_some_and(|(base, index)| base == "command-alias" && index.is_some())
                    .then(|| value.split_once('='))
                    .flatten()
                    // Only the `=` separator is required: an entry may carry an
                    // empty command, which matches the name and expands to no
                    // commands at all.
                    .filter(|(alias, _)| !alias.is_empty())
                    .map(|(alias, command)| (alias.to_string(), command.to_string()))
            })
            .collect()
    }

    pub(crate) fn session_options(&self, target: &str) -> io::Result<OptionsView<'_>> {
        let target = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        Ok(self.sessions[target.session].options(&self.global_options))
    }

    pub(crate) fn window_options(&self, target: &str) -> io::Result<OptionsView<'_>> {
        let target = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        Ok(self
            .window(target.session, target.window)
            .options(&self.global_options))
    }

    pub(crate) fn pane_options(&self, target: &str) -> io::Result<OptionsView<'_>> {
        let target = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let window = self.window(target.session, target.window);
        Ok(window.panes[target.pane].options(window, &self.global_options))
    }

    pub(crate) fn option_for_target<'a>(&'a self, target: &str, name: &str) -> Option<&'a str> {
        let base = super::options::parse_option_name(name)
            .map(|(base, _)| base)
            .unwrap_or(name);
        match super::options::option_scope(base)? {
            super::options::OptionScope::Server => self.server_options().get(name),
            super::options::OptionScope::Session => self.session_options(target).ok()?.get(name),
            super::options::OptionScope::Window => self.window_options(target).ok()?.get(name),
            super::options::OptionScope::WindowPane => self.pane_options(target).ok()?.get(name),
        }
    }

    pub(crate) fn format_option_entries<'a>(
        &'a self,
        target: &str,
    ) -> io::Result<impl Iterator<Item = (&'a str, &'a str)>> {
        let target = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session = &self.sessions[target.session];
        let window = self.window(target.session, target.window);
        let pane = &window.panes[target.pane];
        let mut entries = BTreeMap::new();
        for (name, value) in session.options(&self.global_options).iter_effective() {
            entries.insert(name, value);
        }
        for (name, value) in pane.options(window, &self.global_options).iter_effective() {
            entries.insert(name, value);
        }
        for (name, value) in self.server_options().iter_effective() {
            entries.insert(name, value);
        }
        Ok(entries.into_iter())
    }

    /// Set a built-in option in its global table. Command execution uses the
    /// object-scoped accessors directly; this remains a compact engine/test API.
    #[cfg(test)]
    pub(crate) fn set_global_option(&mut self, key: &str, value: &str) {
        let base = super::options::parse_option_name(key)
            .map(|(name, _)| name)
            .unwrap_or(key);
        let Some(scope) = super::options::option_scope(base) else {
            return;
        };
        let table = self.global_options.for_scope_mut(scope);
        let changed = table.get(key) != Some(value);
        table.set(key, value);
        if changed && option_affects_render(key) {
            self.invalidate_all_clients(option_invalidation(key));
        }
    }

    pub(crate) fn push_config_error(&mut self, error: String) {
        self.pending_config_errors.push(error);
    }

    pub(crate) fn take_config_errors(&mut self) -> Vec<String> {
        std::mem::take(&mut self.pending_config_errors)
    }

    /// Toggle the target window's zoom state (`resize-pane -Z`), returning the
    /// new state. Errors via [`Self::resolve_window`] on a bad target.
    pub fn toggle_zoom(&mut self, target: &str) -> io::Result<bool> {
        let t = self.resolve_window(target)?;
        let session_id = self.sessions[t.session].id;
        let win = self.window_mut(t.session, t.window);
        win.zoomed = !win.zoomed;
        let zoomed = win.zoomed;
        self.invalidate_session(session_id, RenderInvalidation::LAYOUT);
        Ok(zoomed)
    }

    pub(crate) fn resize_pane(
        &mut self,
        target: &str,
        direction: SplitDirection,
        forward: bool,
        amount: u16,
    ) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let pane_id = self.window(resolved.session, resolved.window).panes[resolved.pane].id;
        let window = self.window_mut(resolved.session, resolved.window);
        window.zoomed = false;
        window
            .layout
            .resize_pane_toward(pane_id, direction, forward, amount.max(1));
        resize_panes_to_layout(window)?;
        self.invalidate_session(
            session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
        );
        Ok(())
    }

    pub(crate) fn resize_pane_to(
        &mut self,
        target: &str,
        direction: SplitDirection,
        size: u16,
    ) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let pane_id = self.window(resolved.session, resolved.window).panes[resolved.pane].id;
        let window = self.window_mut(resolved.session, resolved.window);
        window.zoomed = false;
        window.layout.resize_pane_to(pane_id, direction, size);
        resize_panes_to_layout(window)?;
        self.invalidate_session(
            session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
        );
        Ok(())
    }

    pub(crate) fn select_named_layout(&mut self, target: &str, layout: usize) -> io::Result<()> {
        let resolved = self.resolve_window(target)?;
        let main_height = self
            .option_for_target(target, "main-pane-height")
            .and_then(|value| value.parse().ok())
            .unwrap_or(24);
        let main_width = self
            .option_for_target(target, "main-pane-width")
            .and_then(|value| value.parse().ok())
            .unwrap_or(80);
        let other_height = self
            .option_for_target(target, "other-pane-height")
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        let other_width = self
            .option_for_target(target, "other-pane-width")
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        let max_columns = self
            .option_for_target(target, "tiled-layout-max-columns")
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        let window_id = self.sessions[resolved.session].windows[resolved.window].id;
        let affected_sessions = self
            .sessions
            .iter()
            .filter(|session| session.windows.iter().any(|link| link.id == window_id))
            .map(|session| session.id)
            .collect::<Vec<_>>();
        let window = self.window_mut(resolved.session, resolved.window);
        let pane_ids = window
            .panes
            .iter()
            .filter(|pane| pane.floating.is_none())
            .map(|pane| pane.id)
            .collect::<Vec<_>>();
        if pane_ids.len() > 1 {
            let rect = window.layout.rect();
            window.layout = match layout {
                0 => LayoutCell::even(&pane_ids, SplitDirection::LeftRight, rect),
                1 => LayoutCell::even(&pane_ids, SplitDirection::TopBottom, rect),
                2 => LayoutCell::main(
                    &pane_ids,
                    SplitDirection::TopBottom,
                    rect,
                    main_height,
                    other_height,
                    false,
                ),
                3 => LayoutCell::main(
                    &pane_ids,
                    SplitDirection::TopBottom,
                    rect,
                    main_height,
                    other_height,
                    true,
                ),
                4 => LayoutCell::main(
                    &pane_ids,
                    SplitDirection::LeftRight,
                    rect,
                    main_width,
                    other_width,
                    false,
                ),
                5 => LayoutCell::main(
                    &pane_ids,
                    SplitDirection::LeftRight,
                    rect,
                    main_width,
                    other_width,
                    true,
                ),
                6 => LayoutCell::tiled(&pane_ids, rect, max_columns),
                _ => return Err(io::Error::other("invalid layout")),
            };
            resize_panes_to_layout(window)?;
        }
        window.last_layout = Some(layout);
        window.zoomed = false;
        for session_id in affected_sessions {
            self.invalidate_session(
                session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        }
        // tmux notifies twice for a named layout: once from `layout_set_*` when
        // the cells are rebuilt, and once from the command itself.
        self.notify_window("window-layout-changed", window_id);
        self.notify_window("window-layout-changed", window_id);
        Ok(())
    }

    pub(crate) fn cycle_layout(&mut self, target: &str, forward: bool) -> io::Result<()> {
        let resolved = self.resolve_window(target)?;
        let current = self.window(resolved.session, resolved.window).last_layout;
        let next = match (current, forward) {
            (None, true) => 0,
            (None, false) => 6,
            (Some(6), true) => 0,
            (Some(0), false) => 6,
            (Some(value), true) => value + 1,
            (Some(value), false) => value - 1,
        };
        self.select_named_layout(target, next)
    }

    pub(crate) fn select_custom_layout(&mut self, target: &str, value: &str) -> io::Result<()> {
        let mut layout = parse_custom_layout(value)?;
        let resolved = self.resolve_window(target)?;
        let window_id = self.sessions[resolved.session].windows[resolved.window].id;
        let affected_sessions = self
            .sessions
            .iter()
            .filter(|session| session.windows.iter().any(|link| link.id == window_id))
            .map(|session| session.id)
            .collect::<Vec<_>>();
        let pane_ids = self
            .window(resolved.session, resolved.window)
            .panes
            .iter()
            .filter(|pane| pane.floating.is_none())
            .map(|pane| pane.id)
            .collect::<Vec<_>>();
        if layout.panes().len() != pane_ids.len() {
            return Err(io::Error::other(format!(
                "have {} panes but need {}",
                pane_ids.len(),
                layout.panes().len()
            )));
        }
        layout.assign_panes(&mut pane_ids.into_iter());
        let size = (layout.rect().width, layout.rect().height);
        let window = self.window_mut(resolved.session, resolved.window);
        window.layout = layout;
        window.manual_size = Some(size);
        window.last_layout = None;
        window.zoomed = false;
        resize_panes_to_layout(window)?;
        for session_id in affected_sessions {
            self.invalidate_session(
                session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        }
        self.notify_window("window-layout-changed", window_id);
        Ok(())
    }

    /// `resize-window -x cols -y rows`: force the target window to a manual size,
    /// observable via `#{window_width}`/`#{window_height}`. A dimension left
    /// unspecified keeps the window's current effective size for that axis.
    pub fn resize_window(
        &mut self,
        target: &str,
        cols: Option<u16>,
        rows: Option<u16>,
    ) -> io::Result<()> {
        let t = self.resolve_window(target)?;
        let session_id = self.sessions[t.session].id;
        let sess_cols = self.sessions[t.session].cols;
        let sess_rows = self.sessions[t.session].rows;
        let win = self.window_mut(t.session, t.window);
        let window_id = win.id;
        let previous = win.manual_size;
        let (cur_cols, cur_rows) = previous.unwrap_or((sess_cols, sess_rows));
        let size = (cols.unwrap_or(cur_cols), rows.unwrap_or(cur_rows));
        win.manual_size = Some(size);
        win.layout.resize(size.0, size.1);
        resize_panes_to_layout(win)?;
        self.invalidate_session(session_id, RenderInvalidation::LAYOUT);
        if previous != Some(size) {
            self.notify_window("window-resized", window_id);
        }
        Ok(())
    }

    pub(crate) fn resize_linked_window(
        &mut self,
        target: &str,
        cols: u16,
        rows: u16,
    ) -> io::Result<()> {
        let resolved = self.resolve_window_arg(target)?;
        let window_id = self.sessions[resolved.session].windows[resolved.window].id;
        let affected_sessions: Vec<u32> = self
            .sessions
            .iter()
            .filter_map(|session| {
                session
                    .windows
                    .iter()
                    .any(|window| window.id == window_id)
                    .then_some(session.id)
            })
            .collect();
        let window = self.windows.get_mut(&window_id).expect("window present");
        window.manual_size = Some((cols, rows));
        window.layout.resize(cols, rows);
        let _ = resize_panes_to_layout(window);
        for session_id in affected_sessions {
            self.invalidate_session(session_id, RenderInvalidation::LAYOUT);
        }
        Ok(())
    }

    /// The effective `base-index` (first/lowest window index), default 0.
    fn base_index(&self) -> u32 {
        self.global_options
            .session()
            .get("base-index")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    }

    /// `kill-server`: tear down all sessions. A command client sees exit 0.
    pub fn kill_server(&mut self) {
        let had_sessions = !self.sessions.is_empty();
        self.sessions.clear();
        self.windows.clear();
        self.session_groups.clear();
        self.request_shutdown_if_became_empty(had_sessions);
        self.invalidate_all_clients(RenderInvalidation::SESSION_GONE);
    }

    /// Remove a session by name. Returns whether it existed (its panes' children
    /// are killed as the panes drop).
    pub fn kill_session(&mut self, name: &str) -> bool {
        let had_sessions = !self.sessions.is_empty();
        let removed_id = self
            .sessions
            .iter()
            .find(|session| session.name == name)
            .map(|session| session.id);
        if let Some(session_id) = removed_id {
            self.apply_detach_on_destroy(session_id);
        }
        let before = self.sessions.len();
        self.sessions.retain(|s| s.name != name);
        let removed = self.sessions.len() != before;
        if removed {
            self.remove_unlinked_windows();
            self.request_shutdown_if_became_empty(had_sessions);
            if let Some(session_id) = removed_id {
                self.invalidate_session(session_id, RenderInvalidation::SESSION_GONE);
                self.notify_closed_session("session-closed", session_id, name);
            }
        }
        removed
    }

    /// Record an event notification for the command queue to turn into hook
    /// bodies. Mirrors tmux's `notify_add`.
    fn notify(&mut self, name: &str, target: Option<String>, mut vars: Vec<(String, String)>) {
        if !super::options::is_hook(name) {
            return;
        }
        vars.insert(0, ("hook".to_string(), name.to_string()));
        self.pending_notifications.push(Notification {
            name: name.to_string(),
            target,
            vars,
            deferred: self.notifications_are_deferred,
        });
    }

    /// Mark notifications raised inside `body` as server-loop work rather than
    /// command-queue work.
    fn deferred_notifications<T>(&mut self, body: impl FnOnce(&mut Self) -> T) -> T {
        let previous = std::mem::replace(&mut self.notifications_are_deferred, true);
        let result = body(self);
        self.notifications_are_deferred = previous;
        result
    }

    /// Take the notifications the server loop owns, leaving the rest for the
    /// command queue.
    pub(crate) fn take_deferred_notifications(&mut self) -> Vec<Notification> {
        let mut deferred = Vec::new();
        self.pending_notifications.retain(|notification| {
            if notification.deferred {
                deferred.push(notification.clone());
                false
            } else {
                true
            }
        });
        deferred
    }

    /// tmux's `notify_session`: an event about a session as a whole.
    pub(crate) fn notify_session(&mut self, name: &str, session_id: u32) {
        let Some(session) = self.sessions.iter().find(|s| s.id == session_id) else {
            // A closed session is still named by its own event.
            return;
        };
        let vars = vec![
            ("hook_session".to_string(), format!("${session_id}")),
            ("hook_session_name".to_string(), session.name.clone()),
        ];
        self.notify(name, Some(format!("${session_id}")), vars);
    }

    /// Like [`Self::notify_session`], for a session that has already been
    /// removed from the tree and can only be named by the caller.
    pub(crate) fn notify_closed_session(&mut self, name: &str, session_id: u32, session: &str) {
        let vars = vec![
            ("hook_session".to_string(), format!("${session_id}")),
            ("hook_session_name".to_string(), session.to_string()),
        ];
        self.notify(name, None, vars);
    }

    /// tmux's `notify_window`: an event about a window, carrying no session.
    pub(crate) fn notify_window(&mut self, name: &str, window_id: u32) {
        let Some(window) = self.windows.get(&window_id) else {
            return;
        };
        let vars = vec![
            ("hook_window".to_string(), format!("@{window_id}")),
            ("hook_window_name".to_string(), window.name.clone()),
        ];
        self.notify(name, Some(format!("@{window_id}")), vars);
    }

    /// tmux's `notify_session_window`/`notify_winlink`: an event about a window
    /// as seen from one session, so both layers are published.
    pub(crate) fn notify_session_window(&mut self, name: &str, session_id: u32, window_id: u32) {
        let session = self.sessions.iter().find(|s| s.id == session_id);
        let window_name = self.windows.get(&window_id).map(|w| w.name.clone());
        let mut vars = Vec::new();
        if let Some(session) = session {
            vars.push(("hook_session".to_string(), format!("${session_id}")));
            vars.push(("hook_session_name".to_string(), session.name.clone()));
        }
        if let Some(window_name) = window_name {
            vars.push(("hook_window".to_string(), format!("@{window_id}")));
            vars.push(("hook_window_name".to_string(), window_name));
        }
        // The window may already be unlinked, so the session is the target that
        // still resolves.
        let target = self
            .sessions
            .iter()
            .find(|s| s.id == session_id && s.windows.iter().any(|link| link.id == window_id))
            .map(|_| format!("${session_id}:@{window_id}"))
            .or_else(|| session.map(|_| format!("${session_id}")));
        self.notify(name, target, vars);
    }

    /// tmux's `notify_pane`: an event about a pane, which also publishes the
    /// window the pane belongs to.
    pub(crate) fn notify_pane(&mut self, name: &str, pane_id: u32) {
        let Some((window_id, window_name)) = self.windows.iter().find_map(|(id, window)| {
            window
                .panes
                .iter()
                .any(|node| node.id == pane_id)
                .then(|| (*id, window.name.clone()))
        }) else {
            return;
        };
        let vars = vec![
            ("hook_pane".to_string(), format!("%{pane_id}")),
            ("hook_window".to_string(), format!("@{window_id}")),
            ("hook_window_name".to_string(), window_name),
        ];
        self.notify(name, Some(format!("%{pane_id}")), vars);
    }

    /// tmux's `notify_client`: an event about one client.
    pub(crate) fn notify_client(&mut self, name: &str, client: &str, session_id: Option<u32>) {
        let vars = vec![("hook_client".to_string(), client.to_string())];
        self.notify(name, session_id.map(|id| format!("${id}")), vars);
    }

    /// Take everything raised since the last drain.
    pub(crate) fn take_notifications(&mut self) -> Vec<Notification> {
        std::mem::take(&mut self.pending_notifications)
    }

    /// tmux's `server_client_set_session` tail: the client's new current window
    /// loses its alert flags, and the whole session's alert state is examined
    /// again now that somebody is looking at it.
    fn take_session_for_client(&mut self, session_id: u32) {
        if let Some(session) = self.sessions.iter_mut().find(|s| s.id == session_id) {
            let active = session.active;
            if let Some(link) = session.windows.get_mut(active) {
                link.alert_flags = 0;
            }
        }
        self.alert_check_sessions.insert(session_id);
    }

    /// Raise the client-layer notifications tmux emits from `server_client_*`.
    ///
    /// The client registry is owned by the attach loops rather than by the
    /// command path, so what changed is read from a snapshot of it instead of
    /// from a call at each site.
    fn sync_client_notifications(&mut self) {
        let current = self
            .attached_clients()
            .into_iter()
            .map(|client| (client.name, (client.session_id, client.cols, client.rows)))
            .collect::<BTreeMap<_, _>>();
        let previous = std::mem::replace(&mut self.known_clients, current.clone());
        let was_deferred = std::mem::replace(&mut self.notifications_are_deferred, true);
        for (name, (session_id, cols, rows)) in &current {
            match previous.get(name) {
                None => {
                    self.notify_client("client-attached", name, Some(*session_id));
                    self.notify_client("client-session-changed", name, Some(*session_id));
                    // A new client announces its terminal size as part of
                    // attaching, which tmux reports as a resize of its own.
                    self.notify_client("client-resized", name, Some(*session_id));
                    self.take_session_for_client(*session_id);
                }
                Some((old_session, old_cols, old_rows)) => {
                    if old_session != session_id {
                        self.notify_client("client-session-changed", name, Some(*session_id));
                        self.take_session_for_client(*session_id);
                    }
                    if (old_cols, old_rows) != (cols, rows) {
                        self.notify_client("client-resized", name, Some(*session_id));
                    }
                }
            }
        }
        for name in previous.keys().filter(|name| !current.contains_key(*name)) {
            self.notify_client("client-detached", name, None);
        }
        self.notifications_are_deferred = was_deferred;
        // Gaining or losing a client moves pane focus, for the window it
        // arrived at and the one it left.
        let touched = previous
            .values()
            .chain(current.values())
            .filter_map(|(session_id, _, _)| self.current_window_of_session(*session_id))
            .collect::<BTreeSet<_>>();
        for window_id in touched {
            self.update_window_focus(window_id);
        }
    }

    /// tmux's `session_update_activity`. `attached` additionally stamps
    /// `last_attached_time`, which only a client taking the session does.
    pub(crate) fn touch_session_activity(&mut self, session_id: u32, attached: bool) {
        let now = now_micros();
        if let Some(session) = self
            .sessions
            .iter_mut()
            .find(|session| session.id == session_id)
        {
            session.activity_micros = now;
            if attached {
                session.last_attached_micros = now;
            }
        }
    }

    /// tmux stamps `c->activity_time` alongside the session's whenever a key
    /// arrives; it is what orders `cmd_find_best_client`.
    pub(crate) fn touch_client_activity(&mut self, client: &str, session_id: u32) {
        self.touch_session_activity(session_id, false);
        self.client_renders.touch_client_activity(client, now_micros());
    }

    /// tmux's `session_lock_timer`: lock every client of a session whose
    /// clients have gone `lock-after-time` seconds without touching a key.
    ///
    /// tmux arms the timer from `session_update_activity` and lets libevent
    /// fire it once; hmux polls instead, so `locked_at_activity_micros` is what
    /// keeps an idle session from being locked again on every server loop.
    /// Returns how long the loop may sleep before the next session is due.
    pub(crate) fn refresh_lock_timers(&mut self) -> Option<Duration> {
        let attached = self.attached_session_ids();
        if attached.is_empty() {
            return None;
        }
        let now = now_micros();
        let mut due = Vec::new();
        let mut next = None;
        for session in self
            .sessions
            .iter()
            .filter(|session| attached.contains(&session.id))
        {
            let seconds = session
                .options(&self.global_options)
                .get("lock-after-time")
                .and_then(|value| value.parse::<i64>().ok())
                .unwrap_or(0);
            if seconds <= 0 || session.locked_at_activity_micros == Some(session.activity_micros) {
                continue;
            }
            let deadline = session.activity_micros + seconds * 1_000_000;
            if deadline <= now {
                due.push(session.id);
            } else {
                let remaining = Duration::from_micros((deadline - now) as u64);
                next = Some(next.map_or(remaining, |next: Duration| next.min(remaining)));
            }
        }
        let commands = self.lock_commands();
        for session_id in due {
            if let Some(session) = self
                .sessions
                .iter_mut()
                .find(|session| session.id == session_id)
            {
                session.locked_at_activity_micros = Some(session.activity_micros);
            }
            if let Some(command) = commands.get(&session_id) {
                self.client_renders.lock_session(session_id, command);
            }
        }
        next
    }

    /// The sessions that currently have at least one client attached.
    fn attached_session_ids(&self) -> BTreeSet<u32> {
        self.attached_clients()
            .into_iter()
            .map(|client| client.session_id)
            .collect()
    }

    /// The session other than `exclude` with the newest activity time, in
    /// tmux's `server_find_session(server_newer_session)` order: sessions are
    /// walked by name and the comparison is strict, so equal stamps keep the
    /// alphabetically first candidate.
    fn newest_session_excluding(&self, exclude: u32, detached_only: bool) -> Option<u32> {
        let attached = detached_only.then(|| self.attached_session_ids());
        let mut candidates = self
            .sessions
            .iter()
            .filter(|session| session.id != exclude)
            .filter(|session| {
                attached
                    .as_ref()
                    .is_none_or(|attached| !attached.contains(&session.id))
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|a, b| a.name.cmp(&b.name));
        candidates
            .into_iter()
            .fold(None::<&Session>, |best, session| match best {
                Some(best) if session.activity_micros <= best.activity_micros => Some(best),
                _ => Some(session),
            })
            .map(|session| session.id)
    }

    /// tmux's `session_previous_session`/`session_next_session`: the neighbour
    /// of `exclude` in the name-ordered session list, wrapping at both ends.
    fn neighbour_session(&self, exclude: u32, forward: bool) -> Option<u32> {
        let mut ordered = self.sessions.iter().collect::<Vec<_>>();
        ordered.sort_by(|a, b| a.name.cmp(&b.name));
        let count = ordered.len();
        if count < 2 {
            return None;
        }
        let position = ordered.iter().position(|session| session.id == exclude)?;
        let next = if forward {
            (position + 1) % count
        } else {
            (position + count - 1) % count
        };
        Some(ordered[next].id)
    }

    /// tmux's `server_destroy_session`: move the clients of a session that is
    /// about to disappear, according to that session's `detach-on-destroy`.
    /// Clients with no destination are left in place and exit themselves.
    fn apply_detach_on_destroy(&mut self, session_id: u32) {
        let Some(session) = self.sessions.iter().find(|session| session.id == session_id) else {
            return;
        };
        let policy = session
            .options(&self.global_options)
            .get("detach-on-destroy")
            .unwrap_or("on")
            .to_owned();
        let new_session = match policy.as_str() {
            "off" | "0" => self.newest_session_excluding(session_id, false),
            "no-detached" => self.newest_session_excluding(session_id, true),
            "previous" => self.neighbour_session(session_id, false),
            "next" => self.neighbour_session(session_id, true),
            _ => None,
        };
        // `on` and `no-detached` still offer a session to clients that asked
        // not to be detached when the policy itself found none.
        let no_detach_session = match (&new_session, policy.as_str()) {
            (None, "on" | "1" | "no-detached") => self.newest_session_excluding(session_id, false),
            _ => None,
        };
        self.client_renders
            .reassign_session(session_id, new_session, no_detach_session);
        // Taking over a session counts as activity on it, exactly as an
        // ordinary attach does.
        for adopted in [new_session, no_detach_session].into_iter().flatten() {
            self.touch_session_activity(adopted, true);
        }
    }

    /// tmux's `server_check_unattached`, run whenever a client is lost or
    /// changes session — not only when the option is set.
    pub(crate) fn enforce_unattached_options(&mut self) {
        // tmux walks its name-ordered session tree and destroys in place, so a
        // group shrinking under `keep-last` spares whichever member is reached
        // once it is the last one. Re-scanning after each destroy reproduces
        // that without depending on hmux's creation-ordered session vector.
        while let Some(name) = self.next_unattached_session_to_destroy() {
            self.kill_session(&name);
        }
    }

    fn next_unattached_session_to_destroy(&self) -> Option<String> {
        let attached = self.attached_session_ids();
        let mut ordered = self.sessions.iter().collect::<Vec<_>>();
        ordered.sort_by(|a, b| a.name.cmp(&b.name));
        ordered
            .into_iter()
            .find(|session| {
                if attached.contains(&session.id) {
                    return false;
                }
                let grouped = self.is_grouped(session);
                let group_size = if grouped {
                    self.session_group_size(session)
                } else {
                    1
                };
                match session
                    .options(&self.global_options)
                    .get("destroy-unattached")
                    .unwrap_or("off")
                {
                    "on" | "1" => true,
                    // Only a session sharing its group with others is destroyed.
                    "keep-last" => grouped && group_size > 1,
                    // A session alone in an explicit group is spared.
                    "keep-group" => !(grouped && group_size == 1),
                    _ => false,
                }
            })
            .map(|session| session.name.clone())
    }

    /// One pass of tmux's per-loop lifecycle policies: destroy sessions that
    /// lost their last client, then decide whether the server itself should
    /// exit. The unattached sweep is skipped while the client set is unchanged.
    pub(crate) fn enforce_lifecycle_policies(&mut self) {
        let generation = self.client_renders.generation();
        if generation != self.lifecycle_generation {
            self.lifecycle_generation = generation;
            self.sync_client_notifications();
            self.enforce_unattached_options();
        }
        self.enforce_exit_options();
    }

    /// tmux's `server_loop` shutdown test: with `exit-empty` on, the server
    /// exits once nothing holds it — no sessions (or `exit-unattached` on) and
    /// no client still attached to one.
    pub(crate) fn enforce_exit_options(&mut self) {
        // An hmux daemon is launched before any client exists, so the policy
        // only starts applying once the server has held a session at least
        // once; tmux never sees this window because its server is forked by
        // the client that creates the first session.
        if self.initial_attach_pending {
            return;
        }
        if !self.server_option_is_on("exit-empty", true) {
            return;
        }
        if !self.server_option_is_on("exit-unattached", false) && !self.sessions.is_empty() {
            return;
        }
        if !self.attached_session_ids().is_empty() {
            return;
        }
        self.shutdown_requested = true;
    }

    /// `kill-session -g`: destroy every member when the target is grouped, or
    /// just the target when it is not.
    pub fn kill_session_group(&mut self, name: &str) -> bool {
        let Some(position) = self.session_index(name) else {
            return false;
        };
        let link_set_id = self.sessions[position].link_set_id;
        if !self.session_groups.contains_key(&link_set_id) {
            return self.kill_session(name);
        }
        let had_sessions = !self.sessions.is_empty();
        let removed = self
            .sessions
            .iter()
            .filter_map(|session| (session.link_set_id == link_set_id).then_some(session.id))
            .collect::<Vec<_>>();
        self.sessions
            .retain(|session| session.link_set_id != link_set_id);
        self.remove_unlinked_windows();
        self.request_shutdown_if_became_empty(had_sessions);
        for session_id in removed {
            self.invalidate_session(session_id, RenderInvalidation::SESSION_GONE);
        }
        true
    }

    /// `kill-session -a [-t keep]`: destroy every session except `keep`.
    /// Errors (`can't find session`) if `keep` doesn't exist.
    pub fn kill_other_sessions(&mut self, keep: &str) -> io::Result<()> {
        if self.session_index(keep).is_none() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {keep}"),
            ));
        }
        let keep_id = self.sessions[self.session_index(keep).unwrap()].id;
        let removed: Vec<u32> = self
            .sessions
            .iter()
            .filter_map(|session| (session.id != keep_id).then_some(session.id))
            .collect();
        self.sessions.retain(|s| s.id == keep_id);
        self.remove_unlinked_windows();
        for session_id in removed {
            self.invalidate_session(session_id, RenderInvalidation::SESSION_GONE);
        }
        Ok(())
    }

    /// Find a session mutably.
    pub fn find_mut(&mut self, name: &str) -> Option<&mut Session> {
        let pos = self.session_index(name)?;
        self.sessions.get_mut(pos)
    }

    /// Get the active pane of a session, if the session exists and has windows.
    pub fn active_pane(&self, session_name: &str) -> Option<&Pane> {
        let session = self.session_index(session_name)?;
        let sess = &self.sessions[session];
        let win = self.window(session, sess.active);
        let pane_node = win.panes.get(win.active)?;
        Some(&pane_node.pane)
    }

    /// Get a mutable reference to the active pane.
    pub fn active_pane_mut(&mut self, session_name: &str) -> Option<&mut Pane> {
        let session = self.session_index(session_name)?;
        let win_idx = self.sessions[session].active;
        let win = self.window_mut(session, win_idx);
        let pane_idx = win.active;
        let pane_node = win.panes.get_mut(pane_idx)?;
        Some(&mut pane_node.pane)
    }

    /// The active window's name in `session` (tmux's `#W`). Used to fill in the
    /// `confirm-before` prompt text for `C-b &` (kill-window), matching tmux's
    /// default `"kill-window #W? (y/n)"`.
    pub(crate) fn active_window_name(&self, session_name: &str) -> Option<String> {
        let session = self.session_index(session_name)?;
        let sess = &self.sessions[session];
        Some(self.window(session, sess.active).name.clone())
    }

    /// The active pane's index in `session` (tmux's `#P`). Used to fill in the
    /// `confirm-before` prompt text for `C-b x` (kill-pane), matching tmux's
    /// default `"kill-pane #P? (y/n)"`.
    pub(crate) fn active_pane_index(&self, session_name: &str) -> Option<usize> {
        let session = self.session_index(session_name)?;
        let sess = &self.sessions[session];
        Some(self.window(session, sess.active).active)
    }

    /// Stable identities for every pane displayed in the active window plus
    /// the active pane's position. Attach clients use this as the key for their
    /// shared compositor-output subscription.
    pub(crate) fn active_window_pane_identities(
        &self,
        session_name: &str,
    ) -> Option<(Vec<(u32, u64)>, usize)> {
        let session = self.session_index(session_name)?;
        let sess = &self.sessions[session];
        let win = self.window(session, sess.active);
        Some((
            win.panes
                .iter()
                .map(|pane| (pane.id, pane.pane.runtime_id()))
                .collect(),
            win.active,
        ))
    }

    pub(crate) fn control_snapshot(&self, session_name: &str) -> Option<ControlStateSnapshot> {
        let session_pos = self.session_index(session_name)?;
        let session = &self.sessions[session_pos];
        let active_window_id = session.windows.get(session.active)?.id;
        let mut windows = BTreeMap::new();
        for (position, link) in session.windows.iter().enumerate() {
            let window = self.windows.get(&link.id)?;
            let body = window.layout.dump();
            let checksum = body.bytes().fold(0u16, |sum, byte| {
                sum.rotate_right(1).wrapping_add(u16::from(byte))
            });
            let flags = self.printable_window_flags(session, position, false);
            windows.insert(
                window.id,
                ControlWindowSnapshot {
                    id: window.id,
                    index: link.index,
                    name: window.name.clone(),
                    layout: format!("{checksum:04x},{body}"),
                    flags,
                    active_pane_id: window.panes.get(window.active)?.id,
                    panes: window
                        .panes
                        .iter()
                        .map(|pane| ControlPaneSnapshot {
                            id: pane.id,
                            runtime_id: pane.pane.runtime_id(),
                            observation: pane.pane.observation_state(),
                        })
                        .collect(),
                },
            );
        }
        let sessions = self
            .sessions
            .iter()
            .map(|session| (session.id, session.name.clone()))
            .collect::<BTreeMap<_, _>>();
        let global_windows = self
            .windows
            .iter()
            .map(|(id, window)| {
                let links = self
                    .sessions
                    .iter()
                    .flat_map(|session| &session.windows)
                    .filter(|link| link.id == *id)
                    .count();
                (
                    *id,
                    ControlGlobalWindowSnapshot {
                        name: window.name.clone(),
                        links,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        let pane_modes = self
            .windows
            .values()
            .flat_map(|window| &window.panes)
            .map(|pane| (pane.id, pane.mode.clone()))
            .collect::<BTreeMap<_, _>>();
        let buffers = self
            .buffers
            .iter()
            .map(|(name, data)| {
                let mut hasher = DefaultHasher::new();
                data.hash(&mut hasher);
                (name.clone(), hasher.finish())
            })
            .collect::<BTreeMap<_, _>>();
        let clients = self
            .attached_clients()
            .into_iter()
            .filter_map(|client| {
                sessions
                    .get(&client.session_id)
                    .cloned()
                    .map(|name| (client.name, (client.session_id, name)))
            })
            .collect::<BTreeMap<_, _>>();
        Some(ControlStateSnapshot {
            session_id: session.id,
            session_name: session.name.clone(),
            active_window_id,
            windows,
            sessions,
            global_windows,
            pane_modes,
            buffers,
            clients,
        })
    }

    pub(crate) fn control_checkpoint_end(&self) -> u64 {
        self.next_control_checkpoint
    }

    pub(crate) fn record_control_checkpoint(&mut self) {
        let session_ids = self
            .sessions
            .iter()
            .map(|session| session.id)
            .collect::<Vec<_>>();
        let snapshots = session_ids
            .into_iter()
            .filter_map(|session_id| {
                self.control_snapshot(&format!("${session_id}"))
                    .map(|snapshot| (session_id, snapshot))
            })
            .collect::<BTreeMap<_, _>>();
        self.next_control_checkpoint = self.next_control_checkpoint.saturating_add(1);
        self.control_checkpoints.push_back(ControlCheckpoint {
            sequence: self.next_control_checkpoint,
            snapshots,
        });
        while self.control_checkpoints.len() > CONTROL_CHECKPOINT_LIMIT {
            self.control_checkpoints.pop_front();
        }
    }

    pub(crate) fn control_checkpoints_since(
        &self,
        session_id: u32,
        sequence: u64,
    ) -> (u64, Vec<ControlStateSnapshot>) {
        let snapshots = self
            .control_checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.sequence > sequence)
            .filter_map(|checkpoint| checkpoint.snapshots.get(&session_id).cloned())
            .collect();
        (self.next_control_checkpoint, snapshots)
    }

    #[cfg(test)]
    pub(crate) fn subscribe_active_pane_output(
        &self,
        session_name: &str,
    ) -> io::Result<super::pane::OutputSubscription> {
        self.active_pane(session_name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no active pane"))?
            .subscribe_output()
    }

    pub(crate) fn subscribe_active_window_output(
        &self,
        session_name: &str,
    ) -> io::Result<super::pane::OutputSubscription> {
        let (window, active) = self.active_window_panes(session_name)?;
        let active_pane = window
            .panes
            .get(active)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no active pane"))?;
        super::pane::OutputSubscription::for_panes(
            window.panes.iter().map(|pane| &pane.pane),
            &active_pane.pane,
        )
    }

    fn resize_active_window_to_session_size(&mut self, session: usize) -> io::Result<()> {
        let cols = self.sessions[session].cols;
        let rows = self.sessions[session].rows;
        let window_id = self.sessions[session].windows[self.sessions[session].active].id;
        let window = self.windows.get_mut(&window_id).expect("window present");
        window.layout.resize(cols, rows);
        resize_panes_to_layout(window)
    }

    /// Resize a session and its active pane to `cols`×`rows`.
    pub fn resize_session(&mut self, session_name: &str, cols: u16, rows: u16) -> io::Result<()> {
        let session = self.session_index(session_name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {session_name}"),
            )
        })?;
        self.sessions[session].cols = cols;
        self.sessions[session].rows = rows;
        self.resize_active_window_to_session_size(session)
    }

    /// Send input bytes to the active pane of a session.
    pub fn input_to_active_pane(&self, session_name: &str, bytes: &[u8]) -> io::Result<()> {
        if let Some(pane) = self.active_pane(session_name) {
            pane.input(bytes)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("no active pane for session: {session_name}"),
            ))
        }
    }

    /// Send input bytes to the pane selected by a tmux target expression.
    pub(crate) fn input_to_pane(&self, target: &str, bytes: &[u8]) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let window = self.window(resolved.session, resolved.window);
        if window.panes[resolved.pane].input_off {
            return Ok(());
        }
        let synchronized = window
            .options(&self.global_options)
            .get("synchronize-panes")
            .is_some_and(|value| value == "on" || value == "1");
        if synchronized {
            for pane in &window.panes {
                if !pane.input_off {
                    pane.pane.input(bytes)?;
                }
            }
            Ok(())
        } else {
            window.panes[resolved.pane].pane.input(bytes)
        }
    }

    pub(crate) fn input_mouse_to_pane(
        &self,
        target: &str,
        event: ghostty_sys::MouseEvent,
    ) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let pane = &self.window(resolved.session, resolved.window).panes[resolved.pane];
        if pane.input_off {
            return Ok(());
        }
        let bytes = pane.pane.encode_mouse(event)?;
        pane.pane.input(&bytes)
    }

    pub(crate) fn pane_bracketed_paste(&self, target: &str) -> io::Result<bool> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        Ok(
            self.window(resolved.session, resolved.window).panes[resolved.pane]
                .pane
                .bracketed_paste_enabled(),
        )
    }

    pub(crate) fn encode_pane_key(
        &self,
        target: &str,
        event: ghostty_sys::KeyEvent<'_>,
    ) -> io::Result<Vec<u8>> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        self.window(resolved.session, resolved.window).panes[resolved.pane]
            .pane
            .encode_key(event)
    }

    pub(crate) fn reset_pane_terminal(&self, target: &str) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        self.window(resolved.session, resolved.window).panes[resolved.pane]
            .pane
            .reset_terminal()
    }

    pub(crate) fn clear_pane_history(&mut self, target: &str) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let node = &mut self.window_mut(resolved.session, resolved.window).panes[resolved.pane];
        node.pane.clear_history()?;
        node.mode = None;
        node.copy = None;
        node.mode_view = None;
        self.invalidate_session(session_id, RenderInvalidation::RESET_MODE);
        Ok(())
    }

    pub(crate) fn pane_observation_state(
        &self,
        target: &str,
    ) -> io::Result<Arc<super::pane::NativePaneObservation>> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        Ok(
            self.window(resolved.session, resolved.window).panes[resolved.pane]
                .pane
                .observation_state(),
        )
    }

    pub(crate) fn append_view_output(&mut self, target: &str, output: &[u8]) -> io::Result<()> {
        if output.is_empty() {
            return Ok(());
        }
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let node = &mut self.window_mut(resolved.session, resolved.window).panes[resolved.pane];
        let (cols, rows) = node.pane.size();
        let (mut combined, previous_view) = match (
            &node.mode,
            node.copy.as_ref().map(|copy| (&copy.backing, copy)),
        ) {
            (Some(mode), Some((CopyBacking::ViewOutput(previous), copy)))
                if mode == "view-mode" =>
            {
                let top = copy.grid.scrollback_rows.saturating_sub(copy.scroll);
                (previous.clone(), Some((top, copy.cursor.clone())))
            }
            _ => (Vec::new(), None),
        };
        combined.extend_from_slice(output);
        let mut copy = view_copy_state(combined, cols, rows)?;
        if let Some((top, cursor)) = previous_view {
            copy.scroll = copy.grid.scrollback_rows.saturating_sub(top);
            copy.cursor.row = cursor.row.min(copy.grid.rows.len().saturating_sub(1));
            copy.cursor.col = cursor.col.min(copy.grid.cols.saturating_sub(1) as usize);
            copy.desired_col = copy.cursor.col;
        }
        node.mode = Some("view-mode".to_string());
        node.copy = Some(copy);
        node.mode_view = None;
        self.invalidate_session(session_id, RenderInvalidation::MODE);
        Ok(())
    }

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
                    let (grid, vt, (col, row)) = node.pane.copy_snapshot()?;
                    let vt_rows = copy_vt_row_ranges(&vt);
                    Some(CopyState {
                        backing: CopyBacking::PaneSnapshot,
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
                        search: node.search_string.as_ref().map(|pattern| CopySearch {
                            pattern: pattern.clone(),
                            regex: node.search_regex,
                            direction: CopySearchDirection::Backward,
                            last_direction: CopySearchDirection::Backward,
                            matches: Vec::new(),
                        }),
                        search_count: Some(0),
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
            self.notify_pane("pane-mode-changed", pane_id);
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
            self.notify_pane("pane-mode-changed", pane_id);
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
    pub(crate) fn mode_view_key(
        &mut self,
        target: &str,
        key: &str,
        visible_rows: usize,
    ) -> io::Result<ModeViewKeyResult> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let node = &mut self.window_mut(resolved.session, resolved.window).panes[resolved.pane];
        let Some(view) = node.mode_view.as_mut() else {
            return Ok(ModeViewKeyResult::None);
        };
        if view.kind == ModeKind::Clock {
            node.mode = None;
            node.mode_view = None;
            self.invalidate_session(session_id, RenderInvalidation::MODE);
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

        let last = view.items.len().saturating_sub(1);
        match key {
            "q" | "Escape" | "C-c" => {
                node.mode = None;
                node.mode_view = None;
            }
            "Up" | "k" | "C-p" => view.selected = view.selected.saturating_sub(1),
            "Down" | "j" | "C-n" => view.selected = (view.selected + 1).min(last),
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
        Ok(ModeViewKeyResult::None)
    }

    pub(crate) fn mode_view_search(&mut self, target: &str, search: &str) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let view = self.window_mut(resolved.session, resolved.window).panes[resolved.pane]
            .mode_view
            .as_mut()
            .ok_or_else(|| io::Error::other("not in a mode"))?;
        view.search = search.to_string();
        if !search.is_empty() && !view.items.is_empty() {
            let folded = search.to_lowercase();
            let start = (view.selected + 1) % view.items.len();
            if let Some(offset) = (0..view.items.len()).find(|offset| {
                view.items[(start + offset) % view.items.len()]
                    .label
                    .to_lowercase()
                    .contains(&folded)
            }) {
                view.selected = (start + offset) % view.items.len();
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
            source.pane.copy_snapshot()?
        };
        let session_id = self.sessions[target_resolved.session].id;
        let node = &mut self
            .window_mut(target_resolved.session, target_resolved.window)
            .panes[target_resolved.pane];
        if node.copy.is_none() {
            let (grid, vt, (col, row)) = snapshot;
            let vt_rows = copy_vt_row_ranges(&vt);
            node.copy = Some(CopyState {
                backing: CopyBacking::PaneSnapshot,
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
                hide_position,
                search: node.search_string.as_ref().map(|pattern| CopySearch {
                    pattern: pattern.clone(),
                    regex: node.search_regex,
                    direction: CopySearchDirection::Backward,
                    last_direction: CopySearchDirection::Backward,
                    matches: Vec::new(),
                }),
                search_count: Some(0),
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
            self.notify_pane("pane-mode-changed", pane_id);
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

    pub(crate) fn scroll_copy_to_mouse(
        &mut self,
        target: &str,
        y: u16,
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
        self.set_copy_scroll_from_mouse(target, y, height, vi)?;
        let exited = scroll_exit
            && self.window(resolved.session, resolved.window).panes[resolved.pane]
                .copy
                .as_ref()
                .is_some_and(|copy| copy.scroll == 0);
        if exited {
            let pane = &mut self.window_mut(resolved.session, resolved.window).panes[resolved.pane];
            pane.mode = None;
            pane.copy = None;
            pane.mode_view = None;
            self.invalidate_session(session_id, RenderInvalidation::MODE);
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
        let result = {
            let node = &mut self.window_mut(resolved.session, resolved.window).panes[resolved.pane];
            if node.copy.is_none() {
                return Err(io::Error::other("not in a mode"));
            }
            if command == "refresh-from-pane" {
                let (grid, vt, _) = node.pane.copy_snapshot()?;
                let state = node.copy.as_mut().expect("copy mode checked above");
                state.backing = CopyBacking::PaneSnapshot;
                state.grid = grid;
                state.replace_vt(vt);
                clamp_copy_state(state, vi);
                ensure_copy_cursor_visible(state);
                if let Some(search) = state.search.as_mut() {
                    search.matches.clear();
                }
                state.search_count = None;
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
                            state.cursor.col = 0;
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
                            for _ in 0..prefix {
                                copy_reader_cursor_right(&mut state.cursor, &state.grid);
                            }
                            ensure_copy_cursor_visible(state);
                            None
                        }
                        "back-to-indentation" => {
                            state.cursor.col = copy_first_nonblank(&state.grid, state.cursor.row);
                            None
                        }
                        "end-of-line" => {
                            state.cursor.col = copy_cursor_limit(&state.grid, state.cursor.row, vi);
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
                                    std::mem::swap(&mut selection.anchor, &mut selection.end);
                                    state.cursor.row = selection.end.0;
                                    state.cursor.col = selection.end.1;
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
                                state.cursor.col =
                                    copy_cursor_limit(&state.grid, state.cursor.row, vi);
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
                                move_previous(&mut state.cursor, &state.grid, vi, separators);
                            }
                            ensure_copy_cursor_visible(state);
                            None
                        }
                        "previous-space" => {
                            for _ in 0..prefix {
                                move_previous(&mut state.cursor, &state.grid, vi, "");
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
                            let data = copy_selection(state, vi);
                            if !command.ends_with("no-clear") {
                                state.selection = None;
                            }
                            end_mode = command.ends_with("and-cancel");
                            Some(data)
                        }
                        "copy-end-of-line"
                        | "copy-end-of-line-and-cancel"
                        | "copy-pipe-end-of-line"
                        | "copy-pipe-end-of-line-and-cancel" => {
                            let data = copy_from_cursor_to_line_end(state, vi);
                            state.selection = None;
                            end_mode = command.ends_with("and-cancel");
                            Some(data)
                        }
                        "copy-line"
                        | "copy-line-and-cancel"
                        | "copy-pipe-line"
                        | "copy-pipe-line-and-cancel" => {
                            let data = copy_current_line(state, vi);
                            state.selection = None;
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
                }
                output
            }
        };
        self.invalidate_session(session_id, RenderInvalidation::MODE);
        // A copy that produced data writes the terminal selection unless
        // `set-clipboard` is `off`, and tmux notifies the pane whenever it
        // does — including under `external`, where the write is the client's.
        if result.is_some()
            && self
                .option_for_target(target, "set-clipboard")
                .is_none_or(|value| value != "off")
        {
            let pane_id = self.window(resolved.session, resolved.window).panes[resolved.pane].id;
            self.notify_pane("pane-set-clipboard", pane_id);
        }
        Ok(result)
    }

    pub(crate) fn respawn_pane_process(
        &mut self,
        target: &str,
        argv: Option<Vec<String>>,
        cwd: Option<PathBuf>,
    ) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let (cols, rows, saved) = {
            let node = &self.window(resolved.session, resolved.window).panes[resolved.pane];
            let (cols, rows) = node.pane.size();
            (cols, rows, node.pane.spawn_spec())
        };
        let spec = match argv {
            Some(argv) if !argv.is_empty() => PaneSpawnSpec { argv, cwd },
            _ => match saved {
                Some(spec) => spec,
                None => return Ok(()),
            },
        };
        let pane = Pane::spawn_from_spec_mode(&spec, cols, rows, self.pane_io_mode)?;
        let node = &mut self.window_mut(resolved.session, resolved.window).panes[resolved.pane];
        node.pane = pane;
        node.mode = None;
        node.copy = None;
        node.mode_view = None;
        self.invalidate_session(
            session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS | RenderInvalidation::MODE,
        );
        Ok(())
    }

    pub(crate) fn respawn_window_process(
        &mut self,
        target: &str,
        argv: Option<Vec<String>>,
        cwd: Option<PathBuf>,
    ) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find window: {target}"),
            )
        })?;
        let window_id = self.sessions[resolved.session].windows[resolved.window].id;
        let affected_sessions = self
            .sessions
            .iter()
            .filter(|session| session.windows.iter().any(|link| link.id == window_id))
            .map(|session| session.id)
            .collect::<Vec<_>>();
        if argv.as_ref().is_none_or(Vec::is_empty) {
            let io_mode = self.pane_io_mode;
            let replacements = self
                .window(resolved.session, resolved.window)
                .panes
                .iter()
                .map(|node| {
                    let Some(spec) = node.pane.spawn_spec() else {
                        return Ok(None);
                    };
                    let (cols, rows) = node.pane.size();
                    Pane::spawn_from_spec_mode(&spec, cols, rows, io_mode).map(Some)
                })
                .collect::<io::Result<Vec<_>>>()?;
            let window = self.window_mut(resolved.session, resolved.window);
            for (node, pane) in window.panes.iter_mut().zip(replacements) {
                if let Some(pane) = pane {
                    node.pane = pane;
                    node.mode = None;
                    node.copy = None;
                    node.mode_view = None;
                }
            }
            for session_id in affected_sessions {
                self.invalidate_session(
                    session_id,
                    RenderInvalidation::LAYOUT
                        | RenderInvalidation::STATUS
                        | RenderInvalidation::MODE,
                );
            }
            return Ok(());
        }
        let argv = argv.expect("nonempty argv checked above");
        let pane = {
            let session = &self.sessions[resolved.session];
            let window = self.window(resolved.session, resolved.window);
            let cols = window.manual_size.map_or(session.cols, |size| size.0);
            let rows = window.manual_size.map_or(session.rows, |size| size.1);
            let spec = PaneSpawnSpec { argv, cwd };
            Pane::spawn_from_spec_mode(&spec, cols, rows, self.pane_io_mode)?
        };
        let window = self.window_mut(resolved.session, resolved.window);
        let id = window.panes[resolved.pane].id;
        window.panes.clear();
        window.panes.push(PaneNode {
            id,
            pane,
            start_command: String::new(),
            input_off: false,
            title: None,
            exit_notified: false,
            mode: None,
            copy: None,
            mode_view: None,
            search_string: None,
            search_regex: false,
            floating: None,
            options: OptionSet::default(),
        });
        window.layout.keep_only(id);
        resize_panes_to_layout(window)?;
        window.active = 0;
        window.last_pane = None;
        for session_id in affected_sessions {
            self.invalidate_session(
                session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS | RenderInvalidation::MODE,
            );
        }
        Ok(())
    }

    /// Instrumented input path for the attach latency monitor. The public input
    /// method above keeps its existing API while this reports whether bytes
    /// reached the non-blocking PTY immediately, remained queued, or were
    /// dropped because the bounded queue was full.
    pub(crate) fn input_to_active_pane_with_stats(
        &self,
        session_name: &str,
        bytes: &[u8],
    ) -> io::Result<super::pane::PaneInputStats> {
        if let Some(pane) = self.active_pane(session_name) {
            pane.input_with_stats(bytes)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("no active pane for session: {session_name}"),
            ))
        }
    }

    /// Drain terminal queries waiting to be sent to the attached client.
    pub fn take_active_pane_terminal_queries(
        &self,
        session_name: &str,
    ) -> io::Result<Vec<Vec<u8>>> {
        let pane = self
            .active_pane(session_name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no active pane"))?;
        Ok(pane.take_terminal_queries())
    }

    /// Dump the active pane as VT sequences, if present.
    pub fn dump_active_pane_vt(&self, session_name: &str) -> io::Result<Vec<u8>> {
        let pane = self
            .active_pane(session_name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no active pane"))?;
        pane.dump_vt()
    }

    pub(crate) fn dump_active_pane_viewport_vt(
        &self,
        session_name: &str,
        scroll_offset: usize,
        visible_rows: usize,
    ) -> io::Result<(Vec<u8>, usize)> {
        let pane = self
            .active_pane(session_name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no active pane"))?;
        pane.dump_viewport_vt(scroll_offset, visible_rows)
    }

    pub(crate) fn active_window_panes(&self, session_name: &str) -> io::Result<(&Window, usize)> {
        let session = self
            .session_index(session_name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no active session"))?;
        let sess = &self.sessions[session];
        let win = self.window(session, sess.active);
        Ok((win, win.active))
    }

    pub(crate) fn active_copy_state(&self, session_name: &str) -> Option<&CopyState> {
        let (window, active) = self.active_window_panes(session_name).ok()?;
        window.panes.get(active)?.copy.as_ref()
    }

    pub(crate) fn active_mode_view(&self, session_name: &str) -> Option<&ModeView> {
        let (window, active) = self.active_window_panes(session_name).ok()?;
        window.panes.get(active)?.mode_view.as_ref()
    }

    /// Whether the active pane's cursor is visible (DEC mode 25), for the
    /// compositor to mirror onto the client tty.
    pub fn active_pane_cursor_visible(&self, session_name: &str) -> io::Result<bool> {
        let pane = self
            .active_pane(session_name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no active pane"))?;
        pane.cursor_visible()
    }

    /// DECSCUSR parameter selected by the active pane.
    pub fn active_pane_cursor_shape(&self, session_name: &str) -> io::Result<u8> {
        let pane = self
            .active_pane(session_name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no active pane"))?;
        Ok(pane.cursor_shape())
    }

    /// How many leading scrollback (history) rows the active pane's VT dump
    /// carries before the visible viewport. The compositor skips these so it
    /// paints the on-screen rows, not the oldest history (see `report.md`).
    pub fn active_pane_scrollback_rows(&self, session_name: &str) -> io::Result<usize> {
        let pane = self
            .active_pane(session_name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no active pane"))?;
        pane.scrollback_rows()
    }

    /// How many leading scrollback (history) rows the pane named by `target`
    /// carries before its viewport. `capture-pane -p` skips these to print the
    /// visible screen rather than the top of history.
    pub fn pane_scrollback_rows(&self, target: &str) -> io::Result<usize> {
        let t = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        self.window(t.session, t.window).panes[t.pane]
            .pane
            .scrollback_rows()
    }

    /// Dump the active pane as plain text (for debugging / tests).
    pub fn dump_active_pane_plain(&self, session_name: &str) -> io::Result<String> {
        let pane = self
            .active_pane(session_name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no active pane"))?;
        pane.dump()
    }
}

fn option_affects_render(key: &str) -> bool {
    matches!(
        key,
        "pane-border-indicators"
            | "set-titles"
            | "set-titles-string"
            | "status"
            | "status-interval"
            | "status-justify"
            | "status-left"
            | "status-left-length"
            | "status-left-style"
            | "status-position"
            | "status-right"
            | "status-right-length"
            | "status-right-style"
            | "status-style"
            | "status-format"
            | "terminal-features"
            | "terminal-overrides"
            | "window-pane-current-status-format"
            | "window-pane-status-format"
            | "pane-status-current-style"
            | "pane-status-style"
            | "session-status-current-style"
            | "session-status-style"
            | "window-status-activity-style"
            | "window-status-bell-style"
            | "window-status-current-format"
            | "window-status-current-style"
            | "window-status-format"
            | "window-status-last-style"
            | "window-status-separator"
            | "window-status-style"
    ) || key.starts_with("status-format[")
        || key.starts_with("terminal-features[")
        || key.starts_with("terminal-overrides[")
}

fn option_invalidation(key: &str) -> RenderInvalidation {
    if matches!(key, "terminal-features" | "terminal-overrides")
        || key.starts_with("terminal-features[")
        || key.starts_with("terminal-overrides[")
    {
        RenderInvalidation::TERMINAL | RenderInvalidation::STATUS
    } else if key == "status" || key == "status-position" {
        RenderInvalidation::STATUS | RenderInvalidation::LAYOUT
    } else if key == "pane-border-indicators" {
        RenderInvalidation::LAYOUT
    } else {
        RenderInvalidation::STATUS
    }
}

/// Split a pane target `session[:window][.pane]` into its window-target part and
/// the optional pane part (the text after the last `.`).
fn split_pane_target(target: &str) -> (&str, Option<&str>) {
    match target.rsplit_once('.') {
        Some((win, pane)) => (win, Some(pane)),
        None => (target, None),
    }
}

/// Parse a `[session][:index]` destination into `(session, index)`. A bare
/// number with no colon is an index in the current session.
fn parse_index_target(target: &str) -> (Option<&str>, Option<u32>) {
    match target.split_once(':') {
        Some((sess, idx)) => {
            let session = if sess.is_empty() { None } else { Some(sess) };
            (session, idx.parse().ok())
        }
        None => match target.parse::<u32>() {
            Ok(n) => (None, Some(n)),
            Err(_) => (Some(target), None),
        },
    }
}

/// Resolve one of tmux's special window tokens within a session to a `Vec`
/// position, relative to the session's active/last window. Returns `None` if the
/// token isn't a special form (the caller then tries a numeric index).
///
/// Handled: `{start}`/`^` (first), `{end}`/`$` (last), `{last}` (previous),
/// `{next}`/`+` (next by order, wrapping), `{previous}`/`-` (previous by order).
fn window_special(sess: &Session, spec: &str) -> Option<usize> {
    let n = sess.windows.len();
    if n == 0 {
        return None;
    }
    match spec {
        "{start}" | "^" => Some(0),
        "{end}" | "$" => Some(n - 1),
        "{last}" => sess.last_active.filter(|&p| p < n),
        "{next}" | "+" => Some((sess.active + 1) % n),
        "{previous}" | "-" => Some((sess.active + n - 1) % n),
        _ => None,
    }
}

/// Resolve a pane spec within `win` to a `Vec` position: a numeric index, a
/// `%id`, or a special token (`{top}`, `{bottom}`, `{last}`, `{next}`/`+`,
/// `{previous}`/`-`). Returns `None` on a miss.
fn pane_pos_in(win: &Window, spec: &str, base: usize) -> Option<usize> {
    let n = win.panes.len();
    if n == 0 {
        return None;
    }
    if let Some(id) = spec.strip_prefix('%') {
        let id: u32 = id.parse().ok()?;
        return win.panes.iter().position(|p| p.id == id);
    }
    match spec {
        "{top}" => return Some(0),
        "{bottom}" => return Some(n - 1),
        "{last}" => return win.last_pane.filter(|&p| p < n),
        "{next}" | "+" => return Some((win.active + 1) % n),
        "{previous}" | "-" => return Some((win.active + n - 1) % n),
        _ => {}
    }
    let idx: usize = spec.parse().ok()?;
    let idx = idx.checked_sub(base)?;
    (idx < n).then_some(idx)
}

fn copy_cell_is_padding(cell: &ghostty_sys::GridCellSnapshot) -> bool {
    matches!(
        cell.width,
        ghostty_sys::GridCellWidth::SpacerTail | ghostty_sys::GridCellWidth::SpacerHead
    )
}

fn copy_line_length(grid: &ghostty_sys::GridSnapshot, row: usize) -> usize {
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

fn copy_cell_in_set(grid: &ghostty_sys::GridSnapshot, cursor: &CopyCursor, set: &str) -> bool {
    let Some(cell) = grid
        .rows
        .get(cursor.row)
        .and_then(|row| row.cells.get(cursor.col))
    else {
        return set.contains(' ');
    };
    if copy_cell_is_padding(cell) {
        return false;
    }
    if cell.text.is_empty() {
        return set.contains(' ');
    }
    let mut chars = cell.text.chars();
    let Some(ch) = chars.next() else {
        return set.contains(' ');
    };
    chars.next().is_none() && set.chars().any(|candidate| candidate == ch)
}

fn copy_cursor_limit(grid: &ghostty_sys::GridSnapshot, row: usize, vi: bool) -> usize {
    let length = copy_line_length(grid, row);
    if vi && length != 0 {
        length - 1
    } else {
        length
    }
}

fn clamp_copy_cursor(cursor: &mut CopyCursor, grid: &ghostty_sys::GridSnapshot, vi: bool) {
    cursor.row = cursor.row.min(grid.rows.len().saturating_sub(1));
    cursor.col = cursor.col.min(copy_cursor_limit(grid, cursor.row, vi));
}

fn clamp_copy_point(point: &mut (usize, usize), grid: &ghostty_sys::GridSnapshot, vi: bool) {
    point.0 = point.0.min(grid.rows.len().saturating_sub(1));
    point.1 = point.1.min(copy_cursor_limit(grid, point.0, vi));
}

fn clamp_copy_state(state: &mut CopyState, vi: bool) {
    clamp_copy_cursor(&mut state.cursor, &state.grid, vi);
    state.desired_col = state.desired_col.min(state.grid.cols as usize);
    if let Some(selection) = state.selection.as_mut() {
        clamp_copy_point(&mut selection.anchor, &state.grid, vi);
        clamp_copy_point(&mut selection.end, &state.grid, vi);
    }
}

fn copy_view_top(state: &CopyState) -> usize {
    state.grid.scrollback_rows.saturating_sub(state.scroll)
}

fn ensure_copy_cursor_visible(state: &mut CopyState) {
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

fn move_copy_row(state: &mut CopyState, up: bool, vi: bool) {
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

fn move_copy_page(state: &mut CopyState, up: bool, vi: bool) {
    let amount = (state.grid.viewport_rows as usize).saturating_sub(2).max(1);
    move_copy_rows(state, up, amount, vi);
}

fn move_copy_rows(state: &mut CopyState, up: bool, amount: usize, vi: bool) {
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
enum CopyViewLine {
    Top,
    Middle,
    Bottom,
}

fn move_copy_view_line(state: &mut CopyState, line: CopyViewLine) {
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

fn copy_cursor_view_y(state: &CopyState) -> usize {
    state.cursor.row.saturating_sub(copy_view_top(state))
}

fn centre_copy_cursor_vertical(state: &mut CopyState, vi: bool) {
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

fn centre_copy_cursor_horizontal(state: &mut CopyState, vi: bool) {
    let centre = state.grid.cols as usize / 2;
    state.cursor.col = if state.rectangle {
        centre
    } else {
        centre.min(copy_cursor_limit(&state.grid, state.cursor.row, vi))
    };
}

fn position_copy_cursor(state: &mut CopyState, row: usize, col: usize) {
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

fn move_copy_paragraph(state: &mut CopyState, backward: bool) {
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

fn align_copy_cursor_in_view(state: &mut CopyState, target: usize) {
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

fn scroll_copy_content(state: &mut CopyState, up: bool, vi: bool) {
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

fn goto_copy_line(state: &mut CopyState, argument: &str, absolute: bool) {
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

fn recentre_copy_cursor(state: &mut CopyState) {
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

fn copy_ascii_cell(grid: &ghostty_sys::GridSnapshot, row: usize, col: usize) -> Option<u8> {
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
    grid: &ghostty_sys::GridSnapshot,
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
    grid: &ghostty_sys::GridSnapshot,
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

fn move_copy_matching_bracket(state: &mut CopyState, backward: bool, vi: bool) {
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
            move_previous(&mut cursor, &state.grid, false, "}])");
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

fn copy_first_nonblank(grid: &ghostty_sys::GridSnapshot, row: usize) -> usize {
    let limit = copy_line_length(grid, row);
    (0..limit)
        .find(|&col| {
            let cursor = CopyCursor { row, col };
            !copy_cell_in_set(grid, &cursor, " \t")
        })
        .unwrap_or(limit)
}

fn move_copy_prompt(state: &mut CopyState, forward: bool, output: bool) {
    let is_prompt_row = |row: usize| {
        state.grid.rows[row]
            .cells
            .iter()
            .any(|cell| cell.semantic == ghostty_sys::GridCellSemantic::Prompt)
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

fn repeat_copy_jump(state: &mut CopyState, reverse: bool) {
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

fn synchronize_copy_selection(state: &mut CopyState) {
    if let Some(selection) = state
        .selection
        .as_mut()
        .filter(|selection| selection.active)
    {
        selection.end = (state.cursor.row, state.cursor.col);
    }
}

fn select_copy_line(state: &mut CopyState, vi: bool) {
    let row = state.cursor.row;
    let end = copy_cursor_limit(&state.grid, row, vi);
    state.cursor.col = end;
    state.selection = Some(CopySelection {
        anchor: (row, 0),
        end: (row, end),
        active: true,
    });
}

fn copy_word_class(grid: &ghostty_sys::GridSnapshot, cursor: &CopyCursor, separators: &str) -> u8 {
    if copy_cell_in_set(grid, cursor, " \t") {
        0
    } else if copy_cell_in_set(grid, cursor, separators) {
        1
    } else {
        2
    }
}

fn select_copy_word(state: &mut CopyState, vi: bool, separators: &str) {
    let row = state.cursor.row;
    let length = copy_line_length(&state.grid, row);
    if length == 0 {
        return;
    }
    let col = state.cursor.col.min(length - 1);
    let cursor = CopyCursor { row, col };
    let class = copy_word_class(&state.grid, &cursor, separators);
    let mut start = col;
    while start > 0 {
        let candidate = CopyCursor {
            row,
            col: start - 1,
        };
        if copy_word_class(&state.grid, &candidate, separators) != class {
            break;
        }
        start -= 1;
    }
    let mut end = col + 1;
    while end < length {
        let candidate = CopyCursor { row, col: end };
        if copy_word_class(&state.grid, &candidate, separators) != class {
            break;
        }
        end += 1;
    }
    let endpoint = if vi { end - 1 } else { end };
    state.cursor.col = endpoint;
    state.selection = Some(CopySelection {
        anchor: (row, start),
        end: (row, endpoint),
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

fn search_uses_posix_regex(pattern: &str) -> bool {
    pattern.bytes().any(|byte| b"^$*+()?[].\\".contains(&byte))
}

fn copy_search_matches(
    grid: &ghostty_sys::GridSnapshot,
    pattern: &str,
    regex: bool,
) -> Vec<CopySearchMatch> {
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
                let width = usize::from(matches!(cell.width, ghostty_sys::GridCellWidth::Wide)) + 1;
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

fn start_copy_search(
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
    state.search_count = Some(
        state
            .search
            .as_ref()
            .map(|search| search.matches.len())
            .unwrap_or(0),
    );
}

fn incremental_search_argument(argument: &str) -> Option<(char, &str)> {
    let mut chars = argument.char_indices();
    let (_, prefix) = chars.next()?;
    let pattern_at = chars.next().map(|(at, _)| at).unwrap_or(argument.len());
    Some((prefix, &argument[pattern_at..]))
}

fn incremental_copy_search(
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
        state.search_count = None;
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
        state.search_count = None;
    }
}

fn repeat_copy_search(state: &mut CopyState, reverse: bool, vi: bool, wrap: bool) {
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
    state.search_count = Some(
        state
            .search
            .as_ref()
            .map(|search| search.matches.len())
            .unwrap_or(0),
    );
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

fn clear_copy_search_marks_after_command(state: &mut CopyState, command: &str, vi: bool) {
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
    state.search_count = None;
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

fn copy_reader_handle_wrap(
    cursor: &mut CopyCursor,
    grid: &ghostty_sys::GridSnapshot,
    line_end: &mut usize,
) -> bool {
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

fn copy_reader_cursor_right(cursor: &mut CopyCursor, grid: &ghostty_sys::GridSnapshot) {
    let end = copy_line_length(grid, cursor.row).saturating_sub(1);
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

fn copy_reader_cursor_left(cursor: &mut CopyCursor, grid: &ghostty_sys::GridSnapshot, wrap: bool) {
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

fn move_previous(
    cursor: &mut CopyCursor,
    grid: &ghostty_sys::GridSnapshot,
    vi: bool,
    separators: &str,
) {
    if grid.rows.is_empty() {
        return;
    }
    let stop_at_eol = !vi;
    let word_is_letters;

    loop {
        if cursor.col > 0 {
            cursor.col -= 1;
            if !copy_cell_in_set(grid, cursor, " \t") {
                word_is_letters = !copy_cell_in_set(grid, cursor, separators);
                break;
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
                    word_is_letters = false;
                    break;
                }
            }
        }
    }

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

fn move_next_start(
    cursor: &mut CopyCursor,
    grid: &ghostty_sys::GridSnapshot,
    vi: bool,
    separators: &str,
) {
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
    while copy_reader_handle_wrap(cursor, grid, &mut line_end)
        && copy_cell_in_set(grid, cursor, " \t")
    {
        cursor.col += 1;
    }
    clamp_copy_cursor(cursor, grid, vi);
}

fn copy_reader_next_word_end(
    cursor: &mut CopyCursor,
    grid: &ghostty_sys::GridSnapshot,
    separators: &str,
) {
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

fn move_next_end(
    cursor: &mut CopyCursor,
    grid: &ghostty_sys::GridSnapshot,
    vi: bool,
    separators: &str,
) {
    if grid.rows.is_empty() {
        return;
    }
    if vi {
        if !copy_cell_in_set(grid, cursor, " \t") {
            copy_reader_cursor_right(cursor, grid);
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

fn append_copy_cells(
    output: &mut String,
    grid: &ghostty_sys::GridSnapshot,
    row: usize,
    from: usize,
    to: usize,
) {
    if from >= to {
        return;
    }
    for cell in &grid.rows[row].cells[from..to] {
        match cell.width {
            ghostty_sys::GridCellWidth::SpacerTail => continue,
            ghostty_sys::GridCellWidth::SpacerHead => {
                output.push(' ');
                continue;
            }
            ghostty_sys::GridCellWidth::Narrow | ghostty_sys::GridCellWidth::Wide => {}
        }
        if cell.text.is_empty() {
            output.push(' ');
        } else {
            output.push_str(&cell.text);
        }
    }
}

fn copy_selection(state: &CopyState, vi: bool) -> String {
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
        let mut output = String::new();
        for (index, (row, from, to)) in segments.into_iter().enumerate() {
            if index != 0 {
                output.push('\n');
            }
            append_copy_cells(&mut output, grid, row, from, to);
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

fn copy_from_cursor_to_line_end(state: &CopyState, vi: bool) -> String {
    let mut temporary = state.clone();
    let row = temporary.cursor.row;
    let end = copy_cursor_limit(&temporary.grid, row, vi);
    temporary.selection = Some(CopySelection {
        anchor: (row, temporary.cursor.col),
        end: (row, end),
        active: false,
    });
    copy_selection(&temporary, vi)
}

fn copy_current_line(state: &CopyState, vi: bool) -> String {
    let mut temporary = state.clone();
    let row = temporary.cursor.row;
    let end = copy_cursor_limit(&temporary.grid, row, vi);
    temporary.selection = Some(CopySelection {
        anchor: (row, 0),
        end: (row, end),
        active: false,
    });
    copy_selection(&temporary, vi)
}

/// tmux's `can't find pane: <part>` error.
fn pane_not_found(part: &str) -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, format!("can't find pane: {part}"))
}

/// tmux's `can't find window: <part>` error.
fn window_not_found(part: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::NotFound,
        format!("can't find window: {part}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_vt_rows_exclude_crlf_and_trailing_cursor() {
        let vt = b"one\r\ntwo\r\nlast\x1b[3;4H";
        let rows = copy_vt_row_ranges(vt)
            .into_iter()
            .map(|range| &vt[range])
            .collect::<Vec<_>>();
        assert_eq!(rows, [b"one".as_slice(), b"two", b"last"]);
    }

    #[test]
    fn copy_vt_rows_preserve_content_without_cursor_sequence() {
        let vt = b"first\nsecond\r";
        let rows = copy_vt_row_ranges(vt)
            .into_iter()
            .map(|range| &vt[range])
            .collect::<Vec<_>>();
        assert_eq!(rows, [b"first".as_slice(), b"second"]);
        assert_eq!(
            copy_vt_row_ranges(b""),
            std::iter::once(0..0).collect::<Vec<_>>()
        );
    }

    #[test]
    fn copy_selection_preserves_eight_leading_spaces() {
        let text = "        Indented";
        let mut cells = text
            .chars()
            .map(|ch| ghostty_sys::GridCellSnapshot {
                text: ch.to_string(),
                width: ghostty_sys::GridCellWidth::Narrow,
                semantic: ghostty_sys::GridCellSemantic::Output,
                hyperlink: None,
                hyperlink_id: None,
            })
            .collect::<Vec<_>>();
        cells.resize(
            20,
            ghostty_sys::GridCellSnapshot {
                text: String::new(),
                width: ghostty_sys::GridCellWidth::Narrow,
                semantic: ghostty_sys::GridCellSemantic::Output,
                hyperlink: None,
                hyperlink_id: None,
            },
        );
        let grid = ghostty_sys::GridSnapshot {
            cols: 20,
            viewport_rows: 1,
            scrollback_rows: 0,
            rows: vec![ghostty_sys::GridRowSnapshot {
                cells,
                wrapped: false,
            }],
        };
        let state = CopyState {
            backing: CopyBacking::PaneSnapshot,
            cursor: CopyCursor { row: 0, col: 8 },
            desired_col: 8,
            selection: Some(CopySelection {
                anchor: (0, 0),
                end: (0, 8),
                active: true,
            }),
            rectangle: false,
            selection_mode: CopySelectionMode::Character,
            mark: None,
            jump: None,
            hide_position: false,
            search: None,
            search_count: Some(0),
            incremental_search_origin: None,
            prefix: 1,
            scroll_exit: false,
            recentre: CopyRecentre {
                state: CopyRecentreState::Middle,
                line: 0,
            },
            grid,
            vt: Vec::new(),
            vt_rows: std::iter::once(0..0).collect(),
            scroll: 0,
        };
        assert_eq!(copy_selection(&state, false), "        ");
    }

    #[test]
    fn client_prompt_registry_routes_by_tty() {
        let registry = Arc::new(ClientPromptRegistry::new());
        let client_a = registry
            .attach("/dev/pts/7".to_string(), Some(700), 7)
            .expect("attach client A");
        let client_b = registry
            .attach("/dev/pts/8".to_string(), Some(800), 8)
            .expect("attach client B");

        let requester = Arc::clone(&registry);
        let request = std::thread::spawn(move || {
            requester.request_command(
                Some("/dev/pts/7"),
                None,
                vec!["command-prompt".to_string()],
                true,
            )
        });
        let mut prompt_fd = libc::pollfd {
            fd: client_a.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        assert_eq!(unsafe { libc::poll(&mut prompt_fd, 1, 1_000) }, 1);
        assert!(client_b.take_command_prompt().is_none());
        let prompt = client_a.take_command_prompt().expect("client A prompt");
        assert_eq!(prompt.args(), &["command-prompt".to_string()]);
        prompt.complete(PromptCompletion {
            stdout: "Up".to_string(),
            stderr: String::new(),
            exit: 0,
            inserted: true,
        });
        assert!(matches!(
            request.join().expect("request thread"),
            CommandPromptRequestResult::Completed(result)
                if result.stdout == "Up" && result.exit == 0
        ));
    }

    #[test]
    fn detaching_client_cancels_its_queued_prompt() {
        let registry = Arc::new(ClientPromptRegistry::new());
        let client = registry
            .attach("/dev/pts/7".to_string(), Some(700), 7)
            .expect("attach client");
        let requester = Arc::clone(&registry);
        let request = std::thread::spawn(move || {
            requester.request_command(
                Some("/dev/pts/7"),
                None,
                vec!["command-prompt".to_string()],
                true,
            )
        });
        let mut prompt_fd = libc::pollfd {
            fd: client.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        assert_eq!(unsafe { libc::poll(&mut prompt_fd, 1, 1_000) }, 1);
        drop(client);
        assert!(matches!(
            request.join().expect("request thread"),
            CommandPromptRequestResult::Completed(PromptCompletion {
                stdout,
                stderr,
                exit: 0,
                ..
            }) if stdout.is_empty() && stderr.is_empty()
        ));
    }

    #[test]
    fn default_state_has_one_session() {
        let state = ServerState::with_test_session().expect("state");
        assert_eq!(state.sessions().len(), 1);
        assert_eq!(state.sessions()[0].name, "0");
        assert!(state.summary_contains("0: 1 windows"));
    }

    #[test]
    fn selecting_window_applies_the_sessions_current_size() {
        let mut state = ServerState::with_test_session().expect("state");
        state
            .new_window("0", None, false)
            .expect("create inactive window");
        state.resize_session("0", 20, 4).expect("resize session");

        assert_eq!(state.window(0, 0).panes[0].pane.size(), (20, 4));
        assert_eq!(
            state.window(0, 1).panes[0].pane.size(),
            (80, 24),
            "inactive window retains its previous size"
        );

        state.select_window("0:1").expect("select inactive window");
        assert_eq!(
            state.window(0, 1).panes[0].pane.size(),
            (20, 4),
            "selected window inherits the stored session viewport"
        );
    }

    #[test]
    fn view_mode_appends_output_without_moving_the_viewport() {
        let mut state = ServerState::with_test_session().expect("state");
        state.resize_session("0", 20, 4).expect("resize");
        state
            .append_view_output("0", b"FIRST_VIEW_LINE\n")
            .expect("enter view mode");
        let first_top = {
            let copy = state.active_copy_state("0").expect("view backing");
            copy.grid.scrollback_rows.saturating_sub(copy.scroll)
        };

        let mut tail = Vec::new();
        for index in 0..20 {
            tail.extend_from_slice(format!("tail-{index:02}\n").as_bytes());
        }
        state
            .append_view_output("0", &tail)
            .expect("append view output");

        let (window, active) = state.active_window_panes("0").expect("active pane");
        let pane = &window.panes[active];
        assert_eq!(pane.mode.as_deref(), Some("view-mode"));
        let copy = pane.copy.as_ref().expect("view backing");
        assert_eq!(
            copy.grid.scrollback_rows.saturating_sub(copy.scroll),
            first_top
        );
        match &copy.backing {
            CopyBacking::ViewOutput(output) => {
                assert!(output.starts_with(b"FIRST_VIEW_LINE\n"));
                assert!(output.ends_with(b"tail-19\n"));
            }
            CopyBacking::PaneSnapshot => panic!("view mode used a pane snapshot"),
        }
    }

    #[test]
    fn respawn_window_replaces_pane_with_running_command() {
        let mut state = ServerState::with_test_session().expect("state");
        state
            .respawn_window_process(
                "0:0",
                Some(vec![
                    "/bin/sh".into(),
                    "-c".into(),
                    "printf RESPAWNED; sleep 1".into(),
                ]),
                None,
            )
            .expect("respawn");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
        loop {
            let dump = state.dump_pane("0:0").expect("dump");
            if dump.contains("RESPAWNED") {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "replacement pane never produced output: {dump:?}"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    #[test]
    fn create_and_kill_session() {
        let mut state = ServerState::with_test_session().expect("state");
        state
            .create_session("work", PaneSpec::Inert)
            .expect("create");
        assert!(state.find("work").is_some());
        assert!(state.kill_session("work"));
        assert!(state.find("work").is_none());
        assert!(!state.kill_session("work")); // already gone
    }

    #[test]
    fn killing_last_window_requests_exit_empty_shutdown() {
        let mut state = ServerState::with_test_session().expect("state");
        state.kill_window("0:0").expect("kill last window");
        assert!(state.sessions().is_empty());
        assert!(state.shutdown_requested());
    }

    #[test]
    fn killing_last_session_requests_exit_empty_shutdown() {
        let mut state = ServerState::with_test_session().expect("state");
        assert!(state.kill_session("0"));
        assert!(state.shutdown_requested());
    }

    #[test]
    fn duplicate_session_rejected() {
        let mut state = ServerState::with_test_session().expect("state");
        let err = state.create_session("0", PaneSpec::Inert).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
    }

    #[test]
    fn exited_last_pane_requests_exit_empty_shutdown() {
        let mut state = ServerState::empty();
        state
            .create_session(
                "0",
                PaneSpec::Command(vec!["/bin/sh".into(), "-c".into(), "exit 0".into()]),
            )
            .expect("create short-lived pane");

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while !state.active_pane("0").is_some_and(Pane::has_exited) {
            assert!(
                std::time::Instant::now() < deadline,
                "pane child did not exit"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        assert!(state.reap_exited_panes());
        assert!(state.sessions().is_empty());
        assert!(state.shutdown_requested());
    }

    #[test]
    fn select_window_accepts_bare_and_empty_session_targets() {
        let mut state = ServerState::with_test_session().expect("state");
        // Session "0" starts with a single window at index 0; add index 1.
        state.new_window("0", Some(1), true).expect("new window");

        // Real tmux: a bare number is a *window* index in the current session.
        state
            .select_window("1")
            .expect("bare numeric window target should select window 1");
        assert_eq!(
            state.sessions()[0].windows[state.sessions()[0].active].index,
            1
        );

        // Real tmux: an empty session part means the current session.
        state
            .select_window(":0")
            .expect("empty-session window target should select window 0");
        assert_eq!(
            state.sessions()[0].windows[state.sessions()[0].active].index,
            0
        );

        let error = state
            .select_window("missing")
            .expect_err("missing bare target should fail");
        assert_eq!(error.to_string(), "can't find window: missing");
    }

    impl ServerState {
        fn summary_contains(&self, needle: &str) -> bool {
            self.sessions.iter().any(|s| s.summary().contains(needle))
        }
    }
}
