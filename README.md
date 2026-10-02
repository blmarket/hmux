# hmux

c2rust translation of tmux

New windows use the **scrolling** layout: full-height panes in a horizontal strip.
Each pane keeps a half-width or full-width preference as the terminal resizes.
Focus scrolls just enough to reveal the selected pane, and stops at either end.
A lone half-width pane leaves the rest of the window empty.

The default prefix is **Ctrl+a**. Release the prefix before the next key:

| Key | Action |
| --- | --- |
| `c` | Insert a half-width pane after the current pane, in its current directory |
| `h` / `l` | Focus left / right |
| `f` | Toggle half / full width |
| `H` / `L` | Move the current pane left / right, keeping focus |
| `x` | Close the current pane |
| `Ctrl+a` | Send a literal Ctrl+a to the application |

Uppercase keys use Shift. Configuration can override the prefix and bindings.

The corresponding commands are `new-pane -L -c '#{pane_current_path}'`,
`select-pane -L` / `-R`, `resize-pane -W`, `swap-pane -U` / `-D`, and `kill-pane`.
Insertion accepts `-t` for a target and `-b` to insert before it. Plain `new-pane`
creates a floating overlay. Existing `-c` directory expansion and repeated
`-e NAME=value` environment overrides still apply; live shell exports are not
copied. Without `-c`, the existing client/session directory fallback applies.

Scrolling panes reject `split-window` and its aliases, split geometry on
`new-pane -L`, ordinary `resize-pane` size/direction flags, `select-layout -E`,
and border dragging. Use `resize-pane -W` to change their width. Floating panes
keep their normal resize controls. `resize-pane -Z` temporarily zooms a pane to
the visible window; unzoom restores the strip and its width preferences.

Use `refresh-client -L 10` / `-R 10` to pan explicitly, and `refresh-client -c`
to resume focus following. Changing focus also resumes following. Each client
has its own viewport; shared pane sizes follow the existing `window-size` policy.
A client narrower than a pane follows that pane's cursor within the strip.

`select-layout scrolling` returns to scrolling after another layout has been
selected. Traditional layouts keep their split/resize behavior and remain selected
across terminal resizing and respawning. `select-layout -o` restores the previously
selected arrangement and policy. JSON layouts include optional top-level
`"scrolling":{"width":80,"height":24}` sizing metadata and a `"full"` boolean
on each pane. Old JSON layouts still load. Legacy control-client layout strings
describe geometry only; internal previous-layout restoration retains the policy
even when that legacy output format is in use.

Half widths use `max(minimum, (visible_width - 1) / 2)` so two halves and their
separator fit. Full widths use the visible width. Borders, status rows, and reserved
scrollbars count within those dimensions. Tiny terminals honor pane minima while
retaining width preferences for later expansion. The visible sizing basis remains
limited to 10,000 cells per axis; the strip may be wider, within signed coordinate
limits.

`agentmon` creates and monitors coding-agent runs in git worktrees through hmux:

```sh
nix run .#agentmon
```

Run it inside an hmux pane to discover the current server, or pass
`-- --socket /path/to/socket` from another terminal. `nix develop` also puts
`agentmon` and its companion `looper` on `PATH`.

See [agentmon-tui/README.md](agentmon-tui/README.md) for usage and development.
