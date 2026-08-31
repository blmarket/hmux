pub const GRID_ATTR_BRIGHT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const GRID_ATTR_DIM: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const GRID_ATTR_BLINK: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const GRID_ATTR_REVERSE: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const GRID_ATTR_HIDDEN: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const GRID_ATTR_ITALICS: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const GRID_ATTR_CHARSET: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const GRID_ATTR_STRIKETHROUGH: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE_2: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE_3: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE_4: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE_5: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const GRID_ATTR_OVERLINE: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const GRID_ATTR_NOATTR: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;

/// Attribute bits in the order `attributes_tostring` prints them.
const PRINTED: [(::core::ffi::c_int, &str); 15] = [
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
const PARSED: [(&str, ::core::ffi::c_int); 15] = [
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
fn describe(attr: ::core::ffi::c_int) -> String {
    let mut out = String::new();
    for (bit, name) in PRINTED {
        if attr & bit != 0 {
            if !out.is_empty() {
                out.push(',');
            }
            out.push_str(name);
        }
    }
    out
}

/// Parse a list of attribute names separated by spaces, commas or bars into
/// the bits they set, or `None` if any of it is not understood.
fn parse(s: &[u8]) -> Option<::core::ffi::c_int> {
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
pub fn attributes_tostring(attr: ::core::ffi::c_int) -> ::std::ffi::CString {
    if attr == 0 {
        return c"none".to_owned();
    }
    ::std::ffi::CString::new(describe(attr)).expect("an attribute name has no interior NUL")
}

pub fn attributes_fromstring(s: &::core::ffi::CStr) -> ::core::ffi::c_int {
    parse(s.to_bytes()).unwrap_or(-1)
}

#[cfg(test)]
#[path = "../tests/test_attributes.rs"]
mod tests;
