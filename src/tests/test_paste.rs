use super::*;
use crate::options::options_set_number;
use crate::tests::test_fixtures::globals;
use ::core::ffi::c_char;
use ::core::ptr::{null, null_mut};
use ::std::ffi::CString;
use ::std::sync::MutexGuard;

/// A turn at the store, starting from no buffers at all and with the name
/// and order counters back at zero. The buffers and the counters are this
/// module's own globals, so each test empties them itself.
fn store() -> MutexGuard<'static, ()> {
    let guard = globals();
    unsafe {
        let mut pb = paste_walk(null_mut::<paste_buffer>());
        while !pb.is_null() {
            let next = paste_walk(pb);
            paste_free(pb);
            pb = next;
        }
        paste_next_index = 0;
        paste_next_order = 0;
        paste_num_automatic = 0;
    }
    guard
}

/// A copy of `s` on the heap, the way every caller hands a paste buffer its
/// data.
fn data(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

/// The names of every buffer, newest first.
fn names() -> Vec<String> {
    unsafe {
        let mut out = Vec::new();
        let mut pb = paste_walk(null_mut::<paste_buffer>());
        while !pb.is_null() {
            out.push(paste_buffer_name(&*pb).to_string_lossy().into_owned());
            pb = paste_walk(pb);
        }
        out
    }
}

/// What a buffer holds, read back through the public accessor.
unsafe fn contents(pb: *mut paste_buffer) -> (String, size_t) {
    unsafe {
        let bytes = paste_buffer_data(&*pb);
        (
            String::from_utf8_lossy(bytes).into_owned(),
            bytes.len() as size_t,
        )
    }
}

/// Runs `body` with `buffer-limit` set to `limit`, putting the option back
/// afterwards.
fn with_limit(limit: ::core::ffi::c_longlong, body: impl FnOnce()) {
    unsafe {
        let before = options_get_number(global_options, c"buffer-limit".as_ptr());
        options_set_number(global_options, c"buffer-limit".as_ptr(), limit);
        body();
        options_set_number(global_options, c"buffer-limit".as_ptr(), before);
    }
}

#[test]
fn an_empty_store_answers_nothing() {
    let _guard = store();
    unsafe {
        assert_eq!(paste_is_empty(), 1);
        assert!(paste_walk(null_mut::<paste_buffer>()).is_null());
        assert!(paste_get_top(None).is_null());
        assert!(paste_get_name(null::<c_char>()).is_null());
        assert!(paste_get_name(c"".as_ptr()).is_null());
        assert!(paste_get_name(c"buffer0".as_ptr()).is_null());
    }
}

#[test]
fn an_added_buffer_is_named_after_the_prefix_and_the_running_index() {
    let _guard = store();
    unsafe {
        paste_add(null::<c_char>(), data("one"));
        paste_add(c"my".as_ptr(), data("two"));
        assert_eq!(names(), vec!["my1", "buffer0"]);
        assert_eq!(paste_is_empty(), 0);

        let first = paste_get_name(c"buffer0".as_ptr());
        assert_eq!(contents(first), ("one".to_string(), 3));
        assert_eq!(paste_buffer_order(&*first), 0);
        assert_ne!(paste_buffer_created(&*first), 0);
        assert_eq!((*first).automatic, 1);

        let second = paste_get_name(c"my1".as_ptr());
        assert_eq!(paste_buffer_order(&*second), 1);
        assert_eq!({ paste_num_automatic }, 2);
    }
}

#[test]
fn adding_nothing_adds_no_buffer() {
    let _guard = store();
    unsafe {
        paste_add(null::<c_char>(), Vec::new());
        assert_eq!(paste_is_empty(), 1);
        assert_eq!({ paste_next_index }, 0);
    }
}

#[test]
fn adding_a_buffer_walks_past_a_name_already_taken() {
    let _guard = store();
    unsafe {
        assert!(paste_set(data("held"), c"buffer0".as_ptr()).is_ok());
        paste_add(null::<c_char>(), data("one"));
        assert_eq!(names(), vec!["buffer1", "buffer0"]);
        assert_eq!({ paste_next_index }, 2);
    }
}

#[test]
fn the_newest_buffer_comes_first_when_walking() {
    let _guard = store();
    unsafe {
        paste_add(null::<c_char>(), data("a"));
        paste_add(null::<c_char>(), data("b"));
        paste_add(null::<c_char>(), data("c"));
        assert_eq!(names(), vec!["buffer2", "buffer1", "buffer0"]);
    }
}

#[test]
fn the_top_buffer_is_the_newest_automatic_one() {
    let _guard = store();
    unsafe {
        paste_add(null::<c_char>(), data("auto"));
        assert!(paste_set(data("named"), c"kept".as_ptr()).is_ok());
        assert_eq!(names(), vec!["kept", "buffer0"]);

        let mut name: Option<CString> = None;
        let top = paste_get_top(Some(&mut name));
        assert_eq!(paste_buffer_name(&*top).to_string_lossy(), "buffer0");
        assert_eq!(name.as_ref().unwrap().to_string_lossy(), "buffer0");
        assert_eq!(
            paste_buffer_name(&*paste_get_top(None)).to_string_lossy(),
            "buffer0"
        );

        paste_free(top);
        assert!(paste_get_top(None).is_null());
    }
}

#[test]
fn the_buffer_limit_frees_the_oldest_automatic_buffers() {
    let _guard = store();
    unsafe {
        with_limit(3, || {
            for _ in 0..5 {
                paste_add(null::<c_char>(), data("x"));
            }
            assert_eq!(names(), vec!["buffer4", "buffer3", "buffer2"]);
            assert_eq!({ paste_num_automatic }, 3);
        });
    }
}

#[test]
fn the_buffer_limit_leaves_named_buffers_alone() {
    let _guard = store();
    unsafe {
        with_limit(1, || {
            assert!(paste_set(data("named"), c"kept".as_ptr()).is_ok());
            paste_add(null::<c_char>(), data("a"));
            paste_add(null::<c_char>(), data("b"));
            assert_eq!(names(), vec!["buffer1", "kept"]);
            assert_eq!({ paste_num_automatic }, 1);
        });
    }
}

#[test]
fn freeing_a_buffer_takes_it_out_of_both_trees() {
    let _guard = store();
    unsafe {
        paste_add(null::<c_char>(), data("one"));
        let pb = paste_get_name(c"buffer0".as_ptr());
        paste_free(pb);
        assert_eq!(paste_is_empty(), 1);
        assert!(paste_get_name(c"buffer0".as_ptr()).is_null());
        assert_eq!({ paste_num_automatic }, 0);
    }
}

#[test]
fn renaming_reports_what_is_wrong_with_the_names() {
    let _guard = store();
    unsafe {
        assert_eq!(
            paste_rename(null::<c_char>(), c"new".as_ptr())
                .unwrap_err()
                .to_string_lossy(),
            "no buffer"
        );
        assert_eq!(
            paste_rename(c"".as_ptr(), c"new".as_ptr())
                .unwrap_err()
                .to_string_lossy(),
            "no buffer"
        );
        assert_eq!(
            paste_rename(c"old".as_ptr(), null::<c_char>())
                .unwrap_err()
                .to_string_lossy(),
            "new name is empty"
        );
        assert_eq!(
            paste_rename(c"old".as_ptr(), c"".as_ptr())
                .unwrap_err()
                .to_string_lossy(),
            "new name is empty"
        );
        assert_eq!(
            paste_rename(c"old".as_ptr(), c"\xff".as_ptr())
                .unwrap_err()
                .to_string_lossy(),
            "invalid buffer name: \u{fffd}"
        );
        assert_eq!(
            paste_rename(c"old".as_ptr(), c"new".as_ptr())
                .unwrap_err()
                .to_string_lossy(),
            "no buffer old"
        );
        assert!(paste_rename(null::<c_char>(), c"new".as_ptr()).is_err());
        assert!(paste_rename(c"old".as_ptr(), c"".as_ptr()).is_err());
        assert!(paste_rename(c"old".as_ptr(), c"\xff".as_ptr()).is_err());
        assert!(paste_rename(c"old".as_ptr(), c"new".as_ptr()).is_err());
    }
}

#[test]
fn renaming_a_buffer_makes_it_one_the_user_named() {
    let _guard = store();
    unsafe {
        paste_add(null::<c_char>(), data("one"));
        assert_eq!({ paste_num_automatic }, 1);
        assert!(paste_rename(c"buffer0".as_ptr(), c"mine".as_ptr()).is_ok());
        assert!(paste_get_name(c"buffer0".as_ptr()).is_null());
        let pb = paste_get_name(c"mine".as_ptr());
        assert_eq!(paste_buffer_name(&*pb).to_string_lossy(), "mine");
        assert_eq!(paste_buffer_automatic(&*pb), 0);
        assert_eq!({ paste_num_automatic }, 0);
        assert_eq!(paste_buffer_order(&*pb), 0);
    }
}

#[test]
fn a_new_name_is_cleaned_before_it_is_used() {
    let _guard = store();
    unsafe {
        paste_add(null::<c_char>(), data("one"));
        assert!(paste_rename(c"buffer0".as_ptr(), c"a\\b".as_ptr()).is_ok());
        assert_eq!(names(), vec!["a\\\\b"]);
    }
}

/// Renaming a buffer to the name it already has stops at the "same buffer"
/// check, so it stays automatic — the rest of the rename never runs.
#[test]
fn renaming_a_buffer_to_its_own_name_leaves_it_automatic() {
    let _guard = store();
    unsafe {
        paste_add(null::<c_char>(), data("one"));
        assert!(paste_rename(c"buffer0".as_ptr(), c"buffer0".as_ptr()).is_ok());
        let pb = paste_get_name(c"buffer0".as_ptr());
        assert_eq!(paste_buffer_automatic(&*pb), 1);
        assert_eq!({ paste_num_automatic }, 1);
    }
}

#[test]
fn renaming_over_another_buffer_frees_the_one_it_lands_on() {
    let _guard = store();
    unsafe {
        assert!(paste_set(data("first"), c"one".as_ptr()).is_ok());
        assert!(paste_set(data("second"), c"two".as_ptr()).is_ok());
        assert!(paste_rename(c"two".as_ptr(), c"one".as_ptr()).is_ok());
        assert_eq!(names(), vec!["one"]);
        assert_eq!(
            contents(paste_get_name(c"one".as_ptr())),
            ("second".to_string(), 6)
        );
    }
}

#[test]
fn setting_a_buffer_reports_what_is_wrong_with_the_name() {
    let _guard = store();
    unsafe {
        assert_eq!(
            paste_set(data("x"), c"".as_ptr())
                .unwrap_err()
                .to_string_lossy(),
            "empty buffer name"
        );
        assert_eq!(
            paste_set(data("x"), c"\xff".as_ptr())
                .unwrap_err()
                .to_string_lossy(),
            "invalid buffer name: \u{fffd}"
        );
        assert!(paste_set(data("x"), c"".as_ptr()).is_err());
        assert!(paste_set(data("x"), c"\xff".as_ptr()).is_err());
        assert_eq!(paste_is_empty(), 1);
    }
}

#[test]
fn setting_nothing_or_no_name_falls_back() {
    let _guard = store();
    unsafe {
        assert!(paste_set(Vec::new(), c"name".as_ptr()).is_ok());
        assert_eq!(paste_is_empty(), 1);

        assert!(paste_set(data("x"), null::<c_char>()).is_ok());
        assert_eq!(names(), vec!["buffer0"]);
        assert_eq!({ paste_num_automatic }, 1);
    }
}

#[test]
fn setting_a_buffer_of_a_name_already_there_replaces_it() {
    let _guard = store();
    unsafe {
        assert!(paste_set(data("first"), c"one".as_ptr()).is_ok());
        let before = paste_buffer_order(&*paste_get_name(c"one".as_ptr()));
        assert!(paste_set(data("second"), c"one".as_ptr()).is_ok());
        assert_eq!(names(), vec!["one"]);
        let pb = paste_get_name(c"one".as_ptr());
        assert_eq!(contents(pb), ("second".to_string(), 6));
        assert_eq!(paste_buffer_order(&*pb), before + 1);
        assert_eq!(paste_buffer_automatic(&*pb), 0);
    }
}

#[test]
fn a_set_name_is_cleaned_before_it_is_used() {
    let _guard = store();
    unsafe {
        assert!(paste_set(data("x"), c"a\\b".as_ptr()).is_ok());
        assert_eq!(names(), vec!["a\\\\b"]);
    }
}

#[test]
fn replacing_a_buffer_swaps_its_data_and_keeps_its_name() {
    let _guard = store();
    unsafe {
        paste_add(null::<c_char>(), data("one"));
        let pb = paste_get_name(c"buffer0".as_ptr());
        paste_replace(pb, data("longer"));
        assert_eq!(contents(pb), ("longer".to_string(), 6));
        assert_eq!(paste_buffer_name(&*pb).to_string_lossy(), "buffer0");
    }
}

#[test]
fn a_sample_escapes_what_it_shows() {
    let _guard = store();
    unsafe {
        paste_add(null::<c_char>(), data("a\tb\nc"));
        let pb = paste_get_name(c"buffer0".as_ptr());
        assert_eq!(paste_make_sample(&*pb).to_string_lossy(), "a\\tb\\nc");
    }
}

#[test]
fn a_sample_of_more_than_two_hundred_bytes_ends_in_dots() {
    let _guard = store();
    unsafe {
        let long = "x".repeat(250);
        paste_add(null::<c_char>(), data(&long));
        let pb = paste_get_name(c"buffer0".as_ptr());
        let sample = paste_make_sample(&*pb).to_string_lossy().into_owned();
        assert_eq!(sample.len(), 203);
        assert_eq!(&sample[..200], &long[..200]);
        assert_eq!(&sample[200..], "...");
    }
}

/// A short buffer whose escaped form runs past two hundred characters gets
/// the same trailing dots, written over the middle of what was escaped.
#[test]
fn a_sample_that_grows_past_two_hundred_when_escaped_ends_in_dots_too() {
    let _guard = store();
    unsafe {
        let raw = "\x01".repeat(60);
        paste_add(null::<c_char>(), data(&raw));
        let pb = paste_get_name(c"buffer0".as_ptr());
        let sample = paste_make_sample(&*pb).to_string_lossy().into_owned();
        assert_eq!(sample.len(), 203);
        assert_eq!(&sample[..8], "\\001\\001");
        assert_eq!(&sample[200..], "...");
    }
}

#[test]
fn the_two_trees_stay_sorted_as_buffers_come_and_go() {
    let _guard = store();
    unsafe {
        let mut seed = 0x2545_f491_4f6c_dd1du64;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let mut live: Vec<String> = Vec::new();
        for round in 0..400 {
            if live.is_empty() || next() % 3 != 0 {
                let name = format!("b{}", next() % 500);
                let c = CString::new(name.clone()).expect("no NUL");
                assert!(paste_set(data("x"), c.as_ptr()).is_ok());
                live.retain(|n| *n != name);
                live.push(name);
            } else {
                let i = (next() as usize) % live.len();
                let name = live.remove(i);
                let c = CString::new(name).expect("no NUL");
                paste_free(paste_get_name(c.as_ptr()));
            }

            let mut walked = names();
            assert_eq!(walked.len(), live.len(), "round {round}");
            let mut orders: Vec<u_int> = Vec::new();
            for name in &walked {
                let c = CString::new(name.clone()).expect("no NUL");
                let pb = paste_get_name(c.as_ptr());
                assert!(!pb.is_null(), "{name} is not in the name tree");
                orders.push(paste_buffer_order(&*pb));
            }
            let mut sorted = orders.clone();
            sorted.sort_by(|a, b| b.cmp(a));
            assert_eq!(orders, sorted, "the time tree is out of order");
            walked.sort();
            let mut expected = live.clone();
            expected.sort();
            assert_eq!(walked, expected);
        }
    }
}

#[test]
fn a_buffer_reports_its_data_without_a_size_asked_for() {
    let _guard = store();
    unsafe {
        paste_add(null::<c_char>(), data("one"));
        let pb = paste_get_name(c"buffer0".as_ptr());
        assert_eq!(paste_buffer_data(&*pb), b"one");
    }
}
