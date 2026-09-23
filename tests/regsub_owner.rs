//! Exercise the public substitution result and its libc-owned return boundary.
use hmux2::src::regsub::regsub;
use std::ffi::CStr;

#[test]
fn substitution_preserves_matches_and_returns_freeable_bytes() {
    unsafe {
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
            let result = regsub(
                pattern.as_ptr().cast(),
                replacement.as_ptr().cast(),
                text.as_ptr().cast(),
                libc::REG_EXTENDED,
            );
            assert!(!result.is_null());
            assert_eq!(CStr::from_ptr(result).to_bytes(), expected);
            libc::free(result.cast());
        }

        let invalid = regsub(
            b"[\0".as_ptr().cast(),
            b"x\0".as_ptr().cast(),
            b"abc\0".as_ptr().cast(),
            libc::REG_EXTENDED,
        );
        assert!(invalid.is_null());
    }
}
