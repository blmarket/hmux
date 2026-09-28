use crate::src::session::session_remove_ref;
use crate::src::options::options_owner_ptr;
use crate::src::arguments::{
    args_has, args_make_commands, args_make_commands_prepare, args_strtonum_result,
};
use crate::src::cmd::cmd_mouse_at;
use crate::src::cmd::queue::{
    cmdq_append, cmdq_error, cmdq_get_cmd, cmdq_get_command, cmdq_get_error, cmdq_get_source,
    cmdq_get_target,
};
use crate::src::ffi::libc::memcpy;
use crate::src::format::bytes::write_cstr;
use crate::src::format::bytes::{xformat, xformat_with};
use crate::src::format::{format_create_defaults, format_free, format_single_cstring};
use crate::src::format_draw::format_draw;
use crate::src::grid::grid_default_cell;
use crate::src::layout::layout_add_horizontal_border;
use crate::src::options::{options_get_number, options_get_string};
use crate::src::reactor::{event_add, event_del, event_set};
use crate::src::screen::{screen_free, screen_init, screen_resize};
use crate::src::screen_write::{
    screen_write_cell, screen_write_clearscreen, screen_write_cursormove, screen_write_fast_copy,
    screen_write_preview, screen_write_putc, screen_write_puts, screen_write_start,
    screen_write_stop,
};
use crate::src::server_fn::{
    server_redraw_window, server_redraw_window_borders, server_status_window, server_unzoom_window,
};
use crate::src::session::{session_alive, session_find_by_id};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args;
use crate::src::shared::arguments::args_command_state;
use crate::src::shared::borders::CELL_BORDERS;
use crate::src::shared::client::client;
use crate::src::shared::command::{cmd, cmd_find_state, cmdq_item, cmdq_state};
use crate::src::shared::event::*;
use crate::src::shared::format::format_tree;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::layout::layout_cell;
use crate::src::shared::layout::*;
use crate::src::shared::limits::UINT_MAX;
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::options::options;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::{PANE_REDRAW, PANE_STATUS_BOTTOM, PANE_STATUS_TOP};
use crate::src::shared::rc;
use crate::src::shared::screen::{screen, MODE_CURSOR};
use crate::src::shared::screen_write::screen_write_ctx;
use crate::src::shared::session::session;
use crate::src::shared::style::*;
use crate::src::shared::window::{window, window_mode, window_mode_entry, winlink};
use crate::src::shared::window::{
    WINDOW_MODE_HIDE_PANE_STATUS, WINDOW_MODE_HIDE_SCROLLBARS, WINDOW_MODE_NO_STACK, WINDOW_ZOOMED,
};
use crate::src::style::style_apply;
use crate::src::text::utf8::utf8_set;
use crate::src::window::{window_pane_upgrade, window_pane_weak};
use crate::src::window::{
    window_find_by_id, window_get_pane_status, window_pane_at_index, window_pane_find_by_id,
    window_pane_first, window_pane_index, window_pane_is_visible, window_pane_next,
    window_pane_reset_mode, window_pane_z_last, window_pane_z_previous, window_unzoom, window_zoom,
    winlink_find_by_window,
};
use crate::src::window_clock::window_clock_table;
use std::cell::UnsafeCell;
use std::ffi::CString;
use std::rc::{Rc, Weak};

#[repr(C)]
pub struct window_panes_modedata {
    pub wp: Weak<UnsafeCell<window_pane>>,
    pub session: Weak<UnsafeCell<session>>,
    pub source_session: u_int,
    pub source_window: u_int,
    pub screen: screen,
    preview: Option<Box<screen>>,
    pub timer: event,
    pub state: Option<Box<args_command_state>>,
    pub delay: u_int,
    pub ignore_keys: ::core::ffi::c_int,
    pub zoomed: ::core::ffi::c_int,
    areas: Vec<window_panes_area>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_panes_area {
    pub id: u_int,
    pub x: u_int,
    pub y: u_int,
    pub sx: u_int,
    pub sy: u_int,
}

pub const WINDOW_MODE_FILL_WINDOW: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub static window_panes_mode: window_mode = {
    window_mode {
        name: c"panes-mode",
        default_format: None,
        flags: WINDOW_MODE_HIDE_PANE_STATUS
            | WINDOW_MODE_NO_STACK
            | WINDOW_MODE_FILL_WINDOW
            | WINDOW_MODE_HIDE_SCROLLBARS,
        init: Some(
            window_panes_init
                as unsafe fn(
                    *mut window_mode_entry,
                    *mut cmdq_item,
                    *mut cmd_find_state,
                    *mut args,
                ) -> *mut screen,
        ),
        free: Some(window_panes_free as unsafe fn(*mut window_mode_entry) -> ()),
        resize: Some(window_panes_resize as unsafe fn(*mut window_mode_entry, u_int, u_int) -> ()),
        update: None,
        style_changed: None,
        key: Some(
            window_panes_key
                as unsafe fn(
                    *mut window_mode_entry,
                    &std::rc::Rc<std::cell::UnsafeCell<client>>,
                    *mut winlink,
                    key_code,
                    *mut mouse_event,
                ) -> (),
        ),
        key_table: None,
        command: None,
        formats: None,
        get_screen: None,
    }
};
pub const WINDOW_PANES_BORDER_L: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const WINDOW_PANES_BORDER_R: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const WINDOW_PANES_BORDER_U: ::core::ffi::c_int = 4;
pub const WINDOW_PANES_BORDER_D: ::core::ffi::c_int = 8;
unsafe fn window_panes_session(data: *mut window_panes_modedata) -> Option<Rc<UnsafeCell<session>>> {
    let owner = (*data).session.upgrade()?;
    if session_alive(Some(&*owner.get())) == 0 {
        session_remove_ref(owner, c"window_panes_session");
        return None;
    }
    Some(owner)
}

// The caller holds session_owner through every use of the raw output
// parameters. Session membership still determines logical liveness.
unsafe fn window_panes_get_source(
    mut data: *mut window_panes_modedata,
    mut sp: *mut *mut session,
    mut wlp: *mut *mut winlink,
    mut wp: *mut *mut window,
    session_owner: &mut Option<Rc<UnsafeCell<session>>>,
) -> ::core::ffi::c_int {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let window_owner = window_find_by_id((*data).source_window);
    w = window_owner.as_ref().map_or(
        std::ptr::null_mut(),
        crate::src::shared::window::WindowOwner::as_ptr,
    );
    if w.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    let next_owner = session_find_by_id((*data).source_session);
    s = next_owner.as_ref().map_or(std::ptr::null_mut(), rc::as_ptr);
    if let Some(owner) = std::mem::replace(session_owner, next_owner) {
        session_remove_ref(owner, c"window_panes_get_source");
    }
    if !s.is_null() {
        wl = winlink_find_by_window(&raw mut (*s).windows, w);
    }
    if wl.is_null() {
        if let Some(owner) = std::mem::replace(session_owner, window_panes_session(data)) {
            session_remove_ref(owner, c"window_panes_get_source");
        }
        s = session_owner.as_ref().map_or(std::ptr::null_mut(), rc::as_ptr);
    }
    if !s.is_null() {
        wl = winlink_find_by_window(&raw mut (*s).windows, w);
    }
    if !sp.is_null() {
        *sp = s;
    }
    if !wlp.is_null() {
        *wlp = wl;
    }
    if !wp.is_null() {
        *wp = w;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn window_panes_set_preview(mut data: *mut window_panes_modedata) {
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut wp: *mut window_pane = mode_pane;
    let mut src: *mut screen = &raw mut (*wp).base;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut sx: u_int = (*src).grid().sx;
    let mut sy: u_int = (*src).grid().sy;
    (*data).preview = Some(Box::new(screen::empty()));
    let dst = (*data).preview.as_deref_mut().unwrap() as *mut screen;
    screen_init(&mut *dst, sx, sy, 0 as u_int);
    screen_write_start(&mut ctx, dst);
    screen_write_fast_copy(&mut ctx, &*src, 0 as u_int, (*src).grid().hsize, sx, sy);
    screen_write_stop(&mut ctx);
    (*dst).mode = (*src).mode;
    (*dst).cx = (*src).cx;
    (*dst).cy = (*src).cy;
}
unsafe fn window_panes_free_areas(mut data: *mut window_panes_modedata) {
    (*data).areas = Vec::new();
}
unsafe fn window_panes_add_area(
    mut data: *mut window_panes_modedata,
    mut wp: *mut window_pane,
    mut x: u_int,
    mut y: u_int,
    mut sx: u_int,
    mut sy: u_int,
) {
    (*data).areas.push(window_panes_area {
        id: (*wp).id,
        x,
        y,
        sx,
        sy,
    });
}
unsafe fn window_panes_pane_floating(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut lc: *mut layout_cell = (*wp).saved_layout_cell;
    if lc.is_null() {
        lc = (*wp).layout_cell as *mut layout_cell;
    }
    if lc.is_null() || !(*lc).flags & LAYOUT_CELL_FLOATING != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn window_panes_pane_visible(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    if !(*wp).saved_layout_cell.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    return window_pane_is_visible(&*wp);
}
unsafe fn window_panes_get_geometry(
    mut wp: *mut window_pane,
    mut root: *mut layout_cell,
    mut osx: u_int,
    mut osy: u_int,
    mut dsx: u_int,
    mut dsy: u_int,
    mut xp: *mut u_int,
    mut yp: *mut u_int,
    mut sxp: *mut u_int,
    mut syp: *mut u_int,
) -> ::core::ffi::c_int {
    let mut lc: *mut layout_cell = (*wp).saved_layout_cell;
    let mut status: ::core::ffi::c_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut x2: u_int = 0;
    let mut y2: u_int = 0;
    if lc.is_null() {
        lc = (*wp).layout_cell as *mut layout_cell;
    }
    if lc.is_null()
        || osx == 0 as u_int
        || osy == 0 as u_int
        || dsx == 0 as u_int
        || dsy == 0 as u_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if osx <= dsx && osy <= dsy {
        x = (*lc).g.xoff as u_int;
        y = (*lc).g.yoff as u_int;
        x2 = x.wrapping_add((*lc).g.sx);
        y2 = y.wrapping_add((*lc).g.sy);
    } else {
        x = ((*lc).g.xoff as u_int).wrapping_mul(dsx).wrapping_div(osx);
        y = ((*lc).g.yoff as u_int).wrapping_mul(dsy).wrapping_div(osy);
        x2 = ((*lc).g.xoff as u_int)
            .wrapping_add((*lc).g.sx)
            .wrapping_mul(dsx)
            .wrapping_div(osx);
        y2 = ((*lc).g.yoff as u_int)
            .wrapping_add((*lc).g.sy)
            .wrapping_mul(dsy)
            .wrapping_div(osy);
    }
    if x >= dsx || y >= dsy {
        return 0 as ::core::ffi::c_int;
    }
    if x2 <= x {
        x2 = x.wrapping_add(1 as u_int);
    }
    if y2 <= y {
        y2 = y.wrapping_add(1 as u_int);
    }
    if x2 > dsx {
        x2 = dsx;
    }
    if y2 > dsy {
        y2 = dsy;
    }
    sx = x2.wrapping_sub(x);
    sy = y2.wrapping_sub(y);
    if sx == 0 as u_int || sy == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    status = window_get_pane_status(&*(*wp).window);
    if layout_add_horizontal_border(root, lc, status) != 0 && sy > 1 as u_int {
        if status == PANE_STATUS_TOP {
            y = y.wrapping_add(1);
        }
        sy = sy.wrapping_sub(1);
    }
    *xp = x;
    *yp = y;
    *sxp = sx;
    *syp = sy;
    return 1 as ::core::ffi::c_int;
}
unsafe fn window_panes_get_border_cell(
    mut data: *mut window_panes_modedata,
    mut gc: *mut grid_cell,
) {
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut wp: *mut window_pane = mode_pane;
    let session_owner = window_panes_session(data);
    let mut s = session_owner.as_ref().map_or(std::ptr::null_mut(), rc::as_ptr);
    memcpy(
        gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        s,
        if s.is_null() { std::ptr::null_mut() } else { (*s).curw },
        wp,
    );
    style_apply(
        gc,
        options_owner_ptr(&mut (*(*wp).window).options).map_or(std::ptr::null_mut(), |options| options),
        b"display-panes-border-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    format_free(ft);
    if let Some(owner) = session_owner {
        session_remove_ref(owner, c"window_panes_get_border_cell");
    }
}
unsafe fn window_panes_map_x(mut x: u_int, mut osx: u_int, mut dsx: u_int) -> ::core::ffi::c_int {
    if osx <= dsx {
        return x as ::core::ffi::c_int;
    }
    return x.wrapping_mul(dsx).wrapping_div(osx) as ::core::ffi::c_int;
}
unsafe fn window_panes_map_y(mut y: u_int, mut osy: u_int, mut dsy: u_int) -> ::core::ffi::c_int {
    if osy <= dsy {
        return y as ::core::ffi::c_int;
    }
    return y.wrapping_mul(dsy).wrapping_div(osy) as ::core::ffi::c_int;
}
unsafe fn window_panes_next_tiled_cell(mut lc: *mut layout_cell) -> *mut layout_cell {
    let mut next: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    next = layout_cell_next(lc);
    while !next.is_null() {
        if !(*next).flags & LAYOUT_CELL_FLOATING != 0 {
            return next;
        }
        next = layout_cell_next(next);
    }
    return ::core::ptr::null_mut::<layout_cell>();
}
unsafe fn window_panes_mark_border(
    mut map: *mut u_char,
    mut dsx: u_int,
    mut dsy: u_int,
    mut x: u_int,
    mut y: u_int,
    mut mask: u_char,
) {
    if x < dsx && y < dsy {
        let ref mut fresh0 = *map.offset(y.wrapping_mul(dsx).wrapping_add(x) as isize);
        *fresh0 = (*fresh0 as ::core::ffi::c_int | mask as ::core::ffi::c_int) as u_char;
    }
}
unsafe fn window_panes_mark_vline(
    mut map: *mut u_char,
    mut dsx: u_int,
    mut dsy: u_int,
    mut x: ::core::ffi::c_int,
    mut y: ::core::ffi::c_int,
    mut y2: ::core::ffi::c_int,
) {
    let mut mask: u_char = 0;
    let mut yy: ::core::ffi::c_int = 0;
    if x < 0 as ::core::ffi::c_int || x as u_int >= dsx || y2 <= y {
        return;
    }
    if y < 0 as ::core::ffi::c_int {
        y = 0 as ::core::ffi::c_int;
    }
    if y2 as u_int > dsy {
        y2 = dsy as ::core::ffi::c_int;
    }
    yy = y;
    while yy < y2 {
        mask = 0 as u_char;
        if yy > y {
            mask = (mask as ::core::ffi::c_int | WINDOW_PANES_BORDER_U) as u_char;
        }
        if (yy + 1 as ::core::ffi::c_int) < y2 {
            mask = (mask as ::core::ffi::c_int | WINDOW_PANES_BORDER_D) as u_char;
        }
        if mask as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            mask = (WINDOW_PANES_BORDER_U | WINDOW_PANES_BORDER_D) as u_char;
        }
        window_panes_mark_border(map, dsx, dsy, x as u_int, yy as u_int, mask);
        yy += 1;
    }
}
unsafe fn window_panes_mark_hline(
    mut map: *mut u_char,
    mut dsx: u_int,
    mut dsy: u_int,
    mut x: ::core::ffi::c_int,
    mut x2: ::core::ffi::c_int,
    mut y: ::core::ffi::c_int,
) {
    let mut mask: u_char = 0;
    let mut xx: ::core::ffi::c_int = 0;
    if y < 0 as ::core::ffi::c_int || y as u_int >= dsy || x2 <= x {
        return;
    }
    if x < 0 as ::core::ffi::c_int {
        x = 0 as ::core::ffi::c_int;
    }
    if x2 as u_int > dsx {
        x2 = dsx as ::core::ffi::c_int;
    }
    xx = x;
    while xx < x2 {
        mask = 0 as u_char;
        if xx > x {
            mask = (mask as ::core::ffi::c_int | WINDOW_PANES_BORDER_L) as u_char;
        }
        if (xx + 1 as ::core::ffi::c_int) < x2 {
            mask = (mask as ::core::ffi::c_int | WINDOW_PANES_BORDER_R) as u_char;
        }
        if mask as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            mask = (WINDOW_PANES_BORDER_L | WINDOW_PANES_BORDER_R) as u_char;
        }
        window_panes_mark_border(map, dsx, dsy, xx as u_int, y as u_int, mask);
        xx += 1;
    }
}
unsafe fn window_panes_mark_borders_cell(
    mut map: *mut u_char,
    mut lc: *mut layout_cell,
    mut osx: u_int,
    mut osy: u_int,
    mut dsx: u_int,
    mut dsy: u_int,
) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcnext: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut x: ::core::ffi::c_int = 0;
    let mut y: ::core::ffi::c_int = 0;
    let mut x2: ::core::ffi::c_int = 0;
    let mut y2: ::core::ffi::c_int = 0;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    lcchild = layout_cells_first(&*lc);
    while !lcchild.is_null() {
        window_panes_mark_borders_cell(map, lcchild, osx, osy, dsx, dsy);
        if !((*lcchild).flags & LAYOUT_CELL_FLOATING != 0) {
            lcnext = window_panes_next_tiled_cell(lcchild);
            if !lcnext.is_null() {
                if (*lc).type_0 as ::core::ffi::c_uint
                    == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    x = window_panes_map_x(
                        ((*lcchild).g.xoff as u_int).wrapping_add((*lcchild).g.sx),
                        osx,
                        dsx,
                    );
                    y = window_panes_map_y((*lc).g.yoff as u_int, osy, dsy);
                    y2 = window_panes_map_y(
                        ((*lc).g.yoff as u_int).wrapping_add((*lc).g.sy),
                        osy,
                        dsy,
                    );
                    window_panes_mark_vline(map, dsx, dsy, x, y, y2);
                } else {
                    x = window_panes_map_x((*lc).g.xoff as u_int, osx, dsx);
                    x2 = window_panes_map_x(
                        ((*lc).g.xoff as u_int).wrapping_add((*lc).g.sx),
                        osx,
                        dsx,
                    );
                    y = window_panes_map_y(
                        ((*lcchild).g.yoff as u_int).wrapping_add((*lcchild).g.sy),
                        osy,
                        dsy,
                    );
                    window_panes_mark_hline(map, dsx, dsy, x, x2, y);
                }
            }
        }
        lcchild = layout_cell_next(lcchild);
    }
}
unsafe fn window_panes_mark_pane_status_borders(
    mut map: *mut u_char,
    mut w: *mut window,
    mut root: *mut layout_cell,
    mut osx: u_int,
    mut osy: u_int,
    mut dsx: u_int,
    mut dsy: u_int,
) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut status: ::core::ffi::c_int = 0;
    let mut x: ::core::ffi::c_int = 0;
    let mut y: ::core::ffi::c_int = 0;
    let mut x2: ::core::ffi::c_int = 0;
    let mut y2: ::core::ffi::c_int = 0;
    status = window_get_pane_status(&*w);
    if status != PANE_STATUS_TOP && status != PANE_STATUS_BOTTOM {
        return;
    }
    wp = window_pane_first(w).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !wp.is_null() {
        if !(window_panes_pane_visible(wp) == 0) {
            lc = (*wp).saved_layout_cell;
            if lc.is_null() {
                lc = (*wp).layout_cell as *mut layout_cell;
            }
            if !(lc.is_null() || layout_add_horizontal_border(root, lc, status) == 0) {
                x = window_panes_map_x((*lc).g.xoff as u_int, osx, dsx);
                x2 = window_panes_map_x(((*lc).g.xoff as u_int).wrapping_add((*lc).g.sx), osx, dsx);
                if status == PANE_STATUS_TOP {
                    y = window_panes_map_y((*lc).g.yoff as u_int, osy, dsy);
                } else {
                    y2 = window_panes_map_y(
                        ((*lc).g.yoff as u_int).wrapping_add((*lc).g.sy),
                        osy,
                        dsy,
                    );
                    y = y2 - 1 as ::core::ffi::c_int;
                }
                window_panes_mark_hline(map, dsx, dsy, x, x2, y);
            }
        }
        wp = window_pane_next(wp).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
unsafe fn window_panes_get_floating_borders(
    mut wp: *mut window_pane,
    mut osx: u_int,
    mut osy: u_int,
    mut dsx: u_int,
    mut dsy: u_int,
    mut xp: *mut ::core::ffi::c_int,
    mut yp: *mut ::core::ffi::c_int,
    mut x2p: *mut ::core::ffi::c_int,
    mut y2p: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    lc = (*wp).saved_layout_cell;
    if lc.is_null() {
        lc = (*wp).layout_cell as *mut layout_cell;
    }
    if lc.is_null() || !(*lc).flags & LAYOUT_CELL_FLOATING != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*lc).g.xoff == 0 as ::core::ffi::c_int {
        *xp = -(1 as ::core::ffi::c_int);
    } else {
        *xp = window_panes_map_x(((*lc).g.xoff - 1 as ::core::ffi::c_int) as u_int, osx, dsx);
    }
    if (*lc).g.yoff == 0 as ::core::ffi::c_int {
        *yp = -(1 as ::core::ffi::c_int);
    } else {
        *yp = window_panes_map_y(((*lc).g.yoff - 1 as ::core::ffi::c_int) as u_int, osy, dsy);
    }
    *x2p = window_panes_map_x(((*lc).g.xoff as u_int).wrapping_add((*lc).g.sx), osx, dsx);
    *y2p = window_panes_map_y(((*lc).g.yoff as u_int).wrapping_add((*lc).g.sy), osy, dsy);
    return 1 as ::core::ffi::c_int;
}
unsafe fn window_panes_clip_floating_pane(
    mut wp: *mut window_pane,
    mut osx: u_int,
    mut osy: u_int,
    mut dsx: u_int,
    mut dsy: u_int,
    mut xp: *mut u_int,
    mut yp: *mut u_int,
    mut sxp: *mut u_int,
    mut syp: *mut u_int,
) -> ::core::ffi::c_int {
    let mut x: ::core::ffi::c_int = 0;
    let mut y: ::core::ffi::c_int = 0;
    let mut x2: ::core::ffi::c_int = 0;
    let mut y2: ::core::ffi::c_int = 0;
    let mut bx: ::core::ffi::c_int = 0;
    let mut by: ::core::ffi::c_int = 0;
    let mut bx2: ::core::ffi::c_int = 0;
    let mut by2: ::core::ffi::c_int = 0;
    if window_panes_get_floating_borders(
        wp,
        osx,
        osy,
        dsx,
        dsy,
        &raw mut x,
        &raw mut y,
        &raw mut x2,
        &raw mut y2,
    ) == 0
    {
        return 1 as ::core::ffi::c_int;
    }
    bx = *xp as ::core::ffi::c_int;
    by = *yp as ::core::ffi::c_int;
    bx2 = (bx as u_int).wrapping_add(*sxp).wrapping_sub(1 as u_int) as ::core::ffi::c_int;
    by2 = (by as u_int).wrapping_add(*syp).wrapping_sub(1 as u_int) as ::core::ffi::c_int;
    if x >= 0 as ::core::ffi::c_int && bx <= x {
        bx = x + 1 as ::core::ffi::c_int;
    }
    if y >= 0 as ::core::ffi::c_int && by <= y {
        by = y + 1 as ::core::ffi::c_int;
    }
    if (x2 as u_int) < dsx && bx2 >= x2 {
        bx2 = x2 - 1 as ::core::ffi::c_int;
    }
    if (y2 as u_int) < dsy && by2 >= y2 {
        by2 = y2 - 1 as ::core::ffi::c_int;
    }
    if bx2 < bx || by2 < by {
        return 0 as ::core::ffi::c_int;
    }
    *xp = bx as u_int;
    *yp = by as u_int;
    *sxp = (bx2 - bx + 1 as ::core::ffi::c_int) as u_int;
    *syp = (by2 - by + 1 as ::core::ffi::c_int) as u_int;
    return 1 as ::core::ffi::c_int;
}
unsafe fn window_panes_border_cell_type(mut mask: u_char) -> ::core::ffi::c_int {
    match mask as ::core::ffi::c_int {
        15 => return 11 as ::core::ffi::c_int,
        7 => return 8 as ::core::ffi::c_int,
        11 => return 7 as ::core::ffi::c_int,
        3 | WINDOW_PANES_BORDER_L | WINDOW_PANES_BORDER_R => {
            return 2 as ::core::ffi::c_int;
        }
        13 => return 10 as ::core::ffi::c_int,
        5 => return 6 as ::core::ffi::c_int,
        9 => return 4 as ::core::ffi::c_int,
        14 => return 9 as ::core::ffi::c_int,
        6 => return 5 as ::core::ffi::c_int,
        10 => return 3 as ::core::ffi::c_int,
        12 | WINDOW_PANES_BORDER_U | WINDOW_PANES_BORDER_D => {
            return 1 as ::core::ffi::c_int;
        }
        _ => {}
    }
    return 12 as ::core::ffi::c_int;
}
unsafe fn window_panes_border_has_horizontal(mut mask: u_char) -> ::core::ffi::c_int {
    return (mask as ::core::ffi::c_int & (WINDOW_PANES_BORDER_L | WINDOW_PANES_BORDER_R)
        != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe fn window_panes_border_has_vertical(mut mask: u_char) -> ::core::ffi::c_int {
    return (mask as ::core::ffi::c_int & (WINDOW_PANES_BORDER_U | WINDOW_PANES_BORDER_D)
        != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe fn window_panes_mark_border_joins_cell(
    mut map: *mut u_char,
    mut lc: *mut layout_cell,
    mut osx: u_int,
    mut osy: u_int,
    mut dsx: u_int,
    mut dsy: u_int,
) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcnext: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut x: ::core::ffi::c_int = 0;
    let mut y: ::core::ffi::c_int = 0;
    let mut x2: ::core::ffi::c_int = 0;
    let mut y2: ::core::ffi::c_int = 0;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    lcchild = layout_cells_first(&*lc);
    while !lcchild.is_null() {
        window_panes_mark_border_joins_cell(map, lcchild, osx, osy, dsx, dsy);
        if !((*lcchild).flags & LAYOUT_CELL_FLOATING != 0) {
            lcnext = window_panes_next_tiled_cell(lcchild);
            if !lcnext.is_null() {
                if (*lc).type_0 as ::core::ffi::c_uint
                    == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    x = window_panes_map_x(
                        ((*lcchild).g.xoff as u_int).wrapping_add((*lcchild).g.sx),
                        osx,
                        dsx,
                    );
                    y = window_panes_map_y((*lc).g.yoff as u_int, osy, dsy);
                    y2 = window_panes_map_y(
                        ((*lc).g.yoff as u_int).wrapping_add((*lc).g.sy),
                        osy,
                        dsy,
                    );
                    if !(x < 0 as ::core::ffi::c_int || x as u_int >= dsx) {
                        if y > 0 as ::core::ffi::c_int
                            && window_panes_border_has_horizontal(
                                *map.offset(
                                    ((y - 1 as ::core::ffi::c_int) as u_int)
                                        .wrapping_mul(dsx)
                                        .wrapping_add(x as u_int)
                                        as isize,
                                ),
                            ) != 0
                        {
                            window_panes_mark_border(
                                map,
                                dsx,
                                dsy,
                                x as u_int,
                                (y - 1 as ::core::ffi::c_int) as u_int,
                                WINDOW_PANES_BORDER_D as u_char,
                            );
                            window_panes_mark_border(
                                map,
                                dsx,
                                dsy,
                                x as u_int,
                                y as u_int,
                                WINDOW_PANES_BORDER_U as u_char,
                            );
                        }
                        if (y2 as u_int) < dsy
                            && window_panes_border_has_horizontal(
                                *map.offset(
                                    (y2 as u_int).wrapping_mul(dsx).wrapping_add(x as u_int)
                                        as isize,
                                ),
                            ) != 0
                        {
                            window_panes_mark_border(
                                map,
                                dsx,
                                dsy,
                                x as u_int,
                                y2 as u_int,
                                WINDOW_PANES_BORDER_U as u_char,
                            );
                            window_panes_mark_border(
                                map,
                                dsx,
                                dsy,
                                x as u_int,
                                (y2 - 1 as ::core::ffi::c_int) as u_int,
                                WINDOW_PANES_BORDER_D as u_char,
                            );
                        }
                    }
                } else {
                    x = window_panes_map_x((*lc).g.xoff as u_int, osx, dsx);
                    x2 = window_panes_map_x(
                        ((*lc).g.xoff as u_int).wrapping_add((*lc).g.sx),
                        osx,
                        dsx,
                    );
                    y = window_panes_map_y(
                        ((*lcchild).g.yoff as u_int).wrapping_add((*lcchild).g.sy),
                        osy,
                        dsy,
                    );
                    if !(y < 0 as ::core::ffi::c_int || y as u_int >= dsy) {
                        if x > 0 as ::core::ffi::c_int
                            && window_panes_border_has_vertical(
                                *map.offset(
                                    (y as u_int)
                                        .wrapping_mul(dsx)
                                        .wrapping_add(x as u_int)
                                        .wrapping_sub(1 as u_int)
                                        as isize,
                                ),
                            ) != 0
                        {
                            window_panes_mark_border(
                                map,
                                dsx,
                                dsy,
                                (x - 1 as ::core::ffi::c_int) as u_int,
                                y as u_int,
                                WINDOW_PANES_BORDER_R as u_char,
                            );
                            window_panes_mark_border(
                                map,
                                dsx,
                                dsy,
                                x as u_int,
                                y as u_int,
                                WINDOW_PANES_BORDER_L as u_char,
                            );
                        }
                        if (x2 as u_int) < dsx
                            && window_panes_border_has_vertical(
                                *map.offset(
                                    (y as u_int).wrapping_mul(dsx).wrapping_add(x2 as u_int)
                                        as isize,
                                ),
                            ) != 0
                        {
                            window_panes_mark_border(
                                map,
                                dsx,
                                dsy,
                                x2 as u_int,
                                y as u_int,
                                WINDOW_PANES_BORDER_L as u_char,
                            );
                            window_panes_mark_border(
                                map,
                                dsx,
                                dsy,
                                (x2 - 1 as ::core::ffi::c_int) as u_int,
                                y as u_int,
                                WINDOW_PANES_BORDER_R as u_char,
                            );
                        }
                    }
                }
            }
        }
        lcchild = layout_cell_next(lcchild);
    }
}
unsafe fn window_panes_draw_borders(
    mut ctx: *mut screen_write_ctx,
    mut w: *mut window,
    mut lc: *mut layout_cell,
    mut gc: *const grid_cell,
    mut osx: u_int,
    mut osy: u_int,
    mut dsx: u_int,
    mut dsy: u_int,
) {
    let mut border_gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut xx: u_int = 0;
    let mut yy: u_int = 0;
    let mut cell_type: ::core::ffi::c_int = 0;
    if dsx == 0 as u_int || dsy == 0 as u_int {
        return;
    }
    let map_size = (dsx as usize).checked_mul(dsy as usize).unwrap();
    let mut map = vec![0; map_size];
    window_panes_mark_borders_cell(map.as_mut_ptr(), lc, osx, osy, dsx, dsy);
    window_panes_mark_pane_status_borders(map.as_mut_ptr(), w, lc, osx, osy, dsx, dsy);
    window_panes_mark_border_joins_cell(map.as_mut_ptr(), lc, osx, osy, dsx, dsy);
    yy = 0 as u_int;
    while yy < dsy {
        xx = 0 as u_int;
        while xx < dsx {
            let border = map[yy.wrapping_mul(dsx).wrapping_add(xx) as usize];
            if border != 0 {
                cell_type = window_panes_border_cell_type(border);
                memcpy(
                    &raw mut border_gc as *mut ::core::ffi::c_void,
                    gc as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<grid_cell>() as size_t,
                );
                border_gc.attr =
                    (border_gc.attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
                utf8_set(
                    &mut border_gc.data,
                    CELL_BORDERS[cell_type as usize] as u_char,
                );
                screen_write_cursormove(
                    &mut *ctx,
                    xx as ::core::ffi::c_int,
                    yy as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                screen_write_cell(&mut *ctx, &border_gc);
            }
            xx = xx.wrapping_add(1);
        }
        yy = yy.wrapping_add(1);
    }
}
unsafe fn window_panes_draw_floating_border(
    mut ctx: *mut screen_write_ctx,
    mut wp: *mut window_pane,
    mut gc: *const grid_cell,
    mut osx: u_int,
    mut osy: u_int,
    mut dsx: u_int,
    mut dsy: u_int,
) {
    let mut border_gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut xx: u_int = 0;
    let mut yy: u_int = 0;
    let mut x: ::core::ffi::c_int = 0;
    let mut y: ::core::ffi::c_int = 0;
    let mut x2: ::core::ffi::c_int = 0;
    let mut y2: ::core::ffi::c_int = 0;
    let mut cell_type: ::core::ffi::c_int = 0;
    if dsx == 0 as u_int || dsy == 0 as u_int {
        return;
    }
    if window_panes_get_floating_borders(
        wp,
        osx,
        osy,
        dsx,
        dsy,
        &raw mut x,
        &raw mut y,
        &raw mut x2,
        &raw mut y2,
    ) == 0
    {
        return;
    }
    let map_size = (dsx as usize).checked_mul(dsy as usize).unwrap();
    let mut map = vec![0; map_size];
    window_panes_mark_hline(
        map.as_mut_ptr(),
        dsx,
        dsy,
        x,
        x2 + 1 as ::core::ffi::c_int,
        y,
    );
    window_panes_mark_hline(
        map.as_mut_ptr(),
        dsx,
        dsy,
        x,
        x2 + 1 as ::core::ffi::c_int,
        y2,
    );
    window_panes_mark_vline(
        map.as_mut_ptr(),
        dsx,
        dsy,
        x,
        y,
        y2 + 1 as ::core::ffi::c_int,
    );
    window_panes_mark_vline(
        map.as_mut_ptr(),
        dsx,
        dsy,
        x2,
        y,
        y2 + 1 as ::core::ffi::c_int,
    );
    yy = 0 as u_int;
    while yy < dsy {
        xx = 0 as u_int;
        while xx < dsx {
            let border = map[yy.wrapping_mul(dsx).wrapping_add(xx) as usize];
            if border != 0 {
                cell_type = window_panes_border_cell_type(border);
                memcpy(
                    &raw mut border_gc as *mut ::core::ffi::c_void,
                    gc as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<grid_cell>() as size_t,
                );
                border_gc.attr =
                    (border_gc.attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
                utf8_set(
                    &mut border_gc.data,
                    CELL_BORDERS[cell_type as usize] as u_char,
                );
                screen_write_cursormove(
                    &mut *ctx,
                    xx as ::core::ffi::c_int,
                    yy as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                screen_write_cell(&mut *ctx, &border_gc);
            }
            xx = xx.wrapping_add(1);
        }
        yy = yy.wrapping_add(1);
    }
}
unsafe fn window_panes_clear_floating_area(
    mut ctx: *mut screen_write_ctx,
    mut wp: *mut window_pane,
    mut osx: u_int,
    mut osy: u_int,
    mut dsx: u_int,
    mut dsy: u_int,
) {
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut x: ::core::ffi::c_int = 0;
    let mut y: ::core::ffi::c_int = 0;
    let mut x2: ::core::ffi::c_int = 0;
    let mut y2: ::core::ffi::c_int = 0;
    let mut xx: ::core::ffi::c_int = 0;
    let mut yy: ::core::ffi::c_int = 0;
    if window_panes_get_floating_borders(
        wp,
        osx,
        osy,
        dsx,
        dsy,
        &raw mut x,
        &raw mut y,
        &raw mut x2,
        &raw mut y2,
    ) == 0
    {
        return;
    }
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    if x < 0 as ::core::ffi::c_int {
        x = 0 as ::core::ffi::c_int;
    }
    if y < 0 as ::core::ffi::c_int {
        y = 0 as ::core::ffi::c_int;
    }
    if x2 as u_int >= dsx {
        x2 = dsx.wrapping_sub(1 as u_int) as ::core::ffi::c_int;
    }
    if y2 as u_int >= dsy {
        y2 = dsy.wrapping_sub(1 as u_int) as ::core::ffi::c_int;
    }
    if x2 < x || y2 < y {
        return;
    }
    yy = y;
    while yy <= y2 {
        screen_write_cursormove(&mut *ctx, x, yy, 0 as ::core::ffi::c_int);
        xx = x;
        while xx <= x2 {
            screen_write_putc(&mut *ctx, &gc, ' ' as i32 as u_char);
            xx += 1;
        }
        yy += 1;
    }
}
unsafe fn window_panes_draw_format(
    mut data: *mut window_panes_modedata,
    mut ctx: *mut screen_write_ctx,
    mut wp: *mut window_pane,
    mut x: u_int,
    mut y: u_int,
    mut sx: u_int,
    mut gc: *const grid_cell,
) {
    let mut source_session_owner = None;
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let session_owner = window_panes_session(data);
    let mut s = session_owner.as_ref().map_or(std::ptr::null_mut(), rc::as_ptr);
    let mut wl: *mut winlink = if s.is_null() { std::ptr::null_mut() } else { (*s).curw };
    let mut format: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if sx == 0 as u_int {
        if let Some(owner) = session_owner {
            session_remove_ref(owner, c"window_panes_draw_format");
        }
        return;
    }
    format = options_get_string(
        options_owner_ptr(&mut (*(*mode_pane).window).options).map_or(std::ptr::null_mut(), |options| options),
        b"display-panes-format\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if *format as ::core::ffi::c_int == '\0' as i32 {
        if let Some(owner) = session_owner {
            session_remove_ref(owner, c"window_panes_draw_format");
        }
        return;
    }
    window_panes_get_source(
        data,
        &raw mut s,
        &raw mut wl,
        ::core::ptr::null_mut::<*mut window>(),
        &mut source_session_owner,
    );
    if s.is_null() {
        if let Some(owner) = session_owner {
            session_remove_ref(owner, c"window_panes_draw_format");
        }
        if let Some(owner) = source_session_owner {
            session_remove_ref(owner, c"window_panes_draw_format");
        }
        return;
    }
    let expanded = format_single_cstring(
        ::core::ptr::null_mut::<cmdq_item>(),
        format,
        ::core::ptr::null_mut::<client>(),
        s,
        wl,
        wp,
    );
    if !expanded.is_empty() {
        screen_write_cursormove(
            &mut *ctx,
            x as ::core::ffi::c_int,
            y as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        format_draw(
            ctx,
            gc,
            sx,
            expanded.as_ptr(),
            ::core::ptr::null_mut::<style_ranges>(),
            0 as ::core::ffi::c_int,
        );
    }
    if let Some(owner) = session_owner {
        session_remove_ref(owner, c"window_panes_draw_format");
    }
    if let Some(owner) = source_session_owner {
        session_remove_ref(owner, c"window_panes_draw_format");
    }
}
unsafe fn window_panes_draw_number(
    mut data: *mut window_panes_modedata,
    mut ctx: *mut screen_write_ctx,
    mut wp: *mut window_pane,
    mut pane: u_int,
    mut x: u_int,
    mut y: u_int,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut source_session_owner = None;
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let session_owner = window_panes_session(data);
    let mut s = session_owner.as_ref().map_or(std::ptr::null_mut(), rc::as_ptr);
    let mut w: *mut window = (*wp).window as *mut window;
    let mut wl: *mut winlink = if s.is_null() { std::ptr::null_mut() } else { (*s).curw };
    let mut oo: *mut options = options_owner_ptr(&mut (*(*mode_pane).window).options).map_or(std::ptr::null_mut(), |options| options);
    let mut fgc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut bgc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut buf: [::core::ffi::c_char; 16] = [0; 16];
    let mut lbuf: [::core::ffi::c_char; 16] = [
        0 as ::core::ffi::c_int as ::core::ffi::c_char,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ];
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut llen: size_t = 0 as size_t;
    let mut width: size_t = 0;
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut idx: u_int = 0;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut format: u_int = 0;
    len = xformat(&mut buf, format_args!("{}", pane as u32)) as size_t;
    if pane > 9 as u_int && pane < 35 as u_int {
        llen = xformat_with(&mut lbuf, |out| {
            out.write_all(&[
                (('a' as i32 as u_int).wrapping_add(pane.wrapping_sub(10 as u_int))) as u8,
            ])
        }) as size_t;
    }
    if (sx as size_t) < len {
        if let Some(owner) = session_owner {
            session_remove_ref(owner, c"window_panes_draw_number");
        }
        return;
    }
    window_panes_get_source(
        data,
        &raw mut s,
        &raw mut wl,
        ::core::ptr::null_mut::<*mut window>(),
        &mut source_session_owner,
    );
    if !s.is_null() {
        if wl.is_null() {
            wl = (*s).curw;
        }
    }
    if (*w).active == wp {
        name = b"display-panes-active-colour\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        name = b"display-panes-colour\0" as *const u8 as *const ::core::ffi::c_char;
    }
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        s,
        wl,
        wp,
    );
    style_apply(&raw mut fgc, oo, name, ft);
    format_free(ft);
    memcpy(
        &raw mut bgc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    bgc.bg = fgc.fg;
    format = 0 as u_int;
    if *options_get_string(
        oo,
        b"display-panes-format\0" as *const u8 as *const ::core::ffi::c_char,
    )
    .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        != '\0' as i32
    {
        format = 1 as u_int;
    }
    width = len.wrapping_mul(6 as size_t).wrapping_sub(1 as size_t);
    if (sx as size_t) < width
        || sy
            < (if format != 0 {
                7 as ::core::ffi::c_int
            } else {
                5 as ::core::ffi::c_int
            }) as u_int
    {
        width = len;
        if llen != 0 as size_t && sx as size_t >= len.wrapping_add(llen).wrapping_add(1 as size_t) {
            width = width.wrapping_add(llen.wrapping_add(1 as size_t));
        }
        cx = (x as size_t)
            .wrapping_add((sx as size_t).wrapping_sub(width).wrapping_div(2 as size_t))
            as u_int;
        cy = y.wrapping_add(sy.wrapping_div(2 as u_int));
        screen_write_cursormove(
            &mut *ctx,
            cx as ::core::ffi::c_int,
            cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_puts(&mut *ctx, &fgc, |out| {
            write_cstr(out, &raw mut buf as *mut ::core::ffi::c_char)
        });
        if width > len {
            screen_write_puts(&mut *ctx, &fgc, |out| {
                out.write_all(b" ")?;
                write_cstr(out, &raw mut lbuf as *mut ::core::ffi::c_char)
            });
        }
        if format != 0 && sy > 1 as u_int {
            window_panes_draw_format(data, ctx, wp, x, y, sx, &raw mut fgc);
        }
        if let Some(owner) = session_owner {
            session_remove_ref(owner, c"window_panes_draw_number");
        }
        if let Some(owner) = source_session_owner {
            session_remove_ref(owner, c"window_panes_draw_number");
        }
        return;
    }
    px = (sx as size_t).wrapping_sub(width).wrapping_div(2 as size_t) as u_int;
    py = sy.wrapping_sub(5 as u_int).wrapping_div(2 as u_int);
    ptr = &raw mut buf as *mut ::core::ffi::c_char;
    while *ptr as ::core::ffi::c_int != '\0' as i32 {
        if !((*ptr as ::core::ffi::c_int) < '0' as i32 || *ptr as ::core::ffi::c_int > '9' as i32) {
            idx = (*ptr as ::core::ffi::c_int - '0' as i32) as u_int;
            j = 0 as u_int;
            while j < 5 as u_int {
                i = 0 as u_int;
                while i < 5 as u_int {
                    if !(window_clock_table[idx as usize][j as usize][i as usize] == 0) {
                        screen_write_cursormove(
                            &mut *ctx,
                            x.wrapping_add(px).wrapping_add(i) as ::core::ffi::c_int,
                            y.wrapping_add(py).wrapping_add(j) as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                        );
                        screen_write_putc(&mut *ctx, &bgc, ' ' as i32 as u_char);
                    }
                    i = i.wrapping_add(1);
                }
                j = j.wrapping_add(1);
            }
            px = px.wrapping_add(6 as u_int);
        }
        ptr = ptr.offset(1);
    }
    if sy <= 6 as u_int {
        if let Some(owner) = session_owner {
            session_remove_ref(owner, c"window_panes_draw_number");
        }
        if let Some(owner) = source_session_owner {
            session_remove_ref(owner, c"window_panes_draw_number");
        }
        return;
    }
    window_panes_draw_format(data, ctx, wp, x, y, sx, &raw mut fgc);
    if llen != 0 as size_t {
        cx = (x.wrapping_add(px) as size_t)
            .wrapping_sub(llen)
            .wrapping_sub(1 as size_t) as u_int;
        cy = y.wrapping_add(py).wrapping_add(5 as u_int);
        screen_write_cursormove(
            &mut *ctx,
            cx as ::core::ffi::c_int,
            cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_puts(&mut *ctx, &fgc, |out| {
            write_cstr(out, &raw mut lbuf as *mut ::core::ffi::c_char)
        });
    }
    if let Some(owner) = session_owner {
        session_remove_ref(owner, c"window_panes_draw_number");
    }
    if let Some(owner) = source_session_owner {
        session_remove_ref(owner, c"window_panes_draw_number");
    }
}
unsafe fn window_panes_draw_pane(
    mut data: *mut window_panes_modedata,
    mut ctx: *mut screen_write_ctx,
    mut wp: *mut window_pane,
    mut root: *mut layout_cell,
    mut osx: u_int,
    mut osy: u_int,
    mut dsx: u_int,
    mut dsy: u_int,
) {
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut s: *mut screen = &raw mut (*wp).base;
    let mut pane: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if window_panes_pane_visible(wp) == 0 {
        return;
    }
    if window_panes_get_geometry(
        wp,
        root,
        osx,
        osy,
        dsx,
        dsy,
        &raw mut x,
        &raw mut y,
        &raw mut sx,
        &raw mut sy,
    ) == 0
    {
        return;
    }
    if window_panes_clip_floating_pane(
        wp,
        osx,
        osy,
        dsx,
        dsy,
        &raw mut x,
        &raw mut y,
        &raw mut sx,
        &raw mut sy,
    ) == 0
    {
        return;
    }
    if !window_pane_index(&*wp).map(|value| { pane = value; }).is_some() {
        return;
    }
    window_panes_add_area(data, wp, x, y, sx, sy);
    screen_write_cursormove(
        &mut *ctx,
        x as ::core::ffi::c_int,
        y as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    let preview = (*data)
        .preview
        .as_deref_mut()
        .map_or(::core::ptr::null_mut(), |preview| preview as *mut screen);
    if !preview.is_null()
        && wp == mode_pane
        && sx <= (*preview).grid().sx
        && sy <= (*preview).grid().sy
    {
        s = preview;
    }
    if osx <= dsx && osy <= dsy {
        screen_write_fast_copy(&mut *ctx, &*s, 0 as u_int, (*s).grid().hsize, sx, sy);
    } else {
        screen_write_preview(&mut *ctx, &*s, sx, sy);
    }
    window_panes_draw_number(data, ctx, wp, pane, x, y, sx, sy);
}
unsafe fn window_panes_draw_screen(mut wme: *mut window_mode_entry) {
    let mut source_session_owner = None;
    let mut data: *mut window_panes_modedata = (*wme).data as *mut window_panes_modedata;
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut root: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut border_gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut osx: u_int = 0;
    let mut osy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if window_panes_get_source(
        data,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
        &raw mut w,
        &mut source_session_owner,
    ) == 0
    {
        if let Some(owner) = source_session_owner {
            session_remove_ref(owner, c"window_panes_draw_screen");
        }
        return;
    }
    root = (*w).saved_layout_root_ptr().map_or(std::ptr::null_mut(), |root| root);
    if root.is_null() {
        root = (*w).layout_root_ptr().map_or(std::ptr::null_mut(), |root| root);
    }
    if root.is_null() {
        if let Some(owner) = source_session_owner {
            session_remove_ref(owner, c"window_panes_draw_screen");
        }
        return;
    }
    osx = (*root).g.sx;
    osy = (*root).g.sy;
    sx = (*data).screen.grid().sx;
    sy = (*data).screen.grid().sy;
    window_panes_free_areas(data);
    screen_write_start(&mut ctx, &raw mut (*data).screen);
    screen_write_clearscreen(&mut ctx, 8 as u_int);
    wp = window_pane_first(w).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !wp.is_null() {
        if !(window_panes_pane_floating(wp) != 0) {
            window_panes_draw_pane(data, &raw mut ctx, wp, root, osx, osy, sx, sy);
        }
        wp = window_pane_next(wp).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    }
    window_panes_get_border_cell(data, &raw mut border_gc);
    window_panes_draw_borders(&raw mut ctx, w, root, &raw mut border_gc, osx, osy, sx, sy);
    wp = window_pane_z_last(w).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !wp.is_null() {
        if !(window_panes_pane_floating(wp) == 0) {
            window_panes_clear_floating_area(&raw mut ctx, wp, osx, osy, sx, sy);
            window_panes_draw_pane(data, &raw mut ctx, wp, root, osx, osy, sx, sy);
            window_panes_draw_floating_border(
                &raw mut ctx,
                wp,
                &raw mut border_gc,
                osx,
                osy,
                sx,
                sy,
            );
        }
        wp = window_pane_z_previous(wp).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    }
    screen_write_stop(&mut ctx);
    (*mode_pane).flags |= PANE_REDRAW;
    if let Some(owner) = source_session_owner {
        session_remove_ref(owner, c"window_panes_draw_screen");
    }
}
unsafe fn window_panes_timer_callback(mut arg: *mut ::core::ffi::c_void) {
    let mut wme: *mut window_mode_entry = arg as *mut window_mode_entry;
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    window_pane_reset_mode(&mode_pane_owner);
}
unsafe fn window_panes_init(
    mut wme: *mut window_mode_entry,
    mut item: *mut cmdq_item,
    _fs: *mut cmd_find_state,
    mut args: *mut args,
) -> *mut screen {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut w: *mut window = (*wp).window as *mut window;
    let mut data: *mut window_panes_modedata = ::core::ptr::null_mut::<window_panes_modedata>();
    let mut self_0: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut source: *mut cmd_find_state = ::core::ptr::null_mut::<cmd_find_state>();
    let mut target: *mut cmd_find_state = ::core::ptr::null_mut::<cmd_find_state>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut sx: u_int = (*wp).base.grid().sx;
    let mut sy: u_int = (*wp).base.grid().sy;
    let mut delay: u_int = 0;
    if item.is_null() {
        return ::core::ptr::null_mut::<screen>();
    }
    self_0 = cmdq_get_cmd(item);
    if self_0.is_null() {
        return ::core::ptr::null_mut::<screen>();
    }
    source = crate::src::cmd::queue::cmdq_get_source_mut(&mut *item);
    target = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    s = (*target).s_ptr();
    if args_has(args, 'd' as i32 as u_char) == 0 {
        delay = options_get_number(
            options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
            b"display-panes-time\0" as *const u8 as *const ::core::ffi::c_char,
        ) as u_int;
    } else {
        delay = match args_strtonum_result(
            args,
            'd' as i32 as u_char,
            0 as ::core::ffi::c_longlong,
            UINT_MAX as ::core::ffi::c_longlong,
        ) {
            Ok(value) => value as u_int,
            Err(error) => {
                cmdq_error(item, |out| {
                    out.write_all(b"delay ")?;
                    write_cstr(out, error.message().as_ptr())
                });
                return ::core::ptr::null_mut::<screen>();
            }
        };
    }
    data = Box::into_raw(Box::new(window_panes_modedata {
        wp: Weak::new(),
        session: Weak::new(),
        source_session: 0,
        source_window: 0,
        screen: screen::empty(),
        preview: None,
        timer: Default::default(),
        state: None,
        delay: 0,
        ignore_keys: 0,
        zoomed: 0,
        areas: Vec::new(),
    }));
    (*wme).data = data as *mut ::core::ffi::c_void;
    (*data).wp = window_pane_weak(wp);
    (*data).session = (*s).observer.clone();
    screen_init(&mut (*data).screen, sx, sy, 0 as u_int);
    (*data).screen.mode &= !MODE_CURSOR;
    (*data).state = Some(args_make_commands_prepare(
        self_0,
        item,
        0 as u_int,
        b"select-pane -t \"%%%\"\0" as *const u8 as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    ));
    if args_has(args, 's' as i32 as u_char) != 0 {
        (*data).source_session = (*(*source).s_ptr()).id;
        (*data).source_window = (*(*source).w_ptr()).id;
    } else {
        (*data).source_session = (*(*target).s_ptr()).id;
        (*data).source_window = (*(*target).w_ptr()).id;
    }
    (*data).delay = delay;
    (*data).ignore_keys = args_has(args, 'N' as i32 as u_char);
    if args_has(args, 'Z' as i32 as u_char) != 0 {
        (*data).zoomed = -(1 as ::core::ffi::c_int);
    } else {
        (*data).zoomed = (*w).flags & WINDOW_ZOOMED;
        if (*data).zoomed == 0 {
            window_panes_set_preview(data);
        }
        if (*data).zoomed == 0 && window_zoom(wp) == 0 as ::core::ffi::c_int {
            server_redraw_window(w);
        }
    }
    event_set(
        &raw mut (*data).timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        move |_, _| unsafe { window_panes_timer_callback(wme as *mut ::core::ffi::c_void) },
    );
    if (*data).delay != 0 as u_int {
        tv.tv_sec = (*data).delay.wrapping_div(1000 as u_int) as __time_t;
        tv.tv_usec = (*data)
            .delay
            .wrapping_rem(1000 as u_int)
            .wrapping_mul(1000 as u_int) as __suseconds_t;
        event_add(&raw mut (*data).timer, &raw mut tv);
    }
    window_panes_draw_screen(wme);
    return &raw mut (*data).screen;
}
unsafe fn window_panes_free(mut wme: *mut window_mode_entry) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut data: *mut window_panes_modedata = (*wme).data as *mut window_panes_modedata;
    let mut w: *mut window = (*mode_pane).window as *mut window;
    event_del(&raw mut (*data).timer);
    if (*data).zoomed == 0 as ::core::ffi::c_int {
        server_unzoom_window(w);
    }
    server_redraw_window(w);
    server_redraw_window_borders(w);
    server_status_window(w);
    drop((*data).state.take());
    window_panes_free_areas(data);
    if let Some(mut preview) = (*data).preview.take() {
        screen_free(&mut *preview);
    }
    screen_free(&mut (*data).screen);
    drop(Box::from_raw(data));
}
unsafe fn window_panes_resize(mut wme: *mut window_mode_entry, mut sx: u_int, mut sy: u_int) {
    let mut data: *mut window_panes_modedata = (*wme).data as *mut window_panes_modedata;
    screen_resize(&mut (*data).screen, sx, sy, 0 as ::core::ffi::c_int);
    window_panes_draw_screen(wme);
}
unsafe fn window_panes_run_command(
    mut data: *mut window_panes_modedata,
    client_owner: &Rc<UnsafeCell<client>>,
    pane: &window_pane,
) {
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let expanded = CString::new(format!("%{}", pane.id)).expect("pane ID contains NUL");
    match args_make_commands(
        (*data)
            .state
            .as_deref_mut()
            .expect("prepared command state"),
        &vec![expanded],
    ) {
        Err(error) => {
            cmdq_append(
                Some(client_owner),
                cmdq_get_error(
                    error
                        .as_ref()
                        .map_or(::core::ptr::null(), |cause| cause.as_ptr()),
                ),
            );
        }
        Ok(commands) => {
            let cmdlist = commands;
            new_item =
                cmdq_get_command(&cmdlist, None);
            cmdq_append(Some(client_owner), new_item);
            drop(cmdlist);
        }
    }
}
unsafe fn window_panes_find_pane(
    mut data: *mut window_panes_modedata,
    mut x: u_int,
    mut y: u_int,
) -> Option<Rc<UnsafeCell<window_pane>>> {
    for area in (*data).areas.iter().rev() {
        if !(x < area.x || x >= area.x.wrapping_add(area.sx)) {
            if !(y < area.y || y >= area.y.wrapping_add(area.sy)) {
                return window_pane_find_by_id(area.id);
            }
        }
    }
    return None;
}
unsafe fn window_panes_key_pane(
    mut data: *mut window_panes_modedata,
    mut key: key_code,
) -> Option<Rc<UnsafeCell<window_pane>>> {
    let mut source_session_owner = None;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut index: u_int = 0;
    if key >= '0' as i32 as key_code && key <= '9' as i32 as key_code {
        index = key.wrapping_sub('0' as i32 as key_code) as u_int;
    } else if key as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS == 0 as ::core::ffi::c_ulonglong
    {
        key &= KEYC_MASK_KEY;
        if key < 'a' as i32 as key_code || key > 'z' as i32 as key_code {
            return None;
        }
        index = (10 as key_code).wrapping_add(key.wrapping_sub('a' as i32 as key_code)) as u_int;
    } else {
        return None;
    }
    if window_panes_get_source(
        data,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
        &raw mut w,
        &mut source_session_owner,
    ) == 0
    {
        if let Some(owner) = source_session_owner {
            session_remove_ref(owner, c"window_panes_key_pane");
        }
        return None;
    }
    let result = window_pane_at_index(&mut *w, index);
    if let Some(owner) = source_session_owner {
        session_remove_ref(owner, c"window_panes_key_pane");
    }
    return result;
}
unsafe fn window_panes_get_target(
    mut wme: *mut window_mode_entry,
    mut key: key_code,
    mut m: *mut mouse_event,
) -> Option<Rc<UnsafeCell<window_pane>>> {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut data: *mut window_panes_modedata = (*wme).data as *mut window_panes_modedata;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*data).ignore_keys != 0 {
        return None;
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
    {
        if key != KEYC_MOUSEDOWN1_PANE as ::core::ffi::c_ulong as key_code
            || m.is_null()
            || cmd_mouse_at(
                mode_pane,
                m,
                &raw mut x,
                &raw mut y,
                0 as ::core::ffi::c_int,
            ) != 0 as ::core::ffi::c_int
        {
            return None;
        }
        return window_panes_find_pane(data, x, y);
    }
    return window_panes_key_pane(data, key);
}
unsafe fn window_panes_key(
    mut wme: *mut window_mode_entry,
    client_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>,
    _wl: *mut winlink,
    mut key: key_code,
    mut m: *mut mouse_event,
) {
    let c = client_owner.get();
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut target: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut data: *mut window_panes_modedata = (*wme).data as *mut window_panes_modedata;
    if key == '\u{1b}' as i32 as key_code || key == 'q' as i32 as key_code {
        window_pane_reset_mode(&mode_pane_owner);
        return;
    }
    let target_owner = window_panes_get_target(wme, key, m);
    target = target_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if target.is_null() {
        if (*data).ignore_keys == 0
            && !(key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                    >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                        << 32 as ::core::ffi::c_int
                    && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                        <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int
                            as ::core::ffi::c_ulonglong)
                            << 32 as ::core::ffi::c_int)
        {
            window_pane_reset_mode(&mode_pane_owner);
        }
        return;
    }
    if (*(*wp).window).flags & WINDOW_ZOOMED != 0 {
        window_unzoom((*wp).window as *mut window, 1 as ::core::ffi::c_int);
    }
    window_panes_run_command(data, client_owner, &*target);
    window_pane_reset_mode(&mode_pane_owner);
}

#[cfg(test)]
mod session_observer_tests {
    use super::*;
    use crate::src::reactor::shutdown_runtime;
    use crate::src::session::{sessions, sessions_insert, sessions_remove};

    #[test]
    fn session_observer_rejects_removed_sessions_and_defers_guard_cleanup() {
        unsafe {
            let saved = std::ptr::replace(
                &raw mut sessions,
                crate::src::shared::session::sessions { storage: None },
            );
            let owner = session::new();
            let session = rc::as_ptr(&owner);
            (*session).name = c"panes-mode-session".to_owned();
            sessions_insert(&raw mut sessions, owner);
            let observer = (*session).observer.clone();
            let mut mode = window_panes_modedata {
                wp: Weak::new(),
                session: observer.clone(),
                source_session: 0,
                source_window: 0,
                screen: screen::empty(),
                preview: None,
                timer: Default::default(),
                state: None,
                delay: 0,
                ignore_keys: 0,
                zoomed: 0,
                areas: Vec::new(),
            };
            assert_eq!((*session).observer.strong_count(), 1);
            let guard = window_panes_session(&mut mode).unwrap();
            assert_eq!(rc::as_ptr(&guard), session);
            let owner = sessions_remove(&raw mut sessions, session).unwrap();
            assert!(window_panes_session(&mut mode).is_none());
            assert_eq!(observer.strong_count(), 3, "rejected upgrade release is deferred");
            crate::src::reactor::event_loop();
            assert_eq!(observer.strong_count(), 2);
            drop(owner);
            assert!(observer.upgrade().is_some(), "guard keeps allocation alive");
            session_remove_ref(guard, c"session-observer-test");
            assert!(observer.upgrade().is_some(), "guard cleanup is deferred");
            shutdown_runtime();
            assert!(observer.upgrade().is_none());
            assert!(window_panes_session(&mut mode).is_none());
            sessions = saved;
        }
    }
}
