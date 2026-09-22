# Execute incremental ownership migration in hmux2

Migrate the easiest remaining ownership boundaries first, preserving behavior,
identity, and destruction semantics. Executing this prompt means making and
validating production code changes, not merely producing another inventory or a
proposal. Creating this document alone does not execute it.

## Agreed type policy

Choose the simplest type justified by actual ownership and aliasing:

| Existing role | Preferred representation |
| --- | --- |
| Owned NUL-terminated string | `CString` |
| Borrowed NUL-terminated string | `&CStr` |
| Owned object without escaping weak references | `Box<T>`, or inline `T` when indirection is unnecessary |
| Owned object with escaping non-owning references | `RefBox<T>` at the owner and `refbox::Weak<T>` at observers |
| Temporary access | `&T` / `&mut T` with scoped lifetimes |
| Object with an existing explicit tmux reference count | Consider `Rc<T>` only after auditing its existing retain/release semantics |
| Owned bytes or elements | `Vec<T>`, boxed slices, or an appropriate standard collection |
| Borrowed bytes or elements | Slices |
| Legitimately absent value | `Option` of the appropriate type |

- Do not introduce generational handles, replacement object-ID registries, or
  new identity semantics. Preserve existing numeric IDs and indexes.
- Do not use raw pointer addresses as identifiers or collection keys. Use
  direct ownership/observers or existing semantic IDs and names.
- Do not put everything in `RefBox`. Prefer ordinary ownership when references
  do not escape. A temporary raw pointer for a synchronous C call does not, by
  itself, require `RefBox`.
- Do not introduce `Rc` or `Arc` for objects without an existing reference
  count. An existing counter is a reason to investigate `Rc`, not proof that
  its semantics match. Document which operations retain and release ownership.
- Required weak access must assert that the target is alive, with a useful
  invariant message. Do not silently filter, skip, or default expired links.
  Inspect the installed refbox API: use its checked access/borrow equivalent
  when it has no `upgrade()` method. Treat borrow conflicts as bugs to resolve,
  not reasons to fall back to unchecked pointers. A failure may reveal a tmux
  bug or a migration bug; investigate both.
- Represent intentional absence explicitly. Remove observers at the proper
  teardown point rather than making expired weak references normal control flow.
- Keep strings byte-preserving. Do not assume UTF-8, add lossy conversion, or
  change embedded-NUL handling. Preserve null versus empty where observable.

## Start by finding the easiest current target

- Locally owned temporary strings that are created, used synchronously, and
  freed in the same operation. Check whether all consumers only borrow them.
- Small leaf records with one allocation/free pair and no retained observers.
- Detached parser scratch buffers, argument-conversion temporaries, and simple
  leaf strings in layout, JSON, format, or message code. Audit each actual
  lifetime; these modules are not presumed uniformly easy.
- The pane resize queue in `src/shared/pane.rs`, `src/window.rs`,
  `src/server_client.rs`, and `src/screen_write.rs`. It already uses boxed
  storage, so select it only if completing its lifetime removes meaningful
  manual ownership. Its callback-adjacent cleanup makes it a later candidate
  than a genuinely local string or leaf object.
- Session/winlink ownership already has partial `RefBox` support. Defer this
  graph work while simpler `CString`/`Box` targets remain.

## Implement one complete boundary at a time

1. Trace every creation, mutation, ownership transfer, alias, and destruction
   path for the selected allocation, including errors and early returns.
2. Introduce the chosen Rust owner and migrate its actual production callers.
   Remove obsolete manual frees, counts/capacities, and pointer manipulation
   where the new type supplies those responsibilities. Prefer safe operations
   with private owner fields over public raw-pointer accessors.
3. Try to preserve allocation / deallocation at a single migration - if we
   migrate one allocation, corresponding deallocation should be also migrated
   at the same commit. As compiler / runtime does not break for mixed C/Rust
   alloc/deallocs (internally they both use the same malloc/free pair), it
   should be your reasoning instead of test based.
4. Audit the containing record before adding a drop-bearing field. `repr(C)`
   alone is not the obstacle: `Copy`, bitwise copies, zero initialization,
   `xcalloc`, `realloc`, and `free` can bypass Rust invariants and destruction.
   Change that whole lifecycle together, or select a smaller target. Remove
   `Copy` when ownership moves into a record. Do not add a custom `Drop` while
   leaving its only destruction path as libc `free`.
5. Preserve required ABI signatures and layouts during a bounded increment.
   Distinguish genuine foreign boundaries from translated internal conventions;
   internal Rust APIs may change with all callers and relevant contracts.
   Keep any necessary compatibility raw pointer private or narrowly exposed,
   document its owner and invalidation rule, and keep unsafe conversion local.
   Merely renaming a raw pointer to `NonNull` or wrapping an existing collection
   without eliminating a manual lifetime responsibility is insufficient.
6. Preserve pointee address stability when aliases depend on it. Never turn
   shared/reentrant raw access into an unrestricted `&mut T`. End Rust borrows
   before callbacks, hooks, or destruction that can reenter the same object.
   Do not extend a borrow's lifetime or add unchecked interior mutability to
   get the conversion compiling.
7. Preserve destruction and notification order, cancellation, traversal order,
   duplicate-key behavior, and existing reference-count lifetime extensions.
   Changes exposed by a failing invariant require investigation and regression
   coverage, not weakening the assertion.

## Continue and leave a resumable result

Unless the invocation specifies another scope, complete up to three small,
independently validated ownership migrations per execution. Re-rank after each
one: prefer the next simple owner rather than expanding into a difficult parent
graph. Finish the current boundary and its validation before starting another.
If a candidate requires a broad redesign, record why it was deferred and pick
an easier independent candidate. Do not stop after discovery when an actionable
small target exists. If no small target remains, document the evidence and the
smallest prerequisite for the next migration rather than forcing a large rewrite.

Update the execution log below after each completed increment. Include concrete
source symbols, removed manual ownership responsibilities, tests/results, and
remaining compatibility pointers. Keep the next-candidate list current so a
later invocation resumes from source reality without repeating completed work.

The final response must identify what actually migrated, why these were the
easiest targets, validation results and limitations, and the next easiest target.
Do not claim all raw pointers are gone or that a safe wrapper proves unexamined
legacy callers safe.

## Execution log

### Increment 1 — list-keys prefix (2026-09-22)

- Ranked local list-keys prefixes first, JSON object-key scratch strings second,
  and refresh-client split scratch strings third. Resize cleanup and the
  session/winlink graph remain deferred because simpler local owners remain.
- `cmd_list_keys_get_prefix` now returns `CString`; `cmd_list_keys_exec` owns
  it through success and unknown-key error paths. The formatting helper takes
  `&CStr`. Removed three strdup branches (including a redundant copy of an
  existing CString) and both manual prefix frees.
- Audited `format_add`: its varargs formatting copies the prefix synchronously.
  The sole compatibility pointer is `prefix.as_ptr()` for that call, valid only
  while the local owner lives. No record layouts or foreign exports changed.
- Validation: key_string and key_bindings_storage tests passed; binary build
  and key_cli_checks.py passed. Extended CLI coverage checks explicit empty,
  non-UTF-8, custom, default and disabled prefixes, plus unknown-key cleanup.
- Next: JSON object-key scratch ownership, then refresh-client split scratch.

### Increment 2 — JSON object-key scratch (2026-09-22)

- `json_parse_key` now returns `Option<CString>` to its sole caller,
  `json_parse_object`. Removed the scratch strndup and both success/error frees.
  Loop scope drops the key before partial-object destruction on errors.
- Audited tokenizer spans, lookup, and all child parsers: `json_create_node`
  duplicates non-null keys. Its retained key allocation/free pair is unchanged.
  Scratch pointers from `as_ptr()` only borrow during lookup/child parsing;
  no owning field, record layout, or exported ABI changed.
- Validation: json_scratch (2), layout_custom (12), remaining_json (1) passed.
  The new scratch tests also passed against the original JSON source. They
  cover non-UTF-8 and literal escape bytes, independent retained node keys,
  duplicate keys, and partial-object failures. Baseline investigation found
  empty keys and numeric array elements already rejected; no parser behavior
  fixes were folded into this ownership change.
- Next: refresh-client split scratch strings. Graph/resize work remains deferred.

### Increment 3 — refresh-client pane argument scratch (2026-09-22)

- `cmd_refresh_client_update_offset` and `cmd_refresh_report` now share
  `cmd_refresh_parse_pane`. A local CString supplies the terminated pane prefix
  to sscanf; the suffix is a byte-preserving `&CStr` borrowed from the argument.
  Removed both whole-argument strdup/free pairs, delimiter pointer writes and
  pointer advancement. Invalid/missing prefixes return `None`.
- Audited both argument callers, control action dispatch, and tty_keys_colours.
  Only pane IDs escape parsing. Control actions do not retain the suffix;
  tty_keys_colours consumes it synchronously. The prefix drops after scanf;
  the borrowed suffix remains backed by the command argument. Numeric IDs,
  scanf's permissive syntax, action order and colour/theme updates are unchanged.
- Compatibility pointers remain only at sscanf (local CString) and
  tty_keys_colours (borrowed suffix), plus existing raw argument entry points.
  No drop-bearing record fields, retain/release operations, or ABI changes.
- Validation: new parsing regression test passed for whitespace/signs/trailing
  bytes, empty and non-UTF-8 suffixes, first-colon splitting, first-NUL behavior,
  and invalid prefixes. `cargo test --workspace` passed, including all existing
  control/client tests and all earlier increment tests. Existing compiler
  warnings remain. No sanitizer or dedicated live control-client colour-report
  scenario was run; helper tests do not establish safety of unrelated callers.

### Increment 4 — monitor subscription parse scratch (2026-09-22)

- `monitor_parse` now borrows its input as `&CStr` and splits its byte view at
  the first two colons. Removed the whole-input `xstrdup` and its success/error
  `free` paths; a short-lived `CString` terminates only the isolated target
  passed to `sscanf`, so the original numeric parsing semantics remain intact.
- The name and format outputs still use C-owned `xstrndup`/`xstrdup` storage.
  `cmd_refresh_client_update_subscription` and `cmd_set_hook_monitor_exec`
  continue to free those outputs on their existing paths, including invalid
  and unsubscribe handling. No record fields, retain/release operations, or
  exported ABI signatures changed.
- Audited all three `monitor_parse` call sites: only the duplicated name and
  format escape the helper; the borrowed input and `CString::as_ptr()` are
  used synchronously. Compatibility pointers remain at the raw exported
  `monitor_parse` boundary, the synchronous numeric `sscanf` call, and the
  C-owned output allocations.
- Validation: `cargo test --test monitor_parse` passed (2 tests),
  `cargo test --workspace` passed, `cargo build --bin hmux2` passed,
  `rustfmt --check src/monitor.rs tests/monitor_parse.rs` passed, and
  `git diff --check` passed. The repository-wide `cargo fmt --check` still
  reports the pre-existing formatting difference in `src/key_string.rs`.
  Existing compiler warnings remain; no sanitizer or live subscription command
  scenario was run.

### Increment 5 — layout serializer scratch buffer (2026-09-22)

- Replaced the private `layout_string` `dat`/`size`/`capacity` record and its
  `layout_string_init`/`layout_string_free` lifecycle with `LayoutString`, a
  `Vec<u8>` owner that maintains the temporary trailing NUL. The live
  `layout_append_v1`/`layout_append_v2` callers use scoped `&mut LayoutString`
  access.
- Removed manual serializer reallocations, capacity accounting, and raw buffer
  copies. `layout_dump` still creates its returned C string with `xasprintf`.
  The separate detached `LayoutDescription` parser and serializer, which were
  present when this increment ran, were later removed to restore tmux's direct
  `layout_cell`/`layout_parse_ctx` construction path.
- Audited the compatibility pointers: `LayoutString::as_c_ptr()` is borrowed
  only synchronously by `layout_checksum` and `xasprintf`; the variadic
  `layout_string_write` still frees only its separate `xvasprintf` temporary.
  Live layout-cell pointers and compatibility-tree cleanup are unchanged, and
  no exported ABI or layout record was changed.
- At the time, added a 4096-byte identifier regression for detached
  serialization. Those tests were removed with the detached API. Validation
  for this increment included `cargo test --test layout_custom` (13 tests),
  `cargo test --workspace`, `cargo build --bin hmux2`,
  `rustfmt --check tests/layout_custom.rs`, and `git diff --check`.
  Existing compiler warnings remain; `cargo fmt -- --check` still reports the
  pre-existing difference in `src/key_string.rs`. No sanitizer or dedicated
  live `layout_dump` integration scenario was run.

### Increment 6 — pane resize queue owner (2026-09-22)

- `window_pane_resizes::storage` now owns its queue as
  `Option<Box<VecDeque<Box<window_pane_resize>>>>`. Each resize remains in its
  own `Box`, preserving the stable node addresses used by exception-based
  cancellation while the deque grows or entries are removed. Removed the
  queue's `Box::into_raw`/`Box::from_raw` storage lifecycle and made empty
  queues explicit as `None`.
- Changed the containing `window_pane` lifecycle together with the drop-bearing
  queue field: `window_pane_create` allocates a `Box<window_pane>`,
  `window_pane_free` drops it after the existing C-owned fields are released,
  and the editor test fixture uses the same owner. `window_pane` is no longer
  `Copy`; its `repr(C)` field offsets and the two-pointer queue layout remain
  unchanged. `window_pane_destroy` still removes the resize timer before
  clearing queued entries.
- Audited the queue consumers in `src/window.rs`, `src/server_client.rs`, and
  `src/screen_write.rs`: alternate-screen cancellation still clears the queue,
  cancels the timer, and sends the current size when needed; the delayed client
  resize path snapshots queue values, releases its borrow before sending or
  clearing, and preserves the 250ms/10ms timer behavior. The remaining
  compatibility pointer is `last_ptr` in `server_client_check_pane_resize`,
  which is valid only until its paired `clear_except` call. The resize record's
  unused ABI `entry` pointers and the exported raw pane APIs remain unchanged.
- `window_pane_resize` now snapshots `old_sx`/`old_sy` after the existing sync
  stop and before screen, mode, and event callbacks. The event payload no
  longer dereferences a queue node after reentrant code may have cancelled or
  replaced it, while the original read ordering is preserved.
- Expanded `tests/pane_resize_queue.rs` to cover empty cancellation, stable
  identity, exception removal, and dropping a boxed pane with queued storage.
  Validation: focused queue/pane/layout tests passed (2 queue, 2 pane storage,
  1 layout), `cargo test --workspace` passed, `cargo build --bin hmux2`
  passed, edition-2021 rustfmt checks and `git diff --check` passed, and the
  focused queue binary passed Valgrind with no definite or indirect leaks.
  Existing compiler warnings remain; Valgrind reported only the Rust test
  harness's possible thread-local allocation. No dedicated live pane-resize
  timer integration scenario or sanitizer run was available.

### Increment 7 — session-owned winlink graph storage (2026-09-22)

- `winlinks.storage` now owns its ordered index as
  `Option<Box<OrderedIndex<i32, winlink>>>`; the index continues to own the
  `RefBox<winlink>` nodes created by `winlink_add`. Added typed boxed-index
  insertion/removal/owner-transfer helpers while retaining the raw index
  pointer only in `winlink.entry.owner` for traversal compatibility.
- `winlink_stack.storage` now owns its weak visit history as
  `Option<Box<VecDeque<Weak<winlink>>>>`. Stack traversal uses checked
  `try_access_mut` access and asserts on dropped or conflicting observers;
  `winlink_remove` tears down the session’s history entry before releasing the
  owning `RefBox`.
- Changed the containing `session` lifecycle together with its drop-bearing
  fields: `session_create` allocates a `Box<session>`, `session_free` drops it,
  and `winlinks`/`winlink_stack` are no longer manually raw-owned. Replaced
  the byte-copy transfers in `session_group_synchronize1` and
  `session_renumber_windows` with typed `ptr::replace` moves, preserving index
  and winlink addresses. The empty option representation remains pointer-sized
  and the translated record fields retain their offsets.
- Added coverage for moved winlink indexes, weak-history order and teardown,
  weak invalidation after owner drop, and dropping a boxed session with a live
  winlink owner. Compatibility pointers remain at raw C-style APIs,
  `winlink.entry.owner`, the reserved stack slot, and the existing session
  reference-count callbacks. The global `sessions`/`session_groups` indexes
  and their raw entry-owner pointers remain outside this increment.
- Validation: `cargo test --workspace` passed, focused winlink/session-history
  tests passed (6 tests), `cargo build --bin hmux2` passed,
  edition-2021 rustfmt checks for the changed files passed, and `git diff
  --check` passed. Existing compiler warnings remain; no sanitizer or live
  session-group integration scenario was run.

### Increment 8 — global sessions ordered index (2026-09-22)

- `sessions.storage` now owns its map allocation as
  `Option<Box<OrderedIndex<Vec<u8>, session>>>`. `sessions_find`,
  `sessions_nfind`, `sessions_insert`, `sessions_remove`, `sessions_minmax`,
  and `sessions_after` use the boxed-index helpers; empty-map cleanup is now
  handled by `remove_boxed` instead of the raw `Box::from_raw` path in the
  generic index lifecycle. The session records remain owned by their existing
  `session_create`/reference-counted `session_free` lifecycle.
- `session.entry.owner` remains the narrow traversal compatibility pointer. It
  is set to the stable boxed index address only after a successful insertion
  and cleared before the last map allocation is dropped. `sessions` is no
  longer `Copy`; its one-pointer representation is preserved. Server startup,
  attach-session checks, and sorted next/previous checks now use `None`/
  `Some` explicitly.
- Audited `session_destroy`: it removes the record before releasing the final
  session reference, so dropping the index never owns or destroys a live
  session record. Existing names, IDs, duplicate behavior, traversal order,
  destructive-walk successor handling, and entry-pointer invalidation are
  unchanged.
- Added the boxed-slot layout assertion and retained coverage for duplicate
  keys, moved-head traversal, rename/reinsert identity, saved-name walks, and
  complete index teardown in `tests/session_storage.rs`.
- Validation: focused session-storage tests passed (4 tests), the later full
  workspace run passed, `cargo build --bin hmux2` passed, edition-2021
  rustfmt checks for changed files passed, and `git diff --check` passed.
  Existing compiler warnings remain; no sanitizer or live server restart
  scenario was run.

### Increment 9 — global session-groups ordered index (2026-09-22)

- `session_groups.storage` now owns its map allocation as
  `Option<Box<OrderedIndex<Vec<u8>, session_group>>>`. The find, insertion,
  removal, edge, and neighbor helpers use the boxed-index view while
  `session_group.entry.owner` remains the stable raw traversal compatibility
  pointer and is cleared on successful removal.
- `session_group_new` and `session_group_remove` retain ownership of group
  records and names through their existing `xcalloc`/`xstrdup`/`free` paths;
  only the global ordered-index allocation moved to Rust ownership. Empty
  groups still remove from the index before their C-owned record is freed.
  Duplicate-key behavior, byte ordering, group reuse, and traversal order are
  unchanged.
- Validation: focused session-group tests passed (2 tests), the later full
  workspace run passed, `cargo build --bin hmux2` passed, edition-2021
  rustfmt checks for changed files passed, and `git diff --check` passed.
  No live group-command integration scenario or sanitizer run was added.

### Increment 10 — global windows ordered index (2026-09-22)

- `windows.storage` now owns its global map allocation as
  `Option<Box<OrderedIndex<u_int, window>>>`; all window index helpers use
  `boxed_ptr`, `insert_boxed`, and `remove_boxed`. `window.entry.owner` is
  documented and remains a compatibility view that is set after insertion and
  nulled on removal.
- The `window` records still use their existing `window_create` `xcalloc` and
  `window_destroy`/`free` lifecycle. No window record fields or numeric IDs
  changed. The server-start reset now drops the empty-index owner explicitly,
  while the head remains pointer-sized and movable.
- Added the boxed-slot layout assertion and retained coverage for duplicate
  IDs, numeric ordering, moved-head traversal, identity-preserving global
  lookup, and final index teardown in `tests/window_storage.rs`.
- Validation: focused window/session/session-group storage tests passed (9
  tests), `cargo test --workspace` passed, `cargo build --bin hmux2` passed,
  edition-2021 rustfmt checks for changed files passed, and `git diff --check`
  passed. Existing compiler warnings remain; no sanitizer or live window
  lifecycle scenario was run.

### Increment 11 — global all-window-panes ordered index (2026-09-22)

- `window_pane_tree.storage` now owns its allocation as
  `Option<Box<OrderedIndex<u_int, window_pane>>>`. The pane find, lower-bound,
  insertion, removal, edge, and neighbor helpers use the boxed-index view.
  Removed the raw `Box::into_raw`/`Box::from_raw` index lifecycle; pane records
  remain owned by the existing `window_pane_create`/`window_pane_free`
  lifecycle.
- `window_pane_tree` is no longer `Copy`; its one-pointer representation is
  preserved. `window_pane.tree_entry.owner` remains the narrow compatibility
  pointer, set only after a successful insertion and cleared after removal.
  Startup reset now drops the empty option owner. Normal pane destruction
  removes the pane from the global index before clearing its resize queue and
  releasing the existing pane reference, so the already drop-bearing resize
  queue does not retain an index observer.
- Added the boxed-slot layout assertion and retained coverage for duplicate
  IDs, moved-head traversal, stable pane identity, owner invalidation, and
  final index teardown in `tests/pane_storage.rs`.
- Validation: focused pane, resize-queue, remaining-pane layout, and window
  storage tests passed; `cargo test --workspace` passed; the reported
  `cargo test --no-run --message-format json-render-diagnostics --workspace`
  command passed; `cargo build --bin hmux2`, edition-2021 rustfmt checks for
  changed files, and `git diff --check` passed. Existing compiler warnings
  remain; no sanitizer or live pane lifecycle scenario was run.

### Increment 12 — per-client client-file ordered index (2026-09-22)

- `client_files.storage` now owns its allocation as
  `Option<Box<OrderedIndex<i32, client_file>>>`. The client-file lookup,
  lower-bound, insertion, removal, edge, and neighbor helpers use the boxed
  index view; the stream records remain externally allocated and retain only
  `client_file.entry.owner` as a traversal compatibility pointer.
- The containing `client` lifecycle changed with the drop-bearing field:
  `server_client_create` allocates a `Box<client>`, `client` and
  `client_files` are no longer `Copy`, and `server_client_free` asserts that
  all client streams have already removed themselves before dropping the
  client. The client-process global stream index uses the same owner and
  releases its allocation when its last stream is removed. The embedded
  `client_files` slot remains pointer-sized and the existing raw client/file
  ABI signatures are unchanged.
- Added `OrderedIndex::remove_boxed_with` so client-file owner pointers are
  cleared before the final index allocation is dropped while preserving the
  existing three-argument `remove_boxed` API used by the other migrated
  indexes. This fixes the previous attempt's five `E0061` validation errors,
  which came from changing that shared signature without updating all callers.
- Audited stream callbacks and teardown: delayed write callbacks retain only a
  client-file reference, client references keep the containing client alive,
  and the final `file_free` removes the record before its client reference can
  schedule `server_client_free`. The `client_file.tree` pointer is valid for
  the stable boxed client or process-global head; it remains a narrow raw
  compatibility pointer. Stream ordering, duplicate handling, reference
  counts, callback order, and explicit double-removal behavior are unchanged.
- Validation: `cargo test --test client_files_storage` passed (2 tests), the
  exact reported `cargo test --no-run --message-format json-render-diagnostics
  --workspace` command passed, `cargo test --workspace` passed, and
  `cargo build --bin hmux2` passed. Edition-2021 rustfmt checks for changed
  files and `git diff --check` passed. Existing compiler warnings remain; no
  sanitizer or live client teardown integration scenario was run.

### Increment 13 — format_log1 varargs scratch and index invalidation ordering (2026-09-22)

- Added the internal `xvasprintf_cstring` allocator bridge in `src/xmalloc.rs`.
  It copies the libc-allocated `vasprintf` result into a Rust `CString`, then
  releases the original with its matching libc `free`; direct
  `CString::from_raw` adoption is intentionally avoided because the foreign
  allocation has no Rust `CString` ownership provenance. The bridge preserves
  arbitrary non-UTF-8 bytes and the same first-NUL view used by the C `%s`
  consumers, and its test covers both byte preservation and embedded-NUL
  termination behavior.
- `format_log1` now owns its formatted message as a local `CString`. Its
  `as_ptr()` is borrowed only for the synchronous `log_debug` and optional
  `cmdq_print` calls, and the manual scratch `free` and raw local are gone.
  The variadic ABI, logging order, verbose output, and early logging-disabled
  return are unchanged. `format.rs` retains the raw `xvasprintf` import only
  for the existing format-tree implementation in its child module.
- Corrected the earlier boxed-index teardown ordering in `sessions_remove`,
  `session_groups_remove`, `windows_remove`, `winlinks_remove`, and
  `window_pane_tree_remove`: each now uses `remove_boxed_with` and clears its
  record's compatibility owner pointer in the pre-drop hook. The client-file
  removal already used this path. Added an `OrderedIndex` regression that
  proves the hook observes the final index allocation still alive before the
  option is cleared; existing storage tests cover the wrapper owner pointers
  after removal.
- Validation: the allocator bridge unit test, ordered-index test, and focused
  session/session-group/window/winlink/pane/client-file storage tests passed;
  `cargo test --workspace` passed; the exact
  `cargo test --no-run --message-format json-render-diagnostics --workspace`
  command passed; `cargo build --bin hmux2`, edition-2021 rustfmt checks for
  changed files, and `git diff --check` passed. Existing compiler warnings
  remain; no sanitizer or live logging/index teardown integration scenario
  was run.
- Workspace-scope audit found no `pull.log` in this repository, and this
  execution did not access or modify another repository. External history
  mentioned by an earlier review cannot be independently established from
  this workspace, so strict claims about it remain out of scope.

### Increment 14 — remove OrderedIndex and the address-keyed owner map (2026-09-22)

- Removed `OrderedIndex` from all 15 index families. Winlinks now live in one
  `BTreeMap<i32, RefBox<winlink>>`; other indexes directly use pointer-valued
  `BTreeMap`s while retaining their existing record lifetimes. No replacement
  generic wrapper or address-to-owner registry was added.
- `winlinks_reindex` moves the owner between window indexes during shuffling,
  preserving its allocation and weak history observers. Removal clears the
  compatibility map pointer before dropping the final map and owner. Existing
  next/previous APIs still use map backpointers; these pointers are not keys.
- Name-based session/group/monitor/key-table lookups borrow bytes instead of
  allocating temporary keys. Deleted wrapper-only and redundant pointer-size
  tests; added production owner, shuffle, growth, and expired-observer coverage.
- Validation: `cargo test --workspace`, binary build, key/layout CLI checks,
  and `scripts/session_cli_checks.py` passed. The session script also compares
  renumbering, shuffle/history, group synchronization, rename and teardown
  snapshots with the pinned tmux. Compiler warnings remain.
- Address-identity audit found separate existing uses in reactor `EVENTS`,
  `STREAMS`, and `BUFFERS`, plus tags in `window_tree`, `window_switch`,
  `window_client`, and `window_customize`. These have not been migrated here.
  Reactor removal requires callback cancellation/fork/owner-lifetime work;
  UI tags need semantic identities preserving selection across rebuilds.
  `hmux-rt` token IDs come from a checked counter, not pointer addresses.

### Increment 15 — options array formatted lookup key (2026-09-22)

- `options_array_getv` now owns its local formatted key as a `CString` through
  `xvasprintf_cstring`, removing its direct `xvasprintf`/`free` pair. The
  exported variadic signature and option-value owner are unchanged.
- `options_array_get` consumes only a borrowed `key.as_ptr()`: it normalizes
  into a separate allocation, looks up the item, and frees that allocation
  before returning. No formatted-key pointer escapes. The allocator bridge
  preserves arbitrary bytes and the existing first-NUL C-string view.
- Added formatted numeric, non-UTF-8, and empty-key lookups to the existing
  option-array regression. `cargo test --test options_storage` passed (3
  tests), as did `cargo build --bin hmux2`, edition-2021 rustfmt checks for
  changed files, and `git diff --check`. Existing compiler warnings remain;
  no live option-command scenario was run.

### Increment 16 — customize pane formatted display value (2026-09-22)

- `window_customize_write_value` now owns its local formatted value as a
  `CString` through `xvasprintf_cstring`, removing the direct
  `xvasprintf`/`free` pair. The only compatibility pointer is
  `value.as_ptr()` during the synchronous `screen_write_text` call.
- The early height and label returns occur before formatting. After
  formatting, `screen_write_text` creates its own text through
  `xvasprintf`/`utf8_fromcstr` before returning; it does not retain this
  value. The bridge preserves arbitrary bytes and first-NUL behavior. No
  record layout, caller, or variadic ABI changed.
- The existing allocator-bridge byte/NUL regression passed, as did
  `cargo build --bin hmux2`, edition-2021 rustfmt for the changed file, and
  `git diff --check`. Existing compiler warnings remain; no live customize
  pane scenario was run.

### Increment 17 — command-queue formatted value (2026-09-22)

- `cmdq_add_format` now owns its local formatted value as a `CString` through
  `xvasprintf_cstring`, removing its direct `xvasprintf`/`free` pair. Its
  `as_ptr()` is borrowed only for the synchronous `format_add` call.
- `format_add` duplicates the key and creates its own formatted value,
  including on duplicate-key replacement; `format_free` retains responsibility
  for that separate allocation. The command-queue format tree, config callers,
  exported variadic ABI, and byte behavior are unchanged.
- The existing allocator-bridge byte/NUL regression passed; all 59 library
  tests passed; `cargo build --bin hmux2`, edition-2021 rustfmt for the changed
  file, and `git diff --check` passed. Existing compiler warnings remain; no
  live config-command scenario or sanitizer run was performed.
- After integrating increments 15–17, `cargo test --workspace` and the binary
  build passed together; rustfmt checks for all changed Rust files and
  `git diff --check` passed.

### Increment 18 — screen_write_text formatted temporary (2026-09-22)

- `screen_write_text` now owns its local formatted `tmp` as a scoped
  `CString` through `xvasprintf_cstring`, removing its direct
  `xvasprintf`/`free` pair. `utf8_fromcstr` copies the bytes into a separate
  `utf8_data` allocation before the `CString` drops at the old free point.
- The decoded text still uses its existing two return-path frees. The only
  compatibility pointer is `tmp.as_ptr()` during `utf8_fromcstr`; exported
  variadic ABI, arbitrary bytes, and the first-NUL view are unchanged.
- The allocator-bridge regression, `cargo test --test utf8_decode`, binary
  build, edition-2021 rustfmt for the changed file, and `git diff --check`
  passed. Existing compiler warnings remain; no live screen scenario or
  sanitizer run was performed.

### Increment 19 — command argv logging prefix (2026-09-22)

- `cmd_log_argv` now owns its formatted prefix as a local `CString` through
  `xvasprintf_cstring`, removing its direct `xvasprintf`/`free` pair. Each
  `prefix.as_ptr()` is borrowed only for one synchronous `log_debug` call.
- `log_debug` either returns immediately when logging is disabled or calls
  `log_vwrite`, which formats and copies the bytes before returning. With
  `argc <= 0`, the prefix is still formatted and dropped without reading
  `argv`. Arbitrary bytes, first-NUL behavior, and the exported variadic ABI
  are unchanged.
- The allocator-bridge regression, binary build, edition-2021 rustfmt for the
  changed file, and `git diff --check` passed. Existing compiler warnings
  remain; no live logging scenario or sanitizer run was performed.

### Increment 20 — event payload logging prefix (2026-09-22)

- `event_payload_log` now owns its formatted prefix as a local `CString`
  through `xvasprintf_cstring`, removing the direct `xvasprintf`/`free` pair.
  Its `as_ptr()` is borrowed only during the synchronous `log_debug` call.
- The event buffer owns independent payload bytes, and `log_vwrite` formats
  the prefix and pulled-up bytes into its own allocation before returning.
  The buffer still frees after logging. The fatal allocation-error path still
  exits; no prefix pointer escapes. Arbitrary bytes, first-NUL behavior, and
  the exported variadic ABI are unchanged.
- The allocator-bridge regression, binary build, edition-2021 rustfmt for the
  changed file, and `git diff --check` passed. Existing compiler warnings
  remain; no live event-log scenario or sanitizer run was performed.
- After integrating increments 18–20, `cargo test --workspace` and the binary
  build passed together; rustfmt checks for all changed Rust files and
  `git diff --check` passed.

### Increment 21 — screen_write_strlen formatted scan buffer (2026-09-22)

- `screen_write_strlen` now owns its formatted message as a local `CString`
  through `xvasprintf_cstring`, removing the direct `xvasprintf`/`free` pair.
  Its read-only byte pointer remains backed by the owner through the complete
  UTF-8/control-byte scan; no pointer escapes the call.
- The existing scan algorithm, width accounting, first-NUL view, and exported
  variadic ABI are unchanged. Direct regression tests cover tab/control/DEL
  bytes, non-UTF-8 bytes, invalid and incomplete UTF-8, and first-NUL input.
- `cargo test --test screen_write_strlen` passed (2 tests), as did the binary
  build, edition-2021 rustfmt for changed files, and `git diff --check`.
  Existing compiler warnings remain; no live screen-rendering scenario or
  sanitizer run was performed.

### Increment 22 — screen_write_vnputs formatted scan buffer (2026-09-22)

- `screen_write_vnputs` now owns its formatted message as a local `CString`
  through `xvasprintf_cstring`, removing the direct `xvasprintf`/`free` pair.
  Its read-only byte pointer remains backed by the owner through the whole
  scan and synchronous screen-write calls; cells copy the selected data and
  do not retain the message pointer.
- The pointer scan, UTF-8 handling, width limit, screen writes, first-NUL
  view, and exported ABI are unchanged. There are no early returns after
  formatting; the owner drops on normal exit. This was the final direct
  `xvasprintf` call in `src/screen_write.rs`.
- `cargo test --workspace`, the binary build, edition-2021 rustfmt for the
  changed file, and `git diff --check` passed. The focused
  `screen_write_strlen` regressions also pass. Existing compiler warnings
  remain; no live rendering scenario or sanitizer run was performed.

### Increment 23 — previous window name during rename (2026-09-22)

- `window_set_name` now keeps the previous name in a local `CString`, cloned
  from the non-null C-owned `window.name` before replacing it. Removed the
  `xstrdup`/`free(last)` pair. The current name still follows its existing
  `clean_name`/`free` record lifecycle.
- `clean_name` runs before the old name is freed, preserving input aliasing.
  `window_fire_renamed` copies old and new names into its event payload before
  synchronous `events_fire`; the local old-name owner remains alive through
  reentrant callbacks. The only compatibility pointer is `last.as_ptr()` for
  that call. The `window` layout and exported ABI are unchanged.
- Added a regression for nested rename notifications, old/new payload bytes,
  final name and reference count, and invalid UTF-8 no-op. The focused test,
  `cargo test --workspace`, binary build, edition-2021 rustfmt for changed
  files, and `git diff --check` passed. Existing compiler warnings remain; no
  live rename-command scenario or sanitizer run was performed.
- After integrating increments 21–23, `cargo test --workspace` and the binary
  build passed together; rustfmt checks for all changed Rust files and
  `git diff --check` passed.

### Increment 24 — parser error formatting temporary (2026-09-22)

- `yyerror` now owns its formatted error as a local `CString` through
  `xvasprintf_cstring`, removing its direct `xvasprintf`/`free` pair.
  `cmd_parse_get_error` copies that C-string view with `xstrdup` or
  `xasprintf` before the local owner drops; its returned error remains
  C-owned. An already-recorded parser error still returns before formatting.
- The only compatibility pointer is `error.as_ptr()` during that synchronous
  copy. File/line prefixes, arbitrary bytes, first-NUL behavior, and the
  variadic calling convention are unchanged.
- Added a lexer-error regression that verifies the retained message and
  source prefix after the temporary drops. `cargo test --test
  parse_error_ownership`, the binary build, edition-2021 rustfmt for changed
  files, and `git diff --check` passed. Existing compiler warnings remain;
  no sanitizer run was performed.

### Increment 25 — command hook event name (2026-09-22)

- `cmdq_insert_hook` now owns its formatted event name as a local `CString`
  through `xvasprintf_cstring`, removing the direct `xvasprintf`/`free` pair.
  The early `CMDQ_STATE_NOHOOKS` return still occurs before formatting.
- The only compatibility pointer is `name.as_ptr()` during `events_fire`.
  That function copies the name into the event payload and dispatches sinks
  synchronously; the local owner remains alive through reentrant callbacks.
  Event name bytes, first-NUL behavior, and the exported variadic ABI are
  unchanged.
- The allocator-bridge regression, binary build, edition-2021 rustfmt for
  the changed file, and `git diff --check` passed. Existing compiler warnings
  remain; no live hook-command scenario or sanitizer run was performed.

### Increment 26 — command error formatted message (2026-09-22)

- `cmdq_error` now owns its formatted message as a local `CString` through
  `xvasprintf_cstring`, removing the direct `xvasprintf`/final `free` pair.
  Logging and each delivery branch borrow `msg.as_ptr()` only for synchronous
  calls; `cfg_add_cause`, `server_add_message`, and `status_message_set` make
  their own retained copies.
- The control/client sanitization branch copies the C-owned
  `utf8_sanitize` result into a new `CString` and releases that C allocation
  with `free`. The attached-client branch uppercases the first owned byte
  with the original `__ctype_toupper_loc` table lookup, then reconstructs the
  same NUL-terminated owner. Branch order, return value updates, arbitrary
  bytes, and the exported variadic ABI remain unchanged.
- Added `scripts/cmdq_cli_checks.py` for hook dispatch and regular/non-UTF-8
  command errors. `cargo test --workspace`, the binary build, this CLI script,
  edition-2021 rustfmt for changed files, and `git diff --check` passed.
  Existing compiler warnings remain; attached status-message and control
  client sanitization paths were source-audited but not exercised live, and
  no sanitizer run was performed.
- After integrating increments 24–26, `cargo test --workspace`, the binary
  build, and the command-queue CLI script passed together; rustfmt checks for
  all changed Rust files and `git diff --check` passed.

### Next candidates

The later layout-equivalence cleanup removed the detached
`LayoutDescription` API and its API-only tests. `layout_parse` again builds
`layout_cell` and `layout_parse_ctx` directly, as in tmux, while the
`LayoutString` serializer owner remains. `cargo test --workspace`, the binary
build, and `scripts/layout_cli_checks.py` passed; the same CLI script also
passed with the pinned tmux binary, including an ignored `I` field with a
non-string value.

1. `layout_string_write` and `args_print_add` both use `xvasprintf`'s returned
   byte length. They need a length-preserving byte owner or bridge; the
   first-NUL `CString` bridge would lose bytes after an embedded NUL.
2. `format_printf` returns a C-owned allocation, while status/message,
   input/control, environment, option, and format values escape into records
   or queues. Audit their owners and teardown before changing those calls.
   The session/winlink graph remains deferred.

Current validation is recorded in increments 15–26. The remaining
address-based registries and UI tags above are separate migration candidates.
Each of increments 15–26 has its own local commit; none was pushed.
