# Current architecture

This is the architecture that exists in the repository on `main`; it is not a
proposal for a clean-room Rust rewrite. `hmux2` is a C2Rust translation of
tmux. The generated translation units still contain raw pointers, C ABIs,
`static mut` state, and C-heap ownership. The boundaries below make those
facts visible and prevent new copies from being mistaken for completed
refactoring.

The crate is a single package with an `rlib`, a `staticlib`, and the `hmux2`
binary. `src/lib.rs` exposes `src/modules.rs` as the public `hmux2::src`
namespace. `src/main.rs` delegates to `hmux2::src::tmux::main`. The historical
namespace and its explicit compatibility re-exports remain part of the
interface; a module move is not permission to remove an old path.

## Module responsibilities

The outer `src` files are still the translated application modules. They are
organized by the original C responsibilities, not by Rust ownership layers.

| Area | Modules and responsibility |
| --- | --- |
| Entry and registry | `lib.rs` supplies crate attributes and the public registry; `modules.rs` wires translation units and compatibility aliases; `main.rs` is the binary shim; `tmux.rs` owns command-line startup, global options, socket/process setup, and the main entry point. |
| Commands | `cmd/core.rs` owns command descriptors and the command table; `cmd/find.rs` resolves targets; `cmd/parse.rs` parses command input; `cmd/queue.rs` owns queue state and callback dispatch; `cmd/entries/` contains individual command implementations. |
| Grid, layout, style, and text | `grid/{core,reader,view}.rs` implement storage and views; `layout/{core,custom,set}.rs` implement layout trees and serialization; `style/{attributes,colour,parsing}.rs` implement style and colour APIs; `text/{utf8,utf8_combined}.rs` implement UTF-8 and combined-character handling. |
| Server model and control | `server.rs`, `server_client.rs`, `session.rs`, `window*.rs`, `client.rs`, `control*.rs`, `environ.rs`, `options*.rs`, `format*.rs`, `hooks.rs`, `job.rs`, `proc.rs`, and `spawn.rs` retain the tmux server/client/session/window object graph and event-driven control paths. |
| Terminal and input | `tty*.rs`, `input*.rs`, `screen*.rs`, `status.rs`, `menu.rs`, `popup.rs`, `prompt*.rs`, `key_*.rs`, `mode_tree.rs`, `monitor.rs`, `paste.rs`, and `resize.rs` implement terminal I/O, input state machines, screen writing, prompts, modes, and user-facing views. |
| Compatibility | `compat/` contains local replacements and portability shims: getopt, forkpty, peer/program-name helpers, imsg, allocation helpers, `strtonum`, `unvis`, `vis`, `systemd`, and `utf8proc`. These are implementations, not foreign declarations. |
| Foreign boundary | `ffi/` is the only location for `extern "C" { ... }` provider declarations. It groups libc, libevent, libm, ncurses, resolver, systemd, utempter, and utf8proc imports. |
| Shared ABI declarations | `shared/` is the authoritative home for declarations used by more than one translated unit. Its 59 subject modules (plus `mod.rs`) contain the `repr(C)` records, unions, aliases, constants, bitfields, and callback types that must have one Rust identity. |
| Allocation and logging | `xmalloc.rs` is the fatal C-heap wrapper surface; `log.rs` owns logging and fatal paths. `environ.rs` additionally provides the narrow Rust ownership wrapper described below. |

The shared subjects are grouped as follows. The grouping is descriptive; the
Rust module paths remain the individual files listed in `src/shared/mod.rs`.

| Subject group | Shared modules |
| --- | --- |
| ABI and platform declarations | `abi`, `account`, `ctype`, `errno`, `limits`, `posix_io`, `posix_terminal`, `regex`, `signal`, `socket`, `stdio`, `time`, `utf8`, `variadic`, `vis` |
| Core object graph | `arguments`, `client`, `command`, `control`, `environment`, `event`, `events`, `format`, `job`, `key`, `layout`, `menu`, `message`, `mode_tree`, `monitor`, `options`, `pane`, `paste`, `popup`, `process`, `prompt`, `redraw`, `screen`, `screen_write`, `server_acl`, `session`, `spawn`, `status`, `terminal`, `tree`, `tty`, `window` |
| Rendering, input, and presentation | `alerts`, `borders`, `colour`, `control_character`, `display`, `grid`, `hyperlinks`, `input`, `json`, `mouse`, `sort`, `style` |

## Inventory and enforced boundaries

### Shared declarations

The current public-declaration inventory reports 4,709 `pub struct`, `pub
union`, `pub type`, and `pub const` records across `src`; 3,525 records are
rooted in `src/shared`. Those numbers include constants and aliases and are
inventory counts, not claims that every generated anonymous C type is
interchangeable. `scripts/declaration_inventory.py` reports the current
locations and declaration fingerprints.

`tests/shared_definitions.rs` and the architecture guard require each
authoritative shared name to have one definition in `src/shared` and reject a
copy in another source module. Compatibility modules use `pub use` to preserve
the old path. The generated `C2RustUnnamed*` names remain translation-unit
local. The five intentionally duplicated named private domains are `NONE`,
`LEFT`, `RIGHT`, `TOP`, and `BOTTOM`; their exception is source-checked rather
than silently generalized.

### Internal and external foreign declarations

Rust implementations are imported directly by translated modules. A foreign
declaration is reserved for a symbol supplied by a native provider and must be
inside `src/ffi`. The current provider inventory is:

| Provider | Functions | Mutable globals | Opaque types |
| --- | ---: | ---: | ---: |
| libc | 168 | 4 | 14 |
| libevent | 33 | 0 | 3 |
| libm | 3 | 0 | 0 |
| ncurses | 6 | 1 | 0 |
| resolv | 2 | 0 | 0 |
| systemd | 20 | 0 | 3 |
| utempter | 2 | 0 | 0 |
| utf8proc | 6 | 0 | 0 |
| **total** | **240** | **5** | **20** |

`tests/ffi_boundary.rs` rejects internal implementations redeclared as foreign,
duplicate foreign symbols, foreign blocks outside `src/ffi`, and lost baseline
C exports. `tests/architecture.rs` freezes the provider counts as well. The
1,496 required `no_mangle`/`export_name` symbols remain listed in
[`required-exports.tsv`](required-exports.tsv); use
`scripts/check_ffi_exports.py target/debug/libhmux2.a` when an archive is
available.

An `extern "C" fn` definition in an application module is an implementation or
callback entry point, not a foreign import. It must retain its C calling
convention and any existing export attribute.

### Mutable scratch and global state

There are currently 466 `static mut` declarations in the source inventory:

| Kind | Count | Meaning |
| --- | ---: | --- |
| Function-local mutable statics | 81 | Translated C local statics used as scratch buffers, parser state, caches, or reusable result storage. |
| Module-level translated mutable statics | 380 | Command tables, mode tables, lookup tables, event state, and process/object-graph globals. |
| Imported FFI mutable globals | 5 | `environ`, stdio globals, `program_invocation_short_name`, and ncurses `cur_term`. |

The exact function-local inventory is frozen in
[`architecture-mutable-scratch.tsv`](architecture-mutable-scratch.tsv), with a
count for repeated names such as the sorting buffers. The architecture test
requires the inventory to match; adding, removing, or moving one requires an
ownership and concurrency review. These statics are not thread-safe Rust
state. The current translated code relies on the original C process/event-loop
usage and does not provide a general `Sync` or reentrancy guarantee.

The style compatibility shims are a separate, intentional case:
`attributes_tostring` and `colour_toescape` use thread-local `RefCell<CString>`
storage. Their returned pointers are borrowed C strings valid until the next
call on the same thread (or thread exit), and callers must not free or use them
concurrently. The owned Rust formatting APIs should be used when a result must
outlive that call.

### Allocation and ownership

The default ownership model is still the C model:

| API or storage | Rule |
| --- | --- |
| `xmalloc`, `xcalloc`, `xrealloc`, `xreallocarray`, `xrecallocarray` | Use libc-compatible allocation. Zero sizes and checked overflow are fatal in the `x*` wrappers; allocation failure is fatal. The returned pointer is a C-heap owner. |
| `xstrdup`, `xstrndup`, `xmemdup`, `xasprintf`/`xvasprintf` | Return C-heap memory. Release it with the matching C `free` path, not a Rust allocator. Variadic formatting retains the C `VaList` ABI. |
| `recallocarray` and `freezero` | Compatibility implementations use libc allocation and deallocation; `freezero` clears before `free`. Their buffers follow the same C-heap rule. |
| `malloc`/`calloc`/`realloc`/`reallocarray`/`strdup`/`strndup`/`vasprintf` | These are foreign libc calls. A pointer returned by one is released by the corresponding C provider (`free` where libc specifies it). Libevent, systemd, glob, FILE, and other providers keep their own release functions. |
| C records and opaque views | They remain `Copy`/raw-pointer representations where the translated ABI requires it. They must not acquire a Rust `Drop` owner inside a `calloc`/`free`-managed record. |
| `EnvironOwner` | The explicit exception is a non-`Copy` `NonNull<environ>` owner. `Drop` calls `environ_free`; `transfer`/`from_raw` make the one-way ownership hand-off explicit, and borrowed views carry a lifetime tied to the owner. |
| Rust-facing style/text APIs | Owned `CString`, `Vec`, and byte-slice APIs allocate and return Rust-owned values. They are not passed into C records as if C owned them. |

Translated source must not reconstruct a Rust `Box`, `Vec`, or `CString` from a
C allocation. The architecture guard rejects the corresponding raw
deallocator conversions. This is a conservative boundary, not a claim that
all raw-pointer ownership has been made safe; most application APIs remain
unsafe C-style APIs.

### Callback ABI

The callback contract is `Option<unsafe extern "C" fn(...) -> ...>` whenever a
C callback may be null. Non-null callback storage, such as fixed dispatch
arrays, keeps a non-optional `unsafe extern "C" fn` where the C declaration is
non-null. Raw pointer arguments, integer aliases, return values, and variadic
`VaList` handling are part of the ABI and must not be changed to Rust closures
or the Rust ABI.

There are 43 named callback aliases in the shared/provider declarations. They
cover command queues, jobs, event payloads and libevent, client overlays,
prompts and modes, terminal drawing, monitor/spawn/status callbacks, signal
handlers, libc comparators, and systemd. Inline callback fields in `event`,
`process`, `input`, `tty`, `window`, `signal`, and `posix_io` follow the same
rule. `tests/architecture.rs` parses every function-pointer type in `src` and
rejects a missing C ABI or missing `unsafe`; `tests/consolidation_callbacks.rs`,
`tests/model_callbacks.rs`, and `tests/ffi_variadic.rs` compile and invoke
representative assignments across old compatibility paths.

## Validation commands

Run these from the repository root. They do not regenerate source files.

```sh
cargo build
cargo test
cargo clippy --all-targets
cargo test --test architecture --test ffi_boundary --test shared_definitions
python3 scripts/declaration_inventory.py
python3 scripts/check_ffi_exports.py target/debug/libhmux2.a
python3 scripts/cli_regressions.py
python3 scripts/ffi_callbacks.py
git diff --check
```

`cargo clippy --all-targets` is currently an informative debt check: it is
expected to exit non-zero until the pre-existing translated-code errors are
addressed. The architecture and boundary tests must pass independently. The
runtime scripts use repository-local temporary sockets and a private server;
`HMUX_BINARY` can select another already-built binary for comparison.

## Regeneration limitations

`build.rs` only emits native link directives. It is not a source generator.
The checked-in Rust is the source used by the crate, while the ABI/layout
fixtures record observations made on the current Linux/glibc and linked-library
environment. Re-running an external C2Rust/header conversion is not a
reproducible validation step here: it can renumber anonymous types, recreate
translation-unit-local declarations, move foreign blocks, alter callback
spellings, and change generated ownership or export details.

Do not run destructive regeneration, delete generated modules, or bless new
fixtures as a substitute for reviewing those changes. If regeneration becomes
necessary, compare shared declaration ownership, foreign-provider inventory,
layout/bitfields, callback signatures, C exports, allocation/free pairs, and
runtime transcripts before accepting it. No such regeneration was performed
for this architecture baseline.

## Remaining work and explicit debt baseline

The following are still open and are intentionally documented as debt:

- The translated application remains largely raw-pointer/`unsafe` C-style
  code. The 380 module-level mutable statics and 81 function-local statics have
  not been converted to ownership-bearing Rust state or synchronized storage.
- Anonymous generated records and aliases remain local unless their C identity
  was audited. The five named private enum exceptions remain; matching layout
  alone is not permission to merge another `C2RustUnnamed*` type.
- The current build emits a large generated-code warning set, and Clippy still
  reports 30 unique translated-code errors (25 equal-expression comparisons,
  four self-assignments, and one loop with unchanging condition variables).
  This task does not suppress or rewrite those findings.
- Most C allocation lifetimes, event ownership, opaque provider objects, and
  callback lifetimes are still expressed by raw pointers and conventions. The
  `EnvironOwner` and style/text APIs are narrow wrappers, not a whole-crate
  ownership conversion.
- ABI measurements and native links are platform-specific. The frozen tests
  establish the current translated baseline on this target; they are not an
  independent certification for another libc, terminal library, compiler, or
  architecture.
- There is no safe, repository-local regeneration pipeline or fixture-blessing
  mode. Future migration work must be incremental, preserve compatibility
  exports, and update the inventories only after a source/ABI review.

Historical migration reports in `docs/shared-declarations.md`,
`docs/direct-imports.md`, and `docs/module-organization.md` retain their
before/after measurements. They are useful provenance, but this file and the
source guards are the current architecture contract; neither implies that the
remaining debt above has already been refactored away.

## Before/after validation

The baseline was measured before adding the architecture guards, on
`rustc 1.99.0-nightly (12c36e253 2026-08-10)`:

| Command | Before guards | After guards |
| --- | --- | --- |
| `cargo build` | Pass; 1,890 library warnings | Pass; 1,890 library warnings |
| `cargo test` | Pass; 124 test cases | Pass; 130 test cases |
| `cargo clippy --all-targets` | Exit 101; 60 error diagnostics (30 repeated errors) and 14,863 warning diagnostics | Exit 101; same 60 error diagnostics and 14,863 warning diagnostics |
| `git diff --check` | Clean baseline | Pass |

The after column is from the final verification run. The six additional test
cases are the architecture guard tests; the existing build and Clippy debt is
unchanged.
