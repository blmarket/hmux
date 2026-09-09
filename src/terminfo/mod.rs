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
    AlternateCharacterSet, BorderCharacterSet, RustAlternateCharacterSet, tty_acs_double_borders,
    tty_acs_entry, tty_acs_get, tty_acs_heavy_borders, tty_acs_needed, tty_acs_reverse_entry,
    tty_acs_reverse_get, tty_acs_rounded_borders,
};
pub use features::{
    RustTerminalFeatureSet, TerminalFeatureSet, tty_add_features, tty_default_features,
    tty_feature, tty_get_features,
};
pub use term::{RustTerminalCapabilities, TerminalCapabilities, TerminalCapabilityRef};
pub(crate) use term::{
    TerminalRef, TerminalRegistration, TtyCode, tty_term_apply_overrides, tty_term_create,
    tty_term_free, tty_term_of, tty_term_opt_mut, tty_term_read_list,
};

pub(crate) use term::tty_term_snapshots_for_client;
