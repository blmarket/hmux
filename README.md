# hmux2

A fresh c2rust 0.22.1 translation of tmux
`e880cf63e0a9fe095d7c5d313761520fb1a8653c` (`tmux next-3.9`).
The source is the patched archive from `../hmux/nix/tmux-target.nix`,
including its two retained patches (0010 and 0014). Generation uses that
package's configure flags, including UTF8PROC, systemd and utempter on Linux,
with SIXEL and debug mode disabled.

This is a separate baseline, with no engine or runtime code copied from hmux.
The generated code retains C allocation, pointer and external-library usage.
It has not undergone hmux's ownership migration. The checked-in translation
targets x86_64 Linux and requires the repository's nightly Rust toolchain.

Run from the repository root:

```sh
nix develop -c cargo build --manifest-path hmux2/Cargo.toml
nix develop -c bash hmux2/scripts/regenerate.sh
```

The executable is `hmux2/target/debug/hmux2` and accepts the tmux CLI.
Regeneration extracts the pinned patched source into a temporary directory,
builds it under Bear, and translates every compilation unit. The script checks
the oracle's version and revision before starting, and rejects missing output.
It overwrites generated Rust files and the manifest. `fixups.py` adapts the
variadic API to the current nightly, preserves non-UTF-8 Unix command-line
arguments, and names the package and executable `hmux2`; `build.rs` supplies
the C library link dependencies.

Upstream licensing is preserved in `COPYING` and `COPYRIGHT`.
