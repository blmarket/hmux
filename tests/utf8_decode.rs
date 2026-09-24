use hmux2::src::shared::grid::utf8_data;
use hmux2::src::shared::utf8::{UTF8_DONE, UTF8_ERROR, UTF8_MORE};
use hmux2::src::text::utf8::{utf8_append, utf8_cstrhas, utf8_open, utf8_towc};
use hmux2::src::text::utf8_decode::{decode_utf8, DecodeResult};

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
fn cstrhas_respects_utf8_cells_and_the_first_nul() {
    let query = |bytes: &[u8]| {
        let mut cell = empty_data();
        cell.size = bytes.len() as u8;
        cell.data[..bytes.len()].copy_from_slice(bytes);
        cell
    };
    unsafe {
        assert_eq!(utf8_cstrhas(b"a\0b\0".as_ptr().cast(), &query(b"b")), 0);
        assert_eq!(
            utf8_cstrhas(b"a\xc3\xa9z\0".as_ptr().cast(), &query(&[0xa9])),
            0
        );
        assert_eq!(
            utf8_cstrhas(b"\xe2(\xa1\0".as_ptr().cast(), &query(&[0xe2, b'(', 0xa1])),
            0
        );
        assert_eq!(
            utf8_cstrhas(b"\xe2(\xa1\0".as_ptr().cast(), &query(b"(")),
            1
        );
        assert_eq!(
            utf8_cstrhas(b"\xf0\x9f\x92\0".as_ptr().cast(), &query(&[0x9f])),
            1
        );
        assert_eq!(utf8_cstrhas(b"\xff\0".as_ptr().cast(), &query(&[0xff])), 1);
        assert_eq!(
            utf8_cstrhas(b"a\xe2(\xa1\0".as_ptr().cast(), &query(b"a")),
            1
        );
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
