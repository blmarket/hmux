//! The tiling layout: the panes divide the window between them, and nothing
//! extends past it. A fixed rule over the pane list places them: the first
//! pane takes the window, and each next pane splits the previous pane's
//! rectangle in two along its longer side, the earlier pane keeping the first
//! half.

use super::{window_geometry, LayoutNode};
use crate::src::shared::layout::layout_geometry;

/// `count` panes, each splitting the previous one's rectangle.
pub(super) fn tree(count: usize, size: (u32, u32)) -> LayoutNode {
    split_from(0, count, window_geometry(size))
}

/// Panes `first` onwards of `count`, in rectangle `g`.
fn split_from(first: usize, count: usize, g: layout_geometry) -> LayoutNode {
    if first + 1 == count {
        return LayoutNode::Pane(first);
    }
    let across = splits_across(g);
    LayoutNode::Split {
        across,
        children: vec![
            (1, LayoutNode::Pane(first)),
            (1, split_from(first + 1, count, second_half(g, across))),
        ],
    }
}

/// Whether a split of `g` puts its halves side by side: its longer side, as
/// the eye sees it, is across. A terminal cell is about twice as tall as it is
/// wide.
fn splits_across(g: layout_geometry) -> bool {
    g.sx >= 2 * g.sy
}

/// The second half of `g` split `across` or stacked, as `LayoutNode::place`
/// shares it: one separator between the halves, the odd cell to the first.
fn second_half(g: layout_geometry, across: bool) -> layout_geometry {
    let extent = if across { g.sx } else { g.sy };
    let shared = extent.saturating_sub(1);
    let first = shared - shared / 2;
    let offset = (first + 1) as i32;
    if across {
        layout_geometry {
            sx: shared / 2,
            xoff: g.xoff + offset,
            ..g
        }
    } else {
        layout_geometry {
            sy: shared / 2,
            yoff: g.yoff + offset,
            ..g
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::LayoutKind;
    use crate::src::shared::pane::window_pane;
    use std::cell::UnsafeCell;
    use std::rc::Rc;

    fn panes(count: usize) -> Vec<Rc<UnsafeCell<window_pane>>> {
        (0..count)
            .map(|_| Rc::new(UnsafeCell::new(window_pane::default())))
            .collect()
    }

    fn rects(count: usize, size: (u32, u32)) -> Vec<(u32, u32, i32, i32)> {
        LayoutKind::Tiling
            .arrange(&panes(count), size)
            .iter()
            .map(|g| (g.sx, g.sy, g.xoff, g.yoff))
            .collect()
    }

    #[test]
    fn a_lone_pane_fills_the_window() {
        assert_eq!(rects(1, (80, 24)), [(80, 24, 0, 0)]);
    }

    #[test]
    fn each_pane_splits_the_previous_one_along_its_longer_side() {
        // 80x24 looks wider than tall: side by side.
        assert_eq!(rects(2, (80, 24)), [(40, 24, 0, 0), (39, 24, 41, 0)]);
        // The second pane's 39x24 looks taller than wide: stacked.
        assert_eq!(
            rects(3, (80, 24)),
            [(40, 24, 0, 0), (39, 12, 41, 0), (39, 11, 41, 13)]
        );
        // The third pane's 39x11 looks wider again: side by side.
        assert_eq!(
            rects(4, (80, 24)),
            [
                (40, 24, 0, 0),
                (39, 12, 41, 0),
                (19, 11, 41, 13),
                (19, 11, 61, 13)
            ]
        );
    }

    #[test]
    fn the_same_list_always_tiles_the_same_way() {
        assert_eq!(rects(5, (81, 25)), rects(5, (81, 25)));
    }

    #[test]
    fn panes_below_the_minimum_are_refused() {
        let list = panes(3);
        assert!(LayoutKind::Tiling.admits(&list, (81, 25)).is_ok());
        assert!(LayoutKind::Tiling.admits(&list, (2, 2)).is_err());
    }
}
