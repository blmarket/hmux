//! Exercise the public substitution API's owned result.
use hmux2::src::regsub::regsub_cstring;
use std::ffi::CStr;

#[test]
fn substitution_preserves_matches_and_returns_freeable_bytes() {
    for (pattern, replacement, text, expected) in [
        (
            b"(a)(b)\0".as_slice(),
            b"\\2\\1\0".as_slice(),
            b"zabzab\0".as_slice(),
            b"zbazba".as_slice(),
        ),
        (b"^\0", b"<\0", b"abc\0", b"<abc"),
        (b"x\0", b"Y\0", b"abc\0", b"abc"),
        (b"a\0", b"\0", b"banana\0", b"bnn"),
        (b"a\0", b"b\0", b"\0", b""),
    ] {
        let result = regsub_cstring(
            CStr::from_bytes_with_nul(pattern).unwrap(),
            CStr::from_bytes_with_nul(replacement).unwrap(),
            CStr::from_bytes_with_nul(text).unwrap(),
            libc::REG_EXTENDED,
        )
        .unwrap();
        assert_eq!(result.as_bytes(), expected);
    }

    let invalid = regsub_cstring(c"[", c"x", c"abc", libc::REG_EXTENDED);
    assert!(invalid.is_none());
}
