use hmux2::src::monitor::{
    monitor_parse, MONITOR_ALL_PANES, MONITOR_ALL_WINDOWS, MONITOR_PANE, MONITOR_SESSION,
    MONITOR_WINDOW,
};
use std::ffi::{CStr, CString};
use std::ptr;

unsafe fn parse(input: &[u8]) -> (i32, Option<Vec<u8>>, u32, i32, Option<Vec<u8>>) {
    let input = CString::new(input).unwrap();
    let mut name = ptr::null_mut();
    let mut type_0 = MONITOR_SESSION;
    let mut id = 0;
    let mut format = ptr::null_mut();

    let status = monitor_parse(input.as_ptr(), &mut name, &mut type_0, &mut id, &mut format);
    let name_bytes = (!name.is_null()).then(|| CStr::from_ptr(name).to_bytes().to_vec());
    let format_bytes = (!format.is_null()).then(|| CStr::from_ptr(format).to_bytes().to_vec());
    if !name.is_null() {
        libc::free(name.cast());
    }
    if !format.is_null() {
        libc::free(format.cast());
    }
    (status, name_bytes, type_0, id, format_bytes)
}

#[test]
fn parses_monitor_targets_without_mutating_input() {
    unsafe {
        let input = CString::new(b"\xffhook:%+12:format\xfe:tail".as_slice()).unwrap();
        let original = input.as_bytes_with_nul().to_vec();
        let mut name = ptr::null_mut();
        let mut type_0 = MONITOR_SESSION;
        let mut id = 0;
        let mut format = ptr::null_mut();

        assert_eq!(
            monitor_parse(input.as_ptr(), &mut name, &mut type_0, &mut id, &mut format,),
            0
        );
        assert_eq!(CStr::from_ptr(name).to_bytes(), b"\xffhook");
        assert_eq!(type_0, MONITOR_PANE);
        assert_eq!(id, 12);
        assert_eq!(CStr::from_ptr(format).to_bytes(), b"format\xfe:tail");
        assert_eq!(input.as_bytes_with_nul(), original.as_slice());

        libc::free(name.cast());
        libc::free(format.cast());
    }
}

#[test]
fn preserves_monitor_target_variants_and_rejects_invalid_input() {
    unsafe {
        for (input, expected_type, expected_id) in [
            (b"hook:%*:fmt".as_slice(), MONITOR_ALL_PANES, -1),
            (b"hook:@*:fmt".as_slice(), MONITOR_ALL_WINDOWS, -1),
            (b"hook::fmt".as_slice(), MONITOR_SESSION, -1),
            (b"hook:@7junk:fmt".as_slice(), MONITOR_WINDOW, 7),
        ] {
            let (status, name, type_0, id, format) = parse(input);
            assert_eq!(status, 0, "{input:?}");
            assert_eq!(name.as_deref(), Some(&b"hook"[..]), "{input:?}");
            assert_eq!(type_0, expected_type, "{input:?}");
            assert_eq!(id, expected_id, "{input:?}");
            assert_eq!(format.as_deref(), Some(&b"fmt"[..]), "{input:?}");
        }

        for input in [b"hook".as_slice(), b"hook:bogus:fmt".as_slice()] {
            let (status, name, _, _, format) = parse(input);
            assert_eq!(status, -1, "{input:?}");
            assert!(name.is_none(), "{input:?}");
            assert!(format.is_none(), "{input:?}");
        }
    }
}
