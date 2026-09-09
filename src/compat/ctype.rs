#[inline]
pub(crate) fn tolower(byte: u8) -> u8 {
    unsafe { libc::tolower(byte.into()) as u8 }
}

#[inline]
pub(crate) fn toupper(byte: u8) -> u8 {
    unsafe { libc::toupper(byte.into()) as u8 }
}

/// Compares complete C strings using the current locale's byte case mapping.
pub(crate) fn cstr_eq_ignore_case(left: &core::ffi::CStr, right: &core::ffi::CStr) -> bool {
    let left = left.to_bytes();
    let right = right.to_bytes();
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(&a, &b)| tolower(a) == tolower(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_comparison_preserves_lengths_and_non_utf8_bytes() {
        for (left, right) in [(c"", c""), (c"WoRd", c"word"), (c"\xffA", c"\xffa")] {
            assert!(cstr_eq_ignore_case(left, right));
        }
        for (left, right) in [(c"", c"a"), (c"word", c"wordy"), (c"\xffa", c"\xfea")] {
            assert!(!cstr_eq_ignore_case(left, right));
            assert!(!cstr_eq_ignore_case(right, left));
        }
    }
}
