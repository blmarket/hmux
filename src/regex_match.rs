//! Stable access to one regular-expression match range.

/// The byte offsets reported for one regular-expression match or capture.
pub trait RegexMatch {
    /// Builds a match range from its inclusive start and exclusive end offsets.
    fn from_regex_match(start: i32, end: i32) -> Self
    where
        Self: Sized;

    /// Returns the inclusive start offset, or `-1` for an unmatched capture.
    fn regex_match_start(&self) -> i32;

    /// Returns the exclusive end offset, or `-1` for an unmatched capture.
    fn regex_match_end(&self) -> i32;
}

impl RegexMatch for crate::types::regmatch_t {
    fn from_regex_match(start: i32, end: i32) -> Self {
        Self {
            rm_so: start,
            rm_eo: end,
        }
    }
    fn regex_match_start(&self) -> i32 {
        self.rm_so
    }
    fn regex_match_end(&self) -> i32 {
        self.rm_eo
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::regmatch_t;

    #[test]
    fn offsets_round_trip() {
        let matched = regmatch_t::from_regex_match(3, 8);
        assert_eq!(matched.regex_match_start(), 3);
        assert_eq!(matched.regex_match_end(), 8);
        let unmatched = regmatch_t::from_regex_match(-1, -1);
        assert_eq!(unmatched.regex_match_start(), -1);
        assert_eq!(unmatched.regex_match_end(), -1);
    }
}
