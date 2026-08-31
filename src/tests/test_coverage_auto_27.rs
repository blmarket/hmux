//! Coverage for [`crate::grid`] pure helpers.
//!
//! `hyperlinks.rs` keeps OSC 8 links in a per-screen set keyed by inner
//! number and by (internal-id, URI), with a process-wide oldest-first list.
//! The helpers below are deterministic and avoid touching the real server;
//! each test owns its set via [`Links`] and frees it at the end, so the
//! global list is balanced without reaching `MAX_HYPERLINKS`.

use crate::grid::{
    HyperlinksRef, MAX_HYPERLINKS, RB_BLACK, RB_NEGINF, RB_RED, VIS_CSTYLE, VIS_OCTAL, hyperlinks,
    hyperlinks_by_inner_tree, hyperlinks_by_uri_tree, hyperlinks_get, hyperlinks_put,
    hyperlinks_uri_key,
};
use crate::tests::test_fixtures::{globals, seen};
use ::std::ffi::CString;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A hyperlink set freed at the end of the test.
struct Links(HyperlinksRef);

impl Links {
    fn new() -> Links {
        Links(HyperlinksRef::new())
    }
    fn ptr(&self) -> *mut hyperlinks {
        self.0.as_ptr()
    }
    /// The inner number `uri` was given under `id`, or under no id when `None`.
    fn put(&self, uri: &str, id: Option<&str>) -> u32 {
        let uri = CString::new(uri).expect("no NUL");
        let id = id.map(|s| CString::new(s).expect("no NUL"));
        unsafe { hyperlinks_put(&self.0, &uri, id.as_deref()) }
    }
    /// The URI, internal id and external id stored under `inner`.
    fn get(&self, inner: u32) -> Option<(String, String, String)> {
        unsafe {
            let (uri, internal, external) = hyperlinks_get(&*self.ptr(), inner)?;
            Some((
                seen(uri.as_ptr()),
                seen(internal.as_ptr()),
                seen(external.as_ptr()),
            ))
        }
    }
    fn len(&self) -> usize {
        unsafe { (*self.ptr()).by_inner.len() }
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

#[test]
fn hyperlink_constants_match_upstream_values() {
    assert_eq!(RB_BLACK, 0);
    assert_eq!(RB_RED, 1);
    assert_eq!(RB_NEGINF, -1);
    assert_eq!(VIS_OCTAL, 0x1);
    assert_eq!(VIS_CSTYLE, 0x2);
    assert_eq!(MAX_HYPERLINKS, 5000);
    // by construction MAX is large enough that unit tests never hit eviction
    assert!(MAX_HYPERLINKS > 100);
}

#[test]
fn hyperlink_type_aliases_are_btreemaps() {
    // Cheap smoke test that the public type aliases resolve to BTreeMap-like
    // containers – they must be empty when freshly constructed.
    let by_inner = hyperlinks_by_inner_tree::new();
    let by_uri = hyperlinks_by_uri_tree::new();
    assert!(by_inner.is_empty());
    assert!(by_uri.is_empty());
}

// ---------------------------------------------------------------------------
// hyperlinks_uri_key ordering – pure, no globals
// ---------------------------------------------------------------------------

#[test]
fn hyperlinks_uri_key_named_sorts_before_anonymous_and_by_inner() {
    // Named keys sort before Anonymous, and Anonymous sorts by inner number.
    let named_a = hyperlinks_uri_key::Named(
        CString::new("id1").unwrap(),
        CString::new("http://a").unwrap(),
    );
    let named_b = hyperlinks_uri_key::Named(
        CString::new("id1").unwrap(),
        CString::new("http://b").unwrap(),
    );
    let named_other_id = hyperlinks_uri_key::Named(
        CString::new("id2").unwrap(),
        CString::new("http://a").unwrap(),
    );
    let anon3 = hyperlinks_uri_key::Anonymous(3);
    let anon7 = hyperlinks_uri_key::Anonymous(7);

    assert!(named_a < named_b);
    assert!(named_b > named_a);
    assert_eq!(named_a.clone(), named_a);
    assert!(named_a < named_other_id);
    assert!(named_a < anon3);
    assert!(anon3 > named_a);
    assert!(anon3 < anon7);
    assert_eq!(anon3, anon3);
    assert!(anon7 > anon3);
}

// ---------------------------------------------------------------------------
// hyperlinks_init / put / get
// ---------------------------------------------------------------------------

#[test]
fn new_set_is_empty_and_numbers_from_one() {
    let _guard = globals();
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
fn named_link_is_deduplicated_and_anonymous_is_not() {
    let _guard = globals();
    let hl = Links::new();
    let first = hl.put("http://a", Some("id1"));
    assert_eq!(first, 1);
    // same id+uri returns same inner
    assert_eq!(hl.put("http://a", Some("id1")), first);
    assert_eq!(hl.len(), 1);

    // anonymous never deduplicates, even for same URI
    assert_eq!(hl.put("http://a", None), 2);
    assert_eq!(hl.put("http://a", None), 3);
    assert_eq!(hl.put("http://a", Some("")), 4);
    assert_eq!(hl.len(), 4);

    // named with same id+uri still deduplicates
    assert_eq!(hl.put("http://a", Some("id1")), first);
}

#[test]
fn same_id_different_uri_creates_distinct_links() {
    let _guard = globals();
    let hl = Links::new();
    assert_eq!(hl.put("http://a", Some("id1")), 1);
    assert_eq!(hl.put("http://b", Some("id1")), 2);
    assert_eq!(hl.put("http://a", Some("id2")), 3);
    assert_eq!(hl.len(), 3);
}

#[test]
fn get_returns_stored_uris_and_ids_and_handles_missing() {
    let _guard = globals();
    let hl = Links::new();
    let inner = hl.put("http://example", Some("myid"));
    let (uri, internal, external) = hl.get(inner).expect("stored");
    assert_eq!(uri, "http://example");
    assert_eq!(internal, "myid");
    assert!(external.starts_with("tmux"), "external was {external:?}");

    // missing inner returns None; also via raw hyperlinks_get with null outs
    assert_eq!(hl.get(inner + 100), None);
    unsafe {
        let (only_uri, _, _) = hyperlinks_get(&*hl.ptr(), inner).expect("the link is stored");
        assert_eq!(only_uri, c"http://example");
        assert_eq!(hyperlinks_get(&*hl.ptr(), inner + 100), None);
    }
}

#[test]
fn uri_and_id_are_escaped_before_stored() {
    let _guard = globals();
    let hl = Links::new();
    let inner = hl.put("http://a\\b", Some("i\\d"));
    let (uri, internal, _) = hl.get(inner).expect("stored");
    // VIS_OCTAL|VIS_CSTYLE doubles backslashes via utf8_stravis
    assert_eq!(uri, "http://a\\\\b");
    assert_eq!(internal, "i\\\\d");
}

#[test]
fn reset_empties_set_but_keeps_inner_counter() {
    let _guard = globals();
    let hl = Links::new();
    hl.put("http://a", Some("id1"));
    hl.put("http://b", Some("id2"));
    assert_eq!(hl.len(), 2);
    hl.0.reset();
    assert_eq!(hl.len(), 0);
    assert_eq!(hl.get(1), None);
    assert_eq!(hl.get(2), None);
    // next_inner was 3 before reset, so next put gets 3
    assert_eq!(hl.put("http://a", Some("id1")), 3);
}

#[test]
fn cloned_handle_keeps_set_alive_until_last_drop() {
    let _guard = globals();
    let hl = Links::new();
    hl.put("http://a", Some("id1"));
    let weak = hl.0.downgrade();
    let clone = hl.0.clone();
    drop(hl);
    assert!(weak.upgrade().is_some());
    drop(clone);
    assert!(weak.upgrade().is_none());
}
