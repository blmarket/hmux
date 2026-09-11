//! Stable access to a parsed command's arguments.

use crate::cmd::cmdq_item;
use crate::types::{ArgsValue, u_char, u_int};
use core::ffi::{CStr, c_int, c_longlong};
use std::ffi::CString;

/// The observable flag and positional-value state of parsed arguments.
pub trait Arguments {
    /// Returns how many times `flag` was given.
    fn argument_flag_count(&self, flag: u_char) -> c_int;

    /// Adds one occurrence of `flag`, optionally carrying a value.
    fn set_argument_flag(&mut self, flag: u_char, value: Option<ArgsValue>, flags: c_int);

    /// Returns the last string value of `flag` when it has one.
    fn argument_flag_string(&self, flag: u_char) -> Option<&CStr>;

    /// Returns the flags in flag order.
    fn argument_flags(&self) -> Vec<u_char>;

    /// Returns the number of positional values.
    fn argument_count(&self) -> u_int;

    /// Returns all positional values in command-line order.
    fn argument_values(&self) -> &[ArgsValue];

    /// Returns one positional value.
    fn argument_value(&self, index: u_int) -> Option<&ArgsValue> {
        self.argument_values().get(index as usize)
    }

    /// Replaces one positional value with a string.
    fn set_argument_string(&mut self, index: u_int, value: &CStr) -> bool;

    /// Returns one positional value in its string form.
    fn argument_string(&self, index: u_int) -> Option<&CStr>;

    /// Returns every value attached to `flag`, in insertion order.
    fn argument_flag_values(&self, flag: u_char) -> Vec<&ArgsValue>;

    /// Copies the arguments with `%1`…`%9` template substitution.
    unsafe fn copy_with_arguments(&self, argv: &[CString]) -> Box<Self>
    where
        Self: Sized;

    /// Positional arguments as strings, printing command lists and skipping empty values.
    unsafe fn to_vector(&self) -> Vec<CString>;

    /// Prints the arguments as a command-line fragment.
    unsafe fn print(&self) -> CString;

    /// Reads a number from the last string value of `flag` within `[minimum, maximum]`.
    fn strtonum(
        &self,
        flag: u_char,
        minimum: c_longlong,
        maximum: c_longlong,
        cause: &mut Option<CString>,
    ) -> c_longlong;

    /// Like [`Self::strtonum`] but expanding the value as a format first.
    unsafe fn strtonum_and_expand(
        &self,
        flag: u_char,
        minimum: c_longlong,
        maximum: c_longlong,
        item: &cmdq_item,
        cause: &mut Option<CString>,
    ) -> c_longlong;

    /// Reads a number or percentage from the last string value of `flag`.
    fn percentage(
        &self,
        flag: u_char,
        minimum: c_longlong,
        maximum: c_longlong,
        current: c_longlong,
        cause: &mut Option<CString>,
    ) -> c_longlong;

    /// Like [`Self::percentage`] but expanding the value as a format first.
    unsafe fn percentage_and_expand(
        &self,
        flag: u_char,
        minimum: c_longlong,
        maximum: c_longlong,
        current: c_longlong,
        item: &cmdq_item,
        cause: &mut Option<CString>,
    ) -> c_longlong;
}
