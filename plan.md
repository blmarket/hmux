# Replace libevent with hmux-rt

## Goal and scope

Build and run hmux2 with the copied `hmux-rt` runtime and no libevent link,
header, package, or runtime dependency. Preserve client/server protocol, terminal
behavior, command ordering, timers, signal handling, jobs, and control mode.
This document plans the migration; only the runtime copy/workspace setup is
implemented so far. No commits or publication are part of this planning task.

## Progress

- [x] Copy `~/proj/hmux/hmux/hmux-rt` to `./hmux-rt` unchanged.
- [x] Add the workspace member and required workspace lint settings; update lockfile.
- [x] Inspect libevent declarations, callers, shared layouts, and upstream adapters.
- [x] Validate the copied runtime and write this migration plan.
- [ ] 1. Record baseline behavior and introduce host adapter boundaries.
- [ ] 2. Replace evbuffer storage and operations.
- [ ] 3. Implement hmux-rt event and stream adapters with contract tests.
- [ ] 4. Switch process loops, signals, timers, and all I/O to hmux-rt.
- [ ] 5. Remove the libevent ABI/dependency and validate a clean build.

Each implementation step must leave a working build. Record changed files,
commands, outcomes, and outstanding problems in the execution log before marking
it complete. Resume from the first unfinished step.

## Source and design references

The copied runtime comes from the local source tree at repository HEAD
`3ce9b8f72b1afaa0ef87373d266d7016abc9facf` (runtime subtree clean when inspected).
Its Rust source, README, example, and manifest are copied verbatim. The root
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

Those adapters depend on that project's process owner and types. Port only the
needed behavior into this project's module tree. Copying hmux-rt alone does not
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

Use host-side `ByteBuffer`, timer/watch handles, a buffered-stream registry, and
one runtime owner. Keep business callbacks synchronous initially. It is not
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

### 2. Replace byte buffers

Implement/test `ByteBuffer` independently, then migrate standalone buffers and
stream-facing buffer APIs. Until streams switch backend, use explicit adapters
or copies at the libevent boundary; never pass Rust buffer pointers to C evbuffer
functions. Update callback signatures (notably file/load/save/source paths),
formatting, line readers, and buffer destruction. Land coherent caller groups.

Validation: append/drain/pullup equivalents, binary data, split line endings,
empty/partial lines, large formatting, transfer/drain, short reads/writes and
`EAGAIN`. Run format, argument, file and control-related regressions.

### 3. Implement and test the runtime backend

Implement host turns, timer/watch registry, deferred queue, signal tasks and
buffered streams against hmux-rt without selecting them for production yet.
Keep libevent as the active backend until the new backend can drive every
source; avoid two independently blocking event loops. Test adapters with isolated
runtime instances and socketpairs/PTYS, taking behavioral expectations from the
baseline and reference project's adapter tests.

Required cases: rearm/cancel/self-delete, stale callback suppression, deferred
ordering, wake after append/re-enable, read watermarks, partial writes, EOF/error,
regular files, large transfers exceeding budgets, and fd/task cleanup. Tests
must verify behavior, not just mirror the implementation.

### 4. Switch all sources and process lifecycle

Select the hmux-rt backend as one coherent increment once its adapters pass.
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

For changed hmux2 Rust files run `rustfmt --edition 2021 --config
skip_children=true <changed-files>` and the same command with `--check`; avoid
formatting the entire translated tree. Record existing lint failures separately
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

`scripts/check_ffi_exports.py` currently requires the absent
`docs/required-exports.tsv`. Before relying on it, restore the authoritative
export list or capture and review the baseline archive exports; record this
existing validation blocker rather than silently skipping export coverage.
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
