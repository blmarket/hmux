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
        "src/arguments.rs::args_command_state",
        hmux2::src::arguments::args_command_state,
        [cmdlist, cmd, pi]
    );
    record!(
        "src/cmd_command_prompt.rs::args_command_state",
        *mut hmux2::src::cmd_command_prompt::args_command_state,
        []
    );
    record!(
        "src/cmd_if_shell.rs::args_command_state",
        *mut hmux2::src::cmd_if_shell::args_command_state,
        []
    );
    record!(
        "src/cmd_run_shell.rs::args_command_state",
        *mut hmux2::src::cmd_run_shell::args_command_state,
        []
    );
    record!(
        "src/window_panes.rs::args_command_state",
        *mut hmux2::src::window_panes::args_command_state,
        []
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/remaining-arguments.txt"));
}
