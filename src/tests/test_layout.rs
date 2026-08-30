use super::*;
use crate::cmd::cmd_get_args;
use crate::layout::layout_root_ptr;
use crate::options::options_set_number;
use crate::tests::test_fixtures::{Item, Pane, Window, globals};
use crate::window::PANE_SCROLLBARS_ALWAYS;
use crate::window::window_get_active;
use crate::window::{window_pane_is_floating, window_pane_show_scrollbar};
use ::core::ffi::c_int;
use ::core::ptr::null_mut;
use ::std::ffi::CString;
use ::std::sync::MutexGuard;

/// A window carrying a layout tree and the panes that hang off it. The
/// window and its panes are the server-free fixtures; the tree is real, and
/// is freed before the panes go.
struct Layout {
    window: Window,
    panes: Vec<Pane>,
    next_id: u_int,
}

impl Layout {
    /// A window of `sx` by `sy` with one pane filling it, as `layout_init`
    /// leaves a freshly created window.
    fn new(sx: u_int, sy: u_int) -> Layout {
        let mut l = Layout {
            window: Window::new(1, "layout", sx, sy),
            panes: Vec::new(),
            next_id: 0,
        };
        l.add_pane(sx, sy);
        unsafe { layout_init(l.w(), l.pane(0)) };
        l
    }

    fn w(&mut self) -> *mut window {
        self.window.ptr()
    }

    fn pane(&mut self, i: usize) -> *mut window_pane {
        self.panes[i].ptr()
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
            let wp = self.pane(i);
            let lc = layout_split_pane(wp, type_0, size, flags);
            if lc.is_null() {
                return None;
            }
            let j = self.add_pane(1, 1);
            layout_assign_pane(lc, self.pane(j), 0);
            Some(j)
        }
    }

    /// The tree as one line: each node is its type, size and offset, with
    /// its children in brackets.
    fn dump(&mut self) -> String {
        unsafe { dump_cell(layout_root_ptr(&(*self.w()).layout_root)) }
    }

    /// The sizes and offsets the panes themselves were given.
    fn panes(&mut self) -> Vec<String> {
        unsafe {
            let mut out = Vec::new();
            let w = self.w();
            let mut wp = window_panes_first(w);
            while !wp.is_null() {
                out.push(format!(
                    "%{} {}x{}+{}+{}",
                    (*wp).id,
                    (*wp).sx,
                    (*wp).sy,
                    (*wp).xoff,
                    (*wp).yoff
                ));
                wp = window_panes_next(w, wp);
            }
            out
        }
    }
}

impl Drop for Layout {
    fn drop(&mut self) {
        unsafe { layout_free(self.window.ptr()) };
    }
}

unsafe fn dump_cell(lc: *mut layout_cell) -> String {
    unsafe {
        if lc.is_null() {
            return "-".to_string();
        }
        let here = format!("{}x{}+{}+{}", (*lc).sx, (*lc).sy, (*lc).xoff, (*lc).yoff);
        let floating = if (*lc).flags & LAYOUT_CELL_FLOATING != 0 {
            "*"
        } else {
            ""
        };
        match (*lc).type_0 {
            LAYOUT_WINDOWPANE => format!("%{}{floating} {here}", (*lc).wp_id.unwrap_or(u_int::MAX)),
            LAYOUT_LEFTRIGHT | LAYOUT_TOPBOTTOM => {
                let kids: Vec<String> = crate::list::foreach_owned(&raw mut (*lc).cells)
                    .map(|child| dump_cell(child))
                    .collect();
                let name = if (*lc).type_0 == LAYOUT_LEFTRIGHT {
                    "LR"
                } else {
                    "TB"
                };
                format!("{name}{floating} {here} [{}]", kids.join(" | "))
            }
            _ => format!("?{floating} {here}"),
        }
    }
}

/// Runs `body` with `pane-border-status` set to `status` on the window's
/// own options.
fn with_status(l: &mut Layout, status: c_int, body: impl FnOnce(&mut Layout)) {
    unsafe {
        options_set_number(
            options_ptr(&(*l.w()).options),
            c"pane-border-status".as_ptr(),
            status as ::core::ffi::c_longlong,
        );
        body(l);
        options_set_number(
            options_ptr(&(*l.w()).options),
            c"pane-border-status".as_ptr(),
            0,
        );
    }
}

fn guard() -> MutexGuard<'static, ()> {
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
            layout_count_cells(layout_root_ptr(&(*l.w()).layout_root)),
            1
        );
        assert_eq!(
            (*l.pane(0)).layout_cell,
            layout_root_ptr(&(*l.w()).layout_root)
        );
    }
}

#[test]
fn a_fresh_cell_starts_at_the_largest_size_there_is() {
    let _g = guard();
    unsafe {
        let lc = layout_create_cell(null_mut::<layout_cell>());
        let lc = &raw const *lc as *mut layout_cell;
        assert_eq!((*lc).type_0, LAYOUT_WINDOWPANE);
        assert_eq!((*lc).flags, 0);
        assert!((*lc).parent.is_null());
        assert_eq!((*lc).sx, UINT_MAX as u_int);
        assert_eq!((*lc).sy, UINT_MAX as u_int);
        assert_eq!((*lc).xoff, INT_MAX);
        assert_eq!((*lc).yoff, INT_MAX);
        assert!((*lc).wp_id.is_none());
        layout_free_cell(null_mut(), None);
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
            layout_count_cells(layout_root_ptr(&(*l.w()).layout_root)),
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
fn closing_a_pane_gives_its_room_back_to_its_neighbour() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe { layout_close_pane(l.pane(1)) };
    assert_eq!(l.dump(), "%1 80x24+0+0");
    unsafe { assert!((*l.pane(1)).layout_cell.is_null()) };

    unsafe { layout_close_pane(l.pane(1)) };
    assert_eq!(l.dump(), "%1 80x24+0+0");
}

#[test]
fn closing_the_only_pane_leaves_no_tree() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe { layout_close_pane(l.pane(0)) };
    assert_eq!(l.dump(), "-");
}

#[test]
fn closing_one_of_three_leaves_the_node_in_place() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe { layout_close_pane(l.pane(1)) };
    assert_eq!(l.dump(), "LR 80x24+0+0 [%1 60x24+0+0 | %3 19x24+61+0]");
}

#[test]
fn closing_a_nested_pane_folds_the_node_away() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe { layout_close_pane(l.pane(2)) };
    assert_eq!(l.dump(), "LR 80x24+0+0 [%1 40x24+0+0 | %2 39x24+41+0]");
}

#[test]
fn resizing_the_window_shares_the_change_out() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe { layout_resize(l.w(), 100, 30) };
    assert_eq!(l.dump(), "LR 100x30+0+0 [%1 50x30+0+0 | %2 49x30+51+0]");
    unsafe { layout_resize(l.w(), 40, 12) };
    assert_eq!(l.dump(), "LR 40x12+0+0 [%1 20x12+0+0 | %2 19x12+21+0]");
}

#[test]
fn a_window_cannot_shrink_past_what_its_panes_need() {
    let _g = guard();
    let mut l = Layout::new(20, 10);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe { layout_resize(l.w(), 1, 1) };
    assert_eq!(
        l.dump(),
        "LR 5x1+0+0 [%1 1x1+0+0 | %2 1x1+2+0 | %3 1x1+4+0]"
    );
}

#[test]
fn resizing_a_single_pane_window_only_grows_it() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe { layout_resize(l.w(), 100, 30) };
    assert_eq!(l.dump(), "%1 100x30+0+0");
    unsafe { layout_resize(l.w(), 40, 12) };
    assert_eq!(l.dump(), "%1 40x12+0+0");
}

#[test]
fn a_pane_can_be_resized_by_hand_in_either_direction() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe { layout_resize_pane(l.pane(0), LAYOUT_LEFTRIGHT, 10, 1) };
    assert_eq!(l.dump(), "LR 80x24+0+0 [%1 50x24+0+0 | %2 29x24+51+0]");
    unsafe { layout_resize_pane(l.pane(0), LAYOUT_LEFTRIGHT, -20, 1) };
    assert_eq!(l.dump(), "LR 80x24+0+0 [%1 30x24+0+0 | %2 49x24+31+0]");
}

#[test]
fn resizing_the_last_pane_moves_the_border_before_it() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe { layout_resize_pane(l.pane(1), LAYOUT_LEFTRIGHT, 10, 1) };
    assert_eq!(l.dump(), "LR 80x24+0+0 [%1 50x24+0+0 | %2 29x24+51+0]");
}

#[test]
fn resizing_across_a_kind_the_pane_is_not_in_does_nothing() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    let before = l.dump();
    unsafe { layout_resize_pane(l.pane(0), LAYOUT_TOPBOTTOM, 5, 1) };
    assert_eq!(l.dump(), before);
    unsafe { layout_resize_pane_to(l.pane(0), LAYOUT_TOPBOTTOM, 5) };
    assert_eq!(l.dump(), before);
}

#[test]
fn a_pane_can_be_resized_to_a_size() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe { layout_resize_pane_to(l.pane(0), LAYOUT_LEFTRIGHT, 20) };
    assert_eq!(l.dump(), "LR 80x24+0+0 [%1 20x24+0+0 | %2 59x24+21+0]");
    unsafe { layout_resize_pane_to(l.pane(1), LAYOUT_LEFTRIGHT, 20) };
    assert_eq!(l.dump(), "LR 80x24+0+0 [%1 59x24+0+0 | %2 20x24+60+0]");
}

#[test]
fn growing_without_the_opposite_side_stops_at_the_last_pane() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe { layout_resize_pane(l.pane(1), LAYOUT_LEFTRIGHT, 10, 0) };
    assert_eq!(l.dump(), "LR 80x24+0+0 [%1 50x24+0+0 | %2 29x24+51+0]");
}

#[test]
fn spreading_out_gives_every_pane_the_same_room() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe { layout_spread_out(l.pane(0)) };
    assert_eq!(
        l.dump(),
        "LR 80x24+0+0 [%1 26x24+0+0 | %2 26x24+27+0 | %3 26x24+54+0]"
    );
    unsafe { layout_spread_out(l.pane(0)) };
    assert_eq!(
        l.dump(),
        "LR 80x24+0+0 [%1 26x24+0+0 | %2 26x24+27+0 | %3 26x24+54+0]"
    );
}

#[test]
fn spreading_out_a_single_pane_does_nothing() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe { layout_spread_out(l.pane(0)) };
    assert_eq!(l.dump(), "%1 80x24+0+0");
}

#[test]
fn spreading_a_cell_that_cannot_be_shared_answers_no() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        assert_eq!(
            layout_spread_cell(l.w(), layout_root_ptr(&(*l.w()).layout_root)),
            0
        );
    }
    let mut l = Layout::new(4, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        assert_eq!(
            layout_spread_cell(l.w(), layout_root_ptr(&(*l.w()).layout_root)),
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
    unsafe { layout_spread_out(l.pane(0)) };
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
        let root = layout_root_ptr(&(*l.w()).layout_root);
        assert_eq!(
            layout_search_by_border(root, 40, 5),
            (*l.pane(0)).layout_cell
        );
        assert!(layout_search_by_border(root, 0, 0).is_null());
        assert!(layout_search_by_border(root, 79, 0).is_null());
    }
}

#[test]
fn the_border_search_works_top_to_bottom_and_through_nodes() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        let root = layout_root_ptr(&(*l.w()).layout_root);
        assert_eq!(
            layout_search_by_border(root, 5, 12),
            (*l.pane(0)).layout_cell
        );
        assert!(layout_search_by_border(root, 5, 0).is_null());
    }

    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        let root = layout_root_ptr(&(*l.w()).layout_root);
        assert_eq!(
            layout_search_by_border(root, 50, 12),
            (*l.pane(1)).layout_cell
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
            layout_fix_panes(l.w(), null_mut::<window_pane>());
            assert_eq!(l.panes(), vec!["%1 80x11+0+1", "%2 80x11+0+13"]);
        });
        with_status(&mut l, PANE_STATUS_BOTTOM, |l| {
            layout_fix_panes(l.w(), null_mut::<window_pane>());
            assert_eq!(l.panes(), vec!["%1 80x12+0+0", "%2 80x10+0+13"]);
        });
        layout_fix_panes(l.w(), null_mut::<window_pane>());
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
            layout_fix_panes(l.w(), skip);
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
        (*l.w()).sb = PANE_SCROLLBARS_ALWAYS;
        (*l.pane(0)).scrollbar_style.width = 2;
        (*l.pane(0)).scrollbar_style.pad = 1;
        assert_eq!(window_pane_show_scrollbar(l.pane(0)), 1);

        layout_fix_panes(l.w(), null_mut::<window_pane>());
        assert_eq!(l.panes(), vec!["%1 77x24+0+0"]);

        (*l.w()).sb_pos = PANE_SCROLLBARS_LEFT;
        layout_fix_panes(l.w(), null_mut::<window_pane>());
        assert_eq!(l.panes(), vec!["%1 77x24+3+0"]);
    }
}

#[test]
fn a_scrollbar_wider_than_the_pane_leaves_one_column() {
    let _g = guard();
    let mut l = Layout::new(4, 24);
    unsafe {
        (*l.w()).sb = PANE_SCROLLBARS_ALWAYS;
        (*l.pane(0)).scrollbar_style.width = 8;
        (*l.pane(0)).scrollbar_style.pad = -1;

        layout_fix_panes(l.w(), null_mut::<window_pane>());
        assert_eq!(l.panes(), vec!["%1 1x24+0+0"]);

        (*l.w()).sb_pos = PANE_SCROLLBARS_LEFT;
        layout_fix_panes(l.w(), null_mut::<window_pane>());
        assert_eq!(l.panes(), vec!["%1 1x24+3+0"]);

        (*l.pane(0)).scrollbar_style.width = 0;
        (*l.pane(0)).scrollbar_style.pad = 0;
        layout_fix_panes(l.w(), null_mut::<window_pane>());
        assert_eq!(l.panes(), vec!["%1 3x24+1+0"]);
    }
}

#[test]
fn a_scrollbar_makes_a_side_by_side_split_need_more_room() {
    let _g = guard();
    let mut l = Layout::new(5, 24);
    unsafe {
        (*l.w()).sb = PANE_SCROLLBARS_ALWAYS;
        (*l.pane(0)).scrollbar_style.width = 3;
        (*l.pane(0)).scrollbar_style.pad = 1;
    }
    assert_eq!(l.split(0, LAYOUT_LEFTRIGHT, -1, 0), None);
}

#[test]
fn a_scrollbar_is_kept_out_of_the_room_a_pane_can_give_up() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        (*l.w()).sb = PANE_SCROLLBARS_ALWAYS;
        (*window_get_active(l.w())).scrollbar_style.width = 3;
        (*window_get_active(l.w())).scrollbar_style.pad = 1;
        let root = layout_root_ptr(&(*l.w()).layout_root);
        assert_eq!(layout_resize_check(l.w(), root, LAYOUT_LEFTRIGHT), 69);
        (*l.w()).sb = PANE_SCROLLBARS_OFF;
        assert_eq!(layout_resize_check(l.w(), root, LAYOUT_LEFTRIGHT), 77);
        assert_eq!(layout_resize_check(l.w(), root, LAYOUT_TOPBOTTOM), 23);
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
        let lc = layout_floating_pane(l.w(), 20, 10, 4, 2);
        let j = l.add_pane(20, 10);
        layout_assign_pane(lc, l.pane(j), 0);
        assert_eq!(l.dump(), "TB 80x24+0+0 [%1 80x24+0+0 | %2* 20x10+4+2]");
        assert_eq!(window_pane_is_floating(l.pane(1)), 1);
        assert_eq!(window_pane_is_floating(l.pane(0)), 0);

        layout_fix_offsets(l.w());
        assert_eq!(l.dump(), "TB 80x24+0+0 [%1 80x24+0+0 | %2* 20x10+4+2]");

        let second = layout_floating_pane(l.w(), 10, 5, 1, 1);
        let k = l.add_pane(10, 5);
        layout_assign_pane(second, l.pane(k), 1);
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
        (*layout_root_ptr(&(*l.w()).layout_root)).flags |= LAYOUT_CELL_FLOATING;
        layout_resize(l.w(), 100, 30);
        assert_eq!(l.dump(), "%1* 80x24+0+0");
        layout_fix_offsets(l.w());
        assert_eq!(l.dump(), "%1* 80x24+0+0");
        (*layout_root_ptr(&(*l.w()).layout_root)).flags &= !LAYOUT_CELL_FLOATING;
    }
}

#[test]
fn closing_a_floating_pane_leaves_the_others_alone() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let lc = layout_floating_pane(l.w(), 20, 10, 4, 2);
        let j = l.add_pane(20, 10);
        layout_assign_pane(lc, l.pane(j), 0);
        layout_close_pane(l.pane(j));
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
        layout_fix_zindexes(l.w(), layout_root_ptr(&(*l.w()).layout_root));
        let order: Vec<u_int> = (*l.w()).z_index.clone();
        assert_eq!(order, vec![1, 3, 2]);
        layout_fix_zindexes(l.w(), null_mut::<layout_cell>());
    }
}

#[test]
fn a_cell_can_be_turned_into_a_node_and_back_into_a_leaf() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let lc = layout_create_cell(null_mut::<layout_cell>());
        let lc = &raw const *lc as *mut layout_cell;
        layout_make_leaf(lc, l.pane(0));
        assert_eq!((*l.pane(0)).layout_cell, lc);
        layout_make_node(l.w(), lc, LAYOUT_TOPBOTTOM);
        assert_eq!((*lc).type_0, LAYOUT_TOPBOTTOM);
        assert!((*lc).wp_id.is_none());
        assert!((*l.pane(0)).layout_cell.is_null());
        layout_make_node(l.w(), lc, LAYOUT_LEFTRIGHT);
        layout_init(l.w(), l.pane(0));
    }
}

#[test]
fn printing_a_cell_walks_the_whole_tree() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        layout_print_cell(layout_root_ptr(&(*l.w()).layout_root), c"test".as_ptr(), 0);
        layout_print_cell(null_mut::<layout_cell>(), c"test".as_ptr(), 0);
        let mut lc = layout_create_cell(null_mut::<layout_cell>());
        lc.type_0 = 99;
        layout_print_cell(&raw mut *lc, c"test".as_ptr(), 0);
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
        let lc = layout_get_tiled_cell(
            item.ptr(),
            cmd_get_args(&*item.cmd()),
            l.w(),
            l.pane(0),
            0,
            &mut cause,
        );
        assert!(!lc.is_null());
        assert!(cause.as_bytes().is_empty());
        let j = l.add_pane(1, 1);
        layout_assign_pane(lc, l.pane(j), 0);
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
            let lc = layout_get_tiled_cell(
                item.ptr(),
                cmd_get_args(&*item.cmd()),
                l.w(),
                l.pane(0),
                0,
                &mut cause,
            );
            assert!(!lc.is_null(), "{line:?}");
            let j = l.add_pane(1, 1);
            layout_assign_pane(lc, l.pane(j), 0);
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
            layout_get_tiled_cell(
                item.ptr(),
                cmd_get_args(&*item.cmd()),
                l.w(),
                l.pane(0),
                0,
                &mut cause
            )
            .is_null()
        );
        assert_eq!(cause.to_str().unwrap(), "no space for a new pane");

        let mut item = Item::new().with_args(c"split-window -l bad");
        let mut cause = CString::default();
        assert!(
            layout_get_tiled_cell(
                item.ptr(),
                cmd_get_args(&*item.cmd()),
                l.w(),
                l.pane(0),
                0,
                &mut cause
            )
            .is_null()
        );
        assert_eq!(cause.to_str().unwrap(), "invalid tiled geometry");
    }
}

#[test]
fn a_floating_pane_cannot_be_split() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let lc = layout_floating_pane(l.w(), 20, 10, 4, 2);
        let j = l.add_pane(20, 10);
        layout_assign_pane(lc, l.pane(j), 0);
        let mut cause = CString::default();
        let mut item = Item::new().with_args(c"split-window -h");
        assert!(
            layout_get_tiled_cell(
                item.ptr(),
                cmd_get_args(&*item.cmd()),
                l.w(),
                l.pane(j),
                0,
                &mut cause
            )
            .is_null()
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
        let lc = layout_get_floating_cell(
            item.ptr(),
            cmd_get_args(&*item.cmd()),
            l.w(),
            l.pane(0),
            &mut cause,
        );
        assert!(!lc.is_null());
        let j = l.add_pane(1, 1);
        layout_assign_pane(lc, l.pane(j), 0);
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
        let first = layout_get_floating_cell(
            item.ptr(),
            cmd_get_args(&*item.cmd()),
            l.w(),
            l.pane(0),
            &mut cause,
        );
        assert_eq!(
            ((*first).sx, (*first).sy, (*first).xoff, (*first).yoff),
            (40, 6, 4, 2)
        );
        let second = layout_get_floating_cell(
            item.ptr(),
            cmd_get_args(&*item.cmd()),
            l.w(),
            l.pane(0),
            &mut cause,
        );
        assert_eq!(((*second).xoff, (*second).yoff), (8, 4));

        (*l.w()).last_new_pane_x = 200;
        (*l.w()).last_new_pane_y = 200;
        let third = layout_get_floating_cell(
            item.ptr(),
            cmd_get_args(&*item.cmd()),
            l.w(),
            l.pane(0),
            &mut cause,
        );
        assert_eq!(((*third).xoff, (*third).yoff), (4, 2));
    }
}

#[test]
fn a_floating_cell_stops_at_the_first_bad_argument() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
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
                layout_get_floating_cell(
                    item.ptr(),
                    cmd_get_args(&*item.cmd()),
                    l.w(),
                    l.pane(0),
                    &mut cause
                )
                .is_null(),
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
    unsafe { layout_print_cell(layout_root_ptr(&(*l.w()).layout_root), c"test".as_ptr(), 0) };
}

#[test]
fn a_cell_of_an_unknown_kind_is_freed_without_touching_children() {
    let _g = guard();
    unsafe {
        let mut lc = layout_create_cell(null_mut::<layout_cell>());
        lc.type_0 = 99;
        layout_free_cell(null_mut(), Some(lc));
    }
}

#[test]
fn the_border_search_walks_past_a_gap_that_is_not_the_one_clicked() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        let root = layout_root_ptr(&(*l.w()).layout_root);
        assert_eq!(
            layout_search_by_border(root, 60, 5),
            (*l.pane(1)).layout_cell
        );
        assert_eq!(
            layout_search_by_border(root, 40, 5),
            (*l.pane(0)).layout_cell
        );
    }

    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        let root = layout_root_ptr(&(*l.w()).layout_root);
        assert_eq!(
            layout_search_by_border(root, 5, 18),
            (*l.pane(1)).layout_cell
        );
    }
}

#[test]
fn the_border_search_ignores_a_parent_of_an_unknown_kind() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        let root = layout_root_ptr(&(*l.w()).layout_root);
        (*root).type_0 = 99;
        assert!(layout_search_by_border(root, 40, 5).is_null());
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
        layout_fix_offsets(l.w());
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
        let mut lc = layout_create_cell(null_mut::<layout_cell>());
        let lc = &raw mut *lc;
        assert_eq!(layout_add_horizontal_border(l.w(), lc, PANE_STATUS_TOP), 0);
        assert_eq!(
            layout_add_horizontal_border(l.w(), lc, PANE_STATUS_BOTTOM),
            0
        );
        assert_eq!(layout_add_horizontal_border(l.w(), lc, 0), 0);
    }
}

/// A floating cell is not an edge: the search for the top or bottom cell
/// walks past it to the first one that is really there.
#[test]
fn a_floating_cell_is_not_the_top_or_bottom_of_its_node() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        let first = (*l.pane(0)).layout_cell;
        let second = (*l.pane(1)).layout_cell;
        assert_eq!(
            layout_add_horizontal_border(l.w(), first, PANE_STATUS_TOP),
            1
        );
        assert_eq!(
            layout_add_horizontal_border(l.w(), second, PANE_STATUS_TOP),
            0
        );
        assert_eq!(
            layout_add_horizontal_border(l.w(), second, PANE_STATUS_BOTTOM),
            1
        );

        (*first).flags |= LAYOUT_CELL_FLOATING;
        assert_eq!(
            layout_add_horizontal_border(l.w(), second, PANE_STATUS_TOP),
            1
        );
        assert_eq!(
            layout_add_horizontal_border(l.w(), first, PANE_STATUS_TOP),
            0
        );
        (*first).flags &= !LAYOUT_CELL_FLOATING;

        (*second).flags |= LAYOUT_CELL_FLOATING;
        assert_eq!(
            layout_add_horizontal_border(l.w(), first, PANE_STATUS_BOTTOM),
            1
        );
        assert_eq!(
            layout_add_horizontal_border(l.w(), second, PANE_STATUS_BOTTOM),
            0
        );
        (*second).flags &= !LAYOUT_CELL_FLOATING;
    }
}

#[test]
fn a_status_line_is_kept_out_of_the_room_a_pane_can_give_up() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        let root = layout_root_ptr(&(*l.w()).layout_root);
        assert_eq!(layout_resize_check(l.w(), root, LAYOUT_TOPBOTTOM), 21);
        with_status(&mut l, PANE_STATUS_TOP, |l| {
            let root = layout_root_ptr(&(*l.w()).layout_root);
            assert_eq!(layout_resize_check(l.w(), root, LAYOUT_TOPBOTTOM), 20);
        });
    }
}

#[test]
fn adjusting_a_cell_as_a_pane_only_changes_its_size() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let root = layout_root_ptr(&(*l.w()).layout_root);
        layout_resize_adjust(l.w(), root, LAYOUT_WINDOWPANE, 5);
        assert_eq!(l.dump(), "%1 80x29+0+0");
        layout_resize_adjust(l.w(), root, LAYOUT_WINDOWPANE, -5);
    }
}

#[test]
fn closing_the_first_pane_gives_its_room_to_the_next() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe { layout_close_pane(l.pane(0)) };
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
    unsafe { layout_close_pane(l.pane(3)) };
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
        layout_resize(l.w(), 1, 1);
        assert_eq!(
            l.dump(),
            "LR 5x1+0+0 [%1 1x1+0+0 | %2 1x1+2+0 | %3 1x1+4+0]"
        );
        layout_resize(l.w(), 1, 1);
        assert_eq!(
            l.dump(),
            "LR 5x1+0+0 [%1 1x1+0+0 | %2 1x1+2+0 | %3 1x1+4+0]"
        );
        layout_resize(l.w(), 8, 4);
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
        layout_resize(l.w(), 80, 1);
        assert_eq!(
            l.dump(),
            "TB 80x5+0+0 [%1 80x1+0+0 | %2 80x1+0+2 | %3 80x1+0+4]"
        );
        layout_resize(l.w(), 80, 1);
        assert_eq!(
            l.dump(),
            "TB 80x5+0+0 [%1 80x1+0+0 | %2 80x1+0+2 | %3 80x1+0+4]"
        );
        layout_resize(l.w(), 80, 8);
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
    unsafe { layout_resize_pane_to(l.pane(0), LAYOUT_TOPBOTTOM, 6) };
    assert_eq!(l.dump(), "TB 80x24+0+0 [%1 80x6+0+0 | %2 80x17+0+7]");
}

#[test]
fn a_resize_that_can_take_nothing_stops_where_it_is() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        layout_resize(l.w(), 5, 24);
        assert_eq!(
            l.dump(),
            "LR 5x24+0+0 [%1 1x24+0+0 | %2 1x24+2+0 | %3 1x24+4+0]"
        );
        layout_resize_pane(l.pane(0), LAYOUT_LEFTRIGHT, 10, 1);
        assert_eq!(
            l.dump(),
            "LR 5x24+0+0 [%1 1x24+0+0 | %2 1x24+2+0 | %3 1x24+4+0]"
        );
        layout_resize_pane(l.pane(0), LAYOUT_LEFTRIGHT, -10, 1);
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
        layout_resize_pane_to(l.pane(2), LAYOUT_LEFTRIGHT, 1);
        assert_eq!(
            l.dump(),
            "LR 80x24+0+0 [%1 40x24+0+0 | %2 37x24+41+0 | %3 1x24+79+0]"
        );
        let second = (*l.pane(1)).layout_cell;
        layout_resize_layout(l.w(), second, LAYOUT_LEFTRIGHT, 5, 1);
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
        let last = (*l.pane(1)).layout_cell;
        layout_resize_layout(l.w(), last, LAYOUT_LEFTRIGHT, -5, 1);
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
        let root = layout_root_ptr(&(*l.w()).layout_root);
        assert_eq!(
            layout_new_pane_size(l.w(), 80, root, LAYOUT_LEFTRIGHT, 80, 1, 30),
            30
        );
        assert_eq!(
            layout_new_pane_size(l.w(), 80, root, LAYOUT_LEFTRIGHT, 80, 2, 40),
            35
        );
        assert_eq!(
            layout_new_pane_size(l.w(), 80, root, LAYOUT_LEFTRIGHT, 8, 2, 40),
            8
        );
        assert_eq!(
            layout_new_pane_size(l.w(), 80, root, LAYOUT_LEFTRIGHT, 1, 2, 40),
            1
        );
    }

    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        let root = layout_root_ptr(&(*l.w()).layout_root);
        assert_eq!(
            layout_new_pane_size(l.w(), 24, root, LAYOUT_TOPBOTTOM, 24, 2, 12),
            7
        );
        assert_eq!(
            layout_new_pane_size(l.w(), 24, root, LAYOUT_TOPBOTTOM, 2, 2, 12),
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
        let root = layout_root_ptr(&(*l.w()).layout_root);
        assert_eq!(layout_set_size_check(l.w(), root, LAYOUT_LEFTRIGHT, 40), 1);
        assert_eq!(layout_set_size_check(l.w(), root, LAYOUT_LEFTRIGHT, 4), 0);
        assert_eq!(layout_set_size_check(l.w(), root, LAYOUT_LEFTRIGHT, 5), 1);
        assert_eq!(layout_set_size_check(l.w(), root, LAYOUT_LEFTRIGHT, 0), 0);
        assert_eq!(layout_set_size_check(l.w(), root, LAYOUT_TOPBOTTOM, 24), 1);
        assert_eq!(
            layout_set_size_check(l.w(), (*l.pane(0)).layout_cell, LAYOUT_LEFTRIGHT, 0),
            0
        );
    }

    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        let root = layout_root_ptr(&(*l.w()).layout_root);
        assert_eq!(layout_set_size_check(l.w(), root, LAYOUT_TOPBOTTOM, 24), 1);
        assert_eq!(layout_set_size_check(l.w(), root, LAYOUT_TOPBOTTOM, 4), 0);
    }
}

#[test]
fn a_size_check_walks_into_nodes_of_the_other_kind() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        let root = layout_root_ptr(&(*l.w()).layout_root);
        assert_eq!(layout_set_size_check(l.w(), root, LAYOUT_TOPBOTTOM, 24), 1);
        assert_eq!(layout_set_size_check(l.w(), root, LAYOUT_TOPBOTTOM, 2), 0);
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
        let root = layout_root_ptr(&(*l.w()).layout_root);
        (*root).type_0 = 99;
        assert_eq!(layout_spread_cell(l.w(), root), 0);
        (*root).type_0 = LAYOUT_LEFTRIGHT;

        (*root).sx = 1;
        assert_eq!(layout_spread_cell(l.w(), root), 0);
        (*root).sx = 2;
        assert_eq!(layout_spread_cell(l.w(), root), 0);
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
            layout_spread_cell(l.w(), layout_root_ptr(&(*l.w()).layout_root)),
            0
        );
        layout_spread_out(l.pane(0));
        assert_eq!(l.dump(), "LR 80x24+0+0 [%1 40x24+0+0 | %2 39x24+41+0]");
    }
}

#[test]
fn spreading_out_with_a_status_line_leaves_room_for_it() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        with_status(&mut l, PANE_STATUS_TOP, |l| {
            layout_spread_out(l.pane(0));
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
        layout_fix_offsets(l.w());
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
        let root = layout_root_ptr(&(*l.w()).layout_root);
        assert_eq!(
            layout_new_pane_size(l.w(), 80, root, LAYOUT_LEFTRIGHT, 0, 2, 40),
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
        let root = layout_root_ptr(&(*l.w()).layout_root);
        (*root).sx = 1;
        assert_eq!(layout_spread_cell(l.w(), root), 0);
        (*root).sx = 3;
        assert_eq!(layout_spread_cell(l.w(), root), 0);
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
        layout_resize_pane_to(l.pane(3), LAYOUT_LEFTRIGHT, 1);
        layout_resize_pane_to(l.pane(1), LAYOUT_LEFTRIGHT, 1);
        assert_eq!(
            l.dump(),
            "LR 80x24+0+0 [%1 40x24+0+0 | %2 1x24+41+0 | %3 35x24+43+0 | %4 1x24+79+0]"
        );
        let third = (*l.pane(2)).layout_cell;
        layout_resize_layout(l.w(), third, LAYOUT_LEFTRIGHT, 5, 1);
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
    unsafe {
        let first = (*l.pane(0)).layout_cell;
        assert_eq!(
            layout_add_horizontal_border(l.w(), first, PANE_STATUS_TOP),
            1
        );
        assert_eq!(
            layout_add_horizontal_border(l.w(), first, PANE_STATUS_BOTTOM),
            1
        );
    }
}

#[test]
fn shrinking_walks_back_to_the_first_cell_with_room_to_give() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        layout_resize_pane_to(l.pane(1), LAYOUT_LEFTRIGHT, 1);
        assert_eq!(
            l.dump(),
            "LR 80x24+0+0 [%1 40x24+0+0 | %2 1x24+41+0 | %3 37x24+43+0]"
        );
        let second = (*l.pane(1)).layout_cell;
        layout_resize_layout(l.w(), second, LAYOUT_LEFTRIGHT, -5, 1);
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
        let lc = layout_floating_pane(l.w(), 20, 10, 4, 2);
        let j = l.add_pane(20, 10);
        layout_assign_pane(lc, l.pane(j), 0);
        layout_resize(l.w(), 100, 30);
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
        let lc = layout_floating_pane(l.w(), 20, 10, 4, 2);
        let j = l.add_pane(20, 10);
        layout_assign_pane(lc, l.pane(j), 0);
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
        let root = layout_root_ptr(&(*l.w()).layout_root);
        (*root).type_0 = 99;
        assert_eq!(l.dump(), "? 80x24+0+0");
        (*root).type_0 = LAYOUT_WINDOWPANE;
    }
}
