//! Unit tests for [`crate::cmd::cmd_refresh_client`], the exec hook behind the
//! `refresh-client` command.
//!
//! What is exercised is everything the command decides before anything leaves
//! the process: which entry ran, how the plain, `-S`, `-l`, `-f`/`-F`, `-r`,
//! panning and control-only `-A`/`-B`/`-C` forms are told apart, and what each
//! form leaves behind — redraw flags, pan anchors, client flags, control-mode
//! subscriptions, per-pane pause state and control window sizes. Every fixture
//! client reports to a peer marked bad, which turns `proc_send` away at the
//! door, so no descriptor, event or other process is ever reached; the control
//! fixtures write into a buffer event over an in-process socket pair, where
//! what was sent can be read back byte for byte. Errors travel through the
//! unattached branch of `cmdq_error` into the server's message log, where they
//! can be read back; assertions look only at lines appended after the count was
//! taken, so lines from earlier tests stay out of the way.

use crate::arguments::{args_count, args_get, args_has, args_string};
use crate::cmd::CMD_RETURN_NORMAL;
use crate::cmd::cmd_get_args;
use crate::cmd::cmd_refresh_client::{
    CLIENT_CONTROL, CLIENT_SIZECHANGED, CLIENT_STATUSFORCE, CLIENT_WINDOWSIZECHANGED,
    CMD_AFTERHOOK, CMD_CLIENT_TFLAG, CMD_FIND_PANE, CONTROL_SUB_ALL_PANES, CONTROL_SUB_ALL_WINDOWS,
    CONTROL_SUB_PANE, CONTROL_SUB_SESSION, CONTROL_SUB_WINDOW, INT_MAX, MSG_FLAGS, WINDOW_MAXIMUM,
    WINDOW_MINIMUM, cmd_refresh_client_entry,
};
use crate::cmd::cmdq_set_target_client;
use crate::control::{CONTROL_PANE_OFF, CONTROL_PANE_PAUSED, control_remove_sub, control_state};
use crate::proc::PEER_BAD;
use crate::reactor::Timer;
use crate::server::client_get_pan_window;
use crate::server::message_log;
use crate::server::{CLIENT_ALLREDRAWFLAGS, CLIENT_REDRAWWINDOW};
use crate::server::{CLIENT_ATTACHED, CLIENT_IGNORESIZE, CLIENT_READONLY, CLIENT_REDRAWSTATUS};
use crate::tests::test_fixtures::{
    Clients, Item, Pane, Registry, Session, StreamBuffer, Window, globals, link, seen, unlink,
    zeroed,
};
use crate::types::*;
use ::core::ffi::{c_char, c_int};
use ::core::ptr::null_mut;
use ::std::ffi::CString;

/// A peer for the fixture clients, marked bad so `proc_send` refuses any
/// message before it reaches the buffer underneath it.
fn bad_peer() -> Box<tmuxpeer> {
    let mut p = zeroed::<tmuxpeer>();
    p.flags |= PEER_BAD;
    p
}

/// Gives `c` its peer, its session — which may be null — and exactly `flags`.
unsafe fn wire(c: *mut client, session: *mut session, flags: uint64_t) {
    unsafe {
        (*c).peer = Some(bad_peer());
        (*c).session = session;
        (*c).flags = flags;
    }
}

/// Points the item's caller and target client where the test wants them.
unsafe fn aim(item: &mut Item, caller: *mut client, target: *mut client) {
    unsafe {
        item.set_client(caller);
        cmdq_set_target_client(item.ptr(), target);
    }
}

/// Runs the item's parsed command through the entry's exec hook, the way the
/// command queue would.
unsafe fn run(item: &mut Item) -> cmd_retval {
    unsafe {
        let exec = cmd_refresh_client_entry.exec;
        exec(&*item.cmd(), item.ptr())
    }
}

/// The lines the server has recorded so far, oldest first.
unsafe fn server_messages() -> Vec<String> {
    unsafe {
        let mut out = Vec::new();
        for m in message_log.queue().iter() {
            out.push(seen(m.msg.as_ptr()));
        }
        out
    }
}

/// A control-mode client's state: real trees, a timer of its own and a buffer
/// event over a socket pair standing in for the control stream. The client
/// owns the state once [`Self::attach`] gives it one. Anything the command
/// installs is taken down again here, so no allocation outlives its test and
/// no armed timer outlives the memory it points into.
struct ControlState {
    bev: StreamBuffer,
    named: Vec<CString>,
    client: *mut client,
}

impl ControlState {
    fn new() -> ControlState {
        ControlState {
            bev: StreamBuffer::new(),
            named: Vec::new(),
            client: null_mut(),
        }
    }

    fn ptr(&mut self) -> *mut control_state {
        unsafe {
            &raw mut **(*self.client)
                .control_state
                .as_mut()
                .expect("the client keeps its control state")
        }
    }

    /// Gives the control-mode client these subscriptions belong to a state
    /// writing through the buffer event, so that they can be taken down again
    /// through the command's own removal path.
    fn attach(&mut self, c: *mut client) {
        unsafe {
            let cs = (*c)
                .control_state
                .insert(Box::new(control_state::default()));
            cs.write_event = self.bev.ptr();
        }
        self.client = c;
    }

    /// Records a subscription name this test asked for, so it can be freed
    /// again even if an assertion stops before the command itself would.
    fn track(&mut self, name: &str) {
        self.named.push(CString::new(name).expect("no NUL"));
    }

    /// What has been written to the control stream since the last time this
    /// was asked.
    fn written(&self) -> Vec<u8> {
        self.bev.written()
    }
}

impl Drop for ControlState {
    fn drop(&mut self) {
        unsafe {
            if self.client.is_null() {
                return;
            }
            let cs = self.ptr();
            (*cs).subs_timer.disarm();
            for name in &self.named {
                control_remove_sub(self.client, name.as_ptr());
            }
            for cp in ::core::mem::take(&mut (*cs).panes).into_values() {
                drop(cp);
            }
        }
    }
}

/// Every subscription in `cs`, walked in name order.
unsafe fn collect_subs(cs: *mut control_state) -> Vec<(String, c_char, u_int, String)> {
    unsafe {
        let mut out = Vec::new();
        for sub in (*cs).subs.values() {
            let sub = &raw const **sub;
            out.push((
                seen((*sub).name.as_ptr()),
                (*sub).type_0 as c_char,
                (*sub).id,
                seen((*sub).format.as_ptr()),
            ));
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }
}

/// Every control pane in `cs`, walked in pane-id order.
unsafe fn collect_panes(cs: *mut control_state) -> Vec<(u_int, c_int)> {
    unsafe { (*cs).panes.values().map(|cp| (cp.pane, cp.flags)).collect() }
}

/// Counts the per-window sizes recorded against `c`, dropping them on the way
/// out, since nothing here tears the client down through `server_destroy`.
unsafe fn take_client_windows(c: *mut client) -> u_int {
    unsafe { ::core::mem::take(&mut (*c).windows).len() as u_int }
}

#[test]
fn the_entry_advertises_its_name_alias_arguments_and_flags() {
    let _guard = globals();
    unsafe {
        let e = &raw const cmd_refresh_client_entry;
        assert_eq!((*e).name.to_string_lossy(), "refresh-client");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "refresh"
        );
        assert_eq!(
            (*e).args.template.to_string_lossy(),
            "A:B:cC:Df:r:F:lLRSt:U"
        );
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 1);
        assert!((*e).args.cb.is_none());
        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-cDlLRSU] [-A pane:state] [-B name:what:format] [-C XxY] [-f flags] \
             [-r pane:report] [-t target-client] [adjustment]"
        );
        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, 0);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, 0);
        assert_eq!((*e).flags, CMD_AFTERHOOK | CMD_CLIENT_TFLAG);

        assert_eq!(INT_MAX, 2147483647);
        assert_eq!(WINDOW_MINIMUM, 1);
        assert_eq!(WINDOW_MAXIMUM, 10000);
        assert_eq!(MSG_FLAGS, 218);
        assert_eq!(CLIENT_CONTROL, 0x2000);
        assert_eq!(CLIENT_STATUSFORCE, 0x80000);
        assert_eq!(CLIENT_SIZECHANGED, 0x400000);
        assert_eq!(CLIENT_WINDOWSIZECHANGED, 0x400000000u64);
        assert_eq!(CONTROL_SUB_SESSION, 0);
        assert_eq!(CONTROL_SUB_PANE, 1);
        assert_eq!(CONTROL_SUB_ALL_PANES, 2);
        assert_eq!(CONTROL_SUB_WINDOW, 3);
        assert_eq!(CONTROL_SUB_ALL_WINDOWS, 4);
    }
}

#[test]
fn a_plain_refresh_marks_the_target_for_a_full_redraw() {
    let _guard = globals();
    let mut clients = Clients::new();
    let caller = clients.add("caller", 80, 24);
    let target = clients.add("target", 80, 24);
    unsafe {
        wire(caller, null_mut(), CLIENT_ATTACHED as uint64_t);
        wire(target, null_mut(), 0);

        let mut item = Item::new().with_args(c"refresh-client");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        let want = CLIENT_STATUSFORCE as uint64_t | CLIENT_ALLREDRAWFLAGS;
        assert_eq!((*target).flags & want, want, "the target was not redrawn");
        assert_eq!((*caller).flags & want, 0, "the caller was redrawn instead");
        assert_eq!((*caller).retval, 0);
    }
}

#[test]
fn status_only_refreshes_the_status_line_and_nothing_else() {
    let _guard = globals();
    let mut clients = Clients::new();
    let caller = clients.add("caller", 80, 24);
    let target = clients.add("target", 80, 24);
    unsafe {
        wire(caller, null_mut(), CLIENT_ATTACHED as uint64_t);
        wire(target, null_mut(), 0);

        let mut item = Item::new().with_args(c"refresh-client -S");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(
            (*target).flags & (CLIENT_STATUSFORCE as uint64_t | CLIENT_REDRAWSTATUS as uint64_t),
            CLIENT_STATUSFORCE as uint64_t | CLIENT_REDRAWSTATUS as uint64_t
        );
        assert_eq!((*target).flags & (CLIENT_REDRAWWINDOW as uint64_t), 0);
    }
}

#[test]
fn clipboard_queries_answer_normal_when_the_terminal_never_started() {
    let _guard = globals();
    let mut clients = Clients::new();
    let caller = clients.add("caller", 80, 24);
    let target = clients.add("target", 80, 24);
    unsafe {
        wire(caller, null_mut(), CLIENT_ATTACHED as uint64_t);
        wire(target, null_mut(), 0);

        let mut item = Item::new().with_args(c"refresh-client -l");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        let touched = CLIENT_ALLREDRAWFLAGS | CLIENT_STATUSFORCE as uint64_t;
        assert_eq!((*target).flags & touched, 0, "-l drew something");
        assert_eq!((*target).tty.flags, 0, "the query reached a dead terminal");
    }
}

#[test]
fn flag_forms_set_flags_through_the_peer_and_still_refresh() {
    let _guard = globals();
    let mut clients = Clients::new();
    let caller = clients.add("caller", 80, 24);
    let target = clients.add("target", 80, 24);
    unsafe {
        wire(caller, null_mut(), CLIENT_ATTACHED as uint64_t);
        wire(target, null_mut(), 0);

        let mut item = Item::new().with_args(c"refresh-client -F read-only");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_ne!((*target).flags & CLIENT_READONLY as uint64_t, 0);

        let mut item = Item::new().with_args(c"refresh-client -f ignore-size");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_ne!((*target).flags & CLIENT_IGNORESIZE as uint64_t, 0);
        assert_eq!(
            (*target).flags & (CLIENT_STATUSFORCE as uint64_t | CLIENT_ALLREDRAWFLAGS),
            CLIENT_STATUSFORCE as uint64_t | CLIENT_ALLREDRAWFLAGS,
            "setting flags did not refresh"
        );
    }
}

#[test]
fn panning_anchors_its_offset_on_the_current_window_then_moves_it() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut s = Session::new(1, "panned");
    let mut w = Window::new(1, "home", 100, 50);
    let wl = link(&mut s, &mut w, 0);
    let caller = clients.add("caller", 80, 24);
    let target = clients.add("target", 80, 24);
    unsafe {
        wire(caller, null_mut(), CLIENT_ATTACHED as uint64_t);
        wire(target, s.ptr(), 0);
        (*target).tty.oox = 7;
        (*target).tty.ooy = 3;
        (*target).tty.osx = 20;
        (*target).tty.osy = 10;

        let mut item = Item::new().with_args(c"refresh-client -R 5");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(
            client_get_pan_window(target),
            w.ptr(),
            "the anchored window was not recorded"
        );
        assert_eq!((*target).pan_ox, 12);
        assert_eq!((*target).pan_oy, 3);

        let mut item = Item::new().with_args(c"refresh-client -L 99");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!((*target).pan_ox, 0, "left did not stop at the edge");

        let mut item = Item::new().with_args(c"refresh-client -U 2");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!((*target).pan_oy, 1, "up did not move by two");

        let mut item = Item::new().with_args(c"refresh-client -c");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert!(
            client_get_pan_window(target).is_null(),
            "-c left the anchor behind"
        );

        unlink(&mut s, wl);
    }
}

#[test]
fn panning_down_and_right_stop_at_the_window_edge() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut s = Session::new(2, "capped");
    let mut w = Window::new(1, "small", 40, 30);
    let wl = link(&mut s, &mut w, 0);
    let caller = clients.add("caller", 80, 24);
    let target = clients.add("target", 80, 24);
    unsafe {
        wire(caller, null_mut(), CLIENT_ATTACHED as uint64_t);
        wire(target, s.ptr(), 0);
        (*target).tty.oox = 30;
        (*target).tty.ooy = 15;
        (*target).tty.osx = 25;
        (*target).tty.osy = 20;

        let mut item = Item::new().with_args(c"refresh-client -D 9999");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!((*target).pan_oy, 10, "down passed the bottom edge");

        let mut item = Item::new().with_args(c"refresh-client -R 50");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!((*target).pan_ox, 15, "right passed the right edge");

        unlink(&mut s, wl);
    }
}

#[test]
fn offsets_turn_a_named_pane_off_and_back_on_for_a_control_client() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut cs = ControlState::new();
    let mut registry = Registry::new();
    let mut pane = Pane::new(3, 80, 24, 100);
    registry.add_pane(&mut pane);
    let caller = clients.add("caller", 80, 24);
    let target = clients.add("target", 80, 24);
    unsafe {
        wire(caller, null_mut(), CLIENT_ATTACHED as uint64_t);
        wire(target, null_mut(), CLIENT_CONTROL as uint64_t);
        cs.attach(target);

        let mut item = Item::new().with_args(c"refresh-client -A '%3:off'");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(collect_panes(cs.ptr()), [(3, CONTROL_PANE_OFF)]);

        let mut item = Item::new().with_args(c"refresh-client -A '%3:on'");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(collect_panes(cs.ptr()), [(3, 0)]);
    }
}

#[test]
fn pausing_a_pane_writes_one_line_and_continue_writes_its_answer() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut cs = ControlState::new();
    let mut registry = Registry::new();
    let mut pane = Pane::new(3, 80, 24, 100);
    registry.add_pane(&mut pane);
    let caller = clients.add("caller", 80, 24);
    let target = clients.add("target", 80, 24);
    unsafe {
        wire(caller, null_mut(), CLIENT_ATTACHED as uint64_t);
        wire(target, null_mut(), CLIENT_CONTROL as uint64_t);
        cs.attach(target);

        let mut item = Item::new().with_args(c"refresh-client -A '%3:pause'");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(collect_panes(cs.ptr()), [(3, CONTROL_PANE_PAUSED)]);
        assert_eq!(cs.written(), b"%pause %3\n".to_vec());

        let mut item = Item::new().with_args(c"refresh-client -A '%3:continue'");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(collect_panes(cs.ptr()), [(3, 0)]);
        assert_eq!(cs.written(), b"%continue %3\n".to_vec());
        assert_eq!(cs.written(), Vec::<u8>::new());
    }
}

#[test]
fn offset_values_that_name_nothing_are_ignored() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut cs = ControlState::new();
    let mut registry = Registry::new();
    let mut pane = Pane::new(3, 80, 24, 100);
    registry.add_pane(&mut pane);
    let caller = clients.add("caller", 80, 24);
    let target = clients.add("target", 80, 24);
    unsafe {
        wire(caller, null_mut(), CLIENT_ATTACHED as uint64_t);
        wire(target, null_mut(), CLIENT_CONTROL as uint64_t);
        cs.attach(target);

        for line in [
            c"refresh-client -A 5:on",
            c"refresh-client -A '%99:on'",
            c"refresh-client -A '%3:bogus'",
        ] {
            let mut item = Item::new().with_args(line);
            aim(&mut item, caller, target);
            assert_eq!(run(&mut item), CMD_RETURN_NORMAL, "{line:?}");
            assert!(collect_panes(cs.ptr()).is_empty(), "{line:?} added a pane");
            assert_eq!(cs.written(), Vec::<u8>::new());
        }
    }
}

#[test]
fn subscriptions_are_keyed_by_name_across_every_kind() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut cs = ControlState::new();
    let caller = clients.add("caller", 80, 24);
    let target = clients.add("target", 80, 24);
    unsafe {
        wire(caller, null_mut(), CLIENT_ATTACHED as uint64_t);
        wire(target, null_mut(), CLIENT_CONTROL as uint64_t);
        cs.attach(target);
        for name in ["all", "pn", "win", "ses", "m1", "m2"] {
            cs.track(name);
        }
        let none = u32::MAX;

        let mut item = Item::new().with_args(c"refresh-client -B all:%*:fmt-all");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(
            collect_subs(cs.ptr()),
            [(
                "all".to_string(),
                CONTROL_SUB_ALL_PANES as c_char,
                none,
                "fmt-all".to_string()
            )]
        );

        let mut item = Item::new().with_args(c"refresh-client -B pn:%4:fmt-pn -B win:@7:fmt-win");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        let mut item = Item::new().with_args(c"refresh-client -B ses:junk:fmt-ses");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(
            collect_subs(cs.ptr()),
            [
                (
                    "all".to_string(),
                    CONTROL_SUB_ALL_PANES as c_char,
                    none,
                    "fmt-all".to_string()
                ),
                (
                    "pn".to_string(),
                    CONTROL_SUB_PANE as c_char,
                    4,
                    "fmt-pn".to_string()
                ),
                (
                    "ses".to_string(),
                    CONTROL_SUB_SESSION as c_char,
                    none,
                    "fmt-ses".to_string()
                ),
                (
                    "win".to_string(),
                    CONTROL_SUB_WINDOW as c_char,
                    7,
                    "fmt-win".to_string()
                ),
            ]
        );

        let mut item = Item::new().with_args(c"refresh-client -B pn:%9:fmt-new");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        let subs = collect_subs(cs.ptr());
        assert_eq!(subs.len(), 4, "{subs:?}");
        assert_eq!(
            subs[1],
            (
                "pn".to_string(),
                CONTROL_SUB_PANE as c_char,
                9,
                "fmt-new".to_string()
            )
        );

        let mut item = Item::new().with_args(c"refresh-client -B pn");
        aim(&mut item, caller, target);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        let subs = collect_subs(cs.ptr());
        assert_eq!(subs.len(), 3, "{subs:?}");

        for line in [c"refresh-client -B ghost", c"refresh-client -B half:x"] {
            let mut item = Item::new().with_args(line);
            aim(&mut item, caller, target);
            assert_eq!(run(&mut item), CMD_RETURN_NORMAL, "{line:?}");
        }
        let subs = collect_subs(cs.ptr());
        assert_eq!(
            subs.into_iter().map(|s| s.0).collect::<Vec<String>>(),
            ["all".to_string(), "ses".to_string(), "win".to_string()]
        );
    }
}

#[test]
fn the_parser_hands_the_hook_exactly_the_letters_in_the_template() {
    let _guard = globals();
    unsafe {
        let mut item = Item::new().with_args(c"refresh-client -lS -r junk-report");
        let args = cmd_get_args(&*item.cmd());
        assert_eq!(args_has(args, b'l'), 1);
        assert_eq!(args_has(args, b'S'), 1);
        assert_eq!(args_has(args, b'R'), 0);
        assert_eq!(seen(args_get(args, b'r')), "junk-report");
        assert_eq!(args_count(args), 0);

        let mut item = Item::new().with_args(c"refresh-client -R 5");
        let args = cmd_get_args(&*item.cmd());
        assert_eq!(args_has(args, b'R'), 1);
        assert_eq!(args_count(args), 1);
        assert_eq!(seen(args_string(args, 0)), "5");
    }
}
