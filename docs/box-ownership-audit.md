# Box ownership audit

Reviewed 2026-09-27 against tmux revision
`e880cf63e0a9fe095d7c5d313761520fb1a8653c` from the pinned Nix source.

Scope: `Box::into_raw` and `Box::from_raw` in workspace Rust sources, excluding
`src/compat/`. This is an ownership-path review and migration backlog, not a claim
that the remaining raw APIs are safe or required by C. The reactor and translated
C-shaped functions are Rust code and remain migration candidates.

Reproduce the inventory with:

```sh
rg -n 'Box(?:::<[^;]+>)?::(into_raw|from_raw)' --glob '*.rs' --glob '!src/compat/**'
```

The workspace crates `hmux-buffer`, `hmux-cmdparse`, and `hmux-rt` had no matching
sites during this review. A match count includes tests, constructors without a
local destructor, and multiple transfers of the same allocation; it is not a
count of production ownership pairs.

Preserve existing heap allocations and pinned tmux behavior. Prefer an owning
`Box` plus borrowed readers. Use shared owners and weak observers when callbacks
can outlive a borrow or destroy their own logical owner. Never retain a map or
mutable state borrow across a callback that can replace or remove it.

## Completed migrations

| Allocation | Result |
| --- | --- |
| Argument flag entries and values | `BTreeMap<u8, Box<args_entry>>`; values remain boxed. Flag/value iterators borrow the argument set. Drop releases entries and command references. |
| Customize prompt/editor records | Typed owned records with shared callback lifetime where needed; weak input callbacks and automatic cancellation cleanup. |
| Command-prompt and confirm-before callback records | The input closure owns the original `Box`; callback functions borrow it. Drop performs the prior cleanup. Active prompt dispatch already defers cleanup until callback return. |
| Run-shell callback record | A reactor one-shot owns the `Box` until timer dispatch; successful startup transfers it into the job completion closure. No embedded callback ownership cycle. Reactor shutdown also drains deferred cleanup scheduled while cancelling these owners. |
| If-shell callback record | Successful job startup transfers the Box into its completion closure; cancellation drops it without reconstructing ownership from a pointer. |
| Control window size overrides | The window-ID map owns `Box<control_window>`; lookups borrow entries and removal drops them. |
| Control pane entries and pending output | Pane map keeps boxed owners; readers borrow entries and offsets. Pending output stores pane IDs and resolves entries around operations that may discard output. |
| Buffer editor callback record | Shared typed record is captured by the completion closure; a cell publishes editor identity after startup. Cancellation drops the closure, and weak buffer identity prevents updating a replacement buffer. |
| Synchronized-output dirty bitmap | Pane stores `Option<Box<[bitstr_t]>>`; readers borrow/index the slice. Resize, explicit clear, and pane destruction release it normally. |
| Prepared command state | Shell, prompt, and pane-mode records own `Box<args_command_state>` and borrow it during expansion. Retained command lists and clients have typed `Rc` owners; Drop preserves deferred client release. |
| Parser nodes and nested payloads | Constructors and containers transfer boxes directly. An enum owns string, nested command-set, or shared compiled-command payloads; ordinary Drop handles success and error cleanup. |
| Source-file callback state | A read callback owns an optional Box, borrows it during progress, then transfers it to the next read or command callback. Drop handles cancellation and exactly-once depth cleanup, preserving the startup exception. |
| Control-state root | Client stores `Option<Box<control_state>>`. Callbacks resolve short borrows through the client; teardown keeps state available until monitor and stream cleanup finishes. |

Each migration has its own commit. Validation includes the workspace suite,
focused lifecycle tests, live command comparisons with the pinned tmux build,
and focused Valgrind checks. Changes from three independent worktrees were reviewed
and integrated as separate commits.

## Remaining production ownership paths

These are proposed directions, with the lifetime constraint identified at each
ownership boundary. Broader migrations need their callers changed together.

| Sites / records | Current ownership and proposed migration |
| --- | --- |
| `src/arguments.rs`: argument root | Command owns the argument allocation through a raw field. Return `Box<args>`, store it in the command, and borrow at consumers; parser failure and command-copy paths must transfer the owner explicitly. |
| `src/cmd/core.rs`: `cmd` | Command-list storage owns command records, but constructors/free helpers and traversal use raw handles plus a membership side table. Transfer `Box<cmd>` into/out of the list and borrow records; migrate argument ownership with it and eliminate detached-pointer traversal. |
| `src/cmd/queue.rs`: queue root and detached items | Queue stores `Box<cmdq_item>`; detached construction and enqueue recover owners from pointers. Return/accept typed detached owners. Queue callbacks, waiting commands, and cancellation retain item identities, so borrowing alone does not cover every observer; use IDs or weak handles across dispatch. |
| `src/window.rs`: `window_pane_input_data` | File-progress and cancellation callbacks share data; the file handle is assigned after startup. Use a shared typed record and a cell for file identity, with owned client-reference cleanup. Cancelling a progress read schedules a terminal event and must not release its client prematurely. |
| `src/environ.rs`: environment root | Entries already have typed ownership; environments still transfer through raw create/free APIs. Store `Box<environ>` in sessions, clients, globals, and temporary spawn owners; borrow for copying/lookup and retain nullable ownership where present. |
| `src/events_payload.rs`: payload root/items | Item index stores raw owning addresses; value cleanup releases retained client/session/window/pane references. Own entries in the map and detach an owner before cleanup that can fire callbacks. Convert payload transfer through event dispatch to a typed owner with explicit target/value Drop ordering. |
| `src/format/tree.rs`: tree and entries | Entries are raw map values; lazy callbacks can add/replace entries while evaluating. Map can own boxes, but evaluation must take callback state, release the entry borrow, invoke it, then resolve the key/identity again. Tree Drop must preserve client release and callback cleanup ordering. |
| `src/format/jobs.rs`: per-client cache and jobs | Caches own raw job records observed by async callbacks. Use a boxed cache and typed shared job records/weak callback handles (or stable IDs); remove expired owners before cancellation and avoid borrowing the cache across `job_free`. |
| `src/key_bindings.rs`: key bindings | Key tables index raw binding owners; bindings retain command lists. Make indexes own boxed bindings, with Drop releasing commands. Migrate next-entry traversal and snapshot command references before actions can replace bindings. |
| `src/options.rs`: option root/entries/array items | Maps own raw entries/items; parent, owner, and monitor links cross scopes. Own entries/items as boxes and borrow lookup results; cleanup must release values before monitor teardown. Parent/owner links and callbacks need typed observers or IDs before all readers can become safe borrows. |
| `src/hooks.rs`: `hooks_monitor` | An option's `monitor_data: void*` owns the record; monitor and event callbacks observe it and may destroy it reentrantly. Use typed ownership in the option with weak callback captures upgraded for dispatch. `_hooks_monitor` event payload also carries raw identity and needs a weak typed variant or stable ID. |
| `src/monitor.rs`: monitor set/item/pane/window | Maps store raw owning records; reporting callbacks can remove an item or destroy the whole set. Use owning maps, short borrows, and key/generation relookup across callbacks. The set's timer and callback lifetime need a typed retained dispatch owner or weak registration. |
| `src/layout/core.rs`, `src/shared/layout.rs`: layout cells | Child vectors already own boxes, but detach/reparent/replace return implicit raw owners. Transfer `Box<layout_cell>` explicitly, including detached leaf preservation for `only_nodes`; parent and pane links need IDs or weak observers. A container type change alone cannot fix reparenting ownership. |
| `src/screen_redraw.rs`: redraw scene | Client owns one cached scene with boxed rows/spans. Store `Option<Box<redraw_scene>>` on the client; take/drop old scene before rebuild and borrow it for rendering. Check client/window observer lifetime and callbacks while drawing. |
| `src/window_copy.rs`: backing screens | Clone/view setup allocates a screen, stores it in `backing`, and manually frees it during refresh or mode teardown. Use an owned boxed screen and replace/drop it after screen resource cleanup. Preserve any explicit `screen_free` duties not yet handled by screen Drop. |
| `src/window_clock.rs`, `src/window_switch.rs`, `src/window_panes.rs`, `src/window_copy.rs`, `src/window_buffer.rs`, `src/window_client.rs`: mode roots | `window_mode_entry.data: void*` owns heterogeneous boxed records. Replace it with a typed owning enum; callbacks must resolve a live mode and release borrows before commands can destroy it. Timers, prompt cleanup, editor cancellation, zoom restoration, and nested mode-tree ownership must keep their teardown order. |
| `src/job.rs`: job root | Intrusive global list owns jobs; stream callbacks, popups, and format caches retain raw observers. Use a typed registry owner with weak/ID observers and a logical-close operation. Callbacks can cancel themselves; release state borrows before dispatch and retain an active owner until return. Preserve unlink, free callback, process termination, stream cancellation, and fd close ordering. |
| `src/reactor/streams.rs`: `bufferevent` | `STREAMS` retains `Rc<StreamState>` but its stream field is a raw owning pointer; tasks retain state and `BUFFERS` has weak state links. Store the stream under typed ownership and expose handles. Dispatch snapshots callbacks and rechecks liveness because callbacks can free the stream. Preserve deferred destruction and PID-sensitive fd flag restoration. This is not a foreign ownership boundary. |
| `src/spawn.rs`: editor state | Success transfers the boxed state to `window_pane.editor`; mode records observe it. Pane teardown clears its editor field before invoking completion and unlinking the temporary file. Use a pane-owned `Option<Box<_>>` plus editor/pane IDs or weak observers. Cancellation drops only the callback, intentionally keeping process/state/temp-file ownership alive until finish. |
| `src/input.rs`: input context | Pane/popup/view mode owns the context; two timer callbacks capture its address. Use a typed context owner and weak/ID timer callbacks. Teardown must release requests, cancel timers, and stop synchronized output before physical destruction. |
| `src/popup.rs`: popup record | Startup converts a Box to raw, then recovers it for the owned overlay; job and overlay callbacks retain its address. Keep the Box through startup and use typed overlay state with safe dispatch handles. Handle popup destruction from job completion, cancellation, and self-closing input without an owner borrow across callbacks. |
| `src/tty_term.rs`: terminal record | TTY owns a terminal record also linked in the global intrusive terminal list. Keep a typed TTY owner and replace list links with weak handles/IDs or a registry owner. Failure cleanup and unlinking must precede destruction. |
| `src/proc.rs`: `tmuxproc` | Constructor publishes a process-lifetime raw owner without a matching Box destructor. Signal and peer callbacks borrow it. Use an explicit process-root owner with weak/ID callbacks and ordered signal/peer shutdown; do not treat the unmatched allocation as an ordinary local pair. |

## Tests and fixtures

`src/input.rs`, `src/mode_tree.rs`, `src/monitor.rs`, `src/spawn.rs`,
`src/window.rs`, `src/events_payload.rs`, and
`src/format/jobs.rs` include raw allocation fixtures for APIs still under
migration. The command-prompt lifecycle fixture now uses a boxed prepared state.
Convert remaining fixtures when their corresponding API changes; they are not
evidence of an external C ownership requirement.

`tests/fixtures/architecture/rust-deallocation.rs` intentionally contains
`Box::from_raw` as input to an architecture test and must retain that example.
