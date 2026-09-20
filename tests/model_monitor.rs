//! Sizes, alignments, and every named field offset across re-exported copies.
//! Internal owner fixtures include the hmux-rt handle migration (128 -> 56 byte events).
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
        "src/control.rs::monitor_cb",
        hmux2::src::control::monitor_cb,
        []
    );
    record!(
        "src/hooks.rs::monitor_cb",
        hmux2::src::hooks::monitor_cb,
        []
    );
    record!(
        "src/monitor.rs::monitor_cb",
        hmux2::src::monitor::monitor_cb,
        []
    );
    record!(
        "src/control.rs::monitor_change",
        hmux2::src::control::monitor_change,
        [name, value, last, c, s, wl, wp]
    );
    record!(
        "src/hooks.rs::monitor_change",
        hmux2::src::hooks::monitor_change,
        [name, value, last, c, s, wl, wp]
    );
    record!(
        "src/monitor.rs::monitor_change",
        hmux2::src::monitor::monitor_change,
        [name, value, last, c, s, wl, wp]
    );
    record!(
        "src/monitor.rs::monitor_item",
        hmux2::src::monitor::monitor_item,
        [name, format, type_0, id, flags, last, panes, windows, fire_count, fire_time, entry]
    );
    record!(
        "src/monitor.rs::C2RustUnnamed_35",
        hmux2::src::monitor::monitor_item_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/monitor.rs::monitor_items",
        hmux2::src::monitor::monitor_items,
        [rbh_root]
    );
    record!(
        "src/monitor.rs::monitor_pane",
        hmux2::src::monitor::monitor_pane,
        [pane, idx, last, generation, entry]
    );
    record!(
        "src/monitor.rs::C2RustUnnamed_37",
        hmux2::src::monitor::monitor_pane_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/monitor.rs::monitor_panes",
        hmux2::src::monitor::monitor_panes,
        [rbh_root]
    );
    record!(
        "src/control.rs::monitor_set",
        *mut hmux2::src::control::monitor_set,
        []
    );
    record!(
        "src/hooks.rs::monitor_set",
        *mut hmux2::src::hooks::monitor_set,
        []
    );
    record!(
        "src/monitor.rs::monitor_set",
        hmux2::src::monitor::monitor_set,
        [client, session, cb, data, items, timer, generation]
    );
    record!(
        "src/monitor.rs::monitor_window",
        hmux2::src::monitor::monitor_window,
        [window, idx, last, generation, entry]
    );
    record!(
        "src/monitor.rs::C2RustUnnamed_36",
        hmux2::src::monitor::monitor_window_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/monitor.rs::monitor_windows",
        hmux2::src::monitor::monitor_windows,
        [rbh_root]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/model-monitor.txt"));
}
