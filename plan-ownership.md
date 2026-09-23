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

1. `client.cwd` has one identify branch with validated-path, home, and root
   fallback choices and a single free on client loss. It is a candidate for
   `ClientOwner` `CString` ownership, but the many synchronous consumers of
   `server_client_get_cwd` need an alias audit first. `client.term_type` is a
   separate nearby candidate; its one writer is in `tty_keys` and requires a
   shared setter.
2. Disconnected file-reading clients can leave a waiting command-queue item.
   Skipped terminal callbacks for `source-file` and pane stdin also retain
   callback data and client references. Releasing those alone can reach
   `cmdq_free` with a nonempty queue and abort, so queue cancellation needs
   wait-owner detach hooks, callback-data destruction, and queue draining as
   one coordinated boundary. A closed file may still invoke its normal
   callback after client loss. Terminal event scheduling is now idempotent;
   further progress callbacks and terminal error ordering still need audit.
3. Exported `fuzzy_match`, `args_from_vector`, and `monitor_parse` retain
   C-owned output contracts for external callers; no in-tree production caller
   uses their raw-output paths now. `ibufq_new`/`ibufq_free` are a small
   standalone allocation pair, but have no in-tree production caller. The
   exported contracts are deferred while live client fields remain. Revisit
   this ranking after each completed boundary.
4. The remaining address-based registries, other UI tags, and session/winlink
   graph require separate migrations. The typed mode-tree key permits further
   semantic tags, but each mode still needs its own identity and alias audit.
   `window_client` has no existing guaranteed unique semantic key: names and
   PIDs can repeat, and creation timestamps are not unique by contract. Its
   pointer tag must wait for a client owner/observer migration; a new tag-only
   generated ID would violate the agreed type policy.
5. The only direct `xvasprintf` production caller outside the `xmalloc`
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
6. `cmd_save_buffer_exec` now borrows its static detached `show-buffer` path.
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
- The old OSC 52 decoded allocation transfers to paste_add until paste_free. set-buffer data transfers to paste_set on success and remains with its caller on error. A local Vec alone would add a copy without retiring either transfer.

## Validation history and known test conditions

- Per-increment validation is in the corresponding commits and the archived detailed log. The current normal gate is serialized workspace tests, a binary build, relevant focused CLI or API checks, changed-file rustfmt with edition 2021, and git diff --check.
- A cached hmux-rt test binary once retained a deleted worktree path; cargo clean -p hmux-rt corrected it. Timestamp-based PTY tests have collided under parallel runs, so use RUST_TEST_THREADS=1 for the full suite.
- Some generated translation files have pre-existing rustfmt import-order differences. Compare a formatting complaint with the pre-migration file before treating it as caused by a boundary change.
- The pinned baseline customize-mode subprocess timed out during increment 365; candidate-only customize mutation checks passed and the spawned baseline test processes were terminated. No combined sanitizer run is recorded for the recent increments.
