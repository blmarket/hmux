use crate::src::cmd::{cmd_mouse_pane, cmd_mouse_window};
use crate::src::cmd_queue::{cmdq_error, cmdq_get_client, cmdq_get_current, cmdq_get_event};
use crate::src::compat::strtonum::strtonum;
use crate::src::environ::environ_find;
use crate::src::ffi::libc::{fnmatch, free, memset, strchr, strcmp, strlcat, strlen, strncmp};
use crate::src::log::{fatalx, log_debug};
pub use crate::src::server::clients;
use crate::src::server::{marked_pane, server_check_marked};
pub use crate::src::session::sessions;
use crate::src::session::{
    session_alive, session_find, session_find_by_id_str, session_has, sessions_minmax,
    sessions_next,
};
pub use crate::src::shared::pane::window_pane_tree;
use crate::src::window::{
    all_window_panes, window_find_by_id_str, window_find_string, window_has_pane,
    window_pane_at_index, window_pane_find_by_id_str, window_pane_find_down, window_pane_find_left,
    window_pane_find_right, window_pane_find_up, window_pane_next_by_number,
    window_pane_previous_by_number, window_pane_tree_RB_MINMAX, window_pane_tree_RB_NEXT,
    winlink_find_by_index, winlink_next_by_number, winlink_previous_by_number, winlinks_RB_MINMAX,
    winlinks_RB_NEXT,
};
use crate::src::xmalloc::{xreallocarray, xstrdup};

use crate::src::shared::abi::*;
pub use crate::src::shared::arguments::args;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::command::{
    CMD_FIND_CANFAIL, CMD_FIND_DEFAULT_MARKED, CMD_FIND_EXACT_SESSION, CMD_FIND_EXACT_WINDOW,
    CMD_FIND_PREFER_UNATTACHED, CMD_FIND_QUIET, CMD_FIND_WINDOW_INDEX,
};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::{environ, environ_entry};
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
pub use crate::src::shared::limits::{__INT_MAX__, INT_MAX};
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
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tree::{RB_INF, RB_NEGINF};
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

pub const _PATH_DEV: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"/dev/\0") };
static mut cmd_find_session_table: [[*const ::core::ffi::c_char; 2]; 1] = [[
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
]];
static mut cmd_find_window_table: [[*const ::core::ffi::c_char; 2]; 6] = [
    [
        b"{start}\0" as *const u8 as *const ::core::ffi::c_char,
        b"^\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{last}\0" as *const u8 as *const ::core::ffi::c_char,
        b"!\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{end}\0" as *const u8 as *const ::core::ffi::c_char,
        b"$\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{next}\0" as *const u8 as *const ::core::ffi::c_char,
        b"+\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{previous}\0" as *const u8 as *const ::core::ffi::c_char,
        b"-\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
];
static mut cmd_find_pane_table: [[*const ::core::ffi::c_char; 2]; 16] = [
    [
        b"{last}\0" as *const u8 as *const ::core::ffi::c_char,
        b"!\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{next}\0" as *const u8 as *const ::core::ffi::c_char,
        b"+\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{previous}\0" as *const u8 as *const ::core::ffi::c_char,
        b"-\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{top}\0" as *const u8 as *const ::core::ffi::c_char,
        b"top\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{bottom}\0" as *const u8 as *const ::core::ffi::c_char,
        b"bottom\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{left}\0" as *const u8 as *const ::core::ffi::c_char,
        b"left\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{right}\0" as *const u8 as *const ::core::ffi::c_char,
        b"right\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{top-left}\0" as *const u8 as *const ::core::ffi::c_char,
        b"top-left\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{top-right}\0" as *const u8 as *const ::core::ffi::c_char,
        b"top-right\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{bottom-left}\0" as *const u8 as *const ::core::ffi::c_char,
        b"bottom-left\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{bottom-right}\0" as *const u8 as *const ::core::ffi::c_char,
        b"bottom-right\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{up-of}\0" as *const u8 as *const ::core::ffi::c_char,
        b"{up-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{down-of}\0" as *const u8 as *const ::core::ffi::c_char,
        b"{down-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{left-of}\0" as *const u8 as *const ::core::ffi::c_char,
        b"{left-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{right-of}\0" as *const u8 as *const ::core::ffi::c_char,
        b"{right-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
];
unsafe extern "C" fn cmd_find_inside_pane(mut c: *mut client) -> *mut window_pane {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    if c.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    wp = window_pane_tree_RB_MINMAX(&raw mut all_window_panes, RB_NEGINF);
    while !wp.is_null() {
        if (*wp).fd != -(1 as ::core::ffi::c_int)
            && strcmp(&raw mut (*wp).tty as *mut ::core::ffi::c_char, (*c).ttyname)
                == 0 as ::core::ffi::c_int
        {
            break;
        }
        wp = window_pane_tree_RB_NEXT(wp);
    }
    if wp.is_null() {
        envent = environ_find(
            (*c).environ,
            b"TMUX_PANE\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if !envent.is_null() {
            wp = window_pane_find_by_id_str((*envent).value);
        }
    }
    if !wp.is_null() {
        log_debug(
            b"%s: got pane %%%u (%s)\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_find_inside_pane\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).id,
            &raw mut (*wp).tty as *mut ::core::ffi::c_char,
        );
    }
    return wp;
}
unsafe extern "C" fn cmd_find_client_better(
    mut c: *mut client,
    mut than: *mut client,
) -> ::core::ffi::c_int {
    if than.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    return if (*c).activity_time.tv_sec == (*than).activity_time.tv_sec {
        ((*c).activity_time.tv_usec > (*than).activity_time.tv_usec) as ::core::ffi::c_int
    } else {
        ((*c).activity_time.tv_sec > (*than).activity_time.tv_sec) as ::core::ffi::c_int
    };
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_best_client(mut s: *mut session) -> *mut client {
    let mut c_loop: *mut client = ::core::ptr::null_mut::<client>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if (*s).attached == 0 as u_int {
        s = ::core::ptr::null_mut::<session>();
    }
    c = ::core::ptr::null_mut::<client>();
    c_loop = clients.tqh_first;
    while !c_loop.is_null() {
        if !(*c_loop).session.is_null() {
            if !(!s.is_null() && (*c_loop).session != s) {
                if cmd_find_client_better(c_loop, c) != 0 {
                    c = c_loop;
                }
            }
        }
        c_loop = (*c_loop).entry.tqe_next;
    }
    return c;
}
unsafe extern "C" fn cmd_find_session_better(
    mut s: *mut session,
    mut than: *mut session,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut attached: ::core::ffi::c_int = 0;
    if than.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    if flags & CMD_FIND_PREFER_UNATTACHED != 0 {
        attached = ((*than).attached != 0 as u_int) as ::core::ffi::c_int;
        if attached != 0 && (*s).attached == 0 as u_int {
            return 1 as ::core::ffi::c_int;
        } else if attached == 0 && (*s).attached != 0 as u_int {
            return 0 as ::core::ffi::c_int;
        }
    }
    return if (*s).activity_time.tv_sec == (*than).activity_time.tv_sec {
        ((*s).activity_time.tv_usec > (*than).activity_time.tv_usec) as ::core::ffi::c_int
    } else {
        ((*s).activity_time.tv_sec > (*than).activity_time.tv_sec) as ::core::ffi::c_int
    };
}
unsafe extern "C" fn cmd_find_session_valid(mut s: *mut session) -> ::core::ffi::c_int {
    if session_alive(s) == 0
        || (*s).curw.is_null()
        || (*(*s).curw).window.is_null()
        || (*(*(*s).curw).window).active.is_null()
    {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn cmd_find_best_session(
    mut slist: *mut *mut session,
    mut ssize: u_int,
    mut flags: ::core::ffi::c_int,
) -> *mut session {
    let mut s_loop: *mut session = ::core::ptr::null_mut::<session>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut i: u_int = 0;
    log_debug(
        b"%s: %u sessions to try\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_best_session\0" as *const u8 as *const ::core::ffi::c_char,
        ssize,
    );
    s = ::core::ptr::null_mut::<session>();
    if !slist.is_null() {
        i = 0 as u_int;
        while i < ssize {
            if !(cmd_find_session_valid(*slist.offset(i as isize)) == 0) {
                if cmd_find_session_better(*slist.offset(i as isize), s, flags) != 0 {
                    s = *slist.offset(i as isize);
                }
            }
            i = i.wrapping_add(1);
        }
    } else {
        s_loop = sessions_minmax(&raw mut sessions, RB_NEGINF);
        while !s_loop.is_null() {
            if !(cmd_find_session_valid(s_loop) == 0) {
                if cmd_find_session_better(s_loop, s, flags) != 0 {
                    s = s_loop;
                }
            }
            s_loop = sessions_next(s_loop);
        }
    }
    return s;
}
unsafe extern "C" fn cmd_find_best_session_with_window(
    mut fs: *mut cmd_find_state,
) -> ::core::ffi::c_int {
    let mut slist: *mut *mut session = ::core::ptr::null_mut::<*mut session>();
    let mut ssize: u_int = 0;
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    log_debug(
        b"%s: window is @%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_best_session_with_window\0" as *const u8 as *const ::core::ffi::c_char,
        (*(*fs).w).id,
    );
    ssize = 0 as u_int;
    s = sessions_minmax(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        if !(session_has(s, (*fs).w) == 0) {
            slist = xreallocarray(
                slist as *mut ::core::ffi::c_void,
                ssize.wrapping_add(1 as u_int) as size_t,
                ::core::mem::size_of::<*mut session>() as size_t,
            ) as *mut *mut session;
            let fresh2 = ssize;
            ssize = ssize.wrapping_add(1);
            let ref mut fresh3 = *slist.offset(fresh2 as isize);
            *fresh3 = s;
        }
        s = sessions_next(s);
    }
    if !(ssize == 0 as u_int) {
        (*fs).s = cmd_find_best_session(slist, ssize, (*fs).flags);
        if !(*fs).s.is_null() {
            free(slist as *mut ::core::ffi::c_void);
            return cmd_find_best_winlink_with_window(fs);
        }
    }
    free(slist as *mut ::core::ffi::c_void);
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn cmd_find_best_winlink_with_window(
    mut fs: *mut cmd_find_state,
) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wl_loop: *mut winlink = ::core::ptr::null_mut::<winlink>();
    log_debug(
        b"%s: window is @%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_best_winlink_with_window\0" as *const u8 as *const ::core::ffi::c_char,
        (*(*fs).w).id,
    );
    wl = ::core::ptr::null_mut::<winlink>();
    if !(*(*fs).s).curw.is_null() && (*(*(*fs).s).curw).window == (*fs).w {
        wl = (*(*fs).s).curw;
    } else {
        wl_loop = winlinks_RB_MINMAX(&raw mut (*(*fs).s).windows, RB_NEGINF);
        while !wl_loop.is_null() {
            if (*wl_loop).window == (*fs).w {
                wl = wl_loop;
                break;
            } else {
                wl_loop = winlinks_RB_NEXT(wl_loop);
            }
        }
    }
    if wl.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).wl = wl;
    (*fs).idx = (*(*fs).wl).idx;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn cmd_find_map_table(
    mut table: *mut [*const ::core::ffi::c_char; 2],
    mut s: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while !(*table.offset(i as isize))[0 as ::core::ffi::c_int as usize].is_null() {
        if strcmp(
            s,
            (*table.offset(i as isize))[0 as ::core::ffi::c_int as usize],
        ) == 0 as ::core::ffi::c_int
        {
            return (*table.offset(i as isize))[1 as ::core::ffi::c_int as usize];
        }
        i = i.wrapping_add(1);
    }
    return s;
}
unsafe extern "C" fn cmd_find_get_session(
    mut fs: *mut cmd_find_state,
    mut session: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut s_loop: *mut session = ::core::ptr::null_mut::<session>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_get_session\0" as *const u8 as *const ::core::ffi::c_char,
        session,
    );
    if *session as ::core::ffi::c_int == '$' as i32 {
        (*fs).s = session_find_by_id_str(session);
        if (*fs).s.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    }
    (*fs).s = session_find(session);
    if !(*fs).s.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    c = cmd_find_client(
        ::core::ptr::null_mut::<cmdq_item>(),
        session,
        1 as ::core::ffi::c_int,
    );
    if !c.is_null() && !(*c).session.is_null() {
        (*fs).s = (*c).session;
        return 0 as ::core::ffi::c_int;
    }
    if (*fs).flags & CMD_FIND_EXACT_SESSION != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    s = ::core::ptr::null_mut::<session>();
    s_loop = sessions_minmax(&raw mut sessions, RB_NEGINF);
    while !s_loop.is_null() {
        if strncmp(session, (*s_loop).name, strlen(session)) == 0 as ::core::ffi::c_int {
            if !s.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            s = s_loop;
        }
        s_loop = sessions_next(s_loop);
    }
    if !s.is_null() {
        (*fs).s = s;
        return 0 as ::core::ffi::c_int;
    }
    s = ::core::ptr::null_mut::<session>();
    s_loop = sessions_minmax(&raw mut sessions, RB_NEGINF);
    while !s_loop.is_null() {
        if fnmatch(session, (*s_loop).name, 0 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int {
            if !s.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            s = s_loop;
        }
        s_loop = sessions_next(s_loop);
    }
    if !s.is_null() {
        (*fs).s = s;
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn cmd_find_get_window(
    mut fs: *mut cmd_find_state,
    mut window: *const ::core::ffi::c_char,
    mut only: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_get_window\0" as *const u8 as *const ::core::ffi::c_char,
        window,
    );
    if *window as ::core::ffi::c_int == '@' as i32 {
        (*fs).w = window_find_by_id_str(window);
        if (*fs).w.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return cmd_find_best_session_with_window(fs);
    }
    (*fs).s = (*(*fs).current).s;
    if cmd_find_get_window_with_session(fs, window) == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if only == 0 && cmd_find_get_session(fs, window) == 0 as ::core::ffi::c_int {
        (*fs).wl = (*(*fs).s).curw;
        (*fs).w = (*(*fs).wl).window;
        if !(*fs).flags & CMD_FIND_WINDOW_INDEX != 0 {
            (*fs).idx = (*(*fs).wl).idx;
        }
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn cmd_find_get_window_with_session(
    mut fs: *mut cmd_find_state,
    mut window: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut exact: ::core::ffi::c_int = 0;
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_get_window_with_session\0" as *const u8 as *const ::core::ffi::c_char,
        window,
    );
    exact = (*fs).flags & CMD_FIND_EXACT_WINDOW;
    (*fs).wl = (*(*fs).s).curw;
    (*fs).w = (*(*fs).wl).window;
    if *window as ::core::ffi::c_int == '@' as i32 {
        (*fs).w = window_find_by_id_str(window);
        if (*fs).w.is_null() || session_has((*fs).s, (*fs).w) == 0 {
            return -(1 as ::core::ffi::c_int);
        }
        return cmd_find_best_winlink_with_window(fs);
    }
    if exact == 0
        && (*window.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '+' as i32
            || *window.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '-' as i32)
    {
        if *window.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32 {
            n = strtonum(
                window.offset(1 as ::core::ffi::c_int as isize),
                1 as ::core::ffi::c_longlong,
                INT_MAX as ::core::ffi::c_longlong,
                &raw mut errstr,
            ) as ::core::ffi::c_int;
            if !errstr.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
        } else {
            n = 1 as ::core::ffi::c_int;
        }
        s = (*fs).s;
        if (*fs).flags & CMD_FIND_WINDOW_INDEX != 0 {
            if *window.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '+' as i32
            {
                if INT_MAX - (*(*s).curw).idx < n {
                    return -(1 as ::core::ffi::c_int);
                }
                (*fs).idx = (*(*s).curw).idx + n;
            } else {
                if n > (*(*s).curw).idx {
                    return -(1 as ::core::ffi::c_int);
                }
                (*fs).idx = (*(*s).curw).idx - n;
            }
            return 0 as ::core::ffi::c_int;
        }
        if *window.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '+' as i32 {
            (*fs).wl = winlink_next_by_number((*s).curw, s, n);
        } else {
            (*fs).wl = winlink_previous_by_number((*s).curw, s, n);
        }
        if !(*fs).wl.is_null() {
            (*fs).idx = (*(*fs).wl).idx;
            (*fs).w = (*(*fs).wl).window;
            return 0 as ::core::ffi::c_int;
        }
    }
    if exact == 0 {
        if strcmp(window, b"!\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            (*fs).wl = (*(*fs).s).lastw.tqh_first;
            if (*fs).wl.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).idx = (*(*fs).wl).idx;
            (*fs).w = (*(*fs).wl).window;
            return 0 as ::core::ffi::c_int;
        } else if strcmp(window, b"^\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            (*fs).wl = winlinks_RB_MINMAX(&raw mut (*(*fs).s).windows, RB_NEGINF);
            if (*fs).wl.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).idx = (*(*fs).wl).idx;
            (*fs).w = (*(*fs).wl).window;
            return 0 as ::core::ffi::c_int;
        } else if strcmp(window, b"$\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            (*fs).wl = winlinks_RB_MINMAX(&raw mut (*(*fs).s).windows, RB_INF);
            if (*fs).wl.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).idx = (*(*fs).wl).idx;
            (*fs).w = (*(*fs).wl).window;
            return 0 as ::core::ffi::c_int;
        }
    }
    if *window.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '+' as i32
        && *window.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '-' as i32
    {
        idx = strtonum(
            window,
            0 as ::core::ffi::c_longlong,
            INT_MAX as ::core::ffi::c_longlong,
            &raw mut errstr,
        ) as ::core::ffi::c_int;
        if errstr.is_null() {
            (*fs).wl = winlink_find_by_index(&raw mut (*(*fs).s).windows, idx);
            if !(*fs).wl.is_null() {
                (*fs).idx = (*(*fs).wl).idx;
                (*fs).w = (*(*fs).wl).window;
                return 0 as ::core::ffi::c_int;
            }
            if (*fs).flags & CMD_FIND_WINDOW_INDEX != 0 {
                (*fs).idx = idx;
                return 0 as ::core::ffi::c_int;
            }
        }
    }
    (*fs).wl = ::core::ptr::null_mut::<winlink>();
    wl = winlinks_RB_MINMAX(&raw mut (*(*fs).s).windows, RB_NEGINF);
    while !wl.is_null() {
        if strcmp(window, (*(*wl).window).name) == 0 as ::core::ffi::c_int {
            if !(*fs).wl.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).wl = wl;
        }
        wl = winlinks_RB_NEXT(wl);
    }
    if !(*fs).wl.is_null() {
        (*fs).idx = (*(*fs).wl).idx;
        (*fs).w = (*(*fs).wl).window;
        return 0 as ::core::ffi::c_int;
    }
    if exact != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).wl = ::core::ptr::null_mut::<winlink>();
    wl = winlinks_RB_MINMAX(&raw mut (*(*fs).s).windows, RB_NEGINF);
    while !wl.is_null() {
        if strncmp(window, (*(*wl).window).name, strlen(window)) == 0 as ::core::ffi::c_int {
            if !(*fs).wl.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).wl = wl;
        }
        wl = winlinks_RB_NEXT(wl);
    }
    if !(*fs).wl.is_null() {
        (*fs).idx = (*(*fs).wl).idx;
        (*fs).w = (*(*fs).wl).window;
        return 0 as ::core::ffi::c_int;
    }
    (*fs).wl = ::core::ptr::null_mut::<winlink>();
    wl = winlinks_RB_MINMAX(&raw mut (*(*fs).s).windows, RB_NEGINF);
    while !wl.is_null() {
        if fnmatch(window, (*(*wl).window).name, 0 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int
        {
            if !(*fs).wl.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).wl = wl;
        }
        wl = winlinks_RB_NEXT(wl);
    }
    if !(*fs).wl.is_null() {
        (*fs).idx = (*(*fs).wl).idx;
        (*fs).w = (*(*fs).wl).window;
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn cmd_find_get_pane(
    mut fs: *mut cmd_find_state,
    mut pane: *const ::core::ffi::c_char,
    mut only: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_get_pane\0" as *const u8 as *const ::core::ffi::c_char,
        pane,
    );
    if *pane as ::core::ffi::c_int == '%' as i32 {
        (*fs).wp = window_pane_find_by_id_str(pane);
        if (*fs).wp.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        (*fs).w = (*(*fs).wp).window as *mut window;
        return cmd_find_best_session_with_window(fs);
    }
    (*fs).s = (*(*fs).current).s;
    (*fs).wl = (*(*fs).current).wl;
    (*fs).idx = (*(*fs).current).idx;
    (*fs).w = (*(*fs).current).w;
    if cmd_find_get_pane_with_window(fs, pane) == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if only == 0
        && cmd_find_get_window(fs, pane, 0 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int
    {
        (*fs).wp = (*(*fs).w).active;
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn cmd_find_get_pane_with_session(
    mut fs: *mut cmd_find_state,
    mut pane: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_get_pane_with_session\0" as *const u8 as *const ::core::ffi::c_char,
        pane,
    );
    if *pane as ::core::ffi::c_int == '%' as i32 {
        (*fs).wp = window_pane_find_by_id_str(pane);
        if (*fs).wp.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        (*fs).w = (*(*fs).wp).window as *mut window;
        return cmd_find_best_winlink_with_window(fs);
    }
    (*fs).wl = (*(*fs).s).curw;
    (*fs).idx = (*(*fs).wl).idx;
    (*fs).w = (*(*fs).wl).window;
    return cmd_find_get_pane_with_window(fs, pane);
}
unsafe extern "C" fn cmd_find_get_pane_with_window(
    mut fs: *mut cmd_find_state,
    mut pane: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_int = 0;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut n: u_int = 0;
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_get_pane_with_window\0" as *const u8 as *const ::core::ffi::c_char,
        pane,
    );
    if *pane as ::core::ffi::c_int == '%' as i32 {
        (*fs).wp = window_pane_find_by_id_str(pane);
        if (*fs).wp.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        if (*(*fs).wp).window != (*fs).w {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    }
    if strcmp(pane, b"!\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        (*fs).wp = (*(*fs).w).last_panes.tqh_first;
        if (*fs).wp.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    } else if strcmp(
        pane,
        b"{up-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        (*fs).wp = window_pane_find_up((*(*fs).w).active);
        if (*fs).wp.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    } else if strcmp(
        pane,
        b"{down-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        (*fs).wp = window_pane_find_down((*(*fs).w).active);
        if (*fs).wp.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    } else if strcmp(
        pane,
        b"{left-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        (*fs).wp = window_pane_find_left((*(*fs).w).active);
        if (*fs).wp.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    } else if strcmp(
        pane,
        b"{right-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        (*fs).wp = window_pane_find_right((*(*fs).w).active);
        if (*fs).wp.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    }
    if *pane.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '+' as i32
        || *pane.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '-' as i32
    {
        if *pane.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32 {
            n = strtonum(
                pane.offset(1 as ::core::ffi::c_int as isize),
                1 as ::core::ffi::c_longlong,
                INT_MAX as ::core::ffi::c_longlong,
                &raw mut errstr,
            ) as u_int;
            if !errstr.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
        } else {
            n = 1 as u_int;
        }
        wp = (*(*fs).w).active;
        if *pane.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '+' as i32 {
            (*fs).wp = window_pane_next_by_number((*fs).w, wp, n);
        } else {
            (*fs).wp = window_pane_previous_by_number((*fs).w, wp, n);
        }
        if !(*fs).wp.is_null() {
            return 0 as ::core::ffi::c_int;
        }
    }
    idx = strtonum(
        pane,
        0 as ::core::ffi::c_longlong,
        INT_MAX as ::core::ffi::c_longlong,
        &raw mut errstr,
    ) as ::core::ffi::c_int;
    if errstr.is_null() {
        (*fs).wp = window_pane_at_index((*fs).w, idx as u_int);
        if !(*fs).wp.is_null() {
            return 0 as ::core::ffi::c_int;
        }
    }
    (*fs).wp = window_find_string((*fs).w, pane);
    if !(*fs).wp.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_clear_state(
    mut fs: *mut cmd_find_state,
    mut flags: ::core::ffi::c_int,
) {
    memset(
        fs as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_find_state>() as size_t,
    );
    (*fs).flags = flags;
    (*fs).idx = -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_empty_state(mut fs: *mut cmd_find_state) -> ::core::ffi::c_int {
    if (*fs).s.is_null() && (*fs).wl.is_null() && (*fs).w.is_null() && (*fs).wp.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_valid_state(mut fs: *mut cmd_find_state) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if (*fs).s.is_null() || (*fs).wl.is_null() || (*fs).w.is_null() || (*fs).wp.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if session_alive((*fs).s) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    wl = winlinks_RB_MINMAX(&raw mut (*(*fs).s).windows, RB_NEGINF);
    while !wl.is_null() {
        if (*wl).window == (*fs).w && wl == (*fs).wl {
            break;
        }
        wl = winlinks_RB_NEXT(wl);
    }
    if wl.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*fs).w != (*(*fs).wl).window {
        return 0 as ::core::ffi::c_int;
    }
    return window_has_pane((*fs).w, (*fs).wp);
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_copy_state(
    mut dst: *mut cmd_find_state,
    mut src: *mut cmd_find_state,
) {
    (*dst).s = (*src).s;
    (*dst).wl = (*src).wl;
    (*dst).idx = (*src).idx;
    (*dst).w = (*src).w;
    (*dst).wp = (*src).wp;
}
unsafe extern "C" fn cmd_find_log_state(
    mut prefix: *const ::core::ffi::c_char,
    mut fs: *mut cmd_find_state,
) {
    if !(*fs).s.is_null() {
        log_debug(
            b"%s: s=$%u %s\0" as *const u8 as *const ::core::ffi::c_char,
            prefix,
            (*(*fs).s).id,
            (*(*fs).s).name,
        );
    } else {
        log_debug(
            b"%s: s=none\0" as *const u8 as *const ::core::ffi::c_char,
            prefix,
        );
    }
    if !(*fs).wl.is_null() {
        log_debug(
            b"%s: wl=%u %d w=@%u %s\0" as *const u8 as *const ::core::ffi::c_char,
            prefix,
            (*(*fs).wl).idx,
            ((*(*fs).wl).window == (*fs).w) as ::core::ffi::c_int,
            (*(*fs).w).id,
            (*(*fs).w).name,
        );
    } else {
        log_debug(
            b"%s: wl=none\0" as *const u8 as *const ::core::ffi::c_char,
            prefix,
        );
    }
    if !(*fs).wp.is_null() {
        log_debug(
            b"%s: wp=%%%u\0" as *const u8 as *const ::core::ffi::c_char,
            prefix,
            (*(*fs).wp).id,
        );
    } else {
        log_debug(
            b"%s: wp=none\0" as *const u8 as *const ::core::ffi::c_char,
            prefix,
        );
    }
    if (*fs).idx != -(1 as ::core::ffi::c_int) {
        log_debug(
            b"%s: idx=%d\0" as *const u8 as *const ::core::ffi::c_char,
            prefix,
            (*fs).idx,
        );
    } else {
        log_debug(
            b"%s: idx=none\0" as *const u8 as *const ::core::ffi::c_char,
            prefix,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_from_session(
    mut fs: *mut cmd_find_state,
    mut s: *mut session,
    mut flags: ::core::ffi::c_int,
) {
    cmd_find_clear_state(fs, flags);
    (*fs).s = s;
    (*fs).wl = (*(*fs).s).curw;
    (*fs).w = (*(*fs).wl).window;
    (*fs).wp = (*(*fs).w).active;
    cmd_find_log_state(
        b"cmd_find_from_session\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_from_winlink(
    mut fs: *mut cmd_find_state,
    mut wl: *mut winlink,
    mut flags: ::core::ffi::c_int,
) {
    cmd_find_clear_state(fs, flags);
    (*fs).s = (*wl).session;
    (*fs).wl = wl;
    (*fs).w = (*wl).window;
    (*fs).wp = (*(*wl).window).active;
    cmd_find_log_state(
        b"cmd_find_from_winlink\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_from_session_window(
    mut fs: *mut cmd_find_state,
    mut s: *mut session,
    mut w: *mut window,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    cmd_find_clear_state(fs, flags);
    (*fs).s = s;
    (*fs).w = w;
    if cmd_find_best_winlink_with_window(fs) != 0 as ::core::ffi::c_int {
        cmd_find_clear_state(fs, flags);
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).wp = (*(*fs).w).active;
    cmd_find_log_state(
        b"cmd_find_from_session_window\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_from_window(
    mut fs: *mut cmd_find_state,
    mut w: *mut window,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    cmd_find_clear_state(fs, flags);
    (*fs).w = w;
    if cmd_find_best_session_with_window(fs) != 0 as ::core::ffi::c_int {
        cmd_find_clear_state(fs, flags);
        return -(1 as ::core::ffi::c_int);
    }
    if cmd_find_best_winlink_with_window(fs) != 0 as ::core::ffi::c_int {
        cmd_find_clear_state(fs, flags);
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).wp = (*(*fs).w).active;
    cmd_find_log_state(
        b"cmd_find_from_window\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_from_winlink_pane(
    mut fs: *mut cmd_find_state,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
    mut flags: ::core::ffi::c_int,
) {
    cmd_find_clear_state(fs, flags);
    (*fs).s = (*wl).session;
    (*fs).wl = wl;
    (*fs).idx = (*(*fs).wl).idx;
    (*fs).w = (*(*fs).wl).window;
    (*fs).wp = wp;
    cmd_find_log_state(
        b"cmd_find_from_winlink_pane\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_from_pane(
    mut fs: *mut cmd_find_state,
    mut wp: *mut window_pane,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if cmd_find_from_window(fs, (*wp).window as *mut window, flags) != 0 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).wp = wp;
    cmd_find_log_state(
        b"cmd_find_from_pane\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_from_nothing(
    mut fs: *mut cmd_find_state,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    cmd_find_clear_state(fs, flags);
    (*fs).s = cmd_find_best_session(::core::ptr::null_mut::<*mut session>(), 0 as u_int, flags);
    if (*fs).s.is_null() {
        cmd_find_clear_state(fs, flags);
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).wl = (*(*fs).s).curw;
    (*fs).idx = (*(*fs).wl).idx;
    (*fs).w = (*(*fs).wl).window;
    (*fs).wp = (*(*fs).w).active;
    cmd_find_log_state(
        b"cmd_find_from_nothing\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_from_mouse(
    mut fs: *mut cmd_find_state,
    mut m: *mut mouse_event,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    cmd_find_clear_state(fs, flags);
    if (*m).valid == 0 {
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).wp = cmd_mouse_pane(m, &raw mut (*fs).s, &raw mut (*fs).wl);
    if (*fs).wp.is_null() {
        cmd_find_clear_state(fs, flags);
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).w = (*(*fs).wl).window;
    cmd_find_log_state(
        b"cmd_find_from_mouse\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_from_client(
    mut fs: *mut cmd_find_state,
    mut c: *mut client,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if c.is_null() {
        return cmd_find_from_nothing(fs, flags);
    }
    if !(*c).session.is_null() {
        cmd_find_clear_state(fs, flags);
        (*fs).wp = (*(*(*(*c).session).curw).window).active;
        if (*fs).wp.is_null() {
            cmd_find_from_session(fs, (*c).session, flags);
            return 0 as ::core::ffi::c_int;
        }
        (*fs).s = (*c).session;
        (*fs).wl = (*(*fs).s).curw;
        (*fs).w = (*(*fs).wl).window;
        cmd_find_log_state(
            b"cmd_find_from_client\0" as *const u8 as *const ::core::ffi::c_char,
            fs,
        );
        return 0 as ::core::ffi::c_int;
    }
    cmd_find_clear_state(fs, flags);
    wp = cmd_find_inside_pane(c);
    if !wp.is_null() {
        (*fs).w = (*wp).window as *mut window;
        if !(cmd_find_best_session_with_window(fs) != 0 as ::core::ffi::c_int) {
            (*fs).wl = (*(*fs).s).curw;
            (*fs).w = (*(*fs).wl).window;
            (*fs).wp = (*(*fs).w).active;
            cmd_find_log_state(
                b"cmd_find_from_client\0" as *const u8 as *const ::core::ffi::c_char,
                fs,
            );
            return 0 as ::core::ffi::c_int;
        }
    }
    return cmd_find_from_nothing(fs, flags);
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_target(
    mut fs: *mut cmd_find_state,
    mut item: *mut cmdq_item,
    mut target: *const ::core::ffi::c_char,
    mut type_0: cmd_find_type,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut m: *mut mouse_event = ::core::ptr::null_mut::<mouse_event>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut current: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut colon: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut period: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tmp: [::core::ffi::c_char; 256] = [0; 256];
    let mut session: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut window: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut pane: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut window_only: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut pane_only: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if flags & CMD_FIND_CANFAIL != 0 {
        flags |= CMD_FIND_QUIET;
    }
    if type_0 as ::core::ffi::c_uint == CMD_FIND_PANE as ::core::ffi::c_int as ::core::ffi::c_uint {
        s = b"pane\0" as *const u8 as *const ::core::ffi::c_char;
    } else if type_0 as ::core::ffi::c_uint
        == CMD_FIND_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        s = b"window\0" as *const u8 as *const ::core::ffi::c_char;
    } else if type_0 as ::core::ffi::c_uint
        == CMD_FIND_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        s = b"session\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        s = b"unknown\0" as *const u8 as *const ::core::ffi::c_char;
    }
    *(&raw mut tmp as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    if flags & CMD_FIND_PREFER_UNATTACHED != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"PREFER_UNATTACHED,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & CMD_FIND_QUIET != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"QUIET,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & CMD_FIND_WINDOW_INDEX != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"WINDOW_INDEX,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & CMD_FIND_DEFAULT_MARKED != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"DEFAULT_MARKED,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & CMD_FIND_EXACT_SESSION != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"EXACT_SESSION,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & CMD_FIND_EXACT_WINDOW != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"EXACT_WINDOW,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & CMD_FIND_CANFAIL != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"CANFAIL,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if *(&raw mut tmp as *mut ::core::ffi::c_char) as ::core::ffi::c_int != '\0' as i32 {
        tmp[strlen(&raw mut tmp as *mut ::core::ffi::c_char).wrapping_sub(1 as size_t) as usize] =
            '\0' as i32 as ::core::ffi::c_char;
    } else {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"NONE\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    log_debug(
        b"%s: target %s, type %s, item %p, flags %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_target\0" as *const u8 as *const ::core::ffi::c_char,
        if target.is_null() {
            b"none\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            target
        },
        s,
        item,
        &raw mut tmp as *mut ::core::ffi::c_char,
    );
    cmd_find_clear_state(fs, flags);
    if server_check_marked() != 0 && flags & CMD_FIND_DEFAULT_MARKED != 0 {
        (*fs).current = &raw mut marked_pane;
        log_debug(
            b"%s: current is marked pane\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_find_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
        current_block = 1836292691772056875;
    } else if cmd_find_valid_state(cmdq_get_current(item)) != 0 {
        (*fs).current = cmdq_get_current(item);
        log_debug(
            b"%s: current is from queue\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_find_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
        current_block = 1836292691772056875;
    } else if cmd_find_from_client(&raw mut current, cmdq_get_client(item), flags)
        == 0 as ::core::ffi::c_int
    {
        (*fs).current = &raw mut current;
        log_debug(
            b"%s: current is from client\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_find_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
        current_block = 1836292691772056875;
    } else {
        if !flags & CMD_FIND_QUIET != 0 {
            cmdq_error(
                item,
                b"no current target\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        current_block = 5193823237153215208;
    }
    match current_block {
        1836292691772056875 => {
            if cmd_find_valid_state((*fs).current) == 0 {
                fatalx(b"invalid current find state\0" as *const u8 as *const ::core::ffi::c_char);
            }
            if target.is_null() || *target as ::core::ffi::c_int == '\0' as i32 {
                current_block = 6284300254771030961;
            } else if strcmp(target, b"@\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
                || strcmp(
                    target,
                    b"{active}\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                || strcmp(
                    target,
                    b"{current}\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
            {
                c = cmdq_get_client(item);
                if c.is_null() || (*c).session.is_null() {
                    cmdq_error(
                        item,
                        b"no current client\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                    current_block = 5193823237153215208;
                } else {
                    (*fs).wl = (*(*c).session).curw;
                    (*fs).wp = (*(*(*(*c).session).curw).window).active;
                    (*fs).w = (*(*(*c).session).curw).window;
                    current_block = 15319680530019787978;
                }
            } else if strcmp(target, b"=\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
                || strcmp(
                    target,
                    b"{mouse}\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
            {
                m = &raw mut (*(cmdq_get_event
                    as unsafe extern "C" fn(*mut cmdq_item) -> *mut key_event)(
                    item
                ))
                .m;
                let mut current_block_56: u64;
                match type_0 as ::core::ffi::c_uint {
                    0 => {
                        (*fs).wp = cmd_mouse_pane(m, &raw mut (*fs).s, &raw mut (*fs).wl);
                        if !(*fs).wp.is_null() {
                            (*fs).w = (*(*fs).wl).window;
                            current_block_56 = 7343950298149844727;
                        } else {
                            current_block_56 = 2308649987175926278;
                        }
                    }
                    1 | 2 => {
                        current_block_56 = 2308649987175926278;
                    }
                    _ => {
                        current_block_56 = 7343950298149844727;
                    }
                }
                match current_block_56 {
                    2308649987175926278 => {
                        (*fs).wl = cmd_mouse_window(m, &raw mut (*fs).s);
                        if (*fs).wl.is_null() && !(*fs).s.is_null() {
                            (*fs).wl = (*(*fs).s).curw;
                        }
                        if !(*fs).wl.is_null() {
                            (*fs).w = (*(*fs).wl).window;
                            (*fs).wp = (*(*fs).w).active;
                        }
                    }
                    _ => {}
                }
                if (*fs).wp.is_null() {
                    if !flags & CMD_FIND_QUIET != 0 {
                        cmdq_error(
                            item,
                            b"no mouse target\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                    }
                    current_block = 5193823237153215208;
                } else {
                    current_block = 15319680530019787978;
                }
            } else if strcmp(target, b"~\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
                || strcmp(
                    target,
                    b"{marked}\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
            {
                if server_check_marked() == 0 {
                    if !flags & CMD_FIND_QUIET != 0 {
                        cmdq_error(
                            item,
                            b"no marked target\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                    }
                    current_block = 5193823237153215208;
                } else {
                    cmd_find_copy_state(fs, &raw mut marked_pane);
                    current_block = 15319680530019787978;
                }
            } else {
                copy = xstrdup(target);
                colon = strchr(copy, ':' as i32);
                if !colon.is_null() {
                    let fresh0 = colon;
                    colon = colon.offset(1);
                    *fresh0 = '\0' as i32 as ::core::ffi::c_char;
                }
                if colon.is_null() {
                    period = strchr(copy, '.' as i32);
                } else {
                    period = strchr(colon, '.' as i32);
                }
                if !period.is_null() {
                    let fresh1 = period;
                    period = period.offset(1);
                    *fresh1 = '\0' as i32 as ::core::ffi::c_char;
                }
                pane = ::core::ptr::null::<::core::ffi::c_char>();
                window = pane;
                session = window;
                if !colon.is_null() && !period.is_null() {
                    session = copy;
                    window = colon;
                    window_only = 1 as ::core::ffi::c_int;
                    pane = period;
                    pane_only = 1 as ::core::ffi::c_int;
                } else if !colon.is_null() && period.is_null() {
                    session = copy;
                    window = colon;
                    window_only = 1 as ::core::ffi::c_int;
                } else if colon.is_null() && !period.is_null() {
                    window = copy;
                    pane = period;
                    pane_only = 1 as ::core::ffi::c_int;
                } else if *copy as ::core::ffi::c_int == '$' as i32 {
                    session = copy;
                } else if *copy as ::core::ffi::c_int == '@' as i32 {
                    window = copy;
                } else if *copy as ::core::ffi::c_int == '%' as i32 {
                    pane = copy;
                } else {
                    match type_0 as ::core::ffi::c_uint {
                        2 => {
                            session = copy;
                        }
                        1 => {
                            window = copy;
                        }
                        0 => {
                            pane = copy;
                        }
                        _ => {}
                    }
                }
                if !session.is_null() && *session as ::core::ffi::c_int == '=' as i32 {
                    session = session.offset(1);
                    (*fs).flags |= CMD_FIND_EXACT_SESSION;
                }
                if !window.is_null() && *window as ::core::ffi::c_int == '=' as i32 {
                    window = window.offset(1);
                    (*fs).flags |= CMD_FIND_EXACT_WINDOW;
                }
                if !session.is_null() && *session as ::core::ffi::c_int == '\0' as i32 {
                    session = ::core::ptr::null::<::core::ffi::c_char>();
                }
                if !window.is_null() && *window as ::core::ffi::c_int == '\0' as i32 {
                    window = ::core::ptr::null::<::core::ffi::c_char>();
                }
                if !pane.is_null() && *pane as ::core::ffi::c_int == '\0' as i32 {
                    pane = ::core::ptr::null::<::core::ffi::c_char>();
                }
                if !session.is_null() {
                    session = cmd_find_map_table(
                        &raw mut cmd_find_session_table as *mut [*const ::core::ffi::c_char; 2],
                        session,
                    );
                }
                if !window.is_null() {
                    window = cmd_find_map_table(
                        &raw mut cmd_find_window_table as *mut [*const ::core::ffi::c_char; 2],
                        window,
                    );
                }
                if !pane.is_null() {
                    pane = cmd_find_map_table(
                        &raw mut cmd_find_pane_table as *mut [*const ::core::ffi::c_char; 2],
                        pane,
                    );
                }
                if !session.is_null() || !window.is_null() || !pane.is_null() {
                    log_debug(
                        b"%s: target %s is %s%s%s%s%s%s\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"cmd_find_target\0" as *const u8 as *const ::core::ffi::c_char,
                        target,
                        if session.is_null() {
                            b"\0" as *const u8 as *const ::core::ffi::c_char
                        } else {
                            b"session \0" as *const u8 as *const ::core::ffi::c_char
                        },
                        if session.is_null() {
                            b"\0" as *const u8 as *const ::core::ffi::c_char
                        } else {
                            session
                        },
                        if window.is_null() {
                            b"\0" as *const u8 as *const ::core::ffi::c_char
                        } else {
                            b"window \0" as *const u8 as *const ::core::ffi::c_char
                        },
                        if window.is_null() {
                            b"\0" as *const u8 as *const ::core::ffi::c_char
                        } else {
                            window
                        },
                        if pane.is_null() {
                            b"\0" as *const u8 as *const ::core::ffi::c_char
                        } else {
                            b"pane \0" as *const u8 as *const ::core::ffi::c_char
                        },
                        if pane.is_null() {
                            b"\0" as *const u8 as *const ::core::ffi::c_char
                        } else {
                            pane
                        },
                    );
                }
                if !pane.is_null() && flags & CMD_FIND_WINDOW_INDEX != 0 {
                    if !flags & CMD_FIND_QUIET != 0 {
                        cmdq_error(
                            item,
                            b"can't specify pane here\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                    }
                    current_block = 5193823237153215208;
                } else {
                    if !session.is_null() {
                        if cmd_find_get_session(fs, session) != 0 as ::core::ffi::c_int {
                            if !flags & CMD_FIND_QUIET != 0 {
                                cmdq_error(
                                    item,
                                    b"can't find session: %s\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    session,
                                );
                            }
                            current_block = 5193823237153215208;
                        } else if window.is_null() && pane.is_null() {
                            (*fs).wl = (*(*fs).s).curw;
                            (*fs).idx = -(1 as ::core::ffi::c_int);
                            (*fs).w = (*(*fs).wl).window;
                            (*fs).wp = (*(*fs).w).active;
                            current_block = 15319680530019787978;
                        } else if !window.is_null() && pane.is_null() {
                            if cmd_find_get_window_with_session(fs, window)
                                != 0 as ::core::ffi::c_int
                            {
                                current_block = 2743676411188200708;
                            } else {
                                if !(*fs).wl.is_null() {
                                    (*fs).wp = (*(*(*fs).wl).window).active;
                                }
                                current_block = 15319680530019787978;
                            }
                        } else if window.is_null() && !pane.is_null() {
                            if cmd_find_get_pane_with_session(fs, pane) != 0 as ::core::ffi::c_int {
                                current_block = 14917847580669770662;
                            } else {
                                current_block = 15319680530019787978;
                            }
                        } else if cmd_find_get_window_with_session(fs, window)
                            != 0 as ::core::ffi::c_int
                        {
                            current_block = 2743676411188200708;
                        } else if cmd_find_get_pane_with_window(fs, pane) != 0 as ::core::ffi::c_int
                        {
                            current_block = 14917847580669770662;
                        } else {
                            current_block = 15319680530019787978;
                        }
                    } else if !window.is_null() && !pane.is_null() {
                        if cmd_find_get_window(fs, window, window_only) != 0 as ::core::ffi::c_int {
                            current_block = 2743676411188200708;
                        } else if cmd_find_get_pane_with_window(fs, pane) != 0 as ::core::ffi::c_int
                        {
                            current_block = 14917847580669770662;
                        } else {
                            current_block = 15319680530019787978;
                        }
                    } else if !window.is_null() && pane.is_null() {
                        if cmd_find_get_window(fs, window, window_only) != 0 as ::core::ffi::c_int {
                            current_block = 2743676411188200708;
                        } else {
                            if !(*fs).wl.is_null() {
                                (*fs).wp = (*(*(*fs).wl).window).active;
                            }
                            current_block = 15319680530019787978;
                        }
                    } else if window.is_null() && !pane.is_null() {
                        if cmd_find_get_pane(fs, pane, pane_only) != 0 as ::core::ffi::c_int {
                            current_block = 14917847580669770662;
                        } else {
                            current_block = 15319680530019787978;
                        }
                    } else {
                        current_block = 6284300254771030961;
                    }
                    match current_block {
                        5193823237153215208 => {}
                        15319680530019787978 => {}
                        6284300254771030961 => {}
                        _ => {
                            match current_block {
                                2743676411188200708 => {
                                    if !flags & CMD_FIND_QUIET != 0 {
                                        cmdq_error(
                                            item,
                                            b"can't find window: %s\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            window,
                                        );
                                    }
                                }
                                _ => {
                                    if !flags & CMD_FIND_QUIET != 0 {
                                        cmdq_error(
                                            item,
                                            b"can't find pane: %s\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            pane,
                                        );
                                    }
                                }
                            }
                            current_block = 5193823237153215208;
                        }
                    }
                }
            }
            match current_block {
                5193823237153215208 => {}
                _ => {
                    match current_block {
                        6284300254771030961 => {
                            cmd_find_copy_state(fs, (*fs).current);
                            if flags & CMD_FIND_WINDOW_INDEX != 0 {
                                (*fs).idx = -(1 as ::core::ffi::c_int);
                            }
                        }
                        _ => {}
                    }
                    (*fs).current = ::core::ptr::null_mut::<cmd_find_state>();
                    cmd_find_log_state(
                        b"cmd_find_target\0" as *const u8 as *const ::core::ffi::c_char,
                        fs,
                    );
                    free(copy as *mut ::core::ffi::c_void);
                    return 0 as ::core::ffi::c_int;
                }
            }
        }
        _ => {}
    }
    (*fs).current = ::core::ptr::null_mut::<cmd_find_state>();
    log_debug(
        b"%s: error\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_target\0" as *const u8 as *const ::core::ffi::c_char,
    );
    free(copy as *mut ::core::ffi::c_void);
    if flags & CMD_FIND_CANFAIL != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn cmd_find_current_client(
    mut item: *mut cmdq_item,
    mut quiet: ::core::ffi::c_int,
) -> *mut client {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut found: *mut client = ::core::ptr::null_mut::<client>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    if !item.is_null() {
        c = cmdq_get_client(item);
    }
    if !c.is_null() && !(*c).session.is_null() {
        return c;
    }
    found = ::core::ptr::null_mut::<client>();
    if !c.is_null() && {
        wp = cmd_find_inside_pane(c);
        !wp.is_null()
    } {
        cmd_find_clear_state(&raw mut fs, CMD_FIND_QUIET);
        fs.w = (*wp).window as *mut window;
        if cmd_find_best_session_with_window(&raw mut fs) == 0 as ::core::ffi::c_int {
            found = cmd_find_best_client(fs.s);
        }
    } else {
        s = cmd_find_best_session(
            ::core::ptr::null_mut::<*mut session>(),
            0 as u_int,
            CMD_FIND_QUIET,
        );
        if !s.is_null() {
            found = cmd_find_best_client(s);
        }
    }
    if found.is_null() && !item.is_null() && quiet == 0 {
        cmdq_error(
            item,
            b"no current client\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    log_debug(
        b"%s: no target, return %p\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_current_client\0" as *const u8 as *const ::core::ffi::c_char,
        found,
    );
    return found;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_client(
    mut item: *mut cmdq_item,
    mut target: *const ::core::ffi::c_char,
    mut quiet: ::core::ffi::c_int,
) -> *mut client {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut size: size_t = 0;
    if target.is_null() {
        return cmd_find_current_client(item, quiet);
    }
    copy = xstrdup(target);
    size = strlen(copy);
    if size != 0 as size_t
        && *copy.offset(size.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int == ':' as i32
    {
        *copy.offset(size.wrapping_sub(1 as size_t) as isize) = '\0' as i32 as ::core::ffi::c_char;
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if !(*c).session.is_null() {
            if strcmp(copy, (*c).name) == 0 as ::core::ffi::c_int {
                break;
            }
            if !(*(*c).ttyname as ::core::ffi::c_int == '\0' as i32) {
                if strcmp(copy, (*c).ttyname) == 0 as ::core::ffi::c_int {
                    break;
                }
                if !(strncmp(
                    (*c).ttyname,
                    _PATH_DEV.as_ptr(),
                    (::core::mem::size_of::<[::core::ffi::c_char; 6]>() as size_t)
                        .wrapping_sub(1 as size_t),
                ) != 0 as ::core::ffi::c_int)
                {
                    if strcmp(
                        copy,
                        (*c).ttyname
                            .offset(::core::mem::size_of::<[::core::ffi::c_char; 6]>() as usize
                                as isize)
                            .offset(-(1 as ::core::ffi::c_int as isize)),
                    ) == 0 as ::core::ffi::c_int
                    {
                        break;
                    }
                }
            }
        }
        c = (*c).entry.tqe_next;
    }
    if c.is_null() && quiet == 0 {
        cmdq_error(
            item,
            b"can't find client: %s\0" as *const u8 as *const ::core::ffi::c_char,
            copy,
        );
    }
    free(copy as *mut ::core::ffi::c_void);
    log_debug(
        b"%s: target %s, return %p\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_client\0" as *const u8 as *const ::core::ffi::c_char,
        target,
        c,
    );
    return c;
}
