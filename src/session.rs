use crate::src::cmd_find::{cmd_find_from_session, cmd_find_from_winlink};
use crate::src::compat::strtonum::strtonum;
use crate::src::environ::environ_free;
use crate::src::events::{events_fire, events_fire_session, events_fire_winlink};
use crate::src::events_payload::{
    event_payload_create, event_payload_set_int, event_payload_set_session,
    event_payload_set_string, event_payload_set_target, event_payload_set_uint,
    event_payload_set_window,
};
use crate::src::ffi::libc::{free, gettimeofday, memcpy, strcmp};
use crate::src::grid::grid_collect_history;
use crate::src::log::{fatal, fatalx, log_debug};
use crate::src::options::{options_free, options_get_number};
use crate::src::reactor::{event_add, event_del, event_initialized, event_once, event_set};
use crate::src::resize::recalculate_sizes;
use crate::src::server::{marked_pane, server_clear_marked};
use crate::src::server_fn::server_lock_session;
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
use crate::src::shared::event::*;
pub use crate::src::shared::events::event_payload;
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
    PANE_THEMECHANGED,
};
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::session::{
    session_group, session_group_entry, session_group_sessions, session_groups, sessions,
};
pub use crate::src::shared::sort::sort_criteria;
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tree::{RB_BLACK, RB_INF, RB_NEGINF, RB_RED};
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::shared::window::{
    WINLINK_ACTIVITY, WINLINK_ALERTFLAGS, WINLINK_BELL, WINLINK_SILENCE, WINLINK_VISITED,
};
use crate::src::sort::sort_get_sessions;
use crate::src::status::status_update_cache;
use crate::src::tmux::global_options;
use crate::src::tty::tty_update_window_offset;
use crate::src::window::{
    window_update_activity, window_update_focus, winlink_add, winlink_clear_flags,
    winlink_find_by_index, winlink_find_by_window, winlink_find_by_window_id, winlink_next,
    winlink_previous, winlink_remove, winlink_set_window, winlink_stack_push, winlink_stack_remove,
    winlinks_minmax, winlinks_next,
};
use crate::src::xmalloc::{xasprintf, xcalloc, xmalloc, xstrdup};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[no_mangle]
pub static mut sessions: sessions = sessions { storage: None };
#[no_mangle]
pub static mut next_session_id: u_int = 0;
#[no_mangle]
pub static mut session_groups: session_groups = session_groups { storage: None };
#[no_mangle]
pub unsafe extern "C" fn session_cmp(
    mut s1: *mut session,
    mut s2: *mut session,
) -> ::core::ffi::c_int {
    return strcmp((*s1).name, (*s2).name);
}
pub(crate) unsafe fn sessions_key(elm: *mut session) -> Vec<u8> {
    std::ffi::CStr::from_ptr((*elm).name).to_bytes().to_vec()
}
pub unsafe fn sessions_find(head: *mut sessions, elm: *mut session) -> *mut session {
    crate::src::shared::tree::OrderedIndex::<Vec<u8>, session>::find(
        crate::src::shared::tree::OrderedIndex::boxed_ptr(&(*head).storage),
        &sessions_key(elm),
    )
}
pub unsafe fn sessions_nfind(head: *mut sessions, elm: *mut session) -> *mut session {
    crate::src::shared::tree::OrderedIndex::<Vec<u8>, session>::nfind(
        crate::src::shared::tree::OrderedIndex::boxed_ptr(&(*head).storage),
        &sessions_key(elm),
    )
}
pub unsafe fn sessions_insert(head: *mut sessions, elm: *mut session) -> *mut session {
    let found = crate::src::shared::tree::OrderedIndex::<Vec<u8>, session>::insert_boxed(
        &mut (*head).storage,
        sessions_key(elm),
        elm,
    );
    if found.is_null() {
        (*elm).entry.owner = crate::src::shared::tree::OrderedIndex::boxed_ptr(&(*head).storage);
    }
    found
}
pub unsafe fn sessions_remove(head: *mut sessions, elm: *mut session) -> *mut session {
    let key = sessions_key(elm);
    crate::src::shared::tree::OrderedIndex::<Vec<u8>, session>::remove_boxed_with(
        &mut (*head).storage,
        &key,
        elm,
        |node| unsafe {
            (*node).entry.owner = std::ptr::null_mut();
        },
    )
}
pub unsafe fn sessions_minmax(head: *mut sessions, direction: ::core::ffi::c_int) -> *mut session {
    crate::src::shared::tree::OrderedIndex::<Vec<u8>, session>::edge(
        crate::src::shared::tree::OrderedIndex::boxed_ptr(&(*head).storage),
        direction < 0,
    )
}
/// Resume a potentially destructive walk using a saved name and the live index.
/// The named session and any of its successors may already have been removed.
pub unsafe fn sessions_after(head: *mut sessions, name: &[u8]) -> *mut session {
    crate::src::shared::tree::OrderedIndex::<Vec<u8>, session>::neighbor(
        crate::src::shared::tree::OrderedIndex::boxed_ptr(&(*head).storage),
        name,
        true,
    )
}

/// The session must still belong to its index. Destructive walks use sessions_after.
pub unsafe fn sessions_next(elm: *mut session) -> *mut session {
    crate::src::shared::tree::OrderedIndex::<Vec<u8>, session>::neighbor(
        (*elm).entry.owner,
        std::ffi::CStr::from_ptr((*elm).name).to_bytes(),
        true,
    )
}
/// The session must still belong to its index.
pub unsafe fn sessions_prev(elm: *mut session) -> *mut session {
    crate::src::shared::tree::OrderedIndex::<Vec<u8>, session>::neighbor(
        (*elm).entry.owner,
        std::ffi::CStr::from_ptr((*elm).name).to_bytes(),
        false,
    )
}

#[no_mangle]
pub unsafe extern "C" fn session_group_cmp(
    mut s1: *mut session_group,
    mut s2: *mut session_group,
) -> ::core::ffi::c_int {
    return strcmp((*s1).name, (*s2).name);
}
unsafe fn session_groups_key(elm: *mut session_group) -> Vec<u8> {
    std::ffi::CStr::from_ptr((*elm).name).to_bytes().to_vec()
}
pub unsafe fn session_groups_find(
    head: *mut session_groups,
    elm: *mut session_group,
) -> *mut session_group {
    crate::src::shared::tree::OrderedIndex::<Vec<u8>, session_group>::find(
        crate::src::shared::tree::OrderedIndex::boxed_ptr(&(*head).storage),
        &session_groups_key(elm),
    )
}
pub unsafe fn session_groups_nfind(
    head: *mut session_groups,
    elm: *mut session_group,
) -> *mut session_group {
    crate::src::shared::tree::OrderedIndex::<Vec<u8>, session_group>::nfind(
        crate::src::shared::tree::OrderedIndex::boxed_ptr(&(*head).storage),
        &session_groups_key(elm),
    )
}
pub unsafe fn session_groups_insert(
    head: *mut session_groups,
    elm: *mut session_group,
) -> *mut session_group {
    let found = crate::src::shared::tree::OrderedIndex::<Vec<u8>, session_group>::insert_boxed(
        &mut (*head).storage,
        session_groups_key(elm),
        elm,
    );
    if found.is_null() {
        (*elm).entry.owner = crate::src::shared::tree::OrderedIndex::boxed_ptr(&(*head).storage);
    }
    found
}
pub unsafe fn session_groups_remove(
    head: *mut session_groups,
    elm: *mut session_group,
) -> *mut session_group {
    let key = session_groups_key(elm);
    crate::src::shared::tree::OrderedIndex::<Vec<u8>, session_group>::remove_boxed_with(
        &mut (*head).storage,
        &key,
        elm,
        |node| unsafe {
            (*node).entry.owner = std::ptr::null_mut();
        },
    )
}
pub unsafe fn session_groups_minmax(
    head: *mut session_groups,
    direction: ::core::ffi::c_int,
) -> *mut session_group {
    crate::src::shared::tree::OrderedIndex::<Vec<u8>, session_group>::edge(
        crate::src::shared::tree::OrderedIndex::boxed_ptr(&(*head).storage),
        direction < 0,
    )
}
pub unsafe fn session_groups_next(elm: *mut session_group) -> *mut session_group {
    crate::src::shared::tree::OrderedIndex::<Vec<u8>, session_group>::neighbor(
        (*elm).entry.owner,
        &session_groups_key(elm),
        true,
    )
}
pub unsafe fn session_groups_prev(elm: *mut session_group) -> *mut session_group {
    crate::src::shared::tree::OrderedIndex::<Vec<u8>, session_group>::neighbor(
        (*elm).entry.owner,
        &session_groups_key(elm),
        false,
    )
}

#[no_mangle]
pub unsafe extern "C" fn session_alive(mut s: *mut session) -> ::core::ffi::c_int {
    let mut s_loop: *mut session = ::core::ptr::null_mut::<session>();
    s_loop = sessions_minmax(&raw mut sessions, RB_NEGINF);
    while !s_loop.is_null() {
        if s_loop == s {
            return 1 as ::core::ffi::c_int;
        }
        s_loop = sessions_next(s_loop);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn session_find(mut name: *const ::core::ffi::c_char) -> *mut session {
    let mut s: session = session {
        id: 0,
        name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cwd: ::core::ptr::null::<::core::ffi::c_char>(),
        creation_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        last_attached_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        activity_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        last_activity_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        lock_timer: event {
            ev_evcallback: event_callback {
                evcb_active_next: event_callback_entry {
                    tqe_next: ::core::ptr::null_mut::<event_callback>(),
                    tqe_prev: ::core::ptr::null_mut::<*mut event_callback>(),
                },
                evcb_flags: 0,
                evcb_pri: 0,
                evcb_closure: 0,
                evcb_cb_union: event_callback_union {
                    evcb_callback: None,
                },
                evcb_arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            },
            ev_timeout_pos: event_timeout_pos {
                ev_next_with_common_timeout: event_timeout_entry {
                    tqe_next: ::core::ptr::null_mut::<event>(),
                    tqe_prev: ::core::ptr::null_mut::<*mut event>(),
                },
            },
            ev_fd: 0,
            ev_base: ::core::ptr::null_mut::<event_base>(),
            ev_: event_io_or_signal {
                ev_io: event_io {
                    ev_io_next: event_io_entry {
                        le_next: ::core::ptr::null_mut::<event>(),
                        le_prev: ::core::ptr::null_mut::<*mut event>(),
                    },
                    ev_timeout: timeval {
                        tv_sec: 0,
                        tv_usec: 0,
                    },
                },
            },
            ev_events: 0,
            ev_res: 0,
            ev_timeout: timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
        },
        curw: ::core::ptr::null_mut::<winlink>(),
        lastw: winlink_stack {
            storage: None,
            reserved: std::ptr::null_mut(),
        },
        windows: winlinks { storage: None },
        statusat: 0,
        statuslines: 0,
        options: ::core::ptr::null_mut::<options>(),
        flags: 0,
        attached: 0,
        tio: ::core::ptr::null_mut::<termios>(),
        environ: ::core::ptr::null_mut::<environ>(),
        references: 0,
        gentry: session_gentry {
            tqe_next: ::core::ptr::null_mut::<session>(),
            tqe_prev: ::core::ptr::null_mut::<*mut session>(),
        },
        entry: session_entry {
            owner: std::ptr::null_mut(),
        },
    };
    s.name = name as *mut ::core::ffi::c_char;
    return sessions_find(&raw mut sessions, &raw mut s);
}
#[no_mangle]
pub unsafe extern "C" fn session_find_by_id_str(mut s: *const ::core::ffi::c_char) -> *mut session {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut id: u_int = 0;
    if *s as ::core::ffi::c_int != '$' as i32 {
        return ::core::ptr::null_mut::<session>();
    }
    id = strtonum(
        s.offset(1 as ::core::ffi::c_int as isize),
        0 as ::core::ffi::c_longlong,
        UINT_MAX as ::core::ffi::c_longlong,
        &raw mut errstr,
    ) as u_int;
    if !errstr.is_null() {
        return ::core::ptr::null_mut::<session>();
    }
    return session_find_by_id(id);
}
#[no_mangle]
pub unsafe extern "C" fn session_find_by_id(mut id: u_int) -> *mut session {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    s = sessions_minmax(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        if (*s).id == id {
            return s;
        }
        s = sessions_next(s);
    }
    return ::core::ptr::null_mut::<session>();
}
#[no_mangle]
pub unsafe extern "C" fn session_create(
    mut prefix: *const ::core::ffi::c_char,
    mut name: *const ::core::ffi::c_char,
    mut cwd: *const ::core::ffi::c_char,
    mut env: *mut environ,
    mut oo: *mut options,
    mut tio: *mut termios,
) -> *mut session {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    s = Box::into_raw(Box::new(std::mem::zeroed::<session>()));
    (*s).references = 1 as ::core::ffi::c_int;
    (*s).flags = 0 as ::core::ffi::c_int;
    (*s).cwd = xstrdup(cwd);
    (*s).lastw.storage = None;
    (*s).lastw.reserved = std::ptr::null_mut();
    (*s).windows.storage = None;
    (*s).environ = env;
    (*s).options = oo;
    status_update_cache(s);
    (*s).tio = ::core::ptr::null_mut::<termios>();
    if !tio.is_null() {
        (*s).tio = xmalloc(::core::mem::size_of::<termios>() as size_t) as *mut termios;
        memcpy(
            (*s).tio as *mut ::core::ffi::c_void,
            tio as *const ::core::ffi::c_void,
            ::core::mem::size_of::<termios>() as size_t,
        );
    }
    if !name.is_null() {
        (*s).name = xstrdup(name);
        let fresh0 = next_session_id;
        next_session_id = next_session_id.wrapping_add(1);
        (*s).id = fresh0;
    } else {
        loop {
            let fresh1 = next_session_id;
            next_session_id = next_session_id.wrapping_add(1);
            (*s).id = fresh1;
            free((*s).name as *mut ::core::ffi::c_void);
            if !prefix.is_null() {
                xasprintf(
                    &raw mut (*s).name,
                    b"%s-%u\0" as *const u8 as *const ::core::ffi::c_char,
                    prefix,
                    (*s).id,
                );
            } else {
                xasprintf(
                    &raw mut (*s).name,
                    b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                    (*s).id,
                );
            }
            if sessions_find(&raw mut sessions, s).is_null() {
                break;
            }
        }
    }
    sessions_insert(&raw mut sessions, s);
    log_debug(
        b"new session %s $%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).name,
        (*s).id,
    );
    if gettimeofday(&raw mut (*s).creation_time, NULL) != 0 as ::core::ffi::c_int {
        fatal(b"gettimeofday failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    session_update_activity(s, &raw mut (*s).creation_time);
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn session_add_ref(
    mut s: *mut session,
    mut from: *const ::core::ffi::c_char,
) {
    (*s).references += 1;
    log_debug(
        b"%s: %s %s, now %d\0" as *const u8 as *const ::core::ffi::c_char,
        b"session_add_ref\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).name,
        from,
        (*s).references,
    );
}
#[no_mangle]
pub unsafe extern "C" fn session_remove_ref(
    mut s: *mut session,
    mut from: *const ::core::ffi::c_char,
) {
    (*s).references -= 1;
    log_debug(
        b"%s: %s %s, now %d\0" as *const u8 as *const ::core::ffi::c_char,
        b"session_remove_ref\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).name,
        from,
        (*s).references,
    );
    if (*s).references == 0 as ::core::ffi::c_int {
        event_once(
            -(1 as ::core::ffi::c_int),
            EV_TIMEOUT as ::core::ffi::c_short,
            Some(
                session_free
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            s as *mut ::core::ffi::c_void,
            ::core::ptr::null::<timeval>(),
        );
    }
}
unsafe extern "C" fn session_free(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut s: *mut session = arg as *mut session;
    log_debug(
        b"session %s freed (%d references)\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).name,
        (*s).references,
    );
    if (*s).references == 0 as ::core::ffi::c_int {
        environ_free((*s).environ);
        options_free((*s).options);
        crate::src::window::winlink_stack_clear(&raw mut (*s).lastw);
        free((*s).name as *mut ::core::ffi::c_void);
        drop(Box::from_raw(s));
    }
}
#[no_mangle]
pub unsafe extern "C" fn session_destroy(
    mut s: *mut session,
    mut notify: ::core::ffi::c_int,
    mut from: *const ::core::ffi::c_char,
) {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    log_debug(
        b"session %s destroyed (%s)\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).name,
        from,
    );
    if (*s).curw.is_null() {
        return;
    }
    (*s).curw = ::core::ptr::null_mut::<winlink>();
    sessions_remove(&raw mut sessions, s);
    if notify != 0 {
        events_fire_session(
            b"session-closed\0" as *const u8 as *const ::core::ffi::c_char,
            s,
        );
    }
    free((*s).tio as *mut ::core::ffi::c_void);
    if event_initialized(&raw mut (*s).lock_timer) != 0 {
        event_del(&raw mut (*s).lock_timer);
    }
    session_group_remove(s);
    while !crate::src::window::winlink_stack_first(&raw const (*s).lastw, &raw mut (*s).windows)
        .is_null()
    {
        let first =
            crate::src::window::winlink_stack_first(&raw const (*s).lastw, &raw mut (*s).windows);
        winlink_stack_remove(&raw mut (*s).lastw, first);
    }
    crate::src::window::winlink_stack_clear(&raw mut (*s).lastw);
    while (*s).windows.storage.is_some() {
        wl = winlinks_minmax(&raw mut (*s).windows, RB_NEGINF);
        events_fire_winlink(
            b"window-unlinked\0" as *const u8 as *const ::core::ffi::c_char,
            wl,
        );
        winlink_remove(&raw mut (*s).windows, wl);
    }
    free((*s).cwd as *mut ::core::ffi::c_void);
    session_remove_ref(
        s,
        b"session_destroy\0" as *const u8 as *const ::core::ffi::c_char,
    );
}
unsafe extern "C" fn session_lock_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut s: *mut session = arg as *mut session;
    if (*s).attached == 0 as u_int {
        return;
    }
    log_debug(
        b"session %s locked, activity time %lld\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).name,
        (*s).activity_time.tv_sec as ::core::ffi::c_longlong,
    );
    server_lock_session(s);
    recalculate_sizes();
}
#[no_mangle]
pub unsafe extern "C" fn session_update_activity(mut s: *mut session, mut from: *mut timeval) {
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if from.is_null() {
        gettimeofday(&raw mut (*s).activity_time, NULL);
    } else {
        memcpy(
            &raw mut (*s).activity_time as *mut ::core::ffi::c_void,
            from as *const ::core::ffi::c_void,
            ::core::mem::size_of::<timeval>() as size_t,
        );
    }
    log_debug(
        b"session $%u %s activity %lld.%06d\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).id,
        (*s).name,
        (*s).activity_time.tv_sec as ::core::ffi::c_longlong,
        (*s).activity_time.tv_usec as ::core::ffi::c_int,
    );
    if event_initialized(&raw mut (*s).lock_timer) != 0 {
        event_del(&raw mut (*s).lock_timer);
    } else {
        event_set(
            &raw mut (*s).lock_timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            Some(
                session_lock_timer
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            s as *mut ::core::ffi::c_void,
        );
    }
    if (*s).attached != 0 as u_int {
        tv.tv_usec = 0 as __suseconds_t;
        tv.tv_sec = tv.tv_usec as __time_t;
        tv.tv_sec = options_get_number(
            (*s).options,
            b"lock-after-time\0" as *const u8 as *const ::core::ffi::c_char,
        ) as __time_t;
        if tv.tv_sec != 0 as __time_t {
            event_add(&raw mut (*s).lock_timer, &raw mut tv);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn session_next_session(
    mut s: *mut session,
    mut sort_crit: *mut sort_criteria,
) -> *mut session {
    let mut l: *mut *mut session = ::core::ptr::null_mut::<*mut session>();
    let mut n: u_int = 0;
    let mut i: u_int = 0;
    if sessions.storage.is_none() || session_alive(s) == 0 {
        return ::core::ptr::null_mut::<session>();
    }
    l = sort_get_sessions(&raw mut n, sort_crit);
    i = 0 as u_int;
    while i < n {
        if *l.offset(i as isize) == s {
            break;
        }
        i = i.wrapping_add(1);
    }
    if i == n {
        fatalx(
            b"session %s not found in sorted list\0" as *const u8 as *const ::core::ffi::c_char,
            (*s).name,
        );
    }
    i = i.wrapping_add(1);
    if i == n {
        i = 0 as u_int;
    }
    return *l.offset(i as isize);
}
#[no_mangle]
pub unsafe extern "C" fn session_previous_session(
    mut s: *mut session,
    mut sort_crit: *mut sort_criteria,
) -> *mut session {
    let mut l: *mut *mut session = ::core::ptr::null_mut::<*mut session>();
    let mut n: u_int = 0;
    let mut i: u_int = 0;
    if sessions.storage.is_none() || session_alive(s) == 0 {
        return ::core::ptr::null_mut::<session>();
    }
    l = sort_get_sessions(&raw mut n, sort_crit);
    i = 0 as u_int;
    while i < n {
        if *l.offset(i as isize) == s {
            break;
        }
        i = i.wrapping_add(1);
    }
    if i == n {
        fatalx(
            b"session %s not found in sorted list\0" as *const u8 as *const ::core::ffi::c_char,
            (*s).name,
        );
    }
    if i == 0 as u_int {
        i = n;
    }
    i = i.wrapping_sub(1);
    return *l.offset(i as isize);
}
#[no_mangle]
pub unsafe extern "C" fn session_attach(
    mut s: *mut session,
    mut w: *mut window,
    mut idx: ::core::ffi::c_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut winlink {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = winlink_add(&raw mut (*s).windows, idx);
    if wl.is_null() {
        xasprintf(
            cause,
            b"index in use: %d\0" as *const u8 as *const ::core::ffi::c_char,
            idx,
        );
        return ::core::ptr::null_mut::<winlink>();
    }
    (*wl).session = s;
    winlink_set_window(wl, w);
    events_fire_winlink(
        b"window-linked\0" as *const u8 as *const ::core::ffi::c_char,
        wl,
    );
    session_group_synchronize_from(s);
    return wl;
}
#[no_mangle]
pub unsafe extern "C" fn session_detach(
    mut s: *mut session,
    mut wl: *mut winlink,
) -> ::core::ffi::c_int {
    if winlinks_minmax(&raw mut (*s).windows, RB_NEGINF) == wl
        && winlinks_minmax(&raw mut (*s).windows, RB_INF) == wl
    {
        return 1 as ::core::ffi::c_int;
    }
    if (*s).curw == wl
        && session_last(s) != 0 as ::core::ffi::c_int
        && session_previous(s, 0 as ::core::ffi::c_int) != 0 as ::core::ffi::c_int
    {
        session_next(s, 0 as ::core::ffi::c_int);
    }
    (*wl).flags &= !WINLINK_ALERTFLAGS;
    events_fire_winlink(
        b"window-unlinked\0" as *const u8 as *const ::core::ffi::c_char,
        wl,
    );
    winlink_stack_remove(&raw mut (*s).lastw, wl);
    winlink_remove(&raw mut (*s).windows, wl);
    session_group_synchronize_from(s);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn session_has(
    mut s: *mut session,
    mut w: *mut window,
) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        if (*wl).session == s {
            return 1 as ::core::ffi::c_int;
        }
        wl = (*wl).wentry.tqe_next;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn session_is_linked(
    mut s: *mut session,
    mut w: *mut window,
) -> ::core::ffi::c_int {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_group_contains(s);
    if !sg.is_null() {
        return ((*w).references != session_group_count(sg)) as ::core::ffi::c_int;
    }
    return ((*w).references != 1 as u_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn session_next_alert(mut wl: *mut winlink) -> *mut winlink {
    while !wl.is_null() {
        if (*wl).flags & WINLINK_ALERTFLAGS != 0 {
            break;
        }
        wl = winlink_next(wl);
    }
    return wl;
}
#[no_mangle]
pub unsafe extern "C" fn session_next(
    mut s: *mut session,
    mut alert: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if (*s).curw.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    wl = winlink_next((*s).curw);
    if alert != 0 {
        wl = session_next_alert(wl);
    }
    if wl.is_null() {
        wl = winlinks_minmax(&raw mut (*s).windows, RB_NEGINF);
        if alert != 0 && {
            wl = session_next_alert(wl);
            wl.is_null()
        } {
            return -(1 as ::core::ffi::c_int);
        }
    }
    return session_set_current(s, wl);
}
unsafe extern "C" fn session_previous_alert(mut wl: *mut winlink) -> *mut winlink {
    while !wl.is_null() {
        if (*wl).flags & WINLINK_ALERTFLAGS != 0 {
            break;
        }
        wl = winlink_previous(wl);
    }
    return wl;
}
#[no_mangle]
pub unsafe extern "C" fn session_previous(
    mut s: *mut session,
    mut alert: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if (*s).curw.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    wl = winlink_previous((*s).curw);
    if alert != 0 {
        wl = session_previous_alert(wl);
    }
    if wl.is_null() {
        wl = winlinks_minmax(&raw mut (*s).windows, RB_INF);
        if alert != 0 && {
            wl = session_previous_alert(wl);
            wl.is_null()
        } {
            return -(1 as ::core::ffi::c_int);
        }
    }
    return session_set_current(s, wl);
}
#[no_mangle]
pub unsafe extern "C" fn session_select(
    mut s: *mut session,
    mut idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = winlink_find_by_index(&raw mut (*s).windows, idx);
    return session_set_current(s, wl);
}
#[no_mangle]
pub unsafe extern "C" fn session_last(mut s: *mut session) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = crate::src::window::winlink_stack_first(&raw const (*s).lastw, &raw mut (*s).windows);
    if wl.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    if wl == (*s).curw {
        return 1 as ::core::ffi::c_int;
    }
    return session_set_current(s, wl);
}
unsafe extern "C" fn session_fire_window_changed(
    mut s: *mut session,
    mut wl: *mut winlink,
    mut old: *mut winlink,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    ep = event_payload_create();
    cmd_find_from_winlink(&raw mut fs, wl, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_session(
        ep,
        b"session\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).window,
    );
    event_payload_set_window(
        ep,
        b"new_window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).window,
    );
    event_payload_set_int(
        ep,
        b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).idx,
    );
    event_payload_set_int(
        ep,
        b"new_window_index\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).idx,
    );
    if !old.is_null() {
        event_payload_set_window(
            ep,
            b"old_window\0" as *const u8 as *const ::core::ffi::c_char,
            (*old).window,
        );
        event_payload_set_int(
            ep,
            b"old_window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (*old).idx,
        );
    }
    events_fire(
        b"session-window-changed\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
#[no_mangle]
pub unsafe extern "C" fn session_set_current(
    mut s: *mut session,
    mut wl: *mut winlink,
) -> ::core::ffi::c_int {
    let mut old: *mut winlink = (*s).curw;
    if wl.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    if wl == (*s).curw {
        return 1 as ::core::ffi::c_int;
    }
    winlink_stack_remove(&raw mut (*s).lastw, wl);
    winlink_stack_push(&raw mut (*s).lastw, (*s).curw);
    (*s).curw = wl;
    if options_get_number(
        global_options,
        b"focus-events\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        if !old.is_null() {
            window_update_focus((*old).window);
        }
        window_update_focus((*wl).window);
    }
    winlink_clear_flags(wl);
    window_update_activity((*wl).window);
    tty_update_window_offset((*wl).window);
    session_fire_window_changed(s, wl, old);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn session_group_contains(mut target: *mut session) -> *mut session_group {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    sg = session_groups_minmax(&raw mut session_groups, RB_NEGINF);
    while !sg.is_null() {
        s = (*sg).sessions.tqh_first;
        while !s.is_null() {
            if s == target {
                return sg;
            }
            s = (*s).gentry.tqe_next;
        }
        sg = session_groups_next(sg);
    }
    return ::core::ptr::null_mut::<session_group>();
}
#[no_mangle]
pub unsafe extern "C" fn session_group_find(
    mut name: *const ::core::ffi::c_char,
) -> *mut session_group {
    let mut sg: session_group = session_group {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        sessions: session_group_sessions {
            tqh_first: ::core::ptr::null_mut::<session>(),
            tqh_last: ::core::ptr::null_mut::<*mut session>(),
        },
        entry: session_group_entry {
            owner: std::ptr::null_mut(),
        },
    };
    sg.name = name;
    return session_groups_find(&raw mut session_groups, &raw mut sg);
}
#[no_mangle]
pub unsafe extern "C" fn session_group_new(
    mut name: *const ::core::ffi::c_char,
) -> *mut session_group {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_group_find(name);
    if !sg.is_null() {
        return sg;
    }
    sg = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<session_group>() as size_t,
    ) as *mut session_group;
    (*sg).name = xstrdup(name);
    (*sg).sessions.tqh_first = ::core::ptr::null_mut::<session>();
    (*sg).sessions.tqh_last = &raw mut (*sg).sessions.tqh_first;
    session_groups_insert(&raw mut session_groups, sg);
    return sg;
}
unsafe extern "C" fn session_group_fire(
    mut name: *const ::core::ffi::c_char,
    mut sg: *mut session_group,
    mut s: *mut session,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    ep = event_payload_create();
    if session_alive(s) != 0 {
        cmd_find_from_session(&raw mut fs, s, 0 as ::core::ffi::c_int);
        event_payload_set_target(ep, &raw mut fs);
    }
    event_payload_set_session(
        ep,
        b"session\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    event_payload_set_string(
        ep,
        b"group\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        (*sg).name,
    );
    event_payload_set_uint(
        ep,
        b"group_size\0" as *const u8 as *const ::core::ffi::c_char,
        session_group_count(sg),
    );
    events_fire(name, ep);
}
#[no_mangle]
pub unsafe extern "C" fn session_group_add(mut sg: *mut session_group, mut s: *mut session) {
    if session_group_contains(s).is_null() {
        (*s).gentry.tqe_next = ::core::ptr::null_mut::<session>();
        (*s).gentry.tqe_prev = (*sg).sessions.tqh_last;
        *(*sg).sessions.tqh_last = s;
        (*sg).sessions.tqh_last = &raw mut (*s).gentry.tqe_next;
        session_group_fire(
            b"session-added-to-group\0" as *const u8 as *const ::core::ffi::c_char,
            sg,
            s,
        );
    }
}
unsafe extern "C" fn session_group_remove(mut s: *mut session) {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_group_contains(s);
    if sg.is_null() {
        return;
    }
    session_group_fire(
        b"session-removed-from-group\0" as *const u8 as *const ::core::ffi::c_char,
        sg,
        s,
    );
    if !(*s).gentry.tqe_next.is_null() {
        (*(*s).gentry.tqe_next).gentry.tqe_prev = (*s).gentry.tqe_prev;
    } else {
        (*sg).sessions.tqh_last = (*s).gentry.tqe_prev;
    }
    *(*s).gentry.tqe_prev = (*s).gentry.tqe_next;
    if (*sg).sessions.tqh_first.is_null() {
        session_groups_remove(&raw mut session_groups, sg);
        free((*sg).name as *mut ::core::ffi::c_void);
        free(sg as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn session_group_count(mut sg: *mut session_group) -> u_int {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut n: u_int = 0;
    n = 0 as u_int;
    s = (*sg).sessions.tqh_first;
    while !s.is_null() {
        n = n.wrapping_add(1);
        s = (*s).gentry.tqe_next;
    }
    return n;
}
#[no_mangle]
pub unsafe extern "C" fn session_group_attached_count(mut sg: *mut session_group) -> u_int {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut n: u_int = 0;
    n = 0 as u_int;
    s = (*sg).sessions.tqh_first;
    while !s.is_null() {
        n = n.wrapping_add((*s).attached);
        s = (*s).gentry.tqe_next;
    }
    return n;
}
#[no_mangle]
pub unsafe extern "C" fn session_group_synchronize_to(mut s: *mut session) {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut target: *mut session = ::core::ptr::null_mut::<session>();
    sg = session_group_contains(s);
    if sg.is_null() {
        return;
    }
    target = ::core::ptr::null_mut::<session>();
    target = (*sg).sessions.tqh_first;
    while !target.is_null() {
        if target != s {
            break;
        }
        target = (*target).gentry.tqe_next;
    }
    if !target.is_null() {
        session_group_synchronize1(target, s);
    }
}
#[no_mangle]
pub unsafe extern "C" fn session_group_synchronize_from(mut target: *mut session) {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    sg = session_group_contains(target);
    if sg.is_null() {
        return;
    }
    s = (*sg).sessions.tqh_first;
    while !s.is_null() {
        if s != target {
            session_group_synchronize1(target, s);
        }
        s = (*s).gentry.tqe_next;
    }
}
unsafe extern "C" fn session_group_synchronize1(mut target: *mut session, mut s: *mut session) {
    let mut ww: *mut winlinks = ::core::ptr::null_mut::<winlinks>();
    let mut old_windows: winlinks;
    let mut old_lastw: winlink_stack;
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wl2: *mut winlink = ::core::ptr::null_mut::<winlink>();
    ww = &raw mut (*target).windows;
    if (*ww).storage.is_none() {
        return;
    }
    if !(*s).curw.is_null()
        && winlink_find_by_index(ww, (*(*s).curw).idx).is_null()
        && session_last(s) != 0 as ::core::ffi::c_int
        && session_previous(s, 0 as ::core::ffi::c_int) != 0 as ::core::ffi::c_int
    {
        session_next(s, 0 as ::core::ffi::c_int);
    }
    old_windows = std::ptr::replace(&mut (*s).windows, winlinks { storage: None });
    wl = winlinks_minmax(ww, RB_NEGINF);
    while !wl.is_null() {
        wl2 = winlink_add(&raw mut (*s).windows, (*wl).idx);
        (*wl2).session = s;
        winlink_set_window(wl2, (*wl).window);
        events_fire_winlink(
            b"window-linked\0" as *const u8 as *const ::core::ffi::c_char,
            wl2,
        );
        (*wl2).flags |= (*wl).flags & WINLINK_ALERTFLAGS;
        wl = winlinks_next(wl);
    }
    if !(*s).curw.is_null() {
        (*s).curw = winlink_find_by_index(&raw mut (*s).windows, (*(*s).curw).idx);
    } else if !(*target).curw.is_null() {
        (*s).curw = winlink_find_by_index(&raw mut (*s).windows, (*(*target).curw).idx);
    }
    if (*s).curw.is_null() {
        (*s).curw = winlinks_minmax(&raw mut (*s).windows, RB_NEGINF);
    }
    old_lastw = std::ptr::replace(
        &raw mut (*s).lastw,
        winlink_stack {
            storage: None,
            reserved: std::ptr::null_mut(),
        },
    );
    for old_idx in crate::src::window::winlink_stack_indices(&raw const old_lastw) {
        wl2 = winlink_find_by_index(&raw mut (*s).windows, old_idx);
        if !wl2.is_null() {
            crate::src::window::winlink_stack_append(&raw mut (*s).lastw, wl2);
        }
    }
    crate::src::window::winlink_stack_clear(&raw mut old_lastw);
    while old_windows.storage.is_some() {
        wl = winlinks_minmax(&raw mut old_windows, RB_NEGINF);
        wl2 = winlink_find_by_window_id(&raw mut (*s).windows, (*(*wl).window).id);
        if wl2.is_null() {
            events_fire_winlink(
                b"window-unlinked\0" as *const u8 as *const ::core::ffi::c_char,
                wl,
            );
        }
        winlink_remove(&raw mut old_windows, wl);
    }
}
#[no_mangle]
pub unsafe extern "C" fn session_renumber_windows(mut s: *mut session) {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wl1: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wl_new: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut old_wins: winlinks = std::ptr::replace(&mut (*s).windows, winlinks { storage: None });
    let mut old_lastw: winlink_stack;
    let mut new_idx: ::core::ffi::c_int = 0;
    let mut new_curw_idx: ::core::ffi::c_int = 0;
    let mut marked_idx: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    new_idx = options_get_number(
        (*s).options,
        b"base-index\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    new_curw_idx = 0 as ::core::ffi::c_int;
    wl = winlinks_minmax(&raw mut old_wins, RB_NEGINF);
    while !wl.is_null() {
        wl_new = winlink_add(&raw mut (*s).windows, new_idx);
        (*wl_new).session = s;
        winlink_set_window(wl_new, (*wl).window);
        (*wl_new).flags |= (*wl).flags & WINLINK_ALERTFLAGS;
        if wl == marked_pane.wl {
            marked_idx = (*wl_new).idx;
        }
        if wl == (*s).curw {
            new_curw_idx = (*wl_new).idx;
        }
        new_idx += 1;
        wl = winlinks_next(wl);
    }
    old_lastw = std::ptr::replace(
        &raw mut (*s).lastw,
        winlink_stack {
            storage: None,
            reserved: std::ptr::null_mut(),
        },
    );
    for old_idx in crate::src::window::winlink_stack_indices(&raw const old_lastw) {
        wl = crate::src::shared::tree::OrderedIndex::<::core::ffi::c_int, winlink>::find(
            crate::src::shared::tree::OrderedIndex::boxed_ptr(&old_wins.storage),
            &old_idx,
        );
        if wl.is_null() {
            continue;
        }
        (*wl).flags &= !WINLINK_VISITED;
        wl_new = winlink_find_by_window(&raw mut (*s).windows, (*wl).window);
        if !wl_new.is_null() {
            crate::src::window::winlink_stack_append(&raw mut (*s).lastw, wl_new);
        }
    }
    crate::src::window::winlink_stack_clear(&raw mut old_lastw);
    if marked_idx != -(1 as ::core::ffi::c_int) {
        marked_pane.wl = winlink_find_by_index(&raw mut (*s).windows, marked_idx);
        if marked_pane.wl.is_null() {
            server_clear_marked();
        }
    }
    (*s).curw = winlink_find_by_index(&raw mut (*s).windows, new_curw_idx);
    wl = winlinks_minmax(&raw mut old_wins, RB_NEGINF);
    while !wl.is_null() && {
        wl1 = winlinks_next(wl);
        1 as ::core::ffi::c_int != 0
    } {
        winlink_remove(&raw mut old_wins, wl);
        wl = wl1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn session_theme_changed(mut s: *mut session) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if !s.is_null() {
        wl = winlinks_minmax(&raw mut (*s).windows, RB_NEGINF);
        while !wl.is_null() {
            wp = (*(*wl).window).panes.tqh_first;
            while !wp.is_null() {
                (*wp).flags |= PANE_THEMECHANGED;
                wp = (*wp).entry.tqe_next;
            }
            wl = winlinks_next(wl);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn session_update_history(mut s: *mut session) {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut limit: u_int = 0;
    let mut osize: u_int = 0;
    limit = options_get_number(
        (*s).options,
        b"history-limit\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    wl = winlinks_minmax(&raw mut (*s).windows, RB_NEGINF);
    while !wl.is_null() {
        wp = (*(*wl).window).panes.tqh_first;
        while !wp.is_null() {
            gd = (*wp).base.grid;
            osize = (*gd).hsize;
            (*gd).hlimit = limit;
            grid_collect_history(gd, 1 as ::core::ffi::c_int);
            if (*gd).hsize != osize {
                log_debug(
                    b"%s: %%%u %u -> %u\0" as *const u8 as *const ::core::ffi::c_char,
                    b"session_update_history\0" as *const u8 as *const ::core::ffi::c_char,
                    (*wp).id,
                    osize,
                    (*gd).hsize,
                );
            }
            wp = (*wp).entry.tqe_next;
        }
        wl = winlinks_next(wl);
    }
}
