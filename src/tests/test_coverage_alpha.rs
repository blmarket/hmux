//! Unit tests for functions whose modules carry no test suite of their own,
//! kept in a file of their own so that parallel efforts to widen coverage stay
//! out of each other's way.
//!
//! Four modules are covered here. [`crate::sort`] is the sort-criteria
//! engine behind choose-tree and friends; its order names, cycling and the
//! collectors over buffers, sessions, clients and winlinks are all reachable
//! without a server. [`crate::session`] contributes lookup by name and by
//! `$id` and the window renumbering pass. [`crate::terminfo`] contributes
//! capability lookup against a term built by hand. [`crate::server`]
//! contributes the access-control list, which lives in a global tree and so
//! wants [`globals`] like everything else that touches process-wide state.

use crate::session::winlink_of;
use crate::session::{session_get_curw, session_set_curw};
use crate::types::*;

use crate::options::options_set_number;
use crate::paste::{
    paste_add, paste_buffer_data, paste_buffer_name, paste_buffer_order, paste_free, paste_walk,
};
use crate::server::{
    SERVER_ACL_READONLY, server_acl_get_uid, server_acl_init, server_acl_user_allow,
    server_acl_user_allow_write, server_acl_user_deny, server_acl_user_deny_write,
    server_acl_user_find,
};
use crate::session::{
    session_alive, session_find, session_find_by_id, session_find_by_id_str,
    session_renumber_windows,
};
use crate::sort::{
    CLIENT_ATTACHED, CLIENT_DEAD, SORT_ACTIVITY, SORT_CREATION, SORT_END, SORT_INDEX,
    SORT_MODIFIER, SORT_NAME, SORT_ORDER, SORT_SIZE, SORT_Z, sort_get_buffers, sort_get_clients,
    sort_get_sessions, sort_get_winlinks, sort_next_order, sort_order_from_string,
    sort_order_to_string, sort_would_window_tree_swap,
};
use crate::terminfo::{
    TTYC_AM, TTYC_BEL, TTYC_COLORS, TTYC_XT, TtyCode, tty_term_describe, tty_term_flag,
    tty_term_has, tty_term_ncodes, tty_term_number, tty_term_string,
};
use crate::tests::test_fixtures::{
    Clients, Registry, Session, Window, globals, link, seen, unlink_all, zeroed_term,
};
use crate::window::{WINLINK_VISITED, winlink_find_by_index, winlink_find_by_window};
use ::core::ffi::{CStr, c_int};
use ::core::ptr::{null, null_mut};

/// The alert flag `alerts_queue` leaves on a winlink; renumbering carries it
/// over.
const WINLINK_BELL: c_int = 0x1;

/// A sort criterion in the shape the commands build one.
fn crit(order: u32, reversed: c_int) -> Box<sort_criteria_t> {
    Box::new(sort_criteria_t {
        order,
        reversed,
        order_seq: None,
    })
}

/// A copy of `s` on the heap, the way every caller hands a paste buffer its
/// data.
fn heap_copy(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

/// Frees every buffer in the store, so that a test leaves it as it found it.
unsafe fn empty_the_store() {
    unsafe {
        let mut pb = paste_walk(null_mut::<paste_buffer>());
        while !pb.is_null() {
            let next = paste_walk(pb);
            paste_free(pb);
            pb = next;
        }
    }
}

//
// sort.rs
//

#[test]
fn sort_order_to_string_names_each_order_and_answers_null_for_end() {
    assert_eq!(sort_order_to_string(SORT_ACTIVITY), Some(c"activity"));
    assert_eq!(sort_order_to_string(SORT_CREATION), Some(c"creation"));
    assert_eq!(sort_order_to_string(SORT_INDEX), Some(c"index"));
    assert_eq!(sort_order_to_string(SORT_MODIFIER), Some(c"modifier"));
    assert_eq!(sort_order_to_string(SORT_NAME), Some(c"name"));
    assert_eq!(sort_order_to_string(SORT_ORDER), Some(c"order"));
    assert_eq!(sort_order_to_string(SORT_SIZE), Some(c"size"));
    assert_eq!(sort_order_to_string(SORT_Z), Some(c"z"));
    assert_eq!(sort_order_to_string(SORT_END), None);
}

#[test]
fn sort_order_from_string_parses_the_names_and_their_aliases() {
    assert_eq!(sort_order_from_string(Some(c"activity")), SORT_ACTIVITY);
    assert_eq!(sort_order_from_string(Some(c"ACTIVITY")), SORT_ACTIVITY);
    assert_eq!(sort_order_from_string(Some(c"Creation")), SORT_CREATION);
    assert_eq!(sort_order_from_string(Some(c"index")), SORT_INDEX);
    assert_eq!(sort_order_from_string(Some(c"key")), SORT_INDEX);
    assert_eq!(sort_order_from_string(Some(c"modifier")), SORT_MODIFIER);
    assert_eq!(sort_order_from_string(Some(c"name")), SORT_NAME);
    assert_eq!(sort_order_from_string(Some(c"title")), SORT_NAME);
    assert_eq!(sort_order_from_string(Some(c"order")), SORT_ORDER);
    assert_eq!(sort_order_from_string(Some(c"size")), SORT_SIZE);
    assert_eq!(sort_order_from_string(Some(c"z")), SORT_Z);
    assert_eq!(sort_order_from_string(Some(c"no-such-order")), SORT_END);
    assert_eq!(sort_order_from_string(None), SORT_END);
}

#[test]
fn sort_next_order_walks_the_sequence_and_wraps_at_both_ends() {
    let seq: &[u32] = &[SORT_ACTIVITY, SORT_NAME];

    let mut c = crit(SORT_ACTIVITY, 0);
    c.order_seq = Some(seq);
    sort_next_order(&mut c);
    assert_eq!(c.order, SORT_NAME);
    sort_next_order(&mut c);
    assert_eq!(c.order, SORT_ACTIVITY);

    // An order the sequence does not hold restarts from its first entry.
    let mut c = crit(SORT_SIZE, 0);
    c.order_seq = Some(seq);
    sort_next_order(&mut c);
    assert_eq!(c.order, SORT_ACTIVITY);

    // The last entry of a sequence wraps back to the first.
    let mut c = crit(SORT_NAME, 0);
    c.order_seq = Some(&[SORT_NAME]);
    sort_next_order(&mut c);
    assert_eq!(c.order, SORT_NAME);

    // Without a sequence there is nothing to walk.
    let mut c = crit(SORT_Z, 0);
    sort_next_order(&mut c);
    assert_eq!(c.order, SORT_Z);
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
    let mut c = crit(SORT_INDEX, 0);
    unsafe {
        assert_eq!(sort_would_window_tree_swap(&c, wl0, wl1), 0);
    }

    // Name order swaps two winlinks whose windows differ in name.
    let mut c = crit(SORT_NAME, 0);
    unsafe {
        assert_eq!(sort_would_window_tree_swap(&c, wl0, wl1), 1);
    }

    // Two windows with the same name compare equal, so nothing swaps.
    let mut twin = Window::new(3, "first", 80, 24);
    let wl2 = link(&mut s, &mut twin, 2);
    let mut c = crit(SORT_NAME, 0);
    unsafe {
        assert_eq!(sort_would_window_tree_swap(&c, wl0, wl2), 0);
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
        if session_get_curw(s.ptr()) == wl {
            session_set_curw(s.ptr(), null_mut::<winlink>());
        }
        crate::window::winlink_remove(&raw mut (*s.ptr()).windows, wl);
    }
}

#[test]
fn sort_get_buffers_orders_the_buffer_store_by_its_criteria() {
    let _guard = globals();
    unsafe {
        empty_the_store();
        paste_add(null(), heap_copy("bbb"));
        paste_add(null(), heap_copy("cccccccc"));
        paste_add(null(), heap_copy("eeeee"));

        // Size order puts the smallest buffer first, largest last.
        let mut c = crit(SORT_SIZE, 0);
        let l = sort_get_buffers(&c);
        assert_eq!(l.len(), 3);
        assert_eq!(
            l.iter()
                .map(|pb| paste_buffer_data(&**pb).len())
                .collect::<Vec<_>>(),
            vec![3, 5, 8]
        );

        // Reversed size order flips it.
        let mut c = crit(SORT_SIZE, 1);
        let l = sort_get_buffers(&c);
        assert_eq!(
            l.iter()
                .map(|pb| paste_buffer_data(&**pb).len())
                .collect::<Vec<_>>(),
            vec![8, 5, 3]
        );

        // Name order breaks ties and sorts lexicographically; the automatic
        // names share their prefix, so they come out oldest first.
        let mut c = crit(SORT_NAME, 0);
        let l = sort_get_buffers(&c);
        let names: Vec<String> = l
            .iter()
            .map(|pb| paste_buffer_name(&**pb).to_string_lossy().into_owned())
            .collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);

        // Order order keeps the store's own order — newest first — unless it
        // is reversed, which makes it oldest first.
        let mut c = crit(SORT_ORDER, 0);
        let l = sort_get_buffers(&c);
        let orders: Vec<u32> = l.iter().map(|pb| paste_buffer_order(&**pb)).collect();
        let mut newest_first = orders.clone();
        newest_first.reverse();
        assert_ne!(orders, newest_first);
        let mut c = crit(SORT_ORDER, 1);
        let l = sort_get_buffers(&c);
        let reversed: Vec<u32> = l.iter().map(|pb| paste_buffer_order(&**pb)).collect();
        assert_eq!(reversed, newest_first);

        // An unusable criterion sorts nothing at all.
        let mut c = crit(SORT_END, 0);
        let l = sort_get_buffers(&c);
        assert_eq!(l.len(), 3);
        let untouched: Vec<u32> = l.iter().map(|pb| paste_buffer_order(&**pb)).collect();
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
        let mut c = crit(SORT_NAME, 0);
        let l = sort_get_sessions(&c);
        assert_eq!(l.len(), 2);
        assert_eq!(l, vec![alpha.ptr(), beta.ptr()]);

        // Creation order puts the older session first.
        let mut c = crit(SORT_CREATION, 0);
        let l = sort_get_sessions(&c);
        assert_eq!(l, vec![beta.ptr(), alpha.ptr()]);
        let mut c = crit(SORT_CREATION, 1);
        let l = sort_get_sessions(&c);
        assert_eq!(l, vec![alpha.ptr(), beta.ptr()]);

        // Index order is the session id.
        let mut c = crit(SORT_INDEX, 0);
        let l = sort_get_sessions(&c);
        assert_eq!(l, vec![beta.ptr(), alpha.ptr()]);
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
        let mut c = crit(SORT_NAME, 0);
        let l = sort_get_clients(&c);
        assert_eq!(l.len(), 2);
        assert_eq!(l, vec![alpha, zeta]);

        // Size order compares the terminal, width before height.
        let mut c = crit(SORT_SIZE, 0);
        let l = sort_get_clients(&c);
        assert_eq!(l, vec![alpha, zeta]);
        let mut c = crit(SORT_SIZE, 1);
        let l = sort_get_clients(&c);
        assert_eq!(l, vec![zeta, alpha]);
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
    let wla0 = link(&mut sa, &mut one, 0);
    let wla1 = link(&mut sa, &mut two, 1);
    let wlb0 = link(&mut sb, &mut three, 0);
    unsafe {
        // Index order sorts every winlink together, by index and then by the
        // window name where two sessions hold the same index.
        let mut c = crit(SORT_INDEX, 0);
        let l = sort_get_winlinks(&c);
        assert_eq!(l.len(), 3);
        assert_eq!(l, vec![wla0, wlb0, wla1]);

        // Name order follows the windows instead.
        let mut c = crit(SORT_NAME, 0);
        let l = sort_get_winlinks(&c);
        assert_eq!(
            l.iter()
                .map(|wl| seen(cstr_ptr(&(*(**wl).window()).name)))
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
        assert_eq!(session_alive(s.ptr()), 0);
        registry.add_session(&mut s);
        assert_eq!(session_alive(s.ptr()), 1);
    }
}

#[test]
fn session_find_looks_up_by_name() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(1, "found");
    registry.add_session(&mut s);
    unsafe {
        assert_eq!(session_find(c"found".as_ptr()), s.ptr());
        assert!(session_find(c"missing".as_ptr()).is_null());
    }
}

#[test]
fn session_find_by_id_str_parses_dollar_ids() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(7, "seven");
    registry.add_session(&mut s);
    unsafe {
        assert_eq!(session_find_by_id_str(c"$7".as_ptr()), s.ptr());
        assert_eq!(session_find_by_id(7), s.ptr());

        // Without the dollar sign it is not an id at all.
        assert!(session_find_by_id_str(c"7".as_ptr()).is_null());
        // A number that will not parse answers nothing.
        assert!(session_find_by_id_str(c"$x".as_ptr()).is_null());
        assert!(session_find_by_id_str(c"$4294967296".as_ptr()).is_null());
        // A well-formed id nobody carries answers nothing.
        assert!(session_find_by_id_str(c"$8".as_ptr()).is_null());
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
    let _wl5 = link(&mut s, &mut first, 5);
    let wl9 = link(&mut s, &mut second, 9);
    unsafe {
        (*wl9).flags |= WINLINK_BELL;
        (*s.ptr()).lastw.push((*wl9).idx);

        session_renumber_windows(s.ptr());

        let n0 = winlink_find_by_index(&raw mut (*s.ptr()).windows, 0);
        let n1 = winlink_find_by_index(&raw mut (*s.ptr()).windows, 1);
        assert!(!n0.is_null() && !n1.is_null());
        assert_eq!((*n0).window(), first.ptr());
        assert_eq!((*n1).window(), second.ptr());
        assert_eq!((*n1).flags & WINLINK_BELL, WINLINK_BELL);
        assert_eq!(session_get_curw(s.ptr()), n0);
        assert_eq!(winlink_of(s.ptr(), (*s.ptr()).lastw.first().copied()), n1);
        assert_eq!((*n1).flags & WINLINK_VISITED, WINLINK_VISITED);
        assert!(winlink_find_by_index(&raw mut (*s.ptr()).windows, 5).is_null());
        assert_eq!(
            winlink_find_by_window(&raw mut (*s.ptr()).windows, first.ptr()),
            n0
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
        options_set_number(s.options(), c"base-index".as_ptr(), 1);
        session_renumber_windows(s.ptr());
        assert_eq!(
            (*winlink_find_by_index(&raw mut (*s.ptr()).windows, 1)).window(),
            first.ptr()
        );
        assert_eq!(
            (*winlink_find_by_index(&raw mut (*s.ptr()).windows, 2)).window(),
            second.ptr()
        );
        assert!(winlink_find_by_index(&raw mut (*s.ptr()).windows, 0).is_null());
    }
    unlink_all(&mut s);
}

//
// tty_term.rs
//

/// A terminal carrying nothing but a full-length table of absent capabilities,
/// built here rather than borrowed from the fixtures so that this file stands
/// on its own.
struct Term {
    term: Box<tty_term>,
}

impl Term {
    fn new() -> Term {
        Term {
            term: zeroed_term(),
        }
    }

    fn ptr(&mut self) -> &tty_term {
        &self.term
    }

    fn set_string(&mut self, code: tty_code_code, s: &'static CStr) {
        self.term.codes[code as usize] = TtyCode::String(s.to_owned());
    }

    fn set_number(&mut self, code: tty_code_code, number: c_int) {
        self.term.codes[code as usize] = TtyCode::Number(number);
    }

    fn set_flag(&mut self, code: tty_code_code, flag: c_int) {
        self.term.codes[code as usize] = TtyCode::Flag(flag);
    }
}

#[test]
fn tty_term_ncodes_is_the_length_of_the_code_table() {
    {
        assert_eq!(tty_term_ncodes(), 233);
    }
}

#[test]
fn an_absent_capability_answers_nothing() {
    let mut t = Term::new();
    unsafe {
        assert_eq!(tty_term_has(t.ptr(), TTYC_BEL), 0);
        assert_eq!(seen(tty_term_string(t.ptr(), TTYC_BEL)), "");
        assert_eq!(tty_term_number(t.ptr(), TTYC_COLORS), 0);
        assert_eq!(tty_term_flag(t.ptr(), TTYC_AM), 0);
    }
}

#[test]
fn tty_term_reads_back_whatever_kind_of_capability_it_carries() {
    let mut t = Term::new();
    t.set_string(TTYC_BEL, c"\x07");
    t.set_number(TTYC_COLORS, 256);
    t.set_flag(TTYC_AM, 1);
    unsafe {
        assert_eq!(tty_term_has(t.ptr(), TTYC_BEL), 1);
        assert_eq!(seen(tty_term_string(t.ptr(), TTYC_BEL)), "\x07");
        assert_eq!(tty_term_number(t.ptr(), TTYC_COLORS), 256);
        assert_eq!(tty_term_flag(t.ptr(), TTYC_AM), 1);

        // An absent capability still reads as present-but-empty once another
        // of the same kind is set.
        assert_eq!(tty_term_has(t.ptr(), TTYC_XT), 0);
    }
}

#[test]
fn tty_term_describe_describes_each_kind_of_capability() {
    let mut t = Term::new();
    unsafe {
        assert_eq!(
            tty_term_describe(t.ptr(), TTYC_AM)
                .to_string_lossy()
                .into_owned(),
            "   1: am: [missing]"
        );

        t.set_string(TTYC_BEL, c"hi");
        assert_eq!(
            tty_term_describe(t.ptr(), TTYC_BEL)
                .to_string_lossy()
                .into_owned(),
            "   4: bel: (string) hi"
        );

        t.set_number(TTYC_COLORS, 256);
        assert_eq!(
            tty_term_describe(t.ptr(), TTYC_COLORS)
                .to_string_lossy()
                .into_owned(),
            "  13: colors: (number) 256"
        );

        t.set_flag(TTYC_AM, 1);
        assert_eq!(
            tty_term_describe(t.ptr(), TTYC_AM)
                .to_string_lossy()
                .into_owned(),
            "   1: am: (flag) true"
        );

        t.set_flag(TTYC_XT, 0);
        assert_eq!(
            tty_term_describe(t.ptr(), TTYC_XT)
                .to_string_lossy()
                .into_owned(),
            " 232: XT: (flag) false"
        );
    }
}

//
// server_acl.rs
//

#[test]
fn server_acl_user_allow_registers_a_user_exactly_once() {
    let _guard = globals();
    unsafe {
        server_acl_init();
        server_acl_user_allow(4242 as uid_t);
        let user = server_acl_user_find(4242 as uid_t);
        assert!(!user.is_null());
        assert_eq!(server_acl_get_uid(user), 4242 as uid_t);

        // Allowing the same user again keeps the one entry.
        server_acl_user_allow(4242 as uid_t);
        assert_eq!(server_acl_user_find(4242 as uid_t), user);

        // A user who was never allowed is not found.
        assert!(server_acl_user_find(5252 as uid_t).is_null());
    }
}

#[test]
fn server_acl_user_deny_removes_a_user_and_forgives_strangers() {
    let _guard = globals();
    {
        server_acl_init();
        server_acl_user_allow(4242 as uid_t);
        server_acl_user_deny(4242 as uid_t);
        assert!(server_acl_user_find(4242 as uid_t).is_null());

        // Denying somebody who was never allowed does nothing.
        server_acl_user_deny(5252 as uid_t);
        assert!(server_acl_user_find(5252 as uid_t).is_null());
    }
}

#[test]
fn server_acl_write_permission_flips_with_the_user_flags() {
    let _guard = globals();
    unsafe {
        server_acl_init();
        server_acl_user_allow(4242 as uid_t);
        let user = server_acl_user_find(4242 as uid_t);
        assert_eq!((*user).flags & SERVER_ACL_READONLY, 0);

        server_acl_user_deny_write(4242 as uid_t);
        assert_eq!((*user).flags & SERVER_ACL_READONLY, SERVER_ACL_READONLY);
        server_acl_user_allow_write(4242 as uid_t);
        assert_eq!((*user).flags & SERVER_ACL_READONLY, 0);

        // Either call on a stranger is quietly ignored.
        server_acl_user_deny_write(5252 as uid_t);
        server_acl_user_allow_write(5252 as uid_t);
        assert!(server_acl_user_find(5252 as uid_t).is_null());
    }
}

#[test]
fn server_acl_init_allows_this_user_and_clears_everything_else() {
    let _guard = globals();
    unsafe {
        server_acl_user_allow(4242 as uid_t);
        server_acl_init();

        // The list starts from scratch...
        assert!(server_acl_user_find(4242 as uid_t).is_null());
        // ...and carries the server's own user, plus root when the server is
        // not root itself.
        assert!(!server_acl_user_find(::libc::getuid()).is_null());
        if ::libc::getuid() != 0 {
            assert!(!server_acl_user_find(0).is_null());
        }
    }
}
