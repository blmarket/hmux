# Option-name and array-index parsing

The byte parser in `src/options_parse.rs` describes only the option-name
grammar and table matching. The option tree, inheritance and storage remain in
`src/options.rs`; C-facing callers still own C-heap results and use the
historical allocation and free paths.

## Characterized behavior

- An input is a nonempty byte string with at most one trailing `[array-key]`
  suffix. The first `[` starts the suffix, the first following `]` must be the
  final byte, and an empty index is rejected. The base name may be empty; it
  then participates in normal matching and is generally ambiguous.
- An index made entirely of ASCII decimal bytes is canonicalized to `u32`.
  `0`, leading zeroes, and `u32::MAX` are accepted; a decimal value above
  `u32::MAX` is rejected as overflow. Any other nonempty byte sequence is a
  textual key and is retained byte-for-byte, including non-UTF-8 bytes and
  digits followed by text.
- Canonical names accept unique prefixes, while an exact canonical name wins
  over a prefix candidate. Exact spelling aliases are applied before prefix
  matching. Multiple prefix candidates are ambiguous; no candidate is
  invalid.
- A name whose base begins with `@` is a user option and bypasses the built-in
  table. Its optional array key is still parsed and returned.
- `set-option` and `show-options` retain the existing diagnostics: ambiguous
  input reports `ambiguous option: <input>`, and all parse, overflow, and
  lookup failures report `invalid option: <input>` (unless quiet mode
  suppresses the command error).

The table-driven unit tests in `options_parse.rs` cover accepted and rejected
array keys and names, abbreviations, aliases, ambiguity, user options, empty
indices and overflow. `tests/options_parsing.rs` compares the new command
adapter with the ABI adapter over the same cases. The command-entry tests in
`src/cmd/entries/options_tests.rs` parse and execute both production commands,
drain the existing configuration cause storage, and assert the exact bytes
sent through `cmdq_error` for ambiguous, empty-index, overflow and unknown
inputs.

## Validation comparison

Measured on the existing clean `main` at `3e473d6` before the test changes and
on the final worktree after them. Full logs are retained under the ignored
`target/option-parsing-comparison/` directory.

| Check | Before | After | Result |
| --- | --- | --- | --- |
| `cargo build` | Pass; 1,890 library warnings | Pass; 1,890 library warnings | No build regression |
| `cargo test` | 136 passed, 0 failed | 138 passed, 0 failed | Two command-level tests added |
| `cargo clippy --all-targets` | Exit 101; 30 errors | Exit 101; same 30 errors | Existing failure, no parser diagnostics |
| Clippy warning summaries | 7,429 library; 7,432 library-test | 7,429 library; 7,432 library-test | Unchanged |

The 30 Clippy errors are the pre-existing translated-code findings: 25 equal
expression comparisons, four `px` self-assignments, and one loop with an
unchanging condition. They are unchanged in kind and count; no new error or
warning is reported for the byte parser, command tests, or adapter test.
