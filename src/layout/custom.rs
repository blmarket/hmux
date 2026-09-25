use crate::src::events::events_fire_window;
use crate::src::ffi::libc::{__ctype_b_loc, qsort, sscanf, strcmp};
use crate::src::json::{
    json_array_first, json_array_next, json_destroy_node, json_find, json_find_array,
    json_find_boolean, json_find_number, json_find_object, json_find_string, json_get_object,
    json_parse,
};
use crate::src::layout::{
    layout_cell_has_tiled_child, layout_cell_is_tiled, layout_count_cells, layout_create_cell,
    layout_destroy_cell, layout_fix_offsets, layout_fix_panes, layout_free_cell, layout_make_leaf,
    layout_print_cell, layout_replace_with_node, layout_set_size,
};
use crate::src::resize::recalculate_sizes;
use crate::src::window::{
    window_count_panes, window_pane_first, window_pane_index, window_pane_is_floating,
    window_pane_last_index, window_pane_next, window_pane_stack_first, window_pane_stack_push,
    window_pane_stack_remove, window_pane_z_first, window_pane_z_insert_front, window_pane_z_next,
    window_pane_z_remove, window_pane_zindex, window_resize, window_set_active_pane,
};
use crate::src::xmalloc::xvasprintf_cstring;
use std::ffi::{CStr, CString};

macro_rules! layout_format_cause {
    ($cause:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {{
        let cause = $cause;
        if !cause.is_null() {
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

/// A temporary NUL-terminated serializer buffer.
///
/// The buffer is only used inside this module. Its final bytes become the
/// private owned layout dump or are copied for the exported C result.
struct LayoutString {
    bytes: Vec<u8>,
}

impl LayoutString {
    fn new() -> Self {
        Self { bytes: vec![0] }
    }

    fn as_c_ptr(&self) -> *const ::core::ffi::c_char {
        self.bytes.as_ptr() as *const ::core::ffi::c_char
    }

    fn append(&mut self, value: &[u8]) {
        self.bytes.pop();
        self.bytes.extend_from_slice(value);
        self.bytes.push(0);
    }

    fn remove_last_byte(&mut self) {
        assert!(self.bytes.len() >= 2, "layout serializer underflow");
        self.bytes.truncate(self.bytes.len() - 2);
        self.bytes.push(0);
    }
}
pub struct layout_parse_ctx {
    pub version: int64_t,
    pub num_active: ::core::ffi::c_int,
    pub root: *mut layout_cell,
    pub cause: *mut Option<CString>,
    pub cctxs: Vec<layout_parse_cell_ctx>,
}
#[derive(Copy, Clone)]
pub struct layout_parse_cell_ctx {
    pub lc: *mut layout_cell,
    pub active: ::core::ffi::c_int,
    pub last: ::core::ffi::c_int,
    pub index: ::core::ffi::c_int,
    pub zindex: ::core::ffi::c_int,
}

unsafe extern "C" fn layout_parse_index_cmp(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut cca: *const layout_parse_cell_ctx = a as *const layout_parse_cell_ctx;
    let mut ccb: *const layout_parse_cell_ctx = b as *const layout_parse_cell_ctx;
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*cca).index < (*ccb).index {
        retval = -(1 as ::core::ffi::c_int);
    }
    if (*cca).index > (*ccb).index {
        retval = 1 as ::core::ffi::c_int;
    }
    return retval;
}
unsafe extern "C" fn layout_parse_zindex_cmp(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut cca: *const layout_parse_cell_ctx = a as *const layout_parse_cell_ctx;
    let mut ccb: *const layout_parse_cell_ctx = b as *const layout_parse_cell_ctx;
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*cca).zindex > (*ccb).zindex {
        retval = -(1 as ::core::ffi::c_int);
    }
    if (*cca).zindex < (*ccb).zindex {
        retval = 1 as ::core::ffi::c_int;
    }
    return retval;
}
unsafe extern "C" fn layout_parse_last_cmp(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut cca: *const layout_parse_cell_ctx = a as *const layout_parse_cell_ctx;
    let mut ccb: *const layout_parse_cell_ctx = b as *const layout_parse_cell_ctx;
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*cca).last > (*ccb).last {
        retval = -(1 as ::core::ffi::c_int);
    }
    if (*cca).last < (*ccb).last {
        retval = 1 as ::core::ffi::c_int;
    }
    return retval;
}
unsafe extern "C" fn layout_string_write(
    ls: &mut LayoutString,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    unsafe {
        let value = xvasprintf_cstring(fmt, args.clone());
        ls.append(value.as_bytes());
    }
}

#[cfg(test)]
mod layout_string_tests {
    use super::{layout_string_write, LayoutString};

    #[test]
    fn serialized_layout_closes_after_removing_trailing_comma() {
        let mut layout = LayoutString::new();
        unsafe {
            layout_string_write(&mut layout, c"{\"t\":\"%c\",\"c\":[".as_ptr(), b'h' as i32);
            layout_string_write(&mut layout, c"%ux%u,".as_ptr(), 80_u32, 24_u32);
            layout.remove_last_byte();
            layout_string_write(&mut layout, c"]}".as_ptr());
        }
        assert_eq!(layout.bytes, b"{\"t\":\"h\",\"c\":[80x24]}\0");
    }
}
unsafe extern "C" fn layout_parse_init_ctx(
    mut pctx: *mut layout_parse_ctx,
    mut cause: *mut Option<CString>,
) {
    (*pctx).version = -(1 as ::core::ffi::c_int) as int64_t;
    (*pctx).num_active = 0 as ::core::ffi::c_int;
    (*pctx).root = ::core::ptr::null_mut::<layout_cell>();
    (*pctx).cause = cause;
    (*pctx).cctxs.clear();
}
unsafe extern "C" fn layout_parse_free_ctx(mut pctx: *mut layout_parse_ctx) {
    layout_free_cell((*pctx).root, 0 as ::core::ffi::c_int);
    (*pctx).root = ::core::ptr::null_mut::<layout_cell>();
    (*pctx).cctxs.clear();
}
unsafe extern "C" fn layout_parse_add_cctx(
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
unsafe extern "C" fn layout_parse_remove_cctx(
    mut pctx: *mut layout_parse_ctx,
    mut lc: *mut layout_cell,
) -> ::core::ffi::c_int {
    if let Some(i) = (*pctx).cctxs.iter().position(|cctx| cctx.lc == lc) {
        (*pctx).cctxs.swap_remove(i);
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn layout_find_bottomright(mut lc: *mut layout_cell) -> *mut layout_cell {
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return lc;
    }
    lc = layout_cells_last(&*lc);
    return layout_find_bottomright(lc);
}
unsafe extern "C" fn layout_checksum(mut layout: *const ::core::ffi::c_char) -> u_short {
    let mut csum: u_short = 0;
    csum = 0 as u_short;
    while *layout as ::core::ffi::c_int != '\0' as i32 {
        csum = ((csum as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
            + ((csum as ::core::ffi::c_int & 1 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int))
            as u_short;
        csum = (csum as ::core::ffi::c_int + *layout as ::core::ffi::c_int) as u_short;
        layout = layout.offset(1);
    }
    return csum;
}
pub(crate) unsafe fn layout_dump_owned(
    _w: *mut window,
    lcroot: *mut layout_cell,
    flags: ::core::ffi::c_int,
) -> Option<CString> {
    if lcroot.is_null() {
        return None;
    }
    let mut layout_string = LayoutString::new();
    if layout_append(lcroot, &mut layout_string, flags) != 0 {
        return None;
    }
    // Preserve the C formatter's first-NUL view of the serialized body.
    let body = CStr::from_ptr(layout_string.as_c_ptr()).to_bytes();
    let mut output = Vec::new();
    if flags & LAYOUT_CUSTOM_OLD_FORMAT != 0 {
        output.extend_from_slice(
            format!("{:04x},", layout_checksum(layout_string.as_c_ptr())).as_bytes(),
        );
        output.extend_from_slice(body);
    } else {
        output.extend_from_slice(b"{\"V\":2,\"L\":");
        output.extend_from_slice(body);
        output.push(b'}');
    }
    Some(CString::new(output).expect("layout serializer produced an interior NUL"))
}
unsafe extern "C" fn layout_append_v2(
    mut lc: *mut layout_cell,
    ls: &mut LayoutString,
) -> ::core::ffi::c_int {
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
    layout_string_write(
        ls,
        b"{\"t\":\"%c\",\"w\":%u,\"h\":%u,\"x\":%d,\"y\":%d\0" as *const u8
            as *const ::core::ffi::c_char,
        c as ::core::ffi::c_int,
        (*lc).g.sx,
        (*lc).g.sy,
        (*lc).g.xoff,
        (*lc).g.yoff,
    );
    if type_0 as ::core::ffi::c_uint
        != LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        layout_string_write(ls, b",\"c\":[\0" as *const u8 as *const ::core::ffi::c_char);
        n = 0 as u_int;
        lcchild = layout_cells_first(&*lc);
        while !lcchild.is_null() {
            if layout_append_v2(lcchild, ls) != 0 as ::core::ffi::c_int {
                return -(1 as ::core::ffi::c_int);
            }
            layout_string_write(ls, b",\0" as *const u8 as *const ::core::ffi::c_char);
            n = n.wrapping_add(1);
            lcchild = layout_cell_next(lcchild);
        }
        if n == 0 as u_int {
            return -(1 as ::core::ffi::c_int);
        }
        ls.remove_last_byte();
        layout_string_write(ls, b"]\0" as *const u8 as *const ::core::ffi::c_char);
    } else {
        wp = (*lc).wp;
        if wp.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        if wp == (*(*wp).window).active {
            layout_string_write(
                ls,
                b",\"a\":true\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else if window_pane_last_index(wp, &raw mut i) == 0 as ::core::ffi::c_int {
            layout_string_write(
                ls,
                b",\"l\":%u\0" as *const u8 as *const ::core::ffi::c_char,
                i,
            );
        }
        if window_pane_index(wp, &raw mut i) != 0 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int);
        }
        layout_string_write(
            ls,
            b",\"i\":%u\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
        if (*lc).flags & LAYOUT_CELL_FLOATING != 0
            && window_pane_zindex(wp, &raw mut i) == 0 as ::core::ffi::c_int
        {
            layout_string_write(
                ls,
                b",\"z\":%u\0" as *const u8 as *const ::core::ffi::c_char,
                i,
            );
        }
        layout_string_write(
            ls,
            b",\"I\":\"%%%u\"\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).id,
        );
    }
    layout_string_write(ls, b"}\0" as *const u8 as *const ::core::ffi::c_char);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn layout_append_v1(
    mut lc: *mut layout_cell,
    ls: &mut LayoutString,
) -> ::core::ffi::c_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut brackets: *const ::core::ffi::c_char =
        b"[]\0" as *const u8 as *const ::core::ffi::c_char;
    if lc.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    if !(*lc).wp.is_null() {
        layout_string_write(
            ls,
            b"%ux%u,%d,%d,%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*lc).g.sx,
            (*lc).g.sy,
            (*lc).g.xoff,
            (*lc).g.yoff,
            (*(*lc).wp).id,
        );
    } else {
        layout_string_write(
            ls,
            b"%ux%u,%d,%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*lc).g.sx,
            (*lc).g.sy,
            (*lc).g.xoff,
            (*lc).g.yoff,
        );
    }
    let mut current_block_16: u64;
    match (*lc).type_0 as ::core::ffi::c_uint {
        0 => {
            brackets = b"{}\0" as *const u8 as *const ::core::ffi::c_char;
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
            layout_string_write(
                ls,
                b"%c\0" as *const u8 as *const ::core::ffi::c_char,
                *brackets.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
            );
            lcchild = layout_cells_first(&*lc);
            while !lcchild.is_null() {
                if layout_append_v1(lcchild, ls) != 0 as ::core::ffi::c_int {
                    return -(1 as ::core::ffi::c_int);
                }
                layout_string_write(ls, b",\0" as *const u8 as *const ::core::ffi::c_char);
                lcchild = layout_cell_next(lcchild);
            }
            ls.remove_last_byte();
            layout_string_write(
                ls,
                b"%c\0" as *const u8 as *const ::core::ffi::c_char,
                *brackets.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
            );
        }
        _ => {}
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn layout_custom_copy_layout(mut lc: *mut layout_cell) -> *mut layout_cell {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcnewchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lconly: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcnew: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*lc).flags & LAYOUT_CELL_FLOATING != 0
    {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    lcnew = layout_create_cell(::core::ptr::null_mut::<layout_cell>());
    (*lcnew).type_0 = (*lc).type_0;
    (*lcnew).flags = (*lc).flags;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*lcnew).wp = (*lc).wp;
    }
    layout_set_size(lcnew, (*lc).g.sx, (*lc).g.sy, (*lc).g.xoff, (*lc).g.yoff);
    match (*lc).type_0 as ::core::ffi::c_uint {
        1 | 0 => {
            lcchild = layout_cells_first(&*lc);
            while !lcchild.is_null() {
                lcnewchild = layout_custom_copy_layout(lcchild);
                if !lcnewchild.is_null() {
                    layout_cells_push_back(lcnew, lcnewchild);
                    (*lcnewchild).parent = lcnew;
                }
                lcchild = layout_cell_next(lcchild);
            }
            lconly = layout_cells_first(&*lcnew);
            if lconly.is_null() {
                layout_free_cell(lcnew, 0 as ::core::ffi::c_int);
                return ::core::ptr::null_mut::<layout_cell>();
            }
            if (*lcnew).cells.children.len() == 1 {
                layout_cells_remove(lcnew, lconly);
                (*lconly).parent = ::core::ptr::null_mut::<layout_cell>();
                layout_free_cell(lcnew, 0 as ::core::ffi::c_int);
                return lconly;
            }
        }
        2 | _ => {}
    }
    return lcnew;
}
unsafe extern "C" fn layout_custom_create_compat(mut lcroot: *mut layout_cell) -> *mut layout_cell {
    let mut lccompat: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    lccompat = layout_custom_copy_layout(lcroot);
    if !lccompat.is_null() && layout_cell_is_tiled(lccompat) != 0 {
        (*lccompat).g.xoff = 0 as ::core::ffi::c_int;
        (*lccompat).g.yoff = 0 as ::core::ffi::c_int;
    }
    return lccompat;
}
unsafe extern "C" fn layout_custom_unlink_panes(mut lc: *mut layout_cell) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    match (*lc).type_0 as ::core::ffi::c_uint {
        2 => {
            (*lc).wp = ::core::ptr::null_mut::<window_pane>();
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
unsafe extern "C" fn layout_custom_free_compat(mut lcroot: *mut layout_cell) {
    if lcroot.is_null() {
        return;
    }
    layout_custom_unlink_panes(lcroot);
    layout_free_cell(lcroot, 0 as ::core::ffi::c_int);
}
unsafe extern "C" fn layout_append(
    mut lcroot: *mut layout_cell,
    ls: &mut LayoutString,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lccompat: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut result: ::core::ffi::c_int = 0;
    if flags & LAYOUT_CUSTOM_OLD_FORMAT != 0 {
        if layout_cell_is_tiled(lcroot) == 0 && layout_cell_has_tiled_child(lcroot) == 0 {
            return -(1 as ::core::ffi::c_int);
        }
        lccompat = layout_custom_create_compat(lcroot);
        result = layout_append_v1(lccompat, ls);
        layout_custom_free_compat(lccompat);
    } else {
        result = layout_append_v2(lcroot, ls);
    }
    return result;
}
unsafe extern "C" fn layout_check(mut lc: *mut layout_cell) -> ::core::ffi::c_int {
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
    mut w: *mut window,
    mut input: *const ::core::ffi::c_char,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut pctx: layout_parse_ctx = layout_parse_ctx {
        version: 0,
        num_active: 0,
        root: ::core::ptr::null_mut::<layout_cell>(),
        cause: ::core::ptr::null_mut::<Option<CString>>(),
        cctxs: Vec::new(),
    };
    let mut npanes: u_int = 0;
    let mut ncells: u_int = 0;
    let mut sx: u_int = 0 as u_int;
    let mut sy: u_int = 0 as u_int;
    let mut with_floating: ::core::ffi::c_int = 0;
    layout_parse_init_ctx(&raw mut pctx, cause);
    if layout_construct(input, &raw mut pctx) != 0 as ::core::ffi::c_int {
        layout_parse_free_ctx(&raw mut pctx);
        return -(1 as ::core::ffi::c_int);
    }
    with_floating = (pctx.version > 1 as int64_t) as ::core::ffi::c_int;
    npanes = window_count_panes(w, with_floating);
    if npanes == 0 as u_int {
        layout_format_cause!(cause, "window @{} has no panes", (*w).id,);
    } else {
        loop {
            ncells = layout_count_cells(pctx.root, with_floating);
            if npanes > ncells {
                layout_format_cause!(cause, "have {} panes but need {}", npanes, ncells,);
                current_block = 4277046812173491162;
                break;
            } else {
                if npanes == ncells {
                    current_block = 15976848397966268834;
                    break;
                }
                lcchild = layout_find_bottomright(pctx.root);
                if pctx.version > 1 as int64_t
                    && layout_parse_remove_cctx(&raw mut pctx, lcchild) != 0 as ::core::ffi::c_int
                {
                    layout_set_static_cause(
                        cause.as_mut(),
                        b"empty/missing layout parse context\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                    current_block = 4277046812173491162;
                    break;
                } else {
                    layout_destroy_cell(
                        ::core::ptr::null_mut::<window>(),
                        lcchild,
                        &raw mut pctx.root,
                    );
                }
            }
        }
        match current_block {
            4277046812173491162 => {}
            _ => {
                lc = pctx.root;
                pctx.root = ::core::ptr::null_mut::<layout_cell>();
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
                        cause.as_mut(),
                        b"size mismatch after applying layout\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                } else {
                    if layout_cell_is_tiled(lc) != 0 || layout_cell_has_tiled_child(lc) != 0 {
                        window_resize(
                            w,
                            (*lc).g.sx,
                            (*lc).g.sy,
                            -(1 as ::core::ffi::c_int),
                            -(1 as ::core::ffi::c_int),
                        );
                    }
                    if pctx.version == 1 as int64_t {
                        wp = window_pane_first(w);
                        while !wp.is_null() {
                            if !(window_pane_is_floating(wp) == 0) {
                                lcchild = (*wp).layout_cell as *mut layout_cell;
                                layout_cells_remove((*lcchild).parent, lcchild);
                                (*lcchild).parent = ::core::ptr::null_mut::<layout_cell>();
                            }
                            wp = window_pane_next(wp);
                        }
                    }
                    layout_free_cell((*w).layout_root, 0 as ::core::ffi::c_int);
                    (*w).layout_root = lc;
                    layout_assign(w, &raw mut pctx);
                    layout_fix_offsets(w);
                    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
                    if pctx.version > 1 as int64_t {
                        layout_parse_apply_ctx(w, &raw mut pctx);
                    }
                    recalculate_sizes();
                    layout_print_cell(
                        lc,
                        b"layout_parse\0" as *const u8 as *const ::core::ffi::c_char,
                        0 as u_int,
                    );
                    if pctx.version == 1 as int64_t {
                        events_fire_window(
                            b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
                            w,
                        );
                    }
                    layout_parse_free_ctx(&raw mut pctx);
                    return 0 as ::core::ffi::c_int;
                }
            }
        }
    }
    layout_free_cell(lc, 0 as ::core::ffi::c_int);
    layout_parse_free_ctx(&raw mut pctx);
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn layout_assign_from_ctx(mut w: *mut window, mut pctx: *mut layout_parse_ctx) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    qsort(
        (*pctx).cctxs.as_mut_ptr() as *mut ::core::ffi::c_void,
        (*pctx).cctxs.len() as size_t,
        ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
        Some(
            layout_parse_index_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    wp = window_pane_first(w);
    for cctx in &(*pctx).cctxs {
        layout_make_leaf(cctx.lc, wp);
        wp = window_pane_next(wp);
    }
}
unsafe extern "C" fn layout_assign_fallback_tiled(
    mut wp: *mut *mut window_pane,
    mut lc: *mut layout_cell,
) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if lc.is_null() {
        return;
    }
    match (*lc).type_0 as ::core::ffi::c_uint {
        2 => {
            while !(*wp).is_null() && !(**wp).layout_cell.is_null() {
                *wp = window_pane_next(*wp);
            }
            if (*wp).is_null() {
                return;
            }
            layout_make_leaf(lc, *wp);
            *wp = window_pane_next(*wp);
            return;
        }
        0 | 1 => {
            lcchild = layout_cells_first(&*lc);
            while !lcchild.is_null() {
                layout_assign_fallback_tiled(wp, lcchild);
                lcchild = layout_cell_next(lcchild);
            }
            return;
        }
        _ => {}
    };
}
unsafe extern "C" fn layout_assign_fallback(mut w: *mut window, mut lcroot: *mut layout_cell) {
    let mut wp: *mut window_pane = window_pane_first(w);
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    layout_assign_fallback_tiled(&raw mut wp, lcroot);
    if window_count_panes(w, 1 as ::core::ffi::c_int) > 1 as u_int
        && (*lcroot).type_0 as ::core::ffi::c_uint
            == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        lcroot = layout_replace_with_node(w, lcroot, LAYOUT_TOPBOTTOM);
    }
    wp = window_pane_first(w);
    while !wp.is_null() {
        if window_pane_is_floating(wp) != 0 {
            lc = (*wp).layout_cell as *mut layout_cell;
            layout_cells_push_back(lcroot, lc);
        }
        wp = window_pane_next(wp);
    }
}
unsafe extern "C" fn layout_assign(mut w: *mut window, mut pctx: *mut layout_parse_ctx) {
    if !(*pctx).cctxs.is_empty() {
        layout_assign_from_ctx(w, pctx);
    } else {
        layout_assign_fallback(w, (*w).layout_root);
    };
}
unsafe extern "C" fn layout_construct_cell(
    mut lcparent: *mut layout_cell,
    mut layout: *mut *const ::core::ffi::c_char,
) -> *mut layout_cell {
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
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
        return ::core::ptr::null_mut::<layout_cell>();
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
        return ::core::ptr::null_mut::<layout_cell>();
    }
    while *(*__ctype_b_loc()).offset(**layout as u_char as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
    {
        *layout = (*layout).offset(1);
    }
    if **layout as ::core::ffi::c_int != 'x' as i32 {
        return ::core::ptr::null_mut::<layout_cell>();
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
        return ::core::ptr::null_mut::<layout_cell>();
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
        return ::core::ptr::null_mut::<layout_cell>();
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
    lc = layout_create_cell(lcparent);
    (*lc).g.sx = sx;
    (*lc).g.sy = sy;
    (*lc).g.xoff = xoff;
    (*lc).g.yoff = yoff;
    return lc;
}
unsafe extern "C" fn layout_construct_v1(
    mut lcparent: *mut layout_cell,
    mut layout: *mut *const ::core::ffi::c_char,
    mut depth: u_int,
) -> *mut layout_cell {
    let mut current_block: u64;
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if depth > LAYOUT_V1_MAX_DEPTH as u_int {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    lc = layout_construct_cell(lcparent, layout);
    if lc.is_null() {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    match **layout as ::core::ffi::c_int {
        44 | 125 | 93 | 0 => return lc,
        123 => {
            (*lc).type_0 = LAYOUT_LEFTRIGHT;
            current_block = 1917311967535052937;
        }
        91 => {
            (*lc).type_0 = LAYOUT_TOPBOTTOM;
            current_block = 1917311967535052937;
        }
        _ => {
            current_block = 17291956987205268033;
        }
    }
    loop {
        match current_block {
            17291956987205268033 => {
                layout_free_cell(lc, 0 as ::core::ffi::c_int);
                return ::core::ptr::null_mut::<layout_cell>();
            }
            _ => {
                *layout = (*layout).offset(1);
                lcchild = layout_construct_v1(lc, layout, depth.wrapping_add(1 as u_int));
                if lcchild.is_null() {
                    current_block = 17291956987205268033;
                    continue;
                }
                layout_cells_push_back(lc, lcchild);
                if **layout as ::core::ffi::c_int == ',' as i32 {
                    current_block = 1917311967535052937;
                    continue;
                }
                match (*lc).type_0 as ::core::ffi::c_uint {
                    0 => {
                        if **layout as ::core::ffi::c_int != '}' as i32 {
                            current_block = 17291956987205268033;
                        } else {
                            break;
                        }
                    }
                    1 => {
                        if **layout as ::core::ffi::c_int != ']' as i32 {
                            current_block = 17291956987205268033;
                        } else {
                            break;
                        }
                    }
                    _ => {
                        current_block = 17291956987205268033;
                    }
                }
            }
        }
    }
    *layout = (*layout).offset(1);
    return lc;
}
unsafe extern "C" fn layout_parse_json(
    mut jnroot: *mut json_node,
    mut pctx: *mut layout_parse_ctx,
) -> ::core::ffi::c_int {
    let mut jn: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut object: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut num: int64_t = 0;
    let mut cause: *mut Option<CString> = (*pctx).cause;
    if json_get_object(jnroot, &raw mut jn) != 0 as ::core::ffi::c_int {
        layout_set_static_cause(
            cause.as_mut(),
            b"invalid layout json\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else if !(json_find_number(
        jn,
        b"V\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut num,
        cause,
    ) != 0 as ::core::ffi::c_int)
    {
        (*pctx).version = num;
        if !(json_find_object(
            jn,
            b"L\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut object,
            cause,
        ) != 0 as ::core::ffi::c_int)
        {
            (*pctx).root =
                layout_parse_json_layout(object, ::core::ptr::null_mut::<layout_cell>(), pctx);
            if !(*pctx).root.is_null() {
                json_destroy_node(jnroot);
                return 0 as ::core::ffi::c_int;
            }
        }
    }
    json_destroy_node(jnroot);
    if !(*pctx).root.is_null() {
        layout_free_cell((*pctx).root, 0 as ::core::ffi::c_int);
    }
    (*pctx).root = ::core::ptr::null_mut::<layout_cell>();
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn layout_parse_json_layout(
    mut node: *mut json_node,
    mut lcparent: *mut layout_cell,
    mut pctx: *mut layout_parse_ctx,
) -> *mut layout_cell {
    let mut current_block: u64;
    let mut member: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut array: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut lc: *mut layout_cell = layout_create_cell(lcparent);
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut str: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut num: int64_t = 0;
    let mut cause: *mut Option<CString> = (*pctx).cause;
    let mut boolean: ::core::ffi::c_int = 0;
    let mut index: ::core::ffi::c_int = 0;
    let mut zindex: ::core::ffi::c_int = 0;
    let mut active: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut last: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    if !(json_find_string(
        node,
        b"t\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut str,
        cause,
    ) != 0 as ::core::ffi::c_int)
    {
        if strcmp(str, b"p\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
        {
            (*lc).type_0 = LAYOUT_WINDOWPANE;
            current_block = 1394248824506584008;
        } else if strcmp(str, b"v\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            (*lc).type_0 = LAYOUT_TOPBOTTOM;
            current_block = 1394248824506584008;
        } else if strcmp(str, b"h\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            (*lc).type_0 = LAYOUT_LEFTRIGHT;
            current_block = 1394248824506584008;
        } else {
            if !cause.is_null() {
                let mut message = b"unknown cell type \"".to_vec();
                message.extend_from_slice(CStr::from_ptr(str).to_bytes());
                message.push(b'"');
                *cause = Some(CString::new(message).expect("cell type contains no NUL"));
            }
            current_block = 14858222377052930936;
        }
        match current_block {
            14858222377052930936 => {}
            _ => {
                if !(json_find_number(
                    node,
                    b"w\0" as *const u8 as *const ::core::ffi::c_char,
                    &raw mut num,
                    cause,
                ) != 0 as ::core::ffi::c_int)
                {
                    if num < PANE_MINIMUM as int64_t || num > PANE_MAXIMUM as int64_t {
                        layout_format_cause!(
                            cause,
                            "invalid width {}",
                            num as ::core::ffi::c_longlong,
                        );
                    } else {
                        (*lc).g.sx = num as u_int;
                        if !(json_find_number(
                            node,
                            b"h\0" as *const u8 as *const ::core::ffi::c_char,
                            &raw mut num,
                            cause,
                        ) != 0 as ::core::ffi::c_int)
                        {
                            if num < PANE_MINIMUM as int64_t || num > PANE_MAXIMUM as int64_t {
                                layout_format_cause!(
                                    cause,
                                    "invalid height {}",
                                    num as ::core::ffi::c_longlong,
                                );
                            } else {
                                (*lc).g.sy = num as u_int;
                                if !(json_find_number(
                                    node,
                                    b"x\0" as *const u8 as *const ::core::ffi::c_char,
                                    &raw mut num,
                                    cause,
                                ) != 0 as ::core::ffi::c_int)
                                {
                                    if num < -WINDOW_MAXIMUM as int64_t
                                        || num > WINDOW_MAXIMUM as int64_t
                                    {
                                        layout_format_cause!(
                                            cause,
                                            "invalid x-offset {}",
                                            num as ::core::ffi::c_longlong,
                                        );
                                    } else {
                                        (*lc).g.xoff = num as ::core::ffi::c_int;
                                        if !(json_find_number(
                                            node,
                                            b"y\0" as *const u8 as *const ::core::ffi::c_char,
                                            &raw mut num,
                                            cause,
                                        ) != 0 as ::core::ffi::c_int)
                                        {
                                            if num < -WINDOW_MAXIMUM as int64_t
                                                || num > WINDOW_MAXIMUM as int64_t
                                            {
                                                layout_format_cause!(
                                                    cause,
                                                    "invalid y-offset {}",
                                                    num as ::core::ffi::c_longlong,
                                                );
                                            } else {
                                                (*lc).g.yoff = num as ::core::ffi::c_int;
                                                if (*lc).type_0 as ::core::ffi::c_uint
                                                    == LAYOUT_WINDOWPANE as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    if !json_find(
                                                        node,
                                                        b"c\0" as *const u8
                                                            as *const ::core::ffi::c_char,
                                                    )
                                                    .is_null()
                                                    {
                                                        layout_set_static_cause(
                                                            cause.as_mut(),
                                                            b"panes cannot have children\0"
                                                                as *const u8
                                                                as *const ::core::ffi::c_char,
                                                        );
                                                        current_block = 14858222377052930936;
                                                    } else if json_find_number(
                                                        node,
                                                        b"i\0" as *const u8
                                                            as *const ::core::ffi::c_char,
                                                        &raw mut num,
                                                        cause,
                                                    ) != 0 as ::core::ffi::c_int
                                                    {
                                                        current_block = 14858222377052930936;
                                                    } else if num < 0 as int64_t
                                                        || num > INT_MAX as int64_t
                                                    {
                                                        layout_format_cause!(
                                                            cause,
                                                            "invalid index {}",
                                                            num as ::core::ffi::c_longlong,
                                                        );
                                                        current_block = 14858222377052930936;
                                                    } else {
                                                        index = num as ::core::ffi::c_int;
                                                        if !json_find(
                                                            node,
                                                            b"a\0" as *const u8
                                                                as *const ::core::ffi::c_char,
                                                        )
                                                        .is_null()
                                                        {
                                                            if json_find_boolean(
                                                                node,
                                                                b"a\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                &raw mut boolean,
                                                                cause,
                                                            ) != 0 as ::core::ffi::c_int
                                                            {
                                                                current_block =
                                                                    14858222377052930936;
                                                            } else {
                                                                active = boolean;
                                                                if active != 0 {
                                                                    (*pctx).num_active += 1;
                                                                }
                                                                current_block = 6450597802325118133;
                                                            }
                                                        } else if !json_find(
                                                            node,
                                                            b"l\0" as *const u8
                                                                as *const ::core::ffi::c_char,
                                                        )
                                                        .is_null()
                                                        {
                                                            if json_find_number(
                                                                node,
                                                                b"l\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                &raw mut num,
                                                                cause,
                                                            ) != 0 as ::core::ffi::c_int
                                                            {
                                                                current_block =
                                                                    14858222377052930936;
                                                            } else if num < 0 as int64_t
                                                                || num > INT_MAX as int64_t
                                                            {
                                                                layout_format_cause!(
                                                                    cause,
                                                                    "invalid last {}",
                                                                    num as ::core::ffi::c_longlong,
                                                                );
                                                                current_block =
                                                                    14858222377052930936;
                                                            } else {
                                                                last = num as ::core::ffi::c_int;
                                                                current_block = 6450597802325118133;
                                                            }
                                                        } else {
                                                            current_block = 6450597802325118133;
                                                        }
                                                        match current_block {
                                                            14858222377052930936 => {}
                                                            _ => {
                                                                if !json_find(
                                                                        node,
                                                                        b"z\0" as *const u8 as *const ::core::ffi::c_char,
                                                                    )
                                                                    .is_null()
                                                                {
                                                                    if json_find_number(
                                                                        node,
                                                                        b"z\0" as *const u8 as *const ::core::ffi::c_char,
                                                                        &raw mut num,
                                                                        cause,
                                                                    ) != 0 as ::core::ffi::c_int
                                                                    {
                                                                        current_block = 14858222377052930936;
                                                                    } else if num < 0 as int64_t
                                                                        || num > (INT_MAX - 1 as ::core::ffi::c_int) as int64_t
                                                                    {
                                                                        layout_format_cause!(
                                                                            cause,
                                                                            "invalid floating zindex {}",
                                                                            num as ::core::ffi::c_longlong,
                                                                        );
                                                                        current_block = 14858222377052930936;
                                                                    } else {
                                                                        zindex = num as ::core::ffi::c_int;
                                                                        (*lc).flags |= LAYOUT_CELL_FLOATING;
                                                                        current_block = 1345366029464561491;
                                                                    }
                                                                } else {
                                                                    zindex = INT_MAX;
                                                                    current_block = 1345366029464561491;
                                                                }
                                                                match current_block {
                                                                    14858222377052930936 => {}
                                                                    _ => {
                                                                        layout_parse_add_cctx(
                                                                            pctx, lc, active, last,
                                                                            index, zindex,
                                                                        );
                                                                        current_block =
                                                                            11441799814184323368;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else if json_find_array(
                                                    node,
                                                    b"c\0" as *const u8
                                                        as *const ::core::ffi::c_char,
                                                    &raw mut array,
                                                    cause,
                                                ) != 0 as ::core::ffi::c_int
                                                {
                                                    current_block = 14858222377052930936;
                                                } else {
                                                    member = json_array_first(array);
                                                    if member.is_null()
                                                        || json_array_next(member).is_null()
                                                    {
                                                        layout_set_static_cause(
                                                            cause.as_mut(),
                                                            b"nodes must have more than one child\0"
                                                                as *const u8
                                                                as *const ::core::ffi::c_char,
                                                        );
                                                        current_block = 14858222377052930936;
                                                    } else {
                                                        loop {
                                                            if member.is_null() {
                                                                current_block =
                                                                    11441799814184323368;
                                                                break;
                                                            }
                                                            lcchild = layout_parse_json_layout(
                                                                member, lc, pctx,
                                                            );
                                                            if lcchild.is_null() {
                                                                current_block =
                                                                    14858222377052930936;
                                                                break;
                                                            }
                                                            layout_cells_push_back(lc, lcchild);
                                                            member = json_array_next(member);
                                                        }
                                                    }
                                                }
                                                match current_block {
                                                    14858222377052930936 => {}
                                                    _ => return lc,
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    layout_free_cell(lc, 0 as ::core::ffi::c_int);
    return ::core::ptr::null_mut::<layout_cell>();
}
unsafe extern "C" fn layout_construct(
    mut input: *const ::core::ffi::c_char,
    mut pctx: *mut layout_parse_ctx,
) -> ::core::ffi::c_int {
    let mut json: *mut json_node = ::core::ptr::null_mut::<json_node>();
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
                (*pctx).cause.as_mut(),
                b"malformed layout header\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        input = input.offset(n as isize);
        if csum as ::core::ffi::c_int != layout_checksum(input) as ::core::ffi::c_int {
            layout_set_static_cause(
                (*pctx).cause.as_mut(),
                b"invalid layout checksum\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        (*pctx).root = layout_construct_v1(
            ::core::ptr::null_mut::<layout_cell>(),
            &raw mut input,
            0 as u_int,
        );
        if (*pctx).root.is_null() {
            layout_set_static_cause(
                (*pctx).cause.as_mut(),
                b"invalid layout\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        if *input as ::core::ffi::c_int != '\0' as i32 {
            layout_set_static_cause(
                (*pctx).cause.as_mut(),
                b"trailing data\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        (*pctx).version = 1 as int64_t;
    } else {
        json = json_parse(input, (*pctx).cause);
        if json.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        if layout_parse_json(json, pctx) != 0 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int);
        }
        if (*pctx).version != 2 as int64_t {
            layout_set_static_cause(
                (*pctx).cause.as_mut(),
                b"version mismatch\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        if (*pctx).num_active > 1 as ::core::ffi::c_int {
            layout_set_static_cause(
                (*pctx).cause.as_mut(),
                b"more than one active pane\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        if (*pctx).cctxs.is_empty() {
            layout_set_static_cause(
                (*pctx).cause.as_mut(),
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
unsafe extern "C" fn layout_parse_apply_ctx(mut w: *mut window, mut pctx: *mut layout_parse_ctx) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wpnext: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    wp = window_pane_z_first(w);
    while !wp.is_null() {
        wpnext = window_pane_z_next(wp);
        if window_pane_is_floating(wp) != 0 {
            window_pane_z_remove(w, wp);
        }
        wp = wpnext;
    }
    qsort(
        (*pctx).cctxs.as_mut_ptr() as *mut ::core::ffi::c_void,
        (*pctx).cctxs.len() as size_t,
        ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
        Some(
            layout_parse_zindex_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    for cctx in &(*pctx).cctxs {
        wp = (*cctx.lc).wp;
        if window_pane_is_floating(wp) != 0 {
            window_pane_z_insert_front(w, wp);
        }
    }
    for cctx in &(*pctx).cctxs {
        if cctx.active == 1 as ::core::ffi::c_int {
            window_set_active_pane(w, (*cctx.lc).wp, 1 as ::core::ffi::c_int);
            break;
        }
    }
    while !window_pane_stack_first(w).is_null() {
        wp = window_pane_stack_first(w);
        window_pane_stack_remove(&raw mut (*w).last_panes, wp);
    }
    qsort(
        (*pctx).cctxs.as_mut_ptr() as *mut ::core::ffi::c_void,
        (*pctx).cctxs.len() as size_t,
        ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
        Some(
            layout_parse_last_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    for cctx in &(*pctx).cctxs {
        wp = (*cctx.lc).wp;
        if !(cctx.last < 0 as ::core::ffi::c_int || cctx.active == 1 as ::core::ffi::c_int) {
            window_pane_stack_push(&raw mut (*w).last_panes, wp);
        }
    }
}
unsafe extern "C" fn layout_parse_ctx_check_indexes(
    mut pctx: *mut layout_parse_ctx,
) -> ::core::ffi::c_int {
    qsort(
        (*pctx).cctxs.as_mut_ptr() as *mut ::core::ffi::c_void,
        (*pctx).cctxs.len() as size_t,
        ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
        Some(
            layout_parse_index_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    if (*pctx)
        .cctxs
        .windows(2)
        .any(|pair| pair[0].index == pair[1].index)
    {
        layout_set_static_cause(
            (*pctx).cause.as_mut(),
            b"duplicate pane index\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return 0 as ::core::ffi::c_int;
    }
    qsort(
        (*pctx).cctxs.as_mut_ptr() as *mut ::core::ffi::c_void,
        (*pctx).cctxs.len() as size_t,
        ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
        Some(
            layout_parse_zindex_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
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
            (*pctx).cause.as_mut(),
            b"duplicate pane z-index\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return 0 as ::core::ffi::c_int;
    }
    qsort(
        (*pctx).cctxs.as_mut_ptr() as *mut ::core::ffi::c_void,
        (*pctx).cctxs.len() as size_t,
        ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
        Some(
            layout_parse_last_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
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
            (*pctx).cause.as_mut(),
            b"duplicate last pane index\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
