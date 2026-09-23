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

## Current priorities

The later layout-equivalence cleanup removed the detached
`LayoutDescription` API and its API-only tests. `layout_parse` again builds
`layout_cell` and `layout_parse_ctx` directly, as in tmux, while the
`LayoutString` serializer owner remains. `cargo test --workspace`, the binary
build, and `scripts/layout_cli_checks.py` passed; the same CLI script also
passed with the pinned tmux binary, including an ignored `I` field with a
non-string value.

1. `screen.title` and `screen.path` retain C-owned cleaned strings, while the
   title stack duplicates and transfers title text. `screen` is still a Copy
   C-layout record, so migrate the title/path/stack lifecycle together only
   after auditing every by-value copy and embedded screen destruction path.
   Existing title-stack and OSC 7 CLI checks provide normal-path coverage.
2. Exported `fuzzy_match`, `args_from_vector`, and `monitor_parse` retain
   C-owned output contracts for external callers; no in-tree production caller
   uses their raw-output paths now. These string-return contracts are deferred
   while live client fields remain. Revisit this ranking after each completed
   boundary.
3. The remaining address-based registries, other UI tags, and session/winlink
   graph require separate migrations. The typed mode-tree key permits further
   semantic tags, but each mode still needs its own identity and alias audit.
   `window_client` has no existing guaranteed unique semantic key: names and
   PIDs can repeat, and creation timestamps are not unique by contract. Its
   pointer tag must wait for a client owner/observer migration; a new tag-only
   generated ID would violate the agreed type policy.
4. The only direct `xvasprintf` production caller outside the `xmalloc`
   wrappers is `format_printf`. Its callback ABI requires a C-owned return
   that consumers libc-free, so a local `CString` does not remove manual
   ownership. `args_print_add`'s `%c` values are validated nonzero option
   flags, layout's `%c` values are fixed nonzero characters, and
   `format_log1`'s `%c` values come from a non-NUL format scan. An
   attached-client `show-buffer`
   of a binary `A\0B` buffer does reach `window_copy_vadd`'s `%.*s` formatter,
   but C formatting stops at the first NUL: the formatted output is `A`, not
   a string with an interior NUL. This first-NUL behavior is covered by
   `scripts/window_copy_vadd_owner_cli_checks.py`. No supported path has been
   found that puts an interior NUL in any `xvasprintf` output. This audit did
   not include `log_vwrite`'s direct `vasprintf`, which has the supported
   interior-NUL case recorded in increment 289. The related
   `format_find` transforms also return C-owned strings to `format_replace`; local
   `_cstring` conversions would add copies. Revisit these paths when their
   callback/value return contracts can change.
5. `cmd_save_buffer_exec` now borrows its static detached `show-buffer` path.
   Changing only its formatted `save-buffer` path to `CString` would add a
   copy solely to replace the C-owned `format_single_from_target` result.
   The expansion output now has a local owner, but its exported result still
   crosses the C-owned return boundary. Remaining `xstrndup` callers return
   or transfer C-owned strings.

OSC 52 decoded output now moves directly from a `Vec<u8>` into the boxed paste
owner on the query path. The exported `paste_add` still copies and consumes
its C input. The real terminal clipboard-query E2E covers binary `A\0B\xff`,
invalid base64, and empty replies.

In-tree `set-buffer`, `load-buffer`, capture-pane, and window-copy payloads
now move through the private paste owner. Exported `paste_set` still copies
its C producer on success and leaves that allocation with the caller on a
name error.

## Recent migration detail

### Increment 357 — owned monitor parse outputs for in-tree callers (2026-09-23)

- `monitor_parse_owned` now returns a `ParsedMonitor` with owned `CString`
  name and format fields. `cmd_refresh_client_update_subscription` and
  `cmd_set_hook_monitor_exec` lend those strings to the synchronous monitor
  registration calls and let the owner drop on success, errors, and bare-name
  unsubscribe. Removed their explicit output frees and the invalid
  unsubscribe `xstrdup`. The exported `monitor_parse` keeps its existing
  C-owned output contract by duplicating the owned parse strings with
  `xstrdup`; invalid parses still leave name/format outputs untouched and
  preserve numeric target parsing through `sscanf`.
- Monitor parser tests, monitor leaf and string CLI comparisons, hook append
  and invalid/unsubscribe CLI checks, and control-client subscription CLI
  checks passed. The latter two ran against both candidate and pinned baseline.
  Full workspace tests, binary build, changed-file rustfmt, and diff checks
  passed. No sanitizer was run.

### Increment 358 — inline client tty range (2026-09-23)

- `ClientOwner` now holds the single `visible_range` needed by its embedded
  tty's no-overlay path. After `tty_init` clears the public tty record,
  `server_client_dispatch_identify` installs the owner-backed pointer and
  capacity before the first resize/redraw. `tty_check_overlay_range` therefore
  reuses that slot without allocating. `server_client_lost` asserts the view
  still points to the owner, clears it, then calls exported `tty_free`; its
  legacy C-owned free behavior remains for out-of-tree callers. The inline
  slot drops with `ClientOwner`.
- Real PTY popup checks passed on candidate and pinned baseline, including
  no-overlay rendering, overlay display, and client teardown. Full workspace
  tests, binary build, changed-file rustfmt, and diff checks passed. The first
  test build exposed a missing test-module type import, which was fixed before
  rerunning the suite. No sanitizer was run.

### Increment 359 — owned fuzzy match masks (2026-09-23)

- `fuzzy_match_owned` now builds its bit mask in a `Vec<bitstr_t>` on empty
  patterns and successful matches. `format_match_fuzzy` owns the temporary
  mask through boolean or position formatting, and boxed
  `window_switch_itemdata` retains an `Option<Vec<bitstr_t>>` through redraw
  and item teardown. Removed both in-tree mask frees and their `calloc`
  allocations. Window-switch draw only borrows a raw mask pointer for its
  existing synchronous rendering pass. Exported `fuzzy_match` still returns
  a libc-owned, C-freeable mask by copying the owned result into `calloc`
  storage for external callers.
- Fuzzy matcher unit tests passed. Window-switch mode-string and target CLI
  checks passed against the pinned baseline; a new fuzzy format CLI check
  compares positions, boolean matches, empty patterns, misses, UTF-8, and
  invalid bytes with that baseline. Full workspace tests, binary build,
  changed-file rustfmt, and diff checks passed. No sanitizer was run.

### Increment 360 — inline screen-print result buffer (2026-09-23)

- Exported `screen_print` now writes into a fixed inline static 16 KiB array.
  Removed its lazy `xmalloc` and the raw global heap pointer. The returned
  pointer remains stable across calls, with the same capacity, truncation
  checks, and overwrite-on-next-call lifetime. The function has no in-tree
  production caller; this removes a persistent allocation at its exported
  boundary without changing its signature.
- A direct API regression test passed against the old implementation and the
  new one, checking filtered line output and pointer reuse. Full workspace
  tests, binary build, changed-file rustfmt, and diff checks passed. No
  sanitizer was run.

### Increment 361 — single terminal event per file owner (2026-09-23)

- `FileOwner` now records when its terminal callback has been scheduled.
  `file_fire_done` queues at most one event while a stream remains indexed,
  so a read-done message followed by client teardown cannot dispatch a
  second callback after the first frees the owner and its callback data.
  The initial file reference continues to keep the owner alive through that
  one event; no exported layout or callback signature changed.
- A deterministic reactor test schedules terminal completion twice before
  dispatch and checks one callback and final index removal. Disconnected
  client queue draining remains a separate coordinated migration: its wait
  owners include files, prompts, jobs, timers, wait channels, popup overlays,
  and pane wait items, and unfired queue callbacks own data that needs a
  cancellation destructor.
- `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the binary build,
  the attached-client binary-buffer CLI check against the pinned baseline,
  changed-file rustfmt, and `git diff --check` passed. Workspace-wide
  rustfmt still reports unrelated pre-existing formatting differences.

### Increment 362 — owned UTF-8 sanitizer output (2026-09-23)

- Private `utf8_sanitize_cstring` now builds printable ASCII and display-width
  underscores in a `Vec<u8>` and returns a `CString`. The two in-tree callers,
  `server_client_print` and `cmdq_error`, borrow or retain that owner directly;
  removed their matching libc frees and the sanitizer's per-byte
  `xreallocarray` loop. Exported `utf8_sanitize` still returns a libc-freeable
  duplicate for its C return contract. The input still ends at its first NUL.
  The scanner now retries from the saved candidate start on malformed complete
  UTF-8, avoiding the old pointer rewind before the input; it retains the
  old zero-size fatal for a leading zero-width cell.
- Focused tests cover empty input, ASCII controls and DEL, UTF-8 display
  widths, malformed and truncated sequences, first-NUL input, and freeing
  the exported result. `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  the binary build, command-queue and control-line CLI checks, changed-file
  rustfmt, and `git diff --check` passed. Rust allocation failure diagnostics
  can differ from the former `xreallocarray` fatal; no sanitizer was run.

### Increment 363 — file-owned load-buffer callback data (2026-09-23)

- `file_read_with_owned_data` now installs a boxed `load-buffer` payload in
  `FileOwner` before any terminal event can run. The existing
  `client_file.data` pointer borrows that stable payload for progress and
  terminal callbacks. `file_fire_done_cb` takes and drops the payload after
  callback delivery or suppression, replacing the raw Box handoff and the
  separate skipped-done cleanup hook. `cmd_load_buffer_done` borrows its
  data; `cmd_load_buffer_data` releases its retained target-client reference
  on drop, with explicit release before `cmdq_continue` on normal completion
  to preserve the previous order. Exported file callback layouts and
  signatures remain unchanged.
- The load-buffer attached-client CLI check passed against the pinned
  baseline for binary data, empty/error cases, and a pending FIFO read whose
  source client disconnects. `RUST_TEST_THREADS=1 cargo test --workspace
  --quiet`, the binary build, changed-file rustfmt, and `git diff --check`
  passed. The broader disconnected-client command queue remains unretired;
  no sanitizer was run.

### Increment 364 — owned key-table names (2026-09-23)

- Private `KeyTableOwner` now holds the table's name in a `CString`; the
  exported `key_table.name` field borrows its pointer until the table's last
  reference is released. `key_bindings_get_table` creates this owner and
  `key_bindings_unref_table` drops it after removing live/default bindings,
  replacing the separate `xstrdup` and `free` for the table name. Registry
  and client pointers still observe the same stable table address, and the
  exported `key_table` layout is unchanged.
- The key-binding storage test now checks a non-UTF-8 table name, in addition
  to retained-table lifetime; it and the full workspace tests passed. The binary
  build, private-server key CLI check, changed-file rustfmt, and
  `git diff --check` passed. No sanitizer was run.

### Increment 365 — owned key-binding notes (2026-09-23)

- Private `KeyBindingOwner` now holds each live/default binding's optional
  note as a `CString`. Its exported `key_binding.note` field borrows the owner
  until replacement or index removal. `key_bindings_set_note` copies the
  incoming C string before dropping the previous value, including when input
  aliases the current note. Both production constructors, note-only updates,
  resets, startup default snapshots, and both customize-mode mutations use
  the owner; `key_bindings_free` drops it after releasing the command-list
  reference. Removed all in-tree note `xstrdup`/`free` pairs without changing
  the exported record layout. The default snapshot still has a null table
  name and its own note copy while retaining the shared command-list ref.
- Storage tests now cover aliased note input, non-UTF-8 bytes, independent
  default snapshots, reset, and final teardown. The live-server list-keys
  test covers note-only updates. The attached customize E2E now edits a note
  through the prompt and clears it through the editor, checking `list-keys`
  after each operation. Full workspace tests, the binary build, key CLI,
  candidate customize detail/prompt checks, changed-file rustfmt, Python
  syntax, and diff checks passed. The pinned
  baseline customize-mode subprocess timed out in this environment; its
  spawned test processes were terminated. No sanitizer was run.

### Increment 366 — owned client tty name (2026-09-23)

- Existing `ClientOwner` now holds `client.ttyname` in an `Option<CString>`.
  `server_client_set_ttyname` refreshes the exported borrowed pointer after
  identify messages and clears it at the former free site in
  `server_client_lost`, after client hooks and tty teardown. This replaces
  the `MSG_IDENTIFY_TTYNAME` `xstrdup` and the final libc free, and safely
  replaces a repeated identify payload. `client.name` and choose-client
  entries retain their own copies; the public client layout is unchanged.
- A focused owner test covers non-UTF-8 bytes, replacement, empty versus
  absent, and pointer stability. Full workspace tests, the binary build,
  attached-terminal `server_term_caps_cli_checks.py`, changed-file rustfmt,
  and `git diff --check` passed. No sanitizer was run.

### Increment 367 — owned client terminal name (2026-09-23)

- Existing `ClientOwner` now holds `client.term_name` in an
  `Option<CString>`. The identify message setter and the
  missing/empty-to-`unknown` fallback refresh the exported borrowed pointer;
  `server_client_lost` clears it at the former free site after tty teardown.
  Removed both `xstrdup` allocations and both manual frees for this field.
  `tty_term_create` copies the value into its own owner, format callbacks
  duplicate it for their C result, and the public client layout is unchanged.
- A focused owner test covers replacement, non-UTF-8 bytes, absent/empty
  fallback, and clearing. The attached-terminal CLI check now asserts
  `#{client_termname}` in addition to successful tty setup/detach. Full
  workspace tests, the binary build, changed-file rustfmt, and diff checks
  passed. No sanitizer was run.

### Increment 368 — owned client working directory (2026-09-23)

- `ClientOwner` now owns `client.cwd` as `Option<CString>`. The identify
  message preserves its validated path, home, and root fallback choices;
  `server_client_lost` clears the borrowed public pointer at the former free
  site. The audited `server_client_get_cwd` consumers use its result
  synchronously or copy it before retaining it. Repeated identify payloads
  replace the owner and release the old value. The public client layout is
  unchanged.
- A focused owner test checks non-UTF-8 bytes, replacement, empty versus
  absent, pointer stability, and clearing. The attached-client file-path CLI
  check now resolves relative non-UTF-8 paths through IDENTIFY_CWD; candidate
  and pinned baseline passed. Serialized workspace tests, binary build,
  changed-file rustfmt, and diff checks passed. No sanitizer was run.

### Increment 369 — owned client terminal type (2026-09-23)

- `ClientOwner` now owns `client.term_type` as `Option<CString>`. The extended
  device-attributes parser sets the borrowed public pointer through a shared
  setter, preserving its first-NUL C-string view, absent versus empty values,
  and replacement before `tty_update_features`. Client loss clears it at the
  former free site after tty teardown. The format callback still returns its
  separately C-owned copy, and the public client layout is unchanged.
- A focused owner test checks replacement, non-UTF-8 bytes, empty versus
  absent, and clearing. The attached-terminal CLI now sends a reproducible
  `ESC P > | XTerm(370) ESC \\` reply and checks `#{client_termtype}`; both
  candidate and pinned baseline passed. Serialized workspace tests, binary
  build, changed-file rustfmt, and diff checks passed. No sanitizer was run.

### Increment 370 — owned option array item keys (2026-09-23)

- `OwnedOptionArrayItem` now keeps each normalized key in a `CString`; the
  exported `options_array_item.key` borrows it until item removal. The map
  still stores a copied semantic `OptionsArrayKey`, and `options_array_free`
  preserves value destruction before map removal before key destruction.
  Removed the per-item `xstrdup`/`free` pair and the public record's `Copy`
  implementation; the record layout and stable boxed address remain.
- Serialized workspace tests, binary build, array-key CLI checks with raw
  non-UTF-8 keys and deletion, value-rendering CLI comparison with the
  matching 3.8-rc baseline, changed-file rustfmt, and diff checks passed.
  The older 3.7b baseline rejects the raw `0x80` test key, so it is not a
  usable reference for that rendering check. No sanitizer was run.

### Increment 371 — owned client terminal title (2026-09-23)

- `ClientOwner` now owns the retained `client.title` as `Option<CString>`.
  `server_client_set_title` keeps its bytewise changed-title comparison,
  refreshes the exported borrowed pointer before `tty_set_title`, and still
  frees the separate C-owned `format_expand_time` result after the tty call.
  Client loss clears the title at the former free site; the public client
  layout and output order are unchanged.
- A focused owner test covers non-UTF-8 bytes, replacement, empty versus
  absent, pointer stability, and clear. An attached-PTY CLI check enables
  `set-titles`, changes its template, forces redraw, and verifies initial,
  changed, unchanged, and empty OSC title output against the matching 3.8-rc
  baseline. Serialized workspace tests, binary build, changed-file rustfmt,
  and diff checks passed. No sanitizer was run.

### Increment 372 — owned client terminal path (2026-09-23)

- `ClientOwner` now owns the retained `client.path` as `Option<CString>`.
  `server_client_set_path` keeps its active-pane lookup, null-to-empty
  fallback, bytewise changed-path comparison, and `tty_set_path` call after
  refreshing the exported borrowed pointer. Client loss clears it at the
  former free site. The public client layout and tty output order remain.
- A focused owner test covers non-UTF-8 bytes, replacement, empty versus
  absent, pointer stability, and clear. An attached-PTY CLI sends OSC 7
  through the pane and checks a distinctive outer terminal path code for
  first, repeated, changed, and empty values; candidate and matching 3.8-rc
  baseline passed. Serialized workspace tests, binary build, changed-file
  rustfmt, and diff checks passed. No sanitizer was run.

### Increment 373 — owned detached client session name (2026-09-23)

- `ClientOwner` now owns `client.exit_session` as `Option<CString>`.
  `server_client_detach` captures the attached session name after setting
  its exit flags and message type, and `server_client_check_exit` still sends
  its NUL-terminated borrowed view through `MSG_DETACH`. Client loss clears
  it at the former free site. The public client layout and detach ordering
  are unchanged; replacement also releases a prior owner if invoked again.
- A focused owner test checks replacement, non-UTF-8 bytes, empty versus
  absent, pointer stability, and clearing. An attached-PTY CLI renames the
  session before detach and checks the client-reported session name against
  the matching 3.8-rc baseline. Serialized workspace tests, binary build,
  changed-file rustfmt, and diff checks passed. No sanitizer was run.

### Increment 374 — owned cached client username (2026-09-23)

- `ClientOwner` now owns the lazily cached `client.user` as `Option<CString>`.
  `format_cb_client_user` copies the borrowed `getpwuid` name into the owner
  before returning its separately C-owned format result. The public client
  pointer remains a borrowed view until final `server_client_free`, where it
  is cleared at the former free site. Failed UID/passwd lookups still leave
  it absent, and the public client layout is unchanged.
- A focused owner test covers replacement, non-UTF-8 bytes, empty versus
  absent, pointer stability, and clearing. The attached-terminal CLI checks
  repeated `#{client_user}` queries against the local passwd name; candidate
  and matching 3.8-rc baseline passed. Serialized workspace tests, binary
  build, changed-file rustfmt, and diff checks passed. No sanitizer was run.

### Increment 375 — owned client name (2026-09-23)

- `ClientOwner` now owns `client.name` as `Option<CString>`. Identify
  completion copies a nonempty tty name or builds `client-{pid}`, then lends
  its public pointer through client-close hooks and delayed references until
  final `server_client_free`. Removed the identify `xstrdup`/`xasprintf` and
  final manual free. The format callback still duplicates its C-owned result,
  and the public client layout is unchanged.
- A focused owner test checks replacement, non-UTF-8 bytes, empty versus
  absent, pointer stability, and clearing. Terminal CLI checks verify the
  tty-derived name; control-client lookup CLI checks verify `client-{pid}`
  and target resolution. Candidate and matching 3.8-rc baseline passed.
  Serialized workspace tests, binary build, changed-file rustfmt, and diff
  checks passed. No sanitizer was run.

### Increment 376 — owned client exit message (2026-09-23)

- `ClientOwner` now owns `client.exit_message` as `Option<CString>` until
  client loss. Both control `too far behind` writers, the ACL-update writer,
  denied accept, and startup socket error now use one setter. The startup
  error copies its C-allocated cause and frees that allocation once at the
  transfer point. `server_client_check_exit` still sends the exact borrowed
  NUL-terminated bytes for `MSG_EXIT`; the public client layout is unchanged.
  Repeated ACL updates now replace and release the previous message.
- A focused owner test checks non-UTF-8 bytes, replacement, empty versus
  absent, pointer stability, and clear. The existing exit-payload CLI
  compares normal exit and startup socket error with the 3.8-rc baseline;
  ACL entry CLI checks also passed against that baseline. A live denied
  client was not exercised because this test user owns the server and cannot
  revoke its own UID access. Serialized workspace tests, binary build,
  changed-file rustfmt, and diff checks passed. No sanitizer was run.

### Increment 377 — owned client-process detached session name (2026-09-23)

- The private `client_exitsession` static now holds `Option<CString>` rather
  than a raw `xstrdup` result. Validated detach messages copy bytes into the
  owner; `client_exit_message` borrows its pointer for final formatting, and
  `client_main` releases it after output. A later detach packet replaces and
  releases the previous value. No public ABI or wire format changed.
- The attached-PTY CLI renamed a session before detach and checked the
  client-reported final name against the 3.8-rc baseline. Serialized
  workspace tests, binary build, changed-file rustfmt, and diff checks
  passed. No sanitizer was run.

### Increment 378 — owned numeric-key UTF-8 cells (2026-09-23)

- `key_string_parse_numeric` now uses the existing
  `utf8_fromcstr_vec` decoder for its local numeric-key cells. The vector
  preserves the size-zero terminator used by `utf8_from_data` and drops at
  function exit. Removed the local `utf8_fromcstr` allocation and `free`;
  locale-sensitive `sscanf`/`wctomb` and numeric-key rules remain intact.
- Existing numeric-key parser tests, serialized workspace tests, and binary
  build passed. The key CLI now binds `0x41` and checks canonical `A`
  listing; candidate and matching 3.8-rc baseline passed. Changed-file
  rustfmt and diff checks passed. No sanitizer was run.

### Increment 379 — owned client-process exec payload (2026-09-23)

- The private `MSG_EXEC` command and shell now share one
  `Option<(CString, CString)>` owner. The validated packet is copied into both
  values before the pair is installed. `client_main` borrows their pointers
  for `client_exec`, whose successful `execl` replaces the process. Removed
  both static raw pointers and their `xstrdup` calls; repeated exec packets
  replace and release the old pair. The wire format and separate `MSG_SHELL`
  path are unchanged.
- Serialized workspace tests, binary build, and the PTY `detach-client -E`
  CLI check passed against the pinned 3.8-rc baseline, including raw `0xff`
  output and `$SHELL=/bin/sh`. Changed-file rustfmt and diff checks passed.
  No sanitizer was run.

### Increment 380 — owned copy-mode search string (2026-09-23)

- The private boxed `window_copy_mode_data.searchstr` now uses
  `Option<CString>`. Copy-mode initialization and plain/incremental search
  commands copy C-string bytes into the owner; searches borrow its pointer.
  Replacements and mode teardown drop it automatically, removing its local
  `xstrdup`/`free` pairs. The `-F` search still gets a C-owned
  `format_single` result, which is copied into this owner and libc-freed at
  that producer boundary. Absent and empty values remain distinct.
- Serialized workspace tests and binary build passed. Copy-match and
  copy-mode backing CLI checks passed against the pinned 3.8-rc baseline.
  The copy-match CLI now exercises `search-forward -F` and reads
  `pane_search_string` after replacements. Changed-file rustfmt and diff
  checks passed. No sanitizer was run.

### Increment 381 — owned pane cached search string (2026-09-23)

- `WindowPaneOwned` now holds the pane's cached search text as
  `Option<CString>`. `window_copy_search` installs copied C-string bytes;
  the public `window_pane.searchstr` field is a borrowed compatibility view
  invalidated on replacement or clear. Pane teardown clears the owner at
  the former free point. Removed the pane cache's `xstrdup` and manual free;
  the mode's separate search string owner remains independent.
- Serialized workspace tests, binary build, and copy-match and copy-mode
  backing CLI comparisons against the pinned 3.8-rc baseline passed. The
  copy-match CLI reads `pane_search_string` after repeated and formatted
  searches. Changed-file rustfmt and diff checks passed. No sanitizer ran.

### Increment 382 — owned pane shell string (2026-09-23)

- `WindowPaneOwned` now stores `shell_owner: Option<CString>`; the pane's
  public `shell` pointer borrows it until replacement or clear. Initial
  `spawn_pane` copies the validated default-shell bytes into this owner.
  Respawn keeps the existing value. Pane teardown clears it at the former
  free point, removing the shell's `xstrdup` and manual free.
- Serialized workspace tests and binary build passed. Shell argv, command
  format, and live `/proc` CLI checks passed; the first two compared with
  the pinned 3.8-rc baseline. The command-format CLI now changes the default
  shell before respawn and confirms the pane retains its original shell.
  It excludes the initial asynchronous automatic window name from binary
  equality because that name can initially be the executable's own name.
  Changed-file rustfmt and diff checks passed. No sanitizer ran.

### Increment 383 — owned pane working directory (2026-09-23)

- `spawn_pane` now carries its constructed cwd in `Option<CString>` and
  transfers it to `WindowPaneOwned` only when a new cwd was supplied. The
  public `pane.cwd` pointer borrows that owner until replacement or clear.
  Absolute, relative, empty, and non-UTF-8 paths retain C-string bytes.
  `format_single` and relative-path `xasprintf` outputs are copied then
  libc-freed at their producer boundaries; keeping the C formatter retains
  its `%s` behavior. Early active-respawn failure drops the local owner,
  and respawn without `-c` retains the pane's previous cwd. Removed the
  local `xstrdup`/manual frees and the pane teardown free.
- Serialized workspace tests and binary build passed. A new pane-cwd CLI
  compared candidate and pinned 3.8-rc baseline for path creation,
  formatted relative `-c`, actual process cwd, active-respawn error,
  respawn retention/replacement, empty `-c`, and raw `0xff` path bytes.
  Changed-file rustfmt and diff checks passed. No sanitizer ran.

### Increment 384 — local pane-border expansion owner (2026-09-23)

- `window_make_pane_status` now copies the C-owned `format_expand_time`
  result into a local `CString`, libc-frees the producer allocation, and
  lends the owned bytes through `format_draw`. The pane never reads its
  `border_status_line.expanded` cache, so the redundant retained allocation,
  replacement free, and teardown free are gone. The shared
  `style_line_entry.expanded` field remains for status-line instances that
  compare cached text.
- Serialized workspace tests and binary build passed. A new PTY CLI check
  rendered two successive pane-border markers, killed the pane, and passed
  against the pinned 3.8-rc baseline. Changed-file rustfmt and diff checks
  passed. No sanitizer ran.

### Increment 385 — owned saved window layout (2026-09-23)

- `WindowOwned` now keeps the saved `old_layout` as `Option<CString>` behind
  the public window's borrowed pointer. `select-layout` copies and frees the
  C-owned `layout_dump` result before replacing the owner, holds the prior
  owner through `-o` parsing, restores it after parse failure, and releases
  it before redraw on success. Window destruction clears the owner at the
  former free point. Removed saved-layout manual frees and raw ownership
  transfer without changing the public window layout.
- Serialized workspace tests and binary build passed. The layout CLI added
  a failed selection followed by `select-layout -o` and passed with both
  candidate and pinned 3.8-rc baseline. Changed-file rustfmt and diff checks
  passed. No sanitizer ran.

### Increment 386 — owned window name (2026-09-23)

- `WindowOwned` now owns `window.name` as a `CString`; the public pointer
  borrows it until replacement or destruction. Spawn copies explicit name
  bytes directly into the owner. Default-name and `clean_name` producers
  still return C allocations, which are copied and libc-freed at the
  producer boundary. Removed the manual name frees in spawn, break-pane,
  explicit rename, and window destruction. `window_set_name` computes its
  cleaned name before replacing the old one to preserve aliasing, and keeps
  the previous owner alive through synchronous rename callbacks.
- The reentrant rename test now constructs a private `WindowOwned` fixture;
  its former bare public-window fixture could not represent this invariant.
  Serialized workspace tests and binary build passed. A new CLI comparison
  against pinned 3.8-rc covers explicit rename, named and default spawn,
  named and default break-pane, and pane terminal rename. Event payload,
  waiter, and command-stringify CLI comparisons also passed. Changed-file
  rustfmt and diff checks passed. No sanitizer ran.

### Increment 387 — owned session working directory (2026-09-23)

- `SessionOwner` now holds `session.cwd` as `Option<CString>`, and the public
  pointer borrows it until replacement or early destruction. `session_create`
  copies the incoming C-string bytes directly into the owner, removing its
  `xstrdup`. `attach-session -c` copies the C-owned `format_single` result,
  frees that producer allocation, and replaces the owner after expansion,
  preserving formats that read the previous session cwd. `session_destroy`
  clears the owner at the old free point, before delayed session release.
- Serialized workspace tests, binary build, and the pane-cwd CLI comparison
  passed. A focused session-cwd CLI comparison covers initial and attached
  cwd, session path formatting, and subsequent window creation. Changed-file
  rustfmt and diff checks passed. No sanitizer ran.

### Increment 388 — owned session name (2026-09-23)

- `SessionOwner` now holds `session.name` as a byte-preserving `CString`;
  the public pointer borrows it through delayed final session release.
  Explicit creation copies the incoming name directly. Generated creation
  builds the optional raw-byte prefix, hyphen, and decimal ID in Rust while
  retaining the collision loop. `rename-session` copies the C-owned
  `clean_name` result and frees that producer allocation, then removes the
  old map key before replacing the owner and inserting the new key. Rename
  event payload strings are copied before the replacement. Removed the
  creation `xstrdup`, generated-name `xasprintf`/free cycle, rename free,
  and final name free.
- Serialized workspace tests, binary build, generated-prefix, session, and
  sorted-session CLI comparisons passed. A focused name CLI comparison
  covers explicit and generated names, rename hooks, duplicate/invalid
  names, map lookup, sorted listing, and kill/recreate. Changed-file
  rustfmt and diff checks passed. No sanitizer ran.

### Increment 389 — owned job command (2026-09-23)

- `JobOwner` now boxes the public job at offset zero and holds its `cmd` as
  `Option<CString>`. Shell-command jobs copy the borrowed command bytes
  directly; argv jobs take the existing `cmd_stringify_argv_cstring` result
  directly, retaining a null pointer only for its negative-count case.
  `job_transfer` and `job_free` drop the owner after callbacks and resource
  cleanup. Removed the command `xstrdup`, the C-owned stringify result, and
  both manual command frees. No external job-command writer or retained
  alias was found; `job_transfer` has no in-tree caller.
- Serialized workspace tests and binary build passed. Run-shell and if-shell
  CLI comparisons passed against pinned 3.8-rc. The popup CLI now starts a
  null-command argv job, observes its stringified command through
  `show-messages -J` while live, and covers completion and teardown; it
  passed on both candidate and pinned baseline. Changed-file rustfmt and
  diff checks passed. No sanitizer ran.

### Increment 390 — owned command source filename (2026-09-23)

- `CmdOwner` now boxes each parsed or copied `cmd` at offset zero and owns
  its optional source filename as `CString`. The public `cmd.file` pointer
  borrows that value until `cmd_free`. `cmd_parse` copies caller-owned file
  bytes after argument parsing succeeds; `cmd_copy` copies the original
  owner's bytes. Removed both filename `xstrdup` calls and the manual free.
- The command-printer integration fixture now obtains commands through
  `cmd_parse` rather than allocating bare public records. A focused API test
  checks null filenames, raw `0xff` filename bytes, mutation of the input,
  and copy independence after the original is freed. Serialized workspace
  tests and binary build passed; lexer/source-file and verbose parser CLI
  comparisons passed against pinned 3.8-rc. Changed-file rustfmt and diff
  checks passed. No sanitizer ran.

### Increment 391 — owned JSON node key (2026-09-23)

- `JsonNodeOwner` now boxes each public `json_node` at offset zero and owns
  an optional byte-preserving key `CString`. The public key pointer borrows
  it through field-map insertion, serialization, recursive parsing, and
  `json_destroy_node`. Removed `json_create_node`'s `xstrdup` and the
  destructor's manual key free. The raw public node no longer derives
  `Copy` or `Clone`.
- The JSON field-ordering test now creates nodes through the production
  constructor, drops its input key strings before reading the nodes, and
  destroys duplicate and removed nodes through the production destructor.
  Serialized workspace tests and binary build passed. JSON parse/recursive
  cleanup and custom-layout CLI checks passed on candidate and pinned
  3.8-rc. Changed-file rustfmt and diff checks passed. No sanitizer ran.

### Increment 392 — owned command queue item name (2026-09-23)

- `CmdqItemOwner` now boxes public queue items at offset zero and holds
  their printable name as a `CString`. Command and callback constructors
  assemble raw label bytes and the stable item pointer in Rust, replacing
  both `xasprintf` calls. The public `cmdq_item.name` pointer borrows that
  value until queue removal. `cmdq_remove` drops the owner at its old free
  point; a Rust-only detached-item cleanup helper handles unqueued items.
  Removed the name's manual free, and the raw item no longer derives `Copy`
  or `Clone`.
- The options and argument-conversion fixtures now use detached cleanup.
  Focused tests cover command labels, raw `0xff` callback labels, and C
  `%p` pointer formatting. Serialized workspace tests and binary build
  passed; if-shell, hook insertion, and session CLI comparisons passed
  against pinned 3.8-rc. Changed-file rustfmt and diff checks passed. No
  sanitizer ran.

### Increment 393 — owned argument command state strings (2026-09-23)

- `ArgsCommandStateOwner` now boxes the public `args_command_state` at offset
  zero and owns its optional command and parse-source filename as `CString`
  values. Its raw `cmd` and `pi.file` pointers borrow them until
  `args_make_commands_free`; the command-list branch leaves both absent.
  Direct command input copies borrowed bytes, formatted command output is
  copied before its C allocation is freed, and source filenames copy the
  parsed command's bytes. Removed the state command `xstrdup`, source
  filename `xstrdup`, and both final manual frees. The public state no
  longer derives `Copy` or `Clone`.
- Serialized workspace tests and binary build passed. Command-parser and
  run-shell CLI comparisons passed against pinned 3.8-rc. The if-shell CLI
  now sources an invalid nested command and compares its exact normalized
  filename/line diagnostic, with source commands before and after; it also
  covers string, formatted, and background paths. Changed-file rustfmt and
  diff checks passed. No sanitizer ran.

### Increment 394 — owned monitor item name and format (2026-09-23)

- `MonitorItemOwner` now boxes each public `monitor_item` at offset zero and
  owns byte-preserving `CString` name and format fields. `monitor_add` copies
  both incoming strings before finding and removing a replaced item, so
  either input can alias the old item's borrowed fields. The item index is
  removed before owner drop. Removed both creation `xstrdup` calls and both
  final manual frees; the public item no longer derives `Copy` or `Clone`.
  The distinct `last` values retain their separate lifetime.
- The monitor-storage test now uses `monitor_add`, verifies raw `0xff` name
  and format bytes after caller buffer mutation, replaces an item using its
  own name and format pointers, and covers nested pane/window ordering and
  removal. Serialized workspace tests and binary build passed. Monitor
  leaf and hook append/string CLI comparisons passed against pinned 3.8-rc.
  Changed-file rustfmt and diff checks passed. No sanitizer ran.

### Increment 395 — owned paste buffer name (2026-09-23)

- `PasteBufferOwner` now boxes public paste buffers at offset zero and owns
  each name as a byte-preserving `CString`; the public pointer borrows it
  while name and time indexes retain the stable buffer address. Automatic
  names are assembled from raw prefix bytes and the decimal index without
  `xasprintf`. Explicit `paste_set` and `paste_rename` copy `clean_name`'s
  C-owned result and free that producer allocation. Rename removes the old
  name index before replacing the owner and retains the previous `CString`
  through the deletion event, including when `oldname` aliases the public
  pointer. Removed all name manual frees. Paste data retains its separate
  C-owned transfer contract; the raw buffer no longer derives `Copy`.
- A focused unit test calls rename with the buffer's borrowed current-name
  pointer and verifies old/new lookup and teardown. Serialized workspace
  tests and binary build passed. The set-buffer CLI comparison covers
  automatic, explicit, renamed, deleted, and evicted names and now checks
  rename/delete notifications. Changed-file rustfmt and diff checks passed.
  No sanitizer ran.

### Increment 396 — owned run-shell command (2026-09-23)

- `cmd_run_shell_data.cmd` is now an `Option<CString>` inside its boxed callback
  record. `cmd_run_shell_exec` copies the `format_expand` result into that
  owner and immediately frees the C producer allocation. Timer and job
  callbacks borrow the command pointer; `-C` and no-command paths retain
  absence. Removed the final manual command free in `cmd_run_shell_free`.
- Serialized workspace tests and binary build passed. The run-shell CLI
  comparison against pinned 3.8-rc passed for immediate output, actual format
  expansion, cwd, command-list execution, delayed background execution, and
  failure status. Changed-file rustfmt and diff checks passed. No sanitizer ran.

### Increment 397 — owned event payload item names (2026-09-23)

- `EventPayloadItemOwner` now boxes every public payload item at offset zero,
  owns its byte-preserving name as `CString`, and holds an optional string
  value. All setters reach the single name-copy boundary in
  `event_payload_set_item`; the public name pointer borrows that owner through
  map lookup, format addition, iteration, and final removal. The old name is
  copied before a replacement can free its item. Removed the name `xstrdup`
  and final manual free, while preserving value release callbacks before
  owner destruction and the existing duplicate-key replacement sequence.
- A focused unit test replaces an item using its own borrowed non-UTF-8 name;
  the existing tests cover string values, integer replacement, pointer free
  callbacks, and iteration order. Serialized workspace tests and binary build
  passed. Event payload format, hook monitor string, client command payload,
  and server exit payload CLI checks passed against pinned 3.8-rc.
  Changed-file rustfmt and diff checks passed. No sanitizer ran.

### Increment 398 — owned client status expansions (2026-09-23)

- `ClientOwner` now holds five optional `CString` expansions for its public
  `status_line.entries`. `status_redraw` keeps the C-produced string alive
  through comparison and drawing, then copies a changed result into the owner
  and frees the C allocation. A helper invalidates the legacy pointer before
  replacement. `status_free` clears each owner at the old final free point,
  before the client owner is eventually dropped. The unchanged expansion
  remains a short-lived C allocation and is freed immediately.
- Serialized workspace tests and binary build passed. A new PTY CLI comparison
  passed against pinned 3.8-rc: it waits through unchanged status timer ticks,
  changes the expanded option twice, observes both redraws, then detaches the
  client. Changed-file rustfmt and diff checks passed. No sanitizer ran.

### Increment 399 — owned CSI colon parameter strings (2026-09-23)

- `InputCtxOwner` now holds the 24 optional `CString` slots for CSI colon
  parameters. `input_split` copies each colon string directly into the owner
  and lends a pointer through the public `input_param` union. Both the next
  split and `input_free` clear owners at the old release points, invalidating
  the borrowed pointer first. Removed the only parameter `xstrdup` and both
  manual free loops. A numeric parse failure leaves earlier colon parameters
  owned until the next split or parser destruction, as before.
- A focused test checks that partial-list error and subsequent replacement.
  Serialized workspace tests and binary build passed. A live pane CLI check
  sends repeated valid and malformed colon SGR sequences, compares styled
  captures with pinned 3.8-rc, and destroys the parser while colon parameters
  remain. Changed-file rustfmt and diff checks passed. No sanitizer ran.

### Increment 400 — owned option array string values (2026-09-23)

- `OwnedOptionArrayItem` now holds an optional `CString` alongside its key.
  The public `options_value.string` pointer borrows that owner until item
  removal or replacement. `options_array_set` copies direct input bytes or
  assembles append bytes before touching the previous owner, so input may
  alias the current value. Removed its `xstrdup`/`xasprintf` string producer
  and the string branch's manual free in `options_value_free`; command-list
  cleanup stays there. `options_array_free` drops the item owner after map
  removal, releasing the string with the item.
- A focused unit test covers append and replacement using the old borrowed
  pointer with non-UTF-8 bytes. Serialized workspace tests and binary build
  passed. A CLI comparison against pinned 3.8-rc covers set, append with
  byte `0xff`, replace, unset, and an unaffected sibling item. Changed-file
  rustfmt and diff checks passed. No sanitizer ran.

### Increment 401 — owned JSON node string values (2026-09-23)

- `JsonNodeOwner` now holds optional `CString` string values alongside its
  optional key. `json_parse_string` copies token bytes directly into that
  owner and lends the public union pointer. `json_assign_value` copies a
  borrowed string when its private string branch is used; number, boolean,
  object, and array assignment remain as before. Recursive node destruction
  drops the owner, removing the token `xstrndup` and final manual string free.
- A focused parser test releases the input buffer before reading a nested raw
  `0xff` string and destroying the tree. Serialized workspace tests and binary
  build passed. The JSON CLI comparison against pinned 3.8-rc covers raw
  `0xff` and UTF-8 values, empty-string rejection, recursive failure cleanup,
  and successful parsing afterward. The parser accepts strings only as object
  values, not direct array elements, and rejects empty strings before node
  creation; these existing behaviors were retained. Changed-file rustfmt and
  diff checks passed. No sanitizer ran.

### Increment 402 — owned terminal capability strings (2026-09-23)

- `TtyTermOwner` now holds indexed optional `CString` values for all string
  capabilities; public `tty_code` union pointers borrow those slots. Creation
  strips terminfo delay text directly into `CString`, preserving the old
  8191-byte limit. Overrides, feature application, validation, and teardown
  replace or clear owners while keeping the public code type and value order.
  Removed the `xstrdup` producers and the manual string frees. Removing a
  capability or repeating one during creation now also releases the earlier
  string instead of retaining an unreachable allocation.
- A focused strip test checks delay removal, raw high bytes, and the length
  limit. Serialized workspace tests and binary build passed. Attached-client
  CLI checks against pinned 3.8-rc cover terminfo creation, single override,
  sequential replacement, removal, client identification, and failed terminal
  creation cleanup. Changed-file rustfmt and diff checks passed. No sanitizer
  ran.

### Increment 403 — owned paste buffer data (2026-09-23)

- `PasteBufferOwner` now holds optional boxed bytes and lends the public
  `paste_buffer.data` pointer with its exact size. `paste_add`, `paste_set`,
  and `paste_replace` copy accepted C producer allocations into the owner and
  free them at transfer. `paste_free` and replacement no longer free stored
  data manually. Name errors in `paste_set` still leave input with the caller;
  zero-size inputs are still consumed. The `set-buffer -w` caller retains a
  clipboard-only copy after transfer, and `load-buffer -w` uses its live input
  buffer after transfer, preserving success-only clipboard timing.
- A focused test checks bytes across an interior NUL and high bytes before
  and after replacement. Serialized workspace tests and binary build passed.
  Pinned-baseline CLI comparisons passed for named buffer replacement and
  eviction, `set-buffer -w`, binary `load-buffer -w`, and editor replacement
  of binary data. Changed-file rustfmt and diff checks passed. No sanitizer
  ran.

### Increment 404 — owned switch-mode row text (2026-09-23)

- Boxed `window_switch_itemdata` now stores its formatted row text as a
  `CString`. Both session and window `format_expand` results are copied into
  the owner and their C allocations freed at the producer boundary. Filtering
  and drawing borrow `as_ptr()` synchronously; the custom `Drop` and its manual
  free are gone. The box still keeps each row address stable for `matches`.
- Serialized workspace tests and the binary build passed. Pinned-baseline
  switch-mode CLI checks passed for row rendering, incremental filter rebuild,
  selection, cancel, and session/window target paths. Changed-file rustfmt
  and diff checks passed. No sanitizer ran.

### Increment 405 — owned mode command template temporaries (2026-09-23)

- The existing `cmd_template_replace_cstring` is crate-visible. Synchronous
  callers in `mode_tree_draw_help_line`, `mode_tree_run_command`, and
  `window_switch_run_command` now use its `CString` directly and borrow its
  pointer during format expansion or command parsing. Their extra exported
  C copies and matching manual frees are gone. The exported
  `cmd_template_replace` still returns a libc-freeable copy. Percent-template
  debug logging moved into the shared helper, preserving its prior output.
- Serialized workspace tests and the binary build passed. Pinned-baseline
  CLI comparisons passed for choose-tree command execution, attached help
  redraw with `C-h`, and switch-mode selection and target paths. Changed-file
  rustfmt and diff checks passed. No sanitizer ran.

### Increment 406 — owned argument command construction (2026-09-23)

- `args_make_commands` now holds its temporary command as `CString` from the
  initial copy through each argument substitution and synchronous
  `cmd_parse_from_string`. This retires its initial `xstrdup`, each exported
  C replacement allocation, and the matching manual frees. The returned
  command list and parse-error ownership contracts are unchanged.
- Serialized workspace tests and the binary build passed. Pinned-baseline
  CLI comparisons passed for if-shell, run-shell, and two successive
  command-prompt substitutions including a literal semicolon and a parse
  error. The existing UTF-8 prompt-paste CLI check passed. Changed-file
  rustfmt and diff checks passed. No sanitizer ran.

### Increment 407 — owned OSC 52 decoded bytes (2026-09-23)

- `tty_keys_clipboard` now decodes base64 directly into a `Vec<u8>`, lends its
  pointer to the synchronous `input_request_reply`, and moves its exact
  decoded bytes into the paste owner on the pending-query path. Invalid and
  unqueried results drop normally. A private `paste_add_owned` shares the
  existing automatic-buffer creation, eviction, and event path; exported
  `paste_add` still copies and consumes its C input. Its prefix remains valid
  through name construction even if it aliases that C input.
- Serialized workspace tests and binary build passed. A real attached-PTY
  comparison against pinned tmux passed for a query followed by `A\0B\xff`,
  invalid base64, and an empty reply. Existing set-buffer and load-buffer
  CLI comparisons passed. Changed-file rustfmt and diff checks passed. No
  sanitizer ran.

### Increment 408 — owned monitor cached values (2026-09-23)

- Private boxed owners for `monitor_item`, `monitor_pane`, and
  `monitor_window` now hold optional `CString` cached values; public `last`
  pointers borrow those strings. `monitor_check_value` copies each accepted
  `format_expand` result, frees its C producer after any report callback,
  and retains the previous owned value through that callback. Item teardown
  and pane/window generation sweeps drop owners instead of freeing raw
  `last` strings. Production pane/window constructors replace zeroed plain
  boxes, including in the storage test.
- A focused test covers equal-value suppression, non-UTF-8 previous/current
  bytes, and removal of the item during its report callback. The report name
  remains valid throughout that callback. Serialized workspace tests and
  binary build passed. Pinned-baseline monitor leaf, hook string, and control
  subscription CLI comparisons passed for changes and teardown. Changed-file
  rustfmt and diff checks passed. No sanitizer ran. During a reentrant
  callback, the public cache pointer now shows the new value; report
  `value`/`last` fields and notification order retain their prior meaning.

### Increment 409 — owned application OSC 52 decoded bytes (2026-09-23)

- `input_osc_52_parse` now decodes into a `Vec<u8>` and returns it to its
  sole caller. Invalid or unhandled data drops normally. `input_osc_52`
  lends the bytes to synchronous terminal or pane selection, keeps them alive
  through the pane event, and moves the exact decoded bytes to
  `paste_add_owned`. The former xmalloc, invalid/no-client free, and raw
  producer transfer to `paste_add` are removed. The `Vec` retains an allocated
  pointer even for a decoded zero-length result until selection finishes.
- Serialized workspace tests and binary build passed. A real application
  output comparison against pinned tmux passed for pane binary `A\0B\xff`,
  invalid base64, and empty payload, plus popup binary `A\0B\xff`. The pane
  and popup traces wait for a marker after the OSC before inspecting the
  clipboard. Changed-file rustfmt and diff checks passed. No sanitizer ran.

### Increment 410 — owned clipboard request copy (2026-09-23)

- In `input_request_clipboard_reply`, the `get-clipboard=both` path now copies
  nonempty borrowed terminal reply bytes once into a boxed slice and calls
  `paste_add_owned`. The former xmalloc, memcpy, second paste copy, and C
  free are gone. The original borrowed reply remains valid for the subsequent
  `input_reply_clipboard`; an empty reply still creates no paste buffer.
- Serialized workspace tests and binary build passed. A real attached-PTY
  comparison against pinned tmux sets `set-clipboard=on` and
  `get-clipboard=both`, has a pane application issue OSC 52 query, and checks
  the terminal's binary `A\0B\xff` response, the application's response,
  and the exact new paste buffer. Invalid base64 and empty terminal replies
  create neither an application response nor a new buffer. Changed-file
  rustfmt and diff checks passed. No sanitizer ran.

### Increment 411 — owned load-buffer paste data (2026-09-23)

- Private `paste_set_owned` accepts boxed bytes and stores accepted named
  data directly in `PasteBufferOwner`, or moves unnamed data to
  `paste_add_owned`. Exported `paste_set` copies its C input into the owner,
  consumes the C allocation on success, and leaves it with the caller on a
  name error. It still consumes zero-length input before name validation;
  accepted named C data is freed before synchronous change events.
  `cmd_load_buffer_done` now copies completed evbuffer bytes once into a
  boxed slice and calls the owned helper. The evbuffer bytes remain available
  for the synchronous `-w` terminal selection. Its xmalloc, memcpy, and
  error-path C free are removed.
- Serialized workspace tests and binary build passed. Pinned-baseline CLI
  comparisons passed for binary and empty file loads, automatic and named
  replacement buffers, invalid empty/non-UTF-8 names, missing path, FIFO
  client cancellation, and observed `-w` OSC 52 output with `A\0B\xff`.
  Existing set-buffer, capture-pane, and window-copy selection comparisons
  passed for the exported C adapter. Changed-file rustfmt and diff checks
  passed. No sanitizer ran. Named replacement event reentrancy remains a
  pre-existing concern outside this boundary.

### Increment 412 — owned capture-pane paste data (2026-09-23)

- `cmd_capture_pane_exec` now moves its already owned `Vec<u8>` capture
  result into `paste_set_owned` on the buffer path. Its xmalloc, memcpy,
  success transfer, and error-path C free are gone. The separate `-p` print
  path keeps its C-string terminator and existing first-NUL printing rule.
- Serialized workspace tests and binary build passed. The pinned-baseline
  capture-pane comparison passed for history, hyperlinks, raw pending bytes
  including `ESC [ NUL`, empty-name error, named buffer, and newly checked
  automatic buffer. Changed-file rustfmt and diff checks passed. No
  sanitizer ran.

### Increment 413 — owned window-copy selection paste data (2026-09-23)

- `window_copy_copy_buffer` and `window_copy_append_selection` now move their
  selected `Vec<u8>` bytes into `paste_add_owned` and `paste_set_owned` after
  synchronous selection and `pane-set-clipboard` events. The shared
  `window_copy_alloc_paste_data` xmalloc/copy helper and append error-path C
  free are gone. Append still frees the separate name returned by
  `paste_get_top`; paste errors without a cause are still ignored as before.
- Serialized workspace tests and binary build passed. The pinned-baseline
  CLI comparison passed for copy, append, pipe, and binary `P\0Q` append.
  Changed-file rustfmt and diff checks passed. No sanitizer ran.

### Increment 414 — owned set-buffer byte builder (2026-09-23)

- `cmd_set_buffer_exec` now builds replacement and appended bytes in a
  `Vec<u8>` and moves them through `paste_set_owned`. Its xmalloc, xrealloc,
  memcpy, success transfer, and error-path C free are gone. The append path
  copies an existing named buffer before the replacement event; a separate
  `-w` snapshot remains alive for terminal selection after the paste event.
  Empty input still returns before modifying a buffer.
- Serialized workspace tests and binary build passed. The pinned-baseline
  set-buffer CLI comparison passed for named and automatic buffers, rename,
  append, eviction, invalid names, and an attached-PTY binary `A\0B\xff`
  append whose `-w` OSC 52 output is checked byte for byte. Empty input and
  invalid name produce no clipboard output. Changed-file rustfmt and diff
  checks passed. No sanitizer ran.

### Increment 415 — owned paste-buffer escaped bytes (2026-09-23)

- Private `utf8_stravisx_bytes` escapes an explicit byte length into a
  `Vec<u8>`, preserving bytes after an input NUL and omitting the C
  terminator. `cmd_paste_buffer_paste` lends that slice to synchronous
  `bufferevent_write`, which copies into its output evbuffer. Its former
  exported `utf8_stravisx` allocation and manual free are removed. The
  exported escape function keeps its C-owned result contract.
- Serialized workspace tests and binary build passed. A private-session
  candidate/pinned-tmux comparison checked exact delivered bytes for
  `A\0B\xff\nC\tD` in default escaped, `-S` raw, and `-p` bracketed
  modes. The reader records actual pane input, and its completion marker
  follows the paste on the same stream after the first bytes arrive to
  avoid an asynchronous ordering race. Changed-file rustfmt and diff
  checks passed. No sanitizer ran.

### Increment 416 — owned server-client escaped message (2026-09-23)

- `server_client_print(parse=0)` now owns escaped bytes in a `Vec<u8>` with
  a trailing NUL while its logging, control output, file output, and view
  calls borrow the pointer. The former `utf8_stravisx` allocation and final
  C free are removed. `utf8_stravisx_bytes` returns an empty vector before
  touching a possibly null zero-length evbuffer data pointer. The separate
  `parse != 0` evbuffer path is unchanged; exported `utf8_stravisx` still
  returns C-owned storage.
- Serialized workspace tests and binary build passed. A real control-client
  `display-message -c` comparison against pinned tmux passed for plain,
  empty, and tab/LF/CR/BS/ESC/DEL/`0xff` messages. The test frames literal
  LF output with a following distinct control message and compares exact
  bytes. Command argv cannot contain an interior NUL. Changed-file rustfmt
  and diff checks passed. No sanitizer ran.

### Increment 417 — owned menu trim bytes (2026-09-23)

- Private `format_trim_right_bytes` now returns the existing trimmed
  `Vec<u8>` directly, preserving style, UTF-8, invalid bytes, and the
  first-NUL C-string view. `menu_add_item` extends that vector with suffix
  and key text before storing its `CString`; its intermediate C allocation,
  copy, and free are gone. Exported `format_trim_right` wraps the private
  helper with its C-owned output contract.
- Serialized workspace tests, including focused trim cases, and the binary
  build passed. The menu overlay CLI script passed with both candidate and
  pinned tmux, including a long trimmed label and key suffix. Changed-file
  rustfmt and diff checks passed. No sanitizer ran.

### Increment 418 — owned window-tree preview label (2026-09-23)

- Private `format_trim_left_bytes` returns the already built trimmed
  `Vec<u8>`; exported `format_trim_left` still copies into its C-owned
  result for expression callers. `window_tree_draw_label` now owns a local
  `CString` and lends its pointer through width and drawing calls. Its
  zero-width early return drops the owner instead of leaking the former
  `new_label` C allocation.
- Serialized workspace tests and binary build passed. The pinned-baseline
  preview-label CLI comparison passed for normal modes and a 9-column
  attached client. With a five-column preview, ASCII `Z` draws an inner
  label box while the two-column `界` trims to empty at limit one and draws
  only the outer preview border. The narrow comparison passed repeatedly.
  Changed-file rustfmt and diff checks passed. No sanitizer ran.

### Increment 419 — owned option command names (2026-09-23)

- `options_parse_owned` and `options_match_owned` now return a `CString` name
  and optional normalized `CString` array key. `set-option`, `show-options`,
  and the customize add-option prompt borrow those through their synchronous
  calls; their matching-output `free` paths are gone. The exported
  `options_parse`, `options_match`, and `options_match_command` adapters retain
  libc-freeable output. Empty parse still leaves the key output untouched;
  parse failure still leaves the ambiguity output untouched.
- Serialized workspace tests, binary build, candidate array-key CLI, and
  customize add-option CLI with candidate and pinned tmux passed. A direct
  candidate/pinned comparison matched valid normalized array keys, ambiguous
  `status-`, and invalid `not-an-option` in set/show commands. The pinned tmux
  differs on malformed `update-environment[]` and overflow keys: it reports
  ambiguous where the existing candidate parser reports invalid, so the
  candidate-only array-key script cannot pass against pinned tmux. Changed-file
  rustfmt and diff checks passed. No sanitizer ran.

### Increment 420 — parse format option names once (2026-09-23)

- `format_find` now parses the option name and optional array key once into
  `OwnedOptionName`, then borrows them through up to six synchronous scope
  lookups and `options_to_string`. Its repeated `options_parse_get` C name/key
  allocations and array-key `free` are gone. The exported `options_parse_get`
  ABI and its C-owned key result remain for external callers.
- Serialized workspace tests and binary build passed. A direct CLI comparison
  with pinned tmux matched normalized array-key lookup, session option override,
  ordinary format-table fallback, and missing/invalid option names. Changed-file
  rustfmt and diff checks passed. No sanitizer ran.

### Increment 421 — own cleaned paste and session names directly (2026-09-23)

- `clean_name_cstring` now validates, rewrites untrusted `#(`, and escapes into
  a `CString` directly. The exported `clean_name` duplicates that owned result
  only at its C return boundary. `paste_rename`, named `paste_set_inner`, and
  `rename-session` move the owned result into their existing owners, removing
  three C allocation/copy/free round trips. Other `clean_name` callers still
  use its C-owned adapter for their raw C fields.
- Serialized workspace tests, binary build, focused clean-name tests, and
  sorted-buffer CLI checks passed. Named set/rename paste and session-name CLI
  checks passed against pinned tmux, including invalid UTF-8, duplicate names,
  and session rename hooks. Changed-file rustfmt and diff checks passed. No
  sanitizer ran.

### Increment 422 — own cleaned window rename directly (2026-09-23)

- `window_set_name` now passes `clean_name_cstring` directly to the existing
  `WindowOwned.name` setter. Its intermediate C allocation, copy, and free
  are gone. The previous name remains owned through synchronous rename
  notifications, including a reentrant rename; invalid UTF-8 still leaves
  the name unchanged.
- Serialized workspace tests, binary build, and the window-name CLI comparison
  with pinned tmux passed. That script covers explicit rename and terminal
  ESC-k rename as well as other window-name paths. Changed-file rustfmt and
  diff checks passed. No sanitizer ran.

### Increment 423 — own explicit broken-pane name directly (2026-09-23)

- The explicit `break-pane -n` branch now moves `clean_name_cstring` into
  `WindowOwned.name` through `window_replace_name`. Its C allocation, copy,
  and free are gone. The default-name branch still accepts the C-owned result
  of `default_window_name`. Earlier `check_name` validation remains in place.
- Serialized workspace tests, binary build, and the window-name CLI comparison
  with pinned tmux passed, covering explicit and default broken-pane names.
  Changed-file rustfmt and diff checks passed. No sanitizer ran.

### Increment 424 — borrow default new-session cwd (2026-09-23)

- `cmd_new_session_exec` now borrows the default cwd from
  `server_client_get_cwd` until `session_create` immediately copies it into
  `SessionOwner.cwd`. The default-path `xstrdup` and its matching frees are
  gone. The formatted `-c` path still owns its C allocation and frees it on
  both exits. `server_client_open` between selection and creation does not
  replace the client cwd.
- Serialized workspace tests and binary build passed. The extended session
  cwd CLI comparison with pinned tmux covers detached creation without `-c`,
  explicit `-c`, attach replacement, and inherited pane cwd. Changed-file
  rustfmt and diff checks passed. No sanitizer ran.

### Increment 425 — own popup default cwd snapshot (2026-09-23)

- `cmd_display_popup_exec` now copies the default client/session cwd once into
  a local `CString` at its original capture point. `popup_display` passes that
  borrowed pointer to synchronous `job_run`, whose child uses a forked memory
  snapshot; no job or popup record retains cwd. The default-path C allocation
  and its three manual frees are gone. The formatted `-d` result still uses
  C-owned output and is freed on all exits. Keeping an owned snapshot avoids
  relying on cwd pointer stability during later title/style processing.
- Serialized workspace tests, binary build, existing popup owner CLI, and a
  new popup cwd CLI passed. The new CLI checks actual child `$PWD` for default
  and explicit `-d` paths with both candidate and pinned tmux. Changed-file
  rustfmt and diff checks passed. No sanitizer ran.

### Increment 426 — borrow default popup title literal (2026-09-23)

- `cmd_display_popup_exec` now borrows the empty C literal for its default
  title. `popup_display` and `popup_modify` synchronously copy that value into
  `PopupOwner`; the default-path `xstrdup` and matching frees are gone.
  Formatted `-T` titles retain their C-owned result and cleanup on all exits.
- Serialized workspace tests, binary build, popup cwd CLI, and popup owner
  CLI passed. The popup owner script also passed with pinned tmux, exercising
  explicit titles; the popup cwd script exercises the default title branch.
  Changed-file rustfmt and diff checks passed. No sanitizer ran.

### Increment 427 — own new-window cleaned name (2026-09-23)

- `cmd_new_window_exec` now keeps the cleaned `-n` name in a local `CString`
  through `-S` matching, diagnostics, `spawn_context.name`, and hooks.
  `spawn_window` copies the borrowed name into `WindowOwned.name` before
  returning. Four C-owned cleanup paths are gone; the separately formatted
  input remains C-owned until checked and freed.
- Serialized workspace tests and binary build passed. The extended
  window-name CLI comparison with pinned tmux passed for explicit names,
  `-S` reuse, ambiguous duplicate names, and a spawn error. Changed-file
  rustfmt and diff checks passed. No sanitizer ran.

### Increment 428 — own new-session cleaned names (2026-09-23)

- `cmd_new_session_exec` now keeps optional cleaned `-n` and `-s` names in
  local `CString` owners through attach, duplicate-name, spawn, and error
  paths. `session_create` and `spawn_window` copy borrowed pointers into
  `SessionOwner.name` and `WindowOwned.name`. Six manual C frees are gone;
  formatted-name inputs retain their existing C allocation and cleanup.
- Serialized workspace tests, binary build, existing session/window name CLI
  comparisons, and a focused new-session CLI comparison with pinned tmux
  passed. The focused check covers valid names, duplicate rejection, invalid
  UTF-8, and the `-A` branch's no-terminal error. A genuine `spawn_window`
  failure needs controlled fault injection and was not exercised by CLI.
  Changed-file rustfmt and diff checks passed. No sanitizer ran.

### Increment 429 — borrow default menu title literal (2026-09-23)

- `cmd_display_menu_exec` now lends the empty C literal to `menu_create` for
  the default title; `menu_create` copies it immediately into `MenuOwner.title`.
  Its default-path `xstrdup` and matching `free` are gone. A formatted `-T`
  title keeps its C-owned result and cleanup after creation.
- Serialized workspace tests and binary build passed. The extended attached
  menu CLI passed with candidate and pinned tmux: an untitled menu rendered,
  accepted its selection, and closed. Existing explicit-title cases also
  passed. Changed-file rustfmt and diff checks passed. No sanitizer ran.

### Increment 430 — own customize editor input directly (2026-09-23)

- `window_customize_start_edit` now holds option and key-command text in the
  existing private `CString` producers, and borrows key-note and environment
  C strings while `spawn_editor` synchronously writes its input. The old
  C-owned string preparation and final `free` are gone. Empty values still
  send one newline; `spawn_editor` retains no input pointer.
- Serialized workspace tests and binary build passed. Existing option editor
  and key prompt CLI checks passed with candidate and pinned tmux. The key
  prompt check now asserts the note editor's exact input bytes. A new
  environment editor CLI compares UTF-8 and empty input bytes with pinned
  tmux. Changed-file rustfmt and diff checks passed. No sanitizer ran.

### Increment 431 — own selected layout dump directly (2026-09-23)

- `layout_dump_owned` now returns a local `CString` from the private
  `LayoutString` serializer. `cmd_select_layout_exec` transfers it directly
  into the saved-layout owner, removing the C temporary, its copy, and its
  explicit `free`. Exported `layout_dump` still supplies a libc-freeable
  duplicate for format callbacks. The old control-client checksum uses the
  unchanged signed-char checksum routine; serialization preserves the former
  C formatter's first-NUL view.
- Serialized workspace tests and binary build passed. Existing layout CLI
  checks passed on candidate and pinned tmux. A new control-client CLI check
  compared exact old-format bytes, verified its checksum, and restored the
  prior layout with `select-layout -o`; it passed on both binaries. Changed-file
  rustfmt and diff checks passed. No sanitizer ran.

### Increment 432 — own startup socket path (2026-09-23)

- `main_0` now owns its selected `-S`, inherited `TMUX`, or label path in
  `Option<CString>` and lends it to global `socket_path` through `client_main`.
  Repeated `-S` replaces the prior owner directly; inherited `TMUX` is copied
  only through its first comma; private `make_label` builds its result from
  bytes without `xasprintf`. Removed the old path's `xstrdup` and `free`.
  Systemd socket activation may separately replace the global pointer.
- Serialized workspace tests, binary build, and existing socket label/base
  CLI checks passed. A new startup socket CLI compared candidate and pinned
  tmux for repeated raw-byte `-S`, `TMUX` selection and comma truncation,
  literal commas in `-S`, and exact `#{socket_path}` bytes. Changed-file
  rustfmt and diff checks passed. No sanitizer ran.

### Increment 433 — own printed event payload bytes locally (2026-09-23)

- `event_payload_item_print_owned` now copies evbuffer bytes into a local
  `Vec<u8>` with a trailing NUL. In-tree event format, verbose wait-event,
  and control notification consumers borrow that buffer synchronously and
  drop it without C-owned allocation or manual `free`. Exported
  `event_payload_item_print` and `event_payload_print` still duplicate into
  libc-freeable bytes. An interior NUL from a pointer print callback remains
  in the returned allocation; C `%s` consumers still see its first-NUL view.
- Serialized workspace tests and binary build passed. A new unit test checks
  exact `A\0B\0` bytes through both the owned and exported paths, plus the
  missing-item null result. Event format and verbose wait-event CLI checks
  passed with candidate and pinned tmux. The control notification fallback
  for a non-pane `pane` value was not directly exercised by CLI. Changed-file
  rustfmt and diff checks passed. No sanitizer ran.

### Increment 434 — own generated modified-key entries (2026-09-23)

- `input_key_tree` now retains boxed `InputKeyGenerated` records with owned
  `CString` data for the sequences created by `input_key_build`. The public
  entry lends its data pointer, and the numeric tree lends its entry pointer;
  both stay stable as the owner vector grows. Static default entries remain
  borrowed. Duplicate insertion keeps the first entry and drops the rejected
  owner. Removed generated-entry `xstrdup` and `xcalloc`; these entries still
  live for the server process, matching the old no-teardown lifecycle.
- Serialized workspace tests and binary build passed. A unit test checks
  generated pointer stability, bytes, and duplicate behavior. A new pane CLI
  compared exact sequences for modifier values 2 through 8 across function,
  arrow, and edit keys with pinned tmux. Changed-file rustfmt and diff checks
  passed. No sanitizer ran.

### Increment 435 — box standalone imsg buffer queues (2026-09-23)

- `ibufq_new` now uses fallible `Box::try_new`, initializes its intrusive tail
  pointer only after the record reaches its final heap address, and returns
  the same C-visible pointer. Allocation failure still returns null with
  `ENOMEM`. `ibufq_free` flushes linked buffers before reconstructing and
  dropping the Box, removing the standalone queue's `calloc`/`free` pair.
  Embedded queues inside `msgbuf` retain their separate lifecycle.
- Serialized workspace tests and binary build passed. A focused API test
  checks null free, empty initialization, push/pop, concat links and counts,
  flush, and queued-buffer cleanup on free. There is no in-tree production
  caller of standalone `ibufq_new`/`ibufq_free`, so no binary E2E reaches this
  constructor. Allocation failure was not fault-injected. Changed-file
  rustfmt and diff checks passed. No sanitizer ran.

### Increment 436 — own relative pane cwd concatenation (2026-09-23)

- `spawn_pane` now combines the first-NUL bytes of the base cwd and its local
  relative `CString` directly into one `CString`. It inserts `/` only for a
  nonempty relative value, preserving the former `%s%s%s` behavior, including
  glibc's `(null)` rendering for a null base. Removed the temporary
  `xasprintf`, copy, and `free`; the existing pane cwd owner receives the
  result at the same later transfer point.
- Serialized workspace tests, binary build, and pane-cwd CLI comparison with
  pinned tmux passed. The CLI now also creates a pane from a non-UTF-8
  relative `-c` value and checks both its start and process cwd. Changed-file
  rustfmt and diff checks passed. No sanitizer ran.

### Increment 437 — own default window names directly (2026-09-23)

- Private `default_window_name_cstring` and `parse_window_name_cstring` now
  produce the cleaned name as a `CString`. `spawn_window` and `break-pane`
  transfer it directly to `WindowOwned.name`, removing their C allocation,
  copy, free, and the obsolete `window_replace_name_from_c_owned` helper.
  Exported `default_window_name` and `parse_window_name` still return
  libc-freeable duplicates for out-of-tree and format callback consumers.
- Serialized workspace tests and binary build passed. Parser API tests and
  existing window-name and argv CLI comparisons passed. A new exact-byte CLI
  compared candidate and pinned tmux for default names from spawned windows
  and broken panes, including quoted paths and invalid UTF-8 argv bytes.
  A unit test covers the no-active-pane empty result through both APIs.
  Changed-file rustfmt and diff checks passed. No sanitizer ran.

### Increment 438 — own intermediate argument template substitutions (2026-09-23)

- `args_copy_copy_value` now uses `CString` for results between successive
  `cmd_template_replace_cstring` calls. The first substitution borrows the
  source value, and each subsequent result drops when replaced. This removes
  the intermediate C allocation/free loop without adding an allocation;
  zero substitutions still duplicate the source directly. The final
  `args_value.string` remains a libc-freeable duplicate for `args_free_value`.
- A focused argument-conversion test covers two substitutions, single-quote
  escaping, named and positional values, source destruction before reading
  the copy, and final `args_free`. All 12 argument-conversion tests, serialized
  workspace tests, binary build, changed-file rustfmt, and diff check passed.
  No sanitizer ran.

### Increment 439 — own customize key Command prompt text (2026-09-23)

- `window_customize_set_key` now passes `cmd_list_print_cstring`'s owned
  result to `mode_tree_set_prompt` for its Command branch. The prompt copies
  the input into its own cells synchronously, so the local `CString` drops
  after the call. Removed the exported C-return duplicate and manual `free`
  from this production path. The exported `cmd_list_print` contract remains.
- The attached-client customize set-key PTY script passed against both the
  candidate and pinned tmux, covering the Command and Note prompt text and
  edits. Serialized workspace tests, binary build, changed-file rustfmt, and
  diff check passed. No sanitizer ran.

### Increment 440 — own customize option prompt text (2026-09-23)

- `window_customize_set_option` now keeps the existing
  `options_to_cstring` result through `mode_tree_set_prompt`. The prompt
  copies the text into owned cells before returning, so this production
  path no longer makes an exported C-return duplicate or manually frees it.
  The exported `options_to_string` contract remains unchanged.
- The attached-client customize set-option prompt script passed for the
  candidate and pinned tmux. Serialized workspace tests, binary build,
  changed-file rustfmt, and diff check passed. No sanitizer ran.

### Increment 441 — own customize default option text (2026-09-23)

- Private `options_default_to_cstring` now formats built-in scalar defaults
  as owned `CString` values. `window_customize_option_is_changed` borrows its
  default for comparison, and `window_customize_draw_option` retains an
  optional owned default only when it differs from the current value. Their
  C allocation/free paths are removed. Exported
  `options_default_to_string` still returns a libc-freeable duplicate.
- The customize option prompt CLI now checks the rendered default text.
  Candidate and pinned tmux passed that script plus customize reset and
  array-key prompt scripts. Serialized workspace tests, binary build,
  changed-file rustfmt, and diff check passed. No sanitizer ran.

### Increment 442 — own customize changed-value comparisons (2026-09-23)

- `window_customize_option_is_changed` now borrows `CString` results from
  `options_to_cstring` for both current and default array values and for the
  scalar current value. It drops the two array snapshots before freeing the
  temporary defaults tree, preserving the former release order. Removed
  the C-return copies and manual frees from this comparison path.
- A new attached-client CLI check toggles the `C` changed-only filter for
  equal and changed scalar and array values. Candidate and pinned tmux
  passed the same four cases. Serialized workspace tests, binary build,
  changed-file rustfmt, and diff check passed. No sanitizer ran.

### Increment 443 — own customize array-key value snapshot (2026-09-23)

- `window_customize_set_array_key_callback` now keeps the old array value
  as a `CString` from `options_to_cstring` while `options_array_set` parses
  or copies it. Explicit drops preserve the former free points before error
  display and before deleting the old key on success. Removed the
  C-return duplicate and branch-specific manual frees.
- The candidate and pinned tmux passed the customize array-key prompt CLI
  check. Serialized workspace tests, binary build, changed-file rustfmt,
  and diff check passed. No sanitizer ran.

### Increment 444 — own customize option-row text (2026-09-23)

- `window_customize_build_array` and `window_customize_build_option` now
  use `options_to_cstring` for each row's temporary value. `format_add`
  copies the bytes into its own entry before return. Removed both exported
  C-return duplicates and manual frees; the array-row owner drops at the
  former free point after its displayed text is built.
- Candidate and pinned tmux passed customize array-name and changed-only
  CLI checks. Serialized workspace tests, binary build, changed-file
  rustfmt, and diff check passed. No sanitizer ran.

### Increment 445 — own queued error callback text (2026-09-23)

- `CmdqItemOwner` now owns `cmdq_get_error`'s duplicated message as a
  `CString`; `cmdq_error_callback` borrows it. Normal queue removal and
  `cmdq_free_detached` both drop the item owner, so the message is also
  released if the callback never fires. The exported constructor still
  returns a stable `cmdq_item` with a borrowed `data` pointer.
- A focused test changes the source buffer after creating an error item,
  checks exact non-UTF-8 message bytes, then frees the detached item.
  Serialized workspace tests, binary build, changed-file rustfmt, and
  diff check passed. No sanitizer ran. The broader disconnected-client
  queue cleanup is still pending.

### Increment 446 — release unfired callback payloads (2026-09-23)

- `CmdqItemOwner` now holds an optional cancellation destructor. Queue removal
  and detached-item destruction call it only before a callback fires. This
  releases parser-owned `control_error` text, boxed queued key events and
  their optional client reference, counted mode-tree and window-tree
  references, and source-file completion data and depth. Normal callbacks
  retain their existing release paths. Other queue callbacks carry borrowed
  or null data.
- A focused detached-item test checks that the cancellation destructor runs.
  Serialized workspace tests, binary build, key, tree, source-file, and
  control-related CLI checks, changed-file rustfmt, and diff check passed.
  No sanitizer ran. File-wait teardown remains pending.

### Increment 447 — cancel disconnected file-backed queue waits (2026-09-23)

- `FileOwner` registers file-backed command waits before terminal events can
  be scheduled, including immediate read/write errors. `file_cancel_cmdq_wait`
  suppresses callbacks and drops owned or explicitly cancelled payloads;
  terminal events still release the file. Registered waits on dead clients
  receive no progress or terminal callbacks, including a closed stream.
- `CmdqItemOwner` tracks the live file wait. `server_client_lost` cancels a
  registered waiting head item, then removes the queued suffix and its owned
  callback payloads. Source-file reads, pane stdin, load-buffer, and
  save-buffer register their waits. Source-file and pane cancellation release
  their raw callback data and client refs; load-buffer drops its boxed data;
  save-buffer only borrowed its queue item. Pane progress errors now retain
  callback data through terminal cleanup.
- The held-open stdin E2E proves the pinned baseline retains a disconnected
  `source-file -` client while the candidate frees it; its queued suffix never
  runs and the server remains responsive. Serialized workspace tests, binary
  build, source/config, pane input, load/save, changed-file rustfmt, and diff
  checks passed. No sanitizer ran. Queue waits without a registered file
  owner remain outside this scoped disconnect drain.

## Historical migration index

Each retained increment was committed separately; increment 228 was reverted.
For the detailed pre-compaction execution and validation log, use
`git show df2e2b0:plan-ownership.md`.

Numbers 6–139 were never individually recorded in this document.

### Increments 1–5

- 1: list-keys prefix
- 2: JSON object-key scratch
- 3: refresh-client pane argument scratch
- 4: monitor subscription parse scratch
- 5: layout serializer scratch buffer

### Increments 140–199

- 140: numeric prompt callback text
- 141: prompt accept/history callback text
- 142: grid hyperlink ID scratch
- 143: single-key prompt callback text
- 144: copy-mode jump target cells
- 145: command message username suffix
- 146: debug log filename
- 147: pane private mode list scratch
- 148: session attached-client list scratch
- 149: Linux proc lookup paths
- 150: window linked-session list scratch
- 151: window active-session list scratch
- 152: window active-client list scratch
- 153: window-tree numeric row names
- 154: session-group name list scratch
- 155: grouped attached-client list scratch
- 156: event payload format keys
- 157: pane tab-stop list scratch
- 158: pane start-command list scratch
- 159: terminal selection base64 scratch
- 160: single-quote shell format scratch
- 161: recursive command-parser log prefix
- 162: prompt history file path
- 163: mouse completion replacement scratch
- 164: run-shell status message scratch
- 165: display-panes command argument scratch
- 166: pane search glob pattern
- 167: monitor-hook append value scratch
- 168: switch-mode command target scratch
- 169: menu item name owner
- 170: capture-pane cell formatting scratch
- 171: mode-tree menu title scratch
- 172: mode-tree preview label scratch
- 173: customize array option name scratch
- 174: window-tree prompt label scratch
- 175: mode-tree row scratch
- 176: customize key-table title scratch
- 177: capture-pane grid header and row scratch
- 178: pane login-record scratch
- 179: socket label base scratch
- 180: customize key detail scratch
- 181: systemd pane description scratch
- 182: editor command scratch
- 183: customize environment entry scratch
- 184: systemd UUID scope-name scratch
- 185: customize new-key prompt scratch
- 186: customize environment prompt scratch
- 187: customize new-user prompt scratch
- 188: customize array-key prompt scratch
- 189: customize set-option prompt scratch
- 190: customize key-binding prompt scratch
- 191: customize unset confirmation prompt scratch
- 192: customize reset confirmation prompt scratch
- 193: customize scope-text owner
- 194: window-tree target owner
- 195: startup path expansion owner
- 196: client lockfile owner across daemon fork
- 197: hook monitor string owner
- 198: shell argv0 owner for Rust callers
- 199: command-completion scratch owner

### Increments 200–259

- 200: capture-pane cell scratch owner
- 201: socket-label error owner
- 202: option value rendering scratch owner
- 203: JSON tokenizer token owner
- 204: argument escaping scratch owner
- 205: set-buffer scratch name owner
- 206: command-list print buffer owner
- 207: format quote modifier scratch owner
- 208: printed command string owner
- 209: terminal override scratch value owner
- 210: parsed window-copy text owner
- 211: printed arguments owner
- 212: key-binding log scratch owner
- 213: customize key scratch owners
- 214: hook debug print scratch owner
- 215: parser command-print scratch owners
- 216: command-option print owner
- 217: list-keys command text owner
- 218: command-prompt name scratch owner
- 219: detach exec message byte owner
- 220: server exit message payload owner
- 221: client command packet byte owner
- 222: format modifier unescape scratch owner
- 223: stripped time-format owner
- 224: argv stringification owner
- 225: config-file format assembly owner
- 226: format expansion output owner
- 227: client file path record owner
- 228: retracted argument cache owner; see correction below
- 229: hyperlink external ID record owner
- 230: format job record and retained string owner
- 231: customize change option-name owner
- 232: environment assignment name scratch owner
- 233: retained hyperlink URI and internal ID owners
- 234: command-template output owner
- 235: copy-mode match text output owner
- 236: regex substitution output owner
- 237: format trim output owner
- 238: detached show-buffer path borrow
- 239: copy-mode regex string buffer owner
- 240: Linux process command owner
- 241: format job completion output owner
- 242: capture-pane output buffer owner
- 243: positional argument command-cache owner
- 244: file write message scratch owner
- 245: file read message scratch owner
- 246: command lexer scratch owner
- 247: window copy selection owner
- 248: client exit message owner
- 249: grid string cell output owner
- 250: input parser buffer owner
- 251: fallback visible range storage
- 252: pane input callback record owner
- 253: format draw range scratch owner
- 254: file read-open message scratch owner
- 255: file write-open message scratch owner
- 256: pane visible range owner
- 257: display-panes preview screen owner
- 258: session termios owner
- 259: queued key-event owner

### Increments 260–319

- 260: confirmation prompt callback owner
- 261: clock mode data owner
- 262: run-shell callback data owner
- 263: event sink owner
- 264: if-shell callback data owner
- 265: server ACL entry owner
- 266: wait-for event waiter owner
- 267: saved status screen owner
- 268: menu display owner
- 269: choose-buffer editor owner
- 270: wait-for channel and queue owners
- 271: spawned editor state and path
- 272: popup overlay record and strings
- 273: status prompt callback record
- 274: customize-mode format string
- 275: choose-client mode strings
- 276: choose-tree mode strings
- 277: mode-tree prompt callback record
- 278: choose-buffer mode strings
- 279: switch-mode strings and mutable filter
- 280: mode-tree menu callback record
- 281: mode-tree item and row strings
- 282: mode-tree filter and search strings
- 283: redraw scene spans
- 284: option rollback string snapshot
- 285: redraw scene record
- 286: customize editor value buffer
- 287: customize editor callback record
- 288: spawn environment log prefix
- 289: fallible server log message owner
- 290: borrowed CLI socket label
- 291: borrowed unformatted prompt input
- 292: boxed screen selection
- 293: boxed copy-mode search marks
- 294: owned hook-monitor record and format
- 295: boxed outer environment record
- 296: boxed style-range list nodes
- 297: boxed outer format tree
- 298: boxed terminal-key trie nodes
- 299: boxed options-array item records
- 300: boxed outer options record
- 301: boxed monitor set record
- 302: boxed key binding records
- 303: boxed key table records
- 304: boxed monitor item records
- 305: boxed monitor pane leaves
- 306: boxed monitor window leaves
- 307: boxed paste buffer records
- 308: boxed dynamic UTF-8 width entries
- 309: boxed outer event payload records
- 310: boxed outer hyperlinks record
- 311: boxed screen title stack entries
- 312: boxed screen title stack header
- 313: boxed argument flag entries
- 314: boxed argument command states
- 315: boxed command queue records
- 316: boxed JSON parser nodes
- 317: boxed control-window size records
- 318: boxed control-pane records
- 319: boxed local colour palette array

### Increments 320–356

- 320: boxed default colour palette array
- 321: boxed layout cells
- 322: boxed grid records
- 323: boxed command queue items
- 324: boxed command queue states
- 325: boxed screen write-line array
- 326: boxed synchronized-output dirty bitmap
- 327: boxed redraw scene line array
- 328: boxed terminal record
- 329: boxed pane prompt data
- 330: boxed pane mode entries
- 331: boxed terminal capability array
- 332: boxed non-string event payload items
- 333: owned terminal name
- 334: boxed screen write-line data
- 335: boxed window-copy backing screen
- 336: boxed control-client state
- 337: boxed process peers
- 338: boxed jobs
- 339: owned new-session group prefix
- 340: boxed parser conditional scopes
- 341: boxed parser command-list headers
- 342: owned load-buffer callback data
- 343: boxed parser argument-list headers
- 344: boxed command-list queue headers
- 345: boxed command records
- 346: boxed command lists
- 347: boxed parser command records
- 348: boxed parser argument records
- 349: boxed window records
- 350: owned UTF-8 item index
- 351: boxed linked flag argument values
- 352: parser temporary argument vector
- 353: owned positional values inside args
- 354: owned client command argument temporaries
- 355: run-shell working directory owner
- 356: popup overlay range storage

## Historical corrections and compatibility facts

- Increment 228 was reverted: its argument cache used pointer addresses as HashMap keys. The command-list cache remains C-owned until the movable args_value record can own it without changing ABI or copy behavior.
- Private repr(C) owners lend public raw fields only until replacement or final destruction. Exported C-return APIs still supply libc-freeable output where their callers require it; a local CString that merely adds a copy is not a completed boundary.
- The detached LayoutDescription API and its API-only tests were removed after equivalence review. layout_parse again builds layout_cell and layout_parse_ctx directly; the LayoutString serializer owner remains.
- screen_print uses one inline static 16 KiB result buffer. Its pointer is stable across calls but the content is overwritten on the next call.
- args_set retains an out-of-repo Box record contract. Do not change that allocation path solely for in-repo callers.
- utf8_sanitize preserves the first-NUL C-string view and the old fatal for a leading zero-width cell. Its Rust allocation failure diagnostic can differ from xreallocarray; a malformed complete UTF-8 rewind was corrected to retry from the candidate start.
- log_vwrite, which calls vasprintf directly, has a supported interior-NUL E2E. No supported interior-NUL output was found in xvasprintf callers; the binary A\0B show-buffer E2E reaches window_copy_vadd but formats only A.
- OSC 52 decoded bytes move to the paste owner without a C allocation;
  set-buffer producer data is copied and freed on acceptance. `paste_set`
  errors leave producer data with the caller. The public data pointer borrows
  the owner until replacement or deletion.

## Validation history and known test conditions

- Per-increment validation is in the corresponding commits and the archived detailed log. The current normal gate is serialized workspace tests, a binary build, relevant focused CLI or API checks, changed-file rustfmt with edition 2021, and git diff --check.
- A cached hmux-rt test binary once retained a deleted worktree path; cargo clean -p hmux-rt corrected it. Timestamp-based PTY tests have collided under parallel runs, so use RUST_TEST_THREADS=1 for the full suite.
- Some generated translation files have pre-existing rustfmt import-order differences. Compare a formatting complaint with the pre-migration file before treating it as caused by a boundary change.
- The pinned baseline customize-mode subprocess timed out during increment 365; candidate-only customize mutation checks passed and the spawned baseline test processes were terminated. No combined sanitizer run is recorded for the recent increments.
