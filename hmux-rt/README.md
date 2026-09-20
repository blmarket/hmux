# hmux-rt

A single-threaded async runtime embedded in a host-owned event loop. Core APIs
provide descriptor readiness (`AsyncFd`), timers (`sleep`), Unix signals
(`Signals`), and tasks (`spawn`, `JoinHandle`, `block_on`).

Reusable adapters are available without application globals:

- `ByteBuffer` and `LineEnding`: binary storage, contiguous slices, prefix drain,
  transfer, typed line extraction, and bounded I/O.
- `notify`: single-waiter coalescing Notify, yielding, and two-future selection.
- `registry::RuntimeControl`: explicit timer/I/O/signal registrations and
  cancellable next-epoch callbacks. Release IDs before their owners or borrowed
  descriptors are destroyed. Callbacks may cancel/rearm their own registration.
- `stream::StreamRegistry`: buffered reads/writes, independent read watermarks,
  write-drain thresholds, wakeups, EOF/error notification, and explicit release.
  Stream descriptors are borrowed and should be nonblocking (regular files also
  work). Free streams before closing/reusing descriptors. Buffer closures must
  not reenter the same stream registry operation while borrowing its storage.

Connect registries using `respawn_active(&runtime.handle())`. A host turn calls
`dispatch` with a budget, runs the current epoch's deferred callbacks, polls with
zero timeout if work remains, and dispatches again. Advance epochs explicitly;
a deferred callback scheduled during a batch belongs to the following epoch.
`flush_cancelled` releases cancelled tasks/fds without polling live tasks.

For fork rebuild, drop the inherited runtime without dispatch, poll, or flush:
the inherited kernel poller is shared with the parent. Create a fresh runtime,
then respawn registrations and streams. Monotonic timer deadlines survive the
rebuild. Clear registrations/streams and drop the runtime at shutdown.

The host owns process lifecycle, signal disposition policy, C callbacks, and
formatting compatibility. This crate has no dependency on application types.
