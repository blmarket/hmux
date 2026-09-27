use crate::src::tty_term::tty_term_owner_ptr;
use crate::src::options::options_owner_ptr;
use crate::src::ffi::libc::{memcpy, memset};
use crate::src::format::bytes::write_cstr;
use crate::src::format::{format_create_defaults, format_free};
use crate::src::grid::grid_default_cell;
use crate::src::log::{fatalx, log_cstr, log_debug, log_get_level};
use crate::src::menu::{menu_height, menu_screen, menu_update, menu_width, menu_x, menu_y};
use crate::src::options::options_get_number;
use crate::src::prompt::prompt_draw;
use crate::src::screen::{screen_free, screen_init};
use crate::src::screen_write::{
    screen_write_clear_dirty, screen_write_start, screen_write_stop, screen_write_stop_sync,
};
use crate::src::server::{marked_pane, server_is_marked};
use crate::src::server_client::server_client_overlay_draw;
use crate::src::shared::abi::*;
use crate::src::shared::borders::{CELL_NONE, CELL_UD};
use crate::src::shared::client::client;
use crate::src::shared::client::{
    CLIENT_REDRAWBORDERS, CLIENT_REDRAWMENU, CLIENT_REDRAWOVERLAY, CLIENT_REDRAWSTATUS,
    CLIENT_REDRAWWINDOW, CLIENT_SUSPENDED, CLIENT_UTF8,
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
use crate::src::shared::style::*;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::tty::*;
use crate::src::shared::tty::{tty, tty_style_ctx};
use crate::src::shared::window::window;
use crate::src::shared::window::WINDOW_PANE_NO_MODE;
use crate::src::status::{
    status_line_size, status_message_redraw, status_prompt_redraw, status_redraw,
};
use crate::src::style::style_add;
use crate::src::text::utf8::utf8_set;
use crate::src::tty::{
    tty_cell, tty_check_overlay_range, tty_cursor, tty_default_colours, tty_puts, tty_reset,
    tty_sync_start, tty_update_mode, tty_window_offset,
};
use crate::src::tty_draw::tty_draw_line;
use crate::src::tty_term::tty_term_has;
use crate::src::window::windows;
use crate::src::window::{
    window_pane_first, window_pane_get_pane_lines, window_pane_get_pane_status,
    window_pane_is_floating, window_pane_is_visible, window_pane_mode, window_pane_next,
    window_pane_scrollbar_overlay, window_pane_scrollbar_visible, window_pane_z_last,
    window_pane_z_previous, windows_minmax, windows_next,
};
use crate::src::window_border::{
    window_get_border_cell, window_get_fill_cell, window_make_pane_status,
    window_pane_get_border_cell, window_pane_get_border_style,
};
use crate::src::window_copy::window_copy_get_current_offset;
use std::cell::RefCell;

pub const REDRAW_SPAN_MENU: redraw_span_type = 6;
pub const REDRAW_SPAN_SCROLLBAR: redraw_span_type = 5;
pub const REDRAW_SPAN_BORDER: redraw_span_type = 4;
pub const REDRAW_SPAN_STATUS: redraw_span_type = 3;
pub const REDRAW_SPAN_EMPTY: redraw_span_type = 2;
pub const REDRAW_SPAN_OUTSIDE: redraw_span_type = 1;
pub const REDRAW_SPAN_PANE: redraw_span_type = 0;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct redraw_draw_ctx<'scene> {
    pub scene: &'scene redraw_scene,
    pub active: *mut window_pane,
    pub marked: *mut window_pane,
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
    pub c: *mut client,
    pub w: *mut window,
    pub ox: u_int,
    pub oy: u_int,
    pub sx: u_int,
    pub sy: u_int,
    pub ind: ::core::ffi::c_int,
    pub cells: &'a mut [redraw_build_cell],
}

pub const REDRAW_SPAN_TYPES: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
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
pub const REDRAW_MENU: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const REDRAW_OVERLAY: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const REDRAW_ALL: ::core::ffi::c_int = 0x7fffffff as ::core::ffi::c_int;
pub const REDRAW_START_ISOLATE: &std::ffi::CStr = c"\u{2066}";
pub const REDRAW_END_ISOLATE: &std::ffi::CStr = c"\u{2069}";
thread_local! {
    static REDRAW_CELLS: RefCell<Vec<redraw_build_cell>> = RefCell::new(Vec::new());
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
        (REDRAW_MENU, "menu"),
        (REDRAW_OVERLAY, "overlay"),
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
unsafe fn redraw_get_window_offset(c: &mut client) -> tty_window_view {
    let mut view = tty_window_offset(&c.tty);
    view.sx = view.sx.max(c.tty.sx);
    view.sy = view.sy.max(c.tty.sy.wrapping_sub(status_line_size(c)));
    view
}
unsafe fn redraw_set_context(mut c: *mut client, mut bctx: *mut redraw_build_ctx) {
    let mut s: *mut session = (*c).session;
    let mut w: *mut window = (*(*s).curw).window_ptr();
    (*bctx).c = c;
    (*bctx).w = w;
    let view = redraw_get_window_offset(&mut *c);
    (*bctx).ox = view.ox;
    (*bctx).oy = view.oy;
    (*bctx).sx = view.sx;
    (*bctx).sy = view.sy;
    (*bctx).ind = options_get_number(
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
        b"pane-border-indicators\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
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
    let mut w: *mut window = (*bctx).w;
    (*bc).data = redraw_span_data::default();
    if (*bctx).ox.wrapping_add(x) < (*w).sx && (*bctx).oy.wrapping_add(y) < (*w).sy {
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
    if wx as u_int > (*(*bctx).w).sx || wy as u_int > (*(*bctx).w).sy {
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
    return 1 as ::core::ffi::c_int;
}
unsafe fn redraw_pane_to_scene(
    mut bctx: *mut redraw_build_ctx,
    mut wp: *mut window_pane,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
    mut x: *mut u_int,
    mut y: *mut u_int,
) -> ::core::ffi::c_int {
    let mut wx: ::core::ffi::c_int = (*wp).xoff + px;
    let mut wy: ::core::ffi::c_int = (*wp).yoff + py;
    let mut left: ::core::ffi::c_int = 0;
    let mut right: ::core::ffi::c_int = 0;
    let mut top: ::core::ffi::c_int = 0;
    let mut bottom: ::core::ffi::c_int = 0;
    if window_pane_is_floating(wp) != 0 {
        left = (*wp).xoff - 1 as ::core::ffi::c_int;
        right = ((*wp).xoff as u_int).wrapping_add((*wp).sx) as ::core::ffi::c_int;
        top = (*wp).yoff - 1 as ::core::ffi::c_int;
        bottom = ((*wp).yoff as u_int).wrapping_add((*wp).sy) as ::core::ffi::c_int;
        if left < 0 as ::core::ffi::c_int && wx < 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        if right > (*(*bctx).w).sx as ::core::ffi::c_int
            && wx >= (*(*bctx).w).sx as ::core::ffi::c_int
        {
            return 0 as ::core::ffi::c_int;
        }
        if top < 0 as ::core::ffi::c_int && wy < 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        if bottom > (*(*bctx).w).sy as ::core::ffi::c_int
            && wy >= (*(*bctx).w).sy as ::core::ffi::c_int
        {
            return 0 as ::core::ffi::c_int;
        }
    }
    return redraw_window_to_scene(bctx, wx, wy, x, y);
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
    return 12 as ::core::ffi::c_int;
}
unsafe fn redraw_check_two_pane_colours(
    mut w: *mut window,
    mut type_0: *mut layout_type,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut count: u_int = 0 as u_int;
    wp = window_pane_first(w).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !wp.is_null() {
        if !(window_pane_is_floating(wp) != 0 || (*wp).layout_cell.is_null()) {
            count = count.wrapping_add(1);
            if count > 2 as u_int || (*(*wp).layout_cell).parent.is_null() {
                return 0 as ::core::ffi::c_int;
            }
            *type_0 = (*(*(*wp).layout_cell).parent).type_0;
        }
        wp = window_pane_next(wp).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    }
    return (count == 2 as u_int) as ::core::ffi::c_int;
}
unsafe fn redraw_mark_pane_inside(mut bctx: *mut redraw_build_ctx, mut wp: *mut window_pane) {
    let mut bc: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    py = 0 as u_int;
    while py < (*wp).sy {
        px = 0 as u_int;
        while px < (*wp).sx {
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
                (*bc).data.pane_mut().wp = wp;
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
    mut wp: *mut window_pane,
    mut sb_w: ::core::ffi::c_int,
    mut sb_left: ::core::ffi::c_int,
    mut overlay: ::core::ffi::c_int,
) {
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
        sx = (*wp).xoff;
        ex = sx + sb_w - 1 as ::core::ffi::c_int;
    } else if overlay != 0 {
        ex = (*wp).xoff + (*wp).sx as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        sx = ex - sb_w + 1 as ::core::ffi::c_int;
    } else if sb_left != 0 {
        sx = (*wp).xoff - sb_w;
        ex = (*wp).xoff - 1 as ::core::ffi::c_int;
    } else {
        sx = (*wp).xoff + (*wp).sx as ::core::ffi::c_int;
        ex = sx + sb_w - 1 as ::core::ffi::c_int;
    }
    sy = 0 as u_int;
    while sy < (*wp).sy {
        wy = (*wp).yoff + sy as ::core::ffi::c_int;
        wx = sx;
        while wx <= ex {
            if !(redraw_window_to_scene(bctx, wx, wy, &raw mut x, &raw mut y) == 0) {
                bc = redraw_get_build_cell(bctx, x, y);
                (*bc).data = redraw_span_data::Scrollbar(Default::default());
                (*bc).data.scrollbar_mut().wp = wp;
                (*bc).data.scrollbar_mut().y = sy;
                (*bc).data.scrollbar_mut().height = (*wp).sy;
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
unsafe fn redraw_data_has_pane(
    data: &redraw_span_data,
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    if data.border().top_wp == wp {
        return 1 as ::core::ffi::c_int;
    }
    if data.border().bottom_wp == wp {
        return 1 as ::core::ffi::c_int;
    }
    if data.border().left_wp == wp {
        return 1 as ::core::ffi::c_int;
    }
    if data.border().right_wp == wp {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn redraw_mark_border_cell(
    mut bctx: *mut redraw_build_ctx,
    mut wx: ::core::ffi::c_int,
    mut wy: ::core::ffi::c_int,
    mut wp: *mut window_pane,
    mut top_owner: ::core::ffi::c_int,
    mut bottom_owner: ::core::ffi::c_int,
    mut mask: ::core::ffi::c_int,
    mut pane_lines: pane_lines,
    mut floating: ::core::ffi::c_int,
) {
    let mut bc: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut reset: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if redraw_window_to_scene(bctx, wx, wy, &raw mut x, &raw mut y) == 0 {
        return;
    }
    bc = redraw_get_build_cell(bctx, x, y);
    if floating == 0 {
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
    } else if (*bc).data.kind() as ::core::ffi::c_uint
        != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
        || redraw_data_has_pane(&(*bc).data, wp) == 0
    {
        reset = 1 as ::core::ffi::c_int;
    }
    if reset != 0 {
        (*bc).data = redraw_span_data::Border(Default::default());
    }
    if top_owner != 0 {
        (*bc).data.border_mut().top_wp = wp;
        (*bc).data.border_mut().top_lines = pane_lines;
    }
    if bottom_owner != 0 {
        (*bc).data.border_mut().bottom_wp = wp;
        (*bc).data.border_mut().bottom_lines = pane_lines;
    }
    if mask & (REDRAW_BORDER_U | REDRAW_BORDER_D) != 0 {
        if wx < (*wp).xoff {
            (*bc).data.border_mut().right_wp = wp;
            (*bc).data.border_mut().right_lines = pane_lines;
        } else if wx >= (*wp).xoff + (*wp).sx as ::core::ffi::c_int {
            (*bc).data.border_mut().left_wp = wp;
            (*bc).data.border_mut().left_lines = pane_lines;
        }
    }
    mask |= (*bc).data.border().cell_mask;
    (*bc).data.border_mut().cell_mask = mask;
    (*bc).data.border_mut().cell_type = redraw_get_cell_type(mask);
}
unsafe fn redraw_mark_border_status(
    mut bctx: *mut redraw_build_ctx,
    mut wp: *mut window_pane,
    mut right: ::core::ffi::c_int,
    mut top: ::core::ffi::c_int,
    mut bottom: ::core::ffi::c_int,
) {
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
    pane_status = window_pane_get_pane_status(wp);
    if pane_status == PANE_STATUS_OFF {
        return;
    }
    if pane_status == PANE_STATUS_TOP {
        wy = top;
    } else {
        wy = bottom;
    }
    sx = (*wp).xoff + 2 as ::core::ffi::c_int;
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
                (*bc).data.status_mut().wp = wp;
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
    mut wp: *mut window_pane,
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
    wx = (*wp).xoff + 1 as ::core::ffi::c_int;
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
    wy = (*wp).yoff + 1 as ::core::ffi::c_int;
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
    mut wp: *mut window_pane,
    mut sb_w: ::core::ffi::c_int,
    mut sb_left: ::core::ffi::c_int,
) {
    let mut pane_lines: pane_lines = window_pane_get_pane_lines(wp);
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
    let mut floating: ::core::ffi::c_int = window_pane_is_floating(wp);
    if floating != 0
        && pane_lines as ::core::ffi::c_uint
            == PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    pane_status = window_pane_get_pane_status(wp);
    left = (*wp).xoff - 1 as ::core::ffi::c_int;
    right = ((*wp).xoff as u_int).wrapping_add((*wp).sx) as ::core::ffi::c_int;
    if sb_w != 0 as ::core::ffi::c_int {
        if sb_left != 0 {
            left -= sb_w;
        } else {
            right += sb_w;
        }
    }
    top = (*wp).yoff - 1 as ::core::ffi::c_int;
    bottom = ((*wp).yoff as u_int).wrapping_add((*wp).sy) as ::core::ffi::c_int;
    mark_left = (left >= 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    mark_top = (top >= 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    if floating != 0 {
        mark_right = (right < (*(*bctx).w).sx as ::core::ffi::c_int) as ::core::ffi::c_int;
        mark_bottom = (bottom < (*(*bctx).w).sy as ::core::ffi::c_int) as ::core::ffi::c_int;
        if left < 0 as ::core::ffi::c_int {
            left = 0 as ::core::ffi::c_int;
        }
        if right >= (*(*bctx).w).sx as ::core::ffi::c_int {
            right = (*(*bctx).w).sx as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        }
        if top < 0 as ::core::ffi::c_int {
            top = 0 as ::core::ffi::c_int;
        }
        if bottom >= (*(*bctx).w).sy as ::core::ffi::c_int {
            bottom = (*(*bctx).w).sy as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        }
    } else {
        mark_right = (right <= (*(*bctx).w).sx as ::core::ffi::c_int) as ::core::ffi::c_int;
        mark_bottom = (bottom <= (*(*bctx).w).sy as ::core::ffi::c_int) as ::core::ffi::c_int;
        if pane_status == PANE_STATUS_TOP && bottom < (*(*bctx).w).sy as ::core::ffi::c_int {
            mark_bottom = 0 as ::core::ffi::c_int;
        } else if pane_status == PANE_STATUS_BOTTOM {
            mark_top = 0 as ::core::ffi::c_int;
        }
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
                floating,
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
                floating,
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
                floating,
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
                floating,
            );
            wy += 1;
        }
    }
    redraw_mark_border_status(bctx, wp, right, top, bottom);
    redraw_mark_border_arrows(bctx, wp, left, right, top, bottom);
}
unsafe fn redraw_mark_pane(mut bctx: *mut redraw_build_ctx, mut wp: *mut window_pane) {
    let mut sb_w: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut sb_left: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut overlay: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if window_pane_is_visible(wp) == 0 {
        return;
    }
    if window_pane_scrollbar_visible(wp) != 0 {
        overlay = window_pane_scrollbar_overlay(wp);
        if overlay != 0 {
            sb_w = (*wp).scrollbar_style.width + (*wp).scrollbar_style.pad;
            if sb_w > (*wp).sx as ::core::ffi::c_int {
                sb_w = (*wp).scrollbar_style.width;
                if sb_w > (*wp).sx as ::core::ffi::c_int {
                    sb_w = (*wp).sx as ::core::ffi::c_int;
                }
            }
        } else {
            sb_w = (*wp).scrollbar_style.width + (*wp).scrollbar_style.pad;
        }
    }
    if sb_w != 0 as ::core::ffi::c_int && (*(*bctx).w).sb_pos == PANE_SCROLLBARS_LEFT {
        sb_left = 1 as ::core::ffi::c_int;
    }
    redraw_mark_pane_inside(bctx, wp);
    redraw_mark_pane_borders(
        bctx,
        wp,
        if overlay != 0 {
            0 as ::core::ffi::c_int
        } else {
            sb_w
        },
        sb_left,
    );
    redraw_mark_pane_scrollbar(bctx, wp, sb_w, sb_left, overlay);
}
unsafe fn redraw_mark_two_pane_colours(mut bctx: *mut redraw_build_ctx) {
    let mut bc: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut sd: *mut redraw_span_data = ::core::ptr::null_mut::<redraw_span_data>();
    let mut type_0: layout_type = LAYOUT_LEFTRIGHT;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut wx: u_int = 0;
    let mut wy: u_int = 0;
    if (*bctx).ind != PANE_BORDER_COLOUR && (*bctx).ind != PANE_BORDER_BOTH {
        return;
    }
    if redraw_check_two_pane_colours((*bctx).w, &raw mut type_0) == 0 {
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
                wx = (*bctx).ox.wrapping_add(x);
                wy = (*bctx).oy.wrapping_add(y);
                if type_0 as ::core::ffi::c_uint
                    == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
                    && !(*sd).border().left_wp.is_null()
                    && !(*sd).border().right_wp.is_null()
                {
                    if wy <= (*(*bctx).w).sy.wrapping_div(2 as u_int) {
                        (*sd).border_mut().style_wp = (*sd).border().left_wp;
                    } else {
                        (*sd).border_mut().style_wp = (*sd).border().right_wp;
                    }
                } else if type_0 as ::core::ffi::c_uint
                    == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
                    && !(*sd).border().top_wp.is_null()
                    && !(*sd).border().bottom_wp.is_null()
                {
                    if wx <= (*(*bctx).w).sx.wrapping_div(2 as u_int) {
                        (*sd).border_mut().style_wp = (*sd).border().top_wp;
                    } else {
                        (*sd).border_mut().style_wp = (*sd).border().bottom_wp;
                    }
                }
            }
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
}
unsafe fn redraw_mark_menu(bctx: *mut redraw_build_ctx) {
    let Some(owner) = (*(*bctx).w).menu.as_ref().map(|menu| menu.downgrade()) else {
        return;
    };
    let md = owner.try_borrow_mut().expect("live unborrowed menu");
    let observer = owner.clone();
    for py in 0..menu_height(&md) {
        for px in 0..menu_width(&md) {
            let (mut x, mut y) = (0, 0);
            if redraw_window_to_scene(
                bctx,
                menu_x(&md).wrapping_add(px) as ::core::ffi::c_int,
                menu_y(&md).wrapping_add(py) as ::core::ffi::c_int,
                &mut x,
                &mut y,
            ) != 0
            {
                let cell = &mut *redraw_get_build_cell(bctx, x, y);
                cell.data = redraw_span_data::Menu(crate::src::shared::redraw::RedrawMenuSpan {
                    md: observer.clone(),
                    px,
                    py,
                });
            }
        }
    }
}
fn redraw_compare_data(a: &redraw_build_cell, b: &redraw_build_cell) -> bool {
    use redraw_span_data::*;
    match (&a.data, &b.data) {
        (Pane(a), Pane(b)) => a.wp == b.wp && a.py == b.py && a.px.wrapping_add(1) == b.px,
        (Border(a), Border(b)) => a == b && a.flags & REDRAW_BORDER_IS_ARROW == 0,
        (Status(a), Status(b)) => {
            a.wp == b.wp && a.offset.wrapping_add(1) == b.offset && a.cell_type == b.cell_type
        }
        (Scrollbar(a), Scrollbar(b)) => a == b,
        (Menu(a), Menu(b)) => a.md == b.md && a.py == b.py && a.px.wrapping_add(1) == b.px,
        (Outside, Outside) | (Empty, Empty) => true,
        _ => false,
    }
}
unsafe fn redraw_build_cells<'a>(mut bctx: *mut redraw_build_ctx<'a>, cells: &'a mut Vec<redraw_build_cell>) {
    let mut w: *mut window = (*bctx).w;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut ncells: size_t = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*bctx).sx != 0 as u_int
        && (*bctx).sy as ::core::ffi::c_ulong
            > SIZE_MAX.wrapping_div((*bctx).sx as ::core::ffi::c_ulong)
    {
        fatalx(|out| {
            write_cstr(
                out,
                b"redraw_build_cells\0" as *const u8 as *const ::core::ffi::c_char,
            )?;
            out.write_all(b": too many cells")
        });
    }
    ncells = ((*bctx).sx as size_t).wrapping_mul((*bctx).sy as size_t);
    if ncells > cells.len() {
        if cells.try_reserve_exact(ncells - cells.len()).is_err() {
            fatalx(|out| {
                write_cstr(
                    out,
                    b"redraw_build_cells\0" as *const u8 as *const ::core::ffi::c_char,
                )?;
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
    wp = window_pane_z_last(w).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !wp.is_null() {
        redraw_mark_pane(bctx, wp);
        wp = window_pane_z_previous(wp).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    }
    redraw_mark_two_pane_colours(bctx);
    redraw_mark_menu(bctx);
}
unsafe fn redraw_make_scene(mut c: *mut client) -> Option<Box<redraw_scene>> {
    let mut s: *mut session = (*c).session;
    let mut w: *mut window = (*(*s).curw).window_ptr();
    let mut bctx: redraw_build_ctx = redraw_build_ctx {
        c: ::core::ptr::null_mut::<client>(),
        w: ::core::ptr::null_mut::<window>(),
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
    if (*c).flags & CLIENT_SUSPENDED as uint64_t != 0 {
        return None;
    }
    redraw_set_context(c, &raw mut bctx);
    let mut cells = RedrawCellScratch::take();
    log_debug(format_args!(
        "{}: building @{} scene ({}x{} {},{}; generation {})",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        ((*w).id) as u32,
        (bctx.sx) as u32,
        (bctx.sy) as u32,
        (bctx.ox) as u32,
        (bctx.oy) as u32,
        (*w).redraw_scene_generation as ::core::ffi::c_ulonglong
    ));
    redraw_build_cells(&raw mut bctx, &mut cells.0);
    let mut scene = Box::new(redraw_scene {
        c,
        w,
        lines: Box::default(),
        generation: (*w).redraw_scene_generation,
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
            line.spans[type_0 as usize].push(redraw_span {
                x: x0,
                width: x.wrapping_sub(x0),
                data: (*bc).data.clone(),
            });
        }
        y = y.wrapping_add(1);
    }
    log_debug(format_args!(
        "{}: finished building @{} scene",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        ((*w).id) as u32
    ));
    return Some(scene);
}

pub unsafe fn redraw_invalidate_scene(mut w: *mut window) {
    (*w).redraw_scene_generation = (*w).redraw_scene_generation.wrapping_add(1);
}
pub unsafe fn redraw_invalidate_all_scenes() {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    w = windows_minmax(&*std::ptr::addr_of!(windows));
    while !w.is_null() {
        redraw_invalidate_scene(w);
        w = windows_next(&*w);
    }
}
unsafe fn redraw_get_scene(c: *mut client) -> Option<Box<redraw_scene>> {
    let w = (*(*(*c).session).curw).window_ptr();
    let tty_window_view { ox, oy, sx, sy, .. } = redraw_get_window_offset(&mut *c);
    let scene = (*c).redraw_scene.take();
    let reason = match scene.as_deref() {
        None => Some("missing"),
        Some(scene) if scene.w != w => Some("window changed"),
        Some(scene) if scene.generation != (*w).redraw_scene_generation => {
            Some("generation changed")
        }
        Some(scene) if scene.ox != ox || scene.oy != oy => Some("offset changed"),
        Some(scene) if scene.sx != sx || scene.sy != sy => Some("size changed"),
        Some(_) => None,
    };
    if let Some(reason) = reason {
        log_debug(format_args!(
            "{}: @{} scene invalid: {}",
            log_cstr(
                (*c).name
                    .as_ref()
                    .map_or(std::ptr::null(), |name| name.as_ptr())
            ),
            (*w).id,
            reason
        ));
        drop(scene);
        redraw_make_scene(c)
    } else {
        scene
    }
}

// An active draw owns its scene. Nested draws may publish a replacement while
// callbacks run; keep that newer cache instead of overwriting it on return.
fn redraw_restore_scene(c: &mut client, scene: Box<redraw_scene>) {
    if c.redraw_scene.is_none() {
        c.redraw_scene = Some(scene);
    }
}
unsafe fn redraw_draw_pane_span(
    dctx: &mut redraw_draw_ctx<'_>,
    span: &redraw_span,
    mut x: u_int,
    mut y: u_int,
    mut n: u_int,
) {
    let scene = dctx.scene;
    let mut c: *mut client = scene.c;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut wp: *mut window_pane = span.data.pane().wp;
    let mut s: *mut screen = (*wp).screen;
    let mut defaults: grid_cell = grid_cell {
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
    let mut style_ctx: tty_style_ctx = tty_style_ctx {
        defaults: grid_cell::default(),
        palette: ::core::ptr::null_mut::<colour_palette>(),
        dim: 0,
        hyperlinks: None,
    };
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    (defaults, style_ctx.dim) = tty_default_colours(wp);
    style_ctx.defaults = defaults;
    style_ctx.palette = &raw mut (*wp).palette;
    style_ctx.hyperlinks = (*s).hyperlinks.clone();
    px = (*span).data.pane().px.wrapping_add(x.wrapping_sub(span.x));
    py = span.data.pane().py;
    tty_draw_line(tty, &*s, px, py, n, x, y, Some(&style_ctx));
}
unsafe fn redraw_get_default_border_style(
    dctx: &mut redraw_draw_ctx<'_>,
    mut gc: *mut grid_cell,
    mut pane_lines: *mut pane_lines,
) {
    let scene = dctx.scene;
    let mut c: *mut client = scene.c;
    let mut s: *mut session = (*c).session;
    let mut oo: *mut options = options_owner_ptr(&mut (*scene.w).options).map_or(std::ptr::null_mut(), |options| options);
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut dgc: *mut grid_cell = &mut dctx.default_gc;
    if !dctx.flags & REDRAW_DEFAULT_SET != 0 {
        ft = format_create_defaults(
            ::core::ptr::null_mut::<cmdq_item>(),
            c,
            s,
            (*s).curw,
            ::core::ptr::null_mut::<window_pane>(),
        );
        memcpy(
            dgc as *mut ::core::ffi::c_void,
            &raw const grid_default_cell as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
        style_add(
            dgc,
            oo,
            b"pane-border-style\0" as *const u8 as *const ::core::ffi::c_char,
            ft,
        );
        format_free(ft);
        dctx.pane_lines = options_get_number(
            oo,
            b"pane-border-lines\0" as *const u8 as *const ::core::ffi::c_char,
        ) as pane_lines;
        dctx.flags |= REDRAW_DEFAULT_SET;
    }
    memcpy(
        gc as *mut ::core::ffi::c_void,
        dgc as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    *pane_lines = dctx.pane_lines;
}
unsafe fn redraw_get_pane_for_border_style(
    dctx: &mut redraw_draw_ctx<'_>,
    span: &redraw_span,
) -> *mut window_pane {
    let mut active: *mut window_pane = dctx.active;
    if span.data.kind() as ::core::ffi::c_uint
        != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<window_pane>();
    }
    if !span.data.border().style_wp.is_null() {
        return span.data.border().style_wp;
    }
    if !active.is_null() && redraw_data_has_pane(&span.data, active) != 0 {
        return active;
    }
    if !span.data.border().top_wp.is_null() {
        return span.data.border().top_wp;
    }
    if !span.data.border().bottom_wp.is_null() {
        return span.data.border().bottom_wp;
    }
    if !span.data.border().left_wp.is_null() {
        return span.data.border().left_wp;
    }
    if !span.data.border().right_wp.is_null() {
        return span.data.border().right_wp;
    }
    return ::core::ptr::null_mut::<window_pane>();
}
unsafe fn redraw_draw_border_arrow(
    dctx: &mut redraw_draw_ctx<'_>,
    span: &redraw_span,
    mut gc: *mut grid_cell,
) {
    let mut active: *mut window_pane = dctx.active;
    let mut ch: ::core::ffi::c_char = 0;
    if span.data.kind() as ::core::ffi::c_uint
        != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
        || active.is_null()
    {
        return;
    }
    if !span.data.border().flags & REDRAW_BORDER_IS_ARROW != 0 {
        return;
    }
    if span.data.border().left_wp == active {
        ch = ',' as i32 as ::core::ffi::c_char;
    } else if span.data.border().right_wp == active {
        ch = '+' as i32 as ::core::ffi::c_char;
    } else if span.data.border().top_wp == active {
        ch = '-' as i32 as ::core::ffi::c_char;
    } else if span.data.border().bottom_wp == active {
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
    let mut c: *mut client = scene.c;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut w: *mut window = scene.w;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
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
    let mut isolates: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if span.data.kind() as ::core::ffi::c_uint
        != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        cell_type = CELL_NONE as u_int;
    } else {
        wp = redraw_get_pane_for_border_style(dctx, span);
        cell_type = span.data.border().cell_type as u_int;
    }
    if wp.is_null() {
        redraw_get_default_border_style(dctx, &raw mut gc, &raw mut pane_lines);
        if span.data.kind() as ::core::ffi::c_uint
            == REDRAW_SPAN_OUTSIDE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            window_get_fill_cell(w, 0 as ::core::ffi::c_int, &raw mut gc);
        } else if span.data.kind() as ::core::ffi::c_uint
            == REDRAW_SPAN_EMPTY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            window_get_fill_cell(w, 1 as ::core::ffi::c_int, &raw mut gc);
        } else {
            if span.data.kind() as ::core::ffi::c_uint
                != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                pane_lines = PANE_LINES_SINGLE;
            }
            window_get_border_cell(
                ::core::ptr::null_mut::<window_pane>(),
                pane_lines,
                cell_type as ::core::ffi::c_int,
                &mut gc,
            );
        }
    } else {
        window_pane_get_border_style(wp, c, &raw mut gc);
        window_pane_get_border_cell(wp, cell_type as ::core::ffi::c_int, &mut gc);
    }
    if span.data.kind() as ::core::ffi::c_uint
        == REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
        && !dctx.marked.is_null()
        && redraw_data_has_pane(&span.data, dctx.marked) != 0
    {
        gc.attr = (gc.attr as ::core::ffi::c_int ^ GRID_ATTR_REVERSE) as u_short;
    }
    redraw_draw_border_arrow(dctx, span, &raw mut gc);
    if cell_type == CELL_UD as u_int && dctx.flags & REDRAW_ISOLATES != 0 {
        isolates = 1 as ::core::ffi::c_int;
    }
    tty_cursor(tty, x, y);
    if isolates != 0 {
        tty_puts(tty, REDRAW_END_ISOLATE);
    }
    i = 0 as u_int;
    while i < n {
        tty_cell(tty, &gc, None);
        i = i.wrapping_add(1);
    }
    if isolates != 0 {
        tty_puts(tty, REDRAW_START_ISOLATE);
    }
}
unsafe fn redraw_draw_status_span(
    dctx: &mut redraw_draw_ctx<'_>,
    span: &redraw_span,
    mut x: u_int,
    mut y: u_int,
    mut n: u_int,
) {
    let scene = dctx.scene;
    let mut c: *mut client = scene.c;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut wp: *mut window_pane = span.data.status().wp;
    let mut s: *mut screen = &raw mut (*wp).status_screen;
    let mut px: u_int = 0;
    let mut sx: u_int = (*s).grid().sx;
    px = (*span)
        .data
        .status()
        .offset
        .wrapping_add(x.wrapping_sub(span.x));
    if px < sx {
        if n > sx.wrapping_sub(px) {
            n = sx.wrapping_sub(px);
        }
        tty_draw_line(tty, &*s, px, 0 as u_int, n, x, y, None);
    }
}
unsafe fn redraw_draw_scrollbar_span(
    dctx: &mut redraw_draw_ctx<'_>,
    span: &redraw_span,
    mut x: u_int,
    mut y: u_int,
    mut n: u_int,
) {
    let scene = dctx.scene;
    let mut wp: *mut window_pane = span.data.scrollbar().wp;
    let mut s: *mut screen = (*wp).screen;
    let mut tty: *mut tty = &raw mut (*scene.c).tty;
    let mut sb_style: *mut style = &raw mut (*wp).scrollbar_style;
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
    let mut slgc: grid_cell = grid_cell {
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
    let mut pad_gc: grid_cell = grid_cell {
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
    let mut gcp: *mut grid_cell = ::core::ptr::null_mut::<grid_cell>();
    let mut pct_view: ::core::ffi::c_double = 0.;
    let mut total_height: u_int = 0;
    let mut slider_h: u_int = 0;
    let mut slider_y: u_int = 0;
    let mut sb_h: u_int = span.data.scrollbar().height;
    let mut sb_y: u_int = span.data.scrollbar().y;
    let mut i: u_int = 0;
    let mut off: u_int = 0;
    let mut sb_w: u_int = 0;
    let mut sb_pad: u_int = 0;
    let mut cm_y: ::core::ffi::c_int = 0;
    let mut cm_size: ::core::ffi::c_int = 0;
    if window_pane_mode(wp) == WINDOW_PANE_NO_MODE {
        total_height = (*s).grid().sy.wrapping_add((*s).grid().hsize);
        if total_height == 0 as u_int {
            return;
        }
        pct_view = sb_h as ::core::ffi::c_double / total_height as ::core::ffi::c_double;
        slider_h = (sb_h as ::core::ffi::c_double * pct_view) as u_int;
        slider_y = sb_h.wrapping_sub(slider_h);
    } else {
        if (*wp).modes.active.is_null() {
            return;
        }
        if window_copy_get_current_offset(
            wp,
            &raw mut cm_y as *mut u_int,
            &raw mut cm_size as *mut u_int,
        ) == 0 as ::core::ffi::c_int
        {
            return;
        }
        total_height = (cm_size as u_int).wrapping_add(sb_h);
        if total_height == 0 as u_int {
            return;
        }
        pct_view = sb_h as ::core::ffi::c_double / total_height as ::core::ffi::c_double;
        slider_h = (sb_h as ::core::ffi::c_double * pct_view) as u_int;
        slider_y = (sb_h.wrapping_add(1 as u_int) as ::core::ffi::c_double
            * (cm_y as ::core::ffi::c_double / total_height as ::core::ffi::c_double))
            as u_int;
    }
    if slider_h < 1 as u_int {
        slider_h = 1 as u_int;
    }
    if slider_y >= sb_h {
        slider_y = sb_h.wrapping_sub(1 as u_int);
    }
    (*wp).sb_slider_y = slider_y;
    (*wp).sb_slider_h = slider_h;
    gc = (*sb_style).gc;
    memcpy(
        &raw mut slgc as *mut ::core::ffi::c_void,
        &raw mut gc as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    slgc.fg = gc.bg;
    slgc.bg = gc.fg;
    pad_gc = tty_default_colours(wp).0;
    sb_w = (*sb_style).width as u_int;
    sb_pad = (*sb_style).pad as u_int;
    off = x.wrapping_sub(span.x);
    tty_cursor(tty, x, y);
    let mut current_block_40: u64;
    i = 0 as u_int;
    while i < n {
        if span.data.scrollbar().flags & REDRAW_SCROLLBAR_LEFT != 0 {
            if off.wrapping_add(i) >= sb_w && off.wrapping_add(i) < sb_w.wrapping_add(sb_pad) {
                tty_cell(tty, &pad_gc, None);
                current_block_40 = 3437258052017859086;
            } else {
                current_block_40 = 7828949454673616476;
            }
        } else if off.wrapping_add(i) < sb_pad {
            tty_cell(tty, &pad_gc, None);
            current_block_40 = 3437258052017859086;
        } else {
            current_block_40 = 7828949454673616476;
        }
        match current_block_40 {
            7828949454673616476 => {
                if sb_y >= slider_y && sb_y < slider_y.wrapping_add(slider_h) {
                    gcp = &raw mut slgc;
                } else {
                    gcp = &raw mut gc;
                }
                tty_cell(tty, &*gcp, None);
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }
}
unsafe fn redraw_draw_menu_span(
    dctx: &mut redraw_draw_ctx<'_>,
    span: &redraw_span,
    x: u_int,
    y: u_int,
    n: u_int,
) {
    let data = span.data.menu();
    let md = match data.md.try_borrow_mut() {
        Ok(md) => md,
        Err(refbox::BorrowError::Dropped) => return,
        Err(refbox::BorrowError::Borrowed) => panic!("menu already borrowed during redraw"),
    };
    if md.closed {
        return;
    }
    let scene = dctx.scene;
    let tty = &raw mut (*scene.c).tty;
    let px = data.px.wrapping_add(x.wrapping_sub(span.x));
    tty_draw_line(tty, menu_screen(&md), px, data.py, n, x, y, None);
}
unsafe fn redraw_draw_span(dctx: &mut redraw_draw_ctx<'_>, span: &redraw_span, mut y: u_int) {
    let scene = dctx.scene;
    let data = &span.data;
    let mut type_0: redraw_span_type = data.kind();
    let mut c: *mut client = scene.c;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut rr: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    let mut i: u_int = 0;
    let mut x: u_int = 0;
    let mut n: u_int = 0;
    if type_0 as ::core::ffi::c_uint
        == REDRAW_SPAN_STATUS as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*data.status().wp).flags & PANE_NEWSTATUS != 0
    {
        return;
    }
    r = tty_check_overlay_range(tty, span.x, y, span.width);
    i = 0 as u_int;
    while i < (*r).used {
        rr = &raw mut (&mut (*r).storage)[i as usize];
        if !((*rr).nx == 0 as u_int) {
            x = (*rr).px;
            n = (*rr).nx;
            match span.data.kind() as ::core::ffi::c_uint {
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
                6 => {
                    redraw_draw_menu_span(dctx, span, x, y, n);
                }
                _ => {}
            }
        }
        i = i.wrapping_add(1);
    }
}
unsafe fn redraw_draw_pane_lines(
    dctx: &mut redraw_draw_ctx<'_>,
    wp: *mut window_pane,
    flags: ::core::ffi::c_int,
) {
    let scene = dctx.scene;
    let top = ((*wp).yoff - scene.oy as i32).max(0);
    let bottom = ((*wp).yoff + (*wp).sy as i32 - scene.oy as i32)
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
            for span in line.spans[REDRAW_SPAN_PANE as usize].iter() {
                if span.data.pane().wp == wp {
                    redraw_draw_span(dctx, span, cy);
                }
            }
        }
        if flags & REDRAW_PANE_SCROLLBAR != 0 {
            for span in line.spans[REDRAW_SPAN_SCROLLBAR as usize].iter() {
                if span.data.scrollbar().wp == wp {
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
        REDRAW_MENU,
    ];
    for (y, line) in scene.lines.iter().enumerate() {
        let cy = if dctx.flags & REDRAW_STATUS_TOP != 0 {
            dctx.status_lines.wrapping_add(y as u_int)
        } else {
            y as u_int
        };
        for (spans, mask) in line.spans.iter().zip(masks) {
            if flags != REDRAW_ALL && flags & mask == 0 {
                continue;
            }
            for span in spans.iter() {
                redraw_draw_span(dctx, span, cy);
            }
        }
    }
}
unsafe fn redraw_draw_menu_lines(dctx: &mut redraw_draw_ctx<'_>) {
    let scene = dctx.scene;
    for (y, line) in scene.lines.iter().enumerate() {
        let cy = if dctx.flags & REDRAW_STATUS_TOP != 0 {
            dctx.status_lines.wrapping_add(y as u_int)
        } else {
            y as u_int
        };
        for span in line.spans[REDRAW_SPAN_MENU as usize].iter() {
            redraw_draw_span(dctx, span, cy);
        }
    }
}
unsafe fn redraw_pane_status_line(
    dctx: &mut redraw_draw_ctx<'_>,
    mut wp: *mut window_pane,
    mut line: *mut u_int,
) -> ::core::ffi::c_int {
    let scene = dctx.scene;
    let mut pane_status: ::core::ffi::c_int = 0;
    let mut wy: ::core::ffi::c_int = 0;
    pane_status = window_pane_get_pane_status(wp);
    if pane_status == PANE_STATUS_OFF {
        return 0 as ::core::ffi::c_int;
    }
    if pane_status == PANE_STATUS_TOP {
        wy = (*wp).yoff - 1 as ::core::ffi::c_int;
    } else {
        wy = ((*wp).yoff as u_int).wrapping_add((*wp).sy) as ::core::ffi::c_int;
    }
    if wy < 0 as ::core::ffi::c_int || wy < scene.oy as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if wy as u_int >= scene.oy.wrapping_add(scene.sy) {
        return 0 as ::core::ffi::c_int;
    }
    *line = (wy as u_int).wrapping_sub(scene.oy);
    return 1 as ::core::ffi::c_int;
}
unsafe fn redraw_pane_status_width<'scene>(
    dctx: &mut redraw_draw_ctx<'scene>,
    wp: *mut window_pane,
) -> Option<(u_int, &'scene redraw_spans, usize)> {
    let mut y = 0;
    if redraw_pane_status_line(dctx, wp, &mut y) == 0 {
        return None;
    }
    let spans = &dctx.scene.lines[y as usize].spans[REDRAW_SPAN_STATUS as usize];
    let mut width = 0;
    let mut first_index = spans.entries.len();
    for (index, span) in spans.iter().enumerate() {
        if span.data.status().wp == wp {
            if first_index == spans.entries.len() {
                first_index = index;
            }
            width = width.max(span.data.status().offset.wrapping_add(span.width));
        }
    }
    Some((width, spans, first_index))
}
unsafe fn redraw_set_draw_context(scene: &redraw_scene) -> redraw_draw_ctx<'_> {
    let c = scene.c;
    let s = (*c).session;
    let mut dctx = redraw_draw_ctx {
        scene,
        active: (*(*(*s).curw).window_ptr()).active,
        marked: std::ptr::null_mut(),
        status_lines: status_line_size(&*c),
        pane_lines: PANE_LINES_SINGLE,
        default_gc: grid_cell::default(),
        flags: 0,
    };
    if server_is_marked(s, (*s).curw, marked_pane.wp) != 0 {
        dctx.marked = marked_pane.wp;
    }
    if options_get_number(options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options), c"status-position".as_ptr()) == 0 {
        dctx.flags |= REDRAW_STATUS_TOP;
    }
    if (*c).flags & CLIENT_UTF8 as uint64_t != 0 && tty_term_has(tty_term_owner_ptr(&(*c).tty.term).map_or(std::ptr::null(), |term| term), TTYC_BIDI) != 0 {
        dctx.flags |= REDRAW_ISOLATES;
    }
    dctx
}
unsafe fn redraw_draw_pane_prompt(dctx: &mut redraw_draw_ctx<'_>, mut wp: *mut window_pane) {
    let scene = dctx.scene;
    let mut c: *mut client = scene.c;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut screen: screen = screen::empty();
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut ox: ::core::ffi::c_int = scene.ox as ::core::ffi::c_int;
    let mut oy: ::core::ffi::c_int = scene.oy as ::core::ffi::c_int;
    let mut sx: ::core::ffi::c_int = scene.sx as ::core::ffi::c_int;
    let mut sy: ::core::ffi::c_int = scene.sy as ::core::ffi::c_int;
    let mut line: ::core::ffi::c_int = 0;
    let mut cy: ::core::ffi::c_int = 0;
    let mut px: ::core::ffi::c_int = 0;
    let mut offset: ::core::ffi::c_int = 0;
    let mut width: ::core::ffi::c_int = 0;
    let mut wy: ::core::ffi::c_int = 0;
    if (*wp).prompt.is_none() || (*wp).sx == 0 as u_int || (*wp).sy == 0 as u_int {
        return;
    }
    if !dctx.flags & REDRAW_STATUS_TOP != 0 {
        wy = (*wp).yoff + (*wp).sy as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    } else {
        wy = (*wp).yoff;
    }
    if wy < oy || wy >= oy + sy {
        return;
    }
    line = wy - oy;
    if dctx.flags & REDRAW_STATUS_TOP != 0 {
        cy = dctx.status_lines.wrapping_add(line as u_int) as ::core::ffi::c_int;
    } else {
        cy = line;
    }
    if (*wp).xoff + (*wp).sx as ::core::ffi::c_int <= ox || (*wp).xoff >= ox + sx {
        return;
    }
    if (*wp).xoff < ox {
        offset = ox - (*wp).xoff;
        px = 0 as ::core::ffi::c_int;
    } else {
        offset = 0 as ::core::ffi::c_int;
        px = (*wp).xoff - ox;
    }
    width = (*wp).sx.wrapping_sub(offset as u_int) as ::core::ffi::c_int;
    if px + width > sx {
        width = sx - px;
    }
    screen_init(&mut screen, (*wp).sx, 1 as u_int, 0 as u_int);
    screen_write_start(&mut ctx, &raw mut screen);
    let pdd = prompt_draw_data {
        area_x: 0 as u_int,
        area_width: (*wp).sx,
        prompt_line: 0 as u_int,
    };
    (*wp).prompt_cx = prompt_draw(
        &(*wp).prompt.as_ref().expect("active prompt").try_borrow_mut().expect("unborrowed prompt"),
        &mut ctx,
        pdd,
    );
    screen_write_stop(&mut ctx);
    tty_draw_line(
        tty,
        &screen,
        0 as u_int,
        offset as u_int,
        width as u_int,
        px as u_int,
        cy as u_int,
        None,
    );
    screen_free(&mut screen);
}
unsafe fn redraw_draw(mut c: *mut client, mut wp: *mut window_pane, mut flags: ::core::ffi::c_int) {
    let s = (*c).session;
    let w = (*(*s).curw).window_ptr();
    let mut redraw = 0;
    if (*c).flags & CLIENT_SUSPENDED as uint64_t != 0 {
        return;
    }
    if flags & REDRAW_STATUS != 0 {
        if !(*c).message_string.is_none() {
            redraw = status_message_redraw(c);
        } else if (*c).prompt.is_some() {
            redraw = status_prompt_redraw(c);
        } else {
            redraw = status_redraw(c);
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
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            ),
            ((*w).id) as u32,
            log_cstr(redraw_flags_to_string(flags).as_ptr())
        ));
    }
    let Some(scene) = redraw_get_scene(c) else {
        return;
    };
    redraw_draw_scene(c, wp, flags, &scene);
    redraw_restore_scene(&mut *c, scene);
}

unsafe fn redraw_draw_scene(
    c: *mut client,
    wp: *mut window_pane,
    mut flags: ::core::ffi::c_int,
    scene: &redraw_scene,
) {
    let mut s: *mut session = (*c).session;
    let mut w: *mut window = (*(*s).curw).window_ptr();
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut sl: *mut screen = ::core::ptr::null_mut::<screen>();
    let mut loop_0: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut i: u_int = 0;
    let mut y: u_int = 0;
    let mut lines: u_int = 0;
    let mut j: u_int = 0;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut rr: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    let mut redraw: ::core::ffi::c_int = 0;
    let mut dctx = redraw_set_draw_context(scene);
    if let Some(menu) = (*w).menu.as_ref().map(|menu| menu.downgrade()) {
        menu_update(&mut menu.try_borrow_mut().expect("live unborrowed menu"));
    }
    if flags & (REDRAW_PANE_BORDER | REDRAW_PANE_STATUS) != 0 {
        loop_0 = window_pane_first(scene.w).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        while !loop_0.is_null() {
            (*loop_0).border_gc_set = 0 as ::core::ffi::c_int;
            (*loop_0).active_border_gc_set = 0 as ::core::ffi::c_int;
            loop_0 = window_pane_next(loop_0).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        }
    }
    if flags & REDRAW_PANE_STATUS != 0 {
        redraw = 0 as ::core::ffi::c_int;
        loop_0 = window_pane_first(scene.w).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        while !loop_0.is_null() {
            if flags == REDRAW_ALL {
                (*loop_0).flags |= PANE_NEWSTATUS;
            } else {
                (*loop_0).flags &= !PANE_NEWSTATUS;
            }
            if let Some((width, status_spans, first_status_span)) =
                redraw_pane_status_width(&mut dctx, loop_0)
            {
                if width != 0
                    && window_make_pane_status(loop_0, c, width, status_spans, first_status_span)
                        != 0
                {
                    (*loop_0).flags |= PANE_NEWSTATUS;
                    redraw = 1 as ::core::ffi::c_int;
                }
            }
            loop_0 = window_pane_next(loop_0).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        }
        if redraw == 0 && !(flags == REDRAW_ALL) {
            flags &= !REDRAW_PANE_STATUS;
            if flags == 0 as ::core::ffi::c_int {
                return;
            }
        }
    }
    if flags & REDRAW_PANE != 0 {
        if !wp.is_null() {
            if (*wp).base.mode & MODE_SYNC != 0 {
                screen_write_stop_sync(wp);
            }
            screen_write_clear_dirty(wp);
        } else {
            loop_0 = window_pane_first(scene.w).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
            while !loop_0.is_null() {
                if !(window_pane_is_visible(loop_0) == 0) {
                    if (*loop_0).base.mode & MODE_SYNC != 0 {
                        screen_write_stop_sync(loop_0);
                    }
                    screen_write_clear_dirty(loop_0);
                }
                loop_0 = window_pane_next(loop_0).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
            }
        }
    }
    tty_sync_start(tty);
    tty_update_mode(tty, (*tty).mode & !CURSOR_MODES, None);
    if !wp.is_null() {
        redraw_draw_pane_lines(&mut dctx, wp, flags);
    } else {
        redraw_draw_lines(&mut dctx, flags);
    }
    if flags & REDRAW_PANE != 0 {
        if !wp.is_null() {
            redraw_draw_pane_prompt(&mut dctx, wp);
        } else {
            loop_0 = window_pane_first(scene.w).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
            while !loop_0.is_null() {
                if window_pane_is_visible(loop_0) != 0 {
                    redraw_draw_pane_prompt(&mut dctx, loop_0);
                }
                loop_0 = window_pane_next(loop_0).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
            }
        }
    }
    if (*w).menu.is_some() && flags & REDRAW_MENU != 0 {
        redraw_draw_menu_lines(&mut dctx);
    }
    if flags & REDRAW_STATUS != 0 {
        lines = dctx.status_lines;
        if !(*c).message_string.is_none() || (*c).prompt.is_some() {
            lines = if lines == 0 as u_int {
                1 as u_int
            } else {
                lines
            };
        }
        if dctx.flags & REDRAW_STATUS_TOP != 0 {
            y = 0 as u_int;
        } else {
            y = (*c).tty.sy.wrapping_sub(lines);
        }
        sl = (*c).status.active_screen();
        i = 0 as u_int;
        while i < lines {
            r = tty_check_overlay_range(tty, 0 as u_int, y.wrapping_add(i), (*tty).sx);
            j = 0 as u_int;
            while j < (*r).used {
                rr = &raw mut (&mut (*r).storage)[j as usize];
                if !((*rr).nx == 0 as u_int) {
                    tty_draw_line(
                        tty,
                        &*sl,
                        (*rr).px,
                        i,
                        (*rr).nx,
                        (*rr).px,
                        y.wrapping_add(i),
                        None,
                    );
                }
                j = j.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
    }
    if flags & REDRAW_OVERLAY != 0 {
        server_client_overlay_draw(c);
    }
    tty_reset(tty);
    log_debug(format_args!(
        "{}: finished @{} redraw",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        ((*scene.w).id) as u32
    ));
}
pub fn redraw_get_status_border_cell_type(
    spans: &redraw_spans,
    span_index: &mut usize,
    mut x: u_int,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut start: u_int = 0;
    let mut end: u_int = 0;
    let entries = &spans.entries;
    let mut index = *span_index;
    if index >= entries.len()
        || entries[index].data.kind() as ::core::ffi::c_uint
            != REDRAW_SPAN_STATUS as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 2 as ::core::ffi::c_int;
    }
    wp = entries[index].data.status().wp;
    while index < entries.len() {
        let span = entries[index].as_ref();
        if !(span.data.kind() as ::core::ffi::c_uint
            != REDRAW_SPAN_STATUS as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            if span.data.status().wp == wp {
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
        }
        index += 1;
    }
    if index == entries.len() {
        *span_index = entries.len();
    }
    return 2 as ::core::ffi::c_int;
}
pub unsafe fn redraw_screen(mut c: *mut client) {
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*c).flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
        if (*c).flags & CLIENT_REDRAWOVERLAY as uint64_t != 0 {
            redraw_draw(c, ::core::ptr::null_mut::<window_pane>(), REDRAW_ALL);
        } else {
            redraw_draw(
                c,
                ::core::ptr::null_mut::<window_pane>(),
                REDRAW_ALL & !REDRAW_OVERLAY,
            );
        }
    } else {
        if (*c).flags & CLIENT_REDRAWBORDERS as uint64_t != 0 {
            flags |= REDRAW_PANE_BORDER | REDRAW_PANE_STATUS;
        }
        if (*c).flags & CLIENT_REDRAWSTATUS as uint64_t != 0 {
            flags |= REDRAW_STATUS | REDRAW_PANE_STATUS;
        }
        if (*c).flags & CLIENT_REDRAWOVERLAY as uint64_t != 0 {
            flags |= REDRAW_OVERLAY;
        }
        if (*c).flags & CLIENT_REDRAWMENU as uint64_t != 0 {
            flags |= REDRAW_MENU;
        }
        if (*(*(*(*c).session).curw).window_ptr()).menu.is_some() {
            flags |= REDRAW_MENU;
        }
        if flags != 0 as ::core::ffi::c_int {
            redraw_draw(c, ::core::ptr::null_mut::<window_pane>(), flags);
        }
    };
}
pub unsafe fn redraw_pane(mut c: *mut client, mut wp: *mut window_pane) {
    redraw_draw(c, wp, REDRAW_PANE | REDRAW_PANE_SCROLLBAR);
    if (*(*(*(*c).session).curw).window_ptr()).menu.is_some() {
        redraw_draw(c, ::core::ptr::null_mut::<window_pane>(), REDRAW_MENU);
    }
}
pub unsafe fn redraw_pane_scrollbar(mut c: *mut client, mut wp: *mut window_pane) {
    redraw_draw(c, wp, REDRAW_PANE_SCROLLBAR);
}

#[cfg(test)]
mod menu_observer_tests {
    use super::*;
    use crate::src::shared::menu::MenuOwner;
    use crate::src::shared::menu::{menu, menu_data};
    use crate::src::shared::redraw::RedrawMenuSpan;

    fn owned_scene(
        menu: &crate::src::shared::menu::MenuOwner,
        generation: u64,
    ) -> Box<redraw_scene> {
        let mut line = redraw_line::default();
        line.spans[REDRAW_SPAN_MENU as usize].push(redraw_span {
            x: 0,
            width: 1,
            data: redraw_span_data::Menu(RedrawMenuSpan {
                md: menu.downgrade(),
                px: 0,
                py: 0,
            }),
        });
        Box::new(redraw_scene {
            c: std::ptr::null_mut(),
            w: std::ptr::null_mut(),
            lines: vec![line].into_boxed_slice(),
            generation,
            sx: 1,
            sy: 1,
            ox: 0,
            oy: 0,
        })
    }

    #[test]
    fn active_scene_borrows_survive_nested_cache_replacement() {
        let menu = MenuOwner::new(menu_data::new(Box::new(menu {
            title: c"Scene".to_owned(),
            items: Vec::new(),
            count: 0,
            width: 1,
        })));
        let mut client = client::empty();
        client.redraw_scene = Some(owned_scene(&menu, 1));
        let active = client.redraw_scene.take().unwrap();
        let original_address = active.as_ref() as *const redraw_scene as usize;
        let span = active.lines[0].spans[REDRAW_SPAN_MENU as usize]
            .iter()
            .next()
            .unwrap();
        client.redraw_scene = Some(owned_scene(&menu, 2));
        assert_eq!(menu.weak_count(), 2);
        assert_eq!(active.generation, 1);
        assert!(span.data.menu().md.is_alive());
        redraw_restore_scene(&mut client, active);
        assert_eq!(menu.weak_count(), 1);
        assert_eq!(client.redraw_scene.as_ref().unwrap().generation, 2);
        assert_ne!(
            client.redraw_scene.as_deref().unwrap() as *const redraw_scene as usize,
            original_address
        );
        // Client teardown owns all remaining rows, spans and their observers.
        drop(client);
        assert_eq!(menu.weak_count(), 0);
    }

    #[test]
    fn returning_an_active_scene_reuses_its_box_and_spans() {
        let menu = MenuOwner::new(menu_data::new(Box::new(menu {
            title: c"Scene".to_owned(),
            items: Vec::new(),
            count: 0,
            width: 1,
        })));
        let mut client = client::empty();
        let scene = owned_scene(&menu, 7);
        let scene_address = scene.as_ref() as *const redraw_scene as usize;
        let span_address = scene.lines[0].spans[REDRAW_SPAN_MENU as usize].entries[0].as_ref()
            as *const redraw_span as usize;
        redraw_restore_scene(&mut client, scene);
        let active = client.redraw_scene.take().unwrap();
        assert_eq!(
            active.as_ref() as *const redraw_scene as usize,
            scene_address
        );
        assert_eq!(
            active.lines[0].spans[REDRAW_SPAN_MENU as usize].entries[0].as_ref() as *const redraw_span
                as usize,
            span_address
        );
        redraw_restore_scene(&mut client, active);
        assert_eq!(menu.weak_count(), 1);
        drop(client.redraw_scene.take());
        assert_eq!(menu.weak_count(), 0);
    }

    #[test]
    fn cached_spans_observe_menu_lifetimes_and_scratch_releases_observers() {
        let owner = MenuOwner::new(menu_data::new(Box::new(menu {
            title: c"Observed".to_owned(),
            items: Vec::new(),
            count: 0,
            width: 10,
        })));
        let mut cell = redraw_build_cell {
            data: redraw_span_data::Menu(RedrawMenuSpan {
                md: owner.downgrade(),
                px: 0,
                py: 0,
            }),
        };
        let mut next = cell.clone();
        next.data.menu_mut().px = 1;
        assert!(redraw_compare_data(&cell, &next));
        let mut span = redraw_span {
            x: 0,
            width: 2,
            data: cell.data.clone(),
        };
        assert!(cell.data.menu().md.is(&owner));
        let weak_count = owner.weak_count();
        {
            let mut scratch = RedrawCellScratch::take();
            scratch.0.extend([cell.clone(), next.clone()]);
            assert_eq!(owner.weak_count(), weak_count + 2);
        }
        assert_eq!(owner.weak_count(), weak_count);
        REDRAW_CELLS.with(|cache| assert!(cache.borrow().is_empty()));
        let scene = redraw_scene {
            c: std::ptr::null_mut(),
            w: std::ptr::null_mut(),
            lines: Box::default(),
            generation: 0,
            sx: 0,
            sy: 0,
            ox: 0,
            oy: 0,
        };
        let mut dctx = redraw_draw_ctx {
            scene: &scene,
            active: std::ptr::null_mut(),
            marked: std::ptr::null_mut(),
            status_lines: 0,
            pane_lines: PANE_LINES_SINGLE,
            default_gc: grid_cell::default(),
            flags: 0,
        };
        owner.try_borrow_mut().unwrap().closed = true;
        // Closed and expired spans return before accessing the drawing context.
        unsafe {
            redraw_draw_menu_span(&mut dctx, &span, 0, 0, 2);
        }
        drop(owner);
        assert!(!span.data.menu().md.is_alive());
        unsafe {
            redraw_draw_menu_span(&mut dctx, &span, 0, 0, 2);
        }
        assert!(redraw_compare_data(&cell, &next));
        cell.data = redraw_span_data::Empty;
        assert!(!redraw_compare_data(&cell, &next));
    }
}
