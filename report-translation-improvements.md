# Translation improvements: memory safety and maintainability

## Implementation progress

The report remains in progress. Completed slices:

- Hyperlink registry ownership (items 2–4, 6–7, 9): sets retain a checked,
  thread-confined registry owner; eviction releases the registry borrow before
  visiting another set. Cleanup works after the thread-local owner is destroyed.
  Removed `GlobalLinkedHashMap` and its unconditional unsafe `Sync` assertion.
  Identity counters reject exhaustion rather than recycling a live identity.
  Four focused tests cover eviction, thread isolation, last-owner cleanup, and
  thread-exit ordering. Validation: 3,085 hmux unit tests, 1,279 conformance tests,
  lint, and 3,223 leak-gate tests passed with tmux 3.7b. Miri is unavailable in
  the installed toolchain; no Miri result is claimed.

- README architecture refresh (item 7): corrected package/library naming, module
  ownership, FFI layout, and reference-versus-subject test instructions against
  the current source and Makefiles. Documentation-only validation: diff check.
- Wait-channel confinement (items 2–3, 6, 9): channel transitions use checked
  thread-local access; signal, unlock, and flush release the registry borrow
  before resuming queue items. Existing weak observations still skip destroyed
  waiters. The 19 focused tests include cross-thread signal/flush isolation;
  all six `wait_for` command conformance cases pass. Full validation: 3,086 unit
  tests, 1,279 conformance tests, lint, and 3,224 leak tests passed with tmux 3.7b.
- Buffer boundary and instrumentation (items 5, 8, 10): the `bytes::Buf`
  implementation now rejects oversized copies before consuming bytes, as its
  existing contract requires; the inherent operation still clamps requests.
  A regression reproduced the prior silent truncation. Deterministic sequences
  exercise fragmented operations against a contiguous byte model. The module
  denies implicit unsafe operations in unsafe functions. Added the focused
  `make check-buffer-memory` AddressSanitizer gate; all nine buffer tests pass
  under it. Full validation: 3,088 unit tests, 1,279 conformance tests, lint, and
  3,226 leak tests passed with tmux 3.7b.
- Key-table registry (items 1–3, 6, 9): completed thread confinement around the
  already checked key-table handles. Lookups and enumeration return owners;
  removal releases the registry borrow before client updates or table cleanup.
  Tests cover conflicting borrows through clones, retained owners versus a
  replacement with the same name, and independent thread registries. Validation:
  3,091 unit tests, 1,279 conformance tests, 34 key-command conformance tests,
  lint, and 3,229 leak tests passed with tmux 3.7b.
- Literal-table safety (items 2, 5, 7): removed `ReadOnly<T>` and its generic
  unsafe `Sync` implementation. Both callers use immutable `&CStr` arrays;
  default binding parsing no longer reconstructs references from raw pointers.
  All 275 binding strings and seven drawing-state labels were checked unchanged
  and in the same order. Validation: 17 focused tests, 3,091 unit tests,
  1,279 conformance tests, 34 key-command conformance tests, and lint passed.
- Paste-store confinement (items 2, 4, 6, 9): replaced the global wrapper and
  its unsafe `Sync` assertion with checked thread-local storage. Notification
  dispatch still follows the committed mutation and release of the borrow.
  Tests exercise independent stores, checked nested access, and recovery after
  a rejected borrow. Validation: 3,093 unit tests, 1,279 conformance tests,
  70 paste-command cases, six paste-notification cases, lint, and 3,231 leak
  tests passed with tmux 3.7b.
- Test independence gate (item 9): all 3,093 hmux unit tests passed in separate
  processes with no default-filter exclusions or ignored tests held back.
  The parent unit gate now uses nextest and runs doctests separately, replacing
  the stale shared-process requirement. The updated full `make test` passed,
  including all 1,279 conformance cases; hmux0 and c2rs target selection was
  checked with Makefile dry runs.
- Pane/window identity exhaustion (items 3–4, 7, 9): entity counters record an
  exhausted state after issuing the final ID, preventing stale observations
  from matching a replacement through wraparound. Pane creation reserves its ID
  before allocating resources; the separate activity stamp still wraps. Tests
  cover the final IDs, repeated failure, the window constructor, and fixture
  reservations that cannot rewind or revive an exhausted counter. README
  documents the exhaustion difference. Validation: 3,095 isolated unit tests,
  1,279 conformance tests, lint, and 3,233 leak tests passed with tmux 3.7b.

- Session handle registry ownership (items 2–4, 6, 9): the weak address index
  is thread-confined, and each allocation owns its registration cleanup. Tokens
  retain the index through teardown and cannot remove a replacement entry.
  The shared registry helper releases its borrow before dropping observations.
  Seven focused tests cover replacement, cleanup reentry, thread isolation, and
  session cleanup after the thread-local index owner is gone. Its four isolated
  helper tests also pass under AddressSanitizer. Full validation: 3,102 unit
  tests, 1,279 conformance tests, lint, and 3,240 leak tests passed with tmux 3.7b.
  Session payload borrowing and the live-session registry remain unchanged.

- Client/window handle registry ownership (items 2–4, 6, 9): both weak
  address indexes now use thread-confined registrations retained by each
  allocation through teardown. Four additional tests cover last-owner removal
  and cleanup after the thread-local owner is gone. All 11 registry tests pass.
  Full validation: 3,106 unit tests, 1,279 conformance tests, lint, and 3,244
  leak tests passed with tmux 3.7b. Payload borrowing remains unchanged.

- Window-ID registry ownership (items 2–4, 6, 9): windows now own their
  thread-confined ID registrations. A regression reproduced a stale weak entry
  left by final-owner destruction, where upgrading that same dying allocation
  could never succeed. Token cleanup removes it without upgrading and preserves
  replacement registrations. Enumeration still snapshots IDs and revalidates
  individual lookups. Tests also cover retained owners, thread isolation, and
  thread-exit cleanup. Validation: 3,108 unit tests, 1,279 conformance tests,
  lint, and 3,246 leak tests passed with tmux 3.7b.

- Command-queue item registry ownership (items 2–4, 6, 9): queue items retain
  their thread-confined address registrations through teardown. Cleanup releases
  the index before destroying callback data; a regression exercises a callback
  payload destructor that creates and resolves another item. Additional checks
  cover retained owners, expired weak observations, thread isolation, and
  thread-local destruction order. Validation: 13 focused registry tests,
  3,110 unit tests, 1,279 conformance tests, lint, and 3,248 leak tests passed
  with tmux 3.7b. The global command queue remains a separate migration.

- UTF-8 character-store synchronization (items 2–3, 6, 9): one checked lock
  now protects both intern indexes and their allocation counter. The shared
  store preserves packed-character decoding across threads, and logging happens
  after releasing the lock. Tests cover concurrent interning/decoding and final
  index exhaustion while existing identities remain usable. Validation: 34
  focused UTF-8 tests, 3,112 unit tests, 1,279 conformance tests, lint, and 3,250
  leak tests passed with tmux 3.7b. Width configuration remains separate.

- UTF-8 width configuration (items 2, 5–7, 9): width overrides use checked
  thread-local storage, and temporary width suppression restores its previous
  value after nested parsing or unwinding. The shared packed-character store is
  unchanged. Tests cover independent thread configuration and nested/panicking
  suppression scopes. Validation: 36 focused UTF-8 tests, 3,114 unit tests,
  1,279 conformance tests, lint, and 3,252 leak tests passed with tmux 3.7b.

- Pane registration teardown (items 3–4, 6, 9): each registered pane now
  retains its index owner and removes only its own observation. A regression
  reproduced a process abort when a pane outlived the thread-local index; the
  same test now passes. Another test preserves a replacement entry when an old
  pane is dropped. Validation: 3,116 unit tests, 1,279 conformance tests, lint,
  and 3,254 leak tests passed with tmux 3.7b. Pane lookup remains thread-local.

- Session identity exhaustion (items 2–4, 7, 9): sessions share the checked
  monotonic allocation helper used by panes and windows, with a thread-local
  counter that never wraps. Creation reserves an ID before allocating the
  session; name collisions still consume IDs. Exhaustion and a collision at
  the final ID leave no partially registered session. The next-session format
  is empty when no IDs remain. Validation: 73 focused session/command/identity
  tests, 3,118 unit tests, 1,279 conformance tests, lint, and 3,256 leak tests
  passed with tmux 3.7b. README records the exhaustion behavior.

- Collected screen-item pool (items 2–5, 6, 9): one checked lock protects
  the shared item pool and its FIFO recycling list, preserving late screen
  cleanup and item snapshots across callers. Returning an item leaves its data
  readable until reuse, when it is reset. Allocation rejects the reserved
  sentinel and index truncation. All mutation closures were checked to perform
  local field updates without dispatching callbacks. Validation: 91 focused
  screen-write tests, 3,121 unit tests, 1,279 conformance tests, lint, and 3,259
  leak tests passed with tmux 3.7b. No performance benchmark is claimed.

- Session-group registry confinement (items 2–4, 6, 9): group lookup and
  mutation use checked thread-local storage; removal releases the registry
  borrow before dropping weak observations. Tests show independent thread
  registries, sessions that expire while still observed by a group, and group
  teardown before a retained session. Validation: 69 focused session/group
  command tests, 3,123 unit tests, 1,279 conformance tests, lint, and 3,261 leak
  tests passed with tmux 3.7b. Group compatibility views remain raw pointers.

- Terminal-description registry ownership (items 2–4, 6, 9): terminal
  descriptions own their registrations in a thread-confined observer list and
  retain that list through teardown. A regression reproduced the stale pointer
  left by ordinary `Box` destruction; both ordinary and explicit cleanup now
  unregister before releasing capability storage. Newest-first listing order
  and partial-construction cleanup are preserved. Validation: 14 focused
  registry/show-messages tests, 3,127 unit tests, 1,279 conformance tests, lint,
  and 3,265 leak tests passed with tmux 3.7b. Terminal observations remain raw
  pointers into their owning boxes.

- Job identity exhaustion (items 2–5, 7, 9): jobs use the shared monotonic
  allocation helper with a thread-local counter. An exhausted counter returns
  the existing start-failure result and `EOVERFLOW` before environment setup,
  descriptor allocation, or fork. Callback data is dropped before setting the
  error, so its destructors cannot overwrite the reported cause. Validation:
  11 focused job tests, 3,128 unit tests, 1,279 conformance tests, lint, and
  3,266 leak tests passed with tmux 3.7b. The owning job registry remains global.

- Explicit session payload access (items 1, 3, 6–7, 9): `SessionRef` no
  longer implements safe `Deref`/`DerefMut`. Compatibility views require unsafe
  calls with lifetime/exclusivity contracts; mutable views also require a
  mutable handle borrow. Target state records owner observations directly,
  address-only consumers use an identity operation, and group walks reuse the
  incoming reference when visiting that same session. Both forbidden coercions
  were verified to compile before the change and are now compile-fail tests.
  Validation: 434 focused tests, 3,129 unit tests, eight compile-fail doctests,
  1,279 conformance tests, lint, and 3,267 leak tests passed with tmux 3.7b.
  After integrating subsequent upstream queue/alternate-screen changes, all
  3,130 unit tests, doctests, the tmux-sys unit gate, and lint passed again.
  This removes implicit safe reference exposure; checked borrowing throughout
  the unsafe engine and its raw back-pointers remains unfinished.

- Explicit window payload access (items 1, 3, 6–7, 9): `WindowRef` no
  longer implements safe `Deref`/`DerefMut`. Compatibility views have explicit
  unsafe contracts, and target state records weak owners without payload
  references. Pan-window comparisons use allocation addresses directly.
  Shared and mutable coercions compiled before the change and now fail their
  compile-fail checks. Validation: 411 focused tests, 3,131 unit tests, ten
  compile-fail doctests, 1,279 conformance tests, lint, and 3,269 leak tests
  passed with tmux 3.7b. Raw engine borrowing remains a separate audit.

- Control notification client access (items 1, 3, 6, 9): recipient payloads
  now use an explicit unsafe mutable view. Session-change and detach broadcasts
  reuse the incoming mutable client when it is also a recipient, and snapshot
  its name before output. Coverage includes self-detach output as well as the
  existing self/other session-change messages. Validation: 15 focused tests,
  3,131 unit tests, ten doctests, 1,279 conformance tests, lint, and 3,269 leak
  tests passed with tmux 3.7b. Client handle coercions elsewhere remain open.

- Command-target client lookup (items 1, 3, 6, 9): activity, session, name,
  and terminal comparisons use explicit shared client views; inside-pane
  lookup requests its mutable view explicitly. Selection still returns the
  owning handle. Validation: 196 focused tests, 3,132 unit tests, ten doctests,
  1,279 conformance tests, lint, and 3,270 leak tests passed with tmux 3.7b.

- Listing-command client views (items 1, 3, 6, 9): the seven listing paths
  obtain explicit shared client views for formatting and client filtering.
  Validation: 415 focused tests (including lookup, queue, and parser coverage),
  all 53 listing-command conformance cases, and lint passed with tmux 3.7b.

- Client mutation commands (items 1, 3, 6, 9): attach/detach, server access,
  and pane-selection redraw paths request explicit client views. Attach skips
  the current client by address before accessing another payload. Validation:
  61 focused tests, 46 command conformance cases, and lint passed with tmux 3.7b.

- Deferred-command client views (items 1, 3, 6, 9): confirmation and shell
  callbacks request explicit client views for status, output, queue insertion,
  and return-code updates. Removed two unused unchecked payload getters.
  Validation: 43 focused tests, 27 command conformance cases, and lint passed
  with tmux 3.7b.

- Command I/O client views (items 1, 3, 6, 9): display, buffer loading/saving,
  pipe, message, and source-file commands request explicit client views for
  formatting and I/O callbacks. Validation: 65 focused tests, 49 command
  conformance cases, and lint passed with tmux 3.7b.

- Command dispatch client views (items 1, 3, 6, 9): parser formatting,
  queue dispatch, target fallback, and remaining display formatting now use
  explicit client payload views. Validation: 262 focused tests, all 45 queue
  conformance cases, and lint passed with tmux 3.7b.

The accumulated client command migrations passed the full gate at `0e23f2dbe`:
3,133 unit tests, ten doctests, all 1,279 active conformance cases, and 3,271
leak tests passed with tmux 3.7b. Each command slice also passed lint and its
focused checks before being committed.

- Server client lifecycle views (items 1, 3, 6, 9): timers, teardown,
  redraw, and pane-loss cleanup request explicit client payload views. Latest
  client reselection skips the incoming client by address before payload
  access; file callbacks run over a separate snapshot of owners. Validation:
  26 focused lifecycle/handle tests, 49 client-command conformance cases, and
  lint passed with tmux 3.7b; an earlier 170-test selection also covered the
  newly integrated layout changes.

- Server dispatch client views (items 1, 3, 6, 9): access updates,
  session/window redraws, locking, shutdown, and queue dispatch request
  explicit client views while retaining owners across operations. Validation:
  nine focused tests, 41 command conformance cases, and lint passed with
  tmux 3.7b.

- Resize client views (items 1, 3, 6, 9): sizing policies use explicit
  shared client views and shared client-window map lookups, eliminating two
  unnecessary mutable lookups during size calculation. Status flag updates
  request a mutable view explicitly. Validation: 90 focused tests, 33 resize
  and refresh command conformance cases, and lint passed with tmux 3.7b.

- Window-side client views (items 1, 3, 6, 9): focus, control output,
  pane-input setup, terminal colour, and theme paths request explicit shared
  or mutable client views. Validation: 480 focused tests, 52 command
  conformance cases, and lint passed with tmux 3.7b.

- Formatting client views (items 1, 3, 6, 9): format callbacks, loops,
  target setup, and per-client job storage request explicit payload views;
  user-name caching requests mutation explicitly. Validation: 76 focused
  tests, 32 formatting/listing command conformance cases, and lint passed
  with tmux 3.7b.

The combined server, resize, window, and formatting migrations passed the full
gate at `9089b7343`: 3,136 unit tests, ten doctests, all 1,279 active conformance
cases, and 3,274 leak tests passed with tmux 3.7b.

- Terminal client views (items 1, 3, 6, 9): terminal timers, I/O callbacks,
  draw setup, and terminal-description lookups request explicit client views.
  Validation: 58 focused tests, 25 refresh/send-keys command conformance cases,
  and lint passed with tmux 3.7b.

- Popup and client-mode views (items 1, 3, 6, 9): previews, formatting,
  detach actions, popup completion, and overlay updates request explicit
  client payload views. Validation: 35 focused tests, 15 popup/client-mode
  command conformance cases, and lint passed with tmux 3.7b.

- Input and I/O client views (items 1, 3, 6, 9): input requests, clipboard
  output, control callbacks, file events, and key-binding dispatch request
  explicit client views. Validation: 264 focused tests, 48 command conformance
  cases, and lint passed; 137 input tests and lint passed again after narrowing
  the request-client selection views. Tests used tmux 3.7b.

- Remaining helper client views (items 1, 3, 6, 9): alerts, configuration,
  options, sorting, status, and spawning request explicit payload views;
  removed the unused unchecked notification client getter. Validation after
  upstream spawn integration: 387 focused tests, 66 command conformance cases,
  and lint passed with tmux 3.7b. The name-format conformance fixture now names
  its initial window explicitly so startup process naming does not affect the
  assertion; that narrowed case also passed with hmux0 and no longer needs its
  old ignore annotation.

- Core client test views (items 1, 3, 9): shared fixtures, control and
  server tests, leak regressions, and terminal tests use explicit client views;
  address-only fixtures use pointers or weak handles directly. Validation:
  335 focused tests and lint passed. Production also compiled successfully
  with client coercions temporarily removed before migrating test callers.

- UI client test views (items 1, 3, 9): pane-number overlays, menus,
  customization/client modes, widget/tree tests, and redraw fixtures use
  explicit client payload views or address-only handle access. Validation:
  175 focused tests and lint passed.

- Command coverage client views (items 1, 3, 9): switching, I/O, redraw,
  layout, and command-coverage tests use explicit client views; pointer-only
  uses access handle addresses directly. Validation: 133 focused tests and
  lint passed.

- Explicit client payload access (items 1, 3, 6–7, 9): `ClientRef` no longer
  implements safe `Deref`/`DerefMut`, and its unused unchecked closure accessor
  is gone. Shared and mutable coercions compiled before this change and now
  fail their compile-fail checks. Validation: 3,140 unit tests, 12 compile-fail
  doctests, all 1,279 active conformance cases, lint, and 3,278 leak tests passed
  with tmux 3.7b. Raw engine borrowing remains a separate audit.

- Library unsafe-operation gate (items 5, 8, 10): the library now denies
  `unsafe_op_in_unsafe_fn`. Forty formatting payload-view calls and two calls
  in key-table/terminal helpers now have explicit unsafe blocks; the compiler
  rejected these 42 operations before the fixes. Validation: 76 focused format
  tests, 3,140 unit tests, 12 compile-fail doctests, all 1,279 active conformance
  cases, lint, and 3,278 leak tests passed with tmux 3.7b. The gate enforces
  explicit unsafe regions; it does not establish their soundness.

- Global collection thread ownership (items 2, 5–6, 9–10): `GlobalTree`
  and `GlobalQueue` now claim their owning thread through `OnceLock` before
  accessing their `RefCell`. Other threads are rejected before collection
  access; same-thread nested borrows retain checked behavior. Three new tests
  failed before the change; all seven isolated collection tests now pass,
  including under ThreadSanitizer with an instrumented standard library.
  Full validation: 3,143 unit tests, 12 compile-fail doctests, all 1,279 active
  conformance cases, lint, and 3,281 leak tests passed with tmux 3.7b. The
  collections still have global lifetimes; this enforces thread ownership and
  does not provide multiple independent server instances.

- Repeatable collection race-detection gate (item 10):
  `make check-collection-memory` builds the actual collection module and its
  tests through a standalone manifest, with ThreadSanitizer and a matching
  instrumented standard library. All seven tests passed. The gate requires
  nightly Rust and `rust-src` and does not claim coverage of the full engine.

- Environment output snapshot (items 3–6): `show-environment` renders
  owned lines before sending output, so output callbacks do not retain a
  borrow into the selected environment. Validation: 11 focused tests, seven
  command conformance cases, and lint passed with tmux 3.7b.

- Environment mutation preparation (items 4–7): `set-environment`
  validates names, target selection, flags, and expanded values before applying
  a typed set, clear, or unset operation. Mutation no longer includes error
  output. Validation: five focused tests, six command conformance cases, and
  lint passed with tmux 3.7b.

- Starting environment confinement (items 2–7, 9): removed the raw global
  environment pointer and its one-element global queue. Checked thread-local
  borrows now cover initialization, command mutation, parser assignment, lookup,
  session inheritance, and cleanup. Output and expansion callbacks receive owned
  values after the borrow ends. Tests cover thread isolation, conflicting reentry
  and recovery, and cleared session variables suppressing global format fallback.
  Validation: 238 focused tests plus the format-fallback regression, 13 command
  conformance cases, 3,154 unit tests, 12 doctests, 1,279 conformance cases, lint,
  and 3,292 leak tests passed with tmux 3.7b.

Remaining structural inventory in `src/`: five `Rc<UnsafeCell<_>>` handle
families; two unsafe `Sync` implementations (`GlobalTree` and `GlobalQueue`),
now backed by synchronized thread-ownership checks; 8 statics using those
collection wrappers. These counts locate remaining
boundaries and do not measure whole-program soundness. Mutable server globals,
raw owning/observational edges, and broad lint exemptions still require audit.

`SessionRef`, `WindowRef`, and `ClientRef` now separate opaque ownership from
explicitly unsafe compatibility views. Target state can record a weak observation
directly from an owner without borrowing its payload. These payloads still use
`UnsafeCell`: the explicit views and existing raw back-pointers require an
aliasing and callback-scope audit; this change does not claim checked borrowing
throughout the engine.

The central entity handles, remaining global trees/queues, callback migrations,
input-boundary audit, fixture cleanup, and broader instrumentation remain open.

This report recommends incremental improvements to the active hmux implementation.
It is based on source inspection, not a completed soundness audit, benchmark, or
test run. Priorities reflect the breadth of the affected invariants and the
potential consequences of getting them wrong.

The code already has substantial ownership work: strong and weak handles,
`Drop` implementations, registered pane ownership, owned buffers, and callbacks
that resolve IDs. The next step is to make those boundaries enforce lifetime,
mutation, and teardown rules throughout their callers.

## Memory safety

### 1. Make shared entity handles enforce borrowing — highest priority

**Current boundary:** `ClientRef`, `SessionRef`, and `WindowRef` in
[src/types.rs](src/types.rs) remain cloneable `Rc<UnsafeCell<...>>` wrappers,
but no longer expose safe `Deref` or `DerefMut` coercions. Their explicit unsafe
payload views and raw engine access still need auditing. Reference counting
keeps allocations alive; it does not establish safe mutation. The completed
slices above record specific callback and aliasing fixes, not a whole-engine
soundness result.

**Direction:** Separate ownership from access. Use checked borrow guards or
operations mediated by the entity's owner, and keep any unavoidable unchecked
access explicitly unsafe. Audit both shared and mutable access, including raw
pointer escape paths. A closure-based accessor alone is insufficient if its
callback can reenter and mutate the same entity.

`UnsafeCell` does not relax the uniqueness requirement for mutable references,
including in single-threaded code. See the
[Rust aliasing documentation](https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html#aliasing-rules).

**Completion evidence:** Safe callers cannot obtain conflicting entity borrows
through cloned handles. Focused tests exercise nested access and callback
reentry; compatible isolated tests run under Miri. Borrow checking must not
merely turn normal server workflows into borrow panics.

### 2. Enforce the single-threaded state boundary — highest priority

**Current boundary:** [src/tree.rs](src/tree.rs) provides global collection
wrappers whose unsafe `Sync` implementations are now backed by synchronized
thread-ownership checks before access. Eight wrapper statics remain, alongside
mutable server globals in [src/server/run.rs](src/server/run.rs). Thread checks
prevent cross-thread collection access but do not replace an explicit server
owner or establish safe access to the remaining raw globals.

**Direction:** Put daemon state under an explicit server owner, migrating one
registry or subsystem at a time. Where ambient access remains necessary, use
an access mechanism that enforces thread confinement. Audit unsafe `Sync`
implementations and avoid granting thread safety solely because current callers
are expected to serialize themselves. Preserve runtime initialization after
fork and explicit shutdown ordering.

**Completion evidence:** Access to migrated state requires the appropriate owner
or thread context. Independent instances and tests do not share that state
accidentally. No unchecked `Sync` assertion is needed for those collections.

### 3. Complete the ownership graph and callback lifetime migration — high priority

**Starting point:** Sessions and windows have both owning/weak handles and
pointer lookup registries. Panes have `RegisteredPane` ownership but remain
reachable through raw pointers. ID-based observation in
[src/plugin/host.rs](src/plugin/host.rs) and callback adapters in
[src/job.rs](src/job.rs) provide useful existing patterns.

**Direction:** For each relationship, identify its role: sole owner, shared
owner, temporary borrow, or observation of a possibly destroyed entity. Use
weak handles or validated IDs for deferred observation and strong handles only
when work must keep an allocation alive. Distinguish an allocation that still
exists from an entity that remains active in the server. Audit ID wraparound
and stale-event handling before relying on IDs as lifetime protection.

**Completion evidence:** Destroying a pane, client, session, or job while work is
queued cannot leave a callback using freed storage or acting on a replacement
entity. Strong-reference cycles and redundant ownership registries are removed
where their callers no longer require them.

### 4. Separate observable teardown from resource release — high priority

**Starting point:** `SessionStorage`, `WindowStorage`, and `ClientStorage` already
implement `Drop`; reactor shutdown explicitly accommodates teardown that calls
back into the runtime. Cleanup is consequently more than freeing allocations.

**Direction:** Keep protocol-visible lifecycle transitions explicit: unlinking,
notifications, cancellation, and client detachment should happen at defined
points. Let `Drop` release the remaining owned resources, with clear dependencies
and no requirement to rediscover a partially destroyed server. Complete remaining
resource ownership migrations using the appropriate owner for objects, arrays,
strings, and descriptors. Verify allocation provenance before changing any
deallocator; wrapping an arbitrary C allocation in `Box::from_raw` is not a
valid general migration.

**Completion evidence:** Normal destruction, partial initialization failure,
callback cancellation, and server shutdown each release resources once while
preserving observable notification order. Lifecycle tests and the existing leak
gate cover the changed paths.

### 5. Contain unsafe memory operations behind validated boundaries — high priority

**Starting point:** The crate has centralized FFI declarations in
[src/ffi.rs](src/ffi.rs), an owned segmented buffer in
[src/reactor/buffer.rs](src/reactor/buffer.rs), and many remaining unsafe engine
functions. Buffer access, wire-message handling, and terminal input are useful
boundaries for further work.

**Direction:** Validate lengths, arithmetic, initialization, alignment, and
ownership at the boundary. Pass byte slices and owned values into the engine
where possible, retaining byte-oriented behavior rather than assuming UTF-8.
Shrink unsafe regions only after their reference and lifetime invariants are
established. Put safety contracts in rustdoc and progressively enable stricter
lints for migrated modules.

**Completion evidence:** Malformed, truncated, oversized, and fragmented input
has defined behavior. Each remaining unsafe boundary has a small reviewable
contract. Safe engine callers cannot invalidate it by ordinary API use.

## Maintainability

### 6. Organize modules around ownership and invariant-preserving operations — high priority

**Starting point:** Command and server code already have subsystem directories,
while entity state and compatibility types remain spread across
[src/types.rs](src/types.rs), [src/window.rs](src/window.rs), and their callers.
Session membership operations still cross the session/window boundary.

**Direction:** Give each entity module responsibility for its state and the
operations that must change related fields together. Make implementation fields
private as callers migrate. Move compatibility exports into a thin transitional
layer. Group code by the invariant it maintains; simply splitting a large file
or creating getters for every field does not establish encapsulation.

**Completion evidence:** A change to session membership, pane ownership, or
client lifecycle can be understood and reviewed within a bounded subsystem.
Callers cannot bypass the operations that maintain registry and entity consistency.

### 7. Finish semantic migrations and remove their scaffolding — medium priority

**Starting point:** The tree combines Rust-owned state, transitional raw-pointer
views, old naming, broad type reexports, and C-shaped control flow. Some README
descriptions, including package naming and module layout, lag the current code.

**Direction:** Migrate one concept end to end: its storage, callers, cleanup, and
tests. Then remove obsolete adapters, duplicated constants or declarations, and
unused compatibility machinery. Replace sentinel combinations with enums or
`Option` where that makes invalid states harder to represent. Simplify control
flow after characterization tests establish ordering and error behavior.

**Completion evidence:** Each migration leaves fewer representations and a clear
authoritative implementation. Documentation describes the resulting architecture
and the remaining transitional boundaries accurately.

### 8. Use existing traits as meaningful capability boundaries — medium priority

**Starting point:** The project already exposes entity capability traits,
`CommandParser`, plugin contracts, and runtime contracts. These are versioned
compatibility commitments, unlike ordinary public implementation structs.

**Direction:** Improve implementations and route callers through the appropriate
existing capabilities. Keep engine-only operations private when they do not need
a public contract. Preserve distinct parser operations and attached/control-mode
paths when their contexts require different semantics. Keep optional capabilities
separate from `TmuxServer`.

**Completion evidence:** Boundaries have clear responsibilities and consumers
depend on only the capabilities they need. Any proposed change to an existing
public trait, including its semantics, is separately reviewed and receives human
signoff before implementation. Trait proliferation is not a success metric.

### 9. Make tests independent and focused on invariants — high priority

**Current boundary:** [src/tests/test_fixtures.rs](src/tests/test_fixtures.rs)
provides global-state guards and owned builders. The standard unit gate now runs
nextest tests in separate processes, and those tests pass without initialization
by other tests. Process isolation remains necessary for ambient server globals;
some fixture entities intentionally bypass production construction.

**Direction:** Complete fixture initialization and cleanup so every test establishes
its own preconditions. As state acquires an owner, give tests fresh instances.
Pair lightweight fixtures with targeted production-lifecycle tests. Prioritize
destruction during callbacks, cancellation, registry consistency, error paths,
and ordering over tests that merely repeat accessor implementations.

**Completion evidence:** Tests pass individually and in different execution
orders. A failure can be reproduced without first running unrelated tests.
Production construction and destruction receive direct coverage.

### 10. Measure safety and maintenance progress alongside conformance — medium priority

**Direction:** Extend existing gates with targeted memory-error detection and
fuzz/property testing where they provide evidence the conformance suite cannot.
Use Miri for compatible isolated Rust components and address-sanitized integration
runs where supported; establish toolchain and FFI limitations before depending on
either. Fuzz the existing parsers and buffer operations without adding a second
engine. Keep the current leak gate: allocation lifetime and aliasing require
different evidence.

Track remaining unchecked handle-access boundaries, ambient mutable registries,
raw owning edges, test-order dependencies, and broad lint exemptions. Pair those
counts with the invariants each change now enforces. Lower unsafe-line counts,
more traits, or more tests alone do not prove improvement.

**Completion evidence:** Every migration has relevant regression evidence and
retains the tmux-facing contract. For implementation changes, run focused checks
during development and the matching full `make test` gate from the parent
repository before completion, plus applicable lint and leak gates. Reject a
reference tmux other than 3.7b. Preserve the hmux profile's execution of ignored
conformance cases and its explicit exclusion backlog.

## Suggested sequence

1. Audit and fix shared-handle borrowing and global collection thread safety
   first, supported by focused tests (items 1, 2, and 9).
2. Complete one representative entity lifecycle through callbacks and teardown
   before expanding the pattern to others (items 3 and 4).
3. Encapsulate the migrated entities and remove their transitional machinery
   (items 6–8), while tightening unsafe input boundaries (item 5).
4. Add regression measurements as each boundary becomes testable (item 10).

Preserve chosen foundational components and their distinct operations. Replacing
one or adding a competing implementation requires human signoff. Document narrow
observable differences in README.md and characterize them explicitly. The aim is
to make existing behavior easier to reason about and safer to change.
