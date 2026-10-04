# Scrolling window implementation plan

Status: written on 2026-10-03 against `main` at `f457d591`; it replaces the
layout-based design. Steps 1 to 6 are done (see "Where things stand"); step 7
is next. Line references are current as of step 6.

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
| 4 | `pane-border-status` (per-pane title row, off by default) | Settled: drop | Step 6 |
| 5 | Active-pane indicator: half-coloured border on every separator, or tmux's rule | Settled: keep tmux's rule | Step 6 |
| 6 | Blank area beside a trailing half pane: inside fill or outside fill | Settled: outside fill, as tmux paints beyond a smaller window | Step 6 |
| 7 | Menus and popups, the client overlays | Settled: delete, with the overlay mechanism | Done after step 5 |

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
- No floating panes, popups or menus; nothing is drawn over the strip. A
  temporary pane is created in the strip and closed.
- No zoom. Toggling a pane to full width replaces it.
- Borders follow tmux's existing rule: a pane draws its right border whenever
  there is room. A lone half pane therefore has a border at its right, and two
  halves on an even width show the last pane's border in the final column.
- Borders are vertical separators as tall as the panes; nothing draws a
  horizontal border or a junction. Whatever no pane, scrollbar or separator
  covers shows the outside fill, beside the last pane and below a shorter
  window alike.
- The active pane's separators take `pane-active-border-style` as in tmux: each
  separator beside the active pane is coloured whole, except that with exactly
  two panes their one separator is split, the top half taking the left pane's
  style and the bottom half the right pane's (decision 5).
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
| Step 3: the strip replaces the tree | `c61820db` | Done |
| Step 4: width preference and `resize-pane -W` | `ebd35f47` | Done |
| Step 5: viewport | The commit after `ebd35f47` | Done |
| Overlays removed: menus, popups (decision 7) | With step 5 | Done |
| Step 6: borders, pane status rows removed (decisions 4 to 6) | The commit after `f09340de` | Done |
| Window sizing split (`Window::size()` vs. `layout::logical_size`) | `8e6e80db` | Kept. `Window::size()` stays the sizing basis; `Window::logical_size()` replaces `layout::logical_size` and, since step 5, is the scrollable extent (see Geometry) |
| Sticky layouts (`sticky-layout` option, `sticky` and `lastlayout` fields) | `4b6e6435` | Removed in step 3 with `tests/sticky_layout.rs` |
| Shared server harness (`tests/common/mod.rs`) | `4b6e6435` | Kept |

Reference material only, none of it mergeable: branch `h1` (worktree
`/home/blmarket/h1`) and `stash@{0}`. Step 5 ported its PTY test client and
replaced its minimal-scroll viewport with pane-boundary following. Still to
port: the key map. Its mode flag and `src/layout/scrolling.rs` are
superseded; step 6 wrote its own rendering cases.

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
  When the removed pane was active, `window_take_pane`
  (`src/window/mod.rs:731`) makes its replacement active before arranging but
  fires `window-pane-changed` only after `window-layout-changed`, the order
  tmux and the tree used. Step 3 had reversed them; the fix came after step 6
  and `tests/window_sizes.rs` checks the order for `kill-pane` and
  `break-pane`.
- The pane lost its `layout_cell` link, `place_in_layout`, `detach_layout`,
  `split_minimum_width` and `set_layout_offset`. `apply_layout` takes a
  `layout_geometry`; until step 6 every strip pane had the status row when
  `pane-border-status` was on. `prepare_render` treats a pane outside its
  window's order as unplaced.
- The horizontal minimum is the largest `minimum_layout_width` among the panes
  (it was the active pane's), so focus changes never change geometry.
- Until step 5, `Window::logical_size()` was the window size widened to the
  strip (last pane's right edge), so the cursor-following viewport did not scroll
  a strip that fit. Step 5 replaced it with the extent rule.
- `#{window_layout}` is a single-row layout: a lone pane cell, or an `h` node of
  pane cells. `%layout-change` repeats it in the visible-layout field.
- `display-panes` draws separators (and, until step 6, status rows) from
  `Window::pane_cells()`. The redraw two-pane colour split always splits a
  vertical separator.

### Step 4

- `PaneWidth` (`src/shared/layout.rs`) is `Half` or `Full`. The preference is
  the pane's `width` (`src/window_pane/model.rs:19`), read and written through
  `WindowPane::width_preference` and `set_width_preference`. It starts `Half`
  and only `Window::toggle_pane_width` writes it, so it travels with the pane
  through swaps, rotation, `join-pane` and `break-pane`, and respawn-window
  keeps it.
- `strip::cells` and `strip::width` take the panes' widths; the arrange step
  reads them from the panes in order.
- `Window::toggle_pane_width` (`src/window/api.rs:365`) flips the preference,
  arranges (which updates the client offsets since step 5), redraws and fires
  `window-layout-changed`, as `resize_window` does. Widening is refused when the
  extent would pass `WINDOW_MAXIMUM` (the strip width until step 5); narrowing
  never fails, even in a strip a resize took past it.
- `has_room_for_pane` takes the incoming pane's width: `new-pane` asks for a half
  pane, `join-pane` for the moving pane's preference.
- `resize-pane -W` (template `TWt:`) toggles the target and fails with "no space
  for a full-width pane" on refusal. `-W` and `-T` combine.

### Step 5

- The extent: `strip::extent` (`src/window/strip.rs:58`) is the last pane's
  first column plus the window width, at least the last pane's right edge, and
  the window width without panes. The arrange step records it in `extent`
  (`src/window/model.rs:54`) beside `strip_width`; `Window::logical_size()`
  (`src/window/api.rs:411`) returns it. `window_strip_fits` checks the extent
  against `WINDOW_MAXIMUM`, so three halves of a 5000-column window are the
  most it holds.
- Following: `tty_window_offset1` (`src/tty.rs:1077`) runs `tty_follow_cell`
  (`src/tty.rs:1047`) once per axis. Horizontally the stops are the panes' first
  columns; vertically the only stop is the top row. Among the stops that show the
  active pane's cell completely and stay within the extent, the view takes the
  one nearest its previous offset, the left one on a tie. It ignores the cursor,
  so cursor movement in a visible pane never moves it. A view too short for the
  cell stays within the cell and moves the least distance that keeps the cursor
  visible.
- `bigger` (terminal clipping, `#{window_bigger}`, `#{window_offset_x}`) is set
  only when the view leaves part of a pane out: a nonzero offset, or a strip
  wider or a window taller than the view. A view at the origin that shows every
  pane keeps the unclipped drawing path, as two halves did before step 5.
- The arrange step (`window_arrange`, `src/window/mod.rs:798`) ends in
  `tty_update_window_offset`, so every geometry change re-follows. The explicit
  calls after the arrange in `toggle_pane_width` and `resize_window` went.
- Panning: `Client::reset_pan` (`src/server_client/api.rs:476`) takes an
  optional window and clears only that window's pan. `window_set_active_pane`
  (`src/window/mod.rs:528`) resets every client's pan of the window when the
  active pane changes; selecting the already active pane keeps the pan.
  `refresh-client -c` and the follow path reset unconditionally, as before.
- Navigation: `select-pane -L`/`-R` and the `{left-of}`/`{right-of}` targets use
  `Window::step_pane`, the previous/next pane without wrapping. Applying decision
  1, `select-pane -U`/`-D`, the `{up-of}`/`{down-of}` targets and the `Up`/`Down`
  bindings are removed; the defaults array has 259 entries. The geometric
  `neighbor_left`/`_right`/`_up`/`_down` lookups left the pane trait with
  `window_pane_find_*`, `window_pane_choose_best` and
  `src/window_pane/storage_tests.rs`.
- Menus were placed within the opening client's view, then removed with all
  overlays (see "Overlays removed").
- Mouse events were already translated by the client's view offset; the PTY
  tests confirm clicks after panning.
- Positions measured across the panes rather than the extent: the `{left}`,
  `{right}`, `{top}`, … targets (`window_find_string`, `src/window/mod.rs:619`)
  use the strip width, `#{pane_at_right}` is the last pane in order, and the
  `display-panes` preview scales the strip widened to the window, not the
  extent.
- Tests: `tests/common/mod.rs` gained `TerminalClient` (a pseudo-terminal
  client with a small screen model, `wait_screen`, `click` and per-client
  offsets). `tests/viewport.rs` covers focus in both directions on even and odd
  widths, the extent, cursor drift, pan and reset, clients of different widths,
  a client narrower than the active pane, mouse selection after panning,
  navigation ends and positional targets. `tests/window_sizes.rs`
  checks the maximum against the extent.

### Step 6

- Borders: `redraw_mark_pane_borders` (`src/screen_redraw.rs:389`) marks only
  the vertical separators beside a pane, over the pane's rows: the one before
  it when a column precedes the pane, the one after it while that column is
  within the extent. The top and bottom borders, their corners and the
  junction cell types (`redraw_get_cell_type`, the `REDRAW_BORDER_L/R/U/D`
  masks) went, as did the top and bottom arrow indicators. A border span keeps
  only its left and right panes, its style pane and its arrow flag
  (`RedrawBorderSpan`, `src/shared/redraw.rs`); every separator draws the
  vertical glyph, so `window_get_border_cell` and `WindowPane::border_cell`
  lost their cell type. The `CELL_*` junctions stay for `screen_write` boxes.
- Fill (decision 6): `redraw_reset_cell` (`src/screen_redraw.rs:186`) starts
  every cell as outside, so whatever no pane, scrollbar or separator covers
  shows the outside fill, past the last pane as much as below a shorter window.
  The inside fill had no other use and went: the `Empty` span and
  `REDRAW_EMPTY`, the Window's second fill cell (`fill_cell` in
  `src/window/model.rs:68`, rendered by `Window::refresh_fill_cell`,
  `src/window/api.rs:436`) and the `is_inside`/`is_outside` format variables.
  The `fill-character` default is `#[fg=themelightgrey]#[acs]~`.
- Indicator (decision 5): unchanged. `redraw_mark_two_pane_colours`
  (`src/screen_redraw.rs:454`) still splits the one separator of a two-pane
  window; with more panes every separator beside the active pane takes its
  style whole, and the last pane's trailing border, which has one neighbour,
  follows the same rule.
- Pane status rows (decision 4): the `pane-border-status` and
  `pane-border-format` options (the table has 256 entries), the status span
  and `REDRAW_PANE_STATUS`, `make_status`, `redraw_get_status_border_cell_type`,
  the pane's status screen and status-line ranges, `PANE_NEWSTATUS`,
  `WindowPane::border_status`/`status_range` and
  `Window::pane_border_status`. `apply_layout` no longer takes a row from each
  pane, `#{pane_at_top}`/`#{pane_at_bottom}` and the `{top}`/`{bottom}`
  targets use the full height, and a click on a pane border never selects a
  control range (status-line control ranges are unchanged).
- Mouse: a pane owns its rows and the separator column after it
  (`window_get_active_at`, `src/window/mod.rs:595`); the row below a shorter
  window is empty area, not a border. `mouse_location_in`
  (`src/window_pane/mouse.rs:57`) reports anything outside the pane's
  interior and scrollbar as its border.
- `display-panes`: the preview draws one vertical line after every pane but
  the last (`window_panes_draw_borders`, `src/window_panes.rs:341`).
- Tests: the PTY screen model in `tests/common/mod.rs` keeps each cell's
  foreground colour and maps DEC line drawing to UTF-8, so ACS and UTF-8
  clients compare alike. `tests/borders.rs` covers the lone half pane's
  border and fill, the even and odd widths, a shorter window and the panned
  blank extent with no horizontal border, and the indicator with three panes
  and with two. `tests/window_sizes.rs` checks the status options are gone.

### Overlays removed (decision 7)

Nothing is drawn over the strip any more; menus and popups went with the client
overlay mechanism that only popups used.

- Commands: `display-menu` and `display-popup`; the command table has 86
  entries. Files: `src/menu.rs`, `src/popup.rs`,
  `src/cmd/entries/display_menu.rs`, `src/shared/menu.rs`,
  `src/shared/popup.rs`, `src/server_client/overlay.rs` and
  `tests/screen_write_menu.rs`.
- Window: the `menu` and last menu position fields with `menu_observer`,
  `take_menu`, `replace_menu`, `place_menu` and `last_menu_position`.
- Client: the overlay and its generation, the overlay methods of the `Client`
  trait (`set_overlay` to `prepare_overlay_render`, including
  `clips_terminal_output`), the overlay callback types, `CLIENT_REDRAWOVERLAY`
  and `CLIENT_REDRAWMENU`.
- Drawing: menu spans and `REDRAW_MENU`/`REDRAW_OVERLAY` in the redraw scene,
  overlay clipping (`tty_check_overlay_range`) in terminal output,
  `TTY_CTX_OVERLAY_SYNC`, `screen_write_menu`, `visible_ranges::exclude_box`,
  the popup palette sources and `job_resize`.
- Tree modes: the right-click menus of the client, buffer, customize and tree
  choosers, and `mode_tree`'s `menu`/`menucb`. A right click in a chooser does
  nothing.
- Options: `menu-style`, `menu-selected-style`, `menu-border-style`,
  `menu-border-lines`, `popup-style`, `popup-border-style` and
  `popup-border-lines`; the table has 258 entries. The default
  `pane-border-format` lost its `[x]` close control, whose binding was a menu.
- Bindings: `<`, `>`, `MouseDown1Control9`, the status-line and empty-area
  right-click menus and `M-MouseDown3Pane`. `MouseDown3Pane` keeps only
  `select-pane -t=; send -M`.
- Tests: the popup lifecycle test in `tests/mode_prompt_cleanup.rs` and the
  viewport menu test went; `tests/window_sizes.rs` checks the commands, options
  and bindings are gone.

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
- Kept in step 3: `select-pane -U/-D` and the `{up-of}`/`{down-of}` targets,
  which reach no pane in one row. Step 5 removed them (see "Step 5").

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
| Strip width | `strip_width` (`src/window/model.rs:51`) | Added in step 3; written only by the arrange step |
| Active pane, pane history | `active`, `last_panes` | Existing |
| Size | `sx`, `sy`, pending and manual sizes | Existing |
| Extent | `extent` (`src/window/model.rs:54`) | Added in step 5; written only by the arrange step |
| Width preference | `width` in the pane model (`src/window_pane/model.rs:19`) | Added in step 4; half by default, written only by the width toggle |
| Pane rectangle | Pane `sx`, `sy`, `xoff`, `yoff` (`src/window_pane/model.rs:13-16`) | Existing; written only by the arrange step |
| Removed in steps 1 and 2 | `z_index`, `modal`, `modal_last`, `last_new_pane_x/y`, `saved_layout_root`, `was_zoomed`, the `WINDOW_ZOOMED` flag; the pane's `saved_layout_cell` and `PANE_ZOOMED` flag | Done |
| Removed in step 3 | `layout_root`, `old_layout`, `lastlayout`, `sticky`; the pane's `layout_cell` | Done |

### Arrange

Done in steps 3 to 5. `window_arrange` (`src/window/mod.rs:798`) computes every
pane's rectangle from the pane order, the panes' width preferences,
`Window::size()` and the pane scrollbar options (`src/window/strip.rs`), applies
them through `apply_layout` (`src/window_pane/api.rs:800`), records the strip
width and the extent, invalidates the scene on movement and updates the offsets
of the clients showing the window. Every Window operation that changes an input
ends in it; see "Step 3" and "Step 4" for the operations and their
notifications.

### Geometry

With `(W, H) = Window::size()` and the panes in order:

| Quantity | Value |
| --- | --- |
| Half width | `(W - 1) / 2` rounded down, raised to the horizontal minimum (the largest pane minimum, `src/window/mod.rs:754`) |
| Full width | `W`, raised to the same minimum |
| Pane height | `H`, less scrollbar and status adjustments as today |
| Pane first column | Previous pane's first column, plus its width, plus one separator |
| Scrollable extent | Last pane's first column plus `W`, and at least the last pane's right edge; `W` without panes. `Window::logical_size()` (`src/window/api.rs:411`) returns it |

- Two halves and their separator always fit in `W`. An even `W` leaves one spare
  column at the right edge.
- The extent rule makes every pane boundary reachable and gives the last pane's
  right border room to be drawn. A lone pane has an extent of exactly `W`.
- Removing a pane closes the gap without enlarging its neighbors.
- Refuse an insertion or a widening that would take the extent past
  `WINDOW_MAXIMUM` (10000); resizes, narrowing and removals cannot fail.

### Viewport

Done in step 5; see "Step 5" for the details.

- `tty_window_offset1` (`src/tty.rs:1077`) follows the active pane's bounds:
  it rests on the offset nearest the previous one that is a pane's first column
  and shows the active pane completely. Cursor movement inside a visible pane
  does not move the view.
- A client narrower than the active pane cannot show it fully; it keeps the
  cursor visible instead. Pane geometry stays shared and follows `window-size`.
- Explicit panning stays in `Client` (`reset_pan`, `apply_pan`, `pan_window`,
  `src/server_client/api.rs:476-528`) and may leave the view off a boundary.
  Selecting a pane resets it; so does `refresh-client -c`.
- Left/right navigation is the previous/next pane in order and stops at the
  ends. Up/down navigation is removed (decision 1).
- Menus and popups are removed (decision 7).

### Borders

Done in step 6; see "Step 6" for the details.

- `redraw_mark_pane_borders` (`src/screen_redraw.rs:389`) keeps the marking
  rule: left when there is a column before the pane, right when it is within
  the extent. Separators are as tall as the panes; there are no horizontal
  borders or junctions. The floating border and clipping code went in step 1.
- Everything else is the outside fill (decision 6).
- Active-pane indicator: tmux's rule (decision 5).
  `redraw_check_two_pane_colours` (`src/screen_redraw.rs:230`) splits the
  separator of a two-pane window; otherwise the separators beside the active
  pane take its style whole.
- No pane status rows (decision 4).
- The `display-panes` preview (`src/window_panes.rs:341`) draws one vertical
  line after every pane but the last.

### Commands

"Confirmed" rows follow the confirmed behavior; decisions 1 to 3 settled the
rest. "Done" marks rows step 3 completed.

| Command | In a strip window | Status |
| --- | --- | --- |
| `new-pane` | Inserts a half-width pane after the target; `-b` before it. `-c`, `-e`, `-E` and `-d` as today. Floating and modal flags (`-L`, `-M`, `-O`, `-x`, `-y`, `-X`, `-Y`) and the split flags (`-f`, `-h`, `-l`, `-p`, `-v`) are removed | Done |
| `kill-pane` | Unchanged; the strip closes the gap | Done |
| `select-pane -L` / `-R`, `{left-of}` / `{right-of}` | Previous/next pane, stopping at the ends | Done |
| `select-pane -U` / `-D`, `{up-of}` / `{down-of}` | Removed; one row has no up or down | Done (decision 1) |
| `resize-pane -W` | New. Toggles the target between half and full width; a widening past `WINDOW_MAXIMUM` fails | Done |
| `swap-pane -U` / `-D`, `-s`/`-t` | Reorder the pane list; the width preference travels with the pane; `-U`/`-D` stop at the ends | Done |
| `refresh-client -L` / `-R` / `-c` | Manual pan and reset, unchanged; selecting a pane also resets the pan | Done |
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

- The prefix default is `C-b` at `src/options_table.rs:1308`.
- Uppercase `H` and `L` are bound as literal characters, distinct from lowercase.
- Remove the bindings for split and break features: splits (`"`, `%`) and `!`.
  Step 3 already removed the layout, tiled resize and border-drag bindings, and
  steps 1 and 2 the floating and zoom ones; see "Removed in step 3".
- `Tab` and `BTab` already open their choosers in a temporary pane with
  `new-pane -E` (`src/key_bindings.rs:252-253`, step 1).
- The defaults array has a hardcoded length of 249 (`src/key_bindings.rs:201`);
  step 5 removed the `Up`/`Down` pane selection bindings and decision 7 the
  menu bindings.

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
4. **Width preference.** Done. `PaneWidth` on the pane, the strip geometry over
   it, `Window::toggle_pane_width` and `resize-pane -W`. Reordering and moving
   between windows carry the preference, and `join-pane` checks room for the
   moving pane's width. `tests/window_sizes.rs` covers the toggle, mixed widths,
   resizing, movement and the maximum. See "Step 4".
5. **Viewport.** Done. Pane-boundary following, the extent rule, pan reset on
   selection, navigation that stops at the ends with up/down removed, and
   positional targets measured across the panes. `tests/common/mod.rs` has a PTY client and `tests/viewport.rs`
   covers the viewport. See "Step 5".
6. **Borders.** Done. Vertical separators only, the outside fill for whatever
   no pane covers, tmux's active-pane indicator, and no pane status rows
   (decisions 4 to 6). `tests/borders.rs` covers the rendering. See "Step 6".
7. **Defaults and keys.** `C-a` prefix, the key map, removal of the split and
   break bindings, accurate help text. Fix tests that assume the old defaults;
   `h1` touched `client_file_protocol`, `copy_regex_cells`,
   `mode_prompt_cleanup`, `plugins` and `session_group_membership`.
8. **Documentation.** README section describing the strip, the removed names
   and the `split-window` alias. `agentmon-tui/src/agentmon/services.py:1274`
   keeps working through the alias (decision 2).

## Validation

`tests/common/mod.rs` provides `Server` (isolated server, `window-size manual`,
event counters), a control client and, since step 5, `TerminalClient`, a PTY
client that models its screen. Viewport and rendering cases use it.

| Area | Required cases |
| --- | --- |
| Geometry | Lone half pane; lone full pane; two halves; mixed widths; odd and even widths; tiny terminals; scrollbars; the extent limit |
| Lifecycle | Insert after focus; insert before target; remove first/middle/last; reorder keeps focus and width preference; move between windows |
| Width | Toggle twice restores the width; focus and other widths unchanged; preference survives terminal resize |
| Resizing | Grow and shrink the terminal; no repeated resize scheduling; silent when nothing moves |
| Focus | Already visible pane; partially visible pane; offscreen pane in each direction; no wrapping at either end; no cursor-driven drift |
| Viewport | The view rests on a pane boundary from both directions on even and odd widths; manual pan and reset; clients of different widths; active pane wider than a client |
| Rendering | Right border of a lone half pane; spare column on even widths; no horizontal border below a shorter window; outside fill beside the last pane; active-pane indicator with two and with three panes; mouse selection after panning |
| Environment | Session environment and `-e` overrides; `-c '#{pane_current_path}'` after `cd`; paths with spaces; directory fallback |
| Defaults and keys | `C-a` prefix; c/h/l/f/H/L/x actions; uppercase/lowercase distinct; prefix passthrough; user overrides |
| Compatibility | `split-window` inserts and ignores its split flags; removed commands, flags and options (including `pane-border-status` and `pane-border-format`) are rejected; `#{window_layout}` is a single-row layout |
| Regression | `tests/model_trait_boundary.rs`; copy mode and choosers; `src/compat/` untouched |

Run the Rust tests after each step, then a build and isolated interactive smoke
checks.

## Known gaps

- A pending resize survives when no eligible client remains. Intentional.
- Session-restore tools that replay `select-layout <string>` fail on the
  removed command; the panes they create still form a strip in order.
- Two-dimensional tiling is gone whatever a command asks for.
- A `swap-pane` between windows does not check room, so carrying a full pane
  into a long strip can take it past `WINDOW_MAXIMUM`, as a window resize can.
- The view keeps the nearest boundary rather than following content. Removing a
  pane before the active one can leave it showing the blank extent past the last
  pane until another pane is selected.
- The two-pane separator split counts the window's panes, not the visible
  ones: a view showing two of three panes colours a separator beside the
  active pane whole (decision 5).
- `job_run` keeps its `JOB_PTY` path (and the `sx`/`sy` arguments every caller
  passes as -1), although no caller asks for a pseudo-terminal since popups went.

## Completion criteria

The confirmed behavior works through normal hmux commands. Pane geometry remains
stable across creation, removal, focus changes and terminal resizing. The Window
has no layout tree, and the tests exercise the strip and its lifecycle.
