# Model boundary migration checkpoints

Session, Client, Window and WindowPane fields, constructors and core helpers are
now private to their respective owner modules. External code uses retained-holder
traits and bounded component operations. The current interfaces and lifetime
contract are described in [trait-models.md](trait-models.md).

The final source audit checks every Rust source and integration fixture for
external model storage types and representation casts. Compiler inventories
include tests and report zero external field accesses. Independent storage probes
for all four models make their get() projection private in disposable source
copies, catching inferred/unused projections and generic storage helpers.

The sections below preserve earlier checkpoints. Their diagnostic counts and
remaining-work statements describe those historical revisions, not the current
migration status. The final migration report records the complete validation.

## Historical checkpoints

## Completed Session storage and group follow-up

Following Window commit `c7c4700a`, Session now uses SessionRef/SessionWeak holder
aliases throughout its consumers. Unused projections, pointer null checks and
model-address UI tags were removed. UI identities use the retained allocation's
identity without interpreting its payload. All Session fields and whole-model
helpers remain private; the public factory returns a holder.

Group registry and membership orchestration live in sibling `session_group.rs`,
outside Session's implementation visibility. The Session trait supplies attached
count and a window-index synchronization operation. Each state/index loan ends
before linked/unlinked notifications; selection fallback, live source traversal,
post-notification alert copying, MRU remapping and old-index cleanup keep their
previous order. A callback regression adds a source link during traversal and
checks partial publication and both Sessions through their traits.

Session environment rows now carry stable entry IDs instead of component
addresses. Updates and clears preserve identity; removal/recreation and cloning
an entire environment create new identities. Detached entry snapshots retain
provenance. The environment and static-option tag namespaces remain disjoint.

Validation: the workspace passed 880 tests; Session, Window and Client opaque
storage probes passed all targets. The compiler field inventory for these three
models reports zero external accesses and zero other diagnostics. Its probe
preserves owner-module visibility for `owner/model.rs`, so it no longer invents
errors inside legitimate implementations; rustc mutation tests check that
external aliased/raw accesses still fail. Python discovery passed 18 checks plus
the existing 16 compiled options/layout cases. Formatting and diff checks pass.
Default Clippy still stops at the existing hmux-refbox `mut_from_ref` denial;
the complete capped run passes. `tools/session_boundary_smoke.py` passes group
membership, window ordering/selection, per-session renumbering, environment
inheritance and explicit final teardown against an isolated daemon.

Existing logical-lifetime preconditions remain: group-removal callbacks keep the
captured group/membership alive until removal completes, and linked callbacks
keep the current source/replacement winlinks alive until the alert-copy step.
Those preconditions were not replaced by Drop or by a broad implementation scope.
Production model storage remains UnsafeCell. Pane privacy and screen/parser
component boundaries are the remaining model migration phase.

## Completed Window boundary checkpoint

This checkpoint closes Window field/helper access, storage projections and
escaped layout/options/monitor component observers. The complete workspace
passed 874 tests, including the monitor and format-lifetime changes. Disposable
Window and Client private-storage probes passed all targets. This follows the
Client boundary commit (`b054cb33`); Session's remaining storage projections and
Pane's private fields/helpers still need follow-up. The older incremental notes
below describe what was unresolved at those points, rather than the current state.

The options inheritance boundary now uses `OptionsScope`: global scope or the
owning Session/Window/Pane's Weak identity. No parent table pointer crosses the
model boundary. Session's remaining `pub(crate) options` exception is now closed:
all its fields are private to its implementation, and the visibility audit no
longer permits an options exception. Resolution retains the defining scope across callbacks, so a
new shadow entry or reparenting cannot silently redirect an in-flight operation.
Numbers, strings, serialized values and styles leave visits as owned/copy values;
command lists retain their independent existing Rc. Temporary option reads do
not enqueue Session deferred releases or prolong its lifetime. Explicit logical
Session destruction/free remains unchanged.

Set/show options, hook dispatch and customization now keep scopes, names, keys,
and numeric record IDs between parsing, formatting and callbacks. Array dispatch
rereads each next value and preserves first-missing/first-error behavior.
Command parsing runs outside the model borrow. Monitor cleanup runs after value
cleanup and before unlinking the same entry, with a replacement identity check.
Monitor hook closures capture scope/generation and an owned fallback find state;
stale generations cannot dispatch replacement hooks. Monitor sets now have an
opaque, independently retained MonitorRef/MonitorWeak identity. Option/Client
borrows end before monitor operations. Timers upgrade weak identities; dispatch
copies each record and its prefetched successor identity before formatting or
callbacks. Replacing an item cannot receive its predecessor's result. Explicit
monitor destruction still cancels the timer, clears records, releases Session
ownership once, and retires callbacks; active dispatch stops when it observes
logical destruction.

`tools/monitor_borrow_boundary.py` runs the actual ten monitor regressions with a
single RefCell for the entire monitor state in a disposable source copy. They
pass, including destruction during the timer's first phase, successor removal,
same-name replacement, last-value/count ordering, and weak timer expiration.
The Client smoke covers all five subscription target kinds, initial/update
delivery, removal, and detach while subscriptions remain active. Source guards
reject exposed monitor state and raw monitor-set observers.

`WindowRef`/`WindowWeak` centralize holder types without changing production
storage. Whole-Window helper functions are now private; external traversal,
identity comparisons and scalar queries use the trait. The disposable
`tools/window_storage_boundary.py` wrapper makes `.get()` implementation-private,
including inferred and unused projections and storage-specific helper arguments.
Syntax tests also reject model casts, whole-model public helpers and exposed
component-return patterns covered by the current audit.

A durable actual-source harness (`tools/test_options_scope_borrows.py`) compiles
scope and scalar/string lookup with one RefCell per stub model. Seven Rust cases
cover released borrows/temporary owners, owned values outliving entries, parent
resolution through distinct models, reparenting and Empty versus empty strings.
It does not establish the remaining layout or full application callback paths.
Actual application tests have passed for options mutation (4), owned show rows
(5), customization (9), target observers (6) and ten integration suites (26).
The isolated Client smoke test passes terminal input, prompt/overlay callbacks,
resize, control protocol and detach. The new `tools/window_boundary_smoke.py`
also passes option inheritance through pane transfer, local override/unset,
array edits, rename hooks, layout resize/swap/rotation/zoom and explicit teardown.
The default workspace Clippy command still stops at the existing `mut_from_ref`
denial in hmux-refbox; the full run with `--cap-lints warn` passes.

Pane-held `layout_cell`/`saved_layout_cell` are now optional numeric cell IDs.
Rotation, swapping, pane resizing, spawning and border helpers resolve identities
under bounded Window guards. Mapped cell guards search both visible and saved
trees, so zoom preserves an already captured reservation. Removed IDs cannot
retarget a replacement cell. Preset reconstruction resolves detached IDs in its
owned leaf collection. Direct Pane field access remains for the Pane phase.

The actual-source `tools/test_window_layout_borrows.py` harness runs production
operation bodies with one RefCell per stub Window and mapped Ref/RefMut guards.
Its nine Rust cases include saved-tree assignment after reparenting, tree
replacement during resize, and policy/geometry/invalidation order. Actual command
tests cover both rotation directions and replacement during argument formatting.
The daemon smoke also covers all seven presets with two and three panes,
cross-Window swaps, and floating panes created while zoomed. These complement the
compiler probe and source guards. Format lookup and option-loop callbacks also
release retained Windows explicitly on every return path; regressions verify
that removing the last published owner closes it only after callback completion.
Production models still use UnsafeCell; no per-field RefCells were introduced.

Validation for this checkpoint:

- `cargo fmt --all -- --check` and `git diff --check` pass.
- `cargo test --workspace`: 874 passed, none failed or ignored.
- `python3 -m unittest discover -s tools -p 'test_*.py'`: 12 Python checks,
  plus 7 options-scope and 9 layout Rust cases, pass.
- `python3 tools/window_storage_boundary.py` and
  `python3 tools/client_storage_boundary.py`: all targets pass.
- `python3 tools/monitor_borrow_boundary.py`: 10 actual monitor cases pass with
  one RefCell for the entire monitor state in a disposable build.
- After `cargo build --bin hmux2`, both `tools/client_boundary_smoke.py` and
  `tools/window_boundary_smoke.py` pass against isolated temporary servers.
- Default Clippy still fails at the pre-existing `hmux-refbox` `mut_from_ref`
  denial. The supplemental complete `-- --cap-lints warn` run passes; this does
  not constitute a clean default lint run.

## Earlier Window migration notes (historical)

`window.rs` is now `window/mod.rs`, with Window storage and inherent storage
helpers in `window/model.rs`. Every Window field is `pub(super)`. Pane helpers
and the Pane trait are in the sibling `window_pane` module, so they do not inherit
Window's private visibility. Session-owned winlink helpers are in `winlink.rs`,
also outside the Window boundary. The compiler inventory preserves these separate
owners; layout, rendering, commands and winlinks are not Window implementation.

External identity reconstruction now reuses existing holders in 183 migrated
observer paths. Another 233 scalar/active-pane/scrollbar/time reads use existing
or needed Window operations. These counts describe the edits in this pass, not
a proof that all projections have gone. Directional pane search copies Window
geometry and traverses an owned pane snapshot. Pane index lookup, wrapping
navigation, stacking/history indices and association edits use Window operations.
Sorting reads copied timestamps and owned names without forwarding a Window
reference. No raw-pointer-returning API was added.

Winlink cleanup calls `prepare_release` while its existing Rc is still published,
then detaches and drops that exact Rc. It does not introduce a temporary Rc before
the final-owner decision. Swapping associations removes their old membership,
swaps the existing owners and appends membership in the established order. The
underlying explicit destruction and close notification procedures are retained.

Automatic naming now uses Window timer operations and Pane change/default-name
operations. Option format strings and the previous name are owned copies before
format expansion or rename notifications. Terminal input's rename path also ends
option access and copies its input string before notifications. Window-operation
replies copy pixel dimensions; title notifications keep the original parent
identity even if the pane moves. Temporary retained Windows use explicit release.
Redraw/status helpers accept Window holders instead of whole-model references;
some command callers still need migration. Swap-pane reuses its already retained
source owner after source selection instead of reconstructing it through storage.

Menu ownership now uses bounded Window detach/publication operations and weak
identities for the existing independently owned RefBox menu. Cancellation stays
outside Window access, including when it installs a replacement or releases the
last Window owner. Menu styles use scoped options across format evaluation.
Remembered menu positions and scene generations use Window operations; no external
access to those fields remains. The whole-Window scene-invalidation helper was
removed. Scene lookup/build and menu positioning retain their original Window
identity rather than requerying a mutable current link after format callbacks.
The menu tests use the trait and additionally reject a stale close request after
a replacement is installed. Source boundary checks now include menu consumers
and the migrated scene helpers.

Window sizing now exposes owned deferred-request snapshots, bounded request
publication, manual-size selection and latest-client identity checks. Reading a
deferred request does not consume it: the existing resize notification sequence
still clears its flag at the end. `resize.rs` no longer projects model storage.
Mouse move/resize and switch-client paths retain the original Window identity,
release it explicitly on all operation exits, and read scrollbar/modal policy
through the trait. Tiled mouse resizing still has an explicit, unmigrated layout
root/cell access. Client implementation consumers now use Window operations for
modal/active panes, traversal, redraw checks and pending sizing; there are no
remaining Window private-field diagnostics in that module, but this is not yet a
full private-storage probe.

Copy-mode, clock-mode, tree-mode and client-preview option reads now use bounded
Window option access. Where a function previously kept an option pointer across
formatting or other callbacks, it keeps the original Window's Weak identity and
upgrades it only during each access. Format strings that outlive the access are
owned CStrings. Style evaluation releases its option access before expanding
formats, then reacquires it for cache updates. The existing assumption that the
resolved option entry and its parent chain stay alive during evaluation remains;
the shared options-parent representation still needs migration.

Menu/popup border argument validation uses the static built-in choice definition,
without acquiring a Window options pointer. Tiled-pane border policy reads use
Window operations. Fill rendering publishes each fallback before expansion,
computes an owned cell outside model access, and publishes the result in the
original inside/outside order. Its renderer no longer receives a pointer into
Window storage. Default border styles and fill-rendering helpers are covered by
the syntax boundary audit. Client previews use selection history (not z-order)
when replacing the current mode pane; the history query has a regression assertion
where those orders differ. Copy/clock/client-preview modules have no remaining
Window privacy diagnostics, though their Pane projections remain for that phase.

Preset layout algorithms now use a GAT mutable root guard, compatible with a
future mapped `RefMut` from a single whole-Window RefCell. The preset module stays
outside Window's implementation. Before mutation it copies dimensions, pane order
and resize policy; percentage option strings are copied before parsing. Tree
rebuilding and distribution use pure geometry helpers under one bounded guard.
All tree pointers and that guard end before pane resizing, notifications and
redraw. Preset metadata still updates after arrangement notifications, preserving
the existing ordering. Snapshot iteration stays linear in pane count.

Root offset traversal, layout initialization/free and root resizing also use the
component API. Per-pane resize releases its root access before invoking resize
callbacks. The geometry helpers accept copied border/scrollbar constraints rather
than a Window owner, avoiding recursive Window borrows during distribution and
shrinking. Floating-position parsing records omitted axes through one Window
operation, preserving explicit-axis behavior and wrapping against the previous
offset. Raw cell creation/splitting/closing APIs and Pane-held cell pointers are
still unmigrated; the new guard does not make those remaining paths complete.

Format layout queries now request an owned layout string for the visible or
unzoomed view. Window first captures geometry, tree structure and weak pane
identities, releases model access, then serializes through Pane/Window trait
queries. Legacy floating-leaf exclusion, singleton collapse, root origin reset
and checksum output are preserved. The old pointer-based serializer and its
compatibility-tree cleanup helpers were replaced by an owned structural snapshot.
A syntax test rejects model projections inside the snapshot implementation.

Unzoomed pane width/height use a copied Window layout geometry record with saved/
current provenance and border-edge flags. Pane policy is evaluated after the
Window borrow ends; scrollbar reservation and saved-tree status-line behavior
remain separate calculations. Format callbacks for names, modal state, zoom state,
linked-session counts and bottom-edge checks also use holders. Neighbor-window
user options become owned name/value pairs before insertion into the format tree;
window loops copy their ID without retaining a raw Window pointer across expansion.
These migrated format queries no longer have Window privacy diagnostics; other
unused projections and legacy helpers still require the final storage probe.

Command target setters now accept the existing Window Rc rather than a borrowed
model. Target lookup/name matching, Session focus updates and Client pan-identity
fixtures no longer reconstruct observers through Window storage. The dead
whole-model `session_has` helper was removed. Unlink eligibility delegates to a
Window operation while borrowing the published winlink Rc without cloning it;
its existing strong-count-based result is preserved. An integration regression
covers one link, two links and removal of the extra link.

Monitor traversal uses Window holders while preserving live association traversal
across formatting callbacks and explicit release afterward. Pane scrollbar/theme/
printable-flag consumers use Window operations. Rendering copies active-pane weak
identity and stacking/pane snapshots before callbacks; draw-scene exits explicitly
release the original Window. The two-pane border helper still reads Pane-held
layout-cell pointers, so it is not a completed layout boundary.

Join-pane holds the original source/destination identities through notifications
and transfers the source Rc to its existing closer before the destination layout
notification. Error exits also explicitly release retained owners. A regression
checks source close/cleanup before the destination notification, including model
queries from callbacks. Remaining order helpers and options/layout pointers in
join-pane are explicitly unfinished; replacing observer reconstruction alone is
not a completed command migration. New/select-window latest-client bookkeeping
uses the existing Window operation.

Pane order and selection history now have GAT component borrows. Rotation,
swapping, join/break-pane and respawn finish these borrows before resizing,
selection callbacks or notifications. The ten whole-Window order mutation helpers
are private; the two old whole-model swap helpers were removed. The swap operation
handles aliased Rc holders with one Window borrow and borrows distinct Windows
only for cross-window swaps. Membership stays weak. Layout restoration uses
stacking snapshots and bounded history edits; the later restoration checkpoint
below removes its cell-pointer context from callback dispatch.

Break-pane retains its original source identity and explicitly releases it on
all exits. Newly created Window membership/name publication uses operations that
do not issue selection/rename events. The creator reference is still released
after attachment and layout initialization, before window-created/pane-moved
notifications. Respawn preserves removal, layout cleanup, pane destruction,
reinsertion, resize, layout initialization and active-selection order, ending
component borrows before each potentially reentrant step. Options parent pointers
in pane transfer remain exposed and unfinished.

Cursor-driven offset timers are now scheduled by Window. Existing 10 ms timing
and pending-event coalescing remain; callbacks hold only Weak, acquire an Rc at
dispatch, update terminal offsets and explicitly release it. The terminal helper
compares holder identities instead of projecting Window storage. An added test
checks weak timer ownership, repeated scheduling, dispatch ownership cleanup and
callbacks retained beyond explicit Window teardown. The initialization test
checks weak pane/client membership, byte-preserving names, automatic-name policy
and absence of premature selection/rename notifications.

The ordering integration test now uses only holders and component guards, and
the syntax audit rejects reintroducing its model projections or widening the
private mutation helpers. A temporary isolated harness ran that same test using
the actual ordering collection and swap method with one RefCell per stub Window
and mapped RefMut guards (1 passed). This checks same/cross-window borrowing and
weak ownership, not application teardown or the entire trait's RefCell readiness.
Production model storage remains UnsafeCell.

Tree previews now use Window holders for selection, membership and owned names.
Their option formats are copied before expansion; scoped style evaluation keeps
only the original Window's Weak identity between option visits. Retained preview
owners are released explicitly after formatting. Pane model projections in those
renderers remain for the Pane phase.

Display-panes previews copy pane layout geometry, including offsets and border
placement. The border walker borrows the root only during pure cell traversal;
that guard and every derived cell pointer end before model queries or screen
writes. A separate readonly GAT guard permits a future mapped Ref from the same
whole-Window RefCell as the mutable guard. Preview drawing no longer retains a
raw root through format callbacks. Two preview scaling tests passed in an isolated
harness using the actual helper and test bodies: mixed-axis scaling, minimum cell
size, clipping, zero dimensions and offscreen origins. This does not validate
application rendering or teardown.

Pane spawning, split validation and editor startup retain the original Window
identity through callbacks and release it explicitly on every returning path.
Modal publication is a bounded state operation: it remembers the previous active
pane weakly and leaves redraw, selection and notifications in the caller's
original order. The initialization regression now also checks weak modal/previous
ownership and absence of premature selection events. Options-change dispatch
refreshes cached scrollbar policy through a bounded operation, ends that access
before resizing, and traverses Window holders live after callbacks. Raw options
scope selectors and parent links remain unresolved; these changes do not hide
those pointers behind a new accessor.

Custom layout replacement now borrows the root only for detaching legacy
floating cells, dropping the old tree, installing and assigning the replacement,
and capturing restoration metadata. Dropping old cells still precedes publishing
new Pane cell links. Assignment helpers receive the borrowed tree and a pane
snapshot; they no longer project or recursively query Window. Parser cell pointers
are cleared before resizing, selection notifications or size recalculation.
Restoration carries only weak pane identities plus active/history/stacking values.
Its selection and history order remains unchanged. Diagnostic printing reacquires
the current visible tree after callbacks instead of retaining an old root pointer.

Two new behavior tests exercise weak restoration identities surviving tree
replacement and legacy fallback retaining floating-cell allocation/geometry and
pane links. The actual helper/test bodies and shared layout module pass three
isolated tests with a minimal Pane stub, including the existing layout ABI test.
They do not establish full application callback or lifecycle correctness. The
syntax audit also rejects cell pointers, component references and owning Rc fields
in restoration records, and model projections in restoration dispatch.

Split-space checks obtain Window policy and the split Pane's scrollbar width
outside the root borrow. The borrowed calculation uses only copied policy and
layout geometry. A new regression checks the outer top/bottom status-edge rule
and pane-specific horizontal minimum. Together with the existing distribution
regression and shared ABI test, three isolated geometry tests pass. Reservation and tiled-resize APIs are addressed by the later checkpoint below.
Closure and other callers of Pane-held cell pointers still need migration; the
scoped checker alone does not complete those callers.

Layout reservations now use a private numeric `LayoutCellId` minted by the cell
constructor. IDs are independent of addresses, remain stable when an owned Box
moves between parents, and do not retain a cell or pane. Removed IDs cannot match
new allocations. Spawn contexts, split/join commands, editor creation and zoom
reconstruction now carry these IDs instead of pointers into Window's tree.
Assignment resolves the ID during a short mutable root borrow and ends that
borrow before pane resize callbacks. Existing logical guarantees that a spawn
reservation remains live are asserted on assignment; IDs do not substitute for
explicit cleanup of failed operations.

Tiled/floating insertion resolves its anchor within the tree borrow. Split sizing
uses copied Window policy and pure geometry helpers, and floating-position format
expansion occurs outside the borrow. Node wrapping now accepts the already
borrowed tree, stays private to layout algorithms and does not expose a Window
storage adapter. Production storage remains UnsafeCell; there is no per-field
RefCell or whole-model RefCell conversion.

Tiled resize and mouse border selection also carry IDs. Border IDs are deduplicated
in the original search order, then resolved again after each notification. A cell
removed by an earlier callback is skipped; stale IDs do not query resize options
or dispatch another notification. Grow/shrink calculations finish under one root
borrow before offset updates, pane resizing and layout notifications. The existing
zero-change fix/notify behavior is retained for a live cell.

New tests cover ID transfer/removal/replacement, weak pane association during
floating assignment, an event callback removing the layout before a later border
is processed, and tiled width transfers preserving floating geometry. The actual
reservation/resize function and test bodies passed four tests in a temporary
harness with one RefCell per stub Window and mapped Ref/RefMut guards. The harness
uses a small event dispatcher and resize stub that reborrows Window, so it verifies
borrow release and callback reentry for these operations, not application teardown
or the complete event system. Syntax checks reject pointers/borrows in spawn
reservations and reservation results, and cell-pointer inputs in migrated layout
operations.

Cell closure now mutates the tree under its root guard using copied resize policy.
It clears the Pane link before releasing the guard, then fixes offsets, resizes
panes and fires the layout notification. The notification still resolves the Pane's
parent after resizing, preserving reparenting by callbacks. New regressions cover
surviving cell identity and geometry after removal, and a close callback rebuilding
the layout after observing cleared links. Seven isolated geometry/shared-layout
tests pass.

Floating conversion copies the saved geometry before argument/format expansion,
then resolves the Pane again under a fresh root guard to publish geometry and
reclaim tile space. It no longer carries a pointer to the cell's saved geometry
through expansion. Spreading performs its ancestor traversal and geometry edits
under one guard and releases it before pane resizing. The public pointer-taking
remove-tile and spread-cell adapters were removed. Tests cover weak association,
stable cell identity, redistributed widths/offsets, and the single-pane early
return without consulting options. Eight tests pass in the temporary whole-Window
RefCell harness, including the actual floating/spread test bodies with Window
fixture initialization stubbed. Production storage remains UnsafeCell. Tile
insertion, other Pane-held cell accesses and options parent/scope paths remain
unfinished.

The latest library compiler check reports 11 privacy errors, all options-field
accesses.
This is a diagnostic count, not a count of independent remaining changes. There
are still exported helpers accepting whole Window references and retained layout
component pointers; field privacy and the module move do not complete the trait
boundary. Pane fields themselves remain public pending their own phase.

The 19 syntax boundary tests passed in a standalone harness using the compiled
syn dependency. Five Python inventory/probe tests, formatting and diff whitespace
checks pass. A new behavior test covers base indices, forward/backward wrapping,
stacking traversal and snapshots surviving source cleanup. Additional tests cover
automatic-name deadlines, a timer callback surviving explicit Window teardown
without retaining the Window, and the deferred resize request remaining visible
through ordered layout/resize notifications. A fill-rendering test covers inside/
outside formatting, border style composition and snapshots surviving refresh.
A geometry regression verifies distribution, scrollbar minimum widths and unchanged
floating geometry. Its actual pure helper bodies and the shared layout module
passed two tests in a temporary isolated harness (with minimal pane/error stubs);
this does not validate full application orchestration. A floating-cascade test
also covers mixed explicit/default axes and wrapping after crossing the bounds.
New tests also cover JSON pane/selection/history/stacking metadata, legacy floating
exclusion, saved-vs-visible geometry, status-line and scrollbar reservation. The
legacy snapshot test passed in an isolated harness, and a differential check of
256 nested trees matched the previous serializer byte-for-byte (three isolated
tests including shared layout ABI checks). That harness uses model stubs and does
not establish JSON or application lifecycle correctness.
Full application execution of these new tests awaits a compiling worktree. An all-target compiler inventory reports 63 diagnostics, including
duplicates and unmigrated fixtures; no diagnostics point at the new name/timer
tests, the migrated winlink/order tests, the initialization/offset-timer tests,
the new join-ordering/preview-geometry/restoration tests, the split-space/reservation/resize/closure/floating/spread regressions,
or the migrated menu module/tests. The new behavioral tests have not run in the application yet. Full
lint, behavior tests and runtime validation for
Window remain pending. The Client results below describe its completed commit,
not this incomplete Window worktree.

## Client visibility checkpoint

Client state and storage helpers now live in `server_client/model.rs` with
`pub(super)` visibility. Only `server_client` implements Client internals; control,
TTY, status, formatting, queues and command consumers remain outside that boundary.
`ClientRef` and `ClientWeak` centralize the existing Rc/Weak cell type. No production
RefCell conversion or per-field RefCell was introduced.

External operations use Client methods or associated component guards. Terminal,
status, control, queue and format-job guards can become mapped Ref/RefMut guards
from one whole-model RefCell. Terminal output needs several Client fields together,
so terminal output helpers now retain the Client holder and open brief loans for
individual reads and writes. The former TerminalOutput view and both constructors
were removed. Status drawing extracts and restores its exact screen owner before
terminal calls.

Callbacks retain Rc/Weak Client identities and reacquire components on execution:

- Queue callbacks retain a weak queue owner, not a queue pointer. Dispatch and
  explicit item cancellation happen after releasing the queue guard.
- Format jobs retain Client/key/entry identities, reject callbacks from retired
  entries, and run expansion, process startup, cancellation and notifications
  outside cache borrows.
- Overlay callbacks are taken out before invocation and restored only if their
  generation remains current. Mode callbacks return copied ScreenMode values;
  clipping returns owned ranges, including during nested queries.
- TTY events retain Weak Client identities. Input decoding uses SegmentedBuf's
  contiguous bytes directly, with no snapshot copy, Rc cache or buffer pointer
  retained across dispatch. Clipboard/palette reply data is owned before callbacks;
  a regression listener replaces the input allocation during a decoded event.
- Terminal diagnostics retain weak Client and term identities, then copy each
  installed description under a fresh guard. The registry holds no term pointers.
- File callbacks, source-file completion and input-request operations reacquire
  Client indexes. Input contexts still own their independent request allocations;
  selection/cancellation releases Client before explicit request cleanup.

Prompt replacement detaches its old owner/screen before free callbacks. Status
rendering clears its borrowed screen target before formatting and reacquires it
without introducing additional flushes. Scene rendering owns its temporary cache
and preserves a replacement installed by nested rendering. Control teardown keeps
its established stream/monitor order and releases Client borrows before explicit
frees. Existing deferred Client release and logical destruction remain in place.

Validation:

- `cargo test --workspace`: 793 passed, none failed or ignored, including 14 source
  boundary tests and behavioral coverage for reentry, cancellation, byte output,
  snapshot ownership, retired registry entries and callback identity.
- `python3 tools/client_storage_boundary.py`: all targets compile with Client's
  storage projection private to its implementation in a disposable source copy.
  This catches inferred/unused `.get()` and storage-specific generic helpers that
  field privacy alone misses. Production source/storage is not changed by the probe.
- `python3 tools/model_boundary_inventory.py --model client`: zero external field
  diagnostics and no unexpected errors (748 implementation-owned accesses).
- Python inventory/probe tests: 5 passed. Formatting and `git diff --check` pass.
- `python3 tools/client_boundary_smoke.py`: isolated PTY input, prompt completion,
  menu rendering/selection/close, resize, terminal registry reporting, control
  protocol reply and explicit detach pass. The temporary server is stopped.
- Default Clippy remains blocked by `clippy::mut_from_ref` in
  `hmux-refbox/src/internals.rs:161`. `-- --cap-lints warn` completes with warnings;
  this is not a clean default lint run.

The compiler probe establishes the external Client storage boundary, not arbitrary
aliasing correctness. Syntax checks additionally reject model representation casts,
whole-model signatures, raw queue/cache observations and borrowed overlay results.
Behavior tests cover specific reentry paths, including component-only output under
one test RefCell. The unsafe single-thread/lifecycle contracts remain necessary;
this checkpoint does not claim that the existing model implementation is already
ready to compile unchanged with RefCell.

Window and WindowPane privacy work remains pending. Session's options exception
below is unchanged: shared option access is necessary, but direct access through
Session storage is still unfinished.

## Session visibility checkpoint (partial)

`session.rs` is now `session/mod.rs`. Session state and methods that expose a
whole Session reference are private to that module, **except `options`**, which
remains explicitly `pub(crate)` while its pointer consumers are migrated. This
exception is visible in the model and in the boundary test; it is not a new
pointer-returning trait adapter. This checkpoint does not complete Session or
the four-model migration.

Command target lookup, selection/ranking, registry traversal, identity checks,
status notifications and history/index operations use retained holders and
Session operations. Target setters accept holders and derive Weak identity
without projecting storage. Shifting indices and removing a replacement link
are operations rather than mutable index or whole-model accessors.

Option number reads finish within a component scope. Strings used by jobs,
copy mode, menus and title formatting are owned snapshots. The client key-table
helper now returns an owned string. Prompt/style evaluation releases option
access before format callbacks and revisits the original inherited entry to
publish the parsed cache. Status rendering likewise preserves the original
entry's parent depth while copying each line before expansion. No RefCell
conversion or per-field RefCell was introduced.

New tests lock field/helper visibility and verify stable winlink identity,
history ordering, and reentrant notification state during replacement: unlink
notification, alert/history clearing, window release, then current-link clearing.
The style callback test now checks that a simulated whole-model RefCell borrow
has ended before reentry. Tests that exercise private Session index ownership
live inside Session; other modules use small, cfg(test)-only fixture operations.
Production helper bodies were not moved into Session merely to exempt them.

A fresh Session-only private-field probe reports eight external diagnostics:
five in customization, two in options-scope selection and one in hook insertion.
There is also one follow-up inference error in the probe, so eight is an
inventory count, not proof of exactly eight independent changes. These consumers
retain options/entry pointers through later operations or callbacks. Completing
them requires target identity plus bounded component access, with snapshots or
reacquisition around formatting, editing and hook delivery. Their direct access
remains visible; wrapping it in a raw-pointer trait method would not solve it.
The broader non-Session boundaries remain unfinished.

Validation for this checkpoint:

- `cargo test --workspace`: 771 passed, none failed or ignored.
- Python boundary inventory unit tests: 4 passed.
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- Isolated daemon smoke test: session/window creation, ambiguous and glob targets,
  linking/replacement, insertion with index shifting, rename/selection, options,
  environment and destruction with a shared window. The test server was stopped.
- Default Clippy still fails at the pre-existing `mut_from_ref` denial in
  `hmux-refbox/src/internals.rs:161`; `-- --cap-lints warn` completes with warnings.

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

The previous checkpoint's all-model probe reported 3,271 candidate external field accesses:
287 Session, 821 Window, 1,341 WindowPane and 822 Client. Five additional type
inference errors in the private-field probe mean these counts are not an
exhaustive proof. They count field-access diagnostics, not independent changes.
The full migration is not complete. Remaining work includes:

- Session options ownership and pointers retained by option selectors,
  customization and hook insertion (see the newer visibility checkpoint above).
- Window layout/scene state and cross-model legacy helpers.
- Pane base/current-screen pointers, input/parser and mode state, output paths.
- Client internal storage conversion itself remains a separate, future change.

In particular, `screen_write_ctx::screen_ptr`, options-scope pointers, and helper
conversions to whole-model references still prevent a whole-model RefCell swap.
Those paths need actual component borrow lifetimes and callback restructuring;
a pointer-returning adapter or a source-file move would not finish them. No
RefCell conversion or per-field RefCell has been made in these checkpoints.

Validation at the previous checkpoint:

- `cargo fmt --all -- --check` and `git diff --check`.
- `cargo test --workspace`: 768 passed, none failed or ignored.
- `python3 -m unittest discover -s tools -p 'test_*.py'`: 4 passed.
- Each of the four implementation commits also compiled independently with
  `cargo check --offline --all-targets` from its staged source tree.
- Default `cargo clippy --workspace --all-targets` stops at the existing
  `clippy::mut_from_ref` denial in `hmux-refbox/src/internals.rs:161`.
  A supplemental `-- --cap-lints warn` run finishes with warnings; this is not a
  clean default lint run. Existing code was not changed just to suppress it.
