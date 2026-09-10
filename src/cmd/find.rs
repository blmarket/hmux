use crate::WindowPane;

use crate::cmd::{cmd_mouse_pane, cmd_mouse_window};
use crate::compat::strtonum;
use crate::environ::EnvironmentStore;
use crate::ffi::fnmatch;
use crate::fmt_args;
use crate::log::{fatalx, log_debug};
use crate::server::marked_pane;
use crate::server::server_check_marked;
use crate::server::server_client_get_pane;
use crate::server::with_clients;

use crate::session::SESSIONS_FIELD;

pub use crate::consts::{
    CMD_FIND_CANFAIL, CMD_FIND_DEFAULT_MARKED, CMD_FIND_PANE, CMD_FIND_PREFER_UNATTACHED,
    CMD_FIND_QUIET, CMD_FIND_SESSION, CMD_FIND_WINDOW, CMD_FIND_WINDOW_INDEX, INT_MAX,
};
pub type cmd_find_type = core::ffi::c_uint;
pub use crate::cmd::cmdq_item;
pub use crate::types::{
    ClientRef, RustWindowPaneWeak, SessionRef, WindowRef, cmd_find_state, mouse_event, session,
    u_int, winlink,
};
#[cfg(test)]
use crate::types::{RustWindowPaneRef, window_pane};
use crate::window::pane_walk;
use crate::window::{
    window_pane_find_by_id_str, window_pane_find_down, window_pane_find_left,
    window_pane_find_right, window_pane_find_up, winlink_next_by_number,
    winlink_previous_by_number,
};
use crate::{Session, SessionAttachmentState};
use ::core::ffi::CStr;
use ::std::ffi::CString;

pub const _PATH_DEV: &CStr = c"/dev/";

pub const CMD_FIND_EXACT_SESSION: core::ffi::c_int = 0x10 as core::ffi::c_int;
pub const CMD_FIND_EXACT_WINDOW: core::ffi::c_int = 0x20 as core::ffi::c_int;

static cmd_find_session_table: &[(&CStr, &CStr)] = &[];
static cmd_find_window_table: &[(&CStr, &CStr)] = &[
    (c"{start}", c"^"),
    (c"{last}", c"!"),
    (c"{end}", c"$"),
    (c"{next}", c"+"),
    (c"{previous}", c"-"),
];
static cmd_find_pane_table: &[(&CStr, &CStr)] = &[
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
unsafe fn cmd_find_inside_pane(c: Option<&ClientRef>) -> Option<RustWindowPaneWeak> {
    unsafe {
        let c = c?;
        let mut pane = pane_walk().find(|pane| {
            let wp = pane.get().expect("registered pane");
            *wp.fd() != -1 && c.ttyname_ref().as_deref() == Some(wp.terminal_name())
        });
        if pane.is_none()
            && let Some(value) = c
                .environ_ref()
                .find(c"TMUX_PANE")
                .and_then(|entry| entry.value)
        {
            pane = window_pane_find_by_id_str(value);
        }
        if let Some(pane) = &pane {
            let wp = pane.get().expect("registered pane");
            log_debug(
                c"%s: got pane %%%u (%s)",
                fmt_args![c"cmd_find_inside_pane", pane.id(), wp.terminal_name()],
            );
        }
        pane
    }
}
fn cmd_find_client_better(c: &ClientRef, than: Option<&ClientRef>) -> core::ffi::c_int {
    let Some(than) = than else {
        return 1 as core::ffi::c_int;
    };
    {
        if c.activity_time().tv_sec == than.activity_time().tv_sec {
            (c.activity_time().tv_usec > than.activity_time().tv_usec) as core::ffi::c_int
        } else {
            (c.activity_time().tv_sec > than.activity_time().tv_sec) as core::ffi::c_int
        }
    }
}
/// Selects a retained client using the retained session's current attachment state.
///
/// If the session has attached clients, only its clients are eligible. Otherwise
/// any attached client is eligible. The most recently active client wins, with
/// registry order breaking ties; no eligible client returns `None`. The session
/// need not be registered. This query runs no callbacks and changes no state.
///
/// # Safety
/// Run on the initialized server thread without mutable session or client payload
/// access during the query. No payload borrow escapes the call.
pub(crate) unsafe fn cmd_find_best_client_for_session(s: &SessionRef) -> Option<ClientRef> {
    unsafe { cmd_find_best_client(s.as_session()) }
}

pub fn cmd_find_best_client(s: &session) -> Option<ClientRef> {
    {
        let want = (s.session_attached() != 0 as u_int).then_some(s);
        with_clients(|clients| {
            let mut best: Option<&ClientRef> = None;
            for candidate in clients {
                let view = candidate;
                if view
                    .attached_session()
                    .is_some_and(|session| want.is_none_or(|s| session.points_to(s)))
                    && cmd_find_client_better(view, best) != 0
                {
                    best = Some(candidate);
                }
            }
            best.cloned()
        })
    }
}
fn cmd_find_session_better<S: Session>(
    s: &S,
    than: Option<&S>,
    flags: core::ffi::c_int,
) -> core::ffi::c_int {
    let attached: core::ffi::c_int;
    let Some(than) = than else {
        return 1 as core::ffi::c_int;
    };
    if flags & CMD_FIND_PREFER_UNATTACHED != 0 {
        attached = (than.session_attached() != 0 as u_int) as core::ffi::c_int;
        if attached != 0 && s.session_attached() == 0 as u_int {
            return 1 as core::ffi::c_int;
        } else if attached == 0 && s.session_attached() != 0 as u_int {
            return 0 as core::ffi::c_int;
        }
    }
    let activity = s.session_timestamps().activity;
    let than_activity = than.session_timestamps().activity;
    if activity.tv_sec == than_activity.tv_sec {
        (activity.tv_usec > than_activity.tv_usec) as core::ffi::c_int
    } else {
        (activity.tv_sec > than_activity.tv_sec) as core::ffi::c_int
    }
}
pub(crate) unsafe fn cmd_find_best_session(
    slist: &[SessionRef],
    flags: core::ffi::c_int,
) -> Option<SessionRef> {
    let SESSIONS = SESSIONS_FIELD.get();

    unsafe {
        let mut s: Option<SessionRef> = None;
        let mut i: u_int;
        log_debug(
            c"%s: %u sessions to try",
            fmt_args![c"cmd_find_best_session".as_ptr(), slist.len() as u_int],
        );
        if !slist.is_empty() {
            i = 0 as u_int;
            while i < slist.len() as u_int {
                let candidate = &slist[i as usize];
                if cmd_find_session_better(
                    candidate.as_session(),
                    s.as_ref().map(|reference| reference.as_session()),
                    flags,
                ) != 0
                {
                    s = Some(candidate.clone());
                }
                i = i.wrapping_add(1);
            }
        } else {
            for s_loop in SESSIONS.read().values() {
                if cmd_find_session_better(
                    s_loop.as_session(),
                    s.as_ref().map(|reference| reference.as_session()),
                    flags,
                ) != 0
                {
                    s = Some(s_loop.clone());
                }
            }
        }
        s
    }
}
unsafe fn cmd_find_best_session_with_window(fs: &mut cmd_find_state) -> core::ffi::c_int {
    let Some(window) = fs.window() else {
        return -1;
    };
    unsafe { window.find_best_session(fs) }
}
unsafe fn cmd_find_best_winlink_with_window(fs: &mut cmd_find_state) -> core::ffi::c_int {
    let Some(window) = fs.window() else {
        return -1;
    };
    unsafe { window.find_best_winlink(fs) }
}
/// `s` from byte `skip` on, still as a C string. The C walked the pointer
/// forward; a `CStr` tail is the same bytes up to the same NUL.
fn cstr_tail(s: &CStr, skip: usize) -> &CStr {
    CStr::from_bytes_with_nul(&s.to_bytes_with_nul()[skip..])
        .expect("the tail of a C string ends at the same NUL")
}
fn cmd_find_map_table<'a>(table: &[(&'a CStr, &'a CStr)], s: &'a CStr) -> &'a CStr {
    for &(from, to) in table {
        if s == from {
            return to;
        }
    }
    s
}
unsafe fn cmd_find_get_session(fs: &mut cmd_find_state, session: &CStr) -> core::ffi::c_int {
    let SESSIONS = SESSIONS_FIELD.get();

    unsafe {
        let mut selected: Option<SessionRef> = None;
        log_debug(
            c"%s: %s",
            fmt_args![c"cmd_find_get_session".as_ptr(), session],
        );
        if session.to_bytes().first() == Some(&b'$') {
            let found = SessionRef::find_by_id_str(session);
            (*fs).set_session_ref(found.as_ref());
            if (*fs).session().is_none() {
                return -(1 as core::ffi::c_int);
            }
            return 0 as core::ffi::c_int;
        }
        let found = SessionRef::find(session);
        (*fs).set_session_ref(found.as_ref());
        if !(*fs).session().is_none() {
            return 0 as core::ffi::c_int;
        }
        let c = cmd_find_client(None, Some(session), 1 as core::ffi::c_int);
        if let Some(session) = c.as_ref().and_then(|owner| owner.attached_session()) {
            fs.set_session_ref(Some(&session));
            return 0 as core::ffi::c_int;
        }
        if fs.flags & CMD_FIND_EXACT_SESSION != 0 {
            return -(1 as core::ffi::c_int);
        }
        for s_loop in SESSIONS.read().values() {
            if s_loop
                .name()
                .as_deref()
                .is_some_and(|name| name.to_bytes().starts_with(session.to_bytes()))
            {
                if selected.is_some() {
                    return -(1 as core::ffi::c_int);
                }
                selected = Some(s_loop.clone());
            }
        }
        if selected.is_some() {
            fs.set_session_ref(selected.as_ref());
            return 0 as core::ffi::c_int;
        }
        for s_loop in SESSIONS.read().values() {
            if fnmatch(
                session.as_ptr(),
                s_loop
                    .name()
                    .as_deref()
                    .expect("the session has a name")
                    .as_ptr(),
                0 as core::ffi::c_int,
            ) == 0 as core::ffi::c_int
            {
                if selected.is_some() {
                    return -(1 as core::ffi::c_int);
                }
                selected = Some(s_loop.clone());
            }
        }
        if selected.is_some() {
            fs.set_session_ref(selected.as_ref());
            return 0 as core::ffi::c_int;
        }
        -(1 as core::ffi::c_int)
    }
}
unsafe fn cmd_find_get_window(
    current: &cmd_find_state,
    fs: &mut cmd_find_state,
    window: &CStr,
    only: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        log_debug(
            c"%s: %s",
            fmt_args![c"cmd_find_get_window".as_ptr(), window],
        );
        if window.to_bytes().first() == Some(&b'@') {
            fs.set_window_ref(WindowRef::find_by_id_str(window).as_ref());
            if (*fs).window().is_none() {
                return -(1 as core::ffi::c_int);
            }
            return cmd_find_best_session_with_window(fs);
        }
        (*fs).set_session_ref((*current).session().as_ref());
        if cmd_find_get_window_with_session(fs, window) == 0 as core::ffi::c_int {
            return 0 as core::ffi::c_int;
        }
        if only == 0 && cmd_find_get_session(fs, window) == 0 as core::ffi::c_int {
            let session = fs.session().expect("the state names a session");
            let link = session.curw().expect("the session has a current link");
            fs.set_winlink(link.get());
            fs.set_window_ref(link.window().as_ref());
            if !fs.flags & CMD_FIND_WINDOW_INDEX != 0 {
                fs.idx = link.index();
            }
            return 0 as core::ffi::c_int;
        }
        -(1 as core::ffi::c_int)
    }
}
unsafe fn cmd_find_get_window_with_session(
    fs: &mut cmd_find_state,
    window: &CStr,
) -> core::ffi::c_int {
    unsafe {
        let idx: core::ffi::c_int;
        let n: core::ffi::c_int;

        log_debug(
            c"%s: %s",
            fmt_args![c"cmd_find_get_window_with_session".as_ptr(), window],
        );
        let exact: core::ffi::c_int = fs.flags & CMD_FIND_EXACT_WINDOW;
        let session = fs.session().expect("the state names a session");
        let s = session.as_session();
        let current = s.curw().expect("the session has a current link");
        fs.set_winlink(Some(current));
        fs.set_window_ref(current.window_handle());
        if window.to_bytes().first() == Some(&b'@') {
            fs.set_window_ref(WindowRef::find_by_id_str(window).as_ref());
            if (*fs).window().is_none_or(|w| !session.has(&w)) {
                return -(1 as core::ffi::c_int);
            }
            return cmd_find_best_winlink_with_window(fs);
        }
        if exact == 0
            && (window.to_bytes().first() == Some(&b'+')
                || window.to_bytes().first() == Some(&b'-'))
        {
            if window.to_bytes().len() > 1 {
                n = strtonum(
                    cstr_tail(window, 1),
                    1 as core::ffi::c_longlong,
                    INT_MAX as core::ffi::c_longlong,
                )
                .unwrap_or(0) as core::ffi::c_int;
            } else {
                n = 1 as core::ffi::c_int;
            }
            if fs.flags & CMD_FIND_WINDOW_INDEX != 0 {
                if window.to_bytes().first() == Some(&b'+') {
                    if INT_MAX - current.idx < n {
                        return -(1 as core::ffi::c_int);
                    }
                    fs.idx = current.idx + n;
                } else {
                    if n > current.idx {
                        return -(1 as core::ffi::c_int);
                    }
                    fs.idx = current.idx - n;
                }
                return 0 as core::ffi::c_int;
            }
            let link = if window.to_bytes().first() == Some(&b'+') {
                winlink_next_by_number(current, s, n)
            } else {
                winlink_previous_by_number(current, s, n)
            };
            fs.set_winlink(link);
            if let Some(link) = link {
                fs.idx = link.idx;
                fs.set_window_ref(link.window_handle());
                return 0;
            }
        }
        if exact == 0 && matches!(window.to_bytes(), b"!" | b"^" | b"$") {
            let link = match window.to_bytes() {
                b"!" => s.lastw.first().and_then(|index| s.windows.get(index)),
                b"^" => s.windows.first_key_value().map(|(_, link)| link),
                b"$" => s.windows.last_key_value().map(|(_, link)| link),
                _ => unreachable!("the target is a window alias"),
            }
            .map(Box::as_ref);
            fs.set_winlink(link);
            let Some(link) = link else {
                return -1;
            };
            fs.idx = link.idx;
            fs.set_window_ref(link.window_handle());
            return 0;
        }
        if window.to_bytes().first() != Some(&b'+') && window.to_bytes().first() != Some(&b'-') {
            let parsed = strtonum(
                window,
                0 as core::ffi::c_longlong,
                INT_MAX as core::ffi::c_longlong,
            );
            idx = parsed.unwrap_or(0) as core::ffi::c_int;
            if parsed.is_ok() {
                let link = s.windows.get(&idx).map(Box::as_ref);
                fs.set_winlink(link);
                if let Some(link) = link {
                    fs.idx = link.idx;
                    fs.set_window_ref(link.window_handle());
                    return 0;
                }
                if fs.flags & CMD_FIND_WINDOW_INDEX != 0 {
                    fs.idx = idx;
                    return 0 as core::ffi::c_int;
                }
            }
        }
        fs.set_winlink(None);
        for link in session.as_session().windows.values() {
            if window
                == link
                    .window_handle()
                    .expect("a session link has a window")
                    .window_name()
                    .as_deref()
                    .expect("a live window has a name")
            {
                if fs.wl.is_some() {
                    return -(1 as core::ffi::c_int);
                }
                fs.set_winlink(Some(link));
            }
        }
        if let Some(link) = fs.winlink_ref() {
            fs.idx = link.index();
            fs.set_window_ref(
                link.get()
                    .expect("the selected link is present")
                    .window_handle(),
            );
            return 0 as core::ffi::c_int;
        }
        if exact != 0 {
            return -(1 as core::ffi::c_int);
        }
        (*fs).set_winlink(None);
        for link in session.as_session().windows.values() {
            if link
                .window_handle()
                .expect("a session link has a window")
                .window_name()
                .as_deref()
                .expect("a live window has a name")
                .to_bytes()
                .starts_with(window.to_bytes())
            {
                if fs.wl.is_some() {
                    return -(1 as core::ffi::c_int);
                }
                fs.set_winlink(Some(link));
            }
        }
        if let Some(link) = fs.winlink_ref() {
            fs.idx = link.index();
            fs.set_window_ref(
                link.get()
                    .expect("the selected link is present")
                    .window_handle(),
            );
            return 0 as core::ffi::c_int;
        }
        (*fs).set_winlink(None);
        for link in session.as_session().windows.values() {
            if fnmatch(
                window.as_ptr(),
                link.window_handle()
                    .expect("a session link has a window")
                    .window_name()
                    .as_deref()
                    .expect("a live window has a name")
                    .as_ptr(),
                0 as core::ffi::c_int,
            ) == 0 as core::ffi::c_int
            {
                if fs.wl.is_some() {
                    return -(1 as core::ffi::c_int);
                }
                fs.set_winlink(Some(link));
            }
        }
        if let Some(link) = fs.winlink_ref() {
            fs.idx = link.index();
            fs.set_window_ref(
                link.get()
                    .expect("the selected link is present")
                    .window_handle(),
            );
            return 0 as core::ffi::c_int;
        }
        -(1 as core::ffi::c_int)
    }
}
unsafe fn cmd_find_get_pane(
    current: &cmd_find_state,
    fs: &mut cmd_find_state,
    pane: &CStr,
    only: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        log_debug(c"%s: %s", fmt_args![c"cmd_find_get_pane".as_ptr(), pane]);
        if pane.to_bytes().first() == Some(&b'%') {
            fs.wp = window_pane_find_by_id_str(pane);
            let Some(pane) = fs.pane_list_ref() else {
                return -1;
            };
            let Some(pane) = pane.get() else { return -1 };
            fs.set_window_ref(pane.window_context().as_ref());
            return cmd_find_best_session_with_window(fs);
        }
        (*fs).set_session_ref((*current).session().as_ref());
        fs.wl = current.winlink_ref().map(|link| link.index());
        fs.idx = current.idx;
        (*fs).set_window_ref((*current).window().as_ref());
        if cmd_find_get_pane_with_window(fs, pane) == 0 as core::ffi::c_int {
            return 0 as core::ffi::c_int;
        }
        if only == 0
            && cmd_find_get_window(current, fs, pane, 0 as core::ffi::c_int)
                == 0 as core::ffi::c_int
        {
            let window = fs.window().expect("the state names a window");
            fs.wp = window.active_pane();
            return 0 as core::ffi::c_int;
        }
        -(1 as core::ffi::c_int)
    }
}
unsafe fn cmd_find_get_pane_with_session(fs: &mut cmd_find_state, pane: &CStr) -> core::ffi::c_int {
    unsafe {
        log_debug(
            c"%s: %s",
            fmt_args![c"cmd_find_get_pane_with_session".as_ptr(), pane],
        );
        if pane.to_bytes().first() == Some(&b'%') {
            fs.wp = window_pane_find_by_id_str(pane);
            let Some(pane) = fs.pane_list_ref() else {
                return -1;
            };
            let Some(pane) = pane.get() else { return -1 };
            fs.set_window_ref(pane.window_context().as_ref());
            return cmd_find_best_winlink_with_window(fs);
        }
        let session = fs.session().expect("the state names a session");
        let link = session.curw().expect("the session has a current link");
        fs.set_winlink(link.get());
        fs.idx = link.index();
        fs.set_window_ref(link.window().as_ref());
        cmd_find_get_pane_with_window(fs, pane)
    }
}
unsafe fn cmd_find_get_pane_with_window(fs: &mut cmd_find_state, pane: &CStr) -> core::ffi::c_int {
    unsafe {
        let n: u_int;
        log_debug(
            c"%s: %s",
            fmt_args![c"cmd_find_get_pane_with_window".as_ptr(), pane],
        );
        if pane.to_bytes().first() == Some(&b'%') {
            fs.wp = window_pane_find_by_id_str(pane);
            let Some(pane) = fs.pane_list_ref() else {
                return -1;
            };
            let Some(pane) = pane.get() else { return -1 };
            let window = fs.window();
            let same_window = match (pane.window_context(), window) {
                (Some(pane_window), Some(window)) => pane_window.ptr_eq(&window),
                (None, None) => true,
                _ => false,
            };
            if !same_window {
                return -1;
            }
            return 0 as core::ffi::c_int;
        }
        if pane == c"!" {
            let last = fs.window().and_then(|window| window.last_pane());
            let pane = last.as_ref().and_then(|pane| pane.get());
            fs.set_pane(pane);
            return if pane.is_some() { 0 } else { -1 };
        } else if pane == c"{up-of}" {
            let window = fs.window().expect("the state names a window");
            let active = window.active_pane_id().and_then(|id| window.pane_by_id(id));
            let selected = window_pane_find_up(active.as_ref().and_then(|pane| pane.get()));
            fs.set_pane(selected.as_ref().and_then(|pane| pane.get()));
            return if selected.is_some() { 0 } else { -1 };
        } else if pane == c"{down-of}" {
            let window = fs.window().expect("the state names a window");
            let active = window.active_pane_id().and_then(|id| window.pane_by_id(id));
            let selected = window_pane_find_down(active.as_ref().and_then(|pane| pane.get()));
            fs.set_pane(selected.as_ref().and_then(|pane| pane.get()));
            return if selected.is_some() { 0 } else { -1 };
        } else if pane == c"{left-of}" {
            let window = fs.window().expect("the state names a window");
            let active = window.active_pane_id().and_then(|id| window.pane_by_id(id));
            let selected = window_pane_find_left(active.as_ref().and_then(|pane| pane.get()));
            fs.set_pane(selected.as_ref().and_then(|pane| pane.get()));
            return if selected.is_some() { 0 } else { -1 };
        } else if pane == c"{right-of}" {
            let window = fs.window().expect("the state names a window");
            let active = window.active_pane_id().and_then(|id| window.pane_by_id(id));
            let selected = window_pane_find_right(active.as_ref().and_then(|pane| pane.get()));
            fs.set_pane(selected.as_ref().and_then(|pane| pane.get()));
            return if selected.is_some() { 0 } else { -1 };
        }
        if pane.to_bytes().first() == Some(&b'+') || pane.to_bytes().first() == Some(&b'-') {
            if pane.to_bytes().len() > 1 {
                n = strtonum(
                    cstr_tail(pane, 1),
                    1 as core::ffi::c_longlong,
                    INT_MAX as core::ffi::c_longlong,
                )
                .unwrap_or(0) as u_int;
            } else {
                n = 1 as u_int;
            }
            let window = fs.window().expect("the state names a window");
            let active = window.active_pane();
            let selected = if pane.to_bytes().first() == Some(&b'+') {
                window.next_pane_by_number(active.as_ref(), n)
            } else {
                window.previous_pane_by_number(active.as_ref(), n)
            };
            fs.wp = selected.clone();
            if selected.is_some() {
                return 0;
            }
        }
        let parsed = strtonum(
            pane,
            0 as core::ffi::c_longlong,
            INT_MAX as core::ffi::c_longlong,
        );
        let idx: core::ffi::c_int = parsed.unwrap_or(0) as core::ffi::c_int;
        if parsed.is_ok() {
            let window = fs.window().expect("the state names a window");
            let selected = window.pane_at_index(idx as u_int);
            fs.wp = selected.clone();
            if selected.is_some() {
                return 0 as core::ffi::c_int;
            }
        }
        let window = fs.window().expect("the state names a window");
        let selected = window.find_pane_string(pane);
        fs.wp = selected.clone();
        if selected.is_some() {
            return 0;
        }
        -(1 as core::ffi::c_int)
    }
}
pub fn cmd_find_clear_state(fs: &mut cmd_find_state, flags: core::ffi::c_int) {
    *fs = cmd_find_state::default();
    fs.flags = flags;
    fs.idx = -(1 as core::ffi::c_int);
}
pub fn cmd_find_empty_state(fs: &cmd_find_state) -> core::ffi::c_int {
    if fs.session().is_none()
        && fs.winlink_ref().is_none()
        && fs.window().is_none()
        && fs.pane_list_ref().is_none()
    {
        return 1 as core::ffi::c_int;
    }
    0 as core::ffi::c_int
}
pub unsafe fn cmd_find_valid_state(fs: &cmd_find_state) -> core::ffi::c_int {
    unsafe {
        let (Some(link), Some(window), Some(target_pane)) =
            (fs.winlink_ref(), fs.window(), fs.pane_ref())
        else {
            return 0;
        };
        if !link.session().is_registered() {
            return 0;
        }
        if !link
            .get()
            .and_then(|link| link.window_handle())
            .is_some_and(|linked| linked.ptr_eq(&window))
        {
            return 0;
        }
        window
            .panes()
            .iter()
            .any(|pane| *pane == target_pane && pane.get().is_some()) as core::ffi::c_int
    }
}

pub fn cmd_find_copy_state(dst: &mut cmd_find_state, src: &cmd_find_state) {
    dst.set_session_ref(src.session().as_ref());
    dst.wl = src.winlink_ref().map(|link| link.index());
    dst.idx = src.idx;
    dst.set_window_ref(src.window().as_ref());
    dst.wp = src.wp.clone();
}
unsafe fn cmd_find_log_state(prefix: &CStr, fs: &cmd_find_state) {
    unsafe { cmd_find_log_state_with_window(prefix, fs, None) }
}
pub(crate) unsafe fn cmd_find_log_state_with_window(
    prefix: &CStr,
    fs: &cmd_find_state,
    payload: Option<(u_int, Option<CString>)>,
) {
    unsafe {
        if let Some(session) = fs.session() {
            let s = session.as_session();
            log_debug(
                c"%s: s=$%u %s",
                fmt_args![
                    prefix,
                    crate::SessionIdentity::session_id(s),
                    crate::SessionNameState::session_name(s)
                ],
            );
        } else {
            log_debug(c"%s: s=none", fmt_args![prefix]);
        }
        if let Some(link) = fs.winlink_ref() {
            let window = fs.window().expect("the state names a window");
            let matches = link
                .get()
                .and_then(|link| link.window_handle())
                .is_some_and(|linked| linked.ptr_eq(&window));
            let (id, name) = payload.unwrap_or_else(|| (window.window_id(), window.window_name()));
            log_debug(
                c"%s: wl=%u %d w=@%u %s",
                fmt_args![
                    prefix,
                    link.index(),
                    matches as core::ffi::c_int,
                    id,
                    name.as_deref()
                ],
            );
        } else {
            log_debug(c"%s: wl=none", fmt_args![prefix]);
        }
        let pane = fs.pane_list_ref();
        if let Some(pane) = pane {
            log_debug(c"%s: wp=%%%u", fmt_args![prefix, pane.id()]);
        } else {
            log_debug(c"%s: wp=none", fmt_args![prefix]);
        }
        if fs.idx != -1 {
            log_debug(c"%s: idx=%d", fmt_args![prefix, fs.idx]);
        } else {
            log_debug(c"%s: idx=none", fmt_args![prefix]);
        }
    }
}
pub unsafe fn cmd_find_from_session(fs: &mut cmd_find_state, s: &session, flags: core::ffi::c_int) {
    unsafe {
        cmd_find_clear_state(fs, flags);
        (*fs).set_session(Some(s));
        let link = s.curw().expect("the session has a current link");
        fs.set_winlink(Some(link));
        let window = link.window_handle().expect("a session link has a window");
        fs.set_window_ref(Some(window));
        fs.wp = window.active_pane();
        cmd_find_log_state(c"cmd_find_from_session", fs);
    }
}
pub unsafe fn cmd_find_from_winlink(
    fs: &mut cmd_find_state,
    wl: &winlink,
    flags: core::ffi::c_int,
) {
    unsafe {
        cmd_find_clear_state(fs, flags);
        (*fs).set_session_ref(wl.session().as_ref());
        (*fs).set_winlink(Some(wl));
        let window = wl.window_handle().expect("a session link has a window");
        fs.set_window_ref(Some(window));
        fs.wp = window.active_pane();
        cmd_find_log_state(c"cmd_find_from_winlink", fs);
    }
}
pub unsafe fn cmd_find_from_session_window(
    fs: &mut cmd_find_state,
    s: &session,
    w: &WindowRef,
    flags: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe { w.find_from_session(fs, s, flags) }
}

pub unsafe fn cmd_find_from_window(
    fs: &mut cmd_find_state,
    w: &WindowRef,
    flags: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe { w.find_from_window(fs, flags) }
}

pub unsafe fn cmd_find_from_winlink_pane(
    fs: &mut cmd_find_state,
    wl: &winlink,
    wp: &(impl crate::WindowPane + ?Sized),
    flags: core::ffi::c_int,
) {
    unsafe {
        cmd_find_clear_state(fs, flags);
        fs.set_session_ref(wl.session().as_ref());
        fs.set_winlink(Some(wl));
        fs.idx = wl.idx;
        fs.set_window_ref(wl.window_handle());
        fs.wp = (wp).observation();
        cmd_find_log_state(c"cmd_find_from_winlink_pane", fs);
    }
}
pub unsafe fn cmd_find_from_pane(
    fs: &mut cmd_find_state,
    wp: &(impl crate::WindowPane + ?Sized),
    flags: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        let Some(window) = wp.window_context() else {
            return -1;
        };
        cmd_find_from_pane_in_window(fs, wp, &window, flags)
    }
}

pub(crate) unsafe fn cmd_find_from_pane_in_window(
    fs: &mut cmd_find_state,
    wp: &(impl crate::WindowPane + ?Sized),
    window: &WindowRef,
    flags: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        if cmd_find_from_window(fs, window, flags) != 0 {
            return -1;
        }
        fs.set_pane(Some(wp));
        cmd_find_log_state(c"cmd_find_from_pane", fs);
        0
    }
}

pub unsafe fn cmd_find_from_nothing(
    fs: &mut cmd_find_state,
    flags: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        cmd_find_clear_state(fs, flags);
        let Some(session) = cmd_find_best_session(&[], flags) else {
            cmd_find_clear_state(fs, flags);
            return -1;
        };
        fs.set_session_ref(Some(&session));
        let link = session.curw().expect("the session has a current link");
        fs.set_winlink(link.get());
        fs.idx = link.index();
        let window = link.window().expect("a session link has a window");
        fs.set_window_ref(Some(&window));
        fs.wp = window.active_pane();
        cmd_find_log_state(c"cmd_find_from_nothing", fs);
        0 as core::ffi::c_int
    }
}
pub unsafe fn cmd_find_from_mouse(
    fs: &mut cmd_find_state,
    m: &mouse_event,
    flags: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        cmd_find_clear_state(fs, flags);
        let Some((session, link, pane)) = cmd_mouse_pane(m) else {
            return -1;
        };
        fs.set_session_ref(Some(&session));
        fs.wl = Some(link.index());
        fs.set_window_ref(pane.window().as_ref());
        fs.wp = Some(pane.clone());
        cmd_find_log_state(c"cmd_find_from_mouse", fs);
        0
    }
}
pub unsafe fn cmd_find_from_client(
    fs: &mut cmd_find_state,
    c: Option<&ClientRef>,
    flags: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        let Some(c) = c else {
            return cmd_find_from_nothing(fs, flags);
        };
        if let Some(session) = c.attached_session() {
            cmd_find_clear_state(fs, flags);
            let pane = server_client_get_pane(c.as_client());
            fs.set_pane(pane.as_ref().and_then(|pane| pane.get()));
            if fs.pane_list_ref().is_none() {
                cmd_find_from_session(fs, session.as_session(), flags);
                return 0;
            }
            fs.set_session_ref(Some(&session));
            let link = session.curw().expect("the session has a current link");
            fs.set_winlink(link.get());
            fs.set_window_ref(link.window().as_ref());
            cmd_find_log_state(c"cmd_find_from_client", fs);
            return 0;
        }
        cmd_find_clear_state(fs, flags);
        if let Some(pane) = cmd_find_inside_pane(Some(c)) {
            fs.set_window_ref(pane.window().as_ref());
            if cmd_find_best_session_with_window(fs) == 0 {
                let session = fs.session().expect("the state names a session");
                let link = session.curw().expect("the session has a current link");
                fs.set_winlink(link.get());
                let window = link.window().expect("a session link has a window");
                fs.set_window_ref(Some(&window));
                fs.wp = window.active_pane();
                cmd_find_log_state(c"cmd_find_from_client", fs);
                return 0;
            }
        }
        cmd_find_from_nothing(fs, flags)
    }
}
pub unsafe fn cmd_find_target(
    fs: &mut cmd_find_state,
    item: &cmdq_item,
    target: Option<&CStr>,
    type_0: cmd_find_type,
    mut flags: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        let mut current_block: u64;
        let mut current = cmd_find_state::default();
        let window_only: core::ffi::c_int;
        let pane_only: core::ffi::c_int;
        if flags & CMD_FIND_CANFAIL != 0 {
            flags |= CMD_FIND_QUIET;
        }
        let kind = match type_0 {
            CMD_FIND_PANE => c"pane",
            CMD_FIND_WINDOW => c"window",
            CMD_FIND_SESSION => c"session",
            _ => c"unknown",
        };
        let mut named: Vec<&CStr> = Vec::new();
        for (bit, name) in [
            (CMD_FIND_PREFER_UNATTACHED, c"PREFER_UNATTACHED"),
            (CMD_FIND_QUIET, c"QUIET"),
            (CMD_FIND_WINDOW_INDEX, c"WINDOW_INDEX"),
            (CMD_FIND_DEFAULT_MARKED, c"DEFAULT_MARKED"),
            (CMD_FIND_EXACT_SESSION, c"EXACT_SESSION"),
            (CMD_FIND_EXACT_WINDOW, c"EXACT_WINDOW"),
            (CMD_FIND_CANFAIL, c"CANFAIL"),
        ] {
            if flags & bit != 0 {
                named.push(name);
            }
        }
        let mut tmp: Vec<u8> = Vec::new();
        if named.is_empty() {
            tmp.extend_from_slice(b"NONE");
        } else {
            for (at, name) in named.iter().enumerate() {
                if at != 0 {
                    tmp.push(b',');
                }
                tmp.extend_from_slice(name.to_bytes());
            }
        }
        let tmp = CString::new(tmp).expect("flag names have no NUL");
        log_debug(
            c"%s: target %s, type %s, item %p, flags %s",
            fmt_args![
                c"cmd_find_target",
                target.unwrap_or(c"none"),
                kind,
                core::ptr::from_ref(item),
                tmp.as_c_str()
            ],
        );
        cmd_find_clear_state(fs, flags);
        let queue_current = item.current().clone();
        let marked_state = marked_pane.get();
        let mut current_state: Option<&cmd_find_state> = None;
        if server_check_marked() != 0 && flags & CMD_FIND_DEFAULT_MARKED != 0 {
            current_state = Some(&marked_state);
            log_debug(c"%s: current is marked pane", fmt_args![c"cmd_find_target"]);
            current_block = 1836292691772056875;
        } else if cmd_find_valid_state(&queue_current) != 0 {
            current_state = Some(&queue_current);
            log_debug(c"%s: current is from queue", fmt_args![c"cmd_find_target"]);
            current_block = 1836292691772056875;
        } else if cmd_find_from_client(&mut current, item.client().as_ref(), flags)
            == 0 as core::ffi::c_int
        {
            current_state = Some(&current);
            log_debug(c"%s: current is from client", fmt_args![c"cmd_find_target"]);
            current_block = 1836292691772056875;
        } else {
            if !flags & CMD_FIND_QUIET != 0 {
                item.error(c"no current target", fmt_args![]);
            }
            current_block = 5874756215481722497;
        }
        if current_block == 1836292691772056875 {
            let current_state = current_state.expect("a current target was selected");
            if cmd_find_valid_state(current_state) == 0 {
                fatalx(c"invalid current find state", fmt_args![]);
            }
            let given = target.filter(|target| !target.to_bytes().is_empty());
            if given.is_none() {
                current_block = 1976276580247140074;
            } else if given == Some(c"@")
                || given == Some(c"{active}")
                || given == Some(c"{current}")
            {
                if let Some(c) = item.client() {
                    let session = c.attached_session().expect("the client has a session");
                    let link = session.curw().expect("the session has a current link");
                    fs.set_winlink(link.get());
                    let window = link.window().expect("a session link has a window");
                    fs.wp = window.active_pane();
                    fs.set_window_ref(Some(&window));
                    current_block = 8711307700714518445;
                } else {
                    item.error(c"no current client", fmt_args![]);
                    current_block = 5874756215481722497;
                }
            } else if given == Some(c"=") || given == Some(c"{mouse}") {
                let event = item.event().clone();
                let m = &event.m;
                let current_block_56: u64;
                match type_0 {
                    CMD_FIND_PANE => {
                        if let Some((session, link, pane)) = cmd_mouse_pane(m) {
                            fs.set_session_ref(Some(&session));
                            fs.wl = Some(link.index());
                            fs.set_window_ref(pane.window().as_ref());
                            fs.wp = Some(pane.clone());
                            current_block_56 = 7343950298149844727;
                        } else {
                            fs.wp = None;
                            current_block_56 = 1142519184231123645;
                        }
                    }
                    CMD_FIND_WINDOW | CMD_FIND_SESSION => {
                        current_block_56 = 1142519184231123645;
                    }
                    _ => {
                        current_block_56 = 7343950298149844727;
                    }
                }
                if current_block_56 == 1142519184231123645 {
                    if let Some((session, link)) = cmd_mouse_window(m) {
                        fs.set_session_ref(Some(&session));
                        fs.wl = link.map(|link| link.index());
                    } else {
                        fs.wl = None;
                    }
                    if let Some(session) = fs.session() {
                        let s = session.as_session();
                        if fs.wl.is_none() {
                            fs.wl = s.curw.filter(|index| s.windows.contains_key(index));
                        }
                        if let Some(link) = fs.wl.and_then(|index| s.windows.get(&index)) {
                            fs.set_window_ref(link.window_handle());
                            fs.wp = link.window_handle().and_then(|owner| {
                                owner
                                    .active_pane()
                                    .filter(|target| owner.panes().contains(target))
                            });
                        }
                    }
                }
                if fs.wp.is_none() {
                    if !flags & CMD_FIND_QUIET != 0 {
                        item.error(c"no mouse target", fmt_args![]);
                    }
                    current_block = 5874756215481722497;
                } else {
                    current_block = 8711307700714518445;
                }
            } else if given == Some(c"~") || given == Some(c"{marked}") {
                if server_check_marked() == 0 {
                    if !flags & CMD_FIND_QUIET != 0 {
                        item.error(c"no marked target", fmt_args![]);
                    }
                    current_block = 5874756215481722497;
                } else {
                    cmd_find_copy_state(fs, &marked_pane.get());
                    current_block = 8711307700714518445;
                }
            } else if let Some(given) = given {
                let mut copy = given.to_bytes_with_nul().to_vec();
                let colon = copy.iter().position(|&byte| byte == b':');
                let suffix = colon.map_or(0, |at| at + 1);
                let period = copy[suffix..]
                    .iter()
                    .position(|&byte| byte == b'.')
                    .map(|at| suffix + at);
                if let Some(at) = colon {
                    copy[at] = 0;
                }
                if let Some(at) = period {
                    copy[at] = 0;
                }
                let part = |at: usize| {
                    CStr::from_bytes_until_nul(&copy[at..])
                        .expect("target components retain a terminator")
                };
                let first = part(0);
                let (mut session, mut window, mut pane) = match (colon, period) {
                    (Some(colon), Some(period)) => {
                        (Some(first), Some(part(colon + 1)), Some(part(period + 1)))
                    }
                    (Some(colon), None) => (Some(first), Some(part(colon + 1)), None),
                    (None, Some(period)) => (None, Some(first), Some(part(period + 1))),
                    (None, None) => match first.to_bytes().first() {
                        Some(b'$') => (Some(first), None, None),
                        Some(b'@') => (None, Some(first), None),
                        Some(b'%') => (None, None, Some(first)),
                        _ => match type_0 {
                            CMD_FIND_SESSION => (Some(first), None, None),
                            CMD_FIND_WINDOW => (None, Some(first), None),
                            CMD_FIND_PANE => (None, None, Some(first)),
                            _ => (None, None, None),
                        },
                    },
                };
                window_only = core::ffi::c_int::from(colon.is_some());
                pane_only = core::ffi::c_int::from(period.is_some());
                if session.is_some_and(|name| name.to_bytes().starts_with(b"=")) {
                    session = session.map(|name| cstr_tail(name, 1));
                    fs.flags |= CMD_FIND_EXACT_SESSION;
                }
                if window.is_some_and(|name| name.to_bytes().starts_with(b"=")) {
                    window = window.map(|name| cstr_tail(name, 1));
                    fs.flags |= CMD_FIND_EXACT_WINDOW;
                }
                session = session
                    .filter(|name| !name.is_empty())
                    .map(|name| cmd_find_map_table(cmd_find_session_table, name));
                window = window
                    .filter(|name| !name.is_empty())
                    .map(|name| cmd_find_map_table(cmd_find_window_table, name));
                pane = pane
                    .filter(|name| !name.is_empty())
                    .map(|name| cmd_find_map_table(cmd_find_pane_table, name));
                if session.is_some() || window.is_some() || pane.is_some() {
                    log_debug(
                        c"%s: target %s is %s%s%s%s%s%s",
                        fmt_args![
                            c"cmd_find_target",
                            target,
                            if session.is_some() { c"session " } else { c"" },
                            session.unwrap_or(c""),
                            if window.is_some() { c"window " } else { c"" },
                            window.unwrap_or(c""),
                            if pane.is_some() { c"pane " } else { c"" },
                            pane.unwrap_or(c"")
                        ],
                    );
                }
                if pane.is_some() && flags & CMD_FIND_WINDOW_INDEX != 0 {
                    if !flags & CMD_FIND_QUIET != 0 {
                        item.error(c"can't specify pane here", fmt_args![]);
                    }
                    current_block = 5874756215481722497;
                } else {
                    if let Some(session) = session {
                        if cmd_find_get_session(fs, session) != 0 as core::ffi::c_int {
                            if !flags & CMD_FIND_QUIET != 0 {
                                item.error(c"can't find session: %s", fmt_args![session]);
                            }
                            current_block = 5874756215481722497;
                        } else if window.is_none() && pane.is_none() {
                            let session = fs.session().expect("the state names a session");
                            let link = session.curw().expect("the session has a current link");
                            fs.set_winlink(link.get());
                            fs.idx = -1;
                            let window = link.window().expect("a session link has a window");
                            fs.set_window_ref(Some(&window));
                            fs.wp = window.active_pane();
                            current_block = 8711307700714518445;
                        } else if let (Some(window), None) = (window, pane) {
                            if cmd_find_get_window_with_session(fs, window) != 0 as core::ffi::c_int
                            {
                                current_block = 8113074638138919487;
                            } else {
                                if let Some(link) = fs.winlink_ref() {
                                    let window = link
                                        .get()
                                        .expect("the selected link is present")
                                        .window_handle()
                                        .expect("a session link has a window");
                                    fs.wp = window.active_pane();
                                }
                                current_block = 8711307700714518445;
                            }
                        } else if let (None, Some(pane)) = (window, pane) {
                            if cmd_find_get_pane_with_session(fs, pane) != 0 as core::ffi::c_int {
                                current_block = 251766546907006956;
                            } else {
                                current_block = 8711307700714518445;
                            }
                        } else if cmd_find_get_window_with_session(
                            fs,
                            window.expect("window target is present"),
                        ) != 0 as core::ffi::c_int
                        {
                            current_block = 8113074638138919487;
                        } else if cmd_find_get_pane_with_window(
                            fs,
                            pane.expect("pane target is present"),
                        ) != 0 as core::ffi::c_int
                        {
                            current_block = 251766546907006956;
                        } else {
                            current_block = 8711307700714518445;
                        }
                    } else if let (Some(window), Some(pane)) = (window, pane) {
                        if cmd_find_get_window(current_state, fs, window, window_only)
                            != 0 as core::ffi::c_int
                        {
                            current_block = 8113074638138919487;
                        } else if cmd_find_get_pane_with_window(fs, pane) != 0 as core::ffi::c_int {
                            current_block = 251766546907006956;
                        } else {
                            current_block = 8711307700714518445;
                        }
                    } else if let (Some(window), None) = (window, pane) {
                        if cmd_find_get_window(current_state, fs, window, window_only)
                            != 0 as core::ffi::c_int
                        {
                            current_block = 8113074638138919487;
                        } else {
                            if let Some(link) = fs.winlink_ref() {
                                let window = link
                                    .get()
                                    .expect("the selected link is present")
                                    .window_handle()
                                    .expect("a session link has a window");
                                fs.wp = window.active_pane();
                            }
                            current_block = 8711307700714518445;
                        }
                    } else if let (None, Some(pane)) = (window, pane) {
                        if cmd_find_get_pane(current_state, fs, pane, pane_only)
                            != 0 as core::ffi::c_int
                        {
                            current_block = 251766546907006956;
                        } else {
                            current_block = 8711307700714518445;
                        }
                    } else {
                        current_block = 1976276580247140074;
                    }
                    match current_block {
                        5874756215481722497 => {}
                        8711307700714518445 => {}
                        1976276580247140074 => {}
                        _ => {
                            match current_block {
                                8113074638138919487 => {
                                    if !flags & CMD_FIND_QUIET != 0 {
                                        item.error(c"can't find window: %s", fmt_args![window]);
                                    }
                                }
                                _ => {
                                    if !flags & CMD_FIND_QUIET != 0 {
                                        item.error(c"can't find pane: %s", fmt_args![pane]);
                                    }
                                }
                            }
                            current_block = 5874756215481722497;
                        }
                    }
                }
            }
            match current_block {
                5874756215481722497 => {}
                _ => {
                    if current_block == 1976276580247140074 {
                        cmd_find_copy_state(fs, current_state);
                        if flags & CMD_FIND_WINDOW_INDEX != 0 {
                            fs.idx = -(1 as core::ffi::c_int);
                        }
                    }
                    cmd_find_log_state(c"cmd_find_target", fs);
                    return 0 as core::ffi::c_int;
                }
            }
        }
        log_debug(c"%s: error", fmt_args![c"cmd_find_target"]);
        if flags & CMD_FIND_CANFAIL != 0 {
            return 0 as core::ffi::c_int;
        }
        -(1 as core::ffi::c_int)
    }
}
unsafe fn cmd_find_current_client(
    item: Option<&cmdq_item>,
    quiet: core::ffi::c_int,
) -> Option<ClientRef> {
    unsafe {
        let mut fs = cmd_find_state::default();
        let c = item.and_then(cmdq_item::client);
        if c.as_ref().is_some_and(|c| !c.attached_session().is_none()) {
            return c;
        }
        let pane = cmd_find_inside_pane(c.as_ref());
        let found = if let Some(pane) = pane {
            cmd_find_clear_state(&mut fs, CMD_FIND_QUIET);
            fs.set_window_ref(pane.window().as_ref());
            if cmd_find_best_session_with_window(&mut fs) == 0 as core::ffi::c_int {
                fs.session()
                    .as_ref()
                    .map(|reference| reference.as_session())
                    .and_then(|s| cmd_find_best_client(s))
            } else {
                None
            }
        } else {
            cmd_find_best_session(&[], CMD_FIND_QUIET)
                .as_ref()
                .map(|reference| reference.as_session())
                .and_then(|s| cmd_find_best_client(s))
        };
        if found.is_none()
            && quiet == 0
            && let Some(item) = item
        {
            item.error(c"no current client", fmt_args![]);
        }
        log_debug(
            c"%s: no target, return %p",
            fmt_args![
                c"cmd_find_current_client".as_ptr(),
                found
                    .as_ref()
                    .map_or(core::ptr::null_mut(), ClientRef::as_ptr)
            ],
        );
        found
    }
}
pub unsafe fn cmd_find_client(
    item: Option<&cmdq_item>,
    target: Option<&CStr>,
    quiet: core::ffi::c_int,
) -> Option<ClientRef> {
    unsafe {
        let Some(target) = target else {
            return cmd_find_current_client(item, quiet);
        };
        let target_bytes = target.to_bytes();
        let target_bytes = target_bytes.strip_suffix(b":").unwrap_or(target_bytes);
        let found = with_clients(|clients| {
            let mut found = None;
            for candidate in clients {
                if candidate.attached_session().is_none() {
                    continue;
                }
                let view = candidate;
                if view
                    .name()
                    .is_some_and(|name| name.to_bytes() == target_bytes)
                {
                    found = Some(candidate);
                    break;
                }
                let ttyname = view
                    .ttyname_ref()
                    .as_deref()
                    .expect("an attached client has a terminal name")
                    .to_bytes();
                if ttyname.is_empty() {
                    continue;
                }
                if ttyname == target_bytes
                    || ttyname.strip_prefix(_PATH_DEV.to_bytes()) == Some(target_bytes)
                {
                    found = Some(candidate);
                    break;
                }
            }
            found.cloned()
        });
        if found.is_none()
            && quiet == 0
            && let Some(item) = item
        {
            item.error(c"can't find client: %s", fmt_args![target_bytes]);
        }
        log_debug(
            c"%s: target %s, return %p",
            fmt_args![
                c"cmd_find_client".as_ptr(),
                target,
                found
                    .as_ref()
                    .map_or(core::ptr::null_mut(), ClientRef::as_ptr)
            ],
        );
        found
    }
}

#[cfg(test)]
#[path = "find_focused_tests.rs"]
mod focused_tests;

/// Builds the current target from a retained session using existing target rules.
///
/// # Safety
/// The session must have a current live link and pane. Exclude conflicting session,
/// window and target access. This synchronous operation dispatches no callbacks.
pub(crate) unsafe fn cmd_find_from_session_ref(
    fs: &mut cmd_find_state,
    session: &SessionRef,
    flags: core::ffi::c_int,
) {
    unsafe { cmd_find_from_session(fs, session.as_session(), flags) };
}

/// Builds a link target, optionally naming a specific existing pane allocation.
/// An absent pane selects the link's active pane with the existing index semantics.
///
/// # Safety
/// The link must be present, and a supplied pane must be live and belong to its
/// window. Exclude conflicting session/window/pane/target access. No callbacks run.
pub(crate) unsafe fn cmd_find_from_link_ref(
    fs: &mut cmd_find_state,
    link: &crate::window::WinlinkRef,
    pane: Option<&RustWindowPaneWeak>,
    flags: core::ffi::c_int,
) {
    unsafe {
        let link = link.get().expect("the target window link is present");
        match pane {
            Some(pane) => cmd_find_from_winlink_pane(
                fs,
                link,
                pane.get().expect("the target pane is present"),
                flags,
            ),
            None => cmd_find_from_winlink(fs, link, flags),
        }
    }
}
