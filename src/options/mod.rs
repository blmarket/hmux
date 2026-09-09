//! Opaque option handles implement [`OptionsRef`]; [`OptionsEngine`] provides server adapters.
//!
//! Store contents, entries and values have private representations. Store
//! handles expose store operations without exposing their storage.
//!
//! ```compile_fail
//! use tmux_c2rs::options::options_get_only_ptr;
//! ```
//!
//! ```compile_fail
//! use tmux_c2rs::options::RustOptionsRef;
//! fn bypass(store: &RustOptionsRef) { let _ = store.borrow_mut(); }
//! ```
//!
//! ```compile_fail
//! use tmux_c2rs::options::RustOptions;
//! ```
//!
//! ```compile_fail
//! use tmux_c2rs::options::RustOptionsRef;
//! fn bypass(store: &RustOptionsRef) { let _ = store.as_ptr(); }
//! ```
//!
//! ```compile_fail
//! use tmux_c2rs::options::options_entry;
//! fn bypass(entry: &options_entry) { let _ = &entry.value; }
//! ```
//!
//! ```compile_fail
//! use tmux_c2rs::options::options_value;
//! fn bypass(value: &options_value) { let _ = value.string(); }
//! ```

mod engine;
mod reference;
mod store;
mod table;

pub use engine::{OptionsEngine, RustOptionsEngine};
pub use reference::OptionsRef;
pub use store::{RustOptionsRef, options_array_item_t, options_entry, options_value};

#[cfg(test)]
pub(crate) use table::{
    OPTIONS_TABLE_CHOICE, OPTIONS_TABLE_COLOUR, OPTIONS_TABLE_COMMAND, OPTIONS_TABLE_FLAG,
    OPTIONS_TABLE_IS_ARRAY, OPTIONS_TABLE_IS_HOOK, OPTIONS_TABLE_IS_STYLE, OPTIONS_TABLE_KEY,
    OPTIONS_TABLE_NUMBER, OPTIONS_TABLE_PANE, OPTIONS_TABLE_SERVER, OPTIONS_TABLE_SESSION,
    OPTIONS_TABLE_STRING, OPTIONS_TABLE_WINDOW,
};
