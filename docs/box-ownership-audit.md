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
`Box` plus borrowed readers. Only retain `Rc` for types that already had
reference counting. Check whether observers can use bounded borrows or stable
identities first; when weak pointers are necessary, use a single `RefBox` owner.
An active RefBox borrow can defer destruction if a callback removes its owner,
but must not permit conflicting access. Never retain a map borrow across a
callback that can replace or remove entries.

The Rc reversal backlog is tracked in `../plan.md` relative to the repository
root. The completed conversions below supersede the earlier shared-ownership
guidance; other listed migrations still require that review.

## Completed migrations

| Allocation | Result |
| --- | --- |
| Mode-tree rows | Parent lists solely own RefBox rows. Parent links, visible lines, and traversal snapshots hold weak identities. Key/search/tag callbacks release row borrows before dispatch and stop when rows or trees are removed; names and action inputs use independent snapshots. Already reference-counted mode data remains retained during callback dispatch. |
| Prompt roots | Clients, panes, and modes solely own RefBox prompts. Dispatch observes weakly and takes both callbacks out before input, deferring cleanup until input returns even if the sole owner is removed. Status cleanup detaches its screen before callbacks can install replacements. |
| Menu state | Windows solely own RefBox menu records. Redraw scenes and key dispatch hold weak identities; key dispatch releases borrows before callbacks. Cancellation takes ownership out of the window and keeps its screen alive until the callback returns, preserving replacements and window teardown. |
| Mode buffer/client/customize/tree payloads | Mode lists solely own RefBox records; rows retain weak identities. Actions copy independent input snapshots before callbacks can rebuild the list. Client snapshots retain the already reference-counted client and preserve deferred release. |
| Pane and mode-tree prompt callback records | The cleanup closure solely owns a RefBox. Input closures and parent identity slots hold weak observers. Dispatch takes out the callback and ends its record borrow before invoking it; cleanup clears only the matching identity and preserves client/tree release order. |
| Paste buffers | The creation-order index solely owns `RefBox<paste_buffer>` records. Name lookup, formats, and editors hold weak identities; reads borrow only during use. Deletion invalidates observers, replacement cannot be mistaken for the old buffer, and event dispatch releases borrows before listeners run. |
| Hyperlink entries | The already reference-counted table owns boxed URI records; lookup returns a table-bound borrow. Style transfers copy values before insertion can evict their source. Entries have no weak observers. |
| Prompt history and mode-tree names | Owners store `CString`; readers that cross mutation or teardown take independent string snapshots. Neither string store needs weak observers. |
| Argument flag entries and values | `BTreeMap<u8, Box<args_entry>>`; values remain boxed. Flag/value iterators borrow the argument set. Drop releases entries and command references. |
| Customize prompt/editor records | Detached prompt items have one `RefBox` owner in cleanup; input closures observe it weakly and borrow only for dispatch, including self-close. Already reference-counted mode records retain their Rc protocol. Editor completion closures own boxed records and receive editor identity at dispatch. |
| Command-prompt and confirm-before callback records | The input closure owns the original `Box`; callback functions borrow it. Drop performs the prior cleanup. Active prompt dispatch already defers cleanup until callback return. |
| Run-shell callback record | A reactor one-shot owns the `Box` until timer dispatch; successful startup transfers it into the job completion closure. No embedded callback ownership cycle. Reactor shutdown also drains deferred cleanup scheduled while cancelling these owners. |
| If-shell callback record | Successful job startup transfers the Box into its completion closure; cancellation drops it without reconstructing ownership from a pointer. |
| Control window size overrides | The window-ID map owns `Box<control_window>`; lookups borrow entries and removal drops them. |
| Control pane entries and pending output | Pane map keeps boxed owners; readers borrow entries and offsets. Pending output stores pane IDs and resolves entries around operations that may discard output. |
| Buffer editor callback record | The completion closure owns a Box; dispatch supplies editor identity directly, removing startup sharing. Cancellation drops the closure, and weak buffer identity prevents updating a replacement buffer. |
| Synchronized-output dirty bitmap | Pane stores `Option<Box<[bitstr_t]>>`; readers borrow/index the slice. Resize, explicit clear, and pane destruction release it normally. |
| Prepared command state | Shell, prompt, and pane-mode records own `Box<args_command_state>` and borrow it during expansion. Retained command lists and clients have typed `Rc` owners; Drop preserves deferred client release. |
| Parser nodes and nested payloads | Constructors and containers transfer boxes directly. An enum owns string, nested command-set, or shared compiled-command payloads; ordinary Drop handles success and error cleanup. |
| Source-file callback state | A read callback owns an optional Box, borrows it during progress, then transfers it to the next read or command callback. Drop handles cancellation and exactly-once depth cleanup, preserving the startup exception. |
| Control-state root | Client stores `Option<Box<control_state>>`. Callbacks resolve short borrows through the client; teardown keeps state available until monitor and stream cleanup finishes. |
| Prepared command results | Expansion returns an owned command-list `Rc`; shell/prompt/pane callers borrow it for queue insertion. Confirmation callbacks retain a typed command-list owner. |
| Argument roots | Parsing/copying returns `Box<args>`. Commands and copy-mode dispatch own their argument sets; temporary find-window arguments and failed parses drop normally. |
| Event payload items and root | The ordered map owns boxed entries and readers borrow them. Event dispatch consumes the boxed payload; Drop releases entries in key order before the target. Replacement detaches the old entry before releasing model references. |
| Redraw scene cache | Client owns an optional boxed scene. Active rendering takes ownership and lends scene/span borrows; returning the scene preserves any newer cache produced by nested rendering. |
| Pane-input callback record | The read callback owns a Box. File creation initializes its weak file identity before opening or scheduling events, removing startup sharing. Drop preserves deferred client release. Failed opens and cancellation release the box normally; empty-pane stream teardown remains intact. |
| Command records | Command lists own `Vec<Box<cmd>>`; parse/copy/append transfer boxes and readers borrow records. The membership side table and detached next-pointer traversal are removed. Queue items retain their command list while observing stable boxed command addresses. |
| Parser input buffers | Argument and command parsing borrow slices; flag scanning uses bounded bytes and borrowed configuration/state. Empty-input behavior, libc character classification, positional callbacks, and diagnostics retain pinned semantics. |
| Copy/view backing screens | Mode data owns `Option<Box<screen>>`; snapshot cloning returns a box and readers borrow it. Refresh and teardown run `screen_free` before dropping the box, preserving parser/backing/visible-screen cleanup order. |
| Format entries | The ordered map owns boxes with typed text/time/lazy/cache state. Evaluation moves out the callback, ends the entry borrow, invokes it, then resolves the key and evaluation identity again. Entry cleanup occurs outside map borrows and before client release. |
| Key bindings | Lazily allocated indexes own boxed bindings and lookup/sort/list readers borrow them. Bindings retain typed command-list owners; dispatch snapshots key, flags, and a command-list reference before replacement or removal can invalidate a binding. |
| Argument payloads | Stored command payloads own typed `Rc` references; parser values borrow `&CStr` or `&Rc` with explicit lifetimes. Copying into stored arguments owns strings and retains command lists without carrying parser borrows or caches. Raw value/count parser entrypoints and unused raw value-array getters are removed. |
| Parser templates | Command and copy-mode argument descriptors borrow static C strings. Flag parsing reads their bounded bytes directly; all 196 template byte sequences are unchanged. |
| Queue roots | Clients and the lazy global queue own boxes directly. Client teardown drops its empty queue before releasing the client registry entry; queue Drop preserves the nonempty-queue invariant. Detached item ownership and active-item observers remain separate. |
| Layout cells and roots | Constructors, detach/replace operations, and custom parsers transfer boxes directly. Windows own active/saved root boxes; presets explicitly retain detached leaves until reinsertion, including floating panes. Drop clears pane observers; zoom transfers root owners without moving cells. Teardown restores zoom links without resizing dying panes, avoiding zero-count Rc retention. |
| Environment roots | Global, client, session, spawn, and temporary environments own boxes directly. Lookup/iteration/copy/update borrow actual values. Customize rows resolve weak session identities through short retained guards whose release remains deferred; nullable spawn/job environments use `Option<&environ>`. |
| Popup state | Startup keeps a box, owned by typed shared state. Overlay/job/input callbacks hold weak handles; active guards retain the allocation until callbacks return, including self-close. Drop preserves waiting-command, client, job, input, screen, and palette cleanup order. |
| Command descriptors | Commands store immutable static references and construction requires a descriptor. Lookup returns references; listing/completion borrow the bounded table, and dispatch/identity checks borrow descriptors and flags. All 92 descriptors and their order are preserved. |

Each migration has its own commit. Validation includes the workspace suite,
focused lifecycle tests, live command comparisons with the pinned tmux build,
and focused Valgrind checks. Changes from three independent worktrees were reviewed
and integrated as separate commits. The latest combined tree passes 573 workspace
tests and builds; 867 command name/alias/prefix/error observations match pinned
tmux. Integrated environment, layout, popup, queue, parser, and callback lifecycle
checks pass, including focused unit and live Valgrind checks with zero errors and
zero definite/indirect leaks.

## Remaining production ownership paths

These are proposed directions, with the lifetime constraint identified at each
ownership boundary. Broader migrations need their callers changed together.

| Sites / records | Current ownership and proposed migration |
| --- | --- |
| `src/cmd/core.rs`, `src/cmd/queue.rs`: command lists and observers | Records and arguments now have typed owners. List creation/free/append still expose raw retained `Rc` handles, and queue items observe commands through raw pointers backed by list references. Transfer typed list owners through parser results/queue state and represent observers with bounded borrows or stable identities. |
| `src/cmd/queue.rs`: detached items and observers | Queue roots and linked items have boxed owners; detached construction and enqueue still recover item owners from pointers. Return/accept typed detached owners. Queue callbacks, waiting commands, and cancellation retain item identities, so borrowing alone does not cover every observer; use IDs or weak handles across dispatch. |
| `src/events_payload.rs`, `src/shared/events.rs`: retained payload models | Root/items now have boxed ownership. Model variants and target s/w/wp still retain raw references; use typed model owners that preserve deferred client/session release and the window last-reference close event. Generic Rc Drop alone does not preserve window-release behavior. `_cmdq_item` and `_hooks_monitor` are remaining raw nonowning identities requiring typed variants/IDs or weak observers. |
| `src/format/tree.rs`: tree root | Entries now have boxed owners. Root create/free still transfer raw Box ownership; consumers and callbacks borrow the root through legacy pointers. Use an explicit tree owner and borrowed/typed callback access while preserving entry cleanup before retained-client release. |
| `src/format/jobs.rs`: per-client cache and jobs | Caches own raw job records observed by async callbacks. Use a boxed cache and typed shared job records/weak callback handles (or stable IDs); remove expired owners before cancellation and avoid borrowing the cache across `job_free`. |
| `src/key_bindings.rs`: key tables | Binding indexes now own boxes. The global table index and client table references still expose raw retained `Rc` handles with weak traversal links. Use typed registry/client owners and borrowed or identity-based traversal; active dispatch and listing must retain tables across removal. |
| `src/options.rs`: option root/entries/array items | Maps own raw entries/items; parent, owner, and monitor links cross scopes. Own entries/items as boxes and borrow lookup results; cleanup must release values before monitor teardown. Parent/owner links and callbacks need typed observers or IDs before all readers can become safe borrows. |
| `src/hooks.rs`: `hooks_monitor` | An option's `monitor_data: void*` owns the record; monitor and event callbacks observe it and may destroy it reentrantly. Use typed ownership in the option with weak callback captures upgraded for dispatch. `_hooks_monitor` event payload also carries raw identity and needs a weak typed variant or stable ID. |
| `src/monitor.rs`: monitor set/item/pane/window | Maps store raw owning records; reporting callbacks can remove an item or destroy the whole set. Use owning maps, short borrows, and key/generation relookup across callbacks. The set's timer and callback lifetime need a typed retained dispatch owner or weak registration. |
| `src/layout/core.rs`, `src/shared/layout.rs`: layout observers | Cells, detached leaves, and window/parser roots now have explicit boxed owners. Parent, pane, sibling traversal, and active/saved pane-cell links still use raw observers; replace these with borrowed traversal or stable identities while preserving reparent/zoom behavior. Legacy layout geometry/resize APIs also retain borrowed raw parameters. |
| `src/window_clock.rs`, `src/window_switch.rs`, `src/window_panes.rs`, `src/window_copy.rs`, `src/window_buffer.rs`, `src/window_client.rs`: mode roots | `window_mode_entry.data: void*` owns heterogeneous boxed records. Replace it with a typed owning enum; callbacks must resolve a live mode and release borrows before commands can destroy it. Timers, prompt cleanup, editor cancellation, zoom restoration, and nested mode-tree ownership must keep their teardown order. |
| `src/job.rs`: job root | Intrusive global list owns jobs; stream callbacks, popups, and format caches retain raw observers. Use a typed registry owner with weak/ID observers and a logical-close operation. Callbacks can cancel themselves; release state borrows before dispatch and retain an active owner until return. Preserve unlink, free callback, process termination, stream cancellation, and fd close ordering. |
| `src/reactor/streams.rs`: `bufferevent` | `STREAMS` retains `Rc<StreamState>` but its stream field is a raw owning pointer; tasks retain state and `BUFFERS` has weak state links. Store the stream under typed ownership and expose handles. Dispatch snapshots callbacks and rechecks liveness because callbacks can free the stream. Preserve deferred destruction and PID-sensitive fd flag restoration. This is not a foreign ownership boundary. |
| `src/spawn.rs`: editor state | Success transfers the boxed state to `window_pane.editor`; mode records observe it. Pane teardown clears its editor field before invoking completion and unlinking the temporary file. Use a pane-owned `Option<Box<_>>` plus editor/pane IDs or weak observers. Cancellation drops only the callback, intentionally keeping process/state/temp-file ownership alive until finish. |
| `src/input.rs`: input context | Pane/popup/view mode owns the context; two timer callbacks capture its address. Use a typed context owner and weak/ID timer callbacks. Teardown must release requests, cancel timers, and stop synchronized output before physical destruction. |
| `src/tty_term.rs`: terminal record | TTY owns a terminal record also linked in the global intrusive terminal list. Keep a typed TTY owner and replace list links with weak handles/IDs or a registry owner. Failure cleanup and unlinking must precede destruction. |
| `src/proc.rs`: `tmuxproc` | Constructor publishes a process-lifetime raw owner without a matching Box destructor. Signal and peer callbacks borrow it. Use an explicit process-root owner with weak/ID callbacks and ordered signal/peer shutdown; do not treat the unmatched allocation as an ordinary local pair. |

## Remaining borrowed model APIs

Removing raw ownership transfers does not complete the broader raw-pointer goal.
Layout parent/pane/sibling links, popup client/job/input/queue observers, command
and queue model references, and other client/session/window/pane links still need
borrowed APIs or typed identities. Popup's private active-guard field projections
also remain raw until its screen/input/TTY APIs can borrow those fields directly.
Environment APIs still accept legacy raw C-string arguments at some call sites.

Overlay dispatch now checks owner generations before restoring callbacks or using
their results. Cleanup can close or replace an overlay from inside any callback;
retired callbacks must not be restored onto the replacement owner.

## Tests and fixtures

`src/input.rs`, `src/mode_tree.rs`, `src/monitor.rs`, `src/spawn.rs`,
`src/window.rs` and `src/format/jobs.rs` include raw allocation fixtures for APIs
still under migration. The command-prompt lifecycle fixture now uses a boxed prepared state.
Convert remaining fixtures when their corresponding API changes; they are not
evidence of an external C ownership requirement.

`tests/fixtures/architecture/rust-deallocation.rs` intentionally contains
`Box::from_raw` as input to an architecture test and must retain that example.
