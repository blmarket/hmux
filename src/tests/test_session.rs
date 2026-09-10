use super::*;
use crate::WindowPane;
use crate::environ::new_environment_box;
use crate::grid::grid_scroll_history;
use crate::options::OptionsRef;
use crate::sort::{RustSortCriteria, SortCriteria};
use crate::tests::test_fixtures::seen_str;
use crate::tests::test_fixtures::{
    Options, Pane, Session, Window, ensure_reactor, globals, link, unlink, zeroed,
};
use crate::window::PANE_FOCUSED;
use crate::window::window_set_active;
use ::core::ffi::{CStr, c_int};
use ::core::ptr::null_mut;
use ::std::ffi::CString;

#[test]
fn moved_session_payload_cannot_recover_its_previous_owner() {
    let mut original = SessionRef::new(session::default());
    let moved = std::mem::take(unsafe { original.as_session_mut() });
    assert!(session_ref_of(&moved).is_none());
    let replacement = SessionRef::new(moved);
    assert!(
        session_ref_of(unsafe { replacement.as_session() })
            .unwrap()
            .ptr_eq(&replacement)
    );
}

#[test]
fn session_observer_expires_when_its_last_owner_drops() {
    std::thread::spawn(|| {
        let reference = SessionRef::new(session::default());
        let observer = reference.downgrade();
        assert!(session_ref_of(unsafe { reference.as_session() }).is_some());
        drop(reference);
        assert!(observer.upgrade().is_none());
    })
    .join()
    .unwrap();
}

#[test]
fn session_cleanup_survives_thread_local_owner_teardown() {
    thread_local! {
        static LAST_SESSION: std::cell::RefCell<Option<SessionRef>> = const {
            std::cell::RefCell::new(None)
        };
    }
    std::thread::spawn(|| {
        LAST_SESSION.with_borrow_mut(|last| {
            *last = Some(SessionRef::new(session::default()));
        });
    })
    .join()
    .unwrap();
}

/// A turn at the server-wide state these tests reach — the session tree,
/// the session groups, the id the next session is given and the marked
/// pane — starting from empty trees and leaving them empty.
fn server() -> crate::tests::test_fixtures::GlobalsGuard {
    let SESSIONS = SESSIONS_FIELD.get();

    let guard = globals();
    ensure_reactor();
    assert!(SESSIONS.map().is_empty(), "the session tree is not empty");
    assert!(session_groups_empty(), "the session groups are not empty");
    guard
}

/// The names in a tree, in the order it walks them.
unsafe fn walk(head: &sessions_t) -> Vec<String> {
    unsafe {
        head.values()
            .map(|s| seen_str(s.as_session().name.as_deref()))
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
            let reference = SessionRef::create(
                prefix,
                name,
                c"/tmp",
                new_environment_box(),
                Options::session().owned(),
                None,
            );
            let s = reference.as_ptr();
            self.0.push(reference);
            s
        }
    }
}

impl Drop for Created {
    fn drop(&mut self) {
        for reference in &self.0 {
            let s = reference.as_ptr();
            if i32::from(
                (unsafe { s.as_ref() })
                    .and_then(crate::session::session_ref_of)
                    .is_some_and(|session| session.is_registered()),
            ) == 0
            {
                continue;
            }
            session_registry_remove(unsafe { &*s });
        }
    }
}

/// The names of every session the server has.
fn registered() -> Vec<String> {
    let SESSIONS = SESSIONS_FIELD.get();

    let registered = SESSIONS.map();
    unsafe { walk(&registered) }
}

#[test]
fn a_session_is_created_with_the_name_it_is_given() {
    let _guard = server();
    let mut created = Created::new();
    unsafe {
        let was = next_session_id().unwrap();
        let s = created.session(None, Some(c"named"));
        assert_eq!(seen_str((*s).name.as_deref()), "named");
        assert_eq!((*s).id, was);
        assert_eq!(next_session_id().unwrap(), was + 1);
        assert_eq!(seen_str((*s).cwd.as_deref()), "/tmp");
        assert!(session_ref_of(&*s).is_some());
        assert_eq!((*s).flags, 0);
        assert!((*s).tio.is_none());
        assert!((&*s).curw().is_none());
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
            seen_str((*first).name.as_deref()),
            format!("pre-{}", (*first).id)
        );
        assert_eq!(
            seen_str((*second).name.as_deref()),
            format!("{}", (*second).id)
        );
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
        let next_id = next_session_id().unwrap();
        let taken_name = CString::new(format!("pre-{}", next_id + 1)).expect("no NUL");
        let held = created.session(None, Some(taken_name.as_c_str()));
        let next = created.session(Some(c"pre"), None);
        assert_eq!(
            seen_str((*held).name.as_deref()),
            format!("pre-{}", (*held).id + 1)
        );
        assert_eq!(
            seen_str((*next).name.as_deref()),
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
        let s = SessionRef::create(
            None,
            Some(c"tio"),
            c"/tmp",
            new_environment_box(),
            Options::session().owned(),
            Some(&tio),
        );
        created.0.push(s.clone());
        tio.c_iflag = 0;
        assert_eq!(s.as_session().tio.as_ref().unwrap().c_iflag, 0x2d5);
    }
}

#[test]
fn a_session_is_alive_while_the_server_holds_it() {
    let _guard = server();
    let apart = Session::new(1, "apart");
    let reference = unsafe {
        SessionRef::create(
            None,
            Some(c"held"),
            c"/tmp",
            new_environment_box(),
            Options::session().owned(),
            None,
        )
    };
    assert!(reference.is_registered());
    assert!(!apart.handle().is_registered());
    session_registry_remove(unsafe { reference.as_session() });
    assert!(!reference.is_registered());
    assert_eq!(reference.name().as_deref(), Some(c"held"));
}

#[test]
fn a_session_is_found_by_name_and_by_id() {
    let _guard = server();
    let mut created = Created::new();
    unsafe {
        let s = created.session(None, Some(c"findable"));
        let id = (*s).id;
        assert!(SessionRef::find(c"findable").is_some_and(|found| found.as_ptr() == s));
        assert!(SessionRef::find(c"nonesuch").is_none());
        assert!(SessionRef::find_by_id(id).is_some_and(|found| found.as_ptr() == s));
        assert!(SessionRef::find_by_id(id + 1000).is_none());
        let by_str = CString::new(format!("${id}")).expect("no NUL");
        assert!(
            SessionRef::find_by_id_str(by_str.as_c_str()).is_some_and(|found| found.as_ptr() == s)
        );
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
        assert!(SessionRef::find_by_id_str(plain.as_c_str()).is_none());
        assert!(SessionRef::find_by_id_str(c"$").is_none());
        assert!(SessionRef::find_by_id_str(c"$x").is_none());
        assert!(SessionRef::find_by_id_str(c"$-1").is_none());
        assert!(SessionRef::find_by_id_str(c"$99999999999").is_none());
        assert!(SessionRef::find_by_id_str(c"").is_none());
    }
}

/// Removing a session from the live registry does not reclaim it while a
/// strong handle remains, and the weak handle stops upgrading after the
/// last strong handle goes away.
#[test]
fn a_session_is_kept_while_anything_holds_a_handle_to_it() {
    let SESSIONS = SESSIONS_FIELD.get();

    let _guard = server();
    let mut created = Created::new();
    unsafe {
        let s = created.session(None, Some(c"counted"));
        let reference = SessionRef::find_by_id((*s).id).expect("session owner");
        let weak = reference.downgrade();
        let name = (*s)
            .name
            .as_deref()
            .expect("a created session has a name")
            .to_owned();
        session_registry_remove(&*s);
        assert_eq!(seen_str((*s).name.as_deref()), "counted");
        assert!(weak.upgrade().is_some());
        SESSIONS.map().insert(name, reference);
        session_registry_remove(&*s);
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
    winlinks: Vec<crate::window::WinlinkRef>,
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
        let current = linked.winlinks.first().map(|link| link.index());
        unsafe { linked.session.handle().clone().as_session_mut().curw_idx = current };

        linked
    }

    /// Links a new window in at `idx` through the real `session_attach`.
    /// The window's id is one nothing else has: the server tells two
    /// windows apart by it, and two sessions holding windows of the same
    /// id look to it like two sessions holding one window.
    fn attach(&mut self, name: &str, idx: c_int) -> crate::window::WinlinkRef {
        let id = crate::server::server_proc.with(|state| {
            let id = state.test_next_window_id.get();
            state.test_next_window_id.set(id + 1);
            id
        }) as u_int;
        let w = Window::new(id, name, 80, 24);
        let session = self.handle().clone();
        let mut cause = None;
        let index = unsafe { session.attach(w.reference(), idx, &mut cause) }
            .expect("the fixture index is available");
        let link =
            crate::window::WinlinkRef::new(session, index).expect("the fixture link was attached");
        self.windows.push(w);
        self.winlinks.push(link.clone());
        link
    }

    fn ptr(&mut self) -> *mut session {
        self.session.ptr()
    }

    fn handle(&self) -> &SessionRef {
        self.session.handle()
    }

    fn window(&mut self, i: usize) -> *mut window {
        self.windows[i].ptr()
    }

    fn wl(&self, i: usize) -> crate::window::WinlinkRef {
        self.winlinks[i].clone()
    }

    /// Which window is current, by name.
    fn current(&self) -> Option<String> {
        {
            let link = self.handle().curw()?;
            let window = link.window()?;
            Some(
                window
                    .window_name()
                    .as_deref()?
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    }

    /// The windows the session has been in, most recent first.
    fn last(&self) -> Vec<String> {
        unsafe {
            let session = self.handle().as_session();
            session
                .lastw
                .iter()
                .map(|index| {
                    let link = session
                        .windows
                        .get(index)
                        .expect("a stacked link belongs to the session");
                    link.window_handle()
                        .expect("a fixture link has a window")
                        .window_name()
                        .as_deref()
                        .map_or(String::new(), |name| name.to_string_lossy().into_owned())
                })
                .collect()
        }
    }

    /// The indexes the session's windows are linked at.
    fn indexes(&self) -> Vec<c_int> {
        unsafe { self.handle().as_session().windows.keys().copied().collect() }
    }
}

impl Drop for Linked {
    fn drop(&mut self) {
        unsafe {
            let mut owner = self.handle().clone();
            owner.as_session_mut().curw_idx = None;
            while let Some(index) = owner.as_session().lastw.first().copied() {
                let session = owner.as_session_mut();
                winlink_stack_remove(
                    &mut session.lastw,
                    session.windows.get_mut(&index).map(Box::as_mut),
                );
            }
            while let Some(index) = owner.as_session().windows.keys().next().copied() {
                winlink_remove(&mut owner.as_session_mut().windows, index);
            }
        }
    }
}

#[test]
fn a_window_is_linked_into_a_session_at_an_index_of_its_own() {
    let _guard = server();
    let mut linked = Linked::new("attach", 0);
    {
        let link = linked.attach("only", 3);
        assert_eq!(link.index(), 3);
        assert!(link.session().ptr_eq(linked.handle()));
        assert!(
            link.get()
                .unwrap()
                .window_handle()
                .unwrap()
                .ptr_eq(linked.windows[0].handle())
        );
        assert_eq!(linked.indexes(), [3]);
    }
}

/// An index that is already linked is refused, and what comes back with
/// the refusal is the reason, for the caller to hand on.
#[test]
fn an_index_that_is_in_use_is_refused_with_a_reason() {
    let _guard = server();
    let linked = Linked::new("attach", 1);
    let spare = Window::new(9, "spare", 80, 24);
    unsafe {
        let mut cause = None;
        let index = linked.handle().attach(spare.reference(), 1, &mut cause);
        assert!(index.is_none());
        assert_eq!(cause.unwrap().to_str().unwrap(), "index in use: 1");
    }
}

#[test]
fn a_session_knows_which_windows_are_linked_into_it() {
    let _guard = server();
    let first = Linked::new("first", 1);
    let second = Linked::new("second", 1);
    assert!(first.handle().has(first.windows[0].handle()));
    assert!(!first.handle().has(second.windows[0].handle()));
    assert!(second.handle().has(second.windows[0].handle()));
}

/// A session group holding sessions for the length of a test, which is
/// taken apart again at the end of it whatever happened in between — the
/// groups are a global, and a group left behind is the next test's
/// failure. A group the last session leaves is freed by the server
/// itself; one nothing ever joined is taken out here.
struct Group {
    name: CString,
    sessions: Vec<SessionRef>,
}

impl Group {
    fn new(name: &CStr) -> Group {
        session_group_ensure(name);
        Group {
            name: name.to_owned(),
            sessions: Vec::new(),
        }
    }

    fn add(&mut self, s: SessionRef) {
        s.join_group(&self.name);
        self.sessions.push(s);
    }

    fn count(&self) -> u_int {
        with_session_group_named(&self.name, session_group_count).expect("the group exists")
    }
}

impl Drop for Group {
    fn drop(&mut self) {
        for member in &self.sessions {
            member.leave_group();
        }
        session_group_registry_remove(&self.name);
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
    for (name, group) in [(c"alpha", &alpha), (c"beta", &beta), (c"gamma", &gamma)] {
        assert_eq!(
            with_session_group_named(name, |group| session_group_name(group).to_owned()),
            Some(group.name.clone())
        );
    }
    assert!(with_session_group_named(c"delta", |_| ()).is_none());
    assert_eq!(alpha.count(), 0);
}

#[test]
fn a_window_is_linked_when_a_winlink_is_outside_the_session_or_group() {
    let _guard = server();
    let mut linked = Linked::new("linked", 1);
    let mut second = Linked::new("second", 0);
    let second_wl = link(&mut second.session, &mut linked.windows[0], 0);
    {
        assert!(linked.handle().is_linked(linked.windows[0].handle()));

        let mut group = Group::new(c"group");
        group.add(linked.session.reference());
        group.add(second.session.reference());
        assert_eq!(group.count(), 2);
        assert!(!linked.handle().is_linked(linked.windows[0].handle()));

        let mut outside = Session::new(3, "outside");
        let outside_wl = link(&mut outside, &mut linked.windows[0], 0);
        assert!(linked.handle().is_linked(linked.windows[0].handle()));
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
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .set_current(None),
            -1
        );
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .set_current(Some(99)),
            -1
        );
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .set_current(Some(linked.wl(0).index())),
            1
        );
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .set_current(Some(linked.wl(1).index())),
            0
        );
        assert_eq!(linked.current().as_deref(), Some("w1"));
        assert_eq!(linked.last(), ["w0"]);
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .set_current(Some(linked.wl(2).index())),
            0
        );
        assert_eq!(linked.last(), ["w1", "w0"]);
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .set_current(Some(linked.wl(0).index())),
            0
        );
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
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .last(),
            -1
        );
        crate::session::session_ref_of(&mut *s)
            .expect("session owner")
            .set_current(Some(linked.wl(1).index()));
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .last(),
            0
        );
        assert_eq!(linked.current().as_deref(), Some("w0"));
        {
            let session = &mut *s;
            let current = session
                .curw_idx
                .and_then(|index| session.windows.get_mut(&index));
            winlink_stack_push(&mut session.lastw, current.map(|link| &mut **link));
        }
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .last(),
            1
        );
    }
}

#[test]
fn a_window_is_selected_by_its_index() {
    let _guard = server();
    let mut linked = Linked::new("select", 2);
    unsafe {
        let s = linked.ptr();
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .select(2),
            0
        );
        assert_eq!(linked.current().as_deref(), Some("w1"));
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .select(2),
            1
        );
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .select(99),
            -1
        );
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
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .next(0),
            0
        );
        assert_eq!(linked.current().as_deref(), Some("w1"));
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .next(0),
            0
        );
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .next(0),
            0
        );
        assert_eq!(linked.current().as_deref(), Some("w0"));
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .previous(0),
            0
        );
        assert_eq!(linked.current().as_deref(), Some("w2"));
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .previous(0),
            0
        );
        assert_eq!(linked.current().as_deref(), Some("w1"));

        let current = (*s).curw_idx.take();
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .next(0),
            -1
        );
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .previous(0),
            -1
        );
        (*s).curw_idx = current;
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
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .next(1),
            -1
        );
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .previous(1),
            -1
        );
        linked.wl(2).get_mut().unwrap().flags |= WINLINK_BELL;
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .next(1),
            0
        );
        assert_eq!(linked.current().as_deref(), Some("w2"));
        linked.wl(2).get_mut().unwrap().flags |= WINLINK_BELL;
        linked.wl(0).get_mut().unwrap().flags |= WINLINK_ACTIVITY;
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .next(1),
            0
        );
        assert_eq!(linked.current().as_deref(), Some("w0"));
        linked.wl(3).get_mut().unwrap().flags |= WINLINK_SILENCE;
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .previous(1),
            0
        );
        assert_eq!(linked.current().as_deref(), Some("w3"));
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .previous(1),
            0
        );
        assert_eq!(linked.current().as_deref(), Some("w2"));
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .previous(1),
            -1
        );
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
        crate::session::session_ref_of(&mut *s)
            .expect("session owner")
            .set_current(Some(linked.wl(1).index()));
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .detach(linked.wl(1).index()),
            0
        );
        assert_eq!(linked.current().as_deref(), Some("w0"));
        assert_eq!(linked.indexes(), [1, 3]);
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .detach(2),
            0
        );
        assert_eq!(linked.current().as_deref(), Some("w0"));
        assert_eq!(linked.indexes(), [1, 3]);
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .detach(linked.wl(2).index()),
            0
        );
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .detach(linked.wl(0).index()),
            1
        );
        assert!(linked.indexes().is_empty());
        crate::session::session_ref_of(&mut *s)
            .expect("session owner")
            .set_curw(null_mut::<winlink>().as_ref());
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
        crate::session::session_ref_of(&mut *s)
            .expect("session owner")
            .update_activity(Some(&from));
        assert_eq!((*s).activity_time.tv_sec, 1234);
        assert_eq!((*s).activity_time.tv_usec, 567);
        assert!((*s).lock_timer.is_set());
        assert!(!locking(s));

        crate::session::session_ref_of(&mut *s)
            .expect("session owner")
            .update_activity(None);
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
        ((*s).options_ref()).set_number(c"lock-after-time", 60);
        crate::session::session_ref_of(&mut *s)
            .expect("session owner")
            .update_activity(None);
        assert!(!locking(s), "nobody is attached");

        (*s).attached = 1;
        crate::session::session_ref_of(&mut *s)
            .expect("session owner")
            .update_activity(None);
        assert!(locking(s));

        ((*s).options_ref()).set_number(c"lock-after-time", 0);
        crate::session::session_ref_of(&mut *s)
            .expect("session owner")
            .update_activity(None);
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
        crate::session::session_ref_of(&mut *s)
            .expect("session owner")
            .destroy(1, c"a test");
        assert_eq!(
            i32::from(
                (s.as_ref())
                    .and_then(crate::session::session_ref_of)
                    .is_some_and(|session| session.is_registered())
            ),
            1
        );
        assert!(session_ref_of(&*s).is_some());
    }
}

/// Destroying a session takes it out of the server's tree, unlinks every
/// window it held and leaves queued notifications with strong handles.
#[test]
fn destroying_a_session_unlinks_everything_it_held() {
    let _guard = server();
    let mut created = Created::new();
    let first = Window::new(1, "w1", 80, 24);
    let second = Window::new(2, "w2", 80, 24);
    unsafe {
        let s = created.session(None, Some(c"doomed"));
        let mut cause = None;
        let wl = crate::session::session_ref_of(&mut *s)
            .expect("session owner")
            .attach(first.reference(), 1, &mut cause);
        let wl2 = crate::session::session_ref_of(&mut *s)
            .expect("session owner")
            .attach(second.reference(), 2, &mut cause);
        (*s).curw_idx = wl;
        crate::session::session_ref_of(&mut *s)
            .expect("session owner")
            .set_current(wl2);
        assert!(!(*s).lastw.is_empty());
        let weak = session_ref_of(&*s).expect("session owner").downgrade();
        crate::session::session_ref_of(&mut *s)
            .expect("session owner")
            .destroy(1, c"a test");
        assert_eq!(
            i32::from(
                (s.as_ref())
                    .and_then(crate::session::session_ref_of)
                    .is_some_and(|session| session.is_registered())
            ),
            0
        );
        assert!(registered().is_empty());
        assert!((&*s).curw().is_none());
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
        let mut crit = RustSortCriteria::new(SORT_NAME, false);
        assert!(
            crate::session::session_ref_of(&*a)
                .and_then(|session| session.next_session(&crit))
                .is_some_and(|s| s.as_ptr() == b)
        );
        assert!(
            crate::session::session_ref_of(&*c)
                .and_then(|session| session.next_session(&crit))
                .is_some_and(|s| s.as_ptr() == a)
        );
        assert!(
            crate::session::session_ref_of(&*a)
                .and_then(|session| session.previous_session(&crit))
                .is_some_and(|s| s.as_ptr() == c)
        );
        assert!(
            crate::session::session_ref_of(&*b)
                .and_then(|session| session.previous_session(&crit))
                .is_some_and(|s| s.as_ptr() == a)
        );
        crit.set_reversed(true);
        assert!(
            crate::session::session_ref_of(&*a)
                .and_then(|session| session.next_session(&crit))
                .is_some_and(|s| s.as_ptr() == c)
        );
    }
}

/// A session the server does not have has no next and no previous, and
/// neither has anything at all when there are no sessions.
#[test]
fn a_session_the_server_has_given_up_has_no_neighbours() {
    let _guard = server();
    let mut apart = Session::new(1, "apart");
    let crit = RustSortCriteria::new(SORT_NAME, false);
    unsafe {
        assert!(
            crate::session::session_ref_of(&*apart.ptr())
                .and_then(|session| session.next_session(&crit))
                .is_none()
        );
        assert!(
            crate::session::session_ref_of(&*apart.ptr())
                .and_then(|session| session.previous_session(&crit))
                .is_none()
        );
        let mut created = Created::new();
        created.session(None, Some(c"only"));
        assert!(
            crate::session::session_ref_of(&*apart.ptr())
                .and_then(|session| session.next_session(&crit))
                .is_none()
        );
        assert!(
            crate::session::session_ref_of(&*apart.ptr())
                .and_then(|session| session.previous_session(&crit))
                .is_none()
        );
    }
}

/// A group is made once and found again by its name afterwards; a session
/// joins it once, however many times it is added.
#[test]
fn a_group_is_made_once_and_holds_each_session_once() {
    let _guard = server();
    let first = Session::new(1, "one");
    let second = Session::new(2, "two");
    let mut first = first.reference();
    let mut second = second.reference();
    assert!(with_session_group_named(c"group", |_| ()).is_none());
    let group = Group::new(c"group");
    assert_eq!(group.count(), 0);
    unsafe {
        first.join_group(&group.name);
        first.join_group(&group.name);
        session_group_ensure(&group.name);
        assert_eq!(group.count(), 1);
        second.join_group(&group.name);
        assert_eq!(group.count(), 2);
        for member in [&first, &second] {
            assert_eq!(
                member.with_group(|group| session_group_name(group).to_owned()),
                Some(group.name.clone())
            );
        }
        first.as_session_mut().attached = 2;
        second.as_session_mut().attached = 1;
        assert_eq!(
            with_session_group_named(&group.name, session_group_attached_count),
            Some(3)
        );
        first.leave_group();
        assert_eq!(group.count(), 1);
        assert!(first.with_group(|_| ()).is_none());
        second.leave_group();
        assert!(with_session_group_named(&group.name, |_| ()).is_none());
    }
}

#[test]
fn safe_group_walk_survives_members_leaving_the_group() {
    let _guard = server();
    let first = Session::new(1, "first");
    let second = Session::new(2, "second");
    let mut group = Group::new(c"snapshot");
    group.add(second.reference());
    group.add(first.reference());
    let first = first.reference();
    let second = second.reference();
    {
        let mut walk = first.group_walk_safe().unwrap();
        let second_member = walk.next().unwrap();
        assert!(second_member.ptr_eq(&second));
        second.leave_group();
        let first_member = walk.next().unwrap();
        assert!(first_member.ptr_eq(&first));
        first.leave_group();
        assert!(first.with_group(|_| ()).is_none());
        assert!(with_session_group_named(c"snapshot", |_| ()).is_none());
        assert!(walk.next().is_none());
        assert_eq!(second_member.name().as_deref(), Some(c"second"));
        assert_eq!(first_member.name().as_deref(), Some(c"first"));
    }
}

/// Taking a session out of no group at all is nothing, and a session in no
/// group is in no group.
#[test]
fn a_session_that_is_in_no_group_is_left_alone() {
    let _guard = server();
    let mut apart = Session::new(1, "apart");
    unsafe {
        assert!(apart.reference().with_group(|_| ()).is_none());
        if let Some(session) = crate::session::session_ref_of(&mut *apart.ptr()) {
            session.leave_group();
        };
        if let Some(session) = crate::session::session_ref_of(&mut *apart.ptr()) {
            session.synchronize_group_to();
        };
        if let Some(session) = crate::session::session_ref_of(&mut *apart.ptr()) {
            session.synchronize_group_from();
        };
        assert!(apart.reference().with_group(|_| ()).is_none());
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
        group.add(target.session.reference());
        group.add(other.session.reference());
        assert_eq!(target.indexes(), [1, 2]);
        assert_eq!(other.indexes(), [1]);

        if let Some(session) = crate::session::session_ref_of(&mut *target.ptr()) {
            session.synchronize_group_from();
        };
        assert_eq!(other.indexes(), [1, 2]);
        assert_eq!(other.current().as_deref(), Some("w0"));
        assert_eq!(
            other
                .session
                .handle()
                .as_session()
                .windows
                .get(&2)
                .and_then(|link| link.window_handle())
                .and_then(|window| window.window_name())
                .as_deref(),
            Some(c"w1")
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
    let holding = Linked::new("holding", 2);
    let mut joining = Linked::new("joining", 1);
    unsafe {
        let mut group = Group::new(c"group");
        group.add(holding.session.reference());
        group.add(joining.session.reference());

        if let Some(session) = crate::session::session_ref_of(&mut *joining.ptr()) {
            session.synchronize_group_to();
        };
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
        group.add(alone.session.reference());
        if let Some(session) = crate::session::session_ref_of(&mut *alone.ptr()) {
            session.synchronize_group_to();
        };
        if let Some(session) = crate::session::session_ref_of(&mut *alone.ptr()) {
            session.synchronize_group_from();
        };
        assert_eq!(alone.indexes(), [1]);

        group.add(empty.session.reference());
        if let Some(session) = crate::session::session_ref_of(&mut *empty.ptr()) {
            session.synchronize_group_from();
        };
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
        crate::session::session_ref_of(&mut *s)
            .expect("session owner")
            .set_curw(linked.wl(1).get());
        crate::session::session_ref_of(&mut *s)
            .expect("session owner")
            .set_current(Some(linked.wl(2).index()));

        crate::session::session_ref_of(&mut *s)
            .expect("session owner")
            .renumber_windows();
        assert_eq!(linked.indexes(), [0, 1, 2]);
        assert_eq!(linked.current().as_deref(), Some("w2"));
        assert_eq!(linked.last(), ["w1"]);

        ((*s).options_ref()).set_number(c"base-index", 5);
        crate::session::session_ref_of(&mut *s)
            .expect("session owner")
            .renumber_windows();
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
        if let Some(session) = linked.ptr().as_mut() {
            crate::session::session_ref_of(session)
                .expect("session owner")
                .theme_changed();
        };
        assert_ne!(*(*first.ptr()).flags() & PANE_THEMECHANGED, 0);
        assert_ne!(*(*second.ptr()).flags() & PANE_THEMECHANGED, 0);
        ();
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
        let gd = RustScreen::grid_mut((*pane.ptr()).base_mut());
        for _ in 0..20 {
            grid_scroll_history(&mut *gd, 8);
        }
        assert_eq!((*gd).hsize, 20);

        (*(*linked.ptr()).options_ref()).set_number(c"history-limit", 5);
        linked.handle().update_history();
        assert_eq!((*gd).hlimit, 5);
        assert_eq!((*gd).hsize, 5);

        linked.handle().update_history();
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
    let reference = session_registry_remove(unsafe { &*s }).expect("session owner");
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
        session_lock_timer(&mut *s);
        (*s).attached = 1;
        session_lock_timer(&mut *s);
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
        window_set_active(&mut *w, Some(&*wp));
        *(*wp).flags_mut() |= PANE_FOCUSED;
        (global_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .set_number(c"focus-events", 1);
        assert_eq!(
            crate::session::session_ref_of(&mut *s)
                .expect("session owner")
                .set_current(Some(linked.wl(1).index())),
            0
        );
        (global_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .set_number(c"focus-events", 0);
        assert_eq!(*(*wp).flags() & PANE_FOCUSED, 0);
        window_set_active(&mut *w, None::<&crate::types::window_pane>);
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
        target.handle().set_curw(target.wl(1).get());
        other.attach("o0", 7);
        other.attach("o1", 8);
        other.handle().set_curw(other.wl(0).get());
        other.handle().set_current(Some(other.wl(1).index()));

        let mut group = Group::new(c"group");
        group.add(target.session.reference());
        group.add(other.session.reference());
        if let Some(session) = crate::session::session_ref_of(&mut *target.ptr()) {
            session.synchronize_group_from();
        };

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
        target.handle().set_curw(target.wl(1).get());
        other.attach("o0", 1);
        other.attach("o1", 2);
        other.handle().set_curw(other.wl(0).get());
        other.handle().set_current(Some(other.wl(1).index()));
        assert_eq!(other.last(), ["o0"]);
        other.handle().set_curw(null_mut::<winlink>().as_ref());

        let mut group = Group::new(c"group");
        group.add(target.session.reference());
        group.add(other.session.reference());
        if let Some(session) = crate::session::session_ref_of(&mut *target.ptr()) {
            session.synchronize_group_from();
        };

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
        target.handle().set_curw(target.wl(0).get());
        other.attach("o0", 9);
        other.handle().set_curw(other.wl(0).get());
        assert!((*other.ptr()).lastw.is_empty());

        let mut group = Group::new(c"group");
        group.add(target.session.reference());
        group.add(other.session.reference());
        if let Some(session) = crate::session::session_ref_of(&mut *target.ptr()) {
            session.synchronize_group_from();
        };

        assert_eq!(other.indexes(), [1]);
        assert_eq!(other.current(), None);
        other.winlinks.clear();
    }
}

#[test]
fn renumbering_does_not_retarget_another_sessions_mark_at_the_same_index() {
    let _guard = server();
    let mut renumbered = Linked::new("renumbered", 0);
    let mut marked = Linked::new("marked", 0);
    unsafe {
        renumbered.attach("renumbered-window", 6);
        marked.attach("marked-window", 6);
        marked_pane.with_mut(|current| current.set_session(marked.ptr().as_ref()));
        marked_pane.with_mut(|current| current.set_winlink(marked.wl(0).get()));
        marked_pane.with_mut(|current| current.set_window_ref(Some(marked.windows[0].handle())));
        let marked_window = marked_pane.get().window().unwrap();

        renumbered.handle().renumber_windows();

        assert_eq!(renumbered.indexes(), [0]);
        assert_eq!(marked_pane.get().wl_idx, Some(6));
        assert!(marked_pane.get().session().unwrap().ptr_eq(marked.handle()));
        assert!(marked_pane.get().window().unwrap().ptr_eq(&marked_window));
        server_clear_marked();
        renumbered.winlinks.clear();
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
        linked.handle().set_curw(linked.wl(0).get());
        marked_pane.with_mut(|current| current.set_winlink(linked.wl(1).get()));
        marked_pane.with_mut(|current| current.set_session(linked.ptr().as_ref()));
        marked_pane.with_mut(|current| current.set_window_ref(Some(linked.windows[1].handle())));

        linked.handle().renumber_windows();
        assert_eq!(linked.indexes(), [0, 1]);
        let marked = marked_pane.get().winlink_ref().unwrap();
        let link = marked.get().unwrap();
        assert_eq!(link.idx, 1);
        assert_eq!(
            link.window_handle()
                .unwrap()
                .window_name()
                .as_deref()
                .unwrap(),
            c"w1"
        );
        server_clear_marked();
        linked.winlinks.clear();
    }
}
use crate::screen::RustScreen;

#[test]
fn session_id_exhaustion_prevents_reuse_and_partial_registration() {
    let _guard = server();
    let mut created = Created::new();
    NEXT_SESSION_ID.set(Some(u_int::MAX));
    let last = created.session(None, Some(c"last"));
    assert_eq!(unsafe { (*last).id }, u_int::MAX);
    assert_eq!(next_session_id(), None);
    unsafe {
        let mut format = crate::format::format_create(None, None, 0, 0);
        assert_eq!(
            crate::format::format_expand(&mut format, c"#{next_session_id}").as_c_str(),
            c""
        );
    }
    for _ in 0..2 {
        let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            created.session(None, Some(c"rejected"));
        }));
        assert!(failed.is_err());
        assert_eq!(registered(), ["last"]);
        assert_eq!(next_session_id(), None);
    }
    std::thread::spawn(|| assert_eq!(next_session_id(), Some(0)))
        .join()
        .unwrap();
}

#[test]
fn session_id_exhaustion_while_skipping_a_name_leaves_no_partial_session() {
    let _guard = server();
    let mut created = Created::new();
    NEXT_SESSION_ID.set(Some(u_int::MAX - 1));
    let name = CString::new(format!("pre-{}", u_int::MAX)).unwrap();
    created.session(None, Some(&name));
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        created.session(Some(c"pre"), None);
    }));
    assert!(failed.is_err());
    assert_eq!(registered(), [name.to_str().unwrap()]);
    assert_eq!(next_session_id(), None);
}

#[test]
fn session_group_registry_is_thread_local_and_observes_sessions_weakly() {
    std::thread::spawn(|| {
        let reference = SessionRef::new(session::default());
        let observer = reference.downgrade();
        session_group_ensure(c"isolated");
        reference.join_group(c"isolated");
        assert_eq!(
            with_session_group_named(c"isolated", session_group_count),
            Some(1)
        );
        std::thread::spawn(|| {
            assert!(session_groups_empty());
            assert!(with_session_group_named(c"isolated", |_| ()).is_none());
            session_group_ensure(c"isolated");
            session_group_registry_remove(c"isolated");
            assert!(session_groups_empty());
        })
        .join()
        .unwrap();
        assert!(with_session_group_named(c"isolated", |_| ()).is_some());
        drop(reference);
        assert!(observer.upgrade().is_none());
        assert_eq!(
            with_session_group_named(c"isolated", session_group_count),
            Some(0)
        );
        session_group_registry_remove(c"isolated");
        assert!(session_groups_empty());
    })
    .join()
    .unwrap();
}

#[test]
fn session_group_registry_teardown_can_precede_a_retained_session() {
    thread_local! {
        static LAST_SESSION: std::cell::RefCell<Option<SessionRef>> = const { std::cell::RefCell::new(None) };
    }
    std::thread::spawn(|| {
        LAST_SESSION.with_borrow_mut(|last| {
            let reference = SessionRef::new(session::default());
            session_group_ensure(c"retained");
            reference.join_group(c"retained");
            *last = Some(reference);
        });
    })
    .join()
    .unwrap();
}

#[test]
fn target_session_observation_uses_owners_without_borrowing_the_payload() {
    let reference = SessionRef::new(session::default());
    let mut target = cmd_find_state::default();
    target.set_session_ref(Some(&reference));
    assert!(target.session().unwrap().ptr_eq(&reference));
    drop(reference);
    assert!(target.session().is_none());
}

pub(crate) fn session_groups_empty() -> bool {
    SESSION_GROUPS.with_borrow(|groups| groups.is_empty())
}

pub(crate) fn session_registry_clear() {
    let SESSIONS = SESSIONS_FIELD.get();

    SESSIONS.map().clear();
}

/// A session of `name` that is not in the server's registry and has no
/// windows, for a test that wants one to hand to the function under test
/// rather than one the server will run.
pub(crate) fn session_new_detached(
    id: u_int,
    name: CString,
    cwd: CString,
    oo: crate::options::RustOptionsRef,
    env: Box<RustEnvironment>,
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

#[test]
fn session_property_snapshots_survive_changes_through_another_handle() {
    let _guard = server();
    let fixture = Session::new(17, "before");
    let session = fixture.reference();
    let other = session.clone();
    unsafe { session.set_cwd(c"/before".to_owned()) };
    let name = session.name();
    let cwd = session.cwd();
    let options = session.options();
    unsafe {
        other.rename(c"after".to_owned());
        other.set_cwd(c"/after".to_owned());
        other.add_attached();
        other.set_alerted(true);
        other.set_activity_time(timeval {
            tv_sec: 123,
            tv_usec: 456,
        });
    }
    assert_eq!(name.as_deref(), Some(c"before"));
    assert_eq!(cwd.as_deref(), Some(c"/before"));
    assert_eq!(session.name().as_deref(), Some(c"after"));
    assert_eq!(session.cwd().as_deref(), Some(c"/after"));
    assert_eq!(session.id(), 17);
    assert_eq!(session.attached(), 1);
    assert!(session.alerted());
    assert_eq!(session.activity_time().tv_sec, 123);
    assert_eq!(session.activity_time().tv_usec, 456);
    assert!(options.ptr_eq(&session.options()));
}

#[test]
fn a_current_window_snapshot_survives_unlinking_its_session_link() {
    let _guard = server();
    let linked = Linked::new("linked", 1);
    let session = linked.handle();
    let link = session.curw().expect("the session has a current link");
    let window = session
        .current_window()
        .expect("the session has a current window");
    assert!(link.window().unwrap().ptr_eq(&window));
    assert_eq!(unsafe { session.detach(link.index()) }, 1);
    assert!(session.curw().is_none());
    assert!(session.current_window().is_none());
    assert!(link.window().is_none());
    assert!(link.key().is_none());
    assert!(unsafe { link.clone().set_window(window.clone()) }.is_none());
    assert_eq!(window.window_name().as_deref(), Some(c"w0"));
}
