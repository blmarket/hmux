//! Frozen pre-migration sizes, alignments, and every named field offset.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_copies_match() {
    let mut records = Vec::new();
    macro_rules! record {
        ($label:literal, $ty:ty, [$($field:ident),*]) => {
            records.push(format!("{} {} {} {:?}", $label, size_of::<$ty>(), align_of::<$ty>(),
                &[$(offset_of!($ty, $field)),*] as &[usize]));
        };
    }
    record!(
        "src/input.rs::input_request_clipboard_data",
        hmux2::src::input::input_request_clipboard_data,
        [buf, len, clip]
    );
    record!(
        "src/tty_keys.rs::input_request_clipboard_data",
        hmux2::src::tty_keys::input_request_clipboard_data,
        [buf, len, clip]
    );
    record!(
        "src/input.rs::input_request_palette_data",
        hmux2::src::input::input_request_palette_data,
        [idx, c]
    );
    record!(
        "src/tty_keys.rs::input_request_palette_data",
        hmux2::src::tty_keys::input_request_palette_data,
        [idx, c]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/remaining-input.txt"));
}
