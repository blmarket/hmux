//! Original anonymous key-enum aliases; measured before migration.
use std::mem::{align_of, size_of};
#[test]
fn original_key_enum_aliases_match() {
    let mut records = Vec::new();
    macro_rules! record {
        ($label:literal, $ty:ty) => {
            // Direct assignments prove type identity without conversion casts.
            let _: Option<::core::ffi::c_ulong> = None::<$ty>;
            let _: Option<$ty> = None::<::core::ffi::c_ulong>;
            let _: Option<hmux2::src::shared::key::key_code_enum> = None::<$ty>;
            let _: Option<$ty> = None::<hmux2::src::shared::key::key_code_enum>;
            records.push(format!(
                "{} {} {}",
                $label,
                size_of::<$ty>(),
                align_of::<$ty>()
            ));
        };
    }
    record!(
        "src/cmd_bind_key.rs::C2RustUnnamed_35",
        hmux2::src::cmd_bind_key::C2RustUnnamed_35
    );
    record!(
        "src/cmd_copy_mode.rs::C2RustUnnamed_35",
        hmux2::src::cmd_copy_mode::C2RustUnnamed_35
    );
    record!(
        "src/cmd_list_keys.rs::C2RustUnnamed_35",
        hmux2::src::cmd_list_keys::C2RustUnnamed_35
    );
    record!(
        "src/cmd_queue.rs::C2RustUnnamed_36",
        hmux2::src::cmd_queue::C2RustUnnamed_36
    );
    record!(
        "src/cmd_send_keys.rs::C2RustUnnamed_35",
        hmux2::src::cmd_send_keys::C2RustUnnamed_35
    );
    record!(
        "src/cmd_unbind_key.rs::C2RustUnnamed_1",
        hmux2::src::cmd_unbind_key::C2RustUnnamed_1
    );
    record!(
        "src/input_keys.rs::C2RustUnnamed_36",
        hmux2::src::input_keys::C2RustUnnamed_36
    );
    record!(
        "src/key_string.rs::C2RustUnnamed_0",
        hmux2::src::key_string::C2RustUnnamed_0
    );
    record!(
        "src/menu.rs::C2RustUnnamed_38",
        hmux2::src::menu::C2RustUnnamed_38
    );
    record!(
        "src/mode_tree.rs::C2RustUnnamed_39",
        hmux2::src::mode_tree::C2RustUnnamed_39
    );
    record!(
        "src/options.rs::C2RustUnnamed_38",
        hmux2::src::options::C2RustUnnamed_38
    );
    record!(
        "src/options_table.rs::C2RustUnnamed",
        hmux2::src::options_table::C2RustUnnamed
    );
    record!(
        "src/popup.rs::C2RustUnnamed_38",
        hmux2::src::popup::C2RustUnnamed_38
    );
    record!(
        "src/prompt.rs::C2RustUnnamed_38",
        hmux2::src::prompt::C2RustUnnamed_38
    );
    record!(
        "src/status.rs::C2RustUnnamed_38",
        hmux2::src::status::C2RustUnnamed_38
    );
    record!(
        "src/tty_keys.rs::C2RustUnnamed_37",
        hmux2::src::tty_keys::C2RustUnnamed_37
    );
    record!(
        "src/window.rs::C2RustUnnamed_36",
        hmux2::src::window::C2RustUnnamed_36
    );
    record!(
        "src/window_buffer.rs::C2RustUnnamed_38",
        hmux2::src::window_buffer::C2RustUnnamed_38
    );
    record!(
        "src/window_client.rs::C2RustUnnamed_38",
        hmux2::src::window_client::C2RustUnnamed_38
    );
    record!(
        "src/window_customize.rs::C2RustUnnamed_38",
        hmux2::src::window_customize::C2RustUnnamed_38
    );
    record!(
        "src/window_panes.rs::C2RustUnnamed_38",
        hmux2::src::window_panes::C2RustUnnamed_38
    );
    record!(
        "src/window_switch.rs::C2RustUnnamed_38",
        hmux2::src::window_switch::C2RustUnnamed_38
    );
    record!(
        "src/window_tree.rs::C2RustUnnamed_38",
        hmux2::src::window_tree::C2RustUnnamed_38
    );
    assert_eq!(
        records.join("\n") + "\n",
        include_str!("fixtures/key-enum-layout.txt")
    );
}
