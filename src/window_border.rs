use crate::src::options::options_owner_ptr;
use crate::src::ffi::libc::memcpy;
use crate::src::format::{
    format_add, format_create, format_create_defaults, format_defaults, format_expand_cstring,
    format_expand_time_cstring, format_free,
};
use crate::src::format_draw::format_draw;
use crate::src::grid::view::grid_view_get_cell;
use crate::src::grid::{grid_compare, grid_default_cell};
use crate::src::options::options_get_string;
use crate::src::screen::{screen_free, screen_init};
use crate::src::screen_redraw::redraw_get_status_border_cell_type;
use crate::src::screen_write::{
    screen_write_cell, screen_write_cursormove, screen_write_start, screen_write_stop,
};
use crate::src::shared::abi::*;
use crate::src::shared::borders::{CELL_BORDERS, CELL_NONE, SIMPLE_BORDERS};
use crate::src::shared::client::client;
use crate::src::shared::command::cmdq_item;
use crate::src::shared::format::format_tree;
use crate::src::shared::format::{FORMAT_NOJOBS, FORMAT_PANE, FORMAT_STATUS, FORMAT_WINDOW};
use crate::src::shared::grid::*;
use crate::src::shared::layout::*;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::PANE_STATUS_OFF;
use crate::src::shared::redraw::redraw_spans;
use crate::src::shared::screen::screen;
use crate::src::shared::screen_write::screen_write_ctx;
use crate::src::shared::session::session;
use crate::src::shared::style::*;
use crate::src::shared::window::{window, winlink};
use crate::src::style::{style_apply, style_ranges_clear};
use crate::src::text::utf8::{utf8_copy, utf8_set};
use crate::src::tty_acs::{tty_acs_double_borders, tty_acs_heavy_borders, tty_acs_rounded_borders};
use crate::src::window::{
    window_pane_get_pane_lines, window_pane_get_pane_status, window_pane_index,
};

unsafe fn window_set_fill_cell(
    w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>,
    mut inside: ::core::ffi::c_int,
    mut gc: *mut grid_cell,
) {
    let mut w = w_owner.get();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut s: screen = screen::empty();
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        target: Default::default(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
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
    memcpy(
        gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
    utf8_set(&mut (*gc).data, CELL_BORDERS[CELL_NONE as usize] as u_char);
    let mut ft_owner = format_create(
        None,
        None,
        (FORMAT_WINDOW | (*w).id) as ::core::ffi::c_int,
        FORMAT_NOJOBS,
    );
    ft = &raw mut *ft_owner;
    format_defaults(
        ft,
        None,
        None,
        (refbox::Weak::new()).clone(),
        ((*w).active_pane().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get())).as_ref().and_then(|model| model.observer.upgrade()).as_ref(),
    );
    format_add(
        ft,
        b"is_inside\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (inside) as i32),
    );
    format_add(
        ft,
        b"is_outside\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", ((inside == 0) as ::core::ffi::c_int) as i32),
    );
    value = options_get_string(
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
        b"fill-character\0" as *const u8 as *const ::core::ffi::c_char,
    );
    let expanded = format_expand_cstring(ft, value);
    format_free(ft_owner);
    screen_init(&mut s, 1 as u_int, 1 as u_int, 0 as u_int);
    screen_write_start(&mut ctx, &raw mut s);
    format_draw(
        &raw mut ctx,
        &raw const grid_default_cell,
        1 as u_int,
        expanded.as_ptr(),
        ::core::ptr::null_mut::<style_ranges>(),
        0 as ::core::ffi::c_int,
    );
    screen_write_stop(&mut ctx);
    grid_view_get_cell(s.grid(), 0 as u_int, 0 as u_int, &mut new_gc);
    if new_gc.data.width as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        memcpy(
            gc as *mut ::core::ffi::c_void,
            &raw mut new_gc as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    }
    screen_free(&mut s);
}
pub unsafe fn window_set_fill_cells(w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>) {
    let mut w = w_owner.get();
    window_set_fill_cell(w_owner, 1 as ::core::ffi::c_int, &raw mut (*w).inside_cell);
    window_set_fill_cell(w_owner, 0 as ::core::ffi::c_int, &raw mut (*w).outside_cell);
}
unsafe fn window_copy_fill_cell(mut gc: *mut grid_cell, mut fill: *const grid_cell) {
    (*gc).data = utf8_copy(&(*fill).data);
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
pub unsafe fn window_get_fill_cell(
    w_value: &window,
    mut inside: ::core::ffi::c_int,
    mut gc: *mut grid_cell,
) {
    let w: *mut window = w_value as *const _ as *mut _;
    if inside != 0 {
        window_copy_fill_cell(gc, &raw mut (*w).inside_cell);
    } else {
        window_copy_fill_cell(gc, &raw mut (*w).outside_cell);
    };
}
pub unsafe fn window_get_border_cell(
    wp_value: Option<&window_pane>,
    mut pane_lines: pane_lines,
    mut cell_type: ::core::ffi::c_int,
    gc: &mut grid_cell,
) {
    let wp: *mut window_pane = wp_value.map_or(std::ptr::null_mut(), |value| value as *const _ as *mut _);
    let mut idx: u_int = 0;
    match pane_lines as ::core::ffi::c_uint {
        4 => {
            if cell_type == CELL_NONE {
                gc.attr = (gc.attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
                utf8_set(&mut gc.data, CELL_BORDERS[CELL_NONE as usize] as u_char);
            } else {
                gc.attr = (gc.attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
                if !wp.is_null() && window_pane_index(&*wp).map(|value| { idx = value; }).is_some() {
                    utf8_set(
                        &mut gc.data,
                        ('0' as i32 as u_int).wrapping_add(idx.wrapping_rem(10 as u_int)) as u_char,
                    );
                } else {
                    utf8_set(&mut gc.data, '*' as i32 as u_char);
                }
            }
        }
        1 => {
            gc.attr = (gc.attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
            gc.data = utf8_copy(tty_acs_double_borders(cell_type));
        }
        2 => {
            gc.attr = (gc.attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
            gc.data = utf8_copy(tty_acs_heavy_borders(cell_type));
        }
        7 => {
            gc.attr = (gc.attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
            gc.data = utf8_copy(tty_acs_rounded_borders(cell_type));
        }
        3 => {
            gc.attr = (gc.attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
            utf8_set(&mut gc.data, SIMPLE_BORDERS[cell_type as usize] as u_char);
        }
        6 | 5 => {
            gc.attr = (gc.attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
            utf8_set(&mut gc.data, ' ' as i32 as u_char);
        }
        _ => {
            gc.attr = (gc.attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
            utf8_set(&mut gc.data, CELL_BORDERS[cell_type as usize] as u_char);
        }
    };
}
pub unsafe fn window_pane_get_border_cell(
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut cell_type: ::core::ffi::c_int,
    gc: &mut grid_cell,
) {
    let mut wp = wp_owner.get();
    let mut pane_lines: pane_lines = window_pane_get_pane_lines(&*wp);
    window_get_border_cell((wp).as_ref(), pane_lines, cell_type, gc);
}
pub unsafe fn window_pane_get_border_style(
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    c_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>,
    mut gc: *mut grid_cell,
) {
    let mut wp = wp_owner.get();
    let mut c = c_owner.get();
    let mut s: *mut session = (*c).session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut option: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut saved: *mut grid_cell = ::core::ptr::null_mut::<grid_cell>();
    let mut flag: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
    if wp == (*((*(*c).session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get())).current_winlink()).get_unchecked().window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get())).active_pane().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()) {
        flag = &raw mut (*wp).active_border_gc_set;
        saved = &raw mut (*wp).active_border_gc;
        option = b"pane-active-border-style\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        flag = &raw mut (*wp).border_gc_set;
        saved = &raw mut (*wp).border_gc;
        option = b"pane-border-style\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if *flag == 0 {
        let mut ft_owner = format_create_defaults(None, Some(c_owner), (s).as_ref().and_then(|model| model.observer.upgrade()).as_ref(), ((*s).current_winlink()).clone(), Some(wp_owner));
        ft = &raw mut *ft_owner;
        style_apply(saved, options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options), option, ft);
        format_free(ft_owner);
        *flag = 1 as ::core::ffi::c_int;
    }
    memcpy(
        gc as *mut ::core::ffi::c_void,
        saved as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
}
pub unsafe fn window_make_pane_status(
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    c_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>,
    mut width: u_int,
    spans: &redraw_spans,
    mut span_index: usize,
) -> ::core::ffi::c_int {
    let mut wp = wp_owner.get();
    let mut c = c_owner.get();
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
        wp: std::rc::Weak::new(),
        target: Default::default(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut old: screen = screen::empty();
    let mut i: u_int = 0;
    let mut pane_lines: pane_lines = PANE_LINES_SINGLE;
    let mut pane_status: ::core::ffi::c_int = 0;
    let mut cell_type: ::core::ffi::c_int = 0;
    pane_status = window_pane_get_pane_status(&*wp);
    if pane_status == PANE_STATUS_OFF || width == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    let mut ft_owner = format_create(
        Some(c_owner),
        None,
        (FORMAT_PANE | (*wp).id) as ::core::ffi::c_int,
        FORMAT_STATUS,
    );
    ft = &raw mut *ft_owner;
    format_defaults(ft, Some(c_owner), ((*c).session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get())).as_ref().and_then(|model| model.observer.upgrade()).as_ref(), ((*(*c).session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get())).current_winlink()).clone(), Some(wp_owner));
    fmt = options_get_string(
        options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options),
        b"pane-border-format\0" as *const u8 as *const ::core::ffi::c_char,
    );
    let expanded = format_expand_time_cstring(ft, fmt);
    old = std::ptr::replace(&raw mut (*wp).status_screen, screen::empty());
    screen_init(&mut (*wp).status_screen, width, 1 as u_int, 0 as u_int);
    (*wp).status_screen.mode = 0 as ::core::ffi::c_int;
    screen_write_start(&mut ctx, &raw mut (*wp).status_screen);
    window_pane_get_border_style(wp_owner, c_owner, &raw mut gc);
    pane_lines = window_pane_get_pane_lines(&*wp);
    i = 0 as u_int;
    while i < width {
        cell_type = redraw_get_status_border_cell_type(spans, &mut span_index, i);
        window_get_border_cell((wp).as_ref(), pane_lines, cell_type, &mut gc);
        screen_write_cell(&mut ctx, &gc);
        i = i.wrapping_add(1);
    }
    gc.attr = (gc.attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
    screen_write_cursormove(
        &mut ctx,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    style_ranges_clear(&raw mut (*sle).ranges);
    format_draw(
        &raw mut ctx,
        &raw mut gc,
        width,
        expanded.as_ptr(),
        &raw mut (*sle).ranges,
        0 as ::core::ffi::c_int,
    );
    screen_write_stop(&mut ctx);
    format_free(ft_owner);
    if grid_compare((*wp).status_screen.grid(), old.grid()) == 0 as ::core::ffi::c_int {
        screen_free(&mut old);
        return 0 as ::core::ffi::c_int;
    }
    screen_free(&mut old);
    return 1 as ::core::ffi::c_int;
}
