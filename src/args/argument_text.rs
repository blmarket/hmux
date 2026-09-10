//! Text conversion for command arguments.

use crate::args::{copy_of, no_number, number, percentage_of, share_of};
use crate::consts::{VIS_CSTYLE, VIS_DQ, VIS_NL, VIS_OCTAL, VIS_TAB};
use crate::text::{RustUtf8VisModel, Utf8VisModel};
use core::ffi::c_longlong;
use std::ffi::{CStr, CString};

/// Quoting and numeric conversion for command argument text.
pub trait ArgumentTextCodec {
    /// Quotes one argument for command text.
    fn escape(&self, value: &CStr) -> CString;

    /// Parses a number or a percentage of `current` within the allowed range.
    fn percentage(
        &self,
        value: &CStr,
        minimum: c_longlong,
        maximum: c_longlong,
        current: c_longlong,
        cause: &mut Option<CString>,
    ) -> c_longlong;
}

/// Argument text conversion implemented by hmux.
#[derive(Clone, Copy, Debug, Default)]
pub struct RustArgumentTextCodec;

/// The quoting a string needs to survive being read back as one word.
#[derive(Clone, Copy, PartialEq)]
enum Quotes {
    None,
    Single,
    Double,
}

/// The quoting a string needs: double quotes for the bytes the parser would
/// otherwise read as syntax, single quotes for the ones only they survive.
fn quotes_for(text: &[u8]) -> Quotes {
    if text.iter().any(|b| b" #';${}%".contains(b)) {
        Quotes::Double
    } else if text.iter().any(|b| b" \"".contains(b)) {
        Quotes::Single
    } else {
        Quotes::None
    }
}

impl ArgumentTextCodec for RustArgumentTextCodec {
    fn escape(&self, value: &CStr) -> CString {
        unsafe {
            let text = value.to_bytes();
            let Some(&first) = text.first() else {
                return CString::from_vec_unchecked(b"''".to_vec());
            };
            let quotes = quotes_for(text);
            if first != b' ' && text.len() == 1 && (quotes != Quotes::None || first == b'~') {
                return CString::from_vec_unchecked(vec![b'\\', first]);
            }
            let mut flags = VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL;
            if quotes == Quotes::Double {
                flags |= VIS_DQ;
            }
            let escaped = RustUtf8VisModel.encode_utf8(text, flags);
            let visible = escaped.as_bytes();
            let tilde = visible.first() == Some(&b'~');
            let mut result: Vec<u8> = Vec::new();
            match quotes {
                Quotes::Single => {
                    result.push(b'\'');
                    result.extend_from_slice(visible);
                    result.push(b'\'');
                }
                Quotes::Double => {
                    result.push(b'"');
                    if tilde {
                        result.push(b'\\');
                    }
                    result.extend_from_slice(visible);
                    result.push(b'"');
                }
                Quotes::None => {
                    if tilde {
                        result.push(b'\\');
                    }
                    result.extend_from_slice(visible);
                }
            }
            CString::from_vec_unchecked(result)
        }
    }

    fn percentage(
        &self,
        value: &CStr,
        minimum: c_longlong,
        maximum: c_longlong,
        current: c_longlong,
        cause: &mut Option<CString>,
    ) -> c_longlong {
        {
            let text = value.to_bytes();
            if text.is_empty() {
                return no_number(cause, c"empty");
            }
            let Some(percent) = percentage_of(text) else {
                return match number(value, minimum, maximum) {
                    Ok(ll) => {
                        *cause = None;
                        ll
                    }
                    Err(errstr) => no_number(cause, errstr),
                };
            };
            let copy = copy_of(percent);
            let result = number(&copy, 0, 100);
            match result {
                Ok(percent) => share_of(percent, minimum, maximum, current, cause),
                Err(errstr) => no_number(cause, errstr),
            }
        }
    }
}
