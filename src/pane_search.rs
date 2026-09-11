//! The last search retained by a pane for copy mode and formats.

use core::ffi::CStr;

/// A pane's retained search text and interpretation.
pub trait PaneSearchState {
    /// Returns the retained search text, if any.
    fn query(&self) -> Option<&CStr>;

    /// Returns whether the retained text is a regular expression.
    fn is_regex(&self) -> bool;

    /// Replaces the retained search.
    fn set(&mut self, query: &CStr, regex: bool);

    /// Returns whether both retained search properties match the arguments.
    fn matches(&self, query: &CStr, regex: bool) -> bool;
}
