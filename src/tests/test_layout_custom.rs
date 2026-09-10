use crate::pane_identity::PaneIdentity;
use crate::window_dimensions::WindowDimensionsState;

use crate::pane_geometry::PaneGeometryState;

use super::*;
use crate::layout::layout_cell_set_pane;
use crate::layout::{LAYOUT_CELL_FLOATING, SPAWN_BEFORE, layout_create_cell};
use crate::tests::test_fixtures::{Pane, Window, globals};
use crate::window::window_pane_is_floating;
use ::core::ffi::c_int;
use ::std::ffi::CString;

/// A window carrying a layout tree and the panes that hang off it, the same
/// server-free shape the layout tests use. The tree is freed before the
/// panes go.
struct Layout {
    window: Window,
    panes: Vec<Pane>,
    next_id: u_int,
}

impl Layout {
    fn reference(&self) -> WindowRef {
        self.window.reference()
    }

    fn new(sx: u_int, sy: u_int) -> Layout {
        let mut l = Layout {
            window: Window::new(1, "custom", sx, sy),
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

    fn add_pane(&mut self, sx: u_int, sy: u_int) -> usize {
        self.next_id += 1;
        let mut pane = Pane::new(self.next_id, sx, sy, 100);
        self.window.add_pane(&mut pane);
        self.panes.push(pane);
        self.panes.len() - 1
    }

    fn split(&mut self, i: usize, type_0: layout_type, size: c_int, flags: c_int) -> usize {
        unsafe {
            let pane_id = (*self.pane(i)).pane_id();
            let lc = (self.reference()).split_pane_layout(
                &crate::window::window_pane_find_by_id(pane_id).expect("the layout pane exists"),
                type_0,
                size,
                flags,
            );
            assert!(lc.is_some(), "there was no room to split");
            let j = self.add_pane(1, 1);
            (self.reference()).assign_pane_layout(
                lc.as_ref().unwrap(),
                &crate::window::window_pane_find_by_id((*self.pane(j)).pane_id())
                    .expect("the layout pane exists"),
                0,
            );
            j
        }
    }

    /// Adds a floating pane over the layout, at the head of the z-index
    /// list where the server keeps the ones drawn on top.
    fn float(&mut self, sx: u_int, sy: u_int, ox: c_int, oy: c_int) -> usize {
        unsafe {
            let lc = (self.reference()).float_pane_layout(sx, sy, ox, oy);
            let j = self.add_pane(sx, sy);
            (self.reference()).assign_pane_layout(
                &lc,
                &crate::window::window_pane_find_by_id((*self.pane(j)).pane_id())
                    .expect("the layout pane exists"),
                0,
            );
            self.raise_floating();
            j
        }
    }

    /// Puts the floating panes in front of the rest on the z-index list.
    fn raise_floating(&mut self) {
        unsafe {
            let w = self.w();
            let owner = crate::window::window_ref_of(&*w).unwrap();
            let mut order = owner
                .as_window()
                .z_index
                .iter()
                .map(|pane| pane.id())
                .collect::<Vec<_>>();
            order.sort_by_key(|id| {
                window_pane_is_floating(
                    &owner.as_window(),
                    &crate::window::window_pane_find_by_id(*id)
                        .expect("the pane allocation exists"),
                ) == 0
            });
            (*w).z_index = order
                .into_iter()
                .filter_map(crate::window::window_pane_find_by_id)
                .collect();
        }
    }

    fn dump(&mut self) -> String {
        unsafe {
            (self.reference())
                .dump_layout_cell((*self.w()).layout_root.as_deref())
                .expect("layout dump")
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Reads `layout` into the window, answering the failure it reported.
    fn parse(&mut self, layout: &str) -> Result<(), String> {
        unsafe {
            let s = CString::new(layout).expect("no NUL");
            match (self.reference()).parse_layout(s.as_c_str()) {
                Ok(()) => Ok(()),
                Err(cause) => Err(cause.to_string_lossy().into_owned()),
            }
        }
    }

    /// The sizes and offsets the panes themselves were given.
    fn panes(&mut self) -> Vec<String> {
        unsafe {
            (*self.w())
                .panes
                .iter()
                .map(|pane| {
                    let geometry = pane.as_pane().geometry();
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

fn guard() -> crate::tests::test_fixtures::GlobalsGuard {
    globals()
}

/// The checksum tmux writes in front of a layout.
fn checksum(body: &str) -> String {
    let s = CString::new(body).expect("no NUL");
    format!("{:04x}", layout_checksum(&s))
}

#[test]
fn one_pane_dumps_as_one_cell() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    assert_eq!(l.dump(), format!("{},80x24,0,0,1", checksum("80x24,0,0,1")));
}

#[test]
fn a_side_by_side_split_dumps_in_braces() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    let body = "80x24,0,0{40x24,0,0,1,39x24,41,0,2}";
    assert_eq!(l.dump(), format!("{},{body}", checksum(body)));
}

#[test]
fn a_stacked_split_dumps_in_square_brackets() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    let body = "80x24,0,0[80x12,0,0,1,80x11,0,13,2]";
    assert_eq!(l.dump(), format!("{},{body}", checksum(body)));
}

#[test]
fn a_nested_split_dumps_one_inside_the_other() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    let body = "80x24,0,0{40x24,0,0,1,39x24,41,0[39x12,41,0,2,39x11,41,13,3]}";
    assert_eq!(l.dump(), format!("{},{body}", checksum(body)));
}

/// A floating pane is dumped after the tree, inside angle brackets.
#[test]
fn floating_panes_are_dumped_in_angle_brackets() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.float(20, 10, 4, 2);
    l.float(10, 5, 1, 1);
    let body = "80x24,0,0[80x24,0,0,1,20x10,4,2,2,10x5,1,1,3]<20x10,4,2,2,10x5,1,1,3>";
    assert_eq!(l.dump(), format!("{},{body}", checksum(body)));
}

#[test]
fn the_checksum_turns_the_string_round_a_bit_at_a_time() {
    let _g = guard();
    assert_eq!(layout_checksum(c""), 0);
    assert_eq!(layout_checksum(c"a"), 0x61);
    assert_eq!(layout_checksum(c"ab"), 32914);
    assert_eq!(layout_checksum(c"ba"), 146);
    assert_eq!(checksum("80x24,0,0,1"), "b25e");
}

#[test]
fn a_layout_reads_back_into_a_window_of_its_own() {
    let _g = guard();
    let mut source = Layout::new(80, 24);
    source.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    source.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    let dumped = source.dump();

    let mut target = Layout::new(10, 10);
    target.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    target.split(0, LAYOUT_TOPBOTTOM, -1, 0);
    assert_eq!(target.parse(&dumped), Ok(()));
    assert_eq!(target.dump(), dumped);
    assert_eq!(
        target.panes(),
        vec!["%1 40x24+0+0", "%2 39x12+41+0", "%3 39x11+41+13",]
    );
    unsafe {
        assert_eq!((*target.w()).dimensions().size.width, 80);
        assert_eq!((*target.w()).dimensions().size.height, 24);
    }
}

#[test]
fn a_layout_that_is_not_a_layout_is_refused() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    for bad in ["", "zzzz", "80x24,0,0,1", "8dc5", "8dc5x"] {
        assert_eq!(l.parse(bad), Err("invalid layout".to_string()), "{bad:?}");
    }
}

#[test]
fn a_layout_with_the_wrong_checksum_is_refused() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    assert_eq!(
        l.parse("0000,80x24,0,0,1"),
        Err("invalid layout".to_string())
    );
}

#[test]
fn a_layout_the_parser_cannot_follow_is_refused() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    for body in [
        "80x24,0,0{40x24,0,0,1",
        "80x24,0,0[40x24,0,0,1}",
        "80x24,0,0{40x24,0,0,1]",
        "80x24,0,0{40x24,0,0,1;",
        "80x24,0,0{zz",
        "x24,0,0,1",
        "80y24,0,0,1",
        "80x24;0,0,1",
        "80x24,0;0,1",
    ] {
        let line = format!("{},{body}", checksum(body));
        assert_eq!(
            l.parse(&line),
            Err("invalid layout".to_string()),
            "{body:?}"
        );
    }
}

/// A bracket with an empty slot — a comma standing where a cell should be —
/// is refused. 3.7b links the missing cell as a null pointer and the server
/// dies; the `is_null` guard in `layout_construct` rejects it instead, the
/// way the patched oracle and tmux master do.
#[test]
fn a_layout_with_an_empty_slot_is_refused() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    for body in [
        "80x24,0,0{,40x24,0,0,1}",
        "80x24,0,0{40x24,0,0,1,,39x24,41,0,2}",
        "80x24,0,0[,80x12,0,0,1]",
        "80x24,0,0{40x24,0,0,1,}",
    ] {
        let line = format!("{},{body}", checksum(body));
        assert_eq!(
            l.parse(&line),
            Err("invalid layout".to_string()),
            "{body:?}"
        );
    }
}

/// The floating section the dump writes is not something the parser knows,
/// so a layout carrying one is turned away.
#[test]
fn a_layout_with_a_floating_section_is_refused() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    let body = "80x24,0,0,1<20x10,4,2,2>";
    let line = format!("{},{body}", checksum(body));
    assert_eq!(l.parse(&line), Err("invalid layout".to_string()));
}

#[test]
fn a_layout_with_fewer_cells_than_panes_says_so() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    let body = "80x24,0,0,1";
    let line = format!("{},{body}", checksum(body));
    assert_eq!(l.parse(&line), Err("have 2 panes but need 1".to_string()));
}

#[test]
fn a_layout_with_more_cells_than_panes_drops_the_bottom_right_ones() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    let body = "80x24,0,0{40x24,0,0,1,39x24,41,0[39x12,41,0,2,39x11,41,13,3]}";
    let line = format!("{},{body}", checksum(body));
    assert_eq!(l.parse(&line), Ok(()));
    assert_eq!(l.dump(), format!("{},80x24,0,0,1", checksum("80x24,0,0,1")));
}

#[test]
fn a_layout_whose_sizes_do_not_add_up_is_refused() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    for body in [
        "80x24,0,0{40x24,0,0,1,39x20,41,0,2}",
        "80x24,0,0[80x12,0,0,1,70x11,0,13,2]",
    ] {
        let line = format!("{},{body}", checksum(body));
        assert_eq!(
            l.parse(&line),
            Err("size mismatch after applying layout".to_string()),
            "{body:?}"
        );
    }
}

/// A tree whose own size does not match what its children add up to is
/// resized to fit them rather than turned away.
#[test]
fn a_tree_that_is_the_wrong_size_is_made_to_fit_its_children() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    let body = "99x24,0,0{40x24,0,0,1,39x24,41,0,2}";
    let line = format!("{},{body}", checksum(body));
    assert_eq!(l.parse(&line), Ok(()));
    let want = "80x24,0,0{40x24,0,0,1,39x24,41,0,2}";
    assert_eq!(l.dump(), format!("{},{want}", checksum(want)));
}

#[test]
fn a_cell_with_a_pane_id_is_told_from_one_with_a_child_size() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    let body = "80x24,0,0{40x24,0,0,1,39x24,41,0,2}";
    let line = format!("{},{body}", checksum(body));
    assert_eq!(l.parse(&line), Ok(()));
    assert_eq!(l.dump(), format!("{},{body}", checksum(body)));

    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    let body = "80x24,0,0{40x24,0,0{20x24,0,0,1,19x24,21,0,2},39x24,41,0,3}";
    let line = format!("{},{body}", checksum(body));
    assert_eq!(l.parse(&line), Ok(()));
    let want = "80x24,0,0{40x24,0,0,1,39x24,41,0,2}";
    assert_eq!(l.dump(), format!("{},{want}", checksum(want)));
}

#[test]
fn the_bottom_right_cell_is_the_last_one_all_the_way_down() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_TOPBOTTOM, -1, 0);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        let found = layout_find_bottomright(&mut *root);
        assert_eq!(
            found
                .get(&*root)
                .unwrap()
                .wp
                .as_ref()
                .map(|pane| pane.id()),
            Some((*l.pane(2)).pane_id())
        );
        let pane_id = (*l.pane(0)).pane_id();
        let leaf = layout_cell_for_pane(
            (*l.w()).layout_root.as_deref(),
            &crate::window::window_pane_find_by_id(pane_id).expect("the pane allocation exists"),
        )
        .unwrap()
        .0;
        assert_eq!(layout_find_bottomright(&*leaf), LayoutCellPath::root());
    }
}

#[test]
fn a_layout_of_a_single_cell_takes_over_a_window_of_several() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    let body = "40x12,0,0,1";
    let line = format!("{},{body}", checksum(body));
    assert_eq!(l.parse(&line), Ok(()));
    assert_eq!(l.dump(), format!("{},{body}", checksum(body)));
    unsafe {
        assert_eq!((*l.w()).dimensions().size.width, 80);
        assert_eq!((*l.w()).dimensions().size.height, 24);
    }
}

#[test]
fn appending_stops_when_the_buffer_is_full() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let mut buf: Vec<u8> = Vec::new();
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        assert_eq!(layout_append(Some(root), &mut buf, 0), -1);
        assert_eq!(layout_append(None, &mut buf, 8), 0);
        assert_eq!(layout_append(Some(root), &mut buf, 8), -1);
    }
}

#[test]
fn appending_a_tree_stops_when_the_buffer_is_full() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        for len in [12, 14, 24, 26] {
            let mut buf: Vec<u8> = Vec::new();
            assert_eq!(layout_append(Some(root), &mut buf, len), -1, "{len}");
        }
    }
}

#[test]
fn a_dump_that_does_not_fit_answers_nothing() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        (*root).sx = u_int::MAX;
        (*root).sy = u_int::MAX;
        (*root).xoff = c_int::MIN;
        (*root).yoff = c_int::MIN;
        assert!((l.reference()).dump_layout_cell(Some(&*root)).is_some());
    }
}

#[test]
fn assigning_panes_walks_the_tree_in_order() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        let w = &mut *l.w();
        let root = w.layout_root.as_deref_mut().unwrap();
        let mut panes = w.panes.iter_mut();
        layout_assign(&mut panes, Some(root), LAYOUT_CELL_FLOATING);
        assert_eq!(panes.len(), 0);
        layout_assign(&mut panes, None, 0);
        for (pane, cell) in w.panes.iter().zip(&mut root.cells) {
            assert_eq!(
                cell.wp.as_ref().map(|pane| pane.id()),
                Some(pane.pane_id())
            );
            assert_eq!(cell.flags & LAYOUT_CELL_FLOATING, LAYOUT_CELL_FLOATING);
            cell.flags &= !LAYOUT_CELL_FLOATING;
        }
    }
}

#[test]
fn a_cell_of_an_unknown_kind_is_walked_past() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let w = &mut *l.w();
        let root = w.layout_root.as_deref_mut().unwrap();
        root.type_0 = 99;
        let mut panes = w.panes.iter_mut();
        layout_assign(&mut panes, Some(root), 0);
        assert_eq!(panes.len(), 1);
        assert_eq!(layout_check(root), 1);
        root.type_0 = LAYOUT_WINDOWPANE;
    }
}

#[test]
fn a_split_before_dumps_the_new_pane_first() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, 20, SPAWN_BEFORE);
    let body = "80x24,0,0{20x24,0,0,2,59x24,21,0,1}";
    assert_eq!(l.dump(), format!("{},{body}", checksum(body)));
}

#[test]
fn the_layout_string_is_read_back_as_bytes_the_parser_walked_over() {
    let _g = guard();
    {
        let s = c"80x24,0,0,1";
        let mut p = LayoutCursor::new(s, 0);
        let lc = layout_construct_cell(None, &mut p).expect("the cell was read");
        assert_eq!(lc.sx, 80);
        assert_eq!(lc.sy, 24);
        assert_eq!(lc.xoff, 0);
        assert_eq!(lc.yoff, 0);
        assert_eq!(p.as_cstr().to_str().unwrap(), "");
        layout_free_cell(Some(lc));

        let s = c"80x24,0,0,40x24,0,0,1";
        let mut p = LayoutCursor::new(s, 0);
        let lc = layout_construct_cell(None, &mut p);
        assert_eq!(p.as_cstr().to_str().unwrap(), ",40x24,0,0,1");
        layout_free_cell(lc);

        let s = c"80x24,0,0{";
        let mut p = LayoutCursor::new(s, 0);
        let lc = layout_construct_cell(None, &mut p);
        assert_eq!(p.as_cstr().to_str().unwrap(), "{");
        layout_free_cell(lc);
    }
}

/// An empty layout body is refused. 3.7b built a stub cell for it and then
/// failed the pane count ("have 1 panes but need 0"); the `is_null` guard
/// in `layout_construct` now turns the empty top-level cell away as an
/// invalid layout, the way the patched oracle and tmux master do.
#[test]
fn an_empty_layout_is_refused() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    assert_eq!(
        l.parse(&format!("{},", checksum(""))),
        Err("invalid layout".to_string())
    );
}

#[test]
fn a_layout_with_something_left_over_is_refused() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    let body = "80x24,0,0,1>";
    assert_eq!(
        l.parse(&format!("{},{body}", checksum(body))),
        Err("invalid layout".to_string())
    );
}

#[test]
fn a_cell_whose_numbers_are_not_where_they_should_be_is_refused() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    for body in ["80x 24,0,0,1", "80x24, 0,0,1", "80x24,-1,0,1"] {
        let line = format!("{},{body}", checksum(body));
        assert_eq!(
            l.parse(&line),
            Err("invalid layout".to_string()),
            "{body:?}"
        );
    }
}

#[test]
fn a_node_inside_the_tree_that_does_not_add_up_is_refused() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    l.split(1, LAYOUT_LEFTRIGHT, -1, 0);
    for body in [
        "80x24,0,0{40x24,0,0{20x24,0,0,1,15x24,21,0,2},39x24,41,0,3}",
        "80x24,0,0[80x12,0,0[80x6,0,0,1,80x3,0,7,2],80x11,0,13,3]",
    ] {
        let line = format!("{},{body}", checksum(body));
        assert_eq!(
            l.parse(&line),
            Err("size mismatch after applying layout".to_string()),
            "{body:?}"
        );
    }
}

#[test]
fn appending_a_node_stops_when_the_bracket_will_not_fit() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        let root = (*l.w()).layout_root.as_deref_mut().unwrap();
        for len in [10, 11, 22, 23] {
            let mut buf: Vec<u8> = Vec::new();
            assert_eq!(layout_append(Some(root), &mut buf, len), -1, "{len}");
        }
    }
}

/// A tree too long for the eight kilobytes `layout_dump` writes into gets
/// no string at all.
#[test]
fn a_tree_too_long_for_the_buffer_dumps_nothing() {
    let _g = guard();
    let l = Layout::new(80, 24);
    unsafe {
        let mut node = layout_create_cell(None);
        let node_ptr = &raw mut *node;
        node.type_0 = LAYOUT_LEFTRIGHT;
        for _ in 0..2000 {
            let mut child = layout_create_cell(node_ptr.as_mut());
            child.sx = 8;
            child.sy = 8;
            child.xoff = 0;
            child.yoff = 0;
            node.cells.push(child);
        }
        assert!((l.reference()).dump_layout_cell(Some(&*node_ptr)).is_none());
        layout_free_cell(Some(node));
    }
}

/// The same for the floating panes written after the tree.
#[test]
fn too_many_floating_panes_dump_nothing() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let mut cells: Vec<Box<layout_cell>> = Vec::new();
        let mut ids: Vec<u_int> = Vec::new();
        for _ in 0..1000 {
            let mut lc = layout_create_cell(None);
            lc.sx = 8;
            lc.sy = 8;
            lc.xoff = 0;
            lc.yoff = 0;
            lc.flags |= LAYOUT_CELL_FLOATING;
            let lc_ptr = &raw mut *lc;
            cells.push(lc);

            let at = l.add_pane(8, 8);
            let wp = l.pane(at);
            layout_cell_set_pane(
                &mut *lc_ptr,
                wp.as_ref()
                    .and_then(|pane| (pane).observation()),
            );
            ids.push((*wp).pane_id());
        }
        (*l.w()).z_index.clear();
        (*l.w()).z_index.extend(
            ids.into_iter()
                .filter_map(crate::window::window_pane_find_by_id),
        );
        (*l.w()).layout_root.as_mut().unwrap().cells.extend(cells);
        assert!((l.reference()).dump_layout_cell(None).is_none());
    }
}

#[test]
fn geometry_numbers_preserve_unsigned_wrapping_and_signed_saturation() {
    for (number, size, offset) in [
        ("0", 0, 0),
        ("2147483647", 2_147_483_647, 2_147_483_647),
        ("2147483648", 2_147_483_648, i32::MIN),
        ("4294967295", u32::MAX, -1),
        ("4294967296", 0, 0),
        ("9223372036854775807", u32::MAX, -1),
        ("9223372036854775808", 0, -1),
        ("18446744073709551615", u32::MAX, -1),
        ("18446744073709551616", u32::MAX, -1),
        ("999999999999999999999999999999999999", u32::MAX, -1),
    ] {
        let text = CString::new(format!("{number}x{number},{number},{number},99")).unwrap();
        let mut cursor = LayoutCursor::new(&text, 0);
        let cell = layout_construct_cell(None, &mut cursor).unwrap();
        assert_eq!((cell.sx, cell.sy), (size, size), "{number}");
        assert_eq!((cell.xoff, cell.yoff), (offset, offset), "{number}");
        assert!(cursor.as_cstr().is_empty());
    }
}

#[test]
fn signed_or_spaced_geometry_is_rejected_without_replacing_the_layout() {
    let _g = guard();
    let mut layout = Layout::new(80, 24);
    let original = layout.dump();
    for body in [
        "80x+24,0,0,1",
        "80x 24,0,0,1",
        "80x-24,0,0,1",
        "80x24,+0,0,1",
        "80x24,-0,0,1",
        "80x24, 0,0,1",
        "80x24,0,+0,1",
        "80x24,0,-0,1",
        "80x24,0, 0,1",
    ] {
        assert_eq!(
            layout.parse(&format!("{},{body}", checksum(body))),
            Err("invalid layout".to_string()),
            "{body}"
        );
        assert_eq!(layout.dump(), original);
    }
}

#[test]
fn layout_checksum_prefix_preserves_scan_width_sign_and_whitespace() {
    for (text, expected) in [
        (c"0000,body", Some(0)),
        (c"abCD,body", Some(0xabcd)),
        (c" 12a,body", Some(0x12a)),
        (c"+12a,body", Some(0x12a)),
        (c"-12a,body", Some(0xfed6)),
        (c"0x12,body", Some(0x12)),
        (c"0X12,body", Some(0x12)),
        (c"\t fF,body", Some(0xff)),
        (c"  0x,body", None),
        (c" +0x,body", None),
        (c" fF ,body", None),
        (c"12345,body", None),
        (c"123,body", None),
        (c"000z,body", None),
        (c"0000", None),
        (c"", None),
    ] {
        assert_eq!(layout_parse_checksum(text), expected, "{text:?}");
    }
}
