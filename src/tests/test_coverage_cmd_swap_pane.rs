//! Unit tests for the `swap-pane` command metadata and parser behavior.

use crate::arguments::{args_count, args_get, args_has};
use crate::cmd::cmd_swap_pane::*;
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::cmd::{cmd_find, cmd_table};
use crate::tests::test_fixtures::{Args, globals, seen};
use ::core::ffi::c_char;
use ::core::ptr::null_mut;

fn entry() -> *const cmd_entry {
    &raw const cmd_swap_pane_entry
}

#[test]
fn entry_metadata_matches_upstream() {
    unsafe {
        let e = entry();
        assert_eq!((*e).name.to_bytes(), b"swap-pane");
        assert_eq!(
            (*e).alias.expect("the entry has an alias").to_bytes(),
            b"swapp"
        );
        assert_eq!((*e).args.template.to_bytes(), b"dDs:t:UZ");
        assert_eq!(
            (*e).usage.to_bytes(),
            b"[-dDUZ] [-s src-pane] [-t dst-pane]"
        );
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 0);
        assert_eq!((*e).source.flag, 's' as c_char);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, CMD_FIND_DEFAULT_MARKED);
        assert_eq!((*e).target.flag, 't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, 0);
    }
}

#[test]
fn entry_is_registered_once_and_findable_by_name_alias_and_prefix() {
    let _guard = globals();
    unsafe {
        let found = cmd_table
            .iter()
            .filter(|slot| ::core::ptr::eq(**slot, entry()))
            .count();
        assert_eq!(found, 1);
        let mut cause = None;
        for name in [c"swap-pane", c"swapp", c"swap-p"] {
            assert_eq!(cmd_find(name.as_ptr(), &mut cause), entry());
            assert!(cause.is_none());
        }
    }
}

#[test]
fn parsing_carries_direction_detach_zoom_and_targets() {
    let _guard = globals();
    unsafe {
        let parsed = Args::parse(c"swap-pane -dDUZ -s %1 -t %2");
        assert!(::core::ptr::eq((*parsed.cmd()).entry, entry()));
        assert_eq!(args_count(&*parsed.ptr()), 0);
        for flag in *b"dDUZ" {
            assert_eq!(args_has(&*parsed.ptr(), flag), 1);
        }
        assert_eq!(seen(args_get(&*parsed.ptr(), b's')), "%1");
        assert_eq!(seen(args_get(&*parsed.ptr(), b't')), "%2");
        let alias = Args::parse(c"swapp -D");
        assert!(::core::ptr::eq((*alias.cmd()).entry, entry()));
        assert_eq!(args_has(&*alias.ptr(), b'D'), 1);
    }
}

#[test]
fn parsing_rejects_operands_unknown_flags_and_missing_targets() {
    let _guard = globals();
    unsafe {
        let mut operand = cmd_parse_from_string(c"swap-pane extra".as_ptr(), null_mut());
        assert_eq!(operand.status, CMD_PARSE_ERROR);
        assert!(operand.take_error().contains("too many arguments"));
        let mut flag = cmd_parse_from_string(c"swap-pane -x".as_ptr(), null_mut());
        assert_eq!(flag.status, CMD_PARSE_ERROR);
        assert!(flag.take_error().contains("unknown flag -x"));
        for line in [c"swap-pane -s", c"swap-pane -t"] {
            let mut missing = cmd_parse_from_string(line.as_ptr(), null_mut());
            assert_eq!(missing.status, CMD_PARSE_ERROR);
            assert!(missing.take_error().contains("expects an argument"));
        }
        let mut ok = cmd_parse_from_string(c"swap-pane".as_ptr(), null_mut());
        assert_eq!(ok.status, CMD_PARSE_SUCCESS);
    }
}
