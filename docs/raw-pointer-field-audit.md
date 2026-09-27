# Raw pointer field lifecycle audit

Reviewed 2026-09-27. Scope: workspace Rust struct, union, tuple and enum fields,
including raw pointers nested in collections or smart pointers, excluding
`src/compat/`. Function-pointer parameters/results are not stored data pointers.
Tests and foreign ABI records are included. The requested
[remaining-field report](../../report.md) is written outside the repository.
The three supporting workspace crates contain no such fields.

The [field-by-field inventory](raw-pointer-fields.tsv) records all **320** original
fields, their disposition, and the lifecycle reason. **65 fields were migrated;
255 remain raw.** A skipped candidate is not a claim that its current raw API is
safe, nor that migration is impossible. It means this review did not establish
the ownership, aliasing, and callback guarantees needed for that substitution.

Reproduce the remaining inventory and check coverage:

```sh
cargo run --quiet --example raw_pointer_fields
cargo run --quiet --example raw_pointer_fields -- --check
```

The scanner uses `syn`, so it distinguishes fields from local variables,
arguments, comments and literals. It includes inactive `cfg` items but does not
expand macros or resolve aliases; no pointer type aliases or macro-generated
data fields were found in the reviewed sources. The coverage check requires a
disposition for every remaining field and rejects stale skipped entries.

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
| `cmd_if_shell_data.client`, `cmd_load_buffer_data.client`, `cmd_run_shell_data.client`, `popup_data.c`, `format_tree.client` | `Option<ClientOwner>` holds each existing retained reference. Drop uses deferred client release even on early exits; explicit teardown takes the owner at the original point. Readers project only a raw observer, without a client-lifetime borrow. |
| `cmd_run_shell_data.s`, `monitor_set.session` | `Option<SessionOwner>` holds the existing retained session reference and preserves deferred session release and diagnostic labels. Teardown takes it at the same point as before. Current/last-session observers in clients remain raw. |


Prompt option setup now takes a mutable session borrow because style application
updates the option cache. It previously hid that mutation behind a shared session
reference and raw option pointer.

## Why the other fields remain pointers

The TSV gives a separate decision for every field. The main blockers are:

- **Parent and cached model links:** layout parents, selected panes/winlinks,
  active-mode pointers, cached redraw spans and command-find states span owner mutation,
  reparenting or callback dispatch. Resolve stable identities before borrowing;
  neither `&'static T` nor a long-lived `&mut T` is justified.
- **Existing shared ownership:** clients, sessions, windows, panes
  and mode trees have existing retain/release contracts. Typed retained handles
  are candidates, but must preserve deferred release and last-window-close
  notifications. A `Box` would claim sole ownership incorrectly.
- **Erased or registry ownership:** mode payloads mix Box, RefBox and retained state with mode-specific callbacks. Intrusive job links combine registry membership and traversal. Those fields do not have an independent, uniformly typed owner that can simply replace each pointer.
- **Foreign or non-dereferenced addresses:** libc/ncurses/systemd records,
  fault addresses, formatting-only pointers and reserved slots have no valid
  Rust reference/Box substitution based on this review.

## Validation

`cargo test --workspace`: **598 passed**. The inventory coverage check passes
for all **255** remaining fields, and `git diff --check` is clean.

**38 focused lifecycle tests** also pass under Valgrind, including the editor
subprocess (`--trace-children=yes`), with no memory-access errors or
definite/indirect leaks. Each test process reports the existing 48-byte
possibly-lost Rust test-harness allocation. The final key-table and event-payload
ownership changes also pass their focused Valgrind groups. The retained-client
follow-up passes **19** focused Valgrind tests covering deferred dispatch/cancellation,
popup cleanup, format callbacks/jobs, and load-buffer cancellation. Session-owner
dispatch/cancellation and monitor teardown also pass focused Valgrind checks.

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
