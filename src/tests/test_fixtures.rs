//! Objects the crate's unit tests build on, so that each module's tests do not
//! hand-roll their own scaffolding.
//!
//! [`globals`] initializes the calling thread's server state, including its
//! environment, option trees, parser state and UTF-8 caches. Each test thread
//! owns its state independently, so this fixture never waits for another test.
//!
//! The second is a set of owned builders that free what they made when they go
//! out of scope. [`Grid`], [`Screen`], [`Options`], [`Args`],
//! [`Item`] and [`Format`] wrap the real constructors. [`Session`], [`Window`]
//! and [`Pane`] do not: they are hand-built structs carrying just the
//! invariants a unit test needs, because the real `session_create`/
//! `window_create`/`window_pane_create` paths register with the live server
//! trees, take timestamps and — for a pane — would eventually want a shell.
//! See each type's own note. [`Target`] arranges those into the registered
//! session–winlink–window–pane chain the target-taking code walks, [`Paste`]
//! is a turn at the paste store and [`KeyTable`] is a key table of the test's
//! own.

use crate::args::args_parse_t;
use crate::cmd::cmdq_item;
use crate::cmd::{CmdListRef, cmd, cmd_entry, cmd_entry_flag, cmd_retval};
use crate::options::{OptionsEngine, RustOptionsEngine};
use crate::pane_identity::PaneIdentity;
use crate::window_dimensions::WindowDimensionsState;
use crate::window_name::WindowNameState;

use crate::pane_geometry::PaneGeometryState;

use crate::cmd::CmdqItemRef;

use crate::screen::RustScreen;
use crate::session::session_new_detached;
pub use crate::types::*;
use crate::window::winlinks_into;
use crate::window::{window_active_pane, window_set_active};
use crate::window::{
    window_pane_reset_mode_all, window_panes_insert_tail, window_ref_of, winlink_insert,
};

/// Every call a [`Prompt::Recorder`] prompt made to its input callback, as the
/// answer it carried and whether it was the final one. The list belongs to
/// the calling thread's server state.
const PROMPT_ANSWERS: crate::server_state::LocalField<std::cell::RefCell<Vec<(String, c_int)>>> =
    crate::server_state::LocalField::new(|state| &state.prompt_test_answers);

/// What [`Prompt::Recorder`] answers with. Returning zero is what a one-shot
/// prompt's callback does, and what makes the accepting paths take the prompt
/// back down.
pub unsafe fn prompt_recorder(
    _c: &mut client,
    _data: &mut PromptData,
    s: Option<&CStr>,
    done: c_int,
) -> c_int {
    let answer = match s {
        None => "<none>".to_string(),
        Some(s) => unsafe { seen(s.as_ptr()) },
    };
    PROMPT_ANSWERS.with_borrow_mut(|answers| answers.push((answer, done)));
    0
}

/// The answers recorded so far, oldest first.
pub fn prompt_answers() -> Vec<(String, c_int)> {
    PROMPT_ANSWERS.with_borrow(Clone::clone)
}

/// Forgets every recorded answer, so a test reads only its own prompt's.
pub fn prompt_answers_clear() {
    PROMPT_ANSWERS.with_borrow_mut(Vec::clear);
}

use crate::cmd::CmdqListOps;
use crate::cmd::cmd_find_from_winlink;
use crate::cmd::{CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::cmd::{CmdqStateRef, CmdqType};
use crate::environ::{RustEnvironment, new_environment_box};
use crate::file::CLIENT_DEAD;
use crate::format::{
    FORMAT_NONE, format_create, format_defaults, format_expand, format_expand_time,
};
use crate::grid::{grid_create, grid_default_cell, grid_get_cell, grid_set_cell};
use crate::key_bindings::{
    key_binding_cmdlist_ref, key_binding_note, key_bindings_add, key_bindings_get_table,
    key_bindings_remove,
};
use crate::message_log::{
    MessageLogStore, MessageLogTime, RustMessageLog, with_message_log, with_message_log_mut,
};
use crate::options::{
    OPTIONS_TABLE_PANE, OPTIONS_TABLE_SERVER, OPTIONS_TABLE_SESSION, OPTIONS_TABLE_WINDOW,
};

use crate::paste::{PasteBufferStore, with_paste_buffers, with_paste_buffers_mut};
use crate::reactor;
use crate::reactor::{IoWatch, Reactor, Timer};
use crate::status::status_free;
use crate::terminfo::tty_term_of;
use crate::terminfo::{RustTerminalCapabilities, TerminalCapabilities};
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr::null_mut;
use ::std::ffi::CString;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LoggedMessage {
    pub text: CString,
    pub number: u32,
    pub time: MessageLogTime,
}

pub(crate) fn message_log_entries() -> Vec<LoggedMessage> {
    with_message_log(|log| {
        log.entries()
            .map(|entry| LoggedMessage {
                text: entry.text.to_owned(),
                number: entry.number,
                time: entry.time,
            })
            .collect()
    })
}

pub(crate) fn logged_messages() -> Vec<String> {
    let mut messages = message_log_entries();
    messages.reverse();
    messages
        .into_iter()
        .map(|entry| String::from_utf8_lossy(entry.text.as_bytes()).into_owned())
        .collect()
}

pub(crate) fn message_log_count() -> usize {
    with_message_log(|log| log.entries().count())
}

pub(crate) fn reset_message_log() {
    with_message_log_mut(|log| *log = RustMessageLog::new());
}

/// The globals `main` sets up that the modules' tests need — the environment,
/// the three option trees and the socket path the format engine reports.
///
/// Each test thread initializes its own process state independently.
pub(crate) fn globals_ready() {
    if crate::tmux::global_options.get().is_some() {
        return;
    }
    unsafe {
        crate::tmux::global_options_create();
        defaults(
            crate::tmux::global_options.get().as_ref().unwrap(),
            OPTIONS_TABLE_SERVER,
        );
        defaults(
            crate::tmux::global_s_options.get().as_ref().unwrap(),
            OPTIONS_TABLE_SESSION,
        );
        defaults(
            crate::tmux::global_w_options.get().as_ref().unwrap(),
            OPTIONS_TABLE_WINDOW,
        );
        crate::tmux::socket_path.set(Some(c"/tmp/tmux-fixture/default".to_owned()));
    }
}

/// Restores the calling thread's previous process handle on scope exit.
pub(crate) struct GlobalsGuard {
    previous: Option<crate::proc::ProcessRef>,
}
impl Drop for GlobalsGuard {
    fn drop(&mut self) {
        crate::server::server_process.set(self.previous.take());
    }
}

/// Initializes this test thread's options and process handle.
pub(crate) fn globals() -> GlobalsGuard {
    globals_ready();
    let previous = crate::server::server_process.replace(Some(crate::proc::ProcessRef::default()));
    GlobalsGuard { previous }
}

#[test]
fn fixture_process_is_restored_after_nested_scopes() {
    let outer = globals();
    let process = crate::server::server_process.get().unwrap();
    let inner = globals();
    assert!(!std::rc::Rc::ptr_eq(
        &process,
        &crate::server::server_process.get().unwrap()
    ));
    drop(inner);
    assert!(std::rc::Rc::ptr_eq(
        &process,
        &crate::server::server_process.get().unwrap()
    ));
    drop(outer);
    assert!(crate::server::server_process.get().is_none());
}

/// Takes every event a client can arm back off the event loop.
pub(crate) unsafe fn quiesce_client(c: *mut client) {
    unsafe {
        (*c).event.disable();
        (*c).repeat_timer.disarm();
        (*c).click_timer.disarm();
        (*c).message_timer.disarm();
        (*c).overlay_timer.disarm();
        (*c).status.timer.disarm();
        (*c).tty.start_timer.disarm();
        (*c).tty.clipboard_timer.disarm();
    }
}

/// Everything a fixture client has to give back when it goes away: the events
/// it may have armed, the status line a test asked `status_init` for, the
/// files `cmdq_error` opened on it to carry a message, and the exit strings
/// `server_client_detach` left on it. A fixture client is ordinary memory and
/// never reaches `server_client_lost`, which is where the server does all of
/// this.
///
/// Outstanding files finish before the fixture owner is released, because
/// their callbacks may still hold typed client handles. Deferred completion
/// first returns callback-owned data, then the fixture marks the client dead
/// and drains any retry work before releasing its owner.
pub(crate) unsafe fn release_client(c: *mut client) {
    unsafe {
        quiesce_client(c);
        (*c).tty.flags &= !crate::tty::TTY_OPENED;
        if !(*c).files.is_empty() {
            reactor::current().run_once();
            for cf in (*c).files.values().cloned().collect::<Vec<_>>() {
                cf.borrow_mut().error = libc::EINTR;
                cf.fire_done();
            }
            reactor::current().run_once();
            (*c).flags |= CLIENT_DEAD as uint64_t;
            reactor::current().run_once();
            while !(*c).files.is_empty() {
                for cf in (*c).files.values().cloned().collect::<Vec<_>>() {
                    cf.borrow_mut().error = libc::EINTR;
                    cf.fire_done();
                }
                reactor::current().run_once();
            }
        }
        status_free(&mut *c);
        (*c).status.screen = RustScreen::default();
        (*c).exit_session = None;
        (*c).exit_message = None;
    }
}

#[cfg(test)]
mod client_handle_tests {
    use super::{globals, zeroed_client};

    #[test]
    fn an_explicit_handle_view_can_be_used_without_an_owning_pointer() {
        let _guard = globals();
        let client = zeroed_client();
        let weak = client.downgrade();
        let pointer = unsafe { client.as_client() as *const _ };
        assert!(core::ptr::eq(pointer, client.as_ptr()));
        drop(client);
        assert!(weak.upgrade().is_none());
    }
}

/// Gives `oo` the default value of every option in `scope`.
unsafe fn defaults(oo: &RustOptionsRef, scope: c_int) {
    for oe in RustOptionsEngine.table() {
        if oe.scope & scope != 0 {
            unsafe { oo.set_default(oe) };
        }
    }
}

/// How many terminal capabilities `tty_term` keeps a slot for.
///
/// A zeroed value of a `#[repr(C)]` struct, the way `xcalloc` hands one out.
/// Only for a type every one of whose fields zero is a valid value for — a
/// `Vec` is not one, since a null buffer pointer is not a value it may even
/// hold. A type that has a `Default` says so itself, and is built with that
/// instead; this is for the C structs that have none.
pub(crate) fn zeroed<T>() -> Box<T> {
    Box::new(unsafe { core::mem::zeroed() })
}

/// The entry a fixture command carries until one is parsed for it. A command
/// borrows its entry for as long as it lives, so there is no null to start
/// from; running this one is a fixture that was never given a command.
static PLACEHOLDER_ENTRY: cmd_entry = cmd_entry {
    name: c"",
    alias: None,
    args: args_parse_t {
        template: c"",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"",
    source: cmd_entry_flag {
        flag: 0,
        type_0: crate::cmd::CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: 0,
        type_0: crate::cmd::CMD_FIND_PANE,
        flags: 0,
    },
    flags: 0,
    exec: placeholder_exec,
};

fn placeholder_exec(_cmd: &cmd, _item: &cmdq_item) -> cmd_retval {
    panic!("the fixture command was never given an entry")
}

/// A command with no arguments, waiting for a parsed one to fill it in.
pub(crate) fn empty_cmd() -> Box<cmd> {
    Box::new(cmd {
        entry: &PLACEHOLDER_ENTRY,
        args: None,
        group: 0,
        file: None,
        line: 0,
        parse_flags: 0,
    })
}

pub(crate) fn zeroed_cmdq_item(state: CmdqStateRef) -> CmdqItemRef {
    CmdqItemRef::from_type(
        CmdqType::Command {
            cmdlist: None,
            at: 0,
        },
        state,
    )
}

/// An empty valid screen ready to initialize.
pub(crate) fn zeroed_screen() -> Box<RustScreen> {
    Box::new(RustScreen::default())
}

/// A client the way `server_client_create` hands one out, near enough: zeroed,
/// with the five status lines' range lists and the prompt buffer as real empty
/// `Vec`s, as required for a live client.
pub(crate) fn zeroed_client() -> ClientRef {
    ClientRef::new(client::default())
}

/// The same for a pane, whose border status line carries one such list and
/// whose visible ranges are a real empty `Vec`.
pub(crate) fn zeroed_window() -> Box<window> {
    Box::new(window::default())
}

pub(crate) fn zeroed_pane() -> Box<window_pane> {
    let mut pane = Box::new(window_pane::default());
    // A fixture owns no descriptors. Zero would make window teardown close
    // stdin, or a different test's descriptor after that number is reused.
    *pane.fd_mut() = -1;
    *pane.pipe_fd_mut() = -1;
    pane
}

/// A terminal description the way `tty_term_create` leaves one, near enough:
/// zeroed, but with a full-length code table of missing entries.
pub(crate) fn zeroed_term() -> crate::terminfo::TerminalRef {
    std::rc::Rc::new(std::cell::RefCell::new(RustTerminalCapabilities::new(c"")))
}

/// A terminal the way `tty_init` leaves one, near enough: zeroed, but with the
/// visible range list made into a real empty `Vec` first.
pub(crate) fn zeroed_tty() -> Box<tty> {
    Box::new(tty::default())
}

/// A grid, destroyed at the end of the test.
pub(crate) struct Grid(Box<grid>);

impl Grid {
    pub(crate) fn new(sx: u_int, sy: u_int, hlimit: u_int) -> Grid {
        Grid(grid_create(sx, sy, hlimit))
    }

    /// Writes `s` from (px, py), one ASCII cell per byte.
    pub(crate) fn write(&mut self, px: u_int, py: u_int, s: &str) {
        for (i, byte) in s.bytes().enumerate() {
            let gc = ascii(byte);
            grid_set_cell(&mut *self, px + i as u_int, py, &gc);
        }
    }

    pub(crate) fn cell(&self, px: u_int, py: u_int) -> grid_cell {
        grid_get_cell(&*self, px, py)
    }
}

impl core::ops::Deref for Grid {
    type Target = grid;

    fn deref(&self) -> &grid {
        &self.0
    }
}

impl core::ops::DerefMut for Grid {
    fn deref_mut(&mut self) -> &mut grid {
        &mut self.0
    }
}

/// One cell holding the ASCII byte `ch` in the default style.
pub(crate) fn ascii(ch: u8) -> grid_cell {
    let mut gc = { grid_default_cell };
    gc.data.data[0] = ch;
    gc.data.have = 1;
    gc.data.size = 1;
    gc.data.width = 1;
    gc
}

/// A screen owned by the test fixture.
pub(crate) struct Screen(Box<RustScreen>);

impl Screen {
    pub(crate) fn new(sx: u_int, sy: u_int, hlimit: u_int) -> Screen {
        Screen(Box::new(RustScreen::new_with_server_options(
            sx, sy, hlimit,
        )))
    }

    pub(crate) fn ptr(&mut self) -> *mut RustScreen {
        &raw mut *self.0
    }
}

impl core::ops::Deref for Screen {
    type Target = RustScreen;

    fn deref(&self) -> &RustScreen {
        &self.0
    }
}

impl core::ops::DerefMut for Screen {
    fn deref_mut(&mut self) -> &mut RustScreen {
        &mut self.0
    }
}

/// An option set, freed at the end of the test.
pub(crate) struct Options(RustOptionsRef);

impl Options {
    /// An empty set below `parent`.
    pub(crate) fn empty(parent: Option<&RustOptionsRef>) -> Options {
        Options(RustOptionsEngine.create(parent))
    }

    /// A set holding the default value of every option in `scope`, one of the
    /// `OPTIONS_TABLE_*` bits.
    pub(crate) fn defaults(scope: c_int) -> Options {
        let oo = Options::empty(None);
        unsafe { defaults(&oo, scope) };
        oo
    }

    /// A session option set, as `session_create` would be given.
    pub(crate) fn session() -> Options {
        Options::defaults(OPTIONS_TABLE_SESSION)
    }

    /// A window option set, as `window_create` would make.
    pub(crate) fn window() -> Options {
        Options::defaults(OPTIONS_TABLE_WINDOW)
    }

    /// A pane option set, as `window_pane_create` would make.
    pub(crate) fn pane() -> Options {
        Options::defaults(OPTIONS_TABLE_PANE)
    }

    /// Hands shared ownership of the option set to a caller, such as a session.
    pub(crate) fn owned(self) -> crate::options::RustOptionsRef {
        self.0
    }
}

impl core::ops::Deref for Options {
    type Target = RustOptionsRef;

    fn deref(&self) -> &RustOptionsRef {
        &self.0
    }
}

/// The parsed form of a command line, freed at the end of the test. The
/// command list owns the arguments, so it is kept alive alongside them.
pub(crate) struct Args {
    cmdlist: CmdListRef,
}

impl Args {
    /// Parses `s` the way the command parser would, and keeps its first
    /// command. Panics if `s` is not a command line.
    pub(crate) fn parse(s: &CStr) -> Args {
        unsafe {
            let mut pr = cmd_parse_from_string(s, None);
            assert_eq!(pr.status, CMD_PARSE_SUCCESS, "{s:?} did not parse");
            let cmdlist = pr.cmdlist.take().unwrap();
            Args { cmdlist }
        }
    }

    /// Borrows the parsed arguments through their command-list owner.
    pub(crate) fn borrow(&self) -> std::cell::Ref<'_, crate::RustArguments> {
        std::cell::Ref::map(
            self.cmdlist.command(0).expect("the parsed command"),
            |command| command.args.as_deref().expect("the parsed arguments"),
        )
    }

    /// The list the parsed command sits in, which is what a queue item names
    /// it through.
    pub(crate) fn cmdlist(&self) -> CmdListRef {
        self.cmdlist.clone()
    }

    /// Borrows the first parsed command through its list owner.
    pub(crate) fn command(&self) -> std::cell::Ref<'_, cmd> {
        self.cmdlist.command(0).expect("the parsed command")
    }

    /// Borrows the first parsed command exclusively through its list owner.
    pub(crate) fn command_mut(&self) -> std::cell::RefMut<'_, cmd> {
        self.cmdlist.command_mut(0).expect("the parsed command")
    }

    pub(crate) fn list_ref(&self) -> &CmdListRef {
        &self.cmdlist
    }
}

/// Retains a command, its queue item, and the state used by fixture operations.
pub(crate) struct Item {
    item: CmdqItemRef,
    queued: bool,
    cmdlist: CmdListRef,
    client: ClientRef,
    state: CmdqStateRef,
    args: Option<Args>,
}

impl Item {
    /// An item with no client behind it.
    pub(crate) fn new() -> Item {
        let cmdlist = CmdListRef::empty();
        cmdlist.append(empty_cmd());
        let state = unsafe { CmdqStateRef::create(None, None, 0) };
        let it = Item {
            item: zeroed_cmdq_item(state.clone()),
            queued: false,
            cmdlist: cmdlist.clone(),
            client: zeroed_client(),
            state,
            args: None,
        };
        it.item.item().type_0 = CmdqType::Command {
            cmdlist: Some(cmdlist),
            at: 0,
        };
        it
    }

    /// An item a client is waiting on, as the command queue would hand it to a
    /// command.
    ///
    pub(crate) fn with_client() -> Item {
        let it = Item::new();
        it.item.item().client = Some(it.client.clone());
        it
    }

    pub(crate) fn set_client(&mut self, c: *mut client) {
        self.item.item().client = unsafe { crate::server::client_ref_of(&*c) };
    }

    /// Where the command came from, which is what `cmdq_error` reports.
    pub(crate) fn with_file(self, file: &'static CStr, line: u_int) -> Item {
        {
            let mut command = self.cmdlist.command_mut(0).expect("the fixture command");
            command.file = Some(file.to_owned());
            command.line = line;
        }
        self
    }

    /// Runs the command line `s` through the parser and points the item's
    /// command at the arguments it produced.
    pub(crate) fn with_args(mut self, s: &CStr) -> Item {
        let args = Args::parse(s);
        {
            let mut source = args.cmdlist.command_mut(0).expect("the parsed command");
            let mut target = self.cmdlist.command_mut(0).expect("the fixture command");
            target.entry = source.entry;
            target.args = source.args.take();
        }
        self.args = Some(args);
        self
    }

    /// Points the item's target, source and current states at `target`'s
    /// winlink, the way the command queue prepares an item before running its
    /// command. The item's own client, if it has one, becomes the target
    /// client too.
    pub(crate) fn targeting(self, target: &mut Target) -> Item {
        let fs = target.state();
        {
            let mut item = self.item.item();
            item.target = fs.clone();
            item.source = fs.clone();
            if let Some(client) = item.client.as_ref().map(ClientRef::downgrade) {
                item.target_client = Some(client);
            }
        }
        self.state.state().current = fs;
        self
    }

    pub(crate) fn ptr(&mut self) -> *mut cmdq_item {
        self.item.as_ptr()
    }

    pub(crate) fn handle(&self) -> CmdqItemRef {
        self.item.clone()
    }

    pub(crate) fn item_mut(&self) -> std::cell::RefMut<'_, cmdq_item> {
        self.item.item()
    }

    pub(crate) fn read(&self) -> std::cell::Ref<'_, cmdq_item> {
        self.item.read()
    }

    /// Shares the item with the queue while retaining the fixture's own handle.
    pub(crate) fn queue_onto(&mut self, queue: &CmdqListRef) {
        if !self.queued {
            self.item.item().queue = Some(queue.downgrade());
            queue.append_item(self.item.clone());
            self.queued = true;
        }
    }

    pub(crate) fn command(&self) -> std::cell::Ref<'_, cmd> {
        self.cmdlist.command(0).expect("the fixture command")
    }

    pub(crate) fn with_command<R>(&self, operation: impl FnOnce(&cmd, &cmdq_item) -> R) -> R {
        let command = self.cmdlist.command(0).expect("the fixture command");
        let item = self.handle();
        operation(&command, &item.read())
    }

    /// The command arguments under a shared borrow of their list.
    pub(crate) fn args(&self) -> std::cell::Ref<'_, crate::RustArguments> {
        std::cell::Ref::map(
            self.cmdlist.command(0).expect("the fixture command"),
            |command| command.args.as_deref().expect("the fixture arguments"),
        )
    }

    pub(crate) fn client(&mut self) -> *mut client {
        unsafe { self.client.as_client_mut() }
    }

    /// The state the item shares with the rest of its queue, as a handle of
    /// the caller's own.
    pub(crate) fn state_ref(&self) -> CmdqStateRef {
        self.item.state_ref()
    }

    pub(crate) fn flags(&self) -> c_int {
        self.item.read().flags
    }

    pub(crate) fn set_flags(&mut self, flags: c_int) {
        self.item.item().flags = flags;
    }
}

impl Drop for Item {
    fn drop(&mut self) {
        unsafe { release_client(self.client.as_client_mut()) };
    }
}

/// A session that is **not** registered with the server's session tree, has no
/// group, no client attached and no timers running. It carries an id, a name,
/// an environment, an option set and the empty window collections, which is
/// what a unit test reaching for `*mut session` needs. Anything that walks the
/// live `sessions` tree, spawns a process or arms the lock timer wants a real
/// server, not this.
pub(crate) struct Session {
    session: SessionRef,
}

impl Session {
    pub(crate) fn new(id: u_int, name: &str) -> Session {
        let name = CString::new(name).expect("a session name has no NUL");
        let session = session_new_detached(
            id,
            name,
            CString::new("/").expect("no NUL"),
            Options::session().owned(),
            new_environment_box(),
        );
        let s = Session { session };
        unsafe { (*s.session.as_ptr()).lastw.clear() };
        s
    }

    pub(crate) fn ptr(&mut self) -> *mut session {
        self.session.as_ptr()
    }

    pub(crate) fn weak(&self) -> SessionWeak {
        self.session.downgrade()
    }

    pub(crate) fn reference(&self) -> SessionRef {
        self.session.clone()
    }

    /// The handle the fixture holds, for a callee that takes one rather than a
    /// pointer.
    pub(crate) fn handle(&self) -> &SessionRef {
        &self.session
    }

    /// Borrows the session environment for reading.
    ///
    /// # Safety
    /// No other session owner may mutate the environment during this borrow.
    pub(crate) unsafe fn environ(&self) -> &RustEnvironment {
        unsafe { self.session.as_session().environ_ref() }
    }

    /// Borrows the session environment for mutation.
    ///
    /// # Safety
    /// Other session owners must not access the environment during this borrow.
    pub(crate) unsafe fn environ_mut(&mut self) -> &mut RustEnvironment {
        unsafe { self.session.as_session_mut().environ_mut() }
    }

    pub(crate) fn options(&self) -> RustOptionsRef {
        unsafe { self.session.as_session().options_ref().clone() }
    }
}

/// A window that is **not** registered with the server's window tree and has
/// no timers running. It carries an id, a name, an option set and the empty
/// pane, winlink and stack collections.
pub(crate) struct Window {
    window: WindowRef,
}

impl Window {
    pub(crate) fn new(id: u_int, name: &str, sx: u_int, sy: u_int) -> Window {
        let mut value = zeroed_window();
        value.id = id;
        let name = CString::new(name).expect("a window name has no NUL");
        value.set_window_name(Some(&name));
        value.options = Some(Options::window().owned());
        value.set_size(crate::pane_resize::PaneSize {
            width: sx,
            height: sy,
        });
        value.set_manual_size(crate::pane_resize::PaneSize {
            width: sx,
            height: sy,
        });
        value.winlinks = window_winlinks::new();
        let window = WindowRef::new(*value);
        Window { window }
    }

    pub(crate) fn ptr(&mut self) -> *mut window {
        self.window.as_ptr()
    }

    pub(crate) fn weak(&self) -> WindowWeak {
        self.window.downgrade()
    }

    pub(crate) fn reference(&self) -> WindowRef {
        self.window.clone()
    }

    /// The handle the fixture holds, for a callee that takes one rather than a
    /// pointer.
    pub(crate) fn handle(&self) -> &WindowRef {
        &self.window
    }

    pub(crate) fn options(&self) -> RustOptionsRef {
        self.window.options()
    }

    /// Puts `pane` at the end of the window's pane list and makes it active if
    /// there is none yet, the way `window_add_pane` would.
    ///
    /// The window takes the pane over, as a real one does: `pane` keeps its
    /// pointer, so a test still reads it through [`Pane::ptr`], but the window
    /// is what gives it back. A pane handed over twice, or handed to a window
    /// that has already gone, is a test that has lost track of its own
    /// fixtures.
    pub(crate) fn add_pane(&mut self, pane: &mut Pane) {
        {
            let owned = RustWindowPaneRef::new(pane.take());
            pane.observer = Some(owned.downgrade());
            let id = owned.pane_id();
            let mut w = self.window.as_window_mut();
            window_panes_insert_tail(&mut w, owned);
            w.z_index
                .push(crate::window::window_pane_find_by_id(id).unwrap());
            if crate::window::window_active_pane(&w).is_none() {
                w.active_pane = w
                    .panes
                    .iter()
                    .find(|pane| pane.pane_id() == id)
                    .map(|pane| pane.downgrade());
            }
        }
    }
}

/// Gives back what [`Pane::new`] made: the two screens, the timers and the
/// option set. A fixture pane has no process behind it, so this is the whole
/// of its teardown before its owning registration is dropped.
fn free_pane(pane: &mut impl crate::WindowPane) {
    {
        pane.resize_timer_mut().disarm();
        pane.sync_timer_mut().disarm();
        if let Some(oo) = pane.options_mut().take() {
            RustOptionsEngine.destroy(oo);
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            let w = self.window.as_ptr();
            for mut pane in core::mem::take(&mut (*w).panes) {
                free_pane(pane.as_pane_mut());
                crate::window::window_pane_set_window_ref(pane.as_pane_mut(), None);
                drop(pane.into_pane());
            }
            (*w).z_index.clear();
            (*w).last_panes.clear();
            window_set_active(&mut *w, None::<&crate::types::window_pane>);
        }
    }
}

/// A pane that is **not** in the server's `all_window_panes` tree, has no
/// process behind it (`fd` and `pipe_fd` stay -1, `argv` and `shell` stay
/// null), no timers armed and no input parser. Its base screen is
/// real and owned by the pane, and
/// `screen` points at that base, which is what the drawing and copy-mode code
/// reads. Nothing here spawns a shell; a test that wants one wants the
/// conformance suite.
///
/// The pane owns itself until a [`Window`] takes it over, which is what
/// [`Window::add_pane`] does; after that the window is what gives it back and
/// this is only the pointer to it.
pub(crate) struct Pane {
    pane: Option<Box<window_pane>>,
    observer: Option<RustWindowPaneWeak>,
    ptr: *mut window_pane,
}

impl Pane {
    pub(crate) fn new(id: u_int, sx: u_int, sy: u_int, hlimit: u_int) -> Pane {
        let mut pane = zeroed_pane();
        pane.set_pane_id(id);
        crate::window::window_pane_reserve_id(id);
        *pane.options_mut() = Some(Options::pane().owned());
        pane.set_size(crate::pane_resize::PaneSize {
            width: sx,
            height: sy,
        });
        *pane.fd_mut() = -1;
        *pane.pipe_fd_mut() = -1;
        {
            *pane.base_mut() = RustScreen::new_with_server_options(sx, sy, hlimit);
            *pane.status_screen_mut() = RustScreen::new_with_server_options(1, 1, 0);
        }
        *pane.shown_mut() = PaneScreen::Base;
        let ptr = &raw mut *pane;
        Pane {
            pane: Some(pane),
            observer: None,
            ptr,
        }
    }

    pub(crate) fn ptr(&mut self) -> *mut window_pane {
        self.observer
            .as_ref()
            .map_or(self.ptr, |pane| pane.as_mut_ptr())
    }

    /// Gives the pane itself up, for a window to take over. What is left
    /// behind still answers [`Pane::ptr`].
    fn take(&mut self) -> Box<window_pane> {
        self.pane
            .take()
            .expect("the pane has not been handed to a window yet")
    }

    /// Puts the pane at the end of `w`'s pane list and on its stacking order,
    /// the way [`Window::add_pane`] does, for a test holding the window as a
    /// bare pointer. `w` takes the pane over and must outlive it.
    pub(crate) fn hand_to(&mut self, w: *mut window) -> *mut window_pane {
        unsafe {
            let wp = self.ptr();
            let owned = RustWindowPaneRef::new(self.take());
            self.observer = Some(owned.downgrade());
            window_panes_insert_tail(&mut *w, owned);
            crate::window::window_pane_set_window(&mut *wp, w.as_ref());
            (*w).z_index
                .push(crate::window::window_pane_find_by_id((*wp).pane_id()).unwrap());
            wp
        }
    }

    pub(crate) fn base(&self) -> &RustScreen {
        unsafe { (*self.ptr).base() }
    }

    pub(crate) fn base_mut(&mut self) -> &mut RustScreen {
        unsafe { (*self.ptr).base_mut() }
    }

    pub(crate) fn options(&self) -> RustOptionsRef {
        unsafe { (*self.ptr).options_ref().clone() }
    }
}

impl Drop for Pane {
    fn drop(&mut self) {
        if let Some(mut pane) = self.pane.take() {
            free_pane(&mut *pane);
        }
    }
}

/// A window carrying a real layout tree and the panes that hang off it. The
/// window and its panes are the server-free [`Window`] and [`Pane`] above; the
/// tree is the real one `layout_init` builds, and it is freed before the panes
/// go. Nothing here is in the server's trees, so a layout can be arranged,
/// resized and dumped without a server.
pub(crate) struct Layout {
    window: Window,
    panes: Vec<Pane>,
    next_id: u_int,
}

impl Layout {
    pub(crate) fn reference(&self) -> WindowRef {
        self.window.reference()
    }

    pub(crate) fn cell(&mut self, pane: usize) -> Option<std::cell::Ref<'_, layout_cell>> {
        let id = unsafe { (*self.pane(pane)).pane_id() };
        let w = { self.window.handle().as_window() };
        std::cell::Ref::filter_map(w, |w| {
            crate::layout::layout_cell_for_pane(
                w.layout_root.as_deref(),
                &crate::window::window_pane_find_by_id(id).expect("the pane allocation exists"),
            )
            .map(|(cell, _)| cell)
        })
        .ok()
    }

    /// A window of `sx` by `sy` with one pane filling it, as `layout_init`
    /// leaves a freshly created window.
    pub(crate) fn new(sx: u_int, sy: u_int) -> Layout {
        let mut l = Layout {
            window: Window::new(1, "layout", sx, sy),
            panes: Vec::new(),
            next_id: 0,
        };
        l.add_pane(sx, sy);
        unsafe {
            (l.reference()).init_layout(
                &crate::window::window_pane_find_by_id((*l.pane(0)).pane_id())
                    .expect("the layout pane exists"),
            )
        };
        l
    }

    pub(crate) fn w(&mut self) -> *mut window {
        self.window.ptr()
    }

    pub(crate) fn window(&mut self) -> &mut Window {
        &mut self.window
    }

    pub(crate) fn pane(&mut self, i: usize) -> *mut window_pane {
        self.panes[i].ptr()
    }

    pub(crate) fn count(&self) -> usize {
        self.panes.len()
    }

    /// A pane at the end of the window's pane list, not yet in the layout
    /// tree. Answers its index.
    pub(crate) fn add_pane(&mut self, sx: u_int, sy: u_int) -> usize {
        self.next_id += 1;
        let mut pane = Pane::new(self.next_id, sx, sy, 100);
        self.window.add_pane(&mut pane);
        self.panes.push(pane);
        self.panes.len() - 1
    }

    /// The tree as one line: each node is its type, size and offset, with its
    /// children in brackets.
    pub(crate) fn dump(&mut self) -> String {
        unsafe { dump_cell((*self.w()).layout_root.as_deref()) }
    }

    /// The sizes and offsets the panes themselves were given.
    pub(crate) fn panes(&self) -> Vec<String> {
        unsafe {
            self.window
                .handle()
                .as_window()
                .panes
                .iter()
                .map(|pane| {
                    let pane = pane.as_pane();
                    let geometry = pane.geometry();
                    format!(
                        "%{} {}x{}+{}+{}",
                        pane.pane_id(),
                        geometry.width,
                        geometry.height,
                        geometry.x,
                        geometry.y
                    )
                })
                .collect()
        }
    }
}

impl Drop for Layout {
    fn drop(&mut self) {
        (self.window.reference()).free_layout();
    }
}

/// One cell of a layout tree as a string, with its children in brackets. A
/// floating cell is marked with a star.
pub(crate) fn dump_cell(lc: Option<&layout_cell>) -> String {
    use crate::layout::{
        LAYOUT_CELL_FLOATING, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE,
    };
    let Some(lc) = lc else {
        return "-".to_string();
    };
    let here = format!("{}x{}+{}+{}", lc.sx, lc.sy, lc.xoff, lc.yoff);
    let floating = if lc.flags & LAYOUT_CELL_FLOATING != 0 {
        "*"
    } else {
        ""
    };
    match lc.type_0 {
        LAYOUT_WINDOWPANE => format!(
            "%{}{floating} {here}",
            lc.wp_ref
                .as_ref()
                .map(|pane| pane.id())
                .unwrap_or(u_int::MAX)
        ),
        LAYOUT_LEFTRIGHT | LAYOUT_TOPBOTTOM => {
            let kids: Vec<String> = lc
                .cells
                .iter()
                .map(|child| dump_cell(Some(child)))
                .collect();
            let name = if lc.type_0 == LAYOUT_LEFTRIGHT {
                "LR"
            } else {
                "TB"
            };
            format!("{name}{floating} {here} [{}]", kids.join(" | "))
        }
        _ => format!("?{floating} {here}"),
    }
}

/// A terminal that is **not** attached to anything: a zeroed `tty` pointing at
/// an empty terminal capability description and the zeroed `client` behind it.
/// No terminfo entry is read, no descriptor is open and no timer is armed; a
/// test that wants a real terminal wants the
/// conformance suite.
///
/// The client behind it is a [`ClientRef`], which registers itself in the
/// process-wide handle tree, so a test that builds one holds [`globals`].
pub(crate) struct Tty {
    tty: Box<tty>,
    client: ClientRef,
}

impl Tty {
    pub(crate) fn new() -> Tty {
        let mut t = Tty {
            tty: zeroed_tty(),
            client: zeroed_client(),
        };
        let term = zeroed_term();
        t.tty.term = Some(term);
        t.tty.owner = Some(t.client.downgrade());
        t
    }

    pub(crate) fn ptr(&mut self) -> *mut tty {
        &raw mut *self.tty
    }

    pub(crate) fn term(&self) -> std::cell::Ref<'_, tty_term> {
        tty_term_of(&self.tty)
    }

    pub(crate) fn term_mut(&mut self) -> std::cell::RefMut<'_, tty_term> {
        self.tty
            .term
            .as_ref()
            .expect("the fixture built a term")
            .borrow_mut()
    }

    /// Gives `code` a number, as a terminfo entry carrying that capability
    /// would.
    pub(crate) fn set_number(&mut self, code: tty_code_code, number: c_int) {
        let name = self.term().capability_name(code).to_owned();
        let capability = CString::new(format!(
            "{}={number}",
            String::from_utf8_lossy(name.to_bytes())
        ))
        .expect("a terminal capability override has no NUL");
        self.term_mut().apply_overrides(&capability);
    }

    /// Gives `code` a string, as a terminfo entry carrying that capability
    /// would.
    pub(crate) fn set_string(&mut self, code: tty_code_code, s: &CStr) {
        let mut capability = self.term().capability_name(code).to_bytes().to_vec();
        capability.push(b'=');
        for &byte in s.to_bytes() {
            capability.push(byte);
            if byte == b':' {
                capability.push(byte);
            }
        }
        self.term_mut().apply_overrides(
            &CString::new(capability).expect("a terminal capability override has no NUL"),
        );
    }

    /// Gives `code` a flag, as a boolean terminfo capability would.
    pub(crate) fn set_flag(&mut self, code: tty_code_code, flag: c_int) {
        assert_ne!(flag, 0, "the public override syntax only adds true flags");
        let capability = self.term().capability_name(code).to_owned();
        self.term_mut().apply_overrides(&capability);
    }

    /// Takes `code` back out of the terminal's table, leaving it missing.
    pub(crate) fn clear_code(&mut self, code: tty_code_code) {
        let mut capability = self.term().capability_name(code).to_bytes().to_vec();
        capability.push(b'@');
        self.term_mut().apply_overrides(
            &CString::new(capability).expect("a terminal capability override has no NUL"),
        );
    }

    /// The one-byte ACS translation the terminal reports for `ch`, as the
    /// `acsc` capability would fill in. An empty string means it has none.
    pub(crate) fn set_acs(&mut self, ch: u8, to: &str) {
        let bytes = to.as_bytes();
        assert!(bytes.len() < 2, "an ACS translation is a single byte");
        let mut capability = b"acsc=".to_vec();
        capability.push(ch);
        capability.extend(bytes);
        self.term_mut().apply_overrides(
            &CString::new(capability).expect("an ACS capability override has no NUL"),
        );
        self.term_mut().refresh_derived();
    }

    pub(crate) fn set_client_flags(&mut self, flags: u64) {
        *unsafe { self.client.flags_mut() } = flags;
    }
}

/// Initializes the process-local reactor used by tests.
pub(crate) fn ensure_reactor() {
    reactor::current();
}

/// A stream over one end of a socket pair used by tests that need buffered
/// output without a peer.
pub(crate) struct StreamBuffer {
    bev: Stream,
    fds: [c_int; 2],
    seen: std::cell::Cell<usize>,
}

impl StreamBuffer {
    pub(crate) fn new() -> StreamBuffer {
        ensure_reactor();
        let mut fds = [-1 as c_int; 2];
        unsafe {
            assert_eq!(
                libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()),
                0,
                "no socket pair"
            );
            let bev = Stream::new(fds[0], None, None, None);
            assert!(!bev.is_none(), "no buffer event");
            StreamBuffer {
                bev,
                fds,
                seen: std::cell::Cell::new(0),
            }
        }
    }

    pub(crate) fn ptr(&self) -> Stream {
        self.bev
    }

    /// What has been written to it since the last time this was asked.
    pub(crate) fn written(&self) -> Vec<u8> {
        let len = self.bev.output_len();
        let seen = self.seen.replace(len);
        if len <= seen {
            return Vec::new();
        }
        self.bev
            .with_output(|out| out.as_slice()[seen..len].to_vec())
            .unwrap_or_default()
    }
}

impl Drop for StreamBuffer {
    fn drop(&mut self) {
        unsafe {
            self.bev.free();
            libc::close(self.fds[0]);
            libc::close(self.fds[1]);
        }
    }
}

/// The server's global client list, holding clients that are **not** connected
/// to anything: each is a zeroed `client` carrying a name, a terminal size and
/// a reference, with no descriptor open, no session attached until a test gives
/// it one and an empty per-window size tree. The list itself is a global, so a
/// test that builds one takes [`globals`] too; it starts empty and is emptied
/// again at the end of the test.
pub(crate) struct Clients {
    clients: Vec<ClientRef>,
}

impl Clients {
    pub(crate) fn new() -> Clients {
        {
            assert!(
                crate::server::with_clients(|clients| clients.is_empty()),
                "the client list is not empty"
            );
        }
        Clients {
            clients: Vec::new(),
        }
    }

    /// Adds a client at the end of the list, its terminal reporting `sx` by
    /// `sy` and no pixel size.
    pub(crate) fn add(&mut self, name: &str, sx: u_int, sy: u_int) -> *mut client {
        let mut c = zeroed_client();
        (unsafe { c.as_client_mut() }).name =
            Some(CString::new(name).expect("a client name has no NUL"));
        unsafe { c.as_tty_mut() }.sx = sx;
        unsafe { c.as_tty_mut() }.sy = sy;
        let p: *mut client = unsafe { c.as_client_mut() };
        {
            crate::server::with_clients_mut(|clients| clients.push(c.clone()));
        }
        self.clients.push(c);
        p
    }
}

impl Drop for Clients {
    fn drop(&mut self) {
        unsafe {
            crate::server::with_clients_mut(|clients| clients.clear());
            for c in &mut self.clients {
                release_client(c.as_client_mut());
            }
        }
    }
}

/// The server's session and window indexes, holding fixture objects for the
/// length of a test. Panes register through the same owning [`RustWindowPane`]
/// as production panes and are not managed here. A test that builds these
/// indexes takes [`globals`] too, and [`Registry::new`] asserts they start
/// empty.
pub(crate) struct Registry;

impl Registry {
    pub(crate) fn new() -> Registry {
        assert!(
            crate::session::sessions_empty(),
            "the session tree is not empty"
        );
        assert!(
            crate::window::window_ids().is_empty(),
            "the window tree is not empty"
        );
        assert!(
            crate::window::pane_walk().next().is_none(),
            "the pane tree is not empty"
        );
        Registry
    }

    /// Puts `s` in the session tree, which is keyed by name.
    pub(crate) fn add_session(&mut self, s: &mut Session) {
        crate::session::session_registry_insert(&s.session);
    }

    /// Puts `w` in the window tree, which is keyed by id.
    pub(crate) fn add_window(&mut self, w: &mut Window) {
        w.window.register_id();
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        crate::session::session_registry_clear();
        crate::window::window_registry_clear();
    }
}

/// Links `window` into `session` at index `idx` through the real winlink tree.
/// Returns the winlink, which the session's tree owns until [`unlink`] takes it
/// away.
pub(crate) fn link(session: &mut Session, window: &mut Window, idx: c_int) -> *mut winlink {
    unsafe {
        let mut owner = session.reference();
        let observer = owner.downgrade();
        let link = winlink_insert(&mut owner.as_session_mut().windows, idx)
            .expect("the fixture index is available");
        link.session_ref = Some(observer);
        link.set_window(window.window.clone());
        let index = link.idx;
        if owner.curw().is_none() {
            owner.as_session_mut().curw_idx = Some(index);
        }
        let link = owner
            .as_session_mut()
            .windows
            .get_mut(&index)
            .expect("the fixture link remains registered");
        &raw mut **link
    }
}

/// Takes `wl` back out of its session, freeing it.
pub(crate) fn unlink(session: &mut Session, wl: *mut winlink) {
    unsafe {
        if session.handle().curw().is_some_and(|current| {
            current
                .get()
                .is_some_and(|current| core::ptr::eq(current, wl))
        }) {
            session.handle().set_curw(null_mut::<winlink>().as_ref());
        }
        crate::window::winlink_remove(&mut (*session.ptr()).windows, (*wl).idx);
    }
}

/// Gives back every winlink still in `session`'s tree, and empties the last-
/// window stack that pointed into it. A test that linked a window and left it
/// there loses the winlink, and so does one whose command renumbered or
/// relinked the session behind it: the winlinks the test held pointers to are
/// gone and fresh ones stand in their place, so sweeping the tree is the only
/// way to give back what is actually in it. The windows must still be alive,
/// which is what `winlink_remove` walks to drop its reference.
pub(crate) fn unlink_all(session: &mut Session) {
    unsafe {
        let mut owner = session.reference();
        let session = owner.as_session_mut();
        session.curw_idx = None;
        while let Some(index) = session.windows.keys().next().copied() {
            crate::window::winlink_remove(&mut session.windows, index);
        }
        session.lastw.clear();
    }
}

/// A session holding linked windows with panes, everything registered in the
/// server's global trees the way `cmd_find`, the `format_defaults` family and
/// the target-taking commands expect to walk them. It starts as session `$0`
/// named "0" whose window `@0` at index 0 holds the active pane `%0`, and
/// [`Target::add_window`] links further windows behind it. Everything in it is
/// the server-free [`Session`], [`Window`] and [`Pane`] above, so it takes
/// [`globals`] like they do and there is at most one at a time, which
/// [`Registry`] asserts.
pub(crate) struct Target {
    registry: Registry,
    session: Session,
    windows: Vec<Window>,
    panes: Vec<Pane>,
    winlinks: Vec<*mut winlink>,
}

impl Target {
    pub(crate) fn new(sx: u_int, sy: u_int) -> Target {
        let mut t = Target {
            registry: Registry::new(),
            session: Session::new(0, "0"),
            windows: Vec::new(),
            panes: Vec::new(),
            winlinks: Vec::new(),
        };
        t.registry.add_session(&mut t.session);
        t.add_window(0, sx, sy);
        t
    }

    /// Links a fresh window holding one pane at index `idx`, and answers its
    /// position among the target's windows.
    pub(crate) fn add_window(&mut self, idx: c_int, sx: u_int, sy: u_int) -> usize {
        let id = self.windows.len() as u_int;
        let mut w = Window::new(id, "target", sx, sy);
        let mut p = Pane::new(self.panes.len() as u_int, sx, sy, 100);
        w.add_pane(&mut p);
        self.registry.add_window(&mut w);
        let wl = link(&mut self.session, &mut w, idx);
        self.windows.push(w);
        self.panes.push(p);
        self.winlinks.push(wl);
        self.windows.len() - 1
    }

    pub(crate) fn session(&mut self) -> *mut session {
        self.session.ptr()
    }

    /// The handle on the target's session, for a callee that takes one.
    pub(crate) fn session_handle(&self) -> &SessionRef {
        self.session.handle()
    }

    pub(crate) fn winlink(&mut self, i: usize) -> *mut winlink {
        self.winlinks[i]
    }

    pub(crate) fn window(&mut self, i: usize) -> *mut window {
        self.windows[i].ptr()
    }

    pub(crate) fn pane(&mut self, i: usize) -> *mut window_pane {
        self.panes[i].ptr()
    }

    /// The find state a resolved target comes as: the session's current
    /// winlink, its window and that window's active pane.
    pub(crate) fn state(&mut self) -> cmd_find_state {
        let mut fs = *Box::new(cmd_find_state::default());
        unsafe {
            cmd_find_from_winlink(
                &mut fs,
                self.session
                    .handle()
                    .curw()
                    .expect("the target has a current window")
                    .get()
                    .expect("the current link is live"),
                0,
            )
        };
        fs
    }
}

impl Drop for Target {
    fn drop(&mut self) {
        for p in &mut self.panes {
            unsafe {
                if let Some(pane) = p.ptr().as_mut() {
                    window_pane_reset_mode_all(pane);
                }
            }
        }
        self.winlinks.clear();
        unlink_all(&mut self.session);
    }
}

/// A format tree, dropped at the end of the test. Filling one reads the paste
/// store and expanding one reads the global options, so a test that builds one
/// holds [`globals`].
pub(crate) struct Format(Box<format_tree>);

impl Format {
    /// An empty tree with no client or item behind it, as `format_create`
    /// leaves one.
    pub(crate) fn new() -> Format {
        Format(format_create(None, None, FORMAT_NONE, 0))
    }

    /// A tree carrying the defaults of `target`'s current winlink, with no
    /// client.
    pub(crate) fn from_target(target: &mut Target) -> Format {
        let fs = target.state();
        let session = fs.session();
        let link = fs.winlink_ref();
        let pane = fs.pane_list_ref();
        let mut format = Format::new();
        unsafe {
            format_defaults(
                format.tree(),
                None,
                session.as_ref().map(|session| session.as_session()),
                link.as_ref().and_then(|link| link.get()),
                pane.as_ref().and_then(|pane| pane.get()),
            );
        }
        format
    }

    pub(crate) fn ptr(&self) -> *mut format_tree {
        &raw const *self.0 as *mut format_tree
    }

    /// The tree itself, to hand to the calls that borrow it.
    pub(crate) fn tree(&mut self) -> &mut format_tree {
        &mut self.0
    }

    /// What `fmt` expands to.
    pub(crate) fn expand(&self, fmt: &CStr) -> String {
        unsafe {
            String::from_utf8_lossy(format_expand(&mut *self.ptr(), fmt).as_bytes()).into_owned()
        }
    }

    /// What `fmt` expands to when `strftime` conversions are honoured too.
    pub(crate) fn expand_time(&self, fmt: &CStr) -> String {
        unsafe {
            String::from_utf8_lossy(format_expand_time(&mut *self.ptr(), fmt).as_bytes())
                .into_owned()
        }
    }
}

/// A key table holding bindings a test gives it, taken away again at the end.
/// Every binding goes in through `key_bindings_add`, which takes over a
/// freshly parsed command list, and comes out through `key_bindings_remove`,
/// which drops the table itself once its last binding has gone. The tables
/// hang off a global tree, so a test that builds one holds [`globals`]. The
/// *default* tables stay out of reach: `key_bindings_init` only queues its
/// bind commands on the global command queue, and no unit test ever runs that
/// queue — every notification the tests raise sits on it pointing at
/// long-gone fixtures.
pub(crate) struct KeyTable {
    name: CString,
    keys: Vec<key_code>,
}

impl KeyTable {
    pub(crate) fn new(name: &str) -> KeyTable {
        KeyTable {
            name: CString::new(name).expect("a table name has no NUL"),
            keys: Vec::new(),
        }
    }

    /// Binds `key` to the command line `s`, with a note when one is given.
    pub(crate) fn bind(&mut self, key: key_code, s: &CStr, note: Option<&CStr>) {
        unsafe {
            let mut pr = cmd_parse_from_string(s, None);
            assert_eq!(pr.status, CMD_PARSE_SUCCESS, "{s:?} did not parse");
            key_bindings_add(&self.name, key, note, 0, pr.cmdlist.take());
        }
        self.keys.push(key);
    }

    /// The table itself, which exists once something is bound in it.
    pub(crate) fn handle(&self) -> KeyTableRef {
        key_bindings_get_table(&self.name, 0).unwrap()
    }
}

impl Drop for KeyTable {
    fn drop(&mut self) {
        {
            for key in &self.keys {
                key_bindings_remove(&self.name, *key);
            }
        }
    }
}

/// A turn at the paste store, emptied of buffers when taken and again when
/// given back. The store is a global, so a test that takes one holds
/// [`globals`]. The store's name and order counters are `paste`'s own and stay
/// where they are, so buffers are added here under explicit names; a test that
/// cares how automatic names are numbered belongs in `paste`'s own suite,
/// whose guard resets the counters too.
pub(crate) struct Paste(());

impl Paste {
    pub(crate) fn new() -> Paste {
        Paste::empty();
        Paste(())
    }

    /// A buffer named `name` holding `data`, owned by the store.
    pub(crate) fn add(&self, name: &CStr, data: &str) -> CString {
        assert!(
            with_paste_buffers_mut(|buffers| { buffers.set_named(name, data.as_bytes().to_vec()) })
                .is_ok(),
            "buffer {name:?} was not set"
        );
        name.to_owned()
    }

    fn empty() {
        let names = with_paste_buffers(|buffers| {
            buffers
                .buffers()
                .map(|buffer| buffer.name.to_owned())
                .collect::<Vec<_>>()
        });
        for name in names {
            with_paste_buffers_mut(|buffers| buffers.remove(name.as_c_str()));
        }
    }
}

impl Drop for Paste {
    fn drop(&mut self) {
        Paste::empty();
    }
}

/// The contents of a C string somebody else still owns.
pub(crate) unsafe fn seen(p: *const c_char) -> String {
    unsafe {
        seen_str(if p.is_null() {
            None
        } else {
            Some(CStr::from_ptr(p))
        })
    }
}

/// The contents of a borrowed string somebody else still owns.
pub(crate) fn seen_str(value: Option<&CStr>) -> String {
    value
        .expect("the string is missing")
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_borrows_check_cloned_owners_and_release_missing_slots() {
        let _guard = globals();
        let list = CmdListRef::empty();
        list.append(empty_cmd());
        assert!(list.command(1).is_none());
        let other = list.clone();
        let shared = list.command(0).unwrap();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { other.command_mut(0) }))
                .is_err()
        );
        drop(shared);
        other.command_mut(0).unwrap().line = 42;
        assert_eq!(list.command(0).unwrap().line, 42);
        assert!(other.command_mut(1).is_none());
        assert!(list.command(0).is_some());
    }

    #[test]
    fn a_fixture_retains_its_item_after_the_queue_releases_it() {
        let _guard = globals();
        let mut fixture = Item::new().with_file(c"queued.conf", 23);
        let weak = fixture.handle().downgrade();
        let queue = CmdqListRef::empty();
        fixture.queue_onto(&queue);
        drop(queue);
        assert!(weak.upgrade().is_some());
        fixture.with_command(|command, item| {
            assert_eq!(command.file.as_deref(), Some(c"queued.conf"));
            assert!(item.queue.as_ref().unwrap().upgrade().is_none());
        });
        drop(fixture);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn a_retained_command_survives_replacing_its_queue_item() {
        let _guard = globals();
        let fixture = Item::new().with_file(c"retained.conf", 17);
        let mut item = fixture.item_mut();
        let (list, at) = item.command_location().unwrap();
        let list = list.clone();
        let command = list.command(at).unwrap();
        item.type_0 = CmdqType::Callback { callback: None };
        assert!(item.command_location().is_none());
        assert!(item.command().is_none());
        drop(item);
        drop(fixture);
        assert_eq!(command.file.as_deref(), Some(c"retained.conf"));
        assert_eq!(command.line, 17);
    }
    use crate::environ::EnvironmentStore;
    use crate::grid::grid_string_cells;

    use crate::window::winlink_count;

    #[test]
    fn a_buffer_event_keeps_what_is_written_to_it() {
        let _guard = globals();
        let bev = StreamBuffer::new();
        bev.ptr().write(b"hi");
        assert_eq!(bev.written(), b"hi");
        assert_eq!(bev.written(), b"");
    }

    #[test]
    fn the_option_sets_carry_the_defaults_of_their_scope() {
        let _guard = globals();
        let session = Options::session();
        let window = Options::window();
        let pane = Options::pane();
        {
            assert_eq!(
                session
                    .string_ref(c"word-separators")
                    .to_string_lossy()
                    .into_owned(),
                "!\"#$%&'()*+,-./:;<=>?@[\\]^`{|}~"
            );
            assert!(session.with_entry(c"window-status-format", false, |entry| entry.is_none()));
            assert_eq!(
                window
                    .string_ref(c"window-status-format")
                    .to_string_lossy()
                    .into_owned(),
                "#I:#W#{?window_flags,#{window_flags}, }"
            );
            assert!(!pane.with_entry(c"pane-border-format", false, |entry| entry.is_none()));
            assert!(pane.with_entry(c"word-separators", false, |entry| entry.is_none()));
            let child = Options::empty(Some(&session));
            assert!(child.with_entry(c"word-separators", true, |entry| entry.is_none()));
            assert_eq!(
                child
                    .string_ref(c"word-separators")
                    .to_string_lossy()
                    .into_owned(),
                "!\"#$%&'()*+,-./:;<=>?@[\\]^`{|}~"
            );
        }
    }

    #[test]
    fn a_grid_takes_text_and_reads_it_back() {
        let _guard = globals();
        let mut grid = Grid::new(10, 5, 100);
        grid.write(0, 0, "abc");
        assert_eq!(grid.sx, 10);
        assert_eq!(grid.cell(1, 0).data.data[0], b'b');
        {
            let p = grid_string_cells(&grid, 0, 0, 10, None, 0, None);
            assert_eq!(p.to_string_lossy(), "abc");
        }
    }

    #[test]
    fn a_screen_comes_with_a_grid_of_its_own() {
        let _guard = globals();
        let mut s = Screen::new(10, 5, 100);
        assert!(!s.ptr().is_null());
        {
            assert_eq!((*s.grid()).sx, 10);
            assert_eq!((*s.grid()).sy, 5);
        }
    }

    #[test]
    fn an_environment_holds_what_is_put_in_it() {
        let _guard = globals();
        let mut env = new_environment_box();
        env.set(c"FOO", 0, c"bar");
        assert_eq!(env.find(c"FOO").and_then(|entry| entry.value), Some(c"bar"));
    }

    #[test]
    fn a_parsed_command_line_hands_over_its_arguments() {
        let _guard = globals();
        let args = Args::parse(c"wait-for -S chan");
        assert_eq!(args.borrow().argument_flag_count(b'S'), 1);
        assert_eq!(seen_str(args.borrow().argument_string(0)), "chan");
        assert!(args.list_ref().command(0).is_some());
    }

    #[test]
    fn an_item_carries_a_client_a_command_and_its_arguments() {
        let _guard = globals();
        let mut plain = Item::new();
        assert!(unsafe { (*plain.ptr()).client() }.is_none());
        let handle = plain.handle();
        assert!(core::ptr::eq(
            &*handle.item().command().unwrap(),
            &*plain.command()
        ));

        let mut item = Item::with_client()
            .with_file(c"fixture.conf", 7)
            .with_args(c"display-message hello");
        item.set_flags(3);
        assert_eq!(item.flags(), 3);
        unsafe {
            assert!(
                (*item.ptr())
                    .client()
                    .is_some_and(|client| client.ptr_eq(&item.client))
            );
            assert_eq!(
                seen_str(item.cmdlist.command(0).unwrap().file.as_deref()),
                "fixture.conf"
            );
            assert_eq!((*item.command()).line, 7);
            assert_eq!(seen_str(item.args().argument_string(0)), "hello");
        }
    }

    #[test]
    fn a_session_carries_an_id_a_name_and_trees_of_its_own() {
        let _guard = globals();
        let mut s = Session::new(4, "fixture");
        unsafe {
            assert_eq!(s.handle().id(), 4);
            assert_eq!(s.handle().name().as_deref(), Some(c"fixture"));
            assert!((*s.ptr()).windows.is_empty());
            assert!((*s.ptr()).lastw.is_empty());
            assert!(s.handle().options().ptr_eq(&s.options()));
            assert!(core::ptr::eq(s.handle().clone().environ(), s.environ()));
        }
    }

    #[test]
    fn a_window_holds_panes_and_a_pane_holds_a_screen() {
        let _guard = globals();
        let mut w = Window::new(2, "fixture", 80, 24);
        let mut first = Pane::new(1, 80, 24, 100);
        let mut second = Pane::new(2, 80, 24, 100);
        w.add_pane(&mut first);
        w.add_pane(&mut second);
        unsafe {
            assert!(window_active_pane(&*w.ptr()).is_some_and(|pane| {
                pane.get()
                    .is_some_and(|active| core::ptr::addr_eq(active, first.ptr()))
            }));
            let payload = w.handle().as_window();
            let mut panes = payload.panes.iter();
            assert!(core::ptr::addr_eq(
                panes.next().unwrap().get().unwrap(),
                first.ptr()
            ));
            assert!(core::ptr::addr_eq(
                panes.next().unwrap().get().unwrap(),
                second.ptr()
            ));
            assert!(panes.next().is_none());
            drop(payload);
            assert!((*first.ptr()).window_context().unwrap().ptr_eq(w.handle()));
            assert!(core::ptr::eq(&*(*first.ptr()).screen_ref(), first.base()));
            assert!(first.base().is_initialized());
            assert_eq!(*(*first.ptr()).fd(), -1);
            assert!((*first.ptr()).options_ref().ptr_eq(&first.options()));
            assert_eq!(
                seen(
                    (*w.ptr())
                        .window_name()
                        .expect("a window has a name")
                        .as_ptr()
                ),
                "fixture"
            );
            assert!((*w.ptr()).options_ref().ptr_eq(&w.options()));
        }
    }

    #[test]
    fn a_layout_starts_as_one_pane_filling_the_window() {
        let _guard = globals();
        let mut l = Layout::new(80, 24);
        assert_eq!(l.dump(), "%1 80x24+0+0");
        assert_eq!(l.panes(), ["%1 80x24+0+0"]);
        assert_eq!(l.count(), 1);
        l.add_pane(80, 24);
        assert_eq!(l.count(), 2);
        assert_eq!(unsafe { (*l.pane(1)).pane_id() }, 2);
        assert_eq!(unsafe { (*l.w()).dimensions().size.width }, 80);
        assert!(
            l.window()
                .options()
                .ptr_eq(unsafe { (*l.w()).options_ref() })
        );
    }

    #[test]
    fn a_target_is_a_registered_session_window_and_pane() {
        let _guard = globals();
        let mut t = Target::new(80, 24);
        unsafe {
            assert!(SessionRef::find(c"0").is_some_and(|found| found.as_ptr() == t.session()));
            assert!(
                WindowRef::find_by_id(0)
                    .is_some_and(|owner| core::ptr::eq(owner.as_ptr(), t.window(0)))
            );
            assert_eq!(
                crate::window::window_pane_find_by_id(0)
                    .unwrap()
                    .as_mut_ptr(),
                t.pane(0)
            );
            assert!(core::ptr::eq((&*t.session()).curw().unwrap(), t.winlink(0)));
            let fs = t.state();
            assert_eq!(
                fs.session().as_ref().map_or(null_mut(), |s| s.as_ptr()),
                t.session()
            );
            assert!(core::ptr::eq(
                fs.winlink_ref().unwrap().get().unwrap(),
                t.winlink(0)
            ));
            assert_eq!(
                fs.window().as_ref().map_or(null_mut(), |w| w.as_ptr()),
                t.window(0)
            );
            assert!(core::ptr::addr_eq(
                fs.pane_list_ref().unwrap().get().unwrap(),
                t.pane(0)
            ));
            let i = t.add_window(5, 80, 24);
            assert_eq!((*t.winlink(i)).idx, 5);
            assert!(
                t.session.handle().as_session().windows[&5]
                    .window_handle()
                    .unwrap()
                    .ptr_eq(t.windows[i].handle())
            );
            assert_eq!(winlink_count(&(*t.session()).windows), 2);
        }
    }

    #[test]
    fn an_item_targeting_a_target_carries_its_find_states() {
        let _guard = globals();
        let mut t = Target::new(80, 24);
        let (s, wl, wp) = (t.session(), t.winlink(0), t.pane(0));
        let mut item = Item::with_client().targeting(&mut t);
        unsafe {
            let target = (*item.ptr()).target();
            assert_eq!((*target).session().as_ref().map(|s| s.as_ptr()), Some(s));
            assert!(core::ptr::eq(
                (*target).winlink_ref().unwrap().get().unwrap(),
                wl
            ));
            assert!(core::ptr::addr_eq(
                (*target).pane_list_ref().unwrap().get().unwrap(),
                wp
            ));
            let current = (*item.ptr()).current();
            assert_eq!((*current).session().as_ref().map(|s| s.as_ptr()), Some(s));
            assert_eq!(
                (*item.ptr())
                    .target_client()
                    .as_ref()
                    .map(ClientRef::as_ptr),
                Some(item.client())
            );
        }
    }

    #[test]
    fn a_format_tree_expands_the_defaults_of_its_target() {
        let _guard = globals();
        let plain = Format::new();
        assert_eq!(plain.expand(c"#{session_name}"), "");
        assert_eq!(plain.expand(c"literal"), "literal");
        let mut t = Target::new(80, 24);
        let ft = Format::from_target(&mut t);
        assert_eq!(ft.expand(c"#{session_name}"), "0");
        assert_eq!(ft.expand(c"#{?window_width,yes,no}"), "yes");
        assert_eq!(ft.expand(c"#{window_width}x#{window_height}"), "80x24");
        assert_eq!(ft.expand(c"#{pane_id}"), "%0");
        assert_eq!(ft.expand_time(c"%%"), "%");
    }

    #[test]
    fn a_key_table_holds_its_bindings_and_goes_away_again() {
        let _guard = globals();
        {
            let mut table = KeyTable::new("fixture-keys");
            table.bind(
                b'x' as key_code,
                c"display-message hello",
                Some(c"a fixture binding"),
            );
            let kt = table.handle();
            {
                let bd = crate::key_bindings::key_bindings_get(&kt.borrow(), b'x' as key_code);
                assert!(bd.is_some(), "no binding for x");
                assert_eq!(
                    key_binding_note(bd.as_ref().unwrap()),
                    Some(c"a fixture binding")
                );
                assert!(key_binding_cmdlist_ref(bd.as_ref().unwrap()).is_some());
            }
        }
        assert!(
            { key_bindings_get_table(c"fixture-keys", 0) }.is_none(),
            "the table is still there"
        );
    }

    #[test]
    fn a_paste_turn_starts_empty_and_holds_named_buffers() {
        let _guard = globals();
        {
            let store = Paste::new();
            assert!(with_paste_buffers(PasteBufferStore::is_empty));
            store.add(c"fixture", "hello");
            assert_eq!(
                with_paste_buffers(|buffers| {
                    buffers.get(c"fixture").map(|buffer| buffer.data.to_vec())
                }),
                Some(b"hello".to_vec())
            );
        }
        assert!(with_paste_buffers(PasteBufferStore::is_empty));
    }

    #[test]
    fn window_link_snapshot_checks_removed_links_and_preserves_order() {
        let _guard = globals();
        let mut session = Session::new(1, "snapshot");
        let mut window = Window::new(1, "win", 80, 24);
        let first = link(&mut session, &mut window, 4);
        let second = link(&mut session, &mut window, 2);
        let owner = window.reference();
        let mut snapshot = { owner.winlinks() };

        unlink(&mut session, first);
        let removed = snapshot.next().expect("the snapshot keeps its first entry");
        assert_eq!(removed.index(), 4);
        assert!(removed.get().is_none());
        let remaining = snapshot.next().expect("the second link remains");
        assert_eq!(remaining.get().map(|link| link.idx), Some(2));
        assert!(snapshot.next().is_none());
        unlink(&mut session, second);
        assert!(remaining.get().is_none());
    }

    #[test]
    fn a_winlink_joins_a_session_and_a_window() {
        let _guard = globals();
        let mut s = Session::new(1, "linked");
        let mut w = Window::new(1, "win", 80, 24);
        let wl = link(&mut s, &mut w, 0);
        unsafe {
            assert_eq!((*wl).idx, 0);
            assert_eq!((*wl).session().map_or(null_mut(), |s| s.as_ptr()), s.ptr());
            assert!(
                s.handle().as_session().windows[&0]
                    .window_handle()
                    .unwrap()
                    .ptr_eq(w.handle())
            );
            assert!(core::ptr::eq((&*s.ptr()).curw().unwrap(), wl));
            assert_eq!(winlink_count(&(*s.ptr()).windows), 1);
            assert!(
                (*s.ptr())
                    .windows
                    .get(&0)
                    .map(Box::as_ref)
                    .is_some_and(|link| core::ptr::eq(link, wl))
            );
            assert!(
                winlinks_into(&*w.ptr())
                    .next()
                    .is_some_and(|held| held.get().is_some_and(|link| core::ptr::eq(link, wl)))
            );
            assert!((*wl).window_ref.is_some());
        }
        unlink(&mut s, wl);
        unsafe {
            assert_eq!(winlink_count(&(*s.ptr()).windows), 0);
            assert!(s.handle().curw().is_none());
            assert!(window_ref_of(&*w.ptr()).is_some());
        }
    }
}

/// Gives a test pane an owned layout cell without changing its geometry or z-order.
pub(crate) fn set_pane_floating(w: &mut window, pane_id: u_int, floating: bool) {
    use crate::layout::{
        LAYOUT_CELL_FLOATING, LAYOUT_LEFTRIGHT, LayoutCellPath, layout_create_cell,
    };

    let path = w.layout_root.as_deref().and_then(|root| {
        LayoutCellPath::for_pane(
            root,
            &crate::window::window_pane_find_by_id(pane_id).expect("the pane allocation exists"),
        )
    });
    if path.is_none() {
        let geometry = unsafe {
            w.panes
                .iter()
                .find(|pane| pane.pane_id() == pane_id)
                .unwrap()
                .as_pane()
        }
        .geometry();
        let mut cell = layout_create_cell(None);
        cell.wp_ref = w
            .panes
            .iter()
            .find(|pane| pane.pane_id() == pane_id)
            .map(|pane| pane.downgrade());
        (cell.sx, cell.sy, cell.xoff, cell.yoff) =
            (geometry.width, geometry.height, geometry.x, geometry.y);
        if let Some(root) = w.layout_root.as_deref() {
            if root.wp_ref.as_ref().map(|pane| pane.id()).is_some() {
                let mut parent = layout_create_cell(None);
                parent.type_0 = LAYOUT_LEFTRIGHT;
                let size = w.dimensions().size;
                (parent.sx, parent.sy, parent.xoff, parent.yoff) = (size.width, size.height, 0, 0);
                let mut only = w.layout_root.replace(parent).unwrap();
                only.has_parent = true;
                w.layout_root.as_deref_mut().unwrap().cells.push(only);
            }
            cell.has_parent = true;
            w.layout_root.as_deref_mut().unwrap().cells.push(cell);
        } else {
            w.layout_root = Some(cell);
        }
    }
    let root = w.layout_root.as_deref_mut().unwrap();
    let path = LayoutCellPath::for_pane(
        root,
        &crate::window::window_pane_find_by_id(pane_id).expect("the pane allocation exists"),
    )
    .unwrap();
    let cell = path.get_mut(root).unwrap();
    if floating {
        cell.flags |= LAYOUT_CELL_FLOATING;
    } else {
        cell.flags &= !LAYOUT_CELL_FLOATING;
    }
}
