# Key-name parsing and formatting

The original implementation in `src/key_string.rs` used a table of aliases,
`strcasecmp`/`sscanf` parsing, and a process-global `static mut out[64]` for
formatting. The Rust entry points keep the byte-oriented C semantics while
moving formatting ownership to the caller.

## Characterized behavior

`key_string_parse(&[u8])` first rejects embedded NUL bytes and otherwise passes
the bytes to the existing parser without UTF-8 conversion. The parser returns
`Some(key_code)` for a valid key and `None` for the historical
`KEYC_UNKNOWN` result. `key_string_parse_cstr(&CStr)` has the same result for an
already NUL-terminated byte string.

- Modifier prefixes are repeated, case-insensitive `C-`, `M-`, and `S-` forms.
  Unknown prefixes and a trailing prefix fail. Formatting always orders present
  modifiers as `C-M-S-`, regardless of input order.
- `^x` is the two-byte control shorthand and uses the active C-locale lowercase
  operation. A bare `^` is an ordinary caret key. Longer strings beginning with
  `^` enter the control/modifier path used by the original parser.
- The generated key table has 1,379 entries and compares names with libc
  `strcasecmp`. Its aliases include
  `IC`/`Insert`, `NPage`/`PageDown`/`PgDn`, function keys, cursor keys, keypad
  keys, mouse names, and the other generated named-key entries. Formatting
  chooses the first table spelling, so `Insert` formats canonically as `IC` and
  `PageDown` as `NPage`. Table entries can carry inherent flags such as cursor
  or implied-meta; the parser preserves those flags except that implied-meta is
  removed when no explicit `M-` was supplied.
- `None` and `Any` are successful sentinel parses. `Unknown` is a formatter
  spelling for `KEYC_UNKNOWN`, but is not accepted by the parser. Other event
  sentinels (`FocusIn`, `PasteStart`, mouse-move values, and so on) have named
  formatter output but are not table aliases unless present in the generated
  table. Command callers still reject `KEYC_NONE` or `KEYC_UNKNOWN` where the
  old command contract did.
- `User0` through `User1000` are accepted by the historical `sscanf` form
  (including its case-sensitive `User` spelling and partial-conversion
  behavior). They format as `UserN`.
- A lowercase `0x` numeric form is parsed with `%x` (including its
  partial-conversion/trailing-text behavior). Values below 32 become raw
  control key codes; other values go through `wctomb` and the existing packed
  UTF-8 representation. Thus numeric `0x41` formats as `A`, but its internal
  key code is not the raw integer 65. Numeric parsing occurs before modifier
  parsing, so `C-0x41` is not the numeric form.
- A single ASCII byte from 32 through 127 is accepted; bytes below 32 are
  invalid. A complete valid UTF-8 sequence is accepted as one packed Unicode
  key. Invalid or incomplete UTF-8, unknown names, malformed modifiers,
  embedded NULs, and out-of-range users return `None`. Terminal escape
  decoding is intentionally outside these interfaces.

`key_string_format(key, with_flags)` returns an owned `CString`. It preserves
the existing canonical order and spellings:

- special values format as `None`, `Unknown`, `Any`, `FocusIn`, `FocusOut`,
  paste/theme names, and mouse names;
- table keys use the first matching table alias;
- Unicode keys become their UTF-8 bytes; printable ASCII is emitted directly;
  DEL is `C-?`, and other non-named values are empty, use the legacy octal
  fallback where applicable, or become `Invalid#<lowercase-hex>` according to
  the original branches. If malformed packed-Unicode data would contain a
  NUL, formatting truncates at that C-string terminator before appending flags,
  so it remains non-panicking and preserves the visible legacy result;
- with `with_flags`, the historical flag letters are appended in `L`, `K`,
  `C`, `I`, `B`, `S` order. Without it, those letters are omitted. The flag
  suffix is display text and is not a parse input.

`key_string_format_into` writes the same bytes plus a trailing NUL to a caller
buffer. It returns the byte count excluding the NUL and leaves a short buffer
unchanged. The owned result has no relationship to later formatting calls, so
multiple `CString`s can safely be retained at once.

## C compatibility

The required `key_string_lookup_string` and `key_string_lookup_key` symbols
remain as narrow adapters. The parser adapter maps `None` back to
`KEYC_UNKNOWN`. The formatter adapter stores a newly owned `CString` in
thread-local storage and returns its pointer for synchronous legacy callers;
that pointer is valid only until the next adapter call on the same thread (or
thread exit), and must not be freed. Production Rust callers use the owned API
directly. Binding, unbinding, listing, menu, prompt, tty, option, and logging
callers now retain owned values for the duration of each C call.

## Verification

The focused Rust tests cover modifier order, aliases, named values, sentinels,
Unicode, numeric forms, invalid bytes, bounded output, retained results across
consecutive calls, empty inputs, malformed packed-Unicode output, C adapters,
and canonical parse/format round trips.
`scripts/key_cli_checks.py` creates a private socket and exercises `bind-key`,
`unbind-key`, and `list-keys`, including canonical `Insert` → `IC` listing.

For this migration (2026-09-20), the repository checks compared as follows:

| Check | Before | After |
| --- | --- | --- |
| `cargo build` | Pass; 1,890 library warnings | Pass; 1,890 library warnings |
| `cargo test` | Pass; 139 listed tests | Pass; 147 listed tests |
| `cargo clippy --all-targets` | Fails with 30 existing errors; 7,419 library warnings | Fails with the same 30 errors; 7,415 library warnings |
| private-socket bind/list/unbind check | Pass | Pass |
| `python3 scripts/check_ffi_exports.py` | — | Pass; all 1,496 required exports |
