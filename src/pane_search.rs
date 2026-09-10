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

    /// Forgets the retained search and its interpretation.
    fn clear(&mut self);

    /// Returns whether both retained search properties match the arguments.
    fn matches(&self, query: &CStr, regex: bool) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_search_is_replaced_matched_and_cleared_as_one_value() {
        let mut state = crate::types::window_pane::default();
        assert_eq!(state.query(), None);
        assert!(!state.is_regex());
        assert!(!state.matches(c"first", false));

        state.set(c"first", false);
        assert_eq!(state.query(), Some(c"first"));
        assert!(state.matches(c"first", false));
        assert!(!state.matches(c"first", true));

        state.set(c"s[ée]cond", true);
        assert_eq!(state.query(), Some(c"s[ée]cond"));
        assert!(state.is_regex());
        state.clear();
        assert_eq!(state.query(), None);
        assert!(!state.is_regex());
    }
}
