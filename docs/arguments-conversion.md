# Argument conversion and storage

`src/arguments.rs` keeps the historical C ABI and C-heap ownership model, but
the numeric value paths now have a Rust-result layer between storage and the
exported wrappers. The storage layer selects values; the conversion layer
only interprets a borrowed `CStr` or a formatted temporary.

## Storage and string paths

| Path | Input and result | Ownership |
| --- | --- | --- |
| `args_set` and `args_first_value`/`args_next_value` | Store every repeated flag value in insertion order under the flag’s tree entry. | The `args` object owns each `args_value`; `args_free` releases strings, command lists, and cached text. |
| `args_last_value` and `args_last_string` | Select the last repeated value for numeric lookup. Non-string or absent values report `missing` to numeric helpers. | Borrowed raw pointers; no allocation or transfer. |
| `args_value_as_string` and `args_string` | Positional strings are returned directly. Command values are rendered with `cmd_list_print` and cached on the value. | The caller borrows the returned pointer; the argument value owns the cached C string. |
| `args_get` and `args_to_vector` | Preserve the existing C-compatible flag lookup and positional/vector behavior. | Existing C union and C-heap conventions are unchanged. |

Command-valued arguments therefore remain command lists rather than being
coerced to strings during storage. Repeated flags remain ordered, while the
numeric paths retain the existing last-value behavior.

## Numeric conversion paths

The pure helpers are `parse_number` and `parse_percentage`. They accept
`&CStr` and return `Result<i64, ArgumentValueError>`. Number syntax is still
delegated to the local `strtonum` implementation, preserving libc decimal
syntax, leading whitespace/sign handling, complete-input validation, and
inclusive bounds.

| Typed helper | Historical C export | Production users |
| --- | --- | --- |
| `args_strtonum_result` | `args_strtonum` | display-message delay, display-panes delay, resize-window width/height, display-menu starting choice |
| `args_strtonum_and_expand_result` | `args_strtonum_and_expand` | capture-pane offsets, send-keys repeat count, tiled split percentage input |
| `args_percentage_result` | `args_percentage` | resize-pane dimensions, display-menu dimensions |
| `args_percentage_and_expand_result` | `args_percentage_and_expand` | join-pane offsets and floating-layout positions |
| `args_string_percentage_result` | `args_string_percentage` | the four main/other pane height/width layout options |
| `args_string_percentage_and_expand_result` | `args_string_percentage_and_expand` | retained for the C ABI and expansion callers |

Percent suffixes remain bounded to `0..=1000`; the computed share is then
checked against the caller’s inclusive `minval..=maxval`. A plain empty
percentage value reports `empty`. The expanded helper intentionally preserves
the historical path where an empty value is formatted and then reports
`invalid`.

`ArgumentValueError` maps one-to-one to the existing diagnostic strings:
`missing`, `empty`, `invalid`, `too small`, and `too large`. The six exported
functions translate `Result<i64, ArgumentValueError>` back to the old
`c_longlong`/`cause` contract: success returns the number and a null cause;
failure returns zero and an allocated diagnostic string. No C export or
required symbol was removed.

Layout option callers retain their old defaults: invalid main height/width
uses `24`/`80`, invalid or zero other dimensions use the complementary size,
and the command callers keep their existing command-specific error prefixes.

## Validation

`tests/arguments_conversion.rs` covers integer endpoints, malformed input,
percentage endpoints and bounds, repeated flag order, command-valued storage,
cached string ownership, and C-boundary diagnostics. The focused CLI check in
`scripts/arguments_cli_checks.py` exercises the same boundary/error cases
through private-socket real commands; its `finally` block always asks that
private server to exit.
