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
        "src/format_draw.rs::screen_write_ctx",
        hmux2::src::format_draw::screen_write_ctx,
        [wp, s, flags, init_ctx_cb, arg, item, scrolled, bg]
    );
    record!(
        "src/input.rs::screen_write_ctx",
        hmux2::src::input::screen_write_ctx,
        [wp, s, flags, init_ctx_cb, arg, item, scrolled, bg]
    );
    record!(
        "src/menu.rs::screen_write_ctx",
        hmux2::src::menu::screen_write_ctx,
        [wp, s, flags, init_ctx_cb, arg, item, scrolled, bg]
    );
    record!(
        "src/mode_tree.rs::screen_write_ctx",
        hmux2::src::mode_tree::screen_write_ctx,
        [wp, s, flags, init_ctx_cb, arg, item, scrolled, bg]
    );
    record!(
        "src/popup.rs::screen_write_ctx",
        hmux2::src::popup::screen_write_ctx,
        [wp, s, flags, init_ctx_cb, arg, item, scrolled, bg]
    );
    record!(
        "src/prompt.rs::screen_write_ctx",
        hmux2::src::prompt::screen_write_ctx,
        [wp, s, flags, init_ctx_cb, arg, item, scrolled, bg]
    );
    record!(
        "src/screen_redraw.rs::screen_write_ctx",
        hmux2::src::screen_redraw::screen_write_ctx,
        [wp, s, flags, init_ctx_cb, arg, item, scrolled, bg]
    );
    record!(
        "src/screen_write.rs::screen_write_ctx",
        hmux2::src::screen_write::screen_write_ctx,
        [wp, s, flags, init_ctx_cb, arg, item, scrolled, bg]
    );
    record!(
        "src/server_fn.rs::screen_write_ctx",
        hmux2::src::server_fn::screen_write_ctx,
        [wp, s, flags, init_ctx_cb, arg, item, scrolled, bg]
    );
    record!(
        "src/status.rs::screen_write_ctx",
        hmux2::src::status::screen_write_ctx,
        [wp, s, flags, init_ctx_cb, arg, item, scrolled, bg]
    );
    record!(
        "src/window_border.rs::screen_write_ctx",
        hmux2::src::window_border::screen_write_ctx,
        [wp, s, flags, init_ctx_cb, arg, item, scrolled, bg]
    );
    record!(
        "src/window_buffer.rs::screen_write_ctx",
        hmux2::src::window_buffer::screen_write_ctx,
        [wp, s, flags, init_ctx_cb, arg, item, scrolled, bg]
    );
    record!(
        "src/window_client.rs::screen_write_ctx",
        hmux2::src::window_client::screen_write_ctx,
        [wp, s, flags, init_ctx_cb, arg, item, scrolled, bg]
    );
    record!(
        "src/window_clock.rs::screen_write_ctx",
        hmux2::src::window_clock::screen_write_ctx,
        [wp, s, flags, init_ctx_cb, arg, item, scrolled, bg]
    );
    record!(
        "src/window_copy.rs::screen_write_ctx",
        hmux2::src::window_copy::screen_write_ctx,
        [wp, s, flags, init_ctx_cb, arg, item, scrolled, bg]
    );
    record!(
        "src/window_customize.rs::screen_write_ctx",
        hmux2::src::window_customize::screen_write_ctx,
        [wp, s, flags, init_ctx_cb, arg, item, scrolled, bg]
    );
    record!(
        "src/window_panes.rs::screen_write_ctx",
        hmux2::src::window_panes::screen_write_ctx,
        [wp, s, flags, init_ctx_cb, arg, item, scrolled, bg]
    );
    record!(
        "src/window_switch.rs::screen_write_ctx",
        hmux2::src::window_switch::screen_write_ctx,
        [wp, s, flags, init_ctx_cb, arg, item, scrolled, bg]
    );
    record!(
        "src/window_tree.rs::screen_write_ctx",
        hmux2::src::window_tree::screen_write_ctx,
        [wp, s, flags, init_ctx_cb, arg, item, scrolled, bg]
    );
    record!(
        "src/format_draw.rs::screen_write_init_ctx_cb",
        hmux2::src::format_draw::screen_write_init_ctx_cb,
        []
    );
    record!(
        "src/input.rs::screen_write_init_ctx_cb",
        hmux2::src::input::screen_write_init_ctx_cb,
        []
    );
    record!(
        "src/menu.rs::screen_write_init_ctx_cb",
        hmux2::src::menu::screen_write_init_ctx_cb,
        []
    );
    record!(
        "src/mode_tree.rs::screen_write_init_ctx_cb",
        hmux2::src::mode_tree::screen_write_init_ctx_cb,
        []
    );
    record!(
        "src/popup.rs::screen_write_init_ctx_cb",
        hmux2::src::popup::screen_write_init_ctx_cb,
        []
    );
    record!(
        "src/prompt.rs::screen_write_init_ctx_cb",
        hmux2::src::prompt::screen_write_init_ctx_cb,
        []
    );
    record!(
        "src/screen_redraw.rs::screen_write_init_ctx_cb",
        hmux2::src::screen_redraw::screen_write_init_ctx_cb,
        []
    );
    record!(
        "src/screen_write.rs::screen_write_init_ctx_cb",
        hmux2::src::screen_write::screen_write_init_ctx_cb,
        []
    );
    record!(
        "src/server_fn.rs::screen_write_init_ctx_cb",
        hmux2::src::server_fn::screen_write_init_ctx_cb,
        []
    );
    record!(
        "src/status.rs::screen_write_init_ctx_cb",
        hmux2::src::status::screen_write_init_ctx_cb,
        []
    );
    record!(
        "src/window_border.rs::screen_write_init_ctx_cb",
        hmux2::src::window_border::screen_write_init_ctx_cb,
        []
    );
    record!(
        "src/window_buffer.rs::screen_write_init_ctx_cb",
        hmux2::src::window_buffer::screen_write_init_ctx_cb,
        []
    );
    record!(
        "src/window_client.rs::screen_write_init_ctx_cb",
        hmux2::src::window_client::screen_write_init_ctx_cb,
        []
    );
    record!(
        "src/window_clock.rs::screen_write_init_ctx_cb",
        hmux2::src::window_clock::screen_write_init_ctx_cb,
        []
    );
    record!(
        "src/window_copy.rs::screen_write_init_ctx_cb",
        hmux2::src::window_copy::screen_write_init_ctx_cb,
        []
    );
    record!(
        "src/window_customize.rs::screen_write_init_ctx_cb",
        hmux2::src::window_customize::screen_write_init_ctx_cb,
        []
    );
    record!(
        "src/window_panes.rs::screen_write_init_ctx_cb",
        hmux2::src::window_panes::screen_write_init_ctx_cb,
        []
    );
    record!(
        "src/window_switch.rs::screen_write_init_ctx_cb",
        hmux2::src::window_switch::screen_write_init_ctx_cb,
        []
    );
    record!(
        "src/window_tree.rs::screen_write_init_ctx_cb",
        hmux2::src::window_tree::screen_write_init_ctx_cb,
        []
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/model-screen_write.txt"));
}
