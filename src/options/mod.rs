//! The option store: option sets and the scopes they nest in, the arrays and
//! strings they hold, and the table that says what every option is.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod store;
mod table;

pub use store::{
    options, options_array_assign, options_array_clear, options_array_first, options_array_get,
    options_array_item_index, options_array_item_t, options_array_item_value, options_array_next,
    options_array_set, options_codepoint_widths, options_create_boxed, options_default,
    options_default_to_string, options_empty, options_entry, options_find_choice, options_first,
    options_free, options_from_string, options_get_number, options_get_only_ptr,
    options_get_parent, options_get_ptr, options_get_string, options_is_array, options_is_string,
    options_load_pane_colours, options_match, options_name, options_next, options_owner,
    options_parse_get, options_ptr, options_push_changes, options_remove_or_default,
    options_scope_from_flags, options_scope_from_name, options_set_number, options_set_parent,
    options_set_string, options_string_to_style, options_table_entry, options_to_string,
};
pub(crate) use store::{options_array_item_command, options_get_command};
pub use table::options_table;

#[cfg(test)]
pub(crate) use store::options_set_command;
#[cfg(test)]
pub(crate) use table::{
    OPTIONS_TABLE_CHOICE, OPTIONS_TABLE_COLOUR, OPTIONS_TABLE_COMMAND, OPTIONS_TABLE_FLAG,
    OPTIONS_TABLE_IS_ARRAY, OPTIONS_TABLE_IS_HOOK, OPTIONS_TABLE_IS_STYLE, OPTIONS_TABLE_KEY,
    OPTIONS_TABLE_NUMBER, OPTIONS_TABLE_PANE, OPTIONS_TABLE_SERVER, OPTIONS_TABLE_SESSION,
    OPTIONS_TABLE_STRING, OPTIONS_TABLE_WINDOW, options_other_names,
};
