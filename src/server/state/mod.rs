//! The server's session/window/pane model.
//!
//! A deliberately small mirror of tmux's `session`/`winlink`/`window`/
//! `window_pane` tree — enough to back the commands the prototype implements
//! (`list-sessions`, `new-session`, `has-session`, `kill-session`) with real
//! state. Panes hold a libghostty-backed [`Pane`], so a created session is a
//! genuinely running terminal, not a stub.

mod buffers;
mod client;
mod copy;
mod environ;
mod jobs;
mod layout;
mod mode;
mod panes;
mod resolve;
mod sessions;
mod sizing;
mod target;
mod windows;

pub(crate) use client::{
    ActiveCommandPrompt, ClientFlagState, ClientPromptAttachment, ClientPromptRegistry,
    ClientRenderAttachment, ClientRenderRegistry, ClientSnapshot, CommandPromptRequestResult,
    ControlGlobalWindowSnapshot, ControlPaneSnapshot, ControlStateSnapshot, ControlWindowSnapshot,
    PromptCompletion, PromptReply, RenderInvalidation, TerminalReply, TerminalRequest,
    TerminalRequestKind, ViewportClient,
};
use client::{ControlCheckpoint, CONTROL_CHECKPOINT_LIMIT};
// Copy mode is used pervasively by the pane methods still in this module, so
// its internals come in wholesale rather than as a fifty-name import list.
use copy::*;
pub(crate) use copy::{
    copy_search_segments, copy_selection_segments, CopySelectionMode, CopyState,
};
pub(crate) use jobs::{BackgroundJobRegistry, WaitOutcome, WaitRegistry};
pub(crate) use layout::{checksummed_layout_dump, LayoutCell, PaneRect, SplitDirection};
use layout::{parse_custom_layout, resize_panes_to_layout};
use mode::update_mode_edit_item;
pub(crate) use mode::{
    CustomizeOption, ModeBindingUpdate, ModeEdit, ModeItem, ModeKind, ModePrompt, ModeView,
    ModeViewKeyResult,
};
pub(crate) use sizing::{pane_slider, WindowResizeAdjust, WindowResizeRequest, WindowSizePolicy};
use target::pane_not_found;
pub(crate) use target::Target;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use super::key::{parse_key_name, KeyCode};
use super::options::{GlobalOptions, OptionSet, OptionsView};
use super::pane::Pane;
use super::term::ResolvedTerm;
use crate::vt::screen::VtScreen;
use crate::vt::width;
use crate::vt::PaneScreen;

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
    pub(crate) pending_size: Option<(u16, u16)>,
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
    /// Current native consumers use client-owned status caches; this remains a
    /// distinct owner for no-client format contexts as those are implemented.
    #[allow(dead_code)]
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

    /// tmux 3.7b's `root` mouse table, in the same order and shape as
    /// `key_bindings_init`.
    ///
    /// The guards are the load-bearing part: a pane whose program enabled a
    /// DECSET mouse mode (`#{mouse_any_flag}`) or that is in a mode gets the
    /// report forwarded with `send-keys -M`, and only an inert pane lets the
    /// client act on the click itself.
    fn install_default_key_bindings(&mut self) {
        // tmux's `DEFAULT_PANE_MENU`/`DEFAULT_WINDOW_MENU`/`DEFAULT_SESSION_MENU`,
        // spliced into the bindings below. An operand whose name expands empty
        // is a separator line.
        const DEFAULT_PANE_MENU: &str = concat!(
            " '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Top,}' '<' 'send-keys -X history-top'",
            " '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Bottom,}' '>'",
            " 'send-keys -X history-bottom'",
            " ''",
            " '#{?#{&&:#{buffer_size},#{!:#{pane_in_mode}}},",
            "Paste #[underscore]#{=/9/...:buffer_sample},}' 'p' 'paste-buffer'",
            " ''",
            " '#{?mouse_word,Search For #[underscore]#{=/9/...:mouse_word},}' 'C-r'",
            " 'if-shell -F \"#{?#{m/r:(copy|view)-mode,#{pane_mode}},0,1}\" \"copy-mode -t =\" ;",
            " send-keys -X -t = search-backward -- \"#{q:mouse_word}\"'",
            " '#{?mouse_word,Type #[underscore]#{=/9/...:mouse_word},}' 'C-y'",
            " 'copy-mode -q ; send-keys -l -- \"#{q:mouse_word}\"'",
            " '#{?mouse_word,Copy #[underscore]#{=/9/...:mouse_word},}' 'c'",
            " 'copy-mode -q ; set-buffer -- \"#{q:mouse_word}\"'",
            " '#{?mouse_line,Copy Line,}' 'l' 'copy-mode -q ; set-buffer -- \"#{q:mouse_line}\"'",
            " ''",
            " '#{?mouse_hyperlink,Type #[underscore]#{=/9/...:mouse_hyperlink},}' 'C-h'",
            " 'copy-mode -q ; send-keys -l -- \"#{q:mouse_hyperlink}\"'",
            " '#{?mouse_hyperlink,Copy #[underscore]#{=/9/...:mouse_hyperlink},}' 'h'",
            " 'copy-mode -q ; set-buffer -- \"#{q:mouse_hyperlink}\"'",
            " ''",
            " '#{?#{!:#{pane_floating_flag}},Horizontal Split,}' 'h' 'split-window -h'",
            " '#{?#{!:#{pane_floating_flag}},Vertical Split,}' 'v' 'split-window -v'",
            " ''",
            " '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Up,}' 'u'",
            " 'swap-pane -U'",
            " '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Down,}' 'd'",
            " 'swap-pane -D'",
            " '#{?pane_marked_set,,-}Swap Marked' 's' 'swap-pane'",
            " ''",
            " 'Kill' 'X' 'kill-pane'",
            " 'Respawn' 'R' 'respawn-pane -k'",
            " '#{?pane_marked,Unmark,Mark}' 'm' 'select-pane -m'",
            " '#{?#{>:#{window_panes},1},,-}#{?window_zoomed_flag,Unzoom,Zoom}' 'z'",
            " 'resize-pane -Z'",
        );
        const DEFAULT_WINDOW_MENU: &str = concat!(
            " '#{?#{>:#{session_windows},1},,-}Swap Left' 'l' 'swap-window -t :-1'",
            " '#{?#{>:#{session_windows},1},,-}Swap Right' 'r' 'swap-window -t :+1'",
            " '#{?pane_marked_set,,-}Swap Marked' 's' 'swap-window'",
            " ''",
            " 'Kill' 'X' 'kill-window'",
            " 'Respawn' 'R' 'respawn-window -k'",
            " '#{?pane_marked,Unmark,Mark}' 'm' 'select-pane -m'",
            " 'Rename' 'n'",
            " 'command-prompt -F -I \"#W\" \"rename-window -t #{window_id} -- %%\"'",
            " ''",
            " 'New After' 'w' 'new-window -a'",
            " 'New At End' 'W' 'new-window'",
        );
        const DEFAULT_SESSION_MENU: &str = concat!(
            " 'Next' 'n' 'switch-client -n'",
            " 'Previous' 'p' 'switch-client -p'",
            " ''",
            " 'Renumber' 'N' 'move-window -r'",
            " 'Rename' 'r' 'command-prompt -I \"#S\" \"rename-session -- %%\"'",
            " 'Detach' 'd' 'detach-client'",
            " ''",
            " 'New Session' 's' 'new-session'",
            " 'New Window' 'w' 'new-window'",
        );
        const ROOT_MOUSE_DEFAULTS: &[(&str, &str)] = &[
            ("MouseDown1Pane", "select-pane -t = ; send-keys -M"),
            ("C-MouseDown1Pane", "swap-pane -s ="),
            (
                "MouseDrag1Pane",
                "if-shell -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' \
                 'send-keys -M' 'copy-mode -M'",
            ),
            (
                "WheelUpPane",
                "if-shell -F '#{||:#{alternate_on},#{||:#{pane_in_mode},#{mouse_any_flag}}}' \
                 'send-keys -M' 'copy-mode -e'",
            ),
            (
                "MouseDown2Pane",
                "select-pane -t = ; if-shell -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' \
                 'send-keys -M' 'paste-buffer -p'",
            ),
            (
                "DoubleClick1Pane",
                "select-pane -t = ; if-shell -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' \
                 'send-keys -M' \
                 'copy-mode -H ; send-keys -X select-word ; run-shell -d 0.3 ; \
                 send-keys -X copy-pipe-and-cancel'",
            ),
            (
                "TripleClick1Pane",
                "select-pane -t = ; if-shell -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' \
                 'send-keys -M' \
                 'copy-mode -H ; send-keys -X select-line ; run-shell -d 0.3 ; \
                 send-keys -X copy-pipe-and-cancel'",
            ),
            ("MouseDown1Border", "select-pane -M"),
            ("MouseDrag1Border", "resize-pane -M"),
            ("MouseDown1Status", "switch-client -t ="),
            ("C-MouseDown1Status", "swap-window -t ="),
            ("WheelUpStatus", "previous-window"),
            ("WheelDownStatus", "next-window"),
            ("MouseDown1Control8", "resize-pane -Z"),
            (
                "MouseDown1Control9",
                "display-menu -O -t = -x M -y M -T 'Kill pane #{pane_index}?' \
                 'Yes' 'y' 'kill-pane -t =' 'No' 'n' ''",
            ),
            (
                "MouseDown1ScrollbarUp",
                "if-shell -F -t = '#{pane_in_mode}' 'send-keys -X page-up' 'copy-mode -u'",
            ),
            (
                "MouseDown1ScrollbarDown",
                "if-shell -F -t = '#{pane_in_mode}' 'send-keys -X page-down' 'copy-mode -d'",
            ),
            (
                "MouseDrag1ScrollbarSlider",
                "if-shell -F -t = '#{pane_in_mode}' \
                 'send-keys -X scroll-to-mouse' 'copy-mode -S'",
            ),
            (
                "MouseDown3Pane",
                "if-shell -F -t = \
                 '#{||:#{mouse_any_flag},#{&&:#{pane_in_mode},\
                 #{?#{m/r:(copy|view)-mode,#{pane_mode}},0,1}}}' \
                 'select-pane -t = ; send-keys -M' \
                 'display-menu -t = -x M -y M \
                 -T \"#[align=centre]#{pane_index} (#{pane_id})\" {PANE_MENU}'",
            ),
            (
                "M-MouseDown3Pane",
                "display-menu -t = -x M -y M \
                 -T '#[align=centre]#{pane_index} (#{pane_id})' {PANE_MENU}",
            ),
            (
                "MouseDown3Status",
                "display-menu -t = -x W -y W \
                 -T '#[align=centre]#{window_index}:#{window_name}' {WINDOW_MENU}",
            ),
            (
                "M-MouseDown3Status",
                "display-menu -t = -x W -y W \
                 -T '#[align=centre]#{window_index}:#{window_name}' {WINDOW_MENU}",
            ),
            (
                "MouseDown3StatusLeft",
                "display-menu -t = -x M -y W -T '#[align=centre]#{session_name}' {SESSION_MENU}",
            ),
            (
                "M-MouseDown3StatusLeft",
                "display-menu -t = -x M -y W -T '#[align=centre]#{session_name}' {SESSION_MENU}",
            ),
        ];
        const DEFAULTS: &[(&str, &str, &[&str])] = &[
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
        for &(name, command) in ROOT_MOUSE_DEFAULTS {
            let key = parse_key_name(name)
                .unwrap_or_else(|| panic!("invalid default mouse key name: {name}"));
            let command = command
                .replace("{PANE_MENU}", DEFAULT_PANE_MENU)
                .replace("{WINDOW_MENU}", DEFAULT_WINDOW_MENU)
                .replace("{SESSION_MENU}", DEFAULT_SESSION_MENU);
            self.bind_key(
                DEFAULT_KEY_TABLE,
                key,
                super::command::binding_words(&command),
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

        // The prefix table is also a user-visible catalog: `list-keys -F`
        // exposes these notes and repeat flags even when a client never
        // exercises the corresponding binding. Keep the catalog complete so
        // listing it has the same shape as tmux's default table.
        const PREFIX_DEFAULTS: &[(&str, bool, &str, &str)] = &[
            ("Space", false, "Select next layout", "next-layout"),
            ("!", false, "Break pane to a new window", "break-pane"),
            ("\"", false, "Split window vertically", "split-window"),
            ("#", false, "List all paste buffers", "list-buffers"),
            ("$", false, "Rename current session", "rename-session"),
            ("%", false, "Split window horizontally", "split-window -h"),
            ("&", false, "Kill current window", "kill-window"),
            (
                "'",
                false,
                "Prompt for window index to select",
                "select-window",
            ),
            ("(", false, "Switch to previous client", "switch-client -p"),
            (")", false, "Switch to next client", "switch-client -n"),
            ("*", false, "New floating pane", "new-window"),
            (",", false, "Rename current window", "rename-window"),
            (
                "-",
                false,
                "Delete the most recent paste buffer",
                "delete-buffer",
            ),
            (".", false, "Move the current window", "move-window"),
            ("/", false, "Describe key binding", "list-keys"),
            ("0", false, "Select window 0", "select-window -t :=0"),
            ("1", false, "Select window 1", "select-window -t :=1"),
            ("2", false, "Select window 2", "select-window -t :=2"),
            ("3", false, "Select window 3", "select-window -t :=3"),
            ("4", false, "Select window 4", "select-window -t :=4"),
            ("5", false, "Select window 5", "select-window -t :=5"),
            ("6", false, "Select window 6", "select-window -t :=6"),
            ("7", false, "Select window 7", "select-window -t :=7"),
            ("8", false, "Select window 8", "select-window -t :=8"),
            ("9", false, "Select window 9", "select-window -t :=9"),
            (":", false, "Prompt for a command", "command-prompt"),
            (
                ";",
                false,
                "Move to the previously active pane",
                "last-pane",
            ),
            ("<", false, "Display window menu", "display-menu"),
            (
                "=",
                false,
                "Choose a paste buffer from a list",
                "choose-buffer -Z",
            ),
            (">", false, "Display pane menu", "display-menu"),
            ("?", false, "List key bindings", "list-keys -N"),
            ("C", false, "Customize options", "customize-mode -Z"),
            (
                "D",
                false,
                "Choose and detach a client from a list",
                "choose-client -Z",
            ),
            ("E", false, "Spread panes out evenly", "select-layout -E"),
            ("L", false, "Switch to the last client", "switch-client -l"),
            ("M", false, "Clear the marked pane", "select-pane -M"),
            ("[", false, "Enter copy mode", "copy-mode"),
            (
                "]",
                false,
                "Paste the most recent paste buffer",
                "paste-buffer -p",
            ),
            ("c", false, "Create a new window", "new-window"),
            ("d", false, "Detach the current client", "detach-client"),
            ("f", false, "Search for a pane", "find-window"),
            ("i", false, "Display window information", "display-message"),
            (
                "l",
                false,
                "Select the previously current window",
                "last-window",
            ),
            ("m", false, "Toggle the marked pane", "select-pane -m"),
            ("n", false, "Select the next window", "next-window"),
            ("o", false, "Select the next pane", "select-pane -t :.+"),
            ("p", false, "Select the previous window", "previous-window"),
            ("q", false, "Display pane numbers", "display-panes"),
            ("r", false, "Redraw the current client", "refresh-client"),
            (
                "s",
                false,
                "Choose a session from a list",
                "choose-tree -Zs",
            ),
            ("t", false, "Show a clock", "clock-mode"),
            ("w", false, "Choose a window from a list", "choose-tree -Zw"),
            ("x", false, "Kill the active pane", "kill-pane"),
            ("z", false, "Zoom the active pane", "resize-pane -Z"),
            (
                "{",
                false,
                "Swap the active pane with the pane above",
                "swap-pane -U",
            ),
            (
                "}",
                false,
                "Swap the active pane with the pane below",
                "swap-pane -D",
            ),
            ("~", false, "Show messages", "show-messages"),
            (
                "DC",
                true,
                "Reset so the visible part of the window follows the cursor",
                "refresh-client -c",
            ),
            (
                "PPage",
                false,
                "Enter copy mode and scroll up",
                "copy-mode -u",
            ),
            (
                "Up",
                true,
                "Select the pane above the active pane",
                "select-pane -U",
            ),
            (
                "Down",
                true,
                "Select the pane below the active pane",
                "select-pane -D",
            ),
            (
                "Left",
                true,
                "Select the pane to the left of the active pane",
                "select-pane -L",
            ),
            (
                "Right",
                true,
                "Select the pane to the right of the active pane",
                "select-pane -R",
            ),
            (
                "M-1",
                false,
                "Set the even-horizontal layout",
                "select-layout even-horizontal",
            ),
            (
                "M-2",
                false,
                "Set the even-vertical layout",
                "select-layout even-vertical",
            ),
            (
                "M-3",
                false,
                "Set the main-horizontal layout",
                "select-layout main-horizontal",
            ),
            (
                "M-4",
                false,
                "Set the main-vertical layout",
                "select-layout main-vertical",
            ),
            (
                "M-5",
                false,
                "Select the tiled layout",
                "select-layout tiled",
            ),
            (
                "M-6",
                false,
                "Set the main-horizontal-mirrored layout",
                "select-layout main-horizontal-mirrored",
            ),
            (
                "M-7",
                false,
                "Set the main-vertical-mirrored layout",
                "select-layout main-vertical-mirrored",
            ),
            (
                "M-n",
                false,
                "Select the next window with an alert",
                "next-window -a",
            ),
            (
                "M-o",
                false,
                "Rotate through the panes in reverse",
                "rotate-window -D",
            ),
            (
                "M-p",
                false,
                "Select the previous window with an alert",
                "previous-window -a",
            ),
            ("M-Up", true, "Resize the pane up by 5", "resize-pane -U 5"),
            (
                "M-Down",
                true,
                "Resize the pane down by 5",
                "resize-pane -D 5",
            ),
            (
                "M-Left",
                true,
                "Resize the pane left by 5",
                "resize-pane -L 5",
            ),
            (
                "M-Right",
                true,
                "Resize the pane right by 5",
                "resize-pane -R 5",
            ),
            ("C-b", false, "Send the prefix key", "send-prefix"),
            ("C-o", false, "Rotate through the panes", "rotate-window"),
            ("C-z", false, "Suspend the current client", "suspend-client"),
            ("C-Up", true, "Resize the pane up", "resize-pane -U"),
            ("C-Down", true, "Resize the pane down", "resize-pane -D"),
            ("C-Left", true, "Resize the pane left", "resize-pane -L"),
            ("C-Right", true, "Resize the pane right", "resize-pane -R"),
            (
                "S-Up",
                true,
                "Move the visible part of the window up",
                "refresh-client -U 10",
            ),
            (
                "S-Down",
                true,
                "Move the visible part of the window down",
                "refresh-client -D 10",
            ),
            (
                "S-Left",
                true,
                "Move the visible part of the window left",
                "refresh-client -L 10",
            ),
            (
                "S-Right",
                true,
                "Move the visible part of the window right",
                "refresh-client -R 10",
            ),
        ];
        for &(name, repeat, note, command) in PREFIX_DEFAULTS {
            let key =
                parse_key_name(name).unwrap_or_else(|| panic!("invalid default key name: {name}"));
            if let Some(binding) = self
                .key_tables
                .get_mut("prefix")
                .and_then(|bindings| bindings.get_mut(&key))
            {
                binding.repeat = repeat;
                binding.note = Some(note.to_string());
            } else {
                self.bind_key(
                    "prefix",
                    key,
                    super::command::binding_words(command),
                    repeat,
                    Some(note.to_string()),
                );
            }
        }
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

    pub(crate) fn client_prompt_registry(&self) -> Rc<ClientPromptRegistry> {
        Rc::clone(&self.client_prompts)
    }

    pub(crate) fn client_render_registry(&self) -> Rc<ClientRenderRegistry> {
        Rc::clone(&self.client_renders)
    }

    pub(crate) fn client_snapshots(&self) -> Vec<ClientSnapshot> {
        self.client_renders.snapshots()
    }

    /// Each attached client's resolved terminal description, for
    /// `show-messages -T`: `(name, TERM, resolved)`. The only path that pays
    /// for a `ResolvedTerm` clone.
    pub(crate) fn client_terminals(&self) -> Vec<(String, String, ResolvedTerm)> {
        self.client_renders.with_entries(|entries| {
            entries
                .filter_map(|entry| {
                    let terminal = entry.terminal.clone()?;
                    Some((entry.name.clone(), entry.term.clone(), terminal))
                })
                .collect()
        })
    }

    /// Whether a client other than `client_name` holds the session's size:
    /// tmux's `ignore_client_size` — an `ignore-size` client is ignored only
    /// while some counted client is unflagged.
    pub(crate) fn other_client_constrains_size(&self, session_id: u32, client_name: &str) -> bool {
        self.client_renders.with_entries(|mut entries| {
            entries.any(|entry| {
                entry.session_id == session_id
                    && entry.name != client_name
                    && !entry.ignore_size
                    && entry.counts_for_sizing()
            })
        })
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
        reply: Option<PromptReply>,
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
        reply: Option<PromptReply>,
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

    pub(crate) fn wait_registry(&self) -> Rc<WaitRegistry> {
        Rc::clone(&self.wait_registry)
    }

    #[allow(dead_code)]
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
        exec: Option<&str>,
    ) -> ClientActionResult {
        self.client_renders.detach_client(target, invoking_tty, exec)
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

    /// Toggle `switch-client -r` on the target client, reporting its name.
    pub(crate) fn toggle_client_read_only(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> Result<String, ClientActionResult> {
        self.client_renders
            .toggle_client_read_only(target, invoking_tty)
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

    /// The prompt types tmux keeps a history for, in the order it writes them
    /// to the history file.
    const PROMPT_TYPES: [&'static str; 4] = ["command", "search", "target", "window-target"];

    /// The file `history-file` names, or `None` when the option is empty or
    /// names something tmux's `status_prompt_find_history_file` refuses: only
    /// an absolute path or one under `~/` is accepted.
    fn prompt_history_file(&self) -> Option<PathBuf> {
        let value = self.server_options().get("history-file")?;
        if value.is_empty() {
            return None;
        }
        if value.starts_with('/') {
            return Some(PathBuf::from(value));
        }
        let rest = value.strip_prefix("~/")?;
        Some(PathBuf::from(std::env::var_os("HOME")?).join(rest))
    }

    /// tmux's `status_prompt_load_history`, run once per history file.
    ///
    /// tmux loads it when the configuration has been read; the hmux daemon has
    /// no configuration interface, so the load happens when the option first
    /// names a file instead.
    pub(crate) fn load_prompt_history(&mut self) {
        let Some(path) = self.prompt_history_file() else {
            return;
        };
        if self.prompt_history_file_loaded.as_ref() == Some(&path) {
            return;
        }
        self.prompt_history_file_loaded = Some(path.clone());
        let Ok(contents) = std::fs::read_to_string(&path) else {
            return;
        };
        for line in contents.lines().filter(|line| !line.is_empty()) {
            // An unknown type is an old-format file, whose lines are all
            // command history.
            match line.split_once(':') {
                Some((prompt_type, value)) if Self::PROMPT_TYPES.contains(&prompt_type) => {
                    self.add_prompt_history(prompt_type, value)
                }
                _ => self.add_prompt_history("command", line),
            }
        }
    }

    /// tmux's `status_prompt_save_history`, run as the server exits.
    pub(crate) fn save_prompt_history(&self) {
        let Some(path) = self.prompt_history_file() else {
            return;
        };
        let mut contents = String::new();
        for prompt_type in Self::PROMPT_TYPES {
            for value in self.prompt_history(prompt_type) {
                contents.push_str(prompt_type);
                contents.push(':');
                contents.push_str(value);
                contents.push('\n');
            }
        }
        let _ = std::fs::write(&path, contents);
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

    pub(crate) fn option_changed(&mut self, name: &str) {
        if name == "monitor-silence" {
            self.reset_silence_timers();
        }
        if name == "automatic-rename" {
            // tmux's `options_push_changes`: turning the option on has to name
            // the window even though nothing in the pane moved.
            for window in self.windows.values() {
                if let Some(node) = window.panes.get(window.active) {
                    node.pane.observation_state().note_changed();
                }
            }
        }
        if option_affects_render(name) {
            self.invalidate_all_clients(option_invalidation(name));
        }
        if matches!(
            name,
            "alternate-screen"
                | "allow-set-title"
                | "allow-passthrough"
                | "history-limit"
                | "input-buffer-size"
                | "scroll-on-clear"
        ) {
            self.refresh_pane_options();
        }
        // The width policy is server-wide in tmux — one cache, rebuilt whenever
        // either option is set — so it is pushed rather than looked up. Only
        // characters written after this see the change, as in tmux: a cell
        // already in the grid keeps the width it was placed with.
        if name == "codepoint-widths" {
            let options = OptionsView::one(self.global_options.server());
            width::set_codepoint_widths(options.array_values(name));
        }
        if name == "variation-selector-always-wide" {
            let options = OptionsView::one(self.global_options.server());
            width::set_variation_selector_always_wide(options.get(name) != Some("off"));
        }
    }

    /// Whether the outer listener should stop accepting clients.
    pub fn shutdown_requested(&self) -> bool {
        self.shutdown_requested
    }

    fn server_option_is_on(&self, name: &str, default: bool) -> bool {
        match self.server_options().get(name) {
            Some("on" | "yes" | "true" | "1") => true,
            Some("off" | "no" | "false" | "0") => false,
            Some(_) | None => default,
        }
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

    /// Record the pathname the server is listening on.
    pub fn set_socket_path(&mut self, path: impl Into<PathBuf>) {
        self.environment_generation += 1;
        self.socket_path = path.into();
    }

    /// The pathname the server is listening on (`#{socket_path}`).
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
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

    /// The option rows customize mode shows, grouped under the headings tmux's
    /// `window_customize_build` uses — one per option table, each entry
    /// carrying the scope text `window_customize_scope_text` prints for it
    /// (empty for a global one).
    pub(crate) fn customize_option_sections(
        &self,
        target: &str,
    ) -> io::Result<Vec<(&'static str, Vec<CustomizeOption>)>> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session = &self.sessions[resolved.session];
        let window = self.window(resolved.session, resolved.window);
        let pane = &window.panes[resolved.pane];
        let collect = |view: OptionsView<'_>, scope: String| {
            let mut entries = Vec::new();
            let mut arrays = BTreeMap::new();
            for (name, value) in view.iter_effective() {
                let (base, index) = super::options::parse_option_name(name).unwrap_or((name, None));
                if super::options::is_array_option(base) {
                    arrays
                        .entry(base.to_owned())
                        .and_modify(|has_entries| {
                            *has_entries |= index.is_some() && !value.is_empty();
                        })
                        .or_insert(index.is_some() && !value.is_empty());
                } else {
                    entries.push(CustomizeOption {
                        name: name.to_owned(),
                        value: value.to_owned(),
                        scope: scope.clone(),
                        is_array: false,
                        array_has_entries: false,
                    });
                }
            }
            entries.extend(
                arrays
                    .into_iter()
                    .map(|(name, array_has_entries)| CustomizeOption {
                        name,
                        value: String::new(),
                        scope: scope.clone(),
                        is_array: true,
                        array_has_entries,
                    }),
            );
            entries.sort_by(|left, right| left.name.cmp(&right.name));
            entries
        };
        let mut sections = Vec::new();
        sections.push((
            "Server Options",
            collect(
                OptionsView::one(self.global_options.server()),
                String::new(),
            ),
        ));
        let mut session_options = collect(
            OptionsView::one(self.global_options.session()),
            String::new(),
        );
        let session_overrides = collect(
            OptionsView::one(session.option_overrides()),
            format!("session {}", session.name),
        );
        // tmux 3.7b's customize-mode builder resolves a session-only user
        // option through the global session table when it builds this list.
        // Keep that presentation quirk here without changing option lookup or
        // storage, which continue to expose the real session-local value.
        if let Some(global_user) = session_options
            .iter()
            .find(|entry| entry.name.starts_with('@'))
            .cloned()
        {
            if session_overrides
                .iter()
                .any(|entry| entry.name.starts_with('@'))
            {
                let insert_at = session_options
                    .iter()
                    .position(|entry| entry.name == global_user.name)
                    .map_or(session_options.len(), |index| index + 1);
                session_options.insert(insert_at, global_user);
            }
            session_options.extend(session_overrides);
        } else {
            session_options.extend(session_overrides);
        }
        sections.push(("Session Options", session_options));
        let mut window_options = collect(
            OptionsView::one(self.global_options.window()),
            String::new(),
        );
        window_options.extend(collect(
            OptionsView::one(window.option_overrides()),
            format!("window {}", session.windows[resolved.window].index),
        ));
        window_options.extend(collect(
            OptionsView::one(pane.option_overrides()),
            format!("pane {}", resolved.pane),
        ));
        sections.push(("Window & Pane Options", window_options));
        Ok(sections)
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

    /// Snapshot the target window's layout into tmux's `w->old_layout` slot,
    /// returning the value it replaces — what `select-layout -o` restores.
    pub(crate) fn snapshot_window_layout(&mut self, target: &str) -> io::Result<Option<String>> {
        let resolved = self.resolve_window_target(target)?;
        let window = self.window_mut(resolved.session, resolved.window);
        let dump = checksummed_layout_dump(&window.layout);
        Ok(window.old_layout.replace(dump))
    }

    /// Put a previously snapshot `old_layout` back, for a failed
    /// `select-layout` (tmux's error path restores `w->old_layout`).
    pub(crate) fn restore_window_old_layout(&mut self, target: &str, value: Option<String>) {
        if let Ok(resolved) = self.resolve_window_target(target) {
            self.window_mut(resolved.session, resolved.window)
                .old_layout = value;
        }
    }

    /// The last preset layout applied to the target window, which a bare
    /// `select-layout` reapplies (tmux's `w->lastlayout`).
    pub(crate) fn window_last_preset_layout(&self, target: &str) -> io::Result<Option<usize>> {
        let resolved = self.resolve_window_target(target)?;
        Ok(self.window(resolved.session, resolved.window).last_layout)
    }

    /// `select-layout -E`: spread the target pane's siblings evenly, climbing
    /// outward from its innermost split (tmux's `layout_spread_out`).
    pub(crate) fn spread_window_layout(&mut self, target: &str) -> io::Result<()> {
        let t = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let window_id = self.sessions[t.session].windows[t.window].id;
        let affected_sessions = self
            .sessions
            .iter()
            .filter(|session| session.windows.iter().any(|link| link.id == window_id))
            .map(|session| session.id)
            .collect::<Vec<_>>();
        let win = self.window_mut(t.session, t.window);
        let pane_id = win.panes[t.pane].id;
        if win.layout.spread_pane(pane_id) {
            resize_panes_to_layout(win)?;
            for session_id in affected_sessions {
                self.invalidate_session(
                    session_id,
                    RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
                );
            }
            self.notify_window("window-layout-changed", window_id);
        }
        Ok(())
    }

    pub(crate) fn cycle_layout(&mut self, target: &str, forward: bool) -> io::Result<()> {
        let resolved = self.resolve_window_target(target)?;
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
        // tmux's `layout_parse` resizes the window to the layout it just parsed,
        // then runs `recalculate_sizes` — so on an attached window the client
        // set wins straight back, and only a clientless one keeps this size.
        window.cols = size.0;
        window.rows = size.1;
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

    /// tmux's `tty_window_offset1`: where a client's terminal sits over the
    /// window it is showing.
    ///
    /// When the window fits, the client sees all of it at the origin and any
    /// pan is abandoned. When it does not, the client shows a `sx`×`sy`
    /// viewport at `(ox, oy)` — an explicit `refresh-client` pan if one is live
    /// for this window, otherwise a window that follows the cursor so the
    /// active pane's cursor stays on screen.
    /// The `user-keys` array for a target, by index: entry *n* is the sequence
    /// tmux's `tty_keys_user` reads as the key `Usern`.
    pub(crate) fn user_key_sequences(&self, target: &str) -> Vec<String> {
        let Some(resolved) = self.resolve(target) else {
            return Vec::new();
        };
        OptionsView::two(
            self.sessions[resolved.session].option_overrides(),
            self.global_options.server(),
        )
        .array_values("user-keys")
        .into_iter()
        .map(str::to_owned)
        .collect()
    }

    /// The effective `base-index` (first/lowest window index), default 0.
    fn base_index(&self) -> u32 {
        self.global_options
            .session()
            .get("base-index")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    }

    /// tmux stamps `c->activity_time` alongside the session's whenever a key
    /// arrives; it is what orders `cmd_find_best_client`.
    pub(crate) fn touch_client_activity(&mut self, client: &str, session_id: u32) {
        self.touch_session_activity(session_id, false);
        self.client_renders
            .touch_client_activity(client, now_micros());
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
        let clients = self.client_renders.with_entries(|entries| {
            entries
                .filter_map(|entry| {
                    sessions
                        .get(&entry.session_id)
                        .cloned()
                        .map(|name| (entry.name.clone(), (entry.session_id, name)))
                })
                .collect::<BTreeMap<_, _>>()
        });
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
                                clear_copy_selection(state);
                            }
                            end_mode = command.ends_with("and-cancel");
                            Some(data)
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
                }
                output
            }
        };
        self.invalidate_session(session_id, RenderInvalidation::MODE);
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
            self.notify_pane("pane-set-clipboard", pane_id);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_loop::test_driver::run_on_loop;
    use crate::vt::screen::{CellSemantic, CellWidth, Grid, GridCell, GridRow, RowFlags};

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
                hyperlink_id: None,
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
                hyperlink_id: None,
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
