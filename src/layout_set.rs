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
pub use crate::src::shared::redraw::{redraw_scene};
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::{spawn_editor_state};
pub use crate::src::shared::status::{status_line};
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::shared::environment::{environ};
pub use crate::src::shared::pane::{
    PANE_MINIMUM, window_pane_offset, window_pane_resize, window_pane_resize_entry,
    window_pane_resizes,
};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
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
extern "C" {

    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn events_fire_window(_: *const ::core::ffi::c_char, _: *mut window);
    fn options_get_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn args_string_percentage(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn server_redraw_window(_: *mut window);
    fn window_resize(
        _: *mut window,
        _: u_int,
        _: u_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn window_count_panes(_: *mut window, _: ::core::ffi::c_int) -> u_int;
    fn layout_create_cell(_: *mut layout_cell) -> *mut layout_cell;
    fn layout_print_cell(_: *mut layout_cell, _: *const ::core::ffi::c_char, _: u_int);
    fn layout_set_size(
        _: *mut layout_cell,
        _: u_int,
        _: u_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn layout_make_node(_: *mut layout_cell, _: layout_type);
    fn layout_cell_is_tiled(_: *mut layout_cell) -> ::core::ffi::c_int;
    fn layout_fix_offsets(_: *mut window);
    fn layout_fix_panes(_: *mut window, _: *mut window_pane);
    fn layout_resize_adjust(
        _: *mut window,
        _: *mut layout_cell,
        _: layout_type,
        _: ::core::ffi::c_int,
    );
    fn layout_free(_: *mut window, _: ::core::ffi::c_int);
    fn layout_spread_cell(_: *mut window, _: *mut layout_cell) -> ::core::ffi::c_int;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_12 {
    pub offset: u_int,
    pub data: C2RustUnnamed_13,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_13 {
    pub attr: u_char,
    pub fg: u_char,
    pub bg: u_char,
    pub data: u_char,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub name: *const ::core::ffi::c_char,
    pub arrange: Option<unsafe extern "C" fn(*mut window) -> ()>,
}
static mut layout_sets: [C2RustUnnamed_35; 7] = unsafe {
    [
        C2RustUnnamed_35 {
            name: b"even-horizontal\0" as *const u8 as *const ::core::ffi::c_char,
            arrange: Some(layout_set_even_h as unsafe extern "C" fn(*mut window) -> ()),
        },
        C2RustUnnamed_35 {
            name: b"even-vertical\0" as *const u8 as *const ::core::ffi::c_char,
            arrange: Some(layout_set_even_v as unsafe extern "C" fn(*mut window) -> ()),
        },
        C2RustUnnamed_35 {
            name: b"main-horizontal\0" as *const u8 as *const ::core::ffi::c_char,
            arrange: Some(layout_set_main_h as unsafe extern "C" fn(*mut window) -> ()),
        },
        C2RustUnnamed_35 {
            name: b"main-horizontal-mirrored\0" as *const u8 as *const ::core::ffi::c_char,
            arrange: Some(layout_set_main_h_mirrored as unsafe extern "C" fn(*mut window) -> ()),
        },
        C2RustUnnamed_35 {
            name: b"main-vertical\0" as *const u8 as *const ::core::ffi::c_char,
            arrange: Some(layout_set_main_v as unsafe extern "C" fn(*mut window) -> ()),
        },
        C2RustUnnamed_35 {
            name: b"main-vertical-mirrored\0" as *const u8 as *const ::core::ffi::c_char,
            arrange: Some(layout_set_main_v_mirrored as unsafe extern "C" fn(*mut window) -> ()),
        },
        C2RustUnnamed_35 {
            name: b"tiled\0" as *const u8 as *const ::core::ffi::c_char,
            arrange: Some(layout_set_tiled as unsafe extern "C" fn(*mut window) -> ()),
        },
    ]
};
#[no_mangle]
pub unsafe extern "C" fn layout_set_lookup(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    let mut matched: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
    {
        if strcmp(layout_sets[i as usize].name, name) == 0 as ::core::ffi::c_int {
            return i as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
    {
        if strncmp(layout_sets[i as usize].name, name, strlen(name)) == 0 as ::core::ffi::c_int {
            if matched != -(1 as ::core::ffi::c_int) {
                return -(1 as ::core::ffi::c_int);
            }
            matched = i as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    return matched;
}
#[no_mangle]
pub unsafe extern "C" fn layout_set_select(mut w: *mut window, mut layout: u_int) -> u_int {
    if layout as usize
        > (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
            .wrapping_sub(1 as usize)
    {
        layout = (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
            .wrapping_sub(1 as usize) as u_int;
    }
    if layout_sets[layout as usize].arrange.is_some() {
        layout_sets[layout as usize]
            .arrange
            .expect("non-null function pointer")(w);
    }
    (*w).lastlayout = layout as ::core::ffi::c_int;
    return layout;
}
#[no_mangle]
pub unsafe extern "C" fn layout_set_next(mut w: *mut window) -> u_int {
    let mut layout: u_int = 0;
    if (*w).lastlayout == -(1 as ::core::ffi::c_int) {
        layout = 0 as u_int;
    } else {
        layout = ((*w).lastlayout + 1 as ::core::ffi::c_int) as u_int;
        if layout as usize
            > (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
                .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
                .wrapping_sub(1 as usize)
        {
            layout = 0 as u_int;
        }
    }
    if layout_sets[layout as usize].arrange.is_some() {
        layout_sets[layout as usize]
            .arrange
            .expect("non-null function pointer")(w);
    }
    (*w).lastlayout = layout as ::core::ffi::c_int;
    return layout;
}
#[no_mangle]
pub unsafe extern "C" fn layout_set_previous(mut w: *mut window) -> u_int {
    let mut layout: u_int = 0;
    if (*w).lastlayout == -(1 as ::core::ffi::c_int) {
        layout = (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
            .wrapping_sub(1 as usize) as u_int;
    } else {
        layout = (*w).lastlayout as u_int;
        if layout == 0 as u_int {
            layout = (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
                .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
                .wrapping_sub(1 as usize) as u_int;
        } else {
            layout = layout.wrapping_sub(1);
        }
    }
    if layout_sets[layout as usize].arrange.is_some() {
        layout_sets[layout as usize]
            .arrange
            .expect("non-null function pointer")(w);
    }
    (*w).lastlayout = layout as ::core::ffi::c_int;
    return layout;
}
unsafe extern "C" fn layout_set_first_tiled(mut w: *mut window) -> *mut window_pane {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if !(*wp).layout_cell.is_null()
            && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) != 0
        {
            return wp;
        }
        wp = (*wp).entry.tqe_next;
    }
    return ::core::ptr::null_mut::<window_pane>();
}
unsafe extern "C" fn layout_set_link_floating(mut w: *mut window, mut lcroot: *mut layout_cell) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        lc = (*wp).layout_cell as *mut layout_cell;
        if layout_cell_is_tiled(lc) == 0 {
            (*lc).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
            (*lc).entry.tqe_prev = (*lcroot).cells.tqh_last;
            *(*lcroot).cells.tqh_last = lc;
            (*lcroot).cells.tqh_last = &raw mut (*lc).entry.tqe_next;
            (*lc).parent = lcroot;
        }
        wp = (*wp).entry.tqe_next;
    }
}
unsafe extern "C" fn layout_set_even(mut w: *mut window, mut type_0: layout_type) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_even\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    n = window_count_panes(w, 0 as ::core::ffi::c_int);
    if n <= 1 as u_int {
        return;
    }
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        sx = n
            .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
            .wrapping_sub(1 as u_int);
        if sx < (*w).sx {
            sx = (*w).sx;
        }
        sy = (*w).sy;
    } else {
        sy = n
            .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
            .wrapping_sub(1 as u_int);
        if sy < (*w).sy {
            sy = (*w).sy;
        }
        sx = (*w).sx;
    }
    layout_free(w, 1 as ::core::ffi::c_int);
    (*w).layout_root = layout_create_cell(::core::ptr::null_mut::<layout_cell>());
    lcroot = (*w).layout_root;
    layout_set_size(
        lcroot,
        sx,
        sy,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_make_node(lcroot, type_0);
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        lcchild = (*wp).layout_cell as *mut layout_cell;
        (*lcchild).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
        (*lcchild).entry.tqe_prev = (*lcroot).cells.tqh_last;
        *(*lcroot).cells.tqh_last = lcchild;
        (*lcroot).cells.tqh_last = &raw mut (*lcchild).entry.tqe_next;
        (*lcchild).parent = lcroot;
        if layout_cell_is_tiled(lcchild) != 0 {
            (*lcchild).g.sx = (*w).sx;
            (*lcchild).g.sy = (*w).sy;
        }
        wp = (*wp).entry.tqe_next;
    }
    layout_spread_cell(w, lcroot);
    layout_fix_offsets(w);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_even\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    window_resize(
        w,
        (*lcroot).g.sx,
        (*lcroot).g.sy,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    server_redraw_window(w);
}
unsafe extern "C" fn layout_set_even_h(mut w: *mut window) {
    layout_set_even(w, LAYOUT_LEFTRIGHT);
}
unsafe extern "C" fn layout_set_even_v(mut w: *mut window) {
    layout_set_even(w, LAYOUT_TOPBOTTOM);
}
unsafe extern "C" fn layout_set_main_h(mut w: *mut window) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wpmain: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcmain: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcother: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0;
    let mut mainh: u_int = 0;
    let mut otherh: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_main_h\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    n = window_count_panes(w, 0 as ::core::ffi::c_int);
    if n <= 1 as u_int {
        return;
    }
    n = n.wrapping_sub(1);
    sy = (*w).sy.wrapping_sub(1 as u_int);
    s = options_get_string(
        (*w).options,
        b"main-pane-height\0" as *const u8 as *const ::core::ffi::c_char,
    );
    mainh = args_string_percentage(
        s,
        0 as ::core::ffi::c_longlong,
        sy as ::core::ffi::c_longlong,
        sy as ::core::ffi::c_longlong,
        &raw mut cause,
    ) as u_int;
    if !cause.is_null() {
        mainh = 24 as u_int;
        free(cause as *mut ::core::ffi::c_void);
    }
    if mainh.wrapping_add(PANE_MINIMUM as u_int) >= sy {
        if sy <= (PANE_MINIMUM + PANE_MINIMUM) as u_int {
            mainh = PANE_MINIMUM as u_int;
        } else {
            mainh = sy.wrapping_sub(PANE_MINIMUM as u_int);
        }
        otherh = PANE_MINIMUM as u_int;
    } else {
        s = options_get_string(
            (*w).options,
            b"other-pane-height\0" as *const u8 as *const ::core::ffi::c_char,
        );
        otherh = args_string_percentage(
            s,
            0 as ::core::ffi::c_longlong,
            sy as ::core::ffi::c_longlong,
            sy as ::core::ffi::c_longlong,
            &raw mut cause,
        ) as u_int;
        if !cause.is_null() || otherh == 0 as u_int {
            otherh = sy.wrapping_sub(mainh);
            free(cause as *mut ::core::ffi::c_void);
        } else if otherh > sy || sy.wrapping_sub(otherh) < mainh {
            otherh = sy.wrapping_sub(mainh);
        } else {
            mainh = sy.wrapping_sub(otherh);
        }
    }
    sx = n
        .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
        .wrapping_sub(1 as u_int);
    if sx < (*w).sx {
        sx = (*w).sx;
    }
    layout_free(w, 1 as ::core::ffi::c_int);
    (*w).layout_root = layout_create_cell(::core::ptr::null_mut::<layout_cell>());
    lcroot = (*w).layout_root;
    layout_set_size(
        lcroot,
        sx,
        mainh.wrapping_add(otherh).wrapping_add(1 as u_int),
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_make_node(lcroot, LAYOUT_TOPBOTTOM);
    wpmain = layout_set_first_tiled(w);
    lcmain = (*wpmain).layout_cell as *mut layout_cell;
    (*lcmain).parent = lcroot;
    layout_set_size(
        lcmain,
        sx,
        mainh,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    (*lcmain).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
    (*lcmain).entry.tqe_prev = (*lcroot).cells.tqh_last;
    *(*lcroot).cells.tqh_last = lcmain;
    (*lcroot).cells.tqh_last = &raw mut (*lcmain).entry.tqe_next;
    if n == 1 as u_int {
        wp = (*wpmain).entry.tqe_next;
        while !wp.is_null() && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) == 0 {
            wp = (*wp).entry.tqe_next;
        }
        (*(*wp).layout_cell).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
        (*(*wp).layout_cell).entry.tqe_prev = (*lcroot).cells.tqh_last;
        *(*lcroot).cells.tqh_last = (*wp).layout_cell as *mut layout_cell;
        (*lcroot).cells.tqh_last = &raw mut (*(*wp).layout_cell).entry.tqe_next;
        (*(*wp).layout_cell).parent = lcroot;
        layout_set_size(
            (*wp).layout_cell as *mut layout_cell,
            sx,
            otherh,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_set_link_floating(w, lcroot);
    } else {
        lcother = layout_create_cell(lcroot);
        layout_set_size(
            lcother,
            sx,
            otherh,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_make_node(lcother, LAYOUT_LEFTRIGHT);
        (*lcother).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
        (*lcother).entry.tqe_prev = (*lcroot).cells.tqh_last;
        *(*lcroot).cells.tqh_last = lcother;
        (*lcroot).cells.tqh_last = &raw mut (*lcother).entry.tqe_next;
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            if !(wp == wpmain) {
                lcchild = (*wp).layout_cell as *mut layout_cell;
                (*lcchild).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
                (*lcchild).entry.tqe_prev = (*lcother).cells.tqh_last;
                *(*lcother).cells.tqh_last = lcchild;
                (*lcother).cells.tqh_last = &raw mut (*lcchild).entry.tqe_next;
                (*lcchild).parent = lcother;
                if layout_cell_is_tiled(lcchild) != 0 {
                    layout_set_size(
                        lcchild,
                        PANE_MINIMUM as u_int,
                        otherh,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
            }
            wp = (*wp).entry.tqe_next;
        }
        layout_spread_cell(w, lcother);
    }
    layout_fix_offsets(w);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_main_h\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    window_resize(
        w,
        (*lcroot).g.sx,
        (*lcroot).g.sy,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    server_redraw_window(w);
}
unsafe extern "C" fn layout_set_main_h_mirrored(mut w: *mut window) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wpmain: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcmain: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcother: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0;
    let mut mainh: u_int = 0;
    let mut otherh: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_main_h_mirrored\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    n = window_count_panes(w, 0 as ::core::ffi::c_int);
    if n <= 1 as u_int {
        return;
    }
    n = n.wrapping_sub(1);
    sy = (*w).sy.wrapping_sub(1 as u_int);
    s = options_get_string(
        (*w).options,
        b"main-pane-height\0" as *const u8 as *const ::core::ffi::c_char,
    );
    mainh = args_string_percentage(
        s,
        0 as ::core::ffi::c_longlong,
        sy as ::core::ffi::c_longlong,
        sy as ::core::ffi::c_longlong,
        &raw mut cause,
    ) as u_int;
    if !cause.is_null() {
        mainh = 24 as u_int;
        free(cause as *mut ::core::ffi::c_void);
    }
    if mainh.wrapping_add(PANE_MINIMUM as u_int) >= sy {
        if sy <= (PANE_MINIMUM + PANE_MINIMUM) as u_int {
            mainh = PANE_MINIMUM as u_int;
        } else {
            mainh = sy.wrapping_sub(PANE_MINIMUM as u_int);
        }
        otherh = PANE_MINIMUM as u_int;
    } else {
        s = options_get_string(
            (*w).options,
            b"other-pane-height\0" as *const u8 as *const ::core::ffi::c_char,
        );
        otherh = args_string_percentage(
            s,
            0 as ::core::ffi::c_longlong,
            sy as ::core::ffi::c_longlong,
            sy as ::core::ffi::c_longlong,
            &raw mut cause,
        ) as u_int;
        if !cause.is_null() || otherh == 0 as u_int {
            otherh = sy.wrapping_sub(mainh);
            free(cause as *mut ::core::ffi::c_void);
        } else if otherh > sy || sy.wrapping_sub(otherh) < mainh {
            otherh = sy.wrapping_sub(mainh);
        } else {
            mainh = sy.wrapping_sub(otherh);
        }
    }
    sx = n
        .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
        .wrapping_sub(1 as u_int);
    if sx < (*w).sx {
        sx = (*w).sx;
    }
    layout_free(w, 1 as ::core::ffi::c_int);
    (*w).layout_root = layout_create_cell(::core::ptr::null_mut::<layout_cell>());
    lcroot = (*w).layout_root;
    layout_set_size(
        lcroot,
        sx,
        mainh.wrapping_add(otherh).wrapping_add(1 as u_int),
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_make_node(lcroot, LAYOUT_TOPBOTTOM);
    wpmain = layout_set_first_tiled(w);
    lcmain = (*wpmain).layout_cell as *mut layout_cell;
    (*lcmain).parent = lcroot;
    layout_set_size(
        lcmain,
        sx,
        mainh,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    (*lcmain).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
    (*lcmain).entry.tqe_prev = (*lcroot).cells.tqh_last;
    *(*lcroot).cells.tqh_last = lcmain;
    (*lcroot).cells.tqh_last = &raw mut (*lcmain).entry.tqe_next;
    if n == 1 as u_int {
        wp = (*wpmain).entry.tqe_next;
        while !wp.is_null() && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) == 0 {
            wp = (*wp).entry.tqe_next;
        }
        (*(*wp).layout_cell).entry.tqe_next = (*lcroot).cells.tqh_first;
        if !(*(*wp).layout_cell).entry.tqe_next.is_null() {
            (*(*lcroot).cells.tqh_first).entry.tqe_prev =
                &raw mut (*(*wp).layout_cell).entry.tqe_next;
        } else {
            (*lcroot).cells.tqh_last = &raw mut (*(*wp).layout_cell).entry.tqe_next;
        }
        (*lcroot).cells.tqh_first = (*wp).layout_cell as *mut layout_cell;
        (*(*wp).layout_cell).entry.tqe_prev = &raw mut (*lcroot).cells.tqh_first;
        (*(*wp).layout_cell).parent = lcroot;
        layout_set_size(
            (*wp).layout_cell as *mut layout_cell,
            sx,
            otherh,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_set_link_floating(w, lcroot);
    } else {
        lcother = layout_create_cell(lcroot);
        layout_set_size(
            lcother,
            sx,
            otherh,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_make_node(lcother, LAYOUT_LEFTRIGHT);
        (*lcother).entry.tqe_next = (*lcroot).cells.tqh_first;
        if !(*lcother).entry.tqe_next.is_null() {
            (*(*lcroot).cells.tqh_first).entry.tqe_prev = &raw mut (*lcother).entry.tqe_next;
        } else {
            (*lcroot).cells.tqh_last = &raw mut (*lcother).entry.tqe_next;
        }
        (*lcroot).cells.tqh_first = lcother;
        (*lcother).entry.tqe_prev = &raw mut (*lcroot).cells.tqh_first;
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            if !(wp == wpmain) {
                lcchild = (*wp).layout_cell as *mut layout_cell;
                (*lcchild).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
                (*lcchild).entry.tqe_prev = (*lcother).cells.tqh_last;
                *(*lcother).cells.tqh_last = lcchild;
                (*lcother).cells.tqh_last = &raw mut (*lcchild).entry.tqe_next;
                (*lcchild).parent = lcother;
                if layout_cell_is_tiled(lcchild) != 0 {
                    layout_set_size(
                        lcchild,
                        PANE_MINIMUM as u_int,
                        otherh,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
            }
            wp = (*wp).entry.tqe_next;
        }
        layout_spread_cell(w, lcother);
    }
    layout_fix_offsets(w);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_main_h_mirrored\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    window_resize(
        w,
        (*lcroot).g.sx,
        (*lcroot).g.sy,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    server_redraw_window(w);
}
unsafe extern "C" fn layout_set_main_v(mut w: *mut window) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wpmain: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcmain: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcother: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0;
    let mut mainw: u_int = 0;
    let mut otherw: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_main_v\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    n = window_count_panes(w, 0 as ::core::ffi::c_int);
    if n <= 1 as u_int {
        return;
    }
    n = n.wrapping_sub(1);
    sx = (*w).sx.wrapping_sub(1 as u_int);
    s = options_get_string(
        (*w).options,
        b"main-pane-width\0" as *const u8 as *const ::core::ffi::c_char,
    );
    mainw = args_string_percentage(
        s,
        0 as ::core::ffi::c_longlong,
        sx as ::core::ffi::c_longlong,
        sx as ::core::ffi::c_longlong,
        &raw mut cause,
    ) as u_int;
    if !cause.is_null() {
        mainw = 80 as u_int;
        free(cause as *mut ::core::ffi::c_void);
    }
    if mainw.wrapping_add(PANE_MINIMUM as u_int) >= sx {
        if sx <= (PANE_MINIMUM + PANE_MINIMUM) as u_int {
            mainw = PANE_MINIMUM as u_int;
        } else {
            mainw = sx.wrapping_sub(PANE_MINIMUM as u_int);
        }
        otherw = PANE_MINIMUM as u_int;
    } else {
        s = options_get_string(
            (*w).options,
            b"other-pane-width\0" as *const u8 as *const ::core::ffi::c_char,
        );
        otherw = args_string_percentage(
            s,
            0 as ::core::ffi::c_longlong,
            sx as ::core::ffi::c_longlong,
            sx as ::core::ffi::c_longlong,
            &raw mut cause,
        ) as u_int;
        if !cause.is_null() || otherw == 0 as u_int {
            otherw = sx.wrapping_sub(mainw);
            free(cause as *mut ::core::ffi::c_void);
        } else if otherw > sx || sx.wrapping_sub(otherw) < mainw {
            otherw = sx.wrapping_sub(mainw);
        } else {
            mainw = sx.wrapping_sub(otherw);
        }
    }
    sy = n
        .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
        .wrapping_sub(1 as u_int);
    if sy < (*w).sy {
        sy = (*w).sy;
    }
    layout_free(w, 1 as ::core::ffi::c_int);
    (*w).layout_root = layout_create_cell(::core::ptr::null_mut::<layout_cell>());
    lcroot = (*w).layout_root;
    layout_set_size(
        lcroot,
        mainw.wrapping_add(otherw).wrapping_add(1 as u_int),
        sy,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_make_node(lcroot, LAYOUT_LEFTRIGHT);
    wpmain = layout_set_first_tiled(w);
    lcmain = (*wpmain).layout_cell as *mut layout_cell;
    (*lcmain).parent = lcroot;
    layout_set_size(
        lcmain,
        mainw,
        sy,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    (*lcmain).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
    (*lcmain).entry.tqe_prev = (*lcroot).cells.tqh_last;
    *(*lcroot).cells.tqh_last = lcmain;
    (*lcroot).cells.tqh_last = &raw mut (*lcmain).entry.tqe_next;
    if n == 1 as u_int {
        wp = (*wpmain).entry.tqe_next;
        while !wp.is_null() && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) == 0 {
            wp = (*wp).entry.tqe_next;
        }
        (*(*wp).layout_cell).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
        (*(*wp).layout_cell).entry.tqe_prev = (*lcroot).cells.tqh_last;
        *(*lcroot).cells.tqh_last = (*wp).layout_cell as *mut layout_cell;
        (*lcroot).cells.tqh_last = &raw mut (*(*wp).layout_cell).entry.tqe_next;
        (*(*wp).layout_cell).parent = lcroot;
        layout_set_size(
            (*wp).layout_cell as *mut layout_cell,
            otherw,
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_set_link_floating(w, lcroot);
    } else {
        lcother = layout_create_cell(lcroot);
        layout_make_node(lcother, LAYOUT_TOPBOTTOM);
        layout_set_size(
            lcother,
            otherw,
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        (*lcother).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
        (*lcother).entry.tqe_prev = (*lcroot).cells.tqh_last;
        *(*lcroot).cells.tqh_last = lcother;
        (*lcroot).cells.tqh_last = &raw mut (*lcother).entry.tqe_next;
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            if !(wp == wpmain) {
                lcchild = (*wp).layout_cell as *mut layout_cell;
                (*lcchild).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
                (*lcchild).entry.tqe_prev = (*lcother).cells.tqh_last;
                *(*lcother).cells.tqh_last = lcchild;
                (*lcother).cells.tqh_last = &raw mut (*lcchild).entry.tqe_next;
                (*lcchild).parent = lcother;
                if layout_cell_is_tiled(lcchild) != 0 {
                    layout_set_size(
                        lcchild,
                        otherw,
                        PANE_MINIMUM as u_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
            }
            wp = (*wp).entry.tqe_next;
        }
        layout_spread_cell(w, lcother);
    }
    layout_fix_offsets(w);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_main_v\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    window_resize(
        w,
        (*lcroot).g.sx,
        (*lcroot).g.sy,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    server_redraw_window(w);
}
unsafe extern "C" fn layout_set_main_v_mirrored(mut w: *mut window) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wpmain: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcmain: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcother: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0;
    let mut mainw: u_int = 0;
    let mut otherw: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_main_v_mirrored\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    n = window_count_panes(w, 0 as ::core::ffi::c_int);
    if n <= 1 as u_int {
        return;
    }
    n = n.wrapping_sub(1);
    sx = (*w).sx.wrapping_sub(1 as u_int);
    s = options_get_string(
        (*w).options,
        b"main-pane-width\0" as *const u8 as *const ::core::ffi::c_char,
    );
    mainw = args_string_percentage(
        s,
        0 as ::core::ffi::c_longlong,
        sx as ::core::ffi::c_longlong,
        sx as ::core::ffi::c_longlong,
        &raw mut cause,
    ) as u_int;
    if !cause.is_null() {
        mainw = 80 as u_int;
        free(cause as *mut ::core::ffi::c_void);
    }
    if mainw.wrapping_add(PANE_MINIMUM as u_int) >= sx {
        if sx <= (PANE_MINIMUM + PANE_MINIMUM) as u_int {
            mainw = PANE_MINIMUM as u_int;
        } else {
            mainw = sx.wrapping_sub(PANE_MINIMUM as u_int);
        }
        otherw = PANE_MINIMUM as u_int;
    } else {
        s = options_get_string(
            (*w).options,
            b"other-pane-width\0" as *const u8 as *const ::core::ffi::c_char,
        );
        otherw = args_string_percentage(
            s,
            0 as ::core::ffi::c_longlong,
            sx as ::core::ffi::c_longlong,
            sx as ::core::ffi::c_longlong,
            &raw mut cause,
        ) as u_int;
        if !cause.is_null() || otherw == 0 as u_int {
            otherw = sx.wrapping_sub(mainw);
            free(cause as *mut ::core::ffi::c_void);
        } else if otherw > sx || sx.wrapping_sub(otherw) < mainw {
            otherw = sx.wrapping_sub(mainw);
        } else {
            mainw = sx.wrapping_sub(otherw);
        }
    }
    sy = n
        .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
        .wrapping_sub(1 as u_int);
    if sy < (*w).sy {
        sy = (*w).sy;
    }
    layout_free(w, 1 as ::core::ffi::c_int);
    (*w).layout_root = layout_create_cell(::core::ptr::null_mut::<layout_cell>());
    lcroot = (*w).layout_root;
    layout_set_size(
        lcroot,
        mainw.wrapping_add(otherw).wrapping_add(1 as u_int),
        sy,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_make_node(lcroot, LAYOUT_LEFTRIGHT);
    wpmain = layout_set_first_tiled(w);
    lcmain = (*wpmain).layout_cell as *mut layout_cell;
    (*lcmain).parent = lcroot;
    layout_set_size(
        lcmain,
        mainw,
        sy,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    (*lcmain).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
    (*lcmain).entry.tqe_prev = (*lcroot).cells.tqh_last;
    *(*lcroot).cells.tqh_last = lcmain;
    (*lcroot).cells.tqh_last = &raw mut (*lcmain).entry.tqe_next;
    if n == 1 as u_int {
        wp = (*wpmain).entry.tqe_next;
        while !wp.is_null() && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) == 0 {
            wp = (*wp).entry.tqe_next;
        }
        (*(*wp).layout_cell).entry.tqe_next = (*lcroot).cells.tqh_first;
        if !(*(*wp).layout_cell).entry.tqe_next.is_null() {
            (*(*lcroot).cells.tqh_first).entry.tqe_prev =
                &raw mut (*(*wp).layout_cell).entry.tqe_next;
        } else {
            (*lcroot).cells.tqh_last = &raw mut (*(*wp).layout_cell).entry.tqe_next;
        }
        (*lcroot).cells.tqh_first = (*wp).layout_cell as *mut layout_cell;
        (*(*wp).layout_cell).entry.tqe_prev = &raw mut (*lcroot).cells.tqh_first;
        (*(*wp).layout_cell).parent = lcroot;
        layout_set_size(
            (*wp).layout_cell as *mut layout_cell,
            otherw,
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_set_link_floating(w, lcroot);
    } else {
        lcother = layout_create_cell(lcroot);
        layout_make_node(lcother, LAYOUT_TOPBOTTOM);
        layout_set_size(
            lcother,
            otherw,
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        (*lcother).entry.tqe_next = (*lcroot).cells.tqh_first;
        if !(*lcother).entry.tqe_next.is_null() {
            (*(*lcroot).cells.tqh_first).entry.tqe_prev = &raw mut (*lcother).entry.tqe_next;
        } else {
            (*lcroot).cells.tqh_last = &raw mut (*lcother).entry.tqe_next;
        }
        (*lcroot).cells.tqh_first = lcother;
        (*lcother).entry.tqe_prev = &raw mut (*lcroot).cells.tqh_first;
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            if !(wp == wpmain) {
                lcchild = (*wp).layout_cell as *mut layout_cell;
                (*lcchild).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
                (*lcchild).entry.tqe_prev = (*lcother).cells.tqh_last;
                *(*lcother).cells.tqh_last = lcchild;
                (*lcother).cells.tqh_last = &raw mut (*lcchild).entry.tqe_next;
                (*lcchild).parent = lcother;
                if layout_cell_is_tiled(lcchild) != 0 {
                    layout_set_size(
                        lcchild,
                        otherw,
                        PANE_MINIMUM as u_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
            }
            wp = (*wp).entry.tqe_next;
        }
        layout_spread_cell(w, lcother);
    }
    layout_fix_offsets(w);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_main_v_mirrored\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    window_resize(
        w,
        (*lcroot).g.sx,
        (*lcroot).g.sy,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    server_redraw_window(w);
}
unsafe extern "C" fn layout_set_tiled(mut w: *mut window) {
    let mut oo: *mut options = (*w).options;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcrow: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0;
    let mut width: u_int = 0;
    let mut height: u_int = 0;
    let mut used: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut columns: u_int = 0;
    let mut rows: u_int = 0;
    let mut max_columns: u_int = 0;
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_tiled\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    n = window_count_panes(w, 0 as ::core::ffi::c_int);
    if n <= 1 as u_int {
        return;
    }
    max_columns = options_get_number(
        oo,
        b"tiled-layout-max-columns\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    columns = 1 as u_int;
    rows = columns;
    while rows.wrapping_mul(columns) < n {
        rows = rows.wrapping_add(1);
        if rows.wrapping_mul(columns) < n && (max_columns == 0 as u_int || columns < max_columns) {
            columns = columns.wrapping_add(1);
        }
    }
    width = (*w)
        .sx
        .wrapping_sub(columns.wrapping_sub(1 as u_int))
        .wrapping_div(columns);
    if width < PANE_MINIMUM as u_int {
        width = PANE_MINIMUM as u_int;
    }
    height = (*w)
        .sy
        .wrapping_sub(rows.wrapping_sub(1 as u_int))
        .wrapping_div(rows);
    if height < PANE_MINIMUM as u_int {
        height = PANE_MINIMUM as u_int;
    }
    sx = width
        .wrapping_add(1 as u_int)
        .wrapping_mul(columns)
        .wrapping_sub(1 as u_int);
    if sx < (*w).sx {
        sx = (*w).sx;
    }
    sy = height
        .wrapping_add(1 as u_int)
        .wrapping_mul(rows)
        .wrapping_sub(1 as u_int);
    if sy < (*w).sy {
        sy = (*w).sy;
    }
    layout_free(w, 1 as ::core::ffi::c_int);
    (*w).layout_root = layout_create_cell(::core::ptr::null_mut::<layout_cell>());
    lcroot = (*w).layout_root;
    layout_set_size(
        lcroot,
        sx,
        sy,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_make_node(lcroot, LAYOUT_TOPBOTTOM);
    wp = (*w).panes.tqh_first;
    j = 0 as u_int;
    while j < rows {
        while !wp.is_null() && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) == 0 {
            wp = (*wp).entry.tqe_next;
        }
        if wp.is_null() {
            break;
        }
        lcchild = (*wp).layout_cell as *mut layout_cell;
        if n.wrapping_sub(j.wrapping_mul(columns)) == 1 as u_int || columns == 1 as u_int {
            (*lcchild).parent = lcroot;
            (*lcchild).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
            (*lcchild).entry.tqe_prev = (*lcroot).cells.tqh_last;
            *(*lcroot).cells.tqh_last = lcchild;
            (*lcroot).cells.tqh_last = &raw mut (*lcchild).entry.tqe_next;
            layout_set_size(
                lcchild,
                (*w).sx,
                height,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            wp = (*wp).entry.tqe_next;
        } else {
            lcrow = layout_create_cell(lcroot);
            layout_make_node(lcrow, LAYOUT_LEFTRIGHT);
            layout_set_size(
                lcrow,
                (*w).sx,
                height,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            (*lcrow).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
            (*lcrow).entry.tqe_prev = (*lcroot).cells.tqh_last;
            *(*lcroot).cells.tqh_last = lcrow;
            (*lcroot).cells.tqh_last = &raw mut (*lcrow).entry.tqe_next;
            i = 0 as u_int;
            while i < columns {
                (*lcchild).parent = lcrow;
                (*lcchild).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
                (*lcchild).entry.tqe_prev = (*lcrow).cells.tqh_last;
                *(*lcrow).cells.tqh_last = lcchild;
                (*lcrow).cells.tqh_last = &raw mut (*lcchild).entry.tqe_next;
                layout_set_size(
                    lcchild,
                    width,
                    height,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                wp = (*wp).entry.tqe_next;
                while !wp.is_null()
                    && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) == 0
                {
                    wp = (*wp).entry.tqe_next;
                }
                if wp.is_null() {
                    break;
                }
                lcchild = (*wp).layout_cell as *mut layout_cell;
                i = i.wrapping_add(1);
            }
            if i == columns {
                i = i.wrapping_sub(1);
            }
            used = i
                .wrapping_add(1 as u_int)
                .wrapping_mul(width.wrapping_add(1 as u_int))
                .wrapping_sub(1 as u_int);
            if !((*w).sx <= used) {
                lcchild = *(*((*lcrow).cells.tqh_last as *mut layout_cells)).tqh_last;
                layout_resize_adjust(
                    w,
                    lcchild,
                    LAYOUT_LEFTRIGHT,
                    (*w).sx.wrapping_sub(used) as ::core::ffi::c_int,
                );
            }
        }
        j = j.wrapping_add(1);
    }
    used = rows
        .wrapping_mul(height)
        .wrapping_add(rows)
        .wrapping_sub(1 as u_int);
    if (*w).sy > used {
        lcrow = *(*((*lcroot).cells.tqh_last as *mut layout_cells)).tqh_last;
        layout_resize_adjust(
            w,
            lcrow,
            LAYOUT_TOPBOTTOM,
            (*w).sy.wrapping_sub(used) as ::core::ffi::c_int,
        );
    }
    layout_set_link_floating(w, lcroot);
    layout_fix_offsets(w);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_tiled\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    window_resize(
        w,
        (*lcroot).g.sx,
        (*lcroot).g.sy,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    server_redraw_window(w);
}
