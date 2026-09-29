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

## Window alert queue

Window owns alert flags, silence timers and queue membership. Alert dispatch uses
trait operations and retains the queued owner until after delivery and flag
clearing, then calls the existing explicit release. Session delivery uses Window
association traversal and Client's audible/visual alert operation. No model borrow
spans event delivery or status-message callbacks.

Tests retain the queue's existing behavior when callbacks append at or before its
tail, and cover disabled monitoring and duplicate queue membership. Layout and
other Window consumers still require migration.

## Client notifications and retained targets

Control event consumers and event-payload target resolution use the four traits.
Client now supplies notification eligibility, owned names, attachment accounting,
and detach/exec decisions. Live registry iteration is preserved, including changes
made by callbacks. Non-UTF-8 names remain byte-preserving.

The notification, reply and guard helpers also use a mapped-borrow-compatible
Client control-component API. Formatting happens before acquiring the component;
immediate stream writes happen after releasing it. Reply pressure is a Client
operation that preserves exit-message, exit-flag and discard ordering. Discard's
component helper accepts only control state and cannot access Client storage.
Tests cover recursive notification formatting, stopping control during formatting,
reply-limit behavior for an already exiting client, and exact notification bytes
and recipient order.

Other control paths, command-target helpers, Client terminal/status state and
cross-model legacy helper bodies are still outside the completed portions.

## Boundary checks and remaining work

`cargo test --test model_trait_boundary` rejects storage projections in the
migrated notification consumers and their reply helpers, whole-model references
or raw components in trait results, and known model/cell representation casts
throughout application source (including test fixtures). Syntax checks include
import/type aliases and pointer-to-integer cast chains; they are not a Rust type
checker and do not prove arbitrary expressions free of aliasing.

`python3 tools/model_boundary_inventory.py --json /tmp/model-boundary.json`
privatizes all four model structs in a disposable source copy, then classifies
Rust's field-privacy diagnostics by implementation owner. It never rewrites the
working tree. `--model session` restricts the probe for work on one trait. Window
helpers are not exempt from Pane boundaries, or vice versa. This conservative
inventory is intentionally nonzero while migration remains incomplete; there is
no baseline allowlist silently accepting the remaining field accesses.

The latest all-model probe still reports 3,271 candidate external field accesses:
287 Session, 821 Window, 1,341 WindowPane and 822 Client. Five additional type
inference errors in the private-field probe mean these counts are not an
exhaustive proof. They count field-access diagnostics, not independent changes.
The full migration is not complete. Remaining work includes:

- Session identity observers, options ownership, mutable winlink/index operations,
  and helpers still accepting `&session`.
- Window layout/scene state and cross-model legacy helpers.
- Pane base/current-screen pointers, input/parser and mode state, output paths.
- Client terminal/status/prompt state and other control-mode helpers.

In particular, `screen_write_ctx::screen_ptr`, options-scope pointers, and helper
conversions to whole-model references still prevent a whole-model RefCell swap.
Those paths need actual component borrow lifetimes and callback restructuring;
a pointer-returning adapter or a source-file move would not finish them. No
RefCell conversion or per-field RefCell has been made in these checkpoints.

Validation at this checkpoint:

- `cargo fmt --all -- --check` and `git diff --check`.
- `cargo test --workspace`: 768 passed, none failed or ignored.
- `python3 -m unittest discover -s tools -p 'test_*.py'`: 4 passed.
- Each of the four implementation commits also compiled independently with
  `cargo check --offline --all-targets` from its staged source tree.
- Default `cargo clippy --workspace --all-targets` stops at the existing
  `clippy::mut_from_ref` denial in `hmux-refbox/src/internals.rs:161`.
  A supplemental `-- --cap-lints warn` run finishes with warnings; this is not a
  clean default lint run. Existing code was not changed just to suppress it.
