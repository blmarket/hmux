use hmux2::src::screen_write::screen_write_strlen;
use std::ffi::CString;

const STRING_FORMAT: &[u8] = b"%s\0";

fn display_width(bytes: &[u8]) -> usize {
    let input = CString::new(bytes).unwrap();
    unsafe { screen_write_strlen(STRING_FORMAT.as_ptr().cast(), input.as_ptr()) }
}

#[test]
fn scans_display_bytes_without_requiring_utf8() {
    // Tab counts as one cell; newline, control bytes, DEL, and lone high
    // bytes do not. The high byte must remain a byte, not a replacement char.
    assert_eq!(display_width(b"ab\t\n\x01\x7f\xffc"), 4);

    // The legacy scanner consumes a complete invalid UTF-8 candidate as a
    // group, including the ASCII byte inside it, then resumes scanning.
    assert_eq!(display_width(b"a\xe2(\xa1z"), 2);
    // An incomplete candidate at the terminator ends the scan.
    assert_eq!(display_width(b"ab\xe2\x82"), 2);
}

#[test]
fn formatted_input_stops_at_first_nul() {
    let input = b"ab\0hidden\0";
    let width = unsafe {
        screen_write_strlen(
            STRING_FORMAT.as_ptr().cast(),
            input.as_ptr().cast::<std::ffi::c_char>(),
        )
    };
    assert_eq!(width, 2);
}
