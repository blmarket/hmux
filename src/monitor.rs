use crate::src::ffi::libc::{free, sscanf, strchr, strcmp};
use crate::src::format::{format_create, format_defaults, format_expand, format_free, format_true};
use crate::src::log::log_debug;
use crate::src::reactor::{event_add, event_del, event_initialized, event_pending, event_set};
use crate::src::server::current_time;
pub use crate::src::session::sessions;
use crate::src::session::{
    session_add_ref, session_find_by_id, session_remove_ref, sessions_RB_MINMAX,
};
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
pub use crate::src::shared::event::EV_TIMEOUT;
pub use crate::src::shared::format::FORMAT_NOJOBS;
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
pub use crate::src::shared::monitor::{
    monitor_cb, monitor_change, monitor_item, monitor_item_entry, monitor_items, monitor_pane,
    monitor_pane_entry, monitor_panes, monitor_set, monitor_window, monitor_window_entry,
    monitor_windows,
};
pub use crate::src::shared::monitor::{
    monitor_type, MONITOR_ALL_PANES, MONITOR_ALL_WINDOWS, MONITOR_NOTIFY_INITIAL,
    MONITOR_NOTIFY_TRUE, MONITOR_PANE, MONITOR_SESSION, MONITOR_WINDOW,
};
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
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tree::{RB_BLACK, RB_NEGINF, RB_RED};
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::window::{
    window_find_by_id, window_pane_find_by_id, winlinks_minmax, winlinks_next,
};
use crate::src::xmalloc::{xcalloc, xstrdup};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

unsafe extern "C" fn monitor_get_session(mut ms: *mut monitor_set) -> *mut session {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    if !(*ms).client.is_null() {
        return (*(*ms).client).session;
    }
    s = (*ms).session;
    if s.is_null() {
        return sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
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
    return strcmp((*m1).name, (*m2).name);
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
    mp = monitor_panes_minmax(&raw mut (*me).panes, RB_NEGINF);
    while !mp.is_null() && {
        mp1 = monitor_panes_next(mp);
        1 as ::core::ffi::c_int != 0
    } {
        monitor_panes_remove(&raw mut (*me).panes, mp);
        free((*mp).last as *mut ::core::ffi::c_void);
        free(mp as *mut ::core::ffi::c_void);
        mp = mp1;
    }
    mw = monitor_windows_minmax(&raw mut (*me).windows, RB_NEGINF);
    while !mw.is_null() && {
        mw1 = monitor_windows_next(mw);
        1 as ::core::ffi::c_int != 0
    } {
        monitor_windows_remove(&raw mut (*me).windows, mw);
        free((*mw).last as *mut ::core::ffi::c_void);
        free(mw as *mut ::core::ffi::c_void);
        mw = mw1;
    }
    free((*me).last as *mut ::core::ffi::c_void);
    monitor_items_remove(&raw mut (*ms).items, me);
    free((*me).name as *mut ::core::ffi::c_void);
    free((*me).format as *mut ::core::ffi::c_void);
    free(me as *mut ::core::ffi::c_void);
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
        (*me).name,
        value,
    );
    (*me).fire_count = (*me).fire_count.wrapping_add(1);
    (*me).fire_time = current_time;
    change.name = (*me).name;
    change.value = value;
    change.last = last;
    change.c = (*ms).client;
    change.s = s;
    change.wl = wl;
    change.wp = wp;
    (*ms).cb.expect("non-null function pointer")(&raw mut change, (*ms).data);
}
unsafe extern "C" fn monitor_check_value(
    mut ms: *mut monitor_set,
    mut me: *mut monitor_item,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
    mut value: *mut ::core::ffi::c_char,
    mut last: *mut *mut ::core::ffi::c_char,
) {
    if (*last).is_null() {
        *last = value;
        if (*me).flags & MONITOR_NOTIFY_INITIAL != 0
            && (!(*me).flags & MONITOR_NOTIFY_TRUE != 0 || format_true(value) != 0)
        {
            monitor_report(
                ms,
                me,
                s,
                wl,
                wp,
                value,
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
        }
        return;
    }
    if strcmp(value, *last) == 0 as ::core::ffi::c_int {
        free(value as *mut ::core::ffi::c_void);
        return;
    }
    if !(*me).flags & MONITOR_NOTIFY_TRUE != 0 || format_true(value) != 0 {
        monitor_report(ms, me, s, wl, wp, value, *last);
    }
    free(*last as *mut ::core::ffi::c_void);
    *last = value;
}
unsafe extern "C" fn monitor_check_session(
    mut ms: *mut monitor_set,
    mut me: *mut monitor_item,
    mut ft: *mut format_tree,
) {
    let mut s: *mut session = monitor_get_session(ms);
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    value = format_expand(ft, (*me).format);
    monitor_check_value(
        ms,
        me,
        s,
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
        value,
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
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut mp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut find: monitor_pane = monitor_pane {
        pane: 0,
        idx: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
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
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        if !((*wl).session != s) {
            ft = monitor_create_formats(c, s, wl, wp);
            value = format_expand(ft, (*me).format);
            format_free(ft);
            find.pane = (*wp).id;
            find.idx = (*wl).idx as u_int;
            mp = monitor_panes_find(&raw mut (*me).panes, &raw mut find);
            if mp.is_null() {
                mp = xcalloc(
                    1 as size_t,
                    ::core::mem::size_of::<monitor_pane>() as size_t,
                ) as *mut monitor_pane;
                (*mp).pane = (*wp).id;
                (*mp).idx = (*wl).idx as u_int;
                monitor_panes_insert(&raw mut (*me).panes, mp);
            }
            monitor_check_value(ms, me, s, wl, wp, value, &raw mut (*mp).last);
        }
        wl = (*wl).wentry.tqe_next;
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
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut mp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut find: monitor_pane = monitor_pane {
        pane: 0,
        idx: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        generation: 0,
        entry: monitor_pane_entry {
            owner: std::ptr::null_mut(),
        },
    };
    value = format_expand(ft, (*me).format);
    find.pane = (*wp).id;
    find.idx = (*wl).idx as u_int;
    mp = monitor_panes_find(&raw mut (*me).panes, &raw mut find);
    if mp.is_null() {
        mp = xcalloc(
            1 as size_t,
            ::core::mem::size_of::<monitor_pane>() as size_t,
        ) as *mut monitor_pane;
        (*mp).pane = (*wp).id;
        (*mp).idx = (*wl).idx as u_int;
        monitor_panes_insert(&raw mut (*me).panes, mp);
    }
    (*mp).generation = (*ms).generation;
    monitor_check_value(ms, me, s, wl, wp, value, &raw mut (*mp).last);
}
unsafe extern "C" fn monitor_sweep_all_panes(mut me: *mut monitor_item, mut generation: u_int) {
    let mut mp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut mp1: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    mp = monitor_panes_minmax(&raw mut (*me).panes, RB_NEGINF);
    while !mp.is_null() && {
        mp1 = monitor_panes_next(mp);
        1 as ::core::ffi::c_int != 0
    } {
        if !((*mp).generation == generation) {
            monitor_panes_remove(&raw mut (*me).panes, mp);
            free((*mp).last as *mut ::core::ffi::c_void);
            free(mp as *mut ::core::ffi::c_void);
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
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut mw: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut find: monitor_window = monitor_window {
        window: 0,
        idx: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        generation: 0,
        entry: monitor_window_entry {
            owner: std::ptr::null_mut(),
        },
    };
    w = window_find_by_id((*me).id);
    if w.is_null() {
        return;
    }
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        if !((*wl).session != s) {
            ft = monitor_create_formats(c, s, wl, ::core::ptr::null_mut::<window_pane>());
            value = format_expand(ft, (*me).format);
            format_free(ft);
            find.window = (*w).id;
            find.idx = (*wl).idx as u_int;
            mw = monitor_windows_find(&raw mut (*me).windows, &raw mut find);
            if mw.is_null() {
                mw = xcalloc(
                    1 as size_t,
                    ::core::mem::size_of::<monitor_window>() as size_t,
                ) as *mut monitor_window;
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
                value,
                &raw mut (*mw).last,
            );
        }
        wl = (*wl).wentry.tqe_next;
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
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut mw: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut find: monitor_window = monitor_window {
        window: 0,
        idx: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        generation: 0,
        entry: monitor_window_entry {
            owner: std::ptr::null_mut(),
        },
    };
    value = format_expand(ft, (*me).format);
    find.window = (*w).id;
    find.idx = (*wl).idx as u_int;
    mw = monitor_windows_find(&raw mut (*me).windows, &raw mut find);
    if mw.is_null() {
        mw = xcalloc(
            1 as size_t,
            ::core::mem::size_of::<monitor_window>() as size_t,
        ) as *mut monitor_window;
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
        value,
        &raw mut (*mw).last,
    );
}
unsafe extern "C" fn monitor_sweep_all_windows(mut me: *mut monitor_item, mut generation: u_int) {
    let mut mw: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut mw1: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    mw = monitor_windows_minmax(&raw mut (*me).windows, RB_NEGINF);
    while !mw.is_null() && {
        mw1 = monitor_windows_next(mw);
        1 as ::core::ffi::c_int != 0
    } {
        if !((*mw).generation == generation) {
            monitor_windows_remove(&raw mut (*me).windows, mw);
            free((*mw).last as *mut ::core::ffi::c_void);
            free(mw as *mut ::core::ffi::c_void);
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
    me = monitor_items_minmax(&raw mut (*ms).items, RB_NEGINF);
    while !me.is_null() && {
        me1 = monitor_items_next(me);
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
    me = monitor_items_minmax(&raw mut (*ms).items, RB_NEGINF);
    while !me.is_null() && {
        me1 = monitor_items_next(me);
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
        wp = (*(*wl).window).panes.tqh_first;
        while !wp.is_null() {
            ft = monitor_create_formats(c, s, wl, wp);
            me = monitor_items_minmax(&raw mut (*ms).items, RB_NEGINF);
            while !me.is_null() && {
                me1 = monitor_items_next(me);
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
            wp = (*wp).entry.tqe_next;
        }
        wl = winlinks_next(wl);
    }
    me = monitor_items_minmax(&raw mut (*ms).items, RB_NEGINF);
    while !me.is_null() && {
        me1 = monitor_items_next(me);
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
        me = monitor_items_minmax(&raw mut (*ms).items, RB_NEGINF);
        while !me.is_null() && {
            me1 = monitor_items_next(me);
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
    me = monitor_items_minmax(&raw mut (*ms).items, RB_NEGINF);
    while !me.is_null() && {
        me1 = monitor_items_next(me);
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
    me = monitor_items_minmax(&raw mut (*ms).items, RB_NEGINF);
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
        me = monitor_items_next(me);
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
    ms = xcalloc(1 as size_t, ::core::mem::size_of::<monitor_set>() as size_t) as *mut monitor_set;
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
        if event_initialized(&raw mut (*ms).timer) != 0 {
            event_del(&raw mut (*ms).timer);
        }
        me = monitor_items_minmax(&raw mut (*ms).items, RB_NEGINF);
        while !me.is_null() && {
            me1 = monitor_items_next(me);
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
        free(ms as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn monitor_parse(
    mut value: *const ::core::ffi::c_char,
    mut name: *mut *mut ::core::ffi::c_char,
    mut type_0: *mut monitor_type,
    mut id: *mut ::core::ffi::c_int,
    mut format: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut what: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut split: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    copy = xstrdup(value);
    *id = -(1 as ::core::ffi::c_int);
    what = strchr(copy, ':' as i32);
    if !what.is_null() {
        let fresh0 = what;
        what = what.offset(1);
        *fresh0 = '\0' as i32 as ::core::ffi::c_char;
        split = strchr(what, ':' as i32);
        if !split.is_null() {
            let fresh1 = split;
            split = split.offset(1);
            *fresh1 = '\0' as i32 as ::core::ffi::c_char;
            if strcmp(what, b"%*\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                *type_0 = MONITOR_ALL_PANES;
                current_block = 3512920355445576850;
            } else if sscanf(
                what,
                b"%%%d\0" as *const u8 as *const ::core::ffi::c_char,
                id,
            ) == 1 as ::core::ffi::c_int
                && *id >= 0 as ::core::ffi::c_int
            {
                *type_0 = MONITOR_PANE;
                current_block = 3512920355445576850;
            } else if strcmp(what, b"@*\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                *type_0 = MONITOR_ALL_WINDOWS;
                current_block = 3512920355445576850;
            } else if sscanf(
                what,
                b"@%d\0" as *const u8 as *const ::core::ffi::c_char,
                id,
            ) == 1 as ::core::ffi::c_int
                && *id >= 0 as ::core::ffi::c_int
            {
                *type_0 = MONITOR_WINDOW;
                current_block = 3512920355445576850;
            } else if *what as ::core::ffi::c_int == '\0' as i32 {
                *type_0 = MONITOR_SESSION;
                current_block = 3512920355445576850;
            } else {
                current_block = 7799373935801088419;
            }
            match current_block {
                7799373935801088419 => {}
                _ => {
                    *name = xstrdup(copy);
                    *format = xstrdup(split);
                    free(copy as *mut ::core::ffi::c_void);
                    return 0 as ::core::ffi::c_int;
                }
            }
        }
    }
    free(copy as *mut ::core::ffi::c_void);
    return -(1 as ::core::ffi::c_int);
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
    let mut find: monitor_item = monitor_item {
        name: name as *mut ::core::ffi::c_char,
        format: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        type_0: MONITOR_SESSION,
        id: 0,
        flags: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
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
    me = monitor_items_find(&raw mut (*ms).items, &raw mut find);
    if !me.is_null() {
        monitor_free_item(ms, me);
    }
    me = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<monitor_item>() as size_t,
    ) as *mut monitor_item;
    (*me).name = xstrdup(name);
    (*me).format = xstrdup(format);
    (*me).type_0 = type_0;
    (*me).id = id as u_int;
    (*me).flags = flags;
    (*me).panes.storage = std::ptr::null_mut();
    (*me).windows.storage = std::ptr::null_mut();
    monitor_items_insert(&raw mut (*ms).items, me);
    if event_initialized(&raw mut (*ms).timer) == 0 {
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
        name: name as *mut ::core::ffi::c_char,
        format: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        type_0: MONITOR_SESSION,
        id: 0,
        flags: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
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
    me = monitor_items_find(&raw mut (*ms).items, &raw mut find);
    if !me.is_null() {
        monitor_free_item(ms, me);
    }
    if (*ms).items.storage.is_null() && event_initialized(&raw mut (*ms).timer) != 0 {
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
        name: name as *mut ::core::ffi::c_char,
        format: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        type_0: MONITOR_SESSION,
        id: 0,
        flags: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
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
    me = monitor_items_find(&raw mut (*ms).items, &raw mut find);
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
        name: name as *mut ::core::ffi::c_char,
        format: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        type_0: MONITOR_SESSION,
        id: 0,
        flags: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
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
    me = monitor_items_find(&raw mut (*ms).items, &raw mut find);
    if me.is_null() {
        return 0 as time_t;
    }
    return (*me).fire_time;
}

unsafe fn monitor_items_key(elm: *mut monitor_item) -> Vec<u8> {
    std::ffi::CStr::from_ptr((*elm).name).to_bytes().to_vec()
}
pub unsafe fn monitor_items_find(
    head: *mut monitor_items,
    elm: *mut monitor_item,
) -> *mut monitor_item {
    crate::src::shared::tree::OrderedIndex::<Vec<u8>, monitor_item>::find(
        (*head).storage,
        &monitor_items_key(elm),
    )
}
pub unsafe fn monitor_items_nfind(
    head: *mut monitor_items,
    elm: *mut monitor_item,
) -> *mut monitor_item {
    crate::src::shared::tree::OrderedIndex::<Vec<u8>, monitor_item>::nfind(
        (*head).storage,
        &monitor_items_key(elm),
    )
}
pub unsafe fn monitor_items_insert(
    head: *mut monitor_items,
    elm: *mut monitor_item,
) -> *mut monitor_item {
    let found = crate::src::shared::tree::OrderedIndex::<Vec<u8>, monitor_item>::insert(
        &raw mut (*head).storage,
        monitor_items_key(elm),
        elm,
    );
    if found.is_null() {
        (*elm).entry.owner = (*head).storage;
    }
    found
}
pub unsafe fn monitor_items_remove(
    head: *mut monitor_items,
    elm: *mut monitor_item,
) -> *mut monitor_item {
    let removed = crate::src::shared::tree::OrderedIndex::<Vec<u8>, monitor_item>::remove(
        &raw mut (*head).storage,
        &monitor_items_key(elm),
        elm,
    );
    if !removed.is_null() {
        (*elm).entry.owner = std::ptr::null_mut();
    }
    removed
}
pub unsafe fn monitor_items_minmax(
    head: *mut monitor_items,
    direction: ::core::ffi::c_int,
) -> *mut monitor_item {
    crate::src::shared::tree::OrderedIndex::<Vec<u8>, monitor_item>::edge(
        (*head).storage,
        direction < 0,
    )
}
pub unsafe fn monitor_items_next(elm: *mut monitor_item) -> *mut monitor_item {
    crate::src::shared::tree::OrderedIndex::<Vec<u8>, monitor_item>::neighbor(
        (*elm).entry.owner,
        &monitor_items_key(elm),
        true,
    )
}
pub unsafe fn monitor_items_prev(elm: *mut monitor_item) -> *mut monitor_item {
    crate::src::shared::tree::OrderedIndex::<Vec<u8>, monitor_item>::neighbor(
        (*elm).entry.owner,
        &monitor_items_key(elm),
        false,
    )
}

unsafe fn monitor_panes_key(elm: *mut monitor_pane) -> (u32, u32) {
    ((*elm).pane, (*elm).idx)
}
pub unsafe fn monitor_panes_find(
    head: *mut monitor_panes,
    elm: *mut monitor_pane,
) -> *mut monitor_pane {
    crate::src::shared::tree::OrderedIndex::<(u32, u32), monitor_pane>::find(
        (*head).storage,
        &monitor_panes_key(elm),
    )
}
pub unsafe fn monitor_panes_nfind(
    head: *mut monitor_panes,
    elm: *mut monitor_pane,
) -> *mut monitor_pane {
    crate::src::shared::tree::OrderedIndex::<(u32, u32), monitor_pane>::nfind(
        (*head).storage,
        &monitor_panes_key(elm),
    )
}
pub unsafe fn monitor_panes_insert(
    head: *mut monitor_panes,
    elm: *mut monitor_pane,
) -> *mut monitor_pane {
    let found = crate::src::shared::tree::OrderedIndex::<(u32, u32), monitor_pane>::insert(
        &raw mut (*head).storage,
        monitor_panes_key(elm),
        elm,
    );
    if found.is_null() {
        (*elm).entry.owner = (*head).storage;
    }
    found
}
pub unsafe fn monitor_panes_remove(
    head: *mut monitor_panes,
    elm: *mut monitor_pane,
) -> *mut monitor_pane {
    let removed = crate::src::shared::tree::OrderedIndex::<(u32, u32), monitor_pane>::remove(
        &raw mut (*head).storage,
        &monitor_panes_key(elm),
        elm,
    );
    if !removed.is_null() {
        (*elm).entry.owner = std::ptr::null_mut();
    }
    removed
}
pub unsafe fn monitor_panes_minmax(
    head: *mut monitor_panes,
    direction: ::core::ffi::c_int,
) -> *mut monitor_pane {
    crate::src::shared::tree::OrderedIndex::<(u32, u32), monitor_pane>::edge(
        (*head).storage,
        direction < 0,
    )
}
pub unsafe fn monitor_panes_next(elm: *mut monitor_pane) -> *mut monitor_pane {
    crate::src::shared::tree::OrderedIndex::<(u32, u32), monitor_pane>::neighbor(
        (*elm).entry.owner,
        &monitor_panes_key(elm),
        true,
    )
}
pub unsafe fn monitor_panes_prev(elm: *mut monitor_pane) -> *mut monitor_pane {
    crate::src::shared::tree::OrderedIndex::<(u32, u32), monitor_pane>::neighbor(
        (*elm).entry.owner,
        &monitor_panes_key(elm),
        false,
    )
}

unsafe fn monitor_windows_key(elm: *mut monitor_window) -> (u32, u32) {
    ((*elm).window, (*elm).idx)
}
pub unsafe fn monitor_windows_find(
    head: *mut monitor_windows,
    elm: *mut monitor_window,
) -> *mut monitor_window {
    crate::src::shared::tree::OrderedIndex::<(u32, u32), monitor_window>::find(
        (*head).storage,
        &monitor_windows_key(elm),
    )
}
pub unsafe fn monitor_windows_nfind(
    head: *mut monitor_windows,
    elm: *mut monitor_window,
) -> *mut monitor_window {
    crate::src::shared::tree::OrderedIndex::<(u32, u32), monitor_window>::nfind(
        (*head).storage,
        &monitor_windows_key(elm),
    )
}
pub unsafe fn monitor_windows_insert(
    head: *mut monitor_windows,
    elm: *mut monitor_window,
) -> *mut monitor_window {
    let found = crate::src::shared::tree::OrderedIndex::<(u32, u32), monitor_window>::insert(
        &raw mut (*head).storage,
        monitor_windows_key(elm),
        elm,
    );
    if found.is_null() {
        (*elm).entry.owner = (*head).storage;
    }
    found
}
pub unsafe fn monitor_windows_remove(
    head: *mut monitor_windows,
    elm: *mut monitor_window,
) -> *mut monitor_window {
    let removed = crate::src::shared::tree::OrderedIndex::<(u32, u32), monitor_window>::remove(
        &raw mut (*head).storage,
        &monitor_windows_key(elm),
        elm,
    );
    if !removed.is_null() {
        (*elm).entry.owner = std::ptr::null_mut();
    }
    removed
}
pub unsafe fn monitor_windows_minmax(
    head: *mut monitor_windows,
    direction: ::core::ffi::c_int,
) -> *mut monitor_window {
    crate::src::shared::tree::OrderedIndex::<(u32, u32), monitor_window>::edge(
        (*head).storage,
        direction < 0,
    )
}
pub unsafe fn monitor_windows_next(elm: *mut monitor_window) -> *mut monitor_window {
    crate::src::shared::tree::OrderedIndex::<(u32, u32), monitor_window>::neighbor(
        (*elm).entry.owner,
        &monitor_windows_key(elm),
        true,
    )
}
pub unsafe fn monitor_windows_prev(elm: *mut monitor_window) -> *mut monitor_window {
    crate::src::shared::tree::OrderedIndex::<(u32, u32), monitor_window>::neighbor(
        (*elm).entry.owner,
        &monitor_windows_key(elm),
        false,
    )
}
