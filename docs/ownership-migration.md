# Ownership migration inventory

Status: refreshed from the current `main` tree at `HEAD 5097d9e` on
2026-09-22. The runtime sources are unchanged from the audited `5d596e0`
baseline; this change restores the deleted document only. This is a design and
validation document, not a runtime-code change.

The translated application still has two different kinds of records:

* Rust records with a real owner and a `Drop` implementation; and
* C-shaped `#[repr(C)]`, frequently `Copy`, records allocated with
  `xcalloc`/released with `free`, whose pointer fields are only meaningful
  when their allocation, alias, callback, and destruction protocol is known.

The second kind must not be made idiomatic by putting `Vec`, `String`, `Box`,
`Rc`, `RefCell`, `OwnedFd`, or another drop-bearing value directly into the
record. The containing record and every C-facing constructor/free path have to
move together.

## Executive summary

The following boundaries are already deliberate ownership implementations and
should be treated as dependencies, not as new migration proposals:

* `EnvironOwner` owns a C-allocated `environ` tree, while
  `EnvironView`/`EnvironEntry` are lifetime-bound borrows.
* `LayoutDescriptionBytes`, `LayoutDescriptionChildren`, and the detached
  `LayoutDescription` tree own custom layout-description data. They do not yet
  own the live `layout_cell` tree or every temporary C-shaped parser buffer.
* `OrderedIndex` and the `RefBox`/`Weak` winlink path provide stable identity
  and weak aliases for selected indexes. They do not make the enclosing C
  records owners.
* `hmux-buffer::SegmentedBuf` owns its segmented bytes and exposes borrowed
  chunks. `src/reactor/buffer.rs` supplies a paired C-compatibility
  `evbuffer_new`/`evbuffer_free` boundary.
* `hmux-rt` owns tasks, timers, readiness state, signal resources, and
  duplicated descriptors. `src/reactor` owns the compatibility side tables
  and cancels tasks before releasing C handles.

The bounded increment selected here is the heap behind
`window_pane_resizes::storage`. Its whole queue lifetime can be represented by
one Rust side owner: lazy allocation, stable entry addresses, borrowed
inspection, retain-one cleanup, alternate-screen cleanup, empty cleanup, and
pane destruction. The two-pointer C handle remains a layout-preserving ABI view
until the containing `window_pane` record is migrated later.

## 1. Current ownership boundaries

### `EnvironOwner` is complete for its scope

`src/environ.rs` contains the clearest existing owner. `environ_create` uses
`xcalloc` for the ABI-visible `environ`, then stores a `Box<environ_storage>` in
`environ.entries`. Entries and their `name`/`value` strings use the project's
`xcalloc`/`xstrdup`/`xvasprintf` allocation family. `environ_free` removes each
entry, calls libc `free` for the strings and entry, drops the boxed ordered
index, and finally frees the C record.

`EnvironOwner` stores a `NonNull<environ>`, is not `Copy` or `Clone`, and calls
`environ_free` in `Drop`. `EnvironView<'a>` and `EnvironEntry<'a>` carry
`PhantomData<&'a EnvironOwner>`; they borrow the tree rather than extending its
lifetime. `into_raw`/`transfer` explicitly suppress `Drop` for a hand-off to a
C caller. `job.rs` and `spawn.rs` keep this owner outside their C-shaped spawn
contexts and drop it on the synchronous call's success and failure paths.

This is an owner around an ABI record, not a drop-bearing field embedded in the
record. Preserve that pattern when migrating `session.environ`, client
environments, or other C-owned pointers.

### Custom layout-description owners are detached owners, not live-layout owners

`src/layout/custom.rs` already has useful, allocator-correct owners:

* `LayoutDescriptionBytes` owns `xmalloc` storage and frees it with libc
  `free`; it deep-clones bytes and does not assume UTF-8.
* `LayoutDescriptionChildren` owns an `xcalloc`/`xreallocarray` array of
  non-`Copy` `LayoutDescriptionNode` values, drops initialized children, then
  frees the array.
* `LayoutDescriptionPane`, `LayoutDescriptionNode`, `LayoutDescription`, and
  `LayoutParseError` compose those owners and therefore clean up a detached
  parse result with ordinary Rust destruction.

`parse_layout_description` copies the input, adopts an allocated parse error
string, and destroys the temporary JSON tree. The detached model deliberately
contains no pane or live `layout_cell` pointer.

The following are still raw/manual and must not be counted as migrated:

* `layout_string { dat, size, capacity }` uses `xmalloc`/`xreallocarray` and
  `free` through `layout_string_init`/`layout_string_free`.
* `layout_parse_ctx` owns a `cctxs` allocation and conditionally owns a live
  root during parsing; `layout_parse_free_ctx` and `layout_parse_cleanup` are
  the protocol rather than a Rust `Drop` type.
* live `layout_cell` trees are allocated and released through
  `layout_create_cell`/`layout_free_cell`; parent and intrusive-list links are
  aliases, not ownership indicators.
* `layout_dump` returns an `xasprintf`/`xmalloc` C string for the caller to
  free. Serialization borrows live cells synchronously.

The detached owners are a prerequisite for later parser migration, but they do
not justify changing the live layout ABI in the same increment.

### `hmux-buffer` is already an owned storage implementation

`hmux-buffer/src/segmented.rs` owns a `VecDeque<Segment>`, each segment owns a
`Vec<u8>`, and `Drop` is supplied by those Rust containers. `From<Vec<u8>>`
adopts an initialized allocation without copying. `SegmentedBuf::append` moves
segments from the source, leaving it empty. `Chunks<'a>` and `pullup` return
borrows whose validity ends at the next relevant mutation or destruction;
`read_line` returns an owned `Vec<u8>`.

`src/reactor/buffer.rs` deliberately keeps the application-facing ABI raw:
`evbuffer_new` is `Box::into_raw(Box<SegmentedBuf>)`, `evbuffer_free` is the
matching `Box::from_raw`, and `evbuffer_readln` returns libc-`malloc` storage
for its caller. The `bufferevent.input` and `.output` fields are therefore
still raw handles in C-shaped records. The buffer implementation itself is
not new ownership work.

### `hmux-rt` and the reactor boundary are already lifecycle-aware

`hmux-rt` owns a local runtime core, task futures, timers, readiness tables,
signal resources, and `Rc<OwnedFd>` descriptor leases. `Task::Drop` cancels and
removes its future; `Sleep::Drop` removes its timer; `IoState::Drop`
deregisters/clears readiness and releases its descriptor lease; signal resources
unregister subscriptions; and `Runtime::Drop` closes the runtime tables. The
public `Descriptor` keeps an `Rc<OwnedFd>` lease and either a registered `Io` or
a zero-time-poll fallback.

`src/reactor` adds the C compatibility protocol: `StreamState` owns the
`hmux-rt` task and a `live` flag; `STREAMS` and `BUFFERS` hold side-table
references; `bufferevent_free` removes those references, cancels the task,
restores descriptor flags, frees both buffers, and finally drops the boxed C
handle. Callbacks are invoked without a registry borrow, and the task checks
`live` after a callback because the callback may free its owner.

Do not propose replacing `hmux-rt`, `SegmentedBuf`, or the reactor task
lifecycle as part of application-record ownership migration. The remaining
work is to make the raw fields that point at these boundaries explicit at their
call sites.

### Existing index owners are useful but partial

`src/shared/tree.rs` stores a Rust `BTreeMap` plus selected `RefBox<T>` owners
behind a raw `storage` pointer. The winlink path uses `insert_owned`,
`take_owned`, and `downgrade` to preserve stable addresses and weak visit
history. `src/shared/session_refbox.rs` is a tested `SessionWindows` design
probe, not a replacement for the translated `session` record. The raw record,
its C allocator, and its aliases still have to move together.

## 2. What is already RAII, and what is still raw

The following are existing ownership implementations to reuse, not new work:

| Existing owner | What it actually owns | Raw fields that remain at its boundary |
| --- | --- | --- |
| `EnvironOwner` (`src/environ.rs:13-22,158-160`) | The C `environ` record, its `environ_storage` index, entries, and entry strings through `environ_free` | `environ.entries`, `environ_entry.name/value/owner`, and any borrowed `EnvironView` pointers |
| `LayoutDescriptionBytes` and `LayoutDescriptionChildren` (`src/layout/custom.rs:130-332`) | Detached layout bytes and detached child nodes with allocator-correct `Drop` | `layout_string`, `layout_parse_ctx`, live `layout_cell` trees, and `layout_dump`'s returned C string |
| `OrderedIndex`/`RefBox` (`src/shared/tree.rs:8-17,77-130`) | Selected index maps and, for winlinks, stable node owners plus weak aliases | Most containing C records, their `storage` handles, and all non-owning tree/list links |
| `SegmentedBuf` plus reactor `StreamState` | Segmented bytes, boxed `evbuffer` handles, task registration, and callback-safe side state | `bufferevent.input/output`, `bufferevent.cbarg`, and the C-shaped `bufferevent` record |
| `hmux-rt` (`hmux-rt/src/mio/runtime.rs:27-40,95-130,181-189`) | Tasks/futures, timers, readiness registrations, signal subscriptions, and descriptor leases | Application records still store raw descriptors, event handles, and callback arguments |

A raw field pointing at one of these owners is not itself a missing RAII
implementation. Conversely, a C-shaped `Copy` record that contains a pointer
to an allocation is not made safe merely because the allocation happens to be
backed by `Box`, `VecDeque`, `Rc`, or `OwnedFd`. The owner, C view, assignment
sites, callback revocation, and destructor must be migrated as one boundary.

## 2. ABI rules that constrain every proposal

1. Declarations in `src/shared` are the authoritative C-shaped declarations.
   They are commonly `#[repr(C)]`, `Copy`, embedded in larger records, passed
   by value in translated code, and allocated with `xcalloc`/released with
   `free`. A Rust owner may sit beside such a record or behind a pointer-sized
   handle; it must not be embedded until the containing record is migrated.

2. `src/ffi` is the only foreign-declaration provider. The current architecture
   guard counts 207 foreign functions, 5 foreign statics, and 17 opaque foreign
   types, and rejects foreign blocks elsewhere. New wrappers must use those
   declarations rather than adding a duplicate ABI.

3. Callback types are ABI, not ordinary Rust function pointers. The current
   guard requires every bare callback type to be `unsafe extern "C" fn`; all
   callback fields remain `Option<unsafe extern "C" fn(...)>` with the existing
   argument and return types. A callback's `*mut c_void` argument does not by
   itself prove ownership.

4. Allocators cannot be mixed. `xmalloc`/`xcalloc`/`xreallocarray`/`xstrdup`
   storage is released with libc `free`; `Box::into_raw` storage is released
   with `Box::from_raw`; `RefBox` storage is released by its `RefBox` owner;
   foreign factories require their matching foreign destructor. This is a
   deallocation rule, not a stylistic preference.

5. Public raw-pointer symbols retain their signatures in an ownership
   increment. This includes `environ_create`/`environ_free`, layout parse and
   dump functions, `bufferevent_new`/`bufferevent_free`, and C callback entry
   points. Rust owners may be introduced at the Rust boundary with explicit
   `from_raw`/`into_raw` or a C view, but the ABI is not silently changed.

6. The frozen layout tests describe the Linux x86_64 translation baseline.
   The Nix development matrix also names aarch64-darwin, but the root build
   links Linux-oriented libraries unconditionally and the numeric layout
   fixtures are not portable across libc/OS/architecture. Numeric assertions
   must be target-gated or supplied with a per-target fixture; pointer-pair
   invariants may be tested separately on all supported 64-bit targets.

7. Foreign or system-owned pointers (`FILE`, terminfo/ncurses objects,
   `regex_t`, `DIR`/glob state, passwd/group records, socket and signal
   structures, and opaque event bases) remain raw or opaque until the matching
   API and destructor are modeled. A Rust `Box<T>` is not a valid substitute
   for a foreign allocation.

## 3. Pointer-field classification

The labels below are used consistently in the inventory:

* **A — allocation:** who allocates the pointed-to object and with which
  allocator.
* **S — assignment:** the constructor, insertion, replacement, or attach path
  that writes the field.
* **X — aliases:** back-pointers, intrusive links, indexes, and other pointers
  to an object owned elsewhere.
* **B — borrowing:** how long a dereference or returned view may be used.
* **T — transfer:** a hand-off, detach, adoption, move, or weak/reference
  conversion.
* **C — callbacks:** callback arguments, re-entry, and whether a callback can
  release the owner.
* **D — destruction:** the exact drop/free/remove/cancel operation and order.

This is the current candidate inventory. It groups fields with the same
protocol; the named fields are the source locations to audit before changing a
record.

| Candidate fields / family | A — allocation | S — assignment | X — aliases | B — borrowing | T — transfer | C — callbacks | D — destruction / status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `environ.entries`, `environ_entry.name/value/owner` (`src/environ.rs`, `src/shared/environment.rs`) | `xcalloc` C record; boxed `environ_storage`; C strings/entries via `x*` allocators | `environ_create`, `environ_insert`, set/clear/unset replacement | `entry.owner` points back to the environment; map values point at stable C entries | `EnvironView`/`EnvironEntry` are synchronous lifetime-bound views | `EnvironOwner::into_raw`/`transfer` and `from_raw_owned` | Spawn/job calls borrow the raw tree synchronously; no async callback owns it | `EnvironOwner::Drop` → `environ_free`; complete owner boundary; keep raw ABI records |
| `layout_string.dat`, `layout_parse_ctx.root/cause/cctxs`, live `layout_cell` links (`src/layout/custom.rs`, `src/shared/layout.rs`) | `xmalloc`/`xreallocarray`; parser context array; live cells through `layout_create_cell` | init/write/append, parser context growth, attach/commit root | parent, `tqe_*`, and cell references are aliases/intrusive links | serialization and live-tree traversal borrow synchronously | adopt error C string; detach root on successful commit; bytes transfer into `LayoutDescriptionBytes` | no owner callback in the parser; `layout_dump` result is a caller-owned C string | `layout_string_free`, `layout_parse_free_ctx`, `layout_free_cell`, JSON destroy; detached model is RAII, live parser records remain raw |
| `window_pane.resize_queue.storage` and returned `window_pane_resize *` (`src/shared/pane.rs:29-70`) | lazy `Box<VecDeque<Box<window_pane_resize>>>`; the handle is inside xcalloc storage | `window_pane_create` initializes null; `window_pane_resize` pushes entries | each entry is separately boxed for stable identity; `entry.tqe_*` are unused alias-shaped links; `window_pane` copies the handle | `server_client_check_pane_resize` borrows only while copying scalars and `last_ptr`; `screen_write_alternateon` checks/clears synchronously | `clear_except(last_ptr)` retains by address; no external transfer | resize timer has raw `window_pane` data; `window_pane_send_resize` and pane/bufferevent paths require no queue borrow across re-entry | `server_client`, `screen_write`, and `window_pane_destroy` clear; `window_pane_remove_ref` calls `window_pane_free`, which currently does not clear, so the implementation must close that final path; selected bounded increment below |
| `*.storage` index fields (`winlinks`, `windows`, `window_pane_tree`, `session`, `options`, `client`, `control`, `monitor`, `hyperlinks`, `key`, and related records) | usually `Box<OrderedIndex<...>>` or a boxed Rust map behind a raw pointer; selected nodes use `RefBox` | first insert allocates; insert/remove updates field and back-pointer | record `owner` fields and index values are aliases; raw nodes require stable addresses | lookup/traversal borrows records while index is alive | `insert_owned`/`take_owned`/`downgrade`; map movement must not move owned nodes | callbacks may use owner/back-pointer and re-enter removal | last removal drops the map; node destruction belongs to its separate owner/refcount; partial migration |
| Owned C strings and arrays: session/window/client/pane names, paths, `argv`, `term_caps`, option values, argument values, JSON keys/strings, menu/prompt text | `xstrdup`, `xasprintf`, `xmalloc`/`xreallocarray`, or foreign allocation; several arrays have element-by-element ownership | constructors and setter paths replace old pointers; arrays are attached after count/capacity setup | exported raw pointers alias the containing record or an array element; static table strings are borrowed | C calls receive temporary `CStr`/slice views; no UTF-8 assumption | returned strings (`layout_dump`, read-line results, command lists) transfer to their caller; `cmdlist` and argv paths hand off explicitly | callback data may be a separately allocated payload or borrowed string; do not infer from `*mut c_void` | matching `free`, `cmd_free_argv`, `cmd_list_free`, or foreign free in record-specific order; still raw |
| Nested trees/lists: `layout_cell`, grids/screens, JSON nodes/tokens, options/arguments trees, control lines/blocks, messages, jobs, client files, mode data | C records via `xcalloc` or `Box`; child arrays and strings use their own allocator | append/insert/attach writes parent and intrusive links | parent/next/prev/tree-entry pointers alias owner collections; some nodes are refcounted | traversal is a borrow; returned node pointers cannot outlive removal/destruction | detach/reinsert, move `Vec` segments, or `RefBox` weak conversion must be explicit | callbacks can remove the current node or its containing record; no collection borrow may cross callback | recursive `*_free`/drop, often after unlinking and canceling events; migrate owner and C view together |
| Foreign/resource handles: `event.ev_base`, pane/client `bufferevent`, `bufferevent.input/output`, `FILE`, fd, terminal/regex/glob objects | foreign factories, libc, `OwnedFd`, or reactor's boxed compatibility handles | factory result assigned into record; descriptor leases enter side tables | opaque pointer and callback `cbarg` are not Rust ownership; fd numbers are borrowed identities | use only while the foreign handle is live; buffer chunks end at mutation | `OwnedFd::into_raw_fd`/`from_raw_fd`, `fdopen`/`fclose` contracts, or explicit `bufferevent_free`; never generic `Box` | reactor callbacks may free the stream; `StreamState.live` is checked after callbacks | matching foreign destroy/`close`/`fclose`, task cancellation, `evbuffer_free`; reactor boundary already handles its side state, record fields remain raw |
| Callback/context fields: `event_callback.evcb_arg`, `bufferevent.cbarg`, `tmuxpeer.arg`, job/prompt/menu/mode/event-payload data | caller-owned `Box`/C record, static data, or borrowed application object; unknown from type alone | callback and argument must be installed as one protocol; clear/revoke before owner drop | callback context aliases the registered owner; `data` may be a weak identity or an owned payload | callback receives a temporary raw view; lifetime is registration-specific | registration may transfer a boxed context to the callback/finalizer; require a named hand-off | exact C ABI, re-entry, and callback-can-free-owner cases are central | cancel/delete event, run finalizer if required, reclaim context exactly once, then free owner; still needs typed representations |
| Non-owning links: session/window/pane/options pointers, `parent`, active/modal pointers, `tqe_*`/`rbe_*` links, format-tree context, target pointers | no allocation at the link itself | attach/detach, list/tree rotation, active selection | aliases and back-pointers only; an `owner` field is not an owner | borrow until detach or containing owner destruction; clear links before drop | normally no transfer; use `NonNull`, `Weak`, or an identity handle only in a Rust owner | callbacks may observe stale links unless removal revokes them first | unlink/clear before destroying the owner; raw ABI alias remains until containing record migration |

The important distinction is that a pointer field can be **owned by a side
allocation** while remaining a raw field in the C record. That is the current
state of environments, indexes, buffer handles, and the selected resize queue;
it is not the same as putting an RAII value in the record.

## 4. Proposed Rust representations

These are target representations for future increments, not types to add all at
once.

| Ownership problem | Proposed Rust representation | ABI/containing-record rule |
| --- | --- | --- |
| C-allocated NUL-terminated bytes | `CStrOwner { ptr: NonNull<c_char> }` (or a byte-oriented `CBuffer`) with `Drop` calling the matching C `free`; `CStr`/`&[u8]` views | Do not use `CString` if the pointer must be released by `free` from the `x*` family; keep a raw pointer in a C view until the containing record moves |
| C-allocated pointer arrays | `CArray<T>`/`CStringArray` with length, capacity, initialized-element cleanup, and allocator-specific `Drop` | The C view retains `*mut T` plus count/capacity; array ownership belongs to a Rust owner, not to a `Copy` header |
| Rust index behind a C record | `IndexOwner<K, T>` wrapping `Box<OrderedIndex<K,T>>`; `RefBox<T>` for stable nodes and `Weak<T>` for non-owning history | C record keeps a nullable pointer-sized handle and owner/back-pointer fields; create, insert, remove, and last-owner cleanup migrate together |
| Intrusive child/tree ownership | `Box<T>`/`VecDeque<Box<T>>` or a dedicated `OwnedList<T>`/`TreeOwner` with a C-layout node view | Link fields remain raw in the view; unlink before `Drop`; never let a Rust collection move a node whose C address is retained |
| Live layout cells | `LayoutCellOwner` whose `Drop` calls `layout_free_cell`, plus an explicit `detach`/`into_raw` commit operation | Keep `layout_parse`/`layout_dump` raw signatures; the owner must live outside `layout_parse_ctx` until the root is committed |
| Custom parser scratch | `LayoutStringOwner` and `LayoutParseContextOwner` with explicit adoption of `cause` and root transfer | `layout_string`/`layout_parse_ctx` can first become private Rust owners; public C entry points still use a view or raw pointer |
| Selected resize queue | `ResizeQueueOwner { entries: VecDeque<Box<window_pane_resize>> }` behind a nullable pointer-sized handle | `window_pane_resizes` stays `#[repr(C)]`, two pointers, and `Copy` as an ABI view; do not embed `ResizeQueueOwner` in xcalloc/free `window_pane` |
| Reactor byte/event handles | Rust-only `EvBufferOwner`/`BufferEventOwner` around the existing `Box` and `hmux-rt` side state; `OwnedFd` for duplicated descriptors | Keep `bufferevent`/`evbuffer` public raw symbols and fields; retain the existing cancellation-before-free order |
| Callback context | A function-specific `CallbackOwner<T>`/trampoline with one explicit ownership mode (`Box`, `Rc`/`Weak`, borrowed, or static) | The trampoline remains `unsafe extern "C" fn`; registration and revocation must be paired and re-entrant-safe |

`NonNull<T>` is useful inside an owner or a borrow wrapper, but it is not a
substitute for deciding who drops `T`. `Rc`, `Weak`, and `RefBox::Weak` model
identity/borrowing only when their actual owner is already defined.

## 5. Selected bounded increment: migrate the resize-queue lifetime

### Current complete path to preserve

The relevant path is small but is not limited to the one focused test:

| Phase | Current code and ownership fact |
| --- | --- |
| Containing record | `window_pane_create` allocates the `window_pane` with `xcalloc` (`src/window.rs:2947-2969`) and initializes `resize_queue.storage` and `reserved` to null. The record is `#[repr(C)]`, `Copy`, and freed by `free`; it cannot contain a drop-bearing owner in this increment. |
| Allocation | `window_pane_resize` (`src/window.rs:3245-3275`) calls `window_pane_resizes::push_back` (`src/shared/pane.rs:45-52`). The first push allocates one boxed deque; every entry is separately boxed so its address survives deque growth. |
| Inspection/retain | `server_client_check_pane_resize` (`src/server_client.rs:2943-2990`) copies all dimensions and the predecessor before calling `window_pane_send_resize`; only then does it clear all entries or retain the entry identified by `last_ptr`. The queue borrow must remain lexical. |
| Alternate-screen path | `screen_write_alternateon` (`src/screen_write.rs:5306-5318`) clears pending entries before `layout_fix_panes`, then may send the current size and clears again. Both cleanup calls are part of the queue lifetime and must migrate with the owner. |
| Explicit cleanup | `window_pane_clear_resizes` (`src/window.rs:3237-3243`) calls `clear_except`; null storage is a harmless no-op, and a successful clear must null the handle after `Box::from_raw`. |
| Pane destruction | `window_pane_destroy` (`src/window.rs:3094-3133`) deletes the resize timer before clearing resizes and dropping the final pane reference. `window_pane_remove_ref` (`src/window.rs:1312-1326`) calls `window_pane_free` at zero references. `window_pane_free` (`src/window.rs:3135-3152`) currently frees other pane fields and the xcalloc record but does not clear `resize_queue`; the future increment must make the final free path explicitly idempotent, with destroy's earlier clear becoming a null no-op. |
| Callback/re-entry constraint | `server_client_resize_timer` stores `wp` as `*mut c_void` (`src/server_client.rs:2893-2905`); reactor callbacks store `bufferevent.cbarg` and can free the stream. No owner borrow or saved entry pointer may survive a callback, pane free, or queue clear. |

This is the entire allocation and cleanup surface for the side allocation. The
implementation must test both normal destroy ordering and a direct/defensive
`window_pane_free` path so there is exactly one `Box::from_raw` for each
non-null storage pointer.

### Proposed owner and containing-record boundary

Introduce one private owner for the heap object, conceptually:

```rust
struct ResizeQueueOwner {
    entries: VecDeque<Box<window_pane_resize>>,
}
```

The `window_pane_resizes` ABI shape remains a nullable two-pointer handle:

```rust
#[repr(C)]
#[derive(Copy, Clone)]
struct window_pane_resizes {
    storage: *mut window_pane_resize_storage, // points at ResizeQueueOwner
    reserved: *mut c_void,                    // remains null
}
```

`window_pane_resize_storage` may become an alias for the owner, so the field
still has one pointer and the same offset. It must not be replaced by an
embedded `VecDeque` or by a field with `Drop`. `window_pane` remains an
xcalloc/free C record in this increment. Its copied C handle is therefore an
unsafe ABI view, not a second Rust owner; a future `window_pane` owner/view
migration must remove that copy hazard separately.

The owner methods must provide this entire lifetime:

* first push: allocate exactly one `Box<ResizeQueueOwner>` and one boxed entry;
* subsequent push: add boxed entries without moving their pointees;
* borrow: expose a queue view only for the synchronous inspection block;
* retain: compare the saved entry identity, drop every other boxed entry, and
  keep the owner alive only if the retained entry remains;
* clear: consume the raw owner with `Box::from_raw`, drop all entries, and set
  `storage` to null before returning;
* pane cleanup: make the destroy/free invariant explicit so every path that
  releases a pane also handles a non-null queue, while an already-cleared
  queue is harmless.

This is a bounded ownership migration because it does not change the pane
event ABI, the callback ABI, the pane allocation, or the runtime. It does
migrate every allocation and cleanup path for the queue side allocation. The
remaining raw pointer is the compatibility handle required by the containing
record, not an untracked allocation.

### Dependencies and implementation files

The implementation must update these paths together:

* `src/shared/pane.rs`: define the owner, handle operations, identity-safe
  retain, and nulling cleanup; keep the C handle layout.
* `src/shared/pane.rs`: define the side owner, handle operations,
  identity-safe retain, and nulling cleanup; keep the C handle layout and its
  `Copy` status for now.
* `src/window.rs`: cover the constructor initialization, enqueue, public clear,
  `window_pane_destroy`, `window_pane_remove_ref`, and `window_pane_free` paths;
  make cleanup idempotent before the xcalloc record is released.
* `src/server_client.rs`: keep the queue borrow lexical, copy every value before
  `window_pane_send_resize`, and never retain a pointer past clear or callback
  re-entry.
* `src/screen_write.rs`: preserve both alternate-screen clears and their order
  around `layout_fix_panes` and `window_pane_send_resize`.
* `tests/pane_resize_queue.rs`: retain order, stable entry addresses,
  exception removal, empty cleanup, null-handle cleanup, and repeated cleanup;
  add a destroy/free-order test that detects a second drop.
* `tests/consolidation_layout.rs` and `tests/model_pane.rs` plus the relevant
  fixtures: continue checking `window_pane_resizes`, `window_pane_resize`,
  `window_pane_resize_entry`, and every translated copy. If `window_pane` is
  touched, update its `resize_queue` offset snapshot.
* `tests/architecture.rs`: retain the foreign/shared-declaration/callback
  guards and add an owner-boundary guard that rejects a drop-bearing queue owner
  embedded in a `#[repr(C)]` shared record; add a source-level assertion that
  each non-null queue storage is consumed exactly once. The current guard does
  not provide either ownership/double-drop assertion yet.

This increment has no dependency on `hmux-buffer` or `hmux-rt`; those are
already-tested owners at the reactor boundary. Its dependency is the existing
`window_pane` C-view contract and the callback/timer invalidation order. The
custom-layout and reactor owners remain prerequisites for later increments, not
part of this queue change.

## 6. Migration order after the bounded increment

1. **Keep the inventory and guards current.** For every candidate, record the
   allocator, field assignment, aliases, borrow duration, transfer operation,
   callback/re-entry rule, and destructor. Add an ABI/layout assertion before
   changing a shared record.
2. **Complete the resize-queue side owner.** It is the smallest nested
   allocation with a real stable-address and callback-adjacent lifetime.
3. **Finish custom-layout scratch ownership.** Wrap `layout_string` and
   `layout_parse_ctx`, then model live-root commit as an explicit owner detach.
   The existing detached `LayoutDescription*` types are the target model.
4. **Migrate leaf storage with simple owners.** Use allocator-specific C string
   and array owners for arguments, JSON, options, format entries, messages,
   control lines, and screen/grid child storage. Keep C views for translated
   call sites and test recursive cleanup.
5. **Migrate stable identity and nested application records.** Move pane/grid
   and screen owners before window/winlink owners; move windows and winlinks
   before session aliases; move sessions/windows before client, job, and
   control ownership. Reuse `OrderedIndex`, `RefBox`, and `Weak` rather than
   copying records whose addresses are retained.
6. **Type callback contexts before converting callback-bearing records.** Each
   `cbarg`/`data` field needs an explicit owner, borrow, weak identity, or
   static policy, plus cancellation/finalization ordering. Preserve the C ABI
   trampoline types.
7. **Wrap remaining foreign resources at the boundary.** Use `OwnedFd`, an
   allocator-correct file wrapper, or a function-specific foreign handle
   wrapper only where a matching destructor exists. Do not replace the already
   working `hmux-rt`/reactor lifecycle.
8. **Only then split Rust owners from C views of large records.** `session`,
   `window`, `window_pane`, `client`, and similar records can stop being
   drop-in `Copy` C storage only after all callers, callbacks, allocation paths,
   and layout fixtures have moved to the owner/view boundary.

The dependency direction is intentional: leaf ownership and callback
revocation must be settled before parent records can safely drop their child
graphs. A raw alias should become a Rust borrow or weak identity only after its
owner and invalidation point are known.

## 7. Validation evidence and limitations

The retained review record reports that the workspace test run and 1,639
conformance tests passed (`test.log:3295` and `test.log:10608`). It also records
that `cargo fmt --check` failed and that strict Clippy failed. Those failures
are baseline evidence to resolve or explain before an ownership implementation
is called green; this documentation refresh does not claim to fix them.

The sanitizer/leak evidence was limited to focused runs
(`implement.log:161-213,261-264`), not the complete application lifetime.
In particular, it does not prove that alternate-screen cleanup, every
pane-reference destruction path, callbacks that free their owner, or all
allocator pairs are covered. Linux Valgrind/ASan is available in the Nix
development shell; Valgrind is intentionally unavailable on
`aarch64-darwin` (`flake.nix:10-13,88,117-120`). The repository's numeric
layout fixtures and the root application link are Linux-oriented, so a
non-Linux run cannot be treated as equivalent without target-specific guards
or fixtures.

## 7. Validation commands and architecture guards

The ownership implementation should run the focused checks first and then the
full workspace checks:

```sh
cargo fmt --check
cargo test --test pane_resize_queue
cargo test --test architecture --test ffi_boundary --test consolidation_layout --test consolidation_callbacks
cargo test --test pane_resize_queue --test layout_custom --test reactor
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

For the existing application-level behavior, build the binary and run the
private-socket scripts from the repository root:

```sh
cargo build
python3 scripts/layout_cli_checks.py
python3 scripts/key_cli_checks.py
```

The selected increment must keep or extend these guards:

* `tests/consolidation_layout.rs` currently compares the size, alignment, and
  fields of copied `window_pane_resizes`, `window_pane_resize`, and related
  records across translated modules. The implementation must add explicit
  size/alignment/offset assertions for the two-pointer handle and update
  `tests/model_pane.rs`/fixtures if the containing pane view changes. Numeric
  snapshots from the current fixtures are Linux x86_64-only and require an
  explicit `#[cfg(all(target_os = "linux", target_arch = "x86_64"))]` guard or
  a target-specific fixture.
* A portable pointer-shape guard may assert that `storage` and `reserved` are
  pointer-sized and that `reserved` stays null after construction and cleanup;
  this can run on both Nix-supported targets without claiming identical libc
  layouts.
* `tests/pane_resize_queue.rs` currently covers one order/retain/clear scenario
  (`lines 17-69`). It must cover first allocation, multiple stable addresses,
  retain-one, clear-all, empty-state nulling, alternate-screen-style clearing,
  direct/free-order cleanup, and repeated cleanup. It should exercise cleanup
  from the same order used by pane destruction and prove no double `from_raw`.
* `tests/architecture.rs` currently enforces one authoritative shared
  declaration, all foreign declarations in `src/ffi` (207 functions, 5
  statics, 17 opaque types), and `unsafe extern "C"` callbacks (44 callback
  aliases). It does not yet enforce owner-in-record, allocator-pair, or
  double-drop rules. Add those guards when the new owner is introduced; keep
  `tests/ffi_boundary.rs` and the callback tests green as well.
* `tests/consolidation_callbacks.rs` must remain green; callback ABI types and
  the rule that a callback may free its owner must not change.
* `hmux-buffer` and `hmux-rt` tests remain part of
  `cargo test --workspace --all-targets`; their existing ownership contracts
  should not be weakened to accommodate an application record.
* On Linux, run focused leak/use-after-free checks for the queue and the pane
  destroy path when available (for example `cargo test --no-run` followed by
  the focused test binary under Valgrind, or the Nix shell's ASan toolchain).
  Add the same focused case to the full run before claiming coverage. Do not
  require Valgrind on aarch64-darwin, where the development shell explicitly
  does not provide it. Record the command, target, allocator diagnostics, and
  whether the callback/free re-entry case was actually exercised.

Any implementation that changes a `#[repr(C)]` record, a public raw-pointer
function, a callback alias, or an allocator pair must update the corresponding
architecture/layout guard in the same change. This document intentionally
proposes no runtime code change by itself.

