# Model API migration

Objective: no raw pointers to `session`, `window`, `window_pane`, or `client`
in struct fields, function parameters, or return types. Brief local pointers
for UnsafeCell access remain permitted. Preserve explicit cleanup and the
existing guarantees for borrowed upgrades; do not add cleanup wrappers or
replace pointers with aliases/NonNull to evade the objective.

## Verification

Run `cargo run --example model_pointer_inventory`. It parses Rust source in
the application, examples, tests and workspace libraries, reporting pointer
types in fields, signatures and type aliases, including nested callback types
and NonNull, wrapped pointees, closure signatures, and local function-pointer
types. Ordinary local pointer variables and casts are deliberately excluded. It exits nonzero while
occurrences remain. Completion also requires checking aliases and macro-generated
APIs that syntax traversal alone may not resolve, and running the tests.

Initial application/test audit: 448 explicit raw pointer occurrences. It also
had a NonNull client prompt callback argument, now migrated.

Final inventory: zero model pointer types in fields, signatures, and aliases.
The source review also checked model import renames, raw Self types, local
macros, build scripts, and the generated command-parser grammar. No additional
model-pointer APIs were found. The syntax audit is not a Rust type resolver;
future aliases, generic substitutions, or macros still require review.

## Completed implementation batch

- Session index insertion returns an optional Rc for the existing allocation;
  removal borrows an Rc and checks allocation identity. Duplicate-name and
  retained-result regression coverage was added.
- Session destruction, attachment, detachment, window selection, group
  synchronization, and renumbering borrow Rc handles. Short raw locals still
  access UnsafeCell within these operations; explicit destruction/deferred
  release remains intact. `src/session.rs` has no targeted model pointers in
  its signatures.
- Event payload getters return `Option<&Rc<UnsafeCell<T>>>`, borrowing the
  payload's existing owner without creating a new release obligation.
- The Box-returning format constructor accepts an optional borrowed client Rc;
  hooks pass the payload's typed owner directly. Retention and deferred cleanup
  tests cover both format constructor variants.
- Status/pane prompt callbacks accept optional borrowed client owners instead
  of NonNull pointers. Status callbacks capture a Weak and upgrade for dispatch.

- Event dispatch and payload setters consume retained Rc handles. The payload
  owns each transferred reference and releases it through the existing explicit
  cleanup functions; dispatch does not add a temporary final window drop.
- Control APIs borrow Rc handles across reentrant callbacks. Leaf operations
  borrow client data directly. Pane lookup returns an optional retained pane.
- Alert checks borrow owners; the alert queue clones the window handle and
  retains its existing explicit release path.
- Monitor APIs use optional borrowed Rc handles for nullable model contexts.
  Session lookup compares Rc identity directly; change reports store Weak links.
- Format constructors and default/single-format APIs accept optional borrowed
  handles, with typed defaults for inferred sessions and active panes. The cycle
  timer captures a Weak client; a regression checks non-retention and dispatch
  after expiry.
- Layout operations borrow window/pane handles. Optional panes and detached
  cell destruction use Option. Layout selection callbacks borrow the same
  window handle; the first-tiled-pane lookup returns an optional retained pane.
- Core window/pane operations accept borrowed Rc handles or data references.
  Pane creation and insertion return Rc handles. Window-index removal returns
  a boolean after checking Weak identity, without retaining the window.
- Window release borrows the existing owner through preparation and destruction;
  winlinks keep their owner field published during close callbacks. The final
  release check uses Rc::strong_count and does not acquire an extra guard.
- Shared session/window/pane accessors return retained or borrowed handles.
  Observer setters accept optional data references; raw pointer projections
  remain only at legacy use sites. Copy-mode/source-file accessors were migrated
  as well. No targeted signatures remain in the shared model modules.
- Command-find APIs take typed owners. Mouse lookup's optional session output
  is an Option<Rc>, retained by callers while they use their local raw view.
- Client operation APIs borrow owners across callbacks. Read-only operations
  borrow client data; paste-state mutations and final client allocation cleanup
  use mutable references. Overlay tests exercise reentrant self-close using
  Rc-backed clients. No targeted signatures remain in src/server_client.rs or
  src/window.rs.

- Command/server, input, popup, resize, status, and drawing APIs now borrow
  owners or model data. Timer callbacks store Weak observers and upgrade only
  for dispatch. Nullable arguments use Option.
- Pane spawning returns Option<Rc<UnsafeCell<window_pane>>>. Pane-mode source
  lookup returns a window observer and retains the source session for callers;
  its temporary window owner still passes through explicit release.
- Screen-write synchronization borrows UnsafeCell directly so final pane
  allocation cleanup does not need to upgrade an expired observer.

Validation: `cargo test` passed (657 tests); the audit's two regression tests
passed with `cargo test --example model_pointer_inventory`; the inventory
reported zero and exited successfully; `git diff --check` passed.

This completes the model field/signature migration. UnsafeCell access and brief
raw locals still require the existing lifetime and aliasing reasoning. This is
not a conversion of every operation to safe Rust. Explicit model cleanup remains
in place; no teardown scope or new cleanup-on-Drop wrapper was introduced.
