//! Unit tests for functions whose modules carry no test suite of their own,
//! kept in a file of their own so that parallel efforts to widen coverage stay
//! out of each other's way.
//!
//! Four modules are covered here. [`crate::sort`] is the sort-criteria
//! engine behind choose-tree and friends; its order names, cycling and the
//! collectors over buffers, sessions, clients and winlinks are all reachable
//! without a server. [`crate::session`] contributes lookup by name and by
//! `$id` and the window renumbering pass. [`crate::server`] contributes the
//! process-owner initialization of its access-control list.

use crate::paste::{PasteBufferStore, with_paste_buffers, with_paste_buffers_mut};
use crate::server::{ServerAclStore, server_acl_init, with_server_acl, with_server_acl_mut};
use crate::types::*;

use crate::sort::{
    CLIENT_ATTACHED, CLIENT_DEAD, RustSortCriteria, SORT_ACTIVITY, SORT_CREATION, SORT_END,
    SORT_INDEX, SORT_MODIFIER, SORT_NAME, SORT_ORDER, SORT_SIZE, SORT_Z, SortCriteria,
    sort_get_buffers, sort_get_clients, sort_get_sessions, sort_get_winlinks,
    sort_would_window_tree_swap,
};
use crate::tests::test_fixtures::{Clients, Registry, Session, Window, globals, link, unlink_all};
use crate::window::{WINLINK_VISITED, winlink_find_by_window};
use ::core::ffi::c_int;
use ::core::ptr::null_mut;

/// The alert flag `alerts_queue` leaves on a winlink; renumbering carries it
/// over.
const WINLINK_BELL: c_int = 0x1;

/// A sort criterion in the shape the commands build one.
fn crit(order: u32, reversed: c_int) -> Box<sort_criteria_t> {
    Box::new(RustSortCriteria::new(order, reversed != 0))
}

/// Frees every buffer in the store, so that a test leaves it as it found it.
fn empty_the_store() {
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

//
// sort.rs
//

#[test]
fn sort_order_to_string_names_each_order_and_answers_null_for_end() {
    assert_eq!(
        RustSortCriteria::order_name(SORT_ACTIVITY),
        Some(c"activity")
    );
    assert_eq!(
        RustSortCriteria::order_name(SORT_CREATION),
        Some(c"creation")
    );
    assert_eq!(RustSortCriteria::order_name(SORT_INDEX), Some(c"index"));
    assert_eq!(
        RustSortCriteria::order_name(SORT_MODIFIER),
        Some(c"modifier")
    );
    assert_eq!(RustSortCriteria::order_name(SORT_NAME), Some(c"name"));
    assert_eq!(RustSortCriteria::order_name(SORT_ORDER), Some(c"order"));
    assert_eq!(RustSortCriteria::order_name(SORT_SIZE), Some(c"size"));
    assert_eq!(RustSortCriteria::order_name(SORT_Z), Some(c"z"));
    assert_eq!(RustSortCriteria::order_name(SORT_END), None);
}

#[test]
fn sort_order_from_string_parses_the_names_and_their_aliases() {
    assert_eq!(
        RustSortCriteria::parse_order(Some(c"activity")),
        SORT_ACTIVITY
    );
    assert_eq!(
        RustSortCriteria::parse_order(Some(c"ACTIVITY")),
        SORT_ACTIVITY
    );
    assert_eq!(
        RustSortCriteria::parse_order(Some(c"Creation")),
        SORT_CREATION
    );
    assert_eq!(RustSortCriteria::parse_order(Some(c"index")), SORT_INDEX);
    assert_eq!(RustSortCriteria::parse_order(Some(c"key")), SORT_INDEX);
    assert_eq!(
        RustSortCriteria::parse_order(Some(c"modifier")),
        SORT_MODIFIER
    );
    assert_eq!(RustSortCriteria::parse_order(Some(c"name")), SORT_NAME);
    assert_eq!(RustSortCriteria::parse_order(Some(c"title")), SORT_NAME);
    assert_eq!(RustSortCriteria::parse_order(Some(c"order")), SORT_ORDER);
    assert_eq!(RustSortCriteria::parse_order(Some(c"size")), SORT_SIZE);
    assert_eq!(RustSortCriteria::parse_order(Some(c"z")), SORT_Z);
    assert_eq!(
        RustSortCriteria::parse_order(Some(c"no-such-order")),
        SORT_END
    );
    assert_eq!(RustSortCriteria::parse_order(None), SORT_END);
}

#[test]
fn sort_next_order_walks_the_sequence_and_wraps_at_both_ends() {
    let seq: &[u32] = &[SORT_ACTIVITY, SORT_NAME];

    let mut c = crit(SORT_ACTIVITY, 0);
    c.set_cycle(seq);
    c.advance();
    assert_eq!(c.order(), SORT_NAME);
    c.advance();
    assert_eq!(c.order(), SORT_ACTIVITY);

    // An order the sequence does not hold restarts from its first entry.
    let mut c = crit(SORT_SIZE, 0);
    c.set_cycle(seq);
    c.advance();
    assert_eq!(c.order(), SORT_ACTIVITY);

    // The last entry of a sequence wraps back to the first.
    let mut c = crit(SORT_NAME, 0);
    c.set_cycle(&[SORT_NAME]);
    c.advance();
    assert_eq!(c.order(), SORT_NAME);

    // Without a sequence there is nothing to walk.
    let mut c = crit(SORT_Z, 0);
    c.advance();
    assert_eq!(c.order(), SORT_Z);
}

#[test]
fn sort_would_window_tree_swap_compares_windows_by_its_criteria() {
    let _guard = globals();
    let mut s = Session::new(1, "swap");
    let mut first = Window::new(1, "first", 80, 24);
    let mut second = Window::new(2, "second", 80, 24);
    let wl0 = link(&mut s, &mut first, 0);
    let wl1 = link(&mut s, &mut second, 1);

    // Index order never swaps: the tree already holds winlinks by index.
    let c = crit(SORT_INDEX, 0);
    unsafe {
        assert_eq!(sort_would_window_tree_swap(&c, &*wl0, &*wl1), 0);
    }

    // Name order swaps two winlinks whose windows differ in name.
    let c = crit(SORT_NAME, 0);
    unsafe {
        assert_eq!(sort_would_window_tree_swap(&c, &*wl0, &*wl1), 1);
    }

    // Two windows with the same name compare equal, so nothing swaps.
    let mut twin = Window::new(3, "first", 80, 24);
    let wl2 = link(&mut s, &mut twin, 2);
    let c = crit(SORT_NAME, 0);
    unsafe {
        assert_eq!(sort_would_window_tree_swap(&c, &*wl0, &*wl2), 0);
    }

    unsafe {
        unlink_for_test(&mut s, wl0);
        unlink_for_test(&mut s, wl1);
        unlink_for_test(&mut s, wl2);
    }
}

/// Takes one winlink back out of its session, freeing it, without touching the
/// session's current window.
unsafe fn unlink_for_test(s: &mut Session, wl: *mut winlink) {
    unsafe {
        if s.handle().curw().is_some_and(|current| {
            current
                .get()
                .is_some_and(|current| core::ptr::eq(current, wl))
        }) {
            s.handle().set_curw(null_mut::<winlink>().as_ref());
        }
        crate::window::winlink_remove(&mut (*s.ptr()).windows, (*wl).idx);
    }
}

#[test]
fn sort_get_buffers_orders_the_buffer_store_by_its_criteria() {
    let _guard = globals();
    {
        empty_the_store();
        with_paste_buffers_mut(|buffers| {
            buffers.add_automatic(None, b"bbb".to_vec(), 50);
            buffers.add_automatic(None, b"cccccccc".to_vec(), 50);
            buffers.add_automatic(None, b"eeeee".to_vec(), 50);
        });

        // Size order puts the smallest buffer first, largest last.
        let c = crit(SORT_SIZE, 0);
        let l = sort_get_buffers(&c);
        assert_eq!(l.len(), 3);
        assert_eq!(
            l.iter().map(|buffer| buffer.size).collect::<Vec<_>>(),
            vec![3, 5, 8]
        );

        // Reversed size order flips it.
        let c = crit(SORT_SIZE, 1);
        let l = sort_get_buffers(&c);
        assert_eq!(
            l.iter().map(|buffer| buffer.size).collect::<Vec<_>>(),
            vec![8, 5, 3]
        );

        // Name order breaks ties and sorts lexicographically; the automatic
        // names share their prefix, so they come out oldest first.
        let c = crit(SORT_NAME, 0);
        let l = sort_get_buffers(&c);
        let names: Vec<String> = l
            .iter()
            .map(|buffer| buffer.name.to_string_lossy().into_owned())
            .collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);

        // Order order keeps the store's own order — newest first — unless it
        // is reversed, which makes it oldest first.
        let c = crit(SORT_ORDER, 0);
        let l = sort_get_buffers(&c);
        let orders: Vec<u32> = l.iter().map(|buffer| buffer.order).collect();
        let mut newest_first = orders.clone();
        newest_first.reverse();
        assert_ne!(orders, newest_first);
        let c = crit(SORT_ORDER, 1);
        let l = sort_get_buffers(&c);
        let reversed: Vec<u32> = l.iter().map(|buffer| buffer.order).collect();
        assert_eq!(reversed, newest_first);

        // An unusable criterion sorts nothing at all.
        let c = crit(SORT_END, 0);
        let l = sort_get_buffers(&c);
        assert_eq!(l.len(), 3);
        let untouched: Vec<u32> = l.iter().map(|buffer| buffer.order).collect();
        assert_eq!(untouched, orders);

        empty_the_store();
    }
}

#[test]
fn sort_get_sessions_orders_the_session_tree_by_its_criteria() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut beta = Session::new(1, "beta");
    let mut alpha = Session::new(2, "alpha");
    registry.add_session(&mut beta);
    registry.add_session(&mut alpha);
    unsafe {
        (*beta.ptr()).creation_time.tv_sec = 100;
        (*alpha.ptr()).creation_time.tv_sec = 200;

        // Name order.
        let c = crit(SORT_NAME, 0);
        let l = sort_get_sessions(&c);
        assert_eq!(l.len(), 2);
        assert_eq!(
            l.iter().map(SessionRef::as_ptr).collect::<Vec<_>>(),
            vec![alpha.ptr(), beta.ptr()]
        );

        // Creation order puts the older session first.
        let c = crit(SORT_CREATION, 0);
        let l = sort_get_sessions(&c);
        assert_eq!(
            l.iter().map(SessionRef::as_ptr).collect::<Vec<_>>(),
            vec![beta.ptr(), alpha.ptr()]
        );
        let c = crit(SORT_CREATION, 1);
        let l = sort_get_sessions(&c);
        assert_eq!(
            l.iter().map(SessionRef::as_ptr).collect::<Vec<_>>(),
            vec![alpha.ptr(), beta.ptr()]
        );

        // Index order is the session id.
        let c = crit(SORT_INDEX, 0);
        let l = sort_get_sessions(&c);
        assert_eq!(
            l.iter().map(SessionRef::as_ptr).collect::<Vec<_>>(),
            vec![beta.ptr(), alpha.ptr()]
        );
    }
}

#[test]
fn sort_get_clients_skips_unusable_clients_and_orders_the_rest() {
    let _guard = globals();
    let mut clients = Clients::new();
    let zeta = clients.add("zeta", 100, 50);
    let alpha = clients.add("alpha", 80, 24);
    let dead = clients.add("dead", 10, 10);
    unsafe {
        (*zeta).flags = CLIENT_ATTACHED as u64;
        (*alpha).flags = CLIENT_ATTACHED as u64;
        (*dead).flags = (CLIENT_ATTACHED | CLIENT_DEAD) as u64;

        // The dead client is left out; the rest come in name order.
        let c = crit(SORT_NAME, 0);
        let l = sort_get_clients(&c);
        assert_eq!(l.len(), 2);
        assert_eq!(
            l.iter().map(ClientRef::as_ptr).collect::<Vec<_>>(),
            vec![alpha, zeta]
        );

        // Size order compares the terminal, width before height.
        let c = crit(SORT_SIZE, 0);
        let l = sort_get_clients(&c);
        assert_eq!(
            l.iter().map(ClientRef::as_ptr).collect::<Vec<_>>(),
            vec![alpha, zeta]
        );
        let c = crit(SORT_SIZE, 1);
        let l = sort_get_clients(&c);
        assert_eq!(
            l.iter().map(ClientRef::as_ptr).collect::<Vec<_>>(),
            vec![zeta, alpha]
        );
    }
}

#[test]
fn sort_get_winlinks_lists_every_winlink_of_every_session() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut sa = Session::new(1, "asess");
    let mut sb = Session::new(2, "bsess");
    registry.add_session(&mut sa);
    registry.add_session(&mut sb);
    let mut one = Window::new(1, "one", 80, 24);
    let mut two = Window::new(2, "two", 80, 24);
    let mut three = Window::new(3, "three", 80, 24);
    let _wla0 = link(&mut sa, &mut one, 0);
    let _wla1 = link(&mut sa, &mut two, 1);
    let _wlb0 = link(&mut sb, &mut three, 0);
    {
        // Index order sorts every winlink together, by index and then by the
        // window name where two sessions hold the same index.
        let c = crit(SORT_INDEX, 0);
        let l = sort_get_winlinks(&c);
        assert_eq!(l.len(), 3);
        assert_eq!(
            l.iter()
                .map(|wl| (wl.session().id(), wl.index()))
                .collect::<Vec<_>>(),
            [(1, 0), (2, 0), (1, 1)]
        );

        // Name order follows the windows instead.
        let c = crit(SORT_NAME, 0);
        let l = sort_get_winlinks(&c);
        assert_eq!(
            l.iter()
                .map(|wl| {
                    wl.get()
                        .unwrap()
                        .window_handle()
                        .unwrap()
                        .window_name()
                        .as_deref()
                        .expect("a window has a name")
                        .to_string_lossy()
                        .into_owned()
                })
                .collect::<Vec<_>>(),
            vec!["one", "three", "two"]
        );
    }
}

//
// session.rs
//

#[test]
fn session_alive_answers_for_sessions_in_the_tree_only() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(1, "alive");
    unsafe {
        assert_eq!(
            i32::from(
                (s.ptr().as_ref())
                    .and_then(crate::session::session_ref_of)
                    .is_some_and(|session| session.is_registered())
            ),
            0
        );
        registry.add_session(&mut s);
        assert_eq!(
            i32::from(
                (s.ptr().as_ref())
                    .and_then(crate::session::session_ref_of)
                    .is_some_and(|session| session.is_registered())
            ),
            1
        );
    }
}

#[test]
fn session_find_looks_up_by_name() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(1, "found");
    registry.add_session(&mut s);
    assert!(SessionRef::find(c"found").is_some_and(|found| found.as_ptr() == s.ptr()));
    assert!(SessionRef::find(c"missing").is_none());
}

#[test]
fn session_find_by_id_str_parses_dollar_ids() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(7, "seven");
    registry.add_session(&mut s);
    {
        assert!(SessionRef::find_by_id_str(c"$7").is_some_and(|found| found.as_ptr() == s.ptr()));
        assert!(SessionRef::find_by_id(7).is_some_and(|found| found.as_ptr() == s.ptr()));

        // Without the dollar sign it is not an id at all.
        assert!(SessionRef::find_by_id_str(c"7").is_none());
        // A number that will not parse answers nothing.
        assert!(SessionRef::find_by_id_str(c"$x").is_none());
        assert!(SessionRef::find_by_id_str(c"$4294967296").is_none());
        // A well-formed id nobody carries answers nothing.
        assert!(SessionRef::find_by_id_str(c"$8").is_none());
    }
}

#[test]
fn session_renumber_windows_reindexes_from_base_index() {
    let _guard = globals();

    // With the default base index the windows come out at 0 and 1, keeping
    // their order, the current window, the alert flags and the last-visited
    // stack.
    let mut s = Session::new(1, "renumber");
    let mut first = Window::new(1, "first", 80, 24);
    let mut second = Window::new(2, "second", 80, 24);
    link(&mut s, &mut first, 5);
    link(&mut s, &mut second, 9);
    unsafe {
        let mut owner = s.reference();
        let session = owner.as_session_mut();
        session.windows.get_mut(&9).unwrap().flags |= WINLINK_BELL;
        session.lastw.push(9);

        crate::session::session_ref_of(session)
            .expect("session owner")
            .renumber_windows();

        let n0 = session
            .windows
            .get(&0)
            .map(Box::as_ref)
            .expect("the indexed window is linked");
        let n1 = session
            .windows
            .get(&1)
            .map(Box::as_ref)
            .expect("the indexed window is linked");

        assert!(n0.window_handle().unwrap().ptr_eq(first.handle()));
        assert!(n1.window_handle().unwrap().ptr_eq(second.handle()));
        assert_eq!(n1.flags & WINLINK_BELL, WINLINK_BELL);
        assert!(core::ptr::eq(session.curw().unwrap(), n0));
        assert!(
            session
                .lastw
                .first()
                .and_then(|index| session.windows.get(index))
                .is_some_and(|link| core::ptr::eq(link.as_ref(), n1))
        );
        assert_eq!(n1.flags & WINLINK_VISITED, WINLINK_VISITED);
        assert!(session.windows.get(&5).is_none());
        assert!(
            winlink_find_by_window(&session.windows, &first.handle().as_window())
                .is_some_and(|link| core::ptr::eq(link, n0))
        );
    }
    unlink_all(&mut s);

    // A base index of one shifts every window up by one instead.
    let mut s = Session::new(2, "renumber-one");
    let mut first = Window::new(3, "first", 80, 24);
    let mut second = Window::new(4, "second", 80, 24);
    let _ = link(&mut s, &mut first, 0);
    let _ = link(&mut s, &mut second, 1);
    unsafe {
        s.options().set_number(c"base-index", 1);
        let mut owner = s.reference();
        let session = owner.as_session_mut();
        crate::session::session_ref_of(session)
            .expect("session owner")
            .renumber_windows();
        assert!(
            session
                .windows
                .get(&1)
                .expect("the indexed window is linked")
                .window_handle()
                .unwrap()
                .ptr_eq(first.handle())
        );
        assert!(
            session
                .windows
                .get(&2)
                .expect("the indexed window is linked")
                .window_handle()
                .unwrap()
                .ptr_eq(second.handle())
        );
        assert!(session.windows.get(&0).is_none());
    }
    unlink_all(&mut s);
}

//
// server_acl.rs
//

#[test]
fn server_acl_init_allows_this_user_and_clears_everything_else() {
    let _guard = globals();
    unsafe {
        with_server_acl_mut(|acl| acl.allow(4242 as uid_t));
        server_acl_init();

        // The list starts from scratch...
        assert!(with_server_acl(|acl| acl.find(4242 as uid_t)).is_none());
        // ...and carries the server's own user, plus root when the server is
        // not root itself.
        assert!(with_server_acl(|acl| acl.find(libc::getuid())).is_some());
        if libc::getuid() != 0 {
            assert!(with_server_acl(|acl| acl.find(0)).is_some());
        }
    }
}
