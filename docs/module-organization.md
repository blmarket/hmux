# Module responsibilities

The library entrypoint is `src/lib.rs` (selected in `Cargo.toml`). It retains
`hmux2::src` through the explicit registry in `src/modules.rs`. The binary
startup in `src/main.rs` continues to call `hmux2::src::tmux::main`.

| Family | Responsibilities |
| --- | --- |
| `cmd` | The facade preserves command construction and command-list APIs from private `core.rs`. |
| `cmd::find` | Resolve command targets and maintain target lookup state. |
| `cmd::parse` | Parse command text and construct command lists. |
| `cmd::queue` | Queue execution, callbacks, and command execution state. |
| `cmd::entries` | Individual command descriptors and execution callbacks, in named modules such as `attach_session`. |
| `style` | The facade exposes parsing, formatting, and style application from private `parsing.rs`. |
| `style::attributes` | Convert terminal attribute names and bit masks. |
| `style::colour` | Parse, format, and convert colours and palettes. |
| `grid` | The facade exposes grid storage and cell operations from private `core.rs`. |
| `grid::view` | Access and modify cells through the visible grid viewport. |
| `grid::reader` | Traverse grid content and track reader position. |
| `layout` | The facade exposes layout-tree construction, sizing, and pane geometry from private `core.rs`. |
| `layout::set` | Select and apply preset pane arrangements. |
| `layout::custom` | Parse and serialize custom layout descriptions. |
| `text::utf8` | UTF-8 encoding, decoding, validation, display width, and character utilities. |
| `text::utf8_combined` | Combined-character and joiner handling. |

Every path in the table is relative to `hmux2::src`. Family facades use explicit
item exports, not glob exports. The registry retains each historical flat
module path with one explicit module alias, for example:

```rust
pub use self::cmd::find as cmd_find;
pub use self::cmd::entries::attach_session as cmd_attach_session;
pub use self::style::attributes;
pub use self::grid::view as grid_view;
pub use self::layout::set as layout_set;
pub use self::text::utf8;
```

These aliases refer to the same functions, types, and statics; they introduce no
wrappers or duplicate definitions. Existing consumers and implementation imports
can keep their old paths. New code can use the family paths. The implementation
files were moved intact: no algorithms, allocation/free rules, ownership, C
exports, shared type definitions, or startup behavior were changed.

Source-scanning tests recurse through `src`, including the new entrypoint, and
retain `build.rs` where previously covered. The private `NONE` exception and
key-enum audit paths follow the command-family relocation. Historical layout
fixture labels remain unchanged because they also verify the legacy public API.

## Validation

Each completed stage is the before-state for the next family. Checks use the
installed `rustc 1.99.0-nightly (12c36e253 2026-08-10)` and the same commands:
`cargo build`, `cargo test`, and `cargo clippy --all-targets`.

| Stage | Build | Tests | Clippy |
| --- | --- | --- | --- |
| Before moves | Pass | 107 passed | Same 30 errors |
| After cmd / before style | Pass | 107 passed | Same 30 errors |
| After style / before grid | Pass | 107 passed | Same 30 errors |
| After grid / before layout | Pass | 107 passed | Same 30 errors |
| After layout / before text | Pass | 107 passed | Same 30 errors |
| After text / before entrypoint | Pass | 107 passed | Same 30 errors |
| After entrypoint | Pass | 107 passed | Same 30 errors |

Build warnings remain at 1,890 at every stage. Clippy exits 101 at every
stage with the same 25 equal-expression comparisons, four self-assignments of
`px`, and one loop whose condition variables are not mutated. Its warning
counts also remain unchanged: 7,445 for the library and 7,448 for library tests.
These pre-existing findings were left unchanged to preserve algorithms.

The initial command test run found one stale physical-path expectation for the
private `NONE` exception. Updating that path restored all 107 passing tests
before moving the style family. The key-enum source audit inventory was also
updated to keep its checks attached to the relocated implementations.

All 76 moved implementation files were compared against `HEAD` and are
byte-for-byte identical. Only new facades, the module registry/entrypoint, and
the two touched source-scanning tests were checked with `rustfmt` using
`skip_children=true`; moved implementations were not reformatted.
`git diff --check` passes. Full per-stage command logs are available locally
under `target/module-migration/` (ignored build output).
