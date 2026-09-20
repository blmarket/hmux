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
        "src/alerts.rs::clients",
        hmux2::src::alerts::clients,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cfg.rs::clients",
        hmux2::src::cfg::clients,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_attach_session.rs::clients",
        hmux2::src::cmd_attach_session::clients,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_detach_client.rs::clients",
        hmux2::src::cmd_detach_client::clients,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_find.rs::clients",
        hmux2::src::cmd_find::clients,
        [tqh_first, tqh_last]
    );
    record!(
        "src/cmd_select_pane.rs::clients",
        hmux2::src::cmd_select_pane::clients,
        [tqh_first, tqh_last]
    );
    record!(
        "src/control_notify.rs::clients",
        hmux2::src::control_notify::clients,
        [tqh_first, tqh_last]
    );
    record!(
        "src/format.rs::clients",
        hmux2::src::format::clients,
        [tqh_first, tqh_last]
    );
    record!(
        "src/input.rs::clients",
        hmux2::src::input::clients,
        [tqh_first, tqh_last]
    );
    record!(
        "src/key_bindings.rs::clients",
        hmux2::src::key_bindings::clients,
        [tqh_first, tqh_last]
    );
    record!(
        "src/options.rs::clients",
        hmux2::src::options::clients,
        [tqh_first, tqh_last]
    );
    record!(
        "src/resize.rs::clients",
        hmux2::src::resize::clients,
        [tqh_first, tqh_last]
    );
    record!(
        "src/server.rs::clients",
        hmux2::src::server::clients,
        [tqh_first, tqh_last]
    );
    record!(
        "src/server_acl.rs::clients",
        hmux2::src::server_acl::clients,
        [tqh_first, tqh_last]
    );
    record!(
        "src/server_client.rs::clients",
        hmux2::src::server_client::clients,
        [tqh_first, tqh_last]
    );
    record!(
        "src/server_fn.rs::clients",
        hmux2::src::server_fn::clients,
        [tqh_first, tqh_last]
    );
    record!(
        "src/sort.rs::clients",
        hmux2::src::sort::clients,
        [tqh_first, tqh_last]
    );
    record!(
        "src/spawn.rs::clients",
        hmux2::src::spawn::clients,
        [tqh_first, tqh_last]
    );
    record!(
        "src/status.rs::clients",
        hmux2::src::status::clients,
        [tqh_first, tqh_last]
    );
    record!(
        "src/tty.rs::clients",
        hmux2::src::tty::clients,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window.rs::clients",
        hmux2::src::window::clients,
        [tqh_first, tqh_last]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/remaining-client.txt"));
}
