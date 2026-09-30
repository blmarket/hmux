# Trait interfaces for shared models (draft)

Keep the existing `session`, `window`, `window_pane`, and `client` allocations.
Implement `Session`, `Window`, `WindowPane`, and `Client` on their existing
`Rc<UnsafeCell<T>>` holders. Reuse existing component types and explicit lifecycle
functions. Prefer existing types; introduce an owned result or stable identity
when a real caller must retain information across a callback.

## Boundary and method selection

Other entities access these four models through their traits. A trait method may
return an existing holder, but the consumer must then use that holder's trait;
it may not project the underlying model. Implementation helpers may access their
own entity only. Window and pane code sharing a source file does not grant them
access to each other's fields.

The interface should expose the decisions and operations an external consumer
needs. Keep the bookkeeping that implements an operation inside its owner.
A short trait returning every internal object would not achieve this boundary.
Conversely, a getter for every field duplicates the representation in the API.

| Entity | External capabilities | State and work kept inside the entity |
| --- | --- | --- |
| Session | Identity, window navigation and membership, attachment/status decisions, configuration, group operations, lifetime | Session index keys, current/last-used link updates, group synchronization, activity/lock timer updates, attached counts, status cache |
| Window | Pane navigation and membership, layout/zoom, sizing, focus, redraw requests, configuration, lifetime | Layout trees, saved layout/zoom, pane ordering/history, pending resize fields, alerts and scene invalidation |
| WindowPane | Geometry, input and modes, screen copying, output consumption, configuration, lifetime | Input parser, screen selection, mode entries, stream offsets, pipe state, resize/sync queues, scrollbar bookkeeping |
| Client | Attachment, focus/size decisions, input dispatch, output/redraw, UI installation, lifetime | TTY, status/prompt state, control state, attachment history, redraw completion, timers and transport cleanup |

Admit a method when an actual external consumer needs it. Combine related writes
when they must preserve one invariant. Move an existing algorithm into the
entity's implementation when it manages that entity's state; keep cross-entity
orchestration explicit through the other traits. Do not move unrelated code just
to evade the boundary.

Three reviews found that identity and selection alone were insufficient:
`resize.rs` changes session accounting, `spawn.rs` edits session links,
`server_client.rs` consumes window resize and pane output state, and
`screen_redraw.rs` traverses mode-dependent screens. The solution is to expose
those operations, rather than their intermediate fields.

## Reuse existing types

| Previous placeholder | Replacement |
| --- | --- |
| `SessionMetadata`, `WindowMetadata`, `PaneMetadata`, `ClientMetadata` | Lazy formatting through existing `format_tree` and `FormatValue`; explicit queries only for operational decisions |
| Geometry and size structures | `(u32, u32)` for cells or pixels; `(u32, u32, i32, i32)` for pane size and signed offsets |
| Option snapshots/edits | Existing `options` and its APIs at a scoped component boundary |
| Environment snapshots/edits | Existing `environ`, which already implements `Clone` |
| Terminal-control snapshot | Existing `termios`, which is already `Copy` |
| Spawn request | Existing `spawn_context` |
| Pane input/output requests | Existing `key_code`, `mouse_event`, `key_event`, `window_pane_offset`, byte slices and buffers |
| Screen/render snapshots | Existing caller-owned `screen`; entity operations for rendering |
| Mode/prompt/overlay requests | Existing descriptors, argument types, and callback aliases, adjusting callbacks that expose a core model |

No auxiliary types are declared by the original sketch below. `FormatValue` already exists in
[format/callbacks.rs](../src/format/callbacks.rs), with `String(CString)` and
`Time(time_t)` variants. It is now public and re-exported by `format`.

Existing types do not automatically satisfy the boundary. `spawn_context` now
keeps a numeric layout-cell reservation instead of the original pointer. The
caller resolves it through its original Window under a bounded component guard.
Legacy helpers still require review for projections into related models.

## Borrow and callback contract

Callbacks retain an Rc or Weak identity, then borrow through the trait only when
they execute. Copy values or retain independent owners before invoking another
model operation, formatting, resizing, or dispatching a notification. Release
every model/component loan before those calls, and borrow again afterward if
needed. Preserve whether the old code captured its target before the callback or
looked up a fresh target afterward; these are observably different operations.

Component APIs use associated guard types where callers need ordinary references.
The current implementations return references; a future implementation can map
Ref/RefMut from one RefCell containing the entire model. A layout cell's raw
parent/child pointers may be used only during that tree loan or while the tree
is independently owned. They must not be stored in another model or callback.
Options inheritance uses owning-scope identities and returns owned values across
format/parser calls. Neither approach requires a RefCell for each field.

Retaining allocation memory does not replace logical ownership. Existing explicit
release/free operations and their notification order remain required. A temporary
scope upgrade that cannot dispatch callbacks is distinct from a transferred
logical owner; in particular it must not enqueue a deferred Session release.

## Trait sketches

These signatures describe the current adapters around observed consumers, not
a completed migration or a proof of the smallest possible interface. Imports
and implementation bodies are omitted. Methods using legacy shared access remain
`unsafe`; an Rc keeps the allocation alive but does not establish exclusive
access. Logical-lifecycle and callback preconditions remain method-specific.
The signatures below record the original adapter checkpoint. For current APIs see
[Session](../src/session/api.rs), [Window](../src/window/api.rs),
[WindowPane](../src/window_pane/api.rs) and [Client](../src/server_client/api.rs).
Implementation and complete encapsulation are separate:
several adapters still delegate to legacy helpers whose cross-entity projections
must migrate before core fields can become private.

```rust,ignore
pub trait Session {
    unsafe fn id(&self) -> u32;
    unsafe fn name(&self) -> CString;
    unsafe fn rename(&self, name: &CStr) -> Result<(), CString>;
    unsafe fn activity_time(&self) -> std::time::SystemTime;
    unsafe fn is_attached(&self) -> bool;
    unsafe fn current_winlink(&self) -> refbox::Weak<winlink>;
    /// Observe MRU history without selecting it (notably command target `!`).
    unsafe fn last_winlink(&self) -> refbox::Weak<winlink>;
    /// Read only. Do not mutate entries, reenter, or let references/pointers escape.
    unsafe fn with_winlinks<R>(&self, read: impl FnOnce(&winlinks) -> R) -> R;
    unsafe fn select_winlink(&self, link: refbox::Weak<winlink>) -> i32;
    unsafe fn detach_window(&self, link: refbox::Weak<winlink>) -> i32;
    /// Adopt an existing window, including break-pane's already-running pane.
    unsafe fn attach_window(
        &self,
        window: &Rc<UnsafeCell<window>>,
        index: i32,
    ) -> Result<refbox::Weak<winlink>, CString>;
    unsafe fn link_window(
        &self,
        source: &Rc<UnsafeCell<session>>,
        link: refbox::Weak<winlink>,
        index: i32,
        replace: bool,
        select: bool,
    ) -> Result<(), CString>;
    unsafe fn spawn_window(
        &self,
        context: &mut spawn_context,
    ) -> Result<refbox::Weak<winlink>, CString>;
    unsafe fn renumber_windows(&self);
    unsafe fn update_activity(&self, from: Option<std::time::SystemTime>);
    unsafe fn on_attached(&self);
    unsafe fn status_layout(&self) -> (i32, u32);
    unsafe fn join_group(&self, name: &CStr);
    /// Scopes cannot reenter model code, run callbacks, change component parents,
    /// free components, or allow references/pointers to escape.
    unsafe fn with_options_mut<R>(&self, edit: impl FnOnce(&mut options) -> R) -> R;
    unsafe fn with_environment_mut<R>(&self, edit: impl FnOnce(&mut environ) -> R) -> R;
    unsafe fn cwd(&self) -> Option<CString>;
    unsafe fn termios(&self) -> Option<termios>;
    /// Evaluate a session builtin in a context already targeting this holder.
    unsafe fn format_value(
        &self,
        key: &CStr,
        context: &mut crate::src::shared::format::format_tree,
    ) -> Option<crate::src::format::FormatValue>;
    unsafe fn destroy(&self, notify: bool, from: &CStr);
}

pub trait Window {
    unsafe fn id(&self) -> u32;
    unsafe fn name(&self) -> CString;
    unsafe fn rename(&self, name: &CStr, untrusted: bool);
    unsafe fn active_pane(&self) -> Option<Rc<UnsafeCell<window_pane>>>;
    /// `None` starts traversal; `Some` continues after that pane, without wrapping.
    unsafe fn next_pane(
        &self,
        after: Option<&Rc<UnsafeCell<window_pane>>>,
    ) -> Option<Rc<UnsafeCell<window_pane>>>;
    unsafe fn select_pane(&self, pane: &Rc<UnsafeCell<window_pane>>, notify: bool) -> i32;
    unsafe fn remove_pane(&self, pane: &Rc<UnsafeCell<window_pane>>);
    /// Allocate the tiled/floating layout internally, then spawn into it. The
    /// context supplies the command, source pane, session/link, and spawn flags.
    /// Its legacy `lc` field must be null on entry and is null on return.
    /// Report errors before restoring zoom, preserving command/control event
    /// ordering. The callback may reenter; no component borrow spans the call.
    unsafe fn split_pane(
        &self,
        context: &mut spawn_context,
        arguments: &mut args,
        lines: pane_lines,
        restore_zoom: bool,
        report_error: impl FnOnce(&CStr),
    ) -> Result<Rc<UnsafeCell<window_pane>>, CString>;
    unsafe fn size(&self) -> (u32, u32);
    /// Pixel dimensions of a terminal cell, for the pane's PTY resize protocol.
    unsafe fn cell_size(&self) -> (u32, u32);
    unsafe fn is_zoomed(&self) -> bool;
    unsafe fn resize(&self, sx: u32, sy: u32, xpixel: i32, ypixel: i32);
    /// Preserve command precedence: cycle, spread, then named/saved layout.
    /// `cycle` is -1 (previous), 0, or 1 (next). `legacy_format` preserves the
    /// attached control client's old custom-layout serialization format.
    unsafe fn select_layout(
        &self,
        name: Option<&CStr>,
        restore_previous: bool,
        cycle: i32,
        spread: Option<&Rc<UnsafeCell<window_pane>>>,
        legacy_format: bool,
    ) -> Result<(), CString>;
    unsafe fn zoom(&self, pane: &Rc<UnsafeCell<window_pane>>) -> i32;
    unsafe fn unzoom(&self, notify: bool) -> i32;
    unsafe fn update_activity(&self);
    /// Return whether the identity changed. Attachment and input dispatch have
    /// different notifications and keep that orchestration in their callers.
    unsafe fn set_latest_client(&self, client: Option<&Rc<UnsafeCell<client>>>) -> bool;
    /// Window-owned modal/menu/selection policy used when a pane's focus changes.
    unsafe fn pane_is_focused(&self, pane: &Rc<UnsafeCell<window_pane>>) -> bool;
    unsafe fn with_options_mut<R>(&self, edit: impl FnOnce(&mut options) -> R) -> R;
    unsafe fn format_value(
        &self,
        key: &CStr,
        context: &mut crate::src::shared::format::format_tree,
    ) -> Option<crate::src::format::FormatValue>;
    /// Required for owners that may be final: close callbacks execute while an
    /// owner is still live, and may retain it to postpone destruction.
    unsafe fn release(self, from: &CStr)
    where
        Self: Sized;
}

pub trait WindowPane {
    unsafe fn id(&self) -> u32;
    unsafe fn window_observer(&self) -> Weak<UnsafeCell<window>>;
    unsafe fn geometry(&self) -> (u32, u32, i32, i32);
    /// Whether the pane still has a PTY, including an exited process being drained.
    unsafe fn has_tty(&self) -> bool;
    unsafe fn resize(&self, sx: u32, sy: u32);
    unsafe fn update_focus(&self, focused: bool);
    /// Return 0 for an unplaced pane, -1 for deferred drawing, or 1 after
    /// installing pane offsets. Client applies terminal/status offsets afterward.
    /// With `window_redraw` false, the pane is never changed; a disposable context
    /// can therefore probe placement even when dirty drawing would be deferred.
    unsafe fn prepare_render(&self, context: &mut crate::src::shared::tty::tty_ctx,
        window_redraw: bool) -> i32;
    unsafe fn request_redraw(&self, scrollbar: bool);
    /// Normal selection records activity; fallback selection only marks change.
    unsafe fn on_selected(&self, record_activity: bool);
    unsafe fn key(&self, client: Option<&Rc<UnsafeCell<client>>>,
        link: refbox::Weak<winlink>, key: key_code, mouse: Option<&mut mouse_event>) -> i32;
    unsafe fn paste(&self, key: key_code, bytes: &[u8]);
    unsafe fn set_mode(&self, source: Option<&Rc<UnsafeCell<window_pane>>>,
        mode: &'static window_mode, item: Option<&Rc<UnsafeCell<cmdq_item>>>,
        find: Option<&mut cmd_find_state>, arguments: Option<&mut args>) -> i32;
    unsafe fn reset_mode(&self);
    unsafe fn reset_all_modes(&self);
    /// Replace destination with an independent rectangular copy. Coordinates
    /// are relative to the visible screen, excluding history. `displayed`
    /// selects the active mode screen; false selects the process's base screen.
    /// The destination must not alias any pane screen; caller must screen_free it.
    unsafe fn copy_screen(&self, destination: &mut screen, x: u32, y: u32,
        sx: u32, sy: u32, displayed: bool);
    unsafe fn copy_output(&self, offset: &window_pane_offset, destination: &mut [u8]) -> usize;
    /// Start a new output consumer at the parser's current position.
    unsafe fn output_offset(&self) -> window_pane_offset;
    unsafe fn advance_output(&self, offset: &mut window_pane_offset, count: usize);
    /// Run once after all clients have consumed the current event-loop cycle.
    unsafe fn finish_cycle(&self);
    unsafe fn with_options_mut<R>(&self, edit: impl FnOnce(&mut options) -> R) -> R;
    unsafe fn format_value(&self, key: &CStr, context: &mut crate::src::shared::format::format_tree)
        -> Option<crate::src::format::FormatValue>;
    unsafe fn destroy_ready(&self) -> bool;
    unsafe fn destroy(&self);
}

pub trait Client {
    unsafe fn attached_session(&self) -> Weak<UnsafeCell<session>>;
    unsafe fn set_session(&self, session: Option<&Rc<UnsafeCell<session>>>);
    unsafe fn reattach_after_session_destroy(&self, target: Option<&Rc<UnsafeCell<session>>>);
    unsafe fn is_dead(&self) -> bool;
    unsafe fn is_control(&self) -> bool;
    unsafe fn uses_legacy_layout_format(&self) -> bool;
    unsafe fn focuses_window(&self, window: &Rc<UnsafeCell<window>>) -> bool;
    unsafe fn participates_in_window_sizing(&self) -> bool;
    unsafe fn window_size(&self, window: Option<&Rc<UnsafeCell<window>>>) -> (u32, u32, u32, u32);
    unsafe fn constrain_window_size(
        &self,
        window: &Rc<UnsafeCell<window>>,
        sx: &mut u32,
        sy: &mut u32,
    );
    /// `flags` contains only CLIENT_*REDRAW* bits, including status-force.
    unsafe fn request_redraw(&self, flags: u64);
    /// Ordered injection used by send-keys -K. The queue insertion point must
    /// survive until dispatch finishes, under the existing command lifetime rules.
    unsafe fn handle_key_after(
        &self,
        event: Box<key_event>,
        after: Option<&Rc<UnsafeCell<cmdq_item>>>,
        next: Option<&mut Weak<UnsafeCell<cmdq_item>>>,
    ) -> i32;
    unsafe fn print(&self, parse: bool, buffer: &mut SegmentedBuf);
    unsafe fn control_write_output(&self, pane: &Rc<UnsafeCell<window_pane>>);
    /// Scoped access to this control client's consumer offset. Call only for an
    /// attached control client; the bool is the existing output-disabled flag.
    /// Do not reenter models, release owners, or let the reference escape.
    unsafe fn with_output_offset<R>(
        &self,
        pane: u32,
        read: impl FnOnce(Option<&mut window_pane_offset>, bool) -> R,
    ) -> R;
    unsafe fn set_return_value(&self, value: i32);
    unsafe fn request_exit(&self, value: i32);
    /// The closure cannot reenter models, destroy owners, or leak component
    /// references. Clone the environment before invoking another entity.
    unsafe fn with_environment<R>(&self, read: impl FnOnce(Option<&environ>) -> R) -> R;
    unsafe fn cwd(&self, fallback: Option<&Rc<UnsafeCell<session>>>) -> Option<CString>;
    unsafe fn set_key_table(&self, name: Option<&CStr>);
    unsafe fn set_prompt(
        &self,
        find: Option<&cmd_find_state>,
        message: &CStr,
        input: Option<&CStr>,
        inputcb: status_prompt_input_cb,
        freecb: prompt_free_cb,
        flags: i32,
        kind: prompt_type,
    );
    /// Callbacks receive the retained holder, never a live client field borrow.
    /// The legacy overlay implementation continues to own callback retirement.
    unsafe fn set_overlay(
        &self,
        check: overlay_check_cb,
        mode: overlay_mode_cb,
        draw: overlay_draw_cb,
        key: overlay_key_cb,
        free: overlay_free_cb,
        resize: overlay_resize_cb,
        data: Box<dyn Any>,
    );
    unsafe fn clear_overlay(&self);
    /// Read the installed caller-owned payload without leaking references or
    /// calling back into models. Returned owned handles may be used afterwards.
    unsafe fn with_overlay_data<R>(&self, read: impl FnOnce(Option<&dyn Any>) -> R) -> R;
    /// Temporarily remove clipping for an overlay's own output. Restore only if
    /// callbacks did not retire/replace the overlay during `draw`; no model
    /// borrow spans that closure. The supplied callback replaces the old one.
    unsafe fn with_overlay_check_disabled<R>(
        &self,
        restore: overlay_check_cb,
        draw: impl FnOnce() -> R,
    ) -> R;
    unsafe fn terminal_size(&self) -> (u32, u32);
    unsafe fn refresh_terminal_size(&self);
    unsafe fn draw_overlay_screen(
        &self,
        screen: &screen,
        x: u32,
        y: u32,
        sx: u32,
        sy: u32,
        style: &tty_style_ctx,
    );
    /// Prepare direct overlay output, deferring it when a full overlay redraw
    /// is already pending. Coordinates refer to the complete terminal.
    unsafe fn prepare_overlay_render(&self, context: &mut tty_ctx, x: u32, y: u32) -> bool;
    /// Select this client's view of a pane for a terminal command: 0 skips it,
    /// -1 defers to full redraw, and 1 permits immediate output.
    unsafe fn prepare_pane_render(
        &self,
        context: &mut tty_ctx,
        pane: &Rc<UnsafeCell<window_pane>>,
    ) -> i32;
    unsafe fn format_value(
        &self,
        key: &CStr,
        context: &mut format_tree,
    ) -> Option<crate::src::format::FormatValue>;
    unsafe fn detach(&self, message: msgtype);
    unsafe fn suspend(&self);
    unsafe fn lost(&self);
}
```

## Method-by-method necessity

Remove one method while holding the remaining interface, opaque fields,
component contracts and lifecycle rules fixed. Each row names a concrete caller
scenario and the behavior, observation or work bound that would be lost.
Formatting cannot serve as an operational getter backdoor. Existing registry
APIs may resolve or compare opaque holders without projecting model storage.

**Needed** justifies an operation under those constraints, not this exact name
or a proof that no different interface could support it. In particular,
`advance_output` preserves constant-time skipping: an O(bytes) scratch-copy
algorithm could produce the same cursor but would lose that work bound.
`status_layout` preserves cached observation timing. Parent lookup must work
during destruction, after registry/membership removal. Those cases are stronger
than convenience during ordinary steady-state operation.

The original adapter checkpoint contained **102 methods: 24 Session, 21 Window,
23 WindowPane and 34 Client**. Each method has an implementation and a
row below. The original three reviews supplied the initial audit; implementation
removed derivable wrappers and added capabilities required by actual callers.
The remaining migration gaps are recorded separately: a working adapter does not
prove that every external caller has stopped accessing fields.

### Session (24 methods)

| Method | Status | Scenario lost without this method, or alternative | Caller evidence |
| --- | --- | --- | --- |
| `id` | Needed | Save numeric session identity in mouse events or chooser items for later resolution; holder identity and name cannot populate those existing numeric fields. | [Mouse targeting](../src/server_client.rs), [chooser items](../src/window_tree.rs) |
| `name` | Needed | Match session targets by name prefix/pattern or search chooser entries by session name. These operational consumers cannot use a formatting backdoor. | [Target matching](../src/cmd/find.rs), [chooser search](../src/window_tree.rs) |
| `rename` | Needed | `rename-session` must rekey the registry and emit its rename event together; no remaining operation changes the name/index. | [rename-session](../src/cmd/entries/rename_session.rs) |
| `activity_time` | Needed | Choose the most recently active session, including microsecond tie-breaking; the activity methods only write this state. | [Session comparison](../src/cmd/find.rs) |
| `is_attached` | Needed | Prefer an unattached session during target resolution. Scanning `Client::attached_session()` cannot reproduce suspended/exiting-client exclusions through the current Client interface. | [Session comparison](../src/cmd/find.rs), [attachment accounting](../src/resize.rs) |
| `current_winlink` | Needed | Resolve default or relative window targets without changing selection; collection order does not identify the current link. | [Relative targets](../src/cmd/find.rs), [default targets](../src/cmd/find.rs) |
| `last_winlink` | Needed | Resolve target `!` without changing current selection or emitting selection events. `session_last` composes this observation with `select_winlink`; no separate selection primitive is needed. | [Target resolution](../src/cmd/find.rs), [history helper](../src/session/mod.rs) |
| `with_winlinks` | Needed | Resolve indexed/start/end targets and validate membership of noncurrent links; current selection cannot enumerate the collection. | [Window targets](../src/cmd/find.rs), [target validation](../src/cmd/find.rs) |
| `select_winlink` | Needed | Select an arbitrary resolved link while updating history, alerts, focus/activity and the window-change event; link/spawn operations cannot perform this independently. | [attach-session](../src/cmd/entries/attach_session.rs), [selection effects](../src/session/mod.rs) |
| `detach_window` | Needed | Unlink one window with fallback selection, history removal and group synchronization, or report that its last link requires session destruction. | [Unlink orchestration](../src/server_fn.rs), [session detach](../src/session/mod.rs) |
| `attach_window` | Needed | `break-pane` attaches a new window containing an already-running pane. `spawn_window` would perform process creation/respawn work, and `link_window` requires an existing source link. | [break-pane](../src/cmd/entries/break_pane.rs) |
| `link_window` | Needed | Attach an existing window at a destination index with replacement/group restrictions. Spawn creates or respawns; detaching cannot establish membership. | [move-window](../src/cmd/entries/move_window.rs), [link operation](../src/server_fn.rs) |
| `spawn_window` | Needed | Create or respawn a window with index reservation/replacement, failure cleanup and deferred notifications; remaining operations do not offer these creation phases. | [new-window](../src/cmd/entries/new_window.rs), [respawn-window](../src/cmd/entries/respawn_window.rs), [spawn phases](../src/session/spawn.rs) |
| `renumber_windows` | Needed | `move-window -r` compacts indexes while preserving current/last-used/marked mappings. Repeated link/detach calls change notifications, intermediate states and history. | [move-window -r](../src/cmd/entries/move_window.rs), [renumbering](../src/session/mod.rs) |
| `update_activity` | Needed | Record an input event's existing timestamp and refresh the lock timer without attachment/theme/alert effects. | [Input handling](../src/server_client.rs), [activity and lock timer](../src/session/mod.rs) |
| `on_attached` | Needed | Update last-attached time and theme/alert state at the existing position in Client attachment. `update_activity` alone cannot perform those effects. | [Client attachment](../src/server_client.rs) |
| `status_layout` | Needed | Preserve the last published status cache between an option edit and the sizing/cache refresh. Recomputing current options would make Client drawing and size deduction observe the new setting early. | [Cache contract test](../src/session/api.rs), [Client sizing](../src/server_client/api.rs) |
| `join_group` | Needed | Establish and synchronize membership for `new-session -t group`; no remaining Session operation does so. Existing `session_group_add` can implement this boundary, but exposing uncontrolled membership mutation would bypass Session-owned invariants. | [Group creation/join](../src/cmd/entries/new_session.rs), [membership changes](../src/session/mod.rs) |
| `with_options_mut` | Needed | `set-option` changes session options, including array edits and removals; no remaining operation permits these changes. | [Option editing](../src/cmd/entries/set_option.rs), [session option resolution](../src/options.rs) |
| `with_environment_mut` | Needed | Set, unset, clear or hide arbitrary session variables. Selected attach-time merging cannot reproduce arbitrary edits. | [set-environment](../src/cmd/entries/set_environment.rs) |
| `cwd` | Needed | Client path resolution falls back to the supplied or attached session directory. Environment and terminal settings do not expose this saved directory. | [Client cwd](../src/server_client/api.rs) |
| `termios` | Needed | Pane startup copies the session's saved terminal control characters into its terminal settings. No remaining operation reads or applies those saved settings. | [Terminal initialization](../src/spawn.rs) |
| `format_value` | Needed | Format creation/last-attached times and other formatting-only state omitted from typed queries while retaining lazy typed time values. | [Session time callbacks](../src/session/format.rs), [format-key registration](../src/format/callbacks.rs) |
| `destroy` | Needed | `kill-session` removes registry/group membership, links, timers and resources while retained holders may remain. Neither holder release nor single-link detach performs logical teardown. | [kill-session](../src/cmd/entries/kill_session.rs), [destruction](../src/session/mod.rs) |

### Window (21 methods)

| Method | Status | Scenario lost without this method, or alternative | Caller evidence |
| --- | --- | --- | --- |
| `id` | Needed | Control messages and monitors cannot identify this window by its stable `@id`; holder identity does not supply the protocol ID. | [control.rs](../src/control.rs), [monitor.rs](../src/monitor.rs) |
| `name` | Needed | Resolve a window target using exact, prefix, or glob name matching. | [cmd/find.rs](../src/cmd/find.rs) |
| `rename` | Needed | Explicit, automatic, and application-triggered renaming cannot update the stored name and fire rename callbacks. | [rename_window.rs](../src/cmd/entries/rename_window.rs), [names.rs](../src/names.rs), [window.rs](../src/window.rs) |
| `active_pane` | Needed | Resolve an omitted pane target to the window's selection; traversal exposes membership, not selection. | [cmd/find.rs](../src/cmd/find.rs) |
| `next_pane` | Needed | Traverse panes in Window's order for tree previews and server processing; the global pane registry does not expose that private ordering. | [window_tree.rs](../src/window_tree.rs), [server_client.rs](../src/server_client.rs) |
| `select_pane` | Needed | Commit `select-pane` selection, history, focus, and notification changes; zooming has additional effects. | [select_pane.rs](../src/cmd/entries/select_pane.rs), [window.rs](../src/window.rs) |
| `remove_pane` | Needed | Kill or failed-spawn cleanup cannot remove membership and repair active/history state; Pane destruction alone leaves Window collections unchanged. | [kill_pane.rs](../src/cmd/entries/kill_pane.rs), [split_window.rs](../src/cmd/entries/split_window.rs), [window.rs](../src/window.rs) |
| `split_pane` | Needed | `split-window` cannot create and place a child pane while Window owns layout allocation and failed-spawn cleanup; Session spawning creates a window instead. | [split_window.rs](../src/cmd/entries/split_window.rs) |
| `size` | Needed | Sizing code cannot compare candidate dimensions with the current window dimensions; pane geometry does not generally describe the entire window. | [resize.rs](../src/resize.rs) |
| `cell_size` | Needed | Pane PTY resize messages require terminal-cell pixel dimensions; window cell counts and pane geometry do not supply them. | [Pane resize](../src/window.rs) |
| `is_zoomed` | Needed | Pane visibility and zoom-toggle commands must distinguish zoom from ordinary active selection. Geometry does not encode saved-layout state. | [Pane visibility helper](../src/window.rs), [resize-pane](../src/cmd/entries/resize_pane.rs) |
| `resize` | Needed | A terminal-size change cannot apply coordinated layout/window resizing and preserve zoom restoration; Pane resize affects a different owner. | [resize.rs](../src/resize.rs) |
| `select_layout` | Needed | Apply a named/custom layout or restore the previous layout. The signature includes next/previous, spread, saved layout and legacy serialization variants. | [select_layout.rs](../src/cmd/entries/select_layout.rs) |
| `zoom` | Needed | `resize-pane -Z` or a chooser cannot enlarge a pane while saving the previous layout; selection does not establish zoom state. | [resize_pane.rs](../src/cmd/entries/resize_pane.rs), [mode_tree.rs](../src/mode_tree.rs) |
| `unzoom` | Needed | Restore the saved layout without changing active selection; selecting a hidden pane unzooms incidentally but changes selection/history/events. | [resize_pane.rs](../src/cmd/entries/resize_pane.rs), [window.rs](../src/window.rs) |
| `update_activity` | Needed | Pane output or Session selection cannot refresh Window activity and queue activity alerts; Session activity updates a different entity. | [input.rs](../src/input.rs), [session.rs](../src/session/mod.rs), [window.rs](../src/window.rs) |
| `set_latest_client` | Needed | Attachment/input cannot update which client drives `window-size latest`. Its changed-identity result lets attachment and input callers preserve their distinct resize/event ordering. | [server_client.rs](../src/server_client.rs), [server_client.rs](../src/server_client.rs), [resize.rs](../src/resize.rs) |
| `pane_is_focused` | Needed | A pane focus transition needs Window menu and active-selection policy combined with Client focus. `WindowPane::update_focus` consumes that decision; active-pane identity alone cannot determine it. | [Pane focus helper](../src/window.rs), [Window focus policy](../src/window/api.rs) |
| `with_options_mut` | Needed | Set/unset window-local options and user options; the remaining operations do not edit arbitrary option storage. | [options.rs](../src/options.rs), [set_option.rs](../src/cmd/entries/set_option.rs) |
| `format_value` | Needed | Evaluate formats using private Window state, such as saved/visible layout or activity time; the other getters cover only a subset. | [format/callbacks.rs](../src/format/callbacks.rs), [format/callbacks.rs](../src/format/callbacks.rs) |
| `release` | Needed | A potentially final owner cannot fire close callbacks while Weak upgrades still work, permit callback retention, and destroy only if still final; final Rc Drop cannot reproduce that sequence. | [window.rs](../src/window.rs), [window.rs](../src/window.rs) |

### WindowPane (23 methods)

| Method | Status | Scenario lost without this method, or alternative | Caller evidence |
| --- | --- | --- | --- |
| `id` | Needed | Control output and subscription state cannot associate the pane with its stable `%id`. | [control.rs](../src/control.rs), [control.rs](../src/control.rs) |
| `window_observer` | Needed | Mode cleanup needs the parent after Window has left its registry and Pane has left window membership. Scanning registries cannot recover that relationship; the explicit-release cleanup test exercises this retained-parent state. | [Parent cleanup test](../src/window.rs) |
| `geometry` | Needed | Mouse handling cannot reject coordinates outside the pane or translate window coordinates into pane coordinates; copied screen dimensions do not supply offsets. | [cmd/core.rs](../src/cmd/core.rs) |
| `has_tty` | Needed | Control pending output distinguishes a closed PTY from a live idle or still-draining PTY. Copied bytes and cursor position cannot answer whether the process terminal remains open. | [Control pending output](../src/control.rs), [Respawn guard](../src/spawn.rs) |
| `resize` | Needed | Window layout cannot apply dimensions to Pane screens, modes, resize queues, and notifications; calling Window resize would recurse into this missing operation. | [layout/core.rs](../src/layout/core.rs), [window.rs](../src/window.rs) |
| `update_focus` | Needed | Window must send a focus transition to a Pane without accessing its mode flags or process stream. Pane owns the escape sequence, event and focus-bit ordering. | [Window focus helper](../src/window.rs), [Pane focus helper](../src/window.rs) |
| `prepare_render` | Needed | A direct terminal write must reject unplaced panes, defer dirty/drop/full-window redraw cases, and install Pane coordinates. Geometry and redraw requests cannot inspect the private layout/dirty/drop decision. | [Client render preparation](../src/server_client/api.rs) |
| `request_redraw` | Needed | A terminal redraw callback must dirty even an unplaced or already-deferred pane, optionally including its scrollbar. Render preparation may return before setting these flags. | [Screen-write redraw callback](../src/screen_write.rs) |
| `on_selected` | Needed | Window selection changes Pane selection-order bookkeeping; normal selection records global activity order, while fallback selection only marks the change. Focus and redraw do not maintain that ordering. | [Window selection/fallback](../src/window.rs) |
| `key` | Needed | `send-keys` cannot direct a key/mouse event to a chosen pane through its active mode or terminal encoding; Client input dispatch applies client targeting and bindings instead. | [send_keys.rs](../src/cmd/entries/send_keys.rs), [send_keys.rs](../src/cmd/entries/send_keys.rs) |
| `paste` | Needed | Deliver the original paste byte sequence with bracket-paste eligibility and synchronization semantics; encoding individual keys is not equivalent. | [server_client.rs](../src/server_client.rs), [window.rs](../src/window.rs) |
| `set_mode` | Needed | `copy-mode`, chooser modes, and shell-output view mode cannot install/promote mode state and its screen. | [copy_mode.rs](../src/cmd/entries/copy_mode.rs), [choose_tree.rs](../src/cmd/entries/choose_tree.rs), [run_shell.rs](../src/cmd/entries/run_shell.rs) |
| `reset_mode` | Needed | Close the current chooser/copy mode and restore the preceding stacked mode; `reset_all_modes` discards that mode too. | [window_tree.rs](../src/window_tree.rs), [window.rs](../src/window.rs) |
| `reset_all_modes` | Needed | Respawn or `copy-mode -q` cannot clear an arbitrary mode stack. Repeated `reset_mode()` cannot detect completion because it returns no progress and the remaining operational interface has no stack-empty query. | [spawn.rs](../src/spawn.rs), [copy_mode.rs](../src/cmd/entries/copy_mode.rs), [window.rs](../src/window.rs) |
| `copy_screen` | Needed | Renderers need existing terminal/mode cells, while tree previews also need base-screen contents. Process bytes cannot reconstruct those screens; the displayed selector chooses the source without exposing a live Pane screen. | [Rendering](../src/screen_redraw.rs), [tree previews](../src/window_tree.rs) |
| `copy_output` | Needed | Control clients cannot encode unread process-output bytes into `%output`; rendered cells lose information from the original byte stream. | [control.rs](../src/control.rs) |
| `output_offset` | Needed | Initialize or reset a control consumer at the current parser position after the absolute base offset has advanced. Copy and advance operations require an already-valid cursor and cannot construct this starting point. | [Control consumer initialization/reset](../src/control.rs) |
| `advance_output` | Needed | Control skip/reset and queue accounting must advance or clamp an absolute cursor without copying the skipped bytes. Scratch-copy loops are functionally possible but turn the existing constant-time operation into O(bytes) work on the server thread. | [Control cursor accounting](../src/control.rs), [Output cursor tests](../src/window/pane_api.rs) |
| `finish_cycle` | Needed | Run Pane resize processing, shared-buffer drain/read re-enable, and dirty-flag clearing once after all clients. Individual cursor advancement cannot perform this maintenance; Client offsets are accessed through its scoped operation. | [Server loop](../src/server_client.rs), [Pane maintenance](../src/window/pane_api.rs) |
| `with_options_mut` | Needed | `set-option -p` and split-pane customization cannot edit pane-local overrides; Window options affect the wrong scope. | [options.rs](../src/options.rs), [split_window.rs](../src/cmd/entries/split_window.rs) |
| `format_value` | Needed | Formats cannot obtain private process/status/mode information such as `pane_pid` and `pane_dead_status`; existing geometric/input methods do not expose it. | [format/callbacks.rs](../src/format/callbacks.rs), [format/callbacks.rs](../src/format/callbacks.rs) |
| `destroy_ready` | Needed | Pipe completion/error handling cannot decide whether exit status is ready and PTY/pipe data has drained before destruction; `copy_output` does not report pending pipe writes or kernel PTY bytes. | [pipe_pane.rs](../src/cmd/entries/pipe_pane.rs), [window.rs](../src/window.rs) |
| `destroy` | Needed | Window removal/final teardown cannot ask Pane to close streams, cancel timers, free modes/parser state, and unregister while holders remain alive. `Window::remove_pane` needs this subordinate operation and cannot replace it recursively. | [window.rs](../src/window.rs), [window.rs](../src/window.rs), [window.rs](../src/window.rs) |

### Client (34 methods)

| Method | Status | Scenario lost without this method, or alternative | Caller evidence |
| --- | --- | --- | --- |
| `attached_session` | Needed | Resolve an implicit command target or infer format context from the client's attached session. Other methods do not return this relationship. | [Target resolution](../src/cmd/find.rs), [format context](../src/format.rs) |
| `set_session` | Needed | Attach or switch normally, remembering the previous session and firing attachment effects. Destruction reattachment deliberately clears that history. | [attach-session](../src/cmd/entries/attach_session.rs), [switch-client](../src/cmd/entries/switch_client.rs) |
| `reattach_after_session_destroy` | Needed | Clear current/history references before attaching a replacement or requesting exit after session destruction. Two public `set_session` calls add an intermediate focus update, unattached check and socket update absent from the current sequence. | [Session destruction](../src/server_fn.rs), [normal attachment effects](../src/server_client.rs) |
| `is_dead` | Needed | A delayed callback must reject a logically dead client whose allocation remains retained. A successful Weak upgrade does not establish logical validity. | [Menu callback](../src/mode_tree.rs), [file completion](../src/file.rs) |
| `is_control` | Needed | A pane applies its control-supplied colour override only when a control client exists. Attachment and sizing eligibility cannot identify control clients. | [Pane background](../src/window.rs), [pane foreground](../src/window.rs) |
| `uses_legacy_layout_format` | Needed | Layout selection must preserve the attached control client's old serialization format while saving/restoring a custom layout. General control-client identity does not expose this compatibility setting. | [select-layout](../src/cmd/entries/select_layout.rs) |
| `focuses_window` | Needed | Determine application focus while respecting client focus and an overlay suppressing focus. Following the attached session only identifies the selected window. | [Pane focus calculation](../src/window.rs) |
| `participates_in_window_sizing` | Needed | Exclude ineligible clients from automatic sizing while allowing an explicitly selected client to bypass eligibility. Dimensions alone cannot express this policy. | [Eligibility rules](../src/resize.rs), [explicit-client bypass](../src/resize.rs) |
| `window_size` | Needed | Calculate usable cells and pixels with status deduction and control overrides, including default sizing before a window exists. Eligibility and cap application return no such dimensions. | [Dimension calculation](../src/resize.rs), [default sizing](../src/resize.rs) |
| `constrain_window_size` | Needed | Apply the later control-size caps to an already selected or manual size, including width-only or height-only caps. The effective dimensions query loses these independent caps. | [Control cap pass](../src/resize.rs) |
| `request_redraw` | Needed | Explicitly refresh a client after a command or option change without changing its attachment, input or UI installation. Restrict `flags` to redraw bits. | [refresh-client](../src/cmd/entries/refresh_client.rs), [option changes](../src/options.rs) |
| `handle_key_after` | Needed | `send-keys -K` injects keys after a specific command item and updates the next insertion point; ordinary dispatch cannot preserve that queue ordering. | [send-keys](../src/cmd/entries/send_keys.rs) |
| `print` | Needed | Route supplied command output to control output, unattached-client stdout or attached-pane view mode. Pane process-output streaming has different input and framing. | [Command printing](../src/cmd/queue.rs), [output routing](../src/server_client.rs) |
| `control_write_output` | Needed | Forward newly available pane process bytes through each eligible control client's buffering and offsets. `print` instead consumes supplied command text. | [Pane read callback](../src/window.rs) |
| `with_output_offset` | Needed | Pane post-client maintenance must inspect and rebase each control consumer's offset while respecting output suspension. It cannot infer this state from copied process bytes or print commands. The scoped existing component cannot escape or reenter model code. | [Pane buffer maintenance](../src/window/pane_api.rs) |
| `set_return_value` | Needed | Record command or process failure while the client continues running. `request_exit` would add an unwanted exit request. | [source-file completion](../src/cmd/entries/source_file.rs), [process result](../src/window.rs) |
| `request_exit` | Needed | Request failure exit when an input callback discovers its pane disappeared. Setting a return value alone cannot initiate exit; `detach` has different guards and protocol reasons. | [Missing-pane callback](../src/window.rs), [detach semantics](../src/server_client.rs) |
| `with_environment` | Needed | Merge configured client environment variables on attachment, or read an unattached client's `PATH` when spawning. The remaining interface exposes neither source. | [attach-session](../src/cmd/entries/attach_session.rs), [spawn PATH](../src/spawn.rs) |
| `cwd` | Needed | Resolve paths using client launch cwd and existing session/startup fallbacks. Preserve `Option<CString>`: the startup client's cwd can be absent. | [source-file path](../src/cmd/entries/source_file.rs), [cwd resolution](../src/server_client.rs) |
| `set_key_table` | Needed | Reset the client's selected key table after attachment or choose a table independently. Dispatching a configurable key binding cannot reliably substitute. | [Attachment reset](../src/cmd/entries/attach_session.rs), [explicit table selection](../src/cmd/entries/switch_client.rs) |
| `set_prompt` | Needed | Install command-prompt input, callback and completion/history behavior. Overlay installation supplies a different UI mechanism. | [command-prompt](../src/cmd/entries/command_prompt.rs), [prompt installation](../src/status.rs) |
| `set_overlay` | Needed | Display a popup with drawing, input, resize and ownership callbacks. Prompt installation cannot provide this interface. | [Popup installation](../src/popup.rs) |
| `clear_overlay` | Needed | Close a completed popup and restore terminal/focus state while preserving attachment. `set_overlay` with empty callbacks still publishes data, freezes the terminal and hides the cursor. | [Popup completion](../src/popup.rs), [installation behavior](../src/server_client.rs) |
| `with_overlay_data` | Needed | Popup lookup and modification need the installed payload identity. Installation publishes payloads but no other method observes which popup is currently installed; copy the existing popup handle out before reentry. | [Popup lookup](../src/popup.rs) |
| `with_overlay_check_disabled` | Needed | Popup drawing/input parsing must suppress its own clipping and restore it only if callbacks have not replaced the overlay. Clearing the overlay would destroy its state and trigger unrelated terminal/focus effects. | [Popup drawing and parsing](../src/popup.rs), [Replacement regression test](../src/server_client/api.rs) |
| `terminal_size` | Needed | Popup bounds, dragging and resize use the full terminal including status rows. Window sizing deducts status rows, and render preparation can reject a pending redraw before returning dimensions. | [Popup placement/resize](../src/popup.rs) |
| `refresh_terminal_size` | Needed | A popup border change must refresh terminal size from the OS before rebuilding its bounds. A dimension query or redraw request does not perform the terminal-size ioctl. | [Popup border change](../src/popup.rs) |
| `draw_overlay_screen` | Needed | Draw composed popup rows with their style through the private TTY. Printing command text or setting redraw flags cannot deliver this supplied screen at terminal coordinates. | [Popup drawing](../src/popup.rs) |
| `prepare_overlay_render` | Needed | Incremental popup parser output must skip direct writes while an overlay redraw is pending and otherwise receive its terminal coordinate view. Other methods cannot query that pending-redraw decision. | [Popup terminal callback](../src/popup.rs) |
| `prepare_pane_render` | Needed | A screen-write callback must combine session/window membership, Pane dirty state, Client viewport and status offsets. Pane geometry alone cannot supply the private Client terminal mapping. | [Screen-write client callback](../src/screen_write.rs) |
| `format_value` | Needed | Evaluate lazy client format keys such as name, PID and typed creation time. Operational methods intentionally do not expose the complete formatting state. | [Client name/PID](../src/format/callbacks.rs), [creation time](../src/format/callbacks.rs) |
| `detach` | Needed | Request detach with its message, reason, exit-session name and eligibility guards. Exit, session reassignment and immediate loss have different effects. | [detach-client](../src/cmd/entries/detach_client.rs), [detach implementation](../src/server_client.rs) |
| `suspend` | Needed | Stop the terminal and send `MSG_SUSPEND` while retaining attachment for resume. Detach, exit and loss cannot preserve that behavior. | [suspend-client](../src/cmd/entries/detach_client.rs), [suspend implementation](../src/server_client.rs) |
| `lost` | Needed | Immediately tear down a suspended client during shutdown, cancelling work, unregistering and freeing resources while retained holders may survive. Exit/detach only request transitions; Rc drop is not logical teardown. | [Server shutdown](../src/server.rs), [client loss](../src/server_client.rs) |

### Wrappers removed during implementation

| Former proposal | Replacement and preserved obligation |
| --- | --- |
| `Session::is_alive` | Existing registry resolution of opaque holders; logical validity remains distinct from allocation lifetime |
| `Session::is_window_shared` | Group/reference-count helper; avoid additional Window clones because counts affect behavior |
| `Session::select_last_window` | `last_winlink` plus `select_winlink` in `session_last`; the read operation also supports nonmutating command targeting |
| `Session::update_environment`, `Session::spawn_environment` | Child-environment construction now uses component scopes and `id`; selected attachment merging still has legacy callers |
| `Session::release`, `Client::release` | Existing deferred-release infrastructure; keep current release sites until their ownership contracts are migrated |
| `Session::group_members` | Group-registry traversal using observer identity; `session_group_for` does not project Session |
| `Session/Window/WindowPane::with_options`, `Session::with_environment` | Read through existing scoped edit access, using a shared reborrow |
| `Client::handle_key` | Use `handle_key_after(event, None, None)` for ordinary dispatch |
| `Window::apply_style` | Reusable style helper over scoped option access, with expansion between scopes |
| `Window::update_focus` | Existing helper composes `active_pane`, `pane_is_focused` and `WindowPane::update_focus` |
| `WindowPane::is_visible` | Existing helper combines parent `is_zoomed` with `prepare_render(default_context, false) != 0`; the false mode has no Pane side effects and preserves the unplaced/deferred distinction |
| `Session::update_size_state` | Registry-wide Session sizing helper owns reset/cache refresh and retained-unregistered counting together; no external caller needs an instance-level count setter |
| `Window::request_redraw` | Server coordination over Client, Session and existing winlink relationships; the helper still needs that internal migration |

## Why these boundaries are smaller

**Session does not wrap the winlink collection API.** `with_winlinks` supplies
scoped read access to the existing `winlinks` component. Lookup by index, ordered
traversal, counting and ordinary containment stay on that component. Resolving a
link's window stays on `winlink`; callers access the resulting window holder
through `Window`. These operations do not each need another Session method.

The retained Session navigation methods have these responsibilities:

| Method | Why it belongs on Session |
| --- | --- |
| `current_winlink` | Reads current selection, which is session state outside the collection |
| `select_winlink`, `last_winlink` | Separate selection/history mutation from MRU observation; `session_last` composes them |
| `link_window`, `detach_window` | Coordinate index ownership, selection/history, group synchronization and notifications |
| `spawn_window`, `attach_window` | Separate process creation/respawn from adoption of a window containing an existing process |
| `renumber_windows` | Coordinates index changes and group behavior |

Next/previous target resolution, including wraparound and alert filtering, can
use the existing collection operations plus current selection; the result is
committed through `select_winlink`. Existing `session_next`/`session_previous`
helpers may remain as convenience algorithms written against that interface,
without adding methods or projecting Session fields. Keep target resolution and
selection at the same callback boundaries as today. `last_winlink` now serves
`-t '!'` without changing selection, and `attach_window` serves
[break-pane](../src/cmd/entries/break_pane.rs) without spawning a replacement
process. The corresponding command callers use these methods.

The read-only `winlink_find_by_index`, window lookup and count helpers now
accept `&winlinks`, and ID lookup uses `Window`. Storage privacy still needs
work: `winlinks.storage` currently exposes a RefBox that permits mutation even
through `&winlinks`. Hide that escape path while migrating remaining component
consumers.

The read closure cannot mutate the collection or its entries, invoke reentrant
callbacks, or leak collection references/pointers through results or captured
storage. It may return copied values and existing weak link observers. Resolve
those observers using the existing lifetime guarantees after the closure ends.
Do not add `with_winlinks_mut`: it would expose the coordinated mutations the
Session methods must own.

**Client exposes its attached-session relationship deliberately.** Keep
`attached_session` (previously named `session_observer`); it is not merely a
convenience for reading client fields. These callers need the Session identity:

| Caller | Use of the attached session |
| --- | --- |
| `cmd_find_from_client`, [cmd/find.rs](../src/cmd/find.rs) | Establishes default session/window/pane target state and delegates to session target resolution |
| Next/previous session selection, [switch_client.rs](../src/cmd/entries/switch_client.rs) | Uses the current session as the starting point in session ordering |
| `format_defaults`, [format.rs](../src/format.rs) | Infers a missing session context from the client, then resolves the session's current window/pane |

These are coordination tasks, not Client-owned algorithms. Removing the accessor
would require Client forwarding APIs for target resolution, session navigation
and context construction. One existing Weak relationship is the smaller boundary;
receivers upgrade under the existing lifetime rules and use the Session trait.

Other callers do not need to follow that relationship: pane-focus checks use
`focuses_window`, and control-output guards belong inside `control_write_output`.
Identity-only filters can compare weak handles without exposing Session fields.
The getter grants neither Session field access nor an unrestricted model borrow.

**Formatting is one existing consumer interface.** Each entity answers its own
existing format keys using `FormatValue`, retaining lazy evaluation, typed times,
cache behavior and current context. Formatting-only fields need no public getter
or metadata copy. Operational consumers must use typed operations, never parse
format strings to make decisions. Session activity remains a typed value query
because target selection also combines it with attachment state outside formatting.

**Registry-wide work stays with its owner.** Session sorting compares private
timestamps inside [session/sort.rs](../src/session/sort.rs); alert delivery keeps
its reset-then-deliver ordering inside [session/alerts.rs](../src/session/alerts.rs).
These existing registry/relationship operations need no per-instance timestamp,
alert-flag, or attachment-count setters. They remain distinct from operations
on a single retained holder. Their legacy access to other entities is still
subject to the same migration requirement.

**Configuration is an intentional component boundary.** `with_options_mut`
gives scoped access to the existing options API; readers use a shared reborrow
inside the scope. The method does not promise rollback, decide whether an arbitrary result
means "changed", or implicitly fire notifications. The existing configuration operation keeps those
policies and runs notifications after releasing storage borrows. Array edits can
partially mutate before failure today; introducing transactional behavior would
be a separate change.
Separate read-only methods were removed because the scoped edit method also
supports reads. Style expansion uses a reusable two-phase component helper in
[style/scoped.rs](../src/style/scoped.rs), preserving cache/reset timing without
holding a component reference across format callbacks. It needs no Window style
method or new metadata structure.

Component closures may not change owner/parent links, free the component, reenter
model code, invoke listeners, or leak references/pointers through results or
captured storage. Captured-owner destruction can also reenter: split that work
out before promising the closure contract. This is deliberately different from
exposing mutable session winlinks, where even ordinary edits would bypass core
selection, group and ownership invariants.

**Bookkeeping follows the operation.** `Session::on_attached` owns the consecutive
activity/theme/time/alert steps in the current Client attachment path. It is
called at that same point, not used to reorder earlier focus/resize or later
client callbacks. The sizing pass now calls a Session-owned registry operation
before Client status/Window sizing. It resets registered Sessions' attachment
counts and status caches, then counts attached clients. A retained Session that
has already left the registry only receives the legacy count increment; its
cached status stays unchanged. Keeping this whole pass inside Session removes
the proposed per-instance `update_size_state` method. The duplicate cache refresh
in `options_push_changes` was removed because recalculation repeats it with no
intermediate callbacks. The regression in [session/size.rs](../src/session/size.rs)
covers registered, unregistered-retained, and unattached-client cases.

Window resize delegates to the coordinated resize path, not just the low-level
size assignment. Client runs rendering internally and preserves partial redraw
behavior. Pane parsing runs internally on stream input; `finish_cycle` keeps
post-client resize/output cleanup at its existing event-loop position. None of
these moves permits accessing another core entity's fields.

**Reuse does not require lending live internals.** Environment readers use
scoped access and clone the existing `environ` only when retention is required.
Direct set/unset/clear operations use its existing storage API. Updating a session
environment from an attached client must preserve
`update-environment` selection and hidden/unset semantics. `copy_screen` copies
into an independent existing screen; it cannot alias the pane's screen, and the
caller retains its existing `screen_free` duty. It does not expose a pane-bound
`screen_write_ctx` or a mode screen pointer.
When transferring environment data between Client and Session, first clone the
existing `environ` out of the Client read scope, then invoke the Session operation;
do not reenter Session while the Client component borrow is still held.
The selected-update and child-environment algorithms can be helpers using these
scopes and `Session::id`; they need no additional Session storage access.

## Lifecycle and cross-entity contracts


The preferred ownership shape is: the logical owner initiates destruction, and
ordinary retained holders simply drop their Rc values. A holder releasing its
reference must not destroy an entity still in use by other owners. Explicit
`destroy` does not by itself establish that all remaining releases are trivial.

| Entity | What release currently adds to ordinary Rc drop | Trait decision |
| --- | --- | --- |
| WindowPane | `window_pane_remove_ref` logs and calls `drop(owner)`; existing `Drop` performs final storage cleanup | Omit `release` from the target trait. Keep explicit pane destruction for logical/resource teardown; migrating the old function's callers adds no new Drop cleanup responsibilities. |
| Session | `session_remove_ref` captures the Rc until event dispatch/cancellation; existing `session::drop` already frees final storage | The trait wrapper is derivable through existing `shared::rc::release_later`. Keep each deferred-release obligation until its caller is migrated; ordinary drop requires a separate dispatch/container ownership audit. |
| Client | `server_client_unref_owned` defers Rc drop; final `client::drop` still frees queue/storage and registry bookkeeping | The same generic deferred-release helper can replace the trait wrapper. Immediate final drop during queue/callback work can invalidate storage still in use. |
| Window | `window_remove_ref` checks the final owner, fires close callbacks while it remains live, and destroys only if callbacks did not retain it | Keep the close/release protocol. A final `Drop` cannot substitute for it: the Rc strong count is already zero and Weak parent upgrades fail. |

For session/client, moving only the registry owner's deferred drop into
`destroy`/`lost` is insufficient. Event payloads, monitors, queued input, files,
and formatting contexts can retain an owner into later turns. Those release
boundaries must be audited too. A lifecycle/dispatch owner can eventually protect
those scopes so ordinary holders need no release method; this sketch does not
claim that guarantee already exists.
Removing an entity-specific `release` method in favor of the existing generic
deferred-release helper is a smaller change: it preserves the deferred lifetime
at every migrated call site. Preserve existing diagnostic logging too, using
trait queries where needed. Neither alternative moves resource cleanup into Drop.

Window cleanup must remain before the last strong owner disappears. A close
callback can retain a still-live window and postpone destruction, and pane mode
cleanup upgrades its parent. Calling destroy unconditionally at every release
would change that behavior. The existing tests
`close_callback_retention_defers_pane_cleanup` and
`dropping_the_last_owner_without_explicit_release_is_rejected` in
[window.rs](../src/window.rs) exercise this distinction.

All current dedicated cleanup/release calls remain authoritative until their
callers and lifetime contracts are migrated. This document changes no runtime
cleanup. `Client::lost` is its logical teardown operation; detach and suspend
retain their separate protocol/terminal meanings.

The `last-window` behavior selects the most recently used history entry, not
the highest index or previous index. `session_last` is now a convenience helper
using `last_winlink` and `select_winlink`; internal link-removal fallback keeps
its existing ordering.

- The Window release method delegates to `window_remove_ref`. Session/Client
  trait release wrappers were omitted; current `session_remove_ref` and
  `server_client_unref_owned` call sites still preserve their deferred lifetime
  duties. Preserve these behaviors and existing destructors during migration. Pane holders can use ordinary Rc
  drop after its trivial release wrapper is migrated; logical destruction remains
  explicit. No new resource-cleanup responsibility moves into Drop.
- A potentially final window owner must take the explicit release path. Do not
  clone inside final release or sharing checks: strong counts affect behavior.
  The sharing helper preserves `session_is_linked`, not ordinary containment.
  Group traversal preserves retained owners and their release obligations.
- Keep guaranteed Weak upgrades where enclosing ownership proves liveness.
  Construction, failed-spawn paths and delayed callbacks may require fallible
  lookup. Allocation liveness alone does not prove logical validity. Query and
  release methods must accept the retired states allowed by existing code.
- `spawn_window` coordinates the session-index phases using existing
  `spawn_context`; its session observer must match the receiver. It must preserve
  deferred notifications, destructive replacement, and failed-spawn cleanup.
  Ordinary attach/detach cannot substitute for these phases. Session rename
  similarly keeps index rekeying and notification together.
- Window membership/layout operations coordinate pane placement and option
  parenting before callbacks see the change. Panes inherit window options;
  windows and sessions inherit their respective global options. Layout helpers
  must call WindowPane operations for pane changes even if in the same module.
- `reattach_after_session_destroy` preserves clear-current, clear-history,
  attach-or-exit ordering. Normal `set_session` remembers the old session and
  cannot replace it.
- Existing `status_prompt_input_cb` already receives a borrowed client holder.
  Overlay aliases and `tty_ctx_set_client_cb` now use borrowed holders too,
  preserving their other contracts.
  End model/component borrows before listeners, reentrant calls or callback-capture
  destruction. Related holders inside existing component types also use traits.

## Implementation coverage still to establish

The adapters compile and several production paths now use them. This is not yet
complete encapsulation: the models still expose legacy fields, old helper bodies
still project other entities, and not every documented capability has all callers
migrated. A delegating method is implemented, but delegation alone does not prove
the trait-only boundary.

| Area | Remaining work |
| --- | --- |
| Link collections and session groups | `last_winlink` and independent window adoption now have callers. Read helpers now accept `&winlinks`; hide mutation-capable storage and migrate group creation/synchronization without exposing Session bookkeeping. |
| Alerts, pane ordering and pane moves | Session alert bookkeeping now lives in its owner, with the original reset/delivery order. Window alert fields and pane placement/reparenting still need owner operations; resize alone cannot change offsets/parent. |
| Layout and spawning | Named/saved/cycled/spread layout selection and floating/tiled split allocation have adapters and command callers. Session spawn transactions now live inside Session; spawn.rs uses its option/environment/cwd/termios interface. Window/Pane layout and process helpers still need migration while preserving every failure/zoom path. |
| Focus and sizing | Window and Pane focus operations and Client dimension/cap decisions now cooperate through traits. Session count/status preparation now belongs to its registry owner. Latest-client reads and remaining Window resize bookkeeping still need migration. |
| Pane modes and process state | Nullable mode arguments and explicit pane cleanup are implemented. Existing mode callbacks, pipe/startup helpers and scrollbar work still require a complete field-access audit. |
| Terminal rendering | `copy_screen` supports displayed/base screens and independent hyperlink storage. Migrate preview/render consumers and define remaining style/border/palette/dirty-region interactions before claiming efficient rendering coverage. |
| Output cycle | `finish_cycle` now runs at the existing post-client point and uses Client's scoped output offset. Control cursor initialization, encoding, advance/skip and subscriptions now use Pane methods. Migrate remaining consumers beyond control without exposing buffers. |
| Client commands and UI | Sizing, detach/suspend, destruction reattachment and key injection have callers. Overlay and TTY callback aliases now receive holders, and popup uses traits for Client/Session/Window access. Migrate remaining message/control/file/cache paths; no TTY/status/control-model getter. |
| Formatting | Builtins with model prefixes now dispatch through the corresponding trait. Receiver and format-context identity must match; unknown/foreign keys return `None`. Session builtin bodies and its history-dependent keys now live in Session; option/environment expression loops leave component scopes before expansion. Other entity callback bodies still need ownership migration. |
| Session configuration/spawn settings | Spawn/environment helpers now use Session scopes and copied cwd/termios. Migrate remaining option/environment consumers while preserving notification timing and explicit cleanup. |
| Interface necessity | Derivable read/dispatch/group wrappers were removed. Preserve the documented cache timing, destruction-state lookup and constant-time output skipping. Continue auditing newly migrated callers instead of treating the current count as a proof of minimality. |

For each operation, record an external caller, the information it requires, and
why an existing method cannot serve it. Remove proposed methods with no remaining
consumer. Revisit repeated sequences that can become one operation, while keeping
observation and callback timing unchanged. This gives a concrete test for a
minimal interface rather than a method-count target.

The final check is to make core fields private and audit all external projections,
nested pointers and callbacks, including cross-entity accesses within one source
module. Existing [model-pointer checks](model-api-migration.md) alone do not prove
trait-only access. During implementation, use the existing model storage and
callback tests, add coverage for changed contracts, and preserve the
[release-order audit](release-order-audit.md).

The approach follows [BufferEvent](../src/reactor/streams/api.rs) and the
[client-file adapter](trait-clientfile.md): expose operations on existing holders,
reuse established types, and preserve lifecycle behavior.

## Verified original implementation checkpoint

For subsequent changes and remaining boundaries, see
[model-boundary-progress.md](model-boundary-progress.md).

`cargo test --workspace` passes: 757 tests, none ignored. The checks include
session rename/index behavior, nonmutating last-window targeting, adoption of an
existing pane process, layout/zoom failure ordering, inherited style expansion
with reentrant callbacks, independent output cursors and screen copies, control
byte framing, overlay replacement, retained-unregistered Session sizing, recursive
option formatting and derived visibility, and explicit Window cleanup while retained
holders survive. A declaration/implementation/document comparison covers all
102 methods; documentation links and whitespace checks pass too.

Session state now lives in [session/model.rs](../src/session/model.rs), re-exported
through the existing shared path. Formatting timestamps, lock timer, alert flags,
status cache and index entry are private to that owner. Remaining fields retain
legacy visibility while their callers migrate.

These checks do not establish repository-wide encapsulation. For example,
[layout/core.rs](../src/layout/core.rs) still changes Pane layout fields,
[options.rs](../src/options.rs) still resolves live model option pointers, and
legacy [format callbacks](../src/format/callbacks.rs) still project Window, Pane
and Client state. Finishing that migration requires a broader caller conversion
and privacy audit than implementing the methods listed here.
