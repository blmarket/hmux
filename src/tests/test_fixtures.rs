//! Objects the crate's unit tests build on, so that each module's tests do not
//! hand-roll their own scaffolding.
//!
//! Two kinds of thing live here. The first is [`globals`], a turn at the
//! process-wide state the server keeps in statics — the environment and the
//! three option trees `main` fills, the command parser's own globals, and the
//! UTF-8 trees and width cache. Cargo runs the tests on parallel threads, so
//! every test that reaches any of it holds that guard.
//!
//! The second is a set of owned builders that free what they made when they go
//! out of scope. [`Grid`], [`Screen`], [`Options`], [`Environ`], [`Args`],
//! [`Item`] and [`Format`] wrap the real constructors. [`Session`], [`Window`]
//! and [`Pane`] do not: they are hand-built structs carrying just the
//! invariants a unit test needs, because the real `session_create`/
//! `window_create`/`window_pane_create` paths register with the live server
//! trees, take timestamps and — for a pane — would eventually want a shell.
//! See each type's own note. [`Target`] arranges those into the registered
//! session–winlink–window–pane chain the target-taking code walks, [`Paste`]
//! is a turn at the paste store and [`KeyTable`] is a key table of the test's
//! own.

use crate::cmd::{CmdqItemRef, CmdqItemWeak};
use crate::options::{options_get_only_ptr, options_get_ptr};
use crate::session::{
    session_environ, session_get_curw, session_id, session_name, session_new_detached,
    session_options, session_set_curw,
};
pub use crate::types::*;
use crate::window::winlinks_into;
use crate::window::{window_get_active, window_set_active};
use crate::window::{
    window_pane_reset_mode_all, window_panes_first, window_panes_insert_tail, window_panes_next,
    window_ref_from_ptr, winlink_add, winlink_set_window_ref,
};

impl cmd_parse_result {
    /// The message a failed parse left, taken out of the result so the test
    /// can match on it. Panics when the parse did not fail.
    pub(crate) fn take_error(&mut self) -> String {
        String::from_utf8_lossy(
            self.error
                .take()
                .expect("parse result has no error")
                .as_bytes(),
        )
        .into_owned()
    }
}

/// Every call a [`Prompt::Recorder`] prompt made to its input callback, as the
/// answer it carried and whether it was the final one. A test holds
/// [`globals`], so the list is only ever touched by one of them at a time.
static PROMPT_ANSWERS: ::std::sync::Mutex<Vec<(String, c_int)>> =
    ::std::sync::Mutex::new(Vec::new());

/// What [`Prompt::Recorder`] answers with. Returning zero is what a one-shot
/// prompt's callback does, and what makes the accepting paths take the prompt
/// back down.
pub unsafe fn prompt_recorder(
    _c: *mut client,
    _data: &mut PromptData,
    s: Option<&CStr>,
    done: c_int,
) -> c_int {
    let answer = match s {
        None => "<none>".to_string(),
        Some(s) => unsafe { seen(s.as_ptr()) },
    };
    PROMPT_ANSWERS.lock().unwrap().push((answer, done));
    0
}

/// The answers recorded so far, oldest first.
pub fn prompt_answers() -> Vec<(String, c_int)> {
    PROMPT_ANSWERS.lock().unwrap().clone()
}

/// Forgets every recorded answer, so a test reads only its own prompt's.
pub fn prompt_answers_clear() {
    PROMPT_ANSWERS.lock().unwrap().clear();
}

use crate::cmd::cmd_find_from_winlink;
use crate::cmd::{CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::cmd::{CmdqStateRef, CmdqType, cmdq_get_state_ref, cmdq_new_state};
use crate::cmd::{cmd_get_args, cmd_get_args_ptr, cmd_list_first};
use crate::environ::{environ_create_box, environ_entry_value, environ_free, environ_t};
use crate::ffi::free;
use crate::file::{CLIENT_DEAD, file_fire_done};
use crate::fmt_args;
use crate::format::{
    FORMAT_NONE, format_create, format_defaults, format_expand, format_expand_time,
};
use crate::grid::{grid_create, grid_default_cell, grid_get_cell, grid_set_cell};
use crate::key_bindings::{
    key_binding_cmdlist_ref, key_binding_note, key_bindings_add, key_bindings_get_table,
    key_bindings_remove,
};
use crate::layout::layout_root_ptr;
use crate::options::{
    OPTIONS_TABLE_PANE, OPTIONS_TABLE_SERVER, OPTIONS_TABLE_SESSION, OPTIONS_TABLE_WINDOW,
    options_table,
};
use crate::options::{options_create_boxed, options_default, options_free, options_ptr};
use crate::paste::{paste_free, paste_get_name, paste_set, paste_walk};
use crate::reactor;
use crate::reactor::{IoWatch, Reactor, Timer};
use crate::screen::{screen_free, screen_grid_ptr, screen_init};
use crate::status::status_free;
use crate::terminfo::TtyCode;
use crate::terminfo::tty_term_of;
use ::core::ffi::{CStr, c_char, c_int, c_void};
use ::core::ptr::{null, null_mut};
use ::std::ffi::CString;
use ::std::sync::MutexGuard;

/// The globals `main` sets up that the modules' tests need — the environment,
/// the three option trees and the socket path the format engine reports —
/// together with a turn at the rest of the process-wide state the server keeps
/// in statics (the option trees themselves, the command parser's own globals,
/// the notification queue and the UTF-8 trees and width cache). Cargo runs the
/// tests on parallel threads, so every test that reaches any of it holds the
/// guard this returns.
pub(crate) fn globals() -> MutexGuard<'static, ()> {
    static GLOBALS: ::std::sync::Mutex<()> = ::std::sync::Mutex::new(());
    static SETUP: ::std::sync::Once = ::std::sync::Once::new();
    let guard = GLOBALS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    SETUP.call_once(|| unsafe {
        crate::tmux::global_options_create();
        defaults(crate::tmux::global_options, OPTIONS_TABLE_SERVER);
        defaults(crate::tmux::global_s_options, OPTIONS_TABLE_SESSION);
        defaults(crate::tmux::global_w_options, OPTIONS_TABLE_WINDOW);
        crate::tmux::socket_path = c"/tmp/tmux-fixture/default".as_ptr();
    });
    guard
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
                (*cf.as_ptr()).error = ::libc::EINTR;
                file_fire_done(cf);
            }
            reactor::current().run_once();
            (*c).flags |= CLIENT_DEAD as uint64_t;
            reactor::current().run_once();
            while !(*c).files.is_empty() {
                for cf in (*c).files.values().cloned().collect::<Vec<_>>() {
                    (*cf.as_ptr()).error = ::libc::EINTR;
                    file_fire_done(cf);
                }
                reactor::current().run_once();
            }
        }
        status_free(c);
        (*c).status.screen = screen::default();
        (*c).exit_session = None;
        (*c).exit_message = None;
    }
}

#[cfg(test)]
mod client_handle_tests {
    use super::{globals, zeroed_client};

    #[test]
    fn an_immutable_handle_view_can_be_used_without_an_owning_pointer() {
        let _guard = globals();
        let client = zeroed_client();
        let weak = client.downgrade();
        let pointer = client.with(|value| value as *const _);
        assert_eq!(pointer as *mut _, client.as_ptr());
        drop(client);
        assert!(weak.upgrade().is_none());
    }
}

/// Gives `oo` the default value of every option in `scope`.
unsafe fn defaults(oo: *mut options, scope: c_int) {
    unsafe {
        for oe in &options_table {
            if oe.scope & scope != 0 {
                options_default(oo, oe);
            }
        }
    }
}

/// How many terminal capabilities `tty_term` keeps a slot for.
const TTY_CODES: usize = 233;

/// A zeroed value of a `#[repr(C)]` struct, the way `xcalloc` hands one out.
/// Only for a type every one of whose fields zero is a valid value for — a
/// `Vec` is not one, since a null buffer pointer is not a value it may even
/// hold. A type that has a `Default` says so itself, and is built with that
/// instead; this is for the C structs that have none.
pub(crate) fn zeroed<T>() -> Box<T> {
    Box::new(unsafe { ::core::mem::zeroed() })
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

unsafe fn placeholder_exec(_cmd: &cmd, _item: *mut cmdq_item) -> cmd_retval {
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

/// The item a stored handle names, or null once its queue has given it up.
pub(crate) fn held_item(held: &Option<CmdqItemWeak>) -> *mut cmdq_item {
    held.as_ref()
        .and_then(CmdqItemWeak::upgrade)
        .map_or(null_mut(), |item| item.as_ptr())
}

pub(crate) fn zeroed_cmdq_item(state: CmdqStateRef) -> CmdqItemRef {
    crate::cmd::cmdq_item_new(
        CmdqType::Command {
            cmdlist: None,
            at: 0,
        },
        state,
    )
}

/// An empty valid screen ready for `screen_init` to overwrite.
pub(crate) fn zeroed_screen() -> Box<screen> {
    Box::new(screen::default())
}

/// A client the way `server_client_create` hands one out, near enough: zeroed,
/// but with the five status lines' range lists and the prompt buffer made into
/// real empty `Vec`s first, which is what `status_init` and
/// `server_client_create` do for a live client.
pub(crate) fn zeroed_client() -> ClientRef {
    ClientRef::new(client::default())
}

/// The same for a pane, whose border status line carries one such list and
/// whose visible ranges are a real empty `Vec`.
pub(crate) fn zeroed_window() -> Box<window> {
    Box::new(window::default())
}

pub(crate) fn zeroed_pane() -> Box<window_pane> {
    Box::new(window_pane::default())
}

/// A terminal description the way `tty_term_create` leaves one, near enough:
/// zeroed, but with a full-length code table of missing entries.
pub(crate) fn zeroed_term() -> Box<tty_term> {
    Box::new(tty_term {
        name: None,
        features: 0,
        acs: [[0; 2]; 256],
        codes: (0..TTY_CODES).map(|_| TtyCode::None).collect(),
        flags: 0,
    })
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

    pub(crate) fn ptr(&self) -> *mut grid {
        self.0.as_ref() as *const grid as *mut grid
    }

    /// Writes `s` from (px, py), one ASCII cell per byte.
    pub(crate) fn write(&self, px: u_int, py: u_int, s: &str) {
        for (i, byte) in s.bytes().enumerate() {
            let gc = ascii(byte);
            unsafe { grid_set_cell(&mut *self.ptr(), px + i as u_int, py, &gc) };
        }
    }

    pub(crate) fn cell(&self, px: u_int, py: u_int) -> grid_cell {
        let mut gc = unsafe { grid_default_cell };
        unsafe { gc = grid_get_cell(&*self.ptr(), px, py) };
        gc
    }
}

impl ::core::ops::Deref for Grid {
    type Target = grid;

    fn deref(&self) -> &grid {
        &self.0
    }
}

/// One cell holding the ASCII byte `ch` in the default style.
pub(crate) fn ascii(ch: u8) -> grid_cell {
    let mut gc = unsafe { grid_default_cell };
    gc.data.data[0] = ch;
    gc.data.have = 1;
    gc.data.size = 1;
    gc.data.width = 1;
    gc
}

/// A screen, freed at the end of the test. The screen itself is owned here
/// rather than by the module under test, since `screen_init` fills a caller's
/// struct.
pub(crate) struct Screen(Box<screen>);

impl Screen {
    pub(crate) fn new(sx: u_int, sy: u_int, hlimit: u_int) -> Screen {
        let mut s = Screen(zeroed_screen());
        unsafe { screen_init(&raw mut *s.0, sx, sy, hlimit) };
        s
    }

    pub(crate) fn ptr(&mut self) -> *mut screen {
        &raw mut *self.0
    }

    pub(crate) fn grid(&self) -> *mut grid {
        unsafe { screen_grid_ptr(&raw const *self.0 as *mut screen) }
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        unsafe { screen_free(&raw mut *self.0) };
    }
}

impl ::core::ops::Deref for Screen {
    type Target = screen;

    fn deref(&self) -> &screen {
        &self.0
    }
}

impl ::core::ops::DerefMut for Screen {
    fn deref_mut(&mut self) -> &mut screen {
        &mut self.0
    }
}

/// An option set, freed at the end of the test.
pub(crate) struct Options(*mut options);

impl Options {
    /// An empty set below `parent`.
    pub(crate) fn empty(parent: *mut options) -> Options {
        Options(Box::into_raw(options_create_boxed(parent)))
    }

    /// A set holding the default value of every option in `scope`, one of the
    /// `OPTIONS_TABLE_*` bits.
    pub(crate) fn defaults(scope: c_int) -> Options {
        let oo = Options::empty(null_mut());
        unsafe { defaults(oo.0, scope) };
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

    pub(crate) fn ptr(&self) -> *mut options {
        self.0
    }

    /// Hands the option set over to a caller that takes ownership of it, such
    /// as a session.
    pub(crate) fn leak(self) -> *mut options {
        let oo = self.0;
        ::core::mem::forget(self);
        oo
    }

    /// The same for a caller that owns it as a box of its own.
    pub(crate) fn owned(self) -> Box<options> {
        unsafe { Box::from_raw(self.leak()) }
    }
}

impl Drop for Options {
    fn drop(&mut self) {
        unsafe { options_free(Box::from_raw(self.0)) };
    }
}

/// An environment, freed at the end of the test.
pub(crate) struct Environ(*mut environ_t);

impl Environ {
    pub(crate) fn new() -> Environ {
        Environ(Box::into_raw(environ_create_box()))
    }

    pub(crate) fn from_box(env: Box<environ_t>) -> Environ {
        Environ(Box::into_raw(env))
    }

    /// Takes over an environment somebody else made, so that it is freed at
    /// the end of the test too.
    pub(crate) fn owning(env: *mut environ_t) -> Environ {
        assert!(!env.is_null(), "the environment is missing");
        Environ(env)
    }

    pub(crate) fn ptr(&self) -> *mut environ_t {
        self.0
    }

    /// Hands the environment over to a caller that takes ownership of it, such
    /// as a session.
    pub(crate) fn leak(self) -> *mut environ_t {
        let p = self.0;
        ::core::mem::forget(self);
        p
    }

    /// The same for a caller that owns it as a box of its own, such as a
    /// client.
    pub(crate) fn owned(self) -> Box<environ_t> {
        unsafe { Box::from_raw(self.leak()) }
    }
}

impl Drop for Environ {
    fn drop(&mut self) {
        unsafe { environ_free(self.0) };
    }
}

/// The parsed form of a command line, freed at the end of the test. The
/// command list owns the arguments, so it is kept alive alongside them.
pub(crate) struct Args {
    cmdlist: CmdListRef,
    cmd: *mut cmd,
}

impl Args {
    /// Parses `s` the way the command parser would, and keeps its first
    /// command. Panics if `s` is not a command line.
    pub(crate) fn parse(s: &CStr) -> Args {
        unsafe {
            let mut pr = cmd_parse_from_string(s.as_ptr(), null_mut::<cmd_parse_input>());
            assert_eq!(pr.status, CMD_PARSE_SUCCESS, "{s:?} did not parse");
            let cmdlist = pr.cmdlist.take().unwrap();
            Args {
                cmd: cmd_list_first(cmdlist.as_ptr()),
                cmdlist,
            }
        }
    }

    pub(crate) fn ptr(&self) -> *mut args {
        unsafe { cmd_get_args_ptr(&*self.cmd) }
    }

    /// The list the parsed command sits in, which is what a queue item names
    /// it through.
    pub(crate) fn cmdlist(&self) -> CmdListRef {
        self.cmdlist.clone()
    }

    pub(crate) fn cmd(&self) -> *mut cmd {
        self.cmd
    }

    pub(crate) fn list(&self) -> *mut cmd_list {
        self.cmdlist.as_ptr()
    }

    pub(crate) fn list_ref(&self) -> &CmdListRef {
        &self.cmdlist
    }
}

/// A command-queue item, the command it is running and the state it shares
/// with the rest of its queue, all zeroed the way `xcalloc` hands them out.
/// That is enough for the fields the commands themselves touch:
/// `cmdq_get_client` reads the item's client, `cmdq_continue` clears its
/// waiting flag, `cmdq_error` reaches `cfg_add_cause` through the command's
/// file name and line, and `cmdq_merge_formats` reads the state's format tree.
/// The item itself, which the fixture owns until a queue takes it over.
enum ItemBox {
    Owned(CmdqItemRef),
    Queued(*mut cmdq_item),
}

impl ::core::ops::Deref for ItemBox {
    type Target = cmdq_item;

    fn deref(&self) -> &cmdq_item {
        match self {
            ItemBox::Owned(item) => item.item(),
            ItemBox::Queued(item) => unsafe { &**item },
        }
    }
}

impl ::core::ops::DerefMut for ItemBox {
    fn deref_mut(&mut self) -> &mut cmdq_item {
        match self {
            ItemBox::Owned(item) => item.item(),
            ItemBox::Queued(item) => unsafe { &mut **item },
        }
    }
}

pub(crate) struct Item {
    item: ItemBox,
    cmdlist: CmdListRef,
    client: ClientRef,
    state: CmdqStateRef,
    args: Option<Args>,
}

impl Item {
    /// An item with no client behind it.
    pub(crate) fn new() -> Item {
        let cmdlist = crate::cmd::cmd_list_new();
        unsafe { crate::cmd::cmd_list_append(cmdlist.as_ptr(), empty_cmd()) };
        let state = unsafe { cmdq_new_state(null_mut(), null_mut(), 0) };
        let mut it = Item {
            item: ItemBox::Owned(zeroed_cmdq_item(state.clone())),
            cmdlist: cmdlist.clone(),
            client: zeroed_client(),
            state,
            args: None,
        };
        it.item.type_0 = CmdqType::Command {
            cmdlist: Some(cmdlist),
            at: 0,
        };
        it
    }

    /// An item a client is waiting on, as the command queue would hand it to a
    /// command.
    ///
    pub(crate) fn with_client() -> Item {
        let mut it = Item::new();
        it.item.client = Some(it.client.clone());
        it
    }

    pub(crate) fn set_client(&mut self, c: *mut client) {
        self.item.client = crate::server::client_ref_from_ptr(c);
    }

    /// Where the command came from, which is what `cmdq_error` reports.
    pub(crate) fn from_file(mut self, file: &'static CStr, line: u_int) -> Item {
        unsafe {
            (*self.cmd()).file = Some(file.to_owned());
            (*self.cmd()).line = line;
        }
        self
    }

    /// Runs the command line `s` through the parser and points the item's
    /// command at the arguments it produced.
    pub(crate) fn with_args(mut self, s: &CStr) -> Item {
        let mut args = Args::parse(s);
        unsafe {
            (*self.cmd()).entry = (*args.cmd()).entry;
            (*self.cmd()).args = Some((*args.cmd()).args.take().unwrap());
        }
        self.args = Some(args);
        self
    }

    /// Points the item's target, source and current states at `target`'s
    /// winlink, the way the command queue prepares an item before running its
    /// command. The item's own client, if it has one, becomes the target
    /// client too.
    pub(crate) fn targeting(mut self, target: &mut Target) -> Item {
        let fs = target.state();
        self.item.target = fs.clone();
        self.item.source = fs.clone();
        unsafe { (*self.state.as_ptr()).current = fs };
        if let Some(client) = self.item.client.as_ref() {
            self.item.target_client = Some(client.downgrade());
        }
        self
    }

    pub(crate) fn ptr(&mut self) -> *mut cmdq_item {
        &raw mut *self.item
    }

    /// Puts the item at the back of `queue`, which takes it over, so that the
    /// command queue can find it there the way it finds an item it queued
    /// itself. The fixture keeps the command and state it points at alive.
    pub(crate) fn queue_onto(&mut self, queue: &mut cmdq_list) -> *mut cmdq_item {
        let ptr = self.ptr();
        if let ItemBox::Owned(item) = ::core::mem::replace(&mut self.item, ItemBox::Queued(ptr)) {
            item.item().queue = &raw mut *queue;
            queue.list.push_back(item);
        }
        ptr
    }

    pub(crate) fn cmd(&mut self) -> *mut cmd {
        unsafe { crate::cmd::cmd_list_at(self.cmdlist.as_ptr(), 0) }
    }

    pub(crate) fn client(&mut self) -> *mut client {
        &raw mut *self.client
    }

    pub(crate) fn state_ref(&self) -> &CmdqStateRef {
        unsafe { cmdq_get_state_ref(&raw const *self.item as *mut cmdq_item) }
    }

    pub(crate) fn flags(&self) -> c_int {
        self.item.flags
    }

    pub(crate) fn set_flags(&mut self, flags: c_int) {
        self.item.flags = flags;
    }
}

impl Drop for Item {
    fn drop(&mut self) {
        unsafe { release_client(&raw mut *self.client) };
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
    name: CString,
}

impl Session {
    pub(crate) fn new(id: u_int, name: &str) -> Session {
        let name = CString::new(name).expect("a session name has no NUL");
        let session = session_new_detached(
            id,
            name.clone(),
            CString::new("/").expect("no NUL"),
            Options::session().owned(),
            Environ::new().owned(),
        );
        let mut s = Session { session, name };
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

    pub(crate) fn environ(&self) -> *mut environ_t {
        unsafe { session_environ(self.session.as_ptr()) }
    }

    pub(crate) fn options(&self) -> *mut options {
        unsafe { session_options(self.session.as_ptr()) }
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
        value.name = Some(CString::new(name).expect("a window name has no NUL"));
        value.old_layout = None;
        value.fill_character = None;
        value.options = Some(Options::window().owned());
        value.sx = sx;
        value.sy = sy;
        value.manual_sx = sx;
        value.manual_sy = sy;
        value.lastlayout = -1;
        value.winlinks = window_winlinks::new();
        let window = WindowRef::new(*value);
        window.mark_unmanaged();
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

    pub(crate) fn options(&self) -> *mut options {
        unsafe { options_ptr(&(*self.window.as_ptr()).options) }
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
        unsafe {
            let w = self.window.as_ptr();
            let owned = pane.take();
            let wp = window_panes_insert_tail(w, owned);
            (*wp).window = w;
            crate::window::pane_registry_add(wp);
            (*w).z_index.push((*wp).id);
            if window_get_active(w).is_null() {
                window_set_active(w, wp);
            }
        }
    }
}

/// Gives back what [`Pane::new`] made: the two screens, the timers and the
/// option set. A fixture pane has no process behind it and no entry in the
/// server's pane tree, so this is the whole of its teardown.
fn free_pane(pane: &mut window_pane) {
    unsafe {
        pane.resize_timer.disarm();
        pane.sync_timer.disarm();
        screen_free(&raw mut pane.status_screen);
        screen_free(&raw mut pane.base);
        if let Some(oo) = pane.options.take() {
            options_free(oo);
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            let w = self.window.as_ptr();
            for mut pane in ::core::mem::take(&mut (*w).panes) {
                crate::window::pane_registry_remove(pane.id);
                free_pane(&mut pane);
            }
            (*w).z_index.clear();
            (*w).last_panes.clear();
            window_set_active(w, null_mut::<window_pane>());
        }
    }
}

/// A pane that is **not** in the server's `all_window_panes` tree, has no
/// process behind it (`fd` and `pipe_fd` stay -1, `argv` and `shell` stay
/// null), no timers armed and no input parser. Its base screen is
/// real — `screen_init` fills it and `screen_free` empties it again — and
/// `screen` points at that base, which is what the drawing and copy-mode code
/// reads. Nothing here spawns a shell; a test that wants one wants the
/// conformance suite.
///
/// The pane owns itself until a [`Window`] takes it over, which is what
/// [`Window::add_pane`] does; after that the window is what gives it back and
/// this is only the pointer to it.
pub(crate) struct Pane {
    pane: Option<Box<window_pane>>,
    ptr: *mut window_pane,
}

impl Pane {
    pub(crate) fn new(id: u_int, sx: u_int, sy: u_int, hlimit: u_int) -> Pane {
        let mut pane = zeroed_pane();
        pane.id = id;
        crate::window::window_pane_reserve_id(id);
        pane.options = Some(Options::pane().owned());
        pane.sx = sx;
        pane.sy = sy;
        pane.fd = -1;
        pane.pipe_fd = -1;
        pane.control_bg = -1;
        pane.control_fg = -1;
        unsafe {
            crate::style::style_ranges_init(&raw mut pane.border_status_line.ranges);
            screen_init(&raw mut pane.base, sx, sy, hlimit);
            screen_init(&raw mut pane.status_screen, 1, 1, 0);
        }
        pane.shown = crate::types::PaneScreen::Base;
        let ptr = &raw mut *pane;
        Pane {
            pane: Some(pane),
            ptr,
        }
    }

    pub(crate) fn ptr(&mut self) -> *mut window_pane {
        self.ptr
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
            let wp = window_panes_insert_tail(w, self.take());
            crate::window::window_pane_set_window(wp, w);
            crate::window::pane_registry_add(wp);
            (*w).z_index.push((*wp).id);
            wp
        }
    }

    pub(crate) fn screen(&mut self) -> *mut screen {
        unsafe { (*self.ptr).screen() }
    }

    pub(crate) fn options(&self) -> *mut options {
        unsafe { options_ptr(&(*self.ptr).options) }
    }
}

impl Drop for Pane {
    fn drop(&mut self) {
        if let Some(mut pane) = self.pane.take() {
            free_pane(&mut pane);
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
    /// A window of `sx` by `sy` with one pane filling it, as `layout_init`
    /// leaves a freshly created window.
    pub(crate) fn new(sx: u_int, sy: u_int) -> Layout {
        let mut l = Layout {
            window: Window::new(1, "layout", sx, sy),
            panes: Vec::new(),
            next_id: 0,
        };
        l.add_pane(sx, sy);
        unsafe { crate::layout::layout_init(l.w(), l.pane(0)) };
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
        unsafe { dump_cell(layout_root_ptr(&(*self.w()).layout_root)) }
    }

    /// The sizes and offsets the panes themselves were given.
    pub(crate) fn panes(&mut self) -> Vec<String> {
        unsafe {
            let mut out = Vec::new();
            let w = self.w();
            let mut wp = window_panes_first(w);
            while !wp.is_null() {
                out.push(format!(
                    "%{} {}x{}+{}+{}",
                    (*wp).id,
                    (*wp).sx,
                    (*wp).sy,
                    (*wp).xoff,
                    (*wp).yoff
                ));
                wp = window_panes_next(w, wp);
            }
            out
        }
    }
}

impl Drop for Layout {
    fn drop(&mut self) {
        unsafe { crate::layout::layout_free(self.window.ptr()) };
    }
}

/// One cell of a layout tree as a string, with its children in brackets. A
/// floating cell is marked with a star.
pub(crate) unsafe fn dump_cell(lc: *mut layout_cell) -> String {
    unsafe {
        use crate::layout::{
            LAYOUT_CELL_FLOATING, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE,
        };
        if lc.is_null() {
            return "-".to_string();
        }
        let here = format!("{}x{}+{}+{}", (*lc).sx, (*lc).sy, (*lc).xoff, (*lc).yoff);
        let floating = if (*lc).flags & LAYOUT_CELL_FLOATING != 0 {
            "*"
        } else {
            ""
        };
        match (*lc).type_0 {
            LAYOUT_WINDOWPANE => format!("%{}{floating} {here}", (*lc).wp_id.unwrap_or(u_int::MAX)),
            LAYOUT_LEFTRIGHT | LAYOUT_TOPBOTTOM => {
                let kids: Vec<String> = crate::list::foreach_owned(&raw mut (*lc).cells)
                    .map(|child| dump_cell(child))
                    .collect();
                let name = if (*lc).type_0 == LAYOUT_LEFTRIGHT {
                    "LR"
                } else {
                    "TB"
                };
                format!("{name}{floating} {here} [{}]", kids.join(" | "))
            }
            _ => format!("?{floating} {here}"),
        }
    }
}

/// A terminal that is **not** attached to anything: a zeroed `tty` pointing at
/// a zeroed `tty_term` and the zeroed `client` behind it. The term's code table
/// is a full-length list of missing entries, so `tty_term_has` answers
/// no for every capability until [`Tty::set_number`] gives one a value, and its
/// ACS table starts empty. No terminfo entry is read, no descriptor is open and
/// no timer is armed; a test that wants a real terminal wants the
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
        let mut term = zeroed_term();
        t.tty.term = Some(term);
        t.tty.owner = crate::server::client_ref_from_ptr(&raw mut *t.client).map(|c| c.downgrade());
        t
    }

    pub(crate) fn ptr(&mut self) -> *mut tty {
        &raw mut *self.tty
    }

    pub(crate) fn term(&self) -> &tty_term {
        tty_term_of(&self.tty)
    }

    /// The terminal as a raw pointer, for the calls that still take one.
    pub(crate) fn term_ptr(&mut self) -> *mut tty_term {
        self.term_mut()
    }

    pub(crate) fn term_mut(&mut self) -> &mut tty_term {
        self.tty.term.as_mut().expect("the fixture built a term")
    }

    /// Gives `code` a number, as a terminfo entry carrying that capability
    /// would.
    pub(crate) fn set_number(&mut self, code: tty_code_code, number: c_int) {
        self.term_mut().codes[code as usize] = TtyCode::Number(number);
    }

    /// Gives `code` a string, as a terminfo entry carrying that capability
    /// would.
    pub(crate) fn set_string(&mut self, code: tty_code_code, s: &CStr) {
        self.term_mut().codes[code as usize] = TtyCode::String(s.to_owned());
    }

    /// Gives `code` a flag, as a boolean terminfo capability would.
    pub(crate) fn set_flag(&mut self, code: tty_code_code, flag: c_int) {
        self.term_mut().codes[code as usize] = TtyCode::Flag(flag);
    }

    /// Takes `code` back out of the terminal's table, leaving it missing.
    pub(crate) fn clear_code(&mut self, code: tty_code_code) {
        self.term_mut().codes[code as usize] = TtyCode::None;
    }

    /// The one-byte ACS translation the terminal reports for `ch`, as the
    /// `acsc` capability would fill in. An empty string means it has none.
    pub(crate) fn set_acs(&mut self, ch: u8, to: &str) {
        let bytes = to.as_bytes();
        assert!(bytes.len() < 2, "an ACS translation is a single byte");
        self.term_mut().acs[ch as usize] =
            [bytes.first().copied().unwrap_or(0) as c_char, 0 as c_char];
    }

    pub(crate) fn set_client_flags(&mut self, flags: u64) {
        self.client.flags = flags;
    }
}

/// Initializes the process-local reactor used by tests.
pub(crate) fn ensure_reactor() {
    static BASE: ::std::sync::Once = ::std::sync::Once::new();
    BASE.call_once(|| {
        reactor::current();
    });
}

/// A stream over one end of a socket pair used by tests that need buffered
/// output without a peer.
pub(crate) struct StreamBuffer {
    bev: Stream,
    fds: [c_int; 2],
    seen: ::std::cell::Cell<usize>,
}

impl StreamBuffer {
    pub(crate) fn new() -> StreamBuffer {
        ensure_reactor();
        let mut fds = [-1 as c_int; 2];
        unsafe {
            assert_eq!(
                ::libc::socketpair(::libc::AF_UNIX, ::libc::SOCK_STREAM, 0, fds.as_mut_ptr()),
                0,
                "no socket pair"
            );
            let bev = Stream::new(fds[0], None, None, None);
            assert!(!bev.is_none(), "no buffer event");
            StreamBuffer {
                bev,
                fds,
                seen: ::std::cell::Cell::new(0),
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
            ::libc::close(self.fds[0]);
            ::libc::close(self.fds[1]);
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
        unsafe {
            assert!(
                crate::server::clients.is_empty(),
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
        c.name = Some(CString::new(name).expect("a client name has no NUL"));
        c.tty.sx = sx;
        c.tty.sy = sy;
        let p = &raw mut *c;
        unsafe {
            crate::server::clients.push(c.clone());
        }
        self.clients.push(c);
        p
    }
}

impl Drop for Clients {
    fn drop(&mut self) {
        unsafe {
            crate::server::clients.clear();
            for c in &mut self.clients {
                release_client(c.as_ptr());
            }
        }
    }
}

/// The server's global `sessions`, `windows` and `all_window_panes` trees,
/// holding fixture sessions, windows and panes for the length of a test. All
/// three are globals, so a test that builds one takes [`globals`] too; each
/// tree starts empty, which [`Registry::new`] asserts, and is emptied again at
/// the end of the test, without ever running session or window teardown.
///
/// Emptying is a reset of the tree heads, the way [`Clients`] gives back the
/// client list, and not a node-by-node removal: a removal reads and writes the
/// nodes, and a test's fixtures are ordinary locals which may already have gone
/// out of scope by the time the registry does.
pub(crate) struct Registry;

impl Registry {
    pub(crate) fn new() -> Registry {
        assert!(
            crate::session::sessions.map().is_empty(),
            "the session tree is not empty"
        );
        assert!(
            crate::window::windows.map().is_empty(),
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
        let p = s.ptr();
        unsafe {
            let name = ::core::ffi::CStr::from_ptr(session_name(p)).to_owned();
            crate::session::sessions
                .map()
                .insert(name, s.session.clone());
        };
    }

    /// Puts `w` in the window tree, which is keyed by id.
    pub(crate) fn add_window(&mut self, w: &mut Window) {
        let p = w.ptr();
        unsafe {
            crate::window::windows
                .map()
                .insert((*p).id, w.window.downgrade())
        };
    }

    /// Puts `pane` in the tree of every pane the server has, which is keyed by
    /// id.
    pub(crate) fn add_pane(&mut self, pane: &mut Pane) {
        let p = pane.ptr();
        unsafe { crate::window::pane_registry_add(p) };
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        crate::session::session_registry_clear();
        crate::window::windows.map().clear();
        crate::window::pane_registry_clear();
    }
}

/// Links `window` into `session` at index `idx` through the real winlink tree.
/// Returns the winlink, which the session's tree owns until [`unlink`] takes it
/// away.
pub(crate) fn link(session: &mut Session, window: &mut Window, idx: c_int) -> *mut winlink {
    unsafe {
        let wl = winlink_add(&raw mut (*session.ptr()).windows, idx);
        assert!(!wl.is_null(), "index {idx} is already linked");
        (*wl).set_session(session.ptr());
        winlink_set_window_ref(wl, window.window.clone());
        if session_get_curw(session.ptr()).is_null() {
            session_set_curw(session.ptr(), wl);
        }
        wl
    }
}

/// Takes `wl` back out of its session, freeing it.
pub(crate) fn unlink(session: &mut Session, wl: *mut winlink) {
    unsafe {
        if session_get_curw(session.ptr()) == wl {
            session_set_curw(session.ptr(), null_mut::<winlink>());
        }
        crate::window::winlink_remove(&raw mut (*session.ptr()).windows, wl);
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
        let s = session.ptr();
        while let Some(wl) = (*s)
            .windows
            .values()
            .next()
            .map(|wl| wl.as_ref() as *const winlink as *mut winlink)
        {
            unlink(session, wl);
        }
        (*s).lastw.clear();
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
        self.registry.add_pane(&mut p);
        let wl = link(&mut self.session, &mut w, idx);
        self.windows.push(w);
        self.panes.push(p);
        self.winlinks.push(wl);
        self.windows.len() - 1
    }

    pub(crate) fn session(&mut self) -> *mut session {
        self.session.ptr()
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
        unsafe { cmd_find_from_winlink(&mut fs, session_get_curw(self.session.ptr()), 0) };
        fs
    }
}

impl Drop for Target {
    fn drop(&mut self) {
        for p in &mut self.panes {
            unsafe { window_pane_reset_mode_all(p.ptr()) };
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
        Format(unsafe {
            format_create(
                null_mut::<client>(),
                null_mut::<cmdq_item>(),
                FORMAT_NONE,
                0,
            )
        })
    }

    /// A tree carrying the defaults for whichever of a client, session,
    /// winlink and pane are given, the way `format_create_defaults` builds one
    /// for a command.
    pub(crate) fn defaults(
        c: *mut client,
        s: *mut session,
        wl: *mut winlink,
        wp: *mut window_pane,
    ) -> Format {
        let ft = Format::new();
        unsafe { format_defaults(&mut *ft.ptr(), c, s, wl, wp) };
        ft
    }

    /// A tree carrying the defaults of `target`'s current winlink, with no
    /// client.
    pub(crate) fn from_target(target: &mut Target) -> Format {
        let fs = target.state();
        Format::defaults(null_mut::<client>(), fs.session(), fs.winlink(), fs.pane())
    }

    pub(crate) fn ptr(&self) -> *mut format_tree {
        &raw const *self.0 as *mut format_tree
    }

    /// The tree itself, to hand to the calls that borrow it.
    pub(crate) fn tree(&mut self) -> &mut format_tree {
        &mut self.0
    }

    pub(crate) fn into_box(self) -> Box<format_tree> {
        self.0
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
            let mut pr = cmd_parse_from_string(s.as_ptr(), null_mut::<cmd_parse_input>());
            assert_eq!(pr.status, CMD_PARSE_SUCCESS, "{s:?} did not parse");
            key_bindings_add(
                self.name.as_ptr(),
                key,
                note.map_or(null::<c_char>(), |n| n.as_ptr()),
                0,
                pr.cmdlist.take(),
            );
        }
        self.keys.push(key);
    }

    /// The table itself, which exists once something is bound in it.
    pub(crate) fn ptr(&self) -> *mut key_table {
        unsafe { key_bindings_get_table(self.name.as_ptr(), 0) }
    }
}

impl Drop for KeyTable {
    fn drop(&mut self) {
        unsafe {
            for key in &self.keys {
                key_bindings_remove(self.name.as_ptr(), *key);
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
        unsafe { Paste::empty() };
        Paste(())
    }

    /// A buffer named `name` holding `data`, owned by the store.
    pub(crate) fn add(&self, name: &CStr, data: &str) -> *mut paste_buffer {
        unsafe {
            assert!(
                paste_set(data.as_bytes().to_vec(), name.as_ptr()).is_ok(),
                "buffer {name:?} was not set"
            );
            paste_get_name(name.as_ptr())
        }
    }

    unsafe fn empty() {
        unsafe {
            let mut pb = paste_walk(null_mut::<paste_buffer>());
            while !pb.is_null() {
                let next = paste_walk(pb);
                paste_free(pb);
                pb = next;
            }
        }
    }
}

impl Drop for Paste {
    fn drop(&mut self) {
        unsafe { Paste::empty() };
    }
}

/// The contents of a C string the caller now owns, freeing it.
pub(crate) unsafe fn taken(p: *mut c_char) -> String {
    unsafe {
        let s = String::from_utf8_lossy(CStr::from_ptr(p).to_bytes()).into_owned();
        free(p as *mut c_void);
        s
    }
}

/// The contents of a C string somebody else still owns.
pub(crate) unsafe fn seen(p: *const c_char) -> String {
    unsafe {
        assert!(!p.is_null(), "the string is missing");
        String::from_utf8_lossy(CStr::from_ptr(p).to_bytes()).into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environ::{environ_find, environ_set};
    use crate::grid::grid_string_cells;
    use crate::options::options_get_string;
    use crate::window::{winlink_count, winlink_find_by_index};

    #[test]
    fn a_buffer_event_keeps_what_is_written_to_it() {
        let _guard = globals();
        let bev = StreamBuffer::new();
        unsafe {
            bev.ptr().write(c"hi".as_ptr() as *const u8, 2);
        }
        assert_eq!(bev.written(), b"hi");
        assert_eq!(bev.written(), b"");
    }

    #[test]
    fn the_option_sets_carry_the_defaults_of_their_scope() {
        let _guard = globals();
        let session = Options::session();
        let window = Options::window();
        let pane = Options::pane();
        unsafe {
            assert_eq!(
                seen(options_get_string(
                    session.ptr(),
                    c"word-separators".as_ptr()
                )),
                "!\"#$%&'()*+,-./:;<=>?@[\\]^`{|}~"
            );
            assert!(options_get_ptr(session.ptr(), c"window-status-format".as_ptr()).is_null());
            assert_eq!(
                seen(options_get_string(
                    window.ptr(),
                    c"window-status-format".as_ptr()
                )),
                "#I:#W#{?window_flags,#{window_flags}, }"
            );
            assert!(!options_get_ptr(pane.ptr(), c"pane-border-format".as_ptr()).is_null());
            assert!(options_get_ptr(pane.ptr(), c"word-separators".as_ptr()).is_null());
            let child = Options::empty(session.ptr());
            assert!(options_get_only_ptr(child.ptr(), c"word-separators".as_ptr()).is_null());
            assert_eq!(
                seen(options_get_string(child.ptr(), c"word-separators".as_ptr())),
                "!\"#$%&'()*+,-./:;<=>?@[\\]^`{|}~"
            );
        }
    }

    #[test]
    fn a_grid_takes_text_and_reads_it_back() {
        let _guard = globals();
        let grid = Grid::new(10, 5, 100);
        grid.write(0, 0, "abc");
        assert_eq!(grid.sx, 10);
        assert_eq!(grid.cell(1, 0).data.data[0], b'b');
        unsafe {
            let p = grid_string_cells(&*grid.ptr(), 0, 0, 10, None, 0, null_mut());
            assert_eq!(p.to_string_lossy(), "abc");
        }
    }

    #[test]
    fn a_screen_comes_with_a_grid_of_its_own() {
        let _guard = globals();
        let mut s = Screen::new(10, 5, 100);
        assert!(!s.ptr().is_null());
        unsafe {
            assert_eq!((*s.grid()).sx, 10);
            assert_eq!((*s.grid()).sy, 5);
        }
    }

    #[test]
    fn an_environment_holds_what_is_put_in_it() {
        let _guard = globals();
        let env = Environ::new();
        unsafe {
            environ_set(
                env.ptr(),
                c"FOO".as_ptr(),
                0,
                c"%s".as_ptr(),
                fmt_args![c"bar".as_ptr()],
            );
            assert_eq!(
                seen(environ_entry_value(
                    environ_find(&*env.ptr(), c"FOO".as_ptr()).expect("the entry just set"),
                )),
                "bar"
            );
        }
    }

    #[test]
    fn a_parsed_command_line_hands_over_its_arguments() {
        let _guard = globals();
        let args = Args::parse(c"wait-for -S chan");
        unsafe {
            assert_eq!(crate::arguments::args_has(&*args.ptr(), b'S'), 1);
            assert_eq!(seen(crate::arguments::args_string(&*args.ptr(), 0)), "chan");
            assert!(!args.list().is_null());
        }
    }

    #[test]
    fn an_item_carries_a_client_a_command_and_its_arguments() {
        let _guard = globals();
        let mut plain = Item::new();
        assert!(unsafe { crate::cmd::cmdq_get_client(&*plain.ptr()) }.is_null());
        assert_eq!(unsafe { (*plain.ptr()).cmd() }, plain.cmd());

        let mut item = Item::with_client()
            .from_file(c"fixture.conf", 7)
            .with_args(c"display-message hello");
        item.set_flags(3);
        assert_eq!(item.flags(), 3);
        unsafe {
            assert_eq!(crate::cmd::cmdq_get_client(&*item.ptr()), item.client());
            assert_eq!(seen(cstr_ptr(&(*item.cmd()).file)), "fixture.conf");
            assert_eq!((*item.cmd()).line, 7);
            assert_eq!(
                seen(crate::arguments::args_string(cmd_get_args(&*item.cmd()), 0)),
                "hello"
            );
        }
    }

    #[test]
    fn a_session_carries_an_id_a_name_and_trees_of_its_own() {
        let _guard = globals();
        let mut s = Session::new(4, "fixture");
        unsafe {
            assert_eq!(session_id(s.ptr()), 4);
            assert_eq!(seen(session_name(s.ptr())), "fixture");
            assert!((*s.ptr()).windows.is_empty());
            assert!((*s.ptr()).lastw.is_empty());
            assert_eq!(session_options(s.ptr()), s.options());
            assert_eq!(session_environ(s.ptr()), s.environ());
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
            assert_eq!(window_get_active(w.ptr()), first.ptr());
            assert_eq!(window_panes_first(w.ptr()), first.ptr());
            assert_eq!(window_panes_next(w.ptr(), first.ptr()), second.ptr());
            assert_eq!(window_panes_next(w.ptr(), second.ptr()), null_mut());
            assert_eq!((*first.ptr()).window, w.ptr());
            assert_eq!(first.screen(), &raw mut (*first.ptr()).base);
            assert!((*first.screen()).grid.is_some());
            assert_eq!((*first.ptr()).fd, -1);
            assert_eq!(options_ptr(&(*first.ptr()).options), first.options());
            assert_eq!(seen(cstr_ptr(&(*w.ptr()).name)), "fixture");
            assert_eq!(options_ptr(&(*w.ptr()).options), w.options());
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
        assert_eq!(unsafe { (*l.pane(1)).id }, 2);
        assert_eq!(unsafe { (*l.w()).sx }, 80);
        assert_eq!(l.window().options(), unsafe {
            options_ptr(&(*l.w()).options)
        });
    }

    #[test]
    fn a_target_is_a_registered_session_window_and_pane() {
        let _guard = globals();
        let mut t = Target::new(80, 24);
        unsafe {
            assert_eq!(
                crate::session::session_find(c"0".as_ptr() as *mut c_char),
                t.session()
            );
            assert_eq!(crate::window::window_find_by_id(0), t.window(0));
            assert_eq!(crate::window::window_pane_find_by_id(0), t.pane(0));
            assert_eq!(session_get_curw(t.session()), t.winlink(0));
            let fs = t.state();
            assert_eq!(fs.session(), t.session());
            assert_eq!(fs.winlink(), t.winlink(0));
            assert_eq!(fs.window(), t.window(0));
            assert_eq!(fs.pane(), t.pane(0));
            let i = t.add_window(5, 80, 24);
            assert_eq!((*t.winlink(i)).idx, 5);
            assert_eq!((*t.winlink(i)).window(), t.window(i));
            assert_eq!(winlink_count(&raw mut (*t.session()).windows), 2);
        }
    }

    #[test]
    fn an_item_targeting_a_target_carries_its_find_states() {
        let _guard = globals();
        let mut t = Target::new(80, 24);
        let (s, wl, wp) = (t.session(), t.winlink(0), t.pane(0));
        let mut item = Item::with_client().targeting(&mut t);
        unsafe {
            let target = crate::cmd::cmdq_get_target(item.ptr());
            assert_eq!((*target).session(), s);
            assert_eq!((*target).winlink(), wl);
            assert_eq!((*target).pane(), wp);
            let current = crate::cmd::cmdq_get_current(item.ptr());
            assert_eq!((*current).session(), s);
            assert_eq!(
                crate::cmd::cmdq_get_target_client(&*item.ptr()),
                item.client()
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
            let kt = table.ptr();
            assert!(!kt.is_null(), "no table");
            unsafe {
                let bd = crate::key_bindings::key_bindings_get(kt, b'x' as key_code);
                assert!(!bd.is_null(), "no binding for x");
                assert_eq!(key_binding_note(bd), Some(c"a fixture binding"));
                assert!(key_binding_cmdlist_ref(bd).is_some());
            }
        }
        assert!(
            unsafe { key_bindings_get_table(c"fixture-keys".as_ptr(), 0) }.is_null(),
            "the table is still there"
        );
    }

    #[test]
    fn a_paste_turn_starts_empty_and_holds_named_buffers() {
        let _guard = globals();
        {
            let store = Paste::new();
            assert!(unsafe { crate::paste::paste_get_top(null_mut()) }.is_null());
            let pb = store.add(c"fixture", "hello");
            assert!(!pb.is_null());
            unsafe {
                assert_eq!(crate::paste::paste_buffer_data(&*pb), b"hello");
                assert_eq!(crate::paste::paste_get_name(c"fixture".as_ptr()), pb);
            }
        }
        assert!(unsafe { crate::paste::paste_get_top(null_mut()) }.is_null());
    }

    #[test]
    fn a_winlink_joins_a_session_and_a_window() {
        let _guard = globals();
        let mut s = Session::new(1, "linked");
        let mut w = Window::new(1, "win", 80, 24);
        let wl = link(&mut s, &mut w, 0);
        unsafe {
            assert_eq!((*wl).idx, 0);
            assert_eq!((*wl).session(), s.ptr());
            assert_eq!((*wl).window(), w.ptr());
            assert_eq!(session_get_curw(s.ptr()), wl);
            assert_eq!(winlink_count(&raw mut (*s.ptr()).windows), 1);
            assert_eq!(winlink_find_by_index(&raw mut (*s.ptr()).windows, 0), wl);
            assert_eq!(
                winlinks_into(w.ptr())
                    .next()
                    .unwrap_or(::core::ptr::null_mut()),
                wl
            );
            assert!((*wl).window_ref.is_some());
        }
        unlink(&mut s, wl);
        unsafe {
            assert_eq!(winlink_count(&raw mut (*s.ptr()).windows), 0);
            assert!(session_get_curw(s.ptr()).is_null());
            assert!(window_ref_from_ptr(w.ptr()).is_some());
        }
    }
}
