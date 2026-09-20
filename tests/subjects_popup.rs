//! Frozen measurements of every original copy; no fixture update mode.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_declarations_match() {
    let mut records = Vec::new();
    let v: ::core::ffi::c_int = hmux2::src::cmd_display_menu::POPUP_CLOSEANYKEY;
    records.push(format!(
        "src/cmd_display_menu.rs::POPUP_CLOSEANYKEY {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::popup::POPUP_CLOSEANYKEY;
    records.push(format!(
        "src/popup.rs::POPUP_CLOSEANYKEY {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_display_menu::POPUP_CLOSEEXIT;
    records.push(format!(
        "src/cmd_display_menu.rs::POPUP_CLOSEEXIT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::popup::POPUP_CLOSEEXIT;
    records.push(format!(
        "src/popup.rs::POPUP_CLOSEEXIT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_display_menu::POPUP_CLOSEEXITZERO;
    records.push(format!(
        "src/cmd_display_menu.rs::POPUP_CLOSEEXITZERO {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::popup::POPUP_CLOSEEXITZERO;
    records.push(format!(
        "src/popup.rs::POPUP_CLOSEEXITZERO {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/subjects-popup.txt"));
}
