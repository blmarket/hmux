//! Exercise modifier parsing and replacement through public format expansion.
use hmux2::src::format::{format_create, format_expand_cstring, format_free};
use std::ffi::{CStr, CString};

#[test]
fn modifier_key_survives_nested_expansion_and_failure_cleanup() {
    unsafe {
        let tree = format_create(None, None, 0, 0);
        for (expression, expected) in [
            (b"#{l:literal}\0".as_slice(), b"literal".as_slice()),
            (b"#{n;l:literal}\0", b"7"),
            (b"#{s|#{l:a}|b|;l:a}\0", b"b"),
            (b"#{s|a|b|;s|b|c|;l:a}\0", b"c"),
            (b"#{s|a||;l:a}\0", b""),
            (b"#{s|\xff|done|;l:\xff}\0", b"done"),
            (b"#{l:\xff}\0", b"\xff"),
            (b"before#{R:bad}after\0", b"before"),
        ] {
            let expression = CStr::from_bytes_with_nul(expression).unwrap();
            let result = format_expand_cstring(tree, expression.as_ptr());
            assert_eq!(result.as_bytes(), expected, "{expression:?}");
        }

        // Force the modifier Vec to grow while parsing, then use saved
        // substitution pointers after parsing has finished.
        let expression =
            CString::new(format!("#{{{}s|a|b|;s|b|c|;l:a}}", "l;".repeat(40))).unwrap();
        let result = format_expand_cstring(tree, expression.as_ptr());
        assert_eq!(result.as_bytes(), b"c");

        format_free(Box::from_raw(tree));
    }
}
