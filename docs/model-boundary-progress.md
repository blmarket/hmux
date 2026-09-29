# Model boundary migration checkpoints

The four traits are still being migrated. A passing test suite is not evidence
that every external projection has been removed. The compiled trait definitions
are authoritative; `trait-models.md` describes the original adapter design.

## Session environment

Session owns its environment field. Consumers use associated borrow types so a
future implementation can return a mapped `Ref` or `RefMut` from one RefCell
containing the entire Session. No additional RefCell is introduced here.

Attach/switch commands copy the client's environment before applying Session's
`update_environment` operation. That operation borrows both Session components
internally and runs no model callbacks. Environment commands and customization
release component borrows before printing, formatting, or rebuilding the UI.
Customization snapshots retain the old opaque row tags, preserving selection
without returning a component pointer.

Tests cover update-environment rules, retaining a target across environment
replacement/removal, and snapshots surviving subsequent edits. Explicit Session
cleanup remains in the existing lifecycle helpers. The fixture replacement helper
is available only in unit tests.

This checkpoint does not complete Session's remaining options, winlink, identity,
and legacy helper boundaries.

## WindowPane synchronized output and palette

The pane trait owns sync-mode transitions, timer cancellation, dirty-row
bookkeeping and palette borrows. Parser/redraw callers no longer cast a pane
pointer to `UnsafeCell<window_pane>`. Palette borrows can become mapped Ref guards;
terminal colour lookup releases them before further terminal operations.
Logical pane destruction still clears dirty rows and cancels timers explicitly.
Final parser cleanup uses the unowned implementation path when Weak cannot upgrade.

Tests cover dirty-row resizing, timer cancellation without retaining the pane,
explicit destruction, final parser cleanup, and palette-source expiration.
The legacy screen-write target still exposes a screen pointer and is a remaining
component-boundary migration; moving sync bookkeeping does not complete it.
