use core::ffi::CStr;

/// Parses with native `strtod` and requires it to consume the whole input.
/// Empty input keeps the native zero result, and range errors keep the native
/// result and `errno`; only an unconsumed suffix is rejected.
pub(crate) fn strtod_complete(input: &CStr) -> Option<f64> {
    let mut end = core::ptr::null_mut();
    let value = unsafe { crate::ffi::strtod(input.as_ptr(), &mut end) };
    let terminator = core::ptr::from_ref(input.to_bytes_with_nul().last()?);
    core::ptr::eq(end.cast_const().cast::<u8>(), terminator).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_native_numbers_preserve_empty_hexadecimal_and_range_results() {
        for (input, expected) in [(c"", 0.0), (c" -1.25e2", -125.0), (c"0x1.8p2", 6.0)] {
            assert_eq!(strtod_complete(input), Some(expected));
        }
        assert!(strtod_complete(c"1e9999").unwrap().is_infinite());
        assert!(strtod_complete(c"nan").unwrap().is_nan());
        for input in [c" ", c"1 ", c"1x", c"-", c"\xff"] {
            assert_eq!(strtod_complete(input), None);
        }
    }
}
