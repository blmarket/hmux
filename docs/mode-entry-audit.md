# Pane mode entry migration

`window_pane_modes.storage` owns an ordered vector of
`refbox::RefBox<window_mode_entry>`. The wrapper owns the vector; each entry has
its own stable allocation and weak identity. `window_pane_mode_insert_front`
returns that weak identity. `active_weak()` now exposes the active entry without
searching from a raw address. Screen-source updates use it, and mode reuse looks
up the entry in the owned vector before passing a borrowed pointer to the
existing callback interface. Immediate read-only queries in copy mode and pane
status borrow the active entry through its `RefBox`.

The copy, clock, and display-panes timers now capture weak mode identities and
borrow their entries only for callback dispatch. Expired modes skip dispatch;
conflicting nested borrows remain explicit errors. The remaining
`*mut window_mode_entry` uses are primarily the `window_mode` callback table,
its mode implementations, and synchronous calls into those callbacks. Revisit
other `active_ptr()` consumers that only inspect mode flags or data and can use
a bounded borrow.

Validation so far: workspace tests pass after the active weak accessor,
owned-stack lookup, read-only borrow changes, and weak timer captures.
