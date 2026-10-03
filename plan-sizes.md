# Window sizing design

Status: proposed correction to the current scrolling implementation. This replaces
the sizing model in `plan.md`.

Each window provides a logical canvas. Each client displays a portion of that
canvas through its own physical viewport. The `window-size` policy resolves the
participating clients' dimensions into one agreed size for arranging the window.

| Quantity | Owner | Meaning |
| --- | --- | --- |
| Viewport size and offset | Client | Available terminal area and position within the window |
| `negotiated_size` | Window sizing state | Committed result of the `window-size` policy |
| `size()` | Window | Logical canvas dimensions exposed to rendering and panning |

```text
Client viewports → window-size policy → negotiated_size → layout → Window.size()
                                                                    ↓
                                                     Each client's visible region
```

## Ownership

Retain one generic `negotiated_size` for every window, alongside the existing
manual and pending resize state. Manual settings and initial defaults supply this
input through the existing sizing rules. Pending dimensions describe a future
input; they do not replace the committed value. Automatic sizing retains the last
negotiated size when no eligible clients remain; explicit manual resizing still
applies.

Layout consumes the negotiated size, arranges panes, and produces the logical
canvas dimensions: at least the negotiated size, enlarged as required by the
layout. A lone half-width pane therefore leaves unused canvas space. Layout owns
scrolling policy.
Window does not need an `is_scrolling()` query. Rendering, clipping, menus, and
panning obtain logical bounds through `Window::size()`.

The negotiated size cannot be reconstructed from geometry: widths 81 and 82 both
produce 40-column half panes. It must remain available for later pane operations.

## Updates and invariants

- Client attachment, detachment, resizing, and sizing-policy changes recalculate
  the negotiated size. Compare a candidate with the pending or committed input,
  never with the expanded logical canvas.
- Arrange using that input and publish the resulting canvas size. Commit geometry
  and dimensions before pane resize callbacks or redraw notifications can observe
  them. Preserve existing explicit resource-release rules.
- Pane insertion and removal reuse the committed negotiated size and recompute
  the canvas. They do not renegotiate client dimensions.
- Zoom uses the negotiated size; unzoom restores the logical extent of the saved
  layout. Neither changes the negotiated size.
- Relative or partial `resize-window` requests operate on negotiated dimensions.
  Each client keeps its own viewport and offset, clamped to the logical canvas.

## Example and migration

With clients of widths 81 and 61, `window-size=largest` selects 81. Three half-width
panes occupy `40 + 1 + 40 + 1 + 40 = 122` columns. `Window::size()` reports 122;
the clients view 81 and 61 columns independently. A fourth pane still uses width 40.

Restore logical canvas semantics to `Window::size()`, introduce generic
`negotiated_size`, and remove `layout::canvas_size()` after migrating its callers.
Update resize comparisons and layout operations to use the appropriate quantity.
Keep the scrolling arrangement and arithmetic fixes. Validate client resizing,
independent panning, insertion/removal, detached windows, zoom, and ordinary
layouts whose minimum geometry exceeds the negotiated size.
