use hmux2::src::shared::grid::utf8_data;
use hmux2::src::shared::utf8::{UTF8_DONE, UTF8_ERROR, UTF8_MORE};
use hmux2::src::utf8::{utf8_append, utf8_fromcstr, utf8_open, utf8_towc};
use hmux2::src::utf8_decode::{decode_utf8, DecodeResult};
use std::ffi::CString;

fn empty_data() -> utf8_data {
    utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    }
}

#[test]
fn legacy_stream_adapter_consumes_an_invalid_candidate_before_recovery() {
    let mut data = empty_data();
    unsafe {
        assert_eq!(utf8_open(&mut data, 0xe2), UTF8_MORE);
        assert_eq!(data.size, 3);
        assert_eq!(data.have, 1);

        // The old byte-at-a-time API keeps collecting the declared sequence
        // even after a bad continuation, so callers can rewind `have` bytes.
        assert_eq!(utf8_append(&mut data, b'('), UTF8_MORE);
        assert_eq!(data.have, 2);
        assert_eq!(utf8_append(&mut data, 0xa1), UTF8_ERROR);
        assert_eq!(data.have, 3);
        assert_eq!(&data.data[..3], &[0xe2, b'(', 0xa1]);
    }
}

#[test]
fn c_string_recovery_still_retries_each_byte_after_a_bad_sequence() {
    let input = CString::new(vec![0xe2, b'(', 0xa1]).unwrap();
    let decoded = unsafe { utf8_fromcstr(input.as_ptr()) };
    assert!(!decoded.is_null());

    unsafe {
        let cells = std::slice::from_raw_parts(decoded, 4);
        assert_eq!(cells[0].size, 1);
        assert_eq!(cells[0].data[0], 0xe2);
        assert_eq!(cells[1].size, 1);
        assert_eq!(cells[1].data[0], b'(');
        assert_eq!(cells[2].size, 1);
        assert_eq!(cells[2].data[0], 0xa1);
        assert_eq!(cells[3].size, 0);
        libc::free(decoded.cast());
    }
}

#[test]
fn legacy_codepoint_conversion_accepts_an_embedded_nul_cell() {
    let mut data = empty_data();
    data.data[0] = 0;
    data.have = 1;
    data.size = 1;
    let mut codepoint = -1;

    unsafe {
        assert_eq!(utf8_towc(&data, &mut codepoint), UTF8_DONE);
    }
    assert_eq!(codepoint, 0);
}

#[test]
fn legacy_codepoint_conversion_keeps_combined_cell_first_codepoint_behavior() {
    let mut data = empty_data();
    data.data[..2].copy_from_slice(b"ab");
    data.have = 2;
    data.size = 2;
    let mut codepoint = -1;

    unsafe {
        assert_eq!(utf8_towc(&data, &mut codepoint), UTF8_DONE);
    }
    assert_eq!(codepoint, b'a' as i32);
}

#[test]
fn helper_keeps_trailing_bytes_for_the_next_decode() {
    assert_eq!(
        decode_utf8("λx".as_bytes()),
        DecodeResult::Complete {
            codepoint: 0x03bb,
            len: 2,
        }
    );
}
