//! Unit tests for [`crate::cmd::cmd_choose_tree`] — the four command entries
//! the file publishes (`choose-tree`, `choose-client`, `choose-buffer` and
//! `customize-mode`, all sharing one exec hook), the message-protocol,
//! sorting, style and layout constants it declares, the argument-parse
//! callback three of them hand the parser, and every deterministic branch of
//! the exec routine the fixtures can reach without a live daemon or a
//! terminal.
//!
//! Exec is reached through each entry's own function pointer, exactly as the
//! command queue calls it, over items whose arguments come from the real
//! command parser and whose targets are resolved against a registered
//! [`Target`]. The mode-opening branches run the real `init`/`free` of
//! window_tree, window_client, window_buffer and window_customize over that
//! registered state, so every test closes the modes it opened with
//! [`window_pane_reset_mode_all`] before its fixtures go away. Two pieces of
//! process-wide state are touched in ways the fixtures cannot undo, matching
//! what the other suites already accept: the one error branch reports through
//! `cmdq_error` onto a client-less item, which only records a cause in
//! `cfg.rs`'s private list where no test can read or free it, and the
//! mode-change notifications are appended to the global command queue, which
//! no unit test ever drains. The globals everything else reads — the option
//! trees, the environment, the session/window/pane trees and the client list
//! — are taken and given back under [`globals`] in every test.

use crate::arguments::args_create;
use crate::cmd::cmd_choose_tree::{
    ARGS_PARSE_COMMANDS, ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_INVALID, ARGS_PARSE_STRING,
    CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN, CMD_FIND_PANE, CMD_FIND_SESSION,
    CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_STOP, CMD_RETURN_WAIT,
    LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL,
    MSG_EXEC, MSG_EXIT, MSG_EXITED, MSG_EXITING, MSG_FLAGS, MSG_IDENTIFY_CLIENTPID,
    MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON, MSG_IDENTIFY_FEATURES,
    MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS, MSG_IDENTIFY_OLDCWD, MSG_IDENTIFY_STDIN,
    MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM, MSG_IDENTIFY_TERMINFO, MSG_IDENTIFY_TTYNAME, MSG_LOCK,
    MSG_OLDSTDERR, MSG_OLDSTDIN, MSG_OLDSTDOUT, MSG_READ, MSG_READ_CANCEL, MSG_READ_DONE,
    MSG_READ_OPEN, MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN, MSG_SUSPEND, MSG_UNLOCK,
    MSG_VERSION, MSG_WAKEUP, MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_OPEN, MSG_WRITE_READY,
    PANE_LINES_DOUBLE, PANE_LINES_HEAVY, PANE_LINES_NUMBER, PANE_LINES_SIMPLE, PANE_LINES_SINGLE,
    PANE_LINES_SPACES, PROGRESS_BAR_ERROR, PROGRESS_BAR_HIDDEN, PROGRESS_BAR_INDETERMINATE,
    PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED, PROMPT_COMMAND, PROMPT_ENTRY, PROMPT_TYPE_COMMAND,
    PROMPT_TYPE_INVALID, PROMPT_TYPE_SEARCH, PROMPT_TYPE_TARGET, PROMPT_TYPE_WINDOW_TARGET,
    SCREEN_CURSOR_BAR, SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT, SCREEN_CURSOR_UNDERLINE,
    SORT_ACTIVITY, SORT_CREATION, SORT_END, SORT_INDEX, SORT_MODIFIER, SORT_NAME, SORT_ORDER,
    SORT_SIZE, SORT_Z, STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT,
    STYLE_ALIGN_LEFT, STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_LIST_FOCUS, STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON,
    STYLE_LIST_RIGHT_MARKER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, cmd_choose_buffer_entry, cmd_choose_client_entry,
    cmd_choose_tree_entry, cmd_customize_mode_entry,
};
use crate::modes::{mode_tree_expand_current, mode_tree_get_current};
use crate::paste::paste_is_empty;
use crate::server::server_client_how_many;
use crate::session::session_options;
use crate::status::{status_init, status_prompt_clear};
use crate::tests::test_fixtures::{Clients, Item, Paste, Target, globals, seen};
use crate::types::*;
use crate::window::window_pane_current_mode;
use crate::window::{PANE_CHANGED, PANE_REDRAW, PANE_REDRAWSCROLLBAR, window_pane_reset_mode_all};
use ::core::ffi::{CStr, c_char};
use ::core::ptr::null_mut;
use ::std::ffi::CString;

/// Where the tests' items claim to come from, which is what `cmdq_error`
/// would report them under.
const FILE: &CStr = c"test-coverage-cmd-choose-tree.conf";

/// The entries whose exec hook is under test.
const TREE: *const cmd_entry = &raw const cmd_choose_tree_entry;
const CLIENT: *const cmd_entry = &raw const cmd_choose_client_entry;
const BUFFER: *const cmd_entry = &raw const cmd_choose_buffer_entry;
const CUSTOMIZE: *const cmd_entry = &raw const cmd_customize_mode_entry;

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

/// The address of an entry's shared exec hook, which is how two entries are
/// recognised as running one routine without comparing function pointers.
fn exec_addr(entry: *const cmd_entry) -> usize {
    unsafe { (*entry).exec as usize }
}

/// The address of an entry's shared argument-parse callback.
fn cb_addr(entry: *const cmd_entry) -> usize {
    unsafe { (*entry).args.cb.map_or(0, |f| f as usize) }
}

/// An item claiming to come from [`FILE`], carrying a parsed command line.
fn item_for(line: &'static CStr, number: u_int) -> Item {
    Item::new().from_file(FILE, number).with_args(line)
}

/// The pane's first mode entry, failing if it opened none.
unsafe fn first_mode(wp: *mut window_pane) -> *mut window_mode_entry {
    unsafe {
        let wme = window_pane_current_mode(wp);
        assert!(!wme.is_null(), "the pane opened no mode");
        wme
    }
}

/// How many mode entries the pane carries.
unsafe fn mode_count(wp: *mut window_pane) -> usize {
    unsafe { (*wp).modes.len() }
}

/// Takes every mode off `wp`, leaving it back on its base screen.
unsafe fn close_modes(wp: *mut window_pane) {
    unsafe {
        window_pane_reset_mode_all(wp);
        assert!((*wp).modes.is_empty(), "modes were not closed");
        assert_eq!((*wp).screen(), &raw mut (*wp).base);
    }
}

/// Links a target's option sets the way the server chains them — the pane's
/// below its window's below [`crate::tmux::global_w_options`], and the
/// session's below [`crate::tmux::global_s_options`] — because
/// `options_get` resolves through that chain of parents and the mode builds
/// read options through it. The fixtures leave every set unparented, so the
/// link is put in for the length of a test and taken out again on drop,
/// before the sets themselves go away.
struct ChainedOptions {
    pane: *mut options,
    window: *mut options,
    session: *mut options,
}

impl ChainedOptions {
    /// Chains `target`'s three sets onto the global trees.
    fn over(target: &mut Target) -> ChainedOptions {
        unsafe {
            let pane = (*target.pane(0)).options_ptr();
            let window = (*target.window(0)).options_ptr();
            let session = session_options(target.session());
            (*pane).parent = window;
            (*window).parent = crate::tmux::global_w_options;
            (*session).parent = crate::tmux::global_s_options;
            ChainedOptions {
                pane,
                window,
                session,
            }
        }
    }
}

impl Drop for ChainedOptions {
    fn drop(&mut self) {
        unsafe {
            (*self.pane).parent = null_mut();
            (*self.window).parent = null_mut();
            (*self.session).parent = null_mut();
        }
    }
}

#[test]
fn the_message_constants_keep_their_protocol_numbers() {
    for (constant, value) in [
        (MSG_READ_CANCEL, 307),
        (MSG_WRITE_CLOSE, 306),
        (MSG_WRITE_READY, 305),
        (MSG_WRITE, 304),
        (MSG_WRITE_OPEN, 303),
        (MSG_READ_DONE, 302),
        (MSG_READ, 301),
        (MSG_READ_OPEN, 300),
        (MSG_FLAGS, 218),
        (MSG_EXEC, 217),
        (MSG_WAKEUP, 216),
        (MSG_UNLOCK, 215),
        (MSG_SUSPEND, 214),
        (MSG_OLDSTDOUT, 213),
        (MSG_OLDSTDIN, 212),
        (MSG_OLDSTDERR, 211),
        (MSG_SHUTDOWN, 210),
        (MSG_SHELL, 209),
        (MSG_RESIZE, 208),
        (MSG_READY, 207),
        (MSG_LOCK, 206),
        (MSG_EXITING, 205),
        (MSG_EXITED, 204),
        (MSG_EXIT, 203),
        (MSG_DETACHKILL, 202),
        (MSG_DETACH, 201),
        (MSG_COMMAND, 200),
        (MSG_IDENTIFY_TERMINFO, 112),
        (MSG_IDENTIFY_LONGFLAGS, 111),
        (MSG_IDENTIFY_STDOUT, 110),
        (MSG_IDENTIFY_FEATURES, 109),
        (MSG_IDENTIFY_CWD, 108),
        (MSG_IDENTIFY_CLIENTPID, 107),
        (MSG_IDENTIFY_DONE, 106),
        (MSG_IDENTIFY_ENVIRON, 105),
        (MSG_IDENTIFY_STDIN, 104),
        (MSG_IDENTIFY_OLDCWD, 103),
        (MSG_IDENTIFY_TTYNAME, 102),
        (MSG_IDENTIFY_TERM, 101),
        (MSG_IDENTIFY_FLAGS, 100),
        (MSG_VERSION, 12),
    ] {
        assert_eq!(constant, value);
    }
}

#[test]
fn the_display_constants_keep_their_values() {
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
    ] {
        assert_eq!(constant, value);
    }
}

#[test]
fn the_command_constants_keep_their_values() {
    for (constant, value) in [
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
        (CMD_FIND_PANE, 0),
        (CMD_FIND_WINDOW, 1),
        (CMD_FIND_SESSION, 2),
        (SORT_ACTIVITY, 0),
        (SORT_CREATION, 1),
        (SORT_INDEX, 2),
        (SORT_MODIFIER, 3),
        (SORT_NAME, 4),
        (SORT_ORDER, 5),
        (SORT_SIZE, 6),
        (SORT_Z, 7),
        (SORT_END, 8),
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

#[test]
fn the_choose_tree_entry_describes_the_choose_tree_command() {
    unsafe {
        assert_eq!((*TREE).name.to_string_lossy(), "choose-tree");
        assert!((*TREE).alias.is_none(), "choose-tree has no alias");
        assert_eq!(
            (*TREE).usage.to_string_lossy(),
            "[-GNrswZ] [-F format] [-f filter] [-K key-format] [-O sort-order] [-t target-pane] [template]"
        );
        assert_eq!((*TREE).args.template.to_string_lossy(), "F:f:GK:NO:rst:wyZ");
        assert_eq!((*TREE).args.lower, 0);
        assert_eq!((*TREE).args.upper, 1);
        assert!((*TREE).args.cb.is_some());

        assert_eq!((*TREE).source.flag, 0);
        assert_eq!((*TREE).source.type_0, CMD_FIND_PANE);
        assert_eq!((*TREE).source.flags, 0);
        assert_eq!((*TREE).target.flag, 't' as c_char);
        assert_eq!((*TREE).target.type_0, CMD_FIND_PANE);
        assert_eq!((*TREE).target.flags, 0);

        assert_eq!((*TREE).flags, 0);
    }
}

#[test]
fn the_choose_client_and_choose_buffer_entries_share_the_choose_tree_hooks() {
    unsafe {
        for (entry, name) in [(CLIENT, "choose-client"), (BUFFER, "choose-buffer")] {
            assert_eq!((*entry).name.to_string_lossy(), name);
            assert!((*entry).alias.is_none(), "{name} has no alias");
            assert_eq!(
                (*entry).usage.to_string_lossy(),
                "[-NrZ] [-F format] [-f filter] [-K key-format] [-O sort-order] [-t target-pane] [template]"
            );
            assert_eq!((*entry).args.template.to_string_lossy(), "F:f:K:NO:rt:yZ");
            assert_eq!((*entry).args.lower, 0);
            assert_eq!((*entry).args.upper, 1);

            assert_eq!((*entry).source.flag, 0);
            assert_eq!((*entry).source.type_0, CMD_FIND_PANE);
            assert_eq!((*entry).source.flags, 0);
            assert_eq!((*entry).target.flag, 't' as c_char);
            assert_eq!((*entry).target.type_0, CMD_FIND_PANE);
            assert_eq!((*entry).target.flags, 0);
            assert_eq!((*entry).flags, 0);

            assert_eq!(
                cb_addr(entry),
                cb_addr(TREE),
                "{name} parses like choose-tree"
            );
            assert_eq!(
                exec_addr(entry),
                exec_addr(TREE),
                "{name} executes like choose-tree"
            );
        }
    }
}

#[test]
fn the_customize_mode_entry_describes_the_customize_mode_command() {
    unsafe {
        assert_eq!((*CUSTOMIZE).name.to_string_lossy(), "customize-mode");
        assert!((*CUSTOMIZE).alias.is_none(), "customize-mode has no alias");
        assert_eq!(
            (*CUSTOMIZE).usage.to_string_lossy(),
            "[-NZ] [-F format] [-f filter] [-t target-pane]"
        );
        assert_eq!((*CUSTOMIZE).args.template.to_string_lossy(), "F:f:Nt:yZ");
        assert_eq!((*CUSTOMIZE).args.lower, 0);
        assert_eq!((*CUSTOMIZE).args.upper, 0);
        assert!(
            (*CUSTOMIZE).args.cb.is_none(),
            "customize-mode takes no callback"
        );

        assert_eq!((*CUSTOMIZE).source.flag, 0);
        assert_eq!((*CUSTOMIZE).source.type_0, CMD_FIND_PANE);
        assert_eq!((*CUSTOMIZE).source.flags, 0);
        assert_eq!((*CUSTOMIZE).target.flag, 't' as c_char);
        assert_eq!((*CUSTOMIZE).target.type_0, CMD_FIND_PANE);
        assert_eq!((*CUSTOMIZE).target.flags, 0);
        assert_eq!((*CUSTOMIZE).flags, 0);
        assert_eq!(
            exec_addr(CUSTOMIZE),
            exec_addr(TREE),
            "it executes like choose-tree"
        );
    }
}

#[test]
fn the_parser_resolves_the_commands_to_these_entries() {
    let _guard = globals();
    unsafe {
        for (i, (line, want)) in [
            (c"choose-tree", TREE),
            (c"choose-client", CLIENT),
            (c"choose-buffer", BUFFER),
            (c"customize-mode", CUSTOMIZE),
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
fn the_argument_callback_always_accepts_commands_or_a_string() {
    unsafe {
        let mut cause = None;
        for entry in [TREE, CLIENT, BUFFER] {
            let cb = (*entry)
                .args
                .cb
                .expect("the entry carries an args callback");
            for idx in [0 as u_int, 7 as u_int] {
                assert_eq!(
                    cb(&args_create(), idx, &mut cause),
                    ARGS_PARSE_COMMANDS_OR_STRING
                );
            }
        }
    }
}

/// The invalid-order refusal for the three commands whose template carries
/// `-O`. customize-mode's has no `-O` at all, so it cannot reach the check,
/// and the check sits ahead of the dispatch anyway, so which of them runs
/// makes no difference.
#[test]
fn an_unknown_sort_order_is_an_error_and_sets_no_mode() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    unsafe {
        for (i, (line, entry)) in [
            (c"choose-tree -O bogus", TREE),
            (c"choose-client -O no-such-order", CLIENT),
            (c"choose-buffer -O Zzz", BUFFER),
        ]
        .into_iter()
        .enumerate()
        {
            let wp = t.pane(0);
            let mut item = item_for(line, i as u_int + 1).targeting(&mut t);
            assert_eq!(exec_via(entry, &mut item), CMD_RETURN_ERROR, "{line:?}");
            assert!((*wp).modes.is_empty(), "{line:?} opened a mode");
            assert_eq!((*wp).screen(), &raw mut (*wp).base, "{line:?}");
            assert_eq!((*wp).flags, 0, "{line:?} touched the pane");
        }
    }
}

#[test]
fn a_named_sort_order_is_accepted_and_opens_the_tree_mode() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let _chain = ChainedOptions::over(&mut t);
    unsafe {
        let wp = t.pane(0);
        let mut item = Item::with_client()
            .from_file(FILE, 1)
            .with_args(c"choose-tree -O index")
            .targeting(&mut t);
        assert_eq!(exec_via(TREE, &mut item), CMD_RETURN_NORMAL);

        let wme = first_mode(wp);
        assert_eq!((*wme).mode(), WindowMode::Tree);
        close_modes(wp);
    }
}

#[test]
fn choose_tree_opens_the_tree_mode_on_the_target_pane() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let _chain = ChainedOptions::over(&mut t);
    unsafe {
        let wp = t.pane(0);
        let mut item = Item::with_client()
            .from_file(FILE, 1)
            .with_args(c"choose-tree")
            .targeting(&mut t);
        assert_eq!(exec_via(TREE, &mut item), CMD_RETURN_NORMAL);

        let wme = first_mode(wp);
        assert_eq!((*wme).mode(), WindowMode::Tree);
        assert_eq!(seen((*wme).mode().name().as_ptr()), "tree-mode");
        assert_eq!((*wme).wp, wp);
        assert!((*wme).swp.is_null(), "no source pane is handed over");
        assert_eq!((*wp).screen(), (*wme).screen);
        assert_ne!((*wp).screen(), &raw mut (*wp).base);
        assert_eq!(
            (*wp).flags & (PANE_REDRAW | PANE_REDRAWSCROLLBAR | PANE_CHANGED),
            PANE_REDRAW | PANE_REDRAWSCROLLBAR | PANE_CHANGED
        );

        close_modes(wp);
    }
}

#[test]
fn running_choose_tree_again_keeps_a_single_mode_entry() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let _chain = ChainedOptions::over(&mut t);
    unsafe {
        let wp = t.pane(0);
        for n in 1..=2 {
            let mut item = Item::with_client()
                .from_file(FILE, n)
                .with_args(c"choose-tree")
                .targeting(&mut t);
            assert_eq!(exec_via(TREE, &mut item), CMD_RETURN_NORMAL);
        }

        let wme = first_mode(wp);
        assert_eq!(mode_count(wp), 1, "the second run stacked another mode");
        assert_eq!((*wme).mode(), WindowMode::Tree);

        close_modes(wp);
    }
}

#[test]
fn customize_mode_always_opens_options_mode() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let _chain = ChainedOptions::over(&mut t);
    unsafe {
        let wp = t.pane(0);
        let mut item = Item::with_client()
            .from_file(FILE, 1)
            .with_args(c"customize-mode")
            .targeting(&mut t);
        assert_eq!(exec_via(CUSTOMIZE, &mut item), CMD_RETURN_NORMAL);

        let wme = first_mode(wp);
        assert_eq!((*wme).mode(), WindowMode::Customize);
        assert_eq!(seen((*wme).mode().name().as_ptr()), "options-mode");
        assert_eq!((*wp).screen(), (*wme).screen);

        close_modes(wp);
    }
}

/// The text a built mode tree carries on the row named `name`, or `None` when
/// no row goes by that name. Rows are walked depth first, since a scope heads
/// a subtree of the options under it.
unsafe fn row_text(list: *mut mode_tree_list, name: &CStr) -> Option<Option<CString>> {
    unsafe {
        for mti in (*list).iter_mut() {
            if mti.name.as_deref() == Some(name) {
                return Some(mti.text.clone());
            }
            if let Some(found) = row_text(&raw mut mti.children, name) {
                return Some(found);
            }
        }
        None
    }
}

/// A plain (non-array) option row carries the text its format expanded to.
/// The array arm of the same builder heads a subtree and carries none, and
/// array *items* are expanded by a different builder — so a change that drops
/// the plain arm's expansion leaves those rows blank while the mode still
/// opens, still draws, and still shows text on every array row.
#[test]
fn options_mode_rows_carry_their_expanded_text() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let _chain = ChainedOptions::over(&mut t);
    unsafe {
        let wp = t.pane(0);
        let mut item = Item::with_client()
            .from_file(FILE, 1)
            .with_args(c"customize-mode")
            .targeting(&mut t);
        assert_eq!(exec_via(CUSTOMIZE, &mut item), CMD_RETURN_NORMAL);

        let wme = first_mode(wp);
        let data = (*wme).state.customize();
        let mtd = (*data).tree();
        assert!(!mtd.is_null(), "the mode built no tree");

        let text = row_text(&raw mut (*mtd).children, c"buffer-limit")
            .expect("the options mode built no buffer-limit row");
        let text = text.expect("the buffer-limit row carries no text at all");
        assert!(
            !text.to_bytes().is_empty(),
            "the buffer-limit row lost its expanded text"
        );

        close_modes(wp);
    }
}

/// The early return when the store holds nothing. A named sort order proves
/// the order check has already been passed when the store is consulted.
#[test]
fn an_empty_paste_store_makes_choose_buffer_return_without_a_mode() {
    let _guard = globals();
    let paste = Paste::new();
    let mut t = Target::new(80, 24);
    unsafe {
        assert_ne!(paste_is_empty(), 0, "the store was not emptied");
        let wp = t.pane(0);
        let mut item = Item::with_client()
            .from_file(FILE, 1)
            .with_args(c"choose-buffer -O index")
            .targeting(&mut t);
        assert_eq!(exec_via(BUFFER, &mut item), CMD_RETURN_NORMAL);

        assert!((*wp).modes.is_empty(), "buffer-mode opened anyway");
        assert_eq!((*wp).screen(), &raw mut (*wp).base);
        drop(paste);
    }
}

#[test]
fn buffers_in_the_store_open_buffer_mode() {
    let _guard = globals();
    let paste = Paste::new();
    paste.add(c"fixture-buffer", "payload");
    let mut t = Target::new(80, 24);
    unsafe {
        assert_eq!(paste_is_empty(), 0, "the buffer did not land");
        let wp = t.pane(0);
        let mut item = Item::with_client()
            .from_file(FILE, 1)
            .with_args(c"choose-buffer")
            .targeting(&mut t);
        assert_eq!(exec_via(BUFFER, &mut item), CMD_RETURN_NORMAL);

        let wme = first_mode(wp);
        assert_eq!((*wme).mode(), WindowMode::Buffer);
        assert_eq!(seen((*wme).mode().name().as_ptr()), "buffer-mode");

        close_modes(wp);
        drop(paste);
    }
}

#[test]
fn without_attached_clients_choose_client_returns_without_a_mode() {
    let _guard = globals();
    let clients = Clients::new();
    let mut t = Target::new(80, 24);
    unsafe {
        assert_eq!(server_client_how_many(), 0, "a client is attached");
        let wp = t.pane(0);
        let mut item = Item::with_client()
            .from_file(FILE, 1)
            .with_args(c"choose-client -O name")
            .targeting(&mut t);
        assert_eq!(exec_via(CLIENT, &mut item), CMD_RETURN_NORMAL);

        assert!((*wp).modes.is_empty(), "client-mode opened anyway");
        assert_eq!((*wp).screen(), &raw mut (*wp).base);
        drop(clients);
    }
}

#[test]
fn an_attached_client_opens_client_mode() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut t = Target::new(80, 24);
    unsafe {
        let attached = clients.add("client-1", 80, 24);
        (*attached).session = t.session();
        assert_ne!(server_client_how_many(), 0, "the client does not count");

        let wp = t.pane(0);
        let mut item = Item::with_client()
            .from_file(FILE, 1)
            .with_args(c"choose-client")
            .targeting(&mut t);
        assert_eq!(exec_via(CLIENT, &mut item), CMD_RETURN_NORMAL);

        let wme = first_mode(wp);
        assert_eq!((*wme).mode(), WindowMode::Client);
        assert_eq!(seen((*wme).mode().name().as_ptr()), "client-mode");

        close_modes(wp);
        drop(clients);
    }
}

/// A client the modes can raise a prompt on: attached to the fixture session
/// and holding the status line the prompt pushes and pops against.
unsafe fn asker(clients: &mut Clients, t: &mut Target) -> *mut client {
    unsafe {
        let c = clients.add("asker", 80, 24);
        (*c).session = t.session();
        (*c).status.active = crate::types::StatusActive::Own;
        status_init(c);
        c
    }
}

/// The command prompt tree mode opens on `:` outlives the mode itself.
/// Closing the mode releases the tree state, so the answer the prompt still
/// carries reaches nothing rather than the freed mode behind it.
#[test]
fn a_command_prompt_outliving_tree_mode_answers_without_the_mode() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut t = Target::new(80, 24);
    unsafe {
        let c1 = asker(&mut clients, &mut t);
        let wp = t.pane(0);
        let mut item = Item::with_client()
            .from_file(FILE, 1)
            .with_args(c"choose-tree")
            .targeting(&mut t);
        assert_eq!(exec_via(TREE, &mut item), CMD_RETURN_NORMAL);

        let wme = first_mode(wp);
        (*wme).mode().key(
            wme,
            c1,
            t.session(),
            t.winlink(0),
            b':' as key_code,
            null_mut(),
        );
        assert_eq!((*c1).prompt, Prompt::WindowTreeCommand);
        let PromptData::WindowTree(held) = &(*c1).prompt_data else {
            panic!("the prompt gave up its handle to the mode");
        };
        assert!(held.upgrade().is_some(), "the prompt reaches no live mode");

        close_modes(wp);
        let PromptData::WindowTree(held) = &(*c1).prompt_data else {
            panic!("the prompt gave up its handle to the mode");
        };
        assert!(
            held.upgrade().is_none(),
            "tree-mode state outlived the mode entry that owned it"
        );

        let answered = (*c1)
            .prompt
            .input(c1, &mut (*c1).prompt_data, Some(c"list-panes"), 1);
        assert_eq!(answered, 0, "the prompt asked to stay up without a mode");

        status_prompt_clear(c1);
        drop(clients);
    }
}

/// The same for the option prompt customize mode opens on `s`, which reaches
/// its mode through the item it hands the prompt rather than through the
/// prompt's own data.
#[test]
fn an_option_prompt_outliving_customize_mode_answers_without_the_mode() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut t = Target::new(80, 24);
    let _chain = ChainedOptions::over(&mut t);
    unsafe {
        let c1 = asker(&mut clients, &mut t);
        let wp = t.pane(0);
        let mut item = Item::with_client()
            .from_file(FILE, 1)
            .with_args(c"customize-mode")
            .targeting(&mut t);
        assert_eq!(exec_via(CUSTOMIZE, &mut item), CMD_RETURN_NORMAL);

        let wme = first_mode(wp);
        let mtd = (*(*wme).state.customize()).tree_ref();
        mode_tree_expand_current(&mtd);
        (*wme).mode().key(
            wme,
            c1,
            t.session(),
            t.winlink(0),
            b'j' as key_code,
            null_mut(),
        );
        assert!(
            !mode_tree_get_current(&mtd).customize().is_null(),
            "no option row to open a prompt over"
        );
        (*wme).mode().key(
            wme,
            c1,
            t.session(),
            t.winlink(0),
            b's' as key_code,
            null_mut(),
        );
        assert_eq!((*c1).prompt, Prompt::CustomizeSetOption);
        let PromptData::CustomizeSet(held) = &(*c1).prompt_data else {
            panic!("the prompt gave up the item it was opened over");
        };
        assert!(
            held.prompt_owner
                .as_ref()
                .is_some_and(|owner| owner.upgrade().is_some()),
            "the prompt item reaches no live mode"
        );

        close_modes(wp);
        let PromptData::CustomizeSet(held) = &(*c1).prompt_data else {
            panic!("the prompt gave up the item it was opened over");
        };
        let owner = held
            .prompt_owner
            .as_ref()
            .expect("a prompt item carries its handle to the mode");
        assert!(
            owner.upgrade().is_none(),
            "options-mode state outlived the mode entry that owned it"
        );

        let answered = (*c1)
            .prompt
            .input(c1, &mut (*c1).prompt_data, Some(c"42"), 1);
        assert_eq!(answered, 0, "the prompt asked to stay up without a mode");

        status_prompt_clear(c1);
        drop(clients);
    }
}
