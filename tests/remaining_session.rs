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
        [name, sessions, entry]
    );
    record!(
        "src/cmd_new_session.rs::session_group",
        hmux2::src::cmd_new_session::session_group,
        [name, sessions, entry]
    );
    record!(
        "src/cmd_swap_window.rs::session_group",
        hmux2::src::cmd_swap_window::session_group,
        [name, sessions, entry]
    );
    record!(
        "src/format.rs::session_group",
        hmux2::src::format::session_group,
        [name, sessions, entry]
    );
    record!(
        "src/server_fn.rs::session_group",
        hmux2::src::server_fn::session_group,
        [name, sessions, entry]
    );
    record!(
        "src/session.rs::session_group",
        hmux2::src::session::session_group,
        [name, sessions, entry]
    );
    record!(
        "src/window_tree.rs::session_group",
        hmux2::src::window_tree::session_group,
        [name, sessions, entry]
    );
    record!(
        "src/cmd_kill_session.rs::C2RustUnnamed_35",
        hmux2::src::cmd_kill_session::session_group_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/cmd_new_session.rs::C2RustUnnamed_35",
        hmux2::src::cmd_new_session::session_group_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/cmd_swap_window.rs::C2RustUnnamed_35",
        hmux2::src::cmd_swap_window::session_group_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/format.rs::C2RustUnnamed_41",
        hmux2::src::format::session_group_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/server_fn.rs::C2RustUnnamed_38",
        hmux2::src::server_fn::session_group_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/session.rs::C2RustUnnamed_35",
        hmux2::src::session::session_group_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/window_tree.rs::C2RustUnnamed_39",
        hmux2::src::window_tree::session_group_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/cmd_kill_session.rs::C2RustUnnamed_36",
        hmux2::src::cmd_kill_session::session_group_sessions,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_new_session.rs::C2RustUnnamed_36",
        hmux2::src::cmd_new_session::session_group_sessions,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_swap_window.rs::C2RustUnnamed_36",
        hmux2::src::cmd_swap_window::session_group_sessions,
        [tqh_first, tqh_last]
    );
    record!(
        "src/format.rs::C2RustUnnamed_42",
        hmux2::src::format::session_group_sessions,
        [tqh_first, tqh_last]
    );
    record!(
        "src/server_fn.rs::C2RustUnnamed_39",
        hmux2::src::server_fn::session_group_sessions,
        [tqh_first, tqh_last]
    );
    record!(
        "src/session.rs::C2RustUnnamed_36",
        hmux2::src::session::session_group_sessions,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_tree.rs::C2RustUnnamed_40",
        hmux2::src::window_tree::session_group_sessions,
        [tqh_first, tqh_last]
    );
    record!(
        "src/format.rs::session_groups",
        hmux2::src::format::session_groups,
        [rbh_root]
    );
    record!(
        "src/session.rs::session_groups",
        hmux2::src::session::session_groups,
        [rbh_root]
    );
    record!(
        "src/cfg.rs::sessions",
        hmux2::src::cfg::sessions,
        [rbh_root]
    );
    record!(
        "src/cmd_attach_session.rs::sessions",
        hmux2::src::cmd_attach_session::sessions,
        [rbh_root]
    );
    record!(
        "src/cmd_find.rs::sessions",
        hmux2::src::cmd_find::sessions,
        [rbh_root]
    );
    record!(
        "src/cmd_kill_session.rs::sessions",
        hmux2::src::cmd_kill_session::sessions,
        [rbh_root]
    );
    record!(
        "src/cmd_list_panes.rs::sessions",
        hmux2::src::cmd_list_panes::sessions,
        [rbh_root]
    );
    record!(
        "src/cmd_rename_session.rs::sessions",
        hmux2::src::cmd_rename_session::sessions,
        [rbh_root]
    );
    record!(
        "src/format.rs::sessions",
        hmux2::src::format::sessions,
        [rbh_root]
    );
    record!(
        "src/monitor.rs::sessions",
        hmux2::src::monitor::sessions,
        [rbh_root]
    );
    record!(
        "src/options.rs::sessions",
        hmux2::src::options::sessions,
        [rbh_root]
    );
    record!(
        "src/resize.rs::sessions",
        hmux2::src::resize::sessions,
        [rbh_root]
    );
    record!(
        "src/server.rs::sessions",
        hmux2::src::server::sessions,
        [rbh_root]
    );
    record!(
        "src/server_fn.rs::sessions",
        hmux2::src::server_fn::sessions,
        [rbh_root]
    );
    record!(
        "src/session.rs::sessions",
        hmux2::src::session::sessions,
        [rbh_root]
    );
    record!(
        "src/sort.rs::sessions",
        hmux2::src::sort::sessions,
        [rbh_root]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/remaining-session.txt"));
}
