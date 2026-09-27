# tmux conformance gaps

## Session reference release

Status: open. Removing `SessionOwner` leaves temporary session owners as ordinary
`Rc<UnsafeCell<session>>` values, without retained diagnostic labels.

The pinned tmux revision, `e880cf63e0a9fe095d7c5d313761520fb1a8653c`,
[decrements the session reference count synchronously, schedules `session_free`
when it reaches zero, and rechecks the count in that callback](https://github.com/tmux/tmux/blob/e880cf63e0a9fe095d7c5d313761520fb1a8653c/session.c#L156-L180).
Immediate count reduction alone is therefore not a conformance gap; immediate
final destruction is.

The Rust implementation has two release paths:

- `session_remove_ref` moves an Rc into `rc::release_later`. The count decrement
  itself waits for event dispatch, cancellation, or scheduling failure. Monitor
  and run-shell teardown also call `rc::release_later` directly.
- Temporary Rc guards drop immediately at scope exit, including early returns,
  or when replaced. These include `monitor_get_session` callers,
  `control_sub_change`, `hooks_monitor_cb`, `window_panes_session` callers, and
  the source guard replaced by `window_panes_get_source`. No
  `session_remove_ref` call runs automatically on an Rc drop.

A concrete immediate-destruction sequence is:

1. An operation upgrades a weak session observer and retains its Rc guard.
2. The session is removed from the registry and other strong owners are released.
   If a release is deferred, that queued release must run while the guard still
   exists for the guard to become the last owner.
3. The operation returns or replaces the guard. Its Rc count drops from one to
   zero, running `session::drop` and `session_free` synchronously, and weak
   observers can no longer upgrade.

Tmux would schedule final cleanup for a later event callback at step 3 and could
observe a new reference before freeing. Rust instead frees immediately. This
can change cleanup ordering and the lifetime available to remaining raw
observers. If another owner remains, dropping a guard only reduces the count
and does not free the session.

The panes-mode `session_observer_rejects_removed_sessions_and_guard_keeps_allocation_alive`
test demonstrates immediate final cleanup after registry removal and release of
the original owner. It characterizes the Rust behavior, not tmux conformance;
a production differential reproducer for externally visible effects is still
missing. The retained-session tests separately cover explicit deferred release
on dispatch and cancellation.
