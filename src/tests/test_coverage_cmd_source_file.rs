//! Unit tests for the `source-file` command metadata and parser behavior.

use crate::arguments::{args_count, args_get, args_has, args_string};
use crate::cmd::cmd_source_file::*;
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::cmd::{cmd_find, cmd_table};
use crate::tests::test_fixtures::{Args, Item, globals, seen};
use ::core::ffi::c_char;
use ::core::ptr::null_mut;

fn entry() -> *const cmd_entry {
    &raw const cmd_source_file_entry
}

#[test]
fn entry_metadata_matches_upstream() {
    unsafe {
        let e = entry();
        assert_eq!((*e).name.to_bytes(), b"source-file");
        assert_eq!(
            (*e).alias.expect("the entry has an alias").to_bytes(),
            b"source"
        );
        assert_eq!((*e).usage.to_bytes(), b"[-Fnqv] [-t target-pane] path ...");
        assert_eq!((*e).args.template.to_bytes(), b"t:Fnqv");
        assert_eq!((*e).args.lower, 1);
        assert_eq!((*e).args.upper, -1);
        assert!((*e).args.cb.is_none());
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flag, 't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags & CMD_FIND_CANFAIL, CMD_FIND_CANFAIL);
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
        for name in [c"source-file", c"source", c"source-f"] {
            assert_eq!(cmd_find(name.as_ptr(), &mut cause), entry());
            assert!(cause.is_none());
        }
    }
}

#[test]
fn parsing_resolves_names_flags_target_and_paths() {
    let _guard = globals();
    unsafe {
        let plain = Args::parse(c"source-file /tmp/a.conf");
        assert!(::core::ptr::eq((*plain.cmd()).entry, entry()));
        assert_eq!(args_count(&*plain.ptr()), 1);
        assert_eq!(seen(args_string(&*plain.ptr(), 0)), "/tmp/a.conf");

        let alias = Args::parse(c"source /tmp/b.conf");
        assert!(::core::ptr::eq((*alias.cmd()).entry, entry()));
        assert_eq!(seen(args_string(&*alias.ptr(), 0)), "/tmp/b.conf");

        let full = Args::parse(c"source -Fnqvt %0 /tmp/c.conf /tmp/d.conf");
        assert!(::core::ptr::eq((*full.cmd()).entry, entry()));
        for flag in *b"Fnqv" {
            assert_eq!(args_has(&*full.ptr(), flag), 1);
        }
        assert_eq!(seen(args_get(&*full.ptr(), b't')), "%0");
        assert_eq!(args_count(&*full.ptr()), 2);
        assert_eq!(seen(args_string(&*full.ptr(), 0)), "/tmp/c.conf");
        assert_eq!(seen(args_string(&*full.ptr(), 1)), "/tmp/d.conf");
    }
}

#[test]
fn parsing_enforces_paths_and_flag_arguments() {
    let _guard = globals();
    unsafe {
        for line in [c"source-file", c"source"] {
            let mut parsed = cmd_parse_from_string(line.as_ptr(), null_mut());
            assert_eq!(parsed.status, CMD_PARSE_ERROR);
            assert!(parsed.take_error().contains("too few arguments"));
        }
        let mut bad = cmd_parse_from_string(c"source-file -z /tmp/x".as_ptr(), null_mut());
        assert_eq!(bad.status, CMD_PARSE_ERROR);
        assert!(bad.take_error().contains("unknown flag -z"));
        let mut missing = cmd_parse_from_string(c"source-file -t".as_ptr(), null_mut());
        assert_eq!(missing.status, CMD_PARSE_ERROR);
        assert!(missing.take_error().contains("-t expects an argument"));
        let mut ok = cmd_parse_from_string(c"source-file /tmp/x".as_ptr(), null_mut());
        assert_eq!(ok.status, CMD_PARSE_SUCCESS);
        let _ = ok.cmdlist.take();
    }
}

#[test]
fn source_file_exec_branches() {
    let _guard = globals();
    unsafe {
        let exec = (*entry()).exec;

        let mut peer = crate::tests::test_fixtures::zeroed::<tmuxpeer>();
        peer.flags |= crate::proc::PEER_BAD;
        let mut client_box = crate::tests::test_fixtures::zeroed_client();
        let caller = &raw mut *client_box;
        (*caller).peer = Some(peer);

        let mut item_err = Item::new().with_args(c"source-file /nonexistent_path_xyz123_abc.conf");
        item_err.set_client(caller);
        assert_eq!(exec(&*item_err.cmd(), item_err.ptr()), CMD_RETURN_ERROR);

        let mut item_quiet =
            Item::new().with_args(c"source-file -q /nonexistent_path_xyz123_abc.conf");
        item_quiet.set_client(caller);
        assert_eq!(
            exec(&*item_quiet.cmd(), item_quiet.ptr()),
            CMD_RETURN_NORMAL
        );

        let mut item_format =
            Item::new().with_args(c"source-file -q -F -n -v /nonexistent_path_xyz123_abc.conf");
        item_format.set_client(caller);
        assert_eq!(
            exec(&*item_format.cmd(), item_format.ptr()),
            CMD_RETURN_NORMAL
        );

        (*caller).source_file_depth = CMD_SOURCE_FILE_DEPTH_LIMIT as u_int;
        let mut item_depth = Item::new().with_args(c"source-file /tmp/any.conf");
        item_depth.set_client(caller);
        assert_eq!(exec(&*item_depth.cmd(), item_depth.ptr()), CMD_RETURN_ERROR);
        (*caller).source_file_depth = 0;

        crate::tests::test_fixtures::release_client(caller);
    }
}
