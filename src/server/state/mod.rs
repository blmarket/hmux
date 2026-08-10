//! The server's session/window/pane model.
//!
//! A deliberately small mirror of tmux's `session`/`winlink`/`window`/
//! `window_pane` tree — enough to back the commands the prototype implements
//! (`list-sessions`, `new-session`, `has-session`, `kill-session`) with real
//! state. Panes hold an emulator-backed [`Pane`], so a created session is a
//! genuinely running terminal, not a stub.
//!
//! This module owns the [`ServerState`] struct and the core model types
//! ([`Session`], [`Winlink`], [`Window`], [`PaneNode`]). Everything that
//! operates on them is split across sibling modules, each carrying its own
//! `impl ServerState` block: `sessions`, `windows`, `panes`, `buffers`,
//! `environ`, `options`, `keys`, `sizing`, `layout`, `modes`, `client`, and
//! the target parsing/lookup pair `target`/`resolve`. Private fields stay
//! private — Rust privacy reaches descendant modules, so the split needs no
//! accessors.

mod buffers;
mod client;
mod copy;
mod environ;
mod jobs;
mod keys;
mod layout;
mod mode;
mod modes;
mod options;
mod panes;
mod resolve;
mod sessions;
mod sizing;
mod target;
mod windows;

use client::ControlCheckpoint;
pub(crate) use client::{
    ActiveCommandPrompt, ClientFlagState, ClientPromptAttachment, ClientPromptRegistry,
    ClientRenderAttachment, ClientRenderRegistry, ClientSnapshot, CommandPromptRequestResult,
    ControlStateSnapshot, PromptCompletion, PromptReply, RenderInvalidation, TerminalReply,
    TerminalRequest, TerminalRequestKind, ViewportClient,
};
pub(crate) use copy::{
    copy_search_segments, copy_selection_segments, CopySelectionMode, CopyState,
};
pub(crate) use jobs::{BackgroundJobRegistry, WaitOutcome, WaitRegistry};
pub(crate) use layout::{LayoutCell, PaneRect, SplitDirection};
pub(crate) use mode::{
    CustomizeOption, ModeBindingUpdate, ModeEdit, ModeItem, ModeKind, ModePrompt, ModeView,
    ModeViewKeyResult,
};
pub(crate) use sizing::{
    pane_slider, WindowResizeAdjust, WindowResizeRequest, WindowSize, WindowSizePolicy,
};
pub(crate) use target::Target;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
#[cfg(test)]
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use super::key::KeyCode;
use super::options::{GlobalOptions, OptionSet, OptionsView};
use super::pane::Pane;
use hmux_vt::PaneScreen;

/// The server state, shared by everything running on the loop.
pub(crate) type SharedState = Rc<RefCell<ServerState>>;

/// Wrap a fresh [`ServerState`] in the handle everything on the loop shares.
pub(crate) fn shared_state(state: ServerState) -> SharedState {
    Rc::new(RefCell::new(state))
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
    /// A pane that already exists, with its child still running, moving into
    /// the layout — tmux's `job_transfer` out of a popup into a new pane. Its
    /// previous owner restores the loop registration before handing it over.
    Existing(Box<Pane>),
}

/// The session a pane's `TMUX` variable names.
///
/// `new-session` spawns its first pane before the session exists, so that
/// session's id can only be filled in at the spawn site — which is where tmux's
/// `spawn_pane` reads it, since tmux creates the session first.
pub(crate) enum SpawnSession<'a> {
    Existing(&'a str),
    Pending,
}

/// Stands in for an id [`ServerState::pane_environment`] cannot know yet.
///
/// hmux freezes a pane's environment into its argv (`env -i NAME=VALUE …`)
/// before the pane exists, so the pane id — and, for `new-session`, the session
/// id — are written as these placeholders and [`fill_spawn_ids`] replaces them
/// at the spawn, where tmux sets them directly.
const SPAWN_ID_MARK: char = '\u{1}';
const PANE_ID_PLACEHOLDER: &str = "\u{1}pane-id\u{1}";
const SESSION_ID_PLACEHOLDER: &str = "\u{1}session-id\u{1}";

/// `_PATH_DEFPATH`: the path tmux gives a pane whose environment supplies none.
const DEFAULT_PATH: &str = "/usr/bin:/bin";

/// Resolve the placeholders a pane's frozen environment carries.
fn fill_spawn_ids(argv: &[String], pane_id: u32, session_id: u32) -> Vec<String> {
    argv.iter()
        .map(|argument| {
            if !argument.contains(SPAWN_ID_MARK) {
                return argument.clone();
            }
            argument
                .replace(PANE_ID_PLACEHOLDER, &format!("%{pane_id}"))
                .replace(SESSION_ID_PLACEHOLDER, &session_id.to_string())
        })
        .collect()
}

/// Flatten a built environment into the `NAME=VALUE` entries a spawn takes.
fn environment_entries(environment: BTreeMap<String, String>) -> Vec<String> {
    environment
        .into_iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect()
}

/// [`fill_spawn_ids`] for a spec that has not been taken apart yet.
fn fill_spec_spawn_ids(spec: PaneSpec, pane_id: u32, session_id: u32) -> PaneSpec {
    match spec {
        PaneSpec::Command(argv) => PaneSpec::Command(fill_spawn_ids(&argv, pane_id, session_id)),
        PaneSpec::CommandIn(argv, cwd) => {
            PaneSpec::CommandIn(fill_spawn_ids(&argv, pane_id, session_id), cwd)
        }
        spec @ (PaneSpec::Inert | PaneSpec::Existing(_)) => spec,
    }
}

fn pane_start_command(spec: &PaneSpec) -> String {
    let argv = match spec {
        PaneSpec::Inert | PaneSpec::Existing(_) => return String::new(),
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
    /// Columns a visible scrollbar takes from this pane — tmux's
    /// `layout_fix_panes` subtracting `sb_w + sb_pad` when
    /// `window_pane_show_scrollbar` accepts the pane.
    pub(crate) scrollbar_columns: u16,
    /// Whether `pane-border-status` puts its row on this pane, and on which
    /// side — tmux's `layout_add_horizontal_border`, which only asks the pane
    /// at the very top (or bottom) of the window for the row, since every other
    /// pane already has a border there to write on.
    pub(crate) border_status: Option<PaneBorderStatus>,
    /// tmux's `PANE_UNSEENCHANGES`: output arrived while the pane was in a mode,
    /// so the grid the mode froze is behind the pane. Cleared when the mode
    /// goes, which is why the format reads it together with `mode`.
    pub(crate) unseen_changes: bool,
    /// tmux's `wp->active_point`: when this pane was last made the active one,
    /// on a server-wide counter. It is what `-O activity` orders panes by, and
    /// the only record of how recently a pane was looked at — a pane that has
    /// never been selected keeps the initial 0.
    pub(crate) active_point: u64,
    options: OptionSet,
}

/// Where a pane's scrollbar is drawn and how much of it the slider fills.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PaneScrollbar {
    pub(crate) columns: u16,
    pub(crate) on_left: bool,
    pub(crate) left: u16,
    pub(crate) top: u16,
    pub(crate) height: u16,
    pub(crate) slider_top: u16,
    pub(crate) slider_height: u16,
}

/// Which edge of a pane `pane-border-status` writes on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaneBorderStatus {
    Top,
    Bottom,
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
    /// The environment the popup's command runs with: `environ_for_session`
    /// with the `-e` overrides tmux's `cmd_display_popup_exec` puts on top.
    pub(crate) environment: Vec<String>,
    pub(crate) cwd: Option<PathBuf>,
    pub(crate) width: Option<String>,
    pub(crate) height: Option<String>,
    pub(crate) x: Option<String>,
    pub(crate) y: Option<String>,
    pub(crate) close_on_exit: bool,
    pub(crate) close_on_success: bool,
    pub(crate) close_on_key: bool,
    pub(crate) border: bool,
    /// A command to run once the popup closes, and a file to remove with it —
    /// how tmux's `popup_editor` reads an edited buffer back in.
    pub(crate) on_close: Vec<String>,
    pub(crate) on_close_remove: Option<PathBuf>,
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

#[derive(Clone, Debug)]
pub(crate) enum ClientAction {
    Lock(String),
    Suspend,
    /// `detach-client`; `Some` is `-E`, the command the client execs instead
    /// of simply detaching (tmux's `server_client_exec`).
    Detach(Option<String>),
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
        reply: Option<PromptReply>,
    },
    Confirm {
        prompt: String,
        command: Vec<String>,
        confirm_key: u8,
        default_yes: bool,
        reply: Option<PromptReply>,
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
    /// tmux's `w->activity_time`, which `#{window_activity}` reports. Written
    /// once, by `window_create`, so it is the window's creation time.
    pub(crate) activity_epoch: i64,
    /// tmux's `w->name_time`: when `automatic-rename` last re-derived this
    /// window's name. It rate-limits the re-derivation to one per
    /// `NAME_INTERVAL`.
    pub(crate) name_time_micros: i64,
    /// Whether the active pane was in a mode when the name was last derived.
    /// tmux flags the pane changed as it enters and leaves a mode, since
    /// `automatic-rename-format` reads `#{pane_in_mode}`; hmux compares the
    /// value instead, so every path in and out of a mode is covered.
    pub(crate) name_in_mode: bool,
    /// Which side `pane-scrollbars-position` puts a scrollbar on, cached
    /// alongside the per-pane reservation it applies to.
    pub(crate) scrollbars_on_left: bool,
    /// The window's own size — tmux's `w->sx`/`w->sy`, published as
    /// `#{window_width}`/`#{window_height}`. A window has one size no matter how
    /// many sessions link it; [`ServerState::recalculate_sizes`] derives it from
    /// the clients that can see it under `window-size`.
    pub cols: u16,
    pub rows: u16,
    /// The size `window-size manual` pins the window at — tmux's
    /// `w->manual_sx`/`w->manual_sy`. Always set (a never-resized window carries
    /// the size it was created at) and moved only by `resize-window`.
    pub manual_size: (u16, u16),
    /// The client that most recently sized this window, as the sequence number
    /// [`AttachedClient::size_seq`] carries — tmux's `w->latest`, which is what
    /// `window-size latest` follows once more than one client can see the
    /// window.
    pub(crate) latest_client: Option<u64>,
    /// A size derived for this window while no attached session was showing it —
    /// tmux's `w->new_sx`/`w->new_sy` behind the `WINDOW_RESIZE` flag. It is
    /// applied by `server_client_check_window_resize` once some session makes
    /// the window current, so a background window keeps the geometry its panes
    /// were last drawn at instead of churning on every client resize.
    pub(crate) pending_size: Option<WindowSize>,
    /// The window's cell size in pixels — tmux's `w->xpixel`/`w->ypixel`,
    /// aggregated from the attached clients' terminals. Only sixel parsing
    /// reads it, to turn an image's pixel dimensions into cells.
    pub(crate) xpixel: u16,
    pub(crate) ypixel: u16,
    pub(crate) layout: LayoutCell,
    pub(crate) last_layout: Option<usize>,
    /// The layout string before the last `select-layout`-family command —
    /// tmux's `w->old_layout`, which `select-layout -o` restores.
    pub(crate) old_layout: Option<String>,
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
    pub active: usize, // Vec position of the active window
    /// MRU winlink identities excluding the current window, matching tmux's
    /// full `lastw` stack. Its head is the previously-active window.
    pub(crate) last_windows: Vec<u64>,
    pub cols: u16,
    pub rows: u16,
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
    /// The session's working directory — tmux's `s->cwd`, the `-c` value of the
    /// `new-session` that created it (else the creating client's directory).
    /// It is what `#()` jobs expanded by an attached client run in.
    cwd: Option<PathBuf>,
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

/// The key table a client is in until a session's `key-table` option or the
/// prefix key moves it elsewhere — tmux's `server_client_get_key_table`.
pub(crate) const DEFAULT_KEY_TABLE: &str = "root";

impl Session {
    pub(crate) fn cwd(&self) -> Option<&Path> {
        self.cwd.as_deref()
    }

    /// Vec position of the previously-active window — tmux's `s->last`, the
    /// top of the `lastw` stack that is still linked here. `None` once every
    /// window the session came from has been unlinked.
    pub fn last_active(&self) -> Option<usize> {
        self.last_windows
            .iter()
            .find_map(|id| self.windows.iter().position(|link| link.link_id == *id))
    }

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
            created_stamp(self.created_epoch),
        )
    }
}

/// tmux's `next_active_point`, a counter shared by every window on the server:
/// each pane that becomes active takes the next value, so the numbers order the
/// panes by how recently they were active.
fn next_active_point() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

impl Window {
    pub(crate) fn options<'a>(&'a self, globals: &'a GlobalOptions) -> OptionsView<'a> {
        OptionsView::two(&self.options, globals.window())
    }

    /// tmux's `window_set_active_pane`: make `index` the active pane and stamp
    /// it with the moment it became active. A pane that was already active is
    /// left alone, as tmux's early return leaves it — re-selecting the pane you
    /// are in does not move it up the activity order.
    ///
    /// Paths that merely re-point `active` at the same pane after the indexes
    /// around it shifted assign the field directly instead; nothing became
    /// active there.
    pub(crate) fn set_active_pane(&mut self, index: usize) {
        if self.active == index {
            return;
        }
        self.active = index;
        if let Some(node) = self.panes.get_mut(index) {
            node.active_point = next_active_point();
            // tmux flags the pane it just made active as changed, so the window
            // takes its name from whatever is running in the new pane.
            node.pane.observation_state().note_changed();
        }
    }

    pub(crate) fn option_overrides(&self) -> &OptionSet {
        &self.options
    }

    pub(crate) fn option_overrides_mut(&mut self) -> &mut OptionSet {
        &mut self.options
    }

    /// The pane `last-pane` and `select-pane -l` act on: the recorded
    /// previously-active pane, or — as tmux's `cmd-select-pane.c` falls back —
    /// the only other pane when the window holds exactly two.
    pub(crate) fn last_pane_index(&self) -> Option<usize> {
        self.last_pane
            .filter(|&pane| pane < self.panes.len())
            .or_else(|| (self.panes.len() == 2).then_some(1 - self.active))
    }

    /// The window's pane indices front to back, tmux's `w->z_index` order:
    /// floating panes sit above the layout (the active one frontmost, then the
    /// most recently created), and the tiled panes keep their layout order
    /// underneath. Mouse hit-testing walks it front to back; `#{pane_z}` counts
    /// how many floating panes a pane sits under.
    pub(crate) fn z_order(&self) -> Vec<usize> {
        let mut order = (0..self.panes.len()).collect::<Vec<_>>();
        order.sort_by_key(|&index| {
            (
                self.panes[index].floating.is_some(),
                index == self.active,
                index,
            )
        });
        order.reverse();
        order
    }

    /// `#{pane_z}`: how far down tmux's z-index list the pane sits, counted as
    /// `window_pane_zindex` does. A floating pane reports its own rank in the
    /// floating stack (0 is frontmost); a tiled pane sits under all of them and
    /// reports one level below, so the plain no-floating window reports 1.
    pub(crate) fn pane_z_index(&self, index: usize) -> usize {
        let mut depth = 0;
        for candidate in self.z_order() {
            if candidate == index {
                if self.panes[index].floating.is_none() {
                    depth += 1;
                }
                break;
            }
            if self.panes[candidate].floating.is_some() {
                depth += 1;
            }
        }
        depth
    }

    /// `#{pane_flags}`: tmux's `window_pane_printable_flags`, the same marker
    /// letters `#{window_flags}` uses for a window — active, last-active,
    /// zoomed, floating, in that order.
    pub(crate) fn printable_pane_flags(&self, index: usize) -> String {
        let Some(pane) = self.panes.get(index) else {
            return String::new();
        };
        let mut flags = String::new();
        if index == self.active {
            flags.push('*');
        }
        if Some(index) == self.last_pane {
            flags.push('-');
        }
        // Zooming makes a pane active, so the window's zoom belongs to it.
        if self.zoomed && index == self.active {
            flags.push('Z');
        }
        if pane.floating.is_some() {
            flags.push('F');
        }
        flags
    }

    pub(crate) fn pane_rect(&self, pane_id: u32) -> Option<PaneRect> {
        let node = self.panes.iter().find(|pane| pane.id == pane_id)?;
        let mut rect = node.floating.or_else(|| self.layout.pane_rect(pane_id))?;
        // A visible scrollbar is drawn beside the pane, not over it, so the
        // pane is that many columns narrower — and starts that many columns
        // further right when the bar is on the left.
        if node.scrollbar_columns != 0 && rect.width > node.scrollbar_columns {
            rect.width -= node.scrollbar_columns;
            if self.scrollbars_on_left {
                rect.left += node.scrollbar_columns;
            }
        }
        // The border-status row comes out of the pane the same way.
        if let Some(side) = node.border_status.filter(|_| rect.height > 1) {
            rect.height -= 1;
            if side == PaneBorderStatus::Top {
                rect.top += 1;
            }
        }
        Some(rect)
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

/// Format `epoch` as tmux's `(created ...)` timestamp (e.g.
/// `Tue Jul  7 20:57:17 2026`). Best-effort; an empty string on failure (the
/// value is normalized away in conformance comparisons regardless).
pub fn created_stamp(epoch: i64) -> String {
    // SAFETY: standard libc localtime/strftime dance. `localtime` returns a
    // pointer into a static buffer, used only until the next call; we copy out
    // immediately via strftime.
    unsafe {
        let t = epoch as libc::time_t;
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
/// tmux's `screen_set_cursor_style` numbering, which DECSCUSR takes.
pub(crate) fn cursor_style_parameter(style: &str) -> u8 {
    match style {
        "blinking-block" => 1,
        "block" => 2,
        "blinking-underline" => 3,
        "underline" => 4,
        "blinking-bar" => 5,
        "bar" => 6,
        _ => 0,
    }
}

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
    // `tv_sec` is `i64` everywhere we support, but `tv_usec` is `i32` on
    // Darwin and `i64` on Linux; `i64::from` covers both widths.
    tv.tv_sec * 1_000_000 + i64::from(tv.tv_usec)
}

/// The whole server's state. Guarded by a mutex at the connection layer.
pub struct ServerState {
    /// True only before this explicitly launched hmux server has created its
    /// first session. An untargeted attach may consume this state by creating
    /// session 0; becoming empty later must not repeat that bootstrap behavior.
    initial_attach_pending: bool,
    /// When this server state was created, as `#{start_time}` reports it.
    started_epoch: i64,
    /// The pathname this server listens on, as `#{socket_path}` reports it and
    /// as `TMUX` names it in a spawned process. Empty in the unit tests and in
    /// any embedding that never binds a socket.
    socket_path: PathBuf,
    /// Pipe jobs that belong to no pane, waiting for the loop to adopt them.
    new_pipes: Vec<super::pane::PanePipeIo>,
    sessions: Vec<Session>,
    /// Windows are owned once by the server and referenced through [`Winlink`].
    windows: BTreeMap<u32, Window>,
    /// Explicit tmux session groups, keyed by their synchronization identity.
    /// Group names outlive the session they were originally named after, and a
    /// one-member group remains grouped until its final session is destroyed.
    session_groups: BTreeMap<u32, String>,
    /// Set once an empty server is one `exit-empty` asks to shut down.
    shutdown_requested: bool,
    /// Client-registry generation the unattached sweep last ran against.
    lifecycle_generation: u64,
    /// Sessions a client just took, whose windows the next alert pass
    /// re-examines in full (tmux's `alerts_check_session`).
    alert_check_sessions: BTreeSet<u32>,
    /// The theme last announced to each pane subscribed with DECSET 2031.
    pane_theme_pushed: BTreeMap<u32, String>,
    /// Questions put to an attached terminal on a pane's behalf and still
    /// waiting for it to answer — tmux's `input_request` list.
    terminal_requests: Vec<TerminalRequest>,
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
    /// Which global names came from the daemon's own environment rather than a
    /// `set-environment`. They are the base a spawn starts from, so the
    /// requesting client's environment overrides them; an explicit assignment
    /// overrides the client in turn.
    seeded_environment: BTreeSet<String>,
    /// Bumped whenever anything a spawned process's environment is built from
    /// changes, so [`ServerState::job_environment`] can cache its answer.
    environment_generation: u64,
    /// The last answer [`ServerState::job_environment`] gave, with the session
    /// and generation it was built for. A command builds its job runner whether
    /// or not it expands a `#()`, and rebuilding the whole environment each
    /// time is not free.
    job_environment_cache: RefCell<Option<(u64, Option<u32>, Rc<Vec<String>>)>>,
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
    /// The `history-file` the prompt history was last loaded from, so it is
    /// read once rather than on every option change.
    prompt_history_file_loaded: Option<PathBuf>,
    message_log: Vec<MessageLogEntry>,
    background_jobs: Rc<BackgroundJobRegistry>,
    running_hooks: BTreeSet<String>,
    /// The `hook*` format variables published to the hook body currently
    /// executing; empty outside hook bodies. Like `command_session_id`, a
    /// transient interpreter hint installed around each hook command.
    hook_format_vars: Vec<(String, String)>,
    /// Stable session selected for the command currently executing. This is a
    /// transient interpreter hint, set while a client-scoped prompt template
    /// runs and restored before releasing the server-state lock.
    command_session_id: Option<u32>,
    /// Mouse event of the command currently executing (tmux's `format_tree.m`),
    /// installed alongside the target hints above.
    command_mouse: Option<super::mouse::MouseEvent>,
    /// `#()` job runner of the command currently executing, so any format it
    /// expands caches its jobs in the tree of the client that ran it.
    command_format_jobs: Option<Rc<super::command::CommandJobs>>,
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
    client_prompts: Rc<ClientPromptRegistry>,
    client_renders: Rc<ClientRenderRegistry>,
    wait_registry: Rc<WaitRegistry>,
    /// No-client format jobs, corresponding to tmux's process-global job tree.
    /// A format expanded with no client caches its `#()` jobs here; per-client
    /// trees live in `client_renders`.
    format_jobs: Rc<super::status::FormatJobRegistry>,
}

/// When an empty server shuts itself down.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ExitEmpty {
    /// `off`: an empty server keeps running.
    Off,
    /// `on`: tmux's setting — an empty server exits, including one that has
    /// never held a session, which for a daemon means at startup.
    On,
    /// `after-session`: an empty server exits once it has held a session.
    AfterSession,
}

impl ServerState {
    /// Build an empty server. Under the default `exit-empty=after-session` it
    /// remains available for the first client, and the policy starts applying
    /// once a session has existed.
    pub fn empty() -> ServerState {
        let client_renders = Rc::new(ClientRenderRegistry::new());
        let mut state = ServerState {
            initial_attach_pending: true,
            started_epoch: now_epoch(),
            socket_path: PathBuf::new(),
            new_pipes: Vec::new(),
            sessions: Vec::new(),
            windows: BTreeMap::new(),
            session_groups: BTreeMap::new(),
            shutdown_requested: false,
            lifecycle_generation: 0,
            alert_check_sessions: BTreeSet::new(),
            focused_panes: BTreeSet::new(),
            pane_theme_pushed: BTreeMap::new(),
            terminal_requests: Vec::new(),
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
            seeded_environment: BTreeSet::new(),
            environment_generation: 0,
            job_environment_cache: RefCell::new(None),
            global_options: GlobalOptions::new(),
            buffers: Vec::new(),
            buffer_created: BTreeMap::new(),
            automatic_buffers: BTreeSet::new(),
            next_buffer_id: 0,
            marked_pane_id: None,
            key_tables: BTreeMap::new(),
            pending_config_errors: Vec::new(),
            prompt_history: BTreeMap::new(),
            prompt_history_file_loaded: None,
            message_log: Vec::new(),
            background_jobs: Rc::new(BackgroundJobRegistry::default()),
            running_hooks: BTreeSet::new(),
            hook_format_vars: Vec::new(),
            command_session_id: None,
            command_mouse: None,
            command_format_jobs: None,
            command_window_id: None,
            command_active_panes: None,
            control_checkpoints: VecDeque::new(),
            next_control_checkpoint: 0,
            pane_alert_seen: BTreeMap::new(),
            window_last_activity: BTreeMap::new(),
            silence_alerted: BTreeSet::new(),
            client_prompts: Rc::new(ClientPromptRegistry::new()),
            format_jobs: Rc::new(super::status::FormatJobRegistry::new(&client_renders)),
            client_renders,
            wait_registry: Rc::new(WaitRegistry::default()),
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

    /// Register a sizing client on `session_name`, as the attach loop does, and
    /// fold it into the window sizes.
    ///
    /// Window sizes come from the clients that can see the window, so a test
    /// that wants panes at a given geometry has to supply a client rather than
    /// set a size directly. `rows` is the client's terminal height with its
    /// status line included, so the resulting window is `rows` minus the
    /// session's status lines tall. The returned attachment must stay alive:
    /// dropping it detaches the client.
    #[cfg(test)]
    pub(crate) fn attach_test_client(
        &mut self,
        session_name: &str,
        cols: u16,
        rows: u16,
    ) -> io::Result<ClientRenderAttachment> {
        let session_id = self.session_id(session_name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {session_name}"),
            )
        })?;
        let attachment = Rc::clone(&self.client_renders)
            .attach(session_id, format!("test-client-{session_id}"))?;
        attachment.update_size(cols, rows);
        self.recalculate_sizes()?;
        Ok(attachment)
    }

    pub(crate) fn initial_attach_pending(&self) -> bool {
        self.initial_attach_pending
    }

    pub fn sessions(&self) -> &[Session] {
        &self.sessions
    }

    /// The id the next session created will get, as `#{next_session_id}`
    /// reports it.
    pub(crate) fn next_session_id(&self) -> u32 {
        self.next_session_id
    }

    pub(crate) fn session_mut(&mut self, session: usize) -> &mut Session {
        &mut self.sessions[session]
    }

    fn links_mut(&mut self, session: usize) -> &mut Vec<Winlink> {
        &mut self.sessions[session].windows
    }

    pub(crate) fn wait_registry(&self) -> Rc<WaitRegistry> {
        Rc::clone(&self.wait_registry)
    }

    pub(crate) fn format_job_registry(&self) -> Rc<super::status::FormatJobRegistry> {
        Rc::clone(&self.format_jobs)
    }

    /// Every `#()` job launched since the last call, across the per-client
    /// trees and the clientless one, for the loop to drive.
    pub(crate) fn take_pending_format_jobs(&self) -> Vec<super::status::FormatJob> {
        let mut pending = self.format_jobs.take_pending();
        for jobs in self.client_renders.all_format_jobs() {
            pending.extend(jobs.take_pending());
        }
        pending
    }

    pub(crate) fn background_job_registry(&self) -> Rc<BackgroundJobRegistry> {
        Rc::clone(&self.background_jobs)
    }

    /// The `#()` job tree a format expanded by `client` uses, with the session
    /// that client is on. tmux caches per client and falls back to one
    /// process-global tree for a format with no client — a command client that
    /// is not attached is not one of ours, so it lands in the global tree.
    pub(crate) fn format_jobs_for_client(
        &self,
        client: Option<&str>,
    ) -> (Rc<super::status::FormatJobRegistry>, Option<u32>) {
        match client.and_then(|name| self.client_renders.client_format_jobs(name)) {
            Some((jobs, session_id)) => (jobs, Some(session_id)),
            // A command client has a job tree of its own that dies with it, so
            // a job it starts is never picked up by the next command — tmux
            // keeps `#()` output in `c->jobs`, and that client is already gone.
            // Only a format with no client at all reaches the server's tree.
            None if client.is_some() => (
                Rc::new(super::status::FormatJobRegistry::new(&self.client_renders)),
                None,
            ),
            None => (self.format_job_registry(), None),
        }
    }

    /// Every `#()` job still running, across the per-client trees and the
    /// clientless one — what `show-messages -J` lists.
    pub(crate) fn running_format_jobs(&self) -> Vec<super::status::FormatJobInfo> {
        let mut running = self.format_jobs.running();
        for jobs in self.client_renders.all_format_jobs() {
            running.extend(jobs.running());
        }
        running.sort_by_key(|job| job.pid);
        running
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

    /// Install the `#()` job runner of the command currently running. Like the
    /// mouse event below it is installed once around a command, so every format
    /// the command expands starts its jobs in the right client's tree.
    pub(crate) fn replace_command_format_jobs(
        &mut self,
        jobs: Option<Rc<super::command::CommandJobs>>,
    ) -> Option<Rc<super::command::CommandJobs>> {
        std::mem::replace(&mut self.command_format_jobs, jobs)
    }

    pub(crate) fn command_format_jobs(&self) -> Option<&Rc<super::command::CommandJobs>> {
        self.command_format_jobs.as_ref()
    }

    /// Install the mouse event of the command currently running, tmux's
    /// `format_tree.m`. Every `#{mouse_*}` expansion and every `-t =` target
    /// resolves against it, so it is installed once around a command rather
    /// than threaded through each consumer.
    pub(crate) fn replace_command_mouse(
        &mut self,
        mouse: Option<super::mouse::MouseEvent>,
    ) -> Option<super::mouse::MouseEvent> {
        std::mem::replace(&mut self.command_mouse, mouse)
    }

    pub(crate) fn command_mouse(&self) -> Option<&super::mouse::MouseEvent> {
        self.command_mouse.as_ref()
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

    fn invalidate_session(&self, session_id: u32, reason: RenderInvalidation) {
        self.client_renders.publish_session(session_id, reason);
    }

    fn invalidate_all_clients(&self, reason: RenderInvalidation) {
        self.client_renders.publish_all(reason);
    }

    /// Whether the outer listener should stop accepting clients.
    pub fn shutdown_requested(&self) -> bool {
        self.shutdown_requested
    }

    /// The `exit-empty` setting, including hmux's `after-session` extension.
    fn exit_empty_policy(&self) -> ExitEmpty {
        match self.server_options().get("exit-empty") {
            Some("off" | "no" | "false" | "0") => ExitEmpty::Off,
            Some(super::options::EXIT_EMPTY_AFTER_SESSION) => ExitEmpty::AfterSession,
            Some(_) | None => ExitEmpty::On,
        }
    }

    /// Apply tmux's `exit-empty` policy after an explicit tree mutation.
    fn request_shutdown_if_became_empty(&mut self, had_sessions: bool) {
        // A server that just lost its last session has held one, so
        // `after-session` and `on` agree here.
        if had_sessions && self.sessions.is_empty() && self.exit_empty_policy() != ExitEmpty::Off {
            self.shutdown_requested = true;
        }
    }

    /// Record the pathname the server is listening on.
    pub fn set_socket_path(&mut self, path: impl Into<PathBuf>) {
        self.environment_generation += 1;
        self.socket_path = path.into();
    }

    /// The pathname the server is listening on (`#{socket_path}`).
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// The effective `base-index` (first/lowest window index), default 0.
    fn base_index(&self) -> u32 {
        self.global_options
            .session()
            .get("base-index")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
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
}

#[cfg(test)]
mod tests {
    use super::copy::*;
    use super::*;
    use crate::event_loop::test_driver::run_on_loop;
    use hmux_vt::{CellSemantic, CellWidth, Grid, GridCell, GridRow, RowFlags};

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
            .map(|ch| GridCell {
                text: ch.to_string(),
                width: CellWidth::Narrow,
                semantic: CellSemantic::Output,
                hyperlink: None,
                hyperlink_slot: 0,
                tab: false,
            })
            .collect::<Vec<_>>();
        cells.resize(
            20,
            GridCell {
                text: String::new(),
                width: CellWidth::Narrow,
                semantic: CellSemantic::Output,
                hyperlink: None,
                hyperlink_slot: 0,
                tab: false,
            },
        );
        let size = cells.len();
        let grid = Grid {
            cols: 20,
            viewport_rows: 1,
            scrollback_rows: 0,
            rows: vec![GridRow {
                cells,
                wrapped: false,
                used: text.chars().count(),
                size,
                extd: 0,
                flags: RowFlags::default(),
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
            search_marks: true,
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
        let registry = Rc::new(ClientPromptRegistry::new());
        let client_a = registry
            .attach("/dev/pts/7".to_string(), Some(700), 7)
            .expect("attach client A");
        let client_b = registry
            .attach("/dev/pts/8".to_string(), Some(800), 8)
            .expect("attach client B");

        let request = registry.request_command(
            Some("/dev/pts/7"),
            None,
            vec!["command-prompt".to_string()],
            true,
        );
        let CommandPromptRequestResult::Waiting(answer) = request else {
            panic!("waiting request");
        };
        assert!(client_b.take_command_prompt().is_none());
        let prompt = client_a.take_command_prompt().expect("client A prompt");
        assert_eq!(prompt.args(), &["command-prompt".to_string()]);
        prompt.complete(PromptCompletion {
            stdout: "Up".to_string(),
            stderr: String::new(),
            exit: 0,
            inserted: true,
        });

        let answered = run_on_loop(answer).expect("answer").expect("completion");
        assert_eq!(answered.stdout, "Up");
        assert_eq!(answered.exit, 0);
    }

    #[test]
    fn detaching_client_cancels_its_queued_prompt() {
        let registry = Rc::new(ClientPromptRegistry::new());
        let client = registry
            .attach("/dev/pts/7".to_string(), Some(700), 7)
            .expect("attach client");
        let request = registry.request_command(
            Some("/dev/pts/7"),
            None,
            vec!["command-prompt".to_string()],
            true,
        );
        let CommandPromptRequestResult::Waiting(answer) = request else {
            panic!("waiting request");
        };
        drop(client);

        // A detached client answers nothing; the queue continues on its own.
        assert!(run_on_loop(answer).expect("answer").is_none());
    }

    #[test]
    fn default_state_has_one_session() {
        let state = ServerState::with_test_session().expect("state");
        assert_eq!(state.sessions().len(), 1);
        assert_eq!(state.sessions()[0].name, "0");
        assert!(state.summary_contains("0: 1 windows"));
    }

    #[test]
    fn selecting_window_applies_its_deferred_client_size() {
        let mut state = ServerState::with_test_session().expect("state");
        state
            .new_window("0", None, false)
            .expect("create inactive window");
        // 20x5 of terminal, one row of which the status line takes.
        let _client = state
            .attach_test_client("0", 20, 5)
            .expect("attach sizing client");

        assert_eq!(state.window(0, 0).panes[0].pane.size(), (20, 4));
        assert_eq!(
            state.window(0, 1).panes[0].pane.size(),
            (80, 24),
            "a window no session is showing defers the size instead of resizing"
        );

        state.select_window("0:1").expect("select inactive window");
        assert_eq!(
            state.window(0, 1).panes[0].pane.size(),
            (20, 4),
            "the deferred size lands once the window becomes current"
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
    fn exit_empty_after_session_spares_a_server_that_never_held_one() {
        // The default: a daemon starts empty and waits for its first client,
        // then becomes ordinary `exit-empty` once a session has existed.
        let mut state = ServerState::empty();
        state.enforce_exit_options();
        assert!(!state.shutdown_requested());

        state.create_session("0", PaneSpec::Inert).expect("create");
        assert!(state.kill_session("0"));
        assert!(state.shutdown_requested());
    }

    #[test]
    fn exit_empty_on_shuts_down_a_server_that_never_held_a_session() {
        // tmux's own setting, applied to a server tmux would never have
        // started: with no session to hold it, it exits where it stands.
        let mut state = ServerState::empty();
        state
            .global_options_mut()
            .for_scope_mut(super::super::options::OptionScope::Server)
            .set("exit-empty", "on");
        state.enforce_exit_options();
        assert!(state.shutdown_requested());
    }

    #[test]
    fn exit_empty_off_spares_a_server_that_lost_its_last_session() {
        let mut state = ServerState::with_test_session().expect("state");
        state
            .global_options_mut()
            .for_scope_mut(super::super::options::OptionScope::Server)
            .set("exit-empty", "off");
        assert!(state.kill_session("0"));
        assert!(!state.shutdown_requested());
    }

    #[test]
    fn duplicate_session_rejected() {
        let mut state = ServerState::with_test_session().expect("state");
        let err = state.create_session("0", PaneSpec::Inert).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
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
