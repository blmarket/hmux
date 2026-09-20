# Shared declaration consolidation

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
