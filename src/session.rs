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
//! process; and the `server_clear_marked` guard in `SessionRef::renumber_windows`,
//! which cannot be reached and says so where it sits. Everything else is
//! covered by the tests below and by `test_coverage_alpha`.
use crate::WindowPane as _;
use crate::compat::strtonum;
use crate::environ::RustEnvironment;
use crate::fmt_args;
use crate::grid::Grid;
use crate::log::{fatalx, log_debug};
use crate::notify::{notify_session, notify_session_window};
use crate::options::{OptionsEngine, RustOptionsEngine};

use crate::pane_identity::PaneIdentity;
use crate::reactor;
use crate::reactor::{Reactor, Timer};
use crate::resize::recalculate_sizes;
use crate::server::server_lock_session;
use crate::server::{marked_pane, server_clear_marked};
use crate::sort::sort_get_sessions;

use crate::tmux::global_options;
use crate::tree::GlobalTree;

pub use crate::consts::{
    EV_TIMEOUT, PANE_THEMECHANGED, RB_BLACK, RB_INF, RB_NEGINF, RB_RED, SORT_NAME, UINT_MAX,
    WINLINK_ACTIVITY, WINLINK_ALERTFLAGS, WINLINK_BELL, WINLINK_SILENCE, WINLINK_VISITED,
};
pub use crate::types::*;
use crate::window::winlink_insert;
use crate::window::{winlink_remove, winlink_stack_push, winlink_stack_remove};
use crate::xmalloc::xasprintf;
use ::core::ffi::{CStr, c_int, c_longlong};
use ::std::ffi::CString;

/// Every session the server holds, by name, which is what holds them alive.
pub(crate) static SESSIONS: GlobalTree<CString, SessionRef> = GlobalTree::new();

thread_local! {
    static NEXT_SESSION_ID: std::cell::Cell<Option<u_int>> = const { std::cell::Cell::new(Some(0)) };
}

pub(crate) fn next_session_id() -> Option<u_int> {
    NEXT_SESSION_ID.get()
}

/// The sessions of a group, in the order they joined it, observed rather than
/// held: a session belongs to the session tree, not to its group.
type session_group_sessions = Vec<SessionWeak>;

/// A set of sessions carrying the same windows, so that a window linked into
/// one of them is linked into all of them.
///
/// Both fields are the group's own: a caller outside this module names a group
/// by `session_group_name` and reads its members through `SessionRef::with_group`, and
/// changes what it holds through `session_group_add_named` and the synchronize
/// calls, which is what keeps the group and its sessions in step.
#[derive(Default)]
#[repr(C)]
pub struct session_group {
    name: Option<CString>,
    pub(crate) sessions: session_group_sessions,
}
impl crate::SessionGroupState for session_group {
    fn from_session_group(name: Option<&CStr>) -> Self {
        Self {
            name: name.map(CStr::to_owned),
            sessions: Vec::new(),
        }
    }

    fn session_group_name(&self) -> Option<&CStr> {
        self.name.as_deref()
    }

    fn set_session_group_name(&mut self, name: Option<&CStr>) {
        self.name = name.map(CStr::to_owned);
    }
}
/// Every session group the server holds, by name.
pub type session_groups_t = std::collections::BTreeMap<CString, Box<session_group>>;

thread_local! {
    pub(crate) static SESSION_GROUPS: std::cell::RefCell<session_groups_t> = const {
        std::cell::RefCell::new(std::collections::BTreeMap::new())
    };
}

pub(crate) fn session_group_registry_remove(name: &CStr) {
    let removed = SESSION_GROUPS.with_borrow_mut(|groups| groups.remove(name));
    drop(removed);
}

/// Upgrades the payload's owner without borrowing its storage.
pub(crate) fn session_ref_of(value: &session) -> Option<SessionRef> {
    value
        .owner
        .as_ref()?
        .upgrade()
        .filter(|owner| owner.points_to(value))
}

/// Files a session in the registry under the name it carries, which is what
/// makes it discoverable and what holds it alive.
pub(crate) fn session_registry_insert(reference: &SessionRef) {
    unsafe {
        let name = (*reference.as_ptr())
            .name
            .as_deref()
            .expect("a registered session has a name")
            .to_owned();
        SESSIONS.map().insert(name, reference.clone());
    }
}

pub(crate) fn session_registry_remove(s: &session) -> Option<SessionRef> {
    SESSIONS
        .map()
        .remove(s.name.as_deref().expect("a registered session has a name"))
}

/// Whether the server holds no session at all.
pub(crate) fn sessions_empty() -> bool {
    SESSIONS.read().is_empty()
}

impl Drop for SessionStorage {
    fn drop(&mut self) {
        let s = &mut self.value;
        s.lock_timer.disarm();
        s.environ = None;
        if let Some(oo) = s.options.take() {
            RustOptionsEngine.destroy(oo);
        }
        s.name = None;
        s.cwd = None;
        s.tio = None;
    }
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
    pub(crate) owner: Option<SessionWeak>,
    pub creation_time: timeval,
    pub last_attached_time: timeval,
    pub last_activity_time: timeval,
    pub lock_timer: TimerHandle,
    /// The index of the link the session is showing, or nothing while it
    /// shows none. A link is named by its index in `windows` and nothing
    /// else, so the session never holds one it has given up.
    pub curw_idx: Option<c_int>,
    pub lastw: winlink_stack,
    pub windows: winlinks,
    pub statusat: c_int,
    pub statuslines: u_int,
    // ---
    id: u_int,
    name: Option<CString>,
    cwd: Option<CString>,
    activity_time: timeval,
    options: Option<crate::options::RustOptionsRef>,
    flags: c_int,
    attached: u_int,
    tio: Option<termios>,
    environ: Option<Box<RustEnvironment>>,
}

impl crate::SessionIdentity for session {
    fn session_id(&self) -> u_int {
        self.id
    }

    fn set_session_id(&mut self, id: u_int) {
        self.id = id;
    }
}

impl crate::SessionNameState for session {
    fn session_name(&self) -> Option<&CStr> {
        self.name.as_deref()
    }

    fn set_session_name(&mut self, name: Option<&CStr>) {
        self.name = name.map(CStr::to_owned);
    }
}

impl crate::SessionDirectoryState for session {
    fn session_directory(&self) -> Option<&CStr> {
        self.cwd.as_deref()
    }

    fn set_session_directory(&mut self, directory: Option<&CStr>) {
        self.cwd = directory.map(CStr::to_owned);
    }
}

impl crate::SessionTimestampState for session {
    fn session_timestamps(&self) -> crate::SessionTimestamps {
        crate::SessionTimestamps {
            creation: self.creation_time,
            last_attached: self.last_attached_time,
            activity: self.activity_time,
            last_activity: self.last_activity_time,
        }
    }

    fn set_session_creation_time(&mut self, time: timeval) {
        self.creation_time = time;
    }

    fn set_session_last_attached_time(&mut self, time: timeval) {
        self.last_attached_time = time;
    }

    fn set_session_activity_time(&mut self, time: timeval) {
        self.activity_time = time;
    }

    fn set_session_last_activity_time(&mut self, time: timeval) {
        self.last_activity_time = time;
    }
}

impl crate::SessionStatusState for session {
    fn session_status(&self) -> crate::SessionStatus {
        crate::SessionStatus {
            position: self.statusat,
            lines: self.statuslines,
        }
    }

    fn set_session_status(&mut self, status: crate::SessionStatus) {
        self.statusat = status.position;
        self.statuslines = status.lines;
    }
}

impl crate::SessionAttachmentState for session {
    fn session_attached(&self) -> u_int {
        self.attached
    }

    fn clear_session_attached(&mut self) {
        self.attached = 0;
    }

    fn add_session_attached(&mut self) {
        self.attached = self.attached.wrapping_add(1);
    }
}

impl crate::SessionAlertState for session {
    fn session_alerted(&self) -> bool {
        self.flags & SESSION_ALERTED != 0
    }

    fn set_session_alerted(&mut self, alerted: bool) {
        if alerted {
            self.flags |= SESSION_ALERTED;
        } else {
            self.flags &= !SESSION_ALERTED;
        }
    }
}

/// Set on a session an alert has already been reported for during the current
/// check, which is what keeps one bell from being reported twice.
const SESSION_ALERTED: c_int = 0x1;

impl session {
    /// The session's shared option handle.
    pub(crate) fn options_ref(&self) -> &RustOptionsRef {
        self.options.as_ref().expect("options are initialized")
    }

    /// The environment a pane started in the session is given.
    pub(crate) fn environ_ref(&self) -> &RustEnvironment {
        self.environ
            .as_deref()
            .expect("a session is created with its environment")
    }

    /// Mutably borrows the environment a pane started in the session is given.
    pub(crate) fn environ_mut(&mut self) -> &mut RustEnvironment {
        self.environ
            .as_deref_mut()
            .expect("a session is created with its environment")
    }
}

/// The link the session shows, for a caller that only means to read it.
impl session {
    pub(crate) fn curw(&self) -> Option<&winlink> {
        self.curw_idx
            .and_then(|idx| self.windows.get(&idx))
            .map(|wl| &**wl)
    }
    pub(crate) fn current_window(&self) -> Option<WindowRef> {
        let current = self.curw()?;
        current.window_handle().cloned()
    }
}

/// The first session the server holds, in name order.
pub fn sessions_first() -> Option<SessionRef> {
    SESSIONS.read().values().next().cloned()
}

fn linked_windows(s: &session) -> impl Iterator<Item = WindowRef> {
    s.windows
        .values()
        .map(|link| {
            link.window_handle()
                .expect("a window link owns its window")
                .clone()
        })
        .collect::<Vec<_>>()
        .into_iter()
}

/// The panes of a window, in the order it carries them.
fn panes_of(w: &WindowRef) -> impl Iterator<Item = RustWindowPaneWeak> {
    w.panes().into_iter()
}

/// The session of `name` in `head`.
fn session_of_name(head: &sessions_t, name: &CStr) -> Option<SessionRef> {
    head.get(name).cloned()
}

fn session_defer_cleanup(reference: SessionRef) {
    reactor::current().defer(move || unsafe {
        let s = reference.as_ptr();
        log_debug(c"session %s freed", fmt_args![(*s).name.as_deref()]);
    });
}

unsafe fn session_lock_timer(s: &mut session) {
    unsafe {
        if s.attached == 0 {
            return;
        }
        log_debug(
            c"session %s locked, activity time %lld",
            fmt_args![s.name.as_deref(), s.activity_time.tv_sec as c_longlong],
        );
        server_lock_session(s);
        recalculate_sizes();
    }
}

impl SessionRef {
    /// Whether this session has a link to the window.
    pub fn has(&self, window: &WindowRef) -> bool {
        window.winlinks().any(|link| link.session().ptr_eq(self))
    }

    /// Whether the window's link count differs from this session or group's size.
    pub fn is_linked(&self, window: &WindowRef) -> bool {
        let expected = self
            .with_group(session_group_count)
            .map_or(1, |count| count as usize);
        window.winlinks().count() != expected
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
    fn step(self, windows: &winlinks, idx: c_int) -> Option<c_int> {
        match self {
            Walk::Next => windows
                .range((std::ops::Bound::Excluded(idx), std::ops::Bound::Unbounded))
                .next()
                .map(|(&idx, _)| idx),
            Walk::Previous => windows
                .range((std::ops::Bound::Unbounded, std::ops::Bound::Excluded(idx)))
                .next_back()
                .map(|(&idx, _)| idx),
        }
    }

    /// The end of the session's windows the walk comes round to.
    fn wrap(self, windows: &winlinks) -> Option<c_int> {
        match self {
            Walk::Next => windows.first_key_value().map(|(&idx, _)| idx),
            Walk::Previous => windows.last_key_value().map(|(&idx, _)| idx),
        }
    }

    /// The first window at or after `idx` carrying an alert, walking this way.
    fn alert(self, windows: &winlinks, mut idx: Option<c_int>) -> Option<c_int> {
        while let Some(current) = idx {
            let wl = windows.get(&current)?;
            if wl.flags & WINLINK_ALERTFLAGS != 0 {
                return Some(current);
            }
            idx = self.step(windows, current);
        }
        None
    }
}

/// Moves `s` to the window `way` of the one it is on, coming round the ends of
/// its windows; with `alert` set, only windows carrying an alert count.
impl SessionRef {
    unsafe fn walk(&self, alert: c_int, way: Walk) -> c_int {
        unsafe {
            let s = self.as_session();
            let Some(current) = s.curw_idx.filter(|idx| s.windows.contains_key(idx)) else {
                return -1;
            };
            let mut idx = way.step(&s.windows, current);
            if alert != 0 {
                idx = way.alert(&s.windows, idx);
            }
            if idx.is_none() {
                idx = way.wrap(&s.windows);
                if alert != 0 {
                    idx = way.alert(&s.windows, idx);
                    if idx.is_none() {
                        return -1;
                    }
                }
            }
            self.set_current(idx)
        }
    }
}

/// The name the group was made under, which is what `#{session_group}` shows
/// and what a session joining by name is matched against.
pub fn session_group_name(sg: &session_group) -> &CStr {
    sg.name.as_deref().expect("a session group has a name")
}

pub(crate) fn with_session_group_named<R>(
    name: &CStr,
    read: impl FnOnce(&session_group) -> R,
) -> Option<R> {
    SESSION_GROUPS.with_borrow(|groups| groups.get(name).map(|group| read(group)))
}

pub(crate) fn session_group_ensure(name: &CStr) {
    SESSION_GROUPS.with_borrow_mut(|groups| {
        groups.entry(name.to_owned()).or_insert_with(|| {
            Box::new(session_group {
                name: Some(name.to_owned()),
                ..session_group::default()
            })
        });
    });
}

pub fn session_group_count(sg: &session_group) -> u_int {
    sg.sessions.iter().filter_map(SessionWeak::upgrade).count() as u_int
}

pub fn session_group_attached_count(sg: &session_group) -> u_int {
    sg.sessions
        .iter()
        .filter_map(SessionWeak::upgrade)
        .fold(0 as u_int, |n, s| n.wrapping_add(s.attached()))
}

#[cfg(test)]
#[path = "tests/test_session.rs"]
mod tests;

#[cfg(test)]
pub(crate) use tests::{session_groups_empty, session_new_detached, session_registry_clear};

impl SessionRef {
    /// Sets the activity timestamp without resetting the lock timer.
    ///
    /// # Safety
    /// The caller must exclude other access to the session payload during this call.
    pub unsafe fn set_activity_time(&self, at: timeval) {
        unsafe { crate::SessionTimestampState::set_session_activity_time(&mut *self.as_ptr(), at) };
    }

    /// Records whether an alert has been reported during the current check.
    ///
    /// # Safety
    /// The caller must exclude other access to the session payload during this call.
    pub unsafe fn set_alerted(&self, alerted: bool) {
        unsafe { crate::SessionAlertState::set_session_alerted(&mut *self.as_ptr(), alerted) };
    }

    /// Starts the attached-client count again for size recalculation.
    ///
    /// # Safety
    /// The caller must exclude other access to the session payload during this call.
    pub(crate) unsafe fn clear_attached(&self) {
        unsafe { crate::SessionAttachmentState::clear_session_attached(&mut *self.as_ptr()) };
    }

    /// Counts one more attached client against the session.
    ///
    /// # Safety
    /// The caller must exclude other access to the session payload during this call.
    pub unsafe fn add_attached(&self) {
        unsafe { crate::SessionAttachmentState::add_session_attached(&mut *self.as_ptr()) };
    }

    /// Changes the directory inherited by new panes.
    ///
    /// # Safety
    /// The caller must exclude other access to the session payload during this call.
    pub unsafe fn set_cwd(&self, cwd: CString) {
        unsafe {
            crate::SessionDirectoryState::set_session_directory(&mut *self.as_ptr(), Some(&cwd))
        };
    }

    /// Changes the session's name and updates its registry entry if registered.
    ///
    /// # Safety
    /// The caller must exclude other access to the session payload during this call.
    pub unsafe fn rename(&self, name: CString) {
        unsafe {
            let s = &mut *self.as_ptr();
            let held = session_registry_remove(s);
            crate::SessionNameState::set_session_name(s, Some(&name));
            if let Some(held) = held {
                session_registry_insert(&held);
            }
        }
    }

    /// Sets the current link without recording a selection or updating windows.
    ///
    /// # Safety
    /// The current-link field must not be borrowed during this call.
    pub unsafe fn set_curw(&self, wl: Option<&winlink>) {
        let index = wl.map(|link| link.idx);
        unsafe { (*self.as_ptr()).curw_idx = index };
    }

    /// Applies the session's history limit to every pane in its windows.
    ///
    /// # Safety
    /// The caller must exclude other access to the session payload during this call.
    pub(crate) unsafe fn update_history(&self) {
        unsafe {
            let limit = self.as_session().options_ref().number(c"history-limit") as u_int;
            for window in linked_windows(self.as_session()) {
                for mut pane in panes_of(&window) {
                    let Some(wp) = pane.get_mut() else { continue };
                    let pane_id = wp.pane_id();
                    let gd = wp.base_mut().grid_mut();
                    let osize = gd.hsize;
                    gd.hlimit = limit;
                    gd.collect_history(true);
                    if gd.hsize != osize {
                        log_debug(
                            c"%s: %%%u %u -> %u",
                            fmt_args![c"session_update_history".as_ptr(), pane_id, osize, gd.hsize],
                        );
                    }
                }
            }
        }
    }
}

impl SessionRef {
    /// # Safety
    /// The caller must exclude other access to the session payload during this call.
    pub unsafe fn destroy(&self, notify: c_int, from: &CStr) {
        unsafe {
            let s = &mut *self.as_ptr();
            log_debug(
                c"session %s destroyed (%s)",
                fmt_args![s.name.as_deref(), from],
            );
            if s.curw_idx.is_none() {
                return;
            }
            s.curw_idx = None;
            session_registry_remove(s);
            if notify != 0 {
                notify_session(c"session-closed", Some(&*s));
            }
            s.tio = None;
            s.lock_timer.disarm();
            self.leave_group();
            for index in core::mem::take(&mut s.lastw) {
                if let Some(link) = s.windows.get_mut(&index) {
                    link.flags &= !WINLINK_VISITED;
                }
            }
            while let Some((&index, link)) = s.windows.first_key_value() {
                let window = link
                    .window_handle()
                    .expect("a link owns its window")
                    .clone();
                notify_session_window(c"window-unlinked", s, &window);
                winlink_remove(&mut s.windows, index);
            }
            s.cwd = None;
            session_defer_cleanup(self.clone());
        }
    }

    /// Records that something happened in `s` at `from`, or now when there is no
    /// time, and arms the lock timer when somebody is attached and the session has
    /// a `lock-after-time`.
    ///
    /// # Safety
    /// The caller must exclude other access to the session payload during this call.
    pub unsafe fn update_activity(&self, from: Option<&timeval>) {
        unsafe {
            let s = &mut *self.as_ptr();
            match from {
                Some(from) => crate::SessionTimestampState::set_session_activity_time(s, *from),
                None => {
                    s.activity_time = timeval::now();
                }
            }
            log_debug(
                c"session $%u %s activity %lld.%06d",
                fmt_args![
                    s.id,
                    s.name.as_deref(),
                    s.activity_time.tv_sec as c_longlong,
                    s.activity_time.tv_usec as c_int
                ],
            );
            if s.lock_timer.is_set() {
                s.lock_timer.disarm();
            } else {
                let watching = self.downgrade();
                s.lock_timer.set_callback(move || {
                    if let Some(mut s) = watching.upgrade() {
                        session_lock_timer(s.as_session_mut());
                    }
                });
            }
            if s.attached != 0 {
                let tv = timeval {
                    tv_sec: ((*s).options_ref()).number(c"lock-after-time") as __time_t,
                    tv_usec: 0 as __suseconds_t,
                };
                if tv.tv_sec != 0 {
                    s.lock_timer.arm(tv);
                }
            }
        }
    }

    /// Links `w` into `s` at `idx`, or answers nothing and the reason when the
    /// index is in use.
    ///
    /// # Safety
    /// The caller must exclude other access to the session payload during this call.
    pub unsafe fn attach(
        &self,
        w: WindowRef,
        idx: c_int,
        cause: &mut Option<CString>,
    ) -> Option<c_int> {
        unsafe {
            let s = &mut *self.as_ptr();
            let Some(link) = winlink_insert(&mut s.windows, idx) else {
                *cause = Some(xasprintf(c"index in use: %d", fmt_args![idx]));
                return None;
            };
            link.session_ref = Some(self.downgrade());
            link.set_window(w.clone());
            let index = link.idx;
            notify_session_window(c"window-linked", s, &w);
            self.synchronize_group_from();
            Some(index)
        }
    }

    /// Removes the link at `index`, moving off it first when it is current,
    /// and answers whether the session has no windows left.
    ///
    /// # Safety
    /// The caller must exclude other access to the session payload during this call.
    pub unsafe fn detach(&self, index: c_int) -> c_int {
        unsafe {
            let s = &mut *self.as_ptr();
            let Some(window) = s
                .windows
                .get(&index)
                .and_then(|link| link.window_handle())
                .cloned()
            else {
                return s.windows.is_empty() as c_int;
            };
            if s.curw().is_some_and(|link| link.idx == index)
                && self.last() != 0
                && self.previous(0) != 0
            {
                self.next(0);
            }
            let s = &mut *self.as_ptr();
            if let Some(link) = s.windows.get_mut(&index) {
                link.flags &= !WINLINK_ALERTFLAGS;
            }
            notify_session_window(c"window-unlinked", s, &window);
            drop(window);
            winlink_stack_remove(&mut s.lastw, s.windows.get_mut(&index).map(Box::as_mut));
            winlink_remove(&mut s.windows, index);
            self.synchronize_group_from();
            self.as_session().windows.is_empty() as c_int
        }
    }

    /// # Safety
    /// The caller must exclude other access to the session payload during this call.
    pub unsafe fn next(&self, alert: c_int) -> c_int {
        unsafe { self.walk(alert, Walk::Next) }
    }

    /// # Safety
    /// The caller must exclude other access to the session payload during this call.
    pub unsafe fn previous(&self, alert: c_int) -> c_int {
        unsafe { self.walk(alert, Walk::Previous) }
    }

    /// # Safety
    /// The caller must exclude other access to the session payload during this call.
    pub unsafe fn select(&self, idx: c_int) -> c_int {
        unsafe {
            if idx < 0 {
                fatalx(c"bad index", fmt_args![]);
            }
            self.set_current(Some(idx))
        }
    }

    /// Moves `s` back to the window on top of the stack of the ones it has been
    /// in.
    ///
    /// # Safety
    /// The caller must exclude other access to the session payload during this call.
    pub unsafe fn last(&self) -> c_int {
        unsafe {
            let index = self.as_session().lastw.first().copied();
            self.set_current(index)
        }
    }

    /// Makes the link at `index` current, remembering the window the session was on.
    ///
    /// # Safety
    /// The caller must exclude other access to the session payload during this call.
    pub unsafe fn set_current(&self, index: Option<c_int>) -> c_int {
        unsafe {
            let s = &mut *self.as_ptr();
            let Some(index) = index.filter(|index| s.windows.contains_key(index)) else {
                return -1;
            };
            let old_index = s.curw().map(|link| link.idx);
            if old_index == Some(index) {
                return 1;
            }
            let window = s
                .windows
                .get(&index)
                .and_then(|link| link.window_handle())
                .expect("a link owns its window")
                .clone();
            let old_window = old_index
                .and_then(|index| s.windows.get(&index))
                .and_then(|link| link.window_handle())
                .cloned();
            winlink_stack_remove(&mut s.lastw, s.windows.get_mut(&index).map(Box::as_mut));
            winlink_stack_push(
                &mut s.lastw,
                old_index
                    .and_then(|index| s.windows.get_mut(&index))
                    .map(Box::as_mut),
            );
            s.curw_idx = Some(index);
            if global_options
                .as_ref()
                .expect("global options are initialized")
                .number(c"focus-events")
                != 0
            {
                if let Some(old_window) = old_window {
                    (old_window).update_focus();
                }
                (window).update_focus();
            }
            (window.clone()).clear_alerts();
            window.update_activity();
            window.update_client_offsets();
            notify_session(c"session-window-changed", Some(s));
            0
        }
    }

    /// Closes the gaps between the indexes of `s`'s windows, starting at its own
    /// `base-index`, keeping the current window current and the marked pane on the
    /// window it is in.
    ///
    /// The `server_clear_marked` guard is kept as the C wrote it, but no test
    /// reaches it: the marked window is remembered by the index it has just been
    /// given, and a window that has just been linked in at an index is found at
    /// it.
    ///
    /// # Safety
    /// The caller must exclude other access to the session payload during this call.
    pub unsafe fn renumber_windows(&self) {
        unsafe {
            let s = &mut *self.as_ptr();
            let current_index = s.curw().map(|link| link.idx);
            let marked_index = marked_pane
                .session()
                .filter(|marked| self.ptr_eq(marked))
                .and(marked_pane.wl_idx);
            let mut old_windows = core::mem::take(&mut s.windows);
            let mut next_index = s.options_ref().number(c"base-index") as c_int;
            let mut current_index_new = 0;
            let mut marked_index_new = None;
            for link in old_windows.values() {
                let new = winlink_insert(&mut s.windows, next_index)
                    .expect("renumbering assigns each link an unused index");
                new.session_ref = Some(self.downgrade());
                new.set_window(
                    link.window_handle()
                        .expect("a link owns its window")
                        .clone(),
                );
                new.flags |= link.flags & WINLINK_ALERTFLAGS;
                if Some(link.idx) == marked_index {
                    marked_index_new = Some(new.idx);
                }
                if Some(link.idx) == current_index {
                    current_index_new = new.idx;
                }
                next_index = next_index.wrapping_add(1);
            }
            for index in core::mem::take(&mut s.lastw) {
                let Some(link) = old_windows.get_mut(&index) else {
                    continue;
                };
                link.flags &= !WINLINK_VISITED;
                let window = link.window_handle().expect("a link owns its window");
                if let Some(new) = s.windows.values_mut().find(|new| {
                    new.window_handle()
                        .is_some_and(|candidate| candidate.ptr_eq(window))
                }) {
                    s.lastw.push(new.idx);
                    new.flags |= WINLINK_VISITED;
                }
            }
            if let Some(index) = marked_index_new {
                let marked = s.windows.get(&index).map(Box::as_ref);
                marked_pane.set_winlink(marked);
                if marked.is_none() {
                    server_clear_marked();
                }
            }
            s.curw_idx = s.windows.get(&current_index_new).map(|link| link.idx);
            while let Some(index) = old_windows.keys().next().copied() {
                winlink_remove(&mut old_windows, index);
            }
        }
    }

    /// Tells every pane of every window in `s` that the theme changed.
    ///
    /// # Safety
    /// The caller must exclude other access to the session payload during this call.
    pub unsafe fn theme_changed(&self) {
        unsafe {
            let s = &mut *self.as_ptr();
            for window in linked_windows(s) {
                for mut pane in panes_of(&window) {
                    let Some(wp) = pane.get_mut() else { continue };
                    *wp.flags_mut() |= PANE_THEMECHANGED;
                }
            }
        }
    }
}

impl SessionRef {
    /// Copies the session name without retaining a payload borrow.
    pub fn name(&self) -> Option<CString> {
        unsafe { crate::SessionNameState::session_name_owned(self.as_session()) }
    }

    /// The identifier used to target the session as `$id`.
    pub fn id(&self) -> u_int {
        unsafe { crate::SessionIdentity::session_id(self.as_session()) }
    }

    /// Copies the directory inherited by new panes.
    pub fn cwd(&self) -> Option<CString> {
        unsafe {
            crate::SessionDirectoryState::session_directory(self.as_session()).map(CStr::to_owned)
        }
    }

    /// Clones the shared option handle without retaining a session borrow.
    pub fn options(&self) -> RustOptionsRef {
        unsafe { self.as_session().options_ref().clone() }
    }

    /// Copies the terminal settings supplied by the first attached client.
    pub fn tio(&self) -> Option<termios> {
        unsafe { self.as_session().tio }
    }

    /// The number of attached clients.
    pub fn attached(&self) -> u_int {
        unsafe { crate::SessionAttachmentState::session_attached(self.as_session()) }
    }

    /// The most recent activity timestamp.
    pub fn activity_time(&self) -> timeval {
        unsafe { crate::SessionTimestampState::session_timestamps(self.as_session()).activity }
    }

    /// Whether an alert has been reported during the current check.
    pub fn alerted(&self) -> bool {
        unsafe { crate::SessionAlertState::session_alerted(self.as_session()) }
    }

    /// Updates this session's environment from the client using its configured
    /// `update-environment` patterns. Unmatched names and valueless entries keep
    /// the store's clearing semantics; no environment snapshot is retained.
    ///
    /// # Safety
    /// Exclude other session-environment access and mutation of the client's
    /// environment or session options during this synchronous call. It invokes
    /// no command, hook or client callbacks, and releases the borrows on return.
    pub(crate) unsafe fn update_environment_from(&mut self, client: &ClientRef) {
        unsafe {
            let options = self.options();
            crate::environ::update_environment(&options, client.environ_ref(), self.environ());
        }
    }

    /// Borrows the environment inherited by new panes.
    ///
    /// # Safety
    /// The caller must exclude all other access to the environment for the
    /// returned borrow's lifetime, including through cloned session handles.
    pub unsafe fn environ(&mut self) -> &mut RustEnvironment {
        unsafe { self.as_session_mut().environ_mut() }
    }
}

impl SessionRef {
    /// Reassigns or exits this session's clients according to `detach-on-destroy`
    /// and recalculates sizes. The caller performs session destruction afterwards.
    ///
    /// # Safety
    /// Run on the initialized server thread without outstanding client, session,
    /// window or pane payload borrows. Client reassignment may revisit handles
    /// and queue notifications before this operation returns.
    pub(crate) unsafe fn prepare_destruction(&mut self) {
        unsafe { crate::server::server_destroy_session(self.as_session_mut()) }
    }

    pub fn find(name: &CStr) -> Option<SessionRef> {
        session_of_name(&SESSIONS.read(), name)
    }
    pub fn find_by_id_str(s: &CStr) -> Option<SessionRef> {
        unsafe {
            if s.to_bytes().first() != Some(&b'$') {
                return None;
            }
            let id = CStr::from_bytes_with_nul(&s.to_bytes_with_nul()[1..])
                .expect("the tail of a C string ends at the same NUL");
            let Ok(id) = strtonum(id, 0 as c_longlong, UINT_MAX as c_longlong) else {
                return None;
            };
            SessionRef::find_by_id(id as u_int)
        }
    }
    pub fn find_by_id(id: u_int) -> Option<SessionRef> {
        SESSIONS.read().values().find(|s| s.id() == id).cloned()
    }
    /// Creates a session of `name`, or one named after `prefix` and the id it is
    /// given when there is no name. The session takes over `env` and `oo`, and
    /// copies the terminal settings so the caller keeps its own.
    ///
    /// # Safety
    /// The caller must initialize server options and the reactor on this thread.
    pub unsafe fn create(
        prefix: Option<&CStr>,
        name: Option<&CStr>,
        cwd: &CStr,
        env: Box<RustEnvironment>,
        oo: crate::options::RustOptionsRef,
        tio: Option<&termios>,
    ) -> SessionRef {
        unsafe {
            let id = crate::entity_id::next_entity_id(&NEXT_SESSION_ID);
            let reference = SessionRef::new(session {
                cwd: Some(cwd.to_owned()),
                options: Some(oo),
                environ: Some(env),
                tio: tio.copied(),
                ..session::default()
            });
            let s = reference.as_ptr();
            reference.update_status_cache();
            (*s).id = id;
            if let Some(name) = name {
                (*s).name = Some(name.to_owned());
            } else {
                loop {
                    (*s).name = Some(match prefix {
                        Some(prefix) => xasprintf(c"%s-%u", fmt_args![prefix.as_ptr(), (*s).id]),
                        None => xasprintf(c"%u", fmt_args![(*s).id]),
                    });
                    if session_of_name(
                        &SESSIONS.read(),
                        (*s).name.as_deref().expect("a new session has a name"),
                    )
                    .is_none()
                    {
                        break;
                    }
                    (*s).id = crate::entity_id::next_entity_id(&NEXT_SESSION_ID);
                }
            }
            session_registry_insert(&reference);
            log_debug(
                c"new session %s $%u",
                fmt_args![(*s).name.as_deref(), (*s).id],
            );
            (*s).creation_time = timeval::now();
            let created = (*s).creation_time;
            reference.update_activity(Some(&created));
            reference
        }
    }
    /// Whether the server still holds this session in its live registry.
    pub fn is_registered(&self) -> bool {
        SESSIONS.read().values().any(|session| session.ptr_eq(self))
    }

    fn in_sorted_order(&self, sort_crit: &sort_criteria_t) -> Option<(Vec<SessionRef>, usize)> {
        if !self.is_registered() {
            return None;
        }
        let list = sort_get_sessions(sort_crit);
        match list.iter().position(|session| session.ptr_eq(self)) {
            Some(index) => Some((list, index)),
            None => {
                fatalx(
                    c"session %s not found in sorted list",
                    fmt_args![self.name().as_deref()],
                )
            },
        }
    }

    /// The next registered session in the requested order, wrapping at the end.
    pub fn next_session(&self, sort_crit: &sort_criteria_t) -> Option<SessionRef> {
        self.in_sorted_order(sort_crit)
            .map(|(list, index)| list[(index + 1) % list.len()].clone())
    }

    /// The previous registered session in the requested order, wrapping at the start.
    pub fn previous_session(&self, sort_crit: &sort_criteria_t) -> Option<SessionRef> {
        self.in_sorted_order(sort_crit)
            .map(|(list, index)| list[(index + list.len() - 1) % list.len()].clone())
    }
}

impl SessionRef {
    /// Whether the other session belongs to the same registered group.
    pub(crate) fn shares_group(&self, other: &SessionRef) -> bool {
        let this = self.downgrade();
        let other = other.downgrade();
        SESSION_GROUPS.with_borrow(|groups| {
            groups.values().any(|group| {
                group.sessions.iter().any(|member| member.ptr_eq(&this))
                    && group.sessions.iter().any(|member| member.ptr_eq(&other))
            })
        })
    }

    /// Reads the session's group while the group registry is borrowed.
    pub(crate) fn with_group<R>(&self, read: impl FnOnce(&session_group) -> R) -> Option<R> {
        let this = self.downgrade();
        SESSION_GROUPS.with_borrow(|groups| {
            groups
                .values()
                .find(|group| group.sessions.iter().any(|member| member.ptr_eq(&this)))
                .map(|group| read(group))
        })
    }

    /// Saves the next group member before each body, as TAILQ_FOREACH_SAFE does.
    /// Registry borrows end before yielding; only the current and next members are retained.
    pub(crate) fn group_walk_safe(&self) -> Option<impl Iterator<Item = SessionRef> + use<>> {
        let (name, mut next) = self.with_group(|group| {
            (
                session_group_name(group).to_owned(),
                group.sessions.iter().find_map(SessionWeak::upgrade),
            )
        })?;
        Some(std::iter::from_fn(move || {
            let current = next.take()?;
            let identity = current.downgrade();
            next = SESSION_GROUPS.with_borrow(|groups| {
                let group = groups.get(&name)?;
                let at = group
                    .sessions
                    .iter()
                    .position(|member| member.ptr_eq(&identity))?;
                group.sessions[at + 1..]
                    .iter()
                    .find_map(SessionWeak::upgrade)
            });
            Some(current)
        }))
    }

    /// Joins an existing named group if the session is not already grouped.
    pub(crate) fn join_group(&self, name: &CStr) {
        if self.with_group(|_| ()).is_none() {
            SESSION_GROUPS.with_borrow_mut(|groups| {
                let group = groups.get_mut(name).expect("the session group exists");
                group.sessions.push(self.downgrade());
            });
        }
    }

    /// Leaves the session's group, removing the group when its last member leaves.
    pub(crate) fn leave_group(&self) {
        let empty_group =
            SESSION_GROUPS.with_borrow_mut(|groups| {
                for (name, group) in groups {
                    if let Some(at) = group.sessions.iter().position(|member| {
                        member.upgrade().is_some_and(|member| member.ptr_eq(self))
                    }) {
                        group.sessions.remove(at);
                        return group.sessions.is_empty().then(|| name.clone());
                    }
                }
                None
            });
        if let Some(name) = empty_group {
            session_group_registry_remove(&name);
        }
    }

    /// Gives this session the windows held by the first other group member.
    ///
    /// # Safety
    /// The caller must exclude other access to the group members' payloads during this call.
    pub unsafe fn synchronize_group_to(&self) {
        self.with_group(|group| {
            if let Some(source) = group
                .sessions
                .iter()
                .filter_map(SessionWeak::upgrade)
                .find(|member| !member.ptr_eq(self))
            {
                unsafe { self.synchronize_from_session(&source) };
            }
        });
    }

    /// Gives this session's windows to every other group member.
    ///
    /// # Safety
    /// The caller must exclude other access to the group members' payloads during this call.
    pub unsafe fn synchronize_group_from(&self) {
        self.with_group(|group| {
            for member in group.sessions.iter().filter_map(SessionWeak::upgrade) {
                if !member.ptr_eq(self) {
                    unsafe { member.synchronize_from_session(self) };
                }
            }
        });
    }
    unsafe fn synchronize_from_session(&self, source: &SessionRef) {
        unsafe {
            let target = source.as_session();
            let s = self.as_session();
            if target.windows.is_empty() {
                return;
            }
            if s.curw()
                .is_some_and(|link| !target.windows.contains_key(&link.idx))
                && self.last() != 0
                && self.previous(0) != 0
            {
                self.next(0);
            }
            let sources: Vec<_> = target
                .windows
                .values()
                .map(|link| {
                    (
                        link.idx,
                        link.window_handle()
                            .expect("a link owns its window")
                            .clone(),
                        link.flags & WINLINK_ALERTFLAGS,
                    )
                })
                .collect();
            let s = &mut *self.as_ptr();
            let mut old_windows = core::mem::take(&mut s.windows);
            for (index, window, flags) in sources {
                let new = winlink_insert(&mut s.windows, index)
                    .expect("source session indexes are unique");
                new.session_ref = Some(self.downgrade());
                new.set_window(window.clone());
                notify_session_window(c"window-linked", s, &window);
                if let Some(new) = s.windows.get_mut(&index) {
                    new.flags |= flags;
                }
            }
            let current = s.curw_idx.or_else(|| target.curw().map(|link| link.idx));
            s.curw_idx = current.and_then(|index| s.windows.get(&index).map(|link| link.idx));
            for index in core::mem::take(&mut s.lastw) {
                if let Some(link) = s.windows.get_mut(&index) {
                    s.lastw.push(link.idx);
                    link.flags |= WINLINK_VISITED;
                }
            }
            while let Some((&index, link)) = old_windows.first_key_value() {
                let window = link
                    .window_handle()
                    .expect("a link owns its window")
                    .clone();
                let id = window.window_id();
                if !s.windows.values().any(|link| {
                    link.window_handle()
                        .is_some_and(|window| window.window_id() == id)
                }) {
                    notify_session_window(c"window-unlinked", s, &window);
                }
                winlink_remove(&mut old_windows, index);
            }
        }
    }
}

impl SessionRef {
    /// Retains the session and index of its current window link.
    pub(crate) fn curw(&self) -> Option<crate::window::WinlinkRef> {
        let index = unsafe { self.as_session().curw()?.idx };
        crate::window::WinlinkRef::new(self.clone(), index)
    }

    /// Clones the current window handle without retaining a session payload borrow.
    pub(crate) fn current_window(&self) -> Option<WindowRef> {
        unsafe { self.as_session().current_window() }
    }
}
