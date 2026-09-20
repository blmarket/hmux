//! Frozen measurements of every original copy; no fixture update mode.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_declarations_match() {
    let mut records = Vec::new();
    let v: ::core::ffi::c_int = hmux2::src::alerts::ALERT_ANY;
    records.push(format!(
        "src/alerts.rs::ALERT_ANY {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::options_table::ALERT_ANY;
    records.push(format!(
        "src/options_table.rs::ALERT_ANY {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::alerts::ALERT_CURRENT;
    records.push(format!(
        "src/alerts.rs::ALERT_CURRENT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::alerts::ALERT_OTHER;
    records.push(format!(
        "src/alerts.rs::ALERT_OTHER {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::options_table::ALERT_OTHER;
    records.push(format!(
        "src/options_table.rs::ALERT_OTHER {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::alerts::VISUAL_BOTH;
    records.push(format!(
        "src/alerts.rs::VISUAL_BOTH {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::alerts::VISUAL_OFF;
    records.push(format!(
        "src/alerts.rs::VISUAL_OFF {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::options_table::VISUAL_OFF;
    records.push(format!(
        "src/options_table.rs::VISUAL_OFF {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/subjects-alerts.txt"));
}
