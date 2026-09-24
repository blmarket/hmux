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
        "src/arguments.rs::cmd_parse_input",
        hmux2::src::arguments::cmd_parse_input,
        [flags, file, line, item, c, fs]
    );
    record!(
        "src/cfg.rs::cmd_parse_input",
        hmux2::src::cfg::cmd_parse_input,
        [flags, file, line, item, c, fs]
    );
    record!(
        "src/client.rs::cmd_parse_input",
        hmux2::src::client::cmd_parse_input,
        [flags, file, line, item, c, fs]
    );
    record!(
        "src/cmd_bind_key.rs::cmd_parse_input",
        hmux2::src::cmd_bind_key::cmd_parse_input,
        [flags, file, line, item, c, fs]
    );
    record!(
        "src/cmd_parse.rs::cmd_parse_input",
        hmux2::src::cmd_parse::cmd_parse_input,
        [flags, file, line, item, c, fs]
    );
    record!(
        "src/control.rs::cmd_parse_input",
        hmux2::src::control::cmd_parse_input,
        [flags, file, line, item, c, fs]
    );
    record!(
        "src/hooks.rs::cmd_parse_input",
        hmux2::src::hooks::cmd_parse_input,
        [flags, file, line, item, c, fs]
    );
    record!(
        "src/key_bindings.rs::cmd_parse_input",
        hmux2::src::key_bindings::cmd_parse_input,
        [flags, file, line, item, c, fs]
    );
    record!(
        "src/menu.rs::cmd_parse_input",
        hmux2::src::menu::cmd_parse_input,
        [flags, file, line, item, c, fs]
    );
    record!(
        "src/mode_tree.rs::cmd_parse_input",
        hmux2::src::mode_tree::cmd_parse_input,
        [flags, file, line, item, c, fs]
    );
    record!(
        "src/options.rs::cmd_parse_input",
        hmux2::src::options::cmd_parse_input,
        [flags, file, line, item, c, fs]
    );
    record!(
        "src/server_client.rs::cmd_parse_input",
        hmux2::src::server_client::cmd_parse_input,
        [flags, file, line, item, c, fs]
    );
    record!(
        "src/window_customize.rs::cmd_parse_input",
        hmux2::src::window_customize::cmd_parse_input,
        [flags, file, line, item, c, fs]
    );
    record!(
        "src/window_switch.rs::cmd_parse_input",
        hmux2::src::window_switch::cmd_parse_input,
        [flags, file, line, item, c, fs]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/remaining-command.txt"));
}
