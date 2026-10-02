# Scrolling layout implementation plan

Status: requirements and implementation plan only. No implementation changes yet.

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
- Provide an explicit half/full width toggle for the selected pane. Toggling keeps
  focus and pane order, preserves all other panes' widths, and scrolls only enough
  to reveal the resized pane. The width preference survives terminal resizing.
  Toggling to full width does not hide other panes or enter zoom mode.
- Do not offer explicit width setters for tiled scrolling panes, including
  `resize-pane -x 50%` and `resize-pane -x 100%`. Half/full are persistent width
  preferences relative to the visible sizing basis, not the entire strip.
- Reject `split-window` (including its alias and horizontal/vertical variants) in
  scrolling windows with a clear error, before changing zoom, layout, or spawning
  a process. Existing split commands keep their behavior in other layouts.
- Reuse the existing `new-pane` command rather than add another creation command.
  Proposed insertion syntax is `new-pane -L`, which already requests a tiled pane.
  Insert a full-height pane after the target; `-b` explicitly inserts before it.
  Plain `new-pane` retains its current floating-pane behavior. Do not add a second
  pane-creation command or an overlapping model API.
- Keep existing pane selection commands and explicit panning through
  `refresh-client -L` / `-R`. Selecting another pane resumes visibility following
  after manual panning; `refresh-client -c` also resumes automatic following.
- Reuse `resize-pane -Z` for zoom. Zoom fills the visible area; unzoom restores the
  strip, pane width preferences, and focus.
- Preserve floating panes as overlays outside the strip's pane order and width count.
- Install the requested default prefix bindings below, replacing conflicting
  built-in entries. Continue allowing user configuration to override these defaults.

For tiled panes in scrolling mode, reject ordinary incremental resizing, height
changes, and border dragging rather than reinterpret them as width toggles. Errors
must leave layout and focus unchanged. Floating panes retain their existing resize
behavior. Unsupported explicit widths should also fail clearly.

### Default keys

`Ctrl+a` is a prefix sequence: release it before pressing the following key.
The requested map takes precedence over previous built-in assignments.

| Keys | Action | Command |
| --- | --- | --- |
| `Ctrl+a c` | Insert a half-width pane after the active pane, using its current directory | `new-pane -L -c '#{pane_current_path}'` |
| `Ctrl+a h` / `Ctrl+a l` | Focus the pane to the left/right; stop at the strip ends | `select-pane -L` / `select-pane -R` |
| `Ctrl+a f` | Toggle the active pane between half-width and full-width | `resize-pane -W` (proposed new flag) |
| `Ctrl+a H` / `Ctrl+a L` | Reorder the active pane one position left/right | `swap-pane -U` / `swap-pane -D` |
| `Ctrl+a x` | Close the active pane | `kill-pane` |

Uppercase `H` and `L` are supported and distinct from lowercase keys. Bind the
literal uppercase characters; users type them with Shift after releasing the
prefix. This does not require an extended terminal keyboard protocol.

Use the single explicit width-toggle action for `f`; the proposed `-W` flag must
be implemented rather than treated as an existing option. Reordering should keep
focus on the pane being moved and carry its width preference with it. Proposed
edge behavior is to stop rather than wrap, consistent with focus navigation.

Changing the prefix also requires keeping `send-prefix` accessible; the proposed
binding is `Ctrl+a Ctrl+a` to send a literal Ctrl+a to the application.

### Zoom and optional view commands

- **Half/full width (`f`)** changes a pane's persistent width within the strip.
  Neighboring panes remain in the strip, retain their widths, and are reachable
  with normal left/right focus navigation.
- **Zoom (`resize-pane -Z`)** temporarily displays one pane in isolation. Unzoom
  restores the previous arrangement and width preferences. It can look identical
  to a full-width scrolling pane, but it is separate temporary layout state.
- **Manual pan (`refresh-client -L 10` / `-R 10`)** moves the visible portion of
  the strip without changing the active pane. Ordinary focus navigation already
  scrolls automatically, so manual pan is not required for the primary workflow.
- **Reset (`refresh-client -c`)** ends manual panning and resumes following the
  active pane in scrolling mode.
- **Previous layout (`select-layout -o`)** restores the arrangement saved before
  the most recent layout selection, for example after switching to another layout.
  It is not a general undo command.

No new bindings are proposed for zoom, manual pan, reset, or previous-layout
restoration. Keep those commands available and support their existing semantics.

### Working directory and environment

Current behavior, verified in `src/cmd/entries/split_window.rs`,
`src/window_pane/spawning.rs`, `src/server_client/api.rs`, and `src/environ.rs`:

- The spawn environment starts with hmux's global environment, overlaid by the
  session environment and repeated `-e NAME=value` arguments. Spawn code also sets
  child-specific variables such as `TMUX_PANE`, `SHELL`, and `PWD`, and handles PATH.
- Without `-c`, directory selection uses the invoking client's directory when it
  is an unattached command client; an attached client's key binding normally uses
  the session directory. It does not automatically follow the target pane's `cd`.
- `-c '#{pane_current_path}'` requests the pane's detected current directory. The
  Linux implementation reads the foreground process group's directory, with a
  terminal-session fallback.
- Later `export`/`unset` changes inside a pane's shell are not copied automatically.
  The server has no current protocol for receiving that shell's live environment.

Keep these existing spawn semantics for scrolling-pane insertion. To follow the
current pane's directory, use the existing command/format support in the invocation
or key binding:

```sh
hmux new-pane -L -c '#{pane_current_path}'
```

Preserve `-c` format expansion, repeated `-e` overrides, directory fallback behavior,
and the child's generated variables. The existing spawn path changes the actual
working directory and sets `PWD`; setting `PWD` alone would not change directory.
Do not add automatic cwd inheritance, live environment copying, shell hooks, or a
new environment transport as part of this layout feature.

## Existing implementation and constraints

- `src/layout/set.rs` implements seven preset arrangements. These arrange a tree;
  they do not maintain an ongoing layout policy as panes change.
- `src/layout/core.rs` handles splitting, closing, resizing, and pane assignment.
  Its existing tiled operations redistribute space between neighboring panes.
- `src/cmd/entries/split_window.rs` already implements both `split-window` and
  `new-pane`. They share an execution path; `new-pane -L` requests a tiled pane,
  while plain `new-pane` requests a floating pane. The scrolling policy must
  distinguish insertion from splitting before entering shared creation logic.
- `src/window/api.rs` coordinates layout selection and window resizing.
  `src/resize.rs` calculates the requested size from clients and window options.
- `src/tty.rs` already clips oversized windows and stores a viewport per client.
  Its automatic offset currently follows the terminal cursor rather than pane bounds.
- `src/server_client/api.rs` already owns explicit panning behavior.
- `src/key_bindings.rs` owns built-in bindings; `src/options_table.rs` defines the
  default prefix. Both need updating for the requested default key map.
- `src/layout/custom.rs` serializes and restores layouts; previous-layout restoration
  must preserve the new mode as well as its geometry.

Follow `AGENTS.md` and the model boundaries documented in `docs/trait-models.md`:
do not edit `src/compat/`, introduce new shared ownership, bypass the model traits,
or replace established explicit release/free operations with incidental `Drop`.
End layout/component borrows before resizing panes or dispatching callbacks.

## State and geometry

### Separate the sizing basis from the strip extent

Retain the existing window-size policy as the source of the layout's visible sizing
basis. Store scrolling mode and that basis in Window-owned value state. Keep the
existing window geometry as the full canvas extent so clipping, hit testing, and
explicit panning can reuse their current coordinate system.

For multiple clients, pane geometry remains shared and follows the existing
window-size policy. Each client retains its own viewport and scroll position.
If a client is narrower than a pane, full visibility is impossible; clamp its view
within the canvas and keep the active cursor visible without changing shared widths.

Store each pane's half/full preference as a small value in the pane model. Keep the
layout tree and existing pane order authoritative; do not add a parallel collection
of pane owners or reconstruct preferences from rounded terminal dimensions.
Rendered geometry is derived from those preferences and the current sizing basis.

### Arrangement rules

- Use the existing left/right layout tree with full-height tiled leaves.
- Include pane borders, status rows, and scrollbars when calculating sizes.
- Two half-width panes plus their separator must fit in the sizing width. Specify
  deterministic rounding for odd dimensions and honor minimum pane sizes.
- A full-width pane must fit in the sizing width without an off-by-one overflow.
- Calculate canvas width from pane widths and separators, with enough canvas to
  cover the visible area when the strip is shorter than the viewport.
- Keep a lone half-width pane at half-width; do not stretch it to fill that canvas.
- Removing a pane closes the gap without enlarging its neighbors.
- Preserve width preferences through narrow terminal sizes and subsequent expansion.
- Check arithmetic and the applicable pane/window limits before committing changes.

### Focus following

Reuse the current client viewport offset. Leave it unchanged when the selected pane
is fully visible. Otherwise move it to the nearest offset that reveals the pane,
then clamp to the canvas bounds. Account for the pane's displayed border geometry.
Cursor movement within an already visible pane must not make the strip drift.

## Implementation sequence

1. **A working selectable scrolling layout**
   - Register `scrolling` in `src/layout/set.rs`; replace hardcoded preset counts
     with the collection length while updating lookup and cycling.
   - Make `select-layout scrolling` arrange an existing window as full-height,
     half-width panes. Implement its geometry as part of the actual layout, not
     as an unused standalone API or a separate prerequisite milestone.
   - Introduce the Window-owned mode/sizing basis needed to keep the canvas wider
     than the visible window. Reuse existing clipping and panning.
   - Update resize coordination so client-size changes resize the sizing basis and
     regenerate the strip instead of compressing it into the viewport.
   - Compare pending/requested sizes against the sizing basis to avoid repeatedly
     scheduling resizes because the canvas is wider than the terminal.
   - Verify this first milestone through layout selection and terminal resizing.
     Extend existing model operations rather than adding overlapping APIs.

2. **Pane lifecycle and commands**
   - Reject split commands before side effects in scrolling mode. Enforce the
     no-splitting invariant in shared layout operations as well as command dispatch.
   - Adapt existing `new-pane` creation for strip insertion.
     Route assignment, close, permitted width changes, and spread behavior through
     the active layout policy; never let a generic split silently reshape the strip.
   - Implement a single half/full toggle operation; reject explicit sizing and
     unsupported resize forms before changing geometry.
   - Preserve existing `new-pane` spawning semantics, including `-c`/`-e` support;
     use `-c '#{pane_current_path}'` in the documented insertion command/binding.
   - Preserve cell ownership and pane associations during reconstruction.
   - Make failed pane creation restore the original tree, dimensions, and focus.
   - Check join/move, break, swap/rotate, pane exit, and floating/tiled transitions.
     Their final layout must satisfy the strip invariant and respect target order.
   - In scrolling layouts, moving left from the first pane or right from the last
     pane leaves focus and the viewport unchanged.
   - Preserve per-pane half/full preferences through reordering, layout changes,
     and re-entry into scrolling mode.

3. **Viewport and zoom**
   - Add pane-based following to the existing offset calculation in `src/tty.rs`.
   - Integrate explicit-pan reset on focus changes through Client's existing API.
   - Keep mouse coordinates, borders, cursor placement, and redraws consistent with
     the viewport offset, including partially visible neighboring panes.
   - Update zoom/unzoom and resize-while-zoomed behavior to use the visible sizing
     basis rather than zooming to the full canvas width.

4. **Make scrolling the default and install the key map**
   - Initialize scrolling policy on new-window/new-session creation, including a
     lone half-width pane. Do not reset an explicitly selected layout on resize,
     reattachment, zoom restoration, or respawn.
   - Update the default prefix to `C-a` in `src/options_table.rs` and implement the
     requested bindings in `src/key_bindings.rs`, with accurate help descriptions.
   - Bind `h`/`l` separately from `H`/`L`; wire `f` to the implemented width toggle
     and `c` to existing `new-pane -L -c '#{pane_current_path}'` spawning.
   - Replace conflicting built-in entries and update prefix passthrough. Preserve
     the usual ability to customize options and bindings through configuration.

5. **Restoration and documentation**
   - Preserve mode and width preferences with previous-layout state so
     `select-layout -o` restores behavior, not just a snapshot of pane positions.
   - Extend the existing JSON layout representation with optional scrolling metadata
     and validate it before mutation. Continue accepting existing layouts.
   - Keep legacy layout output usable as a geometry snapshot; document that its
     format cannot describe scrolling behavior. Internal previous-layout restoration
     must retain policy independently when legacy output is selected.
   - Document activation, width toggling, pane insertion, existing `-c`/`-e` usage,
     blocked split and resize forms, navigation, and manual panning.

## Validation

Use focused geometry tests, model integration tests, and an isolated hmux server
with PTY clients for rendering checks. Preserve explicit lifecycle teardown and the
repository's handling of tests that use process-global server state.

| Area | Required cases |
| --- | --- |
| Defaults and keys | New sessions/windows start scrolling with a half-width pane; explicit alternative layouts persist; `C-a` prefix; requested c/h/l/f/H/L/x actions; uppercase/lowercase distinct; prefix passthrough; user overrides |
| Geometry | Lone half-width pane with empty space; lone full-width pane; two halves; mixed half/full widths; odd widths; tiny terminals; pane borders/status/scrollbars; size limits |
| Lifecycle | Insert after focus; insert before target; remove first/middle/last pane; failed spawn rollback; join/move/break; reorder keeps focus and pane width preference; swap/rotate |
| Commands | Split commands and aliases rejected before side effects; tiled insertion distinct from floating creation; unsupported resize forms rejected; shared operations cannot bypass the layout rules |
| Environment | Existing session environment and `-e` overrides preserved; `-c '#{pane_current_path}'` after changing directory; existing target/format semantics; paths containing spaces; directory fallback behavior unchanged; actual cwd matches PWD |
| Resizing | Half/full toggle twice restores preference; focus and other widths unchanged; explicit setters rejected; grow/shrink terminal; maintain preferences; no repeated resize scheduling |
| Focus | Already visible pane; partially visible pane; offscreen pane in each direction; no wrapping or viewport movement at either end; no cursor-driven drift |
| Viewport | Manual pan and reset; focus after panning; clients of different widths; active pane wider than a client |
| Rendering | Borders and cursor at viewport edges; mouse selection after panning; partially visible neighbors; offscreen pane output |
| Transitions | Enter/leave scrolling; layout cycling; previous-layout restore; JSON round trip; legacy geometry output; zoom/unzoom; resize while zoomed |
| Regression | Existing layouts, ordinary splits/resizes, floating panes, and existing explicit panning |

Run relevant Rust tests and existing model-boundary checks after implementation,
then a build and the isolated interactive smoke checks. Review the final diff to
confirm `src/compat/` is untouched and there are no unrelated changes.

## Completion criteria

The confirmed behavior works through normal hmux commands. Pane geometry remains
stable across creation, removal, focus changes, terminal resizing, and zoom.
Existing layouts retain their behavior, and the tests
exercise both scrolling behavior and its integration with the existing lifecycle.
