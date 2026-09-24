//! Current spawn context sizes, alignments, and named field offsets.
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
        "src/cmd_new_session.rs::spawn_context",
        hmux2::src::cmd_new_session::spawn_context,
        [item, s, wl, tc, wp0, lc, name, argv, environ, idx, cwd, flags]
    );
    record!(
        "src/cmd_new_window.rs::spawn_context",
        hmux2::src::cmd_new_window::spawn_context,
        [item, s, wl, tc, wp0, lc, name, argv, environ, idx, cwd, flags]
    );
    record!(
        "src/cmd_respawn_pane.rs::spawn_context",
        hmux2::src::cmd_respawn_pane::spawn_context,
        [item, s, wl, tc, wp0, lc, name, argv, environ, idx, cwd, flags]
    );
    record!(
        "src/cmd_respawn_window.rs::spawn_context",
        hmux2::src::cmd_respawn_window::spawn_context,
        [item, s, wl, tc, wp0, lc, name, argv, environ, idx, cwd, flags]
    );
    record!(
        "src/cmd_split_window.rs::spawn_context",
        hmux2::src::cmd_split_window::spawn_context,
        [item, s, wl, tc, wp0, lc, name, argv, environ, idx, cwd, flags]
    );
    record!(
        "src/spawn.rs::spawn_context",
        hmux2::src::spawn::spawn_context,
        [item, s, wl, tc, wp0, lc, name, argv, environ, idx, cwd, flags]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/remaining-spawn.txt"));
}
