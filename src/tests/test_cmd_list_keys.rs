use super::*;
use crate::options::OptionsRef;
use crate::tmux::global_s_options;

use crate::tests::test_fixtures::{Item, globals};
use ::core::ffi::{CStr, c_longlong};

/// A turn at the session-wide `prefix` option, put back to the value it
/// was found with even through a failed assertion.
struct Prefix(c_longlong);

impl Prefix {
    /// Remembers what `prefix` is set to now.
    fn guard() -> Prefix {
        unsafe {
            Prefix(
                (global_s_options
                    .as_ref()
                    .expect("global options are initialized"))
                .number(c"prefix"),
            )
        }
    }

    /// Makes `prefix` the key `name` spells, or no key at all when it
    /// spells none.
    fn set(name: &CStr) {
        unsafe {
            let key = RustKeyStringCodec.parse_key(name);
            (global_s_options
                .as_ref()
                .expect("global options are initialized"))
            .set_number(c"prefix", key as c_longlong);
        }
    }
}

impl Drop for Prefix {
    fn drop(&mut self) {
        unsafe {
            (global_s_options
                .as_ref()
                .expect("global options are initialized"))
            .set_number(c"prefix", self.0)
        };
    }
}

/// The prefix string the `list-keys` line `s` would start each `-N` line
/// with, freed again.
fn prefix_of(s: &CStr) -> String {
    unsafe {
        let item = Item::new().with_args(s);
        let p = cmd_list_keys_get_prefix(&*item.args());
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
