# Attribute and colour text compatibility

Characterized from the original implementations and `tests/style_text.rs` before
migration. All names use libc case comparisons in the active locale, not Unicode
case folding. Inputs are bytes, with no UTF-8 decoding or replacement. C strings
stop at NUL; the new byte-slice APIs reject embedded NUL instead of truncating.

Attributes accept `acs`, `bright`/`bold`, `dim`, `underscore`, `blink`, `reverse`,
`hidden`, `italics`, `strikethrough`, `double-underscore`, `curly-underscore`,
`dotted-underscore`, `dashed-underscore`, and `overline`. Comma, ASCII space and
pipe delimit tokens; runs inside a list are accepted, but leading/trailing
separators, empty input, tabs, unknown names and mixed `none`/`default` lists fail
with -1. Standalone `none` and `default` mean zero. Duplicate bits are ORed.
Formatting emits that order (with `bright`, never `bold`), then `noattr`, separated
by commas. Zero formats as `none`; unknown bits are ignored (unknown-only is an
empty string). `noattr` is deliberately not parseable.

Colour parsing recognizes basic names and exact decimal aliases 0–7 and 90–97;
8/9 are not numeric aliases for `default`/`terminal`. `color` and `colour` prefixes
accept decimal 0–255 using strtonum: leading locale whitespace, sign and leading
zeros are allowed; trailing whitespace, missing digits, overflow and out-of-range
values fail. `#` needs exactly six locale-classified hex digits. Theme names use
`grey`, not `gray`. Fallback X11 names include spaced names and gray/grey aliases;
gray/grey suffixes use strtonum 0–100 and C round(2.55 * n), including floating
point rounding quirks. The basic names take precedence over the X11 RGB table.
Invalid input returns -1; `none` is not a successful parse.

Colour formatting checks -1 (`none`), theme, RGB, then indexed flags, in that
order. Theme/indexed values use only the low byte; RGB uses the low 24 bits.
Unknown themes or unflagged values are `invalid`. Basic/default/terminal and
bright names are canonical, indexed values use `colourN`, RGB uses lowercase hex.
This is intentionally not a bijection for arbitrary integers.

The X11 parser additionally accepts rgb: and # components with 2 or 4 hex digits,
decimal comma triples (components narrow to bytes), and cmy:/cmyk: floating point
components in [0,1]. It retains scanf whitespace, partial-conversion and trailing
text behavior, and active-locale decimal points. Fallback names trim ASCII space
only. Non-UTF-8 bytes are passed to libc unchanged, so acceptance is determined by
the locale and grammar, not by UTF-8 validity.

## Rust entry points and ownership

- `attributes_parse(&[u8])` / `attributes_parse_cstr(&CStr)` return `Option<i32>`.
- `colour_parse` / `colour_parse_cstr` parse application colour syntax.
- `colour_parse_name` / `colour_parse_name_cstr` parse X11 names directly (so
  `red` means RGB red here, versus basic colour 1 in application syntax).
- `colour_parse_x11` / `colour_parse_x11_cstr` parse the extended X11 grammar.
- `attributes_format(i32)` and `colour_format(i32)` return owned `CString`s.
- `colour_format_escape(i32, bool, i32)` returns `Option<CString>` for SGR text;
  arguments are colour, background selection and terminal capability flags.

`None` represents invalid input; the C ABI translates it to -1. Safe results
contain no borrowed scratch storage. To retain multiple values, keep their owners:

```rust
let foreground = colour_format(COLOUR_FLAG_RGB | 0x112233);
let background = colour_format(COLOUR_FLAG_256 | 123);
// Both values remain valid regardless of any later formatting calls.
```

Production C calls consume `.as_ptr()` from these owned values within the same
statement, which keeps each temporary alive through the entire consuming call.
Any pointer retained beyond that statement must instead have a named `CString`
owner kept alive for its entire use. C consumers copy synchronously with xstrdup,
format synchronously with xsnprintf, or log synchronously.

The retained `*_fromstring`, `colour_byname`, and `colour_parseX11` ABI symbols
require readable NUL-terminated pointers for the duration of the call. The
`*_tostring` and `colour_toescape` ABI symbols own separate thread-local buffers;
their pointers must not be freed and are valid only until the next call to the
same shim on that thread or thread exit. There are no production callers of
these scratch-backed ABI formatters. Safe APIs never access their buffers.

The libc-dependent colour parser kernels are private. They retain scanf,
strtonum, ctype and case comparison semantics; byte slices first become checked
CStrings. Read-only lookup tables are constants. X11 application debug logging
is isolated in `colour_parse_x11_logged`, an unsafe application-thread adapter
around the safe parser. The C X11 shim uses that adapter too. Safe parsing itself
has no application logging side effects. The log now prints the original input,
including leading spaces, rather than the advanced fallback-name pointer.

Client-specific escape generation uses an unsafe adapter to read raw client
state, then calls the owned formatter. The safe escape API uses terminal theme
fallbacks. Escape formatting preserves existing theme precedence (including -1
mapping through the invalid-theme fallback to the default colour), low-byte
index masking, default/terminal reset sequences and null/None for invalid
unflagged colours. The original capability-mask expressions are deliberately
unchanged: with the current flag values, even zero terminal flags do not strip
RGB/indexed output.

## Verification (2026-09-20)

| Check | Before | After |
| --- | --- | --- |
| `cargo build` | Pass; 1890 library warnings | Pass; 1890 library warnings |
| `cargo test` | 107 tests pass | 115 tests pass, plus 5 isolated locale runs |
| `cargo clippy --all-targets` | Fails: 30 errors; 7445 library warnings | Same 30 errors; 7432 library warnings |
| `python3 scripts/cli_regressions.py` | Pass | Pass; identical JSON transcript |
| `python3 scripts/style_cli_checks.py` | Added during migration | Pass |

Clippy's existing errors are 25 equal-expression comparisons, four `px`
self-assignments, and one loop with an unchanging condition. They are outside
this API migration. Raw logs and before/after CLI transcripts are in the ignored
`target/api-checks/` directory.

The original ABI characterization test passed before edits and after the
attribute migration, before colour changes. Extended escape cases passed before
changing that formatter. The tests freeze all 578 X11 name entries, attribute
aliases and ordering, numeric limits, embedded NUL behavior, non-UTF-8 inputs,
sentinel/flag precedence and owned result retention across calls and threads.
Locale tests run in separate processes for available C, C.utf8, Turkish UTF-8
and ISO-8859-9, and German UTF-8 locales; they do not race global setlocale in the
test process. The standalone style CLI script creates and removes its own socket
and server under target and can also run where library test linking is unavailable
(`HMUX_BINARY` selects an existing executable).
