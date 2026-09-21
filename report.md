# Runtime redesign checkpoint

## Reviewed history

- `f9753df`: extends format-split validation; unrelated to runtime introduction.
- `edc0f22`: deletes old audit documents and logs; chosen pre-runtime baseline.
- `e0c7e02`: copies hmux-rt, adds workspace membership and migration plan
  (14 files, 3,021 insertions).
- `d66caf7`: extends the runtime and switches the application, layouts, tests,
  build/dependency configuration and audits together (89 files, 7,864 insertions,
  1,580 deletions).

The prior working tree also contained buffer-generalization, fork-reinitialization
and session-group teardown changes, plus an untracked report. Those were read and
preserved before restoring source.

## Why restart

The old public interface supplied task-based and callback-based timers, I/O and
signals simultaneously. It exposed executor dispatch/poll/block_on, cancellation
flushing, registry epochs, stream wake helpers, and three-stage fork recovery.
The compatibility facade added another ownership layer. A large part of the
review burden comes from these overlapping models, not just method count.

The replacement proposal in `plan.md` follows the user's clarification: support
hmux requirements through a local-waker async runtime. It chooses one task model,
one driver, and one cancellation lifecycle. Callback compatibility, buffers and
streams compose those primitives outside the runtime; they do not define another
public scheduler. The proposal does not claim that narrowing the crate makes
those requirements disappear. Local-waker-only futures are an explicit restriction,
and host reconstruction after fork remains necessary.

## Preservation and rollback

HEAD/history remain unchanged. Application/build/test tracked contents were
restored from `edc0f22ba4ee0d0458d950b524812309aab17765`. A subsequent review step
added the standalone trait definition. The current step implements that runtime
and moves byte storage to hmux-buffer with SegmentedBuf. There is still no hmux2
dependency/integration. Changes remain uncommitted.

The complete previous dirty state is saved as a stash, pinned independently at
`refs/backup/hmux-rt-before-redesign`:

```text
200d699a14b96c8110b095642ebb332f2e43b8b7
```

Its first parent retains the old HEAD; its working-tree snapshot includes tracked
edits; its third parent holds the old untracked report. Ignored build artifacts
were left in place. To inspect without applying anything:

```sh
git show refs/backup/hmux-rt-before-redesign:plan.md
git show refs/backup/hmux-rt-before-redesign^3:report.md
git diff refs/backup/hmux-rt-before-redesign^1 refs/backup/hmux-rt-before-redesign
```

The session-group fix was reverted with the rest, not lost. Treat it as a separate
bugfix candidate, with its own reproduction, rather than mixing it into the new
runtime definition. Prior conformance results describe the saved implementation;
they are not validation of this proposal.

## Rollback validation (before adding traits)

- Compared every tracked baseline file's Git blob hash against the restored
  filesystem: all match. The removed runtime files are absent from source.
- `cargo build --locked -p hmux2`: passed, with 1,890 existing warnings.
  Log: `/tmp/hmux2-pre-rt-build.log`.
- `python3 scripts/cli_regressions.py`: passed. Transcript:
  `/tmp/hmux2-pre-rt-cli.json`.

The baseline also retains the audit-fixture deletions made by `edc0f22`;
full suite repair belongs in its own change. At that checkpoint, no runtime implementation or performance validation had
been performed for the new design.

## Earlier trait review checkpoint

The user requested public Rust traits and a documentation-only libevent
replacement plan. At that checkpoint, the standalone hmux-rt workspace member defined Runtime,
Handle, Io, ReadyGuard, Signals, and Buffer. Task lifetime is an associated owner;
waits use Future. Buffer is independent of scheduling, supports borrowed chunks
and explicit pullup, and includes typed line policies. No backend, concrete buffer,
stream implementation, C facade, or application migration was added.

The signatures then lived in `hmux-rt/src/lib.rs` and `hmux-rt/src/buffer.rs`,
`hmux-rt/CONTRACT.md` for implementation obligations, and `hmux-rt/LIBEVENT.md`
for the per-API replacement mapping. The mapping was checked against all 33
function declarations in the current libevent FFI module.

Validation for this checkpoint:

- Crate all-target check, four generic usage doctests, formatting, strict Clippy,
  and rustdoc with warnings denied passed.
- `cargo check --locked -p hmux2` passed with its existing 1,890 warnings;
  log: `/tmp/hmux2-traits-check.log`.
- Compared baseline tracked files, excluding Cargo.toml/Cargo.lock: unchanged.
  Cargo metadata confirms hmux2 has no hmux-rt dependency and hmux-rt has no
  dependencies of its own. Root changes only add workspace membership/lock entry.

These checks validate declarations and example clients, not runtime behavior,
buffer semantics, or performance. Backend conformance remains future work.

Runtime API refinement: removed the separate cleanup operation. Dropping a task
owner synchronously destroys its future and owned resources; an executing poll
finishes before destruction. Updated trait, ownership contracts and migration
notes consistently. That checkpoint was a definition-only change.


## Standalone implementation checkpoint

Implemented the agreed runtime and buffer contracts in separate workspace crates.
Neither is a dependency of hmux2, and neither depends on the other. Application,
build, and test sources still match the pre-runtime baseline `edc0f22`; only the
workspace manifest and lockfile differ among files tracked in that baseline.

- `hmux-rt/src/lib.rs` defines Runtime, Handle, Io, ReadyGuard, and Signals.
  `LocalRuntime` implements budgeted local task polling, cancellation on owner
  drop, cached directional readiness, monotonic timers, next-turn suspension,
  coalescing signal delivery, and child-side reset after a single-threaded fork.
  There is no reap, stream, notification, join, or detach API. The runtime needs
  nightly Rust's LocalWaker; ordinary cross-thread wakes are deliberately inert.
- `hmux-buffer/src/buffer.rs` defines Buffer and line-ending policies.
  `SegmentedBuf` implements borrowed chunks, payload-preserving segment transfer,
  prefix-only pullup, and resumable line scanning. Buffer storage has no runtime,
  watermark, or notification dependency.
- `hmux-rt/LIBEVENT.md` maps all 33 existing libevent FFI functions to these
  contracts and future host responsibilities. This remains documentation only.

Validation passed:

- Six buffer integration tests, two runtime unit tests, fourteen runtime
  integration tests, six isolated process cases, and six doctests.
- Strict Clippy for both crates and all targets, formatting checks, and rustdoc
  with warnings denied.
- Baseline blob comparison and Cargo metadata checks confirm no application
  integration or cross-dependency between the new crates.

Process cases cover signals, waking a blocked poll, fork reset and retry after
resource failure, inherited destruction without disrupting the parent, descriptor
churn, and bounded waits interrupted by signals. Buffer tests include differential
operation sequences and every supported line policy at every chunk boundary.

Validation ran on Linux. Application conformance and comparative performance
benchmarks have not been run for these implementations because application
integration is explicitly outside this change. Signal subscriptions remove their
callbacks on drop; signal-hook leaves its OS disposition installed, so the host
must manage dispositions for children that exec. The crate README documents this
and the local-waker limitation. No commit was created.

## Buffer standard-trait refinement

SegmentedBuf now implements and hmux-buffer reexports bytes::Buf and BufMut.
Buffer extends those traits and Default, removing len, is_empty, append, and
clamping drain. Standard remaining/has_remaining, put_slice, and advance replace
them; the documented libevent adapter must clamp explicitly before advance.
Buf supplies chunk and bounded chunks_vectored; Buffer retains borrowed chunk
iteration for scans, pullup, line extraction, and transfer. Transfer now requires
moving payload allocations without copying, distinguishing it from BufMut::put's
append-and-consume contract. Uncommitted writable capacity is excluded from all
readable views, and only initialized bytes are published by advance_mut.

Nine buffer integration tests and two doctests pass, including direct writable
capacity, standard Buf consumers, vectored reads, and overrun rejection. Strict
Clippy, rustdoc with warnings denied, and formatting pass. The migration plan
and examples use the standard traits. Neither hmux nor hmux-rt implementation
was changed or given a dependency on hmux-buffer.

## Unified put API

Removed public Buffer::transfer. SegmentedBuf overrides BufMut::put using a
private min_specialization dispatcher: owned SegmentedBuf and &mut SegmentedBuf
sources move segment allocations without copying payloads, including calls through
generic Buf/BufMut bounds. Other source types and wrappers copy through Buf chunks.
The crate now requires nightly Rust. Buffer retains only chunks, pullup, and
read_line beyond its standard-trait bounds. Updated examples and libevent mapping.

All eleven buffer integration tests and two doctests pass, including allocation
identity checks for owned and borrowed specialized puts and fallback checks for
slices, Take, Chain, and dyn Buf. Strict Clippy, rustdoc with warnings denied,
and formatting pass. No application integration or runtime code changed.

## Runtime surface refinement

Removed Progress; Runtime::turn now returns io::Result<()> and keeps wait decisions
inside the driver. Moved concrete implementation types and source modules under
hmux_rt::mio, dropping Local prefixes. Root Runtime, Handle, Io, ReadyGuard, and
Signals remain traits; concrete counterparts and Task/Ready/Recv/Sleep/NextTurn
are exposed only through the mio namespace. IoState remains private registration
state shared by the public owner and the driver's weak registration references.

All two runtime unit tests, fourteen integration tests, and six isolated process
cases pass after both changes. Strict all-target Clippy, formatting, rustdoc with
warnings denied, and diff whitespace checks pass. No application integration,
buffer changes, or additional commit was made.

Runtime driving is now poll(max_wait) -> io::Result<()>. Removed the caller's
poll budget and renamed turn to poll throughout the trait, implementation, tests,
and current design docs. Mio retains a private 128-task-poll limit per call so
self-waking work returns control to the host. All sixteen runtime unit/integration
tests and six process cases pass; strict Clippy and formatting checks pass.
