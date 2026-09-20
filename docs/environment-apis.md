# Environment ownership and borrowed views

The ABI model remains the C-allocated red-black tree in
`src/shared/environment.rs`. `environ`, `environ_entry`, and their link fields
are still `#[repr(C)]`, `Copy` records because they are embedded in and passed
through C-managed structures. The tree was not replaced by a Rust map.

## Rust owner

`src/environ.rs::EnvironOwner` is the non-`Copy` owner for one tree allocated by
`environ_create`. Its `Drop` calls the matching `environ_free`. `from_raw` and
`into_raw`/`transfer` make ownership hand-offs explicit; a raw pointer is never
adopted by more than one owner. The owner is kept in Rust locals and is not put
inside a `calloc`/`free`-managed record such as `spawn_context` or `job`.

`EnvironView<'a>`, `EnvironEntry<'a>`, and `EnvironIter<'a>` carry a lifetime
from an owner borrow. Entry names and values are borrowed as `CStr`/byte
slices, so non-UTF-8 data is preserved. A missing name returns `None`; a
present valueless name returns an entry whose value is `None`. Flags and the
existing libc `strcmp` bytewise ordering are exposed without decoding or
reimplementing the red-black tree.

Byte-slice mutation and lookup helpers reject embedded NUL bytes rather than
silently truncating. C-string helpers remain available for translated call
sites. `copy_from` accepts a borrowed view, so an entry pointer cannot outlive
the owner borrow in safe Rust.

## Production lifetime

`spawn_pane` owns the temporary environment returned by
`environ_for_session` until every parent success or failure return. The child
uses its raw borrowed pointer through `environ_push`, drops its private
post-fork owner copy, and then reaches `exec`/`_exit`; the parent drops its
copy after fork handling. The temporary
editor environment in `spawn_editor` is likewise owned outside the C spawn
context. `job_run` uses the same pattern across socket/fork failure, parent
transfer, and child exec paths. Existing persistent session, client, and
global trees retain their C ownership and ABI layout.

## Verification

`tests/environment.rs` covers missing versus valueless entries, flags,
bytewise ordering, arbitrary bytes, borrowed entry pointers, copying and
owner transfer/drop. `scripts/environment_cli_checks.py` starts a server on a
unique temporary socket, checks environment propagation through a new window,
checks `-r` valueless state, respawns the pane, and always sends `kill-server`
in `finally`; `TemporaryDirectory` removes the socket directory afterward.

Run the focused checks with:

```sh
cargo test --test environment
python3 scripts/environment_cli_checks.py
```

The final build/test/lint comparison is recorded below after running the same
commands against this checkout:

| Check | Before | After |
| --- | --- | --- |
| `cargo build` | Pass; 1,890 library warnings | Pass; 1,890 library warnings |
| `cargo test` | Pass; 116 tests | Pass; 119 tests |
| `cargo clippy --all-targets` | Fails; 30 existing errors, 7,430 library warnings | Same 30 errors, 7,430 library warnings |

The Clippy failures are the translated tree/bitfield diagnostics already
present before this migration; the owner adds no new error records or library
warning count. The three-test increase is the focused owner/borrow/transfer
coverage in `tests/environment.rs`.
