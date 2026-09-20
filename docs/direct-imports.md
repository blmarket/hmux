# Direct Rust imports and the foreign boundary

Baseline: `a075620b099bbf570d3647052044b047f157d91e` on `main`.

All 4,244 foreign declarations referring to Rust implementations now import
those implementations directly. Functions retain their existing `extern "C"`
ABI, including variadic functions and callbacks. Globals and command tables
retain their original definitions, mutability, initializers, and exports.
No executable bodies or signature-conversion casts were changed.

## Subsystem migration

[internal-ffi-before.tsv](internal-ffi-before.tsv) records every original
consumer, symbol, Rust provider, and migration subsystem. Each provider group
was migrated and successfully built before proceeding to the next group.

| Provider subsystem | Internal foreign declarations replaced |
| --- | ---: |
| Support: compatibility implementations, allocation, logging, UTF-8, colours, attributes | 614 |
| Commands: command tables, parser, queue, arguments, configuration | 951 |
| Terminal: grid, screen, input, tty, layout, window modes | 699 |
| Model: session, window, options, environment, formatting and remaining subjects | 1,516 |
| Server: server, client, process, jobs, control, spawning, file I/O, entry point | 464 |

Shared types were sufficient for every internal function and global import.
Some Rust modules expose a type and a function/global with the same name
(`args_parse`, `args_value`, `clients`, `sessions`, `windows`, `tty_terms`,
`options_table_entry`, and `window_pane_resize`). Consumers that previously
re-exported the type now re-export both namespaces from the implementation
module. This keeps the original public type paths without duplicate imports.

## Foreign providers

`src/ffi` owns all remaining foreign blocks. The original 1,093 genuine foreign
declarations reduce to 265 declarations: 240 functions, five globals, and
20 opaque types. All copies of each declaration matched after removing
parameter names, visibility and whitespace; no incompatible variant was
silently chosen. Calling conventions, variadics, pointer constness and nullable
callback signatures remain unchanged.

| Provider module | Functions | Globals | Opaque types |
| --- | ---: | ---: | ---: |
| `libc` | 168 | 4 | 14 |
| `libevent` | 33 | 0 | 3 |
| `libm` | 3 | 0 | 0 |
| `ncurses` | 6 | 1 | 0 |
| `resolv` | 2 | 0 | 0 |
| `systemd` | 20 | 0 | 3 |
| `utempter` | 2 | 0 | 0 |
| `utf8proc` | 6 | 0 | 0 |

The libc module includes `forkpty`, which this repository's glibc target
resolves from libc; it does not introduce a new libutil link dependency.
Native link configuration in `build.rs` is unchanged.

Foreign signatures reuse the existing shared type modules. Twenty local
records/aliases needed by those signatures, and the twenty opaque declarations,
move to their provider modules with re-exports at their original public paths.
[ffi-types.tsv](ffi-types.tsv) lists these moves. Every moved declaration is
identical to its original after whitespace normalization, including record
attributes and field order. Unrelated implementation types remain in place.

`tests/ffi_boundary.rs` parses all Rust source under `src`, including binary,
compatibility and shared code, including `src/lib.rs`. It rejects internal functions or
globals redeclared as foreign, duplicate foreign symbols, and foreign blocks
outside `src/ffi`. It understands `link_name`, `export_name`, unsafe attribute
wrappers, private definitions, nested modules, callbacks and variadics. Lexer
fixtures ensure comments and strings do not create false declarations. Parsing
failures and unparsed items fail the test. The test also requires all baseline
C export attributes to remain present.

## Exports and validation

[required-exports.tsv](required-exports.tsv) freezes all 1,496 original
`no_mangle` exports, including their providers and kinds. No exports were
removed based on assumptions about external consumers. Run
`python3 scripts/check_ffi_exports.py [path/to/libhmux2.a]` to check the actual
archive with `nm`; both baseline and final archives contain every required
symbol. Rust's compiler-generated mangled symbols are not treated as stable
C exports.

| Check | Before | After |
| --- | --- | --- |
| `cargo build` | Pass; 1,891 library warnings | Pass; 1,890 library warnings |
| `cargo test -j 2` | 104 pass | 107 pass |
| `cargo clippy --all-targets --message-format=json` | 30 existing errors per library build | Same errors, messages, files and occurrence counts |
| Required C exports in static archive | 1,496 present | All 1,496 retained |
| Original CLI regression transcript | Pass | Pass; byte-identical |
| Expanded callback regression transcript | Pass | Pass; byte-identical |

Clippy's 30 existing errors are 25 equal-expression comparisons, four
self-assignments and one unchanging loop condition. Across its library and
library-test targets there are 60 error records in each run. The full diagnostic
multiset, ignoring shifted line numbers, differs only by removal of the
`fatal` redeclaration warning in `compat/getdtablecount.rs` (two records).
That declaration previously omitted the Rust definition's diverging return
type. Warning records fall from 14,895 to 14,893. Newly unused imports were
removed; no lint allowances were added.

The three new tests are two boundary/inventory tests and a direct variadic-call
test. The latter passes pointer, integer and floating-point arguments through
a typed C function pointer to `xasprintf`, which forwards a `VaList` through
`xvasprintf` to libc. Existing type-layout and callback tests continue to pass.

Run both runtime checks with:

```sh
python3 scripts/cli_regressions.py
python3 scripts/ffi_callbacks.py
```

Both accept `HMUX_BINARY` to select the binary. The new script records the command-table entry count (92) and checks
representative command names, hook dispatch,
successful and unsuccessful job completions, command-queue resumption, timer
callbacks, background jobs and wait channels, client file writes, control-client
reads/writes/notifications/detach, and server socket recreation after SIGUSR1.
Each run uses its own repository-local temporary socket, disables user
configuration, closes the control client, stops its server in `finally`, and
removes the socket. It never addresses the user's default server.

Full logs, before/after transcripts, diagnostic comparisons and subsystem build
logs are retained under ignored `target/ffi-migration`. The baseline was also
rebuilt from a repository-local archive there to run the expanded callback
checks against the original binary. A final source comparison removed imports,
foreign blocks and type/constant declarations, then normalized whitespace; all
existing source files matched, establishing that executable bodies and exported
definitions were unchanged. All work remains on `main`, with no commits,
pushes, history rewrites or modifications to other repositories.
