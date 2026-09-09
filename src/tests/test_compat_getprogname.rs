use super::*;

#[test]
fn returns_the_invocation_short_name() {
    let p = getprogname();
    let invocation = unsafe { CStr::from_ptr(program_invocation_short_name) };
    assert_eq!(p.as_c_str(), invocation);
}

#[test]
fn matches_the_executable_file_name() {
    let program = getprogname();
    let name = program.to_str().expect("program name is valid UTF-8");
    let exe = std::env::current_exe().expect("current_exe");
    let base = exe
        .file_name()
        .and_then(|s| s.to_str())
        .expect("executable file name");
    assert_eq!(name, base);
}

#[test]
fn is_stable_across_calls() {
    assert_eq!(getprogname(), getprogname());
}
