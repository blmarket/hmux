use crate::src::ffi::libc::{free, memcpy, memset, strlcat, strlen};
use crate::src::format::{format_create_defaults, format_free};
use crate::src::grid::grid_default_cell;
use crate::src::log::{fatalx, log_debug, log_get_level};
use crate::src::menu::{menu_height, menu_screen, menu_update, menu_width, menu_x, menu_y};
use crate::src::options::options_get_number;
use crate::src::prompt::prompt_draw;
use crate::src::screen::{screen_free, screen_init};
use crate::src::screen_write::{
    screen_write_clear_dirty, screen_write_start, screen_write_stop, screen_write_stop_sync,
};
use crate::src::server::{marked_pane, server_is_marked};
use crate::src::shared::abi::*;
pub use crate::src::shared::arguments::args;
pub use crate::src::shared::borders::{
    CELL_LD, CELL_LR, CELL_LRD, CELL_LRU, CELL_LRUD, CELL_LU, CELL_NONE, CELL_RD, CELL_RU, CELL_UD,
    CELL_ULD, CELL_URD,
};
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::client::{
    CLIENT_REDRAWBORDERS, CLIENT_REDRAWMENU, CLIENT_REDRAWOVERLAY, CLIENT_REDRAWSTATUS,
    CLIENT_REDRAWWINDOW, CLIENT_SUSPENDED, CLIENT_UTF8,
};
use crate::src::shared::colour::*;
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
pub use crate::src::shared::format::{format_job_tree, format_tree};
use crate::src::shared::grid::*;
pub use crate::src::shared::hyperlinks::hyperlinks;
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
use crate::src::shared::key::*;
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::limits::SIZE_MAX;
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::options;
pub use crate::src::shared::pane::window_panes_zindex;
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
    PANE_BORDER_ARROWS, PANE_BORDER_BOTH, PANE_BORDER_COLOUR, PANE_NEWSTATUS, PANE_SCROLLBARS_LEFT,
    PANE_STATUS_BOTTOM, PANE_STATUS_OFF, PANE_STATUS_TOP,
};
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::prompt::prompt_draw_data;
pub use crate::src::shared::redraw::{
    redraw_line, redraw_scene, redraw_span, redraw_span_data, redraw_span_data_c2rust_unnamed,
    redraw_span_data_c2rust_unnamed_b, redraw_span_data_c2rust_unnamed_m,
    redraw_span_data_c2rust_unnamed_p, redraw_span_data_c2rust_unnamed_sb,
    redraw_span_data_c2rust_unnamed_st, redraw_span_entry, redraw_span_type, redraw_spans,
};
pub use crate::src::shared::screen::{
    screen, screen_sel, screen_titles, CURSOR_MODES, MODE_CURSOR, MODE_CURSOR_BLINKING,
    MODE_CURSOR_VERY_VISIBLE, MODE_SYNC,
};
pub use crate::src::shared::screen_write::{screen_write_citem, screen_write_cline};
pub use crate::src::shared::screen_write::{screen_write_ctx, screen_write_init_ctx_cb};
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::tty::*;
pub use crate::src::shared::tty::{
    tty, tty_code, tty_ctx, tty_ctx_c2rust_unnamed, tty_ctx_c2rust_unnamed_data,
    tty_ctx_c2rust_unnamed_sel, tty_ctx_redraw_cb, tty_ctx_set_client_cb, tty_key, tty_style_ctx,
    tty_term, tty_term_entry,
};
pub use crate::src::shared::window::WINDOW_PANE_NO_MODE;
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::status::{
    status_line_size, status_message_redraw, status_prompt_redraw, status_redraw,
};
use crate::src::style::style_add;
use crate::src::tty::{
    tty_cell, tty_check_overlay_range, tty_cursor, tty_default_colours, tty_puts, tty_reset,
    tty_sync_start, tty_update_mode, tty_window_offset,
};
use crate::src::tty_draw::tty_draw_line;
use crate::src::tty_term::tty_term_has;
use crate::src::utf8::utf8_set;
pub use crate::src::window::windows;
use crate::src::window::{
    window_pane_get_pane_lines, window_pane_get_pane_status, window_pane_is_floating,
    window_pane_is_visible, window_pane_mode, window_pane_scrollbar_overlay,
    window_pane_scrollbar_visible, windows_minmax, windows_next,
};
use crate::src::window_border::{
    window_get_border_cell, window_get_fill_cell, window_make_pane_status,
    window_pane_get_border_cell, window_pane_get_border_style,
};
use crate::src::window_copy::window_copy_get_current_offset;
use crate::src::xmalloc::{xcalloc, xreallocarray};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

pub const REDRAW_SPAN_MENU: redraw_span_type = 6;
pub const REDRAW_SPAN_SCROLLBAR: redraw_span_type = 5;
pub const REDRAW_SPAN_BORDER: redraw_span_type = 4;
pub const REDRAW_SPAN_STATUS: redraw_span_type = 3;
pub const REDRAW_SPAN_EMPTY: redraw_span_type = 2;
pub const REDRAW_SPAN_OUTSIDE: redraw_span_type = 1;
pub const REDRAW_SPAN_PANE: redraw_span_type = 0;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct redraw_draw_ctx {
    pub scene: *mut redraw_scene,
    pub active: *mut window_pane,
    pub marked: *mut window_pane,
    pub status_lines: u_int,
    pub pane_lines: pane_lines,
    pub default_gc: grid_cell,
    pub flags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct redraw_build_cell {
    pub data: redraw_span_data,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct redraw_build_ctx {
    pub c: *mut client,
    pub w: *mut window,
    pub ox: u_int,
    pub oy: u_int,
    pub sx: u_int,
    pub sy: u_int,
    pub ind: ::core::ffi::c_int,
    pub cells: *mut redraw_build_cell,
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
pub const REDRAW_START_ISOLATE: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"\xE2\x81\xA6\0") };
pub const REDRAW_END_ISOLATE: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"\xE2\x81\xA9\0") };
static mut redraw_cells: *mut redraw_build_cell =
    ::core::ptr::null::<redraw_build_cell>() as *mut redraw_build_cell;
static mut redraw_ncells: size_t = 0;
pub const REDRAW_ISOLATES: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const REDRAW_DEFAULT_SET: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const REDRAW_STATUS_TOP: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
unsafe extern "C" fn redraw_flags_to_string(
    mut flags: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    static mut s: [::core::ffi::c_char; 128] = [0; 128];
    *(&raw mut s as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    if flags & REDRAW_STATUS != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"status \0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & REDRAW_PANE != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"pane \0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & REDRAW_PANE_BORDER != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"border \0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & REDRAW_PANE_STATUS != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"pane-status \0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & REDRAW_PANE_SCROLLBAR != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"scrollbar \0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & REDRAW_MENU != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"menu \0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & REDRAW_OVERLAY != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"overlay \0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags == REDRAW_ALL {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"all \0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if *(&raw mut s as *mut ::core::ffi::c_char) as ::core::ffi::c_int != '\0' as i32 {
        s[strlen(&raw mut s as *mut ::core::ffi::c_char).wrapping_sub(1 as size_t) as usize] =
            '\0' as i32 as ::core::ffi::c_char;
    }
    return &raw mut s as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn redraw_get_window_offset(
    mut c: *mut client,
    mut ox: *mut u_int,
    mut oy: *mut u_int,
    mut sx: *mut u_int,
    mut sy: *mut u_int,
) {
    let mut tty_sx: u_int = 0;
    let mut tty_sy: u_int = 0;
    tty_window_offset(&raw mut (*c).tty, ox, oy, sx, sy);
    tty_sx = (*c).tty.sx;
    tty_sy = (*c).tty.sy.wrapping_sub(status_line_size(c));
    if *sx < tty_sx {
        *sx = tty_sx;
    }
    if *sy < tty_sy {
        *sy = tty_sy;
    }
}
unsafe extern "C" fn redraw_set_context(mut c: *mut client, mut bctx: *mut redraw_build_ctx) {
    let mut s: *mut session = (*c).session;
    let mut w: *mut window = (*(*s).curw).window;
    memset(
        bctx as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<redraw_build_ctx>() as size_t,
    );
    (*bctx).c = c;
    (*bctx).w = w;
    redraw_get_window_offset(
        c,
        &raw mut (*bctx).ox,
        &raw mut (*bctx).oy,
        &raw mut (*bctx).sx,
        &raw mut (*bctx).sy,
    );
    (*bctx).ind = options_get_number(
        (*w).options,
        b"pane-border-indicators\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
}
unsafe extern "C" fn redraw_get_build_cell(
    mut bctx: *mut redraw_build_ctx,
    mut x: u_int,
    mut y: u_int,
) -> *mut redraw_build_cell {
    return (*bctx)
        .cells
        .offset(y.wrapping_mul((*bctx).sx).wrapping_add(x) as isize)
        as *mut redraw_build_cell;
}
unsafe extern "C" fn redraw_reset_cell(
    mut bctx: *mut redraw_build_ctx,
    mut x: u_int,
    mut y: u_int,
) {
    let mut bc: *mut redraw_build_cell = redraw_get_build_cell(bctx, x, y);
    let mut w: *mut window = (*bctx).w;
    memset(
        bc as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<redraw_build_cell>() as size_t,
    );
    if (*bctx).ox.wrapping_add(x) < (*w).sx && (*bctx).oy.wrapping_add(y) < (*w).sy {
        (*bc).data.type_0 = REDRAW_SPAN_EMPTY;
    } else {
        (*bc).data.type_0 = REDRAW_SPAN_OUTSIDE;
    };
}
unsafe extern "C" fn redraw_window_to_scene(
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
unsafe extern "C" fn redraw_pane_to_scene(
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
unsafe extern "C" fn redraw_get_cell_type(mut mask: ::core::ffi::c_int) -> ::core::ffi::c_int {
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
unsafe extern "C" fn redraw_check_two_pane_colours(
    mut w: *mut window,
    mut type_0: *mut layout_type,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut count: u_int = 0 as u_int;
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if !(window_pane_is_floating(wp) != 0 || (*wp).layout_cell.is_null()) {
            count = count.wrapping_add(1);
            if count > 2 as u_int || (*(*wp).layout_cell).parent.is_null() {
                return 0 as ::core::ffi::c_int;
            }
            *type_0 = (*(*(*wp).layout_cell).parent).type_0;
        }
        wp = (*wp).entry.tqe_next;
    }
    return (count == 2 as u_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn redraw_mark_pane_inside(
    mut bctx: *mut redraw_build_ctx,
    mut wp: *mut window_pane,
) {
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
                memset(
                    bc as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    ::core::mem::size_of::<redraw_build_cell>() as size_t,
                );
                (*bc).data.type_0 = REDRAW_SPAN_PANE;
                (*bc).data.c2rust_unnamed.p.wp = wp;
                (*bc).data.c2rust_unnamed.p.px = px;
                (*bc).data.c2rust_unnamed.p.py = py;
            }
            px = px.wrapping_add(1);
        }
        py = py.wrapping_add(1);
    }
}
unsafe extern "C" fn redraw_mark_pane_scrollbar(
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
                memset(
                    bc as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    ::core::mem::size_of::<redraw_build_cell>() as size_t,
                );
                (*bc).data.type_0 = REDRAW_SPAN_SCROLLBAR;
                (*bc).data.c2rust_unnamed.sb.wp = wp;
                (*bc).data.c2rust_unnamed.sb.y = sy;
                (*bc).data.c2rust_unnamed.sb.height = (*wp).sy;
                if sb_left != 0 {
                    (*bc).data.c2rust_unnamed.sb.flags |= REDRAW_SCROLLBAR_LEFT;
                } else {
                    (*bc).data.c2rust_unnamed.sb.flags |= REDRAW_SCROLLBAR_RIGHT;
                }
                if overlay != 0 {
                    (*bc).data.c2rust_unnamed.sb.flags |= REDRAW_SCROLLBAR_OVERLAY;
                }
            }
            wx += 1;
        }
        sy = sy.wrapping_add(1);
    }
}
unsafe extern "C" fn redraw_data_has_pane(
    mut data: *mut redraw_span_data,
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    if (*data).c2rust_unnamed.b.top_wp == wp {
        return 1 as ::core::ffi::c_int;
    }
    if (*data).c2rust_unnamed.b.bottom_wp == wp {
        return 1 as ::core::ffi::c_int;
    }
    if (*data).c2rust_unnamed.b.left_wp == wp {
        return 1 as ::core::ffi::c_int;
    }
    if (*data).c2rust_unnamed.b.right_wp == wp {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn redraw_mark_border_cell(
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
        if (*bc).data.type_0 as ::core::ffi::c_uint
            == REDRAW_SPAN_EMPTY as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*bc).data.type_0 as ::core::ffi::c_uint
                == REDRAW_SPAN_OUTSIDE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            reset = 1 as ::core::ffi::c_int;
        } else if (*bc).data.type_0 as ::core::ffi::c_uint
            != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return;
        }
    } else if (*bc).data.type_0 as ::core::ffi::c_uint
        != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
        || redraw_data_has_pane(&raw mut (*bc).data, wp) == 0
    {
        reset = 1 as ::core::ffi::c_int;
    }
    if reset != 0 {
        memset(
            bc as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<redraw_build_cell>() as size_t,
        );
        (*bc).data.type_0 = REDRAW_SPAN_BORDER;
    }
    if top_owner != 0 {
        (*bc).data.c2rust_unnamed.b.top_wp = wp;
        (*bc).data.c2rust_unnamed.b.top_lines = pane_lines;
    }
    if bottom_owner != 0 {
        (*bc).data.c2rust_unnamed.b.bottom_wp = wp;
        (*bc).data.c2rust_unnamed.b.bottom_lines = pane_lines;
    }
    if mask & (REDRAW_BORDER_U | REDRAW_BORDER_D) != 0 {
        if wx < (*wp).xoff {
            (*bc).data.c2rust_unnamed.b.right_wp = wp;
            (*bc).data.c2rust_unnamed.b.right_lines = pane_lines;
        } else if wx >= (*wp).xoff + (*wp).sx as ::core::ffi::c_int {
            (*bc).data.c2rust_unnamed.b.left_wp = wp;
            (*bc).data.c2rust_unnamed.b.left_lines = pane_lines;
        }
    }
    mask |= (*bc).data.c2rust_unnamed.b.cell_mask;
    (*bc).data.c2rust_unnamed.b.cell_mask = mask;
    (*bc).data.c2rust_unnamed.b.cell_type = redraw_get_cell_type(mask);
}
unsafe extern "C" fn redraw_mark_border_status(
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
            if !((*bc).data.type_0 as ::core::ffi::c_uint
                != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                cell_type = (*bc).data.c2rust_unnamed.b.cell_type;
                (*bc).data.type_0 = REDRAW_SPAN_STATUS;
                (*bc).data.c2rust_unnamed.st.wp = wp;
                (*bc).data.c2rust_unnamed.st.offset = off;
                (*bc).data.c2rust_unnamed.st.cell_type = cell_type;
            }
        }
        wx += 1;
        off = off.wrapping_add(1);
    }
}
unsafe extern "C" fn redraw_mark_border_arrows(
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
            if (*bc).data.type_0 as ::core::ffi::c_uint
                == REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*bc).data.c2rust_unnamed.b.flags |= REDRAW_BORDER_IS_ARROW;
            }
        }
        wy = bottom;
        if redraw_window_to_scene(bctx, wx, wy, &raw mut x, &raw mut y) != 0 {
            bc = redraw_get_build_cell(bctx, x, y);
            if (*bc).data.type_0 as ::core::ffi::c_uint
                == REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*bc).data.c2rust_unnamed.b.flags |= REDRAW_BORDER_IS_ARROW;
            }
        }
    }
    wy = (*wp).yoff + 1 as ::core::ffi::c_int;
    if wy >= top && wy <= bottom {
        wx = left;
        if redraw_window_to_scene(bctx, wx, wy, &raw mut x, &raw mut y) != 0 {
            bc = redraw_get_build_cell(bctx, x, y);
            if (*bc).data.type_0 as ::core::ffi::c_uint
                == REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*bc).data.c2rust_unnamed.b.flags |= REDRAW_BORDER_IS_ARROW;
            }
        }
        wx = right;
        if redraw_window_to_scene(bctx, wx, wy, &raw mut x, &raw mut y) != 0 {
            bc = redraw_get_build_cell(bctx, x, y);
            if (*bc).data.type_0 as ::core::ffi::c_uint
                == REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*bc).data.c2rust_unnamed.b.flags |= REDRAW_BORDER_IS_ARROW;
            }
        }
    }
}
unsafe extern "C" fn redraw_mark_pane_borders(
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
    redraw_mark_border_status(bctx, wp, left, right, top, bottom);
    redraw_mark_border_arrows(bctx, wp, left, right, top, bottom);
}
unsafe extern "C" fn redraw_mark_pane(mut bctx: *mut redraw_build_ctx, mut wp: *mut window_pane) {
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
unsafe extern "C" fn redraw_mark_two_pane_colours(mut bctx: *mut redraw_build_ctx) {
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
            if !((*bc).data.type_0 as ::core::ffi::c_uint
                != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                sd = &raw mut (*bc).data;
                wx = (*bctx).ox.wrapping_add(x);
                wy = (*bctx).oy.wrapping_add(y);
                if type_0 as ::core::ffi::c_uint
                    == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
                    && !(*sd).c2rust_unnamed.b.left_wp.is_null()
                    && !(*sd).c2rust_unnamed.b.right_wp.is_null()
                {
                    if wy <= (*(*bctx).w).sy.wrapping_div(2 as u_int) {
                        (*sd).c2rust_unnamed.b.style_wp = (*sd).c2rust_unnamed.b.left_wp;
                    } else {
                        (*sd).c2rust_unnamed.b.style_wp = (*sd).c2rust_unnamed.b.right_wp;
                    }
                } else if type_0 as ::core::ffi::c_uint
                    == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
                    && !(*sd).c2rust_unnamed.b.top_wp.is_null()
                    && !(*sd).c2rust_unnamed.b.bottom_wp.is_null()
                {
                    if wx <= (*(*bctx).w).sx.wrapping_div(2 as u_int) {
                        (*sd).c2rust_unnamed.b.style_wp = (*sd).c2rust_unnamed.b.top_wp;
                    } else {
                        (*sd).c2rust_unnamed.b.style_wp = (*sd).c2rust_unnamed.b.bottom_wp;
                    }
                }
            }
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
}
unsafe extern "C" fn redraw_mark_menu(mut bctx: *mut redraw_build_ctx) {
    let mut md: *mut menu_data = (*(*bctx).w).menu;
    let mut bc: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if md.is_null() {
        return;
    }
    sx = menu_width(md);
    sy = menu_height(md);
    py = 0 as u_int;
    while py < sy {
        px = 0 as u_int;
        while px < sx {
            if !(redraw_window_to_scene(
                bctx,
                menu_x(md).wrapping_add(px) as ::core::ffi::c_int,
                menu_y(md).wrapping_add(py) as ::core::ffi::c_int,
                &raw mut x,
                &raw mut y,
            ) == 0)
            {
                bc = redraw_get_build_cell(bctx, x, y);
                memset(
                    bc as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    ::core::mem::size_of::<redraw_build_cell>() as size_t,
                );
                (*bc).data.type_0 = REDRAW_SPAN_MENU;
                (*bc).data.c2rust_unnamed.m.md = md;
                (*bc).data.c2rust_unnamed.m.px = px;
                (*bc).data.c2rust_unnamed.m.py = py;
            }
            px = px.wrapping_add(1);
        }
        py = py.wrapping_add(1);
    }
}
unsafe extern "C" fn redraw_compare_data(
    mut a: *mut redraw_build_cell,
    mut b: *mut redraw_build_cell,
) -> ::core::ffi::c_int {
    let mut ad: *mut redraw_span_data = &raw mut (*a).data;
    let mut bd: *mut redraw_span_data = &raw mut (*b).data;
    if (*ad).type_0 as ::core::ffi::c_uint != (*bd).type_0 as ::core::ffi::c_uint {
        return 0 as ::core::ffi::c_int;
    }
    match (*ad).type_0 as ::core::ffi::c_uint {
        0 => {
            if (*ad).c2rust_unnamed.p.wp != (*bd).c2rust_unnamed.p.wp
                || (*ad).c2rust_unnamed.p.py != (*bd).c2rust_unnamed.p.py
                || (*ad).c2rust_unnamed.p.px.wrapping_add(1 as u_int) != (*bd).c2rust_unnamed.p.px
            {
                return 0 as ::core::ffi::c_int;
            }
            return 1 as ::core::ffi::c_int;
        }
        4 => {
            if (*ad).c2rust_unnamed.b.top_wp != (*bd).c2rust_unnamed.b.top_wp
                || (*ad).c2rust_unnamed.b.bottom_wp != (*bd).c2rust_unnamed.b.bottom_wp
                || (*ad).c2rust_unnamed.b.left_wp != (*bd).c2rust_unnamed.b.left_wp
                || (*ad).c2rust_unnamed.b.right_wp != (*bd).c2rust_unnamed.b.right_wp
                || (*ad).c2rust_unnamed.b.style_wp != (*bd).c2rust_unnamed.b.style_wp
                || (*ad).c2rust_unnamed.b.top_lines as ::core::ffi::c_uint
                    != (*bd).c2rust_unnamed.b.top_lines as ::core::ffi::c_uint
                || (*ad).c2rust_unnamed.b.bottom_lines as ::core::ffi::c_uint
                    != (*bd).c2rust_unnamed.b.bottom_lines as ::core::ffi::c_uint
                || (*ad).c2rust_unnamed.b.left_lines as ::core::ffi::c_uint
                    != (*bd).c2rust_unnamed.b.left_lines as ::core::ffi::c_uint
                || (*ad).c2rust_unnamed.b.right_lines as ::core::ffi::c_uint
                    != (*bd).c2rust_unnamed.b.right_lines as ::core::ffi::c_uint
                || (*ad).c2rust_unnamed.b.cell_type != (*bd).c2rust_unnamed.b.cell_type
                || (*ad).c2rust_unnamed.b.cell_mask != (*bd).c2rust_unnamed.b.cell_mask
                || (*ad).c2rust_unnamed.b.flags != (*bd).c2rust_unnamed.b.flags
            {
                return 0 as ::core::ffi::c_int;
            }
            if (*ad).c2rust_unnamed.b.flags & REDRAW_BORDER_IS_ARROW != 0 {
                return 0 as ::core::ffi::c_int;
            }
            return 1 as ::core::ffi::c_int;
        }
        3 => {
            if (*ad).c2rust_unnamed.st.wp != (*bd).c2rust_unnamed.st.wp
                || (*ad).c2rust_unnamed.st.offset.wrapping_add(1 as u_int)
                    != (*bd).c2rust_unnamed.st.offset
                || (*ad).c2rust_unnamed.st.cell_type != (*bd).c2rust_unnamed.st.cell_type
            {
                return 0 as ::core::ffi::c_int;
            }
            return 1 as ::core::ffi::c_int;
        }
        5 => {
            if (*ad).c2rust_unnamed.sb.wp != (*bd).c2rust_unnamed.sb.wp
                || (*ad).c2rust_unnamed.sb.y != (*bd).c2rust_unnamed.sb.y
                || (*ad).c2rust_unnamed.sb.height != (*bd).c2rust_unnamed.sb.height
                || (*ad).c2rust_unnamed.sb.flags != (*bd).c2rust_unnamed.sb.flags
            {
                return 0 as ::core::ffi::c_int;
            }
            return 1 as ::core::ffi::c_int;
        }
        6 => {
            if (*ad).c2rust_unnamed.m.md != (*bd).c2rust_unnamed.m.md
                || (*ad).c2rust_unnamed.m.py != (*bd).c2rust_unnamed.m.py
                || (*ad).c2rust_unnamed.m.px.wrapping_add(1 as u_int) != (*bd).c2rust_unnamed.m.px
            {
                return 0 as ::core::ffi::c_int;
            }
            return 1 as ::core::ffi::c_int;
        }
        1 | 2 => return 1 as ::core::ffi::c_int,
        _ => {}
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn redraw_build_cells(mut bctx: *mut redraw_build_ctx) {
    let mut w: *mut window = (*bctx).w;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut ncells: size_t = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*bctx).sx != 0 as u_int
        && (*bctx).sy as ::core::ffi::c_ulong
            > SIZE_MAX.wrapping_div((*bctx).sx as ::core::ffi::c_ulong)
    {
        fatalx(
            b"%s: too many cells\0" as *const u8 as *const ::core::ffi::c_char,
            b"redraw_build_cells\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    ncells = ((*bctx).sx as size_t).wrapping_mul((*bctx).sy as size_t);
    if ncells > redraw_ncells {
        redraw_cells = xreallocarray(
            redraw_cells as *mut ::core::ffi::c_void,
            ncells,
            ::core::mem::size_of::<redraw_build_cell>() as size_t,
        ) as *mut redraw_build_cell;
        redraw_ncells = ncells;
    }
    (*bctx).cells = redraw_cells;
    y = 0 as u_int;
    while y < (*bctx).sy {
        x = 0 as u_int;
        while x < (*bctx).sx {
            redraw_reset_cell(bctx, x, y);
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
    wp = *(*((*w).z_index.tqh_last as *mut window_panes_zindex)).tqh_last;
    while !wp.is_null() {
        redraw_mark_pane(bctx, wp);
        wp = *(*((*wp).zentry.tqe_prev as *mut window_panes_zindex)).tqh_last;
    }
    redraw_mark_two_pane_colours(bctx);
    redraw_mark_menu(bctx);
}
unsafe extern "C" fn redraw_make_scene(mut c: *mut client) -> *mut redraw_scene {
    let mut s: *mut session = (*c).session;
    let mut w: *mut window = (*(*s).curw).window;
    let mut bctx: redraw_build_ctx = redraw_build_ctx {
        c: ::core::ptr::null_mut::<client>(),
        w: ::core::ptr::null_mut::<window>(),
        ox: 0,
        oy: 0,
        sx: 0,
        sy: 0,
        ind: 0,
        cells: ::core::ptr::null_mut::<redraw_build_cell>(),
    };
    let mut scene: *mut redraw_scene = ::core::ptr::null_mut::<redraw_scene>();
    let mut bc: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut last: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut line: *mut redraw_line = ::core::ptr::null_mut::<redraw_line>();
    let mut span: *mut redraw_span = ::core::ptr::null_mut::<redraw_span>();
    let mut type_0: redraw_span_type = REDRAW_SPAN_PANE;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut x0: u_int = 0;
    if (*c).flags & CLIENT_SUSPENDED as uint64_t != 0 {
        return ::core::ptr::null_mut::<redraw_scene>();
    }
    redraw_set_context(c, &raw mut bctx);
    log_debug(
        b"%s: building @%u scene (%ux%u %u,%u; generation %llu)\0" as *const u8
            as *const ::core::ffi::c_char,
        (*c).name,
        (*w).id,
        bctx.sx,
        bctx.sy,
        bctx.ox,
        bctx.oy,
        (*w).redraw_scene_generation as ::core::ffi::c_ulonglong,
    );
    redraw_build_cells(&raw mut bctx);
    scene = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<redraw_scene>() as size_t,
    ) as *mut redraw_scene;
    (*scene).c = c;
    (*scene).w = w;
    (*scene).lines = xcalloc(
        bctx.sy as size_t,
        ::core::mem::size_of::<redraw_line>() as size_t,
    ) as *mut redraw_line;
    (*scene).generation = (*w).redraw_scene_generation;
    (*scene).sx = bctx.sx;
    (*scene).sy = bctx.sy;
    (*scene).ox = bctx.ox;
    (*scene).oy = bctx.oy;
    y = 0 as u_int;
    while y < bctx.sy {
        line = (*scene).lines.offset(y as isize) as *mut redraw_line;
        type_0 = REDRAW_SPAN_PANE;
        while (type_0 as ::core::ffi::c_uint) < REDRAW_SPAN_TYPES as ::core::ffi::c_uint {
            (*line).spans[type_0 as usize].tqh_first = ::core::ptr::null_mut::<redraw_span>();
            (*line).spans[type_0 as usize].tqh_last =
                &raw mut (*(&raw mut (*line).spans as *mut redraw_spans).offset(type_0 as isize))
                    .tqh_first;
            type_0 += 1;
        }
        x = 0 as u_int;
        while x < bctx.sx {
            x0 = x;
            last = redraw_get_build_cell(&raw mut bctx, x, y);
            x = x.wrapping_add(1);
            while x < bctx.sx {
                bc = redraw_get_build_cell(&raw mut bctx, x, y);
                if redraw_compare_data(last, bc) == 0 {
                    break;
                }
                last = bc;
                x = x.wrapping_add(1);
            }
            bc = redraw_get_build_cell(&raw mut bctx, x0, y);
            type_0 = (*bc).data.type_0;
            span = xcalloc(1 as size_t, ::core::mem::size_of::<redraw_span>() as size_t)
                as *mut redraw_span;
            (*span).x = x0;
            (*span).width = x.wrapping_sub(x0);
            (*span).data = (*bc).data;
            (*span).entry.tqe_next = ::core::ptr::null_mut::<redraw_span>();
            (*span).entry.tqe_prev = (*line).spans[type_0 as usize].tqh_last;
            *(*line).spans[type_0 as usize].tqh_last = span;
            (*line).spans[type_0 as usize].tqh_last = &raw mut (*span).entry.tqe_next;
        }
        y = y.wrapping_add(1);
    }
    log_debug(
        b"%s: finished building @%u scene\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        (*w).id,
    );
    return scene;
}
#[no_mangle]
pub unsafe extern "C" fn redraw_free_scene(mut scene: *mut redraw_scene) {
    let mut spans: *mut redraw_spans = ::core::ptr::null_mut::<redraw_spans>();
    let mut span: *mut redraw_span = ::core::ptr::null_mut::<redraw_span>();
    let mut span1: *mut redraw_span = ::core::ptr::null_mut::<redraw_span>();
    let mut y: u_int = 0;
    let mut type_0: u_int = 0;
    if scene.is_null() {
        return;
    }
    y = 0 as u_int;
    while y < (*scene).sy {
        type_0 = 0 as u_int;
        while type_0 < REDRAW_SPAN_TYPES as u_int {
            spans = (&raw mut (*(*scene).lines.offset(y as isize)).spans as *mut redraw_spans)
                .offset(type_0 as isize) as *mut redraw_spans;
            span = (*spans).tqh_first;
            while !span.is_null() && {
                span1 = (*span).entry.tqe_next;
                1 as ::core::ffi::c_int != 0
            } {
                if !(*span).entry.tqe_next.is_null() {
                    (*(*span).entry.tqe_next).entry.tqe_prev = (*span).entry.tqe_prev;
                } else {
                    (*spans).tqh_last = (*span).entry.tqe_prev;
                }
                *(*span).entry.tqe_prev = (*span).entry.tqe_next;
                free(span as *mut ::core::ffi::c_void);
                span = span1;
            }
            type_0 = type_0.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
    free((*scene).lines as *mut ::core::ffi::c_void);
    free(scene as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn redraw_invalidate_scene(mut w: *mut window) {
    (*w).redraw_scene_generation = (*w).redraw_scene_generation.wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn redraw_invalidate_all_scenes() {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    w = windows_minmax(&raw mut windows, RB_NEGINF);
    while !w.is_null() {
        redraw_invalidate_scene(w);
        w = windows_next(w);
    }
}
unsafe extern "C" fn redraw_get_scene(mut c: *mut client) -> *mut redraw_scene {
    let mut scene: *mut redraw_scene = (*c).redraw_scene;
    let mut w: *mut window = (*(*(*c).session).curw).window;
    let mut reason: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    redraw_get_window_offset(c, &raw mut ox, &raw mut oy, &raw mut sx, &raw mut sy);
    if scene.is_null() {
        reason = b"missing\0" as *const u8 as *const ::core::ffi::c_char;
    } else if (*scene).w != w {
        reason = b"window changed\0" as *const u8 as *const ::core::ffi::c_char;
    } else if (*scene).generation != (*w).redraw_scene_generation {
        reason = b"generation changed\0" as *const u8 as *const ::core::ffi::c_char;
    } else if (*scene).ox != ox || (*scene).oy != oy {
        reason = b"offset changed\0" as *const u8 as *const ::core::ffi::c_char;
    } else if (*scene).sx != sx || (*scene).sy != sy {
        reason = b"size changed\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if !reason.is_null() {
        log_debug(
            b"%s: @%u scene invalid: %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            (*w).id,
            reason,
        );
        redraw_free_scene(scene);
        scene = redraw_make_scene(c);
        (*c).redraw_scene = scene;
    }
    return scene;
}
unsafe extern "C" fn redraw_draw_pane_span(
    mut dctx: *mut redraw_draw_ctx,
    mut span: *mut redraw_span,
    mut x: u_int,
    mut y: u_int,
    mut n: u_int,
) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut c: *mut client = (*scene).c;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut wp: *mut window_pane = (*span).data.c2rust_unnamed.p.wp;
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
        defaults: ::core::ptr::null::<grid_cell>(),
        palette: ::core::ptr::null_mut::<colour_palette>(),
        dim: 0,
        hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
    };
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    tty_default_colours(&raw mut defaults, wp, &raw mut style_ctx.dim);
    style_ctx.defaults = &raw mut defaults;
    style_ctx.palette = &raw mut (*wp).palette;
    style_ctx.hyperlinks = (*s).hyperlinks;
    px = (*span)
        .data
        .c2rust_unnamed
        .p
        .px
        .wrapping_add(x.wrapping_sub((*span).x));
    py = (*span).data.c2rust_unnamed.p.py;
    tty_draw_line(tty, s, px, py, n, x, y, &raw mut style_ctx);
}
unsafe extern "C" fn redraw_get_default_border_style(
    mut dctx: *mut redraw_draw_ctx,
    mut gc: *mut grid_cell,
    mut pane_lines: *mut pane_lines,
) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut c: *mut client = (*scene).c;
    let mut s: *mut session = (*c).session;
    let mut oo: *mut options = (*(*scene).w).options;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut dgc: *mut grid_cell = &raw mut (*dctx).default_gc;
    if !(*dctx).flags & REDRAW_DEFAULT_SET != 0 {
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
        (*dctx).pane_lines = options_get_number(
            oo,
            b"pane-border-lines\0" as *const u8 as *const ::core::ffi::c_char,
        ) as pane_lines;
        (*dctx).flags |= REDRAW_DEFAULT_SET;
    }
    memcpy(
        gc as *mut ::core::ffi::c_void,
        dgc as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    *pane_lines = (*dctx).pane_lines;
}
unsafe extern "C" fn redraw_get_pane_for_border_style(
    mut dctx: *mut redraw_draw_ctx,
    mut span: *mut redraw_span,
) -> *mut window_pane {
    let mut active: *mut window_pane = (*dctx).active;
    if (*span).data.type_0 as ::core::ffi::c_uint
        != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<window_pane>();
    }
    if !(*span).data.c2rust_unnamed.b.style_wp.is_null() {
        return (*span).data.c2rust_unnamed.b.style_wp;
    }
    if !active.is_null() && redraw_data_has_pane(&raw mut (*span).data, active) != 0 {
        return active;
    }
    if !(*span).data.c2rust_unnamed.b.top_wp.is_null() {
        return (*span).data.c2rust_unnamed.b.top_wp;
    }
    if !(*span).data.c2rust_unnamed.b.bottom_wp.is_null() {
        return (*span).data.c2rust_unnamed.b.bottom_wp;
    }
    if !(*span).data.c2rust_unnamed.b.left_wp.is_null() {
        return (*span).data.c2rust_unnamed.b.left_wp;
    }
    if !(*span).data.c2rust_unnamed.b.right_wp.is_null() {
        return (*span).data.c2rust_unnamed.b.right_wp;
    }
    return ::core::ptr::null_mut::<window_pane>();
}
unsafe extern "C" fn redraw_draw_border_arrow(
    mut dctx: *mut redraw_draw_ctx,
    mut span: *mut redraw_span,
    mut gc: *mut grid_cell,
) {
    let mut active: *mut window_pane = (*dctx).active;
    let mut ch: ::core::ffi::c_char = 0;
    if (*span).data.type_0 as ::core::ffi::c_uint
        != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
        || active.is_null()
    {
        return;
    }
    if !(*span).data.c2rust_unnamed.b.flags & REDRAW_BORDER_IS_ARROW != 0 {
        return;
    }
    if (*span).data.c2rust_unnamed.b.left_wp == active {
        ch = ',' as i32 as ::core::ffi::c_char;
    } else if (*span).data.c2rust_unnamed.b.right_wp == active {
        ch = '+' as i32 as ::core::ffi::c_char;
    } else if (*span).data.c2rust_unnamed.b.top_wp == active {
        ch = '-' as i32 as ::core::ffi::c_char;
    } else if (*span).data.c2rust_unnamed.b.bottom_wp == active {
        ch = '.' as i32 as ::core::ffi::c_char;
    } else {
        return;
    }
    utf8_set(&raw mut (*gc).data, ch as u_char);
    (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
}
unsafe extern "C" fn redraw_draw_border_span(
    mut dctx: *mut redraw_draw_ctx,
    mut span: *mut redraw_span,
    mut x: u_int,
    mut y: u_int,
    mut n: u_int,
) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut c: *mut client = (*scene).c;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut w: *mut window = (*scene).w;
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
    if (*span).data.type_0 as ::core::ffi::c_uint
        != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        cell_type = CELL_NONE as u_int;
    } else {
        wp = redraw_get_pane_for_border_style(dctx, span);
        cell_type = (*span).data.c2rust_unnamed.b.cell_type as u_int;
    }
    if wp.is_null() {
        redraw_get_default_border_style(dctx, &raw mut gc, &raw mut pane_lines);
        if (*span).data.type_0 as ::core::ffi::c_uint
            == REDRAW_SPAN_OUTSIDE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            window_get_fill_cell(w, 0 as ::core::ffi::c_int, &raw mut gc);
        } else if (*span).data.type_0 as ::core::ffi::c_uint
            == REDRAW_SPAN_EMPTY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            window_get_fill_cell(w, 1 as ::core::ffi::c_int, &raw mut gc);
        } else {
            if (*span).data.type_0 as ::core::ffi::c_uint
                != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                pane_lines = PANE_LINES_SINGLE;
            }
            window_get_border_cell(
                ::core::ptr::null_mut::<window_pane>(),
                pane_lines,
                cell_type as ::core::ffi::c_int,
                &raw mut gc,
            );
        }
    } else {
        window_pane_get_border_style(wp, c, &raw mut gc);
        window_pane_get_border_cell(wp, cell_type as ::core::ffi::c_int, &raw mut gc);
    }
    if (*span).data.type_0 as ::core::ffi::c_uint
        == REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*dctx).marked.is_null()
        && redraw_data_has_pane(&raw mut (*span).data, (*dctx).marked) != 0
    {
        gc.attr = (gc.attr as ::core::ffi::c_int ^ GRID_ATTR_REVERSE) as u_short;
    }
    redraw_draw_border_arrow(dctx, span, &raw mut gc);
    if cell_type == CELL_UD as u_int && (*dctx).flags & REDRAW_ISOLATES != 0 {
        isolates = 1 as ::core::ffi::c_int;
    }
    tty_cursor(tty, x, y);
    if isolates != 0 {
        tty_puts(tty, REDRAW_END_ISOLATE.as_ptr());
    }
    i = 0 as u_int;
    while i < n {
        tty_cell(tty, &raw mut gc, ::core::ptr::null::<tty_style_ctx>());
        i = i.wrapping_add(1);
    }
    if isolates != 0 {
        tty_puts(tty, REDRAW_START_ISOLATE.as_ptr());
    }
}
unsafe extern "C" fn redraw_draw_status_span(
    mut dctx: *mut redraw_draw_ctx,
    mut span: *mut redraw_span,
    mut x: u_int,
    mut y: u_int,
    mut n: u_int,
) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut c: *mut client = (*scene).c;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut wp: *mut window_pane = (*span).data.c2rust_unnamed.st.wp;
    let mut s: *mut screen = &raw mut (*wp).status_screen;
    let mut px: u_int = 0;
    let mut sx: u_int = (*(*s).grid).sx;
    px = (*span)
        .data
        .c2rust_unnamed
        .st
        .offset
        .wrapping_add(x.wrapping_sub((*span).x));
    if px < sx {
        if n > sx.wrapping_sub(px) {
            n = sx.wrapping_sub(px);
        }
        tty_draw_line(
            tty,
            s,
            px,
            0 as u_int,
            n,
            x,
            y,
            ::core::ptr::null::<tty_style_ctx>(),
        );
    }
}
unsafe extern "C" fn redraw_draw_scrollbar_span(
    mut dctx: *mut redraw_draw_ctx,
    mut span: *mut redraw_span,
    mut x: u_int,
    mut y: u_int,
    mut n: u_int,
) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut wp: *mut window_pane = (*span).data.c2rust_unnamed.sb.wp;
    let mut s: *mut screen = (*wp).screen;
    let mut tty: *mut tty = &raw mut (*(*scene).c).tty;
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
    let mut sb_h: u_int = (*span).data.c2rust_unnamed.sb.height;
    let mut sb_y: u_int = (*span).data.c2rust_unnamed.sb.y;
    let mut i: u_int = 0;
    let mut off: u_int = 0;
    let mut sb_w: u_int = 0;
    let mut sb_pad: u_int = 0;
    let mut cm_y: ::core::ffi::c_int = 0;
    let mut cm_size: ::core::ffi::c_int = 0;
    if window_pane_mode(wp) == WINDOW_PANE_NO_MODE {
        total_height = (*(*s).grid).sy.wrapping_add((*(*s).grid).hsize);
        if total_height == 0 as u_int {
            return;
        }
        pct_view = sb_h as ::core::ffi::c_double / total_height as ::core::ffi::c_double;
        slider_h = (sb_h as ::core::ffi::c_double * pct_view) as u_int;
        slider_y = sb_h.wrapping_sub(slider_h);
    } else {
        if (*wp).modes.tqh_first.is_null() {
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
    tty_default_colours(&raw mut pad_gc, wp, ::core::ptr::null_mut::<u_int>());
    sb_w = (*sb_style).width as u_int;
    sb_pad = (*sb_style).pad as u_int;
    off = x.wrapping_sub((*span).x);
    tty_cursor(tty, x, y);
    let mut current_block_40: u64;
    i = 0 as u_int;
    while i < n {
        if (*span).data.c2rust_unnamed.sb.flags & REDRAW_SCROLLBAR_LEFT != 0 {
            if off.wrapping_add(i) >= sb_w && off.wrapping_add(i) < sb_w.wrapping_add(sb_pad) {
                tty_cell(tty, &raw mut pad_gc, ::core::ptr::null::<tty_style_ctx>());
                current_block_40 = 3437258052017859086;
            } else {
                current_block_40 = 7828949454673616476;
            }
        } else if off.wrapping_add(i) < sb_pad {
            tty_cell(tty, &raw mut pad_gc, ::core::ptr::null::<tty_style_ctx>());
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
                tty_cell(tty, gcp, ::core::ptr::null::<tty_style_ctx>());
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn redraw_draw_menu_span(
    mut dctx: *mut redraw_draw_ctx,
    mut span: *mut redraw_span,
    mut x: u_int,
    mut y: u_int,
    mut n: u_int,
) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut tty: *mut tty = &raw mut (*(*scene).c).tty;
    let mut s: *mut screen = menu_screen((*span).data.c2rust_unnamed.m.md);
    let mut px: u_int = 0;
    px = (*span)
        .data
        .c2rust_unnamed
        .m
        .px
        .wrapping_add(x.wrapping_sub((*span).x));
    tty_draw_line(
        tty,
        s,
        px,
        (*span).data.c2rust_unnamed.m.py,
        n,
        x,
        y,
        ::core::ptr::null::<tty_style_ctx>(),
    );
}
unsafe extern "C" fn redraw_draw_span(
    mut dctx: *mut redraw_draw_ctx,
    mut span: *mut redraw_span,
    mut y: u_int,
) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut data: *mut redraw_span_data = &raw mut (*span).data;
    let mut type_0: redraw_span_type = (*data).type_0;
    let mut c: *mut client = (*scene).c;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut rr: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    let mut i: u_int = 0;
    let mut x: u_int = 0;
    let mut n: u_int = 0;
    if type_0 as ::core::ffi::c_uint
        == REDRAW_SPAN_STATUS as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*(*data).c2rust_unnamed.st.wp).flags & PANE_NEWSTATUS != 0
    {
        return;
    }
    r = tty_check_overlay_range(tty, (*span).x, y, (*span).width);
    i = 0 as u_int;
    while i < (*r).used {
        rr = (*r).ranges.offset(i as isize) as *mut visible_range;
        if !((*rr).nx == 0 as u_int) {
            x = (*rr).px;
            n = (*rr).nx;
            match (*span).data.type_0 as ::core::ffi::c_uint {
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
unsafe extern "C" fn redraw_draw_pane_lines(
    mut dctx: *mut redraw_draw_ctx,
    mut wp: *mut window_pane,
    mut flags: ::core::ffi::c_int,
) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut line: *mut redraw_line = ::core::ptr::null_mut::<redraw_line>();
    let mut spans: *mut redraw_spans = ::core::ptr::null_mut::<redraw_spans>();
    let mut span: *mut redraw_span = ::core::ptr::null_mut::<redraw_span>();
    let mut cy: u_int = 0;
    let mut y: ::core::ffi::c_int = 0;
    let mut top: ::core::ffi::c_int = 0;
    let mut bottom: ::core::ffi::c_int = 0;
    top = (*wp).yoff - (*scene).oy as ::core::ffi::c_int;
    if top < 0 as ::core::ffi::c_int {
        top = 0 as ::core::ffi::c_int;
    }
    bottom = (*wp).yoff + (*wp).sy as ::core::ffi::c_int - (*scene).oy as ::core::ffi::c_int;
    if bottom < 0 as ::core::ffi::c_int {
        bottom = 0 as ::core::ffi::c_int;
    }
    if bottom > (*scene).sy as ::core::ffi::c_int {
        bottom = (*scene).sy as ::core::ffi::c_int;
    }
    y = top;
    while y < bottom {
        line = (*scene).lines.offset(y as isize) as *mut redraw_line;
        if (*dctx).flags & REDRAW_STATUS_TOP != 0 {
            cy = (*dctx).status_lines.wrapping_add(y as u_int);
        } else {
            cy = y as u_int;
        }
        if flags & REDRAW_PANE != 0 {
            spans = (&raw mut (*line).spans as *mut redraw_spans)
                .offset(REDRAW_SPAN_PANE as ::core::ffi::c_int as isize)
                as *mut redraw_spans;
            span = (*spans).tqh_first;
            while !span.is_null() {
                if (*span).data.c2rust_unnamed.p.wp == wp {
                    redraw_draw_span(dctx, span, cy);
                }
                span = (*span).entry.tqe_next;
            }
        }
        if flags & REDRAW_PANE_SCROLLBAR != 0 {
            spans = (&raw mut (*line).spans as *mut redraw_spans)
                .offset(REDRAW_SPAN_SCROLLBAR as ::core::ffi::c_int as isize)
                as *mut redraw_spans;
            span = (*spans).tqh_first;
            while !span.is_null() {
                if (*span).data.c2rust_unnamed.sb.wp == wp {
                    redraw_draw_span(dctx, span, cy);
                }
                span = (*span).entry.tqe_next;
            }
        }
        y += 1;
    }
}
unsafe extern "C" fn redraw_draw_lines(
    mut dctx: *mut redraw_draw_ctx,
    mut flags: ::core::ffi::c_int,
) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut line: *mut redraw_line = ::core::ptr::null_mut::<redraw_line>();
    let mut spans: *mut redraw_spans = ::core::ptr::null_mut::<redraw_spans>();
    let mut span: *mut redraw_span = ::core::ptr::null_mut::<redraw_span>();
    let mut y: u_int = 0;
    let mut cy: u_int = 0;
    let mut type_0: u_int = 0;
    y = 0 as u_int;
    while y < (*scene).sy {
        line = (*scene).lines.offset(y as isize) as *mut redraw_line;
        if (*dctx).flags & REDRAW_STATUS_TOP != 0 {
            cy = (*dctx).status_lines.wrapping_add(y);
        } else {
            cy = y;
        }
        let mut current_block_9: u64;
        type_0 = 0 as u_int;
        while type_0 < REDRAW_SPAN_TYPES as u_int {
            if !(flags == REDRAW_ALL) {
                match type_0 {
                    0 => {
                        current_block_9 = 12159231939852168738;
                        match current_block_9 {
                            10170200351409165677 => {
                                if !flags & REDRAW_MENU != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            16319640041109108851 => {
                                if !flags & REDRAW_OUTSIDE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            11740996042087507061 => {
                                if !flags & REDRAW_EMPTY != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            10899678633117585587 => {
                                if !flags & REDRAW_PANE_BORDER != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            7695618138115701323 => {
                                if !flags & REDRAW_PANE_STATUS != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            6251484932569512093 => {
                                if !flags & REDRAW_PANE_SCROLLBAR != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            _ => {
                                if !flags & REDRAW_PANE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                        }
                    }
                    1 => {
                        current_block_9 = 16319640041109108851;
                        match current_block_9 {
                            10170200351409165677 => {
                                if !flags & REDRAW_MENU != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            16319640041109108851 => {
                                if !flags & REDRAW_OUTSIDE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            11740996042087507061 => {
                                if !flags & REDRAW_EMPTY != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            10899678633117585587 => {
                                if !flags & REDRAW_PANE_BORDER != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            7695618138115701323 => {
                                if !flags & REDRAW_PANE_STATUS != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            6251484932569512093 => {
                                if !flags & REDRAW_PANE_SCROLLBAR != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            _ => {
                                if !flags & REDRAW_PANE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                        }
                    }
                    2 => {
                        current_block_9 = 11740996042087507061;
                        match current_block_9 {
                            10170200351409165677 => {
                                if !flags & REDRAW_MENU != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            16319640041109108851 => {
                                if !flags & REDRAW_OUTSIDE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            11740996042087507061 => {
                                if !flags & REDRAW_EMPTY != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            10899678633117585587 => {
                                if !flags & REDRAW_PANE_BORDER != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            7695618138115701323 => {
                                if !flags & REDRAW_PANE_STATUS != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            6251484932569512093 => {
                                if !flags & REDRAW_PANE_SCROLLBAR != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            _ => {
                                if !flags & REDRAW_PANE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                        }
                    }
                    4 => {
                        current_block_9 = 10899678633117585587;
                        match current_block_9 {
                            10170200351409165677 => {
                                if !flags & REDRAW_MENU != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            16319640041109108851 => {
                                if !flags & REDRAW_OUTSIDE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            11740996042087507061 => {
                                if !flags & REDRAW_EMPTY != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            10899678633117585587 => {
                                if !flags & REDRAW_PANE_BORDER != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            7695618138115701323 => {
                                if !flags & REDRAW_PANE_STATUS != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            6251484932569512093 => {
                                if !flags & REDRAW_PANE_SCROLLBAR != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            _ => {
                                if !flags & REDRAW_PANE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                        }
                    }
                    3 => {
                        current_block_9 = 7695618138115701323;
                        match current_block_9 {
                            10170200351409165677 => {
                                if !flags & REDRAW_MENU != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            16319640041109108851 => {
                                if !flags & REDRAW_OUTSIDE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            11740996042087507061 => {
                                if !flags & REDRAW_EMPTY != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            10899678633117585587 => {
                                if !flags & REDRAW_PANE_BORDER != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            7695618138115701323 => {
                                if !flags & REDRAW_PANE_STATUS != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            6251484932569512093 => {
                                if !flags & REDRAW_PANE_SCROLLBAR != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            _ => {
                                if !flags & REDRAW_PANE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                        }
                    }
                    5 => {
                        current_block_9 = 6251484932569512093;
                        match current_block_9 {
                            10170200351409165677 => {
                                if !flags & REDRAW_MENU != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            16319640041109108851 => {
                                if !flags & REDRAW_OUTSIDE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            11740996042087507061 => {
                                if !flags & REDRAW_EMPTY != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            10899678633117585587 => {
                                if !flags & REDRAW_PANE_BORDER != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            7695618138115701323 => {
                                if !flags & REDRAW_PANE_STATUS != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            6251484932569512093 => {
                                if !flags & REDRAW_PANE_SCROLLBAR != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            _ => {
                                if !flags & REDRAW_PANE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                        }
                    }
                    6 => {
                        current_block_9 = 10170200351409165677;
                        match current_block_9 {
                            10170200351409165677 => {
                                if !flags & REDRAW_MENU != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            16319640041109108851 => {
                                if !flags & REDRAW_OUTSIDE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            11740996042087507061 => {
                                if !flags & REDRAW_EMPTY != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            10899678633117585587 => {
                                if !flags & REDRAW_PANE_BORDER != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            7695618138115701323 => {
                                if !flags & REDRAW_PANE_STATUS != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            6251484932569512093 => {
                                if !flags & REDRAW_PANE_SCROLLBAR != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            _ => {
                                if !flags & REDRAW_PANE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                        }
                    }
                    _ => {
                        current_block_9 = 7351195479953500246;
                    }
                }
            } else {
                current_block_9 = 11194104282611034094;
            }
            match current_block_9 {
                11194104282611034094 => {
                    spans = (&raw mut (*line).spans as *mut redraw_spans).offset(type_0 as isize)
                        as *mut redraw_spans;
                    span = (*spans).tqh_first;
                    while !span.is_null() {
                        redraw_draw_span(dctx, span, cy);
                        span = (*span).entry.tqe_next;
                    }
                }
                _ => {}
            }
            type_0 = type_0.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
}
unsafe extern "C" fn redraw_draw_menu_lines(mut dctx: *mut redraw_draw_ctx) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut line: *mut redraw_line = ::core::ptr::null_mut::<redraw_line>();
    let mut span: *mut redraw_span = ::core::ptr::null_mut::<redraw_span>();
    let mut y: u_int = 0;
    let mut cy: u_int = 0;
    y = 0 as u_int;
    while y < (*scene).sy {
        line = (*scene).lines.offset(y as isize) as *mut redraw_line;
        if (*dctx).flags & REDRAW_STATUS_TOP != 0 {
            cy = (*dctx).status_lines.wrapping_add(y);
        } else {
            cy = y;
        }
        span = (*line).spans[REDRAW_SPAN_MENU as ::core::ffi::c_int as usize].tqh_first;
        while !span.is_null() {
            redraw_draw_span(dctx, span, cy);
            span = (*span).entry.tqe_next;
        }
        y = y.wrapping_add(1);
    }
}
unsafe extern "C" fn redraw_pane_status_line(
    mut dctx: *mut redraw_draw_ctx,
    mut wp: *mut window_pane,
    mut line: *mut u_int,
) -> ::core::ffi::c_int {
    let mut scene: *mut redraw_scene = (*dctx).scene;
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
    if wy < 0 as ::core::ffi::c_int || wy < (*scene).oy as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if wy as u_int >= (*scene).oy.wrapping_add((*scene).sy) {
        return 0 as ::core::ffi::c_int;
    }
    *line = (wy as u_int).wrapping_sub((*scene).oy);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn redraw_pane_status_width(
    mut dctx: *mut redraw_draw_ctx,
    mut wp: *mut window_pane,
    mut first: *mut *mut redraw_span,
) -> u_int {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut span: *mut redraw_span = ::core::ptr::null_mut::<redraw_span>();
    let mut y: u_int = 0;
    let mut width: u_int = 0 as u_int;
    let mut end: u_int = 0;
    if redraw_pane_status_line(dctx, wp, &raw mut y) == 0 {
        return 0 as u_int;
    }
    *first = ::core::ptr::null_mut::<redraw_span>();
    span = (*(*scene).lines.offset(y as isize)).spans
        [REDRAW_SPAN_STATUS as ::core::ffi::c_int as usize]
        .tqh_first;
    while !span.is_null() {
        if (*span).data.c2rust_unnamed.st.wp == wp {
            if (*first).is_null() {
                *first = span;
            }
            end = (*span)
                .data
                .c2rust_unnamed
                .st
                .offset
                .wrapping_add((*span).width);
            if end > width {
                width = end;
            }
        }
        span = (*span).entry.tqe_next;
    }
    return width;
}
unsafe extern "C" fn redraw_set_draw_context(
    mut dctx: *mut redraw_draw_ctx,
    mut scene: *mut redraw_scene,
) {
    let mut c: *mut client = (*scene).c;
    let mut s: *mut session = (*c).session;
    let mut oo: *mut options = (*s).options;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut lines: u_int = 0;
    memset(
        dctx as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<redraw_draw_ctx>() as size_t,
    );
    (*dctx).scene = scene;
    if server_is_marked(s, (*s).curw, marked_pane.wp) != 0 {
        (*dctx).marked = marked_pane.wp;
    }
    (*dctx).active = (*(*(*s).curw).window).active;
    lines = status_line_size(c);
    if options_get_number(
        oo,
        b"status-position\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_longlong
    {
        (*dctx).flags |= REDRAW_STATUS_TOP;
    }
    (*dctx).status_lines = lines;
    if (*c).flags & CLIENT_UTF8 as uint64_t != 0 && tty_term_has((*tty).term, TTYC_BIDI) != 0 {
        (*dctx).flags |= REDRAW_ISOLATES;
    }
}
unsafe extern "C" fn redraw_draw_pane_prompt(
    mut dctx: *mut redraw_draw_ctx,
    mut wp: *mut window_pane,
) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut c: *mut client = (*scene).c;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut screen: screen = screen {
        title: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        titles: ::core::ptr::null_mut::<screen_titles>(),
        ntitles: 0,
        grid: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
        cstyle: SCREEN_CURSOR_DEFAULT,
        default_cstyle: SCREEN_CURSOR_DEFAULT,
        ccolour: 0,
        default_ccolour: 0,
        rupper: 0,
        rlower: 0,
        mode: 0,
        default_mode: 0,
        saved_cx: 0,
        saved_cy: 0,
        saved_grid: ::core::ptr::null_mut::<grid>(),
        saved_cell: grid_cell {
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
        saved_flags: 0,
        tabs: ::core::ptr::null_mut::<bitstr_t>(),
        sel: ::core::ptr::null_mut::<screen_sel>(),
        write_list: ::core::ptr::null_mut::<screen_write_cline>(),
        hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        progress_bar: progress_bar {
            state: PROGRESS_BAR_HIDDEN,
            progress: 0,
        },
    };
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        item: ::core::ptr::null_mut::<screen_write_citem>(),
        scrolled: 0,
        bg: 0,
    };
    let mut pdd: prompt_draw_data = prompt_draw_data {
        ctx: ::core::ptr::null_mut::<screen_write_ctx>(),
        cursor_x: ::core::ptr::null_mut::<u_int>(),
        area_x: 0,
        area_width: 0,
        prompt_line: 0,
    };
    let mut ox: ::core::ffi::c_int = (*scene).ox as ::core::ffi::c_int;
    let mut oy: ::core::ffi::c_int = (*scene).oy as ::core::ffi::c_int;
    let mut sx: ::core::ffi::c_int = (*scene).sx as ::core::ffi::c_int;
    let mut sy: ::core::ffi::c_int = (*scene).sy as ::core::ffi::c_int;
    let mut line: ::core::ffi::c_int = 0;
    let mut cy: ::core::ffi::c_int = 0;
    let mut px: ::core::ffi::c_int = 0;
    let mut offset: ::core::ffi::c_int = 0;
    let mut width: ::core::ffi::c_int = 0;
    let mut wy: ::core::ffi::c_int = 0;
    if (*wp).prompt.is_null() || (*wp).sx == 0 as u_int || (*wp).sy == 0 as u_int {
        return;
    }
    if !(*dctx).flags & REDRAW_STATUS_TOP != 0 {
        wy = (*wp).yoff + (*wp).sy as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    } else {
        wy = (*wp).yoff;
    }
    if wy < oy || wy >= oy + sy {
        return;
    }
    line = wy - oy;
    if (*dctx).flags & REDRAW_STATUS_TOP != 0 {
        cy = (*dctx).status_lines.wrapping_add(line as u_int) as ::core::ffi::c_int;
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
    screen_init(&raw mut screen, (*wp).sx, 1 as u_int, 0 as u_int);
    screen_write_start(&raw mut ctx, &raw mut screen);
    pdd.ctx = &raw mut ctx;
    pdd.cursor_x = &raw mut (*wp).prompt_cx;
    pdd.area_x = 0 as u_int;
    pdd.area_width = (*wp).sx;
    pdd.prompt_line = 0 as u_int;
    prompt_draw((*wp).prompt, &raw mut pdd);
    screen_write_stop(&raw mut ctx);
    tty_draw_line(
        tty,
        &raw mut screen,
        0 as u_int,
        offset as u_int,
        width as u_int,
        px as u_int,
        cy as u_int,
        ::core::ptr::null::<tty_style_ctx>(),
    );
    screen_free(&raw mut screen);
}
unsafe extern "C" fn redraw_draw(
    mut c: *mut client,
    mut wp: *mut window_pane,
    mut flags: ::core::ffi::c_int,
) {
    let mut dctx: redraw_draw_ctx = redraw_draw_ctx {
        scene: ::core::ptr::null_mut::<redraw_scene>(),
        active: ::core::ptr::null_mut::<window_pane>(),
        marked: ::core::ptr::null_mut::<window_pane>(),
        status_lines: 0,
        pane_lines: PANE_LINES_SINGLE,
        default_gc: grid_cell {
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
        flags: 0,
    };
    let mut s: *mut session = (*c).session;
    let mut w: *mut window = (*(*s).curw).window;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut sl: *mut screen = ::core::ptr::null_mut::<screen>();
    let mut scene: *mut redraw_scene = ::core::ptr::null_mut::<redraw_scene>();
    let mut loop_0: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut width: u_int = 0;
    let mut i: u_int = 0;
    let mut y: u_int = 0;
    let mut lines: u_int = 0;
    let mut j: u_int = 0;
    let mut first: *mut redraw_span = ::core::ptr::null_mut::<redraw_span>();
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut rr: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    let mut redraw: ::core::ffi::c_int = 0;
    if (*c).flags & CLIENT_SUSPENDED as uint64_t != 0 {
        return;
    }
    if flags & REDRAW_STATUS != 0 {
        if !(*c).message_string.is_null() {
            redraw = status_message_redraw(c);
        } else if !(*c).prompt.is_null() {
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
        log_debug(
            b"%s: starting @%u redraw (%s)\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            (*w).id,
            redraw_flags_to_string(flags),
        );
    }
    scene = redraw_get_scene(c);
    if scene.is_null() {
        return;
    }
    redraw_set_draw_context(&raw mut dctx, scene);
    if !(*w).menu.is_null() {
        menu_update((*w).menu);
    }
    if flags & (REDRAW_PANE_BORDER | REDRAW_PANE_STATUS) != 0 {
        loop_0 = (*(*scene).w).panes.tqh_first;
        while !loop_0.is_null() {
            (*loop_0).border_gc_set = 0 as ::core::ffi::c_int;
            (*loop_0).active_border_gc_set = 0 as ::core::ffi::c_int;
            loop_0 = (*loop_0).entry.tqe_next;
        }
    }
    if flags & REDRAW_PANE_STATUS != 0 {
        redraw = 0 as ::core::ffi::c_int;
        loop_0 = (*(*scene).w).panes.tqh_first;
        while !loop_0.is_null() {
            if flags == REDRAW_ALL {
                (*loop_0).flags |= PANE_NEWSTATUS;
            } else {
                (*loop_0).flags &= !PANE_NEWSTATUS;
            }
            width = redraw_pane_status_width(&raw mut dctx, loop_0, &raw mut first);
            if !(width == 0 as u_int) {
                if window_make_pane_status(loop_0, c, width, first) != 0 {
                    (*loop_0).flags |= PANE_NEWSTATUS;
                    redraw = 1 as ::core::ffi::c_int;
                }
            }
            loop_0 = (*loop_0).entry.tqe_next;
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
            loop_0 = (*(*scene).w).panes.tqh_first;
            while !loop_0.is_null() {
                if !(window_pane_is_visible(loop_0) == 0) {
                    if (*loop_0).base.mode & MODE_SYNC != 0 {
                        screen_write_stop_sync(loop_0);
                    }
                    screen_write_clear_dirty(loop_0);
                }
                loop_0 = (*loop_0).entry.tqe_next;
            }
        }
    }
    tty_sync_start(tty);
    tty_update_mode(
        tty,
        (*tty).mode & !CURSOR_MODES,
        ::core::ptr::null_mut::<screen>(),
    );
    if !wp.is_null() {
        redraw_draw_pane_lines(&raw mut dctx, wp, flags);
    } else {
        redraw_draw_lines(&raw mut dctx, flags);
    }
    if flags & REDRAW_PANE != 0 {
        if !wp.is_null() {
            redraw_draw_pane_prompt(&raw mut dctx, wp);
        } else {
            loop_0 = (*(*scene).w).panes.tqh_first;
            while !loop_0.is_null() {
                if window_pane_is_visible(loop_0) != 0 {
                    redraw_draw_pane_prompt(&raw mut dctx, loop_0);
                }
                loop_0 = (*loop_0).entry.tqe_next;
            }
        }
    }
    if !(*w).menu.is_null() && flags & REDRAW_MENU != 0 {
        redraw_draw_menu_lines(&raw mut dctx);
    }
    if flags & REDRAW_STATUS != 0 {
        lines = dctx.status_lines;
        if !(*c).message_string.is_null() || !(*c).prompt.is_null() {
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
        sl = (*c).status.active;
        i = 0 as u_int;
        while i < lines {
            r = tty_check_overlay_range(tty, 0 as u_int, y.wrapping_add(i), (*tty).sx);
            j = 0 as u_int;
            while j < (*r).used {
                rr = (*r).ranges.offset(j as isize) as *mut visible_range;
                if !((*rr).nx == 0 as u_int) {
                    tty_draw_line(
                        tty,
                        sl,
                        (*rr).px,
                        i,
                        (*rr).nx,
                        (*rr).px,
                        y.wrapping_add(i),
                        ::core::ptr::null::<tty_style_ctx>(),
                    );
                }
                j = j.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
    }
    if (*c).overlay_draw.is_some() && flags & REDRAW_OVERLAY != 0 {
        (*c).overlay_draw.expect("non-null function pointer")(c, (*c).overlay_data);
    }
    tty_reset(tty);
    log_debug(
        b"%s: finished @%u redraw\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        (*(*scene).w).id,
    );
}
#[no_mangle]
pub unsafe extern "C" fn redraw_get_status_border_cell_type(
    mut spanp: *mut *mut redraw_span,
    mut x: u_int,
) -> ::core::ffi::c_int {
    let mut span: *mut redraw_span = *spanp;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut start: u_int = 0;
    let mut end: u_int = 0;
    if span.is_null()
        || (*span).data.type_0 as ::core::ffi::c_uint
            != REDRAW_SPAN_STATUS as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 2 as ::core::ffi::c_int;
    }
    wp = (*span).data.c2rust_unnamed.st.wp;
    while !span.is_null() {
        if !((*span).data.type_0 as ::core::ffi::c_uint
            != REDRAW_SPAN_STATUS as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            if !((*span).data.c2rust_unnamed.st.wp != wp) {
                start = (*span).data.c2rust_unnamed.st.offset;
                end = start.wrapping_add((*span).width);
                if x >= start && x < end {
                    *spanp = span;
                    return (*span).data.c2rust_unnamed.st.cell_type;
                }
                if start > x {
                    *spanp = span;
                    break;
                }
            }
        }
        span = (*span).entry.tqe_next;
    }
    if span.is_null() {
        *spanp = ::core::ptr::null_mut::<redraw_span>();
    }
    return 2 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn redraw_screen(mut c: *mut client) {
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
        if !(*(*(*(*c).session).curw).window).menu.is_null() {
            flags |= REDRAW_MENU;
        }
        if flags != 0 as ::core::ffi::c_int {
            redraw_draw(c, ::core::ptr::null_mut::<window_pane>(), flags);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn redraw_pane(mut c: *mut client, mut wp: *mut window_pane) {
    redraw_draw(c, wp, REDRAW_PANE | REDRAW_PANE_SCROLLBAR);
    if !(*(*(*(*c).session).curw).window).menu.is_null() {
        redraw_draw(c, ::core::ptr::null_mut::<window_pane>(), REDRAW_MENU);
    }
}
#[no_mangle]
pub unsafe extern "C" fn redraw_pane_scrollbar(mut c: *mut client, mut wp: *mut window_pane) {
    redraw_draw(c, wp, REDRAW_PANE_SCROLLBAR);
}
