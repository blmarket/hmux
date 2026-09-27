use hmux2::src::hyperlinks::*;
use std::ffi::CString;

// A single test keeps the process-global eviction list serialized.
#[test]
fn deduplication_reference_lifetime_reset_and_global_eviction() {
    unsafe {
        let a = hyperlinks_init();
        let b = hyperlinks_init();
        let first = hyperlinks_put(a, c"https://example.com".as_ptr(), c"id".as_ptr());
        assert_eq!(
            first,
            hyperlinks_put(a, c"https://example.com".as_ptr(), c"id".as_ptr())
        );
        assert_ne!(
            first,
            hyperlinks_put(a, c"https://other.example".as_ptr(), c"id".as_ptr())
        );
        let anonymous = hyperlinks_put(a, c"https://example.com".as_ptr(), c"".as_ptr());
        assert_ne!(
            anonymous,
            hyperlinks_put(a, c"https://example.com".as_ptr(), std::ptr::null())
        );
        assert!(hyperlinks_get(&*a, 0).is_none());
        let link = hyperlinks_get(&*a, first).unwrap();
        assert_eq!(link.uri.as_c_str(), c"https://example.com");
        assert_eq!(link.internal_id.as_c_str(), c"id");
        assert_eq!(link.external_id.as_c_str(), c"tmux1");
        // Retain only an address value, never a borrow across table mutations.
        let first_external_address = link.external_id.as_ptr() as usize;
        let shared = hyperlinks_copy(a);
        hyperlinks_free(a);
        let link = hyperlinks_get(&*shared, first).unwrap();
        assert_eq!(link.external_id.as_ptr() as usize, first_external_address);
        assert_eq!(link.external_id.as_c_str(), c"tmux1");
        let mut retained_inner = 0;
        let mut retained_external_address = 0;
        for index in 0..MAX_HYPERLINKS {
            let inserted = hyperlinks_put(b, c"new".as_ptr(), c"".as_ptr());
            if index == 10 {
                retained_inner = inserted;
                retained_external_address =
                    hyperlinks_get(&*b, inserted).unwrap().external_id.as_ptr() as usize;
            }
        }
        assert_eq!(
            hyperlinks_get(&*b, retained_inner)
                .unwrap()
                .external_id
                .as_ptr() as usize,
            retained_external_address
        );
        // Eviction applies across owners, removing both indexes in the victim.
        assert!(hyperlinks_get(&*shared, first).is_none());
        assert!((*shared).by_inner.storage.is_none());
        assert!((*shared).by_uri.storage.is_none());
        hyperlinks_reset(b);
        assert!((*b).by_inner.storage.is_none());
        assert!((*b).by_uri.storage.is_none());
        let again = hyperlinks_put(b, c"again".as_ptr(), c"id".as_ptr());
        assert_eq!(
            hyperlinks_get(&*b, again).unwrap().external_id.as_c_str(),
            c"tmux138D"
        );

        // Both retained fields must survive the caller's input going away.
        let escaped_uri = CString::new(b"https://example.com/a\n\xff".to_vec()).unwrap();
        let escaped_id = CString::new(b"id\n\xff".to_vec()).unwrap();
        let escaped = hyperlinks_put(b, escaped_uri.as_ptr(), escaped_id.as_ptr());
        drop(escaped_uri);
        drop(escaped_id);
        let link = hyperlinks_get(&*b, escaped).unwrap();
        assert_eq!(link.uri.as_bytes(), b"https://example.com/a\n\\377");
        assert_eq!(link.internal_id.as_bytes(), b"id\n\\377");
        assert_eq!(
            hyperlinks_put(
                b,
                CString::new(b"https://example.com/a\n\xff".to_vec())
                    .unwrap()
                    .as_ptr(),
                CString::new(b"id\n\xff".to_vec()).unwrap().as_ptr()
            ),
            escaped
        );

        // Resetting one table removes its interleaved records without
        // disturbing another table's entry in global insertion order.
        let c = hyperlinks_init();
        let d = hyperlinks_init();
        let c_first = hyperlinks_put(c, c"c-first".as_ptr(), c"".as_ptr());
        let d_only = hyperlinks_put(d, c"d-only".as_ptr(), c"".as_ptr());
        let c_second = hyperlinks_put(c, c"c-second".as_ptr(), c"".as_ptr());
        hyperlinks_reset(c);
        assert!(hyperlinks_get(&*c, c_first).is_none());
        assert!(hyperlinks_get(&*c, c_second).is_none());
        assert_eq!(
            hyperlinks_get(&*d, d_only).unwrap().uri.as_c_str(),
            c"d-only"
        );
        hyperlinks_free(c);
        hyperlinks_free(d);
        hyperlinks_free(shared);
        hyperlinks_free(b);
    }
}
