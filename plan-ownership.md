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

### Next candidates

The later layout-equivalence cleanup removed the detached
`LayoutDescription` API and its API-only tests. `layout_parse` again builds
`layout_cell` and `layout_parse_ctx` directly, as in tmux, while the
`LayoutString` serializer owner remains. `cargo test --workspace`, the binary
build, and `scripts/layout_cli_checks.py` passed; the same CLI script also
passed with the pinned tmux binary, including an ignored `I` field with a
non-string value.

1. The nearby `PROMPT_SINGLE` branch is now the only production
   `utf8_tocstr((*pr).buffer)` call in `prompt.rs`; it still has a local
   allocation/free pair. Audit its callback outcome and any combined flags
   before using `utf8_tocstr_cstring` there.
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

Current validation is recorded in increments 15–141. The remaining
address-based registries and UI tags above are separate migration candidates.
Each of increments 15–141 has its own local commit; none was pushed.
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
