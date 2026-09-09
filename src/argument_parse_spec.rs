//! Stable access to an argument parser specification.

use crate::types::args_parse_cb;
use core::ffi::{CStr, c_int};

/// The flag template, positional bounds, and classifier used to parse arguments.
pub trait ArgumentParseSpec {
    /// Builds a parser specification.
    fn from_argument_parse_spec(
        template: &'static CStr,
        lower: c_int,
        upper: c_int,
        callback: args_parse_cb,
    ) -> Self
    where
        Self: Sized;

    /// Returns the flag template.
    fn argument_parse_template(&self) -> &CStr;

    /// Returns the minimum positional argument count, or `-1` when unbounded.
    fn argument_parse_lower_bound(&self) -> c_int;

    /// Returns the maximum positional argument count, or `-1` when unbounded.
    fn argument_parse_upper_bound(&self) -> c_int;

    /// Returns the positional argument classifier, when one is configured.
    fn argument_parse_callback(&self) -> args_parse_cb;
}

impl ArgumentParseSpec for crate::types::args_parse_t {
    fn from_argument_parse_spec(
        template: &'static CStr,
        lower: c_int,
        upper: c_int,
        callback: args_parse_cb,
    ) -> Self {
        Self {
            template,
            lower,
            upper,
            cb: callback,
        }
    }

    fn argument_parse_template(&self) -> &CStr {
        self.template
    }

    fn argument_parse_lower_bound(&self) -> c_int {
        self.lower
    }

    fn argument_parse_upper_bound(&self) -> c_int {
        self.upper
    }

    fn argument_parse_callback(&self) -> args_parse_cb {
        self.cb
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::args_parse_t;

    #[test]
    fn const_spec_exposes_its_configuration() {
        const SPEC: args_parse_t = args_parse_t::new(c"ab:", 1, 3, None);
        assert_eq!(SPEC.argument_parse_template(), c"ab:");
        assert_eq!(SPEC.argument_parse_lower_bound(), 1);
        assert_eq!(SPEC.argument_parse_upper_bound(), 3);
        assert!(SPEC.argument_parse_callback().is_none());
    }
}
