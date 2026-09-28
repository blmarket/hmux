# Reference-counted entity and holder inventory

Reviewed 2026-09-27. This is the tracking list for migrating model pointer holders.
It distinguishes the allocated entity from each individual holder: the same
entity can have owning, weak, and borrowed holders simultaneously.

## Classification rules

- **Owning:** construction, cloning, or an upgrade supplies a strong reference;
  the holder releases that reference on completion, cancellation, or destruction.
  A corresponding `remove_ref`, `unref`, or consuming `free` is evidence of an
  owning holder. Follow the actual value passed to release, not just its type.
  Use `Rc`, `Option<Rc>`, or the existing release-policy guard.
- **Weak:** stored observation across operations without a matching strong release.
  Use `Weak` and retain the upgraded owner throughout access. Weak expiry detects
  allocation destruction, not logical destruction: still check dead/destroyed flags
  and membership where required.
- **Borrowed:** access is bounded by a caller's owner or a guard. Prefer `&Rc`,
  `&T`, `&mut T`, or a borrow guard according to actual aliasing guarantees.
  A raw pointer projected from an owner is only a borrowed view; it must not outlive
  that owner. In particular, immediately dropping an upgrade after extracting its
  pointer does not make subsequent pointer use safe.
- A function named `free` may perform logical teardown without releasing every
  strong reference. Conversely, ordinary Rust `Drop` releases an owner without a
  named release function. Do not infer ownership from names alone.
- `Rc<UnsafeCell<T>>` makes retention explicit, but does **not** make simultaneous
  mutable access safe. Replacing every pointer with `&mut T` would be incorrect.

## Application entities (12)

| Entity | Owning holders and release evidence | Weak holders / borrowed views | Tracking status |
| --- | --- | --- | --- |
| `session` | Session index; event targets/payloads; monitor session; run-shell data; spawn context. `session_remove_ref` consumes an `Rc` and defers release. | Self observer, find state, monitor changes, mode selections; group membership; client session/last-session and winlink session are weak; format context is weak. The current winlink field is a weak RefBox observer. | Owners typed; raw relationships and traversal projections remain. |
| `window` | Creation returns `WindowOwner`; winlinks own `WindowOwner`; alerts and event payloads retain `Rc`. `window_remove_ref` performs live close notification before release. | Self observer, find state, menus, monitor changes; pane parent and format/redraw context. Global index is weak. | **Migrated:** index entries, index traversal, ID lookup and all their callers retain `WindowOwner` while accessing the window. |
| `window_pane` | Pane index; retained event targets/payloads. `window_pane_remove_ref` consumes an `Rc`; logical destruction removes the index owner. | Pane lists/history, find state, mode tree, prompts and monitor changes; window active/modal/zoom panes use weak handles; raw layout cells, input and render contexts remain. | Owners and many observers typed; remaining raw relationships and short-lived upgrade projections need migration. |
| `client` | Client registry; queue items; prepared commands; format trees; popups; asynchronous command data; file records; queued key events and event payloads. `server_client_unref_owned` defers a consumed owner. | Self observer, target-client/find state, monitor, prompt and key-event observers, window latest-client identity; raw tty parent, input/render/format contexts. | Owners typed; borrowed constructor migration in progress. Preserve deferred release. |
| `client_file` | Client file index plus completion/retry captures. Completion unlinks index ownership even on cancellation; ordinary `Rc` drop frees the record. The record itself owns its client reference. | Self observer and command wait-file observer; callback pointers borrow a retained record. | Typed retention already present; avoid reintroducing file/client cycles. |
| `cmd_list` | Parse results, argument values, options, key bindings, queue items, prepared commands and confirmation data. Owners clone/move `Rc`; final drop cleans commands. | `BorrowedCommand(&Rc<...>)`, binding accessors and synchronous command processing. | **Migrated:** `Rc<RefCell<cmd_list>>`; checked borrows replace all raw list access in production and tests. |
| `cmdq_state` | Queue-state constructors and queue items hold `Rc<cmdq_state>`; ordinary field drop releases formats and target observers. | State accessors retain owners; target and format mutation use scoped `RefCell` guards; events and hook targets use snapshots. | **Migrated:** no raw queue-state API or unchecked state access. |
| `key_table` | Table index, client keytable and dispatch-held handles. Index removal returns an `Rc`; final drop releases bindings. | Binding/search views use checked guards borrowing retained table handles. | **Migrated:** `Rc<RefCell<key_table>>`, retained index results and numeric customization identity; no raw table API or self-observer. |
| `mode_tree_data` | Buffer/client/tree/customization modes and prompt records retain `Rc`. `mode_tree_free` consumes the mode's reference and marks it dead; outstanding prompt owners may remain. | Self observer; transient draw/input callbacks borrow a live retained allocation. Pane link is weak. | Typed owners already present; preserve logical-dead checks. |
| `window_tree_modedata` | Mode entry's `Rc<dyn Any>` plus typed prompt/queued command captures. Mode free removes entry ownership; callbacks may retain allocation. | Self observer; mode callback's erased data pointer is a borrowed view of the stored owner. | Typed ownership; erased callback ABI remains. |
| `window_customize_modedata` | Mode entry's `Rc<dyn Any>` plus typed editor/prompt captures. Mode free removes entry ownership. | Self observer; erased mode callback pointer borrows the allocation. | Typed ownership; erased callback ABI remains. |
| `hyperlinks` | Screens own `HyperlinksRef(Rc<RefCell<hyperlinks>>)`; `hyperlinks_copy` clones and `hyperlinks_free` consumes it. | Global history holds `Weak<RefCell<hyperlinks>>`; entry lookup returns a `Ref` guard. | Already uses checked borrows; keep this pattern. |

Definitions and evidence: `src/shared/{client,session,window,pane,command,key,mode_tree,arguments,events,monitor,format}.rs`,
`src/{session,window,server_client,file,events_payload,mode_tree,window_tree,window_customize,hyperlinks}.rs`,
`src/cmd/{core,queue}.rs` and `src/key_bindings.rs`.

## Infrastructure also using Rc

These are reference-counted entities too, but are separate from the 12 application
models and should not be accidentally folded into the model migration.

| Entity or family | Owners | Observers / borrows |
| --- | --- | --- |
| Reactor `EventState` | Event registry and executing/scheduled tasks | Callback access borrows retained state; cancellation marks state not live. |
| Reactor `StreamState` | Stream registry and executing tasks | Buffer registry and task observers use `Weak`; state owns boxed `bufferevent`. |
| Reactor `Descriptor`, `OwnedFd` | Active I/O work and descriptor leases | Descriptor cache uses `Weak`; raw file descriptor integers do not own a descriptor. |
| Runtime `Core` | Runtime, handles and driving guard | Tasks, I/O, signal and wait records use `Weak`. |
| Runtime `TaskState`, `TaskWake` | Task registry/handles; local waker owns `TaskWake` | Ready queue and wake targets use `Weak<TaskState>`. |
| Runtime `IoState`, `SignalState` | I/O and signal handles | Runtime indexes use `Weak`. |
| Callback values | Event/bufferevent callback slots, monitor/event sinks, job update slots and dispatch clones | Borrowed through `RefCell` or invoked through retained `Rc<dyn Fn...>`. |

See `src/reactor/`, `src/shared/{event,events,monitor,job}.rs` and `hmux-rt/src/mio/`.
Signal delivery additionally uses `Arc<OwnedFd>` and `Arc<AtomicBool>`; these are
thread-safe resources, not model `Rc` holders. Test-only counters and capture
fixtures are not production entities.

## Related entities that are not shared Rc models

`winlink`, pane mode entries, prompts, menus, mode-tree items and several mode item records use
`RefBox` with `refbox::Weak`. Their ownership is **single-owner**, not shared `Rc`.
Sessions own boxed session groups; tty terms, format trees, jobs, queue items,
options, layouts and input contexts also have their own non-Rc storage contracts.
Some of these objects *contain* Rc owners, which does not make the enclosing
object reference-counted. Track those contained holders under the entity above.

## Implemented window-index migration

`WindowIndex` is now `BTreeMap<u_int, Weak<UnsafeCell<window>>>`.
Insertion borrows an existing `Rc` and downgrades it. Lookup and traversal upgrade
into `WindowOwner`, preserving last-release notification even if callbacks remove
the previous owner. All 14 global traversal loops retain current owners, and child
exit traversal retains the next owner before callbacks can remove it. Five ID
lookup consumers also keep the returned owner for their operation.

Index removal compares weak pointer **identity only**: final destruction runs after
strong count reaches zero, so upgrading there would fail. The weak control block
prevents address reuse while its identity remains stored. Removal does not create
or release a strong reference. No ownership is reconstructed from an address.

Legacy operations still accept borrowed raw model views within these owner scopes.
This pass does not claim that all model pointers or aliasing hazards are eliminated.
The raw relationships listed above remain the next migration worklist.

Validation: `tests/window_storage.rs` covers nonowning registration, retained lookup,
ordered traversal, duplicate-ID rejection, wrong-index/wrong-allocation removal,
unlink without destruction, and final-drop unlink with an expired weak reference.
Existing window/event tests cover live close notification and reentrant retention.

`cargo check --workspace` and `cargo test --workspace` passed after this migration.

## Implemented session-group membership migration

Group insertion borrows an existing `Rc`; new-session retains its target owner
through insertion. Group membership stores `Weak<UnsafeCell<session>>`. It does not add strong
references to globally owned sessions. Membership snapshots upgrade into a vector
of `Rc` owners, preserving insertion order and skipping expired allocations.
Formatting, tree display, synchronization, redraw and group destruction retain
those owners for their accesses, including across group removal by callbacks.
Membership identity comparisons do not dereference weak addresses.

The unit test checks that membership does not retain allocations, expiry is skipped,
attachment counts remain correct, and a snapshot outlives the group and original
owners. The process integration test checks group order and shared windows through
member removal. Raw session APIs and accesses within these scopes still remain;
this is progress toward the full no-raw-entity-usage goal, not completion of it.

Session-group validation: workspace tests passed; after the typed-insertion change,
`cargo test --test session_group_membership --test session_group_storage` also passed.

## Implemented command-list access migration

All command-list owners now use `Rc<RefCell<cmd_list>>`. Construction, parse results,
argument values, options, bindings, confirmation data and queue items agree on that
type. List append/move APIs borrow the owner handles and use `Rc::ptr_eq` for
self-operation detection before taking conflicting borrows. Readers take ordinary
references backed by `Ref` guards. Copying, hooks and parser grouping keep borrows
bounded so subsequent mutation can obtain a checked mutable borrow.

Command-list tests use `Weak::ptr_eq` for allocation identity, without reconstructing
or projecting raw list addresses. Existing command boxes and their command callback
APIs remain distinct from the reference-counted list: queue construction retains the
list owner and borrows the list while obtaining its command entries.

`cargo test --workspace` passed. Source review and searches across `src/` and `tests/`
found no `*mut cmd_list`, `*const cmd_list`, or `UnsafeCell<cmd_list>` declarations;
command-list reads and mutations now pass through checked borrows. This completes
the list-access migration, not the overall goal: other entities still use raw APIs.

## Implemented key-table access migration

Key-table indexes, clients, dispatch and customization now retain
`Rc<RefCell<key_table>>`. Lookup, insertion, removal and ordered traversal return
owners rather than raw table pointers. Identity checks use `Rc::ptr_eq`, so removing
an unindexed allocation with the same name cannot remove the indexed table.
The former raw-pointer self-observer and pointer-view accessor are removed.

Dispatch borrows only long enough to snapshot its binding and releases that borrow
before executing commands or changing tables. Listing keeps read guards alongside
its retained table vector; customization releases mutable guards before rebuilding
the mode or installing prompts, and releases read guards before resetting bindings.
Customization table sections use a monotonically allocated numeric identity in a
separate tag namespace instead of deriving their tag from a table address.

The index regression test covers duplicate names, distinct allocation identities,
ordered traversal, retained lookup lifetime and final index cleanup. Existing tests
exercise binding reset/replacement, dispatch after removal, listing and customization.
The full no-raw-entity-usage goal remains open for other model types.

Key-table validation: `cargo test --workspace` passed, including the new index
regression. Source checks found no raw `key_table` pointer types,
`UnsafeCell<key_table>` storage, or `key_table_owner_ptr` accessor in `src/` or `tests/`.

## Implemented command-queue state access migration

Queue items, configuration loading and hooks use `Rc<cmdq_state>` directly.
Flags and event metadata are read through ordinary references. Mutable target and
format fields use `RefCell`, allowing independent access without a whole-state
mutable borrow. Copy/add-format APIs borrow the typed state rather than projecting
an address. Normal field destruction replaces the raw-pointer cleanup function.

`cmdq_get_state_owned` retains the state for command handlers. Target mutations
borrow only for their individual operation. Target resolution and hook delivery
use snapshots; `current_snapshot` ends the borrow before calling event listeners.
`cmdq_get_event` returns a metadata snapshot instead of a pointer into shared state.
Legacy mouse/target APIs still operate on local snapshots; their contained model
observers are part of the remaining client/session/window/pane migration.

The queue-state regression covers shared target updates, conflicting-borrow
rejection, independent event reads, detached snapshots and retention after the
queue item releases its owner. Workspace tests passed; the subsequent hook-borrow
scope fix is checked with queue-state, mode teardown and session-group tests.

## Implemented key-event and latest-client observers

`key_event.client` is now `Option<Weak<UnsafeCell<client>>>`. Absence selects the
queue client; an expired explicit target skips dispatch rather than redirecting
input. Metadata snapshots clone the weak observer. `resolve_client` upgrades to an
owner retained through dispatch. `QueuedKeyEvent` continues to own and defer-release
its separately retained client; snapshot creation does not alter that contract.

`window.latest` now stores `Weak<UnsafeCell<client>>`, replacing the erased `c_void`
address. Selection, spawning, client activity and loss handling assign or compare
weak identities. Resize filtering uses `Weak::ptr_eq`; it does not dereference an
expired client or confuse its identity with a subsequently reused allocation.

Regression coverage checks absent versus expired explicit clients, snapshot weak
retention, retained dispatch lifetime and latest-client identity after expiry.
Other raw client relationships and client function signatures remain pending.

Validation for these client observers: `cargo test --workspace` passed, including
snapshot/expiry coverage and the existing deferred queued-event release test.

## Implemented input client observers

Input contexts and terminal requests now store `Weak<UnsafeCell<client>>`.
`input_init` borrows an optional client `Rc` and downgrades it, so popup setup no
longer passes a raw client address into the context. Clipboard output and request
unlinking upgrade and retain the client throughout access; expired clients are
not dereferenced.

Cancellation accepts a client borrow and drains its request index before releasing
requests. Each detached request loses its client observer, so cleanup does not need
to upgrade a client during final destruction. The index-access helper now returns
a reference bounded by its client argument, replacing the unconstrained lifetime
formerly derived from a raw pointer.

Tests cover context/request cleanup with live and expired clients, absence of
strong retention, cancellation without an upgradeable observer, and existing reply
ordering and cancellation behavior. Input's pane relationship and remaining raw
model function parameters are still pending migration.

Input-client observer validation: `cargo test --workspace` passed, including both
new lifetime/cancellation regressions. `git diff --check` passed.

## Implemented input pane observer

`input_ctx.wp` now stores `Weak<UnsafeCell<window_pane>>`. Construction borrows an
optional pane `Rc`; each input operation upgrades once and retains the owner for
all legacy pane calls within that operation. The parser no longer keeps an
unchecked pane address between operations and does not create a pane/parser cycle.

Input teardown stops synchronized output only when its pane can still be upgraded.
During final pane release, the pane destructor explicitly stops synchronization
after dropping its input context and before freeing screens. This preserves timer
cancellation when the pane strong count has already reached zero.

Workspace tests passed. The subsequently added input regressions also passed
(`cargo test --lib input::`, 14 tests), covering a context outliving its pane and
final pane release cancelling a pending sync timer without a successful weak
upgrade. Raw pane APIs, the embedded screen-write context, and palette borrows
remain work for the full no-raw-entity-usage objective.

## Implemented payload handle and format-job client migrations

Event-payload model accessors return borrowed `Rc` handles instead of model
addresses. Target capture upgrades the existing session/window/pane observers
directly; its winlink fallback borrows the link and clones the window owner. It
no longer projects an address solely to recover an owner from a self-observer.
Monitor payload construction releases its link borrow before target capture, then
dispatches without that borrow. The reentrant-unlink regression now also verifies
that the window, session and window index are actually retained in the payload.
Payload serialization and legacy model setter APIs still require further work.

`format_job.client` stores `Weak<UnsafeCell<client>>`, supplied by borrowing the
format tree's client owner during cache insertion. Status callbacks upgrade and
retain the client while updating it; an expired client is skipped. Jobs do not
create a client/cache ownership cycle. The completion regression verifies output
is recorded and cache cleanup succeeds after the observed client expires.

`cargo test --workspace` passed after resolving the monitor borrow conflict.
The strengthened monitor assertions passed in `cargo test --lib hooks_events_tests`.
The full goal remains open for the raw model APIs and stored relationships listed
above; typed payload accessors alone do not eliminate their unchecked consumers.

## Implemented previous-session observer migration

`client.last_session` is now `Weak<UnsafeCell<session>>`. Attach, switch, new-session
and detach paths preserve its nonowning behavior. The last-session command and
format callback upgrade through `sessions_resolve`, which checks index membership
by `Rc` identity and returns a retained handle. A removed session remains an
invalid target even if queued work still owns it; a new session with the same name
does not satisfy the old observer. The command retains the resolved owner through
its legacy raw session operations.

`cargo test --workspace` passed. `cargo test --test session_storage` then passed
both tests, including the added regression for removal with outstanding owners,
same-name replacement and expiration. Current-session fields and raw session APIs
remain to be migrated.

## Implemented parser client observer migration

`cmd_parse_input.c` is `Weak<UnsafeCell<client>>`, including cloned parser inputs.
Format expansion upgrades once, retains the client through expansion and passes
its handle to the typed format constructor. Prepared commands keep their existing
separate strong client owner and deferred-release policy; they now transfer the
queue-provided owner directly instead of projecting a raw pointer to recover it.
Both configuration loaders accept borrowed optional client owners and create weak
parser observers. Source-file callbacks pass their existing owner directly.

`cargo test --workspace` passed. The added integration regression
`cargo test --test parser_client_observer` passed with initialized format options:
a conditional referencing `client_name` selects the absent-client branch after
the observed client expires. Parser-input clone lifetime coverage also passed in
the workspace run. Legacy find/default-format calls still project a borrowed
client pointer while the local owner remains alive; these APIs remain pending.

## Implemented startup client and directory lookup migration

`cfg_client` now stores a weak client observer. Client loss clears it using weak
identity, and directory lookup retains an upgraded client while copying its cwd.
`server_client_get_cwd` accepts optional borrowed client/session values and returns
`Option<CString>` instead of a pointer into a model allocation. Its consumers in
file resolution, spawning, format jobs and commands retain that returned string.
The existing startup-client precedence and absent-startup-cwd behavior are kept.

`cargo test --workspace` passed, including `client_cwd_owner`, which verifies
precedence, absent cwd, observer expiration and result survival after client
release. The helper still reads the raw `client.session` relationship internally;
its migration and the raw client-registry iteration storage remain pending.

## Implemented client registry traversal migration

The registry's ordered entries and cached successors now store weak client
handles. `first` and `next` return retained optional `Rc` handles; `next` borrows
the current owner. Insertion consumes an owner, while removal and final cleanup
accept weak identity handles. All 142 production first/next call sites were
updated to retain their current client. Shutdown retains both current and saved
successor across client-loss cleanup and transfers the saved owner on advance.
Startup configuration uses the registry-provided owner directly.

Sorted client results are vectors of `Rc` owners. The client chooser, list-clients
and format client loops retain that vector throughout processing; the chooser
and format constructor reuse its handles instead of recovering ownership through
raw pointers. The client comparator borrows client values.

`cargo test --workspace` passed. The registry regression verifies removal order,
cached successors, weak identity cleanup and a saved successor surviving registry
clear until its retained handle is dropped. Registry keys still use address
identity as integers, and legacy operations inside traversal consumers still use
raw projections from retained handles. These remain boundaries for the full
no-raw-model-usage objective, not a claim that client migration is complete.

## Implemented sorted session and pane ownership

Session and pane sort results now contain retained `Rc` handles. Their consumers
in format loops, list commands and tree/switch modes keep those vectors through
processing. Session comparison borrows session values; pane sorting borrows its
window and obtains a retained snapshot of the weak pane order. Neither sorter
returns a vector of raw model pointers.

Next/previous-session selection accepts a borrowed optional session and returns
an optional owner. Selection matches weak allocation identity against the sorted
registered sessions, rejecting removed sessions and retaining the selected result
through switch-client and destroy-session operations.

`cargo test --workspace` passed, including new `sorted_session_owners` and
`sorted_pane_owners` tests. They verify retained results after source/index removal,
wraparound and singleton selection, rejection of removed sessions, and final
release when the sorted results and selected owners are dropped. Legacy pane
comparison/index lookup, raw client-session relationships and raw projections
inside sorted-result consumers remain pending.

## Implemented destroy-session selection ownership

The replacement-session selector borrows its index and excluded session, retains
its current candidate and returns an optional `Rc`. `server_destroy_session`
requires a borrowed source owner and keeps both preferred replacement and fallback
owners through client reassignment and size recalculation. Its callers reuse
existing index/group/target owners or retain the tree-selected session for the
operation. The selection API no longer returns a raw session pointer.

The workspace test suite passed. New selector coverage checks source exclusion,
newest-session preference, detached-session preference, no eligible replacement,
and a selected session surviving index removal until its owner is dropped.
Raw `client.session` storage and the legacy reassignment/physical-session-destroy
APIs still require migration; this changes retention without claiming those
remaining raw operations are checked borrows.

## Implemented pane-order identity migration

All `window_panes` and `window_pane_history` collection APIs are safe Rust methods.
Traversal, position, removal, insertion anchors and swaps use weak allocation
identity instead of raw pane pointers. Removed the helper that upgraded a weak
handle only to return its raw pointer after dropping that owner. Insertion still
requires a live observer; identity cleanup can now remove expired entries without
upgrading any pane. Legacy window wrappers pass the existing pane observer.

Pane index, stacking index and visit-history index accept borrowed panes. Normal
and history indices use typed identity lookup; stacking index retains a snapshot.
The pane-sort comparator borrows pane values, and every index consumer was updated.

`cargo test --workspace` passed. The pane-storage regressions cover typed ordering,
visit deduplication, retained lookup results and removal of expired observers,
including an expired entry preceding the removed entry. Raw pane/window wrapper
APIs, the pane's raw window parent and floating-layout access remain pending.

## Implemented pane ID lookup ownership

Pane-index lookup accepts the numeric ID directly and clones the stored owner;
removed the temporary full pane object previously constructed for lookup.
`window_pane_find_by_id` and its `&CStr` parser return optional retained handles.
Command, control, monitor, editor, prompt and mouse consumers retain those handles
through pane access, including lookups made conditionally inside larger functions.

Mouse-pane resolution, inside-pane discovery and the pane-picker helper chain now
return owners through to their consumers. Tree-row resolution returns retained
session/pane owners alongside its legacy raw output parameters; every caller
keeps that result, including both sides of a swap. Find-state assignment creates
weak observers directly from lookup handles.

`cargo test --workspace` passed. Updated pane-storage coverage then passed all five
tests, including valid/invalid ID parsing and lookup-result survival after index
removal and release of the source owner. Raw index traversal, mouse session/window
outputs, tree output pointers and legacy operations using borrowed projections
remain pending for the full objective.

## Implemented pane-index traversal and numbered selection

Pane-index first/next operations return retained optional handles. Every traversal
consumer holds the current owner; inside-pane discovery transfers that owner
instead of recovering it from an address. Insertion accepts a borrowed index and
returns an owner for duplicate IDs. Removal accepts borrowed index/pane values
and compares weak identity; traversal and removal are safe collection functions.

Pane-number selection borrows its window and returns an owner. Relative pane
selection uses safe weak-order operations, retaining the current pane through each
step and returning the selected owner. Find-state consumers assign weak handles
directly; the pane-picker passes the selected owner through its helper chain.

The workspace suite passed. All seven pane-storage tests subsequently passed,
including retained current/successor results after index removal and relative
selection wraparound, zero-step behavior, empty ordering and final release.
Raw parent fields, other window/pane traversal wrappers and legacy operations
inside retained traversal loops remain pending for the full objective.

## Implemented directional pane selection ownership

Up/down/left/right selection accepts an optional borrowed pane owner, gathers
retained candidate handles from the window's pane snapshot and returns an owner.
The activity comparator retains the first candidate for ties. Select-pane keeps
both source and result through zoom restoration and later operations; directional
find-state branches assign weak handles directly from their retained result.

`cargo test --workspace` passed. The subsequently added
`directional_pane_owners` regression passed, covering absent input, stable ties,
activity preference, horizontal wraparound and retention after order removal.
Direction geometry still uses legacy raw pane/window helpers internally; coordinate
selection and zoomed-pane access still need migration.

## Implemented coordinate and saved-zoom pane ownership

Coordinate lookup borrows a window owner and returns a pane owner, retaining
stacking-order candidates while checking geometry. Named-position lookup accepts
`&CStr` and forwards the owner. Mouse hit testing retains its result through the
rest of dispatch; find-state consumers assign a weak observer directly.
Zoomed-pane lookup borrows the window and returns a retained pane through resize
and zoom operations.

`window.was_zoomed` is now a weak observer. Save/restore preserves its nonowning
behavior; restoration retains the saved or active fallback pane, and pane removal
clears the observer by weak identity. Expired saved panes are never dereferenced.

The workspace suite passed with the coordinate regression's required option
definition initialized. The strengthened `coordinate_pane_owners` regression also
passed, covering coordinate/named selection, misses, result retention after order
removal, nonowning saved zoom, and restoring an expired target. Raw active/modal
fields, parent relationships and geometry helpers remain pending.

## Implemented saved-modal observer and typed pane membership

`window.modal_last` is a weak observer. Modal creation clones the previous active
pane's observer without ownership. Pane loss clears matching saved identity;
restoration upgrades the saved target and retains whichever pane wins the history/
order fallback through notification and focus updates. Expired saved targets fall
back without dereferencing an old address.

`window_has_pane` is now a safe function accepting a borrowed window and weak pane
identity. It requires a live allocation and searches typed pane ordering, removing
its raw traversal. Every membership consumer has been updated.

`cargo test --workspace` passed. The two modal-observer regressions and all eight
pane-storage tests passed, covering expired modal restoration, clearing a lost
saved target, same-ID/different-allocation rejection and expired membership.
Current active/modal fields, raw parent fields and the pane-loss entry API remain
pending for the full objective.

## Implemented current modal-pane observer migration

`window.modal` now stores a weak pane observer. Spawn and loss paths set or clear
that observer; command restrictions and pane flags use weak allocation identity.
Coordinate selection, pane stacking, format output and mouse dismissal upgrade
and retain the modal pane through access. Expired modal observers act as absent
panes. Nullable proposed active panes are rejected without dereferencing null.

`cargo test --workspace` passed. The strengthened coordinate/modal regressions
also passed, covering modal hit-test exclusion, selection after clearing modal
state, observer expiration, modal-loss cleanup and a null proposed active pane.
The current active-pane field and pane/window parent relationships remain raw;
model operations invoked through retained handles still need checked borrowing.

## Implemented screen-write model observers

`screen_write_ctx.wp` is a weak pane observer. Starting a pane context borrows an
`Rc`; pane-aware operations upgrade once and retain that handle through their
legacy pane calls. All constructors were updated, including the constant format
scratch context. Input paths reuse their retained input pane where available.
Redraw and client callbacks capture weak panes and upgrade on invocation, and the
cursor-offset timer captures a weak window instead of an erased raw pointer.

`cargo test --workspace` passed. All six `write_ctx_tests` subsequently passed,
including an independently owned screen continuing after its observed pane is
dropped, live redraw behavior, callbacks not retaining a pane, and expired callback
targets being skipped. The raw screen borrow, palette views, and legacy model APIs
remain pending: this does not authorize a context to outlive a pane-owned screen.

## Implemented asynchronous model callback observers

Pane scrollbar and resize timers and pane stream read/error callbacks now capture
weak pane handles. Client repeat, click and exit timers and peer dispatch capture
weak client handles; window alert timers capture weak window handles. Each callback
upgrades its target and retains the resulting owner through invocation. Expired
targets are skipped. Timer and stream callback entry points borrow the retained
`Rc` where migrated; peer dispatch still projects a raw pointer while retaining
its upgraded owner.

`cargo test --workspace` passed. A regression using production client timer setup
checks that the click timer clears a live client's double-click flag, does not
retain the client, and safely dispatches after its event is detached and the client
is dropped. Scheduling and cancellation behavior remain unchanged. Legacy callback
bodies still project raw pointers from retained owners, and model parent fields,
active-pane storage and checked borrowing remain pending.

## Implemented layout-cell pane observers

`layout_cell.wp` is a weak pane association: cells do not own or release a strong
pane reference. Leaf creation now borrows an `Rc`, and rotation/swap operations
store the pane's weak identity. Serialization, layout application, node conversion
and cell destruction upgrade before pane access. Destruction safely skips an
expired pane. Compatibility layout copies clone the weak association.

Context assignment and fallback assignment retain pane snapshots; fallback leaf
assignment consumes a typed owner iterator instead of a raw pane cursor. Applying
z-order and history changes retains each affected pane through mutation.

`cargo test --workspace` passed, including leaf rebuild/root-collapse regressions
and a new test covering node conversion, weak identity, nonownership and dropping
a cell after its pane expires. Raw layout-cell pointers describe the separate
Box-owned layout tree and remain unchanged. Legacy model entry points and raw
projections from retained pane owners still require further migration.

## Implemented cached pane/status/scrollbar span observers

`RedrawPaneSpan`, `RedrawStatusSpan` and `RedrawScrollbarSpan` now store weak pane
associations. Cached spans do not own panes. Building spans clones the pane's weak
identity; span merging, selection and status-border lookup compare allocation
identity without dereferencing a cached address. Drawing upgrades and retains a
pane through screen/palette access and skips expired targets. Status dirty-flag
inspection also upgrades before access.

`cargo test --workspace` passed. The new regression checks merging adjacent spans
for one pane, rejecting a different pane, nonownership and safely invoking all
three drawing functions after the observed pane expires. Border span pane fields
and scene client/window fields remain raw, as do legacy drawing API projections;
this is not yet a complete redraw-model migration.

## Implemented cached border-span pane observers

All five `RedrawBorderSpan` pane associations now use weak handles. Border building,
merging, adjacency checks and arrow selection compare allocation identity. The
safe border-style selector accepts an optional borrowed weak active-pane identity
and returns an `Rc`, keeping the selected pane alive throughout border rendering.
It preserves explicit-style, active-neighbor and ordered-neighbor priority while
skipping expired observers.

`cargo test --workspace` passed. The added regression covers selection priority,
unrelated active panes, retaining the selected pane, expired candidate fallback,
span identity comparison and complete expiration. No cached redraw span stores a
raw pane pointer now. Scene client/window fields and transient build/draw context
model pointers still remain, along with legacy drawing calls through raw views.

## Implemented draw-context active and marked pane observers

`redraw_draw_ctx.active` and `.marked` now hold weak pane identities. Context
creation clones observers, border-style selection upgrades the active observer,
and arrow/marked-border checks use live weak identity without dereferencing a
stored pane address. The context is clonable but no longer implicitly copied.

`cargo test --workspace` passed, including a regression proving live arrow output,
marked-pane identity, nonownership and unchanged arrow output after the observed
pane expires. Scene and build-context client/window pointers remain pending;
context construction still obtains the active pane from the legacy window field.

## Implemented redraw-scene client observer

`redraw_scene.c` now stores a weak client handle, avoiding an ownership cycle with
the client's cached scene. Drawing helpers upgrade and retain that client before
accessing its terminal. Draw-context setup returns `None` for an expired client;
its caller skips drawing. Scene creation clones the live client's observer.

`cargo test --workspace` passed. A detached-scene regression checks live allocation
identity, client expiration while the scene survives, context-setup rejection and
safe early return in pane, status, scrollbar, border and prompt drawing. The scene
window field, build-context fields and legacy model projections remain pending.

## Implemented redraw-scene window observer and retained pane passes

`redraw_scene.w` now stores a weak window handle. Cache validation compares weak
allocation identity; context setup and border/scene drawing upgrade and retain
the window before accessing it. Expired windows prevent context setup and drawing.
Scene pane passes now iterate retained snapshots for border resets, status updates,
dirty-screen handling and prompts instead of projecting temporary lookup owners
into raw traversal cursors.

`cargo test --workspace` passed. The window-scene regression covers allocation
identity, nonownership, expiration and safe early returns in context setup, scene
drawing and border helpers. Cached scenes and spans now store typed model handles;
build-context client/window fields and legacy function arguments/projections remain
pending for the full migration.

## Implemented borrowed scene-build window handle

`redraw_build_ctx` no longer stores raw model fields. Its unused client field was
removed, and its window field borrows an `Rc` retained by scene construction.
Context setup borrows the client directly. Scene construction retains both models
while building cells, and pane marking iterates a retained z-order snapshot in
reverse order instead of raw predecessor cursors.

`cargo test --workspace` passed. The build-context regression checks inside/outside
cell classification against the borrowed window and confirms that the context
does not add ownership. Legacy entry points and internal model projections still
remain; typed storage alone does not complete the checked-borrow migration.

## Implemented typed redraw entry points

Screen, pane and scrollbar redraw entry points now borrow client/pane `Rc` handles.
The internal draw and scene-draw functions use an optional borrowed pane owner for
whole-window drawing, and scene lookup/construction borrow the retained client.
The server redraw checker receives the registry's client owner directly and uses
a retained pane snapshot for individual redraw calls.

`cargo test --workspace` passed with the complete call chain migrated, including
the existing scene-expiration and rendering regressions. Legacy session/window
relationships and internal projections remain raw; these API changes establish
retained lifetimes but do not yet replace unchecked model borrowing.

## Implemented read-only pane borrows in redraw helpers

Pane coordinate mapping, border-cell marking and border-arrow marking now accept
shared pane borrows. `window_pane_is_floating` also accepts `&window_pane`, with all
callers updated. Two-pane border-color detection borrows the window, inspects a
retained pane snapshot and returns an optional layout direction instead of writing
through a raw output parameter.

`cargo test --workspace` passed after the complete helper/caller migration. These
helpers remain unsafe where they access the separately Box-owned layout tree or
legacy drawing storage. Many callers still obtain their short borrows from raw
model views; removing those sources and adopting checked borrowing remain pending.

## Implemented winlink session back-reference observers

`winlink.session` is now weak: sessions own their window links, while links do not
retain or release sessions. Creation, synchronization, renumbering and spawning
clone the session observer. Membership checks compare allocation identity, and
find-state capture copies the observer directly. Alerts, formatting, events,
window switching and listing retain upgraded sessions while accessing them and
skip session-dependent work for expired targets. Removal clears live session
history and safely tolerates an expired parent during teardown.

`cargo test --workspace` passed. Existing association-order, shuffle/history,
event-target and hook regressions passed with typed fixtures. A new detached-link
regression proves session nonownership and safe removal after session expiration.
The winlink itself remains RefBox-owned; its separate raw access APIs and remaining
client/session and pane/window relationships still require migration.

## Implemented safe session/window membership query

`session_has` now accepts borrowed session and window values and uses guarded
winlink access with weak session identity. It skips dropped links, rejects an
empty/expired session observer, and reports a borrow conflict instead of accessing
an already borrowed link. All membership consumers now pass explicit borrows.

`cargo test --workspace` passed. The new regression covers membership, distinct
allocations with default IDs, expired session back-references and link removal.
Callers still obtaining their borrows from raw client/session and pane/window
fields remain pending; the membership implementation itself no longer traverses
raw model pointers.

## Implemented retained command session candidates

Command session selection now accepts an optional slice of `Rc` candidates and
returns an owned selected handle. Global candidates are retained during selection;
window-filtered candidates no longer use a raw-pointer vector. Target capture keeps
the chosen session alive while resolving its window link. Session ranking borrows
values and preserves unattached preference, recency and first-candidate tie order.

`cargo test --workspace` passed, including ranking regressions and selection from
an explicit candidate list, an empty list and retention after the candidate vector
is dropped. Legacy validity checks and best-client selection still expose raw
model views and remain pending.

## Implemented retained command client selection

Best-client, current-client and named-client lookup now return `Option<Rc<client>>`
(with the existing `UnsafeCell` model wrapper). Best-client lookup borrows the
session and keeps the winning registry owner. Queue target selection, session
target fallback and display-message retain results through their use; client
ranking uses shared borrows rather than raw nullable candidates.

`cargo test --workspace` passed. The new regression covers attached-session
filtering, global fallback, recency, first-candidate ties and selected-client
retention after registry/external owners are dropped. Client session fields and
legacy accesses inside target resolution still require migration.

## Implemented borrowed session liveness and retained event-target resolution

Session liveness accepts an optional shared session borrow and resolves its weak
identity against registry ownership instead of traversing raw session addresses.
The command session-validity helper also borrows its session. Event-target
resolution upgrades and retains session/window/pane handles once for its complete
fallback sequence and copies their observers directly into find state.

`cargo test --workspace` passed. Target-validation coverage now explicitly checks
absent, unregistered, registered and removed-but-retained sessions. Liveness still
requires unsafe access to the global registry; raw parent relationships and legacy
resolution calls remain pending for the full objective.

## Implemented format context client observer

`format_tree.c` now stores a weak context-client association. It remains distinct
from `format_tree.client`, which owns the client used for format jobs. Context
capture clones the observer, and client-dependent callbacks and expression loops
upgrade once and retain the context client through access. Expired context clients
follow the existing missing-client branches.

`cargo test --workspace` passed, including a new regression covering live client
name output, nonownership and missing name/width output after expiration. Format
session/window/pane fields remain raw, and legacy callback bodies still project
raw views from their retained owners.

## Implemented format context session observer

`format_tree.s` now stores a weak session association. Session/client defaults clone
the session observer, and callbacks and expression loops retain upgraded sessions
through access. Expired sessions follow the existing absent-session branches;
client-default capture can replace an expired session observer.

`cargo test --workspace` passed. The new regression exercises production session
capture, live name output, nonownership and missing name/ID output after expiration.
Format window/pane fields remain raw, and callback internals still contain legacy
projections requiring further migration.

## Implemented format context window observer

`format_tree.w` now stores a weak window association. Default capture clones a
live observer or clears it for a null input. Window-dependent callbacks and
expression loops upgrade and retain the window while accessing it; expired
windows follow the existing absent-window branches. Pane/link default capture can
replace an expired window association.

`cargo test --workspace` passed. The new regression covers production default
capture, live window-name output, explicit clearing, nonownership and absent
name/ID output after expiration. The format pane field and winlink view remain
raw, along with legacy callback projections.

## Implemented format context pane observer and retained getter

`format_tree.wp` now stores a weak pane association. Pane-dependent callbacks and
expression loops upgrade once and retain the pane through screen/mode access.
`format_get_pane` borrows the tree and returns an optional owner; copy-mode
callbacks retain that result and return no value for an expired pane.

`cargo test --workspace` passed. The new regression covers production pane capture,
live ID/width output, selected-owner retention, final expiration and missing output
afterward. All four Rc model fields in format contexts now use weak handles.
Winlink/queue views, internal raw projections and checked borrowing remain pending.

## Implemented spawn-context target-client owner

`spawn_context.tc` now owns an optional client `Rc` for the operation's duration.
Command contexts clone their existing retained target/client handles; editor
startup upgrades its caller's observer. Pane spawning keeps a local owner when
using the context client, and window latest-client state receives only a weak
observer. Context destruction releases its temporary ownership.

`cargo test --workspace` passed, including a new regression proving that a context
retains its target after the external owner is dropped and releases it when the
context ends. Spawn session/source-pane fields and legacy model projections remain
pending.

## Implemented spawn-context source-pane owner

`spawn_context.wp0` now owns an optional pane `Rc`. Split/respawn/editor contexts
capture a live owner. Window respawn retains its selected source pane while
removing it from ordering, rebuilding the layout and reinserting it; pane spawning
keeps a local owner throughout reset, callback and creation work.

`cargo test --workspace` passed. The context-lifetime regression now verifies both
client and source-pane retention after external owners are dropped and release
when the context ends. Spawn session storage, layout/winlink views and legacy raw
projections remain pending.

## Implemented spawn-context session owner

`spawn_context.s` now owns an optional session `Rc` for the spawn operation.
Command/editor contexts retain their session, and window/pane spawning keeps a
local owner throughout access. Context destruction releases this temporary owner.

`cargo test --workspace` passed. The context-lifetime regression now verifies
session, client and source-pane retention after external owners are dropped and
release when the context ends. All three Rc model fields in spawn contexts now
use owning handles. Layout/winlink views and legacy raw projections remain pending.

## Implemented mode source-pane observer

`window_mode_entry.swp` now stores a weak pane association. Mode entries did not
retain/release this source, and copy mode owns an independent screen snapshot.
Initialization upgrades and retains the source while copying; synchronization and
refresh do the same while reading. An expired source prevents initialization or
later refresh, preserving an existing snapshot without accessing freed pane data.

`cargo test --workspace` passed. The incremental-copy regression now uses an Rc
pane and verifies source expiration, disabled synchronization/refresh and retained
snapshot contents. The owning-pane backreference, mode API raw parameters and
legacy raw projections remain pending.

## Implemented retained pane arguments for mode creation

`window_pane_set_mode` now borrows an Rc target pane and an optional Rc source
pane. Callers keep these owners alive through mode initialization and callbacks;
the stored source association is downgraded directly to Weak. Copy-mode commands
upgrade their saved source target directly, removing the intermediate raw source
lookup. Modes without a source pass `None`.

`cargo test --workspace` and `git diff --check` passed. All mode-creation callers
compile against the typed pane arguments. The mode's owning-pane backreference,
other raw callback arguments and internal model projections remain pending.

## Implemented mode parent-pane observer

`window_mode_entry.wp` now stores a weak parent-pane association, avoiding a cycle
with the pane-owned mode stack. Mode consumers upgrade once and retain the parent
through access; required live-parent callbacks explicitly check that invariant.
Stack traversal returns no successor for an expired parent. Source/parent identity
comparisons use weak identity directly without recovering raw model addresses.
Normal logical pane destruction retains its registry owner through mode cleanup.

`cargo test --workspace` and `git diff --check` passed. The mode-stack regression
now uses a real Rc pane and verifies that a detached entry neither retains its
parent nor traverses an expired stack. The existing copy snapshot regression also
passes with weak parent/source handles. Mode-entry callback pointers, local raw
model projections and other model fields/APIs remain pending.

## Implemented terminal client observer

`tty.client` now stores a weak backreference to its owning client. Terminal I/O,
key parsing, drawing, capabilities and terminal diagnostics retain an upgraded
client during access. The unopened-terminal stop path needs no client. Terminal
initialization replaces the old byte reset with a normal default assignment,
keeping Weak and the other Rust-owned fields valid and releasing their contents.

`cargo test --workspace` and `git diff --check` passed. Terminal, overlay and byte
output fixtures now use real Rc clients. The terminal-record lifecycle regression
also verifies that a surviving terminal does not keep its client alive. Raw
terminal/event callback pointers, model API arguments and local raw projections
remain pending.

## Implemented retained customization reset/unset dispatch

Reset/unset helpers and the shared per-item change dispatcher borrow the retained
customization-mode Rc. Current-item and tagged prompt handlers receive that same
typed owner, and current-item changes reuse the shared dispatcher. The prompt
factory weakly observes its cleanup-owned mode, retains it during invocation, and
rejects logically closed modes before entering the handler.

`cargo test --workspace` and `git diff --check` passed. Lifetime checks cover live
callback retention, cleanup release, closed-mode rejection and cached invocation
after expiration. Validation/draw-waiting helpers, lifecycle callbacks, and
internal raw projections remain pending.

## Implemented retained client path for terminal initialization

Peer-message dispatch and client identification now borrow the retained Rc client
from the peer callback. `tty_init` accepts that owner and derives its terminal,
removing both raw arguments and the possibility of a mismatched terminal/client
pair. The terminal stores only a downgrade of the supplied owner.

`cargo test --workspace` and `git diff --check` passed. A real pseudo-terminal
regression covers successful and repeated initialization, stable weak-reference
counts, preservation on invalid-descriptor failure and client expiration after
detaching the terminal. Other dispatch branches, terminal APIs and internal raw
model projections remain pending.

## Implemented typed client arguments for remaining dispatch helpers

Command-message dispatch now borrows the peer callback's retained Rc client
through parsing and queue construction. Shell-message dispatch takes a shared
client borrow for its read-only peer access. Both remove raw client arguments;
identification already receives the same retained owner.

`cargo test --workspace` and `git diff --check` passed. Stored `client.session`,
`window.active` and `window_pane.window` pointers, downstream queue APIs and local
raw projections remain pending.

## Implemented retained client argument for queue append

`cmdq_append` now accepts an optional borrowed Rc client. Each appended item clones
that owner and stores its weak execution-context association directly. `None`
selects the global queue. Dispatch and startup configuration pass existing owners;
legacy raw callers upgrade their client before invoking the typed API. Existing
queue-item release behavior remains in place.

`cargo test --workspace` and `git diff --check` passed. A chained-append regression
checks per-item ownership, weak execution-context identity, retained lifetime after
the external owner is dropped and release when detached items are dropped. Other
queue APIs and raw caller projections remain pending.

## Implemented retained client arguments for queue execution and cancellation

`cmdq_next` and internal queue lookup now accept optional borrowed Rc clients;
the server loop passes its registry owner directly. `cmdq_abort_file_wait` requires
a retained client through cancellation and item removal. Global queue access uses
`None`. Queue-item removal still transfers client owners to deferred release.

`cargo test --workspace` and `git diff --check` passed. The execution regression
covers a waiting callback, repeated polling, continuation, subsequent callback
client identity, queue draining and expiration after deferred releases dispatch.
Other queue helpers and internal raw model projections remain pending.

## Implemented direct client ownership in queue insertion

`cmdq_insert_after` now clones its retained execution-context client directly into
each inserted item and downgrades that owner for the observer field. It no longer
projects a raw client and upgrades it again. Queue-name formatting takes an
optional shared client borrow, eliminating logging-only client pointer locals in
append, insertion and queue execution.

`cargo test --workspace` and `git diff --check` passed. The queue ownership
regression now covers insertion between appended items, ordering, execution-context
identity and per-item retention/release. Other model pointers and queue-item raw
APIs remain pending.

## Implemented retained client argument for parse-and-append

`cmd_parse_and_append` now borrows an optional Rc client and passes that handle
directly to queue append. Control input, menu commands, mode-tree commands and
window-switch commands retain their client through parsing and error handling.
Error queue items reuse that same owner instead of recovering it from a raw view.

`cargo test --workspace` and `git diff --check` passed. Outer command callbacks,
control stream callback captures and other model projections remain pending.

## Implemented weak client captures for control streams

Control read/write/error callbacks capture weak client handles and upgrade them
for dispatch. Expired clients are skipped; live clients remain retained through
callback processing. `control_start` and the callback bodies borrow Rc clients,
and control reads pass their owner directly through parsing and queue append.
Both shared and separate control output streams use these callback captures.

`cargo test --workspace` and `git diff --check` passed. A detached-callback
regression verifies nonownership, live error dispatch, stopped-state handling and
safe invocation after client expiration. Existing shared/separate stream cleanup
ordering tests now use real Rc clients and pass. Other control APIs and local raw
model projections remain pending.

## Implemented weak client captures for terminal events

Terminal read, write, clipboard-query, startup and output-throttling events now
capture weak client handles instead of raw terminal addresses. Dispatch upgrades
and retains the client; callbacks accept that owner and derive its terminal.
Expired clients are skipped before terminal memory is accessed.

`cargo test --workspace` and `git diff --check` passed. The callback regression
checks nonownership, live clipboard-query flag clearing and invocation of all five
callbacks after client expiration. Terminal key timers, other terminal APIs and
internal model projections remain pending.

## Implemented weak client capture for the terminal key timer

The ambiguous-key timer uses the terminal weak-client dispatch helper. Its callback
borrows the upgraded Rc client and derives the terminal, removing the stored raw
terminal address and erased callback argument. A client can expire while a detached
callback survives; invoking that callback then skips dispatch.

`cargo test --workspace` and `git diff --check` passed. The regression checks a
cancelled timer callback, nonownership and invocation after client expiration.
Other terminal APIs and internal model projections remain pending.

## Implemented retained client path for terminal opening

`server_client_open` and `tty_open` now borrow an Rc client. Attach-session and
new-session pass their existing retained queue client through both APIs. Terminal
opening derives the terminal from that owner and registers weak event callbacks
directly from it, eliminating the raw terminal argument and owner reconstruction.

`cargo test --workspace` and `git diff --check` passed. Other terminal APIs and
internal raw model projections remain pending.

## Implemented retained client arguments for key dispatch

`server_client_handle_key`, its insertion variant and their shared implementation
now borrow an Rc client. Terminal parsing, click timers and send-keys pass their
existing retained owners. Queued key events clone that owner directly when needed,
and ordinary queue append receives it without recovering ownership from a pointer.

`cargo test --workspace` and `git diff --check` passed. Existing key-event early
return and deferred-cancellation coverage uses the typed API and direct Rc counts.
Downstream key handlers, session/pane associations and local raw projections remain
pending.

## Implemented retained client arguments for menu key handling

The server menu-key helper borrows the client owner from key dispatch. `menu_key`
accepts an optional retained client and passes it directly through command parsing
and queue append, eliminating the raw client argument and observer re-upgrade.
Callback-only menus retain their no-client path using `None`.

`cargo test --workspace` and `git diff --check` passed, including menu callback
reentry, menu destruction during callbacks and expired-menu handling. Session/window
associations in the outer helper and other raw model APIs remain pending.

## Implemented retained client arguments for mode command runners

Mode-tree and window-switch command runners now borrow optional Rc clients through
parsing and error reporting. Tree, client-list, buffer-list and switch-mode callers
upgrade their client before dispatch; the runners pass that owner directly to the
parser. Error reporting derives its client view from the retained owner.

`cargo test --workspace` and `git diff --check` passed. Outer mode callback client
arguments, saved model associations and local raw projections remain pending.

## Implemented retained client argument in mode key callbacks

The `window_mode.key` interface now borrows an Rc client. Clock, tree, client,
buffer, customize, panes and switch modes use the typed signature. Menu callbacks
pass their existing owners directly; pane key dispatch retains the client before
calling the mode. Tree/client/switch key handlers pass that owner directly to
command runners rather than upgrading a raw view again.

`cargo test --workspace` and `git diff --check` passed. The compiler verifies all
mode registrations and call sites against the typed callback interface. Mode
command callbacks, session arguments and internal raw model projections remain
pending.

## Implemented borrowed client owners in mode command callbacks

`window_mode.command` accepts an optional borrowed Rc client from send-keys.
Copy/view-mode callbacks pass that borrow into `window_copy_cmd_state`, whose
client field is now lifetime-bound instead of a raw pointer. Command handlers
and drag-state checks read through this retained client association. The command
state does not independently own the client; its caller retains it throughout.

`cargo test --workspace` and `git diff --check` passed. Callback registrations and
all state consumers compile against the typed client field. Session callback
arguments, downstream client APIs and local raw projections remain pending.

## Implemented typed session inputs for mode callbacks

The unused session argument has been removed from all seven mode key callbacks.
Mode command callbacks borrow an optional Rc session. Send-keys upgrades its saved
target directly; copy mode clones the borrowed owner and retains its existing
session_remove_ref cleanup, preserving deferred release behavior.

`cargo test --workspace` and `git diff --check` passed. All callback registrations
and callers compile against the updated signatures. Winlink arguments, downstream
session APIs and local raw model projections remain pending.

## Implemented retained pane and client inputs for pane key dispatch

`window_pane_key` borrows its pane owner and an optional client owner through mode
callbacks and input processing. Server key paths and send-keys pass their existing
clients and retain the selected pane before dispatch. The unused session argument
was removed; mode callbacks receive the borrowed client directly.

`cargo test --workspace` and `git diff --check` passed. All pane-key callers compile
against the typed model arguments. Winlink/mouse views, caller pane lookup paths
and local raw model projections remain pending.

## Implemented retained panes through key encoding and synchronization

`input_key_pane` and mouse encoding borrow retained pane owners from pane key
dispatch. Synchronized key delivery borrows its source pane, retains the associated
window and carries each destination owner through input processing and successor
lookup instead of extracting pointers from temporary owners.

`cargo test --workspace` and `git diff --check` passed. Pane/window association
storage, paste delivery and local raw projections remain pending.

## Implemented retained panes through paste delivery

Pane paste delivery borrows its Rc pane and a byte slice. Synchronized paste
retains the source window and each destination pane through output and successor
lookup, replacing pointers extracted from temporary pane owners. Key dispatch
retains its selected pane before invoking the paste API.

`cargo test --workspace` and `git diff --check` passed. Pane/window association
storage, outer pane lookup paths and local raw model projections remain pending.

## Implemented retained model arguments for pane prompt dispatch

`window_pane_prompt_key` borrows an Rc pane and optional Rc client. Prompt callback
records receive only a downgraded client association. Dispatch retains both inputs
through callbacks and preserves the registry lookup afterward to detect logical
pane removal before updating prompt state.

`cargo test --workspace` and `git diff --check` passed, including the prompt
replacement regression through the typed API. Prompt setup/update APIs, model
association storage and local raw projections remain pending.

## Implemented retained models for pane prompt setup and updates

Pane prompt setup borrows retained pane/client handles and downgrades the client
for callback storage. It retains the client's session before clearing the old
prompt, keeping that session available while cleanup callbacks run and prompt
options are read. Prompt updates borrow the pane owner already held by the command
callback.

`cargo test --workspace` and `git diff --check` passed. Prompt clearing, client
session storage and local raw model projections remain pending.

## Implemented retained pane argument for prompt clearing

`window_pane_clear_prompt` requires a borrowed Rc pane through prompt cleanup and
hook callbacks. Pane destruction passes its removed registry owner; setup and key
dispatch pass their retained pane handles. The prompt-replacement regression's
callback now captures a weak pane and retains its upgrade while replacing state.

`cargo test --workspace` and `git diff --check` passed, including prompt replacement
and cleanup coverage. Other pane APIs and local raw model projections remain
pending.

## Implemented retained panes for mode cleanup

Mode reset, reset-all and free-all now borrow an Rc pane, keeping the parent
available through mode cleanup callbacks and subsequent state updates. Mode
callbacks pass their upgraded parent; destruction passes its removed registry
owner; respawn passes its retained source. Copy-mode and capture commands retain
the selected target. Copy scrolling and page-down carry that owner through paths
that can exit the mode.

`cargo test --workspace` and `git diff --check` passed. A regression exercises
multiple mode cleanup callbacks, checks removal before callback invocation and
parent availability, and verifies final pane release. Mode-entry pointers,
parent-window storage and internal raw model projections still require migration.

## Implemented weak client captures for copy-mode dragging

Copy-mode drag update/release closures hold Weak clients instead of NonNull raw
addresses. Each invocation upgrades and retains the client, or skips work after
its expiration. Drag startup borrows the optional Rc client already held by the
command or copy-command state; update/release take retained client arguments.
These callback captures are observers, avoiding a client/terminal callback cycle.

`cargo test --workspace` and `git diff --check` passed. The detached-callback
regression verifies the callbacks do not retain the client and can be invoked
after it expires. Raw client captures in pane resize/move commands and internal
model pointer projections remain pending.

## Implemented weak clients for pane resize and move callbacks

Tiled resize, floating resize/move, join-pane move, and split-window resize
callbacks now capture Weak clients through `tty_mouse_client_callback`. The
helper upgrades for each invocation and holds the Rc through the handler; expired
clients skip the callback. The four handlers and immediate invocations borrow
retained clients already available in their command contexts. These are observer
captures, not owners of the terminal's client.

`cargo test --workspace` and `git diff --check` passed. A regression checks that
callback storage adds no strong owner, invocation temporarily retains the client,
and expiration prevents further invocation. Raw pane/window/session projections
inside the handlers and other model APIs remain pending.

## Implemented retained and borrowed copy-mode pane APIs

Copy output, page-up and line-number updates borrow Rc pane handles. Configuration
errors, shell output and client output retain the pane before entering view mode
and keep it through writing. Copy-mode commands reuse their selected pane owner.
The read-only offset query borrows `&window_pane` and returns `Option<(offset,
size)>`, removing raw output parameters and rejecting absent or unrelated modes
before interpreting mode data.

`cargo test --workspace` and `git diff --check` passed. Existing backing tests now
cover the offset query for copy/view modes and absent/unrelated modes. The public
copy-mode functions no longer take raw refcounted model arguments; private mode
callbacks, erased mode data and internal raw model projections remain pending.

## Implemented retained mode trees for tagged dispatch

`mode_tree_each_tagged` borrows an Rc tree and no longer forwards a raw client
argument. Callers retain a local tree handle across dispatch so callbacks may
clear rows or destroy the mode without invalidating the iteration's storage.
Callbacks capture their own context: buffer paste borrows its key handler's
client owner, and tree commands retain the prompt client before iterating. Pane
command execution also borrows its client owner and reads its target through a
pane borrow instead of raw model arguments.

`cargo test --workspace` and `git diff --check` passed, including callback-driven
tree destruction and row clearing tests. Prompt callback client adapters, other
mode-tree APIs and internal raw projections remain pending.

## Implemented retained models for mode-tree initialization

`mode_tree_start` borrows the parent pane's Rc and stores its Weak observer
directly. Buffer, client, customize and tree mode initialization pass their
already-retained parent and also construct their own parent observers by
`Rc::downgrade`, removing raw-pointer observer recovery from these constructors.
Initial zoom borrows a retained mode tree; callers clone its handle across the
operation. Parent links remain non-owning to avoid pane/mode ownership cycles.

`cargo test --workspace` and `git diff --check` passed. Mode-tree building,
drawing, callback data and internal raw model projections remain pending.

## Implemented retained mode trees for rebuilding

Mode-tree rebuild, resize, expansion and row swapping now borrow Rc tree handles.
The four owning modes clone their stored handle across calls; key dispatch reuses
its existing owner. Legacy search/filter entry points retain the tree before
rebuilding and keep it through subsequent state updates. Rebuild no longer
recovers its ownership from a raw pointer.

`cargo test --workspace` and `git diff --check` passed. A new regression destroys
the tree from its build callback, verifying dispatch stops and the tree is
released after the rebuild returns. Drawing, prompt adapters and internal raw
model projections still require migration.

## Implemented retained mode trees for drawing

Mode-tree drawing and help display now borrow Rc tree handles. All four mode
implementations retain their stored tree across drawing; resize, key, prompt and
filter paths pass their existing owners. Drawing skips logically destroyed trees
before accessing their released screen. The pane association remains weak and is
upgraded only for the duration of drawing.

`cargo test --workspace` and `git diff --check` passed. Regression coverage checks
drawing with an expired parent and with a destroyed tree whose screen is already
released. Drawing internals, callback payloads and other raw model APIs remain
pending.

## Implemented retained trees for prompt cleanup and search/filter dispatch

Mode-tree prompt clearing borrows an Rc through prompt free callbacks and the
subsequent cursor update. Search/filter handlers and their rebuild paths also
borrow retained trees. Their stored prompt closures capture Weak trees instead
of raw pointers, upgrade during invocation, and close on expiration or logical
destruction. Existing prompt callback-record ownership and cleanup ordering are
preserved.

`cargo test --workspace` and `git diff --check` passed. A regression verifies no
strong ownership from closure storage, temporary retention during live dispatch,
and skipped callbacks after destruction or expiration. Prompt client adapters,
row operations and internal raw model projections remain pending.

## Implemented retained clients for mode-tree key and menu dispatch

Mode-tree key dispatch borrows an optional Rc client from each mode's key handler
or queued prompt acceptance. Prompt callback records downgrade that handle
directly. Menu display borrows both the retained tree and client, preserving the
menu callback's owning tree capture and weak client capture without recovering
either through raw model pointers. With no client, menu display returns early.

`cargo test --workspace` and `git diff --check` passed, including existing menu
callback lifetime and expired-parent key-dispatch coverage. Prompt setup and its
client callback adapters still use raw client projections and remain pending.

## Implemented retained clients for mode-tree prompt setup

Prompt setup borrows an optional Rc client, downgrades it for prompt callback
storage, and passes the same owner to queued acceptance. Key dispatch and tree
mode callers pass their existing owners. Legacy customize helpers upgrade at the
setup boundary. Setup additionally retains the client's session across old-prompt
cleanup before reading prompt options; the client/session association itself is
still raw and remains pending.

`cargo test --workspace` and `git diff --check` passed, including existing prompt
cleanup and queued acceptance tests. Customize helper arguments, client callback
adapters and internal raw projections still require migration.

## Implemented retained client borrows for mode-tree prompt callbacks

The mode-tree prompt callback type now receives `Option<&Rc<UnsafeCell<client>>>`
instead of `Option<NonNull<client>>`. Dispatch passes its weak-client upgrade by
borrow for the callback duration. Tree command/kill callbacks carry that borrow
through command execution and completion queueing without pointer recovery.
Customize callback factories and handlers accept the same typed borrow; their
remaining internal legacy calls still project raw pointers.

`cargo test --workspace` and `git diff --check` passed. A regression verifies client
identity and retention during live dispatch, release after return, and `None`
after expiration. Customize internals and other raw model APIs remain pending.

## Implemented retained clients for customize action helpers

Nine customize helpers now borrow optional Rc client handles: environment/option
editing and creation, array-key editing, key editing/creation, editor startup and
current-group creation. Key dispatch passes its retained client through this
chain. Prompt setup no longer recovers ownership from raw client addresses in
these helpers. Editor startup projects a pointer only at the remaining legacy
spawn API boundary.

`cargo test --workspace` and `git diff --check` passed, including existing
customize prompt payload/cleanup tests. Customize model-data pointers, internal
client projections and editor spawning APIs remain pending.

## Implemented retained clients and sessions for editor spawning

Editor spawning borrows the client Rc and retains its session before invoking the
file writer, then clones both handles into the spawn context. Buffer and customize
editor callers pass their existing client owners. A client without a live session
returns without creating an editor or invoking its callbacks; supplied callback
captures are released on that return.

`cargo test --workspace` and `git diff --check` passed. A regression covers the
no-session return, suppressed file/completion callbacks and capture release.
Client session storage, window/winlink access and internal raw projections remain
pending.

## Implemented safe borrows for mode-tree row operations

Row insertion/removal, up/down navigation and selection now take mutable tree
borrows; tag counting and line counting take shared borrows. These operations do
not invoke application callbacks and do not acquire ownership. Their bodies no
longer dereference raw tree pointers, and their APIs are safe. Insertion retains a
small unsafe block for the existing global logger. All mode callers and row tests
use the borrowed interfaces.

`cargo test --workspace` and `git diff --check` passed, including row invalidation,
selection restoration and callback-time removal coverage. Legacy callers still
project borrows from retained UnsafeCell models; internal tree traversal and
other model APIs remain pending.

## Implemented retained trees for traversal and search

Recursive line building and forward/backward search now borrow retained tree
handles from their callers, carrying the handle through recursion and callback
dispatch instead of recovering ownership from a raw address. Selection-offset
adjustment and line clearing invoke no callbacks and now use safe mutable borrows.
Production callers and traversal tests use the typed interfaces.

`cargo test --workspace` and `git diff --check` passed, including key/search
callbacks that remove rows and tests for nested traversal and selection. Internal
raw projections, drawing helpers and other refcounted model APIs remain pending.

## Implemented retained trees for drawing helpers

Height calculation, prompt drawing and help drawing now borrow retained tree
handles from their callers. Height calculation detaches its callback during
invocation, stops if the callback destroys the tree, and preserves a replacement
callback instead of accessing the released screen or overwriting replacement
state. The detached callback is restored only when the live tree still needs it.

`cargo test --workspace` and `git diff --check` passed. Regressions cover a height
callback that destroys its tree and one that replaces itself. Drawing bodies and
other model implementations still contain raw projections and remain pending.

## Implemented weak tree-mode callback captures

Tree mode's build, draw, menu and key-label closures now observe the refcounted
mode through Weak handles instead of capturing NonNull addresses. Each callback
retains a live upgrade and skips closed/expired modes. Their entry points and the
nested session/window/pane builders borrow the mode Rc, removing erased mode-data
arguments and casts along that chain. These captures must remain weak because the
mode owns the callback-bearing tree.

`cargo test --workspace` and `git diff --check` passed. Existing mode lifetime tests
now verify live observer upgrades, temporary retention, and rejection of closed
or expired modes. Preview helpers, prompt captures and other internal raw model
projections remain pending.

## Implemented weak customize-mode callback captures

Customize mode's build, draw and menu callbacks capture Weak mode handles instead
of NonNull raw addresses. Each upgrades and retains a live mode for invocation,
skipping closed or expired modes. Their entry points borrow Rc mode handles,
removing erased mode-data arguments. Weak captures avoid a cycle between the mode
and its owned callback-bearing tree.

`cargo test --workspace` and `git diff --check` passed. Existing customize lifetime
coverage now verifies observer identity, temporary ownership and rejection after
logical destruction or final release. Nested customize helpers and remaining
internal raw model projections still require migration.

## Implemented typed mode ownership for tree prompts

Tree-mode command and kill prompts use a callback factory: cleanup owns the mode
Rc, while cached input closures observe it weakly and retain upgrades only during
live invocation. Prompt handlers and per-item command execution borrow the mode
owner. Completion queueing clones that owner directly. This replaces raw mode
captures while preserving cleanup-before-final-release ordering.

`cargo test --workspace` and `git diff --check` passed. Lifetime tests cover live
input retention, rejection of closed modes, cleanup release, and cached input
invocation after mode expiration. Preview helpers, key-dispatch model access and
other internal raw projections remain pending.

## Implemented retained models for tree previews and builders

Session/window/info preview helpers borrow the tree-mode Rc. Session/window
previews also borrow the session owner already returned by target resolution.
The nested builders carry retained sessions and panes from sorted collections,
and pane filtering takes retained handles through format evaluation. These
synchronous arguments borrow existing ownership without adding persistent links.

`cargo test --workspace` and `git diff --check` passed. Target resolution still
uses raw output parameters, and preview/formatting internals and winlink access
remain pending.

## Implemented typed tree-mode target results

Tree-mode target resolution returns `WindowTreeTarget` with optional owning Rc
session/pane handles and a weak winlink identity. It no longer writes raw model
pointers through output parameters or returns a separate ownership tuple. All
eleven callers retain the result while using the resolved models. Missing or
mismatched pane targets return an empty result; session/window targets preserve
their session/link when no active pane exists.

`cargo test --workspace` and `git diff --check` passed. Callers still derive local
raw projections for legacy operations; winlink/window internals and those caller
bodies remain pending.

## Implemented retained tree-mode input and mouse traversal

Key dispatch clones and downcasts the mode entry's owning payload before invoking
callbacks, replacing its erased-pointer cast and late observer upgrade. Prompt
creation clones this retained owner. Mouse handling borrows the mode Rc and
retains pane traversal handles through selection. Expansion checks logical mode
closure and weak winlink expiry before subsequent access; key dispatch also stops
when mouse expansion closes the mode.

`cargo test --workspace` and `git diff --check` passed. Internal UnsafeCell
projections, session winlink traversal, and pointer-valued row tags remain pending.

## Implemented retained customization builders and previews

The option-array, option, option-group, key-table, and environment builders now
borrow the retained customization-mode Rc from their parent builder. Option and
environment preview helpers likewise borrow the draw callback's retained owner.
These seven helper arguments replace raw mode pointers without adding persistent
strong references or changing row ownership.

`cargo test --workspace` and `git diff --check` passed. Internal UnsafeCell
projections, customization action helpers, and raw model access through find-state
compatibility methods remain pending.

## Implemented retained customization action arguments

Nine prompt/editor actions (set environment, add option/environment, start edit,
set option/array key/key, add key, and add current) borrow the customization-mode
Rc. Prompt records clone that owner directly instead of recovering ownership
through a raw mode pointer. Key dispatch downcasts and retains the mode entry's
owning payload before invoking tree callbacks, and checks logical closure before
accessing the resulting selection.

`cargo test --workspace` and `git diff --check` passed. Reset/unset helpers,
validation helpers, mode lifecycle callbacks, and internal raw model projections
remain pending.

## Implemented typed tree/customization lifecycle payload access

Mode entries expose `retained_data<T>()` to clone and downcast their owning Rc
payload. Tree/customization free, resize, update and key callbacks use this
accessor instead of casting the erased callback pointer. Editor completion retains
the discovered customization payload, and waiting-screen drawing borrows it.
Resize/update paths check logical closure; tree update rechecks after rebuilding.

`cargo test --workspace` and `git diff --check` passed. Neither mode now casts the
erased entry data pointer to its Rc model type. The erased compatibility field,
internal UnsafeCell projections, validation helpers and other model raw accesses
remain pending.

## Implemented borrowed customization validation

Item validation and option-editability checks take shared mode-data borrows for
synchronous inspection. Validation takes optional borrowed find-state output and
copies target fields from a local snapshot, preserving the existing output flag
semantics. Option-tree resolution borrows find state. All callers supply scoped
borrows backed by their retained mode owners.

`cargo test --workspace` and `git diff --check` passed. No customization helper
accepts a raw mode-data argument; constructor/internal UnsafeCell projections and
raw session/window/pane compatibility access remain pending.

## Implemented borrowed tree row cleanup

Removing an empty tree group takes explicit mutable borrows of the item-owner
collection and tree data. The helper is safe Rust and no longer receives or
projects a raw tree-mode pointer. Its callers retain the tree while supplying
those borrows. Kill-current selection borrows the stored tree owner after the
closed-mode check; obsolete raw tree temporaries were removed from kill prompts.

`cargo test --workspace` and `git diff --check` passed. Neither tree nor
customization helpers now accept raw mode-data arguments. Internal UnsafeCell
projections, pointer-valued row tags and raw session/window/pane uses remain.

## Implemented typed tree target actions

Target command formatting keeps resolved session/pane handles and snapshots the
winlink index under a checked borrow. Kill actions use the resolved owners
without raw model locals or recovering a session owner from its pointer. Window
kills retain a WindowOwner before releasing the winlink borrow and entering
destructive code, preserving the window release-notification policy.

`cargo test --workspace` and `git diff --check` passed. Raw projections remain at
legacy find-state/destruction API boundaries; target resolution internals and
other preview/key-handler raw accesses remain pending.

## Implemented retained pane destruction entry points

`server_kill_pane` borrows an owning pane Rc across layout and destruction calls.
Command, modal mouse/key, mode-reset and tree-mode callers supply retained
handles. The kill-all command keeps owning current/next traversal handles through
filtering and removal, and its filter helper borrows the current pane owner.

`cargo test --workspace` and `git diff --check` passed. Parent-window access,
kill-all target identity, session/winlink locals and lower-level teardown APIs
still use raw projections and remain pending.

## Implemented owning window-kill dispatch

`server_kill_window` consumes an Rc window owner instead of recovering ownership
from a raw argument. Its existing `window_remove_ref` release and close-notification
ordering remain intact. WindowOwner exposes a borrowed Rc handle for callers to
clone; command/tree callers use stored owners, and join-pane upgrades its source
find-state observer. Pane teardown upgrades the remaining raw parent relationship
at the call boundary.

`cargo test --workspace` and `git diff --check` passed. Window-kill internals,
parent-window fields, command traversal and other raw model projections remain
pending.

## Implemented retained pane-exit destruction

`server_destroy_pane` borrows a retained pane handle through exit handling and
teardown. Error, dead-key, child-exit and pipe callers supply owners; child-exit
traversal retains its current pane through wait/editor completion. Pipe command
setup upgrades the target observer, and its read/write/error callbacks capture
Weak handles instead of raw pane pointers, retaining each successful upgrade
through callback execution without creating event/pane ownership cycles.

`cargo test --workspace` and `git diff --check` passed. Destruction internals,
raw parent-window links, dead-key entry arguments and lower-level removal APIs
remain pending.

## Implemented retained pane removal and logical destructor

`window_remove_pane` and the internal `window_pane_destroy` borrow retained pane
handles. Bulk window destruction forwards its traversal owner; server teardown
and kill-all forward existing owners. Spawn/split failure cleanup retains the new
pane before client/layout removal. The destructor still removes and releases the
registry owner in its existing order, while the caller retains the allocation.

`cargo test --workspace` and `git diff --check` passed, including pane observer
expiry and empty-pane stream callback release tests adapted to the typed API.
Raw window arguments, pane-parent links and internal teardown projections remain
pending.

## Implemented retained dead-pane key dispatch

Dead-pane key handling accepts an optional borrowed pane owner. Queued and direct
key dispatch retain their selected pane before checking exit behavior and reuse
that handle for key delivery or modal cancellation. The dead-key helper forwards
its owner to destruction without recovering it from a raw pointer.

`cargo test --workspace` and `git diff --check` passed. Initial active/target-pane
lookup, prompt selection traversal and other key-handler raw model projections
remain pending.

## Implemented retained prompt-pane selection

Direct key dispatch carries owning pane handles while searching for a visible
prompt and passes the selected owner through prompt-key callbacks. Active-pane
prompt priority and visibility checks are preserved. The prompt-presence query
is now safe and takes a shared pane borrow; cursor and command callers provide
scoped borrows at their existing boundaries.

`cargo test --workspace` and `git diff --check` passed. The window active-pane
relationship, cursor helper arguments, command targets and other raw projections
remain pending.

## Implemented borrowed pane visibility and prompt cursor inputs

Prompt cursor calculation borrows pane data. Horizontal visibility queries take
an optional shared pane borrow, preserving the no-pane unclipped-span behavior.
The z-order walk keeps each pane owner until its geometry and visibility have
been consumed. Screen-write, cursor and test callers use the typed input.

`cargo test --workspace` and `git diff --check` passed, including clipping,
occlusion, edge-coordinate and reusable visibility-buffer tests. Raw pane-parent
window access and lower-level geometry helper arguments remain pending.

## Implemented borrowed pane visibility and exit queries

Pane visibility and exited-state queries take shared pane borrows. All command,
input, redraw, prompt and traversal callers supply scoped borrows. The exited
query no longer needs unsafe code; visibility remains unsafe because it still
reads the raw parent-window relationship.

`cargo test --workspace` and `git diff --check` passed. Caller-local raw
projections, parent-window links and remaining geometry APIs are still pending.

## Implemented borrowed window pane counting

Pane counting borrows the window and traverses its ordered weak entries directly,
retaining each upgraded pane during the floating-state query. It no longer keeps
raw pane traversal pointers or performs repeated next-pane lookups. Layout,
formatting, command, tree-mode and teardown callers supply window borrows.

`cargo test --workspace` and `git diff --check` passed. Floating-state layout
access, caller-local raw window projections and other traversal APIs remain
pending.

## Implemented borrowed pane search

Pane search and its format-expression adapter take shared pane borrows. Search
reads the base grid directly through that borrow instead of creating a raw
mutable screen pointer. Format expansion supplies the borrow from its retained
pane owner; regex/glob behavior and one-based result numbering are unchanged.

`cargo test --workspace` and `git diff --check` passed. Other format-expression
raw locals, pane model fields and remaining pane helper APIs remain pending.

## Implemented retained pane geometry lookup

Full-size geometry lookup borrows a retained pane Rc and returns its offset/size
tuple instead of writing through four raw output pointers. Containment checks
also borrow a retained pane, and coordinate/directional lookup passes its existing
source or candidate owner. Modal mouse handling forwards its retained handle.

`cargo test --workspace` and `git diff --check` passed, including coordinate-pane
ownership coverage. Internal pane/window projections and scrollbar/layout helper
APIs remain pending.

## Implemented borrowed mode and scrollbar queries

Seven read-only helpers (pane mode, scrollbar show/reserve/overlay/visible,
auto-hide, and visible overlay) take shared pane borrows. Their nested calls pass
those borrows directly, and layout, rendering, copy mode, formatting, resize and
input callers use the typed signatures.

`cargo test --workspace` and `git diff --check` passed. Queries remain unsafe
where they follow raw parent-window or active-mode fields; scrollbar mutation,
timer APIs and other internal model projections remain pending.

## Implemented retained scrollbar mutations and scheduling

Scrollbar redraw, visibility redraw, timer setup, show and hide borrow retained
pane handles. The timer callback forwards its existing owner, as do copy-mode,
screen-write and option-update callers. Mouse-hover traversal retains its current
pane through scrollbar updates instead of extracting pointers from temporary
traversal owners. Timer callbacks continue to observe panes weakly.

`cargo test --workspace` and `git diff --check` passed. Internal raw parent-window
access, mouse-hover client/window arguments and other model projections remain
pending.

## Implemented retained mouse hit testing

Mouse hit testing borrows a retained target pane and writes slider position
through a mutable reference. Its border traversal retains each visited pane.
Scrollbar-area checks take shared pane borrows. The existing selection boundary
upgrades the selected pane observer before dispatching the hit test.

`cargo test --workspace` and `git diff --check` passed. Initial mouse target
selection, parent-window access, hover client arguments and geometry helper
internals still contain raw projections and remain pending.

## Implemented retained client/session/window mouse dispatch

Mouse checking and scrollbar-hover updates borrow the queued dispatch's client
owner. Each retains the current session and its window while inspecting models
and invoking nested helpers. Coordinate lookup reuses the retained window handle;
WindowOwner preserves window release notifications. Detached clients return
without accessing a missing session.

`cargo test --workspace` and `git diff --check` passed. Client session links,
pane selection locals, winlink access and other internal raw projections remain
pending; retained allocation lifetime does not replace logical liveness checks.

## Implemented owning mouse pane selection state

Mouse dispatch stores its previous drag pane and selected pane as optional owning
handles. Modal identity compares Rc handles; drag selection clones the previous
owner and hit testing borrows the selected owner directly. Status-range pane
validation tests the lookup result without extracting a raw pointer.

`cargo test --workspace` and `git diff --check` passed. Selected-pane projections
for legacy status-range and active-pane APIs, raw window relationships and other
mouse internals remain pending.

## Implemented retained pane status-range lookup

Pane status-range lookup takes a retained pane handle and returns an optional
copied style range instead of a pointer into pane-owned storage. Mouse border
handling passes its selected owner and consumes the copied argument directly.
Range matching borrows the collection for the lookup only.

`cargo test --workspace` and `git diff --check` passed. Added checks cover range
boundaries, wrong-row rejection, and a result surviving range-storage cleanup.
Client status-range lookup, pane-status options queries and other internal raw
model projections remain pending.

## Implemented borrowed client status-range snapshots

Client status-range lookup is safe Rust taking a shared client borrow and returning
an optional copied range. It bounds-checks the status row and searches borrowed
range storage. Mouse dispatch and formatting consume values rather than raw range
pointers; mouse session-range validation also avoids a temporary raw session.

`cargo test --workspace` and `git diff --check` passed. New tests cover horizontal
boundaries, missing/out-of-bounds rows and snapshots surviving storage cleanup and
client release. Raw client/session relationships and other status APIs remain
pending.

## Implemented borrowed pane border-option queries

The four window/pane border-line and border-status queries take shared model
borrows. Their inputs neither retain nor release references. Numeric option
lookup now has a shared-borrow accessor, including ancestor fallback; the legacy
raw accessor delegates to it. All callers supply scoped borrows. Floating-only
status mapping and zoomed-mode status suppression are preserved.

`cargo test --workspace` and `git diff --check` passed. Option-storage coverage
checks inherited numeric values, local overrides and fallback after
removal through the shared accessor. Raw pane/window relationships, active-mode
pointers and option parent links still require unsafe access and remain pending.

## Implemented borrowed pane-order traversal inputs

All ten pane-order traversal helpers now take shared window/pane borrows, with
optional inputs preserving null-input behavior. Ordering collections observe panes
with Weak handles; successful traversal returns an owning Rc. Window-only and
history traversal are safe Rust. Advancing from a pane still requires unsafe
access to its raw parent-window relationship. Callers across layout, commands,
rendering, sessions and server dispatch supply scoped borrows. Empty-order checks
in pane creation and destruction consume Option directly.

`cargo test --workspace` and `git diff --check` passed. Several callers still
project traversal results into raw locals, and pane-to-window relationships and
ordering mutation APIs remain pending.

## Implemented retained pane traversal during zoom restoration

Zoom accepts a retained pane handle. Mode initialization, saved-zoom restoration
and resize callers forward their existing owners; the resize-pane command upgrades
its target once and holds it through dispatch. Window resizing keeps its optional
zoomed owner directly instead of introducing a nullable raw pane local.

Zoom and unzoom traversal retain the current pane while saving/restoring layout
links. Unzoom also retains the selected zoomed pane across layout replacement and
the insertion neighbor while restoring stacking order. Pure field updates use
scoped mutable pane borrows. The raw window input to internal unzoom remains:
window final-drop cleanup invokes it after upgrading that window is impossible.
Legacy layout and ordering-mutation boundaries still use raw projections.

`cargo test --workspace` and `git diff --check` passed. Raw model relationships and
remaining zoom/unzoom window APIs are still pending.

## Implemented borrowed pane-order mutation APIs

All twelve insertion, removal and swap helpers for pane order and stacking order
now use safe Rust with mutable window borrows and shared pane borrows. The
collections clone or transfer weak pane identities only; these operations neither
retain nor release pane ownership. Same-window swaps explicitly omit the second
window borrow, avoiding aliased mutable window references. All production callers
have migrated to the borrowed interfaces.

`cargo test --workspace` and `git diff --check` passed. A regression test covers
same-window and cross-window swaps in both orderings and verifies pane strong
counts remain unchanged. Raw model relationships and raw projections in calling
commands/layout routines remain pending.

## Implemented retained swap-pane selection and dispatch

Swap-pane upgrades its source and destination panes and windows before dispatch.
Window handles use WindowOwner so their normal release policy is preserved.
Directional tiled-pane searches consume/return optional Rc handles and retain each
candidate while advancing; wraparound stays in the owning-handle path. The final
source/destination pane identity check uses Rc::ptr_eq. Both selected panes remain
owned through layout, focus, redraw and event operations.

`cargo test --workspace` and `git diff --check` passed. Legacy layout/focus APIs and
pane-to-window links still use raw projections within the retained operation.

## Implemented retained pane rotation dispatch

Rotate-window retains its target window with WindowOwner. In both rotation
directions, the moved pane, current pane and neighboring pane stay in Rc handles
while order and geometry are updated. The final active candidate is selected as
an Option<Rc> with wraparound and remains retained through focus/state updates,
zoom restoration and redraw. Traversal no longer drops a temporary owner before
using a raw pane local.

`cargo test --workspace` and `git diff --check` passed. The active-pane relationship
and legacy resize/focus/layout calls still require raw projections from retained
models; these remain pending.

## Implemented retained pane resize inputs

Pane resize requires a retained Rc handle, including across mode-resize callbacks
and event creation. Rotation, swap and respawn forward existing owners; layout
repair now retains each traversed pane through resize and subsequent field checks.
Resize-queue cleanup is safe Rust using a mutable pane borrow. Terminal-size
notification takes a shared pane borrow because it only reads model data before
issuing the ioctl.

`cargo test --workspace` and `git diff --check` passed. The raw parent-window link,
mode-entry pointers and internal event/resize projections remain pending.

## Implemented retained server pane maintenance

The server maintenance passes keep each pane in an Rc while invoking style
callbacks, checking resize/buffer state, clearing flags and sending theme updates.
The resize, buffer and theme helpers take retained panes; window-resize checking
receives the existing retained window. Resize timers derive Weak captures directly
from the pane owner. Client traversal in the main pass and buffer accounting uses
owning cursors, with scoped mutable client borrows for buffer accounting.

`cargo test --workspace` and `git diff --check` passed. Raw client/session and
pane/window relationships, mode-entry pointers and several maintenance helper
internals remain pending.

## Implemented retained client mode maintenance

Client mode updates and terminal-state reset take retained client handles. Both
retain the associated session and current window, returning early when either
association is absent. Window release uses WindowOwner. Mode-update traversal
retains each pane across its callback; state reset retains the active pane and
uses owning traversal with scoped pane borrows when gathering mouse modes.

`cargo test --workspace` and `git diff --check` passed. Client/session and active
pane relationship fields still require raw access when acquiring the handles;
mode-screen pointers and legacy terminal APIs also remain pending.

## Implemented retained client redraw models

Client redraw retains the session and current window across rendering, with
WindowOwner preserving window release policy. Deferred-redraw traversal keeps
pane owners while reading flags. The redraw-needed query accepts shared client
and window borrows and inspects upgraded weak ordering entries without deriving
the window through raw client/session links. Existing snapshot-based pane redraw
uses scoped shared pane borrows from its owners.

`cargo test --workspace` and `git diff --check` passed. Acquiring session/window
handles still crosses raw relationship fields; title/path/progress helpers and
other rendering internals remain pending.

## Implemented borrowed terminal metadata updates

Title updates accept the existing client owner, retain its session through format
expansion, and use the format constructor that accepts a client handle. Path and
progress updates borrow the client mutably and retain its active pane. Progress
state is copied as a Rust value rather than through a raw pointer and memcpy.
A shared-borrow helper resolves the active pane without callbacks, confining the
remaining raw session/current-window/active-pane relationship reads to acquisition.

`cargo test --workspace` and `git diff --check` passed. Raw relationship fields and
legacy format-defaults/terminal API projections remain pending.

## Implemented retained client exit checks

Client exit checks now accept the existing Rc from the maintenance loop or exit
timer callback. Timer setup and control discard operations take mutable client
borrows; control-output completion takes a shared borrow and reads control state
without requesting mutable access. Pending-file traversal borrows each client_file
from its retained handle instead of maintaining a raw client_file local.

`cargo test --workspace` and `git diff --check` passed. Client-loss dispatch and
remaining control/transport internals still use raw projections and remain pending.

## Implemented retained client-loss boundary

Client-loss handling accepts a retained client from peer dispatch, terminal input,
the exit timer or server shutdown. File-wait cancellation reuses that handle;
file completion traversal borrows records from retained file handles. Registry
removal and deferred release of its owning client reference remain at their
original cleanup points. Server shutdown uses an owning cursor and retains its
successor before disconnecting the current client.

`cargo test --workspace` and `git diff --check` passed. Client cleanup internals,
file-completion scheduling and raw session relationships remain pending.

## Implemented retained file completion and output scheduling

File completion and output pushing require retained client_file handles. All
callers forward creation, lookup or traversal owners. Terminal completion and
output-retry closures clone those owners directly instead of upgrading observers
from raw pointers. Completion's existing scheduled-once guard and FileCompletion
cleanup policy are unchanged.

`cargo test --workspace` and `git diff --check` passed, including existing duplicate
completion, cancellation and owner-release coverage updated to the typed inputs.
File read callbacks, index removal and other internal raw projections remain
pending.

## Implemented retained file read notifications and cancellation

File read notifications and command-wait cancellation accept retained file
handles, including from terminal completion and queue-abort dispatch. Sending a
read-cancel message takes a mutable file borrow. Callers forward existing lookup,
callback or wait owners, keeping allocations alive while cancellation callbacks
run. Existing repeated-cancellation and dead-client semantics are preserved.

`cargo test --workspace` and `git diff --check` passed, including command-wait
cancellation and file-input callback tests adapted to the typed inputs. File-event
client payloads, index removal and other raw internal projections remain pending.

## Implemented typed clients in file callback events

File callback events now carry an optional borrowed Rc client handle instead of
NonNull<client>. Read and completion dispatch retain a client snapshot for the
callback's duration and check its flags through that handle. Callbacks may clone
the borrowed handle explicitly to extend ownership beyond dispatch.

`cargo test --workspace` and `git diff --check` passed. A new test verifies a
completion callback can retain its client after the file leaves the index and is
freed, then release the client by dropping its saved handle. File-index removal
and other internal raw projections remain pending.

## Implemented borrowed file-index boundaries

File-index removal is safe Rust taking a mutable file borrow and comparing weak
allocation identities. It requires no upgrade, preserving final-drop cleanup and
idempotent removal after terminal completion. Next-file lookup is also safe Rust;
insertion takes a mutable index borrow and still consumes an owning file handle.
Callers in completion, cancellation, I/O cleanup and tests use the borrowed API.

`cargo test --workspace` and `git diff --check` passed, including index replacement,
retained lookup guards and completion cancellation coverage. Internal file/peer
projections and legacy file-transfer entry points remain pending.

## Implemented retained file message dispatch

Write completion accepts a retained file handle through cleanup, callback dispatch
and index removal. Write-data/close/ready/done and read-cancel/data/done handlers
borrow their file index and resolve an owning file handle directly, preserving
their existing missing-stream behavior. Read cancellation forwards that owner to
the error callback rather than recovering it from a raw file. Pending-write checks
also borrow the index and inspect files through scoped borrows from owning cursors.

`cargo test --workspace` and `git diff --check` passed. File-open/read-transfer
entry points and remaining model/transport projections remain pending.

## Implemented borrowed file-open initialization

Peer-file creation and read/write-open handlers take mutable file-index borrows.
They initialize the created client_file through a scoped mutable borrow from its
Rc owner, removing nullable raw file locals. I/O callbacks derive weak handles
directly from that owner and upgrade them on invocation. The index owns the file;
callbacks observe it without creating a reference cycle.

`cargo test --workspace` and `git diff --check` passed. Raw peer descriptors are
outside the refcounted model migration; client-backed transfer creation and other
internal model projections remain pending.

## Implemented typed file-wait publication and clearing

Read startup no longer returns an unused raw file pointer. Transfer ownership
stays with the index or scheduled terminal event, while callback initialization
receives a Weak derived from the creation owner. Wait setup takes the retained
file handle; queue publication safely downgrades it. Clearing compares weak
identities and does not cast allocation pointers. Cancellation and terminal
completion preserve the existing wait-cleanup order.

`cargo test --workspace` and `git diff --check` passed, including file-backed wait
and cancellation coverage. Raw client inputs to transfer creation and remaining
internal file projections remain pending.

## Implemented typed clients for file transfer startup

Client-backed file creation takes an optional borrowed Rc and explicitly clones
the client it owns, preserving the rule that attached clients are excluded.
Read/write transfer startup accepts the same typed input. Load/save/source-file
commands and pane input forward existing handles; source-file reading retains a
client while moving its callback state. File-record initialization uses a scoped
mutable borrow. Legacy printing entry points still adapt their raw client inputs
at the constructor boundary.

`cargo test --workspace` and `git diff --check` passed. Raw printing inputs,
transfer internals and broader model relationships remain pending.

## Implemented typed client inputs for printing

File printing, byte-buffer printing and error output accept optional borrowed Rc
client handles. Eligibility checking is safe Rust with an optional shared client
borrow. Record creation forwards the handle directly, eliminating raw-pointer
client retention from file printing. New and existing output records are accessed
through scoped borrows from owners. Server printing and command/capture/display
callers forward existing client handles through the output path.

`cargo test --workspace` and `git diff --check` passed. Transfer and display
internals still project retained models through UnsafeCell; raw model relationship
fields and other APIs remain pending.

## Implemented client borrows for file-transfer routing

File path resolution accepts an optional shared client borrow. Read/write startup
checks client flags through short borrows from the supplied Rc, without keeping
raw client locals across initialization callbacks. Retry and write acknowledgement
checks likewise borrow from the file's owning client handle, removing all
client_rc_ptr conversions from file.rs. Missing-client, attached-client,
control-client, dead-client and acknowledgement behavior is preserved.

`cargo test --workspace` and `git diff --check` passed. Internal file projections,
other client access paths and raw model relationship fields remain pending.

## Implemented file borrows during transfer startup

Read/write transfer startup accesses its retained file through a mutable borrow.
Write creation and wait registration are shared by both path branches; read
callback initialization runs before acquiring the file borrow. These borrows end
before terminal completion scheduling. The transfer Rc still owns the allocation,
with index ownership and scheduled completion ownership unchanged. This removes
the last explicit raw client_file declaration in file.rs; inferred raw file
projections in other operations still require migration.

`cargo test --workspace` and `git diff --check` passed, including existing file
completion and cancellation tests. The broader model migration remains pending.

## Implemented file borrows for completion and message dispatch

Terminal scheduling, read delivery, wait cancellation and incoming file messages
access file fields through borrows from retained Rc handles. Terminal dispatch
copies the client handles before cancellation, borrows callback event fields only
for delivery, and reacquires a file borrow for index retirement after callback
destruction. Message handlers finish field access before handing their retained
file to read delivery, completion scheduling or output processing. Read-cancel
logging needs only a shared file borrow. Existing ownership and delivery order
remain unchanged.

`cargo test --workspace` and `git diff --check` passed. These operations still use
UnsafeCell at borrow acquisition; other file operations and the broader model
relationships remain to be migrated.

## Implemented file borrows for buffered I/O

Output push/retry, buffered read/write callbacks, write-data and write-close
handlers now access retained files through shared or mutable borrows. Retry
eligibility and write-data forwarding need only shared file access. Write
completion reacquires a file borrow after callback delivery to retire the stream;
handlers end field access before invoking other file operations. Retry events
continue to own their file, and buffered event callbacks continue to hold Weak
observers that upgrade for dispatch.

`cargo test --workspace` and `git diff --check` passed. Raw client locals in
printing, file index insertion projections, and broader model relationships still
require migration; UnsafeCell borrow acquisition remains an unsafe boundary.

## Implemented client printing and file index borrows

Printing and error output use short client borrows for eligibility, index lookup
and peer access. No client borrow spans file creation or formatting callbacks.
File index insertion borrows the incoming owner to obtain its key and install its
weak index observer before moving the Rc into the map. Ownership tests use Rc
identity comparison and fresh borrows instead of raw file aliases across calls.
The shared raw Rc projection helper is no longer used by file.rs.

`cargo test --workspace` and `git diff --check` passed, including index replacement,
completion lifetime and wait cancellation coverage. UnsafeCell remains at borrow
acquisition; this does not establish complete alias safety or finish the broader
refcounted model migration.

## Implemented owning mode-tree accessors in chooser modes

Buffer, client, tree and customize modes expose an Rc-cloning tree accessor
instead of returning a nullable raw mode_tree_data pointer. Existing callers
require an initialized tree and now explicitly check that invariant. Field/query
operations borrow from these owners for the call. Buffer and client key handlers
retain a tree owner across key dispatch instead of caching a raw tree pointer
for later selection and preview operations.

`cargo test --workspace` and `git diff --check` passed. No explicit raw
mode_tree_data declarations remain in these four chooser modules. UnsafeCell
projections, mode payload pointers, other model accesses and raw mode-tree test
helpers remain pending; this is not a complete alias-safety migration.

## Implemented tree borrows for setup, sizing and cleanup

Mode-tree construction borrows its new tree and source pane. Height calculation
takes the callback out of the tree before dispatch, then reacquires a mutable
borrow and checks logical destruction before restoring the callback or updating
height. Default preview sizing borrows the tree directly. Resize finishes its
screen borrow before rebuilding/drawing and reacquires the retained pane to mark
redraw. Cleanup borrows again after unzoom and prompt cleanup, preserving the
special allowance for a logically destroyed pane during teardown.

`cargo test --workspace` and `git diff --check` passed. Tree build/draw internals,
zoom access and broader raw model relationships remain pending.

## Implemented scoped tree borrows during rebuild

Rebuild uses scoped tree borrows between callback and recursive line-building
operations. Build callbacks are taken out for dispatch, receive local sort
criteria and an owned filter snapshot, and are restored only when the tree is
live and no replacement was installed. Sort changes are written back on live
return. Child rows are snapshotted before recursive traversal. Height callbacks
are followed by a logical-liveness check before selection adjustment.

A regression test verifies callback replacement, filter mutation without
invalidating callback input, sort-result preservation and eventual tree release.
`cargo test --workspace` and `git diff --check` passed, including existing callback
self-destruction coverage. Raw projections in recursive line building, rendering
and other model operations remain pending.

## Implemented scoped tree borrows during recursive line building

Recursive line building borrows the tree only for local depth and line updates,
ending each borrow before descending into children. Key callbacks are removed
from the tree for dispatch and restored only while live and without overwriting
a replacement. Row payloads and child traversal snapshots remain typed values;
logical destruction and row expiry checks are preserved.

A regression test checks key callback replacement across rebuilds and eventual
release of both the tree and its rows. `cargo test --workspace` and
`git diff --check` passed, including existing row-removal callback coverage.
Rendering, other tree interactions and broader model relationships remain pending.

## Implemented tree borrows for search and filter handling

Search snapshots its query and starting row, borrows the tree for individual
traversal steps, and takes search callbacks out for dispatch. Callback restoration
preserves replacements and skips logically destroyed trees; existing row-expiry
checks remain. Search selection reacquires a tree borrow after rebuilding and
checks destruction before selecting. Search/filter prompt handlers use local tree
borrows and retained pane handles rather than raw tree and pane locals.

`cargo test --workspace` and `git diff --check` passed, including search-order and
callback row-removal coverage. Rendering, swap/menu interactions, remaining tree
projections and broader raw model relationships remain pending.

## Implemented tree borrows for row swapping and expansion

Row swapping selects payloads through a scoped tree borrow, takes its callback
out, and dispatches with local sort criteria. It checks logical destruction before
restoring state or rebuilding and preserves callback replacements. Expansion
uses shared tree borrows to select rows before rebuilding. This migration uses
only existing refcounted entity holders; no new Rc-managed data type is added.

A regression test uses the existing tree holder to verify that a swap callback
can logically destroy the tree without a subsequent rebuild or ownership cycle.
`cargo test --workspace` and `git diff --check` passed. Rendering, menus and other
raw model access remain pending. The reverted test-helper migration remains
reverted.

## Implemented tree and pane borrows for mode menus

Menu action dispatch borrows the retained tree for validation and callback
extraction, then reacquires a borrow after the action before restoring the
callback. Client liveness uses a shared borrow. Display setup borrows tree state
and pane offsets without retaining raw model locals. Existing menu APIs still
require raw client pointers at their call boundaries; those APIs remain pending.
No new Rc-managed type is introduced.

`cargo test --workspace` and `git diff --check` passed, including menu callback
replacement, mode destruction, expired-client and cancellation coverage. Rendering
and remaining raw model APIs still require migration.

## Implemented typed client inputs for menu item construction

Menu item and item-list builders accept optional borrowed client Rc handles.
Mode-tree menus and display-menu commands forward existing holders, preserving
missing-client support for separator and suppressed rows. Visible-row sizing uses
a short shared client borrow. Formatting calls still adapt to their existing raw
client interfaces, which remain pending. Menu-row tests forward their existing
client owner and no longer keep a raw client local. No new Rc-managed type is
introduced.

`cargo test --workspace` and `git diff --check` passed, including menu row growth,
separator, expansion ownership and client reference-count coverage. Menu display
and formatting interfaces still require migration.

## Implemented typed client input for menu display

Menu display accepts an optional borrowed existing client Rc. Mode-tree and
command callers forward their current holders. With an explicit target no client
is required; otherwise target resolution borrows the client to follow its current
session. The overlay continues to observe its window weakly, preserving callback
replacement and window-destruction behavior. No new Rc-managed type is added.

`cargo test --workspace` and `git diff --check` passed, including menu replacement
and target-window destruction coverage. Raw window access in menu display and raw
session relationships still require migration.

## Implemented retained window setup for menu display

Menu display acquires its setup window from the current link's existing holder
or the target state's Weak window. A WindowOwner preserves window release policy;
sizing and remembered menu coordinates use a mutable window borrow. Setup releases
that owner before closing an existing menu, so cancellation callbacks retain the
ability to destroy the target. Installation borrows the upgraded window directly.
No new Rc-managed entity type is introduced.

`cargo test --workspace` and `git diff --check` passed, including replacement
callbacks that release the final target-window owner. Client/session link access
and command-find/redraw raw API boundaries remain pending.

## Implemented typed window input for menu redraw

Menu redraw accepts a borrowed existing window Rc. Its client traversal retains
one registry client at a time and marks clients by comparing the current link's
window holder with Rc identity, removing raw client cursor and window-pointer
comparison. Menu close takes its overlay through a mutable window borrow, ending
that scope before cancellation callbacks. No new Rc-managed type is introduced.

`cargo test --workspace` and `git diff --check` passed. Client/session and current
link relationships still require unsafe acquisition; broader redraw APIs and
remaining menu style access remain pending.

## Implemented borrowed windows for border redraw

Border redraw accepts a shared window borrow and walks retained client registry
handles. Matching uses weak window identity against the current link's existing
window holder instead of raw pointer equality. All 25 callers pass window borrows;
legacy raw relationships are adapted at those call boundaries. This operation
only marks redraw flags and does not add persistent ownership or new Rc-managed
types.

`cargo test --workspace` and `git diff --check` passed. Raw parent/session
relationships at callers and other redraw/status APIs remain pending.

## Implemented borrowed sessions for status redraw

Session status redraw accepts a shared session borrow and traverses retained
client registry handles. It compares session weak identities and sets the status
redraw flag through a client borrow, removing the raw client cursor and raw
session comparison. All ten callers pass borrows; existing raw relationships are
adapted at the call boundary. No new Rc-managed type or persistent owner is added.

`cargo test --workspace` and `git diff --check` passed. Client session fields,
group/window status traversal and other redraw APIs still require migration.

## Implemented borrowed windows for status updates

Window status updates accept a shared window borrow and traverse retained session
registry handles. Membership checks and session status marking borrow each
retained session, eliminating the raw session cursor. All callers pass window
borrows at the existing access boundary. No new Rc-managed type or persistent
owner is introduced.

`cargo test --workspace` and `git diff --check` passed. Raw window relationships at
callers and remaining redraw/group traversal APIs are still pending.

## Implemented safe borrowed client redraw flag setters

Client full-redraw and status-redraw setters are safe functions taking mutable
client borrows. All callers now pass short borrows, including format-job paths
with existing retained client handles. The setters only update flags and require
no ownership changes. No new Rc-managed type or persistent owner is introduced.

`cargo test --workspace` and `git diff --check` passed. Raw client acquisition at
legacy callers, session/window redraw traversal and other model relationships
remain pending.

## Implemented borrowed sessions for full redraw

Session-wide full redraw accepts a shared session borrow, traverses retained
client handles and compares session weak identities. Matching clients are updated
through the safe borrowed redraw setter. All callers pass session borrows. No
new Rc-managed type or persistent owner is added.

`cargo test --workspace` and `git diff --check` passed. Group lookup, raw client
session fields and remaining window redraw interfaces are still pending.

## Implemented borrowed windows for full redraw

Window-wide full redraw accepts a shared window borrow and traverses retained
client handles. It compares weak window identities through current link holders
and marks matching clients through the safe redraw setter. All 51 callers pass
window borrows. No new Rc-managed type or persistent owner is introduced.

`cargo test --workspace` and `git diff --check` passed. Raw parent/session
relationships at callers and remaining group traversal APIs still require
migration.

## Implemented session borrows for group lookup and redraw

Session group lookup takes an optional shared session borrow and compares member
Weak identities, removing casts from weak session pointers. Missing sessions
still produce no group. Group full/status redraw accepts a session borrow and
borrows retained members for updates. All lookup and redraw callers are adapted.
Group storage remains its existing non-Rc representation; no new Rc-managed type
or persistent owner is introduced.

`cargo test --workspace` and `git diff --check` passed. Raw session acquisition at
legacy callers and broader model relationships remain pending.

## Implemented retained pane traversal for session maintenance

Session theme propagation accepts an optional session borrow, preserving the
missing-session no-op. History-limit updates borrow their session and options.
Both retain each pane cursor while updating fields, replacing immediately
projected raw pane traversal pointers. History trimming borrows the grid directly
and preserves its existing collection and logging behavior. All callers are
updated; no new Rc-managed type is introduced.

`cargo test --workspace` and `git diff --check` passed. Winlink/window acquisition
and remaining session APIs, including linked-session queries, remain pending.

## Implemented weak session activity timer captures

Activity updates borrow their session and accept an optional copied timestamp,
removing raw session inputs and timestamp aliasing during creation. Lock timers
capture the existing session Weak observer, upgrade for dispatch, and check
registry liveness and attachment before locking. The callback retains its session
through locking and resize recalculation without creating a self-owning timer
cycle. Lock timeout lookup uses borrowed options. No new Rc-managed type is added.

`cargo test --workspace` and `git diff --check` passed. The server lock API still
requires a raw session at its boundary; broader session/client relationships
remain pending.

## Implemented typed session and client lock dispatch

Server/session lock traversal retains each registry client through terminal
locking. Session locking accepts an existing session Rc and matches weak session
identity. Client locking accepts the borrowed existing client handle; command and
activity-timer callers forward their owners. Traversal advances after locking as
before. No new Rc-managed type or persistent owner is introduced.

`cargo test --workspace` and `git diff --check` passed. Client-lock internals still
project a raw client for terminal operations and session option lookup; those
accesses and broader model relationships remain pending.

## Implemented weak pan-window identity

`client.pan_window` now stores the existing window's weak observer. Terminal offset
calculation and `refresh-client` compare allocation identities only while the
window is live; clearing stores an empty weak handle. The field adds no window
reference and can no longer retain a dangling address after the window dies. A
focused test covers identity, expiration and unchanged strong count.

The nested mode-tree rebuild test also replaced its raw callback-state tree
pointer with a weak capture that is upgraded for each build. Its rebuild flag is
shared independently, and the callback owns its captures without manual raw-Box
cleanup. The control-stop cleanup test also replaced its raw client address with
a weak client observer; its Drop assertion upgrades while the caller retains the
client. All three focused tests pass. The restored raw-field TSV now reconciles
with the current scanner; 85 in-scope raw fields remain.

## Implemented owned copy-mode word separators

Copy mode now stores selected word separators as `Option<CString>`. The mode
snapshots the option when word selection starts, so later cursor movement reads
its own stable string even if the option entry is replaced. A focused regression
replaces that option and verifies the stored selection still reads the original
bytes. The field inventory now counts 84 remaining in-scope raw fields.

## Implemented owned spawn window names

`spawn_context.name` now stores `Option<CString>`. New-window and new-session
snapshot the requested name while their argument storage is live. Spawn logging
borrows the name, and window creation clones it into the window. An absent name
continues to select the default. The field inventory now counts 83 remaining
in-scope raw fields.

## Implemented owned spawn working directories

`spawn_context.cwd` now stores `Option<CString>`. New, split, respawn, and editor
paths copy the requested directory into the context; spawn formatting borrows the
owned value and keeps the existing absent-directory behavior. Failed spawns
release the snapshot through ordinary Drop. The field inventory now counts 82
remaining in-scope raw fields.

## Implemented weak editor identity

The buffer and customize modes store an `EditorHandle` with a weak reference to
the floating editor pane and a unique editor ID. Spawn returns the handle after
installing the pane-owned `Box<spawn_editor_state>`. Cancellation and PID reads
resolve the current slot only when its ID matches; completion compares the
detached editor before clearing mode state. The stale-handle regression covers
editor replacement and pane expiration. The field inventory now counts 69
remaining in-scope raw fields.

## Implemented weak control block queue

The control state owns output and reply blocks through `RefBox`; each pane queue
stores ordered weak handles to its blocks. A block's handle expires when the
state removes it. Creation returns the weak handle directly, and release keeps
reply-byte accounting and pane queue order. Local legacy pointer projections
still rely on the state retaining the block for each operation. The field
inventory now counts 68 remaining in-scope raw fields.

## Implemented weak queued command identity

Commands are allocated as `RefBox<cmd>` at parse and copy time. The command
list remains their sole owner, and queue items retain their existing list
reference while storing a weak command handle. Moving a command between lists
keeps its address and weak identity; dropping its final list owner expires the
handle. `cmd_ptr` still projects a legacy raw view for queue execution. The
field inventory now counts 67 remaining in-scope raw fields.

## Removed stored current-target alias

`cmd_find_state` no longer stores a pointer to another find state. Each target
lookup selects a local snapshot of the marked, queue, or client context and
passes a borrowed reference to its window and pane helpers. The resolved state
stores only target observers, so no stack context pointer survives the lookup.
The field inventory now counts 66 remaining in-scope raw fields.

## Implemented weak input reply stream

`input_ctx.event` now stores a `StreamHandle` that weakly observes the runtime
stream state. Each reply borrows the live stream slot while writing, and an
expired stream makes the reply a no-op. A focused regression frees the stream
before the input parser sends a reply. The field inventory now counts 65
remaining in-scope raw fields.
