//! Authoritative format-sort order values.

pub type sort_order = ::core::ffi::c_uint;
pub const SORT_END: sort_order = 8;
pub const SORT_Z: sort_order = 7;
pub const SORT_SIZE: sort_order = 6;
pub const SORT_ORDER: sort_order = 5;
pub const SORT_NAME: sort_order = 4;
pub const SORT_MODIFIER: sort_order = 3;
pub const SORT_INDEX: sort_order = 2;
pub const SORT_CREATION: sort_order = 1;
pub const SORT_ACTIVITY: sort_order = 0;

#[cfg(test)]
mod tests {
    use super::*;
    use ::core::mem::{align_of, size_of};

    #[test]
    fn sort_domain_matches_translated_c_baseline() {
        assert_eq!(size_of::<sort_order>(), 4);
        assert_eq!(align_of::<sort_order>(), 4);
        assert_eq!(SORT_ACTIVITY, 0);
        assert_eq!(SORT_END, 8);
    }
}
