//! Unit tests for [`crate::cmd::cmd_send_keys`] — the `send-keys` and
//! `send-prefix` entries, the constants the file re-declares for its own
//! compilation, and every deterministic branch of the shared exec hook and
//! its two injection helpers that the fixtures can reach without a daemon,
//! a live terminal or a spawned process.
//!
//! Exec is reached exactly as the command queue reaches it, through the entry
//! the parsed command resolved to, over items whose arguments come from the
//! real command parser and whose target find state has been filled from a
//! registered session, winlink and window. The key event exec reads lives in
//! the item's command-queue state, so tests write keys and mouse events into
//! it directly before running.
//!
//! Injected keys are observed through a real mode opened on the target pane.
//! Copy mode advertises a key table, so an injected key resolves through the
//! bindings a test puts in that table and splices its command into a queue
//! wired behind the item, where an unbound key leaves the queue alone; it also
//! takes `-X` commands, so the repeat prefix `-N` lands on its entry. Clock
//! mode takes keys through its own callback and closes on any of them, which
//! is how the `-M` path is watched.
//! Client-less items report their refusals into the config-file cause list,
//! which is drained again; the read-only refusal is also driven once through
//! a fixture client whose peer is marked bad, so the error-file message it
//! composes is refused before any descriptor exists.
//!
//! Two places stay out of reach, deliberately: `-R` resets the pane's input
//! parser, which no unit fixture builds (`input_reset` dereferences it), and
//! nothing here runs the ensure_reactor loop, since expiry and real dispatch of
//! queued commands belong to the server.

use crate::arguments::{args_count, args_get, args_has, args_string};
use crate::cfg::cfg_print_causes;
use crate::cmd::cmd_get_args;
use crate::cmd::{CMD_PARSE_ERROR, cmd_parse_from_string};
use crate::cmd::{CmdqType, cmdq_new};
use crate::file::{file_find_ref, file_free};
use crate::proc::PEER_BAD;
use crate::server::message_log;
use crate::session::session_id;
use crate::tests::test_fixtures::{
    Args, Item, KeyTable, Target, ensure_reactor, globals, seen, zeroed,
};
use crate::text::KEYC_CTRL;
use crate::types::*;
use crate::window::{window_pane_reset_mode, window_pane_set_mode};
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr::null_mut;

use crate::cmd::cmd_send_keys::{
    __INT_MAX__, ARGS_PARSE_COMMANDS, ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_INVALID,
    ARGS_PARSE_STRING, CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN,
    CLIENT_READONLY, CMD_AFTERHOOK, CMD_CLIENT_CANFAIL, CMD_CLIENT_CFLAG, CMD_FIND_PANE,
    CMD_FIND_SESSION, CMD_FIND_WINDOW, CMD_READONLY, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
    CMD_RETURN_STOP, CMD_RETURN_WAIT, KEYC_ANY, KEYC_BSPACE, KEYC_DC, KEYC_END, KEYC_F1, KEYC_F12,
    KEYC_HOME, KEYC_IC, KEYC_LITERAL, KEYC_MASK_FLAGS, KEYC_MOUSE, KEYC_MOUSEDOWN1_PANE, KEYC_NONE,
    KEYC_SENT, KEYC_UNKNOWN, KEYC_USER, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE,
    MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL, MSG_EXEC, MSG_EXIT, MSG_EXITED, MSG_EXITING,
    MSG_FLAGS, MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON,
    MSG_IDENTIFY_FEATURES, MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS, MSG_IDENTIFY_OLDCWD,
    MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM, MSG_IDENTIFY_TERMINFO,
    MSG_IDENTIFY_TTYNAME, MSG_LOCK, MSG_OLDSTDERR, MSG_OLDSTDIN, MSG_OLDSTDOUT, MSG_READ,
    MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN, MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN,
    MSG_SUSPEND, MSG_UNLOCK, MSG_VERSION, MSG_WAKEUP, MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_OPEN,
    MSG_WRITE_READY, PANE_LINES_DOUBLE, PANE_LINES_HEAVY, PANE_LINES_NUMBER, PANE_LINES_SIMPLE,
    PANE_LINES_SINGLE, PANE_LINES_SPACES, PANE_REDRAW, PANE_STYLECHANGED, PANE_THEMECHANGED,
    PROGRESS_BAR_ERROR, PROGRESS_BAR_HIDDEN, PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL,
    PROGRESS_BAR_PAUSED, PROMPT_COMMAND, PROMPT_ENTRY, PROMPT_TYPE_COMMAND, PROMPT_TYPE_INVALID,
    PROMPT_TYPE_SEARCH, PROMPT_TYPE_TARGET, PROMPT_TYPE_WINDOW_TARGET, SCREEN_CURSOR_BAR,
    SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT, SCREEN_CURSOR_UNDERLINE,
    STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT, STYLE_ALIGN_LEFT,
    STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_LIST_FOCUS, STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON,
    STYLE_LIST_RIGHT_MARKER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, UINT_MAX, UTF8_DONE, UTF8_ERROR, UTF8_MORE,
    cmd_send_keys_entry, cmd_send_prefix_entry,
};
use crate::window::window_pane_current_mode;

/// Where the tests' items claim to come from, which is what `cfg_add_cause`
/// would report them under.
const FILE: &CStr = c"test-coverage-cmd-send-keys.conf";

/// The key table copy mode hands back, which is where a test binds the keys
/// it expects an injected key to resolve to.
const MODE_TABLE: &CStr = c"copy-mode";

/// The packed [`utf8_char`] [`crate::text::utf8_from_data`] builds for
/// `bytes`, which is what the literal branch of the injection hands on for a
/// wide character.
unsafe fn packed_utf8_char(bytes: &[u8]) -> utf8_char {
    unsafe {
        assert!(bytes.len() <= 3, "a wide character of three bytes at most");
        let mut ud = zeroed::<utf8_data>();
        ud.size = bytes.len() as u_char;
        ud.width = bytes.len() as u_char - 1;
        for (i, b) in bytes.iter().enumerate() {
            ud.data[i] = *b;
        }
        let mut uc: utf8_char = 0;
        assert_eq!(crate::text::utf8_from_data(&ud, &raw mut uc), UTF8_DONE);
        uc
    }
}

/// A real mode, opened on the target pane for the length of one exec run and
/// closed again on the way out. Which mode decides what an injected key can be
/// watched through: copy mode routes keys through [`MODE_TABLE`], clock mode
/// takes them through its own callback and closes the pane's mode on any key.
struct Mode {
    wp: *mut window_pane,
    _args: Args,
}

impl Mode {
    fn open(t: &mut Target, mode: WindowMode) -> Mode {
        ensure_reactor();
        let wp = t.pane(0);
        let args = Args::parse(c"copy-mode");
        unsafe {
            assert!((*wp).modes.is_empty(), "a mode is already open");
            let mut fs = t.state();
            assert_eq!(
                window_pane_set_mode(wp, wp, mode, &raw mut fs, args.ptr()),
                0,
                "the mode did not open"
            );
        }
        Mode { wp, _args: args }
    }

    fn entry(&self) -> *mut window_mode_entry {
        unsafe { window_pane_current_mode(self.wp) }
    }
}

impl Drop for Mode {
    fn drop(&mut self) {
        unsafe { window_pane_reset_mode(self.wp) };
    }
}

/// The `send-keys` entry as a raw pointer, so field reads stay explicit
/// unsafe dereferences rather than references into a `static mut`.
fn keys_entry() -> *const cmd_entry {
    &raw const cmd_send_keys_entry
}

/// The `send-prefix` entry likewise.
fn prefix_entry() -> *const cmd_entry {
    &raw const cmd_send_prefix_entry
}

/// Runs the parsed command an item carries through its own entry's exec hook,
/// the way the command queue calls it.
unsafe fn exec(item: &mut Item) -> cmd_retval {
    unsafe {
        let e = (*item.cmd()).entry;
        assert!(
            ::core::ptr::eq(e, keys_entry()) || ::core::ptr::eq(e, prefix_entry()),
            "the item is not running send-keys"
        );
        (e.exec)(&*item.cmd(), item.ptr())
    }
}

/// An item with its own client attached to `t`, whose find state points at
/// the target's registered session, current winlink and active pane.
fn aimed(line: &'static CStr, t: &mut Target) -> Item {
    Item::with_client()
        .from_file(FILE, 1)
        .with_args(line)
        .targeting(t)
}

/// The lines the server has recorded so far, oldest first. Entries
/// accumulate across the whole binary, so assertions look for their own
/// wording rather than exact contents.
unsafe fn logged_messages() -> Vec<String> {
    unsafe {
        let mut out = Vec::new();
        for m in message_log.queue().iter() {
            out.push(seen(m.msg.as_ptr()));
        }
        out
    }
}

/// Hands cfg.rs's cause list to `cfg_print_causes`, which frees every entry.
/// With no client behind them each cause only reaches `log_debug`.
unsafe fn drain_config_causes() {
    unsafe {
        let mut item = Item::new();
        cfg_print_causes(item.ptr());
    }
}

/// A peer for the fixture clients, marked bad so `proc_send` refuses any
/// message before it reaches an imsg buffer underneath it.
fn bad_peer() -> Box<tmuxpeer> {
    ensure_reactor();
    let mut p = zeroed::<tmuxpeer>();
    p.flags |= PEER_BAD;
    p
}

/// What has been written to `c`'s error stream, freeing the file entry
/// afterwards so the stream is empty for the next ask. Empty when nothing
/// was written.
unsafe fn take_stderr(c: *mut client) -> String {
    unsafe {
        let Some(cf) = file_find_ref(&raw mut (*c).files, 2 as c_int) else {
            return String::new();
        };
        let cf_ptr = cf.as_ptr();
        let text = String::from_utf8_lossy((*cf_ptr).buffer.as_mut().as_slice()).into_owned();
        file_free(cf);
        text
    }
}

/// A command-queue list the test owns, wired behind an item exactly as a live
/// queue holding that item would look from the inside, so code inserting
/// commands after it splices somewhere readable instead of nowhere.
struct Queue(Box<cmdq_list>);

impl Queue {
    fn for_item(item: &mut Item) -> Queue {
        let mut q = Queue(cmdq_new());
        unsafe { item.queue_onto(&mut q.0) };
        q
    }

    fn ptr(&self) -> *mut cmdq_list {
        &raw const *self.0 as *mut cmdq_list
    }
}

/// The command lines the injected keys dispatched, in the order the queue
/// holds them behind `item`. They are taken off again, so an item can be run
/// more than once.
unsafe fn dispatched(item: *mut cmdq_item, queue: &Queue) -> Vec<String> {
    unsafe {
        let mut out = Vec::new();
        for p in (*queue.ptr())
            .list
            .iter()
            .map(|queued| queued.as_ptr())
            .filter(|&queued| queued != item)
        {
            assert!(
                matches!((*p).type_0, CmdqType::Command { .. }),
                "something else was queued"
            );
            assert_eq!(
                crate::cmd::cmd_get_entry(&*(*p).cmd())
                    .name
                    .to_string_lossy(),
                "display-message"
            );
            out.push(seen(args_string(cmd_get_args(&*(*p).cmd()), 0)));
        }
        (*queue.ptr()).list.retain(|queued| queued.as_ptr() == item);
        out
    }
}

/// A queue of the target client's own, which is where a key binding lands
/// when the injection has no item for it to sit behind. Reading it empties it,
/// and it is freed with the fixture.
struct ClientQueue(*mut client);

impl ClientQueue {
    fn new(c: *mut client) -> ClientQueue {
        unsafe {
            assert!((*c).queue.is_none(), "the client already has a queue");
            (*c).queue = Some(cmdq_new());
        }
        ClientQueue(c)
    }

    fn list(&self) -> *mut cmdq_list {
        unsafe {
            (*self.0)
                .queue
                .as_mut()
                .map(|q| &raw mut **q)
                .expect("a queue")
        }
    }

    /// The command lines appended to it, taken off and freed as they are read.
    unsafe fn taken(&self) -> Vec<String> {
        unsafe {
            let queue = self.list();
            let out = (*queue)
                .list
                .iter()
                .map(|queued| {
                    assert!(
                        matches!(queued.item().type_0, CmdqType::Command { .. }),
                        "something else was queued"
                    );
                    seen(args_string(cmd_get_args(&*queued.item().cmd()), 0))
                })
                .collect();
            (*queue).list.clear();
            out
        }
    }
}

impl Drop for ClientQueue {
    fn drop(&mut self) {
        unsafe { drop((*self.0).queue.take()) };
    }
}

/// A key table of the tests' own under [`MODE_TABLE`], carrying one
/// `display-message` per key so [`dispatched`] can name what arrived.
fn bindings(keys: &[(key_code, &'static CStr)]) -> KeyTable {
    let mut table = KeyTable::new(MODE_TABLE.to_str().expect("a table name"));
    for (key, line) in keys {
        table.bind(*key, line, None);
    }
    table
}

#[test]
fn the_entries_advertise_their_commands_and_share_one_hook() {
    unsafe {
        let e = keys_entry();
        assert_eq!((*e).name.to_string_lossy(), "send-keys");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "send"
        );
        assert_eq!((*e).args.template.to_string_lossy(), "c:FHKlMN:Rt:X");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, -1);
        assert!((*e).args.cb.is_none());
        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-FHKlMRX] [-c target-client] [-N repeat-count] [-t target-pane] [key ...]"
        );
        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, 0);
        assert_eq!(
            (*e).flags,
            CMD_AFTERHOOK | CMD_CLIENT_CFLAG | CMD_CLIENT_CANFAIL | CMD_READONLY
        );

        let p = prefix_entry();
        assert_eq!((*p).name.to_string_lossy(), "send-prefix");
        assert!((*p).alias.is_none());
        assert_eq!((*p).args.template.to_string_lossy(), "2t:");
        assert_eq!((*p).args.lower, 0);
        assert_eq!((*p).args.upper, 0);
        assert!((*p).args.cb.is_none());
        assert_eq!((*p).usage.to_string_lossy(), "[-2] [-t target-pane]");
        assert_eq!((*p).source.flag, 0);
        assert_eq!((*p).source.type_0, CMD_FIND_PANE);
        assert_eq!((*p).source.flags, 0);
        assert_eq!((*p).target.flag, b't' as c_char);
        assert_eq!((*p).target.type_0, CMD_FIND_PANE);
        assert_eq!((*p).target.flags, 0);
        assert_eq!((*p).flags, CMD_AFTERHOOK);
        assert!(::core::ptr::fn_addr_eq((*p).exec, (*e).exec));

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
            (STYLE_DEFAULT_BASE, 0),
            (STYLE_DEFAULT_PUSH, 1),
            (STYLE_DEFAULT_POP, 2),
            (STYLE_DEFAULT_SET, 3),
            (STYLE_RANGE_NONE, 0),
            (STYLE_RANGE_LEFT, 1),
            (STYLE_RANGE_RIGHT, 2),
            (STYLE_RANGE_PANE, 3),
            (STYLE_RANGE_WINDOW, 4),
            (STYLE_RANGE_SESSION, 5),
            (STYLE_RANGE_USER, 6),
            (STYLE_RANGE_CONTROL, 7),
            (STYLE_LIST_OFF, 0),
            (STYLE_LIST_ON, 1),
            (STYLE_LIST_FOCUS, 2),
            (STYLE_LIST_LEFT_MARKER, 3),
            (STYLE_LIST_RIGHT_MARKER, 4),
            (STYLE_ALIGN_DEFAULT, 0),
            (STYLE_ALIGN_LEFT, 1),
            (STYLE_ALIGN_CENTRE, 2),
            (STYLE_ALIGN_RIGHT, 3),
            (STYLE_ALIGN_ABSOLUTE_CENTRE, 4),
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
            (UTF8_MORE, 0),
            (UTF8_DONE, 1),
            (UTF8_ERROR, 2),
        ] {
            assert_eq!(constant, value);
        }
        for (constant, value) in [
            (CMD_RETURN_NORMAL, 0 as cmd_retval),
            (CMD_RETURN_WAIT, 1),
            (CMD_RETURN_STOP, 2),
            (CMD_RETURN_ERROR, -1i32 as cmd_retval),
        ] {
            assert_eq!(constant, value);
        }
        assert_eq!(KEYC_NONE, 8589934592);
        assert_eq!(KEYC_UNKNOWN, 8589934593);
        assert_eq!(KEYC_BSPACE, 8589934599);
        assert_eq!(KEYC_ANY, 8589934596);
        assert_eq!(KEYC_USER, 4294967296);
        assert_eq!(KEYC_F1, 8589934600);
        assert_eq!(KEYC_F12, 8589934611);
        assert_eq!(KEYC_HOME, 8589934614);
        assert_eq!(KEYC_DC, 8589934613);
        assert_eq!(KEYC_IC, 8589934612);
        assert_eq!(KEYC_END, 8589934615);
        assert_eq!(KEYC_MOUSE, 8589934641);
        assert_eq!(KEYC_MOUSEDOWN1_PANE, 17179869440);
        assert_eq!(KEYC_LITERAL, 0x1000000000000);
        assert_eq!(KEYC_SENT, 0x40000000000000);
        assert_eq!(KEYC_MASK_FLAGS, 0xff000000000000);
        assert_eq!(PANE_REDRAW, 0x1);
        assert_eq!(PANE_STYLECHANGED, 0x1000);
        assert_eq!(PANE_THEMECHANGED, 0x2000);
        assert_eq!(CMD_READONLY, 0x2);
        assert_eq!(CMD_AFTERHOOK, 0x4);
        assert_eq!(CMD_CLIENT_CFLAG, 0x8);
        assert_eq!(CMD_CLIENT_CANFAIL, 0x20);
        assert_eq!(CLIENT_READONLY, 0x800);
        assert_eq!(__INT_MAX__, 2147483647);
        assert_eq!(UINT_MAX, 4294967295);
    }
}

#[test]
fn parsing_resolves_both_names_the_alias_and_their_letters() {
    let _guard = globals();
    unsafe {
        let plain = Args::parse(c"send-keys C-a");
        assert!(::core::ptr::eq((*plain.cmd()).entry, keys_entry()));

        let alias = Args::parse(c"send -X");
        assert!(::core::ptr::eq((*alias.cmd()).entry, keys_entry()));
        assert_eq!(args_has(&*alias.ptr(), b'X'), 1);

        let full = Args::parse(c"send-keys -c foo -FHKlMR -N 5 -t 0.0 -X abc");
        assert!(::core::ptr::eq((*full.cmd()).entry, keys_entry()));
        let a = full.ptr();
        assert_eq!(args_count(&*a), 1);
        assert_eq!(seen(args_string(&*a, 0)), "abc");
        for flag in *b"FHKlMRX" {
            assert_eq!(args_has(&*a, flag), 1, "-{flag} missing");
        }
        assert_eq!(seen(args_get(&*a, b'N')), "5");
        assert_eq!(seen(args_get(&*a, b'c')), "foo");
        assert_eq!(seen(args_get(&*a, b't')), "0.0");

        let prefix = Args::parse(c"send-prefix -2");
        assert!(::core::ptr::eq((*prefix.cmd()).entry, prefix_entry()));
        assert_eq!(args_has(&*prefix.ptr(), b'2'), 1);

        let mut unknown = cmd_parse_from_string(c"send-keys -q".as_ptr(), null_mut());
        assert_eq!(unknown.status, CMD_PARSE_ERROR);
        let err = unknown.take_error();
        assert!(err.contains("unknown flag"), "{err}");
        assert!(err.contains("-q"), "{err}");

        let mut hungry = cmd_parse_from_string(c"send-keys -N".as_ptr(), null_mut());
        assert_eq!(hungry.status, CMD_PARSE_ERROR);
        let err = hungry.take_error();
        assert!(err.contains("expects an argument"), "{err}");
        assert!(err.contains("-N"), "{err}");
    }
}

#[test]
fn a_read_only_client_is_refused_before_anything_is_sent() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut item = aimed(c"send-keys C-a", &mut t);
    unsafe {
        let c = item.client();
        (*c).name = Some(c"ro-fixture".to_owned());
        (*c).peer = Some(bad_peer());
        (*c).flags |= CLIENT_READONLY as u64;

        assert_eq!(exec(&mut item), CMD_RETURN_ERROR);
        assert_eq!((*c).retval, 1);
        assert!(
            logged_messages()
                .iter()
                .any(|m| m.contains("ro-fixture") && m.contains("client is read-only")),
            "the refusal did not reach the server message log"
        );
        let text = take_stderr(c);
        assert!(text.contains("client is read-only"), "{text:?}");
        assert_eq!((*c).flags & CLIENT_READONLY as u64, CLIENT_READONLY as u64);
    }
}

#[test]
fn an_unusable_repeat_count_is_an_error() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut item = Item::new()
        .from_file(FILE, 2)
        .with_args(c"send-keys -N bogus C-a")
        .targeting(&mut t);
    unsafe {
        assert_eq!(exec(&mut item), CMD_RETURN_ERROR);
    }
    unsafe { drain_config_causes() };
}

#[test]
fn a_repeat_prefix_needs_a_mode_that_takes_commands() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut item = Item::new()
        .from_file(FILE, 3)
        .with_args(c"send-keys -N 3 -X")
        .targeting(&mut t);
    let _mode = Mode::open(&mut t, WindowMode::Clock);
    unsafe {
        assert_eq!(exec(&mut item), CMD_RETURN_ERROR);
    }
    unsafe { drain_config_causes() };
}

#[test]
fn x_without_a_mode_is_refused() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut item = Item::new()
        .from_file(FILE, 3)
        .with_args(c"send-keys -X")
        .targeting(&mut t);
    unsafe {
        assert_eq!(exec(&mut item), CMD_RETURN_ERROR);
    }
    unsafe { drain_config_causes() };
}

#[test]
fn x_runs_the_mode_command_and_carries_the_repeat_count() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut item = aimed(c"send-keys -N 7 -X", &mut t);
    let mode = Mode::open(&mut t, WindowMode::Copy);
    unsafe {
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(
            (*mode.entry()).prefix,
            7,
            "the repeat count did not reach the mode entry"
        );
    }
}

#[test]
fn m_without_a_mouse_target_is_an_error() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut item = Item::new()
        .from_file(FILE, 4)
        .with_args(c"send-keys -M")
        .targeting(&mut t);
    unsafe {
        assert_eq!((*(*item.ptr()).state()).event.m.valid, 0);
        assert_eq!(exec(&mut item), CMD_RETURN_ERROR);
    }
    unsafe { drain_config_causes() };
}

#[test]
fn m_delivers_the_mouse_key_to_its_pane() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut item = aimed(c"send-keys -M", &mut t);
    let _mode = Mode::open(&mut t, WindowMode::Clock);
    unsafe {
        let m = &raw mut (*(*item.ptr()).state()).event.m;
        (*m).valid = 1;
        (*m).s = session_id(t.session()) as c_int;
        (*m).w = -1;
        (*m).wp = -1;
        (*m).key = KEYC_MOUSEDOWN1_PANE as key_code;

        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        assert!(
            (*t.pane(0)).modes.is_empty(),
            "the mouse key never reached the mode, which closes on any key"
        );
    }
}

#[test]
fn send_prefix_sends_the_session_prefix_options() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut plain = aimed(c"send-prefix", &mut t);
    let mut doubled = aimed(c"send-prefix -2", &mut t);
    let table = bindings(&[(KEYC_CTRL | b'b' as u64, c"display-message prefix")]);
    let _mode = Mode::open(&mut t, WindowMode::Copy);
    let plain_queue = Queue::for_item(&mut plain);
    let doubled_queue = Queue::for_item(&mut doubled);
    unsafe {
        assert!(::core::ptr::eq((*plain.cmd()).entry, prefix_entry()));
        assert_eq!(exec(&mut plain), CMD_RETURN_NORMAL);
        assert_eq!(dispatched(plain.ptr(), &plain_queue), vec!["prefix"]);

        assert_eq!(exec(&mut doubled), CMD_RETURN_NORMAL);
        assert!(
            dispatched(doubled.ptr(), &doubled_queue).is_empty(),
            "an unset prefix2 sent a key anyway"
        );
    }
    drop(table);
}

#[test]
fn without_arguments_the_event_key_itself_is_sent() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut bare = aimed(c"send-keys", &mut t);
    let mut counted = aimed(c"send-keys -N 4", &mut t);
    let table = bindings(&[(b'q' as key_code, c"display-message q")]);
    let _mode = Mode::open(&mut t, WindowMode::Copy);
    let bare_queue = ClientQueue::new(bare.client());
    let counted_queue = ClientQueue::new(counted.client());
    unsafe {
        (*(*bare.ptr()).state()).event.key = b'q' as key_code;
        assert_eq!(exec(&mut bare), CMD_RETURN_NORMAL);
        assert_eq!(bare_queue.taken(), vec!["q"]);

        assert!(::core::ptr::eq((*counted.cmd()).entry, keys_entry()));
        assert_eq!(exec(&mut counted), CMD_RETURN_NORMAL);
        assert!(
            counted_queue.taken().is_empty(),
            "-N repeated nothing to repeat"
        );
    }
    drop(table);
}

#[test]
fn keys_repeat_for_every_repeat_count() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut item = aimed(c"send-keys -N 3 abc", &mut t);
    let table = bindings(&[
        (b'a' as key_code, c"display-message a"),
        (b'b' as key_code, c"display-message b"),
        (b'c' as key_code, c"display-message c"),
    ]);
    let _mode = Mode::open(&mut t, WindowMode::Copy);
    let queue = Queue::for_item(&mut item);
    unsafe {
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(
            dispatched(item.ptr(), &queue),
            vec!["a", "b", "c", "a", "b", "c", "a", "b", "c"]
        );
    }
    drop(table);
}

#[test]
fn hex_arguments_become_literal_bytes_or_are_refused() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut good = aimed(c"send-keys -H 41", &mut t);
    let mut letters = aimed(c"send-keys -H zz", &mut t);
    let mut trailing = aimed(c"send-keys -H 4x", &mut t);
    let mut huge = aimed(c"send-keys -H 100", &mut t);
    let table = bindings(&[(0x41 as key_code, c"display-message A")]);
    let _mode = Mode::open(&mut t, WindowMode::Copy);
    let good_queue = Queue::for_item(&mut good);
    let letters_queue = Queue::for_item(&mut letters);
    let trailing_queue = Queue::for_item(&mut trailing);
    let huge_queue = Queue::for_item(&mut huge);
    unsafe {
        assert_eq!(exec(&mut good), CMD_RETURN_NORMAL);
        assert_eq!(dispatched(good.ptr(), &good_queue), vec!["A"]);

        for (bad, queue) in [
            (&mut letters, &letters_queue),
            (&mut trailing, &trailing_queue),
            (&mut huge, &huge_queue),
        ] {
            assert_eq!(exec(bad), CMD_RETURN_NORMAL, "a refusal became an error");
            assert!(
                dispatched(bad.ptr(), queue).is_empty(),
                "an unusable hex string was sent"
            );
        }
    }
    drop(table);
}

#[test]
fn literal_arguments_inject_each_character() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut marked = aimed(c"send-keys -l héllo", &mut t);
    let mut fallback = aimed(c"send-keys hello", &mut t);
    let mut named = aimed(c"send-keys C-a", &mut t);
    let accented = unsafe { packed_utf8_char(&[0xc3, 0xa9]) as key_code };
    let table = bindings(&[
        (b'h' as key_code, c"display-message h"),
        (b'e' as key_code, c"display-message e"),
        (b'l' as key_code, c"display-message l"),
        (b'o' as key_code, c"display-message o"),
        (KEYC_CTRL | b'a' as u64, c"display-message C-a"),
    ]);
    let mut accents = KeyTable::new(MODE_TABLE.to_str().expect("a table name"));
    accents.bind(accented, c"display-message accented", None);
    let _mode = Mode::open(&mut t, WindowMode::Copy);
    let marked_queue = Queue::for_item(&mut marked);
    let fallback_queue = Queue::for_item(&mut fallback);
    let named_queue = Queue::for_item(&mut named);
    unsafe {
        assert_eq!(exec(&mut marked), CMD_RETURN_NORMAL);
        assert_eq!(
            dispatched(marked.ptr(), &marked_queue),
            vec!["h", "accented", "l", "l", "o"]
        );

        assert_eq!(exec(&mut fallback), CMD_RETURN_NORMAL);
        assert_eq!(
            dispatched(fallback.ptr(), &fallback_queue),
            vec!["h", "e", "l", "l", "o"]
        );

        assert_eq!(exec(&mut named), CMD_RETURN_NORMAL);
        assert_eq!(dispatched(named.ptr(), &named_queue), vec!["C-a"]);
    }
    drop(accents);
    drop(table);
}

#[test]
fn k_hands_keys_straight_to_the_target_client() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut with_client = aimed(c"send-keys -K C-a", &mut t);
    let mut alone = Item::new()
        .from_file(FILE, 5)
        .with_args(c"send-keys -K C-a")
        .targeting(&mut t);
    let table = bindings(&[(KEYC_CTRL | b'a' as u64, c"display-message hit")]);
    let _mode = Mode::open(&mut t, WindowMode::Copy);
    let with_client_queue = Queue::for_item(&mut with_client);
    let alone_queue = Queue::for_item(&mut alone);
    unsafe {
        assert!((*with_client.client()).session.is_null(), "a live session");
        assert_eq!(exec(&mut with_client), CMD_RETURN_NORMAL);
        assert!(
            dispatched(with_client.ptr(), &with_client_queue).is_empty(),
            "a -K key reached the pane anyway"
        );

        assert!(crate::cmd::cmdq_get_target_client(&*alone.ptr()).is_null());
        assert_eq!(exec(&mut alone), CMD_RETURN_NORMAL);
        assert!(dispatched(alone.ptr(), &alone_queue).is_empty());
    }
    drop(table);
}

#[test]
fn an_unbound_mode_key_leaves_the_queue_alone() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let table = bindings(&[(KEYC_CTRL | b'm' as u64, c"display-message hi")]);
    let mut item = aimed(c"send-keys x", &mut t);
    let _mode = Mode::open(&mut t, WindowMode::Copy);
    let queue = Queue::for_item(&mut item);
    unsafe {
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        assert_eq!((*queue.ptr()).list.len(), 1, "something was spliced in");
        assert_eq!((*queue.ptr()).list[0].as_ptr(), item.ptr());
    }
    drop(table);
}

#[test]
fn a_bound_mode_key_dispatches_behind_the_item() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let table = bindings(&[(b'x' as key_code, c"display-message hi")]);
    let mut item = aimed(c"send-keys x", &mut t);
    let _mode = Mode::open(&mut t, WindowMode::Copy);
    let queue = Queue::for_item(&mut item);
    unsafe {
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);

        assert_eq!((*queue.ptr()).list.len(), 2, "nothing was spliced in");
        let inserted = (*queue.ptr()).list[1].as_ptr();
        assert!(matches!((*inserted).type_0, CmdqType::Command { .. }));
        assert_eq!((*inserted).queue, queue.ptr());
        assert_eq!(
            crate::cmd::cmd_get_entry(&*(*inserted).cmd())
                .name
                .to_string_lossy(),
            "display-message"
        );
        assert_eq!(
            seen(args_string(cmd_get_args(&*(*inserted).cmd()), 0)),
            "hi"
        );

        (*queue.ptr()).list.truncate(1);
        assert_eq!((*queue.ptr()).list[0].as_ptr(), item.ptr());
    }
    drop(table);
}
