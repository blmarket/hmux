//! Rust formatting into owned bytes
//!
//! ```
//! use hmux2::src::format::bytes::format_bytes;
//! use std::ffi::CString;
//! use std::io::Write;
//!
//! let name = CString::new(vec![0xff, b'x']).unwrap();
//! let mut bytes = format_bytes(format_args!("client {}: ", 7));
//! bytes.extend_from_slice(name.as_bytes());
//! write!(&mut bytes, " ({:04x})", 42).unwrap();
//! let value = CString::new(bytes).unwrap();
//! assert_eq!(value.as_bytes(), b"client 7: \xffx (002a)");
//! ```

use std::ffi::{CString, NulError};
use std::fmt;
use std::io::Write;

/// Format into bytes. Embedded NUL bytes are preserved.
pub fn format_bytes(args: fmt::Arguments<'_>) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes
        .write_fmt(args)
        .expect("formatting into a byte vector failed");
    bytes
}

/// Format into a NUL-terminated string, rejecting embedded NUL bytes.
pub fn format_cstring(args: fmt::Arguments<'_>) -> Result<CString, NulError> {
    CString::new(format_bytes(args))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_output_preserves_nul_while_cstring_reports_it() {
        assert_eq!(format_bytes(format_args!("a\0{}", 7)), b"a\x007");
        let error = format_cstring(format_args!("a\0{}", 7)).unwrap_err();
        assert_eq!(error.nul_position(), 1);
        assert_eq!(error.into_vec(), b"a\x007");
    }
}
