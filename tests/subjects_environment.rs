//! Frozen measurements of every original copy; no fixture update mode.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_declarations_match() {
    let mut records = Vec::new();
    let v: ::core::ffi::c_int = hmux2::src::cmd_parse::ENVIRON_HIDDEN;
    records.push(format!(
        "src/cmd_parse.rs::ENVIRON_HIDDEN {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_set_environment::ENVIRON_HIDDEN;
    records.push(format!(
        "src/cmd_set_environment.rs::ENVIRON_HIDDEN {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_show_environment::ENVIRON_HIDDEN;
    records.push(format!(
        "src/cmd_show_environment.rs::ENVIRON_HIDDEN {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::environ::ENVIRON_HIDDEN;
    records.push(format!(
        "src/environ.rs::ENVIRON_HIDDEN {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::ENVIRON_HIDDEN;
    records.push(format!(
        "src/format.rs::ENVIRON_HIDDEN {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_customize::ENVIRON_HIDDEN;
    records.push(format!(
        "src/window_customize.rs::ENVIRON_HIDDEN {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/subjects-environment.txt"));
}
