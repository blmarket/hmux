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
        "src/cmd_show_messages.rs::message_entry",
        hmux2::src::cmd_show_messages::message_entry,
        [msg, msg_num, msg_time, entry]
    );
    record!(
        "src/server.rs::message_entry",
        hmux2::src::server::message_entry,
        [msg, msg_num, msg_time, entry]
    );
    record!(
        "src/cmd_show_messages.rs::C2RustUnnamed_35",
        hmux2::src::cmd_show_messages::message_entry_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/server.rs::C2RustUnnamed_36",
        hmux2::src::server::message_entry_entry,
        [tqe_next, tqe_prev]
    );
    record!(
        "src/cmd_show_messages.rs::message_list",
        hmux2::src::cmd_show_messages::message_list,
        [tqh_first, tqh_last]
    );
    record!(
        "src/server.rs::message_list",
        hmux2::src::server::message_list,
        [tqh_first, tqh_last]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/remaining-status.txt"));
}
