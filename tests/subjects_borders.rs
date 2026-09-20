//! Frozen measurements of every original copy; no fixture update mode.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_declarations_match() {
    let mut records = Vec::new();
    let v: [::core::ffi::c_char; 14] = hmux2::src::screen_write::CELL_BORDERS;
    records.push(format!(
        "src/screen_write.rs::CELL_BORDERS {:?} {} {}",
        v,
        size_of::<[::core::ffi::c_char; 14]>(),
        align_of::<[::core::ffi::c_char; 14]>()
    ));
    let v: [::core::ffi::c_char; 14] = hmux2::src::window_border::CELL_BORDERS;
    records.push(format!(
        "src/window_border.rs::CELL_BORDERS {:?} {} {}",
        v,
        size_of::<[::core::ffi::c_char; 14]>(),
        align_of::<[::core::ffi::c_char; 14]>()
    ));
    let v: [::core::ffi::c_char; 14] = hmux2::src::window_panes::CELL_BORDERS;
    records.push(format!(
        "src/window_panes.rs::CELL_BORDERS {:?} {} {}",
        v,
        size_of::<[::core::ffi::c_char; 14]>(),
        align_of::<[::core::ffi::c_char; 14]>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen_redraw::CELL_LD;
    records.push(format!(
        "src/screen_redraw.rs::CELL_LD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen_write::CELL_LD;
    records.push(format!(
        "src/screen_write.rs::CELL_LD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_panes::CELL_LD;
    records.push(format!(
        "src/window_panes.rs::CELL_LD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen_redraw::CELL_LR;
    records.push(format!(
        "src/screen_redraw.rs::CELL_LR {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen_write::CELL_LR;
    records.push(format!(
        "src/screen_write.rs::CELL_LR {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_panes::CELL_LR;
    records.push(format!(
        "src/window_panes.rs::CELL_LR {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen_redraw::CELL_LRD;
    records.push(format!(
        "src/screen_redraw.rs::CELL_LRD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_panes::CELL_LRD;
    records.push(format!(
        "src/window_panes.rs::CELL_LRD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen_redraw::CELL_LRU;
    records.push(format!(
        "src/screen_redraw.rs::CELL_LRU {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_panes::CELL_LRU;
    records.push(format!(
        "src/window_panes.rs::CELL_LRU {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen_redraw::CELL_LRUD;
    records.push(format!(
        "src/screen_redraw.rs::CELL_LRUD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_panes::CELL_LRUD;
    records.push(format!(
        "src/window_panes.rs::CELL_LRUD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen_redraw::CELL_LU;
    records.push(format!(
        "src/screen_redraw.rs::CELL_LU {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen_write::CELL_LU;
    records.push(format!(
        "src/screen_write.rs::CELL_LU {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_panes::CELL_LU;
    records.push(format!(
        "src/window_panes.rs::CELL_LU {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen_redraw::CELL_NONE;
    records.push(format!(
        "src/screen_redraw.rs::CELL_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_border::CELL_NONE;
    records.push(format!(
        "src/window_border.rs::CELL_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_panes::CELL_NONE;
    records.push(format!(
        "src/window_panes.rs::CELL_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen_redraw::CELL_RD;
    records.push(format!(
        "src/screen_redraw.rs::CELL_RD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen_write::CELL_RD;
    records.push(format!(
        "src/screen_write.rs::CELL_RD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_panes::CELL_RD;
    records.push(format!(
        "src/window_panes.rs::CELL_RD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen_redraw::CELL_RU;
    records.push(format!(
        "src/screen_redraw.rs::CELL_RU {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen_write::CELL_RU;
    records.push(format!(
        "src/screen_write.rs::CELL_RU {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_panes::CELL_RU;
    records.push(format!(
        "src/window_panes.rs::CELL_RU {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen_redraw::CELL_UD;
    records.push(format!(
        "src/screen_redraw.rs::CELL_UD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen_write::CELL_UD;
    records.push(format!(
        "src/screen_write.rs::CELL_UD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_panes::CELL_UD;
    records.push(format!(
        "src/window_panes.rs::CELL_UD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen_redraw::CELL_ULD;
    records.push(format!(
        "src/screen_redraw.rs::CELL_ULD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen_write::CELL_ULD;
    records.push(format!(
        "src/screen_write.rs::CELL_ULD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_panes::CELL_ULD;
    records.push(format!(
        "src/window_panes.rs::CELL_ULD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen_redraw::CELL_URD;
    records.push(format!(
        "src/screen_redraw.rs::CELL_URD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen_write::CELL_URD;
    records.push(format!(
        "src/screen_write.rs::CELL_URD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_panes::CELL_URD;
    records.push(format!(
        "src/window_panes.rs::CELL_URD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: [::core::ffi::c_char; 14] = hmux2::src::screen_write::SIMPLE_BORDERS;
    records.push(format!(
        "src/screen_write.rs::SIMPLE_BORDERS {:?} {} {}",
        v,
        size_of::<[::core::ffi::c_char; 14]>(),
        align_of::<[::core::ffi::c_char; 14]>()
    ));
    let v: [::core::ffi::c_char; 14] = hmux2::src::window_border::SIMPLE_BORDERS;
    records.push(format!(
        "src/window_border.rs::SIMPLE_BORDERS {:?} {} {}",
        v,
        size_of::<[::core::ffi::c_char; 14]>(),
        align_of::<[::core::ffi::c_char; 14]>()
    ));
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/subjects-borders.txt"));
}
