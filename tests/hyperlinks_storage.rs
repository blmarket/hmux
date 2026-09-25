use hmux2::src::hyperlinks::*;
use std::{
    ffi::{CStr, CString},
    ptr::null_mut,
};

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
        let mut uri = std::ptr::null();
        let mut external_id = std::ptr::null();
        assert_eq!(
            hyperlinks_get(a, first, &mut uri, null_mut(), &mut external_id),
            1
        );
        assert_eq!(CStr::from_ptr(uri), c"https://example.com");
        assert_eq!(CStr::from_ptr(external_id), c"tmux1");
        let first_external_id = external_id;
        let shared = hyperlinks_copy(a);
        hyperlinks_free(a);
        assert_eq!(
            hyperlinks_get(shared, first, &mut uri, null_mut(), &mut external_id),
            1
        );
        assert_eq!(external_id, first_external_id);
        assert_eq!(CStr::from_ptr(external_id), c"tmux1");
        let mut retained_inner = 0;
        let mut retained_external_id = std::ptr::null();
        for index in 0..MAX_HYPERLINKS {
            let inserted = hyperlinks_put(b, c"new".as_ptr(), c"".as_ptr());
            if index == 10 {
                retained_inner = inserted;
                assert_eq!(
                    hyperlinks_get(
                        b,
                        retained_inner,
                        &mut uri,
                        null_mut(),
                        &mut retained_external_id,
                    ),
                    1
                );
            }
        }
        let mut external_id_after_growth = std::ptr::null();
        assert_eq!(
            hyperlinks_get(
                b,
                retained_inner,
                &mut uri,
                null_mut(),
                &mut external_id_after_growth,
            ),
            1
        );
        assert_eq!(external_id_after_growth, retained_external_id);
        // Eviction applies across owners, removing both indexes in the victim.
        assert_eq!(
            hyperlinks_get(shared, first, &mut uri, null_mut(), null_mut()),
            0
        );
        assert!((*shared).by_inner.storage.is_none());
        assert!((*shared).by_uri.storage.is_none());
        hyperlinks_reset(b);
        assert!((*b).by_inner.storage.is_none());
        assert!((*b).by_uri.storage.is_none());
        let again = hyperlinks_put(b, c"again".as_ptr(), c"id".as_ptr());
        assert_eq!(
            hyperlinks_get(b, again, &mut uri, null_mut(), &mut external_id),
            1
        );
        assert_eq!(CStr::from_ptr(external_id), c"tmux138D");

        // Both retained fields must survive the caller's input going away.
        let escaped_uri = CString::new(b"https://example.com/a\n\xff".to_vec()).unwrap();
        let escaped_id = CString::new(b"id\n\xff".to_vec()).unwrap();
        let escaped = hyperlinks_put(b, escaped_uri.as_ptr(), escaped_id.as_ptr());
        drop(escaped_uri);
        drop(escaped_id);
        let mut internal_id = std::ptr::null();
        assert_eq!(
            hyperlinks_get(b, escaped, &mut uri, &mut internal_id, null_mut()),
            1
        );
        assert_eq!(
            CStr::from_ptr(uri).to_bytes(),
            b"https://example.com/a\n\\377"
        );
        assert_eq!(CStr::from_ptr(internal_id).to_bytes(), b"id\n\\377");
        assert_eq!(
            hyperlinks_put(
                b,
                CString::new(b"https://example.com/a\n\xff".to_vec())
                    .unwrap()
                    .as_ptr(),
                CString::new(b"id\n\xff".to_vec()).unwrap().as_ptr(),
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
        assert_eq!(
            hyperlinks_get(c, c_first, &mut uri, null_mut(), null_mut()),
            0
        );
        assert_eq!(
            hyperlinks_get(c, c_second, &mut uri, null_mut(), null_mut()),
            0
        );
        assert_eq!(
            hyperlinks_get(d, d_only, &mut uri, null_mut(), null_mut()),
            1
        );
        assert_eq!(CStr::from_ptr(uri), c"d-only");
        hyperlinks_free(c);
        hyperlinks_free(d);

        hyperlinks_free(shared);
        hyperlinks_free(b);
    }
}
