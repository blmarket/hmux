//! Persistent horizontal strips. The existing tree owns every pane cell;
//! Window owns the sizing basis and Pane owns the half/full width preference.

use super::{layout_create_cell, layout_fix_panes, layout_take_leaves};
use crate::src::events::events_fire_window;
use crate::src::server_fn::server_redraw_window;
use crate::src::shared::layout::*;
use crate::src::shared::pane::{window_pane, PANE_MINIMUM, PANE_SCROLLBARS_ALWAYS};
use crate::src::shared::window::{WindowRef, WINDOW_MAXIMUM};
use crate::src::tty::tty_update_window_offset;
use crate::src::window::{LayoutView, Window};
use crate::src::window_pane::WindowPane;
use std::cell::UnsafeCell;
use std::ffi::CString;
use std::rc::{Rc, Weak};

fn pane_width(visible: u32, full: bool, minimum: u32) -> u32 {
    let width = if full {
        visible
    } else {
        visible.saturating_sub(1) / 2
    };
    width.max(minimum).max(PANE_MINIMUM as u32)
}

// Leave room for every tiled pane to become full width at the maximum sizing
// basis. Once adopted, resizing and preference changes cannot overflow offsets.
pub(super) fn check_capacity(count: usize) -> Result<(), CString> {
    if count as u64 > (i32::MAX as u64 + 1) / (WINDOW_MAXIMUM as u64 + 1) {
        Err(c"scrolling layout is too wide".to_owned())
    } else {
        Ok(())
    }
}

fn strip_width(widths: impl IntoIterator<Item = u32>) -> Result<u32, CString> {
    let mut width = 0u32;
    for next in widths {
        width = width
            .checked_add(u32::from(width != 0))
            .and_then(|width| width.checked_add(next))
            .filter(|width| *width <= i32::MAX as u32)
            .ok_or_else(|| c"scrolling layout is too wide".to_owned())?;
    }
    Ok(width)
}

fn leaves(
    cell: &layout_cell,
    out: &mut Vec<(*mut layout_cell, bool, Weak<UnsafeCell<window_pane>>)>,
) {
    if cell.type_0 == LAYOUT_WINDOWPANE {
        out.push((cell.id(), cell.is_floating(), cell.wp.clone()));
    } else {
        for child in &cell.cells {
            leaves(child, out);
        }
    }
}

/// Arrange the live tree without invoking pane resize callbacks. All validation
/// and model queries finish before taking ownership of its cells.
pub(crate) unsafe fn arrange(
    window: &WindowRef,
    basis: (u32, u32),
    by_index: bool,
) -> Result<(), CString> {
    let mut snapshots = Vec::new();
    if let Some(root) = window.borrow_layout_root(LayoutView::Visible) {
        leaves(root, &mut snapshots);
    } else {
        return Ok(());
    }
    if by_index {
        let panes = window.pane_snapshot();
        snapshots.sort_by_key(|(_, floating, pane)| {
            (
                *floating,
                panes
                    .iter()
                    .position(|owner| pane.ptr_eq(&Rc::downgrade(owner)))
                    .unwrap_or(usize::MAX),
            )
        });
    }
    check_capacity(
        snapshots
            .iter()
            .filter(|(_, floating, _)| !floating)
            .count(),
    )?;
    let reserve = window.scrollbar_mode() == PANE_SCROLLBARS_ALWAYS;
    let height = basis
        .1
        .max(PANE_MINIMUM as u32 + u32::from(window.pane_border_status() != 0));
    let widths: Vec<_> = snapshots
        .iter()
        .map(|(_, floating, pane)| {
            if *floating {
                return 0;
            }
            let (full, minimum) = pane.upgrade().map_or((false, PANE_MINIMUM as u32), |pane| {
                (
                    pane.scrolling_full_width(),
                    pane.minimum_layout_width(reserve),
                )
            });
            pane_width(basis.0, full, minimum)
        })
        .collect();
    let width = strip_width(widths.iter().copied().filter(|width| *width != 0))?;
    let geometry = layout_geometry {
        sx: width.max(1),
        sy: height,
        xoff: 0,
        yoff: 0,
    };
    {
        let mut tree = window.borrow_layout_root_mut();
        let mut detached = layout_take_leaves(tree.take());
        let mut root = layout_create_cell();
        root.type_0 = LAYOUT_LEFTRIGHT;
        root.g = geometry;
        let mut x = 0;
        for ((id, floating, _), width) in snapshots.into_iter().zip(widths) {
            let index = detached
                .iter()
                .position(|cell| cell.id() == id)
                .expect("owned scrolling leaf");
            let mut cell = detached.remove(index);
            if !floating {
                cell.g = layout_geometry {
                    sx: width,
                    sy: height,
                    xoff: x as i32,
                    yoff: 0,
                };
                x += width + 1;
            }
            layout_cells_push_back(&mut *root, cell);
        }
        assert!(detached.is_empty());
        // Keep single-pane trees valid for the existing custom-layout format.
        *tree = if root.cells.len() == 1 {
            let mut only = root.cells.pop().unwrap();
            only.parent = std::ptr::null_mut();
            only.sibling_index = 0;
            Some(only)
        } else {
            Some(root)
        };
    }
    window.set_layout_size(width.max(basis.0), height, Some(basis));
    Ok(())
}

/// Reserve a new half-width leaf without dividing an existing pane. The normal
/// spawn/rollback path still owns assignment and removal of the reservation.
pub(crate) unsafe fn insert(
    window: &WindowRef,
    anchor: &Rc<UnsafeCell<window_pane>>,
    before: bool,
) -> Result<*mut layout_cell, CString> {
    let basis = window.sizing_size();
    let extra = pane_width(basis.0, false, PANE_MINIMUM as u32);
    check_capacity(
        window
            .pane_snapshot()
            .iter()
            .filter(|pane| !pane.is_floating())
            .count()
            + 1,
    )?;
    let anchor = anchor
        .layout_identity(false)
        .expect("scrolling insertion anchor");
    let mut cell = layout_create_cell();
    cell.g = layout_geometry {
        sx: extra,
        sy: basis.1.max(1),
        xoff: 0,
        yoff: 0,
    };
    let id = cell.id();
    {
        let mut tree = window.borrow_layout_root_mut();
        let detached = layout_take_leaves(tree.take());
        let mut root = layout_create_cell();
        root.type_0 = LAYOUT_LEFTRIGHT;
        let mut reserved = Some(cell);
        for leaf in detached {
            let at_anchor = leaf.id() == anchor;
            if at_anchor && before {
                layout_cells_push_back(&mut *root, reserved.take().unwrap());
            }
            layout_cells_push_back(&mut *root, leaf);
            if at_anchor && !before {
                layout_cells_push_back(&mut *root, reserved.take().unwrap());
            }
        }
        assert!(reserved.is_none(), "insertion anchor belongs to tree");
        *tree = Some(root);
    }
    arrange(window, basis, false).expect("validated scrolling insertion");
    Ok(id)
}

pub(crate) unsafe fn toggle(
    window: &WindowRef,
    pane: &Rc<UnsafeCell<window_pane>>,
) -> Result<(), CString> {
    let tiled = window
        .pane_layout_cell(&Rc::downgrade(pane), LayoutView::Unzoomed)
        .is_some_and(|cell| !cell.is_floating());
    if !window.is_scrolling() || !tiled {
        return Err(c"width toggle requires a tiled pane in a scrolling layout".to_owned());
    }
    let previous = pane.scrolling_full_width();
    window.unzoom(true);
    pane.set_scrolling_full_width(!previous);
    layout_fix_panes(window, None);
    tty_update_window_offset(window);
    server_redraw_window(window);
    events_fire_window(c"window-layout-changed".as_ptr(), window.clone());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths_reserve_the_separator_and_preserve_minima() {
        for visible in [1, 2, 3, 79, 80, 81, 10000] {
            let half = pane_width(visible, false, 1);
            assert_eq!(pane_width(visible, true, 1), visible);
            if visible >= 3 {
                assert!(2 * half + 1 <= visible);
            }
            assert!(half >= 1);
        }
        assert_eq!(pane_width(80, false, 1), 39);
        assert_eq!(pane_width(1, false, 4), 4);
        assert_eq!(strip_width([39, 80, 39]).unwrap(), 160);
        assert_eq!(strip_width([39]).unwrap(), 39);
        assert!(strip_width([i32::MAX as u32, 1]).is_err());
        let capacity = ((i32::MAX as u64 + 1) / (WINDOW_MAXIMUM as u64 + 1)) as usize;
        assert!(check_capacity(capacity).is_ok());
        assert!(check_capacity(capacity + 1).is_err());
    }
}
