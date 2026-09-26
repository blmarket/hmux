use hmux2::src::json::{json_destroy_node, json_find, json_parse};
use std::ffi::{CStr, CString};

#[test]
fn object_keys_survive_scratch_and_input_destruction() {
    unsafe {
        let input = CString::new(
            b"{\"\xff\":2,\"a\\u0000b\":3,\"nested\":{\"child\":true},\"array\":[]}".as_slice(),
        )
        .unwrap();
        let mut cause: Option<CString> = None;
        let object = json_parse(input.as_ptr(), &mut cause);
        assert!(!object.is_null(), "parse error: {:?}", cause);
        assert!(cause.is_none());
        drop(input);
        for key in [b"\xff".as_slice(), b"a\\u0000b", b"nested", b"array"] {
            let key = CString::new(key).unwrap();
            let node = json_find(object, key.as_ptr());
            assert!(!node.is_null(), "missing {key:?}");
            assert_eq!(
                CStr::from_ptr(
                    ((*node).key)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                ),
                key.as_c_str()
            );
        }
        json_destroy_node(object);
    }
}

#[test]
fn object_key_errors_destroy_partial_objects() {
    for (input, diagnostic) in [
        (r#"{"":1}"#, "invalid key"),
        (r#"{"a":1,"a":2}"#, "duplicate key"),
        (r#"{"a" "b"}"#, "missing colon"),
        (r#"{"a":}"#, "unexpected value"),
        (r#"{"a":false,}"#, "invalid object"),
        (r#"{"a":1 "b":2}"#, "invalid object"),
        (r#"{"a":{ "b":true, }}"#, "invalid object"),
        (r#"{"a":1, false:2}"#, "tokenization error"),
    ] {
        unsafe {
            let input = CString::new(input).unwrap();
            let mut cause: Option<CString> = None;
            let object = json_parse(input.as_ptr(), &mut cause);
            assert!(object.is_null(), "accepted {input:?}");
            assert!(cause.is_some());
            let message = cause.as_ref().unwrap().as_bytes();
            assert!(
                message
                    .windows(diagnostic.len())
                    .any(|w| w == diagnostic.as_bytes()),
                "{input:?}: {message:?}"
            );
        }
    }
}

#[test]
fn token_growth_keeps_late_keys_and_cleans_up_on_error() {
    let fields = (0..350)
        .map(|i| format!("\"key{i}\":{i}"))
        .collect::<Vec<_>>()
        .join(",");
    for (input, valid) in [
        (format!("{{{fields}}}"), true),
        (format!("{{{fields},\"bad\":}}"), false),
    ] {
        unsafe {
            let input = CString::new(input).unwrap();
            let mut cause: Option<CString> = None;
            let object = json_parse(input.as_ptr(), &mut cause);
            if valid {
                assert!(!object.is_null());
                assert!(cause.is_none());
                drop(input);
                for key in ["key0", "key255", "key349"] {
                    let key = CString::new(key).unwrap();
                    assert!(
                        !json_find(object, key.as_ptr()).is_null(),
                        "missing {key:?}"
                    );
                }
                json_destroy_node(object);
            } else {
                assert!(object.is_null());
                assert!(cause.is_some());
                assert!(cause
                    .as_ref()
                    .unwrap()
                    .to_bytes()
                    .starts_with(b"unexpected value"));
            }
        }
    }
}
