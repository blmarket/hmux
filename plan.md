# Scrolling layout implementation plan

Status: reassessed on 2026-10-03 against `main` at `e7c5b0dd`. The two
foundation milestones have landed. The scrolling layout itself has not been
started on `main`. The requirements below are unchanged; the design and sequence
are rewritten on top of what now exists.

## Where things stand

| Piece | Commit | State on `main` |
| --- | --- | --- |
| Window sizing split (`Window::size()` vs. `layout::logical_size`) | `8e6e80db` | Done. `tests/window_sizes.rs`, 6 tests pass |
| Sticky layouts (`sticky-layout` window option) | `4b6e6435` | Done. `tests/sticky_layout.rs`, 20 tests pass |
| Shared server harness for integration tests | `4b6e6435` | Done. `tests/common/mod.rs` |
| Scrolling strip, width toggle, insertion, viewport following, default layout, key map, restoration | none | Not started |

`tests/model_trait_boundary.rs` (18 tests) also passes. The plans for the two
landed pieces (`plan-sizes.md`, `plan-1.md`) were removed after implementation;
their "left for later" items are carried into "Known gaps" below.

### Earlier work that is not on `main`

Use these as reference material only. Neither can be merged.

- **Branch `h1`** (9 commits from `9d746ea8`, worktree `/home/blmarket/h1`)
  implements the whole original plan on the old base. It has its own Window
  `scrolling` flag, `src/layout/scrolling.rs` and a 1954-line
  `tests/scrolling_layout.rs`. It predates both foundations, so its mode flag
  and sizing handling duplicate what sticky layouts and `logical_size` now
  provide. Port from it: pane-based viewport following (`src/tty.rs`,
  `src/server_client/api.rs`), menu placement per client viewport, navigation
  that stops at the strip ends, the key map, the README section and test cases.
- **`stash@{0}`** ("plan-2 scrolling WIP") holds a strip arrangement written
  against `8e6e80db` with the old table contract. Its geometry is reusable. Its
  mode flag, `is_scrolling()` and `layout_set_is_strip` are superseded.
- `review/scrolling-split`, `rewrite/*` and the `backup/*` branches are
  rewrites of `h1`. Six linked worktrees are marked prunable.

## Confirmed behavior

- Scrolling is hmux's default layout for newly created windows, including the
  first window of a new session. Users do not need to run `select-layout scrolling`.
- Arrange panes in one horizontal strip that may extend beyond the visible window.
- Each pane uses the full available height; no vertical stacks within columns.
- Each pane independently uses half or all of the visible width. Mixed widths are allowed.
- New panes start at half-width and appear immediately to the right of the active pane.
- Selecting a pane scrolls only as far as necessary to show that pane completely.
- Adding panes extends the strip instead of squeezing the existing panes.
- Left/right navigation stops at the first/last pane; it does not wrap around.
- A lone half-width pane stays half-width, leaving empty space beside it.
- Use `new-pane` for pane insertion and block existing split commands in scrolling
  mode; adding a pane is a distinct operation from splitting one.
- Width changes use only an explicit half/full toggle, not explicit size setters.
- Reuse the current `new-pane` environment and working-directory capabilities;
  do not add inheritance mechanics or shell integration. Use the existing `-c`
  option when the new pane should follow the source pane's current directory.
- Use `Ctrl+a` as the default prefix, with the key bindings specified below.

## Proposed command behavior

These are implementation proposals, separate from the confirmed requirements above.

- Initialize new windows in scrolling mode. Keep `select-layout scrolling` as an
  explicit way to return to it and include it in layout cycling. Selecting another
  layout exits scrolling mode; the default must not overwrite that explicit choice
  during subsequent resizing, attaching clients, or adding panes.
- Provide an explicit half/full width toggle for the selected pane, `resize-pane
  -W`. `W` is free in the current template (`src/cmd/entries/resize_pane.rs:43`).
  Toggling keeps focus and pane order, preserves all other panes' widths, and
  scrolls only enough to reveal the resized pane. The width preference survives
  terminal resizing. Toggling to full width does not hide other panes or enter
  zoom mode.
- Do not offer explicit width setters for tiled scrolling panes. Half/full are
  persistent width preferences relative to `Window::size()`, not the strip.
- Reject `split-window` (including its alias and horizontal/vertical variants) in
  scrolling windows with a clear error, before changing zoom, layout, or spawning
  a process. Existing split commands keep their behavior in other layouts.
- Reuse the existing `new-pane` command rather than add another creation command.
  Insertion syntax is `new-pane -L`, which already requests a tiled pane
  (`src/cmd/entries/split_window.rs:171`). Insert a full-height pane after the
  target; `-b` inserts before it. Plain `new-pane` retains its floating-pane
  behavior. Do not add a second pane-creation command or an overlapping model API.
- Keep existing pane selection commands and explicit panning through
  `refresh-client -L` / `-R`. Selecting another pane resumes visibility following
  after manual panning; `refresh-client -c` also resumes automatic following.
- Reuse `resize-pane -Z` for zoom. Zoom fills the visible area; unzoom restores the
  strip, pane width preferences, and focus.
- Preserve floating panes as overlays outside the strip's pane order and width count.
- Install the requested default prefix bindings below, replacing conflicting
  built-in entries. Continue allowing user configuration to override these defaults.

Already enforced for every sticky layout, and therefore for scrolling: tiled
`resize-pane -x`, `-y`, `-U`, `-D`, `-L`, `-R` and `select-layout -E` fail with
`layout is sticky` before any unzoom, and dragging a tiled border is ignored.
Floating panes keep their resize behavior.

### Default keys

`Ctrl+a` is a prefix sequence: release it before pressing the following key.
The requested map takes precedence over previous built-in assignments.

| Keys | Action | Command | Today (`src/key_bindings.rs`) |
| --- | --- | --- | --- |
| `Ctrl+a c` | Insert a half-width pane after the active pane, using its current directory | `new-pane -L -c '#{pane_current_path}'` | `new-window` (`:242`) |
| `Ctrl+a h` / `Ctrl+a l` | Focus the pane to the left/right; stop at the strip ends | `select-pane -L` / `select-pane -R` | `h` unbound; `l` is `last-window` (`:246`) |
| `Ctrl+a f` | Toggle the active pane between half-width and full-width | `resize-pane -W` | `find-window` prompt (`:244`) |
| `Ctrl+a H` / `Ctrl+a L` | Reorder the active pane one position left/right | `swap-pane -U` / `swap-pane -D` | `H` unbound; `L` is `switch-client -l` (`:237`) |
| `Ctrl+a x` | Close the active pane | `kill-pane` | `confirm-before … kill-pane` (`:259`) |
| `Ctrl+a Ctrl+a` | Send a literal Ctrl+a to the application | `send-prefix` | `C-b` (`:202`) |

The prefix default is `C-b` at `src/options_table.rs:1389`. The defaults array
has a hardcoded length of 308 (`src/key_bindings.rs:201`); two entries are added.

Uppercase `H` and `L` are distinct from lowercase keys. Bind the literal uppercase
characters; this does not require an extended terminal keyboard protocol.
Reordering keeps focus on the pane being moved and carries its width preference
with it. Reordering stops at the strip ends rather than wrapping.

### Zoom and optional view commands

- **Half/full width (`f`)** changes a pane's persistent width within the strip.
  Neighboring panes remain in the strip, retain their widths, and are reachable
  with normal left/right focus navigation.
- **Zoom (`resize-pane -Z`)** temporarily displays one pane in isolation. Unzoom
  restores the previous arrangement and width preferences. It can look identical
  to a full-width scrolling pane, but it is separate temporary layout state.
- **Manual pan (`refresh-client -L 10` / `-R 10`)** moves the visible portion of
  the strip without changing the active pane.
- **Reset (`refresh-client -c`)** ends manual panning and resumes following the
  active pane in scrolling mode.
- **Previous layout (`select-layout -o`)** restores the arrangement saved before
  the most recent layout selection. It is not a general undo command.

No new bindings are proposed for zoom, manual pan, reset, or previous-layout
restoration.

### Working directory and environment

Unchanged from the earlier assessment (`src/cmd/entries/split_window.rs`,
`src/window_pane/spawning.rs`, `src/server_client/api.rs`, `src/environ.rs`):

- The spawn environment is the global environment, overlaid by the session
  environment and repeated `-e NAME=value` arguments, plus child-specific
  variables such as `TMUX_PANE`, `SHELL` and `PWD`.
- Without `-c`, an unattached command client's directory is used; an attached
  client's key binding normally uses the session directory. Neither follows the
  target pane's `cd`.
- `-c '#{pane_current_path}'` requests the pane's detected current directory.
- Later `export`/`unset` changes inside a pane's shell are not copied.

Keep these semantics for scrolling-pane insertion. Do not add automatic cwd
inheritance, live environment copying, shell hooks, or a new environment
transport as part of this feature.

## What the foundations already provide

| Original plan item | Now |
| --- | --- |
| Store a sizing basis separate from the strip extent | Not needed. `Window::size()` is the basis and is never inflated by layout. `layout::logical_size` (`src/layout/core.rs:740`) is the extent, computed on demand |
| Avoid repeated resize scheduling when the layout is wider than the terminal | Done. `recalculate_size` compares the policy result with `Window::size()` (`src/resize.rs:365`) |
| Clip, pan and bound scenes across the wider layout | Done. Clients, redraw, pane clipping, navigation edges and menus read `logical_size` |
| Zoom to the visible area, not the strip | Done. Zoom arranges against `Window::size()` and never changes it |
| A scrolling mode flag on the Window | Not needed. `sticky` plus `lastlayout` (`src/window/model.rs:57-60`), read through `Window::sticky_layout()`. `Window::select_layout` is the only writer |
| Regenerate the layout on resize, pane add, pane remove, float and tile | Done for any sticky preset by `layout_set_arrange_sticky` (`src/layout/set.rs:133`), called from `layout_resize`, `layout_assign_pane`, `layout_close_pane`, `layout_float_pane` and `layout_tile_pane` |
| Regenerate on option changes; stay silent when nothing moves | Done. `refit_layout` (`src/resize.rs:457`) acts on the movement flag that `layout_resize` returns |
| Replace hardcoded preset counts | Done. `layout_sets.len()` |
| Preset contract usable from the engine | Done. `arrange(&WindowRef, (sx, sy)) -> bool` rebuilds the tree only; `layout_set_select` owns pane fixing, the event and the redraw |
| Reject manual tiled geometry | Done for sticky layouts, see above |

## Remaining design

Follow `AGENTS.md` and `docs/trait-models.md`: do not edit `src/compat/`, add
shared ownership, bypass the model traits, or replace explicit release
operations with incidental `Drop`. End layout borrows before resizing panes or
dispatching callbacks. Scrolling must extend the sticky machinery, not sit
beside it: no second mode flag, no second set of triggers.

### The preset

- Add `scrolling` as the eighth `layout_sets` entry (`src/layout/set.rs:34`).
- Add a per-entry property: always in force. Selecting `scrolling` makes the
  layout sticky regardless of `sticky-layout`. The other seven keep reading the
  option at selection (`src/window/api.rs:1093`).
- The strip function arranges for a given `(sx, sy)` like the other presets, with
  two differences: it arranges a lone tiled pane too, so it never declines, and
  its root may be wider than `sx`.
- `layout_set_cells` (`src/layout/set.rs:155`) already yields placed panes in pane
  order with their floating flag. Floating leaves are relinked under the root
  unchanged, as the other presets do.

### Geometry

With `(W, H) = Window::size()` and the tiled panes in pane order:

| Quantity | Value |
| --- | --- |
| Half width | `(W - 1) / 2` rounded down, raised to the horizontal minimum from `layout_resize_limits` |
| Full width | `W`, raised to the same minimum |
| Pane height | `H`; pane status and scrollbars are handled by `layout_fix_panes` |
| Strip (root) width | Sum of pane widths plus one separator between neighbors |
| Logical size | Existing rule: the larger of `Window::size()` and the root |

- Two halves and their separator always fit in `W`. An even `W` leaves one spare
  column, given to neither pane.
- A lone half-width pane leaves a root narrower than the window; the logical
  size is then `Window::size()` and the rest of the view is empty.
- Removing a pane closes the gap without enlarging its neighbors.
- Arithmetic must be overflow-safe. `WINDOW_MAXIMUM` and `PANE_MAXIMUM` are both
  10000, and layout import bounds widths and offsets by them
  (`src/layout/custom.rs:715-760`). Refuse an insertion that would take the strip
  past what a layout string can describe; presets and resizes cannot fail.

### Width preference

Store half/full as one value in the pane model (`src/window_pane/model.rs`).
The layout tree and pane order stay authoritative for structure; geometry is
derived from the preferences and `Window::size()`, never the reverse. The
preference persists while another layout is selected and through zoom.

### Triggers still missing

| Change | Why the existing triggers do not cover it |
| --- | --- |
| Width toggle | New input to the arrangement. Arrange, fix panes, notify on movement |
| `swap-pane`, `rotate-window` | They exchange cells and geometry directly (`src/cmd/entries/swap_pane.rs:184-211`, `src/cmd/entries/rotate_window.rs:83-104`). That keeps the seven presets valid, but in a strip the width preference travels with the pane, so mixed widths need arranging afterwards |
| A free layout whose tiled root is narrower than the window | `refit_layout` returns early when `logical_size` equals `Window::size()`, which is also true for a narrow root. Leaving scrolling with one pane keeps it half-width, because the seven presets decline with one pane. Refit a free layout when the tiled root differs from `Window::size()` |

### Insertion and split rejection

`new-pane -L` and `split-window` share one execution path
(`src/cmd/entries/split_window.rs:170`), which reserves a cell by splitting the
target and requires the target to be splittable
(`layout_split_check_space_with_limits`, `src/layout/core.rs:1639`). In a
sticky layout the result is rearranged, so a split already behaves as an
insertion after the target. For scrolling:

- Reject `split-window` before unzoom, layout change or spawn.
- `new-pane -L` reserves the new cell next to the target without taking space
  from it and without a space check; the arrangement gives it its width.
  Reject split-geometry arguments on it.
- Enforce the no-split rule in the shared layout operation as well as in the
  command, so `join-pane` and `move-pane` cannot stack panes in a column.

### Viewport

- `tty_window_offset1` (`src/tty.rs:1055`) follows the active pane's cursor and
  centers on it. For a scrolling window, follow the active pane's bounds: keep
  the offset when the pane, including its border, is fully visible; otherwise
  move to the nearest offset that reveals it and clamp to the logical size.
  Cursor movement inside a visible pane must not move the view.
- A client narrower than the pane cannot show it fully; fall back to keeping the
  cursor visible. Pane geometry stays shared and follows `window-size`.
- Explicit panning lives in `Client` (`apply_pan`, `pan_window`, `reset_pan`,
  `src/server_client/api.rs:506-545`). Selecting a pane resets it.
- Left/right navigation wraps at the edges using the logical width
  (`window_pane_find_left`/`_right`, `src/window_pane/mod.rs:1904`, `:1949`).
  In a scrolling window it must stop.
- Menus and popups are placed against the logical size
  (`src/cmd/entries/display_menu.rs:522`, `src/window/mod.rs:531`). Place them
  within the target client's viewport.

### Default and restoration

- New windows start with `lastlayout = -1` and not sticky
  (`src/window/mod.rs:384`). Window creation must leave them in `scrolling`,
  through the same writer as selection. `break-pane` windows follow the same rule.
- `select-layout -o` saves and restores a layout string, so it restores geometry
  and leaves the layout free. Keep the preset and its in-force state with the
  previous-layout value so `-o` restores behavior.
- Extend the JSON layout (`layout_append_v2`, `src/layout/custom.rs:157`) with
  optional scrolling metadata and a per-pane width preference. Validate before
  mutating. Existing layouts keep loading. The legacy string stays a geometry
  snapshot.

## Implementation sequence

Each step ends with passing tests and leaves existing layouts unchanged.

1. **Selectable strip.** Add the `scrolling` entry, the always-in-force property
   and the strip function (every pane half-width). Fix the free-layout refit
   rule. Covers: selection, cycling, resize, pane add and remove through the
   existing triggers, lone pane, zoom, floating panes.
2. **Width preference.** Add the pane value, `resize-pane -W`, and arranging
   after swap and rotate. Covers: mixed widths, toggle twice, reorder carries
   the preference, preference survives leaving and re-entering scrolling.
3. **Insertion.** `new-pane -L` and `-b` without splitting; reject
   `split-window`; enforce no stacking for join and move; refuse an over-long
   strip.
4. **Viewport.** Pane-based following, pan reset on selection, navigation that
   stops at the ends, menu and popup placement, mouse coordinates and borders at
   viewport edges.
5. **Default and keys.** New windows start in scrolling; `C-a` prefix; the key
   map above with accurate help text. Fix tests that assume the old defaults;
   `h1` touched `client_file_protocol`, `copy_regex_cells`,
   `mode_prompt_cleanup`, `plugins` and `session_group_membership`.
6. **Restoration and documentation.** Previous-layout policy, JSON metadata,
   README section for scrolling and for `sticky-layout`.

## Validation

`tests/common/mod.rs` provides `Server` (isolated server, `window-size manual`,
event counters) and a control client. It has no terminal client; viewport and
rendering cases need a PTY client added to it, as `h1`'s test file had.

| Area | Required cases | Coverage today |
| --- | --- | --- |
| Defaults and keys | New sessions/windows start scrolling with a half-width pane; explicit alternative layouts persist; `C-a` prefix; c/h/l/f/H/L/x actions; uppercase/lowercase distinct; prefix passthrough; user overrides | None |
| Geometry | Lone half-width pane; lone full-width pane; two halves; mixed widths; odd widths; tiny terminals; pane borders/status/scrollbars; size limits | None |
| Lifecycle | Insert after focus; insert before target; remove first/middle/last; join/move/break; reorder keeps focus and width preference; swap/rotate | Sticky presets only |
| Commands | Split commands and aliases rejected before side effects; tiled insertion distinct from floating creation; unsupported resize forms rejected | Resize rejection, for sticky presets |
| Environment | Session environment and `-e` overrides; `-c '#{pane_current_path}'` after `cd`; paths with spaces; directory fallback; actual cwd matches `PWD` | None for `new-pane -L` |
| Resizing | Toggle twice restores preference; focus and other widths unchanged; grow/shrink terminal; no repeated resize scheduling | Resize and silent recalculation, for sticky presets and narrow windows |
| Focus | Already visible pane; partially visible pane; offscreen pane in each direction; no wrapping at either end; no cursor-driven drift | None |
| Viewport | Manual pan and reset; focus after panning; clients of different widths; active pane wider than a client | None |
| Rendering | Borders and cursor at viewport edges; mouse selection after panning; partially visible neighbors; offscreen pane output | None |
| Transitions | Enter/leave scrolling; layout cycling; previous-layout restore; JSON round trip; legacy geometry output; zoom/unzoom; resize while zoomed | Zoom and cycling, for sticky presets |
| Regression | Existing layouts, ordinary splits/resizes, floating panes, explicit panning | `window_sizes`, `sticky_layout`, existing suite |

Run the Rust tests and `tests/model_trait_boundary.rs` after each step, then a
build and isolated interactive smoke checks. Confirm `src/compat/` is untouched.

## Known gaps

Carried from the landed milestones; none blocks step 1.

- In a sticky layout `split-window` still needs a splittable target, so a full
  `tiled` window can fail with "no space for new pane" although the arranged
  grid would have room. Step 3 removes this for scrolling only.
- An option change made while zoomed is applied at the first recalculation after
  unzoom, not at the unzoom itself.
- `select-layout -o` and layout strings do not record stickiness. Step 6.
- `sticky-layout` is not documented outside its option text. Step 6.
- A pending resize survives when no eligible client remains, and layout strings
  report the layout root rather than `Window::size()`. Both are intentional.

## Completion criteria

The confirmed behavior works through normal hmux commands. Pane geometry remains
stable across creation, removal, focus changes, terminal resizing, and zoom.
Existing layouts retain their behavior, and the tests exercise both scrolling
behavior and its integration with the existing lifecycle.
