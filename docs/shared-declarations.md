# Shared declaration consolidation

Current status: the core-model continuation below completes the inventoried
shared named families. Earlier tables and remaining-work notes are historical.

Baseline: `39092336a9291bf0baa49b631fcf57cbe4cfc586`, on `main`.
Measurements use Linux x86_64 and rustc `1.99.0-nightly (12c36e253 2026-08-10)`.
No history was rewritten, and no commits or pushes were made.

## Inventory and scope

[declarations-before.tsv](declarations-before.tsv) records the initial inventory
of public structs, unions, aliases, opaque types, and constants under `src`,
including the compatibility modules and binary code. Run
`python3 scripts/declaration_inventory.py` for a current inventory with source
locations and whitespace-normalized declaration fingerprints. Fingerprints
identify candidates; they do not establish C type identity. In particular,
`C2RustUnnamed` numbering is local to a translation unit.

| Inventory measure | Before | After |
| --- | ---: | ---: |
| Source declarations | 17,730 | 14,995 |
| Duplicated named declaration groups, excluding `C2RustUnnamed*` | 670 | 359 |
| Excess copies in those named groups | 10,059 | 7,448 |

Each family was migrated and built before proceeding to the next family.
Shared modules own definitions; translation units re-export the declarations
that they previously exposed. This also preserves access through their original
Rust module paths. The binary entry point calls `src/tmux.rs`, whose flags,
stdio declarations, and ABI imports were migrated along with the library.

| Subject modules | Migrated declarations |
| --- | --- |
| `abi`, `stdio`, `variadic` | Primitive ABI aliases, comparison callback, `FILE` and its opaque pointer dependencies, signed stdio bitfield, variadic argument representation |
| `mouse`, `layout`, `display`, `sort`, `grid` | Mouse event and constants, geometry, visible ranges, sort criteria, grid reader |
| `pane` | Pane offsets, resize queue and links, pane flags |
| `hyperlinks`, `screen_write`, `screen` | Hyperlink storage, screen-write storage, screen selection and titles, complete screen structure, screen modes |
| `menu`, `prompt` | Menu data, prompt result and callbacks, prompt/menu flags |
| `client`, `command`, `options`, `colour` | Client, command, option and colour constants and scalar domains |
| `event`, `monitor`, `utf8` | Event flags and EOL domain, monitor domain, Hangul composition state |
| `tty`, `tree`, `window`, `spawn` | Terminal, tree, window/winlink and spawn flags |

## Identity and ABI evidence

Nested anonymous types were resolved through their owning fields:

- `window_pane_resize.entry` → `window_pane_resize_entry` (all 120 copies).
- `screen_title_entry.entry` → `screen_title_link`.
- `screen_write_cline.items` → `screen_write_items`.
- `screen_write_citem.entry` → `screen_write_item_link`.
- `screen_write_citem.type_0` → `screen_write_item_type`.
- The three `hyperlinks_uri` links → separate `hyperlink_list_entry`,
  `hyperlink_inner_entry`, and `hyperlink_uri_entry` types. The two tree links
  remain distinct even though their layouts agree.

Their complete field declarations and referenced types were checked before
renaming. No pointer or callback casts were added. Apart from these audited
renames and import changes, a comparison with the original source found no
changes outside declarations.

`repr(C)`, field order, pointer constness, array lengths, callback argument and
return types, nullable `Option<unsafe extern "C" fn(...)>` representations, and
bitfield attributes were preserved. The two spellings of `va_list` were resolved
through the verified alias chain `__gnuc_va_list = __builtin_va_list`.
Four integer constants had decimal versus hexadecimal/cast spellings; their
identical declared `c_int` types and values were verified before normalization.

The tests compare frozen fixtures, without a test-time update/blessing mode:

- `consolidation_layout`: sizes, alignments and every named field offset for
  every original concrete copy in the migrated structural families; 70 distinct
  layout records, including aliases and callback pointers.
- `consolidation_bitfields`: all ten original stdio copies; raw bytes, signed
  accessor results, truncation at the 24-bit boundary, and an adjacent-byte
  sentinel for seven representative values.
- `consolidation_constants`: 1,150 original copies of 307 constants. Generic
  assignments require matching Rust types; values and layouts match a fixture
  measured by compiling this test against the original commit.
- `consolidation_callbacks`: direct assignments and calls through the migrated
  nullable C callback types, with no signature-conversion casts.
- `shared_definitions`: automatically discovers authoritative names, then checks
  all source files, including binary and compatibility code, for other copies.
  It recognizes private/restricted visibility, wrapped declarations, opaque
  declarations, comments and literals, and distinguishes `*const` from a
  constant definition. Source files are read once, rather than once per name.

These fixtures establish equivalence to this translated baseline on this
platform; they are not an independent certification against another libc ABI.

## Private exceptions and remaining work

The command parser's `cmd_parse_argument_type`, the command queue's `cmdq_type`,
the tty drawing state machine, and mode-tree search/preview enums remain with
their implementation-only constants. They do not constitute duplicated shared
families.

The remaining client/session/window/window-pane/key-binding/command object
graphs still contain mutually dependent, translation-unit-specific pointer and
callback types. Their associated queue/tree links and opaque views remain local
until that entire dependency closure can be migrated coherently. Identical
names or sizes alone would not justify combining them. Platform-specific
networking, signal and regex families also remain for later subject audits.
Thus this is a substantial partial consolidation, not a claim that all 359
remaining name groups describe interchangeable types.

## Validation comparison

| Check | Before | After |
| --- | --- | --- |
| `cargo build` | Pass; 2,892 library warnings | Pass; 2,831 library warnings |
| `cargo test` | 21 tests pass | 26 tests pass |
| `cargo clippy --all-targets` | Fail; 30 errors, 8,447 library warnings | Same 30 errors in the same files; 8,386 library warnings |
| Isolated CLI regressions | Pass | Pass; identical JSON transcript |

The existing Clippy errors are 25 equal-expression comparisons, four
self-assignments of `px`, and one loop with an unchanging condition. Error
messages and file identities were compared while ignoring shifted line numbers.
No lint allowances were added to suppress them.

Run CLI checks with `python3 scripts/cli_regressions.py`; `HMUX_BINARY` can select
another binary for comparison. The script uses a unique socket beneath this
repository's `target/consolidation`, disables user configuration, and kills its
server in a `finally` block. It checks version, session creation, pane splitting
and resizing, layout selection, buffers, styles, key bindings, formats, invalid
commands, and rendered plain/coloured/hyperlinked text and pane titles. The
original commit was rebuilt in an isolated directory under `target/consolidation`
for the expanded before/after run. Full build/test/Clippy logs, the detailed
inventories and both CLI transcripts are retained there as ignored artifacts.

## Platform-family continuation

The continuation started at `38a8fedec45d5fd53cc4691ffe73f31f35095c21` on
`main`. The tables above describe the retained initial consolidation. This
continuation adds eight authoritative subjects, each built before moving to
the next family:

| Module | Declarations |
| --- | --- |
| `time` | `timespec`, `tm`, `CLOCK_REALTIME`, including the binary's `tmux` module |
| `signal` | Signal sets, values, info payloads, action callbacks and signal constants |
| `socket` | IPv4/IPv6/Unix addresses, address argument unions and opaque address views, socket domains and constants |
| `regex` | Regex buffer, opaque DFA, match offsets, syntax aliases and flags |
| `posix_terminal` | Window size and terminal settings constants, including compatibility code |
| `account` | Password and group records |
| `limits` | Integer limit constants and their underlying compiler limit constants |
| `errno` | Shared error number constants |

[platform-declarations.tsv](platform-declarations.tsv) records every migrated
copy, its subject, original name, authoritative name and normalized declaration
fingerprint. Anonymous names were resolved by traversing owning fields, then
comparing complete declarations and dependencies. For example,
`in6_addr.__in6_u` becomes `in6_addr___in6_u`, and the nested signal payloads
retain separate identities for each owning union field. No anonymous types were
combined merely because their generated names or layouts matched. The signal
handler union retains its nullable C callbacks, and the regex buffer retains
all bitfield attributes and padding. Numeric signal spellings were checked for
identical declared type and value before normalization. No casts were added to
conceal mismatches.

The eight `platform_*` integration tests compare frozen pre-migration
measurements for every original concrete copy: sizes, alignments, all named
field offsets (including union alternatives), and typed constant values.
The fixture labels intentionally preserve original anonymous names so they can
be traced back to the inventory. Regex tests additionally compare raw storage,
all seven accessor results, truncation, and adjacent padding sentinels for seven
inputs through each of the four original module paths. A signal callback test
assigns and invokes both handler variants and the restorer using the client and
process module paths, without conversion casts. The existing automatic
shared-definition guard discovers all eight new modules and passes.

| Check | Continuation before | Continuation after |
| --- | --- | --- |
| Named duplicated groups, excluding `C2RustUnnamed*` | 359 | 257 |
| Excess copies in those groups | 7,448 | 7,145 |
| Total source declarations | 14,995 | 14,679 |
| `cargo build` | Pass | Pass |
| `cargo test` | 26 pass | 35 pass |
| `cargo clippy --all-targets` | 30 errors | Same 30 errors, same messages and files |
| Isolated CLI regression transcript | Pass | Pass, byte-identical |

Artifacts are under `target/consolidation/next-*`, with per-family build logs,
pre-migration layout runs, and before/after JSON inventories alongside them.
A declaration-stripping comparison against the continuation starting commit
found no function-body differences after accounting for audited role renames
and imports. Newly unused bitfield imports were removed from the four original
regex translation units.

This continuation removes 102 duplicated named groups. The mutually dependent
client/session/window/pane graph and other unlisted families remain unfinished;
they are not declared private exceptions. The implementation-only enums listed
above remain justified private declarations. The duplicate guard establishes
uniqueness of migrated authoritative names, not completion of the remaining
257 groups.

## Core-model and remaining-family completion

This continuation starts at `c7e544ddfe3080a306e2fc40c5d0a30bc2233148` on
`main`. It supersedes the earlier remaining-work notes above: all inventoried
shared named families are now consolidated. The five remaining duplicated
names are the declaration-specific private exceptions described below.

[model-declarations.tsv](model-declarations.tsv) records 10,133 migrated
original declarations, including opaque views, concrete implementation records,
anonymous owning roles, callbacks, scalar domains, constants, binary code, and
compatibility code. Each row includes the original declaration fingerprint and
its authoritative subject. Fingerprints remain audit aids, not type-identity
criteria.

The migration sequence was:

1. Independent constant families, with baseline typed value/layout measurements
   and a successful build after each family: alerts, borders, format flags,
   input requests, jobs, popups, character escaping, POSIX I/O, environment flags,
   key modes, message headers, stdio, UTF-8, server ACLs, grid whitespace, null
   pointers, libc character classification, and C0 control characters.
2. Environment records, including their opaque views and tree links.
3. The mutually dependent client/session/window/pane graph, distributed among
   subject modules. Its closure includes arguments, command queues, control
   state, options, formatting, input, jobs, keys, layouts, menus, monitoring,
   paste buffers, processes, prompts, redraw state, screen writing, spawning,
   status lines, and terminal/tty state. These cannot be migrated separately
   while their pointer and callback identities still refer to local copies.
4. Smaller remaining families, each followed by a build: parser records,
   argument-command state, client lists, event payloads, input-request data,
   JSON, command messages, mode trees, option-name maps, pane indexes, popup
   callbacks, filesystem records, prompt data, session groups, spawn contexts,
   message history, and tty-term lists.

Anonymous structures were reached through their owning fields and checked using
complete declarations and dependencies. For example, `client.entry`,
`client_file.entry`, `session.entry`, and `window_pane.entry` become separate
subject-owned link types. Distinct links on a single object remain distinct.
The anonymous enum behind `_IS*` was identified by its complete enumerator set,
as was the C0 control-character enum; their local generated names were not used
as identities.

Opaque views now resolve to the corresponding concrete implementation record
when one exists. The audit includes both the implementation and its consumers;
full field layouts are measured for original concrete records, and pointer
sizes/alignments for original opaque views. `dirent`, which has no concrete
implementation here, remains an authoritative opaque type. Callback signatures,
constness, nullable function pointers, `repr(C)`, unions, and field order are
preserved. No conversion casts were added to reconcile types. A comparison of
all non-shared source files against the continuation starting commit found no
function-body differences after removing declarations/imports and applying the
audited anonymous-role renames. Existing bitfield declarations and their tests
remain intact; the newly migrated model records do not contain bitfields.

The `subjects_*`, `model_*`, and `remaining_*` tests use frozen fixtures measured
before their migrations. They check every original concrete copy's size,
alignment, and all named field offsets, plus opaque pointers, callback aliases,
and typed constant values. They have no fixture-update mode. `model_callbacks`
adds direct C callback assignments and calls across former translation-unit
boundaries, including argument parsing, tty callbacks, client overlays, and
window modes.

### Declaration-specific private exceptions

These constants name different implementation enum domains. Their original
`c_uint` aliases and values remain local, along with the other constants in each
domain. In particular, equal values alone do not establish shared identity.

| Name | Popup resize edge | Other private declaration | Reason |
| --- | ---: | --- | --- |
| `NONE` | 0 | `cmd_parse`: 1 | Parser quote state, not a popup edge |
| `LEFT` | 1 | `format_draw`: 0 | Text alignment domain |
| `RIGHT` | 2 | `format_draw`: 2 | Text alignment domain; equal value is incidental |
| `TOP` | 3 | `window_copy`: 1 | Copy-mode vertical position domain |
| `BOTTOM` | 4 | `window_copy`: 2 | Copy-mode vertical position domain |

Other uniquely named implementation-only records/enums remain with their code.
Some previously private records, such as command-queue state and mode-tree
state, now live with their subject because they are part of the consolidated
concrete dependency closure. Their implementation constants need not move.

The duplicate-definition guard now checks **all** named declarations, including
private/restricted definitions, binary code, and compatibility modules. It
permits the five names above only at their exact original file pairs. New
unreviewed duplicated names fail even if they have not yet been added to a
shared module. Generated `C2RustUnnamed*` identifiers are excluded from that
name-based check because they lack cross-unit identity; migrated owning-role
names are checked by the authoritative-definition guard. Lexer regression
coverage includes `&raw const` expressions as well as `*const`, literals,
comments, and wrapped declarations.

| Check | Continuation before | Continuation after |
| --- | --- | --- |
| Source declarations | 14,679 | 4,970 |
| Duplicated named groups, excluding generated anonymous names | 257 | 5 justified private exceptions |
| Excess copies in those groups | 7,145 | 5 private exceptions |
| `cargo build` | Pass; 2,821 library warnings | Pass; 1,891 library warnings |
| `cargo test -j 2` | 35 tests pass | 98 tests pass |
| `cargo clippy --all-targets` | 30 errors; 8,376 library warnings | Same 30 messages/files; 7,446 library warnings |
| Isolated CLI regressions | Pass | Pass; byte-identical JSON transcript |

Clippy's existing failures remain 25 equal-expression comparisons, four
self-assignments, and one unchanging loop condition. No lint allowances were
added. Source locations were compared by file and message, excluding line
numbers shifted by declaration removal. CLI tests use the existing unique
repository-local socket and guaranteed server cleanup.

Retained validation artifacts are under `target/consolidation/round3-*`,
`model-*`, and `remaining-*`: before/after inventories, build/test/Clippy logs,
per-family builds, pre-migration fixture runs, callback tests, and CLI
transcripts. All work remains uncommitted on `main`; no push or history rewrite
was performed.

## Grid-storage anonymous-role cleanup

This follow-up starts at `2797f28` on `main` and closes the review finding that
123 unused grid-storage union/struct pairs escaped the generated-name exclusion.
The named-family completion above did not account for these 246 declarations.

[grid-storage-declarations.tsv](grid-storage-declarations.tsv) inventories every
copy before this cleanup, with its source file, original name, owning-role name,
and normalized declaration fingerprint. For **each** pair, the audit traversed
`grid_cell_entry.c2rust_unnamed` and then the union's `data` field in the original
transpilation (`5606919`). Each surviving declaration matched that historical
definition and the authoritative declaration in `src/shared/grid.rs`, after
substituting only the two role names. This includes `repr(C)`, derives, field
order, `u_int`/`u_char` dependencies, and both union alternatives. Every union
name occurred only at its declaration; every nested data name occurred only at
its declaration and in its union's field. Generated names were never treated as
cross-module identities.

All 123 pairs now re-export `grid_cell_entry_storage` and
`grid_cell_entry_data` under their original public names, preserving public
module paths without redundant definitions. The authoritative packed enclosing
record is unchanged. Unrelated generated types, including `grid`'s mask/code
record, remain private to their implementation roles. No casts, callback
changes, bitfield changes, or executable-body changes were introduced. An exact
source comparison verified that each of the 123 production-file diffs consists
only of the two declaration-to-re-export replacements, plus removal of the
newly unused ABI import in `cmd_lock_server.rs`. The source inventory
falls from 4,971 to 4,725 declarations.

`grid_storage_layout` records size, alignment, and every field offset for all
246 original copies. Its frozen fixture was captured by compiling/running the
test **before** replacing the definitions. All storage unions measure size 4,
alignment 4, offsets `[0, 0]`; all data records measure size 4, alignment 1,
offsets `[0, 1, 2, 3]` on the validation platform. The final test also requires
direct type assignments between every original public path and its authoritative
role, without conversion casts. Existing packed-container, bitfield, and
callback regression tests remain in the full suite.

The duplicate guard now additionally checks the two audited grid-storage field
sets across all source files, including binary code. It flags candidates
regardless of generated identifier or field-type spelling, without declaring
unrelated anonymous names equivalent. Both candidates must occur exactly once,
with their role names in `src/shared/grid.rs`. Lexer regression cases cover
restricted visibility, wrapped declarations, comments/literals, changed types,
and an unrelated same-generated-name record. The new guard was run against the
pre-cleanup source and failed on the redundant pair in `alerts.rs` as expected.

Validation artifacts are retained under `target/consolidation/grid-cleanup-*`:
fresh before/after inventories, build/test/Clippy logs, frozen pre-cleanup layout
measurements, the expected guard failure, and isolated CLI transcripts. The CLI
runner creates a unique repository-local socket, disables user configuration,
and cleans up its server. No other repository was modified, and no commits,
pushes, branch changes, or history rewrites were performed.

| Check | Before cleanup | After cleanup |
| --- | --- | --- |
| `cargo build` | Pass | Pass |
| `cargo test -j 2` | 98 pass | 101 pass |
| `cargo clippy --all-targets --message-format=json` | 30 existing errors | Same 30 errors; identical diagnostic multiset by level/message/file |
| Isolated CLI regressions | Pass | Pass; byte-identical JSON transcript |

Clippy emits the 30 existing errors twice across the library and its test build
(60 error records); both runs also contain 14,895 warning records. The complete
diagnostic multiset, including occurrence counts and warnings, is unchanged
when excluding shifted line numbers. No lint allowances were added.

## Anonymous key-enum alias cleanup

This follow-up starts at `bba3908` on `main` and closes the remaining review
finding: 23 unused aliases of the same historical anonymous key enum.
[key-enum-declarations.tsv](key-enum-declarations.tsv) records the **pre-migration**
file, public alias, line, original scalar type, enumerator count, and fingerprint.
For every candidate, the audit read its definition at `5606919`, required the
same `::core::ffi::c_ulong` declaration, and extracted every constant typed by
that alias. All 23 lists contain exactly the same 2,053 `KEYC_*` names and values.
The fingerprint is SHA-256 of the sorted `(name, value-expression)` pairs encoded
as compact JSON (`separators=(',', ':')`). Each surviving alias appeared only at
its declaration. This establishes provenance independently of generated names
or equal layouts. The inventory scanned all Rust sources, including binary
code; no additional copy of this family was found in the binary implementation.

`src/shared/key.rs::key_code_enum` is now the authoritative alias, retaining
`::core::ffi::c_ulong`. It is deliberately separate from `key_code`, whose
original type is `::core::ffi::c_ulonglong`. All 23 original public paths now
re-export `key_code_enum`. Exact source comparison against the starting revision
confirmed that every affected translation unit changed only that declaration
into a re-export. Constants, executable bodies, layouts, bitfields, and callback
signatures are unchanged; no casts or lint allowances were added. The source
inventory falls from 4,725 to 4,703 declarations.

`tests/key_enum_layout.rs` was compiled and run against the original aliases
before migration. Its frozen fixture records size 8 and alignment 8 for every
alias on the validation platform. Scalar aliases have no field offsets. The
final test compares all 23 measurements and checks direct assignments in both
directions between every compatibility path, `c_ulong`, and the authoritative
alias. Existing record-offset, bitfield, and callback tests also pass.

The duplicate guard now checks the exact audited file/name pairs even when
an alias's underlying type changes. It additionally flags anonymous aliases
whose target mentions `c_ulong` or `key_code_enum` across all source files,
including binary modules. Such new candidates require provenance review; this
is not an assertion that all unsigned-long enums represent the same C domain.
Lexer tests cover wrapped/restricted declarations, comments, strings, re-exports,
and unrelated generated records and `c_uint` aliases. The guard was run before
migration and failed on the original copy in `cmd_bind_key.rs`, as intended.
The five previously documented distinct private enum domains remain exceptions.

Fresh validation artifacts are retained under `target/consolidation/key-enum-*`:
before/after inventories and build/test/Clippy logs, the original layout capture,
the expected pre-migration guard failure, and isolated CLI transcripts. Clippy
comparison uses the complete diagnostic multiset by level/message/file,
including occurrence counts, while ignoring shifted line numbers.

| Check | Before cleanup | After cleanup |
| --- | --- | --- |
| `cargo build` | Pass | Pass |
| `cargo test -j 2` | 101 pass | 104 pass |
| `cargo clippy --all-targets --message-format=json` | 30 existing errors per library build | Identical diagnostics, including 60 error and 14,895 warning records across targets |
| Isolated CLI regressions | Pass | Pass; byte-identical JSON transcript |

The existing Clippy failures remain 25 equal-expression comparisons, four
self-assignments, and one unchanging loop condition per library build. CLI
coverage includes key binding/listing, isolated session and pane operations,
buffers, styles, rendered terminal output, and an invalid command. Each run uses
a unique repository-local socket, disables user configuration, and cleans up
its server. All changes remain on `main`; no commit, push, history rewrite, or
modification of another repository was performed.
