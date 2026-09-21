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
        "src/json.rs::json_fields",
        hmux2::src::json::json_fields,
        [entries]
    );
    record!(
        "src/json.rs::json_members",
        hmux2::src::json::json_members,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_display_message.rs::json_node",
        *mut hmux2::src::cmd_display_message::json_node,
        []
    );
    record!(
        "src/json.rs::json_node",
        hmux2::src::json::json_node,
        [type_0, key, parent, c2rust_unnamed, oentry, aentry]
    );
    record!(
        "src/layout_custom.rs::json_node",
        *mut hmux2::src::layout_custom::json_node,
        []
    );
    record!(
        "src/json.rs::C2RustUnnamed_0",
        hmux2::src::json::json_node_aentry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/json.rs::C2RustUnnamed_2",
        hmux2::src::json::json_node_c2rust_unnamed,
        [str_0, num, boolean, fields, members]
    );
    record!(
        "src/json.rs::C2RustUnnamed_1",
        hmux2::src::json::json_node_oentry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/json.rs::json_node_type",
        hmux2::src::json::json_node_type,
        []
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/remaining-json.txt"));
}
