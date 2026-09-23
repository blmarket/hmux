use hmux2::src::tty_features::{tty_get_features, tty_parse_features};
use std::ffi::CStr;

fn feature_names(bits: i32) -> String {
    unsafe { CStr::from_ptr(tty_get_features(bits)) }
        .to_str()
        .unwrap()
        .to_owned()
}

#[test]
fn feature_list_separators_removal_and_unknown_feature() {
    unsafe {
        let mut enabled = 0;
        let mut disabled = 0;
        let separators = b":,\0".as_ptr().cast();

        tty_parse_features(
            b"RGB:256,256@:sixel\0".as_ptr().cast(),
            separators,
            &mut enabled,
            &mut disabled,
        );
        assert_eq!(feature_names(enabled), "RGB,sixel");
        assert_eq!(feature_names(disabled), "256");

        // A disabled feature cannot be re-added later in the same list.
        tty_parse_features(
            b"256:mouse\0".as_ptr().cast(),
            separators,
            &mut enabled,
            &mut disabled,
        );
        assert_eq!(feature_names(enabled), "mouse,RGB,sixel");

        // An unknown feature stops parsing before the later known one.
        tty_parse_features(
            b"title:missing:clipboard\0".as_ptr().cast(),
            separators,
            &mut enabled,
            &mut disabled,
        );
        assert_eq!(feature_names(enabled), "mouse,RGB,sixel,title");

        // As a C string, bytes after the first NUL are not part of the list.
        tty_parse_features(
            b"osc7\0,clipboard\0".as_ptr().cast(),
            separators,
            &mut enabled,
            &mut disabled,
        );
        assert_eq!(feature_names(enabled), "mouse,osc7,RGB,sixel,title");
    }
}
