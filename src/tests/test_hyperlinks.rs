use super::*;
use crate::tests::test_fixtures::{globals, seen};
use ::std::ffi::CString;
use ::std::sync::{Mutex, MutexGuard};

/// A turn at the global list every set's links hang in, starting from no
/// links at all, and at the server-wide state the other suites put links
/// in from. That list and its count are this module's own globals but the
/// grid, screen and capture-pane suites reach them too, so every test
/// holds both, always in this order.
fn store() -> (MutexGuard<'static, ()>, MutexGuard<'static, ()>) {
    static LINKS: Mutex<()> = Mutex::new(());
    let outer = globals();
    LINKS.clear_poison();
    let inner = LINKS.lock().expect("just cleared any poison");
    assert!(
        global_hyperlinks.queue().is_empty(),
        "a test left links behind"
    );
    (outer, inner)
}

/// A hyperlink set, freed at the end of the test.
struct Links(HyperlinksRef);

impl Links {
    fn new() -> Links {
        Links(HyperlinksRef::new())
    }

    fn ptr(&self) -> *mut hyperlinks {
        self.0.as_ptr()
    }

    /// The inner number `uri` was given under `id`, or under no id at all
    /// when `id` is `None`.
    fn put(&self, uri: &str, id: Option<&str>) -> u_int {
        let uri = CString::new(uri).expect("no NUL");
        let id = id.map(|s| CString::new(s).expect("no NUL"));
        unsafe { hyperlinks_put(&self.0, &uri, id.as_deref()) }
    }

    /// The URI, internal id and external id stored under `inner`.
    fn get(&self, inner: u_int) -> Option<(String, String, String)> {
        unsafe {
            let (uri, internal, external) = hyperlinks_get(&*self.ptr(), inner)?;
            Some((
                seen(uri.as_ptr()),
                seen(internal.as_ptr()),
                seen(external.as_ptr()),
            ))
        }
    }

    /// How many links this set holds.
    fn len(&self) -> usize {
        unsafe { (*self.ptr()).by_inner.len() }
    }
}

/// The internal ids in the by-URI tree, in key order.
unsafe fn by_uri_in_order(hl: *mut hyperlinks, out: &mut Vec<String>) {
    unsafe {
        for inner in (*hl).by_uri.values() {
            let hlu = (*hl).by_inner.get(inner).expect("the link is held");
            out.push(seen(cstr_ptr(&hlu.internal_id)));
        }
    }
}

/// The inner numbers on the global list, oldest first.
fn listed() -> Vec<u_int> {
    unsafe {
        global_hyperlinks
            .queue()
            .iter()
            .map(|link| link.inner)
            .collect()
    }
}

#[test]
fn a_new_set_holds_nothing_and_numbers_from_one() {
    let _guard = store();
    let hl = Links::new();
    unsafe {
        assert_eq!((*hl.ptr()).next_inner, 1);
        assert!((*hl.ptr()).by_uri.is_empty());
        assert!((*hl.ptr()).by_inner.is_empty());
    }
    assert_eq!(hl.len(), 0);
    assert_eq!(hl.get(1), None);
}

#[test]
fn a_uri_with_an_id_is_handed_back_the_same_inner() {
    let _guard = store();
    let hl = Links::new();
    let first = hl.put("http://a", Some("id1"));
    assert_eq!(first, 1);
    assert_eq!(hl.put("http://a", Some("id1")), first);
    assert_eq!(hl.len(), 1);
    assert_eq!(global_hyperlinks.queue().len(), 1);
    assert_eq!(listed(), vec![first]);
    let second = hl.put("http://b", Some("id2"));
    assert_eq!(listed(), vec![first, second]);
}

#[test]
fn the_same_id_with_another_uri_is_a_link_of_its_own() {
    let _guard = store();
    let hl = Links::new();
    assert_eq!(hl.put("http://a", Some("id1")), 1);
    assert_eq!(hl.put("http://b", Some("id1")), 2);
    assert_eq!(hl.put("http://a", Some("id2")), 3);
    assert_eq!(hl.len(), 3);
}

/// A link with no id is anonymous, and the comparator falls back to the
/// inner number so that two of them never match — even for the same URI.
#[test]
fn anonymous_links_are_never_shared() {
    let _guard = store();
    let hl = Links::new();
    assert_eq!(hl.put("http://a", None), 1);
    assert_eq!(hl.put("http://a", None), 2);
    assert_eq!(hl.put("http://a", Some("")), 3);
    assert_eq!(hl.len(), 3);
    assert_eq!(hl.put("http://a", Some("id")), 4);
    assert_eq!(hl.put("http://a", Some("id")), 4);
}

#[test]
fn a_uri_and_an_id_are_escaped_before_they_are_stored() {
    let _guard = store();
    let hl = Links::new();
    let inner = hl.put("http://a\\b", Some("i\\d"));
    let (uri, internal, _) = hl.get(inner).expect("the link is there");
    assert_eq!(uri, "http://a\\\\b");
    assert_eq!(internal, "i\\\\d");
}

#[test]
fn getting_a_link_answers_only_what_was_asked_for() {
    let _guard = store();
    let hl = Links::new();
    let inner = hl.put("http://a", Some("id1"));
    let (uri, internal, external) = hl.get(inner).expect("the link is there");
    assert_eq!(uri, "http://a");
    assert_eq!(internal, "id1");
    assert!(external.starts_with("tmux"));

    unsafe {
        let (only_uri, _, _) = hyperlinks_get(&*hl.ptr(), inner).expect("the link is stored");
        assert_eq!(only_uri, c"http://a");
        assert_eq!(hyperlinks_get(&*hl.ptr(), inner + 100), None);
    }
}

#[test]
fn external_ids_are_hexadecimal_and_count_up_across_every_set() {
    let _guard = store();
    let first = Links::new();
    let second = Links::new();
    let a = first.put("http://a", Some("id"));
    let b = second.put("http://b", Some("id"));
    let a = first.get(a).expect("there").2;
    let b = second.get(b).expect("there").2;
    let a = i64::from_str_radix(a.strip_prefix("tmux").expect("the prefix"), 16).expect("hex");
    let b = i64::from_str_radix(b.strip_prefix("tmux").expect("the prefix"), 16).expect("hex");
    assert_eq!(b, a + 1);
    assert_eq!(
        unsafe { hyperlinks_next_external_id },
        (b + 1) as ::core::ffi::c_longlong
    );
}

/// Resetting empties the set but leaves the inner counter where it was, so
/// the numbers the grid still holds are never handed out again.
#[test]
fn resetting_a_set_empties_it_and_keeps_counting_inners() {
    let _guard = store();
    let hl = Links::new();
    hl.put("http://a", Some("id1"));
    hl.put("http://b", Some("id2"));
    assert_eq!(hl.len(), 2);
    assert_eq!(global_hyperlinks.queue().len(), 2);

    hl.0.reset();
    assert_eq!(hl.len(), 0);
    assert_eq!(global_hyperlinks.queue().len(), 0);
    assert_eq!(listed(), Vec::<u_int>::new());
    assert_eq!(hl.put("http://a", Some("id1")), 3);
}

#[test]
fn a_cloned_set_lives_until_the_last_handle_is_dropped() {
    let _guard = store();
    let hl = Links::new();
    hl.put("http://a", Some("id1"));
    let weak = hl.0.downgrade();
    let clone = hl.0.clone();
    drop(hl);
    assert!(weak.upgrade().is_some());
    assert_eq!(unsafe { (*clone.as_ptr()).by_inner.len() }, 1);
    drop(clone);
    assert!(weak.upgrade().is_none());
    assert_eq!(global_hyperlinks.queue().len(), 0);
}

#[test]
fn freeing_the_last_reference_takes_the_links_off_the_global_list() {
    let _guard = store();
    {
        let hl = Links::new();
        hl.put("http://a", Some("id1"));
        assert_eq!(global_hyperlinks.queue().len(), 1);
    }
    assert_eq!(global_hyperlinks.queue().len(), 0);
    assert_eq!(listed(), Vec::<u_int>::new());
}

/// The list is global, so the link that goes when the total reaches the
/// limit is the oldest of *any* set, not of the set being added to. The
/// count is put back below the limit on the same step, so it never gets
/// past it and every further link costs the oldest one.
#[test]
fn reaching_the_limit_drops_the_oldest_link_of_any_set() {
    let _guard = store();
    let first = Links::new();
    let second = Links::new();
    let oldest = first.put("http://oldest", Some("first"));
    let next_oldest = second.put("http://next", Some("next"));
    for i in 2..(MAX_HYPERLINKS as u_int - 1) {
        second.put(&format!("http://{i}"), Some(&format!("id{i}")));
    }
    assert_eq!(global_hyperlinks.queue().len(), MAX_HYPERLINKS as usize - 1);
    assert!(first.get(oldest).is_some());

    second.put("http://last", Some("last"));
    assert_eq!(global_hyperlinks.queue().len(), MAX_HYPERLINKS as usize - 1);
    assert_eq!(first.get(oldest), None);
    assert_eq!(first.len(), 0);
    assert!(second.get(next_oldest).is_some());

    second.put("http://after", Some("after"));
    assert_eq!(global_hyperlinks.queue().len(), MAX_HYPERLINKS as usize - 1);
    assert_eq!(second.get(next_oldest), None);
}

#[test]
fn the_keys_order_by_id_then_uri_and_anonymous_links_by_inner() {
    let named = |id: &CStr, uri: &CStr| hyperlinks_uri_key::new(id, uri, 0);
    let anonymous = |inner| hyperlinks_uri_key::new(c"", c"http://x", inner);

    let a = named(c"id1", c"http://a");
    let b = named(c"id1", c"http://b");
    assert!(a < b);
    assert!(b > a);
    assert_eq!(a, a.clone());

    assert!(a < named(c"id2", c"http://a"));

    assert!(a < anonymous(0));
    assert!(anonymous(0) > a);

    assert!(anonymous(3) < anonymous(7));
    assert_eq!(anonymous(3), anonymous(3));
}

#[test]
fn both_trees_stay_sorted_as_links_come_and_go() {
    let _guard = store();
    let hl = Links::new();
    unsafe {
        let mut seed = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let mut live: Vec<(String, u_int)> = Vec::new();
        for round in 0..400 {
            if live.is_empty() || next() % 3 != 0 {
                let id = format!("id{}", next() % 40);
                let inner = hl.put("http://x", Some(&id));
                if !live.iter().any(|(n, _)| *n == id) {
                    live.push((id, inner));
                }
            } else {
                let i = (next() as usize) % live.len();
                let (_, inner) = live.remove(i);
                assert!((*hl.ptr()).by_inner.contains_key(&inner), "round {round}");
                hyperlinks_remove(&mut *hl.ptr(), inner);
            }

            assert_eq!(hl.len(), live.len(), "round {round}");
            let inners: Vec<u_int> = (*hl.ptr()).by_inner.values().map(|hlu| hlu.inner).collect();
            let mut sorted = inners.clone();
            sorted.sort();
            assert_eq!(inners, sorted, "the inner tree is out of order");

            let mut ids: Vec<String> = Vec::new();
            by_uri_in_order(hl.ptr(), &mut ids);
            let mut sorted = ids.clone();
            sorted.sort();
            assert_eq!(ids, sorted, "the uri tree is out of order");
            assert_eq!(ids.len(), live.len());
        }
    }
}
