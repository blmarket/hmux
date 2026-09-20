//! Frozen measurements of every original copy; no fixture update mode.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_declarations_match() {
    let mut records = Vec::new();
    let v: ::core::ffi::c_int = hmux2::src::cmd_list_keys::KEY_BINDING_REPEAT;
    records.push(format!(
        "src/cmd_list_keys.rs::KEY_BINDING_REPEAT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::key_bindings::KEY_BINDING_REPEAT;
    records.push(format!(
        "src/key_bindings.rs::KEY_BINDING_REPEAT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::server_client::KEY_BINDING_REPEAT;
    records.push(format!(
        "src/server_client.rs::KEY_BINDING_REPEAT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_customize::KEY_BINDING_REPEAT;
    records.push(format!(
        "src/window_customize.rs::KEY_BINDING_REPEAT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::options_table::MODEKEY_EMACS;
    records.push(format!(
        "src/options_table.rs::MODEKEY_EMACS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::screen::MODEKEY_EMACS;
    records.push(format!(
        "src/screen.rs::MODEKEY_EMACS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::tmux::MODEKEY_EMACS;
    records.push(format!(
        "src/tmux.rs::MODEKEY_EMACS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_copy::MODEKEY_EMACS;
    records.push(format!(
        "src/window_copy.rs::MODEKEY_EMACS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::prompt::MODEKEY_VI;
    records.push(format!(
        "src/prompt.rs::MODEKEY_VI {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::tmux::MODEKEY_VI;
    records.push(format!(
        "src/tmux.rs::MODEKEY_VI {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_copy::MODEKEY_VI;
    records.push(format!(
        "src/window_copy.rs::MODEKEY_VI {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/subjects-key.txt"));
}
