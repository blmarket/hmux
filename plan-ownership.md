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

### Increment 140 — numeric prompt callback text (2026-09-22)

- `prompt_key`'s numeric non-digit path now owns the buffer text via
  `utf8_tocstr_cstring` through its synchronous close callback. Removed the
  local `utf8_tocstr` allocation and manual free; closed-state and return
  behavior are unchanged.
- Extended the attached-client mutable-prompt check: `command-prompt -N`
  receives `42x`, submits `42` without Enter, and closes. Isolated
  library/binary build, workspace tests, attached-client check, and
  `git diff --check` passed. No sanitizer was run.

### Increment 141 — prompt accept/history callback text (2026-09-22)

- Replaced all 26 identical `prompt_key` accept/history scratch paths with
  `prompt_done_with_history`. It owns a `CString` converted directly from the
  editable UTF-8 cells, skips empty history input, records history before
  invoking `prompt_done`, and drops the text after the callback. This removes
  26 `utf8_tocstr` allocations and 26 manual frees without changing the
  numeric, single-key, or final incremental branches.
- The helper preserves arbitrary non-UTF-8 bytes and the first-NUL view
  that `prompt_add_history` and `prompt_done` receive as C strings. The
  existing history owner test and attached-client history recall check pass;
  isolated library/binary build and `git diff --check` also pass. No
  sanitizer was run.

### Increment 142 — grid hyperlink ID scratch (2026-09-22)

- `grid_string_cells_add_hyperlink` now owns its temporary `id=%s;`
  fragment as a byte-preserving `CString` built from the borrowed ID's
  first-NUL C-string view. `strlcat` consumes that pointer synchronously;
  removed the only `xasprintf` import in `src/grid/core.rs` and the local
  allocation/free pair. Length checks, empty-ID output, and escape modes
  remain unchanged.
- Isolated library/binary build, `tests/osc8_hyperlink_id.rs`, changed-file
  rustfmt, and `git diff --check` passed. The OSC 8 test verifies captured
  hyperlink IDs and exact escape output. No sanitizer was run.

### Increment 143 — single-key prompt callback text (2026-09-22)

- The `PROMPT_SINGLE` branch in `prompt_key` now owns its local text with
  `utf8_tocstr_cstring` through `prompt_done`. Removed the final production
  `utf8_tocstr` allocation/free pair in `prompt.rs`, its obsolete `s`
  scratch variable, and the legacy import. The callback result and possible
  final incremental callback path are unchanged.
- Extended the attached-client mutable-prompt check to submit a Unicode
  single character with `command-prompt -1`. Isolated build, focused
  conversion test, mutable/completion/input-format/paste CLI checks, and
  `git diff --check` passed. No sanitizer was run.

### Increment 144 — copy-mode jump target cells (2026-09-22)

- `window_copy_mode_data.jumpchar` now owns its decoded, size-zero-terminated
  cells in a `Vec<utf8_data>`. All four jump commands replace that Vec; the
  four cursor-search calls borrow its cells for their synchronous reads.
  Removed the four replacement frees and the teardown free.
- Audited the containing record: its sole allocation in
  `window_copy_common_init` now uses a Box. The translated C fields retain
  their zeroed initial state, while the Vec is initialized before the record
  is used. Removed `Copy`/`Clone`; `window_copy_free` drops the Box after the
  existing event, input, and screen teardown. `window_mode_entry.data`
  remains a raw compatibility pointer to the stable Box allocation.
- Isolated library/binary build, copy-regex test, changed-file rustfmt, and
  `git diff --check` passed. New attached-client
  `scripts/copy_jump_cli_checks.py` covers four jump commands, repeat and
  reverse, Unicode targets, replacement, and mode teardown/reentry. No
  sanitizer was run.

### Increment 145 — command message username suffix (2026-09-22)

- `cmdq_add_message` now owns its local `[username]`, `[unknown]`, or empty
  suffix as a `CString`. The passwd name is read as C-string bytes without
  assuming UTF-8, then bracketed; `server_add_message` formats and copies
  the borrowed pointer synchronously. Removed the suffix's `xasprintf` and
  two `xstrdup` allocations and its manual free. The separate `cmd_print`
  result remains C-owned by its producer and is still freed here.
- Isolated library/binary build, `scripts/server_messages_cli_checks.py`,
  changed-file rustfmt, and `git diff --check` passed. The CLI exercises
  same-UID command messages and message retention; a peer with a different
  UID was not available for an attached-client test. No sanitizer was run.

### Increment 146 — debug log filename (2026-09-22)

- `log_open` now owns `tmux-{name}-{pid}.log` in a local, byte-preserving
  `CString` through synchronous `fopen`. Removed the local `xasprintf`
  allocation/free pair. The internal `proc_start` callers supply the fixed
  `server` and `client` names; `log_toggle` uses the process name copied
  from that input. The file and its `FILE *` lifecycle are unchanged.
- Isolated library/binary build, changed-file rustfmt, and `git diff --check`
  passed. A detached `-vv` server created a nonempty
  `tmux-server-<pid>.log` and shut down successfully. No sanitizer was run.

### Increment 147 — pane private mode list scratch (2026-09-22)

- `format_cb_pane_private_modes` now appends its comma-separated numeric mode
  list to a Rust `String`, avoiding a C allocation and free for each added
  mode. One `xstrdup` still returns a C-owned result required by the format
  callback cache, which libc-frees it. The no-pane null result, empty string,
  table order, and cursor-blink filtering are unchanged.
- Isolated library/binary build, changed-file rustfmt, and `git diff --check`
  passed. New live-server `scripts/pane_private_modes_cli_checks.py` passed
  with both old and new callbacks for default, empty, multiple, blinking,
  and cleared modes. No sanitizer was run.

### Increment 148 — session attached-client list scratch (2026-09-22)

- `format_cb_session_attached_list` now builds client-name bytes and commas
  in a local `Vec<u8>`, borrowing each C-string name without UTF-8 conversion.
  Removed the intermediate `evbuffer_new`/`evbuffer_free` lifecycle and its
  append/pullup calls. One `xmemdup` still produces the C-owned callback
  result, and an empty list still returns null. Global client traversal order
  and the rule that an empty name contributes no bytes remain unchanged.
- Isolated library/binary build, changed-file rustfmt, and `git diff --check`
  passed. New `scripts/session_attached_list_cli_checks.py` passed with both
  the old and new binaries for empty, multiple, other-session, and detached
  client cases. No sanitizer was run.

### Increment 149 — Linux proc lookup paths (2026-09-22)

- `osdep_get_name` and `osdep_get_cwd` now own all three local
  `/proc/<pid>/cmdline` and `/proc/<pid>/cwd` path strings in `CString`s.
  `fopen` and `readlink` borrow their pointers synchronously; removed three
  `xasprintf` allocations and matching frees. Exported function signatures
  and the returned C-owned command-name buffer are unchanged.
- Isolated library/binary build, changed-file rustfmt, and `git diff --check`
  passed. New `scripts/osdep_proc_cli_checks.py` passed with both old and
  new binaries for a live pane's current command and cwd. The rare cwd SID
  fallback is source-audited but not exercised by that E2E. No sanitizer was
  run.

### Increment 150 — window linked-session list scratch (2026-09-22)

- `format_cb_window_linked_sessions_list` now builds the ordered session-name
  bytes in a local `Vec<u8>`, removing its intermediate `evbuffer` allocation,
  append, pullup, and free. It still returns a C-owned `xmemdup` result for
  the format callback and null for an empty byte list. Duplicate linked
  sessions and non-UTF-8 bytes keep their existing output semantics.
- Isolated library/binary build, changed-file rustfmt, and `git diff --check`
  passed. New `scripts/format_linked_sessions_cli_checks.py` passed with old
  and new binaries for single and duplicate links, a UTF-8 session name,
  unlink, and session teardown. No sanitizer was run.

### Increment 151 — window active-session list scratch (2026-09-22)

- `format_cb_window_active_sessions_list` now assembles the filtered,
  comma-separated session-name bytes in a local `Vec<u8>`. Removed its
  `evbuffer` allocation/free and append/pullup calls; the final `xmemdup`
  still supplies the C-owned callback result. The `session.curw == wl`
  filter, winlink order, and null result for an empty byte list remain.
- Isolated library/binary build, changed-file rustfmt, and `git diff --check`
  passed. New `scripts/format_active_sessions_cli_checks.py` passed against
  old and new binaries for inactive links, active links, duplicate links,
  Unicode names, and an empty result. No sanitizer was run.

### Increment 152 — window active-client list scratch (2026-09-22)

- `format_cb_window_active_clients_list` now collects filtered client-name
  bytes in a local `Vec<u8>`, removing its `evbuffer` allocation/free and
  append/pullup calls. It retains the client traversal order, current-window
  filter, separator-on-nonempty rule, null result for no bytes, and final
  C-owned `xmemdup` callback result.
- Isolated library/binary build, changed-file rustfmt, and `git diff --check`
  passed. New `scripts/format_active_clients_cli_checks.py` passed against
  old and new binaries for empty, multiple, linked-window, and detached
  client cases. No sanitizer was run.

### Increment 153 — window-tree numeric row names (2026-09-22)

- `window_tree_build_pane` and `window_tree_build_window` now own their
  decimal mode-tree row names in local `CString`s. Removed two `xasprintf`
  allocations and matching frees. `mode_tree_add_identity` duplicates each
  borrowed name synchronously; pane indexes still format as `%u`, and the
  signed window index is cast to `u_int` to preserve its former `%u` bytes.
  The separately allocated row text remains with its existing producer.
- Isolated library/binary build, five mode-tree unit tests, changed-file
  rustfmt, and `git diff --check` passed. An attached-client E2E created a
  split pane and second window, then entered and exited `choose-tree`.
  No sanitizer was run.

### Increment 154 — session-group name list scratch (2026-09-22)

- `format_cb_session_group_list` now appends group session-name bytes into a
  local `Vec<u8>`, removing its `evbuffer` allocation/free and append/pullup
  calls. It retains group traversal order, no-group null result,
  separator-on-nonempty behavior, and final C-owned `xmemdup` callback
  result. The name bytes are read as C strings without UTF-8 conversion.
- Isolated library/binary build, changed-file rustfmt, and `git diff --check`
  passed. New `scripts/format_session_group_list_cli_checks.py` passed with
  old and new binaries for an ungrouped session, multiple grouped sessions,
  a Unicode name, rename, and teardown. No sanitizer was run.

### Increment 155 — grouped attached-client list scratch (2026-09-22)

- `format_cb_session_group_attached_list` now collects the names of clients
  attached to any session in the group in a local `Vec<u8>`. Removed its
  `evbuffer` allocation/free and append/pullup calls. The nested membership
  scan, global client order, separator-on-nonempty rule, null result for no
  bytes, and final C-owned `xmemdup` callback result remain unchanged.
- Isolated library/binary build, changed-file rustfmt, and `git diff --check`
  passed. New `scripts/format_group_attached_cli_checks.py` passed with old
  and new binaries for empty grouped/ungrouped sessions, clients on both
  group members and an outsider, and detach. No sanitizer was run.

### Increment 156 — event payload format keys (2026-09-22)

- `event_payload_add_formats` now owns its `prefix + key` and optional
  `_name` format keys in local, byte-preserving `CString`s. Removed the three
  `xasprintf` allocation/free pairs. `format_add` copies each key into its
  format entry synchronously; the separate C-owned
  `event_payload_item_print` result keeps its existing free.
- Isolated library/binary build, focused event-payload unit test,
  changed-file rustfmt, and `git diff --check` passed. New
  `scripts/event_payload_formats_cli_checks.py` passed with old and new
  binaries for window-renamed and session-created hook formats, including
  `_name` keys and a Unicode window name. No sanitizer was run.

### Increment 157 — pane tab-stop list scratch (2026-09-22)

- `format_cb_pane_tabs` now writes comma-separated decimal tab positions
  into a local Rust `String`, removing the callback file's final local
  `evbuffer` allocation/free and append/pullup calls. It retains the tab-bit
  traversal, no-pane null result, null for an empty list, and final C-owned
  `xmemdup` callback result.
- Isolated library/binary build, changed-file rustfmt, and `git diff --check`
  passed. New `scripts/pane_tabs_cli_checks.py` passed on old and new binaries
  for default tab stops, all cleared stops, and two custom stops set by
  terminal escape sequences. No sanitizer was run.

### Increment 158 — pane start-command list scratch (2026-09-22)

- `format_cb_start_command_list` now accumulates shell-quoted argument bytes
  in a local `Vec<u8>`, removing its manual `xrealloc` growth and `strlcat`
  assembly. One `xmemdup` supplies the required C-owned callback result;
  `format_quote_shell_single` still returns separately C-owned quoted text
  that this caller frees after copying. No-pane and negative-argc states
  still return null, while argc zero returns an allocated empty string.
- Isolated library/binary build, changed-file rustfmt, and `git diff --check`
  passed. New `scripts/format_start_command_list_cli_checks.py` passed with
  old and new binaries for no-pane/empty, spacing, apostrophe quoting,
  non-UTF-8 bytes, and a long argument. No sanitizer was run.

### Increment 159 — terminal selection base64 scratch (2026-09-22)

- `tty_set_selection` now owns the local `__b64_ntop` output in a `Vec<u8>`
  through synchronous `tty_putcode_ss`/terminal buffer output. Removed its
  `xmalloc` allocation/free pair and sole `xmalloc` import in `tty.rs`.
  Early returns for an unstarted tty or missing selection capability remain;
  the OSC 52 payload and NUL terminator are preserved.
- Isolated focused `bracketed_paste_pty` suite (3 tests), changed-file
  rustfmt, and `git diff --check` passed. Its new attached-PTY test checks
  OSC 52 output from `set-buffer -w` for text and `load-buffer -w` for
  binary `A\0B`. No sanitizer was run.

### Increment 160 — single-quote shell format scratch (2026-09-22)

- The internal `format_quote_shell_single` helper now returns a
  byte-preserving `CString`. `format_cb_start_command_list` borrows each
  quoted argument directly, removing its per-argument C allocation/free.
  `format_replace_modifier` still makes one C-owned copy for its transform
  chain and frees the prior value at the same point. The helper has no
  exported ABI and has no other callers.
- Isolated library/binary build, changed-file rustfmt, and `git diff --check`
  passed. Extended `scripts/format_start_command_list_cli_checks.py` passed
  with old and new binaries for the `q,s` modifier's plain, apostrophe,
  non-UTF-8, and empty values, alongside the existing pane argv cases.
  No sanitizer was run.

### Increment 161 — recursive command-parser log prefix (2026-09-22)

- The recursive case of `cmd_parse_log_commands` now owns its temporary
  `prefix + " %u:%u"` text in a byte-preserving local `CString`, removing
  its `xasprintf` allocation/free pair. The recursive call borrows the
  pointer synchronously. Both top-level prefixes are fixed non-null C
  strings; `i` and `j` retain `%u` decimal formatting. The separate
  `cmd_list_print` result remains C-owned and freed in its own case.
- Isolated library/binary build, focused `source_file_pattern` test,
  changed-file rustfmt, and `git diff --check` passed. A private-socket
  `-vv` source-file scenario produced a nested
  `cmd_parse_build_commands 0:3 0:0` log prefix. No sanitizer was run.

### Increment 162 — prompt history file path (2026-09-22)

- The private `prompt_find_history_file` helper now returns
  `Option<CString>`, and both `prompt_load_history` and
  `prompt_save_history` borrow that path through logging and `fopen` before
  dropping it. Removed the absolute-path `xstrdup`, the `~/` `xasprintf`,
  and the callers' four success/error frees. Raw C-string bytes are
  preserved; empty and unsupported relative paths still return no path.
- Isolated library/binary build, `prompt_history_owner` integration test,
  changed-file rustfmt, and `git diff --check` passed. A live private-HOME
  server using `history-file '~/history'` created the history file during
  shutdown. The absolute-path and invalid-relative branches were source
  audited but not exercised by that E2E. No sanitizer was run.

### Increment 163 — mouse completion replacement scratch (2026-09-22)

- `prompt_mouse_complete` now copies the selected completion name's C-string
  bytes, appends its trailing space, and owns the result in a local `CString`.
  `prompt_replace_complete` only reads that pointer synchronously, including
  its early-return path. Removed the local `xasprintf` allocation/free pair;
  the independent copy remains valid if replacement clears the completion
  choices. Existing prompt text and C callback contracts remain unchanged.
- The library/binary build and extended `prompt_completion_cli_checks.py`
  passed. The attached-client scenario enables mouse input, clicks a displayed
  completion choice, submits the prompt, and checks the selected command with
  its trailing space. `git diff --check` passed. Changed-file rustfmt reports
  the two pre-existing import-layout differences in `prompt.rs`. No sanitizer
  was run.

### Increment 164 — run-shell status message scratch (2026-09-22)

- `cmd_run_shell_callback` now owns its optional nonzero-exit or signal
  message in a local byte-preserving `CString`. Removed both `xasprintf`
  branches and the matching free. `cmd_run_shell_print` borrows the message
  synchronously and formats its output before the local owner is dropped;
  `cdata.cmd` remains owned by the existing callback data lifecycle.
- Isolated library/binary build, `run_shell_partial_line` integration test,
  changed-file rustfmt, and `git diff --check` passed. Private-socket E2E
  output matched the pre-migration binary for `exit 7`, `kill -TERM $$`, and
  a command containing byte `0xff`. The view-pane output path was source
  audited but not live tested. No sanitizer was run.

### Increment 165 — display-panes command argument scratch (2026-09-22)

- `window_panes_run_command` now owns the formatted `%<pane-id>` argument as
  a local `CString`. `args_make_commands` only borrows its temporary argv
  pointer while synchronously substituting and parsing the command, so the
  scratch is dropped when this function returns. Removed its `xasprintf` and
  matching free; no queued command retains the pointer.
- Isolated library/binary build, changed-file rustfmt, and `git diff --check`
  passed. The new attached-client `display_panes_command_cli_checks.py`
  splits a window, selects pane 1 in display-panes mode, and verifies the
  custom command receives the exact `%<pane-id>` text. It passed with both
  the pre-migration and migrated binaries. Only the success path was live
  tested; no sanitizer was run.

### Increment 166 — pane search glob pattern (2026-09-22)

- `window_pane_search` now builds the nonregex `"*%s*"` pattern from the
  C-string bytes of the search term and owns it in a local `CString`.
  `fnmatch` borrows it during the synchronous grid scan; the local owner
  drops after matching. Removed its `xasprintf`/free pair. The regex path
  still compiles and releases its separate `regex_t` with `regfree`.
- The library/binary build, `format_search_cli_checks.py`, and
  `git diff --check` passed. The private-server CLI check passed with both
  the pre-migration and migrated binaries for first/second-line hits,
  misses, case-insensitive matching, and regex matching. Changed-file
  rustfmt reports one pre-existing import-layout difference in `window.rs`;
  the changed code follows rustfmt. No sanitizer was run.

### Increment 167 — monitor-hook append value scratch (2026-09-22)

- `cmd_set_hook_monitor_exec` now joins an existing option value and the new
  monitor value as bytes in a local `CString`. `options_set_string` borrows
  the pointer synchronously and copies the value; the temporary drops before
  later hook setup. Removed the append `xasprintf` and both later frees.
  The existing glibc `"(null)"` expansion for a null old `%s` is preserved.
- Isolated library/binary build, changed-file rustfmt, and `git diff --check`
  passed. The new private-server `hook_monitor_append_cli_checks.py` passed
  with both pre-migration and migrated binaries for initial value, ASCII
  append, byte `0xff` append, and empty reset. No sanitizer was run.

### Increment 168 — switch-mode command target scratch (2026-09-22)

- `window_switch_run_command` now owns its session or window target text in
  a local `CString`, preserving raw session-name bytes and `%u` window-index
  formatting. `cmd_template_replace` borrows the target synchronously and
  returns its separate C-owned command, which keeps its existing free.
  Removed both target `xasprintf` branches and the matching free; the owner
  also drops on command-error paths.
- Isolated library/binary build, changed-file rustfmt, Python syntax check,
  and `git diff --check` passed. The new attached-client
  `window_switch_target_cli_checks.py` passed with pre-migration and migrated
  binaries for `=alpha:` and `=alpha:7.` targets. No sanitizer was run.

### Increment 169 — menu item name owner (2026-09-22)

- `menu_add_item` now builds its rendered row name directly from the
  `format_trim_right` result, optional truncation marker, and optional key
  label as raw C-string bytes in a `CString`. It moves that owner into the
  existing `MenuRowStrings` record, removing both name `xasprintf` branches,
  their C allocation, and the immediate `menu_take_string` copy/free.
  The formatted source, trimmed source, and optional key retain their
  separate existing owners and cleanup; the row's borrowed pointer remains
  valid until menu teardown.
- The library/binary build, changed-file rustfmt, and `git diff --check`
  passed. The extended attached-client `menu_owner_cli_checks.py` passed
  with the baseline and migrated binaries for ordinary rows, separators,
  omitted rows, key labels, a truncated long row, selection, and teardown.
  No sanitizer was run.

### Increment 170 — capture-pane cell formatting scratch (2026-09-22)

- `cmd_capture_pane_cell` now owns its hyperlink URI, optional internal ID,
  and three formatted colour pieces as local `CString`s. Removed their
  `xasprintf` allocations and matching frees. The final cell line remains
  C-owned for the caller, and `utf8_stravis` data retains its existing
  C-owned allocation/free. A successful `hyperlinks_get` provides a nonnull
  URI from the stored `hyperlinks_put` entry; `CStr` preserves `%s` truncation
  at NUL. Hexadecimal colour suffixes preserve `%x` unsigned formatting.
- The library/binary build, changed-file rustfmt, and `git diff --check`
  passed. The new private-server `capture_pane_grid_cell_cli_checks.py`
  passed against the baseline and migrated binaries with exact capture
  output comparison for a red cell and OSC 8 links with and without an ID.
  No sanitizer was run.

### Increment 171 — mode-tree menu title scratch (2026-09-22)

- `mode_tree_display_menu` now owns either the centered item title or the
  empty generic-menu title in a local byte-preserving `CString`, removing
  its `xasprintf`/`xstrdup` branches and matching free. `menu_create`
  copies the title synchronously into its own `MenuOwner`; the temporary
  drops at the former free point after `menu_add_items`.
- The library/binary build, changed-file rustfmt, and `git diff --check`
  passed. The new attached-client `mode_tree_menu_title_cli_checks.py`
  passed with the baseline and migrated binaries: right-clicking a tree row
  rendered its item title, and right-clicking the preview rendered the
  empty-title generic menu. No sanitizer was run.

### Increment 172 — mode-tree preview label scratch (2026-09-22)

- `mode_tree_draw` now builds its preview-box label in a local byte-preserving
  `CString`, removing the sorted and unsorted `xasprintf` branches and their
  matching free. The label is borrowed for width checks and synchronous
  `screen_write_puts`, then dropped at the former free point before the
  preview callback. Sort order, reversed marker, optional view name, and
  glibc null `%s` rendering are preserved.
- The library/binary build, changed-file rustfmt, and `git diff --check`
  passed. The new attached-client `mode_tree_preview_label_cli_checks.py`
  passed with baseline and migrated binaries for sorted, reversed, and
  unsorted preview labels. No sanitizer was run.

### Increment 173 — customize array option name scratch (2026-09-22)

- `window_customize_build_array` now owns the composed `option[key]` name
  in a local byte-preserving `CString`, removing its `xasprintf` allocation
  and matching free. `format_add` copies the name through its formatting
  owner, and `mode_tree_add_identity` duplicates the displayed name and
  identity before the local owner drops. Other value and text allocations
  keep their existing lifetimes.
- Isolated library/binary build, `options_array_key_cli_checks.py`,
  changed-file rustfmt, and `git diff --check` passed. The new attached-client
  `customize_array_name_cli_checks.py` passed against baseline and migrated
  binaries for a filtered `status-format[7]` entry and its value. No
  sanitizer was run.

### Increment 174 — window-tree prompt label scratch (2026-09-22)

- `window_tree_key` now owns its kill-session, kill-window, kill-pane,
  tagged-kill, and command prompt labels in local `CString`s. Removed their
  `xasprintf` allocations and matching frees. `mode_tree_set_prompt` calls
  `prompt_create` synchronously, which copies the label into the prompt's
  owned string; the local owner then drops. Session names retain their raw
  C-string bytes and `%u` window indexes retain unsigned formatting.
- The library/binary build, changed-file rustfmt, and `git diff --check`
  passed. The new attached-client `window_tree_prompt_cli_checks.py` passed
  with baseline and migrated binaries for session, window, pane, tagged-kill,
  and tagged-command labels. The untagged `(current)` branch is source
  audited but not in that E2E. No sanitizer was run.

### Increment 175 — mode-tree row scratch (2026-09-22)

- `mode_tree_draw` now owns each formatted row with `CString` through both
  synchronous `format_width` and `format_draw` calls. Removed its local
  `xasprintf` allocation and matching `free`; the borrowing raw pointer is
  valid only while this row owner lives.
- Audited `%*s%s%s` byte behavior: signed width pads the C string name by
  byte length, with spaces before or after according to sign. The existing
  printf-string helper preserves glibc's `(null)` representation if a name is
  null. Tag and separator bytes are copied before constructing the CString.
- Isolated library/binary build, changed-file rustfmt, and `git diff --check`
  passed. The attached-client `mode_tree_row_cli_checks.py` compared normal
  and tagged choose-tree rows byte for byte with the pinned baseline binary.
  A larger tree's raw PTY redraw could not reliably reconstruct all rows;
  positive-width padding is source audited but lacks that E2E case. No
  sanitizer was run.

### Increment 176 — customize key-table title scratch (2026-09-22)

- `window_customize_build_keys` now builds the key-table title in a local,
  byte-preserving `CString`. Removed its `xasprintf` allocation and matching
  `free`. `mode_tree_add_identity` copies the title synchronously, so the
  temporary `as_ptr()` does not escape; the key-table name remains borrowed.
- In the isolated branch, library/binary build, serialized
  `key_bindings_storage` tests, changed-file rustfmt, and `git diff --check`
  passed. The new attached-client `customize_key_table_title_cli_checks.py`
  rendered a custom key table containing UTF-8 `é` on both the pre-change
  main binary and the migrated binary. No sanitizer was run.

### Increment 177 — capture-pane grid header and row scratch (2026-09-22)

- `cmd_capture_pane_grid` now owns its ASCII header in a Rust `String` and
  each grid-row description in a `Vec<u8>`. Removed three local `xasprintf`
  allocations and the matching header/row frees. `cmd_capture_pane_append`
  copies each temporary byte slice synchronously; its returned C-owned
  aggregate buffer and the separately allocated cell descriptions retain
  their existing ownership paths.
- Source audit preserves `%u`, `%x`, line and OSC 133 field ordering, and
  appends C string fields as bytes without UTF-8 conversion. The temporary
  pointers from `as_ptr()` remain valid through each append call.
- In the isolated branch, library/binary build, the focused
  `osc133_exit_status` test, changed-file rustfmt, and `git diff --check`
  passed. The capture-pane CLI check compared complete output byte for byte
  with the pinned baseline for two OSC 8 cases and an OSC 133 row. No
  sanitizer was run.

### Increment 178 — pane login-record scratch (2026-09-22)

- `spawn_pane` now owns the `tmux(pid).%pane` login-record label with a local
  `CString` through the synchronous `utempter_add_record` call and following
  `kill`. Removed that label's `xasprintf` allocation and matching `free`.
  The other `cp` uses in spawn (command text and shell path) retain their
  existing lifetimes; no record layout or exported ABI changed.
- Library/binary build, `remaining_spawn` tests, changed-file rustfmt, and
  `git diff --check` passed. The new private-server
  `spawn_utempter_record_cli_checks.py` interposed utempter and checked labels
  for both initial and split panes, including their pane IDs; it passed for
  the migrated and pinned baseline binaries. No sanitizer was run.

### Increment 179 — socket label base scratch (2026-09-22)

- `make_label` now owns its byte-preserving socket-directory base with a
  local `CString` through `mkdir`, `lstat`, error formatting, and returned
  path construction. Removed the base `xasprintf` allocation and both
  success/error frees. The returned socket path and error causes remain
  C-owned; `CString::as_ptr()` is borrowed only during synchronous calls.
- In the isolated branch, library/binary build, three
  `expand_path_environment` tests, changed-file rustfmt, and
  `git diff --check` passed. The new private `TMUX_TMPDIR` CLI check used
  non-UTF-8 path bytes and compared success, regular-file failure, and
  unsafe-permission failure output with the pinned baseline. A race-driven
  `lstat` failure was source audited but not E2E covered. No sanitizer was run.

### Increment 180 — customize key detail scratch (2026-09-22)

- `window_customize_build_keys` now owns Command, Note, and Repeat detail
  text with byte-preserving local `CString`s through
  `mode_tree_add_identity`. Removed three local `xasprintf` allocations, the
  no-note `xstrdup`, and all four matching frees. The tree duplicates the
  strings synchronously; the separate C-owned `cmd_list_print` result still
  follows its allocation/free path.
- In the isolated branch, library/binary build, the focused
  `list_keys_ownership` test, changed-file rustfmt, Python syntax check, and
  `git diff --check` passed. The new attached-client customize-mode CLI check
  matched all six Command, Note, and Repeat rows against the pinned baseline,
  including empty note, repeat on/off, and UTF-8 note/command text. No
  sanitizer was run.

### Increment 181 — systemd pane description scratch (2026-09-22)

- `systemd_move_to_new_cgroup` now owns its `Description` text in a local
  `CString` through `sd_bus_message_append`. Removed the local `xasprintf`
  allocation and matching `free`; the bus message contains its own encoded
  copy after append, as the former immediate free required. PID formatting
  and the surrounding bus message/cause ownership are unchanged.
- Library/binary build, `remaining_spawn` tests, changed-file rustfmt, and
  `git diff --check` passed. The new private-server
  `systemd_pane_description_cli_checks.py` found the live pane scope in the
  user systemd manager and checked its exact child/parent PID description for
  both migrated and pinned baseline binaries. This E2E requires a running
  user systemd manager. No sanitizer was run.

### Increment 182 — editor command scratch (2026-09-22)

- `spawn_editor` now owns its byte-preserving editor command with a local
  `CString` through the synchronous `spawn_pane` call. Removed the local
  `xasprintf` allocation and matching `free`. `spawn_pane` copies `sc.argv`
  with `cmd_copy_argv` before forking, so the temporary raw pointer and its
  stack slot do not escape. The pane owns its copied command afterward.
- In the isolated branch, library/binary build, two focused spawn tests,
  changed-file rustfmt, Python syntax check, and `git diff --check` passed.
  The new attached-client choose-buffer editor check used a UTF-8 editor
  filename and flag, confirmed the original temporary file contents and
  updated buffer, and matched the pinned baseline output. No sanitizer was
  run.

### Increment 183 — customize environment entry scratch (2026-09-22)

- `window_customize_build_environment` now borrows each set or empty
  environment value as `&CStr` through the synchronous `format_add` copy.
  It uses `Cow<CStr>` for the displayed name: borrowed for set entries and
  owned `-`-prefixed text for removed entries, copied synchronously by
  `mode_tree_add_identity`. Removed the value `xstrdup`/free pair on both
  normal and filter-rejected paths and the name `xasprintf`/`xstrdup`/free
  pairs. Null removed values still display empty in formats and keep their
  leading `-` in the tree; the separately expanded row text remains C-owned.
- In the isolated branch, library/binary build, environment,
  model_environment, and format_entry_owner tests, changed-file rustfmt,
  Python syntax check, and `git diff --check` passed. The new attached-client
  CLI check compared set UTF-8, removed, and empty rows with the pinned
  baseline and verified filter rejection. No sanitizer was run.

### Increment 184 — systemd UUID scope-name scratch (2026-09-22)

- `systemd_move_to_new_cgroup` now owns its UUID-derived scope name in a
  local `CString` through the synchronous `sd_bus_message_append` call.
  Removed the scope-name `xasprintf` allocation and matching `free`. The
  byte formatter preserves the original 16-byte UUID order, lowercase
  two-digit hex for each byte, 8-4-4-4-12 grouping, and `.scope` suffix.
  The bus message and error causes keep their existing ownership paths.
- Library/binary build, `remaining_spawn` tests, changed-file rustfmt, and
  `git diff --check` passed. The private-server systemd pane CLI check now
  verifies the live unit name's exact UUID-shaped byte layout as well as
  its description, and passed for migrated and pinned baseline binaries.
  A running user systemd manager is required; no sanitizer was run.

### Increment 185 — customize new-key prompt scratch (2026-09-22)

- `window_customize_add_key` now owns the `New key in ...` prompt with a
  byte-preserving local `CString` through `mode_tree_set_prompt`. Removed its
  `xasprintf` allocation and matching `free`. `prompt_create` copies the
  prompt synchronously, so the raw pointer does not escape the call.
- In the isolated branch, library/binary build, the focused
  `key_bindings_storage` test, changed-file rustfmt, and `git diff --check`
  passed. The new attached-client customize-mode CLI check displayed a prompt
  for a key table containing UTF-8 `é`, added an F11 binding, and matched
  `list-keys` output with the pinned baseline. No sanitizer was run.

### Increment 186 — customize environment prompt scratch (2026-09-22)

- `window_customize_set_environment` now owns its composed prompt with a
  byte-preserving local `CString` through `mode_tree_set_prompt`. Removed
  its `xasprintf` allocation and matching `free`; the separate C-owned
  `window_customize_scope_text` result is still freed after its bytes are
  copied. `prompt_create` copies the prompt synchronously.
- In the isolated branch, library/binary build, three focused
  window_customize tests, changed-file rustfmt, and `git diff --check`
  passed. The new attached-client CLI check matched session and global
  prompts with the pinned baseline, including UTF-8 session/name text and
  an empty global value. No sanitizer was run.

### Increment 187 — customize new-user prompt scratch (2026-09-22)

- `window_customize_add_option` now borrows one of two static `CStr` prompt
  labels, `New user option: ` or `New user hook: `, through
  `mode_tree_set_prompt`. Removed the local `xasprintf` allocation and
  matching `free`; `prompt_create` copies either label synchronously. The
  option/hook decision and default `@` input are unchanged.
- Library/binary build, focused window_customize tests, changed-file
  rustfmt, and `git diff --check` passed. The new private attached-client
  customize-mode CLI check displayed both prompts and matched the pinned
  baseline binary. No sanitizer was run.

### Increment 188 — customize array-key prompt scratch (2026-09-22)

- `window_customize_set_array_key` now owns its name/key prompt with a
  byte-preserving local `CString` through `mode_tree_set_prompt`. Removed
  the `xasprintf` allocation and matching `free`; `prompt_create` copies
  the text synchronously. The array-key input and callback ownership are
  unchanged.
- In the isolated branch, library/binary build, five focused
  `options_storage` tests, changed-file rustfmt, and `git diff --check`
  passed. The new attached-client CLI check displayed the `status-format[7]`
  prompt, renamed its key to 8, and matched the pinned baseline output.
  No sanitizer was run.

### Increment 189 — customize set-option prompt scratch (2026-09-22)

- `window_customize_set_option` now builds the scalar, array-add, and
  array-entry prompt labels in one local, byte-preserving `CString`.
  Removed the three `xasprintf` branches and matching `free`;
  `mode_tree_set_prompt` copies the prompt synchronously. The separate
  C-owned scope text and option value keep their existing lifetimes.
- In the isolated branch, library/binary build, six focused option tests,
  changed-file rustfmt, and `git diff --check` passed. The new private
  attached-client CLI check displayed all three prompt variants and matched
  the pinned baseline, including UTF-8 session/value text. No sanitizer was
  run.

### Increment 190 — customize key-binding prompt scratch (2026-09-22)

- `window_customize_set_key` now uses a local `CString` for both the Command
  and Note prompt labels, built from the byte-preserving `key_string_format`
  result. Removed both `xasprintf` allocations and matching frees;
  `mode_tree_set_prompt` copies each prompt synchronously. The separate
  C-owned command value keeps its existing lifetime.
- Library/binary build, three focused customize tests, one key-binding
  storage test, changed-file rustfmt, Python syntax check, and
  `git diff --check` passed. The new private attached-client CLI check
  displayed both prompt variants and matched the pinned baseline, including
  UTF-8 command and note values. No sanitizer was run.

### Increment 191 — customize unset confirmation prompt scratch (2026-09-22)

- `window_customize_key` now builds the current scalar/array-entry and tagged
  unset confirmation labels in local `CString` owners. Removed the three
  `xasprintf` allocations and matching frees; `mode_tree_set_prompt` copies
  the prompt synchronously. Option names and array keys retain their bytes.
- In the isolated branch, library/binary build, six focused option tests,
  changed-file rustfmt, and `git diff --check` passed. The new attached-client
  CLI check matched the scalar, array-entry, and tagged prompts with the
  pinned baseline and confirmed cancellation preserves option values.
  No sanitizer was run.

### Increment 192 — customize reset confirmation prompt scratch (2026-09-22)

- `window_customize_key` now builds current-item and tagged reset
  confirmation labels in local `CString` owners. Removed both `xasprintf`
  allocations, matching frees, and the now-unused function-level prompt
  pointer; `mode_tree_set_prompt` copies each prompt synchronously.
- In the isolated branch, library/binary build, five focused option tests,
  changed-file rustfmt, Python syntax check, and `git diff --check` passed.
  The new attached-client CLI check matched both prompt variants with the
  pinned baseline and confirmed accepting each removes the user option.
  No sanitizer was run.

### Increment 193 — customize scope-text owner (2026-09-22)

- `window_customize_scope_text` now returns a byte-preserving `CString` for
  pane, session, window, and empty scopes. Its four callers borrow the local
  owner for `format_add` or copy its bytes into a prompt, then drop it at the
  former free point. Removed three `xasprintf` allocations, the empty-scope
  `xstrdup`, and all four matching frees. The helper is an internal Rust
  function; its foreign calling convention was unnecessary.
- Library/binary build, six focused option tests, changed-file rustfmt,
  Python syntax check, and `git diff --check` passed. A new attached-client
  CLI check matched server, UTF-8 session, window, pane, and session
  environment scope labels with the pinned baseline. Existing environment
  and set-option prompt checks also matched the baseline. No sanitizer was
  run.

### Increment 194 — window-tree target owner (2026-09-22)

- `window_tree_get_target` now returns `Option<CString>`, preserving the
  absent-target branch and exact session-name bytes and numeric target
  syntax. Both callers borrow the target only during synchronous
  `mode_tree_run_command` calls. Removed three `xasprintf` branches and both
  matching frees; the internal helper no longer needs a C calling
  convention.
- In the isolated branch, library/binary build, two focused mode-tree/window
  tests, changed-file rustfmt, Python syntax check, and `git diff --check`
  passed. The new attached-client CLI check selected session, window, and
  pane targets through both Enter and `:` and matched exact target bytes
  with the pinned baseline. No sanitizer was run.

### Increment 195 — startup path expansion owner (2026-09-22)

- `expand_path` now returns `Option<CString>` for tilde, environment, and
  literal paths. Its sole `expand_paths` caller moves the owner directly
  when realpath is skipped, or borrows its pointer for realpath/logging and
  drops it before duplicate filtering. Removed two `xasprintf` allocations,
  the literal-path `xstrdup`, and the matching free. Missing home/variable
  remains absent; a cleared environment value retains glibc `%s`'s `(null)`
  rendering.
- In the isolated branch, library/binary build, three focused path tests,
  changed-file rustfmt, Python syntax check, and `git diff --check` passed.
  The new private CLI check matched non-UTF-8 home/config/socket paths,
  symlink canonicalization, absent XDG configuration, and `/tmp` fallback
  with the pinned baseline. The existing socket-label check also matched.
  No sanitizer was run.

### Increment 196 — client lockfile owner across daemon fork (2026-09-22)

- `client_connect` now owns its byte-preserving lockfile name as
  `Option<CString>`. A private `server_start_owned` path borrows that owner
  across fork, so the parent drops its process-local copy after return and
  the child takes and drops its copy after unlink. Removed the local
  `xasprintf` allocation and four manual frees. The exported `server_start`
  C signature and its raw-pointer libc-free contract remain for compatibility;
  systemd's null lockfile path is unchanged.
- In the isolated branch, library/binary build, one focused platform-socket
  test, changed-file rustfmt, Python syntax check, and `git diff --check`
  passed. The new private-socket CLI check matched normal daemon startup,
  lock contention/retry, unavailable lockfile, and non-UTF-8 path bytes with
  the pinned baseline. No sanitizer was run.

### Increment 197 — hook monitor string owner (2026-09-22)

- `hooks_monitor_to_cstring` now builds all five monitor target forms in a
  byte-preserving `Option<CString>`. Three in-repo callers borrow the result
  synchronously for `format_add` or drawing and no longer free raw strings.
  Removed five `xasprintf` branches and their caller frees. The exported
  `hooks_monitor_to_string` C wrapper still returns a libc-freeable duplicate
  and preserves its null result.
- In the isolated branch, library/binary build, six focused tests,
  changed-file rustfmt, Python syntax check, and `git diff --check` passed.
  The new private-server CLI check matched all five monitor target forms,
  non-UTF-8 format bytes, and attached customize detail with the pinned
  baseline. No sanitizer was run.

### Increment 198 — shell argv0 owner for Rust callers (2026-09-23)

- `shell_argv0_cstring` now builds the basename and optional login prefix
  byte-preservingly. `job_run` owns it through its fork: the parent drops it
  at the former success/failure frees, while the child passes a borrowed
  pointer to `execl` before exec/exit. `client_exec` likewise borrows it
  through `execl`. Removed both `xasprintf` branches and both `job_run`
  frees. The exported `shell_argv0` C wrapper still returns a libc-freeable
  duplicate.
- Library/binary build, three focused job/client/shell tests, changed-file
  rustfmt, Python syntax check, and `git diff --check` passed. The new
  private-server CLI check matched `run-shell`'s job argv0 and client `-c`
  with and without login mode, using a UTF-8 shell basename, against the
  pinned baseline. No sanitizer was run.

### Increment 199 — command-completion scratch owner (2026-09-23)

- `prompt_complete_prefix` now returns a `CString` for the bytewise common
  prefix. `prompt_complete` owns either that prefix or the single-match name
  plus space and returns `Option<CString>` to its sole caller.
  `prompt_replace_complete` borrows the pointer only while replacing prompt
  text; the owner drops on return. Removed the completion `xasprintf`, the
  prefix `xstrdup` and in-place NUL write, and both completion frees.
- Library/binary build, `model_prompt` and `remaining_prompt` tests, and
  attached-client prompt-completion CLI checks passed on candidate and pinned
  baseline. The source is byte-preserving, including non-UTF-8 command alias
  bytes. `git diff --check` passed. No sanitizer was run.

### Increment 200 — capture-pane cell scratch owner (2026-09-23)

- The private `cmd_capture_pane_cell` now returns a `CString`; its sole grid
  caller borrows the bytes through `cmd_capture_pane_append`. The escaped cell
  data uses a `Vec<u8>` sized to the former `utf8_stravis` maximum and filled
  by `utf8_strvis`. Removed the cell `xasprintf`, the escaped-data allocation
  and free, and the caller's line free. Existing color and hyperlink string
  owners remain local.
- Library/binary build, `remaining_command`, `subjects_grid`, and
  `format_grid_line` tests, changed-file rustfmt, Python syntax check, and
  `git diff --check` passed. The expanded private-server grid-cell CLI check
  matched the pinned baseline byte for byte, including UTF-8, escaped
  backslash, and underline cells. No sanitizer was run.

### Increment 201 — socket-label error owner (2026-09-23)

- Private `make_label` now returns `Result<*mut c_char, CString>`: all five
  error causes are byte-preserving Rust owners. Its sole CLI caller prints a
  borrowed cause pointer and drops the owner before exit. Removed five error
  `xasprintf` allocations and the caller's cause free. The successful socket
  path remains C-owned because `socket_path` is global and shared with the
  client/server startup contract.
- Library/binary build, `platform_socket` and `expand_path_environment`
  tests, changed-file rustfmt, Python syntax check, and `git diff --check`
  passed. The private socket-label CLI check matched the pinned baseline,
  including mkdir permission failure, non-directory, unsafe permissions, and
  success. No sanitizer was run.

### Increment 202 — option value rendering scratch owner (2026-09-23)

- Private `options_value_to_cstring` now owns numeric, flag, key, colour,
  choice, string, and command-list renderings. `options_to_cstring` joins
  array items in one `Vec<u8>` and handles indexed and empty values. This
  removed repeated array `xasprintf` and intermediate frees; the sole
  `show-options` caller of the private owner no longer duplicates and frees
  a C result. Exported `options_to_string` still returns a libc-freeable
  duplicate. `cmd_list_print` remains C-owned for its other callers, so its
  option rendering is copied and freed at that producer boundary.
- Library/binary build, five `options_storage` tests, changed-file rustfmt,
  Python syntax check, and `git diff --check` passed. A private-server CLI
  check matched the pinned baseline for empty and non-UTF-8 arrays, indexed
  values, scalar kinds, and a command-list hook. No sanitizer was run.

### Increment 203 — JSON tokenizer token owner (2026-09-23)

- `json_tokenize_input` now grows a `Vec<json_token>` through tokenization,
  and `json_parse` owns it through recursive parsing. Removed the private
  `json_tokens` raw size/capacity/pointer record, its malloc/realloc/free,
  and explicit success/error token teardown. Tokenization borrows the prior
  token only before an append; parser cursors borrow stable storage after all
  appends. Exported `json_parse`, node, and C-owned error contracts remain.
- Library/binary build, `json_scratch` and `remaining_json` tests, and
  changed-file rustfmt and `git diff --check` passed. A new test crosses the
  former 1,024-token capacity on success and parse error. The layout CLI
  check passed on candidate and pinned baseline. No sanitizer was run.

### Increment 204 — argument escaping scratch owner (2026-09-23)

- Private `args_escape_cstring` now owns quote selection, UTF-8 escaping,
  and byte-preserving assembly. `args_print_add_value` borrows it only through
  its synchronous append; its string branch no longer allocates and frees a
  C result. Removed the escape helper's `xasprintf` branches, temporary
  `utf8_stravis` allocation/free, and the print caller's escaped-string free.
  Exported `args_escape` still returns a libc-freeable `xstrdup` duplicate
  for its other callers. The command-list branch still frees the C-owned
  `cmd_list_print` result.
- Library/binary build, eight `arguments_conversion` tests, changed-file
  rustfmt, Python syntax check, and `git diff --check` passed. A private
  server `bind-key`/`list-keys` CLI check matched the pinned baseline for ten
  cases including empty, quotes, tilde, newline, UTF-8, and non-UTF-8 bytes.
  No sanitizer was run.

### Increment 205 — set-buffer scratch name owner (2026-09-23)

- `cmd_set_buffer_exec` now keeps its optional temporary buffer name as
  `Option<CString>` across delete, rename, and set branches. The automatic
  top-buffer paths call `paste_get_top` without requesting a C-owned name,
  then clone `paste_buffer_name` before callbacks can delete or rename its
  record. Removed the local `xstrdup`, the top-name allocation, and all five
  local name frees. `paste_buffer.name` and the public paste API stay C-owned.
- Library/binary build, six `paste::tests`, changed-file rustfmt, Python
  syntax check, and `git diff --check` passed. A private-server CLI check
  matched the pinned baseline across named/automatic buffer set, append,
  rename, delete, and error paths, including UTF-8 and invalid non-UTF-8
  names. No sanitizer was run.

### Increment 206 — command-list print buffer owner (2026-09-23)

- Private `cmd_list_print_cstring` now assembles output in a `Vec<u8>` and
  returns `CString`, eliminating its raw buffer length/realloc lifecycle.
  `args_print_add_value`, `args_to_vector`, and both `cmd_list_copy` debug
  paths borrow that owner synchronously, removing their returned-string
  frees. Exported `cmd_list_print` still returns a libc-freeable duplicate.
  `cmd_print` remains a C-owned producer and is copied/freed within the
  helper; `args_value_as_string.cached` remains C-owned in its C-layout
  record.
- Library/binary build, `cmd_list_print_owner` and `arguments_conversion`
  tests, changed-file rustfmt, Python syntax check, and `git diff --check`
  passed. The focused test covers empty lists, grouped separators, and both
  print flags. A private-server command/hook CLI check matched the pinned
  baseline, including nested commands and non-UTF-8 bytes. No sanitizer was
  run.

### Increment 207 — format quote modifier scratch owner (2026-09-23)

- `format_quote_shell` and `format_quote_style` now construct byte-preserving
  `CString` values. `format_find` retains an `Option<CString>` across its
  shell, single-quote, and style-quote modifier subchain, replacing three
  intermediate manual frees with Rust drops. The original C-owned `found`
  string is freed once after the chain, and the returned `found` remains
  libc-freeable for its callers. Removed both helper xmalloc buffers and
  their pointer writes.
- Library/binary build, `format_modifier_copy` and
  `format_condition_scratch` tests, changed-file rustfmt, Python syntax
  check, and `git diff --check` passed. A private-server CLI check matched
  the pinned baseline for empty, non-UTF-8, newline, and combined quote
  modifiers. No sanitizer was run.

### Increment 208 — printed command string owner (2026-09-23)

- Private `cmd_print_cstring` now assembles command name and printed argument
  bytes in a `CString`, replacing its `xasprintf`/`xstrdup` branches. It
  consumes and frees the still C-owned `args_print` result locally.
  `cmd_list_print_cstring` and both synchronous command-queue logging paths
  borrow this owner, removing their `cmd_print` result frees. Exported
  `cmd_print` keeps a libc-freeable duplicate for C callers.
- Library/binary build, two `cmd_list_print_owner` focused tests,
  changed-file rustfmt, Python syntax check, and `git diff --check` passed.
  A private-server command-log CLI check matched the pinned baseline for
  empty, non-UTF-8, and nested command arguments. No sanitizer was run.

### Increment 209 — terminal override scratch value owner (2026-09-23)

- `tty_term_apply` now keeps each loop-local decoded override as
  `Option<CString>`; removals remain absent. `tty_term_override_value`
  preserves invalid-escape fallback and the old first-NUL C-string view
  when `strunvis` decodes an escaped NUL. Removed the initial/fallback
  `xstrdup` and the loop-local frees. Stored terminal capabilities still use
  their existing C-layout ownership and duplicate the temporary value.
- Library/binary build, `tty_parse_features`, `model_terminal`, and
  `model_tty` tests, changed-file rustfmt, Python syntax check, and
  `git diff --check` passed. An attached-terminal PTY check matched the
  pinned baseline byte for byte across seven capability rows, including
  non-UTF-8, invalid escape, escaped NUL, empty, numeric, flag, and removal
  cases. No sanitizer was run.

### Increment 210 — parsed window-copy text owner (2026-09-23)

- `window_copy_vadd` now holds its parse-branch formatted text in a local
  `CString` through `input_parse_screen`, removing its `vasprintf` allocation
  and matching `free`. `xvasprintf_cstring` preserves arbitrary bytes through
  the first NUL, matching the old `strlen(text)` length. The exported
  variadic `window_copy_add`/`window_copy_vadd` signatures remain unchanged.
- Library/binary build, `run_shell_partial_line` test, changed-file rustfmt,
  Python syntax check, and `git diff --check` passed. The private-server
  view-mode CLI check matched the pinned baseline for ANSI, UTF-8, invalid
  bytes, and an exact interior-NUL scenario: load `A\0B` with `load-buffer`,
  invoke `show-buffer` from an attached client, then copy the view-mode
  line. The loaded buffer has all three bytes; the copied line is `A\n` on
  both binaries because `server_client_print` passes `%.*s` into
  `window_copy_vadd` and the first-NUL view is parsed. No sanitizer was run.

### Increment 211 — printed arguments owner (2026-09-23)

- Private `args_print_cstring` now assembles option flags and values in a
  `Vec<u8>` and returns a `CString`. `args_print_add` appends the first-NUL
  C-string view of each variadic fragment. This removes the printer's
  `xcalloc`/`xrealloc`, capacity count, and raw append pointer. The in-repo
  `cmd_print_cstring` and `cmdq_insert_hook` callers borrow the owner and no
  longer free a temporary; exported `args_print` still returns a libc-freeable
  duplicate. No supported input to `args_print_add` supplies a middle NUL:
  `%c` receives nonzero option flags, and `%s` receives terminated strings.
- Ten focused argument/command-list tests, library/binary build, changed-file
  rustfmt, Python syntax, and diff checks passed. Private-server CLI checks
  matched the pinned baseline for command printing, argument escaping, and
  `#{hook_arguments}` with empty, non-UTF-8, and plain values. No sanitizer was
  run.

### Increment 212 — key-binding log scratch owner (2026-09-23)

- `key_bindings_add` now borrows the existing `cmd_list_print_cstring` owner
  through its synchronous `log_debug` call. Removed the local C-owned
  `cmd_list_print` result and matching `free`; key binding record ownership,
  printed bytes, and exported signatures remain unchanged.
- Binary build, changed-file rustfmt, and diff checks passed. The
  private-server `scripts/key_cli_checks.py` passed with both the pinned
  baseline and candidate binaries. No sanitizer was run.

### Increment 213 — customize key scratch owners (2026-09-23)

- `window_customize_key_is_changed`, `window_customize_build_keys`, and
  `window_customize_draw_key` now use the existing
  `cmd_list_print_cstring` owner for synchronous comparison, mode-tree detail,
  and screen writing. Removed their C-owned print results and frees, including
  early-return cleanup. Byte comparisons match the former `strcmp` results;
  `%s` callers borrow live owners. Delayed edit prompts still receive C-owned
  strings, and exported signatures are unchanged.
- Binary build, changed-file rustfmt, and diff checks passed. The attached
  client `scripts/customize_key_detail_cli_checks.py` matched the pinned
  baseline byte for byte. No sanitizer was run.
- Combined validation after increments 211–213: `RUST_TEST_THREADS=1 cargo
  test --workspace --quiet`, library/binary build, changed-file rustfmt,
  Python AST check, and commit diff checks passed. Six private-server CLI
  checks passed against the pinned baseline, covering argument and command
  printing, argument escaping, key bindings, customize key detail, and the
  attached `show-buffer` middle-NUL view-mode path. No sanitizer was run.

### Increment 214 — hook debug print scratch owner (2026-09-23)

- `hooks_insert_one` now holds the existing `cmd_list_print_cstring` owner
  through its synchronous debug log. Removed the C-owned print result and
  matching free; hook queueing and exported signatures are unchanged.
- Binary build, changed-file rustfmt, Python syntax, and diff checks passed.
  `scripts/hooks_insert_print_owner_cli_checks.py` triggered an actual
  `after-new-window` hook under verbose logging and matched the rendered
  `hooks_insert_one` log bytes against the pinned baseline. No sanitizer was
  run.

### Increment 215 — parser command-print scratch owners (2026-09-23)

- `cmd_parse_print_commands`, the command-list branch of
  `cmd_parse_log_commands`, and `cmd_parse_build_commands` now hold the
  existing `cmd_list_print_cstring` owner through their synchronous
  `cmdq_print` or `log_debug` calls. Removed three C-owned results and their
  frees; parser output bytes and exported signatures remain unchanged.
- Library/binary build, changed-file rustfmt, and diff checks passed. A new
  private-server `scripts/cmd_parse_print_owner_cli_checks.py` exercises
  `source-file -v` with grouped and nested commands, and checks the parsed
  command argument and final command list debug log lines under `-vv`.
  Candidate output matched the pinned baseline. No sanitizer was run.

### Increment 216 — command-option print owner (2026-09-23)

- The command branch of `options_value_to_cstring` now returns the existing
  `cmd_list_print_cstring` result directly. Removed the intermediate C-owned
  result, its byte copy into another `CString`, and its manual free.
  `options_to_string` remains libc-freeable at its exported boundary; printed
  command bytes and record ownership remain unchanged.
- Workspace tests, library/binary build, changed-file rustfmt, and diff
  checks passed in the isolated worktree. The private-server
  `scripts/options_value_owner_cli_checks.py` matched the pinned baseline,
  including command-option rendering. No sanitizer was run.
- Combined validation after increments 214–216: `RUST_TEST_THREADS=1 cargo
  test --workspace --quiet`, library/binary build, changed-file rustfmt,
  Python AST checks, and commit diff checks passed. The hook debug,
  parser verbose/debug, and command-option CLI checks all matched the
  pinned baseline. No sanitizer was run.

### Increment 217 — list-keys command text owner (2026-09-23)

- `cmd_list_keys_format_add_key_binding` now holds the existing
  `cmd_list_print_cstring` result through its synchronous `format_add("%s")`
  call. Removed its C-owned print result and matching free. The escaped and
  no-groups flags, printed bytes, and exported signatures remain unchanged.
- Library/binary build, changed-file rustfmt, and diff checks passed.
  `scripts/cmd_list_print_owner_cli_checks.py` and
  `scripts/arguments_escape_owner_cli_checks.py` matched the pinned baseline
  for escaped separators, nested commands, and non-UTF-8 key-command text.
  No sanitizer was run.

### Increment 218 — command-prompt name scratch owner (2026-09-23)

- Private `args_make_commands_get_command_cstring` now owns the first parsed
  command name or the raw command prefix. `cmd_command_prompt_exec` borrows it
  while building the default prompt, removing its C-owned temporary and free.
  The exported `args_make_commands_get_command` remains libc-freeable. The
  prefix keeps the original `strcspn(" ,")` split, `c_int` printf precision
  behavior, arbitrary bytes, and first-NUL view.
- Workspace tests, library/binary build, changed-file rustfmt, Python syntax,
  and diff checks passed. An attached-client
  `scripts/command_prompt_name_owner_cli_checks.py` matched the pinned
  baseline for raw-string and parsed command-list prompt labels and effects.
  No sanitizer was run.

### Increment 219 — detach exec message byte owner (2026-09-23)

- `server_client_exec` now assembles the two NUL-terminated command and shell
  strings in a `Vec<u8>` and borrows its bytes for `proc_send`. Removed the
  local `xmalloc`, two `memcpy` operations, manual lengths, and `free`.
  `proc_send` calls `imsg_compose`, whose `ibuf_add` copies the payload before
  returning, so the vector drops after the complete wire message is queued.
  Exported signatures and message bytes remain unchanged.
- Binary build, two focused `server_client` tests, changed-file rustfmt, and
  diff checks passed. The attached-client
  `scripts/server_client_exec_owner_cli_checks.py` drove `detach-client -E`
  with a non-UTF-8 command byte and verified the command and shell results
  against the pinned baseline. No sanitizer was run.
- Combined validation after increments 217–219: `RUST_TEST_THREADS=1 cargo
  test --workspace --quiet`, library/binary build, changed-file rustfmt,
  Python AST checks, and commit diff checks passed. The command-list,
  argument-escape, attached prompt, and attached exec CLI checks matched
  the pinned baseline. No sanitizer was run.

### Increment 220 — server exit message payload owner (2026-09-23)

- `server_client_check_exit` now builds its `MSG_EXIT` payload in a `Vec<u8>`:
  native `c_int` return-code bytes followed by the optional exit message
  including its NUL. Removed the local `xmalloc`, `memcpy`, length bookkeeping,
  and `free`. `proc_send` copies the complete payload through `imsg_compose`
  and `ibuf_add` before the vector drops; message type, wire bytes, and
  exported signatures remain unchanged.
- Library/binary build, Python syntax, and diff checks passed. The new
  `scripts/server_exit_payload_cli_checks.py` matched the pinned baseline for
  a normal no-message exit, an attached-client exit, and a socket startup
  failure with an error message and return code 1. No sanitizer was run.

### Increment 221 — client command packet byte owner (2026-09-23)

- `client_main` now owns the outgoing `MSG_COMMAND` packet in a `Vec<u8>`.
  A compile-time size assertion confirms the one-field `msg_command` header
  matches native `c_int` bytes; `cmd_pack_argv` writes after that header.
  Removed the local `xmalloc` and all success/error frees. `proc_send` copies
  the complete packet synchronously, preserving argv bytes, native header,
  packet length, and exported signatures.
- Workspace tests, binary build, Python syntax, and diff checks passed in the
  isolated worktree. `scripts/client_command_payload_cli_checks.py` matched
  the pinned baseline for normal, non-UTF-8, zero-argument, large successful,
  failed-send, and too-long command paths. No sanitizer was run.

### Increment 222 — format modifier unescape scratch owner (2026-09-23)

- Private `format_unescape_cstring` now builds escaped modifier argument bytes
  in a `Vec<u8>` and returns a `CString`. Both modifier-parser call sites
  borrow that owner through `format_expand_modifier_arg`, removing their
  C-owned result frees and the producer's raw allocation and pointer writes.
  `format_unescape` keeps a C-owned duplicate for literal format output.
  The owned result preserves high bytes, first-NUL C-string viewing, and the
  empty result on a format time limit.
- Workspace tests, library/binary build, changed-file rustfmt, Python syntax,
  and diff checks passed in the isolated worktree. The new
  `scripts/format_unescape_owner_cli_checks.py` matched the pinned baseline
  for escaped commas, chained modifiers, non-UTF-8 bytes, and literal output.
  No sanitizer was run.
- Combined validation after increments 220–222: `RUST_TEST_THREADS=1 cargo
  test --workspace --quiet`, library/binary build, changed-file rustfmt,
  Python AST checks, and commit diff checks passed. The exit-payload,
  client-command-packet, and modifier-unescape CLI checks matched the pinned
  baseline. No sanitizer was run.

### Increment 223 — stripped time-format owner (2026-09-23)

- Private `format_strip_cstring` now assembles stripped time-format bytes in
  a `Vec<u8>` and returns a `CString`. `format_replace` keeps
  `Option<CString>` through both `format_find` paths and replaces an earlier
  time format by assignment, removing the raw allocation, pointer writes,
  and three manual frees. The old `strchr` lookahead treats a trailing NUL
  as a match, so a trailing `#` is still stripped outside brackets.
- Workspace tests, binary build, changed-file rustfmt, Python syntax, and
  diff checks passed. `scripts/format_strip_owner_cli_checks.py` matched the
  pinned baseline for escaped and trailing `#`, nested formats, non-UTF-8
  bytes, repeated `t/f` modifiers, and conditional lookup. No sanitizer was
  run.

### Increment 224 — argv stringification owner (2026-09-23)

- Private `cmd_stringify_argv_cstring` now joins escaped argv bytes in a
  `Vec<u8>` and returns `Option<CString>`, preserving the legacy null result
  for negative argc. This removes the producer's `xrealloc`, `strlcat`,
  length bookkeeping, and per-argument C result frees. Synchronous callers
  in spawn events/logging, default window naming, and current-command
  formatting borrow the owned result. The exported `cmd_stringify_argv`
  remains libc-freeable for retained and callback results.
- Workspace tests, library/binary build, and diff checks passed in the
  isolated worktree. `scripts/cmd_stringify_owner_cli_checks.py` matched the
  pinned baseline for empty argc, pane-created events, apostrophe/empty/
  non-UTF-8 arguments, pane start commands, dead-pane current-command
  fallback, and window naming. No sanitizer was run.

### Increment 225 — config-file format assembly owner (2026-09-23)

- `format_cb_config_files` now joins `cfg_files()` bytes in a local `Vec<u8>`
  and returns one NUL-terminated C allocation through `xmemdup`, preserving
  the callback's libc-freeable contract. Removed its `xrealloc` growth,
  manual length, and pointer writes. Index-based separators preserve empty
  path entries at the first, middle, and last positions and non-UTF-8 bytes.
- Library/binary build, changed-file rustfmt, and diff checks passed. The
  expanded `scripts/config_paths_cli_checks.py` matched the pinned baseline
  for ordinary, non-UTF-8, and mixed empty config paths. No sanitizer was
  run.
- Combined validation after increments 223–225: `RUST_TEST_THREADS=1 cargo
  test --workspace --quiet`, library/binary build, changed-file rustfmt,
  Python AST checks, and commit diff checks passed. The stripped time-format,
  argv-stringification, and config-path CLI checks matched the pinned
  baseline. No sanitizer was run.

### Increment 226 — format expansion output owner (2026-09-23)

- `format_expand1` and its private `format_replace` helper now assemble all
  literal, style, modifier, and nested expansion output in one `Vec<u8>`.
  The exported result still makes one NUL-terminated C allocation for callers
  that free it with libc. This removes the raw capacity, offset, and
  `xreallocarray` growth from the complete output path.
- Workspace tests, library/binary build, changed-file rustfmt, and diff checks
  passed. `scripts/format_expand_output_owner_cli_checks.py` matched the pinned
  baseline for long literals, nested and quoted expressions, style, non-UTF-8
  bytes, and a trailing `#`; format job and loop CLI checks also passed. No
  sanitizer was run.

### Increment 227 — client file path record owner (2026-09-23)

- Both file-record constructors now allocate a containing `FileOwner` with a
  `client_file` at offset zero. Its `Option<CString>` owns every assigned path;
  the public `client_file.path` remains a borrowed C pointer until the final
  `file_free`, after callbacks and references have finished. `file_get_path`
  builds the path from byte slices, preserving `~/`, absolute and relative
  paths, and non-UTF-8 bytes.
- Workspace tests, library/binary build, changed-file rustfmt, and diff checks
  passed. `scripts/file_path_owner_cli_checks.py` matched the pinned baseline
  for read/write callbacks, missing-path errors, `-` streams, binary contents,
  and non-UTF-8 paths. No sanitizer was run.

### Correction — argument cache owner retracted (2026-09-23)

- Increment 228 used `HashMap<usize, CString>` keyed by the cached character
  pointer. This violated the agreed rule against pointer-address collection
  keys, so its code, test, and CLI script were reverted. Passing tests did not
  make that ownership model acceptable. The command-list cache remains C-owned
  until its movable `args_value` record can carry a genuine owner without
  changing the required ABI or copy semantics.
- The combined workspace tests, library/binary build, changed-file rustfmt,
  Python AST checks, and the format expansion, file path, and embedded-NUL
  copy-view baseline checks passed for increments 226–227 before this
  correction. Revalidate the corrected main tree with the next integrations.

### Increment 229 — hyperlink external ID record owner (2026-09-23)

- `hyperlinks_put` now allocates a stable `HyperlinkUriOwner` containing the
  public C-layout record at offset zero and the generated external ID in a
  `CString`. `hyperlinks_uri.external_id` borrows that string until
  `hyperlinks_remove` unlinks and drops the owner. This removes the
  `xasprintf`/`free` pair for the ID and the `xcalloc`/`free` pair for the
  record while preserving the `tmux%llX` bytes.
- Focused hyperlink storage and real-pane OSC 8 tests, changed-file rustfmt,
  diff checks, and the capture-pane baseline CLI check passed. No sanitizer
  was run.

### Increment 230 — format job record and retained string owner (2026-09-23)

- The format-job cache now stores stable `FormatJobOwner` allocations with
  the public `format_job` record at offset zero. Its command, expanded
  command, and output pointers borrow `CString` fields. All update,
  completion, timeout, and process-start failure paths replace those owned
  values; cache tidy removes the entry, frees any active process job, then
  drops the containing owner. The callback address and C-layout record remain
  stable.
- Four focused format-job tests, library/binary build, changed-file rustfmt,
  and diff checks passed. The live attached-client job-output replacement CLI
  check and existing job-name CLI check passed against both the migrated and
  pinned baseline binaries. No sanitizer was run.

### Increment 231 — customize change option-name owner (2026-09-23)

- `window_customize_change_each` and
  `window_customize_change_current_callback` now copy option names into local
  `Option<CString>` values before destructive unset/reset operations. The
  copies stay alive through the synchronous `options_push_changes` call and
  then drop automatically. This removes both `xstrdup`/`free` pairs without
  changing record layout or callback ABI.
- Workspace tests, binary build, changed-file rustfmt, and diff checks passed.
  The customize reset and unset prompt CLI checks matched the pinned baseline.
  No sanitizer was run.
- Combined validation after the correction and increments 229–231:
  `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, library/binary build,
  changed-file rustfmt, Python AST, and commit diff checks passed. The
  hyperlink capture, format-job output, customize reset/unset, format
  expansion, file path, and embedded-NUL copy-view CLI checks passed with the
  pinned baseline. No sanitizer was run.

### Increment 232 — environment assignment name scratch owner (2026-09-23)

- `environ_put` and `window_customize_add_environment_callback` now isolate
  the name before the first `=` in local `CString` values. `environ_set` copies
  a new name or only looks up an existing one; its variadic formatter consumes
  the borrowed value synchronously. Removed both whole-assignment
  `xstrdup`/delimiter-write/`free` paths without changing callback ABI or
  record layout. The first-NUL and non-UTF-8 behavior remains byte-preserving.
- `cargo test --test environment` passed, including a new case for the first
  `=`, empty name, missing `=`, non-UTF-8 bytes, and first NUL. Binary build,
  changed-file rustfmt, diff checks, and the attached customize environment
  prompt/row CLI comparisons with the pinned baseline passed. No sanitizer was
  run.

### Increment 233 — retained hyperlink URI and internal ID owners (2026-09-23)

- `HyperlinkUriOwner` now holds the URI and internal ID as `CString` fields
  alongside its external ID. The public `hyperlinks_uri` string pointers
  borrow these fields until `hyperlinks_remove` unlinks and drops the owner.
  A private `utf8_stravis_cstring` writes the unchanged `utf8_strvis` byte
  conversion directly into Rust-owned storage; duplicate lookup, length
  rejection, eviction, and reset no longer allocate and free C strings for
  these fields.
- Hyperlink storage and real-pane OSC 8 tests, binary build, changed-file
  rustfmt, and diff checks passed. The new live CLI check matched the pinned
  baseline for invalid UTF-8 in both retained fields. No sanitizer was run.

### Increment 234 — command-template output owner (2026-09-23)

- `cmd_template_replace` now builds substituted and quoted bytes in a local
  `Vec<u8>` and exposes a private `CString` result. It removes per-character
  `xrealloc`, manual output length, and pointer writes. The exported function
  still duplicates once for its callers' libc-freeable return contract;
  literal, numbered, first-`%%`, single-quote, and double-quote behavior stays
  byte-preserving through the first input NUL.
- Focused substitution/quoting tests, binary build, changed-file rustfmt, and
  diff checks passed. The attached-client mutable-prompt CLI check passed on
  both the migrated and pinned baseline binaries. No sanitizer was run.
- Combined validation after increments 232–234: `RUST_TEST_THREADS=1 cargo
  test --workspace --quiet`, library/binary build, changed-file rustfmt,
  Python AST, and commit diff checks passed. Hyperlink URI/ID, customize
  environment prompt/rows, mutable prompt, and embedded-NUL copy-view CLI
  checks passed with the pinned baseline. No sanitizer was run.

### Increment 235 — copy-mode match text output owner (2026-09-23)

- `window_copy_match_at_cursor` now accumulates tab and grid-cell bytes in a
  local `Vec<u8>` and makes one final NUL-terminated C allocation for the
  `search_match` format callback and selection fallback. Removed the
  per-cell `xrealloc`, manual length, and pointer writes. Empty matches still
  return null; interior NUL bytes stay in the allocated result even though
  both C-string consumers stop at the first NUL.
- Binary build, `copy_regex_cells` test, changed-file rustfmt, Python AST, and
  diff checks passed. `scripts/copy_match_owner_cli_checks.py` matched the
  pinned baseline for search-match formatting and copy fallback with a tab
  and wide characters. No sanitizer was run.

### Increment 236 — regex substitution output owner (2026-09-23)

- `regsub_copy`, `regsub_expand`, and `regsub` now build substitution bytes in
  a local `Vec<u8>` instead of growing a raw buffer with `xrealloc` and a
  manual length. The exported `regsub` still makes one NUL-terminated C
  allocation for the format caller's libc-freeable return contract. Backrefs,
  anchors, empty matches, and deletion keep their existing byte behavior.
- Focused `regsub_owner` and existing format modifier tests, binary build,
  changed-file rustfmt, and diff checks passed. The format-unescape CLI check
  matched the pinned baseline. No sanitizer was run.

### Increment 237 — format trim output owner (2026-09-23)

- `format_trim_left` and `format_trim_right` now assemble style tokens,
  multibyte cells, and ordinary bytes in local `Vec<u8>` values. This removes
  their oversized `xcalloc` scratch buffers and manual output-pointer writes.
  Each result still makes one C-owned copy for the format, menu, and window
  tree callers that libc-free it; the right-trim early `xstrdup` return keeps
  its existing contract.
- Focused trim tests for styles, UTF-8, invalid bytes, and first NUL,
  library/binary build, changed-file rustfmt, Python AST, and diff checks
  passed. `scripts/format_trim_owner_cli_checks.py` matched the pinned
  baseline for positive and negative widths. No sanitizer was run.
- Combined validation after increments 235–237: `RUST_TEST_THREADS=1 cargo
  test --workspace --quiet`, library/binary build, changed-file rustfmt,
  Python AST, and each commit's diff check passed. Copy-match, trim,
  format-unescape, and embedded-NUL copy-view CLI checks matched the pinned
  baseline. No sanitizer was run.

### Increment 238 — detached show-buffer path borrow (2026-09-23)

- `cmd_save_buffer_exec` now borrows the static `"-"` path used by detached
  `show-buffer`, removing its `xstrdup` and matching `free`. The synchronous
  `file_write` call copies the path. The formatted `save-buffer` path still
  returns a C-owned allocation and is freed after that call.
- Library/binary build, changed-file rustfmt, diff check, and
  `scripts/file_path_owner_cli_checks.py` against the pinned baseline passed.
  No sanitizer was run.

### Increment 239 — copy-mode regex string buffer owner (2026-09-23)

- `window_copy_stringify` now appends into a `Vec<u8>` owned by each of its
  four caller paths. This removes the shared `(buf, size)` `xrealloc` lifecycle
  and all matching frees. The vector keeps interior NUL bytes and one final
  terminator; `regexec`, `regcomp`, and cell mapping borrow its pointer only
  for synchronous calls.
- The live `copy_regex_cells` test now covers regex searches across wrapped
  lines in both directions, including wide and multibyte cells. Both focused
  tests, changed-file rustfmt, diff check, and a pinned-baseline wrapped-line
  trace passed. No sanitizer was run.

### Increment 240 — Linux process command owner (2026-09-23)

- `osdep_get_name_cstring` now reads `/proc/<pgrp>/cmdline` into a local byte
  vector and returns an optional `CString`, stopping at the first NUL as
  before. The format current-command callback and window-tree pane search own
  that string through their synchronous consumers, removing their manual
  frees. The exported `osdep_get_name` keeps its libc-owned return contract
  with one final duplicate for external callers.
- Library/binary build, `scripts/osdep_proc_cli_checks.py`, changed-file
  rustfmt, and diff checks passed. No sanitizer was run.
- Combined validation after increments 238–240: `RUST_TEST_THREADS=1 cargo
  test --workspace --quiet`, library/binary build, changed-file rustfmt, and
  per-commit diff checks passed. File-path, copy-match, and embedded-NUL
  copy-view CLI checks matched the pinned baseline; the Linux `/proc` CLI
  check passed. No sanitizer was run.

### Increment 241 — format job completion output owner (2026-09-23)

- `format_job_complete` now owns its completion output in a `CString`. The
  no-newline evbuffer branch copies bytes through the first NUL directly into
  that owner, removing its temporary `xmalloc`, manual terminator write, and
  matching free. The `evbuffer_readline` branch still copies and frees its
  libc result before the same owner takes over. Empty output and `updated`
  behavior are unchanged.
- Focused format-job tests, library/binary build, changed-file rustfmt,
  Python compile, diff check, and the live status-job CLI check passed. The
  CLI check includes `visible\\000hidden` and passed with the pinned baseline.
  No sanitizer was run.

### Increment 242 — capture-pane output buffer owner (2026-09-23)

- `cmd_capture_pane_append` and its grid, pending, hyperlink, and history
  producers now use `Vec<u8>` for captured bytes. This removes repeated
  `xrealloc`, scratch `xstrdup`, manual length, and associated frees. The
  `-p` paths borrow bytes for synchronous control/file output; `paste_set`
  alone receives a C-owned copy because it retains the payload on success.
  Empty output remains distinct from a history error, and the control `%.*s`
  path still stops at the first NUL.
- Library/binary build, `osc8_hyperlink_id` test, changed-file rustfmt,
  diff check, and live capture-pane checks passed. The new CLI check matched
  the pinned baseline for grid, history, hyperlink, pending, paste transfer,
  and error paths, including raw `ESC [ NUL` and escaped pending input. No
  sanitizer was run.

### Increment 243 — positional argument command-cache owner (2026-09-23)

- `args_create` now allocates an `ArgsOwner` with the public `args` record at
  offset zero and a `Vec<Option<CString>>` indexed by positional argument.
  `args_string` caches command-list print output there and publishes a
  borrowed pointer through the unchanged `args_value.cached` field. This
  removes that cache's C allocation/free pair while allowing positional
  values to move under `xrecallocarray`. `args_free` clears only these borrowed
  pointers before the exported `args_free_value` handles independently
  C-owned values, then drops the owner. `args_copy` starts with independent
  cache storage; parse/log-only values use scoped `Cow<CStr>` and do not
  acquire a cache. The `args` and `args_value` layouts and exported signatures
  remain unchanged, and `args` is no longer `Copy` in Rust.
- Every in-tree `args` allocation uses `args_create` and destruction uses
  `args_free`; callers of exported `args_free_value` on separate source arrays
  retain their libc-owned cache contract. The enclosing owner requires this
  creator/destructor pairing for foreign callers as well.
- All 10 `arguments_conversion` tests, library/binary build, changed-file
  rustfmt, diff check, and pinned-baseline command-list, command-parse, and
  argument-print CLI comparisons passed. The new test covers caching before
  positional-array growth, independent copied caches, and rejection cleanup.
  No sanitizer was run.
- Combined validation after increments 241–243: `RUST_TEST_THREADS=1 cargo
  test --workspace --quiet`, library/binary build, changed-file rustfmt, and
  per-commit diff checks passed. Format-job and capture-pane CLI checks passed;
  capture-pane and the three argument-print checks matched the pinned
  baseline. No sanitizer was run.

### Increment 244 — file write message scratch owner (2026-09-23)

- `file_push` now builds each `MSG_WRITE` header and payload in a reusable
  `Vec<u8>`, removing the `msg_write_data` `xmalloc`/`xrealloc`/`free`
  lifecycle. `proc_send` copies bytes synchronously through `imsg_compose`
  and `ibuf_add`; the existing send-failure break and retry scheduling remain.
- Library/binary build, workspace tests, changed-file rustfmt, diff check,
  and `scripts/file_message_scratch_cli_checks.py` passed. The CLI check
  matched the pinned baseline while transferring 65,809 binary bytes with
  embedded NUL and `0xff` through load, show, and save paths. No sanitizer
  was run.

### Increment 245 — file read message scratch owner (2026-09-23)

- `file_read_callback` now reuses a `Vec<u8>` for each `MSG_READ` header and
  payload, removing its `msg_read_data` `xmalloc`/`xrealloc`/`free` lifecycle.
  `proc_send` copies the message synchronously before evbuffer input is
  drained. The previous chunking, send result handling, and drain order stay
  the same.
- Library/binary build, workspace tests, changed-file rustfmt, diff check,
  and the 65,809-byte pinned-baseline file-message CLI comparison passed. No
  sanitizer was run.

### Increment 246 — command lexer scratch owner (2026-09-23)

- `yylex_get_word`, `yylex_format`, `yylex_token`, and their escape, variable,
  and tilde helpers now grow a `Vec<u8>` lexer scratch buffer. This removes
  per-byte `xrealloc`, manual `(buf, len)` updates, and explicit error-path
  frees. A completed token gets one C allocation for the parser's existing
  libc-freeable `YYSTYPE` contract. Embedded bytes are preserved through the
  handoff; C-string consumers still stop at the first NUL.
- Workspace tests, library/binary build, changed-file rustfmt, diff check,
  the existing parser CLI check, and
  `scripts/lexer_scratch_owner_cli_checks.py` passed. The new CLI check matched
  the pinned baseline for an 8 KiB token, nested formats, expansions,
  Unicode and octal escapes, first-NUL behavior, and lexer errors. Parser
  semantic-value cleanup remains outside this scratch migration. No sanitizer
  was run.
- Combined validation after increments 244–246: `RUST_TEST_THREADS=1 cargo
  test --workspace --quiet`, library/binary build, changed-file rustfmt,
  per-commit diff checks, and the file-message and lexer CLI comparisons
  passed. Both CLI checks matched the pinned baseline. No sanitizer was run.

### Increment 247 — window copy selection owner (2026-09-23)

- `window_copy_get_selection` and `window_copy_copy_line` now collect selection
  bytes in `Vec<u8>`. The pipe and clipboard paths borrow those bytes during
  synchronous calls; copy and append allocate a C buffer only when handing
  ownership to paste storage. The search-match fallback keeps its previous
  first-NUL length behavior, while the format callback still returns a
  libc-freeable C string as required by its ABI.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, and
  `scripts/window_copy_selection_owner_cli_checks.py` passed. The CLI check
  matched the pinned baseline for copy, append, pipe, copy-pipe, UTF-8/tab
  text, and appending after a binary paste buffer containing an embedded NUL.
  No sanitizer was run.

### Increment 248 — client exit message owner (2026-09-23)

- `client_exitmessage` now stores the received `MSG_EXIT` payload in
  `Option<Vec<u8>>`. The C printf path borrows its pointer until final exit
  output, then `client_main` drops the owner. A repeated message also drops
  the previous payload. The last received byte is still forced to NUL.
- Library/binary build, changed-file rustfmt, diff check, the agent's
  workspace test run, and `scripts/server_exit_payload_cli_checks.py` passed.
  The CLI check matched the pinned baseline for a normal exit, an attached
  exit, and a socket startup error. No sanitizer was run.

### Increment 249 — grid string cell output owner (2026-09-23)

- `grid_string_cells_bytes` now builds grid-line output in `Vec<u8>` instead
  of growing a C allocation. Capture-pane consumes those bytes directly and
  keeps its former first-NUL truncation. Window-pane search also consumes the
  owned bytes, preserving first-NUL truncation, libc whitespace trimming,
  and a terminal NUL for its synchronous C search and logging calls. The
  exported `grid_string_cells` and `grid_view_string_cells` wrappers still
  produce libc-freeable results for their C ABI.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, and the pinned-baseline
  `capture_pane_buffer_owner_cli_checks.py`,
  `capture_pane_grid_cell_cli_checks.py`, and `format_search_cli_checks.py`
  passed on main after increments 247–249. No sanitizer was run.

### Increment 250 — input parser buffer owner (2026-09-23)

- `InputCtxOwner` now owns the growable parser `Vec<u8>` alongside its stable
  C-layout `input_ctx` prefix. `input_buf` and `input_space` remain borrowed
  ABI views refreshed after growth or shrink. `input_init` and `input_free`
  allocate and drop the full owner, removing the context `xcalloc`/`free`
  and buffer `xmalloc`/`xrealloc`/`free` lifecycle together.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, `input_reply_cli_checks.py`, and
  `input_osc_104_cli_checks.py` passed. The new
  `input_buffer_owner_cli_checks.py` passed on both main and the pinned
  baseline with a long OSC title followed by a short title after returning
  to ground. No sanitizer was run.

### Increment 251 — fallback visible range storage (2026-09-23)

- `window_visible_ranges` now uses one static `visible_range` for its
  null-range fallback instead of lazily `xcalloc`-allocating a process-lifetime
  slot in two branches. The returned `visible_ranges` ABI shape, `used` and
  `size` values, and shared mutable fallback behavior remain the same. Pane
  range storage is a separate owner and is unchanged.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, pinned-baseline attached-client
  `window_copy_vadd_owner_cli_checks.py`, and `layout_cli_checks.py` passed.
  No sanitizer was run.

### Increment 252 — pane input callback record owner (2026-09-23)

- `window_pane_input_data` is now a private `Box` carried as callback data
  through `file_read` and dropped by the terminal callback. This removes its
  `xmalloc`/`free` pair and obsolete `Copy` implementation. The client
  reference is taken before `file_read`; a null return leaves the callback
  record's file field null for its scheduled completion, avoiding an access
  after that callback may release the owner.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, and
  `scripts/pane_input_owner_cli_checks.py` passed. The CLI check matched the
  pinned baseline for large stdin, later `display-message -I` input, a
  control-client failed open, and pane cancellation after streamed input.
  No sanitizer was run.

### Increment 253 — format draw range scratch owner (2026-09-23)

- `format_draw` now owns its temporary `format_range` sequence in `Vec` and
  its in-progress range in `Option`. This removes each range's `xcalloc` and
  `free`, the intrusive queue links, manual unlinking, and early-return
  cleanup. The output `style_range` list remains C-owned for its existing
  caller contract. During integration, the final output loop was changed to
  read range fields directly instead of casting a borrowed element to a raw
  pointer.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, and
  `scripts/format_draw_style_cli_checks.py` passed on main. The CLI check
  passed with the pinned baseline too, including clicks on two distinct user
  status ranges. No sanitizer was run.

### Increment 254 — file read-open message scratch owner (2026-09-23)

- `file_read` now builds each `MSG_READ_OPEN` header and NUL-terminated path
  in a local `Vec<u8>`. `proc_send` copies the message synchronously through
  `imsg_compose` and `ibuf_add`, so the vector drops after the send result.
  This removes the temporary message's `xmalloc`/`free` pair on both success
  and failure without changing the file stream owner.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, and the pinned-baseline
  `file_path_owner_cli_checks.py` and `file_message_scratch_cli_checks.py`
  passed. No sanitizer was run.

### Increment 255 — file write-open message scratch owner (2026-09-23)

- `file_write` now constructs each `MSG_WRITE_OPEN` header and path in a local
  `Vec<u8>`, removing that temporary message's `xmalloc`/`free` pair. The
  message is borrowed only until `proc_send` copies it through `imsg_compose`
  and `ibuf_add`; the existing send-failure error and completion path remain.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, and pinned-baseline
  `file_message_scratch_cli_checks.py` and `file_path_owner_cli_checks.py`
  passed on main after both open-message migrations. The message check
  transferred 65,809 binary bytes including NUL and `0xff`. No sanitizer was
  run.

### Increment 256 — pane visible range owner (2026-09-23)

- `WindowPaneOwned` now places the C-layout `window_pane` at offset zero and
  owns pane visible-range elements in a trailing `Vec<visible_range>`.
  `window_pane_create` and `window_pane_free` allocate and drop that complete
  owner, removing the pane range's C realloc/free lifecycle. The public
  `pane.r` pointer/count remains a synchronous ABI view and is refreshed
  after growth. The shared `server_client_ensure_ranges` allocation path
  remains for tty, popup, and caller-supplied range views.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, and
  `scripts/pane_visible_ranges_cli_checks.py` passed on main. The attached
  floating-pane redraw check also passed with the pinned baseline. No
  sanitizer was run.

### Increment 257 — display-panes preview screen owner (2026-09-23)

- `window_panes_modedata.preview` now holds `Option<Box<screen>>` instead of a
  separately malloc-allocated raw screen. The preview is created once during
  mode initialization; draw paths take a short-lived raw pointer for their C
  screen calls. At mode teardown, `screen_free` still precedes the box drop,
  removing the preview `xmalloc`/`free` pair without changing its grid cleanup.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, and
  `scripts/display_panes_command_cli_checks.py` passed on main. The attached
  display-panes check also passed with the pinned baseline. No sanitizer was
  run.

### Increment 258 — session termios owner (2026-09-23)

- `SessionOwner` now holds the C-layout `session` at offset zero and owns its
  optional copied `termios` value in `Option<Box<termios>>`. The public
  `session.tio` pointer borrows that stable value. `session_destroy` nulls the
  view and drops the termios box at the former C free point; deferred
  `session_free` drops the enclosing owner. The session allocation, termios
  allocation, and matching destruction paths migrate together.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, and
  `scripts/session_tio_owner_cli_checks.py` passed on main. The pinned-baseline
  attached PTY check verified custom VINTR/VEOF settings across two panes and
  session teardown. No sanitizer was run.

### Increment 259 — queued key-event owner (2026-09-23)

- Terminal input, `send-keys -K`, and the click timer now create an
  `OwnedKeyEvent` box. Terminal bytes live in its `Option<Vec<u8>>` until the
  queued callback returns; rejected events drop immediately. The copied
  command state clears its borrowed byte pointer and length because its
  consumers inspect only the key and mouse fields. The C-layout `key_event`
  remains a view for synchronous callbacks.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, commit diff check, and the attached-PTY
  `scripts/tty_key_event_owner_cli_checks.py` passed on main and the pinned
  baseline. Both captured `[A^@B, AB, C, C]` for terminal `A\0B` and
  `send-keys -K`. The click timer was source-audited but not live E2E tested.
  No sanitizer was run.

### Increment 260 — confirmation prompt callback owner (2026-09-23)

- `cmd_confirm_before_data` now has a `Box` owner. Creation keeps the box local
  through command construction and key validation; the prompt receives it
  only after setup succeeds. The registered free callback releases the nested
  C-owned command list and drops the box. Prompt replacement and cancellation
  still use the same status prompt free callback.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, commit diff check, and
  `scripts/confirm_before_owner_cli_checks.py` passed on main. The attached
  PTY comparison with the pinned baseline covered accept, reject, custom key
  with default yes, prompt replacement, and invalid-key cleanup. No sanitizer
  was run.

### Increment 261 — clock mode data owner (2026-09-23)

- `window_clock_mode_data` now has a stable `Box` allocation through the
  screen and timer lifetime. `window_clock_free` cancels the timer, frees the
  screen, then drops the box at the previous C free point. The timer callback
  still rearms only while the mode is active.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, commit diff check, and
  `scripts/window_clock_owner_cli_checks.py` passed on main and the pinned
  baseline. The attached PTY check entered clock mode twice, observed redraw,
  resized the window, waited for timer redraw, and exited through a key.
  No sanitizer was run.

### Increment 262 — run-shell callback data owner (2026-09-23)

- `cmd_run_shell_data` now uses `Box` for its stable allocation across timer,
  command, and job callbacks. Its single free callback cancels the timer,
  releases session/client references and nested C-owned state/strings, then
  drops the box. The record is no longer `Copy`.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, and
  `scripts/run_shell_data_owner_cli_checks.py` passed on main. The pinned
  baseline comparison covered immediate output, command mode, delayed
  background execution, and a failing job. No sanitizer was run.

### Increment 263 — event sink owner (2026-09-23)

- `EventsSinkOwner` now holds the C-layout `events_sink` prefix at a stable
  address and owns its name as `CString`. Immediate removal and deferred
  removal after dispatch both unlink the sink and drop the box, releasing the
  name with it. The public sink pointer and queue layout remain unchanged.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, commit diff check, and hook-monitor string/append CLI
  checks passed on main and the pinned baseline. The focused ownership test
  covers mutation of the caller's name, removal during dispatch, and adding a
  new generation while dispatching. No sanitizer was run.

### Increment 264 — if-shell callback data owner (2026-09-23)

- `cmd_if_shell_data` now uses `Box` through its job callback lifetime, and
  the record is no longer `Copy`. Successful job creation transfers the box to
  the existing free callback; job creation failure reclaims it there. The
  callback still releases nested command states and the client reference in
  the previous order.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, commit diff check, and
  `scripts/if_shell_owner_cli_checks.py` passed on main. The pinned-baseline
  comparison covered true/false jobs, a callback parse error, background
  execution, and the `-F` branch. Job creation failure was source-audited but
  not fault injected. No sanitizer was run.

### Increment 265 — server ACL entry owner (2026-09-23)

- `server_acl_entries` now stores `Box<server_acl_entry>` values under its
  existing user/group semantic keys. Lookup and display borrow stable entry
  pointers; deny removes and drops the box, while ACL initialization clears
  and drops any previous entries. Removed the entry's `Copy` derive and its
  `xcalloc`/`free` pair.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, and
  `scripts/server_acl_owner_cli_checks.py` passed on main. The pinned-baseline
  comparison covered group grant, duplicate grant, read-only/write updates,
  listing, and revocation. No sanitizer was run.

### Increment 266 — wait-for event waiter owner (2026-09-23)

- `WaitEventOwner` now keeps the C-layout `wait_event_item` prefix stable and
  owns its event name and optional filter as `CString` values. Completion,
  explicit wake, and server flush remove the registered sink before dropping
  the box. Removal during event dispatch marks the sink dead until dispatch
  ends, so its callback data is not read after the waiter drops.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, commit diff check, and
  `scripts/wait_event_owner_cli_checks.py` passed on main. The pinned-baseline
  comparison covered filter rejection, event completion with verbose payload,
  explicit wake, and server flush. No sanitizer was run.

### Increment 267 — saved status screen owner (2026-09-23)

- `ClientOwner` now holds `Option<Box<screen>>` for the saved status screen.
  `status.active` remains a borrowed C-layout pointer view while messages or
  prompts hold references. Final pop and client status teardown call
  `screen_free`, clear the view, and drop the box in that order.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, commit diff check, and
  `scripts/status_saved_screen_owner_cli_checks.py` passed on main. The
  attached-PTY comparison with the pinned baseline covered message
  replacement and timer expiry, a prompt, and client teardown with an active
  message or prompt. No sanitizer was run.

### Increment 268 — menu display owner (2026-09-23)

- `MenuDisplayOwner` now holds the C-layout `menu_data` prefix and owns its
  optional style, selected-style, and border-style strings as `CString`.
  Window menu pointers remain borrowed stable views. Menu close or window
  destruction still calls the choice callback, frees the screen and menu,
  then drops the owner and its strings; `menu_data` is no longer `Copy`.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, and the expanded
  `scripts/menu_owner_cli_checks.py` passed on main and the pinned baseline.
  The attached check covered styled rendering, menu replacement, selection,
  and overlay teardown. No sanitizer was run.

### Increment 269 — choose-buffer editor owner (2026-09-23)

- `window_buffer_editdata` now has one stable `Box` allocation and owns its
  buffer name as `CString`. The spawn callback and mode keep borrowed pointers;
  normal editor completion, spawn failure, and mode teardown release the box
  at their existing finalization points. The record is no longer `Copy`.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, commit diff check, and
  `scripts/window_buffer_edit_owner_cli_checks.py` passed on main. The
  attached-client comparison with the pinned baseline covered an editor that
  updates the buffer and a mode reset while an editor is still running.
  No sanitizer was run.

### Increment 270 — wait-for channel and queue owners (2026-09-23)

- The `wait_channels` semantic-key map now owns stable boxed channel records
  and their `CString` names. Intrusive waiter and locker queues borrow each
  channel's C-layout prefix. Their `wait_item` nodes use `Box` and drop after
  unlinking on explicit wake, signal, unlock, and server flush. Channel
  removal drops the map value after its queues are empty.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, commit diff check, and
  `scripts/wait_channels_owner_cli_checks.py` passed on main. The pinned
  baseline comparison covered remembered signal, explicit wake, multiple
  waiters, lock handoff/unlock, and server flush of waiters and lockers.
  No sanitizer was run.

### Increment 271 — spawned editor state and path (2026-09-23)

- `SpawnEditorOwner` now holds the stable C-layout `spawn_editor_state` prefix
  in a `Box` and owns its temporary path as `CString`. Panes and editor mode
  data retain the existing borrowed state pointer. Editor completion, canceled
  callback, and spawn failure unlink the path before dropping the owner; the
  callback and PID ABI remain unchanged.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, and attached-client
  `scripts/window_buffer_edit_owner_cli_checks.py` and
  `scripts/spawn_editor_command_cli_checks.py` passed. The pinned baseline
  comparison covered normal edit, mode cancellation while the editor was
  running, and temporary-file removal after both paths. No sanitizer was run.

### Increment 272 — popup overlay record and strings (2026-09-23)

- `PopupOwner` now holds the stable C-layout `popup_data` prefix and owns its
  optional title, style, and border-style strings as `CString`. The overlay
  and job callbacks retain borrowed pointers. `popup_modify` copies a new
  string before replacing the old one; the free callback releases the job,
  input, ranges, screen, and palette before dropping the owner.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, and `scripts/popup_owner_cli_checks.py`
  passed on main. The attached-client check also passed with the pinned
  baseline and covered rendering, modification, explicit close, job exit,
  and client detachment. No sanitizer was run.

### Increment 273 — status prompt callback record (2026-09-23)

- `status_prompt_data` now has one stable `Box` allocation; its C-layout
  pointer remains callback data for the prompt. The free callback invokes the
  caller's `freecb` first, then drops the box, preserving replacement and
  client-teardown order.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, and
  `scripts/confirm_before_owner_cli_checks.py` and
  `scripts/prompt_mutable_cli_checks.py` passed on main. The attached-client
  check also passed with the pinned baseline and covered accepting, rejecting,
  replacing, and detaching a client with an active prompt. No sanitizer was
  run.

### Increment 274 — customize-mode format string (2026-09-23)

- `window_customize_modedata` now stores its `-F` format as `CString` in the
  existing stable `Box`. Synchronous `format_expand` calls borrow its C view;
  the string remains valid through any deferred prompt or editor reference and
  drops with the mode record at its final reference.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, `scripts/customize_option_cli_checks.py`,
  and `scripts/customize_scope_text_cli_checks.py` passed on main. The scope
  check also passed with the pinned baseline and covered custom `-F` output
  for server, session, window, pane, and environment rows, followed by mode
  teardown. No sanitizer was run.

### Increment 275 — choose-client mode strings (2026-09-23)

- `window_client_modedata` now owns its format, key format, and command as
  `CString` fields in the existing stable `Box`. Row formatting, key parsing,
  and command execution borrow C views for their synchronous calls. The three
  `xstrdup` allocations and matching manual frees are gone; item and client
  reference cleanup still precedes box destruction.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, and
  `scripts/window_client_mode_strings_cli_checks.py` passed on main and the
  pinned baseline. The attached-client check covered custom `-F`, `-K`, and
  selection command, cancel/recreate, and detachment with a live mode.
  No sanitizer was run.

### Increment 276 — choose-tree mode strings (2026-09-23)

- `window_tree_modedata` now owns its format, key format, and command as
  `CString` fields in the existing stable `Box`. Row formatting, key parsing,
  and command execution borrow C views. The three `xstrdup` allocations and
  matching frees are gone; deferred prompt callbacks keep the mode record and
  its strings alive until the last reference is released.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check,
  `scripts/window_tree_strings_owner_cli_checks.py`, and
  `scripts/window_tree_target_cli_checks.py` passed on main. The attached
  string check also passed on the pinned baseline and covered custom `-F`,
  `-K`, and command values, empty and accepted prompts, deferred command
  execution, and teardown. No sanitizer was run.

### Increment 277 — mode-tree prompt callback record (2026-09-23)

- `mode_tree_prompt` now uses a stable `Box` for callback data rather than
  `xcalloc` and a matching manual `free`. Its free callback clears the
  mode-tree observer, invokes the caller's free callback, releases the
  mode-tree reference, then drops the box in the original order. The public
  callback record and C layout remain unchanged.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, and attached-client
  `scripts/window_tree_prompt_cli_checks.py` and the extended
  `scripts/window_tree_strings_owner_cli_checks.py` passed on main. The latter
  also passed with the pinned baseline and now covers destroying a pane while
  its mode-tree prompt is active, alongside empty and accepted prompts.
  No sanitizer was run.

### Increment 278 — choose-buffer mode strings (2026-09-23)

- `window_buffer_modedata` now owns its command, row format, and key format as
  `CString` fields in the existing stable `Box`. Formatting, key parsing, and
  command execution borrow C views. The three `xstrdup` allocations and
  matching frees are gone; editor cancellation, mode-tree teardown, and item
  cleanup still precede dropping the box.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check,
  `scripts/window_buffer_mode_strings_cli_checks.py`, and
  `scripts/window_buffer_edit_owner_cli_checks.py` passed on main. Both
  attached-client checks passed with the pinned baseline; they covered
  custom `-F`, `-K`, command expansion, cancel, detachment with a live mode,
  and editor completion/cancellation. No sanitizer was run.

### Increment 279 — switch-mode strings and mutable filter (2026-09-23)

- `window_switch_modedata` now owns its row format, command, and mutable
  filter as `CString` fields in the existing stable `Box`. The incremental
  prompt callback copies input before replacing the filter, so an input view
  into existing mode storage remains valid. Formatting and command execution
  borrow C views; the `xstrdup` allocations and matching frees are gone.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check,
  `scripts/window_switch_mode_strings_cli_checks.py`, and
  `scripts/window_switch_target_cli_checks.py` passed on main. The new
  attached-client check also passed with the pinned baseline and covered
  custom `-F` and command values, incremental typing and deletion, selection
  of two targets, and cancel teardown. No sanitizer was run.

### Increment 280 — mode-tree menu callback record (2026-09-23)

- `mode_tree_menu` now uses a stable `Box` for the menu callback data in place
  of `xmalloc` and manual `free`. The callback releases its mode-tree
  reference before dropping the box; the `menu_display` error path keeps the
  same reference, record, and menu cleanup order. The record's C layout and
  callback pointer remain unchanged.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, and the extended attached-client
  `scripts/mode_tree_menu_title_cli_checks.py` passed on main and the pinned
  baseline. It covered inside and outside menus, selection/cancel callbacks,
  and mode teardown. The current `menu_display` implementation returns zero
  on every path, so its error branch was reviewed but could not be reached
  through this E2E. No sanitizer was run.

### Increment 281 — mode-tree item and row strings (2026-09-23)

- `ModeTreeItemOwner` now holds the C-layout `mode_tree_item` prefix in a
  stable `Box` and owns its optional identity name/detail, required row name,
  optional row text, and mutable key label as `CString`. Intrusive lists and
  callbacks keep the existing item pointer. Recursive item cleanup drops the
  owner after its children; repeated key-label builds replace the old string
  without leaking it. A compile-time offset check fixes the prefix layout.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, diff check, a focused repeated-key-build test, and
  attached-client `mode_tree_row`, `window_tree_target`, and
  `customize_array_key_prompt` scripts passed on main. The three E2E checks
  also passed with the pinned baseline. No sanitizer was run.

### Increment 282 — mode-tree filter and search strings (2026-09-23)

- `ModeTreeOwner` now embeds the unchanged C-layout `mode_tree_data` prefix
  in a stable `Box` and owns its mutable search and filter strings as optional
  `CString`s. A compile-time offset check protects the callback pointer cast.
  The raw fields remain borrowed views updated only by the owner setters;
  prompt callbacks copy input before replacing an owner. Mode teardown drops
  both strings with the record, replacing the `xstrdup`/`free` pairs.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, and diff checks passed in the isolated worktree.
  The attached-client filter/search script matched the pinned baseline for
  initial `-f`, replacement, clear, successive searches, selection, and
  active-prompt teardown. No sanitizer was run.

### Increment 283 — redraw scene spans (2026-09-23)

- `redraw_make_scene` now boxes each `redraw_span` at its stable intrusive-list
  address, and `redraw_free_scene` drops that box after unlinking the span.
  This removes the span `xcalloc`/`free` pair while keeping list links,
  traversal, scene invalidation, and public pointer layout unchanged. The
  span record is no longer `Copy`; the containing scene and line array retain
  their separate C allocations.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, and diff checks passed. Attached-client
  `pane_visible_ranges` and `window_clock_owner` checks passed on both the
  candidate and pinned baseline, exercising scene rebuild on floating-pane
  changes, resize, timer redraw, and teardown. No sanitizer was run.

### Increment 284 — option rollback string snapshot (2026-09-23)

- The string arm of `options_from_string` now owns the previous option value
  as a local `CString`. It copies that value before `options_set_string`
  invalidates the old storage, then borrows the snapshot for synchronous
  rollback on validation failure. This removes its `xstrdup` and both frees;
  success and failure still return at the same points.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, Python syntax, and diff checks passed. The extended
  `options_string` CLI check passed on candidate and pinned baseline for
  non-UTF-8, append, empty, valid `default-shell`, rejected replacement, and
  rejected append with rollback. `options_value_owner` also passed on the
  candidate. No sanitizer was run.

### Increment 285 — redraw scene record (2026-09-23)

- `redraw_make_scene` now creates the stable `redraw_scene` record in a `Box`;
  `redraw_free_scene` releases it after unlinking and dropping spans and
  freeing the line array. `client.redraw_scene` remains a borrowed pointer
  invalidated by the same free/rebuild path. The scene record is no longer
  `Copy`, and its `xcalloc`/`free` pair is gone. Its line array remains C-owned
  pending a separate alias audit.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, and diff checks passed. Attached-client
  `pane_visible_ranges` and `window_clock_owner` checks passed on the
  candidate; both also passed on the pinned baseline during increment 283.
  No sanitizer was run.

### Increment 286 — customize editor value buffer (2026-09-23)

- `window_customize_edit_close_cb` now owns the copied editor bytes in a
  local `Vec<u8>` with one appended NUL, removing its `xmalloc`/`free` pair
  and manual `memcpy`/terminator write. The C-owned editor input is still freed
  after copying, and the vector is dropped before `window_customize_finish_edit`.
  Synchronous option, command, note, and environment setters borrow its
  first-NUL C view; bytes after an embedded NUL remain in the temporary
  vector just as they did in the old counted allocation.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, Python syntax, and diff checks passed. The new
  attached-client customize-editor script matched the pinned baseline for
  UTF-8, `A\0B` with first-NUL storage, empty editor output, cancellation,
  and editor temporary-file cleanup. Existing customize-option and
  spawn-editor command checks also passed. No sanitizer was run.

### Increment 287 — customize editor callback record (2026-09-23)

- `window_customize_start_edit` now creates `window_customize_editdata` in a
  stable `Box`; `window_customize_finish_edit` drops it after freeing the
  copied item. The callback and mode's `edit` field retain the same borrowed
  pointer until completion or cancellation. The record is no longer `Copy`,
  and its `xcalloc`/`free` pair is gone. `spawn_cancel_editor` clears the
  callback before mode teardown drops the record; failed editor startup
  takes the same finish path.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, and diff checks passed. The attached-client customize
  editor script passed against the pinned baseline for valid, embedded-NUL,
  empty, and cancelled edits, including temporary-file cleanup. Failed editor
  startup was reviewed but not forced by this E2E. No sanitizer was run.

### Increment 288 — spawn environment log prefix (2026-09-23)

- `environ_log` now holds its formatted prefix as a local `CString` through
  the synchronous environment log loop. It reuses `xvasprintf_cstring`,
  removing the direct `vasprintf` allocation, raw pointer, and matching free.
  The only production caller supplies a fixed `"%s: environment "` format and
  `"spawn_pane"` argument, so no supported input can insert a middle NUL in
  this prefix. The exported variadic function signature is unchanged.
- Library/binary build, `RUST_TEST_THREADS=1 cargo test --workspace --quiet`,
  changed-file rustfmt, Python syntax, and diff checks passed. A private `-vv`
  server E2E compared normalized environment log lines with the pinned
  baseline for ASCII and non-UTF-8 values (`OWNER_RAW=owner-\\377`). No
  sanitizer was run.

### Next candidates

The later layout-equivalence cleanup removed the detached
`LayoutDescription` API and its API-only tests. `layout_parse` again builds
`layout_cell` and `layout_parse_ctx` directly, as in tmux, while the
`LayoutString` serializer owner remains. `cargo test --workspace`, the binary
build, and `scripts/layout_cli_checks.py` passed; the same CLI script also
passed with the pinned tmux binary, including an ignored `I` field with a
non-string value.

1. `log_vwrite` in `src/log.rs` still formats a local C-owned message with
   `vasprintf`, passes its first-NUL view synchronously to `stravis`, and frees
   it on success and encoding failure. A local `CString` can remove that pair
   after auditing `%c` inputs and preserving its failure behavior. The
   `window_copy_mode_data.backing` screen is a larger owner whose borrowed
   pointer is invalidated on refresh. The redraw scene's line array has
   self-referential intrusive list tails and needs its own alias audit.
2. The remaining address-based registries, other UI tags, and session/winlink
   graph require separate migrations. The typed mode-tree key permits further
   semantic tags, but each mode still needs its own identity and alias audit.
   `window_client` has no existing guaranteed unique semantic key: names and
   PIDs can repeat, and creation timestamps are not unique by contract. Its
   pointer tag must wait for a client owner/observer migration; a new tag-only
   generated ID would violate the agreed type policy.
3. The only direct `xvasprintf` production caller outside the `xmalloc`
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
   found that puts an interior NUL in any `xvasprintf` output. The related
   `format_find` transforms also return C-owned strings to `format_replace`; local
   `_cstring` conversions would add copies. Revisit these paths when their
   callback/value return contracts can change.
4. `cmd_save_buffer_exec` now borrows its static detached `show-buffer` path.
   Changing only its formatted `save-buffer` path to `CString` would add a
   copy solely to replace the C-owned `format_single_from_target` result.
   The expansion output now has a local owner, but its exported result still
   crosses the C-owned return boundary. Remaining `xstrndup` callers return
   or transfer C-owned strings.

OSC 52 decode output transfers directly into `paste_add`, which retains its C
allocation until `paste_free`; a local Vec would add a copy without removing
the lifetime. The existing clipboard-reply E2E covers decoded `A\0B` and a
first NUL in encoded input.

`set-buffer`'s payload is a less useful local target: `paste_set` retains the
libc allocation on success and leaves it with the caller on error, so a local
`Vec` alone would add an allocation and copy.

Historical validation for increments 15–225 follows. Newer validation is
recorded in increments 226–270 above, with increment 228 explicitly retracted.
The remaining address-based registries and UI tags above are separate
migration candidates. Each retained increment has its own local commit; none
was pushed.
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
After increments 130–132 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, prompt-mutable, prompt-completion, prompt-paste, sorted-window-pane,
and format-loop CLI checks, staticlib symbol absence, and `git diff --check`
passed on main. Changed-file rustfmt reports one import-layout difference in
`sort.rs`; checking its exact pre-increment `3f35a0a` version reports the
same site. No combined sanitizer was run.
After increments 133–135 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, prompt-completion, prompt-mutable, prompt-paste, customize-option,
send-keys, and key CLI checks, and `git diff --check` passed on main.
Changed-file rustfmt reports two import-layout differences in `prompt.rs`;
checking its exact pre-increment `0c279d5` version reports the same sites.
The other changed Rust files pass rustfmt. No combined sanitizer was run.
After increments 136–138 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, prompt-mutable, prompt-input-format, prompt-completion, prompt-paste,
and format-loop CLI checks, and `git diff --check` passed on main.
Changed-file rustfmt reports only two import-layout differences in
`prompt.rs`; checking its exact pre-increment `1c3d196` version reports the
same sites. `src/text/utf8.rs` passes rustfmt. No combined sanitizer was run.
After increments 139–141 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, prompt-mutable, prompt-input-format, prompt-completion, and
prompt-paste CLI checks, and `git diff --check` passed on main. Source search
finds 26 `prompt_done_with_history` calls and only the `PROMPT_SINGLE`
production `utf8_tocstr` call in `prompt.rs`. Changed-file rustfmt reports
two import-layout differences already present in the exact pre-increment
`5188b96` file. No combined sanitizer was run.
After increments 142–144 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, prompt-mutable, prompt-input-format, prompt-completion, prompt-paste,
and copy-jump CLI checks, and `git diff --check` passed on main. The workspace
suite includes `osc8_hyperlink_id` and `copy_regex_cells`; there is no
separate OSC 8 CLI script. Changed-file rustfmt passed for `grid/core.rs`
and `window_copy.rs`. `prompt.rs` retains the two pre-existing import-layout
differences documented above. No combined sanitizer was run.
After increments 145–147 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, server-message and pane-private-mode CLI checks, a detached `-vv`
server log-file check, changed-file rustfmt, and `git diff --check` passed on
main. The log check created one nonempty `tmux-server-<pid>.log` in a private
temporary directory and shut down the server. No combined sanitizer was run.
After increments 148–150 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, attached-client-list, Linux proc lookup, and linked-session-list CLI
checks, changed-file rustfmt, Python syntax checks for the three new scripts,
and `git diff --check` passed on main. No combined sanitizer was run.
After increments 151–153 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, active-session-list and active-client-list CLI checks, changed-file
rustfmt, and `git diff --check` passed on main. A private-socket scenario
created a split pane and second window and entered `choose-tree`; the
attached-client entry/exit check was run in increment 153's isolated
validation. No combined sanitizer was run.
After increments 154–156 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, session-group-list, grouped-attached-list, and event-payload hook CLI
checks, changed-file rustfmt, and `git diff --check` passed on main. No
combined sanitizer was run.
After increments 157–159 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, pane-tabs and start-command-list CLI checks, changed-file rustfmt, and
`git diff --check` passed on main. The workspace suite includes the new OSC
52 attached-PTY test with text and binary clipboard data. No combined
sanitizer was run.
After increments 160–162 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, start-command-list CLI checks, changed-file rustfmt, and
`git diff --check` passed on main. The isolated parser log-prefix and prompt
history-file E2E scenarios also passed. No combined sanitizer was run.
After increments 163–165 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, prompt-completion and display-panes-command CLI checks, the focused
`run_shell_partial_line` integration test, changed-file rustfmt for
`run_shell.rs` and `window_panes.rs`, and `git diff --check` passed on main.
`prompt.rs` still has the two pre-existing import-layout differences.
No combined sanitizer was run.
After increments 166–168 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, format-search, hook-monitor-append, and window-switch-target CLI
checks, changed-file rustfmt for `set_option.rs` and `window_switch.rs`, and
`git diff --check` passed on main. `window.rs` still has its pre-existing
mouse-import layout difference; the changed pane-search code follows
rustfmt. No combined sanitizer was run.
After increments 169–171 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, menu-owner, capture-pane-grid-cell, and mode-tree-menu-title CLI
checks, changed-file rustfmt, and `git diff --check` passed on main. The
capture-pane check compared complete output against the pinned baseline
binary for both OSC 8 cases. No combined sanitizer was run.
After increments 172–174 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, mode-tree-preview-label, customize-array-name, and window-tree-prompt
CLI checks, changed-file rustfmt, and `git diff --check` passed on main.
No combined sanitizer was run.
After increments 175–177 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, mode-tree-row, customize-key-table-title, and capture-pane-grid-cell
CLI checks, changed-file rustfmt, and `git diff --check` passed on main.
The mode-tree-row and capture-pane checks compared against the pinned
baseline binary. No combined sanitizer was run.
After increments 178–180 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, pane login-record, socket-label-base, and customize-key-detail CLI
checks, changed-file rustfmt, Python syntax checks, and `git diff --check`
passed on main. The socket-label-base and customize-key-detail checks
compared with the pinned baseline binary; the login-record check confirmed
matching pane IDs and label shape on both binaries. No combined sanitizer
was run.
After increments 181–183 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, systemd-pane-description, spawn-editor-command, and
customize-environment CLI checks, changed-file rustfmt, Python syntax checks,
and `git diff --check` passed on main. All three CLI checks also passed with
the pinned baseline binary. No combined sanitizer was run.
After increments 184–186 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, systemd-pane-description with UUID scope-name assertion,
customize-new-key-prompt, and customize-environment-prompt CLI checks,
changed-file rustfmt, Python syntax checks, and `git diff --check` passed on
main. All three CLI checks also passed with the pinned baseline binary.
No combined sanitizer was run.
After increments 187–189 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, customize-new-user-prompt, customize-array-key-prompt, and
customize-set-option-prompt attached-client CLI checks, changed-file rustfmt,
Python syntax checks, and `git diff --check` passed on main. All three CLI
checks also passed with the pinned baseline binary. No combined sanitizer
was run.
After increments 190–192 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, customize-set-key-prompt, customize-unset-prompt, and
customize-reset-prompt attached-client CLI checks, changed-file rustfmt,
Python syntax checks, and `git diff --check` passed on main. All three CLI
checks also passed with the pinned baseline binary. No combined sanitizer
was run.
After increments 193–195 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, customize-scope-text, window-tree-target, and expand-path-owner CLI
checks, plus existing environment-prompt, set-option-prompt, and socket-label
checks, changed-file rustfmt, Python syntax checks, and `git diff --check`
passed on main. All six CLI checks also passed with the pinned baseline
binary. No combined sanitizer was run.
After increments 196–198 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, lockfile-owner, hook-monitor-string, and shell-argv0-owner CLI checks,
changed-file rustfmt, Python syntax checks, and `git diff --check` passed on
main. All three CLI checks also passed with the pinned baseline binary. No
combined sanitizer was run.
After increments 199–201 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, prompt-completion, capture-pane-grid-cell, and socket-label-base CLI
checks, Python syntax checks, and `git diff --check` passed on main. The
prompt-completion check passed separately on the pinned baseline. Capture
output matched byte for byte; socket-label output matched after normalizing
the private socket paths. Rustfmt passed for `capture_pane.rs` and `tmux.rs`.
`prompt.rs` has the same two pre-existing rustfmt differences at lines 1 and
62 as its pre-increment-199 version; rustfmt reports no differences in the
changed code. No combined
sanitizer was run.
After increments 202–204 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, option-value-owner, layout, and argument-escape-owner CLI checks,
changed-file rustfmt, Python syntax checks, and `git diff --check` passed on
main. Option-value and argument-escape output matched the pinned baseline
byte for byte; the layout check passed separately on candidate and baseline.
No combined sanitizer was run.
After increments 205–207 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, set-buffer-name, command-list-print, and format-quote CLI checks,
changed-file rustfmt, Python syntax checks, and `git diff --check` passed on
main. All three CLI checks matched the pinned baseline byte for byte. No
combined sanitizer was run.
After increments 208–210 were integrated, `cargo clean -p hmux-rt` followed
by `RUST_TEST_THREADS=1 cargo test --workspace --quiet`, the library/binary
build, command-print, terminal-override, and window-copy-vadd CLI checks,
changed-file rustfmt, Python syntax checks, and `git diff --check` passed on
main. All three CLI checks matched the pinned baseline byte for byte; the
window-copy check includes the attached-client `A\0B` first-NUL scenario.
No combined sanitizer was run.
