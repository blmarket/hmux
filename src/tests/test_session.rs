//! What is left uncovered here, and why. The `fatal` arm for a clock that
//! would not answer and the `fatalx` arm for a session the sorted list does
//! not hold end the process, so a unit test that entered one would take the
//! whole run with it. `server_clear_marked` inside
//! `session_renumber_windows` cannot be reached: the marked winlink is only
//! remembered by the index it was just given, and a winlink that has just
//! been added at an index is found at it. Nothing else is left.

use super::*;
use crate::grid::grid_scroll_history;
use crate::options::options_set_number;
use crate::screen::screen_grid_ptr;
use crate::session::winlink_of;
use crate::session::{session_get_curw, session_set_curw};
use crate::tests::test_fixtures::{
    Environ, Options, Pane, Session, Window, ensure_reactor, globals, link, seen, unlink, zeroed,
};
use crate::window::PANE_FOCUSED;
use crate::window::window_set_active;
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr::{null, null_mut};
use ::std::ffi::CString;
use ::std::sync::MutexGuard;

/// A turn at the server-wide state these tests reach — the session tree,
/// the session groups, the id the next session is given and the marked
/// pane — starting from empty trees and leaving them empty.
fn server() -> MutexGuard<'static, ()> {
    let guard = globals();
    ensure_reactor();
    assert!(sessions.map().is_empty(), "the session tree is not empty");
    assert!(
        session_groups.map().is_empty(),
        "the session groups are not empty"
    );
    guard
}

/// A number sequence that is the same every run, which is what shuffles
/// the entries the tree tests put in and take out again.
struct Order(u64);

impl Order {
    fn next(&mut self, below: usize) -> usize {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.0 >> 33) as usize % below
    }
}

/// A session that is nothing but a name, which is all the tree's own
/// comparison reads. It is in no tree of the server's and owns nothing
/// else.
struct Named {
    session: Box<session>,
    _name: CString,
}

impl Named {
    fn new(name: &str) -> Named {
        let name = CString::new(name).expect("no NUL");
        let mut session = Box::new(session::default());
        session.name = Some(name.clone());
        Named {
            session,
            _name: name,
        }
    }

    fn ptr(&mut self) -> *mut session {
        &raw mut *self.session
    }
}

/// The names in a tree, in the order it walks them.
unsafe fn walk(head: *mut sessions_t) -> Vec<String> {
    unsafe {
        (*head)
            .values()
            .map(|s| seen(cstr_ptr(&(*s.as_ptr()).name)))
            .collect()
    }
}

/// The sessions a test made through the real `session_create`, kept alive
/// until the test has finished inspecting them.
struct Created(Vec<SessionRef>);

impl Created {
    fn new() -> Created {
        Created(Vec::new())
    }

    /// A session of `name`, or one named after `prefix` and its id when
    /// there is no name, with a fresh environment and a session option set
    /// of its own — which is what `cmd-new-session` hands over.
    fn session(&mut self, prefix: Option<&CStr>, name: Option<&CStr>) -> *mut session {
        unsafe {
            let s = session_create(
                prefix.map_or(null::<c_char>(), |p| p.as_ptr()),
                name.map_or(null::<c_char>(), |n| n.as_ptr()),
                c"/tmp".as_ptr(),
                Environ::new().owned(),
                Options::session().owned(),
                null_mut::<termios>(),
            );
            self.0
                .push(session_ref_from_ptr(s).expect("created session handle"));
            s
        }
    }
}

impl Drop for Created {
    fn drop(&mut self) {
        for reference in &self.0 {
            let s = reference.as_ptr();
            if unsafe { session_alive(s) } == 0 {
                continue;
            }
            session_registry_remove(s);
        }
    }
}

/// The names of every session the server has.
fn registered() -> Vec<String> {
    unsafe { walk(sessions.map()) }
}

#[test]
fn a_session_is_created_with_the_name_it_is_given() {
    let _guard = server();
    let mut created = Created::new();
    unsafe {
        let was = next_session_id;
        let s = created.session(None, Some(c"named"));
        assert_eq!(seen(cstr_ptr(&(*s).name)), "named");
        assert_eq!((*s).id, was);
        assert_eq!(::core::ptr::read(&raw const next_session_id), was + 1);
        assert_eq!(seen(cstr_ptr(&(*s).cwd)), "/tmp");
        assert!(session_ref_from_ptr(s).is_some());
        assert_eq!((*s).flags, 0);
        assert!((*s).tio.is_none());
        assert!(session_get_curw(s).is_null());
        assert!((*s).windows.is_empty());
        assert_eq!(registered(), ["named"]);
        assert_eq!((*s).activity_time.tv_sec, (*s).creation_time.tv_sec);
        assert!((*s).creation_time.tv_sec > 0);
    }
}

/// A session with no name is named after the prefix it was given and the
/// id it was handed, and after the id alone when there is no prefix.
#[test]
fn a_session_with_no_name_is_named_after_its_id() {
    let _guard = server();
    let mut created = Created::new();
    unsafe {
        let first = created.session(Some(c"pre"), None);
        let second = created.session(None, None);
        assert_eq!(
            seen(cstr_ptr(&(*first).name)),
            format!("pre-{}", (*first).id)
        );
        assert_eq!(seen(cstr_ptr(&(*second).name)), format!("{}", (*second).id));
        assert_eq!((*second).id, (*first).id + 1);
    }
}

/// The name it works out has to be one nobody has, so it keeps taking ids
/// until it finds one — which leaves the ids in between spent.
#[test]
fn a_name_that_is_taken_costs_the_next_session_its_id() {
    let _guard = server();
    let mut created = Created::new();
    unsafe {
        let next_id = ::core::ptr::read(&raw const next_session_id);
        let taken_name = CString::new(format!("pre-{}", next_id + 1)).expect("no NUL");
        let held = created.session(None, Some(taken_name.as_c_str()));
        let next = created.session(Some(c"pre"), None);
        assert_eq!(
            seen(cstr_ptr(&(*held).name)),
            format!("pre-{}", (*held).id + 1)
        );
        assert_eq!(
            seen(cstr_ptr(&(*next).name)),
            format!("pre-{}", (*held).id + 2)
        );
        assert_eq!((*next).id, (*held).id + 2);
    }
}

/// The terminal settings a session is created with are copied, so the
/// caller's own are its to keep.
#[test]
fn the_terminal_settings_are_copied_into_the_session() {
    let _guard = server();
    let mut created = Created::new();
    unsafe {
        let mut tio = zeroed::<termios>();
        tio.c_iflag = 0x2d5;
        let s = session_create(
            null::<c_char>(),
            c"tio".as_ptr(),
            c"/tmp".as_ptr(),
            Environ::new().owned(),
            Options::session().owned(),
            &raw mut *tio,
        );
        created
            .0
            .push(session_ref_from_ptr(s).expect("created session handle"));
        tio.c_iflag = 0;
        assert_eq!((*s).tio.as_ref().unwrap().c_iflag, 0x2d5);
    }
}

#[test]
fn a_session_is_alive_while_the_server_holds_it() {
    let _guard = server();
    let mut created = Created::new();
    let mut apart = Session::new(1, "apart");
    unsafe {
        let s = created.session(None, Some(c"held"));
        assert_eq!(session_alive(s), 1);
        assert_eq!(session_alive(apart.ptr()), 0);
        let name = name_of(cstr_ptr(&(*s).name)).to_owned();
        let reference = session_registry_remove(s).expect("session owner");
        assert_eq!(session_alive(s), 0);
        sessions.map().insert(name, reference);
    }
}

#[test]
fn a_session_is_found_by_name_and_by_id() {
    let _guard = server();
    let mut created = Created::new();
    unsafe {
        let s = created.session(None, Some(c"findable"));
        let id = (*s).id;
        assert_eq!(session_find(c"findable".as_ptr()), s);
        assert!(session_find(c"nonesuch".as_ptr()).is_null());
        assert_eq!(session_find_by_id(id), s);
        assert!(session_find_by_id(id + 1000).is_null());
        let by_str = CString::new(format!("${id}")).expect("no NUL");
        assert_eq!(session_find_by_id_str(by_str.as_ptr()), s);
    }
}

/// An id is only read after a dollar, and only when what follows it is a
/// number the tree could hold.
#[test]
fn an_id_that_is_not_a_number_after_a_dollar_finds_nothing() {
    let _guard = server();
    let mut created = Created::new();
    unsafe {
        let s = created.session(None, Some(c"findable"));
        let plain = CString::new(format!("{}", (*s).id)).expect("no NUL");
        assert!(session_find_by_id_str(plain.as_ptr()).is_null());
        assert!(session_find_by_id_str(c"$".as_ptr()).is_null());
        assert!(session_find_by_id_str(c"$x".as_ptr()).is_null());
        assert!(session_find_by_id_str(c"$-1".as_ptr()).is_null());
        assert!(session_find_by_id_str(c"$99999999999".as_ptr()).is_null());
        assert!(session_find_by_id_str(c"".as_ptr()).is_null());
    }
}

/// Removing a session from the live registry does not reclaim it while a
/// strong handle remains, and the weak handle stops upgrading after the
/// last strong handle goes away.
#[test]
fn a_session_is_kept_while_anything_holds_a_handle_to_it() {
    let _guard = server();
    let mut created = Created::new();
    unsafe {
        let s = created.session(None, Some(c"counted"));
        let reference = session_ref_from_ptr(s).expect("session owner");
        let weak = reference.downgrade();
        let name = name_of(cstr_ptr(&(*s).name)).to_owned();
        session_registry_remove(s);
        assert_eq!(seen(cstr_ptr(&(*s).name)), "counted");
        assert!(weak.upgrade().is_some());
        sessions.map().insert(name, reference);
        session_registry_remove(s);
        created.0.clear();
        assert!(weak.upgrade().is_none());
    }
}

/// A session with windows linked into it, all of them the server-free
/// fixtures, put together the way `session_attach` does it and taken apart
/// again at the end of the test.
struct Linked {
    session: Session,
    windows: Vec<Window>,
    winlinks: Vec<*mut winlink>,
}

impl Linked {
    /// A session carrying `windows` many windows at indexes one upwards,
    /// the first of them current.
    fn new(name: &str, windows: u_int) -> Linked {
        let mut linked = Linked {
            session: Session::new(1, name),
            windows: Vec::new(),
            winlinks: Vec::new(),
        };
        for i in 0..windows {
            linked.attach(&format!("w{i}"), i as c_int + 1);
        }
        if let Some(first) = linked.winlinks.first() {
            unsafe { session_set_curw(linked.session.ptr(), *first) };
        }
        linked
    }

    /// Links a new window in at `idx` through the real `session_attach`.
    /// The window's id is one nothing else has: the server tells two
    /// windows apart by it, and two sessions holding windows of the same
    /// id look to it like two sessions holding one window.
    fn attach(&mut self, name: &str, idx: c_int) -> *mut winlink {
        static NEXT_ID: ::std::sync::atomic::AtomicU32 = ::std::sync::atomic::AtomicU32::new(1);
        let id = NEXT_ID.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed) as u_int;
        let mut w = Window::new(id, name, 80, 24);
        let wl = unsafe {
            let mut cause = None;
            let wl = session_attach(self.session.ptr(), w.ptr(), idx, &mut cause);
            assert!(!wl.is_null(), "index {idx} was in use");
            wl
        };
        self.windows.push(w);
        self.winlinks.push(wl);
        wl
    }

    fn ptr(&mut self) -> *mut session {
        self.session.ptr()
    }

    fn window(&mut self, i: usize) -> *mut window {
        self.windows[i].ptr()
    }

    fn wl(&self, i: usize) -> *mut winlink {
        self.winlinks[i]
    }

    /// Which window is current, by name.
    fn current(&mut self) -> Option<String> {
        unsafe {
            let curw = session_get_curw(self.ptr());
            (!curw.is_null()).then(|| seen(cstr_ptr(&(*(*curw).window()).name)))
        }
    }

    /// The windows the session has been in, most recent first.
    fn last(&mut self) -> Vec<String> {
        unsafe {
            (*self.ptr())
                .lastw
                .iter()
                .map(|&idx| {
                    let wl = winlink_of(self.ptr(), Some(idx));
                    seen(cstr_ptr(&(*(*wl).window()).name))
                })
                .collect()
        }
    }

    /// The indexes the session's windows are linked at.
    fn indexes(&mut self) -> Vec<c_int> {
        unsafe {
            let mut out = Vec::new();
            let mut wl = winlinks_first(&raw mut (*self.ptr()).windows);
            while !wl.is_null() {
                out.push((*wl).idx);
                wl = winlinks_after(wl);
            }
            out
        }
    }
}

impl Drop for Linked {
    fn drop(&mut self) {
        unsafe {
            let s = self.session.ptr();
            session_set_curw(s, null_mut::<winlink>());
            while !(*s).lastw.is_empty() {
                winlink_stack_remove(
                    &raw mut (*s).lastw,
                    winlink_of(s, (*s).lastw.first().copied()),
                );
            }
            while let Some(wl) = (*s)
                .windows
                .values()
                .next()
                .map(|wl| wl.as_ref() as *const winlink as *mut winlink)
            {
                winlink_remove(&raw mut (*s).windows, wl);
            }
        }
    }
}

#[test]
fn a_window_is_linked_into_a_session_at_an_index_of_its_own() {
    let _guard = server();
    let mut linked = Linked::new("attach", 0);
    unsafe {
        let wl = linked.attach("only", 3);
        assert_eq!((*wl).idx, 3);
        assert_eq!((*wl).session(), linked.ptr());
        assert_eq!((*wl).window(), linked.window(0));
        assert!((*wl).window_ref.is_some());
        assert_eq!(linked.indexes(), [3]);
    }
}

/// An index that is already linked is refused, and what comes back with
/// the refusal is the reason, for the caller to hand on.
#[test]
fn an_index_that_is_in_use_is_refused_with_a_reason() {
    let _guard = server();
    let mut linked = Linked::new("attach", 1);
    let mut spare = Window::new(9, "spare", 80, 24);
    unsafe {
        let mut cause = None;
        let wl = session_attach(linked.ptr(), spare.ptr(), 1, &mut cause);
        assert!(wl.is_null());
        assert_eq!(cause.unwrap().to_str().unwrap(), "index in use: 1");
    }
}

#[test]
fn a_session_knows_which_windows_are_linked_into_it() {
    let _guard = server();
    let mut first = Linked::new("first", 1);
    let mut second = Linked::new("second", 1);
    unsafe {
        assert_eq!(session_has(first.ptr(), first.window(0)), 1);
        assert_eq!(session_has(first.ptr(), second.window(0)), 0);
        assert_eq!(session_has(second.ptr(), second.window(0)), 1);
    }
}

/// A session group holding sessions for the length of a test, which is
/// taken apart again at the end of it whatever happened in between — the
/// groups are a global, and a group left behind is the next test's
/// failure. A group the last session leaves is freed by the server
/// itself; one nothing ever joined is taken out here.
struct Group {
    group: *mut session_group,
    name: CString,
    sessions: Vec<*mut session>,
}

impl Group {
    fn new(name: &CStr) -> Group {
        Group {
            group: unsafe { session_group_new(name.as_ptr()) },
            name: name.to_owned(),
            sessions: Vec::new(),
        }
    }

    fn add(&mut self, s: *mut session) {
        unsafe { session_group_add(self.group, s) };
        self.sessions.push(s);
    }

    fn ptr(&self) -> *mut session_group {
        self.group
    }
}

impl Drop for Group {
    fn drop(&mut self) {
        unsafe {
            for s in &self.sessions {
                session_group_remove(*s);
            }
            let _ = session_groups.map().remove(&self.name);
        }
    }
}

/// A group is found by the name it was made with, whichever side of the
/// tree it is on, and a name no group has finds nothing. A group nobody
/// ever joined stays where it is until it is taken out by hand.
#[test]
fn a_group_is_found_by_name_among_the_ones_the_server_holds() {
    let _guard = server();
    let alpha = Group::new(c"alpha");
    let beta = Group::new(c"beta");
    let gamma = Group::new(c"gamma");
    unsafe {
        assert_eq!(session_group_find(c"alpha".as_ptr()), alpha.ptr());
        assert_eq!(session_group_find(c"beta".as_ptr()), beta.ptr());
        assert_eq!(session_group_find(c"gamma".as_ptr()), gamma.ptr());
        assert!(session_group_find(c"delta".as_ptr()).is_null());
        assert_eq!(session_group_count(alpha.ptr()), 0);
    }
}

#[test]
fn a_window_is_linked_when_a_winlink_is_outside_the_session_or_group() {
    let _guard = server();
    let mut linked = Linked::new("linked", 1);
    let mut second = Linked::new("second", 0);
    let second_wl = link(&mut second.session, &mut linked.windows[0], 0);
    unsafe {
        let w = linked.window(0);
        let s = linked.ptr();
        assert_eq!(session_is_linked(s, w), 1);

        let mut group = Group::new(c"group");
        group.add(s);
        group.add(second.ptr());
        assert_eq!(session_group_count(group.ptr()), 2);
        assert_eq!(session_is_linked(s, w), 0);

        let mut outside = Session::new(3, "outside");
        let outside_wl = link(&mut outside, &mut linked.windows[0], 0);
        assert_eq!(session_is_linked(s, w), 1);
        unlink(&mut outside, outside_wl);
    }
    unlink(&mut second.session, second_wl);
}

/// The current window is only moved to one that is there, and moving to
/// the one that is current already is no move at all.
#[test]
fn the_current_window_is_set_and_the_one_before_it_is_remembered() {
    let _guard = server();
    let mut linked = Linked::new("current", 3);
    unsafe {
        let s = linked.ptr();
        assert_eq!(session_set_current(s, null_mut::<winlink>()), -1);
        assert_eq!(session_set_current(s, linked.wl(0)), 1);
        assert_eq!(session_set_current(s, linked.wl(1)), 0);
        assert_eq!(linked.current().as_deref(), Some("w1"));
        assert_eq!(linked.last(), ["w0"]);
        assert_eq!(session_set_current(s, linked.wl(2)), 0);
        assert_eq!(linked.last(), ["w1", "w0"]);
        assert_eq!(session_set_current(s, linked.wl(0)), 0);
        assert_eq!(linked.last(), ["w2", "w1"]);
    }
}

/// Going back is a move to the window on top of the stack, and there is
/// nothing to go back to until the session has been somewhere else.
#[test]
fn a_session_goes_back_to_the_window_it_came_from() {
    let _guard = server();
    let mut linked = Linked::new("last", 2);
    unsafe {
        let s = linked.ptr();
        assert_eq!(session_last(s), -1);
        session_set_current(s, linked.wl(1));
        assert_eq!(session_last(s), 0);
        assert_eq!(linked.current().as_deref(), Some("w0"));
        winlink_stack_push(&raw mut (*s).lastw, session_get_curw(s));
        assert_eq!(session_last(s), 1);
    }
}

#[test]
fn a_window_is_selected_by_its_index() {
    let _guard = server();
    let mut linked = Linked::new("select", 2);
    unsafe {
        let s = linked.ptr();
        assert_eq!(session_select(s, 2), 0);
        assert_eq!(linked.current().as_deref(), Some("w1"));
        assert_eq!(session_select(s, 2), 1);
        assert_eq!(session_select(s, 99), -1);
    }
}

/// The next and previous windows wrap round the ends of the session, and
/// answer nothing at all when the session is nowhere.
#[test]
fn the_next_and_previous_windows_wrap_round() {
    let _guard = server();
    let mut linked = Linked::new("walk", 3);
    unsafe {
        let s = linked.ptr();
        assert_eq!(session_next(s, 0), 0);
        assert_eq!(linked.current().as_deref(), Some("w1"));
        assert_eq!(session_next(s, 0), 0);
        assert_eq!(session_next(s, 0), 0);
        assert_eq!(linked.current().as_deref(), Some("w0"));
        assert_eq!(session_previous(s, 0), 0);
        assert_eq!(linked.current().as_deref(), Some("w2"));
        assert_eq!(session_previous(s, 0), 0);
        assert_eq!(linked.current().as_deref(), Some("w1"));

        let curw = session_get_curw(s);
        session_set_curw(s, null_mut::<winlink>());
        assert_eq!(session_next(s, 0), -1);
        assert_eq!(session_previous(s, 0), -1);
        session_set_curw(s, curw);
    }
}

/// Asked for a window with an alert, the walk goes past the ones without
/// one — and answers nothing when no window has any. What takes an alert
/// off a window is arriving at it, so a window left behind still carries
/// the alert it was given: the second walk back finds `w2` again, since
/// the bell it was given while it was current was never cleared.
#[test]
fn the_walk_can_be_asked_for_windows_with_alerts_only() {
    let _guard = server();
    let mut linked = Linked::new("alerts", 4);
    unsafe {
        let s = linked.ptr();
        assert_eq!(session_next(s, 1), -1);
        assert_eq!(session_previous(s, 1), -1);
        (*linked.wl(2)).flags |= WINLINK_BELL;
        assert_eq!(session_next(s, 1), 0);
        assert_eq!(linked.current().as_deref(), Some("w2"));
        (*linked.wl(2)).flags |= WINLINK_BELL;
        (*linked.wl(0)).flags |= WINLINK_ACTIVITY;
        assert_eq!(session_next(s, 1), 0);
        assert_eq!(linked.current().as_deref(), Some("w0"));
        (*linked.wl(3)).flags |= WINLINK_SILENCE;
        assert_eq!(session_previous(s, 1), 0);
        assert_eq!(linked.current().as_deref(), Some("w3"));
        assert_eq!(session_previous(s, 1), 0);
        assert_eq!(linked.current().as_deref(), Some("w2"));
        assert_eq!(session_previous(s, 1), -1);
    }
}

/// Detaching takes the winlink away and says whether the session has any
/// windows left; when what goes is the current window, the session moves
/// to another one first.
#[test]
fn detaching_a_window_moves_off_it_first() {
    let _guard = server();
    let mut linked = Linked::new("detach", 3);
    unsafe {
        let s = linked.ptr();
        session_set_current(s, linked.wl(1));
        assert_eq!(session_detach(s, linked.wl(1)), 0);
        assert_eq!(linked.current().as_deref(), Some("w0"));
        assert_eq!(linked.indexes(), [1, 3]);
        assert_eq!(session_detach(s, linked.wl(2)), 0);
        assert_eq!(session_detach(s, linked.wl(0)), 1);
        assert!(linked.indexes().is_empty());
        session_set_curw(s, null_mut::<winlink>());
        linked.winlinks.clear();
    }
}

/// Whether the session's lock timer is armed.
unsafe fn locking(s: *mut session) -> bool {
    unsafe { (*s).lock_timer.is_armed() }
}

/// The activity time is either the one the caller hands over or the time
/// now, and setting it sets up the lock timer the first time round.
#[test]
fn activity_is_taken_from_the_caller_or_from_the_clock() {
    let _guard = server();
    let mut fixture = Session::new(1, "activity");
    unsafe {
        let s = fixture.ptr();
        let from = timeval {
            tv_sec: 1234,
            tv_usec: 567,
        };
        session_update_activity(s, &raw const from as *mut timeval);
        assert_eq!((*s).activity_time.tv_sec, 1234);
        assert_eq!((*s).activity_time.tv_usec, 567);
        assert!((*s).lock_timer.is_set());
        assert!(!locking(s));

        session_update_activity(s, null_mut::<timeval>());
        assert!((*s).activity_time.tv_sec > 1234);
        (*s).lock_timer.disarm();
    }
}

/// The lock timer is only armed for a session somebody is attached to, and
/// only when `lock-after-time` is set to something.
#[test]
fn an_attached_session_locks_after_the_time_it_is_given() {
    let _guard = server();
    let mut fixture = Session::new(1, "locking");
    unsafe {
        let s = fixture.ptr();
        options_set_number(options_ptr(&(*s).options), c"lock-after-time".as_ptr(), 60);
        session_update_activity(s, null_mut::<timeval>());
        assert!(!locking(s), "nobody is attached");

        (*s).attached = 1;
        session_update_activity(s, null_mut::<timeval>());
        assert!(locking(s));

        options_set_number(options_ptr(&(*s).options), c"lock-after-time".as_ptr(), 0);
        session_update_activity(s, null_mut::<timeval>());
        assert!(!locking(s), "there is no time to lock after");
        (*s).attached = 0;
        (*s).lock_timer.disarm();
    }
}

/// A session the server has never been in — one with no current window —
/// is not destroyed at all, which is what keeps a session being created
/// from being torn down halfway through.
#[test]
fn a_session_with_no_current_window_is_not_destroyed() {
    let _guard = server();
    let mut created = Created::new();
    unsafe {
        let s = created.session(None, Some(c"halfway"));
        session_destroy(s, 1, c"a test".as_ptr());
        assert_eq!(session_alive(s), 1);
        assert!(session_ref_from_ptr(s).is_some());
    }
}

/// Destroying a session takes it out of the server's tree, unlinks every
/// window it held and leaves queued notifications with strong handles.
#[test]
fn destroying_a_session_unlinks_everything_it_held() {
    let _guard = server();
    let mut created = Created::new();
    let mut first = Window::new(1, "w1", 80, 24);
    let mut second = Window::new(2, "w2", 80, 24);
    unsafe {
        let s = created.session(None, Some(c"doomed"));
        let mut cause = None;
        let wl = session_attach(s, first.ptr(), 1, &mut cause);
        let wl2 = session_attach(s, second.ptr(), 2, &mut cause);
        session_set_curw(s, wl);
        session_set_current(s, wl2);
        assert!(!(*s).lastw.is_empty());
        let weak = session_ref_from_ptr(s).expect("session owner").downgrade();
        session_destroy(s, 1, c"a test".as_ptr());
        assert_eq!(session_alive(s), 0);
        assert!(registered().is_empty());
        assert!(session_get_curw(s).is_null());
        assert!((*s).windows.is_empty());
        assert!(weak.upgrade().is_some());
    }
}

/// The next and previous session are worked out from the sorted list, and
/// both wrap round its ends.
#[test]
fn the_next_and_previous_sessions_wrap_round_the_sorted_list() {
    let _guard = server();
    let mut created = Created::new();
    unsafe {
        let a = created.session(None, Some(c"aaa"));
        let b = created.session(None, Some(c"bbb"));
        let c = created.session(None, Some(c"ccc"));
        let mut crit = sort_criteria_t {
            order: SORT_NAME,
            reversed: 0,
            order_seq: None,
        };
        assert_eq!(session_next_session(a, &crit), b);
        assert_eq!(session_next_session(c, &crit), a);
        assert_eq!(session_previous_session(a, &crit), c);
        assert_eq!(session_previous_session(b, &crit), a);
        crit.reversed = 1;
        assert_eq!(session_next_session(a, &crit), c);
    }
}

/// A session the server does not have has no next and no previous, and
/// neither has anything at all when there are no sessions.
#[test]
fn a_session_the_server_has_given_up_has_no_neighbours() {
    let _guard = server();
    let mut apart = Session::new(1, "apart");
    let mut crit = sort_criteria_t {
        order: SORT_NAME,
        reversed: 0,
        order_seq: None,
    };
    unsafe {
        assert!(session_next_session(apart.ptr(), &crit).is_null());
        assert!(session_previous_session(apart.ptr(), &crit).is_null());
        let mut created = Created::new();
        created.session(None, Some(c"only"));
        assert!(session_next_session(apart.ptr(), &crit).is_null());
        assert!(session_previous_session(apart.ptr(), &crit).is_null());
    }
}

/// A group is made once and found again by its name afterwards; a session
/// joins it once, however many times it is added.
#[test]
fn a_group_is_made_once_and_holds_each_session_once() {
    let _guard = server();
    let mut first = Session::new(1, "one");
    let mut second = Session::new(2, "two");
    unsafe {
        assert!(session_group_find(c"group".as_ptr()).is_null());
        let sg = session_group_new(c"group".as_ptr());
        assert_eq!(session_group_new(c"group".as_ptr()), sg);
        assert_eq!(session_group_find(c"group".as_ptr()), sg);
        assert_eq!(seen(session_group_name(sg)), "group");
        assert_eq!(session_group_count(sg), 0);

        session_group_add(sg, first.ptr());
        session_group_add(sg, first.ptr());
        assert_eq!(session_group_count(sg), 1);
        session_group_add(sg, second.ptr());
        assert_eq!(session_group_count(sg), 2);
        assert_eq!(session_group_contains(first.ptr()), sg);
        assert_eq!(session_group_contains(second.ptr()), sg);

        (*first.ptr()).attached = 2;
        (*second.ptr()).attached = 1;
        assert_eq!(session_group_attached_count(sg), 3);

        session_group_remove(first.ptr());
        assert_eq!(session_group_count(sg), 1);
        assert!(session_group_contains(first.ptr()).is_null());
        session_group_remove(second.ptr());
        assert!(session_group_find(c"group".as_ptr()).is_null());
    }
}

/// Taking a session out of no group at all is nothing, and a session in no
/// group is in no group.
#[test]
fn a_session_that_is_in_no_group_is_left_alone() {
    let _guard = server();
    let mut apart = Session::new(1, "apart");
    unsafe {
        assert!(session_group_contains(apart.ptr()).is_null());
        session_group_remove(apart.ptr());
        session_group_synchronize_to(apart.ptr());
        session_group_synchronize_from(apart.ptr());
        assert!(session_group_contains(apart.ptr()).is_null());
    }
}

/// Every session of a group holds the same windows at the same indexes:
/// synchronising from one of them gives its windows to the rest, and the
/// windows they held before go.
#[test]
fn a_group_is_synchronised_from_one_session_to_the_others() {
    let _guard = server();
    let mut target = Linked::new("target", 2);
    let mut other = Linked::new("other", 1);
    unsafe {
        let mut group = Group::new(c"group");
        group.add(target.ptr());
        group.add(other.ptr());
        assert_eq!(target.indexes(), [1, 2]);
        assert_eq!(other.indexes(), [1]);

        session_group_synchronize_from(target.ptr());
        assert_eq!(other.indexes(), [1, 2]);
        assert_eq!(other.current().as_deref(), Some("w0"));
        assert_eq!(
            seen(cstr_ptr(
                &(*(*winlink_find_by_index(&raw mut (*other.ptr()).windows, 2)).window()).name
            )),
            "w1"
        );
        other.winlinks.clear();
    }
}

/// Synchronising *to* a session takes the windows of the first other
/// session in the group, which is how a session joining one is brought up
/// to date.
#[test]
fn a_session_is_synchronised_to_what_the_rest_of_its_group_holds() {
    let _guard = server();
    let mut holding = Linked::new("holding", 2);
    let mut joining = Linked::new("joining", 1);
    unsafe {
        let mut group = Group::new(c"group");
        group.add(holding.ptr());
        group.add(joining.ptr());

        session_group_synchronize_to(joining.ptr());
        assert_eq!(joining.indexes(), [1, 2]);
        assert_eq!(holding.indexes(), [1, 2]);
        joining.winlinks.clear();
    }
}

/// A session on its own in a group has nothing to synchronise with, and a
/// group whose target holds no windows leaves the others alone.
#[test]
fn a_group_of_one_has_nothing_to_synchronise() {
    let _guard = server();
    let mut alone = Linked::new("alone", 1);
    let mut empty = Linked::new("empty", 0);
    unsafe {
        let mut group = Group::new(c"group");
        group.add(alone.ptr());
        session_group_synchronize_to(alone.ptr());
        session_group_synchronize_from(alone.ptr());
        assert_eq!(alone.indexes(), [1]);

        group.add(empty.ptr());
        session_group_synchronize_from(empty.ptr());
        assert_eq!(alone.indexes(), [1]);
    }
}

/// Renumbering closes the gaps between the indexes, starting at the
/// session's own `base-index`, and keeps the current window current.
#[test]
fn renumbering_closes_the_gaps_between_the_windows() {
    let _guard = server();
    let mut linked = Linked::new("renumber", 0);
    unsafe {
        let s = linked.ptr();
        linked.attach("w0", 1);
        linked.attach("w1", 4);
        linked.attach("w2", 9);
        session_set_curw(s, linked.wl(1));
        session_set_current(s, linked.wl(2));

        session_renumber_windows(s);
        assert_eq!(linked.indexes(), [0, 1, 2]);
        assert_eq!(linked.current().as_deref(), Some("w2"));
        assert_eq!(linked.last(), ["w1"]);

        options_set_number(options_ptr(&(*s).options), c"base-index".as_ptr(), 5);
        session_renumber_windows(s);
        assert_eq!(linked.indexes(), [5, 6, 7]);
        assert_eq!(linked.current().as_deref(), Some("w2"));
        linked.winlinks.clear();
    }
}

/// Every pane of every window in the session is told the theme changed,
/// and a session that is not there at all is nothing to tell.
#[test]
fn a_theme_change_reaches_every_pane_of_the_session() {
    let _guard = server();
    let mut linked = Linked::new("theme", 1);
    let mut first = Pane::new(1, 80, 24, 100);
    let mut second = Pane::new(2, 80, 24, 100);
    unsafe {
        let w = linked.window(0);
        first.hand_to(w);
        second.hand_to(w);
        session_theme_changed(linked.ptr());
        assert_ne!((*first.ptr()).flags & PANE_THEMECHANGED, 0);
        assert_ne!((*second.ptr()).flags & PANE_THEMECHANGED, 0);
        session_theme_changed(null_mut::<session>());
    }
}

/// The history limit of the session is given to every pane in it, and what
/// a pane holds beyond it is collected there and then. Applying the same
/// limit again takes another line off: the collection is entered as soon as
/// the history is *at* the limit, and it never takes less than one line.
#[test]
fn the_history_limit_reaches_every_pane_of_the_session() {
    let _guard = server();
    let mut linked = Linked::new("history", 1);
    let mut pane = Pane::new(1, 80, 24, 100);
    unsafe {
        let w = linked.window(0);
        pane.hand_to(w);
        let gd = screen_grid_ptr(&raw mut (*pane.ptr()).base);
        for _ in 0..20 {
            grid_scroll_history(&mut *gd, 8);
        }
        assert_eq!((*gd).hsize, 20);

        options_set_number(
            options_ptr(&(*linked.ptr()).options),
            c"history-limit".as_ptr(),
            5,
        );
        session_update_history(linked.ptr());
        assert_eq!((*gd).hlimit, 5);
        assert_eq!((*gd).hsize, 5);

        session_update_history(linked.ptr());
        assert_eq!((*gd).hsize, 4);
    }
}

/// Session teardown leaves a deferred strong handle for the next reactor
/// turn, after which the allocation is reclaimed when no other handle
/// remains.
#[test]
fn a_session_is_freed_once_nothing_holds_it() {
    let _guard = server();
    let mut created = Created::new();
    let s = created.session(None, Some(c"doomed"));
    let reference = session_registry_remove(s).expect("session owner");
    let weak = reference.downgrade();
    session_defer_cleanup(reference);
    created.0.clear();
    assert!(weak.upgrade().is_some());
    reactor::current().run_once();
    assert!(weak.upgrade().is_none());
}

/// The lock timer locks the session the moment it fires, unless nobody is
/// attached to it by then.
#[test]
fn the_lock_timer_locks_a_session_somebody_is_attached_to() {
    let _guard = server();
    let mut fixture = Session::new(1, "locked");
    unsafe {
        let s = fixture.ptr();
        session_lock_timer(s);
        (*s).attached = 1;
        session_lock_timer(s);
        (*s).attached = 0;
    }
}

/// With `focus-events` on, moving between windows tells the pane that was
/// in front and the one now in front that the focus moved.
#[test]
fn moving_between_windows_can_carry_the_focus_with_it() {
    let _guard = server();
    let mut linked = Linked::new("focus", 2);
    let mut pane = Pane::new(1, 80, 24, 100);
    unsafe {
        let s = linked.ptr();
        let w = linked.window(0);
        let wp = pane.hand_to(w);
        window_set_active(w, wp);
        (*wp).flags |= PANE_FOCUSED;
        options_set_number(global_options, c"focus-events".as_ptr(), 1);
        assert_eq!(session_set_current(s, linked.wl(1)), 0);
        options_set_number(global_options, c"focus-events".as_ptr(), 0);
        assert_eq!((*wp).flags & PANE_FOCUSED, 0);
        window_set_active(w, null_mut::<window_pane>());
    }
}

/// A session whose current window the target has not got is moved off it
/// first — and then ends up nowhere at all, since what it was moved to is
/// an index the target has not got either, and the window it lands on is
/// looked up by index in the windows it has just been given. Two sessions
/// of a group hold the same indexes in the ordinary way of things, so this
/// is a shape only a test builds.
#[test]
fn synchronising_moves_a_session_off_a_window_the_target_has_not_got() {
    let _guard = server();
    let mut target = Linked::new("target", 0);
    let mut other = Linked::new("other", 0);
    unsafe {
        target.attach("t0", 1);
        target.attach("t1", 2);
        session_set_curw(target.ptr(), target.wl(1));
        other.attach("o0", 7);
        other.attach("o1", 8);
        session_set_curw(other.ptr(), other.wl(0));
        session_set_current(other.ptr(), other.wl(1));

        let mut group = Group::new(c"group");
        group.add(target.ptr());
        group.add(other.ptr());
        session_group_synchronize_from(target.ptr());

        assert_eq!(other.indexes(), [1, 2]);
        assert_eq!(other.current(), None);
        assert!(other.last().is_empty());
        other.winlinks.clear();
    }
}

/// A session with nowhere to be at all lands on the window the target is
/// on, and the windows it has been in that the target also has are kept in
/// the order it visited them.
#[test]
fn synchronising_keeps_the_windows_a_session_has_been_in() {
    let _guard = server();
    let mut target = Linked::new("target", 0);
    let mut other = Linked::new("other", 0);
    unsafe {
        target.attach("t0", 1);
        target.attach("t1", 2);
        session_set_curw(target.ptr(), target.wl(1));
        other.attach("o0", 1);
        other.attach("o1", 2);
        session_set_curw(other.ptr(), other.wl(0));
        session_set_current(other.ptr(), other.wl(1));
        assert_eq!(other.last(), ["o0"]);
        session_set_curw(other.ptr(), null_mut::<winlink>());

        let mut group = Group::new(c"group");
        group.add(target.ptr());
        group.add(other.ptr());
        session_group_synchronize_from(target.ptr());

        assert_eq!(other.indexes(), [1, 2]);
        assert_eq!(other.current().as_deref(), Some("t1"));
        assert_eq!(other.last(), ["t0"]);
        other.winlinks.clear();
    }
}

/// A session with one window it cannot stay on has nowhere to go: the walk
/// back answers that there is nowhere it has been, and the walk to the
/// window before it comes round to the one window it has, which is where
/// it already is. What is left is the walk forward, which comes round to
/// the same place again.
#[test]
fn synchronising_a_session_of_one_window_has_nowhere_to_move_it() {
    let _guard = server();
    let mut target = Linked::new("target", 0);
    let mut other = Linked::new("other", 0);
    unsafe {
        target.attach("t0", 1);
        session_set_curw(target.ptr(), target.wl(0));
        other.attach("o0", 9);
        session_set_curw(other.ptr(), other.wl(0));
        assert!((*other.ptr()).lastw.is_empty());

        let mut group = Group::new(c"group");
        group.add(target.ptr());
        group.add(other.ptr());
        session_group_synchronize_from(target.ptr());

        assert_eq!(other.indexes(), [1]);
        assert_eq!(other.current(), None);
        other.winlinks.clear();
    }
}

/// The marked pane follows its window through a renumbering, and is
/// cleared when the window it was in is not there any more.
#[test]
fn renumbering_carries_the_marked_pane_over() {
    let _guard = server();
    let mut linked = Linked::new("marked", 0);
    unsafe {
        linked.attach("w0", 2);
        linked.attach("w1", 6);
        session_set_curw(linked.ptr(), linked.wl(0));
        marked_pane.set_winlink(linked.wl(1));
        marked_pane.set_session(linked.ptr());
        marked_pane.set_window(linked.window(1));

        session_renumber_windows(linked.ptr());
        assert_eq!(linked.indexes(), [0, 1]);
        assert_eq!((*marked_pane.winlink()).idx, 1);
        assert_eq!(
            seen(cstr_ptr(&(*(*marked_pane.winlink()).window()).name)),
            "w1"
        );
        server_clear_marked();
        linked.winlinks.clear();
    }
}
