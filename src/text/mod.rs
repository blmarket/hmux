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
    RustUtf8Compositor, Utf8Compositor, hanguljamo_check_state, hanguljamo_state, utf8_has_zwj,
    utf8_is_hangul_filler, utf8_is_vs, utf8_is_zwj, utf8_should_combine,
};
pub use key_string::{
    KEYC_UNKNOWN, KeyStringCodec, RustKeyStringCodec, key_code, key_code_type,
    key_string_table_entry,
};
pub use utf8::{
    RustUtf8VisModel, Utf8VisModel, utf8_append, utf8_build_one, utf8_char, utf8_copy,
    utf8_cstrhas, utf8_data, utf8_from_data, utf8_fromcstr, utf8_fromwc, utf8_open, utf8_set,
    utf8_state, utf8_to_data, utf8_towc, utf8_update_width_cache, utf8_vec_strlen,
    utf8_vec_strwidth, utf8_vec_tocstr,
};
