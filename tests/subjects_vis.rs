//! Frozen measurements of every original copy; no fixture update mode.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_declarations_match() {
    let mut records = Vec::new();
    let v: ::core::ffi::c_int = hmux2::src::arguments::VIS_CSTYLE;
    records.push(format!(
        "src/arguments.rs::VIS_CSTYLE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_capture_pane::VIS_CSTYLE;
    records.push(format!(
        "src/cmd_capture_pane.rs::VIS_CSTYLE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::compat::vis::VIS_CSTYLE;
    records.push(format!(
        "src/compat/vis.rs::VIS_CSTYLE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::hyperlinks::VIS_CSTYLE;
    records.push(format!(
        "src/hyperlinks.rs::VIS_CSTYLE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::log::VIS_CSTYLE;
    records.push(format!(
        "src/log.rs::VIS_CSTYLE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::paste::VIS_CSTYLE;
    records.push(format!(
        "src/paste.rs::VIS_CSTYLE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::server_client::VIS_CSTYLE;
    records.push(format!(
        "src/server_client.rs::VIS_CSTYLE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::tmux::VIS_CSTYLE;
    records.push(format!(
        "src/tmux.rs::VIS_CSTYLE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::tty_term::VIS_CSTYLE;
    records.push(format!(
        "src/tty_term.rs::VIS_CSTYLE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_buffer::VIS_CSTYLE;
    records.push(format!(
        "src/window_buffer.rs::VIS_CSTYLE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::arguments::VIS_DQ;
    records.push(format!(
        "src/arguments.rs::VIS_DQ {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::compat::vis::VIS_DQ;
    records.push(format!(
        "src/compat/vis.rs::VIS_DQ {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::utf8::VIS_DQ;
    records.push(format!(
        "src/utf8.rs::VIS_DQ {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::arguments::VIS_NL;
    records.push(format!(
        "src/arguments.rs::VIS_NL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_capture_pane::VIS_NL;
    records.push(format!(
        "src/cmd_capture_pane.rs::VIS_NL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::compat::vis::VIS_NL;
    records.push(format!(
        "src/compat/vis.rs::VIS_NL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::log::VIS_NL;
    records.push(format!(
        "src/log.rs::VIS_NL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::paste::VIS_NL;
    records.push(format!(
        "src/paste.rs::VIS_NL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::tmux::VIS_NL;
    records.push(format!(
        "src/tmux.rs::VIS_NL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::tty_term::VIS_NL;
    records.push(format!(
        "src/tty_term.rs::VIS_NL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_paste_buffer::VIS_NOSLASH;
    records.push(format!(
        "src/cmd_paste_buffer.rs::VIS_NOSLASH {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::compat::vis::VIS_NOSLASH;
    records.push(format!(
        "src/compat/vis.rs::VIS_NOSLASH {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::server_client::VIS_NOSLASH;
    records.push(format!(
        "src/server_client.rs::VIS_NOSLASH {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::arguments::VIS_OCTAL;
    records.push(format!(
        "src/arguments.rs::VIS_OCTAL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_capture_pane::VIS_OCTAL;
    records.push(format!(
        "src/cmd_capture_pane.rs::VIS_OCTAL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::compat::vis::VIS_OCTAL;
    records.push(format!(
        "src/compat/vis.rs::VIS_OCTAL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::hyperlinks::VIS_OCTAL;
    records.push(format!(
        "src/hyperlinks.rs::VIS_OCTAL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::log::VIS_OCTAL;
    records.push(format!(
        "src/log.rs::VIS_OCTAL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::paste::VIS_OCTAL;
    records.push(format!(
        "src/paste.rs::VIS_OCTAL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::server_client::VIS_OCTAL;
    records.push(format!(
        "src/server_client.rs::VIS_OCTAL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::tmux::VIS_OCTAL;
    records.push(format!(
        "src/tmux.rs::VIS_OCTAL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::tty_term::VIS_OCTAL;
    records.push(format!(
        "src/tty_term.rs::VIS_OCTAL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_buffer::VIS_OCTAL;
    records.push(format!(
        "src/window_buffer.rs::VIS_OCTAL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_paste_buffer::VIS_SAFE;
    records.push(format!(
        "src/cmd_paste_buffer.rs::VIS_SAFE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::compat::vis::VIS_SAFE;
    records.push(format!(
        "src/compat/vis.rs::VIS_SAFE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::arguments::VIS_TAB;
    records.push(format!(
        "src/arguments.rs::VIS_TAB {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_capture_pane::VIS_TAB;
    records.push(format!(
        "src/cmd_capture_pane.rs::VIS_TAB {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::compat::vis::VIS_TAB;
    records.push(format!(
        "src/compat/vis.rs::VIS_TAB {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::log::VIS_TAB;
    records.push(format!(
        "src/log.rs::VIS_TAB {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::paste::VIS_TAB;
    records.push(format!(
        "src/paste.rs::VIS_TAB {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::tmux::VIS_TAB;
    records.push(format!(
        "src/tmux.rs::VIS_TAB {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::tty_term::VIS_TAB;
    records.push(format!(
        "src/tty_term.rs::VIS_TAB {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_buffer::VIS_TAB;
    records.push(format!(
        "src/window_buffer.rs::VIS_TAB {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/subjects-vis.txt"));
}
