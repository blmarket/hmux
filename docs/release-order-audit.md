# Release ordering before further Drop migration

Reviewed 2026-09-28 for the third prompt in `~/issues.md`. This is a review of
migration blockers, not a request to replace the current explicit release
protocol. The existing functions must continue to run until each corresponding
protocol has been migrated and verified.

## Findings

A final `Rc<T>` destructor runs after its strong count reaches zero. Its weak
backlinks cannot be upgraded even though `&mut T` still exists.

`RefBox` requires a separate caution: in the current
[implementation](../hmux-refbox/src/internals.rs), `drop_data` marks the allocation
Dropped **after** `drop_in_place` returns. An unborrowed owner's destructor can
therefore still be reached through a Weak, but that must not be treated as safe
parent access: a new mutable borrow can alias the destructor's `&mut self` or
observe partially destroyed fields. Dropping an owner while borrowed instead
marks DroppedWhileBorrowed and defers data destruction until the borrow ends.
A future library change that rejects new weak borrows during destruction would
also expose any cleanup relying on the current behavior. Do callback cleanup
while the owner is explicitly live and unborrowed, before invoking Drop.
Neither pointer type automatically preserves the C parent-during-free contract.

The following paths were read in the current sources. “Blocker” here means a
condition a future Drop implementation must satisfy, not necessarily a defect
in the current explicit cleanup path.

| Path and current evidence | Blocker and mitigation |
| --- | --- |
| [Window release](../src/window.rs): `window_remove_ref`, `window_destroy`, `window_destroy_panes`; [mode cleanup](../src/mode_tree.rs): `mode_tree_free` | The final window owner stays held while unzooming, dropping layouts and menus, and destroying panes. Mode cleanup upgrades pane/window backlinks to unzoom. Moving `window_destroy` into `window::drop` makes the window upgrade fail. Keep a live outer owner during logical close, or move child cleanup to functions receiving explicit parent context; only let the final destructor release already-detached storage. Preserve the Live → Destroying → Destroyed guard. |
| [Pane teardown](../src/window.rs): `window_pane_destroy`, `window_pane_free`, `window_pane_free_modes`; [input destructor](../src/input.rs): `input_ctx::drop` | Wait/editor completion, prompt cancellation and mode cleanup precede parser, streams, timers and screens. The parser upgrades its pane to stop synchronized output. At the final pane Rc drop that upgrade cannot succeed; `window_pane_free` already compensates with direct pane access. A full migration needs a shutdown context that can stop synchronization and release requests before screens disappear, without upgrading the dying pane. |
| [Layout cells](../src/shared/layout.rs): `layout_cell::drop`; [window teardown](../src/window.rs): `window_destroy` | Cell destruction upgrades its pane and clears the pane's layout backlink. Window teardown deliberately releases current/saved layouts before panes. Pure field-order destruction, or dropping panes first, changes that unlinking behavior. Detach both directions explicitly, or pass live pane access into layout teardown before releasing pane owners. Do not let child destructors traverse partially destroyed parents. |
| [Client loss](../src/server_client.rs): `server_client_lost`, `server_client_unref_owned`, `server_client_free` | Loss is an observable operation while the client is alive: clear overlay/prompts, cancel file waits, fire file/detach events, stop control/TTY/status, cancel timers and jobs, close peer/descriptors, then defer the registry owner's release. `client::drop` currently only performs final storage/index checks. Putting loss in that destructor breaks weak upgrades and lets callbacks see already-dropped dependencies. Retain a live client through a separate close phase; a future owner wrapper could initiate that phase while an inner allocation remains upgradeable. |
| [Queue cleanup](../src/cmd/queue.rs): `cmdq_remove`, `cmdq_abort_file_wait`, `cmdq_list::drop`; [file waits](../src/file.rs): `file_fire_done_cb`, `file_cancel_cmdq_wait` | An item must be unlinked with its sole strong owner, and its file wait must finish/cancel before item removal. Cancellation runs before client, command-list, state and callback captures are released. The client release is deferred; immediate destruction could drop its queue while removal still accesses that queue. Keep queue mutation private, finish cancellation while queue/client storage remains valid, then release captures outside container borrows. A queue destructor must not simply drain waiting items without their cancellation protocols. |
| [Session teardown](../src/session/mod.rs): `session_destroy`, `session_remove_ref`, `session_free`; [winlink removal](../src/window.rs): `winlink_remove` | Session close notifications, group removal, timer cancellation and window unlink events run with an owning session reference. Removing the last winlink can destroy a window and invoke further cleanup. Final `session::drop` only releases environment/options/history. A migration must separate notifying/unlinking from final storage release and retain the session through all callbacks, preserving window release via `window_remove_ref`. |
| [Prompts](../src/prompt.rs): `prompt_free`, `prompt_fire_callback`; [menus](../src/menu.rs): `menu_close`, `menu_free_data` | Prompt input may remove its own owner. Dispatch takes the input and free callbacks out and defers free until input returns; `PROMPT_FREE_PENDING` handles an explicit close during input. Menu cancellation detaches the menu first, takes out its callback, then frees its screen after cancellation returns. Calling these operations from ordinary field Drop would change callback order, lose weak access or free callback data during execution. Keep the dispatch-owned callbacks and explicit close state; detach an old instance before callbacks can install a replacement. |
| [Popup destructor](../src/popup.rs): `popup_data::drop`, `PopupGuard`; [editor completion](../src/spawn.rs): `spawn_editor_finish`, `spawn_editor_state::drop` | Popup Drop already resumes a published waiting command, defers client release, cancels its job, destroys input, then frees screen/palette. Active popup guards keep that destructor out of in-progress dispatch. Editor completion takes its owner out of the pane before invoking completion; temporary-file unlink happens after the callback returns. Preserve these dispatch guards, startup-failure distinction and completion-before-storage-release order. Merely owning these objects in fields is insufficient. |
| [Jobs](../src/job.rs): `job_free`, `job_finish`, `job_read_callback`; [format cache](../src/format/jobs.rs): `format_job_tidy_at` | The registry now owns RefBoxes and holders use weak handles. Explicit `job_free` removes the owner from the registry, runs the free callback with allocation/stream/fd still live, then kills a live child, frees the stream, closes the fd and drops the owner. Completion/update run without job borrows and can cancel the job. Format caches cancel before dropping callback targets. Moving all of this to `job::drop` would make callback weak access depend on the unsafe during-Drop behavior described above and could retain a registry borrow across reentry. Keep detached ownership through cleanup, or move callbacks/resources into a separate close context before owner destruction. |
| [Control shutdown](../src/control.rs): `control_stop`, `control_free_block`; [monitors](../src/monitor.rs): `monitor_clear`, `monitor_destroy` | Control state stays published while monitors, streams, offsets and blocks are cleared. Control-control mode aliases the read/write stream and frees it only once. Monitor timers stop before indexed records and retained sessions are released. A destructor must preserve publication, stream alias handling and timer cancellation; move the owner out only after dependent teardown or pass explicit state to every dependent cleanup. |
| [Formatting](../src/format/tree.rs): `format_clear`, `format_free`; [options](../src/options.rs): `options::drop`; [hook monitors](../src/hooks.rs): `hooks_monitor::drop` | Format callback-capture destructors can add entries to the same format tree. The current loop pops entries and drops each outside the map borrow, then defers client release. Default map field Drop would not preserve this reentry contract. Options explicitly release values before monitor data, and hook-monitor destruction unregisters the sink before destroying the monitor set. Preserve these orderings; to migrate format trees, first prohibit/restructure callbacks that mutate a tree already being destroyed. |
| [Event payloads](../src/events_payload.rs): `event_payload::drop`, `event_payload_item::drop`; [prepared commands](../src/arguments.rs): `args_command_state::drop` | These already have custom destructors because ordinary field Drop changes model-release order. Payload items are removed in key order, retained models are released while names remain alive, and target references are released after item cleanup. Windows still require their explicit remove-ref function. Keep those protocols until model release itself is migrated; replacing explicit calls with plain Rc drops can bypass logical window destruction. |
| [Reactor streams](../src/reactor/streams.rs): `StreamHandle::free`, `bufferevent_free`; [application futures](../src/reactor/tasks.rs): `Task::cancel`; [processes](../src/proc.rs): `proc_free` | StreamHandle is an observer, so dropping it must not replace freeing the runtime-owned stream. Stream freeing unregisters tasks/buffer indexes before dropping callback captures outside registry borrows. Application task handles also require explicit cancellation before their callback targets disappear. Peer and terminal loops yield after dispatch so an executing future cannot revisit an owner released by its callback. `proc_free` stops signals and explicitly removes peers while the process allocation remains valid. Task factories recreate runtime resources after fork. A future owning stream/task wrapper needs a single close authority and dispatch-aware cancellation, not Drop on every cloned observer. |

## Other release families checked

Leaf storage (environment strings, argument values, grid lines, colours, regex
and compatibility resource wrappers) does not have the same parent-upgrade
requirement. This does not license skipping its existing explicit release calls.
`screen_free` is also an in-place reset: it clears queued writes before grids and
hyperlinks, and is used on reusable embedded screens. Replacing it solely with
Drop would lose that operation. TTY teardown stops terminal/event activity before
freeing key tables and terminal data; terminal Drop unlinks its intrusive index,
so that index must remain alive. Global option owners have raw child-parent
observers and must outlive the instances that consult them.

These are source-level findings for the release families above, not a proof that
every unsafe callback in the repository is reentrant-safe. Runtime shutdown
cancels deferred work as well as dispatching it: changing deferred owner release
to an immediate drop needs tests for both paths.

## Suggested migration sequence

1. Keep explicit close/release functions authoritative. Name and enforce logical
   lifecycle states separately from allocation liveness.
2. Remove child cleanup's need to upgrade a dying parent: use an explicit cleanup
   context, detach backlinks while owners are live, or keep a live inner model
   behind a distinct outer owner whose destructor initiates close.
3. Take callbacks and dependent owners out before dispatch; release all RefBox,
   RefCell and registry borrows before callbacks or callback-capture destruction.
4. Introduce owning resource wrappers only after deciding who closes each stream,
   fd and timer. Weak observers must remain nonowning, and aliases must not close
   shared resources twice.
5. Move final storage cleanup into Drop one domain at a time. Retain the manual
   entry point as a delegating wrapper until every caller is migrated. Test
   self-close, replacement during close, callback cancellation, last-owner release,
   both process/stream completion orders, and runtime shutdown without dispatch.

Existing regression evidence includes `freeing_modes_keeps_parent_alive_until_all_callbacks_finish`,
`mode_tree_cleanup_unzooms_a_logically_destroyed_pane`, the prompt/menu/overlay
self-close tests, format-tree reentrant capture cleanup, deferred-release
cancellation tests, the queue's sole-owner and cancellation tests, and the job
registry/completion tests. These validate current protocols; they do not by
themselves validate a future Drop-only implementation.
