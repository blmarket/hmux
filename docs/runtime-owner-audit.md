# Runtime ownership audit

The application retains its C-allocated owners and exported callback signatures.
Embedded `event` values contain only integers and raw pointers, have no Drop,
and are valid when zero-initialized. Each cancellation retires registry entries;
configured-but-inactive events retain only their callback metadata. The event
layout shrinks from 128 to 56 bytes; stream handles shrink from 392 to 24 bytes.
Updated model fixtures measure the resulting owner sizes and all field offsets.
Protocol, serialization, and non-owner layout fixtures are unchanged.

| Owner | Construction / configuration | Release boundary |
| --- | --- | --- |
| Process and peer | proc_start, proc_add_peer, proc_set_signals | proc_remove_peer cancels before freeing; proc_clear_signals releases sources before resetting dispositions |
| Client and status | server_client_create, status_init, status_timer_start | server_client_lost/free, status_free, message/prompt/overlay cleanup cancel embedded timers |
| Terminal and input | tty_init/open/start, input_init | tty_stop/free and input_free cancel timers/watchers and free standalone buffers; input's pane stream is borrowed |
| Session and window | session_create, window_create | session_destroy/free and window_destroy cancel timers before final owner release |
| Pane and modes | window_pane_create, pane mode initialization | window_pane_destroy/free, window_copy_free and window_clock_free release timers/streams before freeing mode state |
| Monitor | monitor_create and timer setup | monitor_destroy removes pending timer before owner free |
| Job | job_run | job_free and job completion cancel/free stream, then close fd and free owner |
| Client file | file_create_with_peer/client, file_read/write_open | file_cancel/read_cancel and file_free/finished free stream before fd close and free standalone buffer |
| Control client | control_start | control_stop releases distinct read/write streams once and frees queued buffers |
| Pipe pane | pipe-pane command | pipe close/replacement and pane teardown free stream before fd close |
| Run-shell and format jobs | command/job creation | completion callbacks cancel timers or fire as one-shot; free job state after stream release |
| Deferred application work | alerts, session, client, file, run-shell | existing callback references remain held until the deferred callback; cancellation suppresses extracted batches as well as queued work |

`docs/runtime-lifetimes.tsv` inventories allocation, cancellation, and release
sites with their containing function. The source remains authoritative when line
numbers shift. Required exported C symbols are checked against the restored
historical export list; pointers to runtime buffers/handles remain opaque to C
consumers. No external consumer may rely on the removed event-loop struct ABI.

The adapter allocates standalone ByteBuffers and small stream handles with Box;
only adapter functions deallocate these. Runtime registries own stream buffer
Boxes and tasks. No Box/Vec is embedded in a calloc/free-managed owner. Accessors
borrow stream buffers until mutation or stream destruction. Appends and drains
notify the owning task; buffers are never passed to foreign buffer functions.
`server_client_print` now reacquires its pointer after appending a terminator.

Callbacks clone their callable before releasing registry borrows. Cancellation
invalidates registrations synchronously; `flush_cancelled` then drops tasks and
releases duplicated descriptors without running live callbacks. Fork children
must drop inherited runtime state without operating on the parent's epoll
instance. Server startup rebuilds surviving registrations against a fresh
runtime; exec children reset dispositions and close remaining descriptors.

## Embedded fields

```text
src/cmd/entries/run_shell.rs:105: pub timer: event,
src/shared/client.rs:133: pub event: event,
src/shared/client.rs:156: pub repeat_timer: event,
src/shared/client.rs:157: pub click_timer: event,
src/shared/client.rs:160: pub exit_timer: event,
src/shared/client.rs:164: pub cycle_timer: event,
src/shared/client.rs:178: pub message_timer: event,
src/shared/client.rs:194: pub overlay_timer: event,
src/shared/client.rs:224: pub buffer: *mut evbuffer,
src/shared/client.rs:225: pub event: *mut bufferevent,
src/shared/control.rs:18: pub read_event: *mut bufferevent,
src/shared/control.rs:19: pub write_event: *mut bufferevent,
src/shared/event.rs:37: pub input: *mut evbuffer,
src/shared/event.rs:38: pub output: *mut evbuffer,
src/shared/input.rs:24: pub event: *mut bufferevent,
src/shared/input.rs:51: pub request_timer: event,
src/shared/input.rs:52: pub since_ground: *mut evbuffer,
src/shared/input.rs:53: pub ground_timer: event,
src/shared/job.rs:27: pub event: *mut bufferevent,
src/shared/monitor.rs:26: pub timer: event,
src/shared/pane.rs:122: pub sb_auto_timer: event,
src/shared/pane.rs:140: pub event: *mut bufferevent,
src/shared/pane.rs:144: pub resize_timer: event,
src/shared/pane.rs:145: pub sync_timer: event,
src/shared/pane.rs:156: pub pipe_event: *mut bufferevent,
src/shared/process.rs:12: pub event: event,
src/shared/process.rs:27: pub ev_sigint: event,
src/shared/process.rs:28: pub ev_sighup: event,
src/shared/process.rs:29: pub ev_sigchld: event,
src/shared/process.rs:30: pub ev_sigcont: event,
src/shared/process.rs:31: pub ev_sigterm: event,
src/shared/process.rs:32: pub ev_sigusr1: event,
src/shared/process.rs:33: pub ev_sigusr2: event,
src/shared/process.rs:34: pub ev_sigwinch: event,
src/shared/session.rs:20: pub lock_timer: event,
src/shared/status.rs:14: pub timer: event,
src/shared/tty.rs:324: pub start_timer: event,
src/shared/tty.rs:325: pub clipboard_timer: event,
src/shared/tty.rs:347: pub event_in: event,
src/shared/tty.rs:348: pub in_0: *mut evbuffer,
src/shared/tty.rs:349: pub event_out: event,
src/shared/tty.rs:350: pub out: *mut evbuffer,
src/shared/tty.rs:351: pub timer: event,
src/shared/tty.rs:370: pub key_timer: event,
src/shared/window.rs:88: pub name_event: event,
src/shared/window.rs:90: pub alerts_timer: event,
src/shared/window.rs:91: pub offset_timer: event,
src/window_clock.rs:86: pub timer: event,
src/window_copy.rs:194: pub dragtimer: event,
src/window_copy.rs:195: pub refresh_timer: event,
src/window_panes.rs:124: pub timer: event,
```
