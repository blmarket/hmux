use crate::src::events::events_fire_window;
use crate::src::ffi::libc::{__ctype_b_loc, sscanf};
use crate::src::json::{
    json_find, json_find_array, json_find_boolean, json_find_number, json_find_object,
    json_find_string, json_get_object, json_parse,
};
use crate::src::layout::{
    layout_cell_has_tiled_child, layout_cell_is_tiled, layout_count_cells, layout_create_cell,
    layout_destroy_cell, layout_fix_offsets, layout_fix_panes, layout_make_leaf, layout_print_cell,
    layout_replace_with_node, layout_set_size, layout_take_leaf,
};
use crate::src::resize::recalculate_sizes;
use crate::src::window::{
    window_count_panes, window_pane_first, window_pane_index, window_pane_is_floating,
    window_pane_last_index, window_pane_next, window_pane_stack_first, window_pane_stack_push,
    window_pane_stack_remove, window_pane_z_first, window_pane_z_insert_front, window_pane_z_next,
    window_pane_z_remove, window_pane_zindex, window_resize, window_set_active_pane,
};
use std::ffi::{CStr, CString};

macro_rules! layout_format_cause {
    ($cause:expr_2021, $fmt:literal $(, $arg:expr_2021)* $(,)?) => {{
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
    unsafe {
        if let Some(cause) = cause {
            *cause = Some(CStr::from_ptr(message).to_owned());
        }
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
use crate::src::shared::window::WINDOW_MAXIMUM;
use crate::src::shared::window::window;

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
    unsafe {
        drop((*pctx).root.take());
        (*pctx).root = None;
        (*pctx).cctxs.clear();
    }
}
unsafe fn layout_parse_add_cctx(
    mut pctx: *mut layout_parse_ctx,
    mut lc: *mut layout_cell,
    mut active: ::core::ffi::c_int,
    mut last: ::core::ffi::c_int,
    mut index: ::core::ffi::c_int,
    mut zindex: ::core::ffi::c_int,
) {
    unsafe {
        (*pctx).cctxs.push(layout_parse_cell_ctx {
            lc,
            active,
            last,
            index,
            zindex,
        });
    }
}
unsafe fn layout_parse_remove_cctx(
    mut pctx: *mut layout_parse_ctx,
    mut lc: *mut layout_cell,
) -> ::core::ffi::c_int {
    unsafe {
        if let Some(i) = (*pctx).cctxs.iter().position(|cctx| cctx.lc == lc) {
            (*pctx).cctxs.swap_remove(i);
            return 0 as ::core::ffi::c_int;
        }
        return -(1 as ::core::ffi::c_int);
    }
}
unsafe fn layout_find_bottomright(mut lc: *mut layout_cell) -> *mut layout_cell {
    unsafe {
        if (*lc).type_0 as ::core::ffi::c_uint
            == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return lc;
        }
        lc = layout_cells_last(&*lc);
        return layout_find_bottomright(lc);
    }
}
fn layout_checksum(layout: &[u8]) -> u_short {
    layout.iter().fold(0u16, |checksum, &byte| {
        checksum
            .rotate_right(1)
            .wrapping_add(byte as ::core::ffi::c_char as u16)
    })
}
pub(crate) unsafe fn layout_dump_owned(
    lcroot: *mut layout_cell,
    flags: ::core::ffi::c_int,
) -> Option<CString> {
    unsafe {
        if lcroot.is_null() {
            return None;
        }
        let mut body = Vec::new();
        if layout_append(lcroot, &mut body, flags) != 0 {
            return None;
        }
        let mut output = Vec::new();
        if flags & LAYOUT_CUSTOM_OLD_FORMAT != 0 {
            output.extend_from_slice(format!("{:04x},", layout_checksum(&body)).as_bytes());
            output.extend_from_slice(&body);
        } else {
            output.extend_from_slice(b"{\"V\":2,\"L\":");
            output.extend_from_slice(&body);
            output.push(b'}');
        }
        Some(CString::new(output).expect("layout serializer produced an interior NUL"))
    }
}
unsafe fn layout_append_v2(mut lc: *mut layout_cell, ls: &mut Vec<u8>) -> ::core::ffi::c_int {
    unsafe {
        let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut type_0: layout_type = LAYOUT_LEFTRIGHT;
        let mut c: ::core::ffi::c_char = 0;
        let mut i: u_int = 0;
        let mut n: u_int = 0;
        if lc.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        type_0 = (*lc).type_0;
        if type_0 as ::core::ffi::c_uint
            == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            c = 'v' as i32 as ::core::ffi::c_char;
        } else if type_0 as ::core::ffi::c_uint
            == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            c = 'h' as i32 as ::core::ffi::c_char;
        } else if type_0 as ::core::ffi::c_uint
            == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            c = 'p' as i32 as ::core::ffi::c_char;
        } else {
            return -(1 as ::core::ffi::c_int);
        }
        ls.extend_from_slice(
            format!(
                "{{\"t\":\"{}\",\"w\":{},\"h\":{},\"x\":{},\"y\":{}",
                c as u8 as char,
                (*lc).g.sx,
                (*lc).g.sy,
                (*lc).g.xoff,
                (*lc).g.yoff
            )
            .as_bytes(),
        );
        if type_0 as ::core::ffi::c_uint
            != LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ls.extend_from_slice(b",\"c\":[");
            n = 0 as u_int;
            lcchild = layout_cells_first(&*lc);
            while !lcchild.is_null() {
                if layout_append_v2(lcchild, ls) != 0 as ::core::ffi::c_int {
                    return -(1 as ::core::ffi::c_int);
                }
                ls.extend_from_slice(b",");
                n = n.wrapping_add(1);
                lcchild = layout_cell_next(lcchild);
            }
            if n == 0 as u_int {
                return -(1 as ::core::ffi::c_int);
            }
            ls.pop().expect("layout serializer underflow");
            ls.extend_from_slice(b"]");
        } else {
            let Some(pane_owner) = (*lc).wp.upgrade() else {
                return -1;
            };
            wp = pane_owner.get();
            if wp
                == (*(*wp)
                    .window_handle()
                    .as_ref()
                    .map_or(std::ptr::null_mut(), |owner| owner.get()))
                .active_pane()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get())
            {
                ls.extend_from_slice(b",\"a\":true");
            } else if window_pane_last_index(&*wp)
                .map(|value| {
                    i = value;
                })
                .is_some()
            {
                ls.extend_from_slice(format!(",\"l\":{}", i).as_bytes());
            }
            if !window_pane_index(&*wp)
                .map(|value| {
                    i = value;
                })
                .is_some()
            {
                return -(1 as ::core::ffi::c_int);
            }
            ls.extend_from_slice(format!(",\"i\":{}", i).as_bytes());
            if (*lc).flags & LAYOUT_CELL_FLOATING != 0
                && window_pane_zindex(&*wp)
                    .map(|value| {
                        i = value;
                    })
                    .is_some()
            {
                ls.extend_from_slice(format!(",\"z\":{}", i).as_bytes());
            }
            ls.extend_from_slice(format!(",\"I\":\"%{}\"", (*wp).id).as_bytes());
        }
        ls.extend_from_slice(b"}");
        return 0 as ::core::ffi::c_int;
    }
}
unsafe fn layout_append_v1(mut lc: *mut layout_cell, ls: &mut Vec<u8>) -> ::core::ffi::c_int {
    unsafe {
        let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut brackets = b"[]";
        if lc.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        match (*lc).wp.upgrade() {
            Some(pane_owner) => {
                ls.extend_from_slice(
                    format!(
                        "{}x{},{},{},{}",
                        (*lc).g.sx,
                        (*lc).g.sy,
                        (*lc).g.xoff,
                        (*lc).g.yoff,
                        (*pane_owner.get()).id
                    )
                    .as_bytes(),
                );
            }
            _ => {
                ls.extend_from_slice(
                    format!(
                        "{}x{},{},{}",
                        (*lc).g.sx,
                        (*lc).g.sy,
                        (*lc).g.xoff,
                        (*lc).g.yoff
                    )
                    .as_bytes(),
                );
            }
        }
        let mut current_block_16: u64;
        match (*lc).type_0 as ::core::ffi::c_uint {
            0 => {
                brackets = b"{}";
                current_block_16 = 14129903220312603109;
            }
            1 => {
                current_block_16 = 14129903220312603109;
            }
            2 | _ => {
                current_block_16 = 1054647088692577877;
            }
        }
        match current_block_16 {
            14129903220312603109 => {
                ls.extend_from_slice(&brackets[0..1]);
                lcchild = layout_cells_first(&*lc);
                while !lcchild.is_null() {
                    if layout_append_v1(lcchild, ls) != 0 as ::core::ffi::c_int {
                        return -(1 as ::core::ffi::c_int);
                    }
                    ls.extend_from_slice(b",");
                    lcchild = layout_cell_next(lcchild);
                }
                ls.pop().expect("layout serializer underflow");
                ls.extend_from_slice(&brackets[1..2]);
            }
            _ => {}
        }
        return 0 as ::core::ffi::c_int;
    }
}
unsafe fn layout_custom_copy_layout(lc: *mut layout_cell) -> Option<Box<layout_cell>> {
    unsafe {
        if (*lc).type_0 == LAYOUT_WINDOWPANE && (*lc).flags & LAYOUT_CELL_FLOATING != 0 {
            return None;
        }
        let mut owner = layout_create_cell();
        let copy: *mut layout_cell = &mut *owner;
        owner.type_0 = (*lc).type_0;
        owner.flags = (*lc).flags;
        owner.g = (*lc).g;
        if owner.type_0 == LAYOUT_WINDOWPANE {
            owner.wp = (*lc).wp.clone();
        } else {
            let mut child = layout_cells_first(&*lc);
            while !child.is_null() {
                if let Some(copy_child) = layout_custom_copy_layout(child) {
                    layout_cells_push_back(copy, copy_child);
                }
                child = layout_cell_next(child);
            }
            match owner.cells.children.len() {
                0 => return None,
                1 => return layout_cells_remove(copy, layout_cells_first(&owner)),
                _ => {}
            }
        }
        Some(owner)
    }
}
unsafe fn layout_custom_create_compat(lcroot: *mut layout_cell) -> Option<Box<layout_cell>> {
    unsafe {
        let mut root = layout_custom_copy_layout(lcroot)?;
        if layout_cell_is_tiled(&mut *root) != 0 {
            root.g.xoff = 0;
            root.g.yoff = 0;
        }
        Some(root)
    }
}

unsafe fn layout_custom_unlink_panes(mut lc: *mut layout_cell) {
    unsafe {
        let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        match (*lc).type_0 as ::core::ffi::c_uint {
            2 => {
                (*lc).wp = std::rc::Weak::new();
            }
            0 | 1 => {
                lcchild = layout_cells_first(&*lc);
                while !lcchild.is_null() {
                    layout_custom_unlink_panes(lcchild);
                    lcchild = layout_cell_next(lcchild);
                }
            }
            _ => {}
        };
    }
}
unsafe fn layout_custom_free_compat(mut root: Option<Box<layout_cell>>) {
    unsafe {
        if let Some(root) = root.as_deref_mut() {
            layout_custom_unlink_panes(root);
        }
        drop(root);
    }
}

unsafe fn layout_append(
    mut lcroot: *mut layout_cell,
    ls: &mut Vec<u8>,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut result: ::core::ffi::c_int = 0;
        if flags & LAYOUT_CUSTOM_OLD_FORMAT != 0 {
            if layout_cell_is_tiled(lcroot) == 0 && layout_cell_has_tiled_child(lcroot) == 0 {
                return -(1 as ::core::ffi::c_int);
            }
            let mut lccompat = layout_custom_create_compat(lcroot);
            result = layout_append_v1(
                lccompat
                    .as_deref_mut()
                    .map_or(std::ptr::null_mut(), |root| root),
                ls,
            );
            layout_custom_free_compat(lccompat);
        } else {
            result = layout_append_v2(lcroot, ls);
        }
        return result;
    }
}
unsafe fn layout_check(mut lc: *mut layout_cell) -> ::core::ffi::c_int {
    unsafe {
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
}
pub unsafe fn layout_parse(
    w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>,
    mut input: *const ::core::ffi::c_char,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    unsafe {
        let mut w = w_owner.get();
        let mut current_block: u64;
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
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
        npanes = window_count_panes(&*w, with_floating);
        if npanes == 0 as u_int {
            layout_format_cause!(
                pctx.cause.as_deref_mut(),
                "window @{} has no panes",
                (*w).id,
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
                        && layout_parse_remove_cctx(&raw mut pctx, lcchild)
                            != 0 as ::core::ffi::c_int
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
                            window_resize(
                                &(*(w)).observer.upgrade().expect("live window"),
                                (*lc).g.sx,
                                (*lc).g.sy,
                                -(1 as ::core::ffi::c_int),
                                -(1 as ::core::ffi::c_int),
                            );
                        }
                        let mut floating = Vec::new();
                        if pctx.version == 1 as int64_t {
                            wp = window_pane_first(w.as_ref())
                                .as_ref()
                                .map_or(std::ptr::null_mut(), |owner| owner.get());
                            while !wp.is_null() {
                                if !(window_pane_is_floating(&*wp) == 0) {
                                    lcchild = (*wp).layout_cell as *mut layout_cell;
                                    floating.push(
                                        layout_cells_remove((*lcchild).parent, lcchild)
                                            .expect("floating cell is owned"),
                                    );
                                    (*lcchild).parent = ::core::ptr::null_mut::<layout_cell>();
                                }
                                wp = window_pane_next(wp.as_ref())
                                    .as_ref()
                                    .map_or(std::ptr::null_mut(), |owner| owner.get());
                            }
                        }
                        drop((*w).layout_root.take());
                        (*w).layout_root = candidate.take();
                        layout_assign(w_owner, &raw mut pctx, &mut floating);
                        assert!(floating.is_empty());
                        layout_fix_offsets(w_owner);
                        layout_fix_panes(w_owner, None);
                        if pctx.version > 1 as int64_t {
                            layout_parse_apply_ctx(w_owner, &raw mut pctx);
                        }
                        recalculate_sizes();
                        layout_print_cell(
                            lc,
                            b"layout_parse\0" as *const u8 as *const ::core::ffi::c_char,
                            0 as u_int,
                        );
                        if pctx.version == 1 as int64_t {
                            events_fire_window(
                                b"window-layout-changed\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                (*(w)).observer.upgrade().expect("live window"),
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
}
unsafe fn layout_assign_from_ctx(
    w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>,
    mut pctx: *mut layout_parse_ctx,
) {
    unsafe {
        let mut w = w_owner.get();
        (*pctx).cctxs.sort_unstable_by(|a, b| a.index.cmp(&b.index));
        let panes = (*w).panes.snapshot();
        assert!(
            panes.len() >= (*pctx).cctxs.len(),
            "layout requires enough live panes"
        );
        for (cctx, pane_owner) in (*pctx).cctxs.iter().zip(&panes) {
            layout_make_leaf(cctx.lc, pane_owner);
        }
    }
}
unsafe fn layout_assign_fallback_tiled(
    panes: &mut impl Iterator<Item = std::rc::Rc<std::cell::UnsafeCell<window_pane>>>,
    lc: *mut layout_cell,
) {
    unsafe {
        let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        if lc.is_null() {
            return;
        }
        match (*lc).type_0 as ::core::ffi::c_uint {
            2 => {
                if let Some(owner) = panes.find(|owner| (*owner.get()).layout_cell.is_null()) {
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
}
unsafe fn layout_assign_fallback(
    w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>,
    mut lcroot: *mut layout_cell,
    floating: &mut Vec<Box<layout_cell>>,
) {
    unsafe {
        let mut w = w_owner.get();
        let panes = (*w).panes.snapshot();
        layout_assign_fallback_tiled(&mut panes.iter().cloned(), lcroot);
        if window_count_panes(&*w, 1 as ::core::ffi::c_int) > 1 as u_int
            && (*lcroot).type_0 as ::core::ffi::c_uint
                == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            lcroot = layout_replace_with_node(w_owner, lcroot, LAYOUT_TOPBOTTOM);
        }
        for owner in &panes {
            let wp = owner.get();
            if window_pane_is_floating(&*wp) != 0 {
                let lc = (*wp).layout_cell;
                layout_cells_push_back(lcroot, layout_take_leaf(floating, lc));
            }
        }
    }
}
unsafe fn layout_assign(
    w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>,
    mut pctx: *mut layout_parse_ctx,
    floating: &mut Vec<Box<layout_cell>>,
) {
    unsafe {
        let mut w = w_owner.get();
        if !(*pctx).cctxs.is_empty() {
            layout_assign_from_ctx(w_owner, pctx);
        } else {
            layout_assign_fallback(
                w_owner,
                (*w).layout_root_ptr()
                    .map_or(std::ptr::null_mut(), |root| root),
                floating,
            );
        };
    }
}
unsafe fn layout_construct_cell(
    mut layout: *mut *const ::core::ffi::c_char,
) -> Option<Box<layout_cell>> {
    unsafe {
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
}
unsafe fn layout_construct_v1(
    layout: *mut *const ::core::ffi::c_char,
    depth: u_int,
) -> Option<Box<layout_cell>> {
    unsafe {
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
}

unsafe fn layout_parse_json(root: &json_node, pctx: *mut layout_parse_ctx) -> ::core::ffi::c_int {
    unsafe {
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
    unsafe {
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
}
unsafe fn layout_construct(
    mut input: *const ::core::ffi::c_char,
    mut pctx: *mut layout_parse_ctx,
) -> ::core::ffi::c_int {
    unsafe {
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
}
unsafe fn layout_parse_apply_ctx(
    w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>,
    mut pctx: *mut layout_parse_ctx,
) {
    unsafe {
        let mut w = w_owner.get();
        let mut wp: *mut window_pane;
        for pane_owner in (*w).z_index.snapshot() {
            wp = pane_owner.get();
            if window_pane_is_floating(&*wp) != 0 {
                window_pane_z_remove(&mut *w, &*wp);
            }
        }
        (*pctx)
            .cctxs
            .sort_unstable_by(|a, b| b.zindex.cmp(&a.zindex));
        for cctx in &(*pctx).cctxs {
            let Some(pane_owner) = (*cctx.lc).wp.upgrade() else {
                continue;
            };
            wp = pane_owner.get();
            if window_pane_is_floating(&*wp) != 0 {
                window_pane_z_insert_front(&mut *w, &*wp);
            }
        }
        for cctx in &(*pctx).cctxs {
            if cctx.active == 1 as ::core::ffi::c_int {
                if let Some(pane_owner) = (*cctx.lc).wp.upgrade() {
                    window_set_active_pane(
                        &(*(w)).observer.upgrade().expect("live window"),
                        &pane_owner,
                        1 as ::core::ffi::c_int,
                    );
                }
                break;
            }
        }
        while let Some(pane_owner) = window_pane_stack_first(w.as_ref()) {
            wp = pane_owner.get();
            window_pane_stack_remove(
                &raw mut (*w).last_panes,
                (wp).as_ref()
                    .and_then(|model| model.observer.upgrade())
                    .as_ref(),
            );
        }
        (*pctx).cctxs.sort_unstable_by(|a, b| b.last.cmp(&a.last));
        for cctx in &(*pctx).cctxs {
            let Some(pane_owner) = (*cctx.lc).wp.upgrade() else {
                continue;
            };
            wp = pane_owner.get();
            if !(cctx.last < 0 as ::core::ffi::c_int || cctx.active == 1 as ::core::ffi::c_int) {
                window_pane_stack_push(
                    &raw mut (*w).last_panes,
                    (wp).as_ref()
                        .and_then(|model| model.observer.upgrade())
                        .as_ref(),
                );
            }
        }
    }
}
unsafe fn layout_parse_ctx_check_indexes(mut pctx: *mut layout_parse_ctx) -> ::core::ffi::c_int {
    unsafe {
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
