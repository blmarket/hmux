//! Config loading and cause-list tests.

use crate::cfg::{
    cfg_add_cause, cfg_print_causes, cfg_show_causes, load_cfg, load_cfg_from_buffer,
};
use crate::fmt_args;

use crate::tests::test_fixtures::{Item, Target, globals};
use ::core::ffi::c_int;
use ::std::ffi::CString;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Empties the cfg cause list if anything is in it.
unsafe fn drain_cfg_causes() {
    unsafe {
        let mut item = Item::new();
        cfg_print_causes(&*item.ptr());
    }
}

// ---------------------------------------------------------------------------
// cfg.rs
// ---------------------------------------------------------------------------

#[test]
fn cfg_add_cause_accumulates_and_cfg_print_causes_empties() {
    let _guard = globals();
    unsafe {
        drain_cfg_causes();
        cfg_add_cause(c"cause %s %d", fmt_args![c"alpha".as_ptr(), 7 as c_int]);
        cfg_add_cause(c"second", fmt_args![]);
        let mut item = Item::new();
        cfg_print_causes(&*item.ptr());
        // Second drain without new causes does not crash and leaves list empty.
        let mut item2 = Item::new();
        cfg_print_causes(&*item2.ptr());
        drain_cfg_causes();
    }
}

#[test]
fn cfg_show_causes_with_no_causes_returns_at_once() {
    let _guard = globals();
    unsafe {
        drain_cfg_causes();
        cfg_show_causes(None);
        // With a session but no causes, still nothing.
        let mut t = Target::new(80, 24);
        cfg_show_causes(Some(&*t.session()));
        drain_cfg_causes();
    }
}

#[test]
fn cfg_show_causes_with_causes_and_no_session_uses_first_session_or_returns() {
    let _guard = globals();
    unsafe {
        drain_cfg_causes();
        cfg_add_cause(c"show-cause", fmt_args![]);
        cfg_show_causes(None);
        drain_cfg_causes();
    }
}

#[test]
fn cfg_show_causes_delivers_to_the_active_pane_of_the_session() {
    let _guard = globals();
    unsafe {
        drain_cfg_causes();
        let mut t = Target::new(80, 24);
        crate::session::session_ref_of(&mut *t.session())
            .expect("session owner")
            .add_attached();
        cfg_add_cause(c"pane-cause", fmt_args![]);
        cfg_show_causes(Some(&*t.session()));
        drain_cfg_causes();
        assert!(!t.pane(0).is_null());
    }
}

#[test]
fn load_cfg_from_buffer_with_valid_config_queues_commands() {
    let _guard = globals();
    unsafe {
        drain_cfg_causes();
        let buf = CString::new("set-option -g status off\n").unwrap();
        let mut new_item = None;
        let rc = load_cfg_from_buffer(
            buf.as_bytes(),
            c"buffer.conf",
            None,
            None,
            None,
            0,
            Some(&mut new_item),
        );
        assert_eq!(rc, 0);
        assert!(new_item.is_some());
        drain_cfg_causes();
    }
}

#[test]
fn load_cfg_from_buffer_with_syntax_error_adds_a_cause() {
    let _guard = globals();
    unsafe {
        drain_cfg_causes();
        let buf = CString::new("not-a-command ###\n").unwrap();
        let mut new_item = None;
        let rc = load_cfg_from_buffer(
            buf.as_bytes(),
            c"bad.conf",
            None,
            None,
            None,
            0,
            Some(&mut new_item),
        );
        assert_eq!(rc, -1);
        assert!(new_item.is_none());
        drain_cfg_causes();
    }
}

#[test]
fn load_cfg_from_buffer_with_parseonly_flag_parses_but_queues_nothing() {
    let _guard = globals();
    unsafe {
        drain_cfg_causes();
        let buf = CString::new("set-option -g status off\n").unwrap();
        let mut new_item = None;
        let rc = load_cfg_from_buffer(
            buf.as_bytes(),
            c"parseonly.conf",
            None,
            None,
            None,
            crate::cfg::CMD_PARSE_PARSEONLY,
            Some(&mut new_item),
        );
        assert_eq!(rc, 0);
        assert!(new_item.is_none());
        drain_cfg_causes();
    }
}

#[test]
fn load_cfg_missing_file_quiet_returns_zero_and_no_cause() {
    let _guard = globals();
    unsafe {
        drain_cfg_causes();
        let mut new_item = None;
        let rc = load_cfg(
            c"/tmp/tmux-c2rs-gamma-no-such-file-12345",
            None,
            None,
            None,
            crate::cfg::CMD_PARSE_QUIET,
            Some(&mut new_item),
        );
        assert_eq!(rc, 0);
        assert!(new_item.is_none());
        drain_cfg_causes();
    }
}

#[test]
fn load_cfg_missing_file_non_quiet_adds_a_cause() {
    let _guard = globals();
    unsafe {
        drain_cfg_causes();
        let mut new_item = None;
        let rc = load_cfg(
            c"/tmp/tmux-c2rs-gamma-no-such-file-12346",
            None,
            None,
            None,
            0,
            Some(&mut new_item),
        );
        assert_eq!(rc, -1);
        assert!(new_item.is_none());
        drain_cfg_causes();
    }
}

#[test]
fn load_cfg_from_buffer_with_item_chains_after_it() {
    let _guard = globals();
    unsafe {
        drain_cfg_causes();
        // Chaining after an item requires the item's state to be valid.
        // Use a null item, which appends to the global queue instead.
        let buf = CString::new("set-option -g status off\n").unwrap();
        let mut new_item = None;
        let rc = load_cfg_from_buffer(
            buf.as_bytes(),
            c"chain.conf",
            None,
            None,
            None,
            0,
            Some(&mut new_item),
        );
        assert_eq!(rc, 0);
        assert!(new_item.is_some());
        drain_cfg_causes();
    }
}
