use super::*;
use ::core::ffi::{CStr, c_char, c_int};

fn step(cp: &mut c_char, c: u8, state: &mut c_int, flag: c_int) -> c_int {
    unsafe { unvis(&raw mut *cp, c as c_char, &raw mut *state, flag) }
}

/// Feed every byte of `input` through `unvis`, collecting the characters it
/// accepts. Returns `None` as soon as a byte is rejected.
fn decode(input: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut state = S_GROUND;
    let mut cp: c_char = 0;
    for &b in input {
        loop {
            match step(&mut cp, b, &mut state, 0) {
                UNVIS_VALID => {
                    out.push(cp as u8);
                    break;
                }
                UNVIS_VALIDPUSH => out.push(cp as u8),
                UNVIS_NOCHAR | 0 => break,
                _ => return None,
            }
        }
    }
    if step(&mut cp, 0, &mut state, UNVIS_END) == UNVIS_VALID {
        out.push(cp as u8);
    }
    Some(out)
}

fn strunvis_call(src: &CStr) -> Option<Vec<u8>> {
    strunvis(src).map(|out| out.into_bytes())
}

fn strnunvis_call(src: &CStr, sz: usize) -> (ssize_t, Vec<u8>) {
    let mut buf = vec![0xaau8 as c_char; sz + 16];
    let dst = unsafe { buf.as_mut_ptr().add(8) };
    let n = unsafe { strnunvis(dst, src.as_ptr(), sz as size_t) };
    let written = buf[8..8 + sz].iter().map(|&b| b as u8).collect();
    (n, written)
}

#[test]
fn ground_state_passes_plain_characters_through() {
    assert_eq!(decode(b"abc").unwrap(), b"abc");
    assert_eq!(decode(b"").unwrap(), b"");
}

#[test]
fn a_backslash_escapes_the_next_character() {
    assert_eq!(decode(b"a\\\\b").unwrap(), b"a\\b");
}

#[test]
fn the_named_escapes_decode_to_their_control_characters() {
    for (name, want) in [
        (b"\\n".as_slice(), b"\n".as_slice()),
        (b"\\r", b"\r"),
        (b"\\b", b"\x08"),
        (b"\\a", b"\x07"),
        (b"\\v", b"\x0b"),
        (b"\\t", b"\t"),
        (b"\\f", b"\x0c"),
        (b"\\s", b" "),
        (b"\\E", b"\x1b"),
    ] {
        assert_eq!(decode(name).unwrap(), want, "{name:?}");
    }
}

#[test]
fn a_hidden_newline_and_a_dollar_produce_nothing() {
    assert_eq!(decode(b"a\\\nb").unwrap(), b"ab");
    assert_eq!(decode(b"a\\$b").unwrap(), b"ab");
}

#[test]
fn an_unknown_escape_is_rejected() {
    assert_eq!(decode(b"\\z"), None);
}

#[test]
fn octal_escapes_decode_at_one_two_and_three_digits() {
    assert_eq!(decode(b"\\101").unwrap(), b"A");
    assert_eq!(decode(b"\\101B").unwrap(), b"AB");
    assert_eq!(decode(b"\\12x").unwrap(), b"\nx");
    assert_eq!(decode(b"\\1").unwrap(), b"\x01");
    assert_eq!(decode(b"\\12").unwrap(), b"\n");
    assert_eq!(decode(b"\\1x").unwrap(), b"\x01x");
}

#[test]
fn meta_sets_the_high_bit() {
    assert_eq!(decode(b"\\M-A").unwrap(), b"\xc1");
    assert_eq!(decode(b"\\M^A").unwrap(), b"\x81");
    assert_eq!(decode(b"\\M-A"), Some(vec![0o301]));
}

#[test]
fn a_bad_meta_qualifier_is_rejected() {
    assert_eq!(decode(b"\\Mx"), None);
}

#[test]
fn caret_makes_a_control_character() {
    assert_eq!(decode(b"\\^A").unwrap(), b"\x01");
    assert_eq!(decode(b"\\^?").unwrap(), b"\x7f");
}

#[test]
fn the_end_flag_reports_the_state_it_stops_in() {
    let mut cp: c_char = 0;
    let mut state = S_GROUND;
    assert_eq!(step(&mut cp, 0, &mut state, UNVIS_END), UNVIS_NOCHAR);

    let mut state = S_START;
    assert_eq!(step(&mut cp, 0, &mut state, UNVIS_END), UNVIS_SYNBAD);

    let mut state = S_OCTAL2;
    assert_eq!(step(&mut cp, 0, &mut state, UNVIS_END), UNVIS_VALID);
    assert_eq!(state, S_GROUND);

    let mut state = S_OCTAL3;
    assert_eq!(step(&mut cp, 0, &mut state, UNVIS_END), UNVIS_VALID);
    assert_eq!(state, S_GROUND);
}

#[test]
fn an_unknown_state_resets_to_ground_and_fails() {
    let mut cp: c_char = 0;
    let mut state: c_int = 42;
    assert_eq!(step(&mut cp, b'a', &mut state, 0), UNVIS_SYNBAD);
    assert_eq!(state, S_GROUND);

    let mut state: c_int = 42;
    assert_eq!(step(&mut cp, 0, &mut state, UNVIS_END), UNVIS_SYNBAD);
    assert_eq!(state, 42);
}

#[test]
fn strunvis_decodes_the_whole_string() {
    assert_eq!(strunvis_call(c"abc"), Some(b"abc".to_vec()));
    assert_eq!(strunvis_call(c""), Some(b"".to_vec()));
    assert_eq!(strunvis_call(c"a\\nb"), Some(b"a\nb".to_vec()));
    assert_eq!(strunvis_call(c"\\101\\102"), Some(b"AB".to_vec()));
    assert_eq!(strunvis_call(c"\\1x"), Some(b"\x01x".to_vec()));
    assert_eq!(strunvis_call(c"a\\$b"), Some(b"ab".to_vec()));
}

#[test]
fn strunvis_flushes_a_pending_octal_escape_at_the_end() {
    assert_eq!(strunvis_call(c"\\1"), Some(b"\x01".to_vec()));
    assert_eq!(strunvis_call(c"a\\12"), Some(b"a\n".to_vec()));
}

#[test]
fn strunvis_reports_a_syntax_error() {
    assert_eq!(strunvis_call(c"a\\z"), None);
}

#[test]
fn strunvis_stops_at_a_decoded_nul() {
    assert_eq!(strunvis_call(c"a\\0b"), Some(b"a".to_vec()));
    assert_eq!(strunvis_call(c"\\000x"), Some(b"".to_vec()));
}

#[test]
fn strnunvis_decodes_within_the_buffer() {
    assert_eq!(
        strnunvis_call(c"abc", 8),
        (3, b"abc\0\xaa\xaa\xaa\0".to_vec())
    );
    assert_eq!(
        strnunvis_call(c"\\1x", 8),
        (2, b"\x01x\0\xaa\xaa\xaa\xaa\0".to_vec())
    );
}

#[test]
fn strnunvis_truncates_but_returns_the_full_length() {
    assert_eq!(strnunvis_call(c"abcdef", 4), (6, b"abc\0".to_vec()));
    assert_eq!(strnunvis_call(c"\\101\\102\\103", 3), (3, b"AB\0".to_vec()));
    assert_eq!(strnunvis_call(c"ab\\1x", 3), (4, b"ab\0".to_vec()));
}

#[test]
fn strnunvis_flushes_a_pending_octal_escape_at_the_end() {
    assert_eq!(
        strnunvis_call(c"ab\\1", 8),
        (3, b"ab\x01\0\xaa\xaa\xaa\0".to_vec())
    );
    assert_eq!(strnunvis_call(c"ab\\1", 3), (3, b"ab\0".to_vec()));
}

#[test]
fn strnunvis_writes_nothing_when_there_is_no_room() {
    assert_eq!(strnunvis_call(c"abc", 0), (3, Vec::new()));
}

#[test]
fn strnunvis_reports_a_syntax_error() {
    assert_eq!(
        strnunvis_call(c"a\\z", 8),
        (-1, b"a\0\xaa\xaa\xaa\xaa\xaa\0".to_vec())
    );
    assert_eq!(strnunvis_call(c"a\\z", 1), (-1, b"\0".to_vec()));
}
