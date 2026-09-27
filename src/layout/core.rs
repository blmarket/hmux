use crate::src::arguments::{
    args_has, args_percentage_and_expand_result, args_strtonum_and_expand_result,
};
use crate::src::events::events_fire_window;
use crate::src::ffi::libc::memcpy;
use crate::src::log::{fatalx, log_cstr, log_cstr_width, log_debug, log_pointer};
use crate::src::screen_redraw::redraw_invalidate_scene;
use crate::src::shared::abi::*;
pub use crate::src::shared::arguments::args;
pub use crate::src::shared::command::cmdq_item;
pub use crate::src::shared::environment::environ;
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::key::key_event;
pub use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cells};
pub use crate::src::shared::limits::{INT_MAX, UINT_MAX};
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::options;
pub use crate::src::shared::pane::window_pane;
pub use crate::src::shared::pane::{
    PANE_MAXIMUM, PANE_MINIMUM, PANE_REDRAWSCROLLBAR, PANE_SCROLLBARS_ALWAYS, PANE_SCROLLBARS_LEFT,
    PANE_STATUS_BOTTOM, PANE_STATUS_TOP,
};
pub use crate::src::shared::spawn::{
    SPAWN_BEFORE, SPAWN_FLOATOVERZOOM, SPAWN_FULLSIZE, SPAWN_HORIZONTAL, SPAWN_SPLIT, SPAWN_ZOOM,
};
use crate::src::shared::style::*;
pub use crate::src::shared::tty::tty_term;
pub use crate::src::shared::window::window;
pub use crate::src::window::window_pane_resize;
use crate::src::window::{
    window_active_pane_is_over_zoom, window_get_pane_status, window_pane_first,
    window_pane_get_pane_lines, window_pane_get_pane_status, window_pane_is_floating,
    window_pane_next, window_pane_scrollbar_reserve, window_pane_z_first, window_pane_z_next,
    window_push_zoom,
};
use std::ffi::{CStr, CString};

unsafe fn layout_geometry_init(mut lg: *mut layout_geometry) {
    (*lg).sx = UINT_MAX as u_int;
    (*lg).sy = UINT_MAX as u_int;
    (*lg).xoff = INT_MAX;
    (*lg).yoff = INT_MAX;
}
pub fn layout_create_cell() -> Box<layout_cell> {
    Box::new(layout_cell {
        type_0: LAYOUT_WINDOWPANE,
        flags: 0,
        parent: std::ptr::null_mut(),
        sibling_index: 0,
        g: layout_geometry {
            sx: UINT_MAX,
            sy: UINT_MAX,
            xoff: INT_MAX,
            yoff: INT_MAX,
        },
        fg: layout_geometry {
            sx: UINT_MAX,
            sy: UINT_MAX,
            xoff: INT_MAX,
            yoff: INT_MAX,
        },
        wp: std::ptr::null_mut(),
        cells: layout_cells {
            children: Vec::new(),
        },
    })
}

/// Free internal nodes while keeping detached pane cells alive for a new layout.
pub fn layout_take_leaves(root: Option<Box<layout_cell>>) -> Vec<Box<layout_cell>> {
    fn collect(mut cell: Box<layout_cell>, leaves: &mut Vec<Box<layout_cell>>) {
        cell.parent = std::ptr::null_mut();
        cell.sibling_index = 0;
        if cell.type_0 == LAYOUT_WINDOWPANE {
            leaves.push(cell);
        } else {
            for child in std::mem::take(&mut cell.cells.children) {
                collect(child, leaves);
            }
        }
    }
    let mut leaves = Vec::new();
    if let Some(root) = root {
        collect(root, &mut leaves);
    }
    leaves
}

pub fn layout_take_leaf(
    leaves: &mut Vec<Box<layout_cell>>,
    cell: *mut layout_cell,
) -> Box<layout_cell> {
    let index = leaves
        .iter()
        .position(|leaf| std::ptr::eq(&**leaf, cell))
        .expect("detached layout leaf is owned");
    leaves.remove(index)
}

pub unsafe fn layout_print_cell(
    mut lc: *mut layout_cell,
    mut hdr: *const ::core::ffi::c_char,
    mut n: u_int,
) {
    let _lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut type_0: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if lc.is_null() {
        return;
    }
    match (*lc).type_0 as ::core::ffi::c_uint {
        0 => {
            type_0 = b"LEFTRIGHT\0" as *const u8 as *const ::core::ffi::c_char;
        }
        1 => {
            type_0 = b"TOPBOTTOM\0" as *const u8 as *const ::core::ffi::c_char;
        }
        2 => {
            type_0 = b"WINDOWPANE\0" as *const u8 as *const ::core::ffi::c_char;
        }
        _ => {
            type_0 = b"UNKNOWN\0" as *const u8 as *const ::core::ffi::c_char;
        }
    }
    log_debug(format_args!(
        "{}:{}{} type {} [parent {}] wp={} [{},{} {}x{}]",
        log_cstr((hdr) as *const _),
        log_cstr_width(
            (b" \0" as *const u8 as *const ::core::ffi::c_char) as *const _,
            n as i32
        ),
        log_pointer((lc) as *const ::core::ffi::c_void),
        log_cstr((type_0) as *const _),
        log_pointer(((*lc).parent) as *const ::core::ffi::c_void),
        log_pointer(((*lc).wp) as *const ::core::ffi::c_void),
        ((*lc).g.xoff) as i32,
        ((*lc).g.yoff) as i32,
        ((*lc).g.sx) as u32,
        ((*lc).g.sy) as u32
    ));
    match (*lc).type_0 as ::core::ffi::c_uint {
        0 | 1 => {
            let children = (*lc)
                .cells
                .children
                .iter()
                .map(|child| &**child as *const layout_cell as *mut layout_cell)
                .collect::<Vec<_>>();
            for lcchild in children {
                layout_print_cell(lcchild, hdr, n.wrapping_add(1 as u_int));
            }
        }
        2 | _ => {}
    };
}
pub unsafe fn layout_search_by_border(
    mut lc: *mut layout_cell,
    mut x: u_int,
    mut y: u_int,
) -> *mut layout_cell {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut last: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    lcchild = layout_cells_first(&*lc);
    while !lcchild.is_null() {
        if x as ::core::ffi::c_int >= (*lcchild).g.xoff
            && (x as ::core::ffi::c_int) < (*lcchild).g.xoff + (*lcchild).g.sx as ::core::ffi::c_int
            && y as ::core::ffi::c_int >= (*lcchild).g.yoff
            && (y as ::core::ffi::c_int) < (*lcchild).g.yoff + (*lcchild).g.sy as ::core::ffi::c_int
        {
            return layout_search_by_border(lcchild, x, y);
        }
        if last.is_null() {
            last = lcchild;
        } else {
            match (*lc).type_0 as ::core::ffi::c_uint {
                0 => {
                    if (x as ::core::ffi::c_int) < (*lcchild).g.xoff
                        && x as ::core::ffi::c_int
                            >= (*last).g.xoff + (*last).g.sx as ::core::ffi::c_int
                    {
                        return last;
                    }
                }
                1 => {
                    if (y as ::core::ffi::c_int) < (*lcchild).g.yoff
                        && y as ::core::ffi::c_int
                            >= (*last).g.yoff + (*last).g.sy as ::core::ffi::c_int
                    {
                        return last;
                    }
                }
                2 | _ => {}
            }
            last = lcchild;
        }
        lcchild = layout_cell_next(lcchild);
    }
    return ::core::ptr::null_mut::<layout_cell>();
}
pub unsafe fn layout_set_size(
    mut lc: *mut layout_cell,
    mut sx: u_int,
    mut sy: u_int,
    mut xoff: ::core::ffi::c_int,
    mut yoff: ::core::ffi::c_int,
) {
    (*lc).g.sx = sx;
    (*lc).g.sy = sy;
    (*lc).g.xoff = xoff;
    (*lc).g.yoff = yoff;
}
pub unsafe fn layout_make_leaf(mut lc: *mut layout_cell, mut wp: *mut window_pane) {
    (*lc).type_0 = LAYOUT_WINDOWPANE;
    layout_cells_require_empty(&*lc);
    (*wp).layout_cell = lc as *mut layout_cell;
    (*lc).wp = wp;
}
pub unsafe fn layout_make_node(mut lc: *mut layout_cell, mut type_0: layout_type) {
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        fatalx(|out| out.write_all(b"bad layout type"));
    }
    (*lc).type_0 = type_0;
    layout_cells_require_empty(&*lc);
    if !(*lc).wp.is_null() {
        (*(*lc).wp).layout_cell = ::core::ptr::null_mut::<layout_cell>();
    }
    (*lc).wp = ::core::ptr::null_mut::<window_pane>();
}
pub unsafe fn layout_cell_is_tiled(mut lc: *mut layout_cell) -> ::core::ffi::c_int {
    let mut is_leaf: ::core::ffi::c_int = ((*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
    let mut is_floating: ::core::ffi::c_int = (*lc).flags & LAYOUT_CELL_FLOATING;
    return (is_leaf != 0 && is_floating == 0) as ::core::ffi::c_int;
}
pub unsafe fn layout_cell_has_tiled_child(mut lc: *mut layout_cell) -> ::core::ffi::c_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as ::core::ffi::c_int;
    }
    lcchild = layout_cells_first(&*lc);
    while !lcchild.is_null() {
        if layout_cell_is_tiled(lcchild) != 0 || layout_cell_has_tiled_child(lcchild) != 0 {
            return 1 as ::core::ffi::c_int;
        }
        lcchild = layout_cell_next(lcchild);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn layout_cell_is_first_tiled(mut lc: *mut layout_cell) -> ::core::ffi::c_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = (*lc).parent;
    if lcparent.is_null() {
        return layout_cell_is_tiled(lc);
    }
    lcchild = layout_cells_first(&*lcparent);
    while !lcchild.is_null() {
        if layout_cell_is_tiled(lcchild) != 0 || layout_cell_has_tiled_child(lcchild) != 0 {
            break;
        }
        lcchild = layout_cell_next(lcchild);
    }
    return (lcchild == lc) as ::core::ffi::c_int;
}
unsafe fn layout_cell_get_first_tiled(mut lc: *mut layout_cell) -> *mut layout_cell {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild2: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if layout_cell_is_tiled(lc) != 0 {
        return lc;
    }
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    lcchild = layout_cells_first(&*lc);
    while !lcchild.is_null() {
        if layout_cell_is_tiled(lcchild) != 0 {
            return lcchild;
        }
        if (*lcchild).type_0 as ::core::ffi::c_uint
            != LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            lcchild2 = layout_cell_get_first_tiled(lcchild);
            if !lcchild2.is_null() {
                return lcchild2;
            }
        }
        lcchild = layout_cell_next(lcchild);
    }
    return ::core::ptr::null_mut::<layout_cell>();
}
unsafe fn layout_fix_offsets1(mut lc: *mut layout_cell) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        xoff = (*lc).g.xoff;
        lcchild = layout_cells_first(&*lc);
        while !lcchild.is_null() {
            if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
                (*lcchild).g.xoff = xoff;
                (*lcchild).g.yoff = (*lc).g.yoff;
                if (*lcchild).type_0 as ::core::ffi::c_uint
                    != LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    layout_fix_offsets1(lcchild);
                }
                xoff = (xoff as u_int).wrapping_add((*lcchild).g.sx.wrapping_add(1 as u_int))
                    as ::core::ffi::c_int as ::core::ffi::c_int;
            }
            lcchild = layout_cell_next(lcchild);
        }
    } else {
        yoff = (*lc).g.yoff;
        lcchild = layout_cells_first(&*lc);
        while !lcchild.is_null() {
            if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
                (*lcchild).g.xoff = (*lc).g.xoff;
                (*lcchild).g.yoff = yoff;
                if (*lcchild).type_0 as ::core::ffi::c_uint
                    != LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    layout_fix_offsets1(lcchild);
                }
                yoff = (yoff as u_int).wrapping_add((*lcchild).g.sy.wrapping_add(1 as u_int))
                    as ::core::ffi::c_int as ::core::ffi::c_int;
            }
            lcchild = layout_cell_next(lcchild);
        }
    };
}
pub unsafe fn layout_fix_offsets(mut w: *mut window) {
    let mut lc: *mut layout_cell = (*w).layout_root_ptr().map_or(std::ptr::null_mut(), |root| root);
    if (*lc).flags & LAYOUT_CELL_FLOATING != 0 {
        return;
    }
    (*lc).g.xoff = 0 as ::core::ffi::c_int;
    (*lc).g.yoff = 0 as ::core::ffi::c_int;
    layout_fix_offsets1(lc);
}
unsafe fn layout_cell_is_last_tiled(mut lc: *mut layout_cell) -> ::core::ffi::c_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = (*lc).parent;
    if lcparent.is_null() {
        return layout_cell_is_tiled(lc);
    }
    lcchild = layout_cells_last(&*lcparent);
    while !lcchild.is_null() {
        if layout_cell_is_tiled(lcchild) != 0 || layout_cell_has_tiled_child(lcchild) != 0 {
            break;
        }
        lcchild = layout_cell_prev(lcchild);
    }
    return (lcchild == lc) as ::core::ffi::c_int;
}
unsafe fn layout_cell_is_top(
    mut root: *mut layout_cell,
    mut lc: *mut layout_cell,
) -> ::core::ffi::c_int {
    let mut next: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    while lc != root {
        next = (*lc).parent;
        if next.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        if (*next).type_0 as ::core::ffi::c_uint
            == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            && layout_cell_is_first_tiled(lc) == 0
        {
            return 0 as ::core::ffi::c_int;
        }
        lc = next;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn layout_cell_is_bottom(
    mut root: *mut layout_cell,
    mut lc: *mut layout_cell,
) -> ::core::ffi::c_int {
    let mut next: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    while lc != root {
        next = (*lc).parent;
        if next.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        if (*next).type_0 as ::core::ffi::c_uint
            == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            && layout_cell_is_last_tiled(lc) == 0
        {
            return 0 as ::core::ffi::c_int;
        }
        lc = next;
    }
    return 1 as ::core::ffi::c_int;
}
pub unsafe fn layout_add_horizontal_border(
    mut root: *mut layout_cell,
    mut lc: *mut layout_cell,
    mut status: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if status == PANE_STATUS_TOP {
        return layout_cell_is_top(root, lc);
    }
    if status == PANE_STATUS_BOTTOM {
        return layout_cell_is_bottom(root, lc);
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn layout_fix_panes(mut w: *mut window, mut skip: *mut window_pane) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut root: *mut layout_cell = (*w).layout_root_ptr().map_or(std::ptr::null_mut(), |root| root);
    let mut status: ::core::ffi::c_int = 0;
    let mut sb_w: ::core::ffi::c_int = 0;
    let mut sb_pad: ::core::ffi::c_int = 0;
    let mut old_xoff: ::core::ffi::c_int = 0;
    let mut old_yoff: ::core::ffi::c_int = 0;
    let mut changed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut old_sx: u_int = 0;
    let mut old_sy: u_int = 0;
    wp = window_pane_first(w);
    while !wp.is_null() {
        lc = (*wp).layout_cell as *mut layout_cell;
        if !(lc.is_null() || wp == skip) {
            old_xoff = (*wp).xoff;
            old_yoff = (*wp).yoff;
            old_sx = (*wp).sx;
            old_sy = (*wp).sy;
            (*wp).xoff = (*lc).g.xoff;
            (*wp).yoff = (*lc).g.yoff;
            sx = (*lc).g.sx;
            sy = (*lc).g.sy;
            status = window_pane_get_pane_status(wp);
            if window_pane_is_floating(wp) == 0
                && layout_add_horizontal_border(root, lc, status) != 0
            {
                if status == PANE_STATUS_TOP {
                    (*wp).yoff += 1;
                }
                if sy > 1 as u_int {
                    sy = sy.wrapping_sub(1);
                }
            }
            if window_pane_scrollbar_reserve(wp) != 0 {
                sb_w = (*wp).scrollbar_style.width;
                sb_pad = (*wp).scrollbar_style.pad;
                if sb_w < 1 as ::core::ffi::c_int {
                    sb_w = 1 as ::core::ffi::c_int;
                }
                if sb_pad < 0 as ::core::ffi::c_int {
                    sb_pad = 0 as ::core::ffi::c_int;
                }
                if (*w).sb_pos == PANE_SCROLLBARS_LEFT {
                    if sx as ::core::ffi::c_int - sb_w - sb_pad < PANE_MINIMUM {
                        (*wp).xoff = (*wp).xoff + sx as ::core::ffi::c_int - PANE_MINIMUM;
                        sx = PANE_MINIMUM as u_int;
                    } else {
                        sx = sx.wrapping_sub(sb_w as u_int).wrapping_sub(sb_pad as u_int);
                        (*wp).xoff = (*wp).xoff + sb_w + sb_pad;
                    }
                } else if sx as ::core::ffi::c_int - sb_w - sb_pad < PANE_MINIMUM {
                    sx = PANE_MINIMUM as u_int;
                } else {
                    sx = sx.wrapping_sub(sb_w as u_int).wrapping_sub(sb_pad as u_int);
                }
                (*wp).flags |= PANE_REDRAWSCROLLBAR;
            }
            window_pane_resize(wp, sx, sy);
            if (*wp).xoff != old_xoff
                || (*wp).yoff != old_yoff
                || (*wp).sx != old_sx
                || (*wp).sy != old_sy
            {
                changed = 1 as ::core::ffi::c_int;
            }
        }
        wp = window_pane_next(wp);
    }
    if changed != 0 {
        redraw_invalidate_scene(w);
    }
}
pub unsafe fn layout_count_cells(
    mut lc: *mut layout_cell,
    mut with_floating: ::core::ffi::c_int,
) -> u_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut count: u_int = 0 as u_int;
    match (*lc).type_0 as ::core::ffi::c_uint {
        2 => {
            if (*lc).flags & LAYOUT_CELL_FLOATING != 0 && with_floating == 0 {
                return 0 as u_int;
            }
            return 1 as u_int;
        }
        0 | 1 => {
            lcchild = layout_cells_first(&*lc);
            while !lcchild.is_null() {
                count = count.wrapping_add(layout_count_cells(lcchild, with_floating));
                lcchild = layout_cell_next(lcchild);
            }
            return count;
        }
        _ => {
            fatalx(|out| out.write_all(b"bad layout type"));
        }
    };
}
unsafe fn layout_resize_check(
    mut w: *mut window,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
) -> u_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut root: *mut layout_cell = (*w).layout_root_ptr().map_or(std::ptr::null_mut(), |root| root);
    let mut sb_style: *mut style = &raw mut (*(*w).active).scrollbar_style;
    let mut available: u_int = 0;
    let mut minimum: u_int = 0;
    let mut status: ::core::ffi::c_int = 0;
    status = window_get_pane_status(w);
    if layout_cell_is_tiled(lc) == 0 && layout_cell_has_tiled_child(lc) == 0 {
        return 0 as u_int;
    }
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if type_0 as ::core::ffi::c_uint
            == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            available = (*lc).g.sx;
            if (*w).sb == PANE_SCROLLBARS_ALWAYS {
                minimum = (PANE_MINIMUM + (*sb_style).width + (*sb_style).pad) as u_int;
            } else {
                minimum = PANE_MINIMUM as u_int;
            }
        } else {
            available = (*lc).g.sy;
            if layout_add_horizontal_border(root, lc, status) != 0 {
                minimum = (PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int;
            } else {
                minimum = PANE_MINIMUM as u_int;
            }
        }
        if available > minimum {
            available = available.wrapping_sub(minimum);
        } else {
            available = 0 as u_int;
        }
    } else if (*lc).type_0 as ::core::ffi::c_uint == type_0 as ::core::ffi::c_uint {
        available = 0 as u_int;
        lcchild = layout_cells_first(&*lc);
        while !lcchild.is_null() {
            available = available.wrapping_add(layout_resize_check(w, lcchild, type_0));
            lcchild = layout_cell_next(lcchild);
        }
    } else {
        minimum = UINT_MAX as u_int;
        lcchild = layout_cells_first(&*lc);
        while !lcchild.is_null() {
            if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
                available = layout_resize_check(w, lcchild, type_0);
                if available < minimum {
                    minimum = available;
                }
            }
            lcchild = layout_cell_next(lcchild);
        }
        available = minimum;
    }
    return available;
}
pub unsafe fn layout_resize_adjust(
    mut w: *mut window,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
    mut change: ::core::ffi::c_int,
) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut changed: ::core::ffi::c_int = 0;
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*lc).g.sx = (*lc).g.sx.wrapping_add(change as u_int);
    } else {
        (*lc).g.sy = (*lc).g.sy.wrapping_add(change as u_int);
    }
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    if (*lc).type_0 as ::core::ffi::c_uint != type_0 as ::core::ffi::c_uint {
        lcchild = layout_cells_first(&*lc);
        while !lcchild.is_null() {
            if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
                layout_resize_adjust(w, lcchild, type_0, change);
            }
            lcchild = layout_cell_next(lcchild);
        }
        return;
    }
    if layout_cell_has_tiled_child(lc) == 0 {
        return;
    }
    while change != 0 as ::core::ffi::c_int {
        changed = 0 as ::core::ffi::c_int;
        lcchild = layout_cells_first(&*lc);
        while !lcchild.is_null() {
            if change == 0 as ::core::ffi::c_int {
                break;
            }
            if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
                if change > 0 as ::core::ffi::c_int {
                    layout_resize_adjust(w, lcchild, type_0, 1 as ::core::ffi::c_int);
                    change -= 1;
                    changed = 1 as ::core::ffi::c_int;
                } else if layout_resize_check(w, lcchild, type_0) > 0 as u_int {
                    layout_resize_adjust(w, lcchild, type_0, -(1 as ::core::ffi::c_int));
                    change += 1;
                    changed = 1 as ::core::ffi::c_int;
                }
            }
            lcchild = layout_cell_next(lcchild);
        }
        if changed == 0 {
            break;
        }
    }
}
pub unsafe fn layout_resize_set_size(
    mut w: *mut window,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
    mut size: u_int,
) {
    let mut change: ::core::ffi::c_int = 0;
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        change = size.wrapping_sub((*lc).g.sx) as ::core::ffi::c_int;
    } else {
        change = size.wrapping_sub((*lc).g.sy) as ::core::ffi::c_int;
    }
    layout_resize_adjust(w, lc, type_0, change);
}
unsafe fn layout_cell_get_neighbour_dir(
    mut lc: *mut layout_cell,
    mut direction: ::core::ffi::c_int,
) -> *mut layout_cell {
    let mut lcn: *mut layout_cell = lc;
    loop {
        if direction != 0 {
            lcn = layout_cell_next(lcn);
        } else {
            lcn = layout_cell_prev(lcn);
        }
        if lcn.is_null() || layout_cell_is_tiled(lcn) != 0 || layout_cell_has_tiled_child(lcn) != 0
        {
            return lcn;
        }
    }
}
pub unsafe fn layout_cell_get_neighbour(mut lc: *mut layout_cell) -> *mut layout_cell {
    let mut lcother: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = (*lc).parent;
    let mut direction: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    if lcparent.is_null() {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    if lc == layout_cells_last(&*lcparent) {
        direction = (direction == 0) as ::core::ffi::c_int;
    }
    lcother = layout_cell_get_neighbour_dir(lc, direction);
    if lcother.is_null() {
        lcother = layout_cell_get_neighbour_dir(lc, (direction == 0) as ::core::ffi::c_int);
    }
    return lcother;
}
pub unsafe fn layout_destroy_cell(
    w: *mut window,
    lc: *mut layout_cell,
    root: &mut Option<Box<layout_cell>>,
) {
    let parent = (*lc).parent;
    if parent.is_null() {
        assert!(root.as_deref().is_some_and(|owner| std::ptr::eq(owner, lc)));
        drop(root.take());
        return;
    }
    if layout_cell_is_tiled(lc) != 0 {
        let other = layout_cell_get_neighbour(lc);
        if !other.is_null() {
            let change = if (*parent).type_0 == LAYOUT_LEFTRIGHT {
                (*lc).g.sx.wrapping_add(1)
            } else {
                (*lc).g.sy.wrapping_add(1)
            };
            layout_resize_adjust(w, other, (*parent).type_0, change as i32);
        } else {
            layout_remove_tile(w, parent);
        }
    }
    drop(layout_cells_remove(parent, lc).expect("removed cell is owned"));
    if (*parent).cells.children.len() == 1 {
        let child = layout_cells_first(&*parent);
        let mut child_owner = layout_cells_remove(parent, child).expect("remaining child is owned");
        let grandparent = (*parent).parent;
        if grandparent.is_null() {
            if layout_cell_is_tiled(child) != 0 {
                child_owner.g.xoff = 0;
                child_owner.g.yoff = 0;
            }
            *root = Some(child_owner);
        } else {
            drop(layout_cells_replace(grandparent, parent, child_owner));
        }
    }
}

pub unsafe fn layout_init(mut w: *mut window, mut wp: *mut window_pane) {
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    (*w).layout_root = Some(layout_create_cell());
    lc = (*w).layout_root_ptr().map_or(std::ptr::null_mut(), |root| root);
    layout_set_size(
        lc,
        (*w).sx,
        (*w).sy,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_make_leaf(lc, wp);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
}
pub unsafe fn layout_free(w: *mut window) {
    drop((*w).layout_root.take());
}
unsafe fn layout_clamp_floating_panes(mut w: *mut window, mut sx: u_int, mut sy: u_int) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut pad: u_int = 0;
    let mut avail: u_int = 0;
    let mut csx: u_int = 0;
    let mut csy: u_int = 0;
    wp = window_pane_z_first(w);
    while !wp.is_null() {
        lc = (*wp).layout_cell as *mut layout_cell;
        if !(lc.is_null() || !(*lc).flags & LAYOUT_CELL_FLOATING != 0) {
            if window_pane_get_pane_lines(wp) as ::core::ffi::c_uint
                == PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                pad = 0 as u_int;
            } else {
                pad = 1 as u_int;
            }
            csx = (*lc).g.sx;
            avail = if sx > (2 as u_int).wrapping_mul(pad) {
                sx.wrapping_sub((2 as u_int).wrapping_mul(pad))
            } else {
                0 as u_int
            };
            if csx > avail {
                csx = if avail > PANE_MINIMUM as u_int {
                    avail
                } else {
                    PANE_MINIMUM as u_int
                };
            }
            csy = (*lc).g.sy;
            avail = if sy > (2 as u_int).wrapping_mul(pad) {
                sy.wrapping_sub((2 as u_int).wrapping_mul(pad))
            } else {
                0 as u_int
            };
            if csy > avail {
                csy = if avail > PANE_MINIMUM as u_int {
                    avail
                } else {
                    PANE_MINIMUM as u_int
                };
            }
            if csx != (*lc).g.sx || csy != (*lc).g.sy {
                layout_set_size(lc, csx, csy, (*lc).g.xoff, (*lc).g.yoff);
            }
            if ((*lc).g.xoff as u_int)
                .wrapping_add((*lc).g.sx)
                .wrapping_add(pad)
                > sx
            {
                if (*lc).g.sx.wrapping_add((2 as u_int).wrapping_mul(pad)) >= sx {
                    (*lc).g.xoff = pad as ::core::ffi::c_int;
                } else {
                    (*lc).g.xoff =
                        sx.wrapping_sub((*lc).g.sx).wrapping_sub(pad) as ::core::ffi::c_int;
                }
            }
            if ((*lc).g.yoff as u_int)
                .wrapping_add((*lc).g.sy)
                .wrapping_add(pad)
                > sy
            {
                if (*lc).g.sy.wrapping_add((2 as u_int).wrapping_mul(pad)) >= sy {
                    (*lc).g.yoff = pad as ::core::ffi::c_int;
                } else {
                    (*lc).g.yoff =
                        sy.wrapping_sub((*lc).g.sy).wrapping_sub(pad) as ::core::ffi::c_int;
                }
            }
        }
        wp = window_pane_z_next(wp);
    }
}
pub unsafe fn layout_resize(mut w: *mut window, mut sx: u_int, mut sy: u_int) {
    let mut lc: *mut layout_cell = (*w).layout_root_ptr().map_or(std::ptr::null_mut(), |root| root);
    let mut xlimit: ::core::ffi::c_int = 0;
    let mut ylimit: ::core::ffi::c_int = 0;
    let mut xchange: ::core::ffi::c_int = 0;
    let mut ychange: ::core::ffi::c_int = 0;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*lc).flags & LAYOUT_CELL_FLOATING != 0
    {
        layout_clamp_floating_panes(w, sx, sy);
        layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
        return;
    }
    xchange = sx.wrapping_sub((*lc).g.sx) as ::core::ffi::c_int;
    xlimit = layout_resize_check(w, lc, LAYOUT_LEFTRIGHT) as ::core::ffi::c_int;
    if xchange < 0 as ::core::ffi::c_int && xchange < -xlimit {
        xchange = -xlimit;
    }
    if xlimit == 0 as ::core::ffi::c_int {
        if sx <= (*lc).g.sx {
            xchange = 0 as ::core::ffi::c_int;
        } else {
            xchange = sx.wrapping_sub((*lc).g.sx) as ::core::ffi::c_int;
        }
    }
    if xchange != 0 as ::core::ffi::c_int {
        layout_resize_adjust(w, lc, LAYOUT_LEFTRIGHT, xchange);
    }
    ychange = sy.wrapping_sub((*lc).g.sy) as ::core::ffi::c_int;
    ylimit = layout_resize_check(w, lc, LAYOUT_TOPBOTTOM) as ::core::ffi::c_int;
    if ychange < 0 as ::core::ffi::c_int && ychange < -ylimit {
        ychange = -ylimit;
    }
    if ylimit == 0 as ::core::ffi::c_int {
        if sy <= (*lc).g.sy {
            ychange = 0 as ::core::ffi::c_int;
        } else {
            ychange = sy.wrapping_sub((*lc).g.sy) as ::core::ffi::c_int;
        }
    }
    if ychange != 0 as ::core::ffi::c_int {
        layout_resize_adjust(w, lc, LAYOUT_TOPBOTTOM, ychange);
    }
    layout_fix_offsets(w);
    layout_clamp_floating_panes(w, sx, sy);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
}
pub unsafe fn layout_resize_pane_to(
    mut wp: *mut window_pane,
    mut type_0: layout_type,
    mut new_size: u_int,
) {
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut change: ::core::ffi::c_int = 0;
    let mut size: ::core::ffi::c_int = 0;
    lc = (*wp).layout_cell as *mut layout_cell;
    lcparent = (*lc).parent;
    while !lcparent.is_null()
        && (*lcparent).type_0 as ::core::ffi::c_uint != type_0 as ::core::ffi::c_uint
    {
        lc = lcparent;
        lcparent = (*lc).parent;
    }
    if lcparent.is_null() {
        return;
    }
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        size = (*lc).g.sx as ::core::ffi::c_int;
    } else {
        size = (*lc).g.sy as ::core::ffi::c_int;
    }
    if layout_cell_is_last_tiled(lc) != 0 {
        change = (size as u_int).wrapping_sub(new_size) as ::core::ffi::c_int;
    } else {
        change = new_size.wrapping_sub(size as u_int) as ::core::ffi::c_int;
    }
    layout_resize_pane(wp, type_0, change);
}

#[cfg(test)]
mod layout_cell_collection_tests {
    use super::*;

    #[test]
    fn parent_owned_children_preserve_order_and_neighbor_navigation() {
        unsafe {
            let mut root_owner = layout_create_cell();
            let root: *mut layout_cell = &mut *root_owner;
            layout_make_node(root, LAYOUT_LEFTRIGHT);

            let mut first_owner = layout_create_cell();
            let first: *mut layout_cell = &mut *first_owner;
            let mut second_owner = layout_create_cell();
            let second: *mut layout_cell = &mut *second_owner;
            let mut third_owner = layout_create_cell();
            let third: *mut layout_cell = &mut *third_owner;
            let mut fourth_owner = layout_create_cell();
            let fourth: *mut layout_cell = &mut *fourth_owner;
            let mut fifth_owner = layout_create_cell();
            let fifth: *mut layout_cell = &mut *fifth_owner;
            layout_cells_push_back(root, first_owner);
            layout_cells_push_back(root, second_owner);
            layout_cells_push_back(root, third_owner);
            layout_cells_push_back(root, fourth_owner);
            layout_cells_push_back(root, fifth_owner);

            assert_eq!(layout_cells_first(&*root), first);
            assert_eq!(layout_cells_last(&*root), fifth);
            assert_eq!(layout_cell_next(first), second);
            assert_eq!(layout_cell_prev(fifth), fourth);
            assert_eq!((*first).sibling_index, 0);
            assert_eq!((*second).sibling_index, 1);
            assert_eq!((*third).sibling_index, 2);
            assert_eq!(layout_cell_get_neighbour(first), second);
            assert_eq!(layout_cell_get_neighbour(fifth), fourth);

            let mut inserted_owner = layout_create_cell();
            let inserted: *mut layout_cell = &mut *inserted_owner;
            layout_cells_insert_before(root, second, inserted_owner);
            assert_eq!(layout_cell_next(first), inserted);
            assert_eq!(layout_cell_prev(second), inserted);
            assert_eq!((*inserted).sibling_index, 1);
            assert_eq!((*second).sibling_index, 2);
            assert_eq!((*third).sibling_index, 3);
            assert_eq!((*fifth).sibling_index, 5);
            assert_eq!(layout_cell_get_neighbour(inserted), second);

            let inserted_owner = layout_cells_remove(root, inserted).unwrap();
            assert!((*inserted).parent.is_null());
            drop(inserted_owner);
            assert_eq!(layout_cell_next(first), second);
            assert_eq!((*second).sibling_index, 1);
            assert_eq!((*fifth).sibling_index, 4);
            assert_eq!(layout_cell_get_neighbour(fifth), fourth);

            drop(root_owner);
        }
    }
    #[test]
    fn detached_leaf_owners_preserve_panes_during_rebuild() {
        unsafe {
            let mut first_pane = window_pane::empty();
            let mut floating_pane = window_pane::empty();
            let mut root = layout_create_cell();
            layout_make_node(&mut *root, LAYOUT_TOPBOTTOM);
            let mut nested = layout_create_cell();
            layout_make_node(&mut *nested, LAYOUT_LEFTRIGHT);
            let mut first = layout_create_cell();
            layout_make_leaf(&mut *first, &mut first_pane);
            let first_ptr = first_pane.layout_cell;
            let mut floating = layout_create_cell();
            floating.flags = LAYOUT_CELL_FLOATING;
            layout_make_leaf(&mut *floating, &mut floating_pane);
            let floating_ptr = floating_pane.layout_cell;
            layout_cells_push_back(&mut *nested, first);
            layout_cells_push_back(&mut *nested, floating);
            layout_cells_push_back(&mut *root, nested);

            let mut leaves = layout_take_leaves(Some(root));
            assert_eq!(leaves.len(), 2);
            assert_eq!(first_pane.layout_cell, first_ptr);
            assert_eq!(floating_pane.layout_cell, floating_ptr);
            assert!((*first_ptr).parent.is_null());
            assert!((*floating_ptr).parent.is_null());
            let mut replacement = layout_create_cell();
            layout_make_node(&mut *replacement, LAYOUT_LEFTRIGHT);
            layout_cells_push_back(
                &mut *replacement,
                layout_take_leaf(&mut leaves, floating_ptr),
            );
            layout_cells_push_back(&mut *replacement, layout_take_leaf(&mut leaves, first_ptr));
            assert!(leaves.is_empty());
            assert_eq!(layout_cells_first(&replacement), floating_ptr);
            assert_eq!(layout_cell_next(floating_ptr), first_ptr);
            drop(replacement);
            assert!(first_pane.layout_cell.is_null());
            assert!(floating_pane.layout_cell.is_null());
        }
    }

    #[test]
    fn replacement_and_root_collapse_transfer_boxes_without_moving_cells() {
        unsafe {
            let mut pane = window_pane::empty();
            let mut root = layout_create_cell();
            layout_make_node(&mut *root, LAYOUT_TOPBOTTOM);
            let mut discarded = layout_create_cell();
            discarded.flags = LAYOUT_CELL_FLOATING;
            let discarded_ptr: *mut layout_cell = &mut *discarded;
            let mut retained = layout_create_cell();
            layout_make_leaf(&mut *retained, &mut pane);
            let retained_ptr = pane.layout_cell;
            layout_cells_push_back(&mut *root, discarded);
            layout_cells_push_back(&mut *root, retained);
            let mut root = Some(root);
            layout_destroy_cell(std::ptr::null_mut(), discarded_ptr, &mut root);
            assert!(std::ptr::eq(root.as_deref().unwrap(), retained_ptr));
            assert!((*retained_ptr).parent.is_null());
            assert_eq!(pane.layout_cell, retained_ptr);

            let mut parent = layout_create_cell();
            layout_make_node(&mut *parent, LAYOUT_LEFTRIGHT);
            layout_cells_push_back(&mut *parent, root.take().unwrap());
            let replacement = layout_create_cell();
            let detached = layout_cells_replace(&mut *parent, retained_ptr, replacement);
            assert!(std::ptr::eq(&*detached, retained_ptr));
            assert_eq!(pane.layout_cell, retained_ptr);
            drop(parent);
            assert_eq!(pane.layout_cell, retained_ptr);
            drop(detached);
            assert!(pane.layout_cell.is_null());
        }
    }
}

pub unsafe fn layout_resize_floating_pane_to(
    mut wp: *mut window_pane,
    mut type_0: layout_type,
    mut size: u_int,
) -> Result<(), std::ffi::CString> {
    let mut lc: *mut layout_cell = (*wp).layout_cell as *mut layout_cell;
    if !(*lc).flags & LAYOUT_CELL_FLOATING != 0 {
        return Err(c"pane is not floating".to_owned());
    }
    if window_pane_get_pane_lines(wp) as ::core::ffi::c_uint
        != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        && size >= (PANE_MINIMUM + 2 as ::core::ffi::c_int) as u_int
    {
        size = size.wrapping_sub(2 as u_int);
    }
    if size < PANE_MINIMUM as u_int || size > PANE_MAXIMUM as u_int {
        return Err(c"size is too big or too small".to_owned());
    }
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*lc).g.sy == size {
            return Ok(());
        }
        (*lc).g.sy = size;
    } else {
        if (*lc).g.sx == size {
            return Ok(());
        }
        (*lc).g.sx = size;
    }
    redraw_invalidate_scene((*wp).window as *mut window);
    Ok(())
}
pub unsafe fn layout_resize_floating_pane(
    mut wp: *mut window_pane,
    mut type_0: layout_type,
    mut change: ::core::ffi::c_int,
    mut opposite: ::core::ffi::c_int,
) -> Result<(), std::ffi::CString> {
    let mut lc: *mut layout_cell = (*wp).layout_cell as *mut layout_cell;
    let mut size: u_int = 0;
    if !(*lc).flags & LAYOUT_CELL_FLOATING != 0 {
        return Err(c"pane is not floating".to_owned());
    }
    if change == 0 as ::core::ffi::c_int {
        return Ok(());
    }
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        size = (*lc).g.sy.wrapping_add(change as u_int);
        if size < PANE_MINIMUM as u_int || size > PANE_MAXIMUM as u_int {
            return Err(c"change is too big or too small".to_owned());
        }
        (*lc).g.sy = size;
        if opposite != 0 {
            (*lc).g.yoff -= change;
        }
    } else {
        size = (*lc).g.sx.wrapping_add(change as u_int);
        if size < PANE_MINIMUM as u_int || size > PANE_MAXIMUM as u_int {
            return Err(c"change is too big or too small".to_owned());
        }
        (*lc).g.sx = size;
        if opposite != 0 {
            (*lc).g.xoff -= change;
        }
    }
    redraw_invalidate_scene((*wp).window as *mut window);
    Ok(())
}
pub unsafe fn layout_resize_layout(
    mut w: *mut window,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
    mut change: ::core::ffi::c_int,
    mut opposite: ::core::ffi::c_int,
) {
    let mut needed: ::core::ffi::c_int = 0;
    let mut size: ::core::ffi::c_int = 0;
    needed = change;
    while needed != 0 as ::core::ffi::c_int {
        if change > 0 as ::core::ffi::c_int {
            size = layout_resize_pane_grow(w, lc, type_0, needed, opposite);
            needed -= size;
        } else {
            size = layout_resize_pane_shrink(w, lc, type_0, needed);
            needed += size;
        }
        if size == 0 as ::core::ffi::c_int {
            break;
        }
    }
    layout_fix_offsets(w);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
}
pub unsafe fn layout_resize_pane(
    mut wp: *mut window_pane,
    mut type_0: layout_type,
    mut change: ::core::ffi::c_int,
) {
    let mut opposite: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut lc: *mut layout_cell = (*wp).layout_cell as *mut layout_cell;
    let mut lcparent: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    lcparent = (*lc).parent;
    while !lcparent.is_null()
        && (*lcparent).type_0 as ::core::ffi::c_uint != type_0 as ::core::ffi::c_uint
    {
        lc = lcparent;
        lcparent = (*lc).parent;
    }
    if lcparent.is_null() {
        return;
    }
    if layout_cell_is_last_tiled(lc) != 0 {
        lc = layout_cell_get_neighbour_dir(lc, 0 as ::core::ffi::c_int);
        if lc.is_null() {
            return;
        }
    }
    layout_resize_layout((*wp).window as *mut window, lc, type_0, change, opposite);
}
unsafe fn layout_resize_pane_grow(
    mut w: *mut window,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
    mut needed: ::core::ffi::c_int,
    mut opposite: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lcadd: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcremove: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut size: u_int = 0 as u_int;
    lcadd = lc;
    lcremove = layout_cell_get_neighbour_dir(lc, 1 as ::core::ffi::c_int);
    while !lcremove.is_null() {
        size = layout_resize_check(w, lcremove, type_0);
        if size > 0 as u_int {
            break;
        }
        lcremove = layout_cell_get_neighbour_dir(lcremove, 1 as ::core::ffi::c_int);
    }
    if opposite != 0 && lcremove.is_null() {
        lcremove = layout_cell_get_neighbour_dir(lc, 0 as ::core::ffi::c_int);
        while !lcremove.is_null() {
            size = layout_resize_check(w, lcremove, type_0);
            if size > 0 as u_int {
                break;
            }
            lcremove = layout_cell_get_neighbour_dir(lcremove, 0 as ::core::ffi::c_int);
        }
    }
    if lcremove.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if size > needed as u_int {
        size = needed as u_int;
    }
    layout_resize_adjust(w, lcadd, type_0, size as ::core::ffi::c_int);
    layout_resize_adjust(
        w,
        lcremove,
        type_0,
        size.wrapping_neg() as ::core::ffi::c_int,
    );
    return size as ::core::ffi::c_int;
}
unsafe fn layout_resize_pane_shrink(
    mut w: *mut window,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
    mut needed: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lcadd: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcremove: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut size: u_int = 0;
    lcremove = lc;
    loop {
        size = layout_resize_check(w, lcremove, type_0);
        if size != 0 as u_int {
            break;
        }
        lcremove = layout_cell_get_neighbour_dir(lcremove, 0 as ::core::ffi::c_int);
        if lcremove.is_null() {
            break;
        }
    }
    if lcremove.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    lcadd = layout_cell_get_neighbour_dir(lc, 1 as ::core::ffi::c_int);
    if lcadd.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    // The magnitude of i32::MIN fits in the unsigned layout size.
    if size > needed.unsigned_abs() {
        size = needed.unsigned_abs();
    }
    layout_resize_adjust(w, lcadd, type_0, size as ::core::ffi::c_int);
    layout_resize_adjust(
        w,
        lcremove,
        type_0,
        size.wrapping_neg() as ::core::ffi::c_int,
    );
    return size as ::core::ffi::c_int;
}
pub unsafe fn layout_assign_pane(
    mut lc: *mut layout_cell,
    mut wp: *mut window_pane,
    mut do_not_resize: ::core::ffi::c_int,
) {
    layout_make_leaf(lc, wp);
    if do_not_resize != 0 {
        layout_fix_panes((*wp).window as *mut window, wp);
    } else {
        layout_fix_panes(
            (*wp).window as *mut window,
            ::core::ptr::null_mut::<window_pane>(),
        );
    };
}
unsafe fn layout_new_pane_size(
    mut w: *mut window,
    mut previous: u_int,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
    mut size: u_int,
    mut count_left: u_int,
    mut size_left: u_int,
) -> u_int {
    let mut new_size: u_int = 0;
    let mut min: u_int = 0;
    let mut max: u_int = 0;
    let mut available: u_int = 0;
    if count_left == 1 as u_int {
        return size_left;
    }
    available = layout_resize_check(w, lc, type_0);
    min = ((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
        .wrapping_mul(count_left.wrapping_sub(1 as u_int));
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*lc).g.sx.wrapping_sub(available) > min {
            min = (*lc).g.sx.wrapping_sub(available);
        }
        new_size = (*lc).g.sx.wrapping_mul(size).wrapping_div(previous);
    } else {
        if (*lc).g.sy.wrapping_sub(available) > min {
            min = (*lc).g.sy.wrapping_sub(available);
        }
        new_size = (*lc).g.sy.wrapping_mul(size).wrapping_div(previous);
    }
    max = size_left.wrapping_sub(min);
    if new_size > max {
        new_size = max;
    }
    if new_size < PANE_MINIMUM as u_int {
        new_size = PANE_MINIMUM as u_int;
    }
    return new_size;
}
unsafe fn layout_set_size_check(
    mut w: *mut window,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
    mut size: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut new_size: u_int = 0;
    let mut available: u_int = 0;
    let mut previous: u_int = 0;
    let mut count: u_int = 0;
    let mut idx: u_int = 0;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return (size >= PANE_MINIMUM) as ::core::ffi::c_int;
    }
    available = size as u_int;
    count = 0 as u_int;
    lcchild = layout_cells_first(&*lc);
    while !lcchild.is_null() {
        count = count.wrapping_add(1);
        lcchild = layout_cell_next(lcchild);
    }
    if (*lc).type_0 as ::core::ffi::c_uint == type_0 as ::core::ffi::c_uint {
        if available < count.wrapping_mul(2 as u_int).wrapping_sub(1 as u_int) {
            return 0 as ::core::ffi::c_int;
        }
        if type_0 as ::core::ffi::c_uint
            == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            previous = (*lc).g.sx;
        } else {
            previous = (*lc).g.sy;
        }
        idx = 0 as u_int;
        lcchild = layout_cells_first(&*lc);
        while !lcchild.is_null() {
            new_size = layout_new_pane_size(
                w,
                previous,
                lcchild,
                type_0,
                size as u_int,
                count.wrapping_sub(idx),
                available,
            );
            if idx == count.wrapping_sub(1 as u_int) {
                if new_size > available {
                    return 0 as ::core::ffi::c_int;
                }
                available = available.wrapping_sub(new_size);
            } else {
                if new_size.wrapping_add(1 as u_int) > available {
                    return 0 as ::core::ffi::c_int;
                }
                available = available.wrapping_sub(new_size.wrapping_add(1 as u_int));
            }
            if layout_set_size_check(w, lcchild, type_0, new_size as ::core::ffi::c_int) == 0 {
                return 0 as ::core::ffi::c_int;
            }
            idx = idx.wrapping_add(1);
            lcchild = layout_cell_next(lcchild);
        }
    } else {
        lcchild = layout_cells_first(&*lc);
        while !lcchild.is_null() {
            if !((*lcchild).type_0 as ::core::ffi::c_uint
                == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                if layout_set_size_check(w, lcchild, type_0, size) == 0 {
                    return 0 as ::core::ffi::c_int;
                }
            }
            lcchild = layout_cell_next(lcchild);
        }
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn layout_resize_child_cells(mut w: *mut window, mut lc: *mut layout_cell) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut prev: u_int = 0;
    let mut available: u_int = 0;
    let mut count: u_int = 0;
    let mut idx: u_int = 0;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    count = 0 as u_int;
    prev = 0 as u_int;
    lcchild = layout_cells_first(&*lc);
    while !lcchild.is_null() {
        if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
            count = count.wrapping_add(1);
            if (*lc).type_0 as ::core::ffi::c_uint
                == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                prev = prev.wrapping_add((*lcchild).g.sx);
            } else if (*lc).type_0 as ::core::ffi::c_uint
                == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                prev = prev.wrapping_add((*lcchild).g.sy);
            }
        }
        lcchild = layout_cell_next(lcchild);
    }
    prev = prev.wrapping_add(count.wrapping_sub(1 as u_int));
    available = 0 as u_int;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        available = (*lc).g.sx;
    } else if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        available = (*lc).g.sy;
    }
    idx = 0 as u_int;
    lcchild = layout_cells_first(&*lc);
    while !lcchild.is_null() {
        if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
            if (*lc).type_0 as ::core::ffi::c_uint
                == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*lcchild).g.sx = (*lc).g.sx;
                (*lcchild).g.xoff = (*lc).g.xoff;
            } else {
                (*lcchild).g.sx = layout_new_pane_size(
                    w,
                    prev,
                    lcchild,
                    (*lc).type_0,
                    (*lc).g.sx,
                    count.wrapping_sub(idx),
                    available,
                );
                available = available.wrapping_sub((*lcchild).g.sx.wrapping_add(1 as u_int));
            }
            if (*lc).type_0 as ::core::ffi::c_uint
                == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*lcchild).g.sy = (*lc).g.sy;
                (*lcchild).g.yoff = (*lc).g.yoff;
            } else {
                (*lcchild).g.sy = layout_new_pane_size(
                    w,
                    prev,
                    lcchild,
                    (*lc).type_0,
                    (*lc).g.sy,
                    count.wrapping_sub(idx),
                    available,
                );
                available = available.wrapping_sub((*lcchild).g.sy.wrapping_add(1 as u_int));
            }
            layout_resize_child_cells(w, lcchild);
            idx = idx.wrapping_add(1);
        }
        lcchild = layout_cell_next(lcchild);
    }
}
pub unsafe fn layout_replace_with_node(
    w: *mut window,
    lc: *mut layout_cell,
    type_0: layout_type,
) -> *mut layout_cell {
    let mut node = layout_create_cell();
    let parent: *mut layout_cell = &mut *node;
    layout_make_node(parent, type_0);
    layout_set_size(parent, (*lc).g.sx, (*lc).g.sy, (*lc).g.xoff, (*lc).g.yoff);
    let detached = if (*lc).parent.is_null() {
        (*w).layout_root
            .replace(node)
            .expect("replaced root is owned")
    } else {
        layout_cells_replace((*lc).parent, lc, node)
    };
    layout_cells_push_front(parent, detached);
    parent
}

pub unsafe fn layout_split_check_space(
    mut wp: *mut window_pane,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
) -> ::core::ffi::c_int {
    let mut root: *mut layout_cell = (*(*wp).window).layout_root_ptr().map_or(std::ptr::null_mut(), |root| root);
    let mut sb_style: *mut style = &raw mut (*wp).scrollbar_style;
    let mut minimum: u_int = 0;
    let mut sx: u_int = (*lc).g.sx;
    let mut sy: u_int = (*lc).g.sy;
    let mut status: ::core::ffi::c_int = 0;
    if (*lc).flags & LAYOUT_CELL_FLOATING != 0 {
        fatalx(|out| out.write_all(b"floating cells cannot be split"));
    }
    status = window_get_pane_status((*wp).window as *mut window);
    match type_0 as ::core::ffi::c_uint {
        0 => {
            if (*(*wp).window).sb == PANE_SCROLLBARS_ALWAYS {
                minimum = (PANE_MINIMUM * 2 as ::core::ffi::c_int
                    + (*sb_style).width
                    + (*sb_style).pad) as u_int;
            } else {
                minimum =
                    (PANE_MINIMUM * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as u_int;
            }
            if sx < minimum {
                return 0 as ::core::ffi::c_int;
            }
        }
        1 => {
            if layout_add_horizontal_border(root, lc, status) != 0 {
                minimum =
                    (PANE_MINIMUM * 2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as u_int;
            } else {
                minimum =
                    (PANE_MINIMUM * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as u_int;
            }
            if sy < minimum {
                return 0 as ::core::ffi::c_int;
            }
        }
        _ => {
            fatalx(|out| out.write_all(b"bad layout type"));
        }
    }
    return 1 as ::core::ffi::c_int;
}
pub unsafe fn layout_split_sizes(
    mut lc: *mut layout_cell,
    mut size: ::core::ffi::c_int,
    mut before: ::core::ffi::c_int,
    mut type_0: layout_type,
    mut size1: *mut u_int,
    mut size2: *mut u_int,
    mut saved_size: *mut u_int,
) {
    let mut s1: u_int = 0;
    let mut s2: u_int = 0;
    let mut ss: u_int = 0;
    let mut sx: u_int = (*lc).g.sx;
    let mut sy: u_int = (*lc).g.sy;
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        ss = sx;
    } else {
        ss = sy;
    }
    if size < 0 as ::core::ffi::c_int {
        s2 = ss
            .wrapping_add(1 as u_int)
            .wrapping_div(2 as u_int)
            .wrapping_sub(1 as u_int);
    } else if before != 0 {
        s2 = ss.wrapping_sub(size as u_int).wrapping_sub(1 as u_int);
    } else {
        s2 = size as u_int;
    }
    if s2 < PANE_MINIMUM as u_int {
        s2 = PANE_MINIMUM as u_int;
    } else if s2 > ss.wrapping_sub(2 as u_int) {
        s2 = ss.wrapping_sub(2 as u_int);
    }
    s1 = ss.wrapping_sub(1 as u_int).wrapping_sub(s2);
    *size1 = s1;
    *size2 = s2;
    *saved_size = ss;
}
pub unsafe fn layout_split_pane(
    mut wp: *mut window_pane,
    mut type_0: layout_type,
    mut size: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> *mut layout_cell {
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcnew: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lc1: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lc2: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut xoff: u_int = 0;
    let mut yoff: u_int = 0;
    let mut size1: u_int = 0;
    let mut size2: u_int = 0;
    let mut new_size: u_int = 0;
    let mut saved_size: u_int = 0;
    let mut resize_first: u_int = 0 as u_int;
    let mut full_size: ::core::ffi::c_int = flags & SPAWN_FULLSIZE;
    let mut before: ::core::ffi::c_int = flags & SPAWN_BEFORE;
    if full_size != 0 {
        lc = (*(*wp).window).layout_root_ptr().map_or(std::ptr::null_mut(), |root| root);
    } else {
        lc = (*wp).layout_cell as *mut layout_cell;
    }
    sx = (*lc).g.sx;
    sy = (*lc).g.sy;
    xoff = (*lc).g.xoff as u_int;
    yoff = (*lc).g.yoff as u_int;
    if layout_split_check_space(wp, lc, type_0) == 0 {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    layout_split_sizes(
        lc,
        size,
        before,
        type_0,
        &raw mut size1,
        &raw mut size2,
        &raw mut saved_size,
    );
    if flags & SPAWN_BEFORE != 0 {
        new_size = size2;
    } else {
        new_size = size1;
    }
    if full_size != 0
        && layout_set_size_check(
            (*wp).window as *mut window,
            lc,
            type_0,
            new_size as ::core::ffi::c_int,
        ) == 0
    {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    if !(*lc).parent.is_null()
        && (*(*lc).parent).type_0 as ::core::ffi::c_uint == type_0 as ::core::ffi::c_uint
    {
        lcparent = (*lc).parent;
        let mut new_owner = layout_create_cell();
        lcnew = &mut *new_owner;
        if flags & SPAWN_BEFORE != 0 {
            layout_cells_insert_before(lcparent, lc, new_owner);
        } else {
            layout_cells_insert_after(lcparent, lc, new_owner);
        }
    } else if full_size != 0
        && (*lc).parent.is_null()
        && (*lc).type_0 as ::core::ffi::c_uint == type_0 as ::core::ffi::c_uint
    {
        if (*lc).type_0 as ::core::ffi::c_uint
            == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*lc).g.sx = new_size;
            layout_resize_child_cells((*wp).window as *mut window, lc);
            (*lc).g.sx = saved_size;
        } else if (*lc).type_0 as ::core::ffi::c_uint
            == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*lc).g.sy = new_size;
            layout_resize_child_cells((*wp).window as *mut window, lc);
            (*lc).g.sy = saved_size;
        }
        resize_first = 1 as u_int;
        let mut new_owner = layout_create_cell();
        lcnew = &mut *new_owner;
        size = saved_size.wrapping_sub(1 as u_int).wrapping_sub(new_size) as ::core::ffi::c_int;
        if (*lc).type_0 as ::core::ffi::c_uint
            == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            layout_set_size(
                lcnew,
                size as u_int,
                sy,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        } else if (*lc).type_0 as ::core::ffi::c_uint
            == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            layout_set_size(
                lcnew,
                sx,
                size as u_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        }
        if flags & SPAWN_BEFORE != 0 {
            layout_cells_push_front(lc, new_owner);
        } else {
            layout_cells_push_back(lc, new_owner);
        }
    } else {
        lcparent = layout_replace_with_node((*wp).window as *mut window, lc, type_0);
        let mut new_owner = layout_create_cell();
        lcnew = &mut *new_owner;
        if flags & SPAWN_BEFORE != 0 {
            layout_cells_push_front(lcparent, new_owner);
        } else {
            layout_cells_push_back(lcparent, new_owner);
        }
    }
    if flags & SPAWN_BEFORE != 0 {
        lc1 = lcnew;
        lc2 = lc;
    } else {
        lc1 = lc;
        lc2 = lcnew;
    }
    if resize_first == 0
        && type_0 as ::core::ffi::c_uint
            == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        layout_set_size(
            lc1,
            size1,
            sy,
            xoff as ::core::ffi::c_int,
            yoff as ::core::ffi::c_int,
        );
        layout_set_size(
            lc2,
            size2,
            sy,
            xoff.wrapping_add((*lc1).g.sx).wrapping_add(1 as u_int) as ::core::ffi::c_int,
            yoff as ::core::ffi::c_int,
        );
    } else if resize_first == 0
        && type_0 as ::core::ffi::c_uint
            == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        layout_set_size(
            lc1,
            sx,
            size1,
            xoff as ::core::ffi::c_int,
            yoff as ::core::ffi::c_int,
        );
        layout_set_size(
            lc2,
            sx,
            size2,
            xoff as ::core::ffi::c_int,
            yoff.wrapping_add((*lc1).g.sy).wrapping_add(1 as u_int) as ::core::ffi::c_int,
        );
    }
    if full_size != 0 {
        if resize_first == 0 {
            layout_resize_child_cells((*wp).window as *mut window, lc);
        }
        layout_fix_offsets((*wp).window as *mut window);
    } else {
        layout_make_leaf(lc, wp);
    }
    return lcnew;
}
pub unsafe fn layout_floating_pane(
    mut w: *mut window,
    mut wp: *mut window_pane,
    mut lg: *mut layout_geometry,
) -> *mut layout_cell {
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcnew: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if wp.is_null() {
        lc = (*w).layout_root_ptr().map_or(std::ptr::null_mut(), |root| root);
    } else {
        lc = (*wp).layout_cell as *mut layout_cell;
    }
    lcparent = (*lc).parent;
    if lcparent.is_null() {
        lcparent = layout_replace_with_node(w, lc, LAYOUT_TOPBOTTOM);
    }
    let mut new_owner = layout_create_cell();
        lcnew = &mut *new_owner;
    layout_cells_insert_after(lcparent, lc, new_owner);
    (*lcnew).flags |= LAYOUT_CELL_FLOATING;
    layout_set_size(lcnew, (*lg).sx, (*lg).sy, (*lg).xoff, (*lg).yoff);
    return lcnew;
}
pub unsafe fn layout_close_pane(mut wp: *mut window_pane) {
    let mut w: *mut window = (*wp).window as *mut window;
    if (*wp).layout_cell.is_null() {
        return;
    }
    layout_destroy_cell(
        w,
        (*wp).layout_cell as *mut layout_cell,
        &mut (*w).layout_root,
    );
    (*wp).layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if !(*w).layout_root_ptr().map_or(std::ptr::null_mut(), |root| root).is_null() {
        layout_fix_offsets(w);
        layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    }
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
}
pub unsafe fn layout_spread_cell(
    mut w: *mut window,
    mut parent: *mut layout_cell,
) -> ::core::ffi::c_int {
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut root: *mut layout_cell = (*w).layout_root_ptr().map_or(std::ptr::null_mut(), |root| root);
    let mut number: u_int = 0;
    let mut each: u_int = 0;
    let mut size: u_int = 0;
    let mut this: u_int = 0;
    let mut remainder: u_int = 0;
    let mut change: ::core::ffi::c_int = 0;
    let mut changed: ::core::ffi::c_int = 0;
    let mut status: ::core::ffi::c_int = 0;
    number = 0 as u_int;
    lc = layout_cells_first(&*parent);
    while !lc.is_null() {
        if layout_cell_is_tiled(lc) != 0 {
            number = number.wrapping_add(1);
        }
        lc = layout_cell_next(lc);
    }
    if number <= 1 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    status = window_get_pane_status(w);
    if (*parent).type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        size = (*parent).g.sx;
    } else if (*parent).type_0 as ::core::ffi::c_uint
        == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if layout_add_horizontal_border(root, parent, status) != 0 {
            size = (*parent).g.sy.wrapping_sub(1 as u_int);
        } else {
            size = (*parent).g.sy;
        }
    } else {
        return 0 as ::core::ffi::c_int;
    }
    if size < number.wrapping_sub(1 as u_int) {
        return 0 as ::core::ffi::c_int;
    }
    each = size
        .wrapping_sub(number.wrapping_sub(1 as u_int))
        .wrapping_div(number);
    if each == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    remainder = size
        .wrapping_sub(number.wrapping_mul(each.wrapping_add(1 as u_int)))
        .wrapping_add(1 as u_int);
    changed = 0 as ::core::ffi::c_int;
    lc = layout_cells_first(&*parent);
    while !lc.is_null() {
        if !(layout_cell_is_tiled(lc) == 0) {
            change = 0 as ::core::ffi::c_int;
            if (*parent).type_0 as ::core::ffi::c_uint
                == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                change = each.wrapping_sub((*lc).g.sx as ::core::ffi::c_int as u_int)
                    as ::core::ffi::c_int;
                if remainder > 0 as u_int {
                    change += 1;
                    remainder = remainder.wrapping_sub(1);
                }
                layout_resize_adjust(w, lc, LAYOUT_LEFTRIGHT, change);
            } else if (*parent).type_0 as ::core::ffi::c_uint
                == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if layout_add_horizontal_border(root, lc, status) != 0 {
                    this = each.wrapping_add(1 as u_int);
                } else {
                    this = each;
                }
                if remainder > 0 as u_int {
                    this = this.wrapping_add(1);
                    remainder = remainder.wrapping_sub(1);
                }
                change = this.wrapping_sub((*lc).g.sy as ::core::ffi::c_int as u_int)
                    as ::core::ffi::c_int;
                layout_resize_adjust(w, lc, LAYOUT_TOPBOTTOM, change);
            }
            if change != 0 as ::core::ffi::c_int {
                changed = 1 as ::core::ffi::c_int;
            }
        }
        lc = layout_cell_next(lc);
    }
    return changed;
}
pub unsafe fn layout_spread_out(mut wp: *mut window_pane) {
    let mut parent: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut w: *mut window = (*wp).window as *mut window;
    parent = (*(*wp).layout_cell).parent;
    if parent.is_null() {
        return;
    }
    loop {
        if layout_spread_cell(w, parent) != 0 {
            layout_fix_offsets(w);
            layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
            break;
        } else {
            parent = (*parent).parent;
            if parent.is_null() {
                break;
            }
        }
    }
}
pub unsafe fn layout_get_tiled_cell(
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut w: *mut window,
    mut wp: *mut window_pane,
    mut flags: ::core::ffi::c_int,
) -> Result<*mut layout_cell, std::ffi::CString> {
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut type_0: layout_type = LAYOUT_TOPBOTTOM;
    let mut curval: u_int = 0;
    let mut size: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    if window_pane_is_floating(wp) != 0 {
        return Err(c"can't split a floating pane".to_owned());
    }
    if flags & SPAWN_HORIZONTAL != 0 {
        type_0 = LAYOUT_LEFTRIGHT;
    }
    if args_has(args, 'l' as i32 as u_char) != 0 || args_has(args, 'p' as i32 as u_char) != 0 {
        if flags & SPAWN_FULLSIZE != 0 {
            if type_0 as ::core::ffi::c_uint
                == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                curval = (*w).sy;
            } else {
                curval = (*w).sx;
            }
        } else if type_0 as ::core::ffi::c_uint
            == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            curval = (*wp).sy;
        } else {
            curval = (*wp).sx;
        }
    }
    if args_has(args, 'l' as i32 as u_char) != 0 {
        size = match args_percentage_and_expand_result(
            args,
            'l' as i32 as u_char,
            0 as ::core::ffi::c_longlong,
            INT_MAX as ::core::ffi::c_longlong,
            curval as ::core::ffi::c_longlong,
            item,
        ) {
            Ok(value) => value as ::core::ffi::c_int,
            Err(error) => {
                let mut message = b"invalid tiled geometry ".to_vec();
                message.extend_from_slice(error.message().to_bytes());
                return Err(std::ffi::CString::new(message).expect("diagnostic contains no NUL"));
            }
        };
    } else if args_has(args, 'p' as i32 as u_char) != 0 {
        size = match args_strtonum_and_expand_result(
            args,
            'p' as i32 as u_char,
            0 as ::core::ffi::c_longlong,
            100 as ::core::ffi::c_longlong,
            item,
        ) {
            Ok(value) => curval
                .wrapping_mul(value as u_int)
                .wrapping_div(100 as u_int) as ::core::ffi::c_int,
            Err(error) => {
                let mut message = b"invalid tiled geometry ".to_vec();
                message.extend_from_slice(error.message().to_bytes());
                return Err(std::ffi::CString::new(message).expect("diagnostic contains no NUL"));
            }
        };
    }
    if window_active_pane_is_over_zoom(w) != 0 {
        window_push_zoom(w, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    } else {
        window_push_zoom(w, 1 as ::core::ffi::c_int, flags & SPAWN_ZOOM);
    }
    lc = layout_split_pane(wp, type_0, size, flags);
    if lc.is_null() {
        return Err(c"no space for a new pane".to_owned());
    }
    Ok(lc)
}
pub unsafe fn layout_get_floating_cell(
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut lines: pane_lines,
    mut w: *mut window,
    mut wp: *mut window_pane,
    mut flags: ::core::ffi::c_int,
) -> Result<*mut layout_cell, CString> {
    let mut lcnew: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lc: *mut layout_cell = (*wp).layout_cell as *mut layout_cell;
    let mut fg: layout_geometry = layout_geometry {
        sx: 0,
        sy: 0,
        xoff: 0,
        yoff: 0,
    };
    layout_geometry_init(&raw mut fg);
    if flags & SPAWN_SPLIT != 0 {
        layout_split_floating_cell(lc, w, &raw mut fg, lines, flags)?;
    } else {
        layout_floating_args_parse(item, args, lines, w, &raw mut fg)?;
    }
    if flags & SPAWN_FLOATOVERZOOM != 0 {
        window_push_zoom(
            (*wp).window as *mut window,
            0 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        );
    } else if window_active_pane_is_over_zoom(w) != 0 {
        window_push_zoom(
            (*wp).window as *mut window,
            0 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        );
    } else {
        window_push_zoom(
            (*wp).window as *mut window,
            1 as ::core::ffi::c_int,
            flags & SPAWN_ZOOM,
        );
    }
    lcnew = layout_floating_pane(w, wp, &raw mut fg);
    Ok(lcnew)
}
fn layout_position_error(message: &CStr) -> CString {
    let mut bytes = b"position ".to_vec();
    bytes.extend_from_slice(message.to_bytes());
    CString::new(bytes).expect("diagnostic contains no NUL")
}

pub unsafe fn layout_floating_args_parse(
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut lines: pane_lines,
    mut w: *mut window,
    mut lg: *mut layout_geometry,
) -> Result<(), CString> {
    let mut sx: ::core::ffi::c_int = 0;
    let mut sy: ::core::ffi::c_int = 0;
    let mut ox: ::core::ffi::c_int = 0;
    let mut oy: ::core::ffi::c_int = 0;
    sx = (if (*lg).sx == UINT_MAX {
        (*w).sx.wrapping_div(2 as u_int)
    } else {
        (*lg).sx
    }) as ::core::ffi::c_int;
    sy = (if (*lg).sy == UINT_MAX {
        (*w).sy.wrapping_div(4 as u_int)
    } else {
        (*lg).sy
    }) as ::core::ffi::c_int;
    ox = (*lg).xoff;
    oy = (*lg).yoff;
    if args_has(args, 'x' as i32 as u_char) != 0 {
        sx = match args_percentage_and_expand_result(
            args,
            'x' as i32 as u_char,
            0 as ::core::ffi::c_longlong,
            PANE_MAXIMUM as ::core::ffi::c_longlong,
            (*w).sx as ::core::ffi::c_longlong,
            item,
        ) {
            Ok(value) => value as ::core::ffi::c_int,
            Err(error) => {
                return Err(layout_position_error(error.message()));
            }
        };
        if lines as ::core::ffi::c_uint
            != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            sx -= 2 as ::core::ffi::c_int;
        }
    }
    if args_has(args, 'y' as i32 as u_char) != 0 {
        sy = match args_percentage_and_expand_result(
            args,
            'y' as i32 as u_char,
            0 as ::core::ffi::c_longlong,
            PANE_MAXIMUM as ::core::ffi::c_longlong,
            (*w).sy as ::core::ffi::c_longlong,
            item,
        ) {
            Ok(value) => value as ::core::ffi::c_int,
            Err(error) => {
                return Err(layout_position_error(error.message()));
            }
        };
        if lines as ::core::ffi::c_uint
            != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            sy -= 2 as ::core::ffi::c_int;
        }
    }
    if args_has(args, 'X' as i32 as u_char) != 0 {
        ox = match args_percentage_and_expand_result(
            args,
            'X' as i32 as u_char,
            -sx as ::core::ffi::c_longlong,
            (*w).sx as ::core::ffi::c_longlong,
            (*w).sx as ::core::ffi::c_longlong,
            item,
        ) {
            Ok(value) => value as ::core::ffi::c_int,
            Err(error) => {
                return Err(layout_position_error(error.message()));
            }
        };
    }
    if args_has(args, 'Y' as i32 as u_char) != 0 {
        oy = match args_percentage_and_expand_result(
            args,
            'Y' as i32 as u_char,
            -sy as ::core::ffi::c_longlong,
            (*w).sy as ::core::ffi::c_longlong,
            (*w).sy as ::core::ffi::c_longlong,
            item,
        ) {
            Ok(value) => value as ::core::ffi::c_int,
            Err(error) => {
                return Err(layout_position_error(error.message()));
            }
        };
    }
    if ox == INT_MAX {
        if (*w).last_new_pane_x == 0 as u_int {
            ox = 4 as ::core::ffi::c_int;
        } else {
            ox = (*w).last_new_pane_x.wrapping_add(4 as u_int) as ::core::ffi::c_int;
            if (*w).last_new_pane_x > (*w).sx {
                ox = 4 as ::core::ffi::c_int;
            }
        }
        (*w).last_new_pane_x = ox as u_int;
    } else if args_has(args, 'X' as i32 as u_char) != 0 {
        if lines as ::core::ffi::c_uint
            != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ox += 1 as ::core::ffi::c_int;
        }
    }
    if oy == INT_MAX {
        if (*w).last_new_pane_y == 0 as u_int {
            oy = 2 as ::core::ffi::c_int;
        } else {
            oy = (*w).last_new_pane_y.wrapping_add(2 as u_int) as ::core::ffi::c_int;
            if (*w).last_new_pane_y > (*w).sy {
                oy = 2 as ::core::ffi::c_int;
            }
        }
        (*w).last_new_pane_y = oy as u_int;
    } else if args_has(args, 'Y' as i32 as u_char) != 0 {
        if lines as ::core::ffi::c_uint
            != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            oy += 1 as ::core::ffi::c_int;
        }
    }
    if sx < PANE_MINIMUM || sx > PANE_MAXIMUM {
        return Err(c"invalid width".to_owned());
    }
    if sy < PANE_MINIMUM || sy > PANE_MAXIMUM {
        return Err(c"invalid height".to_owned());
    }
    (*lg).sx = sx as u_int;
    (*lg).sy = sy as u_int;
    (*lg).xoff = ox;
    (*lg).yoff = oy;
    Ok(())
}
pub unsafe fn layout_split_floating_cell(
    mut lc: *mut layout_cell,
    mut w: *mut window,
    mut out: *mut layout_geometry,
    mut lines: pane_lines,
    mut flags: ::core::ffi::c_int,
) -> Result<(), CString> {
    let mut old: layout_geometry = layout_geometry {
        sx: 0,
        sy: 0,
        xoff: 0,
        yoff: 0,
    };
    let mut new: layout_geometry = layout_geometry {
        sx: 0,
        sy: 0,
        xoff: 0,
        yoff: 0,
    };
    let mut tborder: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut bborder: ::core::ffi::c_int = (*w).sy.wrapping_sub(1 as u_int) as ::core::ffi::c_int;
    let mut lborder: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
    let mut rborder: ::core::ffi::c_int = (*w).sx.wrapping_sub(3 as u_int) as ::core::ffi::c_int;
    let mut border: ::core::ffi::c_int = if lines as ::core::ffi::c_uint
        != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    let mut size: ::core::ffi::c_int = 0;
    let mut space: ::core::ffi::c_int = 0;
    memcpy(
        &raw mut old as *mut ::core::ffi::c_void,
        &raw mut (*lc).g as *const ::core::ffi::c_void,
        ::core::mem::size_of::<layout_geometry>() as size_t,
    );
    if lborder > old.xoff - border {
        old.xoff = lborder + border;
    }
    if rborder < old.xoff + old.sx as ::core::ffi::c_int + border {
        old.xoff = rborder - old.sx as ::core::ffi::c_int - border;
    }
    if tborder > old.yoff - border {
        old.yoff = tborder + border;
    }
    if bborder < old.yoff + old.sy as ::core::ffi::c_int + border {
        old.yoff = bborder - old.sy as ::core::ffi::c_int - border;
    }
    memcpy(
        &raw mut new as *mut ::core::ffi::c_void,
        &raw mut old as *const ::core::ffi::c_void,
        ::core::mem::size_of::<layout_geometry>() as size_t,
    );
    if flags & SPAWN_HORIZONTAL != 0 {
        if flags & SPAWN_BEFORE != 0 {
            new.xoff = (new.xoff as u_int).wrapping_sub(
                old.sx
                    .wrapping_add((2 as ::core::ffi::c_int * border) as u_int),
            ) as ::core::ffi::c_int as ::core::ffi::c_int;
        } else {
            new.xoff = (new.xoff as u_int).wrapping_add(
                old.sx
                    .wrapping_add((2 as ::core::ffi::c_int * border) as u_int),
            ) as ::core::ffi::c_int as ::core::ffi::c_int;
        }
    } else if flags & SPAWN_BEFORE != 0 {
        new.yoff = (new.yoff as u_int).wrapping_sub(
            old.sy
                .wrapping_add((2 as ::core::ffi::c_int * border) as u_int),
        ) as ::core::ffi::c_int as ::core::ffi::c_int;
    } else {
        new.yoff = (new.yoff as u_int).wrapping_add(
            old.sy
                .wrapping_add((2 as ::core::ffi::c_int * border) as u_int),
        ) as ::core::ffi::c_int as ::core::ffi::c_int;
    }
    if lborder > new.xoff - border {
        space = (old.xoff as u_int)
            .wrapping_add(old.sx)
            .wrapping_sub(lborder as u_int)
            .wrapping_sub((3 as ::core::ffi::c_int * border) as u_int)
            .wrapping_add(1 as u_int) as ::core::ffi::c_int;
        size = space / 2 as ::core::ffi::c_int;
        new.sx = size as u_int;
        old.sx = size as u_int;
        new.xoff = lborder + border;
        old.xoff = (new.xoff as u_int)
            .wrapping_add(new.sx)
            .wrapping_add((2 as ::core::ffi::c_int * border) as u_int)
            as ::core::ffi::c_int;
        if space % 2 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            old.sx = old.sx.wrapping_sub(1 as u_int);
        }
    } else if rborder < new.xoff + new.sx as ::core::ffi::c_int + border {
        space = rborder - old.xoff - 3 as ::core::ffi::c_int * border + 1 as ::core::ffi::c_int;
        size = space / 2 as ::core::ffi::c_int;
        new.sx = size as u_int;
        old.sx = size as u_int;
        new.xoff = (old.xoff as u_int)
            .wrapping_add(old.sx)
            .wrapping_add((2 as ::core::ffi::c_int * border) as u_int)
            as ::core::ffi::c_int;
        if space % 2 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            new.sx = new.sx.wrapping_sub(1 as u_int);
        }
    } else if tborder > new.yoff - border {
        space = old
            .sy
            .wrapping_add(old.yoff as u_int)
            .wrapping_sub(tborder as u_int)
            .wrapping_sub((3 as ::core::ffi::c_int * border) as u_int)
            .wrapping_add(1 as u_int) as ::core::ffi::c_int;
        size = space / 2 as ::core::ffi::c_int;
        new.sy = size as u_int;
        old.sy = size as u_int;
        new.yoff = tborder + border;
        old.yoff = (new.yoff as u_int)
            .wrapping_add(new.sy)
            .wrapping_add((2 as ::core::ffi::c_int * border) as u_int)
            as ::core::ffi::c_int;
        if space % 2 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            old.sy = old.sy.wrapping_sub(1 as u_int);
        }
    } else if bborder < new.yoff + new.sy as ::core::ffi::c_int + border {
        space = bborder - old.yoff - 3 as ::core::ffi::c_int * border + 1 as ::core::ffi::c_int;
        size = space / 2 as ::core::ffi::c_int;
        new.sy = size as u_int;
        old.sy = size as u_int;
        new.yoff = (old.yoff as u_int)
            .wrapping_add(old.sy)
            .wrapping_add((2 as ::core::ffi::c_int * border) as u_int)
            as ::core::ffi::c_int;
        if space % 2 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            new.sy = new.sy.wrapping_sub(1 as u_int);
        }
    }
    if flags & SPAWN_FULLSIZE != 0 {
        if flags & SPAWN_HORIZONTAL != 0 {
            new.yoff = tborder + border;
            new.sy = (bborder - tborder - 2 as ::core::ffi::c_int * border) as u_int;
            if flags & SPAWN_BEFORE != 0 {
                new.xoff = lborder + border;
                new.sx = (old.xoff - new.xoff - 2 as ::core::ffi::c_int * border) as u_int;
            } else {
                new.sx = (rborder - new.xoff - border) as u_int;
            }
        } else {
            new.xoff = lborder + border;
            new.sx = (rborder - lborder - 2 as ::core::ffi::c_int * border) as u_int;
            if flags & SPAWN_BEFORE != 0 {
                new.yoff = tborder + border;
                new.sy = (old.yoff - new.yoff - 2 as ::core::ffi::c_int * border) as u_int;
            } else {
                new.sy = (bborder - new.yoff - border) as u_int;
            }
        }
    }
    if new.sx < PANE_MINIMUM as u_int
        || new.sy < PANE_MINIMUM as u_int
        || old.sx < PANE_MINIMUM as u_int
        || old.sy < PANE_MINIMUM as u_int
    {
        return Err(c"no space for a new pane".to_owned());
    }
    layout_set_size(lc, old.sx, old.sy, old.xoff, old.yoff);
    memcpy(
        out as *mut ::core::ffi::c_void,
        &raw mut new as *const ::core::ffi::c_void,
        ::core::mem::size_of::<layout_geometry>() as size_t,
    );
    Ok(())
}
pub unsafe fn layout_remove_tile(
    mut w: *mut window,
    mut lc: *mut layout_cell,
) -> ::core::ffi::c_int {
    let mut lcneighbour: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut type_0: layout_type = LAYOUT_LEFTRIGHT;
    let mut change: ::core::ffi::c_int = 0;
    if (*lc).flags & LAYOUT_CELL_FLOATING != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    lcneighbour = layout_cell_get_neighbour(lc);
    if lcneighbour.is_null() {
        if !(*lc).parent.is_null() {
            layout_remove_tile(w, (*lc).parent);
        }
    } else {
        lcparent = (*lcneighbour).parent;
        if !lcparent.is_null() {
            type_0 = (*lcparent).type_0;
            if type_0 as ::core::ffi::c_uint
                == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                change = (*lc).g.sy.wrapping_add(1 as u_int) as ::core::ffi::c_int;
            } else {
                change = (*lc).g.sx.wrapping_add(1 as u_int) as ::core::ffi::c_int;
            }
            layout_resize_adjust(w, lcneighbour, type_0, change);
        }
    }
    if !(*lc).parent.is_null() {
        layout_set_size(
            lc,
            0 as u_int,
            0 as u_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn layout_insert_tile(
    mut w: *mut window,
    mut lc: *mut layout_cell,
) -> ::core::ffi::c_int {
    let mut lcneighbour: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lctiled: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut type_0: layout_type = LAYOUT_LEFTRIGHT;
    let mut size1: u_int = 0;
    let mut size2: u_int = 0;
    let mut saved_size: u_int = 0;
    if lc.is_null() {
        fatalx(|out| out.write_all(b"layout cell cannot be null when tiling"));
    }
    if layout_cell_is_tiled(lc) != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    lcparent = (*lc).parent;
    if lcparent.is_null() {
        layout_set_size(
            lc,
            (*w).sx,
            (*w).sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        return 0 as ::core::ffi::c_int;
    }
    type_0 = (*lcparent).type_0;
    lcneighbour = layout_cell_get_neighbour(lc);
    if lcneighbour.is_null() {
        layout_insert_tile(w, lcparent);
        if type_0 as ::core::ffi::c_uint
            == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            size1 = (*lcparent).g.sx;
        } else {
            size1 = (*lcparent).g.sy;
        }
        layout_resize_set_size(w, lc, type_0, size1);
    } else {
        lctiled = layout_cell_get_first_tiled(lcneighbour);
        if layout_split_check_space((*lctiled).wp, lcneighbour, type_0) == 0 {
            return -(1 as ::core::ffi::c_int);
        }
        layout_split_sizes(
            lcneighbour,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_int,
            type_0,
            &raw mut size1,
            &raw mut size2,
            &raw mut saved_size,
        );
        layout_resize_set_size(w, lc, type_0, size1);
        layout_resize_set_size(w, lcneighbour, type_0, size2);
    }
    if (*lcparent).type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        size1 = (*lcparent).g.sy;
        type_0 = LAYOUT_TOPBOTTOM;
    } else {
        size1 = (*lcparent).g.sx;
        type_0 = LAYOUT_LEFTRIGHT;
    }
    layout_resize_set_size(w, lc, type_0, size1);
    return 0 as ::core::ffi::c_int;
}
