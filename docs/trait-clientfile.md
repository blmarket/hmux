# Client file Stream adapter

`client_file` now implements `futures_core::Stream<Item = std::io::Result<Vec<u8>>>`
directly. This is a consumer-side adapter over the existing file subsystem, not a
new transfer protocol or I/O scheduler.

## Compatibility boundary

Protocol version remains 8. Message IDs, payloads, peer dispatch, and producer
behavior are unchanged. There are no read-request, credit, or pause messages.
Local-file reads still use their existing I/O path; off-thread reads are outside
this change.

The earlier version-9 implementation was rejected and reverted. Testing only a
modified client against a modified server was insufficient. The regression test
in `tests/client_file_protocol.rs` exercises both mixed pairings against the
unmodified reference selected by `TMUX_UPDATE_REFERENCE`:

- tmux client with hmux server;
- hmux client with tmux server.

It covers multi-message file and stdin transfers, pane input, empty input, read
errors, source-file completion, and saved-buffer contents. It supplements the
workspace tests; it is not a claim of exhaustive tmux conformance.

## Polling contract

The private read state owns pending input and a local-task wake registration.
Existing `MSG_READ` handling appends bytes. Existing completion handling records
EOF or an error and schedules terminal dispatch as before.

`poll_next` returns:

- `Ready(Some(Ok(bytes)))` for an owned chunk of at most 8 KiB;
- `Pending` when more input or completion is needed, registering the latest
  `LocalWaker` supplied by the repository's local runtime;
- one `Ready(Some(Err(error)))` after buffered bytes if the transfer failed;
- `Ready(None)` after successful EOF or the terminal error. Subsequent polls
  remain terminated.

Polling a write record reports `InvalidInput` once and then terminates without
changing its output buffer or closed state. A file has one consumer: the legacy
callback adapter or a direct stream reader, never both simultaneously.

Wakeups are deferred until the caller releases its model borrow. Dropping a
pending next-item future does not consume input or cancel the transfer.

## Existing callback consumers

The existing callback path drains available Stream chunks into its accumulation
buffer at the same progress and terminal dispatch points. Source-file and
load-buffer therefore retain their whole-file accumulation behavior. Pane input
can continue consuming progress immediately. Completion remains deferred through
the existing one-shot event, with command waits cleared at the existing point.

Progress callbacks receive a buffer temporarily moved outside the file model.
This avoids holding a mutable buffer/whole-file borrow while a pane callback
cancels its input. The buffer and callback are restored after dispatch unless the
command wait was cancelled.

No additional consumer tasks or worker threads are introduced. Rewriting command
consumers into async functions can be a separate migration.

## Pause and memory

Not polling pauses consumption only. An unmodified peer can keep sending data,
and the pending input buffer can grow. The 8 KiB chunk limit bounds each yielded
allocation, not total queued memory. Source-file and load-buffer consumers also
continue to accumulate complete contents.

There is no new producer pause/resume operation or bounded-memory guarantee.
Stronger backpressure, spooling, and incremental local I/O need separate designs
that preserve tmux behavior and the existing wire protocol.

## Ownership, privacy, and cleanup

The model and its fields live inside the file module. Keep
`Rc<UnsafeCell<client_file>>` owners and `Weak<UnsafeCell<client_file>>` observers
directly, without new type aliases or handle wrappers. Existing shared type paths
are re-exported for callers.

Collection iteration stays on `client_files`, including the accepted `.fuse()`.
Collection readiness and interruption operations keep server callers from
inspecting individual file fields. Readiness accounts for both pending stream
input and the existing callback/write buffer.

`Rc` keeps the allocation alive but does not establish exclusive access. Code
obtaining a reference from `UnsafeCell` must respect borrowing rules for each
poll. No model reference may cross a listener invocation or re-entrant wakeup.

Read cancellation continues to use the existing `MSG_READ_CANCEL` path. Command
wait cancellation remains a separate operation with its existing behavior. The
adapter does not reinterpret dropping an owner or a pending future as abort.

Preserve explicit stream free, descriptor close, index removal, client unref,
and logically guaranteed weak lifetimes. Terminal dispatch or cancellation still
retires the index independently of future polling. A direct reader retaining an
owner can consume already-buffered bytes after that retirement. No additional
cleanup responsibility moves into `Drop`.
