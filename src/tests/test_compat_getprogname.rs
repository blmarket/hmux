use super::*;

#[test]
fn returns_the_invocation_short_name() {
    let p = getprogname();
    let invocation = unsafe { program_invocation_short_name } as *const _;
    assert_eq!(p.as_ptr(), invocation);
}

#[test]
fn matches_the_executable_file_name() {
    let name = getprogname().to_str().expect("program name is valid UTF-8");
    let exe = ::std::env::current_exe().expect("current_exe");
    let base = exe
        .file_name()
        .and_then(|s| s.to_str())
        .expect("executable file name");
    assert_eq!(name, base);
}

#[test]
fn is_stable_across_calls() {
    assert_eq!(getprogname().as_ptr(), getprogname().as_ptr());
}
