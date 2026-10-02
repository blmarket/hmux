use crate::src::events::events_fire_window;
use crate::src::ffi::libc::{__ctype_b_loc, sscanf};
use crate::src::json::{
    json_find, json_find_array, json_find_boolean, json_find_number, json_find_object,
    json_find_string, json_get_object, json_parse,
};
use crate::src::layout::{
    layout_cell_has_tiled_child, layout_cell_is_tiled, layout_count_cells, layout_create_cell,
    layout_destroy_cell, layout_fix_offsets, layout_fix_panes, layout_make_leaf, layout_print_cell,
    layout_set_size, layout_take_leaf,
};
use crate::src::resize::recalculate_sizes;
use crate::src::shared::window::WindowRef;
use crate::src::window::Window as _;
use crate::src::window_pane::WindowPane as _;
use std::ffi::{CStr, CString};

macro_rules! layout_format_cause {
    ($cause:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {{
        if let Some(cause) = $cause {
            *cause = Some(CString::new(format!($fmt $(, $arg)*))
                .expect("layout diagnostic contains no NUL"));
        }
    }};
}

unsafe fn layout_set_static_cause(
    cause: Option<&mut Option<CString>>,
    message: *const ::core::ffi::c_char,
) {
    if let Some(cause) = cause {
        *cause = Some(CStr::from_ptr(message).to_owned());
    }
}
use crate::src::shared::abi::int64_t;
use crate::src::shared::abi::*;
use crate::src::shared::ctype::{_ISdigit, _ISspace};
use crate::src::shared::json::json_node;
use crate::src::shared::layout::layout_cell;
use crate::src::shared::layout::*;
use crate::src::shared::limits::INT_MAX;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::{PANE_MAXIMUM, PANE_MINIMUM};
use crate::src::shared::window::window;
use crate::src::shared::window::WINDOW_MAXIMUM;

#[repr(C)]
pub struct layout_parse_ctx<'a> {
    pub version: int64_t,
    pub num_active: ::core::ffi::c_int,
    pub scrolling: Option<(u32, u32)>,
    pub root: Option<Box<layout_cell>>,
    pub cause: Option<&'a mut Option<CString>>,
    pub cctxs: Vec<layout_parse_cell_ctx>,
}
impl layout_parse_ctx<'_> {
    fn root_ptr(&mut self) -> *mut layout_cell {
        self.root
            .as_deref_mut()
            .map_or(std::ptr::null_mut(), |root| root)
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct layout_parse_cell_ctx {
    pub lc: *mut layout_cell,
    pub active: ::core::ffi::c_int,
    pub last: ::core::ffi::c_int,
    pub index: ::core::ffi::c_int,
    pub zindex: ::core::ffi::c_int,
    pub full_width: Option<bool>,
}

unsafe fn layout_parse_free_ctx(mut pctx: *mut layout_parse_ctx) {
    drop((*pctx).root.take());
    (*pctx).root = None;
    (*pctx).cctxs.clear();
}
unsafe fn layout_parse_add_cctx(
    mut pctx: *mut layout_parse_ctx,
    mut lc: *mut layout_cell,
    mut active: ::core::ffi::c_int,
    mut last: ::core::ffi::c_int,
    mut index: ::core::ffi::c_int,
    mut zindex: ::core::ffi::c_int,
    full_width: Option<bool>,
) {
    (*pctx).cctxs.push(layout_parse_cell_ctx {
        lc,
        active,
        last,
        index,
        zindex,
        full_width,
    });
}
unsafe fn layout_parse_remove_cctx(
    mut pctx: *mut layout_parse_ctx,
    mut lc: *mut layout_cell,
) -> ::core::ffi::c_int {
    if let Some(i) = (*pctx).cctxs.iter().position(|cctx| cctx.lc == lc) {
        (*pctx).cctxs.swap_remove(i);
        return 0 as ::core::ffi::c_int;
    }
    -(1 as ::core::ffi::c_int)
}
unsafe fn layout_find_bottomright(mut lc: *mut layout_cell) -> *mut layout_cell {
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return lc;
    }
    lc = layout_cells_last(&*lc);
    layout_find_bottomright(lc)
}
fn layout_checksum(layout: &[u8]) -> u_short {
    layout.iter().fold(0u16, |checksum, &byte| {
        checksum
            .rotate_right(1)
            .wrapping_add(byte as ::core::ffi::c_char as u16)
    })
}
// Serialize the live tree. These reads do not dispatch callbacks or mutate layout.
pub(crate) unsafe fn layout_dump(
    root: &layout_cell,
    legacy: bool,
    scrolling: Option<(u32, u32)>,
) -> Option<CString> {
    let mut body = Vec::new();
    if legacy {
        layout_append_v1(layout_compat_cell(root)?, &mut body, true);
    } else {
        layout_append_v2(root, &mut body)?;
    }
    let mut output = Vec::new();
    if legacy {
        output.extend_from_slice(format!("{:04x},", layout_checksum(&body)).as_bytes());
        output.extend_from_slice(&body);
    } else {
        output.extend_from_slice(b"{\"V\":2,\"L\":");
        output.extend_from_slice(&body);
        if let Some((width, height)) = scrolling {
            output.extend_from_slice(
                format!(",\"scrolling\":{{\"width\":{width},\"height\":{height}}}").as_bytes(),
            );
        }
        output.push(b'}');
    }
    Some(CString::new(output).expect("layout serializer produced an interior NUL"))
}

// Legacy layouts omit floating leaves and collapse nodes with one tiled child.
fn layout_compat_cell(cell: &layout_cell) -> Option<&layout_cell> {
    if cell.type_0 == LAYOUT_WINDOWPANE {
        return (cell.flags & LAYOUT_CELL_FLOATING == 0).then_some(cell);
    }
    let mut children = cell
        .cells
        .iter()
        .filter_map(|child| layout_compat_cell(child));
    let first = children.next()?;
    Some(if children.next().is_none() {
        first
    } else {
        cell
    })
}

unsafe fn layout_append_v2(cell: &layout_cell, bytes: &mut Vec<u8>) -> Option<()> {
    let kind = match cell.type_0 {
        LAYOUT_TOPBOTTOM => 'v',
        LAYOUT_LEFTRIGHT => 'h',
        LAYOUT_WINDOWPANE => 'p',
        _ => return None,
    };
    let g = cell.g;
    bytes.extend_from_slice(
        format!(
            "{{\"t\":\"{kind}\",\"w\":{},\"h\":{},\"x\":{},\"y\":{}",
            g.sx, g.sy, g.xoff, g.yoff
        )
        .as_bytes(),
    );
    if cell.type_0 != LAYOUT_WINDOWPANE {
        if cell.cells.is_empty() {
            return None;
        }
        bytes.extend_from_slice(b",\"c\":[");
        for (index, child) in cell.cells.iter().enumerate() {
            if index != 0 {
                bytes.push(b',');
            }
            layout_append_v2(child, bytes)?;
        }
        bytes.push(b']');
    } else {
        let pane = cell.wp.upgrade()?;
        let window = pane
            .window_observer()
            .upgrade()
            .expect("layout pane window");
        bytes.extend_from_slice(format!(",\"full\":{}", pane.scrolling_full_width()).as_bytes());
        let observer = std::rc::Rc::downgrade(&pane);
        if window
            .active_pane()
            .as_ref()
            .is_some_and(|active| std::rc::Rc::ptr_eq(active, &pane))
        {
            bytes.extend_from_slice(b",\"a\":true");
        } else if let Some(index) = window.pane_history_index(&observer) {
            bytes.extend_from_slice(format!(",\"l\":{index}").as_bytes());
        }
        let index = window.pane_index(&observer)?;
        bytes.extend_from_slice(format!(",\"i\":{index}").as_bytes());
        if cell.flags & LAYOUT_CELL_FLOATING != 0 {
            if let Some(index) = window.pane_stacking_index(&observer) {
                bytes.extend_from_slice(format!(",\"z\":{index}").as_bytes());
            }
        }
        bytes.extend_from_slice(format!(",\"I\":\"%{}\"", pane.id()).as_bytes());
    }
    bytes.push(b'}');
    Some(())
}

unsafe fn layout_append_v1(cell: &layout_cell, bytes: &mut Vec<u8>, root: bool) {
    let g = &cell.g;
    let (x, y) = if root && cell.type_0 == LAYOUT_WINDOWPANE {
        (0, 0)
    } else {
        (g.xoff, g.yoff)
    };
    bytes.extend_from_slice(format!("{}x{},{},{}", g.sx, g.sy, x, y).as_bytes());
    if cell.type_0 == LAYOUT_WINDOWPANE {
        if let Some(pane) = cell.wp.upgrade() {
            bytes.extend_from_slice(format!(",{}", pane.id()).as_bytes());
        }
        return;
    }
    let brackets = match cell.type_0 {
        LAYOUT_LEFTRIGHT => b"{}",
        LAYOUT_TOPBOTTOM => b"[]",
        _ => return,
    };
    bytes.push(brackets[0]);
    for (index, child) in cell
        .cells
        .iter()
        .filter_map(|child| layout_compat_cell(child))
        .enumerate()
    {
        if index != 0 {
            bytes.push(b',');
        }
        layout_append_v1(child, bytes, false);
    }
    bytes.push(brackets[1]);
}

unsafe fn layout_check(mut lc: *mut layout_cell) -> ::core::ffi::c_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0 as u_int;
    match (*lc).type_0 as ::core::ffi::c_uint {
        0 => {
            lcchild = layout_cells_first(&*lc);
            while !lcchild.is_null() {
                if !(layout_cell_is_tiled(lcchild) == 0
                    && layout_cell_has_tiled_child(lcchild) == 0)
                {
                    if (*lcchild).g.sy != (*lc).g.sy {
                        return 0 as ::core::ffi::c_int;
                    }
                    if layout_check(lcchild) == 0 {
                        return 0 as ::core::ffi::c_int;
                    }
                    n = n.wrapping_add((*lcchild).g.sx.wrapping_add(1 as u_int));
                }
                lcchild = layout_cell_next(lcchild);
            }
            if n != 0 as u_int && n.wrapping_sub(1 as u_int) != (*lc).g.sx {
                return 0 as ::core::ffi::c_int;
            }
        }
        1 => {
            lcchild = layout_cells_first(&*lc);
            while !lcchild.is_null() {
                if !(layout_cell_is_tiled(lcchild) == 0
                    && layout_cell_has_tiled_child(lcchild) == 0)
                {
                    if (*lcchild).g.sx != (*lc).g.sx {
                        return 0 as ::core::ffi::c_int;
                    }
                    if layout_check(lcchild) == 0 {
                        return 0 as ::core::ffi::c_int;
                    }
                    n = n.wrapping_add((*lcchild).g.sy.wrapping_add(1 as u_int));
                }
                lcchild = layout_cell_next(lcchild);
            }
            if n != 0 as u_int && n.wrapping_sub(1 as u_int) != (*lc).g.sy {
                return 0 as ::core::ffi::c_int;
            }
        }
        _ => {}
    }
    1 as ::core::ffi::c_int
}
pub unsafe fn layout_parse(
    w_owner: &WindowRef,
    mut input: *const ::core::ffi::c_char,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut candidate: Option<Box<layout_cell>> = None;
    let mut pctx: layout_parse_ctx = layout_parse_ctx {
        version: -1,
        num_active: 0,
        scrolling: None,
        root: None,
        cause: cause.as_mut(),
        cctxs: Vec::new(),
    };
    let mut npanes: u_int = 0;
    let mut ncells: u_int = 0;
    let mut sx: u_int = 0 as u_int;
    let mut sy: u_int = 0 as u_int;
    let mut with_floating: ::core::ffi::c_int = 0;
    if layout_construct(input, &raw mut pctx) != 0 as ::core::ffi::c_int {
        layout_parse_free_ctx(&raw mut pctx);
        return -(1 as ::core::ffi::c_int);
    }
    with_floating = (pctx.version > 1 as int64_t) as ::core::ffi::c_int;
    npanes = w_owner
        .pane_snapshot()
        .iter()
        .filter(|pane| with_floating != 0 || !pane.is_floating())
        .count() as u_int;
    if npanes == 0 as u_int {
        layout_format_cause!(
            pctx.cause.as_deref_mut(),
            "window @{} has no panes",
            (w_owner).id(),
        );
    } else {
        loop {
            ncells = layout_count_cells(pctx.root_ptr(), with_floating);
            if npanes > ncells {
                layout_format_cause!(
                    pctx.cause.as_deref_mut(),
                    "have {} panes but need {}",
                    npanes,
                    ncells,
                );
                current_block = 4277046812173491162;
                break;
            } else {
                if npanes == ncells {
                    current_block = 15976848397966268834;
                    break;
                }
                lcchild = layout_find_bottomright(pctx.root_ptr());
                if pctx.version > 1 as int64_t
                    && layout_parse_remove_cctx(&raw mut pctx, lcchild) != 0 as ::core::ffi::c_int
                {
                    layout_set_static_cause(
                        pctx.cause.as_deref_mut(),
                        c"empty/missing layout parse context".as_ptr(),
                    );
                    current_block = 4277046812173491162;
                    break;
                } else {
                    layout_destroy_cell(Some(w_owner), lcchild, &mut pctx.root);
                }
            }
        }
        match current_block {
            4277046812173491162 => {}
            _ => {
                candidate = pctx.root.take();
                lc = candidate.as_deref_mut().expect("parsed layout is owned");
                match (*lc).type_0 as ::core::ffi::c_uint {
                    0 => {
                        lcchild = layout_cells_first(&*lc);
                        while !lcchild.is_null() {
                            if layout_cell_is_tiled(lcchild) != 0
                                || layout_cell_has_tiled_child(lcchild) != 0
                            {
                                sy = (*lcchild).g.sy.wrapping_add(1 as u_int);
                                sx = sx.wrapping_add((*lcchild).g.sx.wrapping_add(1 as u_int));
                            }
                            lcchild = layout_cell_next(lcchild);
                        }
                    }
                    1 => {
                        lcchild = layout_cells_first(&*lc);
                        while !lcchild.is_null() {
                            if layout_cell_is_tiled(lcchild) != 0
                                || layout_cell_has_tiled_child(lcchild) != 0
                            {
                                sx = (*lcchild).g.sx.wrapping_add(1 as u_int);
                                sy = sy.wrapping_add((*lcchild).g.sy.wrapping_add(1 as u_int));
                            }
                            lcchild = layout_cell_next(lcchild);
                        }
                    }
                    _ => {}
                }
                if (*lc).type_0 as ::core::ffi::c_uint
                    != LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
                    && sx != 0 as u_int
                    && sy != 0 as u_int
                    && ((*lc).g.sx != sx || (*lc).g.sy != sy)
                {
                    layout_print_cell(lc, c"layout_parse".as_ptr(), 0 as u_int);
                    (*lc).g.sx = sx.wrapping_sub(1 as u_int);
                    (*lc).g.sy = sy.wrapping_sub(1 as u_int);
                }
                if layout_check(lc) == 0 {
                    layout_set_static_cause(
                        pctx.cause.as_deref_mut(),
                        c"size mismatch after applying layout".as_ptr(),
                    );
                } else {
                    w_owner.unzoom(true);
                    let size =
                        if layout_cell_is_tiled(lc) != 0 || layout_cell_has_tiled_child(lc) != 0 {
                            ((*lc).g.sx, (*lc).g.sy)
                        } else {
                            pctx.scrolling.unwrap_or(w_owner.sizing_size())
                        };
                    w_owner.set_layout_size(size.0, size.1, pctx.scrolling);
                    if pctx.scrolling.is_some() {
                        w_owner.remember_layout_preset(super::set::SCROLLING_LAYOUT as i32);
                    }
                    // Resizing may dispatch callbacks. Acquire the current pane order
                    // afterward, then keep all tree edits in one bounded borrow.
                    let panes = w_owner.pane_snapshot();
                    {
                        let mut tree = w_owner.borrow_layout_root_mut();
                        let mut floating = Vec::new();
                        if pctx.version == 1 {
                            for pane in &panes {
                                let cell = pane
                                    .layout_identity(false)
                                    .and_then(|id| {
                                        tree.as_deref_mut().and_then(|root| root.find_mut(id))
                                    })
                                    .map_or(std::ptr::null_mut(), |cell| cell as *mut layout_cell);
                                if !cell.is_null() && (*cell).flags & LAYOUT_CELL_FLOATING != 0 {
                                    floating.push(
                                        layout_cells_remove((*cell).parent, cell)
                                            .expect("floating cell is owned"),
                                    );
                                    (*cell).parent = std::ptr::null_mut();
                                }
                            }
                        }
                        // Dropping the old cells clears Pane's old cell links; this
                        // must precede assigning the panes into the replacement.
                        drop(tree.take());
                        *tree = candidate.take();
                        layout_assign(&panes, tree, &mut pctx, &mut floating);
                        assert!(floating.is_empty());
                    }
                    lc = std::ptr::null_mut();
                    lcchild = std::ptr::null_mut();
                    drop(panes);
                    layout_fix_offsets(w_owner);
                    layout_fix_panes(w_owner, None);
                    if pctx.version > 1 {
                        layout_parse_apply_ctx(w_owner, &mut pctx.cctxs);
                    }
                    pctx.cctxs.clear();
                    recalculate_sizes();
                    {
                        let tree =
                            w_owner.borrow_layout_root(crate::src::window::LayoutView::Visible);
                        if let Some(root) = tree {
                            layout_print_cell(
                                (root as *const layout_cell).cast_mut(),
                                c"layout_parse".as_ptr(),
                                0,
                            );
                        }
                    }
                    if pctx.version == 1 as int64_t {
                        events_fire_window(
                            c"window-layout-changed".as_ptr(),
                            std::rc::Rc::clone(w_owner),
                        );
                    }
                    layout_parse_free_ctx(&raw mut pctx);
                    return 0 as ::core::ffi::c_int;
                }
            }
        }
    }
    drop(candidate);
    layout_parse_free_ctx(&raw mut pctx);
    -(1 as ::core::ffi::c_int)
}
unsafe fn layout_assign_from_ctx(
    panes: &[std::rc::Rc<std::cell::UnsafeCell<window_pane>>],
    mut pctx: *mut layout_parse_ctx,
) {
    (*pctx).cctxs.sort_unstable_by_key(|a| a.index);
    assert!(
        panes.len() >= (*pctx).cctxs.len(),
        "layout requires enough live panes"
    );
    for (cctx, pane_owner) in (*pctx).cctxs.iter().zip(panes) {
        layout_make_leaf(cctx.lc, pane_owner);
        if let Some(full) = cctx.full_width.or((*pctx).scrolling.map(|_| false)) {
            pane_owner.set_scrolling_full_width(full);
        }
    }
}
unsafe fn layout_assign_fallback_tiled(
    panes: &mut impl Iterator<Item = std::rc::Rc<std::cell::UnsafeCell<window_pane>>>,
    lc: *mut layout_cell,
) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if lc.is_null() {
        return;
    }
    match (*lc).type_0 as ::core::ffi::c_uint {
        2 => {
            if let Some(owner) = panes.find(|owner| owner.layout_identity(false).is_none()) {
                layout_make_leaf(lc, &owner);
            }
        }
        0 | 1 => {
            lcchild = layout_cells_first(&*lc);
            while !lcchild.is_null() {
                layout_assign_fallback_tiled(panes, lcchild);
                lcchild = layout_cell_next(lcchild);
            }
        }
        _ => {}
    }
}
unsafe fn layout_assign_fallback(
    panes: &[std::rc::Rc<std::cell::UnsafeCell<window_pane>>],
    tree: &mut Option<Box<layout_cell>>,
    floating: &mut Vec<Box<layout_cell>>,
) {
    let mut root = tree.as_deref_mut().expect("assigned layout root") as *mut layout_cell;
    layout_assign_fallback_tiled(&mut panes.iter().cloned(), root);
    if panes.len() > 1 && (*root).type_0 == LAYOUT_WINDOWPANE {
        let mut node = layout_create_cell();
        crate::src::layout::layout_make_node(&mut *node, LAYOUT_TOPBOTTOM);
        node.g = (*root).g;
        let previous = tree.replace(node).expect("replaced root is owned");
        root = tree.as_deref_mut().unwrap();
        layout_cells_push_front(root, previous);
    }
    for pane in panes {
        if let Some(id) = pane.layout_identity(false) {
            if floating
                .iter()
                .any(|cell| cell.id() == id && cell.flags & LAYOUT_CELL_FLOATING != 0)
            {
                layout_cells_push_back(root, layout_take_leaf(floating, id));
            }
        }
    }
}
unsafe fn layout_assign(
    panes: &[std::rc::Rc<std::cell::UnsafeCell<window_pane>>],
    tree: &mut Option<Box<layout_cell>>,
    pctx: &mut layout_parse_ctx,
    floating: &mut Vec<Box<layout_cell>>,
) {
    if !pctx.cctxs.is_empty() {
        layout_assign_from_ctx(panes, pctx);
    } else {
        layout_assign_fallback(panes, tree, floating);
    }
}

unsafe fn layout_construct_cell(
    mut layout: *mut *const ::core::ffi::c_char,
) -> Option<Box<layout_cell>> {
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut saved: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if *(*__ctype_b_loc()).offset(**layout as u_char as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        == 0
    {
        return None;
    }
    if sscanf(
        *layout,
        c"%ux%u,%d,%d".as_ptr(),
        &raw mut sx,
        &raw mut sy,
        &raw mut xoff,
        &raw mut yoff,
    ) != 4 as ::core::ffi::c_int
    {
        return None;
    }
    while *(*__ctype_b_loc()).offset(**layout as u_char as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
    {
        *layout = (*layout).offset(1);
    }
    if **layout as ::core::ffi::c_int != 'x' as i32 {
        return None;
    }
    *layout = (*layout).offset(1);
    while *(*__ctype_b_loc()).offset(**layout as u_char as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
    {
        *layout = (*layout).offset(1);
    }
    if **layout as ::core::ffi::c_int != ',' as i32 {
        return None;
    }
    *layout = (*layout).offset(1);
    while *(*__ctype_b_loc()).offset(**layout as u_char as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
    {
        *layout = (*layout).offset(1);
    }
    if **layout as ::core::ffi::c_int != ',' as i32 {
        return None;
    }
    *layout = (*layout).offset(1);
    while *(*__ctype_b_loc()).offset(**layout as u_char as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
    {
        *layout = (*layout).offset(1);
    }
    if **layout as ::core::ffi::c_int == ',' as i32 {
        saved = *layout;
        *layout = (*layout).offset(1);
        while *(*__ctype_b_loc()).offset(**layout as u_char as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
        {
            *layout = (*layout).offset(1);
        }
        if **layout as ::core::ffi::c_int == 'x' as i32 {
            *layout = saved;
        }
    }
    let mut lc = layout_create_cell();
    lc.g.sx = sx;
    lc.g.sy = sy;
    lc.g.xoff = xoff;
    lc.g.yoff = yoff;
    Some(lc)
}
unsafe fn layout_construct_v1(
    layout: *mut *const ::core::ffi::c_char,
    depth: u_int,
) -> Option<Box<layout_cell>> {
    if depth > LAYOUT_V1_MAX_DEPTH as u_int {
        return None;
    }
    let mut owner = layout_construct_cell(layout)?;
    match **layout as u8 {
        b',' | b'}' | b']' | 0 => return Some(owner),
        b'{' => owner.type_0 = LAYOUT_LEFTRIGHT,
        b'[' => owner.type_0 = LAYOUT_TOPBOTTOM,
        _ => return None,
    }
    loop {
        *layout = (*layout).add(1);
        let child = layout_construct_v1(layout, depth.wrapping_add(1))?;
        layout_cells_push_back(&mut *owner, child);
        if **layout as u8 == b',' {
            continue;
        }
        let closing = if owner.type_0 == LAYOUT_LEFTRIGHT {
            b'}'
        } else {
            b']'
        };
        if **layout as u8 != closing {
            return None;
        }
        *layout = (*layout).add(1);
        return Some(owner);
    }
}

unsafe fn layout_parse_json(root: &json_node, pctx: *mut layout_parse_ctx) -> ::core::ffi::c_int {
    let result = (|| {
        let root = json_get_object(root).ok_or_else(|| c"invalid layout json".to_owned())?;
        (*pctx).version = json_find_number(root, c"V")?;
        if json_find(root, c"scrolling").is_some() {
            let metadata = json_find_object(root, c"scrolling")?;
            let width = layout_json_number(
                metadata,
                c"width",
                1,
                WINDOW_MAXIMUM as i64,
                "scrolling width",
            )? as u32;
            let height = layout_json_number(
                metadata,
                c"height",
                1,
                WINDOW_MAXIMUM as i64,
                "scrolling height",
            )? as u32;
            (*pctx).scrolling = Some((width, height));
        }
        let object = json_find_object(root, c"L")?;
        (*pctx).root = Some(layout_parse_json_layout(object, pctx)?);
        Ok::<_, CString>(())
    })();
    if let Err(error) = result {
        if let Some(cause) = (*pctx).cause.as_deref_mut() {
            *cause = Some(error);
        }
        drop((*pctx).root.take());
        return -1;
    }
    0
}

fn layout_json_number(
    node: &json_node,
    key: &CStr,
    minimum: i64,
    maximum: i64,
    description: &str,
) -> Result<i64, CString> {
    let number = json_find_number(node, key)?;
    if number < minimum || number > maximum {
        return Err(CString::new(format!("invalid {description} {number}")).unwrap());
    }
    Ok(number)
}

unsafe fn layout_parse_json_layout(
    node: &json_node,
    pctx: *mut layout_parse_ctx,
) -> Result<Box<layout_cell>, CString> {
    let mut owner = layout_create_cell();
    let lc: *mut layout_cell = &mut *owner;
    let result = (|| {
        let kind = json_find_string(node, c"t")?;
        (*lc).type_0 = match kind.to_bytes() {
            b"p" => LAYOUT_WINDOWPANE,
            b"v" => LAYOUT_TOPBOTTOM,
            b"h" => LAYOUT_LEFTRIGHT,
            _ => {
                let mut message = b"unknown cell type \"".to_vec();
                message.extend_from_slice(kind.to_bytes());
                message.push(b'"');
                return Err(CString::new(message).expect("cell type contains no NUL"));
            }
        };
        (*lc).g.sx = layout_json_number(
            node,
            c"w",
            PANE_MINIMUM as i64,
            if (*pctx).scrolling.is_some() && (*lc).type_0 != LAYOUT_WINDOWPANE {
                i32::MAX as i64
            } else {
                PANE_MAXIMUM as i64
            },
            "width",
        )? as u_int;
        (*lc).g.sy = layout_json_number(
            node,
            c"h",
            PANE_MINIMUM as i64,
            PANE_MAXIMUM as i64,
            "height",
        )? as u_int;
        (*lc).g.xoff = layout_json_number(
            node,
            c"x",
            -WINDOW_MAXIMUM as i64,
            if (*pctx).scrolling.is_some() {
                i32::MAX as i64 - PANE_MAXIMUM as i64 - 1
            } else {
                WINDOW_MAXIMUM as i64
            },
            "x-offset",
        )? as i32;
        (*lc).g.yoff = layout_json_number(
            node,
            c"y",
            -WINDOW_MAXIMUM as i64,
            WINDOW_MAXIMUM as i64,
            "y-offset",
        )? as i32;
        if (*lc).type_0 == LAYOUT_WINDOWPANE {
            if json_find(node, c"c").is_some() {
                return Err(c"panes cannot have children".to_owned());
            }
            let index = layout_json_number(node, c"i", 0, INT_MAX as i64, "index")? as i32;
            let mut active = -1;
            let mut last = -1;
            if json_find(node, c"a").is_some() {
                active = json_find_boolean(node, c"a")?;
                if active != 0 {
                    (*pctx).num_active += 1;
                }
            } else if json_find(node, c"l").is_some() {
                last = layout_json_number(node, c"l", 0, INT_MAX as i64, "last")? as i32;
            }
            let zindex = if json_find(node, c"z").is_some() {
                let zindex =
                    layout_json_number(node, c"z", 0, (INT_MAX - 1) as i64, "floating zindex")?
                        as i32;
                (*lc).flags |= LAYOUT_CELL_FLOATING;
                zindex
            } else {
                INT_MAX
            };
            let full_width = if json_find(node, c"full").is_some() {
                Some(json_find_boolean(node, c"full")? != 0)
            } else {
                None
            };
            layout_parse_add_cctx(pctx, lc, active, last, index, zindex, full_width);
        } else {
            let members = json_find_array(node, c"c")?;
            if members.len() < 2 {
                return Err(c"nodes must have more than one child".to_owned());
            }
            for member in members {
                let child = layout_parse_json_layout(member, pctx)?;
                layout_cells_push_back(lc, child);
            }
        }
        Ok::<_, CString>(())
    })();
    result?;
    Ok(owner)
}
unsafe fn layout_construct(
    mut input: *const ::core::ffi::c_char,
    mut pctx: *mut layout_parse_ctx,
) -> ::core::ffi::c_int {
    let mut csum: u_short = 0;
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while *(*__ctype_b_loc()).offset(*input as u_char as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
    {
        input = input.offset(1);
    }
    if *input as ::core::ffi::c_int != '{' as i32 {
        if sscanf(input, c"%hx,%n".as_ptr(), &raw mut csum, &raw mut n) != 1 as ::core::ffi::c_int
            || n != 5 as ::core::ffi::c_int
        {
            layout_set_static_cause(
                (*pctx).cause.as_deref_mut(),
                c"malformed layout header".as_ptr(),
            );
            return -(1 as ::core::ffi::c_int);
        }
        input = input.offset(n as isize);
        if csum as ::core::ffi::c_int
            != layout_checksum(CStr::from_ptr(input).to_bytes()) as ::core::ffi::c_int
        {
            layout_set_static_cause(
                (*pctx).cause.as_deref_mut(),
                c"invalid layout checksum".as_ptr(),
            );
            return -(1 as ::core::ffi::c_int);
        }
        (*pctx).root = layout_construct_v1(&raw mut input, 0 as u_int);
        if (*pctx).root.is_none() {
            layout_set_static_cause((*pctx).cause.as_deref_mut(), c"invalid layout".as_ptr());
            return -(1 as ::core::ffi::c_int);
        }
        if *input as ::core::ffi::c_int != '\0' as i32 {
            layout_set_static_cause((*pctx).cause.as_deref_mut(), c"trailing data".as_ptr());
            return -(1 as ::core::ffi::c_int);
        }
        (*pctx).version = 1 as int64_t;
    } else {
        let Some(json) = json_parse(CStr::from_ptr(input), (*pctx).cause.as_deref_mut()) else {
            return -(1 as ::core::ffi::c_int);
        };
        if layout_parse_json(&json, pctx) != 0 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int);
        }
        if (*pctx).version != 2 as int64_t {
            layout_set_static_cause((*pctx).cause.as_deref_mut(), c"version mismatch".as_ptr());
            return -(1 as ::core::ffi::c_int);
        }
        if (*pctx).num_active > 1 as ::core::ffi::c_int {
            layout_set_static_cause(
                (*pctx).cause.as_deref_mut(),
                c"more than one active pane".as_ptr(),
            );
            return -(1 as ::core::ffi::c_int);
        }
        if (*pctx).cctxs.is_empty() {
            layout_set_static_cause((*pctx).cause.as_deref_mut(), c"no panes".as_ptr());
            return -(1 as ::core::ffi::c_int);
        }
        if let Some(error) = (*pctx)
            .scrolling
            .and_then(|_| super::scrolling::check_capacity((*pctx).cctxs.len()).err())
        {
            if let Some(cause) = (*pctx).cause.as_deref_mut() {
                *cause = Some(error);
            }
            return -1;
        }
        if layout_parse_ctx_check_indexes(pctx) == 0 {
            return -(1 as ::core::ffi::c_int);
        }
    }
    0 as ::core::ffi::c_int
}
unsafe fn layout_parse_apply_ctx(w_owner: &WindowRef, restored: &mut [layout_parse_cell_ctx]) {
    for pane_owner in w_owner.stacking_snapshot() {
        if pane_owner.is_floating() {
            assert!(
                w_owner
                    .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                    .remove(&std::rc::Rc::downgrade(&pane_owner)),
                "pane is not in its stacking order"
            );
        }
    }
    restored.sort_unstable_by_key(|a| std::cmp::Reverse(a.zindex));
    for cctx in restored.iter() {
        let Some(pane_owner) = (*cctx.lc).wp.upgrade() else {
            continue;
        };
        if pane_owner.is_floating() {
            w_owner
                .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                .push_front(std::rc::Rc::downgrade(&pane_owner));
        }
    }
    for cctx in restored.iter() {
        if cctx.active == 1 as ::core::ffi::c_int {
            if let Some(pane_owner) = (*cctx.lc).wp.upgrade() {
                w_owner.select_pane(&pane_owner, true);
            }
            break;
        }
    }
    w_owner.borrow_pane_history_mut().clear();
    restored.sort_unstable_by_key(|a| std::cmp::Reverse(a.last));
    for cctx in restored.iter() {
        let Some(pane_owner) = (*cctx.lc).wp.upgrade() else {
            continue;
        };
        if !(cctx.last < 0 as ::core::ffi::c_int || cctx.active == 1 as ::core::ffi::c_int) {
            crate::src::shared::pane::pane_history_push(
                w_owner.borrow_pane_history_mut(),
                std::rc::Rc::downgrade(&pane_owner),
            );
        }
    }
}
unsafe fn layout_parse_ctx_check_indexes(mut pctx: *mut layout_parse_ctx) -> ::core::ffi::c_int {
    (*pctx).cctxs.sort_unstable_by_key(|a| a.index);
    if (*pctx)
        .cctxs
        .windows(2)
        .any(|pair| pair[0].index == pair[1].index)
    {
        layout_set_static_cause(
            (*pctx).cause.as_deref_mut(),
            c"duplicate pane index".as_ptr(),
        );
        return 0 as ::core::ffi::c_int;
    }
    (*pctx)
        .cctxs
        .sort_unstable_by_key(|a| std::cmp::Reverse(a.zindex));
    let n = (*pctx)
        .cctxs
        .iter()
        .take_while(|cctx| cctx.zindex == INT_MAX)
        .count();
    if (&(*pctx).cctxs)[n..]
        .windows(2)
        .any(|pair| pair[0].zindex == pair[1].zindex)
    {
        layout_set_static_cause(
            (*pctx).cause.as_deref_mut(),
            c"duplicate pane z-index".as_ptr(),
        );
        return 0 as ::core::ffi::c_int;
    }
    (*pctx)
        .cctxs
        .sort_unstable_by_key(|a| std::cmp::Reverse(a.last));
    let n = (*pctx)
        .cctxs
        .iter()
        .take_while(|cctx| cctx.last >= 0)
        .count();
    if (&(*pctx).cctxs)[..n]
        .windows(2)
        .any(|pair| pair[0].last == pair[1].last)
    {
        layout_set_static_cause(
            (*pctx).cause.as_deref_mut(),
            c"duplicate last pane index".as_ptr(),
        );
        return 0 as ::core::ffi::c_int;
    }
    1 as ::core::ffi::c_int
}

#[cfg(test)]
mod json_tests {
    use super::*;

    fn construct(input: &CStr) -> Result<Vec<(i32, i32, i32, i32)>, CString> {
        unsafe {
            let mut cause = None;
            let mut ctx = layout_parse_ctx {
                version: -1,
                num_active: 0,
                scrolling: None,
                root: None,
                cause: Some(&mut cause),
                cctxs: Vec::new(),
            };
            let result = layout_construct(input.as_ptr(), &mut ctx);
            let cells = ctx
                .cctxs
                .iter()
                .map(|cell| (cell.index, cell.active, cell.last, cell.zindex))
                .collect();
            layout_parse_free_ctx(&mut ctx);
            if result == 0 {
                Ok(cells)
            } else {
                Err(cause.expect("layout failure has a diagnostic"))
            }
        }
    }

    #[test]
    fn borrowed_json_preserves_layout_pane_metadata() {
        let input = c"{\"V\":2,\"L\":{\"t\":\"h\",\"w\":80,\"h\":24,\"x\":0,\"y\":0,\"c\":[{\"t\":\"p\",\"w\":40,\"h\":24,\"x\":0,\"y\":0,\"i\":0,\"a\":false,\"l\":\"ignored\"},{\"t\":\"p\",\"w\":39,\"h\":24,\"x\":41,\"y\":0,\"i\":1,\"l\":7,\"z\":3}]}}";
        let mut cells = construct(input).unwrap();
        cells.sort_by_key(|cell| cell.0);
        assert_eq!(cells, [(0, 0, -1, INT_MAX), (1, -1, 7, 3)]);
    }

    #[test]
    fn borrowed_json_layout_errors_release_partial_trees() {
        for (input, expected) in [
            (r#"{}"#, "key \"V\" not found"),
            (r#"{"V":2,"L":[]}"#, "key \"L\" expected an object"),
            (
                r#"{"V":2,"L":{"t":"unknown"}}"#,
                "unknown cell type \"unknown\"",
            ),
            (r#"{"V":2,"L":{"t":"p","w":0}}"#, "invalid width 0"),
            (
                r#"{"V":2,"L":{"t":"h","w":80,"h":24,"x":0,"y":0,"c":[]}}"#,
                "nodes must have more than one child",
            ),
            (
                r#"{"V":2,"L":{"t":"h","w":80,"h":24,"x":0,"y":0,"c":[{"t":"p","w":40,"h":24,"x":0,"y":0,"i":0},{"t":"p","w":39,"h":24,"x":41,"y":0,"i":-1}]}}"#,
                "invalid index -1",
            ),
        ] {
            let input = CString::new(input).unwrap();
            assert_eq!(
                construct(&input).unwrap_err().to_bytes(),
                expected.as_bytes(),
                "{input:?}"
            );
        }
    }
}

#[cfg(test)]
mod serialization_tests {
    use super::*;

    #[test]
    fn legacy_dump_collapses_floating_siblings() {
        unsafe {
            let mut root = layout_create_cell();
            root.type_0 = LAYOUT_LEFTRIGHT;
            let mut tiled = layout_create_cell();
            tiled.g = layout_geometry {
                sx: 40,
                sy: 24,
                xoff: 8,
                yoff: 3,
            };
            let mut floating = layout_create_cell();
            floating.flags = LAYOUT_CELL_FLOATING;
            layout_cells_push_back(&mut *root, tiled);
            layout_cells_push_back(&mut *root, floating);
            let result = layout_dump(&root, true, None).unwrap();
            let body = b"40x24,0,0";
            assert_eq!(
                result.to_bytes(),
                format!("{:04x},40x24,0,0", layout_checksum(body)).as_bytes()
            );
            let mut floating = layout_create_cell();
            floating.flags = LAYOUT_CELL_FLOATING;
            assert!(layout_dump(&floating, true, None).is_none());
        }
    }
}
