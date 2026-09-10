use super::*;
use crate::options::OptionsRef;
use crate::pane_geometry::PaneGeometryState;
use crate::pane_identity::PaneIdentity;
use crate::screen::PANE_SCROLLBARS_RIGHT;
use crate::tests::test_fixtures::{Item, Pane, Window, globals};
use crate::window::PANE_SCROLLBARS_ALWAYS;
use crate::window::window_active_pane;
use crate::window::{window_pane_is_floating, window_pane_show_scrollbar};
use crate::window_dimensions::WindowDimensionsState;
use crate::window_scrollbar::{WindowScrollbarSettings, WindowScrollbarState};
use ::core::ffi::c_int;
use ::std::ffi::CString;

/// A window carrying a layout tree and the panes that hang off it. The
/// window and its panes are the server-free fixtures; the tree is real, and
/// is freed before the panes go.
struct Layout {
    window: Window,
    panes: Vec<Pane>,
    next_id: u_int,
}

fn set_scrollbar_dimensions(wp: &mut (impl crate::WindowPane + ?Sized), width: c_int, padding: c_int) {
    let mut style = wp.scrollbar_style();
    style.width = width;
    style.padding = padding;
    wp.set_scrollbar_style(style);
}

impl Layout {
    fn reference(&self) -> WindowRef {
        self.window.reference()
    }

    fn cell(&mut self, pane: usize) -> Option<std::cell::Ref<'_, layout_cell>> {
        let id = unsafe { (*self.pane(pane)).pane_id() };
        let w = { self.window.handle().as_window() };
        std::cell::Ref::filter_map(w, |w| {
            crate::layout::layout_cell_for_pane(
                w.layout_root.as_deref(),
                &crate::window::window_pane_find_by_id(id).expect("the pane allocation exists"),
            )
            .map(|(cell, _)| cell)
        })
        .ok()
    }

    fn cell_path(&mut self, pane: usize) -> LayoutCellPath {
        let id = unsafe { (*self.pane(pane)).pane_id() };
        let w = { self.window.handle().as_window() };
        LayoutCellPath::for_pane(
            w.layout_root.as_deref().unwrap(),
            &crate::window::window_pane_find_by_id(id).expect("the pane allocation exists"),
        )
        .unwrap()
    }

    fn border(&mut self, pane: usize, status: c_int) -> c_int {
        let id = unsafe { (*self.pane(pane)).pane_id() };
        let w = { self.window.handle().as_window() };
        layout_add_horizontal_border(
            w.layout_root.as_deref(),
            layout_cell_for_pane(
                w.layout_root.as_deref(),
                &crate::window::window_pane_find_by_id(id).expect("the pane allocation exists"),
            )
            .unwrap()
            .0,
            status,
        )
    }

    fn set_floating(&mut self, pane: usize, floating: bool) {
        let id = unsafe { (*self.pane(pane)).pane_id() };
        let owner = self.window.reference();
        crate::tests::test_fixtures::set_pane_floating(&mut owner.as_window_mut(), id, floating);
    }

    /// A window of `sx` by `sy` with one pane filling it, as `layout_init`
    /// leaves a freshly created window.
    fn new(sx: u_int, sy: u_int) -> Layout {
        let mut l = Layout {
            window: Window::new(1, "layout", sx, sy),
            panes: Vec::new(),
            next_id: 0,
        };
        l.add_pane(sx, sy);
        unsafe {
            (l.reference()).init_layout(
                &crate::window::window_pane_find_by_id((*l.pane(0)).pane_id())
                    .expect("the layout pane exists"),
            )
        };
        l
    }

    fn w(&mut self) -> *mut window {
        self.window.ptr()
    }

    fn pane(&mut self, i: usize) -> *mut (dyn crate::WindowPane + 'static) {
        self.panes[i].ptr()
    }

    fn resize_pane(&mut self, pane: usize, axis: layout_type, change: c_int, opposite: c_int) {
        unsafe {
            let id = (*self.pane(pane)).pane_id();
            let owner = self.window.reference();
            owner.resize_pane(
                &crate::window::window_pane_find_by_id(id).expect("the layout pane exists"),
                axis,
                change,
                opposite,
            );
        }
    }

    fn close(&mut self, pane: usize) {
        unsafe {
            let id = (*self.pane(pane)).pane_id();
            let owner = self.window.reference();
            owner.close_pane_layout(
                &crate::window::window_pane_find_by_id(id).expect("the layout pane exists"),
            );
        }
    }

    fn spread_out(&mut self, pane: usize) {
        unsafe {
            let id = (*self.pane(pane)).pane_id();
            let owner = self.window.reference();
            owner.spread_pane_layout(
                &crate::window::window_pane_find_by_id(id).expect("the layout pane exists"),
            );
        }
    }

    fn resize_pane_to(&mut self, pane: usize, axis: layout_type, size: u_int) {
        unsafe {
            let id = (*self.pane(pane)).pane_id();
            let owner = self.window.reference();
            owner.resize_pane_to(
                &crate::window::window_pane_find_by_id(id).expect("the layout pane exists"),
                axis,
                size,
            );
        }
    }

    /// A pane in the window's list, not yet in the layout tree.
    fn add_pane(&mut self, sx: u_int, sy: u_int) -> usize {
        self.next_id += 1;
        let mut pane = Pane::new(self.next_id, sx, sy, 100);
        self.window.add_pane(&mut pane);
        self.panes.push(pane);
        self.panes.len() - 1
    }

    /// Splits pane `i` and gives the new cell a pane of its own, the way
    /// the spawn path does. Answers the new pane's index, or `None` if
    /// there was no room.
    fn split(&mut self, i: usize, type_0: layout_type, size: c_int, flags: c_int) -> Option<usize> {
        unsafe {
            let pane_id = (*self.pane(i)).pane_id();
            let lc = (self.reference()).split_pane_layout(
                &crate::window::window_pane_find_by_id(pane_id).expect("the layout pane exists"),
                type_0,
                size,
                flags,
            );
            if lc.is_none() {
                return None;
            }
            let j = self.add_pane(1, 1);
            (self.reference()).assign_pane_layout(
                lc.as_ref().unwrap(),
                &crate::window::window_pane_find_by_id((*self.pane(j)).pane_id())
                    .expect("the layout pane exists"),
                0,
            );
            Some(j)
        }
    }

    /// The tree as one line: each node is its type, size and offset, with
    /// its children in brackets.
    fn dump(&mut self) -> String {
        unsafe { dump_cell((*self.w()).layout_root.as_deref()) }
    }

    /// The sizes and offsets the panes themselves were given.
    fn panes(&self) -> Vec<String> {
        unsafe {
            self.window
                .handle()
                .as_window()
                .panes
                .iter()
                .map(|pane| {
                    let pane = pane.as_pane();
                    let geometry = pane.geometry();
                    format!(
                        "%{} {}x{}+{}+{}",
                        pane.pane_id(),
                        geometry.sx,
                        geometry.sy,
                        geometry.xoff,
                        geometry.yoff
                    )
                })
                .collect()
        }
    }
}

impl Drop for Layout {
    fn drop(&mut self) {
        (self.window.reference()).free_layout();
    }
}

fn dump_cell(lc: Option<&layout_cell>) -> String {
    let Some(lc) = lc else {
        return "-".to_string();
    };
    let here = format!("{}x{}+{}+{}", lc.sx, lc.sy, lc.xoff, lc.yoff);
    let floating = if lc.flags & LAYOUT_CELL_FLOATING != 0 {
        "*"
    } else {
        ""
    };
    match lc.type_0 {
        LAYOUT_WINDOWPANE => format!(
            "%{}{floating} {here}",
            lc.wp
                .as_ref()
                .map(|pane| pane.id())
                .unwrap_or(u_int::MAX)
        ),
        LAYOUT_LEFTRIGHT | LAYOUT_TOPBOTTOM => {
            let kids: Vec<String> = lc
                .cells
                .iter()
                .map(|child| dump_cell(Some(child)))
                .collect();
            let name = if lc.type_0 == LAYOUT_LEFTRIGHT {
                "LR"
            } else {
                "TB"
            };
            format!("{name}{floating} {here} [{}]", kids.join(" | "))
        }
        _ => format!("?{floating} {here}"),
    }
}

/// Runs `body` with `pane-border-status` set to `status` on the window's
/// own options.
fn with_status(l: &mut Layout, status: c_int, body: impl FnOnce(&mut Layout)) {
    unsafe {
        (*(*l.w()).options_ref())
            .set_number(c"pane-border-status", status as core::ffi::c_longlong);
        body(l);
        (*(*l.w()).options_ref()).set_number(c"pane-border-status", 0);
    }
}

fn guard() -> crate::tests::test_fixtures::GlobalsGuard {
    globals()
}

#[test]
fn a_new_layout_is_one_cell_filling_the_window() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    assert_eq!(l.dump(), "%1 80x24+0+0");
    assert_eq!(l.panes(), vec!["%1 80x24+0+0"]);
    unsafe {
        assert_eq!(
            layout_count_cells(&mut *(*l.w()).layout_root.as_deref_mut().unwrap()),
            1
        );
        assert_eq!(
            (*l.w())
                .layout_root
                .as_deref()
                .unwrap()
                .wp
                .as_ref()
                .map(|pane| pane.id()),
            Some((*l.pane(0)).pane_id())
        );
    }
}

#[test]
fn a_fresh_cell_starts_at_the_largest_size_there_is() {
    let _g = guard();
    {
        let lc = layout_create_cell(None);
        assert_eq!((*lc).type_0, LAYOUT_WINDOWPANE);
        assert_eq!((*lc).flags, 0);
        assert!(!(*lc).parent);
        assert_eq!((*lc).sx, UINT_MAX as u_int);
        assert_eq!((*lc).sy, UINT_MAX as u_int);
        assert_eq!((*lc).xoff, INT_MAX);
        assert_eq!((*lc).yoff, INT_MAX);
        assert!((*lc).wp.as_ref().map(|pane| pane.id()).is_none());
        layout_free_cell(None);
    }
}

#[test]
fn splitting_left_and_right_halves_the_window() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    assert_eq!(l.split(0, LAYOUT_LEFTRIGHT, -1, 0), Some(1));
    assert_eq!(l.dump(), "LR 80x24+0+0 [%1 40x24+0+0 | %2 39x24+41+0]");
    assert_eq!(l.panes(), vec!["%1 40x24+0+0", "%2 39x24+41+0"]);
}

#[test]
fn splitting_top_and_bottom_halves_the_window() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    assert_eq!(l.split(0, LAYOUT_TOPBOTTOM, -1, 0), Some(1));
    assert_eq!(l.dump(), "TB 80x24+0+0 [%1 80x12+0+0 | %2 80x11+0+13]");
}

#[test]
fn a_split_of_a_given_size_gives_the_new_pane_that_size() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, 20, 0);
    assert_eq!(l.dump(), "LR 80x24+0+0 [%1 59x24+0+0 | %2 20x24+60+0]");
}

#[test]
fn a_split_before_puts_the_new_pane_first() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, 20, SPAWN_BEFORE);
    assert_eq!(l.dump(), "LR 80x24+0+0 [%2 20x24+0+0 | %1 59x24+21+0]");
}

#[test]
fn a_split_size_is_held_between_one_and_two_short_of_the_whole() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, 0, 0);
    assert_eq!(l.dump(), "LR 80x24+0+0 [%1 78x24+0+0 | %2 1x24+79+0]");

    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, 100, 0);
    assert_eq!(l.dump(), "LR 80x24+0+0 [%1 1x24+0+0 | %2 78x24+2+0]");
}

#[test]
fn a_window_too_small_to_split_says_so() {
    let _g = guard();
    let mut l = Layout::new(2, 24);
    assert_eq!(l.split(0, LAYOUT_LEFTRIGHT, -1, 0), None);
    let mut l = Layout::new(80, 2);
    assert_eq!(l.split(0, LAYOUT_TOPBOTTOM, -1, 0), None);
}

#[test]
fn a_third_split_of_the_same_kind_joins_the_same_node() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    assert_eq!(
        l.dump(),
        "LR 80x24+0+0 [%1 40x24+0+0 | %2 19x24+41+0 | %3 19x24+61+0]"
    );
    unsafe {
        assert_eq!(
            layout_count_cells(&mut *(*l.w()).layout_root.as_deref_mut().unwrap()),
            3
        )
    };
}

#[test]
fn a_split_of_the_other_kind_nests_a_node() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    assert_eq!(
        l.dump(),
        "LR 80x24+0+0 [%1 40x24+0+0 | TB 39x24+41+0 [%2 39x12+41+0 | %3 39x11+41+13]]"
    );
}

#[test]
fn splitting_resolves_siblings_from_the_owned_tree() {
    let _g = guard();
    let mut layout = Layout::new(80, 24);
    layout.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    let owner = layout.window.reference();
    {
        owner
            .as_window_mut()
            .layout_root
            .as_deref_mut()
            .unwrap()
            .cells[1]
            .parent = false;
    }
    layout.split(1, LAYOUT_LEFTRIGHT, -1, SPAWN_BEFORE);
    assert_eq!(
        layout.dump(),
        "LR 80x24+0+0 [%1 40x24+0+0 | %3 19x24+41+0 | %2 19x24+61+0]"
    );
}

#[test]
fn closing_a_pane_gives_its_room_back_to_its_neighbour() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.close(1);
    assert_eq!(l.dump(), "%1 80x24+0+0");
    {
        assert!(l.cell(1).is_none())
    };

    l.close(1);
    assert_eq!(l.dump(), "%1 80x24+0+0");
}

#[test]
fn parent_presence_follows_splitting_and_root_promotion() {
    use crate::layout_cell::LayoutCell;

    let _g = guard();
    let mut layout = Layout::new(80, 24);
    layout.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    let owner = layout.window.reference();
    {
        let payload = owner.as_window();
        let root = payload.layout_root.as_deref().unwrap();
        assert!(!root.layout_cell_has_parent());
        assert!(root.cells.iter().all(|cell| cell.layout_cell_has_parent()));
    }
    layout.close(1);
    {
        assert!(
            !owner
                .as_window()
                .layout_root
                .as_deref()
                .unwrap()
                .layout_cell_has_parent()
        );
    }
}

#[test]
fn closing_the_only_pane_leaves_no_tree() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.close(0);
    assert_eq!(l.dump(), "-");
}

#[test]
fn closing_one_of_three_leaves_the_node_in_place() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    l.close(1);
    assert_eq!(l.dump(), "LR 80x24+0+0 [%1 60x24+0+0 | %3 19x24+61+0]");
}

#[test]
fn closing_a_nested_pane_folds_the_node_away() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    l.close(2);
    assert_eq!(l.dump(), "LR 80x24+0+0 [%1 40x24+0+0 | %2 39x24+41+0]");
}

#[test]
fn closing_resolves_parent_and_neighbor_from_the_owned_tree() {
    let _g = guard();
    let mut layout = Layout::new(80, 24);
    layout.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    layout.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    let owner = layout.window.reference();
    {
        owner
            .as_window_mut()
            .layout_root
            .as_deref_mut()
            .unwrap()
            .cells[1]
            .cells[1]
            .parent = false;
        layout.close(2);
    }
    assert_eq!(layout.dump(), "LR 80x24+0+0 [%1 40x24+0+0 | %2 39x24+41+0]");
}

#[test]
fn resizing_the_window_shares_the_change_out() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe { (l.reference()).resize_layout(100, 30) };
    assert_eq!(l.dump(), "LR 100x30+0+0 [%1 50x30+0+0 | %2 49x30+51+0]");
    unsafe { (l.reference()).resize_layout(40, 12) };
    assert_eq!(l.dump(), "LR 40x12+0+0 [%1 20x12+0+0 | %2 19x12+21+0]");
}

#[test]
fn a_window_cannot_shrink_past_what_its_panes_need() {
    let _g = guard();
    let mut l = Layout::new(20, 10);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe { (l.reference()).resize_layout(1, 1) };
    assert_eq!(
        l.dump(),
        "LR 5x1+0+0 [%1 1x1+0+0 | %2 1x1+2+0 | %3 1x1+4+0]"
    );
}

#[test]
fn resizing_a_single_pane_window_only_grows_it() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe { (l.reference()).resize_layout(100, 30) };
    assert_eq!(l.dump(), "%1 100x30+0+0");
    unsafe { (l.reference()).resize_layout(40, 12) };
    assert_eq!(l.dump(), "%1 40x12+0+0");
}

#[test]
fn a_pane_can_be_resized_by_hand_in_either_direction() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.resize_pane(0, LAYOUT_LEFTRIGHT, 10, 1);
    assert_eq!(l.dump(), "LR 80x24+0+0 [%1 50x24+0+0 | %2 29x24+51+0]");
    l.resize_pane(0, LAYOUT_LEFTRIGHT, -20, 1);
    assert_eq!(l.dump(), "LR 80x24+0+0 [%1 30x24+0+0 | %2 49x24+31+0]");
}

#[test]
fn resizing_the_last_pane_moves_the_border_before_it() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.resize_pane(1, LAYOUT_LEFTRIGHT, 10, 1);
    assert_eq!(l.dump(), "LR 80x24+0+0 [%1 50x24+0+0 | %2 29x24+51+0]");
}

#[test]
fn resizing_across_a_kind_the_pane_is_not_in_does_nothing() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    let before = l.dump();
    l.resize_pane(0, LAYOUT_TOPBOTTOM, 5, 1);
    assert_eq!(l.dump(), before);
    l.resize_pane_to(0, LAYOUT_TOPBOTTOM, 5);
    assert_eq!(l.dump(), before);
}

#[test]
fn a_pane_can_be_resized_to_a_size() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.resize_pane_to(0, LAYOUT_LEFTRIGHT, 20);
    assert_eq!(l.dump(), "LR 80x24+0+0 [%1 20x24+0+0 | %2 59x24+21+0]");
    l.resize_pane_to(1, LAYOUT_LEFTRIGHT, 20);
    assert_eq!(l.dump(), "LR 80x24+0+0 [%1 59x24+0+0 | %2 20x24+60+0]");
}

#[test]
fn growing_without_the_opposite_side_stops_at_the_last_pane() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.resize_pane(1, LAYOUT_LEFTRIGHT, 10, 0);
    assert_eq!(l.dump(), "LR 80x24+0+0 [%1 50x24+0+0 | %2 29x24+51+0]");
}

#[test]
fn spreading_out_gives_every_pane_the_same_room() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    l.spread_out(0);
    assert_eq!(
        l.dump(),
        "LR 80x24+0+0 [%1 26x24+0+0 | %2 26x24+27+0 | %3 26x24+54+0]"
    );
    l.spread_out(0);
    assert_eq!(
        l.dump(),
        "LR 80x24+0+0 [%1 26x24+0+0 | %2 26x24+27+0 | %3 26x24+54+0]"
    );
}

#[test]
fn spreading_resolves_ancestors_from_the_owned_tree() {
    let _g = guard();
    let mut layout = Layout::new(80, 24);
    layout.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    layout.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    let owner = layout.window.reference();
    {
        owner
            .as_window_mut()
            .layout_root
            .as_deref_mut()
            .unwrap()
            .cells[0]
            .parent = false;
    }
    layout.spread_out(0);
    assert_eq!(
        layout.dump(),
        "LR 80x24+0+0 [%1 26x24+0+0 | %2 26x24+27+0 | %3 26x24+54+0]"
    );
}

#[test]
fn spreading_out_a_single_pane_does_nothing() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.spread_out(0);
    assert_eq!(l.dump(), "%1 80x24+0+0");
}

#[test]
fn spreading_a_cell_that_cannot_be_shared_answers_no() {
    let _g = guard();
    let l = Layout::new(80, 24);
    unsafe {
        assert_eq!(
            (l.reference()).spread_layout_cell(&LayoutCellPath::root()),
            0
        );
    }
    let mut l = Layout::new(4, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        assert_eq!(
            (l.reference()).spread_layout_cell(&LayoutCellPath::root()),
            0
        );
    }
}

#[test]
fn spreading_out_top_to_bottom_shares_the_rows() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    l.spread_out(0);
    assert_eq!(
        l.dump(),
        "TB 80x24+0+0 [%1 80x8+0+0 | %2 80x7+0+9 | %3 80x7+0+17]"
    );
}

#[test]
fn the_border_search_finds_the_cell_a_click_is_next_to() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        assert!(
            layout_search_by_border(&mut *root, 40, 5).is_some_and(|path| path
                .get(&*root)
                .is_some_and(|cell| cell.wp.as_ref().map(|pane| pane.id())
                    == Some((*l.pane(0)).pane_id())))
        );
        assert!(layout_search_by_border(&mut *root, 0, 0).is_none());
        assert!(layout_search_by_border(&mut *root, 79, 0).is_none());
    }
}

#[test]
fn the_border_search_works_top_to_bottom_and_through_nodes() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        assert!(
            layout_search_by_border(&mut *root, 5, 12).is_some_and(|path| path
                .get(&*root)
                .is_some_and(|cell| cell.wp.as_ref().map(|pane| pane.id())
                    == Some((*l.pane(0)).pane_id())))
        );
        assert!(layout_search_by_border(&mut *root, 5, 0).is_none());
    }

    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        assert!(
            layout_search_by_border(&mut *root, 50, 12).is_some_and(|path| path
                .get(&*root)
                .is_some_and(|cell| cell.wp.as_ref().map(|pane| pane.id())
                    == Some((*l.pane(1)).pane_id())))
        );
    }
}

#[test]
fn a_pane_border_status_line_takes_a_row_from_every_pane() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        with_status(&mut l, PANE_STATUS_TOP, |l| {
            (l.reference()).fix_layout_panes(None);
            assert_eq!(l.panes(), vec!["%1 80x11+0+1", "%2 80x11+0+13"]);
        });
        with_status(&mut l, PANE_STATUS_BOTTOM, |l| {
            (l.reference()).fix_layout_panes(None);
            assert_eq!(l.panes(), vec!["%1 80x12+0+0", "%2 80x10+0+13"]);
        });
        (l.reference()).fix_layout_panes(None);
        assert_eq!(l.panes(), vec!["%1 80x12+0+0", "%2 80x11+0+13"]);
    }
}

#[test]
fn a_status_line_leaves_the_pane_it_is_told_to_skip_alone() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        let skip = l.pane(0);
        with_status(&mut l, PANE_STATUS_TOP, |l| {
            (l.reference()).fix_layout_panes(
                skip.as_ref()
                    .and_then(|pane| (pane).observation())
                    .as_ref(),
            );
            assert_eq!(l.panes(), vec!["%1 80x12+0+0", "%2 80x11+0+13"]);
        });
    }
}

#[test]
fn a_status_line_makes_a_split_need_one_more_row() {
    let _g = guard();
    let mut l = Layout::new(80, 3);
    {
        with_status(&mut l, PANE_STATUS_TOP, |l| {
            assert_eq!(l.split(0, LAYOUT_TOPBOTTOM, -1, 0), None);
        });
    }
    let mut l = Layout::new(80, 3);
    assert!(l.split(0, LAYOUT_TOPBOTTOM, -1, 0).is_some());
}

#[test]
fn a_scrollbar_takes_columns_off_the_pane() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        (*l.w()).set_scrollbar_settings(WindowScrollbarSettings {
            sb: PANE_SCROLLBARS_ALWAYS,
            sb_pos: PANE_SCROLLBARS_RIGHT,
        });
        set_scrollbar_dimensions(&mut *l.pane(0), 2, 1);
        assert_eq!(
            window_pane_show_scrollbar(&*l.pane(0), (*l.w()).scrollbar_settings().sb),
            1
        );

        (l.reference()).fix_layout_panes(None);
        assert_eq!(l.panes(), vec!["%1 77x24+0+0"]);

        let mut scrollbar = (*l.w()).scrollbar_settings();
        scrollbar.sb_pos = PANE_SCROLLBARS_LEFT;
        (*l.w()).set_scrollbar_settings(scrollbar);
        (l.reference()).fix_layout_panes(None);
        assert_eq!(l.panes(), vec!["%1 77x24+3+0"]);
    }
}

#[test]
fn a_scrollbar_wider_than_the_pane_leaves_one_column() {
    let _g = guard();
    let mut l = Layout::new(4, 24);
    unsafe {
        (*l.w()).set_scrollbar_settings(WindowScrollbarSettings {
            sb: PANE_SCROLLBARS_ALWAYS,
            sb_pos: PANE_SCROLLBARS_RIGHT,
        });
        set_scrollbar_dimensions(&mut *l.pane(0), 8, -1);

        (l.reference()).fix_layout_panes(None);
        assert_eq!(l.panes(), vec!["%1 1x24+0+0"]);

        let mut scrollbar = (*l.w()).scrollbar_settings();
        scrollbar.sb_pos = PANE_SCROLLBARS_LEFT;
        (*l.w()).set_scrollbar_settings(scrollbar);
        (l.reference()).fix_layout_panes(None);
        assert_eq!(l.panes(), vec!["%1 1x24+3+0"]);

        set_scrollbar_dimensions(&mut *l.pane(0), 0, 0);
        (l.reference()).fix_layout_panes(None);
        assert_eq!(l.panes(), vec!["%1 3x24+1+0"]);
    }
}

#[test]
fn a_scrollbar_makes_a_side_by_side_split_need_more_room() {
    let _g = guard();
    let mut l = Layout::new(5, 24);
    unsafe {
        (*l.w()).set_scrollbar_settings(WindowScrollbarSettings {
            sb: PANE_SCROLLBARS_ALWAYS,
            sb_pos: PANE_SCROLLBARS_RIGHT,
        });
        set_scrollbar_dimensions(&mut *l.pane(0), 3, 1);
    }
    assert_eq!(l.split(0, LAYOUT_LEFTRIGHT, -1, 0), None);
}

#[test]
fn a_scrollbar_is_kept_out_of_the_room_a_pane_can_give_up() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        (*l.w()).set_scrollbar_settings(WindowScrollbarSettings {
            sb: PANE_SCROLLBARS_ALWAYS,
            sb_pos: PANE_SCROLLBARS_RIGHT,
        });
        let mut active = window_active_pane(&*l.w()).unwrap();
        set_scrollbar_dimensions(active.as_pane_mut(), 3, 1);
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        assert_eq!(
            (l.reference()).layout_resize_check(&mut *root, LAYOUT_LEFTRIGHT),
            69
        );
        let mut scrollbar = (*l.w()).scrollbar_settings();
        scrollbar.sb = PANE_SCROLLBARS_OFF;
        (*l.w()).set_scrollbar_settings(scrollbar);
        assert_eq!(
            (l.reference()).layout_resize_check(&mut *root, LAYOUT_LEFTRIGHT),
            77
        );
        assert_eq!(
            (l.reference()).layout_resize_check(&mut *root, LAYOUT_TOPBOTTOM),
            23
        );
    }
}

#[test]
fn a_full_size_split_keeps_the_other_panes_whole() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    assert_eq!(l.split(0, LAYOUT_LEFTRIGHT, -1, SPAWN_FULLSIZE), Some(2));
    assert_eq!(
        l.dump(),
        "LR 80x24+0+0 [TB 40x24+0+0 [%1 40x12+0+0 | %2 40x11+0+13] | %3 39x24+41+0]"
    );
}

#[test]
fn a_full_size_split_of_the_same_kind_adds_a_column() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    assert_eq!(l.split(0, LAYOUT_LEFTRIGHT, -1, SPAWN_FULLSIZE), Some(2));
    assert_eq!(
        l.dump(),
        "LR 80x24+0+0 [%1 20x24+0+0 | %2 19x24+21+0 | %3 39x24+41+0]"
    );
}

#[test]
fn a_full_size_split_before_puts_the_new_pane_first() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    l.split(0, LAYOUT_LEFTRIGHT, -1, SPAWN_FULLSIZE | SPAWN_BEFORE);
    assert_eq!(
        l.dump(),
        "LR 80x24+0+0 [%3 40x24+0+0 | TB 39x24+41+0 [%1 39x12+41+0 | %2 39x11+41+13]]"
    );

    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(0, LAYOUT_LEFTRIGHT, -1, SPAWN_FULLSIZE | SPAWN_BEFORE);
    assert_eq!(
        l.dump(),
        "LR 80x24+0+0 [%3 40x24+0+0 | %1 19x24+41+0 | %2 19x24+61+0]"
    );
}

#[test]
fn a_full_size_split_with_no_room_for_the_others_says_so() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    assert_eq!(l.split(0, LAYOUT_TOPBOTTOM, 20, SPAWN_FULLSIZE), None);
}

#[test]
fn a_floating_cell_hangs_off_the_root_and_is_left_out_of_the_sums() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let lc = (l.reference()).float_pane_layout(20, 10, 4, 2);
        let j = l.add_pane(20, 10);
        (l.reference()).assign_pane_layout(
            &lc,
            &crate::window::window_pane_find_by_id((*l.pane(j)).pane_id())
                .expect("the layout pane exists"),
            0,
        );
        assert_eq!(l.dump(), "TB 80x24+0+0 [%1 80x24+0+0 | %2* 20x10+4+2]");
        assert_eq!(
            window_pane_is_floating(
                &*l.w(),
                &{ (&(*l.pane(1))).observation() }
                    .expect("the pane allocation exists")
            ),
            1
        );
        assert_eq!(
            window_pane_is_floating(
                &*l.w(),
                &{ (&(*l.pane(0))).observation() }
                    .expect("the pane allocation exists")
            ),
            0
        );

        (l.reference()).fix_layout_offsets();
        assert_eq!(l.dump(), "TB 80x24+0+0 [%1 80x24+0+0 | %2* 20x10+4+2]");

        let second = (l.reference()).float_pane_layout(10, 5, 1, 1);
        let k = l.add_pane(10, 5);
        (l.reference()).assign_pane_layout(
            &second,
            &crate::window::window_pane_find_by_id((*l.pane(k)).pane_id())
                .expect("the layout pane exists"),
            1,
        );
        assert_eq!(
            l.dump(),
            "TB 80x24+0+0 [%1 80x24+0+0 | %2* 20x10+4+2 | %3* 10x5+1+1]"
        );
    }
}

#[test]
fn a_floating_root_is_left_out_of_resizing_and_offsets() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        (*(*l.w()).layout_root.as_deref_mut().unwrap()).flags |= LAYOUT_CELL_FLOATING;
        (l.reference()).resize_layout(100, 30);
        assert_eq!(l.dump(), "%1* 80x24+0+0");
        (l.reference()).fix_layout_offsets();
        assert_eq!(l.dump(), "%1* 80x24+0+0");
        (*(*l.w()).layout_root.as_deref_mut().unwrap()).flags &= !LAYOUT_CELL_FLOATING;
    }
}

#[test]
fn closing_a_floating_pane_leaves_the_others_alone() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let lc = (l.reference()).float_pane_layout(20, 10, 4, 2);
        let j = l.add_pane(20, 10);
        (l.reference()).assign_pane_layout(
            &lc,
            &crate::window::window_pane_find_by_id((*l.pane(j)).pane_id())
                .expect("the layout pane exists"),
            0,
        );
        l.close(j);
        assert_eq!(l.dump(), "%1 80x24+0+0");
    }
}

#[test]
fn the_z_index_follows_the_tree_from_left_to_right() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        (*l.w()).z_index.clear();
        (l.reference()).fix_layout_zindexes();
        let order: Vec<u_int> = (*l.w())
            .z_index
            .iter()
            .map(|pane| pane.id())
            .collect::<Vec<_>>();
        assert_eq!(order, vec![1, 3, 2]);
        let root = (*l.w()).layout_root.take();
        (l.reference()).fix_layout_zindexes();
        assert_eq!(
            (*l.w())
                .z_index
                .iter()
                .map(|pane| pane.id())
                .collect::<Vec<_>>(),
            order
        );
        (*l.w()).layout_root = root;
    }
}

#[test]
fn a_cell_can_be_turned_into_a_node_and_back_into_a_leaf() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let mut lc = layout_create_cell(None);
        layout_make_leaf(&mut *lc, &mut *l.pane(0));
        assert_eq!(
            (*lc).wp.as_ref().map(|pane| pane.id()),
            Some((*l.pane(0)).pane_id())
        );
        layout_make_node(&mut *lc, LAYOUT_TOPBOTTOM);
        assert_eq!((*lc).type_0, LAYOUT_TOPBOTTOM);
        assert!((*lc).wp.as_ref().map(|pane| pane.id()).is_none());
        layout_make_node(&mut *lc, LAYOUT_LEFTRIGHT);
        (l.reference()).init_layout(
            &crate::window::window_pane_find_by_id((*l.pane(0)).pane_id())
                .expect("the layout pane exists"),
        );
    }
}

#[test]
fn printing_a_cell_walks_the_whole_tree() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        layout_print_cell((*l.w()).layout_root.as_deref(), c"test", 0);
        layout_print_cell(None, c"test", 0);
        let mut lc = layout_create_cell(None);
        lc.type_0 = 99;
        layout_print_cell(Some(&lc), c"test", 0);
        lc.type_0 = LAYOUT_WINDOWPANE;
    }
}

#[test]
fn a_tiled_cell_comes_from_the_split_arguments() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let mut cause = CString::default();
        let mut item = Item::new().with_args(c"split-window -h");
        let lc = (l.window.reference()).tiled_layout_cell(
            &*item.ptr(),
            &*item.args(),
            &crate::window::window_pane_find_by_id((*l.pane(0)).pane_id())
                .expect("the layout pane exists"),
            0,
            &mut cause,
        );
        assert!(lc.is_some());
        assert!(cause.as_bytes().is_empty());
        let j = l.add_pane(1, 1);
        (l.reference()).assign_pane_layout(
            lc.as_ref().unwrap(),
            &crate::window::window_pane_find_by_id((*l.pane(j)).pane_id())
                .expect("the layout pane exists"),
            0,
        );
        assert_eq!(l.dump(), "LR 80x24+0+0 [%1 40x24+0+0 | %2 39x24+41+0]");
    }
}

#[test]
fn a_tiled_cell_reads_a_length_a_percentage_and_the_before_flag() {
    let _g = guard();
    for (line, want) in [
        (
            c"split-window -l 20",
            "TB 80x24+0+0 [%1 80x3+0+0 | %2 80x20+0+4]",
        ),
        (
            c"split-window -p 25",
            "TB 80x24+0+0 [%1 80x17+0+0 | %2 80x6+0+18]",
        ),
        (
            c"split-window -hbl 20",
            "LR 80x24+0+0 [%2 20x24+0+0 | %1 59x24+21+0]",
        ),
        (
            c"split-window -hfl 20",
            "LR 80x24+0+0 [%1 59x24+0+0 | %2 20x24+60+0]",
        ),
        (
            c"split-window -fp 25",
            "TB 80x24+0+0 [%1 80x17+0+0 | %2 80x6+0+18]",
        ),
    ] {
        let mut l = Layout::new(80, 24);
        unsafe {
            let mut cause = CString::default();
            let mut item = Item::new().with_args(line);
            let lc = (l.window.reference()).tiled_layout_cell(
                &*item.ptr(),
                &*item.args(),
                &crate::window::window_pane_find_by_id((*l.pane(0)).pane_id())
                    .expect("the layout pane exists"),
                0,
                &mut cause,
            );
            assert!(lc.is_some(), "{line:?}");
            let j = l.add_pane(1, 1);
            (l.reference()).assign_pane_layout(
                lc.as_ref().unwrap(),
                &crate::window::window_pane_find_by_id((*l.pane(j)).pane_id())
                    .expect("the layout pane exists"),
                0,
            );
            assert_eq!(l.dump(), want, "{line:?}");
        }
    }
}

#[test]
fn a_tiled_cell_says_what_is_wrong() {
    let _g = guard();
    let mut l = Layout::new(2, 2);
    unsafe {
        let mut cause = CString::default();
        let mut item = Item::new().with_args(c"split-window -h");
        assert!(
            (l.window.reference())
                .tiled_layout_cell(
                    &*item.ptr(),
                    &*item.args(),
                    &crate::window::window_pane_find_by_id((*l.pane(0)).pane_id())
                        .expect("the layout pane exists"),
                    0,
                    &mut cause
                )
                .is_none()
        );
        assert_eq!(cause.to_str().unwrap(), "no space for a new pane");

        let mut item = Item::new().with_args(c"split-window -l bad");
        let mut cause = CString::default();
        assert!(
            (l.window.reference())
                .tiled_layout_cell(
                    &*item.ptr(),
                    &*item.args(),
                    &crate::window::window_pane_find_by_id((*l.pane(0)).pane_id())
                        .expect("the layout pane exists"),
                    0,
                    &mut cause
                )
                .is_none()
        );
        assert_eq!(cause.to_str().unwrap(), "invalid tiled geometry");
    }
}

#[test]
fn a_floating_pane_cannot_be_split() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let lc = (l.reference()).float_pane_layout(20, 10, 4, 2);
        let j = l.add_pane(20, 10);
        (l.reference()).assign_pane_layout(
            &lc,
            &crate::window::window_pane_find_by_id((*l.pane(j)).pane_id())
                .expect("the layout pane exists"),
            0,
        );
        let mut cause = CString::default();
        let mut item = Item::new().with_args(c"split-window -h");
        assert!(
            (l.window.reference())
                .tiled_layout_cell(
                    &*item.ptr(),
                    &*item.args(),
                    &crate::window::window_pane_find_by_id((*l.pane(j)).pane_id())
                        .expect("the layout pane exists"),
                    0,
                    &mut cause
                )
                .is_none()
        );
        assert_eq!(cause.to_str().unwrap(), "can't split a floating pane");
    }
}

#[test]
fn a_floating_cell_takes_its_size_and_place_from_the_arguments() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let mut cause = None;
        let mut item = Item::new().with_args(c"new-pane -x 30 -y 8 -X 5 -Y 3");
        let lc =
            (l.window.reference()).floating_layout_cell(&*item.ptr(), &*item.args(), &mut cause);
        assert!(lc.is_some());
        let j = l.add_pane(1, 1);
        (l.reference()).assign_pane_layout(
            lc.as_ref().unwrap(),
            &crate::window::window_pane_find_by_id((*l.pane(j)).pane_id())
                .expect("the layout pane exists"),
            0,
        );
        assert_eq!(l.dump(), "TB 80x24+0+0 [%1 80x24+0+0 | %2* 30x8+5+3]");
    }
}

#[test]
fn a_floating_cell_walks_along_when_it_is_not_told_where_to_go() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let mut cause = None;
        let mut item = Item::new().with_args(c"new-pane");
        let first =
            (l.window.reference()).floating_layout_cell(&*item.ptr(), &*item.args(), &mut cause);
        let first = first.unwrap();
        let first = first.get((*l.w()).layout_root.as_deref().unwrap()).unwrap();
        assert_eq!(
            ((*first).sx, (*first).sy, (*first).xoff, (*first).yoff),
            (40, 6, 4, 2)
        );
        let second =
            (l.window.reference()).floating_layout_cell(&*item.ptr(), &*item.args(), &mut cause);
        let second = second.unwrap();
        let second = second
            .get((*l.w()).layout_root.as_deref().unwrap())
            .unwrap();
        assert_eq!(((*second).xoff, (*second).yoff), (8, 4));

        (*l.w()).set_last_new_pane(crate::window_dimensions::WindowCellPosition { x: 200, y: 200 });
        let third =
            (l.window.reference()).floating_layout_cell(&*item.ptr(), &*item.args(), &mut cause);
        let third = third.unwrap();
        let third = third.get((*l.w()).layout_root.as_deref().unwrap()).unwrap();
        assert_eq!(((*third).xoff, (*third).yoff), (4, 2));
    }
}

#[test]
fn a_floating_cell_stops_at_the_first_bad_argument() {
    let _g = guard();
    let l = Layout::new(80, 24);
    for line in [
        c"new-pane -x bad",
        c"new-pane -y bad",
        c"new-pane -X bad",
        c"new-pane -Y bad",
    ] {
        unsafe {
            let mut cause = None;
            let mut item = Item::new().with_args(line);
            assert!(
                (l.window.reference())
                    .floating_layout_cell(&*item.ptr(), &*item.args(), &mut cause)
                    .is_none(),
                "{line:?}"
            );
            assert!(cause.is_some(), "{line:?}");
        }
    }
}

#[test]
fn printing_a_cell_names_every_kind_of_node() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe { layout_print_cell((*l.w()).layout_root.as_deref(), c"test", 0) };
}

#[test]
fn a_cell_of_an_unknown_kind_is_freed_without_touching_children() {
    let _g = guard();
    {
        let mut lc = layout_create_cell(None);
        lc.type_0 = 99;
        layout_free_cell(Some(lc));
    }
}

#[test]
fn the_border_search_walks_past_a_gap_that_is_not_the_one_clicked() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        assert!(
            layout_search_by_border(&mut *root, 60, 5).is_some_and(|path| path
                .get(&*root)
                .is_some_and(|cell| cell.wp.as_ref().map(|pane| pane.id())
                    == Some((*l.pane(1)).pane_id())))
        );
        assert!(
            layout_search_by_border(&mut *root, 40, 5).is_some_and(|path| path
                .get(&*root)
                .is_some_and(|cell| cell.wp.as_ref().map(|pane| pane.id())
                    == Some((*l.pane(0)).pane_id())))
        );
    }

    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        assert!(
            layout_search_by_border(&mut *root, 5, 18).is_some_and(|path| path
                .get(&*root)
                .is_some_and(|cell| cell.wp.as_ref().map(|pane| pane.id())
                    == Some((*l.pane(1)).pane_id())))
        );
    }
}

#[test]
fn the_border_search_ignores_a_parent_of_an_unknown_kind() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        (*root).type_0 = 99;
        assert!(layout_search_by_border(&mut *root, 40, 5).is_none());
        (*root).type_0 = LAYOUT_LEFTRIGHT;
    }
}

#[test]
fn fixing_offsets_walks_into_a_node_under_a_left_right_one() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        (l.reference()).fix_layout_offsets();
        assert_eq!(
            l.dump(),
            "LR 80x24+0+0 [%1 40x24+0+0 | TB 39x24+41+0 [%2 39x12+41+0 | %3 39x11+41+13]]"
        );
    }
}

#[test]
fn a_cell_with_no_parent_that_is_not_the_root_is_neither_top_nor_bottom() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let mut lc = layout_create_cell(None);
        let lc = &raw mut *lc;
        assert_eq!(
            layout_add_horizontal_border(
                (*l.w()).layout_root.as_deref(),
                &mut *lc,
                PANE_STATUS_TOP
            ),
            0
        );
        assert_eq!(
            layout_add_horizontal_border(
                (*l.w()).layout_root.as_deref(),
                &mut *lc,
                PANE_STATUS_BOTTOM
            ),
            0
        );
        assert_eq!(
            layout_add_horizontal_border((*l.w()).layout_root.as_deref(), &mut *lc, 0),
            0
        );
    }
}

/// A floating cell is not an edge: the search for the top or bottom cell
/// walks past it to the first one that is really there.
#[test]
fn a_floating_cell_is_not_the_top_or_bottom_of_its_node() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    {
        assert_eq!(l.border(0, PANE_STATUS_TOP), 1);
        assert_eq!(l.border(1, PANE_STATUS_TOP), 0);
        assert_eq!(l.border(1, PANE_STATUS_BOTTOM), 1);

        l.set_floating(0, true);
        assert_eq!(l.border(1, PANE_STATUS_TOP), 1);
        assert_eq!(l.border(0, PANE_STATUS_TOP), 0);
        l.set_floating(0, false);

        l.set_floating(1, true);
        assert_eq!(l.border(0, PANE_STATUS_BOTTOM), 1);
        assert_eq!(l.border(1, PANE_STATUS_BOTTOM), 0);
        l.set_floating(1, false);
    }
}

#[test]
fn a_status_line_is_kept_out_of_the_room_a_pane_can_give_up() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        assert_eq!(
            (l.reference()).layout_resize_check(&mut *root, LAYOUT_TOPBOTTOM),
            21
        );
        with_status(&mut l, PANE_STATUS_TOP, |l| {
            let root = (*l.w()).layout_root.as_deref_mut().unwrap();
            assert_eq!(
                (l.reference()).layout_resize_check(&mut *root, LAYOUT_TOPBOTTOM),
                20
            );
        });
    }
}

#[test]
fn adjusting_a_cell_as_a_pane_only_changes_its_size() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        layout_resize_adjust(&l.reference(), &mut *root, LAYOUT_WINDOWPANE, 5);
        assert_eq!(l.dump(), "%1 80x29+0+0");
        layout_resize_adjust(&l.reference(), &mut *root, LAYOUT_WINDOWPANE, -5);
    }
}

#[test]
fn closing_the_first_pane_gives_its_room_to_the_next() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    l.close(0);
    assert_eq!(l.dump(), "LR 80x24+0+0 [%2 60x24+0+0 | %3 19x24+61+0]");
}

#[test]
fn a_node_that_folds_away_takes_its_place_among_its_siblings() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    assert_eq!(
        l.dump(),
        "LR 80x24+0+0 [%1 40x24+0+0 | TB 19x24+41+0 [%2 19x12+41+0 | %4 19x11+41+13] | %3 19x24+61+0]"
    );
    l.close(3);
    assert_eq!(
        l.dump(),
        "LR 80x24+0+0 [%1 40x24+0+0 | %2 19x24+41+0 | %3 19x24+61+0]"
    );
}

#[test]
fn a_window_with_no_room_left_is_only_ever_grown() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        (l.reference()).resize_layout(1, 1);
        assert_eq!(
            l.dump(),
            "LR 5x1+0+0 [%1 1x1+0+0 | %2 1x1+2+0 | %3 1x1+4+0]"
        );
        (l.reference()).resize_layout(1, 1);
        assert_eq!(
            l.dump(),
            "LR 5x1+0+0 [%1 1x1+0+0 | %2 1x1+2+0 | %3 1x1+4+0]"
        );
        (l.reference()).resize_layout(8, 4);
        assert_eq!(
            l.dump(),
            "LR 8x4+0+0 [%1 2x4+0+0 | %2 2x4+3+0 | %3 2x4+6+0]"
        );
    }
}

#[test]
fn a_tall_window_shrinks_only_as_far_as_its_rows_allow() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        (l.reference()).resize_layout(80, 1);
        assert_eq!(
            l.dump(),
            "TB 80x5+0+0 [%1 80x1+0+0 | %2 80x1+0+2 | %3 80x1+0+4]"
        );
        (l.reference()).resize_layout(80, 1);
        assert_eq!(
            l.dump(),
            "TB 80x5+0+0 [%1 80x1+0+0 | %2 80x1+0+2 | %3 80x1+0+4]"
        );
        (l.reference()).resize_layout(80, 8);
        assert_eq!(
            l.dump(),
            "TB 80x8+0+0 [%1 80x2+0+0 | %2 80x2+0+3 | %3 80x2+0+6]"
        );
    }
}

#[test]
fn a_pane_can_be_resized_to_a_size_top_to_bottom() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    l.resize_pane_to(0, LAYOUT_TOPBOTTOM, 6);
    assert_eq!(l.dump(), "TB 80x24+0+0 [%1 80x6+0+0 | %2 80x17+0+7]");
}

#[test]
fn a_resize_that_can_take_nothing_stops_where_it_is() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        (l.reference()).resize_layout(5, 24);
        assert_eq!(
            l.dump(),
            "LR 5x24+0+0 [%1 1x24+0+0 | %2 1x24+2+0 | %3 1x24+4+0]"
        );
        l.resize_pane(0, LAYOUT_LEFTRIGHT, 10, 1);
        assert_eq!(
            l.dump(),
            "LR 5x24+0+0 [%1 1x24+0+0 | %2 1x24+2+0 | %3 1x24+4+0]"
        );
        l.resize_pane(0, LAYOUT_LEFTRIGHT, -10, 1);
        assert_eq!(
            l.dump(),
            "LR 5x24+0+0 [%1 1x24+0+0 | %2 1x24+2+0 | %3 1x24+4+0]"
        );
    }
}

#[test]
fn growing_takes_room_from_behind_when_there_is_none_ahead() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        l.resize_pane_to(2, LAYOUT_LEFTRIGHT, 1);
        assert_eq!(
            l.dump(),
            "LR 80x24+0+0 [%1 40x24+0+0 | %2 37x24+41+0 | %3 1x24+79+0]"
        );
        let second = l.cell_path(1);
        (l.reference()).resize_layout_cell(&second, LAYOUT_LEFTRIGHT, 5, 1);
        assert_eq!(
            l.dump(),
            "LR 80x24+0+0 [%1 35x24+0+0 | %2 42x24+36+0 | %3 1x24+79+0]"
        );
    }
}

#[test]
fn shrinking_the_last_cell_of_a_node_has_nowhere_to_put_the_room() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        let last = l.cell_path(1);
        (l.reference()).resize_layout_cell(&last, LAYOUT_LEFTRIGHT, -5, 1);
        assert_eq!(l.dump(), "LR 80x24+0+0 [%1 40x24+0+0 | %2 39x24+41+0]");
    }
}

#[test]
fn a_new_pane_size_is_held_between_what_is_left_and_the_minimum() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        assert_eq!(
            (l.reference()).layout_new_pane_size(80, &mut *root, LAYOUT_LEFTRIGHT, 80, 1, 30),
            30
        );
        assert_eq!(
            (l.reference()).layout_new_pane_size(80, &mut *root, LAYOUT_LEFTRIGHT, 80, 2, 40),
            35
        );
        assert_eq!(
            (l.reference()).layout_new_pane_size(80, &mut *root, LAYOUT_LEFTRIGHT, 8, 2, 40),
            8
        );
        assert_eq!(
            (l.reference()).layout_new_pane_size(80, &mut *root, LAYOUT_LEFTRIGHT, 1, 2, 40),
            1
        );
    }

    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        assert_eq!(
            (l.reference()).layout_new_pane_size(24, &mut *root, LAYOUT_TOPBOTTOM, 24, 2, 12),
            7
        );
        assert_eq!(
            (l.reference()).layout_new_pane_size(24, &mut *root, LAYOUT_TOPBOTTOM, 2, 2, 12),
            2
        );
    }
}

#[test]
fn a_size_check_turns_down_what_will_not_fit() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        assert_eq!(
            (l.reference()).layout_set_size_check(&mut *root, LAYOUT_LEFTRIGHT, 40),
            1
        );
        assert_eq!(
            (l.reference()).layout_set_size_check(&mut *root, LAYOUT_LEFTRIGHT, 4),
            0
        );
        assert_eq!(
            (l.reference()).layout_set_size_check(&mut *root, LAYOUT_LEFTRIGHT, 5),
            1
        );
        assert_eq!(
            (l.reference()).layout_set_size_check(&mut *root, LAYOUT_LEFTRIGHT, 0),
            0
        );
        assert_eq!(
            (l.reference()).layout_set_size_check(&mut *root, LAYOUT_TOPBOTTOM, 24),
            1
        );
        assert_eq!(
            (l.reference()).layout_set_size_check(&l.cell(0).unwrap(), LAYOUT_LEFTRIGHT, 0),
            0
        );
    }

    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        assert_eq!(
            (l.reference()).layout_set_size_check(&mut *root, LAYOUT_TOPBOTTOM, 24),
            1
        );
        assert_eq!(
            (l.reference()).layout_set_size_check(&mut *root, LAYOUT_TOPBOTTOM, 4),
            0
        );
    }
}

#[test]
fn a_size_check_walks_into_nodes_of_the_other_kind() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        assert_eq!(
            (l.reference()).layout_set_size_check(&mut *root, LAYOUT_TOPBOTTOM, 24),
            1
        );
        assert_eq!(
            (l.reference()).layout_set_size_check(&mut *root, LAYOUT_TOPBOTTOM, 2),
            0
        );
    }
}

#[test]
fn a_split_before_into_a_node_of_the_same_kind_goes_in_front() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, SPAWN_BEFORE);
    assert_eq!(
        l.dump(),
        "LR 80x24+0+0 [%1 40x24+0+0 | %3 19x24+41+0 | %2 19x24+61+0]"
    );
}

#[test]
fn a_full_size_split_of_a_top_to_bottom_root_adds_a_row() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    assert_eq!(l.split(0, LAYOUT_TOPBOTTOM, -1, SPAWN_FULLSIZE), Some(2));
    assert_eq!(
        l.dump(),
        "TB 80x24+0+0 [%1 80x6+0+0 | %2 80x5+0+7 | %3 80x11+0+13]"
    );

    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    assert_eq!(
        l.split(0, LAYOUT_TOPBOTTOM, -1, SPAWN_FULLSIZE | SPAWN_BEFORE),
        Some(2)
    );
    assert_eq!(
        l.dump(),
        "TB 80x24+0+0 [%3 80x12+0+0 | %1 80x5+0+13 | %2 80x5+0+19]"
    );
}

#[test]
fn spreading_a_node_of_an_unknown_kind_answers_no() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        (*root).type_0 = 99;
        assert_eq!(
            (l.reference()).spread_layout_cell(&LayoutCellPath::root()),
            0
        );
        (*root).type_0 = LAYOUT_LEFTRIGHT;

        (*root).sx = 1;
        assert_eq!(
            (l.reference()).spread_layout_cell(&LayoutCellPath::root()),
            0
        );
        (*root).sx = 2;
        assert_eq!(
            (l.reference()).spread_layout_cell(&LayoutCellPath::root()),
            0
        );
        (*root).sx = 80;
    }
}

#[test]
fn spreading_a_window_already_shared_out_changes_nothing() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        assert_eq!(
            (l.reference()).spread_layout_cell(&LayoutCellPath::root()),
            0
        );
        l.spread_out(0);
        assert_eq!(l.dump(), "LR 80x24+0+0 [%1 40x24+0+0 | %2 39x24+41+0]");
    }
}

#[test]
fn spreading_out_with_a_status_line_leaves_room_for_it() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    {
        with_status(&mut l, PANE_STATUS_TOP, |l| {
            l.spread_out(0);
            assert_eq!(
                l.dump(),
                "TB 80x24+0+0 [%1 80x8+0+0 | %2 80x7+0+9 | %3 80x7+0+17]"
            );
        });
    }
}

#[test]
fn fixing_offsets_walks_into_a_node_under_a_top_to_bottom_one() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        (l.reference()).fix_layout_offsets();
        assert_eq!(
            l.dump(),
            "TB 80x24+0+0 [%1 80x12+0+0 | LR 80x11+0+13 [%2 40x11+0+13 | %3 39x11+41+13]]"
        );
    }
}

#[test]
fn a_new_pane_size_is_never_smaller_than_one_column() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        assert_eq!(
            (l.reference()).layout_new_pane_size(80, &mut *root, LAYOUT_LEFTRIGHT, 0, 2, 40),
            1
        );
    }
}

#[test]
fn spreading_a_node_with_no_room_for_its_borders_answers_no() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        (*root).sx = 1;
        assert_eq!(
            (l.reference()).spread_layout_cell(&LayoutCellPath::root()),
            0
        );
        (*root).sx = 3;
        assert_eq!(
            (l.reference()).spread_layout_cell(&LayoutCellPath::root()),
            0
        );
        (*root).sx = 80;
    }
}

#[test]
fn growing_walks_back_past_every_cell_with_no_room_to_give() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(2, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        l.resize_pane_to(3, LAYOUT_LEFTRIGHT, 1);
        l.resize_pane_to(1, LAYOUT_LEFTRIGHT, 1);
        assert_eq!(
            l.dump(),
            "LR 80x24+0+0 [%1 40x24+0+0 | %2 1x24+41+0 | %3 35x24+43+0 | %4 1x24+79+0]"
        );
        let third = l.cell_path(2);
        (l.reference()).resize_layout_cell(&third, LAYOUT_LEFTRIGHT, 5, 1);
        assert_eq!(
            l.dump(),
            "LR 80x24+0+0 [%1 35x24+0+0 | %2 1x24+36+0 | %3 40x24+38+0 | %4 1x24+79+0]"
        );
    }
}

#[test]
fn a_cell_under_a_side_by_side_node_is_both_top_and_bottom() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    {
        assert_eq!(l.border(0, PANE_STATUS_TOP), 1);
        assert_eq!(l.border(0, PANE_STATUS_BOTTOM), 1);
    }
}

#[test]
fn shrinking_walks_back_to_the_first_cell_with_room_to_give() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        l.resize_pane_to(1, LAYOUT_LEFTRIGHT, 1);
        assert_eq!(
            l.dump(),
            "LR 80x24+0+0 [%1 40x24+0+0 | %2 1x24+41+0 | %3 37x24+43+0]"
        );
        let second = l.cell_path(1);
        (l.reference()).resize_layout_cell(&second, LAYOUT_LEFTRIGHT, -5, 1);
        assert_eq!(
            l.dump(),
            "LR 80x24+0+0 [%1 35x24+0+0 | %2 1x24+36+0 | %3 42x24+38+0]"
        );
    }
}

#[test]
fn a_floating_cell_is_left_out_when_the_window_is_resized() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        let lc = (l.reference()).float_pane_layout(20, 10, 4, 2);
        let j = l.add_pane(20, 10);
        (l.reference()).assign_pane_layout(
            &lc,
            &crate::window::window_pane_find_by_id((*l.pane(j)).pane_id())
                .expect("the layout pane exists"),
            0,
        );
        (l.reference()).resize_layout(100, 30);
        assert_eq!(
            l.dump(),
            "TB 100x30+0+0 [%1 100x15+0+0 | %2 100x14+0+16 | %3* 20x10+4+2]"
        );
    }
}

#[test]
fn a_full_size_split_leaves_a_floating_cell_where_it_is() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        let lc = (l.reference()).float_pane_layout(20, 10, 4, 2);
        let j = l.add_pane(20, 10);
        (l.reference()).assign_pane_layout(
            &lc,
            &crate::window::window_pane_find_by_id((*l.pane(j)).pane_id())
                .expect("the layout pane exists"),
            0,
        );
    }
    assert_eq!(l.split(0, LAYOUT_TOPBOTTOM, -1, SPAWN_FULLSIZE), Some(3));
    assert_eq!(
        l.dump(),
        "TB 80x24+0+0 [%1 80x6+0+0 | %2 80x5+0+7 | %3* 20x10+4+2 | %4 80x11+0+13]"
    );
}

#[test]
fn a_cell_of_an_unknown_kind_prints_as_unknown() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        (*root).type_0 = 99;
        assert_eq!(l.dump(), "? 80x24+0+0");
        (*root).type_0 = LAYOUT_WINDOWPANE;
    }
}

#[test]
fn resizing_a_zoomed_layout_leaves_panes_in_the_saved_tree_unchanged() {
    let _g = guard();
    let mut layout = Layout::new(80, 24);
    layout.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        let hidden = (*layout.pane(1)).geometry();
        let id = (*layout.pane(0)).pane_id();
        assert_eq!(
            (layout.reference()).zoom(
                &crate::window::window_pane_find_by_id(id).expect("the selected pane exists")
            ),
            0
        );
        (layout.reference()).resize_layout(100, 30);
        let visible = (*layout.pane(0)).geometry();
        assert_eq!((visible.sx, visible.sy), (100, 30));
        assert_eq!((*layout.pane(1)).geometry(), hidden);
        assert_eq!((layout.reference()).unzoom(0), 0);
    }
}

#[test]
fn border_edges_follow_tree_ownership_instead_of_parent_back_pointers() {
    let _g = guard();
    let fixture = Window::new(90, "edges", 80, 24);
    let owner = fixture.reference();
    {
        let mut w = owner.as_window_mut();
        let mut root = layout_create_cell(None);
        root.type_0 = LAYOUT_LEFTRIGHT;
        root.cells.push(layout_create_cell(None));
        w.layout_root = Some(root);
        let foreign = layout_create_cell(w.layout_root.as_deref_mut());
        let child = &w.layout_root.as_deref().unwrap().cells[0];
        assert_eq!(
            layout_add_horizontal_border(w.layout_root.as_deref(), child, PANE_STATUS_TOP),
            1
        );
        assert_eq!(
            layout_add_horizontal_border(w.layout_root.as_deref(), child, PANE_STATUS_BOTTOM),
            1
        );
        assert_eq!(
            layout_add_horizontal_border(w.layout_root.as_deref(), &foreign, PANE_STATUS_TOP),
            0
        );
        assert_eq!(
            layout_add_horizontal_border(w.layout_root.as_deref(), &foreign, PANE_STATUS_BOTTOM),
            0
        );
    }
}

#[test]
fn growth_and_leaf_adjustments_work_before_a_pane_is_active() {
    let _g = guard();
    let mut w = Window::new(1, "unassigned", 10, 10);
    let mut root = layout_cell::default();
    root.type_0 = LAYOUT_LEFTRIGHT;
    root.sx = 10;
    for width in [4, 5] {
        let mut child = Box::new(layout_cell::default());
        child.type_0 = LAYOUT_WINDOWPANE;
        child.sx = width;
        root.cells.push(child);
    }
    unsafe {
        assert!((*w.ptr()).active_pane_id().is_none());
        layout_resize_adjust(w.handle(), &mut root, LAYOUT_LEFTRIGHT, 3);
        assert_eq!(root.sx, 13);
        assert_eq!((root.cells[0].sx, root.cells[1].sx), (6, 6));
        layout_resize_adjust(w.handle(), &mut root.cells[0], LAYOUT_LEFTRIGHT, -1);
        assert_eq!(root.cells[0].sx, 5);
    }
}

#[test]
fn unzoom_restores_geometry_from_the_owned_saved_tree() {
    fn rebuild(cell: &layout_cell, parent: Option<&mut layout_cell>) -> Box<layout_cell> {
        let mut copy = layout_create_cell(parent);
        copy.type_0 = cell.type_0;
        copy.flags = cell.flags;
        copy.wp = cell.wp.clone();
        (copy.sx, copy.sy, copy.xoff, copy.yoff) = (cell.sx, cell.sy, cell.xoff, cell.yoff);
        for child in &cell.cells {
            let child = rebuild(child, Some(&mut copy));
            copy.cells.push(child);
        }
        copy
    }
    let _g = guard();
    let mut layout = Layout::new(80, 24);
    layout.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    let owner = layout.window.reference();
    unsafe {
        let id = owner.as_window().panes[0].pane_id();
        assert_eq!(
            owner.zoom(
                &crate::window::window_pane_find_by_id(id).expect("the selected pane exists")
            ),
            0
        );
        let saved = owner.as_window_mut().saved_layout_root.take().unwrap();
        owner.as_window_mut().saved_layout_root = Some(rebuild(&saved, None));
        assert_eq!(owner.unzoom(0), 0);
        let w = owner.as_window();
        for pane in &w.panes {
            let (cell, _) =
                layout_cell_for_pane(w.layout_root.as_deref(), &pane.downgrade()).unwrap();
            let geometry = pane.as_pane().geometry();
            assert_eq!(
                (geometry.sx, geometry.sy, geometry.xoff, geometry.yoff),
                (cell.sx, cell.sy, cell.xoff, cell.yoff)
            );
        }
    }
}

#[test]
fn a_new_slot_survives_removing_a_pane_from_the_same_window() {
    let _g = guard();
    for count in [2, 3] {
        for source in 0..count {
            for target in 0..count {
                if source == target {
                    continue;
                }
                for axis in [LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM] {
                    for flags in [
                        0,
                        SPAWN_BEFORE,
                        SPAWN_FULLSIZE,
                        SPAWN_FULLSIZE | SPAWN_BEFORE,
                    ] {
                        let mut l = Layout::new(100, 40);
                        for _ in 1..count {
                            l.split(0, LAYOUT_LEFTRIGHT, -1, 0).unwrap();
                        }
                        unsafe {
                            let source_id = (*l.pane(source)).pane_id();
                            let target_id = (*l.pane(target)).pane_id();
                            let w = &mut *l.w();
                            let mut slot = (l.reference())
                                .split_pane_layout(
                                    &crate::window::window_pane_find_by_id(target_id)
                                        .expect("the layout pane exists"),
                                    axis,
                                    -1,
                                    flags,
                                )
                                .unwrap();
                            (l.reference()).layout_close_pane_with_slot(
                                &crate::window::window_pane_find_by_id(source_id)
                                    .expect("the layout pane exists"),
                                Some(&mut slot),
                            );
                            assert!(
                                slot.get(w.layout_root.as_deref().unwrap())
                                    .unwrap()
                                    .wp
                                    .as_ref()
                                    .map(|pane| pane.id())
                                    .is_none()
                            );
                            (l.reference()).assign_pane_layout(
                                &slot,
                                &crate::window::window_pane_find_by_id(source_id)
                                    .expect("the layout pane exists"),
                                0,
                            );
                            for pane in &w.panes {
                                assert!(
                                    layout_cell_for_pane(
                                        w.layout_root.as_deref(),
                                        &pane.downgrade()
                                    )
                                    .is_some()
                                );
                            }
                            assert_eq!(
                                layout_count_cells(w.layout_root.as_deref_mut().unwrap()),
                                count as u_int
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Adds `change` to the size of `lc`, distributing it among its descendants.
pub(crate) unsafe fn layout_resize_adjust(
    w: &WindowRef,
    lc: &mut layout_cell,
    axis: layout_type,
    change: c_int,
) {
    unsafe { LayoutResizeLimits::for_adjustment(w, lc, axis, change).adjust(lc, axis, change) }
}
