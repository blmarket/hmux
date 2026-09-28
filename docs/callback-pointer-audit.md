# Callback pointer type audit

The private timer and stream callbacks now take their actual pointer types:
`window_mode_entry`, `input_ctx`, `window_pane`, `window`, `client`, `monitor_set`,
`job`, `tmuxpeer`, or `tmuxproc`. Buffer and client mode callbacks take their
specific mode-data types. Input replies use `InputRequestReply` to distinguish
palette and clipboard payloads; captured file output uses a byte slice. The
unused erased command-queue callback adapter was removed.

A source scan for function parameters containing `*mut c_void` leaves one
project-defined callback: `compat/systemd.rs::job_removed_handler`. Its
`extern "C"` signature must match `sd_bus_message_handler_t`, which requires
opaque user data. The C boundary casts that argument to `systemd_job_watch`;
the other remaining `c_void` declarations are libc and systemd FFI functions.

Validation: `cargo test --workspace --quiet` and `git diff --check` passed after
the callback and payload changes.
