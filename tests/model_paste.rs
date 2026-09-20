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
        "src/cmd_list_buffers.rs::paste_buffer",
        hmux2::src::cmd_list_buffers::paste_buffer,
        [data, size, name, created, automatic, order, name_entry, time_entry]
    );
    record!(
        "src/cmd_paste_buffer.rs::paste_buffer",
        hmux2::src::cmd_paste_buffer::paste_buffer,
        [data, size, name, created, automatic, order, name_entry, time_entry]
    );
    record!(
        "src/cmd_save_buffer.rs::paste_buffer",
        hmux2::src::cmd_save_buffer::paste_buffer,
        [data, size, name, created, automatic, order, name_entry, time_entry]
    );
    record!(
        "src/cmd_set_buffer.rs::paste_buffer",
        hmux2::src::cmd_set_buffer::paste_buffer,
        [data, size, name, created, automatic, order, name_entry, time_entry]
    );
    record!(
        "src/format.rs::paste_buffer",
        hmux2::src::format::paste_buffer,
        [data, size, name, created, automatic, order, name_entry, time_entry]
    );
    record!(
        "src/input.rs::paste_buffer",
        hmux2::src::input::paste_buffer,
        [data, size, name, created, automatic, order, name_entry, time_entry]
    );
    record!(
        "src/paste.rs::paste_buffer",
        hmux2::src::paste::paste_buffer,
        [data, size, name, created, automatic, order, name_entry, time_entry]
    );
    record!(
        "src/prompt.rs::paste_buffer",
        hmux2::src::prompt::paste_buffer,
        [data, size, name, created, automatic, order, name_entry, time_entry]
    );
    record!(
        "src/sort.rs::paste_buffer",
        hmux2::src::sort::paste_buffer,
        [data, size, name, created, automatic, order, name_entry, time_entry]
    );
    record!(
        "src/window_buffer.rs::paste_buffer",
        hmux2::src::window_buffer::paste_buffer,
        [data, size, name, created, automatic, order, name_entry, time_entry]
    );
    record!(
        "src/window_copy.rs::paste_buffer",
        hmux2::src::window_copy::paste_buffer,
        [data, size, name, created, automatic, order, name_entry, time_entry]
    );
    record!(
        "src/cmd_list_buffers.rs::C2RustUnnamed_36",
        hmux2::src::cmd_list_buffers::paste_buffer_name_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/cmd_paste_buffer.rs::C2RustUnnamed_36",
        hmux2::src::cmd_paste_buffer::paste_buffer_name_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/cmd_save_buffer.rs::C2RustUnnamed_36",
        hmux2::src::cmd_save_buffer::paste_buffer_name_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/cmd_set_buffer.rs::C2RustUnnamed_36",
        hmux2::src::cmd_set_buffer::paste_buffer_name_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/format.rs::C2RustUnnamed_32",
        hmux2::src::format::paste_buffer_name_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/input.rs::C2RustUnnamed_43",
        hmux2::src::input::paste_buffer_name_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/paste.rs::C2RustUnnamed_0",
        hmux2::src::paste::paste_buffer_name_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/prompt.rs::C2RustUnnamed_40",
        hmux2::src::prompt::paste_buffer_name_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/sort.rs::C2RustUnnamed_36",
        hmux2::src::sort::paste_buffer_name_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/window_buffer.rs::C2RustUnnamed_40",
        hmux2::src::window_buffer::paste_buffer_name_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/window_copy.rs::C2RustUnnamed_41",
        hmux2::src::window_copy::paste_buffer_name_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/cmd_list_buffers.rs::C2RustUnnamed_35",
        hmux2::src::cmd_list_buffers::paste_buffer_time_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/cmd_paste_buffer.rs::C2RustUnnamed_35",
        hmux2::src::cmd_paste_buffer::paste_buffer_time_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/cmd_save_buffer.rs::C2RustUnnamed_35",
        hmux2::src::cmd_save_buffer::paste_buffer_time_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/cmd_set_buffer.rs::C2RustUnnamed_35",
        hmux2::src::cmd_set_buffer::paste_buffer_time_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/format.rs::C2RustUnnamed_31",
        hmux2::src::format::paste_buffer_time_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/input.rs::C2RustUnnamed_42",
        hmux2::src::input::paste_buffer_time_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/paste.rs::C2RustUnnamed",
        hmux2::src::paste::paste_buffer_time_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/prompt.rs::C2RustUnnamed_39",
        hmux2::src::prompt::paste_buffer_time_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/sort.rs::C2RustUnnamed_35",
        hmux2::src::sort::paste_buffer_time_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/window_buffer.rs::C2RustUnnamed_39",
        hmux2::src::window_buffer::paste_buffer_time_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/window_copy.rs::C2RustUnnamed_40",
        hmux2::src::window_copy::paste_buffer_time_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/model-paste.txt"));
}
