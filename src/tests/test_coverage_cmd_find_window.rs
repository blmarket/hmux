//! Unit tests for [`crate::cmd::cmd_find_window`] — the `find-window` entry
//! (name, alias, template, usage, find flags and exec hook), the block of
//! message-protocol, display and command constants the file declares, and the
//! deterministic behaviour of [`cmd_find_window_exec`] as reached through the
//! entry's own function pointer over items built by the real command parser.
//!
//! The exec routine's whole job is to translate its `-C`/`-N`/`-T`/`-i`/`-r`
//! flags into one tree-mode filter format string, hand that to a fresh set of
//! arguments as `-f` (plus `-Z` when asked) and open the tree mode on the
//! target state's pane. Every branch of that translation is pinned here by
//! reading the filter the opened mode actually stored, and the flag shapes are
//! backed up by behaviour: a filter matching the fixture window keeps the tree
//! populated while one matching nothing leaves the mode reporting no matches,
//! `-Z` reaches the mode's zoom bookkeeping, a second run leaves the already
//! open mode and its filter untouched, and the mode lands on the pane the
//! target state carries even when that is not the window's active pane.
//!
//! Three limits worth recording. The exec routine has no error branch at all —
//! it always answers [`CMD_RETURN_NORMAL`] — so refusals live either in the
//! parser's enforcement of the template's one-argument bound (tested) or in
//! [`window_pane_set_mode`]'s already-open short circuit (tested). A `-Z` run
//! against a multi-pane window would really zoom, re-layout and redraw, which
//! needs more server than the fixtures carry, so the `-Z` test pins the
//! single-pane refusal path instead. And like the other mode-opening suites,
//! these tests leave their pane-mode-changed notifications sitting on the
//! global command queue nothing ever drains; every other process-wide state
//! they touch is taken and given back under [`globals`].

use crate::cmd::cmd_find_from_winlink;
use crate::cmd::cmd_find_window::{
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
    STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT, STYLE_ALIGN_LEFT,
    STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_LIST_FOCUS, STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON,
    STYLE_LIST_RIGHT_MARKER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, cmd_find_window_entry,
};
use crate::cmd::cmdq_get_current;
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::modes::mode_tree_data;
use crate::options::options_ptr;
use crate::session::session_options;
use crate::tests::test_fixtures::{
    Item, Pane, Registry, Session, Target, Window, globals, link, seen, unlink,
};
use crate::types::*;
use crate::window::window_pane_current_mode;
use crate::window::{
    PANE_CHANGED, PANE_REDRAW, PANE_REDRAWSCROLLBAR, PANE_ZOOMED, WINDOW_ZOOMED,
    window_pane_reset_mode_all,
};
use ::core::ffi::{CStr, c_char};
use ::core::ptr::null_mut;

/// Where the tests' items claim to come from.
const FILE: &CStr = c"test-coverage-cmd-find-window.conf";

/// The entry under test.
const ENTRY: *const cmd_entry = &raw const cmd_find_window_entry;

/// Runs the parsed command an item carries through the entry's exec hook, the
/// way the command queue calls it. The item must be running this entry.
unsafe fn exec_via(item: &mut Item) -> cmd_retval {
    unsafe {
        assert!(
            ::core::ptr::eq((*item.cmd()).entry, ENTRY),
            "the item is not running find-window"
        );
        let exec = (*ENTRY).exec;
        exec(&*item.cmd(), item.ptr())
    }
}

/// An item claiming to come from [`FILE`], carrying a parsed `find-window`
/// command line.
fn item_for(line: &'static CStr, number: u_int) -> Item {
    Item::with_client().from_file(FILE, number).with_args(line)
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
/// below its window's below the global window tree, and the session's below
/// the global session tree — because the tree mode builds read options through
/// that chain of parents. The fixtures leave every set unparented, so the link
/// is put in for the length of a test and taken out again on drop, before the
/// sets themselves go away.
struct ChainedOptions {
    pane: *mut options,
    window: *mut options,
    session: *mut options,
}

impl ChainedOptions {
    /// Chains `target`'s three sets onto the global trees.
    fn over(target: &mut Target) -> ChainedOptions {
        unsafe {
            let pane = options_ptr(&(*target.pane(0)).options);
            let window = options_ptr(&(*target.window(0)).options);
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

/// Opens the tree mode on `wp` by running `line`, chaining options first, and
/// hands back the pane for further inspection.
unsafe fn open_tree_mode(target: &mut Target, line: &'static CStr, number: u_int) -> cmd_retval {
    unsafe {
        let _chain = ChainedOptions::over(target);
        let mut item = item_for(line, number).targeting(target);
        exec_via(&mut item)
    }
}

/// The mode-tree data of `wp`'s first mode entry, failing if it is not there.
unsafe fn tree_mtd(wp: *mut window_pane) -> *mut mode_tree_data {
    unsafe {
        let wme = first_mode(wp);
        let data = (*wme).state.tree();
        (*data).tree()
    }
}

/// The filter the tree mode opened on `wp` was given, read out of the mode's
/// own copy.
unsafe fn open_filter(wp: *mut window_pane) -> String {
    unsafe { seen(cstr_ptr(&(*tree_mtd(wp)).filter)) }
}

/// How many top-level items the opened tree holds.
unsafe fn child_count(wp: *mut window_pane) -> usize {
    unsafe {
        let mtd = tree_mtd(wp);
        (*mtd).children.len()
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
fn the_entry_describes_the_find_window_command() {
    unsafe {
        assert_eq!((*ENTRY).name.to_string_lossy(), "find-window");
        assert_eq!(
            (*ENTRY)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "findw"
        );
        assert_eq!(
            (*ENTRY).usage.to_string_lossy(),
            "[-CiNrTZ] [-t target-pane] match-string"
        );
        assert_eq!((*ENTRY).args.template.to_string_lossy(), "CiNrt:TZ");
        assert_eq!((*ENTRY).args.lower, 1);
        assert_eq!((*ENTRY).args.upper, 1);
        assert!(
            (*ENTRY).args.cb.is_none(),
            "find-window takes no args callback"
        );

        assert_eq!((*ENTRY).source.flag, 0);
        assert_eq!((*ENTRY).source.type_0, CMD_FIND_PANE);
        assert_eq!((*ENTRY).source.flags, 0);
        assert_eq!((*ENTRY).target.flag, 't' as c_char);
        assert_eq!((*ENTRY).target.type_0, CMD_FIND_PANE);
        assert_eq!((*ENTRY).target.flags, 0);

        assert_eq!((*ENTRY).flags, 0);
    }
}

#[test]
fn the_parser_resolves_the_name_the_alias_and_a_prefix() {
    let _guard = globals();
    unsafe {
        for (i, line) in [c"find-window foo", c"findw foo", c"find-w foo"]
            .into_iter()
            .enumerate()
        {
            let mut item = Item::new().from_file(FILE, i as u_int + 1).with_args(line);
            assert!(::core::ptr::eq((*item.cmd()).entry, ENTRY), "{line:?}");
        }
    }
}

#[test]
fn the_template_bounds_allow_exactly_one_argument() {
    let _guard = globals();
    unsafe {
        let mut none = cmd_parse_from_string(c"find-window".as_ptr(), null_mut());
        assert_eq!(none.status, CMD_PARSE_ERROR);
        let err = none.take_error();
        assert!(err.contains("find-window"), "{err}");
        assert!(err.contains("too few arguments"), "{err}");

        let mut one = cmd_parse_from_string(c"find-window foo".as_ptr(), null_mut());
        assert_eq!(one.status, CMD_PARSE_SUCCESS);
        let _ = one.cmdlist.take();

        let mut two = cmd_parse_from_string(c"find-window foo bar".as_ptr(), null_mut());
        assert_eq!(two.status, CMD_PARSE_ERROR);
        let err = two.take_error();
        assert!(err.contains("find-window"), "{err}");
        assert!(err.contains("too many arguments"), "{err}");
    }
}

#[test]
fn find_window_opens_the_tree_mode_on_the_target_pane() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    unsafe {
        assert_eq!(
            open_tree_mode(&mut t, c"find-window target", 1),
            CMD_RETURN_NORMAL
        );

        let wp = t.pane(0);
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
fn every_flag_shape_builds_its_own_filter() {
    let _guard = globals();
    unsafe {
        for (i, (line, want)) in [
            (
                c"find-window foo",
                "#{||:#{C:foo},#{||:#{m:*foo*,#{window_name}},#{m:*foo*,#{pane_title}}}}",
            ),
            (c"find-window -C foo", "#{C:foo}"),
            (c"find-window -N foo", "#{m:*foo*,#{window_name}}"),
            (c"find-window -T foo", "#{m:*foo*,#{pane_title}}"),
            (
                c"find-window -CN foo",
                "#{||:#{C:foo},#{m:*foo*,#{window_name}}}",
            ),
            (
                c"find-window -CT foo",
                "#{||:#{C:foo},#{m:*foo*,#{pane_title}}}",
            ),
            (
                c"find-window -NT bar",
                "#{||:#{m:*bar*,#{window_name}},#{m:*bar*,#{pane_title}}}",
            ),
            (
                c"find-window -CNT foo",
                "#{||:#{C:foo},#{||:#{m:*foo*,#{window_name}},#{m:*foo*,#{pane_title}}}}",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let mut t = Target::new(80, 24);
            assert_eq!(
                open_tree_mode(&mut t, line, i as u_int + 1),
                CMD_RETURN_NORMAL,
                "{line:?}"
            );
            let wp = t.pane(0);
            assert_eq!(open_filter(wp), want, "{line:?}");
            close_modes(wp);
        }
    }
}

#[test]
fn search_modifiers_change_the_suffix_and_the_star() {
    let _guard = globals();
    unsafe {
        for (i, (line, want)) in [
            (c"find-window -Ni foo", "#{m/i:*foo*,#{window_name}}"),
            (c"find-window -Nr foo", "#{m/r:foo,#{window_name}}"),
            (c"find-window -Nri foo", "#{m/ri:foo,#{window_name}}"),
            (c"find-window -Ci foo", "#{C/i:foo}"),
            (c"find-window -Tr foo", "#{m/r:foo,#{pane_title}}"),
            (
                c"find-window -ri foo",
                "#{||:#{C/ri:foo},#{||:#{m/ri:foo,#{window_name}},#{m/ri:foo,#{pane_title}}}}",
            ),
            (
                c"find-window -i foo",
                "#{||:#{C/i:foo},#{||:#{m/i:*foo*,#{window_name}},#{m/i:*foo*,#{pane_title}}}}",
            ),
            (
                c"find-window -r foo",
                "#{||:#{C/r:foo},#{||:#{m/r:foo,#{window_name}},#{m/r:foo,#{pane_title}}}}",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let mut t = Target::new(80, 24);
            assert_eq!(
                open_tree_mode(&mut t, line, i as u_int + 1),
                CMD_RETURN_NORMAL,
                "{line:?}"
            );
            let wp = t.pane(0);
            assert_eq!(open_filter(wp), want, "{line:?}");
            close_modes(wp);
        }
    }
}

#[test]
fn a_matching_filter_keeps_the_pane_in_the_tree() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    unsafe {
        assert_eq!(
            open_tree_mode(&mut t, c"find-window target", 1),
            CMD_RETURN_NORMAL
        );
        let wp = t.pane(0);
        assert_eq!((*tree_mtd(wp)).no_matches, 0);
        assert_ne!(child_count(wp), 0, "the matching pane was left out");

        close_modes(wp);
    }
}

#[test]
fn a_filter_matching_nothing_reports_no_matches() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    unsafe {
        assert_eq!(
            open_tree_mode(&mut t, c"find-window nothing-matches-this", 1),
            CMD_RETURN_NORMAL
        );
        let wp = t.pane(0);
        assert_eq!((*tree_mtd(wp)).no_matches, 1);
        assert_ne!(
            child_count(wp),
            0,
            "the fallback rebuild left the tree empty"
        );
        assert_eq!(
            open_filter(wp),
            "#{||:#{C:nothing-matches-this},#{||:#{m:*nothing-matches-this*,#{window_name}},#{m:*nothing-matches-this*,#{pane_title}}}}",
            "the filter did not survive the fallback"
        );

        close_modes(wp);
    }
}

#[test]
fn zoom_is_requested_only_through_Z() {
    let _guard = globals();
    unsafe {
        {
            let mut plain = Target::new(80, 24);
            assert_eq!(
                open_tree_mode(&mut plain, c"find-window target", 1),
                CMD_RETURN_NORMAL
            );
            let wp = plain.pane(0);
            assert_eq!(
                (*tree_mtd(wp)).zoomed,
                -1,
                "the mode must not zoom on its own"
            );
            assert_eq!((*(*wp).window).flags & WINDOW_ZOOMED, 0);
            close_modes(wp);
        }

        {
            let mut zoomed_target = Target::new(80, 24);
            assert_eq!(
                open_tree_mode(&mut zoomed_target, c"find-window -Z target", 2),
                CMD_RETURN_NORMAL
            );
            let wp = zoomed_target.pane(0);
            assert_eq!(
                (*tree_mtd(wp)).zoomed,
                0,
                "-Z records the window's unzoomed state even though one pane cannot zoom"
            );
            assert_eq!((*(*wp).window).flags & WINDOW_ZOOMED, 0);
            assert_eq!((*wp).flags & PANE_ZOOMED, 0);
            close_modes(wp);
        }
    }
}

#[test]
fn running_again_leaves_the_open_mode_and_its_filter_alone() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);
        let mut first = item_for(c"find-window -N first-run", 1).targeting(&mut t);
        assert_eq!(exec_via(&mut first), CMD_RETURN_NORMAL);
        assert_eq!(open_filter(wp), "#{m:*first-run*,#{window_name}}");

        let mut second = item_for(c"find-window -T second-run", 2).targeting(&mut t);
        assert_eq!(exec_via(&mut second), CMD_RETURN_NORMAL);

        assert_eq!(mode_count(wp), 1, "the second run stacked another mode");
        assert_eq!(
            open_filter(wp),
            "#{m:*first-run*,#{window_name}}",
            "the short-circuited run replaced the filter anyway"
        );

        close_modes(wp);
    }
}

#[test]
fn the_mode_opens_on_the_target_states_pane_even_when_it_is_not_active() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(0, "0");
    let mut w = Window::new(0, "target", 80, 24);
    let mut active = Pane::new(0, 80, 24, 100);
    let mut other = Pane::new(1, 80, 24, 100);
    w.add_pane(&mut active);
    w.add_pane(&mut other);
    registry.add_session(&mut s);
    registry.add_window(&mut w);
    registry.add_pane(&mut active);
    registry.add_pane(&mut other);
    let wl = link(&mut s, &mut w, 0);
    unsafe {
        let mut fs = *Box::new(cmd_find_state::default());
        cmd_find_from_winlink(&mut fs, wl, 0);
        assert_eq!(fs.pane(), active.ptr(), "resolution picked the active pane");
        fs.set_pane(other.ptr());

        let mut item = item_for(c"find-window target", 1);
        (*item.ptr()).target = fs.clone();
        (*item.ptr()).source = fs.clone();
        *cmdq_get_current(item.ptr()) = fs;
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);

        let wme = first_mode(other.ptr());
        assert_eq!((*wme).mode(), WindowMode::Tree);
        assert_eq!((*other.ptr()).screen(), (*wme).screen);
        assert!(
            (*active.ptr()).modes.is_empty(),
            "the active pane got the mode instead"
        );

        close_modes(other.ptr());
        unlink(&mut s, wl);
    }
}
