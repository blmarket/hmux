//! Shared option-store operations for Rust and native tmux handles.

use crate::cmd::CmdListRef;
use super::{RustOptionsRef, store};
use crate::fmt_engine::FmtArg;
use crate::types::*;
use core::ffi::{CStr, c_int, c_longlong};
use std::ffi::CString;
use std::rc::Rc;

/// An opaque shared option store. Cloning retains it; equality compares identity.
///
/// Parent links retain ancestors and reject cycles. Storage remains private.
///
/// # Safety
/// Unsafe operations require live entry/value handles without conflicting borrows.
/// Operations that use server adapters additionally require initialized server state and live targets.
pub trait OptionsRef: Clone + Eq + std::fmt::Debug {
    /// An opaque entry belonging to this store implementation.
    type Entry;
    /// Clones the parent handle, or returns none for a root.
    fn parent(&self) -> Option<Self>;
    /// Retains a new fallback parent without changing local entries.
    /// Panics if the new link would create a parent cycle.
    fn set_parent(&self, parent: Option<&Self>);
    /// Borrows a named entry for a callback, searching ancestors unless `local` is true.
    /// The entry's owning store remains alive throughout the callback, even after reparenting.
    ///
    /// ```compile_fail
    /// use tmux_c2rs::OptionsRef;
    /// fn escape<R: OptionsRef>(store: &R) -> Option<&R::Entry> {
    ///     store.with_entry(c"status", false, |entry| entry)
    /// }
    /// ```
    fn with_entry<R>(
        &self,
        name: &CStr,
        local: bool,
        read: impl FnOnce(Option<&Self::Entry>) -> R,
    ) -> R;
    /// Borrows a named entry exclusively for a callback, retaining its owning store.
    /// Panics if that store already has a conflicting scoped borrow.
    /// Replacing or removing the entry must happen after the callback returns.
    ///
    /// ```compile_fail
    /// use tmux_c2rs::OptionsRef;
    /// fn escape<R: OptionsRef>(store: &R) -> Option<&mut R::Entry> {
    ///     store.with_entry_mut(c"status", false, |entry| entry)
    /// }
    /// ```
    fn with_entry_mut<R>(
        &self,
        name: &CStr,
        local: bool,
        write: impl FnOnce(Option<&mut Self::Entry>) -> R,
    ) -> R;
    /// Copies local entry names in bytewise order, independently of later mutations.
    fn local_names(&self) -> Vec<CString>;
    /// Replaces the named local entry with an uninitialized entry of this definition.
    fn insert_empty(&self, oe: &'static options_table_entry_t);
    /// Replaces the named local entry with its definition’s scalar or array default.
    ///
    /// # Safety
    /// The handle and server-state requirements of this trait apply.
    unsafe fn set_default(&self, oe: &'static options_table_entry_t);
    /// Copies the `codepoint-widths` strings in array order for the Unicode-width adapter.
    fn codepoint_widths(&self) -> Vec<CString>;
    /// Copies valid `pane-colours` indices into a palette, or returns none for an empty array.
    fn pane_colours(&self) -> Option<[c_int; 256]>;
    /// Updates a palette’s defaults from `pane-colours` without changing explicit colours.
    fn load_pane_colours(&self, p: Option<&mut colour_palette>);
    /// Retains an immutable inherited string snapshot; missing or nonstring options are fatal errors.
    /// The snapshot remains valid after replacement, removal, reparenting, or dropping the store.
    fn string_ref(&self, name: &CStr) -> Rc<CStr>;
    /// Reads an inherited numeric value; missing or nonnumeric options are fatal errors.
    fn number(&self, name: &CStr) -> c_longlong;
    /// Clones an inherited command handle; missing or noncommand options are fatal errors.
    fn command(&self, name: &CStr) -> Option<CmdListRef>;
    /// Formats and sets a string, optionally appending with its definition’s separator.
    ///
    /// # Safety
    /// The handle and server-state requirements of this trait apply.
    unsafe fn set_string(&self, name: &CStr, append: c_int, fmt: &CStr, args: &[FmtArg]);
    /// Sets a local numeric value, materializing its inherited definition when needed.
    ///
    /// # Safety
    /// The handle and server-state requirements of this trait apply.
    unsafe fn set_number(&self, name: &CStr, value: c_longlong);
    /// Sets a local command handle, materializing its inherited definition when needed.
    ///
    /// # Safety
    /// The handle and server-state requirements of this trait apply.
    unsafe fn set_command(&self, name: &CStr, value: Option<CmdListRef>);
    /// Copies the cached style, expanding formats when needed, or returns none on missing or invalid input.
    ///
    /// # Safety
    /// The handle and server-state requirements of this trait apply.
    unsafe fn style_value(&self, name: &CStr, ft: Option<&mut format_tree>) -> Option<style>;
    /// Validates and sets text according to its definition; flags and choices may toggle with no value.
    ///
    /// # Safety
    /// The handle and server-state requirements of this trait apply.
    unsafe fn set_from_string(
        &self,
        oe: Option<&options_table_entry_t>,
        name: &CStr,
        value: Option<&CStr>,
        append: c_int,
        cause: &mut Option<CString>,
    ) -> c_int;
}

impl OptionsRef for RustOptionsRef {
    type Entry = options_entry;
    fn parent(&self) -> Option<RustOptionsRef> {
        store::options_get_parent(self)
    }
    fn set_parent(&self, parent: Option<&RustOptionsRef>) {
        store::options_set_parent(self, parent)
    }
    fn with_entry<R>(
        &self,
        name: &CStr,
        local: bool,
        read: impl FnOnce(Option<&options_entry>) -> R,
    ) -> R {
        store::with_entry(self, name, local, read)
    }
    fn local_names(&self) -> Vec<CString> {
        store::local_names(self)
    }
    fn with_entry_mut<R>(
        &self,
        name: &CStr,
        local: bool,
        write: impl FnOnce(Option<&mut options_entry>) -> R,
    ) -> R {
        store::with_entry_mut(self, name, local, write)
    }
    fn insert_empty(&self, oe: &'static options_table_entry_t) {
        {
            store::options_empty(self, oe);
        }
    }
    unsafe fn set_default(&self, oe: &'static options_table_entry_t) {
        unsafe {
            store::options_default(self, oe);
        }
    }
    fn codepoint_widths(&self) -> Vec<CString> {
        store::options_codepoint_widths(self)
    }
    fn pane_colours(&self) -> Option<[c_int; 256]> {
        store::options_pane_colours(self)
    }
    fn load_pane_colours(&self, p: Option<&mut colour_palette>) {
        store::options_load_pane_colours(self, p)
    }
    fn string_ref(&self, name: &CStr) -> Rc<CStr> {
        store::options_string_ref(self, name)
    }
    fn number(&self, name: &CStr) -> c_longlong {
        store::options_get_number(self, name)
    }
    fn command(&self, name: &CStr) -> Option<CmdListRef> {
        store::options_get_command(self, name)
    }
    unsafe fn set_string(&self, name: &CStr, append: c_int, fmt: &CStr, args: &[FmtArg]) {
        unsafe {
            store::options_set_string(self, name, append, fmt, args);
        }
    }
    unsafe fn set_number(&self, name: &CStr, value: c_longlong) {
        unsafe {
            store::options_set_number(self, name, value);
        }
    }
    unsafe fn set_command(&self, name: &CStr, value: Option<CmdListRef>) {
        unsafe {
            store::options_set_command(self, name, value);
        }
    }
    unsafe fn style_value(&self, name: &CStr, ft: Option<&mut format_tree>) -> Option<style> {
        unsafe { store::options_string_to_style(self, name, ft) }
    }
    unsafe fn set_from_string(
        &self,
        oe: Option<&options_table_entry_t>,
        name: &CStr,
        value: Option<&CStr>,
        append: c_int,
        cause: &mut Option<CString>,
    ) -> c_int {
        unsafe { store::options_from_string(self, oe, name, value, append, cause) }
    }
}
