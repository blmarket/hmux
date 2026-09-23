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

### Increment 27 — layout serializer full formatted bytes (2026-09-22)

- Added `xvasprintf_bytes`: it copies exactly `vasprintf`'s returned byte
  length into a `Vec<u8>`, including bytes after embedded NULs, then releases
  the original libc allocation with matching `free`. The vector does not
  include the extra C terminator. `layout_string_write` now appends this owned
  byte slice to `LayoutString`, removing its direct `xvasprintf`/`free` pair.
- `LayoutString::append` still supplies its own final NUL. A direct regression
  covers bytes after an embedded NUL and continued writes; another checks
  trailing-comma removal and closing output. The existing exported layout ABI
  and `layout_dump`'s returned C allocation are unchanged.
- The byte-bridge and two layout-string unit tests passed, as did the binary
  build, `rustfmt --check src/xmalloc.rs`, and `git diff --check`.
  `rustfmt --check src/layout/custom.rs` still reports pre-existing import
  formatting differences; no live layout scenario or sanitizer run was
  performed.
- This implementation was replaced in increment 29 after auditing all live
  layout serializer callers; the byte bridge and synthetic embedded-NUL test
  were removed.

### Increment 28 — configuration cause queue (2026-09-22)

- `cfg_add_cause` now formats into a `CString`, and the private
  `cfg_causes`/`cfg_ncauses` pointer array is replaced by a
  `Mutex<VecDeque<CString>>`. This removes the per-cause C allocation/free
  responsibility, manual pointer-array reallocation, count maintenance, and
  all three array teardown paths in `cfg_print_causes`, `cfg_show_causes`,
  and the test helper.
- Each cause is popped before delivery and the lock is released before
  callbacks. Reentrant additions remain in FIFO order and are drained in
  the same pass. The detached-session `cfg_show_causes` early return still
  keeps causes queued. Consumers borrow `cause.as_ptr()` only during
  synchronous formatting; exported command/notification ABI is unchanged.
- Added a regression for non-UTF-8 bytes, order, and an addition during
  delivery; shared the test lock with option-command tests that inspect
  causes. `cargo test --workspace`, binary build, edition-2021 rustfmt for
  changed files, and `git diff --check` passed. Existing compiler warnings
  remain; no live config-cause display scenario or sanitizer run was
  performed.

### Increment 29 — use CString for supported layout formats (2026-09-22)

- Audited every `layout_string_write` call: it receives fixed punctuation,
  numeric formats, a nonzero layout type (`h`, `v`, or `p`), or fixed nonzero
  bracket characters. There is no supported layout input that puts NUL inside
  a formatted fragment. The earlier embedded-NUL unit test called the private
  helper with arguments no production caller supplies.
- `layout_string_write` now uses `xvasprintf_cstring` and appends its byte view.
  Removed the unused `xvasprintf_bytes` bridge and synthetic test. The
  existing `LayoutString` still owns the final terminator and the returned
  layout dump remains C-owned. Supported layout output and ABI are unchanged.
- `cargo test --lib layout_string_tests`, the binary build, and
  `scripts/layout_cli_checks.py` passed. `rustfmt --check src/xmalloc.rs` and
  `git diff --check` passed. The pre-existing `src/layout/custom.rs`
  import-format difference remains; no sanitizer run was performed.

### Increment 30 — argument-print formatting temporary (2026-09-22)

- `args_print_add` now owns its formatted fragment as a local `CString` via
  `xvasprintf_cstring`, removing the direct `xvasprintf`/`free` pair. It still
  grows the separately C-owned result buffer and passes a borrowed
  `formatted.as_ptr()` to synchronous `strlcat`.
- Audited every production format: `%s` receives C strings, and `%c` prints
  parsed nonzero option flags at the end of its fragment. No supported user
  input puts NUL between visible bytes in a fragment. A direct unsupported
  `args_set(..., flag=0, ...)` can produce a NUL final format byte; the
  C-visible printed output stays the same, while the private spare-capacity
  count now follows the visible C-string length. The returned `args_print`
  allocation and ABI remain C-owned.
- Added a focused normal-output regression for flag and string fragments.
  `cargo test --test arguments_conversion` passed. Edition-2021 rustfmt and
  `git diff --check` passed; combined workspace and binary validation is
  recorded below. Existing compiler warnings remain.
- After integrating increments 27–30, `cargo test --workspace`, the binary
  build, and `scripts/layout_cli_checks.py` passed; changed-file rustfmt
  checks (except the pre-existing layout import difference) and
  `git diff --check` passed. The attempted `cargo test --test layout_custom`
  target does not exist in the current repository; the layout unit test and
  CLI script provided the focused checks instead.

### Increment 31 — environment entry values (2026-09-22)

- `environ_set` now formats with `xvasprintf_cstring`; `environ_storage`
  owns boxed ABI entries, `CString` names, and optional `CString` values.
  `environ_clear`, `environ_unset`, and `environ_free` release those owners
  through the index instead of manually freeing entry fields and records.
- Boxed entries keep their addresses stable for existing `environ_entry`
  pointers. The value pointer borrows the owner's buffer; null and empty
  values remain distinct. Formatting completes before replacing an old value,
  so a `%s` argument may refer to that same entry during an update.
- All 23 production `environ_set` callers use literal C formats with `%s`,
  numeric conversions, or fixed text. None supplies a supported embedded NUL
  with meaningful trailing bytes. Focused environment tests (including the old-value regression),
  `cargo test --workspace`, the binary build, changed-file rustfmt,
  `git diff --check`, and a live set/show/hidden/empty/clear/unset CLI check
  passed in the isolated worktree. Existing compiler warnings remain.

### Increment 32 — server message queue (2026-09-22)

- `server_add_message` now formats with `xvasprintf_cstring`. A private
  `VecDeque<Box<OwnedMessageEntry>>` owns stable ABI-compatible nodes and their
  `CString` text. The existing `message_log` intrusive list remains the
  traversal view. Pruning unlinks a node, checks it is the oldest owner, then
  drops that owner; startup and normal shutdown clear the collection.
- All eight production call sites pass C strings to `%s`, with one numeric
  `%u`. No supported call has meaningful bytes after an embedded NUL.
  `show-messages` order, message limits 3/0/2, and a non-UTF-8 error are
  covered by `scripts/server_messages_cli_checks.py`.
- The isolated worktree passed the CLI check, `cargo test --workspace`, the
  binary build, rustfmt, and `git diff --check`. Existing compiler warnings
  remain; no sanitizer run was performed.

### Increment 33 — control reply and deferred line strings (2026-09-22)

- `control_write` and `control_notify_write` now format with
  `xvasprintf_cstring`. Private boxed owners hold `CString` lines beside the
  existing C-layout reply and deferred records. Immediate output borrows the
  string; queued replies and notifications retain it through flush, discard,
  or stop. Guard lines route through `control_write`.
- Both `control_block` creation paths and the `control_line` creation path
  use the new wrappers. The matching free/flush paths drop their strings;
  queued reply byte accounting and intrusive list order remain intact.
  Intrusive record links and `Box::into_raw`/`Box::from_raw` lifetimes are still
  manual and remain a later ownership boundary. The records no longer derive
  `Copy`, and first-field offsets are asserted at compile time.
- Production control formats use `%s`, `%.*s`, and numeric conversions, but
  no `%c`. `capture-pane` can supply NUL-containing bytes to `%.*s`; the old
  `strlen` output path already stopped at the first NUL, as does this one.
  There is no supported E2E case that emits meaningful trailing bytes.
- Two focused unit tests, `scripts/control_lines_cli_checks.py` (reply,
  deferred notification ordering, pane output), `cargo test --workspace`,
  binary build, rustfmt, and `git diff --check` passed in the isolated
  worktree. Existing compiler warnings remain; no sanitizer run was performed.

### Increment 34 — event payload string items (2026-09-22)

- `event_payload_set_string` now formats with `xvasprintf_cstring` and stores
  the result in a private `EventPayloadStringItem` box. Its ABI-compatible
  first field exposes the unchanged `event_payload_item` pointer, while the
  union's string pointer borrows the adjacent `CString`.
- Duplicate-key replacement and payload teardown now use
  `event_payload_free_item` to drop string owners. Other item variants keep
  their existing C allocation and reference/callback cleanup. The item name,
  intrusive pointer, and tree remain manually managed.
- Production value formats are `%s` or literal `1`; getters and printers
  observe C strings, so there is no supported embedded-NUL trailing-byte E2E
  case. A focused regression covers non-UTF-8 text, string and nonstring
  replacements, pointer release callback, empty text, and payload free.
  `cargo test --workspace`, binary build, rustfmt, and `git diff --check`
  passed in the isolated worktree. Existing compiler warnings remain.

### Increment 35 — scalar option strings (2026-09-22)

- `options_set_string` now formats with `xvasprintf_cstring`. A private
  `OwnedOptionEntry` box holds a stable ABI-compatible `options_entry`, its
  `CString` name, and an optional `CString` scalar value. `options_default`,
  `options_add`, replacement/append, and `options_remove` use that lifecycle.
  `options_entry` no longer derives `Copy`.
- Scalar string and name frees are removed. Array item strings retain their
  separate C allocation/free path; entry pointers, the option tree, and its
  raw lifecycle remain compatibility boundaries. Formatting completes before
  replacing an old value, allowing `%s` to read that value during update.
  Null versus empty remains distinct; append to a null value preserves the
  existing glibc `(null)` result.
- Production formats are `%s` or `%ux%u`, with no supported meaningful bytes
  after an embedded NUL. Focused storage tests and
  `scripts/options_string_cli_checks.py` cover non-UTF-8 text, append, empty
  text, built-in options, and unset. `cargo test --workspace`, the binary
  build, rustfmt, and `git diff --check` passed in the isolated worktree.
  Existing compiler warnings remain; no sanitizer run was performed.

### Increment 36 — format entry values and keys (2026-09-22)

- `format_add` now formats with `xvasprintf_cstring`. Its private
  `FormatEntryOwner` box owns stable ABI-compatible `format_entry` storage,
  a `CString` key, and an optional `CString` value. `format_add_tv`,
  `format_add_cb`, duplicate replacement, both lazy callback cache sites,
  and `format_free` use the same lifecycle. The callback cache copies its
  C-owned result into a `CString`, releases the C allocation, and uses an
  empty string for a null result.
- Manual key/value/entry frees are removed. The tree retains compatibility
  raw pointers, with the first-field offset asserted. The distinct
  `format_printf` helper still returns a C-owned allocation to its callers.
- All 260 production `format_add` calls use literal formats with no `%c`;
  consumers observe C strings. There is no supported E2E case that needs
  bytes after an embedded NUL. Unit coverage checks replacement, old-value
  `%s` aliasing, non-UTF-8 data, callback cache/null fallback, and time value
  replacement. `tests/format_entry_owner.rs` checks lazy caching through
  public `format_expand`. A private-socket expansion check, workspace tests,
  binary build, rustfmt, and `git diff --check` passed in the isolated
  worktree. Existing compiler warnings remain; no sanitizer run was performed.

### Increment 37 — format boolean operand scratch (2026-09-22)

- `format_bool_op_n` now owns each comma-delimited operand in a local
  `CString`, borrowing its pointer only during `format_expand1`. This removes
  the `xstrdup`/`xstrndup` and matching manual `free` for the operand scratch.
  Result strings still follow the existing C-owned format expansion ABI.
- `tests/format_boolean_scratch.rs` covers multi-operand AND/OR, nested
  expansion, empty operands, and non-UTF-8 input. Focused and workspace tests,
  binary build, changed-file rustfmt, and `git diff --check` passed in the
  isolated worktree.
- Audited the remaining `format_printf` helper: all 72 production calls use
  literal numeric or `%s` formats (plus escaped percent), with no `%c` and no
  supported embedded-NUL suffix case. Its malloc-owned return crosses the
  callback ABI and is libc-freed by `format_each`/`format_replace`; the lazy
  entry cache copies then libc-frees it. A local `CString` returned as a raw
  pointer would preserve manual ownership, so this helper is deferred until
  the callback return contract can change.

### Increment 38 — client status message strings (2026-09-22)

- `status_message_set` now formats through `xvasprintf_cstring`. A private
  first-field `ClientOwner` wrapper holds the unchanged ABI `client` node and
  an optional `CString` for `message_string`. The sole production client
  constructor and final destructor allocate/drop the wrapper. Status set,
  clear, and client-loss cleanup replace or release the owner at their
  original lifecycle points; the raw message field only borrows its bytes.
- Removed the formatted message's direct `xvasprintf`/`free` pair and both
  client message frees. No pointer-key registry or raw CString ownership
  handoff was added. All 23 production formats use literal text, `%s`, or
  `%d`, and their C-string readers have no supported embedded-NUL suffix E2E.
- A focused owner test covers stable client pointer, replacement, non-UTF-8,
  empty, and a direct `status_message_clear` call.
  `scripts/status_message_cli_checks.py` exercises attached messages,
  replacement, a timer interval, and disconnect with an active
  message on a private socket. Workspace tests, binary build, rustfmt, and
  `git diff --check` passed in the isolated worktree. The timer's owner state
  and null-client branch were source-audited but lack direct live assertions;
  no sanitizer run was performed.

### Increment 39 — queued input reply strings (2026-09-22)

- `input_reply` now formats with `xvasprintf_cstring`. A private
  first-field `InputRequestOwner` keeps the ABI `input_request` pointer
  stable and owns an optional `CString` for queued replies. The `data`
  pointer borrows that buffer. All request variants use the same constructor
  and `input_free_request` destructor, covering completion, timeout,
  cancellation, and input-context teardown. Immediate replies borrow their
  local owner only during synchronous delivery.
- Removed the reply's direct `xvasprintf` and manual `free` paths. Intrusive
  request lists and Box raw-pointer record lifetimes remain compatibility
  boundaries. Production reply formats use fixed text, numeric conversions,
  or `%s` from existing C strings; none has `%c` or a supported observable
  suffix after an embedded NUL.
- Focused unit tests cover queued non-UTF-8 bytes after an earlier request is
  removed and the immediate path. `scripts/input_reply_cli_checks.py` covers
  direct CSI 6n and a queued CSI 6n delayed behind an unanswered OSC 4 palette
  query until timeout. Workspace tests, binary build, rustfmt, and
  `git diff --check` passed in the isolated worktree. No sanitizer run was
  performed.

### Increment 40 — escaped status message scratch (2026-09-22)

- `status_message_escape` now returns a byte-preserving `CString` to its sole
  caller. `status_message_redraw` borrows that pointer only while `format_add`
  copies it, removing the scratch `xmalloc` and matching `free`. The client
  message owner and format-entry ABI remain unchanged.
- The focused regression checks repeated `#`, an empty string, and non-UTF-8
  bytes. `cargo test --lib status_message_escape_tests`, the binary build,
  `scripts/status_message_cli_checks.py`, changed-file rustfmt, and
  `git diff --check` passed. The CLI exercises status messages but does not
  directly select the ignore-styles branch; the focused test covers the
  escape helper. No sanitizer run was performed.

### Increment 41 — clipboard reply base64 scratch (2026-09-22)

- `input_reply_clipboard` now uses a local `Vec<u8>` for the base64 encoder's
  output. This removes the scratch `xmalloc` and both manual `free` paths.
  The existing size guard, FFI signature, encoder result length, optional
  clipboard selector, and BEL/ST terminators remain unchanged. The vector is
  borrowed only while `__b64_ntop` writes and `bufferevent_write` copies.
- `tests/input_reply_clipboard.rs` checks binary input containing NUL, BEL
  and ST output, null/empty input, and the oversize guard. Workspace tests,
  binary build, changed-file rustfmt, and `git diff --check` passed in the
  isolated worktree. The encoder-error return remains source-audited but was
  not forced by the test; no sanitizer run was performed.

### Increment 42 — neighbouring window format keys (2026-09-22)

- `format_add_window_neighbour` now builds its three local key forms with
  byte-preserving `CString` values from C-string prefix and option names.
  This removes the `xasprintf`/`free` pairs for index, active, and user-option
  keys. `format_add` copies each key synchronously into its entry owner; the
  separate `options_to_string` result remains C-owned and is still freed.
- Production callers pass static `next` or `prev` prefixes. Workspace tests,
  binary build, rustfmt, and `git diff --check` passed in the isolated
  worktree. A private-socket three-window `#{W:...}` expansion checked
  previous/next indexes, active flags, and `@label` option values. No
  sanitizer run was performed.

### Increment 43 — OSC 4 mutable parser copy (2026-09-22)

- `input_osc_4` now owns a byte-exact, NUL-terminated `Vec<u8>` while
  `strtol` and `strsep` use scoped raw pointers into its writable storage.
  This removes the local `xstrdup`/`free` pair. The original input string
  remains available for diagnostics; palette set/query, reply terminators,
  and redraw flow are unchanged.
- `scripts/osc4_cli_checks.py` uses a live pane to set and query two palette
  entries in one OSC and query again with ST, asserting exact BEL/ST replies.
  Workspace tests, binary build, rustfmt, and `git diff --check` passed in
  the isolated worktree. Malformed numeric/color branches were source-audited
  but not forced by the live check; no sanitizer run was performed.

### Increment 44 — OSC 104 mutable parser copy (2026-09-22)

- `input_osc_104` now keeps its writable NUL-terminated parser copy in a
  local `Vec<u8>`. `strtol` advances a scoped pointer within that storage;
  normal and malformed-list exits drop the vector. This removes the local
  `xstrdup`/`free` pair without changing palette clear, redraw, or logging
  order. The empty-input clear-all branch still returns before copying.
- `scripts/input_osc_104_cli_checks.py` exercises individual and full palette
  clears, a malformed list that stops after a prior valid index, a two-index
  clear, and forwarded palette queries on a private socket. Workspace tests,
  binary build, rustfmt, and `git diff --check` passed in the isolated
  worktree. Redraw pixels and log text were source-audited rather than
  asserted directly; no sanitizer run was performed.

### Increment 45 — OSC 133 exit-status token (2026-09-22)

- `input_osc_133_exit_status` now borrows the source through `CStr`, copies
  only the first semicolon-delimited numeric token into a local `CString`,
  and passes its pointer synchronously to `strtonum`. This removes the
  `xstrdup`/`xstrndup` alternatives and their manual frees. Syntax checks,
  accepted numeric range, and exported parser behavior remain unchanged.
- A focused parser test covers missing/empty status, `=`, signed and spaced
  values, bounds, malformed and non-UTF-8 tokens, following parameters, and
  bytes after the first NUL. `tests/osc133_exit_status.rs` uses a live pane
  emitting `OSC 133;D;42;k=v` and observes `pane_command_status=42`.
  Workspace tests, binary build, rustfmt, and `git diff --check` passed in
  the isolated worktree. No sanitizer run was performed.

### Increment 46 — SGR colon parser copy (2026-09-22)

- `input_csi_dispatch_sgr_colon` now tokenizes a writable, NUL-terminated
  `Vec<u8>` copied from the parameter's first-NUL C-string view. This removes
  its `xstrdup` and the normal and two early-return `free` calls. `strsep`
  borrows stable vector storage; the original parameter remains untouched.
- Direct parser tests cover RGB, indexed color, underline, malformed and
  overflow tokens, both token-limit exits, incomplete/out-of-range RGB, and
  first-NUL handling. Workspace tests, binary build, changed-file rustfmt,
  and `git diff --check` passed in the isolated worktree. No live pane or
  sanitizer run was performed.

### Increment 47 — OSC 8 hyperlink ID scratch (2026-09-22)

- `input_osc_8` now holds its optional parsed ID in a local `CString` until
  the parser returns, removing `xstrndup` and the normal, empty-URI, and
  malformed-input frees. `hyperlinks_put` copies the ID with `utf8_stravis`
  before returning; the compatibility pointer is borrowed only during that
  call and synchronous logging. Null and empty IDs stay distinct.
- `tests/osc8_hyperlink_id.rs` drives a live pane and checks capture output
  for reused IDs, a different URI, empty ID, duplicate-ID parse error,
  missing separator, and empty-URI closure. Workspace tests, binary build,
  focused E2E test after formatting, rustfmt, and `git diff --check` passed
  in the isolated worktree. No sanitizer run was performed.

### Increment 48 — expanded format choice operands (2026-09-22)

- `format_choose` now owns the two scratch operands for its `expand != 0`
  branch as byte-preserving `CString` values, removing their `xstrndup` /
  `xstrdup` and manual `free` pairs. Both are cloned before expansion; the
  left owner drops before right expansion as before. `format_expand1` still
  returns C-owned strings to the three arithmetic, repeat, and comparison
  callers. The three `expand == 0` loop callers still receive and free their
  C-owned operands.
- Direct tests cover escaped commas, nested expressions, non-UTF-8 bytes,
  empty operands, missing delimiters, and both ownership branches. A CLI
  integration test exercises compare, repeat, arithmetic, and nested
  expansion. Workspace tests, binary build, changed-file rustfmt, and
  `git diff --check` passed in the isolated worktree. No sanitizer run was
  performed.

### Increment 49 — trimmed X11 color name (2026-09-22)

- `colour_parseX11_impl` now owns its space-trimmed name as a local,
  byte-preserving `CString`. This removes the `xstrndup`/`free` pair.
  `colour_byname_impl` compares against static names synchronously and
  returns an integer; no pointer escapes. The source span comes from
  `strlen` and cannot contain a NUL before its end.
- A focused integration test covers spaces, case, tab distinction,
  non-UTF-8, empty names, and the C ABI's first-NUL view. Workspace tests,
  binary build, changed-file rustfmt, and `git diff --check` passed in the
  isolated worktree. No live terminal or sanitizer run was performed.

### Increment 50 — startup path environment name (2026-09-22)

- `expand_path` now owns the temporary environment variable name as a
  byte-preserving `CString` in both the variable-only and slash-suffixed
  branches. This removes its `xstrdup`/`xstrndup` and `free` pair.
  `environ_find` reads the name synchronously without retaining it; the
  expanded path still has the existing C-owned return contract.
- A live-server test checks a non-UTF-8 `TMUX_TMPDIR` value in the socket
  path, `$XDG_CONFIG_HOME/tmux/tmux.conf` loading, and unset-variable
  fallback to `/tmp`. Focused and workspace tests, binary build, rustfmt,
  and `git diff --check` passed in the isolated worktree. Startup uses fixed
  variable-name literals, so the live test does not supply a non-UTF-8 name.
  No sanitizer run was performed.

### Increment 51 — format job command name (2026-09-22)

- `format_expand1` now owns its `#()` command-name scratch as a local,
  byte-preserving `CString`, removing the `xstrndup`/`free` pair. The balanced
  scan ends before the format C string's terminating NUL. `format_log1`
  reads the name synchronously; `format_job_get` copies it into its cache
  key and job record before recursive expansion or job launch. Output
  remains C-owned under the existing format ABI.
- `scripts/format_job_name_cli_checks.py` attaches a persistent PTY client,
  checks asynchronous `#()` command execution through a marker, and checks
  verbose trace parsing of nested parentheses. Workspace tests, binary
  build, the live CLI check, changed-file rustfmt, and `git diff --check`
  passed in the isolated worktree. The script does not assert cached job
  output display; no sanitizer run was performed.

### Increment 52 — drawn format style scratch (2026-09-22)

- `format_draw` now owns the bracketed style substring as a byte-preserving
  `CString`, removing its `xstrndup` and both branch-local `free` calls.
  `style_parse` copies parsed tokens into the style and hyperlink storage;
  `log_debug` only borrows the text. Explicit drops at the original free
  sites preserve release order before later style and link callbacks.
- `scripts/format_draw_style_cli_checks.py` checks a live status line with
  a valid red style, an invalid style that keeps red, and a valid green
  style. Workspace tests, binary build, changed-file rustfmt,
  `git diff --check`, and the live CLI check passed in the isolated
  worktree. No sanitizer run was performed.

### Increment 53 — fuzzy bracketed style scratch (2026-09-22)

- `fuzzy_scan` now owns its bracketed style substring as a local,
  byte-preserving `CString`, removing `xstrndup`/`free`. `format_skip`
  locates the closing bracket before the first NUL; `style_parse` copies
  tokens into local or fixed storage, and hyperlink text through
  `hyperlinks_put`, before the owner drops.
- A focused fuzzy-match test checks valid right alignment at column 7,
  invalid style fallback, and an unmatched bracket. Workspace tests,
  binary build, changed-file rustfmt, and `git diff --check` passed in the
  isolated worktree. A live format smoke check succeeded but did not
  distinguish alignment; no sanitizer run was performed.

### Increment 54 — conditional format scratch strings (2026-09-22)

- The `?` branch of `format_replace` now owns both its condition token and
  its selected true-branch substring as byte-preserving `CString` values.
  This removes their `xstrndup`/`free` pairs. Explicit drops preserve the
  original order before freeing `found`. `format_find`, nested
  `format_expand1`, and `format_log1` consume the borrowed text
  synchronously; `found` and `value` keep their C-owned return contract.
- `tests/format_condition_scratch.rs` exercises public format expansion
  with true, false, missing-key, fallback, empty, escaped-comma,
  non-UTF-8, and nested conditions. Focused and workspace tests, binary
  build, changed-file rustfmt, and `git diff --check` passed in the
  isolated worktree. No sanitizer run was performed.

### Increment 55 — parsed option array-key scratch (2026-09-22)

- `options_parse` now owns its temporary bracketed array key as a
  byte-preserving `CString`, removing the `xstrndup`/`free` pair.
  `options_array_correct_key` reads it synchronously and returns a separate
  C allocation for normalized numeric or text keys. The parsed option
  name and returned `*key` retain their C-owned contract.
- A direct parser regression covers numeric normalization, non-UTF-8
  bytes, first-NUL truncation, empty and overflow keys, and C-freeable
  outputs. Focused and workspace tests, binary build, changed-file
  rustfmt, and `git diff --check` passed in the isolated worktree.
  Repository-wide rustfmt still reports unrelated existing differences;
  no live CLI or sanitizer run was performed.

### Increment 56 — customize-mode key prompt scratch (2026-09-22)

- `window_customize_add_key_callback` now owns the parsed key-name prefix
  as a byte-preserving `CString`, removing `xstrndup` and both frees.
  `strcspn` bounds the prefix before the first NUL. `key_string_parse_cstr`
  reads it synchronously; `status_message_set` copies unknown-key text.
  The valid path explicitly drops the owner before command parsing, as
  the old code did.
- An attached-client private-socket check entered an unknown key through
  customize mode and observed `Unknown key: DefinitelyNotAKey`, then
  entered `C-9 display-message ownership-key` and observed its binding
  through `list-keys`. Workspace tests, binary build, file-specific
  rustfmt, and `git diff --check` passed in the isolated worktree. The live
  steps are not yet a committed test script; no sanitizer run was performed.

### Increment 57 — customize-mode user-option name scratch (2026-09-22)

- `window_customize_add_option_callback` now owns its parsed option-name
  prefix as a byte-preserving `CString`, removing `xstrndup`/`free`.
  `strcspn` stops before the first space, tab, or NUL. `options_match`
  copies the input through `options_parse` and returns independently
  C-owned name and optional array key; the local owner drops at the
  original free point.
- `scripts/customize_option_cli_checks.py` uses a private socket and
  attached customize-mode prompt to set `@probe` to a value with spaces,
  then checks `show-options`. It passed four runs in the isolated
  worktree. Workspace tests, binary build, changed-file rustfmt, Python
  compilation, and `git diff --check` passed. The live check covers the
  valid prompt path; no sanitizer run was performed.

### Increment 58 — startup path-list parser copy (2026-09-22)

- `expand_paths` now owns its writable, NUL-terminated colon-list copy as
  `Vec<u8>`, removing the local `xstrdup`/`free` pair. `strsep` borrows
  pointers into stable vector storage during synchronous `expand_path`
  and logging calls. The returned path array and individual paths retain
  their C-owned contract, including duplicate and error branches.
- A live-server regression checks that colon-separated HOME and XDG
  configuration paths load in order, with the latter overriding the former.
  Focused and workspace tests, binary build, changed-file rustfmt, and
  `git diff --check` passed in the isolated worktree. No sanitizer run was
  performed.

### Increment 59 — format modifier key scratch (2026-09-22)

- `format_replace` now owns its modifier-key copy as a local,
  byte-preserving `CString`, removing `xstrndup` and both exit-path frees.
  A bounded `strnlen` preserves `xstrndup`'s first-NUL behavior without
  reading beyond `keylen`. `format_build_modifiers` advances a pointer into
  the stable owner and copies modifier names and expanded arguments into
  separate allocations. Explicit drops keep the old order after modifier
  cleanup and before freeing `time_format`.
- A public-format regression checks literal and combined modifiers,
  nested modifier arguments, a high byte, and malformed repeat cleanup.
  Workspace tests, binary build, changed-file rustfmt, and
  `git diff --check` passed in the isolated worktree. No sanitizer run was
  performed.

### Increment 60 — normalized option array keys (2026-09-22)

- The private `options_array_correct_key` now returns `Option<CString>`.
  `options_array_get`, `options_array_set`, and `options_to_string` borrow
  normalized keys synchronously, removing their repeated C allocation/free
  paths. `options_array_new` copies keys retained in array items.
  `options_parse` makes a separate `xstrdup` only at its public C-owned
  `*key` transfer boundary; its callers retain the same `free` contract.
  Decimal numeric normalization and arbitrary non-UTF-8 text bytes remain.
- Focused options tests cover C-owned string output. A private-server CLI
  check covers leading-zero numeric aliases, non-UTF-8 text indexes,
  invalid empty/overflow indexes, and deletion. Workspace tests, binary
  build, changed-file rustfmt, `git diff --check`, and the live CLI check
  passed in the isolated worktree. Repository-wide rustfmt reports unrelated
  existing differences; no sanitizer run was performed.

### Increment 61 — clean-name rewrite buffer (2026-09-22)

- `clean_name` now owns its mutable, NUL-terminated copy as `Vec<u8>`,
  removing `xstrdup`/`free`. After UTF-8 validation, it rewrites `#(`
  only for untrusted names and lends the buffer to synchronous
  `utf8_stravis`; that function still returns a separate C-owned string.
  The first-NUL C-string view and output ABI are unchanged.
- `tests/clean_name.rs` covers trusted and untrusted names, repeated
  markers, trailing `#`, first-NUL handling, valid Unicode, and invalid
  UTF-8. Focused and workspace tests, binary build, changed-file rustfmt,
  and `git diff --check` passed in the isolated worktree. Repository-wide
  rustfmt reports unrelated existing differences; no live CLI or sanitizer
  run was performed.

### Increment 62 — terminal capability-name scratch (2026-09-22)

- `tty_feature_present` now owns each writable, NUL-terminated capability
  copy as `Vec<u8>`, removing its per-item `xstrdup` and both frees. It
  replaces the first `=` with NUL, then lends the prefix to synchronous
  `tty_term_has_name`, which only compares it. The separate
  `tty_parse_features` ownership boundary is unchanged.
- A focused feature-presence test covers `Ms=value`, no-`=` `AX`, a missing
  later `setaf=value`, and the early-return path. Workspace tests, binary
  build, changed-file rustfmt, and `git diff --check` passed in the
  isolated worktree. No live attached terminal or sanitizer run was
  performed.

### Increment 63 — format-loop split operands (2026-09-22)

- The session, window, and pane format loops now use a typed
  `format_choose_loop` splitter that owns the all-item operand as `CString`
  and the active operand as `Option<CString>`. This removes their
  `format_choose(expand=0)` C allocations, `xstrdup(fmt)` fallbacks, and
  manual frees. The unused unexpanded branch of `format_choose` is gone;
  its expanded arithmetic/repeat/comparison results remain C-owned.
  Nested `format_expand1` calls borrow stable pointers, and explicit drops
  preserve active-then-all cleanup order before buffer extraction.
- `scripts/format_loops_cli_checks.py` exercises live session, window,
  and pane loops with no comma, nested and escaped commas, active window
  and pane branches, and an attached-client active-session branch.
  Workspace tests, binary build, changed-file rustfmt, `git diff --check`,
  and the live CLI check passed in the isolated worktree. No sanitizer run
  was performed.

### Increment 64 — parsed window-name copy (2026-09-22)

- `parse_window_name` now owns its writable, NUL-terminated input copy as
  `Vec<u8>`, removing `xstrdup`/`free`. The stable buffer supports quote,
  `exec `, leading space/dash, slash basename, and trailing-character
  edits. `__xpg_basename` may write within it; `clean_name` reads it
  synchronously and returns a separate C-owned string. An explicit drop
  preserves the old release point before the empty-name fallback.
- A direct C ABI test covers those parser branches, first-NUL behavior,
  invalid UTF-8 fallback, empty input, and caller `free`. Workspace tests,
  binary build, changed-file rustfmt, and `git diff --check` passed in the
  isolated worktree. Repository-wide rustfmt reports unrelated existing
  differences; no live rename or sanitizer run was performed.

### Increment 65 — terminal feature-list parser copy (2026-09-22)

- `tty_parse_features` now owns its writable, NUL-terminated feature list
  as `Vec<u8>`, removing its `xstrdup`/`free` pair. `strsep` and trailing
  `@` removal write within stable vector storage; feature comparisons and
  logging borrow token pointers synchronously. Unknown-feature break and
  first-NUL behavior are unchanged.
- `tests/tty_parse_features.rs` exercises mixed `:` and `,` separators,
  enable-then-remove, disabled suppression, unknown-feature break, and
  first-NUL handling. Focused and workspace tests, binary build,
  changed-file rustfmt, and `git diff --check` passed in the isolated
  worktree. No live attached terminal or sanitizer run was performed.

### Increment 66 — UTF-8 width-cache parser copy (2026-09-22)

- `utf8_add_to_width_cache` now owns its writable, NUL-terminated input
  copy as `Vec<u8>`, removing `xstrdup` and all early/success-path frees.
  The stable buffer is split at `=`; `strtonum`, `strtoull`, and
  `utf8_fromcstr` borrow it synchronously. Cache entries retain only
  numeric codepoint and width values. `errno`, `utf8_no_width`, and the
  separate decoded-array free order are unchanged.
- A focused unit test covers single codepoints, ranges, UTF-8 characters,
  invalid widths/ranges/strings, missing `=`, and first-NUL handling;
  it removes entries inserted during the test. Workspace tests, binary
  build, changed-file rustfmt, and `git diff --check` passed in the
  isolated worktree. No sanitizer run was performed.

### Increment 67 — options array assignment split copy (2026-09-22)

- `options_array_assign` now owns its mutable `strsep` input as `Vec<u8>`
  copied through `CStr::to_bytes_with_nul`. The old `xstrdup` and both
  success/error `free` paths are gone. `options_array_set` copies string
  tokens or parses command/colour values before returning; it retains no
  pointer into the split buffer. Empty tokens, first-NUL parsing, and the
  partial array left after an error are unchanged.
- `tests/options_storage.rs` covers mixed separators, non-UTF-8 tokens,
  first-NUL handling, stable stored values after input mutation, and a
  colour error after one successful element. Focused and workspace tests,
  binary build, changed-file rustfmt, `git diff --check`, and
  `scripts/options_array_key_cli_checks.py` passed in the isolated worktree.
  No sanitizer run was performed.

### Increment 68 — client-target lookup copy (2026-09-22)

- `cmd_find_client` now reads the first-NUL target bytes, removes one
  optional trailing colon, and owns the lookup/error string as `CString`.
  This replaces `xstrdup`, the writable terminator, and the matching `free`.
  Client names and terminal paths borrow the string only during synchronous
  comparisons; the error formatter copies it before the owner drops at the
  former free point. The NULL-target branch still uses the current client.
- `scripts/find_client_cli_checks.py` checks a live control client's name
  with and without a trailing colon and exact missing-client errors for
  empty, colon-only, non-UTF-8, and internal-colon inputs. Workspace tests,
  binary build, CLI check, changed-file rustfmt, and `git diff --check`
  passed in the isolated worktree. The CLI cannot pass embedded NUL in an
  argument; CStr retains the previous first-NUL behavior. No sanitizer run
  was performed.

### Increment 69 — client flag-list scratch (2026-09-22)

- `server_client_set_flags` now owns its writable comma-list as `Vec<u8>`
  from `CStr::to_bytes_with_nul`, replacing `xstrdup` and the matching
  `free`. `strsep` still mutates the stable buffer, and the flag parser,
  logging, and reset call borrow tokens only during the loop. The Vec drops
  at the former free point before `proc_send`; first-NUL and non-UTF-8
  bytes retain their C-string behavior.
- `tests/client_flag_list.rs` runs a private server with a live control
  client. It verifies setting two comma-delimited flags, ignoring empty and
  unknown tokens, and clearing only `ignore-size` with `!ignore-size`.
  Focused and workspace tests, binary build, changed-file rustfmt, and
  staged `git diff --check` passed in the isolated worktree. No sanitizer
  run was performed.

### Increment 70 — candidate session list (2026-09-22)

- `cmd_find_best_session_with_window` now gathers borrowed session pointers
  in a local `Vec`, replacing its `xreallocarray` growth, explicit count,
  and both `free` paths. `cmd_find_best_session` borrows the array only for
  its synchronous ranking pass. The selected session is stored before the
  Vec drops at the former free point; winlink lookup follows that drop.
  The zero-candidate branch still skips selection.
- `tests/cmd_find_session_list.rs` uses a private server to check lookup of
  a window owned by one session, shared by two sessions, and left with one
  session after unlinking. Workspace tests, binary build, changed-file
  rustfmt, and `git diff --check` passed in the isolated worktree. A live
  window cannot exercise the zero-member branch; no sanitizer run was done.

### Increment 71 — split target lookup buffer (2026-09-22)

- `cmd_find_target` now owns the first-NUL, writable target copy as a local
  `Vec<u8>`, replacing `xstrdup` and the success/error `free` pair. Session,
  window, and pane components may alias this buffer; `cmd_find_map_table`
  returns either a static mapping or its input, and all later lookup and
  error consumers use the aliases before the Vec drops at the former free
  point. Numeric IDs and target selection are unchanged.
- `scripts/find_target_cli_checks.py` covers session:window.pane lookup,
  exact-session and abbreviated targets, all three component error paths,
  and non-UTF-8 error bytes. Workspace tests, binary build, CLI check,
  changed-file rustfmt, and staged `git diff --check` passed in the isolated
  worktree. CLI arguments cannot contain NUL; the CStr copy preserves the
  previous first-NUL view. No sanitizer run was performed.

### Increment 72 — clipboard reply base64 scratch (2026-09-22)

- `tty_keys_clipboard` now copies its counted encoded bytes into a local
  `Vec<u8>` with a trailing NUL. `__b64_pton` borrows that stable buffer;
  the old `xmalloc`/`memcpy` and three `free(copy)` paths are gone. On
  success the Vec drops before logging and `input_request_reply`, as the
  old copy did. Invalid and empty decode paths retain their cleanup order.
  The separate decoded `out` allocation and paste-buffer transfer are
  unchanged.
- A PTY test in `tests/bracketed_paste_pty.rs` requests clipboard data and
  checks decoded `A\0B`, then sends an encoded response with a middle NUL
  and checks that the decoder still sees only the prefix. Focused and
  workspace tests, binary build, changed-file rustfmt, and `git diff
  --check` passed in the isolated worktree. The test does not exercise the
  invalid-base64 decode branch; no sanitizer run was performed.

### Increment 73 — run-shell trailing line copy (2026-09-22)

- `cmd_run_shell_callback` now copies a counted trailing, unterminated
  evbuffer line into `Vec<u8>` and appends a NUL for `cmd_run_shell_print`.
  This removes its `xmalloc`, `memcpy`, and matching `free`; the Vec drops
  after the synchronous print, at the former free point. Complete lines
  returned by `evbuffer_readln` retain their distinct libc ownership.
  Embedded NUL still ends the C-visible printed line.
- `tests/run_shell_partial_line.rs` runs a private server and checks a
  complete line followed by a partial line, then a partial line containing
  NUL and hidden suffix bytes. Focused and workspace tests, binary build,
  changed-file rustfmt, and `git diff --check` passed in the isolated
  worktree. No sanitizer run was performed.

### Increment 74 — UTF-8 C-string membership scan (2026-09-22)

- `utf8_cstrhas` now decodes each candidate into a stack `utf8_data`
  instead of allocating and freeing the full `utf8_fromcstr` array. It
  preserves `utf8_open`/`utf8_append` recovery for malformed sequences,
  compares only the recorded cell bytes, and continues through the entire
  input after a match so suffix decoding effects remain. The target borrow
  is scoped after each decode call; C input still ends at its first NUL.
- `tests/utf8_decode.rs` compares membership against cells produced by
  `utf8_fromcstr` for ASCII, multibyte, malformed, truncated, non-UTF-8,
  and first-NUL inputs, with negative and suffix cases. Workspace tests,
  binary build, changed-file rustfmt, and `git diff --check` passed in the
  isolated worktree. No sanitizer run was performed.

### Increment 75 — show-environment shell escape (2026-09-22)

- The private `cmd_show_environment_escape` now builds and returns a
  `CString` instead of an `xmalloc` buffer. `cmd_show_environment_print`
  borrows its pointer for synchronous `cmdq_print` and no longer frees it.
  Escaping of dollar signs, backticks, quotes, and backslashes, non-UTF-8
  bytes, and the first-NUL view are unchanged. Cleared (NULL) and empty
  values still take distinct print paths.
- `tests/show_environment_shell.rs` checks live shell-form output for all
  four escapes and empty versus cleared entries. A direct helper test
  checks non-UTF-8 and first-NUL bytes because command argument parsing
  normalizes non-UTF-8 before this helper. Focused and workspace tests,
  binary build, changed-file rustfmt, and `git diff --check` passed in the
  isolated worktree. No sanitizer run was performed.

### Increment 76 — width-cache character decode (2026-09-22)

- `utf8_add_to_width_cache` now parses its character operand into a stack
  `utf8_data`, removing the local `utf8_fromcstr` decoded array and four
  manual frees. It retains the existing `utf8_no_width` toggle, retries
  malformed candidates as the decoder did, and accepts only one complete
  cell before inserting a width. Its already Rust-owned writable input
  copy remains in place; the `U+` codepoint branch is unchanged.
- The existing parser test now covers ASCII, a valid multibyte character
  followed by another cell, malformed and truncated sequences, overlong
  bytes, and first-NUL input. Focused and workspace tests, binary build,
  changed-file rustfmt, and `git diff --check` passed in the isolated
  worktree. No sanitizer run was performed.

### Increment 77 — source-file glob pattern (2026-09-22)

- `cmd_source_file_exec` now owns each absolute or cwd-prefixed glob
  pattern as `CString`, replacing its per-path `xstrdup`/`xasprintf` and
  matching frees. It builds relative paths from C-string bytes without
  UTF-8 conversion. Logging and `glob` borrow the pointer; on success it
  drops before iterating `g.gl_pathv`, and on error it drops after
  `globfree`, matching the prior order. `cwd`, `expanded`, and asynchronous
  file-path copies retain their existing ownership.
- `tests/source_file_pattern.rs` uses a private server to cover absolute
  paths, relative paths with glob characters in the cwd, two matched glob
  files, no-match errors, and quiet no-match. Workspace tests, binary
  build, changed-file rustfmt, and `git diff --check` passed in the
  isolated worktree. No sanitizer run was performed.

### Increment 78 — command-prompt split scratch (2026-09-22)

- In `cmd_command_prompt_exec`, the nonliteral comma-split prompt and
  input copies now live in stable `Vec<u8>` buffers rather than local
  `xstrdup`/`xasprintf` allocations and frees. `strsep` mutates those
  buffers; each stored prompt/input token remains separately C-owned.
  The `-l` branch still transfers C-owned whole strings into `cdata`.
  The input Vec drops before the prompt Vec at the former two-free point,
  before prompt type checks and callback setup. First-NUL and empty-token
  behavior are unchanged.
- Workspace tests, binary build, changed-file rustfmt, and `git diff
  --check` passed in the isolated worktree. A live attached PTY check
  used `-p First,,Third, -I one,,three,` with four Enter keys and saw
  `one//three/`; the `-l -p Literal,Prompt -I seed,tail` check saw
  `seed,tail`. This PTY check was run manually rather than added as an
  automated test. No sanitizer run was performed.

### Increment 79 — source-file cwd glob quoting (2026-09-22)

- `cmd_source_file_quote_for_glob` now returns `CString` built from the cwd's
  C-string bytes. Its sole caller, `cmd_source_file_exec`, borrows those bytes
  for relative glob patterns and drops the owner at the former `free(cwd)`
  point. This removes the helper's `xmalloc` buffer and caller's manual free.
  ASCII glob punctuation is escaped with the same ctype check; non-ASCII bytes
  remain unchanged. The asynchronous file list still receives its own copies.
- The private-server `source_file_pattern` test passed with a cwd containing
  brackets, plus, and UTF-8 bytes. `git diff --check` passed. No sanitizer run
  was performed.

### Increment 80 — hook-monitor target string (2026-09-22)

- `cmd_show_hooks_print_monitor` now builds its local target as
  `Option<CString>`, removing three `xstrdup` branches, two `xasprintf`
  branches, and the matching `free`. `format_add` copies the borrowed string
  synchronously. The unknown monitor type still passes a null pointer, and
  the owner drops after `free(line)` and before `free(value)` as before.
- `cargo check --bin hmux2`, the binary build, and `git diff --check` passed.
  A live server check exercised `set-hook -B` and `show-hooks -B -F` for
  session, pane, all panes, window, and all windows targets. No automated E2E
  test or sanitizer run was added. After both increments, the combined main
  branch passed `cargo test --workspace --quiet`, `cargo build --bin hmux2
  --quiet`, and changed-file `rustfmt --check`.

### Increment 81 — display-message local message (2026-09-22)

- `cmd_display_message_exec` now keeps its literal, expanded, and JSON
  message as one local `CString`. The literal branch borrows `template`
  directly instead of `xstrdup`. A private adapter copies the first-NUL
  C-string view from the C-owned `format_expand_time` and `json_to_string`
  outputs, then frees each foreign allocation with libc. The three former
  `msg` frees are now owner drops at the same error, JSON replacement, and
  final output points. JSON nodes, causes, and the format tree retain their
  existing teardown order. Output routines borrow `msg.as_ptr()` only for
  synchronous calls.
- The binary build, changed-file rustfmt, and `git diff --check` passed.
  A private live server check covered literal output, format expansion,
  valid JSON with an escaped NUL, and invalid JSON. The CLI normalized the
  non-ASCII inputs attempted before this path, so those checks do not prove
  byte preservation for such input. No sanitizer run was performed.

### Increment 82 — capture-pane hyperlink ID scratch (2026-09-22)

- `cmd_capture_pane_history` now owns its `-H` hyperlink deduplication IDs in
  `Vec<u_int>`. `cmd_capture_pane_hyperlinks` borrows that Vec while scanning
  each line. This removes the `xreallocarray` allocation, manual count and
  indexed writes, and final `free`; the Vec drops at the former free point.
  The original `gd.sx` bound, first-seen order, and cross-line deduplication
  remain. The output line and buffer retain their existing C ownership.
- The live OSC 8 test now covers an ID repeated across lines and a newly seen
  ID on the next line. Focused `osc8_hyperlink_id` tests, changed-file rustfmt,
  and `git diff --check` passed in the isolated worktree. No sanitizer run was
  performed.

### Increment 83 — copy-mode cell bytes (2026-09-22)

- `window_copy_cellstring` now returns `Cow<[u8]>`: static and ordinary grid
  bytes are borrowed, while decoded extended-cell bytes are owned by Vec.
  Its two callers, `window_copy_stringify` and
  `window_copy_cstrtocellpos`, consume that owner directly. The latter keeps
  a `Vec<Cow<[u8]>>` through regex byte-to-cell mapping. This removes the
  extended-cell `xmalloc`/`memcpy`/per-cell `free` pair, the manual allocated
  flags and lengths, and the C array allocation/free. A local `utf8_data`
  also replaces the shared static decode scratch. Zero-size cells retain an
  empty byte view; the grid and static byte views are not copied.
- A new live copy-mode test checks forward and backward regex search and
  cursor positions across multibyte and wide cells. The focused test,
  binary check and build, changed-file rustfmt, and `git diff --check`
  passed in the isolated worktree. After all three increments, the combined
  main branch passed `cargo test --workspace --quiet`, the binary build,
  changed-file rustfmt, and `git diff --check`. No sanitizer run was
  performed.

### Increment 84 — fuzzy-match scratch arrays (2026-09-22)

- `fuzzy_match_fuzzy` now owns its backtracking positions as `Vec<u_int>`.
  `fuzzy_match` owns its matched flags, best flags, and decoded-token slots
  as Vecs. This removes four local calloc/reallocarray allocations and their
  corresponding frees, as well as the explicit memset and memcpy between
  matched and best flags. Early no-match returns drop the position Vec; the
  remaining explicit drops follow the former free order before the C-owned
  `fuzzy_scan` result is freed. The libc-owned result mask retains its ABI.
- Focused fuzzy tests passed, including repeated-character backtracking,
  alternative group ordering/reset, and no match. Changed-file rustfmt and
  `git diff --check` passed in the isolated worktree. No sanitizer run was
  performed.

### Increment 85 — show-options local value (2026-09-22)

- `cmd_show_options_print` now keeps its value as a local `CString`. Empty
  arrays use `CString::default()` instead of `xstrdup("")`; scalar and keyed
  array values copy the first-NUL view returned by C-owned
  `options_to_string`, then free that foreign allocation at the boundary.
  `format_add` copies the borrowed value synchronously, and the local owner
  drops after `free(line)` at the former value-free point. Recursive array
  traversal and the empty-array `-v` return still occur before value creation.
- A new live test covers string and number scalar values, empty arrays under
  default, `-v`, and custom templates, recursive two-item arrays, and a
  keyed item. The focused test, binary build, changed-file rustfmt, and
  `git diff --check` passed in the isolated worktree. No sanitizer run was
  performed.

### Increment 86 — confirm-before prompt (2026-09-22)

- `cmd_confirm_before_exec` now builds both custom and default prompts
  directly as `CString` from borrowed C-string bytes and literal bytes.
  This removes the local `xasprintf` allocation and matching free. The
  existing `-c` validation ensures the inserted confirm key is one printable
  ASCII byte. `status_prompt_set` reaches `prompt_create`, which duplicates
  the prompt synchronously; the local owner drops at the former free point.
- Binary check and build, changed-file rustfmt, and `git diff --check`
  passed in the isolated worktree. A manual attached-PTY check displayed
  custom and default prompts and confirmed both commands ran. No automated
  E2E test or sanitizer run was added. After all three increments, the
  combined main branch passed `cargo test --workspace --quiet`, the binary
  build, changed-file rustfmt, and `git diff --check`.

### Increment 87 — remove unused window-switch pointer tag (2026-09-22)

- `window_switch_itemdata.tag` and its two pointer-to-integer assignments
  were never read; selection uses the stored session ID and winlink index.
  Removed the dead field and assignments, eliminating this mode's unused
  address-derived tag. This is a cleanup of a false identity field, not an
  ownership migration or a replacement ID scheme. No owner or teardown path
  changed.
- `cargo check --bin hmux2 --quiet`, changed-file rustfmt, and
  `git diff --check` passed. No separate runtime test or sanitizer run was
  performed.

### Increment 88 — fuzzy scan character array (2026-09-22)

- Private `fuzzy_scan` now returns `Vec<fuzzy_char>` and `fuzzy_add` pushes
  each copied UTF-8 cell. Its sole caller, `fuzzy_match`, takes the count
  from the Vec, lends a stable pointer only during matching, and drops the
  owner at both former `free(cs)` sites. This removes the growable
  `xreallocarray` buffer, manual count and capacity bookkeeping, and both
  frees. Alignment widths, scan order, and the libc-owned result mask remain.
- A focused test crosses the former 64-cell growth boundary and confirms
  first-NUL behavior. Focused fuzzy tests, binary build, changed-file
  rustfmt, and `git diff --check` passed in the isolated worktree. No
  sanitizer run was performed.

### Increment 89 — format grid-line cell scratch (2026-09-22)

- `format_grid_line` now collects copied `utf8_data` cells in a local Vec,
  appends the same size-zero terminator, and drops the Vec after
  `utf8_tocstr` has made the caller-owned C string. This removes its
  per-cell `xreallocarray` growth, manual count, and matching free. Empty
  lines still return null; tabs, padding, and UTF-8 cell bytes retain their
  existing conversion paths and order.
- A new grid test covers an empty line, ASCII, a tab, a wide UTF-8 cell with
  padding, and returned-string lifetime after grid destruction. The focused
  test and `git diff --check` passed in the isolated worktree. The new test
  file passes rustfmt; whole-file rustfmt for `src/format.rs` reports
  preexisting import-order differences outside this increment. No sanitizer
  run was performed.

### Increment 90 — list-keys prefix/root merge storage (2026-09-22)

- Private `cmd_list_keys_get_root_and_prefix` now returns a Vec of binding
  pointers instead of reusing a process-static `xreallocarray` buffer with a
  manual capacity. It copies each table's sorted scratch output before
  asking the sorter for the next table, preserving prefix-then-root order.
  Its sole caller, `cmd_list_keys_exec`, retains the Vec through filtering,
  width and repeat scans, formatting, and output, then drops it after
  `format_free`. An empty Vec supplies a null compatibility pointer when the
  count is zero. The `-T` and all-key paths retain their existing sort
  buffers.
- A live private-server test covers both tables, repeated listing, key
  filtering, `-T`, and one empty table. Focused test, binary check,
  changed-file rustfmt, and `git diff --check` passed in the isolated
  worktree. Listing with both prefix and root tables empty exits the server
  on both the unmodified baseline and migrated binary; that existing failure
  is outside this owner change and is not covered by the new test.
  `key_bindings_get_table` can return null after a table is removed, while
  `sort_get_key_bindings_table` dereferences its table argument; this is the
  likely cause and needs a separate behavior fix. After all three owner
  increments, the combined main branch passed `cargo test --workspace
  --quiet`, the binary build, formatting checks for changed files other
  than `src/format.rs`, and `git diff --check`. No sanitizer run was
  performed.

### Increment 91 — format grid-word cell scratch (2026-09-22)

- `format_grid_word` now collects copied `utf8_data` cells in a local Vec,
  appends the same size-zero terminator, and drops the Vec after `utf8_tocstr`
  makes the caller-owned C string. This removes its per-cell `xreallocarray`,
  manual count, `memcpy`, and matching free. An empty word still returns null;
  separator and padding traversal are unchanged.
- A focused test covers two words, a wide cell with padding, empty output, and
  returned-string lifetime after grid and option teardown. The focused grid
  tests, binary build, changed-file rustfmt, and `git diff --check` passed in
  the isolated worktree. No sanitizer run was performed.

### Increment 92 — prompt history entries and list (2026-09-22)

- `prompt_hlist` now owns each type's history as `Vec<CString>`. This removes
  the separate count, pointer-array `xreallocarray`/`memmove`, entry `xstrdup`,
  and manual frees on pruning and clearing. `prompt_history_get` and navigation
  lend pointers into CString allocations; moving the Vec leaves these buffers
  stable until that entry is pruned or cleared. Save/load order is unchanged.
- `prompt_add_history` copies an incoming borrowed entry before pruning it,
  avoiding the prior use-after-free if the argument aliases the oldest entry.
  Limit-zero and duplicate-at-limit behavior remain. The focused test covers
  pointer stability across growth, non-UTF-8 bytes, duplicates, pruning,
  navigation, alias input, and save/reload. Focused test, `cargo check`,
  changed-file rustfmt, and `git diff --check` passed in the isolated
  worktree. No sanitizer run was performed.

### Increment 93 — window-tree semantic selection keys (2026-09-22)

- `window_tree` now uses typed `ModeTreeIdentity` values for session IDs, pane
  IDs, and `(session ID, winlink index)` pairs. This removes its pointer-address
  tags from item construction, requested selection, mouse/H navigation, and
  expansion. `mode_tree` matches these keys when rebuilding saved tagged and
  expanded state, and when restoring current selection or search results.
  Legacy numeric-tag APIs remain for the other modes; no replacement object
  registry or new object ID was introduced. Pane IDs remain global as before.
- The Rust-owned mode-tree record layout and its model fixture changed only
  for the typed identity and build-selection slot; no foreign callback
  signature changed. A direct state test covers key-domain and winlink-pair
  separation, saved tag/expansion restoration, and current selection.
  Workspace tests, binary build, changed-file rustfmt, and `git diff --check`
  passed in the isolated worktree. A detached `choose-tree` smoke reached
  tree mode, but screen capture did not render and is not behavioral proof.
  No sanitizer run was performed.

### Increment 94 — choose-buffer preview line scratch (2026-09-22)

- `window_buffer_draw` now owns its per-line `utf8_strvis` output in a local
  `Vec<u8>`, resized to the same `4 * (line length + 1)` bound. This removes
  the loop's `xreallocarray` and final `free`. `utf8_strvis` and
  `screen_write_nputs` borrow its pointer only during each synchronous call;
  line order and first-NUL display behavior are unchanged.
- An attached-terminal test renders a long buffer line followed by a short
  line with an escaped tab. The focused test, workspace suite, binary build,
  changed-file rustfmt, and `git diff --check` passed in the isolated
  worktree. No sanitizer run was performed.

### Increment 95 — session-group node and name owner (2026-09-22)

- The `session_groups` index now owns each group as `Box<SessionGroupOwner>`:
  a stable C-layout node plus a byte-preserving `CString` name. This removes
  `session_group_new`'s `xcalloc`/`xstrdup` and the last-member `free` pair.
  Internal insert consumes the owner; removal drops it after the existing
  group event and after detaching the final session. Borrowed group pointers
  remain valid while indexed, and byte-order traversal is unchanged.
- Updated the focused storage test for owned insertion/removal, duplicate
  names, outsider non-removal, stable pointers, and non-UTF-8 borrowed input.
  Workspace tests, binary build, session CLI reference comparison,
  changed-file rustfmt, and `git diff --check` passed in the isolated
  worktree. No sanitizer run was performed.

### Increment 96 — customize-tree semantic selection keys (2026-09-22)

- `window_customize` now gives every option, array, key table/binding/field,
  environment, and section row an exact semantic `ModeTreeIdentity`. The
  identities use section numbers, key codes, and byte-preserving names/array
  keys. This removes all pointer-address mode-tree tags in this mode without
  hashes or generated object IDs. A row copies any named key and frees it
  with the row; build selection and search copy names while old rows are
  destroyed. Legacy numeric mode-tree clients remain supported.
- Direct tests cover non-UTF-8 and first-NUL name handling, section and
  array separation, key fields, saved tagging/expansion, and selection after
  a full rebuild. Workspace tests, binary build, changed-file rustfmt, and
  `git diff --check` passed in the isolated worktree. A detached
  `customize-mode` smoke entered options mode, but its render capture did
  not prove row behavior. No sanitizer run was performed.

### Increment 97 — choose-client item list and rows (2026-09-22)

- `window_client_modedata` now owns a `Vec<Box<window_client_itemdata>>`.
  Each stable boxed row owns its tty name as `CString` and releases its
  retained client in `Drop` before releasing the name, matching the former
  `window_client_free_item` order. The mode data itself is Box-owned so the
  Vec drops at teardown after `mode_tree_free`. This removes the list's
  `xreallocarray`, per-item `xcalloc`, copied-name `xstrdup`, manual item
  frees, count, and list free. The client pointer tag is unchanged.
- A focused test covers stable row pointers across list growth, name
  snapshot lifetime, and clearing/rebuilding the list. The focused test,
  workspace suite, binary build, changed-file rustfmt, and `git diff
  --check` passed in the isolated worktree. No sanitizer run was performed.

### Increment 98 — choose-buffer item list and names (2026-09-22)

- `window_buffer_modedata` is Box-owned and holds a
  `Vec<Box<WindowBufferItemOwner>>`. Each stable item owns a byte-preserving
  `CString` name and lends its C-layout item pointer only while listed.
  This removes the list `xreallocarray`, per-item `xcalloc`/`xstrdup`, count,
  ordered manual frees, and final list free. Ordered Vec draining preserves
  teardown order. The editor keeps its separate copied name for callbacks
  that outlive a rebuild; mode-tree callback borrows end before reentrancy.
- A focused owner test covers empty/non-UTF-8 names and pointer stability
  across 512 additions. The attached-terminal choose-buffer test now builds
  nine buffers. Focused and workspace tests, binary build, changed-file
  rustfmt, and `git diff --check` passed in the isolated worktree. No
  sanitizer run was performed.

### Increment 99 — customize-tree item list and detached items (2026-09-22)

- `window_customize_modedata` is Box-owned until its existing callback
  reference count reaches zero. It owns a `Vec<Box<CustomizeItemOwner>>`,
  whose stable first-field C item pointers are lent to mode-tree rows.
  `table`, `name`, and `array_key` are borrowed views into private
  byte-preserving `Option<CString>` fields. All detached prompt allocation
  sites and the editor copy use the same boxed owner, then release it once
  in their existing callback path. Rebuild and final destruction drain list
  items in the original forward order. This removes the list's manual
  realloc/count/free and the items' xcalloc/xstrdup/free lifecycle.
- Two focused tests cover detached string copies and last-reference release
  after mode close. Workspace tests, binary build, customize-option CLI
  checks, changed-file rustfmt, and `git diff --check` passed in the
  isolated worktree. No sanitizer run was performed.

### Increment 100 — pane border rendering scratch maps (2026-09-22)

- `window_panes_draw_borders` and `window_panes_draw_floating_border` now own
  their zero-filled byte maps in local `Vec<u8>` values. The synchronous border
  mark helpers borrow a stable pointer while each map stays allocated. This
  removes both `xcalloc`/`free` pairs; checked multiplication retains the old
  allocation overflow check, and byte lookup keeps the existing u32 index
  calculation.
- `window_panes_modedata.areas` remains C-owned because moving its array into
  the mode record requires changing that record's zeroed allocation and final
  destruction together. The local map change has no retained compatibility
  pointer or ABI change.
- Validation in the isolated worktree: workspace tests, binary build,
  changed-file rustfmt, and `git diff --check` passed. No sanitizer or live
  attached-terminal border scenario was run.

### Increment 101 — switch-mode item and match lists (2026-09-22)

- `window_switch_modedata` is Box-owned and holds stable boxed item rows and a
  `Vec` of borrowed match pointers. Rebuild, sort, drawing, selection, and
  teardown now use these containers. Removed both list `xreallocarray` paths,
  manual counts, row and mode `xcalloc`, item/free loops, and array frees.
  Each row's `Drop` releases its still-C-owned fuzzy bitset before its format
  text, as the former free helper did.
- Match pointers remain temporary aliases into boxed rows; the C `qsort`
  callback sorts those pointer entries. The prompt callback and mode entry
  still use raw mode-data pointers. Rebuild and teardown release rows before
  match aliases, preserving the old order; prompt teardown still precedes
  final mode-data destruction.
- Validation in the isolated worktree: workspace tests, binary build,
  changed-file rustfmt, and `git diff --check` passed. No dedicated live
  interactive switch-mode scenario or sanitizer was run.

### Increment 102 — window-tree item list (2026-09-22)

- `window_tree_modedata` is Box-owned through its existing callback reference
  count and holds `Vec<Box<window_tree_itemdata>>`. The boxed numeric rows keep
  stable addresses while mode-tree items borrow them. Removed list
  `xreallocarray`, `item_size`, per-row `xcalloc`/`free`, and the rebuild and
  final free loops. The list clears at the previous rebuild point and drops
  with the mode data when its final reference is released.
- Filtered window/session parents only remove their just-added final row after
  all rejected children have been removed. `mode_tree_remove` unlinks the
  borrowed row before the Box is popped; its implementation and
  `mode_tree_free_item` do not read `itemdata`. Raw mode-tree item pointers
  remain borrowed compatibility views, valid only while their boxed row is
  listed; no foreign ABI changed.
- Validation in the isolated worktree: cargo check, workspace tests, binary
  build, changed-file rustfmt, and `git diff --check` passed. An attached PTY
  scenario exercised choose-tree with an all-filter fallback, filter clear
  and rebuild, then quit/teardown. No sanitizer run was performed.

### Increment 103 — display-panes area list (2026-09-22)

- `window_panes_modedata` is Box-owned from mode init through timer deletion,
  screen/preview teardown, and final drop. Its private `Vec<window_panes_area>`
  replaces the raw area pointer, manual count, `xreallocarray`, and list free.
  `window_panes_draw_screen` replaces old area storage at the previous reset
  point, while `window_panes_find_pane` still searches in reverse display
  order. Area records remain plain numeric values; no retained pointer into
  the Vec or foreign ABI changed.
- Validation in the isolated worktree: cargo check, workspace tests, binary
  build, changed-file rustfmt, and `git diff --check` passed. A private-socket
  two-pane scenario exercised display-panes, resize/redraw, and timer teardown.
  Mouse hit testing was source-audited but not driven with mouse input; no
  sanitizer was run.

### Increment 104 — queued source-file paths (2026-09-22)

- `cmd_source_file_data` is Box-owned through its existing immediate or
  deferred completion path. Its `Vec<CString>` now owns copied, byte-preserving
  paths, replacing the raw files array, `nfiles`, `xreallocarray`, per-path
  `xstrdup`/`free`, and final array/record free. The callback reads files in
  the same order and drops all path copies before `server_client_unref`, as
  the previous cleanup did.
- `file_read` copies each borrowed path synchronously into its own client-file
  state; its completion is scheduled by `event_once`, so CString `as_ptr()`
  remains valid through handoff. The callback record itself remains exposed
  as a raw `void *` at the queue/file callback boundary, then is reconstructed
  as Box exactly once in the existing cleanup paths. No foreign ABI changed.
- Validation in the isolated worktree: focused source-file pattern test,
  workspace tests, binary build, changed-file rustfmt, and `git diff --check`
  passed. Live-server coverage now checks two files applied in order alongside
  existing absolute/relative/glob/no-match cases. No sanitizer was run.

### Increment 105 — layout parser cell-context scratch (2026-09-22)

- `layout_parse_ctx.cctxs` is now a `Vec<layout_parse_cell_ctx>` owned by the
  parser's stack record. Added records use `push`; removal uses `swap_remove`,
  matching the previous final-entry replacement. Removed `size`, `capacity`,
  `xcalloc`, `xreallocarray`, `free`, and `memmove` for this array. The parser
  still frees its separate layout-cell graph at the same error/success points.
- All six C `qsort` calls still sort the contiguous POD records with their
  original comparators, using a temporary `Vec::as_mut_ptr()` view. No record
  reference crosses a sort and no context alias escapes `layout_parse`;
  its C signature is unchanged.
- Validation in the isolated worktree: workspace tests, binary build,
  `scripts/layout_cli_checks.py`, and `git diff --check` passed. Direct
  changed-file rustfmt reports only pre-existing import-order differences in
  `layout/custom.rs`; the edited body is formatted. No sanitizer was run.

### Increment 106 — menu rows and strings (2026-09-22)

- `MenuOwner` embeds the unchanged C-shaped `menu` view and owns its title,
  contiguous item rows, and per-row name/command `CString`s. `menu_create`
  returns the first-field compatibility pointer; `menu_add_item` refreshes
  that view after Vec growth, and `menu_free` drops row strings before row
  storage and title. This removes menu/row `xcalloc` and `xreallocarray`,
  manual count maintenance as an owner, and the item/string/free loop.
- Formatting helpers still return libc-owned strings, so `menu_take_string`
  copies their first-NUL C view into the menu owner and frees the source
  immediately. Both production callers finish building a local menu before
  `menu_display` publishes it; no Rust owner borrow spans formatting calls.
  Overlay callbacks continue to receive the C-shaped view and its borrowed
  contiguous item pointer, valid until `menu_free` or a later add.
- Validation in the isolated worktree: cargo check, workspace tests, binary
  build, changed-file rustfmt, `git diff --check`, and a real attached-terminal
  menu scenario passed. The scenario covers row rendering, duplicate
  separator suppression, empty formatted-name filtering, selected command,
  and overlay teardown. No sanitizer was run.

### Increment 107 — command-prompt rows (2026-09-22)

- `cmd_command_prompt_cdata` is Box-owned through the prompt free callback.
  Its `Vec<cmd_command_prompt_prompt>` replaces the raw prompt array and
  count; each row owns a `CString` prompt and optional `CString` input.
  Removed the row `xcalloc`/`xreallocarray`, per-row strdup/asprintf/free,
  array free, and record xcalloc/free. The command argument/state cleanup
  remains at the original free-callback point.
- Literal `-l` without `-I` still passes a null input, while nonliteral rows
  use an owned empty input where the old splitter did. Comma splitting,
  empty fields, prompt order, and first-NUL C-string views are preserved.
  Prompt and input pointers are captured before set/update callbacks; no
  Rust row borrow crosses a call that can reenter prompt teardown. The raw
  callback data pointer remains the compatibility boundary until freecb.
- Validation in the isolated worktree: focused splitter/null-input tests,
  workspace tests, binary build, changed-file rustfmt, and `git diff --check`
  passed. An attached PTY scenario completed two prompt steps with initial
  inputs and checked the resulting option value. No sanitizer was run.

### Increment 108 — mode-tree line array (2026-09-22)

- `mode_tree_data` is Box-owned from `mode_tree_start` until its existing
  reference count reaches zero. Its inline `Vec<mode_tree_line>` replaces
  `line_list` and `line_size` across build, recursion, selection, drawing,
  search, mouse handling, and teardown. Removed the array `xreallocarray`,
  free, count/pointer synchronization, and record `xcalloc`/`free`.
- Line records are POD with borrowed mode-tree item pointers. Readers use
  Vec length or temporary buffer pointers; no pointer into a line survives
  growth or rebuild. The Box allocator establishes the translated zero state
  for the other C fields before initializing the Vec. Callback signatures
  are unchanged. The internal Rust-only record layout changed from 512 to
  520 bytes, and its authoritative layout fixture/test were updated.
- Validation in the isolated worktree: four focused identity tests including
  65 nested rows, selection, and empty rebuild; workspace tests; binary
  build; changed-file rustfmt; and `git diff --check` passed. A detached
  choose-tree smoke entered mode and killed the session for teardown.
  Detached `send-keys q` did not exit mode, so no quit-key coverage is
  claimed. No sanitizer was run.

### Increment 109 — format substitution pointer list (2026-09-22)

- `format_replace` now owns its local substitution pointers in
  `Vec<*mut format_modifier>`. Removed the list `xreallocarray`, `nsub`
  counter, and success/error `free(sub)` calls. Both exits drop the Vec before
  `format_free_modifiers` to preserve destruction order.
- `format_build_modifiers` finishes all growth of the backing modifier array
  before the substitution pointers are collected. That array is not mutated
  while substitutions run; recursive expansion has its own local list. The
  modifier records and their C allocation/free contract remain unchanged.
- Isolated validation: a new ordered two-substitution regression case,
  workspace tests, binary build, changed-file rustfmt, and `git diff --check`
  passed. No sanitizer was run.

### Increment 110 — redraw scene scratch cells (2026-09-22)

- `redraw_make_scene` now holds a `RedrawCellScratch` owner while building a
  scene. `redraw_build_cells` grows a `Vec<MaybeUninit<redraw_build_cell>>`
  instead of the global `redraw_cells`/`redraw_ncells` reallocarray cache.
  The cell data is copied into spans before the owner returns its capacity
  to a thread-local cache. Removed the global pointer/count and realloc path.
- Taking the cached Vec leaves an empty slot, so a nested scene takes a
  distinct buffer and cannot alias the outer scene's cells. The raw
  `redraw_build_ctx.cells` pointer remains a borrow into the active Vec,
  valid only until the scene build finishes; every used cell is reset before
  reading. The existing size-product check remains, and failed reservation
  still takes the fatal error path.
- Isolated validation: workspace tests, binary build, changed-file rustfmt,
  `git diff --check`, menu CLI checks, and an attached two-pane render,
  layout, and resize scenario passed. A forced nested-scene runtime case was
  not run; no sanitizer was run.

### Increment 111 — customize user-option name list (2026-09-22)

- `window_customize_find_user_options` now fills a `Vec<CString>` for the
  three option scopes. Removed the temporary name-array `xreallocarray`,
  manual size/index tracking, and `free(list)` from
  `window_customize_build_options`; the Vec drops at the same point before
  built-in option traversal.
- Deduplication compares the original C-string bytes and keeps first-seen
  order across global, window, and pane scopes. Each selected name is looked
  up in pane, window, then global order as before. Names are copied before
  row-building format callbacks can run; retained mode-tree rows copy their
  names separately. `name.as_ptr()` is only a synchronous lookup borrow.
- Isolated validation: workspace tests, binary build, changed-file rustfmt,
  `git diff --check`, customize-option CLI checks, and an attached scenario
  with shared and unique names across all three scopes passed. The shared
  name appeared once with the pane value. No sanitizer was run.

### Increment 112 — directional pane candidate lists (2026-09-22)

- `window_pane_find_up`, `window_pane_find_down`, `window_pane_find_left`,
  and `window_pane_find_right` now collect candidate pane pointers in local
  Vecs. The internal `window_pane_choose_best` takes a slice. Removed four
  `xreallocarray`/`free` pairs and their separate size counters.
- Candidate order is unchanged. The chooser still selects the greatest
  `active_point`, keeping the first candidate on ties. Window-owned pane
  pointers are borrowed only through the synchronous collection and choice;
  the geometry/status helpers do not invoke callbacks.
- Isolated validation: workspace tests with serialized test threads, binary
  build, changed-file rustfmt, `git diff --check`, and a private-socket pane
  navigation CLI scenario passed. The scenario checks empty candidates, all
  four directions, equal-activity order, and recent-activity priority. No
  sanitizer was run.

### Increment 113 — format modifier backing array (2026-09-22)

- `format_build_modifiers` now returns `Vec<format_modifier>`, and
  `format_add_modifier` pushes initialized records into it. Removed the
  backing-array `xreallocarray`, `free(list)`, and count out parameter.
  `format_free_modifiers` still frees each C-owned argv before the Vec drops.
- `format_replace` derives count from the Vec and saves element pointers only
  after parsing has stopped growing the array. The selected modifier pointers
  and substitution list remain valid until cleanup. Parse failure and both
  replacement exits keep their prior argv cleanup order; recursive expansion
  has its own local Vec.
- Isolated validation: a 40-modifier growth case followed by two
  substitutions, workspace tests, binary build, format-loop CLI checks,
  changed-file rustfmt, and `git diff --check` passed. No sanitizer was run.

### Increment 114 — prompt paste UTF-8 scratch (2026-09-22)

- `prompt_paste` now owns its temporary `utf8_data` conversion cells in a
  local Vec. Removed the scratch `xreallocarray` and conditional `free(ud)`;
  the Vec drops at function exit. The prompt record and its existing buffer
  allocation remain unchanged.
- `ud` still borrows `pr.copied` when a copied prompt word exists and otherwise
  points into the fixed-length Vec during paste decoding and insertion. The
  Vec is not grown after `ud` is set. The prompt buffer can reallocate while
  the scratch pointer remains valid.
- Isolated validation: workspace tests with serialized test threads, binary
  build, changed-file rustfmt, `git diff --check`, and an attached-client CLI
  scenario passed. The scenario checks UTF-8 top-buffer insertion in the
  middle of text and copied-word precedence. No sanitizer was run.

### Increment 115 — format modifier arguments (2026-09-22)

- `format_modifier.argv` now owns `Vec<CString>`; its argument count derives
  from the Vec. Removed the raw argv array and count, `xcalloc`/`xreallocarray`
  growth, and `cmd_free_argv` cleanup for modifier arguments. The record is no
  longer `Copy` or `repr(C)` because its sole use is internal Rust parsing.
- `format_build_modifiers` copies each C-owned `format_expand1` result into a
  byte-preserving CString and frees the C result at that boundary. All
  modifier operations borrow argument pointers from the stable CString
  allocations; `format_replace` drops the owned records after use on success
  and failure. Recursive expansion uses independent owners.
- Isolated validation: focused empty and non-UTF-8 argument cases, workspace
  tests with serialized test threads, binary build, changed-file rustfmt,
  and `git diff --check` passed. No sanitizer was run.

### Increment 116 — startup configuration paths (2026-09-22)

- `expand_paths` now returns `Vec<CString>` to both socket-label construction
  and startup config handling. Removed its raw path-array realloc/count/free
  lifecycle. The first socket path is borrowed while formatting the label.
- `main_0` collects default paths and repeated `-f` overrides locally, then
  publishes the final Vec once in `CFG_FILES`. `start_cfg` and the
  `config_files` format callback borrow its immutable entries. Removed the
  `cfg_files`/`cfg_nfiles` raw globals, their override free/realloc loop,
  and the related string frees. Deduplication and path order are unchanged;
  repeated `-f` arguments remain repeated. `main_0` has one process-exiting
  call site. No in-tree foreign/header contract for the old exported globals
  was found; staticlib consumers of those symbols would need the new API.
- Isolated validation: workspace tests with serialized test threads, binary
  build, changed-file rustfmt, `git diff --check`, and a private-server CLI
  scenario passed. It covers default path order and deduplication, repeated
  `-f`, non-UTF-8 and empty paths, and socket-label resolution. No sanitizer
  was run.

### Increment 117 — prompt command completion names and display (2026-09-22)

- The prompt record now owns completion names in `Vec<CString>` and its
  rendered display in `Option<CString>`. `prompt_complete_add`/`commands`,
  sorting, prefix calculation, drawing, mouse selection, storage, and clear
  use these owners. Removed the raw completion list/count, its `xreallocarray`
  and element/list frees, and the repeated display `xasprintf`/free chain.
- `prompt_create` allocates the record in a Box, establishes the existing
  zero state for legacy fields, and explicitly initializes the drop-bearing
  completion field before use; `prompt_free` drops the Box after the existing
  free callback and legacy field cleanup. The record is no longer `Copy`.
  `prompt_clear_complete` releases names and Vec backing before display,
  preserving the old clear point and order. Mouse selection copies its name
  into a separate C-owned replacement before a possible clear. The internal
  prompt layout fixture changed from 344 to 360 bytes.
- Isolated validation: workspace tests with serialized test threads, binary
  build, changed-file rustfmt, `git diff --check`, and attached prompt
  completion/paste CLI scenarios passed. The completion scenario covers a
  duplicate alias, unique and ambiguous matches, sorted display, clear after
  typing, and cancel after display. No sanitizer was run.

### Increment 118 — immutable prompt option strings (2026-09-22)

- The Box-owned prompt record now holds `style_str`, `command_style_str`,
  `message_format`, and `word_separators` as CStrings. `prompt_create` copies
  each at its original initialization point; draw, format, and word-navigation
  calls borrow `as_ptr()` synchronously. Removed four `xstrdup` allocations
  and their `prompt_free` frees.
- Each producer already supplied a non-null C string, and copying through
  `CStr` preserves non-UTF-8 bytes and the first-NUL view. The prompt free
  callback still precedes owned-field destruction. The internal prompt
  record layout fixture changed from 360 to 392 bytes; `prompt_create_data`
  and its external callback signatures are unchanged.
- Isolated validation: workspace tests with serialized test threads, binary
  build, attached prompt paste and completion CLI scenarios, changed-file
  rustfmt, and `git diff --check` passed. No sanitizer was run.

### Increment 119 — sorted paste-buffer candidate list (2026-09-22)

- `sort_get_buffers` now returns a local `Vec<*mut paste_buffer>` to its two
  production callers, `window_buffer_build` and `cmd_list_buffers_exec`.
  Removed its process-global raw list, `lsz` capacity counter, and
  `xreallocarray` cache. Each caller owns the list through its traversal;
  the paste-buffer objects themselves remain owned by the paste registry.
- Sorting still uses the same comparator and `sort_qsort` on temporary
  contiguous Vec storage, retaining order, reverse, and tie behavior. A
  recursive sorted-buffer call gets its own list. The translated
  `sort_get_buffers` symbol is no longer exported by the staticlib; no
  in-tree or documented foreign caller was found.
- Isolated validation: workspace tests with serialized test threads, binary
  build, `git diff --check`, and a private-server CLI scenario covering empty,
  order/name/size/reverse/filter, and deletion cases passed. Changed-file
  rustfmt reports only three import layout differences also present in the
  base commit. No sanitizer was run.

### Increment 120 — server client terminal capability list (2026-09-22)

- `ClientOwner` now holds server-side identify capabilities as
  `Vec<CString>` and a `Vec<*mut c_char>` compatibility view for the existing
  `client.term_caps`/`term_ncaps` fields. Removed the identify-path
  `xreallocarray`, per-capability `xstrdup`, and `tty_term_free_list` call for
  this server-side list. The public client record layout is unchanged.
- Before each append the public pointer/count view is invalidated; it is
  rebuilt after both Vecs finish growing. CString allocations keep entry
  addresses stable when the owner Vec moves. `server_client_lost` invalidates
  the view after `tty_free`, then releases strings and pointer storage at the
  old free point and in the old order, even if other references keep the
  ClientOwner alive. Duplicates, order, non-UTF-8 bytes, and first-NUL C views
  are preserved. The client-side terminal-capability producer remains C-owned.
- Isolated validation: a 64-entry view-growth/byte test, workspace tests with
  serialized test threads, binary build, `git diff --check`, and an attached
  custom-terminfo PTY scenario passed. Rustfmt reports only three import
  layout differences also present in the base commit. No sanitizer was run.

### Increment 121 — sorted key-binding candidate lists (2026-09-22)

- `sort_get_key_bindings` and `sort_get_key_bindings_table` now return a
  caller-owned `Vec<*mut key_binding>`. `cmd_list_keys_exec` keeps that list
  through filtering, width calculation, repeat checks, formatting, and output;
  the prefix/root merge extends the same owned list. Key-binding objects stay
  owned by their tables. Both process-global `xreallocarray` caches and their
  capacity counters are gone.
- Sorting retains the existing comparator and contiguous `sort_qsort` storage.
  The translated sort symbols are no longer exported by the staticlib; no
  in-tree or documented foreign consumer was found. An empty table now gives
  an empty list, including `list-keys -N` after both relevant tables are
  removed.
- Isolated validation: serialized workspace tests, binary build, key and
  command-queue CLI checks, and `git diff --check` passed. A live-server test
  covers all-table enumeration and the empty-table case. Rustfmt reports two
  import-layout differences also present in the base commit. No sanitizer
  was run.

### Increment 122 — client terminal capability producer (2026-09-22)

- `tty_term_read_list` now returns `Result<Vec<CString>, CString>` to
  `client_main`. The client keeps each capability and any setup error text in
  Rust-owned storage; the raw capability array, per-entry `xasprintf`,
  `xreallocarray`, and `tty_term_free_list` loop are gone. The server-side
  client owner from increment 120 is unchanged except for a stale comment.
- `client_send_identify` borrows the capability slice and sends each complete
  C string, including its terminator. `proc_send`/`imsg_compose`/`ibuf_add`
  copy those bytes synchronously, so the local Vec may drop after sending.
  Terminfo iteration order, duplicates, non-UTF-8 bytes, first-NUL views,
  empty lists, and setup error messages are preserved. The translated
  `tty_term_read_list` and `tty_term_free_list` symbols are no longer exported;
  no in-tree or documented foreign caller was found.
- Isolated validation: serialized workspace tests, library/binary build,
  `git diff --check`, and the attached custom-terminfo PTY scenario passed.
  That scenario checks normal capability delivery, not a user path to a
  middle NUL. Rustfmt reports four import-layout differences also present in
  the base commit. No sanitizer was run.

### Increment 123 — mutable prompt label and saved input (2026-09-22)

- The Box-owned `prompt.string` is now a `CString` and its nullable
  incremental `last` is `Option<CString>`. Creation initializes both fields;
  `prompt_update` copies a new label before replacing the old one; incremental
  restore borrows the saved C string only while that option is present. The
  Box drop releases both owners at `prompt_free`, replacing the two manual
  frees. Null versus empty saved input remains distinct.
- The prompt layout fixture reflects the internal record's new 408-byte size.
  No public prompt ABI consumer was identified. The rest of the prompt record
  and callback return ownership are unchanged.
- Isolated validation: serialized workspace tests, binary build,
  `git diff --check`, changed-file rustfmt, and attached-client checks for
  a two-step label/input update, Ctrl-R saved-input restore, and completion
  passed. No sanitizer was run.

### Increment 124 — sorted client candidate list (2026-09-22)

- `sort_get_clients` now returns a caller-owned `Vec<*mut client>` to
  `cmd_list_clients_exec`, `format_loop_clients`, and `window_client_build`.
  The process-global `xreallocarray` scratch list and capacity counter are
  gone. Client objects remain owned by the client registry; each caller owns
  only its sorted pointer list until its traversal ends.
- Existing attached-client filtering, comparator, reverse order, and list
  indexes are preserved. A nested client-format expansion gets a separate
  list rather than overwriting the outer traversal. The translated
  `sort_get_clients` symbol/signature is no longer exported by the staticlib;
  no in-tree or documented foreign caller was found.
- Isolated validation: serialized workspace tests, binary build,
  `git diff --check`, and a private-server scenario with two attached control
  clients, reverse sorting, nested `L` formatting, and client chooser passed.
  Rustfmt reports one import-layout difference also present in the base
  commit. No sanitizer was run.

### Increment 125 — sorted session candidate list (2026-09-22)

- `sort_get_sessions` now returns a caller-owned `Vec<*mut session>` to the
  session list command, next/previous navigation, format `S:` loop, and
  window tree/switch builders. Removed its process-global `xreallocarray`
  scratch list and capacity counter. The session registry still owns the
  pointees; callers own only the sorted snapshot through traversal.
- The comparator, name/activity/reverse ordering, list indexes, and live
  session lifecycle are preserved. Nested `S:` expansion gets a separate
  snapshot instead of overwriting an outer traversal. The translated
  `sort_get_sessions` staticlib symbol/signature is gone; no in-tree or
  documented foreign caller was found.
- Isolated validation: serialized workspace tests, binary build, cargo
  check, `git diff --check`, and private-server checks for sorting, reverse,
  filtering, nested `S:` loops, deletion, format loops, and session CLI
  reference comparison passed. Rustfmt reports three import-layout
  differences also present in the base commit. No sanitizer was run.

### Increment 126 — prompt copied UTF-8 data (2026-09-22)

- The Box-owned prompt record now holds `copied` as
  `Option<Box<[utf8_data]>>`. One helper saves a boxed copy of the cut range
  plus its zero sentinel before the live input buffer is moved. All 26 cut
  branches call that helper; a later cut drops the previous copy on
  replacement, and `prompt_free` drops the final owner with the prompt Box.
  Removed each `xcalloc`/`memcpy`/`free` sequence for this field.
- `None` still means that no word has been copied; `Some` with only a
  terminator represents an empty copy. `prompt_paste` borrows the boxed
  storage through its paste operation. The internal prompt layout fixture
  now records 416 bytes and the shifted completion field; no foreign prompt
  layout consumer was identified.
- Isolated validation: serialized workspace tests, binary build, changed-file
  rustfmt, `git diff --check`, and an attached-client cut/yank check passed.
  The CLI check now covers replacing a saved word with UTF-8 input and pasting
  the replacement. No sanitizer was run.

### Increment 127 — all-session sorted winlink list (2026-09-22)

- `sort_get_winlinks` now returns a caller-owned `Vec<*mut winlink>` to
  `list-windows -a` and the window-switch mode builder. Removed its
  process-global `xreallocarray` pointer list and capacity counter. Session
  winlink trees still own the pointees; the command or mode build owns only
  its sorted snapshot through traversal.
- Enumeration retains duplicate links across grouped sessions, the existing
  comparator and reverse order, and `list-windows`' total-count `#{line}`
  value. The command temporarily borrows a pointer into its local Vec while
  its other branch still uses a raw sorter; that pointer expires when the
  command returns. The translated `sort_get_winlinks` staticlib symbol is
  gone; no in-tree or documented foreign caller was found.
- Isolated validation: serialized workspace tests, binary build,
  `git diff --check`, grouped-session sorting and window-switch CLI checks,
  and session CLI comparison with reference tmux passed. Rustfmt reports two
  import-layout differences also present in the base commit. No sanitizer
  was run.

### Increment 128 — sorted panes within a window (2026-09-22)

- `sort_get_panes_window` now returns a caller-owned
  `Vec<*mut window_pane>` to `list-panes`, the format `P:` loop, and the
  window-tree builder. Removed its process-global `xreallocarray` pointer
  list and capacity counter. Pane objects remain owned by their windows;
  each caller owns only its sorted snapshot through traversal.
- Pane index/reverse/filter behavior, format loop index/last flags, and the
  existing total-count `#{line}` value are preserved. Nested `P:` expansion
  gets separate storage instead of overwriting an outer traversal. The
  translated `sort_get_panes_window` staticlib symbol is gone; no in-tree or
  documented foreign caller was found.
- Isolated validation: serialized workspace tests, binary build,
  `git diff --check`, focused format-loop checks, and a private-server pane
  sorting, nested loop, and deletion scenario passed. Rustfmt reports two
  import-layout differences also present in the base commit. No sanitizer
  was run.

### Increment 129 — session sorted winlink list (2026-09-22)

- `sort_get_winlinks_session` now returns a caller-owned
  `Vec<*mut winlink>` to `list-windows`, the format `W:` loop, and the
  window-tree builder. Removed its process-global `xreallocarray` pointer
  list and capacity counter. Session winlink trees still own the pointees;
  each caller owns only its sorted snapshot through traversal.
- Name/index/reverse order, nested `W:` formatting, deletion behavior, and
  `list-windows`' total-count `#{line}` value are preserved. After both
  winlink sorter migrations, `cmd_list_windows_exec` now chooses one owned
  Vec for either branch and indexes it directly, removing its local raw-list
  pointer and branch-specific compatibility views. The translated
  `sort_get_winlinks_session` staticlib symbol is gone; no in-tree or
  documented foreign caller was found.
- Isolated validation: serialized workspace tests, binary build,
  `git diff --check`, focused format-loop and session CLI checks, and a
  private-server scenario for sorting, nested `W:` loops, and deletion
  passed. Rustfmt reports two import-layout differences also present in the
  base commit. No sanitizer was run.

### Increment 130 — remove unused global pane sorter (2026-09-22)

- Removed `sort_get_panes` and its process-global `xreallocarray` pointer
  cache/capacity counter. Repository-wide source search found no in-tree
  caller, header, or documented foreign contract. Its traversal and
  comparator duplicated the used window-scoped pane sorter; deletion removes
  a permanent allocation with no production caller.
- The `no_mangle` staticlib symbol is intentionally gone. An undocumented
  foreign consumer would need updating; the session-scoped sorter remains
  exported until its separate audit completes.
- Isolated validation: library build, serialized workspace tests,
  `git diff --check`, and `nm` confirmed the removed symbol is absent while
  `sort_get_panes_session` remains. Rustfmt reports one import-layout
  difference also present in the base commit. No sanitizer was run.

### Increment 131 — remove unused session pane sorter (2026-09-22)

- Removed `sort_get_panes_session` and its process-global `xreallocarray`
  pointer cache/capacity counter. It had no in-tree caller. With both unused
  pane sorters gone, `src/sort.rs` no longer imports `xreallocarray` or keeps
  any static pointer-list cache. The used window-scoped pane sorter remains
  caller-owned from increment 128.
- The `no_mangle` staticlib symbol is intentionally gone. The upstream tmux
  source header declares the historical function, but this repository has no
  header, caller, or documented foreign consumer of that ABI. Unknown
  out-of-tree staticlib consumers would need updating.
- Isolated validation: library build, serialized workspace tests,
  `git diff --check`, and `nm` confirmed the session symbol is absent while
  the global symbol remained in that isolated branch. After integrating both
  removals, the library build and `nm` confirmed that neither symbol remains.
  Rustfmt reports one import-layout difference also present in the base
  commit. No sanitizer was run.

### Next candidates

The later layout-equivalence cleanup removed the detached
`LayoutDescription` API and its API-only tests. `layout_parse` again builds
`layout_cell` and `layout_parse_ctx` directly, as in tmux, while the
`LayoutString` serializer owner remains. `cargo test --workspace`, the binary
build, and `scripts/layout_cli_checks.py` passed; the same CLI script also
passed with the pinned tmux binary, including an ignored `I` field with a
non-string value.

1. The Box-owned prompt record still has its editable `buffer` as a separately
   allocated UTF-8 data array. Audit its edit, resize, and callback paths
   before moving that complete field to Rust-owned storage.
2. The only direct `xvasprintf` production caller outside the `xmalloc`
   wrappers is `format_printf`. Its callback ABI requires a C-owned return
   that consumers libc-free, so a local `CString` does not remove manual
   ownership. The migrated `xvasprintf_cstring` callers have no identified
   user path that emits a middle NUL: `args_print_add`'s `%c` values are
   validated nonzero option flags, and layout's `%c` values are fixed
   nonzero characters. A synthetic variadic FFI call could emit one, but it
   would not be a supported E2E scenario. Revisit the direct caller when the
   callback return contract can change.
3. `cmd_save_buffer_exec`'s `file_write` call copies its path
   synchronously, but changing only its local expanded path to `CString`
   would add a copy solely to replace the C-owned
   `format_single_from_target` result. Revisit
   with the format expansion producer. Remaining `xstrndup` callers return
   or transfer C-owned strings; `window_copy` regex buffers grow through a
   shared C API.
4. The remaining address-based registries, other UI tags, and session/winlink
   graph require separate migrations. The typed mode-tree key permits further
   semantic tags, but each mode still needs its own identity and alias audit.
   `window_client` has no existing guaranteed unique semantic key: names and
   PIDs can repeat, and creation timestamps are not unique by contract.
   Its pointer tag must wait for a client owner/observer migration; a new
   tag-only generated ID would violate the agreed type policy.

Current validation is recorded in increments 15–131. The remaining
address-based registries and UI tags above are separate migration candidates.
Each of increments 15–131 has its own local commit; none was pushed.
The combined main-branch workspace test initially reused a cached `hmux-rt`
test binary containing a removed worktree's compile-time manifest path.
After `cargo clean -p hmux-rt`, the workspace suite and binary build passed;
changed-file rustfmt and `git diff --check` also passed.
After increments 91–93 were integrated, `cargo test --workspace --quiet`,
`cargo build --bin hmux2 --quiet`, and `git diff --check` passed on main.
After increments 94–96 were integrated, the workspace suite, binary build,
changed-file `rustfmt --edition 2021 --check`, and `git diff --check` passed
on main. The direct `rustfmt` invocation requires the crate's 2021 edition
for the C string literals in the session-group test.
After increments 97–99 were integrated, `cargo test --workspace --quiet`,
`cargo build --bin hmux2 --quiet`, the customize-option CLI checks,
changed-file `rustfmt --edition 2021 --check`, and `git diff --check` passed
on main.
After increments 100–102 were integrated, `cargo test --workspace --quiet`,
`cargo build --bin hmux2 --quiet`, and `git diff --check` passed on main.
Changed-file rustfmt still reports only two import layout differences in
`window_switch.rs` and `window_tree.rs`; running the same check on those exact
files from the pre-migration `a5292e8` commit reports the same differences.
No combined sanitizer or additional attached-terminal scenario was run.
After increments 103–105 were integrated, `cargo test --workspace --quiet`,
`cargo build --bin hmux2 --quiet`, `scripts/layout_cli_checks.py`, and
`git diff --check` passed on main. Changed-file rustfmt reports only two
pre-existing import-layout differences in `layout/custom.rs` at lines 7 and
58; the same check on the pre-increment `204ab04` file reports both sites.
No combined sanitizer or additional live UI scenario was run.
After increments 106–108 were integrated, `cargo test --workspace --quiet`,
`cargo build --bin hmux2 --quiet`, `scripts/menu_owner_cli_checks.py`, and
`git diff --check` passed on main. Changed-file rustfmt reports only three
pre-existing import-layout differences in `menu.rs`, `command_prompt.rs`,
and `mode_tree.rs`; the same check on their pre-increment `75ec29b` versions
reports those exact sites. No combined sanitizer was run.
After increments 109–111 were integrated, `RUST_TEST_THREADS=1 cargo test
--workspace --quiet`, `cargo build --bin hmux2 --quiet`, the customize-option
and menu-owner CLI checks, changed-file `rustfmt --edition 2021 --check`, and
`git diff --check` passed on main. The first workspace run reused an
`hmux-rt` test binary with a removed worktree's compile-time manifest path;
`cargo clean -p hmux-rt` resolved it. The next parallel run had a collision
between timestamp-based PTY test directories; that test passed by itself and
the serialized full suite passed. No combined sanitizer was run.
After increments 112–114 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the binary build,
pane-navigation, prompt-paste, and format-loop CLI checks, changed-file
rustfmt, and `git diff --check` passed on main. Serial test threads avoid the
known timestamp-based PTY test-directory collision. No combined sanitizer
was run.
After increments 115–117 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the binary build,
config-path, prompt-completion, prompt-paste, and format-loop CLI checks, and
`git diff --check` passed on main. Changed-file rustfmt reports one import
layout difference in `tmux.rs` at line 24; checking the exact file from
pre-increment `b61432d` reports the same difference. No combined sanitizer
was run.
After increments 118–120 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the binary build,
prompt-paste, prompt-completion, sorted-buffer, and server-term-capability CLI
checks, and `git diff --check` passed on main. Changed-file rustfmt reports
six import layout differences in `sort.rs`, `window_buffer.rs`,
`cmd/entries/list_buffers.rs`, and `server_client.rs`; checking those exact
files from pre-increment `32a39a5` reports the same six sites. No combined
sanitizer was run.
After increments 121–123 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the binary build,
prompt-mutable, prompt-completion, server-term-capability, key, and command
queue CLI checks, and `git diff --check` passed on main. Changed-file rustfmt
reports six import-layout differences in `cmd/entries/list_keys.rs`,
`sort.rs`, `client.rs`, and `server_client.rs`; checking the exact files from
pre-increment `9eadbcb` reports the same six sites. No combined sanitizer
was run.
After increments 124–126 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the binary build,
sorted-client, sorted-session, prompt-paste, format-loop, and session-reference
CLI checks, and `git diff --check` passed on main. Changed-file rustfmt reports
three import-layout differences in `sort.rs`, `window_tree.rs`, and
`window_switch.rs`; checking the exact files from pre-increment `803b3b3`
reports the same three sites. No combined sanitizer was run.
After increments 127–129 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the binary build,
all-session winlink, session-winlink, window-pane, format-loop, and
session-reference CLI checks, and `git diff --check` passed on main.
Changed-file rustfmt reports three import-layout differences in `sort.rs`,
`window_switch.rs`, and `window_tree.rs`; checking the exact files from
pre-increment `b7a5535` reports the same three sites. No combined sanitizer
was run.
