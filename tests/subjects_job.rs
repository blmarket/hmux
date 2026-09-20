//! Frozen measurements of every original copy; no fixture update mode.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_declarations_match() {
    let mut records = Vec::new();
    let v: ::core::ffi::c_int = hmux2::src::job::JOB_DEFAULTSHELL;
    records.push(format!(
        "src/job.rs::JOB_DEFAULTSHELL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::popup::JOB_DEFAULTSHELL;
    records.push(format!(
        "src/popup.rs::JOB_DEFAULTSHELL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::job::JOB_KEEPWRITE;
    records.push(format!(
        "src/job.rs::JOB_KEEPWRITE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::popup::JOB_KEEPWRITE;
    records.push(format!(
        "src/popup.rs::JOB_KEEPWRITE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_run_shell::JOB_NOWAIT;
    records.push(format!(
        "src/cmd_run_shell.rs::JOB_NOWAIT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::JOB_NOWAIT;
    records.push(format!(
        "src/format.rs::JOB_NOWAIT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::job::JOB_NOWAIT;
    records.push(format!(
        "src/job.rs::JOB_NOWAIT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::popup::JOB_NOWAIT;
    records.push(format!(
        "src/popup.rs::JOB_NOWAIT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_copy::JOB_NOWAIT;
    records.push(format!(
        "src/window_copy.rs::JOB_NOWAIT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::job::JOB_PTY;
    records.push(format!(
        "src/job.rs::JOB_PTY {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::popup::JOB_PTY;
    records.push(format!(
        "src/popup.rs::JOB_PTY {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_run_shell::JOB_SHOWSTDERR;
    records.push(format!(
        "src/cmd_run_shell.rs::JOB_SHOWSTDERR {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::job::JOB_SHOWSTDERR;
    records.push(format!(
        "src/job.rs::JOB_SHOWSTDERR {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/subjects-job.txt"));
}
