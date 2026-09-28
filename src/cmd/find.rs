use crate::src::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_state_owned, cmdq_get_event};
use crate::src::cmd::{cmd_mouse_pane, cmd_mouse_window};
use crate::src::compat::strtonum::strtonum;
use crate::src::environ::environ_find;
use crate::src::ffi::libc::{fnmatch, strchr, strcmp, strlcat, strlen, strncmp};
use crate::src::format::bytes::write_cstr;
use crate::src::log::{fatalx, log_cstr, log_debug, log_pointer};
use crate::src::server::clients;
use crate::src::server::{marked_pane, server_check_marked};
use crate::src::session::sessions;
use crate::src::session::{
    session_alive, session_find, session_find_by_id_str, session_has, sessions_minmax,
    sessions_next,
};
use crate::src::window::{
    all_window_panes, window_find_by_id_str, window_find_string, window_has_pane,
    window_pane_at_index, window_pane_find_by_id_str, window_pane_find_down, window_pane_find_left,
    window_pane_find_right, window_pane_find_up, window_pane_next_by_number,
    window_pane_previous_by_number, window_pane_stack_first, window_pane_tree_minmax,
    window_pane_tree_next, winlink_find_by_index, winlink_next_by_number,
    winlink_previous_by_number, winlinks_minmax, winlinks_next,
};
use std::ffi::{CStr, CString};

use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd_find_state, cmdq_item};
use crate::src::shared::command::{
    CMD_FIND_CANFAIL, CMD_FIND_DEFAULT_MARKED, CMD_FIND_EXACT_SESSION, CMD_FIND_EXACT_WINDOW,
    CMD_FIND_PREFER_UNATTACHED, CMD_FIND_QUIET, CMD_FIND_WINDOW_INDEX,
};
use crate::src::shared::environment::environ_entry;
use crate::src::shared::key::key_event;
use crate::src::shared::limits::INT_MAX;
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::tree::{RB_INF, RB_NEGINF};
use crate::src::shared::window::{window, winlink};

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
unsafe fn cmd_find_inside_pane(mut c: *mut client) -> Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
    let mut inside_pane_owner = None;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut envent: Option<&environ_entry> = None;
    if c.is_null() {
        return None;
    }
    let mut indexed_pane_owner = window_pane_tree_minmax(&*std::ptr::addr_of!(all_window_panes));
    wp = indexed_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !wp.is_null() {
        if (*wp).fd != -(1 as ::core::ffi::c_int)
            && strcmp(
                &raw mut (*wp).tty as *mut ::core::ffi::c_char,
                ((*c).ttyname)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            ) == 0 as ::core::ffi::c_int
        {
            break;
        }
        indexed_pane_owner = window_pane_tree_next(&*wp);
        wp = indexed_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    }
    if wp.is_null() {
        envent = environ_find(
            (*c).environ.as_deref().expect("environment"),
            b"TMUX_PANE\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if !envent.is_none() {
            inside_pane_owner = envent.unwrap().value.as_deref().and_then(|value| window_pane_find_by_id_str(value));
            wp = inside_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        }
    }
    if !wp.is_null() {
        log_debug(format_args!(
            "{}: got pane %{} ({})",
            "cmd_find_inside_pane",
            ((*wp).id) as u32,
            log_cstr((&raw mut (*wp).tty as *mut ::core::ffi::c_char) as *const _)
        ));
    }
    if inside_pane_owner.is_none() {
        inside_pane_owner = indexed_pane_owner;
    }
    return inside_pane_owner;
}
fn cmd_find_client_better(c: &client, than: Option<&client>) -> bool {
    than.is_none_or(|than| (c.activity_time.tv_sec, c.activity_time.tv_usec)
        > (than.activity_time.tv_sec, than.activity_time.tv_usec))
}
pub unsafe fn cmd_find_best_client(s: &session) -> Option<std::rc::Rc<std::cell::UnsafeCell<client>>> {
    let mut best: Option<std::rc::Rc<std::cell::UnsafeCell<client>>> = None;
    let mut cursor = clients.first();
    while let Some(owner) = cursor {
        cursor = clients.next(&owner);
        let candidate = &*owner.get();
        let Some(attached_session) = candidate.session.as_ref() else { continue; };
        if s.attached != 0 && !attached_session.observer.ptr_eq(&s.observer) { continue; }
        if cmd_find_client_better(candidate, best.as_ref().map(|owner| &*owner.get())) {
            best = Some(owner);
        }
    }
    best
}
fn cmd_find_session_better(s: &session, than: Option<&session>, flags: ::core::ffi::c_int) -> bool {
    let Some(than) = than else { return true; };
    if flags & CMD_FIND_PREFER_UNATTACHED != 0 {
        if than.attached != 0 && s.attached == 0 { return true; }
        if than.attached == 0 && s.attached != 0 { return false; }
    }
    (s.activity_time.tv_sec, s.activity_time.tv_usec)
        > (than.activity_time.tv_sec, than.activity_time.tv_usec)
}
unsafe fn cmd_find_session_valid(s: &session) -> ::core::ffi::c_int {
    if session_alive(Some(s)) == 0
        || (*s).curw.is_null()
        || (*(*s).curw).window_ptr().is_null()
        || (*(*(*s).curw).window_ptr()).active.is_null()
    {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn cmd_find_best_session(
    candidates: Option<&[std::rc::Rc<std::cell::UnsafeCell<session>>]>,
    flags: ::core::ffi::c_int,
) -> Option<std::rc::Rc<std::cell::UnsafeCell<session>>> {
    let mut all = Vec::new();
    let candidates = match candidates {
        Some(candidates) => candidates,
        None => {
            let mut cursor = sessions_minmax(&*std::ptr::addr_of!(sessions));
            while let Some(owner) = cursor {
                cursor = sessions_next(&*owner.get());
                all.push(owner);
            }
            &all
        }
    };
    log_debug(format_args!("cmd_find_best_session: {} sessions to try", candidates.len()));
    let mut best: Option<std::rc::Rc<std::cell::UnsafeCell<session>>> = None;
    for owner in candidates {
        if cmd_find_session_valid(&*owner.get()) != 0
            && cmd_find_session_better(&*owner.get(), best.as_ref().map(|owner| &*owner.get()), flags)
        {
            best = Some(owner.clone());
        }
    }
    best
}
unsafe fn cmd_find_best_session_with_window(fs: *mut cmd_find_state) -> ::core::ffi::c_int {
    let Some(window_owner) = (*fs).w.upgrade() else { return -1; };
    let mut candidates = Vec::new();
    let mut cursor = sessions_minmax(&*std::ptr::addr_of!(sessions));
    while let Some(owner) = cursor {
        cursor = sessions_next(&*owner.get());
        if session_has(&*owner.get(), &*window_owner.get()) != 0 {
            candidates.push(owner);
        }
    }
    let Some(best) = cmd_find_best_session(Some(&candidates), (*fs).flags) else { return -1; };
    (*fs).s = std::rc::Rc::downgrade(&best);
    cmd_find_best_winlink_with_window(fs)
}
unsafe fn cmd_find_best_winlink_with_window(mut fs: *mut cmd_find_state) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wl_loop: *mut winlink = ::core::ptr::null_mut::<winlink>();
    log_debug(format_args!(
        "{}: window is @{}",
        "cmd_find_best_winlink_with_window",
        ((*(*fs).w_ptr()).id) as u32
    ));
    wl = ::core::ptr::null_mut::<winlink>();
    if !(*(*fs).s_ptr()).curw.is_null() && (*(*(*fs).s_ptr()).curw).window_ptr() == (*fs).w_ptr() {
        wl = (*(*fs).s_ptr()).curw;
    } else {
        wl_loop = winlinks_minmax(&(*(*fs).s_ptr()).windows, RB_NEGINF);
        while !wl_loop.is_null() {
            if (*wl_loop).window_ptr() == (*fs).w_ptr() {
                wl = wl_loop;
                break;
            } else {
                wl_loop = winlinks_next(&*wl_loop);
            }
        }
    }
    if wl.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).set_wl(wl);
    (*fs).idx = (*(*fs).wl_ptr()).idx;
    return 0 as ::core::ffi::c_int;
}
unsafe fn cmd_find_map_table(
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
unsafe fn cmd_find_get_session(
    mut fs: *mut cmd_find_state,
    mut session: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut s_loop: *mut session = ::core::ptr::null_mut::<session>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    log_debug(format_args!(
        "{}: {}",
        "cmd_find_get_session",
        log_cstr((session) as *const _)
    ));
    if *session as ::core::ffi::c_int == '$' as i32 {
        (*fs).set_s(session_find_by_id_str(std::ffi::CStr::from_ptr(session)).as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr));
        if (*fs).s_ptr().is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    }
    (*fs).set_s(session_find(std::ffi::CStr::from_ptr(session)).as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr));
    if !(*fs).s_ptr().is_null() {
        return 0 as ::core::ffi::c_int;
    }
    let matched_client_owner = cmd_find_client(
        ::core::ptr::null_mut::<cmdq_item>(),
        session,
        1 as ::core::ffi::c_int,
    );
    c = matched_client_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if !c.is_null() && !(*c).session.is_null() {
        (*fs).set_s((*c).session);
        return 0 as ::core::ffi::c_int;
    }
    if (*fs).flags & CMD_FIND_EXACT_SESSION != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    s = ::core::ptr::null_mut::<session>();
    let mut s_loop_owner = sessions_minmax(&*std::ptr::addr_of!(sessions));
    s_loop = s_loop_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    while !s_loop.is_null() {
        if strncmp(
            session,
            ((*s_loop).name).as_ptr().cast_mut(),
            strlen(session),
        ) == 0 as ::core::ffi::c_int
        {
            if !s.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            s = s_loop;
        }
        s_loop_owner = sessions_next(&*s_loop);
        s_loop = s_loop_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    }
    if !s.is_null() {
        (*fs).set_s(s);
        return 0 as ::core::ffi::c_int;
    }
    s = ::core::ptr::null_mut::<session>();
    let mut s_loop_owner = sessions_minmax(&*std::ptr::addr_of!(sessions));
    s_loop = s_loop_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    while !s_loop.is_null() {
        if fnmatch(
            session,
            ((*s_loop).name).as_ptr().cast_mut(),
            0 as ::core::ffi::c_int,
        ) == 0 as ::core::ffi::c_int
        {
            if !s.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            s = s_loop;
        }
        s_loop_owner = sessions_next(&*s_loop);
        s_loop = s_loop_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    }
    if !s.is_null() {
        (*fs).set_s(s);
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe fn cmd_find_get_window(
    mut fs: *mut cmd_find_state,
    mut window: *const ::core::ffi::c_char,
    mut only: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    log_debug(format_args!(
        "{}: {}",
        "cmd_find_get_window",
        log_cstr((window) as *const _)
    ));
    if *window as ::core::ffi::c_int == '@' as i32 {
        let window_owner = window_find_by_id_str(window);
        (*fs).set_w(window_owner.as_ref().map_or(
            std::ptr::null_mut(),
            crate::src::shared::window::WindowOwner::as_ptr,
        ));
        if (*fs).w_ptr().is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return cmd_find_best_session_with_window(fs);
    }
    (*fs).s = (*(*fs).current).s.clone();
    if cmd_find_get_window_with_session(fs, window) == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if only == 0 && cmd_find_get_session(fs, window) == 0 as ::core::ffi::c_int {
        (*fs).set_wl((*(*fs).s_ptr()).curw);
        (*fs).set_w((*(*fs).wl_ptr()).window_ptr());
        if !(*fs).flags & CMD_FIND_WINDOW_INDEX != 0 {
            (*fs).idx = (*(*fs).wl_ptr()).idx;
        }
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe fn cmd_find_get_window_with_session(
    mut fs: *mut cmd_find_state,
    mut window: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut exact: ::core::ffi::c_int = 0;
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    log_debug(format_args!(
        "{}: {}",
        "cmd_find_get_window_with_session",
        log_cstr((window) as *const _)
    ));
    exact = (*fs).flags & CMD_FIND_EXACT_WINDOW;
    (*fs).set_wl((*(*fs).s_ptr()).curw);
    (*fs).set_w((*(*fs).wl_ptr()).window_ptr());
    if *window as ::core::ffi::c_int == '@' as i32 {
        let window_owner = window_find_by_id_str(window);
        (*fs).set_w(window_owner.as_ref().map_or(
            std::ptr::null_mut(),
            crate::src::shared::window::WindowOwner::as_ptr,
        ));
        if (*fs).w_ptr().is_null() || session_has(&*(*fs).s_ptr(), &*(*fs).w_ptr()) == 0 {
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
        s = (*fs).s_ptr();
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
            (*fs).set_wl(winlink_next_by_number((*s).curw, s, n));
        } else {
            (*fs).set_wl(winlink_previous_by_number((*s).curw, s, n));
        }
        if !(*fs).wl_ptr().is_null() {
            (*fs).idx = (*(*fs).wl_ptr()).idx;
            (*fs).set_w((*(*fs).wl_ptr()).window_ptr());
            return 0 as ::core::ffi::c_int;
        }
    }
    if exact == 0 {
        if strcmp(window, b"!\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            (*fs).set_wl(crate::src::window::winlink_stack_first(&(*(*fs).s_ptr()).lastw));
            if (*fs).wl_ptr().is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).idx = (*(*fs).wl_ptr()).idx;
            (*fs).set_w((*(*fs).wl_ptr()).window_ptr());
            return 0 as ::core::ffi::c_int;
        } else if strcmp(window, b"^\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            (*fs).set_wl(winlinks_minmax(&(*(*fs).s_ptr()).windows, RB_NEGINF));
            if (*fs).wl_ptr().is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).idx = (*(*fs).wl_ptr()).idx;
            (*fs).set_w((*(*fs).wl_ptr()).window_ptr());
            return 0 as ::core::ffi::c_int;
        } else if strcmp(window, b"$\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            (*fs).set_wl(winlinks_minmax(&(*(*fs).s_ptr()).windows, RB_INF));
            if (*fs).wl_ptr().is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).idx = (*(*fs).wl_ptr()).idx;
            (*fs).set_w((*(*fs).wl_ptr()).window_ptr());
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
            (*fs).set_wl(winlink_find_by_index(&raw mut (*(*fs).s_ptr()).windows, idx));
            if !(*fs).wl_ptr().is_null() {
                (*fs).idx = (*(*fs).wl_ptr()).idx;
                (*fs).set_w((*(*fs).wl_ptr()).window_ptr());
                return 0 as ::core::ffi::c_int;
            }
            if (*fs).flags & CMD_FIND_WINDOW_INDEX != 0 {
                (*fs).idx = idx;
                return 0 as ::core::ffi::c_int;
            }
        }
    }
    (*fs).set_wl(::core::ptr::null_mut::<winlink>());
    wl = winlinks_minmax(&(*(*fs).s_ptr()).windows, RB_NEGINF);
    while !wl.is_null() {
        if strcmp(window, (*(*wl).window_ptr()).name.as_ptr()) == 0 as ::core::ffi::c_int {
            if !(*fs).wl_ptr().is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).set_wl(wl);
        }
        wl = winlinks_next(&*wl);
    }
    if !(*fs).wl_ptr().is_null() {
        (*fs).idx = (*(*fs).wl_ptr()).idx;
        (*fs).set_w((*(*fs).wl_ptr()).window_ptr());
        return 0 as ::core::ffi::c_int;
    }
    if exact != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).set_wl(::core::ptr::null_mut::<winlink>());
    wl = winlinks_minmax(&(*(*fs).s_ptr()).windows, RB_NEGINF);
    while !wl.is_null() {
        if strncmp(window, (*(*wl).window_ptr()).name.as_ptr(), strlen(window)) == 0 as ::core::ffi::c_int
        {
            if !(*fs).wl_ptr().is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).set_wl(wl);
        }
        wl = winlinks_next(&*wl);
    }
    if !(*fs).wl_ptr().is_null() {
        (*fs).idx = (*(*fs).wl_ptr()).idx;
        (*fs).set_w((*(*fs).wl_ptr()).window_ptr());
        return 0 as ::core::ffi::c_int;
    }
    (*fs).set_wl(::core::ptr::null_mut::<winlink>());
    wl = winlinks_minmax(&(*(*fs).s_ptr()).windows, RB_NEGINF);
    while !wl.is_null() {
        if fnmatch(
            window,
            (*(*wl).window_ptr()).name.as_ptr(),
            0 as ::core::ffi::c_int,
        ) == 0 as ::core::ffi::c_int
        {
            if !(*fs).wl_ptr().is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).set_wl(wl);
        }
        wl = winlinks_next(&*wl);
    }
    if !(*fs).wl_ptr().is_null() {
        (*fs).idx = (*(*fs).wl_ptr()).idx;
        (*fs).set_w((*(*fs).wl_ptr()).window_ptr());
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe fn cmd_find_get_pane(
    mut fs: *mut cmd_find_state,
    mut pane: *const ::core::ffi::c_char,
    mut only: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    log_debug(format_args!(
        "{}: {}",
        "cmd_find_get_pane",
        log_cstr((pane) as *const _)
    ));
    if *pane as ::core::ffi::c_int == '%' as i32 {
        let pane_owner = window_pane_find_by_id_str(CStr::from_ptr(pane));
        (*fs).wp = pane_owner.as_ref().map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
        if (*fs).wp_ptr().is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        (*fs).set_w((*(*fs).wp_ptr()).window as *mut window);
        return cmd_find_best_session_with_window(fs);
    }
    (*fs).s = (*(*fs).current).s.clone();
    (*fs).wl = (*(*fs).current).wl.clone();
    (*fs).idx = (*(*fs).current).idx;
    (*fs).w = (*(*fs).current).w.clone();
    if cmd_find_get_pane_with_window(fs, pane) == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if only == 0
        && cmd_find_get_window(fs, pane, 0 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int
    {
        (*fs).set_wp((*(*fs).w_ptr()).active);
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe fn cmd_find_get_pane_with_session(
    mut fs: *mut cmd_find_state,
    mut pane: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    log_debug(format_args!(
        "{}: {}",
        "cmd_find_get_pane_with_session",
        log_cstr((pane) as *const _)
    ));
    if *pane as ::core::ffi::c_int == '%' as i32 {
        let pane_owner = window_pane_find_by_id_str(CStr::from_ptr(pane));
        (*fs).wp = pane_owner.as_ref().map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
        if (*fs).wp_ptr().is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        (*fs).set_w((*(*fs).wp_ptr()).window as *mut window);
        return cmd_find_best_winlink_with_window(fs);
    }
    (*fs).set_wl((*(*fs).s_ptr()).curw);
    (*fs).idx = (*(*fs).wl_ptr()).idx;
    (*fs).set_w((*(*fs).wl_ptr()).window_ptr());
    return cmd_find_get_pane_with_window(fs, pane);
}
unsafe fn cmd_find_get_pane_with_window(
    mut fs: *mut cmd_find_state,
    mut pane: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_int = 0;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut n: u_int = 0;
    log_debug(format_args!(
        "{}: {}",
        "cmd_find_get_pane_with_window",
        log_cstr((pane) as *const _)
    ));
    if *pane as ::core::ffi::c_int == '%' as i32 {
        let pane_owner = window_pane_find_by_id_str(CStr::from_ptr(pane));
        (*fs).wp = pane_owner.as_ref().map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
        if (*fs).wp_ptr().is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        if (*(*fs).wp_ptr()).window != (*fs).w_ptr() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    }
    if strcmp(pane, b"!\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        (*fs).set_wp(window_pane_stack_first(((*fs).w_ptr()).as_ref()).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()));
        if (*fs).wp_ptr().is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    } else if strcmp(
        pane,
        b"{up-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        let window_owner = (*fs).w.upgrade().expect("target window");
        let active = (*window_owner.get()).active.as_ref().and_then(|pane| pane.observer.upgrade());
        let selected = window_pane_find_up(active.as_ref());
        (*fs).wp = selected.as_ref().map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
        if (*fs).wp_ptr().is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    } else if strcmp(
        pane,
        b"{down-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        let window_owner = (*fs).w.upgrade().expect("target window");
        let active = (*window_owner.get()).active.as_ref().and_then(|pane| pane.observer.upgrade());
        let selected = window_pane_find_down(active.as_ref());
        (*fs).wp = selected.as_ref().map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
        if (*fs).wp_ptr().is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    } else if strcmp(
        pane,
        b"{left-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        let window_owner = (*fs).w.upgrade().expect("target window");
        let active = (*window_owner.get()).active.as_ref().and_then(|pane| pane.observer.upgrade());
        let selected = window_pane_find_left(active.as_ref());
        (*fs).wp = selected.as_ref().map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
        if (*fs).wp_ptr().is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    } else if strcmp(
        pane,
        b"{right-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        let window_owner = (*fs).w.upgrade().expect("target window");
        let active = (*window_owner.get()).active.as_ref().and_then(|pane| pane.observer.upgrade());
        let selected = window_pane_find_right(active.as_ref());
        (*fs).wp = selected.as_ref().map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
        if (*fs).wp_ptr().is_null() {
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
        let window_owner = (*fs).w.upgrade().expect("target window");
        let window = &*window_owner.get();
        let active_owner = window.active.as_ref().and_then(|pane| pane.observer.upgrade());
        let selected = if *pane == b'+' as std::ffi::c_char {
            window_pane_next_by_number(window, active_owner.as_ref(), n)
        } else {
            window_pane_previous_by_number(window, active_owner.as_ref(), n)
        };
        (*fs).wp = selected.as_ref().map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
        if !(*fs).wp_ptr().is_null() {
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
        let window_owner = (*fs).w.upgrade().expect("target window");
        let selected = window_pane_at_index(&mut *window_owner.get(), idx as u_int);
        (*fs).wp = selected.as_ref().map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
        if !(*fs).wp_ptr().is_null() {
            return 0 as ::core::ffi::c_int;
        }
    }
    let window_owner = (*fs).w.upgrade().expect("target window");
    let selected = window_find_string(&window_owner, CStr::from_ptr(pane));
    (*fs).wp = selected.as_ref().map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
    if !(*fs).wp_ptr().is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
pub unsafe fn cmd_find_clear_state(mut fs: *mut cmd_find_state, mut flags: ::core::ffi::c_int) {
    *fs = cmd_find_state {
        flags,
        idx: -1,
        ..Default::default()
    };
}
pub fn cmd_find_empty_state(fs: &cmd_find_state) -> ::core::ffi::c_int {
    (fs.s.ptr_eq(&std::rc::Weak::new())
        && fs.wl.is_empty()
        && fs.w.ptr_eq(&std::rc::Weak::new())
        && fs.wp.ptr_eq(&std::rc::Weak::new())) as ::core::ffi::c_int
}
pub unsafe fn cmd_find_valid_state(fs: &cmd_find_state) -> ::core::ffi::c_int {
    // Pin the Rc allocations while checking membership. No callbacks run here,
    // so the existing window owner remains responsible for close notification.
    let (Some(s), Some(w), Some(wp)) = (fs.s.upgrade(), fs.w.upgrade(), fs.wp.upgrade()) else {
        return 0;
    };
    let Ok(wl) = fs.wl.try_borrow_mut() else {
        return 0;
    };
    let s = crate::src::shared::rc::as_ptr(&s);
    let w = crate::src::shared::rc::as_ptr(&w);
    if session_alive(s.as_ref()) == 0 || wl.window_ptr() != w {
        return 0;
    }
    // Identity matters: a replacement at the same index is a different target.
    let member = winlink_find_by_index(&raw mut (*s).windows, wl.idx);
    if !std::ptr::eq(member, &*wl) {
        return 0;
    }
    window_has_pane(&*w, &fs.wp) as ::core::ffi::c_int
}
pub unsafe fn cmd_find_copy_state(mut dst: *mut cmd_find_state, mut src: *mut cmd_find_state) {
    let source = (*src).clone();
    (*dst).s = source.s;
    (*dst).wl = source.wl;
    (*dst).idx = source.idx;
    (*dst).w = source.w;
    (*dst).wp = source.wp;
}
unsafe fn cmd_find_log_state(mut prefix: *const ::core::ffi::c_char, mut fs: *mut cmd_find_state) {
    if !(*fs).s_ptr().is_null() {
        log_debug(format_args!(
            "{}: s=${} {}",
            log_cstr((prefix) as *const _),
            ((*(*fs).s_ptr()).id) as u32,
            log_cstr((((*(*fs).s_ptr()).name).as_ptr().cast_mut()) as *const _)
        ));
    } else {
        log_debug(format_args!("{}: s=none", log_cstr((prefix) as *const _)));
    }
    if !(*fs).wl_ptr().is_null() {
        log_debug(format_args!(
            "{}: wl={} {} w=@{} {}",
            log_cstr((prefix) as *const _),
            ((*(*fs).wl_ptr()).idx) as u32,
            ((*(*fs).wl_ptr()).window_ptr() == (*fs).w_ptr()) as ::core::ffi::c_int,
            ((*(*fs).w_ptr()).id) as u32,
            log_cstr(((*(*fs).w_ptr()).name.as_ptr()) as *const _)
        ));
    } else {
        log_debug(format_args!("{}: wl=none", log_cstr((prefix) as *const _)));
    }
    if !(*fs).wp_ptr().is_null() {
        log_debug(format_args!(
            "{}: wp=%{}",
            log_cstr((prefix) as *const _),
            ((*(*fs).wp_ptr()).id) as u32
        ));
    } else {
        log_debug(format_args!("{}: wp=none", log_cstr((prefix) as *const _)));
    }
    if (*fs).idx != -(1 as ::core::ffi::c_int) {
        log_debug(format_args!(
            "{}: idx={}",
            log_cstr((prefix) as *const _),
            ((*fs).idx) as i32
        ));
    } else {
        log_debug(format_args!("{}: idx=none", log_cstr((prefix) as *const _)));
    };
}
pub unsafe fn cmd_find_from_session(
    mut fs: *mut cmd_find_state,
    mut s: *mut session,
    mut flags: ::core::ffi::c_int,
) {
    cmd_find_clear_state(fs, flags);
    (*fs).set_s(s);
    (*fs).set_wl((*(*fs).s_ptr()).curw);
    (*fs).set_w((*(*fs).wl_ptr()).window_ptr());
    (*fs).set_wp((*(*fs).w_ptr()).active);
    cmd_find_log_state(
        b"cmd_find_from_session\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
}
pub unsafe fn cmd_find_from_winlink(
    mut fs: *mut cmd_find_state,
    mut wl: *mut winlink,
    mut flags: ::core::ffi::c_int,
) {
    cmd_find_clear_state(fs, flags);
    (*fs).s = (*wl).session.clone();
    (*fs).set_wl(wl);
    (*fs).set_w((*wl).window_ptr());
    (*fs).set_wp((*(*wl).window_ptr()).active);
    cmd_find_log_state(
        b"cmd_find_from_winlink\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
}
pub unsafe fn cmd_find_from_session_window(
    mut fs: *mut cmd_find_state,
    mut s: *mut session,
    mut w: *mut window,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    cmd_find_clear_state(fs, flags);
    (*fs).set_s(s);
    (*fs).set_w(w);
    if cmd_find_best_winlink_with_window(fs) != 0 as ::core::ffi::c_int {
        cmd_find_clear_state(fs, flags);
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).set_wp((*(*fs).w_ptr()).active);
    cmd_find_log_state(
        b"cmd_find_from_session_window\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn cmd_find_from_window(
    mut fs: *mut cmd_find_state,
    mut w: *mut window,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    cmd_find_clear_state(fs, flags);
    (*fs).set_w(w);
    if cmd_find_best_session_with_window(fs) != 0 as ::core::ffi::c_int {
        cmd_find_clear_state(fs, flags);
        return -(1 as ::core::ffi::c_int);
    }
    if cmd_find_best_winlink_with_window(fs) != 0 as ::core::ffi::c_int {
        cmd_find_clear_state(fs, flags);
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).set_wp((*(*fs).w_ptr()).active);
    cmd_find_log_state(
        b"cmd_find_from_window\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn cmd_find_from_winlink_pane(
    mut fs: *mut cmd_find_state,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
    mut flags: ::core::ffi::c_int,
) {
    cmd_find_clear_state(fs, flags);
    (*fs).s = (*wl).session.clone();
    (*fs).set_wl(wl);
    (*fs).idx = (*(*fs).wl_ptr()).idx;
    (*fs).set_w((*(*fs).wl_ptr()).window_ptr());
    (*fs).set_wp(wp);
    cmd_find_log_state(
        b"cmd_find_from_winlink_pane\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
}
pub unsafe fn cmd_find_from_pane(
    mut fs: *mut cmd_find_state,
    mut wp: *mut window_pane,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if cmd_find_from_window(fs, (*wp).window as *mut window, flags) != 0 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).set_wp(wp);
    cmd_find_log_state(
        b"cmd_find_from_pane\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn cmd_find_from_nothing(
    mut fs: *mut cmd_find_state,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    cmd_find_clear_state(fs, flags);
    let best_session_owner = cmd_find_best_session(None, flags);
    (*fs).s = best_session_owner.as_ref().map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
    if (*fs).s_ptr().is_null() {
        cmd_find_clear_state(fs, flags);
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).set_wl((*(*fs).s_ptr()).curw);
    (*fs).idx = (*(*fs).wl_ptr()).idx;
    (*fs).set_w((*(*fs).wl_ptr()).window_ptr());
    (*fs).set_wp((*(*fs).w_ptr()).active);
    cmd_find_log_state(
        b"cmd_find_from_nothing\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn cmd_find_from_mouse(
    mut fs: *mut cmd_find_state,
    mut m: *mut mouse_event,
) -> ::core::ffi::c_int {
    let mouse_pane_owner;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    cmd_find_clear_state(fs, flags);
    if (*m).valid == 0 {
        return -(1 as ::core::ffi::c_int);
    }
    {
        let mut s = std::ptr::null_mut();
        let mut wl = std::ptr::null_mut();
        mouse_pane_owner = cmd_mouse_pane(m, &raw mut s, &raw mut wl);
        let wp = mouse_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        (*fs).set_s(s);
        (*fs).set_wl(wl);
        (*fs).set_wp(wp);
    }
    if (*fs).wp_ptr().is_null() {
        cmd_find_clear_state(fs, flags);
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).set_w((*(*fs).wl_ptr()).window_ptr());
    cmd_find_log_state(
        b"cmd_find_from_mouse\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn cmd_find_from_client(
    mut fs: *mut cmd_find_state,
    mut c: *mut client,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let inside_pane_owner;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if c.is_null() {
        return cmd_find_from_nothing(fs, flags);
    }
    if !(*c).session.is_null() {
        cmd_find_clear_state(fs, flags);
        (*fs).set_wp((*(*(*(*c).session).curw).window_ptr()).active);
        if (*fs).wp_ptr().is_null() {
            cmd_find_from_session(fs, (*c).session, flags);
            return 0 as ::core::ffi::c_int;
        }
        (*fs).set_s((*c).session);
        (*fs).set_wl((*(*fs).s_ptr()).curw);
        (*fs).set_w((*(*fs).wl_ptr()).window_ptr());
        cmd_find_log_state(
            b"cmd_find_from_client\0" as *const u8 as *const ::core::ffi::c_char,
            fs,
        );
        return 0 as ::core::ffi::c_int;
    }
    cmd_find_clear_state(fs, flags);
    inside_pane_owner = cmd_find_inside_pane(c);
    wp = inside_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if !wp.is_null() {
        (*fs).set_w((*wp).window as *mut window);
        if !(cmd_find_best_session_with_window(fs) != 0 as ::core::ffi::c_int) {
            (*fs).set_wl((*(*fs).s_ptr()).curw);
            (*fs).set_w((*(*fs).wl_ptr()).window_ptr());
            (*fs).set_wp((*(*fs).w_ptr()).active);
            cmd_find_log_state(
                b"cmd_find_from_client\0" as *const u8 as *const ::core::ffi::c_char,
                fs,
            );
            return 0 as ::core::ffi::c_int;
        }
    }
    return cmd_find_from_nothing(fs, flags);
}
pub unsafe fn cmd_find_target(
    mut fs: *mut cmd_find_state,
    mut item: *mut cmdq_item,
    mut target: *const ::core::ffi::c_char,
    mut type_0: cmd_find_type,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mouse_pane_owner;
    let queue_client = cmdq_get_client(item);
    let queue_client_ptr = queue_client.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut current_block: u64;
    let mut m: *mut mouse_event = ::core::ptr::null_mut::<mouse_event>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut current: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    let mut colon: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut period: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut copy_owned = Vec::<u8>::new();
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
    log_debug(format_args!(
        "{}: target {}, type {}, item {}, flags {}",
        "cmd_find_target",
        log_cstr(
            (if target.is_null() {
                b"none\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                target
            }) as *const _
        ),
        log_cstr((s) as *const _),
        log_pointer((item) as *const ::core::ffi::c_void),
        log_cstr((&raw mut tmp as *mut ::core::ffi::c_char) as *const _)
    ));
    let mut queue_current = cmdq_get_state_owned(item).current_snapshot();
    let mut queue_event = cmdq_get_event(item);
    cmd_find_clear_state(fs, flags);
    if server_check_marked() != 0 && flags & CMD_FIND_DEFAULT_MARKED != 0 {
        (*fs).current = &raw mut marked_pane;
        log_debug(format_args!(
            "{}: current is marked pane",
            "cmd_find_target"
        ));
        current_block = 1836292691772056875;
    } else if cmd_find_valid_state(&queue_current) != 0 {
        (*fs).current = &mut queue_current;
        log_debug(format_args!("{}: current is from queue", "cmd_find_target"));
        current_block = 1836292691772056875;
    } else if cmd_find_from_client(&raw mut current, queue_client_ptr, flags)
        == 0 as ::core::ffi::c_int
    {
        (*fs).current = &raw mut current;
        log_debug(format_args!(
            "{}: current is from client",
            "cmd_find_target"
        ));
        current_block = 1836292691772056875;
    } else {
        if !flags & CMD_FIND_QUIET != 0 {
            cmdq_error(item, |out| out.write_all(b"no current target"));
        }
        current_block = 5193823237153215208;
    }
    match current_block {
        1836292691772056875 => {
            if cmd_find_valid_state(&*(*fs).current) == 0 {
                fatalx(|out| out.write_all(b"invalid current find state"));
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
                c = queue_client_ptr;
                if c.is_null() || (*c).session.is_null() {
                    cmdq_error(item, |out| out.write_all(b"no current client"));
                    current_block = 5193823237153215208;
                } else {
                    (*fs).set_wl((*(*c).session).curw);
                    (*fs).set_wp((*(*(*(*c).session).curw).window_ptr()).active);
                    (*fs).set_w((*(*(*c).session).curw).window_ptr());
                    current_block = 15319680530019787978;
                }
            } else if strcmp(target, b"=\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
                || strcmp(
                    target,
                    b"{mouse}\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
            {
                m = &raw mut queue_event.m;
                let mut current_block_56: u64;
                match type_0 as ::core::ffi::c_uint {
                    0 => {
                        {
                            let mut s = std::ptr::null_mut();
                            let mut wl = std::ptr::null_mut();
                            mouse_pane_owner = cmd_mouse_pane(m, &raw mut s, &raw mut wl);
                            let wp = mouse_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
                            (*fs).set_s(s);
                            (*fs).set_wl(wl);
                            (*fs).set_wp(wp);
                        }
                        if !(*fs).wp_ptr().is_null() {
                            (*fs).set_w((*(*fs).wl_ptr()).window_ptr());
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
                        {
                            let mut s = std::ptr::null_mut();
                            let wl = cmd_mouse_window(m, &raw mut s);
                            (*fs).set_s(s);
                            (*fs).set_wl(wl);
                        }
                        if (*fs).wl_ptr().is_null() && !(*fs).s_ptr().is_null() {
                            (*fs).set_wl((*(*fs).s_ptr()).curw);
                        }
                        if !(*fs).wl_ptr().is_null() {
                            (*fs).set_w((*(*fs).wl_ptr()).window_ptr());
                            (*fs).set_wp((*(*fs).w_ptr()).active);
                        }
                    }
                    _ => {}
                }
                if (*fs).wp_ptr().is_null() {
                    if !flags & CMD_FIND_QUIET != 0 {
                        cmdq_error(item, |out| out.write_all(b"no mouse target"));
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
                        cmdq_error(item, |out| out.write_all(b"no marked target"));
                    }
                    current_block = 5193823237153215208;
                } else {
                    cmd_find_copy_state(fs, &raw mut marked_pane);
                    current_block = 15319680530019787978;
                }
            } else {
                copy_owned = CStr::from_ptr(target).to_bytes_with_nul().to_vec();
                copy = copy_owned.as_mut_ptr().cast();
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
                    log_debug(format_args!(
                        "{}: target {} is {}{}{}{}{}{}",
                        "cmd_find_target",
                        log_cstr((target) as *const _),
                        log_cstr(
                            (if session.is_null() {
                                b"\0" as *const u8 as *const ::core::ffi::c_char
                            } else {
                                b"session \0" as *const u8 as *const ::core::ffi::c_char
                            }) as *const _
                        ),
                        log_cstr(
                            (if session.is_null() {
                                b"\0" as *const u8 as *const ::core::ffi::c_char
                            } else {
                                session
                            }) as *const _
                        ),
                        log_cstr(
                            (if window.is_null() {
                                b"\0" as *const u8 as *const ::core::ffi::c_char
                            } else {
                                b"window \0" as *const u8 as *const ::core::ffi::c_char
                            }) as *const _
                        ),
                        log_cstr(
                            (if window.is_null() {
                                b"\0" as *const u8 as *const ::core::ffi::c_char
                            } else {
                                window
                            }) as *const _
                        ),
                        log_cstr(
                            (if pane.is_null() {
                                b"\0" as *const u8 as *const ::core::ffi::c_char
                            } else {
                                b"pane \0" as *const u8 as *const ::core::ffi::c_char
                            }) as *const _
                        ),
                        log_cstr(
                            (if pane.is_null() {
                                b"\0" as *const u8 as *const ::core::ffi::c_char
                            } else {
                                pane
                            }) as *const _
                        )
                    ));
                }
                if !pane.is_null() && flags & CMD_FIND_WINDOW_INDEX != 0 {
                    if !flags & CMD_FIND_QUIET != 0 {
                        cmdq_error(item, |out| out.write_all(b"can't specify pane here"));
                    }
                    current_block = 5193823237153215208;
                } else {
                    if !session.is_null() {
                        if cmd_find_get_session(fs, session) != 0 as ::core::ffi::c_int {
                            if !flags & CMD_FIND_QUIET != 0 {
                                cmdq_error(item, |out| {
                                    out.write_all(b"can't find session: ")?;
                                    write_cstr(out, session)
                                });
                            }
                            current_block = 5193823237153215208;
                        } else if window.is_null() && pane.is_null() {
                            (*fs).set_wl((*(*fs).s_ptr()).curw);
                            (*fs).idx = -(1 as ::core::ffi::c_int);
                            (*fs).set_w((*(*fs).wl_ptr()).window_ptr());
                            (*fs).set_wp((*(*fs).w_ptr()).active);
                            current_block = 15319680530019787978;
                        } else if !window.is_null() && pane.is_null() {
                            if cmd_find_get_window_with_session(fs, window)
                                != 0 as ::core::ffi::c_int
                            {
                                current_block = 2743676411188200708;
                            } else {
                                if !(*fs).wl_ptr().is_null() {
                                    (*fs).set_wp((*(*(*fs).wl_ptr()).window_ptr()).active);
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
                            if !(*fs).wl_ptr().is_null() {
                                (*fs).set_wp((*(*(*fs).wl_ptr()).window_ptr()).active);
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
                                        cmdq_error(item, |out| {
                                            out.write_all(b"can't find window: ")?;
                                            write_cstr(out, window)
                                        });
                                    }
                                }
                                _ => {
                                    if !flags & CMD_FIND_QUIET != 0 {
                                        cmdq_error(item, |out| {
                                            out.write_all(b"can't find pane: ")?;
                                            write_cstr(out, pane)
                                        });
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
                    drop(copy_owned);
                    return 0 as ::core::ffi::c_int;
                }
            }
        }
        _ => {}
    }
    (*fs).current = ::core::ptr::null_mut::<cmd_find_state>();
    log_debug(format_args!("{}: error", "cmd_find_target"));
    drop(copy_owned);
    if flags & CMD_FIND_CANFAIL != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe fn cmd_find_current_client(
    mut item: *mut cmdq_item,
    mut quiet: ::core::ffi::c_int,
) -> Option<std::rc::Rc<std::cell::UnsafeCell<client>>> {
    let inside_pane_owner;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut found = None;
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    let mut c_owner = None;
    if !item.is_null() {
        c_owner = cmdq_get_client(item);
        c = c_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    }
    if !c.is_null() && !(*c).session.is_null() {
        return c_owner;
    }
    found = None;
    if !c.is_null() && {
        inside_pane_owner = cmd_find_inside_pane(c);
        wp = inside_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        !wp.is_null()
    } {
        cmd_find_clear_state(&raw mut fs, CMD_FIND_QUIET);
        fs.set_w((*wp).window as *mut window);
        if cmd_find_best_session_with_window(&raw mut fs) == 0 as ::core::ffi::c_int {
            if let Some(session_owner) = fs.s.upgrade() {
                found = cmd_find_best_client(&*session_owner.get());
            }
        }
    } else {
        if let Some(session_owner) = cmd_find_best_session(None, CMD_FIND_QUIET) {
            found = cmd_find_best_client(&*session_owner.get());
        }
    }
    if found.is_none() && !item.is_null() && quiet == 0 {
        cmdq_error(item, |out| out.write_all(b"no current client"));
    }
    log_debug(format_args!(
        "{}: no target, return {}",
        "cmd_find_current_client",
        log_pointer(found.as_ref().map_or(std::ptr::null(), |owner| owner.get().cast()))
    ));
    return found;
}
pub unsafe fn cmd_find_client(
    mut item: *mut cmdq_item,
    mut target: *const ::core::ffi::c_char,
    mut quiet: ::core::ffi::c_int,
) -> Option<std::rc::Rc<std::cell::UnsafeCell<client>>> {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if target.is_null() {
        return cmd_find_current_client(item, quiet);
    }
    let target_bytes = CStr::from_ptr(target).to_bytes();
    let trimmed = target_bytes.strip_suffix(b":").unwrap_or(target_bytes);
    let copy = CString::new(trimmed).expect("client target came from a C string");
    let copy_ptr = copy.as_ptr();
    let mut registry_c_owner = clients.first();
    c = registry_c_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if !(*c).session.is_null() {
            if strcmp(
                copy_ptr,
                ((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            ) == 0 as ::core::ffi::c_int
            {
                break;
            }
            if !(*(*c).ttyname.as_ref().unwrap().as_ptr() as ::core::ffi::c_int == '\0' as i32) {
                if strcmp(
                    copy_ptr,
                    ((*c).ttyname)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                ) == 0 as ::core::ffi::c_int
                {
                    break;
                }
                if !(strncmp(
                    ((*c).ttyname)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                    _PATH_DEV.as_ptr(),
                    (::core::mem::size_of::<[::core::ffi::c_char; 6]>() as size_t)
                        .wrapping_sub(1 as size_t),
                ) != 0 as ::core::ffi::c_int)
                {
                    if strcmp(
                        copy_ptr,
                        (*c).ttyname
                            .as_ref()
                            .unwrap()
                            .as_ptr()
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
        registry_c_owner = clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    }
    if c.is_null() && quiet == 0 {
        cmdq_error(item, |out| {
            out.write_all(b"can't find client: ")?;
            write_cstr(out, copy_ptr)
        });
    }
    drop(copy);
    log_debug(format_args!(
        "{}: target {}, return {}",
        "cmd_find_client",
        log_cstr((target) as *const _),
        log_pointer((c) as *const ::core::ffi::c_void)
    ));
    return registry_c_owner;
}

#[cfg(test)]
mod target_observer_tests {
    use super::*;
    use crate::src::shared::rc;
    use crate::src::window::{winlink_add, winlink_remove, winlink_set_window, winlinks_reindex};
    use std::rc::Rc;

    #[test]
    fn best_client_selection_preserves_filtering_ties_and_retains_result() {
        unsafe {
            let saved = std::mem::replace(
                &mut *std::ptr::addr_of_mut!(clients),
                crate::src::server_client::ClientRegistry::new(),
            );
            let session_owner = session::new();
            let other_session = session::new();
            let first = client::new();
            let second = client::new();
            (*first.get()).session = session_owner.get();
            (*second.get()).session = other_session.get();
            (*first.get()).activity_time.tv_sec = 10;
            (*second.get()).activity_time.tv_sec = 20;
            clients.push_back(first.clone());
            clients.push_back(second.clone());
            (*session_owner.get()).attached = 1;
            assert!(Rc::ptr_eq(&cmd_find_best_client(&*session_owner.get()).unwrap(), &first));
            (*session_owner.get()).attached = 0;
            assert!(Rc::ptr_eq(&cmd_find_best_client(&*session_owner.get()).unwrap(), &second));
            (*second.get()).activity_time.tv_sec = 10;
            assert!(Rc::ptr_eq(&cmd_find_best_client(&*session_owner.get()).unwrap(), &first));
            (*second.get()).activity_time.tv_usec = 1;
            let selected = cmd_find_best_client(&*session_owner.get()).unwrap();
            let observer = Rc::downgrade(&second);
            let registry = std::mem::replace(&mut *std::ptr::addr_of_mut!(clients), saved);
            drop(registry);
            (*first.get()).session = std::ptr::null_mut();
            (*second.get()).session = std::ptr::null_mut();
            drop(first);
            drop(second);
            assert!(observer.upgrade().is_some());
            drop(selected);
            assert!(observer.upgrade().is_none());
        }
    }

    #[test]
    fn session_ranking_preserves_attachment_preference_recency_and_ties() {
        let mut first = session::empty();
        let mut second = session::empty();
        first.activity_time.tv_sec = 10;
        second.activity_time.tv_sec = 9;
        assert!(cmd_find_session_better(&first, None, 0));
        assert!(cmd_find_session_better(&first, Some(&second), 0));
        first.attached = 1;
        assert!(!cmd_find_session_better(&first, Some(&second), CMD_FIND_PREFER_UNATTACHED));
        assert!(cmd_find_session_better(&second, Some(&first), CMD_FIND_PREFER_UNATTACHED));
        second.activity_time.tv_sec = 10;
        assert!(!cmd_find_session_better(&first, Some(&second), 0));
        first.activity_time.tv_usec = 1;
        assert!(cmd_find_session_better(&first, Some(&second), 0));
    }

    #[test]
    fn expired_targets_keep_identity_until_explicitly_cleared() {
        unsafe {
            let session = session::new();
            let window = window::new();
            let pane = window_pane::new();
            let mut links = crate::src::shared::window::winlinks { storage: None };
            let link = winlink_add(&mut links, 7);
            let mut state = cmd_find_state::default();
            state.set_s(rc::as_ptr(&session));
            state.set_w(rc::as_ptr(&window));
            state.set_wp(rc::as_ptr(&pane));
            state.set_wl(link);
            assert_eq!(Rc::strong_count(&session), 1);
            assert_eq!(Rc::strong_count(&window), 1);
            assert_eq!(Rc::strong_count(&pane), 1);
            let snapshot = state.clone();
            drop(session);
            drop(window);
            drop(pane);
            winlink_remove(&mut links, link);
            assert!(state.s_ptr().is_null());
            assert!(state.w_ptr().is_null());
            assert!(state.wp_ptr().is_null());
            assert!(state.wl_ptr().is_null());
            assert_eq!(cmd_find_valid_state(&state), 0);
            assert_eq!(cmd_find_empty_state(&state), 0);
            assert!(state.s.ptr_eq(&snapshot.s));
            assert!(state.w.ptr_eq(&snapshot.w));
            assert!(state.wp.ptr_eq(&snapshot.wp));
            assert!(state.wl == snapshot.wl);
            cmd_find_clear_state(&mut state, CMD_FIND_QUIET);
            assert_eq!(state.flags, CMD_FIND_QUIET);
            assert_eq!(state.idx, -1);
            assert!(state.current.is_null());
            assert_eq!(cmd_find_empty_state(&state), 1);
            assert_eq!(cmd_find_empty_state(&snapshot), 0);
        }
    }

    #[test]
    fn copying_releases_old_handles_and_preserves_search_context() {
        unsafe {
            let session = session::new();
            let other = session::new();
            let mut source = cmd_find_state {
                flags: 11,
                idx: 17,
                ..Default::default()
            };
            source.set_s(rc::as_ptr(&session));
            let mut destination = cmd_find_state {
                flags: 22,
                ..Default::default()
            };
            destination.current = &raw mut source;
            destination.set_s(rc::as_ptr(&other));
            let old_count = Rc::weak_count(&other);
            let new_count = Rc::weak_count(&session);
            cmd_find_copy_state(&mut destination, &mut source);
            assert_eq!(Rc::weak_count(&other), old_count - 1);
            assert_eq!(Rc::weak_count(&session), new_count + 1);
            assert_eq!(destination.flags, 22);
            assert_eq!(destination.current, &raw mut source);
            assert_eq!(destination.idx, 17);
            let ptr = &raw mut destination;
            cmd_find_copy_state(ptr, ptr);
            assert_eq!(Rc::weak_count(&session), new_count + 1);
            cmd_find_clear_state(&mut destination, 0);
            assert_eq!(Rc::weak_count(&session), new_count);
        }
    }

    #[test]
    fn validation_checks_membership_and_rejects_replacement_links() {
        unsafe {
            let saved = std::mem::replace(
                &mut *std::ptr::addr_of_mut!(sessions),
                crate::src::shared::session::sessions { storage: None },
            );
            let session_owner = session::new();
            let window_owner = window::new();
            let pane_owner = window_pane::new();
            assert_eq!(session_alive(None), 0);
            assert_eq!(session_alive(Some(&*session_owner.get())), 0);
            let s = rc::as_ptr(&session_owner);
            let w = rc::as_ptr(&window_owner);
            let wp = rc::as_ptr(&pane_owner);
            crate::src::session::sessions_insert(&raw mut sessions, session_owner.clone());
            assert_eq!(session_alive(Some(&*session_owner.get())), 1);
            let links = &raw mut (*s).windows;
            let wl = winlink_add(links, 1);
            (*wl).session = (*s).observer.clone();
            winlink_set_window(wl, w);
            (*wp).window = w;
            (*w).panes.push_back(Rc::downgrade(&pane_owner));
            (*w).active = wp;
            (*s).curw = wl;
            assert!(cmd_find_best_session(Some(&[]), 0).is_none());
            let candidates = vec![session_owner.clone()];
            let selected = cmd_find_best_session(Some(&candidates), 0).unwrap();
            assert!(Rc::ptr_eq(&selected, &session_owner));
            drop(candidates);
            assert!(Rc::ptr_eq(&selected, &session_owner));
            drop(selected);
            (*s).curw = std::ptr::null_mut();
            let mut state = cmd_find_state::default();
            cmd_find_from_winlink_pane(&mut state, wl, wp, 0);
            assert_eq!(cmd_find_valid_state(&state), 1);
            winlinks_reindex(links, wl, 3);
            assert_eq!(cmd_find_valid_state(&state), 1);

            let panes = (*w).panes.storage.take();
            assert_eq!(cmd_find_valid_state(&state), 0);
            (*w).panes.storage = panes;
            drop(crate::src::session::sessions_remove(&raw mut sessions, s));
            assert!(state.s.upgrade().is_some());
            assert_eq!(session_alive(Some(&*session_owner.get())), 0);
            assert_eq!(cmd_find_valid_state(&state), 0);
            crate::src::session::sessions_insert(&raw mut sessions, session_owner.clone());
            assert_eq!(cmd_find_valid_state(&state), 1);

            winlink_remove(links, wl);
            let replacement = winlink_add(links, 3);
            (*replacement).session = (*s).observer.clone();
            winlink_set_window(replacement, w);
            assert_eq!(cmd_find_valid_state(&state), 0);
            state.set_wl(replacement);
            assert_eq!(cmd_find_valid_state(&state), 1);
            winlink_remove(links, replacement);
            (*w).panes.storage = None;
            (*wp).window = std::ptr::null_mut();
            drop(crate::src::session::sessions_remove(&raw mut sessions, s));
            *std::ptr::addr_of_mut!(sessions) = saved;
        }
    }
}
