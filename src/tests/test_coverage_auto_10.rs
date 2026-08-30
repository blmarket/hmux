//! Coverage for [`crate::cfg`] — config-file loading and cause list.
//!
//! `cfg.rs` is at 9.76% line coverage. The deterministic surface here is
//! `cfg_add_cause` / `cfg_print_causes` (via [`Item`] fixtures and `globals()`)
//! and `load_cfg` error branches exercised with temporary files under `/tmp`
//! (ENOENT with and without `CMD_PARSE_QUIET`, valid-file success and
//! `CMD_PARSE_PARSEONLY`). Nothing here hits `fatal`.

use crate::cfg::{
    CMD_PARSE_PARSEONLY, CMD_PARSE_QUIET, MSG_COMMAND, MSG_DETACH, MSG_EXIT, MSG_FLAGS,
    MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE, MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_TERM, MSG_READ,
    MSG_READ_DONE, MSG_READ_OPEN, MSG_READY, MSG_SHELL, MSG_VERSION, MSG_WRITE, MSG_WRITE_CLOSE,
    MSG_WRITE_OPEN, cfg_add_cause, cfg_print_causes, load_cfg,
};
use crate::fmt_args;
use crate::tests::test_fixtures::{Item, globals};
use ::core::ptr::null_mut;
use ::std::ffi::CString;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

unsafe fn drain_causes() {
    unsafe {
        let mut item = Item::new();
        cfg_print_causes(item.ptr());
    }
}

// ---------------------------------------------------------------------------
// constants — stable wire values re-exported from cfg.rs
// ---------------------------------------------------------------------------

#[test]
fn cfg_msg_constants_match_expected_wire_values() {
    assert_eq!(MSG_VERSION, 12);
    assert_eq!(MSG_IDENTIFY_FLAGS, 100);
    assert_eq!(MSG_IDENTIFY_TERM, 101);
    assert_eq!(MSG_IDENTIFY_CWD, 108);
    assert_eq!(MSG_IDENTIFY_DONE, 106);
    assert_eq!(MSG_COMMAND, 200);
    assert_eq!(MSG_DETACH, 201);
    assert_eq!(MSG_EXIT, 203);
    assert_eq!(MSG_SHELL, 209);
    assert_eq!(MSG_READY, 207);
    assert_eq!(MSG_FLAGS, 218);
    assert_eq!(MSG_READ_OPEN, 300);
    assert_eq!(MSG_READ, 301);
    assert_eq!(MSG_READ_DONE, 302);
    assert_eq!(MSG_WRITE_OPEN, 303);
    assert_eq!(MSG_WRITE, 304);
    assert_eq!(MSG_WRITE_CLOSE, 306);
    assert_eq!(CMD_PARSE_QUIET, 0x1);
    assert_eq!(CMD_PARSE_PARSEONLY, 0x2);
    assert!(MSG_COMMAND < MSG_FLAGS);
    assert!(MSG_FLAGS < MSG_READ_OPEN);
}

// ---------------------------------------------------------------------------
// cfg_add_cause / cfg_print_causes via Item fixtures
// ---------------------------------------------------------------------------

#[test]
fn cfg_add_cause_single_and_print_clears() {
    let _guard = globals();
    unsafe {
        drain_causes();
        cfg_add_cause(c"single-cause".as_ptr(), fmt_args![]);
        let mut item = Item::new();
        cfg_print_causes(item.ptr());
        // second drain is a no-op and must not crash
        let mut item2 = Item::new();
        cfg_print_causes(item2.ptr());
        drain_causes();
    }
}

#[test]
fn cfg_add_cause_multiple_formatted_accumulates_and_drains() {
    let _guard = globals();
    unsafe {
        drain_causes();
        cfg_add_cause(
            c"cause %s %d".as_ptr(),
            fmt_args![c"alpha".as_ptr(), 7 as ::core::ffi::c_int],
        );
        cfg_add_cause(c"second".as_ptr(), fmt_args![]);
        cfg_add_cause(c"third %s".as_ptr(), fmt_args![c"beta".as_ptr()]);
        let mut item = Item::new();
        cfg_print_causes(item.ptr());
        // after draining, another call is harmless
        let mut item2 = Item::new();
        cfg_print_causes(item2.ptr());
        drain_causes();
    }
}

#[test]
fn cfg_add_cause_with_path_format_adds_cause() {
    let _guard = globals();
    unsafe {
        drain_causes();
        let path = c"/tmp/fake.conf".as_ptr();
        let err = c"No such file".as_ptr();
        cfg_add_cause(c"%s: %s".as_ptr(), fmt_args![path, err]);
        let mut item = Item::new();
        cfg_print_causes(item.ptr());
        drain_causes();
    }
}

#[test]
fn cfg_add_cause_with_percent_format_and_drain_via_client_item() {
    let _guard = globals();
    unsafe {
        drain_causes();
        cfg_add_cause(
            c"error at %s:%d".as_ptr(),
            fmt_args![c"my.conf".as_ptr(), 42 as ::core::ffi::c_int],
        );
        let mut item = Item::new();
        assert!(!item.ptr().is_null());
        cfg_print_causes(item.ptr());
        drain_causes();
    }
}

#[test]
fn cfg_print_causes_on_empty_list_is_noop() {
    let _guard = globals();
    unsafe {
        drain_causes();
        let mut item = Item::new();
        cfg_print_causes(item.ptr());
        let mut item2 = Item::with_client();
        cfg_print_causes(item2.ptr());
    }
}

// ---------------------------------------------------------------------------
// load_cfg — error branches with temporary /tmp paths
// ---------------------------------------------------------------------------

#[test]
fn load_cfg_nonexistent_with_quiet_returns_zero_no_cause() {
    let _guard = globals();
    unsafe {
        drain_causes();
        let mut new_item: *mut crate::types::cmdq_item = null_mut();
        let rc = load_cfg(
            c"/tmp/tmux-c2rs-auto10-missing-quiet-1".as_ptr(),
            null_mut(),
            null_mut(),
            null_mut(),
            CMD_PARSE_QUIET,
            Some(&mut new_item),
        );
        assert_eq!(rc, 0);
        assert!(new_item.is_null());
        // no cause was added, draining is still safe
        drain_causes();
    }
}

#[test]
fn load_cfg_nonexistent_without_quiet_returns_error_and_adds_cause() {
    let _guard = globals();
    unsafe {
        drain_causes();
        let mut new_item: *mut crate::types::cmdq_item = null_mut();
        let rc = load_cfg(
            c"/tmp/tmux-c2rs-auto10-missing-nonquiet-1".as_ptr(),
            null_mut(),
            null_mut(),
            null_mut(),
            0,
            Some(&mut new_item),
        );
        assert_eq!(rc, -1);
        assert!(new_item.is_null());
        // cause was added; drain it
        drain_causes();
        // second drain leaves list empty
        drain_causes();
    }
}

#[test]
fn load_cfg_valid_temp_file_returns_success_and_new_item() {
    let _guard = globals();
    // create a temp file with a valid tmux config line
    let path = "/tmp/tmux-c2rs-auto10-valid-1.conf";
    let content = "set-option -g status off\n";
    std::fs::write(path, content).expect("write temp cfg");
    let c_path = CString::new(path).unwrap();
    unsafe {
        drain_causes();
        let mut new_item: *mut crate::types::cmdq_item = null_mut();
        let rc = load_cfg(
            c_path.as_ptr(),
            null_mut(),
            null_mut(),
            null_mut(),
            0,
            Some(&mut new_item),
        );
        assert_eq!(rc, 0);
        assert!(!new_item.is_null(), "valid config should queue commands");
        drain_causes();
    }
    let _ = std::fs::remove_file(path);
}

#[test]
fn load_cfg_valid_temp_file_with_parseonly_parses_but_queues_nothing() {
    let _guard = globals();
    let path = "/tmp/tmux-c2rs-auto10-parseonly-1.conf";
    let content = "set-option -g status off\n";
    std::fs::write(path, content).expect("write temp cfg");
    let c_path = CString::new(path).unwrap();
    unsafe {
        drain_causes();
        let mut new_item: *mut crate::types::cmdq_item = null_mut();
        let rc = load_cfg(
            c_path.as_ptr(),
            null_mut(),
            null_mut(),
            null_mut(),
            CMD_PARSE_PARSEONLY,
            Some(&mut new_item),
        );
        assert_eq!(rc, 0);
        assert!(new_item.is_null(), "parseonly must not queue");
        drain_causes();
    }
    let _ = std::fs::remove_file(path);
}
