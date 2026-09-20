//! Frozen measurements of every original copy; no fixture update mode.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_declarations_match() {
    let mut records = Vec::new();
    let v: [::core::ffi::c_char; 3] = hmux2::src::grid_reader::WHITESPACE;
    records.push(format!(
        "src/grid_reader.rs::WHITESPACE {:?} {} {}",
        v,
        size_of::<[::core::ffi::c_char; 3]>(),
        align_of::<[::core::ffi::c_char; 3]>()
    ));
    let v: [::core::ffi::c_char; 3] = hmux2::src::window_copy::WHITESPACE;
    records.push(format!(
        "src/window_copy.rs::WHITESPACE {:?} {} {}",
        v,
        size_of::<[::core::ffi::c_char; 3]>(),
        align_of::<[::core::ffi::c_char; 3]>()
    ));
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/subjects-grid.txt"));
}
