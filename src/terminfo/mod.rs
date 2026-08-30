//! The terminal capability database: what terminfo says a terminal can do,
//! the feature sets layered on top of it, and the alternate character set it
//! draws lines with.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod acs;
mod features;
mod term;

pub use acs::{
    tty_acs_double_borders, tty_acs_get, tty_acs_heavy_borders, tty_acs_needed,
    tty_acs_reverse_get, tty_acs_rounded_borders,
};
pub use features::{tty_add_features, tty_apply_features, tty_default_features, tty_get_features};
pub use term::{
    TtyCode, tty_term_apply_overrides, tty_term_create, tty_term_describe, tty_term_entry,
    tty_term_flag, tty_term_free, tty_term_has, tty_term_ncodes, tty_term_number, tty_term_of,
    tty_term_opt, tty_term_opt_mut, tty_term_read_list, tty_term_string, tty_term_string_i,
    tty_term_string_ii, tty_term_string_iii, tty_term_string_s, tty_term_string_ss, tty_terms,
};

#[cfg(test)]
pub(crate) use term::{
    TTYC_AM, TTYC_AX, TTYC_BEL, TTYC_COLORS, TTYC_CSR, TTYC_CUP, TTYC_FSL, TTYC_KF1, TTYC_KMOUS,
    TTYC_MS, TTYC_SETRGBF, TTYC_SMOL, TTYC_TSL, TTYC_XT, TTYCODE_FLAG, TTYCODE_NONE,
    TTYCODE_NUMBER, TTYCODE_STRING,
};
