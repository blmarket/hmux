use core::ffi::c_int;
use std::ffi::{CStr, CString};

pub(crate) fn signal_description(signal: c_int) -> Option<CString> {
    let description = unsafe { libc::strsignal(signal) };
    (!description.is_null()).then(|| unsafe { CStr::from_ptr(description).to_owned() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_descriptions_match_libc_for_known_and_unknown_signals() {
        for signal in [libc::SIGTERM, libc::SIGCHLD, 0, c_int::MIN, c_int::MAX] {
            let description = unsafe { libc::strsignal(signal) };
            let expected =
                (!description.is_null()).then(|| unsafe { CStr::from_ptr(description).to_owned() });
            assert_eq!(signal_description(signal), expected);
        }
    }

    #[test]
    fn later_lookups_do_not_replace_a_retained_unknown_signal() {
        let retained = signal_description(c_int::MIN);
        let expected = retained.clone();
        let next = signal_description(c_int::MAX);
        assert_ne!(retained, next);
        assert_eq!(retained, expected);
    }
}
