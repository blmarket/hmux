use crate::src::ffi::libc::{calloc, free, memcpy, snprintf, strlcat, strlen};
use crate::src::grid::{
    grid_adjust_lines, grid_check_is_clear, grid_clear_lines, grid_create, grid_destroy,
    grid_duplicate_lines, grid_empty_line, grid_reflow, grid_unwrap_position, grid_wrap_position,
};
use crate::src::grid_view::{grid_view_clear, grid_view_delete_lines};
use crate::src::hyperlinks::{hyperlinks_free, hyperlinks_init, hyperlinks_reset};
use crate::src::log::{fatal, fatalx, log_debug};
use crate::src::options::options_get_number;
use crate::src::screen_write::{screen_write_free_list, screen_write_make_list};
use crate::src::shared::abi::*;
pub use crate::src::shared::arguments::args;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
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
pub use crate::src::shared::key::MODEKEY_EMACS;
use crate::src::shared::key::*;
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::limits::{__INT_MAX__, UINT_MAX};
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::options;
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{
    screen, screen_sel, screen_title_entry, screen_title_link, screen_titles, ALL_MODES,
    EXTENDED_KEY_MODES, MODE_BRACKETPASTE, MODE_CRLF, MODE_CURSOR, MODE_CURSOR_BLINKING,
    MODE_CURSOR_BLINKING_SET, MODE_CURSOR_VERY_VISIBLE, MODE_FOCUSON, MODE_INSERT, MODE_KCURSOR,
    MODE_KEYS_EXTENDED, MODE_KEYS_EXTENDED_2, MODE_KKEYPAD, MODE_MOUSE_ALL, MODE_MOUSE_BUTTON,
    MODE_MOUSE_SGR, MODE_MOUSE_STANDARD, MODE_MOUSE_UTF8, MODE_ORIGIN, MODE_SYNC,
    MODE_THEME_UPDATES, MODE_WRAP,
};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::style::style_apply;
use crate::src::tmux::{clean_name, global_options};
use crate::src::tty_acs::tty_acs_get;
use crate::src::utf8::{utf8_copy, utf8_to_data};
use crate::src::xmalloc::xstrdup;

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

unsafe extern "C" fn screen_free_titles(mut s: *mut screen) {
    let mut title_entry: *mut screen_title_entry = ::core::ptr::null_mut::<screen_title_entry>();
    if (*s).titles.is_null() {
        return;
    }
    loop {
        title_entry = (*(*s).titles).tqh_first;
        if title_entry.is_null() {
            break;
        }
        if !(*title_entry).entry.tqe_next.is_null() {
            (*(*title_entry).entry.tqe_next).entry.tqe_prev = (*title_entry).entry.tqe_prev;
        } else {
            (*(*s).titles).tqh_last = (*title_entry).entry.tqe_prev;
        }
        *(*title_entry).entry.tqe_prev = (*title_entry).entry.tqe_next;
        free((*title_entry).text as *mut ::core::ffi::c_void);
        drop(Box::from_raw(title_entry));
    }
    drop(Box::from_raw((*s).titles));
    (*s).titles = ::core::ptr::null_mut::<screen_titles>();
    (*s).ntitles = 0 as u_int;
}
#[no_mangle]
pub unsafe extern "C" fn screen_init(
    mut s: *mut screen,
    mut sx: u_int,
    mut sy: u_int,
    mut hlimit: u_int,
) {
    (*s).grid = grid_create(sx, sy, hlimit);
    (*s).saved_grid = ::core::ptr::null_mut::<grid>();
    (*s).title = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    (*s).titles = ::core::ptr::null_mut::<screen_titles>();
    (*s).ntitles = 0 as u_int;
    (*s).path = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*s).cstyle = SCREEN_CURSOR_DEFAULT;
    (*s).default_cstyle = SCREEN_CURSOR_DEFAULT;
    (*s).mode = MODE_CURSOR;
    (*s).default_mode = 0 as ::core::ffi::c_int;
    (*s).ccolour = -(1 as ::core::ffi::c_int);
    (*s).default_ccolour = -(1 as ::core::ffi::c_int);
    (*s).tabs = ::core::ptr::null_mut::<bitstr_t>();
    std::ptr::write(&raw mut (*s).sel, None);
    (*s).write_list = ::core::ptr::null_mut::<screen_write_cline>();
    (*s).hyperlinks = ::core::ptr::null_mut::<hyperlinks>();
    screen_reinit(s, 1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_reinit(mut s: *mut screen, mut check: ::core::ffi::c_int) {
    (*s).cx = 0 as u_int;
    (*s).cy = 0 as u_int;
    (*s).rupper = 0 as u_int;
    (*s).rlower = (*(*s).grid).sy.wrapping_sub(1 as u_int);
    (*s).mode = MODE_CURSOR | MODE_WRAP | (*s).mode & MODE_CRLF;
    if options_get_number(
        global_options,
        b"extended-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 2 as ::core::ffi::c_longlong
    {
        (*s).mode = (*s).mode & !EXTENDED_KEY_MODES | MODE_KEYS_EXTENDED;
    }
    if !(*s).saved_grid.is_null() {
        screen_alternate_off(
            s,
            ::core::ptr::null_mut::<grid_cell>(),
            0 as ::core::ffi::c_int,
        );
    }
    (*s).saved_cx = UINT_MAX as u_int;
    (*s).saved_cy = UINT_MAX as u_int;
    screen_reset_tabs(s);
    if check != 0 {
        grid_check_is_clear((*s).grid);
    }
    grid_clear_lines((*s).grid, (*(*s).grid).hsize, (*(*s).grid).sy, 8 as u_int);
    screen_clear_selection(s);
    screen_free_titles(s);
    screen_set_progress_bar(s, PROGRESS_BAR_HIDDEN, 0 as ::core::ffi::c_int);
    screen_reset_hyperlinks(s);
}
#[no_mangle]
pub unsafe extern "C" fn screen_reset_hyperlinks(mut s: *mut screen) {
    if (*s).hyperlinks.is_null() {
        (*s).hyperlinks = hyperlinks_init();
    } else {
        hyperlinks_reset((*s).hyperlinks);
    };
}
#[no_mangle]
pub unsafe extern "C" fn screen_free(mut s: *mut screen) {
    drop((*s).sel.take());
    free((*s).tabs as *mut ::core::ffi::c_void);
    free((*s).path as *mut ::core::ffi::c_void);
    free((*s).title as *mut ::core::ffi::c_void);
    (*s).tabs = std::ptr::null_mut();
    (*s).path = std::ptr::null_mut();
    (*s).title = std::ptr::null_mut();
    if !(*s).write_list.is_null() {
        screen_write_free_list(s);
    }
    if !(*s).saved_grid.is_null() {
        grid_destroy((*s).saved_grid);
    }
    grid_destroy((*s).grid);
    if !(*s).hyperlinks.is_null() {
        hyperlinks_free((*s).hyperlinks);
    }
    screen_free_titles(s);
    (*s).grid = std::ptr::null_mut();
    (*s).saved_grid = std::ptr::null_mut();
    (*s).hyperlinks = std::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn screen_reset_tabs(mut s: *mut screen) {
    let mut i: u_int = 0;
    free((*s).tabs as *mut ::core::ffi::c_void);
    (*s).tabs = calloc(
        ((*(*s).grid).sx.wrapping_add(7 as u_int) >> 3 as ::core::ffi::c_int) as size_t,
        ::core::mem::size_of::<bitstr_t>() as size_t,
    ) as *mut bitstr_t;
    if (*s).tabs.is_null() {
        fatal(b"bit_alloc failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    i = 8 as u_int;
    while i < (*(*s).grid).sx {
        let ref mut fresh0 = *(*s).tabs.offset((i >> 3 as ::core::ffi::c_int) as isize);
        *fresh0 = (*fresh0 as ::core::ffi::c_int | (1 as ::core::ffi::c_int) << (i & 0x7 as u_int))
            as bitstr_t;
        i = i.wrapping_add(8 as u_int);
    }
}
pub(crate) unsafe fn screen_share_hyperlinks(dst: *mut screen, src: *const screen) {
    let shared = if (*src).hyperlinks.is_null() {
        std::ptr::null_mut()
    } else {
        crate::src::hyperlinks::hyperlinks_copy((*src).hyperlinks)
    };
    if !(*dst).hyperlinks.is_null() {
        crate::src::hyperlinks::hyperlinks_free((*dst).hyperlinks);
    }
    (*dst).hyperlinks = shared;
}
pub(crate) unsafe fn screen_has_tab(s: *const screen, column: u_int) -> bool {
    *(*s).tabs.add(column as usize / 8) & (1 << (column % 8)) != 0
}
pub(crate) unsafe fn screen_set_tab(s: *mut screen, column: u_int, set: bool) {
    let byte = &mut *(*s).tabs.add(column as usize / 8);
    let mask = 1 << (column % 8);
    if set {
        *byte |= mask;
    } else {
        *byte &= !mask;
    }
}
pub(crate) unsafe fn screen_clear_tabs(s: *mut screen) {
    std::ptr::write_bytes((*s).tabs, 0, ((*(*s).grid).sx as usize).div_ceil(8));
}
#[no_mangle]
pub unsafe extern "C" fn screen_set_default_cursor(mut s: *mut screen, mut oo: *mut options) {
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
    let mut c: ::core::ffi::c_int = 0;
    style_apply(
        &raw mut gc,
        oo,
        b"cursor-colour\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    (*s).default_ccolour = gc.fg;
    c = options_get_number(
        oo,
        b"cursor-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    (*s).default_mode = 0 as ::core::ffi::c_int;
    screen_set_cursor_style(
        c as u_int,
        &raw mut (*s).default_cstyle,
        &raw mut (*s).default_mode,
    );
}
#[no_mangle]
pub unsafe extern "C" fn screen_set_cursor_style(
    mut style: u_int,
    mut cstyle: *mut screen_cursor_style,
    mut mode: *mut ::core::ffi::c_int,
) {
    match style {
        0 => {
            *cstyle = SCREEN_CURSOR_DEFAULT;
        }
        1 => {
            *cstyle = SCREEN_CURSOR_BLOCK;
            *mode |= MODE_CURSOR_BLINKING;
        }
        2 => {
            *cstyle = SCREEN_CURSOR_BLOCK;
            *mode &= !MODE_CURSOR_BLINKING;
        }
        3 => {
            *cstyle = SCREEN_CURSOR_UNDERLINE;
            *mode |= MODE_CURSOR_BLINKING;
        }
        4 => {
            *cstyle = SCREEN_CURSOR_UNDERLINE;
            *mode &= !MODE_CURSOR_BLINKING;
        }
        5 => {
            *cstyle = SCREEN_CURSOR_BAR;
            *mode |= MODE_CURSOR_BLINKING;
        }
        6 => {
            *cstyle = SCREEN_CURSOR_BAR;
            *mode &= !MODE_CURSOR_BLINKING;
        }
        _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn screen_set_cursor_colour(
    mut s: *mut screen,
    mut colour: ::core::ffi::c_int,
) {
    (*s).ccolour = colour;
}
#[no_mangle]
pub unsafe extern "C" fn screen_set_title(
    mut s: *mut screen,
    mut title: *const ::core::ffi::c_char,
    mut untrusted: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut new_title: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    new_title = clean_name(title, untrusted);
    if new_title.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    free((*s).title as *mut ::core::ffi::c_void);
    (*s).title = new_title;
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn screen_set_path(
    mut s: *mut screen,
    mut path: *const ::core::ffi::c_char,
    mut untrusted: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut new_path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    new_path = clean_name(path, untrusted);
    if new_path.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    free((*s).path as *mut ::core::ffi::c_void);
    (*s).path = new_path;
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn screen_push_title(mut s: *mut screen) {
    let mut title_entry: *mut screen_title_entry = ::core::ptr::null_mut::<screen_title_entry>();
    log_debug(
        b"%s: %u\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_push_title\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).ntitles,
    );
    while (*s).ntitles >= 10 as u_int {
        title_entry = *(*((*(*s).titles).tqh_last as *mut screen_titles)).tqh_last;
        free((*title_entry).text as *mut ::core::ffi::c_void);
        if !(*title_entry).entry.tqe_next.is_null() {
            (*(*title_entry).entry.tqe_next).entry.tqe_prev = (*title_entry).entry.tqe_prev;
        } else {
            (*(*s).titles).tqh_last = (*title_entry).entry.tqe_prev;
        }
        *(*title_entry).entry.tqe_prev = (*title_entry).entry.tqe_next;
        drop(Box::from_raw(title_entry));
        (*s).ntitles = (*s).ntitles.wrapping_sub(1);
    }
    if (*s).titles.is_null() {
        (*s).titles = Box::into_raw(Box::new(::core::mem::zeroed::<screen_titles>()));
        (*(*s).titles).tqh_first = ::core::ptr::null_mut::<screen_title_entry>();
        (*(*s).titles).tqh_last = &raw mut (*(*s).titles).tqh_first;
    }
    title_entry = Box::into_raw(Box::new(screen_title_entry {
        text: xstrdup((*s).title),
        entry: screen_title_link {
            tqe_next: ::core::ptr::null_mut(),
            tqe_prev: ::core::ptr::null_mut(),
        },
    }));
    (*title_entry).entry.tqe_next = (*(*s).titles).tqh_first;
    if !(*title_entry).entry.tqe_next.is_null() {
        (*(*(*s).titles).tqh_first).entry.tqe_prev = &raw mut (*title_entry).entry.tqe_next;
    } else {
        (*(*s).titles).tqh_last = &raw mut (*title_entry).entry.tqe_next;
    }
    (*(*s).titles).tqh_first = title_entry;
    (*title_entry).entry.tqe_prev = &raw mut (*(*s).titles).tqh_first;
    (*s).ntitles = (*s).ntitles.wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn screen_pop_title(mut s: *mut screen) {
    let mut title_entry: *mut screen_title_entry = ::core::ptr::null_mut::<screen_title_entry>();
    if (*s).titles.is_null() {
        return;
    }
    log_debug(
        b"%s: %u\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_pop_title\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).ntitles,
    );
    title_entry = (*(*s).titles).tqh_first;
    if !title_entry.is_null() {
        free((*s).title as *mut ::core::ffi::c_void);
        (*s).title = (*title_entry).text;
        if !(*title_entry).entry.tqe_next.is_null() {
            (*(*title_entry).entry.tqe_next).entry.tqe_prev = (*title_entry).entry.tqe_prev;
        } else {
            (*(*s).titles).tqh_last = (*title_entry).entry.tqe_prev;
        }
        *(*title_entry).entry.tqe_prev = (*title_entry).entry.tqe_next;
        drop(Box::from_raw(title_entry));
        (*s).ntitles = (*s).ntitles.wrapping_sub(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_set_progress_bar(
    mut s: *mut screen,
    mut pbs: progress_bar_state,
    mut p: ::core::ffi::c_int,
) {
    (*s).progress_bar.state = pbs;
    if p >= 0 as ::core::ffi::c_int
        && pbs as ::core::ffi::c_uint
            != PROGRESS_BAR_INDETERMINATE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*s).progress_bar.progress = p;
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_resize_cursor(
    mut s: *mut screen,
    mut sx: u_int,
    mut sy: u_int,
    mut reflow: ::core::ffi::c_int,
    mut eat_empty: ::core::ffi::c_int,
    mut cursor: ::core::ffi::c_int,
) {
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*(*s).grid).hsize.wrapping_add((*s).cy);
    let had_write_list = !(*s).write_list.is_null();
    if had_write_list {
        screen_write_free_list(s);
    }
    log_debug(
        b"%s: new size %ux%u, now %ux%u (cursor %u,%u = %u,%u)\0" as *const u8
            as *const ::core::ffi::c_char,
        b"screen_resize_cursor\0" as *const u8 as *const ::core::ffi::c_char,
        sx,
        sy,
        (*(*s).grid).sx,
        (*(*s).grid).sy,
        (*s).cx,
        (*s).cy,
        cx,
        cy,
    );
    if sx < 1 as u_int {
        sx = 1 as u_int;
    }
    if sy < 1 as u_int {
        sy = 1 as u_int;
    }
    if sx != (*(*s).grid).sx {
        (*(*s).grid).sx = sx;
        screen_reset_tabs(s);
    } else {
        reflow = 0 as ::core::ffi::c_int;
    }
    if sy != (*(*s).grid).sy {
        screen_resize_y(s, sy, eat_empty, &raw mut cy);
    }
    if reflow != 0 {
        screen_reflow(s, sx, &raw mut cx, &raw mut cy, cursor);
    }
    if cy >= (*(*s).grid).hsize {
        (*s).cx = cx;
        (*s).cy = cy.wrapping_sub((*(*s).grid).hsize);
    } else {
        (*s).cx = 0 as u_int;
        (*s).cy = 0 as u_int;
    }
    log_debug(
        b"%s: cursor finished at %u,%u = %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_resize_cursor\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).cx,
        (*s).cy,
        cx,
        cy,
    );
    if had_write_list {
        screen_write_make_list(s);
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_resize(
    mut s: *mut screen,
    mut sx: u_int,
    mut sy: u_int,
    mut reflow: ::core::ffi::c_int,
) {
    screen_resize_cursor(
        s,
        sx,
        sy,
        reflow,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
}
unsafe extern "C" fn screen_resize_y(
    mut s: *mut screen,
    mut sy: u_int,
    mut eat_empty: ::core::ffi::c_int,
    mut cy: *mut u_int,
) {
    let mut gd: *mut grid = (*s).grid;
    let mut needed: u_int = 0;
    let mut available: u_int = 0;
    let mut oldy: u_int = 0;
    let mut i: u_int = 0;
    if sy == 0 as u_int {
        fatalx(b"zero size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    oldy = (*(*s).grid).sy;
    if sy < oldy {
        needed = oldy.wrapping_sub(sy);
        if eat_empty != 0 {
            available = oldy.wrapping_sub(1 as u_int).wrapping_sub((*s).cy);
            if available > 0 as u_int {
                if available > needed {
                    available = needed;
                }
                grid_view_delete_lines(gd, oldy.wrapping_sub(available), available, 8 as u_int);
            }
            needed = needed.wrapping_sub(available);
        }
        available = (*s).cy;
        if (*gd).flags & GRID_HISTORY != 0 {
            (*gd).hscrolled = (*gd).hscrolled.wrapping_add(needed);
            (*gd).hsize = (*gd).hsize.wrapping_add(needed);
        } else if needed > 0 as u_int && available > 0 as u_int {
            if available > needed {
                available = needed;
            }
            grid_view_delete_lines(gd, 0 as u_int, available, 8 as u_int);
            *cy = (*cy).wrapping_sub(available);
        }
    }
    grid_adjust_lines(gd, (*gd).hsize.wrapping_add(sy));
    if sy > oldy {
        needed = sy.wrapping_sub(oldy);
        available = (*gd).hscrolled;
        if (*gd).flags & GRID_HISTORY != 0 && available > 0 as u_int {
            if available > needed {
                available = needed;
            }
            (*gd).hscrolled = (*gd).hscrolled.wrapping_sub(available);
            (*gd).hsize = (*gd).hsize.wrapping_sub(available);
        } else {
            available = 0 as u_int;
        }
        needed = needed.wrapping_sub(available);
        i = (*gd).hsize.wrapping_add(sy).wrapping_sub(needed);
        while i < (*gd).hsize.wrapping_add(sy) {
            grid_empty_line(gd, i, 8 as u_int);
            i = i.wrapping_add(1);
        }
    }
    (*gd).sy = sy;
    (*s).rupper = 0 as u_int;
    (*s).rlower = (*(*s).grid).sy.wrapping_sub(1 as u_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_set_selection(
    mut s: *mut screen,
    mut sx: u_int,
    mut sy: u_int,
    mut ex: u_int,
    mut ey: u_int,
    mut rectangle: u_int,
    mut clipx: u_int,
    mut modekeys: ::core::ffi::c_int,
    mut gc: *mut grid_cell,
) {
    let selection = screen_sel {
        hidden: 0,
        rectangle: rectangle as ::core::ffi::c_int,
        modekeys,
        sx,
        sy,
        ex,
        ey,
        clipx,
        cell: *gc,
    };
    if let Some(existing) = (*s).sel.as_mut() {
        **existing = selection;
    } else {
        (*s).sel = Some(Box::new(selection));
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_clear_selection(mut s: *mut screen) {
    drop((*s).sel.take());
}
#[no_mangle]
pub unsafe extern "C" fn screen_hide_selection(mut s: *mut screen) {
    if let Some(selection) = (*s).sel.as_mut() {
        selection.hidden = 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_check_selection(
    mut s: *mut screen,
    mut px: u_int,
    mut py: u_int,
) -> ::core::ffi::c_int {
    let Some(sel) = (*s).sel.as_ref() else {
        return 0;
    };
    let mut xx: u_int = 0;
    if sel.hidden != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if px < (*sel).clipx {
        return 0 as ::core::ffi::c_int;
    }
    if (*sel).rectangle != 0 {
        if (*sel).sy < (*sel).ey {
            if py < (*sel).sy || py > (*sel).ey {
                return 0 as ::core::ffi::c_int;
            }
        } else if (*sel).sy > (*sel).ey {
            if py > (*sel).sy || py < (*sel).ey {
                return 0 as ::core::ffi::c_int;
            }
        } else if py != (*sel).sy {
            return 0 as ::core::ffi::c_int;
        }
        if (*sel).ex < (*sel).sx {
            if px < (*sel).ex {
                return 0 as ::core::ffi::c_int;
            }
            if px > (*sel).sx {
                return 0 as ::core::ffi::c_int;
            }
        } else {
            if px < (*sel).sx {
                return 0 as ::core::ffi::c_int;
            }
            if px > (*sel).ex {
                return 0 as ::core::ffi::c_int;
            }
        }
    } else if (*sel).sy < (*sel).ey {
        if py < (*sel).sy || py > (*sel).ey {
            return 0 as ::core::ffi::c_int;
        }
        if py == (*sel).sy && px < (*sel).sx {
            return 0 as ::core::ffi::c_int;
        }
        if (*sel).modekeys == MODEKEY_EMACS {
            xx = if (*sel).ex == 0 as u_int {
                0 as u_int
            } else {
                (*sel).ex.wrapping_sub(1 as u_int)
            };
        } else {
            xx = (*sel).ex;
        }
        if py == (*sel).ey && px > xx {
            return 0 as ::core::ffi::c_int;
        }
    } else if (*sel).sy > (*sel).ey {
        if py > (*sel).sy || py < (*sel).ey {
            return 0 as ::core::ffi::c_int;
        }
        if py == (*sel).ey && px < (*sel).ex {
            return 0 as ::core::ffi::c_int;
        }
        if (*sel).modekeys == MODEKEY_EMACS {
            xx = (*sel).sx.wrapping_sub(1 as u_int);
        } else {
            xx = (*sel).sx;
        }
        if py == (*sel).sy && ((*sel).sx == 0 as u_int || px > xx) {
            return 0 as ::core::ffi::c_int;
        }
    } else {
        if py != (*sel).sy {
            return 0 as ::core::ffi::c_int;
        }
        if (*sel).ex < (*sel).sx {
            if (*sel).modekeys == MODEKEY_EMACS {
                xx = (*sel).sx.wrapping_sub(1 as u_int);
            } else {
                xx = (*sel).sx;
            }
            if px > xx || px < (*sel).ex {
                return 0 as ::core::ffi::c_int;
            }
        } else {
            if (*sel).modekeys == MODEKEY_EMACS {
                xx = if (*sel).ex == 0 as u_int {
                    0 as u_int
                } else {
                    (*sel).ex.wrapping_sub(1 as u_int)
                };
            } else {
                xx = (*sel).ex;
            }
            if px < (*sel).sx || px > xx {
                return 0 as ::core::ffi::c_int;
            }
        }
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn screen_select_cell(
    mut s: *mut screen,
    mut dst: *mut grid_cell,
    mut src: *const grid_cell,
) -> ::core::ffi::c_int {
    let Some(selection) = (*s).sel.as_ref() else {
        return 0;
    };
    if selection.hidden != 0 {
        return 0 as ::core::ffi::c_int;
    }
    memcpy(
        dst as *mut ::core::ffi::c_void,
        &raw const selection.cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    if (*dst).fg == 8 as ::core::ffi::c_int || (*dst).fg == 9 as ::core::ffi::c_int {
        (*dst).fg = (*src).fg;
    }
    if (*dst).bg == 8 as ::core::ffi::c_int || (*dst).bg == 9 as ::core::ffi::c_int {
        (*dst).bg = (*src).bg;
    }
    utf8_copy(&raw mut (*dst).data, &raw const (*src).data);
    (*dst).flags = (*src).flags;
    if (*dst).attr as ::core::ffi::c_int & GRID_ATTR_NOATTR != 0 {
        (*dst).attr = ((*dst).attr as ::core::ffi::c_int
            | (*src).attr as ::core::ffi::c_int & GRID_ATTR_CHARSET)
            as u_short;
    } else {
        (*dst).attr =
            ((*dst).attr as ::core::ffi::c_int | (*src).attr as ::core::ffi::c_int) as u_short;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn screen_reflow(
    mut s: *mut screen,
    mut new_x: u_int,
    mut cx: *mut u_int,
    mut cy: *mut u_int,
    mut cursor: ::core::ffi::c_int,
) {
    let mut wx: u_int = 0;
    let mut wy: u_int = 0;
    if cursor != 0 {
        grid_wrap_position((*s).grid, *cx, *cy, &raw mut wx, &raw mut wy);
        log_debug(
            b"%s: cursor %u,%u is %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_reflow\0" as *const u8 as *const ::core::ffi::c_char,
            *cx,
            *cy,
            wx,
            wy,
        );
    }
    grid_reflow((*s).grid, new_x);
    if cursor != 0 {
        grid_unwrap_position((*s).grid, cx, cy, wx, wy);
        log_debug(
            b"%s: new cursor is %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_reflow\0" as *const u8 as *const ::core::ffi::c_char,
            *cx,
            *cy,
        );
    } else {
        *cx = 0 as u_int;
        *cy = (*(*s).grid).hsize;
    };
}
#[no_mangle]
pub unsafe extern "C" fn screen_alternate_on(
    mut s: *mut screen,
    mut gc: *mut grid_cell,
    mut cursor: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if !(*s).saved_grid.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    sx = (*(*s).grid).sx;
    sy = (*(*s).grid).sy;
    (*s).saved_grid = grid_create(sx, sy, 0);
    grid_duplicate_lines(
        (*s).saved_grid,
        0 as u_int,
        (*s).grid,
        (*(*s).grid).hsize,
        sy,
    );
    if cursor != 0 {
        (*s).saved_cx = (*s).cx;
        (*s).saved_cy = (*s).cy;
    }
    memcpy(
        &raw mut (*s).saved_cell as *mut ::core::ffi::c_void,
        gc as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    grid_view_clear((*s).grid, 0 as u_int, 0 as u_int, sx, sy, 8 as u_int);
    (*s).saved_flags = (*(*s).grid).flags;
    (*(*s).grid).flags &= !GRID_HISTORY;
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn screen_alternate_off(
    mut s: *mut screen,
    mut gc: *mut grid_cell,
    mut cursor: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut sx: u_int = (*(*s).grid).sx;
    let mut sy: u_int = (*(*s).grid).sy;
    if !(*s).saved_grid.is_null() {
        screen_resize(
            s,
            (*(*s).saved_grid).sx,
            (*(*s).saved_grid).sy,
            0 as ::core::ffi::c_int,
        );
    }
    if cursor != 0 && (*s).saved_cx != UINT_MAX && (*s).saved_cy != UINT_MAX {
        (*s).cx = (*s).saved_cx;
        (*s).cy = (*s).saved_cy;
        if !gc.is_null() {
            memcpy(
                gc as *mut ::core::ffi::c_void,
                &raw mut (*s).saved_cell as *const ::core::ffi::c_void,
                ::core::mem::size_of::<grid_cell>() as size_t,
            );
        }
    }
    if (*s).saved_grid.is_null() {
        if (*s).cx > (*(*s).grid).sx.wrapping_sub(1 as u_int) {
            (*s).cx = (*(*s).grid).sx.wrapping_sub(1 as u_int);
        }
        if (*s).cy > (*(*s).grid).sy.wrapping_sub(1 as u_int) {
            (*s).cy = (*(*s).grid).sy.wrapping_sub(1 as u_int);
        }
        return 0 as ::core::ffi::c_int;
    }
    grid_duplicate_lines(
        (*s).grid,
        (*(*s).grid).hsize,
        (*s).saved_grid,
        0 as u_int,
        (*(*s).saved_grid).sy,
    );
    if (*s).saved_flags & GRID_HISTORY != 0 {
        (*(*s).grid).flags |= GRID_HISTORY;
    }
    screen_resize(s, sx, sy, 1 as ::core::ffi::c_int);
    grid_destroy((*s).saved_grid);
    (*s).saved_grid = ::core::ptr::null_mut::<grid>();
    if (*s).cx > (*(*s).grid).sx.wrapping_sub(1 as u_int) {
        (*s).cx = (*(*s).grid).sx.wrapping_sub(1 as u_int);
    }
    if (*s).cy > (*(*s).grid).sy.wrapping_sub(1 as u_int) {
        (*s).cy = (*(*s).grid).sy.wrapping_sub(1 as u_int);
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn screen_mode_to_string(
    mut mode: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    static mut tmp: [::core::ffi::c_char; 1024] = [0; 1024];
    if mode == 0 as ::core::ffi::c_int {
        return b"NONE\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if mode == ALL_MODES {
        return b"ALL\0" as *const u8 as *const ::core::ffi::c_char;
    }
    *(&raw mut tmp as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    if mode & MODE_CURSOR != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"CURSOR,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_INSERT != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"INSERT,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_KCURSOR != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"KCURSOR,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_KKEYPAD != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"KKEYPAD,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_WRAP != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"WRAP,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_MOUSE_STANDARD != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"MOUSE_STANDARD,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_MOUSE_BUTTON != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"MOUSE_BUTTON,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_CURSOR_BLINKING != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"CURSOR_BLINKING,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_CURSOR_VERY_VISIBLE != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"CURSOR_VERY_VISIBLE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_CURSOR_BLINKING_SET != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"CURSOR_BLINKING_SET,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_MOUSE_UTF8 != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"MOUSE_UTF8,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_MOUSE_SGR != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"MOUSE_SGR,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_BRACKETPASTE != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"BRACKETPASTE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_FOCUSON != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"FOCUSON,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_MOUSE_ALL != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"MOUSE_ALL,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_ORIGIN != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"ORIGIN,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_CRLF != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"CRLF,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_KEYS_EXTENDED != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"KEYS_EXTENDED,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_KEYS_EXTENDED_2 != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"KEYS_EXTENDED_2,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_THEME_UPDATES != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"THEME_UPDATES,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_SYNC != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"SYNC,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if *(&raw mut tmp as *mut ::core::ffi::c_char) as ::core::ffi::c_int != '\0' as i32 {
        tmp[strlen(&raw mut tmp as *mut ::core::ffi::c_char).wrapping_sub(1 as size_t) as usize] =
            '\0' as i32 as ::core::ffi::c_char;
    }
    return &raw mut tmp as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn screen_print(
    mut s: *mut screen,
    mut line: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    // The exported result remains valid until the next call, as before.
    static mut PRINT_BUFFER: [::core::ffi::c_char; 16384] = [0; 16384];
    let buf = (&raw mut PRINT_BUFFER).cast::<::core::ffi::c_char>();
    let len: size_t = 16384;
    let mut acs: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut last: size_t = 0 as size_t;
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut gce: *mut grid_cell_entry = ::core::ptr::null_mut::<grid_cell_entry>();
    y = 0 as u_int;
    's_28: while y < (*(*s).grid).hsize.wrapping_add((*(*s).grid).sy) {
        if !(line >= 0 as ::core::ffi::c_int && y != line as u_int) {
            n = snprintf(
                buf.offset(last as isize),
                len.wrapping_sub(last),
                b"%.4d \"\0" as *const u8 as *const ::core::ffi::c_char,
                y,
            );
            if n <= 0 as ::core::ffi::c_int || n as u_int as size_t >= len.wrapping_sub(last) {
                break;
            }
            last = last.wrapping_add(n as size_t);
            gl = (*(*s).grid).linedata.as_mut_ptr().offset(y as isize) as *mut grid_line;
            x = 0 as u_int;
            while x < (*gl).cellused as u_int {
                gce = (*gl).celldata.as_mut_ptr().offset(x as isize) as *mut grid_cell_entry;
                if !((*gce).flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0) {
                    if !((*gce).flags as ::core::ffi::c_int) & GRID_FLAG_EXTENDED != 0 {
                        if last.wrapping_add(2 as size_t) >= len {
                            break 's_28;
                        }
                        let fresh1 = last;
                        last = last.wrapping_add(1);
                        *buf.offset(fresh1 as isize) =
                            (*gce).c2rust_unnamed.data.data as ::core::ffi::c_char;
                    } else if (*gce).flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                        if last.wrapping_add(2 as size_t) >= len {
                            break 's_28;
                        }
                        let fresh2 = last;
                        last = last.wrapping_add(1);
                        *buf.offset(fresh2 as isize) = '\t' as i32 as ::core::ffi::c_char;
                    } else if (*gce).flags as ::core::ffi::c_int & GRID_ATTR_CHARSET != 0 {
                        acs = tty_acs_get(
                            ::core::ptr::null_mut::<tty>(),
                            (*gce).c2rust_unnamed.data.data,
                        );
                        if !acs.is_null() {
                            n = strlen(acs) as ::core::ffi::c_int;
                        } else {
                            acs = &raw mut (*gce).c2rust_unnamed.data.data
                                as *const ::core::ffi::c_char;
                            n = 1 as ::core::ffi::c_int;
                        }
                        if last.wrapping_add(n as size_t).wrapping_add(1 as size_t) >= len {
                            break 's_28;
                        }
                        memcpy(
                            buf.offset(last as isize) as *mut ::core::ffi::c_void,
                            acs as *const ::core::ffi::c_void,
                            n as size_t,
                        );
                        last = last.wrapping_add(n as size_t);
                    } else {
                        utf8_to_data(
                            (*(*gl)
                                .extddata
                                .as_mut_ptr()
                                .offset((*gce).c2rust_unnamed.offset as isize))
                            .data,
                            &raw mut ud,
                        );
                        if ud.size as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
                            if last
                                .wrapping_add(ud.size as size_t)
                                .wrapping_add(1 as size_t)
                                >= len
                            {
                                break 's_28;
                            }
                            memcpy(
                                buf.offset(last as isize) as *mut ::core::ffi::c_void,
                                &raw mut ud.data as *mut u_char as *const ::core::ffi::c_void,
                                ud.size as size_t,
                            );
                            last = last.wrapping_add(ud.size as size_t);
                        }
                    }
                }
                x = x.wrapping_add(1);
            }
            if last.wrapping_add(3 as size_t) >= len {
                break;
            }
            let fresh3 = last;
            last = last.wrapping_add(1);
            *buf.offset(fresh3 as isize) = '"' as i32 as ::core::ffi::c_char;
            let fresh4 = last;
            last = last.wrapping_add(1);
            *buf.offset(fresh4 as isize) = '\n' as i32 as ::core::ffi::c_char;
        }
        y = y.wrapping_add(1);
    }
    *buf.offset(last as isize) = '\0' as i32 as ::core::ffi::c_char;
    return buf;
}

#[cfg(test)]
mod text_owner_tests {
    use super::*;
    use crate::src::options::{options_create, options_default, options_free};
    use crate::src::options_table::options_table;
    use std::ffi::CStr;

    #[test]
    fn screen_text_survives_moves_alias_updates_reset_and_reinitialization() {
        unsafe {
            let saved_options = global_options;
            let options = options_create(std::ptr::null_mut());
            let extended_keys = (*(&raw const options_table))
                .iter()
                .find(|entry| {
                    !entry.name.is_null() && CStr::from_ptr(entry.name) == c"extended-keys"
                })
                .unwrap();
            options_default(options, extended_keys);
            global_options = options;
            let mut current: screen = std::mem::zeroed();
            screen_init(&raw mut current, 10, 2, 0);
            assert_eq!(CStr::from_ptr(current.title), c"");
            assert!(current.path.is_null());
            assert_eq!(
                screen_set_title(&raw mut current, c"original".as_ptr(), 0),
                1
            );
            screen_push_title(&raw mut current);
            assert_eq!(screen_set_title(&raw mut current, current.title, 0), 1);
            assert_eq!(
                screen_set_path(&raw mut current, c"/tmp/path".as_ptr(), 0),
                1
            );
            assert_eq!(screen_set_path(&raw mut current, current.path, 0), 1);
            assert_eq!(
                screen_set_title(&raw mut current, b"\xff\0".as_ptr().cast(), 0),
                0
            );
            assert_eq!(CStr::from_ptr(current.title), c"original");

            assert!(screen_has_tab(&raw const current, 8));
            screen_set_tab(&raw mut current, 3, true);
            assert!(screen_has_tab(&raw const current, 3));
            screen_set_tab(&raw mut current, 8, false);
            assert!(!screen_has_tab(&raw const current, 8));
            screen_clear_tabs(&raw mut current);
            assert!((0..10).all(|x| !screen_has_tab(&raw const current, x)));
            screen_resize(&raw mut current, 17, 2, 0);
            assert!(screen_has_tab(&raw const current, 8));
            assert!(screen_has_tab(&raw const current, 16));
            assert!(!screen_has_tab(&raw const current, 3));
            let mut cell = crate::src::grid::grid_default_cell;
            screen_set_selection(
                &raw mut current,
                1,
                0,
                4,
                1,
                0,
                0,
                MODEKEY_EMACS,
                &raw mut cell,
            );
            assert_eq!(screen_check_selection(&raw mut current, 1, 0), 1);
            assert_eq!(screen_check_selection(&raw mut current, 0, 0), 0);
            assert_eq!(screen_check_selection(&raw mut current, 4, 1), 0);
            screen_hide_selection(&raw mut current);
            assert_eq!(screen_check_selection(&raw mut current, 1, 0), 0);
            screen_set_selection(
                &raw mut current,
                1,
                0,
                4,
                1,
                1,
                2,
                MODEKEY_EMACS,
                &raw mut cell,
            );
            assert_eq!(screen_check_selection(&raw mut current, 1, 0), 0);
            assert_eq!(screen_check_selection(&raw mut current, 2, 0), 1);

            crate::src::screen_write::screen_write_make_list(&raw mut current);
            let rows = current.write_list;
            (*rows).data.resize_with(17, || b'A' as std::ffi::c_char);
            (*rows.add(1))
                .data
                .resize_with(17, || b'B' as std::ffi::c_char);
            // Alternate-screen owners must move with the screen, too.
            let original_grid = current.grid;
            assert_eq!(screen_alternate_on(&raw mut current, &raw mut cell, 1), 1);
            let saved_grid = current.saved_grid;
            assert_eq!(screen_alternate_on(&raw mut current, &raw mut cell, 1), 0);
            assert_eq!(current.saved_grid, saved_grid);
            let link = crate::src::hyperlinks::hyperlinks_put(
                current.hyperlinks,
                c"https://retained.test".as_ptr(),
                c"shared".as_ptr(),
            );
            screen_share_hyperlinks(&raw mut current, &raw const current);
            assert_eq!((*current.hyperlinks).references, 1);
            let mut old = std::mem::replace(&mut current, std::mem::zeroed());
            screen_init(&raw mut current, 10, 2, 0);
            screen_set_title(&raw mut current, c"new".as_ptr(), 0);
            screen_set_title(&raw mut old, c"changed".as_ptr(), 0);
            screen_pop_title(&raw mut old);
            assert_eq!(CStr::from_ptr(old.title), c"original");
            assert_eq!(CStr::from_ptr(old.path), c"/tmp/path");
            assert_eq!(old.write_list, rows);
            assert_eq!((&(*old.write_list).data)[0], b'A' as std::ffi::c_char);
            assert_eq!(
                (&(*old.write_list.add(1)).data)[0],
                b'B' as std::ffi::c_char
            );
            assert!(current.write_list.is_null());
            assert_eq!(old.grid, original_grid);
            assert_eq!(old.saved_grid, saved_grid);
            assert!(current.saved_grid.is_null());
            screen_share_hyperlinks(&raw mut current, &raw const old);
            assert_eq!(current.hyperlinks, old.hyperlinks);
            assert_eq!((*current.hyperlinks).references, 2);
            assert!(old.sel.is_some());
            assert!(current.sel.is_none());
            assert!(screen_has_tab(&raw const old, 16));
            screen_free(&raw mut old);
            assert!(old.titles.is_null());
            assert!(old.sel.is_none());
            assert!(old.tabs.is_null());
            assert!(old.write_list.is_null());
            assert!(old.grid.is_null());
            assert!(old.saved_grid.is_null());
            assert!(old.hyperlinks.is_null());
            assert_eq!((*current.hyperlinks).references, 1);
            let mut uri = std::ptr::null();
            assert_eq!(
                crate::src::hyperlinks::hyperlinks_get(
                    current.hyperlinks,
                    link,
                    &raw mut uri,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                1
            );
            assert_eq!(CStr::from_ptr(uri), c"https://retained.test");
            // Reset clears entries without replacing the shared table identity.
            let table = current.hyperlinks;
            screen_reset_hyperlinks(&raw mut current);
            assert_eq!(current.hyperlinks, table);
            assert_eq!(
                crate::src::hyperlinks::hyperlinks_get(
                    current.hyperlinks,
                    link,
                    &raw mut uri,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0
            );
            assert_eq!(CStr::from_ptr(current.title), c"new");

            cell.data.data[0] = b'X';
            crate::src::grid::grid_set_cell(current.grid, 0, 0, &cell);
            crate::src::screen_write::screen_write_make_list(&raw mut current);
            (*current.write_list).data.resize_with(10, || 42);
            let original_grid = current.grid;
            for width in [6, 19, 10] {
                assert_eq!(screen_alternate_on(&raw mut current, &raw mut cell, 1), 1);
                screen_resize(&raw mut current, width, 2, 1);
                assert_eq!(screen_alternate_off(&raw mut current, &raw mut cell, 1), 1);
                assert!(current.saved_grid.is_null());
                assert_eq!(current.grid, original_grid);
                assert!(!current.write_list.is_null());
                assert!((&(*current.write_list).data).is_empty());
                crate::src::grid::grid_get_cell(current.grid, 0, 0, &raw mut cell);
                assert_eq!(cell.data.data[0], b'X');
                assert_eq!(screen_alternate_off(&raw mut current, &raw mut cell, 1), 0);
            }

            screen_push_title(&raw mut current);
            screen_set_selection(
                &raw mut current,
                1,
                0,
                4,
                1,
                0,
                0,
                MODEKEY_EMACS,
                &raw mut cell,
            );
            screen_clear_tabs(&raw mut current);
            screen_reinit(&raw mut current, 0);
            assert_eq!(current.ntitles, 0);
            assert!(current.sel.is_none());
            assert!(screen_has_tab(&raw const current, 8));
            assert_eq!(CStr::from_ptr(current.title), c"new");
            screen_free(&raw mut current);
            assert!(current.titles.is_null());
            screen_init(&raw mut current, 10, 2, 0);
            assert_eq!(CStr::from_ptr(current.title), c"");
            assert!(current.path.is_null());
            screen_free(&raw mut current);
            global_options = saved_options;
            options_free(options);
        }
    }
}
