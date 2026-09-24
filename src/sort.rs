use crate::src::ffi::libc::{qsort, strcasecmp, strcmp};
use crate::src::key_bindings::{
    key_bindings_first, key_bindings_first_table, key_bindings_next, key_bindings_next_table,
};
use crate::src::paste::paste_walk;
use crate::src::server::clients;
use crate::src::session::sessions;
use crate::src::session::{sessions_minmax, sessions_next};
use crate::src::shared::abi::__compar_fn_t;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args;
use crate::src::shared::client::*;
use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files, overlay_check_cb,
    overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb, overlay_resize_cb,
};
use crate::src::shared::client::{
    CLIENT_ATTACHED, CLIENT_DEAD, CLIENT_EXIT, CLIENT_SUSPENDED, CLIENT_UNATTACHEDFLAGS,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
use crate::src::shared::display::{visible_range, visible_ranges};
use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
use crate::src::shared::format::{format_job_tree, format_tree};
use crate::src::shared::grid::*;
use crate::src::shared::hyperlinks::hyperlinks;
use crate::src::shared::input::{input_ctx, input_request, input_requests};
use crate::src::shared::key::*;
use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::options::options;
use crate::src::shared::pane::{
    window_pane, window_pane_modes, window_pane_prompt, window_pane_tree_entry, window_panes,
};
use crate::src::shared::pane::{window_pane_offset, window_pane_resize, window_pane_resizes};
use crate::src::shared::paste::{
    paste_buffer, paste_buffer_name_entry, paste_buffer_time_entry,
};
use crate::src::shared::process::tmuxpeer;
use crate::src::shared::prompt::prompt;
use crate::src::shared::redraw::redraw_scene;
use crate::src::shared::screen::{screen, screen_sel, screen_titles};
use crate::src::shared::screen_write::screen_write_cline;
use crate::src::shared::session::{session, session_entry};
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::sort::*;
use crate::src::shared::spawn::spawn_editor_state;
use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
use crate::src::shared::window::{
    window, window_entry, window_mode, window_mode_entry, window_winlinks, winlink, winlink_entry,
    winlink_stack, winlinks,
};
use crate::src::window::{
    window_pane_first, window_pane_index, window_pane_next, window_pane_zindex, winlinks_minmax,
    winlinks_next,
};

use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

static mut sort_criteria: *mut sort_criteria =
    ::core::ptr::null::<sort_criteria>() as *mut sort_criteria;
unsafe extern "C" fn sort_qsort(
    mut l: *mut ::core::ffi::c_void,
    mut len: u_int,
    mut size: u_int,
    mut cmp: Option<
        unsafe extern "C" fn(
            *const ::core::ffi::c_void,
            *const ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    mut sort_crit: *mut sort_criteria,
) {
    let mut i: u_int = 0;
    let mut tmp: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut ll: *mut *mut ::core::ffi::c_void = ::core::ptr::null_mut::<*mut ::core::ffi::c_void>();
    if len < 2 as u_int
        || (*sort_crit).order as ::core::ffi::c_uint
            == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    if (*sort_crit).order as ::core::ffi::c_uint
        == SORT_ORDER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*sort_crit).reversed != 0 {
            ll = l as *mut *mut ::core::ffi::c_void;
            i = 0 as u_int;
            while i < len.wrapping_div(2 as u_int) {
                tmp = *ll.offset(i as isize);
                let ref mut fresh2 = *ll.offset(i as isize);
                *fresh2 = *ll.offset(len.wrapping_sub(1 as u_int).wrapping_sub(i) as isize);
                let ref mut fresh3 =
                    *ll.offset(len.wrapping_sub(1 as u_int).wrapping_sub(i) as isize);
                *fresh3 = tmp;
                i = i.wrapping_add(1);
            }
        }
    } else {
        sort_criteria = sort_crit;
        qsort(l, len as size_t, size as size_t, cmp as __compar_fn_t);
    };
}
unsafe extern "C" fn sort_buffer_cmp(
    mut a0: *const ::core::ffi::c_void,
    mut b0: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut sort_crit: *mut sort_criteria = sort_criteria;
    let mut a: *const *const paste_buffer = a0 as *const *const paste_buffer;
    let mut b: *const *const paste_buffer = b0 as *const *const paste_buffer;
    let mut pa: *const paste_buffer = *a;
    let mut pb: *const paste_buffer = *b;
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match (*sort_crit).order as ::core::ffi::c_uint {
        4 => {
            result = strcmp(
                ((*pa).name).as_ptr().cast_mut(),
                ((*pb).name).as_ptr().cast_mut(),
            );
        }
        1 => {
            if (*pa).order > (*pb).order {
                result = -(1 as ::core::ffi::c_int);
            } else if (*pa).order < (*pb).order {
                result = 1 as ::core::ffi::c_int;
            } else {
                result = 0 as ::core::ffi::c_int;
            }
        }
        6 => {
            result = (*pa).size.wrapping_sub((*pb).size) as ::core::ffi::c_int;
        }
        0 | 2 | 3 | 5 | 7 | 8 | _ => {}
    }
    if result == 0 as ::core::ffi::c_int {
        result = strcmp(
            ((*pa).name).as_ptr().cast_mut(),
            ((*pb).name).as_ptr().cast_mut(),
        );
    }
    if (*sort_crit).reversed != 0 {
        result = -result;
    }
    return result;
}
unsafe extern "C" fn sort_client_cmp(
    mut a0: *const ::core::ffi::c_void,
    mut b0: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut sort_crit: *mut sort_criteria = sort_criteria;
    let mut a: *const *const client = a0 as *const *const client;
    let mut b: *const *const client = b0 as *const *const client;
    let mut ca: *const client = *a;
    let mut cb: *const client = *b;
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match (*sort_crit).order as ::core::ffi::c_uint {
        4 => {
            result = strcmp(
                ((*ca).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                ((*cb).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            );
        }
        6 => {
            result = (*ca).tty.sx.wrapping_sub((*cb).tty.sx) as ::core::ffi::c_int;
            if result == 0 as ::core::ffi::c_int {
                result = (*ca).tty.sy.wrapping_sub((*cb).tty.sy) as ::core::ffi::c_int;
            }
        }
        1 => {
            if if (*ca).creation_time.tv_sec == (*cb).creation_time.tv_sec {
                ((*ca).creation_time.tv_usec > (*cb).creation_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*ca).creation_time.tv_sec > (*cb).creation_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = 1 as ::core::ffi::c_int;
            } else if if (*ca).creation_time.tv_sec == (*cb).creation_time.tv_sec {
                ((*ca).creation_time.tv_usec < (*cb).creation_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*ca).creation_time.tv_sec < (*cb).creation_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = -(1 as ::core::ffi::c_int);
            }
        }
        0 => {
            if if (*ca).activity_time.tv_sec == (*cb).activity_time.tv_sec {
                ((*ca).activity_time.tv_usec > (*cb).activity_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*ca).activity_time.tv_sec > (*cb).activity_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = -(1 as ::core::ffi::c_int);
            } else if if (*ca).activity_time.tv_sec == (*cb).activity_time.tv_sec {
                ((*ca).activity_time.tv_usec < (*cb).activity_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*ca).activity_time.tv_sec < (*cb).activity_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = 1 as ::core::ffi::c_int;
            }
        }
        2 | 3 | 5 | 7 | 8 | _ => {}
    }
    if result == 0 as ::core::ffi::c_int {
        result = strcmp(
            ((*ca).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            ((*cb).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        );
    }
    if (*sort_crit).reversed != 0 {
        result = -result;
    }
    return result;
}
unsafe extern "C" fn sort_session_cmp(
    mut a0: *const ::core::ffi::c_void,
    mut b0: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut sort_crit: *mut sort_criteria = sort_criteria;
    let mut a: *const *const session = a0 as *const *const session;
    let mut b: *const *const session = b0 as *const *const session;
    let mut sa: *const session = *a;
    let mut sb: *const session = *b;
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match (*sort_crit).order as ::core::ffi::c_uint {
        2 => {
            result = (*sa).id.wrapping_sub((*sb).id) as ::core::ffi::c_int;
        }
        1 => {
            if if (*sa).creation_time.tv_sec == (*sb).creation_time.tv_sec {
                ((*sa).creation_time.tv_usec > (*sb).creation_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*sa).creation_time.tv_sec > (*sb).creation_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = 1 as ::core::ffi::c_int;
            } else if if (*sa).creation_time.tv_sec == (*sb).creation_time.tv_sec {
                ((*sa).creation_time.tv_usec < (*sb).creation_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*sa).creation_time.tv_sec < (*sb).creation_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = -(1 as ::core::ffi::c_int);
            }
        }
        0 => {
            if if (*sa).activity_time.tv_sec == (*sb).activity_time.tv_sec {
                ((*sa).activity_time.tv_usec > (*sb).activity_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*sa).activity_time.tv_sec > (*sb).activity_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = -(1 as ::core::ffi::c_int);
            } else if if (*sa).activity_time.tv_sec == (*sb).activity_time.tv_sec {
                ((*sa).activity_time.tv_usec < (*sb).activity_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*sa).activity_time.tv_sec < (*sb).activity_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = 1 as ::core::ffi::c_int;
            }
        }
        4 => {
            result = strcmp(
                ((*sa).name).as_ptr().cast_mut(),
                ((*sb).name).as_ptr().cast_mut(),
            );
        }
        3 | 5 | 6 | 7 | 8 | _ => {}
    }
    if result == 0 as ::core::ffi::c_int {
        result = strcmp(
            ((*sa).name).as_ptr().cast_mut(),
            ((*sb).name).as_ptr().cast_mut(),
        );
    }
    if (*sort_crit).reversed != 0 {
        result = -result;
    }
    return result;
}
unsafe extern "C" fn sort_pane_cmp(
    mut a0: *const ::core::ffi::c_void,
    mut b0: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut sort_crit: *mut sort_criteria = sort_criteria;
    let mut a: *mut window_pane = *(a0 as *mut *mut window_pane);
    let mut b: *mut window_pane = *(b0 as *mut *mut window_pane);
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ai: u_int = 0;
    let mut bi: u_int = 0;
    match (*sort_crit).order as ::core::ffi::c_uint {
        0 => {
            result = (*a).active_point.wrapping_sub((*b).active_point) as ::core::ffi::c_int;
        }
        1 => {
            result = (*a).id.wrapping_sub((*b).id) as ::core::ffi::c_int;
        }
        6 => {
            result = (*a)
                .sx
                .wrapping_mul((*a).sy)
                .wrapping_sub((*b).sx.wrapping_mul((*b).sy))
                as ::core::ffi::c_int;
        }
        2 => {
            window_pane_index(a, &raw mut ai);
            window_pane_index(b, &raw mut bi);
            result = ai.wrapping_sub(bi) as ::core::ffi::c_int;
        }
        4 => {
            result = strcmp((*(*a).screen).title.as_ptr(), (*(*b).screen).title.as_ptr());
        }
        7 => {
            window_pane_zindex(a, &raw mut ai);
            window_pane_zindex(b, &raw mut bi);
            result = ai.wrapping_sub(bi) as ::core::ffi::c_int;
        }
        3 | 5 | 8 | _ => {}
    }
    if result == 0 as ::core::ffi::c_int {
        result = strcmp((*(*a).screen).title.as_ptr(), (*(*b).screen).title.as_ptr());
    }
    if (*sort_crit).reversed != 0 {
        result = -result;
    }
    return result;
}
unsafe extern "C" fn sort_winlink_cmp(
    mut a0: *const ::core::ffi::c_void,
    mut b0: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut sort_crit: *mut sort_criteria = sort_criteria;
    let mut a: *const *const winlink = a0 as *const *const winlink;
    let mut b: *const *const winlink = b0 as *const *const winlink;
    let mut wla: *const winlink = *a;
    let mut wlb: *const winlink = *b;
    let mut wa: *mut window = (*wla).window;
    let mut wb: *mut window = (*wlb).window;
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match (*sort_crit).order as ::core::ffi::c_uint {
        2 => {
            result = (*wla).idx - (*wlb).idx;
        }
        1 => {
            if if (*wa).creation_time.tv_sec == (*wb).creation_time.tv_sec {
                ((*wa).creation_time.tv_usec > (*wb).creation_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*wa).creation_time.tv_sec > (*wb).creation_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = 1 as ::core::ffi::c_int;
            } else if if (*wa).creation_time.tv_sec == (*wb).creation_time.tv_sec {
                ((*wa).creation_time.tv_usec < (*wb).creation_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*wa).creation_time.tv_sec < (*wb).creation_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = -(1 as ::core::ffi::c_int);
            }
        }
        0 => {
            if if (*wa).activity_time.tv_sec == (*wb).activity_time.tv_sec {
                ((*wa).activity_time.tv_usec > (*wb).activity_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*wa).activity_time.tv_sec > (*wb).activity_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = -(1 as ::core::ffi::c_int);
            } else if if (*wa).activity_time.tv_sec == (*wb).activity_time.tv_sec {
                ((*wa).activity_time.tv_usec < (*wb).activity_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*wa).activity_time.tv_sec < (*wb).activity_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = 1 as ::core::ffi::c_int;
            }
        }
        4 => {
            result = strcmp((*wa).name, (*wb).name);
        }
        6 => {
            result = (*wa)
                .sx
                .wrapping_mul((*wa).sy)
                .wrapping_sub((*wb).sx.wrapping_mul((*wb).sy))
                as ::core::ffi::c_int;
        }
        3 | 5 | 7 | 8 | _ => {}
    }
    if result == 0 as ::core::ffi::c_int {
        result = strcmp((*wa).name, (*wb).name);
    }
    if (*sort_crit).reversed != 0 {
        result = -result;
    }
    return result;
}
unsafe extern "C" fn sort_key_binding_cmp(
    mut a0: *const ::core::ffi::c_void,
    mut b0: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut sort_crit: *mut sort_criteria = sort_criteria;
    let mut a: *const key_binding = *(a0 as *mut *mut key_binding);
    let mut b: *const key_binding = *(b0 as *mut *mut key_binding);
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match (*sort_crit).order as ::core::ffi::c_uint {
        2 => {
            result = (*a).key.wrapping_sub((*b).key) as ::core::ffi::c_int;
        }
        3 => {
            result = ((*a).key as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS)
                .wrapping_sub((*b).key as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS)
                as ::core::ffi::c_int;
        }
        4 => {
            result = (strcasecmp((*a).tablename.as_ref().map_or(::core::ptr::null(), |s| s.as_ptr()), (*b).tablename.as_ref().map_or(::core::ptr::null(), |s| s.as_ptr())) == 0 as ::core::ffi::c_int)
                as ::core::ffi::c_int;
        }
        0 | 1 | 5 | 6 | 7 | 8 | _ => {}
    }
    if result == 0 as ::core::ffi::c_int {
        result = (strcasecmp((*a).tablename.as_ref().map_or(::core::ptr::null(), |s| s.as_ptr()), (*b).tablename.as_ref().map_or(::core::ptr::null(), |s| s.as_ptr())) == 0 as ::core::ffi::c_int)
            as ::core::ffi::c_int;
    }
    if (*sort_crit).reversed != 0 {
        result = -result;
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn sort_next_order(mut sort_crit: *mut sort_criteria) {
    let mut i: u_int = 0;
    if (*sort_crit).order_seq.is_null() {
        return;
    }
    i = 0 as u_int;
    while *(*sort_crit).order_seq.offset(i as isize) as ::core::ffi::c_uint
        != SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*sort_crit).order as ::core::ffi::c_uint
            == *(*sort_crit).order_seq.offset(i as isize) as ::core::ffi::c_uint
        {
            break;
        }
        i = i.wrapping_add(1);
    }
    if *(*sort_crit).order_seq.offset(i as isize) as ::core::ffi::c_uint
        == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        i = 0 as u_int;
    } else {
        i = i.wrapping_add(1);
        if *(*sort_crit).order_seq.offset(i as isize) as ::core::ffi::c_uint
            == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            i = 0 as u_int;
        }
    }
    (*sort_crit).order = *(*sort_crit).order_seq.offset(i as isize);
}
#[no_mangle]
pub unsafe extern "C" fn sort_order_from_string(
    mut order: *const ::core::ffi::c_char,
) -> sort_order {
    if !order.is_null() {
        if strcasecmp(
            order,
            b"activity\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            return SORT_ACTIVITY;
        }
        if strcasecmp(
            order,
            b"creation\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            return SORT_CREATION;
        }
        if strcasecmp(order, b"index\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
            || strcasecmp(order, b"key\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
        {
            return SORT_INDEX;
        }
        if strcasecmp(
            order,
            b"modifier\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            return SORT_MODIFIER;
        }
        if strcasecmp(order, b"name\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
            || strcasecmp(order, b"title\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
        {
            return SORT_NAME;
        }
        if strcasecmp(order, b"order\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            return SORT_ORDER;
        }
        if strcasecmp(order, b"size\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            return SORT_SIZE;
        }
        if strcasecmp(order, b"z\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            return SORT_Z;
        }
    }
    return SORT_END;
}
#[no_mangle]
pub unsafe extern "C" fn sort_order_to_string(mut order: sort_order) -> *const ::core::ffi::c_char {
    if order as ::core::ffi::c_uint == SORT_ACTIVITY as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"activity\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_CREATION as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"creation\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_INDEX as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"index\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_MODIFIER as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"modifier\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_NAME as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"name\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_ORDER as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"order\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_SIZE as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"size\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_Z as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"z\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn sort_would_window_tree_swap(
    mut sort_crit: *mut sort_criteria,
    mut wla: *mut winlink,
    mut wlb: *mut winlink,
) -> ::core::ffi::c_int {
    if (*sort_crit).order as ::core::ffi::c_uint
        == SORT_INDEX as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as ::core::ffi::c_int;
    }
    sort_criteria = sort_crit;
    return (sort_winlink_cmp(
        &raw mut wla as *const ::core::ffi::c_void,
        &raw mut wlb as *const ::core::ffi::c_void,
    ) != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
pub unsafe fn sort_get_buffers(sort_crit: *mut sort_criteria) -> Vec<*mut paste_buffer> {
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut buffers = Vec::new();
    loop {
        pb = paste_walk(pb);
        if pb.is_null() {
            break;
        }
        buffers.push(pb);
    }
    sort_qsort(
        buffers.as_mut_ptr() as *mut ::core::ffi::c_void,
        u_int::try_from(buffers.len()).expect("too many paste buffers to sort"),
        ::core::mem::size_of::<*mut paste_buffer>() as u_int,
        Some(
            sort_buffer_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        sort_crit,
    );
    buffers
}
pub unsafe fn sort_get_clients(sort_crit: *mut sort_criteria) -> Vec<*mut client> {
    let mut clients_sorted = Vec::new();
    let mut c = clients.first();
    while !c.is_null() {
        if !((*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
            if !(!(*c).flags & CLIENT_ATTACHED as uint64_t != 0) {
                clients_sorted.push(c);
            }
        }
        c = clients.next(c);
    }
    sort_qsort(
        clients_sorted.as_mut_ptr() as *mut ::core::ffi::c_void,
        u_int::try_from(clients_sorted.len()).expect("too many clients to sort"),
        ::core::mem::size_of::<*mut client>() as u_int,
        Some(
            sort_client_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        sort_crit,
    );
    clients_sorted
}
pub unsafe fn sort_get_sessions(sort_crit: *mut sort_criteria) -> Vec<*mut session> {
    let mut l = Vec::new();
    let mut s = sessions_minmax(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        l.push(s);
        s = sessions_next(s);
    }
    sort_qsort(
        l.as_mut_ptr() as *mut ::core::ffi::c_void,
        u_int::try_from(l.len()).expect("too many sessions to sort"),
        ::core::mem::size_of::<*mut session>() as u_int,
        Some(
            sort_session_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        sort_crit,
    );
    l
}
pub unsafe fn sort_get_panes_window(
    w: *mut window,
    sort_crit: *mut sort_criteria,
) -> Vec<*mut window_pane> {
    let mut panes = Vec::new();
    let mut wp = window_pane_first(w);
    while !wp.is_null() {
        panes.push(wp);
        wp = window_pane_next(wp);
    }
    sort_qsort(
        panes.as_mut_ptr() as *mut ::core::ffi::c_void,
        u_int::try_from(panes.len()).expect("too many panes to sort"),
        ::core::mem::size_of::<*mut window_pane>() as u_int,
        Some(
            sort_pane_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        sort_crit,
    );
    panes
}
pub unsafe fn sort_get_winlinks(sort_crit: *mut sort_criteria) -> Vec<*mut winlink> {
    let mut links = Vec::new();
    let mut s = sessions_minmax(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        let mut wl = winlinks_minmax(&raw mut (*s).windows, RB_NEGINF);
        while !wl.is_null() {
            links.push(wl);
            wl = winlinks_next(wl);
        }
        s = sessions_next(s);
    }
    sort_qsort(
        links.as_mut_ptr() as *mut ::core::ffi::c_void,
        u_int::try_from(links.len()).expect("too many winlinks to sort"),
        ::core::mem::size_of::<*mut winlink>() as u_int,
        Some(
            sort_winlink_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        sort_crit,
    );
    links
}
pub unsafe fn sort_get_winlinks_session(
    s: *mut session,
    sort_crit: *mut sort_criteria,
) -> Vec<*mut winlink> {
    let mut l = Vec::new();
    let mut wl = winlinks_minmax(&raw mut (*s).windows, RB_NEGINF);
    while !wl.is_null() {
        l.push(wl);
        wl = winlinks_next(wl);
    }
    sort_qsort(
        l.as_mut_ptr() as *mut ::core::ffi::c_void,
        u_int::try_from(l.len()).expect("too many winlinks to sort"),
        ::core::mem::size_of::<*mut winlink>() as u_int,
        Some(
            sort_winlink_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        sort_crit,
    );
    l
}
pub unsafe fn sort_get_key_bindings(sort_crit: *mut sort_criteria) -> Vec<*mut key_binding> {
    let mut bindings = Vec::new();
    let mut table = key_bindings_first_table();
    while !table.is_null() {
        let mut bd = key_bindings_first(table);
        while !bd.is_null() {
            bindings.push(bd);
            bd = key_bindings_next(table, bd);
        }
        table = key_bindings_next_table(table);
    }
    sort_qsort(
        bindings.as_mut_ptr() as *mut ::core::ffi::c_void,
        u_int::try_from(bindings.len()).expect("too many key bindings to sort"),
        ::core::mem::size_of::<*mut key_binding>() as u_int,
        Some(
            sort_key_binding_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        sort_crit,
    );
    bindings
}
pub unsafe fn sort_get_key_bindings_table(
    table: *mut key_table,
    sort_crit: *mut sort_criteria,
) -> Vec<*mut key_binding> {
    let mut bindings = Vec::new();
    if table.is_null() {
        return bindings;
    }
    let mut bd = key_bindings_first(table);
    while !bd.is_null() {
        bindings.push(bd);
        bd = key_bindings_next(table, bd);
    }
    sort_qsort(
        bindings.as_mut_ptr() as *mut ::core::ffi::c_void,
        u_int::try_from(bindings.len()).expect("too many key bindings to sort"),
        ::core::mem::size_of::<*mut key_binding>() as u_int,
        Some(
            sort_key_binding_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        sort_crit,
    );
    bindings
}
