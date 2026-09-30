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
) {
    (*pctx).cctxs.push(layout_parse_cell_ctx {
        lc,
        active,
        last,
        index,
        zindex,
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
    return -(1 as ::core::ffi::c_int);
}
unsafe fn layout_find_bottomright(mut lc: *mut layout_cell) -> *mut layout_cell {
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return lc;
    }
    lc = layout_cells_last(&*lc);
    return layout_find_bottomright(lc);
}
fn layout_checksum(layout: &[u8]) -> u_short {
    layout.iter().fold(0u16, |checksum, &byte| {
        checksum
            .rotate_right(1)
            .wrapping_add(byte as ::core::ffi::c_char as u16)
    })
}
/// Structural copy only: capturing a tree never queries a pane or its Window.
/// The owning Window releases its borrow before resolving these weak identities.
pub(crate) struct LayoutSnapshot {
    kind: layout_type,
    flags: i32,
    geometry: layout_geometry,
    pane: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    children: Vec<LayoutSnapshot>,
}
impl LayoutSnapshot {
    pub(crate) fn capture(cell: &layout_cell) -> Self {
        Self {
            kind: cell.type_0,
            flags: cell.flags,
            geometry: cell.g,
            pane: cell.wp.clone(),
            children: cell
                .cells
                .iter()
                .map(|child| Self::capture(child))
                .collect(),
        }
    }
    fn has_tiled_leaf(&self) -> bool {
        if self.kind == LAYOUT_WINDOWPANE {
            self.flags & LAYOUT_CELL_FLOATING == 0
        } else {
            self.children.iter().any(Self::has_tiled_leaf)
        }
    }
    fn into_compat(mut self) -> Option<Self> {
        if self.kind == LAYOUT_WINDOWPANE {
            return (self.flags & LAYOUT_CELL_FLOATING == 0).then_some(self);
        }
        self.pane = std::rc::Weak::new();
        self.children = self
            .children
            .into_iter()
            .filter_map(Self::into_compat)
            .collect();
        match self.children.len() {
            0 => None,
            1 => self.children.pop(),
            _ => Some(self),
        }
    }
    pub(crate) unsafe fn dump(self, legacy: bool) -> Option<CString> {
        let mut body = Vec::new();
        if legacy {
            if !self.has_tiled_leaf() {
                return None;
            }
            let mut compatible = self.into_compat()?;
            if compatible.kind == LAYOUT_WINDOWPANE && compatible.flags & LAYOUT_CELL_FLOATING == 0
            {
                compatible.geometry.xoff = 0;
                compatible.geometry.yoff = 0;
            }
            compatible.append_v1(&mut body);
        } else {
            self.append_v2(&mut body)?;
        }
        let mut output = Vec::new();
        if legacy {
            output.extend_from_slice(format!("{:04x},", layout_checksum(&body)).as_bytes());
            output.extend_from_slice(&body);
        } else {
            output.extend_from_slice(b"{\"V\":2,\"L\":");
            output.extend_from_slice(&body);
            output.push(b'}');
        }
        Some(CString::new(output).expect("layout serializer produced an interior NUL"))
    }
    unsafe fn append_v2(&self, bytes: &mut Vec<u8>) -> Option<()> {
        let kind = match self.kind {
            LAYOUT_TOPBOTTOM => 'v',
            LAYOUT_LEFTRIGHT => 'h',
            LAYOUT_WINDOWPANE => 'p',
            _ => return None,
        };
        let g = self.geometry;
        bytes.extend_from_slice(
            format!(
                "{{\"t\":\"{kind}\",\"w\":{},\"h\":{},\"x\":{},\"y\":{}",
                g.sx, g.sy, g.xoff, g.yoff
            )
            .as_bytes(),
        );
        if self.kind != LAYOUT_WINDOWPANE {
            if self.children.is_empty() {
                return None;
            }
            bytes.extend_from_slice(b",\"c\":[");
            for (index, child) in self.children.iter().enumerate() {
                if index != 0 {
                    bytes.push(b',');
                }
                child.append_v2(bytes)?;
            }
            bytes.push(b']');
        } else {
            let pane = self.pane.upgrade()?;
            let window = pane
                .window_observer()
                .upgrade()
                .expect("layout pane window");
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
            if self.flags & LAYOUT_CELL_FLOATING != 0 {
                if let Some(index) = window.pane_stacking_index(&observer) {
                    bytes.extend_from_slice(format!(",\"z\":{index}").as_bytes());
                }
            }
            bytes.extend_from_slice(format!(",\"I\":\"%{}\"", pane.id()).as_bytes());
        }
        bytes.push(b'}');
        Some(())
    }
    unsafe fn append_v1(&self, bytes: &mut Vec<u8>) {
        let g = self.geometry;
        bytes.extend_from_slice(format!("{}x{},{},{}", g.sx, g.sy, g.xoff, g.yoff).as_bytes());
        if let Some(pane) = self.pane.upgrade() {
            bytes.extend_from_slice(format!(",{}", pane.id()).as_bytes());
        }
        let brackets = match self.kind {
            LAYOUT_LEFTRIGHT => b"{}",
            LAYOUT_TOPBOTTOM => b"[]",
            _ => return,
        };
        bytes.push(brackets[0]);
        for (index, child) in self.children.iter().enumerate() {
            if index != 0 {
                bytes.push(b',');
            }
            child.append_v1(bytes);
        }
        bytes.push(brackets[1]);
    }
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
        2 | _ => {}
    }
    return 1 as ::core::ffi::c_int;
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
                        b"empty/missing layout parse context\0" as *const u8
                            as *const ::core::ffi::c_char,
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
                    2 | _ => {}
                }
                if (*lc).type_0 as ::core::ffi::c_uint
                    != LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
                    && sx != 0 as u_int
                    && sy != 0 as u_int
                    && ((*lc).g.sx != sx || (*lc).g.sy != sy)
                {
                    layout_print_cell(
                        lc,
                        b"layout_parse\0" as *const u8 as *const ::core::ffi::c_char,
                        0 as u_int,
                    );
                    (*lc).g.sx = sx.wrapping_sub(1 as u_int);
                    (*lc).g.sy = sy.wrapping_sub(1 as u_int);
                }
                if layout_check(lc) == 0 {
                    layout_set_static_cause(
                        pctx.cause.as_deref_mut(),
                        b"size mismatch after applying layout\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                } else {
                    if layout_cell_is_tiled(lc) != 0 || layout_cell_has_tiled_child(lc) != 0 {
                        w_owner.set_layout_size((*lc).g.sx, (*lc).g.sy);
                    }
                    // Resizing may dispatch callbacks. Acquire the current pane order
                    // afterward, then keep all tree edits in one bounded borrow.
                    let panes = w_owner.pane_snapshot();
                    let mut restored;
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
                        layout_assign(&panes, &mut tree, &mut pctx, &mut floating);
                        assert!(floating.is_empty());
                        restored = layout_parse_capture_restore(&pctx);
                        pctx.cctxs.clear();
                    }
                    // No pointer in the parser context or local root survives into
                    // resize/selection callbacks. Pane restoration carries Weak only.
                    lc = std::ptr::null_mut();
                    lcchild = std::ptr::null_mut();
                    drop(panes);
                    layout_fix_offsets(w_owner);
                    layout_fix_panes(w_owner, None);
                    if pctx.version > 1 {
                        layout_parse_apply_ctx(w_owner, &mut restored);
                    }
                    recalculate_sizes();
                    {
                        let tree =
                            w_owner.borrow_layout_root(crate::src::window::LayoutView::Visible);
                        if let Some(root) = tree.as_deref() {
                            layout_print_cell(
                                (root as *const layout_cell).cast_mut(),
                                c"layout_parse".as_ptr(),
                                0,
                            );
                        }
                    }
                    if pctx.version == 1 as int64_t {
                        events_fire_window(
                            b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
                            std::rc::Rc::clone(&(w_owner)),
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
    return -(1 as ::core::ffi::c_int);
}
unsafe fn layout_assign_from_ctx(
    panes: &[std::rc::Rc<std::cell::UnsafeCell<window_pane>>],
    mut pctx: *mut layout_parse_ctx,
) {
    (*pctx).cctxs.sort_unstable_by(|a, b| a.index.cmp(&b.index));
    assert!(
        panes.len() >= (*pctx).cctxs.len(),
        "layout requires enough live panes"
    );
    for (cctx, pane_owner) in (*pctx).cctxs.iter().zip(panes) {
        layout_make_leaf(cctx.lc, pane_owner);
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
            return;
        }
        0 | 1 => {
            lcchild = layout_cells_first(&*lc);
            while !lcchild.is_null() {
                layout_assign_fallback_tiled(panes, lcchild);
                lcchild = layout_cell_next(lcchild);
            }
            return;
        }
        _ => {}
    };
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

/// Selection metadata outlives the tree borrow, but never keeps a cell alive or
/// reconstructs its pane through a pointer after a callback replaces that tree.
struct LayoutPaneRestore {
    pane: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    active: i32,
    last: i32,
    zindex: i32,
}
unsafe fn layout_parse_capture_restore(pctx: &layout_parse_ctx) -> Vec<LayoutPaneRestore> {
    pctx.cctxs
        .iter()
        .map(|cell| LayoutPaneRestore {
            pane: (*cell.lc).wp.clone(),
            active: cell.active,
            last: cell.last,
            zindex: cell.zindex,
        })
        .collect()
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
        b"%ux%u,%d,%d\0" as *const u8 as *const ::core::ffi::c_char,
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
    (*lc).g.sx = sx;
    (*lc).g.sy = sy;
    (*lc).g.xoff = xoff;
    (*lc).g.yoff = yoff;
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
            PANE_MAXIMUM as i64,
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
            WINDOW_MAXIMUM as i64,
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
            layout_parse_add_cctx(pctx, lc, active, last, index, zindex);
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
        if sscanf(
            input,
            b"%hx,%n\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut csum,
            &raw mut n,
        ) != 1 as ::core::ffi::c_int
            || n != 5 as ::core::ffi::c_int
        {
            layout_set_static_cause(
                (*pctx).cause.as_deref_mut(),
                b"malformed layout header\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        input = input.offset(n as isize);
        if csum as ::core::ffi::c_int
            != layout_checksum(CStr::from_ptr(input).to_bytes()) as ::core::ffi::c_int
        {
            layout_set_static_cause(
                (*pctx).cause.as_deref_mut(),
                b"invalid layout checksum\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        (*pctx).root = layout_construct_v1(&raw mut input, 0 as u_int);
        if (*pctx).root.is_none() {
            layout_set_static_cause(
                (*pctx).cause.as_deref_mut(),
                b"invalid layout\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        if *input as ::core::ffi::c_int != '\0' as i32 {
            layout_set_static_cause(
                (*pctx).cause.as_deref_mut(),
                b"trailing data\0" as *const u8 as *const ::core::ffi::c_char,
            );
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
            layout_set_static_cause(
                (*pctx).cause.as_deref_mut(),
                b"version mismatch\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        if (*pctx).num_active > 1 as ::core::ffi::c_int {
            layout_set_static_cause(
                (*pctx).cause.as_deref_mut(),
                b"more than one active pane\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        if (*pctx).cctxs.is_empty() {
            layout_set_static_cause(
                (*pctx).cause.as_deref_mut(),
                b"no panes\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        if layout_parse_ctx_check_indexes(pctx) == 0 {
            return -(1 as ::core::ffi::c_int);
        }
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn layout_parse_apply_ctx(w_owner: &WindowRef, restored: &mut [LayoutPaneRestore]) {
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
    restored.sort_unstable_by(|a, b| b.zindex.cmp(&a.zindex));
    for cctx in restored.iter() {
        let Some(pane_owner) = cctx.pane.upgrade() else {
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
            if let Some(pane_owner) = cctx.pane.upgrade() {
                w_owner.select_pane(&pane_owner, true);
            }
            break;
        }
    }
    w_owner.borrow_pane_history_mut().clear();
    restored.sort_unstable_by(|a, b| b.last.cmp(&a.last));
    for cctx in restored.iter() {
        let Some(pane_owner) = cctx.pane.upgrade() else {
            continue;
        };
        if !(cctx.last < 0 as ::core::ffi::c_int || cctx.active == 1 as ::core::ffi::c_int) {
            crate::src::shared::pane::pane_history_push(
                &mut w_owner.borrow_pane_history_mut(),
                std::rc::Rc::downgrade(&pane_owner),
            );
        }
    }
}
unsafe fn layout_parse_ctx_check_indexes(mut pctx: *mut layout_parse_ctx) -> ::core::ffi::c_int {
    (*pctx).cctxs.sort_unstable_by(|a, b| a.index.cmp(&b.index));
    if (*pctx)
        .cctxs
        .windows(2)
        .any(|pair| pair[0].index == pair[1].index)
    {
        layout_set_static_cause(
            (*pctx).cause.as_deref_mut(),
            b"duplicate pane index\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return 0 as ::core::ffi::c_int;
    }
    (*pctx)
        .cctxs
        .sort_unstable_by(|a, b| b.zindex.cmp(&a.zindex));
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
            b"duplicate pane z-index\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return 0 as ::core::ffi::c_int;
    }
    (*pctx).cctxs.sort_unstable_by(|a, b| b.last.cmp(&a.last));
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
            b"duplicate last pane index\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
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
mod snapshot_tests {
    use super::*;

    #[test]
    fn legacy_snapshot_outlives_source_tree_and_collapses_floating_siblings() {
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
            let snapshot = LayoutSnapshot::capture(&root);
            drop(root);
            let result = snapshot.dump(true).unwrap();
            let body = b"40x24,0,0";
            assert_eq!(
                result.to_bytes(),
                format!("{:04x},40x24,0,0", layout_checksum(body)).as_bytes()
            );
            let mut floating = layout_create_cell();
            floating.flags = LAYOUT_CELL_FLOATING;
            assert!(LayoutSnapshot::capture(&floating).dump(true).is_none());
        }
    }
}

#[cfg(test)]
mod restoration_borrow_tests {
    use super::*;
    use std::rc::Rc;

    fn context(cells: Vec<layout_parse_cell_ctx>) -> layout_parse_ctx<'static> {
        layout_parse_ctx {
            version: 2,
            num_active: 1,
            root: None,
            cause: None,
            cctxs: cells,
        }
    }

    #[test]
    fn restoration_keeps_weak_pane_identity_after_tree_replacement() {
        unsafe {
            let first = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::allocate();
            let second = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::allocate();
            let mut root = layout_create_cell();
            crate::src::layout::layout_make_node(&mut *root, LAYOUT_LEFTRIGHT);
            let mut cells = Vec::new();
            for (index, pane) in [&first, &second].into_iter().enumerate() {
                let mut cell = layout_create_cell();
                layout_make_leaf(&mut *cell, pane);
                cells.push(layout_parse_cell_ctx {
                    lc: &mut *cell,
                    active: index as i32,
                    last: 7 - index as i32,
                    index: index as i32,
                    zindex: 2 - index as i32,
                });
                layout_cells_push_back(&mut *root, cell);
            }
            let mut ctx = context(cells);
            let restored = layout_parse_capture_restore(&ctx);
            ctx.cctxs.clear();
            // Model a selection callback replacing the tree before history is
            // restored. No saved state may dereference the old cells afterward.
            drop(root);
            assert!(first.layout_identity(false).is_none());
            assert!(second.layout_identity(false).is_none());
            assert!(Rc::ptr_eq(&restored[0].pane.upgrade().unwrap(), &first));
            assert!(Rc::ptr_eq(&restored[1].pane.upgrade().unwrap(), &second));
            assert_eq!(
                (restored[1].active, restored[1].last, restored[1].zindex),
                (1, 6, 1)
            );
            assert_eq!(Rc::strong_count(&first), 1);
            assert_eq!(Rc::strong_count(&second), 1);
            drop(first);
            assert!(restored[0].pane.upgrade().is_none());
        }
    }

    #[test]
    fn legacy_assignment_preserves_detached_floating_cell_and_pane_links() {
        unsafe {
            let tiled = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::allocate();
            let floating = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::allocate();
            let mut old_root = layout_create_cell();
            crate::src::layout::layout_make_node(&mut *old_root, LAYOUT_TOPBOTTOM);
            let mut old_tiled = layout_create_cell();
            layout_make_leaf(&mut *old_tiled, &tiled);
            let mut old_floating = layout_create_cell();
            old_floating.flags = LAYOUT_CELL_FLOATING;
            layout_set_size(&mut *old_floating, 12, 8, 5, 3);
            layout_make_leaf(&mut *old_floating, &floating);
            let floating_id = old_floating.id();
            let floating_ptr = &mut *old_floating as *mut layout_cell;
            layout_cells_push_back(&mut *old_root, old_tiled);
            layout_cells_push_back(&mut *old_root, old_floating);
            let mut detached = vec![layout_cells_remove(&mut *old_root, floating_ptr).unwrap()];
            drop(old_root);
            assert!(tiled.layout_identity(false).is_none());
            assert_eq!(floating.layout_identity(false), Some(floating_id));
            let mut tree = Some(layout_create_cell());
            layout_set_size(tree.as_deref_mut().unwrap(), 80, 24, 0, 0);
            let panes = [tiled.clone(), floating.clone()];
            let mut ctx = context(Vec::new());
            ctx.version = 1;
            layout_assign(&panes, &mut tree, &mut ctx, &mut detached);
            assert!(detached.is_empty());
            let root = tree.as_deref().unwrap();
            assert_eq!(root.type_0, LAYOUT_TOPBOTTOM);
            assert_eq!(root.cells.len(), 2);
            assert!(root.cells[0].wp.ptr_eq(&Rc::downgrade(&tiled)));
            assert_eq!(&*root.cells[1] as *const layout_cell, floating_ptr);
            assert_eq!(
                (
                    root.cells[1].g.sx,
                    root.cells[1].g.sy,
                    root.cells[1].g.xoff,
                    root.cells[1].g.yoff
                ),
                (12, 8, 5, 3)
            );
            drop(tree);
            assert!(tiled.layout_identity(false).is_none());
            assert!(floating.layout_identity(false).is_none());
        }
    }
}
