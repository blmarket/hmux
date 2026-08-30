//! Leak regressions against the pinned tmux 3.7b.
//!
//! Seven paths take a formatted string, hand it out as a raw pointer and —
//! unlike the C they were transpiled from — never give it back. Each is
//! driven here from its own entry point, so `make leak-c2rs` names the one
//! that lost the allocation instead of reporting a total at the end of a run.
//!
//! These tests stay in the leak profile's normal set. A failure means that the
//! path left an allocation reachable at process exit; a pass means that the
//! path reclaimed it. The skip list is reserved for leaks that predate the
//! gate.
//!
//! What each of them loses, and what 3.7b does instead:
//!
//! | path | lost | 3.7b |
//! |---|---|---|
//! | `window_copy_cmd_copy_selection_no_clear` | `prefix`, always | `free(prefix)` before returning |
//! | `window_copy_do_copy_end_of_line` | `prefix`, `command`, on the cancel return | frees both there too |
//! | `window_copy_do_copy_line` | `prefix`, `command`, on the cancel return | frees both there too |
//! | `window_copy_cmd_copy_pipe_no_clear` | `prefix` and `command`, always | `free(command)`, `free(prefix)` |
//! | `window_copy_cmd_pipe_no_clear` | `command`, always | `free(command)` |
//! | `menu_add_item` | `s`, on the empty-item return | `free(s)` before that return |
//! | `spawn_pane` | the first `cwd`, when it is rewritten absolute | `free(cwd)` before `cwd = new_cwd` |
//!
//! Every one of them is paired with a control that runs the neighbouring
//! path — the same command with nothing to format, or the branch that does
//! reach the free. The controls are green now and have to stay green: they
//! are what says the failure belongs to the argument rather than to the
//! fixtures around it.
//!
//! The same profile also contains raw-pointer payload checks for dead-client
//! file completions and command-queue group removal. Their controls keep the
//! callback-owned `LoadBuffer`, `SourceFile`, `PaneInput` and `KeyEvent`
//! allocations on a green path.

use crate::arguments::args_count;
use crate::cmd::cmd;
use crate::cmd::cmd_load_buffer::cmd_load_buffer_data;
use crate::cmd::cmd_source_file::cmd_source_file_data;
use crate::cmd::{
    CMD_FIND_PANE, CmdqType, KEYC_NONE, cmdq_append, cmdq_get_callback1, cmdq_new, cmdq_new_state,
    cmdq_next,
};
use crate::file::{CLIENT_DEAD, file_create_with_client, file_fire_done};
use crate::overlay::{menu_add_item, menu_create};
use crate::reactor::{self, Buf, Reactor};
use crate::spawn::{SPAWN_RESPAWN, spawn_pane};
use crate::tests::test_fixtures::{
    Args, Item, Pane, Session, Target, Window, ensure_reactor, globals, link, unlink_all, zeroed,
    zeroed_client, zeroed_cmdq_item,
};
use crate::types::{
    ClientFileData, CmdqCallbackData, WindowMode, args_parse_t, cmd_entry, cmd_entry_flag,
    cmd_retval, key_code, key_event, menu_item, mouse_event, spawn_context, u_int, window_pane,
    winlink,
};
use crate::types::{PaneInputRef, SourceFileRef};
use crate::window::window_pane_current_mode;
use crate::window::window_pane_input_data;
use crate::window::window_pane_set_mode;
use ::core::ffi::CStr;
use ::core::ptr::null_mut;

/// A descriptor number the spawn rig's pane claims to hold. Nothing opens or
/// closes it; it is there so that a respawn sees a pane that is still live.
const FAKE_FD: ::core::ffi::c_int = 10;

unsafe fn free_file_data(
    _c: *mut crate::types::client,
    _path: *const ::core::ffi::c_char,
    _error: ::core::ffi::c_int,
    _closed: ::core::ffi::c_int,
    _buffer: *mut Buf,
    data: ClientFileData,
) {
    drop(data);
}

unsafe fn run_file_completion(data: ClientFileData, dead: bool, stream: ::core::ffi::c_int) {
    unsafe {
        let c = zeroed_client();
        let c_ptr = c.as_ptr();
        if dead {
            (*c_ptr).flags |= CLIENT_DEAD as u64;
        }
        let cf = file_create_with_client(c_ptr, stream, Some(free_file_data), data);
        file_fire_done(cf);
        reactor::current().run_once();
        assert!((*c_ptr).files.is_empty());
    }
}

unsafe fn free_key_event_data(
    _item: *mut crate::types::cmdq_item,
    data: CmdqCallbackData,
) -> cmd_retval {
    let CmdqCallbackData::KeyEvent(event) = data else {
        panic!("callback data is not a key event");
    };
    drop(event);
    0
}

unsafe fn failing_command(_cmd: &cmd, _item: *mut crate::types::cmdq_item) -> cmd_retval {
    crate::cmd::CMD_RETURN_ERROR
}

/// The command entry the rig's item runs. A `'static` one, because a command
/// borrows the entry it was parsed against for as long as it lives.
static LEAK_ENTRY: cmd_entry = cmd_entry {
    name: c"leak-command",
    alias: Some(c""),
    args: args_parse_t {
        template: c"",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"",
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: 0,
    exec: failing_command,
};

struct CmdqLeakRig {
    client: crate::types::ClientRef,
    command: Args,
}

impl CmdqLeakRig {
    fn new(group: u_int) -> CmdqLeakRig {
        let mut client = zeroed_client();
        client.queue = Some(cmdq_new());
        client.environ = Some(Box::new(::std::collections::BTreeMap::new()));
        let command = Args::parse(c"display-message");
        unsafe {
            (*command.cmd()).entry = &LEAK_ENTRY;
            let state = cmdq_new_state(null_mut(), null_mut(), 0);
            let command_item = zeroed_cmdq_item(state);
            command_item.item().name = Some(c"leak-command".to_owned());
            command_item.item().type_0 = CmdqType::Command {
                cmdlist: Some(command.cmdlist()),
                at: 0,
            };
            command_item.item().group = group;
            let event = Box::new(key_event {
                key: b'k' as key_code,
                m: mouse_event::default(),
                buf: b"callback-data".to_vec(),
            });
            let mut callback = cmdq_get_callback1(
                c"free-key-event".as_ptr(),
                Some(free_key_event_data),
                CmdqCallbackData::KeyEvent(event),
            );
            callback[0].item().group = group;
            cmdq_append(client.as_ptr(), ::std::vec![command_item]);
            cmdq_append(client.as_ptr(), callback);
        }
        CmdqLeakRig { client, command }
    }

    unsafe fn run(&self) -> u_int {
        unsafe { cmdq_next(self.client.as_ptr()) }
    }
}

/// A callback in the failed command's group is removed before it fires, so
/// its raw key-event payload is reclaimed with the queue item.
#[test]
fn a_grouped_callback_gives_up_raw_key_event_data() {
    let _g = globals();
    let rig = CmdqLeakRig::new(1);
    unsafe {
        assert_eq!(rig.run(), 1);
    }
}

/// A callback outside the failed command's group remains queued and frees its
/// raw key-event payload when the queue reaches it.
#[test]
fn an_ungrouped_callback_releases_raw_key_event_data() {
    let _g = globals();
    let rig = CmdqLeakRig::new(0);
    unsafe {
        assert_eq!(rig.run(), 2);
    }
}

/// The callback data carried by a file is owned by its completion callback.
/// A live client reaches that callback for every raw-pointer payload.
#[test]
fn a_live_client_releases_load_buffer_file_data() {
    let _g = globals();
    ensure_reactor();
    unsafe {
        run_file_completion(
            ClientFileData::LoadBuffer(zeroed::<cmd_load_buffer_data>()),
            false,
            401,
        );
    }
}

/// A dead client skips the completion callback but still reclaims its
/// load-buffer payload.
#[test]
fn a_dead_client_gives_up_load_buffer_file_data() {
    let _g = globals();
    ensure_reactor();
    unsafe {
        run_file_completion(
            ClientFileData::LoadBuffer(zeroed::<cmd_load_buffer_data>()),
            true,
            402,
        );
    }
}

/// The source-file payload is also reclaimed when its callback runs.
#[test]
fn a_live_client_releases_source_file_data() {
    let _g = globals();
    ensure_reactor();
    unsafe {
        run_file_completion(
            ClientFileData::SourceFile(SourceFileRef::new(cmd_source_file_data {
                item: None,
                flags: 0,
                after: None,
                retval: 0,
                current: 0,
                files: Vec::new(),
            })),
            false,
            403,
        );
    }
}

/// A dead client reclaims the source-file payload before its callback would
/// have run.
#[test]
fn a_dead_client_gives_up_source_file_data() {
    let _g = globals();
    ensure_reactor();
    unsafe {
        run_file_completion(
            ClientFileData::SourceFile(SourceFileRef::new(cmd_source_file_data {
                item: None,
                flags: 0,
                after: None,
                retval: 0,
                current: 0,
                files: Vec::new(),
            })),
            true,
            404,
        );
    }
}

/// The pane-input payload follows the same callback ownership rule.
#[test]
fn a_live_client_releases_pane_input_file_data() {
    let _g = globals();
    ensure_reactor();
    unsafe {
        run_file_completion(
            ClientFileData::PaneInput(PaneInputRef::new(*zeroed::<window_pane_input_data>())),
            false,
            405,
        );
    }
}

/// A dead client reclaims the pane-input payload as well.
#[test]
fn a_dead_client_gives_up_pane_input_file_data() {
    let _g = globals();
    ensure_reactor();
    unsafe {
        run_file_completion(
            ClientFileData::PaneInput(PaneInputRef::new(*zeroed::<window_pane_input_data>())),
            true,
            406,
        );
    }
}

/// Puts the target's first pane into copy mode and runs one `send-keys -X`
/// line through the mode's own command hook, which is the path a key binding
/// takes. Answers nothing: what these tests watch is what the run allocates.
///
/// `values` is how many argument values the line has to carry — the copy-mode
/// command's own name and then the arguments the test is about. It is checked
/// because a line that quietly parses into fewer allocates nothing, which
/// would leave a leak test passing for the wrong reason.
unsafe fn send_keys_x(t: &mut Target, line: &CStr, values: u_int) {
    unsafe {
        let wp = t.pane(0);
        let mut fs = t.state();
        let open = Args::parse(c"copy-mode");
        window_pane_set_mode(wp, wp, WindowMode::Copy, &raw mut fs, open.ptr());
        let wme = window_pane_current_mode(wp);
        assert!(!wme.is_null(), "the pane did not open copy mode");

        let args = Args::parse(line);
        assert_eq!(
            args_count(&*args.ptr()),
            values,
            "{line:?} did not parse into the arguments the test needs"
        );
        assert!(
            WindowMode::Copy.has_command(),
            "copy mode carries a command hook"
        );
        WindowMode::Copy.command(
            wme,
            null_mut::<crate::types::client>(),
            t.session(),
            t.winlink(0),
            args.ptr(),
            null_mut::<crate::types::mouse_event>(),
        );
    }
}

/// A session holding one window whose only pane still has a descriptor, which
/// is what makes a respawn refuse. Modelled on the rig the spawn suite uses;
/// the descriptor is never touched, because every branch reached from here
/// returns before any descriptor work.
struct SpawnRig {
    session: Session,
    wl: *mut winlink,
    pane: *mut window_pane,
    _window: Window,
    _pane: Pane,
}

impl SpawnRig {
    fn new() -> SpawnRig {
        let mut session = Session::new(0, "0");
        let mut window = Window::new(0, "keep", 80, 24);
        let mut pane = Pane::new(1, 80, 24, 100);
        unsafe { (*pane.ptr()).fd = FAKE_FD };
        window.add_pane(&mut pane);
        let wl = link(&mut session, &mut window, 0);
        SpawnRig {
            session,
            wl,
            pane: pane.ptr(),
            _window: window,
            _pane: pane,
        }
    }

    /// Runs a respawn of the still-attached pane asking for `cwd`, and answers
    /// the cause it was refused with.
    unsafe fn refuse(&mut self, cwd: Option<&::core::ffi::CStr>) -> String {
        unsafe {
            let mut item = Item::new().with_args(c"respawn-pane");
            let mut sc = Box::new(spawn_context::default());
            sc.item = crate::cmd::cmdq_item_weak_from_ptr(item.ptr());
            sc.s = self.session.ptr();
            sc.wl = self.wl;
            sc.wp0 = self.pane;
            sc.idx = -1;
            sc.flags = SPAWN_RESPAWN;
            sc.cwd = cwd;

            let mut cause = None;
            let out = spawn_pane(&mut sc, &mut cause);
            assert!(out.is_null(), "the respawn was not refused");
            cause.unwrap().into_string().unwrap()
        }
    }
}

impl Drop for SpawnRig {
    fn drop(&mut self) {
        unlink_all(&mut self.session);
    }
}

/// `copy-selection` formats its buffer-name argument and never frees it.
/// 3.7b's `window_copy_cmd_copy_selection_no_clear` ends with `free(prefix)`.
#[test]
fn copy_selection_gives_up_its_formatted_buffer_name() {
    let _g = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        send_keys_x(&mut t, c"send-keys -X copy-selection leak-prefix", 2);
    }
}

/// The control for the test above: the same command with no argument to
/// format allocates nothing to lose, so copy mode's own setup and teardown
/// are leak-clean and the failure above belongs to the argument.
#[test]
fn copy_selection_without_an_argument_leaves_nothing_behind() {
    let _g = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        send_keys_x(&mut t, c"send-keys -X copy-selection", 1);
    }
}

/// `copy-end-of-line-and-cancel` formats its buffer-name argument and, on the
/// cancel path only, returns without freeing it. 3.7b's
/// `window_copy_do_copy_end_of_line` frees `prefix` and `command` before the
/// cancel return as well as at the tail.
#[test]
fn copy_end_of_line_and_cancel_gives_up_its_formatted_buffer_name() {
    let _g = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        send_keys_x(
            &mut t,
            c"send-keys -X copy-end-of-line-and-cancel leak-prefix",
            2,
        );
    }
}

/// The control for the test above: the same argument down the path that does
/// reach the tail is freed there, so only the cancel return loses it.
#[test]
fn copy_end_of_line_frees_its_formatted_buffer_name() {
    let _g = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        send_keys_x(&mut t, c"send-keys -X copy-end-of-line leak-prefix", 2);
    }
}

/// The same pair of paths in `window_copy_do_copy_line`.
#[test]
fn copy_line_and_cancel_gives_up_its_formatted_buffer_name() {
    let _g = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        send_keys_x(&mut t, c"send-keys -X copy-line-and-cancel leak-prefix", 2);
    }
}

/// The control for the test above.
#[test]
fn copy_line_frees_its_formatted_buffer_name() {
    let _g = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        send_keys_x(&mut t, c"send-keys -X copy-line leak-prefix", 2);
    }
}

/// `copy-pipe` formats both of its arguments and frees neither. 3.7b's
/// `window_copy_cmd_copy_pipe_no_clear` ends with `free(command)` and
/// `free(prefix)`.
///
/// The command is a format that expands to nothing, which is what keeps the
/// pipe from starting a job: the expansion is empty, so `window_copy_pipe_run`
/// falls back to the `copy-command` option, which is empty by default. The
/// allocation the test is about is made before any of that.
#[test]
fn copy_pipe_gives_up_both_of_its_formatted_arguments() {
    let _g = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        send_keys_x(
            &mut t,
            c"send-keys -X copy-pipe '#{no_such_format}' leak-prefix",
            3,
        );
    }
}

/// `pipe` formats its command and never frees it. 3.7b's
/// `window_copy_cmd_pipe_no_clear` ends with `free(command)`.
#[test]
fn pipe_gives_up_its_formatted_command() {
    let _g = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        send_keys_x(&mut t, c"send-keys -X pipe '#{no_such_format}'", 2);
    }
}

/// A menu item whose name formats to nothing is dropped from the menu, and
/// the formatted name is dropped with it. 3.7b's `menu_add_item` frees `s`
/// before that return.
#[test]
fn a_menu_item_that_formats_to_nothing_gives_up_its_formatted_name() {
    let _g = globals();
    unsafe {
        let mut c = zeroed_client();
        let mut menu = menu_create(c"leak".as_ptr());
        let item = menu_item {
            name: Some(c"#{no_such_format}"),
            key: KEYC_NONE as key_code,
            command: None,
        };
        menu_add_item(
            &raw mut *menu,
            Some(&item),
            null_mut::<crate::types::cmdq_item>(),
            &raw mut *c,
            null_mut::<crate::types::cmd_find_state>(),
        );
        assert_eq!(menu.items.len(), 0, "the empty item was not dropped");
    }
}

/// The control for the test above: an item whose name formats to something is
/// kept, and the path that keeps it does free the expansion.
#[test]
fn a_menu_item_that_formats_to_a_name_frees_the_expansion() {
    let _g = globals();
    unsafe {
        let mut c = zeroed_client();
        let mut menu = menu_create(c"leak".as_ptr());
        let item = menu_item {
            name: Some(c"kept"),
            key: KEYC_NONE as key_code,
            command: None,
        };
        menu_add_item(
            &raw mut *menu,
            Some(&item),
            null_mut::<crate::types::cmdq_item>(),
            &raw mut *c,
            null_mut::<crate::types::cmd_find_state>(),
        );
        assert_eq!(menu.items.len(), 1, "the item was not kept");
    }
}

/// A spawn context asking for a relative working directory has it expanded
/// once and then rewritten onto the session's own directory; the first
/// expansion is dropped on the floor. 3.7b's `spawn_pane` frees `cwd` before
/// `cwd = new_cwd`.
///
/// The spawn is a respawn of a pane that is still attached, which is refused
/// before any descriptor work — the working directory is worked out ahead of
/// that refusal, so the allocation the test is about is already made and
/// already lost.
#[test]
fn a_relative_spawn_cwd_gives_up_its_first_expansion() {
    let _g = globals();
    let mut rig = SpawnRig::new();
    unsafe {
        assert_eq!(rig.refuse(Some(c"relative/dir")), "pane 0:0.0 still active");
    }
}

/// The control for the test above: an absolute directory needs no rewrite, so
/// the one expansion it makes is the one the refusal frees.
#[test]
fn an_absolute_spawn_cwd_is_freed_by_the_refusal() {
    let _g = globals();
    let mut rig = SpawnRig::new();
    unsafe {
        assert_eq!(
            rig.refuse(Some(c"/absolute/dir")),
            "pane 0:0.0 still active"
        );
    }
}
