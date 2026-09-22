use hmux2::src::hyperlinks::*;
use std::{ffi::CStr, ptr::null_mut};

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
        assert_eq!(
            hyperlinks_get(a, first, &mut uri, null_mut(), null_mut()),
            1
        );
        assert_eq!(CStr::from_ptr(uri), c"https://example.com");
        let shared = hyperlinks_copy(a);
        hyperlinks_free(a);
        assert_eq!(
            hyperlinks_get(shared, first, &mut uri, null_mut(), null_mut()),
            1
        );
        for _ in 0..MAX_HYPERLINKS {
            hyperlinks_put(b, c"new".as_ptr(), c"".as_ptr());
        }
        // Eviction applies across owners, removing both indexes in the victim.
        assert_eq!(
            hyperlinks_get(shared, first, &mut uri, null_mut(), null_mut()),
            0
        );
        assert!((*shared).by_inner.storage.is_null());
        assert!((*shared).by_uri.storage.is_null());
        hyperlinks_reset(b);
        assert!((*b).by_inner.storage.is_null());
        assert!((*b).by_uri.storage.is_null());
        let again = hyperlinks_put(b, c"again".as_ptr(), c"id".as_ptr());
        assert_eq!(
            hyperlinks_get(b, again, &mut uri, null_mut(), null_mut()),
            1
        );
        hyperlinks_free(shared);
        hyperlinks_free(b);
    }
}
