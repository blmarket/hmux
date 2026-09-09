use super::*;
use ::core::ffi::CStr;

/// What the C library makes of `fmt`, and the length it reports.
macro_rules! libc_format {
    ($fmt:expr $(, $arg:expr)*) => {{
        let mut buf = [0u8; 512];
        let n = unsafe {
            snprintf(
                buf.as_mut_ptr() as *mut c_char,
                buf.len(),
                $fmt.as_ptr()
                $(, $arg)*
            )
        };
        assert!(n >= 0 && (n as usize) < buf.len(), "test buffer too small");
        (buf[..n as usize].to_vec(), n)
    }};
}

/// Asserts the engine agrees with the C library, byte for byte and on the
/// length it would have needed.
macro_rules! same {
    ($fmt:expr $(, $arg:expr)*) => {{
        let (want, want_len) = libc_format!($fmt $(, $arg)*);
        let got = format_bytes($fmt, fmt_args![$($arg),*]);
        assert_eq!(
            String::from_utf8_lossy(&got),
            String::from_utf8_lossy(&want),
            "format {:?}",
            $fmt
        );
        assert_eq!(got.len() as c_int, want_len, "length of {:?}", $fmt);
    }};
}

const SIGNED_INT: &[&CStr] = &[
    c"%d", c"%i", c"%5d", c"%-5d", c"%+d", c"% d", c"%05d", c"%.5d", c"%.0d", c"%06d", c"%3d",
    c"%-3d", c"%+05d", c"%+.3d", c"%-+8d", c"% .4d", c"%1d",
];

const SHORT_INT: &[&CStr] = &[
    c"%hhx", c"%02hhx", c"%hhu", c"%hhd", c"%hx", c"%04hx", c"%hi",
];

const UNSIGNED_INT: &[&CStr] = &[
    c"%u", c"%x", c"%X", c"%o", c"%02x", c"%04x", c"%08x", c"%08X", c"%#x", c"%#o", c"%4u",
    c"%06u", c"%05X", c"%06X", c"%06x", c"%8x", c"%4x", c"%03o", c"%.0u", c"%#010x", c"%-8x",
    c"%.6x", c"%#.4o", c"%-#10x", c"%1u",
];

const LONG_INT: &[&CStr] = &[
    c"%ld", c"% li", c"%12ld", c"%-12ld", c"%.10ld", c"%lld", c"%+lld", c"%08lld",
];

const LONG_UNSIGNED: &[&CStr] = &[
    c"%lu", c"%llu", c"%llx", c"%#llx", c"%llX", c"%zu", c"%016llx", c"%#llo", c"%20zu",
];

const STRINGS: &[&CStr] = &[
    c"%s", c"%1s", c"%10s", c"%-10s", c"%.3s", c"%.0s", c"%.10s", c"%3.2s", c"[%s]",
];

const CHARS: &[&CStr] = &[c"%c", c"%3c", c"%-3c", c"<%c>"];

const FLOATS: &[&CStr] = &[
    c"%f", c"%.2f", c"%.0f", c"%e", c"%E", c"%g", c"%10.3f", c"%-10.3f", c"%+f", c"%08.2f",
    c"%#.0f", c"%.17g", c"%lf", c"%.3lf", c"%a", c"%.4e",
];

#[test]
fn signed_conversions_match_libc() {
    for fmt in SIGNED_INT {
        for v in [0i32, 1, -1, 42, -42, 12345, -12345, i32::MIN, i32::MAX, 7] {
            same!(fmt, v);
        }
    }
}

#[test]
fn narrow_conversions_match_libc() {
    for fmt in SHORT_INT {
        for v in [0i32, 1, -1, 0x1234, 0xff, 0x100, 255, 65535, -128, i32::MAX] {
            same!(fmt, v);
        }
    }
}

#[test]
fn unsigned_conversions_match_libc() {
    for fmt in UNSIGNED_INT {
        for v in [0u32, 1, 8, 9, 10, 255, 256, 4095, 0xdeadbeef, u32::MAX] {
            same!(fmt, v);
        }
    }
}

#[test]
fn long_conversions_match_libc() {
    for fmt in LONG_INT {
        for v in [0i64, 1, -1, 1 << 40, -(1 << 40), i64::MIN, i64::MAX] {
            same!(fmt, v);
        }
    }
    for fmt in LONG_UNSIGNED {
        for v in [0u64, 1, 255, 1 << 40, u32::MAX as u64 + 1, u64::MAX] {
            same!(fmt, v);
        }
    }
}

#[test]
fn size_conversions_match_libc() {
    for v in [0usize, 1, 4096, usize::MAX] {
        same!(c"%zu", v);
        same!(c"%10zu", v);
    }
}

#[test]
fn strings_match_libc() {
    for fmt in STRINGS {
        for s in [
            c"".as_ptr(),
            c"a".as_ptr(),
            c"hello".as_ptr(),
            c"a longer piece of text".as_ptr(),
            core::ptr::null(),
        ] {
            same!(fmt, s);
        }
    }
}

#[test]
fn borrowed_strings_match_libc_with_width_precision_and_missing_values() {
    let owner = CString::new([b'a', 0xfe, b'b', b'c']).unwrap();
    for value in [None, Some(c""), Some(c"hello"), Some(owner.as_c_str())] {
        let pointer = value.map_or(core::ptr::null(), CStr::as_ptr);
        for fmt in STRINGS {
            let (expected, _) = libc_format!(fmt, pointer);
            assert_eq!(format_bytes(fmt, fmt_args![value]), expected);
        }
        for precision in [-1, 0, 1, 5, 6, 9] {
            let (expected, _) = libc_format!(c"[%*.*s]", -12, precision, pointer);
            assert_eq!(
                format_bytes(c"[%*.*s]", fmt_args![-12, precision, value]),
                expected
            );
        }
        let (expected, _) = libc_format!(c"%p", pointer);
        assert_eq!(format_bytes(c"%p", fmt_args![value]), expected);
    }
    assert_eq!(
        format_bytes(c"%.2s", fmt_args![owner.as_c_str()]),
        [b'a', 0xfe]
    );
}

#[test]
fn chars_match_libc() {
    for fmt in CHARS {
        for v in [
            b'a' as c_int,
            b'Z' as c_int,
            b' ' as c_int,
            b'\\' as c_int,
            0x7f,
        ] {
            same!(fmt, v);
        }
    }
}

#[test]
fn pointers_match_libc() {
    let x = 42u32;
    for p in [
        core::ptr::null::<u8>(),
        &x as *const u32 as *const u8,
        std::ptr::dangling::<u8>(),
        usize::MAX as *const u8,
    ] {
        same!(c"%p", p);
        same!(c"[%p]", p);
        same!(c"%20p", p);
        same!(c"%-20p", p);
    }
}

#[test]
#[allow(clippy::approx_constant)]
fn floats_match_libc() {
    for fmt in FLOATS {
        for v in [
            0.0f64,
            1.0,
            -1.0,
            0.5,
            2.5,
            3.14159265358979,
            -0.000123,
            1e300,
            1e-300,
            f64::INFINITY,
            f64::NAN,
        ] {
            same!(fmt, v);
        }
    }
}

#[test]
fn literals_match_libc() {
    same!(c"");
    same!(c"plain text");
    same!(c"%%");
    same!(c"100%% done");
    same!(c"a%%b%%c");
    same!(c"\n\ttabbed\n");
}

#[test]
#[allow(clippy::approx_constant)]
fn star_width_and_precision_match_libc() {
    for w in [0i32, 1, 3, 10, -6] {
        same!(c"%*d", w, 42i32);
        same!(c"%*u", w, 42u32);
        same!(c"%*s", w, c"abc".as_ptr());
        same!(c"%-*s", w, c"abc".as_ptr());
    }
    for p in [-1i32, 0, 1, 3, 20] {
        same!(c"%.*s", p, c"abcdefgh".as_ptr());
        same!(c"%.*f", p.max(0), 3.14159f64);
        same!(c"%.*d", p, 42i32);
    }
    same!(c"%*.*s", 10i32, 3i32, c"abcdefgh".as_ptr());
    for p in 0i32..9 {
        same!(c"%.*s", p, core::ptr::null::<c_char>());
    }
}

#[test]
fn mixed_formats_match_libc() {
    same!(
        c"%s: %u sessions, %d clients",
        c"server".as_ptr(),
        3u32,
        -1i32
    );
    same!(
        c"%s@%p (%zu bytes)",
        c"pane".as_ptr(),
        0x1000 as *const u8,
        64usize
    );
    same!(
        c"%c%c%c %02x:%02x",
        b'a' as c_int,
        b'b' as c_int,
        b'c' as c_int,
        0xffu32,
        0u32
    );
    same!(c"%%%s%%", c"x".as_ptr());
    same!(c"%s%s%s", c"".as_ptr(), c"".as_ptr(), c"joined".as_ptr());
}

/// A fixed-seed generator, so a disagreement is reproducible.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 11
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    fn chance(&mut self, n: u64) -> bool {
        self.below(n) == 0
    }

    fn pick<T: Copy>(&mut self, from: &[T]) -> T {
        from[self.below(from.len() as u64) as usize]
    }
}

/// A random specifier over the flag, width, precision and length grid,
/// returned NUL-terminated with the length modifier it was built with.
fn random_spec(rng: &mut Rng, lengths: &[&str], convs: &[u8]) -> (Vec<u8>, String) {
    let mut spec = vec![b'%'];
    for flag in *b"-+ #0" {
        if rng.chance(4) {
            spec.push(flag);
        }
    }
    if rng.chance(2) {
        spec.extend_from_slice(rng.below(21).to_string().as_bytes());
    }
    if rng.chance(3) {
        spec.push(b'.');
        spec.extend_from_slice(rng.below(13).to_string().as_bytes());
    }
    let len = rng.pick(lengths).to_string();
    spec.extend_from_slice(len.as_bytes());
    spec.push(rng.pick(convs));
    spec.push(0);
    (spec, len)
}

fn as_cstr(spec: &[u8]) -> &CStr {
    CStr::from_bytes_with_nul(spec).unwrap()
}

#[test]
fn fuzzed_integer_specifiers_match_libc() {
    let mut rng = Rng(0x5eed);
    let narrow = [
        0i32,
        1,
        -1,
        7,
        42,
        -42,
        255,
        256,
        0x1234,
        i32::MIN,
        i32::MAX,
    ];
    let wide = [
        0i64,
        1,
        -1,
        1 << 40,
        -(1 << 40),
        i64::MIN,
        i64::MAX,
        123456789,
    ];
    for _ in 0..4000 {
        let (spec, len) = random_spec(&mut rng, &["", "hh", "h", "l", "ll", "z"], b"diouxX");
        let fmt = as_cstr(&spec);
        match len.as_str() {
            "" | "hh" | "h" => {
                let v = rng.pick(&narrow);
                same!(fmt, v);
            }
            "z" => {
                let v = rng.pick(&wide).unsigned_abs() as usize;
                same!(fmt, v);
            }
            _ => {
                let v = rng.pick(&wide);
                same!(fmt, v);
            }
        }
    }
}

#[test]
fn fuzzed_string_and_char_specifiers_match_libc() {
    let mut rng = Rng(0xf00d);
    let strings = [
        c"".as_ptr(),
        c"a".as_ptr(),
        c"pane".as_ptr(),
        c"a longer piece of text".as_ptr(),
        core::ptr::null(),
    ];
    for _ in 0..2000 {
        let (spec, _) = random_spec(&mut rng, &[""], b"s");
        let v = rng.pick(&strings);
        same!(as_cstr(&spec), v);
    }
    for _ in 0..1000 {
        let (spec, _) = random_spec(&mut rng, &[""], b"c");
        let v = rng.below(128) as c_int;
        same!(as_cstr(&spec), v);
    }
}

#[test]
#[allow(clippy::approx_constant)]
fn fuzzed_float_specifiers_match_libc() {
    let mut rng = Rng(0xd00d);
    let values = [
        0.0f64,
        1.0,
        -1.0,
        0.5,
        2.5,
        1.0 / 3.0,
        3.14159265358979,
        -0.000123,
        1e300,
        1e-300,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
    ];
    for _ in 0..2000 {
        let (spec, _) = random_spec(&mut rng, &["", "l"], b"feEgGaA");
        let v = rng.pick(&values);
        same!(as_cstr(&spec), v);
    }
}

#[test]
fn string_reading_stops_at_precision_without_nul() {
    let raw = b"abcdef";
    let got = format_bytes(c"%.3s", fmt_args![raw.as_ptr()]);
    assert_eq!(got, b"abc");
}

#[test]
fn byte_string_borrows_bound_reads_and_stop_at_nul() {
    for bytes in [&b"abc\0ignored"[..], &b"\xfe\xffx"[..], &b""[..]] {
        for precision in [-1, 0, 1, 3, 20] {
            let mut terminated = bytes.to_vec();
            terminated.push(0);
            let (expected, _) = libc_format!(c"[%*.*s]", 8, precision, terminated.as_ptr());
            assert_eq!(
                format_bytes(c"[%*.*s]", fmt_args![8, precision, bytes]),
                expected
            );
        }
    }
    assert_eq!(
        format_bytes(c"%s", fmt_args![b"no terminator"]),
        b"no terminator"
    );
}

#[test]
fn incomplete_conversions_preserve_the_borrowed_suffix() {
    for fmt in [c"prefix %", c"%#", c"%.", c"%.12", c"%hh", c"%ll", c"%L"] {
        assert_eq!(format_bytes(fmt, &[]), fmt.to_bytes());
    }
}

#[test]
fn format_into_truncates_like_snprintf() {
    let mut buf = [0x7f as c_char; 16];
    let n = {
        format_into(
            &mut buf[..8],
            c"%s-%u",
            fmt_args![c"abcdef".as_ptr(), 12u32],
        )
    };
    assert_eq!(n, 9);
    assert_eq!(
        &buf[..8],
        &[
            b'a' as c_char,
            b'b' as c_char,
            b'c' as c_char,
            b'd' as c_char,
            b'e' as c_char,
            b'f' as c_char,
            b'-' as c_char,
            0
        ]
    );
    assert_eq!(buf[8], 0x7f);

    let n = format_into(&mut [], c"%u", fmt_args![1000u32]);
    assert_eq!(n, 4);
}

#[test]
fn format_alloc_returns_an_owned_string() {
    let s = format_alloc(c"%s/%d", fmt_args![c"path".as_ptr(), 7i32]);
    assert_eq!(s.as_bytes().len(), 6);
    assert_eq!(s.as_c_str(), c"path/7");
}

#[test]
fn format_len_measures_without_writing() {
    let n = { format_len(c"%s %s", fmt_args![c"ab".as_ptr(), c"cde".as_ptr()]) };
    assert_eq!(n, 6);
}

/// The length `fmt` expands to, the answer `vsnprintf(NULL, 0, ...)` gives.
pub(crate) fn format_len(fmt: &CStr, args: &[FmtArg]) -> usize {
    format_bytes(fmt, args).len()
}
