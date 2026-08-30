use super::*;
use ::core::ffi::CStr;

#[test]
fn returns_the_invocation_short_name() {
    let p = getprogname();
    assert!(!p.is_null());
    assert_eq!(p, unsafe { program_invocation_short_name } as *const _);
}

#[test]
fn matches_the_executable_file_name() {
    let name = unsafe { CStr::from_ptr(getprogname()) }
        .to_str()
        .expect("program name is valid UTF-8");
    let exe = ::std::env::current_exe().expect("current_exe");
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
