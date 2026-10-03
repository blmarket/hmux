# Scrolling window implementation plan

Status: written on 2026-10-03 against `main` at `f457d591`; it replaces the
layout-based design. Steps 1 to 3 are done (see "Where things stand"); step 4
is next. Line references are current as of step 3.

## Direction

The earlier plan made scrolling one layout among tmux's presets and kept it in
force by rearranging the layout tree after every command that edited it. That
was the wrong shape. The Window is the manager of its panes:

- The Window owns the pane order and each pane's width preference, and derives
  every pane's rectangle from them. Nothing else writes pane geometry, so the
  arrangement holds by construction rather than by triggers.
- The strip is the only arrangement. The layout tree, presets, sticky layouts,
  layout import, floating panes and zoom are removed, not adapted.
- Historical tmux names with no strip meaning are deleted, not kept as stubs
  (decision 1). `split-window` stays as an alias for insertion (decision 2).

## Decisions

Each open decision must be settled before the step named. The plan text
assumes the proposed answer to an open decision and marks it "proposed".

| # | Decision | Answer | Needed by |
| --- | --- | --- | --- |
| 1 | Keep tmux's command, option and format names as translations and stubs, or delete what has no strip meaning | Settled: delete | Step 3 |
| 2 | `split-window`: block it, or make it an alias for insertion | Settled: alias | Step 3 |
| 3 | `join-pane` and `break-pane` between windows | Settled: keep, as remove plus insert | Step 3 |
| 4 | `pane-border-status` (per-pane title row, off by default) | Proposed: drop | Step 6 |
| 5 | Active-pane indicator: half-coloured border on every separator | Proposed: adopt | Step 6 |
| 6 | Blank area beside a trailing half pane: inside fill or outside fill | Proposed: outside fill, as tmux paints beyond a smaller window | Step 6 |

Steps 1 and 2 had kept stub names while decision 1 was open; step 3 deleted
them (see "Removed in step 3").

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
- Use `new-pane` for pane insertion; adding a pane is a distinct operation from
  splitting one. `split-window` is accepted as an alias for insertion and
  ignores its direction and size flags (decision 2).
- Reuse the current `new-pane` environment and working-directory capabilities;
  do not add inheritance mechanics or shell integration. Use the existing `-c`
  option when the new pane should follow the source pane's current directory.
- Use `Ctrl+a` as the default prefix, with the key bindings specified below.

## Where things stand

| Piece | Commit | Fate |
| --- | --- | --- |
| Step 1: floating and modal panes removed | `b35d5683` | Done |
| Step 2: zoom removed | `bfc3730b` | Done |
| Step 3: the strip replaces the tree | The commit after `bfc3730b` | Done |
| Window sizing split (`Window::size()` vs. `layout::logical_size`) | `8e6e80db` | Kept. `Window::size()` stays the sizing basis; `Window::logical_size()` replaces `layout::logical_size`, and step 5 changes its extent rule (see Geometry) |
| Sticky layouts (`sticky-layout` option, `sticky` and `lastlayout` fields) | `4b6e6435` | Removed in step 3 with `tests/sticky_layout.rs` |
| Shared server harness (`tests/common/mod.rs`) | `4b6e6435` | Kept |

Reference material only, none of it mergeable: branch `h1` (worktree
`/home/blmarket/h1`) and `stash@{0}`. Port from `h1` the pane-based viewport
following (`src/tty.rs`, `src/server_client/api.rs`), menu placement per client
viewport, navigation that stops at the ends, the key map, the PTY test client
and test cases. Its mode flag and `src/layout/scrolling.rs` are superseded.

### Step 3

- `src/layout/` is gone. `src/window/strip.rs` holds the strip geometry and the
  layout-string serializer; `window_arrange` (`src/window/mod.rs`) is the arrange
  step and the only writer of pane geometry.
- The Window owns every membership and order edit: `add_pane`, `insert_pane`,
  `detach_pane`, `remove_pane`, `swap_pane_order`, `rotate_panes`,
  `reset_to_pane` (respawn-window) and `initialize_pane` (break-pane) each end in
  the arrange step; so do `resize` and `refit` (option changes, pane modes, the
  refit in `src/resize.rs`). `new_pane` checks room, then spawns. The pane-order
  and layout-tree guards, `forget_pane`, `split_pane`, `has_layout`,
  `sticky_layout` and `select_layout` left the trait.
- Notifications are unchanged in shape: removal and detaching fire
  `window-layout-changed` (as `layout_close_pane` did); insertion is silent and
  its caller notifies; resize always fires; refit fires only on movement.
- The pane lost its `layout_cell` link, `place_in_layout`, `detach_layout`,
  `split_minimum_width` and `set_layout_offset`. `apply_layout` takes a
  `layout_geometry`; every strip pane has the status row when
  `pane-border-status` is on. `prepare_render` treats a pane outside its
  window's order as unplaced.
- The horizontal minimum is the largest `minimum_layout_width` among the panes
  (it was the active pane's), so focus changes never change geometry.
- Until step 5, `Window::logical_size()` is the window size widened to the strip
  (last pane's right edge), not the extent rule, so the cursor-following viewport
  does not scroll a strip that fits.
- `#{window_layout}` is a single-row layout: a lone pane cell, or an `h` node of
  pane cells. `%layout-change` repeats it in the visible-layout field.
- `display-panes` draws separators and status rows from `Window::pane_cells()`.
  The redraw two-pane colour split always splits a vertical separator.

### Removed in step 3 (decision 1)

- Commands: `select-layout`, `next-layout`, `previous-layout`.
- Flags: `resize-pane -D/-L/-M/-R/-U/-x/-y/-Z` (only `-T` remains); `-Z` on
  `select-pane`, `last-pane`, `swap-pane`, `rotate-window`, `switch-client`,
  `new-pane`, `split-window`, `choose-tree`, `choose-client`, `choose-buffer`,
  `customize-mode`, `switch-mode`, `display-panes` and `find-window`;
  `new-pane -f/-h/-l/-p/-v`; `join-pane -f/-h/-l/-p/-v`.
- Options: `main-pane-height`, `main-pane-width`, `other-pane-height`,
  `other-pane-width`, `tiled-layout-max-columns`, `sticky-layout`; hooks
  `after-select-layout`, `window-zoomed`, `window-unzoomed`.
- Formats: `pane_floating_flag`, `pane_modal_flag`, `pane_zoomed_flag`,
  `window_zoomed_flag`, `window_visible_layout`.
- Bindings whose commands or flags are gone: `Space`, `E`, `M-1` to `M-7`, the
  eight resize arrows, `MouseDrag1Border`; `-Z` dropped from the `=`, `D`, `f`,
  `C`, `s` and `w` bindings. The defaults array has 261 entries.
- Already removed in steps 1 and 2: `move-pane`, `break-pane -W`, `pane_z`,
  `window_modal_pane`, `pane_unzoomed_width`/`_height`, the `move` key table, the
  float and zoom bindings and menu items.
- Kept: `select-pane -U/-D` and the `{up-of}`/`{down-of}` targets reach no pane
  in one row; step 5 decides them with navigation.

## Design

Follow `AGENTS.md` and `docs/trait-models.md`: do not edit `src/compat/` (it has
no layout references), add no shared ownership, keep model state behind the
holder traits, and keep explicit release operations. The arrangement is an
algorithm over Window-owned storage, so it lives inside the window module. End
borrows before resizing panes or dispatching callbacks.

### Window state

| State | Where | Change |
| --- | --- | --- |
| Pane order | `panes` (`src/window/model.rs:47`) | Existing; it is the strip order |
| Strip width | `strip_width` (`src/window/model.rs:49`) | Added in step 3; written only by the arrange step |
| Active pane, pane history | `active`, `last_panes` | Existing |
| Size | `sx`, `sy`, pending and manual sizes | Existing |
| Width preference | New value in the pane model (`src/window_pane/model.rs`) | Half by default |
| Pane rectangle | Pane `sx`, `sy`, `xoff`, `yoff` (`src/window_pane/model.rs:13-16`) | Existing; written only by the arrange step |
| Removed in steps 1 and 2 | `z_index`, `modal`, `modal_last`, `last_new_pane_x/y`, `saved_layout_root`, `was_zoomed`, the `WINDOW_ZOOMED` flag; the pane's `saved_layout_cell` and `PANE_ZOOMED` flag | Done |
| Removed in step 3 | `layout_root`, `old_layout`, `lastlayout`, `sticky`; the pane's `layout_cell` | Done |

### Arrange

Done in step 3. `window_arrange` (`src/window/mod.rs:804`) computes every
pane's rectangle from the pane order, `Window::size()` and the pane scrollbar
options (`src/window/strip.rs`), applies them through `apply_layout`
(`src/window_pane/api.rs:844`), records the strip width and invalidates the
scene on movement. Every Window operation that changes an input ends in it; see
"Step 3" for the operations and their notifications. Step 4 adds the width
preferences as an input; the width toggle notifies like the other operations.

### Geometry

With `(W, H) = Window::size()` and the panes in order:

| Quantity | Value |
| --- | --- |
| Half width | `(W - 1) / 2` rounded down, raised to the horizontal minimum (the largest pane minimum, `src/window/mod.rs:778`) |
| Full width | `W`, raised to the same minimum |
| Pane height | `H`, less scrollbar and status adjustments as today |
| Pane first column | Previous pane's first column, plus its width, plus one separator |
| Scrollable extent | Last pane's first column plus `W`, from step 5. Until then `Window::logical_size()` (`src/window/api.rs:402`) is `W` widened to the last pane's right edge |

- Two halves and their separator always fit in `W`. An even `W` leaves one spare
  column at the right edge.
- The extent rule makes every pane boundary reachable and gives the last pane's
  right border room to be drawn. A lone pane has an extent of exactly `W`.
- Removing a pane closes the gap without enlarging its neighbors.
- Refuse an insertion that would take the extent past `WINDOW_MAXIMUM` (10000);
  resizes and removals cannot fail.

### Viewport

- `tty_window_offset1` (`src/tty.rs:1043`) follows the active pane's cursor and
  centers on it. Follow the active pane's bounds instead: rest on the nearest
  offset that is a pane's first column and shows the active pane completely.
  Cursor movement inside a visible pane must not move the view.
- A client narrower than the active pane cannot show it fully; fall back to
  keeping the cursor visible. Pane geometry stays shared and follows
  `window-size`.
- Explicit panning stays in `Client` (`apply_pan`, `pan_window`, `reset_pan`,
  `src/server_client/api.rs:506-550`) and may leave the view off a boundary.
  Selecting a pane resets it; so does `refresh-client -c`.
- Left/right navigation (`window_pane_find_left`/`_right`,
  `src/window_pane/mod.rs:1804`, `:1849`) becomes the previous/next pane in
  order and stops at the ends. Up/down navigation has no target.
- Menus and popups are placed against the logical size
  (`src/cmd/entries/display_menu.rs:522`, `src/window/api.rs:487`). Place them
  within the target client's viewport.

### Borders

- Keep the marking rule in `redraw_mark_pane_borders`
  (`src/screen_redraw.rs:542`): left when there is a column before the pane,
  right when it is within the extent.
- Delete horizontal borders and junction cell types. The floating border and
  clipping code went in step 1. Panes never overlap, so visible-range clipping
  against other panes goes too.
- Active-pane indicator (proposed): on every separator the top half takes the
  left pane's style and the bottom half the right pane's. This replaces
  `redraw_check_two_pane_colours` (`src/screen_redraw.rs:271`), which only
  handles exactly two panes.
- The `display-panes` preview (`src/window_panes.rs`) draws from the pane
  rectangles since step 3.

### Commands

"Confirmed" rows follow the confirmed behavior; decisions 1 to 3 settled the
rest. "Done" marks rows step 3 completed.

| Command | In a strip window | Status |
| --- | --- | --- |
| `new-pane` | Inserts a half-width pane after the target; `-b` before it. `-c`, `-e`, `-E` and `-d` as today. Floating and modal flags (`-L`, `-M`, `-O`, `-x`, `-y`, `-X`, `-Y`) and the split flags (`-f`, `-h`, `-l`, `-p`, `-v`) are removed | Done |
| `kill-pane` | Unchanged; the strip closes the gap | Done |
| `select-pane -L` / `-R` | Previous/next pane, stopping at the ends | Confirmed |
| `resize-pane -W` | New. Toggles the target between half and full width. The template is `Tt:` (`src/cmd/entries/resize_pane.rs:16`) | Confirmed |
| `swap-pane -U` / `-D`, `-s`/`-t` | Reorder the pane list; the width preference travels with the pane; `-U`/`-D` stop at the ends | Done, except the width preference (step 4) |
| `refresh-client -L` / `-R` / `-c` | Manual pan and reset, unchanged | Confirmed |
| `display-panes` | Kept; the preview draws the strip | Done |
| `split-window` | Alias for insertion; `-f`, `-h`, `-l`, `-p` and `-v` are accepted and ignored | Done (decision 2) |
| `-Z` on `resize-pane`, choosers and other commands | Removed | Done (decision 1) |
| `select-layout`, `next-layout`, `previous-layout` | Removed | Done (decision 1) |
| `resize-pane -x/-y/-U/-D/-L/-R/-M` | Removed; `-T` remains | Done (decision 1) |
| `rotate-window` | Rotates the pane order | Done |
| `join-pane`, `break-pane` | Remove from one strip and insert into another, or into a new window. `join-pane` keeps `-b`, `-d`, `-s` and `-t`; a full destination strip refuses | Done (decision 3) |
| `move-pane`, `break-pane -W` | Removed in step 1; they positioned or created floating panes | Done |
| `main-pane-*`, `other-pane-*`, `tiled-layout-max-columns`, `after-select-layout` | Removed | Done (decision 1) |
| `sticky-layout` | Removed; it was added in `4b6e6435` and is not a tmux option | Done |
| `#{window_layout}`, `%layout-change` | Generated from the pane rectangles as a single-row layout string; `%layout-change` repeats it as its visible layout | Done |
| `#{window_visible_layout}`, zoom and floating format variables, `window-zoomed`/`window-unzoomed` hooks | Removed | Done (decision 1) |

### Default keys

`Ctrl+a` is a prefix sequence: release it before pressing the following key.

| Keys | Action | Command | Today (`src/key_bindings.rs`) |
| --- | --- | --- | --- |
| `Ctrl+a c` | Insert a half-width pane after the active pane, using its current directory | `new-pane -c '#{pane_current_path}'` | `new-window` (`:238`) |
| `Ctrl+a h` / `Ctrl+a l` | Focus the pane to the left/right; stop at the strip ends | `select-pane -L` / `select-pane -R` | `h` unbound; `l` is `last-window` (`:242`) |
| `Ctrl+a f` | Toggle the active pane between half-width and full-width | `resize-pane -W` | `find-window` prompt (`:240`) |
| `Ctrl+a H` / `Ctrl+a L` | Reorder the active pane one position left/right | `swap-pane -U` / `swap-pane -D` | `H` unbound; `L` is `switch-client -l` (`:233`) |
| `Ctrl+a x` | Close the active pane | `kill-pane` | `confirm-before … kill-pane` (`:255`) |
| `Ctrl+a Ctrl+a` | Send a literal Ctrl+a to the application | `send-prefix` | `C-b` (`:202`) |

- The prefix default is `C-b` at `src/options_table.rs:1383`.
- Uppercase `H` and `L` are bound as literal characters, distinct from lowercase.
- Remove the bindings for split and break features: splits (`"`, `%`) and `!`.
  Step 3 already removed the layout, tiled resize and border-drag bindings, and
  steps 1 and 2 the floating and zoom ones; see "Removed in step 3".
- `Tab` and `BTab` already open their choosers in a temporary pane with
  `new-pane -E` (`src/key_bindings.rs:252-253`, step 1).
- The defaults array has a hardcoded length of 261 (`src/key_bindings.rs:201`).

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
2. **Remove zoom.** Done in `bfc3730b`. Removed the saved tree, the
   visible/unzoomed views (`LayoutView`), push/pop zoom and the unzoom guards
   around commands, the zoom events, pane visibility, and the chooser zoom
   state. The `-Z` flags it left as no-ops went in step 3.
3. **The strip replaces the tree.** Done. The arrange step places every pane
   half-width, and pane add, remove and reorder go through the Window. All of
   `src/layout/` is deleted; the strip geometry and the layout string live in
   `src/window/strip.rs`. The sticky machinery and the tree methods on the
   Window trait are gone, the command table is applied with decisions 1 to 3,
   and `display-panes` draws from rectangles. `tests/sticky_layout.rs` is
   deleted, `tests/window_sizes.rs` covers the strip and its lifecycle, and the
   harness and four other test files use `new-pane`. `copy_regex_cells` widens
   its window so its lone half pane keeps ten columns. See "Step 3".
4. **Width preference.** Add the pane value and `resize-pane -W`. Reordering
   carries the preference.
5. **Viewport.** Pane-boundary following, the extent rule, pan reset on
   selection, navigation that stops at the ends, menu and popup placement,
   mouse coordinates after panning. Add a PTY client to `tests/common/mod.rs`.
6. **Borders.** Delete horizontal borders and junctions, add the active-pane
   indicator, and apply decisions 4 and 6.
7. **Defaults and keys.** `C-a` prefix, the key map, removal of the split and
   break bindings, accurate help text. Fix tests that assume the old defaults;
   `h1` touched `client_file_protocol`, `copy_regex_cells`,
   `mode_prompt_cleanup`, `plugins` and `session_group_membership`.
8. **Documentation.** README section describing the strip, the removed names
   and the `split-window` alias. `agentmon-tui/src/agentmon/services.py:1274`
   keeps working through the alias (decision 2).

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
| Compatibility | `split-window` inserts and ignores its split flags; removed commands, flags and options are rejected; `#{window_layout}` is a single-row layout |
| Regression | `tests/model_trait_boundary.rs`; copy mode, choosers, menus and popups; `src/compat/` untouched |

Run the Rust tests after each step, then a build and isolated interactive smoke
checks.

## Known gaps

- A pending resize survives when no eligible client remains. Intentional.
- Session-restore tools that replay `select-layout <string>` fail on the
  removed command; the panes they create still form a strip in order.
- Two-dimensional tiling is gone whatever a command asks for.

## Completion criteria

The confirmed behavior works through normal hmux commands. Pane geometry remains
stable across creation, removal, focus changes and terminal resizing. The Window
has no layout tree, and the tests exercise the strip and its lifecycle.
