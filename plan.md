# Scrolling window implementation plan

Status: written on 2026-10-03 against `main` at `f457d591`; it replaces the
layout-based design. Steps 1 and 2 are done (see "Where things stand"); step 3
is next. Line references are current as of step 2.

## Direction

The earlier plan made scrolling one layout among tmux's presets and kept it in
force by rearranging the layout tree after every command that edited it. That
was the wrong shape. The Window is the manager of its panes:

- The Window owns the pane order and each pane's width preference, and derives
  every pane's rectangle from them. Nothing else writes pane geometry, so the
  arrangement holds by construction rather than by triggers.
- The strip is the only arrangement. The layout tree, presets, sticky layouts,
  layout import, floating panes and zoom are removed, not adapted.
- Whether historical tmux commands stay as translations onto the strip is an
  open decision; see "Open decisions".

## Open decisions

Each must be settled before the step named. The plan text assumes the proposed
answer and marks it "proposed".

| # | Decision | Proposed | Needed by |
| --- | --- | --- | --- |
| 1 | Keep tmux's command, option and format names as translations and stubs, or delete what has no strip meaning | Keep the names | Step 3 |
| 2 | `split-window`: the confirmed behavior below blocks it; decision 1 would make it an alias for insertion | Alias | Step 3 |
| 3 | `join-pane` and `break-pane` between windows | Keep, as remove plus insert | Step 3 |
| 4 | `pane-border-status` (per-pane title row, off by default) | Drop | Step 6 |
| 5 | Active-pane indicator: half-coloured border on every separator | Adopt | Step 6 |
| 6 | Blank area beside a trailing half pane: inside fill or outside fill | Outside fill, as tmux paints beyond a smaller window | Step 6 |

Steps 1 and 2 followed the proposed answer to decision 1 wherever they had to
choose; "Left by steps 1 and 2" lists what they kept. If decision 1 is
rejected, step 3 deletes those names along with the layout ones.

## Confirmed behavior

- Every window is a scrolling strip, including the first window of a new session.
- Panes sit in one horizontal strip that may extend beyond the visible window.
- Each pane uses the full available height; no vertical stacks.
- Each pane independently uses half or all of the visible width. Mixed widths are allowed.
- New panes start at half-width and appear immediately to the right of the active pane.
- Selecting a pane scrolls to the nearest pane boundary that shows that pane
  completely. A pane boundary is a view whose first column is a pane's first
  column; the view always rests on one, so it never sits one column off. On an
  even width the spare column is therefore always at the right edge.
- Adding panes extends the strip instead of squeezing the existing panes.
- Left/right navigation stops at the first/last pane; it does not wrap around.
- A lone half-width pane stays half-width, leaving empty space beside it.
- Width changes use only an explicit half/full toggle, not explicit size setters.
- No floating panes. A temporary pane is created in the strip and closed.
- No zoom. Toggling a pane to full width replaces it.
- Borders follow tmux's existing rule: a pane draws its right border whenever
  there is room. A lone half pane therefore has a border at its right, and two
  halves on an even width show the last pane's border in the final column.
- Use `new-pane` for pane insertion and block existing split commands; adding a
  pane is a distinct operation from splitting one. (Open decision 2 revisits this.)
- Reuse the current `new-pane` environment and working-directory capabilities;
  do not add inheritance mechanics or shell integration. Use the existing `-c`
  option when the new pane should follow the source pane's current directory.
- Use `Ctrl+a` as the default prefix, with the key bindings specified below.

## Where things stand

| Piece | Commit | Fate |
| --- | --- | --- |
| Step 1: floating and modal panes removed | `b35d5683` | Done |
| Step 2: zoom removed | The commit after `b35d5683` | Done |
| Window sizing split (`Window::size()` vs. `layout::logical_size`) | `8e6e80db` | Kept. `Window::size()` stays the sizing basis; the extent rule changes (see Geometry) |
| Sticky layouts (`sticky-layout` option, `sticky` and `lastlayout` fields) | `4b6e6435` | Removed in step 3 with `tests/sticky_layout.rs` (17 tests) |
| Shared server harness (`tests/common/mod.rs`) | `4b6e6435` | Kept |

Reference material only, none of it mergeable: branch `h1` (worktree
`/home/blmarket/h1`) and `stash@{0}`. Port from `h1` the pane-based viewport
following (`src/tty.rs`, `src/server_client/api.rs`), menu placement per client
viewport, navigation that stops at the ends, the key map, the PTY test client
and test cases. Its mode flag and `src/layout/scrolling.rs` are superseded.

### Left by steps 1 and 2

- Format variables kept as constant `0`: `pane_floating_flag`,
  `pane_modal_flag`, `pane_zoomed_flag`, `window_zoomed_flag`. Removed:
  `pane_z`, `window_modal_pane`, and `pane_unzoomed_width`/`_height`, which
  equalled `pane_width`/`pane_height` once zoom was gone. The
  `display-panes-format` default now uses `pane_width`/`pane_height`.
  `#{window_visible_layout}` returns the same string as `#{window_layout}`.
- The `window-zoomed` and `window-unzoomed` hooks are accepted and never run.
- `-Z` is accepted and ignored by `resize-pane`, `select-pane`, `swap-pane`,
  `rotate-window`, `switch-client`, `split-window`/`new-pane`, the mode-tree
  choosers (`choose-tree`, `choose-buffer`, `choose-client`, `customize-mode`,
  `find-window`), `display-panes` and `switch-mode`.
- `display-panes` draws its preview inside the mode pane instead of zooming it.
- `WindowPane::is_visible` was false only for panes hidden by zoom and is gone.
  Step 5 decides visibility per client from the viewport.
- `layout_fix_panes` lost its skip argument and `layout_assign_pane` its
  do-not-resize flag; only zoomed splits used them.
- Bindings already removed: `*`, `@`, the `move` key table, the floating mouse
  bindings (`C-MouseDrag1Pane`, `C-MouseDrag1Empty`, `M-MouseDrag1Pane`,
  `M-MouseDrag1Border`, `MouseDown1Control7`), `z`, `MouseDown1Control8`, and
  the Zoom item in the three pane menus. The matching float and zoom buttons
  are gone from `pane-border-format`, and the tree-mode flag lines no longer
  show "zoomed".

## Design

Follow `AGENTS.md` and `docs/trait-models.md`: do not edit `src/compat/` (it has
no layout references), add no shared ownership, keep model state behind the
holder traits, and keep explicit release operations. The arrangement is an
algorithm over Window-owned storage, so it lives inside the window module. End
borrows before resizing panes or dispatching callbacks.

### Window state

| State | Where | Change |
| --- | --- | --- |
| Pane order | `panes` (`src/window/model.rs:49`) | Existing; it is the strip order |
| Active pane, pane history | `active`, `last_panes` | Existing |
| Size | `sx`, `sy`, pending and manual sizes | Existing |
| Width preference | New value in the pane model (`src/window_pane/model.rs`) | Half by default |
| Pane rectangle | Pane `sx`, `sy`, `xoff`, `yoff` (`src/window_pane/model.rs:15-18`) | Existing; written only by the arrange step |
| Removed in steps 1 and 2 | `z_index`, `modal`, `modal_last`, `last_new_pane_x/y`, `saved_layout_root`, `was_zoomed`, the `WINDOW_ZOOMED` flag; the pane's `saved_layout_cell` and `PANE_ZOOMED` flag | Done |
| Removed in step 3 | `layout_root`, `old_layout`, `lastlayout`, `sticky`; the pane's `layout_cell` | |

### Arrange

One private step in the window module computes every pane's rectangle from the
pane order, the width preferences, `Window::size()` and the pane scrollbar
options, applies them to the panes, and reports whether anything moved. Every
Window operation that changes an input ends in it: add pane, remove pane,
reorder, width toggle, resize, and the option refit in `src/resize.rs:457`.
On movement it invalidates the scene, fires `window-layout-changed` and redraws,
as `layout_set_select` and `refit_layout` do today; with no movement it is
silent.

`apply_layout` (`src/window_pane/api.rs:868`) takes a layout cell today; it
takes a rectangle instead. `layout_fix_panes` and `layout_resize_limits` lose
their tree and move into the arrange step.

### Geometry

With `(W, H) = Window::size()` and the panes in order:

| Quantity | Value |
| --- | --- |
| Half width | `(W - 1) / 2` rounded down, raised to the horizontal minimum |
| Full width | `W`, raised to the same minimum |
| Pane height | `H`, less scrollbar and status adjustments as today |
| Pane first column | Previous pane's first column, plus its width, plus one separator |
| Scrollable extent | Last pane's first column plus `W`; this replaces `layout::logical_size` (`src/layout/core.rs:557`) |

- Two halves and their separator always fit in `W`. An even `W` leaves one spare
  column at the right edge.
- The extent rule makes every pane boundary reachable and gives the last pane's
  right border room to be drawn. A lone pane has an extent of exactly `W`.
- Removing a pane closes the gap without enlarging its neighbors.
- Refuse an insertion that would take the extent past `WINDOW_MAXIMUM` (10000);
  resizes and removals cannot fail.

### Viewport

- `tty_window_offset1` (`src/tty.rs:1055`) follows the active pane's cursor and
  centers on it. Follow the active pane's bounds instead: rest on the nearest
  offset that is a pane's first column and shows the active pane completely.
  Cursor movement inside a visible pane must not move the view.
- A client narrower than the active pane cannot show it fully; fall back to
  keeping the cursor visible. Pane geometry stays shared and follows
  `window-size`.
- Explicit panning stays in `Client` (`apply_pan`, `pan_window`, `reset_pan`,
  `src/server_client/api.rs:506-545`) and may leave the view off a boundary.
  Selecting a pane resets it; so does `refresh-client -c`.
- Left/right navigation (`window_pane_find_left`/`_right`,
  `src/window_pane/mod.rs:1806`, `:1851`) becomes the previous/next pane in
  order and stops at the ends. Up/down navigation has no target.
- Menus and popups are placed against the logical size
  (`src/cmd/entries/display_menu.rs:522`, `src/window/mod.rs:522`). Place them
  within the target client's viewport.

### Borders

- Keep the marking rule in `redraw_mark_pane_borders`
  (`src/screen_redraw.rs:559`): left when there is a column before the pane,
  right when it is within the extent.
- Delete horizontal borders and junction cell types. The floating border and
  clipping code went in step 1. Panes never overlap, so visible-range clipping
  against other panes goes too.
- Active-pane indicator (proposed): on every separator the top half takes the
  left pane's style and the bottom half the right pane's. This replaces
  `redraw_check_two_pane_colours` (`src/screen_redraw.rs:270`), which only
  handles exactly two panes and reads the tree.
- The `display-panes` preview (`src/window_panes.rs`) walks the tree to draw
  scaled borders; draw it from the pane rectangles.

### Commands

"Confirmed" rows follow the confirmed behavior. "Proposed" rows depend on open
decisions 1 to 3.

| Command | In a strip window | Status |
| --- | --- | --- |
| `new-pane` | Inserts a half-width pane after the target; `-b` before it. `-c`, `-e`, `-E` and `-d` as today. Floating and modal flags (`-L`, `-M`, `-O`, `-x`, `-y`, `-X`, `-Y`) are removed | Confirmed |
| `kill-pane` | Unchanged; the strip closes the gap | Confirmed |
| `select-pane -L` / `-R` | Previous/next pane, stopping at the ends | Confirmed |
| `resize-pane -W` | New. Toggles the target between half and full width. `W` is free in the template (`src/cmd/entries/resize_pane.rs:40`) | Confirmed |
| `swap-pane -U` / `-D`, `-s`/`-t` | Reorder the pane list; the width preference travels with the pane; `-U`/`-D` stop at the ends | Confirmed |
| `refresh-client -L` / `-R` / `-c` | Manual pan and reset, unchanged | Confirmed |
| `display-panes` | Kept | Confirmed |
| `split-window` | Alias for insertion; direction and size flags ignored. Otherwise rejected with a clear error before any side effect | Proposed (decision 2) |
| `resize-pane -Z`, `-Z` on choosers | `resize-pane -Z` toggles full width; the chooser flag is accepted and ignored. Both have been accepted no-ops since step 2 | Proposed |
| `select-layout`, `next-layout`, `previous-layout` | Parse, check the target, succeed, do nothing | Proposed |
| `resize-pane -x/-y/-U/-D/-L/-R/-M` | Succeed, do nothing | Proposed |
| `rotate-window` | Rotates the pane order | Proposed |
| `join-pane`, `break-pane` | Remove from one strip and insert into another, or into a new window | Proposed (decision 3) |
| `move-pane`, `break-pane -W` | Removed in step 1; they positioned or created floating panes | Confirmed |
| `main-pane-*`, `other-pane-*`, `tiled-layout-max-columns` | Still accepted, no effect | Proposed |
| `sticky-layout` | Removed; it was added in `4b6e6435` and is not a tmux option | Confirmed |
| `#{window_layout}`, `#{window_visible_layout}`, `%layout-change` | Generated from the pane rectangles as a single-row layout string | Proposed |
| Zoom and floating format variables | Constant 0, done in steps 1 and 2; see "Left by steps 1 and 2" | Proposed |

### Default keys

`Ctrl+a` is a prefix sequence: release it before pressing the following key.

| Keys | Action | Command | Today (`src/key_bindings.rs`) |
| --- | --- | --- | --- |
| `Ctrl+a c` | Insert a half-width pane after the active pane, using its current directory | `new-pane -c '#{pane_current_path}'` | `new-window` (`:240`) |
| `Ctrl+a h` / `Ctrl+a l` | Focus the pane to the left/right; stop at the strip ends | `select-pane -L` / `select-pane -R` | `h` unbound; `l` is `last-window` (`:244`) |
| `Ctrl+a f` | Toggle the active pane between half-width and full-width | `resize-pane -W` | `find-window` prompt (`:242`) |
| `Ctrl+a H` / `Ctrl+a L` | Reorder the active pane one position left/right | `swap-pane -U` / `swap-pane -D` | `H` unbound; `L` is `switch-client -l` (`:235`) |
| `Ctrl+a x` | Close the active pane | `kill-pane` | `confirm-before … kill-pane` (`:257`) |
| `Ctrl+a Ctrl+a` | Send a literal Ctrl+a to the application | `send-prefix` | `C-b` (`:202`) |

- The prefix default is `C-b` at `src/options_table.rs:1383`.
- Uppercase `H` and `L` are bound as literal characters, distinct from lowercase.
- Remove the bindings for deleted features: splits (`"`, `%`), layouts (`Space`,
  `E`, `M-1` to `M-7`), tiled resize (the eight arrow bindings), `!`, and the
  mouse bindings that resize panes by dragging. Steps 1 and 2 already removed
  the floating and zoom bindings; see "Left by steps 1 and 2".
- `Tab` and `BTab` already open their choosers in a temporary pane with
  `new-pane -E` (`src/key_bindings.rs:254-255`, step 1).
- The defaults array has a hardcoded length of 279 (`src/key_bindings.rs:201`).

### Working directory and environment

Unchanged (`src/cmd/entries/split_window.rs`, `src/window_pane/spawning.rs`,
`src/server_client/api.rs`, `src/environ.rs`):

- The spawn environment is the global environment, overlaid by the session
  environment and repeated `-e NAME=value` arguments, plus child-specific
  variables such as `TMUX_PANE`, `SHELL` and `PWD`.
- Without `-c`, an unattached command client's directory is used; an attached
  client's key binding normally uses the session directory. Neither follows the
  target pane's `cd`.
- `-c '#{pane_current_path}'` requests the pane's detected current directory.
- Later `export`/`unset` changes inside a pane's shell are not copied.

Do not add automatic cwd inheritance, live environment copying, shell hooks, or
a new environment transport.

## Implementation sequence

Each step ends with passing tests. Steps 1 and 2 delete features while the
tree still exists, so step 3 replaces a purely tiled tree.

1. **Remove floating and modal panes.** Done in `b35d5683`. Plain `new-pane`
   creates a tiled pane. Removed `move-pane`, `break-pane -W`, the float/tile
   toggle, the stacking order, the `move` key table and the floating code in
   redraw, the `display-panes` preview and `src/layout/core.rs`. The `Tab`/`BTab`
   choosers use a temporary tiled pane.
2. **Remove zoom.** Done in the commit after `b35d5683`. Removed the saved
   tree, the visible/unzoomed views (`LayoutView`), push/pop zoom and the
   unzoom guards around commands, the zoom events, pane visibility, and the
   chooser zoom state. `resize-pane -Z` and the chooser `-Z` flag are accepted
   and do nothing until step 4. `tests/window_sizes.rs` checks that the `-Z`
   flags change nothing.
3. **The strip replaces the tree.** Add the arrange step with every pane
   half-width, and route pane add, remove and reorder through the Window.
   Delete `src/layout/set.rs`, layout import in `src/layout/custom.rs`, the tree
   operations in `src/layout/core.rs`, the sticky machinery and the tree methods
   on the Window trait. Apply the command table. Draw the `display-panes`
   preview from rectangles. Delete `tests/sticky_layout.rs`; rewrite
   `tests/window_sizes.rs`; move the seven test files that use `split-window`.
4. **Width preference.** Add the pane value and `resize-pane -W`; map
   `resize-pane -Z` to it (proposed). Reordering carries the preference.
5. **Viewport.** Pane-boundary following, the extent rule, pan reset on
   selection, navigation that stops at the ends, menu and popup placement,
   mouse coordinates after panning. Add a PTY client to `tests/common/mod.rs`.
6. **Borders.** Delete horizontal borders and junctions, add the active-pane
   indicator, and apply decisions 4 and 6.
7. **Defaults and keys.** `C-a` prefix, the key map, removal of dead bindings,
   accurate help text. Fix tests that assume the old defaults; `h1` touched
   `client_file_protocol`, `copy_regex_cells`, `mode_prompt_cleanup`, `plugins`
   and `session_group_membership`.
8. **Compatibility and documentation.** One test that runs a list of historical
   commands and a typical configuration and asserts they succeed (if decision 1
   is accepted). README section. Replace the `split-window` call in
   `agentmon-tui/src/agentmon/services.py:1274` if decision 2 rejects it.

## Validation

`tests/common/mod.rs` provides `Server` (isolated server, `window-size manual`,
event counters) and a control client. Viewport and rendering cases need the PTY
client from step 5.

| Area | Required cases |
| --- | --- |
| Geometry | Lone half pane; lone full pane; two halves; mixed widths; odd and even widths; tiny terminals; scrollbars; the extent limit |
| Lifecycle | Insert after focus; insert before target; remove first/middle/last; reorder keeps focus and width preference; move between windows |
| Width | Toggle twice restores the width; focus and other widths unchanged; preference survives terminal resize |
| Resizing | Grow and shrink the terminal; no repeated resize scheduling; silent when nothing moves |
| Focus | Already visible pane; partially visible pane; offscreen pane in each direction; no wrapping at either end; no cursor-driven drift |
| Viewport | The view rests on a pane boundary from both directions on even and odd widths; manual pan and reset; clients of different widths; active pane wider than a client |
| Rendering | Right border of a lone half pane; spare column on even widths; active-pane indicator with three or more panes; mouse selection after panning |
| Environment | Session environment and `-e` overrides; `-c '#{pane_current_path}'` after `cd`; paths with spaces; directory fallback |
| Defaults and keys | `C-a` prefix; c/h/l/f/H/L/x actions; uppercase/lowercase distinct; prefix passthrough; user overrides |
| Compatibility | Historical commands and a typical configuration succeed; removed options are accepted; `#{window_layout}` parses as a tmux layout |
| Regression | `tests/model_trait_boundary.rs`; copy mode, choosers, menus and popups; `src/compat/` untouched |

Run the Rust tests after each step, then a build and isolated interactive smoke
checks.

## Known gaps

- A pending resize survives when no eligible client remains. Intentional.
- `select-layout <string>` from session-restore tools cannot restore geometry;
  only pane count and order survive.
- Two-dimensional tiling is gone whatever a command asks for.

## Completion criteria

The confirmed behavior works through normal hmux commands. Pane geometry remains
stable across creation, removal, focus changes and terminal resizing. The Window
has no layout tree, and the tests exercise the strip and its lifecycle.
