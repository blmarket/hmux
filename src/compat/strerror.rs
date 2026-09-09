use core::ffi::c_int;
use std::ffi::{CStr, CString};

pub(crate) fn error_message(error: c_int) -> CString {
    let message = unsafe { libc::strerror(error) };
    assert!(!message.is_null(), "libc supplies an error message");
    unsafe { CStr::from_ptr(message).to_owned() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_messages_match_libc_for_known_and_unknown_errors() {
        for error in [
            0,
            libc::ENOENT,
            libc::EACCES,
            libc::EINTR,
            c_int::MIN,
            c_int::MAX,
        ] {
            let expected = unsafe { CStr::from_ptr(libc::strerror(error)).to_owned() };
            assert_eq!(error_message(error), expected);
        }
    }

    #[test]
    fn later_lookups_do_not_replace_a_retained_unknown_error() {
        let retained = error_message(c_int::MIN);
        let expected = retained.clone();
        let next = error_message(c_int::MAX);
        assert_ne!(retained, next);
        assert_eq!(retained, expected);
    }
}
