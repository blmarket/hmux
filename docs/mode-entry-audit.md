# Pane mode entry migration

`window_pane_modes.storage` owns an ordered vector of
`refbox::RefBox<window_mode_entry>`. The wrapper owns the vector; each entry has
its own stable allocation and weak identity. `window_pane_mode_insert_front`
returns that weak identity. `active_weak()` now exposes the active entry without
searching from a raw address. Screen-source updates use it, and mode reuse looks
up the entry in the owned vector before passing a borrowed pointer to the
existing callback interface. Immediate read-only queries in copy mode and pane
status borrow the active entry through its `RefBox`.

The remaining `*mut window_mode_entry` uses are primarily the `window_mode`
callback table, its mode implementations, and synchronous calls into those
callbacks. Timer closures in copy, clock, and display-panes modes still capture
borrowed mode addresses. Review those captures for weak-handle migration while
preserving reentrant mode callbacks; a held `RefBox` borrow can conflict with
nested reads of the active entry. Then revisit other `active_ptr()` consumers
that only inspect mode flags or data and can use a bounded borrow.

Validation so far: workspace tests pass after the active weak accessor,
owned-stack lookup, and read-only borrow changes.
