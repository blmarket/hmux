# hmux

The Rust package and executable are named `hmux`. The library target retains
its `tmux_c2rs` name for the transpilation's existing consumers.

The implementation began as a whole-program
[c2rust](https://github.com/immunant/c2rust) transpilation of tmux 3.7b. It now
combines translated command behavior with Rust-owned engine components and the
`hmux-rt` runtime. The end-user compatibility contract is the tmux-compatible
command line, wire protocol, and observable server behavior, with the deliberate
differences documented below.

## What the source is

`src/` is the source of record and is edited directly; the transpiling pipeline
has been retired. Command, server, screen, grid, input, and other subsystems have
their own module directories. Shared entity and compatibility types remain in
`src/types.rs`, with session state in `src/session.rs`. External C declarations
are centralized in `src/ffi.rs`; Rust subsystem calls use their actual Rust
signatures.

Ownership migration is ongoing. Options and several callback-state families use
`Rc<RefCell<_>>` handles and checked borrow guards. Session, window, and client
owners no longer implicitly dereference to payload references; compatibility
access requires an explicit unsafe call. Each pane's window holds
a strong `RustWindowPaneRef`, moved between windows during transfer.
`RustWindowPaneWeak` is the non-owning observation used by command targets,
modes, callbacks, and lookup results. Cloning an observation does not extend
the pane's lifetime; explicitly upgrading it produces a strong reference.
The global pane index stores weak observations without traversing a window's pane list.
Pane moves and swaps preserve allocation identity and update a weak window
back-reference. Stream callbacks use weak pane observations and skip removed
or temporarily detached panes.

Pane destruction unregisters the pane at the equivalent point to tmux 3.7b's
`RB_REMOVE`, tears down its resources, and consumes its sole strong owner at the
equivalent point to `free(wp)`. Outstanding observations then return `None`;
there is no separate live flag or retained pane payload. Destruction rejects
extra strong owners instead of silently postponing the free. Detached panes
remain registered during transfer, and membership-specific lookup checks their
current weak window reference. Pane IDs are never reused during a server's
lifetime.

Borrowed pane interfaces use `&dyn WindowPane` and `&mut dyn WindowPane`.
The payload's fields are private to its owning module; consumers use the trait
and its state capabilities for process, I/O, screen, mode, and window-context
access. `RustWindowPaneWeak::get` and `get_mut` expose these trait borrows and
return `None` after destruction. Both calls remain unsafe: the caller must
exclude conflicting access and teardown for the borrow's lifetime. Concrete
pane storage implements `Default`; the borrowed capability traits do not
require construction support.

Some registries and server state remain process globals. Global collection
wrappers bind to their first accessing thread and reject other threads before
touching their contents. A mode entry's parent-pane pointer is used only before
mode teardown. Mode trees can outlive their modes; they resolve pane IDs and
detach on mode closure. Shared pane payload mutation uses `UnsafeCell` with
explicit unsafe accessors, as window and session compatibility access does.
Callers must exclude conflicting borrows and reentrant teardown; reference
counting alone does not enforce those rules. These are transitional boundaries,
not a completed memory-safety audit.

Copy mode observes its source pane by ID. Its cloned screen survives source
pane destruction, and `refresh-from-pane` then leaves that snapshot unchanged.
The translated C path retained the source address and could access freed memory
when refreshing after the source disappeared.

Public Rust structs and functions are implementation surfaces. Existing public
traits are versioned compatibility contracts; changes to their signatures or
semantics require human signoff.

The 27 expansions of the BSD `tree.h` `RB_GENERATE` macros the transpile
carried — every `*_RB_INSERT`, `*_RB_REMOVE`, `*_RB_FIND` and their colour
helpers, about 12,600 lines — are gone: each tree is a `BTreeMap` keyed by
what its comparison read, and the `rbe_*` link fields are off the elements.
The keys keep the C order, so everything the order is observable through
(`list-sessions`, `next-window`, `show-options`, `list-keys`, `list-buffers`,
`server-access -l`, control-mode subscriptions) comes out as before. Two
behavioural differences come with it:

- `session_destroy` sent its `window-unlinked` notifications in the order the
  tree happened to be shaped, because it emptied the session by repeatedly
  taking the root. It now sends them in window-index order.
- A paste buffer whose order counter wrapped used to be dropped from the
  by-time tree while the by-name tree kept it, since `RB_INSERT` refuses a key
  it already holds. The map replaces instead. Reaching this needs 2^32 buffers
  in one server.

The reference build it was transpiled from used the pinned oracle's configure
flags (`--enable-systemd --enable-utempter --enable-utf8proc --disable-sixel`,
plus its `--sysconfdir`/`--localstatedir`). Those features are compiled in, so a
flag that differed from the oracle would be a behavioural difference in every
result this crate produces: `--enable-utf8proc` alone decides whether
`utf8_towc` goes through utf8proc or libc `mbrtowc`, which changes both the
width tables and the errno left behind on invalid input.

## Trait pointer migration

The raw string convenience methods on `Arguments` and `SessionNameState` have
been removed. Their `Option<&CStr>` accessors cover scoped reads; a caller that
retains a name can use `session_name_owned` or copy the borrowed string.

`OptionsEngine::array_get` and `array_value` now borrow values from their entry
or item. `array_indices` replaces the raw `array_first`/`array_next` cursor pair
with an ordered index snapshot, allowing each subsequent read or mutation to
have its own borrow. Hook consumers clone command handles through `value_command`
before queueing them. `OptionsRef::string_ref` replaces the raw string accessor
with an immutable `Rc<CStr>` snapshot retained independently of the store. Status,
copy mode, rendering, and command consumers borrow that snapshot while reading it.
`OptionsRef::style_value` replaces the cached style pointer
with an independent copy, which remains valid after an option changes.

`OptionsRef::with_entry` retains the owning store and borrows the entry only for
its callback. Scalar reads, option formatting, and option listing use this path.
`local_names` replaces the raw store cursor pair with a name snapshot. Parsing
and prefix matching return owned names before scoped lookup; scalar setters no
longer return entry pointers. `OptionsEngine` has no raw-pointer-returning methods.
`with_entry_mut` holds an exclusive scoped borrow for array edits; the native
adapter also tracks these borrows and rejects conflicting access through clones.
Hook queues retain cloned command handles, and status rendering retains array
strings through `value_string_ref` before releasing the entry borrow.
Removal and reset take a store and name after any entry borrow ends. Initializers
return no entry pointer; subsequent reads and edits use scoped callbacks.
The Rust store also initializes defaults without raw entry pointers and parses
styles from retained text. Format expansion runs outside the store borrow;
cache writes check that the option still holds that text after expansion.

`LongOption` descriptors retain a lifetime-bound flag borrow. Shared and mutable
flag accessors replace the raw flag pointer, and long-option parsing uses a
bounded descriptor slice. The native adapter retains the same borrow lifetime.

`VariadicArguments` borrows initialized backing regions as slices with shared
and exclusive accessors. `VariadicCursor` replaces the unused raw ABI cursor
record; the native adapter keeps the supplied region lengths and borrow lifetime.

`SystemdJobWatch` owns its optional path and returns a scoped string borrow. The
bus reply is copied when the watch is armed; the native adapter also retains an
owned path through replacement and clearing.

`TerminalCommandData` and `TerminalCommandSelection` borrow complete byte slices
and clipboard names. Drawing contexts carry those lifetimes through synchronous
output, including cloned contexts, without copying the payload. Native adapters
retain the same lifetime and expose scoped slice and string borrows.

`UserAccount` returns borrowed strings from owned account snapshots. Name and
UID lookups use the reentrant libc routines with caller-owned storage, and
server access, tilde expansion, user formats, and startup defaults retain those
snapshots while reading their fields. The native adapter owns its strings too.

`CalendarTime` retains its optional timezone in `Rc<CStr>` and returns a string
borrow. Clock drawing and time formats use owned local-time results; nested
format state clones retain the timezone handle. Raw calendar records exist only
for the duration of libc calls.

`SystemdBusError` owns snapshot strings and returns scoped borrows. Its recorded
ownership marker is metadata; native error cleanup belongs to the private guard
that captures the snapshot, independently of later changes to that metadata.

`ImsgMessage` borrows its original body range from the owned message buffer,
independently of the reader cursor. Shared and exclusive slice access replace
the stored raw data pointer. Client, server, and file dispatch decode bounded
bytes and strings; the native adapter owns the body behind its borrowed view.
Header encoding and decoding use fixed byte arrays instead of struct-pointer
casts, retaining imsg native byte order. Header readers receive the current
size limit as a value refreshed before each read, rather than retaining a raw
pointer to the transport owner. Initialized readers can move without leaving a
callback tied to their former address.

`IoVector` retains initialized storage through `IoSliceMut` and exposes shared
and exclusive slice access. Queued writes use `IoSlice` borrows and reads use
`IoSliceMut` borrows through the syscall. Descriptor bookkeeping uses the queue
owner after those borrows end; native adapters retain the same slice lifetime.
The buffer adapter borrows initialized bytes and spare capacity through
`BytesMut`'s separate slice interfaces.

`ControlMessageHeader` borrows a complete initialized ancillary buffer. Its
payload accessors return bounded shared or exclusive slices, and length updates
must fit that buffer. Received ancillary messages are decoded through disjoint,
validated byte ranges instead of header-pointer arithmetic. Native comparisons
cover the platform header encoding, including storage without header alignment.

`MessageHeader` retains exclusive borrows of its optional address, I/O vector
array, and initialized ancillary storage. Its accessors return scoped slices and
its setters retain the replacement buffers. Socket receive calls keep those
borrows through `recvmsg`; the send path borrows immutable regions through
`sendmsg`. Native adapters retain the same buffer lifetimes and enforce storage
bounds on length changes. Raw libc socket headers are local to syscall adapters;
the exported header copies and the ancillary-storage union are removed.

`RegexBuffer` compiles owned libc patterns and returns match offsets for bounded
borrowed strings. The automaton and lookup-table pointers are no longer exposed:
callers use matching operations instead of inspecting opaque libc storage.
Substitutions, format matching, pane search, and copy-mode search share the same
owned guard, so cleanup follows scope and early returns. The chosen POSIX engine
and its per-context matching flags are preserved.

`GlobResult` owns its path slots and safely borrows individual names. Expansion
uses a private libc result, copies names into shared `Rc<CStr>` storage, and
releases the native allocation before returning. `source-file` moves those names
into its work queue and clones their handles for asynchronous reads. The raw
path-vector constructor and public native glob record are removed; libc remains
the pathname-expansion engine.

This migration removes forty-four raw-returning trait signatures. The audited
traits no longer return raw pointers. Unsafe mutation and native ABI operations still carry
caller obligations; this does not make those operations safe. The tmux command and wire behavior is unchanged by these migrations.

## Layout

- `src/main.rs` forwards command-line arguments to the library entry point in
  `src/tmux.rs`.
- `src/cmd/` and `src/server/` implement command dispatch and server lifecycle.
- `src/screen/`, `src/grid/`, `src/input/`, and `src/tty/` handle terminal state,
  input parsing, and attached-client drawing; `src/control/` handles control mode.
- `src/options/` owns option storage and inheritance. `src/types.rs` retains shared
  entity types and transitional compatibility exports.
- `src/reactor/` adapts the `hmux-rt/` runtime to daemon callbacks and I/O.
- `src/fmt_engine.rs` implements C-style formatting using `FmtArg` slices in place
  of C varargs. Format strings and output storage use slice access, and borrowed
  C-string and byte-slice arguments retain their lifetimes through formatting.
  Byte strings stop at their slice boundary, NUL, or precision limit. Raw-pointer
  string arguments remain for callers awaiting migration.
  `src/ffi.rs` declares external C functions.
- `src/tests/` and subsystem-local test modules cover engine behavior. The parent
  repository supplies the tmux conformance harness and validation gates.
- `hmux-agent/` contains the shared agent-classification implementation.

## Run

```sh
nix develop
make            # cargo build
make check-tmux # refuse a reference tmux that is not 3.7b
```

`make clean` runs `cargo clean`.

## Event loop

Hyperlink sets share an eviction registry within their server thread. Each set
retains the registry owner through cleanup, including thread exit; separate
threads have independent registries and ID counters. Exhausted hyperlink ID
counters fail explicitly rather than recycling identities that grid cells or
queued eviction entries may still reference.

Key tables, named wait channels, and paste buffers also use checked thread-local
registries. Queue wakeups, client updates after table removal, and paste-buffer
notifications run after their registry borrows end, so observers can revisit the
committed state. The starting environment uses checked thread-local storage;
command output and expansion callbacks consume owned snapshots after its borrows
end. Other server collections are still being migrated.

UTF-8 width overrides and temporary width suppression are confined to the
calling thread. Packed character IDs use a shared, synchronized intern store so
encoded grid characters retain their meaning across callers and threads.

Pane, window, and session IDs are never recycled within a server thread. After issuing
the final 32-bit ID, subsequent creation panics instead of wrapping to zero and
allowing a stale observation to address a replacement entity. This differs from
the reference at ID exhaustion; pane activity-order stamps retain their existing
wrapping behavior. The `next_session_id` format is empty once session IDs are
exhausted.

Job IDs also never wrap. Once exhausted, starting a job returns failure with
`EOVERFLOW` before opening descriptors or forking; callback data is released.
Failed startup attempts consume their reserved ID.

Collected screen items use a shared, synchronized pool with FIFO reuse. Released
items remain readable until reuse; allocation panics before an index could
become the reserved sentinel or wrap to an existing item.

The daemon uses the repository's `hmux-rt` runtime and its mio readiness
backend. Timers, descriptor watches, signals, deferred callbacks, and buffered
streams are represented as runtime tasks; the stream input and output sides use
the same segmented `Buf` implementation.

The compatibility host dispatches at most 64 ready tasks before and after a
poll, and a stream drains at most 64 read or write operations before yielding.
When idle, the host waits for at most 10 ms before returning to the daemon's
housekeeping loop. Consequently, simultaneous timer, I/O, signal, and deferred
work may be delivered in a different order or batch size than the reference
libevent loop. This is an intentional scheduling boundary; the existing
callback interfaces and wire-facing behavior remain the compatibility target.

## Plugins

The server carries a plugin layer: a plugin is a bundle of format variables
tmux does not have, plus whatever work it takes to keep them current. It
publishes an id-keyed dictionary — pane id and variable name in, string out —
which `format_find` consults after its own static table, so a plugin's
variables expand anywhere a built-in one does: status formats, `list-panes -F`,
`display-message`, and control-mode `refresh-client -B` subscriptions.

Writing one is implementing `plugin::Plugin`:

    fn name(&self) -> &'static str;              // enabled by this name
    fn variables(&self) -> &'static [&'static str];
    fn interval(&self) -> Option<Duration>;      // how often tick runs
    fn option_defaults(&self) -> &'static [(&'static str, &'static str)];
    fn start(&mut self, host: &dyn Host);
    fn tick(&mut self, host: &dyn Host);
    fn resolve(&self, pane: PaneId, key: &str) -> Option<String>;
    fn on_notify(&mut self, event: &Event<'_>);

and handing it to `plugin::register`. Nothing else in the server has to learn
about it: the variables start expanding, a shared timer picks up the tick, and
the option defaults go in. A built-in plugin is one line in `plugin::builtins`.

Values are pulled, not pushed. `resolve` runs only when an expansion actually
names one of the plugin's variables, so an expensive value costs nothing in a
format that never mentions it — which is why the trait has no lazy-value arm.

What a plugin can read is `plugin::Host`: the pane observability contract from
the `hmux-agent` crate — pane ids, child process, output revision, screen tail,
title — plus `invalidate(pane)`, which marks the pane's window for a status
redraw. Panes are named by id and resolved per call, so plugin state can never
reach a destroyed pane through a pointer it kept.

### Enabling them

`TMUX_C2RS_PLUGINS` is a comma-separated list of plugin names, or `all`, or
`none`. Unset runs the default set, which is the agent and git plugins: a
server nobody has configured is the one worth running.

`TMUX_C2RS_PLUGINS=none` — or an empty value, which is what a shell leaves
behind for a variable someone wanted cleared — turns every plugin off, and a
server running none is byte-identical to tmux: the two format hooks read one
thread-local flag and return, and no option default is touched.

That is the setting the conformance suite runs the subject under, and
`scripts/c2rs-sut.sh` sets it there for the same reason it already passes
`-f /dev/null`: the comparison is of the engine, and the plugin's status line
is not something the oracle draws, so it would land in every rendered
comparison as a difference that is not a finding. The identity stays reachable
and stays measured; it is just no longer what an unconfigured server does.

### The agent plugin

The agent plugin — on unless `TMUX_C2RS_PLUGINS` says otherwise — adds the six
pane variables of `../PROTOCOL.md` §2 —
`#{pane_agent}`, `#{pane_agent_state}`, `#{pane_agent_pid}`,
`#{pane_agent_session_id}`, `#{pane_agent_model}`, `#{pane_state_emoji}` — by
polling every pane at 200 ms. The detection is not in this crate: detectors,
session-id and model resolution, process probing and the pane classifier live
in `hmux-agent`, which the hmux daemon hosts through the same contract, so
both servers classify a pane with one implementation rather than two that
drift. What is here is the wiring: the `ServerObservability` implementation over
`window_pane`, the tick, and the redraw.

These differences from the oracle are deliberate and expected, and are what
`TMUX_C2RS_PLUGINS=none` takes back:

- The six variables exist. Stock tmux expands an unknown `#{...}` to nothing,
  so five of them read the same either way, but `#{pane_agent_state}` says
  `none` where tmux says nothing at all, and `#{pane_state_emoji}` is never
  empty.
- `window-status-format` and `window-status-current-format` differ, because the
  status line this server draws is built around `#{pane_state_emoji}`. That
  default is the server's rather than this plugin's — see below.
- Each pane's output bumps a revision counter, and each pane is probed through
  `/proc` (or `libproc`) once per sweep. Nothing observable follows from
  either, but the server is doing work tmux is not.

`exit-empty` is *not* changed. The hmux0 daemon defaults it to `after-session`
and creates session 0 on a first untargeted attach; that is a lifetime change
rather than a presentation one, and this server keeps tmux's behaviour.

### The git plugin

The git plugin — also on unless `TMUX_C2RS_PLUGINS` says otherwise — answers
where a pane sits in a git worktree, and what the repository holding it is in
the middle of. It exists because `#{b:pane_current_path}` is the wrong label
in a repository with worktrees: every worktree of this one has an `hmux`
directory, so the window labels collide and the component that tells them
apart is the one the basename drops.

| Variable | Values | Meaning |
|----------|--------|---------|
| `#{git_worktree}` | `h1`, or empty outside a repository | The worktree root's own directory name. A linked worktree is named by itself, not by the repository. |
| `#{git_worktree_path}` | absolute path | The worktree root. |
| `#{git_subdir}` | `hmux/src`, empty at the root | Where the pane sits below the root. |
| `#{git_repo}` | `hmux` | The repository every worktree of it shares, from the directory holding the common git directory. |
| `#{git_branch}` | `h1`, empty on a detached HEAD | The branch `HEAD` names; during a rebase, the branch being rebuilt. |
| `#{git_head}` | `h1` or `38b63b0` | The branch when there is one, the short commit when there is not. Never empty in a repository. |
| `#{git_action}` | empty, `rebase`, `am`, `merge`, `bisect`, `cherry-pick`, `revert` | The operation the repository is in the middle of. |
| `#{git_action_step}` / `#{git_action_total}` | `2` / `7`, or empty | How far a rebase has got, when it counts. |

Every value is read out of files — the upward walk for `.git`, the `HEAD` it
names, and the marker files an interrupted operation leaves behind. Nothing
here runs git or reads the index, so there is no dirty-state tier: `git
status` in a status line is the reason `gitstatusd` exists, and none of the
variables above need it. A sweep costs one `readlink` per pane and two `stat`s
per repository, at 500 ms, and the repositories are shared — a dozen panes in
one worktree are one entry. Values are computed on the tick, so expanding a
status format never touches the filesystem; a pane created between two ticks
reads as empty until the next one.

Three things it deliberately does not do:

- The two rebase backends are one `rebase`. The marker that looks like it
  separates an interactive rebase from a plain one is written for every rebase
  the merge backend runs, so reporting it would be wrong for the common case
  rather than right for the rare one.
- A repository whose refs live in a reftable reports no branch and no commit.
  There is no ref file to read there, and the placeholder git leaves in `HEAD`
  for older readers is not a branch name. Everything else — the worktree, the
  repository, the operation — still answers.
- The pane's working directory comes from the server's own pane tree rather
  than through `plugin::Host`, which carries no working directory. A plugin
  wanting to run on the hmux daemon as well would need one; adding it is a
  change to a versioned public trait, so it waits for a reason.

### The default status line

The window label these variables are for is `window-status-format`, and it is
the server's, not a plugin's: the plugins publish variables, and what the
status line does with them is decided in one place — `server::defaults` —
rather than in whichever plugin happens to name them. Two plugins declaring one
format would also make registration order decide it, since an option default
only replaces a value still holding tmux's.

It is the pane's state glyph, then where the pane is: the worktree name at the
root, the worktree name and a trailing `/` anywhere below it, the directory's
own basename outside a repository, and the operation in brackets when there is
one.

    h1        h1/        proj        h1 [rebase 2/7]

Nothing is replaced when no plugin is running. Every variable the format draws
on comes from one, and a server with none of them is meant to be tmux. With
some of them running, a variable whose own plugin is off expands to nothing,
and every branch of the format is written to survive that. Only options still
holding their built-in default are replaced, and this runs before any
configuration file is read, so `.tmux.conf` still wins.

## Testing

The `hmux` executable reports `tmux 3.7b` and speaks the pinned client's wire
protocol. Unit tests exercise Rust components, and the parent repository's
conformance harness compares hmux against the reference `tmux` found on `PATH`.
The reference must report exactly `tmux 3.7b`.

From the parent repository, `make unit`, `make test`, `make lint`, `make asan`, and `make leak`
default to the active hmux implementation. `make test` runs unit tests and the
main conformance suite. Per-command, hook, queue, notification, and VT corpus
suites have separate Makefile targets. The hmux conformance profile also runs
ignored cases, with a small explicit exclusion backlog in the parent nextest
configuration. Passing `SUT=hmux0` selects the retired daemon's gates.

The main conformance suite also runs the release hmux client through basic
start, attach, shell-exit, and detach lifecycles against the tmux oracle.
`make test-client` runs these cases alone. The gate builds the optimized client
and passes its path as `HMUX_CLIENT_BIN`; server conformance continues to use
the selected server build.

The unit gate runs each test in its own process with nextest, then runs doctests
through `cargo test --doc`. Global collection thread bindings last for the
process lifetime, so tests that use process globals require these isolated
gates. The leak gate also uses separate test processes; neither leak detection
nor conformance establishes aliasing soundness. The ASan gate checks address
safety with leak detection left to the separate leak gate. Resolver linkage
explicitly retains `libresolv` because ASan supplies base64 interceptor symbols
that otherwise let the linker discard the real implementation. Socket receives
use full address storage before copying a bounded prefix to the caller, so
ASan's receive interceptor can check the returned address length.

From this directory, `make check-buffer-memory` runs the segmented-buffer tests
under AddressSanitizer on nightly Rust for `x86_64-unknown-linux-gnu`. The suite
includes deterministic operation sequences compared with a contiguous byte
model, and checks oversized trait copies without changing the inherent method's
clamping behavior. This focused gate does not cover other daemon components or
replace the full conformance and leak gates. Miri is not required by this target.

`make check-collection-memory` runs the global-collection tests under
ThreadSanitizer. It uses the production collection module directly through a
small standalone test manifest and rebuilds the standard library with matching
instrumentation. This target requires nightly Rust, `rust-src`, and
`x86_64-unknown-linux-gnu`; it covers the collection thread boundary.
