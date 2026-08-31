use super::*;
use ::core::ffi::CStr;

fn tostring(attr: ::core::ffi::c_int) -> String {
    attributes_tostring(attr).to_str().unwrap().to_owned()
}

fn fromstring(s: &CStr) -> ::core::ffi::c_int {
    attributes_fromstring(s)
}

const NAMED: [(&str, ::core::ffi::c_int); 15] = [
    ("acs", GRID_ATTR_CHARSET),
    ("bright", GRID_ATTR_BRIGHT),
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
    ("noattr", GRID_ATTR_NOATTR),
];

#[test]
fn tostring_of_no_attributes_is_none() {
    assert_eq!(tostring(0), "none");
}

#[test]
fn tostring_names_each_single_attribute() {
    for (name, bit) in NAMED {
        assert_eq!(tostring(bit), name);
    }
}

#[test]
fn tostring_lists_attributes_in_table_order_without_a_trailing_comma() {
    assert_eq!(tostring(GRID_ATTR_BRIGHT | GRID_ATTR_CHARSET), "acs,bright");
    let all = NAMED.iter().fold(0, |acc, (_, bit)| acc | bit);
    assert_eq!(
        tostring(all),
        "acs,bright,dim,underscore,blink,reverse,hidden,italics,strikethrough,\
         double-underscore,curly-underscore,dotted-underscore,dashed-underscore,\
         overline,noattr"
    );
}

#[test]
fn tostring_of_unknown_bits_only_is_empty() {
    assert_eq!(tostring(0x8000), "");
}

#[test]
fn fromstring_rejects_empty_and_delimiter_edges() {
    assert_eq!(fromstring(c""), -1);
    assert_eq!(fromstring(c","), -1);
    assert_eq!(fromstring(c" bright"), -1);
    assert_eq!(fromstring(c"bright,"), -1);
    assert_eq!(fromstring(c"bright|"), -1);
}

#[test]
fn fromstring_accepts_the_reset_names() {
    assert_eq!(fromstring(c"default"), 0);
    assert_eq!(fromstring(c"none"), 0);
    assert_eq!(fromstring(c"NONE"), 0);
    assert_eq!(fromstring(c"Default"), 0);
}

#[test]
fn fromstring_maps_each_name() {
    for (name, bit) in NAMED {
        if name == "noattr" {
            continue;
        }
        let owned = ::std::ffi::CString::new(name).unwrap();
        assert_eq!(fromstring(&owned), bit, "{name}");
        let upper = ::std::ffi::CString::new(name.to_uppercase()).unwrap();
        assert_eq!(fromstring(&upper), bit, "{name}");
    }
    assert_eq!(fromstring(c"bold"), GRID_ATTR_BRIGHT);
    assert_eq!(fromstring(c"noattr"), -1);
}

#[test]
fn fromstring_combines_names_across_every_delimiter() {
    assert_eq!(fromstring(c"bright,dim"), GRID_ATTR_BRIGHT | GRID_ATTR_DIM);
    assert_eq!(
        fromstring(c"bright dim|italics"),
        GRID_ATTR_BRIGHT | GRID_ATTR_DIM | GRID_ATTR_ITALICS
    );
    assert_eq!(
        fromstring(c"bright,, |dim"),
        GRID_ATTR_BRIGHT | GRID_ATTR_DIM
    );
    assert_eq!(fromstring(c"bright,bright"), GRID_ATTR_BRIGHT);
}

#[test]
fn fromstring_rejects_unknown_names() {
    assert_eq!(fromstring(c"nosuch"), -1);
    assert_eq!(fromstring(c"abc"), -1);
    assert_eq!(fromstring(c"bright,nosuch"), -1);
}
