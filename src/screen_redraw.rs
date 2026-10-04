use crate::src::ffi::libc::{memcpy, memset};
use crate::src::format::bytes::write_cstr;
use crate::src::format::{format_create_defaults, format_free};
use crate::src::grid::grid_default_cell;
use crate::src::log::{fatalx, log_cstr, log_debug, log_get_level};
use crate::src::options::options_get_number;
use crate::src::options::options_owner_ptr;
use crate::src::prompt::prompt_draw;
use crate::src::screen::{screen_free, screen_init};
use crate::src::screen_write::{screen_write_start, screen_write_stop};
use crate::src::server::{marked_pane, server_is_marked};
use crate::src::server_client::Client as _;
use crate::src::session::Session;
use crate::src::shared::abi::*;
use crate::src::shared::borders::{CELL_NONE, CELL_UD};
use crate::src::shared::client::client;
use crate::src::shared::client::ClientRef;
use crate::src::shared::client::{
    CLIENT_REDRAWBORDERS, CLIENT_REDRAWSTATUS, CLIENT_REDRAWWINDOW, CLIENT_SUSPENDED, CLIENT_UTF8,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::cmdq_item;
use crate::src::shared::display::{visible_range, visible_ranges};
use crate::src::shared::format::format_tree;
use crate::src::shared::grid::*;
use crate::src::shared::layout::*;
use crate::src::shared::limits::SIZE_MAX;
use crate::src::shared::options::options;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::{
    PANE_BORDER_ARROWS, PANE_BORDER_BOTH, PANE_BORDER_COLOUR, PANE_NEWSTATUS, PANE_SCROLLBARS_LEFT,
    PANE_STATUS_BOTTOM, PANE_STATUS_OFF, PANE_STATUS_TOP,
};
use crate::src::shared::prompt::prompt_draw_data;
use crate::src::shared::redraw::{
    redraw_line, redraw_scene, redraw_span, redraw_span_data, redraw_span_type, redraw_spans,
};
use crate::src::shared::screen::{screen, CURSOR_MODES, MODE_SYNC};
use crate::src::shared::screen_write::screen_write_ctx;
use crate::src::shared::session::session;
use crate::src::shared::session::SessionRef;
use crate::src::shared::style::*;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::tty::*;
use crate::src::shared::tty::{tty, tty_style_ctx};
use crate::src::shared::window::window;
use crate::src::shared::window::WindowRef;
use crate::src::shared::window::WINDOW_PANE_NO_MODE;
use crate::src::status::{
    status_line_size, status_message_redraw, status_prompt_redraw, status_redraw,
};
use crate::src::style::style_apply_with_options;
use crate::src::text::utf8::utf8_set;
use crate::src::tty::{
    tty_cell, tty_cursor, tty_default_colours, tty_puts, tty_reset, tty_sync_start,
    tty_update_mode, tty_window_offset,
};
use crate::src::tty_draw::tty_draw_line;
use crate::src::tty_term::tty_term_has;
use crate::src::tty_term::tty_term_owner_ptr;
use crate::src::window::windows;
use crate::src::window::Window as _;
use crate::src::window::WindowIndex as _;

use crate::src::window::WindowPane;
use crate::src::window_border::{window_get_border_cell, window_get_fill_cell};
use crate::src::window_copy::window_copy_get_current_offset;
use std::cell::RefCell;

pub const REDRAW_SPAN_SCROLLBAR: redraw_span_type = 5;
pub const REDRAW_SPAN_BORDER: redraw_span_type = 4;
pub const REDRAW_SPAN_STATUS: redraw_span_type = 3;
pub const REDRAW_SPAN_EMPTY: redraw_span_type = 2;
pub const REDRAW_SPAN_OUTSIDE: redraw_span_type = 1;
pub const REDRAW_SPAN_PANE: redraw_span_type = 0;

#[derive(Clone)]
#[repr(C)]
pub struct redraw_draw_ctx<'scene> {
    pub scene: &'scene redraw_scene,
    pub active: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    pub marked: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    pub status_lines: u_int,
    pub pane_lines: pane_lines,
    pub default_gc: grid_cell,
    pub flags: ::core::ffi::c_int,
}
#[derive(Clone, Default)]
pub struct redraw_build_cell {
    pub data: redraw_span_data,
}
#[repr(C)]
pub struct redraw_build_ctx<'a> {
    pub w: &'a WindowRef,
    pub ox: u_int,
    pub oy: u_int,
    pub sx: u_int,
    pub sy: u_int,
    pub ind: ::core::ffi::c_int,
    pub cells: &'a mut [redraw_build_cell],
}

pub const REDRAW_SPAN_TYPES: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const REDRAW_BORDER_L: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const REDRAW_BORDER_R: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const REDRAW_BORDER_U: ::core::ffi::c_int = 4;
pub const REDRAW_BORDER_D: ::core::ffi::c_int = 8;
pub const REDRAW_BORDER_IS_ARROW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const REDRAW_SCROLLBAR_LEFT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const REDRAW_SCROLLBAR_RIGHT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const REDRAW_SCROLLBAR_OVERLAY: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const REDRAW_PANE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const REDRAW_OUTSIDE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const REDRAW_EMPTY: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const REDRAW_PANE_BORDER: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const REDRAW_PANE_STATUS: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const REDRAW_PANE_SCROLLBAR: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const REDRAW_STATUS: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const REDRAW_ALL: ::core::ffi::c_int = 0x7fffffff as ::core::ffi::c_int;
pub const REDRAW_START_ISOLATE: &std::ffi::CStr = c"\u{2066}";
pub const REDRAW_END_ISOLATE: &std::ffi::CStr = c"\u{2069}";
thread_local! {
    static REDRAW_CELLS: RefCell<Vec<redraw_build_cell>> = const { RefCell::new(Vec::new()) };
}

// Each scene build owns its scratch cells until it has copied them into spans.
// Taking the cache leaves an empty slot for a nested scene build.
struct RedrawCellScratch(Vec<redraw_build_cell>);

impl RedrawCellScratch {
    fn take() -> Self {
        REDRAW_CELLS.with(|cache| Self(std::mem::take(&mut *cache.borrow_mut())))
    }
}

impl Drop for RedrawCellScratch {
    fn drop(&mut self) {
        self.0.clear();
        REDRAW_CELLS.with(|cache| {
            let mut cached = cache.borrow_mut();
            if self.0.capacity() > cached.capacity() {
                *cached = std::mem::take(&mut self.0);
            }
        });
    }
}
pub const REDRAW_ISOLATES: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const REDRAW_DEFAULT_SET: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const REDRAW_STATUS_TOP: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
fn redraw_flags_to_string(flags: ::core::ffi::c_int) -> std::ffi::CString {
    let mut names = Vec::new();
    for (flag, name) in [
        (REDRAW_STATUS, "status"),
        (REDRAW_PANE, "pane"),
        (REDRAW_PANE_BORDER, "border"),
        (REDRAW_PANE_STATUS, "pane-status"),
        (REDRAW_PANE_SCROLLBAR, "scrollbar"),
    ] {
        if flags & flag != 0 {
            names.push(name);
        }
    }
    if flags == REDRAW_ALL {
        names.push("all");
    }
    std::ffi::CString::new(names.join(" ")).expect("redraw flag names contain no NUL")
}
unsafe fn redraw_get_window_offset(c: &ClientRef) -> tty_window_view {
    let mut view = c.terminal_view();
    let (sx, sy) = c.terminal_size();
    view.sx = view.sx.max(sx);
    view.sy = view.sy.max(sy.wrapping_sub(status_line_size(c)));
    view
}
unsafe fn redraw_set_context(c: &ClientRef, bctx: &mut redraw_build_ctx) {
    let view = redraw_get_window_offset(c);
    bctx.ox = view.ox;
    bctx.oy = view.oy;
    bctx.sx = view.sx;
    bctx.sy = view.sy;
    bctx.ind = bctx
        .w
        .with_options_mut(|options| options_get_number(options, c"pane-border-indicators"))
        as ::core::ffi::c_int;
}
unsafe fn redraw_get_build_cell(
    mut bctx: *mut redraw_build_ctx,
    mut x: u_int,
    mut y: u_int,
) -> *mut redraw_build_cell {
    let index = y as usize * (*bctx).sx as usize + x as usize;
    &raw mut (*bctx).cells[index]
}
unsafe fn redraw_reset_cell(mut bctx: *mut redraw_build_ctx, mut x: u_int, mut y: u_int) {
    let mut bc: *mut redraw_build_cell = redraw_get_build_cell(bctx, x, y);
    (*bc).data = redraw_span_data::default();
    if (*bctx).ox.wrapping_add(x) < (*bctx).w.logical_size().0
        && (*bctx).oy.wrapping_add(y) < (*bctx).w.logical_size().1
    {
        (*bc).data = redraw_span_data::Empty;
    } else {
        (*bc).data = redraw_span_data::Outside;
    };
}
unsafe fn redraw_window_to_scene(
    mut bctx: *mut redraw_build_ctx,
    mut wx: ::core::ffi::c_int,
    mut wy: ::core::ffi::c_int,
    mut x: *mut u_int,
    mut y: *mut u_int,
) -> ::core::ffi::c_int {
    let mut sx: ::core::ffi::c_int = 0;
    let mut sy: ::core::ffi::c_int = 0;
    if wx < 0 as ::core::ffi::c_int || wy < 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if wx as u_int > (*bctx).w.logical_size().0 || wy as u_int > (*bctx).w.logical_size().1 {
        return 0 as ::core::ffi::c_int;
    }
    if wx < (*bctx).ox as ::core::ffi::c_int || wy < (*bctx).oy as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    sx = wx - (*bctx).ox as ::core::ffi::c_int;
    sy = wy - (*bctx).oy as ::core::ffi::c_int;
    if sx as u_int >= (*bctx).sx || sy as u_int >= (*bctx).sy {
        return 0 as ::core::ffi::c_int;
    }
    *x = sx as u_int;
    *y = sy as u_int;
    1 as ::core::ffi::c_int
}
unsafe fn redraw_pane_to_scene(
    mut bctx: *mut redraw_build_ctx,
    wp: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
    mut x: *mut u_int,
    mut y: *mut u_int,
) -> ::core::ffi::c_int {
    let mut wx: ::core::ffi::c_int = wp.geometry().2 + px;
    let mut wy: ::core::ffi::c_int = wp.geometry().3 + py;
    redraw_window_to_scene(bctx, wx, wy, x, y)
}
unsafe fn redraw_get_cell_type(mut mask: ::core::ffi::c_int) -> ::core::ffi::c_int {
    match mask {
        15 => return 11 as ::core::ffi::c_int,
        7 => return 8 as ::core::ffi::c_int,
        11 => return 7 as ::core::ffi::c_int,
        3 | REDRAW_BORDER_L | REDRAW_BORDER_R => return 2 as ::core::ffi::c_int,
        13 => return 10 as ::core::ffi::c_int,
        5 => return 6 as ::core::ffi::c_int,
        9 => return 4 as ::core::ffi::c_int,
        14 => return 9 as ::core::ffi::c_int,
        6 => return 5 as ::core::ffi::c_int,
        10 => return 3 as ::core::ffi::c_int,
        12 | REDRAW_BORDER_U | REDRAW_BORDER_D => return 1 as ::core::ffi::c_int,
        _ => {}
    }
    12 as ::core::ffi::c_int
}
/// Two strip panes share one vertical separator, which splits its colour.
unsafe fn redraw_check_two_pane_colours(w: &WindowRef) -> bool {
    w.pane_snapshot().len() == 2
}

unsafe fn redraw_mark_pane_inside(
    mut bctx: *mut redraw_build_ctx,
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
) {
    let wp = wp_owner;
    let mut bc: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    py = 0 as u_int;
    while py < wp.geometry().1 {
        px = 0 as u_int;
        while px < wp.geometry().0 {
            if !(redraw_pane_to_scene(
                bctx,
                wp,
                px as ::core::ffi::c_int,
                py as ::core::ffi::c_int,
                &raw mut x,
                &raw mut y,
            ) == 0)
            {
                bc = redraw_get_build_cell(bctx, x, y);
                (*bc).data = redraw_span_data::Pane(Default::default());
                (*bc).data.pane_mut().wp = std::rc::Rc::downgrade(wp);
                (*bc).data.pane_mut().px = px;
                (*bc).data.pane_mut().py = py;
            }
            px = px.wrapping_add(1);
        }
        py = py.wrapping_add(1);
    }
}
unsafe fn redraw_mark_pane_scrollbar(
    mut bctx: *mut redraw_build_ctx,
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut sb_w: ::core::ffi::c_int,
    mut sb_left: ::core::ffi::c_int,
    mut overlay: ::core::ffi::c_int,
) {
    let wp = wp_owner;
    let mut bc: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut wx: ::core::ffi::c_int = 0;
    let mut wy: ::core::ffi::c_int = 0;
    let mut sx: ::core::ffi::c_int = 0;
    let mut ex: ::core::ffi::c_int = 0;
    let mut sy: u_int = 0;
    if sb_w == 0 as ::core::ffi::c_int {
        return;
    }
    if overlay != 0 && sb_left != 0 {
        sx = wp.geometry().2;
        ex = sx + sb_w - 1 as ::core::ffi::c_int;
    } else if overlay != 0 {
        ex = wp.geometry().2 + wp.geometry().0 as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        sx = ex - sb_w + 1 as ::core::ffi::c_int;
    } else if sb_left != 0 {
        sx = wp.geometry().2 - sb_w;
        ex = wp.geometry().2 - 1 as ::core::ffi::c_int;
    } else {
        sx = wp.geometry().2 + wp.geometry().0 as ::core::ffi::c_int;
        ex = sx + sb_w - 1 as ::core::ffi::c_int;
    }
    sy = 0 as u_int;
    while sy < wp.geometry().1 {
        wy = wp.geometry().3 + sy as ::core::ffi::c_int;
        wx = sx;
        while wx <= ex {
            if !(redraw_window_to_scene(bctx, wx, wy, &raw mut x, &raw mut y) == 0) {
                bc = redraw_get_build_cell(bctx, x, y);
                (*bc).data = redraw_span_data::Scrollbar(Default::default());
                (*bc).data.scrollbar_mut().wp = std::rc::Rc::downgrade(wp);
                (*bc).data.scrollbar_mut().y = sy;
                (*bc).data.scrollbar_mut().height = wp.geometry().1;
                if sb_left != 0 {
                    (*bc).data.scrollbar_mut().flags |= REDRAW_SCROLLBAR_LEFT;
                } else {
                    (*bc).data.scrollbar_mut().flags |= REDRAW_SCROLLBAR_RIGHT;
                }
                if overlay != 0 {
                    (*bc).data.scrollbar_mut().flags |= REDRAW_SCROLLBAR_OVERLAY;
                }
            }
            wx += 1;
        }
        sy = sy.wrapping_add(1);
    }
}
fn redraw_data_has_pane(
    data: &redraw_span_data,
    pane: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
) -> bool {
    let border = data.border();
    [
        &border.top_wp,
        &border.bottom_wp,
        &border.left_wp,
        &border.right_wp,
    ]
    .iter()
    .any(|observer| observer.ptr_eq(pane))
}
unsafe fn redraw_mark_border_cell(
    mut bctx: *mut redraw_build_ctx,
    mut wx: ::core::ffi::c_int,
    mut wy: ::core::ffi::c_int,
    wp: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut top_owner: ::core::ffi::c_int,
    mut bottom_owner: ::core::ffi::c_int,
    mut mask: ::core::ffi::c_int,
    mut pane_lines: pane_lines,
) {
    let mut bc: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut reset: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if redraw_window_to_scene(bctx, wx, wy, &raw mut x, &raw mut y) == 0 {
        return;
    }
    bc = redraw_get_build_cell(bctx, x, y);
    if (*bc).data.kind() as ::core::ffi::c_uint
        == REDRAW_SPAN_EMPTY as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*bc).data.kind() as ::core::ffi::c_uint
            == REDRAW_SPAN_OUTSIDE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        reset = 1 as ::core::ffi::c_int;
    } else if (*bc).data.kind() as ::core::ffi::c_uint
        != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    if reset != 0 {
        (*bc).data = redraw_span_data::Border(Default::default());
    }
    if top_owner != 0 {
        (*bc).data.border_mut().top_wp = std::rc::Rc::downgrade(wp);
        (*bc).data.border_mut().top_lines = pane_lines;
    }
    if bottom_owner != 0 {
        (*bc).data.border_mut().bottom_wp = std::rc::Rc::downgrade(wp);
        (*bc).data.border_mut().bottom_lines = pane_lines;
    }
    if mask & (REDRAW_BORDER_U | REDRAW_BORDER_D) != 0 {
        if wx < wp.geometry().2 {
            (*bc).data.border_mut().right_wp = std::rc::Rc::downgrade(wp);
            (*bc).data.border_mut().right_lines = pane_lines;
        } else if wx >= wp.geometry().2 + wp.geometry().0 as ::core::ffi::c_int {
            (*bc).data.border_mut().left_wp = std::rc::Rc::downgrade(wp);
            (*bc).data.border_mut().left_lines = pane_lines;
        }
    }
    mask |= (*bc).data.border().cell_mask;
    (*bc).data.border_mut().cell_mask = mask;
    (*bc).data.border_mut().cell_type = redraw_get_cell_type(mask);
}
unsafe fn redraw_mark_border_status(
    mut bctx: *mut redraw_build_ctx,
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut right: ::core::ffi::c_int,
    mut top: ::core::ffi::c_int,
    mut bottom: ::core::ffi::c_int,
) {
    let wp = wp_owner;
    let mut bc: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut off: u_int = 0 as u_int;
    let mut pane_status: ::core::ffi::c_int = 0;
    let mut wy: ::core::ffi::c_int = 0;
    let mut sx: ::core::ffi::c_int = 0;
    let mut ex: ::core::ffi::c_int = 0;
    let mut wx: ::core::ffi::c_int = 0;
    let mut cell_type: ::core::ffi::c_int = 0;
    pane_status = wp.border_status();
    if pane_status == PANE_STATUS_OFF {
        return;
    }
    if pane_status == PANE_STATUS_TOP {
        wy = top;
    } else {
        wy = bottom;
    }
    sx = wp.geometry().2 + 2 as ::core::ffi::c_int;
    ex = right - 1 as ::core::ffi::c_int;
    if sx > ex {
        return;
    }
    wx = sx;
    while wx <= ex {
        if !(redraw_window_to_scene(bctx, wx, wy, &raw mut x, &raw mut y) == 0) {
            bc = redraw_get_build_cell(bctx, x, y);
            if !((*bc).data.kind() as ::core::ffi::c_uint
                != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                cell_type = (*bc).data.border().cell_type;
                (*bc).data = redraw_span_data::Status(Default::default());
                (*bc).data.status_mut().wp = std::rc::Rc::downgrade(wp);
                (*bc).data.status_mut().offset = off;
                (*bc).data.status_mut().cell_type = cell_type;
            }
        }
        wx += 1;
        off = off.wrapping_add(1);
    }
}
unsafe fn redraw_mark_border_arrows(
    mut bctx: *mut redraw_build_ctx,
    wp: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut left: ::core::ffi::c_int,
    mut right: ::core::ffi::c_int,
    mut top: ::core::ffi::c_int,
    mut bottom: ::core::ffi::c_int,
) {
    let mut bc: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut wx: ::core::ffi::c_int = 0;
    let mut wy: ::core::ffi::c_int = 0;
    if (*bctx).ind != PANE_BORDER_ARROWS && (*bctx).ind != PANE_BORDER_BOTH {
        return;
    }
    wx = wp.geometry().2 + 1 as ::core::ffi::c_int;
    if wx >= left && wx <= right {
        wy = top;
        if redraw_window_to_scene(bctx, wx, wy, &raw mut x, &raw mut y) != 0 {
            bc = redraw_get_build_cell(bctx, x, y);
            if (*bc).data.kind() as ::core::ffi::c_uint
                == REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*bc).data.border_mut().flags |= REDRAW_BORDER_IS_ARROW;
            }
        }
        wy = bottom;
        if redraw_window_to_scene(bctx, wx, wy, &raw mut x, &raw mut y) != 0 {
            bc = redraw_get_build_cell(bctx, x, y);
            if (*bc).data.kind() as ::core::ffi::c_uint
                == REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*bc).data.border_mut().flags |= REDRAW_BORDER_IS_ARROW;
            }
        }
    }
    wy = wp.geometry().3 + 1 as ::core::ffi::c_int;
    if wy >= top && wy <= bottom {
        wx = left;
        if redraw_window_to_scene(bctx, wx, wy, &raw mut x, &raw mut y) != 0 {
            bc = redraw_get_build_cell(bctx, x, y);
            if (*bc).data.kind() as ::core::ffi::c_uint
                == REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*bc).data.border_mut().flags |= REDRAW_BORDER_IS_ARROW;
            }
        }
        wx = right;
        if redraw_window_to_scene(bctx, wx, wy, &raw mut x, &raw mut y) != 0 {
            bc = redraw_get_build_cell(bctx, x, y);
            if (*bc).data.kind() as ::core::ffi::c_uint
                == REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*bc).data.border_mut().flags |= REDRAW_BORDER_IS_ARROW;
            }
        }
    }
}
unsafe fn redraw_mark_pane_borders(
    mut bctx: *mut redraw_build_ctx,
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut sb_w: ::core::ffi::c_int,
    mut sb_left: ::core::ffi::c_int,
) {
    let wp = wp_owner;
    let mut pane_lines: pane_lines = wp.pane_lines();
    let mut pane_status: ::core::ffi::c_int = 0;
    let mut left: ::core::ffi::c_int = 0;
    let mut right: ::core::ffi::c_int = 0;
    let mut top: ::core::ffi::c_int = 0;
    let mut bottom: ::core::ffi::c_int = 0;
    let mut wx: ::core::ffi::c_int = 0;
    let mut wy: ::core::ffi::c_int = 0;
    let mut mark_top: ::core::ffi::c_int = 0;
    let mut mark_bottom: ::core::ffi::c_int = 0;
    let mut mark_left: ::core::ffi::c_int = 0;
    let mut mark_right: ::core::ffi::c_int = 0;
    let mut mask: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    pane_status = wp.border_status();
    left = wp.geometry().2 - 1 as ::core::ffi::c_int;
    right = (wp.geometry().2 as u_int).wrapping_add(wp.geometry().0) as ::core::ffi::c_int;
    if sb_w != 0 as ::core::ffi::c_int {
        if sb_left != 0 {
            left -= sb_w;
        } else {
            right += sb_w;
        }
    }
    top = wp.geometry().3 - 1 as ::core::ffi::c_int;
    bottom = (wp.geometry().3 as u_int).wrapping_add(wp.geometry().1) as ::core::ffi::c_int;
    mark_left = (left >= 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    mark_top = (top >= 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    mark_right = (right <= (*bctx).w.logical_size().0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    mark_bottom =
        (bottom <= (*bctx).w.logical_size().1 as ::core::ffi::c_int) as ::core::ffi::c_int;
    if pane_status == PANE_STATUS_TOP && bottom < (*bctx).w.logical_size().1 as ::core::ffi::c_int {
        mark_bottom = 0 as ::core::ffi::c_int;
    } else if pane_status == PANE_STATUS_BOTTOM {
        mark_top = 0 as ::core::ffi::c_int;
    }
    if mark_top != 0 {
        wx = left;
        while wx <= right {
            mask = 0 as ::core::ffi::c_int;
            if wx > left {
                mask |= REDRAW_BORDER_L;
            }
            if wx < right {
                mask |= REDRAW_BORDER_R;
            }
            redraw_mark_border_cell(
                bctx,
                wx,
                top,
                wp,
                0 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
                mask,
                pane_lines,
            );
            wx += 1;
        }
    }
    if mark_bottom != 0 {
        wx = left;
        while wx <= right {
            mask = 0 as ::core::ffi::c_int;
            if wx > left {
                mask |= REDRAW_BORDER_L;
            }
            if wx < right {
                mask |= REDRAW_BORDER_R;
            }
            redraw_mark_border_cell(
                bctx,
                wx,
                bottom,
                wp,
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                mask,
                pane_lines,
            );
            wx += 1;
        }
    }
    if mark_left != 0 {
        wy = top;
        while wy <= bottom {
            mask = 0 as ::core::ffi::c_int;
            if wy > top {
                mask |= REDRAW_BORDER_U;
            }
            if wy < bottom {
                mask |= REDRAW_BORDER_D;
            }
            redraw_mark_border_cell(
                bctx,
                left,
                wy,
                wp,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                mask,
                pane_lines,
            );
            wy += 1;
        }
    }
    if mark_right != 0 {
        wy = top;
        while wy <= bottom {
            mask = 0 as ::core::ffi::c_int;
            if wy > top {
                mask |= REDRAW_BORDER_U;
            }
            if wy < bottom {
                mask |= REDRAW_BORDER_D;
            }
            redraw_mark_border_cell(
                bctx,
                right,
                wy,
                wp,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                mask,
                pane_lines,
            );
            wy += 1;
        }
    }
    redraw_mark_border_status(bctx, wp_owner, right, top, bottom);
    redraw_mark_border_arrows(bctx, wp, left, right, top, bottom);
}
unsafe fn redraw_mark_pane(
    mut bctx: *mut redraw_build_ctx,
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
) {
    let wp = wp_owner;
    let mut sb_w: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut sb_left: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut overlay: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if wp.scrollbar_visible() {
        overlay = wp.scrollbar_overlay() as i32;
        if overlay != 0 {
            sb_w = wp.scrollbar_width() + wp.scrollbar_pad();
            if sb_w > wp.geometry().0 as ::core::ffi::c_int {
                sb_w = wp.scrollbar_width();
                if sb_w > wp.geometry().0 as ::core::ffi::c_int {
                    sb_w = wp.geometry().0 as ::core::ffi::c_int;
                }
            }
        } else {
            sb_w = wp.scrollbar_width() + wp.scrollbar_pad();
        }
    }
    if sb_w != 0 as ::core::ffi::c_int && ((*bctx).w).scrollbar_position() == PANE_SCROLLBARS_LEFT {
        sb_left = 1 as ::core::ffi::c_int;
    }
    redraw_mark_pane_inside(bctx, wp_owner);
    redraw_mark_pane_borders(
        bctx,
        wp_owner,
        if overlay != 0 {
            0 as ::core::ffi::c_int
        } else {
            sb_w
        },
        sb_left,
    );
    redraw_mark_pane_scrollbar(bctx, wp_owner, sb_w, sb_left, overlay);
}
unsafe fn redraw_mark_two_pane_colours(mut bctx: *mut redraw_build_ctx) {
    let mut bc: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut sd: *mut redraw_span_data = ::core::ptr::null_mut::<redraw_span_data>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut wy: u_int = 0;
    if (*bctx).ind != PANE_BORDER_COLOUR && (*bctx).ind != PANE_BORDER_BOTH {
        return;
    }
    if !redraw_check_two_pane_colours((*bctx).w) {
        return;
    }
    y = 0 as u_int;
    while y < (*bctx).sy {
        x = 0 as u_int;
        while x < (*bctx).sx {
            bc = redraw_get_build_cell(bctx, x, y);
            if !((*bc).data.kind() as ::core::ffi::c_uint
                != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                sd = &raw mut (*bc).data;
                wy = (*bctx).oy.wrapping_add(y);
                if (*sd).border().left_wp.strong_count() != 0
                    && (*sd).border().right_wp.strong_count() != 0
                {
                    if wy <= (*bctx).w.logical_size().1.wrapping_div(2 as u_int) {
                        (*sd).border_mut().style_wp = (*sd).border().left_wp.clone();
                    } else {
                        (*sd).border_mut().style_wp = (*sd).border().right_wp.clone();
                    }
                }
            }
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
}
fn redraw_compare_data(a: &redraw_build_cell, b: &redraw_build_cell) -> bool {
    use redraw_span_data::*;
    match (&a.data, &b.data) {
        (Pane(a), Pane(b)) => a.wp.ptr_eq(&b.wp) && a.py == b.py && a.px.wrapping_add(1) == b.px,
        (Border(a), Border(b)) => a == b && a.flags & REDRAW_BORDER_IS_ARROW == 0,
        (Status(a), Status(b)) => {
            a.wp.ptr_eq(&b.wp) && a.offset.wrapping_add(1) == b.offset && a.cell_type == b.cell_type
        }
        (Scrollbar(a), Scrollbar(b)) => a == b,
        (Outside, Outside) | (Empty, Empty) => true,
        _ => false,
    }
}
unsafe fn redraw_build_cells<'a>(
    mut bctx: *mut redraw_build_ctx<'a>,
    cells: &'a mut Vec<redraw_build_cell>,
) {
    let mut ncells: size_t = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*bctx).sx != 0 as u_int
        && (*bctx).sy as ::core::ffi::c_ulong
            > SIZE_MAX.wrapping_div((*bctx).sx as ::core::ffi::c_ulong)
    {
        fatalx(|out| {
            write_cstr(out, c"redraw_build_cells".as_ptr())?;
            out.write_all(b": too many cells")
        });
    }
    ncells = ((*bctx).sx as size_t).wrapping_mul((*bctx).sy as size_t);
    if ncells > cells.len() {
        if cells.try_reserve_exact(ncells - cells.len()).is_err() {
            fatalx(|out| {
                write_cstr(out, c"redraw_build_cells".as_ptr())?;
                out.write_all(b": too many cells")
            });
        }
        cells.resize_with(ncells, redraw_build_cell::default);
    }
    (*bctx).cells = &mut cells[..ncells];
    y = 0 as u_int;
    while y < (*bctx).sy {
        x = 0 as u_int;
        while x < (*bctx).sx {
            redraw_reset_cell(bctx, x, y);
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
    for pane_owner in (*bctx).w.pane_snapshot().into_iter().rev() {
        redraw_mark_pane(bctx, &pane_owner);
    }
    redraw_mark_two_pane_colours(bctx);
}
unsafe fn redraw_make_scene(client_owner: &ClientRef) -> Option<Box<redraw_scene>> {
    let mut c: Option<ClientRef> = Some(client_owner.clone());
    let mut s: Option<SessionRef> = c
        .as_ref()
        .expect("live client")
        .attached_session()
        .upgrade();
    let window_owner = s
        .as_ref()
        .expect("live session")
        .current_winlink()
        .get_unchecked()
        .window_handle()
        .cloned()
        .expect("current window");
    let mut bctx: redraw_build_ctx = redraw_build_ctx {
        w: &window_owner,
        ox: 0,
        oy: 0,
        sx: 0,
        sy: 0,
        ind: 0,
        cells: &mut [],
    };
    let mut bc: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut last: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut type_0: redraw_span_type = REDRAW_SPAN_PANE;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut x0: u_int = 0;
    if c.as_ref().expect("live client").flags() & CLIENT_SUSPENDED as uint64_t != 0 {
        window_owner.release(c"suspended redraw scene");
        return None;
    }
    redraw_set_context(client_owner, &mut bctx);
    let mut cells = RedrawCellScratch::take();
    log_debug(format_args!(
        "{}: building @{} scene ({}x{} {},{}; generation {})",
        log_cstr(
            ((c.as_ref().expect("live client").name())
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        { window_owner.id() },
        { bctx.sx },
        { bctx.sy },
        { bctx.ox },
        { bctx.oy },
        window_owner.scene_generation() as ::core::ffi::c_ulonglong
    ));
    redraw_build_cells(&raw mut bctx, &mut cells.0);
    let mut scene = Box::new(redraw_scene {
        c: std::rc::Rc::downgrade(client_owner),
        w: std::rc::Rc::downgrade(&window_owner).clone(),
        lines: Box::default(),
        generation: window_owner.scene_generation(),
        sx: bctx.sx,
        sy: bctx.sy,
        ox: bctx.ox,
        oy: bctx.oy,
    });
    if bctx.sy == 0 {
        fatalx(|out| out.write_all(b"xcalloc: zero size"));
    }
    // Keep row storage fixed for the lifetime of the scene; boxed spans keep
    // their addresses stable as each collection grows during construction.
    let lines = ::std::iter::repeat_with(redraw_line::default)
        .take(bctx.sy as usize)
        .collect::<Vec<_>>()
        .into_boxed_slice();
    scene.lines = lines;
    y = 0 as u_int;
    while y < bctx.sy {
        let line = &mut scene.lines[y as usize];
        x = 0 as u_int;
        while x < bctx.sx {
            x0 = x;
            last = redraw_get_build_cell(&raw mut bctx, x, y);
            x = x.wrapping_add(1);
            while x < bctx.sx {
                bc = redraw_get_build_cell(&raw mut bctx, x, y);
                if !redraw_compare_data(&*last, &*bc) {
                    break;
                }
                last = bc;
                x = x.wrapping_add(1);
            }
            bc = redraw_get_build_cell(&raw mut bctx, x0, y);
            type_0 = (*bc).data.kind();
            // The scene owns each stable span until its row collection is dropped.
            line[type_0 as usize].push(Box::new(redraw_span {
                x: x0,
                width: x.wrapping_sub(x0),
                data: (*bc).data.clone(),
            }));
        }
        y = y.wrapping_add(1);
    }
    log_debug(format_args!(
        "{}: finished building @{} scene",
        log_cstr(
            ((c.as_ref().expect("live client").name())
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        { window_owner.id() }
    ));
    window_owner.release(c"redraw scene build");
    Some(scene)
}

pub unsafe fn redraw_invalidate_all_scenes() {
    let mut window_cursor = windows.first();
    while let Some(window_owner) = window_cursor.take() {
        window_owner.invalidate_scene();
        window_cursor = window_owner.next_window();
        window_owner.release(c"window traversal");
    }
}
unsafe fn redraw_get_scene(client_owner: &ClientRef) -> Option<Box<redraw_scene>> {
    let mut c: Option<ClientRef> = Some(client_owner.clone());
    let window_owner = (c
        .as_ref()
        .expect("live client")
        .attached_session()
        .upgrade()
        .expect("live session")
        .current_winlink())
    .get_unchecked()
    .window_handle()
    .cloned()
    .expect("current window");
    let tty_window_view { ox, oy, sx, sy, .. } = redraw_get_window_offset(client_owner);
    let scene = client_owner.take_redraw_scene();
    let reason = match scene.as_deref() {
        None => Some("missing"),
        Some(scene) if !scene.w.ptr_eq(&std::rc::Rc::downgrade(&window_owner)) => {
            Some("window changed")
        }
        Some(scene) if scene.generation != window_owner.scene_generation() => {
            Some("generation changed")
        }
        Some(scene) if scene.ox != ox || scene.oy != oy => Some("offset changed"),
        Some(scene) if scene.sx != sx || scene.sy != sy => Some("size changed"),
        Some(_) => None,
    };
    let result = if let Some(reason) = reason {
        log_debug(format_args!(
            "{}: @{} scene invalid: {}",
            log_cstr(
                c.as_ref()
                    .expect("live client")
                    .name()
                    .as_ref()
                    .map_or(std::ptr::null(), |name| name.as_ptr())
            ),
            window_owner.id(),
            reason
        ));
        drop(scene);
        redraw_make_scene(client_owner)
    } else {
        scene
    };
    window_owner.release(c"redraw scene lookup");
    result
}

unsafe fn redraw_draw_pane_span(
    dctx: &mut redraw_draw_ctx<'_>,
    span: &redraw_span,
    mut x: u_int,
    mut y: u_int,
    mut n: u_int,
) {
    let Some(pane) = span.data.pane().wp.upgrade() else {
        return;
    };
    let Some(client) = dctx.scene.c.upgrade() else {
        return;
    };
    pane.draw_line(
        &client,
        (
            span.data.pane().px.wrapping_add(x.wrapping_sub(span.x)),
            span.data.pane().py,
        ),
        n,
        (x, y),
        false,
    );
}
unsafe fn redraw_get_default_border_style(
    dctx: &mut redraw_draw_ctx<'_>,
    mut gc: *mut grid_cell,
    mut pane_lines: *mut pane_lines,
) {
    let scene = dctx.scene;
    let Some(client_owner) = scene.c.upgrade() else {
        return;
    };
    let mut c: Option<ClientRef> = Some(client_owner.clone());
    let mut s: Option<SessionRef> = c
        .as_ref()
        .expect("live client")
        .attached_session()
        .upgrade();
    let Some(window_owner) = scene.w.upgrade() else {
        return;
    };
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut dgc: *mut grid_cell = &mut dctx.default_gc;
    if !dctx.flags & REDRAW_DEFAULT_SET != 0 {
        let mut ft_owner = format_create_defaults(
            None,
            c.as_ref(),
            s.as_ref(),
            (s.as_ref().expect("live session").current_winlink()).clone(),
            None,
        );
        ft = &raw mut *ft_owner;
        style_apply_with_options(&mut *dgc, c"pane-border-style", Some(&mut *ft), |visit| {
            window_owner.with_options_mut(visit)
        });
        format_free(ft_owner);
        dctx.pane_lines = window_owner.pane_border_lines();
        dctx.flags |= REDRAW_DEFAULT_SET;
    }
    memcpy(
        gc as *mut ::core::ffi::c_void,
        dgc as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    *pane_lines = dctx.pane_lines;
    window_owner.release(c"default border style");
}
fn redraw_get_pane_for_border_style(
    active: Option<&std::rc::Weak<std::cell::UnsafeCell<window_pane>>>,
    span: &redraw_span,
) -> Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
    let redraw_span_data::Border(border) = &span.data else {
        return None;
    };
    if let Some(owner) = border.style_wp.upgrade() {
        return Some(owner);
    }
    if let Some(active) = active {
        if redraw_data_has_pane(&span.data, active) {
            if let Some(owner) = active.upgrade() {
                return Some(owner);
            }
        }
    }
    [
        &border.top_wp,
        &border.bottom_wp,
        &border.left_wp,
        &border.right_wp,
    ]
    .iter()
    .find_map(|observer| observer.upgrade())
}
unsafe fn redraw_draw_border_arrow(
    dctx: &mut redraw_draw_ctx<'_>,
    span: &redraw_span,
    mut gc: *mut grid_cell,
) {
    let active = &dctx.active;
    let mut ch: ::core::ffi::c_char = 0;
    if span.data.kind() as ::core::ffi::c_uint
        != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
        || active.strong_count() == 0
    {
        return;
    }
    if !span.data.border().flags & REDRAW_BORDER_IS_ARROW != 0 {
        return;
    }
    if span.data.border().left_wp.ptr_eq(active) {
        ch = ',' as i32 as ::core::ffi::c_char;
    } else if span.data.border().right_wp.ptr_eq(active) {
        ch = '+' as i32 as ::core::ffi::c_char;
    } else if span.data.border().top_wp.ptr_eq(active) {
        ch = '-' as i32 as ::core::ffi::c_char;
    } else if span.data.border().bottom_wp.ptr_eq(active) {
        ch = '.' as i32 as ::core::ffi::c_char;
    } else {
        return;
    }
    utf8_set(&mut (*gc).data, ch as u_char);
    (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
}
unsafe fn redraw_draw_border_span(
    dctx: &mut redraw_draw_ctx<'_>,
    span: &redraw_span,
    mut x: u_int,
    mut y: u_int,
    mut n: u_int,
) {
    let scene = dctx.scene;
    let Some(client_owner) = scene.c.upgrade() else {
        return;
    };
    let _c: Option<ClientRef> = Some(client_owner.clone());
    let Some(window_owner) = scene.w.upgrade() else {
        return;
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
    let mut pane_lines: pane_lines = PANE_LINES_SINGLE;
    let mut i: u_int = 0;
    let mut cell_type: u_int = 0;
    let mut border_pane_owner = None;
    let mut isolates: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if span.data.kind() as ::core::ffi::c_uint
        != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        cell_type = CELL_NONE as u_int;
    } else {
        border_pane_owner = redraw_get_pane_for_border_style(Some(&dctx.active), span);
        cell_type = span.data.border().cell_type as u_int;
    }
    if border_pane_owner.is_none() {
        redraw_get_default_border_style(dctx, &raw mut gc, &raw mut pane_lines);
        if span.data.kind() as ::core::ffi::c_uint
            == REDRAW_SPAN_OUTSIDE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            window_get_fill_cell(&window_owner, 0 as ::core::ffi::c_int, &raw mut gc);
        } else if span.data.kind() as ::core::ffi::c_uint
            == REDRAW_SPAN_EMPTY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            window_get_fill_cell(&window_owner, 1 as ::core::ffi::c_int, &raw mut gc);
        } else {
            if span.data.kind() as ::core::ffi::c_uint
                != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                pane_lines = PANE_LINES_SINGLE;
            }
            window_get_border_cell(None, pane_lines, cell_type as ::core::ffi::c_int, &mut gc);
        }
    } else {
        gc = border_pane_owner
            .as_ref()
            .expect("border pane")
            .border_style(&client_owner);
        border_pane_owner
            .as_ref()
            .expect("border pane")
            .border_cell(cell_type as i32, &mut gc);
    }
    if span.data.kind() as ::core::ffi::c_uint
        == REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
        && dctx.marked.strong_count() != 0
        && redraw_data_has_pane(&span.data, &dctx.marked)
    {
        gc.attr = (gc.attr as ::core::ffi::c_int ^ GRID_ATTR_REVERSE) as u_short;
    }
    redraw_draw_border_arrow(dctx, span, &raw mut gc);
    if cell_type == CELL_UD as u_int && dctx.flags & REDRAW_ISOLATES != 0 {
        isolates = 1 as ::core::ffi::c_int;
    }
    {
        let terminal = &(client_owner);
        tty_cursor(terminal, x, y)
    };
    if isolates != 0 {
        {
            let terminal = &(client_owner);
            tty_puts(terminal, REDRAW_END_ISOLATE)
        };
    }
    i = 0 as u_int;
    while i < n {
        tty_cell(&client_owner, &gc, None);
        i = i.wrapping_add(1);
    }
    if isolates != 0 {
        {
            let terminal = &(client_owner);
            tty_puts(terminal, REDRAW_START_ISOLATE)
        };
    }
}
unsafe fn redraw_draw_status_span(
    dctx: &mut redraw_draw_ctx<'_>,
    span: &redraw_span,
    mut x: u_int,
    mut y: u_int,
    mut n: u_int,
) {
    let Some(pane) = span.data.status().wp.upgrade() else {
        return;
    };
    let Some(client) = dctx.scene.c.upgrade() else {
        return;
    };
    pane.draw_line(
        &client,
        (
            span.data
                .status()
                .offset
                .wrapping_add(x.wrapping_sub(span.x)),
            0,
        ),
        n,
        (x, y),
        true,
    );
}
unsafe fn redraw_draw_scrollbar_span(
    dctx: &mut redraw_draw_ctx<'_>,
    span: &redraw_span,
    mut x: u_int,
    mut y: u_int,
    mut n: u_int,
) {
    let Some(pane_owner) = span.data.scrollbar().wp.upgrade() else {
        return;
    };
    let Some(client_owner) = dctx.scene.c.upgrade() else {
        return;
    };
    pane_owner.draw_scrollbar(&client_owner, span, x, y, n);
}
unsafe fn redraw_draw_span(dctx: &mut redraw_draw_ctx<'_>, span: &redraw_span, mut y: u_int) {
    let data = &span.data;
    let type_0: redraw_span_type = data.kind();
    if dctx.scene.c.strong_count() == 0 {
        return;
    }
    if type_0 as ::core::ffi::c_uint
        == REDRAW_SPAN_STATUS as ::core::ffi::c_int as ::core::ffi::c_uint
        && data
            .status()
            .wp
            .upgrade()
            .is_none_or(|owner| !owner.has_new_status())
    {
        return;
    }
    if span.width == 0 {
        return;
    }
    let (x, n) = (span.x, span.width);
    match type_0 as ::core::ffi::c_uint {
        0 => {
            redraw_draw_pane_span(dctx, span, x, y, n);
        }
        4 | 2 | 1 => {
            redraw_draw_border_span(dctx, span, x, y, n);
        }
        3 => {
            redraw_draw_status_span(dctx, span, x, y, n);
        }
        5 => {
            redraw_draw_scrollbar_span(dctx, span, x, y, n);
        }
        _ => {}
    }
}
unsafe fn redraw_draw_pane_lines(
    dctx: &mut redraw_draw_ctx<'_>,
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    flags: ::core::ffi::c_int,
) {
    let wp = wp_owner;
    let scene = dctx.scene;
    let top = (wp.geometry().3 - scene.oy as i32).max(0);
    let bottom = (wp.geometry().3 + wp.geometry().1 as i32 - scene.oy as i32)
        .max(0)
        .min(scene.sy as i32);
    for y in top..bottom {
        let line = &scene.lines[y as usize];
        let cy = if dctx.flags & REDRAW_STATUS_TOP != 0 {
            dctx.status_lines.wrapping_add(y as u_int)
        } else {
            y as u_int
        };
        if flags & REDRAW_PANE != 0 {
            for span in line[REDRAW_SPAN_PANE as usize].iter() {
                if span.data.pane().wp.ptr_eq(&std::rc::Rc::downgrade(wp)) {
                    redraw_draw_span(dctx, span, cy);
                }
            }
        }
        if flags & REDRAW_PANE_SCROLLBAR != 0 {
            for span in line[REDRAW_SPAN_SCROLLBAR as usize].iter() {
                if span.data.scrollbar().wp.ptr_eq(&std::rc::Rc::downgrade(wp)) {
                    redraw_draw_span(dctx, span, cy);
                }
            }
        }
    }
}
unsafe fn redraw_draw_lines(dctx: &mut redraw_draw_ctx<'_>, flags: ::core::ffi::c_int) {
    let scene = dctx.scene;
    let masks = [
        REDRAW_PANE,
        REDRAW_OUTSIDE,
        REDRAW_EMPTY,
        REDRAW_PANE_STATUS,
        REDRAW_PANE_BORDER,
        REDRAW_PANE_SCROLLBAR,
    ];
    for (y, line) in scene.lines.iter().enumerate() {
        let cy = if dctx.flags & REDRAW_STATUS_TOP != 0 {
            dctx.status_lines.wrapping_add(y as u_int)
        } else {
            y as u_int
        };
        for (spans, mask) in line.iter().zip(masks) {
            if flags != REDRAW_ALL && flags & mask == 0 {
                continue;
            }
            for span in spans.iter() {
                redraw_draw_span(dctx, span, cy);
            }
        }
    }
}
unsafe fn redraw_pane_status_line(
    dctx: &mut redraw_draw_ctx<'_>,
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut line: *mut u_int,
) -> ::core::ffi::c_int {
    let wp = wp_owner;
    let scene = dctx.scene;
    let mut pane_status: ::core::ffi::c_int = 0;
    let mut wy: ::core::ffi::c_int = 0;
    pane_status = wp.border_status();
    if pane_status == PANE_STATUS_OFF {
        return 0 as ::core::ffi::c_int;
    }
    if pane_status == PANE_STATUS_TOP {
        wy = wp.geometry().3 - 1 as ::core::ffi::c_int;
    } else {
        wy = (wp.geometry().3 as u_int).wrapping_add(wp.geometry().1) as ::core::ffi::c_int;
    }
    if wy < 0 as ::core::ffi::c_int || wy < scene.oy as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if wy as u_int >= scene.oy.wrapping_add(scene.sy) {
        return 0 as ::core::ffi::c_int;
    }
    *line = (wy as u_int).wrapping_sub(scene.oy);
    1 as ::core::ffi::c_int
}
unsafe fn redraw_pane_status_width<'scene>(
    dctx: &mut redraw_draw_ctx<'scene>,
    pane: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
) -> Option<(u_int, &'scene redraw_spans, usize)> {
    let wp = pane;
    let mut y = 0;
    if redraw_pane_status_line(dctx, wp, &mut y) == 0 {
        return None;
    }
    let spans = &dctx.scene.lines[y as usize][REDRAW_SPAN_STATUS as usize];
    let mut width = 0;
    let mut first_index = spans.len();
    for (index, span) in spans.iter().enumerate() {
        if span.data.status().wp.ptr_eq(&std::rc::Rc::downgrade(wp)) {
            if first_index == spans.len() {
                first_index = index;
            }
            width = width.max(span.data.status().offset.wrapping_add(span.width));
        }
    }
    Some((width, spans, first_index))
}
unsafe fn redraw_set_draw_context(scene: &redraw_scene) -> Option<redraw_draw_ctx<'_>> {
    let window_owner = scene.w.upgrade()?;
    let client_owner = scene.c.upgrade()?;
    let mut c: Option<ClientRef> = Some(client_owner.clone());
    let s = c
        .as_ref()
        .expect("live client")
        .attached_session()
        .upgrade();
    let mut dctx = redraw_draw_ctx {
        scene,
        active: window_owner.active_pane_observer(),
        marked: std::rc::Weak::new(),
        status_lines: status_line_size(c.as_ref().expect("live client")),
        pane_lines: PANE_LINES_SINGLE,
        default_gc: grid_cell::default(),
        flags: 0,
    };
    if server_is_marked(
        s.as_ref(),
        (s.as_ref().expect("live session").current_winlink()).clone(),
        marked_pane.pane_handle().as_ref(),
    ) != 0
    {
        dctx.marked = marked_pane.wp.clone();
    }
    if s.as_ref()
        .expect("live session")
        .with_options_mut(|options| options_get_number(options, c"status-position"))
        == 0
    {
        dctx.flags |= REDRAW_STATUS_TOP;
    }
    if c.as_ref().expect("live client").flags() & CLIENT_UTF8 as uint64_t != 0 && {
        let terminal = client_owner.borrow_terminal();
        tty_term_has(
            tty_term_owner_ptr(&terminal.term).map_or(std::ptr::null(), |term| term),
            TTYC_BIDI,
        ) != 0
    } {
        dctx.flags |= REDRAW_ISOLATES;
    }
    Some(dctx)
}
unsafe fn redraw_draw_pane_prompt(
    dctx: &mut redraw_draw_ctx<'_>,
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
) {
    wp_owner.draw_prompt(dctx);
}
unsafe fn redraw_draw(
    client_owner: &ClientRef,
    pane_owner: Option<&std::rc::Rc<std::cell::UnsafeCell<window_pane>>>,
    mut flags: ::core::ffi::c_int,
) {
    let mut c: Option<ClientRef> = Some(client_owner.clone());
    let s = c
        .as_ref()
        .expect("live client")
        .attached_session()
        .upgrade();
    let mut redraw = 0;
    if c.as_ref().expect("live client").flags() & CLIENT_SUSPENDED as uint64_t != 0 {
        return;
    }
    if flags & REDRAW_STATUS != 0 {
        if c.as_ref()
            .expect("live client")
            .status_message_text()
            .0
            .is_some()
        {
            redraw = status_message_redraw(&c.clone().expect("live client"));
        } else if c
            .as_ref()
            .expect("live client")
            .prompt_observer()
            .is_alive()
        {
            redraw = status_prompt_redraw(&c.clone().expect("live client"));
        } else {
            redraw = status_redraw(&c.clone().expect("live client"));
        }
        if redraw == 0 && !(flags == REDRAW_ALL) {
            flags &= !REDRAW_STATUS;
            if flags == 0 as ::core::ffi::c_int {
                return;
            }
        }
    }
    if log_get_level() != 0 as ::core::ffi::c_int {
        log_debug(format_args!(
            "{}: starting @{} redraw ({})",
            log_cstr(
                ((c.as_ref().expect("live client").name())
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            ),
            {
                (((s.as_ref().expect("live session").current_winlink())
                    .get_unchecked()
                    .window_handle()
                    .as_ref())
                .expect("live window"))
                .id()
            },
            log_cstr(redraw_flags_to_string(flags).as_ptr())
        ));
    }
    let Some(scene) = redraw_get_scene(client_owner) else {
        return;
    };
    redraw_draw_scene(client_owner, pane_owner, flags, &scene);
    client_owner.restore_redraw_scene(scene);
}

unsafe fn redraw_draw_scene(
    client_owner: &ClientRef,
    pane_owner: Option<&std::rc::Rc<std::cell::UnsafeCell<window_pane>>>,
    mut flags: ::core::ffi::c_int,
    scene: &redraw_scene,
) {
    let mut c: Option<ClientRef> = Some(client_owner.clone());
    let wp = pane_owner;
    let Some(window_owner) = scene.w.upgrade() else {
        return;
    };
    let mut _s: Option<SessionRef> = c
        .as_ref()
        .expect("live client")
        .attached_session()
        .upgrade();
    let mut i: u_int = 0;
    let mut y: u_int = 0;
    let mut lines: u_int = 0;
    let _j: u_int = 0;
    let mut redraw: ::core::ffi::c_int = 0;
    let Some(mut dctx) = redraw_set_draw_context(scene) else {
        window_owner.release(c"unavailable redraw context");
        return;
    };
    if flags & (REDRAW_PANE_BORDER | REDRAW_PANE_STATUS) != 0 {
        for pane_owner in window_owner.pane_snapshot() {
            pane_owner.reset_border_cache();
        }
    }
    if flags & REDRAW_PANE_STATUS != 0 {
        redraw = 0 as ::core::ffi::c_int;
        for pane_owner in window_owner.pane_snapshot() {
            pane_owner.set_new_status(flags == REDRAW_ALL);
            if let Some((width, status_spans, first_status_span)) =
                redraw_pane_status_width(&mut dctx, &pane_owner)
            {
                if width != 0
                    && pane_owner.make_status(client_owner, width, status_spans, first_status_span)
                {
                    pane_owner.set_new_status(true);
                    redraw = 1 as ::core::ffi::c_int;
                }
            }
        }
        if redraw == 0 && !(flags == REDRAW_ALL) {
            flags &= !REDRAW_PANE_STATUS;
            if flags == 0 as ::core::ffi::c_int {
                window_owner.release(c"unchanged pane status");
                return;
            }
        }
    }
    if flags & REDRAW_PANE != 0 {
        if let Some(pane) = pane_owner {
            pane.stop_sync();
            pane.clear_sync_dirty();
        } else {
            for pane in window_owner.pane_snapshot() {
                pane.stop_sync();
                pane.clear_sync_dirty();
            }
        }
    }
    {
        let terminal = &(client_owner);
        tty_sync_start(terminal);
        tty_update_mode(
            terminal,
            {
                let tty_state = terminal.borrow_terminal();
                tty_state.mode
            } & !CURSOR_MODES,
            None,
        );
    };
    if let Some(wp) = wp {
        redraw_draw_pane_lines(&mut dctx, wp, flags);
    } else {
        redraw_draw_lines(&mut dctx, flags);
    }
    if flags & REDRAW_PANE != 0 {
        if let Some(wp) = wp {
            redraw_draw_pane_prompt(&mut dctx, wp);
        } else {
            for pane_owner in window_owner.pane_snapshot() {
                redraw_draw_pane_prompt(&mut dctx, &pane_owner);
            }
        }
    }
    if flags & REDRAW_STATUS != 0 {
        lines = dctx.status_lines;
        if c.as_ref()
            .expect("live client")
            .status_message_text()
            .0
            .is_some()
            || c.as_ref()
                .expect("live client")
                .prompt_observer()
                .is_alive()
        {
            lines = if lines == 0 as u_int {
                1 as u_int
            } else {
                lines
            };
        }
        if dctx.flags & REDRAW_STATUS_TOP != 0 {
            y = 0 as u_int;
        } else {
            y = client_owner.terminal_size().1.wrapping_sub(lines);
        }
        i = 0;
        while i < lines {
            let width = client_owner.terminal_size().0;
            if width != 0 {
                client_owner.draw_status_line(i, 0, width, y.wrapping_add(i));
            }
            i = i.wrapping_add(1);
        }
    }
    {
        let terminal = &(client_owner);
        tty_reset(terminal)
    };
    log_debug(format_args!(
        "{}: finished @{} redraw",
        log_cstr(
            ((c.as_ref().expect("live client").name())
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        { (window_owner).id() }
    ));
    window_owner.release(c"redraw scene");
}
pub fn redraw_get_status_border_cell_type(
    spans: &redraw_spans,
    span_index: &mut usize,
    mut x: u_int,
) -> ::core::ffi::c_int {
    let mut start: u_int = 0;
    let mut end: u_int = 0;
    let entries = &spans;
    let mut index = *span_index;
    if index >= entries.len()
        || entries[index].data.kind() as ::core::ffi::c_uint
            != REDRAW_SPAN_STATUS as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 2 as ::core::ffi::c_int;
    }
    let pane_observer = &entries[index].data.status().wp;
    while index < entries.len() {
        let span = entries[index].as_ref();
        if !(span.data.kind() as ::core::ffi::c_uint
            != REDRAW_SPAN_STATUS as ::core::ffi::c_int as ::core::ffi::c_uint)
            && span.data.status().wp.ptr_eq(pane_observer)
        {
            start = span.data.status().offset;
            end = start.wrapping_add(span.width);
            if x >= start && x < end {
                *span_index = index;
                return span.data.status().cell_type;
            }
            if start > x {
                *span_index = index;
                break;
            }
        }
        index += 1;
    }
    if index == entries.len() {
        *span_index = entries.len();
    }
    2 as ::core::ffi::c_int
}
pub unsafe fn redraw_screen(client_owner: &ClientRef) {
    let mut c: Option<ClientRef> = Some(client_owner.clone());
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if c.as_ref().expect("live client").flags() & CLIENT_REDRAWWINDOW as uint64_t != 0 {
        redraw_draw(client_owner, None, REDRAW_ALL);
    } else {
        if c.as_ref().expect("live client").flags() & CLIENT_REDRAWBORDERS as uint64_t != 0 {
            flags |= REDRAW_PANE_BORDER | REDRAW_PANE_STATUS;
        }
        if c.as_ref().expect("live client").flags() & CLIENT_REDRAWSTATUS as uint64_t != 0 {
            flags |= REDRAW_STATUS | REDRAW_PANE_STATUS;
        }
        if flags != 0 as ::core::ffi::c_int {
            redraw_draw(client_owner, None, flags);
        }
    };
}
pub unsafe fn redraw_pane(
    client_owner: &ClientRef,
    pane_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
) {
    redraw_draw(
        client_owner,
        Some(pane_owner),
        REDRAW_PANE | REDRAW_PANE_SCROLLBAR,
    );
}
pub unsafe fn redraw_pane_scrollbar(
    client_owner: &ClientRef,
    pane_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
) {
    redraw_draw(client_owner, Some(pane_owner), REDRAW_PANE_SCROLLBAR);
}
