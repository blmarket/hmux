use super::*;
use crate::cmd::{CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::input::{KEYC_CTRL, KEYC_META};
use crate::key_bindings::{
    key_binding_key, key_bindings_add, key_bindings_get_table, key_bindings_remove,
};
use crate::pane_activity::PaneActivityState;
use crate::paste::{PasteBufferStore, with_paste_buffers, with_paste_buffers_mut};
use crate::tests::test_fixtures::{
    Clients, Pane, Registry, Session, Window, globals, link, unlink,
};
use crate::window_timestamps::WindowTimestampState;
use ::core::ffi::c_int;
use ::std::ffi::CString;
use ::std::sync::MutexGuard;

/// A turn at the module's own statics — the criteria pointer every
/// comparator reads and the answer list each `sort_get_*` hands back — and
/// at the server-wide state those lists are built from. Cargo runs the
/// tests on parallel threads, so every test that asks for a list holds
/// both, always in this order.
fn sorting() -> (crate::tests::test_fixtures::GlobalsGuard, MutexGuard<'static, ()>) {
    static LISTS: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let outer = globals();
    let inner = LISTS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    (outer, inner)
}

/// What a caller fills in before asking for a sorted list, with no
/// sequence of orders behind it.
fn crit(order: sort_order, reversed: c_int) -> sort_criteria_t {
    RustSortCriteria::new(order, reversed != 0)
}

/// A timeval, as a creation or activity time.
fn at(sec: i64, usec: i64) -> timeval {
    timeval {
        tv_sec: sec as __time_t,
        tv_usec: usec as __suseconds_t,
    }
}

#[test]
fn an_order_is_read_back_from_its_name_whatever_its_case() {
    assert_eq!(
        RustSortCriteria::parse_order(Some(c"activity")),
        SORT_ACTIVITY
    );
    assert_eq!(
        RustSortCriteria::parse_order(Some(c"creation")),
        SORT_CREATION
    );
    assert_eq!(RustSortCriteria::parse_order(Some(c"index")), SORT_INDEX);
    assert_eq!(
        RustSortCriteria::parse_order(Some(c"modifier")),
        SORT_MODIFIER
    );
    assert_eq!(RustSortCriteria::parse_order(Some(c"name")), SORT_NAME);
    assert_eq!(RustSortCriteria::parse_order(Some(c"order")), SORT_ORDER);
    assert_eq!(RustSortCriteria::parse_order(Some(c"size")), SORT_SIZE);
    assert_eq!(RustSortCriteria::parse_order(Some(c"z")), SORT_Z);
    assert_eq!(
        RustSortCriteria::parse_order(Some(c"ACTIVITY")),
        SORT_ACTIVITY
    );
    assert_eq!(RustSortCriteria::parse_order(Some(c"Size")), SORT_SIZE);
    assert_eq!(RustSortCriteria::parse_order(Some(c"Z")), SORT_Z);
}

#[test]
fn key_is_another_name_for_index_and_title_for_name() {
    assert_eq!(RustSortCriteria::parse_order(Some(c"key")), SORT_INDEX);
    assert_eq!(RustSortCriteria::parse_order(Some(c"title")), SORT_NAME);
}

#[test]
fn a_name_that_is_no_order_and_no_name_at_all_are_the_end() {
    assert_eq!(RustSortCriteria::parse_order(Some(c"")), SORT_END);
    assert_eq!(RustSortCriteria::parse_order(Some(c"nonesuch")), SORT_END);
    assert_eq!(RustSortCriteria::parse_order(Some(c"activityx")), SORT_END);
    assert_eq!(RustSortCriteria::parse_order(None), SORT_END);
}

#[test]
fn an_order_prints_as_its_first_name_and_the_end_prints_as_nothing() {
    for (order, name) in [
        (SORT_ACTIVITY, "activity"),
        (SORT_CREATION, "creation"),
        (SORT_INDEX, "index"),
        (SORT_MODIFIER, "modifier"),
        (SORT_NAME, "name"),
        (SORT_ORDER, "order"),
        (SORT_SIZE, "size"),
        (SORT_Z, "z"),
    ] {
        let text = RustSortCriteria::order_name(order).expect("a name");
        assert_eq!(text.to_str().expect("ascii"), name);
        assert_eq!(RustSortCriteria::parse_order(Some(text)), order);
    }
    assert_eq!(RustSortCriteria::order_name(SORT_END), None);
    assert_eq!(RustSortCriteria::order_name(SORT_END + 1), None);
}

#[test]
fn an_order_with_no_sequence_behind_it_stays_where_it_is() {
    let mut c = crit(SORT_NAME, 0);
    c.advance();
    assert_eq!(c.order(), SORT_NAME);
}

#[test]
fn the_next_order_walks_the_sequence_and_wraps_at_its_end() {
    let mut c = crit(SORT_ACTIVITY, 0);
    c.set_cycle(&[SORT_ACTIVITY, SORT_NAME, SORT_SIZE]);
    c.advance();
    assert_eq!(c.order(), SORT_NAME);
    c.advance();
    assert_eq!(c.order(), SORT_SIZE);
    c.advance();
    assert_eq!(c.order(), SORT_ACTIVITY);
}

#[test]
fn an_order_the_sequence_does_not_hold_starts_it_again() {
    let mut c = crit(SORT_INDEX, 0);
    c.set_cycle(&[SORT_NAME, SORT_SIZE]);
    c.advance();
    assert_eq!(c.order(), SORT_NAME);
}

/// A sequence holding nothing at all leaves the order at the end marker,
/// which is what a walk with nowhere to go answers.
#[test]
fn an_empty_sequence_leaves_the_order_at_the_end() {
    let mut c = crit(SORT_NAME, 0);
    c.set_cycle(&[]);
    c.advance();
    assert_eq!(c.order(), SORT_END);
}

#[test]
fn a_sequence_of_one_order_keeps_answering_it() {
    let mut c = crit(SORT_SIZE, 0);
    c.set_cycle(&[SORT_SIZE]);
    c.advance();
    assert_eq!(c.order(), SORT_SIZE);
}

#[test]
fn the_window_tree_never_swaps_by_index_and_swaps_by_a_name_that_differs() {
    let _guard = sorting();
    let mut session = Session::new(1, "s");
    let mut first = Window::new(1, "aaa", 80, 24);
    let mut second = Window::new(2, "bbb", 80, 24);
    let wl1 = link(&mut session, &mut first, 1);
    let wl2 = link(&mut session, &mut second, 2);
    unsafe {
        let c = crit(SORT_INDEX, 0);
        assert_eq!(sort_would_window_tree_swap(&c, &*wl1, &*wl2), 0);
        assert_eq!(sort_would_window_tree_swap(&c, &*wl2, &*wl1), 0);

        let c = crit(SORT_NAME, 0);
        assert_eq!(sort_would_window_tree_swap(&c, &*wl1, &*wl2), 1);
        assert_eq!(sort_would_window_tree_swap(&c, &*wl1, &*wl1), 0);

        let c = crit(SORT_NAME, 1);
        assert_eq!(sort_would_window_tree_swap(&c, &*wl1, &*wl2), 1);
        assert_eq!(sort_would_window_tree_swap(&c, &*wl1, &*wl1), 0);
    }
    unlink(&mut session, wl1);
    unlink(&mut session, wl2);
}

/// A turn at the paste store: it starts with no buffers at all and is
/// emptied again at the end of the test. The buffers are a global, so the
/// module's own turn comes with it.
struct Buffers;

impl Buffers {
    fn new() -> Buffers {
        empty_store();
        Buffers
    }

    /// Adds a named buffer holding `data`, newer than every buffer before
    /// it.
    fn add(&self, name: &str, data: &str) {
        let name = CString::new(name).expect("no NUL");
        assert!(
            with_paste_buffers_mut(|buffers| {
                buffers.set_named(name.as_c_str(), data.as_bytes().to_vec())
            })
            .is_ok()
        );
    }
}

impl Drop for Buffers {
    fn drop(&mut self) {
        empty_store();
    }
}

fn empty_store() {
    let names = with_paste_buffers(|buffers| {
        buffers
            .buffers()
            .map(|buffer| buffer.name.to_owned())
            .collect::<Vec<_>>()
    });
    for name in names {
        with_paste_buffers_mut(|buffers| buffers.remove(name.as_c_str()));
    }
}

/// The names of the buffers the store hands back under `order`.
fn buffers(order: sort_order, reversed: c_int) -> Vec<String> {
    let c = crit(order, reversed);
    sort_get_buffers(&c)
        .iter()
        .map(|buffer| buffer.name.to_string_lossy().into_owned())
        .collect()
}

#[test]
fn buffers_come_back_newest_first_when_there_is_no_order_to_sort_by() {
    let _guard = sorting();
    let store = Buffers::new();
    store.add("one", "aaa");
    store.add("two", "aaaaa");
    store.add("three", "a");
    {
        assert_eq!(buffers(SORT_END, 0), ["three", "two", "one"]);
        assert_eq!(buffers(SORT_END, 1), ["three", "two", "one"]);
    }
}

#[test]
fn buffers_sort_by_name_and_by_size() {
    let _guard = sorting();
    let store = Buffers::new();
    store.add("one", "aaa");
    store.add("two", "aaaaa");
    store.add("three", "a");
    {
        assert_eq!(buffers(SORT_NAME, 0), ["one", "three", "two"]);
        assert_eq!(buffers(SORT_NAME, 1), ["two", "three", "one"]);
        assert_eq!(buffers(SORT_SIZE, 0), ["three", "one", "two"]);
        assert_eq!(buffers(SORT_SIZE, 1), ["two", "one", "three"]);
    }
}

#[test]
fn buffers_of_one_size_fall_back_on_their_names() {
    let _guard = sorting();
    let store = Buffers::new();
    store.add("bbb", "xx");
    store.add("aaa", "yy");
    {
        assert_eq!(buffers(SORT_SIZE, 0), ["aaa", "bbb"]);
        assert_eq!(buffers(SORT_SIZE, 1), ["bbb", "aaa"]);
    }
}

/// The creation order of a buffer is the counter the store hands out, and
/// the newest buffer holds the highest one, so sorting by creation is
/// newest first.
#[test]
fn buffers_sort_by_creation_newest_first() {
    let _guard = sorting();
    let store = Buffers::new();
    store.add("bbb", "x");
    store.add("aaa", "y");
    {
        assert_eq!(buffers(SORT_CREATION, 0), ["aaa", "bbb"]);
        assert_eq!(buffers(SORT_CREATION, 1), ["bbb", "aaa"]);
    }
}

/// Sorting by order never compares anything: the walk is already in that
/// order, so a list asked for it is handed back as it stands and is only
/// turned round when the criteria are reversed.
#[test]
fn sorting_by_order_only_turns_the_list_round() {
    let _guard = sorting();
    let store = Buffers::new();
    store.add("one", "aaa");
    store.add("two", "aaaaa");
    store.add("three", "a");
    {
        assert_eq!(buffers(SORT_ORDER, 0), ["three", "two", "one"]);
        assert_eq!(buffers(SORT_ORDER, 1), ["one", "two", "three"]);
    }
}

#[test]
fn an_empty_store_hands_back_nothing() {
    let _guard = sorting();
    let _store = Buffers::new();
    {
        assert!(buffers(SORT_NAME, 0).is_empty());
        assert!(buffers(SORT_ORDER, 1).is_empty());
    }
}

/// The names of the clients the server hands back under `order`.
fn clients_named(order: sort_order, reversed: c_int) -> Vec<String> {
    unsafe {
        let c = crit(order, reversed);
        let l = sort_get_clients(&c);
        l.iter()
            .map(|c| {
                c.name()
                    .expect("client name")
                    .to_string_lossy()
                    .into_owned()
            })
            .collect()
    }
}

#[test]
fn only_attached_clients_that_are_still_there_are_listed() {
    let _guard = sorting();
    let mut list = Clients::new();
    unsafe {
        (*list.add("attached", 80, 24)).flags = CLIENT_ATTACHED as uint64_t;
        list.add("detached", 80, 24);
        (*list.add("dead", 80, 24)).flags = (CLIENT_ATTACHED | CLIENT_DEAD) as uint64_t;
        (*list.add("suspended", 80, 24)).flags = (CLIENT_ATTACHED | CLIENT_SUSPENDED) as uint64_t;
        (*list.add("exiting", 80, 24)).flags = (CLIENT_ATTACHED | CLIENT_EXIT) as uint64_t;
        assert_eq!(clients_named(SORT_NAME, 0), ["attached"]);
    }
}

#[test]
fn clients_sort_by_name_and_by_the_size_of_their_terminals() {
    let _guard = sorting();
    let mut list = Clients::new();
    unsafe {
        (*list.add("bbb", 80, 24)).flags = CLIENT_ATTACHED as uint64_t;
        (*list.add("aaa", 80, 50)).flags = CLIENT_ATTACHED as uint64_t;
        (*list.add("ccc", 40, 24)).flags = CLIENT_ATTACHED as uint64_t;
        assert_eq!(clients_named(SORT_NAME, 0), ["aaa", "bbb", "ccc"]);
        assert_eq!(clients_named(SORT_NAME, 1), ["ccc", "bbb", "aaa"]);
        assert_eq!(clients_named(SORT_SIZE, 0), ["ccc", "bbb", "aaa"]);
        assert_eq!(clients_named(SORT_SIZE, 1), ["aaa", "bbb", "ccc"]);
    }
}

/// Clients of one size fall back on their names, and a client is only
/// compared by height once the widths are the same.
#[test]
fn clients_of_one_size_fall_back_on_their_names() {
    let _guard = sorting();
    let mut list = Clients::new();
    unsafe {
        (*list.add("bbb", 80, 24)).flags = CLIENT_ATTACHED as uint64_t;
        (*list.add("aaa", 80, 24)).flags = CLIENT_ATTACHED as uint64_t;
        assert_eq!(clients_named(SORT_SIZE, 0), ["aaa", "bbb"]);
    }
}

/// A client's creation is oldest first and its activity newest first,
/// which is the shape every one of the stores sorts times in.
#[test]
fn clients_sort_by_creation_oldest_first_and_activity_newest_first() {
    let _guard = sorting();
    let mut list = Clients::new();
    unsafe {
        let older = list.add("older", 80, 24);
        (*older).flags = CLIENT_ATTACHED as uint64_t;
        (*older).creation_time = at(100, 0);
        (*older).activity_time = at(100, 5);
        let newer = list.add("newer", 80, 24);
        (*newer).flags = CLIENT_ATTACHED as uint64_t;
        (*newer).creation_time = at(100, 7);
        (*newer).activity_time = at(200, 0);
        assert_eq!(clients_named(SORT_CREATION, 0), ["older", "newer"]);
        assert_eq!(clients_named(SORT_CREATION, 1), ["newer", "older"]);
        assert_eq!(clients_named(SORT_ACTIVITY, 0), ["newer", "older"]);
        assert_eq!(clients_named(SORT_ACTIVITY, 1), ["older", "newer"]);
    }
}

#[test]
fn clients_at_one_time_fall_back_on_their_names() {
    let _guard = sorting();
    let mut list = Clients::new();
    unsafe {
        let bbb = list.add("bbb", 80, 24);
        (*bbb).flags = CLIENT_ATTACHED as uint64_t;
        (*bbb).creation_time = at(100, 0);
        let aaa = list.add("aaa", 80, 24);
        (*aaa).flags = CLIENT_ATTACHED as uint64_t;
        (*aaa).creation_time = at(100, 0);
        assert_eq!(clients_named(SORT_CREATION, 0), ["aaa", "bbb"]);
        assert_eq!(clients_named(SORT_ACTIVITY, 0), ["aaa", "bbb"]);
    }
}

#[test]
fn no_clients_at_all_hand_back_nothing() {
    let _guard = sorting();
    let _list = Clients::new();
    {
        assert!(clients_named(SORT_NAME, 0).is_empty());
    }
}

/// The names of the sessions the server hands back under `order`.
fn sessions_named(order: sort_order, reversed: c_int) -> Vec<String> {
    {
        let c = crit(order, reversed);
        let l = sort_get_sessions(&c);
        l.iter()
            .map(|s| {
                s.name()
                    .as_deref()
                    .expect("the session has a name")
                    .to_string_lossy()
                    .into_owned()
            })
            .collect()
    }
}

#[test]
fn sessions_sort_by_name_and_by_id() {
    let _guard = sorting();
    let mut registry = Registry::new();
    let mut first = Session::new(7, "aaa");
    let mut second = Session::new(3, "bbb");
    registry.add_session(&mut first);
    registry.add_session(&mut second);
    {
        assert_eq!(sessions_named(SORT_NAME, 0), ["aaa", "bbb"]);
        assert_eq!(sessions_named(SORT_NAME, 1), ["bbb", "aaa"]);
        assert_eq!(sessions_named(SORT_INDEX, 0), ["bbb", "aaa"]);
        assert_eq!(sessions_named(SORT_INDEX, 1), ["aaa", "bbb"]);
    }
}

#[test]
fn sessions_sort_by_creation_oldest_first_and_activity_newest_first() {
    let _guard = sorting();
    let mut registry = Registry::new();
    let mut older = Session::new(1, "older");
    let mut newer = Session::new(2, "newer");
    unsafe {
        (*older.ptr()).creation_time = at(100, 0);
        older.handle().set_activity_time(at(100, 5));
        (*newer.ptr()).creation_time = at(100, 7);
        newer.handle().set_activity_time(at(200, 0));
    }
    registry.add_session(&mut older);
    registry.add_session(&mut newer);
    {
        assert_eq!(sessions_named(SORT_CREATION, 0), ["older", "newer"]);
        assert_eq!(sessions_named(SORT_CREATION, 1), ["newer", "older"]);
        assert_eq!(sessions_named(SORT_ACTIVITY, 0), ["newer", "older"]);
        assert_eq!(sessions_named(SORT_ACTIVITY, 1), ["older", "newer"]);
    }
}

#[test]
fn sessions_at_one_time_fall_back_on_their_names() {
    let _guard = sorting();
    let mut registry = Registry::new();
    let mut bbb = Session::new(1, "bbb");
    let mut aaa = Session::new(2, "aaa");
    registry.add_session(&mut bbb);
    registry.add_session(&mut aaa);
    {
        assert_eq!(sessions_named(SORT_CREATION, 0), ["aaa", "bbb"]);
        assert_eq!(sessions_named(SORT_ACTIVITY, 0), ["aaa", "bbb"]);
        assert_eq!(sessions_named(SORT_SIZE, 0), ["aaa", "bbb"]);
    }
}

#[test]
fn no_sessions_at_all_hand_back_nothing() {
    let _guard = sorting();
    let _registry = Registry::new();
    {
        assert!(sessions_named(SORT_NAME, 0).is_empty());
    }
}

/// A pane carrying a title, which is what every pane comparison falls back
/// on. The pane and the screen behind it are the server-free fixtures.
fn titled(id: u_int, title: &str, sx: u_int, sy: u_int) -> Pane {
    let mut pane = Pane::new(id, sx, sy, 100);
    {
        let title = CString::new(title).expect("no NUL");
        assert_eq!(pane.base_mut().set_title(&title, 0), 1);
    }
    pane
}

/// The titles of panes, which is how a test tells them apart.
unsafe fn titles(l: &[RustWindowPaneWeak]) -> Vec<String> {
    unsafe {
        l.iter()
            .map(|wp| {
                wp.get()
                    .unwrap()
                    .screen_ref()
                    .title()
                    .unwrap_or(c"")
                    .to_string_lossy()
                    .into_owned()
            })
            .collect()
    }
}

/// The panes of `w`, by title, under `order`.
unsafe fn window_panes(w: &WindowRef, order: sort_order, reversed: c_int) -> Vec<String> {
    unsafe {
        let c = crit(order, reversed);
        let l = w.sorted_panes(&c);
        titles(&l)
    }
}

#[test]
fn panes_sort_by_id_by_size_and_by_title() {
    let _guard = sorting();
    let mut window = Window::new(1, "w", 80, 24);
    let mut first = titled(9, "ccc", 10, 2);
    let mut second = titled(3, "aaa", 4, 4);
    let mut third = titled(5, "bbb", 1, 1);
    window.add_pane(&mut first);
    window.add_pane(&mut second);
    window.add_pane(&mut third);
    unsafe {
        let w = window.reference();
        assert_eq!(window_panes(&w, SORT_CREATION, 0), ["aaa", "bbb", "ccc"]);
        assert_eq!(window_panes(&w, SORT_CREATION, 1), ["ccc", "bbb", "aaa"]);
        assert_eq!(window_panes(&w, SORT_SIZE, 0), ["bbb", "aaa", "ccc"]);
        assert_eq!(window_panes(&w, SORT_SIZE, 1), ["ccc", "aaa", "bbb"]);
        assert_eq!(window_panes(&w, SORT_NAME, 0), ["aaa", "bbb", "ccc"]);
        assert_eq!(window_panes(&w, SORT_NAME, 1), ["ccc", "bbb", "aaa"]);
    }
}

#[test]
fn panes_sort_by_where_they_are_in_the_window_and_by_when_they_were_last_active() {
    let _guard = sorting();
    let mut window = Window::new(1, "w", 80, 24);
    let mut first = titled(1, "first", 80, 24);
    let mut second = titled(2, "second", 80, 24);
    let mut third = titled(3, "third", 80, 24);
    unsafe {
        (*first.ptr()).mark_active_at(30);
        (*second.ptr()).mark_active_at(10);
        (*third.ptr()).mark_active_at(20);
    }
    window.add_pane(&mut first);
    window.add_pane(&mut second);
    window.add_pane(&mut third);
    unsafe {
        let w = window.reference();
        assert_eq!(
            window_panes(&w, SORT_INDEX, 0),
            ["first", "second", "third"]
        );
        assert_eq!(
            window_panes(&w, SORT_INDEX, 1),
            ["third", "second", "first"]
        );
        assert_eq!(
            window_panes(&w, SORT_ACTIVITY, 0),
            ["second", "third", "first"]
        );
    }
}

/// Every pane of a window that has no floating pane sits at the same
/// z-index, so the titles are what tells them apart; a floating pane comes
/// in front of the rest.
#[test]
fn panes_share_a_z_index_until_one_of_them_floats() {
    let _guard = sorting();
    let mut window = Window::new(1, "w", 80, 24);
    let mut first = titled(1, "bbb", 80, 24);
    let mut second = titled(2, "aaa", 80, 24);
    window.add_pane(&mut first);
    window.add_pane(&mut second);
    unsafe {
        let w = window.reference();
        assert_eq!(window_panes(&w, SORT_Z, 0), ["aaa", "bbb"]);
        crate::tests::test_fixtures::set_pane_floating(
            &mut w.as_window_mut(),
            (*first.ptr()).pane_id(),
            true,
        );
        assert_eq!(window_panes(&w, SORT_Z, 0), ["bbb", "aaa"]);
        assert_eq!(window_panes(&w, SORT_Z, 1), ["aaa", "bbb"]);
        crate::tests::test_fixtures::set_pane_floating(
            &mut w.as_window_mut(),
            (*first.ptr()).pane_id(),
            false,
        );
    }
}

#[test]
fn a_window_with_no_panes_hands_back_nothing() {
    let _guard = sorting();
    let window = Window::new(1, "w", 80, 24);
    unsafe {
        assert!(window_panes(window.handle(), SORT_NAME, 0).is_empty());
    }
}

#[test]
fn sorted_pane_observers_do_not_retain_panes_or_the_window() {
    let _guard = sorting();
    let mut window = Window::new(1, "held", 80, 24);
    let mut pane = titled(7, "pane", 80, 24);
    window.add_pane(&mut pane);
    let weak = window.handle().downgrade();
    let panes = (window.handle()).sorted_panes(&crit(SORT_INDEX, 0));
    assert_eq!(panes.len(), 1);
    assert_eq!(panes[0].id(), 7);
    assert!(unsafe { panes[0].get() }.is_some());

    let weak_pane = panes[0].clone();
    drop(window);
    assert!(weak.upgrade().is_none());
    assert!(weak_pane.upgrade().is_none());
    assert!(unsafe { panes[0].get() }.is_none());
    drop(panes);
    assert!(weak_pane.upgrade().is_none());
}

#[test]
fn a_session_hands_back_the_panes_of_every_window_linked_into_it() {
    let _guard = sorting();
    let mut session = Session::new(1, "s");
    let mut first = Window::new(1, "w1", 80, 24);
    let mut second = Window::new(2, "w2", 80, 24);
    let mut one = titled(1, "one", 80, 24);
    let mut two = titled(2, "two", 80, 24);
    let mut three = titled(3, "three", 80, 24);
    first.add_pane(&mut one);
    first.add_pane(&mut two);
    second.add_pane(&mut three);
    let wl1 = link(&mut session, &mut first, 1);
    let wl2 = link(&mut session, &mut second, 2);
    unsafe {
        let c = crit(SORT_NAME, 0);
        let l = sort_get_panes_session(&mut *session.ptr(), &c);
        assert_eq!(titles(&l), ["one", "three", "two"]);

        let c = crit(SORT_END, 0);
        let l = sort_get_panes_session(&mut *session.ptr(), &c);
        assert_eq!(titles(&l), ["one", "two", "three"]);
    }
    unlink(&mut session, wl1);
    unlink(&mut session, wl2);
}

/// Every pane the server has is reached through the sessions, so a window
/// linked into two of them has its panes listed once for each.
#[test]
fn a_window_linked_into_two_sessions_has_its_panes_listed_twice() {
    let _guard = sorting();
    let mut registry = Registry::new();
    let mut first = Session::new(1, "aaa");
    let mut second = Session::new(2, "bbb");
    let mut window = Window::new(1, "w", 80, 24);
    let mut pane = titled(1, "only", 80, 24);
    window.add_pane(&mut pane);
    let wl1 = link(&mut first, &mut window, 1);
    let wl2 = link(&mut second, &mut window, 1);
    registry.add_session(&mut first);
    registry.add_session(&mut second);
    unsafe {
        let c = crit(SORT_END, 0);
        let l = sort_get_panes(&c);
        assert_eq!(titles(&l), ["only", "only"]);
    }
    unlink(&mut first, wl1);
    unlink(&mut second, wl2);
}

#[test]
fn no_sessions_at_all_hand_back_no_panes() {
    let _guard = sorting();
    let _registry = Registry::new();
    unsafe {
        let c = crit(SORT_NAME, 0);
        let l = sort_get_panes(&c);
        assert_eq!(titles(&l), Vec::<String>::new());
    }
}

/// The names of the windows winlinks point at.
fn linked(l: &[WinlinkRef]) -> Vec<String> {
    {
        l.iter()
            .map(|wl| {
                wl.get()
                    .unwrap()
                    .window_handle()
                    .unwrap()
                    .window_name()
                    .as_deref()
                    .map_or(String::new(), |name| name.to_string_lossy().into_owned())
            })
            .collect()
    }
}

/// The winlinks of `s`, by window name, under `order`.
fn session_winlinks(s: &SessionRef, order: sort_order, reversed: c_int) -> Vec<String> {
    {
        let c = crit(order, reversed);
        let l = s.sorted_winlinks(&c);
        linked(&l)
    }
}

#[test]
fn winlinks_sort_by_index_by_window_name_and_by_window_size() {
    let _guard = sorting();
    let mut session = Session::new(1, "s");
    let mut first = Window::new(1, "ccc", 10, 2);
    let mut second = Window::new(2, "aaa", 4, 4);
    let mut third = Window::new(3, "bbb", 1, 1);
    let wl1 = link(&mut session, &mut first, 3);
    let wl2 = link(&mut session, &mut second, 1);
    let wl3 = link(&mut session, &mut third, 2);
    {
        let s = session.handle();
        assert_eq!(session_winlinks(s, SORT_INDEX, 0), ["aaa", "bbb", "ccc"]);
        assert_eq!(session_winlinks(s, SORT_INDEX, 1), ["ccc", "bbb", "aaa"]);
        assert_eq!(session_winlinks(s, SORT_NAME, 0), ["aaa", "bbb", "ccc"]);
        assert_eq!(session_winlinks(s, SORT_SIZE, 0), ["bbb", "aaa", "ccc"]);
        assert_eq!(session_winlinks(s, SORT_SIZE, 1), ["ccc", "aaa", "bbb"]);
    }
    unlink(&mut session, wl1);
    unlink(&mut session, wl2);
    unlink(&mut session, wl3);
}

#[test]
fn winlinks_sort_by_the_creation_and_activity_of_their_windows() {
    let _guard = sorting();
    let mut session = Session::new(1, "s");
    let mut older = Window::new(1, "older", 80, 24);
    let mut newer = Window::new(2, "newer", 80, 24);
    unsafe {
        (*older.ptr()).set_creation_time(at(100, 0));
        (*older.ptr()).set_activity_time(at(100, 5));
        (*newer.ptr()).set_creation_time(at(100, 7));
        (*newer.ptr()).set_activity_time(at(200, 0));
    }
    let wl1 = link(&mut session, &mut older, 1);
    let wl2 = link(&mut session, &mut newer, 2);
    {
        let s = session.handle();
        assert_eq!(session_winlinks(s, SORT_CREATION, 0), ["older", "newer"]);
        assert_eq!(session_winlinks(s, SORT_CREATION, 1), ["newer", "older"]);
        assert_eq!(session_winlinks(s, SORT_ACTIVITY, 0), ["newer", "older"]);
        assert_eq!(session_winlinks(s, SORT_ACTIVITY, 1), ["older", "newer"]);
    }
    unlink(&mut session, wl1);
    unlink(&mut session, wl2);
}

#[test]
fn winlinks_of_windows_at_one_time_fall_back_on_the_window_names() {
    let _guard = sorting();
    let mut session = Session::new(1, "s");
    let mut bbb = Window::new(1, "bbb", 80, 24);
    let mut aaa = Window::new(2, "aaa", 80, 24);
    let wl1 = link(&mut session, &mut bbb, 1);
    let wl2 = link(&mut session, &mut aaa, 2);
    {
        let s = session.handle();
        assert_eq!(session_winlinks(s, SORT_CREATION, 0), ["aaa", "bbb"]);
        assert_eq!(session_winlinks(s, SORT_ACTIVITY, 0), ["aaa", "bbb"]);
        assert_eq!(session_winlinks(s, SORT_Z, 0), ["aaa", "bbb"]);
    }
    unlink(&mut session, wl1);
    unlink(&mut session, wl2);
}

#[test]
fn every_session_hands_over_its_winlinks() {
    let _guard = sorting();
    let mut registry = Registry::new();
    let mut first = Session::new(1, "aaa");
    let mut second = Session::new(2, "bbb");
    let mut one = Window::new(1, "one", 80, 24);
    let mut two = Window::new(2, "two", 80, 24);
    let mut three = Window::new(3, "three", 80, 24);
    let wl1 = link(&mut first, &mut one, 1);
    let wl2 = link(&mut first, &mut two, 2);
    let wl3 = link(&mut second, &mut three, 1);
    registry.add_session(&mut first);
    registry.add_session(&mut second);
    {
        let c = crit(SORT_END, 0);
        let l = sort_get_winlinks(&c);
        assert_eq!(linked(&l), ["one", "two", "three"]);

        let c = crit(SORT_NAME, 0);
        let l = sort_get_winlinks(&c);
        assert_eq!(linked(&l), ["one", "three", "two"]);
    }
    unlink(&mut first, wl1);
    unlink(&mut first, wl2);
    unlink(&mut second, wl3);
}

#[test]
fn a_session_with_no_windows_hands_back_no_winlinks() {
    let _guard = sorting();
    let session = Session::new(1, "s");
    {
        assert!(session_winlinks(session.handle(), SORT_INDEX, 0).is_empty());
    }
}

/// A key table holding bindings of its own, taken away again at the end of
/// the test. Every binding goes in through `key_bindings_add`, which takes
/// over the command list it is given, and comes out through
/// `key_bindings_remove`, which drops the table itself once the last one
/// has gone.
struct Table {
    name: CString,
    keys: Vec<key_code>,
}

impl Table {
    fn new(name: &str) -> Table {
        Table {
            name: CString::new(name).expect("no NUL"),
            keys: Vec::new(),
        }
    }

    fn add(&mut self, key: key_code) {
        unsafe {
            let mut pr = cmd_parse_from_string(c"display-message hi", None);
            assert_eq!(pr.status, CMD_PARSE_SUCCESS);
            key_bindings_add(&self.name, key, None, 0, pr.cmdlist.take());
        }
        self.keys.push(key);
    }

    fn handle(&self) -> KeyTableRef {
        key_bindings_get_table(&self.name, 0).unwrap()
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        {
            for key in &self.keys {
                key_bindings_remove(&self.name, *key);
            }
        }
    }
}

/// A turn at the key tables, holding no bindings at all to start with and
/// none again at the end of the test. What is checked is the bindings
/// rather than the tables themselves: a table nobody has bound anything in
/// contributes nothing to any of these lists, and another module's tests
/// leave an empty one behind them.
fn tables() -> (crate::tests::test_fixtures::GlobalsGuard, MutexGuard<'static, ()>) {
    let guards = sorting();
    {
        let c = crit(SORT_END, 0);
        let l = sort_get_key_bindings(&c);
        assert_eq!(l.len(), 0, "the key tables already hold bindings");
    }
    guards
}

/// The keys of bindings.
fn keys(l: &[key_binding]) -> Vec<key_code> {
    l.iter().map(key_binding_key).collect()
}

/// The bindings of `table` under `order`, by key.
fn table_keys(table: &KeyTableRef, order: sort_order, reversed: c_int) -> Vec<key_code> {
    {
        let c = crit(order, reversed);
        let l = sort_get_key_bindings_table(&mut table.borrow_mut(), &c);
        keys(&l)
    }
}

#[test]
fn key_bindings_sort_by_the_key_itself() {
    let _guards = tables();
    let mut table = Table::new("one-table");
    table.add(b'c' as key_code);
    table.add(b'a' as key_code);
    table.add(b'b' as key_code);
    {
        assert_eq!(
            table_keys(&table.handle(), SORT_INDEX, 0),
            [b'a' as key_code, b'b' as key_code, b'c' as key_code]
        );
        assert_eq!(
            table_keys(&table.handle(), SORT_INDEX, 1),
            [b'c' as key_code, b'b' as key_code, b'a' as key_code]
        );
        assert_eq!(
            table_keys(&table.handle(), SORT_END, 0),
            [b'a' as key_code, b'b' as key_code, b'c' as key_code]
        );
    }
}

/// Every modifier sits above the thirty-second bit of a key, and both the
/// comparison by key and the comparison by modifier cut their answer down
/// to an `int`, so two keys that differ only in their modifiers compare the
/// same under either. What decides them is the tie-break below, which
/// answers *one* — not the zero a comparison usually answers for two things
/// that are equal — for two bindings in the same table, so the pair is
/// turned round.
#[test]
fn the_modifiers_of_a_key_are_cut_off_both_of_the_comparisons() {
    let _guards = tables();
    let mut table = Table::new("one-table");
    table.add(b'a' as key_code);
    table.add(b'a' as key_code | KEYC_META);
    {
        assert_eq!(
            table_keys(&table.handle(), SORT_INDEX, 0),
            [b'a' as key_code | KEYC_META, b'a' as key_code]
        );
        assert_eq!(
            table_keys(&table.handle(), SORT_MODIFIER, 0),
            [b'a' as key_code | KEYC_META, b'a' as key_code]
        );
        assert_eq!(
            table_keys(&table.handle(), SORT_MODIFIER, 1),
            [b'a' as key_code, b'a' as key_code | KEYC_META]
        );
    }
}

/// Sorting key bindings by name compares the tables they are in, and
/// answers *one* when the names are the same rather than the zero a
/// comparison usually answers for two things that are equal. Two bindings
/// of one table are turned round by it.
#[test]
fn two_bindings_of_one_table_are_turned_round_by_their_table_name() {
    let _guards = tables();
    let mut table = Table::new("one-table");
    table.add(b'a' as key_code);
    table.add(b'b' as key_code);
    {
        assert_eq!(
            table_keys(&table.handle(), SORT_NAME, 0),
            [b'b' as key_code, b'a' as key_code]
        );
    }
}

/// Two bindings are not the whole of it: the comparison answers *one* for
/// every pair of one table, so what the list comes out as is decided by
/// the sort's own discipline rather than by any order over the bindings.
/// [`merge_sort`] turns a table's run round, and leaves the tables where
/// the walk put them.
#[test]
fn every_binding_of_a_table_is_turned_round_by_their_table_name() {
    let _guards = tables();
    let mut table = Table::new("one-table");
    for key in b'a'..=b'j' {
        table.add(key as key_code);
    }
    {
        assert_eq!(
            table_keys(&table.handle(), SORT_NAME, 0),
            (b'a'..=b'j')
                .rev()
                .map(|k| k as key_code)
                .collect::<Vec<_>>()
        );
    }
}

/// Two bindings of different tables compare *equal*, so the tables stay
/// where the walk put them and only the runs inside them are turned round.
#[test]
fn each_table_keeps_its_place_when_its_own_run_is_turned_round() {
    let _guards = tables();
    let mut first = Table::new("aaa-table");
    let mut second = Table::new("bbb-table");
    for key in *b"cde" {
        first.add(key as key_code);
    }
    for key in *b"ab" {
        second.add(key as key_code);
    }
    {
        let c = crit(SORT_NAME, 0);
        let l = sort_get_key_bindings(&c);
        assert_eq!(
            keys(&l),
            [
                b'e' as key_code,
                b'd' as key_code,
                b'c' as key_code,
                b'b' as key_code,
                b'a' as key_code
            ]
        );
    }
}

/// A table nobody has bound anything in yet, which the server keeps out of
/// its own tree until something is.
#[test]
fn a_table_with_no_bindings_hands_back_nothing() {
    let _guards = tables();
    let table = KeyTableRef::new(key_table::new(CString::default()));
    {
        assert!(table_keys(&table, SORT_INDEX, 0).is_empty());
    }
}

#[test]
fn every_table_hands_over_its_bindings() {
    let _guards = tables();
    let mut first = Table::new("aaa-table");
    let mut second = Table::new("bbb-table");
    first.add(b'c' as key_code);
    second.add(b'a' as key_code);
    second.add(b'b' as key_code);
    {
        let c = crit(SORT_END, 0);
        let l = sort_get_key_bindings(&c);
        assert_eq!(
            keys(&l),
            [b'c' as key_code, b'a' as key_code, b'b' as key_code]
        );

        let c = crit(SORT_INDEX, 0);
        let l = sort_get_key_bindings(&c);
        assert_eq!(
            keys(&l),
            [b'a' as key_code, b'b' as key_code, b'c' as key_code]
        );
    }
}

/// Two bindings in different tables compare as equal under every order the
/// key comparison knows, since the tie-break only answers something for
/// two bindings of the same table.
#[test]
fn two_bindings_of_different_tables_compare_the_same() {
    let _guards = tables();
    let mut first = Table::new("aaa-table");
    let mut second = Table::new("bbb-table");
    first.add(b'b' as key_code);
    second.add(b'a' as key_code);
    {
        let c = crit(SORT_NAME, 0);
        let l = sort_get_key_bindings(&c);
        assert_eq!(keys(&l), [b'b' as key_code, b'a' as key_code]);
    }
}

#[test]
fn no_key_tables_at_all_hand_back_nothing() {
    let _guards = tables();
    {
        let c = crit(SORT_INDEX, 0);
        let l = sort_get_key_bindings(&c);
        assert_eq!(keys(&l), Vec::<key_code>::new());
    }
}

/// What one of the comparisons answers for `a` and `b` under `order`.
unsafe fn compares<T>(
    cmp: impl Fn(&T, &T, &sort_criteria_t) -> c_int,
    order: sort_order,
    reversed: c_int,
    a: *mut T,
    b: *mut T,
) -> c_int {
    unsafe { cmp(&*a, &*b, &crit(order, reversed)) }
}

/// The two buffers a comparison test works over, the second one newer than
/// the first.
fn two_buffers(store: &Buffers) -> (SortedPasteBuffer, SortedPasteBuffer) {
    store.add("aaa", "xx");
    store.add("bbb", "xxxx");
    let mut buffers = sort_get_buffers(&crit(SORT_NAME, 0));
    (buffers.remove(0), buffers.remove(0))
}

#[test]
fn a_buffer_comparison_answers_the_names_the_orders_and_the_sizes() {
    let _guard = sorting();
    let store = Buffers::new();
    let (aaa, bbb) = two_buffers(&store);
    assert!(sort_buffer_cmp(&aaa, &bbb, &crit(SORT_NAME, 0)) < 0);
    assert!(sort_buffer_cmp(&bbb, &aaa, &crit(SORT_NAME, 0)) > 0);
    assert_eq!(sort_buffer_cmp(&aaa, &aaa, &crit(SORT_NAME, 0)), 0);
    assert!(sort_buffer_cmp(&aaa, &bbb, &crit(SORT_NAME, 1)) > 0);

    assert_eq!(sort_buffer_cmp(&aaa, &bbb, &crit(SORT_CREATION, 0)), 1);
    assert_eq!(sort_buffer_cmp(&bbb, &aaa, &crit(SORT_CREATION, 0)), -1);
    assert_eq!(sort_buffer_cmp(&aaa, &bbb, &crit(SORT_CREATION, 1)), -1);
    assert_eq!(sort_buffer_cmp(&aaa, &aaa, &crit(SORT_CREATION, 0)), 0);

    assert_eq!(sort_buffer_cmp(&aaa, &bbb, &crit(SORT_SIZE, 0)), -2);
    assert_eq!(sort_buffer_cmp(&bbb, &aaa, &crit(SORT_SIZE, 0)), 2);
    assert_eq!(sort_buffer_cmp(&aaa, &bbb, &crit(SORT_SIZE, 1)), 2);
}

/// A buffer knows nothing of the orders the other stores sort by, so it
/// falls back on the names for every one of them.
#[test]
fn a_buffer_comparison_falls_back_on_the_names_for_any_other_order() {
    let _guard = sorting();
    let store = Buffers::new();
    let (aaa, bbb) = two_buffers(&store);
    for order in [SORT_ACTIVITY, SORT_INDEX, SORT_MODIFIER, SORT_ORDER, SORT_Z] {
        assert!(sort_buffer_cmp(&aaa, &bbb, &crit(order, 0)) < 0);
        assert!(sort_buffer_cmp(&bbb, &aaa, &crit(order, 0)) > 0);
    }
}

#[test]
fn a_client_comparison_answers_the_names_and_the_terminal_sizes() {
    let _guard = sorting();
    let mut list = Clients::new();
    unsafe {
        let aaa = list.add("aaa", 80, 24);
        let bbb = list.add("bbb", 90, 20);
        assert!(compares(sort_client_cmp, SORT_NAME, 0, aaa, bbb) < 0);
        assert!(compares(sort_client_cmp, SORT_NAME, 0, bbb, aaa) > 0);
        assert!(compares(sort_client_cmp, SORT_NAME, 1, aaa, bbb) > 0);

        assert_eq!(compares(sort_client_cmp, SORT_SIZE, 0, aaa, bbb), -10);
        assert_eq!(compares(sort_client_cmp, SORT_SIZE, 0, bbb, aaa), 10);
        (*bbb).tty.sx = 80;
        assert_eq!(compares(sort_client_cmp, SORT_SIZE, 0, aaa, bbb), 4);
        (*bbb).tty.sy = 24;
        assert!(compares(sort_client_cmp, SORT_SIZE, 0, aaa, bbb) < 0);

        for order in [SORT_INDEX, SORT_MODIFIER, SORT_ORDER, SORT_Z] {
            assert!(compares(sort_client_cmp, order, 0, aaa, bbb) < 0);
        }
    }
}

/// A time is read as its seconds unless those are the same, and only then
/// as the microseconds inside them. Creation puts the older one first and
/// activity the newer one, and two clients at one time fall back on their
/// names.
#[test]
fn a_client_comparison_answers_the_creation_and_activity_times() {
    let _guard = sorting();
    let mut list = Clients::new();
    unsafe {
        let aaa = list.add("aaa", 80, 24);
        let bbb = list.add("bbb", 80, 24);
        for (order, older, newer) in [(SORT_CREATION, -1, 1), (SORT_ACTIVITY, 1, -1)] {
            for (a, b) in [(at(100, 0), at(200, 0)), (at(100, 0), at(100, 5))] {
                (*aaa).creation_time = a;
                (*bbb).creation_time = b;
                (*aaa).activity_time = a;
                (*bbb).activity_time = b;
                assert_eq!(compares(sort_client_cmp, order, 0, aaa, bbb), older);
                assert_eq!(compares(sort_client_cmp, order, 0, bbb, aaa), newer);
                assert_eq!(compares(sort_client_cmp, order, 1, aaa, bbb), -older);
            }
            (*bbb).creation_time = (*aaa).creation_time;
            (*bbb).activity_time = (*aaa).activity_time;
            assert!(compares(sort_client_cmp, order, 0, aaa, bbb) < 0);
        }
    }
}

#[test]
fn a_session_comparison_answers_the_ids_and_the_names() {
    let _guard = sorting();
    let mut first = Session::new(3, "aaa");
    let mut second = Session::new(7, "bbb");
    unsafe {
        let aaa = first.ptr();
        let bbb = second.ptr();
        assert_eq!(compares(sort_session_cmp, SORT_INDEX, 0, aaa, bbb), -4);
        assert_eq!(compares(sort_session_cmp, SORT_INDEX, 0, bbb, aaa), 4);
        assert_eq!(compares(sort_session_cmp, SORT_INDEX, 1, aaa, bbb), 4);
        assert!(compares(sort_session_cmp, SORT_NAME, 0, aaa, bbb) < 0);
        assert!(compares(sort_session_cmp, SORT_NAME, 0, bbb, aaa) > 0);
        for order in [SORT_MODIFIER, SORT_ORDER, SORT_SIZE, SORT_Z] {
            assert!(compares(sort_session_cmp, order, 0, aaa, bbb) < 0);
        }
    }
}

#[test]
fn a_session_comparison_answers_the_creation_and_activity_times() {
    let _guard = sorting();
    let mut first = Session::new(1, "aaa");
    let mut second = Session::new(2, "bbb");
    unsafe {
        let aaa = first.ptr();
        let bbb = second.ptr();
        for (order, older, newer) in [(SORT_CREATION, -1, 1), (SORT_ACTIVITY, 1, -1)] {
            for (a, b) in [(at(100, 0), at(200, 0)), (at(100, 0), at(100, 5))] {
                (*aaa).creation_time = a;
                (*bbb).creation_time = b;
                crate::session::session_ref_of(&mut *aaa)
                    .expect("session owner")
                    .set_activity_time(a);
                crate::session::session_ref_of(&mut *bbb)
                    .expect("session owner")
                    .set_activity_time(b);
                assert_eq!(compares(sort_session_cmp, order, 0, aaa, bbb), older);
                assert_eq!(compares(sort_session_cmp, order, 0, bbb, aaa), newer);
                assert_eq!(compares(sort_session_cmp, order, 1, aaa, bbb), -older);
            }
            (*bbb).creation_time = (*aaa).creation_time;
            crate::session::session_ref_of(&mut *bbb)
                .expect("session owner")
                .set_activity_time(
                    crate::SessionTimestampState::session_timestamps(&*aaa).activity,
                );
            assert!(compares(sort_session_cmp, order, 0, aaa, bbb) < 0);
        }
    }
}

#[test]
fn a_pane_comparison_answers_everything_a_pane_is_sorted_by() {
    let _guard = sorting();
    let mut window = Window::new(1, "w", 80, 24);
    let mut first = titled(3, "aaa", 4, 4);
    let mut second = titled(7, "bbb", 10, 2);
    window.add_pane(&mut first);
    window.add_pane(&mut second);
    unsafe {
        let owner = window.reference();
        let mut aaa = owner.as_window().panes[0].downgrade();
        let mut bbb = owner.as_window().panes[1].downgrade();
        aaa.as_pane_mut().mark_active_at(5);
        bbb.as_pane_mut().mark_active_at(9);
        assert_eq!(sort_pane_cmp(&aaa, &bbb, &crit(SORT_ACTIVITY, 0)), -4);
        assert_eq!(sort_pane_cmp(&aaa, &bbb, &crit(SORT_ACTIVITY, 1)), 4);
        assert_eq!(sort_pane_cmp(&aaa, &bbb, &crit(SORT_CREATION, 0)), -4);
        assert_eq!(sort_pane_cmp(&aaa, &bbb, &crit(SORT_SIZE, 0)), -4);
        assert_eq!(sort_pane_cmp(&aaa, &bbb, &crit(SORT_INDEX, 0)), -1);
        assert_eq!(sort_pane_cmp(&bbb, &aaa, &crit(SORT_INDEX, 0)), 1);
        assert!(sort_pane_cmp(&aaa, &bbb, &crit(SORT_Z, 0)) < 0);
        crate::tests::test_fixtures::set_pane_floating(&mut owner.as_window_mut(), aaa.id(), true);
        assert_eq!(sort_pane_cmp(&aaa, &bbb, &crit(SORT_Z, 0)), -2);
        assert_eq!(sort_pane_cmp(&aaa, &bbb, &crit(SORT_Z, 1)), 2);
        crate::tests::test_fixtures::set_pane_floating(&mut owner.as_window_mut(), aaa.id(), false);
        assert!(sort_pane_cmp(&aaa, &bbb, &crit(SORT_NAME, 0)) < 0);
        for order in [SORT_MODIFIER, SORT_ORDER] {
            assert!(sort_pane_cmp(&aaa, &bbb, &crit(order, 0)) < 0);
        }
    }
}

#[test]
fn a_winlink_comparison_answers_the_indexes_the_names_and_the_sizes() {
    let _guard = sorting();
    let mut session = Session::new(1, "s");
    let mut first = Window::new(1, "aaa", 4, 4);
    let mut second = Window::new(2, "bbb", 10, 2);
    let wl1 = link(&mut session, &mut first, 1);
    let wl2 = link(&mut session, &mut second, 5);
    unsafe {
        assert_eq!(compares(sort_winlink_cmp, SORT_INDEX, 0, wl1, wl2), -4);
        assert_eq!(compares(sort_winlink_cmp, SORT_INDEX, 0, wl2, wl1), 4);
        assert_eq!(compares(sort_winlink_cmp, SORT_INDEX, 1, wl1, wl2), 4);
        assert!(compares(sort_winlink_cmp, SORT_NAME, 0, wl1, wl2) < 0);
        assert_eq!(compares(sort_winlink_cmp, SORT_SIZE, 0, wl1, wl2), -4);
        for order in [SORT_MODIFIER, SORT_ORDER, SORT_Z] {
            assert!(compares(sort_winlink_cmp, order, 0, wl1, wl2) < 0);
        }
    }
    unlink(&mut session, wl1);
    unlink(&mut session, wl2);
}

#[test]
fn a_winlink_comparison_answers_the_times_of_the_windows_behind_it() {
    let _guard = sorting();
    let mut session = Session::new(1, "s");
    let mut first = Window::new(1, "aaa", 80, 24);
    let mut second = Window::new(2, "bbb", 80, 24);
    let wl1 = link(&mut session, &mut first, 1);
    let wl2 = link(&mut session, &mut second, 2);
    unsafe {
        let aaa = first.ptr();
        let bbb = second.ptr();
        for (order, older, newer) in [(SORT_CREATION, -1, 1), (SORT_ACTIVITY, 1, -1)] {
            for (a, b) in [(at(100, 0), at(200, 0)), (at(100, 0), at(100, 5))] {
                (*aaa).set_creation_time(a);
                (*bbb).set_creation_time(b);
                (*aaa).set_activity_time(a);
                (*bbb).set_activity_time(b);
                assert_eq!(compares(sort_winlink_cmp, order, 0, wl1, wl2), older);
                assert_eq!(compares(sort_winlink_cmp, order, 0, wl2, wl1), newer);
                assert_eq!(compares(sort_winlink_cmp, order, 1, wl1, wl2), -older);
            }
            let times = (*aaa).timestamps();
            (*bbb).set_creation_time(times.creation);
            (*bbb).set_activity_time(times.activity);
            assert!(compares(sort_winlink_cmp, order, 0, wl1, wl2) < 0);
        }
    }
    unlink(&mut session, wl1);
    unlink(&mut session, wl2);
}

/// The key comparison is the odd one of the six. Its tie-break answers
/// *one* for two bindings of the same table rather than the zero the rest
/// answer for two things that are equal, its comparison by table name
/// answers that same one, and the difference it takes between two keys is
/// cut down to an `int`, which drops every modifier.
#[test]
fn a_key_binding_comparison_answers_one_for_two_bindings_of_a_table() {
    let _guards = tables();
    let mut table = Table::new("one-table");
    table.add(b'a' as key_code);
    table.add(b'b' as key_code | KEYC_CTRL);
    {
        let c = crit(SORT_END, 0);
        let l = sort_get_key_bindings(&c);
        let aaa = &l[0];
        let bbb = &l[1];
        assert_eq!(sort_key_binding_cmp(aaa, bbb, &crit(SORT_INDEX, 0)), -1);
        assert_eq!(sort_key_binding_cmp(bbb, aaa, &crit(SORT_INDEX, 0)), 1);
        assert_eq!(sort_key_binding_cmp(aaa, bbb, &crit(SORT_INDEX, 1)), 1);
        assert_eq!(sort_key_binding_cmp(aaa, bbb, &crit(SORT_MODIFIER, 0)), 1);
        assert_eq!(sort_key_binding_cmp(aaa, bbb, &crit(SORT_NAME, 0)), 1);
        assert_eq!(sort_key_binding_cmp(aaa, bbb, &crit(SORT_NAME, 1)), -1);
        for order in [SORT_ACTIVITY, SORT_CREATION, SORT_ORDER, SORT_SIZE, SORT_Z] {
            assert_eq!(sort_key_binding_cmp(aaa, bbb, &crit(order, 0)), 1);
        }
    }
}

/// A comparison that answers *greater* for every pair, which is the shape
/// the key comparison takes for two bindings of one table.
fn always_greater(_a: &*mut c_int, _b: &*mut c_int) -> c_int {
    1
}

/// A comparison by what is pointed at: a consistent order, with a tie
/// wherever two entries carry the same number.
unsafe fn by_value(a: &*mut c_int, b: &*mut c_int) -> c_int {
    unsafe { **a - **b }
}

/// One entry per number, as the list of pointers a collector would hold.
fn entries(store: &mut [c_int]) -> Vec<*mut c_int> {
    store.iter_mut().map(|v| &raw mut *v).collect()
}

/// A comparison that answers *greater* for every pair is no order at all,
/// so what comes out is the sort's own doing: splitting at half and taking
/// from the left run only while it does not compare greater turns the
/// whole list round, at every length.
#[test]
fn a_comparison_that_answers_greater_for_every_pair_turns_the_list_round() {
    for len in 0..40 {
        let mut store: Vec<c_int> = (0..len).collect();
        let mut l = entries(&mut store);
        merge_sort(&mut l, &mut always_greater);
        let seen: Vec<c_int> = l.iter().map(|p| unsafe { **p }).collect();
        assert_eq!(
            seen,
            (0..len).rev().collect::<Vec<c_int>>(),
            "{len} entries"
        );
    }
}

/// Entries that compare equal come out in the order they went in, since
/// the merge takes from the left run for as long as it does not compare
/// greater — which is what lets a list of panes or windows carrying the
/// same title stay in the order the walk found them.
#[test]
fn entries_that_compare_equal_keep_the_order_they_came_in() {
    for len in [0, 1, 2, 7, 40, 101] {
        let mut store: Vec<c_int> = (0..len).map(|i| i % 3).collect();
        let came_in = entries(&mut store);
        let mut l = came_in.clone();
        merge_sort(&mut l, &mut |a, b| unsafe { by_value(a, b) });
        let mut settled = came_in.clone();
        settled.sort_by_key(|p| unsafe { **p });
        assert_eq!(l, settled, "{len} entries");
    }
}

pub(crate) unsafe fn sort_get_panes(sort_crit: &sort_criteria_t) -> Vec<RustWindowPaneWeak> {
    unsafe {
        let mut l = Vec::new();
        for s in SESSIONS.read().values() {
            for wl in s.as_session().windows.values() {
                if let Some(window) = wl.window_handle() {
                    l.extend(panes_of(window));
                }
            }
        }
        sort_list(&mut l, sort_pane_cmp, sort_crit);
        l
    }
}

pub(crate) fn sort_get_panes_session(
    s: &session,
    sort_crit: &sort_criteria_t,
) -> Vec<RustWindowPaneWeak> {
    {
        let mut l = Vec::new();
        for wl in s.windows.values() {
            if let Some(window) = wl.window_handle() {
                l.extend(panes_of(window));
            }
        }
        sort_list(&mut l, sort_pane_cmp, sort_crit);
        l
    }
}
