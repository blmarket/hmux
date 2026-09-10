//! Coverage for [`crate::server`] and [`crate::status`] pure helpers.
//!
//! `server.rs` stores the marked pane in a process-wide `marked_pane` and
//! `status.rs` keeps prompt-type strings and status sizing helpers. All
//! helpers below are deterministic, avoid the fatal/daemon paths and use
//! [`globals`] when touching globals or option trees.

use crate::fmt_args;
use crate::options::OptionsRef;

use crate::server::{
    MSG_COMMAND, MSG_FLAGS, MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE, MSG_IDENTIFY_FLAGS,
    MSG_IDENTIFY_TERM, MSG_READ, MSG_READ_DONE, MSG_READ_OPEN, MSG_VERSION, MSG_WRITE,
    MSG_WRITE_CLOSE, MSG_WRITE_OPEN, PANE_LINES_DOUBLE, PANE_LINES_SINGLE, server_add_message,
    server_check_marked, server_clear_marked, server_is_marked, server_set_marked,
};

use crate::status::{
    CLIENT_CONTROL, CLIENT_STATUSOFF, PROMPT_NTYPES, PROMPT_TYPE_COMMAND, PROMPT_TYPE_INVALID,
    PROMPT_TYPE_SEARCH, PROMPT_TYPE_TARGET, PROMPT_TYPE_WINDOW_TARGET, status_at_line,
    status_line_size, status_prompt_line_at, status_prompt_type, status_prompt_type_string,
};
use crate::tests::test_fixtures::{Clients, Target, globals};

// ---------------------------------------------------------------------------
// Constants — wire values are stable
// ---------------------------------------------------------------------------

#[test]
fn server_and_status_wire_constants_match_upstream() {
    assert_eq!(MSG_VERSION, 12);
    assert_eq!(MSG_IDENTIFY_FLAGS, 100);
    assert_eq!(MSG_IDENTIFY_TERM, 101);
    assert_eq!(MSG_IDENTIFY_CWD, 108);
    assert_eq!(MSG_IDENTIFY_DONE, 106);
    assert_eq!(MSG_COMMAND, 200);
    assert_eq!(MSG_FLAGS, 218);
    assert_eq!(MSG_READ_OPEN, 300);
    assert_eq!(MSG_READ, 301);
    assert_eq!(MSG_READ_DONE, 302);
    assert_eq!(MSG_WRITE_OPEN, 303);
    assert_eq!(MSG_WRITE, 304);
    assert_eq!(MSG_WRITE_CLOSE, 306);
    assert!(MSG_COMMAND < MSG_FLAGS);
    assert!(MSG_FLAGS < MSG_READ_OPEN);

    assert_eq!(PANE_LINES_SINGLE, 0);
    assert_eq!(PANE_LINES_DOUBLE, 1);
    assert_eq!(PROMPT_NTYPES, 4);
    assert_eq!(PROMPT_TYPE_COMMAND, 0);
    assert_eq!(PROMPT_TYPE_SEARCH, 1);
    assert_eq!(PROMPT_TYPE_TARGET, 2);
    assert_eq!(PROMPT_TYPE_WINDOW_TARGET, 3);
    assert_eq!(PROMPT_TYPE_INVALID, 255);
}

// ---------------------------------------------------------------------------
// server marked-pane helpers — use Target so session/winlink/pane are real
// ---------------------------------------------------------------------------

#[test]
fn server_marked_pane_set_check_is_and_clear() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    unsafe {
        // initially nothing is marked
        server_clear_marked();
        assert_eq!(server_check_marked(), 0);

        let s = target.session();
        let wl = target.winlink(0);
        let wp = target.pane(0);
        server_set_marked(s.as_ref(), wl.as_ref(), wp.as_ref());
        assert_eq!(server_check_marked(), 1);
        assert_eq!(server_is_marked(s.as_ref(), wl.as_ref(), wp.as_ref()), 1);

        // different pane is not marked
        let mut other_pane_box = Box::new(crate::types::window_pane::default());
        let other_wp = &raw mut *other_pane_box;
        assert_eq!(
            server_is_marked(s.as_ref(), wl.as_ref(), other_wp.as_ref()),
            0
        );

        // null args never marked
        assert_eq!(server_is_marked(None, wl.as_ref(), wp.as_ref()), 0);
        assert_eq!(server_is_marked(s.as_ref(), None, wp.as_ref()), 0);
        assert_eq!(
            server_is_marked(s.as_ref(), wl.as_ref(), None::<&crate::types::window_pane>),
            0
        );

        server_clear_marked();
        assert_eq!(server_check_marked(), 0);
        assert_eq!(server_is_marked(s.as_ref(), wl.as_ref(), wp.as_ref()), 0);
    }
}

#[test]
fn server_marked_pane_survives_null_session_window() {
    let _guard = globals();
    unsafe {
        server_clear_marked();
        // setting with nulls still stores them; check returns 0 because valid_state fails
        server_set_marked(None, None, None::<&crate::types::window_pane>);
        assert_eq!(server_check_marked(), 0);
        // is_marked with nulls returns 0 regardless
        assert_eq!(
            server_is_marked(None, None, None::<&crate::types::window_pane>),
            0
        );
        server_clear_marked();
    }
}

// ---------------------------------------------------------------------------
// server_add_message — appends to message_log, respects message-limit
// ---------------------------------------------------------------------------

#[test]
fn server_add_message_appends_entries() {
    let _guard = globals();
    unsafe {
        let before = message_count();
        server_add_message(c"auto06 %s", fmt_args![c"hello".as_ptr()]);
        server_add_message(c"auto06 second", fmt_args![]);
        let after = message_count();
        assert_eq!(after, before + 2);
        // last message text contains our prefix
        let txt = crate::tests::test_fixtures::message_log_entries()
            .into_iter()
            .next()
            .map(|entry| String::from_utf8_lossy(entry.text.as_bytes()).into_owned())
            .expect("the two lines just recorded are still there");
        assert!(txt.contains("auto06"), "last msg was {txt:?}");
    }
}

fn message_count() -> usize {
    crate::tests::test_fixtures::message_log_count()
}

// ---------------------------------------------------------------------------
// status_prompt_type helpers — pure string <-> type mapping
// ---------------------------------------------------------------------------

#[test]
fn status_prompt_type_roundtrips_all_known_strings() {
    assert_eq!(status_prompt_type_string(0), c"command");
    assert_eq!(status_prompt_type_string(1), c"search");
    assert_eq!(status_prompt_type_string(2), c"target");
    assert_eq!(status_prompt_type_string(3), c"window-target");
    // out of range returns "invalid"
    assert_eq!(status_prompt_type_string(4), c"invalid");
    assert_eq!(status_prompt_type_string(99), c"invalid");
    assert_eq!(status_prompt_type_string(255), c"invalid");

    assert_eq!(status_prompt_type(c"command"), PROMPT_TYPE_COMMAND);
    assert_eq!(status_prompt_type(c"search"), PROMPT_TYPE_SEARCH);
    assert_eq!(status_prompt_type(c"target"), PROMPT_TYPE_TARGET);
    assert_eq!(
        status_prompt_type(c"window-target"),
        PROMPT_TYPE_WINDOW_TARGET
    );
    assert_eq!(status_prompt_type(c"invalid"), PROMPT_TYPE_INVALID);
    assert_eq!(status_prompt_type(c"unknown"), PROMPT_TYPE_INVALID);
    assert_eq!(status_prompt_type(c""), PROMPT_TYPE_INVALID);
}

// ---------------------------------------------------------------------------
// status sizing helpers — status_update_cache, status_line_size, at_line
// ---------------------------------------------------------------------------

#[test]
fn status_update_cache_sets_statusat_from_options() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    unsafe {
        let s = target.session();
        // default from Target is whatever session defaults set; force status off then on
        ((&mut *s).options_ref().clone()).set_number(c"status", 0);
        (target.session_handle()).update_status_cache();
        assert_eq!((*s).statuslines, 0);
        assert_eq!((*s).statusat, -1);

        ((&mut *s).options_ref().clone()).set_number(c"status", 2);
        ((&mut *s).options_ref().clone()).set_number(c"status-position", 0);
        (target.session_handle()).update_status_cache();
        assert_eq!((*s).statuslines, 2);
        assert_eq!((*s).statusat, 0);

        ((&mut *s).options_ref().clone()).set_number(c"status-position", 1);
        (target.session_handle()).update_status_cache();
        assert_eq!((*s).statuslines, 2);
        assert_eq!((*s).statusat, 1);

        // restore
        ((&mut *s).options_ref().clone()).set_number(c"status", 1);
        ((&mut *s).options_ref().clone()).set_number(c"status-position", 0);
        (target.session_handle()).update_status_cache();
    }
}

#[test]
fn status_line_size_and_at_line_with_flags() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    let mut clients = Clients::new();
    unsafe {
        let s = target.session();
        // ensure cache is consistent: 1 line at top
        ((&mut *s).options_ref().clone()).set_number(c"status", 1);
        ((&mut *s).options_ref().clone()).set_number(c"status-position", 0);
        (target.session_handle()).update_status_cache();

        let c = clients.add("auto06-status", 80, 24);
        (*c).set_attached_session(Some(target.session_handle()));

        // normal: line_size is session's statuslines
        assert_eq!(status_line_size(&*c), 1);
        // top position -> at_line is session statusat (0)
        assert_eq!(status_at_line(&*c), 0);

        // status at bottom -> line is sy - lines
        ((&mut *s).options_ref().clone()).set_number(c"status-position", 1);
        (target.session_handle()).update_status_cache();
        (*c).tty.sy = 24;
        assert_eq!(status_at_line(&*c), 23); // 24 - 1

        // flagged off -> 0 / -1
        (*c).flags |= CLIENT_STATUSOFF as u64;
        assert_eq!(status_line_size(&*c), 0);
        assert_eq!(status_at_line(&*c), -1);
        (*c).flags &= !(CLIENT_STATUSOFF as u64);

        (*c).flags |= CLIENT_CONTROL as u64;
        assert_eq!(status_line_size(&*c), 0);
        assert_eq!(status_at_line(&*c), -1);
        (*c).flags &= !(CLIENT_CONTROL as u64);

        // null session -> falls back to global_s_options "status"
        (*c).set_attached_session(None);
        let expected = (crate::tmux::global_s_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .number(c"status") as u32;
        assert_eq!(status_line_size(&*c), expected);

        // restore
        (*c).set_attached_session(Some(target.session_handle()));
        ((&mut *s).options_ref().clone()).set_number(c"status-position", 0);
        (target.session_handle()).update_status_cache();
    }
}

#[test]
fn status_prompt_line_at_clamps_to_lines() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    let mut clients = Clients::new();
    unsafe {
        let s = target.session();
        ((&mut *s).options_ref().clone()).set_number(c"status", 3);
        ((&mut *s).options_ref().clone()).set_number(c"message-line", 1);
        (target.session_handle()).update_status_cache();

        let c = clients.add("auto06-prompt-line", 80, 24);
        (*c).set_attached_session(Some(target.session_handle()));
        (*c).tty.sy = 24;

        // message-line within range
        assert_eq!(status_prompt_line_at(&*c), 1);

        // clamped when message-line >= lines
        ((&mut *s).options_ref().clone()).set_number(c"message-line", 10);
        assert_eq!(status_prompt_line_at(&*c), 2); // lines -1

        // zero lines -> 0
        ((&mut *s).options_ref().clone()).set_number(c"status", 0);
        (target.session_handle()).update_status_cache();
        assert_eq!(status_prompt_line_at(&*c), 0);

        // restore
        ((&mut *s).options_ref().clone()).set_number(c"status", 1);
        ((&mut *s).options_ref().clone()).set_number(c"message-line", 0);
        (target.session_handle()).update_status_cache();
    }
}

#[test]
fn status_get_range_returns_null_for_out_of_bounds_y() {
    let _guard = globals();
    let mut clients = Clients::new();
    unsafe {
        let c = clients.add("auto06-range", 80, 24);
        // the fixture's entries hold empty range lists; status_get_range checks y < 5
        let r = crate::status::status_get_range(&*c, 0, 5);
        assert!(r.is_none());
        let r2 = crate::status::status_get_range(&*c, 0, 10);
        assert!(r2.is_none());
        // y=0 is within bounds but still returns null when no ranges set
        let r0 = crate::status::status_get_range(&*c, 0, 0);
        assert!(r0.is_none());
    }
}
