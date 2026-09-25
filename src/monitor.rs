use crate::src::ffi::libc::{sscanf, strcmp};
use crate::src::format::{
    format_create, format_defaults, format_expand_cstring, format_free, format_true,
};
use crate::src::log::log_debug;
use crate::src::reactor::{event_add, event_del, event_initialized, event_pending, event_set};
use crate::src::server::current_time;
use crate::src::session::sessions;
use crate::src::session::{
    session_add_ref, session_find_by_id, session_remove_ref, sessions_minmax,
};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args;
use crate::src::shared::client::*;
use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files, overlay_check_cb,
    overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb, overlay_resize_cb,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
use crate::src::shared::display::{visible_range, visible_ranges};
use crate::src::shared::environment::environ;
use crate::src::shared::event::EV_TIMEOUT;
use crate::src::shared::format::FORMAT_NOJOBS;
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
use crate::src::shared::monitor::{
    monitor_cb, monitor_change, monitor_item, monitor_item_entry, monitor_items, monitor_pane,
    monitor_pane_entry, monitor_panes, monitor_set, monitor_window, monitor_window_entry,
    monitor_windows,
};
pub use crate::src::shared::monitor::{
    monitor_type, MONITOR_ALL_PANES, MONITOR_ALL_WINDOWS, MONITOR_NOTIFY_INITIAL,
    MONITOR_NOTIFY_TRUE, MONITOR_PANE, MONITOR_SESSION, MONITOR_WINDOW,
};
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::options::options;
use crate::src::shared::pane::{
    window_pane, window_pane_modes, window_pane_prompt, window_pane_tree_entry, window_panes,
};
use crate::src::shared::pane::{window_pane_offset, window_pane_resize, window_pane_resizes};
use crate::src::shared::process::tmuxpeer;
use crate::src::shared::prompt::prompt;
use crate::src::shared::redraw::redraw_scene;
use crate::src::shared::screen::{screen, screen_sel, screen_titles};
use crate::src::shared::screen_write::screen_write_cline;
use crate::src::shared::session::{session, session_entry};
use crate::src::shared::spawn::spawn_editor_state;
use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
use crate::src::shared::tree::{RB_BLACK, RB_NEGINF, RB_RED};
use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
use crate::src::shared::window::{
    window, window_entry, window_mode, window_mode_entry, window_winlinks, winlink, winlink_entry,
    winlink_stack, winlinks,
};
use crate::src::window::{
    window_find_by_id, window_pane_find_by_id, window_pane_first, window_pane_next,
    window_winlinks_first, window_winlinks_next, winlinks_minmax, winlinks_next,
};
use std::ffi::{CStr, CString};

use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

pub unsafe fn monitor_pane_new() -> *mut monitor_pane {
    Box::into_raw(Box::new(monitor_pane {
        last: None,
        ..monitor_pane::empty()
    }))
    .cast()
}

pub unsafe fn monitor_window_new() -> *mut monitor_window {
    Box::into_raw(Box::new(monitor_window {
        last: None,
        ..monitor_window::empty()
    }))
    .cast()
}

unsafe extern "C" fn monitor_get_session(mut ms: *mut monitor_set) -> *mut session {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    if !(*ms).client.is_null() {
        return (*(*ms).client).session;
    }
    s = (*ms).session;
    if s.is_null() {
        return sessions_minmax(&raw mut sessions, RB_NEGINF);
    }
    if session_find_by_id((*s).id) != s {
        return ::core::ptr::null_mut::<session>();
    }
    return s;
}
unsafe extern "C" fn monitor_create_formats(
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) -> *mut format_tree {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        0 as ::core::ffi::c_int,
        FORMAT_NOJOBS,
    );
    format_defaults(ft, c, s, wl, wp);
    return ft;
}
unsafe extern "C" fn monitor_item_cmp(
    mut m1: *mut monitor_item,
    mut m2: *mut monitor_item,
) -> ::core::ffi::c_int {
    return strcmp(
        ((*m1).name).as_ptr().cast_mut(),
        ((*m2).name).as_ptr().cast_mut(),
    );
}

unsafe extern "C" fn monitor_pane_cmp(
    mut mp1: *mut monitor_pane,
    mut mp2: *mut monitor_pane,
) -> ::core::ffi::c_int {
    if (*mp1).pane < (*mp2).pane {
        return -(1 as ::core::ffi::c_int);
    }
    if (*mp1).pane > (*mp2).pane {
        return 1 as ::core::ffi::c_int;
    }
    if (*mp1).idx < (*mp2).idx {
        return -(1 as ::core::ffi::c_int);
    }
    if (*mp1).idx > (*mp2).idx {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}

unsafe extern "C" fn monitor_window_cmp(
    mut mw1: *mut monitor_window,
    mut mw2: *mut monitor_window,
) -> ::core::ffi::c_int {
    if (*mw1).window < (*mw2).window {
        return -(1 as ::core::ffi::c_int);
    }
    if (*mw1).window > (*mw2).window {
        return 1 as ::core::ffi::c_int;
    }
    if (*mw1).idx < (*mw2).idx {
        return -(1 as ::core::ffi::c_int);
    }
    if (*mw1).idx > (*mw2).idx {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}

unsafe extern "C" fn monitor_free_item(mut ms: *mut monitor_set, mut me: *mut monitor_item) {
    let mut mp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut mp1: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut mw: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut mw1: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    mp = monitor_panes_minmax(&(*me).panes, RB_NEGINF);
    while !mp.is_null() && {
        mp1 = monitor_panes_next(&*mp);
        1 as ::core::ffi::c_int != 0
    } {
        monitor_panes_remove(&raw mut (*me).panes, mp);
        drop(Box::from_raw(mp));
        mp = mp1;
    }
    mw = monitor_windows_minmax(&(*me).windows, RB_NEGINF);
    while !mw.is_null() && {
        mw1 = monitor_windows_next(&*mw);
        1 as ::core::ffi::c_int != 0
    } {
        monitor_windows_remove(&raw mut (*me).windows, mw);
        drop(Box::from_raw(mw));
        mw = mw1;
    }
    monitor_items_remove(&raw mut (*ms).items, me);
    drop(Box::from_raw(me));
}
unsafe extern "C" fn monitor_report(
    mut ms: *mut monitor_set,
    mut me: *mut monitor_item,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
    mut value: *const ::core::ffi::c_char,
    mut last: *const ::core::ffi::c_char,
) {
    // A callback may remove the item while still using the change record.
    let name = CStr::from_ptr(((*me).name).as_ptr().cast_mut()).to_owned();
    let mut change: monitor_change = monitor_change {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        value: ::core::ptr::null::<::core::ffi::c_char>(),
        last: ::core::ptr::null::<::core::ffi::c_char>(),
        c: ::core::ptr::null_mut::<client>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
    };
    log_debug(
        b"%s: %s changed to %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"monitor_report\0" as *const u8 as *const ::core::ffi::c_char,
        name.as_ptr(),
        value,
    );
    (*me).fire_count = (*me).fire_count.wrapping_add(1);
    (*me).fire_time = current_time;
    change.name = name.as_ptr();
    change.value = value;
    change.last = last;
    change.c = (*ms).client;
    change.s = s;
    change.wl = wl;
    change.wp = wp;
    (*ms).cb.expect("non-null function pointer")(&raw mut change, (*ms).data);
}
unsafe fn monitor_check_value(
    mut ms: *mut monitor_set,
    mut me: *mut monitor_item,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
    value: &CStr,
    owned_last: *mut Option<CString>,
) {
    if (*owned_last).is_none() {
        let next = value.to_owned();
        *owned_last = Some(next);
        if (*me).flags & MONITOR_NOTIFY_INITIAL != 0
            && (!(*me).flags & MONITOR_NOTIFY_TRUE != 0 || format_true(value.as_ptr()) != 0)
        {
            monitor_report(
                ms,
                me,
                s,
                wl,
                wp,
                value.as_ptr(),
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
        }
        return;
    }
    if strcmp(value.as_ptr(), (*owned_last).as_ref().unwrap().as_ptr()) == 0 as ::core::ffi::c_int {
        return;
    }
    let notify = !(*me).flags & MONITOR_NOTIFY_TRUE != 0 || format_true(value.as_ptr()) != 0;
    let next = value.to_owned();
    let old = (*owned_last).replace(next);
    let previous = old.as_ref().expect("monitor last value existed");
    if notify {
        monitor_report(ms, me, s, wl, wp, value.as_ptr(), previous.as_ptr());
    }
}
unsafe extern "C" fn monitor_check_session(
    mut ms: *mut monitor_set,
    mut me: *mut monitor_item,
    mut ft: *mut format_tree,
) {
    let mut s: *mut session = monitor_get_session(ms);
    let value = format_expand_cstring(ft, ((*me).format).as_ptr());
    monitor_check_value(
        ms,
        me,
        s,
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
        &value,
        &raw mut (*me).last,
    );
}
unsafe extern "C" fn monitor_check_pane(mut ms: *mut monitor_set, mut me: *mut monitor_item) {
    let mut c: *mut client = (*ms).client;
    let mut s: *mut session = monitor_get_session(ms);
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut mp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut find: monitor_pane = monitor_pane {
        pane: 0,
        idx: 0,
        last: Default::default(),
        generation: 0,
        entry: monitor_pane_entry {
            owner: std::ptr::null_mut(),
        },
    };
    wp = window_pane_find_by_id((*me).id);
    if wp.is_null() || (*wp).fd == -(1 as ::core::ffi::c_int) {
        return;
    }
    w = (*wp).window as *mut window;
    wl = window_winlinks_first(w);
    while !wl.is_null() {
        if !((*wl).session != s) {
            ft = monitor_create_formats(c, s, wl, wp);
            let value = format_expand_cstring(ft, ((*me).format).as_ptr());
            format_free(ft);
            find.pane = (*wp).id;
            find.idx = (*wl).idx as u_int;
            mp = monitor_panes_find(&(*me).panes, &find);
            if mp.is_null() {
                mp = monitor_pane_new();
                (*mp).pane = (*wp).id;
                (*mp).idx = (*wl).idx as u_int;
                monitor_panes_insert(&raw mut (*me).panes, mp);
            }
            monitor_check_value(ms, me, s, wl, wp, &value, &raw mut (*mp).last);
        }
        wl = window_winlinks_next(w, wl);
    }
}
unsafe extern "C" fn monitor_check_all_panes_one(
    mut ms: *mut monitor_set,
    mut me: *mut monitor_item,
    mut ft: *mut format_tree,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) {
    let mut s: *mut session = monitor_get_session(ms);
    let mut mp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut find: monitor_pane = monitor_pane {
        pane: 0,
        idx: 0,
        last: Default::default(),
        generation: 0,
        entry: monitor_pane_entry {
            owner: std::ptr::null_mut(),
        },
    };
    let value = format_expand_cstring(ft, ((*me).format).as_ptr());
    find.pane = (*wp).id;
    find.idx = (*wl).idx as u_int;
    mp = monitor_panes_find(&(*me).panes, &find);
    if mp.is_null() {
        mp = monitor_pane_new();
        (*mp).pane = (*wp).id;
        (*mp).idx = (*wl).idx as u_int;
        monitor_panes_insert(&raw mut (*me).panes, mp);
    }
    (*mp).generation = (*ms).generation;
    monitor_check_value(ms, me, s, wl, wp, &value, &raw mut (*mp).last);
}
unsafe extern "C" fn monitor_sweep_all_panes(mut me: *mut monitor_item, mut generation: u_int) {
    let mut mp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut mp1: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    mp = monitor_panes_minmax(&(*me).panes, RB_NEGINF);
    while !mp.is_null() && {
        mp1 = monitor_panes_next(&*mp);
        1 as ::core::ffi::c_int != 0
    } {
        if !((*mp).generation == generation) {
            monitor_panes_remove(&raw mut (*me).panes, mp);
            drop(Box::from_raw(mp));
        }
        mp = mp1;
    }
}
unsafe extern "C" fn monitor_check_window(mut ms: *mut monitor_set, mut me: *mut monitor_item) {
    let mut c: *mut client = (*ms).client;
    let mut s: *mut session = monitor_get_session(ms);
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut mw: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut find: monitor_window = monitor_window {
        window: 0,
        idx: 0,
        last: Default::default(),
        generation: 0,
        entry: monitor_window_entry {
            owner: std::ptr::null_mut(),
        },
    };
    w = window_find_by_id((*me).id);
    if w.is_null() {
        return;
    }
    wl = window_winlinks_first(w);
    while !wl.is_null() {
        if !((*wl).session != s) {
            ft = monitor_create_formats(c, s, wl, ::core::ptr::null_mut::<window_pane>());
            let value = format_expand_cstring(ft, ((*me).format).as_ptr());
            format_free(ft);
            find.window = (*w).id;
            find.idx = (*wl).idx as u_int;
            mw = monitor_windows_find(&(*me).windows, &find);
            if mw.is_null() {
                mw = monitor_window_new();
                (*mw).window = (*w).id;
                (*mw).idx = (*wl).idx as u_int;
                monitor_windows_insert(&raw mut (*me).windows, mw);
            }
            monitor_check_value(
                ms,
                me,
                s,
                wl,
                ::core::ptr::null_mut::<window_pane>(),
                &value,
                &raw mut (*mw).last,
            );
        }
        wl = window_winlinks_next(w, wl);
    }
}
unsafe extern "C" fn monitor_check_all_windows_one(
    mut ms: *mut monitor_set,
    mut me: *mut monitor_item,
    mut ft: *mut format_tree,
    mut wl: *mut winlink,
) {
    let mut s: *mut session = monitor_get_session(ms);
    let mut w: *mut window = (*wl).window;
    let mut mw: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut find: monitor_window = monitor_window {
        window: 0,
        idx: 0,
        last: Default::default(),
        generation: 0,
        entry: monitor_window_entry {
            owner: std::ptr::null_mut(),
        },
    };
    let value = format_expand_cstring(ft, ((*me).format).as_ptr());
    find.window = (*w).id;
    find.idx = (*wl).idx as u_int;
    mw = monitor_windows_find(&(*me).windows, &find);
    if mw.is_null() {
        mw = monitor_window_new();
        (*mw).window = (*w).id;
        (*mw).idx = (*wl).idx as u_int;
        monitor_windows_insert(&raw mut (*me).windows, mw);
    }
    (*mw).generation = (*ms).generation;
    monitor_check_value(
        ms,
        me,
        s,
        wl,
        ::core::ptr::null_mut::<window_pane>(),
        &value,
        &raw mut (*mw).last,
    );
}
unsafe extern "C" fn monitor_sweep_all_windows(mut me: *mut monitor_item, mut generation: u_int) {
    let mut mw: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut mw1: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    mw = monitor_windows_minmax(&(*me).windows, RB_NEGINF);
    while !mw.is_null() && {
        mw1 = monitor_windows_next(&*mw);
        1 as ::core::ffi::c_int != 0
    } {
        if !((*mw).generation == generation) {
            monitor_windows_remove(&raw mut (*me).windows, mw);
            drop(Box::from_raw(mw));
        }
        mw = mw1;
    }
}
unsafe extern "C" fn monitor_check_sessions(mut ms: *mut monitor_set) {
    let mut c: *mut client = (*ms).client;
    let mut s: *mut session = monitor_get_session(ms);
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut me1: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    ft = monitor_create_formats(
        c,
        s,
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    me = monitor_items_minmax(&(*ms).items, RB_NEGINF);
    while !me.is_null() && {
        me1 = monitor_items_next(&*me);
        1 as ::core::ffi::c_int != 0
    } {
        if (*me).type_0 as ::core::ffi::c_uint
            == MONITOR_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            monitor_check_session(ms, me, ft);
        }
        me = me1;
    }
    format_free(ft);
}
unsafe extern "C" fn monitor_check_panes_windows(mut ms: *mut monitor_set) {
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut me1: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    me = monitor_items_minmax(&(*ms).items, RB_NEGINF);
    while !me.is_null() && {
        me1 = monitor_items_next(&*me);
        1 as ::core::ffi::c_int != 0
    } {
        match (*me).type_0 as ::core::ffi::c_uint {
            1 => {
                monitor_check_pane(ms, me);
            }
            3 => {
                monitor_check_window(ms, me);
            }
            0 | 2 | 4 | _ => {}
        }
        me = me1;
    }
}
unsafe extern "C" fn monitor_check_all_panes(mut ms: *mut monitor_set) {
    let mut c: *mut client = (*ms).client;
    let mut s: *mut session = monitor_get_session(ms);
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut me1: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    (*ms).generation = (*ms).generation.wrapping_add(1);
    if (*ms).generation == 0 as u_int {
        (*ms).generation = 1 as u_int;
    }
    wl = winlinks_minmax(&raw mut (*s).windows, RB_NEGINF);
    while !wl.is_null() {
        wp = window_pane_first((*wl).window);
        while !wp.is_null() {
            ft = monitor_create_formats(c, s, wl, wp);
            me = monitor_items_minmax(&(*ms).items, RB_NEGINF);
            while !me.is_null() && {
                me1 = monitor_items_next(&*me);
                1 as ::core::ffi::c_int != 0
            } {
                if !((*me).type_0 as ::core::ffi::c_uint
                    != MONITOR_ALL_PANES as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    monitor_check_all_panes_one(ms, me, ft, wl, wp);
                }
                me = me1;
            }
            format_free(ft);
            wp = window_pane_next(wp);
        }
        wl = winlinks_next(wl);
    }
    me = monitor_items_minmax(&(*ms).items, RB_NEGINF);
    while !me.is_null() && {
        me1 = monitor_items_next(&*me);
        1 as ::core::ffi::c_int != 0
    } {
        if (*me).type_0 as ::core::ffi::c_uint
            == MONITOR_ALL_PANES as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            monitor_sweep_all_panes(me, (*ms).generation);
        }
        me = me1;
    }
}
unsafe extern "C" fn monitor_check_all_windows(mut ms: *mut monitor_set) {
    let mut c: *mut client = (*ms).client;
    let mut s: *mut session = monitor_get_session(ms);
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut me1: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    (*ms).generation = (*ms).generation.wrapping_add(1);
    if (*ms).generation == 0 as u_int {
        (*ms).generation = 1 as u_int;
    }
    wl = winlinks_minmax(&raw mut (*s).windows, RB_NEGINF);
    while !wl.is_null() {
        ft = monitor_create_formats(c, s, wl, ::core::ptr::null_mut::<window_pane>());
        me = monitor_items_minmax(&(*ms).items, RB_NEGINF);
        while !me.is_null() && {
            me1 = monitor_items_next(&*me);
            1 as ::core::ffi::c_int != 0
        } {
            if !((*me).type_0 as ::core::ffi::c_uint
                != MONITOR_ALL_WINDOWS as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                monitor_check_all_windows_one(ms, me, ft, wl);
            }
            me = me1;
        }
        format_free(ft);
        wl = winlinks_next(wl);
    }
    me = monitor_items_minmax(&(*ms).items, RB_NEGINF);
    while !me.is_null() && {
        me1 = monitor_items_next(&*me);
        1 as ::core::ffi::c_int != 0
    } {
        if (*me).type_0 as ::core::ffi::c_uint
            == MONITOR_ALL_WINDOWS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            monitor_sweep_all_windows(me, (*ms).generation);
        }
        me = me1;
    }
}
unsafe extern "C" fn monitor_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut ms: *mut monitor_set = data as *mut monitor_set;
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut tv: timeval = timeval {
        tv_sec: 1 as __time_t,
        tv_usec: 0,
    };
    let mut have_session: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut have_all_panes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut have_all_windows: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    log_debug(
        b"%s: timer fired\0" as *const u8 as *const ::core::ffi::c_char,
        b"monitor_timer\0" as *const u8 as *const ::core::ffi::c_char,
    );
    event_add(&raw mut (*ms).timer, &raw mut tv);
    if monitor_get_session(ms).is_null() {
        return;
    }
    me = monitor_items_minmax(&(*ms).items, RB_NEGINF);
    while !me.is_null() {
        match (*me).type_0 as ::core::ffi::c_uint {
            0 => {
                have_session = 1 as ::core::ffi::c_int;
            }
            2 => {
                have_all_panes = 1 as ::core::ffi::c_int;
            }
            4 => {
                have_all_windows = 1 as ::core::ffi::c_int;
            }
            1 | 3 | _ => {}
        }
        me = monitor_items_next(&*me);
    }
    if have_session != 0 {
        monitor_check_sessions(ms);
    }
    monitor_check_panes_windows(ms);
    if have_all_panes != 0 {
        monitor_check_all_panes(ms);
    }
    if have_all_windows != 0 {
        monitor_check_all_windows(ms);
    }
}
unsafe extern "C" fn monitor_create(
    mut cb: monitor_cb,
    mut data: *mut ::core::ffi::c_void,
) -> *mut monitor_set {
    let mut ms: *mut monitor_set = ::core::ptr::null_mut::<monitor_set>();
    ms = Box::into_raw(Box::new(::core::mem::zeroed::<monitor_set>()));
    (*ms).cb = cb;
    (*ms).data = data;
    (*ms).items.storage = std::ptr::null_mut();
    return ms;
}
#[no_mangle]
pub unsafe extern "C" fn monitor_create_client(
    mut c: *mut client,
    mut cb: monitor_cb,
    mut data: *mut ::core::ffi::c_void,
) -> *mut monitor_set {
    let mut ms: *mut monitor_set = ::core::ptr::null_mut::<monitor_set>();
    ms = monitor_create(cb, data);
    (*ms).client = c;
    return ms as *mut monitor_set;
}
#[no_mangle]
pub unsafe extern "C" fn monitor_create_session(
    mut s: *mut session,
    mut cb: monitor_cb,
    mut data: *mut ::core::ffi::c_void,
) -> *mut monitor_set {
    let mut ms: *mut monitor_set = ::core::ptr::null_mut::<monitor_set>();
    ms = monitor_create(cb, data);
    (*ms).session = s;
    if !s.is_null() {
        session_add_ref(
            s,
            b"monitor_create_session\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    return ms;
}
#[no_mangle]
pub unsafe extern "C" fn monitor_destroy(mut ms: *mut monitor_set) {
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut me1: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    if !ms.is_null() {
        if event_initialized(&(*ms).timer) != 0 {
            event_del(&raw mut (*ms).timer);
        }
        me = monitor_items_minmax(&(*ms).items, RB_NEGINF);
        while !me.is_null() && {
            me1 = monitor_items_next(&*me);
            1 as ::core::ffi::c_int != 0
        } {
            monitor_free_item(ms, me);
            me = me1;
        }
        if !(*ms).session.is_null() {
            session_remove_ref(
                (*ms).session,
                b"monitor_destroy\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        drop(Box::from_raw(ms));
    }
}
pub struct ParsedMonitor {
    pub name: CString,
    pub type_0: monitor_type,
    pub id: ::core::ffi::c_int,
    pub format: CString,
}

unsafe fn monitor_parse_parts(value: &CStr) -> Option<ParsedMonitor> {
    let value_bytes = value.to_bytes();
    let mut type_0;
    let mut id = -1;
    let Some(first_colon) = value_bytes.iter().position(|&byte| byte == b':') else {
        return None;
    };
    let target_start = first_colon + 1;
    let target_bytes = &value_bytes[target_start..];
    let Some(second_colon) = target_bytes.iter().position(|&byte| byte == b':') else {
        return None;
    };
    let target_bytes = &target_bytes[..second_colon];
    let format_start = target_start + second_colon + 1;

    if target_bytes == b"%*" {
        type_0 = MONITOR_ALL_PANES;
    } else if target_bytes == b"@*" {
        type_0 = MONITOR_ALL_WINDOWS;
    } else if target_bytes.is_empty() {
        type_0 = MONITOR_SESSION;
    } else {
        let target = CString::new(target_bytes).expect("monitor target contains no NUL");
        if sscanf(
            target.as_ptr(),
            b"%%%d\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut id,
        ) == 1 as ::core::ffi::c_int
            && id >= 0 as ::core::ffi::c_int
        {
            type_0 = MONITOR_PANE;
        } else if sscanf(
            target.as_ptr(),
            b"@%d\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut id,
        ) == 1 as ::core::ffi::c_int
            && id >= 0 as ::core::ffi::c_int
        {
            type_0 = MONITOR_WINDOW;
        } else {
            return None;
        }
    }

    Some(ParsedMonitor {
        name: CString::new(&value_bytes[..first_colon]).expect("monitor name contains no NUL"),
        type_0,
        id,
        format: CString::new(&value_bytes[format_start..]).expect("monitor format contains no NUL"),
    })
}

pub fn monitor_parse_owned(value: &CStr) -> Option<ParsedMonitor> {
    unsafe { monitor_parse_parts(value) }
}

#[no_mangle]
pub unsafe extern "C" fn monitor_add(
    mut ms: *mut monitor_set,
    mut name: *const ::core::ffi::c_char,
    mut type_0: monitor_type,
    mut id: ::core::ffi::c_int,
    mut format: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) {
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    // Either input may borrow the item being replaced.
    let owned_name = CStr::from_ptr(name).to_owned();
    let owned_format = CStr::from_ptr(format).to_owned();
    let mut find: monitor_item = monitor_item {
        name: ::std::ffi::CStr::from_ptr(owned_name.as_ptr() as *mut ::core::ffi::c_char)
            .to_owned(),
        format: Default::default(),
        type_0: MONITOR_SESSION,
        id: 0,
        flags: 0,
        last: Default::default(),
        panes: monitor_panes {
            storage: std::ptr::null_mut(),
        },
        windows: monitor_windows {
            storage: std::ptr::null_mut(),
        },
        fire_count: 0,
        fire_time: 0,
        entry: monitor_item_entry {
            owner: std::ptr::null_mut(),
        },
    };
    let mut tv: timeval = timeval {
        tv_sec: 1 as __time_t,
        tv_usec: 0,
    };
    me = monitor_items_find(&(*ms).items, &find);
    if !me.is_null() {
        monitor_free_item(ms, me);
    }
    let mut owner = Box::new(monitor_item {
        name: owned_name,
        format: owned_format,
        last: None,
        ..monitor_item::empty()
    });

    me = Box::into_raw(owner).cast::<monitor_item>();
    (*me).type_0 = type_0;
    (*me).id = id as u_int;
    (*me).flags = flags;
    (*me).panes.storage = std::ptr::null_mut();
    (*me).windows.storage = std::ptr::null_mut();
    monitor_items_insert(&raw mut (*ms).items, me);
    if event_initialized(&(*ms).timer) == 0 {
        event_set(
            &raw mut (*ms).timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            Some(
                monitor_timer
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            ms as *mut ::core::ffi::c_void,
        );
    }
    if event_pending(
        &raw mut (*ms).timer,
        EV_TIMEOUT as ::core::ffi::c_short,
        ::core::ptr::null_mut::<timeval>(),
    ) == 0
    {
        event_add(&raw mut (*ms).timer, &raw mut tv);
    }
}
#[no_mangle]
pub unsafe extern "C" fn monitor_remove(
    mut ms: *mut monitor_set,
    mut name: *const ::core::ffi::c_char,
) {
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut find: monitor_item = monitor_item {
        name: ::std::ffi::CStr::from_ptr(name as *mut ::core::ffi::c_char).to_owned(),
        format: Default::default(),
        type_0: MONITOR_SESSION,
        id: 0,
        flags: 0,
        last: Default::default(),
        panes: monitor_panes {
            storage: std::ptr::null_mut(),
        },
        windows: monitor_windows {
            storage: std::ptr::null_mut(),
        },
        fire_count: 0,
        fire_time: 0,
        entry: monitor_item_entry {
            owner: std::ptr::null_mut(),
        },
    };
    me = monitor_items_find(&(*ms).items, &find);
    if !me.is_null() {
        monitor_free_item(ms, me);
    }
    if (*ms).items.storage.is_null() && event_initialized(&(*ms).timer) != 0 {
        event_del(&raw mut (*ms).timer);
    }
}
#[no_mangle]
pub unsafe extern "C" fn monitor_get_fire_count(
    mut ms: *mut monitor_set,
    mut name: *const ::core::ffi::c_char,
) -> u_int {
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut find: monitor_item = monitor_item {
        name: ::std::ffi::CStr::from_ptr(name as *mut ::core::ffi::c_char).to_owned(),
        format: Default::default(),
        type_0: MONITOR_SESSION,
        id: 0,
        flags: 0,
        last: Default::default(),
        panes: monitor_panes {
            storage: std::ptr::null_mut(),
        },
        windows: monitor_windows {
            storage: std::ptr::null_mut(),
        },
        fire_count: 0,
        fire_time: 0,
        entry: monitor_item_entry {
            owner: std::ptr::null_mut(),
        },
    };
    me = monitor_items_find(&(*ms).items, &find);
    if me.is_null() {
        return 0 as u_int;
    }
    return (*me).fire_count;
}
#[no_mangle]
pub unsafe extern "C" fn monitor_get_fire_time(
    mut ms: *mut monitor_set,
    mut name: *const ::core::ffi::c_char,
) -> time_t {
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut find: monitor_item = monitor_item {
        name: ::std::ffi::CStr::from_ptr(name as *mut ::core::ffi::c_char).to_owned(),
        format: Default::default(),
        type_0: MONITOR_SESSION,
        id: 0,
        flags: 0,
        last: Default::default(),
        panes: monitor_panes {
            storage: std::ptr::null_mut(),
        },
        windows: monitor_windows {
            storage: std::ptr::null_mut(),
        },
        fire_count: 0,
        fire_time: 0,
        entry: monitor_item_entry {
            owner: std::ptr::null_mut(),
        },
    };
    me = monitor_items_find(&(*ms).items, &find);
    if me.is_null() {
        return 0 as time_t;
    }
    return (*me).fire_time;
}

pub unsafe fn monitor_items_find(
    head: &monitor_items,
    elm: &monitor_item,
) -> *mut monitor_item {
    let Some(map) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = elm.name.as_c_str().to_bytes();
    map.get(key).copied().unwrap_or(std::ptr::null_mut())
}
pub unsafe fn monitor_items_nfind(
    head: &monitor_items,
    elm: &monitor_item,
) -> *mut monitor_item {
    let Some(map) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = elm.name.as_c_str().to_bytes();
    map.range::<[u8], _>((std::ops::Bound::Included(key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn monitor_items_insert(
    head: *mut monitor_items,
    elm: *mut monitor_item,
) -> *mut monitor_item {
    let key = std::ffi::CStr::from_ptr(((*elm).name).as_ptr().cast_mut()).to_bytes();
    if (*head).storage.is_null() {
        (*head).storage = Box::into_raw(Box::new(std::collections::BTreeMap::new()));
    }
    let map = &mut *(*head).storage;
    match map.entry(key.to_vec()) {
        std::collections::btree_map::Entry::Occupied(entry) => return *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
        }
    }
    (*elm).entry.owner = map as *mut _;
    std::ptr::null_mut()
}
pub unsafe fn monitor_items_remove(
    head: *mut monitor_items,
    elm: *mut monitor_item,
) -> *mut monitor_item {
    if elm.is_null() {
        return std::ptr::null_mut();
    }
    let key = std::ffi::CStr::from_ptr(((*elm).name).as_ptr().cast_mut()).to_bytes();
    let Some(map) = (*head).storage.as_mut() else {
        return std::ptr::null_mut();
    };
    if map.get(key).copied() != Some(elm) {
        return std::ptr::null_mut();
    }
    map.remove(key);
    (*elm).entry.owner = std::ptr::null_mut();
    if map.is_empty() {
        drop(Box::from_raw((*head).storage));
        (*head).storage = std::ptr::null_mut();
    }
    elm
}
pub unsafe fn monitor_items_minmax(
    head: &monitor_items,
    direction: ::core::ffi::c_int,
) -> *mut monitor_item {
    let Some(map) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let pair = if direction < 0 {
        map.first_key_value()
    } else {
        map.last_key_value()
    };
    pair.map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn monitor_items_next(elm: &monitor_item) -> *mut monitor_item {
    let Some(map) = elm.entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = std::ffi::CStr::from_ptr(elm.name.as_ptr().cast_mut()).to_bytes();
    map.range::<[u8], _>((std::ops::Bound::Excluded(key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn monitor_items_prev(elm: &monitor_item) -> *mut monitor_item {
    let Some(map) = elm.entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = std::ffi::CStr::from_ptr(elm.name.as_ptr().cast_mut()).to_bytes();
    map.range::<[u8], _>((std::ops::Bound::Unbounded, std::ops::Bound::Excluded(key)))
        .next_back()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}

fn monitor_panes_key(elm: &monitor_pane) -> (u32, u32) {
    (elm.pane, elm.idx)
}
pub unsafe fn monitor_panes_find(
    head: &monitor_panes,
    elm: &monitor_pane,
) -> *mut monitor_pane {
    let Some(map) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = monitor_panes_key(elm);
    map.get(&key).copied().unwrap_or(std::ptr::null_mut())
}
pub unsafe fn monitor_panes_nfind(
    head: &monitor_panes,
    elm: &monitor_pane,
) -> *mut monitor_pane {
    let Some(map) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = monitor_panes_key(elm);
    map.range((std::ops::Bound::Included(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn monitor_panes_insert(
    head: *mut monitor_panes,
    elm: *mut monitor_pane,
) -> *mut monitor_pane {
    let key = monitor_panes_key(&*elm);
    if (*head).storage.is_null() {
        (*head).storage = Box::into_raw(Box::new(std::collections::BTreeMap::new()));
    }
    let map = &mut *(*head).storage;
    match map.entry(key) {
        std::collections::btree_map::Entry::Occupied(entry) => return *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
        }
    }
    (*elm).entry.owner = map as *mut _;
    std::ptr::null_mut()
}
pub unsafe fn monitor_panes_remove(
    head: *mut monitor_panes,
    elm: *mut monitor_pane,
) -> *mut monitor_pane {
    if elm.is_null() {
        return std::ptr::null_mut();
    }
    let key = monitor_panes_key(&*elm);
    let Some(map) = (*head).storage.as_mut() else {
        return std::ptr::null_mut();
    };
    if map.get(&key).copied() != Some(elm) {
        return std::ptr::null_mut();
    }
    map.remove(&key);
    (*elm).entry.owner = std::ptr::null_mut();
    if map.is_empty() {
        drop(Box::from_raw((*head).storage));
        (*head).storage = std::ptr::null_mut();
    }
    elm
}
pub unsafe fn monitor_panes_minmax(
    head: &monitor_panes,
    direction: ::core::ffi::c_int,
) -> *mut monitor_pane {
    let Some(map) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let pair = if direction < 0 {
        map.first_key_value()
    } else {
        map.last_key_value()
    };
    pair.map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn monitor_panes_next(elm: &monitor_pane) -> *mut monitor_pane {
    let Some(map) = elm.entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = monitor_panes_key(elm);
    map.range((std::ops::Bound::Excluded(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn monitor_panes_prev(elm: &monitor_pane) -> *mut monitor_pane {
    let Some(map) = elm.entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = monitor_panes_key(elm);
    map.range((std::ops::Bound::Unbounded, std::ops::Bound::Excluded(&key)))
        .next_back()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}

fn monitor_windows_key(elm: &monitor_window) -> (u32, u32) {
    (elm.window, elm.idx)
}
pub unsafe fn monitor_windows_find(
    head: &monitor_windows,
    elm: &monitor_window,
) -> *mut monitor_window {
    let Some(map) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = monitor_windows_key(elm);
    map.get(&key).copied().unwrap_or(std::ptr::null_mut())
}
pub unsafe fn monitor_windows_nfind(
    head: &monitor_windows,
    elm: &monitor_window,
) -> *mut monitor_window {
    let Some(map) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = monitor_windows_key(elm);
    map.range((std::ops::Bound::Included(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn monitor_windows_insert(
    head: *mut monitor_windows,
    elm: *mut monitor_window,
) -> *mut monitor_window {
    let key = monitor_windows_key(&*elm);
    if (*head).storage.is_null() {
        (*head).storage = Box::into_raw(Box::new(std::collections::BTreeMap::new()));
    }
    let map = &mut *(*head).storage;
    match map.entry(key) {
        std::collections::btree_map::Entry::Occupied(entry) => return *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
        }
    }
    (*elm).entry.owner = map as *mut _;
    std::ptr::null_mut()
}
pub unsafe fn monitor_windows_remove(
    head: *mut monitor_windows,
    elm: *mut monitor_window,
) -> *mut monitor_window {
    if elm.is_null() {
        return std::ptr::null_mut();
    }
    let key = monitor_windows_key(&*elm);
    let Some(map) = (*head).storage.as_mut() else {
        return std::ptr::null_mut();
    };
    if map.get(&key).copied() != Some(elm) {
        return std::ptr::null_mut();
    }
    map.remove(&key);
    (*elm).entry.owner = std::ptr::null_mut();
    if map.is_empty() {
        drop(Box::from_raw((*head).storage));
        (*head).storage = std::ptr::null_mut();
    }
    elm
}
pub unsafe fn monitor_windows_minmax(
    head: &monitor_windows,
    direction: ::core::ffi::c_int,
) -> *mut monitor_window {
    let Some(map) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let pair = if direction < 0 {
        map.first_key_value()
    } else {
        map.last_key_value()
    };
    pair.map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn monitor_windows_next(elm: &monitor_window) -> *mut monitor_window {
    let Some(map) = elm.entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = monitor_windows_key(elm);
    map.range((std::ops::Bound::Excluded(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn monitor_windows_prev(elm: &monitor_window) -> *mut monitor_window {
    let Some(map) = elm.entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = monitor_windows_key(elm);
    map.range((std::ops::Bound::Unbounded, std::ops::Bound::Excluded(&key)))
        .next_back()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}

#[cfg(test)]
mod last_owner_tests {
    use super::*;

    struct Capture {
        set: *mut monitor_set,
        name: Vec<u8>,
        value: Vec<u8>,
        last: Vec<u8>,
    }

    unsafe extern "C" fn remove_during_report(
        change: *mut monitor_change,
        data: *mut ::core::ffi::c_void,
    ) {
        let capture = &mut *data.cast::<Capture>();
        capture.value = CStr::from_ptr((*change).value).to_bytes().to_vec();
        capture.last = CStr::from_ptr((*change).last).to_bytes().to_vec();
        monitor_remove(capture.set, (*change).name);
        capture.name = CStr::from_ptr((*change).name).to_bytes().to_vec();
    }

    #[test]
    fn changed_value_survives_reentrant_item_removal() {
        unsafe {
            let mut capture = Box::new(Capture {
                set: ::core::ptr::null_mut(),
                name: Vec::new(),
                value: Vec::new(),
                last: Vec::new(),
            });
            let set = monitor_create_client(
                ::core::ptr::null_mut(),
                Some(remove_during_report),
                (&raw mut *capture).cast(),
            );
            capture.set = set;
            monitor_add(
                set,
                c"reentrant-last".as_ptr(),
                MONITOR_SESSION,
                -1,
                c"value".as_ptr(),
                0,
            );
            let item = monitor_items_minmax(&(*set).items, RB_NEGINF);
            let owner = item;
            let first = CString::from_vec_with_nul(b"\xffold\0".to_vec()).unwrap();
            monitor_check_value(
                set,
                item,
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                &first,
                &raw mut (*owner).last,
            );
            assert_eq!(
                CStr::from_ptr(
                    ((*item).last)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                )
                .to_bytes(),
                first.to_bytes()
            );
            monitor_check_value(
                set,
                item,
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                &first,
                &raw mut (*owner).last,
            );
            assert!(capture.value.is_empty());

            let second = CString::from_vec_with_nul(b"\xfeneW\0".to_vec()).unwrap();
            monitor_check_value(
                set,
                item,
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                &second,
                &raw mut (*owner).last,
            );
            assert_eq!(capture.value, second.to_bytes());
            assert_eq!(capture.last, first.to_bytes());
            assert_eq!(capture.name, b"reentrant-last");
            assert!(monitor_items_minmax(&(*set).items, RB_NEGINF).is_null());
            monitor_destroy(set);
        }
    }
}
