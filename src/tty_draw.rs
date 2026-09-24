use crate::src::ffi::libc::memcpy;
use crate::src::grid::{grid_cells_look_equal, grid_default_cell, grid_get_line};
use crate::src::grid_view::grid_view_get_cell;
use crate::src::log::{fatalx, log_debug, log_get_level};
use crate::src::screen::screen_select_cell;
use crate::src::shared::abi::*;
pub use crate::src::shared::arguments::args;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
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
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::options;
pub use crate::src::shared::pane::{
    window_pane, window_pane_modes, window_pane_prompt, window_pane_tree_entry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tty::TTY_NOCURSOR;
use crate::src::shared::tty::*;
pub use crate::src::shared::tty::{
    tty, tty_code, tty_key, tty_style_ctx, tty_term, tty_term_entry,
};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::tty::{
    tty_attributes, tty_check_codeset, tty_cursor, tty_default_attributes, tty_fake_bce,
    tty_margin_off, tty_putc, tty_putcode, tty_putcode_i, tty_putn, tty_region_off,
    tty_repeat_space, tty_update_mode,
};
use crate::src::tty_term::tty_term_has;

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

pub type tty_draw_line_state = ::core::ffi::c_uint;
pub const TTY_DRAW_LINE_DONE: tty_draw_line_state = 6;
pub const TTY_DRAW_LINE_SAME: tty_draw_line_state = 5;
pub const TTY_DRAW_LINE_EMPTY: tty_draw_line_state = 4;
pub const TTY_DRAW_LINE_NEW2: tty_draw_line_state = 3;
pub const TTY_DRAW_LINE_NEW1: tty_draw_line_state = 2;
pub const TTY_DRAW_LINE_FLUSH: tty_draw_line_state = 1;
pub const TTY_DRAW_LINE_FIRST: tty_draw_line_state = 0;
static mut tty_draw_line_states: [*const ::core::ffi::c_char; 7] = [
    b"FIRST\0" as *const u8 as *const ::core::ffi::c_char,
    b"FLUSH\0" as *const u8 as *const ::core::ffi::c_char,
    b"NEW1\0" as *const u8 as *const ::core::ffi::c_char,
    b"NEW2\0" as *const u8 as *const ::core::ffi::c_char,
    b"EMPTY\0" as *const u8 as *const ::core::ffi::c_char,
    b"SAME\0" as *const u8 as *const ::core::ffi::c_char,
    b"DONE\0" as *const u8 as *const ::core::ffi::c_char,
];
unsafe extern "C" fn tty_draw_line_clear(
    mut tty: *mut tty,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut defaults: *const grid_cell,
    mut bg: u_int,
    mut wrapped: ::core::ffi::c_int,
) {
    if nx == 0 as u_int {
        return;
    }
    if (*(*tty).client).overlay_check.is_none()
        && wrapped == 0
        && nx >= 10 as u_int
        && tty_fake_bce(tty, defaults, bg) == 0
    {
        if px.wrapping_add(nx) >= (*tty).sx && tty_term_has((*tty).term, TTYC_EL) != 0 {
            tty_cursor(tty, px, py);
            tty_putcode(tty, TTYC_EL);
            return;
        }
        if px == 0 as u_int && tty_term_has((*tty).term, TTYC_EL1) != 0 {
            tty_cursor(tty, px.wrapping_add(nx).wrapping_sub(1 as u_int), py);
            tty_putcode(tty, TTYC_EL1);
            return;
        }
        if tty_term_has((*tty).term, TTYC_ECH) != 0 {
            tty_cursor(tty, px, py);
            tty_putcode_i(tty, TTYC_ECH, nx as ::core::ffi::c_int);
            return;
        }
    }
    if px != 0 as u_int || wrapped == 0 {
        tty_cursor(tty, px, py);
    }
    if nx == 1 as u_int {
        tty_putc(tty, ' ' as i32 as u_char);
    } else if nx == 2 as u_int {
        tty_putn(
            tty,
            b"  \0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            2 as size_t,
            2 as u_int,
        );
    } else {
        tty_repeat_space(tty, nx);
    };
}
unsafe extern "C" fn tty_draw_line_get_empty(
    mut gc: *const grid_cell,
    mut last: *const grid_cell,
    mut nx: u_int,
) -> u_int {
    let mut empty: u_int = 0 as u_int;
    if (*gc).data.width as u_int > nx {
        empty = nx;
    } else if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
        empty = 1 as u_int;
    } else if (*gc).data.width as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        empty = 1 as u_int;
    } else if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_SELECTED != 0 {
        empty = 0 as u_int;
    } else if (*gc).bg == (*last).bg
        && (*gc).attr as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        && (*gc).link == 0 as u_int
    {
        if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_CLEARED != 0 {
            empty = 1 as u_int;
        } else if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
            empty = (*gc).data.width as u_int;
        } else if (*gc).data.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            && *(&raw const (*gc).data.data as *const u_char) as ::core::ffi::c_int == ' ' as i32
        {
            empty = 1 as u_int;
        }
    }
    return empty;
}
#[no_mangle]
pub unsafe extern "C" fn tty_draw_line(
    mut tty: *mut tty,
    mut s: *mut screen,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut atx: u_int,
    mut aty: u_int,
    mut style_ctx: *const tty_style_ctx,
) {
    let mut current_block: u64;
    let mut gd: *mut grid = (*s).grid;
    let mut gcp: *const grid_cell = ::core::ptr::null::<grid_cell>();
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
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut last_i: u_int = 0;
    let mut cx: u_int = 0;
    let mut ex: u_int = 0;
    let mut width: u_int = 0;
    let mut cellsize: u_int = 0;
    let mut bg: u_int = 0;
    let mut flags: ::core::ffi::c_int = 0;
    let mut empty: ::core::ffi::c_int = 0;
    let mut wrapped: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut buf: [::core::ffi::c_char; 1000] = [0; 1000];
    let mut len: size_t = 0;
    let mut current_state: tty_draw_line_state = TTY_DRAW_LINE_FIRST;
    let mut next_state: tty_draw_line_state = TTY_DRAW_LINE_FIRST;
    let mut default_style_ctx: tty_style_ctx = tty_style_ctx {
        defaults: ::core::ptr::null::<grid_cell>(),
        palette: ::core::ptr::null_mut::<colour_palette>(),
        dim: 0,
        hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
    };
    let mut defaults: *const grid_cell = ::core::ptr::null::<grid_cell>();
    if style_ctx.is_null() {
        default_style_ctx.defaults = &raw const grid_default_cell;
        default_style_ctx.hyperlinks = (*s).hyperlinks;
        style_ctx = &raw mut default_style_ctx;
    }
    defaults = (*style_ctx).defaults;
    log_debug(
        b"%s: px=%u py=%u nx=%u atx=%u aty=%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"tty_draw_line\0" as *const u8 as *const ::core::ffi::c_char,
        px,
        py,
        nx,
        atx,
        aty,
    );
    if atx >= (*tty).sx {
        return;
    }
    if atx.wrapping_add(nx) >= (*tty).sx {
        nx = (*tty).sx.wrapping_sub(atx);
    }
    if nx == 0 as u_int {
        return;
    }
    cellsize = (*grid_get_line(gd, (*gd).hsize.wrapping_add(py))).cellsize as u_int;
    if (*(*s).grid).sx > cellsize {
        ex = cellsize;
    } else {
        ex = (*(*s).grid).sx;
    }
    log_debug(
        b"%s: drawing %u-%u,%u (end %u) at %u,%u; defaults: fg=%d, bg=%d\0" as *const u8
            as *const ::core::ffi::c_char,
        b"tty_draw_line\0" as *const u8 as *const ::core::ffi::c_char,
        px,
        px.wrapping_add(nx),
        py,
        ex,
        atx,
        aty,
        (*defaults).fg,
        (*defaults).bg,
    );
    flags = (*tty).flags & TTY_NOCURSOR;
    (*tty).flags |= TTY_NOCURSOR;
    tty_update_mode(tty, (*tty).mode, s);
    tty_region_off(tty);
    tty_margin_off(tty);
    memcpy(
        &raw mut last as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    last.bg = (*defaults).bg;
    tty_default_attributes(tty, 8 as u_int, style_ctx);
    cx = 0 as u_int;
    i = px;
    while i < px.wrapping_add(nx) {
        grid_view_get_cell(gd, i, py, &raw mut gc);
        if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
            break;
        }
        cx = cx.wrapping_add(1);
        i = i.wrapping_add(1);
    }
    if cx != 0 as u_int {
        i = px.wrapping_add(1 as u_int);
        while i > 0 as u_int {
            grid_view_get_cell(gd, i.wrapping_sub(1 as u_int), py, &raw mut gc);
            if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
                break;
            }
            i = i.wrapping_sub(1);
        }
        if i == 0 as u_int {
            bg = (*defaults).bg as u_int;
        } else {
            bg = gc.bg as u_int;
            if gc.flags as ::core::ffi::c_int & GRID_FLAG_SELECTED != 0 {
                memcpy(
                    &raw mut ngc as *mut ::core::ffi::c_void,
                    &raw mut gc as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<grid_cell>() as size_t,
                );
                if screen_select_cell(s, &raw mut ngc, &raw mut gc) != 0 {
                    bg = ngc.bg as u_int;
                }
            }
        }
        tty_attributes(tty, &raw mut last, style_ctx);
        log_debug(
            b"%s: clearing %u padding cells\0" as *const u8 as *const ::core::ffi::c_char,
            b"tty_draw_line\0" as *const u8 as *const ::core::ffi::c_char,
            cx,
        );
        tty_draw_line_clear(tty, atx, aty, cx, defaults, bg, 0 as ::core::ffi::c_int);
        if cx == ex {
            current_block = 15064833524635049977;
        } else {
            atx = atx.wrapping_add(cx);
            px = px.wrapping_add(cx);
            nx = nx.wrapping_sub(cx);
            current_block = 16799951812150840583;
        }
    } else {
        current_block = 16799951812150840583;
    }
    match current_block {
        16799951812150840583 => {
            if py != 0 as u_int && atx == 0 as u_int && (*tty).cx >= (*tty).sx && nx == (*tty).sx {
                gl = grid_get_line(gd, (*gd).hsize.wrapping_add(py).wrapping_sub(1 as u_int));
                if (*gl).flags as ::core::ffi::c_int & GRID_LINE_WRAPPED != 0 {
                    wrapped = 1 as ::core::ffi::c_int;
                }
            }
            i = 0 as u_int;
            last_i = i;
            len = 0 as size_t;
            width = 0 as u_int;
            current_state = TTY_DRAW_LINE_FIRST;
            loop {
                if i == nx {
                    empty = 0 as ::core::ffi::c_int;
                    next_state = TTY_DRAW_LINE_DONE;
                    gcp = &raw const grid_default_cell;
                } else {
                    if i > nx {
                        fatalx(
                            b"position %u > width %u\0" as *const u8 as *const ::core::ffi::c_char,
                            i,
                            nx,
                        );
                    }
                    if px >= ex || i >= ex.wrapping_sub(px) {
                        empty = nx.wrapping_sub(i) as ::core::ffi::c_int;
                        gcp = &raw const grid_default_cell;
                    } else {
                        grid_view_get_cell(gd, px.wrapping_add(i), py, &raw mut gc);
                        empty =
                            tty_draw_line_get_empty(&raw mut gc, &raw mut last, nx.wrapping_sub(i))
                                as ::core::ffi::c_int;
                        if empty != 0 as ::core::ffi::c_int {
                            gcp = &raw mut gc;
                        } else {
                            gcp = tty_check_codeset(tty, &raw mut gc);
                            if (*gcp).flags as ::core::ffi::c_int & GRID_FLAG_SELECTED != 0 {
                                memcpy(
                                    &raw mut ngc as *mut ::core::ffi::c_void,
                                    gcp as *const ::core::ffi::c_void,
                                    ::core::mem::size_of::<grid_cell>() as size_t,
                                );
                                if screen_select_cell(s, &raw mut ngc, gcp) != 0 {
                                    gcp = &raw mut ngc;
                                }
                            }
                        }
                    }
                    if empty != 0 as ::core::ffi::c_int {
                        next_state = TTY_DRAW_LINE_EMPTY;
                    } else if current_state as ::core::ffi::c_uint
                        == TTY_DRAW_LINE_FIRST as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        next_state = TTY_DRAW_LINE_SAME;
                    } else if grid_cells_look_equal(gcp, &raw mut last) != 0 {
                        if (*gcp).data.size as usize
                            > (::core::mem::size_of::<[::core::ffi::c_char; 1000]>() as usize)
                                .wrapping_sub(len as usize)
                        {
                            next_state = TTY_DRAW_LINE_FLUSH;
                        } else {
                            next_state = TTY_DRAW_LINE_SAME;
                        }
                    } else if current_state as ::core::ffi::c_uint
                        == TTY_DRAW_LINE_NEW1 as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        next_state = TTY_DRAW_LINE_NEW2;
                    } else {
                        next_state = TTY_DRAW_LINE_NEW1;
                    }
                }
                if log_get_level() != 0 as ::core::ffi::c_int {
                    log_debug(
                        b"%s: cell %u empty %u, bg %u; state: current %s, next %s\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"tty_draw_line\0" as *const u8 as *const ::core::ffi::c_char,
                        px.wrapping_add(i),
                        empty,
                        (*gcp).bg,
                        tty_draw_line_states[current_state as usize],
                        tty_draw_line_states[next_state as usize],
                    );
                }
                if next_state as ::core::ffi::c_uint != current_state as ::core::ffi::c_uint {
                    if current_state as ::core::ffi::c_uint
                        == TTY_DRAW_LINE_EMPTY as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        tty_attributes(tty, &raw mut last, style_ctx);
                        tty_draw_line_clear(
                            tty,
                            atx.wrapping_add(last_i),
                            aty,
                            i.wrapping_sub(last_i),
                            defaults,
                            last.bg as u_int,
                            wrapped,
                        );
                        wrapped = 0 as ::core::ffi::c_int;
                    } else if next_state as ::core::ffi::c_uint
                        != TTY_DRAW_LINE_SAME as ::core::ffi::c_int as ::core::ffi::c_uint
                        && len != 0 as size_t
                    {
                        tty_attributes(tty, &raw mut last, style_ctx);
                        if atx.wrapping_add(i).wrapping_sub(width) != 0 as u_int || wrapped == 0 {
                            tty_cursor(tty, atx.wrapping_add(i).wrapping_sub(width), aty);
                        }
                        if !(last.attr as ::core::ffi::c_int) & GRID_ATTR_CHARSET != 0 {
                            tty_putn(
                                tty,
                                &raw mut buf as *mut ::core::ffi::c_char
                                    as *const ::core::ffi::c_void,
                                len,
                                width,
                            );
                        } else {
                            j = 0 as u_int;
                            while (j as size_t) < len {
                                tty_putc(tty, buf[j as usize] as u_char);
                                j = j.wrapping_add(1);
                            }
                        }
                        len = 0 as size_t;
                        width = 0 as u_int;
                        wrapped = 0 as ::core::ffi::c_int;
                    }
                    last_i = i;
                }
                if next_state as ::core::ffi::c_uint
                    != TTY_DRAW_LINE_EMPTY as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    memcpy(
                        (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize)
                            as *mut ::core::ffi::c_void,
                        &raw const (*gcp).data.data as *const u_char as *const ::core::ffi::c_void,
                        (*gcp).data.size as size_t,
                    );
                    len = len.wrapping_add((*gcp).data.size as size_t);
                    width = width.wrapping_add((*gcp).data.width as u_int);
                }
                if next_state as ::core::ffi::c_uint
                    == TTY_DRAW_LINE_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    break;
                }
                current_state = next_state;
                memcpy(
                    &raw mut last as *mut ::core::ffi::c_void,
                    gcp as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<grid_cell>() as size_t,
                );
                if empty != 0 as ::core::ffi::c_int {
                    i = i.wrapping_add(empty as u_int);
                } else {
                    i = i.wrapping_add((*gcp).data.width as u_int);
                }
            }
        }
        _ => {}
    }
    (*tty).flags = (*tty).flags & !TTY_NOCURSOR | flags;
    tty_update_mode(tty, (*tty).mode, s);
}
