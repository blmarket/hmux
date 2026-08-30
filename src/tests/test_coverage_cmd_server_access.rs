//! Unit tests for [`crate::cmd::cmd_server_access`] and [`crate::server`].

use crate::arguments::{args_count, args_has};
use crate::cmd::cmd_get_args;
use crate::cmd::cmd_server_access::{
    CMD_CLIENT_CANFAIL, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, cmd_server_access_entry,
};
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::proc::PEER_BAD;
use crate::server::*;
use crate::tests::test_fixtures::{Item, globals, zeroed};
use crate::types::*;
use ::core::ffi::CStr;
use ::core::ptr::null_mut;

const ENTRY: *const cmd_entry = &raw const cmd_server_access_entry;
const FILE: &CStr = c"test-coverage-cmd-server-access.conf";

fn bad_peer() -> Box<tmuxpeer> {
    let mut p = zeroed::<tmuxpeer>();
    p.flags |= PEER_BAD;
    p
}

unsafe fn wire(c: *mut client) {
    unsafe {
        (*c).peer = Some(bad_peer());
    }
}

unsafe fn run(item: &mut Item) -> cmd_retval {
    unsafe {
        let exec = (*ENTRY).exec;
        exec(&*item.cmd(), item.ptr())
    }
}

#[test]
fn entry_metadata_matches_upstream() {
    let _guard = globals();
    unsafe {
        assert_eq!((*ENTRY).name.to_string_lossy(), "server-access");
        assert!((*ENTRY).alias.is_none());
        assert_eq!((*ENTRY).args.template.to_string_lossy(), "adlrw");
        assert_eq!((*ENTRY).args.lower, 0);
        assert_eq!((*ENTRY).args.upper, 1);
        assert!((*ENTRY).args.cb.is_none());
        assert_eq!(
            (*ENTRY).usage.to_string_lossy(),
            "[-adlrw] [-t target-pane] [user]"
        );
        assert_eq!((*ENTRY).source.type_0, CMD_FIND_PANE);
        assert_eq!((*ENTRY).target.type_0, CMD_FIND_PANE);
        assert_eq!((*ENTRY).flags, CMD_CLIENT_CANFAIL);
    }
}

#[test]
fn parsing_and_flags_validation() {
    let _guard = globals();
    unsafe {
        let mut plain = Item::new().from_file(FILE, 1).with_args(c"server-access");
        assert!(::core::ptr::eq((*plain.cmd()).entry, ENTRY));

        let mut flagged = Item::new()
            .from_file(FILE, 2)
            .with_args(c"server-access -a -w nobody");
        assert!(::core::ptr::eq((*flagged.cmd()).entry, ENTRY));
        let args = cmd_get_args(&*flagged.cmd());
        assert_eq!(args_has(args, b'a'), 1);
        assert_eq!(args_has(args, b'w'), 1);
        assert_eq!(args_count(args), 1);

        let mut bad_flag = cmd_parse_from_string(c"server-access -z".as_ptr(), null_mut());
        assert_eq!(bad_flag.status, CMD_PARSE_ERROR);
        assert!(bad_flag.take_error().contains("unknown flag"));

        let mut extra = cmd_parse_from_string(c"server-access user1 user2".as_ptr(), null_mut());
        assert_eq!(extra.status, CMD_PARSE_ERROR);
        assert!(extra.take_error().contains("too many arguments"));

        let mut ok = cmd_parse_from_string(c"server-access -l".as_ptr(), null_mut());
        assert_eq!(ok.status, CMD_PARSE_SUCCESS);
        let _ = ok.cmdlist.take();
    }
}

#[test]
fn server_access_exec_list_and_validation_errors() {
    let _guard = globals();
    unsafe {
        server_acl_init();

        let mut client_box = crate::tests::test_fixtures::zeroed_client();
        let caller = &raw mut *client_box;
        wire(caller);

        let mut item_list = Item::new()
            .from_file(FILE, 10)
            .with_args(c"server-access -l");
        item_list.set_client(caller);
        assert_eq!(run(&mut item_list), CMD_RETURN_NORMAL);

        let mut item_no_arg = Item::new().from_file(FILE, 11).with_args(c"server-access");
        item_no_arg.set_client(caller);
        assert_eq!(run(&mut item_no_arg), CMD_RETURN_ERROR);

        let mut item_unknown = Item::new()
            .from_file(FILE, 12)
            .with_args(c"server-access nonexistent_user_abc123");
        item_unknown.set_client(caller);
        assert_eq!(run(&mut item_unknown), CMD_RETURN_ERROR);

        let mut item_root = Item::new()
            .from_file(FILE, 13)
            .with_args(c"server-access root");
        item_root.set_client(caller);
        assert_eq!(run(&mut item_root), CMD_RETURN_ERROR);

        let mut item_ad = Item::new()
            .from_file(FILE, 14)
            .with_args(c"server-access -a -d nobody");
        item_ad.set_client(caller);
        assert_eq!(run(&mut item_ad), CMD_RETURN_ERROR);

        let mut item_rw = Item::new()
            .from_file(FILE, 15)
            .with_args(c"server-access -r -w nobody");
        item_rw.set_client(caller);
        assert_eq!(run(&mut item_rw), CMD_RETURN_ERROR);

        crate::tests::test_fixtures::release_client(caller);
    }
}

#[test]
fn server_access_exec_allow_modify_and_deny() {
    let _guard = globals();
    unsafe {
        server_acl_init();

        let mut client_box = crate::tests::test_fixtures::zeroed_client();
        let caller = &raw mut *client_box;
        wire(caller);

        server_acl_user_deny(65534);

        let mut item_deny_missing = Item::new()
            .from_file(FILE, 20)
            .with_args(c"server-access -d nobody");
        item_deny_missing.set_client(caller);
        assert_eq!(run(&mut item_deny_missing), CMD_RETURN_ERROR);

        let mut item_allow = Item::new()
            .from_file(FILE, 21)
            .with_args(c"server-access -a nobody");
        item_allow.set_client(caller);
        assert_eq!(run(&mut item_allow), CMD_RETURN_NORMAL);
        assert!(!server_acl_user_find(65534).is_null());

        let mut item_allow_again = Item::new()
            .from_file(FILE, 22)
            .with_args(c"server-access -a nobody");
        item_allow_again.set_client(caller);
        assert_eq!(run(&mut item_allow_again), CMD_RETURN_ERROR);

        let mut item_ro = Item::new()
            .from_file(FILE, 23)
            .with_args(c"server-access -r nobody");
        item_ro.set_client(caller);
        assert_eq!(run(&mut item_ro), CMD_RETURN_NORMAL);
        let u = server_acl_user_find(65534);
        assert_eq!((*u).flags, SERVER_ACL_READONLY);

        let mut item_rw = Item::new()
            .from_file(FILE, 24)
            .with_args(c"server-access -w nobody");
        item_rw.set_client(caller);
        assert_eq!(run(&mut item_rw), CMD_RETURN_NORMAL);
        assert_eq!((*u).flags, 0);

        let mut item_list = Item::new()
            .from_file(FILE, 25)
            .with_args(c"server-access -l");
        item_list.set_client(caller);
        assert_eq!(run(&mut item_list), CMD_RETURN_NORMAL);

        let mut item_deny = Item::new()
            .from_file(FILE, 26)
            .with_args(c"server-access -d nobody");
        item_deny.set_client(caller);
        assert_eq!(run(&mut item_deny), CMD_RETURN_NORMAL);
        assert!(server_acl_user_find(65534).is_null());

        crate::tests::test_fixtures::release_client(caller);
    }
}

#[test]
fn server_acl_operations_and_client_join() {
    let _guard = globals();
    unsafe {
        server_acl_init();
        server_acl_user_allow(12345);
        let u = server_acl_user_find(12345);
        assert!(!u.is_null());
        assert_eq!(server_acl_get_uid(u), 12345);

        server_acl_user_deny_write(12345);
        assert_eq!((*u).flags, SERVER_ACL_READONLY);

        server_acl_user_allow_write(12345);
        assert_eq!((*u).flags, 0);

        server_acl_user_deny(12345);
        assert!(server_acl_user_find(12345).is_null());

        let mut client_box = crate::tests::test_fixtures::zeroed_client();
        let c = &raw mut *client_box;
        wire(c);

        assert_eq!(server_acl_join(c), 1);

        server_acl_user_deny(0);
        assert_eq!(server_acl_join(c), 0);

        crate::tests::test_fixtures::release_client(c);
    }
}
