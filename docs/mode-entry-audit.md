# Pane mode entry migration

`window_pane_modes.storage` owns an ordered vector of
`refbox::RefBox<window_mode_entry>`. The outer `Box` owns only the vector; each
entry is owned by its own `RefBox`, has a stable allocation, and supports a weak
identity. `window_pane_mode_insert_front` returns that weak identity.
`active_weak()` exposes the active entry without searching from a raw address.
Screen-source updates use it, and mode reuse looks up the entry in the owned
vector before passing a borrowed pointer to the existing callback interface.
Immediate read-only queries borrow the active entry or use `active_mode()` to
copy its static mode metadata. Empty-stack checks use `is_empty()`.

The copy, clock, and display-panes timers now capture weak mode identities and
borrow their entries only for callback dispatch. Expired modes skip dispatch;
conflicting nested borrows remain explicit errors. The remaining
`*mut window_mode_entry` uses are the `window_mode` callback table, mode
implementations that pass pointers through that table, synchronous mode command
helpers, and stack mutation that transfers `RefBox` owners. These pointer views
stay bounded by a pane owner, an entry borrow, or the active callback. Changing
the entire callback table would be a separate API migration and would require
updating every mode implementation together.

Validation: workspace tests pass after the active weak accessor, owned-stack
lookup, read-only borrow changes, weak timer captures, and empty-stack and
metadata accessors. The mode-stack regression checks stable identity through
promotion and removal, and that weak handles expire when entries are dropped.
