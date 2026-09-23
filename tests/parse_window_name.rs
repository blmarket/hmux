use hmux2::src::names::parse_window_name;
use std::ffi::{c_char, CStr};

unsafe fn parsed(input: &[u8]) -> Vec<u8> {
    let result = parse_window_name(input.as_ptr().cast::<c_char>());
    assert!(!result.is_null());
    let bytes = CStr::from_ptr(result).to_bytes().to_vec();
    libc::free(result.cast());
    bytes
}

#[test]
fn parse_window_name_handles_shell_command_forms() {
    unsafe {
        assert_eq!(parsed(b"\"exec -/usr/bin/vim -u NONE\"\0"), b"vim");
        assert_eq!(parsed(b"  --/bin/zsh -l\0"), b"zsh");
        assert_eq!(parsed(b"exec /bin/bash.sh  --login\0"), b"bash.sh");
        assert_eq!(parsed(b"command!\t\n\0"), b"command!");
    }
}

#[test]
fn parse_window_name_stops_at_first_nul_and_returns_owned_result() {
    unsafe {
        assert_eq!(parsed(b"before\0after\0"), b"before");
        assert_eq!(parsed(b"ba\xffd\0"), b"");
        assert_eq!(parsed(b"\0"), b"");
    }
}
