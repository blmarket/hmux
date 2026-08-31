//! Unit tests for [`crate::cmd::cmd_display_menu`] — the `display-menu` and
//! `display-popup` command entries, the message-protocol and display constants
//! the file re-declares, the argument-parse callback that classifies
//! `name [key] [command]` triples for the parser, and every deterministic
//! branch of both exec hooks the fixtures can reach without a daemon, a real
//! terminal or a spawned process.
//!
//! Exec is reached through each entry's own function pointer, exactly as the
//! command queue calls it, over items whose arguments come from the real
//! command parser and whose targets are resolved against a registered
//! [`Target`]. Refusals report through `cmdq_error` onto client-less items,
//! which only records a cause in `cfg.rs`'s private list where no test can
//! read or free it — the pattern the other suites already accept — so every
//! error test aims the item's target client at a standalone fixture client
//! while leaving the item itself client-less.
//!
//! A menu that passes its size checks ends in `menu_display`, which parks a
//! private [`menu_data`] on the target client as an overlay and answers wait.
//! Those tests read the parked data back — flags, border choice, starting
//! choice, computed position and the built menu itself — and then take the
//! overlay off again with [`server_client_clear_overlay`], which runs
//! boxed menu teardown callback: the screen, the styles and the menu are freed and the
//! waiting item is continued, so nothing dangles when the fixtures go away.
//! Three places stay out of reach, deliberately: `popup_display` spawns its
//! shell command as a job against a real process, `popup_modify` dereferences
//! live popup data that only a running popup owns, and the menu key and draw
//! callbacks want a real redraw cycle over a connected terminal.

use crate::cmd::cmd_display_menu::{
    __INT_MAX__, _PATH_BSHELL, ARGS_PARSE_COMMANDS, ARGS_PARSE_COMMANDS_OR_STRING,
    ARGS_PARSE_INVALID, ARGS_PARSE_STRING, BOX_LINES_DEFAULT, BOX_LINES_DOUBLE, BOX_LINES_HEAVY,
    BOX_LINES_NONE, BOX_LINES_PADDED, BOX_LINES_ROUNDED, BOX_LINES_SIMPLE, BOX_LINES_SINGLE,
    CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN, CMD_AFTERHOOK, CMD_CLIENT_CFLAG,
    CMD_FIND_PANE, CMD_FIND_SESSION, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
    CMD_RETURN_STOP, CMD_RETURN_WAIT, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE,
    MENU_NOMOUSE, MENU_STAYOPEN, MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL, MSG_EXEC, MSG_EXIT,
    MSG_EXITED, MSG_EXITING, MSG_FLAGS, MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD,
    MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON, MSG_IDENTIFY_FEATURES, MSG_IDENTIFY_FLAGS,
    MSG_IDENTIFY_LONGFLAGS, MSG_IDENTIFY_OLDCWD, MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT,
    MSG_IDENTIFY_TERM, MSG_IDENTIFY_TERMINFO, MSG_IDENTIFY_TTYNAME, MSG_LOCK, MSG_OLDSTDERR,
    MSG_OLDSTDIN, MSG_OLDSTDOUT, MSG_READ, MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN,
    MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN, MSG_SUSPEND, MSG_UNLOCK, MSG_VERSION,
    MSG_WAKEUP, MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_OPEN, MSG_WRITE_READY, OPTIONS_TABLE_CHOICE,
    OPTIONS_TABLE_COLOUR, OPTIONS_TABLE_COMMAND, OPTIONS_TABLE_FLAG, OPTIONS_TABLE_KEY,
    OPTIONS_TABLE_NUMBER, OPTIONS_TABLE_STRING, PANE_LINES_DOUBLE, PANE_LINES_HEAVY,
    PANE_LINES_NUMBER, PANE_LINES_SIMPLE, PANE_LINES_SINGLE, PANE_LINES_SPACES, POPUP_CLOSEANYKEY,
    POPUP_CLOSEEXIT, POPUP_CLOSEEXITZERO, PROGRESS_BAR_ERROR, PROGRESS_BAR_HIDDEN,
    PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED, PROMPT_COMMAND,
    PROMPT_ENTRY, PROMPT_TYPE_COMMAND, PROMPT_TYPE_INVALID, PROMPT_TYPE_SEARCH, PROMPT_TYPE_TARGET,
    PROMPT_TYPE_WINDOW_TARGET, SCREEN_CURSOR_BAR, SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT,
    SCREEN_CURSOR_UNDERLINE, STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT,
    STYLE_ALIGN_LEFT, STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_LIST_FOCUS, STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON,
    STYLE_LIST_RIGHT_MARKER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, UINT_MAX, cmd_display_menu_entry,
    cmd_display_popup_entry,
};
use crate::cmd::cmdq_set_target_client;
use crate::screen::{MODE_CURSOR, MODE_MOUSE_ALL, MODE_MOUSE_BUTTON, screen_grid_ptr};
use crate::server::CLIENT_ALLREDRAWFLAGS;
use crate::server::server_client_clear_overlay;
use crate::tests::test_fixtures::held_item;
use crate::tests::test_fixtures::{Args, Item, Target, globals, seen, zeroed_client};
use crate::text::key_string_lookup_string;
use crate::types::*;
use ::core::ffi::{CStr, c_char};

/// Where the tests' items claim to come from, which is what `cfg_add_cause`
/// would report them under.
const FILE: &CStr = c"test-coverage-cmd-display-menu.conf";

/// The entries whose exec and argument-parse hooks are under test.
const MENU: *const cmd_entry = &raw const cmd_display_menu_entry;
const POPUP: *const cmd_entry = &raw const cmd_display_popup_entry;

/// Runs the parsed command an item carries through `entry`'s exec hook, the
/// way the command queue calls it. The item must be running that entry.
unsafe fn exec_via(entry: *const cmd_entry, item: &mut Item) -> cmd_retval {
    unsafe {
        assert!(
            ::core::ptr::eq((*item.cmd()).entry, entry),
            "the item is not running this entry"
        );
        let exec = (*entry).exec;
        exec(&*item.cmd(), item.ptr())
    }
}

/// An item claiming to come from [`FILE`], carrying a parsed command line.
fn item_for(line: &'static CStr, number: u_int) -> Item {
    Item::new().from_file(FILE, number).with_args(line)
}

/// Aims a client-less item's target client at `tc`, which the caller owns.
/// This is what lets the exec hooks find a client to work on while
/// `cmdq_error` still sees the item itself as client-less.
fn aimed_at(mut item: Item, t: &mut Target, tc: *mut client) -> Item {
    unsafe { cmdq_set_target_client(item.ptr(), tc) };
    item.targeting(t)
}

/// A foreign overlay, standing in for whatever another command parked on the
/// client. It is never drawn; only its presence is observed.
const OVERLAY_SENTINEL: Overlay = Overlay::Menu;

/// A target-client-only fixture: a zeroed client with a name and a terminal
/// of `sx` by `sy`, owned by the test. The pointer is handed out alongside
/// the box so the two cannot drift apart.
fn lone_client(sx: u_int, sy: u_int) -> (ClientRef, *mut client) {
    let mut c = zeroed_client();
    c.name = Some(c"lone-fixture".to_owned());
    c.tty.sx = sx;
    c.tty.sy = sy;
    let p = &raw mut *c;
    (c, p)
}

/// The standard display-menu setup: a registered target and an item owning
/// its own client, attached to the target's session and given a `sx` by `sy`
/// terminal, so the whole display path — status line, formats, overlay — has
/// what it reads.
fn displayed(line: &'static CStr, sx: u_int, sy: u_int) -> (Displayed, cmd_retval) {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut item = Item::with_client()
        .from_file(FILE, 1)
        .with_args(line)
        .targeting(&mut t);
    let tc = item.client();
    unsafe {
        (*tc).name = Some(c"display-fixture".to_owned());
        (*tc).session = t.session();
        (*tc).tty.sx = sx;
        (*tc).tty.sy = sy;
    }
    let rv = unsafe { exec_via(MENU, &mut item) };
    (
        Displayed {
            item,
            t,
            tc,
            _guard,
        },
        rv,
    )
}

/// What [`displayed`] hands back, dropped in the reverse order it was built:
/// the item (and the client it owns) goes first, then the registered target,
/// then the globals turn.
struct Displayed {
    item: Item,
    t: Target,
    tc: *mut client,
    _guard: ::std::sync::MutexGuard<'static, ()>,
}

#[test]
fn the_display_menu_entry_describes_the_display_menu_command() {
    unsafe {
        let e = MENU;
        assert_eq!((*e).name.to_string_lossy(), "display-menu");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "menu"
        );
        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-MO] [-b border-lines] [-c target-client] [-C starting-choice] [-H selected-style] [-s style] [-S border-style] [-t target-pane] [-T title] [-x position] [-y position] name [key] [command] ..."
        );
        assert_eq!(
            (*e).args.template.to_string_lossy(),
            "b:c:C:H:s:S:MOt:T:x:y:"
        );
        assert_eq!((*e).args.lower, 1);
        assert_eq!((*e).args.upper, -1);
        assert!((*e).args.cb.is_some());

        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, 't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, 0);

        assert_eq!((*e).flags, CMD_AFTERHOOK | CMD_CLIENT_CFLAG);
        assert_eq!((*e).flags, 0xc);
    }
}

#[test]
fn the_display_popup_entry_describes_the_display_popup_command() {
    unsafe {
        let e = POPUP;
        assert_eq!((*e).name.to_string_lossy(), "display-popup");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "popup"
        );
        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-BCEkN] [-b border-lines] [-c target-client] [-d start-directory] [-e environment] [-h height] [-s style] [-S border-style] [-t target-pane] [-T title] [-w width] [-x position] [-y position] [shell-command [argument ...]]"
        );
        assert_eq!(
            (*e).args.template.to_string_lossy(),
            "Bb:Cc:d:e:Eh:kNs:S:t:T:w:x:y:"
        );
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, -1);
        assert!(
            (*e).args.cb.is_none(),
            "display-popup classifies no arguments"
        );

        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, 't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, 0);

        assert_eq!((*e).flags, CMD_AFTERHOOK | CMD_CLIENT_CFLAG);
    }
}

#[test]
fn the_parser_resolves_both_names_and_aliases_to_these_entries() {
    let _guard = globals();
    unsafe {
        for (i, (line, want)) in [
            (c"display-menu x y z", MENU),
            (c"menu x y z", MENU),
            (c"display-popup", POPUP),
            (c"popup", POPUP),
        ]
        .into_iter()
        .enumerate()
        {
            let mut item = item_for(line, i as u_int + 1);
            assert!(::core::ptr::eq((*item.cmd()).entry, want), "{line:?}");
        }
    }
}

#[test]
fn the_border_flag_and_limit_constants_keep_their_values() {
    for (constant, value) in [
        (BOX_LINES_DEFAULT, -1),
        (BOX_LINES_SINGLE, 0),
        (BOX_LINES_DOUBLE, 1),
        (BOX_LINES_HEAVY, 2),
        (BOX_LINES_SIMPLE, 3),
        (BOX_LINES_ROUNDED, 4),
        (BOX_LINES_PADDED, 5),
        (BOX_LINES_NONE, 6),
    ] {
        assert_eq!(constant, value);
    }
    for (constant, value) in [
        (MENU_NOMOUSE, 0x1),
        (MENU_STAYOPEN, 0x4),
        (POPUP_CLOSEEXIT, 0x1),
        (POPUP_CLOSEEXITZERO, 0x2),
        (POPUP_CLOSEANYKEY, 0x8),
        (CMD_AFTERHOOK, 0x4),
        (CMD_CLIENT_CFLAG, 0x8),
    ] {
        assert_eq!(constant, value);
    }
    assert_eq!(__INT_MAX__, 2147483647);
    assert_eq!(UINT_MAX, 4294967295);
    assert_eq!(_PATH_BSHELL.to_bytes_with_nul(), b"/bin/sh\0");
}

#[test]
fn the_protocol_and_display_constants_keep_their_values() {
    for (constant, value) in [
        (MSG_VERSION, 12),
        (MSG_IDENTIFY_FLAGS, 100),
        (MSG_IDENTIFY_TERM, 101),
        (MSG_IDENTIFY_TTYNAME, 102),
        (MSG_IDENTIFY_OLDCWD, 103),
        (MSG_IDENTIFY_STDIN, 104),
        (MSG_IDENTIFY_ENVIRON, 105),
        (MSG_IDENTIFY_DONE, 106),
        (MSG_IDENTIFY_CLIENTPID, 107),
        (MSG_IDENTIFY_CWD, 108),
        (MSG_IDENTIFY_FEATURES, 109),
        (MSG_IDENTIFY_STDOUT, 110),
        (MSG_IDENTIFY_LONGFLAGS, 111),
        (MSG_IDENTIFY_TERMINFO, 112),
        (MSG_COMMAND, 200),
        (MSG_DETACH, 201),
        (MSG_DETACHKILL, 202),
        (MSG_EXIT, 203),
        (MSG_EXITED, 204),
        (MSG_EXITING, 205),
        (MSG_LOCK, 206),
        (MSG_READY, 207),
        (MSG_RESIZE, 208),
        (MSG_SHELL, 209),
        (MSG_SHUTDOWN, 210),
        (MSG_OLDSTDERR, 211),
        (MSG_OLDSTDIN, 212),
        (MSG_OLDSTDOUT, 213),
        (MSG_SUSPEND, 214),
        (MSG_UNLOCK, 215),
        (MSG_WAKEUP, 216),
        (MSG_EXEC, 217),
        (MSG_FLAGS, 218),
        (MSG_READ_OPEN, 300),
        (MSG_READ, 301),
        (MSG_READ_DONE, 302),
        (MSG_WRITE_OPEN, 303),
        (MSG_WRITE, 304),
        (MSG_WRITE_READY, 305),
        (MSG_WRITE_CLOSE, 306),
        (MSG_READ_CANCEL, 307),
    ] {
        assert_eq!(constant, value);
    }
    for (constant, value) in [
        (PANE_LINES_SINGLE, 0),
        (PANE_LINES_DOUBLE, 1),
        (PANE_LINES_HEAVY, 2),
        (PANE_LINES_SIMPLE, 3),
        (PANE_LINES_NUMBER, 4),
        (PANE_LINES_SPACES, 5),
        (PROGRESS_BAR_HIDDEN, 0),
        (PROGRESS_BAR_NORMAL, 1),
        (PROGRESS_BAR_ERROR, 2),
        (PROGRESS_BAR_INDETERMINATE, 3),
        (PROGRESS_BAR_PAUSED, 4),
        (SCREEN_CURSOR_DEFAULT, 0),
        (SCREEN_CURSOR_BLOCK, 1),
        (SCREEN_CURSOR_UNDERLINE, 2),
        (SCREEN_CURSOR_BAR, 3),
        (STYLE_ALIGN_DEFAULT, 0),
        (STYLE_ALIGN_LEFT, 1),
        (STYLE_ALIGN_CENTRE, 2),
        (STYLE_ALIGN_RIGHT, 3),
        (STYLE_ALIGN_ABSOLUTE_CENTRE, 4),
        (STYLE_LIST_OFF, 0),
        (STYLE_LIST_ON, 1),
        (STYLE_LIST_FOCUS, 2),
        (STYLE_LIST_LEFT_MARKER, 3),
        (STYLE_LIST_RIGHT_MARKER, 4),
        (STYLE_RANGE_NONE, 0),
        (STYLE_RANGE_LEFT, 1),
        (STYLE_RANGE_RIGHT, 2),
        (STYLE_RANGE_PANE, 3),
        (STYLE_RANGE_WINDOW, 4),
        (STYLE_RANGE_SESSION, 5),
        (STYLE_RANGE_USER, 6),
        (STYLE_RANGE_CONTROL, 7),
        (STYLE_DEFAULT_BASE, 0),
        (STYLE_DEFAULT_PUSH, 1),
        (STYLE_DEFAULT_POP, 2),
        (STYLE_DEFAULT_SET, 3),
        (THEME_UNKNOWN, 0),
        (THEME_LIGHT, 1),
        (THEME_DARK, 2),
        (LAYOUT_LEFTRIGHT, 0),
        (LAYOUT_TOPBOTTOM, 1),
        (LAYOUT_WINDOWPANE, 2),
        (OPTIONS_TABLE_STRING, 0),
        (OPTIONS_TABLE_NUMBER, 1),
        (OPTIONS_TABLE_KEY, 2),
        (OPTIONS_TABLE_COLOUR, 3),
        (OPTIONS_TABLE_FLAG, 4),
        (OPTIONS_TABLE_CHOICE, 5),
        (OPTIONS_TABLE_COMMAND, 6),
        (CMD_FIND_PANE, 0),
        (CMD_FIND_WINDOW, 1),
        (CMD_FIND_SESSION, 2),
        (PROMPT_TYPE_COMMAND, 0),
        (PROMPT_TYPE_SEARCH, 1),
        (PROMPT_TYPE_TARGET, 2),
        (PROMPT_TYPE_WINDOW_TARGET, 3),
        (PROMPT_TYPE_INVALID, 255),
        (PROMPT_ENTRY, 0),
        (PROMPT_COMMAND, 1),
        (CLIENT_EXIT_RETURN, 0),
        (CLIENT_EXIT_SHUTDOWN, 1),
        (CLIENT_EXIT_DETACH, 2),
        (ARGS_PARSE_INVALID, 0),
        (ARGS_PARSE_STRING, 1),
        (ARGS_PARSE_COMMANDS_OR_STRING, 2),
        (ARGS_PARSE_COMMANDS, 3),
    ] {
        assert_eq!(constant, value);
    }
    for (constant, value) in [
        (CMD_RETURN_ERROR, -1i32 as cmd_retval),
        (CMD_RETURN_NORMAL, 0),
        (CMD_RETURN_WAIT, 1),
        (CMD_RETURN_STOP, 2),
    ] {
        assert_eq!(constant, value);
    }
}

/// The callback the parser consults for each positional argument walks the
/// line in `name key command` triples: a name and a key stay strings, a
/// command is accepted as commands too.
#[test]
fn the_argument_callback_classifies_name_key_and_command_slots() {
    let _guard = globals();
    unsafe {
        let cb = (*MENU).args.cb.expect("the entry carries its callback");
        let mut cause = None;

        let three = Args::parse(c"display-menu a b c");
        for (idx, want) in [
            (0, ARGS_PARSE_STRING),
            (1, ARGS_PARSE_STRING),
            (2, ARGS_PARSE_COMMANDS_OR_STRING),
        ] {
            assert_eq!(cb(&*three.ptr(), idx, &mut cause), want, "idx {idx}");
        }

        let six = Args::parse(c"display-menu a b c d e f");
        for (idx, want) in [
            (3, ARGS_PARSE_STRING),
            (4, ARGS_PARSE_STRING),
            (5, ARGS_PARSE_COMMANDS_OR_STRING),
        ] {
            assert_eq!(cb(&*six.ptr(), idx, &mut cause), want, "idx {idx}");
        }
    }
}

/// An empty name contributes no triple of its own: the walk steps straight
/// over it, so the following arguments land one slot earlier than their
/// positions suggest.
#[test]
fn an_empty_name_restarts_the_argument_triple_walk() {
    let _guard = globals();
    unsafe {
        let cb = (*MENU).args.cb.expect("the entry carries its callback");
        let mut cause = None;
        let args = Args::parse(c"display-menu \"\" p q r s t");
        for (idx, want) in [
            (0, ARGS_PARSE_STRING),
            (1, ARGS_PARSE_STRING),
            (2, ARGS_PARSE_STRING),
            (3, ARGS_PARSE_COMMANDS_OR_STRING),
            (4, ARGS_PARSE_STRING),
        ] {
            assert_eq!(cb(&*args.ptr(), idx, &mut cause), want, "idx {idx}");
        }
    }
}

/// A client already carrying any overlay is left alone: even before the
/// arguments are looked at, the command answers normal and changes nothing.
#[test]
fn a_busy_client_answers_normal_without_touching_anything() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let (_tc_box, tc) = lone_client(80, 24);
    unsafe {
        (*tc).set_overlay(OVERLAY_SENTINEL, OverlayState::None);
    }
    let mut item = aimed_at(item_for(c"display-menu x y z", 1), &mut t, tc);
    unsafe {
        assert_eq!(exec_via(MENU, &mut item), CMD_RETURN_NORMAL);
        assert_eq!((*tc).overlay(), OVERLAY_SENTINEL);
        assert_eq!((*tc).flags, 0);
        assert!((*t.pane(0)).modes.is_empty());
    }
}

/// A starting choice that is not a number is refused before any menu exists.
#[test]
fn an_unusable_starting_choice_is_an_error() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let (_tc_box, tc) = lone_client(80, 24);
    let mut item = aimed_at(item_for(c"display-menu -C bogus x y z", 1), &mut t, tc);
    unsafe {
        assert_eq!(exec_via(MENU, &mut item), CMD_RETURN_ERROR);
        assert!((*tc).overlay().is_none(), "an overlay was installed");
    }
}

/// A name with neither a key nor a command behind it is refused, again before
/// anything is displayed.
#[test]
fn a_name_without_key_and_command_is_an_error() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let (_tc_box, tc) = lone_client(80, 24);
    let mut item = aimed_at(item_for(c"display-menu lonely", 2), &mut t, tc);
    unsafe {
        assert_eq!(exec_via(MENU, &mut item), CMD_RETURN_ERROR);
        assert!((*tc).overlay().is_none(), "an overlay was installed");
    }
}

/// A menu holding only separators holds nothing: on a terminal one line tall
/// the size check gives way first, the menu is freed and the command answers
/// normal with no overlay left behind.
#[test]
fn an_empty_menu_is_neither_measured_nor_displayed() {
    let (mut p, rv) = displayed(c"display-menu \"\"", 80, 1);
    unsafe {
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert!((*p.tc).overlay().is_none(), "an overlay was installed");
        assert_eq!((*p.tc).flags, 0);
    }
    drop(p);
}

/// More items than the terminal has room for fail the size check before any
/// measuring happens, and the answer is normal with nothing displayed.
#[test]
fn a_menu_taller_than_its_terminal_is_not_displayed() {
    let (mut p, rv) = displayed(c"display-menu a b c d e f g h i", 80, 4);
    unsafe {
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert!((*p.tc).overlay().is_none(), "an overlay was installed");
        assert_eq!((*p.tc).flags, 0);
    }
    drop(p);
}

/// An unknown border choice is refused only after the position has been
/// worked out — the full measuring path runs first, over a client attached
/// to the registered session — and still leaves no overlay behind.
#[test]
fn an_unknown_menu_border_choice_is_an_error_after_measuring() {
    let _guard = globals();
    let mut t = Target::new(100, 40);
    let (_tc_box, tc) = lone_client(100, 40);
    unsafe {
        (*tc).session = t.session();
    }
    let mut item = aimed_at(item_for(c"display-menu -b bogus x y z", 3), &mut t, tc);
    unsafe {
        assert_eq!(exec_via(MENU, &mut item), CMD_RETURN_ERROR);
        assert!((*tc).overlay().is_none(), "an overlay was installed");
    }
}

/// A menu that fits is measured, positioned and parked on the client as an
/// overlay: the command answers wait, and everything the display would read
/// — flags, border choice, starting choice, position, styles, title and the
/// menu's own items — sits in the parked data. Taking the overlay off again
/// frees the menu through its free callback and continues the waiting item.
#[test]
fn a_displayable_menu_installs_its_overlay_and_waits() {
    let (mut p, rv) = displayed(
        c"display-menu -O -T 'Menu #{session_name}' -s bg=#ff0000 -b double -C 1 x y z w v u",
        100,
        40,
    );
    unsafe {
        assert_eq!(rv, CMD_RETURN_WAIT);

        assert_eq!((*p.tc).overlay(), Overlay::Menu);
        assert_ne!(
            (*p.tc).flags & CLIENT_ALLREDRAWFLAGS,
            0,
            "the client was not queued for redraw"
        );

        let md = (*p.tc).overlay_data().menu();
        assert!(!md.is_null(), "no menu data was parked");
        assert_eq!(held_item(&(*md).item), p.item.ptr());
        assert_eq!((*md).fs.session(), p.t.session());
        assert_eq!((*md).fs.pane(), p.t.pane(0));
        assert_eq!((*md).flags, MENU_STAYOPEN | MENU_NOMOUSE);
        assert_eq!((*md).border_lines, BOX_LINES_DOUBLE);
        assert_eq!((*md).choice, 1);
        assert_eq!(seen(cstr_ptr(&(*md).style)), "bg=#ff0000");
        assert!((*md).selected_style.is_none());
        assert!((*md).border_style.is_none());

        let menu = &(*md).menu;
        assert_eq!(seen(cstr_ptr(&menu.title)), "Menu 0");
        assert_eq!(menu.items.len(), 2);
        let items = &menu.items;
        assert_eq!(items[0].key, key_string_lookup_string(c"y".as_ptr()));
        assert_eq!(seen(cstr_ptr(&items[0].command)), "z");
        assert!(seen(cstr_ptr(&items[1].name)).contains("(v)"));

        let w = menu.width + 4;
        assert_eq!((*md).px, (100 - 1) / 2 - w / 2);
        assert_eq!((*md).py, 17);
        assert_eq!((*screen_grid_ptr(&raw mut (*md).s)).sx, w);
        assert_eq!(
            (*screen_grid_ptr(&raw mut (*md).s)).sy,
            menu.items.len() as u_int + 2
        );
        assert_eq!((*md).s.mode & MODE_CURSOR, 0);
        assert_eq!((*md).s.mode & MODE_MOUSE_ALL, 0);

        server_client_clear_overlay(p.tc);
        assert!((*p.tc).overlay().is_none());
        assert!((*p.tc).overlay_data().is_none());
        assert_eq!((*p.item.ptr()).flags & crate::cmd::CMDQ_WAITING, 0);
    }
    drop(p);
}

/// With `-M` the menu is for the mouse: the no-mouse flag is withheld, the
/// screen is put into mouse mode instead, and no choice is pre-selected even
/// though the default starting choice would allow one.
#[test]
fn a_mouse_menu_starts_in_mouse_mode_with_no_choice() {
    let (mut p, rv) = displayed(c"display-menu -M -T t x y z", 100, 40);
    unsafe {
        assert_eq!(rv, CMD_RETURN_WAIT);
        let md = (*p.tc).overlay_data().menu();
        assert!(!md.is_null(), "no menu data was parked");
        assert_eq!((*md).flags, 0);
        assert_eq!((*md).choice, -1);
        assert_ne!((*md).s.mode & MODE_MOUSE_ALL, 0);
        assert_ne!((*md).s.mode & MODE_MOUSE_BUTTON, 0);
        assert_eq!(seen(cstr_ptr(&(*md).menu.title)), "t");
        server_client_clear_overlay(p.tc);
    }
    drop(p);
}

/// `-C` clears whatever overlay the client carries and answers normal at
/// once, whether or not a popup could be modified.
#[test]
fn clearing_with_c_takes_an_overlay_down_and_answers_normal() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let (_tc_box, tc) = lone_client(80, 24);
    unsafe {
        (*tc).set_overlay(OVERLAY_SENTINEL, OverlayState::None);
    }
    let mut item = aimed_at(item_for(c"display-popup -C", 1), &mut t, tc);
    unsafe {
        assert_eq!(exec_via(POPUP, &mut item), CMD_RETURN_NORMAL);
        assert!((*tc).overlay().is_none(), "the overlay survived -C");
        assert!((*tc).overlay_data().is_none());
    }
}

/// An overlay that is not a popup is not modifiable either: display-popup
/// answers normal at once and leaves it exactly as it was.
#[test]
fn a_foreign_overlay_makes_display_popup_return_at_once() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let (_tc_box, tc) = lone_client(80, 24);
    unsafe {
        (*tc).set_overlay(OVERLAY_SENTINEL, OverlayState::None);
    }
    let mut item = aimed_at(item_for(c"display-popup", 2), &mut t, tc);
    unsafe {
        assert_eq!(exec_via(POPUP, &mut item), CMD_RETURN_NORMAL);
        assert_eq!((*tc).overlay(), OVERLAY_SENTINEL);
        assert_eq!((*tc).flags, 0);
    }
}

/// A height that parses as neither a number nor a percentage is refused
/// before any sizing happens.
#[test]
fn an_unusable_height_is_an_error() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let (_tc_box, tc) = lone_client(80, 24);
    let mut item = aimed_at(item_for(c"display-popup -h bogus", 1), &mut t, tc);
    unsafe {
        assert_eq!(exec_via(POPUP, &mut item), CMD_RETURN_ERROR);
        assert!((*tc).overlay().is_none(), "a popup was opened");
    }
}

/// The width is read only after the height has been taken; a bad width is
/// refused the same way.
#[test]
fn an_unusable_width_is_an_error() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let (_tc_box, tc) = lone_client(80, 24);
    let mut item = aimed_at(item_for(c"display-popup -w bogus", 2), &mut t, tc);
    unsafe {
        assert_eq!(exec_via(POPUP, &mut item), CMD_RETURN_ERROR);
        assert!((*tc).overlay().is_none(), "a popup was opened");
    }
}
