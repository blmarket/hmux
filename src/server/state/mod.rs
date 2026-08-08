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
mod resolve;
mod sizing;
mod target;

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
use sizing::{
    clamp_window_size, fold_client_sizes, parse_size_pair, scrollbar_style_columns, SizingClient,
};
pub(crate) use sizing::{
    pane_slider, ClientViewport, WindowResizeAdjust, WindowResizeRequest, WindowSizePolicy,
};
pub(crate) use target::Target;
use target::{pane_not_found, parse_index_target, split_pane_target, window_not_found, TargetKind};

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use super::input_keys::{
    self, ExtendedKeys, ExtendedKeysFormat, PaneKey, PaneKeyEncoding, PaneKeyModes, PaneKeyOptions,
};
use super::key::{parse_key_name, KeyCode};
use super::options::{GlobalOptions, OptionSet, OptionsView};
use super::pane::{
    Pane, PaneClipboardEvent, PaneIo, PaneKeyState, PaneOutputPolicy, PanePassthrough,
    PaneSpawnSpec, PassthroughPolicy,
};
use super::term::ResolvedTerm;
use crate::vt::input::MouseEvent;
use crate::vt::screen::{ScreenOptions, VtScreen};
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

    /// Hand a pipe job the loop owns from here. `copy-pipe` uses this for the
    /// child it feeds a selection to, which belongs to no pane.
    pub(crate) fn adopt_pipe(&mut self, pipe: super::pane::PanePipeIo) {
        self.new_pipes.push(pipe);
    }

    /// Pipe children opened since the last call, across every pane and the
    /// pane-less jobs adopted directly.
    pub(crate) fn take_new_pane_pipes(&mut self) -> Vec<super::pane::PanePipeIo> {
        let mut pipes = std::mem::take(&mut self.new_pipes);
        pipes.extend(
            self.windows
                .values_mut()
                .flat_map(|window| window.panes.iter_mut())
                .flat_map(|node| node.pane.take_new_pipes()),
        );
        pipes
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

    /// When this server state was created (`#{start_time}`).
    pub(crate) fn started_epoch(&self) -> i64 {
        self.started_epoch
    }

    /// The clients whose session currently shows `window_id` — tmux's
    /// `#{window_active_clients}` walk over `c->session->curw`.
    pub(crate) fn window_active_client_names(&self, window_id: u32) -> Vec<String> {
        let showing = self
            .sessions
            .iter()
            .filter(|session| {
                session
                    .windows
                    .get(session.active)
                    .is_some_and(|link| self.window_for_link(link).id == window_id)
            })
            .map(|session| session.id)
            .collect::<BTreeSet<u32>>();
        self.client_renders.names_for_sessions(&showing)
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
            // tmux's `session_select` tail: a selection runs
            // `window_update_activity`, so the alert check fires again and an
            // unattached session's monitor-activity re-flags even the winlink
            // it just selected.
            let window_id = self.sessions[session_pos].windows[position].id;
            let monitor_activity = self.windows.get(&window_id).is_some_and(|window| {
                window.options(&self.global_options).get("monitor-activity") == Some("on")
            });
            self.window_last_activity
                .insert(window_id, std::time::Instant::now());
            if monitor_activity {
                self.deliver_alert(window_id, ALERT_ACTIVITY);
            }
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
        if self.windows.len() != window_count {}
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
        self.client_renders.with_entries(|mut entries| {
            entries.any(|entry| {
                entry.focused
                    && self
                        .current_window_of_session(entry.session_id)
                        .is_some_and(|current| current == window_id)
            })
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

    pub(crate) fn session_id(&self, name: &str) -> Option<u32> {
        self.resolve_session(name).map(|session| session.id)
    }

    /// Record where a session was created, which is where the `#()` jobs of the
    /// clients attached to it run.
    pub(crate) fn set_session_cwd(&mut self, session_id: u32, cwd: Option<PathBuf>) {
        if let Some(session) = self
            .sessions
            .iter_mut()
            .find(|session| session.id == session_id)
        {
            session.cwd = cwd;
        }
    }

    pub(crate) fn session_by_id(&self, session_id: u32) -> Option<&Session> {
        self.sessions
            .iter()
            .find(|session| session.id == session_id)
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

    /// Apply pane child-exit lifecycle (`remain-on-exit`, then `exit-empty`).
    ///
    /// Returns true when at least one pane was removed. Empty windows and
    /// sessions are removed with it, matching tmux's ownership hierarchy.
    pub fn reap_exited_panes(&mut self) -> bool {
        let had_sessions = !self.sessions.is_empty();
        let mut removed = false;
        for window in self.windows.values_mut() {
            for pane in &mut window.panes {
                if pane.pane.has_exited() {
                    pane.pane.collect_exited_child();
                }
            }
        }
        let retained = self
            .windows
            .values()
            .flat_map(|window| {
                window.panes.iter().filter_map(|pane| {
                    // `on` and `key` keep the dead pane; `failed` keeps it
                    // only when the child exited non-zero (or to a signal).
                    let keep = match pane
                        .options(window, &self.global_options)
                        .get("remain-on-exit")
                    {
                        Some("on") | Some("key") => true,
                        Some("failed") => pane
                            .pane
                            .death()
                            .is_some_and(|death| death.status != Some(0)),
                        _ => false,
                    };
                    keep.then_some(pane.id)
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
        // tmux's `server_destroy_pane` paints the expanded remain-on-exit-format
        // onto a pane it keeps: full scroll region, cursor to the bottom-left,
        // one linefeed, then the text — and the cursor is hidden.
        for &pane_id in newly_exited.iter().filter(|id| retained.contains(id)) {
            let Some((window, pane)) = self.windows.values().find_map(|window| {
                window
                    .panes
                    .iter()
                    .find(|pane| pane.id == pane_id)
                    .map(|pane| (window, pane))
            }) else {
                continue;
            };
            let template = pane
                .options(window, &self.global_options)
                .get("remain-on-exit-format")
                .unwrap_or_default()
                .to_string();
            if template.is_empty() {
                continue;
            }
            let death = pane.pane.death();
            let mut vars = super::format::Vars::new();
            vars.set("pane_id", format!("%{pane_id}"))
                .set("pane_dead", "1")
                .set(
                    "pane_dead_status",
                    death
                        .and_then(|death| death.status)
                        .map(|status| status.to_string())
                        .unwrap_or_default(),
                )
                .set(
                    "pane_dead_signal",
                    death
                        .and_then(|death| death.signal)
                        .map(|signal| signal.to_string())
                        .unwrap_or_default(),
                )
                .set(
                    "pane_dead_time",
                    death
                        .map(|death| {
                            death
                                .at
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs()
                                .to_string()
                        })
                        .unwrap_or_default(),
                );
            let expanded = super::format::expand(&template, &vars);
            let (_, rows) = pane.pane.size();
            pane.pane
                .feed(format!("\x1b[r\x1b[{rows};1H\n\x1b[?25l{expanded}").as_bytes());
        }
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
                        && pane.pane.child_reaped()
                        && !retained.contains(&pane.id)
                })
                .map(|pane| pane.id)
                .collect::<Vec<_>>();
            let before = window.panes.len();
            window.panes.retain(|pane| {
                !pane.pane.has_exited() || !pane.pane.child_reaped() || retained.contains(&pane.id)
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

        if had_sessions && self.sessions.is_empty() && self.exit_empty_policy() != ExitEmpty::Off {
            self.shutdown_requested = true;
        }
        if removed {}
        removed
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
        if self.find_exact(name).is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("duplicate session: {name}"),
            ));
        }
        // The session does not exist yet, so only the global `default-size` can
        // apply — tmux reads `global_s_options` here for the same reason.
        let (cols, rows) = self
            .global_options
            .session()
            .get("default-size")
            .and_then(parse_size_pair)
            .map_or((self.default_cols, self.default_rows), clamp_window_size);
        // tmux creates the session and the pane before spawning the pane's
        // process, so both ids are in its environment. hmux allocates them once
        // the spawn has succeeded, so their values are peeked here and consumed
        // unchanged below.
        let spec = fill_spec_spawn_ids(spec, self.next_pane_id, self.next_session_id);
        let start_command = pane_start_command(&spec);
        let pane = match spec {
            PaneSpec::Inert => Pane::inert(cols, rows)?,
            PaneSpec::Command(argv) => {
                let refs: Vec<&str> = argv.iter().map(String::as_str).collect();
                Pane::spawn(&refs, None, cols, rows)?
            }
            PaneSpec::CommandIn(argv, cwd) => {
                let refs: Vec<&str> = argv.iter().map(String::as_str).collect();
                Pane::spawn(&refs, Some(&cwd), cols, rows)?
            }
            // Only the popup conversion produces one of these, and it splits an
            // existing window rather than creating a session.
            PaneSpec::Existing(pane) => *pane,
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
                    scrollbar_columns: 0,
                    border_status: None,
                    unseen_changes: false,
                    active_point: 0,
                    options: OptionSet::default(),
                }],
                active: 0,
                last_pane: None,
                zoomed: false,
                activity_epoch: now_epoch(),
                name_time_micros: 0,
                name_in_mode: false,
                scrollbars_on_left: false,
                cols,
                rows,
                manual_size: (cols, rows),
                latest_client: None,
                pending_size: None,
                layout: LayoutCell::pane(pane_id, cols, rows),
                last_layout: None,
                old_layout: None,
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
            cwd: None,
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
        if self.find_exact(name).is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("duplicate session: {name}"),
            ));
        }
        let target_session = self.session_index(target);
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
        if from != to && self.find_exact(to).is_some() {
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

        // tmux's `default_window_size`: a new window is sized from the clients
        // that can see it, or from the session's `default-size` when none can.
        let session_pos = session_pos.expect("session presence checked above");
        let (cols, rows) = self.default_window_size(session_pos, None, None);
        // Spawn the pane before mutating counters so a spawn failure leaves state
        // untouched; the id the pane will take is peeked for its environment.
        let argv = fill_spawn_ids(argv, self.next_pane_id, session_id);
        let refs: Vec<&str> = argv.iter().map(String::as_str).collect();
        let pane = Pane::spawn(&refs, cwd, cols, rows)?;

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
                    scrollbar_columns: 0,
                    border_status: None,
                    unseen_changes: false,
                    active_point: 0,
                    options: OptionSet::default(),
                }],
                active: 0,
                last_pane: None,
                zoomed: false,
                activity_epoch: now_epoch(),
                name_time_micros: 0,
                name_in_mode: false,
                scrollbars_on_left: false,
                cols,
                rows,
                manual_size: (cols, rows),
                latest_client: None,
                pending_size: None,
                layout: LayoutCell::pane(pane_id, cols, rows),
                last_layout: None,
                old_layout: None,
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

        // tmux's `default_window_size`, as for an appended window. Spawn before
        // mutating counters so a failure leaves state untouched; the id the pane
        // will take is peeked for its environment.
        let (cols, rows) = self.default_window_size(session_pos, None, None);
        let argv = fill_spawn_ids(argv, self.next_pane_id, session_id);
        let refs: Vec<&str> = argv.iter().map(String::as_str).collect();
        let pane = Pane::spawn(&refs, cwd, cols, rows)?;

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
                    scrollbar_columns: 0,
                    border_status: None,
                    unseen_changes: false,
                    active_point: 0,
                    options: OptionSet::default(),
                }],
                active: 0,
                last_pane: None,
                zoomed: false,
                activity_epoch: now_epoch(),
                name_time_micros: 0,
                name_in_mode: false,
                scrollbars_on_left: false,
                cols,
                rows,
                manual_size: (cols, rows),
                latest_client: None,
                pending_size: None,
                layout: LayoutCell::pane(pane_id, cols, rows),
                last_layout: None,
                old_layout: None,
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

    /// Resolve just the session named by a target (its part before any `:`),
    /// accepting a name or a `$id`. Used by `has-session`.
    /// Seed the global environment from the daemon's own, as tmux fills
    /// `global_environ` from the environment its server was started with. The
    /// unit tests build a state without this, so they stay hermetic.
    pub fn seed_global_environment(&mut self) {
        self.environment_generation += 1;
        for (name, value) in std::env::vars() {
            self.seeded_environment.insert(name.clone());
            self.environment.insert(name, value);
        }
        if let Ok(cwd) = std::env::current_dir() {
            self.seeded_environment.insert("PWD".to_owned());
            self.environment
                .insert("PWD".to_owned(), cwd.to_string_lossy().into_owned());
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
        let t = self.resolve_window_target(target)?;
        let window_id = self.sessions[t.session].windows[t.window].id;
        self.destroy_window_id(window_id);
        Ok(())
    }

    /// `kill-window -a [-t target]`: kill every window in the target's session
    /// *except* the target itself, matching tmux. The survivor becomes the
    /// session's only (and active) window.
    pub fn kill_other_windows(&mut self, target: &str) -> io::Result<()> {
        let t = self.resolve_window_target(target)?;
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
        let t = self.resolve_window_target(target)?;
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
        let t = self.resolve_window_target(target)?;
        let session_id = self.sessions[t.session].id;
        if self.sessions[t.session].active != t.window {
            self.select_session_window(t.session, t.window);
            self.recalculate_sizes()?;
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
                self.recalculate_sizes()?;
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

    /// tmux's `alerts_reset_all`, which `options_push_changes` runs whenever
    /// `monitor-silence` is set. Every window drops its raised silence
    /// condition and restarts its timer from the option change, so a window
    /// that has been quiet since before the change is not alerted for the
    /// silence that preceded it.
    fn reset_silence_timers(&mut self) {
        let now = std::time::Instant::now();
        for window in self.windows.values_mut() {
            window.pending_alerts &= !ALERT_SILENCE;
        }
        for window_id in self.windows.keys().copied().collect::<Vec<_>>() {
            self.window_last_activity.insert(window_id, now);
        }
        self.silence_alerted.clear();
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
        let mut unseen_changes = Vec::new();
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
                let pane_output = revision > previous_revision;
                // tmux flags the pane itself when output lands while a mode is
                // showing a frozen grid; the flag is not about the window's
                // activity condition, which is why it is per pane.
                if pane_output && pane.mode.is_some() {
                    unseen_changes.push(pane.id);
                }
                output |= pane_output;
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

        // Leaving the last mode drops the flag, exactly as tmux's
        // `window_pane_reset_mode` does.
        for pane in self.windows.values_mut().flat_map(|w| &mut w.panes) {
            pane.unseen_changes =
                pane.mode.is_some() && (pane.unseen_changes || unseen_changes.contains(&pane.id));
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

    /// Recompute how many columns each pane gives up to a scrollbar, resizing
    /// the panes of any window whose answer moved.
    ///
    /// tmux's `window_pane_show_scrollbar`: `on` shows one for every pane,
    /// `modal` only for a pane in a mode, and neither shows one for a pane on
    /// its alternate screen. `layout_fix_panes` then takes the style's width
    /// and padding off the pane.
    pub(crate) fn refresh_pane_scrollbars(&mut self) -> io::Result<()> {
        let mut changed = Vec::new();
        for (window_id, window) in &mut self.windows {
            let (mode, reserved, on_left) = {
                let options = window.options(&self.global_options);
                let mode = options.get("pane-scrollbars").unwrap_or("off").to_owned();
                let reserved = if mode == "off" {
                    0
                } else {
                    scrollbar_style_columns(options.get("pane-scrollbars-style"))
                };
                let on_left = options.get("pane-scrollbars-position") == Some("left");
                (mode, reserved, on_left)
            };
            window.scrollbars_on_left = on_left;
            let modal = mode == "modal";
            let border_status = match window
                .options(&self.global_options)
                .get("pane-border-status")
                .unwrap_or("off")
            {
                "top" => Some(PaneBorderStatus::Top),
                "bottom" => Some(PaneBorderStatus::Bottom),
                _ => None,
            };
            let window_rows = window.rows;
            let rects = window
                .panes
                .iter()
                .map(|node| window.layout.pane_rect(node.id))
                .collect::<Vec<_>>();
            let mut moved = false;
            for (node, rect) in window.panes.iter_mut().zip(rects) {
                let shown = reserved != 0
                    && (!modal || node.mode.is_some())
                    && !node.pane.alternate_screen().0;
                let columns = if shown { reserved } else { 0 };
                moved |= node.scrollbar_columns != columns;
                node.scrollbar_columns = columns;
                // Only the pane against the window's own top (or bottom) edge
                // gives up a row; the rest write on a border they already have.
                let side = rect.and_then(|rect| match border_status {
                    Some(PaneBorderStatus::Top) if rect.top == 0 => Some(PaneBorderStatus::Top),
                    Some(PaneBorderStatus::Bottom) if rect.top + rect.height >= window_rows => {
                        Some(PaneBorderStatus::Bottom)
                    }
                    _ => None,
                });
                moved |= node.border_status != side;
                node.border_status = side;
            }
            if moved {
                changed.push(*window_id);
            }
        }
        for window_id in changed {
            if let Some(window) = self.windows.get_mut(&window_id) {
                resize_panes_to_layout(window)?;
            }
            let sessions = self
                .sessions
                .iter()
                .filter(|session| session.windows.iter().any(|link| link.id == window_id))
                .map(|session| session.id)
                .collect::<Vec<_>>();
            for session_id in sessions {
                self.invalidate_session(session_id, RenderInvalidation::LAYOUT);
            }
        }
        Ok(())
    }

    /// Push each pane the options it has to consult about its own output: how
    /// the bytes are parsed, and what an operation on the grid does.
    ///
    /// tmux reads them from `wp->options` at the moment they matter, with the
    /// whole server in reach. hmux runs both the tokenizer and the screen off
    /// the state lock, so the resolved values are pushed to the pane instead —
    /// once per server loop, and again as soon as a `set-option` touches one of
    /// them.
    pub(crate) fn refresh_pane_options(&self) {
        // `history-limit` is a session option, but the rows live in panes.
        // Apply each session's effective value to the panes in its windows so
        // lowering it trims already populated grids, as tmux does.
        for session in &self.sessions {
            let history_limit = session
                .options(&self.global_options)
                .get("history-limit")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(2000);
            for link in &session.windows {
                let Some(window) = self.windows.get(&link.id) else {
                    continue;
                };
                for node in &window.panes {
                    node.pane.set_history_limit(history_limit);
                }
            }
        }
        for window in self.windows.values() {
            for node in &window.panes {
                let options = node.options(window, &self.global_options);
                node.pane.set_screen_options(ScreenOptions {
                    scroll_on_clear: options.get("scroll-on-clear") != Some("off"),
                });
                node.pane.set_output_policy(PaneOutputPolicy {
                    alternate_screen: options.get("alternate-screen") != Some("off"),
                    allow_set_title: options.get("allow-set-title") != Some("off"),
                    passthrough: match options.get("allow-passthrough") {
                        Some("on") => PassthroughPolicy::Visible,
                        Some("all") => PassthroughPolicy::Always,
                        _ => PassthroughPolicy::Off,
                    },
                    // Server-scoped, so the window view never sees it.
                    input_buffer_size: self
                        .server_options()
                        .get("input-buffer-size")
                        .and_then(|value| value.parse().ok())
                        .unwrap_or(1_048_576),
                    cursor_style: cursor_style_parameter(
                        options.get("cursor-style").unwrap_or("default"),
                    ),
                    // The index an entry is stored at *is* the palette slot it
                    // fills, so the array is read with its indexes rather than
                    // as a list.
                    palette: {
                        let entries = OptionsView::three(
                            node.option_overrides(),
                            window.option_overrides(),
                            self.global_options.window(),
                        )
                        .array_entries("pane-colours");
                        let mut palette =
                            vec![None; entries.last().map_or(0, |(index, _)| *index as usize + 1)];
                        for (index, value) in entries {
                            palette[index as usize] = super::pane::parse_packed_colour(value);
                        }
                        palette
                    },
                });
            }
        }
    }

    /// Put the `DCS tmux;` payloads panes emitted since the last pass onto the
    /// client ttys they are allowed to reach, mirroring tmux's
    /// `screen_write_rawstring` and the client walk in `tty_write`.
    ///
    /// `allow-passthrough on` reaches the clients whose *current* window holds
    /// the pane; `all` also reaches those that merely have the window linked.
    /// Which of the two applied was decided when the sequence completed, since
    /// that is when tmux reads the option.
    pub(crate) fn process_pane_passthrough(&mut self) {
        let panes = self
            .windows
            .iter()
            .map(|(window_id, window)| {
                (
                    *window_id,
                    window
                        .panes
                        .iter()
                        .map(|node| node.pane.observation_state())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        for (window_id, observations) in panes {
            for observation in observations {
                for PanePassthrough {
                    data,
                    invisible_panes,
                } in observation.take_passthrough()
                {
                    let sessions = self
                        .sessions
                        .iter()
                        .filter(|session| {
                            if invisible_panes {
                                session.windows.iter().any(|link| link.id == window_id)
                            } else {
                                session
                                    .windows
                                    .get(session.active)
                                    .is_some_and(|link| link.id == window_id)
                            }
                        })
                        .map(|session| session.id)
                        .collect::<BTreeSet<_>>();
                    self.client_renders.write_client_output(&sessions, &data);
                }
            }
        }
    }

    /// Apply the clipboard policy to the OSC 52 sequences panes emitted since
    /// the last pass, mirroring tmux's `input_osc_52`.
    ///
    /// Applications may only touch the clipboard under `set-clipboard on`; the
    /// sequence is not even parsed otherwise. A query is then answered from the
    /// newest paste buffer when `get-clipboard` is `buffer`, and put to the
    /// client's own terminal under `request`/`both`, whose answer comes back
    /// through [`Self::deliver_terminal_reply`].
    pub(crate) fn process_pane_clipboard(&mut self) {
        let panes = self
            .windows
            .values()
            .flat_map(|window| window.panes.iter())
            .map(|node| (node.id, node.pane.observation_state()))
            .collect::<Vec<_>>();
        let allow_applications = self.server_options().get("set-clipboard") == Some("on");
        let get_clipboard = self
            .server_options()
            .get("get-clipboard")
            .unwrap_or("buffer");
        let answer_from_buffer = get_clipboard == "buffer";
        let forward_to_terminal = matches!(get_clipboard, "request" | "both");
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
                        if forward_to_terminal {
                            // tmux's `input_add_request`: ask the client's own
                            // terminal instead, with the `Ms` capability the
                            // clipboard writes already use, and remember which
                            // pane is owed the answer. With no client to ask,
                            // the pane goes unanswered as it does in tmux.
                            let Some(client) = self.request_client_for_pane(pane_id) else {
                                continue;
                            };
                            if self.set_client_selection(Some(&client), None, None)
                                == ClientActionResult::Queued
                            {
                                self.add_terminal_request(
                                    client,
                                    pane_id,
                                    TerminalRequestKind::Clipboard {
                                        bel: !string_terminator,
                                    },
                                );
                            }
                            continue;
                        }
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

    /// Put each pane's OSC 4 questions about palette entries it does not hold
    /// to an attached terminal, mirroring tmux's `input_osc_4` falling through
    /// to `input_add_request`.
    pub(crate) fn process_pane_palette_queries(&mut self) {
        let panes = self
            .windows
            .values()
            .flat_map(|window| window.panes.iter())
            .map(|node| (node.id, node.pane.observation_state()))
            .collect::<Vec<_>>();
        for (pane_id, observation) in panes {
            for (index, bel) in observation.take_palette_queries() {
                let Some(client) = self.request_client_for_pane(pane_id) else {
                    continue;
                };
                // tmux asks with a string terminator whatever the pane used;
                // only the answer it writes back mirrors the pane's own.
                self.client_renders.write_client_output_named(
                    &client,
                    format!("\x1b]4;{index};?\x1b\\").as_bytes(),
                );
                self.add_terminal_request(
                    client,
                    pane_id,
                    TerminalRequestKind::Palette { index, bel },
                );
            }
        }
    }

    /// Write `data` to the terminal selection of every client showing this
    /// window, which is the client walk `tty_write` does for any other pane
    /// output.
    fn write_window_selection(&self, window_id: u32, data: Vec<u8>) {
        let sessions = self
            .sessions
            .iter()
            .filter(|session| {
                session
                    .windows
                    .get(session.active)
                    .is_some_and(|link| link.id == window_id)
            })
            .map(|session| session.id)
            .collect::<BTreeSet<_>>();
        for client in self
            .client_snapshots()
            .into_iter()
            .filter(|client| !client.control_mode && sessions.contains(&client.session_id))
        {
            self.set_client_selection(Some(&client.name), None, Some(data.clone()));
        }
    }

    /// The client tmux's `input_add_request` would ask on this pane's behalf:
    /// the most recently active attached client whose session can see the
    /// pane's window.
    fn request_client_for_pane(&self, pane_id: u32) -> Option<String> {
        let window_id = self
            .windows
            .values()
            .find(|window| window.panes.iter().any(|node| node.id == pane_id))
            .map(|window| window.id)?;
        let sessions = self
            .sessions
            .iter()
            .filter(|session| session.windows.iter().any(|link| link.id == window_id))
            .map(|session| session.id)
            .collect::<BTreeSet<_>>();
        self.client_snapshots()
            .into_iter()
            .filter(|client| !client.control_mode && sessions.contains(&client.session_id))
            .max_by_key(|client| client.activity_micros)
            .map(|client| client.name)
    }

    fn add_terminal_request(&mut self, client: String, pane_id: u32, kind: TerminalRequestKind) {
        self.terminal_requests.push(TerminalRequest {
            client,
            pane_id,
            kind,
            at: Instant::now(),
        });
    }

    /// Whether anything is still owed an answer from this client's terminal,
    /// which is what makes its input worth scanning for one.
    pub(crate) fn client_awaits_terminal_reply(&self, client: &str) -> bool {
        self.terminal_requests
            .iter()
            .any(|request| request.client == client)
            || self.client_renders.clipboard_query_outstanding(client)
    }

    /// tmux's `input_request_timer_callback`: a question the terminal never
    /// answered stops being owed an answer.
    pub(crate) fn expire_terminal_requests(&mut self) {
        /// tmux's `INPUT_REQUEST_TIMEOUT`.
        const TIMEOUT: Duration = Duration::from_millis(500);
        let now = Instant::now();
        self.terminal_requests
            .retain(|request| now.saturating_duration_since(request.at) < TIMEOUT);
    }

    /// Route what an attached terminal answered, mirroring tmux's
    /// `input_request_reply`.
    ///
    /// The answer goes to the pane that asked, not to the client's active pane:
    /// the request records which one that was. A clipboard answer additionally
    /// becomes a paste buffer when `get-clipboard both` asked for one, or when
    /// the question came from `refresh-client -l` rather than from a pane.
    pub(crate) fn deliver_terminal_reply(&mut self, client: &str, reply: TerminalReply) {
        if let TerminalReply::Clipboard { data, .. } = &reply {
            // tmux's `TTY_OSC52QUERY`: `refresh-client -l` keeps its own
            // outstanding-query flag, and the answer both fills a buffer and
            // releases the flag so the next `-l` can ask again.
            if self.client_renders.take_clipboard_query(client) && !data.is_empty() {
                let data = data.clone();
                self.set_buffer(None, &data);
            }
        }
        let matching = self.terminal_requests.iter().position(|request| {
            request.client == client
                && match (&request.kind, &reply) {
                    (
                        TerminalRequestKind::Palette { index, .. },
                        TerminalReply::Palette { index: replied, .. },
                    ) => index == replied,
                    (TerminalRequestKind::Clipboard { .. }, TerminalReply::Clipboard { .. }) => {
                        true
                    }
                    _ => false,
                }
        });
        let Some(matching) = matching else {
            return;
        };
        let request = self.terminal_requests.remove(matching);
        match (request.kind, reply) {
            (
                TerminalRequestKind::Palette { index, bel },
                TerminalReply::Palette { colour, .. },
            ) => {
                let (r, g, b) = ((colour >> 16) as u8, (colour >> 8) as u8, colour as u8);
                let terminator = if bel { "\x07" } else { "\x1b\\" };
                let answer = format!(
                    "\x1b]4;{index};rgb:{r:02x}{r:02x}/{g:02x}{g:02x}/{b:02x}{b:02x}{terminator}"
                );
                let _ = self.write_pane_input(request.pane_id, answer.as_bytes());
            }
            (
                TerminalRequestKind::Clipboard { bel },
                TerminalReply::Clipboard { selection, data },
            ) => {
                // tmux re-reads `get-clipboard` when the answer lands: `both`
                // keeps a copy for itself as well as answering the pane.
                match self
                    .server_options()
                    .get("get-clipboard")
                    .unwrap_or("buffer")
                {
                    "both" => self.set_buffer(None, &data),
                    "request" => {}
                    // `off` and `buffer` never asked the terminal anything.
                    _ => return,
                }
                let selection = selection.map(char::from).unwrap_or('c');
                let mut answer = format!(
                    "\x1b]52;{selection};{}",
                    super::cmd_send_keys::base64_encode(&data)
                )
                .into_bytes();
                answer.extend_from_slice(if bel { b"\x07" } else { b"\x1b\\" });
                let _ = self.write_pane_input(request.pane_id, &answer);
            }
            _ => {}
        }
    }

    /// Apply the `allow-rename` policy to the `ESC k … ST` renames panes
    /// emitted since the last pass, mirroring tmux's `input_exit_rename`.
    ///
    /// The option decides a rename once, when it is drained, and a refused
    /// rename is dropped: turning `allow-rename` on later does not replay a
    /// name the pane sent while it was off. An empty payload is tmux's request
    /// to fall back to the automatic name, which hmux resolves from the pane's
    /// command whenever `automatic-rename` is on, so there is nothing to store.
    pub(crate) fn process_pane_renames(&mut self) {
        let mut renamed = Vec::new();
        for window in self.windows.values() {
            for node in &window.panes {
                let allowed = node
                    .options(window, &self.global_options)
                    .get("allow-rename")
                    .is_some_and(|value| value == "on" || value == "1");
                // Drained either way: a refused rename must not outlive the
                // option that refused it.
                let queued = node.pane.observation_state().take_renames();
                let Some(name) = queued.into_iter().next_back() else {
                    continue;
                };
                if allowed && !name.is_empty() {
                    renamed.push((window.id, name));
                }
            }
        }
        for (window_id, name) in renamed {
            // tmux also clears `automatic-rename` here; hmux does not, which is
            // the difference gap07's rename test records.
            let _ = self.set_window_name(&format!("@{window_id}"), &name, false);
        }
    }

    /// tmux's `check_window_name`, run once per server loop: re-derive the name
    /// of every `automatic-rename` window whose active pane has changed since
    /// the last pass, and store it.
    ///
    /// The name is stored rather than resolved whenever a format asks for it,
    /// because that is what makes it lag: a pane that execs a different command
    /// without printing anything keeps the name it already had, since nothing
    /// tells the server to look again. One re-derivation per `NAME_INTERVAL`
    /// keeps a chatty pane from walking `/proc` on every pass.
    pub(crate) fn process_window_names(&mut self) {
        /// tmux's `NAME_INTERVAL`.
        const NAME_INTERVAL_MICROS: i64 = 500_000;
        let now = now_micros();
        let mut pending = Vec::new();
        for window in self.windows.values() {
            let options = window.options(&self.global_options);
            if !options
                .get("automatic-rename")
                .is_some_and(|value| value == "on" || value == "1")
            {
                continue;
            }
            let Some(node) = window.panes.get(window.active) else {
                continue;
            };
            let observation = node.pane.observation_state();
            let in_mode = node.mode.is_some();
            if (!observation.changed() && in_mode == window.name_in_mode)
                || now.saturating_sub(window.name_time_micros) < NAME_INTERVAL_MICROS
            {
                continue;
            }
            pending.push((
                window.id,
                observation,
                options
                    .get("automatic-rename-format")
                    .unwrap_or("#{pane_current_command}")
                    .to_string(),
                node.pane.process_probe(),
                in_mode,
                window.name.clone(),
            ));
        }
        for (window_id, observation, source, probe, in_mode, current) in pending {
            observation.clear_changed();
            if let Some(window) = self.windows.get_mut(&window_id) {
                window.name_time_micros = now;
                window.name_in_mode = in_mode;
            }
            let mut vars = super::format::Vars::new();
            vars.set(
                "pane_current_command",
                probe
                    .as_ref()
                    .and_then(super::pane::PaneProcessProbe::current_command)
                    .unwrap_or_default(),
            )
            .set("pane_in_mode", if in_mode { "1" } else { "0" })
            .set("pane_dead", "0");
            let name = super::format::expand(&source, &vars);
            // An empty name would blank the window; tmux's `format_window_name`
            // cannot produce one, and hmux keeps the name it has instead.
            if name.is_empty() || name == current {
                continue;
            }
            let _ = self.rename_window_automatically(&format!("@{window_id}"), &name);
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
            _ => (
                "Silence",
                "silence-action",
                "visual-silence",
                "alert-silence",
            ),
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
        self.recalculate_sizes()?;
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
                self.recalculate_sizes()?;
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
        self.split_window_direction_with_spawn(
            target,
            select,
            before,
            direction,
            &[shell],
            None,
            None,
        )
    }

    pub(crate) fn split_window_direction_with_spawn(
        &mut self,
        target: &str,
        select: bool,
        before: bool,
        direction: SplitDirection,
        argv: &[String],
        cwd: Option<&Path>,
        new_size: Option<u16>,
    ) -> io::Result<usize> {
        let spec = match cwd {
            Some(cwd) => PaneSpec::CommandIn(argv.to_vec(), cwd.to_path_buf()),
            None => PaneSpec::Command(argv.to_vec()),
        };
        self.split_window_direction_with_spec(target, select, before, direction, spec, new_size)
    }

    pub(crate) fn split_window_direction_with_spec(
        &mut self,
        target: &str,
        select: bool,
        before: bool,
        direction: SplitDirection,
        spec: PaneSpec,
        new_size: Option<u16>,
    ) -> io::Result<usize> {
        let (_, pane_part) = split_pane_target(target);
        let t = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[t.session].id;
        let window = self.window(t.session, t.window);
        let (cols, rows) = (window.cols, window.rows);
        // The id the pane will take, peeked for its environment; the allocation
        // below consumes it.
        let spec = fill_spec_spawn_ids(spec, self.next_pane_id, session_id);
        let pane = match spec {
            PaneSpec::Inert => Pane::inert(cols, rows)?,
            PaneSpec::Command(argv) => {
                let refs = argv.iter().map(String::as_str).collect::<Vec<_>>();
                Pane::spawn(&refs, None, cols, rows)?
            }
            PaneSpec::CommandIn(argv, cwd) => {
                let refs = argv.iter().map(String::as_str).collect::<Vec<_>>();
                Pane::spawn(&refs, Some(&cwd), cols, rows)?
            }
            // The child is already running and its pty already open: this pane
            // only changes owner.
            PaneSpec::Existing(pane) => *pane,
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
        if !win
            .layout
            .split_sized(target_id, pane_id, direction, before, new_size)
        {
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
                scrollbar_columns: 0,
                border_status: None,
                unseen_changes: false,
                active_point: 0,
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
            win.set_active_pane(insert_at);
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
        let window = self.window(target.session, target.window);
        let (cols, rows) = (window.cols, window.rows);
        let width = width
            .unwrap_or(cols / 2)
            .clamp(1, cols.saturating_sub(1).max(1));
        let height = height
            .unwrap_or(rows / 4)
            .clamp(1, rows.saturating_sub(1).max(1));
        let argv = fill_spawn_ids(argv, self.next_pane_id, session_id);
        let refs = argv.iter().map(String::as_str).collect::<Vec<_>>();
        let pane = Pane::spawn(&refs, cwd, width, height)?;
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
                scrollbar_columns: 0,
                border_status: None,
                unseen_changes: false,
                active_point: 0,
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
            window.set_active_pane(insert_at);
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
            win.set_active_pane(t.pane);
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
        win.set_active_pane(next);
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
        let t = self
            .find_target(target, TargetKind::Pane)
            .map_err(|miss| miss.error)?;
        let id = self.window(t.session, t.window).panes[t.pane].id;
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
        let a = self.resolve_window_target(src)?;
        let b = self.resolve_window_target(dst)?;
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
        let s = self.resolve_window_target(src)?;
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
        let s = self.resolve_window_target(src)?;
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
        let t = self.resolve_window_target(target)?;
        let session_id = self.sessions[t.session].id;
        let win = self.window_mut(t.session, t.window);
        match win.last_pane_index() {
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

    /// `last-pane -d`/`-e` (and `select-pane -l` with those flags): set or
    /// clear the input-off flag on the previously-active pane. tmux toggles
    /// the flag without switching to the pane.
    pub(crate) fn set_last_pane_input_off(
        &mut self,
        target: &str,
        input_off: bool,
    ) -> io::Result<()> {
        let t = self.resolve_window_target(target)?;
        let win = self.window_mut(t.session, t.window);
        match win.last_pane_index() {
            Some(lp) => {
                win.panes[lp].input_off = input_off;
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
        let t = self.resolve_window_target(target)?;
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
        let s = self.resolve_window_target(src)?;
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
        let s = self.resolve_window_target(src)?;
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
                activity_epoch: now_epoch(),
                name_time_micros: 0,
                name_in_mode: false,
                scrollbars_on_left: false,
                cols: source_size.0,
                rows: source_size.1,
                manual_size: source_size,
                latest_client: None,
                pending_size: None,
                layout: LayoutCell::pane(node_id, source_size.0, source_size.1),
                last_layout: None,
                old_layout: None,
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

    /// `resize-window`: pin the target window at a manual size.
    ///
    /// tmux's `cmd_resize_window_exec` — every form starts from the window's
    /// current size, applies `-x`/`-y` then one `-L/-R/-U/-D` adjustment, lets
    /// `-a`/`-A` overwrite the result with the smallest/largest client, and
    /// finally switches the window's own `window-size` option to `manual` so the
    /// pinned size survives later client resizes. A shrink wider than the
    /// current size is a no-op rather than a clamp.
    pub fn resize_window(&mut self, target: &str, request: WindowResizeRequest) -> io::Result<()> {
        let t = self.resolve_window_target(target)?;
        let window_id = self.sessions[t.session].windows[t.window].id;
        let window = self.window(t.session, t.window);
        let mut size = (
            request.cols.unwrap_or(window.cols),
            request.rows.unwrap_or(window.rows),
        );
        let adjust = request.adjustment;
        // A shrink wider than the axis is a no-op, not a clamp to the minimum.
        match request.adjust {
            Some(WindowResizeAdjust::Left) if size.0 >= adjust => size.0 -= adjust,
            Some(WindowResizeAdjust::Right) => size.0 = size.0.saturating_add(adjust),
            Some(WindowResizeAdjust::Up) if size.1 >= adjust => size.1 -= adjust,
            Some(WindowResizeAdjust::Down) => size.1 = size.1.saturating_add(adjust),
            _ => {}
        }
        // `-a`/`-A` ignore everything computed above and take the extreme of the
        // clients that can see the window, falling back to `default-size`.
        if let Some(policy) = request.snap {
            size = self.default_window_size(t.session, Some(window_id), Some(policy));
        }
        self.window_mut(t.session, t.window)
            .option_overrides_mut()
            .set("window-size", "manual");
        self.window_mut(t.session, t.window).manual_size = size;
        self.recalculate_sizes()
    }

    pub(crate) fn resize_linked_window(
        &mut self,
        target: &str,
        cols: u16,
        rows: u16,
    ) -> io::Result<()> {
        let resolved = self.resolve_window_target(target)?;
        let window_id = self.sessions[resolved.session].windows[resolved.window].id;
        let window = self.windows.get_mut(&window_id).expect("window present");
        window.manual_size = (cols, rows);
        window.option_overrides_mut().set("window-size", "manual");
        self.recalculate_sizes()
    }

    /// tmux's `default_window_size`: the size to create a window at, or the size
    /// `resize-window -a`/`-A` snaps to.
    ///
    /// `window` scopes the client set — `Some(id)` counts every client whose
    /// session links that window (the `-a`/`-A` case), `None` counts the
    /// clients of `session` itself (the window-creation case). With no
    /// qualifying client, or under `manual`, the session's `default-size`
    /// decides.
    pub(crate) fn default_window_size(
        &self,
        session: usize,
        window: Option<u32>,
        policy: Option<WindowSizePolicy>,
    ) -> (u16, u16) {
        let policy = policy.unwrap_or_else(|| {
            WindowSizePolicy::parse(self.global_options.window().get("window-size"))
        });
        let session_id = self.sessions[session].id;
        if policy != WindowSizePolicy::Manual {
            let clients = self.sizing_clients();
            let visible = clients.iter().filter(|client| match window {
                Some(id) => self.session_links_window(client.session_id, id),
                None => client.session_id == session_id,
            });
            // A window-less `latest` has no `w->latest` to prefer, so tmux lets
            // every candidate through and folds them as for `smallest`.
            if let Some(size) = fold_client_sizes(visible, policy) {
                return clamp_window_size(size);
            }
        }
        let size = self.sessions[session]
            .options(&self.global_options)
            .get("default-size")
            .and_then(parse_size_pair)
            .unwrap_or((80, 24));
        clamp_window_size(size)
    }

    /// tmux's `recalculate_sizes`: re-derive every window's size from the
    /// clients that can see it.
    ///
    /// Windows are global, so one window has one size however many sessions link
    /// it. Each window's own `window-size` picks the policy and
    /// `aggressive-resize` narrows the client set to those actually showing it.
    /// A window with no qualifying client keeps the size it has — only window
    /// *creation* falls back to `default-size`.
    pub(crate) fn recalculate_sizes(&mut self) -> io::Result<()> {
        self.recalculate_sizes_now(false)
    }

    /// [`ServerState::recalculate_sizes`], with tmux's `now` flag.
    ///
    /// Automatic sizing is deferred for a window no attached session is
    /// currently showing: the new size is recorded and applied by
    /// [`ServerState::flush_pending_window_sizes`] once the window becomes
    /// current, so a background window's panes are not resized under a program
    /// that cannot see it happen. A manually sized window, or `now`, resizes
    /// straight away.
    pub(crate) fn recalculate_sizes_now(&mut self, now: bool) -> io::Result<()> {
        let clients = self.sizing_clients();
        let window_ids = self.windows.keys().copied().collect::<Vec<_>>();
        // tmux's `server_client_update_latest`: a client only ever becomes the
        // latest client of the window it is currently showing.
        for window_id in &window_ids {
            let latest = clients
                .iter()
                .filter(|client| self.session_shows_window(client.session_id, *window_id))
                .max_by_key(|client| client.size_seq)
                .map(|client| client.size_seq);
            if let Some(latest) = latest {
                if let Some(window) = self.windows.get_mut(window_id) {
                    window.latest_client = Some(latest);
                }
            }
        }
        let mut result = Ok(());
        for window_id in window_ids {
            let Some(window) = self.windows.get(&window_id) else {
                continue;
            };
            let options = window.options(&self.global_options);
            let policy = WindowSizePolicy::parse(options.get("window-size"));
            let aggressive = matches!(options.get("aggressive-resize"), Some("on" | "1"));
            let Some(size) = self.calculate_window_size(&window_id, policy, aggressive, &clients)
            else {
                continue;
            };
            if now || policy == WindowSizePolicy::Manual {
                if let Some(window) = self.windows.get_mut(&window_id) {
                    window.pending_size = None;
                }
                if let Err(error) = self.apply_window_size(window_id, size) {
                    result = Err(error);
                }
            } else if let Some(window) = self.windows.get_mut(&window_id) {
                window.pending_size = Some(size);
            }
        }
        if let Err(error) = self.flush_pending_window_sizes() {
            result = Err(error);
        }
        result
    }

    /// tmux's `server_client_check_window_resize`: apply a deferred size once
    /// some attached session is showing the window.
    fn flush_pending_window_sizes(&mut self) -> io::Result<()> {
        let visible = self.client_renders.with_entries(|entries| {
            entries
                .filter_map(|entry| self.current_window_of_session(entry.session_id))
                .collect::<BTreeSet<_>>()
        });
        let mut result = Ok(());
        for window_id in visible {
            let Some(size) = self
                .windows
                .get_mut(&window_id)
                .and_then(|window| window.pending_size.take())
            else {
                continue;
            };
            if let Err(error) = self.apply_window_size(window_id, size) {
                result = Err(error);
            }
        }
        result
    }

    /// tmux's `clients_calculate_size` for one window, or `None` when no client
    /// qualifies and the window should keep its current size.
    fn calculate_window_size(
        &self,
        window_id: &u32,
        policy: WindowSizePolicy,
        aggressive: bool,
        clients: &[SizingClient],
    ) -> Option<(u16, u16)> {
        let window = self.windows.get(window_id)?;
        if policy == WindowSizePolicy::Manual {
            return Some(window.manual_size);
        }
        // `latest` narrows to `w->latest` only once more than one client can see
        // the window; a lone client is folded as for `smallest`.
        let linked = clients
            .iter()
            .filter(|client| self.session_links_window(client.session_id, *window_id))
            .count();
        let candidates = clients.iter().filter(|client| {
            let visible = if aggressive {
                self.session_shows_window(client.session_id, *window_id)
            } else {
                self.session_links_window(client.session_id, *window_id)
            };
            visible
                && !(policy == WindowSizePolicy::Latest
                    && linked > 1
                    && window.latest_client != Some(client.size_seq))
        });
        fold_client_sizes(candidates, policy)
    }

    /// Resize one window and everything laid out inside it.
    fn apply_window_size(&mut self, window_id: u32, size: (u16, u16)) -> io::Result<()> {
        let (cols, rows) = clamp_window_size(size);
        let Some(window) = self.windows.get_mut(&window_id) else {
            return Ok(());
        };
        if window.cols == cols && window.rows == rows {
            return Ok(());
        }
        window.cols = cols;
        window.rows = rows;
        window.layout.resize(cols, rows);
        let result = resize_panes_to_layout(window);
        for session_id in self.sessions_linking_window(window_id) {
            self.invalidate_session(
                session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        }
        self.notify_window("window-layout-changed", window_id);
        self.notify_window("window-resized", window_id);
        result
    }

    /// The clients whose terminal size counts, with their pane area already
    /// measured — tmux's `ignore_client_size` plus `status_line_size`.
    fn sizing_clients(&self) -> Vec<SizingClient> {
        self.client_renders.with_entries(|entries| {
            // An `ignore-size` client counts only while every client is
            // flagged.
            let entries = entries.collect::<Vec<_>>();
            let any_unflagged = entries
                .iter()
                .any(|entry| entry.counts_for_sizing() && !entry.ignore_size);
            entries
                .iter()
                .filter(|entry| entry.counts_for_sizing() && !(entry.ignore_size && any_unflagged))
                .filter_map(|entry| {
                    let session = self
                        .sessions
                        .iter()
                        .find(|session| session.id == entry.session_id)?;
                    // A control client has no status line to pay for, and tmux
                    // turns the status line off rather than shrink a window to
                    // nothing on a terminal too short to hold both
                    // (`CLIENT_STATUSOFF`).
                    let status = if entry.control_mode {
                        0
                    } else {
                        self.status_lines(session)
                    };
                    let rows = if entry.rows > status {
                        entry.rows - status
                    } else {
                        entry.rows
                    };
                    Some(SizingClient {
                        session_id: entry.session_id,
                        cols: entry.cols,
                        rows,
                        size_seq: entry.size_seq,
                    })
                })
                .collect()
        })
    }

    /// The rows a session's status line occupies — tmux's `status_line_size`.
    fn status_lines(&self, session: &Session) -> u16 {
        match session.options(&self.global_options).get("status") {
            Some("off" | "0") => 0,
            Some("2") => 2,
            Some("3") => 3,
            Some("4") => 4,
            Some("5") => 5,
            _ => 1,
        }
    }

    /// Whether a session links the window at all — tmux's `session_has`.
    fn session_links_window(&self, session_id: u32, window_id: u32) -> bool {
        self.sessions
            .iter()
            .find(|session| session.id == session_id)
            .is_some_and(|session| session.windows.iter().any(|link| link.id == window_id))
    }

    /// Whether the window is the one a session is currently showing.
    fn session_shows_window(&self, session_id: u32, window_id: u32) -> bool {
        self.current_window_of_session(session_id) == Some(window_id)
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

    /// Move or resize a floating pane by the border the pointer grabbed —
    /// tmux's `cmd_resize_pane_mouse_update_floating`.
    ///
    /// `grabbed` is where the button went down, which names the border, and
    /// `now` is where the pointer has reached. The top border moves the pane
    /// whole; every other one resizes it from the edge that stayed put.
    pub(crate) fn drag_floating_pane(
        &mut self,
        target: &str,
        grabbed: (u16, u16),
        now: (u16, u16),
    ) -> bool {
        const MINIMUM: i32 = 1;
        let Some(resolved) = self.resolve(target) else {
            return false;
        };
        let window = self.window_mut(resolved.session, resolved.window);
        let Some(rect) = window.panes[resolved.pane].floating else {
            return false;
        };
        let (sx, sy) = (i32::from(rect.width), i32::from(rect.height));
        let (xoff, yoff) = (i32::from(rect.left), i32::from(rect.top));
        let (left, right) = (xoff - 1, xoff + sx);
        let (lx, ly) = (i32::from(grabbed.0), i32::from(grabbed.1));
        let (x, y) = (i32::from(now.0), i32::from(now.1));

        let on_left = lx == left || lx == left + 1;
        let on_right = lx == right || lx == right + 1;
        let (mut width, mut height) = (sx, sy);
        let (mut new_left, mut new_top) = (xoff, yoff);
        if ly == yoff - 1 && on_left {
            width = (sx + (lx - x)).max(MINIMUM);
            height = (sy + (ly - y)).max(MINIMUM);
            new_left = x + 1;
            new_top = y + 1;
        } else if ly == yoff - 1 && on_right {
            width = (x - xoff).max(MINIMUM);
            height = (sy + (ly - y)).max(MINIMUM);
            new_top = y + 1;
        } else if ly == yoff + sy && on_left {
            width = (sx + (lx - x)).max(MINIMUM);
            height = y - yoff;
            if height < MINIMUM {
                return false;
            }
            new_left = x + 1;
        } else if ly == yoff + sy && on_right {
            width = (x - xoff).max(MINIMUM);
            height = (y - yoff).max(MINIMUM);
        } else if lx == right {
            width = x - xoff;
            if width < MINIMUM {
                return false;
            }
        } else if lx == left {
            width = sx + (lx - x);
            if width < MINIMUM {
                return false;
            }
            new_left = x + 1;
        } else if ly == yoff + sy {
            height = y - yoff;
            if height < MINIMUM {
                return false;
            }
        } else if ly == yoff - 1 {
            // The top border moves the pane instead of resizing it.
            new_left = xoff + (x - lx);
            new_top = y + 1;
        } else {
            return false;
        }

        let node = &mut window.panes[resolved.pane];
        node.floating = Some(PaneRect {
            left: new_left.max(0) as u16,
            top: new_top.max(0) as u16,
            width: width.max(MINIMUM) as u16,
            height: height.max(MINIMUM) as u16,
        });
        let _ = resize_panes_to_layout(window);
        let session_id = self.sessions[resolved.session].id;
        self.invalidate_session(session_id, RenderInvalidation::LAYOUT);
        true
    }

    /// The scrollbar the target's pane shows: the columns it occupies, whether
    /// they sit to the left of the pane, and the slider's row range within the
    /// pane — tmux's `screen_redraw_draw_pane_scrollbar`.
    pub(crate) fn active_pane_scrollbar(&self, target: &str) -> Option<PaneScrollbar> {
        let resolved = self.resolve(target)?;
        let window = self.window(resolved.session, resolved.window);
        let node = &window.panes[resolved.pane];
        if node.scrollbar_columns == 0 {
            return None;
        }
        let rect = window.pane_rect(node.id)?;
        if rect.height == 0 {
            return None;
        }
        let (slider_top, slider_height) = pane_slider(node, rect.height);
        Some(PaneScrollbar {
            columns: node.scrollbar_columns,
            on_left: window.scrollbars_on_left,
            left: if window.scrollbars_on_left {
                rect.left.saturating_sub(node.scrollbar_columns)
            } else {
                rect.left + rect.width
            },
            top: rect.top,
            height: rect.height,
            slider_top,
            slider_height,
        })
    }

    /// Which side `pane-border-status` puts its row on for the target's pane,
    /// when that pane is the one that gave up a row for it.
    pub(crate) fn active_pane_border_status(&self, target: &str) -> Option<PaneBorderStatus> {
        let resolved = self.resolve(target)?;
        self.window(resolved.session, resolved.window).panes[resolved.pane].border_status
    }

    /// The size of the window a target is showing, and the character
    /// `fill-character` asks for the client area outside it — tmux's
    /// `w->fill_character`, which it paints over every `CELL_OUTSIDE` cell.
    /// `None` for an unset, empty, or wider-than-one-column value, all of
    /// which `window_set_fill_character` refuses.
    pub(crate) fn window_fill(&self, target: &str) -> Option<((u16, u16), String)> {
        let resolved = self.resolve(target)?;
        let window = self.window(resolved.session, resolved.window);
        let fill = window.options(&self.global_options).get("fill-character")?;
        (super::format::display_width(fill) == 1)
            .then(|| ((window.cols, window.rows), fill.to_owned()))
    }

    pub(crate) fn client_window_offset(&self, client: &ViewportClient) -> Option<ClientViewport> {
        let window_id = self.current_window_of_session(client.session_id)?;
        let window = self.windows.get(&window_id)?;
        let session = self
            .sessions
            .iter()
            .find(|session| session.id == client.session_id)?;
        let status = if client.control_mode {
            0
        } else {
            self.status_lines(session)
        };
        let (view_cols, view_rows) = (client.cols, client.rows.saturating_sub(status));
        if view_cols >= window.cols && view_rows >= window.rows {
            return Some(ClientViewport {
                bigger: false,
                ox: 0,
                oy: 0,
                sx: window.cols,
                sy: window.rows,
            });
        }
        let (sx, sy) = (view_cols, view_rows);
        // A pan survives only while it belongs to the window on screen.
        if client.pan_window == Some(window_id) {
            return Some(ClientViewport {
                bigger: true,
                ox: if sx >= window.cols {
                    0
                } else {
                    client.pan_ox.min(window.cols - sx)
                },
                oy: if sy >= window.rows {
                    0
                } else {
                    client.pan_oy.min(window.rows - sy)
                },
                sx,
                sy,
            });
        }
        // Otherwise centre the viewport on the active pane's cursor, as tmux
        // does for a window the client has never panned.
        let (mut ox, mut oy) = (0, 0);
        if let Some(pane) = window.panes.get(window.active) {
            if let (Some(rect), Ok((cursor_x, cursor_y))) =
                (window.pane_rect(pane.id), pane.pane.cursor_position())
            {
                let cx = rect.left.saturating_add(cursor_x);
                let cy = rect.top.saturating_add(cursor_y);
                ox = if cx < sx {
                    0
                } else if cx > window.cols.saturating_sub(sx) {
                    window.cols.saturating_sub(sx)
                } else {
                    cx - sx / 2
                };
                oy = if cy < sy {
                    0
                } else if cy > window.rows.saturating_sub(sy) {
                    window.rows.saturating_sub(sy)
                } else {
                    cy.saturating_sub(sy) + 1
                };
            }
        }
        Some(ClientViewport {
            bigger: true,
            ox,
            oy,
            sx,
            sy,
        })
    }

    /// The viewport of the client with this name, for the render path — which
    /// knows the client it is painting for by name, not by handle.
    pub(crate) fn client_viewport(&self, client_name: &str) -> Option<ClientViewport> {
        let client = self.client_renders.viewport_client(client_name)?;
        self.client_window_offset(&client)
    }

    /// `refresh-client -L/-R/-U/-D`, and `-c` to drop the pan.
    ///
    /// tmux seeds a new pan from the offset the client is already showing, so
    /// panning away from a cursor-following viewport continues from where it
    /// was rather than jumping to the window origin.
    pub(crate) fn pan_client(
        &mut self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        adjust: Option<WindowResizeAdjust>,
        adjustment: u16,
    ) -> ClientActionResult {
        let selected = target
            .and_then(|name| {
                let client = self.client_renders.viewport_client(name)?;
                Some((name.to_string(), client))
            })
            .or_else(|| {
                let tty = invoking_tty?;
                let client = self.client_renders.viewport_client(tty)?;
                Some((tty.to_string(), client))
            });
        let Some((name, client)) = selected else {
            return if target.is_some() {
                ClientActionResult::TargetNotFound
            } else {
                ClientActionResult::NoCurrentClient
            };
        };
        let Some(adjust) = adjust else {
            self.client_renders.set_client_pan(&name, None);
            self.invalidate_session(client.session_id, RenderInvalidation::LAYOUT);
            return ClientActionResult::Queued;
        };
        let Some(window_id) = self.current_window_of_session(client.session_id) else {
            return ClientActionResult::NoCurrentClient;
        };
        let Some(view) = self.client_window_offset(&client) else {
            return ClientActionResult::NoCurrentClient;
        };
        let (mut ox, mut oy) = if client.pan_window == Some(window_id) {
            (client.pan_ox, client.pan_oy)
        } else {
            (view.ox, view.oy)
        };
        let window = self.windows.get(&window_id);
        let (limit_x, limit_y) = window.map_or((0, 0), |window| {
            (
                window.cols.saturating_sub(view.sx),
                window.rows.saturating_sub(view.sy),
            )
        });
        match adjust {
            WindowResizeAdjust::Left => ox = ox.saturating_sub(adjustment),
            WindowResizeAdjust::Right => ox = ox.saturating_add(adjustment).min(limit_x),
            WindowResizeAdjust::Up => oy = oy.saturating_sub(adjustment),
            WindowResizeAdjust::Down => oy = oy.saturating_add(adjustment).min(limit_y),
        }
        self.client_renders
            .set_client_pan(&name, Some((window_id, ox, oy)));
        self.invalidate_session(client.session_id, RenderInvalidation::LAYOUT);
        ClientActionResult::Queued
    }

    fn sessions_linking_window(&self, window_id: u32) -> Vec<u32> {
        self.sessions
            .iter()
            .filter(|session| session.windows.iter().any(|link| link.id == window_id))
            .map(|session| session.id)
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
        let current = self.client_renders.with_entries(|entries| {
            entries
                .map(|entry| {
                    (
                        entry.name.clone(),
                        (entry.session_id, entry.cols, entry.rows),
                    )
                })
                .collect::<BTreeMap<_, _>>()
        });
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

    /// The sessions that currently have at least one client attached.
    fn attached_session_ids(&self) -> BTreeSet<u32> {
        self.client_renders
            .with_entries(|entries| entries.map(|entry| entry.session_id).collect())
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
        let Some(session) = self
            .sessions
            .iter()
            .find(|session| session.id == session_id)
        else {
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
        match self.exit_empty_policy() {
            ExitEmpty::Off => return,
            // An hmux daemon is launched before any client exists, so
            // `after-session` holds the policy back until the server has held
            // a session at least once; tmux never sees this window because its
            // server is forked by the client that creates the first session.
            ExitEmpty::AfterSession if self.initial_attach_pending => return,
            ExitEmpty::AfterSession | ExitEmpty::On => {}
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

    /// Record the viewport a client is showing `session_name` in, then re-derive
    /// every window's size from the clients that can see it.
    ///
    /// `cols`×`rows` is the client's *pane* area (its terminal minus the status
    /// line). tmux has no session size at all — this keeps hmux's, because pane
    /// spawns, popups and overlays are still laid out against it — but the
    /// window sizes it used to set directly now come from
    /// [`ServerState::recalculate_sizes`].
    pub fn resize_session(&mut self, session_name: &str, cols: u16, rows: u16) -> io::Result<()> {
        let session = self.session_index(session_name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {session_name}"),
            )
        })?;
        self.sessions[session].cols = cols;
        self.sessions[session].rows = rows;
        self.recalculate_sizes()
    }

    /// `new-session -x/-y`: pin the session's own `default-size`.
    ///
    /// tmux writes the option onto the new session's option set before creating
    /// it, so the session's first window *and every later one* are created at
    /// that size rather than at the global default. hmux creates the session
    /// first, so the windows already made are resized to match.
    pub(crate) fn set_session_default_size(
        &mut self,
        session_name: &str,
        cols: u16,
        rows: u16,
    ) -> io::Result<()> {
        let Some(session) = self.session_index(session_name) else {
            return Ok(());
        };
        let (cols, rows) = clamp_window_size((cols, rows));
        self.sessions[session]
            .option_overrides_mut()
            .set("default-size", &format!("{cols}x{rows}"));
        self.sessions[session].cols = cols;
        self.sessions[session].rows = rows;
        for window_id in self.sessions[session]
            .windows
            .iter()
            .map(|link| link.id)
            .collect::<Vec<_>>()
        {
            if let Some(window) = self.windows.get_mut(&window_id) {
                window.manual_size = (cols, rows);
            }
            self.apply_window_size(window_id, (cols, rows))?;
        }
        self.recalculate_sizes()
    }

    /// The first `rows` lines of a target's pane, as a mode tree's preview
    /// shows them.
    pub(crate) fn pane_preview_text(&self, target: &str, rows: usize) -> io::Result<String> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let node = &self.window(resolved.session, resolved.window).panes[resolved.pane];
        let text = node.pane.visible_screen()?;
        Ok(text.lines().take(rows).collect::<Vec<_>>().join("\n"))
    }

    /// tmux's `window_pane_search`: the 1-based row of a pane's visible screen
    /// whose trailing-space-trimmed text matches `term`, or 0 when none does.
    pub(crate) fn search_pane_screen(
        &self,
        pane_id: u32,
        term: &str,
        regex: bool,
        ignore_case: bool,
    ) -> u32 {
        let Some(node) = self
            .windows
            .values()
            .flat_map(|window| window.panes.iter())
            .find(|node| node.id == pane_id)
        else {
            return 0;
        };
        let Ok(screen) = node.pane.visible_screen() else {
            return 0;
        };
        let needle = if ignore_case {
            term.to_lowercase()
        } else {
            term.to_owned()
        };
        for (index, line) in screen.lines().enumerate() {
            let line = line.trim_end();
            let line = if ignore_case {
                line.to_lowercase()
            } else {
                line.to_owned()
            };
            // tmux wraps a plain term in `*…*` and fnmatches it; a regular
            // expression is matched unanchored. hmux has no regex engine here,
            // so `r` falls back to the same containment test — which agrees for
            // every pattern that is a literal.
            let _ = regex;
            if line.contains(&needle) {
                return index as u32 + 1;
            }
        }
        0
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
            // tmux's `window_pane_copy_key` fans out only to panes
            // `window_pane_visible` accepts, and a zoomed window shows only its
            // active pane.
            for (index, pane) in window.panes.iter().enumerate() {
                if pane.input_off || (window.zoomed && index != window.active) {
                    continue;
                }
                pane.pane.input(bytes)?;
            }
            Ok(())
        } else {
            window.panes[resolved.pane].pane.input(bytes)
        }
    }

    pub(crate) fn input_mouse_to_pane(&self, target: &str, event: MouseEvent) -> io::Result<()> {
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

    /// Spell one key the way the pane's own terminal type and modes describe
    /// it, per [`super::input_keys`].
    pub(crate) fn encode_pane_key(
        &self,
        target: &str,
        key: PaneKey,
    ) -> io::Result<PaneKeyEncoding> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let state = self.window(resolved.session, resolved.window).panes[resolved.pane]
            .pane
            .key_state();
        Ok(input_keys::encode(
            key,
            self.pane_key_modes(state),
            self.pane_key_options(target),
        ))
    }

    /// Apply the `extended-keys` option to what a pane asked for.
    ///
    /// tmux latches this when the pane's request arrives, so a request made
    /// while the option was `off` stays refused even if the option is turned on
    /// afterwards. Reading the option here instead means a mid-session change
    /// applies to requests the pane already made — the only difference, and one
    /// that needs both an application that asked for extended keys and a user
    /// who changed the option afterwards. See README.md.
    fn pane_key_modes(&self, state: PaneKeyState) -> PaneKeyModes {
        let extended = match self.server_options().get("extended-keys") {
            // `off`: requests are refused outright.
            Some("on") => state.extended_request,
            // `always` keeps a pane in mode 1 even when it asked for nothing,
            // which is what tmux puts a freshly reset screen into.
            Some("always") => match state.extended_request {
                ExtendedKeys::All => ExtendedKeys::All,
                ExtendedKeys::Standard | ExtendedKeys::Off => ExtendedKeys::Standard,
            },
            _ => ExtendedKeys::Off,
        };
        PaneKeyModes {
            cursor_keys: state.cursor_keys,
            application_keypad: state.application_keypad,
            bracketed_paste: state.bracketed_paste,
            extended,
        }
    }

    /// `#{pane_key_mode}`: the name tmux gives the pane's effective
    /// extended-key state, so it follows the same `extended-keys` resolution
    /// the encoder uses rather than the pane's raw request.
    pub(crate) fn pane_key_mode_name(&self, pane: &PaneNode) -> &'static str {
        match self.pane_key_modes(pane.pane.key_state()).extended {
            ExtendedKeys::All => "Ext 2",
            ExtendedKeys::Standard => "Ext 1",
            ExtendedKeys::Off => "VT10x",
        }
    }

    fn pane_key_options(&self, target: &str) -> PaneKeyOptions {
        let options = self.server_options();
        let backspace = match options.get("backspace") {
            // tmux's stored default is the bare `DEL` code; `C-?` is only how
            // it prints that byte back. Reading the printed spelling as a key
            // name would give `'?'` with control instead, which `C-BSpace`
            // then encodes as `DEL` rather than failing the way tmux does.
            // A user who sets `backspace C-?` explicitly is indistinguishable
            // here and gets the default's behaviour. See README.md.
            Some("C-?") | None => PaneKeyOptions::default().backspace,
            Some(name) => parse_key_name(name).unwrap_or(PaneKeyOptions::default().backspace),
        };
        let extended_keys_format = match options.get("extended-keys-format") {
            Some("csi-u") => ExtendedKeysFormat::CsiU,
            _ => ExtendedKeysFormat::Xterm,
        };
        // tmux 3.7b retains this window option in its catalog, but its input
        // key path no longer consults it. Resolve it here so the no-op
        // compatibility behavior is explicit in the runtime path.
        let xterm_keys = self
            .window_options(target)
            .ok()
            .and_then(|options| options.get("xterm-keys"))
            .is_none_or(|value| value != "off");
        PaneKeyOptions {
            backspace,
            extended_keys_format,
            xterm_keys,
        }
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
    ) -> io::Result<Rc<super::pane::NativePaneObservation>> {
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

    pub(crate) fn respawn_pane_process(
        &mut self,
        target: &str,
        argv: Option<Vec<String>>,
        cwd: Option<PathBuf>,
    ) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let (cols, rows, saved, pane_id) = {
            let node = &self.window(resolved.session, resolved.window).panes[resolved.pane];
            let (cols, rows) = node.pane.size();
            (cols, rows, node.pane.spawn_spec(), node.id)
        };
        let spec = match argv {
            // The pane keeps its id across a respawn, so a saved spec already
            // names it and only a rebuilt argv has a placeholder to fill.
            Some(argv) if !argv.is_empty() => PaneSpawnSpec {
                argv: fill_spawn_ids(&argv, pane_id, session_id),
                cwd,
            },
            _ => match saved {
                Some(spec) => spec,
                None => return Ok(()),
            },
        };
        let pane = Pane::spawn_from_spec(&spec, cols, rows)?;
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
            let replacements = self
                .window(resolved.session, resolved.window)
                .panes
                .iter()
                .map(|node| {
                    let Some(spec) = node.pane.spawn_spec() else {
                        return Ok(None);
                    };
                    let (cols, rows) = node.pane.size();
                    Pane::spawn_from_spec(&spec, cols, rows).map(Some)
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
        let session_id = self.sessions[resolved.session].id;
        let pane = {
            let window = self.window(resolved.session, resolved.window);
            let (cols, rows) = (window.cols, window.rows);
            // The surviving pane keeps its id, which its rebuilt environment
            // still has to be told.
            let id = window.panes[resolved.pane].id;
            let spec = PaneSpawnSpec {
                argv: fill_spawn_ids(&argv, id, session_id),
                cwd,
            };
            Pane::spawn_from_spec(&spec, cols, rows)?
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
            scrollbar_columns: 0,
            border_status: None,
            unseen_changes: false,
            active_point: 0,
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
        let shape = pane.cursor_shape();
        if shape != 0 {
            return Ok(shape);
        }
        // tmux's `screen_set_default_cursor`: a pane that has applied no
        // DECSCUSR of its own shows whatever `cursor-style` asks for.
        Ok(cursor_style_parameter(
            self.option_for_target(session_name, "cursor-style")
                .unwrap_or("default"),
        ))
    }

    /// The colour a pane's cursor is drawn in: its own `OSC 12` when it set
    /// one, else the `cursor-colour` option — tmux's `s->default_ccolour`.
    pub(crate) fn active_pane_cursor_colour(&self, session_name: &str) -> Option<String> {
        let pane = self.active_pane(session_name)?;
        let own = pane.osc_state().cursor_colour;
        if !own.is_empty() && own != "none" {
            return Some(own);
        }
        self.option_for_target(session_name, "cursor-colour")
            .filter(|value| !value.is_empty() && *value != "none")
            .map(str::to_owned)
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
