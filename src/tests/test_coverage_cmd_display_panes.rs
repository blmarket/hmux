//! Unit tests for [`crate::cmd::cmd_display_panes`] — the `display-panes`
//! command entry, the message-protocol and display constants the file
//! re-declares, the argument-parse callback that classifies the optional
//! template argument, and every deterministic branch of the exec hook the
//! fixtures can reach without a daemon, a live terminal or a spawned process.
//!
//! Exec is reached through the entry's own function pointer, exactly as the
//! command queue calls it, over items whose arguments come from the real
//! command parser and whose target client is a fixture client attached to a
//! registered [`Target`]. A successful run parks a private
//! [`cmd_display_panes_data`] on that client as an overlay — draw callback,
//! key callback (unless `-N`), free callback and the prepared command state —
//! and answers wait unless `-b` was given. Taking the overlay down again with
//! [`server_client_clear_overlay`] runs the file's own free callback: the
//! prepared state is freed and the waiting item is continued, so nothing
//! dangles when the fixtures go away.
//!
//! The key callback is reached through the stored overlay with
//! synthetic key events: digits map to pane indexes directly, letters continue
//! past nine, anything else (uppercase letters, keys carrying modifier bits)
//! is refused before anything is queued, and an index with no pane behind it
//! is swallowed without touching the queue or the window's zoom state. A hit
//! expands the template against the pane id and either inserts after the
//! waiting item or, with `-b`, appends to a per-client queue wired up by the
//! test; both chains are walked and freed again before the test ends.
//!
//! The draw callback is reached through the stored overlay over
//! a fixture client whose terminal writes into a plain ensure_reactor buffer instead
//! of a descriptor. Every capability in its term table is missing, so cursor
//! and attribute sequences expand to nothing and the only bytes produced are
//! the ones [`cmd_display_panes_draw`] puts itself — which lets the tests read
//! back exactly what was written for a large pane (clock cells as spaces plus
//! the size string), and for a pane outside the redraw context (nothing at
//! all).
//!
//! Two places stay out of reach, deliberately: the delay timer never fires
//! because nothing runs the ensure_reactor loop — expiry belongs to the server — and
//! drawing a pane whose number needs more columns than the pane has is only
//! reachable together with an eleven-or-higher index in a one-column pane,
//! which no fixture layout produces.

use crate::arguments::args_create;
use crate::arguments::args_get;
use crate::cmd::cmd_display_panes::{
    __INT_MAX__, ARGS_PARSE_COMMANDS, ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_INVALID,
    ARGS_PARSE_STRING, CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN, CMD_FIND_PANE,
    CMD_FIND_SESSION, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_STOP,
    CMD_RETURN_WAIT, KEYC_MASK_KEY, KEYC_MASK_MODIFIERS, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM,
    LAYOUT_WINDOWPANE, MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL, MSG_EXEC, MSG_EXIT, MSG_EXITED,
    MSG_EXITING, MSG_FLAGS, MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE,
    MSG_IDENTIFY_ENVIRON, MSG_IDENTIFY_FEATURES, MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS,
    MSG_IDENTIFY_OLDCWD, MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM,
    MSG_IDENTIFY_TERMINFO, MSG_IDENTIFY_TTYNAME, MSG_LOCK, MSG_OLDSTDERR, MSG_OLDSTDIN,
    MSG_OLDSTDOUT, MSG_READ, MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN, MSG_READY, MSG_RESIZE,
    MSG_SHELL, MSG_SHUTDOWN, MSG_SUSPEND, MSG_UNLOCK, MSG_VERSION, MSG_WAKEUP, MSG_WRITE,
    MSG_WRITE_CLOSE, MSG_WRITE_OPEN, MSG_WRITE_READY, PANE_LINES_DOUBLE, PANE_LINES_HEAVY,
    PANE_LINES_NUMBER, PANE_LINES_SIMPLE, PANE_LINES_SINGLE, PANE_LINES_SPACES, PROGRESS_BAR_ERROR,
    PROGRESS_BAR_HIDDEN, PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED,
    PROMPT_COMMAND, PROMPT_ENTRY, PROMPT_TYPE_COMMAND, PROMPT_TYPE_INVALID, PROMPT_TYPE_SEARCH,
    PROMPT_TYPE_TARGET, PROMPT_TYPE_WINDOW_TARGET, SCREEN_CURSOR_BAR, SCREEN_CURSOR_BLOCK,
    SCREEN_CURSOR_DEFAULT, SCREEN_CURSOR_UNDERLINE, STYLE_ALIGN_ABSOLUTE_CENTRE,
    STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT, STYLE_ALIGN_LEFT, STYLE_ALIGN_RIGHT,
    STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH, STYLE_DEFAULT_SET, STYLE_LIST_FOCUS,
    STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON, STYLE_LIST_RIGHT_MARKER,
    STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE, STYLE_RANGE_PANE, STYLE_RANGE_RIGHT,
    STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW, THEME_DARK, THEME_LIGHT,
    THEME_UNKNOWN, UINT_MAX, cmd_display_panes_data, cmd_display_panes_entry,
};
use crate::cmd::cmd_get_args;
use crate::cmd::cmdq_set_target_client;
use crate::cmd::{CMDQ_WAITING, CmdqType};
use crate::reactor::Buf;
use crate::reactor::Timer;
use crate::server::CLIENT_ALLREDRAWFLAGS;
use crate::server::server_client_clear_overlay;
use crate::tests::test_fixtures::held_item;
use crate::tests::test_fixtures::{
    Args, Item, Pane, Target, globals, seen, zeroed_client, zeroed_term,
};
use crate::types::*;
use ::core::ffi::{CStr, c_int};

/// Where the tests' items claim to come from, which is what `cfg_add_cause`
/// would report them under.
const FILE: &CStr = c"test-coverage-cmd-display-panes.conf";

/// The entry whose exec, argument-parse and overlay callbacks are under test.
const PANES: *const cmd_entry = &raw const cmd_display_panes_entry;

/// Runs the parsed command an item carries through the entry's exec hook, the
/// way the command queue calls it. The item must be running this entry.
unsafe fn exec_via(item: &mut Item) -> cmd_retval {
    unsafe {
        assert!(
            ::core::ptr::eq((*item.cmd()).entry, PANES),
            "the item is not running display-panes"
        );
        let exec = (*PANES).exec;
        exec(&*item.cmd(), item.ptr())
    }
}

/// An item claiming to come from [`FILE`], carrying a parsed command line.
fn item_for(line: &'static CStr, number: u_int) -> Item {
    Item::new().from_file(FILE, number).with_args(line)
}

/// Aims a client-less item's target client at `tc`, which the caller owns.
/// This lets the exec hooks find a client to work on while `cmdq_error` still
/// sees the item itself as client-less.
fn aimed_at(mut item: Item, t: &mut Target, tc: *mut client) -> Item {
    unsafe { cmdq_set_target_client(item.ptr(), tc) };
    item.targeting(t)
}

/// A foreign overlay, standing in for whatever another command parked on the
/// client. It is never drawn; only its presence is observed.
const FOREIGN_OVERLAY: Overlay = Overlay::Menu;

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

/// Everything one full exec flow keeps alive, dropped before the globals turn
/// it holds: the queue wiring and terminal buffers go first, then the item
/// and the registered target.
struct Flow {
    item: Item,
    t: Target,
    tc: *mut client,
    queue: Queue,
    _guard: ::std::sync::MutexGuard<'static, ()>,
}

impl Drop for Flow {
    fn drop(&mut self) {
        unsafe { server_client_clear_overlay(self.tc) };
    }
}

impl Flow {
    /// The bytes the client's terminal has been handed so far.
    fn written(&self) -> Vec<u8> {
        let mut out = unsafe { (*self.tc).tty.out.as_ref().unwrap().clone() };
        out.as_slice().to_vec()
    }

    /// The data the running overlay parked on the client.
    fn cdata(&self) -> *mut cmd_display_panes_data {
        unsafe { (*self.tc).overlay_data().display_panes() }
    }

    /// Sends one key through the stored key callback.
    fn press(&self, key: key_code) -> c_int {
        unsafe {
            let mut ev = key_event::default();
            ev.key = key;
            (*self.tc)
                .overlay()
                .key(self.tc, (*self.tc).overlay_data().data(), &raw mut ev)
        }
    }
}

/// Builds the full setup — a registered target, an item owning its own client
/// attached to the target's session, a queue wired to the item so inserted
/// commands land somewhere readable, and a terminal whose output lands in a
/// buffer — runs the parsed `line` through exec once, and hands the result
/// back with the return value it gave.
///
/// The item's shared command-queue state gets the extra reference a real
/// queue would hold on it, and when the command answers wait the item is
/// marked waiting exactly as `cmdq_next` marks one before it parks — the
/// command itself only answers, the queue does the bookkeeping.
fn running(line: &'static CStr) -> (Flow, cmd_retval) {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut item = Item::with_client()
        .from_file(FILE, 1)
        .with_args(line)
        .targeting(&mut t);
    let queue = Queue::for_item(&mut item);
    let tc = item.client();
    unsafe {
        (*tc).name = Some(c"display-fixture".to_owned());
        (*tc).session = t.session();
        (*tc).tty.sx = 80;
        (*tc).tty.sy = 24;
    }
    let term = zeroed_term();
    let out = Box::new(Buf::new());
    unsafe {
        (*tc).tty.owner = crate::server::client_ref_from_ptr(tc).map(|c| c.downgrade());
        (*tc).tty.term = Some(term);
        (*tc).tty.out = Some(out);
    }
    let rv = unsafe { exec_via(&mut item) };
    if rv == CMD_RETURN_WAIT {
        unsafe { (*item.ptr()).flags |= CMDQ_WAITING };
    }
    (
        Flow {
            item,
            t,
            tc,
            queue,
            _guard,
        },
        rv,
    )
}

/// A command-queue list the test owns. With one wired to an item, the real
/// insert-after code splices into this list; with one given to a client, the
/// append code does the same and the client owns it from then on. Nothing
/// global is touched.
struct Queue {
    q: *mut cmdq_list,
    owned: Option<Box<cmdq_list>>,
}

impl Queue {
    fn new() -> Queue {
        let mut owned = crate::cmd::cmdq_new();
        Queue {
            q: &raw mut *owned,
            owned: Some(owned),
        }
    }

    /// Wires the list behind an empty item, exactly as a live queue holding
    /// that item would look from the inside. The queue takes the item over.
    fn for_item(item: &mut Item) -> Queue {
        let q = Queue::new();
        unsafe { item.queue_onto(&mut *q.q) };
        q
    }

    /// Gives the client an empty list as its queue, so `cmdq_append` puts new
    /// commands here instead of on the process-wide one. The client owns it
    /// from then on.
    fn for_client(tc: *mut client) -> Queue {
        let mut q = Queue::new();
        unsafe {
            (*tc).queue = q.owned.take();
        }
        q
    }

    fn ptr(&self) -> *mut cmdq_list {
        self.q
    }

    /// The items queued behind the wired one, in order — what a key press
    /// spliced in after the waiting item.
    fn behind(&self) -> Vec<*mut cmdq_item> {
        unsafe {
            (*self.q)
                .list
                .iter()
                .skip(1)
                .map(|item| item.as_ptr())
                .collect()
        }
    }

    /// Takes those items off again the way `cmdq_remove` would. Assertions on
    /// the commands must be made before calling this.
    fn discard_behind(&self) {
        unsafe { (*self.q).list.truncate(1) };
    }
}

/// Splices `pane` onto the end of `w`'s pane list, the way the fixtures'
/// window builder does, so a window can be given more panes than [`Target`]
/// builds by itself. The window takes the pane over.
unsafe fn append_pane(w: *mut window, pane: &mut Pane) {
    pane.hand_to(w);
}

#[test]
fn the_display_panes_entry_describes_the_display_panes_command() {
    unsafe {
        let e = PANES;
        assert_eq!((*e).name.to_string_lossy(), "display-panes");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "displayp"
        );
        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-bN] [-d duration] [-t target-client] [template]"
        );
        assert_eq!((*e).args.template.to_string_lossy(), "bd:Nt:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 1);
        assert!((*e).args.cb.is_some());

        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, 0);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, 0);

        assert_eq!(
            (*e).flags,
            crate::cmd::cmd_display_panes::CMD_AFTERHOOK
                | crate::cmd::cmd_display_panes::CMD_CLIENT_TFLAG
        );
        assert_eq!((*e).flags, 0x14);
    }
}

#[test]
fn the_parser_resolves_both_names_to_this_entry() {
    let _guard = globals();
    unsafe {
        for (i, line) in [
            c"display-panes",
            c"displayp",
            c"display-panes -b -N -d 750 select-pane",
        ]
        .into_iter()
        .enumerate()
        {
            let mut item = item_for(line, i as u_int + 1);
            assert!(::core::ptr::eq((*item.cmd()).entry, PANES), "{line:?}");
        }
    }
}

/// The callback the parser consults for each positional argument sends every
/// slot to the same place: the template may be either a string or a list of
/// commands, whatever the position and whatever arguments carry it.
#[test]
fn the_argument_callback_always_answers_commands_or_string() {
    let _guard = globals();
    unsafe {
        let cb = (*PANES).args.cb.expect("the entry carries its callback");
        let mut cause = None;

        assert_eq!(
            cb(&args_create(), 0, &mut cause),
            ARGS_PARSE_COMMANDS_OR_STRING
        );

        let none = Args::parse(c"display-panes");
        assert_eq!(
            cb(&*none.ptr(), 0, &mut cause),
            ARGS_PARSE_COMMANDS_OR_STRING
        );

        let some = Args::parse(c"display-panes select-pane");
        for idx in [0 as u_int, 1, 2] {
            assert_eq!(
                cb(&*some.ptr(), idx, &mut cause),
                ARGS_PARSE_COMMANDS_OR_STRING,
                "idx {idx}"
            );
        }
    }
}

#[test]
fn the_reexported_constants_keep_their_values() {
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
    assert_eq!(
        crate::cmd::cmd_display_panes::CMD_AFTERHOOK,
        0x4 as ::core::ffi::c_int
    );
    assert_eq!(
        crate::cmd::cmd_display_panes::CMD_CLIENT_TFLAG,
        0x10 as ::core::ffi::c_int
    );
    assert_eq!(__INT_MAX__, 2147483647);
    assert_eq!(UINT_MAX, 4294967295);
    assert_eq!(KEYC_MASK_MODIFIERS, 0xff0000000000);
    assert_eq!(KEYC_MASK_KEY, 0xffffffffff);
}

/// A client already carrying any overlay is left alone: even before the
/// arguments are looked at, the command answers normal and changes nothing.
#[test]
fn a_busy_client_answers_normal_without_touching_anything() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let (_c_box, tc) = lone_client(80, 24);
    unsafe {
        (*tc).set_overlay(FOREIGN_OVERLAY, OverlayState::None);
    }
    let mut item = aimed_at(item_for(c"display-panes", 1), &mut t, tc);
    unsafe {
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);
        assert_eq!((*tc).overlay(), FOREIGN_OVERLAY);
        assert!((*tc).overlay_data().is_none());
        assert_eq!((*tc).flags, 0);
        assert!((*item.ptr()).flags & CMDQ_WAITING == 0);
    }
}

/// A duration that parses as neither number nor anything else is refused
/// before any overlay exists, and the refusal lands in the config-file cause
/// list because the item has no client of its own.
#[test]
fn an_unusable_delay_is_an_error() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let (_c_box, tc) = lone_client(80, 24);
    let mut item = aimed_at(item_for(c"display-panes -d bogus", 1), &mut t, tc);
    unsafe {
        assert_eq!(exec_via(&mut item), CMD_RETURN_ERROR);
        assert!((*tc).overlay().is_none(), "an overlay was installed");
        assert!((*tc).overlay_data().is_none());
        assert!((*item.ptr()).flags & CMDQ_WAITING == 0);
    }
}

/// A duration beyond the largest the command accepts fails the range check
/// the same way, again leaving nothing displayed.
#[test]
fn an_out_of_range_delay_is_an_error() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let (_c_box, tc) = lone_client(80, 24);
    let mut item = aimed_at(item_for(c"display-panes -d 4294967296", 2), &mut t, tc);
    unsafe {
        assert_eq!(exec_via(&mut item), CMD_RETURN_ERROR);
        assert!((*tc).overlay().is_none(), "an overlay was installed");
        assert!((*tc).overlay_data().is_none());
    }
}

/// The default run parks its data on the client as an overlay with all three
/// callbacks, creates the expiry timer, queues the client for a redraw,
/// freezes the terminal, marks the item waiting and answers wait; taking the
/// overlay down runs the file's own free callback, which frees the prepared
/// state, continues the item and restores the terminal flags.
#[test]
fn the_default_run_installs_its_overlay_and_waits() {
    let (mut f, rv) = running(c"display-panes");
    unsafe {
        assert_eq!(rv, CMD_RETURN_WAIT);

        assert_eq!((*f.tc).overlay(), Overlay::DisplayPanes { keys: true });
        assert_ne!(
            (*f.tc).flags & CLIENT_ALLREDRAWFLAGS,
            0,
            "the client was not queued for redraw"
        );
        assert_eq!(
            (*f.tc).tty.flags & crate::tty::TTY_FREEZE,
            crate::tty::TTY_FREEZE
        );
        assert_eq!(
            (*f.tc).tty.flags & crate::tty::TTY_NOCURSOR,
            crate::tty::TTY_NOCURSOR
        );
        assert!(
            (*f.tc).overlay_timer.is_set(),
            "the expiry timer was not created"
        );

        let cdata = f.cdata();
        assert!(!cdata.is_null(), "no data was parked");
        assert_eq!(
            held_item(&(*cdata).item),
            f.item.ptr(),
            "the waiting item was not kept"
        );
        assert!((*cdata).state.is_some(), "no command state was prepared");

        assert_ne!((*f.item.ptr()).flags & CMDQ_WAITING, 0, "not waiting");

        server_client_clear_overlay(f.tc);

        assert!((*f.tc).overlay().is_none());
        assert!((*f.tc).overlay_data().is_none());
        assert_eq!((*f.tc).tty.flags & crate::tty::TTY_FREEZE, 0);
        assert_eq!((*f.tc).tty.flags & crate::tty::TTY_NOCURSOR, 0);
        assert_eq!(
            (*f.item.ptr()).flags & CMDQ_WAITING,
            0,
            "the item was not continued"
        );
        assert!(f.written().is_empty(), "drawing happened on install");
    }
    drop(f);
}

/// With `-b` the command does not hold its caller: the answer is normal, the
/// overlay still goes up, and nothing about the parked data points at the
/// item, so taking the overlay down later frees cleanly without continuing
/// anything.
#[test]
fn background_mode_answers_normal_and_leaves_the_overlay_up() {
    let (mut f, rv) = running(c"display-panes -b");
    unsafe {
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!(
            (*f.tc).overlay(),
            Overlay::DisplayPanes { keys: true },
            "no overlay was installed"
        );

        let cdata = f.cdata();
        assert!(!cdata.is_null());
        assert!((*cdata).item.is_none(), "the waiting item was kept anyway");
        assert!((*cdata).state.is_some());

        assert_eq!((*f.item.ptr()).flags & CMDQ_WAITING, 0);

        server_client_clear_overlay(f.tc);
        assert!((*f.tc).overlay().is_none());
        assert!((*f.tc).overlay_data().is_none());
    }
    drop(f);
}

/// `-N` installs an overlay with no key callback at all — numbers never reach
/// it by keyboard — while everything else about the display stays the same
/// and the item still waits until the display is taken down.
#[test]
fn no_numbers_mode_installs_an_overlay_without_a_key() {
    let (mut f, rv) = running(c"display-panes -N");
    unsafe {
        assert_eq!(rv, CMD_RETURN_WAIT);
        assert!((*f.tc).overlay().is_some());
        assert!(!(*f.tc).overlay().has_key(), "a key callback was installed");
        assert!((*f.tc).overlay().is_some());
        assert_eq!(held_item(&(*f.cdata()).item), f.item.ptr());
        assert_ne!((*f.item.ptr()).flags & CMDQ_WAITING, 0);
        server_client_clear_overlay(f.tc);
        assert_eq!((*f.item.ptr()).flags & CMDQ_WAITING, 0);
    }
    drop(f);
}

/// An explicit duration overrides the session option: the command still waits
/// and still parks the same data, since only the timer the server arms ever
/// sees the number.
#[test]
fn an_explicit_duration_still_waits_on_the_item() {
    let (mut f, rv) = running(c"display-panes -d 250");
    unsafe {
        assert_eq!(rv, CMD_RETURN_WAIT);
        assert_eq!((*f.tc).overlay(), Overlay::DisplayPanes { keys: true });
        assert_eq!(held_item(&(*f.cdata()).item), f.item.ptr());
        assert_ne!((*f.item.ptr()).flags & CMDQ_WAITING, 0);
        server_client_clear_overlay(f.tc);
        assert_eq!((*f.item.ptr()).flags & CMDQ_WAITING, 0);
    }
    drop(f);
}

/// Keys outside the indexes are refused outright: uppercase letters,
/// punctuation and keys carrying modifier bits all answer -1 without touching
/// the queue, the window's zoom state or the waiting flag.
#[test]
fn keys_outside_the_index_ranges_are_refused() {
    let (mut f, rv) = running(c"display-panes");
    unsafe {
        assert_eq!(rv, CMD_RETURN_WAIT);

        for key in [
            'Z' as i32 as key_code,
            '!' as i32 as key_code,
            KEYC_MASK_MODIFIERS | 'a' as i32 as key_code,
        ] {
            assert_eq!(f.press(key), -1, "key {key:#x} was accepted");
        }

        assert!(f.queue.behind().is_empty(), "something was queued");
        assert_eq!((*f.t.window(0)).flags & crate::window::WINDOW_ZOOMED, 0);
        assert_ne!((*f.item.ptr()).flags & CMDQ_WAITING, 0);
        server_client_clear_overlay(f.tc);
    }
    drop(f);
}

/// Indexes with no pane behind them are swallowed: the command answers one,
/// leaves the window unzoomed, queues nothing and stays up for the next key.
#[test]
fn index_keys_without_a_matching_pane_are_swallowed() {
    let (mut f, rv) = running(c"display-panes");
    unsafe {
        assert_eq!(rv, CMD_RETURN_WAIT);

        for key in ['9' as i32 as key_code, 'z' as i32 as key_code] {
            assert_eq!(f.press(key), 1, "key {key:#x}");
        }

        assert!(f.queue.behind().is_empty(), "something was queued");
        assert_ne!((*f.item.ptr()).flags & CMDQ_WAITING, 0, "not waiting");
        server_client_clear_overlay(f.tc);
        assert_eq!((*f.item.ptr()).flags & CMDQ_WAITING, 0);
    }
    drop(f);
}

/// Pressing a digit selects the pane at that index: the window is unzoomed,
/// the default template is expanded against the pane's own id, and the result
/// is spliced into the queue right after the waiting item, sharing its state.
#[test]
fn a_digit_key_inserts_the_select_pane_after_the_waiting_item() {
    let (mut f, rv) = running(c"display-panes");
    unsafe {
        assert_eq!(rv, CMD_RETURN_WAIT);
        let pane = f.t.pane(0);

        assert_eq!(f.press('0' as i32 as key_code), 1);

        let inserted = *f.queue.behind().first().expect("nothing was inserted");
        assert!(matches!((*inserted).type_0, CmdqType::Command { .. }));
        assert_eq!((*inserted).queue, (*f.item.ptr()).queue);
        assert_eq!(
            crate::cmd::cmd_get_entry(&*(*inserted).cmd())
                .name
                .to_string_lossy(),
            "select-pane"
        );
        assert_eq!(
            seen(args_get(cmd_get_args(&*(*inserted).cmd()), b't' as u_char)),
            format!("%{}", (*pane).id)
        );

        assert_ne!((*f.item.ptr()).flags & CMDQ_WAITING, 0);
        f.queue.discard_behind();
        server_client_clear_overlay(f.tc);
        assert_eq!((*f.item.ptr()).flags & CMDQ_WAITING, 0);
    }
    drop(f);
}

/// Letters continue where the digits stop: `a` is the eleventh pane. A window
/// given a dozen panes answers both letter keys with the pane sitting at the
/// index each letter maps to.
#[test]
fn letter_keys_continue_past_nine_into_the_pane_list() {
    let (mut f, rv) = running(c"display-panes");
    unsafe {
        assert_eq!(rv, CMD_RETURN_WAIT);
        let mut extras: Vec<Pane> = Vec::new();
        for id in 1..12 {
            extras.push(Pane::new(id, 80, 24, 100));
        }
        for p in &mut extras {
            append_pane(f.t.window(0), p);
        }

        for key in ['a' as i32 as key_code, 'b' as i32 as key_code] {
            assert_eq!(f.press(key), 1, "key {key:#x}");
        }

        let seen_ids: Vec<String> = f
            .queue
            .behind()
            .into_iter()
            .map(|p| seen(args_get(cmd_get_args(&*(*p).cmd()), b't' as u_char)))
            .collect();
        assert_eq!(
            seen_ids,
            vec![format!("%{}", 11), format!("%{}", 10)],
            "each press lands directly after the waiting item"
        );

        f.queue.discard_behind();
        server_client_clear_overlay(f.tc);
    }
    drop(f);
}

/// With `-b` there is no waiting item to insert behind, so a key press builds
/// its command free-standing and appends it to the target client's own queue,
/// taking a reference on that client as the queue does.
#[test]
fn a_key_press_in_background_mode_appends_to_the_clients_queue() {
    let (mut f, rv) = running(c"display-panes -b");
    unsafe {
        assert_eq!(rv, CMD_RETURN_NORMAL);
        let queue = Queue::for_client(f.tc);

        let pane = f.t.pane(0);
        assert_eq!(f.press('0' as i32 as key_code), 1);

        assert!(!(*queue.ptr()).list.is_empty(), "nothing was appended");
        let head = (*queue.ptr()).list[0].as_ptr();
        assert_eq!((*queue.ptr()).list.len(), 1);
        assert_eq!((*head).queue, queue.ptr());
        assert_eq!(crate::cmd::cmdq_get_client(&*head), f.tc);
        assert_eq!(
            seen(args_get(cmd_get_args(&*(*head).cmd()), b't' as u_char)),
            format!("%{}", (*pane).id)
        );
        assert!(f.queue.behind().is_empty());

        (*f.tc).queue = None;
    }
    drop(f);
}

/// The template argument replaces `%1` with the chosen pane's target: a
/// custom template produces that command instead of the built-in one.
#[test]
fn a_custom_template_is_expanded_against_the_chosen_pane() {
    let (mut f, rv) = running(c"display-panes -b \"display-message %1\"");
    unsafe {
        assert_eq!(rv, CMD_RETURN_NORMAL);
        let pane = f.t.pane(0);
        let queue = Queue::for_client(f.tc);

        assert_eq!(f.press('0' as i32 as key_code), 1);

        assert!(!(*queue.ptr()).list.is_empty(), "nothing was appended");
        let head = (*queue.ptr()).list[0].as_ptr();
        assert_eq!(
            crate::cmd::cmd_get_entry(&*(*head).cmd())
                .name
                .to_string_lossy(),
            "display-message"
        );
        assert_eq!(
            seen(crate::arguments::args_string(
                cmd_get_args(&*(*head).cmd()),
                0
            )),
            format!("%{}", (*pane).id)
        );

        (*f.tc).queue = None;
    }
    drop(f);
}

/// A redraw context that does not overlap the pane at all stops the draw
/// callback before anything happens: no bytes reach the terminal, the cursor
/// stays where it was and the overlay stays up.
#[test]
fn drawing_skips_a_pane_outside_the_redraw_context() {
    let (mut f, rv) = running(c"display-panes -b");
    unsafe {
        assert_eq!(rv, CMD_RETURN_NORMAL);

        let mut ctx = Box::new(screen_redraw_ctx::default());
        ctx.c = f.tc;
        ctx.sx = 80;
        ctx.sy = 24;
        ctx.ox = 100;
        ctx.oy = 100;

        let overlay = (*f.tc).overlay();
        overlay.draw(f.tc, (*f.tc).overlay_data().data(), &mut ctx);

        assert!(f.written().is_empty(), "bytes were written off-screen");
        assert_eq!((*f.tc).tty.cx, 0);
        assert_eq!((*f.tc).tty.cy, 0);
        assert!((*f.tc).overlay().is_some());
    }
    drop(f);
}

/// Over a real-sized context the draw callback writes exactly what it puts:
/// the clock cells of the pane's number as blank cells first, then the pane
/// size in the top right corner, nothing else, with the cursor left at home
/// and the client's write counter agreeing with the buffer.
#[test]
fn drawing_a_large_pane_writes_its_clock_cells_and_size() {
    let (mut f, rv) = running(c"display-panes -b");
    unsafe {
        assert_eq!(rv, CMD_RETURN_NORMAL);

        let mut ctx = Box::new(screen_redraw_ctx::default());
        ctx.c = f.tc;
        ctx.sx = 80;
        ctx.sy = 24;

        let overlay = (*f.tc).overlay();
        overlay.draw(f.tc, (*f.tc).overlay_data().data(), &mut ctx);

        let bytes = f.written();
        assert!(
            bytes.ends_with(b"80x24"),
            "the size line is missing: {bytes:?}"
        );
        let prefix = &bytes[..bytes.len() - 5];
        assert!(
            !prefix.is_empty(),
            "the clock drew nothing for pane number 0"
        );
        assert!(
            prefix.iter().all(|b| *b == b' '),
            "non-blank bytes among the clock cells: {prefix:?}"
        );
        assert_eq!(
            (*f.tc).written,
            bytes.len(),
            "the write counter disagrees with the buffer"
        );
        assert_eq!((*f.tc).tty.cx, 0, "the cursor was not left at home");
        assert_eq!((*f.tc).tty.cy, 0);

        assert!((*f.tc).overlay().is_some());
        assert_eq!(
            (*f.item.ptr()).flags & CMDQ_WAITING,
            0,
            "a background run left the item waiting"
        );
        server_client_clear_overlay(f.tc);
    }
    drop(f);
}
