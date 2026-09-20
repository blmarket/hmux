# Replace libevent with hmux-rt

## Goal and scope

Build and run hmux2 with the copied `hmux-rt` runtime and no libevent link,
header, package, or runtime dependency. Preserve client/server protocol, terminal
behavior, command ordering, timers, signal handling, jobs, and control mode.
This is the execution plan. The runtime extensions, application switch, lifecycle fixes, and dependency
removal are implemented. The production backend is hmux-rt. No commits or publication are part of this task.

## Progress

- [x] Copy `~/proj/hmux/hmux/hmux-rt` to `./hmux-rt` unchanged.
- [x] Add the workspace member and required workspace lint settings; update lockfile.
- [x] Inspect libevent declarations, callers, shared layouts, and upstream adapters.
- [x] Validate the copied runtime and write this migration plan.
- [x] 1. Record baseline behavior and introduce host adapter boundaries; audit ownership.
- [x] 2. Extend hmux-rt with reusable buffers and wake/select helpers.
- [x] 3. Extend hmux-rt with registration and buffered-stream adapters; test contracts.
- [x] 4. Switch process loops, signals, timers, and all I/O to hmux-rt.
- [x] 5. Remove the libevent ABI/dependency and validate a clean build.

Each implementation step must leave a working build. Record changed files,
commands, outcomes, and outstanding problems in the execution log before marking
it complete. Resume from the first unfinished step.

## Revised strategy: extend hmux-rt, minimize application churn

User direction on 2026-09-20: consider extending hmux-rt with the reusable
parts of the reference project's `src/reactor/` so the application migration
requires fewer changes. Adopt that strategy. The original runtime copy is the
starting point, not an immutable vendor snapshot.

`hmux-rt` replaces event scheduling but does not by itself replace evbuffer or
bufferevent. Move reusable buffer, wake, registration, and stream behavior into
that crate. Keep hmux2's `src/reactor/` as a thin compatibility facade so the
530 translated call sites can keep their existing operation names and mostly
unchanged arguments. These are ordinary Rust functions, not exported replacement
libevent symbols. Do not reproduce libevent's C struct layout in hmux-rt.

| Capability | Owner after migration | Boundary |
| --- | --- | --- |
| ByteBuffer and line policies | hmux-rt | Safe slices, append/drain/transfer, typed LineEnding, bounded read/write |
| Notify, yield, race/select helpers | hmux-rt | Wake stream tasks after append, re-enable, watermark change, or shutdown |
| Timers, I/O watches, signals, deferred work | hmux-rt | Runtime-owned registrations with zero-invalid IDs, cancellation, pending/deadline queries, generations |
| Buffered stream registry | hmux-rt | Owns buffers/tasks; supports watermarks, bounded I/O, callback thresholds, EOF/errors, and explicit release |
| Synchronous translated callbacks | hmux2 facade | Converts Rust notifications into existing C callback signatures and flags |
| C printf/vprintf and malloc-returned lines | hmux2 facade | Preserves varargs and allocation contracts; does not enter generic runtime API |
| Process owner, proc_loop, fork/exec, logging | hmux2 | Owns TaskRuntime and registries; invokes explicit runtime rebuild/release APIs |

Use the reference reactor implementation as a behavior guide. Its process-global
`server_state`, tmux types, raw callback owners, and business logic must not enter
hmux-rt. Make the reusable registries explicit objects driven by a TaskHandle,
not a singleton. The facade supplies process-local ownership. Runtime-owned
callbacks must be invoked without registry borrows and invalidated before raw
application owners can be freed.

The intended reusable surface is ByteBuffer/LineEnding, wake helpers,
registration IDs and registries, and buffered streams. Exact public types for
registrations and streams are to be finalized with contract tests before wiring
application callers. Preserve the existing TaskRuntime/AsyncFd APIs for users
that only need the lower-level runtime.

Avoid the original plan's mixed Rust-buffer/libevent-stream bridge: it adds
conversion code and a temporary ownership model. Test Rust buffers independently
inside hmux-rt, then switch all storage and stream operations through the facade
together once the replacement backend is complete. Until then, the facade uses
libevent. This deliberately changes the ordering of storage migration.

Smaller call-site changes do not eliminate necessary owner layout, fork,
signal-disposition, and cleanup changes. Replace embedded C event structs with
host handles at the production switch, and update their layout tests honestly.
Keep protocol and serialization fixtures fixed.

## Source and design references

The copied runtime comes from the local source tree at repository HEAD
`3ce9b8f72b1afaa0ef87373d266d7016abc9facf` (runtime subtree clean when inspected).
Its Rust source, README, example, and manifest were initially copied verbatim;
the revised strategy intentionally extends the local runtime copy. The root
workspace supplies its inherited Clippy settings. It requires nightly Rust
(`local_waker`) and edition 2024; hmux2 can remain edition 2021.

Read these source-project files as implementation references, not drop-in modules:

- `~/proj/hmux/hmux/src/reactor/runtime.rs`: host turns, process identity,
  rebuilding after fork, deferred callbacks, and shutdown.
- `~/proj/hmux/hmux/src/reactor/registry.rs`: timer/I/O/signal handles and tasks.
- `~/proj/hmux/hmux/src/reactor/stream.rs`: buffered streams, watermarks,
  callbacks, I/O budgets, and readiness.
- `~/proj/hmux/hmux/src/reactor/buffer.rs` and `notify.rs`: byte buffers and
  application-side wake/select helpers.
- `~/proj/hmux/hmux/src/tests/test_reactor_registry.rs` and
  `test_reactor_stream.rs`: adapter behavior tests.

Those adapters depend on that project's process owner and types. Port reusable behavior into hmux-rt and keep application-specific glue in
hmux2's module tree. Copying hmux-rt alone does not
supply an evbuffer or bufferevent replacement.

## Inventory and replacement mapping

`src/ffi/libevent.rs` centralizes foreign functions; `src/shared/event.rs`
centralizes the opaque types, C event/bufferevent layouts, callbacks, and flags.
The appendix lists actual call expressions, including child modules that inherit
imports. Imports alone would miss `src/format/{callbacks,expression,jobs}.rs`.
The application's `events.rs`, hook events, and `event_payload_*` APIs are not
libevent; retain them (but migrate their buffer serialization).

| Area | Current locations | Replacement |
| --- | --- | --- |
| Base and dispatch | `osdep_linux.rs`, `tmux.rs`, `client.rs`, `server.rs`, `proc.rs` | Process-owned `TaskRuntime`; dispatch/poll turns and explicit child rebuild |
| Peer messaging and accept | `proc.rs`, `server.rs` | `AsyncFd` watches around existing imsg/accept operations; preserve descriptor passing and write queue semantics |
| Signals | `proc.rs`, process teardown/fork callers | `Signals` tasks, explicit registration lifetime and child signal reset |
| Timers and pending state | alerts, monitor, names, status, session, server/client, input, screen writing, tty/keys, window/copy/clock/panes, format expression, run-shell | Host timer handles with initialization, armed state, generation, deadline, cancel/rearm operations backed by `sleep_until` |
| Deferred callbacks | `alerts.rs`, `session.rs`, `server_client.rs`, `file.rs`, `cmd/entries/run_shell.rs` | Cancellable host deferred queue with explicit next-turn semantics for `event_once`/`event_active` |
| Raw tty I/O | `tty.rs`, `tty_keys.rs` | Byte buffers and readiness tasks for terminal input/output; retain escape, clipboard, start, and throttle timers |
| Buffered streams | `window.rs`, `job.rs`, `file.rs`, `control.rs`, `cmd/entries/pipe_pane.rs` | Host buffered-stream adapter driven by hmux-rt |
| Stream producers/consumers and cleanup | input, input_keys, window/copy, popup, paste-buffer, server/client, server_fn, spawn, format/jobs, run-shell | Stream methods for input/output access, writes, enable/disable, and release |
| Standalone buffers | format callbacks/expression, JSON, events_payload, command queue, file, input, tty, capture/display/load/save/source-buffer commands | Rust byte buffer, line parsing and formatting helpers |
| Diagnostics | `log.rs`, `proc.rs` | Remove event log callback and version/method queries; report hmux-rt and host errors |

Shared owners in `src/shared/{client,control,input,job,pane,process,session,status,
tty,monitor,window}.rs` embed event structs or buffer/stream pointers. Local
owners also exist (for example run-shell and window modes). Replace these fields,
explicit event literals, allocation/initialization, and destruction together.
Generated layout/re-export tests and fixtures intentionally encode the old ABI;
they must change with each affected owner, not merely be bypassed.

## Adapter contracts and migration hazards

Use hmux-rt `ByteBuffer`, registration and stream registries, thin host handles,
and one application runtime owner. Keep business callbacks synchronous initially. It is not
necessary to rewrite every translated function as async to remove libevent.
Expose operations from an ordinary Rust module (suggested `src/reactor/`), not
new external libevent symbols. Transitional wrappers may keep call sites small,
but the final code must not retain a fake dependency on libevent's struct ABI.

- **Runtime ownership:** only the host dispatches/polls. Callbacks access a
  separate control/registry handle; never hold a mutable registry borrow while
  invoking a callback that may rearm, delete, or free itself. Bound dispatch and
  I/O work, and poll with zero timeout when runnable work remains so I/O and
  timers are not starved. Preserve the `proc_loop` callback and exit checks.
- **Readiness:** `AsyncFd::new` must run inside a task. It duplicates the supplied
  descriptor; dropping it schedules release. Readiness is edge-style: drain to
  `EAGAIN`, retry `EINTR`, and distinguish EOF/errors. If a work budget or high
  watermark stops a drain early, remember that readiness and resume explicitly;
  waiting for another edge can hang. Handle regular files (runtime has an
  always-ready fallback), PTY hangup/EIO, and partial writes.
- **Wakeups/backpressure:** queued output, re-enabling reads, changed watermarks,
  and shutdown must wake a parked stream task. Do not wait on writable readiness
  while output is empty. `hmux-rt` does not export a general Notify/select API;
  implement/port small host helpers. Preserve read/write callback thresholds,
  input limits, output drain callbacks, and error/EOF flags, including numeric
  flag checks at translated call sites.
- **Cancellation/lifetime:** dropping a `JoinHandle` detaches; `cancel` is a
  request processed on dispatch. Mark registrations invalid synchronously and
  use IDs/generations so queued tasks cannot call a freed raw owner. Keep
  callback arguments alive until callbacks cannot run. Drain cancellation and
  release duplicate descriptors before asserting complete close/EOF, reusing
  owners, or completing fork/teardown. Never place a Rust owner requiring Drop
  directly into a structure managed only by `calloc`, `memcpy`, or `free`.
  Use zero-valued IDs with registry ownership initially, or explicitly convert
  allocation and destruction to Rust ownership.
- **Timers:** distinguish configured from pending; preserve event_pending
  queries, including any requested deadline output. Store monotonic deadlines
  rather than restarting elapsed delays on every loop turn. Reset/rearm must
  invalidate the old task; zero delay/activation remains deferred. Preserve
  recurring timers' existing callback rearm policy and destroy-time cancellation.
- **Signals:** `Signals::recv()` returns `()`, not the signal number. Use one
  source/task per signal or add a tested signal-aware adapter; a mixed source
  loses identity. Preserve ignored signals and SIGCHLD reap loops, since signals
  can coalesce. Drop handlers and restore child dispositions before exec.
- **Fork:** replace `event_reinit` with a fresh runtime, descriptor registrations,
  signal sources, and task handles in the child. Audit inherited pending watches
  and timers, both fork/no-fork server startup paths, job children, and detach.
  Do not poll inherited mio state or retain handles to the old runtime.
- **Buffers:** a contiguous buffer with a consumed prefix can replace pullup,
  drain, append, and length. Preserve binary/NUL data, incomplete lines, LF versus
  legacy readline behavior, and allocation/free ownership of returned strings.
  Slices/raw pointers must not outlive mutation. Replace printf/vprintf calls
  with typed formatting where practical or a bounded libc formatting bridge;
  Rust format syntax is not a substitute for existing C varargs formats.
  `bufferevent_write_buffer` transfers/drains its source. Read/write helpers
  must preserve return counts and error semantics used by their callers.

## Ordered implementation

### 1. Baseline and adapter boundaries

Add `hmux-rt = { path = "hmux-rt" }` to hmux2 dependencies when integration begins.
Capture CLI transcripts using the current binary before changing behavior. Add
host buffer/timer/watch/stream interfaces and migrate direct bufferevent field
access (`input`, `output`, etc.) to accessors while the implementation still uses
libevent. This provides a compiling boundary for the backend switch. Inventory
all allocations and releases for each shared owner and its exported signatures.

Validation: baseline build, existing test suite and CLI checks below; compile
all targets after each interface/owner change. Keep protocol/serialization
fixtures fixed; document only intentional internal layout changes.

### 2. Extend hmux-rt with buffers and wake helpers

Implement/test ByteBuffer inside hmux-rt and export safe operations: contiguous
slice access, append/drain/transfer, typed line-ending policies, and bounded I/O
that preserves short-read/write and error results. Keep binary data and partial
lines intact. There is no C allocation or varargs ABI in this crate.

Port/adapt Notify, yield and select/race helpers from the reference reactor.
Test wake-before-wait, coalesced notifications, cancellation of a pending wait,
and competing readiness/control wakes. Document single-threaded use and waiter
limits. Add no second event loop and no daemon global-state dependency.

Production buffer calls still go to libevent through the facade. C formatting
and malloc-returned line adapters are implemented and tested on the replacement
side before selection; do not introduce a mixed-storage production bridge.

Validation: binary append/drain and compaction, split terminators and legacy
readline distinctions, empty/partial lines, large formatting through the facade,
transfer/drain, short reads/writes, EOF, EINTR and EAGAIN. Strict runtime tests,
format and Clippy must pass; existing runtime consumers must still compile.

### 3. Extend hmux-rt with registrations and buffered streams

Adapt the reference registry and stream behavior into explicit reusable owners
inside hmux-rt. They depend on TaskHandle, Rust closures and runtime types, not
hmux2 modules. Registrations track configured versus armed state, monotonic
deadlines, generation, cancellation and explicit release. Signals preserve
identity. Deferred work has cancellable IDs and documented next-turn ordering.

Streams own ByteBuffers, task handles, enabled state and watermarks. Expose
buffer access without requiring business callbacks to become async. Appending
output and re-enabling reads wake parked tasks. Preserve readiness when a budget
or high watermark stops draining early. Distinguish read/write EOF/error and
invoke callbacks outside mutable registry borrows.

The hmux2 facade adapts existing operation names and callbacks to these APIs.
Keep raw-owner lifetime obligations and C formatting there. Implement host turns,
shutdown and fork rebuild using explicit runtime/registry lifecycle methods.
No production backend switch until every source is supported; libevent remains
the sole active loop during development.

Required tests: rearm/cancel/self-delete, stale callback suppression, deferred
ordering/cancellation, wake after append/re-enable, read watermarks, partial writes,
EOF/error, regular files, PTY EIO/hangup, transfers exceeding budgets, and fd/task
cleanup. Use isolated registries/runtimes and socketpairs/PTYS. Test fork rebuild
and disposal of duplicated descriptors as lifecycle contracts, not only source
shape. Port expectations from reference tests without daemon dependencies.

### 4. Switch all sources and process lifecycle

Select the hmux-rt backend as one coherent increment once its adapters pass.
Switch buffer storage and stream operations through the facade together.
Replace initialization and `proc_loop`, accept and peer watchers, signal setup,
all timers/deferred work, tty I/O, pane/job/pipe/file/control streams, and their
cleanup paths. Rebuild after fork; preserve loop callback scheduling and client
exit draining. If this needs multiple increments, use the interface boundary
from step 1 and keep the old backend selected until the complete switch works.

Replace embedded libevent objects with host handles and remove direct layout
initializers. Update affected `tests/model_*.rs`, consolidation tests/fixtures,
shared event unit tests, re-exports, and FFI architecture/export expectations in
the same increments. Preserve real external ABI contracts; libevent's internal
layout is no longer a contract once no C consumer uses those structures.

Validation: all adapter tests and CLI regressions; exercise attach/detach,
resize/SIGWINCH, input escape timeout, status refresh, copy-mode repeat, run-shell
with delay/background output, format jobs, pipe-pane both directions, control
mode slow readers, file load/save, child reaping, and server shutdown. Include
forked startup and repeated create/destroy with fd counts returning to baseline.

### 5. Remove dependency and prove the result

Delete `src/ffi/libevent.rs` and its module registration, remove obsolete
`shared/event.rs` ABI declarations/re-exports, event logging/version calls,
and `event_core` from `build.rs`. Keep only application-defined flags/types
still needed by adapters. Add a source check preventing new libevent externs
or link directives; retain historical documentation only as documentation.

Audit Nix carefully: hmux's explicit package buildInputs already omit libevent,
but the development shell inherits `tmuxTarget.buildInputs` and includes a
reference tmux that depends on it. Give hmux2 an explicit dependency set and a
minimal validation environment. The optional reference tmux may keep libevent;
it must not be in hmux2's package closure or be needed to build/test hmux2.
Correct the flake's existing `meta.mainProgram = "hmux"` to `hmux2` when using
that package as the final validation target. Do not infer independence from a
successful build in the current broad development shell.

Validation: clean debug/release builds and tests in an environment without
libevent, source/symbol/dependency checks below, and full behavioral regression.

## Validation commands and completion criteria

Use the installed nightly toolchain (inspection used rustc 1.99.0-nightly,
2026-08-10). Current repository has no root Makefile; do not use the source
project's make targets. For each implementation increment:

```sh
cargo test --locked -p hmux-rt
cargo fmt -p hmux-rt -- --check
cargo clippy --locked -p hmux-rt --all-targets -- -D warnings
cargo build --locked -p hmux2
cargo test --locked -p hmux2 -- --test-threads=1
cargo clippy --locked -p hmux2 --all-targets
```

For new adapter files run `rustfmt --edition 2021 --config
skip_children=true <changed-files>` and the same command with `--check`.
For mechanical import/accessor edits in translated files, preserve surrounding
formatting to avoid unrelated formatter churn. Format newly written code and
record baseline formatting differences separately; do not reformat the tree. Record existing lint failures separately
and introduce none. If tests touch process globals and fail due to same-process
state, isolate them with `cargo nextest run -p hmux2` and record the reason.

Run the existing behavior checks with `HMUX_BINARY` pointing to the built binary:

```sh
python3 scripts/cli_regressions.py
python3 scripts/ffi_callbacks.py
python3 scripts/arguments_cli_checks.py
python3 scripts/environment_cli_checks.py
python3 scripts/format_display_message_checks.py
python3 scripts/key_cli_checks.py
python3 scripts/layout_cli_checks.py
python3 scripts/style_cli_checks.py
python3 scripts/check_ffi_exports.py
```

`scripts/check_ffi_exports.py` requires `docs/required-exports.tsv`, restored
from the authoritative repository history during execution. The other missing
audit fixtures were restored as well; export and architecture coverage now run.
The README also links to absent docs, so use checked-in source/tests as the
current architecture evidence.

Capture deterministic before/after transcripts for the first two scripts and
compare them. Add targeted runtime scenarios listed above where existing scripts
do not cover them. Validate staticlib as well as binary consumers.

Final independent build, after removing the dependency from source/build inputs:

```sh
CARGO_TARGET_DIR=target/no-libevent cargo build --locked --workspace --all-targets
CARGO_TARGET_DIR=target/no-libevent cargo build --locked -p hmux2 --release
CARGO_TARGET_DIR=target/no-libevent cargo test --locked --workspace -- --test-threads=1
readelf -d target/no-libevent/release/hmux2
ldd target/no-libevent/release/hmux2
nm -u target/no-libevent/release/hmux2
nm -u target/no-libevent/release/libhmux2.a
rg -n 'libevent|event_core|rustc-link-lib.*event' Cargo.toml Cargo.lock build.rs src flake.nix nix
nix build path:.#hmux
nix-store -qR ./result
```

Run the Cargo commands in a container or minimal Nix environment containing the
remaining native libraries but no libevent headers/libraries. `path:.` includes
the newly copied files even before a commit. Check outputs for no libevent
`DT_NEEDED` entry, transitive linked library, unresolved `event_*`, `evbuffer_*`
or `bufferevent_*` imports, or libevent in the hmux package closure. Application
symbols such as `event_payload_*` are not libevent imports. Source scans and
`ldd` alone cannot rule out static linkage, hence the clean environment and
archive symbol check. Audit any Nix reference-tool-only matches explicitly.

Complete only when all sources use hmux-rt, behavior tests pass, no callbacks
outlive owners, and the binary plus staticlib build without libevent present.

## Execution log

- 2026-09-20: copied hmux-rt verbatim, added workspace membership/inherited lints,
  and resolved its dependencies into Cargo.lock. No application backend changed.
- 2026-09-20: `cargo test -p hmux-rt`: 35 passed; doc tests passed (0 tests).

- 2026-09-20: runtime formatting check and strict all-target Clippy passed.
- 2026-09-20: `cargo build --locked -p hmux2` passed with 1,890 existing
  warnings. This baseline still links libevent; no claim of dependency removal.
- 2026-09-20: recursive comparison confirmed the copied runtime is identical
  to its source. Full application regression execution is deferred to migration.

- Execution: step 1 added the `src/reactor` host boundary, routed application
  imports through it, migrated stream input/output field access to accessors,
  added the hmux-rt dependency, and recorded fields in
  `docs/runtime-owner-audit.md`. Restored three authoritative test audit fixtures
  from `edc0f22^` (required exports, key enum declarations, mutable scratch).
- Baseline binary and deterministic CLI transcripts are under
  `target/migration/baseline`. `/tmp` is not executable in this environment;
  baseline capture was rerun successfully from the workspace. All eight behavior
  scripts passed. `cargo build --locked -p hmux2` and the full single-threaded
  application test suite passed after restoring the missing fixtures.
- `cargo test --locked -p hmux-rt`, runtime formatting, and strict runtime Clippy
  passed. Application all-target Clippy reports 30 pre-existing translated-code
  errors (`eq_op`, `while_immutable_condition`, `self_assignment`); output is in
  `/tmp/hmux-step1-clippy.log`. Formatter-only translated-tree churn was removed
  under the revised smaller-change strategy.
- Superseded experiment: mixed Rust standalone buffers and legacy stream buffers
  built but failed CLI checks with corrupt output, and the foreign-boundary test
  rejected duplicate host/foreign operation names. That bridge was removed when
  the user revised the strategy; it is not part of the active backend.
- Revised step 2 started: ByteBuffer lives in `hmux-rt/src/buffer.rs` and is
  re-exported by the thin hmux2 facade. Production buffers still use libevent.
  Wake helpers, registration/stream extensions, and production selection remain
  unfinished. Do not mark the migration or dependency removal complete.

- Revised checkpoint validation: `cargo test --locked -p hmux-rt` passed
  (39 tests); runtime formatting and strict all-target Clippy passed.
  `cargo build --locked -p hmux2` and the full single-threaded hmux2 test suite
  passed. Adapter formatting and `git diff --check` passed.
- Revised CLI checks: cli_regressions, ffi_callbacks, arguments, environment,
  key, layout, style and required-export checks passed. Both deterministic
  transcripts exactly match the saved baseline. Logs are under
  `target/migration/revised`.
- `format_display_message_checks.py` initially passed during baseline capture,
  but repeat runs failed on BOTH the saved baseline binary and current binary
  (format-job initial-response/barrier expectations). Record this as an existing
  reproducibility issue; do not claim uniformly green behavioral coverage.

- Implementation completed under the revised strategy. Added reusable hmux-rt
  Notify/select/yield helpers, explicit timer/watch/signal/deferred registries,
  buffered-stream registry, ByteBuffer fd I/O, and cancellation-only flushing.
  Registry callbacks execute outside mutable registry borrows; cancellations
  synchronously invalidate generations and retire entries. Read watermarks,
  write-drain thresholds, budget continuations, regular files and PTY hangup
  are covered by contract tests. Runtime tests: **64 passed**; strict all-target
  runtime Clippy and formatting passed.
- The hmux2 facade now supplies buffer formatting/malloc-line compatibility,
  zero-initialized event handles, synchronous callback adaptation, and process
  runtime ownership. Removed `src/ffi/libevent.rs`, its registration, legacy
  layouts/initializers, event logging, and the event_core link. Event storage is
  56 bytes (previously 128), stream facade handles 24 bytes (previously 392).
  Updated only affected internal owner layout fixtures; required C exports and
  protocol/serialization fixtures remain unchanged. Restored export coverage
  confirms all **1,496 required C symbols** in the release static library.
- Ownership audit: `docs/runtime-owner-audit.md` describes owner creation/release
  and exported-pointer contracts; `docs/runtime-lifetimes.tsv` indexes allocation,
  registration and destruction sites. Fixed a stale pointer in
  server_client_print by reacquiring it after an append. Fork children discard
  inherited runtime state without deregistering against the parent's epoll fd;
  server rebuild preserves active registrations/deadlines. Client and server
  exit paths explicitly shut down runtime registries.
- Expanded behavior checks in `scripts/runtime_cli_checks.py`: delayed and
  ordinary jobs, 1 MiB binary load/save, both pipe directions, slow control
  readers, controlling-PTY attach/resize/escape/detach, status refresh,
  copy-mode entry/exit, repeated window/job creation with fd counts returning
  to baseline, no unreaped child zombies, and non-fork startup/shutdown passed.
  CLI and FFI-callback transcripts match the saved baseline byte-for-byte.
- Fixed the pre-existing format test harness response offset: consume the
  control client's startup response before sending commands, then validate each
  command's own response. A pending format job preserves its literal prefix;
  an already-completed initial result is also valid. Eventual and cached-repeat
  expectations remain strict. Corrected checks pass on BOTH the saved libevent
  baseline and the migrated release binary.
- All nine behavioral scripts (the original eight plus runtime_cli_checks)
  passed against `target/no-libevent-final/release/hmux2`; logs are in
  `target/migration/final`. All 238 workspace tests passed with one test thread.
  Source/artifact dependency guard, adapter formatting and git diff whitespace
  checks passed. Application all-target Clippy still reports the same 30
  translated-code errors as baseline; the new adapter/runtime files introduce
  no Clippy diagnostics (`/tmp/hmux-final-clippy.log`).
- Independent build environment: `nix develop ...#minimal --ignore-environment
  --keep HOME`, CARGO_TARGET_DIR=target/no-libevent-final. Clean locked workspace
  all-target debug build, release build, and full workspace tests passed.
  Debug info was disabled for this debug build to reduce artifact size.
  Source, readelf, ldd and nm checks pass for release binary and staticlib.
  A C consumer was linked against the staticlib with only the remaining native
  libraries and successfully called getversion.
- Nix package build and its full workspace check phase passed. A source snapshot
  at `target/migration/nix-source` contains the working tree excluding .git,
  target and result, avoiding copying 18 GiB of build artifacts into the flake.
  Built with `nix build path:./target/migration/nix-source#hmux`; result is
  `target/migration/nix-result`. The snapshot's compiled sources match this
  implementation; subsequent changes are validation-script/docs comments only.
  Final derivation: `/nix/store/nsq2x3f2n05awnn8j5d4ykaxghfwvk8a-hmux-0.0.0.drv`.
  The output closure has **51 paths and no libevent**. Direct build/native-input
  closures also contain no libevent headers/libraries (saved closure audits in
  target/migration/final). Some transitive *derivation recipes* mention libevent,
  but no libevent output is in the actual build-input or package closures.
- The closure audit caught a transitive dependency from full systemd via
  libmicrohttpd/GnuTLS/Unbound. Switching hmux package and shell inputs to
  `systemdLibs` removed it. The optional reference tmux stays in the broad dev
  shell only; the minimal shell and hmux package do not require it. Corrected
  meta.mainProgram to hmux2 and enabled package tests. No commits or publication.

## Appendix: direct call inventory

Snapshot of call expressions on 2026-09-20: 530 calls in 39 files, using 33 of 33 declared functions. Counts exclude imports and declarations.

| File | Functions and call counts |
| --- | --- |
| `src/alerts.rs` | `event_add` × 1, `event_del` × 1, `event_initialized` × 1, `event_once` × 1, `event_set` × 1 |
| `src/cmd/entries/capture_pane.rs` | `evbuffer_get_length` × 1, `evbuffer_pullup` × 1 |
| `src/cmd/entries/display_message.rs` | `evbuffer_add_printf` × 1, `evbuffer_free` × 1, `evbuffer_new` × 1 |
| `src/cmd/entries/load_buffer.rs` | `evbuffer_get_length` × 1, `evbuffer_pullup` × 1 |
| `src/cmd/entries/paste_buffer.rs` | `bufferevent_write` × 6 |
| `src/cmd/entries/pipe_pane.rs` | `bufferevent_enable` × 2, `bufferevent_free` × 2, `bufferevent_new` × 1, `bufferevent_write` × 1, `evbuffer_drain` × 1, `evbuffer_get_length` × 1, `evbuffer_pullup` × 1 |
| `src/cmd/entries/run_shell.rs` | `evbuffer_get_length` × 1, `evbuffer_pullup` × 1, `evbuffer_readln` × 1, `event_active` × 1, `event_add` × 1, `event_del` × 1, `event_set` × 1 |
| `src/cmd/entries/save_buffer.rs` | `evbuffer_add` × 1, `evbuffer_free` × 1, `evbuffer_new` × 1 |
| `src/cmd/entries/source_file.rs` | `evbuffer_get_length` × 1, `evbuffer_pullup` × 1 |
| `src/cmd/queue.rs` | `evbuffer_add_vprintf` × 1, `evbuffer_free` × 1, `evbuffer_new` × 1 |
| `src/control.rs` | `bufferevent_disable` × 3, `bufferevent_enable` × 5, `bufferevent_free` × 2, `bufferevent_new` × 2, `bufferevent_setwatermark` × 1, `bufferevent_write` × 5, `bufferevent_write_buffer` × 1, `evbuffer_add` × 2, `evbuffer_add_printf` × 3, `evbuffer_free` × 3, `evbuffer_get_length` × 8, `evbuffer_new` × 2, `evbuffer_pullup` × 1, `evbuffer_read` × 1, `evbuffer_readln` × 2 |
| `src/events_payload.rs` | `evbuffer_add_printf` × 11, `evbuffer_free` × 2, `evbuffer_get_length` × 3, `evbuffer_new` × 2, `evbuffer_pullup` × 2 |
| `src/file.rs` | `bufferevent_enable` × 2, `bufferevent_free` × 3, `bufferevent_new` × 2, `bufferevent_write` × 1, `evbuffer_add` × 5, `evbuffer_add_vprintf` × 4, `evbuffer_drain` × 2, `evbuffer_free` × 1, `evbuffer_get_length` × 6, `evbuffer_new` × 2, `evbuffer_pullup` × 2, `event_once` × 2 |
| `src/format/callbacks.rs` | `evbuffer_add` × 7, `evbuffer_add_printf` × 7, `evbuffer_free` × 7, `evbuffer_get_length` × 14, `evbuffer_new` × 7, `evbuffer_pullup` × 7 |
| `src/format/expression.rs` | `evbuffer_add` × 8, `evbuffer_add_printf` × 1, `evbuffer_free` × 7, `evbuffer_get_length` × 8, `evbuffer_new` × 7, `evbuffer_pullup` × 7, `event_add` × 1, `event_initialized` × 1, `event_pending` × 1, `event_set` × 1 |
| `src/format/jobs.rs` | `evbuffer_get_length` × 1, `evbuffer_pullup` × 1, `evbuffer_readline` × 2 |
| `src/input.rs` | `bufferevent_write` × 6, `evbuffer_add` × 1, `evbuffer_drain` × 1, `evbuffer_free` × 1, `evbuffer_get_length` × 1, `evbuffer_new` × 1, `event_add` × 2, `event_del` × 6, `event_set` × 2 |
| `src/input_keys.rs` | `bufferevent_write` × 1 |
| `src/job.rs` | `bufferevent_disable` × 2, `bufferevent_enable` × 1, `bufferevent_free` × 2, `bufferevent_get_output` × 1, `bufferevent_new` × 1, `evbuffer_get_length` × 1 |
| `src/json.rs` | `evbuffer_add` × 7, `evbuffer_add_printf` × 3, `evbuffer_free` × 1, `evbuffer_get_length` × 1, `evbuffer_new` × 1, `evbuffer_pullup` × 1 |
| `src/log.rs` | `event_set_log_callback` × 2 |
| `src/monitor.rs` | `event_add` × 2, `event_del` × 2, `event_initialized` × 3, `event_pending` × 1, `event_set` × 1 |
| `src/names.rs` | `event_add` × 1, `event_del` × 1, `event_initialized` × 2, `event_pending` × 1, `event_set` × 1 |
| `src/osdep_linux.rs` | `event_init` × 1 |
| `src/popup.rs` | `bufferevent_write` × 1, `evbuffer_drain` × 1, `evbuffer_get_length` × 1, `evbuffer_pullup` × 1 |
| `src/proc.rs` | `event_add` × 9, `event_del` × 10, `event_get_method` × 1, `event_get_version` × 1, `event_loop` × 1, `event_set` × 10 |
| `src/screen_write.rs` | `event_add` × 2, `event_del` × 3, `event_initialized` × 4, `event_pending` × 1, `event_set` × 2 |
| `src/server.rs` | `event_add` × 4, `event_del` × 2, `event_initialized` × 1, `event_reinit` × 1, `event_set` × 3 |
| `src/server_client.rs` | `bufferevent_disable` × 1, `bufferevent_enable` × 1, `evbuffer_add` × 1, `evbuffer_drain` × 1, `evbuffer_get_length` × 6, `evbuffer_pullup` × 3, `evbuffer_readln` × 1, `event_add` × 6, `event_del` × 13, `event_initialized` × 6, `event_once` × 1, `event_pending` × 3, `event_set` × 6 |
| `src/server_fn.rs` | `bufferevent_free` × 2 |
| `src/session.rs` | `event_add` × 1, `event_del` × 2, `event_initialized` × 2, `event_once` × 1, `event_set` × 1 |
| `src/spawn.rs` | `bufferevent_free` × 1 |
| `src/status.rs` | `event_add` × 2, `event_del` × 4, `event_initialized` × 3, `event_set` × 2 |
| `src/tty.rs` | `evbuffer_add` × 1, `evbuffer_drain` × 1, `evbuffer_free` × 2, `evbuffer_get_length` × 4, `evbuffer_new` × 2, `evbuffer_read` × 1, `evbuffer_write` × 1, `event_add` × 7, `event_del` × 10, `event_initialized` × 1, `event_pending` × 1, `event_set` × 5 |
| `src/tty_keys.rs` | `evbuffer_drain` × 2, `evbuffer_get_length` × 1, `evbuffer_pullup` × 1, `event_add` × 1, `event_del` × 3, `event_initialized` × 3, `event_pending` × 1, `event_set` × 1 |
| `src/window.rs` | `bufferevent_disable` × 1, `bufferevent_enable` × 1, `bufferevent_free` × 2, `bufferevent_new` × 1, `bufferevent_write` × 7, `evbuffer_drain` × 1, `evbuffer_get_length` × 6, `evbuffer_pullup` × 2, `event_add` × 1, `event_del` × 9, `event_initialized` × 7, `event_set` × 1 |
| `src/window_clock.rs` | `event_add` × 1, `event_del` × 2, `event_set` × 1 |
| `src/window_copy.rs` | `bufferevent_write` × 1, `event_add` × 5, `event_del` × 6, `event_set` × 2 |
| `src/window_panes.rs` | `event_add` × 1, `event_del` × 1, `event_set` × 1 |

Declared but not directly called: none.
