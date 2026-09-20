//! Frozen measurements of every original copy; no fixture update mode.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_declarations_match() {
    let mut records = Vec::new();
    let v: *mut ::core::ffi::c_void = hmux2::src::format::NULL_0;
    records.push(format!(
        "src/format.rs::NULL_0 {:?} {} {}",
        v,
        size_of::<*mut ::core::ffi::c_void>(),
        align_of::<*mut ::core::ffi::c_void>()
    ));
    let v: *mut ::core::ffi::c_void = hmux2::src::input::NULL_0;
    records.push(format!(
        "src/input.rs::NULL_0 {:?} {} {}",
        v,
        size_of::<*mut ::core::ffi::c_void>(),
        align_of::<*mut ::core::ffi::c_void>()
    ));
    let v: *mut ::core::ffi::c_void = hmux2::src::tty_acs::NULL_0;
    records.push(format!(
        "src/tty_acs.rs::NULL_0 {:?} {} {}",
        v,
        size_of::<*mut ::core::ffi::c_void>(),
        align_of::<*mut ::core::ffi::c_void>()
    ));
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/subjects-abi.txt"));
}
