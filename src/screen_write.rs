use crate::src::ffi::libc::{memcpy, memset, strlen};
use crate::src::format_draw::format_draw;
use crate::src::grid::view::{
    grid_view_clear, grid_view_clear_history, grid_view_delete_cells, grid_view_delete_lines,
    grid_view_delete_lines_region, grid_view_get_cell, grid_view_insert_cells,
    grid_view_insert_lines, grid_view_insert_lines_region, grid_view_scroll_region_down,
    grid_view_scroll_region_up, grid_view_set_cell, grid_view_set_cells, grid_view_set_padding,
};
use crate::src::grid::{
    grid_cells_equal, grid_clear_history, grid_default_cell, grid_get_cell, grid_get_line,
};
use crate::src::layout::layout_fix_panes;
use crate::src::log::{fatal, fatalx, log_debug, log_get_level};
use crate::src::options::options_get_number;
use crate::src::reactor::{event_add, event_del, event_initialized, event_pending, event_set};
use crate::src::screen::{
    screen_alternate_off, screen_alternate_on, screen_check_selection, screen_mode_to_string,
    screen_reset_tabs, screen_select_cell,
};
use crate::src::server_fn::server_redraw_window_borders;
use crate::src::session::session_has;
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::borders::{
    CELL_BORDERS, CELL_LD, CELL_LR, CELL_LU, CELL_RD, CELL_RU, CELL_UD, CELL_ULD, CELL_URD,
    SIMPLE_BORDERS,
};
use crate::src::shared::client::client;
use crate::src::shared::client::CLIENT_REDRAWWINDOW;
use crate::src::shared::colour::*;
use crate::src::shared::display::{visible_range, visible_ranges};
use crate::src::shared::event::EV_TIMEOUT;
use crate::src::shared::grid::*;
use crate::src::shared::hyperlinks::hyperlinks;
use crate::src::shared::layout::*;
use crate::src::shared::limits::UINT_MAX;
use crate::src::shared::menu::menu;
use crate::src::shared::options::options;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::{window_pane_resize, PANE_DROP, PANE_REDRAW, PANE_REDRAWSCROLLBAR};
use crate::src::shared::screen::{
    screen, EXTENDED_KEY_MODES, MODE_CURSOR, MODE_INSERT, MODE_KEYS_EXTENDED, MODE_ORIGIN,
    MODE_SYNC, MODE_WRAP,
};
use crate::src::shared::screen_write::{
    screen_write_citem, screen_write_cline, screen_write_items, CLEAR,
};
use crate::src::shared::screen_write::{screen_write_ctx, screen_write_init_ctx_cb};
use crate::src::shared::style::*;
use crate::src::shared::tty::{
    tty, tty_ctx, tty_ctx_c2rust_unnamed, tty_ctx_redraw_cb, tty_ctx_set_client_cb, tty_style_ctx,
};
use crate::src::shared::tty::{
    TTY_CTX_CELL_INVALIDATE, TTY_CTX_INVISIBLE_PANES, TTY_CTX_OVERLAY_SYNC, TTY_CTX_PANE_OBSCURED,
    TTY_CTX_SYNC, TTY_CTX_WINDOW_BIGGER, TTY_CTX_WRAPPED,
};
use crate::src::shared::utf8::*;
use crate::src::shared::window::window;
use crate::src::status::{status_at_line, status_line_size};
use crate::src::text::utf8::{utf8_append, utf8_copy, utf8_fromcstr_vec, utf8_open, utf8_set};
use crate::src::text::utf8_combined::{
    hanguljamo_check_state, utf8_has_zwj, utf8_is_hangul_filler, utf8_is_vs, utf8_is_zwj,
    utf8_should_combine,
};
use crate::src::tmux::global_options;
use crate::src::tty::{
    tty_cmd_alignmenttest, tty_cmd_cell, tty_cmd_cells, tty_cmd_clearcharacter,
    tty_cmd_clearendofscreen, tty_cmd_clearscreen, tty_cmd_clearstartofscreen,
    tty_cmd_deletecharacter, tty_cmd_deleteline, tty_cmd_insertcharacter, tty_cmd_insertline,
    tty_cmd_rawstring, tty_cmd_redrawline, tty_cmd_reverseindex, tty_cmd_scrolldown,
    tty_cmd_scrollup, tty_cmd_setselection, tty_cmd_syncstart, tty_default_colours,
    tty_update_window_offset, tty_window_offset, tty_write,
};
use crate::src::tty_acs::{tty_acs_double_borders, tty_acs_heavy_borders, tty_acs_rounded_borders};
use crate::src::window::{
    window_pane_clear_resizes, window_pane_is_floating, window_pane_scrollbar_overlay_visible,
    window_pane_scrollbar_redraw, window_pane_send_resize, window_pane_z_previous,
};
use crate::src::window_visible::{window_position_is_visible, window_visible_ranges};
use crate::src::xmalloc::xvasprintf_cstring;

pub const PADDED_BORDERS: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"             \0") };
pub const SCREEN_WRITE_SYNC: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const SCREEN_WRITE_OBSCURED: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const SCREEN_WRITE_CHECKED_IF_OBSCURED: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
thread_local! {
    // Screen writing runs on the event-loop thread. Detached boxes preserve
    // FIFO reuse without owning raw pointers or a static initializer.
    static WRITE_ITEM_POOL: std::cell::RefCell<std::collections::VecDeque<Box<screen_write_citem>>> =
        const { std::cell::RefCell::new(std::collections::VecDeque::new()) };
}
fn screen_write_get_citem() -> Box<screen_write_citem> {
    let mut item = WRITE_ITEM_POOL
        .with(|pool| pool.borrow_mut().pop_front())
        .unwrap_or_else(|| Box::new(Default::default()));
    *item = Default::default();
    item
}
fn screen_write_free_citem(item: Box<screen_write_citem>) {
    WRITE_ITEM_POOL.with(|pool| pool.borrow_mut().push_back(item));
}
unsafe fn screen_write_current_item(ctx: *mut screen_write_ctx) -> *mut screen_write_citem {
    &raw mut **(*ctx).item.as_mut().expect("active screen write context")
}
unsafe fn screen_write_recycle_items(items: &mut screen_write_items) {
    while let Some(item) = items.pop_front() {
        screen_write_free_citem(item);
    }
}
unsafe extern "C" fn screen_write_offset_timer(
    _fd: ::core::ffi::c_int,
    _events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut w: *mut window = data as *mut window;
    tty_update_window_offset(w);
}
unsafe extern "C" fn screen_write_set_cursor(
    mut ctx: *mut screen_write_ctx,
    mut cx: ::core::ffi::c_int,
    mut cy: ::core::ffi::c_int,
) {
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut s: *mut screen = (*ctx).s;
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 10000 as __suseconds_t,
    };
    if cx != -(1 as ::core::ffi::c_int)
        && cx as u_int == (*s).cx
        && cy != -(1 as ::core::ffi::c_int)
        && cy as u_int == (*s).cy
    {
        return;
    }
    if cx != -(1 as ::core::ffi::c_int) {
        if cx as u_int > (*(*s).grid).sx {
            cx = (*(*s).grid).sx.wrapping_sub(1 as u_int) as ::core::ffi::c_int;
        }
        (*s).cx = cx as u_int;
    }
    if cy != -(1 as ::core::ffi::c_int) {
        if cy as u_int > (*(*s).grid).sy.wrapping_sub(1 as u_int) {
            cy = (*(*s).grid).sy.wrapping_sub(1 as u_int) as ::core::ffi::c_int;
        }
        (*s).cy = cy as u_int;
    }
    if wp.is_null() {
        return;
    }
    w = (*wp).window as *mut window;
    if event_initialized(&(*w).offset_timer) == 0 {
        event_set(
            &raw mut (*w).offset_timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            Some(
                screen_write_offset_timer
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            w as *mut ::core::ffi::c_void,
        );
    }
    if event_pending(
        &raw mut (*w).offset_timer,
        EV_TIMEOUT as ::core::ffi::c_short,
        ::core::ptr::null_mut::<timeval>(),
    ) == 0
    {
        event_add(&raw mut (*w).offset_timer, &raw mut tv);
    }
}
unsafe extern "C" fn screen_write_redraw_cb(mut ttyctx: *const tty_ctx) {
    let mut wp: *mut window_pane = (*ttyctx).arg as *mut window_pane;
    if !wp.is_null() {
        (*wp).flags |= PANE_REDRAW;
    }
}
unsafe extern "C" fn screen_write_set_client_cb(
    mut ttyctx: *mut tty_ctx,
    mut c: *mut client,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = (*ttyctx).arg as *mut window_pane;
    if (*ttyctx).flags & TTY_CTX_INVISIBLE_PANES != 0 {
        if session_has((*c).session, (*wp).window as *mut window) != 0 {
            return 1 as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
    if (*(*(*c).session).curw).window != (*wp).window {
        return 0 as ::core::ffi::c_int;
    }
    if (*wp).layout_cell.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*wp).flags & (PANE_REDRAW | PANE_DROP) != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    if (*c).flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
        log_debug(
            b"%s: adding %%%u to deferred redraw\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_write_set_client_cb\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).id,
        );
        (*wp).flags |= PANE_REDRAW | PANE_REDRAWSCROLLBAR;
        return -(1 as ::core::ffi::c_int);
    }
    if tty_window_offset(
        &raw mut (*c).tty,
        &raw mut (*ttyctx).wox,
        &raw mut (*ttyctx).woy,
        &raw mut (*ttyctx).wsx,
        &raw mut (*ttyctx).wsy,
    ) != 0
    {
        (*ttyctx).flags |= TTY_CTX_WINDOW_BIGGER;
    } else {
        (*ttyctx).flags &= !TTY_CTX_WINDOW_BIGGER;
    }
    (*ttyctx).rxoff = (*wp).xoff;
    (*ttyctx).xoff = (*ttyctx).rxoff;
    (*ttyctx).ryoff = (*wp).yoff;
    (*ttyctx).yoff = (*ttyctx).ryoff;
    if status_at_line(c) == 0 as ::core::ffi::c_int {
        (*ttyctx).yoff = ((*ttyctx).yoff as u_int).wrapping_add(status_line_size(c))
            as ::core::ffi::c_int as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn screen_write_pane_is_obscured(
    mut ctx: *mut screen_write_ctx,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    if (*ctx).wp.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*ctx).flags & SCREEN_WRITE_CHECKED_IF_OBSCURED != 0 {
        if (*ctx).flags & SCREEN_WRITE_OBSCURED != 0 {
            return 1 as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
    (*ctx).flags |= SCREEN_WRITE_CHECKED_IF_OBSCURED;
    if (*(*ctx).wp).xoff < 0 as ::core::ffi::c_int
        || (*(*ctx).wp).yoff < 0 as ::core::ffi::c_int
        || ((*(*ctx).wp).xoff as u_int).wrapping_add((*(*ctx).wp).sx) > (*(*(*ctx).wp).window).sx
        || ((*(*ctx).wp).yoff as u_int).wrapping_add((*(*ctx).wp).sy) > (*(*(*ctx).wp).window).sy
    {
        (*ctx).flags |= SCREEN_WRITE_OBSCURED;
        return 1 as ::core::ffi::c_int;
    }
    loop {
        wp = window_pane_z_previous(wp);
        if wp.is_null() {
            break;
        }
        if window_pane_is_floating(wp) != 0
            && ((*wp).yoff >= (*(*ctx).wp).yoff
                && (*wp).yoff <= (*(*ctx).wp).yoff + (*(*ctx).wp).sy as ::core::ffi::c_int
                || (*wp).yoff + (*wp).sy as ::core::ffi::c_int >= (*(*ctx).wp).yoff
                    && ((*wp).yoff as u_int).wrapping_add((*wp).sy)
                        <= ((*(*ctx).wp).yoff as u_int).wrapping_add((*(*ctx).wp).sy))
            && ((*wp).xoff >= (*(*ctx).wp).xoff
                && (*wp).xoff <= (*(*ctx).wp).xoff + (*(*ctx).wp).sx as ::core::ffi::c_int
                || (*wp).xoff + (*wp).sx as ::core::ffi::c_int >= (*(*ctx).wp).xoff
                    && ((*wp).xoff as u_int).wrapping_add((*wp).sx)
                        <= ((*(*ctx).wp).xoff as u_int).wrapping_add((*(*ctx).wp).sx))
        {
            (*ctx).flags |= SCREEN_WRITE_OBSCURED;
            return 1 as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn screen_write_should_draw_lines(
    mut ctx: *mut screen_write_ctx,
    mut y: u_int,
    mut ny: u_int,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    let mut s: *mut screen = (*ctx).s;
    let mut sy: u_int = (*(*s).grid).sy;
    let mut bs: *mut bitstr_t = ::core::ptr::null_mut::<bitstr_t>();
    if !wp.is_null() && (*wp).flags & (PANE_REDRAW | PANE_DROP) != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*s).mode & MODE_SYNC != 0 {
        if !wp.is_null() && y < sy && ny != 0 as u_int {
            bs = (*wp).sync_dirty;
            if ny > sy.wrapping_sub(y) {
                ny = sy.wrapping_sub(y);
            }
            if bs.is_null() || (*wp).sync_dirty_size != sy {
                if !bs.is_null() && (*wp).sync_dirty_size != sy {
                    y = 0 as u_int;
                    ny = sy;
                }
                screen_write_clear_dirty(wp);
                let bytes = (sy.wrapping_add(7 as u_int) >> 3 as ::core::ffi::c_int) as usize;
                let mut dirty = Vec::<bitstr_t>::new();
                if dirty.try_reserve_exact(bytes).is_err() {
                    fatal(b"bit_alloc failed\0" as *const u8 as *const ::core::ffi::c_char);
                }
                dirty.resize(bytes, 0);
                (*wp).sync_dirty = Box::into_raw(dirty.into_boxed_slice()) as *mut bitstr_t;
                bs = (*wp).sync_dirty;
                (*wp).sync_dirty_size = sy;
            }
            let mut _name: *mut bitstr_t = bs;
            let mut _start: ::core::ffi::c_int = y as ::core::ffi::c_int;
            let mut _stop: ::core::ffi::c_int =
                y.wrapping_add(ny).wrapping_sub(1 as u_int) as ::core::ffi::c_int;
            while _start <= _stop {
                let ref mut fresh2 = *_name.offset((_start >> 3 as ::core::ffi::c_int) as isize);
                *fresh2 = (*fresh2 as ::core::ffi::c_int
                    | (1 as ::core::ffi::c_int) << (_start & 0x7 as ::core::ffi::c_int))
                    as bitstr_t;
                _start += 1;
            }
        }
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn screen_write_should_draw_line(
    mut ctx: *mut screen_write_ctx,
    mut y: u_int,
) -> ::core::ffi::c_int {
    return screen_write_should_draw_lines(ctx, y, 1 as u_int);
}
unsafe extern "C" fn screen_write_initctx(
    mut ctx: *mut screen_write_ctx,
    mut ttyctx: *mut tty_ctx,
    mut is_sync: ::core::ffi::c_int,
    mut check_obscured: ::core::ffi::c_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut palette: *mut colour_palette = ::core::ptr::null_mut::<colour_palette>();
    memset(
        ttyctx as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<tty_ctx>() as size_t,
    );
    (*ttyctx).s = s;
    (*ttyctx).sx = (*(*s).grid).sx;
    (*ttyctx).sy = (*(*s).grid).sy;
    (*ttyctx).ocx = (*s).cx;
    (*ttyctx).ocy = (*s).cy;
    (*ttyctx).orlower = (*s).rlower;
    (*ttyctx).orupper = (*s).rupper;
    if check_obscured != 0 && screen_write_pane_is_obscured(ctx) != 0 {
        (*ttyctx).flags |= TTY_CTX_PANE_OBSCURED;
    }
    memcpy(
        &raw mut (*ttyctx).defaults as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    (*ttyctx).style_ctx.defaults = &raw mut (*ttyctx).defaults;
    (*ttyctx).style_ctx.hyperlinks = (*(*ctx).s).hyperlinks;
    if (*ctx).init_ctx_cb.is_some() {
        (*ctx).init_ctx_cb.expect("non-null function pointer")(ctx, ttyctx);
        if !(*ttyctx).style_ctx.palette.is_null() {
            palette = (*ttyctx).style_ctx.palette;
            if (*ttyctx).defaults.fg == 8 as ::core::ffi::c_int {
                (*ttyctx).defaults.fg = (*palette).fg;
            }
            if (*ttyctx).defaults.bg == 8 as ::core::ffi::c_int {
                (*ttyctx).defaults.bg = (*palette).bg;
            }
        }
    } else {
        (*ttyctx).redraw_cb =
            Some(screen_write_redraw_cb as unsafe extern "C" fn(*const tty_ctx) -> ())
                as tty_ctx_redraw_cb;
        if !(*ctx).wp.is_null() {
            tty_default_colours(
                &raw mut (*ttyctx).defaults,
                (*ctx).wp as *mut window_pane,
                &raw mut (*ttyctx).style_ctx.dim,
            );
            (*ttyctx).style_ctx.palette = &raw mut (*(*ctx).wp).palette;
            (*ttyctx).set_client_cb = Some(
                screen_write_set_client_cb
                    as unsafe extern "C" fn(*mut tty_ctx, *mut client) -> ::core::ffi::c_int,
            ) as tty_ctx_set_client_cb;
            (*ttyctx).arg = (*ctx).wp as *mut ::core::ffi::c_void;
        }
    }
    if !(*ctx).flags & SCREEN_WRITE_SYNC != 0 {
        if !(*ctx).wp.is_null()
            && ((*ctx).wp != (*(*(*ctx).wp).window).active
                || (*(*ctx).wp).screen != &raw mut (*(*ctx).wp).base)
        {
            (*ttyctx).flags |= TTY_CTX_SYNC;
        } else {
            if (*ctx).wp.is_null() {
                (*ttyctx).flags |= TTY_CTX_OVERLAY_SYNC;
            }
            if is_sync != 0 {
                (*ttyctx).flags |= TTY_CTX_SYNC;
            }
        }
        tty_write(
            Some(tty_cmd_syncstart as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            ttyctx,
        );
        (*ctx).flags |= SCREEN_WRITE_SYNC;
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_make_list(mut s: *mut screen) {
    if (*(*s).grid).sy == 0 {
        fatalx(b"xcalloc: zero size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    let rows = (0..(*(*s).grid).sy)
        .map(|_| screen_write_cline::default())
        .collect::<Vec<_>>()
        .into_boxed_slice();
    (*s).write_list = Box::into_raw(rows) as *mut screen_write_cline;
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_free_list(mut s: *mut screen) {
    let mut cl: *mut screen_write_cline = ::core::ptr::null_mut::<screen_write_cline>();
    let _ci: *mut screen_write_citem = ::core::ptr::null_mut::<screen_write_citem>();
    let _ci1: *mut screen_write_citem = ::core::ptr::null_mut::<screen_write_citem>();
    let mut y: u_int = 0;
    y = 0 as u_int;
    while y < (*(*s).grid).sy {
        cl = (*s).write_list.offset(y as isize) as *mut screen_write_cline;
        screen_write_recycle_items(&mut (*cl).items);
        y = y.wrapping_add(1);
    }
    let rows = std::ptr::slice_from_raw_parts_mut((*s).write_list, (*(*s).grid).sy as usize);
    drop(Box::from_raw(rows));
    (*s).write_list = ::core::ptr::null_mut::<screen_write_cline>();
}
unsafe extern "C" fn screen_write_init(mut ctx: *mut screen_write_ctx, mut s: *mut screen) {
    // Accept fresh C storage; a previously started context must be stopped.
    std::ptr::write(ctx, Default::default());
    (*ctx).s = s;
    if (*(*ctx).s).write_list.is_null() {
        screen_write_make_list((*ctx).s);
    }
    (*ctx).item = Some(screen_write_get_citem());
    (*ctx).scrolled = 0 as u_int;
    (*ctx).bg = 8 as u_int;
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_start_pane(
    mut ctx: *mut screen_write_ctx,
    mut wp: *mut window_pane,
    mut s: *mut screen,
) {
    if s.is_null() {
        s = (*wp).screen;
    }
    screen_write_init(ctx, s);
    (*ctx).wp = wp as *mut window_pane;
    if log_get_level() != 0 as ::core::ffi::c_int {
        log_debug(
            b"%s: size %ux%u, pane %%%u (at %u,%u)\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_write_start_pane\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*(*ctx).s).grid).sx,
            (*(*(*ctx).s).grid).sy,
            (*wp).id,
            (*wp).xoff,
            (*wp).yoff,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_start_callback(
    mut ctx: *mut screen_write_ctx,
    mut s: *mut screen,
    mut cb: screen_write_init_ctx_cb,
    mut arg: *mut ::core::ffi::c_void,
) {
    screen_write_init(ctx, s);
    (*ctx).init_ctx_cb = cb;
    (*ctx).arg = arg;
    if log_get_level() != 0 as ::core::ffi::c_int {
        log_debug(
            b"%s: size %ux%u, with callback\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_write_start_callback\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*(*ctx).s).grid).sx,
            (*(*(*ctx).s).grid).sy,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_start(mut ctx: *mut screen_write_ctx, mut s: *mut screen) {
    screen_write_init(ctx, s);
    if log_get_level() != 0 as ::core::ffi::c_int {
        log_debug(
            b"%s: size %ux%u, no pane\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_write_start\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*(*ctx).s).grid).sx,
            (*(*(*ctx).s).grid).sy,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_stop(mut ctx: *mut screen_write_ctx) {
    screen_write_collect_end(ctx);
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_stop\0" as *const u8 as *const ::core::ffi::c_char,
    );
    screen_write_free_citem((*ctx).item.take().expect("active screen write context"));
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_reset(mut ctx: *mut screen_write_ctx) {
    let mut s: *mut screen = (*ctx).s;
    screen_reset_tabs(s);
    screen_write_scrollregion(ctx, 0 as u_int, (*(*s).grid).sy.wrapping_sub(1 as u_int));
    (*s).mode = MODE_CURSOR | MODE_WRAP;
    if options_get_number(
        global_options,
        b"extended-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 2 as ::core::ffi::c_longlong
    {
        (*s).mode = (*s).mode & !EXTENDED_KEY_MODES | MODE_KEYS_EXTENDED;
    }
    screen_write_clearscreen(ctx, 8 as u_int);
    screen_write_set_cursor(ctx, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_putc(
    mut ctx: *mut screen_write_ctx,
    mut gcp: *const grid_cell,
    mut ch: u_char,
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
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        gcp as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    utf8_set(&raw mut gc.data, ch);
    screen_write_cell(ctx, &raw mut gc);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_strlen(
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) -> size_t {
    let mut ap: ::core::ffi::VaList;
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut left: size_t = 0;
    let mut size: size_t = 0 as size_t;
    let mut more: utf8_state = UTF8_MORE;
    ap = args.clone();
    let msg = xvasprintf_cstring(fmt, ap);
    let mut ptr = msg.as_ptr() as *const u_char;
    while *ptr as ::core::ffi::c_int != '\0' as i32 {
        if *ptr as ::core::ffi::c_int > 0x7f as ::core::ffi::c_int
            && utf8_open(&raw mut ud, *ptr) as ::core::ffi::c_uint
                == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ptr = ptr.offset(1);
            left = strlen(ptr as *const ::core::ffi::c_char);
            if left < (ud.size as size_t).wrapping_sub(1 as size_t) {
                break;
            }
            loop {
                more = utf8_append(&raw mut ud, *ptr);
                if !(more as ::core::ffi::c_uint
                    == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    break;
                }
                ptr = ptr.offset(1);
            }
            ptr = ptr.offset(1);
            if more as ::core::ffi::c_uint == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                size = size.wrapping_add(ud.width as size_t);
            }
        } else {
            if *ptr as ::core::ffi::c_int == '\t' as i32
                || *ptr as ::core::ffi::c_int > 0x1f as ::core::ffi::c_int
                    && (*ptr as ::core::ffi::c_int) < 0x7f as ::core::ffi::c_int
            {
                size = size.wrapping_add(1);
            }
            ptr = ptr.offset(1);
        }
    }
    return size;
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_text(
    mut ctx: *mut screen_write_ctx,
    mut cx: u_int,
    mut width: u_int,
    mut lines: u_int,
    mut more: ::core::ffi::c_int,
    mut gcp: *const grid_cell,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) -> ::core::ffi::c_int {
    let mut s: *mut screen = (*ctx).s;
    let mut ap: ::core::ffi::VaList;
    let mut cy: u_int = (*s).cy;
    let mut i: u_int = 0;
    let mut end: u_int = 0;
    let mut next: u_int = 0;
    let mut idx: u_int = 0 as u_int;
    let mut at: u_int = 0;
    let mut left: u_int = 0;
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
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        gcp as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    ap = args.clone();
    let cells = {
        let tmp = xvasprintf_cstring(fmt, ap);
        utf8_fromcstr_vec(tmp.as_c_str())
    };
    let text = cells.as_ptr();
    left = cx.wrapping_add(width).wrapping_sub((*s).cx);
    loop {
        at = 0 as u_int;
        end = idx;
        while (*text.offset(end as isize)).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            if (*text.offset(end as isize)).size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                && (*text.offset(end as isize)).data[0 as ::core::ffi::c_int as usize]
                    as ::core::ffi::c_int
                    == '\n' as i32
            {
                break;
            }
            if at.wrapping_add((*text.offset(end as isize)).width as u_int) > left {
                break;
            }
            at = at.wrapping_add((*text.offset(end as isize)).width as u_int);
            end = end.wrapping_add(1);
        }
        if (*text.offset(end as isize)).size as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            next = end;
        } else if (*text.offset(end as isize)).size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            && (*text.offset(end as isize)).data[0 as ::core::ffi::c_int as usize]
                as ::core::ffi::c_int
                == '\n' as i32
        {
            next = end.wrapping_add(1 as u_int);
        } else if (*text.offset(end as isize)).size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            && (*text.offset(end as isize)).data[0 as ::core::ffi::c_int as usize]
                as ::core::ffi::c_int
                == ' ' as i32
        {
            next = end.wrapping_add(1 as u_int);
        } else {
            i = end;
            while i > idx {
                if (*text.offset(i as isize)).size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                    && (*text.offset(i as isize)).data[0 as ::core::ffi::c_int as usize]
                        as ::core::ffi::c_int
                        == ' ' as i32
                {
                    break;
                }
                i = i.wrapping_sub(1);
            }
            if i != idx {
                next = i.wrapping_add(1 as u_int);
                end = i;
            } else {
                next = end;
            }
        }
        i = idx;
        while i < end {
            utf8_copy(&raw mut gc.data, text.offset(i as isize) as *mut utf8_data);
            screen_write_cell(ctx, &raw mut gc);
            i = i.wrapping_add(1);
        }
        idx = next;
        if (*s).cy == cy.wrapping_add(lines).wrapping_sub(1 as u_int)
            || (*text.offset(idx as isize)).size as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        {
            break;
        }
        screen_write_cursormove(
            ctx,
            cx as ::core::ffi::c_int,
            (*s).cy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        left = width;
    }
    if (*s).cy == cy.wrapping_add(lines).wrapping_sub(1 as u_int)
        && (more == 0 || (*s).cx == cx.wrapping_add(width))
        || (*text.offset(idx as isize)).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if more == 0 || (*s).cx == cx.wrapping_add(width) {
        screen_write_cursormove(
            ctx,
            cx as ::core::ffi::c_int,
            (*s).cy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_puts(
    mut ctx: *mut screen_write_ctx,
    mut gcp: *const grid_cell,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    screen_write_vnputs(ctx, -(1 as ::core::ffi::c_int) as ssize_t, gcp, fmt, ap);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_nputs(
    mut ctx: *mut screen_write_ctx,
    mut maxlen: ssize_t,
    mut gcp: *const grid_cell,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    screen_write_vnputs(ctx, maxlen, gcp, fmt, ap);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_vnputs(
    mut ctx: *mut screen_write_ctx,
    mut maxlen: ssize_t,
    mut gcp: *const grid_cell,
    mut fmt: *const ::core::ffi::c_char,
    mut ap: ::core::ffi::VaList,
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
    let mut ud: *mut utf8_data = &raw mut gc.data;
    let mut left: size_t = 0;
    let mut size: size_t = 0 as size_t;
    let mut more: utf8_state = UTF8_MORE;
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        gcp as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    let msg = xvasprintf_cstring(fmt, ap);
    let mut ptr = msg.as_ptr() as *const u_char;
    while *ptr as ::core::ffi::c_int != '\0' as i32 {
        if *ptr as ::core::ffi::c_int > 0x7f as ::core::ffi::c_int
            && utf8_open(ud, *ptr) as ::core::ffi::c_uint
                == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ptr = ptr.offset(1);
            left = strlen(ptr as *const ::core::ffi::c_char);
            if left < ((*ud).size as size_t).wrapping_sub(1 as size_t) {
                break;
            }
            loop {
                more = utf8_append(ud, *ptr);
                if !(more as ::core::ffi::c_uint
                    == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    break;
                }
                ptr = ptr.offset(1);
            }
            ptr = ptr.offset(1);
            if more as ::core::ffi::c_uint != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                continue;
            }
            if maxlen > 0 as ssize_t && size.wrapping_add((*ud).width as size_t) > maxlen as size_t
            {
                while size < maxlen as size_t {
                    screen_write_putc(ctx, &raw mut gc, ' ' as i32 as u_char);
                    size = size.wrapping_add(1);
                }
                break;
            } else {
                size = size.wrapping_add((*ud).width as size_t);
                screen_write_cell(ctx, &raw mut gc);
            }
        } else {
            if maxlen > 0 as ssize_t && size.wrapping_add(1 as size_t) > maxlen as size_t {
                break;
            }
            if *ptr as ::core::ffi::c_int == '\u{1}' as i32 {
                gc.attr = (gc.attr as ::core::ffi::c_int ^ GRID_ATTR_CHARSET) as u_short;
            } else if *ptr as ::core::ffi::c_int == '\n' as i32 {
                screen_write_linefeed(ctx, 0 as ::core::ffi::c_int, 8 as u_int);
                screen_write_carriagereturn(ctx);
            } else if *ptr as ::core::ffi::c_int == '\t' as i32
                || *ptr as ::core::ffi::c_int > 0x1f as ::core::ffi::c_int
                    && (*ptr as ::core::ffi::c_int) < 0x7f as ::core::ffi::c_int
            {
                size = size.wrapping_add(1);
                screen_write_putc(ctx, &raw mut gc, *ptr);
            }
            ptr = ptr.offset(1);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_fast_copy(
    mut ctx: *mut screen_write_ctx,
    mut src: *mut screen,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut ny: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut gd: *mut grid = (*src).grid;
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut sgl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
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
    let mut xx: u_int = 0;
    let mut yy: u_int = 0;
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    let mut xoff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut yoff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    if nx == 0 as u_int || ny == 0 as u_int {
        return;
    }
    if !wp.is_null() {
        xoff = (*wp).xoff;
        yoff = (*wp).yoff;
    }
    yy = py;
    while yy < py.wrapping_add(ny) {
        if yy >= (*gd).hsize.wrapping_add((*gd).sy) {
            break;
        }
        (*s).cx = cx;
        screen_write_initctx(
            ctx,
            &raw mut ttyctx,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        r = window_visible_ranges(
            wp,
            (xoff as u_int).wrapping_add((*s).cx) as ::core::ffi::c_int,
            (*s).cy.wrapping_add(yoff as u_int) as ::core::ffi::c_int,
            nx,
            ::core::ptr::null_mut::<visible_ranges>(),
        );
        xx = px;
        while xx < px.wrapping_add(nx) {
            gl = grid_get_line(gd, yy);
            sgl = grid_get_line((*s).grid, (*s).cy);
            if xx >= (*gl).cellsize as u_int && (*s).cx >= (*sgl).cellsize as u_int {
                break;
            }
            grid_get_cell(gd, xx, yy, &raw mut gc);
            if xx.wrapping_add(gc.data.width as u_int) > px.wrapping_add(nx) {
                break;
            }
            grid_view_set_cell((*s).grid, (*s).cx, (*s).cy, &raw mut gc);
            if window_position_is_visible(r, (xoff as u_int).wrapping_add((*s).cx)) == 0 {
                break;
            }
            ttyctx.cell = &raw mut gc;
            ttyctx.flags &= TTY_CTX_OVERLAY_SYNC | TTY_CTX_SYNC;
            tty_write(
                Some(tty_cmd_cell as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
                &raw mut ttyctx,
            );
            ttyctx.ocx = ttyctx.ocx.wrapping_add(1);
            (*s).cx = (*s).cx.wrapping_add(1);
            xx = xx.wrapping_add(1);
        }
        (*s).cy = (*s).cy.wrapping_add(1);
        yy = yy.wrapping_add(1);
    }
    (*s).cx = cx;
    (*s).cy = cy;
}
unsafe extern "C" fn screen_write_box_border_set(
    mut lines: box_lines,
    mut cell_type: ::core::ffi::c_int,
    mut gc: *mut grid_cell,
) {
    match lines as ::core::ffi::c_int {
        1 => {
            (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
            utf8_copy(&raw mut (*gc).data, tty_acs_double_borders(cell_type));
        }
        2 => {
            (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
            utf8_copy(&raw mut (*gc).data, tty_acs_heavy_borders(cell_type));
        }
        4 => {
            (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
            utf8_copy(&raw mut (*gc).data, tty_acs_rounded_borders(cell_type));
        }
        3 => {
            (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
            utf8_set(
                &raw mut (*gc).data,
                SIMPLE_BORDERS[cell_type as usize] as u_char,
            );
        }
        5 => {
            (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
            utf8_set(
                &raw mut (*gc).data,
                PADDED_BORDERS[cell_type as usize] as u_char,
            );
        }
        0 | -1 => {
            (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
            utf8_set(
                &raw mut (*gc).data,
                CELL_BORDERS[cell_type as usize] as u_char,
            );
        }
        6 | _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_hline(
    mut ctx: *mut screen_write_ctx,
    mut nx: u_int,
    mut left: ::core::ffi::c_int,
    mut right: ::core::ffi::c_int,
    mut lines: box_lines,
    mut border_gc: *const grid_cell,
) {
    let mut s: *mut screen = (*ctx).s;
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
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    let mut i: u_int = 0;
    cx = (*s).cx;
    cy = (*s).cy;
    if !border_gc.is_null() {
        memcpy(
            &raw mut gc as *mut ::core::ffi::c_void,
            border_gc as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    } else {
        memcpy(
            &raw mut gc as *mut ::core::ffi::c_void,
            &raw const grid_default_cell as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    }
    gc.attr = (gc.attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
    if left != 0 {
        screen_write_box_border_set(lines, CELL_URD, &raw mut gc);
    } else {
        screen_write_box_border_set(lines, CELL_LR, &raw mut gc);
    }
    screen_write_cell(ctx, &raw mut gc);
    screen_write_box_border_set(lines, CELL_LR, &raw mut gc);
    i = 1 as u_int;
    while i < nx.wrapping_sub(1 as u_int) {
        screen_write_cell(ctx, &raw mut gc);
        i = i.wrapping_add(1);
    }
    if right != 0 {
        screen_write_box_border_set(lines, CELL_ULD, &raw mut gc);
    } else {
        screen_write_box_border_set(lines, CELL_LR, &raw mut gc);
    }
    screen_write_cell(ctx, &raw mut gc);
    screen_write_set_cursor(ctx, cx as ::core::ffi::c_int, cy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_vline(
    mut ctx: *mut screen_write_ctx,
    mut ny: u_int,
    mut top: ::core::ffi::c_int,
    mut bottom: ::core::ffi::c_int,
    mut gcp: *const grid_cell,
) {
    let mut s: *mut screen = (*ctx).s;
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
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    let mut i: u_int = 0;
    cx = (*s).cx;
    cy = (*s).cy;
    if !gcp.is_null() {
        memcpy(
            &raw mut gc as *mut ::core::ffi::c_void,
            gcp as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    } else {
        memcpy(
            &raw mut gc as *mut ::core::ffi::c_void,
            &raw const grid_default_cell as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    }
    gc.attr = (gc.attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
    screen_write_putc(
        ctx,
        &raw mut gc,
        (if top != 0 { 'w' as i32 } else { 'x' as i32 }) as u_char,
    );
    i = 1 as u_int;
    while i < ny.wrapping_sub(1 as u_int) {
        screen_write_set_cursor(
            ctx,
            cx as ::core::ffi::c_int,
            cy.wrapping_add(i) as ::core::ffi::c_int,
        );
        screen_write_putc(ctx, &raw mut gc, 'x' as i32 as u_char);
        i = i.wrapping_add(1);
    }
    screen_write_set_cursor(
        ctx,
        cx as ::core::ffi::c_int,
        cy.wrapping_add(ny).wrapping_sub(1 as u_int) as ::core::ffi::c_int,
    );
    screen_write_putc(
        ctx,
        &raw mut gc,
        (if bottom != 0 { 'v' as i32 } else { 'x' as i32 }) as u_char,
    );
    screen_write_set_cursor(ctx, cx as ::core::ffi::c_int, cy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_menu(
    mut ctx: *mut screen_write_ctx,
    mut menu: *mut menu,
    mut choice: ::core::ffi::c_int,
    mut lines: box_lines,
    mut menu_gc: *const grid_cell,
    mut border_gc: *const grid_cell,
    mut choice_gc: *const grid_cell,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut default_gc: grid_cell = grid_cell {
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
    let mut gc: *const grid_cell = &raw mut default_gc;
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut width: u_int = (*menu).width;
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    cx = (*s).cx;
    cy = (*s).cy;
    memcpy(
        &raw mut default_gc as *mut ::core::ffi::c_void,
        menu_gc as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    screen_write_box(
        ctx,
        (*menu).width.wrapping_add(4 as u_int),
        (*menu).count.wrapping_add(2 as u_int),
        lines,
        border_gc,
        ((*menu).title).as_ptr().cast_mut(),
    );
    i = 0 as u_int;
    while i < (*menu).count {
        name = (*(*menu).items.as_mut_ptr().offset(i as isize)).name;
        if name.is_null() {
            screen_write_cursormove(
                ctx,
                cx as ::core::ffi::c_int,
                cy.wrapping_add(1 as u_int).wrapping_add(i) as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            screen_write_hline(
                ctx,
                width.wrapping_add(4 as u_int),
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
                lines,
                border_gc,
            );
        } else {
            if choice >= 0 as ::core::ffi::c_int
                && i == choice as u_int
                && *name as ::core::ffi::c_int != '-' as i32
            {
                gc = choice_gc;
            }
            screen_write_cursormove(
                ctx,
                cx.wrapping_add(1 as u_int) as ::core::ffi::c_int,
                cy.wrapping_add(1 as u_int).wrapping_add(i) as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            j = 0 as u_int;
            while j < width.wrapping_add(2 as u_int) {
                screen_write_putc(ctx, gc, ' ' as i32 as u_char);
                j = j.wrapping_add(1);
            }
            screen_write_cursormove(
                ctx,
                cx.wrapping_add(2 as u_int) as ::core::ffi::c_int,
                cy.wrapping_add(1 as u_int).wrapping_add(i) as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            if *name as ::core::ffi::c_int == '-' as i32 {
                default_gc.attr =
                    (default_gc.attr as ::core::ffi::c_int | GRID_ATTR_DIM) as u_short;
                format_draw(
                    ctx,
                    gc,
                    width,
                    name.offset(1 as ::core::ffi::c_int as isize),
                    ::core::ptr::null_mut::<style_ranges>(),
                    0 as ::core::ffi::c_int,
                );
                default_gc.attr =
                    (default_gc.attr as ::core::ffi::c_int & !GRID_ATTR_DIM) as u_short;
            } else {
                format_draw(
                    ctx,
                    gc,
                    width,
                    name,
                    ::core::ptr::null_mut::<style_ranges>(),
                    0 as ::core::ffi::c_int,
                );
                gc = &raw mut default_gc;
            }
        }
        i = i.wrapping_add(1);
    }
    screen_write_set_cursor(ctx, cx as ::core::ffi::c_int, cy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_box(
    mut ctx: *mut screen_write_ctx,
    mut nx: u_int,
    mut ny: u_int,
    mut lines: box_lines,
    mut gcp: *const grid_cell,
    mut title: *const ::core::ffi::c_char,
) {
    let mut s: *mut screen = (*ctx).s;
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
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    let mut i: u_int = 0;
    cx = (*s).cx;
    cy = (*s).cy;
    if !gcp.is_null() {
        memcpy(
            &raw mut gc as *mut ::core::ffi::c_void,
            gcp as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    } else {
        memcpy(
            &raw mut gc as *mut ::core::ffi::c_void,
            &raw const grid_default_cell as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    }
    gc.attr = (gc.attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
    gc.flags = (gc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
    screen_write_box_border_set(lines, CELL_RD, &raw mut gc);
    screen_write_cell(ctx, &raw mut gc);
    screen_write_box_border_set(lines, CELL_LR, &raw mut gc);
    i = 1 as u_int;
    while i < nx.wrapping_sub(1 as u_int) {
        screen_write_cell(ctx, &raw mut gc);
        i = i.wrapping_add(1);
    }
    screen_write_box_border_set(lines, CELL_LD, &raw mut gc);
    screen_write_cell(ctx, &raw mut gc);
    screen_write_set_cursor(
        ctx,
        cx as ::core::ffi::c_int,
        cy.wrapping_add(ny).wrapping_sub(1 as u_int) as ::core::ffi::c_int,
    );
    screen_write_box_border_set(lines, CELL_RU, &raw mut gc);
    screen_write_cell(ctx, &raw mut gc);
    screen_write_box_border_set(lines, CELL_LR, &raw mut gc);
    i = 1 as u_int;
    while i < nx.wrapping_sub(1 as u_int) {
        screen_write_cell(ctx, &raw mut gc);
        i = i.wrapping_add(1);
    }
    screen_write_box_border_set(lines, CELL_LU, &raw mut gc);
    screen_write_cell(ctx, &raw mut gc);
    screen_write_box_border_set(lines, CELL_UD, &raw mut gc);
    i = 1 as u_int;
    while i < ny.wrapping_sub(1 as u_int) {
        screen_write_set_cursor(
            ctx,
            cx as ::core::ffi::c_int,
            cy.wrapping_add(i) as ::core::ffi::c_int,
        );
        screen_write_cell(ctx, &raw mut gc);
        screen_write_set_cursor(
            ctx,
            cx.wrapping_add(nx).wrapping_sub(1 as u_int) as ::core::ffi::c_int,
            cy.wrapping_add(i) as ::core::ffi::c_int,
        );
        screen_write_cell(ctx, &raw mut gc);
        i = i.wrapping_add(1);
    }
    if !title.is_null() {
        gc.attr = (gc.attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
        screen_write_cursormove(
            ctx,
            cx.wrapping_add(2 as u_int) as ::core::ffi::c_int,
            cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        format_draw(
            ctx,
            &raw mut gc,
            nx.wrapping_sub(4 as u_int),
            title,
            ::core::ptr::null_mut::<style_ranges>(),
            0 as ::core::ffi::c_int,
        );
    }
    screen_write_set_cursor(ctx, cx as ::core::ffi::c_int, cy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_preview(
    mut ctx: *mut screen_write_ctx,
    mut src: *mut screen,
    mut nx: u_int,
    mut ny: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
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
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    cx = (*s).cx;
    cy = (*s).cy;
    if (*src).mode & MODE_CURSOR != 0 {
        px = (*src).cx;
        if px < nx.wrapping_div(3 as u_int) {
            px = 0 as u_int;
        } else {
            px = px.wrapping_sub(nx.wrapping_div(3 as u_int));
        }
        if px.wrapping_add(nx) > (*(*src).grid).sx {
            if nx > (*(*src).grid).sx {
                px = 0 as u_int;
            } else {
                px = (*(*src).grid).sx.wrapping_sub(nx);
            }
        }
        py = (*src).cy;
        if py < ny.wrapping_div(3 as u_int) {
            py = 0 as u_int;
        } else {
            py = py.wrapping_sub(ny.wrapping_div(3 as u_int));
        }
        if py.wrapping_add(ny) > (*(*src).grid).sy {
            if ny > (*(*src).grid).sy {
                py = 0 as u_int;
            } else {
                py = (*(*src).grid).sy.wrapping_sub(ny);
            }
        }
    } else {
        px = 0 as u_int;
        py = 0 as u_int;
    }
    screen_write_fast_copy(ctx, src, px, (*(*src).grid).hsize.wrapping_add(py), nx, ny);
    if (*src).mode & MODE_CURSOR != 0 {
        grid_view_get_cell((*src).grid, (*src).cx, (*src).cy, &raw mut gc);
        gc.attr = (gc.attr as ::core::ffi::c_int | GRID_ATTR_REVERSE) as u_short;
        screen_write_set_cursor(
            ctx,
            cx.wrapping_add((*src).cx.wrapping_sub(px)) as ::core::ffi::c_int,
            cy.wrapping_add((*src).cy.wrapping_sub(py)) as ::core::ffi::c_int,
        );
        screen_write_cell(ctx, &raw mut gc);
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_mode_set(
    mut ctx: *mut screen_write_ctx,
    mut mode: ::core::ffi::c_int,
) {
    let mut s: *mut screen = (*ctx).s;
    (*s).mode |= mode;
    if log_get_level() != 0 as ::core::ffi::c_int {
        log_debug(
            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_write_mode_set\0" as *const u8 as *const ::core::ffi::c_char,
            screen_mode_to_string(mode),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_mode_clear(
    mut ctx: *mut screen_write_ctx,
    mut mode: ::core::ffi::c_int,
) {
    let mut s: *mut screen = (*ctx).s;
    (*s).mode &= !mode;
    if log_get_level() != 0 as ::core::ffi::c_int {
        log_debug(
            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_write_mode_clear\0" as *const u8 as *const ::core::ffi::c_char,
            screen_mode_to_string(mode),
        );
    }
}
unsafe extern "C" fn screen_write_sync_callback(
    _fd: ::core::ffi::c_int,
    _events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut wp: *mut window_pane = arg as *mut window_pane;
    log_debug(
        b"%s: %%%u sync timer expired\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_sync_callback\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
    );
    event_del(&raw mut (*wp).sync_timer);
    if (*wp).base.mode & MODE_SYNC != 0 {
        (*wp).base.mode &= !MODE_SYNC;
        screen_write_flush_dirty(wp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_start_sync(mut wp: *mut window_pane) {
    let mut tv: timeval = timeval {
        tv_sec: 1 as __time_t,
        tv_usec: 0 as __suseconds_t,
    };
    if wp.is_null() {
        return;
    }
    (*wp).base.mode |= MODE_SYNC;
    if event_initialized(&(*wp).sync_timer) == 0 {
        event_set(
            &raw mut (*wp).sync_timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            Some(
                screen_write_sync_callback
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            wp as *mut ::core::ffi::c_void,
        );
    }
    event_add(&raw mut (*wp).sync_timer, &raw mut tv);
    log_debug(
        b"%s: %%%u started sync mode\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_start_sync\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
    );
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_stop_sync(mut wp: *mut window_pane) {
    if wp.is_null() || !(*wp).base.mode & MODE_SYNC != 0 {
        return;
    }
    if event_initialized(&(*wp).sync_timer) != 0 {
        event_del(&raw mut (*wp).sync_timer);
    }
    (*wp).base.mode &= !MODE_SYNC;
    screen_write_flush_dirty(wp);
    log_debug(
        b"%s: %%%u stopped sync mode\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_stop_sync\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
    );
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_end_sync(mut ctx: *mut screen_write_ctx) {
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    if wp.is_null() {
        return;
    }
    if (*wp).base.mode & MODE_SYNC != 0 {
        screen_write_collect_flush(
            ctx,
            0 as ::core::ffi::c_int,
            b"screen_write_end_sync\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    screen_write_stop_sync(wp);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_cursorup(mut ctx: *mut screen_write_ctx, mut ny: u_int) {
    let mut s: *mut screen = (*ctx).s;
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    if ny == 0 as u_int {
        ny = 1 as u_int;
    }
    if cy < (*s).rupper {
        if ny > cy {
            ny = cy;
        }
    } else if ny > cy.wrapping_sub((*s).rupper) {
        ny = cy.wrapping_sub((*s).rupper);
    }
    if cx == (*(*s).grid).sx {
        cx = cx.wrapping_sub(1);
    }
    cy = cy.wrapping_sub(ny);
    screen_write_set_cursor(ctx, cx as ::core::ffi::c_int, cy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_cursordown(mut ctx: *mut screen_write_ctx, mut ny: u_int) {
    let mut s: *mut screen = (*ctx).s;
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    if ny == 0 as u_int {
        ny = 1 as u_int;
    }
    if cy > (*s).rlower {
        if ny > (*(*s).grid).sy.wrapping_sub(1 as u_int).wrapping_sub(cy) {
            ny = (*(*s).grid).sy.wrapping_sub(1 as u_int).wrapping_sub(cy);
        }
    } else if ny > (*s).rlower.wrapping_sub(cy) {
        ny = (*s).rlower.wrapping_sub(cy);
    }
    if cx == (*(*s).grid).sx {
        cx = cx.wrapping_sub(1);
    } else if ny == 0 as u_int {
        return;
    }
    cy = cy.wrapping_add(ny);
    screen_write_set_cursor(ctx, cx as ::core::ffi::c_int, cy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_cursorright(mut ctx: *mut screen_write_ctx, mut nx: u_int) {
    let mut s: *mut screen = (*ctx).s;
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    if nx == 0 as u_int {
        nx = 1 as u_int;
    }
    if nx > (*(*s).grid).sx.wrapping_sub(1 as u_int).wrapping_sub(cx) {
        nx = (*(*s).grid).sx.wrapping_sub(1 as u_int).wrapping_sub(cx);
    }
    if nx == 0 as u_int {
        return;
    }
    cx = cx.wrapping_add(nx);
    screen_write_set_cursor(ctx, cx as ::core::ffi::c_int, cy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_cursorleft(mut ctx: *mut screen_write_ctx, mut nx: u_int) {
    let mut s: *mut screen = (*ctx).s;
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    if nx == 0 as u_int {
        nx = 1 as u_int;
    }
    if nx > cx {
        nx = cx;
    }
    if nx == 0 as u_int {
        return;
    }
    cx = cx.wrapping_sub(nx);
    screen_write_set_cursor(ctx, cx as ::core::ffi::c_int, cy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_backspace(mut ctx: *mut screen_write_ctx) {
    let mut s: *mut screen = (*ctx).s;
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    if cx == 0 as u_int {
        if cy == 0 as u_int {
            return;
        }
        gl = grid_get_line(
            (*s).grid,
            (*(*s).grid).hsize.wrapping_add(cy).wrapping_sub(1 as u_int),
        );
        if (*gl).flags as ::core::ffi::c_int & GRID_LINE_WRAPPED != 0 {
            cy = cy.wrapping_sub(1);
            cx = (*(*s).grid).sx.wrapping_sub(1 as u_int);
        }
    } else {
        cx = cx.wrapping_sub(1);
    }
    screen_write_set_cursor(ctx, cx as ::core::ffi::c_int, cy as ::core::ffi::c_int);
}
unsafe extern "C" fn screen_write_cell_is_single(mut gc: *const grid_cell) -> ::core::ffi::c_int {
    if (*gc).data.width as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if (*gc).data.size as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if (*(&raw const (*gc).data.data as *const u_char) as ::core::ffi::c_int)
        < 0x20 as ::core::ffi::c_int
        || *(&raw const (*gc).data.data as *const u_char) as ::core::ffi::c_int
            == 0x7f as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_CLEARED != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn screen_write_redraw_line(
    mut ctx: *mut screen_write_ctx,
    mut ttyctx: *mut tty_ctx,
    mut yy: u_int,
) {
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    let mut s: *mut screen = (*ctx).s;
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
    let mut ngc: grid_cell = grid_cell {
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
    let mut sx: u_int = (*(*s).grid).sx;
    let mut cx: u_int = 0;
    let mut i: u_int = 0;
    let mut xoff: ::core::ffi::c_int = (*wp).xoff;
    let mut yoff: ::core::ffi::c_int = (*wp).yoff;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    r = window_visible_ranges(
        wp,
        xoff,
        (yoff as u_int).wrapping_add(yy) as ::core::ffi::c_int,
        sx,
        ::core::ptr::null_mut::<visible_ranges>(),
    );
    i = 0 as u_int;
    while i < (*r).used {
        ri = (*r).ranges.offset(i as isize) as *mut visible_range;
        if !((*ri).nx == 0 as u_int) {
            cx = (*ri).px.wrapping_sub(xoff as u_int);
            if !(cx >= sx) {
                if cx.wrapping_add((*ri).nx) > sx {
                    (*ttyctx).c2rust_unnamed.n = sx.wrapping_sub(cx);
                } else {
                    (*ttyctx).c2rust_unnamed.n = (*ri).nx;
                }
                if !((*ttyctx).c2rust_unnamed.n == 0 as u_int) {
                    (*ttyctx).ocx = cx;
                    (*ttyctx).ocy = yy;
                    if (*ttyctx).c2rust_unnamed.n != 1 as u_int {
                        tty_write(
                            Some(
                                tty_cmd_redrawline
                                    as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> (),
                            ),
                            ttyctx,
                        );
                    } else {
                        grid_view_get_cell((*s).grid, cx, yy, &raw mut gc);
                        if screen_write_cell_is_single(&raw mut gc) == 0 {
                            tty_write(
                                Some(
                                    tty_cmd_redrawline
                                        as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> (),
                                ),
                                ttyctx,
                            );
                        } else {
                            if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_SELECTED != 0 {
                                (*ttyctx).cell = &raw mut gc;
                            } else {
                                screen_select_cell(s, &raw mut ngc, &raw mut gc);
                                (*ttyctx).cell = &raw mut ngc;
                            }
                            tty_write(
                                Some(
                                    tty_cmd_cell
                                        as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> (),
                                ),
                                ttyctx,
                            );
                        }
                    }
                }
            }
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn screen_write_flush_dirty(mut wp: *mut window_pane) {
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut s: *mut screen = &raw mut (*wp).base;
    let mut y: u_int = 0;
    let mut sy: u_int = (*(*s).grid).sy;
    let mut lines: u_int = 0 as u_int;
    if (*wp).sync_dirty.is_null() {
        return;
    }
    screen_write_start_pane(&raw mut ctx, wp, s);
    screen_write_initctx(
        &raw mut ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    y = 0 as u_int;
    while y < sy {
        if *(*wp)
            .sync_dirty
            .offset((y >> 3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << (y & 0x7 as u_int)
            != 0
        {
            screen_write_redraw_line(&raw mut ctx, &raw mut ttyctx, y);
            lines = lines.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
    log_debug(
        b"%s: %%%u had %u dirty lines\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_flush_dirty\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
        lines,
    );
    screen_write_stop(&raw mut ctx);
    screen_write_clear_dirty(wp);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_clear_dirty(mut wp: *mut window_pane) {
    if !wp.is_null() && !(*wp).sync_dirty.is_null() {
        let bytes =
            ((*wp).sync_dirty_size.wrapping_add(7 as u_int) >> 3 as ::core::ffi::c_int) as usize;
        let dirty = ::core::ptr::slice_from_raw_parts_mut((*wp).sync_dirty, bytes);
        drop(Box::from_raw(dirty));
        (*wp).sync_dirty = ::core::ptr::null_mut::<bitstr_t>();
        (*wp).sync_dirty_size = 0 as u_int;
    }
}
unsafe extern "C" fn screen_write_redraw_pane(
    mut ctx: *mut screen_write_ctx,
    mut ttyctx: *mut tty_ctx,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut yy: u_int = 0;
    yy = 0 as u_int;
    while yy < (*(*s).grid).sy {
        screen_write_redraw_line(ctx, ttyctx, yy);
        yy = yy.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_alignmenttest(mut ctx: *mut screen_write_ctx) {
    let mut s: *mut screen = (*ctx).s;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
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
    let mut xx: u_int = 0;
    let mut yy: u_int = 0;
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    utf8_set(&raw mut gc.data, 'E' as i32 as u_char);
    yy = 0 as u_int;
    while yy < (*(*s).grid).sy {
        xx = 0 as u_int;
        while xx < (*(*s).grid).sx {
            grid_view_set_cell((*s).grid, xx, yy, &raw mut gc);
            xx = xx.wrapping_add(1);
        }
        yy = yy.wrapping_add(1);
    }
    screen_write_set_cursor(ctx, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    (*s).rupper = 0 as u_int;
    (*s).rlower = (*(*s).grid).sy.wrapping_sub(1 as u_int);
    screen_write_collect_clear(ctx, 0 as u_int, (*(*s).grid).sy.wrapping_sub(1 as u_int));
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    if screen_write_should_draw_lines(ctx, 0 as u_int, (*(*s).grid).sy) == 0 {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 || (*ctx).wp.is_null() {
        tty_write(
            Some(tty_cmd_alignmenttest as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
        return;
    }
    screen_write_redraw_pane(ctx, &raw mut ttyctx);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_insertcharacter(
    mut ctx: *mut screen_write_ctx,
    mut nx: u_int,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    if nx == 0 as u_int {
        nx = 1 as u_int;
    }
    if nx > (*(*s).grid).sx.wrapping_sub((*s).cx) {
        nx = (*(*s).grid).sx.wrapping_sub((*s).cx);
    }
    if nx == 0 as u_int {
        return;
    }
    if (*s).cx > (*(*s).grid).sx.wrapping_sub(1 as u_int) {
        return;
    }
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ttyctx.bg = bg;
    grid_view_insert_cells((*s).grid, (*s).cx, (*s).cy, nx, bg);
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_insertcharacter\0" as *const u8 as *const ::core::ffi::c_char,
    );
    ttyctx.c2rust_unnamed.n = nx;
    if screen_write_should_draw_line(ctx, (*s).cy) == 0 {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 || (*ctx).wp.is_null() {
        tty_write(
            Some(tty_cmd_insertcharacter as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
        return;
    }
    screen_write_redraw_line(ctx, &raw mut ttyctx, (*s).cy);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_deletecharacter(
    mut ctx: *mut screen_write_ctx,
    mut nx: u_int,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    if nx == 0 as u_int {
        nx = 1 as u_int;
    }
    if nx > (*(*s).grid).sx.wrapping_sub((*s).cx) {
        nx = (*(*s).grid).sx.wrapping_sub((*s).cx);
    }
    if nx == 0 as u_int {
        return;
    }
    if (*s).cx > (*(*s).grid).sx.wrapping_sub(1 as u_int) {
        return;
    }
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ttyctx.bg = bg;
    grid_view_delete_cells((*s).grid, (*s).cx, (*s).cy, nx, bg);
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_deletecharacter\0" as *const u8 as *const ::core::ffi::c_char,
    );
    ttyctx.c2rust_unnamed.n = nx;
    if screen_write_should_draw_line(ctx, (*s).cy) == 0 {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 || (*ctx).wp.is_null() {
        tty_write(
            Some(tty_cmd_deletecharacter as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
        return;
    }
    screen_write_redraw_line(ctx, &raw mut ttyctx, (*s).cy);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_clearcharacter(
    mut ctx: *mut screen_write_ctx,
    mut nx: u_int,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    if nx == 0 as u_int {
        nx = 1 as u_int;
    }
    if nx > (*(*s).grid).sx.wrapping_sub((*s).cx) {
        nx = (*(*s).grid).sx.wrapping_sub((*s).cx);
    }
    if nx == 0 as u_int {
        return;
    }
    if (*s).cx > (*(*s).grid).sx.wrapping_sub(1 as u_int) {
        return;
    }
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ttyctx.bg = bg;
    grid_view_clear((*s).grid, (*s).cx, (*s).cy, nx, 1 as u_int, bg);
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_clearcharacter\0" as *const u8 as *const ::core::ffi::c_char,
    );
    ttyctx.c2rust_unnamed.n = nx;
    if screen_write_should_draw_line(ctx, (*s).cy) == 0 {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 || (*ctx).wp.is_null() {
        tty_write(
            Some(tty_cmd_clearcharacter as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
        return;
    }
    screen_write_redraw_line(ctx, &raw mut ttyctx, (*s).cy);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_insertline(
    mut ctx: *mut screen_write_ctx,
    mut ny: u_int,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut gd: *mut grid = (*s).grid;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut sy: u_int = (*(*s).grid).sy;
    if ny == 0 as u_int {
        ny = 1 as u_int;
    }
    if (*s).cy < (*s).rupper || (*s).cy > (*s).rlower {
        if ny > sy.wrapping_sub((*s).cy) {
            ny = sy.wrapping_sub((*s).cy);
        }
        if ny == 0 as u_int {
            return;
        }
        screen_write_initctx(
            ctx,
            &raw mut ttyctx,
            1 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        );
        ttyctx.bg = bg;
        grid_view_insert_lines(gd, (*s).cy, ny, bg);
        screen_write_collect_flush(
            ctx,
            0 as ::core::ffi::c_int,
            b"screen_write_insertline\0" as *const u8 as *const ::core::ffi::c_char,
        );
        ttyctx.c2rust_unnamed.n = ny;
        if screen_write_should_draw_lines(ctx, (*s).cy, sy.wrapping_sub((*s).cy)) == 0 {
            return;
        }
        if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 || (*ctx).wp.is_null() {
            tty_write(
                Some(tty_cmd_insertline as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
                &raw mut ttyctx,
            );
            return;
        }
        screen_write_redraw_pane(ctx, &raw mut ttyctx);
        return;
    }
    if ny > (*s).rlower.wrapping_add(1 as u_int).wrapping_sub((*s).cy) {
        ny = (*s).rlower.wrapping_add(1 as u_int).wrapping_sub((*s).cy);
    }
    if ny == 0 as u_int {
        return;
    }
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ttyctx.bg = bg;
    if (*s).cy < (*s).rupper || (*s).cy > (*s).rlower {
        grid_view_insert_lines(gd, (*s).cy, ny, bg);
    } else {
        grid_view_insert_lines_region(gd, (*s).rlower, (*s).cy, ny, bg);
    }
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_insertline\0" as *const u8 as *const ::core::ffi::c_char,
    );
    ttyctx.c2rust_unnamed.n = ny;
    if screen_write_should_draw_lines(
        ctx,
        (*s).cy,
        (*s).rlower.wrapping_add(1 as u_int).wrapping_sub((*s).cy),
    ) == 0
    {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 || (*ctx).wp.is_null() {
        tty_write(
            Some(tty_cmd_insertline as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
        return;
    }
    screen_write_redraw_pane(ctx, &raw mut ttyctx);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_deleteline(
    mut ctx: *mut screen_write_ctx,
    mut ny: u_int,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut gd: *mut grid = (*s).grid;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut sy: u_int = (*(*s).grid).sy;
    let mut ry: u_int = 0;
    if ny == 0 as u_int {
        ny = 1 as u_int;
    }
    if (*s).cy < (*s).rupper || (*s).cy > (*s).rlower {
        if ny > sy.wrapping_sub((*s).cy) {
            ny = sy.wrapping_sub((*s).cy);
        }
        if ny == 0 as u_int {
            return;
        }
        screen_write_initctx(
            ctx,
            &raw mut ttyctx,
            1 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        );
        ttyctx.bg = bg;
        grid_view_delete_lines(gd, (*s).cy, ny, bg);
        screen_write_collect_flush(
            ctx,
            0 as ::core::ffi::c_int,
            b"screen_write_deleteline\0" as *const u8 as *const ::core::ffi::c_char,
        );
        ttyctx.c2rust_unnamed.n = ny;
        ry = (*s)
            .rlower
            .wrapping_add(1 as u_int)
            .wrapping_sub((*s).rupper);
        if screen_write_should_draw_lines(ctx, (*s).rupper, ry) == 0 {
            return;
        }
        if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 || (*ctx).wp.is_null() {
            tty_write(
                Some(tty_cmd_deleteline as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
                &raw mut ttyctx,
            );
            return;
        }
        screen_write_redraw_pane(ctx, &raw mut ttyctx);
        return;
    }
    ry = (*s).rlower.wrapping_add(1 as u_int).wrapping_sub((*s).cy);
    if ny > ry {
        ny = ry;
    }
    if ny == 0 as u_int {
        return;
    }
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ttyctx.bg = bg;
    if (*s).cy < (*s).rupper || (*s).cy > (*s).rlower {
        grid_view_delete_lines(gd, (*s).cy, ny, bg);
    } else {
        grid_view_delete_lines_region(gd, (*s).rlower, (*s).cy, ny, bg);
    }
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_deleteline\0" as *const u8 as *const ::core::ffi::c_char,
    );
    ttyctx.c2rust_unnamed.n = ny;
    if screen_write_should_draw_lines(ctx, (*s).cy, ry) == 0 {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 || (*ctx).wp.is_null() {
        tty_write(
            Some(tty_cmd_deleteline as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
        return;
    }
    screen_write_redraw_pane(ctx, &raw mut ttyctx);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_clearline(mut ctx: *mut screen_write_ctx, mut bg: u_int) {
    let mut s: *mut screen = (*ctx).s;
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut sx: u_int = (*(*s).grid).sx;
    let mut ci: *mut screen_write_citem = screen_write_current_item(ctx);
    let mut od: osc133_data = osc133_data {
        prompt_col: 0,
        cmd_col: 0,
        out_start_col: 0,
        out_end_col: 0,
        exit_status: 0,
    };
    let mut flags: u_int = 0;
    gl = grid_get_line((*s).grid, (*(*s).grid).hsize.wrapping_add((*s).cy));
    if (*gl).cellsize as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        && (bg == 8 as u_int || bg == 9 as u_int)
    {
        return;
    }
    flags = ((*gl).flags as ::core::ffi::c_int & GRID_LINE_OSC133_FLAGS) as u_int;
    memcpy(
        &raw mut od as *mut ::core::ffi::c_void,
        &raw mut (*gl).osc133_data as *const ::core::ffi::c_void,
        ::core::mem::size_of::<osc133_data>() as size_t,
    );
    grid_view_clear((*s).grid, 0 as u_int, (*s).cy, sx, 1 as u_int, bg);
    gl = grid_get_line((*s).grid, (*(*s).grid).hsize.wrapping_add((*s).cy));
    (*gl).flags = ((*gl).flags as u_int | flags) as u_short;
    memcpy(
        &raw mut (*gl).osc133_data as *mut ::core::ffi::c_void,
        &raw mut od as *const ::core::ffi::c_void,
        ::core::mem::size_of::<osc133_data>() as size_t,
    );
    screen_write_collect_clear(ctx, (*s).cy, 1 as u_int);
    (*ci).x = 0 as u_int;
    (*ci).used = sx;
    (*ci).type_0 = CLEAR;
    (*ci).bg = bg;
    (*(*s).write_list.offset((*s).cy as isize))
        .items
        .push_back((*ctx).item.take().expect("active screen write context"));
    (*ctx).item = Some(screen_write_get_citem());
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_clearendofline(
    mut ctx: *mut screen_write_ctx,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut sx: u_int = (*(*s).grid).sx;
    let mut ci: *mut screen_write_citem = screen_write_current_item(ctx);
    if (*s).cx == 0 as u_int {
        screen_write_clearline(ctx, bg);
        return;
    }
    gl = grid_get_line((*s).grid, (*(*s).grid).hsize.wrapping_add((*s).cy));
    if (*s).cx > sx.wrapping_sub(1 as u_int)
        || (*s).cx >= (*gl).cellsize as u_int && (bg == 8 as u_int || bg == 9 as u_int)
    {
        return;
    }
    grid_view_clear(
        (*s).grid,
        (*s).cx,
        (*s).cy,
        sx.wrapping_sub((*s).cx),
        1 as u_int,
        bg,
    );
    (*ci).x = (*s).cx;
    (*ci).used = sx.wrapping_sub((*s).cx);
    (*ci).type_0 = CLEAR;
    (*ci).bg = bg;
    screen_write_collect_insert(ctx, ci);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_clearstartofline(
    mut ctx: *mut screen_write_ctx,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut sx: u_int = (*(*s).grid).sx;
    let mut ci: *mut screen_write_citem = screen_write_current_item(ctx);
    if (*s).cx >= sx.wrapping_sub(1 as u_int) {
        screen_write_clearline(ctx, bg);
        return;
    }
    if (*s).cx > sx.wrapping_sub(1 as u_int) {
        grid_view_clear((*s).grid, 0 as u_int, (*s).cy, sx, 1 as u_int, bg);
    } else {
        grid_view_clear(
            (*s).grid,
            0 as u_int,
            (*s).cy,
            (*s).cx.wrapping_add(1 as u_int),
            1 as u_int,
            bg,
        );
    }
    (*ci).x = 0 as u_int;
    (*ci).used = (*s).cx.wrapping_add(1 as u_int);
    (*ci).type_0 = CLEAR;
    (*ci).bg = bg;
    screen_write_collect_insert(ctx, ci);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_cursormove(
    mut ctx: *mut screen_write_ctx,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
    mut origin: ::core::ffi::c_int,
) {
    let mut s: *mut screen = (*ctx).s;
    if origin != 0 && py != -(1 as ::core::ffi::c_int) && (*s).mode & MODE_ORIGIN != 0 {
        if py as u_int > (*s).rlower.wrapping_sub((*s).rupper) {
            py = (*s).rlower as ::core::ffi::c_int;
        } else {
            py =
                (py as u_int).wrapping_add((*s).rupper) as ::core::ffi::c_int as ::core::ffi::c_int;
        }
    }
    if px != -(1 as ::core::ffi::c_int) && px as u_int > (*(*s).grid).sx.wrapping_sub(1 as u_int) {
        px = (*(*s).grid).sx.wrapping_sub(1 as u_int) as ::core::ffi::c_int;
    }
    if py != -(1 as ::core::ffi::c_int) && py as u_int > (*(*s).grid).sy.wrapping_sub(1 as u_int) {
        py = (*(*s).grid).sy.wrapping_sub(1 as u_int) as ::core::ffi::c_int;
    }
    log_debug(
        b"%s: from %u,%u to %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_cursormove\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).cx,
        (*s).cy,
        px,
        py,
    );
    screen_write_set_cursor(ctx, px, py);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_reverseindex(mut ctx: *mut screen_write_ctx, mut bg: u_int) {
    let mut s: *mut screen = (*ctx).s;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut ry: u_int = 0;
    if (*s).cy != (*s).rupper {
        if (*s).cy > 0 as u_int {
            screen_write_set_cursor(
                ctx,
                -(1 as ::core::ffi::c_int),
                (*s).cy.wrapping_sub(1 as u_int) as ::core::ffi::c_int,
            );
        }
        return;
    }
    grid_view_scroll_region_down((*s).grid, (*s).rupper, (*s).rlower, bg);
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_reverseindex\0" as *const u8 as *const ::core::ffi::c_char,
    );
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ttyctx.bg = bg;
    ry = (*s)
        .rlower
        .wrapping_add(1 as u_int)
        .wrapping_sub((*s).rupper);
    if screen_write_should_draw_lines(ctx, (*s).rupper, ry) == 0 {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 || (*ctx).wp.is_null() {
        tty_write(
            Some(tty_cmd_reverseindex as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
        return;
    }
    screen_write_redraw_pane(ctx, &raw mut ttyctx);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_scrollregion(
    mut ctx: *mut screen_write_ctx,
    mut rupper: u_int,
    mut rlower: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    if rupper > (*(*s).grid).sy.wrapping_sub(1 as u_int) {
        rupper = (*(*s).grid).sy.wrapping_sub(1 as u_int);
    }
    if rlower > (*(*s).grid).sy.wrapping_sub(1 as u_int) {
        rlower = (*(*s).grid).sy.wrapping_sub(1 as u_int);
    }
    if rupper >= rlower {
        return;
    }
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_scrollregion\0" as *const u8 as *const ::core::ffi::c_char,
    );
    screen_write_set_cursor(ctx, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    (*s).rupper = rupper;
    (*s).rlower = rlower;
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_linefeed(
    mut ctx: *mut screen_write_ctx,
    mut wrapped: ::core::ffi::c_int,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut gd: *mut grid = (*s).grid;
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut rupper: u_int = (*s).rupper;
    let mut rlower: u_int = (*s).rlower;
    gl = grid_get_line(gd, (*gd).hsize.wrapping_add((*s).cy));
    if wrapped != 0 {
        (*gl).flags = ((*gl).flags as ::core::ffi::c_int | GRID_LINE_WRAPPED) as u_short;
    }
    log_debug(
        b"%s: at %u,%u (region %u-%u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_linefeed\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).cx,
        (*s).cy,
        rupper,
        rlower,
    );
    if bg != (*ctx).bg {
        screen_write_collect_flush(
            ctx,
            1 as ::core::ffi::c_int,
            b"screen_write_linefeed\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*ctx).bg = bg;
    }
    if (*s).cy != (*s).rlower {
        if (*s).cy < (*(*s).grid).sy.wrapping_sub(1 as u_int) {
            screen_write_set_cursor(
                ctx,
                -(1 as ::core::ffi::c_int),
                (*s).cy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
            );
        }
        return;
    }
    grid_view_scroll_region_up(gd, (*s).rupper, (*s).rlower, bg);
    screen_write_collect_scroll(ctx, bg);
    (*ctx).scrolled = (*ctx).scrolled.wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_scrollup(
    mut ctx: *mut screen_write_ctx,
    mut lines: u_int,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut gd: *mut grid = (*s).grid;
    let mut i: u_int = 0;
    if lines == 0 as u_int {
        lines = 1 as u_int;
    } else if lines
        > (*s)
            .rlower
            .wrapping_sub((*s).rupper)
            .wrapping_add(1 as u_int)
    {
        lines = (*s)
            .rlower
            .wrapping_sub((*s).rupper)
            .wrapping_add(1 as u_int);
    }
    if bg != (*ctx).bg {
        screen_write_collect_flush(
            ctx,
            1 as ::core::ffi::c_int,
            b"screen_write_scrollup\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*ctx).bg = bg;
    }
    i = 0 as u_int;
    while i < lines {
        grid_view_scroll_region_up(gd, (*s).rupper, (*s).rlower, bg);
        screen_write_collect_scroll(ctx, bg);
        i = i.wrapping_add(1);
    }
    (*ctx).scrolled = (*ctx).scrolled.wrapping_add(lines);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_scrolldown(
    mut ctx: *mut screen_write_ctx,
    mut lines: u_int,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut gd: *mut grid = (*s).grid;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut i: u_int = 0;
    let mut ry: u_int = 0;
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ttyctx.bg = bg;
    if lines == 0 as u_int {
        lines = 1 as u_int;
    } else if lines
        > (*s)
            .rlower
            .wrapping_sub((*s).rupper)
            .wrapping_add(1 as u_int)
    {
        lines = (*s)
            .rlower
            .wrapping_sub((*s).rupper)
            .wrapping_add(1 as u_int);
    }
    i = 0 as u_int;
    while i < lines {
        grid_view_scroll_region_down(gd, (*s).rupper, (*s).rlower, bg);
        i = i.wrapping_add(1);
    }
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_scrolldown\0" as *const u8 as *const ::core::ffi::c_char,
    );
    ttyctx.c2rust_unnamed.n = lines;
    ry = (*s)
        .rlower
        .wrapping_add(1 as u_int)
        .wrapping_sub((*s).rupper);
    if screen_write_should_draw_lines(ctx, (*s).rupper, ry) == 0 {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 || (*ctx).wp.is_null() {
        tty_write(
            Some(tty_cmd_scrolldown as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
        return;
    }
    screen_write_redraw_pane(ctx, &raw mut ttyctx);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_carriagereturn(mut ctx: *mut screen_write_ctx) {
    screen_write_set_cursor(ctx, 0 as ::core::ffi::c_int, -(1 as ::core::ffi::c_int));
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_clearendofscreen(
    mut ctx: *mut screen_write_ctx,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut gd: *mut grid = (*s).grid;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut sx: u_int = (*(*s).grid).sx;
    let mut sy: u_int = (*(*s).grid).sy;
    let mut y: u_int = 0;
    let mut i: u_int = 0;
    let mut xoff: u_int = 0;
    let mut yoff: u_int = 0;
    let mut ocx: u_int = 0;
    let mut ocy: u_int = 0;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ttyctx.bg = bg;
    if (*s).cx == 0 as u_int
        && (*s).cy == 0 as u_int
        && (*gd).flags & GRID_HISTORY != 0
        && !(*ctx).wp.is_null()
        && options_get_number(
            (*(*ctx).wp).options,
            b"scroll-on-clear\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
    {
        grid_view_clear_history(gd, bg);
    } else {
        if (*s).cx <= sx.wrapping_sub(1 as u_int) {
            grid_view_clear(
                gd,
                (*s).cx,
                (*s).cy,
                sx.wrapping_sub((*s).cx),
                1 as u_int,
                bg,
            );
        }
        grid_view_clear(
            gd,
            0 as u_int,
            (*s).cy.wrapping_add(1 as u_int),
            sx,
            sy.wrapping_sub((*s).cy.wrapping_add(1 as u_int)),
            bg,
        );
    }
    screen_write_collect_clear(
        ctx,
        (*s).cy.wrapping_add(1 as u_int),
        sy.wrapping_sub((*s).cy.wrapping_add(1 as u_int)),
    );
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_clearendofscreen\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if screen_write_should_draw_lines(ctx, (*s).cy, sy.wrapping_sub((*s).cy)) == 0 {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 {
        tty_write(
            Some(tty_cmd_clearendofscreen as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
        return;
    }
    ocx = (*s).cx;
    ocy = (*s).cy;
    if !(*ctx).wp.is_null() {
        xoff = (*(*ctx).wp).xoff as u_int;
        yoff = (*(*ctx).wp).yoff as u_int;
    } else {
        xoff = 0 as u_int;
        yoff = 0 as u_int;
    }
    if (*s).cx <= sx.wrapping_sub(1 as u_int) {
        r = window_visible_ranges(
            (*ctx).wp as *mut window_pane,
            xoff.wrapping_add((*s).cx) as ::core::ffi::c_int,
            yoff.wrapping_add((*s).cy) as ::core::ffi::c_int,
            sx.wrapping_sub((*s).cx),
            ::core::ptr::null_mut::<visible_ranges>(),
        );
        i = 0 as u_int;
        while i < (*r).used {
            ri = (*r).ranges.offset(i as isize) as *mut visible_range;
            if !((*ri).nx == 0 as u_int) {
                screen_write_collect_insert_clear(ctx, (*ri).px.wrapping_sub(xoff), (*ri).nx, bg);
            }
            i = i.wrapping_add(1);
        }
    }
    y = (*s).cy.wrapping_add(1 as u_int);
    while y < sy {
        screen_write_set_cursor(ctx, 0 as ::core::ffi::c_int, y as ::core::ffi::c_int);
        r = window_visible_ranges(
            (*ctx).wp as *mut window_pane,
            xoff as ::core::ffi::c_int,
            yoff.wrapping_add(y) as ::core::ffi::c_int,
            sx,
            ::core::ptr::null_mut::<visible_ranges>(),
        );
        i = 0 as u_int;
        while i < (*r).used {
            ri = (*r).ranges.offset(i as isize) as *mut visible_range;
            if !((*ri).nx == 0 as u_int) {
                screen_write_collect_insert_clear(ctx, (*ri).px.wrapping_sub(xoff), (*ri).nx, bg);
            }
            i = i.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
    screen_write_set_cursor(ctx, ocx as ::core::ffi::c_int, ocy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_clearstartofscreen(
    mut ctx: *mut screen_write_ctx,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut sx: u_int = (*(*s).grid).sx;
    let mut y: u_int = 0;
    let mut i: u_int = 0;
    let mut xoff: u_int = 0;
    let mut yoff: u_int = 0;
    let mut ocx: u_int = 0;
    let mut ocy: u_int = 0;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ttyctx.bg = bg;
    if (*s).cy > 0 as u_int {
        grid_view_clear((*s).grid, 0 as u_int, 0 as u_int, sx, (*s).cy, bg);
    }
    if (*s).cx > sx.wrapping_sub(1 as u_int) {
        grid_view_clear((*s).grid, 0 as u_int, (*s).cy, sx, 1 as u_int, bg);
    } else {
        grid_view_clear(
            (*s).grid,
            0 as u_int,
            (*s).cy,
            (*s).cx.wrapping_add(1 as u_int),
            1 as u_int,
            bg,
        );
    }
    screen_write_collect_clear(ctx, 0 as u_int, (*s).cy);
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_clearstartofscreen\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if screen_write_should_draw_lines(ctx, 0 as u_int, (*s).cy.wrapping_add(1 as u_int)) == 0 {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 {
        tty_write(
            Some(
                tty_cmd_clearstartofscreen as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> (),
            ),
            &raw mut ttyctx,
        );
        return;
    }
    ocx = (*s).cx;
    ocy = (*s).cy;
    if !(*ctx).wp.is_null() {
        xoff = (*(*ctx).wp).xoff as u_int;
        yoff = (*(*ctx).wp).yoff as u_int;
    } else {
        xoff = 0 as u_int;
        yoff = 0 as u_int;
    }
    y = 0 as u_int;
    while y < (*s).cy {
        screen_write_set_cursor(ctx, 0 as ::core::ffi::c_int, y as ::core::ffi::c_int);
        r = window_visible_ranges(
            (*ctx).wp as *mut window_pane,
            xoff as ::core::ffi::c_int,
            yoff.wrapping_add(y) as ::core::ffi::c_int,
            sx,
            ::core::ptr::null_mut::<visible_ranges>(),
        );
        i = 0 as u_int;
        while i < (*r).used {
            ri = (*r).ranges.offset(i as isize) as *mut visible_range;
            if !((*ri).nx == 0 as u_int) {
                screen_write_collect_insert_clear(ctx, (*ri).px.wrapping_sub(xoff), (*ri).nx, bg);
            }
            i = i.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
    screen_write_set_cursor(ctx, 0 as ::core::ffi::c_int, (*s).cy as ::core::ffi::c_int);
    r = window_visible_ranges(
        (*ctx).wp as *mut window_pane,
        xoff as ::core::ffi::c_int,
        yoff.wrapping_add(ocy) as ::core::ffi::c_int,
        (*s).cx.wrapping_add(1 as u_int),
        ::core::ptr::null_mut::<visible_ranges>(),
    );
    i = 0 as u_int;
    while i < (*r).used {
        ri = (*r).ranges.offset(i as isize) as *mut visible_range;
        if !((*ri).nx == 0 as u_int) {
            screen_write_collect_insert_clear(ctx, (*ri).px.wrapping_sub(xoff), (*ri).nx, bg);
        }
        i = i.wrapping_add(1);
    }
    screen_write_set_cursor(ctx, ocx as ::core::ffi::c_int, ocy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_clearscreen(mut ctx: *mut screen_write_ctx, mut bg: u_int) {
    let mut s: *mut screen = (*ctx).s;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut sx: u_int = (*(*s).grid).sx;
    let mut sy: u_int = (*(*s).grid).sy;
    let mut y: u_int = 0;
    let mut i: u_int = 0;
    let mut xoff: u_int = 0;
    let mut yoff: u_int = 0;
    let mut ocx: u_int = 0;
    let mut ocy: u_int = 0;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ttyctx.bg = bg;
    if (*(*s).grid).flags & GRID_HISTORY != 0
        && !(*ctx).wp.is_null()
        && options_get_number(
            (*(*ctx).wp).options,
            b"scroll-on-clear\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
    {
        grid_view_clear_history((*s).grid, bg);
    } else {
        grid_view_clear((*s).grid, 0 as u_int, 0 as u_int, sx, sy, bg);
    }
    screen_write_collect_clear(ctx, 0 as u_int, sy);
    if screen_write_should_draw_lines(ctx, 0 as u_int, sy) == 0 {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 {
        tty_write(
            Some(tty_cmd_clearscreen as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
        return;
    }
    ocx = (*s).cx;
    ocy = (*s).cy;
    if !(*ctx).wp.is_null() {
        xoff = (*(*ctx).wp).xoff as u_int;
        yoff = (*(*ctx).wp).yoff as u_int;
    } else {
        xoff = 0 as u_int;
        yoff = 0 as u_int;
    }
    y = 0 as u_int;
    while y < sy {
        screen_write_set_cursor(ctx, 0 as ::core::ffi::c_int, y as ::core::ffi::c_int);
        r = window_visible_ranges(
            (*ctx).wp as *mut window_pane,
            xoff as ::core::ffi::c_int,
            yoff.wrapping_add(y) as ::core::ffi::c_int,
            sx,
            ::core::ptr::null_mut::<visible_ranges>(),
        );
        i = 0 as u_int;
        while i < (*r).used {
            ri = (*r).ranges.offset(i as isize) as *mut visible_range;
            if !((*ri).nx == 0 as u_int) {
                screen_write_collect_insert_clear(ctx, (*ri).px.wrapping_sub(xoff), (*ri).nx, bg);
            }
            i = i.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
    screen_write_set_cursor(ctx, ocx as ::core::ffi::c_int, ocy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_clearhistory(mut ctx: *mut screen_write_ctx) {
    grid_clear_history((*(*ctx).s).grid);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_fullredraw(mut ctx: *mut screen_write_ctx) {
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_fullredraw\0" as *const u8 as *const ::core::ffi::c_char,
    );
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if ttyctx.redraw_cb.is_some() {
        ttyctx.redraw_cb.expect("non-null function pointer")(&raw mut ttyctx);
    }
}
unsafe fn screen_write_collect_trim(
    ctx: *mut screen_write_ctx,
    y: u_int,
    x: u_int,
    used: u_int,
    wrapped: *mut ::core::ffi::c_int,
) -> *mut screen_write_citem {
    let items = &mut (*(*(*ctx).s).write_list.offset(y as isize)).items;
    let sx = x;
    let ex = x.wrapping_add(used).wrapping_sub(1);
    let mut index = 0;
    loop {
        let ci = items.get_ptr(index);
        if ci.is_null() {
            break;
        }
        let csx = (*ci).x;
        let cex = csx.wrapping_add((*ci).used).wrapping_sub(1);
        if cex < sx {
            // Entirely before the replacement interval.
            index += 1;
        } else if csx > ex {
            return ci;
        } else if csx >= sx && cex <= ex {
            if csx == 0 && (*ci).wrapped != 0 && !wrapped.is_null() {
                *wrapped = 1;
            }
            screen_write_free_citem(items.remove_at(index));
        } else if csx < sx && cex >= sx && cex <= ex {
            (*ci).used = sx.wrapping_sub(csx);
            index += 1;
        } else if cex > ex && csx >= sx && csx <= ex {
            (*ci).x = ex.wrapping_add(1);
            (*ci).used = cex.wrapping_sub(ex);
            return ci;
        } else {
            let mut right = screen_write_get_citem();
            right.type_0 = (*ci).type_0;
            right.bg = (*ci).bg;
            right.gc = (*ci).gc;
            right.x = ex.wrapping_add(1);
            right.used = cex.wrapping_sub(ex);
            (*ci).used = sx.wrapping_sub(csx);
            let right_ptr = &raw mut *right;
            items.insert_at(index + 1, right);
            return right_ptr;
        }
    }
    std::ptr::null_mut()
}
unsafe extern "C" fn screen_write_collect_clear(
    mut ctx: *mut screen_write_ctx,
    mut y: u_int,
    mut n: u_int,
) {
    let mut cl: *mut screen_write_cline = ::core::ptr::null_mut::<screen_write_cline>();
    let mut i: u_int = 0;
    i = y;
    while i < y.wrapping_add(n) {
        cl = (*(*ctx).s).write_list.offset(i as isize) as *mut screen_write_cline;
        screen_write_recycle_items(&mut (*cl).items);
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn screen_write_collect_scroll(mut ctx: *mut screen_write_ctx, mut bg: u_int) {
    let mut s: *mut screen = (*ctx).s;
    let mut cl: *mut screen_write_cline = ::core::ptr::null_mut::<screen_write_cline>();
    let mut y: u_int = 0;
    let mut ci: *mut screen_write_citem = ::core::ptr::null_mut::<screen_write_citem>();
    log_debug(
        b"%s: at %u,%u (region %u-%u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_collect_scroll\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).cx,
        (*s).cy,
        (*s).rupper,
        (*s).rlower,
    );
    screen_write_collect_clear(ctx, (*s).rupper, 1 as u_int);
    let saved = std::mem::take(&mut (*(*(*ctx).s).write_list.offset((*s).rupper as isize)).data);
    y = (*s).rupper;
    while y < (*s).rlower {
        cl = (*(*ctx).s)
            .write_list
            .offset(y.wrapping_add(1 as u_int) as isize) as *mut screen_write_cline;
        (*(*s).write_list.offset(y as isize))
            .items
            .append(&mut (*cl).items);
        let ref mut fresh5 = (*(*(*ctx).s).write_list.offset(y as isize)).data;
        *fresh5 = std::mem::take(&mut (*cl).data);
        y = y.wrapping_add(1);
    }
    let ref mut fresh6 = (*(*(*ctx).s).write_list.offset((*s).rlower as isize)).data;
    *fresh6 = saved;
    let mut owner = screen_write_get_citem();
    ci = &raw mut *owner;
    (*ci).x = 0 as u_int;
    (*ci).used = (*(*s).grid).sx;
    (*ci).type_0 = CLEAR;
    (*ci).bg = bg;
    (*(*s).write_list.offset((*s).rlower as isize))
        .items
        .push_back(owner);
}
unsafe extern "C" fn screen_write_collect_flush_scrolled(
    mut ctx: *mut screen_write_ctx,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    let mut s: *mut screen = (*ctx).s;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    if ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 && !wp.is_null() {
        screen_write_redraw_pane(ctx, &raw mut ttyctx);
        return 0 as ::core::ffi::c_int;
    }
    if !wp.is_null() && window_pane_scrollbar_overlay_visible(wp) != 0 {
        (*wp).flags |= PANE_REDRAW;
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"%s: scrolled %u (region %u-%u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_collect_flush_scrolled\0" as *const u8 as *const ::core::ffi::c_char,
        (*ctx).scrolled,
        (*s).rupper,
        (*s).rlower,
    );
    if (*ctx).scrolled
        > (*s)
            .rlower
            .wrapping_sub((*s).rupper)
            .wrapping_add(1 as u_int)
    {
        (*ctx).scrolled = (*s)
            .rlower
            .wrapping_sub((*s).rupper)
            .wrapping_add(1 as u_int);
    }
    if !wp.is_null() && ((*wp).yoff as u_int).wrapping_add((*wp).sy) > (*(*wp).window).sy {
        ttyctx.orlower = ttyctx.orlower.wrapping_sub(
            ((*wp).yoff as u_int)
                .wrapping_add((*wp).sy)
                .wrapping_sub((*(*wp).window).sy),
        );
    }
    ttyctx.c2rust_unnamed.n = (*ctx).scrolled;
    ttyctx.bg = (*ctx).bg;
    tty_write(
        Some(tty_cmd_scrollup as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
        &raw mut ttyctx,
    );
    if !wp.is_null() {
        window_pane_scrollbar_redraw(wp);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn screen_write_collect_flush_line(
    mut ctx: *mut screen_write_ctx,
    mut y: u_int,
) -> u_int {
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    let mut s: *mut screen = (*ctx).s;
    let mut ci: *mut screen_write_citem = ::core::ptr::null_mut::<screen_write_citem>();
    let mut cl: *mut screen_write_cline =
        (*s).write_list.offset(y as isize) as *mut screen_write_cline;
    let mut last: u_int = UINT_MAX;
    let mut items: u_int = 0 as u_int;
    let mut wsx: u_int = 0;
    let mut wsy: u_int = 0;
    let mut w_length: u_int = 0;
    let mut i: u_int = 0;
    let mut w_start: ::core::ffi::c_int = 0;
    let mut w_end: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut written: ::core::ffi::c_int = 0;
    let mut r_start: ::core::ffi::c_int = 0;
    let mut r_end: ::core::ffi::c_int = 0;
    let mut c_start: ::core::ffi::c_int = 0;
    let mut c_end: ::core::ffi::c_int = 0;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    if !wp.is_null() {
        wsx = (*(*wp).window).sx;
        wsy = (*(*wp).window).sy;
        xoff = (*wp).xoff;
        yoff = (*wp).yoff;
    } else {
        wsx = (*(*s).grid).sx;
        wsy = (*(*s).grid).sy;
        xoff = 0 as ::core::ffi::c_int;
        yoff = 0 as ::core::ffi::c_int;
    }
    if y.wrapping_add(yoff as u_int) >= wsy {
        return 0 as u_int;
    }
    r = window_visible_ranges(
        wp,
        0 as ::core::ffi::c_int,
        y.wrapping_add(yoff as u_int) as ::core::ffi::c_int,
        wsx,
        ::core::ptr::null_mut::<visible_ranges>(),
    );
    let mut index = 0;
    loop {
        ci = (*cl).items.get_ptr(index);
        if ci.is_null() {
            break;
        }
        log_debug(
            b"collect list: x=%u (last %u), y=%u, used=%u\0" as *const u8
                as *const ::core::ffi::c_char,
            (*ci).x,
            last,
            y,
            (*ci).used,
        );
        if last != UINT_MAX && (*ci).x <= last {
            fatalx(
                b"collect list bad order: %u <= %u\0" as *const u8 as *const ::core::ffi::c_char,
                (*ci).x,
                last,
            );
        }
        w_length = 0 as u_int;
        written = 0 as ::core::ffi::c_int;
        i = 0 as u_int;
        while i < (*r).used {
            ri = (*r).ranges.offset(i as isize) as *mut visible_range;
            if !((*ri).nx == 0 as u_int) {
                r_start = (*ri).px as ::core::ffi::c_int;
                r_end = (*ri).px.wrapping_add((*ri).nx) as ::core::ffi::c_int;
                c_start = (*ci).x as ::core::ffi::c_int;
                c_end = (*ci).x.wrapping_add((*ci).used) as ::core::ffi::c_int;
                if !(c_start + xoff >= r_end || c_end + xoff <= r_start) {
                    if r_start > c_start + xoff {
                        w_start = r_start - xoff;
                    } else {
                        w_start = c_start;
                    }
                    if c_end + xoff > r_end {
                        w_end = r_end - xoff;
                    } else {
                        w_end = c_end;
                    }
                    if !(w_end <= w_start) {
                        w_length = (w_end - w_start) as u_int;
                        if !(w_length <= 0 as u_int) {
                            screen_write_set_cursor(ctx, w_start, y as ::core::ffi::c_int);
                            if (*ci).type_0 as ::core::ffi::c_uint
                                == CLEAR as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                screen_write_initctx(
                                    ctx,
                                    &raw mut ttyctx,
                                    1 as ::core::ffi::c_int,
                                    0 as ::core::ffi::c_int,
                                );
                                ttyctx.bg = (*ci).bg;
                                ttyctx.c2rust_unnamed.n = w_length;
                                tty_write(
                                    Some(
                                        tty_cmd_clearcharacter
                                            as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> (),
                                    ),
                                    &raw mut ttyctx,
                                );
                            } else {
                                screen_write_initctx(
                                    ctx,
                                    &raw mut ttyctx,
                                    0 as ::core::ffi::c_int,
                                    0 as ::core::ffi::c_int,
                                );
                                ttyctx.cell = &raw mut (*ci).gc;
                                if (*ci).wrapped != 0 {
                                    ttyctx.flags |= TTY_CTX_WRAPPED;
                                }
                                ttyctx.c2rust_unnamed.data.data =
                                    (*cl).data.as_mut_ptr().offset(w_start as isize);
                                ttyctx.c2rust_unnamed.data.size = w_length as size_t;
                                tty_write(
                                    Some(
                                        tty_cmd_cells
                                            as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> (),
                                    ),
                                    &raw mut ttyctx,
                                );
                            }
                            items = items.wrapping_add(1);
                            written = 1 as ::core::ffi::c_int;
                        }
                    }
                }
            }
            i = i.wrapping_add(1);
        }
        if written != 0 {
            last = (*ci).x;
            screen_write_free_citem((*cl).items.remove_at(index));
        } else {
            index += 1;
        }
    }
    return items;
}
unsafe extern "C" fn screen_write_collect_flush(
    mut ctx: *mut screen_write_ctx,
    mut scroll_only: ::core::ffi::c_int,
    mut from: *const ::core::ffi::c_char,
) {
    let mut current_block: u64;
    let mut s: *mut screen = (*ctx).s;
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    let mut y: u_int = 0;
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    let mut items: u_int = 0 as u_int;
    let mut cl: *mut screen_write_cline = ::core::ptr::null_mut::<screen_write_cline>();
    if !(!wp.is_null() && (*wp).flags & (PANE_REDRAW | PANE_DROP) != 0) {
        if (*s).mode & MODE_SYNC != 0 {
            if (*ctx).scrolled != 0 as u_int {
                screen_write_should_draw_lines(
                    ctx,
                    (*s).rupper,
                    (*s).rlower
                        .wrapping_add(1 as u_int)
                        .wrapping_sub((*s).rupper),
                );
            }
            y = 0 as u_int;
            while y < (*(*s).grid).sy {
                cl = (*s).write_list.offset(y as isize) as *mut screen_write_cline;
                if !(*cl).items.is_empty() {
                    screen_write_should_draw_line(ctx, y);
                }
                y = y.wrapping_add(1);
            }
        } else {
            if (*ctx).scrolled != 0 as u_int {
                if screen_write_collect_flush_scrolled(ctx) == 0 {
                    current_block = 9123463459983562265;
                } else {
                    (*ctx).scrolled = 0 as u_int;
                    current_block = 8236137900636309791;
                }
            } else {
                current_block = 8236137900636309791;
            }
            match current_block {
                9123463459983562265 => {}
                _ => {
                    (*ctx).bg = 8 as u_int;
                    if scroll_only != 0 {
                        return;
                    }
                    cx = (*s).cx;
                    cy = (*s).cy;
                    y = 0 as u_int;
                    while y < (*(*s).grid).sy {
                        items = items.wrapping_add(screen_write_collect_flush_line(ctx, y));
                        y = y.wrapping_add(1);
                    }
                    (*s).cx = cx;
                    (*s).cy = cy;
                    log_debug(
                        b"%s: flushed %u items (%s)\0" as *const u8 as *const ::core::ffi::c_char,
                        b"screen_write_collect_flush\0" as *const u8 as *const ::core::ffi::c_char,
                        items,
                        from,
                    );
                    return;
                }
            }
        }
    }
    y = 0 as u_int;
    while y < (*(*s).grid).sy {
        cl = (*s).write_list.offset(y as isize) as *mut screen_write_cline;
        while let Some(item) = (*cl).items.pop_front() {
            screen_write_free_citem(item);
        }
        y = y.wrapping_add(1);
    }
    (*ctx).scrolled = 0 as u_int;
    (*ctx).bg = 8 as u_int;
}
unsafe extern "C" fn screen_write_collect_insert(
    mut ctx: *mut screen_write_ctx,
    mut ci: *mut screen_write_citem,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut cl: *mut screen_write_cline =
        (*s).write_list.offset((*s).cy as isize) as *mut screen_write_cline;
    let mut before: *mut screen_write_citem = ::core::ptr::null_mut::<screen_write_citem>();
    before = screen_write_collect_trim(ctx, (*s).cy, (*ci).x, (*ci).used, &raw mut (*ci).wrapped);
    (*cl).items.insert_before(
        before,
        (*ctx).item.take().expect("active screen write context"),
    );
    (*ctx).item = Some(screen_write_get_citem());
}
unsafe extern "C" fn screen_write_collect_insert_clear(
    mut ctx: *mut screen_write_ctx,
    mut px: u_int,
    mut nx: u_int,
    mut bg: u_int,
) {
    let mut ci: *mut screen_write_citem = screen_write_current_item(ctx);
    if nx != 0 as u_int {
        (*ci).x = px;
        (*ci).used = nx;
        (*ci).type_0 = CLEAR;
        (*ci).bg = bg;
        screen_write_collect_insert(ctx, ci);
    }
}
unsafe extern "C" fn screen_write_clear_cell(mut gd: *mut grid, mut px: u_int, mut py: u_int) {
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
    let mut bg: ::core::ffi::c_int = 0;
    grid_view_get_cell(gd, px, py, &raw mut gc);
    bg = gc.bg;
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    gc.bg = bg;
    grid_view_set_cell(gd, px, py, &raw mut gc);
}
unsafe extern "C" fn screen_write_insert_clears(
    mut ctx: *mut screen_write_ctx,
    mut px: u_int,
    mut nx: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
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
    let mut xx: u_int = 0;
    let mut start: u_int = px;
    let mut n: u_int = 0;
    let mut bg: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
    xx = px;
    while xx < px.wrapping_add(nx) {
        grid_view_get_cell((*s).grid, xx, (*s).cy, &raw mut gc);
        if xx == start {
            bg = gc.bg;
        } else if gc.bg != bg {
            n = xx.wrapping_sub(start);
            log_debug(
                b"%s: from %u, size %u\0" as *const u8 as *const ::core::ffi::c_char,
                b"screen_write_insert_clears\0" as *const u8 as *const ::core::ffi::c_char,
                start,
                n,
            );
            screen_write_collect_insert_clear(ctx, start, n, bg as u_int);
            start = xx;
            bg = gc.bg;
        }
        xx = xx.wrapping_add(1);
    }
    log_debug(
        b"%s: from %u, size %u\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_insert_clears\0" as *const u8 as *const ::core::ffi::c_char,
        start,
        xx.wrapping_sub(start),
    );
    screen_write_collect_insert_clear(ctx, start, xx.wrapping_sub(start), bg as u_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_collect_end(mut ctx: *mut screen_write_ctx) {
    let mut s: *mut screen = (*ctx).s;
    let mut ci: *mut screen_write_citem = screen_write_current_item(ctx);
    let mut cl: *mut screen_write_cline =
        (*s).write_list.offset((*s).cy as isize) as *mut screen_write_cline;
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
    let mut xx: u_int = 0;
    let mut bx: u_int = 0 as u_int;
    let mut bnx: u_int = 0 as u_int;
    if (*ci).used == 0 as u_int {
        return;
    }
    (*ci).x = (*s).cx;
    screen_write_collect_insert(ctx, ci);
    log_debug(
        b"%s: %u %.*s (at %u,%u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_collect_end\0" as *const u8 as *const ::core::ffi::c_char,
        (*ci).used,
        (*ci).used as ::core::ffi::c_int,
        (*cl).data.as_mut_ptr().offset((*ci).x as isize),
        (*s).cx,
        (*s).cy,
    );
    if (*s).cx != 0 as u_int {
        xx = (*s).cx;
        while xx > 0 as u_int {
            grid_view_get_cell((*s).grid, xx, (*s).cy, &raw mut gc);
            if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
                break;
            }
            screen_write_clear_cell((*s).grid, xx, (*s).cy);
            log_debug(
                b"%s: padding erased (before) at %u (cx %u)\0" as *const u8
                    as *const ::core::ffi::c_char,
                b"screen_write_collect_end\0" as *const u8 as *const ::core::ffi::c_char,
                xx,
                (*s).cx,
            );
            xx = xx.wrapping_sub(1);
        }
        if xx != (*s).cx {
            if xx == 0 as u_int {
                grid_view_get_cell((*s).grid, 0 as u_int, (*s).cy, &raw mut gc);
            }
            if gc.data.width as ::core::ffi::c_int > 1 as ::core::ffi::c_int
                || gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0
            {
                screen_write_clear_cell((*s).grid, xx, (*s).cy);
                log_debug(
                    b"%s: padding erased (before) at %u (cx %u)\0" as *const u8
                        as *const ::core::ffi::c_char,
                    b"screen_write_collect_end\0" as *const u8 as *const ::core::ffi::c_char,
                    xx,
                    (*s).cx,
                );
            }
            bx = xx;
            bnx = (*s).cx.wrapping_sub(xx);
        }
    }
    grid_view_set_cells(
        (*s).grid,
        (*s).cx,
        (*s).cy,
        &raw mut (*ci).gc,
        (*cl).data.as_mut_ptr().offset((*ci).x as isize),
        (*ci).used as size_t,
    );
    if bnx != 0 as u_int {
        screen_write_insert_clears(ctx, bx, bnx);
    }
    screen_write_set_cursor(
        ctx,
        (*s).cx.wrapping_add((*ci).used) as ::core::ffi::c_int,
        -(1 as ::core::ffi::c_int),
    );
    xx = (*s).cx;
    while xx < (*(*s).grid).sx {
        grid_view_get_cell((*s).grid, xx, (*s).cy, &raw mut gc);
        if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
            break;
        }
        screen_write_clear_cell((*s).grid, xx, (*s).cy);
        log_debug(
            b"%s: padding erased (after) at %u (cx %u)\0" as *const u8
                as *const ::core::ffi::c_char,
            b"screen_write_collect_end\0" as *const u8 as *const ::core::ffi::c_char,
            xx,
            (*s).cx,
        );
        xx = xx.wrapping_add(1);
    }
    if xx != (*s).cx {
        screen_write_insert_clears(ctx, (*s).cx, xx.wrapping_sub((*s).cx));
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_collect_add(
    mut ctx: *mut screen_write_ctx,
    mut gc: *const grid_cell,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut ci: *mut screen_write_citem = ::core::ptr::null_mut::<screen_write_citem>();
    let mut sx: u_int = (*(*s).grid).sx;
    let mut collect: ::core::ffi::c_int = 0;
    collect = 1 as ::core::ffi::c_int;
    if (*gc).data.width as ::core::ffi::c_int != 1 as ::core::ffi::c_int
        || (*gc).data.size as ::core::ffi::c_int != 1 as ::core::ffi::c_int
        || *(&raw const (*gc).data.data as *const u_char) as ::core::ffi::c_int
            >= 0x7f as ::core::ffi::c_int
    {
        collect = 0 as ::core::ffi::c_int;
    } else if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
        collect = 0 as ::core::ffi::c_int;
    } else if (*gc).attr as ::core::ffi::c_int & GRID_ATTR_CHARSET != 0 {
        collect = 0 as ::core::ffi::c_int;
    } else if !(*s).mode & MODE_WRAP != 0 {
        collect = 0 as ::core::ffi::c_int;
    } else if (*s).mode & MODE_INSERT != 0 {
        collect = 0 as ::core::ffi::c_int;
    } else if !(*s).sel.is_none() {
        collect = 0 as ::core::ffi::c_int;
    }
    if collect == 0 {
        screen_write_collect_end(ctx);
        screen_write_collect_flush(
            ctx,
            0 as ::core::ffi::c_int,
            b"screen_write_collect_add\0" as *const u8 as *const ::core::ffi::c_char,
        );
        screen_write_cell(ctx, gc);
        return;
    }
    if (*s).cx > sx.wrapping_sub(1 as u_int)
        || (*screen_write_current_item(ctx)).used
            > sx.wrapping_sub(1 as u_int).wrapping_sub((*s).cx)
    {
        screen_write_collect_end(ctx);
    }
    ci = screen_write_current_item(ctx);
    if (*s).cx > sx.wrapping_sub(1 as u_int) {
        log_debug(
            b"%s: wrapped at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_write_collect_add\0" as *const u8 as *const ::core::ffi::c_char,
            (*s).cx,
            (*s).cy,
        );
        (*ci).wrapped = 1 as ::core::ffi::c_int;
        screen_write_linefeed(ctx, 1 as ::core::ffi::c_int, 8 as u_int);
        screen_write_set_cursor(ctx, 0 as ::core::ffi::c_int, -(1 as ::core::ffi::c_int));
    }
    if (*ci).used == 0 as u_int {
        memcpy(
            &raw mut (*ci).gc as *mut ::core::ffi::c_void,
            gc as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    }
    if (&(*(*(*ctx).s).write_list.offset((*s).cy as isize)).data).is_empty() {
        let width = (*(*(*ctx).s).grid).sx as usize;
        if width == 0 {
            fatalx(b"xmalloc: zero size\0" as *const u8 as *const ::core::ffi::c_char);
        }
        let ref mut fresh11 = (*(*(*ctx).s).write_list.offset((*s).cy as isize)).data;
        fresh11.resize_with(width, || 0);
    }
    let fresh12 = (*ci).used;
    (*ci).used = (*ci).used.wrapping_add(1);
    *(*(*(*ctx).s).write_list.offset((*s).cy as isize))
        .data
        .as_mut_ptr()
        .offset((*s).cx.wrapping_add(fresh12) as isize) =
        (*gc).data.data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_cell(
    mut ctx: *mut screen_write_ctx,
    mut gc: *const grid_cell,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    let mut gd: *mut grid = (*s).grid;
    let mut ud: *const utf8_data = &raw const (*gc).data;
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut gce: *mut grid_cell_entry = ::core::ptr::null_mut::<grid_cell_entry>();
    let mut tmp_gc: grid_cell = grid_cell {
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
    let mut now_gc: grid_cell = grid_cell {
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
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut sx: u_int = (*(*s).grid).sx;
    let mut sy: u_int = (*(*s).grid).sy;
    let mut width: u_int = (*ud).width as u_int;
    let mut xx: u_int = 0;
    let mut not_wrap: u_int = 0;
    let mut i: u_int = 0;
    let mut n: u_int = 0;
    let mut vis: u_int = 0;
    let mut selected: ::core::ffi::c_int = 0;
    let mut skip: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut redraw: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut yoff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut xoff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
        return;
    }
    if screen_write_combine(ctx, gc) != 0 as ::core::ffi::c_int {
        return;
    }
    screen_write_collect_flush(
        ctx,
        1 as ::core::ffi::c_int,
        b"screen_write_cell\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if !(*s).mode & MODE_WRAP != 0
        && width > 1 as u_int
        && (width > sx || (*s).cx != sx && (*s).cx > sx.wrapping_sub(width))
    {
        return;
    }
    if (*s).mode & MODE_INSERT != 0 {
        grid_view_insert_cells((*s).grid, (*s).cx, (*s).cy, width, 8 as u_int);
        skip = 0 as ::core::ffi::c_int;
    }
    if (*s).mode & MODE_WRAP != 0 && (*s).cx > sx.wrapping_sub(width) {
        log_debug(
            b"%s: wrapped at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_write_cell\0" as *const u8 as *const ::core::ffi::c_char,
            (*s).cx,
            (*s).cy,
        );
        screen_write_linefeed(ctx, 1 as ::core::ffi::c_int, 8 as u_int);
        screen_write_set_cursor(ctx, 0 as ::core::ffi::c_int, -(1 as ::core::ffi::c_int));
        screen_write_collect_flush(
            ctx,
            0 as ::core::ffi::c_int,
            b"screen_write_cell\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if (*s).cx > sx.wrapping_sub(width) || (*s).cy > sy.wrapping_sub(1 as u_int) {
        return;
    }
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    gl = grid_get_line((*s).grid, (*(*s).grid).hsize.wrapping_add((*s).cy));
    if (*gl).flags as ::core::ffi::c_int & GRID_LINE_EXTENDED != 0 {
        grid_view_get_cell(gd, (*s).cx, (*s).cy, &raw mut now_gc);
        if screen_write_overwrite(ctx, &raw mut now_gc, width) != 0 {
            redraw = 1 as ::core::ffi::c_int;
            skip = 0 as ::core::ffi::c_int;
        }
    }
    xx = (*s).cx.wrapping_add(1 as u_int);
    while xx < (*s).cx.wrapping_add(width) {
        log_debug(
            b"%s: new padding at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_write_cell\0" as *const u8 as *const ::core::ffi::c_char,
            xx,
            (*s).cy,
        );
        grid_view_set_padding(gd, xx, (*s).cy, (*gc).bg);
        skip = 0 as ::core::ffi::c_int;
        xx = xx.wrapping_add(1);
    }
    if skip != 0 {
        if (*s).cx >= (*gl).cellsize as u_int {
            skip = grid_cells_equal(gc, &raw const grid_default_cell);
        } else {
            gce = (*gl).celldata.as_mut_ptr().offset((*s).cx as isize) as *mut grid_cell_entry;
            if (*gce).flags as ::core::ffi::c_int & GRID_FLAG_EXTENDED != 0 {
                skip = 0 as ::core::ffi::c_int;
            } else if (*gc).flags as ::core::ffi::c_int != (*gce).flags as ::core::ffi::c_int {
                skip = 0 as ::core::ffi::c_int;
            } else if (*gc).attr as ::core::ffi::c_int
                != (*gce).c2rust_unnamed.data.attr as ::core::ffi::c_int
            {
                skip = 0 as ::core::ffi::c_int;
            } else if (*gc).fg != (*gce).c2rust_unnamed.data.fg as ::core::ffi::c_int {
                skip = 0 as ::core::ffi::c_int;
            } else if (*gc).bg != (*gce).c2rust_unnamed.data.bg as ::core::ffi::c_int {
                skip = 0 as ::core::ffi::c_int;
            } else if (*gc).data.width as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
                skip = 0 as ::core::ffi::c_int;
            } else if (*gc).data.size as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
                skip = 0 as ::core::ffi::c_int;
            } else if (*gce).c2rust_unnamed.data.data as ::core::ffi::c_int
                != (*gc).data.data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            {
                skip = 0 as ::core::ffi::c_int;
            }
        }
    }
    selected = screen_check_selection(s, (*s).cx, (*s).cy);
    if selected != 0 && !((*gc).flags as ::core::ffi::c_int) & GRID_FLAG_SELECTED != 0 {
        memcpy(
            &raw mut tmp_gc as *mut ::core::ffi::c_void,
            gc as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
        tmp_gc.flags = (tmp_gc.flags as ::core::ffi::c_int | GRID_FLAG_SELECTED) as u_char;
        grid_view_set_cell(gd, (*s).cx, (*s).cy, &raw mut tmp_gc);
    } else if selected == 0 && (*gc).flags as ::core::ffi::c_int & GRID_FLAG_SELECTED != 0 {
        memcpy(
            &raw mut tmp_gc as *mut ::core::ffi::c_void,
            gc as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
        tmp_gc.flags = (tmp_gc.flags as ::core::ffi::c_int & !GRID_FLAG_SELECTED) as u_char;
        grid_view_set_cell(gd, (*s).cx, (*s).cy, &raw mut tmp_gc);
    } else if skip == 0 {
        grid_view_set_cell(gd, (*s).cx, (*s).cy, gc);
    }
    if selected != 0 {
        skip = 0 as ::core::ffi::c_int;
    }
    if !wp.is_null() {
        xoff = (*wp).xoff;
        yoff = (*wp).yoff;
    }
    r = window_visible_ranges(
        wp,
        (xoff as u_int).wrapping_add((*s).cx) as ::core::ffi::c_int,
        (*s).cy.wrapping_add(yoff as u_int) as ::core::ffi::c_int,
        width,
        ::core::ptr::null_mut::<visible_ranges>(),
    );
    not_wrap = ((*s).mode & MODE_WRAP == 0) as ::core::ffi::c_int as u_int;
    if (*s).cx <= sx.wrapping_sub(not_wrap).wrapping_sub(width) {
        screen_write_set_cursor(
            ctx,
            (*s).cx.wrapping_add(width) as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
        );
    } else {
        screen_write_set_cursor(
            ctx,
            sx.wrapping_sub(not_wrap) as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
        );
    }
    if (*s).mode & MODE_INSERT != 0 {
        screen_write_collect_flush(
            ctx,
            0 as ::core::ffi::c_int,
            b"screen_write_cell\0" as *const u8 as *const ::core::ffi::c_char,
        );
        ttyctx.c2rust_unnamed.n = width;
        if screen_write_should_draw_line(ctx, (*s).cy) != 0 {
            tty_write(
                Some(
                    tty_cmd_insertcharacter as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> (),
                ),
                &raw mut ttyctx,
            );
        }
    }
    if skip != 0 || screen_write_should_draw_line(ctx, (*s).cy) == 0 {
        return;
    }
    if redraw != 0 && !wp.is_null() {
        screen_write_redraw_line(ctx, &raw mut ttyctx, (*s).cy);
        return;
    }
    if selected != 0 {
        screen_select_cell(s, &raw mut tmp_gc, gc);
    } else {
        memcpy(
            &raw mut tmp_gc as *mut ::core::ffi::c_void,
            gc as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    }
    ttyctx.cell = &raw mut tmp_gc;
    i = 0 as u_int;
    vis = 0 as u_int;
    while i < (*r).used {
        vis = vis.wrapping_add((*(*r).ranges.offset(i as isize)).nx);
        i = i.wrapping_add(1);
    }
    if vis >= width {
        if screen_write_should_draw_line(ctx, (*s).cy) != 0 {
            tty_write(
                Some(tty_cmd_cell as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
                &raw mut ttyctx,
            );
        }
        return;
    }
    utf8_set(&raw mut tmp_gc.data, ' ' as i32 as u_char);
    if screen_write_should_draw_line(ctx, (*s).cy) == 0 {
        return;
    }
    i = 0 as u_int;
    while i < (*r).used {
        ri = (*r).ranges.offset(i as isize) as *mut visible_range;
        if !((*ri).nx == 0 as u_int) {
            n = 0 as u_int;
            while n < (*ri).nx {
                ttyctx.ocx =
                    ((*ri).px as ::core::ffi::c_int - xoff + n as ::core::ffi::c_int) as u_int;
                tty_write(
                    Some(tty_cmd_cell as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
                    &raw mut ttyctx,
                );
                n = n.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn screen_write_combine(
    mut ctx: *mut screen_write_ctx,
    mut gc: *const grid_cell,
) -> ::core::ffi::c_int {
    let mut s: *mut screen = (*ctx).s;
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    let mut gd: *mut grid = (*s).grid;
    let mut ud: *const utf8_data = &raw const (*gc).data;
    let mut oo: *mut options = global_options;
    let mut i: u_int = 0;
    let mut n: u_int = 0;
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    let mut vis: u_int = 0;
    let mut last: grid_cell = grid_cell {
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
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut force_wide: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut zero_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut xoff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut yoff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    if utf8_is_hangul_filler(ud) != 0 {
        return 1 as ::core::ffi::c_int;
    }
    if utf8_is_zwj(ud) != 0 {
        zero_width = 1 as ::core::ffi::c_int;
    } else if utf8_is_vs(ud) != 0 {
        zero_width = 1 as ::core::ffi::c_int;
        if options_get_number(
            oo,
            b"variation-selector-always-wide\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
        {
            force_wide = 1 as ::core::ffi::c_int;
        }
    } else if (*ud).width as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        zero_width = 1 as ::core::ffi::c_int;
    }
    if ((*ud).size as ::core::ffi::c_int) < 2 as ::core::ffi::c_int || cx == 0 as u_int {
        return zero_width;
    }
    log_debug(
        b"%s: character %.*s at %u,%u (width %u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_combine\0" as *const u8 as *const ::core::ffi::c_char,
        (*ud).size as ::core::ffi::c_int,
        &raw const (*ud).data as *const u_char,
        cx,
        cy,
        (*ud).width as ::core::ffi::c_int,
    );
    n = 1 as u_int;
    grid_view_get_cell(gd, cx.wrapping_sub(n), cy, &raw mut last);
    if cx != 1 as u_int && last.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
        n = 2 as u_int;
        grid_view_get_cell(gd, cx.wrapping_sub(n), cy, &raw mut last);
    }
    if n != last.data.width as u_int || last.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
        return zero_width;
    }
    if zero_width == 0 {
        match hanguljamo_check_state(&raw mut last.data, ud) as ::core::ffi::c_uint {
            3 => return 1 as ::core::ffi::c_int,
            1 => return 0 as ::core::ffi::c_int,
            0 => {
                if utf8_should_combine(&raw mut last.data, ud) != 0 {
                    force_wide = 1 as ::core::ffi::c_int;
                } else if utf8_should_combine(ud, &raw mut last.data) != 0 {
                    force_wide = 1 as ::core::ffi::c_int;
                } else if utf8_has_zwj(&raw mut last.data) == 0 {
                    return 0 as ::core::ffi::c_int;
                }
            }
            2 | _ => {}
        }
    }
    if (last.data.size as ::core::ffi::c_int + (*ud).size as ::core::ffi::c_int) as usize
        > ::core::mem::size_of::<[u_char; 32]>() as usize
    {
        return zero_width;
    }
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_combine\0" as *const u8 as *const ::core::ffi::c_char,
    );
    log_debug(
        b"%s: %.*s -> %.*s at %u,%u (offset %u, width %u)\0" as *const u8
            as *const ::core::ffi::c_char,
        b"screen_write_combine\0" as *const u8 as *const ::core::ffi::c_char,
        (*ud).size as ::core::ffi::c_int,
        &raw const (*ud).data as *const u_char,
        last.data.size as ::core::ffi::c_int,
        &raw mut last.data.data as *mut u_char,
        cx.wrapping_sub(n),
        cy,
        n,
        last.data.width as ::core::ffi::c_int,
    );
    memcpy(
        (&raw mut last.data.data as *mut u_char)
            .offset(last.data.size as ::core::ffi::c_int as isize)
            as *mut ::core::ffi::c_void,
        &raw const (*ud).data as *const u_char as *const ::core::ffi::c_void,
        (*ud).size as size_t,
    );
    last.data.size =
        (last.data.size as ::core::ffi::c_int + (*ud).size as ::core::ffi::c_int) as u_char;
    if last.data.width as ::core::ffi::c_int == 1 as ::core::ffi::c_int && force_wide != 0 {
        last.data.width = 2 as u_char;
        n = 2 as u_int;
        cx = cx.wrapping_add(1);
    } else {
        force_wide = 0 as ::core::ffi::c_int;
    }
    grid_view_set_cell(gd, cx.wrapping_sub(n), cy, &raw mut last);
    if force_wide != 0 {
        grid_view_set_padding(gd, cx.wrapping_sub(1 as u_int), cy, last.bg);
    }
    if !wp.is_null() {
        xoff = (*wp).xoff;
        yoff = (*wp).yoff;
    }
    r = window_visible_ranges(
        wp,
        (xoff as u_int).wrapping_add(cx).wrapping_sub(n) as ::core::ffi::c_int,
        cy.wrapping_add(yoff as u_int) as ::core::ffi::c_int,
        n,
        ::core::ptr::null_mut::<visible_ranges>(),
    );
    i = 0 as u_int;
    vis = 0 as u_int;
    while i < (*r).used {
        vis = vis.wrapping_add((*(*r).ranges.offset(i as isize)).nx);
        i = i.wrapping_add(1);
    }
    if vis < n {
        return 1 as ::core::ffi::c_int;
    }
    screen_write_set_cursor(
        ctx,
        cx.wrapping_sub(n) as ::core::ffi::c_int,
        cy as ::core::ffi::c_int,
    );
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    ttyctx.cell = &raw mut last;
    if force_wide != 0 {
        ttyctx.flags |= TTY_CTX_CELL_INVALIDATE;
    }
    if screen_write_should_draw_line(ctx, cy) != 0 {
        tty_write(
            Some(tty_cmd_cell as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
    }
    screen_write_set_cursor(ctx, cx as ::core::ffi::c_int, cy as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn screen_write_overwrite(
    mut ctx: *mut screen_write_ctx,
    mut gc: *mut grid_cell,
    mut width: u_int,
) -> ::core::ffi::c_int {
    let mut s: *mut screen = (*ctx).s;
    let mut gd: *mut grid = (*s).grid;
    let mut tmp_gc: grid_cell = grid_cell {
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
    let mut done: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
        xx = (*s).cx.wrapping_add(1 as u_int);
        loop {
            xx = xx.wrapping_sub(1);
            if !(xx > 0 as u_int) {
                break;
            }
            grid_view_get_cell(gd, xx, (*s).cy, &raw mut tmp_gc);
            if !(tmp_gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
                break;
            }
            log_debug(
                b"%s: padding at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
                b"screen_write_overwrite\0" as *const u8 as *const ::core::ffi::c_char,
                xx,
                (*s).cy,
            );
            screen_write_clear_cell(gd, xx, (*s).cy);
        }
        log_debug(
            b"%s: character at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_write_overwrite\0" as *const u8 as *const ::core::ffi::c_char,
            xx,
            (*s).cy,
        );
        screen_write_clear_cell(gd, xx, (*s).cy);
        done = 1 as ::core::ffi::c_int;
    }
    if width != 1 as u_int
        || (*gc).data.width as ::core::ffi::c_int != 1 as ::core::ffi::c_int
        || (*gc).flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0
    {
        xx = (*s).cx.wrapping_add(width).wrapping_sub(1 as u_int);
        loop {
            xx = xx.wrapping_add(1);
            if !(xx < (*(*s).grid).sx) {
                break;
            }
            grid_view_get_cell(gd, xx, (*s).cy, &raw mut tmp_gc);
            if !(tmp_gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
                break;
            }
            log_debug(
                b"%s: overwrite at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
                b"screen_write_overwrite\0" as *const u8 as *const ::core::ffi::c_char,
                xx,
                (*s).cy,
            );
            screen_write_clear_cell(gd, xx, (*s).cy);
            done = 1 as ::core::ffi::c_int;
        }
    }
    return done;
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_setselection(
    mut ctx: *mut screen_write_ctx,
    mut clip: *const ::core::ffi::c_char,
    mut str: *mut u_char,
    mut len: u_int,
) {
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    ttyctx.c2rust_unnamed.sel.clip = clip;
    ttyctx.c2rust_unnamed.sel.data = str as *const ::core::ffi::c_char;
    ttyctx.c2rust_unnamed.sel.size = len as size_t;
    tty_write(
        Some(tty_cmd_setselection as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
        &raw mut ttyctx,
    );
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_rawstring(
    mut ctx: *mut screen_write_ctx,
    mut str: *mut u_char,
    mut len: u_int,
    mut allow_invisible_panes: ::core::ffi::c_int,
) {
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if allow_invisible_panes != 0 {
        ttyctx.flags |= TTY_CTX_INVISIBLE_PANES;
    }
    ttyctx.c2rust_unnamed.data.data = str as *const ::core::ffi::c_char;
    ttyctx.c2rust_unnamed.data.size = len as size_t;
    tty_write(
        Some(tty_cmd_rawstring as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
        &raw mut ttyctx,
    );
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_alternateon(
    mut ctx: *mut screen_write_ctx,
    mut gc: *mut grid_cell,
    mut cursor: ::core::ffi::c_int,
) {
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    if !wp.is_null()
        && options_get_number(
            (*wp).options,
            b"alternate-screen\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0
    {
        return;
    }
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_alternateon\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if screen_alternate_on((*ctx).s, gc, cursor) == 0 {
        return;
    }
    if !wp.is_null() {
        window_pane_clear_resizes(wp, ::core::ptr::null_mut::<window_pane_resize>());
        if event_initialized(&(*wp).resize_timer) != 0 {
            event_del(&raw mut (*wp).resize_timer);
        }
        layout_fix_panes(
            (*wp).window as *mut window,
            ::core::ptr::null_mut::<window_pane>(),
        );
        if !(*wp).resize_queue.is_empty() {
            window_pane_send_resize(wp, (*wp).sx, (*wp).sy);
            window_pane_clear_resizes(wp, ::core::ptr::null_mut::<window_pane_resize>());
        }
        server_redraw_window_borders((*wp).window as *mut window);
    }
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if ttyctx.redraw_cb.is_some() {
        ttyctx.redraw_cb.expect("non-null function pointer")(&raw mut ttyctx);
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_alternateoff(
    mut ctx: *mut screen_write_ctx,
    mut gc: *mut grid_cell,
    mut cursor: ::core::ffi::c_int,
) {
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: tty_ctx_c2rust_unnamed { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        },
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    if !wp.is_null()
        && options_get_number(
            (*wp).options,
            b"alternate-screen\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0
    {
        return;
    }
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_alternateoff\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if screen_alternate_off((*ctx).s, gc, cursor) == 0 {
        return;
    }
    if !wp.is_null() {
        layout_fix_panes(
            (*wp).window as *mut window,
            ::core::ptr::null_mut::<window_pane>(),
        );
        server_redraw_window_borders((*wp).window as *mut window);
    }
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if ttyctx.redraw_cb.is_some() {
        ttyctx.redraw_cb.expect("non-null function pointer")(&raw mut ttyctx);
    }
}

#[cfg(test)]
mod write_row_tests {
    use super::*;

    #[test]
    fn scroll_moves_text_owners_and_tears_down_rows() {
        unsafe {
            let mut grid = crate::src::grid::grid_create_box(4, 3, 0);
            let mut s: screen = screen::empty();
            s.grid = &raw mut *grid;
            s.rupper = 0;
            s.rlower = 2;
            screen_write_make_list(&raw mut s);
            let mut pointers = Vec::new();
            for y in 0..3 {
                let row = &mut *s.write_list.add(y);
                row.data
                    .resize_with(4, || (b'A' + y as u8) as ::core::ffi::c_char);
                pointers.push(row.data.as_ptr());
            }
            let mut ctx: screen_write_ctx = Default::default();
            ctx.s = &raw mut s;
            for _ in 0..4 {
                screen_write_collect_scroll(&raw mut ctx, 8);
                pointers.rotate_left(1);
                for (y, pointer) in pointers.iter().enumerate() {
                    assert_eq!((*s.write_list.add(y)).data.as_ptr(), *pointer);
                }
            }
            // Split and trim queued commands without losing ownership or wrap
            // state, then reuse the detached nodes through the same pool.
            let row = s.write_list;
            screen_write_recycle_items(&mut (*row).items);
            ctx.item = Some(screen_write_get_citem());
            for (x, used, wrapped) in [(0, 10, 1), (3, 3, 0)] {
                let item = screen_write_current_item(&raw mut ctx);
                (*item).x = x;
                (*item).used = used;
                (*item).wrapped = wrapped;
                screen_write_collect_insert(&raw mut ctx, item);
            }
            let mut intervals = Vec::new();
            let mut index = 0;
            loop {
                let item = (*row).items.get_ptr(index);
                if item.is_null() {
                    break;
                }
                intervals.push(((*item).x, (*item).used, (*item).wrapped));
                index += 1;
            }
            assert_eq!(intervals, [(0, 3, 1), (3, 3, 0), (6, 4, 0)]);
            let mut wrapped = 0;
            let tail = screen_write_collect_trim(&raw mut ctx, 0, 0, 8, &raw mut wrapped);
            assert_eq!(wrapped, 1);
            assert_eq!(((*tail).x, (*tail).used), (8, 2));
            screen_write_collect_trim(&raw mut ctx, 0, 9, 1, std::ptr::null_mut());
            assert_eq!(((*tail).x, (*tail).used), (8, 1));
            screen_write_collect_trim(&raw mut ctx, 0, 0, 20, std::ptr::null_mut());
            assert!((*row).items.is_empty());
            screen_write_free_citem(ctx.item.take().unwrap());
            screen_write_free_list(&raw mut s);
            assert!(s.write_list.is_null());
        }
    }
}
