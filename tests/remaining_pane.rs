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
        "src/cmd_find.rs::window_pane_tree",
        hmux2::src::cmd_find::window_pane_tree,
        [storage]
    );
    record!(
        "src/options.rs::window_pane_tree",
        hmux2::src::options::window_pane_tree,
        [storage]
    );
    record!(
        "src/server.rs::window_pane_tree",
        hmux2::src::server::window_pane_tree,
        [storage]
    );
    record!(
        "src/server_client.rs::window_pane_tree",
        hmux2::src::server_client::window_pane_tree,
        [storage]
    );
    record!(
        "src/window.rs::window_pane_tree",
        hmux2::src::window::window_pane_tree,
        [storage]
    );
    record!(
        "src/cmd_join_pane.rs::window_panes_zindex",
        hmux2::src::cmd_join_pane::window_panes_zindex,
        [tqh_first, tqh_last]
    );
    record!(
        "src/screen_redraw.rs::window_panes_zindex",
        hmux2::src::screen_redraw::window_panes_zindex,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window.rs::window_panes_zindex",
        hmux2::src::window::window_panes_zindex,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_panes.rs::window_panes_zindex",
        hmux2::src::window_panes::window_panes_zindex,
        [tqh_first, tqh_last]
    );
    record!(
        "src/window_visible.rs::window_panes_zindex",
        hmux2::src::window_visible::window_panes_zindex,
        [tqh_first, tqh_last]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/remaining-pane.txt"));
}
