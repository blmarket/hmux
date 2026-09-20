//! Frozen measurements from every original translation-unit copy (Linux x86_64).
use std::mem::{align_of, offset_of, size_of};
#[test]
fn platform_layouts() {
    let mut records = Vec::new();
    macro_rules! layout {
    ($label:literal, $ty:path, [$($field:ident),*]) => {
        records.push(format!(concat!($label, " {} {}", $(" ", stringify!($field), "={}"),*),
            size_of::<$ty>(), align_of::<$ty>() $(, offset_of!($ty, $field))*));
    };
}

    layout!(
        "src/cmd_server_access.rs::group",
        hmux2::src::cmd_server_access::group,
        [gr_name, gr_passwd, gr_gid, gr_mem]
    );
    layout!(
        "src/server_acl.rs::group",
        hmux2::src::server_acl::group,
        [gr_name, gr_passwd, gr_gid, gr_mem]
    );
    layout!(
        "src/cmd_parse.rs::passwd",
        hmux2::src::cmd_parse::passwd,
        [pw_name, pw_passwd, pw_uid, pw_gid, pw_gecos, pw_dir, pw_shell]
    );
    layout!(
        "src/cmd_queue.rs::passwd",
        hmux2::src::cmd_queue::passwd,
        [pw_name, pw_passwd, pw_uid, pw_gid, pw_gecos, pw_dir, pw_shell]
    );
    layout!(
        "src/cmd_server_access.rs::passwd",
        hmux2::src::cmd_server_access::passwd,
        [pw_name, pw_passwd, pw_uid, pw_gid, pw_gecos, pw_dir, pw_shell]
    );
    layout!(
        "src/format.rs::passwd",
        hmux2::src::format::passwd,
        [pw_name, pw_passwd, pw_uid, pw_gid, pw_gecos, pw_dir, pw_shell]
    );
    layout!(
        "src/server_acl.rs::passwd",
        hmux2::src::server_acl::passwd,
        [pw_name, pw_passwd, pw_uid, pw_gid, pw_gecos, pw_dir, pw_shell]
    );
    layout!(
        "src/tmux.rs::passwd",
        hmux2::src::tmux::passwd,
        [pw_name, pw_passwd, pw_uid, pw_gid, pw_gecos, pw_dir, pw_shell]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/platform-account.txt"));
}
