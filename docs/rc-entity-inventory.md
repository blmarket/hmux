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
| `session` | Session index; event targets/payloads; monitor session; run-shell data. `session_remove_ref` consumes an `Rc` and defers release. | Self observer, find state, monitor changes, mode selections; group membership; raw client session/last-session, winlink session, format/spawn context. | Owners typed; raw relationships and traversal projections remain. |
| `window` | Creation returns `WindowOwner`; winlinks own `WindowOwner`; alerts and event payloads retain `Rc`. `window_remove_ref` performs live close notification before release. | Self observer, find state, menus, monitor changes; pane parent and format/redraw context. Global index is weak. | **Migrated:** index entries, index traversal, ID lookup and all their callers retain `WindowOwner` while accessing the window. |
| `window_pane` | Pane index; retained event targets/payloads. `window_pane_remove_ref` consumes an `Rc`; logical destruction removes the index owner. | Pane lists/history, find state, mode tree, prompts and monitor changes; raw active/modal/zoom panes, layout cells, input and render contexts. | Owners and many observers typed; remaining raw relationships and short-lived upgrade projections need migration. |
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

`winlink`, prompts, menus, mode-tree items and several mode item records use
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
