//! The text codec: UTF-8 characters and their widths, the combining rules a
//! terminal joins them by, and the names keys are written under.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod combined;
mod key_string;
mod utf8;

pub use combined::{
    HANGULJAMO_STATE_CHOSEONG, HANGULJAMO_STATE_NOT_COMPOSABLE, HANGULJAMO_STATE_NOT_HANGULJAMO,
    hanguljamo_check_state, hanguljamo_state, utf8_has_zwj, utf8_is_hangul_filler, utf8_is_vs,
    utf8_is_zwj, utf8_should_combine,
};
pub use key_string::{
    KEYC_UNKNOWN, key_code, key_code_type, key_string_lookup_key, key_string_lookup_string,
};
pub use utf8::{
    utf8_append, utf8_build_one, utf8_char, utf8_copy, utf8_cstrhas, utf8_cstrwidth, utf8_data,
    utf8_from_data, utf8_fromcstr, utf8_fromwc, utf8_isvalid, utf8_open, utf8_padcstr,
    utf8_rpadcstr, utf8_sanitize, utf8_set, utf8_state, utf8_stravis, utf8_stravisx, utf8_strvis,
    utf8_to_data, utf8_towc, utf8_update_width_cache, utf8_vec_fromcstr, utf8_vec_strlen,
    utf8_vec_strwidth, utf8_vec_tocstr,
};

#[cfg(test)]
pub(crate) use key_string::KEYC_CTRL;
