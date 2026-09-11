//! Coverage for [`crate::layout`] – layout helpers with [`Layout`] fixture.

use super::*;
use crate::WindowPane;

use crate::layout::layout_cell_pane;
use crate::layout::{
    LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, layout_count_cells, layout_create_cell,
    layout_free_cell, layout_make_leaf, layout_make_node, layout_search_by_border, layout_set_size,
};
use crate::tests::test_fixtures::{Layout, Pane, Window, globals};

// helpers

fn split(l: &mut Layout, i: usize, ty: u32) -> usize {
    unsafe {
        let pane_id = (*l.pane(i)).pane_id();
        let lc = (l.reference()).split_pane_layout(
            &crate::window::window_pane_find_by_id(pane_id).expect("the layout pane exists"),
            ty,
            -1,
            0,
        );
        assert!(lc.is_some(), "split failed");
        let j = l.add_pane(1, 1);
        (l.reference()).assign_pane_layout(
            lc.as_ref().unwrap(),
            &crate::window::window_pane_find_by_id((*l.pane(j)).pane_id())
                .expect("the layout pane exists"),
            0,
        );
        j
    }
}

#[test]
fn layout_create_cell_defaults_and_free_null_is_safe() {
    let _g = globals();
    {
        let lc = layout_create_cell(None);
        assert_eq!((*lc).type_0, LAYOUT_WINDOWPANE);
        assert_eq!((*lc).flags, 0);
        assert!(!(*lc).parent);
        assert!((*lc).wp.as_ref().map(|pane| pane.id()).is_none());
        assert_eq!((*lc).sx, u32::MAX);
        assert_eq!((*lc).sy, u32::MAX);
        layout_free_cell(None);
    }
}

#[test]
fn layout_set_size_writes_geometry() {
    let _g = globals();
    {
        let mut lc = layout_create_cell(None);
        layout_set_size(&mut *lc, 42, 17, 3, 5);
        assert_eq!((*lc).sx, 42);
        assert_eq!((*lc).sy, 17);
        assert_eq!((*lc).xoff, 3);
        assert_eq!((*lc).yoff, 5);
    }
}

#[test]
fn layout_make_leaf_and_node_round_trip() {
    let _g = globals();
    let mut w = Window::new(10, "leafnode", 80, 24);
    let mut p = Pane::new(1, 80, 24, 100);
    w.add_pane(&mut p);
    unsafe {
        let mut lc = layout_create_cell(None);
        layout_make_leaf(&mut *lc, &mut *p.ptr());
        assert_eq!((*lc).type_0, LAYOUT_WINDOWPANE);
        assert!(
            layout_cell_pane(&*lc).is_some_and(|pane| (*p.ptr()).observation().as_ref() == Some(&pane))
        );
        assert_eq!(
            (*lc).wp.as_ref().map(|pane| pane.id()),
            Some((*p.ptr()).pane_id())
        );
        layout_make_node(&mut *lc, LAYOUT_LEFTRIGHT);
        assert_eq!((*lc).type_0, LAYOUT_LEFTRIGHT);
        assert!((*lc).wp.as_ref().map(|pane| pane.id()).is_none());
        layout_make_node(&mut *lc, LAYOUT_TOPBOTTOM);
        assert_eq!((*lc).type_0, LAYOUT_TOPBOTTOM);
    }
}

#[test]
fn layout_count_cells_tracks_splits() {
    let _g = globals();
    let mut l = Layout::new(80, 24);
    unsafe {
        assert_eq!(
            layout_count_cells(&mut *(*l.w()).layout_mut(LayoutAccess(())).root.as_deref_mut().unwrap()),
            1
        )
    };
    split(&mut l, 0, LAYOUT_LEFTRIGHT);
    unsafe {
        assert_eq!(
            layout_count_cells(&mut *(*l.w()).layout_mut(LayoutAccess(())).root.as_deref_mut().unwrap()),
            2
        )
    };
    split(&mut l, 0, LAYOUT_TOPBOTTOM);
    unsafe {
        assert_eq!(
            layout_count_cells(&mut *(*l.w()).layout_mut(LayoutAccess(())).root.as_deref_mut().unwrap()),
            3
        )
    };
    assert_eq!(
        l.dump(),
        "LR 80x24+0+0 [TB 40x24+0+0 [%1 40x12+0+0 | %3 40x11+0+13] | %2 39x24+41+0]"
    );
}

#[test]
fn layout_resize_grows_and_shrinks_window() {
    let _g = globals();
    let mut l = Layout::new(80, 24);
    split(&mut l, 0, LAYOUT_LEFTRIGHT);
    unsafe { (l.reference()).resize_layout(100, 30) };
    assert_eq!(l.dump(), "LR 100x30+0+0 [%1 50x30+0+0 | %2 49x30+51+0]");
    unsafe { (l.reference()).resize_layout(60, 12) };
    assert_eq!(l.dump(), "LR 60x12+0+0 [%1 30x12+0+0 | %2 29x12+31+0]");
}

#[test]
fn layout_search_by_border_inside_vs_between() {
    let _g = globals();
    let mut l = Layout::new(80, 24);
    split(&mut l, 0, LAYOUT_LEFTRIGHT);
    unsafe {
        let root = (*l.w()).layout_mut(LayoutAccess(())).root.as_deref_mut().unwrap();
        // between panes at x=40 is the vertical border
        assert!(
            layout_search_by_border(&mut *root, 40, 5).is_some_and(|path| path
                .get(&*root)
                .is_some_and(|cell| cell.wp.as_ref().map(|pane| pane.id())
                    == Some((*l.pane(0)).pane_id())))
        );
        // inside first pane
        assert!(layout_search_by_border(&mut *root, 5, 5).is_none());
        // inside second pane
        assert!(layout_search_by_border(&mut *root, 60, 5).is_none());
    }
}

#[test]
fn layout_fix_offsets_propagates_after_resize() {
    let _g = globals();
    let mut l = Layout::new(80, 24);
    split(&mut l, 0, LAYOUT_LEFTRIGHT);
    split(&mut l, 1, LAYOUT_TOPBOTTOM);
    unsafe {
        (l.reference()).fix_layout_offsets();
        assert_eq!(
            l.dump(),
            "LR 80x24+0+0 [%1 40x24+0+0 | TB 39x24+41+0 [%2 39x12+41+0 | %3 39x11+41+13]]"
        );
        // panes got correct offsets too
        (l.reference()).fix_layout_panes(None);
        let panes = l.panes();
        assert!(panes[0].contains("+0+0"));
        assert!(panes[1].contains("+41+0"));
        assert!(panes[2].contains("+41+13"));
    }
}

#[test]
fn layout_fix_zindexes_follows_left_to_right_depth_first() {
    let _g = globals();
    let mut l = Layout::new(80, 24);
    split(&mut l, 0, LAYOUT_LEFTRIGHT);
    split(&mut l, 0, LAYOUT_TOPBOTTOM);
    unsafe {
        (*l.w()).z_index.clear();
        (l.reference()).fix_layout_zindexes();
        let order: Vec<u32> = (*l.w())
            .z_index
            .iter()
            .map(|pane| pane.id())
            .collect::<Vec<_>>();
        // left side top-bottom children first, then right pane
        assert_eq!(order, vec![1, 3, 2]);
        // null root is safe
        let root = (*l.w()).layout_mut(LayoutAccess(())).root.take();
        (l.reference()).fix_layout_zindexes();
        assert_eq!(
            (*l.w())
                .z_index
                .iter()
                .map(|pane| pane.id())
                .collect::<Vec<_>>(),
            order
        );
        (*l.w()).layout_mut(LayoutAccess(())).root = root;
    }
}

#[test]
fn layout_assign_pane_skip_flag_leaves_skip_pane_size() {
    let _g = globals();
    let mut l = Layout::new(80, 24);
    let j = split(&mut l, 0, LAYOUT_LEFTRIGHT);
    unsafe {
        (*l.pane(0)).configure_test(crate::window_pane::PaneTestSetup::Size(crate::pane_resize::PaneSize {
            width: 5,
            height: 5,
        }));
        (l.reference()).fix_layout_panes(
            l.pane(0)
                .as_ref()
                .and_then(|pane| (pane).observation())
                .as_ref(),
        );
        // skipped pane keeps old size, other pane is fixed to its cell
        assert_eq!((*l.pane(0)).geometry().sx, 5);
        assert_eq!((*l.pane(j)).geometry().sx, l.cell(j).unwrap().sx);
        // without skip both are fixed
        (l.reference()).fix_layout_panes(None);
        assert_eq!((*l.pane(0)).geometry().sx, l.cell(0).unwrap().sx);
    }
}
