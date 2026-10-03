# Window sizing design

Status: proposed. Base is `main` at `24aa7b57`; nothing here is implemented. This
replaces "Separate the sizing basis from the strip extent" in `plan.md` and lands
before the scrolling work.

`Window::size()` is the size the `window-size` policy dictates: the physical view
of the window. How much room the arranged panes occupy is a separate, logical
quantity owned by layout. The two are equal for ordinary layouts and differ when
panes cannot shrink any further, or when a layout deliberately extends past the view.

| Quantity | Owner | Stored | Meaning |
| --- | --- | --- | --- |
| Viewport size and offset | Client | yes | That terminal's area and its pan position |
| `Window::size()` | Window | yes | Result of the `window-size` policy; the basis for arranging panes |
| `layout::logical_size(window)` | Layout | no | Area the arranged window occupies; what clients pan across |

```text
Client viewports → window-size policy → Window::size() → layout → logical size
                                                                      ↓
                                                       Each client's visible region
```

## Current behavior being replaced

`resize_window` raises the stored size to the layout root (`src/window/api.rs:1244`),
and the six presets and layout import write the root size back through
`set_layout_size`. `size()` therefore already means "logical extent", and the
policy result is discarded. Two consequences:

- `recalculate_size` compares the policy result with the inflated size
  (`src/resize.rs:419`), so a window whose layout exceeds its view is resized
  again on every recalculation.
- That same mismatch is the only thing that shrinks the layout back after a pane
  closes or an imported layout is installed.

## Model

- `Window::size()` changes only when the policy result changes: client attach,
  detach or resize, a `window-size` change, or a manual `resize-window`. Layout
  operations never write it. `#{window_width}`, `#{window_height}` and the
  `window-resized` event report it.
- `layout::logical_size(window)` is computed on demand: per dimension, the larger
  of `Window::size()` and the visible tiled root. A floating root or a missing root
  yields `Window::size()`. The saved (unzoomed) tree is not consulted. It is never
  stored, so nothing publishes it and it cannot go stale.
- Layout arranges against `Window::size()`. Presets, zoom, full-size splits and
  percentage arguments already read it and need no change; they stop inheriting an
  inflated value. A zoomed pane is therefore view-sized, and zoom or unzoom never
  changes `Window::size()`.
- Floating panes stay bounded by `Window::size()`, as the existing clamp does.
- Each client clips and pans across the logical size with its own viewport.

## Refit rule

Once `size()` stops absorbing the layout, the implicit shrink described above is
gone. Replace it with one rule: whenever the tiled tree or its minimums change
without a size change, refit the layout to `Window::size()` using the existing
`layout_resize`. Proposed single trigger: `recalculate_size`, when the policy
result equals `size()` but the logical size does not. A refit that moves nothing
must be silent (no redraw, no events); one that moves panes fires
`window-layout-changed`, not `window-resized`. Layout import refits directly
after installing the parsed tree instead of resizing the window to it.

## `Window::size()` usages

Today every caller receives the logical extent. Callers below the line keep
`size()` and become correct; callers above it switch to `layout::logical_size`.

| Needs logical size | Sites |
| --- | --- |
| Client clipping and panning | `src/tty.rs:1042,1063`; `src/server_client/api.rs:510,531` |
| Scene bounds | `src/screen_redraw.rs:204,224,651-653,828,838`; `src/window_pane/render.rs:62,511` |
| Pane write clipping | `src/window_pane/api.rs:659`, used by `src/screen_write.rs:2217-2263` |
| Layout edges | Directional navigation `src/window_pane/mod.rs:1812,1867,1921,1966`; `pane_at_bottom`/`pane_at_right` `src/window_pane/format.rs:487,1064`; position names `src/window/mod.rs:724-752` |
| Menus | `src/window/api.rs:607-615`; `src/cmd/entries/display_menu.rs:534-683`; `menu_resize` in `window_resize` |

| Keeps `Window::size()` | Sites |
| --- | --- |
| Sizing policy | `src/resize.rs:380,419`; `src/cmd/entries/resize_window.rs:91`; `src/sort.rs:119`; `src/format/callbacks.rs:1276,1479`; `src/window/api.rs:1214,1228` |
| Seeding a window or pane | `src/cmd/entries/break_pane.rs:256`; `src/spawn.rs:248`; `src/window/mod.rs:975` |
| Layout basis | `src/layout/core.rs:740,2161,2659`; `src/layout/set.rs:198,314,501,688,875,1062`; `src/cmd/entries/join_pane.rs:115,421-467`; `src/cmd/entries/resize_pane.rs:136,171,198` |
| Floating panes | `src/layout/core.rs:2231,2274-2346`; `src/spawn.rs:389-404`; `src/window/api.rs:629,638`; `src/screen_redraw.rs:261-270,636-648` |

Menus keep today's behavior by moving to the logical size; the scrolling work
revisits them per client viewport.

## Sequence

1. Add `layout::logical_size` and move the "needs logical size" callers to it.
   Both quantities are still equal, so behavior is unchanged and existing tests
   must pass untouched.
2. Stop inflating: remove the clamp in `resize_window`, remove `set_layout_size`
   and its seven callers, and make layout import refit to `Window::size()`.
3. Add the refit trigger in `recalculate_size`.
4. Adopt the untracked `src/window/size_tests.rs`: rename its `canvas_size` to
   `logical_size`, change its `window-resized` and `#{window_width}`
   expectations from the logical size to `Window::size()`, and drop its
   expectation that recalculating without clients cancels a pending resize.

## Validation

- A window narrower than its panes' minimums: `size()` stays at the policy
  result, the logical size is larger, clients pan, and repeated recalculation
  causes no redraw.
- Closing a pane, changing border status or scrollbars, and importing a layout
  each shrink the logical size without a client event.
- Zoom, resize while zoomed, and unzoom leave `size()` untouched.
- Clients of different sizes under `largest`, `smallest`, `latest` and `manual`.
- Floating-only windows and detached windows report `size()` as the logical size.

## Unchanged on purpose

- A pending resize survives when no eligible client remains and is applied when
  the window next becomes current in an attached session, as today. It is the
  most recent policy result.
- Layout strings, including `%layout-change`, keep reporting the layout root.
  The root already exceeds the requested size when panes are at their minimums
  (`resize-window -x 2` with three panes yields a root of 5), so control clients
  see nothing new. `#{window_width}` now reports the requested size instead.
