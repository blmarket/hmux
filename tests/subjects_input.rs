//! Frozen measurements of every original copy; no fixture update mode.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_declarations_match() {
    let mut records = Vec::new();
    let v: ::core::ffi::c_int = hmux2::src::input::INPUT_BUF_DEFAULT_SIZE;
    records.push(format!(
        "src/input.rs::INPUT_BUF_DEFAULT_SIZE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::options_table::INPUT_BUF_DEFAULT_SIZE;
    records.push(format!(
        "src/options_table.rs::INPUT_BUF_DEFAULT_SIZE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: hmux2::src::input::input_request_type = hmux2::src::input::INPUT_REQUEST_CLIPBOARD;
    records.push(format!(
        "src/input.rs::INPUT_REQUEST_CLIPBOARD {:?} {} {}",
        v,
        size_of::<hmux2::src::input::input_request_type>(),
        align_of::<hmux2::src::input::input_request_type>()
    ));
    let v: hmux2::src::tty_keys::input_request_type = hmux2::src::tty_keys::INPUT_REQUEST_CLIPBOARD;
    records.push(format!(
        "src/tty_keys.rs::INPUT_REQUEST_CLIPBOARD {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::input_request_type>(),
        align_of::<hmux2::src::tty_keys::input_request_type>()
    ));
    let v: hmux2::src::input::input_request_type = hmux2::src::input::INPUT_REQUEST_PALETTE;
    records.push(format!(
        "src/input.rs::INPUT_REQUEST_PALETTE {:?} {} {}",
        v,
        size_of::<hmux2::src::input::input_request_type>(),
        align_of::<hmux2::src::input::input_request_type>()
    ));
    let v: hmux2::src::tty_keys::input_request_type = hmux2::src::tty_keys::INPUT_REQUEST_PALETTE;
    records.push(format!(
        "src/tty_keys.rs::INPUT_REQUEST_PALETTE {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::input_request_type>(),
        align_of::<hmux2::src::tty_keys::input_request_type>()
    ));
    let v: hmux2::src::input::input_request_type = hmux2::src::input::INPUT_REQUEST_QUEUE;
    records.push(format!(
        "src/input.rs::INPUT_REQUEST_QUEUE {:?} {} {}",
        v,
        size_of::<hmux2::src::input::input_request_type>(),
        align_of::<hmux2::src::input::input_request_type>()
    ));
    let v: hmux2::src::tty_keys::input_request_type = hmux2::src::tty_keys::INPUT_REQUEST_QUEUE;
    records.push(format!(
        "src/tty_keys.rs::INPUT_REQUEST_QUEUE {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::input_request_type>(),
        align_of::<hmux2::src::tty_keys::input_request_type>()
    ));
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/subjects-input.txt"));
}
