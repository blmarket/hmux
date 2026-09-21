use crate::src::ffi::libc::{free, memcpy};
use crate::src::format::{
    format_add, format_create, format_create_defaults, format_defaults, format_expand,
    format_expand_time, format_free,
};
use crate::src::format_draw::format_draw;
use crate::src::grid::{grid_compare, grid_default_cell};
use crate::src::grid_view::grid_view_get_cell;
use crate::src::options::options_get_string;
use crate::src::screen::{screen_free, screen_init};
use crate::src::screen_redraw::redraw_get_status_border_cell_type;
use crate::src::screen_write::{
    screen_write_cell, screen_write_cursormove, screen_write_start, screen_write_stop,
};
use crate::src::style::{style_apply, style_ranges_free};
use crate::src::tty_acs::{tty_acs_double_borders, tty_acs_heavy_borders, tty_acs_rounded_borders};
use crate::src::utf8::{utf8_copy, utf8_set};
use crate::src::window::{
    window_pane_get_pane_lines, window_pane_get_pane_status, window_pane_index,
};
pub use crate::src::shared::arguments::{args};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::{menu_data};
pub use crate::src::shared::options::{options};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::process::{tmuxpeer};
pub use crate::src::shared::prompt::{prompt};
pub use crate::src::shared::redraw::{redraw_scene, redraw_span};
pub use crate::src::shared::screen_write::{screen_write_ctx, screen_write_init_ctx_cb};
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::{spawn_editor_state};
pub use crate::src::shared::status::{status_line};
pub use crate::src::shared::tty::{
    tty, tty_code, tty_ctx, tty_ctx_c2rust_unnamed, tty_ctx_c2rust_unnamed_data,
    tty_ctx_c2rust_unnamed_sel, tty_ctx_redraw_cb, tty_ctx_set_client_cb, tty_key,
    tty_style_ctx, tty_term, tty_term_entry,
};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::shared::environment::{environ};
pub use crate::src::shared::format::{FORMAT_NOJOBS, FORMAT_PANE, FORMAT_STATUS, FORMAT_WINDOW};
pub use crate::src::shared::borders::{CELL_BORDERS, CELL_NONE, SIMPLE_BORDERS};
pub use crate::src::shared::pane::{
    PANE_STATUS_OFF, window_pane_offset, window_pane_resize, window_pane_resize_entry,
    window_pane_resizes,
};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_citem, screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::layout::*;
use crate::src::shared::terminal::*;
use crate::src::shared::event::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

unsafe extern "C" fn window_set_fill_cell(
    mut w: *mut window,
    mut inside: ::core::ffi::c_int,
    mut gc: *mut grid_cell,
) {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut s: screen = screen {
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
    let mut new_gc: grid_cell = grid_cell {
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
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    memcpy(
        gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
    utf8_set(
        &raw mut (*gc).data,
        CELL_BORDERS[CELL_NONE as usize] as u_char,
    );
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        (FORMAT_WINDOW | (*w).id) as ::core::ffi::c_int,
        FORMAT_NOJOBS,
    );
    format_defaults(
        ft,
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        (*w).active,
    );
    format_add(
        ft,
        b"is_inside\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        inside,
    );
    format_add(
        ft,
        b"is_outside\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        (inside == 0) as ::core::ffi::c_int,
    );
    value = options_get_string(
        (*w).options,
        b"fill-character\0" as *const u8 as *const ::core::ffi::c_char,
    );
    expanded = format_expand(ft, value);
    format_free(ft);
    screen_init(&raw mut s, 1 as u_int, 1 as u_int, 0 as u_int);
    screen_write_start(&raw mut ctx, &raw mut s);
    format_draw(
        &raw mut ctx,
        &raw const grid_default_cell,
        1 as u_int,
        expanded,
        ::core::ptr::null_mut::<style_ranges>(),
        0 as ::core::ffi::c_int,
    );
    screen_write_stop(&raw mut ctx);
    free(expanded as *mut ::core::ffi::c_void);
    grid_view_get_cell(s.grid, 0 as u_int, 0 as u_int, &raw mut new_gc);
    if new_gc.data.width as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        memcpy(
            gc as *mut ::core::ffi::c_void,
            &raw mut new_gc as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    }
    screen_free(&raw mut s);
}
#[no_mangle]
pub unsafe extern "C" fn window_set_fill_cells(mut w: *mut window) {
    window_set_fill_cell(w, 1 as ::core::ffi::c_int, &raw mut (*w).inside_cell);
    window_set_fill_cell(w, 0 as ::core::ffi::c_int, &raw mut (*w).outside_cell);
}
unsafe extern "C" fn window_copy_fill_cell(mut gc: *mut grid_cell, mut fill: *const grid_cell) {
    utf8_copy(&raw mut (*gc).data, &raw const (*fill).data);
    (*gc).attr = ((*gc).attr as ::core::ffi::c_int | (*fill).attr as ::core::ffi::c_int) as u_short;
    (*gc).flags =
        ((*gc).flags as ::core::ffi::c_int | (*fill).flags as ::core::ffi::c_int) as u_char;
    if (*fill).fg != 8 as ::core::ffi::c_int {
        (*gc).fg = (*fill).fg;
    }
    if (*fill).bg != 8 as ::core::ffi::c_int {
        (*gc).bg = (*fill).bg;
    }
    if (*fill).us != 8 as ::core::ffi::c_int {
        (*gc).us = (*fill).us;
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_get_fill_cell(
    mut w: *mut window,
    mut inside: ::core::ffi::c_int,
    mut gc: *mut grid_cell,
) {
    if inside != 0 {
        window_copy_fill_cell(gc, &raw mut (*w).inside_cell);
    } else {
        window_copy_fill_cell(gc, &raw mut (*w).outside_cell);
    };
}
#[no_mangle]
pub unsafe extern "C" fn window_get_border_cell(
    mut wp: *mut window_pane,
    mut pane_lines: pane_lines,
    mut cell_type: ::core::ffi::c_int,
    mut gc: *mut grid_cell,
) {
    let mut idx: u_int = 0;
    match pane_lines as ::core::ffi::c_uint {
        4 => {
            if cell_type == CELL_NONE {
                (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
                utf8_set(
                    &raw mut (*gc).data,
                    CELL_BORDERS[CELL_NONE as usize] as u_char,
                );
            } else {
                (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
                if !wp.is_null() && window_pane_index(wp, &raw mut idx) == 0 as ::core::ffi::c_int {
                    utf8_set(
                        &raw mut (*gc).data,
                        ('0' as i32 as u_int).wrapping_add(idx.wrapping_rem(10 as u_int)) as u_char,
                    );
                } else {
                    utf8_set(&raw mut (*gc).data, '*' as i32 as u_char);
                }
            }
        }
        1 => {
            (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
            utf8_copy(&raw mut (*gc).data, tty_acs_double_borders(cell_type));
        }
        2 => {
            (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
            utf8_copy(&raw mut (*gc).data, tty_acs_heavy_borders(cell_type));
        }
        7 => {
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
        6 | 5 => {
            (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
            utf8_set(&raw mut (*gc).data, ' ' as i32 as u_char);
        }
        _ => {
            (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
            utf8_set(
                &raw mut (*gc).data,
                CELL_BORDERS[cell_type as usize] as u_char,
            );
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_get_border_cell(
    mut wp: *mut window_pane,
    mut cell_type: ::core::ffi::c_int,
    mut gc: *mut grid_cell,
) {
    let mut pane_lines: pane_lines = window_pane_get_pane_lines(wp);
    window_get_border_cell(wp, pane_lines, cell_type, gc);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_get_border_style(
    mut wp: *mut window_pane,
    mut c: *mut client,
    mut gc: *mut grid_cell,
) {
    let mut s: *mut session = (*c).session;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut option: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut saved: *mut grid_cell = ::core::ptr::null_mut::<grid_cell>();
    let mut flag: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
    if wp == (*(*(*(*c).session).curw).window).active {
        flag = &raw mut (*wp).active_border_gc_set;
        saved = &raw mut (*wp).active_border_gc;
        option = b"pane-active-border-style\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        flag = &raw mut (*wp).border_gc_set;
        saved = &raw mut (*wp).border_gc;
        option = b"pane-border-style\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if *flag == 0 {
        ft = format_create_defaults(::core::ptr::null_mut::<cmdq_item>(), c, s, (*s).curw, wp);
        style_apply(saved, (*wp).options, option, ft);
        format_free(ft);
        *flag = 1 as ::core::ffi::c_int;
    }
    memcpy(
        gc as *mut ::core::ffi::c_void,
        saved as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn window_make_pane_status(
    mut wp: *mut window_pane,
    mut c: *mut client,
    mut width: u_int,
    mut span: *mut redraw_span,
) -> ::core::ffi::c_int {
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
    let mut fmt: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut sle: *mut style_line_entry = &raw mut (*wp).border_status_line;
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
    let mut old: screen = screen {
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
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    let mut pane_lines: pane_lines = PANE_LINES_SINGLE;
    let mut pane_status: ::core::ffi::c_int = 0;
    let mut cell_type: ::core::ffi::c_int = 0;
    pane_status = window_pane_get_pane_status(wp);
    if pane_status == PANE_STATUS_OFF || width == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    ft = format_create(
        c,
        ::core::ptr::null_mut::<cmdq_item>(),
        (FORMAT_PANE | (*wp).id) as ::core::ffi::c_int,
        FORMAT_STATUS,
    );
    format_defaults(ft, c, (*c).session, (*(*c).session).curw, wp);
    fmt = options_get_string(
        (*wp).options,
        b"pane-border-format\0" as *const u8 as *const ::core::ffi::c_char,
    );
    expanded = format_expand_time(ft, fmt);
    memcpy(
        &raw mut old as *mut ::core::ffi::c_void,
        &raw mut (*wp).status_screen as *const ::core::ffi::c_void,
        ::core::mem::size_of::<screen>() as size_t,
    );
    screen_init(&raw mut (*wp).status_screen, width, 1 as u_int, 0 as u_int);
    (*wp).status_screen.mode = 0 as ::core::ffi::c_int;
    screen_write_start(&raw mut ctx, &raw mut (*wp).status_screen);
    window_pane_get_border_style(wp, c, &raw mut gc);
    pane_lines = window_pane_get_pane_lines(wp);
    i = 0 as u_int;
    while i < width {
        cell_type = redraw_get_status_border_cell_type(&raw mut span, i);
        window_get_border_cell(wp, pane_lines, cell_type, &raw mut gc);
        screen_write_cell(&raw mut ctx, &raw mut gc);
        i = i.wrapping_add(1);
    }
    gc.attr = (gc.attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
    screen_write_cursormove(
        &raw mut ctx,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    style_ranges_free(&raw mut (*sle).ranges);
    format_draw(
        &raw mut ctx,
        &raw mut gc,
        width,
        expanded,
        &raw mut (*sle).ranges,
        0 as ::core::ffi::c_int,
    );
    screen_write_stop(&raw mut ctx);
    format_free(ft);
    free((*sle).expanded as *mut ::core::ffi::c_void);
    (*sle).expanded = expanded;
    if grid_compare((*wp).status_screen.grid, old.grid) == 0 as ::core::ffi::c_int {
        screen_free(&raw mut old);
        return 0 as ::core::ffi::c_int;
    }
    screen_free(&raw mut old);
    return 1 as ::core::ffi::c_int;
}
