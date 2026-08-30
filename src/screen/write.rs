use super::redraw::screen_redraw_get_visible_ranges;
use super::redraw::screen_redraw_is_visible;
use super::state::screen_mode_to_string;
use super::state::{
    screen_alternate_off, screen_alternate_on, screen_check_selection, screen_grid,
    screen_grid_mut, screen_grid_ptr, screen_reset_tabs, screen_select_cell,
};
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc};
use crate::format::format_draw;
use crate::grid::grid_cells_equal;
use crate::grid::{grid_clear_history, grid_default_cell, grid_get_cell, grid_get_line};
use crate::grid::{
    grid_view_clear, grid_view_clear_history, grid_view_delete_cells, grid_view_delete_lines,
    grid_view_delete_lines_region, grid_view_get_cell, grid_view_insert_cells,
    grid_view_insert_lines, grid_view_insert_lines_region, grid_view_scroll_region_down,
    grid_view_scroll_region_up, grid_view_set_cell, grid_view_set_cells, grid_view_set_padding,
};
use crate::layout::layout_fix_panes;
use crate::log::{fatalx, log_debug, log_get_level};
use crate::options::{options_get_number, options_ptr};
use crate::reactor::Timer;
use crate::server::server_redraw_window_borders;
use crate::session::session_get_curw;
use crate::session::session_has;
use crate::status::{status_at_line, status_line_size};
use crate::terminfo::{tty_acs_double_borders, tty_acs_heavy_borders, tty_acs_rounded_borders};
use crate::text::{
    HANGULJAMO_STATE_CHOSEONG, HANGULJAMO_STATE_NOT_COMPOSABLE, HANGULJAMO_STATE_NOT_HANGULJAMO,
    hanguljamo_check_state, utf8_has_zwj, utf8_is_hangul_filler, utf8_is_vs, utf8_is_zwj,
    utf8_should_combine,
};
use crate::text::{utf8_append, utf8_copy, utf8_fromcstr, utf8_open, utf8_set};
use crate::tmux::global_options;
use crate::tree::GlobalQueue;
use crate::tty::{
    tty_cmd_alignmenttest, tty_cmd_cell, tty_cmd_cells, tty_cmd_clearcharacter,
    tty_cmd_clearendofscreen, tty_cmd_clearscreen, tty_cmd_clearstartofscreen,
    tty_cmd_deletecharacter, tty_cmd_deleteline, tty_cmd_insertcharacter, tty_cmd_insertline,
    tty_cmd_rawstring, tty_cmd_redrawline, tty_cmd_reverseindex, tty_cmd_scrolldown,
    tty_cmd_scrollup, tty_cmd_setselection, tty_cmd_syncstart, tty_default_colours,
    tty_update_window_offset, tty_window_offset, tty_write,
};
pub use crate::types::*;
use crate::window::PaneStack;
use crate::window::window_get_active;
use crate::window::{window_pane_is_floating, window_pane_stack_prev, window_ref_from_ptr};
use ::core::ffi::{CStr, c_char, c_int, c_uint};
use ::core::ptr::null_mut;
#[repr(C)]
pub struct screen_write_cline {
    pub data: Option<Box<[u8]>>,
    /// What the line has collected, left to right. The items belong to the
    /// line until they go back to the free list.
    pub items: citems,
}

/// The collected items of one line, in the order they will be written.
pub type citems = ::std::vec::Vec<CItem>;
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct screen_write_citem {
    pub x: u_int,
    pub wrapped: c_int,
    pub type_0: screen_write_citem_type,
    pub used: u_int,
    pub bg: u_int,
    pub gc: grid_cell,
}
pub type screen_write_citem_type = c_uint;
pub const CLEAR: screen_write_citem_type = 1;
pub const TEXT: screen_write_citem_type = 0;
pub const UTF8_DONE: utf8_state = 1;
pub const UTF8_MORE: utf8_state = 0;
pub const BOX_LINES_NONE: box_lines = 6;
pub const BOX_LINES_PADDED: box_lines = 5;
pub const BOX_LINES_ROUNDED: box_lines = 4;
pub const BOX_LINES_SIMPLE: box_lines = 3;
pub const BOX_LINES_HEAVY: box_lines = 2;
pub const BOX_LINES_DOUBLE: box_lines = 1;
pub const BOX_LINES_SINGLE: box_lines = 0;
pub const BOX_LINES_DEFAULT: box_lines = -1;
pub const UINT_MAX: c_uint = (__INT_MAX__ as c_uint)
    .wrapping_mul(2 as c_uint)
    .wrapping_add(1 as c_uint);
pub const EV_TIMEOUT: c_int = 0x1;
pub const MODE_CURSOR: c_int = 0x1;
pub const MODE_INSERT: c_int = 0x2;
pub const MODE_WRAP: c_int = 0x10;
pub const MODE_ORIGIN: c_int = 0x2000;
pub const MODE_KEYS_EXTENDED: c_int = 0x8000;
pub const MODE_KEYS_EXTENDED_2: c_int = 0x40000;
pub const MODE_SYNC: c_int = 0x100000;
pub const EXTENDED_KEY_MODES: c_int = MODE_KEYS_EXTENDED | MODE_KEYS_EXTENDED_2;
pub const GRID_ATTR_DIM: c_int = 0x2;
pub const GRID_ATTR_REVERSE: c_int = 0x10;
pub const GRID_ATTR_CHARSET: c_int = 0x80;
pub const GRID_FLAG_PADDING: c_int = 0x4;
pub const GRID_FLAG_EXTENDED: c_int = 0x8;
pub const GRID_FLAG_SELECTED: c_int = 0x10;
pub const GRID_FLAG_NOPALETTE: c_int = 0x20;
pub const GRID_FLAG_CLEARED: c_int = 0x40;
pub const GRID_FLAG_TAB: c_int = 0x80;
pub const GRID_LINE_WRAPPED: c_int = 0x1;
pub const GRID_LINE_EXTENDED: c_int = 0x2;
pub const CELL_TOPBOTTOM: c_int = 1;
pub const CELL_LEFTRIGHT: c_int = 2;
pub const CELL_TOPLEFT: c_int = 3;
pub const CELL_TOPRIGHT: c_int = 4;
pub const CELL_BOTTOMLEFT: c_int = 5;
pub const CELL_BOTTOMRIGHT: c_int = 6;
pub const CELL_LEFTJOIN: c_int = 9;
pub const CELL_RIGHTJOIN: c_int = 10;
pub const CELL_BORDERS: [c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [c_char; 14]>(*b" xqlkmjwvtun~\0") };
pub const SIMPLE_BORDERS: [c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [c_char; 14]>(*b" |-+++++++++.\0") };
pub const PADDED_BORDERS: [c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [c_char; 14]>(*b"             \0") };
pub const GRID_HISTORY: c_int = 0x1;
pub const SCREEN_WRITE_SYNC: c_int = 0x1;
pub const SCREEN_WRITE_OBSCURED: c_int = 0x2;
pub const SCREEN_WRITE_CHECKED_IF_OBSCURED: c_int = 0x4;
pub const PANE_REDRAW: c_int = 0x1;
pub const PANE_DROP: c_int = 0x2;
pub const PANE_REDRAWSCROLLBAR: c_int = 0x8000;
pub const TTY_CTX_WRAPPED: c_int = 0x1;
pub const TTY_CTX_INVISIBLE_PANES: c_int = 0x2;
pub const TTY_CTX_WINDOW_BIGGER: c_int = 0x4;
pub const TTY_CTX_SYNC: c_int = 0x8;
pub const TTY_CTX_OVERLAY_SYNC: c_int = 0x10;
pub const TTY_CTX_CELL_INVALIDATE: c_int = 0x20;
pub const TTY_CTX_PANE_OBSCURED: c_int = 0x40;
pub const CLIENT_REDRAWPANES: c_int = 0x20000000;
/// Which collected item, as an index into the pool below. Items are never
/// given back to the allocator, so an index stays good for the life of the
/// server — which is what lets `screen_write_collect_trim` read the wrapped
/// flag of an item it has just given up.
pub type CItem = u32;

/// The index no item has, which a context carries before it is started and
/// after it is stopped.
pub const CITEM_NONE: CItem = CItem::MAX;

/// Every collected item the server has made. An item is never given back to
/// the allocator: it goes on the free list below and is handed out again, so
/// this only ever grows to the most items wanted at once.
static screen_write_citem_pool: GlobalQueue<screen_write_citem> = GlobalQueue::new();

/// The items given up so far, oldest first, which is the order they are
/// handed back out in.
pub static screen_write_citem_freelist: GlobalQueue<CItem> = GlobalQueue::new();

/// The items of one line, as a snapshot that a walk may take from.
unsafe fn citem_list(head: *mut citems) -> Vec<CItem> {
    unsafe { (*head).clone() }
}

/// The item `ci` names.
#[allow(clippy::mut_from_ref)]
pub(crate) fn citem(ci: CItem) -> &'static mut screen_write_citem {
    &mut screen_write_citem_pool.queue()[ci as usize]
}

/// Where `ci` sits in `head`, which is wherever it was put.
unsafe fn citem_position(head: *mut citems, ci: CItem) -> Option<usize> {
    unsafe { (*head).iter().position(|&item| item == ci) }
}

/// Puts `ci` at the end of `head`.
unsafe fn citem_insert_tail(head: *mut citems, ci: CItem) {
    unsafe { (*head).push(ci) }
}

/// Puts `ci` in front of `before`, which is already in `head`.
unsafe fn citem_insert_before(head: *mut citems, before: CItem, ci: CItem) {
    unsafe {
        let at = citem_position(head, before).expect("the anchor is on this line");
        (*head).insert(at, ci);
    }
}

/// Puts `ci` behind `after`, which is already in `head`.
unsafe fn citem_insert_after(head: *mut citems, after: CItem, ci: CItem) {
    unsafe {
        let at = citem_position(head, after).expect("the anchor is on this line");
        (*head).insert(at + 1, ci);
    }
}

/// Takes `ci` out of the list of collected items it hangs in, which is
/// `head`.
unsafe fn citem_remove(head: *mut citems, ci: CItem) {
    unsafe {
        if let Some(at) = citem_position(head, ci) {
            (*head).remove(at);
        }
    }
}

/// Moves every item of `src` to the end of the free list, leaving `src`
/// empty.
unsafe fn citem_free_all(src: *mut citems) {
    unsafe { screen_write_citem_freelist.queue().extend((*src).drain(..)) }
}

/// The screen's collect lists, one for each line of its grid.
fn write_list(s: &mut screen) -> &mut [screen_write_cline] {
    s.write_list.as_mut_slice()
}

/// A collected item to fill in: one off the free list, zeroed again, or a
/// fresh one when the free list is empty.
fn screen_write_get_citem() -> CItem {
    if let Some(ci) = screen_write_citem_freelist.queue().pop_front() {
        *citem(ci) = screen_write_citem::default();
        return ci;
    }
    let pool = screen_write_citem_pool.queue();
    pool.push_back(screen_write_citem::default());
    (pool.len() - 1) as CItem
}

/// Hands `ci` back to the free list. Nothing it carried is written over, so
/// it can still be read back afterwards — which `screen_write_collect_trim`
/// relies on for the wrapped flag.
fn screen_write_free_citem(ci: CItem) {
    screen_write_citem_freelist.queue().push_back(ci)
}
unsafe fn screen_write_offset_timer(w: *mut window) {
    if let Some(w_ref) = window_ref_from_ptr(w) {
        unsafe { tty_update_window_offset(w_ref.as_ptr()) }
    }
}

/// Moves the cursor, clamped to the screen, and arms the timer that works out
/// the window offsets again. A coordinate of -1 leaves that half alone.
unsafe fn screen_write_set_cursor(ctx: &mut screen_write_ctx, mut cx: c_int, mut cy: c_int) {
    unsafe {
        let wp: *mut window_pane = ctx.wp;
        let s: *mut screen = ctx.s;
        let mut tv = timeval::from_usecs(10000 as __suseconds_t);
        if cx != -1 && cx as u_int == (*s).cx && cy != -1 && cy as u_int == (*s).cy {
            return;
        }
        if cx != -1 {
            if cx as u_int > (*screen_grid_ptr(s)).sx {
                cx = (*screen_grid_ptr(s)).sx.wrapping_sub(1) as c_int;
            }
            (*s).cx = cx as u_int;
        }
        if cy != -1 {
            if cy as u_int > (*screen_grid_ptr(s)).sy.wrapping_sub(1) {
                cy = (*screen_grid_ptr(s)).sy.wrapping_sub(1) as c_int;
            }
            (*s).cy = cy as u_int;
        }
        if wp.is_null() {
            return;
        }
        let w: *mut window = (*wp).window;
        if !(*w).offset_timer.is_set() {
            let w_weak = window_ref_from_ptr(w).map(|w_ref| w_ref.downgrade());
            (*w).offset_timer.set_callback(move || {
                let Some(w_ref) = w_weak.as_ref().and_then(WindowWeak::upgrade) else {
                    return;
                };
                screen_write_offset_timer(w_ref.as_ptr());
            });
        }
        if !(*w).offset_timer.is_armed() {
            (*w).offset_timer.arm(tv);
        }
    }
}
unsafe fn screen_write_redraw_cb(ttyctx: &tty_ctx) {
    unsafe {
        let TtyCtxArg::Pane(wp) = ttyctx.arg else {
            return;
        };
        if !wp.is_null() {
            (*wp).flags |= PANE_REDRAW;
        }
    }
}
unsafe fn screen_write_set_client_cb(ttyctx: &mut tty_ctx, c: *mut client) -> c_int {
    unsafe {
        let TtyCtxArg::Pane(wp) = ttyctx.arg else {
            return 0;
        };
        if ttyctx.flags & TTY_CTX_INVISIBLE_PANES != 0 {
            if session_has((*c).session, (*wp).window) != 0 {
                return 1;
            }
            return 0;
        }
        if (*session_get_curw((*c).session)).window() != (*wp).window {
            return 0;
        }
        if (*wp).layout_cell.is_null() {
            return 0;
        }
        if (*wp).flags & (PANE_REDRAW | PANE_DROP) != 0 {
            return -1;
        }
        if (*c).flags & CLIENT_REDRAWPANES as uint64_t != 0 {
            log_debug(
                c"%s: adding %%%u to deferred redraw".as_ptr(),
                fmt_args![c"screen_write_set_client_cb".as_ptr(), (*wp).id],
            );
            (*wp).flags |= PANE_REDRAW | PANE_REDRAWSCROLLBAR;
            return -1;
        }
        let (bigger, ox, oy, sx, sy) = tty_window_offset(&raw mut (*c).tty);
        (ttyctx.wox, ttyctx.woy, ttyctx.wsx, ttyctx.wsy) = (ox, oy, sx, sy);
        if bigger != 0 {
            ttyctx.flags |= TTY_CTX_WINDOW_BIGGER;
        } else {
            ttyctx.flags &= !TTY_CTX_WINDOW_BIGGER;
        }
        ttyctx.rxoff = (*wp).xoff;
        ttyctx.xoff = ttyctx.rxoff;
        ttyctx.ryoff = (*wp).yoff;
        ttyctx.yoff = ttyctx.ryoff;
        if status_at_line(c) == 0 {
            ttyctx.yoff = (ttyctx.yoff as u_int).wrapping_add(status_line_size(c)) as c_int;
        }
        1
    }
}

/// The pane in front of `wp` in its window's pane list, or null at the head.
unsafe fn pane_prev(wp: *mut window_pane) -> *mut window_pane {
    unsafe { window_pane_stack_prev((*wp).window, PaneStack::ZIndex, wp) }
}

/// Whether anything is in front of the pane being written to: it hangs off
/// its window, or a floating pane in front of it overlaps it. The answer is
/// worked out once and kept in the context's flags.
unsafe fn screen_write_pane_is_obscured(ctx: &mut screen_write_ctx) -> c_int {
    unsafe {
        let base: *mut window_pane = ctx.wp;
        if base.is_null() {
            return 0;
        }
        if ctx.flags & SCREEN_WRITE_CHECKED_IF_OBSCURED != 0 {
            if ctx.flags & SCREEN_WRITE_OBSCURED != 0 {
                return 1;
            }
            return 0;
        }
        ctx.flags |= SCREEN_WRITE_CHECKED_IF_OBSCURED;
        let w = &*(*base).window;
        let b = &*base;
        if b.xoff < 0
            || b.yoff < 0
            || (b.xoff as u_int).wrapping_add(b.sx) > w.sx
            || (b.yoff as u_int).wrapping_add(b.sy) > w.sy
        {
            ctx.flags |= SCREEN_WRITE_OBSCURED;
            return 1;
        }
        let mut wp = base;
        loop {
            wp = pane_prev(wp);
            if wp.is_null() {
                break;
            }
            let f = &*wp;
            if window_pane_is_floating(wp) != 0
                && (f.yoff >= b.yoff && f.yoff <= b.yoff + b.sy as c_int
                    || f.yoff + f.sy as c_int >= b.yoff
                        && (f.yoff as u_int).wrapping_add(f.sy)
                            <= (b.yoff as u_int).wrapping_add(b.sy))
                && (f.xoff >= b.xoff && f.xoff <= b.xoff + b.sx as c_int
                    || f.xoff + f.sx as c_int >= b.xoff
                        && (f.xoff as u_int).wrapping_add(f.sx)
                            <= (b.xoff as u_int).wrapping_add(b.sx))
            {
                ctx.flags |= SCREEN_WRITE_OBSCURED;
                return 1;
            }
        }
        0
    }
}

/// Fills in a terminal context from the screen being written to, and starts a
/// synchronised update if this is the first call on this context.
unsafe fn screen_write_initctx(
    ctx: &mut screen_write_ctx,
    ttyctx: &mut tty_ctx,
    is_sync: c_int,
    check_obscured: c_int,
) {
    unsafe {
        let s: *mut screen = ctx.s;
        *ttyctx = tty_ctx::default();
        ttyctx.s = s;
        ttyctx.sx = (*screen_grid_ptr(s)).sx;
        ttyctx.sy = (*screen_grid_ptr(s)).sy;
        ttyctx.ocx = (*s).cx;
        ttyctx.ocy = (*s).cy;
        ttyctx.orlower = (*s).rlower;
        ttyctx.orupper = (*s).rupper;
        if check_obscured != 0 && screen_write_pane_is_obscured(&mut *ctx) != 0 {
            ttyctx.flags |= TTY_CTX_PANE_OBSCURED;
        }
        ttyctx.defaults = grid_default_cell;
        if let Some(cb) = ctx.init_ctx_cb {
            cb(ctx, &mut *ttyctx);
            if !ttyctx.palette.is_null() {
                if ttyctx.defaults.fg == 8 {
                    ttyctx.defaults.fg = (*ttyctx.palette).fg;
                }
                if ttyctx.defaults.bg == 8 {
                    ttyctx.defaults.bg = (*ttyctx.palette).bg;
                }
            }
        } else {
            ttyctx.redraw_cb = Some(screen_write_redraw_cb);
            let wp = ctx.wp;
            if !wp.is_null() {
                tty_default_colours(&raw mut ttyctx.defaults, wp);
                ttyctx.palette = &raw mut (*wp).palette;
                ttyctx.set_client_cb = Some(screen_write_set_client_cb);
                ttyctx.arg = TtyCtxArg::Pane(wp);
            }
        }
        if !ctx.flags & SCREEN_WRITE_SYNC != 0 {
            let wp = ctx.wp;
            if !wp.is_null() && wp != window_get_active((*wp).window) {
                ttyctx.flags |= TTY_CTX_SYNC;
            } else {
                if wp.is_null() {
                    ttyctx.flags |= TTY_CTX_OVERLAY_SYNC;
                }
                if is_sync != 0 {
                    ttyctx.flags |= TTY_CTX_SYNC;
                }
            }
            tty_write(Some(tty_cmd_syncstart), &mut *ttyctx);
            ctx.flags |= SCREEN_WRITE_SYNC;
        }
    }
}

/// Makes the screen's collect lists, one for each line of its grid.
pub unsafe fn screen_write_make_list(s: *mut screen) {
    unsafe {
        let mut list: Vec<screen_write_cline> =
            Vec::with_capacity((*screen_grid_ptr(s)).sy as usize);
        for _ in 0..(*screen_grid_ptr(s)).sy as usize {
            list.push(screen_write_cline {
                data: None,
                items: citems::new(),
            });
        }
        (*s).write_list = list;
    }
}

/// Gives up the screen's collect lists and the text they collected. What a
/// line still holds goes back to the free list: `screen_write_collect_flush`
/// leaves behind whatever a pane in front of this one hid, so a screen can be
/// freed with items still collected.
pub unsafe fn screen_write_free_list(s: *mut screen) {
    unsafe {
        for cl in write_list(&mut *s) {
            citem_free_all(&raw mut cl.items);
            cl.data = None;
        }
        (*s).write_list = Vec::new();
    }
}
unsafe fn screen_write_init(ctx: &mut screen_write_ctx, s: *mut screen) {
    unsafe {
        *ctx = screen_write_ctx::default();
        ctx.s = s;
        if (*s).write_list.is_empty() {
            screen_write_make_list(s);
        }
        ctx.item = screen_write_get_citem();
        ctx.scrolled = 0;
        ctx.bg = 8;
    }
}
pub unsafe fn screen_write_start_pane(
    ctx: &mut screen_write_ctx,
    mut wp: *mut window_pane,
    mut s: *mut screen,
) {
    unsafe {
        if s.is_null() {
            s = (*wp).screen();
        }
        screen_write_init(&mut *ctx, s);
        ctx.wp = wp;
        if log_get_level() != 0 {
            log_debug(
                c"%s: size %ux%u, pane %%%u (at %u,%u)".as_ptr(),
                fmt_args![
                    c"screen_write_start_pane".as_ptr(),
                    (*screen_grid_ptr(ctx.s)).sx,
                    (*screen_grid_ptr(ctx.s)).sy,
                    (*wp).id,
                    (*wp).xoff,
                    (*wp).yoff
                ],
            );
        }
    }
}
pub unsafe fn screen_write_start_callback(
    ctx: &mut screen_write_ctx,
    mut s: *mut screen,
    mut cb: screen_write_init_ctx_cb,
    mut arg: *mut popup_data,
) {
    unsafe {
        screen_write_init(&mut *ctx, s);
        ctx.init_ctx_cb = cb;
        ctx.arg = arg;
        if log_get_level() != 0 {
            log_debug(
                c"%s: size %ux%u, with callback".as_ptr(),
                fmt_args![
                    c"screen_write_start_callback".as_ptr(),
                    (*screen_grid_ptr(ctx.s)).sx,
                    (*screen_grid_ptr(ctx.s)).sy
                ],
            );
        }
    }
}
pub unsafe fn screen_write_start(ctx: &mut screen_write_ctx, mut s: *mut screen) {
    unsafe {
        screen_write_init(&mut *ctx, s);
        if log_get_level() != 0 {
            log_debug(
                c"%s: size %ux%u, no pane".as_ptr(),
                fmt_args![
                    c"screen_write_start".as_ptr(),
                    (*screen_grid_ptr(ctx.s)).sx,
                    (*screen_grid_ptr(ctx.s)).sy
                ],
            );
        }
    }
}
pub unsafe fn screen_write_stop(ctx: &mut screen_write_ctx) {
    unsafe {
        screen_write_collect_end(&mut *ctx);
        screen_write_collect_flush(&mut *ctx, 0, c"screen_write_stop".as_ptr());
        screen_write_free_citem(ctx.item);
    }
}
pub unsafe fn screen_write_reset(ctx: &mut screen_write_ctx) {
    unsafe {
        let mut s: *mut screen = ctx.s;
        screen_reset_tabs(s);
        screen_write_scrollregion(&mut *ctx, 0, (*screen_grid_ptr(s)).sy.wrapping_sub(1));
        (*s).mode = MODE_CURSOR | MODE_WRAP;
        if options_get_number(global_options, c"extended-keys".as_ptr())
            == 2 as ::core::ffi::c_longlong
        {
            (*s).mode = (*s).mode & !EXTENDED_KEY_MODES | MODE_KEYS_EXTENDED;
        }
        screen_write_clearscreen(&mut *ctx, 8);
        screen_write_set_cursor(&mut *ctx, 0, 0);
    }
}
pub unsafe fn screen_write_putc(ctx: &mut screen_write_ctx, gcp: *const grid_cell, ch: u_char) {
    unsafe {
        let mut gc = *gcp;
        utf8_set(&mut gc.data, ch);
        screen_write_cell(&mut *ctx, &raw mut gc);
    }
}

/// Whether a byte is one the writing calls put on the screen as itself.
fn is_printable(b: u8) -> bool {
    b == b'\t' || (0x20..0x7f).contains(&b)
}

/// How wide `text` will be on the screen.
pub unsafe fn screen_write_strlen(fmt: *const c_char, args: &[FmtArg]) -> size_t {
    unsafe {
        let msg = format_alloc(fmt, args);
        let bytes = msg.as_bytes();
        let mut size: size_t = 0;
        let mut i = 0;
        while i < bytes.len() {
            let mut ud = utf8_data::default();
            let b = bytes[i];
            if b > 0x7f && utf8_open(&mut ud, b) == UTF8_MORE {
                i += 1;
                if bytes.len() - i < (ud.size as usize).wrapping_sub(1) {
                    break;
                }
                let mut more = UTF8_MORE;
                while {
                    more = utf8_append(&mut ud, bytes[i]);
                    more == UTF8_MORE
                } {
                    i += 1;
                }
                i += 1;
                if more == UTF8_DONE {
                    size = size.wrapping_add(ud.width as size_t);
                }
            } else {
                if is_printable(b) {
                    size = size.wrapping_add(1);
                }
                i += 1;
            }
        }
        size
    }
}
pub unsafe fn screen_write_text(
    ctx: &mut screen_write_ctx,
    mut cx: u_int,
    mut width: u_int,
    mut lines: u_int,
    mut more: c_int,
    mut gcp: *const grid_cell,
    mut fmt: *const c_char,
    args: &[FmtArg],
) -> c_int {
    unsafe {
        let s: *mut screen = ctx.s;
        let cy: u_int = (*s).cy;
        let mut idx: usize = 0;
        let mut gc = *gcp;
        let tmp = format_alloc(fmt, args);
        let mut owned = utf8_fromcstr(tmp.as_ptr());
        owned.push(utf8_data::default());
        let text = &owned;
        let is = |i: usize, ch: u8| text[i].size == 1 && text[i].data[0] == ch;
        let mut left: u_int = cx.wrapping_add(width).wrapping_sub((*s).cx);
        loop {
            let mut at: u_int = 0;
            let mut end = idx;
            while text[end].size != 0 {
                if is(end, b'\n') {
                    break;
                }
                if at.wrapping_add(text[end].width as u_int) > left {
                    break;
                }
                at = at.wrapping_add(text[end].width as u_int);
                end += 1;
            }
            let next;
            if text[end].size == 0 {
                next = end;
            } else if is(end, b'\n') || is(end, b' ') {
                next = end + 1;
            } else {
                let mut i = end;
                while i > idx && !is(i, b' ') {
                    i -= 1;
                }
                if i != idx {
                    next = i + 1;
                    end = i;
                } else {
                    next = end;
                }
            }
            for i in idx..end {
                utf8_copy(&mut gc.data, &text[i]);
                screen_write_cell(&mut *ctx, &raw mut gc);
            }
            idx = next;
            if (*s).cy == cy.wrapping_add(lines).wrapping_sub(1) || text[idx].size == 0 {
                break;
            }
            screen_write_cursormove(&mut *ctx, cx as c_int, (*s).cy.wrapping_add(1) as c_int, 0);
            left = width;
        }
        let at_last_line = (*s).cy == cy.wrapping_add(lines).wrapping_sub(1);
        let finished = more == 0 || (*s).cx == cx.wrapping_add(width);
        let more_left = text[idx].size != 0;
        if at_last_line && finished || more_left {
            return 0;
        }
        if finished {
            screen_write_cursormove(&mut *ctx, cx as c_int, (*s).cy.wrapping_add(1) as c_int, 0);
        }
        1
    }
}
pub unsafe fn screen_write_puts(
    ctx: &mut screen_write_ctx,
    gcp: *const grid_cell,
    fmt: *const c_char,
    args: &[FmtArg],
) {
    unsafe { screen_write_vnputs(&mut *ctx, -1, gcp, fmt, args) }
}
pub unsafe fn screen_write_nputs(
    ctx: &mut screen_write_ctx,
    maxlen: ssize_t,
    gcp: *const grid_cell,
    fmt: *const c_char,
    args: &[FmtArg],
) {
    unsafe { screen_write_vnputs(&mut *ctx, maxlen, gcp, fmt, args) }
}

/// Writes a formatted string, stopping at `maxlen` columns when that is not
/// negative; the columns a character that would not fit leaves behind are
/// filled with spaces.
pub unsafe fn screen_write_vnputs(
    ctx: &mut screen_write_ctx,
    maxlen: ssize_t,
    gcp: *const grid_cell,
    fmt: *const c_char,
    args: &[FmtArg],
) {
    unsafe {
        let mut gc = *gcp;
        let ud: *mut utf8_data = &raw mut gc.data;
        let msg = format_alloc(fmt, args);
        let bytes = msg.as_bytes();
        let mut size: size_t = 0;
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            if b > 0x7f && utf8_open(&mut *ud, b) == UTF8_MORE {
                i += 1;
                if bytes.len() - i < ((*ud).size as usize).wrapping_sub(1) {
                    break;
                }
                let mut more = UTF8_MORE;
                while {
                    more = utf8_append(&mut *ud, bytes[i]);
                    more == UTF8_MORE
                } {
                    i += 1;
                }
                i += 1;
                if more != UTF8_DONE {
                    continue;
                }
                if maxlen > 0 && size.wrapping_add((*ud).width as size_t) > maxlen as size_t {
                    while size < maxlen as size_t {
                        screen_write_putc(&mut *ctx, &raw mut gc, b' ');
                        size = size.wrapping_add(1);
                    }
                    break;
                }
                size = size.wrapping_add((*ud).width as size_t);
                screen_write_cell(&mut *ctx, &raw mut gc);
            } else {
                if maxlen > 0 && size.wrapping_add(1) > maxlen as size_t {
                    break;
                }
                if b == 0x01 {
                    gc.attr = (gc.attr as c_int ^ GRID_ATTR_CHARSET) as u_short;
                } else if b == b'\n' {
                    screen_write_linefeed(&mut *ctx, 0, 8);
                    screen_write_carriagereturn(&mut *ctx);
                } else if is_printable(b) {
                    size = size.wrapping_add(1);
                    screen_write_putc(&mut *ctx, &raw mut gc, b);
                }
                i += 1;
            }
        }
    }
}
/// Copies a rectangle of another screen onto this one and writes it straight
/// out, without collecting it.
pub unsafe fn screen_write_fast_copy(
    ctx: &mut screen_write_ctx,
    src: *mut screen,
    px: u_int,
    py: u_int,
    nx: u_int,
    ny: u_int,
) {
    unsafe {
        let s: *mut screen = ctx.s;
        let wp: *mut window_pane = ctx.wp;
        let gd: *mut grid = screen_grid_ptr(src);
        let mut ttyctx = tty_ctx::default();
        let mut gc = grid_cell::default();
        let (cx, cy) = ((*s).cx, (*s).cy);
        if nx == 0 || ny == 0 {
            return;
        }
        let (xoff, yoff) = if wp.is_null() {
            (0, 0)
        } else {
            ((*wp).xoff, (*wp).yoff)
        };
        let mut yy = py;
        while yy < py.wrapping_add(ny) {
            if yy >= (*gd).hsize.wrapping_add((*gd).sy) {
                break;
            }
            (*s).cx = cx;
            screen_write_initctx(&mut *ctx, &mut ttyctx, 0, 0);
            let mut ranges = visible_ranges::default();
            screen_redraw_get_visible_ranges(
                wp,
                (xoff as u_int).wrapping_add((*s).cx) as c_int,
                (*s).cy.wrapping_add(yoff as u_int) as c_int,
                nx,
                &mut ranges,
            );
            let r = &raw mut ranges;
            let mut xx = px;
            while xx < px.wrapping_add(nx) {
                let gl = grid_get_line(&mut *gd, yy);
                let sgl = grid_get_line(screen_grid_mut(&mut *s), (*s).cy);
                if xx >= (*gl).cellsize() && (*s).cx >= (*sgl).cellsize() {
                    break;
                }
                gc = grid_get_cell(&*gd, xx, yy);
                if xx.wrapping_add(gc.data.width as u_int) > px.wrapping_add(nx) {
                    break;
                }
                grid_view_set_cell(screen_grid_mut(&mut *s), (*s).cx, (*s).cy, &gc);
                if screen_redraw_is_visible(r, (xoff as u_int).wrapping_add((*s).cx)) == 0 {
                    break;
                }
                ttyctx.cell = &raw mut gc;
                ttyctx.flags &= TTY_CTX_OVERLAY_SYNC | TTY_CTX_SYNC;
                tty_write(Some(tty_cmd_cell), &mut ttyctx);
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
}

/// Puts the character a border of `lines` draws for `cell_type` into `gc`.
fn screen_write_box_border_set(lines: box_lines, cell_type: c_int, gc: &mut grid_cell) {
    unsafe {
        let acs = match lines {
            BOX_LINES_DOUBLE => Some(tty_acs_double_borders(cell_type)),
            BOX_LINES_HEAVY => Some(tty_acs_heavy_borders(cell_type)),
            BOX_LINES_ROUNDED => Some(tty_acs_rounded_borders(cell_type)),
            _ => None,
        };
        if let Some(acs) = acs {
            gc.attr = (gc.attr as c_int & !GRID_ATTR_CHARSET) as u_short;
            utf8_copy(&mut gc.data, &*acs);
            return;
        }
        let table = match lines {
            BOX_LINES_SIMPLE => &SIMPLE_BORDERS,
            BOX_LINES_PADDED => &PADDED_BORDERS,
            BOX_LINES_SINGLE | BOX_LINES_DEFAULT => {
                gc.attr = (gc.attr as c_int | GRID_ATTR_CHARSET) as u_short;
                utf8_set(&mut gc.data, CELL_BORDERS[cell_type as usize] as u_char);
                return;
            }
            _ => return,
        };
        gc.attr = (gc.attr as c_int & !GRID_ATTR_CHARSET) as u_short;
        utf8_set(&mut gc.data, table[cell_type as usize] as u_char);
    }
}
/// Draws one edge of a box: a corner or join, the middle of the line and the
/// other corner or join.
unsafe fn box_edge(
    ctx: &mut screen_write_ctx,
    nx: u_int,
    lines: box_lines,
    gc: &mut grid_cell,
    left: c_int,
    right: c_int,
) {
    unsafe {
        screen_write_box_border_set(lines, left, gc);
        screen_write_cell(&mut *ctx, gc);
        screen_write_box_border_set(lines, CELL_LEFTRIGHT, gc);
        let mut i = 1;
        while i < nx.wrapping_sub(1) {
            screen_write_cell(&mut *ctx, gc);
            i = i.wrapping_add(1);
        }
        screen_write_box_border_set(lines, right, gc);
        screen_write_cell(&mut *ctx, gc);
    }
}

/// Draws a horizontal line of `nx` columns, joined at either end when asked.
pub unsafe fn screen_write_hline(
    ctx: &mut screen_write_ctx,
    nx: u_int,
    left: c_int,
    right: c_int,
    lines: box_lines,
    border_gc: Option<&grid_cell>,
) {
    unsafe {
        let s: *mut screen = ctx.s;
        let (cx, cy) = ((*s).cx, (*s).cy);
        let mut gc = border_gc.copied().unwrap_or(grid_default_cell);
        gc.attr = (gc.attr as c_int | GRID_ATTR_CHARSET) as u_short;
        let starts = if left != 0 {
            CELL_LEFTJOIN
        } else {
            CELL_LEFTRIGHT
        };
        let ends = if right != 0 {
            CELL_RIGHTJOIN
        } else {
            CELL_LEFTRIGHT
        };
        box_edge(&mut *ctx, nx, lines, &mut gc, starts, ends);
        screen_write_set_cursor(&mut *ctx, cx as c_int, cy as c_int);
    }
}

/// Draws a vertical line of `ny` lines, joined at either end when asked.
pub unsafe fn screen_write_vline(ctx: &mut screen_write_ctx, ny: u_int, top: c_int, bottom: c_int) {
    unsafe {
        let s: *mut screen = ctx.s;
        let (cx, cy) = ((*s).cx, (*s).cy);
        let mut gc = grid_default_cell;
        gc.attr = (gc.attr as c_int | GRID_ATTR_CHARSET) as u_short;
        screen_write_putc(&mut *ctx, &raw mut gc, if top != 0 { b'w' } else { b'x' });
        let mut i = 1;
        while i < ny.wrapping_sub(1) {
            screen_write_set_cursor(&mut *ctx, cx as c_int, cy.wrapping_add(i) as c_int);
            screen_write_putc(&mut *ctx, &raw mut gc, b'x');
            i = i.wrapping_add(1);
        }
        screen_write_set_cursor(
            &mut *ctx,
            cx as c_int,
            cy.wrapping_add(ny).wrapping_sub(1) as c_int,
        );
        screen_write_putc(
            &mut *ctx,
            &raw mut gc,
            if bottom != 0 { b'v' } else { b'x' },
        );
        screen_write_set_cursor(&mut *ctx, cx as c_int, cy as c_int);
    }
}

/// Draws a menu: a box with one line per item, the chosen one in its own
/// style and a name starting with `-` drawn dimmed and without that byte.
pub unsafe fn screen_write_menu(
    ctx: &mut screen_write_ctx,
    menu: *mut menu,
    choice: c_int,
    lines: box_lines,
    menu_gc: &grid_cell,
    border_gc: &grid_cell,
    choice_gc: &grid_cell,
) {
    unsafe {
        let s: *mut screen = ctx.s;
        let (cx, cy) = ((*s).cx, (*s).cy);
        let width: u_int = (*menu).width;
        let default_gc = *menu_gc;
        screen_write_box(
            &mut *ctx,
            (*menu).width.wrapping_add(4),
            ((*menu).items.len() as u_int).wrapping_add(2),
            lines,
            Some(border_gc),
            (*menu).title.as_deref(),
        );
        let mut i = 0;
        while (i as usize) < (*menu).items.len() {
            let name = (*menu).items[i as usize]
                .name
                .as_deref()
                .map(CStr::to_bytes);
            let line = cy.wrapping_add(1).wrapping_add(i) as c_int;
            match name {
                None => {
                    screen_write_cursormove(&mut *ctx, cx as c_int, line, 0);
                    screen_write_hline(
                        &mut *ctx,
                        width.wrapping_add(4),
                        1,
                        1,
                        lines,
                        Some(border_gc),
                    );
                }
                Some(name) => {
                    let dim = name.first() == Some(&b'-');
                    let mut gc = if choice >= 0 && i == choice as u_int && !dim {
                        *choice_gc
                    } else {
                        default_gc
                    };
                    screen_write_cursormove(&mut *ctx, cx.wrapping_add(1) as c_int, line, 0);
                    let mut j = 0;
                    while j < width.wrapping_add(2) {
                        screen_write_putc(&mut *ctx, &gc, b' ');
                        j = j.wrapping_add(1);
                    }
                    screen_write_cursormove(&mut *ctx, cx.wrapping_add(2) as c_int, line, 0);
                    if dim {
                        gc.attr = (gc.attr as c_int | GRID_ATTR_DIM) as u_short;
                        format_draw(&mut *ctx, &gc, width, &name[1..], None, 0);
                    } else {
                        format_draw(&mut *ctx, &gc, width, name, None, 0);
                    }
                }
            }
            i = i.wrapping_add(1);
        }
        screen_write_set_cursor(&mut *ctx, cx as c_int, cy as c_int);
    }
}

/// Draws a box of `nx` by `ny` with an optional title along its top.
pub unsafe fn screen_write_box(
    ctx: &mut screen_write_ctx,
    nx: u_int,
    ny: u_int,
    lines: box_lines,
    gcp: Option<&grid_cell>,
    title: Option<&CStr>,
) {
    unsafe {
        let s: *mut screen = ctx.s;
        let (cx, cy) = ((*s).cx, (*s).cy);
        let mut gc = gcp.copied().unwrap_or(grid_default_cell);
        gc.attr = (gc.attr as c_int | GRID_ATTR_CHARSET) as u_short;
        gc.flags = (gc.flags as c_int | GRID_FLAG_NOPALETTE) as u_char;
        box_edge(&mut *ctx, nx, lines, &mut gc, CELL_TOPLEFT, CELL_TOPRIGHT);
        screen_write_set_cursor(
            &mut *ctx,
            cx as c_int,
            cy.wrapping_add(ny).wrapping_sub(1) as c_int,
        );
        box_edge(
            &mut *ctx,
            nx,
            lines,
            &mut gc,
            CELL_BOTTOMLEFT,
            CELL_BOTTOMRIGHT,
        );
        screen_write_box_border_set(lines, CELL_TOPBOTTOM, &mut gc);
        let mut i = 1;
        while i < ny.wrapping_sub(1) {
            screen_write_set_cursor(&mut *ctx, cx as c_int, cy.wrapping_add(i) as c_int);
            screen_write_cell(&mut *ctx, &raw mut gc);
            screen_write_set_cursor(
                &mut *ctx,
                cx.wrapping_add(nx).wrapping_sub(1) as c_int,
                cy.wrapping_add(i) as c_int,
            );
            screen_write_cell(&mut *ctx, &raw mut gc);
            i = i.wrapping_add(1);
        }
        if let Some(title) = title {
            gc.attr = (gc.attr as c_int & !GRID_ATTR_CHARSET) as u_short;
            screen_write_cursormove(&mut *ctx, cx.wrapping_add(2) as c_int, cy as c_int, 0);
            format_draw(
                &mut *ctx,
                &gc,
                nx.wrapping_sub(4),
                title.to_bytes(),
                None,
                0,
            );
        }
        screen_write_set_cursor(&mut *ctx, cx as c_int, cy as c_int);
    }
}

/// Where a preview of `src` starts, so that the cursor is a third of the way
/// in and the preview stays inside the screen.
fn preview_offset(cursor: u_int, want: u_int, have: u_int) -> u_int {
    let mut at = if cursor < want.wrapping_div(3) {
        0
    } else {
        cursor.wrapping_sub(want.wrapping_div(3))
    };
    if at.wrapping_add(want) > have {
        at = if want > have {
            0
        } else {
            have.wrapping_sub(want)
        };
    }
    at
}

/// Draws a small copy of another screen, with its cursor shown in reverse.
pub unsafe fn screen_write_preview(
    ctx: &mut screen_write_ctx,
    src: *mut screen,
    nx: u_int,
    ny: u_int,
) {
    unsafe {
        let s: *mut screen = ctx.s;
        let (cx, cy) = ((*s).cx, (*s).cy);
        let mut gc = grid_cell::default();
        let (px, py) = if (*src).mode & MODE_CURSOR != 0 {
            (
                preview_offset((*src).cx, nx, (*screen_grid_ptr(src)).sx),
                preview_offset((*src).cy, ny, (*screen_grid_ptr(src)).sy),
            )
        } else {
            (0, 0)
        };
        screen_write_fast_copy(
            &mut *ctx,
            src,
            px,
            (*screen_grid_ptr(src)).hsize.wrapping_add(py),
            nx,
            ny,
        );
        if (*src).mode & MODE_CURSOR != 0 {
            gc = grid_view_get_cell(screen_grid(&*src), (*src).cx, (*src).cy);
            gc.attr = (gc.attr as c_int | GRID_ATTR_REVERSE) as u_short;
            screen_write_set_cursor(
                &mut *ctx,
                cx.wrapping_add((*src).cx.wrapping_sub(px)) as c_int,
                cy.wrapping_add((*src).cy.wrapping_sub(py)) as c_int,
            );
            screen_write_cell(&mut *ctx, &raw mut gc);
        }
    }
}
pub unsafe fn screen_write_mode_set(ctx: &mut screen_write_ctx, mut mode: c_int) {
    unsafe {
        let mut s: *mut screen = ctx.s;
        (*s).mode |= mode;
        if log_get_level() != 0 {
            log_debug(
                c"%s: %s".as_ptr(),
                fmt_args![
                    c"screen_write_mode_set".as_ptr(),
                    screen_mode_to_string(mode).as_c_str()
                ],
            );
        }
    }
}
pub unsafe fn screen_write_mode_clear(ctx: &mut screen_write_ctx, mut mode: c_int) {
    unsafe {
        let mut s: *mut screen = ctx.s;
        (*s).mode &= !mode;
        if log_get_level() != 0 {
            log_debug(
                c"%s: %s".as_ptr(),
                fmt_args![
                    c"screen_write_mode_clear".as_ptr(),
                    screen_mode_to_string(mode).as_c_str()
                ],
            );
        }
    }
}
unsafe fn screen_write_sync_callback(wp: *mut window_pane) {
    unsafe {
        log_debug(
            c"%s: %%%u sync timer expired".as_ptr(),
            fmt_args![c"screen_write_sync_callback".as_ptr(), (*wp).id],
        );
        (*wp).sync_timer.disarm();
        if (*wp).base.mode & MODE_SYNC != 0 {
            (*wp).base.mode &= !MODE_SYNC;
            (*wp).flags |= PANE_REDRAW;
        }
    }
}
pub unsafe fn screen_write_start_sync(mut wp: *mut window_pane) {
    unsafe {
        let mut tv = timeval::from_secs(1 as __time_t);
        if wp.is_null() {
            return;
        }
        (*wp).base.mode |= MODE_SYNC;
        if !(*wp).sync_timer.is_set() {
            (*wp)
                .sync_timer
                .set_callback(move || screen_write_sync_callback(wp));
        }
        (*wp).sync_timer.arm(tv);
        log_debug(
            c"%s: %%%u started sync mode".as_ptr(),
            fmt_args![c"screen_write_start_sync".as_ptr(), (*wp).id],
        );
    }
}
pub unsafe fn screen_write_stop_sync(mut wp: *mut window_pane) {
    unsafe {
        if wp.is_null() {
            return;
        }
        (*wp).sync_timer.disarm();
        (*wp).base.mode &= !MODE_SYNC;
        log_debug(
            c"%s: %%%u stopped sync mode".as_ptr(),
            fmt_args![c"screen_write_stop_sync".as_ptr(), (*wp).id],
        );
    }
}
pub unsafe fn screen_write_cursorup(ctx: &mut screen_write_ctx, mut ny: u_int) {
    unsafe {
        let mut s: *mut screen = ctx.s;
        let mut cx: u_int = (*s).cx;
        let mut cy: u_int = (*s).cy;
        if ny == 0 {
            ny = 1;
        }
        if cy < (*s).rupper {
            if ny > cy {
                ny = cy;
            }
        } else if ny > cy.wrapping_sub((*s).rupper) {
            ny = cy.wrapping_sub((*s).rupper);
        }
        if cx == (*screen_grid_ptr(s)).sx {
            cx = cx.wrapping_sub(1);
        }
        cy = cy.wrapping_sub(ny);
        screen_write_set_cursor(&mut *ctx, cx as c_int, cy as c_int);
    }
}
pub unsafe fn screen_write_cursordown(ctx: &mut screen_write_ctx, mut ny: u_int) {
    unsafe {
        let mut s: *mut screen = ctx.s;
        let mut cx: u_int = (*s).cx;
        let mut cy: u_int = (*s).cy;
        if ny == 0 {
            ny = 1;
        }
        if cy > (*s).rlower {
            if ny > (*screen_grid_ptr(s)).sy.wrapping_sub(1).wrapping_sub(cy) {
                ny = (*screen_grid_ptr(s)).sy.wrapping_sub(1).wrapping_sub(cy);
            }
        } else if ny > (*s).rlower.wrapping_sub(cy) {
            ny = (*s).rlower.wrapping_sub(cy);
        }
        if cx == (*screen_grid_ptr(s)).sx {
            cx = cx.wrapping_sub(1);
        } else if ny == 0 {
            return;
        }
        cy = cy.wrapping_add(ny);
        screen_write_set_cursor(&mut *ctx, cx as c_int, cy as c_int);
    }
}
pub unsafe fn screen_write_cursorright(ctx: &mut screen_write_ctx, mut nx: u_int) {
    unsafe {
        let mut s: *mut screen = ctx.s;
        let mut cx: u_int = (*s).cx;
        let mut cy: u_int = (*s).cy;
        if nx == 0 {
            nx = 1;
        }
        if nx > (*screen_grid_ptr(s)).sx.wrapping_sub(1).wrapping_sub(cx) {
            nx = (*screen_grid_ptr(s)).sx.wrapping_sub(1).wrapping_sub(cx);
        }
        if nx == 0 {
            return;
        }
        cx = cx.wrapping_add(nx);
        screen_write_set_cursor(&mut *ctx, cx as c_int, cy as c_int);
    }
}
pub unsafe fn screen_write_cursorleft(ctx: &mut screen_write_ctx, mut nx: u_int) {
    unsafe {
        let mut s: *mut screen = ctx.s;
        let mut cx: u_int = (*s).cx;
        let mut cy: u_int = (*s).cy;
        if nx == 0 {
            nx = 1;
        }
        if nx > cx {
            nx = cx;
        }
        if nx == 0 {
            return;
        }
        cx = cx.wrapping_sub(nx);
        screen_write_set_cursor(&mut *ctx, cx as c_int, cy as c_int);
    }
}
pub unsafe fn screen_write_backspace(ctx: &mut screen_write_ctx) {
    unsafe {
        let mut s: *mut screen = ctx.s;
        let mut gl: *mut grid_line = null_mut::<grid_line>();
        let mut cx: u_int = (*s).cx;
        let mut cy: u_int = (*s).cy;
        if cx == 0 {
            if cy == 0 {
                return;
            }
            gl = grid_get_line(
                screen_grid_mut(&mut *s),
                (*screen_grid_ptr(s)).hsize.wrapping_add(cy).wrapping_sub(1),
            );
            if (*gl).flags & GRID_LINE_WRAPPED != 0 {
                cy = cy.wrapping_sub(1);
                cx = (*screen_grid_ptr(s)).sx.wrapping_sub(1);
            }
        } else {
            cx = cx.wrapping_sub(1);
        }
        screen_write_set_cursor(&mut *ctx, cx as c_int, cy as c_int);
    }
}
/// Whether a cell holds one plain single-width character, which is the only
/// shape a redraw can write out as one cell rather than a whole line.
unsafe fn screen_write_cell_is_single(gc: *const grid_cell) -> c_int {
    let gc = unsafe { &*gc };
    let single = gc.data.width == 1
        && gc.data.size == 1
        && gc.data.data[0] >= 0x20
        && gc.data.data[0] != 0x7f
        && gc.flags as c_int & (GRID_FLAG_CLEARED | GRID_FLAG_PADDING | GRID_FLAG_TAB) == 0;
    single as c_int
}

/// Redraws the visible parts of one line of the pane, a cell at a time where
/// only one column is visible and the whole line otherwise.
///
/// Two guards the C had are gone with the conversion, each unreachable:
/// a range that starts past the pane's right edge (every range starts at or
/// after the pane's `xoff`, and the one arrangement whose start runs past
/// `xoff + sx` — the range cut by a pane in front of it — has `nx` zero,
/// which the guard above skips), and a count of zero (`nx` is not zero, and
/// the clamp only ever replaces it with `sx - cx`, which is at least one
/// since `cx` is less than `sx`).
unsafe fn screen_write_redraw_line(ctx: &mut screen_write_ctx, ttyctx: &mut tty_ctx, yy: u_int) {
    unsafe {
        let wp: *mut window_pane = ctx.wp;
        let s: *mut screen = ctx.s;
        let mut gc = grid_cell::default();
        let mut ngc = grid_cell::default();
        let sx: u_int = (*screen_grid_ptr(s)).sx;
        let (xoff, yoff) = ((*wp).xoff, (*wp).yoff);
        if (*s).mode & MODE_SYNC != 0 {
            return;
        }
        let mut ranges = visible_ranges::default();
        screen_redraw_get_visible_ranges(
            wp,
            xoff,
            (yoff as u_int).wrapping_add(yy) as c_int,
            sx,
            &mut ranges,
        );
        let r = &raw mut ranges;
        let mut i = 0;
        while i < (*r).used {
            let ri = (*r).ranges[i as usize];
            i = i.wrapping_add(1);
            if ri.nx == 0 {
                continue;
            }
            let cx = ri.px.wrapping_sub(xoff as u_int);
            let n = if cx.wrapping_add(ri.nx) > sx {
                sx.wrapping_sub(cx)
            } else {
                ri.nx
            };
            ttyctx.value = TtyCtxValue::Num(n);
            ttyctx.ocx = cx;
            ttyctx.ocy = yy;
            if n != 1 {
                tty_write(Some(tty_cmd_redrawline), &mut *ttyctx);
                continue;
            }
            gc = grid_view_get_cell(screen_grid(&*s), cx, yy);
            if screen_write_cell_is_single(&raw mut gc) == 0 {
                tty_write(Some(tty_cmd_redrawline), &mut *ttyctx);
                continue;
            }
            if !(gc.flags as c_int) & GRID_FLAG_SELECTED != 0 {
                ttyctx.cell = &raw mut gc;
            } else {
                screen_select_cell(s, &raw mut ngc, &raw mut gc);
                ttyctx.cell = &raw mut ngc;
            }
            tty_write(Some(tty_cmd_cell), &mut *ttyctx);
        }
    }
}
unsafe fn screen_write_redraw_pane(ctx: &mut screen_write_ctx, ttyctx: &mut tty_ctx) {
    unsafe {
        let mut s: *mut screen = ctx.s;
        let mut yy: u_int = 0;
        yy = 0;
        while yy < (*screen_grid_ptr(s)).sy {
            screen_write_redraw_line(&mut *ctx, ttyctx, yy);
            yy = yy.wrapping_add(1);
        }
    }
}
/// Whether something is in front of the pane the context writes to, which
/// `screen_write_initctx` works out when it is asked to.
fn pane_obscured(ttyctx: &tty_ctx) -> bool {
    ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0
}

/// Hands `cmd` to the terminal, or redraws the line under the cursor when a
/// pane in front of this one means the command cannot be written as it is.
unsafe fn write_or_redraw_line(
    ctx: &mut screen_write_ctx,
    ttyctx: &mut tty_ctx,
    cmd: unsafe fn(*mut tty, &tty_ctx),
) {
    unsafe {
        if !pane_obscured(&*ttyctx) || ctx.wp.is_null() {
            tty_write(Some(cmd), &mut *ttyctx);
            return;
        }
        let cy = (*ctx.s).cy;
        screen_write_redraw_line(ctx, ttyctx, cy);
    }
}

/// Hands `cmd` to the terminal, or redraws the whole pane when a pane in
/// front of this one means the command cannot be written as it is.
unsafe fn write_or_redraw_pane(
    ctx: &mut screen_write_ctx,
    ttyctx: &mut tty_ctx,
    cmd: unsafe fn(*mut tty, &tty_ctx),
) {
    unsafe {
        if !pane_obscured(&*ttyctx) || ctx.wp.is_null() {
            tty_write(Some(cmd), &mut *ttyctx);
            return;
        }
        screen_write_redraw_pane(&mut *ctx, ttyctx);
    }
}

/// Fills the screen with `E`, which is what the alignment test draws.
pub unsafe fn screen_write_alignmenttest(ctx: &mut screen_write_ctx) {
    unsafe {
        let s: *mut screen = ctx.s;
        let mut ttyctx = tty_ctx::default();
        let mut gc = grid_default_cell;
        utf8_set(&mut gc.data, b'E');
        for yy in 0..(*screen_grid_ptr(s)).sy {
            for xx in 0..(*screen_grid_ptr(s)).sx {
                grid_view_set_cell(screen_grid_mut(&mut *s), xx, yy, &gc);
            }
        }
        screen_write_set_cursor(&mut *ctx, 0, 0);
        (*s).rupper = 0;
        (*s).rlower = (*screen_grid_ptr(s)).sy.wrapping_sub(1);
        screen_write_collect_clear(&mut *ctx, 0, (*screen_grid_ptr(s)).sy.wrapping_sub(1));
        screen_write_initctx(&mut *ctx, &mut ttyctx, 1, 1);
        write_or_redraw_pane(&mut *ctx, &mut ttyctx, tty_cmd_alignmenttest);
    }
}

/// The number of columns a call that takes one may work on: at least one,
/// and never past the right of the screen.
///
/// The guard the C had for a cursor past the last column is gone with the
/// conversion: `screen_write_set_cursor` clamps anything over `sx` back, and
/// at `sx` exactly this clamp is zero, which the caller has already returned
/// on.
unsafe fn columns_left(s: *mut screen, nx: u_int) -> u_int {
    unsafe {
        let nx = if nx == 0 { 1 } else { nx };
        let left = (*screen_grid_ptr(s)).sx.wrapping_sub((*s).cx);
        if nx > left { left } else { nx }
    }
}

pub unsafe fn screen_write_insertcharacter(ctx: &mut screen_write_ctx, nx: u_int, bg: u_int) {
    unsafe {
        let s: *mut screen = ctx.s;
        let mut ttyctx = tty_ctx::default();
        let nx = columns_left(s, nx);
        if nx == 0 {
            return;
        }
        screen_write_initctx(&mut *ctx, &mut ttyctx, 0, 1);
        ttyctx.bg = bg;
        grid_view_insert_cells(screen_grid_mut(&mut *s), (*s).cx, (*s).cy, nx, bg);
        screen_write_collect_flush(&mut *ctx, 0, c"screen_write_insertcharacter".as_ptr());
        ttyctx.value = TtyCtxValue::Num(nx);
        write_or_redraw_line(&mut *ctx, &mut ttyctx, tty_cmd_insertcharacter);
    }
}
pub unsafe fn screen_write_deletecharacter(ctx: &mut screen_write_ctx, nx: u_int, bg: u_int) {
    unsafe {
        let s: *mut screen = ctx.s;
        let mut ttyctx = tty_ctx::default();
        let nx = columns_left(s, nx);
        if nx == 0 {
            return;
        }
        screen_write_initctx(&mut *ctx, &mut ttyctx, 0, 1);
        ttyctx.bg = bg;
        grid_view_delete_cells(screen_grid_mut(&mut *s), (*s).cx, (*s).cy, nx, bg);
        screen_write_collect_flush(&mut *ctx, 0, c"screen_write_deletecharacter".as_ptr());
        ttyctx.value = TtyCtxValue::Num(nx);
        write_or_redraw_line(&mut *ctx, &mut ttyctx, tty_cmd_deletecharacter);
    }
}
pub unsafe fn screen_write_clearcharacter(ctx: &mut screen_write_ctx, nx: u_int, bg: u_int) {
    unsafe {
        let s: *mut screen = ctx.s;
        let mut ttyctx = tty_ctx::default();
        let nx = columns_left(s, nx);
        if nx == 0 {
            return;
        }
        screen_write_initctx(&mut *ctx, &mut ttyctx, 0, 1);
        ttyctx.bg = bg;
        grid_view_clear(screen_grid_mut(&mut *s), (*s).cx, (*s).cy, nx, 1, bg);
        screen_write_collect_flush(&mut *ctx, 0, c"screen_write_clearcharacter".as_ptr());
        ttyctx.value = TtyCtxValue::Num(nx);
        write_or_redraw_line(&mut *ctx, &mut ttyctx, tty_cmd_clearcharacter);
    }
}

/// The number of lines a call that takes one may work on when the cursor sits
/// outside the scroll region: at least one, and never past the bottom of the
/// screen.
///
/// The guard the C had for a count of zero after this clamp is gone with the
/// conversion: the cursor is never below the last line, so the clamp is at
/// least one.
unsafe fn lines_left(s: *mut screen, ny: u_int) -> u_int {
    unsafe {
        let ny = if ny == 0 { 1 } else { ny };
        let left = (*screen_grid_ptr(s)).sy.wrapping_sub((*s).cy);
        if ny > left { left } else { ny }
    }
}

/// The number of lines a call that takes one may work on inside the scroll
/// region: at least one, and never past the bottom of the region.
///
/// As with `lines_left`, the C's guard for a count of zero is unreachable:
/// the cursor is inside the region, so the clamp is at least one.
unsafe fn lines_left_in_region(s: *mut screen, ny: u_int) -> u_int {
    unsafe {
        let ny = if ny == 0 { 1 } else { ny };
        let left = (*s).rlower.wrapping_add(1).wrapping_sub((*s).cy);
        if ny > left { left } else { ny }
    }
}

/// Whether the cursor sits outside the scroll region.
unsafe fn outside_region(s: *mut screen) -> bool {
    unsafe { (*s).cy < (*s).rupper || (*s).cy > (*s).rlower }
}

pub unsafe fn screen_write_insertline(ctx: &mut screen_write_ctx, ny: u_int, bg: u_int) {
    unsafe {
        let s: *mut screen = ctx.s;
        let gd: *mut grid = screen_grid_ptr(s);
        let mut ttyctx = tty_ctx::default();
        if outside_region(s) {
            let ny = lines_left(s, ny);
            screen_write_initctx(&mut *ctx, &mut ttyctx, 1, 1);
            ttyctx.bg = bg;
            grid_view_insert_lines(&mut *gd, (*s).cy, ny, bg);
            screen_write_collect_flush(&mut *ctx, 0, c"screen_write_insertline".as_ptr());
            ttyctx.value = TtyCtxValue::Num(ny);
            write_or_redraw_pane(&mut *ctx, &mut ttyctx, tty_cmd_insertline);
            return;
        }
        let ny = lines_left_in_region(s, ny);
        screen_write_initctx(&mut *ctx, &mut ttyctx, 1, 1);
        ttyctx.bg = bg;
        grid_view_insert_lines_region(&mut *gd, (*s).rlower, (*s).cy, ny, bg);
        screen_write_collect_flush(&mut *ctx, 0, c"screen_write_insertline".as_ptr());
        ttyctx.value = TtyCtxValue::Num(ny);
        write_or_redraw_pane(&mut *ctx, &mut ttyctx, tty_cmd_insertline);
    }
}
pub unsafe fn screen_write_deleteline(ctx: &mut screen_write_ctx, ny: u_int, bg: u_int) {
    unsafe {
        let s: *mut screen = ctx.s;
        let gd: *mut grid = screen_grid_ptr(s);
        let mut ttyctx = tty_ctx::default();
        if outside_region(s) {
            let ny = lines_left(s, ny);
            screen_write_initctx(&mut *ctx, &mut ttyctx, 1, 1);
            ttyctx.bg = bg;
            grid_view_delete_lines(&mut *gd, (*s).cy, ny, bg);
            screen_write_collect_flush(&mut *ctx, 0, c"screen_write_deleteline".as_ptr());
            ttyctx.value = TtyCtxValue::Num(ny);
            write_or_redraw_pane(&mut *ctx, &mut ttyctx, tty_cmd_deleteline);
            return;
        }
        let ny = lines_left_in_region(s, ny);
        screen_write_initctx(&mut *ctx, &mut ttyctx, 1, 1);
        ttyctx.bg = bg;
        grid_view_delete_lines_region(&mut *gd, (*s).rlower, (*s).cy, ny, bg);
        screen_write_collect_flush(&mut *ctx, 0, c"screen_write_deleteline".as_ptr());
        ttyctx.value = TtyCtxValue::Num(ny);
        write_or_redraw_pane(&mut *ctx, &mut ttyctx, tty_cmd_deleteline);
    }
}
pub unsafe fn screen_write_clearline(ctx: &mut screen_write_ctx, bg: u_int) {
    unsafe {
        let s: *mut screen = ctx.s;
        let sx: u_int = (*screen_grid_ptr(s)).sx;
        let ci: CItem = ctx.item;
        let gl = grid_get_line(
            screen_grid_mut(&mut *s),
            (*screen_grid_ptr(s)).hsize.wrapping_add((*s).cy),
        );
        if (*gl).cellsize() == 0 && (bg == 8 || bg == 9) {
            return;
        }
        grid_view_clear(screen_grid_mut(&mut *s), 0, (*s).cy, sx, 1, bg);
        screen_write_collect_clear(&mut *ctx, (*s).cy, 1);
        citem(ci).x = 0;
        citem(ci).used = sx;
        citem(ci).type_0 = CLEAR;
        citem(ci).bg = bg;
        let cy = (*s).cy as usize;
        citem_insert_tail(&raw mut write_list(&mut *s)[cy].items, ci);
        ctx.item = screen_write_get_citem();
    }
}
pub unsafe fn screen_write_clearendofline(ctx: &mut screen_write_ctx, bg: u_int) {
    unsafe {
        let s: *mut screen = ctx.s;
        let sx: u_int = (*screen_grid_ptr(s)).sx;
        let ci: CItem = ctx.item;
        if (*s).cx == 0 {
            screen_write_clearline(&mut *ctx, bg);
            return;
        }
        let gl = grid_get_line(
            screen_grid_mut(&mut *s),
            (*screen_grid_ptr(s)).hsize.wrapping_add((*s).cy),
        );
        if (*s).cx > sx.wrapping_sub(1) || (*s).cx >= (*gl).cellsize() && (bg == 8 || bg == 9) {
            return;
        }
        grid_view_clear(
            screen_grid_mut(&mut *s),
            (*s).cx,
            (*s).cy,
            sx.wrapping_sub((*s).cx),
            1,
            bg,
        );
        citem(ci).x = (*s).cx;
        citem(ci).used = sx.wrapping_sub((*s).cx);
        citem(ci).type_0 = CLEAR;
        citem(ci).bg = bg;
        screen_write_collect_insert(&mut *ctx, ci);
    }
}

/// Clears from the start of the line to the cursor.
///
/// The C cleared the whole line instead when the cursor was past the last
/// column; that arm is gone with the conversion, since a cursor at or after
/// the last column has already gone to `screen_write_clearline` above.
pub unsafe fn screen_write_clearstartofline(ctx: &mut screen_write_ctx, bg: u_int) {
    unsafe {
        let s: *mut screen = ctx.s;
        let sx: u_int = (*screen_grid_ptr(s)).sx;
        let ci: CItem = ctx.item;
        if (*s).cx >= sx.wrapping_sub(1) {
            screen_write_clearline(&mut *ctx, bg);
            return;
        }
        grid_view_clear(
            screen_grid_mut(&mut *s),
            0,
            (*s).cy,
            (*s).cx.wrapping_add(1),
            1,
            bg,
        );
        citem(ci).x = 0;
        citem(ci).used = (*s).cx.wrapping_add(1);
        citem(ci).type_0 = CLEAR;
        citem(ci).bg = bg;
        screen_write_collect_insert(&mut *ctx, ci);
    }
}
pub unsafe fn screen_write_cursormove(
    ctx: &mut screen_write_ctx,
    mut px: c_int,
    mut py: c_int,
    origin: c_int,
) {
    unsafe {
        let s: *mut screen = ctx.s;
        if origin != 0 && py != -1 && (*s).mode & MODE_ORIGIN != 0 {
            if py as u_int > (*s).rlower.wrapping_sub((*s).rupper) {
                py = (*s).rlower as c_int;
            } else {
                py = (py as u_int).wrapping_add((*s).rupper) as c_int;
            }
        }
        if px != -1 && px as u_int > (*screen_grid_ptr(s)).sx.wrapping_sub(1) {
            px = (*screen_grid_ptr(s)).sx.wrapping_sub(1) as c_int;
        }
        if py != -1 && py as u_int > (*screen_grid_ptr(s)).sy.wrapping_sub(1) {
            py = (*screen_grid_ptr(s)).sy.wrapping_sub(1) as c_int;
        }
        log_debug(
            c"%s: from %u,%u to %u,%u".as_ptr(),
            fmt_args![
                c"screen_write_cursormove".as_ptr(),
                (*s).cx,
                (*s).cy,
                px,
                py
            ],
        );
        screen_write_set_cursor(&mut *ctx, px, py);
    }
}
pub unsafe fn screen_write_reverseindex(ctx: &mut screen_write_ctx, bg: u_int) {
    unsafe {
        let s: *mut screen = ctx.s;
        let mut ttyctx = tty_ctx::default();
        if (*s).cy == (*s).rupper {
            grid_view_scroll_region_down(screen_grid_mut(&mut *s), (*s).rupper, (*s).rlower, bg);
            screen_write_collect_flush(&mut *ctx, 0, c"screen_write_reverseindex".as_ptr());
            screen_write_initctx(&mut *ctx, &mut ttyctx, 1, 1);
            ttyctx.bg = bg;
            write_or_redraw_pane(&mut *ctx, &mut ttyctx, tty_cmd_reverseindex);
        } else if (*s).cy > 0 {
            screen_write_set_cursor(&mut *ctx, -1, (*s).cy.wrapping_sub(1) as c_int);
        }
    }
}
pub unsafe fn screen_write_scrollregion(
    ctx: &mut screen_write_ctx,
    mut rupper: u_int,
    mut rlower: u_int,
) {
    unsafe {
        let s: *mut screen = ctx.s;
        if rupper > (*screen_grid_ptr(s)).sy.wrapping_sub(1) {
            rupper = (*screen_grid_ptr(s)).sy.wrapping_sub(1);
        }
        if rlower > (*screen_grid_ptr(s)).sy.wrapping_sub(1) {
            rlower = (*screen_grid_ptr(s)).sy.wrapping_sub(1);
        }
        if rupper >= rlower {
            return;
        }
        screen_write_collect_flush(&mut *ctx, 0, c"screen_write_scrollregion".as_ptr());
        screen_write_set_cursor(&mut *ctx, 0, 0);
        (*s).rupper = rupper;
        (*s).rlower = rlower;
    }
}
pub unsafe fn screen_write_linefeed(ctx: &mut screen_write_ctx, wrapped: c_int, bg: u_int) {
    unsafe {
        let s: *mut screen = ctx.s;
        let gd: *mut grid = screen_grid_ptr(s);
        let gl = grid_get_line(&mut *gd, (*gd).hsize.wrapping_add((*s).cy));
        if wrapped != 0 {
            gl.flags |= GRID_LINE_WRAPPED;
        }
        log_debug(
            c"%s: at %u,%u (region %u-%u)".as_ptr(),
            fmt_args![
                c"screen_write_linefeed".as_ptr(),
                (*s).cx,
                (*s).cy,
                (*s).rupper,
                (*s).rlower
            ],
        );
        if bg != ctx.bg {
            screen_write_collect_flush(&mut *ctx, 1, c"screen_write_linefeed".as_ptr());
            ctx.bg = bg;
        }
        if (*s).cy == (*s).rlower {
            grid_view_scroll_region_up(&mut *gd, (*s).rupper, (*s).rlower, bg);
            screen_write_collect_scroll(&mut *ctx, bg);
            ctx.scrolled = ctx.scrolled.wrapping_add(1);
        } else if (*s).cy < (*screen_grid_ptr(s)).sy.wrapping_sub(1) {
            screen_write_set_cursor(&mut *ctx, -1, (*s).cy.wrapping_add(1) as c_int);
        }
    }
}

/// The number of lines a scroll may move: at least one, and never more than
/// the scroll region holds.
unsafe fn lines_in_region(s: *mut screen, lines: u_int) -> u_int {
    unsafe {
        let region = (*s).rlower.wrapping_sub((*s).rupper).wrapping_add(1);
        if lines == 0 {
            1
        } else if lines > region {
            region
        } else {
            lines
        }
    }
}

pub unsafe fn screen_write_scrollup(ctx: &mut screen_write_ctx, lines: u_int, bg: u_int) {
    unsafe {
        let s: *mut screen = ctx.s;
        let gd: *mut grid = screen_grid_ptr(s);
        let lines = lines_in_region(s, lines);
        if bg != ctx.bg {
            screen_write_collect_flush(&mut *ctx, 1, c"screen_write_scrollup".as_ptr());
            ctx.bg = bg;
        }
        for _ in 0..lines {
            grid_view_scroll_region_up(&mut *gd, (*s).rupper, (*s).rlower, bg);
            screen_write_collect_scroll(&mut *ctx, bg);
        }
        ctx.scrolled = ctx.scrolled.wrapping_add(lines);
    }
}
pub unsafe fn screen_write_scrolldown(ctx: &mut screen_write_ctx, lines: u_int, bg: u_int) {
    unsafe {
        let s: *mut screen = ctx.s;
        let gd: *mut grid = screen_grid_ptr(s);
        let mut ttyctx = tty_ctx::default();
        screen_write_initctx(&mut *ctx, &mut ttyctx, 1, 1);
        ttyctx.bg = bg;
        let lines = lines_in_region(s, lines);
        for _ in 0..lines {
            grid_view_scroll_region_down(&mut *gd, (*s).rupper, (*s).rlower, bg);
        }
        screen_write_collect_flush(&mut *ctx, 0, c"screen_write_scrolldown".as_ptr());
        ttyctx.value = TtyCtxValue::Num(lines);
        write_or_redraw_pane(&mut *ctx, &mut ttyctx, tty_cmd_scrolldown);
    }
}
pub unsafe fn screen_write_carriagereturn(ctx: &mut screen_write_ctx) {
    unsafe { screen_write_set_cursor(&mut *ctx, 0, -1) }
}

/// Where the pane being written to sits in its window.
///
/// This is only asked for once a clear has found the pane obscured, which
/// `screen_write_pane_is_obscured` only answers for a context that has a
/// pane; the C's arm for a context without one is gone with the conversion,
/// and it only set offsets that were already zero.
unsafe fn pane_offset(ctx: &mut screen_write_ctx) -> (u_int, u_int) {
    unsafe {
        let wp = ctx.wp;
        ((*wp).xoff as u_int, (*wp).yoff as u_int)
    }
}

/// Collects a clear of whatever is visible of `nx` columns from `px` on line
/// `y`, which is what a clear falls back to when a pane in front of this one
/// means the terminal cannot be told to clear the line itself.
unsafe fn collect_visible_clear(
    ctx: &mut screen_write_ctx,
    (xoff, yoff): (u_int, u_int),
    y: u_int,
    px: u_int,
    nx: u_int,
    bg: u_int,
) {
    unsafe {
        let mut ranges = visible_ranges::default();
        screen_redraw_get_visible_ranges(
            ctx.wp,
            xoff.wrapping_add(px) as c_int,
            yoff.wrapping_add(y) as c_int,
            nx,
            &mut ranges,
        );
        let r = &raw mut ranges;
        for i in 0..(*r).used {
            let ri = (*r).ranges[i as usize];
            if ri.nx != 0 {
                screen_write_collect_insert_clear(&mut *ctx, ri.px.wrapping_sub(xoff), ri.nx, bg);
            }
        }
    }
}

pub unsafe fn screen_write_clearendofscreen(ctx: &mut screen_write_ctx, bg: u_int) {
    unsafe {
        let s: *mut screen = ctx.s;
        let gd: *mut grid = screen_grid_ptr(s);
        let mut ttyctx = tty_ctx::default();
        let sx: u_int = (*screen_grid_ptr(s)).sx;
        let sy: u_int = (*screen_grid_ptr(s)).sy;
        screen_write_initctx(&mut *ctx, &mut ttyctx, 1, 1);
        ttyctx.bg = bg;
        if (*s).cx == 0
            && (*s).cy == 0
            && (*gd).flags & GRID_HISTORY != 0
            && !ctx.wp.is_null()
            && options_get_number(options_ptr(&(*ctx.wp).options), c"scroll-on-clear".as_ptr()) != 0
        {
            grid_view_clear_history(&mut *gd, bg);
        } else {
            if (*s).cx <= sx.wrapping_sub(1) {
                grid_view_clear(&mut *gd, (*s).cx, (*s).cy, sx.wrapping_sub((*s).cx), 1, bg);
            }
            grid_view_clear(
                &mut *gd,
                0,
                (*s).cy.wrapping_add(1),
                sx,
                sy.wrapping_sub((*s).cy.wrapping_add(1)),
                bg,
            );
        }
        screen_write_collect_clear(
            &mut *ctx,
            (*s).cy.wrapping_add(1),
            sy.wrapping_sub((*s).cy.wrapping_add(1)),
        );
        screen_write_collect_flush(&mut *ctx, 0, c"screen_write_clearendofscreen".as_ptr());
        if !pane_obscured(&ttyctx) {
            tty_write(Some(tty_cmd_clearendofscreen), &mut ttyctx);
            return;
        }
        let (ocx, ocy) = ((*s).cx, (*s).cy);
        let offset = pane_offset(&mut *ctx);
        if ocx <= sx.wrapping_sub(1) {
            collect_visible_clear(&mut *ctx, offset, ocy, ocx, sx.wrapping_sub(ocx), bg);
        }
        for y in ocy.wrapping_add(1)..sy {
            screen_write_set_cursor(&mut *ctx, 0, y as c_int);
            collect_visible_clear(&mut *ctx, offset, y, 0, sx, bg);
        }
        screen_write_set_cursor(&mut *ctx, ocx as c_int, ocy as c_int);
    }
}
/// Clears from the start of the screen to the cursor.
///
/// Two shapes of the C are kept as they are. The walk over the lines above
/// the cursor tests the cursor's own row, which the first step of the walk
/// has already moved to the top, so only the first line is collected; and the
/// clear of the cursor's row reads the cursor's column after that same move,
/// so it is one column wide whatever column the cursor was in.
pub unsafe fn screen_write_clearstartofscreen(ctx: &mut screen_write_ctx, bg: u_int) {
    unsafe {
        let s: *mut screen = ctx.s;
        let mut ttyctx = tty_ctx::default();
        let sx: u_int = (*screen_grid_ptr(s)).sx;
        screen_write_initctx(&mut *ctx, &mut ttyctx, 1, 1);
        ttyctx.bg = bg;
        if (*s).cy > 0 {
            grid_view_clear(screen_grid_mut(&mut *s), 0, 0, sx, (*s).cy, bg);
        }
        if (*s).cx > sx.wrapping_sub(1) {
            grid_view_clear(screen_grid_mut(&mut *s), 0, (*s).cy, sx, 1, bg);
        } else {
            grid_view_clear(
                screen_grid_mut(&mut *s),
                0,
                (*s).cy,
                (*s).cx.wrapping_add(1),
                1,
                bg,
            );
        }
        screen_write_collect_clear(&mut *ctx, 0, (*s).cy);
        screen_write_collect_flush(&mut *ctx, 0, c"screen_write_clearstartofscreen".as_ptr());
        if !pane_obscured(&ttyctx) {
            tty_write(Some(tty_cmd_clearstartofscreen), &mut ttyctx);
            return;
        }
        let (ocx, ocy) = ((*s).cx, (*s).cy);
        let offset = pane_offset(&mut *ctx);
        let mut y = 0;
        while y < (*s).cy {
            screen_write_set_cursor(&mut *ctx, 0, y as c_int);
            collect_visible_clear(&mut *ctx, offset, y, 0, sx, bg);
            y = y.wrapping_add(1);
        }
        screen_write_set_cursor(&mut *ctx, 0, ocy as c_int);
        collect_visible_clear(&mut *ctx, offset, ocy, 0, (*s).cx.wrapping_add(1), bg);
        screen_write_set_cursor(&mut *ctx, ocx as c_int, ocy as c_int);
    }
}
pub unsafe fn screen_write_clearscreen(ctx: &mut screen_write_ctx, bg: u_int) {
    unsafe {
        let s: *mut screen = ctx.s;
        let mut ttyctx = tty_ctx::default();
        let sx: u_int = (*screen_grid_ptr(s)).sx;
        let sy: u_int = (*screen_grid_ptr(s)).sy;
        screen_write_initctx(&mut *ctx, &mut ttyctx, 1, 1);
        ttyctx.bg = bg;
        if (*screen_grid_ptr(s)).flags & GRID_HISTORY != 0
            && !ctx.wp.is_null()
            && options_get_number(options_ptr(&(*ctx.wp).options), c"scroll-on-clear".as_ptr()) != 0
        {
            grid_view_clear_history(screen_grid_mut(&mut *s), bg);
        } else {
            grid_view_clear(screen_grid_mut(&mut *s), 0, 0, sx, sy, bg);
        }
        screen_write_collect_clear(&mut *ctx, 0, sy);
        if !pane_obscured(&ttyctx) {
            tty_write(Some(tty_cmd_clearscreen), &mut ttyctx);
            return;
        }
        let (ocx, ocy) = ((*s).cx, (*s).cy);
        let offset = pane_offset(&mut *ctx);
        for y in 0..sy {
            screen_write_set_cursor(&mut *ctx, 0, y as c_int);
            collect_visible_clear(&mut *ctx, offset, y, 0, sx, bg);
        }
        screen_write_set_cursor(&mut *ctx, ocx as c_int, ocy as c_int);
    }
}
pub unsafe fn screen_write_clearhistory(ctx: &mut screen_write_ctx) {
    unsafe {
        grid_clear_history(screen_grid_mut(&mut *ctx.s));
    }
}
pub unsafe fn screen_write_fullredraw(ctx: &mut screen_write_ctx) {
    unsafe {
        let mut ttyctx = tty_ctx::default();
        screen_write_collect_flush(&mut *ctx, 0, c"screen_write_fullredraw".as_ptr());
        screen_write_initctx(&mut *ctx, &mut ttyctx, 1, 0);
        if ttyctx.redraw_cb.is_some() {
            ttyctx.redraw_cb.expect("non-null function pointer")(&ttyctx);
        }
    }
}
/// The bytes collected for one line, which is as wide as the grid was when
/// the line first collected. Only a line that has collected text has them,
/// which is every line an item of type `TEXT` hangs on; the rest read empty.
unsafe fn line_text<'a>(cl: *mut screen_write_cline) -> &'a mut [u8] {
    unsafe {
        match &mut (*cl).data {
            Some(text) => text,
            None => &mut [],
        }
    }
}

/// Cuts what is already collected on line `y` out of the way of `used`
/// columns from `x`, and answers the item the new one is to go in front of,
/// or null for the end of the line.
///
/// An item given up whole is read back after it has gone to the free list:
/// the free only relinks it, so the wrapped flag it carried is still there
/// and is carried on to the item replacing it. That is what the C did and it
/// is pinned by a test.
unsafe fn screen_write_collect_trim(
    ctx: &mut screen_write_ctx,
    y: u_int,
    x: u_int,
    used: u_int,
    wrapped: *mut c_int,
) -> CItem {
    unsafe {
        let cl: *mut screen_write_cline = &raw mut write_list(&mut *ctx.s)[y as usize];
        let items = &raw mut (*cl).items;
        let sx = x;
        let ex = x.wrapping_add(used).wrapping_sub(1);
        let name = c"screen_write_collect_trim".as_ptr();
        if (*items).is_empty() {
            return CITEM_NONE;
        }
        for ci in citem_list(items) {
            let csx = citem(ci).x;
            let cex = citem(ci).x.wrapping_add(citem(ci).used).wrapping_sub(1);
            if cex < sx {
                log_debug(
                    c"%s: %p %u-%u before %u-%u".as_ptr(),
                    fmt_args![name, ci, csx, cex, sx, ex],
                );
            } else if csx > ex {
                log_debug(
                    c"%s: %p %u-%u after %u-%u".as_ptr(),
                    fmt_args![name, ci, csx, cex, sx, ex],
                );
                return ci;
            } else if csx >= sx && cex <= ex {
                log_debug(
                    c"%s: %p %u-%u inside %u-%u".as_ptr(),
                    fmt_args![name, ci, csx, cex, sx, ex],
                );
                citem_remove(items, ci);
                screen_write_free_citem(ci);
                if csx == 0 && citem(ci).wrapped != 0 && !wrapped.is_null() {
                    *wrapped = 1;
                }
            } else if csx < sx && cex >= sx && cex <= ex {
                log_debug(
                    c"%s: %p %u-%u start %u-%u".as_ptr(),
                    fmt_args![name, ci, csx, cex, sx, ex],
                );
                citem(ci).used = sx.wrapping_sub(csx);
                log_debug(
                    c"%s: %p now %u-%u".as_ptr(),
                    fmt_args![
                        name,
                        ci,
                        citem(ci).x,
                        citem(ci).x.wrapping_add(citem(ci).used).wrapping_add(1)
                    ],
                );
            } else if cex > ex && csx >= sx && csx <= ex {
                log_debug(
                    c"%s: %p %u-%u end %u-%u".as_ptr(),
                    fmt_args![name, ci, csx, cex, sx, ex],
                );
                citem(ci).x = ex.wrapping_add(1);
                citem(ci).used = cex.wrapping_sub(ex);
                log_debug(
                    c"%s: %p now %u-%u".as_ptr(),
                    fmt_args![
                        name,
                        ci,
                        citem(ci).x,
                        citem(ci).x.wrapping_add(citem(ci).used).wrapping_add(1)
                    ],
                );
                return ci;
            } else {
                log_debug(
                    c"%s: %p %u-%u under %u-%u".as_ptr(),
                    fmt_args![name, ci, csx, cex, sx, ex],
                );
                let ci2 = screen_write_get_citem();
                citem(ci2).type_0 = citem(ci).type_0;
                citem(ci2).bg = citem(ci).bg;
                citem(ci2).gc = citem(ci).gc;
                citem_insert_after(items, ci, ci2);
                citem(ci).used = sx.wrapping_sub(csx);
                citem(ci2).x = ex.wrapping_add(1);
                citem(ci2).used = cex.wrapping_sub(ex);
                log_debug(
                    c"%s: %p now %u-%u (%p) and %u-%u (%p)".as_ptr(),
                    fmt_args![
                        name,
                        ci,
                        citem(ci).x,
                        citem(ci).x.wrapping_add(citem(ci).used).wrapping_sub(1),
                        ci,
                        citem(ci2).x,
                        citem(ci2).x.wrapping_add(citem(ci2).used).wrapping_sub(1),
                        ci2
                    ],
                );
                return ci2;
            }
        }
        CITEM_NONE
    }
}

/// Gives up everything collected on `n` lines from `y`.
unsafe fn screen_write_collect_clear(ctx: &mut screen_write_ctx, y: u_int, n: u_int) {
    unsafe {
        let wl = write_list(&mut *ctx.s);
        for i in y..y.wrapping_add(n) {
            citem_free_all(&raw mut wl[i as usize].items);
        }
    }
}

/// Moves what is collected inside the scroll region up a line, taking the top
/// line's text buffer round to the bottom, and collects a clear of the line
/// that comes in at the bottom.
unsafe fn screen_write_collect_scroll(ctx: &mut screen_write_ctx, bg: u_int) {
    unsafe {
        let s: *mut screen = ctx.s;
        log_debug(
            c"%s: at %u,%u (region %u-%u)".as_ptr(),
            fmt_args![
                c"screen_write_collect_scroll".as_ptr(),
                (*s).cx,
                (*s).cy,
                (*s).rupper,
                (*s).rlower
            ],
        );
        screen_write_collect_clear(&mut *ctx, (*s).rupper, 1);
        let (rupper, rlower) = ((*s).rupper, (*s).rlower);
        let wl = write_list(&mut *s);
        let saved = wl[rupper as usize].data.take();
        let mut y = rupper;
        while y < rlower {
            let (above, below) = wl.split_at_mut(y.wrapping_add(1) as usize);
            let (into, from) = (&mut above[y as usize], &mut below[0]);
            into.items.append(&mut from.items);
            into.data = from.data.take();
            y = y.wrapping_add(1);
        }
        wl[rlower as usize].data = saved;
        let ci = screen_write_get_citem();
        citem(ci).x = 0;
        citem(ci).used = (*screen_grid_ptr(s)).sx;
        citem(ci).type_0 = CLEAR;
        citem(ci).bg = bg;
        citem_insert_tail(&raw mut wl[(*s).rlower as usize].items, ci);
    }
}

/// Tells the terminal about the lines that have been scrolled, or redraws the
/// pane when something in front of it means it cannot be told. Answers
/// whether the scroll was written out.
unsafe fn screen_write_collect_flush_scrolled(ctx: &mut screen_write_ctx) -> c_int {
    unsafe {
        let wp: *mut window_pane = ctx.wp;
        let s: *mut screen = ctx.s;
        let mut ttyctx = tty_ctx::default();
        screen_write_initctx(&mut *ctx, &mut ttyctx, 1, 1);
        if pane_obscured(&ttyctx) && !wp.is_null() {
            screen_write_redraw_pane(&mut *ctx, &mut ttyctx);
            return 0;
        }
        log_debug(
            c"%s: scrolled %u (region %u-%u)".as_ptr(),
            fmt_args![
                c"screen_write_collect_flush_scrolled".as_ptr(),
                ctx.scrolled,
                (*s).rupper,
                (*s).rlower
            ],
        );
        let region = (*s).rlower.wrapping_sub((*s).rupper).wrapping_add(1);
        if ctx.scrolled > region {
            ctx.scrolled = region;
        }
        if !wp.is_null() && ((*wp).yoff as u_int).wrapping_add((*wp).sy) > (*(*wp).window).sy {
            ttyctx.orlower = ttyctx.orlower.wrapping_sub(
                ((*wp).yoff as u_int)
                    .wrapping_add((*wp).sy)
                    .wrapping_sub((*(*wp).window).sy),
            );
        }
        ttyctx.value = TtyCtxValue::Num(ctx.scrolled);
        ttyctx.bg = ctx.bg;
        tty_write(Some(tty_cmd_scrollup), &mut ttyctx);
        if !wp.is_null() {
            (*wp).flags |= PANE_REDRAWSCROLLBAR;
        }
        1
    }
}

/// Writes out whatever is collected on line `y` and is visible, and answers
/// how many items that was. What is written is given up; what a pane in front
/// of this one hides is left collected.
///
/// Two guards the C had are gone with the conversion. The overlap test above
/// them has already found that the item and the range meet, and both are at
/// least one column wide, so the start of the part to write is always before
/// its end and the length is never zero.
///
/// The `fatalx` arm for a list out of order stays, and is exempt from the
/// coverage requirement: its only effect is to stop the process.
unsafe fn screen_write_collect_flush_line(ctx: &mut screen_write_ctx, y: u_int) -> u_int {
    unsafe {
        let wp: *mut window_pane = ctx.wp;
        let s: *mut screen = ctx.s;
        let cl: *mut screen_write_cline = &raw mut write_list(&mut *s)[y as usize];
        let mut ttyctx = tty_ctx::default();
        let mut last: u_int = UINT_MAX;
        let mut items: u_int = 0;
        let (wsx, wsy, xoff, yoff) = if wp.is_null() {
            ((*screen_grid_ptr(s)).sx, (*screen_grid_ptr(s)).sy, 0, 0)
        } else {
            (
                (*(*wp).window).sx,
                (*(*wp).window).sy,
                (*wp).xoff,
                (*wp).yoff,
            )
        };
        if y.wrapping_add(yoff as u_int) >= wsy {
            return 0;
        }
        let mut ranges = visible_ranges::default();
        screen_redraw_get_visible_ranges(
            wp,
            0,
            y.wrapping_add(yoff as u_int) as c_int,
            wsx,
            &mut ranges,
        );
        let r = &raw mut ranges;
        for ci in citem_list(&raw mut (*cl).items) {
            log_debug(
                c"collect list: x=%u (last %u), y=%u, used=%u".as_ptr(),
                fmt_args![citem(ci).x, last, y, citem(ci).used],
            );
            if last != UINT_MAX && citem(ci).x <= last {
                fatalx(
                    c"collect list bad order: %u <= %u".as_ptr(),
                    fmt_args![citem(ci).x, last],
                );
            }
            let mut written = false;
            let mut i = 0;
            while i < (*r).used {
                let ri = (*r).ranges[i as usize];
                i = i.wrapping_add(1);
                if ri.nx == 0 {
                    continue;
                }
                let r_start = ri.px as c_int;
                let r_end = ri.px.wrapping_add(ri.nx) as c_int;
                let c_start = citem(ci).x as c_int;
                let c_end = citem(ci).x.wrapping_add(citem(ci).used) as c_int;
                if c_start + xoff >= r_end || c_end + xoff <= r_start {
                    continue;
                }
                let w_start = if r_start > c_start + xoff {
                    r_start - xoff
                } else {
                    c_start
                };
                let w_end = if c_end + xoff > r_end {
                    r_end - xoff
                } else {
                    c_end
                };
                let w_length = (w_end - w_start) as u_int;
                screen_write_set_cursor(&mut *ctx, w_start, y as c_int);
                if citem(ci).type_0 == CLEAR {
                    screen_write_initctx(&mut *ctx, &mut ttyctx, 1, 0);
                    ttyctx.bg = citem(ci).bg;
                    ttyctx.value = TtyCtxValue::Num(w_length);
                    tty_write(Some(tty_cmd_clearcharacter), &mut ttyctx);
                } else {
                    screen_write_initctx(&mut *ctx, &mut ttyctx, 0, 0);
                    ttyctx.cell = &raw mut citem(ci).gc;
                    if citem(ci).wrapped != 0 {
                        ttyctx.flags |= TTY_CTX_WRAPPED;
                    }
                    let text = &line_text(cl)[w_start as usize..];
                    ttyctx.value = TtyCtxValue::Data(tty_ctx_data {
                        data: text.as_ptr() as *const c_char,
                        size: w_length as size_t,
                    });
                    tty_write(Some(tty_cmd_cells), &mut ttyctx);
                }
                items = items.wrapping_add(1);
                written = true;
            }
            if written {
                last = citem(ci).x;
                citem_remove(&raw mut (*cl).items, ci);
                screen_write_free_citem(ci);
            }
        }
        items
    }
}

/// Writes everything collected so far onto the grid.
///
/// `scroll_only` stops once the scrolled lines have been dealt with, leaving
/// the text where it is. A screen in a synchronised update, and one whose
/// scrolled lines could not be written, give up what they collected instead
/// of writing it; the C reached that second case by jumping past the flush,
/// and it is the tail of this function now.
unsafe fn screen_write_collect_flush(
    ctx: &mut screen_write_ctx,
    scroll_only: c_int,
    from: *const c_char,
) {
    unsafe {
        let s = ctx.s;
        let mut give_up = (*s).mode & MODE_SYNC != 0;
        if !give_up {
            if ctx.scrolled != 0 && screen_write_collect_flush_scrolled(&mut *ctx) == 0 {
                give_up = true;
            } else {
                ctx.scrolled = 0;
            }
        }
        if !give_up {
            ctx.bg = 8;
            if scroll_only != 0 {
                return;
            }
            let (cx, cy) = ((*s).cx, (*s).cy);
            let mut items: u_int = 0;
            for y in 0..(*screen_grid_ptr(s)).sy {
                items = items.wrapping_add(screen_write_collect_flush_line(&mut *ctx, y));
            }
            (*s).cx = cx;
            (*s).cy = cy;
            log_debug(
                c"%s: flushed %u items (%s)".as_ptr(),
                fmt_args![c"screen_write_collect_flush".as_ptr(), items, from],
            );
            return;
        }
        for cl in write_list(&mut *s) {
            citem_free_all(&raw mut cl.items);
        }
        ctx.scrolled = 0;
        ctx.bg = 8;
    }
}

/// Puts `ci` on the line the cursor is on, in the order the writing calls
/// expect, and gives the context a fresh item to fill in.
unsafe fn screen_write_collect_insert(ctx: &mut screen_write_ctx, ci: CItem) {
    unsafe {
        let s: *mut screen = ctx.s;
        let cy = (*s).cy as usize;
        let items = &raw mut write_list(&mut *s)[cy].items;
        let before = screen_write_collect_trim(
            &mut *ctx,
            (*s).cy,
            citem(ci).x,
            citem(ci).used,
            &raw mut citem(ci).wrapped,
        );
        if before == CITEM_NONE {
            citem_insert_tail(items, ci);
        } else {
            citem_insert_before(items, before, ci);
        }
        ctx.item = screen_write_get_citem();
    }
}
unsafe fn screen_write_collect_insert_clear(
    ctx: &mut screen_write_ctx,
    px: u_int,
    nx: u_int,
    bg: u_int,
) {
    unsafe {
        let ci: CItem = ctx.item;
        if nx != 0 {
            citem(ci).x = px;
            citem(ci).used = nx;
            citem(ci).type_0 = CLEAR;
            citem(ci).bg = bg;
            screen_write_collect_insert(&mut *ctx, ci);
        }
    }
}

/// Finishes the run of text being collected: it goes onto the grid, and the
/// padding cells of any wide character it wrote over are erased.
pub unsafe fn screen_write_collect_end(ctx: &mut screen_write_ctx) {
    unsafe {
        let s: *mut screen = ctx.s;
        let ci: CItem = ctx.item;
        let cy = (*s).cy as usize;
        let cl: *mut screen_write_cline = &raw mut write_list(&mut *s)[cy];
        let name = c"screen_write_collect_end".as_ptr();
        let mut bci: CItem = CITEM_NONE;
        let mut gc = grid_cell::default();
        if citem(ci).used == 0 {
            return;
        }
        citem(ci).x = (*s).cx;
        screen_write_collect_insert(&mut *ctx, ci);
        log_debug(
            c"%s: %u %.*s (at %u,%u)".as_ptr(),
            fmt_args![
                name,
                citem(ci).used,
                citem(ci).used as c_int,
                line_text(cl)[citem(ci).x as usize..].as_ptr(),
                (*s).cx,
                (*s).cy
            ],
        );
        if (*s).cx != 0 {
            let mut xx = (*s).cx;
            while xx > 0 {
                gc = grid_view_get_cell(screen_grid(&*s), xx, (*s).cy);
                if !(gc.flags as c_int) & GRID_FLAG_PADDING != 0 {
                    break;
                }
                grid_view_set_cell(screen_grid_mut(&mut *s), xx, (*s).cy, &grid_default_cell);
                log_debug(
                    c"%s: padding erased (before) at %u (cx %u)".as_ptr(),
                    fmt_args![name, xx, (*s).cx],
                );
                xx = xx.wrapping_sub(1);
            }
            if xx != (*s).cx {
                if xx == 0 {
                    gc = grid_view_get_cell(screen_grid(&*s), 0, (*s).cy);
                }
                if gc.data.width as c_int > 1 || gc.flags as c_int & GRID_FLAG_PADDING != 0 {
                    grid_view_set_cell(screen_grid_mut(&mut *s), xx, (*s).cy, &grid_default_cell);
                    log_debug(
                        c"%s: padding erased (before) at %u (cx %u)".as_ptr(),
                        fmt_args![name, xx, (*s).cx],
                    );
                }
                bci = ctx.item;
                citem(bci).type_0 = CLEAR;
                citem(bci).x = xx;
                citem(bci).bg = 8;
                citem(bci).used = (*s).cx.wrapping_sub(xx);
                log_debug(
                    c"%s: padding erased (before): from %u, size %u".as_ptr(),
                    fmt_args![name, citem(bci).x, citem(bci).used],
                );
            }
        }
        grid_view_set_cells(
            screen_grid_mut(&mut *s),
            (*s).cx,
            (*s).cy,
            &raw mut citem(ci).gc,
            line_text(cl)[citem(ci).x as usize..].as_ptr() as *const c_char,
            citem(ci).used as size_t,
        );
        if bci != CITEM_NONE {
            screen_write_collect_insert(&mut *ctx, bci);
        }
        screen_write_set_cursor(&mut *ctx, (*s).cx.wrapping_add(citem(ci).used) as c_int, -1);
        let mut xx = (*s).cx;
        while xx < (*screen_grid_ptr(s)).sx {
            gc = grid_view_get_cell(screen_grid(&*s), xx, (*s).cy);
            if !(gc.flags as c_int) & GRID_FLAG_PADDING != 0 {
                break;
            }
            grid_view_set_cell(screen_grid_mut(&mut *s), xx, (*s).cy, &grid_default_cell);
            log_debug(
                c"%s: padding erased (after) at %u (cx %u)".as_ptr(),
                fmt_args![name, xx, (*s).cx],
            );
            xx = xx.wrapping_add(1);
        }
        if xx != (*s).cx {
            let aci = ctx.item;
            citem(aci).type_0 = CLEAR;
            citem(aci).x = (*s).cx;
            citem(aci).bg = 8;
            citem(aci).used = xx.wrapping_sub((*s).cx);
            log_debug(
                c"%s: padding erased (after): from %u, size %u".as_ptr(),
                fmt_args![name, citem(aci).x, citem(aci).used],
            );
            screen_write_collect_insert(&mut *ctx, aci);
        }
    }
}

/// Collects one character to be written later, or writes it out now when it
/// is one the collecting cannot carry.
pub unsafe fn screen_write_collect_add(ctx: &mut screen_write_ctx, gc: *const grid_cell) {
    unsafe {
        let s: *mut screen = ctx.s;
        let sx: u_int = (*screen_grid_ptr(s)).sx;
        let collect = (*gc).data.width == 1
            && (*gc).data.size == 1
            && (*gc).data.data[0] < 0x7f
            && (*gc).flags as c_int & GRID_FLAG_TAB == 0
            && (*gc).attr as c_int & GRID_ATTR_CHARSET == 0
            && (*s).mode & MODE_WRAP != 0
            && (*s).mode & MODE_INSERT == 0
            && (*s).sel.is_none();
        if !collect {
            screen_write_collect_end(&mut *ctx);
            screen_write_collect_flush(&mut *ctx, 0, c"screen_write_collect_add".as_ptr());
            screen_write_cell(&mut *ctx, gc);
            return;
        }
        if (*s).cx > sx.wrapping_sub(1)
            || citem(ctx.item).used > sx.wrapping_sub(1).wrapping_sub((*s).cx)
        {
            screen_write_collect_end(&mut *ctx);
        }
        let ci: CItem = ctx.item;
        if (*s).cx > sx.wrapping_sub(1) {
            log_debug(
                c"%s: wrapped at %u,%u".as_ptr(),
                fmt_args![c"screen_write_collect_add".as_ptr(), (*s).cx, (*s).cy],
            );
            citem(ci).wrapped = 1;
            screen_write_linefeed(&mut *ctx, 1, 8);
            screen_write_set_cursor(&mut *ctx, 0, -1);
        }
        if citem(ci).used == 0 {
            citem(ci).gc = *gc;
        }
        let cy = (*s).cy as usize;
        let cl: *mut screen_write_cline = &raw mut write_list(&mut *s)[cy];
        let text = (*cl)
            .data
            .get_or_insert_with(|| ::std::vec::from_elem(0u8, sx as usize).into_boxed_slice());
        text[(*s).cx.wrapping_add(citem(ci).used) as usize] = (*gc).data.data[0];
        citem(ci).used = citem(ci).used.wrapping_add(1);
    }
}
/// Whether the cell is already what the packed entry holds, which is what
/// lets a write be skipped.
fn cell_matches_entry(gc: &grid_cell, gce: &grid_cell_entry) -> bool {
    unsafe {
        let data = &gce.c2rust_unnamed.data;
        gce.flags as c_int & GRID_FLAG_EXTENDED == 0
            && gc.flags as c_int == gce.flags as c_int
            && gc.attr as c_int == data.attr as c_int
            && gc.fg == data.fg as c_int
            && gc.bg == data.bg as c_int
            && gc.data.width as c_int == 1
            && gc.data.size as c_int == 1
            && data.data == gc.data.data[0]
    }
}

/// Writes one character at the cursor, wrapping to the next line first when
/// it does not fit and putting padding cells behind a wide one.
pub unsafe fn screen_write_cell(ctx: &mut screen_write_ctx, gc: *const grid_cell) {
    unsafe {
        let s: *mut screen = ctx.s;
        let wp: *mut window_pane = ctx.wp;
        let gd: *mut grid = screen_grid_ptr(s);
        let ud: *const utf8_data = &raw const (*gc).data;
        let mut tmp_gc = grid_cell::default();
        let mut now_gc = grid_cell::default();
        let mut ttyctx = tty_ctx::default();
        let sx: u_int = (*screen_grid_ptr(s)).sx;
        let sy: u_int = (*screen_grid_ptr(s)).sy;
        let width: u_int = (*ud).width as u_int;
        let mut skip = true;
        let mut redraw = false;
        if (*gc).flags as c_int & GRID_FLAG_PADDING != 0 {
            return;
        }
        if screen_write_combine(&mut *ctx, gc) != 0 {
            return;
        }
        screen_write_collect_flush(&mut *ctx, 1, c"screen_write_cell".as_ptr());
        if (*s).mode & MODE_WRAP == 0
            && width > 1
            && (width > sx || (*s).cx != sx && (*s).cx > sx.wrapping_sub(width))
        {
            return;
        }
        if (*s).mode & MODE_INSERT != 0 {
            grid_view_insert_cells(screen_grid_mut(&mut *s), (*s).cx, (*s).cy, width, 8);
            skip = false;
        }
        if (*s).mode & MODE_WRAP != 0 && (*s).cx > sx.wrapping_sub(width) {
            log_debug(
                c"%s: wrapped at %u,%u".as_ptr(),
                fmt_args![c"screen_write_cell".as_ptr(), (*s).cx, (*s).cy],
            );
            screen_write_linefeed(&mut *ctx, 1, 8);
            screen_write_set_cursor(&mut *ctx, 0, -1);
            screen_write_collect_flush(&mut *ctx, 0, c"screen_write_cell".as_ptr());
        }
        if (*s).cx > sx.wrapping_sub(width) || (*s).cy > sy.wrapping_sub(1) {
            return;
        }
        screen_write_initctx(&mut *ctx, &mut ttyctx, 0, 0);
        let gl = grid_get_line(
            screen_grid_mut(&mut *s),
            (*screen_grid_ptr(s)).hsize.wrapping_add((*s).cy),
        );
        if gl.flags & GRID_LINE_EXTENDED != 0 {
            now_gc = grid_view_get_cell(&*gd, (*s).cx, (*s).cy);
            if screen_write_overwrite(&mut *ctx, &raw mut now_gc, width) != 0 {
                redraw = true;
                skip = false;
            }
        }
        let mut xx = (*s).cx.wrapping_add(1);
        while xx < (*s).cx.wrapping_add(width) {
            log_debug(
                c"%s: new padding at %u,%u".as_ptr(),
                fmt_args![c"screen_write_cell".as_ptr(), xx, (*s).cy],
            );
            grid_view_set_padding(&mut *gd, xx, (*s).cy);
            skip = false;
            xx = xx.wrapping_add(1);
        }
        if skip {
            skip = if (*s).cx >= (*gl).cellsize() {
                grid_cells_equal(gc, &raw const grid_default_cell) != 0
            } else {
                cell_matches_entry(&*gc, &(*gl).celldata()[(*s).cx as usize])
            };
        }
        let selected = screen_check_selection(s, (*s).cx, (*s).cy);
        if selected != 0 && (*gc).flags as c_int & GRID_FLAG_SELECTED == 0 {
            tmp_gc = *gc;
            tmp_gc.flags = (tmp_gc.flags as c_int | GRID_FLAG_SELECTED) as u_char;
            grid_view_set_cell(&mut *gd, (*s).cx, (*s).cy, &tmp_gc);
        } else if selected == 0 && (*gc).flags as c_int & GRID_FLAG_SELECTED != 0 {
            tmp_gc = *gc;
            tmp_gc.flags = (tmp_gc.flags as c_int & !GRID_FLAG_SELECTED) as u_char;
            grid_view_set_cell(&mut *gd, (*s).cx, (*s).cy, &tmp_gc);
        } else if !skip {
            grid_view_set_cell(&mut *gd, (*s).cx, (*s).cy, &*gc);
        }
        if selected != 0 {
            skip = false;
        }
        let (xoff, yoff) = if wp.is_null() {
            (0, 0)
        } else {
            ((*wp).xoff, (*wp).yoff)
        };
        let mut ranges = visible_ranges::default();
        screen_redraw_get_visible_ranges(
            wp,
            (xoff as u_int).wrapping_add((*s).cx) as c_int,
            (*s).cy.wrapping_add(yoff as u_int) as c_int,
            width,
            &mut ranges,
        );
        let r = &raw mut ranges;
        let not_wrap = ((*s).mode & MODE_WRAP == 0) as u_int;
        if (*s).cx <= sx.wrapping_sub(not_wrap).wrapping_sub(width) {
            screen_write_set_cursor(&mut *ctx, (*s).cx.wrapping_add(width) as c_int, -1);
        } else {
            screen_write_set_cursor(&mut *ctx, sx.wrapping_sub(not_wrap) as c_int, -1);
        }
        if (*s).mode & MODE_INSERT != 0 {
            screen_write_collect_flush(&mut *ctx, 0, c"screen_write_cell".as_ptr());
            ttyctx.value = TtyCtxValue::Num(width);
            tty_write(Some(tty_cmd_insertcharacter), &mut ttyctx);
        }
        if skip || (*s).mode & MODE_SYNC != 0 {
            return;
        }
        if redraw && !wp.is_null() {
            screen_write_redraw_line(&mut *ctx, &mut ttyctx, (*s).cy);
            return;
        }
        if selected != 0 {
            screen_select_cell(s, &raw mut tmp_gc, gc);
        } else {
            tmp_gc = *gc;
        }
        ttyctx.cell = &raw mut tmp_gc;
        let mut vis: u_int = 0;
        for i in 0..(*r).used {
            vis = vis.wrapping_add((*r).ranges[i as usize].nx);
        }
        if vis >= width {
            tty_write(Some(tty_cmd_cell), &mut ttyctx);
            return;
        }
        utf8_set(&mut tmp_gc.data, b' ');
        let mut i = 0;
        while i < (*r).used {
            let ri: *const visible_range = (*r).ranges.as_ptr().add(i as usize);
            let mut n = 0 as u_int;
            while n < (*ri).nx {
                ttyctx.ocx = ((*ri).px as c_int - xoff + n as c_int) as u_int;
                tty_write(Some(tty_cmd_cell), &mut ttyctx);
                n = n.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
    }
}

/// Joins a character to the one in front of it when it is one that combines,
/// and answers whether it was taken that way rather than written on its own.
///
/// The guard on the size of the joined character is the C's, one byte too
/// lenient: a joined character of exactly 32 bytes still goes in, and
/// `utf8_from_data` cannot pack a size that large, so the cell reads back
/// with no size at all. It is kept as it is for parity.
unsafe fn screen_write_combine(ctx: &mut screen_write_ctx, gc: *const grid_cell) -> c_int {
    unsafe {
        let s: *mut screen = ctx.s;
        let wp: *mut window_pane = ctx.wp;
        let gd: *mut grid = screen_grid_ptr(s);
        let ud: *const utf8_data = &raw const (*gc).data;
        let oo: *mut options = global_options;
        let mut cx: u_int = (*s).cx;
        let cy: u_int = (*s).cy;
        let mut n: u_int = 0;
        let mut last = grid_cell::default();
        let mut ttyctx = tty_ctx::default();
        let mut force_wide = false;
        let mut zero_width = false;
        if utf8_is_hangul_filler(ud) != 0 {
            return 1;
        }
        if utf8_is_zwj(ud) != 0 {
            zero_width = true;
        } else if utf8_is_vs(ud) != 0 {
            zero_width = true;
            if options_get_number(oo, c"variation-selector-always-wide".as_ptr()) != 0 {
                force_wide = true;
            }
        } else if (*ud).width as c_int == 0 {
            zero_width = true;
        }
        if ((*ud).size as c_int) < 2 || cx == 0 {
            return zero_width as c_int;
        }
        log_debug(
            c"%s: character %.*s at %u,%u (width %u)".as_ptr(),
            fmt_args![
                c"screen_write_combine".as_ptr(),
                (*ud).size as c_int,
                &raw const (*ud).data as *const u_char,
                cx,
                cy,
                (*ud).width as c_int
            ],
        );
        n = 1;
        last = grid_view_get_cell(&*gd, cx.wrapping_sub(n), cy);
        if cx != 1 && last.flags as c_int & GRID_FLAG_PADDING != 0 {
            n = 2;
            last = grid_view_get_cell(&*gd, cx.wrapping_sub(n), cy);
        }
        if n != last.data.width as u_int || last.flags as c_int & GRID_FLAG_PADDING != 0 {
            return zero_width as c_int;
        }
        if !zero_width {
            match hanguljamo_check_state(&raw mut last.data, ud) {
                HANGULJAMO_STATE_NOT_COMPOSABLE => return 1,
                HANGULJAMO_STATE_CHOSEONG => return 0,
                HANGULJAMO_STATE_NOT_HANGULJAMO => {
                    if utf8_should_combine(&raw mut last.data, ud) != 0 {
                        force_wide = true;
                    } else if utf8_should_combine(ud, &raw mut last.data) != 0 {
                        force_wide = true;
                    } else if utf8_has_zwj(&raw mut last.data) == 0 {
                        return 0;
                    }
                }
                _ => {}
            }
        }
        let size = last.data.size as usize;
        let more = (*ud).size as usize;
        if size + more > last.data.data.len() {
            return 0;
        }
        screen_write_collect_flush(&mut *ctx, 0, c"screen_write_combine".as_ptr());
        log_debug(
            c"%s: %.*s -> %.*s at %u,%u (offset %u, width %u)".as_ptr(),
            fmt_args![
                c"screen_write_combine".as_ptr(),
                (*ud).size as c_int,
                &raw const (*ud).data as *const u_char,
                last.data.size as c_int,
                &raw mut last.data.data as *mut u_char,
                cx.wrapping_sub(n),
                cy,
                n,
                last.data.width as c_int
            ],
        );
        let joining = &(*ud).data;
        last.data.data[size..size + more].copy_from_slice(&joining[..more]);
        last.data.size = (size + more) as u_char;
        if last.data.width as c_int == 1 && force_wide {
            last.data.width = 2;
            n = 2;
            cx = cx.wrapping_add(1);
        } else {
            force_wide = false;
        }
        grid_view_set_cell(&mut *gd, cx.wrapping_sub(n), cy, &last);
        if force_wide {
            grid_view_set_padding(&mut *gd, cx.wrapping_sub(1), cy);
        }
        let yoff = if wp.is_null() { 0 } else { (*wp).yoff as u_int };
        let mut ranges = visible_ranges::default();
        screen_redraw_get_visible_ranges(
            wp,
            cx.wrapping_sub(n) as c_int,
            cy.wrapping_add(yoff) as c_int,
            n,
            &mut ranges,
        );
        let r = &raw mut ranges;
        let mut vis: u_int = 0;
        for i in 0..(*r).used {
            vis = vis.wrapping_add((*r).ranges[i as usize].nx);
        }
        if vis < n {
            return 1;
        }
        screen_write_set_cursor(&mut *ctx, cx.wrapping_sub(n) as c_int, cy as c_int);
        screen_write_initctx(&mut *ctx, &mut ttyctx, 0, 0);
        ttyctx.cell = &raw mut last;
        if force_wide {
            ttyctx.flags |= TTY_CTX_CELL_INVALIDATE;
        }
        tty_write(Some(tty_cmd_cell), &mut ttyctx);
        screen_write_set_cursor(&mut *ctx, cx as c_int, cy as c_int);
        1
    }
}

/// Erases the padding cells around the cursor that a character of `width`
/// columns is about to write over, and answers whether anything was erased.
unsafe fn screen_write_overwrite(
    ctx: &mut screen_write_ctx,
    gc: *mut grid_cell,
    width: u_int,
) -> c_int {
    unsafe {
        let s: *mut screen = ctx.s;
        let gd: *mut grid = screen_grid_ptr(s);
        let mut tmp_gc = grid_cell::default();
        let mut done = 0;
        if (*gc).flags as c_int & GRID_FLAG_PADDING != 0 {
            let mut xx = (*s).cx;
            while xx > 0 {
                tmp_gc = grid_view_get_cell(&*gd, xx, (*s).cy);
                if tmp_gc.flags as c_int & GRID_FLAG_PADDING == 0 {
                    break;
                }
                log_debug(
                    c"%s: padding at %u,%u".as_ptr(),
                    fmt_args![c"screen_write_overwrite".as_ptr(), xx, (*s).cy],
                );
                grid_view_set_cell(&mut *gd, xx, (*s).cy, &grid_default_cell);
                xx = xx.wrapping_sub(1);
            }
            log_debug(
                c"%s: character at %u,%u".as_ptr(),
                fmt_args![c"screen_write_overwrite".as_ptr(), xx, (*s).cy],
            );
            grid_view_set_cell(&mut *gd, xx, (*s).cy, &grid_default_cell);
            done = 1;
        }
        if width != 1
            || (*gc).data.width as c_int != 1
            || (*gc).flags as c_int & GRID_FLAG_PADDING != 0
        {
            let mut xx = (*s).cx.wrapping_add(width);
            while xx < (*screen_grid_ptr(s)).sx {
                tmp_gc = grid_view_get_cell(&*gd, xx, (*s).cy);
                if tmp_gc.flags as c_int & GRID_FLAG_PADDING == 0 {
                    break;
                }
                log_debug(
                    c"%s: overwrite at %u,%u".as_ptr(),
                    fmt_args![c"screen_write_overwrite".as_ptr(), xx, (*s).cy],
                );
                if (*gc).flags as c_int & GRID_FLAG_TAB != 0 {
                    tmp_gc = *gc;
                    tmp_gc.data.data = [0; 32];
                    tmp_gc.data.data[0] = b' ';
                    tmp_gc.data.have = 1;
                    tmp_gc.data.size = tmp_gc.data.have;
                    tmp_gc.data.width = tmp_gc.data.size;
                    grid_view_set_cell(&mut *gd, xx, (*s).cy, &tmp_gc);
                } else {
                    grid_view_set_cell(&mut *gd, xx, (*s).cy, &grid_default_cell);
                }
                done = 1;
                xx = xx.wrapping_add(1);
            }
        }
        done
    }
}
/// Tells the terminal what the selection is.
pub unsafe fn screen_write_setselection(
    ctx: &mut screen_write_ctx,
    clip: *const c_char,
    str: *mut u_char,
    len: u_int,
) {
    unsafe {
        let mut ttyctx = tty_ctx::default();
        screen_write_initctx(&mut *ctx, &mut ttyctx, 0, 0);
        ttyctx.value = TtyCtxValue::Sel(tty_ctx_sel {
            clip,
            data: str as *const c_char,
            size: len as size_t,
        });
        tty_write(Some(tty_cmd_setselection), &mut ttyctx);
    }
}

/// Hands bytes to the terminal as they are.
pub unsafe fn screen_write_rawstring(
    ctx: &mut screen_write_ctx,
    str: *mut u_char,
    len: u_int,
    allow_invisible_panes: c_int,
) {
    unsafe {
        let mut ttyctx = tty_ctx::default();
        screen_write_initctx(&mut *ctx, &mut ttyctx, 0, 0);
        if allow_invisible_panes != 0 {
            ttyctx.flags |= TTY_CTX_INVISIBLE_PANES;
        }
        ttyctx.value = TtyCtxValue::Data(tty_ctx_data {
            data: str as *const c_char,
            size: len as size_t,
        });
        tty_write(Some(tty_cmd_rawstring), &mut ttyctx);
    }
}

/// Switches the pane to its alternate screen, or back, and asks for the
/// redraw the change needs.
unsafe fn screen_write_alternate(
    ctx: &mut screen_write_ctx,
    gc: *mut grid_cell,
    cursor: c_int,
    on: bool,
    from: *const c_char,
) {
    unsafe {
        let mut ttyctx = tty_ctx::default();
        let wp: *mut window_pane = ctx.wp;
        if !wp.is_null()
            && options_get_number(options_ptr(&(*wp).options), c"alternate-screen".as_ptr()) == 0
        {
            return;
        }
        screen_write_collect_flush(&mut *ctx, 0, from);
        if on {
            screen_alternate_on(ctx.s, gc, cursor);
        } else {
            screen_alternate_off(ctx.s, gc, cursor);
        }
        if !wp.is_null() {
            layout_fix_panes((*wp).window, null_mut::<window_pane>());
            server_redraw_window_borders((*wp).window);
        }
        screen_write_initctx(&mut *ctx, &mut ttyctx, 1, 0);
        if let Some(cb) = ttyctx.redraw_cb {
            cb(&ttyctx);
        }
    }
}
pub unsafe fn screen_write_alternateon(
    ctx: &mut screen_write_ctx,
    gc: *mut grid_cell,
    cursor: c_int,
) {
    unsafe {
        screen_write_alternate(
            &mut *ctx,
            gc,
            cursor,
            true,
            c"screen_write_alternateon".as_ptr(),
        )
    }
}
pub unsafe fn screen_write_alternateoff(
    ctx: &mut screen_write_ctx,
    gc: *mut grid_cell,
    cursor: c_int,
) {
    unsafe {
        screen_write_alternate(
            &mut *ctx,
            gc,
            cursor,
            false,
            c"screen_write_alternateoff".as_ptr(),
        )
    }
}
pub const __INT_MAX__: c_int = 2147483647;

#[cfg(test)]
#[path = "../tests/test_screen_write_hooks.rs"]
pub(crate) mod test_hooks;

#[cfg(test)]
#[path = "../tests/test_screen_write.rs"]
mod tests;
