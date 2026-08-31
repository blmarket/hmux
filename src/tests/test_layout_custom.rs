use super::*;
use crate::layout::layout_cell_set_pane;
use crate::layout::{
    LAYOUT_CELL_FLOATING, SPAWN_BEFORE, layout_assign_pane, layout_create_cell,
    layout_floating_pane, layout_free, layout_init, layout_split_pane,
};
use crate::tests::test_fixtures::{Pane, Window, globals};
use crate::window::window_pane_of_id;
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr::null_mut;
use ::std::ffi::CString;
use ::std::sync::MutexGuard;

/// A window carrying a layout tree and the panes that hang off it, the same
/// server-free shape the layout tests use. The tree is freed before the
/// panes go.
struct Layout {
    window: Window,
    panes: Vec<Pane>,
    next_id: u_int,
}

impl Layout {
    fn new(sx: u_int, sy: u_int) -> Layout {
        let mut l = Layout {
            window: Window::new(1, "custom", sx, sy),
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

    fn add_pane(&mut self, sx: u_int, sy: u_int) -> usize {
        self.next_id += 1;
        let mut pane = Pane::new(self.next_id, sx, sy, 100);
        self.window.add_pane(&mut pane);
        self.panes.push(pane);
        self.panes.len() - 1
    }

    fn split(&mut self, i: usize, type_0: layout_type, size: c_int, flags: c_int) -> usize {
        unsafe {
            let wp = self.pane(i);
            let lc = layout_split_pane(wp, type_0, size, flags);
            assert!(!lc.is_null(), "there was no room to split");
            let j = self.add_pane(1, 1);
            layout_assign_pane(lc, self.pane(j), 0);
            j
        }
    }

    /// Adds a floating pane over the layout, at the head of the z-index
    /// list where the server keeps the ones drawn on top.
    fn float(&mut self, sx: u_int, sy: u_int, ox: c_int, oy: c_int) -> usize {
        unsafe {
            let lc = layout_floating_pane(self.w(), sx, sy, ox, oy);
            let j = self.add_pane(sx, sy);
            layout_assign_pane(lc, self.pane(j), 0);
            self.raise_floating();
            j
        }
    }

    /// Puts the floating panes in front of the rest on the z-index list.
    fn raise_floating(&mut self) {
        unsafe {
            let w = self.w();
            (*w).z_index
                .sort_by_key(|id| window_pane_is_floating(window_pane_of_id(w, *id)) == 0);
        }
    }

    fn dump(&mut self) -> String {
        unsafe {
            layout_dump(self.w(), (*self.w()).layout_root_ptr())
                .expect("layout dump")
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Reads `layout` into the window, answering the failure it reported.
    fn parse(&mut self, layout: &str) -> Result<(), String> {
        unsafe {
            let s = CString::new(layout).expect("no NUL");
            match layout_parse(self.w(), s.as_ptr()) {
                Ok(()) => Ok(()),
                Err(cause) => Err(cause.to_string_lossy().into_owned()),
            }
        }
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

fn guard() -> MutexGuard<'static, ()> {
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
        assert_eq!((*target.w()).sx, 80);
        assert_eq!((*target.w()).sy, 24);
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
        let root = (*l.w()).layout_root_ptr();
        let found = layout_find_bottomright(root);
        assert_eq!(found, (*l.pane(2)).layout_cell);
        let leaf = (*l.pane(0)).layout_cell;
        assert_eq!(layout_find_bottomright(leaf), leaf);
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
        assert_eq!((*l.w()).sx, 80);
        assert_eq!((*l.w()).sy, 24);
    }
}

#[test]
fn appending_stops_when_the_buffer_is_full() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let mut buf: Vec<u8> = Vec::new();
        let root = (*l.w()).layout_root_ptr();
        assert_eq!(layout_append(root, &mut buf, 0), -1);
        assert_eq!(layout_append(null_mut::<layout_cell>(), &mut buf, 8), 0);
        assert_eq!(layout_append(root, &mut buf, 8), -1);
    }
}

#[test]
fn appending_a_tree_stops_when_the_buffer_is_full() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        let root = (*l.w()).layout_root_ptr();
        for len in [12, 14, 24, 26] {
            let mut buf: Vec<u8> = Vec::new();
            assert_eq!(layout_append(root, &mut buf, len), -1, "{len}");
        }
    }
}

#[test]
fn a_dump_that_does_not_fit_answers_nothing() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let root = (*l.w()).layout_root_ptr();
        (*root).sx = u_int::MAX;
        (*root).sy = u_int::MAX;
        (*root).xoff = c_int::MIN;
        (*root).yoff = c_int::MIN;
        assert!(layout_dump(l.w(), root).is_some());
    }
}

#[test]
fn assigning_panes_walks_the_tree_in_order() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    l.split(0, LAYOUT_LEFTRIGHT, -1, 0);
    unsafe {
        let w = l.w();
        let root = (*w).layout_root_ptr();
        let mut wp = window_panes_first(w);
        layout_assign(w, &mut wp, root, LAYOUT_CELL_FLOATING);
        assert!(wp.is_null());
        assert_eq!(
            (*(*l.pane(0)).layout_cell).flags & LAYOUT_CELL_FLOATING,
            LAYOUT_CELL_FLOATING
        );
        (*(*l.pane(0)).layout_cell).flags &= !LAYOUT_CELL_FLOATING;
        (*(*l.pane(1)).layout_cell).flags &= !LAYOUT_CELL_FLOATING;
        layout_assign(w, &mut wp, null_mut::<layout_cell>(), 0);
    }
}

#[test]
fn a_cell_of_an_unknown_kind_is_walked_past() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let w = l.w();
        let root = (*w).layout_root_ptr();
        (*root).type_0 = 99;
        let mut wp = window_panes_first(w);
        layout_assign(w, &mut wp, root, 0);
        assert_eq!(wp, window_panes_first(w));
        assert_eq!(layout_check(root), 1);
        (*root).type_0 = LAYOUT_WINDOWPANE;
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
    unsafe {
        let s = c"80x24,0,0,1";
        let mut p = s.as_ptr() as *const c_char;
        let lc =
            layout_construct_cell(null_mut::<layout_cell>(), &mut p).expect("the cell was read");
        assert_eq!(lc.sx, 80);
        assert_eq!(lc.sy, 24);
        assert_eq!(lc.xoff, 0);
        assert_eq!(lc.yoff, 0);
        assert_eq!(CStr::from_ptr(p).to_str().unwrap(), "");
        crate::layout::layout_free_cell(null_mut(), Some(lc));

        let s = c"80x24,0,0,40x24,0,0,1";
        let mut p = s.as_ptr() as *const c_char;
        let lc = layout_construct_cell(null_mut::<layout_cell>(), &mut p);
        assert_eq!(CStr::from_ptr(p).to_str().unwrap(), ",40x24,0,0,1");
        crate::layout::layout_free_cell(null_mut(), lc);

        let s = c"80x24,0,0{";
        let mut p = s.as_ptr() as *const c_char;
        let lc = layout_construct_cell(null_mut::<layout_cell>(), &mut p);
        assert_eq!(CStr::from_ptr(p).to_str().unwrap(), "{");
        crate::layout::layout_free_cell(null_mut(), lc);
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
        let root = (*l.w()).layout_root_ptr();
        for len in [10, 11, 22, 23] {
            let mut buf: Vec<u8> = Vec::new();
            assert_eq!(layout_append(root, &mut buf, len), -1, "{len}");
        }
    }
}

/// A tree too long for the eight kilobytes `layout_dump` writes into gets
/// no string at all.
#[test]
fn a_tree_too_long_for_the_buffer_dumps_nothing() {
    let _g = guard();
    let mut l = Layout::new(80, 24);
    unsafe {
        let mut node = layout_create_cell(null_mut::<layout_cell>());
        let node_ptr = &raw mut *node;
        node.type_0 = LAYOUT_LEFTRIGHT;
        for _ in 0..2000 {
            let mut child = layout_create_cell(node_ptr);
            child.sx = 8;
            child.sy = 8;
            child.xoff = 0;
            child.yoff = 0;
            node.cells.push(child);
        }
        assert!(layout_dump(l.w(), node_ptr).is_none());
        crate::layout::layout_free_cell(null_mut(), Some(node));
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
            let mut lc = layout_create_cell(null_mut::<layout_cell>());
            lc.sx = 8;
            lc.sy = 8;
            lc.xoff = 0;
            lc.yoff = 0;
            lc.flags |= LAYOUT_CELL_FLOATING;
            let lc_ptr = &raw mut *lc;
            cells.push(lc);

            let at = l.add_pane(8, 8);
            let wp = l.pane(at);
            (*wp).layout_cell = lc_ptr;
            layout_cell_set_pane(lc_ptr, wp);
            ids.push((*wp).id);
        }
        (*l.w()).z_index.clear();
        (*l.w()).z_index.extend(ids);
        let root = (*l.w()).layout_root_ptr();
        assert!(layout_dump(l.w(), root).is_none());
        for mut lc in cells {
            lc.wp_id = None;
            crate::layout::layout_free_cell(null_mut(), Some(lc));
        }
        (*l.w()).z_index.clear();
    }
}
