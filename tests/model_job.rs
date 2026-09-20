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
        "src/cmd_if_shell.rs::job",
        *mut hmux2::src::cmd_if_shell::job,
        []
    );
    record!(
        "src/cmd_run_shell.rs::job",
        *mut hmux2::src::cmd_run_shell::job,
        []
    );
    record!("src/format.rs::job", *mut hmux2::src::format::job, []);
    record!(
        "src/job.rs::job",
        hmux2::src::job::job,
        [
            state, flags, cmd, pid, tty, status, fd, event, updatecb, completecb, freecb, data,
            entry
        ]
    );
    record!("src/popup.rs::job", *mut hmux2::src::popup::job, []);
    record!(
        "src/window_copy.rs::job",
        *mut hmux2::src::window_copy::job,
        []
    );
    record!(
        "src/cmd_if_shell.rs::job_complete_cb",
        hmux2::src::cmd_if_shell::job_complete_cb,
        []
    );
    record!(
        "src/cmd_run_shell.rs::job_complete_cb",
        hmux2::src::cmd_run_shell::job_complete_cb,
        []
    );
    record!(
        "src/format.rs::job_complete_cb",
        hmux2::src::format::job_complete_cb,
        []
    );
    record!(
        "src/job.rs::job_complete_cb",
        hmux2::src::job::job_complete_cb,
        []
    );
    record!(
        "src/popup.rs::job_complete_cb",
        hmux2::src::popup::job_complete_cb,
        []
    );
    record!(
        "src/window_copy.rs::job_complete_cb",
        hmux2::src::window_copy::job_complete_cb,
        []
    );
    record!(
        "src/job.rs::C2RustUnnamed_36",
        hmux2::src::job::job_entry,
        [le_next, le_prev]
    );
    record!(
        "src/cmd_if_shell.rs::job_free_cb",
        hmux2::src::cmd_if_shell::job_free_cb,
        []
    );
    record!(
        "src/cmd_run_shell.rs::job_free_cb",
        hmux2::src::cmd_run_shell::job_free_cb,
        []
    );
    record!(
        "src/format.rs::job_free_cb",
        hmux2::src::format::job_free_cb,
        []
    );
    record!("src/job.rs::job_free_cb", hmux2::src::job::job_free_cb, []);
    record!(
        "src/popup.rs::job_free_cb",
        hmux2::src::popup::job_free_cb,
        []
    );
    record!(
        "src/window_copy.rs::job_free_cb",
        hmux2::src::window_copy::job_free_cb,
        []
    );
    record!(
        "src/job.rs::C2RustUnnamed_37",
        hmux2::src::job::job_state,
        []
    );
    record!(
        "src/cmd_if_shell.rs::job_update_cb",
        hmux2::src::cmd_if_shell::job_update_cb,
        []
    );
    record!(
        "src/cmd_run_shell.rs::job_update_cb",
        hmux2::src::cmd_run_shell::job_update_cb,
        []
    );
    record!(
        "src/format.rs::job_update_cb",
        hmux2::src::format::job_update_cb,
        []
    );
    record!(
        "src/job.rs::job_update_cb",
        hmux2::src::job::job_update_cb,
        []
    );
    record!(
        "src/popup.rs::job_update_cb",
        hmux2::src::popup::job_update_cb,
        []
    );
    record!(
        "src/window_copy.rs::job_update_cb",
        hmux2::src::window_copy::job_update_cb,
        []
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/model-job.txt"));
}
