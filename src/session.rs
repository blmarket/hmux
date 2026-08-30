//! Sessions: the server's tree of them, the groups that hold sessions carrying
//! the same windows, and the walks between the windows a session is linked to.
//!
//! A session owns its name, its working directory, the terminal settings a
//! client attached with, an environment and an option set; the windows are
//! not its own, and reach it through the `winlink`s in its tree. A session is
//! given up by reference count, and the last reference going hands it to the
//! event loop rather than freeing it there and then, since what let it go is
//! often still walking it.
//!
//! Both trees are `BTreeMap`s keyed by name, which is the order the C's
//! `tree.h` comparison put them in, and a session's windows are one more,
//! keyed by index.
//!
//! Coverage exemptions: the `fatal` arm for a clock that would not answer and
//! the `fatalx` arm for a session the sorted list does not hold, which end the
//! process; and the `server_clear_marked` guard in `session_renumber_windows`,
//! which cannot be reached and says so where it sits. Everything else is
//! covered by the tests below and by `test_coverage_alpha`.
use crate::compat::strtonum;
use crate::environ::{environ_ptr, environ_t};
use crate::ffi::gettimeofday;
use crate::fmt_args;
use crate::grid::grid_collect_history;
use crate::log::{fatal, fatalx, log_debug};
use crate::notify::{notify_session, notify_session_window};
use crate::options::{options_free, options_get_number, options_ptr};
use crate::reactor;
use crate::reactor::{Reactor, Timer};
use crate::resize::recalculate_sizes;
use crate::screen::screen_grid_ptr;
use crate::server::server_lock_session;
use crate::server::{marked_pane, server_clear_marked};
use crate::sort::sort_get_sessions;
use crate::status::status_update_cache;
use crate::tmux::global_options;
use crate::tree::GlobalTree;
use crate::tty::tty_update_window_offset;
pub use crate::types::*;
use crate::window::winlinks_into;
use crate::window::{
    window_panes_first, window_panes_next, window_update_activity, window_update_focus,
    winlink_add, winlink_clear_flags, winlink_find_by_index, winlink_find_by_window,
    winlink_find_by_window_id, winlink_remove, winlink_set_window, winlink_stack_push,
    winlink_stack_remove, winlinks_after, winlinks_before, winlinks_first, winlinks_last,
};
use crate::xmalloc::xasprintf;
use ::core::ffi::{CStr, c_char, c_int, c_longlong, c_void};
use ::core::iter::successors;
use ::core::ops::Bound;
use ::core::ptr::null_mut;
use ::std::ffi::CString;
pub const SORT_NAME: sort_order = 4;
pub const UINT_MAX: u_int = u_int::MAX;
pub const RB_BLACK: c_int = 0;
pub const RB_RED: c_int = 1;
pub const RB_NEGINF: c_int = -1;
pub const RB_INF: c_int = 1;
pub const EV_TIMEOUT: c_int = 0x1;
pub const PANE_THEMECHANGED: c_int = 0x2000;
pub const WINLINK_BELL: c_int = 0x1;
pub const WINLINK_ACTIVITY: c_int = 0x2;
pub const WINLINK_SILENCE: c_int = 0x4;
pub const WINLINK_ALERTFLAGS: c_int = WINLINK_BELL | WINLINK_ACTIVITY | WINLINK_SILENCE;
pub const WINLINK_VISITED: c_int = 0x8;

/// Every session the server holds, by name, which is what holds them alive.
pub(crate) static sessions: GlobalTree<CString, SessionRef> = GlobalTree::new();

static SESSION_HANDLES: GlobalTree<usize, SessionWeak> = GlobalTree::new();

/// The id the next session is given, which is never handed out twice.
pub static mut next_session_id: u_int = 0;

/// The sessions of a group, in the order they joined it, observed rather than
/// held: a session belongs to the session tree, not to its group.
type session_group_sessions = Vec<SessionWeak>;

/// A set of sessions carrying the same windows, so that a window linked into
/// one of them is linked into all of them.
///
/// Both fields are the group's own: a caller outside this module names a group
/// by `session_group_name` and walks its members through `group_walk`, and
/// changes what it holds through `session_group_add` and the synchronize
/// calls, which is what keeps the group and its sessions in step.
#[derive(Default)]
#[repr(C)]
pub struct session_group {
    name: Option<CString>,
    sessions: session_group_sessions,
}

/// Every session group the server holds, by name.
pub type session_groups_t = ::std::collections::BTreeMap<CString, Box<session_group>>;

/// Every session group the server holds, by name.
pub static session_groups: GlobalTree<CString, Box<session_group>> = GlobalTree::new();

pub(crate) fn register_session_handle(reference: &SessionRef) {
    SESSION_HANDLES
        .map()
        .insert(reference.as_ptr() as usize, reference.downgrade());
}

pub(crate) fn unregister_session_handle(s: *mut session) {
    SESSION_HANDLES.map().remove(&(s as usize));
}

pub(crate) fn session_ref_from_ptr(s: *mut session) -> Option<SessionRef> {
    if s.is_null() {
        return None;
    }
    let key = s as usize;
    let reference = SESSION_HANDLES.map().get(&key).cloned()?.upgrade();
    if reference
        .as_ref()
        .is_some_and(|reference| reference.as_ptr() == s)
    {
        return reference;
    }
    SESSION_HANDLES.map().remove(&key);
    None
}

pub(crate) fn session_registry_remove(s: *mut session) -> Option<SessionRef> {
    unsafe { sessions.map().remove(name_of(cstr_ptr(&(*s).name))) }
}

pub(crate) fn session_registry_clear() {
    sessions.map().clear();
    SESSION_HANDLES.map().clear();
}

impl Drop for SessionStorage {
    fn drop(&mut self) {
        let s = &mut self.value;
        s.lock_timer.disarm();
        s.environ = None;
        if let Some(oo) = s.options.take() {
            unsafe { options_free(oo) };
        }
        s.name = None;
        s.cwd = None;
        s.tio = None;
        unregister_session_handle(s);
    }
}

/// What one of the trees' or lists' walks answered, as nothing once it has run
/// out.
fn walked<T>(p: *mut T) -> Option<*mut T> {
    (!p.is_null()).then_some(p)
}

/// The name a pointer to a C string carries.
unsafe fn name_of(p: *const c_char) -> &'static CStr {
    unsafe { CStr::from_ptr(p) }
}

/// One session: what it is called, where its panes start, the windows it is
/// linked to and the clients attached to it.
///
/// The fields below the divider are the session's own; a caller outside this
/// module reads them through the `session_*` accessors and changes them
/// through the named operations, so that the registry the session is filed
/// under and the session's own name cannot drift apart.
///
/// The fields above the divider are still reached directly: `windows`,
/// `lastw` and `curw_idx` are the link collection, which becomes the
/// session's own once the `winlink_*` free functions in window.rs are fronted
/// as session methods; `statusat`/`statuslines` are the status line's cache
/// of what it worked out; and the times and lock timer are the server's.
#[derive(Default)]
#[repr(C)]
pub struct session {
    pub creation_time: timeval,
    pub last_attached_time: timeval,
    pub last_activity_time: timeval,
    pub lock_timer: TimerHandle,
    /// The index of the link the session is showing, or nothing while it
    /// shows none. A link is named by its index in `windows` and nothing
    /// else, so the session never holds one it has given up.
    pub curw_idx: Option<::core::ffi::c_int>,
    pub lastw: winlink_stack,
    pub windows: winlinks,
    pub statusat: c_int,
    pub statuslines: u_int,
    // ---
    id: u_int,
    name: Option<CString>,
    cwd: Option<CString>,
    activity_time: timeval,
    options: Option<Box<options>>,
    flags: c_int,
    attached: u_int,
    tio: Option<termios>,
    environ: Option<Box<environ_t>>,
}

/// Set on a session an alert has already been reported for during the current
/// check, which is what keeps one bell from being reported twice.
const SESSION_ALERTED: c_int = 0x1;

/// The name the session is filed under.
pub unsafe fn session_name(s: *const session) -> *const c_char {
    unsafe { cstr_ptr(&(*s).name) }
}

/// The name the session is filed under, as a copy of its own, for a caller
/// that keeps it after the session is gone.
pub unsafe fn session_name_owned(s: *const session) -> Option<CString> {
    unsafe { (*s).name.clone() }
}

/// A session of `name` that is not in the server's registry and has no
/// windows, for a test that wants one to hand to the function under test
/// rather than one the server will run.
#[cfg(test)]
pub(crate) fn session_new_detached(
    id: u_int,
    name: CString,
    cwd: CString,
    oo: Box<options>,
    env: Box<environ_t>,
) -> SessionRef {
    SessionRef::new(session {
        id,
        name: Some(name),
        cwd: Some(cwd),
        options: Some(oo),
        environ: Some(env),
        ..session::default()
    })
}

/// The id the session was given, which is what `$0` names it by.
pub unsafe fn session_id(s: *const session) -> u_int {
    unsafe { (*s).id }
}

/// The directory a pane started in the session begins in.
pub unsafe fn session_cwd(s: *const session) -> *const c_char {
    unsafe { cstr_ptr(&(*s).cwd) }
}

/// The session's options, which every option lookup for one of its windows or
/// panes falls back through.
pub unsafe fn session_options(s: *const session) -> *mut options {
    unsafe { options_ptr(&(*s).options) }
}

/// The environment a pane started in the session is given.
pub unsafe fn session_environ(s: *const session) -> *mut environ_t {
    unsafe { environ_ptr(&(*s).environ) }
}

/// The terminal settings the first client attached with, if it had any.
pub unsafe fn session_tio(s: *const session) -> Option<&'static termios> {
    unsafe { (*s).tio.as_ref() }
}

/// How many clients are attached to the session.
pub unsafe fn session_attached(s: *const session) -> u_int {
    unsafe { (*s).attached }
}

/// When the session was last active, which is what the activity sort orders
/// sessions by.
pub unsafe fn session_activity_time(s: *const session) -> timeval {
    unsafe { (*s).activity_time }
}

/// Says when the session was last active. The server goes through
/// `session_update_activity`, which also resets the lock timer.
pub unsafe fn session_set_activity_time(s: *mut session, at: timeval) {
    unsafe { (*s).activity_time = at };
}

/// Whether an alert has already been reported for the session during the
/// current check.
pub unsafe fn session_alerted(s: *const session) -> bool {
    unsafe { (*s).flags & SESSION_ALERTED != 0 }
}

/// Says whether an alert has been reported for the session yet.
pub unsafe fn session_set_alerted(s: *mut session, alerted: bool) {
    unsafe {
        if alerted {
            (*s).flags |= SESSION_ALERTED;
        } else {
            (*s).flags &= !SESSION_ALERTED;
        }
    }
}

/// Starts the count of attached clients again, which is where
/// `recalculate_sizes` begins its recount.
pub unsafe fn session_clear_attached(s: *mut session) {
    unsafe { (*s).attached = 0 };
}

/// Counts one more attached client against the session.
pub unsafe fn session_add_attached(s: *mut session) {
    unsafe { (*s).attached = (*s).attached.wrapping_add(1) };
}

/// Where panes started in the session begin from now on.
pub unsafe fn session_set_cwd(s: *mut session, cwd: CString) {
    unsafe { (*s).cwd = Some(cwd) };
}

/// Files the session under `name`, taking it out of the registry under the
/// old one and putting it back under the new, which is the whole of what a
/// rename is.
pub unsafe fn session_rename(s: *mut session, name: CString) {
    unsafe {
        let held = session_registry_remove(s);
        (*s).name = Some(name.clone());
        if let Some(held) = held {
            sessions.map().insert(name, held);
        }
    }
}

/// The link the session is showing, or null while it shows none. The index
/// is looked up among the session's own links, so one it has given up is not
/// answered with.
pub unsafe fn session_get_curw(s: *mut session) -> *mut winlink {
    unsafe { winlink_of(s, (*s).curw_idx) }
}

/// Makes `wl` the link the session shows, or gives up showing one when it is
/// null.
pub unsafe fn session_set_curw(s: *mut session, wl: *mut winlink) {
    unsafe {
        (*s).curw_idx = wl.as_ref().map(|wl| wl.idx);
    }
}

/// The link `idx` names among `s`'s own links, or null when there is no such
/// session or it holds no such link.
pub(crate) fn winlink_of(s: *mut session, idx: Option<::core::ffi::c_int>) -> *mut winlink {
    unsafe {
        let Some(idx) = idx else {
            return null_mut();
        };
        if s.is_null() {
            return null_mut();
        }
        (*s).windows
            .get(&idx)
            .map(|wl| &raw const **wl as *mut winlink)
            .unwrap_or(null_mut())
    }
}

/// The first session the server holds, in name order.
pub fn sessions_first() -> *mut session {
    sessions
        .map()
        .values()
        .next()
        .map(SessionRef::as_ptr)
        .unwrap_or(null_mut::<session>())
}

/// The session after `s`, in name order.
pub unsafe fn sessions_after(s: *mut session) -> *mut session {
    unsafe {
        sessions
            .map()
            .range::<CStr, _>((
                Bound::Excluded(name_of(cstr_ptr(&(*s).name))),
                Bound::Unbounded,
            ))
            .next()
            .map(|(_, s)| s.as_ptr())
            .unwrap_or(null_mut::<session>())
    }
}

/// The first session group the server holds, in name order.
pub fn session_groups_first() -> *mut session_group {
    session_groups
        .map()
        .values()
        .next()
        .map(|sg| &raw const **sg as *mut session_group)
        .unwrap_or(null_mut::<session_group>())
}

/// The session group after `sg`, in name order.
pub unsafe fn session_groups_after(sg: *mut session_group) -> *mut session_group {
    unsafe {
        session_groups
            .map()
            .range::<CStr, _>((
                Bound::Excluded(name_of(cstr_ptr(&(*sg).name))),
                Bound::Unbounded,
            ))
            .next()
            .map(|(_, sg)| &raw const **sg as *mut session_group)
            .unwrap_or(null_mut::<session_group>())
    }
}

/// Every session the server holds, in name order.
fn each_session() -> impl Iterator<Item = *mut session> {
    let all: Vec<*mut session> = sessions.map().values().map(SessionRef::as_ptr).collect();
    all.into_iter()
}

/// Every session group the server holds, in name order.
fn each_group() -> impl Iterator<Item = *mut session_group> {
    let all: Vec<*mut session_group> = session_groups
        .map()
        .values()
        .map(|sg| &raw const **sg as *mut session_group)
        .collect();
    all.into_iter()
}

/// The sessions of a group, in the order they joined it, walked the way the
/// C's `TAILQ_FOREACH` walked them.
unsafe fn sessions_of(sg: *mut session_group) -> impl Iterator<Item = *mut session> {
    unsafe { group_walk(sg) }
}

/// The sessions of `sg`, in the order they joined it. Each one is read out of
/// the group again as the walk goes, so a session lost while it runs is not
/// walked into.
pub(crate) unsafe fn group_walk(sg: *mut session_group) -> impl Iterator<Item = *mut session> {
    unsafe {
        let all: Vec<*mut session> = (*sg)
            .sessions
            .iter()
            .filter_map(SessionWeak::upgrade)
            .map(|s| s.as_ptr())
            .collect();
        all.into_iter()
    }
}

/// The windows linked into a session, in index order.
unsafe fn winlinks_of(ww: *mut winlinks) -> impl Iterator<Item = *mut winlink> {
    let all: Vec<*mut winlink> = unsafe {
        (*ww)
            .values()
            .map(|wl| wl.as_ref() as *const winlink as *mut winlink)
            .collect()
    };
    all.into_iter()
}

/// The sessions a window is linked into, through the list the window carries.
unsafe fn winlinks_on(w: *mut window) -> impl Iterator<Item = *mut winlink> {
    unsafe { winlinks_into(w) }
}

/// The panes of a window, in the order it carries them.
unsafe fn panes_of(w: *mut window) -> impl Iterator<Item = *mut window_pane> {
    successors(walked(unsafe { window_panes_first(w) }), move |wp| {
        walked(unsafe { window_panes_next(w, *wp) })
    })
}

/// The session of `name` in `head`.
unsafe fn session_of_name(head: *mut sessions_t, name: &CStr) -> *mut session {
    unsafe {
        (*head)
            .get(name)
            .map(SessionRef::as_ptr)
            .unwrap_or(null_mut::<session>())
    }
}

/// The same, over the group tree.
unsafe fn group_of_name(head: *mut session_groups_t, name: &CStr) -> *mut session_group {
    unsafe {
        (*head)
            .get(name)
            .map(|sg| &raw const **sg as *mut session_group)
            .unwrap_or(null_mut::<session_group>())
    }
}

pub unsafe fn session_alive(s: *mut session) -> c_int {
    each_session().any(|s_loop| s_loop == s) as c_int
}

pub unsafe fn session_find(name: *const c_char) -> *mut session {
    unsafe { session_of_name(sessions.map(), name_of(name)) }
}

pub unsafe fn session_find_by_id_str(s: *const c_char) -> *mut session {
    unsafe {
        if *s as c_int != '$' as c_int {
            return null_mut::<session>();
        }
        let Ok(id) = strtonum(s.add(1), 0 as c_longlong, UINT_MAX as c_longlong) else {
            return null_mut::<session>();
        };
        session_find_by_id(id as u_int)
    }
}

pub fn session_find_by_id(id: u_int) -> *mut session {
    unsafe {
        each_session()
            .find(|s| (**s).id == id)
            .unwrap_or(null_mut::<session>())
    }
}

/// Creates a session of `name`, or one named after `prefix` and the id it is
/// given when there is no name. The session takes over `env` and `oo`, and
/// copies the terminal settings so the caller keeps its own.
pub unsafe fn session_create(
    prefix: *const c_char,
    name: *const c_char,
    cwd: *const c_char,
    env: Box<environ_t>,
    oo: Box<options>,
    tio: *mut termios,
) -> *mut session {
    unsafe {
        let reference = SessionRef::new(session {
            cwd: Some(CStr::from_ptr(cwd).to_owned()),
            options: Some(oo),
            environ: Some(env),
            tio: tio.as_ref().copied(),
            ..session::default()
        });
        let s = reference.as_ptr();
        status_update_cache(s);
        if !name.is_null() {
            (*s).name = Some(CStr::from_ptr(name).to_owned());
            (*s).id = next_session_id;
            next_session_id = next_session_id.wrapping_add(1);
        } else {
            loop {
                (*s).id = next_session_id;
                next_session_id = next_session_id.wrapping_add(1);
                (*s).name = Some(if prefix.is_null() {
                    xasprintf(c"%u".as_ptr(), fmt_args![(*s).id])
                } else {
                    xasprintf(c"%s-%u".as_ptr(), fmt_args![prefix, (*s).id])
                });
                if session_of_name(sessions.map(), name_of(cstr_ptr(&(*s).name))).is_null() {
                    break;
                }
            }
        }
        let session_name = name_of(cstr_ptr(&(*s).name)).to_owned();
        sessions.map().insert(session_name, reference);
        log_debug(
            c"new session %s $%u".as_ptr(),
            fmt_args![cstr_ptr(&(*s).name), (*s).id],
        );
        if gettimeofday(&raw mut (*s).creation_time, null_mut::<c_void>()) != 0 {
            fatal(c"gettimeofday failed".as_ptr(), fmt_args![]);
        }
        session_update_activity(s, &raw mut (*s).creation_time);
        s
    }
}

fn session_defer_cleanup(reference: SessionRef) {
    reactor::current().defer(move || unsafe {
        let s = reference.as_ptr();
        log_debug(
            c"session %s freed".as_ptr(),
            fmt_args![cstr_ptr(&(*s).name)],
        );
    });
}

pub unsafe fn session_destroy(s: *mut session, notify: c_int, from: *const c_char) {
    unsafe {
        let session_ref = session_ref_from_ptr(s);
        log_debug(
            c"session %s destroyed (%s)".as_ptr(),
            fmt_args![cstr_ptr(&(*s).name), from],
        );
        if (*s).curw_idx.is_none() {
            return;
        }
        (*s).curw_idx = None;
        let session_name = name_of(cstr_ptr(&(*s).name)).to_owned();
        sessions.map().remove(&session_name);
        if notify != 0 {
            notify_session(c"session-closed".as_ptr(), s);
        }
        (*s).tio = None;
        (*s).lock_timer.disarm();
        session_group_remove(s);
        while let Some(idx) = (*s).lastw.first().copied() {
            let wl = winlink_of(s, Some(idx));
            if wl.is_null() {
                (*s).lastw.remove(0);
                continue;
            }
            winlink_stack_remove(&raw mut (*s).lastw, wl);
        }
        while let Some(wl) = (*s)
            .windows
            .values()
            .next()
            .map(|wl| wl.as_ref() as *const winlink as *mut winlink)
        {
            notify_session_window(c"window-unlinked".as_ptr(), s, (*wl).window());
            winlink_remove(&raw mut (*s).windows, wl);
        }
        (*s).cwd = None;
        if let Some(session_ref) = session_ref {
            session_defer_cleanup(session_ref);
        }
    }
}

unsafe fn session_lock_timer(s: *mut session) {
    unsafe {
        if (*s).attached == 0 {
            return;
        }
        log_debug(
            c"session %s locked, activity time %lld".as_ptr(),
            fmt_args![
                cstr_ptr(&(*s).name),
                (*s).activity_time.tv_sec as c_longlong
            ],
        );
        server_lock_session(s);
        recalculate_sizes();
    }
}

/// Records that something happened in `s` at `from`, or now when there is no
/// time, and arms the lock timer when somebody is attached and the session has
/// a `lock-after-time`.
pub unsafe fn session_update_activity(s: *mut session, from: *mut timeval) {
    unsafe {
        if from.is_null() {
            gettimeofday(&raw mut (*s).activity_time, null_mut::<c_void>());
        } else {
            session_set_activity_time(s, *from);
        }
        log_debug(
            c"session $%u %s activity %lld.%06d".as_ptr(),
            fmt_args![
                (*s).id,
                cstr_ptr(&(*s).name),
                (*s).activity_time.tv_sec as c_longlong,
                (*s).activity_time.tv_usec as c_int
            ],
        );
        if (*s).lock_timer.is_set() {
            (*s).lock_timer.disarm();
        } else {
            (*s).lock_timer.set_callback(move || session_lock_timer(s));
        }
        if (*s).attached != 0 {
            let mut tv = timeval {
                tv_sec: options_get_number(options_ptr(&(*s).options), c"lock-after-time".as_ptr())
                    as __time_t,
                tv_usec: 0 as __suseconds_t,
            };
            if tv.tv_sec != 0 {
                (*s).lock_timer.arm(tv);
            }
        }
    }
}

/// The sorted list of sessions and where `s` sits in it, or nothing at all
/// when the server holds no sessions or has given `s` up.
unsafe fn session_in_sorted_order(
    s: *mut session,
    sort_crit: &sort_criteria_t,
) -> Option<(Vec<*mut session>, usize)> {
    unsafe {
        if sessions.map().is_empty() || session_alive(s) == 0 {
            return None;
        }
        let list = sort_get_sessions(sort_crit);
        match list.iter().position(|held| *held == s) {
            Some(i) => Some((list, i)),
            None => fatalx(
                c"session %s not found in sorted list".as_ptr(),
                fmt_args![cstr_ptr(&(*s).name)],
            ),
        }
    }
}

pub unsafe fn session_next_session(s: *mut session, sort_crit: &sort_criteria_t) -> *mut session {
    unsafe {
        match session_in_sorted_order(s, sort_crit) {
            None => null_mut::<session>(),
            Some((list, i)) => list[(i + 1) % list.len()],
        }
    }
}

pub unsafe fn session_previous_session(
    s: *mut session,
    sort_crit: &sort_criteria_t,
) -> *mut session {
    unsafe {
        match session_in_sorted_order(s, sort_crit) {
            None => null_mut::<session>(),
            Some((list, i)) => list[(i + list.len() - 1) % list.len()],
        }
    }
}

/// Links `w` into `s` at `idx`, or answers nothing and the reason when the
/// index is in use.
pub unsafe fn session_attach(
    s: *mut session,
    w: *mut window,
    idx: c_int,
    cause: &mut Option<CString>,
) -> *mut winlink {
    unsafe {
        let wl = winlink_add(&raw mut (*s).windows, idx);
        if wl.is_null() {
            *cause = Some(xasprintf(c"index in use: %d".as_ptr(), fmt_args![idx]));
            return null_mut::<winlink>();
        }
        (*wl).set_session(s);
        winlink_set_window(wl, w);
        notify_session_window(c"window-linked".as_ptr(), s, w);
        session_group_synchronize_from(s);
        wl
    }
}

/// Takes `wl` out of `s`, moving off it first when it is the current window,
/// and answers whether the session has any windows left.
pub unsafe fn session_detach(s: *mut session, wl: *mut winlink) -> c_int {
    unsafe {
        if session_get_curw(s) == wl && session_last(s) != 0 && session_previous(s, 0) != 0 {
            session_next(s, 0);
        }
        (*wl).flags &= !WINLINK_ALERTFLAGS;
        notify_session_window(c"window-unlinked".as_ptr(), s, (*wl).window());
        winlink_stack_remove(&raw mut (*s).lastw, wl);
        winlink_remove(&raw mut (*s).windows, wl);
        session_group_synchronize_from(s);
        (*s).windows.is_empty() as c_int
    }
}

pub unsafe fn session_has(s: *mut session, w: *mut window) -> c_int {
    unsafe { winlinks_on(w).any(|wl| (*wl).session() == s) as c_int }
}

/// Whether the number of winlinks holding `w` differs from the number expected
/// from `s` alone or from all members of its session group.
pub unsafe fn session_is_linked(s: *mut session, w: *mut window) -> c_int {
    unsafe {
        let sg = session_group_contains(s);
        let links = winlinks_on(w).count();
        if !sg.is_null() {
            return (links != session_group_count(sg) as usize) as c_int;
        }
        (links != 1) as c_int
    }
}

/// Which way a walk between the windows of a session goes.
#[derive(Copy, Clone)]
enum Walk {
    Next,
    Previous,
}

impl Walk {
    /// The window after this one, in the direction being walked.
    unsafe fn step(self, wl: *mut winlink) -> *mut winlink {
        unsafe {
            match self {
                Walk::Next => winlinks_after(wl),
                Walk::Previous => winlinks_before(wl),
            }
        }
    }

    /// The end of the session's windows the walk comes round to.
    unsafe fn wrap(self, s: *mut session) -> *mut winlink {
        unsafe {
            match self {
                Walk::Next => winlinks_first(&raw mut (*s).windows),
                Walk::Previous => winlinks_last(&raw mut (*s).windows),
            }
        }
    }

    /// The first window at or after `wl` carrying an alert, walking this way.
    unsafe fn alert(self, mut wl: *mut winlink) -> *mut winlink {
        unsafe {
            while !wl.is_null() && (*wl).flags & WINLINK_ALERTFLAGS == 0 {
                wl = self.step(wl);
            }
            wl
        }
    }
}

/// Moves `s` to the window `way` of the one it is on, coming round the ends of
/// its windows; with `alert` set, only windows carrying an alert count.
unsafe fn session_walk(s: *mut session, alert: c_int, way: Walk) -> c_int {
    unsafe {
        if session_get_curw(s).is_null() {
            return -1;
        }
        let mut wl = way.step(session_get_curw(s));
        if alert != 0 {
            wl = way.alert(wl);
        }
        if wl.is_null() {
            wl = way.wrap(s);
            if alert != 0 {
                wl = way.alert(wl);
                if wl.is_null() {
                    return -1;
                }
            }
        }
        session_set_current(s, wl)
    }
}

pub unsafe fn session_next(s: *mut session, alert: c_int) -> c_int {
    unsafe { session_walk(s, alert, Walk::Next) }
}

pub unsafe fn session_previous(s: *mut session, alert: c_int) -> c_int {
    unsafe { session_walk(s, alert, Walk::Previous) }
}

pub unsafe fn session_select(s: *mut session, idx: c_int) -> c_int {
    unsafe { session_set_current(s, winlink_find_by_index(&raw mut (*s).windows, idx)) }
}

/// Moves `s` back to the window on top of the stack of the ones it has been
/// in.
pub unsafe fn session_last(s: *mut session) -> c_int {
    unsafe {
        let wl = winlink_of(s, (*s).lastw.first().copied());
        if wl.is_null() {
            return -1;
        }
        if wl == session_get_curw(s) {
            return 1;
        }
        session_set_current(s, wl)
    }
}

/// Makes `wl` the current window of `s`, remembering the one it was on.
pub unsafe fn session_set_current(s: *mut session, wl: *mut winlink) -> c_int {
    unsafe {
        let old = session_get_curw(s);
        if wl.is_null() {
            return -1;
        }
        if wl == old {
            return 1;
        }
        winlink_stack_remove(&raw mut (*s).lastw, wl);
        winlink_stack_push(&raw mut (*s).lastw, old);
        session_set_curw(s, wl);
        if options_get_number(global_options, c"focus-events".as_ptr()) != 0 {
            if !old.is_null() {
                window_update_focus((*old).window());
            }
            window_update_focus((*wl).window());
        }
        winlink_clear_flags(wl);
        window_update_activity((*wl).window());
        tty_update_window_offset((*wl).window());
        notify_session(c"session-window-changed".as_ptr(), s);
        0
    }
}

pub unsafe fn session_group_contains(target: *mut session) -> *mut session_group {
    unsafe {
        each_group()
            .find(|sg| sessions_of(*sg).any(|s| s == target))
            .unwrap_or(null_mut::<session_group>())
    }
}

pub unsafe fn session_group_find(name: *const c_char) -> *mut session_group {
    unsafe { group_of_name(session_groups.map(), name_of(name)) }
}

/// The name the group was made under, which is what `#{session_group}` shows
/// and what a session joining by name is matched against.
pub unsafe fn session_group_name(sg: *mut session_group) -> *const c_char {
    unsafe { cstr_ptr(&(*sg).name) }
}

pub unsafe fn session_group_new(name: *const c_char) -> *mut session_group {
    unsafe {
        let found = session_group_find(name);
        if !found.is_null() {
            return found;
        }
        let mut sg = Box::new(session_group {
            name: Some(CStr::from_ptr(name).to_owned()),
            sessions: session_group_sessions::new(),
        });
        let sg_ptr = &raw mut *sg;
        let key = name_of(cstr_ptr(&(*sg_ptr).name)).to_owned();
        session_groups.map().insert(key, sg);
        sg_ptr
    }
}

/// Puts `s` at the end of `sg`'s list.
unsafe fn group_insert_tail(sg: *mut session_group, s: *mut session) {
    unsafe {
        if let Some(held) = session_ref_from_ptr(s) {
            (*sg).sessions.push(held.downgrade());
        }
    }
}

/// Takes `s` out of `sg`'s list.
unsafe fn group_unlink(sg: *mut session_group, s: *mut session) {
    unsafe {
        if let Some(at) = (*sg)
            .sessions
            .iter()
            .position(|member| member.upgrade().is_some_and(|held| held.as_ptr() == s))
        {
            (*sg).sessions.remove(at);
        }
    }
}

pub unsafe fn session_group_add(sg: *mut session_group, s: *mut session) {
    unsafe {
        if session_group_contains(s).is_null() {
            group_insert_tail(sg, s);
        }
    }
}

/// Takes `s` out of whatever group it is in, and gives the group up once the
/// last session has left it.
pub(crate) unsafe fn session_group_remove(s: *mut session) {
    unsafe {
        let sg = session_group_contains(s);
        if sg.is_null() {
            return;
        }
        group_unlink(sg, s);
        if (*sg).sessions.is_empty() {
            let name = name_of(cstr_ptr(&(*sg).name)).to_owned();
            let _ = session_groups.map().remove(&name);
        }
    }
}

pub unsafe fn session_group_count(sg: *mut session_group) -> u_int {
    unsafe { sessions_of(sg).count() as u_int }
}

pub unsafe fn session_group_attached_count(sg: *mut session_group) -> u_int {
    unsafe { sessions_of(sg).fold(0 as u_int, |n, s| n.wrapping_add((*s).attached)) }
}

/// Brings `s` up to date with the first other session of its group, which is
/// how a session joining one is given the windows the rest hold.
pub unsafe fn session_group_synchronize_to(s: *mut session) {
    unsafe {
        let sg = session_group_contains(s);
        if sg.is_null() {
            return;
        }
        if let Some(target) = sessions_of(sg).find(|target| *target != s) {
            session_group_synchronize1(target, s);
        }
    }
}

/// Gives the windows of `target` to every other session of its group.
pub unsafe fn session_group_synchronize_from(target: *mut session) {
    unsafe {
        let sg = session_group_contains(target);
        if sg.is_null() {
            return;
        }
        for s in sessions_of(sg) {
            if s != target {
                session_group_synchronize1(target, s);
            }
        }
    }
}

/// Gives `s` the windows `target` holds, at the same indexes, and takes away
/// the ones it held before.
///
/// A session whose current window `target` has not got is moved off it first,
/// and then looked up by the index it landed on among the windows it has just
/// been given — an index `target` may not have either, which leaves the
/// session on no window at all. Two sessions of a group hold the same indexes
/// in the ordinary way of things, so that is a shape only a test builds.
unsafe fn session_group_synchronize1(target: *mut session, s: *mut session) {
    unsafe {
        let ww = &raw mut (*target).windows;
        if (*ww).is_empty() {
            return;
        }
        if !session_get_curw(s).is_null()
            && winlink_find_by_index(ww, (*session_get_curw(s)).idx).is_null()
            && session_last(s) != 0
            && session_previous(s, 0) != 0
        {
            session_next(s, 0);
        }
        let mut old_windows = ::core::mem::take(&mut (*s).windows);
        for wl in winlinks_of(ww) {
            let wl2 = winlink_add(&raw mut (*s).windows, (*wl).idx);
            (*wl2).set_session(s);
            winlink_set_window(wl2, (*wl).window());
            notify_session_window(c"window-linked".as_ptr(), s, (*wl2).window());
            (*wl2).flags |= (*wl).flags & WINLINK_ALERTFLAGS;
        }
        let idx = match (*s).curw_idx {
            Some(idx) => idx,
            None => (*session_get_curw(target)).idx,
        };
        session_set_curw(s, winlink_find_by_index(&raw mut (*s).windows, idx));
        for idx in ::core::mem::take(&mut (*s).lastw) {
            let wl2 = winlink_find_by_index(&raw mut (*s).windows, idx);
            if !wl2.is_null() {
                lastw_insert_tail(s, wl2);
                (*wl2).flags |= WINLINK_VISITED;
            }
        }
        while let Some(wl) = old_windows
            .values()
            .next()
            .map(|wl| wl.as_ref() as *const winlink as *mut winlink)
        {
            if winlink_find_by_window_id(&raw mut (*s).windows, (*(*wl).window()).id).is_null() {
                notify_session_window(c"window-unlinked".as_ptr(), s, (*wl).window());
            }
            winlink_remove(&raw mut old_windows, wl);
        }
    }
}

/// Puts `wl` at the end of the stack of windows `s` has been in.
unsafe fn lastw_insert_tail(s: *mut session, wl: *mut winlink) {
    unsafe { (*s).lastw.push((*wl).idx) }
}

/// Closes the gaps between the indexes of `s`'s windows, starting at its own
/// `base-index`, keeping the current window current and the marked pane on the
/// window it is in.
///
/// The `server_clear_marked` guard is kept as the C wrote it, but no test
/// reaches it: the marked window is remembered by the index it has just been
/// given, and a window that has just been linked in at an index is found at
/// it.
pub unsafe fn session_renumber_windows(s: *mut session) {
    unsafe {
        let curw = session_get_curw(s);
        let marked = marked_pane.winlink();
        let mut old_wins = ::core::mem::take(&mut (*s).windows);
        let mut new_idx =
            options_get_number(options_ptr(&(*s).options), c"base-index".as_ptr()) as c_int;
        let mut new_curw_idx = 0 as c_int;
        let mut marked_idx = -1;
        for wl in winlinks_of(&raw mut old_wins) {
            let wl_new = winlink_add(&raw mut (*s).windows, new_idx);
            (*wl_new).set_session(s);
            winlink_set_window(wl_new, (*wl).window());
            (*wl_new).flags |= (*wl).flags & WINLINK_ALERTFLAGS;
            if wl == marked {
                marked_idx = (*wl_new).idx;
            }
            if wl == curw {
                new_curw_idx = (*wl_new).idx;
            }
            new_idx = new_idx.wrapping_add(1);
        }
        for idx in ::core::mem::take(&mut (*s).lastw) {
            let wl = winlink_find_by_index(&raw mut old_wins, idx);
            if wl.is_null() {
                continue;
            }
            (*wl).flags &= !WINLINK_VISITED;
            let wl_new = winlink_find_by_window(&raw mut (*s).windows, (*wl).window());
            if !wl_new.is_null() {
                lastw_insert_tail(s, wl_new);
                (*wl_new).flags |= WINLINK_VISITED;
            }
        }
        if marked_idx != -1 {
            marked_pane.set_winlink(winlink_find_by_index(&raw mut (*s).windows, marked_idx));
            if marked_pane.winlink().is_null() {
                server_clear_marked();
            }
        }
        session_set_curw(
            s,
            winlink_find_by_index(&raw mut (*s).windows, new_curw_idx),
        );
        while let Some(wl) = old_wins
            .values()
            .next()
            .map(|wl| wl.as_ref() as *const winlink as *mut winlink)
        {
            winlink_remove(&raw mut old_wins, wl);
        }
    }
}

/// Tells every pane of every window in `s` that the theme changed.
pub unsafe fn session_theme_changed(s: *mut session) {
    unsafe {
        if s.is_null() {
            return;
        }
        for wl in winlinks_of(&raw mut (*s).windows) {
            for wp in panes_of((*wl).window()) {
                (*wp).flags |= PANE_THEMECHANGED;
            }
        }
    }
}

/// Gives the session's `history-limit` to every pane in it, collecting what a
/// pane holds beyond it there and then.
pub unsafe fn session_update_history(s: *mut session) {
    unsafe {
        let limit =
            options_get_number(options_ptr(&(*s).options), c"history-limit".as_ptr()) as u_int;
        for wl in winlinks_of(&raw mut (*s).windows) {
            for wp in panes_of((*wl).window()) {
                let gd = screen_grid_ptr(&raw mut (*wp).base);
                let osize = (*gd).hsize;
                (*gd).hlimit = limit;
                grid_collect_history(&mut *gd, 1);
                if (*gd).hsize != osize {
                    log_debug(
                        c"%s: %%%u %u -> %u".as_ptr(),
                        fmt_args![
                            c"session_update_history".as_ptr(),
                            (*wp).id,
                            osize,
                            (*gd).hsize
                        ],
                    );
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/test_session.rs"]
mod tests;
