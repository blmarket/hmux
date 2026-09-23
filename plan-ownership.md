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

### Next candidates

The later layout-equivalence cleanup removed the detached
`LayoutDescription` API and its API-only tests. `layout_parse` again builds
`layout_cell` and `layout_parse_ctx` directly, as in tmux, while the
`LayoutString` serializer owner remains. `cargo test --workspace`, the binary
build, and `scripts/layout_cli_checks.py` passed; the same CLI script also
passed with the pinned tmux binary, including an ignored `I` field with a
non-string value.

1. `mode_tree_display_menu` in `src/mode_tree.rs` creates a temporary title
   copied by `menu_create`, then frees it after adding items. A local
   byte-preserving `CString` can own that title across the synchronous call.
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

Current validation is recorded in increments 15–162. The remaining
address-based registries and UI tags above are separate migration candidates.
Each of increments 15–162 has its own local commit; none was pushed.
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
