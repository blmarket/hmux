# Raw pointer field lifecycle audit

Reviewed 2026-09-27. Scope: workspace Rust struct, union, tuple and enum fields,
including raw pointers nested in collections or smart pointers, excluding
`src/compat/`. Function-pointer parameters/results are not stored data pointers.
Foreign ABI/resource fields are excluded from migration work. The offending
integration test under `tests/` has been deleted; the scanner still detects any
new raw fields added there. The requested
[remaining-field report](../../report.md) is written outside the repository.
The three supporting workspace crates contain no such fields.

The [field-by-field inventory](raw-pointer-fields.tsv) records all **320** original
fields and their lifecycle decisions. The current scanner finds **76 raw fields in
scope** and **72 excluded external ABI/resource fields**. Of the original rows,
160 explicitly record a migration and twelve record
removal. The remaining 76 comprise 5 candidates, 23 needing an access or teardown
design, and 48 retained raw under current ownership.

The remaining 76 fields have audit dispositions. Candidate and design entries
are pending work, not implemented changes.
The earlier blanket skips for Rc/RefBox observers and nonowning indexes were too
broad: inability to hold a reference does not rule out a weak handle.

A skipped candidate is not a claim that its current raw API is
safe, nor that migration is impossible. It means this review did not establish
the ownership, aliasing, and callback guarantees needed for that substitution.

Reproduce the remaining inventory and check coverage:

```sh
cargo run --quiet --example raw_pointer_fields
cargo run --quiet --example raw_pointer_fields -- --check
cargo run --quiet --example raw_pointer_fields -- --show-types
```

The scanner uses `syn`, so it distinguishes fields from local variables,
arguments, comments and literals. It includes inactive `cfg` items but does not
expand macros or resolve aliases; no pointer type aliases or macro-generated
data fields were found in the reviewed sources. The coverage check requires a
disposition for every raw field and rejects stale skipped or excluded entries.
Default output omits excluded fields. The TSV keeps exclusions and deleted-field
records so the original inventory remains traceable.
The type view prints each tracked field's current declared type or `<removed>`;
it was used to reconcile 45 previously generic `Resolved` rows and to identify
the removed redraw client field. Their owning, weak, or borrowed classifications
are now recorded individually in the TSV.

## Ownership rules used

- An existing sole owner should store its `Box` directly. Raw observers do not
  prevent that change, provided their address, invalidation and cleanup order
  are preserved. They do prevent automatically converting observers to borrows.
- A `Box` keeps an allocation stable; it does not permit a self-reference or a
  borrow surviving mutation/destruction of its owner. Parent/model links remain
  pointers where those guarantees are absent.
- References in temporary contexts must borrow actual caller-owned storage for
  a bounded operation. Do not invent a lifetime from an unbounded raw pointer.
- Static references require immutable static backing, not merely a pointer in
  a static variable. Existing reference-counted model ownership remains shared;
  this work introduces no new shared ownership, leaked allocation or lifetime cast.
- Foreign allocations retain their foreign deleters. Reactor records are Rust
  records, not foreign allocations merely because their names resemble C APIs.

## Migrations and lifecycle evidence

| Fields | Replacement and ownership evidence |
| --- | --- |
| `client.jobs`, `format_job_tree.entries` | `Option<Box<_>>` and a map of boxes. Cache creation transfers owners directly; tidy removes records before job cancellation, without retaining a map borrow across cleanup. Client loss still tidies before dropping the cache. |
| `options.tree`, `options_array_storage.entries` | Maps own boxed entries/items. Removal releases option values and monitor state in the original order before dropping each box. Back-pointers remain observers. |
| `session.options`, `window.options`, `window_pane.options` | `Option<Box<options>>`. Window/pane creation transfers boxes directly. The legacy session constructor consumes its caller's raw owning handle at the existing transfer boundary. `options::drop` drains entries in key order; model teardown takes the root at the original cleanup point. Readers project only the option field, avoiding a whole-model mutable borrow. |
| `window_pane.ictx`, `popup_data.ictx`, `window_copy_mode_data.ictx` | `Option<Box<input_ctx>>`. `input_init` returns the original allocation directly. Parser Drop drains requests, cancels both timers and stops synchronized output before deallocation. Explicit and fallback model teardown drop the parser before its screens. Timer/request model observers remain raw. |
| `input_key_entry.data` | `Cow<'static, CStr>`: literal defaults are borrowed, generated sequences own their CString. The old adjacent-owner/self-pointer pair is removed; the index retains the existing allocation/duplicate semantics. |
| `input_ctx.state`, `input_state.transitions`, `input_transition.state` | Static reference, static slice, optional static reference. All state/transition statics are now immutable; parser lookup is bounded and preserves ordered matching. |
| `layout_parse_ctx.cause` | `Option<&mut Option<CString>>` with a caller-bound lifetime. All diagnostic writes go through the context; layout-cell observers remain raw because their Box tree is mutated and transferred. |
| `redraw_build_ctx.cells` | Mutable slice borrowed from the caller's scratch owner after growth. Indexed access is bounded; the vector cannot be resized or returned to scratch storage while the context uses it. |
| `window_mode.default_format`, `C2RustUnnamed_46.command` | Optional/nonoptional static C-string references backed by immutable literals. |
| `monitor_change.name/value/last` | Borrowed C strings, with optional previous value. Name is already cloned and previous value moved into local storage before dispatch. These owners survive callbacks that remove the monitor item/set; model identities in the same record remain raw. |
| `tty.term` | `Option<Box<tty_term>>`. Creation returns the validated Box; failed construction and Drop unlink the same stable allocation from the intrusive registry. Read accessors use const raw projections; tests install actual owners. |
| `window_mode_entry.mode` | Static reference to one of nine now-immutable mode descriptors; pointer identity comparisons are retained with `ptr::eq`. |
| `hooks_data.name/formats`, `cmdq_state.formats` | Bounded name reference and `FormatTreeOwner` (optional in queue state). The owner takes its Box before draining entries, so callback capture cleanup may add entries without a whole-tree mutable borrow spanning that reentry. |
| `window_pane.editor` | `Option<Box<spawn_editor_state>>`. Completion takes ownership before invoking the callback. Drop unlinks the temporary file on completion or creation failure (cancellation keeps the owner until completion); mode editor fields remain observers. |
| Nine `options_table_entry` pointer fields | Optional static C strings, static choice slices, and optional static default-array slices. All backing data is immutable, including strings in runtime-built test descriptors. The startup initializer was removed; array iteration is bounded. |
| Three monitor indexes and their three weak traversal fields | Index values own Boxes; weak fields name the typed index. Successful insertion consumes ownership, duplicates keep the caller's allocation, and removal returns the detached owner through the legacy raw API. No map borrow spans callbacks or node destruction. |
| `StreamState.stream` | `RefCell<Option<Box<bufferevent>>>`. Free unregisters/cancels first, then takes and drops the Box outside the slot borrow. A surviving task-state Rc cannot defer physical stream cleanup. |
| `options_entry.monitor_data`, `hooks_monitor.set`, `control_state.subs` | Box-owned hook record and takeable `MonitorOwner` wrappers. Cleanup preserves sink-before-monitor and monitor-before-control-stream order. The monitor owner detaches its Box before draining timers/items/session references/callbacks. |
| `input_key_tree.entries` | Map values distinguish immutable static entries from generated boxed entries. Removed the generated-owner vector plus raw index; stable addresses and duplicate behavior are preserved. |
| `OptionCommand.0`, `cmd_parse_result.cmdlist`, `cmdq_item.cmdlist/state` | Optional existing `Rc<UnsafeCell<_>>` owners. Retain counts and explicit cleanup points are preserved. Parse results now drop their list automatically unless ownership is transferred; queue items retain their independent references. |
| `key_tables.storage`, `key_table_entry.owner`, `client.keytable` | Registry and client fields now hold their existing `Rc<UnsafeCell<key_table>>` references. Weak traversal indexes follow the typed registry. Removal transfers the registry reference back to the legacy caller; client switch/cleanup takes its owner at the former unref point. |
| `EventPayloadValue::Client/Session/Window/Pane` | Existing retained references are stored as `Rc<UnsafeCell<_>>`. Item Drop still detaches the value before invoking the original model-specific release function, preserving deferred release and last-window-close notifications. |
| `cmd_if_shell_data.client`, `cmd_load_buffer_data.client`, `cmd_run_shell_data.client`, `popup_data.c`, `format_tree.client`, `client_file.c` | `Option<Rc<UnsafeCell<client>>>` holds each existing retained reference. Drop uses deferred client release even on early exits; explicit teardown takes the owner at the original point. Readers project only a raw observer, without a client-lifetime borrow. |
| `cmd_run_shell_data.s`, `monitor_set.session` | `Option<Rc<UnsafeCell<session>>>` holds the retained session reference. Teardown takes it at the same point as before and calls `session_remove_ref` directly without storing diagnostic labels. Current/last-session observers in clients remain raw. |
| `window_switch_modedata.matches` | Row indices into the mode-owned list. Rebuild clears the indices before replacing rows; bounded filtering/sorting borrows end before rendering or commands. Filtering, ranking, list growth and replacement are covered by a regression. |
| `wait_event_item.sink`, `hooks_monitor.sink` | `EventSinkId` names a Box owned by the global event registry. Cancellation looks up the ID, so an expired registration cannot remove a replacement at a reused address. Active dispatch marks a sink dead and removes it afterward; detached sink owners are dropped outside the registry borrow. A regression covers self-removal, adding a replacement during dispatch, stale-ID cancellation, and ordinary removal. |
| `window_copy_cmd_state.args` | Removed the raw argument holder. The command state only needed the original `-F` flag, so dispatch now stores that boolean before calling the command handler. Parsed arguments remain owned by the separate `wargs` Box. |
| `window_copy_cmd_state.m` | `Option<mouse_event>` snapshots the copyable input event after the initial cursor update. Selection and scroll commands only read it; drag callbacks receive their later events separately. No raw mouse address is retained by the command state. |
| `WindowPaneModesStorage.entries` | The pane owns stable `RefBox<window_mode_entry>` entries. Insertion returns a weak handle; stack promotion transfers the sole owner, and removal detaches it before mode-specific free callbacks. A weak observer expires when that detached owner is dropped. Legacy callback views remain raw. |
| `window_pane_modes.active` | Removed the stored raw alias. `active_ptr` projects the first owned entry for bounded legacy calls, so insertion, promotion and removal cannot leave a separate active field stale. |
| `window_copy_cmd_state.wme` | A weak mode-entry observer replaces the raw command-state field. Command dispatch finds the pane-owned RefBox, and callbacks check that it is live before projecting a legacy pointer. Dispatch also stops before using saved mode data if a callback removed the entry. |

| `options_entry.tableentry` | Optional static metadata reference. Legacy constructor pointers are resolved against the immutable option table; the reference comes from that array. Removed the test-only stack descriptor. |
| `winlink.window` (now `window_owner`) | Optional `WindowOwner` stores the existing retained window reference and preserves last-close notifications. Swaps transfer owners. Release keeps the field installed through notification before taking/dropping it. Removed stack/unretained-window fixtures; a production-shaped close/retain regression covers this ordering. |

Prompt option setup now takes a mutable session borrow because style application
updates the option cache. It previously hid that mutation behind a shared session
reference and raw option pointer.

## Re-review of the remaining fields

The TSV now distinguishes four pending dispositions, all of which still identify
raw fields in the source. The external report lists every field under its current
review category, with a proposed representation, constraints and source evidence.

- **Candidate (50):** existing Rc or RefBox ownership supports weak observers or
  weak index values. This includes mode-entry pane back-pointers and cached redraw
  fields, selected models, find-state model identities and model registries.
- **Design (36):** a specific typed replacement is plausible, but needs explicit
  access/cleanup semantics: close-aware mode-tree owners, stream handles, mixed
  owner/observer queue clients, screen/editor selectors, bounded input contexts,
  snapshots and stable IDs.
- **Remove (0):** the unused bufferevent slots and reserved pane/winlink slots
  have now been deleted. They had no pointee lifecycle or external ABI users.
- **Skip (48):** there is no justified direct replacement under current ownership.
  Most targets are Box-owned queue items, options, layout cells, peers, jobs or
  intrusive links. Obtaining Weak would require changing that ownership; stable
  IDs need a separate registry design. Inline test fields are called out separately.

Weak upgrades must be held through the accesses they protect; returning a raw
pointer after immediately dropping its guard is not sufficient. RefBox borrows
must end before reentrant callbacks. Allocation liveness is distinct from logical
liveness: preserve CLIENT_DEAD/PANE_DESTROYED, session membership, mode identity,
and cache generation checks. Revalidate or snapshot when callbacks can remove a
mode or embedded child even while its parent's allocation remains alive.

Final Drop is an additional boundary. In particular, window_destroy executes with
zero strong references and destroys panes whose cleanup may still use that window;
input_ctx::drop may access a pane during final pane destruction. A plain Weak
upgrade cannot replace those accesses. The proposed changes need an explicit
teardown context or earlier logical cleanup, without adding parent/child cycles.

Libc/POSIX, ncurses and systemd ABI/resource fields remain excluded.

The first implementation after re-review migrates the five used mode pane fields
(buffer, client, customize, panes and tree) to `Weak<UnsafeCell<window_pane>>`.
Accesses hold a strong guard for their operation and reject logically destroyed
panes; menu callbacks preserve active-mode identity checks, and retained prompts
and queued refreshes retain their dead-mode checks. Cleanup does not depend on
upgrading these fields. The switch-mode pane field was write-only and was removed.
No new Rc owner or parent/child cycle was introduced.

The shared `mode_tree_data.wp` and `cmd_command_prompt_cdata.wp` now also use
weak pane links. Normal mode-tree operations hold live guards; unzoom cleanup
allows logically destroyed panes because the initial pane owner remains alive
until mode teardown finishes. An expired parent does not prevent tree cleanup.
Command prompts distinguish status prompts from expired pane prompts. Removed
the inline input-dispatch test with no owning pane; other row/menu tests no longer
install unused stack panes. New checks cover parent expiration, rejected-prompt
cleanup, and unzoom during logical destruction.

Value-only follow-up: `format_range.s` now uses the existing segment index for
range transformations, and `log_pointer` stores only an address value. Removed
the unused `bufferevent.ev_base/be_ops`, `window_pane_resizes.reserved`, and
`winlink_stack.reserved` slots and their constructor scaffolding.

The file-transfer follow-up migrates the queue's file wait, the file's waiting
client, and both file index fields to existing-owner weak handles. Lookup and
traversal return guards retained through operations. Completion unlinks the
stream before those guards can defer physical destruction. The redundant
`client_file.tree` pointer is replaced by the existing weak index link, allowing
final Drop and cleanup after head replacement without upgrading a dying file.
Empty indexes remain owned by their head; client teardown verifies the map is
empty. Removed the source-file callback fixture that assigned a stack file to
the queue's Rc-file observer. Other production-shaped lifecycle fixtures remain.

The panes mode's session observer now stores `Weak<UnsafeCell<session>>`.
Drawing and source selection retain operation guards through all session uses;
the source resolver passes its guard to the caller alongside its output pointers.
Removed sessions are rejected even while their allocations survive. Every former
`SessionOwner` holder explicitly moves its Rc into `session_remove_ref`, including
early returns, rejected upgrades, and source guard replacement. This preserves
deferred cleanup without a wrapper or stored diagnostic labels. Regressions cover
registry removal, early and normal monitor exits, and deferred physical expiration.

## Find-state and model-observer follow-up

Nine more raw fields now have typed representations:

| Fields | Representation and access |
| --- | --- |
| `cmd_find_state.s`, `.w`, `.wp` | Empty-capable `std::rc::Weak<UnsafeCell<T>>`. Saved targets do not extend object lifetimes. Validation holds upgrades while checking registry and pane membership. |
| `cmd_find_state.wl` | Empty-capable `refbox::Weak<winlink>`, cloned from a self-observer installed by `winlink_add`. Validation borrows the link and checks allocation identity at its current index; replacing a link at the same index does not revive a saved target. |
| `monitor_change.wl` | `refbox::Weak<winlink>`. Consumers hold scoped borrows; hook payload construction completes and releases its borrow before event dispatch. |
| `window_copy_cmd_state.s` | `Option<&UnsafeCell<session>>`, borrowed from an operation-local Rc. An outer dispatch function releases the Rc through `session_remove_ref` even when the inner operation returns early. |
| `window_copy_cmd_state.wl` | `refbox::Weak<winlink>` for the optional command target. |
| `WindowWinlinksStorage.ordered`, `.positions` | Ordered weak winlinks and `HashMap<WinlinkIdentity, usize>`. Identity keys contain addresses only and cannot be dereferenced. The ordered weak handles prevent address reuse while the keys remain indexed. |

Find states are `Clone`, not `Copy`. Clearing uses assignment, selected-target
copying preserves destination flags/current, and tree/customization modes clone
instead of using `memcpy` or `ptr::read`. All enclosing records use Rust
construction and destruction. An expired observer is invalid but remains distinct
from an unspecified target until explicitly cleared.

The translated call sites still use explicitly unsafe, liveness-checked pointer
views. These views do not retain models and are not a general solution for
reentrant access. Existing callback owners must remain in scope; new callback
paths must hold upgrades or take snapshots and re-resolve after dispatch. No
upgrade is created and immediately dropped just to return a raw address.

The broader review keeps the following work explicit rather than treating all
remaining model pointers as permanent raw fields:

| Remaining group | Migration direction and constraint |
| --- | --- |
| Format-tree session/window/link/pane targets | Weak targets with guards or value snapshots scoped around lazy format callbacks. A tree-wide mutable model borrow would conflict with reentrant expansion. |
| Client current/last session, session current link, winlink session | Weak identities, with logical session/link membership checks. Audit close notifications that inspect these relationships during removal. |
| Pane-to-window, input-to-pane, window active/modal/zoom panes, mode-entry panes | Weak observers for normal operations plus explicit teardown access. Window and input final destruction can use these links after the parent's strong count has reached zero. |
| Cached redraw scene/spans and layout pane links | Weak observers or resolved drawing snapshots, retaining cache-generation, layout-membership and destroyed-pane checks. Owning back-links would extend lifetimes or introduce cycles. |
| Spawn context | Borrow retained model owners across synchronous spawning; observe links weakly and revalidate after spawn notifications. Mutable model references must not span callbacks that access the same models. |
| Window index and session-group member collections | Weak values and stable identity checks. Removal must work without upgrading a model undergoing final Drop. |

The inventory also reconciles thirteen stale dispositions already migrated by
recent commits (monitor/client observers, session/pane indexes and mode-tree
owners); these are not new code changes in this follow-up.

Validation for this follow-up: `cargo test --workspace` passes **646 tests**.
Seven focused tests pass under Valgrind: three find-state regressions, three
window-link lifecycle tests, and monitor dispatch that unlinks its own target.
There are no memory-access errors or definite/indirect leaks; the processes retain
the existing 48-byte possibly-lost Rust test-harness allocation. The inventory
coverage check now reports **78** remaining in-scope fields and **72** exclusions.

## Validation

`cargo test --workspace`: **601 passed** after the panes-mode session observer migration.
The lifecycle checks include pane logical destruction while an operation guard
retains its allocation, and queued tree refresh dispatch/cancellation with an
expired parent or closed mode.
The mode observer follow-up also passes 10 focused Valgrind tests (pane teardown,
queued refresh and customize callback ownership), with no memory-access errors or
definite/indirect leaks.
The shared mode-tree/prompt follow-up passes 23 focused Valgrind tests, including
expired-pane cleanup and unzoom during destruction, with the same result.
The end-to-end range rendering regression also passes under Valgrind, covering
normal alignment and clipped ranges after the index substitution.
The file-index/wait follow-up passes eight focused Valgrind tests covering
completion with outstanding guards, final-drop unlinking, index replacement,
pane input, and source/load-buffer callbacks, without memory-access errors or
definite/indirect leaks.
The panes-mode session follow-up passes both session guard/lifetime tests under
Valgrind with no memory-access errors or definite/indirect leaks.
The updated inventory coverage check passes
for all **78** in-scope remaining fields and **72** explicit exclusions, and `git diff --check` is clean.

**38 focused lifecycle tests** also pass under Valgrind, including the editor
subprocess (`--trace-children=yes`), with no memory-access errors or
definite/indirect leaks. Each test process reports the existing 48-byte
possibly-lost Rust test-harness allocation. The final key-table and event-payload
ownership changes also pass their focused Valgrind groups. The retained-client
follow-up passes **19** focused Valgrind tests covering deferred dispatch/cancellation,
popup cleanup, format callbacks/jobs, and load-buffer cancellation. Session-owner
dispatch/cancellation and monitor teardown also pass focused Valgrind checks.
The three window-link lifecycle tests pass under Valgrind after the test-only
ownership follow-up, including last-close inspection and retaining from callbacks.

New regressions cover constructor-failure and out-of-order terminal unlinking,
stream callback captures accessing retained state during cleanup, and a queue
retaining commands after the parse result is dropped. Format-tree reentry during
capture destruction is checked for both the legacy raw API and the typed owner.

The workspace suite covers option replacement with aliased source strings,
array ordering and command retention, cache identity/expiry/client cleanup,
generated-key growth/duplicates, layout diagnostics, redraw paths, monitor
self-removal, and popup/pane lifecycle behavior. A new regression moves an input
Box into an optional owner, drops it with both timers scheduled, then runs the
reactor and verifies neither callback runs and both captures are released.

Remaining unsafe observer APIs are explicit limitations: storing owners in boxes
does not make those APIs safe or certify arbitrary reentrant use of their raw
pointers.

## Review status

The earlier conclusion that all eligible fields had been migrated is superseded.
The current inventory has 5 candidate fields, 25 design-dependent fields, and
48 retained raw fields. No candidate has been counted as migrated merely because
its proposed type exists. All previously generic `Resolved` historical rows now
name their current representation or removal.

The scanner verifies all 78 remaining declarations, including pending candidates,
against the TSV and checks the 72 explicit exclusions. It does not prove that a
candidate implementation is safe. Follow-up work must verify the producer's real
owner, guard lifetime, null/expiration behavior, reentrant invalidation, identity
semantics, and logical/final teardown before claiming a field migrated.

Test-only ownership inconsistencies are not production blockers. Remove offending
fixtures when they are the sole mismatch, as requested. Existing inline test fields
are distinguished from production fields; there are no remaining fields in tests/.
The earlier removed integration tests and 72 external fields stay outside the
remaining-field report. `src/compat/` remains unchanged.
