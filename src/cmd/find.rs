use crate::src::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_event, cmdq_get_state_owned};
use crate::src::cmd::{cmd_mouse_pane, cmd_mouse_window};
use crate::src::compat::strtonum::strtonum;
use crate::src::environ::environ_find;
use crate::src::ffi::libc::{fnmatch, strchr, strcmp, strlcat, strlen, strncmp};
use crate::src::format::bytes::write_cstr;
use crate::src::log::{fatalx, log_cstr, log_debug, log_pointer};
use crate::src::server::clients;
use crate::src::server::{marked_pane, server_check_marked};
use crate::src::server_client::Client as _;
use crate::src::server_client::Client;
use crate::src::session::sessions;
use crate::src::session::Session;
use crate::src::session::{session_find, session_find_by_id_str, sessions_minmax};
use crate::src::shared::client::ClientRef;
use crate::src::shared::rc::same;
use crate::src::shared::session::SessionRef;
use crate::src::shared::window::WindowRef;
use crate::src::window::Window as _;
use crate::src::window::Window;
use crate::src::window::{
    all_window_panes, window_find_by_id_str, window_find_string, window_pane_find_by_id_str,
    window_pane_find_down, window_pane_find_left, window_pane_find_right, window_pane_find_up,
    window_pane_tree_minmax, window_pane_tree_next, winlink_find_by_index, winlink_next_by_number,
    winlink_previous_by_number, winlinks_minmax, winlinks_next,
};
use crate::src::window_pane::WindowPane as _;
use std::ffi::{CStr, CString};
use std::time::{Duration, UNIX_EPOCH};
use std::{cell::UnsafeCell, rc::Rc};

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
const cmd_find_window_table: &[(&CStr, &CStr)] = &[
    (c"{start}", c"^"),
    (c"{last}", c"!"),
    (c"{end}", c"$"),
    (c"{next}", c"+"),
    (c"{previous}", c"-"),
];
const cmd_find_pane_table: &[(&CStr, &CStr)] = &[
    (c"{last}", c"!"),
    (c"{next}", c"+"),
    (c"{previous}", c"-"),
    (c"{top}", c"top"),
    (c"{bottom}", c"bottom"),
    (c"{left}", c"left"),
    (c"{right}", c"right"),
    (c"{top-left}", c"top-left"),
    (c"{top-right}", c"top-right"),
    (c"{bottom-left}", c"bottom-left"),
    (c"{bottom-right}", c"bottom-right"),
    (c"{up-of}", c"{up-of}"),
    (c"{down-of}", c"{down-of}"),
    (c"{left-of}", c"{left-of}"),
    (c"{right-of}", c"{right-of}"),
];
unsafe fn cmd_find_inside_pane(
    c_owner: Option<&ClientRef>,
) -> Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
    let c = c_owner?;
    let tty_name = c.tty_name();
    let mut inside_pane_owner = None;
    let mut wp: *mut window_pane = std::ptr::null_mut();
    let mut indexed_pane_owner = window_pane_tree_minmax(&all_window_panes);
    wp = indexed_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !wp.is_null() {
        if (*wp).fd != -(1 as ::core::ffi::c_int)
            && strcmp(
                &raw mut (*wp).tty as *mut ::core::ffi::c_char,
                tty_name
                    .as_ref()
                    .map_or(std::ptr::null(), |name| name.as_ptr()),
            ) == 0 as ::core::ffi::c_int
        {
            break;
        }
        indexed_pane_owner = window_pane_tree_next(&*wp);
        wp = indexed_pane_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
    if wp.is_null() {
        let pane_id = c.with_environment(|environment| {
            environ_find(environment.expect("environment"), c"TMUX_PANE".as_ptr())
                .and_then(|entry| entry.value.clone())
        });
        inside_pane_owner = pane_id
            .as_deref()
            .and_then(|value| window_pane_find_by_id_str(value));
        wp = inside_pane_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
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
unsafe fn cmd_find_client_better(c: &ClientRef, than: Option<&ClientRef>) -> bool {
    than.is_none_or(|than| {
        let (c, than) = (c.activity_time(), than.activity_time());
        c > than
    })
}
pub unsafe fn cmd_find_best_client(s: &SessionRef) -> Option<ClientRef> {
    let mut best: Option<ClientRef> = None;
    let mut cursor = clients.first();
    while let Some(owner) = cursor {
        cursor = clients.next(&owner);
        let Some(attached) = owner.attached_session().upgrade() else {
            continue;
        };
        if s.is_attached() && !Rc::ptr_eq(&attached, s) {
            continue;
        }
        if cmd_find_client_better(&owner, best.as_ref()) {
            best = Some(owner);
        }
    }
    best
}
unsafe fn cmd_find_session_better(s: &SessionRef, than: Option<&SessionRef>, flags: i32) -> bool {
    let Some(than) = than else { return true };
    if flags & CMD_FIND_PREFER_UNATTACHED != 0 {
        if than.is_attached() && !s.is_attached() {
            return true;
        }
        if !than.is_attached() && s.is_attached() {
            return false;
        }
    }
    let (s, than) = (s.activity_time(), than.activity_time());
    s > than
}
unsafe fn cmd_find_session_valid(s: &SessionRef) -> i32 {
    if !s.is_registered() {
        return 0;
    }
    let link = s.current_winlink();
    if !link.is_alive() {
        return 0;
    }
    link.get_unchecked()
        .window_handle()
        .is_some_and(|window| window.active_pane().is_some()) as i32
}
unsafe fn cmd_find_best_session(
    candidates: Option<&[SessionRef]>,
    flags: ::core::ffi::c_int,
) -> Option<SessionRef> {
    let mut all = Vec::new();
    let candidates = match candidates {
        Some(candidates) => candidates,
        None => {
            let mut cursor = sessions_minmax(&sessions);
            while let Some(owner) = cursor {
                cursor = owner.next_session();
                all.push(owner);
            }
            &all
        }
    };
    log_debug(format_args!(
        "cmd_find_best_session: {} sessions to try",
        candidates.len()
    ));
    let mut best: Option<SessionRef> = None;
    for owner in candidates {
        if cmd_find_session_valid(owner) != 0
            && cmd_find_session_better(owner, best.as_ref(), flags)
        {
            best = Some(owner.clone());
        }
    }
    best
}
unsafe fn cmd_find_best_session_with_window(fs: *mut cmd_find_state) -> ::core::ffi::c_int {
    let Some(window_owner) = (*fs).w.upgrade() else {
        return -1;
    };
    let mut candidates = Vec::new();
    let mut cursor = sessions_minmax(&sessions);
    while let Some(owner) = cursor {
        cursor = owner.next_session();
        if owner.contains_window(&window_owner) {
            candidates.push(owner);
        }
    }
    let Some(best) = cmd_find_best_session(Some(&candidates), (*fs).flags) else {
        return -1;
    };
    (*fs).s = std::rc::Rc::downgrade(&best);
    cmd_find_best_winlink_with_window(fs)
}
unsafe fn cmd_find_best_winlink_with_window(fs: *mut cmd_find_state) -> ::core::ffi::c_int {
    let session = (*fs).session_handle().expect("window target session");
    let window = (*fs).window_handle().expect("target window");
    log_debug(format_args!(
        "{}: window is @{}",
        "cmd_find_best_winlink_with_window",
        window.id()
    ));
    let current = session.current_winlink();
    let mut selected = refbox::Weak::new();
    if current.is_alive() && same(current.get_unchecked().window_handle(), Some(&window)) {
        selected = current;
    } else {
        let mut candidate = session.with_winlinks(|links| winlinks_minmax(links, RB_NEGINF));
        while candidate.is_alive() {
            if same(candidate.get_unchecked().window_handle(), Some(&window)) {
                selected = candidate;
                break;
            }
            candidate = winlinks_next(candidate.get_unchecked());
        }
    }
    window.release(c"find window link");
    if !selected.is_alive() {
        return -1;
    }
    (*fs).set_wl(selected);
    (*fs).idx = (*fs).winlink_handle().get_unchecked().idx;
    0
}

fn cmd_find_map_table<'a>(table: &[(&'static CStr, &'static CStr)], target: &'a CStr) -> &'a CStr {
    table
        .iter()
        .find_map(|(source, mapped)| (*source == target).then_some(*mapped))
        .unwrap_or(target)
}

unsafe fn cmd_find_get_session(fs: *mut cmd_find_state, target: *const ::core::ffi::c_char) -> i32 {
    let target = CStr::from_ptr(target);
    log_debug(format_args!(
        "cmd_find_get_session: {}",
        log_cstr(target.as_ptr())
    ));
    if target.to_bytes().first() == Some(&b'$') {
        (*fs).set_s(session_find_by_id_str(target).as_ref());
        return if (*fs).session_handle().is_some() {
            0
        } else {
            -1
        };
    }
    (*fs).set_s(session_find(target).as_ref());
    if (*fs).session_handle().is_some() {
        return 0;
    }
    if let Some(client) = cmd_find_client(None, target.as_ptr(), 1) {
        if let Some(session) = client.attached_session().upgrade() {
            (*fs).set_s(Some(&session));
            return 0;
        }
    }
    if (*fs).flags & CMD_FIND_EXACT_SESSION != 0 {
        return -1;
    }
    // Prefix matching precedes glob matching; either must be unambiguous.
    for glob in [false, true] {
        let mut matched = None;
        let mut cursor = sessions_minmax(&sessions);
        while let Some(session) = cursor {
            let name = session.name();
            if if glob {
                fnmatch(target.as_ptr(), name.as_ptr(), 0) == 0
            } else {
                name.as_bytes().starts_with(target.to_bytes())
            } {
                if matched.is_some() {
                    return -1;
                }
                matched = Some(session.clone());
            }
            cursor = session.next_session();
        }
        if let Some(session) = matched {
            (*fs).set_s(Some(&session));
            return 0;
        }
    }
    -1
}
unsafe fn cmd_find_get_window(
    mut fs: *mut cmd_find_state,
    mut window: *const ::core::ffi::c_char,
    mut only: ::core::ffi::c_int,
    current: &cmd_find_state,
) -> ::core::ffi::c_int {
    log_debug(format_args!(
        "{}: {}",
        "cmd_find_get_window",
        log_cstr((window) as *const _)
    ));
    if *window as ::core::ffi::c_int == '@' as i32 {
        let window_owner = window_find_by_id_str(window);
        let result = (|| {
            (*fs).set_w(window_owner.as_ref());
            if (*fs).window_handle().is_none() {
                return -(1 as ::core::ffi::c_int);
            }
            return cmd_find_best_session_with_window(fs);
        })();
        if let Some(window) = window_owner {
            crate::src::window::window_remove_ref(window, c"cmd_find_get_window".as_ptr());
        }
        return result;
    }
    (*fs).s = current.s.clone();
    if cmd_find_get_window_with_session(fs, window) == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if only == 0 && cmd_find_get_session(fs, window) == 0 as ::core::ffi::c_int {
        (*fs).set_wl(
            ((*fs)
                .session_handle()
                .expect("live session")
                .current_winlink())
            .clone(),
        );
        (*fs).set_w(((*fs).winlink_handle()).get_unchecked().window_handle());
        if !(*fs).flags & CMD_FIND_WINDOW_INDEX != 0 {
            (*fs).idx = ((*fs).winlink_handle()).get_unchecked().idx;
        }
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe fn cmd_find_get_window_with_session(
    mut fs: *mut cmd_find_state,
    mut window: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut exact: ::core::ffi::c_int = 0;
    log_debug(format_args!(
        "{}: {}",
        "cmd_find_get_window_with_session",
        log_cstr((window) as *const _)
    ));
    exact = (*fs).flags & CMD_FIND_EXACT_WINDOW;
    (*fs).set_wl(
        ((*fs)
            .session_handle()
            .expect("live session")
            .current_winlink())
        .clone(),
    );
    (*fs).set_w(((*fs).winlink_handle()).get_unchecked().window_handle());
    if *window as ::core::ffi::c_int == '@' as i32 {
        let window_owner = window_find_by_id_str(window);
        let result = (|| {
            (*fs).set_w(window_owner.as_ref());
            if (*fs).window_handle().is_none()
                || !(*fs)
                    .session_handle()
                    .expect("target session")
                    .contains_window((*fs).window_handle().as_ref().unwrap())
            {
                return -(1 as ::core::ffi::c_int);
            }
            return cmd_find_best_winlink_with_window(fs);
        })();
        if let Some(window) = window_owner {
            crate::src::window::window_remove_ref(
                window,
                c"cmd_find_get_window_with_session".as_ptr(),
            );
        }
        return result;
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
        let s = (*fs).session_handle().expect("target session");
        if (*fs).flags & CMD_FIND_WINDOW_INDEX != 0 {
            if *window.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '+' as i32
            {
                if INT_MAX - (s.current_winlink()).get_unchecked().idx < n {
                    return -(1 as ::core::ffi::c_int);
                }
                (*fs).idx = (s.current_winlink()).get_unchecked().idx + n;
            } else {
                if n > (s.current_winlink()).get_unchecked().idx {
                    return -(1 as ::core::ffi::c_int);
                }
                (*fs).idx = (s.current_winlink()).get_unchecked().idx - n;
            }
            return 0 as ::core::ffi::c_int;
        }
        if *window.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '+' as i32 {
            (*fs).set_wl((winlink_next_by_number((s.current_winlink()).clone(), &s, n)).clone());
        } else {
            (*fs)
                .set_wl((winlink_previous_by_number((s.current_winlink()).clone(), &s, n)).clone());
        }
        if (*fs).winlink_handle().is_alive() {
            (*fs).idx = ((*fs).winlink_handle()).get_unchecked().idx;
            (*fs).set_w(((*fs).winlink_handle()).get_unchecked().window_handle());
            return 0 as ::core::ffi::c_int;
        }
    }
    if exact == 0 {
        if strcmp(window, b"!\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            let session = (*fs).session_handle().expect("window target session");
            (*fs).set_wl(crate::src::session::Session::last_winlink(&session));
            if !(*fs).winlink_handle().is_alive() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).idx = ((*fs).winlink_handle()).get_unchecked().idx;
            (*fs).set_w(((*fs).winlink_handle()).get_unchecked().window_handle());
            return 0 as ::core::ffi::c_int;
        } else if strcmp(window, b"^\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            (*fs).set_wl(
                ((*fs)
                    .session_handle()
                    .expect("window target session")
                    .with_winlinks(|links| winlinks_minmax(links, RB_NEGINF)))
                .clone(),
            );
            if !(*fs).winlink_handle().is_alive() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).idx = ((*fs).winlink_handle()).get_unchecked().idx;
            (*fs).set_w(((*fs).winlink_handle()).get_unchecked().window_handle());
            return 0 as ::core::ffi::c_int;
        } else if strcmp(window, b"$\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            (*fs).set_wl(
                ((*fs)
                    .session_handle()
                    .expect("window target session")
                    .with_winlinks(|links| winlinks_minmax(links, RB_INF)))
                .clone(),
            );
            if !(*fs).winlink_handle().is_alive() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).idx = ((*fs).winlink_handle()).get_unchecked().idx;
            (*fs).set_w(((*fs).winlink_handle()).get_unchecked().window_handle());
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
            (*fs).set_wl(
                ((*fs)
                    .session_handle()
                    .expect("window target session")
                    .with_winlinks(|links| winlink_find_by_index(links, idx)))
                .clone(),
            );
            if (*fs).winlink_handle().is_alive() {
                (*fs).idx = ((*fs).winlink_handle()).get_unchecked().idx;
                (*fs).set_w(((*fs).winlink_handle()).get_unchecked().window_handle());
                return 0 as ::core::ffi::c_int;
            }
            if (*fs).flags & CMD_FIND_WINDOW_INDEX != 0 {
                (*fs).idx = idx;
                return 0 as ::core::ffi::c_int;
            }
        }
    }
    (*fs).set_wl((refbox::Weak::new()).clone());
    wl = (*fs)
        .session_handle()
        .expect("window target session")
        .with_winlinks(|links| winlinks_minmax(links, RB_NEGINF));
    while wl.is_alive() {
        if strcmp(
            window,
            wl.get_unchecked()
                .window_handle()
                .expect("linked window")
                .name()
                .as_ptr(),
        ) == 0 as ::core::ffi::c_int
        {
            if (*fs).winlink_handle().is_alive() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).set_wl(wl.clone());
        }
        wl = winlinks_next(wl.get_unchecked());
    }
    if (*fs).winlink_handle().is_alive() {
        (*fs).idx = ((*fs).winlink_handle()).get_unchecked().idx;
        (*fs).set_w(((*fs).winlink_handle()).get_unchecked().window_handle());
        return 0 as ::core::ffi::c_int;
    }
    if exact != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).set_wl((refbox::Weak::new()).clone());
    wl = (*fs)
        .session_handle()
        .expect("window target session")
        .with_winlinks(|links| winlinks_minmax(links, RB_NEGINF));
    while wl.is_alive() {
        if strncmp(
            window,
            wl.get_unchecked()
                .window_handle()
                .expect("linked window")
                .name()
                .as_ptr(),
            strlen(window),
        ) == 0 as ::core::ffi::c_int
        {
            if (*fs).winlink_handle().is_alive() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).set_wl(wl.clone());
        }
        wl = winlinks_next(wl.get_unchecked());
    }
    if (*fs).winlink_handle().is_alive() {
        (*fs).idx = ((*fs).winlink_handle()).get_unchecked().idx;
        (*fs).set_w(((*fs).winlink_handle()).get_unchecked().window_handle());
        return 0 as ::core::ffi::c_int;
    }
    (*fs).set_wl((refbox::Weak::new()).clone());
    wl = (*fs)
        .session_handle()
        .expect("window target session")
        .with_winlinks(|links| winlinks_minmax(links, RB_NEGINF));
    while wl.is_alive() {
        if fnmatch(
            window,
            wl.get_unchecked()
                .window_handle()
                .expect("linked window")
                .name()
                .as_ptr(),
            0 as ::core::ffi::c_int,
        ) == 0 as ::core::ffi::c_int
        {
            if (*fs).winlink_handle().is_alive() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).set_wl(wl.clone());
        }
        wl = winlinks_next(wl.get_unchecked());
    }
    if (*fs).winlink_handle().is_alive() {
        (*fs).idx = ((*fs).winlink_handle()).get_unchecked().idx;
        (*fs).set_w(((*fs).winlink_handle()).get_unchecked().window_handle());
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe fn cmd_find_get_pane(
    mut fs: *mut cmd_find_state,
    mut pane: *const ::core::ffi::c_char,
    mut only: ::core::ffi::c_int,
    current: &cmd_find_state,
) -> ::core::ffi::c_int {
    log_debug(format_args!(
        "{}: {}",
        "cmd_find_get_pane",
        log_cstr((pane) as *const _)
    ));
    if *pane as ::core::ffi::c_int == '%' as i32 {
        let pane_owner = window_pane_find_by_id_str(CStr::from_ptr(pane));
        (*fs).wp = pane_owner
            .as_ref()
            .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
        if (*fs).pane_handle().is_none() {
            return -(1 as ::core::ffi::c_int);
        }
        (*fs).set_w(
            (*(*fs)
                .pane_handle()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get()))
            .window_handle()
            .as_ref(),
        );
        return cmd_find_best_session_with_window(fs);
    }
    (*fs).s = current.s.clone();
    (*fs).wl = current.wl.clone();
    (*fs).idx = current.idx;
    (*fs).w = current.w.clone();
    if cmd_find_get_pane_with_window(fs, pane) == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if only == 0
        && cmd_find_get_window(fs, pane, 0 as ::core::ffi::c_int, current)
            == 0 as ::core::ffi::c_int
    {
        (*fs).set_wp(
            ((((*fs).window_handle().as_ref()).expect("live window"))
                .active_pane()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get()))
            .as_ref(),
        );
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
        (*fs).wp = pane_owner
            .as_ref()
            .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
        if (*fs).pane_handle().is_none() {
            return -(1 as ::core::ffi::c_int);
        }
        (*fs).set_w(
            (*(*fs)
                .pane_handle()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get()))
            .window_handle()
            .as_ref(),
        );
        return cmd_find_best_winlink_with_window(fs);
    }
    (*fs).set_wl(
        ((*fs)
            .session_handle()
            .expect("live session")
            .current_winlink())
        .clone(),
    );
    (*fs).idx = ((*fs).winlink_handle()).get_unchecked().idx;
    (*fs).set_w(((*fs).winlink_handle()).get_unchecked().window_handle());
    return cmd_find_get_pane_with_window(fs, pane);
}
unsafe fn cmd_find_get_pane_with_window(
    mut fs: *mut cmd_find_state,
    mut pane: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_int = 0;
    let _wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut n: u_int = 0;
    log_debug(format_args!(
        "{}: {}",
        "cmd_find_get_pane_with_window",
        log_cstr((pane) as *const _)
    ));
    if *pane as ::core::ffi::c_int == '%' as i32 {
        let pane_owner = window_pane_find_by_id_str(CStr::from_ptr(pane));
        (*fs).wp = pane_owner
            .as_ref()
            .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
        let Some(pane_owner) = pane_owner else {
            return -1;
        };
        if !pane_owner.window_observer().ptr_eq(&(*fs).w) {
            return -1;
        }
        return 0 as ::core::ffi::c_int;
    }
    if strcmp(pane, b"!\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        let window = (*fs).window_handle().expect("target window");
        (*fs).wp = window
            .last_active_pane()
            .as_ref()
            .map_or_else(std::rc::Weak::new, Rc::downgrade);
        window.release(c"find last active pane");
        if (*fs).pane_handle().is_none() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    } else if strcmp(
        pane,
        b"{up-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        let window_owner = (*fs).w.upgrade().expect("target window");
        let active = window_owner.active_pane();
        let selected = window_pane_find_up(active.as_ref());
        (*fs).wp = selected
            .as_ref()
            .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
        if (*fs).pane_handle().is_none() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    } else if strcmp(
        pane,
        b"{down-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        let window_owner = (*fs).w.upgrade().expect("target window");
        let active = window_owner.active_pane();
        let selected = window_pane_find_down(active.as_ref());
        (*fs).wp = selected
            .as_ref()
            .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
        if (*fs).pane_handle().is_none() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    } else if strcmp(
        pane,
        b"{left-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        let window_owner = (*fs).w.upgrade().expect("target window");
        let active = window_owner.active_pane();
        let selected = window_pane_find_left(active.as_ref());
        (*fs).wp = selected
            .as_ref()
            .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
        if (*fs).pane_handle().is_none() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    } else if strcmp(
        pane,
        b"{right-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        let window_owner = (*fs).w.upgrade().expect("target window");
        let active = window_owner.active_pane();
        let selected = window_pane_find_right(active.as_ref());
        (*fs).wp = selected
            .as_ref()
            .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
        if (*fs).pane_handle().is_none() {
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
        let active_owner = window_owner.active_pane();
        let selected = if *pane == b'+' as std::ffi::c_char {
            window_owner.pane_by_number(active_owner.as_ref(), n, false)
        } else {
            window_owner.pane_by_number(active_owner.as_ref(), n, true)
        };
        (*fs).wp = selected
            .as_ref()
            .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
        if !(*fs).pane_handle().is_none() {
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
        let selected = window_owner.pane_at_index(idx as u_int);
        (*fs).wp = selected
            .as_ref()
            .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
        if !(*fs).pane_handle().is_none() {
            return 0 as ::core::ffi::c_int;
        }
    }
    let window_owner = (*fs).w.upgrade().expect("target window");
    let selected = window_find_string(&window_owner, CStr::from_ptr(pane));
    (*fs).wp = selected
        .as_ref()
        .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
    if !(*fs).pane_handle().is_none() {
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
    let (Some(s), Some(w), Some(_wp)) = (fs.s.upgrade(), fs.w.upgrade(), fs.wp.upgrade()) else {
        return 0;
    };
    let Ok(wl) = fs.wl.try_borrow_mut() else {
        return 0;
    };
    if crate::src::session::sessions_resolve(&sessions, &fs.s).is_none()
        || !wl
            .window_handle()
            .is_some_and(|window| std::rc::Rc::ptr_eq(window, &w))
    {
        return 0;
    }
    // Identity matters: a replacement at the same index is a different target.
    let member = s.with_winlinks(|links| winlink_find_by_index(links, wl.idx));
    if member != fs.wl {
        return 0;
    }
    let mut cursor = w.next_pane(None);
    while let Some(pane) = cursor {
        if std::rc::Rc::downgrade(&pane).ptr_eq(&fs.wp) {
            return 1;
        }
        cursor = w.next_pane(Some(&pane));
    }
    0
}
pub unsafe fn cmd_find_copy_state(mut dst: *mut cmd_find_state, src: *const cmd_find_state) {
    let source = (*src).clone();
    (*dst).s = source.s;
    (*dst).wl = source.wl;
    (*dst).idx = source.idx;
    (*dst).w = source.w;
    (*dst).wp = source.wp;
}
unsafe fn cmd_find_log_state(mut prefix: *const ::core::ffi::c_char, mut fs: *mut cmd_find_state) {
    if !(*fs).session_handle().is_none() {
        log_debug(format_args!(
            "{}: s=${} {}",
            log_cstr((prefix) as *const _),
            ((*fs).session_handle().expect("live session").id()) as u32,
            log_cstr(
                (((*fs).session_handle().expect("live session").name())
                    .as_ptr()
                    .cast_mut()) as *const _
            )
        ));
    } else {
        log_debug(format_args!("{}: s=none", log_cstr((prefix) as *const _)));
    }
    if (*fs).winlink_handle().is_alive() {
        log_debug(format_args!(
            "{}: wl={} {} w=@{} {}",
            log_cstr((prefix) as *const _),
            (*fs).winlink_handle().get_unchecked().idx as u32,
            same(
                (*fs).winlink_handle().get_unchecked().window_handle(),
                (*fs).window_handle().as_ref(),
            ) as ::core::ffi::c_int,
            ((((*fs).window_handle().as_ref()).expect("live window")).id()) as u32,
            crate::src::log::log_bytes(
                (*fs)
                    .window_handle()
                    .expect("target window")
                    .name()
                    .as_bytes()
            )
        ));
    } else {
        log_debug(format_args!("{}: wl=none", log_cstr((prefix) as *const _)));
    }
    if !(*fs).pane_handle().is_none() {
        log_debug(format_args!(
            "{}: wp=%{}",
            log_cstr((prefix) as *const _),
            ((*(*fs)
                .pane_handle()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get()))
            .id) as u32
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
    s_owner: &SessionRef,
    mut flags: ::core::ffi::c_int,
) {
    cmd_find_clear_state(fs, flags);
    (*fs).set_s(Some(s_owner));
    (*fs).set_wl(
        ((*fs)
            .session_handle()
            .expect("live session")
            .current_winlink())
        .clone(),
    );
    (*fs).set_w(((*fs).winlink_handle()).get_unchecked().window_handle());
    (*fs).set_wp(
        ((((*fs).window_handle().as_ref()).expect("live window"))
            .active_pane()
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get()))
        .as_ref(),
    );
    cmd_find_log_state(
        b"cmd_find_from_session\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
}
pub unsafe fn cmd_find_from_winlink(
    mut fs: *mut cmd_find_state,
    mut wl: refbox::Weak<winlink>,
    mut flags: ::core::ffi::c_int,
) {
    cmd_find_clear_state(fs, flags);
    (*fs).s = wl.get_unchecked().session.clone();
    (*fs).set_wl(wl.clone());
    (*fs).set_w(wl.get_unchecked().window_handle());
    (*fs).set_wp(
        (((wl.get_unchecked().window_handle().as_ref()).expect("live window"))
            .active_pane()
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get()))
        .as_ref(),
    );
    cmd_find_log_state(
        b"cmd_find_from_winlink\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
}
pub unsafe fn cmd_find_from_session_window(
    mut fs: *mut cmd_find_state,
    s_owner: &SessionRef,
    w_owner: &WindowRef,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    cmd_find_clear_state(fs, flags);
    (*fs).set_s(Some(s_owner));
    (*fs).set_w(Some(w_owner));
    if cmd_find_best_winlink_with_window(fs) != 0 as ::core::ffi::c_int {
        cmd_find_clear_state(fs, flags);
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).set_wp(
        ((((*fs).window_handle().as_ref()).expect("live window"))
            .active_pane()
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get()))
        .as_ref(),
    );
    cmd_find_log_state(
        b"cmd_find_from_session_window\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn cmd_find_from_window(
    mut fs: *mut cmd_find_state,
    w_owner: &WindowRef,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    cmd_find_clear_state(fs, flags);
    (*fs).set_w(Some(w_owner));
    if cmd_find_best_session_with_window(fs) != 0 as ::core::ffi::c_int {
        cmd_find_clear_state(fs, flags);
        return -(1 as ::core::ffi::c_int);
    }
    if cmd_find_best_winlink_with_window(fs) != 0 as ::core::ffi::c_int {
        cmd_find_clear_state(fs, flags);
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).set_wp(
        ((((*fs).window_handle().as_ref()).expect("live window"))
            .active_pane()
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get()))
        .as_ref(),
    );
    cmd_find_log_state(
        b"cmd_find_from_window\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn cmd_find_from_winlink_pane(
    mut fs: *mut cmd_find_state,
    mut wl: refbox::Weak<winlink>,
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut flags: ::core::ffi::c_int,
) {
    let wp = wp_owner.get();
    cmd_find_clear_state(fs, flags);
    (*fs).s = wl.get_unchecked().session.clone();
    (*fs).set_wl(wl.clone());
    (*fs).idx = ((*fs).winlink_handle()).get_unchecked().idx;
    (*fs).set_w(((*fs).winlink_handle()).get_unchecked().window_handle());
    (*fs).set_wp((wp).as_ref());
    cmd_find_log_state(
        b"cmd_find_from_winlink_pane\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
}
pub unsafe fn cmd_find_from_pane(
    mut fs: *mut cmd_find_state,
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let wp = wp_owner.get();
    if cmd_find_from_window(
        fs,
        &std::rc::Rc::clone(&(((*wp).window_handle().as_ref()).expect("live window"))),
        flags,
    ) != 0 as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).set_wp((wp).as_ref());
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
    (*fs).s = best_session_owner
        .as_ref()
        .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
    if (*fs).session_handle().is_none() {
        cmd_find_clear_state(fs, flags);
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).set_wl(
        ((*fs)
            .session_handle()
            .expect("live session")
            .current_winlink())
        .clone(),
    );
    (*fs).idx = ((*fs).winlink_handle()).get_unchecked().idx;
    (*fs).set_w(((*fs).winlink_handle()).get_unchecked().window_handle());
    (*fs).set_wp(
        ((((*fs).window_handle().as_ref()).expect("live window"))
            .active_pane()
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get()))
        .as_ref(),
    );
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
        let mut wl = refbox::Weak::new();
        let mut mouse_session_owner = None;
        mouse_pane_owner = cmd_mouse_pane(m, Some(&mut mouse_session_owner), &raw mut wl);
        let wp = mouse_pane_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
        (*fs).set_s(mouse_session_owner.as_ref());
        (*fs).set_wl(wl.clone());
        (*fs).set_wp((wp).as_ref());
    }
    if (*fs).pane_handle().is_none() {
        cmd_find_clear_state(fs, flags);
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).set_w(((*fs).winlink_handle()).get_unchecked().window_handle());
    cmd_find_log_state(
        b"cmd_find_from_mouse\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn cmd_find_from_client(
    mut fs: *mut cmd_find_state,
    c_owner: Option<&ClientRef>,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut c: Option<ClientRef> = c_owner.cloned();
    let inside_pane_owner;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if c.is_none() {
        return cmd_find_from_nothing(fs, flags);
    }
    if !c
        .as_ref()
        .expect("live client")
        .attached_session()
        .upgrade()
        .is_none()
    {
        cmd_find_clear_state(fs, flags);
        (*fs).set_wp(
            ((((c
                .as_ref()
                .expect("live client")
                .attached_session()
                .upgrade()
                .expect("live session")
                .current_winlink())
            .get_unchecked()
            .window_handle()
            .as_ref())
            .expect("live window"))
            .active_pane()
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get()))
            .as_ref(),
        );
        if (*fs).pane_handle().is_none() {
            cmd_find_from_session(
                fs,
                &c.as_ref()
                    .expect("live client")
                    .attached_session()
                    .upgrade()
                    .expect("live session"),
                flags,
            );
            return 0 as ::core::ffi::c_int;
        }
        (*fs).set_s(
            c.as_ref()
                .expect("live client")
                .attached_session()
                .upgrade()
                .as_ref(),
        );
        (*fs).set_wl(
            ((*fs)
                .session_handle()
                .expect("live session")
                .current_winlink())
            .clone(),
        );
        (*fs).set_w(((*fs).winlink_handle()).get_unchecked().window_handle());
        cmd_find_log_state(
            b"cmd_find_from_client\0" as *const u8 as *const ::core::ffi::c_char,
            fs,
        );
        return 0 as ::core::ffi::c_int;
    }
    cmd_find_clear_state(fs, flags);
    inside_pane_owner = cmd_find_inside_pane(c_owner);
    wp = inside_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !wp.is_null() {
        (*fs).set_w((*wp).window_handle().as_ref());
        if !(cmd_find_best_session_with_window(fs) != 0 as ::core::ffi::c_int) {
            (*fs).set_wl(
                ((*fs)
                    .session_handle()
                    .expect("live session")
                    .current_winlink())
                .clone(),
            );
            (*fs).set_w(((*fs).winlink_handle()).get_unchecked().window_handle());
            (*fs).set_wp(
                ((((*fs).window_handle().as_ref()).expect("live window"))
                    .active_pane()
                    .as_ref()
                    .map_or(std::ptr::null_mut(), |owner| owner.get()))
                .as_ref(),
            );
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
    item_handle: Option<&std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>>,
    mut target: *const ::core::ffi::c_char,
    mut type_0: cmd_find_type,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let item = item_handle.map_or(std::ptr::null_mut(), |item| item.get());
    let mouse_pane_owner;
    let queue_client = cmdq_get_client((item).as_ref());
    let mut queue_client_ptr: Option<ClientRef> = queue_client.clone();
    let mut current_block: u64;
    let mut m: *mut mouse_event = ::core::ptr::null_mut::<mouse_event>();
    let mut c: Option<ClientRef> = None;
    let mut current: cmd_find_state = cmd_find_state {
        flags: 0,
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
    let queue_current = cmdq_get_state_owned(&*(item)).current_snapshot();
    let mut queue_event = cmdq_get_event(&*(item));
    let mut current_context: Option<cmd_find_state> = None;
    cmd_find_clear_state(fs, flags);
    if server_check_marked() != 0 && flags & CMD_FIND_DEFAULT_MARKED != 0 {
        current_context = Some((&marked_pane).clone());
        log_debug(format_args!(
            "{}: current is marked pane",
            "cmd_find_target"
        ));
        current_block = 1836292691772056875;
    } else if cmd_find_valid_state(&queue_current) != 0 {
        current_context = Some(queue_current);
        log_debug(format_args!("{}: current is from queue", "cmd_find_target"));
        current_block = 1836292691772056875;
    } else if cmd_find_from_client(&raw mut current, queue_client_ptr.as_ref(), flags)
        == 0 as ::core::ffi::c_int
    {
        current_context = Some(current);
        log_debug(format_args!(
            "{}: current is from client",
            "cmd_find_target"
        ));
        current_block = 1836292691772056875;
    } else {
        if !flags & CMD_FIND_QUIET != 0 {
            cmdq_error(item_handle.expect("command queue item"), |out| {
                out.write_all(b"no current target")
            });
        }
        current_block = 5193823237153215208;
    }
    match current_block {
        1836292691772056875 => {
            let current_context = current_context.as_ref().expect("selected current target");
            if cmd_find_valid_state(current_context) == 0 {
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
                if c.is_none()
                    || c.as_ref()
                        .expect("live client")
                        .attached_session()
                        .upgrade()
                        .is_none()
                {
                    cmdq_error(item_handle.expect("command queue item"), |out| {
                        out.write_all(b"no current client")
                    });
                    current_block = 5193823237153215208;
                } else {
                    (*fs).set_wl(
                        (c.as_ref()
                            .expect("live client")
                            .attached_session()
                            .upgrade()
                            .expect("live session")
                            .current_winlink())
                        .clone(),
                    );
                    (*fs).set_wp(
                        ((((c
                            .as_ref()
                            .expect("live client")
                            .attached_session()
                            .upgrade()
                            .expect("live session")
                            .current_winlink())
                        .get_unchecked()
                        .window_handle()
                        .as_ref())
                        .expect("live window"))
                        .active_pane()
                        .as_ref()
                        .map_or(std::ptr::null_mut(), |owner| owner.get()))
                        .as_ref(),
                    );
                    (*fs).set_w(
                        (c.as_ref()
                            .expect("live client")
                            .attached_session()
                            .upgrade()
                            .expect("live session")
                            .current_winlink())
                        .get_unchecked()
                        .window_handle(),
                    );
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
                            let mut wl = refbox::Weak::new();
                            let mut mouse_session_owner = None;
                            mouse_pane_owner =
                                cmd_mouse_pane(m, Some(&mut mouse_session_owner), &raw mut wl);
                            let wp = mouse_pane_owner
                                .as_ref()
                                .map_or(std::ptr::null_mut(), |owner| owner.get());
                            (*fs).set_s(mouse_session_owner.as_ref());
                            (*fs).set_wl(wl.clone());
                            (*fs).set_wp((wp).as_ref());
                        }
                        if !(*fs).pane_handle().is_none() {
                            (*fs).set_w(((*fs).winlink_handle()).get_unchecked().window_handle());
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
                            let mut mouse_session_owner = None;
                            let wl = cmd_mouse_window(m, Some(&mut mouse_session_owner));
                            (*fs).set_s(mouse_session_owner.as_ref());
                            (*fs).set_wl(wl.clone());
                        }
                        if !(*fs).winlink_handle().is_alive() && !(*fs).session_handle().is_none() {
                            (*fs).set_wl(
                                ((*fs)
                                    .session_handle()
                                    .expect("live session")
                                    .current_winlink())
                                .clone(),
                            );
                        }
                        if (*fs).winlink_handle().is_alive() {
                            (*fs).set_w(((*fs).winlink_handle()).get_unchecked().window_handle());
                            (*fs).set_wp(
                                ((((*fs).window_handle().as_ref()).expect("live window"))
                                    .active_pane()
                                    .as_ref()
                                    .map_or(std::ptr::null_mut(), |owner| owner.get()))
                                .as_ref(),
                            );
                        }
                    }
                    _ => {}
                }
                if (*fs).pane_handle().is_none() {
                    if !flags & CMD_FIND_QUIET != 0 {
                        cmdq_error(item_handle.expect("command queue item"), |out| {
                            out.write_all(b"no mouse target")
                        });
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
                        cmdq_error(item_handle.expect("command queue item"), |out| {
                            out.write_all(b"no marked target")
                        });
                    }
                    current_block = 5193823237153215208;
                } else {
                    cmd_find_copy_state(fs, &raw const marked_pane);
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
                if !window.is_null() {
                    window =
                        cmd_find_map_table(cmd_find_window_table, CStr::from_ptr(window)).as_ptr();
                }
                if !pane.is_null() {
                    pane = cmd_find_map_table(cmd_find_pane_table, CStr::from_ptr(pane)).as_ptr();
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
                        cmdq_error(item_handle.expect("command queue item"), |out| {
                            out.write_all(b"can't specify pane here")
                        });
                    }
                    current_block = 5193823237153215208;
                } else {
                    if !session.is_null() {
                        if cmd_find_get_session(fs, session) != 0 as ::core::ffi::c_int {
                            if !flags & CMD_FIND_QUIET != 0 {
                                cmdq_error(item_handle.expect("command queue item"), |out| {
                                    out.write_all(b"can't find session: ")?;
                                    write_cstr(out, session)
                                });
                            }
                            current_block = 5193823237153215208;
                        } else if window.is_null() && pane.is_null() {
                            (*fs).set_wl(
                                ((*fs)
                                    .session_handle()
                                    .expect("live session")
                                    .current_winlink())
                                .clone(),
                            );
                            (*fs).idx = -(1 as ::core::ffi::c_int);
                            (*fs).set_w(((*fs).winlink_handle()).get_unchecked().window_handle());
                            (*fs).set_wp(
                                ((((*fs).window_handle().as_ref()).expect("live window"))
                                    .active_pane()
                                    .as_ref()
                                    .map_or(std::ptr::null_mut(), |owner| owner.get()))
                                .as_ref(),
                            );
                            current_block = 15319680530019787978;
                        } else if !window.is_null() && pane.is_null() {
                            if cmd_find_get_window_with_session(fs, window)
                                != 0 as ::core::ffi::c_int
                            {
                                current_block = 2743676411188200708;
                            } else {
                                if (*fs).winlink_handle().is_alive() {
                                    (*fs).set_wp(
                                        (((((*fs).winlink_handle())
                                            .get_unchecked()
                                            .window_handle()
                                            .as_ref())
                                        .expect("live window"))
                                        .active_pane()
                                        .as_ref()
                                        .map_or(std::ptr::null_mut(), |owner| owner.get()))
                                        .as_ref(),
                                    );
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
                        if cmd_find_get_window(fs, window, window_only, current_context)
                            != 0 as ::core::ffi::c_int
                        {
                            current_block = 2743676411188200708;
                        } else if cmd_find_get_pane_with_window(fs, pane) != 0 as ::core::ffi::c_int
                        {
                            current_block = 14917847580669770662;
                        } else {
                            current_block = 15319680530019787978;
                        }
                    } else if !window.is_null() && pane.is_null() {
                        if cmd_find_get_window(fs, window, window_only, current_context)
                            != 0 as ::core::ffi::c_int
                        {
                            current_block = 2743676411188200708;
                        } else {
                            if (*fs).winlink_handle().is_alive() {
                                (*fs).set_wp(
                                    (((((*fs).winlink_handle())
                                        .get_unchecked()
                                        .window_handle()
                                        .as_ref())
                                    .expect("live window"))
                                    .active_pane()
                                    .as_ref()
                                    .map_or(std::ptr::null_mut(), |owner| owner.get()))
                                    .as_ref(),
                                );
                            }
                            current_block = 15319680530019787978;
                        }
                    } else if window.is_null() && !pane.is_null() {
                        if cmd_find_get_pane(fs, pane, pane_only, current_context)
                            != 0 as ::core::ffi::c_int
                        {
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
                                            item_handle.expect("command queue item"),
                                            |out| {
                                                out.write_all(b"can't find window: ")?;
                                                write_cstr(out, window)
                                            },
                                        );
                                    }
                                }
                                _ => {
                                    if !flags & CMD_FIND_QUIET != 0 {
                                        cmdq_error(
                                            item_handle.expect("command queue item"),
                                            |out| {
                                                out.write_all(b"can't find pane: ")?;
                                                write_cstr(out, pane)
                                            },
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
                            cmd_find_copy_state(fs, std::ptr::from_ref(current_context));
                            if flags & CMD_FIND_WINDOW_INDEX != 0 {
                                (*fs).idx = -(1 as ::core::ffi::c_int);
                            }
                        }
                        _ => {}
                    }
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
    log_debug(format_args!("{}: error", "cmd_find_target"));
    drop(copy_owned);
    if flags & CMD_FIND_CANFAIL != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe fn cmd_find_current_client(
    item_handle: Option<&std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>>,
    mut quiet: ::core::ffi::c_int,
) -> Option<ClientRef> {
    let item = item_handle.map_or(std::ptr::null_mut(), |item| item.get());
    let inside_pane_owner;
    let mut c: Option<ClientRef> = None;
    let mut found = None;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    let mut c_owner = None;
    if !item.is_null() {
        c_owner = cmdq_get_client((item).as_ref());
        c = c_owner.clone();
    }
    if !c.is_none()
        && !c
            .as_ref()
            .expect("live client")
            .attached_session()
            .upgrade()
            .is_none()
    {
        return c_owner;
    }
    found = None;
    if !c.is_none() && {
        inside_pane_owner = cmd_find_inside_pane(c.as_ref());
        wp = inside_pane_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
        !wp.is_null()
    } {
        cmd_find_clear_state(&raw mut fs, CMD_FIND_QUIET);
        fs.set_w((*wp).window_handle().as_ref());
        if cmd_find_best_session_with_window(&raw mut fs) == 0 as ::core::ffi::c_int {
            if let Some(session_owner) = fs.s.upgrade() {
                found = cmd_find_best_client(&session_owner);
            }
        }
    } else {
        if let Some(session_owner) = cmd_find_best_session(None, CMD_FIND_QUIET) {
            found = cmd_find_best_client(&session_owner);
        }
    }
    if found.is_none() && !item.is_null() && quiet == 0 {
        cmdq_error(item_handle.expect("command queue item"), |out| {
            out.write_all(b"no current client")
        });
    }
    log_debug(format_args!(
        "{}: no target, return {}",
        "cmd_find_current_client",
        log_pointer(
            found
                .as_ref()
                .map_or(std::ptr::null(), |owner| std::rc::Rc::as_ptr(owner).cast())
        )
    ));
    return found;
}
pub unsafe fn cmd_find_client(
    item_handle: Option<&std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>>,
    mut target: *const ::core::ffi::c_char,
    mut quiet: ::core::ffi::c_int,
) -> Option<ClientRef> {
    if target.is_null() {
        return cmd_find_current_client(item_handle, quiet);
    }
    let target_bytes = CStr::from_ptr(target).to_bytes();
    let trimmed = target_bytes.strip_suffix(b":").unwrap_or(target_bytes);
    let copy = CString::new(trimmed).expect("client target came from a C string");
    let mut found = clients.first();
    while let Some(client) = found.as_ref() {
        if client.attached_session().upgrade().is_some() {
            if client
                .name()
                .as_deref()
                .is_some_and(|name| name == copy.as_c_str())
            {
                break;
            }
            if let Some(tty) = client.tty_name() {
                let tty = tty.as_bytes();
                if !tty.is_empty()
                    && (tty == trimmed || tty.strip_prefix(b"/dev/") == Some(trimmed))
                {
                    break;
                }
            }
        }
        found = clients.next(client);
    }
    if found.is_none() && quiet == 0 {
        cmdq_error(item_handle.expect("command queue item"), |out| {
            out.write_all(b"can't find client: ")?;
            out.write_all(copy.as_bytes())
        });
    }
    log_debug(format_args!(
        "cmd_find_client: target {}, return {}",
        log_cstr(target),
        log_pointer(
            found
                .as_ref()
                .map_or(std::ptr::null(), |owner| Rc::as_ptr(owner).cast())
        )
    ));
    found
}

#[cfg(test)]
mod target_observer_tests {
    use super::*;
    use crate::src::shared::rc;
    use crate::src::window::{winlink_add, winlink_remove, winlink_set_window, winlinks_reindex};
    use std::rc::Rc;

    #[test]
    fn matching_window_prefers_current_link_then_lowest_index() {
        unsafe {
            let session = session::new();
            let window = window::new();
            let other = window::new();
            let first = crate::src::session::test_support::add_link(&session, 2);
            let current = crate::src::session::test_support::add_link(&session, 9);
            let unrelated = crate::src::session::test_support::add_link(&session, 1);
            winlink_set_window(first.clone(), &window);
            winlink_set_window(current.clone(), &window);
            winlink_set_window(unrelated.clone(), &other);
            let mut state = cmd_find_state::default();
            state.set_s(Some(&session));
            state.set_w(Some(&window));
            let owners = Rc::strong_count(&window);

            crate::src::session::test_support::current(&session, current.clone());
            assert_eq!(cmd_find_best_winlink_with_window(&mut state), 0);
            assert!(state.wl == current);
            assert_eq!(state.idx, 9);

            crate::src::session::test_support::current(&session, unrelated.clone());
            assert_eq!(cmd_find_best_winlink_with_window(&mut state), 0);
            assert!(state.wl == first);
            assert_eq!(state.idx, 2);
            assert_eq!(Rc::strong_count(&window), owners);

            for link in [first, current, unrelated] {
                crate::src::session::test_support::remove_link(&session, link);
            }
            assert_eq!(cmd_find_best_winlink_with_window(&mut state), -1);
            window.release(c"test target window");
            other.release(c"test other window");
        }
    }

    #[test]
    fn best_client_selection_preserves_filtering_ties_and_retains_result() {
        unsafe {
            let saved = std::mem::replace(
                &mut clients,
                crate::src::server_client::ClientRegistry::new(),
            );
            let session_owner = session::new();
            let other_session = session::new();
            let first = client::with_session_for_test(Some(&session_owner));
            let second = client::with_session_for_test(Some(&other_session));
            client::activity_for_test(&first, 10, 0);
            client::activity_for_test(&second, 20, 0);
            clients.push_back(first.clone());
            clients.push_back(second.clone());
            crate::src::session::test_support::metadata(&session_owner, None, None, Some(1));
            assert!(Rc::ptr_eq(
                &cmd_find_best_client(&session_owner).unwrap(),
                &first
            ));
            crate::src::session::test_support::metadata(&session_owner, None, None, Some(0));
            assert!(Rc::ptr_eq(
                &cmd_find_best_client(&session_owner).unwrap(),
                &second
            ));
            client::activity_for_test(&second, 10, 0);
            assert!(Rc::ptr_eq(
                &cmd_find_best_client(&session_owner).unwrap(),
                &first
            ));
            client::activity_for_test(&second, 10, 1);
            let selected = cmd_find_best_client(&session_owner).unwrap();
            let observer = Rc::downgrade(&second);
            let registry = std::mem::replace(&mut clients, saved);
            drop(registry);
            drop(first);
            drop(second);
            assert!(observer.upgrade().is_some());
            drop(selected);
            assert!(observer.upgrade().is_none());
        }
    }

    #[test]
    fn session_ranking_preserves_attachment_preference_recency_and_ties() {
        unsafe {
            let first = session::new();
            let second = session::new();
            crate::src::session::test_support::activity(
                &first,
                UNIX_EPOCH + Duration::from_secs(10),
            );
            crate::src::session::test_support::activity(
                &second,
                UNIX_EPOCH + Duration::from_secs(9),
            );
            assert!(cmd_find_session_better(&first, None, 0));
            assert!(cmd_find_session_better(&first, Some(&second), 0));
            crate::src::session::test_support::metadata(&first, None, None, Some(1));
            assert!(!cmd_find_session_better(
                &first,
                Some(&second),
                CMD_FIND_PREFER_UNATTACHED
            ));
            assert!(cmd_find_session_better(
                &second,
                Some(&first),
                CMD_FIND_PREFER_UNATTACHED
            ));
            crate::src::session::test_support::activity(
                &second,
                UNIX_EPOCH + Duration::from_secs(10),
            );
            assert!(!cmd_find_session_better(&first, Some(&second), 0));
            crate::src::session::test_support::activity(
                &first,
                UNIX_EPOCH + Duration::from_secs(10) + Duration::from_micros(1),
            );
            assert!(cmd_find_session_better(&first, Some(&second), 0));
        }
    }

    #[test]
    fn expired_targets_keep_identity_until_explicitly_cleared() {
        unsafe {
            let session = session::new();
            let window = window::new();
            let pane = window_pane::new();
            let mut links = Default::default();
            let link = winlink_add(&mut links, 7);
            let mut state = cmd_find_state::default();
            state.set_s(Some(&session));
            state.set_w(Some(&window));
            state.set_wp((rc::as_ptr(&pane)).as_ref());
            state.set_wl(link.clone());
            assert_eq!(Rc::strong_count(&session), 1);
            assert_eq!(Rc::strong_count(&window), 1);
            assert_eq!(Rc::strong_count(&pane), 1);
            let snapshot = state.clone();
            drop(session);
            crate::src::window::window_remove_ref(window, c"test owner".as_ptr());
            drop(pane);
            winlink_remove(&mut links, link.clone());
            assert!(state.session_handle().is_none());
            assert!(state.window_handle().is_none());
            assert!(state.pane_handle().is_none());
            assert!(!state.winlink_handle().is_alive());
            assert_eq!(cmd_find_valid_state(&state), 0);
            assert_eq!(cmd_find_empty_state(&state), 0);
            assert!(state.s.ptr_eq(&snapshot.s));
            assert!(state.w.ptr_eq(&snapshot.w));
            assert!(state.wp.ptr_eq(&snapshot.wp));
            assert!(state.wl == snapshot.wl);
            cmd_find_clear_state(&mut state, CMD_FIND_QUIET);
            assert_eq!(state.flags, CMD_FIND_QUIET);
            assert_eq!(state.idx, -1);
            assert_eq!(cmd_find_empty_state(&state), 1);
            assert_eq!(cmd_find_empty_state(&snapshot), 0);
        }
    }

    #[test]
    fn copying_releases_old_handles_and_preserves_flags() {
        unsafe {
            let session = session::new();
            let other = session::new();
            let mut source = cmd_find_state {
                flags: 11,
                idx: 17,
                ..Default::default()
            };
            source.set_s(Some(&session));
            let mut destination = cmd_find_state {
                flags: 22,
                ..Default::default()
            };
            destination.set_s(Some(&other));
            let old_count = Rc::weak_count(&other);
            let new_count = Rc::weak_count(&session);
            cmd_find_copy_state(&mut destination, &mut source);
            assert_eq!(Rc::weak_count(&other), old_count - 1);
            assert_eq!(Rc::weak_count(&session), new_count + 1);
            assert_eq!(destination.flags, 22);
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
                &mut sessions,
                crate::src::shared::session::sessions { storage: None },
            );
            let session_owner = session::new();
            let window_owner = window::new();
            let pane_owner = window_pane::new();
            assert!(!session_owner.is_registered());
            let wp = rc::as_ptr(&pane_owner);
            crate::src::session::sessions_insert(&mut sessions, session_owner.clone());
            assert!(session_owner.is_registered());
            let mut wl = crate::src::session::test_support::add_link(&session_owner, 1);
            wl.get_mut_unchecked().session = Rc::downgrade(&session_owner);
            winlink_set_window(wl.clone(), &window_owner);
            (*wp).window = Rc::downgrade(&window_owner);
            window_owner.initialize_pane(&pane_owner, None);
            crate::src::session::test_support::current(&session_owner, wl.clone());
            assert!(cmd_find_best_session(Some(&[]), 0).is_none());
            let candidates = vec![session_owner.clone()];
            let selected = cmd_find_best_session(Some(&candidates), 0).unwrap();
            assert!(Rc::ptr_eq(&selected, &session_owner));
            drop(candidates);
            assert!(Rc::ptr_eq(&selected, &session_owner));
            drop(selected);
            crate::src::session::test_support::current(&session_owner, refbox::Weak::new());
            let mut state = cmd_find_state::default();
            cmd_find_from_winlink_pane(&mut state, wl.clone(), &pane_owner, 0);
            assert_eq!(cmd_find_valid_state(&state), 1);
            crate::src::session::test_support::reindex(&session_owner, wl.clone(), 3);
            assert_eq!(cmd_find_valid_state(&state), 1);

            let panes = std::mem::take(
                &mut window_owner
                    .borrow_pane_order_mut(crate::src::window::PaneOrder::Index)
                    .storage,
            );
            assert_eq!(cmd_find_valid_state(&state), 0);
            window_owner
                .borrow_pane_order_mut(crate::src::window::PaneOrder::Index)
                .storage = panes;
            drop(crate::src::session::sessions_remove(
                &mut sessions,
                &session_owner,
            ));
            assert!(state.s.upgrade().is_some());
            assert!(!session_owner.is_registered());
            assert_eq!(cmd_find_valid_state(&state), 0);
            crate::src::session::sessions_insert(&mut sessions, session_owner.clone());
            assert_eq!(cmd_find_valid_state(&state), 1);

            crate::src::session::test_support::remove_link(&session_owner, wl.clone());
            let mut replacement = crate::src::session::test_support::add_link(&session_owner, 3);
            replacement.get_mut_unchecked().session = Rc::downgrade(&session_owner);
            winlink_set_window(replacement.clone(), &window_owner);
            assert_eq!(cmd_find_valid_state(&state), 0);
            state.set_wl((replacement).clone());
            assert_eq!(cmd_find_valid_state(&state), 1);
            crate::src::session::test_support::remove_link(&session_owner, (replacement).clone());
            window_owner
                .borrow_pane_order_mut(crate::src::window::PaneOrder::Index)
                .storage
                .clear();
            (*wp).window = std::rc::Weak::new();
            drop(crate::src::session::sessions_remove(
                &mut sessions,
                &session_owner,
            ));
            sessions = saved;
            crate::src::window::window_remove_ref(window_owner, c"test owner".as_ptr());
        }
    }
}
