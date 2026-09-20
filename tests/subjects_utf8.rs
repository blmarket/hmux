//! Frozen measurements of every original copy; no fixture update mode.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_declarations_match() {
    let mut records = Vec::new();
    let v: ::core::ffi::c_int = hmux2::src::tty::UTF8_SIZE;
    records.push(format!(
        "src/tty.rs::UTF8_SIZE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::utf8::UTF8_SIZE;
    records.push(format!(
        "src/utf8.rs::UTF8_SIZE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    records.push(format!(
        "src/cmd_parse.rs::wchar_t {} {}",
        size_of::<hmux2::src::cmd_parse::wchar_t>(),
        align_of::<hmux2::src::cmd_parse::wchar_t>()
    ));
    records.push(format!(
        "src/compat/utf8proc.rs::wchar_t {} {}",
        size_of::<hmux2::src::compat::utf8proc::wchar_t>(),
        align_of::<hmux2::src::compat::utf8proc::wchar_t>()
    ));
    records.push(format!(
        "src/input_keys.rs::wchar_t {} {}",
        size_of::<hmux2::src::input_keys::wchar_t>(),
        align_of::<hmux2::src::input_keys::wchar_t>()
    ));
    records.push(format!(
        "src/key_string.rs::wchar_t {} {}",
        size_of::<hmux2::src::key_string::wchar_t>(),
        align_of::<hmux2::src::key_string::wchar_t>()
    ));
    records.push(format!(
        "src/tty_keys.rs::wchar_t {} {}",
        size_of::<hmux2::src::tty_keys::wchar_t>(),
        align_of::<hmux2::src::tty_keys::wchar_t>()
    ));
    records.push(format!(
        "src/utf8.rs::wchar_t {} {}",
        size_of::<hmux2::src::utf8::wchar_t>(),
        align_of::<hmux2::src::utf8::wchar_t>()
    ));
    records.push(format!(
        "src/utf8_combined.rs::wchar_t {} {}",
        size_of::<hmux2::src::utf8_combined::wchar_t>(),
        align_of::<hmux2::src::utf8_combined::wchar_t>()
    ));
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/subjects-utf8.txt"));
}
