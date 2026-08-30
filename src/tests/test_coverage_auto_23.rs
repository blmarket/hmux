//! Coverage for [`crate::layout`] – layout helpers with [`Layout`] fixture.

use crate::layout::layout_cell_pane;
use crate::layout::{
    LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, layout_assign_pane, layout_count_cells,
    layout_create_cell, layout_fix_offsets, layout_fix_panes, layout_fix_zindexes,
    layout_free_cell, layout_make_leaf, layout_make_node, layout_resize, layout_root_ptr,
    layout_search_by_border, layout_set_size, layout_split_pane,
};
use crate::tests::test_fixtures::{Layout, Pane, Window, globals};
use ::core::ptr::null_mut;

// helpers

fn split(l: &mut Layout, i: usize, ty: u32) -> usize {
    unsafe {
        let lc = layout_split_pane(l.pane(i), ty, -1, 0);
        assert!(!lc.is_null(), "split failed");
        let j = l.add_pane(1, 1);
        layout_assign_pane(lc, l.pane(j), 0);
        j
    }
}

#[test]
fn layout_create_cell_defaults_and_free_null_is_safe() {
    let _g = globals();
    unsafe {
        let lc = layout_create_cell(null_mut());
        let lc = &raw const *lc as *mut crate::types::layout_cell;
        assert_eq!((*lc).type_0, LAYOUT_WINDOWPANE);
        assert_eq!((*lc).flags, 0);
        assert!((*lc).parent.is_null());
        assert!((*lc).wp_id.is_none());
        assert_eq!((*lc).sx, u32::MAX);
        assert_eq!((*lc).sy, u32::MAX);
        layout_free_cell(null_mut(), None);
    }
}

#[test]
fn layout_set_size_writes_geometry() {
    let _g = globals();
    unsafe {
        let lc = layout_create_cell(null_mut());
        let lc = &raw const *lc as *mut crate::types::layout_cell;
        layout_set_size(lc, 42, 17, 3, 5);
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
        let lc = layout_create_cell(null_mut());
        let lc = &raw const *lc as *mut crate::types::layout_cell;
        layout_make_leaf(lc, p.ptr());
        assert_eq!((*lc).type_0, LAYOUT_WINDOWPANE);
        assert_eq!(layout_cell_pane(w.ptr(), lc), p.ptr());
        assert_eq!((*p.ptr()).layout_cell, lc);
        layout_make_node(w.ptr(), lc, LAYOUT_LEFTRIGHT);
        assert_eq!((*lc).type_0, LAYOUT_LEFTRIGHT);
        assert!((*lc).wp_id.is_none());
        assert!((*p.ptr()).layout_cell.is_null());
        layout_make_node(w.ptr(), lc, LAYOUT_TOPBOTTOM);
        assert_eq!((*lc).type_0, LAYOUT_TOPBOTTOM);
    }
}

#[test]
fn layout_count_cells_tracks_splits() {
    let _g = globals();
    let mut l = Layout::new(80, 24);
    unsafe {
        assert_eq!(
            layout_count_cells(layout_root_ptr(&(*l.w()).layout_root)),
            1
        )
    };
    split(&mut l, 0, LAYOUT_LEFTRIGHT);
    unsafe {
        assert_eq!(
            layout_count_cells(layout_root_ptr(&(*l.w()).layout_root)),
            2
        )
    };
    split(&mut l, 0, LAYOUT_TOPBOTTOM);
    unsafe {
        assert_eq!(
            layout_count_cells(layout_root_ptr(&(*l.w()).layout_root)),
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
    unsafe { layout_resize(l.w(), 100, 30) };
    assert_eq!(l.dump(), "LR 100x30+0+0 [%1 50x30+0+0 | %2 49x30+51+0]");
    unsafe { layout_resize(l.w(), 60, 12) };
    assert_eq!(l.dump(), "LR 60x12+0+0 [%1 30x12+0+0 | %2 29x12+31+0]");
}

#[test]
fn layout_search_by_border_inside_vs_between() {
    let _g = globals();
    let mut l = Layout::new(80, 24);
    split(&mut l, 0, LAYOUT_LEFTRIGHT);
    unsafe {
        let root = layout_root_ptr(&(*l.w()).layout_root);
        // between panes at x=40 is the vertical border
        assert_eq!(
            layout_search_by_border(root, 40, 5),
            (*l.pane(0)).layout_cell
        );
        // inside first pane
        assert!(layout_search_by_border(root, 5, 5).is_null());
        // inside second pane
        assert!(layout_search_by_border(root, 60, 5).is_null());
    }
}

#[test]
fn layout_fix_offsets_propagates_after_resize() {
    let _g = globals();
    let mut l = Layout::new(80, 24);
    split(&mut l, 0, LAYOUT_LEFTRIGHT);
    split(&mut l, 1, LAYOUT_TOPBOTTOM);
    unsafe {
        layout_fix_offsets(l.w());
        assert_eq!(
            l.dump(),
            "LR 80x24+0+0 [%1 40x24+0+0 | TB 39x24+41+0 [%2 39x12+41+0 | %3 39x11+41+13]]"
        );
        // panes got correct offsets too
        layout_fix_panes(l.w(), null_mut());
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
        layout_fix_zindexes(l.w(), layout_root_ptr(&(*l.w()).layout_root));
        let order: Vec<u32> = (*l.w()).z_index.clone();
        // left side top-bottom children first, then right pane
        assert_eq!(order, vec![1, 3, 2]);
        // null root is safe
        layout_fix_zindexes(l.w(), null_mut());
    }
}

#[test]
fn layout_assign_pane_skip_flag_leaves_skip_pane_size() {
    let _g = globals();
    let mut l = Layout::new(80, 24);
    let j = split(&mut l, 0, LAYOUT_LEFTRIGHT);
    unsafe {
        (*l.pane(0)).sx = 5;
        (*l.pane(0)).sy = 5;
        layout_fix_panes(l.w(), l.pane(0));
        // skipped pane keeps old size, other pane is fixed to its cell
        assert_eq!((*l.pane(0)).sx, 5);
        assert_eq!((*l.pane(j)).sx, (*(*l.pane(j)).layout_cell).sx);
        // without skip both are fixed
        layout_fix_panes(l.w(), null_mut());
        assert_eq!((*l.pane(0)).sx, (*(*l.pane(0)).layout_cell).sx);
    }
}
