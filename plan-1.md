# Milestone 1: sticky layouts

Status: proposed. Base is `main` at `8e6e80db`. This is a prerequisite placed
before step 1 of "Implementation sequence" in `plan.md`. `plan-2.md` (the
scrolling layout, formerly `plan-1.md`) builds on it and needs the revisions listed
at the end. The working tree holds an implementation of that former plan; the
"Sequence" section says which parts carry over.

## Outcome

A window's layout is either free or sticky.

- **Free** is today's behavior: the tree is authoritative. A preset arranges it
  once, and later changes edit the tree in place.
- **Sticky** means a preset is in force: the tree is derived from the tiled pane
  order, the window size and the options the preset reads. The layout engine
  derives it again whenever one of those changes.

With the new window option `sticky-layout` on, selecting any of the seven presets
makes the layout sticky. Panes stay arranged when one is added or removed, when
the window is resized, and when an option such as `main-pane-width` changes. The
option is off by default, so nothing changes for a user who does not set it.

Not in this milestone: the scrolling layout, a preset that is sticky regardless of
the option, and restoring stickiness with `select-layout -o`.

## State

Add one value to the Window model, next to `lastlayout`
(`src/window/model.rs:57`): whether the preset recorded there is in force. It is
false for a new window. The trait gains one reader, `sticky_layout()`, which
returns the preset index while it is in force.

`Window::select_layout` (`src/window/api.rs:1069`) is the only writer.

| Selection | Layout afterwards |
| --- | --- |
| A preset by name, by cycling, or by a bare `select-layout` | sticky if `sticky-layout` is on for the window at that moment, else free |
| A layout string, including `select-layout -o` | free; restored if the parse fails, next to `old_layout` (`src/window/api.rs:1118`) |
| Spread (`select-layout -E`) | rejected while sticky, see "Manual geometry" |

- The option is read only at selection. Turning it on later does not capture a
  preset selected earlier, and turning it off does not free a sticky window; the
  next selection decides again. This keeps one writer and avoids tracking whether
  a free tree still matches its preset.
- Clear the value before `layout_parse` installs a tree. Import refits through
  `layout_resize` (`src/layout/custom.rs:449`) and must take the ordinary path.
- Write the value before `recalculate_sizes()` at `src/window/api.rs:1125`.
- A window created by `new-window` or `break-pane` starts free.

`sticky-layout` is a window flag option, default off, defined like
`aggressive-resize` (`src/options_table.rs:1976`).

## Arranging for a size

The preset functions cannot be run again as they are. Each reads the window size
itself (`src/layout/set.rs:198`, `:312`, `:497`, `:682`, `:867`, `:1052`), and each
ends by firing `window-layout-changed` and redrawing. `resize_window` passes the
new size to `layout_resize` before storing it (`src/window/api.rs:1241`), so a
preset run from there would read the old size.

Change the table entry (`src/layout/set.rs:31`) to one function with this contract:

- It takes the window and the target `(sx, sy)`.
- It rebuilds the tree and fixes offsets. It does not resize panes, fire events or
  redraw.
- It returns false when it leaves the tree alone. The seven presets do so with
  fewer than two tiled panes; the caller then takes the ordinary path.
- It considers only panes that have a layout cell, in pane order. A pane whose
  cell was just closed is still in the pane list (`src/window_pane/lifecycle.rs:149`)
  and must be skipped. Use one shared helper instead of `pane_snapshot()` plus
  `expect("pane cell")`.

The common tail (apply to panes, fire the event, redraw) moves out of the seven
functions into `layout_set_select` (`src/layout/set.rs:93`). `layout_set_next` and
`layout_set_previous` compute an index and call it, instead of repeating the call.

One engine function sits on top: arrange the sticky preset, if any, for a size.
It reports whether it arranged. It is silent; each caller keeps its own
notifications.

Movement is not judged by comparing trees. `layout_fix_panes`
(`src/layout/core.rs:389`) already knows whether any pane changed; make it return
that, and make `layout_resize` return it too.

This relies on every preset being a function of pane order, floating flags, size
and options only, so that arranging twice gives the same tree. That holds for
`layout_set_even` and `layout_set_main_h` as read; confirm it for the others while
converting them. The "repeated recalculation" case below checks it.

## Triggers

| Input that changed | Entry point | Change |
| --- | --- | --- |
| Window size | `layout_resize` (`src/layout/core.rs:862`) | While sticky, arrange for `(sx, sy)` instead of adjusting proportionally; fall through when the preset declines |
| Options and pane minimums | `refit_layout` (`src/resize.rs:456`), reached from `recalculate_sizes()` after every option change (`src/options.rs:1748`) | While sticky, refit unless the window is zoomed. Redraw and notify only on movement |
| Tiled pane added | `layout_assign_pane` (`src/layout/core.rs:1362`) | Arrange after the leaf is made, before panes are fixed; keep the `do_not_resize` skip |
| Tiled pane removed | `layout_close_pane` (`src/layout/core.rs:1937`) | Arrange after the cell is destroyed, before panes are fixed |
| Pane floated or tiled | `layout_float_pane` (`:2568`), `layout_tile_pane` (`:2636`) | Arrange after the tree edit; the callers already fix panes |

- Pane changes must not wait for a recalculation. A command sent by `hmux` from a
  shell is followed by one when its client exits (`src/server_client/mod.rs:1299`),
  but a key binding is not. Hence the triggers inside the engine functions, which
  every caller already goes through: split, join, kill, pane exit, break, and the
  rollback of a failed spawn.
- Free layouts keep the current `refit_layout` test (logical size equals window
  size). Both modes judge movement by the flag `layout_resize` returns.
- The tree is arranged only while unzoomed. `resize_window` and the split, join,
  kill, exit and break callers already unzoom first; confirm it for the float
  and tile paths. `refit_layout` skips a zoomed window.
- Nothing arranges between `layout_split_pane` reserving a cell and
  `layout_assign_pane` filling it; the reserved cell has no pane yet.
- `swap-pane` and `rotate-window` exchange cells and pane order together, so the
  arrangement stays valid and they need no trigger.
- End tree borrows before `layout_fix_panes`, as `plan.md` requires.

## Manual geometry

A manual size would be undone by the next arrangement, so it is refused while the
layout is sticky. Errors leave the layout, zoom and focus unchanged.

| Operation | While sticky |
| --- | --- |
| `resize-pane -x`, `-y`, `-U`, `-D`, `-L`, `-R` on a tiled pane | Error `layout is sticky`, checked before `server_unzoom_window` (`src/cmd/entries/resize_pane.rs:117`) |
| Border drag on a tiled pane (`cmd_resize_pane_mouse_resize_tiled`, `:581`) | Ignored |
| `select-layout -E` | Error `layout is sticky`, checked before the unzoom in `select_layout` |
| `resize-pane -Z`, `-T`, and any resize of a floating pane | Unchanged |
| `split-window -l`, `join-pane -l` | Accepted; the size has no lasting effect |

To leave a sticky layout, select a layout string or select a preset with the
option off. Sizes are tuned through the preset's options.

## Sequence

1. In `src/layout/set.rs`, replace the seven hardcoded table lengths with the
   table's length. No behavior change. Carries over from the working tree.
2. Convert the seven presets to the new table contract and move the common tail
   into `layout_set_select`. No behavior change.
3. Make `layout_fix_panes` and `layout_resize` return movement and use it in
   `refit_layout`. No behavior change. Carries over from the working tree.
4. Add `sticky-layout`, the Window value, its writes in `select_layout`, and
   `sticky_layout()`.
5. Add the arranging function and use it in `layout_resize` and `refit_layout`.
6. Add the four pane triggers.
7. Refuse manual geometry.
8. Move the `Server` harness out of `tests/window_sizes.rs` into a shared module
   (carries over as `tests/common/`) and add `tests/sticky_layout.rs`.

From the working tree, the `scrolling` flag, `is_scrolling()`, the strip code and
`tests/scrolling_layout.rs` belong to `plan-2.md` and need reworking there.

## Validation

Sizes use `window-size manual`, as `tests/window_sizes.rs` does. Widths below are
for an 83-column window, where two, three and four even panes divide exactly.
"Matches the preset" means reselecting the preset leaves `#{window_layout}`
unchanged.

| Case | Expected |
| --- | --- |
| Option off: three panes, `select-layout even-horizontal`, `kill-pane` | A neighbor absorbs the space, as today |
| Option on: three panes, `select-layout even-horizontal` | Widths 27, 27, 27 |
| then `kill-pane` | Widths 41, 41 |
| then `split-window` twice | Four panes of width 20 |
| `split-window \; display -p '#{pane_width}'` as one command | Prints the arranged width |
| `resize-window -x 62` with three panes | Widths 20; one `window-resized` |
| Repeated recalculation, each of the seven presets | No `window-layout-changed`, no `window-resized` |
| Add and remove panes under `tiled` and `main-vertical` | Matches the preset after each step |
| Pane exits on its own under `tiled` | Matches the preset |
| `set main-pane-width` under `main-vertical` | The main pane takes the new width without reselecting |
| `swap-pane`, `rotate-window` | Matches the preset; no extra `window-layout-changed` |
| `join-pane` between two sticky windows | Both match their presets |
| `break-pane` | Source matches the preset; the new window is free |
| A floating pane present; float and tile a pane | Floating panes are not counted; the tiled ones match the preset |
| `resize-pane -x 10`, `resize-pane -L`, `select-layout -E` | Error, layout and zoom unchanged |
| `resize-pane -Z`; resize of a floating pane | Work as before |
| Select a layout string, then `kill-pane` | Free: a neighbor absorbs the space |
| Layout string that fails to parse | Error; still sticky |
| Option turned off after selecting, then `kill-pane` | Still sticky |
| Option turned on after selecting, then `kill-pane` | Still free |
| `next-layout` with the option on | Each preset in turn is sticky |
| Zoom, `resize-window`, unzoom | The arrangement returns at the new size |
| Down to one tiled pane, then `resize-window` | The pane fills the window |
| Split whose spawn fails | The original arrangement returns |

Run the existing suite, including `tests/window_sizes.rs` and
`tests/model_trait_boundary.rs`, unchanged apart from the harness move.

## Left for later

- `split-window` still needs its target to be splittable. In a full `tiled`
  window it can fail with "no space for new pane" although the arranged grid
  would have room.
- An option change made while zoomed is applied at the first recalculation after
  unzoom, not at the unzoom itself.
- `select-layout -o` restores geometry, not stickiness. `plan.md` step 5 keeps the
  policy with the previous-layout state.
- Layout strings do not record stickiness, so a saved and restored layout is free.

## Revisions needed in plan-2.md

`plan-2.md` was moved unchanged. On top of this milestone it changes as follows:

- It becomes milestone 2, with this plan as its base.
- "Mode" goes away. There is no `scrolling` value and no `is_scrolling()`. The
  `scrolling` table entry is marked always sticky, regardless of `sticky-layout`;
  that per-entry property is added there.
- "Keeping the strip" shrinks to the strip function itself. It arranges a single
  pane too (half width), so it never declines. `layout_set_is_strip` is not
  needed, because movement comes from `layout_fix_panes`.
- The ordinary-mode `refit_layout` rule (refit when the tiled root differs from
  the window size) stays there; only a lone half-width pane needs it.
- "Other operations" changes: `kill-pane` closes the gap through the trigger, not
  through a recalculation; `split-window` inserts after the target until `plan.md`
  milestone 2 blocks it; tiled `resize-pane` is already refused.
- Sequence steps 1 and 6 (table lengths, harness move) are done here.
