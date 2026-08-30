//! Coverage for [`crate::overlay`] and [`crate::overlay`] — constants and
//! lightweight helpers that need no live server.
//!
//! `popup.rs` is at 0.75% line coverage and `menu.rs` at ~15%. Both files
//! start with a long block of `pub const` definitions that are exercised here
//! purely by value, plus a handful of helpers that operate on fixtures alone:
//! [`popup_present`] on a plain client and [`menu_create`] on heap-allocated
//! menus. All tests are deterministic and stay clear of the
//! `fatal`/`fatalx` paths.

use crate::overlay::{
    BOX_LINES_DEFAULT as MENU_BOX_DEFAULT, BOX_LINES_DOUBLE as MENU_BOX_DOUBLE,
    BOX_LINES_HEAVY as MENU_BOX_HEAVY, BOX_LINES_NONE as MENU_BOX_NONE,
    BOX_LINES_PADDED as MENU_BOX_PADDED, BOX_LINES_ROUNDED as MENU_BOX_ROUNDED,
    BOX_LINES_SIMPLE as MENU_BOX_SIMPLE, BOX_LINES_SINGLE as MENU_BOX_SINGLE, MENU_NOMOUSE,
    MENU_STAYOPEN, MENU_TAB, menu_add_item, menu_add_items, menu_check_cb, menu_create,
    menu_mode_cb,
};
use crate::overlay::{
    BOX_LINES_DEFAULT as POPUP_BOX_DEFAULT, BOX_LINES_DOUBLE as POPUP_BOX_DOUBLE,
    BOX_LINES_HEAVY as POPUP_BOX_HEAVY, BOX_LINES_NONE as POPUP_BOX_NONE,
    BOX_LINES_PADDED as POPUP_BOX_PADDED, BOX_LINES_ROUNDED as POPUP_BOX_ROUNDED,
    BOX_LINES_SIMPLE as POPUP_BOX_SIMPLE, BOX_LINES_SINGLE as POPUP_BOX_SINGLE, MSG_COMMAND,
    MSG_DETACH, MSG_DETACHKILL, MSG_EXIT, MSG_EXITED, MSG_EXITING, MSG_FLAGS, MSG_IDENTIFY_CWD,
    MSG_IDENTIFY_DONE, MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_TERM, MSG_LOCK, MSG_READ, MSG_READ_DONE,
    MSG_READ_OPEN, MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN, MSG_VERSION, MSG_WRITE,
    MSG_WRITE_CLOSE, MSG_WRITE_OPEN, MSG_WRITE_READY, PANE_LINES_DOUBLE, PANE_LINES_HEAVY,
    PANE_LINES_NUMBER, PANE_LINES_SIMPLE, PANE_LINES_SINGLE, PANE_LINES_SPACES, PROGRESS_BAR_ERROR,
    PROGRESS_BAR_HIDDEN, PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED,
    SCREEN_CURSOR_BAR, SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT, SCREEN_CURSOR_UNDERLINE,
    STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT, STYLE_ALIGN_LEFT,
    STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE, STYLE_RANGE_PANE,
    STYLE_RANGE_RIGHT, popup_present,
};
use crate::overlay::{
    CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN, PROMPT_COMMAND, PROMPT_ENTRY,
    PROMPT_TYPE_COMMAND, PROMPT_TYPE_SEARCH,
};
use crate::tests::test_fixtures::{Clients, globals, seen};
use crate::types::*;
use ::core::ptr::null_mut;

// ---------------------------------------------------------------------------
// Popup / menu message constants — stable wire values
// ---------------------------------------------------------------------------

#[test]
fn popup_msg_constants_match_upstream_wire_values() {
    // version + identify block
    assert_eq!(MSG_VERSION, 12);
    assert_eq!(MSG_IDENTIFY_FLAGS, 100);
    assert_eq!(MSG_IDENTIFY_TERM, 101);
    assert_eq!(MSG_IDENTIFY_CWD, 108);
    assert_eq!(MSG_IDENTIFY_DONE, 106);
    // command block
    assert_eq!(MSG_COMMAND, 200);
    assert_eq!(MSG_DETACH, 201);
    assert_eq!(MSG_DETACHKILL, 202);
    assert_eq!(MSG_EXIT, 203);
    assert_eq!(MSG_EXITED, 204);
    assert_eq!(MSG_EXITING, 205);
    assert_eq!(MSG_LOCK, 206);
    assert_eq!(MSG_READY, 207);
    assert_eq!(MSG_RESIZE, 208);
    assert_eq!(MSG_SHELL, 209);
    assert_eq!(MSG_SHUTDOWN, 210);
    assert_eq!(MSG_FLAGS, 218);
    // read/write block
    assert_eq!(MSG_READ_OPEN, 300);
    assert_eq!(MSG_READ, 301);
    assert_eq!(MSG_READ_DONE, 302);
    assert_eq!(MSG_WRITE_OPEN, 303);
    assert_eq!(MSG_WRITE, 304);
    assert_eq!(MSG_WRITE_READY, 305);
    assert_eq!(MSG_WRITE_CLOSE, 306);
    // ordering invariants
    assert!(MSG_COMMAND < MSG_FLAGS);
    assert!(MSG_FLAGS < MSG_READ_OPEN);
    assert!(MSG_READ_OPEN < MSG_WRITE_CLOSE);
}

#[test]
fn popup_and_menu_share_box_line_values() {
    assert_eq!(POPUP_BOX_SINGLE, MENU_BOX_SINGLE);
    assert_eq!(POPUP_BOX_DOUBLE, MENU_BOX_DOUBLE);
    assert_eq!(POPUP_BOX_HEAVY, MENU_BOX_HEAVY);
    assert_eq!(POPUP_BOX_SIMPLE, MENU_BOX_SIMPLE);
    assert_eq!(POPUP_BOX_ROUNDED, MENU_BOX_ROUNDED);
    assert_eq!(POPUP_BOX_PADDED, MENU_BOX_PADDED);
    assert_eq!(POPUP_BOX_NONE, MENU_BOX_NONE);
    assert_eq!(POPUP_BOX_DEFAULT, MENU_BOX_DEFAULT);
    assert_eq!(POPUP_BOX_DEFAULT, -1);
    assert_eq!(POPUP_BOX_SINGLE, 0);
    assert_ne!(POPUP_BOX_SINGLE, POPUP_BOX_NONE);
    // ladder is strict then wraps to default sentinel
    assert!(POPUP_BOX_SINGLE < POPUP_BOX_DOUBLE);
    assert!(POPUP_BOX_DOUBLE < POPUP_BOX_HEAVY);
    assert!(POPUP_BOX_NONE == 6);
}

// ---------------------------------------------------------------------------
// Pane / progress / cursor / style constants
// ---------------------------------------------------------------------------

#[test]
fn pane_lines_constants_cover_all_variants() {
    assert_eq!(PANE_LINES_SINGLE, 0);
    assert_eq!(PANE_LINES_DOUBLE, 1);
    assert_eq!(PANE_LINES_HEAVY, 2);
    assert_eq!(PANE_LINES_SIMPLE, 3);
    assert_eq!(PANE_LINES_NUMBER, 4);
    assert_eq!(PANE_LINES_SPACES, 5);
    let all = [
        PANE_LINES_SINGLE,
        PANE_LINES_DOUBLE,
        PANE_LINES_HEAVY,
        PANE_LINES_SIMPLE,
        PANE_LINES_NUMBER,
        PANE_LINES_SPACES,
    ];
    for (i, &v) in all.iter().enumerate() {
        assert_eq!(v, i as u32);
    }
}

#[test]
fn progress_bar_and_cursor_constants_are_consecutive() {
    assert_eq!(PROGRESS_BAR_HIDDEN, 0);
    assert_eq!(PROGRESS_BAR_NORMAL, 1);
    assert_eq!(PROGRESS_BAR_ERROR, 2);
    assert_eq!(PROGRESS_BAR_INDETERMINATE, 3);
    assert_eq!(PROGRESS_BAR_PAUSED, 4);

    assert_eq!(SCREEN_CURSOR_DEFAULT, 0);
    assert_eq!(SCREEN_CURSOR_BLOCK, 1);
    assert_eq!(SCREEN_CURSOR_UNDERLINE, 2);
    assert_eq!(SCREEN_CURSOR_BAR, 3);
}

#[test]
fn style_align_and_default_constants_are_ladders() {
    assert_eq!(STYLE_ALIGN_DEFAULT, 0);
    assert_eq!(STYLE_ALIGN_LEFT, 1);
    assert_eq!(STYLE_ALIGN_CENTRE, 2);
    assert_eq!(STYLE_ALIGN_RIGHT, 3);
    assert_eq!(STYLE_ALIGN_ABSOLUTE_CENTRE, 4);

    assert_eq!(STYLE_DEFAULT_BASE, 0);
    assert_eq!(STYLE_DEFAULT_PUSH, 1);
    assert_eq!(STYLE_DEFAULT_POP, 2);
    assert_eq!(STYLE_DEFAULT_SET, 3);

    assert_eq!(STYLE_RANGE_NONE, 0);
    assert_eq!(STYLE_RANGE_LEFT, 1);
    assert_eq!(STYLE_RANGE_RIGHT, 2);
    assert_eq!(STYLE_RANGE_PANE, 3);
    assert_eq!(STYLE_RANGE_CONTROL, 7);
}

#[test]
fn prompt_and_client_exit_constants_match_headers() {
    assert_eq!(PROMPT_TYPE_COMMAND, 0);
    assert_eq!(PROMPT_TYPE_SEARCH, 1);
    assert_eq!(PROMPT_ENTRY, 0);
    assert_eq!(PROMPT_COMMAND, 1);
    assert_eq!(CLIENT_EXIT_RETURN, 0);
    assert_eq!(CLIENT_EXIT_SHUTDOWN, 1);
    assert_eq!(CLIENT_EXIT_DETACH, 2);

    assert_eq!(MENU_NOMOUSE, 0x1);
    assert_eq!(MENU_TAB, 0x2);
    assert_eq!(MENU_STAYOPEN, 0x4);
    assert_eq!(MENU_NOMOUSE | MENU_TAB | MENU_STAYOPEN, 0x7);
    assert_eq!((MENU_NOMOUSE & MENU_TAB), 0);
}

// ---------------------------------------------------------------------------
// Lightweight helpers — no server, just fixtures or pure memory
// ---------------------------------------------------------------------------

#[test]
fn popup_present_is_zero_for_a_plain_client() {
    let _guard = globals();
    let mut clients = Clients::new();
    let c = clients.add("plain-auto05", 80, 24);
    unsafe {
        assert_eq!(popup_present(c), 0);
        assert!((*c).overlay_data().is_none());
        assert!((*c).overlay().is_none());
    }
}

#[test]
fn menu_create_and_free_roundtrip_with_title() {
    let _guard = globals();
    unsafe {
        let title = c"auto05".as_ptr();
        let m = menu_create(title);
        assert_eq!(m.items.len(), 0);
        assert!(m.title.is_some());
        assert_eq!(seen(cstr_ptr(&m.title)), "auto05");
        // width is format_width of the title; "auto05" is 6 cells
        assert_eq!(m.width, 6);
        assert!(m.items.is_empty());
    }
}

#[test]
fn menu_with_an_empty_title_works() {
    let _guard = globals();
    unsafe {
        let m = menu_create(c"".as_ptr());
        assert_eq!(m.items.len(), 0);
        assert_eq!(seen(cstr_ptr(&m.title)), "");
        assert_eq!(m.width, 0);
    }
}

#[test]
fn menu_create_width_tracks_title_length() {
    let _guard = globals();
    unsafe {
        let short = menu_create(c"hi".as_ptr());
        let long = menu_create(c"hello world".as_ptr());
        assert!(long.width > short.width);
        assert_eq!(short.width, 2);
        assert_eq!(long.width, 11);
    }
}

#[test]
fn test_menu_add_item_and_items() {
    let _guard = globals();
    unsafe {
        let mut clients = Clients::new();
        let c = clients.add("c", 80, 24);

        let mut m = menu_create(c"test-menu".as_ptr());

        let item1 = menu_item {
            name: Some(c"Item 1"),
            key: b'a' as key_code,
            command: Some(c"cmd 1"),
        };
        menu_add_item(&raw mut *m, Some(&item1), null_mut(), c, null_mut());
        assert_eq!(m.items.len(), 1);

        // Separator
        let sep = menu_item {
            name: None,
            key: 0,
            command: None,
        };
        menu_add_item(&raw mut *m, Some(&sep), null_mut(), c, null_mut());
        assert_eq!(m.items.len(), 2);

        // Multiple items
        let items = [menu_item {
            name: Some(c"Item 2"),
            key: 0,
            command: None,
        }];
        menu_add_items(&raw mut *m, &items, null_mut(), c, null_mut());
        assert_eq!(m.items.len(), 3);
    }
}

#[test]
fn test_menu_callbacks() {
    let _guard = globals();
    unsafe {
        let mut clients = Clients::new();
        let c = clients.add("c", 80, 24);

        let mut m = menu_create(c"menu".as_ptr());
        let item1 = menu_item {
            name: Some(c"One"),
            key: 0,
            command: None,
        };
        menu_add_item(&raw mut *m, Some(&item1), null_mut(), c, null_mut());

        let mut s = screen::default();
        let mut md = menu_data {
            item: None,
            flags: 0,
            style: None,
            border_style: None,
            selected_style: None,
            style_gc: crate::grid::grid_default_cell,
            border_style_gc: crate::grid::grid_default_cell,
            selected_style_gc: crate::grid::grid_default_cell,
            border_lines: MENU_BOX_DEFAULT,
            fs: cmd_find_state::default(),
            s,
            r: visible_ranges {
                ranges: Vec::new(),
                used: 0,
            },
            px: 10,
            py: 5,
            menu: m,
            choice: 0,
            cb: None,
            data: MenuCallbackData::None,
        };

        let (s, cx, cy) = menu_mode_cb(c, &raw mut md);
        assert!(!s.is_null());
        assert_eq!(cx, 12);
        assert_eq!(cy, 6);

        // choice = -1
        md.choice = -1;
        let (_, _, cy) = menu_mode_cb(c, &raw mut md);
        assert_eq!(cy, 5);

        let vr = menu_check_cb(c, &raw mut md, 0, 0, 80);
        assert!(!vr.is_null());
    }
}
