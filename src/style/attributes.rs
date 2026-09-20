//! Byte-oriented attribute text APIs. Case matching follows the active C locale.
use crate::src::shared::grid::*;
use std::ffi::{CStr, CString};

const NAMES: &[(&[u8], i32)] = &[
    (b"acs", GRID_ATTR_CHARSET),
    (b"bright", GRID_ATTR_BRIGHT),
    (b"dim", GRID_ATTR_DIM),
    (b"underscore", GRID_ATTR_UNDERSCORE),
    (b"blink", GRID_ATTR_BLINK),
    (b"reverse", GRID_ATTR_REVERSE),
    (b"hidden", GRID_ATTR_HIDDEN),
    (b"italics", GRID_ATTR_ITALICS),
    (b"strikethrough", GRID_ATTR_STRIKETHROUGH),
    (b"double-underscore", GRID_ATTR_UNDERSCORE_2),
    (b"curly-underscore", GRID_ATTR_UNDERSCORE_3),
    (b"dotted-underscore", GRID_ATTR_UNDERSCORE_4),
    (b"dashed-underscore", GRID_ATTR_UNDERSCORE_5),
    (b"overline", GRID_ATTR_OVERLINE),
    (b"noattr", GRID_ATTR_NOATTR),
];

/// Parse an entire byte slice. Embedded NUL and invalid attribute lists return None.
pub fn attributes_parse(input: &[u8]) -> Option<i32> {
    fn delimiter(b: &u8) -> bool {
        b" ,|".contains(b)
    }
    if input.is_empty()
        || input.contains(&0)
        || delimiter(input.first()?)
        || delimiter(input.last()?)
    {
        return None;
    }
    if equal(input, b"none") || equal(input, b"default") {
        return Some(0);
    }
    let mut attr = 0;
    for token in input.split(delimiter).filter(|token| !token.is_empty()) {
        if equal(token, b"bold") {
            attr |= GRID_ATTR_BRIGHT;
            continue;
        }
        let (_, bit) = NAMES
            .iter()
            .find(|(name, bit)| *bit != GRID_ATTR_NOATTR && equal(token, name))?;
        attr |= bit;
    }
    Some(attr)
}

fn equal(a: &[u8], b: &[u8]) -> bool {
    // Both slices have this many readable bytes; strncasecmp reads at most n.
    // Keeping libc here preserves locale-sensitive case matching.
    a.len() == b.len()
        && unsafe { libc::strncasecmp(a.as_ptr().cast(), b.as_ptr().cast(), a.len()) == 0 }
}

pub fn attributes_parse_cstr(input: &CStr) -> Option<i32> {
    attributes_parse(input.to_bytes())
}

/// Owned canonical text; later formatting calls cannot invalidate this result.
pub fn attributes_format(attr: i32) -> CString {
    let mut bytes = Vec::new();
    if attr == 0 {
        bytes.extend_from_slice(b"none");
    } else {
        for &(name, bit) in NAMES {
            if attr & bit != 0 {
                if !bytes.is_empty() {
                    bytes.push(b',');
                }
                bytes.extend_from_slice(name);
            }
        }
    }
    CString::new(bytes).expect("attribute names contain no NUL")
}

/// C ABI only. The pointer is valid until the next call on this thread or thread
/// exit. Do not free it or access it concurrently with another call on that thread.
/// Rust callers should retain the CString returned by attributes_format instead.
#[no_mangle]
pub unsafe extern "C" fn attributes_tostring(attr: i32) -> *const libc::c_char {
    thread_local! {
        static BUFFER: std::cell::RefCell<CString> = std::cell::RefCell::new(CString::default());
    }
    BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        *buffer = attributes_format(attr);
        buffer.as_ptr()
    })
}

/// C ABI only.
/// # Safety
/// Input must point to a readable NUL-terminated string for this call.
#[no_mangle]
pub unsafe extern "C" fn attributes_fromstring(input: *const libc::c_char) -> i32 {
    attributes_parse_cstr(CStr::from_ptr(input)).unwrap_or(-1)
}
