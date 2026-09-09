//! The complete boundary for option storage and its live-server adapters.

use crate::cmd::CmdListRef;
use super::{RustOptionsRef, store};
use crate::types::*;
use core::ffi::{CStr, c_int, c_longlong};
use std::ffi::CString;
use std::rc::Rc;

/// Factories, entry/value operations, definitions and live-server adapters.
/// Store operations are provided by [`OptionsRef`].
///
/// Local lookup resolves aliases within one set; inherited lookup repeats that
/// operation up the parent chain. Entries and array values are borrowed within
/// store callbacks. Strings and command lists can be retained as independent
/// snapshots. Removing or replacing entries happens after their borrows end.
///
/// # Safety
///
/// Unsafe operations require live handles from this engine, valid parent and
/// owner links, and exclusive access when mutating. Children retain their parent
/// stores; parent links must be acyclic. Server adapters additionally require
/// initialized server state and live target objects.
pub trait OptionsEngine {
    /// An opaque shared store handle. Clones retain the store; equality tests identity.
    type Store: OptionsRef<Entry = Self::Entry>;
    /// An opaque entry belonging to one store.
    type Entry;
    /// An opaque indexed array element.
    type ArrayItem;
    /// An opaque string, number or command value.
    type Value;
    /// Creates an empty store retaining the supplied parent, if any.
    fn create(&self, parent: Option<&Self::Store>) -> Self::Store;
    /// Releases an owner; the last owner destroys the store and its handles.
    fn destroy(&self, oo: Self::Store);
    /// Formats the definition’s default value using tmux’s canonical spelling.
    fn default_text(&self, oe: &options_table_entry_t) -> CString;
    /// Borrows the name under which this entry is stored.
    ///
    /// # Safety
    /// The engine’s handle and server-state requirements apply.
    unsafe fn name<'a>(&self, o: &'a Self::Entry) -> &'a CStr;
    /// Returns the store holding this entry, including an inherited entry’s original owner.
    ///
    /// # Safety
    /// The engine’s handle and server-state requirements apply.
    unsafe fn owner(&self, o: &Self::Entry) -> Self::Store;
    /// Returns the built-in definition; absent entries and user options return none.
    fn definition(&self, o: Option<&Self::Entry>) -> Option<&'static options_table_entry_t>;
    /// Removes all array elements; a scalar entry is unchanged.
    ///
    /// # Safety
    /// The engine’s handle and server-state requirements apply.
    unsafe fn array_clear(&self, o: &mut Self::Entry);
    /// Borrows the value at an index, or returns none for a missing index or scalar entry.
    ///
    /// # Safety
    /// The engine’s handle and server-state requirements apply.
    unsafe fn array_get<'a>(&self, o: &'a Self::Entry, idx: u_int) -> Option<&'a Self::Value>;
    /// Sets or appends at an index, or removes it for `None`; returns 0 or -1 with a cause.
    ///
    /// # Safety
    /// The engine’s handle and server-state requirements apply.
    unsafe fn array_set(
        &self,
        o: &mut Self::Entry,
        idx: u_int,
        value: Option<&CStr>,
        append: c_int,
        cause: &mut Option<CString>,
    ) -> c_int;
    /// Splits text using the definition’s separators and fills vacant indices; returns 0 or -1 with a cause.
    ///
    /// # Safety
    /// The engine’s handle and server-state requirements apply.
    unsafe fn array_assign(
        &self,
        o: &mut Self::Entry,
        s: Option<&CStr>,
        cause: &mut Option<CString>,
    ) -> c_int;
    /// Copies the occupied array indices in ascending order; scalar entries return an empty list.
    fn array_indices(&self, o: &Self::Entry) -> Vec<u_int>;
    /// Returns the element’s array index.
    ///
    /// # Safety
    /// The engine’s handle and server-state requirements apply.
    unsafe fn array_index(&self, a: &Self::ArrayItem) -> u_int;
    /// Borrows the element’s opaque value for the duration of its borrow.
    fn array_value<'a>(&self, a: &'a Self::ArrayItem) -> &'a Self::Value;
    /// Clones the array element’s command handle, if it holds one.
    ///
    /// # Safety
    /// The engine’s handle and server-state requirements apply.
    unsafe fn array_command(&self, a: &Self::ArrayItem) -> Option<CmdListRef>;
    /// Returns 1 for an array entry and 0 for a scalar.
    ///
    /// # Safety
    /// The engine’s handle and server-state requirements apply.
    unsafe fn is_array(&self, o: &Self::Entry) -> c_int;
    /// Returns 1 for a string entry, including user options, and 0 otherwise.
    ///
    /// # Safety
    /// The engine’s handle and server-state requirements apply.
    unsafe fn is_string(&self, o: &Self::Entry) -> c_int;
    /// Formats a scalar or array index; `numeric` requests numeric flag output.
    ///
    /// # Safety
    /// The engine’s handle and server-state requirements apply.
    unsafe fn display(&self, o: &Self::Entry, idx: c_int, numeric: c_int) -> CString;
    /// Parses a complete name and optional nonnegative index; a missing index becomes -1.
    fn parse(&self, name: &CStr, idx: &mut c_int) -> Option<CString>;
    /// Resolves an exact name, alias or unambiguous prefix and index; ambiguity sets `ambiguous`.
    fn match_name(&self, s: &CStr, idx: &mut c_int, ambiguous: &mut c_int) -> Option<CString>;
    /// Selects a target store using the option definition, command flags and target state; reports errors in `cause`.
    ///
    /// # Safety
    /// The engine’s handle and server-state requirements apply.
    unsafe fn scope_from_name(
        &self,
        args: &args,
        window: c_int,
        name: &CStr,
        fs: &cmd_find_state,
        oo: &mut Option<Self::Store>,
        cause: &mut Option<CString>,
    ) -> c_int;
    /// Selects a target store using command flags and target state; reports errors in `cause`.
    ///
    /// # Safety
    /// The engine’s handle and server-state requirements apply.
    unsafe fn scope_from_flags(
        &self,
        args: &args,
        window: c_int,
        fs: &cmd_find_state,
        oo: &mut Option<Self::Store>,
        cause: &mut Option<CString>,
    ) -> c_int;
    /// Returns the matching choice index, or -1 with a cause for an unknown choice.
    ///
    /// # Safety
    /// The engine’s handle and server-state requirements apply.
    unsafe fn find_choice(
        &self,
        oe: &options_table_entry_t,
        value: &CStr,
        cause: &mut Option<CString>,
    ) -> c_int;
    /// Applies the live-server effects of a changed option, including redraw and cache updates.
    ///
    /// # Safety
    /// The engine’s handle and server-state requirements apply.
    unsafe fn push_changes(&self, name: &CStr);
    /// Removes a local index or entry by name; a built-in global entry resets to its default.
    /// Missing local entries are unchanged. Conflicting scoped borrows panic.
    ///
    /// # Safety
    /// The engine’s handle and server-state requirements apply.
    unsafe fn remove_or_default(
        &self,
        store: &Self::Store,
        name: &CStr,
        idx: c_int,
        cause: &mut Option<CString>,
    ) -> c_int;
    /// Returns the immutable built-in definitions in table order.
    fn table(&self) -> &'static [options_table_entry_t];
    /// Returns the immutable accepted alias mappings.
    fn aliases(&self) -> &'static [options_name_map];
    /// Borrows the scalar value of an entry for the duration of its borrow.
    fn value<'a>(&self, entry: &'a Self::Entry) -> &'a Self::Value;
    /// Returns the stored number, or zero for a nonnumeric value.
    fn value_number(&self, value: &Self::Value) -> c_longlong;
    /// Borrows a string value; panics when the value is not a string.
    fn value_string<'a>(&self, value: &'a Self::Value) -> &'a CStr;
    /// Retains an immutable string snapshot independently of later entry or array mutations.
    fn value_string_ref(&self, value: &Self::Value) -> Rc<CStr>;
    /// Clones the command handle, or returns none for a noncommand value.
    fn value_command(&self, value: &Self::Value) -> Option<CmdListRef>;
}

/// The option engine backed by the translated Rust implementation.
#[derive(Clone, Copy, Debug, Default)]
pub struct RustOptionsEngine;

impl OptionsEngine for RustOptionsEngine {
    type Store = RustOptionsRef;
    type Entry = options_entry;
    type ArrayItem = options_array_item_t;
    type Value = options_value;
    fn create(&self, parent: Option<&RustOptionsRef>) -> RustOptionsRef {
        store::options_create(parent)
    }
    fn destroy(&self, oo: RustOptionsRef) {
        drop(oo)
    }

    fn default_text(&self, oe: &options_table_entry_t) -> CString {
        store::options_default_to_string(oe)
    }
    unsafe fn name<'a>(&self, o: &'a options_entry) -> &'a CStr {
        unsafe { store::options_name(o) }
    }
    unsafe fn owner(&self, o: &options_entry) -> RustOptionsRef {
        unsafe { store::options_owner(o) }
    }
    fn definition(&self, o: Option<&options_entry>) -> Option<&'static options_table_entry_t> {
        store::options_table_entry(o)
    }
    unsafe fn array_clear(&self, o: &mut options_entry) {
        unsafe { store::options_array_clear(o) }
    }
    unsafe fn array_get<'a>(&self, o: &'a options_entry, idx: u_int) -> Option<&'a options_value> {
        store::array_value_at(o, idx)
    }
    unsafe fn array_set(
        &self,
        o: &mut options_entry,
        idx: u_int,
        value: Option<&CStr>,
        append: c_int,
        cause: &mut Option<CString>,
    ) -> c_int {
        unsafe { store::options_array_set(o, idx, value, append, cause) }
    }
    unsafe fn array_assign(
        &self,
        o: &mut options_entry,
        s: Option<&CStr>,
        cause: &mut Option<CString>,
    ) -> c_int {
        unsafe { store::options_array_assign(o, s, cause) }
    }
    fn array_indices(&self, o: &options_entry) -> Vec<u_int> {
        store::array_indices(o)
    }
    unsafe fn array_index(&self, a: &options_array_item_t) -> u_int {
        unsafe { store::options_array_item_index(a) }
    }
    fn array_value<'a>(&self, a: &'a options_array_item_t) -> &'a options_value {
        store::array_item_value(a)
    }
    unsafe fn array_command(&self, a: &options_array_item_t) -> Option<CmdListRef> {
        unsafe { store::options_array_item_command(a) }
    }

    unsafe fn is_array(&self, o: &options_entry) -> c_int {
        unsafe { store::options_is_array(o) }
    }
    unsafe fn is_string(&self, o: &options_entry) -> c_int {
        unsafe { store::options_is_string(o) }
    }
    unsafe fn display(&self, o: &options_entry, idx: c_int, numeric: c_int) -> CString {
        unsafe { store::options_to_string(o, idx, numeric) }
    }
    fn parse(&self, name: &CStr, idx: &mut c_int) -> Option<CString> {
        store::options_parse(name, idx)
    }

    fn match_name(&self, s: &CStr, idx: &mut c_int, ambiguous: &mut c_int) -> Option<CString> {
        store::options_match(s, idx, ambiguous)
    }

    unsafe fn scope_from_name(
        &self,
        args: &args,
        window: c_int,
        name: &CStr,
        fs: &cmd_find_state,
        oo: &mut Option<RustOptionsRef>,
        cause: &mut Option<CString>,
    ) -> c_int {
        unsafe {
            store::options_scope_from_name(
                crate::RustArguments::from_ref(args), window, name, fs, oo, cause,
            )
        }
    }
    unsafe fn scope_from_flags(
        &self,
        args: &args,
        window: c_int,
        fs: &cmd_find_state,
        oo: &mut Option<RustOptionsRef>,
        cause: &mut Option<CString>,
    ) -> c_int {
        unsafe {
            store::options_scope_from_flags(
                crate::RustArguments::from_ref(args), window, fs, oo, cause,
            )
        }
    }

    unsafe fn find_choice(
        &self,
        oe: &options_table_entry_t,
        value: &CStr,
        cause: &mut Option<CString>,
    ) -> c_int {
        unsafe { store::options_find_choice(oe, value, cause) }
    }

    unsafe fn push_changes(&self, name: &CStr) {
        unsafe { store::options_push_changes(name) }
    }
    unsafe fn remove_or_default(
        &self,
        store: &RustOptionsRef,
        name: &CStr,
        idx: c_int,
        cause: &mut Option<CString>,
    ) -> c_int {
        unsafe { store::options_remove_or_default(store, name, idx, cause) }
    }
    fn table(&self) -> &'static [options_table_entry_t] {
        &super::table::options_table
    }
    fn aliases(&self) -> &'static [options_name_map] {
        &super::table::options_other_names
    }
    fn value<'a>(&self, entry: &'a options_entry) -> &'a options_value {
        store::entry_value(entry)
    }
    fn value_number(&self, value: &options_value) -> c_longlong {
        store::value_number(value)
    }
    fn value_string<'a>(&self, value: &'a options_value) -> &'a CStr {
        store::value_string(value)
    }
    fn value_string_ref(&self, value: &options_value) -> Rc<CStr> {
        store::value_string_ref(value)
    }
    fn value_command(&self, value: &options_value) -> Option<CmdListRef> {
        store::value_command(value)
    }
}
