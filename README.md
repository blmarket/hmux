# hmux

This product tree was renamed from `tmux-c2rs/` to `hmux/`; the Rust package
and its source-level `tmux-c2rs` provenance names remain unchanged for now.

A whole-program [c2rust](https://github.com/immunant/c2rust) transpilation of
the pinned tmux 3.7b source. Every C translation unit is a Rust module in one
crate, and the crate builds an `hmux` binary that behaves like the C one.

This is the bulk-transpile counterpart to `../tmux-rs`, which migrates tmux one
externally visible C function at a time. The two are independent routes from the
same pinned reference and share nothing.

## What the source is

`src/` was produced by `c2rust transpile` and is now the source of record: the
transpiling pipeline has been retired, so the crate is edited directly and no
longer sits under a `generated/` output directory. It
still reads the way c2rust emits — unsafe, unidiomatic Rust mirroring the C
control flow, with raw pointers, `goto`-shaped `current_block` loops, and C
globals throughout. It is 154 modules and roughly 340k lines from 150
translation units, builds on edition 2024, and compiles with about 1450
warnings, mostly duplicate `extern` function declarations that each module
repeats.

The type declarations c2rust duplicated the same way were collapsed into
`src/types.rs`, so a name like `grid_cell` is one Rust type
crate-wide rather than one per module. What stayed behind is what could not
be proven identical: the anonymous `C2RustUnnamed_NN` structs, whose numbering
is per translation unit and so means different things in different modules,
and every named type that reaches one — which is most of tmux's own core,
`client`, `session`, `window` and `grid` among them.

The declarations c2rust duplicated for *functions* are being resolved a
different way. Where the crate already defines the function, the module's
`extern "C"` declaration of it is replaced by a `use` of the definition: the
call reaches the same `#[no_mangle]` symbol the linker was resolving it to
anyway, but rustc now checks it against the real signature instead of against
a per-module restatement. 820 of the 4,288 declaration sites convert on those
terms today. The rest name a type that is still per-module — `client` and
friends above — so the declared and defined signatures are different Rust
types even where they are the same C prototype, and they stay as they are
until the anonymous types are renamed.

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

## Layout

- `src/` — one module per translation unit, reached through `src/lib.rs`.
  `src/tmux.rs` is tmux.c and owns the entry point; `src/main.rs` is a thin
  binary that calls into it, because a second copy would define every one of
  tmux.c's `#[no_mangle]` globals twice. `src/types.rs` is not a translation
  unit: it holds the type declarations shared by all of them, and every module
  glob-imports it. `src/fmt_engine.rs` is not one either: it is the crate's own
  printf(3) engine, which every one of the format-taking functions
  (`log_debug`, `xasprintf`, `cmdq_print`, `format_add`, …) expands its C
  format string with. Those functions take their arguments as a `&[FmtArg]`
  slice rather than as C varargs, which is why no Rust function definition in
  the tree carries a C calling convention any more.

## Run

```sh
nix develop
make            # cargo build
make check-tmux # refuse a reference tmux that is not 3.7b
```

`make clean` runs `cargo clean`.

## Event loop

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

The build is a whole `hmux` binary that reports `tmux 3.7b` and speaks the
pinned client's wire protocol, so it is tested from the outside rather than
function by function: put `target/debug/hmux` where a suite expects
its tmux and compare the run against the pinned binary. `make test-c2rs` at
the repository root puts this binary on `PATH` as the oracle and runs the
conformance suite against it; the hmux-conformance harness picks its reference
tmux off `PATH`, version-checked against 3.7b.

### Valgrind and AddressSanitizer

`nix develop` carries both. Valgrind runs the unit test binary as it is, which
`cargo test --lib --no-run` builds and names:

    valgrind --error-exitcode=99 <the binary it named> --test-threads=1

AddressSanitizer wants the nightly flag, an explicit target so the build script
is left uninstrumented, and a target directory of its own so an instrumented
build does not displace the plain one:

    RUSTFLAGS=-Zsanitizer=address cargo test --lib \
      --target x86_64-unknown-linux-gnu --target-dir target/asan

LeakSanitizer comes with it and reports on the way out; what both tools count
as still allocated at exit is mostly the globals a server keeps for its whole
run. Neither sees a read that runs past what a `Vec` holds while staying inside
what it allocated -- only one that runs past the allocation itself.

## Reference source

The transpiled source is stock tmux 3.7b, the same the rest of the tree pins. It
is the unpatched upstream tarball: the patches in `hmux/nix/tmux.nix` -- and
`nix/tmux.nix` here, kept identical to it -- are applied when building the
conformance oracle, not here, so a conformance comparison against the oracle
inherits that difference.

Eleven of those patches go the other way: they are leaks 3.7b has, which this
crate now fixes and the oracle carries the same fix for, all but the last
submitted upstream. `window_pane_destroy` freeing neither
`border_status_line.expanded` nor `r.ranges`, `server_client_lost` freeing
neither `c->path` nor the exit strings, tree mode's preview leaking a format
tree per drawn window or pane, `set-buffer` leaking the `-b` name,
`format_draw` dropping its collected items on an unterminated style,
`screen_write_free_list` dropping the ones a line still holds, `if-shell -F` --
with `args_make_commands_now` behind it -- never freeing the command list it
queues, and `environ_push` orphaning the fresh `environ` array the first
`setenv` replaces. A leak is not observable through the command line, the wire
protocol or server behavior, so none of these was ever a conformance difference
-- only a difference from the transpiled source, and now not that either.
