# hmux

c2rust translation of tmux

## Scrolling windows

Every window is one horizontal strip of full-height panes. Each pane is half
or all of the window's width, and the strip extends past the right edge
instead of squeezing its panes. A new pane is half-width and goes right after
the active pane; closing a pane closes the gap. A lone half-width pane leaves
the rest of the window blank.

Each client's view rests on a pane's first column and shows the active pane
completely, so selecting a pane scrolls the strip; moving the cursor inside a
visible pane does not. Left and right selection stop at the ends of the strip.

The `C-b` prefix keeps tmux's bindings. `C-a` enters the `strip` key table
for the strip's own keys; release it before the next key.

| Key | Action | Command |
| --- | --- | --- |
| `c` | Insert a pane after the active pane, in its current directory | `new-pane -c '#{pane_current_path}'` |
| `h` / `l` | Select the pane to the left / right | `select-pane -L` / `-R` |
| `f` | Toggle the active pane between half and full width | `resize-pane -Z` |
| `H` / `L` | Move the active pane left / right | `swap-pane -U` / `-D` |
| `x` | Kill the active pane | `kill-pane` |
| `C-a` | Send a literal `C-a` to the application | `send-keys C-a` |

`bind-key -T strip` changes these keys, and `unbind-key -n C-a` gives `C-a`
back to applications. In emacs copy mode `C-a` stays `start-of-line`.

`new-pane -b` inserts before the target instead. `-c`, `-e` and `-d` work as
in tmux; without `-c` the new pane does not follow the target pane's `cd`.
`join-pane` and `break-pane` move a pane between strips, keeping its width.
`refresh-client -L` / `-R` pan a client's view, and `refresh-client -c` or
selecting another pane returns it to the active pane.

`split-window` is an alias for `new-pane`: it accepts and ignores `-f`, `-h`,
`-l`, `-p` and `-v`, so scripts that split still get a pane in the strip.

Names with no meaning in a strip are gone; commands, flags and options are
rejected and formats expand to nothing:

- Commands: `select-layout`, `next-layout`, `previous-layout`, `move-pane`,
  `display-menu`, `display-popup`.
- Flags: `-Z` everywhere but `resize-pane`; `resize-pane -D/-L/-M/-R/-U/-x/-y`
  (`-T` and `-Z` remain); `select-pane -U/-D`; `break-pane -W`; `-f/-h/-l/-p/-v` on
  `new-pane` and `join-pane`.
- Targets: `{up-of}` and `{down-of}`.
- Options: `main-pane-height`, `main-pane-width`, `other-pane-height`,
  `other-pane-width`, `tiled-layout-max-columns`, `pane-border-status`,
  `pane-border-format` and the `menu-*` and `popup-*` options.
- Formats and hooks: `window_zoomed_flag`, `pane_zoomed_flag`,
  `window_visible_layout`, `after-select-layout`, `window-zoomed`,
  `window-unzoomed`.

`#{window_layout}` describes the strip as a single row of panes.

`agentmon` creates and monitors coding-agent runs in git worktrees through hmux:

```sh
nix run .#agentmon
```

Run it inside an hmux pane to discover the current server, or pass
`-- --socket /path/to/socket` from another terminal. `nix develop` also puts
`agentmon` and its companion `looper` on `PATH`.

See [agentmon-tui/README.md](agentmon-tui/README.md) for usage and development.
