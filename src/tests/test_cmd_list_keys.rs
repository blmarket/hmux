use super::*;
use crate::options::options_set_number;
use crate::tests::test_fixtures::{Item, globals};
use ::core::ffi::{CStr, c_longlong};

/// A turn at the session-wide `prefix` option, put back to the value it
/// was found with even through a failed assertion.
struct Prefix(c_longlong);

impl Prefix {
    /// Remembers what `prefix` is set to now.
    fn guard() -> Prefix {
        unsafe { Prefix(options_get_number(global_s_options, c"prefix".as_ptr())) }
    }

    /// Makes `prefix` the key `name` spells, or no key at all when it
    /// spells none.
    fn set(name: &CStr) {
        unsafe {
            let key = key_string_lookup_string(name.as_ptr());
            options_set_number(global_s_options, c"prefix".as_ptr(), key as c_longlong);
        }
    }
}

impl Drop for Prefix {
    fn drop(&mut self) {
        unsafe { options_set_number(global_s_options, c"prefix".as_ptr(), self.0) };
    }
}

/// The prefix string the `list-keys` line `s` would start each `-N` line
/// with, freed again.
fn prefix_of(s: &CStr) -> String {
    unsafe {
        let mut item = Item::new().with_args(s);
        let p = cmd_list_keys_get_prefix(cmd_get_args(&*item.cmd()));
        p.to_string_lossy().into_owned()
    }
}

#[test]
fn the_prefix_string_is_the_p_argument_the_prefix_key_or_nothing() {
    let _guard = globals();
    {
        let _prefix = Prefix::guard();

        Prefix::set(c"C-b");
        assert_eq!(prefix_of(c"list-keys"), "C-b");
        assert_eq!(
            prefix_of(c"list-keys -P PFX"),
            "PFX",
            "-P is taken as given"
        );

        Prefix::set(c"None");
        assert_eq!(
            prefix_of(c"list-keys"),
            "",
            "an unbound prefix key prints nothing at all"
        );
        assert_eq!(
            prefix_of(c"list-keys -P PFX"),
            "PFX",
            "-P still wins with no prefix key"
        );
    }
}
