use hmux2::src::json::{
    NODE_ARRAY, NODE_OBJECT, json_array_first, json_array_next, json_destroy_node, json_find,
    json_get_number, json_parse, json_to_string,
};
use std::ffi::{CStr, CString};
use std::ptr;

#[test]
fn array_owned_members_preserve_order_and_iteration() {
    unsafe {
        let input = CString::new(r#"{"items":[{"x":1},{"y":2},{"z":3}]}"#).unwrap();
        let mut cause = ptr::null_mut();
        let root = json_parse(input.as_ptr(), &mut cause);
        assert!(!root.is_null(), "parse error: {:?}", CStr::from_ptr(cause));
        assert!(cause.is_null());
        let key = CString::new("items").unwrap();
        let array = json_find(root, key.as_ptr());
        assert_eq!((*array).type_0, NODE_ARRAY);

        let first = json_array_first(array);
        assert_eq!((*first).type_0, NODE_OBJECT);
        let second = json_array_next(first);
        assert_eq!((*second).type_0, NODE_OBJECT);
        let last = json_array_next(second);
        assert_eq!((*last).type_0, NODE_OBJECT);
        assert!(json_array_next(last).is_null());

        let x_key = CString::new("x").unwrap();
        let first_value = json_find(first, x_key.as_ptr());
        let mut number = 0;
        assert_eq!(json_get_number(first_value, &mut number), 0);
        assert_eq!(number, 1);
        let z_key = CString::new("z").unwrap();
        let last_value = json_find(last, z_key.as_ptr());
        assert_eq!(json_get_number(last_value, &mut number), 0);
        assert_eq!(number, 3);

        let serialized = json_to_string(root);
        assert_eq!(
            CStr::from_ptr(serialized).to_bytes(),
            br#"{"items":[{"x":1},{"y":2},{"z":3}]}"#
        );
        libc::free(serialized.cast());
        json_destroy_node(root);
    }
}

#[test]
fn empty_array_has_no_members() {
    unsafe {
        let input = CString::new(r#"{"items":[]}"#).unwrap();
        let mut cause = ptr::null_mut();
        let root = json_parse(input.as_ptr(), &mut cause);
        assert!(!root.is_null(), "parse error: {:?}", CStr::from_ptr(cause));
        assert!(cause.is_null());
        let key = CString::new("items").unwrap();
        assert!(json_array_first(json_find(root, key.as_ptr())).is_null());
        json_destroy_node(root);
    }
}

#[test]
fn malformed_array_destroys_already_parsed_members() {
    unsafe {
        let input = CString::new(r#"{"items":[{"x":1},{"y":2},]}"#).unwrap();
        let mut cause = ptr::null_mut();
        let root = json_parse(input.as_ptr(), &mut cause);
        assert!(root.is_null());
        assert!(!cause.is_null());
        libc::free(cause.cast());
    }
}
