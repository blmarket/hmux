# hmux-rt: define the contract before implementation

Status: standalone crates implemented; application integration is explicitly
out of scope. Application source remains at `edc0f22` and uses libevent.
hmux-rt supplies the traits plus mio::Runtime; hmux-buffer supplies Buffer,
LineEnding and SegmentedBuf. The crates do not depend on each other, and hmux2
depends on neither. Workspace membership only enables their independent builds.

User direction: start with hmux requirements; an async runtime using only local
wakers is appropriate. General-purpose async ecosystem support is not required.

## Definition

**hmux-rt is a single-threaded, local-waker async runtime for hmux. It schedules
local futures and supplies descriptor readiness, monotonic deadlines, Unix signal
notifications.**

There is one task model, one driver, and one cancellation lifecycle. The host owns
process policy, byte storage, buffered-stream behavior and synchronous callback
compatibility. Host adapters use public operation futures; specialized syscall
adapters live inside the framework and use its private scheduling machinery.
These adapters do not introduce a second public callback scheduler.

Minimal means one primitive per responsibility, not merely the fewest method names.
Keep resource lifetime and fairness explicit even where they require an operation.

## Public trait definition

The reviewable signatures are now in [hmux-rt/src/lib.rs](hmux-rt/src/lib.rs),
with associated types for tasks and waits, and an opaque operation future. Separate
AsyncRead and AsyncWrite traits express byte-stream capabilities; other traits
cover driving, work creation and signals. There is no public Io trait or generic
syscall-closure API. Handle::Io names a byte-stream type implementing both
capabilities. Custom syscall scheduling is private to the framework. Readiness
guards, direction selection and acknowledgement are internal to the backend.
Independent byte storage lives in [hmux-buffer](hmux-buffer/README.md), with its
Buffer extension over bytes::Buf + BufMut and SegmentedBuf implementation.
The detailed implementation obligations are in
[CONTRACT.md](hmux-rt/CONTRACT.md). The mapping of all 33 current libevent functions
and required adapter changes is in [LIBEVENT.md](hmux-rt/LIBEVENT.md).

Use those files as the interface authority; the rationale below remains design
context. In particular, signal identities use `c_int`, results use `std::io::Result`,
and `AsyncRead::read` and `AsyncWrite::write` return byte counts directly.
Specialized accept and descriptor-passing adapters will live inside the framework
and expose operation-specific futures. They are not implemented yet. Internal
nonblocking syscall closures may return connections or messages with descriptors.
WouldBlock clears observed readiness internally; Interrupted retries cooperatively. Futures avoid requiring
boxing or Unpin. Public traits do not enforce semantic guarantees by themselves.

`Task` is a non-cloneable cancellation owner, not a public task ID or join handle.
Dropping it cancels and destroys its future and owned resources synchronously,
except that an executing poll finishes before destruction. Owners store
`Option<Task>` and take/drop it to cancel; there is no
second cancel-by-ID interface, detach mode, inline-spawn variant or join API.
Task results and completion state, where needed, live in the adapter's state.

The public implementation types for futures do not create additional operations.
No `sleep(duration)` synonym, separate dispatch/poll/block_on drivers, callback
registry, timer IDs, stream registry, general select combinator or public Notify
is required initially. Test-only counters stay out of the production interface.

## Contract

### Local execution and driving

All runtime-owned handles, tasks and awaitables are local to one OS thread and
are neither Send nor Sync. Spawn accepts non-Send futures and never polls inline.
A future must use local wakeups when it parks. This is a deliberately restricted
async runtime, not a promise that every Rust Future or third-party async library
will work. The previous implementation's standard Waker was a no-op; retaining
that behavior requires explicit documentation and a compatibility test. Do not
silently advertise ordinary Waker-based futures as supported.

`poll` is the only operation that polls live futures. It services readiness and
deadlines, runs a bounded number of future polls, and
settles resulting resource changes before returning. It must poll the kernel even
under a continuously runnable workload. It never sleeps while runnable work is
queued; otherwise the wait is bounded by `max_wait` and the nearest deadline.
No fixed 10 ms polling cadence. The host runs proc-loop/exit checks between turns.
Reentrant driving is rejected.

The runtime bounds task polling internally; poll exposes only max_wait. This
cooperative limit cannot preempt a future that never returns.
Stream and tty adapters therefore also bound bytes/syscalls per poll. Tasks may
self-wake and return Pending to continue work; this does not guarantee waiting
until the next host poll. Spawn never polls inline. Equal timer deadlines have
stable FIFO ordering; independent I/O order is not promised. Repeated wakes of
a queued task coalesce.

Runtime APIs take an explicit handle. Registering I/O must not require hidden
"currently polling task" state; local futures can be composed without a new task
for every leaf. Any internal task association is refreshed on poll.

### Cancellation and resource lifetime

Dropping a Task invalidates future scheduling immediately. Cancellation cannot
unwind a currently executing poll; that poll may finish but must not be repolled.
Wakeups carry generation/runtime identity so stale wakes cannot revive old work.
Identity exhaustion must not silently alias live or retired tasks.

Task-owner drop destroys a parked or unpolled future and its owned resources
without driving unrelated tasks. For a task currently executing, destruction
completes immediately after its poll returns, before another task is polled.
No later turn or separate cleanup call is required. Destructors can have their
own side effects; cancellation does not unwind an executing callback.

Dropping a pending I/O/signal/timer future unregisters its waiter. Dropping Sleep
removes its timer, with bounded stale storage. Adapters invalidate callback-owner
generations synchronously before dropping their tasks. After a callback that can
free/rearm its owner, the adapter must revalidate before any further owner access.
No registry/state borrow may span a business callback. Do not store Rust owners
requiring Drop directly in calloc/memcpy/free-managed application structures.

`Io` holds an Rc<OwnedFd> lease. The caller keeps a lease for its actual syscalls;
the runtime does not duplicate descriptors per waiter. Multiple waiters share
one backend registration per Io, with at most one pending waiter per direction;
conflicting waiters return an error. Registering the same descriptor twice must
have a defined rejection or sharing policy, not depend on backend accidents.
The C adapter explicitly adopts ownership or duplicates once at its boundary,
then cancels/releases before closing or reusing the original descriptor. If the
owning task is executing, final close waits for its poll and destruction to finish.

### Readiness, deadlines and signals

Readiness is cached until the operation closure reports WouldBlock. Internal
acknowledgement clears only the generation observed before that attempt, so newly
arriving readiness is not erased. Successful partial transfers, cancelled waits
and budget exhaustion do not clear readiness. Re-enabling reads or appending output must resume existing
readiness without requiring a new kernel edge. Read and write directions are
independent. With no write waiter, writable readiness must not cause busy polling.
Close/error notifications trigger an operation attempt; actual I/O determines EOF, partial transfers,
EINTR/EAGAIN and PTY EIO semantics.

Sleep uses an absolute monotonic deadline. Relative delays are computed once by
the caller, and repeating timers explicitly rearm. Configured/armed/pending
state and legacy deadline queries are adapter state, not a parallel runtime timer
registry. I/O-with-timeout is a composed future polling an I/O operation and Sleep; the
adapter specifies simultaneous-result priority and cancellation of the losing
wait. Composition helpers remain private until demonstrated public demand.

Signals::recv reports the signal number. Repeated occurrences may coalesce;
SIGCHLD consumers reap until exhausted. Signal handler installation/removal and
wake resources have explicit ownership, and conflicting subscriptions have a
defined policy. Handlers must not allocate or invoke application callbacks. The
host retains ignored/default disposition policy for exec children.

### Fork

Fork happens outside runtime dispatch in a single-threaded process. An inherited
runtime rejects ordinary driving until reset. `reset_after_fork` creates a fresh
executor/poller and invalidates old handles. It discards inherited futures; it
cannot resume/replay arbitrary task state safely as an application contract.
Creation failure leaves the old instance unusable but retryable in the child.
Cleanup must not deregister through the parent's inherited kernel poller. Drop
in an exec child must use the same parent-safe cleanup path.

The host preserves logical registrations, absolute deadlines, stream settings
and buffers in its adapter state. One host rebuild operation recreates tasks on
the new handle. There are no public per-registry respawn methods in hmux-rt.
Test both continued parent readiness and retained child logical work. The cost
of reconstructing host tasks is necessary with this model and must not be hidden
behind a claim that arbitrary async tasks survive fork.

## Covering hmux requirements

| Behavior | Composition and owner |
| --- | --- |
| accept, imsg/descriptor passing | Framework adapters expose typed operations over private syscall scheduling (future work) |
| raw tty | AsyncRead/AsyncWrite; framework adapter for specialized syscalls |
| status/escape/alert timers, delayed commands | Sleep; adapter owns armed state and repeat policy |
| event_active, event_once zero-delay | Cancellable spawned task; exact callback ordering to be validated during integration |
| event_pending and deadline output | Host logical state; monotonic-to-wall-time conversion at C boundary |
| I/O plus timeout | Private adapter future polls an I/O operation and Sleep, delivers one callback |
| pane/job/control/pipe streams | One stream adapter with bounded I/O and buffer watermarks |
| output append, read enable, watermark change | Adapter stores the polling LocalWaker and wakes on state transitions |
| EOF/errors, partial writes, EINTR/EAGAIN, PTY EIO | Stream/tty adapter handles syscall results |
| regular-file transfer | Adapter performs bounded I/O with task continuation; disk I/O may still block |
| byte storage, line parsing, transfer, pullup | One buffer module shared by standalone buffers and streams |
| varargs, malloc lines, legacy flags/raw pointers | hmux compatibility facade |
| daemon fork, pane/job exec, process exit | Host policy using runtime cleanup/reset |

The Buffer extension trait (Buf + BufMut) and SegmentedBuf implementation live in hmux-buffer, independently
of scheduling. The same storage contract can serve standalone buffers and future
application I/O tasks. It supports borrowed chunks, specialized BufMut::put for segment ownership and explicit
pullup. Read-only Reader and write-only Writer wrappers share the backend registration
model; the host chooses the appropriate wrapper for each endpoint. buffered-stream policy
and application compatibility adapters remain future work.
Preserve binary data, split line delimiters, transfer/drain semantics, and explicit
contiguous pullup while allowing normal writes to use chunks. Stream wake logic
is one private helper, with wake-before-wait and cancellation tests. It must not
be another independently scheduled event queue.

## Performance acceptance

This definition does not establish performance by itself. Compare with the same
libevent baseline before switching production:

- Idle CPU and wakeups, without periodic wakeups solely to drive the runtime.
- Input and timer tail latency under heavy PTY, pipe and control-mode traffic.
- Transfer throughput, CPU, allocation rate and copying for large binary buffers.
- Descriptor/task/timer counts and memory after repeated create/destroy/rearm.

Targets: reusable ready/poll storage, one queued entry per runnable task, cached
local wakers rather than fresh Rc allocation per poll, no whole-registry scan per
turn, O(1) runnable queue/slot operations, O(log n) timer updates and bounded stale
timer storage. Readiness notifications should allocate nothing after warmup.
Use a task per independent operation/owner, not extra task layers around every
wait primitive. Derive quantitative regression limits from measured baselines.

## Review sequence

Each item is a separate review/commit scope. The user authorized standalone
runtime and buffer implementation, covering the foundations in items 4-7.
Application adapters, backend switching and dependency removal remain future work;
do not automatically continue into integration.

1. **Rollback only.** Restore edc0f22, retaining history and the saved working state.
   Keep design documents out of the rollback commit.
2. **Definition only.** Review this local-async contract, especially cancellation,
   readiness acknowledgement, local-waker restrictions and fork reconstruction.
3. **Baseline evidence.** Separately restore required audit fixtures deleted by
   edc0f22; isolate test-harness fixes and capture behavior/performance baselines.
   Extract the saved session-group fix into its own regression-backed change.
4. **Executor foundation.** Add local task ownership, poll and Drop cleanup,
   with stale-wake, reentrant-spawn, cancellation and fairness contract tests.
5. **I/O and timers.** Add byte I/O capabilities, private syscall scheduling and Sleep; test budget continuation,
   simultaneous readiness, cancellation, deadline ordering and fd reuse.
6. **Signals and fork.** Add identity-preserving signal delivery and reset.
   Subprocess tests prove parent readiness and child reconstruction/exec cleanup.
7. **Buffer module.** Test bytes, delimiters, transfer/drain and pullup separately;
   production storage stays with libevent.
8. **Stream adapter.** Test watermarks, wake-after-append/re-enable, partial I/O,
   bounded work, EOF/error, files, PTYs and cleanup. Production stays on libevent.
9. **Application boundary.** Introduce accessors and host adapters in buildable
   changes with libevent still selected. Keep owner-layout edits with their users;
   avoid mixed Rust/libevent buffer ownership.
10. **Backend switch.** Wire already-reviewed components and lifecycle coherently.
    Introduce no new scheduling/storage algorithms; retain baseline comparison.
11. **Dependency removal.** After behavior and performance validation, remove the
    old backend and build inputs; prove binary/staticlib builds without libevent.

Run focused contracts and an application build for each increment. At the switch,
run CLI/protocol fixtures, attach/detach/resize, jobs, pipe-pane, slow control
readers, binary transfers, reaping and full conformance tests. Record baseline
failures separately. Isolate process-global signal/fork tests intentionally;
passing serial tests must not hide unexplained parallel failures.
