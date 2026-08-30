//! Unit tests for [`crate::cmd::cmd_load_buffer`] — the `load-buffer` entry
//! metadata, its registration in the command table and resolution through
//! `cmd_find`, the argument bounds and flags the parser enforces for both of
//! its names, the message-protocol, enumeration and flag constants this
//! generated file carries, and the layout of the [`cmd_load_buffer_data`]
//! state that travels with a load.
//!
//! Both functions in the module belong to asynchronous file completion:
//! `cmd_load_buffer_exec` starts a `file_read` and answers
//! [`CMD_RETURN_WAIT`], and `cmd_load_buffer_done` is the completion callback
//! the file code invokes when the read settles. Neither is driven here —
//! nothing arms ensure_reactor or runs an event loop, no descriptor is opened and
//! no completion is waited on — so the coverage is the metadata, the parsing
//! and the state struct.

use crate::arguments::{args_count, args_get, args_has, args_string};
use crate::cmd::cmd_load_buffer::*;
use crate::cmd::cmdq_set_target_client;
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::cmd::{CmdqItemWeak, cmdq_item_weak_from_ptr};
use crate::cmd::{cmd_find, cmd_list_first, cmd_table};
use crate::paste::paste_buffer_data;
use crate::tests::test_fixtures::{Args, globals, seen, zeroed, zeroed_client};
use crate::types::ClientFileData;
use ::core::ptr::null_mut;

/// The command's table entry as a raw pointer, so every field read stays an
/// explicit unsafe dereference rather than a shared reference into a
/// `static mut`.
fn entry() -> *const cmd_entry {
    &raw const cmd_load_buffer_entry
}

#[test]
fn entry_metadata_matches_upstream() {
    unsafe {
        let e = entry();
        assert_eq!((*e).name.to_bytes(), b"load-buffer");
        assert_eq!(
            (*e).alias.expect("the entry has an alias").to_bytes(),
            b"loadb"
        );
        assert_eq!(
            (*e).usage.to_bytes(),
            b"[-b buffer-name] [-t target-client] path"
        );

        assert_eq!((*e).args.template.to_bytes(), b"b:t:w");
        assert_eq!((*e).args.lower, 1);
        assert_eq!((*e).args.upper, 1);
        assert!((*e).args.cb.is_none());

        let flags = [&raw const (*e).source, &raw const (*e).target];
        for flag in flags {
            assert_eq!((*flag).flag, 0);
            assert_eq!((*flag).type_0, CMD_FIND_PANE);
            assert_eq!((*flag).flags, 0);
        }

        assert_eq!(
            (*e).flags,
            CMD_AFTERHOOK | CMD_CLIENT_TFLAG | CMD_CLIENT_CANFAIL
        );
        assert_eq!((*e).flags & CMD_AFTERHOOK, CMD_AFTERHOOK);
        assert_eq!((*e).flags & CMD_CLIENT_TFLAG, CMD_CLIENT_TFLAG);
        assert_eq!((*e).flags & CMD_CLIENT_CANFAIL, CMD_CLIENT_CANFAIL);
        assert_eq!(
            (*e).flags & !(CMD_AFTERHOOK | CMD_CLIENT_TFLAG | CMD_CLIENT_CANFAIL),
            0
        );
    }
}

#[test]
fn entry_is_registered_once_in_cmd_table_and_findable_by_name_alias_and_prefix() {
    let _guard = globals();
    unsafe {
        let found = cmd_table
            .iter()
            .filter(|slot| ::core::ptr::eq(**slot, entry()))
            .count();
        assert_eq!(found, 1, "the entry appears exactly once");

        let mut cause = None;
        assert_eq!(cmd_find(c"load-buffer".as_ptr(), &mut cause), entry());
        assert!(cause.is_none(), "no cause on success");

        assert_eq!(cmd_find(c"loadb".as_ptr(), &mut cause), entry());
        assert!(cause.is_none(), "no cause on success");

        assert_eq!(cmd_find(c"load-b".as_ptr(), &mut cause), entry());
        assert!(cause.is_none(), "no cause on success");
    }
}

#[test]
fn parsing_resolves_both_names_and_carries_the_b_and_w_flags_and_the_path() {
    let _guard = globals();
    unsafe {
        for line in [c"load-buffer /tmp/paste", c"loadb /tmp/paste"] {
            let parsed = Args::parse(line);
            assert!(
                ::core::ptr::eq((*parsed.cmd()).entry, entry()),
                "{line:?} did not resolve"
            );
            let args = parsed.ptr();
            assert_eq!(args_has(&*args, b'b'), 0);
            assert_eq!(args_has(&*args, b'w'), 0);
            assert_eq!(args_count(&*args), 1);
            assert_eq!(seen(args_string(&*args, 0)), "/tmp/paste");
        }

        let full = Args::parse(c"load-buffer -b mybuf -w /tmp/other");
        assert!(::core::ptr::eq((*full.cmd()).entry, entry()));
        let args = full.ptr();
        assert_eq!(args_has(&*args, b'w'), 1);
        assert_eq!(seen(args_get(&*args, b'b')), "mybuf");
        assert_eq!(seen(args_string(&*args, 0)), "/tmp/other");

        let alias_full = Args::parse(c"loadb -w -b other /tmp/third");
        assert!(::core::ptr::eq((*alias_full.cmd()).entry, entry()));
        let args = alias_full.ptr();
        assert_eq!(args_has(&*args, b'w'), 1);
        assert_eq!(seen(args_get(&*args, b'b')), "other");
        assert_eq!(seen(args_string(&*args, 0)), "/tmp/third");
    }
}

#[test]
fn parsing_enforces_exactly_one_path_and_rejects_unknown_flags() {
    let _guard = globals();
    unsafe {
        let mut none = cmd_parse_from_string(c"load-buffer".as_ptr(), null_mut());
        assert_eq!(none.status, CMD_PARSE_ERROR);
        let err = none.take_error();
        assert!(err.contains("too few arguments"), "{err}");
        assert!(err.contains("need at least 1"), "{err}");

        let mut alias_none = cmd_parse_from_string(c"loadb".as_ptr(), null_mut());
        assert_eq!(alias_none.status, CMD_PARSE_ERROR);
        let err = alias_none.take_error();
        assert!(err.contains("too few arguments"), "{err}");

        let mut extra =
            cmd_parse_from_string(c"load-buffer /tmp/one /tmp/two".as_ptr(), null_mut());
        assert_eq!(extra.status, CMD_PARSE_ERROR);
        let err = extra.take_error();
        assert!(err.contains("too many arguments"), "{err}");
        assert!(err.contains("need at most 1"), "{err}");

        let mut bad_flag = cmd_parse_from_string(c"load-buffer -z /tmp/x".as_ptr(), null_mut());
        assert_eq!(bad_flag.status, CMD_PARSE_ERROR);
        let err_flag = bad_flag.take_error();
        assert!(err_flag.contains("unknown flag"), "{err_flag}");

        let mut good = cmd_parse_from_string(c"load-buffer /tmp/x".as_ptr(), null_mut());
        assert_eq!(good.status, CMD_PARSE_SUCCESS);
        let _ = good.cmdlist.take();

        let mut bare_flagless = cmd_parse_from_string(c"loadb -w /tmp/x".as_ptr(), null_mut());
        assert_eq!(bare_flagless.status, CMD_PARSE_SUCCESS);
        let first = cmd_list_first(bare_flagless.cmdlist.as_ref().unwrap().as_ptr());
        assert!(!first.is_null());
        let _ = bare_flagless.cmdlist.take();
    }
}

/// The state a load-buffer carries: the client it holds, the item it does
/// not own and the buffer name it does. A zeroed one starts empty, which is
/// what the transpiled `xcalloc` leaves.
#[test]
fn the_state_struct_zeroes_to_empty_fields() {
    let _guard = globals();
    let mut data = cmd_load_buffer_data {
        client_ref: None,
        item: None,
        name: None,
    };
    assert!(data.client().is_null());
    assert!(data.item.is_none());
    assert!(data.name.is_none());
    let client = zeroed_client();
    let mut item = crate::tests::test_fixtures::Item::new();
    data.client_ref = Some(client.clone());
    data.item = unsafe { cmdq_item_weak_from_ptr(item.ptr()) };
    data.name = Some(c"buffer".to_owned());
    assert_eq!(data.client(), client.as_ptr());
    assert_eq!(
        data.item
            .as_ref()
            .and_then(CmdqItemWeak::upgrade)
            .map(|held| held.as_ptr()),
        Some(item.ptr())
    );
    assert_eq!(data.name.as_deref(), Some(c"buffer"));
}

#[test]
fn load_buffer_exec_initiates_file_read() {
    let _guard = globals();
    unsafe {
        let exec = (*entry()).exec;

        let mut peer = zeroed::<tmuxpeer>();
        peer.flags |= crate::proc::PEER_BAD;

        let mut item = crate::tests::test_fixtures::Item::new()
            .with_args(c"load-buffer -b mybuf -w /tmp/testfile");
        let mut client_box = crate::tests::test_fixtures::zeroed_client();
        let c = &raw mut *client_box;
        (*c).peer = Some(peer);
        item.set_client(c);
        cmdq_set_target_client(item.ptr(), c);

        assert_eq!(exec(&*item.cmd(), item.ptr()), CMD_RETURN_WAIT);

        crate::tests::test_fixtures::release_client(c);
    }
}

#[test]
fn test_cmd_load_buffer_done_callbacks() {
    let _guard = globals();
    unsafe {
        let mut item =
            crate::tests::test_fixtures::Item::new().with_args(c"load-buffer /tmp/testfile");
        let mut cdata = Box::new(cmd_load_buffer_data {
            client_ref: None,
            item: cmdq_item_weak_from_ptr(item.ptr()),
            name: Some(c"my_loaded_buf".to_owned()),
        });
        let cdata_ptr = cdata.as_mut() as *mut cmd_load_buffer_data;

        // 1. closed == 0: early return
        cmd_load_buffer_done(
            null_mut(),
            c"/tmp/testfile".as_ptr(),
            0,
            0,
            null_mut(),
            ClientFileData::LoadBufferView(cdata_ptr),
        );

        // 2. closed == 1, error != 0
        let mut peer = zeroed::<tmuxpeer>();
        peer.flags |= crate::proc::PEER_BAD;
        let mut client_box = crate::tests::test_fixtures::zeroed_client();
        let c = &raw mut *client_box;
        (*c).peer = Some(peer);
        item.set_client(c);

        cmd_load_buffer_done(
            null_mut(),
            c"/tmp/testfile".as_ptr(),
            2,
            1,
            null_mut(),
            ClientFileData::LoadBuffer(cdata),
        );

        // 3. closed == 1, error == 0, with content
        let cdata2 = Box::new(cmd_load_buffer_data {
            client_ref: None,
            item: cmdq_item_weak_from_ptr(item.ptr()),
            name: Some(c"my_loaded_buf".to_owned()),
        });

        let mut buf = Buf::new();
        buf.append(b"hello world buffer content");
        cmd_load_buffer_done(
            null_mut(),
            c"/tmp/testfile".as_ptr(),
            0,
            1,
            &raw mut buf,
            ClientFileData::LoadBuffer(cdata2),
        );

        let pb = crate::paste::paste_get_name(c"my_loaded_buf".as_ptr());
        assert!(!pb.is_null());
        assert_eq!(paste_buffer_data(&*pb), b"hello world buffer content");
        crate::paste::paste_free(pb);

        crate::tests::test_fixtures::release_client(c);
    }
}
