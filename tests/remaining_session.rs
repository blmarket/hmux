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
        "src/cmd_kill_session.rs::session_group",
        hmux2::src::cmd_kill_session::session_group,
        [name, entry]
    );
    record!(
        "src/cmd_new_session.rs::session_group",
        hmux2::src::cmd_new_session::session_group,
        [name, entry]
    );
    record!(
        "src/cmd_swap_window.rs::session_group",
        hmux2::src::cmd_swap_window::session_group,
        [name, entry]
    );
    record!(
        "src/format.rs::session_group",
        hmux2::src::format::session_group,
        [name, entry]
    );
    record!(
        "src/server_fn.rs::session_group",
        hmux2::src::server_fn::session_group,
        [name, entry]
    );
    record!(
        "src/session.rs::session_group",
        hmux2::src::session::session_group,
        [name, entry]
    );
    record!(
        "src/window_tree.rs::session_group",
        hmux2::src::window_tree::session_group,
        [name, entry]
    );
    record!(
        "src/cmd_kill_session.rs::C2RustUnnamed_35",
        hmux2::src::cmd_kill_session::session_group_entry,
        [owner]
    );
    record!(
        "src/cmd_new_session.rs::C2RustUnnamed_35",
        hmux2::src::cmd_new_session::session_group_entry,
        [owner]
    );
    record!(
        "src/cmd_swap_window.rs::C2RustUnnamed_35",
        hmux2::src::cmd_swap_window::session_group_entry,
        [owner]
    );
    record!(
        "src/format.rs::C2RustUnnamed_41",
        hmux2::src::format::session_group_entry,
        [owner]
    );
    record!(
        "src/server_fn.rs::C2RustUnnamed_38",
        hmux2::src::server_fn::session_group_entry,
        [owner]
    );
    record!(
        "src/session.rs::C2RustUnnamed_35",
        hmux2::src::session::session_group_entry,
        [owner]
    );
    record!(
        "src/window_tree.rs::C2RustUnnamed_39",
        hmux2::src::window_tree::session_group_entry,
        [owner]
    );
    record!(
        "src/format.rs::session_groups",
        hmux2::src::format::session_groups,
        [storage]
    );
    record!(
        "src/session.rs::session_groups",
        hmux2::src::session::session_groups,
        [storage]
    );
    record!("src/cfg.rs::sessions", hmux2::src::cfg::sessions, [storage]);
    record!(
        "src/cmd_attach_session.rs::sessions",
        hmux2::src::cmd_attach_session::sessions,
        [storage]
    );
    record!(
        "src/cmd_find.rs::sessions",
        hmux2::src::cmd_find::sessions,
        [storage]
    );
    record!(
        "src/cmd_kill_session.rs::sessions",
        hmux2::src::cmd_kill_session::sessions,
        [storage]
    );
    record!(
        "src/cmd_list_panes.rs::sessions",
        hmux2::src::cmd_list_panes::sessions,
        [storage]
    );
    record!(
        "src/cmd_rename_session.rs::sessions",
        hmux2::src::cmd_rename_session::sessions,
        [storage]
    );
    record!(
        "src/format.rs::sessions",
        hmux2::src::format::sessions,
        [storage]
    );
    record!(
        "src/monitor.rs::sessions",
        hmux2::src::monitor::sessions,
        [storage]
    );
    record!(
        "src/options.rs::sessions",
        hmux2::src::options::sessions,
        [storage]
    );
    record!(
        "src/resize.rs::sessions",
        hmux2::src::resize::sessions,
        [storage]
    );
    record!(
        "src/server.rs::sessions",
        hmux2::src::server::sessions,
        [storage]
    );
    record!(
        "src/server_fn.rs::sessions",
        hmux2::src::server_fn::sessions,
        [storage]
    );
    record!(
        "src/session.rs::sessions",
        hmux2::src::session::sessions,
        [storage]
    );
    record!(
        "src/sort.rs::sessions",
        hmux2::src::sort::sessions,
        [storage]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/remaining-session.txt"));
}
