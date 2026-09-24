# hmux-cmdparse

The LALRPOP grammar and owned syntax tree extracted from `hmux/src/cmd/parse.rs`
and `parse_grammar.lalrpop`. The grammar productions are preserved; application
services are supplied through `Context`.

`parse(context, tokens)` accepts a lazy stream of located tokens and returns owned
commands, including nested command arguments. The caller supplies tokenization,
format expansion, truth evaluation, assignment validation and environment updates.
The lexer must emit a newline after the final statement. Keep tokenization lazy:
assignments performed during parsing can affect expansion in subsequent tokens.
Token strings use `CString` to preserve non-UTF-8 command bytes.

The hmux2 adapter lives in `src/cmd/parse.rs`. It retains the existing lexer,
alias expansion and command construction, while replacing the translated Bison
parser and its raw-pointer semantic stack. The adapter still uses the application's
global lexer state and therefore does not make hmux2 parsing reentrant.

Run `cargo test -p hmux-cmdparse` for standalone grammar tests.
