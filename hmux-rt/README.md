# hmux-rt

A single-threaded async runtime layer for the hmux daemon: futures, leaves,
and wake plumbing that embed into a host-owned event loop instead of owning
one themselves.

## Layout

- `src/tasks.rs` — task set, `TaskHandle`, `AsyncFd`, `Sleep`, `yield_now`,
  `join`, and the `WakeSink` seam the host's wake queue plugs into
- `src/reactor.rs` — `Reactor` trait and the mio backend
- `src/timer.rs` — timer queue
- `src/completion.rs` — `Completion` / `CompletionSender`
- `src/runtime.rs` — `TaskRuntime`, the standalone `block_on` driver for
  tests and examples

This README records the decisions the crate must honor. It is the authority
on scope; if code and this document disagree, raise it.

## Scope: internal-only, in both directions

The runtime is not published outside the project, and external async code is
not imported into the project.

- **No ecosystem interop, by design.** Each poll context is built with
  `ContextBuilder::from_waker(Waker::noop())` plus a live `LocalWaker`. Only
  `cx.local_waker()` is wired to the loop; a future that parks `cx.waker()`
  is never woken and hangs silently. This is a deliberate contract, not a
  gap: every event source lives on the loop thread, and a `Send` wake path
  would buy an `Arc`/mutex/wake-fd apparatus for wakes that never cross a
  thread. Do not propose a real `Waker`, a wake-fd, or `LocalSet`-style
  plumbing here.
- **The invariant:** every future that returns `Poll::Pending` must have
  parked `cx.local_waker()` (or live in a task something else re-polls).
  This is unenforceable in the type system — there is no `LocalFuture`; both
  wakers arrive through the same `Context` — so it is enforced by review and
  by keeping foreign leaves out.
- **Vendoring means porting.** Copying a third-party async primitive into the
  tree does not make it safe; its wake path must be ported to
  `LocalWaker`/`LocalWake`. For `Arc`-waker-based schedulers that is a
  rewrite. New capabilities are new leaves in this crate.
- The asymmetry is one-directional: leaves written against `local_waker()`
  would run on ordinary executors (`LocalWaker` is a same-layout view of
  `Waker`), but that portability is incidental, not a goal.
- Nightly-only (`#![feature(local_waker)]`) is accepted.

## Boundary rule

The point of the extraction is a hard boundary: a future commit that touches
both hmux-rt and other modules is a smell and should be challenged. The
initial extraction commit is the one exception.

Consequences:

- The seam is frozen before the split, because changing it later forces
  cross-boundary commits:
  - crate side: the leaves (`AsyncFd`, `Sleep`, `Completion`, `yield_now`,
    `join`), `TaskHandle::spawn`/`spawn_now`, and a narrow wake-sink trait
    replacing the daemon's `WakeQueue` coupling;
  - host side: the sync contract the daemon's loop drives (`take_spawned`,
    `deliver_io`, `take_new_io`, `take_released_io`, `set_io_token`,
    `deadlines`).
- `Completion` moves into the crate at extraction time, not later.
- The crate never grows its own run queue. A task resuming is an ordinary
  event in the host loop's single FIFO; dispatch order relative to the rest
  of the daemon's work is observable server behavior and belongs to the
  host. `TaskRuntime` exists only so tests and examples can drive the same
  dispatch-then-poll turn without a daemon behind it — it is a standalone
  driver for the one executor, not a second executor.
