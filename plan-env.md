# Can `environ_push` and friends be encapsulated in one module?

Short answer: **yes, and the shape is already half-built.** Every touch of the
*process* environment — the `environ` extern static, `getenv`, `setenv`,
`unsetenv` — can move behind one small gateway that privately owns the libc
declarations, exposes safe reads, and keeps exactly two `unsafe` write
operations whose `# Safety` rustdoc states the real contract. This mirrors the
line `std::env` itself draws (safe `var_os`, unsafe `set_var`), so the design
is precedent-backed rather than invented here.

A prior pass (kept condensed at the bottom) established that the *writes*
cannot become safe: `std::env::set_var` is `unsafe` under edition 2024 and
panics post-`fork` where `setenv` merely returns `-1`. Encapsulation is the
answer to that finding — if the unsafety must stay, give it one home and one
piece of documentation instead of six files of bare libc calls.

## Who touches the process environment today

| File | What | Via |
|---|---|---|
| `src/environ.rs:1` | imports `ffi::{environ, setenv}` | `environ_push` (`:199`) swaps the `environ` static and loops `setenv`; `environ_process` (`:91`) walks the raw array |
| `src/tmux.rs` | `getenv` ×10 | `SHELL` `:526`, `PWD` `:790`, `HOME` `:815`, `TMUX` `:961`/`:1031`, `LC_ALL`/`LC_CTYPE`/`LANG` `:964`–`:969`, `VISUAL`/`EDITOR` `:999`–`:1001` |
| `src/client.rs` | `getenv` `:513` (`TERM`), `setenv` `:779` (`SHELL`, pre-`execl`) | |
| `src/job.rs` | `setenv` `:404` (`SHELL`, post-`fork` pre-`execl`) | |
| `src/compat/getopt_long.rs:231` | `getenv` (`POSIXLY_CORRECT` presence test) | |
| `src/tests/test_environ.rs` | `getenv`/`setenv`/`unsetenv` + raw `environ` walk | save/restore around the push test |

That is the complete list — there is no `execve`/`execle`-with-envp, no
`putenv`, no `clearenv` anywhere in `src/`. Child environments are built
exclusively by `environ_push` followed by `execl`/`execvp`, so a single
gateway genuinely covers everything.

## A soundness wrinkle the audit turned up

`environ_process` (`src/environ.rs:91`) is a **safe** `pub fn` returning
`impl Iterator<Item = &'static CStr>` — borrows of the live `environ` array
with an unbounded lifetime. Any later `setenv` or `environ_push` can free the
strings behind those borrows, so as written the safe signature promises more
than it can keep. No current caller holds the items across a write
(`src/client.rs:750` sends each on the wire immediately, `src/tmux.rs:872`
feeds each to `environ_put` immediately), so nothing is broken *today* — but
this is exactly the kind of implicit invariant the encapsulation should make
explicit. The cheap fix is to yield owned `CString`s; both callers take
`.as_ptr()` on the spot, so the change is mechanical and changes no observable
bytes.

## Proposed shape

A `pub(crate) mod process` submodule inside `src/environ.rs` — the file that
already owns `environ_push` and `environ_process`, so this adds no new home
and no competing implementation, just pulls the raw libc surface into one
place. (A separate `src/proc_env.rs` file works identically if the submodule
makes `environ.rs` feel crowded; nothing below depends on the choice.) The
module holds its **own private** `unsafe extern "C"` block for `environ`,
`getenv`, `setenv`, `unsetenv`, and the four declarations are deleted from
`src/ffi.rs` once the callers have moved — after that, reaching libc's
environment from anywhere else requires writing a new extern declaration,
which is what makes the boundary hold in review.

```rust
pub(crate) mod process {
    unsafe extern "C" {
        static mut environ: *mut *mut c_char;
        fn getenv(name: *const c_char) -> *mut c_char;
        fn setenv(name: *const c_char, value: *const c_char, overwrite: c_int) -> c_int;
        #[cfg(test)]
        fn unsetenv(name: *const c_char) -> c_int;
    }

    /// The value `name` holds in the process environment, copied out, or
    /// none when the variable is not set. A variable set to the empty
    /// string comes back as an empty value, not as none.
    pub(crate) fn get(name: &CStr) -> Option<CString>;

    /// Every `NAME=value` string the process environment holds, copied
    /// out in array order. Entries without an `=` are kept as they are.
    pub(crate) fn entries() -> Vec<CString>;

    /// # Safety
    /// The same contract as `std::env::set_var`: no other thread may be
    /// reading or writing the process environment. Pointers previously
    /// returned by libc `getenv` may be left dangling.
    pub(crate) unsafe fn set(name: &CStr, value: &CStr);

    /// # Safety
    /// As for [`set`]. Points `environ` at a static empty array; the C
    /// library allocates a fresh array on the next `set`, so the old one
    /// is leaked — callers `exec` (or, in tests, rebuild the environment
    /// entry by entry) rather than free it.
    pub(crate) unsafe fn clear();

    #[cfg(test)]
    pub(crate) fn snapshot() -> Vec<(CString, CString)>;
    #[cfg(test)]
    pub(crate) unsafe fn restore(saved: &[(CString, CString)]);
}
```

(Signatures indicative; rustdoc prose to be written in the house style, and
`# Safety` sections are rustdoc, so the no-comments rule is satisfied.)

### Why the reads can be safe while the writes cannot

`std` itself ships safe `var_os` alongside unsafe `set_var`, accepting that C
code calling `setenv` concurrently would be undefined behavior regardless —
the read side's safety rests on writes being disciplined, not on writes being
impossible. This module strengthens that footing rather than weakening it:
after the migration *every* in-process write goes through `set`/`clear`, both
`unsafe`, both documented, and both called only where the process is
single-threaded (server startup, a forked child, or the mutex-guarded test
fixture). `get` copies the value out before returning, so no `&'static`
borrow of libc memory ever escapes the module.

### What each `# Safety` contract documents

`clear` (the `environ`-swap step of `environ_push`) is where the load-bearing
subtleties live, and today they are spread between a code comment and the
reader's knowledge of glibc:

- pointing `environ` at a `static mut` empty array is sound only because
  glibc's `setenv`, on seeing an array it did not allocate last, *mallocs a
  fresh one* instead of reallocating in place — the static is never written
  through;
- the previous array and its strings are deliberately leaked; the two
  production callers (`src/job.rs:357`, `src/spawn.rs:622`) `exec` moments
  later, and the push test rebuilds the environment entry by entry;
- every pointer previously obtained from `getenv` or the old array is
  invalidated — which is the documented reason `get`/`entries` return owned
  copies;
- both production call sites run **between `fork` and `exec`**, where POSIX
  formally permits only async-signal-safe calls and `setenv` (which mallocs)
  is not one. tmux's C code does exactly the same, the server is
  single-threaded so no allocator lock can be held across the fork, and
  matching that behavior is the point of the transpilation — but it belongs
  in writing on the one function that does it.

`environ_push` itself keeps its name and its `unsafe fn` signature (its
callers are inside larger unsafe fork blocks anyway) but its body becomes
`process::clear()` plus a loop over `process::set`, and its own `# Safety`
section points at theirs.

### Side benefits made structural

- **The null-`SHELL` hazard disappears.** `src/client.rs:774`–`779` currently
  maps `Option<&CStr>` to a possibly-null pointer and hands it to `setenv`,
  which glibc dereferences unchecked. `process::set(&CStr, &CStr)` cannot
  express that call — the caller is forced to handle the `None` arm before
  reaching the API, turning an implicit "not reachable today" argument into a
  type-checked one.
- **`environ_process`'s lying lifetime goes away.** It becomes a thin wrapper
  over `entries()` (or its two callers use `entries()` directly), yielding
  owned strings.
- **The test save/restore helpers** (`src/tests/test_environ.rs:18`–`48`)
  move behind `snapshot`/`restore`, and the stale comment at
  `src/tests/test_environ.rs:14` claiming the push "calls `clearenv`" (it has
  not since the `environ`-swap landed) gets corrected in passing.

### Call-site conversion notes

The `getenv` sites split the same way the earlier read analysis found:

- Seven are trivial (`src/tmux.rs:526`, `:790`, `:815`, `:961`, `:1031`,
  `src/client.rs:513`, `src/compat/getopt_long.rs:231`): the value is copied,
  compared, or presence-tested in place; `get` drops straight in. The
  null-or-empty tests (`s.is_null() || *s == '\0'`) map onto
  `matches!(v, None | Some(v) if v.is_empty())` — note `VISUAL`/`EDITOR` at
  `src/tmux.rs:999`–`:1001` tests **null only**, so an empty `VISUAL` must
  stay accepted; `Option<CString>` preserves exactly that distinction.
- Two chains in `main` (`LC_*`/`LANG` at `src/tmux.rs:964`–`:969`,
  `VISUAL`/`EDITOR` at `:999`–`:1010`) keep a bare `*const c_char` across
  several statements and feed `strcasestr`/`strrchr`/`strstr`/
  `options_set_string`; they need an owned local binding with `.as_ptr()` at
  each use — mechanical, but it edits the flow of `main` and wants a careful
  diff.

The `setenv("SHELL", …)` sites (`src/client.rs:779`, `src/job.rs:404`) become
`process::set(c"SHELL", shell)` inside their existing unsafe blocks. Neither
`get` nor `set` may be given new allocation on the write path beyond what
glibc's `setenv` already does — the post-fork sites stay exactly as
allocation-heavy as the C they transpile, no more.

## Plan

1. **Introduce `environ::process`** with the private extern block, `get`,
   `entries`, `set`, `clear`, and the test helpers; rewrite `environ_push`
   and `environ_process` on top of it; move the test file's save/restore onto
   `snapshot`/`restore` and fix the `clearenv` comment. Gate: `make unit`
   (the push test exercises swap + set + restore directly).
2. **Convert the trivial reads** — seven sites, one commit. Gate: `make unit`.
3. **Convert the two `main` read chains**, binding owned locals. Gate:
   `make unit`, `make test-commands CMD=set_environment`, and a manual check
   that `status-keys`/`mode-keys` still follow `EDITOR=vi`.
4. **Convert the two `SHELL` writes**, handling the `None` shell arm in
   `client_exec` explicitly. Gate: `make unit`, plus
   `make test-commands CMD=new_window` or a manual default-shell check.
5. **Delete `environ`, `getenv`, `setenv`, `unsetenv` from `src/ffi.rs`** once
   `grep` shows no importer outside `environ.rs`. Gate: `make lint`, then
   `make test` before calling the work done.

No public trait is touched; the changed items are `pub` functions, which
CLAUDE.md classes as implementation surface. Observable behavior is intended
to be byte-identical throughout — every step is a refactor plus
documentation, not a behavior change — so the conformance suite is a
regression check, not a characterization exercise.

---

## Background: why the writes stay on libc (condensed prior finding)

An earlier pass asked whether these functions could move to safe `std::env`.
The conclusion, kept here because it motivates the design above:

- **`std::env::set_var`/`remove_var` are `unsafe fn`** under this crate's
  edition 2024 (verified on the pinned nightly): migrating the writes gains
  no safety and keeps every `unsafe` block.
- **`set_var` panics where `setenv` returns `-1`** (name containing `=`,
  empty name, NUL in value — all measured). `environ_push` runs after `fork`
  in both callers, past `log_close()` and `closefrom()`; a panic there
  unwinds a child with no log and no descriptors. Today's guard against
  rejected keys lives two modules away (`cmd_set_environment` input checks),
  which is too thin a thread to hang a post-fork panic on.
- **There is no safe way to empty the environment.** `std` has no `clear`;
  the swap of the `environ` static is the only non-quadratic, glibc-clean
  way to start a child from nothing, and it was verified visible to `std`
  readers (no caching in `std::env`).
- **`Command::env_clear().envs(…)`** — the genuinely safe design — does not
  fit: `spawn.rs` needs `forkpty` and both call sites do substantial work
  between fork and exec; converting only `job.rs`'s non-pty branch would
  create the competing-implementation shape CLAUDE.md rules out.
- The **reads** were found migratable to safe `std::env::var_os`; the module
  above supersedes that with `process::get`, which keeps the code on one
  gateway instead of splitting reads to `std` while writes stay libc. Either
  works; one gateway documents better.
