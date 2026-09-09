pub use crate::consts::{
    GRID_ATTR_BLINK, GRID_ATTR_BRIGHT, GRID_ATTR_CHARSET, GRID_ATTR_DIM, GRID_ATTR_HIDDEN,
    GRID_ATTR_ITALICS, GRID_ATTR_NOATTR, GRID_ATTR_OVERLINE, GRID_ATTR_REVERSE,
    GRID_ATTR_STRIKETHROUGH, GRID_ATTR_UNDERSCORE, GRID_ATTR_UNDERSCORE_2, GRID_ATTR_UNDERSCORE_3,
    GRID_ATTR_UNDERSCORE_4, GRID_ATTR_UNDERSCORE_5,
};

/// Attribute bits in the order `attributes_tostring` prints them.
const PRINTED: [(core::ffi::c_int, &str); 15] = [
    (GRID_ATTR_CHARSET, "acs"),
    (GRID_ATTR_BRIGHT, "bright"),
    (GRID_ATTR_DIM, "dim"),
    (GRID_ATTR_UNDERSCORE, "underscore"),
    (GRID_ATTR_BLINK, "blink"),
    (GRID_ATTR_REVERSE, "reverse"),
    (GRID_ATTR_HIDDEN, "hidden"),
    (GRID_ATTR_ITALICS, "italics"),
    (GRID_ATTR_STRIKETHROUGH, "strikethrough"),
    (GRID_ATTR_UNDERSCORE_2, "double-underscore"),
    (GRID_ATTR_UNDERSCORE_3, "curly-underscore"),
    (GRID_ATTR_UNDERSCORE_4, "dotted-underscore"),
    (GRID_ATTR_UNDERSCORE_5, "dashed-underscore"),
    (GRID_ATTR_OVERLINE, "overline"),
    (GRID_ATTR_NOATTR, "noattr"),
];

/// Names `attributes_fromstring` accepts, and the bit each one sets.
const PARSED: [(&str, core::ffi::c_int); 15] = [
    ("acs", GRID_ATTR_CHARSET),
    ("bright", GRID_ATTR_BRIGHT),
    ("bold", GRID_ATTR_BRIGHT),
    ("dim", GRID_ATTR_DIM),
    ("underscore", GRID_ATTR_UNDERSCORE),
    ("blink", GRID_ATTR_BLINK),
    ("reverse", GRID_ATTR_REVERSE),
    ("hidden", GRID_ATTR_HIDDEN),
    ("italics", GRID_ATTR_ITALICS),
    ("strikethrough", GRID_ATTR_STRIKETHROUGH),
    ("double-underscore", GRID_ATTR_UNDERSCORE_2),
    ("curly-underscore", GRID_ATTR_UNDERSCORE_3),
    ("dotted-underscore", GRID_ATTR_UNDERSCORE_4),
    ("dashed-underscore", GRID_ATTR_UNDERSCORE_5),
    ("overline", GRID_ATTR_OVERLINE),
];

const DELIMITERS: [u8; 3] = *b" ,|";

fn is_delimiter(b: u8) -> bool {
    DELIMITERS.contains(&b)
}

/// The comma-separated names of the attribute bits set in `attr`; empty when
/// only unknown bits are set.
fn describe(attr: core::ffi::c_int) -> std::ffi::CString {
    let mut out = Vec::new();
    for (bit, name) in PRINTED {
        if attr & bit != 0 {
            if !out.is_empty() {
                out.push(b',');
            }
            out.extend_from_slice(name.as_bytes());
        }
    }
    std::ffi::CString::new(out).expect("attribute names have no interior NUL")
}

/// Parse a list of attribute names separated by spaces, commas or bars into
/// the bits they set, or `None` if any of it is not understood.
fn parse(s: &[u8]) -> Option<core::ffi::c_int> {
    if s.is_empty() || is_delimiter(s[0]) || is_delimiter(s[s.len() - 1]) {
        return None;
    }
    if s.eq_ignore_ascii_case(b"default") || s.eq_ignore_ascii_case(b"none") {
        return Some(0);
    }
    let mut attr = 0;
    for token in s.split(|&b| is_delimiter(b)).filter(|t| !t.is_empty()) {
        let (_, bit) = PARSED
            .iter()
            .find(|(name, _)| token.eq_ignore_ascii_case(name.as_bytes()))?;
        attr |= bit;
    }
    Some(attr)
}

/// The attributes a cell carries, named as the caller's own string.
fn attributes_tostring(attr: core::ffi::c_int) -> std::ffi::CString {
    if attr == 0 {
        return c"none".to_owned();
    }
    describe(attr)
}

fn attributes_fromstring(s: &core::ffi::CStr) -> core::ffi::c_int {
    parse(s.to_bytes()).unwrap_or(-1)
}

impl AttributeCodec for RustAttributeCodec {
    fn to_string(&self, attributes: c_int) -> CString {
        attributes_tostring(attributes)
    }

    fn from_string(&self, attributes: &CStr) -> c_int {
        attributes_fromstring(attributes)
    }
}

#[cfg(test)]
#[path = "../tests/test_attributes.rs"]
mod tests;
use core::ffi::{CStr, c_int};
use std::ffi::CString;

/// A hermetic interface to tmux's style-attribute name codec.
pub trait AttributeCodec {
    /// Return tmux's canonical comma-separated representation of `attributes`.
    fn to_string(&self, attributes: c_int) -> CString;

    /// Parse tmux style-attribute names, returning `-1` for invalid input.
    #[allow(clippy::wrong_self_convention)]
    fn from_string(&self, attributes: &CStr) -> c_int;
}

/// The Rust implementation of tmux's style-attribute name codec.
#[derive(Clone, Copy, Debug, Default)]
pub struct RustAttributeCodec;
