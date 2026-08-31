use crate::arguments::{
    args_count, args_flags, args_get, args_print, args_string, args_value_list,
};
use crate::cfg::cfg_add_cause;
use crate::cfg::cfg_finished;
use crate::cmd::cmd_list_at;
use crate::cmd::find::{
    cmd_find_clear_state, cmd_find_client, cmd_find_copy_state, cmd_find_from_client,
    cmd_find_target, cmd_find_valid_state,
};
use crate::cmd::{
    cmd_get_args, cmd_get_args_ptr, cmd_get_entry, cmd_get_group, cmd_get_source, cmd_list_all,
    cmd_print,
};
use crate::control::control_write;
use crate::ffi::{__ctype_toupper_loc, getpwuid, getuid, time};
use crate::file::file_error;
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc, format_buf};
use crate::format::{format_add, format_create, format_merge};
use crate::list::foreach_safe_after_by;
use crate::log::{fatalx, log_debug, log_get_level};
use crate::options::options_get_ptr;
use crate::options::{options_array_first, options_array_item_command, options_array_next};
use crate::proc::proc_get_peer_uid;
use crate::server::server_add_message;
use crate::server::{client_ref_from_ptr, server_client_print};
use crate::session::session_options;
use crate::status::status_message_set;
use crate::text::key_string_lookup_key;
use crate::text::utf8_sanitize;
use crate::tmux::global_s_options;
use crate::tree::{GlobalQueue, GlobalTree};
pub use crate::types::*;
use crate::xmalloc::xasprintf;
use ::core::cell::UnsafeCell;
use ::std::ffi::{CStr, CString};
use ::std::rc::{Rc, Weak};
pub const MSG_READ_CANCEL: msgtype = 307;
pub const MSG_WRITE_CLOSE: msgtype = 306;
pub const MSG_WRITE_READY: msgtype = 305;
pub const MSG_WRITE: msgtype = 304;
pub const MSG_WRITE_OPEN: msgtype = 303;
pub const MSG_READ_DONE: msgtype = 302;
pub const MSG_READ: msgtype = 301;
pub const MSG_READ_OPEN: msgtype = 300;
pub const MSG_FLAGS: msgtype = 218;
pub const MSG_EXEC: msgtype = 217;
pub const MSG_WAKEUP: msgtype = 216;
pub const MSG_UNLOCK: msgtype = 215;
pub const MSG_SUSPEND: msgtype = 214;
pub const MSG_OLDSTDOUT: msgtype = 213;
pub const MSG_OLDSTDIN: msgtype = 212;
pub const MSG_OLDSTDERR: msgtype = 211;
pub const MSG_SHUTDOWN: msgtype = 210;
pub const MSG_SHELL: msgtype = 209;
pub const MSG_RESIZE: msgtype = 208;
pub const MSG_READY: msgtype = 207;
pub const MSG_LOCK: msgtype = 206;
pub const MSG_EXITING: msgtype = 205;
pub const MSG_EXITED: msgtype = 204;
pub const MSG_EXIT: msgtype = 203;
pub const MSG_DETACHKILL: msgtype = 202;
pub const MSG_DETACH: msgtype = 201;
pub const MSG_COMMAND: msgtype = 200;
pub const MSG_IDENTIFY_TERMINFO: msgtype = 112;
pub const MSG_IDENTIFY_LONGFLAGS: msgtype = 111;
pub const MSG_IDENTIFY_STDOUT: msgtype = 110;
pub const MSG_IDENTIFY_FEATURES: msgtype = 109;
pub const MSG_IDENTIFY_CWD: msgtype = 108;
pub const MSG_IDENTIFY_CLIENTPID: msgtype = 107;
pub const MSG_IDENTIFY_DONE: msgtype = 106;
pub const MSG_IDENTIFY_ENVIRON: msgtype = 105;
pub const MSG_IDENTIFY_STDIN: msgtype = 104;
pub const MSG_IDENTIFY_OLDCWD: msgtype = 103;
pub const MSG_IDENTIFY_TTYNAME: msgtype = 102;
pub const MSG_IDENTIFY_TERM: msgtype = 101;
pub const MSG_IDENTIFY_FLAGS: msgtype = 100;
pub const MSG_VERSION: msgtype = 12;
pub const PANE_LINES_SPACES: pane_lines = 5;
pub const PANE_LINES_NUMBER: pane_lines = 4;
pub const PANE_LINES_SIMPLE: pane_lines = 3;
pub const PANE_LINES_HEAVY: pane_lines = 2;
pub const PANE_LINES_DOUBLE: pane_lines = 1;
pub const PANE_LINES_SINGLE: pane_lines = 0;
pub const PROGRESS_BAR_PAUSED: progress_bar_state = 4;
pub const PROGRESS_BAR_INDETERMINATE: progress_bar_state = 3;
pub const PROGRESS_BAR_ERROR: progress_bar_state = 2;
pub const PROGRESS_BAR_NORMAL: progress_bar_state = 1;
pub const PROGRESS_BAR_HIDDEN: progress_bar_state = 0;
pub const SCREEN_CURSOR_BAR: screen_cursor_style = 3;
pub const SCREEN_CURSOR_UNDERLINE: screen_cursor_style = 2;
pub const SCREEN_CURSOR_BLOCK: screen_cursor_style = 1;
pub const SCREEN_CURSOR_DEFAULT: screen_cursor_style = 0;
pub const STYLE_DEFAULT_SET: style_default_type = 3;
pub const STYLE_DEFAULT_POP: style_default_type = 2;
pub const STYLE_DEFAULT_PUSH: style_default_type = 1;
pub const STYLE_DEFAULT_BASE: style_default_type = 0;
pub const STYLE_RANGE_CONTROL: style_range_type = 7;
pub const STYLE_RANGE_USER: style_range_type = 6;
pub const STYLE_RANGE_SESSION: style_range_type = 5;
pub const STYLE_RANGE_WINDOW: style_range_type = 4;
pub const STYLE_RANGE_PANE: style_range_type = 3;
pub const STYLE_RANGE_RIGHT: style_range_type = 2;
pub const STYLE_RANGE_LEFT: style_range_type = 1;
pub const STYLE_RANGE_NONE: style_range_type = 0;
pub const STYLE_LIST_RIGHT_MARKER: style_list = 4;
pub const STYLE_LIST_LEFT_MARKER: style_list = 3;
pub const STYLE_LIST_FOCUS: style_list = 2;
pub const STYLE_LIST_ON: style_list = 1;
pub const STYLE_LIST_OFF: style_list = 0;
pub const STYLE_ALIGN_ABSOLUTE_CENTRE: style_align = 4;
pub const STYLE_ALIGN_RIGHT: style_align = 3;
pub const STYLE_ALIGN_CENTRE: style_align = 2;
pub const STYLE_ALIGN_LEFT: style_align = 1;
pub const STYLE_ALIGN_DEFAULT: style_align = 0;
pub const THEME_DARK: client_theme = 2;
pub const THEME_LIGHT: client_theme = 1;
pub const THEME_UNKNOWN: client_theme = 0;
pub const LAYOUT_WINDOWPANE: layout_type = 2;
pub const LAYOUT_TOPBOTTOM: layout_type = 1;
pub const LAYOUT_LEFTRIGHT: layout_type = 0;
pub const PROMPT_TYPE_INVALID: prompt_type = 255;
pub const PROMPT_TYPE_WINDOW_TARGET: prompt_type = 3;
pub const PROMPT_TYPE_TARGET: prompt_type = 2;
pub const PROMPT_TYPE_SEARCH: prompt_type = 1;
pub const PROMPT_TYPE_COMMAND: prompt_type = 0;
pub const PROMPT_COMMAND: client_prompt_mode = 1;
pub const PROMPT_ENTRY: client_prompt_mode = 0;
pub const CLIENT_EXIT_DETACH: client_exit_type = 2;
pub const CLIENT_EXIT_SHUTDOWN: client_exit_type = 1;
pub const CLIENT_EXIT_RETURN: client_exit_type = 0;
#[repr(C)]
pub struct cmdq_list {
    /// Whether the queue is part-way through running the item at the front
    /// of its list. The item itself is never held: the running one is always
    /// the front one.
    pub running: bool,
    pub list: cmdq_item_list,
}

/// The items waiting on one queue, front to back, and the owner of each.
/// [`cmdq_remove`] is what gives one up.
pub type cmdq_item_list = ::std::collections::VecDeque<CmdqItemRef>;
/// A run of items a caller has had made but not yet put on a queue, in the
/// order they are to run. [`cmdq_append`] and [`cmdq_insert_after`] are what
/// take one.
pub type cmdq_items = ::std::vec::Vec<CmdqItemRef>;

/// How a walk reads the `at`th item out of a queue.
fn item_at(list: &cmdq_item_list, at: usize) -> Option<*mut cmdq_item> {
    list.get(at).map(CmdqItemRef::as_ptr)
}
/// What an item runs when the queue fires it, and the fields only an item of
/// that type has: a command out of a parsed command list, or a callback with
/// the data it was queued with.
pub(crate) enum CmdqType {
    Command {
        cmdlist: Option<CmdListRef>,
        /// Where the command sits in `cmdlist`, which is what names it: the
        /// item never holds a pointer into the list it shares.
        at: usize,
    },
    Callback {
        cb: CmdqCallbackFn,
        data: CmdqCallbackData,
    },
}

#[repr(C)]
pub struct cmdq_item {
    pub name: Option<::std::ffi::CString>,
    /// The queue the item is waiting on, which owns it: a queue outlives
    /// every item it holds, so this back-pointer cannot go stale.
    pub queue: *mut cmdq_list,
    pub(crate) client: Option<ClientRef>,
    pub(crate) target_client: Option<ClientWeak>,
    pub(crate) type_0: CmdqType,
    pub group: u_int,
    pub number: u_int,
    pub time: time_t,
    pub flags: ::core::ffi::c_int,
    state_ref: Option<CmdqStateRef>,
    pub source: cmd_find_state,
    pub target: cmd_find_state,
}

/// A strong owner of a queue item. The raw pointer from [`CmdqItemRef::as_ptr`]
/// is only a borrowed compatibility view; the handle must remain alive for
/// every use of that pointer.
#[derive(Clone)]
pub struct CmdqItemRef(Rc<UnsafeCell<cmdq_item>>);

/// A non-owning observation of a queue item. A command that answers later
/// holds the item it is to answer this way: the item stays on its queue for
/// as long as it is waiting, and one whose queue has given it up is found as
/// nothing rather than as a freed item.
#[derive(Clone)]
pub struct CmdqItemWeak(Weak<UnsafeCell<cmdq_item>>);

/// Every item a handle has been made for, by the address of the item, which
/// is what a raw view of it is turned back into a handle through.
static CMDQ_ITEM_HANDLES: GlobalTree<usize, CmdqItemWeak> = GlobalTree::new();

/// The item `item` points at, as a handle, or nothing when no live item is
/// there any more.
pub(crate) fn cmdq_item_ref_from_ptr(item: *mut cmdq_item) -> Option<CmdqItemRef> {
    if item.is_null() {
        return None;
    }
    let key = item as usize;
    let reference = CMDQ_ITEM_HANDLES.map().get(&key).cloned()?.upgrade();
    if reference
        .as_ref()
        .is_some_and(|reference| reference.as_ptr() == item)
    {
        return reference;
    }
    CMDQ_ITEM_HANDLES.map().remove(&key);
    None
}

/// The item `item` points at, as a handle, for a caller running on that item
/// and so holding it on its queue for the length of the call.
pub(crate) fn cmdq_item_ref_of(item: *mut cmdq_item) -> CmdqItemRef {
    cmdq_item_ref_from_ptr(item).expect("the running item is on its queue")
}

/// The same as an observation, which is what a command stores while it waits.
pub(crate) fn cmdq_item_weak_from_ptr(item: *mut cmdq_item) -> Option<CmdqItemWeak> {
    cmdq_item_ref_from_ptr(item).map(|reference| reference.downgrade())
}

impl CmdqItemWeak {
    /// Upgrades the observation if the item is still on its queue.
    pub(crate) fn upgrade(&self) -> Option<CmdqItemRef> {
        self.0.upgrade().map(CmdqItemRef)
    }
}

impl Drop for cmdq_item {
    fn drop(&mut self) {
        CMDQ_ITEM_HANDLES
            .map()
            .remove(&(self as *mut cmdq_item as usize));
    }
}

impl CmdqItemRef {
    fn new(value: cmdq_item) -> Self {
        let reference = Self(Rc::new(UnsafeCell::new(value)));
        CMDQ_ITEM_HANDLES
            .map()
            .insert(reference.as_ptr() as usize, reference.downgrade());
        reference
    }

    /// Makes a non-owning observation of this item.
    pub(crate) fn downgrade(&self) -> CmdqItemWeak {
        CmdqItemWeak(Rc::downgrade(&self.0))
    }

    /// Returns a temporary raw view while this strong handle remains alive.
    pub(crate) fn as_ptr(&self) -> *mut cmdq_item {
        self.0.get()
    }

    /// The item this holds. Reaching it this way does not stop anything else
    /// reaching the same item through its raw view.
    #[allow(clippy::mut_from_ref)]
    pub(crate) fn item(&self) -> &mut cmdq_item {
        unsafe { &mut *self.0.get() }
    }
}

/// A fresh item of `type_0` sharing `state` with the rest of its queue.
pub(crate) fn cmdq_item_new(type_0: CmdqType, state: CmdqStateRef) -> CmdqItemRef {
    CmdqItemRef::new(cmdq_item {
        name: None,
        queue: ::core::ptr::null_mut(),
        client: None,
        target_client: None,
        type_0,
        group: 0,
        number: 0,
        time: 0,
        flags: 0,
        state_ref: Some(state),
        source: cmd_find_state::default(),
        target: cmd_find_state::default(),
    })
}

impl cmdq_item {
    /// The state this item shares with the rest of its command list, or null
    /// before one has been given to it.
    pub(crate) fn state(&self) -> *mut cmdq_state {
        self.state_ref
            .as_ref()
            .map_or(::core::ptr::null_mut(), CmdqStateRef::as_ptr)
    }

    /// The command this item runs, or null when it runs a callback instead.
    pub(crate) fn cmd(&self) -> *mut cmd {
        match &self.type_0 {
            CmdqType::Command { cmdlist, at } => match cmdlist {
                Some(cmdlist) => unsafe { cmd_list_at(cmdlist, *at) },
                None => ::core::ptr::null_mut(),
            },
            CmdqType::Callback { .. } => ::core::ptr::null_mut(),
        }
    }

    /// The name the item is logged under.
    pub(crate) fn name_ptr(&self) -> *mut ::core::ffi::c_char {
        cstr_ptr(&self.name)
    }
}
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
#[repr(C)]
pub struct cmdq_state {
    pub flags: ::core::ffi::c_int,
    pub formats: Option<Box<format_tree>>,
    pub event: key_event,
    pub current: cmd_find_state,
}

#[derive(Clone)]
pub(crate) struct CmdqStateRef(Rc<UnsafeCell<cmdq_state>>);

impl CmdqStateRef {
    fn new(value: cmdq_state) -> Self {
        Self(Rc::new(UnsafeCell::new(value)))
    }

    pub(crate) fn as_ptr(&self) -> *mut cmdq_state {
        self.0.get()
    }

    /// The state this holds. Reaching it this way does not stop anything else
    /// reaching the same state through its raw view.
    #[allow(clippy::mut_from_ref)]
    pub(crate) fn state(&self) -> &mut cmdq_state {
        unsafe { &mut *self.0.get() }
    }
}

impl Drop for cmdq_state {
    fn drop(&mut self) {
        drop(self.formats.take());
    }
}
pub type keyc = ::core::ffi::c_ulong;
pub const KEYC_TRIPLECLICK11_CONTROL9: keyc = 51539610386;
pub const KEYC_TRIPLECLICK10_CONTROL9: keyc = 51539610130;
pub const KEYC_TRIPLECLICK9_CONTROL9: keyc = 51539609874;
pub const KEYC_TRIPLECLICK8_CONTROL9: keyc = 51539609618;
pub const KEYC_TRIPLECLICK7_CONTROL9: keyc = 51539609362;
pub const KEYC_TRIPLECLICK6_CONTROL9: keyc = 51539609106;
pub const KEYC_TRIPLECLICK3_CONTROL9: keyc = 51539608338;
pub const KEYC_TRIPLECLICK2_CONTROL9: keyc = 51539608082;
pub const KEYC_TRIPLECLICK1_CONTROL9: keyc = 51539607826;
pub const KEYC_TRIPLECLICK_CONTROL9: keyc = 51539607570;
pub const KEYC_TRIPLECLICK11_CONTROL8: keyc = 51539610385;
pub const KEYC_TRIPLECLICK10_CONTROL8: keyc = 51539610129;
pub const KEYC_TRIPLECLICK9_CONTROL8: keyc = 51539609873;
pub const KEYC_TRIPLECLICK8_CONTROL8: keyc = 51539609617;
pub const KEYC_TRIPLECLICK7_CONTROL8: keyc = 51539609361;
pub const KEYC_TRIPLECLICK6_CONTROL8: keyc = 51539609105;
pub const KEYC_TRIPLECLICK3_CONTROL8: keyc = 51539608337;
pub const KEYC_TRIPLECLICK2_CONTROL8: keyc = 51539608081;
pub const KEYC_TRIPLECLICK1_CONTROL8: keyc = 51539607825;
pub const KEYC_TRIPLECLICK_CONTROL8: keyc = 51539607569;
pub const KEYC_TRIPLECLICK11_CONTROL7: keyc = 51539610384;
pub const KEYC_TRIPLECLICK10_CONTROL7: keyc = 51539610128;
pub const KEYC_TRIPLECLICK9_CONTROL7: keyc = 51539609872;
pub const KEYC_TRIPLECLICK8_CONTROL7: keyc = 51539609616;
pub const KEYC_TRIPLECLICK7_CONTROL7: keyc = 51539609360;
pub const KEYC_TRIPLECLICK6_CONTROL7: keyc = 51539609104;
pub const KEYC_TRIPLECLICK3_CONTROL7: keyc = 51539608336;
pub const KEYC_TRIPLECLICK2_CONTROL7: keyc = 51539608080;
pub const KEYC_TRIPLECLICK1_CONTROL7: keyc = 51539607824;
pub const KEYC_TRIPLECLICK_CONTROL7: keyc = 51539607568;
pub const KEYC_TRIPLECLICK11_CONTROL6: keyc = 51539610383;
pub const KEYC_TRIPLECLICK10_CONTROL6: keyc = 51539610127;
pub const KEYC_TRIPLECLICK9_CONTROL6: keyc = 51539609871;
pub const KEYC_TRIPLECLICK8_CONTROL6: keyc = 51539609615;
pub const KEYC_TRIPLECLICK7_CONTROL6: keyc = 51539609359;
pub const KEYC_TRIPLECLICK6_CONTROL6: keyc = 51539609103;
pub const KEYC_TRIPLECLICK3_CONTROL6: keyc = 51539608335;
pub const KEYC_TRIPLECLICK2_CONTROL6: keyc = 51539608079;
pub const KEYC_TRIPLECLICK1_CONTROL6: keyc = 51539607823;
pub const KEYC_TRIPLECLICK_CONTROL6: keyc = 51539607567;
pub const KEYC_TRIPLECLICK11_CONTROL5: keyc = 51539610382;
pub const KEYC_TRIPLECLICK10_CONTROL5: keyc = 51539610126;
pub const KEYC_TRIPLECLICK9_CONTROL5: keyc = 51539609870;
pub const KEYC_TRIPLECLICK8_CONTROL5: keyc = 51539609614;
pub const KEYC_TRIPLECLICK7_CONTROL5: keyc = 51539609358;
pub const KEYC_TRIPLECLICK6_CONTROL5: keyc = 51539609102;
pub const KEYC_TRIPLECLICK3_CONTROL5: keyc = 51539608334;
pub const KEYC_TRIPLECLICK2_CONTROL5: keyc = 51539608078;
pub const KEYC_TRIPLECLICK1_CONTROL5: keyc = 51539607822;
pub const KEYC_TRIPLECLICK_CONTROL5: keyc = 51539607566;
pub const KEYC_TRIPLECLICK11_CONTROL4: keyc = 51539610381;
pub const KEYC_TRIPLECLICK10_CONTROL4: keyc = 51539610125;
pub const KEYC_TRIPLECLICK9_CONTROL4: keyc = 51539609869;
pub const KEYC_TRIPLECLICK8_CONTROL4: keyc = 51539609613;
pub const KEYC_TRIPLECLICK7_CONTROL4: keyc = 51539609357;
pub const KEYC_TRIPLECLICK6_CONTROL4: keyc = 51539609101;
pub const KEYC_TRIPLECLICK3_CONTROL4: keyc = 51539608333;
pub const KEYC_TRIPLECLICK2_CONTROL4: keyc = 51539608077;
pub const KEYC_TRIPLECLICK1_CONTROL4: keyc = 51539607821;
pub const KEYC_TRIPLECLICK_CONTROL4: keyc = 51539607565;
pub const KEYC_TRIPLECLICK11_CONTROL3: keyc = 51539610380;
pub const KEYC_TRIPLECLICK10_CONTROL3: keyc = 51539610124;
pub const KEYC_TRIPLECLICK9_CONTROL3: keyc = 51539609868;
pub const KEYC_TRIPLECLICK8_CONTROL3: keyc = 51539609612;
pub const KEYC_TRIPLECLICK7_CONTROL3: keyc = 51539609356;
pub const KEYC_TRIPLECLICK6_CONTROL3: keyc = 51539609100;
pub const KEYC_TRIPLECLICK3_CONTROL3: keyc = 51539608332;
pub const KEYC_TRIPLECLICK2_CONTROL3: keyc = 51539608076;
pub const KEYC_TRIPLECLICK1_CONTROL3: keyc = 51539607820;
pub const KEYC_TRIPLECLICK_CONTROL3: keyc = 51539607564;
pub const KEYC_TRIPLECLICK11_CONTROL2: keyc = 51539610379;
pub const KEYC_TRIPLECLICK10_CONTROL2: keyc = 51539610123;
pub const KEYC_TRIPLECLICK9_CONTROL2: keyc = 51539609867;
pub const KEYC_TRIPLECLICK8_CONTROL2: keyc = 51539609611;
pub const KEYC_TRIPLECLICK7_CONTROL2: keyc = 51539609355;
pub const KEYC_TRIPLECLICK6_CONTROL2: keyc = 51539609099;
pub const KEYC_TRIPLECLICK3_CONTROL2: keyc = 51539608331;
pub const KEYC_TRIPLECLICK2_CONTROL2: keyc = 51539608075;
pub const KEYC_TRIPLECLICK1_CONTROL2: keyc = 51539607819;
pub const KEYC_TRIPLECLICK_CONTROL2: keyc = 51539607563;
pub const KEYC_TRIPLECLICK11_CONTROL1: keyc = 51539610378;
pub const KEYC_TRIPLECLICK10_CONTROL1: keyc = 51539610122;
pub const KEYC_TRIPLECLICK9_CONTROL1: keyc = 51539609866;
pub const KEYC_TRIPLECLICK8_CONTROL1: keyc = 51539609610;
pub const KEYC_TRIPLECLICK7_CONTROL1: keyc = 51539609354;
pub const KEYC_TRIPLECLICK6_CONTROL1: keyc = 51539609098;
pub const KEYC_TRIPLECLICK3_CONTROL1: keyc = 51539608330;
pub const KEYC_TRIPLECLICK2_CONTROL1: keyc = 51539608074;
pub const KEYC_TRIPLECLICK1_CONTROL1: keyc = 51539607818;
pub const KEYC_TRIPLECLICK_CONTROL1: keyc = 51539607562;
pub const KEYC_TRIPLECLICK11_CONTROL0: keyc = 51539610377;
pub const KEYC_TRIPLECLICK10_CONTROL0: keyc = 51539610121;
pub const KEYC_TRIPLECLICK9_CONTROL0: keyc = 51539609865;
pub const KEYC_TRIPLECLICK8_CONTROL0: keyc = 51539609609;
pub const KEYC_TRIPLECLICK7_CONTROL0: keyc = 51539609353;
pub const KEYC_TRIPLECLICK6_CONTROL0: keyc = 51539609097;
pub const KEYC_TRIPLECLICK3_CONTROL0: keyc = 51539608329;
pub const KEYC_TRIPLECLICK2_CONTROL0: keyc = 51539608073;
pub const KEYC_TRIPLECLICK1_CONTROL0: keyc = 51539607817;
pub const KEYC_TRIPLECLICK_CONTROL0: keyc = 51539607561;
pub const KEYC_TRIPLECLICK11_SCROLLBAR_DOWN: keyc = 51539610376;
pub const KEYC_TRIPLECLICK10_SCROLLBAR_DOWN: keyc = 51539610120;
pub const KEYC_TRIPLECLICK9_SCROLLBAR_DOWN: keyc = 51539609864;
pub const KEYC_TRIPLECLICK8_SCROLLBAR_DOWN: keyc = 51539609608;
pub const KEYC_TRIPLECLICK7_SCROLLBAR_DOWN: keyc = 51539609352;
pub const KEYC_TRIPLECLICK6_SCROLLBAR_DOWN: keyc = 51539609096;
pub const KEYC_TRIPLECLICK3_SCROLLBAR_DOWN: keyc = 51539608328;
pub const KEYC_TRIPLECLICK2_SCROLLBAR_DOWN: keyc = 51539608072;
pub const KEYC_TRIPLECLICK1_SCROLLBAR_DOWN: keyc = 51539607816;
pub const KEYC_TRIPLECLICK_SCROLLBAR_DOWN: keyc = 51539607560;
pub const KEYC_TRIPLECLICK11_SCROLLBAR_SLIDER: keyc = 51539610375;
pub const KEYC_TRIPLECLICK10_SCROLLBAR_SLIDER: keyc = 51539610119;
pub const KEYC_TRIPLECLICK9_SCROLLBAR_SLIDER: keyc = 51539609863;
pub const KEYC_TRIPLECLICK8_SCROLLBAR_SLIDER: keyc = 51539609607;
pub const KEYC_TRIPLECLICK7_SCROLLBAR_SLIDER: keyc = 51539609351;
pub const KEYC_TRIPLECLICK6_SCROLLBAR_SLIDER: keyc = 51539609095;
pub const KEYC_TRIPLECLICK3_SCROLLBAR_SLIDER: keyc = 51539608327;
pub const KEYC_TRIPLECLICK2_SCROLLBAR_SLIDER: keyc = 51539608071;
pub const KEYC_TRIPLECLICK1_SCROLLBAR_SLIDER: keyc = 51539607815;
pub const KEYC_TRIPLECLICK_SCROLLBAR_SLIDER: keyc = 51539607559;
pub const KEYC_TRIPLECLICK11_SCROLLBAR_UP: keyc = 51539610374;
pub const KEYC_TRIPLECLICK10_SCROLLBAR_UP: keyc = 51539610118;
pub const KEYC_TRIPLECLICK9_SCROLLBAR_UP: keyc = 51539609862;
pub const KEYC_TRIPLECLICK8_SCROLLBAR_UP: keyc = 51539609606;
pub const KEYC_TRIPLECLICK7_SCROLLBAR_UP: keyc = 51539609350;
pub const KEYC_TRIPLECLICK6_SCROLLBAR_UP: keyc = 51539609094;
pub const KEYC_TRIPLECLICK3_SCROLLBAR_UP: keyc = 51539608326;
pub const KEYC_TRIPLECLICK2_SCROLLBAR_UP: keyc = 51539608070;
pub const KEYC_TRIPLECLICK1_SCROLLBAR_UP: keyc = 51539607814;
pub const KEYC_TRIPLECLICK_SCROLLBAR_UP: keyc = 51539607558;
pub const KEYC_TRIPLECLICK11_BORDER: keyc = 51539610373;
pub const KEYC_TRIPLECLICK10_BORDER: keyc = 51539610117;
pub const KEYC_TRIPLECLICK9_BORDER: keyc = 51539609861;
pub const KEYC_TRIPLECLICK8_BORDER: keyc = 51539609605;
pub const KEYC_TRIPLECLICK7_BORDER: keyc = 51539609349;
pub const KEYC_TRIPLECLICK6_BORDER: keyc = 51539609093;
pub const KEYC_TRIPLECLICK3_BORDER: keyc = 51539608325;
pub const KEYC_TRIPLECLICK2_BORDER: keyc = 51539608069;
pub const KEYC_TRIPLECLICK1_BORDER: keyc = 51539607813;
pub const KEYC_TRIPLECLICK_BORDER: keyc = 51539607557;
pub const KEYC_TRIPLECLICK11_STATUS_DEFAULT: keyc = 51539610372;
pub const KEYC_TRIPLECLICK10_STATUS_DEFAULT: keyc = 51539610116;
pub const KEYC_TRIPLECLICK9_STATUS_DEFAULT: keyc = 51539609860;
pub const KEYC_TRIPLECLICK8_STATUS_DEFAULT: keyc = 51539609604;
pub const KEYC_TRIPLECLICK7_STATUS_DEFAULT: keyc = 51539609348;
pub const KEYC_TRIPLECLICK6_STATUS_DEFAULT: keyc = 51539609092;
pub const KEYC_TRIPLECLICK3_STATUS_DEFAULT: keyc = 51539608324;
pub const KEYC_TRIPLECLICK2_STATUS_DEFAULT: keyc = 51539608068;
pub const KEYC_TRIPLECLICK1_STATUS_DEFAULT: keyc = 51539607812;
pub const KEYC_TRIPLECLICK_STATUS_DEFAULT: keyc = 51539607556;
pub const KEYC_TRIPLECLICK11_STATUS_RIGHT: keyc = 51539610371;
pub const KEYC_TRIPLECLICK10_STATUS_RIGHT: keyc = 51539610115;
pub const KEYC_TRIPLECLICK9_STATUS_RIGHT: keyc = 51539609859;
pub const KEYC_TRIPLECLICK8_STATUS_RIGHT: keyc = 51539609603;
pub const KEYC_TRIPLECLICK7_STATUS_RIGHT: keyc = 51539609347;
pub const KEYC_TRIPLECLICK6_STATUS_RIGHT: keyc = 51539609091;
pub const KEYC_TRIPLECLICK3_STATUS_RIGHT: keyc = 51539608323;
pub const KEYC_TRIPLECLICK2_STATUS_RIGHT: keyc = 51539608067;
pub const KEYC_TRIPLECLICK1_STATUS_RIGHT: keyc = 51539607811;
pub const KEYC_TRIPLECLICK_STATUS_RIGHT: keyc = 51539607555;
pub const KEYC_TRIPLECLICK11_STATUS_LEFT: keyc = 51539610370;
pub const KEYC_TRIPLECLICK10_STATUS_LEFT: keyc = 51539610114;
pub const KEYC_TRIPLECLICK9_STATUS_LEFT: keyc = 51539609858;
pub const KEYC_TRIPLECLICK8_STATUS_LEFT: keyc = 51539609602;
pub const KEYC_TRIPLECLICK7_STATUS_LEFT: keyc = 51539609346;
pub const KEYC_TRIPLECLICK6_STATUS_LEFT: keyc = 51539609090;
pub const KEYC_TRIPLECLICK3_STATUS_LEFT: keyc = 51539608322;
pub const KEYC_TRIPLECLICK2_STATUS_LEFT: keyc = 51539608066;
pub const KEYC_TRIPLECLICK1_STATUS_LEFT: keyc = 51539607810;
pub const KEYC_TRIPLECLICK_STATUS_LEFT: keyc = 51539607554;
pub const KEYC_TRIPLECLICK11_STATUS: keyc = 51539610369;
pub const KEYC_TRIPLECLICK10_STATUS: keyc = 51539610113;
pub const KEYC_TRIPLECLICK9_STATUS: keyc = 51539609857;
pub const KEYC_TRIPLECLICK8_STATUS: keyc = 51539609601;
pub const KEYC_TRIPLECLICK7_STATUS: keyc = 51539609345;
pub const KEYC_TRIPLECLICK6_STATUS: keyc = 51539609089;
pub const KEYC_TRIPLECLICK3_STATUS: keyc = 51539608321;
pub const KEYC_TRIPLECLICK2_STATUS: keyc = 51539608065;
pub const KEYC_TRIPLECLICK1_STATUS: keyc = 51539607809;
pub const KEYC_TRIPLECLICK_STATUS: keyc = 51539607553;
pub const KEYC_TRIPLECLICK11_PANE: keyc = 51539610368;
pub const KEYC_TRIPLECLICK10_PANE: keyc = 51539610112;
pub const KEYC_TRIPLECLICK9_PANE: keyc = 51539609856;
pub const KEYC_TRIPLECLICK8_PANE: keyc = 51539609600;
pub const KEYC_TRIPLECLICK7_PANE: keyc = 51539609344;
pub const KEYC_TRIPLECLICK6_PANE: keyc = 51539609088;
pub const KEYC_TRIPLECLICK3_PANE: keyc = 51539608320;
pub const KEYC_TRIPLECLICK2_PANE: keyc = 51539608064;
pub const KEYC_TRIPLECLICK1_PANE: keyc = 51539607808;
pub const KEYC_TRIPLECLICK_PANE: keyc = 51539607552;
pub const KEYC_DOUBLECLICK11_CONTROL9: keyc = 47244643090;
pub const KEYC_DOUBLECLICK10_CONTROL9: keyc = 47244642834;
pub const KEYC_DOUBLECLICK9_CONTROL9: keyc = 47244642578;
pub const KEYC_DOUBLECLICK8_CONTROL9: keyc = 47244642322;
pub const KEYC_DOUBLECLICK7_CONTROL9: keyc = 47244642066;
pub const KEYC_DOUBLECLICK6_CONTROL9: keyc = 47244641810;
pub const KEYC_DOUBLECLICK3_CONTROL9: keyc = 47244641042;
pub const KEYC_DOUBLECLICK2_CONTROL9: keyc = 47244640786;
pub const KEYC_DOUBLECLICK1_CONTROL9: keyc = 47244640530;
pub const KEYC_DOUBLECLICK_CONTROL9: keyc = 47244640274;
pub const KEYC_DOUBLECLICK11_CONTROL8: keyc = 47244643089;
pub const KEYC_DOUBLECLICK10_CONTROL8: keyc = 47244642833;
pub const KEYC_DOUBLECLICK9_CONTROL8: keyc = 47244642577;
pub const KEYC_DOUBLECLICK8_CONTROL8: keyc = 47244642321;
pub const KEYC_DOUBLECLICK7_CONTROL8: keyc = 47244642065;
pub const KEYC_DOUBLECLICK6_CONTROL8: keyc = 47244641809;
pub const KEYC_DOUBLECLICK3_CONTROL8: keyc = 47244641041;
pub const KEYC_DOUBLECLICK2_CONTROL8: keyc = 47244640785;
pub const KEYC_DOUBLECLICK1_CONTROL8: keyc = 47244640529;
pub const KEYC_DOUBLECLICK_CONTROL8: keyc = 47244640273;
pub const KEYC_DOUBLECLICK11_CONTROL7: keyc = 47244643088;
pub const KEYC_DOUBLECLICK10_CONTROL7: keyc = 47244642832;
pub const KEYC_DOUBLECLICK9_CONTROL7: keyc = 47244642576;
pub const KEYC_DOUBLECLICK8_CONTROL7: keyc = 47244642320;
pub const KEYC_DOUBLECLICK7_CONTROL7: keyc = 47244642064;
pub const KEYC_DOUBLECLICK6_CONTROL7: keyc = 47244641808;
pub const KEYC_DOUBLECLICK3_CONTROL7: keyc = 47244641040;
pub const KEYC_DOUBLECLICK2_CONTROL7: keyc = 47244640784;
pub const KEYC_DOUBLECLICK1_CONTROL7: keyc = 47244640528;
pub const KEYC_DOUBLECLICK_CONTROL7: keyc = 47244640272;
pub const KEYC_DOUBLECLICK11_CONTROL6: keyc = 47244643087;
pub const KEYC_DOUBLECLICK10_CONTROL6: keyc = 47244642831;
pub const KEYC_DOUBLECLICK9_CONTROL6: keyc = 47244642575;
pub const KEYC_DOUBLECLICK8_CONTROL6: keyc = 47244642319;
pub const KEYC_DOUBLECLICK7_CONTROL6: keyc = 47244642063;
pub const KEYC_DOUBLECLICK6_CONTROL6: keyc = 47244641807;
pub const KEYC_DOUBLECLICK3_CONTROL6: keyc = 47244641039;
pub const KEYC_DOUBLECLICK2_CONTROL6: keyc = 47244640783;
pub const KEYC_DOUBLECLICK1_CONTROL6: keyc = 47244640527;
pub const KEYC_DOUBLECLICK_CONTROL6: keyc = 47244640271;
pub const KEYC_DOUBLECLICK11_CONTROL5: keyc = 47244643086;
pub const KEYC_DOUBLECLICK10_CONTROL5: keyc = 47244642830;
pub const KEYC_DOUBLECLICK9_CONTROL5: keyc = 47244642574;
pub const KEYC_DOUBLECLICK8_CONTROL5: keyc = 47244642318;
pub const KEYC_DOUBLECLICK7_CONTROL5: keyc = 47244642062;
pub const KEYC_DOUBLECLICK6_CONTROL5: keyc = 47244641806;
pub const KEYC_DOUBLECLICK3_CONTROL5: keyc = 47244641038;
pub const KEYC_DOUBLECLICK2_CONTROL5: keyc = 47244640782;
pub const KEYC_DOUBLECLICK1_CONTROL5: keyc = 47244640526;
pub const KEYC_DOUBLECLICK_CONTROL5: keyc = 47244640270;
pub const KEYC_DOUBLECLICK11_CONTROL4: keyc = 47244643085;
pub const KEYC_DOUBLECLICK10_CONTROL4: keyc = 47244642829;
pub const KEYC_DOUBLECLICK9_CONTROL4: keyc = 47244642573;
pub const KEYC_DOUBLECLICK8_CONTROL4: keyc = 47244642317;
pub const KEYC_DOUBLECLICK7_CONTROL4: keyc = 47244642061;
pub const KEYC_DOUBLECLICK6_CONTROL4: keyc = 47244641805;
pub const KEYC_DOUBLECLICK3_CONTROL4: keyc = 47244641037;
pub const KEYC_DOUBLECLICK2_CONTROL4: keyc = 47244640781;
pub const KEYC_DOUBLECLICK1_CONTROL4: keyc = 47244640525;
pub const KEYC_DOUBLECLICK_CONTROL4: keyc = 47244640269;
pub const KEYC_DOUBLECLICK11_CONTROL3: keyc = 47244643084;
pub const KEYC_DOUBLECLICK10_CONTROL3: keyc = 47244642828;
pub const KEYC_DOUBLECLICK9_CONTROL3: keyc = 47244642572;
pub const KEYC_DOUBLECLICK8_CONTROL3: keyc = 47244642316;
pub const KEYC_DOUBLECLICK7_CONTROL3: keyc = 47244642060;
pub const KEYC_DOUBLECLICK6_CONTROL3: keyc = 47244641804;
pub const KEYC_DOUBLECLICK3_CONTROL3: keyc = 47244641036;
pub const KEYC_DOUBLECLICK2_CONTROL3: keyc = 47244640780;
pub const KEYC_DOUBLECLICK1_CONTROL3: keyc = 47244640524;
pub const KEYC_DOUBLECLICK_CONTROL3: keyc = 47244640268;
pub const KEYC_DOUBLECLICK11_CONTROL2: keyc = 47244643083;
pub const KEYC_DOUBLECLICK10_CONTROL2: keyc = 47244642827;
pub const KEYC_DOUBLECLICK9_CONTROL2: keyc = 47244642571;
pub const KEYC_DOUBLECLICK8_CONTROL2: keyc = 47244642315;
pub const KEYC_DOUBLECLICK7_CONTROL2: keyc = 47244642059;
pub const KEYC_DOUBLECLICK6_CONTROL2: keyc = 47244641803;
pub const KEYC_DOUBLECLICK3_CONTROL2: keyc = 47244641035;
pub const KEYC_DOUBLECLICK2_CONTROL2: keyc = 47244640779;
pub const KEYC_DOUBLECLICK1_CONTROL2: keyc = 47244640523;
pub const KEYC_DOUBLECLICK_CONTROL2: keyc = 47244640267;
pub const KEYC_DOUBLECLICK11_CONTROL1: keyc = 47244643082;
pub const KEYC_DOUBLECLICK10_CONTROL1: keyc = 47244642826;
pub const KEYC_DOUBLECLICK9_CONTROL1: keyc = 47244642570;
pub const KEYC_DOUBLECLICK8_CONTROL1: keyc = 47244642314;
pub const KEYC_DOUBLECLICK7_CONTROL1: keyc = 47244642058;
pub const KEYC_DOUBLECLICK6_CONTROL1: keyc = 47244641802;
pub const KEYC_DOUBLECLICK3_CONTROL1: keyc = 47244641034;
pub const KEYC_DOUBLECLICK2_CONTROL1: keyc = 47244640778;
pub const KEYC_DOUBLECLICK1_CONTROL1: keyc = 47244640522;
pub const KEYC_DOUBLECLICK_CONTROL1: keyc = 47244640266;
pub const KEYC_DOUBLECLICK11_CONTROL0: keyc = 47244643081;
pub const KEYC_DOUBLECLICK10_CONTROL0: keyc = 47244642825;
pub const KEYC_DOUBLECLICK9_CONTROL0: keyc = 47244642569;
pub const KEYC_DOUBLECLICK8_CONTROL0: keyc = 47244642313;
pub const KEYC_DOUBLECLICK7_CONTROL0: keyc = 47244642057;
pub const KEYC_DOUBLECLICK6_CONTROL0: keyc = 47244641801;
pub const KEYC_DOUBLECLICK3_CONTROL0: keyc = 47244641033;
pub const KEYC_DOUBLECLICK2_CONTROL0: keyc = 47244640777;
pub const KEYC_DOUBLECLICK1_CONTROL0: keyc = 47244640521;
pub const KEYC_DOUBLECLICK_CONTROL0: keyc = 47244640265;
pub const KEYC_DOUBLECLICK11_SCROLLBAR_DOWN: keyc = 47244643080;
pub const KEYC_DOUBLECLICK10_SCROLLBAR_DOWN: keyc = 47244642824;
pub const KEYC_DOUBLECLICK9_SCROLLBAR_DOWN: keyc = 47244642568;
pub const KEYC_DOUBLECLICK8_SCROLLBAR_DOWN: keyc = 47244642312;
pub const KEYC_DOUBLECLICK7_SCROLLBAR_DOWN: keyc = 47244642056;
pub const KEYC_DOUBLECLICK6_SCROLLBAR_DOWN: keyc = 47244641800;
pub const KEYC_DOUBLECLICK3_SCROLLBAR_DOWN: keyc = 47244641032;
pub const KEYC_DOUBLECLICK2_SCROLLBAR_DOWN: keyc = 47244640776;
pub const KEYC_DOUBLECLICK1_SCROLLBAR_DOWN: keyc = 47244640520;
pub const KEYC_DOUBLECLICK_SCROLLBAR_DOWN: keyc = 47244640264;
pub const KEYC_DOUBLECLICK11_SCROLLBAR_SLIDER: keyc = 47244643079;
pub const KEYC_DOUBLECLICK10_SCROLLBAR_SLIDER: keyc = 47244642823;
pub const KEYC_DOUBLECLICK9_SCROLLBAR_SLIDER: keyc = 47244642567;
pub const KEYC_DOUBLECLICK8_SCROLLBAR_SLIDER: keyc = 47244642311;
pub const KEYC_DOUBLECLICK7_SCROLLBAR_SLIDER: keyc = 47244642055;
pub const KEYC_DOUBLECLICK6_SCROLLBAR_SLIDER: keyc = 47244641799;
pub const KEYC_DOUBLECLICK3_SCROLLBAR_SLIDER: keyc = 47244641031;
pub const KEYC_DOUBLECLICK2_SCROLLBAR_SLIDER: keyc = 47244640775;
pub const KEYC_DOUBLECLICK1_SCROLLBAR_SLIDER: keyc = 47244640519;
pub const KEYC_DOUBLECLICK_SCROLLBAR_SLIDER: keyc = 47244640263;
pub const KEYC_DOUBLECLICK11_SCROLLBAR_UP: keyc = 47244643078;
pub const KEYC_DOUBLECLICK10_SCROLLBAR_UP: keyc = 47244642822;
pub const KEYC_DOUBLECLICK9_SCROLLBAR_UP: keyc = 47244642566;
pub const KEYC_DOUBLECLICK8_SCROLLBAR_UP: keyc = 47244642310;
pub const KEYC_DOUBLECLICK7_SCROLLBAR_UP: keyc = 47244642054;
pub const KEYC_DOUBLECLICK6_SCROLLBAR_UP: keyc = 47244641798;
pub const KEYC_DOUBLECLICK3_SCROLLBAR_UP: keyc = 47244641030;
pub const KEYC_DOUBLECLICK2_SCROLLBAR_UP: keyc = 47244640774;
pub const KEYC_DOUBLECLICK1_SCROLLBAR_UP: keyc = 47244640518;
pub const KEYC_DOUBLECLICK_SCROLLBAR_UP: keyc = 47244640262;
pub const KEYC_DOUBLECLICK11_BORDER: keyc = 47244643077;
pub const KEYC_DOUBLECLICK10_BORDER: keyc = 47244642821;
pub const KEYC_DOUBLECLICK9_BORDER: keyc = 47244642565;
pub const KEYC_DOUBLECLICK8_BORDER: keyc = 47244642309;
pub const KEYC_DOUBLECLICK7_BORDER: keyc = 47244642053;
pub const KEYC_DOUBLECLICK6_BORDER: keyc = 47244641797;
pub const KEYC_DOUBLECLICK3_BORDER: keyc = 47244641029;
pub const KEYC_DOUBLECLICK2_BORDER: keyc = 47244640773;
pub const KEYC_DOUBLECLICK1_BORDER: keyc = 47244640517;
pub const KEYC_DOUBLECLICK_BORDER: keyc = 47244640261;
pub const KEYC_DOUBLECLICK11_STATUS_DEFAULT: keyc = 47244643076;
pub const KEYC_DOUBLECLICK10_STATUS_DEFAULT: keyc = 47244642820;
pub const KEYC_DOUBLECLICK9_STATUS_DEFAULT: keyc = 47244642564;
pub const KEYC_DOUBLECLICK8_STATUS_DEFAULT: keyc = 47244642308;
pub const KEYC_DOUBLECLICK7_STATUS_DEFAULT: keyc = 47244642052;
pub const KEYC_DOUBLECLICK6_STATUS_DEFAULT: keyc = 47244641796;
pub const KEYC_DOUBLECLICK3_STATUS_DEFAULT: keyc = 47244641028;
pub const KEYC_DOUBLECLICK2_STATUS_DEFAULT: keyc = 47244640772;
pub const KEYC_DOUBLECLICK1_STATUS_DEFAULT: keyc = 47244640516;
pub const KEYC_DOUBLECLICK_STATUS_DEFAULT: keyc = 47244640260;
pub const KEYC_DOUBLECLICK11_STATUS_RIGHT: keyc = 47244643075;
pub const KEYC_DOUBLECLICK10_STATUS_RIGHT: keyc = 47244642819;
pub const KEYC_DOUBLECLICK9_STATUS_RIGHT: keyc = 47244642563;
pub const KEYC_DOUBLECLICK8_STATUS_RIGHT: keyc = 47244642307;
pub const KEYC_DOUBLECLICK7_STATUS_RIGHT: keyc = 47244642051;
pub const KEYC_DOUBLECLICK6_STATUS_RIGHT: keyc = 47244641795;
pub const KEYC_DOUBLECLICK3_STATUS_RIGHT: keyc = 47244641027;
pub const KEYC_DOUBLECLICK2_STATUS_RIGHT: keyc = 47244640771;
pub const KEYC_DOUBLECLICK1_STATUS_RIGHT: keyc = 47244640515;
pub const KEYC_DOUBLECLICK_STATUS_RIGHT: keyc = 47244640259;
pub const KEYC_DOUBLECLICK11_STATUS_LEFT: keyc = 47244643074;
pub const KEYC_DOUBLECLICK10_STATUS_LEFT: keyc = 47244642818;
pub const KEYC_DOUBLECLICK9_STATUS_LEFT: keyc = 47244642562;
pub const KEYC_DOUBLECLICK8_STATUS_LEFT: keyc = 47244642306;
pub const KEYC_DOUBLECLICK7_STATUS_LEFT: keyc = 47244642050;
pub const KEYC_DOUBLECLICK6_STATUS_LEFT: keyc = 47244641794;
pub const KEYC_DOUBLECLICK3_STATUS_LEFT: keyc = 47244641026;
pub const KEYC_DOUBLECLICK2_STATUS_LEFT: keyc = 47244640770;
pub const KEYC_DOUBLECLICK1_STATUS_LEFT: keyc = 47244640514;
pub const KEYC_DOUBLECLICK_STATUS_LEFT: keyc = 47244640258;
pub const KEYC_DOUBLECLICK11_STATUS: keyc = 47244643073;
pub const KEYC_DOUBLECLICK10_STATUS: keyc = 47244642817;
pub const KEYC_DOUBLECLICK9_STATUS: keyc = 47244642561;
pub const KEYC_DOUBLECLICK8_STATUS: keyc = 47244642305;
pub const KEYC_DOUBLECLICK7_STATUS: keyc = 47244642049;
pub const KEYC_DOUBLECLICK6_STATUS: keyc = 47244641793;
pub const KEYC_DOUBLECLICK3_STATUS: keyc = 47244641025;
pub const KEYC_DOUBLECLICK2_STATUS: keyc = 47244640769;
pub const KEYC_DOUBLECLICK1_STATUS: keyc = 47244640513;
pub const KEYC_DOUBLECLICK_STATUS: keyc = 47244640257;
pub const KEYC_DOUBLECLICK11_PANE: keyc = 47244643072;
pub const KEYC_DOUBLECLICK10_PANE: keyc = 47244642816;
pub const KEYC_DOUBLECLICK9_PANE: keyc = 47244642560;
pub const KEYC_DOUBLECLICK8_PANE: keyc = 47244642304;
pub const KEYC_DOUBLECLICK7_PANE: keyc = 47244642048;
pub const KEYC_DOUBLECLICK6_PANE: keyc = 47244641792;
pub const KEYC_DOUBLECLICK3_PANE: keyc = 47244641024;
pub const KEYC_DOUBLECLICK2_PANE: keyc = 47244640768;
pub const KEYC_DOUBLECLICK1_PANE: keyc = 47244640512;
pub const KEYC_DOUBLECLICK_PANE: keyc = 47244640256;
pub const KEYC_SECONDCLICK11_CONTROL9: keyc = 42949675794;
pub const KEYC_SECONDCLICK10_CONTROL9: keyc = 42949675538;
pub const KEYC_SECONDCLICK9_CONTROL9: keyc = 42949675282;
pub const KEYC_SECONDCLICK8_CONTROL9: keyc = 42949675026;
pub const KEYC_SECONDCLICK7_CONTROL9: keyc = 42949674770;
pub const KEYC_SECONDCLICK6_CONTROL9: keyc = 42949674514;
pub const KEYC_SECONDCLICK3_CONTROL9: keyc = 42949673746;
pub const KEYC_SECONDCLICK2_CONTROL9: keyc = 42949673490;
pub const KEYC_SECONDCLICK1_CONTROL9: keyc = 42949673234;
pub const KEYC_SECONDCLICK_CONTROL9: keyc = 42949672978;
pub const KEYC_SECONDCLICK11_CONTROL8: keyc = 42949675793;
pub const KEYC_SECONDCLICK10_CONTROL8: keyc = 42949675537;
pub const KEYC_SECONDCLICK9_CONTROL8: keyc = 42949675281;
pub const KEYC_SECONDCLICK8_CONTROL8: keyc = 42949675025;
pub const KEYC_SECONDCLICK7_CONTROL8: keyc = 42949674769;
pub const KEYC_SECONDCLICK6_CONTROL8: keyc = 42949674513;
pub const KEYC_SECONDCLICK3_CONTROL8: keyc = 42949673745;
pub const KEYC_SECONDCLICK2_CONTROL8: keyc = 42949673489;
pub const KEYC_SECONDCLICK1_CONTROL8: keyc = 42949673233;
pub const KEYC_SECONDCLICK_CONTROL8: keyc = 42949672977;
pub const KEYC_SECONDCLICK11_CONTROL7: keyc = 42949675792;
pub const KEYC_SECONDCLICK10_CONTROL7: keyc = 42949675536;
pub const KEYC_SECONDCLICK9_CONTROL7: keyc = 42949675280;
pub const KEYC_SECONDCLICK8_CONTROL7: keyc = 42949675024;
pub const KEYC_SECONDCLICK7_CONTROL7: keyc = 42949674768;
pub const KEYC_SECONDCLICK6_CONTROL7: keyc = 42949674512;
pub const KEYC_SECONDCLICK3_CONTROL7: keyc = 42949673744;
pub const KEYC_SECONDCLICK2_CONTROL7: keyc = 42949673488;
pub const KEYC_SECONDCLICK1_CONTROL7: keyc = 42949673232;
pub const KEYC_SECONDCLICK_CONTROL7: keyc = 42949672976;
pub const KEYC_SECONDCLICK11_CONTROL6: keyc = 42949675791;
pub const KEYC_SECONDCLICK10_CONTROL6: keyc = 42949675535;
pub const KEYC_SECONDCLICK9_CONTROL6: keyc = 42949675279;
pub const KEYC_SECONDCLICK8_CONTROL6: keyc = 42949675023;
pub const KEYC_SECONDCLICK7_CONTROL6: keyc = 42949674767;
pub const KEYC_SECONDCLICK6_CONTROL6: keyc = 42949674511;
pub const KEYC_SECONDCLICK3_CONTROL6: keyc = 42949673743;
pub const KEYC_SECONDCLICK2_CONTROL6: keyc = 42949673487;
pub const KEYC_SECONDCLICK1_CONTROL6: keyc = 42949673231;
pub const KEYC_SECONDCLICK_CONTROL6: keyc = 42949672975;
pub const KEYC_SECONDCLICK11_CONTROL5: keyc = 42949675790;
pub const KEYC_SECONDCLICK10_CONTROL5: keyc = 42949675534;
pub const KEYC_SECONDCLICK9_CONTROL5: keyc = 42949675278;
pub const KEYC_SECONDCLICK8_CONTROL5: keyc = 42949675022;
pub const KEYC_SECONDCLICK7_CONTROL5: keyc = 42949674766;
pub const KEYC_SECONDCLICK6_CONTROL5: keyc = 42949674510;
pub const KEYC_SECONDCLICK3_CONTROL5: keyc = 42949673742;
pub const KEYC_SECONDCLICK2_CONTROL5: keyc = 42949673486;
pub const KEYC_SECONDCLICK1_CONTROL5: keyc = 42949673230;
pub const KEYC_SECONDCLICK_CONTROL5: keyc = 42949672974;
pub const KEYC_SECONDCLICK11_CONTROL4: keyc = 42949675789;
pub const KEYC_SECONDCLICK10_CONTROL4: keyc = 42949675533;
pub const KEYC_SECONDCLICK9_CONTROL4: keyc = 42949675277;
pub const KEYC_SECONDCLICK8_CONTROL4: keyc = 42949675021;
pub const KEYC_SECONDCLICK7_CONTROL4: keyc = 42949674765;
pub const KEYC_SECONDCLICK6_CONTROL4: keyc = 42949674509;
pub const KEYC_SECONDCLICK3_CONTROL4: keyc = 42949673741;
pub const KEYC_SECONDCLICK2_CONTROL4: keyc = 42949673485;
pub const KEYC_SECONDCLICK1_CONTROL4: keyc = 42949673229;
pub const KEYC_SECONDCLICK_CONTROL4: keyc = 42949672973;
pub const KEYC_SECONDCLICK11_CONTROL3: keyc = 42949675788;
pub const KEYC_SECONDCLICK10_CONTROL3: keyc = 42949675532;
pub const KEYC_SECONDCLICK9_CONTROL3: keyc = 42949675276;
pub const KEYC_SECONDCLICK8_CONTROL3: keyc = 42949675020;
pub const KEYC_SECONDCLICK7_CONTROL3: keyc = 42949674764;
pub const KEYC_SECONDCLICK6_CONTROL3: keyc = 42949674508;
pub const KEYC_SECONDCLICK3_CONTROL3: keyc = 42949673740;
pub const KEYC_SECONDCLICK2_CONTROL3: keyc = 42949673484;
pub const KEYC_SECONDCLICK1_CONTROL3: keyc = 42949673228;
pub const KEYC_SECONDCLICK_CONTROL3: keyc = 42949672972;
pub const KEYC_SECONDCLICK11_CONTROL2: keyc = 42949675787;
pub const KEYC_SECONDCLICK10_CONTROL2: keyc = 42949675531;
pub const KEYC_SECONDCLICK9_CONTROL2: keyc = 42949675275;
pub const KEYC_SECONDCLICK8_CONTROL2: keyc = 42949675019;
pub const KEYC_SECONDCLICK7_CONTROL2: keyc = 42949674763;
pub const KEYC_SECONDCLICK6_CONTROL2: keyc = 42949674507;
pub const KEYC_SECONDCLICK3_CONTROL2: keyc = 42949673739;
pub const KEYC_SECONDCLICK2_CONTROL2: keyc = 42949673483;
pub const KEYC_SECONDCLICK1_CONTROL2: keyc = 42949673227;
pub const KEYC_SECONDCLICK_CONTROL2: keyc = 42949672971;
pub const KEYC_SECONDCLICK11_CONTROL1: keyc = 42949675786;
pub const KEYC_SECONDCLICK10_CONTROL1: keyc = 42949675530;
pub const KEYC_SECONDCLICK9_CONTROL1: keyc = 42949675274;
pub const KEYC_SECONDCLICK8_CONTROL1: keyc = 42949675018;
pub const KEYC_SECONDCLICK7_CONTROL1: keyc = 42949674762;
pub const KEYC_SECONDCLICK6_CONTROL1: keyc = 42949674506;
pub const KEYC_SECONDCLICK3_CONTROL1: keyc = 42949673738;
pub const KEYC_SECONDCLICK2_CONTROL1: keyc = 42949673482;
pub const KEYC_SECONDCLICK1_CONTROL1: keyc = 42949673226;
pub const KEYC_SECONDCLICK_CONTROL1: keyc = 42949672970;
pub const KEYC_SECONDCLICK11_CONTROL0: keyc = 42949675785;
pub const KEYC_SECONDCLICK10_CONTROL0: keyc = 42949675529;
pub const KEYC_SECONDCLICK9_CONTROL0: keyc = 42949675273;
pub const KEYC_SECONDCLICK8_CONTROL0: keyc = 42949675017;
pub const KEYC_SECONDCLICK7_CONTROL0: keyc = 42949674761;
pub const KEYC_SECONDCLICK6_CONTROL0: keyc = 42949674505;
pub const KEYC_SECONDCLICK3_CONTROL0: keyc = 42949673737;
pub const KEYC_SECONDCLICK2_CONTROL0: keyc = 42949673481;
pub const KEYC_SECONDCLICK1_CONTROL0: keyc = 42949673225;
pub const KEYC_SECONDCLICK_CONTROL0: keyc = 42949672969;
pub const KEYC_SECONDCLICK11_SCROLLBAR_DOWN: keyc = 42949675784;
pub const KEYC_SECONDCLICK10_SCROLLBAR_DOWN: keyc = 42949675528;
pub const KEYC_SECONDCLICK9_SCROLLBAR_DOWN: keyc = 42949675272;
pub const KEYC_SECONDCLICK8_SCROLLBAR_DOWN: keyc = 42949675016;
pub const KEYC_SECONDCLICK7_SCROLLBAR_DOWN: keyc = 42949674760;
pub const KEYC_SECONDCLICK6_SCROLLBAR_DOWN: keyc = 42949674504;
pub const KEYC_SECONDCLICK3_SCROLLBAR_DOWN: keyc = 42949673736;
pub const KEYC_SECONDCLICK2_SCROLLBAR_DOWN: keyc = 42949673480;
pub const KEYC_SECONDCLICK1_SCROLLBAR_DOWN: keyc = 42949673224;
pub const KEYC_SECONDCLICK_SCROLLBAR_DOWN: keyc = 42949672968;
pub const KEYC_SECONDCLICK11_SCROLLBAR_SLIDER: keyc = 42949675783;
pub const KEYC_SECONDCLICK10_SCROLLBAR_SLIDER: keyc = 42949675527;
pub const KEYC_SECONDCLICK9_SCROLLBAR_SLIDER: keyc = 42949675271;
pub const KEYC_SECONDCLICK8_SCROLLBAR_SLIDER: keyc = 42949675015;
pub const KEYC_SECONDCLICK7_SCROLLBAR_SLIDER: keyc = 42949674759;
pub const KEYC_SECONDCLICK6_SCROLLBAR_SLIDER: keyc = 42949674503;
pub const KEYC_SECONDCLICK3_SCROLLBAR_SLIDER: keyc = 42949673735;
pub const KEYC_SECONDCLICK2_SCROLLBAR_SLIDER: keyc = 42949673479;
pub const KEYC_SECONDCLICK1_SCROLLBAR_SLIDER: keyc = 42949673223;
pub const KEYC_SECONDCLICK_SCROLLBAR_SLIDER: keyc = 42949672967;
pub const KEYC_SECONDCLICK11_SCROLLBAR_UP: keyc = 42949675782;
pub const KEYC_SECONDCLICK10_SCROLLBAR_UP: keyc = 42949675526;
pub const KEYC_SECONDCLICK9_SCROLLBAR_UP: keyc = 42949675270;
pub const KEYC_SECONDCLICK8_SCROLLBAR_UP: keyc = 42949675014;
pub const KEYC_SECONDCLICK7_SCROLLBAR_UP: keyc = 42949674758;
pub const KEYC_SECONDCLICK6_SCROLLBAR_UP: keyc = 42949674502;
pub const KEYC_SECONDCLICK3_SCROLLBAR_UP: keyc = 42949673734;
pub const KEYC_SECONDCLICK2_SCROLLBAR_UP: keyc = 42949673478;
pub const KEYC_SECONDCLICK1_SCROLLBAR_UP: keyc = 42949673222;
pub const KEYC_SECONDCLICK_SCROLLBAR_UP: keyc = 42949672966;
pub const KEYC_SECONDCLICK11_BORDER: keyc = 42949675781;
pub const KEYC_SECONDCLICK10_BORDER: keyc = 42949675525;
pub const KEYC_SECONDCLICK9_BORDER: keyc = 42949675269;
pub const KEYC_SECONDCLICK8_BORDER: keyc = 42949675013;
pub const KEYC_SECONDCLICK7_BORDER: keyc = 42949674757;
pub const KEYC_SECONDCLICK6_BORDER: keyc = 42949674501;
pub const KEYC_SECONDCLICK3_BORDER: keyc = 42949673733;
pub const KEYC_SECONDCLICK2_BORDER: keyc = 42949673477;
pub const KEYC_SECONDCLICK1_BORDER: keyc = 42949673221;
pub const KEYC_SECONDCLICK_BORDER: keyc = 42949672965;
pub const KEYC_SECONDCLICK11_STATUS_DEFAULT: keyc = 42949675780;
pub const KEYC_SECONDCLICK10_STATUS_DEFAULT: keyc = 42949675524;
pub const KEYC_SECONDCLICK9_STATUS_DEFAULT: keyc = 42949675268;
pub const KEYC_SECONDCLICK8_STATUS_DEFAULT: keyc = 42949675012;
pub const KEYC_SECONDCLICK7_STATUS_DEFAULT: keyc = 42949674756;
pub const KEYC_SECONDCLICK6_STATUS_DEFAULT: keyc = 42949674500;
pub const KEYC_SECONDCLICK3_STATUS_DEFAULT: keyc = 42949673732;
pub const KEYC_SECONDCLICK2_STATUS_DEFAULT: keyc = 42949673476;
pub const KEYC_SECONDCLICK1_STATUS_DEFAULT: keyc = 42949673220;
pub const KEYC_SECONDCLICK_STATUS_DEFAULT: keyc = 42949672964;
pub const KEYC_SECONDCLICK11_STATUS_RIGHT: keyc = 42949675779;
pub const KEYC_SECONDCLICK10_STATUS_RIGHT: keyc = 42949675523;
pub const KEYC_SECONDCLICK9_STATUS_RIGHT: keyc = 42949675267;
pub const KEYC_SECONDCLICK8_STATUS_RIGHT: keyc = 42949675011;
pub const KEYC_SECONDCLICK7_STATUS_RIGHT: keyc = 42949674755;
pub const KEYC_SECONDCLICK6_STATUS_RIGHT: keyc = 42949674499;
pub const KEYC_SECONDCLICK3_STATUS_RIGHT: keyc = 42949673731;
pub const KEYC_SECONDCLICK2_STATUS_RIGHT: keyc = 42949673475;
pub const KEYC_SECONDCLICK1_STATUS_RIGHT: keyc = 42949673219;
pub const KEYC_SECONDCLICK_STATUS_RIGHT: keyc = 42949672963;
pub const KEYC_SECONDCLICK11_STATUS_LEFT: keyc = 42949675778;
pub const KEYC_SECONDCLICK10_STATUS_LEFT: keyc = 42949675522;
pub const KEYC_SECONDCLICK9_STATUS_LEFT: keyc = 42949675266;
pub const KEYC_SECONDCLICK8_STATUS_LEFT: keyc = 42949675010;
pub const KEYC_SECONDCLICK7_STATUS_LEFT: keyc = 42949674754;
pub const KEYC_SECONDCLICK6_STATUS_LEFT: keyc = 42949674498;
pub const KEYC_SECONDCLICK3_STATUS_LEFT: keyc = 42949673730;
pub const KEYC_SECONDCLICK2_STATUS_LEFT: keyc = 42949673474;
pub const KEYC_SECONDCLICK1_STATUS_LEFT: keyc = 42949673218;
pub const KEYC_SECONDCLICK_STATUS_LEFT: keyc = 42949672962;
pub const KEYC_SECONDCLICK11_STATUS: keyc = 42949675777;
pub const KEYC_SECONDCLICK10_STATUS: keyc = 42949675521;
pub const KEYC_SECONDCLICK9_STATUS: keyc = 42949675265;
pub const KEYC_SECONDCLICK8_STATUS: keyc = 42949675009;
pub const KEYC_SECONDCLICK7_STATUS: keyc = 42949674753;
pub const KEYC_SECONDCLICK6_STATUS: keyc = 42949674497;
pub const KEYC_SECONDCLICK3_STATUS: keyc = 42949673729;
pub const KEYC_SECONDCLICK2_STATUS: keyc = 42949673473;
pub const KEYC_SECONDCLICK1_STATUS: keyc = 42949673217;
pub const KEYC_SECONDCLICK_STATUS: keyc = 42949672961;
pub const KEYC_SECONDCLICK11_PANE: keyc = 42949675776;
pub const KEYC_SECONDCLICK10_PANE: keyc = 42949675520;
pub const KEYC_SECONDCLICK9_PANE: keyc = 42949675264;
pub const KEYC_SECONDCLICK8_PANE: keyc = 42949675008;
pub const KEYC_SECONDCLICK7_PANE: keyc = 42949674752;
pub const KEYC_SECONDCLICK6_PANE: keyc = 42949674496;
pub const KEYC_SECONDCLICK3_PANE: keyc = 42949673728;
pub const KEYC_SECONDCLICK2_PANE: keyc = 42949673472;
pub const KEYC_SECONDCLICK1_PANE: keyc = 42949673216;
pub const KEYC_SECONDCLICK_PANE: keyc = 42949672960;
pub const KEYC_MOUSEDRAGEND11_CONTROL9: keyc = 30064773906;
pub const KEYC_MOUSEDRAGEND10_CONTROL9: keyc = 30064773650;
pub const KEYC_MOUSEDRAGEND9_CONTROL9: keyc = 30064773394;
pub const KEYC_MOUSEDRAGEND8_CONTROL9: keyc = 30064773138;
pub const KEYC_MOUSEDRAGEND7_CONTROL9: keyc = 30064772882;
pub const KEYC_MOUSEDRAGEND6_CONTROL9: keyc = 30064772626;
pub const KEYC_MOUSEDRAGEND3_CONTROL9: keyc = 30064771858;
pub const KEYC_MOUSEDRAGEND2_CONTROL9: keyc = 30064771602;
pub const KEYC_MOUSEDRAGEND1_CONTROL9: keyc = 30064771346;
pub const KEYC_MOUSEDRAGEND_CONTROL9: keyc = 30064771090;
pub const KEYC_MOUSEDRAGEND11_CONTROL8: keyc = 30064773905;
pub const KEYC_MOUSEDRAGEND10_CONTROL8: keyc = 30064773649;
pub const KEYC_MOUSEDRAGEND9_CONTROL8: keyc = 30064773393;
pub const KEYC_MOUSEDRAGEND8_CONTROL8: keyc = 30064773137;
pub const KEYC_MOUSEDRAGEND7_CONTROL8: keyc = 30064772881;
pub const KEYC_MOUSEDRAGEND6_CONTROL8: keyc = 30064772625;
pub const KEYC_MOUSEDRAGEND3_CONTROL8: keyc = 30064771857;
pub const KEYC_MOUSEDRAGEND2_CONTROL8: keyc = 30064771601;
pub const KEYC_MOUSEDRAGEND1_CONTROL8: keyc = 30064771345;
pub const KEYC_MOUSEDRAGEND_CONTROL8: keyc = 30064771089;
pub const KEYC_MOUSEDRAGEND11_CONTROL7: keyc = 30064773904;
pub const KEYC_MOUSEDRAGEND10_CONTROL7: keyc = 30064773648;
pub const KEYC_MOUSEDRAGEND9_CONTROL7: keyc = 30064773392;
pub const KEYC_MOUSEDRAGEND8_CONTROL7: keyc = 30064773136;
pub const KEYC_MOUSEDRAGEND7_CONTROL7: keyc = 30064772880;
pub const KEYC_MOUSEDRAGEND6_CONTROL7: keyc = 30064772624;
pub const KEYC_MOUSEDRAGEND3_CONTROL7: keyc = 30064771856;
pub const KEYC_MOUSEDRAGEND2_CONTROL7: keyc = 30064771600;
pub const KEYC_MOUSEDRAGEND1_CONTROL7: keyc = 30064771344;
pub const KEYC_MOUSEDRAGEND_CONTROL7: keyc = 30064771088;
pub const KEYC_MOUSEDRAGEND11_CONTROL6: keyc = 30064773903;
pub const KEYC_MOUSEDRAGEND10_CONTROL6: keyc = 30064773647;
pub const KEYC_MOUSEDRAGEND9_CONTROL6: keyc = 30064773391;
pub const KEYC_MOUSEDRAGEND8_CONTROL6: keyc = 30064773135;
pub const KEYC_MOUSEDRAGEND7_CONTROL6: keyc = 30064772879;
pub const KEYC_MOUSEDRAGEND6_CONTROL6: keyc = 30064772623;
pub const KEYC_MOUSEDRAGEND3_CONTROL6: keyc = 30064771855;
pub const KEYC_MOUSEDRAGEND2_CONTROL6: keyc = 30064771599;
pub const KEYC_MOUSEDRAGEND1_CONTROL6: keyc = 30064771343;
pub const KEYC_MOUSEDRAGEND_CONTROL6: keyc = 30064771087;
pub const KEYC_MOUSEDRAGEND11_CONTROL5: keyc = 30064773902;
pub const KEYC_MOUSEDRAGEND10_CONTROL5: keyc = 30064773646;
pub const KEYC_MOUSEDRAGEND9_CONTROL5: keyc = 30064773390;
pub const KEYC_MOUSEDRAGEND8_CONTROL5: keyc = 30064773134;
pub const KEYC_MOUSEDRAGEND7_CONTROL5: keyc = 30064772878;
pub const KEYC_MOUSEDRAGEND6_CONTROL5: keyc = 30064772622;
pub const KEYC_MOUSEDRAGEND3_CONTROL5: keyc = 30064771854;
pub const KEYC_MOUSEDRAGEND2_CONTROL5: keyc = 30064771598;
pub const KEYC_MOUSEDRAGEND1_CONTROL5: keyc = 30064771342;
pub const KEYC_MOUSEDRAGEND_CONTROL5: keyc = 30064771086;
pub const KEYC_MOUSEDRAGEND11_CONTROL4: keyc = 30064773901;
pub const KEYC_MOUSEDRAGEND10_CONTROL4: keyc = 30064773645;
pub const KEYC_MOUSEDRAGEND9_CONTROL4: keyc = 30064773389;
pub const KEYC_MOUSEDRAGEND8_CONTROL4: keyc = 30064773133;
pub const KEYC_MOUSEDRAGEND7_CONTROL4: keyc = 30064772877;
pub const KEYC_MOUSEDRAGEND6_CONTROL4: keyc = 30064772621;
pub const KEYC_MOUSEDRAGEND3_CONTROL4: keyc = 30064771853;
pub const KEYC_MOUSEDRAGEND2_CONTROL4: keyc = 30064771597;
pub const KEYC_MOUSEDRAGEND1_CONTROL4: keyc = 30064771341;
pub const KEYC_MOUSEDRAGEND_CONTROL4: keyc = 30064771085;
pub const KEYC_MOUSEDRAGEND11_CONTROL3: keyc = 30064773900;
pub const KEYC_MOUSEDRAGEND10_CONTROL3: keyc = 30064773644;
pub const KEYC_MOUSEDRAGEND9_CONTROL3: keyc = 30064773388;
pub const KEYC_MOUSEDRAGEND8_CONTROL3: keyc = 30064773132;
pub const KEYC_MOUSEDRAGEND7_CONTROL3: keyc = 30064772876;
pub const KEYC_MOUSEDRAGEND6_CONTROL3: keyc = 30064772620;
pub const KEYC_MOUSEDRAGEND3_CONTROL3: keyc = 30064771852;
pub const KEYC_MOUSEDRAGEND2_CONTROL3: keyc = 30064771596;
pub const KEYC_MOUSEDRAGEND1_CONTROL3: keyc = 30064771340;
pub const KEYC_MOUSEDRAGEND_CONTROL3: keyc = 30064771084;
pub const KEYC_MOUSEDRAGEND11_CONTROL2: keyc = 30064773899;
pub const KEYC_MOUSEDRAGEND10_CONTROL2: keyc = 30064773643;
pub const KEYC_MOUSEDRAGEND9_CONTROL2: keyc = 30064773387;
pub const KEYC_MOUSEDRAGEND8_CONTROL2: keyc = 30064773131;
pub const KEYC_MOUSEDRAGEND7_CONTROL2: keyc = 30064772875;
pub const KEYC_MOUSEDRAGEND6_CONTROL2: keyc = 30064772619;
pub const KEYC_MOUSEDRAGEND3_CONTROL2: keyc = 30064771851;
pub const KEYC_MOUSEDRAGEND2_CONTROL2: keyc = 30064771595;
pub const KEYC_MOUSEDRAGEND1_CONTROL2: keyc = 30064771339;
pub const KEYC_MOUSEDRAGEND_CONTROL2: keyc = 30064771083;
pub const KEYC_MOUSEDRAGEND11_CONTROL1: keyc = 30064773898;
pub const KEYC_MOUSEDRAGEND10_CONTROL1: keyc = 30064773642;
pub const KEYC_MOUSEDRAGEND9_CONTROL1: keyc = 30064773386;
pub const KEYC_MOUSEDRAGEND8_CONTROL1: keyc = 30064773130;
pub const KEYC_MOUSEDRAGEND7_CONTROL1: keyc = 30064772874;
pub const KEYC_MOUSEDRAGEND6_CONTROL1: keyc = 30064772618;
pub const KEYC_MOUSEDRAGEND3_CONTROL1: keyc = 30064771850;
pub const KEYC_MOUSEDRAGEND2_CONTROL1: keyc = 30064771594;
pub const KEYC_MOUSEDRAGEND1_CONTROL1: keyc = 30064771338;
pub const KEYC_MOUSEDRAGEND_CONTROL1: keyc = 30064771082;
pub const KEYC_MOUSEDRAGEND11_CONTROL0: keyc = 30064773897;
pub const KEYC_MOUSEDRAGEND10_CONTROL0: keyc = 30064773641;
pub const KEYC_MOUSEDRAGEND9_CONTROL0: keyc = 30064773385;
pub const KEYC_MOUSEDRAGEND8_CONTROL0: keyc = 30064773129;
pub const KEYC_MOUSEDRAGEND7_CONTROL0: keyc = 30064772873;
pub const KEYC_MOUSEDRAGEND6_CONTROL0: keyc = 30064772617;
pub const KEYC_MOUSEDRAGEND3_CONTROL0: keyc = 30064771849;
pub const KEYC_MOUSEDRAGEND2_CONTROL0: keyc = 30064771593;
pub const KEYC_MOUSEDRAGEND1_CONTROL0: keyc = 30064771337;
pub const KEYC_MOUSEDRAGEND_CONTROL0: keyc = 30064771081;
pub const KEYC_MOUSEDRAGEND11_SCROLLBAR_DOWN: keyc = 30064773896;
pub const KEYC_MOUSEDRAGEND10_SCROLLBAR_DOWN: keyc = 30064773640;
pub const KEYC_MOUSEDRAGEND9_SCROLLBAR_DOWN: keyc = 30064773384;
pub const KEYC_MOUSEDRAGEND8_SCROLLBAR_DOWN: keyc = 30064773128;
pub const KEYC_MOUSEDRAGEND7_SCROLLBAR_DOWN: keyc = 30064772872;
pub const KEYC_MOUSEDRAGEND6_SCROLLBAR_DOWN: keyc = 30064772616;
pub const KEYC_MOUSEDRAGEND3_SCROLLBAR_DOWN: keyc = 30064771848;
pub const KEYC_MOUSEDRAGEND2_SCROLLBAR_DOWN: keyc = 30064771592;
pub const KEYC_MOUSEDRAGEND1_SCROLLBAR_DOWN: keyc = 30064771336;
pub const KEYC_MOUSEDRAGEND_SCROLLBAR_DOWN: keyc = 30064771080;
pub const KEYC_MOUSEDRAGEND11_SCROLLBAR_SLIDER: keyc = 30064773895;
pub const KEYC_MOUSEDRAGEND10_SCROLLBAR_SLIDER: keyc = 30064773639;
pub const KEYC_MOUSEDRAGEND9_SCROLLBAR_SLIDER: keyc = 30064773383;
pub const KEYC_MOUSEDRAGEND8_SCROLLBAR_SLIDER: keyc = 30064773127;
pub const KEYC_MOUSEDRAGEND7_SCROLLBAR_SLIDER: keyc = 30064772871;
pub const KEYC_MOUSEDRAGEND6_SCROLLBAR_SLIDER: keyc = 30064772615;
pub const KEYC_MOUSEDRAGEND3_SCROLLBAR_SLIDER: keyc = 30064771847;
pub const KEYC_MOUSEDRAGEND2_SCROLLBAR_SLIDER: keyc = 30064771591;
pub const KEYC_MOUSEDRAGEND1_SCROLLBAR_SLIDER: keyc = 30064771335;
pub const KEYC_MOUSEDRAGEND_SCROLLBAR_SLIDER: keyc = 30064771079;
pub const KEYC_MOUSEDRAGEND11_SCROLLBAR_UP: keyc = 30064773894;
pub const KEYC_MOUSEDRAGEND10_SCROLLBAR_UP: keyc = 30064773638;
pub const KEYC_MOUSEDRAGEND9_SCROLLBAR_UP: keyc = 30064773382;
pub const KEYC_MOUSEDRAGEND8_SCROLLBAR_UP: keyc = 30064773126;
pub const KEYC_MOUSEDRAGEND7_SCROLLBAR_UP: keyc = 30064772870;
pub const KEYC_MOUSEDRAGEND6_SCROLLBAR_UP: keyc = 30064772614;
pub const KEYC_MOUSEDRAGEND3_SCROLLBAR_UP: keyc = 30064771846;
pub const KEYC_MOUSEDRAGEND2_SCROLLBAR_UP: keyc = 30064771590;
pub const KEYC_MOUSEDRAGEND1_SCROLLBAR_UP: keyc = 30064771334;
pub const KEYC_MOUSEDRAGEND_SCROLLBAR_UP: keyc = 30064771078;
pub const KEYC_MOUSEDRAGEND11_BORDER: keyc = 30064773893;
pub const KEYC_MOUSEDRAGEND10_BORDER: keyc = 30064773637;
pub const KEYC_MOUSEDRAGEND9_BORDER: keyc = 30064773381;
pub const KEYC_MOUSEDRAGEND8_BORDER: keyc = 30064773125;
pub const KEYC_MOUSEDRAGEND7_BORDER: keyc = 30064772869;
pub const KEYC_MOUSEDRAGEND6_BORDER: keyc = 30064772613;
pub const KEYC_MOUSEDRAGEND3_BORDER: keyc = 30064771845;
pub const KEYC_MOUSEDRAGEND2_BORDER: keyc = 30064771589;
pub const KEYC_MOUSEDRAGEND1_BORDER: keyc = 30064771333;
pub const KEYC_MOUSEDRAGEND_BORDER: keyc = 30064771077;
pub const KEYC_MOUSEDRAGEND11_STATUS_DEFAULT: keyc = 30064773892;
pub const KEYC_MOUSEDRAGEND10_STATUS_DEFAULT: keyc = 30064773636;
pub const KEYC_MOUSEDRAGEND9_STATUS_DEFAULT: keyc = 30064773380;
pub const KEYC_MOUSEDRAGEND8_STATUS_DEFAULT: keyc = 30064773124;
pub const KEYC_MOUSEDRAGEND7_STATUS_DEFAULT: keyc = 30064772868;
pub const KEYC_MOUSEDRAGEND6_STATUS_DEFAULT: keyc = 30064772612;
pub const KEYC_MOUSEDRAGEND3_STATUS_DEFAULT: keyc = 30064771844;
pub const KEYC_MOUSEDRAGEND2_STATUS_DEFAULT: keyc = 30064771588;
pub const KEYC_MOUSEDRAGEND1_STATUS_DEFAULT: keyc = 30064771332;
pub const KEYC_MOUSEDRAGEND_STATUS_DEFAULT: keyc = 30064771076;
pub const KEYC_MOUSEDRAGEND11_STATUS_RIGHT: keyc = 30064773891;
pub const KEYC_MOUSEDRAGEND10_STATUS_RIGHT: keyc = 30064773635;
pub const KEYC_MOUSEDRAGEND9_STATUS_RIGHT: keyc = 30064773379;
pub const KEYC_MOUSEDRAGEND8_STATUS_RIGHT: keyc = 30064773123;
pub const KEYC_MOUSEDRAGEND7_STATUS_RIGHT: keyc = 30064772867;
pub const KEYC_MOUSEDRAGEND6_STATUS_RIGHT: keyc = 30064772611;
pub const KEYC_MOUSEDRAGEND3_STATUS_RIGHT: keyc = 30064771843;
pub const KEYC_MOUSEDRAGEND2_STATUS_RIGHT: keyc = 30064771587;
pub const KEYC_MOUSEDRAGEND1_STATUS_RIGHT: keyc = 30064771331;
pub const KEYC_MOUSEDRAGEND_STATUS_RIGHT: keyc = 30064771075;
pub const KEYC_MOUSEDRAGEND11_STATUS_LEFT: keyc = 30064773890;
pub const KEYC_MOUSEDRAGEND10_STATUS_LEFT: keyc = 30064773634;
pub const KEYC_MOUSEDRAGEND9_STATUS_LEFT: keyc = 30064773378;
pub const KEYC_MOUSEDRAGEND8_STATUS_LEFT: keyc = 30064773122;
pub const KEYC_MOUSEDRAGEND7_STATUS_LEFT: keyc = 30064772866;
pub const KEYC_MOUSEDRAGEND6_STATUS_LEFT: keyc = 30064772610;
pub const KEYC_MOUSEDRAGEND3_STATUS_LEFT: keyc = 30064771842;
pub const KEYC_MOUSEDRAGEND2_STATUS_LEFT: keyc = 30064771586;
pub const KEYC_MOUSEDRAGEND1_STATUS_LEFT: keyc = 30064771330;
pub const KEYC_MOUSEDRAGEND_STATUS_LEFT: keyc = 30064771074;
pub const KEYC_MOUSEDRAGEND11_STATUS: keyc = 30064773889;
pub const KEYC_MOUSEDRAGEND10_STATUS: keyc = 30064773633;
pub const KEYC_MOUSEDRAGEND9_STATUS: keyc = 30064773377;
pub const KEYC_MOUSEDRAGEND8_STATUS: keyc = 30064773121;
pub const KEYC_MOUSEDRAGEND7_STATUS: keyc = 30064772865;
pub const KEYC_MOUSEDRAGEND6_STATUS: keyc = 30064772609;
pub const KEYC_MOUSEDRAGEND3_STATUS: keyc = 30064771841;
pub const KEYC_MOUSEDRAGEND2_STATUS: keyc = 30064771585;
pub const KEYC_MOUSEDRAGEND1_STATUS: keyc = 30064771329;
pub const KEYC_MOUSEDRAGEND_STATUS: keyc = 30064771073;
pub const KEYC_MOUSEDRAGEND11_PANE: keyc = 30064773888;
pub const KEYC_MOUSEDRAGEND10_PANE: keyc = 30064773632;
pub const KEYC_MOUSEDRAGEND9_PANE: keyc = 30064773376;
pub const KEYC_MOUSEDRAGEND8_PANE: keyc = 30064773120;
pub const KEYC_MOUSEDRAGEND7_PANE: keyc = 30064772864;
pub const KEYC_MOUSEDRAGEND6_PANE: keyc = 30064772608;
pub const KEYC_MOUSEDRAGEND3_PANE: keyc = 30064771840;
pub const KEYC_MOUSEDRAGEND2_PANE: keyc = 30064771584;
pub const KEYC_MOUSEDRAGEND1_PANE: keyc = 30064771328;
pub const KEYC_MOUSEDRAGEND_PANE: keyc = 30064771072;
pub const KEYC_MOUSEDRAG11_CONTROL9: keyc = 25769806610;
pub const KEYC_MOUSEDRAG10_CONTROL9: keyc = 25769806354;
pub const KEYC_MOUSEDRAG9_CONTROL9: keyc = 25769806098;
pub const KEYC_MOUSEDRAG8_CONTROL9: keyc = 25769805842;
pub const KEYC_MOUSEDRAG7_CONTROL9: keyc = 25769805586;
pub const KEYC_MOUSEDRAG6_CONTROL9: keyc = 25769805330;
pub const KEYC_MOUSEDRAG3_CONTROL9: keyc = 25769804562;
pub const KEYC_MOUSEDRAG2_CONTROL9: keyc = 25769804306;
pub const KEYC_MOUSEDRAG1_CONTROL9: keyc = 25769804050;
pub const KEYC_MOUSEDRAG_CONTROL9: keyc = 25769803794;
pub const KEYC_MOUSEDRAG11_CONTROL8: keyc = 25769806609;
pub const KEYC_MOUSEDRAG10_CONTROL8: keyc = 25769806353;
pub const KEYC_MOUSEDRAG9_CONTROL8: keyc = 25769806097;
pub const KEYC_MOUSEDRAG8_CONTROL8: keyc = 25769805841;
pub const KEYC_MOUSEDRAG7_CONTROL8: keyc = 25769805585;
pub const KEYC_MOUSEDRAG6_CONTROL8: keyc = 25769805329;
pub const KEYC_MOUSEDRAG3_CONTROL8: keyc = 25769804561;
pub const KEYC_MOUSEDRAG2_CONTROL8: keyc = 25769804305;
pub const KEYC_MOUSEDRAG1_CONTROL8: keyc = 25769804049;
pub const KEYC_MOUSEDRAG_CONTROL8: keyc = 25769803793;
pub const KEYC_MOUSEDRAG11_CONTROL7: keyc = 25769806608;
pub const KEYC_MOUSEDRAG10_CONTROL7: keyc = 25769806352;
pub const KEYC_MOUSEDRAG9_CONTROL7: keyc = 25769806096;
pub const KEYC_MOUSEDRAG8_CONTROL7: keyc = 25769805840;
pub const KEYC_MOUSEDRAG7_CONTROL7: keyc = 25769805584;
pub const KEYC_MOUSEDRAG6_CONTROL7: keyc = 25769805328;
pub const KEYC_MOUSEDRAG3_CONTROL7: keyc = 25769804560;
pub const KEYC_MOUSEDRAG2_CONTROL7: keyc = 25769804304;
pub const KEYC_MOUSEDRAG1_CONTROL7: keyc = 25769804048;
pub const KEYC_MOUSEDRAG_CONTROL7: keyc = 25769803792;
pub const KEYC_MOUSEDRAG11_CONTROL6: keyc = 25769806607;
pub const KEYC_MOUSEDRAG10_CONTROL6: keyc = 25769806351;
pub const KEYC_MOUSEDRAG9_CONTROL6: keyc = 25769806095;
pub const KEYC_MOUSEDRAG8_CONTROL6: keyc = 25769805839;
pub const KEYC_MOUSEDRAG7_CONTROL6: keyc = 25769805583;
pub const KEYC_MOUSEDRAG6_CONTROL6: keyc = 25769805327;
pub const KEYC_MOUSEDRAG3_CONTROL6: keyc = 25769804559;
pub const KEYC_MOUSEDRAG2_CONTROL6: keyc = 25769804303;
pub const KEYC_MOUSEDRAG1_CONTROL6: keyc = 25769804047;
pub const KEYC_MOUSEDRAG_CONTROL6: keyc = 25769803791;
pub const KEYC_MOUSEDRAG11_CONTROL5: keyc = 25769806606;
pub const KEYC_MOUSEDRAG10_CONTROL5: keyc = 25769806350;
pub const KEYC_MOUSEDRAG9_CONTROL5: keyc = 25769806094;
pub const KEYC_MOUSEDRAG8_CONTROL5: keyc = 25769805838;
pub const KEYC_MOUSEDRAG7_CONTROL5: keyc = 25769805582;
pub const KEYC_MOUSEDRAG6_CONTROL5: keyc = 25769805326;
pub const KEYC_MOUSEDRAG3_CONTROL5: keyc = 25769804558;
pub const KEYC_MOUSEDRAG2_CONTROL5: keyc = 25769804302;
pub const KEYC_MOUSEDRAG1_CONTROL5: keyc = 25769804046;
pub const KEYC_MOUSEDRAG_CONTROL5: keyc = 25769803790;
pub const KEYC_MOUSEDRAG11_CONTROL4: keyc = 25769806605;
pub const KEYC_MOUSEDRAG10_CONTROL4: keyc = 25769806349;
pub const KEYC_MOUSEDRAG9_CONTROL4: keyc = 25769806093;
pub const KEYC_MOUSEDRAG8_CONTROL4: keyc = 25769805837;
pub const KEYC_MOUSEDRAG7_CONTROL4: keyc = 25769805581;
pub const KEYC_MOUSEDRAG6_CONTROL4: keyc = 25769805325;
pub const KEYC_MOUSEDRAG3_CONTROL4: keyc = 25769804557;
pub const KEYC_MOUSEDRAG2_CONTROL4: keyc = 25769804301;
pub const KEYC_MOUSEDRAG1_CONTROL4: keyc = 25769804045;
pub const KEYC_MOUSEDRAG_CONTROL4: keyc = 25769803789;
pub const KEYC_MOUSEDRAG11_CONTROL3: keyc = 25769806604;
pub const KEYC_MOUSEDRAG10_CONTROL3: keyc = 25769806348;
pub const KEYC_MOUSEDRAG9_CONTROL3: keyc = 25769806092;
pub const KEYC_MOUSEDRAG8_CONTROL3: keyc = 25769805836;
pub const KEYC_MOUSEDRAG7_CONTROL3: keyc = 25769805580;
pub const KEYC_MOUSEDRAG6_CONTROL3: keyc = 25769805324;
pub const KEYC_MOUSEDRAG3_CONTROL3: keyc = 25769804556;
pub const KEYC_MOUSEDRAG2_CONTROL3: keyc = 25769804300;
pub const KEYC_MOUSEDRAG1_CONTROL3: keyc = 25769804044;
pub const KEYC_MOUSEDRAG_CONTROL3: keyc = 25769803788;
pub const KEYC_MOUSEDRAG11_CONTROL2: keyc = 25769806603;
pub const KEYC_MOUSEDRAG10_CONTROL2: keyc = 25769806347;
pub const KEYC_MOUSEDRAG9_CONTROL2: keyc = 25769806091;
pub const KEYC_MOUSEDRAG8_CONTROL2: keyc = 25769805835;
pub const KEYC_MOUSEDRAG7_CONTROL2: keyc = 25769805579;
pub const KEYC_MOUSEDRAG6_CONTROL2: keyc = 25769805323;
pub const KEYC_MOUSEDRAG3_CONTROL2: keyc = 25769804555;
pub const KEYC_MOUSEDRAG2_CONTROL2: keyc = 25769804299;
pub const KEYC_MOUSEDRAG1_CONTROL2: keyc = 25769804043;
pub const KEYC_MOUSEDRAG_CONTROL2: keyc = 25769803787;
pub const KEYC_MOUSEDRAG11_CONTROL1: keyc = 25769806602;
pub const KEYC_MOUSEDRAG10_CONTROL1: keyc = 25769806346;
pub const KEYC_MOUSEDRAG9_CONTROL1: keyc = 25769806090;
pub const KEYC_MOUSEDRAG8_CONTROL1: keyc = 25769805834;
pub const KEYC_MOUSEDRAG7_CONTROL1: keyc = 25769805578;
pub const KEYC_MOUSEDRAG6_CONTROL1: keyc = 25769805322;
pub const KEYC_MOUSEDRAG3_CONTROL1: keyc = 25769804554;
pub const KEYC_MOUSEDRAG2_CONTROL1: keyc = 25769804298;
pub const KEYC_MOUSEDRAG1_CONTROL1: keyc = 25769804042;
pub const KEYC_MOUSEDRAG_CONTROL1: keyc = 25769803786;
pub const KEYC_MOUSEDRAG11_CONTROL0: keyc = 25769806601;
pub const KEYC_MOUSEDRAG10_CONTROL0: keyc = 25769806345;
pub const KEYC_MOUSEDRAG9_CONTROL0: keyc = 25769806089;
pub const KEYC_MOUSEDRAG8_CONTROL0: keyc = 25769805833;
pub const KEYC_MOUSEDRAG7_CONTROL0: keyc = 25769805577;
pub const KEYC_MOUSEDRAG6_CONTROL0: keyc = 25769805321;
pub const KEYC_MOUSEDRAG3_CONTROL0: keyc = 25769804553;
pub const KEYC_MOUSEDRAG2_CONTROL0: keyc = 25769804297;
pub const KEYC_MOUSEDRAG1_CONTROL0: keyc = 25769804041;
pub const KEYC_MOUSEDRAG_CONTROL0: keyc = 25769803785;
pub const KEYC_MOUSEDRAG11_SCROLLBAR_DOWN: keyc = 25769806600;
pub const KEYC_MOUSEDRAG10_SCROLLBAR_DOWN: keyc = 25769806344;
pub const KEYC_MOUSEDRAG9_SCROLLBAR_DOWN: keyc = 25769806088;
pub const KEYC_MOUSEDRAG8_SCROLLBAR_DOWN: keyc = 25769805832;
pub const KEYC_MOUSEDRAG7_SCROLLBAR_DOWN: keyc = 25769805576;
pub const KEYC_MOUSEDRAG6_SCROLLBAR_DOWN: keyc = 25769805320;
pub const KEYC_MOUSEDRAG3_SCROLLBAR_DOWN: keyc = 25769804552;
pub const KEYC_MOUSEDRAG2_SCROLLBAR_DOWN: keyc = 25769804296;
pub const KEYC_MOUSEDRAG1_SCROLLBAR_DOWN: keyc = 25769804040;
pub const KEYC_MOUSEDRAG_SCROLLBAR_DOWN: keyc = 25769803784;
pub const KEYC_MOUSEDRAG11_SCROLLBAR_SLIDER: keyc = 25769806599;
pub const KEYC_MOUSEDRAG10_SCROLLBAR_SLIDER: keyc = 25769806343;
pub const KEYC_MOUSEDRAG9_SCROLLBAR_SLIDER: keyc = 25769806087;
pub const KEYC_MOUSEDRAG8_SCROLLBAR_SLIDER: keyc = 25769805831;
pub const KEYC_MOUSEDRAG7_SCROLLBAR_SLIDER: keyc = 25769805575;
pub const KEYC_MOUSEDRAG6_SCROLLBAR_SLIDER: keyc = 25769805319;
pub const KEYC_MOUSEDRAG3_SCROLLBAR_SLIDER: keyc = 25769804551;
pub const KEYC_MOUSEDRAG2_SCROLLBAR_SLIDER: keyc = 25769804295;
pub const KEYC_MOUSEDRAG1_SCROLLBAR_SLIDER: keyc = 25769804039;
pub const KEYC_MOUSEDRAG_SCROLLBAR_SLIDER: keyc = 25769803783;
pub const KEYC_MOUSEDRAG11_SCROLLBAR_UP: keyc = 25769806598;
pub const KEYC_MOUSEDRAG10_SCROLLBAR_UP: keyc = 25769806342;
pub const KEYC_MOUSEDRAG9_SCROLLBAR_UP: keyc = 25769806086;
pub const KEYC_MOUSEDRAG8_SCROLLBAR_UP: keyc = 25769805830;
pub const KEYC_MOUSEDRAG7_SCROLLBAR_UP: keyc = 25769805574;
pub const KEYC_MOUSEDRAG6_SCROLLBAR_UP: keyc = 25769805318;
pub const KEYC_MOUSEDRAG3_SCROLLBAR_UP: keyc = 25769804550;
pub const KEYC_MOUSEDRAG2_SCROLLBAR_UP: keyc = 25769804294;
pub const KEYC_MOUSEDRAG1_SCROLLBAR_UP: keyc = 25769804038;
pub const KEYC_MOUSEDRAG_SCROLLBAR_UP: keyc = 25769803782;
pub const KEYC_MOUSEDRAG11_BORDER: keyc = 25769806597;
pub const KEYC_MOUSEDRAG10_BORDER: keyc = 25769806341;
pub const KEYC_MOUSEDRAG9_BORDER: keyc = 25769806085;
pub const KEYC_MOUSEDRAG8_BORDER: keyc = 25769805829;
pub const KEYC_MOUSEDRAG7_BORDER: keyc = 25769805573;
pub const KEYC_MOUSEDRAG6_BORDER: keyc = 25769805317;
pub const KEYC_MOUSEDRAG3_BORDER: keyc = 25769804549;
pub const KEYC_MOUSEDRAG2_BORDER: keyc = 25769804293;
pub const KEYC_MOUSEDRAG1_BORDER: keyc = 25769804037;
pub const KEYC_MOUSEDRAG_BORDER: keyc = 25769803781;
pub const KEYC_MOUSEDRAG11_STATUS_DEFAULT: keyc = 25769806596;
pub const KEYC_MOUSEDRAG10_STATUS_DEFAULT: keyc = 25769806340;
pub const KEYC_MOUSEDRAG9_STATUS_DEFAULT: keyc = 25769806084;
pub const KEYC_MOUSEDRAG8_STATUS_DEFAULT: keyc = 25769805828;
pub const KEYC_MOUSEDRAG7_STATUS_DEFAULT: keyc = 25769805572;
pub const KEYC_MOUSEDRAG6_STATUS_DEFAULT: keyc = 25769805316;
pub const KEYC_MOUSEDRAG3_STATUS_DEFAULT: keyc = 25769804548;
pub const KEYC_MOUSEDRAG2_STATUS_DEFAULT: keyc = 25769804292;
pub const KEYC_MOUSEDRAG1_STATUS_DEFAULT: keyc = 25769804036;
pub const KEYC_MOUSEDRAG_STATUS_DEFAULT: keyc = 25769803780;
pub const KEYC_MOUSEDRAG11_STATUS_RIGHT: keyc = 25769806595;
pub const KEYC_MOUSEDRAG10_STATUS_RIGHT: keyc = 25769806339;
pub const KEYC_MOUSEDRAG9_STATUS_RIGHT: keyc = 25769806083;
pub const KEYC_MOUSEDRAG8_STATUS_RIGHT: keyc = 25769805827;
pub const KEYC_MOUSEDRAG7_STATUS_RIGHT: keyc = 25769805571;
pub const KEYC_MOUSEDRAG6_STATUS_RIGHT: keyc = 25769805315;
pub const KEYC_MOUSEDRAG3_STATUS_RIGHT: keyc = 25769804547;
pub const KEYC_MOUSEDRAG2_STATUS_RIGHT: keyc = 25769804291;
pub const KEYC_MOUSEDRAG1_STATUS_RIGHT: keyc = 25769804035;
pub const KEYC_MOUSEDRAG_STATUS_RIGHT: keyc = 25769803779;
pub const KEYC_MOUSEDRAG11_STATUS_LEFT: keyc = 25769806594;
pub const KEYC_MOUSEDRAG10_STATUS_LEFT: keyc = 25769806338;
pub const KEYC_MOUSEDRAG9_STATUS_LEFT: keyc = 25769806082;
pub const KEYC_MOUSEDRAG8_STATUS_LEFT: keyc = 25769805826;
pub const KEYC_MOUSEDRAG7_STATUS_LEFT: keyc = 25769805570;
pub const KEYC_MOUSEDRAG6_STATUS_LEFT: keyc = 25769805314;
pub const KEYC_MOUSEDRAG3_STATUS_LEFT: keyc = 25769804546;
pub const KEYC_MOUSEDRAG2_STATUS_LEFT: keyc = 25769804290;
pub const KEYC_MOUSEDRAG1_STATUS_LEFT: keyc = 25769804034;
pub const KEYC_MOUSEDRAG_STATUS_LEFT: keyc = 25769803778;
pub const KEYC_MOUSEDRAG11_STATUS: keyc = 25769806593;
pub const KEYC_MOUSEDRAG10_STATUS: keyc = 25769806337;
pub const KEYC_MOUSEDRAG9_STATUS: keyc = 25769806081;
pub const KEYC_MOUSEDRAG8_STATUS: keyc = 25769805825;
pub const KEYC_MOUSEDRAG7_STATUS: keyc = 25769805569;
pub const KEYC_MOUSEDRAG6_STATUS: keyc = 25769805313;
pub const KEYC_MOUSEDRAG3_STATUS: keyc = 25769804545;
pub const KEYC_MOUSEDRAG2_STATUS: keyc = 25769804289;
pub const KEYC_MOUSEDRAG1_STATUS: keyc = 25769804033;
pub const KEYC_MOUSEDRAG_STATUS: keyc = 25769803777;
pub const KEYC_MOUSEDRAG11_PANE: keyc = 25769806592;
pub const KEYC_MOUSEDRAG10_PANE: keyc = 25769806336;
pub const KEYC_MOUSEDRAG9_PANE: keyc = 25769806080;
pub const KEYC_MOUSEDRAG8_PANE: keyc = 25769805824;
pub const KEYC_MOUSEDRAG7_PANE: keyc = 25769805568;
pub const KEYC_MOUSEDRAG6_PANE: keyc = 25769805312;
pub const KEYC_MOUSEDRAG3_PANE: keyc = 25769804544;
pub const KEYC_MOUSEDRAG2_PANE: keyc = 25769804288;
pub const KEYC_MOUSEDRAG1_PANE: keyc = 25769804032;
pub const KEYC_MOUSEDRAG_PANE: keyc = 25769803776;
pub const KEYC_MOUSEUP11_CONTROL9: keyc = 21474839314;
pub const KEYC_MOUSEUP10_CONTROL9: keyc = 21474839058;
pub const KEYC_MOUSEUP9_CONTROL9: keyc = 21474838802;
pub const KEYC_MOUSEUP8_CONTROL9: keyc = 21474838546;
pub const KEYC_MOUSEUP7_CONTROL9: keyc = 21474838290;
pub const KEYC_MOUSEUP6_CONTROL9: keyc = 21474838034;
pub const KEYC_MOUSEUP3_CONTROL9: keyc = 21474837266;
pub const KEYC_MOUSEUP2_CONTROL9: keyc = 21474837010;
pub const KEYC_MOUSEUP1_CONTROL9: keyc = 21474836754;
pub const KEYC_MOUSEUP_CONTROL9: keyc = 21474836498;
pub const KEYC_MOUSEUP11_CONTROL8: keyc = 21474839313;
pub const KEYC_MOUSEUP10_CONTROL8: keyc = 21474839057;
pub const KEYC_MOUSEUP9_CONTROL8: keyc = 21474838801;
pub const KEYC_MOUSEUP8_CONTROL8: keyc = 21474838545;
pub const KEYC_MOUSEUP7_CONTROL8: keyc = 21474838289;
pub const KEYC_MOUSEUP6_CONTROL8: keyc = 21474838033;
pub const KEYC_MOUSEUP3_CONTROL8: keyc = 21474837265;
pub const KEYC_MOUSEUP2_CONTROL8: keyc = 21474837009;
pub const KEYC_MOUSEUP1_CONTROL8: keyc = 21474836753;
pub const KEYC_MOUSEUP_CONTROL8: keyc = 21474836497;
pub const KEYC_MOUSEUP11_CONTROL7: keyc = 21474839312;
pub const KEYC_MOUSEUP10_CONTROL7: keyc = 21474839056;
pub const KEYC_MOUSEUP9_CONTROL7: keyc = 21474838800;
pub const KEYC_MOUSEUP8_CONTROL7: keyc = 21474838544;
pub const KEYC_MOUSEUP7_CONTROL7: keyc = 21474838288;
pub const KEYC_MOUSEUP6_CONTROL7: keyc = 21474838032;
pub const KEYC_MOUSEUP3_CONTROL7: keyc = 21474837264;
pub const KEYC_MOUSEUP2_CONTROL7: keyc = 21474837008;
pub const KEYC_MOUSEUP1_CONTROL7: keyc = 21474836752;
pub const KEYC_MOUSEUP_CONTROL7: keyc = 21474836496;
pub const KEYC_MOUSEUP11_CONTROL6: keyc = 21474839311;
pub const KEYC_MOUSEUP10_CONTROL6: keyc = 21474839055;
pub const KEYC_MOUSEUP9_CONTROL6: keyc = 21474838799;
pub const KEYC_MOUSEUP8_CONTROL6: keyc = 21474838543;
pub const KEYC_MOUSEUP7_CONTROL6: keyc = 21474838287;
pub const KEYC_MOUSEUP6_CONTROL6: keyc = 21474838031;
pub const KEYC_MOUSEUP3_CONTROL6: keyc = 21474837263;
pub const KEYC_MOUSEUP2_CONTROL6: keyc = 21474837007;
pub const KEYC_MOUSEUP1_CONTROL6: keyc = 21474836751;
pub const KEYC_MOUSEUP_CONTROL6: keyc = 21474836495;
pub const KEYC_MOUSEUP11_CONTROL5: keyc = 21474839310;
pub const KEYC_MOUSEUP10_CONTROL5: keyc = 21474839054;
pub const KEYC_MOUSEUP9_CONTROL5: keyc = 21474838798;
pub const KEYC_MOUSEUP8_CONTROL5: keyc = 21474838542;
pub const KEYC_MOUSEUP7_CONTROL5: keyc = 21474838286;
pub const KEYC_MOUSEUP6_CONTROL5: keyc = 21474838030;
pub const KEYC_MOUSEUP3_CONTROL5: keyc = 21474837262;
pub const KEYC_MOUSEUP2_CONTROL5: keyc = 21474837006;
pub const KEYC_MOUSEUP1_CONTROL5: keyc = 21474836750;
pub const KEYC_MOUSEUP_CONTROL5: keyc = 21474836494;
pub const KEYC_MOUSEUP11_CONTROL4: keyc = 21474839309;
pub const KEYC_MOUSEUP10_CONTROL4: keyc = 21474839053;
pub const KEYC_MOUSEUP9_CONTROL4: keyc = 21474838797;
pub const KEYC_MOUSEUP8_CONTROL4: keyc = 21474838541;
pub const KEYC_MOUSEUP7_CONTROL4: keyc = 21474838285;
pub const KEYC_MOUSEUP6_CONTROL4: keyc = 21474838029;
pub const KEYC_MOUSEUP3_CONTROL4: keyc = 21474837261;
pub const KEYC_MOUSEUP2_CONTROL4: keyc = 21474837005;
pub const KEYC_MOUSEUP1_CONTROL4: keyc = 21474836749;
pub const KEYC_MOUSEUP_CONTROL4: keyc = 21474836493;
pub const KEYC_MOUSEUP11_CONTROL3: keyc = 21474839308;
pub const KEYC_MOUSEUP10_CONTROL3: keyc = 21474839052;
pub const KEYC_MOUSEUP9_CONTROL3: keyc = 21474838796;
pub const KEYC_MOUSEUP8_CONTROL3: keyc = 21474838540;
pub const KEYC_MOUSEUP7_CONTROL3: keyc = 21474838284;
pub const KEYC_MOUSEUP6_CONTROL3: keyc = 21474838028;
pub const KEYC_MOUSEUP3_CONTROL3: keyc = 21474837260;
pub const KEYC_MOUSEUP2_CONTROL3: keyc = 21474837004;
pub const KEYC_MOUSEUP1_CONTROL3: keyc = 21474836748;
pub const KEYC_MOUSEUP_CONTROL3: keyc = 21474836492;
pub const KEYC_MOUSEUP11_CONTROL2: keyc = 21474839307;
pub const KEYC_MOUSEUP10_CONTROL2: keyc = 21474839051;
pub const KEYC_MOUSEUP9_CONTROL2: keyc = 21474838795;
pub const KEYC_MOUSEUP8_CONTROL2: keyc = 21474838539;
pub const KEYC_MOUSEUP7_CONTROL2: keyc = 21474838283;
pub const KEYC_MOUSEUP6_CONTROL2: keyc = 21474838027;
pub const KEYC_MOUSEUP3_CONTROL2: keyc = 21474837259;
pub const KEYC_MOUSEUP2_CONTROL2: keyc = 21474837003;
pub const KEYC_MOUSEUP1_CONTROL2: keyc = 21474836747;
pub const KEYC_MOUSEUP_CONTROL2: keyc = 21474836491;
pub const KEYC_MOUSEUP11_CONTROL1: keyc = 21474839306;
pub const KEYC_MOUSEUP10_CONTROL1: keyc = 21474839050;
pub const KEYC_MOUSEUP9_CONTROL1: keyc = 21474838794;
pub const KEYC_MOUSEUP8_CONTROL1: keyc = 21474838538;
pub const KEYC_MOUSEUP7_CONTROL1: keyc = 21474838282;
pub const KEYC_MOUSEUP6_CONTROL1: keyc = 21474838026;
pub const KEYC_MOUSEUP3_CONTROL1: keyc = 21474837258;
pub const KEYC_MOUSEUP2_CONTROL1: keyc = 21474837002;
pub const KEYC_MOUSEUP1_CONTROL1: keyc = 21474836746;
pub const KEYC_MOUSEUP_CONTROL1: keyc = 21474836490;
pub const KEYC_MOUSEUP11_CONTROL0: keyc = 21474839305;
pub const KEYC_MOUSEUP10_CONTROL0: keyc = 21474839049;
pub const KEYC_MOUSEUP9_CONTROL0: keyc = 21474838793;
pub const KEYC_MOUSEUP8_CONTROL0: keyc = 21474838537;
pub const KEYC_MOUSEUP7_CONTROL0: keyc = 21474838281;
pub const KEYC_MOUSEUP6_CONTROL0: keyc = 21474838025;
pub const KEYC_MOUSEUP3_CONTROL0: keyc = 21474837257;
pub const KEYC_MOUSEUP2_CONTROL0: keyc = 21474837001;
pub const KEYC_MOUSEUP1_CONTROL0: keyc = 21474836745;
pub const KEYC_MOUSEUP_CONTROL0: keyc = 21474836489;
pub const KEYC_MOUSEUP11_SCROLLBAR_DOWN: keyc = 21474839304;
pub const KEYC_MOUSEUP10_SCROLLBAR_DOWN: keyc = 21474839048;
pub const KEYC_MOUSEUP9_SCROLLBAR_DOWN: keyc = 21474838792;
pub const KEYC_MOUSEUP8_SCROLLBAR_DOWN: keyc = 21474838536;
pub const KEYC_MOUSEUP7_SCROLLBAR_DOWN: keyc = 21474838280;
pub const KEYC_MOUSEUP6_SCROLLBAR_DOWN: keyc = 21474838024;
pub const KEYC_MOUSEUP3_SCROLLBAR_DOWN: keyc = 21474837256;
pub const KEYC_MOUSEUP2_SCROLLBAR_DOWN: keyc = 21474837000;
pub const KEYC_MOUSEUP1_SCROLLBAR_DOWN: keyc = 21474836744;
pub const KEYC_MOUSEUP_SCROLLBAR_DOWN: keyc = 21474836488;
pub const KEYC_MOUSEUP11_SCROLLBAR_SLIDER: keyc = 21474839303;
pub const KEYC_MOUSEUP10_SCROLLBAR_SLIDER: keyc = 21474839047;
pub const KEYC_MOUSEUP9_SCROLLBAR_SLIDER: keyc = 21474838791;
pub const KEYC_MOUSEUP8_SCROLLBAR_SLIDER: keyc = 21474838535;
pub const KEYC_MOUSEUP7_SCROLLBAR_SLIDER: keyc = 21474838279;
pub const KEYC_MOUSEUP6_SCROLLBAR_SLIDER: keyc = 21474838023;
pub const KEYC_MOUSEUP3_SCROLLBAR_SLIDER: keyc = 21474837255;
pub const KEYC_MOUSEUP2_SCROLLBAR_SLIDER: keyc = 21474836999;
pub const KEYC_MOUSEUP1_SCROLLBAR_SLIDER: keyc = 21474836743;
pub const KEYC_MOUSEUP_SCROLLBAR_SLIDER: keyc = 21474836487;
pub const KEYC_MOUSEUP11_SCROLLBAR_UP: keyc = 21474839302;
pub const KEYC_MOUSEUP10_SCROLLBAR_UP: keyc = 21474839046;
pub const KEYC_MOUSEUP9_SCROLLBAR_UP: keyc = 21474838790;
pub const KEYC_MOUSEUP8_SCROLLBAR_UP: keyc = 21474838534;
pub const KEYC_MOUSEUP7_SCROLLBAR_UP: keyc = 21474838278;
pub const KEYC_MOUSEUP6_SCROLLBAR_UP: keyc = 21474838022;
pub const KEYC_MOUSEUP3_SCROLLBAR_UP: keyc = 21474837254;
pub const KEYC_MOUSEUP2_SCROLLBAR_UP: keyc = 21474836998;
pub const KEYC_MOUSEUP1_SCROLLBAR_UP: keyc = 21474836742;
pub const KEYC_MOUSEUP_SCROLLBAR_UP: keyc = 21474836486;
pub const KEYC_MOUSEUP11_BORDER: keyc = 21474839301;
pub const KEYC_MOUSEUP10_BORDER: keyc = 21474839045;
pub const KEYC_MOUSEUP9_BORDER: keyc = 21474838789;
pub const KEYC_MOUSEUP8_BORDER: keyc = 21474838533;
pub const KEYC_MOUSEUP7_BORDER: keyc = 21474838277;
pub const KEYC_MOUSEUP6_BORDER: keyc = 21474838021;
pub const KEYC_MOUSEUP3_BORDER: keyc = 21474837253;
pub const KEYC_MOUSEUP2_BORDER: keyc = 21474836997;
pub const KEYC_MOUSEUP1_BORDER: keyc = 21474836741;
pub const KEYC_MOUSEUP_BORDER: keyc = 21474836485;
pub const KEYC_MOUSEUP11_STATUS_DEFAULT: keyc = 21474839300;
pub const KEYC_MOUSEUP10_STATUS_DEFAULT: keyc = 21474839044;
pub const KEYC_MOUSEUP9_STATUS_DEFAULT: keyc = 21474838788;
pub const KEYC_MOUSEUP8_STATUS_DEFAULT: keyc = 21474838532;
pub const KEYC_MOUSEUP7_STATUS_DEFAULT: keyc = 21474838276;
pub const KEYC_MOUSEUP6_STATUS_DEFAULT: keyc = 21474838020;
pub const KEYC_MOUSEUP3_STATUS_DEFAULT: keyc = 21474837252;
pub const KEYC_MOUSEUP2_STATUS_DEFAULT: keyc = 21474836996;
pub const KEYC_MOUSEUP1_STATUS_DEFAULT: keyc = 21474836740;
pub const KEYC_MOUSEUP_STATUS_DEFAULT: keyc = 21474836484;
pub const KEYC_MOUSEUP11_STATUS_RIGHT: keyc = 21474839299;
pub const KEYC_MOUSEUP10_STATUS_RIGHT: keyc = 21474839043;
pub const KEYC_MOUSEUP9_STATUS_RIGHT: keyc = 21474838787;
pub const KEYC_MOUSEUP8_STATUS_RIGHT: keyc = 21474838531;
pub const KEYC_MOUSEUP7_STATUS_RIGHT: keyc = 21474838275;
pub const KEYC_MOUSEUP6_STATUS_RIGHT: keyc = 21474838019;
pub const KEYC_MOUSEUP3_STATUS_RIGHT: keyc = 21474837251;
pub const KEYC_MOUSEUP2_STATUS_RIGHT: keyc = 21474836995;
pub const KEYC_MOUSEUP1_STATUS_RIGHT: keyc = 21474836739;
pub const KEYC_MOUSEUP_STATUS_RIGHT: keyc = 21474836483;
pub const KEYC_MOUSEUP11_STATUS_LEFT: keyc = 21474839298;
pub const KEYC_MOUSEUP10_STATUS_LEFT: keyc = 21474839042;
pub const KEYC_MOUSEUP9_STATUS_LEFT: keyc = 21474838786;
pub const KEYC_MOUSEUP8_STATUS_LEFT: keyc = 21474838530;
pub const KEYC_MOUSEUP7_STATUS_LEFT: keyc = 21474838274;
pub const KEYC_MOUSEUP6_STATUS_LEFT: keyc = 21474838018;
pub const KEYC_MOUSEUP3_STATUS_LEFT: keyc = 21474837250;
pub const KEYC_MOUSEUP2_STATUS_LEFT: keyc = 21474836994;
pub const KEYC_MOUSEUP1_STATUS_LEFT: keyc = 21474836738;
pub const KEYC_MOUSEUP_STATUS_LEFT: keyc = 21474836482;
pub const KEYC_MOUSEUP11_STATUS: keyc = 21474839297;
pub const KEYC_MOUSEUP10_STATUS: keyc = 21474839041;
pub const KEYC_MOUSEUP9_STATUS: keyc = 21474838785;
pub const KEYC_MOUSEUP8_STATUS: keyc = 21474838529;
pub const KEYC_MOUSEUP7_STATUS: keyc = 21474838273;
pub const KEYC_MOUSEUP6_STATUS: keyc = 21474838017;
pub const KEYC_MOUSEUP3_STATUS: keyc = 21474837249;
pub const KEYC_MOUSEUP2_STATUS: keyc = 21474836993;
pub const KEYC_MOUSEUP1_STATUS: keyc = 21474836737;
pub const KEYC_MOUSEUP_STATUS: keyc = 21474836481;
pub const KEYC_MOUSEUP11_PANE: keyc = 21474839296;
pub const KEYC_MOUSEUP10_PANE: keyc = 21474839040;
pub const KEYC_MOUSEUP9_PANE: keyc = 21474838784;
pub const KEYC_MOUSEUP8_PANE: keyc = 21474838528;
pub const KEYC_MOUSEUP7_PANE: keyc = 21474838272;
pub const KEYC_MOUSEUP6_PANE: keyc = 21474838016;
pub const KEYC_MOUSEUP3_PANE: keyc = 21474837248;
pub const KEYC_MOUSEUP2_PANE: keyc = 21474836992;
pub const KEYC_MOUSEUP1_PANE: keyc = 21474836736;
pub const KEYC_MOUSEUP_PANE: keyc = 21474836480;
pub const KEYC_MOUSEDOWN11_CONTROL9: keyc = 17179872018;
pub const KEYC_MOUSEDOWN10_CONTROL9: keyc = 17179871762;
pub const KEYC_MOUSEDOWN9_CONTROL9: keyc = 17179871506;
pub const KEYC_MOUSEDOWN8_CONTROL9: keyc = 17179871250;
pub const KEYC_MOUSEDOWN7_CONTROL9: keyc = 17179870994;
pub const KEYC_MOUSEDOWN6_CONTROL9: keyc = 17179870738;
pub const KEYC_MOUSEDOWN3_CONTROL9: keyc = 17179869970;
pub const KEYC_MOUSEDOWN2_CONTROL9: keyc = 17179869714;
pub const KEYC_MOUSEDOWN1_CONTROL9: keyc = 17179869458;
pub const KEYC_MOUSEDOWN_CONTROL9: keyc = 17179869202;
pub const KEYC_MOUSEDOWN11_CONTROL8: keyc = 17179872017;
pub const KEYC_MOUSEDOWN10_CONTROL8: keyc = 17179871761;
pub const KEYC_MOUSEDOWN9_CONTROL8: keyc = 17179871505;
pub const KEYC_MOUSEDOWN8_CONTROL8: keyc = 17179871249;
pub const KEYC_MOUSEDOWN7_CONTROL8: keyc = 17179870993;
pub const KEYC_MOUSEDOWN6_CONTROL8: keyc = 17179870737;
pub const KEYC_MOUSEDOWN3_CONTROL8: keyc = 17179869969;
pub const KEYC_MOUSEDOWN2_CONTROL8: keyc = 17179869713;
pub const KEYC_MOUSEDOWN1_CONTROL8: keyc = 17179869457;
pub const KEYC_MOUSEDOWN_CONTROL8: keyc = 17179869201;
pub const KEYC_MOUSEDOWN11_CONTROL7: keyc = 17179872016;
pub const KEYC_MOUSEDOWN10_CONTROL7: keyc = 17179871760;
pub const KEYC_MOUSEDOWN9_CONTROL7: keyc = 17179871504;
pub const KEYC_MOUSEDOWN8_CONTROL7: keyc = 17179871248;
pub const KEYC_MOUSEDOWN7_CONTROL7: keyc = 17179870992;
pub const KEYC_MOUSEDOWN6_CONTROL7: keyc = 17179870736;
pub const KEYC_MOUSEDOWN3_CONTROL7: keyc = 17179869968;
pub const KEYC_MOUSEDOWN2_CONTROL7: keyc = 17179869712;
pub const KEYC_MOUSEDOWN1_CONTROL7: keyc = 17179869456;
pub const KEYC_MOUSEDOWN_CONTROL7: keyc = 17179869200;
pub const KEYC_MOUSEDOWN11_CONTROL6: keyc = 17179872015;
pub const KEYC_MOUSEDOWN10_CONTROL6: keyc = 17179871759;
pub const KEYC_MOUSEDOWN9_CONTROL6: keyc = 17179871503;
pub const KEYC_MOUSEDOWN8_CONTROL6: keyc = 17179871247;
pub const KEYC_MOUSEDOWN7_CONTROL6: keyc = 17179870991;
pub const KEYC_MOUSEDOWN6_CONTROL6: keyc = 17179870735;
pub const KEYC_MOUSEDOWN3_CONTROL6: keyc = 17179869967;
pub const KEYC_MOUSEDOWN2_CONTROL6: keyc = 17179869711;
pub const KEYC_MOUSEDOWN1_CONTROL6: keyc = 17179869455;
pub const KEYC_MOUSEDOWN_CONTROL6: keyc = 17179869199;
pub const KEYC_MOUSEDOWN11_CONTROL5: keyc = 17179872014;
pub const KEYC_MOUSEDOWN10_CONTROL5: keyc = 17179871758;
pub const KEYC_MOUSEDOWN9_CONTROL5: keyc = 17179871502;
pub const KEYC_MOUSEDOWN8_CONTROL5: keyc = 17179871246;
pub const KEYC_MOUSEDOWN7_CONTROL5: keyc = 17179870990;
pub const KEYC_MOUSEDOWN6_CONTROL5: keyc = 17179870734;
pub const KEYC_MOUSEDOWN3_CONTROL5: keyc = 17179869966;
pub const KEYC_MOUSEDOWN2_CONTROL5: keyc = 17179869710;
pub const KEYC_MOUSEDOWN1_CONTROL5: keyc = 17179869454;
pub const KEYC_MOUSEDOWN_CONTROL5: keyc = 17179869198;
pub const KEYC_MOUSEDOWN11_CONTROL4: keyc = 17179872013;
pub const KEYC_MOUSEDOWN10_CONTROL4: keyc = 17179871757;
pub const KEYC_MOUSEDOWN9_CONTROL4: keyc = 17179871501;
pub const KEYC_MOUSEDOWN8_CONTROL4: keyc = 17179871245;
pub const KEYC_MOUSEDOWN7_CONTROL4: keyc = 17179870989;
pub const KEYC_MOUSEDOWN6_CONTROL4: keyc = 17179870733;
pub const KEYC_MOUSEDOWN3_CONTROL4: keyc = 17179869965;
pub const KEYC_MOUSEDOWN2_CONTROL4: keyc = 17179869709;
pub const KEYC_MOUSEDOWN1_CONTROL4: keyc = 17179869453;
pub const KEYC_MOUSEDOWN_CONTROL4: keyc = 17179869197;
pub const KEYC_MOUSEDOWN11_CONTROL3: keyc = 17179872012;
pub const KEYC_MOUSEDOWN10_CONTROL3: keyc = 17179871756;
pub const KEYC_MOUSEDOWN9_CONTROL3: keyc = 17179871500;
pub const KEYC_MOUSEDOWN8_CONTROL3: keyc = 17179871244;
pub const KEYC_MOUSEDOWN7_CONTROL3: keyc = 17179870988;
pub const KEYC_MOUSEDOWN6_CONTROL3: keyc = 17179870732;
pub const KEYC_MOUSEDOWN3_CONTROL3: keyc = 17179869964;
pub const KEYC_MOUSEDOWN2_CONTROL3: keyc = 17179869708;
pub const KEYC_MOUSEDOWN1_CONTROL3: keyc = 17179869452;
pub const KEYC_MOUSEDOWN_CONTROL3: keyc = 17179869196;
pub const KEYC_MOUSEDOWN11_CONTROL2: keyc = 17179872011;
pub const KEYC_MOUSEDOWN10_CONTROL2: keyc = 17179871755;
pub const KEYC_MOUSEDOWN9_CONTROL2: keyc = 17179871499;
pub const KEYC_MOUSEDOWN8_CONTROL2: keyc = 17179871243;
pub const KEYC_MOUSEDOWN7_CONTROL2: keyc = 17179870987;
pub const KEYC_MOUSEDOWN6_CONTROL2: keyc = 17179870731;
pub const KEYC_MOUSEDOWN3_CONTROL2: keyc = 17179869963;
pub const KEYC_MOUSEDOWN2_CONTROL2: keyc = 17179869707;
pub const KEYC_MOUSEDOWN1_CONTROL2: keyc = 17179869451;
pub const KEYC_MOUSEDOWN_CONTROL2: keyc = 17179869195;
pub const KEYC_MOUSEDOWN11_CONTROL1: keyc = 17179872010;
pub const KEYC_MOUSEDOWN10_CONTROL1: keyc = 17179871754;
pub const KEYC_MOUSEDOWN9_CONTROL1: keyc = 17179871498;
pub const KEYC_MOUSEDOWN8_CONTROL1: keyc = 17179871242;
pub const KEYC_MOUSEDOWN7_CONTROL1: keyc = 17179870986;
pub const KEYC_MOUSEDOWN6_CONTROL1: keyc = 17179870730;
pub const KEYC_MOUSEDOWN3_CONTROL1: keyc = 17179869962;
pub const KEYC_MOUSEDOWN2_CONTROL1: keyc = 17179869706;
pub const KEYC_MOUSEDOWN1_CONTROL1: keyc = 17179869450;
pub const KEYC_MOUSEDOWN_CONTROL1: keyc = 17179869194;
pub const KEYC_MOUSEDOWN11_CONTROL0: keyc = 17179872009;
pub const KEYC_MOUSEDOWN10_CONTROL0: keyc = 17179871753;
pub const KEYC_MOUSEDOWN9_CONTROL0: keyc = 17179871497;
pub const KEYC_MOUSEDOWN8_CONTROL0: keyc = 17179871241;
pub const KEYC_MOUSEDOWN7_CONTROL0: keyc = 17179870985;
pub const KEYC_MOUSEDOWN6_CONTROL0: keyc = 17179870729;
pub const KEYC_MOUSEDOWN3_CONTROL0: keyc = 17179869961;
pub const KEYC_MOUSEDOWN2_CONTROL0: keyc = 17179869705;
pub const KEYC_MOUSEDOWN1_CONTROL0: keyc = 17179869449;
pub const KEYC_MOUSEDOWN_CONTROL0: keyc = 17179869193;
pub const KEYC_MOUSEDOWN11_SCROLLBAR_DOWN: keyc = 17179872008;
pub const KEYC_MOUSEDOWN10_SCROLLBAR_DOWN: keyc = 17179871752;
pub const KEYC_MOUSEDOWN9_SCROLLBAR_DOWN: keyc = 17179871496;
pub const KEYC_MOUSEDOWN8_SCROLLBAR_DOWN: keyc = 17179871240;
pub const KEYC_MOUSEDOWN7_SCROLLBAR_DOWN: keyc = 17179870984;
pub const KEYC_MOUSEDOWN6_SCROLLBAR_DOWN: keyc = 17179870728;
pub const KEYC_MOUSEDOWN3_SCROLLBAR_DOWN: keyc = 17179869960;
pub const KEYC_MOUSEDOWN2_SCROLLBAR_DOWN: keyc = 17179869704;
pub const KEYC_MOUSEDOWN1_SCROLLBAR_DOWN: keyc = 17179869448;
pub const KEYC_MOUSEDOWN_SCROLLBAR_DOWN: keyc = 17179869192;
pub const KEYC_MOUSEDOWN11_SCROLLBAR_SLIDER: keyc = 17179872007;
pub const KEYC_MOUSEDOWN10_SCROLLBAR_SLIDER: keyc = 17179871751;
pub const KEYC_MOUSEDOWN9_SCROLLBAR_SLIDER: keyc = 17179871495;
pub const KEYC_MOUSEDOWN8_SCROLLBAR_SLIDER: keyc = 17179871239;
pub const KEYC_MOUSEDOWN7_SCROLLBAR_SLIDER: keyc = 17179870983;
pub const KEYC_MOUSEDOWN6_SCROLLBAR_SLIDER: keyc = 17179870727;
pub const KEYC_MOUSEDOWN3_SCROLLBAR_SLIDER: keyc = 17179869959;
pub const KEYC_MOUSEDOWN2_SCROLLBAR_SLIDER: keyc = 17179869703;
pub const KEYC_MOUSEDOWN1_SCROLLBAR_SLIDER: keyc = 17179869447;
pub const KEYC_MOUSEDOWN_SCROLLBAR_SLIDER: keyc = 17179869191;
pub const KEYC_MOUSEDOWN11_SCROLLBAR_UP: keyc = 17179872006;
pub const KEYC_MOUSEDOWN10_SCROLLBAR_UP: keyc = 17179871750;
pub const KEYC_MOUSEDOWN9_SCROLLBAR_UP: keyc = 17179871494;
pub const KEYC_MOUSEDOWN8_SCROLLBAR_UP: keyc = 17179871238;
pub const KEYC_MOUSEDOWN7_SCROLLBAR_UP: keyc = 17179870982;
pub const KEYC_MOUSEDOWN6_SCROLLBAR_UP: keyc = 17179870726;
pub const KEYC_MOUSEDOWN3_SCROLLBAR_UP: keyc = 17179869958;
pub const KEYC_MOUSEDOWN2_SCROLLBAR_UP: keyc = 17179869702;
pub const KEYC_MOUSEDOWN1_SCROLLBAR_UP: keyc = 17179869446;
pub const KEYC_MOUSEDOWN_SCROLLBAR_UP: keyc = 17179869190;
pub const KEYC_MOUSEDOWN11_BORDER: keyc = 17179872005;
pub const KEYC_MOUSEDOWN10_BORDER: keyc = 17179871749;
pub const KEYC_MOUSEDOWN9_BORDER: keyc = 17179871493;
pub const KEYC_MOUSEDOWN8_BORDER: keyc = 17179871237;
pub const KEYC_MOUSEDOWN7_BORDER: keyc = 17179870981;
pub const KEYC_MOUSEDOWN6_BORDER: keyc = 17179870725;
pub const KEYC_MOUSEDOWN3_BORDER: keyc = 17179869957;
pub const KEYC_MOUSEDOWN2_BORDER: keyc = 17179869701;
pub const KEYC_MOUSEDOWN1_BORDER: keyc = 17179869445;
pub const KEYC_MOUSEDOWN_BORDER: keyc = 17179869189;
pub const KEYC_MOUSEDOWN11_STATUS_DEFAULT: keyc = 17179872004;
pub const KEYC_MOUSEDOWN10_STATUS_DEFAULT: keyc = 17179871748;
pub const KEYC_MOUSEDOWN9_STATUS_DEFAULT: keyc = 17179871492;
pub const KEYC_MOUSEDOWN8_STATUS_DEFAULT: keyc = 17179871236;
pub const KEYC_MOUSEDOWN7_STATUS_DEFAULT: keyc = 17179870980;
pub const KEYC_MOUSEDOWN6_STATUS_DEFAULT: keyc = 17179870724;
pub const KEYC_MOUSEDOWN3_STATUS_DEFAULT: keyc = 17179869956;
pub const KEYC_MOUSEDOWN2_STATUS_DEFAULT: keyc = 17179869700;
pub const KEYC_MOUSEDOWN1_STATUS_DEFAULT: keyc = 17179869444;
pub const KEYC_MOUSEDOWN_STATUS_DEFAULT: keyc = 17179869188;
pub const KEYC_MOUSEDOWN11_STATUS_RIGHT: keyc = 17179872003;
pub const KEYC_MOUSEDOWN10_STATUS_RIGHT: keyc = 17179871747;
pub const KEYC_MOUSEDOWN9_STATUS_RIGHT: keyc = 17179871491;
pub const KEYC_MOUSEDOWN8_STATUS_RIGHT: keyc = 17179871235;
pub const KEYC_MOUSEDOWN7_STATUS_RIGHT: keyc = 17179870979;
pub const KEYC_MOUSEDOWN6_STATUS_RIGHT: keyc = 17179870723;
pub const KEYC_MOUSEDOWN3_STATUS_RIGHT: keyc = 17179869955;
pub const KEYC_MOUSEDOWN2_STATUS_RIGHT: keyc = 17179869699;
pub const KEYC_MOUSEDOWN1_STATUS_RIGHT: keyc = 17179869443;
pub const KEYC_MOUSEDOWN_STATUS_RIGHT: keyc = 17179869187;
pub const KEYC_MOUSEDOWN11_STATUS_LEFT: keyc = 17179872002;
pub const KEYC_MOUSEDOWN10_STATUS_LEFT: keyc = 17179871746;
pub const KEYC_MOUSEDOWN9_STATUS_LEFT: keyc = 17179871490;
pub const KEYC_MOUSEDOWN8_STATUS_LEFT: keyc = 17179871234;
pub const KEYC_MOUSEDOWN7_STATUS_LEFT: keyc = 17179870978;
pub const KEYC_MOUSEDOWN6_STATUS_LEFT: keyc = 17179870722;
pub const KEYC_MOUSEDOWN3_STATUS_LEFT: keyc = 17179869954;
pub const KEYC_MOUSEDOWN2_STATUS_LEFT: keyc = 17179869698;
pub const KEYC_MOUSEDOWN1_STATUS_LEFT: keyc = 17179869442;
pub const KEYC_MOUSEDOWN_STATUS_LEFT: keyc = 17179869186;
pub const KEYC_MOUSEDOWN11_STATUS: keyc = 17179872001;
pub const KEYC_MOUSEDOWN10_STATUS: keyc = 17179871745;
pub const KEYC_MOUSEDOWN9_STATUS: keyc = 17179871489;
pub const KEYC_MOUSEDOWN8_STATUS: keyc = 17179871233;
pub const KEYC_MOUSEDOWN7_STATUS: keyc = 17179870977;
pub const KEYC_MOUSEDOWN6_STATUS: keyc = 17179870721;
pub const KEYC_MOUSEDOWN3_STATUS: keyc = 17179869953;
pub const KEYC_MOUSEDOWN2_STATUS: keyc = 17179869697;
pub const KEYC_MOUSEDOWN1_STATUS: keyc = 17179869441;
pub const KEYC_MOUSEDOWN_STATUS: keyc = 17179869185;
pub const KEYC_MOUSEDOWN11_PANE: keyc = 17179872000;
pub const KEYC_MOUSEDOWN10_PANE: keyc = 17179871744;
pub const KEYC_MOUSEDOWN9_PANE: keyc = 17179871488;
pub const KEYC_MOUSEDOWN8_PANE: keyc = 17179871232;
pub const KEYC_MOUSEDOWN7_PANE: keyc = 17179870976;
pub const KEYC_MOUSEDOWN6_PANE: keyc = 17179870720;
pub const KEYC_MOUSEDOWN3_PANE: keyc = 17179869952;
pub const KEYC_MOUSEDOWN2_PANE: keyc = 17179869696;
pub const KEYC_MOUSEDOWN1_PANE: keyc = 17179869440;
pub const KEYC_MOUSEDOWN_PANE: keyc = 17179869184;
pub const KEYC_WHEELUP11_CONTROL9: keyc = 38654708498;
pub const KEYC_WHEELUP10_CONTROL9: keyc = 38654708242;
pub const KEYC_WHEELUP9_CONTROL9: keyc = 38654707986;
pub const KEYC_WHEELUP8_CONTROL9: keyc = 38654707730;
pub const KEYC_WHEELUP7_CONTROL9: keyc = 38654707474;
pub const KEYC_WHEELUP6_CONTROL9: keyc = 38654707218;
pub const KEYC_WHEELUP3_CONTROL9: keyc = 38654706450;
pub const KEYC_WHEELUP2_CONTROL9: keyc = 38654706194;
pub const KEYC_WHEELUP1_CONTROL9: keyc = 38654705938;
pub const KEYC_WHEELUP_CONTROL9: keyc = 38654705682;
pub const KEYC_WHEELUP11_CONTROL8: keyc = 38654708497;
pub const KEYC_WHEELUP10_CONTROL8: keyc = 38654708241;
pub const KEYC_WHEELUP9_CONTROL8: keyc = 38654707985;
pub const KEYC_WHEELUP8_CONTROL8: keyc = 38654707729;
pub const KEYC_WHEELUP7_CONTROL8: keyc = 38654707473;
pub const KEYC_WHEELUP6_CONTROL8: keyc = 38654707217;
pub const KEYC_WHEELUP3_CONTROL8: keyc = 38654706449;
pub const KEYC_WHEELUP2_CONTROL8: keyc = 38654706193;
pub const KEYC_WHEELUP1_CONTROL8: keyc = 38654705937;
pub const KEYC_WHEELUP_CONTROL8: keyc = 38654705681;
pub const KEYC_WHEELUP11_CONTROL7: keyc = 38654708496;
pub const KEYC_WHEELUP10_CONTROL7: keyc = 38654708240;
pub const KEYC_WHEELUP9_CONTROL7: keyc = 38654707984;
pub const KEYC_WHEELUP8_CONTROL7: keyc = 38654707728;
pub const KEYC_WHEELUP7_CONTROL7: keyc = 38654707472;
pub const KEYC_WHEELUP6_CONTROL7: keyc = 38654707216;
pub const KEYC_WHEELUP3_CONTROL7: keyc = 38654706448;
pub const KEYC_WHEELUP2_CONTROL7: keyc = 38654706192;
pub const KEYC_WHEELUP1_CONTROL7: keyc = 38654705936;
pub const KEYC_WHEELUP_CONTROL7: keyc = 38654705680;
pub const KEYC_WHEELUP11_CONTROL6: keyc = 38654708495;
pub const KEYC_WHEELUP10_CONTROL6: keyc = 38654708239;
pub const KEYC_WHEELUP9_CONTROL6: keyc = 38654707983;
pub const KEYC_WHEELUP8_CONTROL6: keyc = 38654707727;
pub const KEYC_WHEELUP7_CONTROL6: keyc = 38654707471;
pub const KEYC_WHEELUP6_CONTROL6: keyc = 38654707215;
pub const KEYC_WHEELUP3_CONTROL6: keyc = 38654706447;
pub const KEYC_WHEELUP2_CONTROL6: keyc = 38654706191;
pub const KEYC_WHEELUP1_CONTROL6: keyc = 38654705935;
pub const KEYC_WHEELUP_CONTROL6: keyc = 38654705679;
pub const KEYC_WHEELUP11_CONTROL5: keyc = 38654708494;
pub const KEYC_WHEELUP10_CONTROL5: keyc = 38654708238;
pub const KEYC_WHEELUP9_CONTROL5: keyc = 38654707982;
pub const KEYC_WHEELUP8_CONTROL5: keyc = 38654707726;
pub const KEYC_WHEELUP7_CONTROL5: keyc = 38654707470;
pub const KEYC_WHEELUP6_CONTROL5: keyc = 38654707214;
pub const KEYC_WHEELUP3_CONTROL5: keyc = 38654706446;
pub const KEYC_WHEELUP2_CONTROL5: keyc = 38654706190;
pub const KEYC_WHEELUP1_CONTROL5: keyc = 38654705934;
pub const KEYC_WHEELUP_CONTROL5: keyc = 38654705678;
pub const KEYC_WHEELUP11_CONTROL4: keyc = 38654708493;
pub const KEYC_WHEELUP10_CONTROL4: keyc = 38654708237;
pub const KEYC_WHEELUP9_CONTROL4: keyc = 38654707981;
pub const KEYC_WHEELUP8_CONTROL4: keyc = 38654707725;
pub const KEYC_WHEELUP7_CONTROL4: keyc = 38654707469;
pub const KEYC_WHEELUP6_CONTROL4: keyc = 38654707213;
pub const KEYC_WHEELUP3_CONTROL4: keyc = 38654706445;
pub const KEYC_WHEELUP2_CONTROL4: keyc = 38654706189;
pub const KEYC_WHEELUP1_CONTROL4: keyc = 38654705933;
pub const KEYC_WHEELUP_CONTROL4: keyc = 38654705677;
pub const KEYC_WHEELUP11_CONTROL3: keyc = 38654708492;
pub const KEYC_WHEELUP10_CONTROL3: keyc = 38654708236;
pub const KEYC_WHEELUP9_CONTROL3: keyc = 38654707980;
pub const KEYC_WHEELUP8_CONTROL3: keyc = 38654707724;
pub const KEYC_WHEELUP7_CONTROL3: keyc = 38654707468;
pub const KEYC_WHEELUP6_CONTROL3: keyc = 38654707212;
pub const KEYC_WHEELUP3_CONTROL3: keyc = 38654706444;
pub const KEYC_WHEELUP2_CONTROL3: keyc = 38654706188;
pub const KEYC_WHEELUP1_CONTROL3: keyc = 38654705932;
pub const KEYC_WHEELUP_CONTROL3: keyc = 38654705676;
pub const KEYC_WHEELUP11_CONTROL2: keyc = 38654708491;
pub const KEYC_WHEELUP10_CONTROL2: keyc = 38654708235;
pub const KEYC_WHEELUP9_CONTROL2: keyc = 38654707979;
pub const KEYC_WHEELUP8_CONTROL2: keyc = 38654707723;
pub const KEYC_WHEELUP7_CONTROL2: keyc = 38654707467;
pub const KEYC_WHEELUP6_CONTROL2: keyc = 38654707211;
pub const KEYC_WHEELUP3_CONTROL2: keyc = 38654706443;
pub const KEYC_WHEELUP2_CONTROL2: keyc = 38654706187;
pub const KEYC_WHEELUP1_CONTROL2: keyc = 38654705931;
pub const KEYC_WHEELUP_CONTROL2: keyc = 38654705675;
pub const KEYC_WHEELUP11_CONTROL1: keyc = 38654708490;
pub const KEYC_WHEELUP10_CONTROL1: keyc = 38654708234;
pub const KEYC_WHEELUP9_CONTROL1: keyc = 38654707978;
pub const KEYC_WHEELUP8_CONTROL1: keyc = 38654707722;
pub const KEYC_WHEELUP7_CONTROL1: keyc = 38654707466;
pub const KEYC_WHEELUP6_CONTROL1: keyc = 38654707210;
pub const KEYC_WHEELUP3_CONTROL1: keyc = 38654706442;
pub const KEYC_WHEELUP2_CONTROL1: keyc = 38654706186;
pub const KEYC_WHEELUP1_CONTROL1: keyc = 38654705930;
pub const KEYC_WHEELUP_CONTROL1: keyc = 38654705674;
pub const KEYC_WHEELUP11_CONTROL0: keyc = 38654708489;
pub const KEYC_WHEELUP10_CONTROL0: keyc = 38654708233;
pub const KEYC_WHEELUP9_CONTROL0: keyc = 38654707977;
pub const KEYC_WHEELUP8_CONTROL0: keyc = 38654707721;
pub const KEYC_WHEELUP7_CONTROL0: keyc = 38654707465;
pub const KEYC_WHEELUP6_CONTROL0: keyc = 38654707209;
pub const KEYC_WHEELUP3_CONTROL0: keyc = 38654706441;
pub const KEYC_WHEELUP2_CONTROL0: keyc = 38654706185;
pub const KEYC_WHEELUP1_CONTROL0: keyc = 38654705929;
pub const KEYC_WHEELUP_CONTROL0: keyc = 38654705673;
pub const KEYC_WHEELUP11_SCROLLBAR_DOWN: keyc = 38654708488;
pub const KEYC_WHEELUP10_SCROLLBAR_DOWN: keyc = 38654708232;
pub const KEYC_WHEELUP9_SCROLLBAR_DOWN: keyc = 38654707976;
pub const KEYC_WHEELUP8_SCROLLBAR_DOWN: keyc = 38654707720;
pub const KEYC_WHEELUP7_SCROLLBAR_DOWN: keyc = 38654707464;
pub const KEYC_WHEELUP6_SCROLLBAR_DOWN: keyc = 38654707208;
pub const KEYC_WHEELUP3_SCROLLBAR_DOWN: keyc = 38654706440;
pub const KEYC_WHEELUP2_SCROLLBAR_DOWN: keyc = 38654706184;
pub const KEYC_WHEELUP1_SCROLLBAR_DOWN: keyc = 38654705928;
pub const KEYC_WHEELUP_SCROLLBAR_DOWN: keyc = 38654705672;
pub const KEYC_WHEELUP11_SCROLLBAR_SLIDER: keyc = 38654708487;
pub const KEYC_WHEELUP10_SCROLLBAR_SLIDER: keyc = 38654708231;
pub const KEYC_WHEELUP9_SCROLLBAR_SLIDER: keyc = 38654707975;
pub const KEYC_WHEELUP8_SCROLLBAR_SLIDER: keyc = 38654707719;
pub const KEYC_WHEELUP7_SCROLLBAR_SLIDER: keyc = 38654707463;
pub const KEYC_WHEELUP6_SCROLLBAR_SLIDER: keyc = 38654707207;
pub const KEYC_WHEELUP3_SCROLLBAR_SLIDER: keyc = 38654706439;
pub const KEYC_WHEELUP2_SCROLLBAR_SLIDER: keyc = 38654706183;
pub const KEYC_WHEELUP1_SCROLLBAR_SLIDER: keyc = 38654705927;
pub const KEYC_WHEELUP_SCROLLBAR_SLIDER: keyc = 38654705671;
pub const KEYC_WHEELUP11_SCROLLBAR_UP: keyc = 38654708486;
pub const KEYC_WHEELUP10_SCROLLBAR_UP: keyc = 38654708230;
pub const KEYC_WHEELUP9_SCROLLBAR_UP: keyc = 38654707974;
pub const KEYC_WHEELUP8_SCROLLBAR_UP: keyc = 38654707718;
pub const KEYC_WHEELUP7_SCROLLBAR_UP: keyc = 38654707462;
pub const KEYC_WHEELUP6_SCROLLBAR_UP: keyc = 38654707206;
pub const KEYC_WHEELUP3_SCROLLBAR_UP: keyc = 38654706438;
pub const KEYC_WHEELUP2_SCROLLBAR_UP: keyc = 38654706182;
pub const KEYC_WHEELUP1_SCROLLBAR_UP: keyc = 38654705926;
pub const KEYC_WHEELUP_SCROLLBAR_UP: keyc = 38654705670;
pub const KEYC_WHEELUP11_BORDER: keyc = 38654708485;
pub const KEYC_WHEELUP10_BORDER: keyc = 38654708229;
pub const KEYC_WHEELUP9_BORDER: keyc = 38654707973;
pub const KEYC_WHEELUP8_BORDER: keyc = 38654707717;
pub const KEYC_WHEELUP7_BORDER: keyc = 38654707461;
pub const KEYC_WHEELUP6_BORDER: keyc = 38654707205;
pub const KEYC_WHEELUP3_BORDER: keyc = 38654706437;
pub const KEYC_WHEELUP2_BORDER: keyc = 38654706181;
pub const KEYC_WHEELUP1_BORDER: keyc = 38654705925;
pub const KEYC_WHEELUP_BORDER: keyc = 38654705669;
pub const KEYC_WHEELUP11_STATUS_DEFAULT: keyc = 38654708484;
pub const KEYC_WHEELUP10_STATUS_DEFAULT: keyc = 38654708228;
pub const KEYC_WHEELUP9_STATUS_DEFAULT: keyc = 38654707972;
pub const KEYC_WHEELUP8_STATUS_DEFAULT: keyc = 38654707716;
pub const KEYC_WHEELUP7_STATUS_DEFAULT: keyc = 38654707460;
pub const KEYC_WHEELUP6_STATUS_DEFAULT: keyc = 38654707204;
pub const KEYC_WHEELUP3_STATUS_DEFAULT: keyc = 38654706436;
pub const KEYC_WHEELUP2_STATUS_DEFAULT: keyc = 38654706180;
pub const KEYC_WHEELUP1_STATUS_DEFAULT: keyc = 38654705924;
pub const KEYC_WHEELUP_STATUS_DEFAULT: keyc = 38654705668;
pub const KEYC_WHEELUP11_STATUS_RIGHT: keyc = 38654708483;
pub const KEYC_WHEELUP10_STATUS_RIGHT: keyc = 38654708227;
pub const KEYC_WHEELUP9_STATUS_RIGHT: keyc = 38654707971;
pub const KEYC_WHEELUP8_STATUS_RIGHT: keyc = 38654707715;
pub const KEYC_WHEELUP7_STATUS_RIGHT: keyc = 38654707459;
pub const KEYC_WHEELUP6_STATUS_RIGHT: keyc = 38654707203;
pub const KEYC_WHEELUP3_STATUS_RIGHT: keyc = 38654706435;
pub const KEYC_WHEELUP2_STATUS_RIGHT: keyc = 38654706179;
pub const KEYC_WHEELUP1_STATUS_RIGHT: keyc = 38654705923;
pub const KEYC_WHEELUP_STATUS_RIGHT: keyc = 38654705667;
pub const KEYC_WHEELUP11_STATUS_LEFT: keyc = 38654708482;
pub const KEYC_WHEELUP10_STATUS_LEFT: keyc = 38654708226;
pub const KEYC_WHEELUP9_STATUS_LEFT: keyc = 38654707970;
pub const KEYC_WHEELUP8_STATUS_LEFT: keyc = 38654707714;
pub const KEYC_WHEELUP7_STATUS_LEFT: keyc = 38654707458;
pub const KEYC_WHEELUP6_STATUS_LEFT: keyc = 38654707202;
pub const KEYC_WHEELUP3_STATUS_LEFT: keyc = 38654706434;
pub const KEYC_WHEELUP2_STATUS_LEFT: keyc = 38654706178;
pub const KEYC_WHEELUP1_STATUS_LEFT: keyc = 38654705922;
pub const KEYC_WHEELUP_STATUS_LEFT: keyc = 38654705666;
pub const KEYC_WHEELUP11_STATUS: keyc = 38654708481;
pub const KEYC_WHEELUP10_STATUS: keyc = 38654708225;
pub const KEYC_WHEELUP9_STATUS: keyc = 38654707969;
pub const KEYC_WHEELUP8_STATUS: keyc = 38654707713;
pub const KEYC_WHEELUP7_STATUS: keyc = 38654707457;
pub const KEYC_WHEELUP6_STATUS: keyc = 38654707201;
pub const KEYC_WHEELUP3_STATUS: keyc = 38654706433;
pub const KEYC_WHEELUP2_STATUS: keyc = 38654706177;
pub const KEYC_WHEELUP1_STATUS: keyc = 38654705921;
pub const KEYC_WHEELUP_STATUS: keyc = 38654705665;
pub const KEYC_WHEELUP11_PANE: keyc = 38654708480;
pub const KEYC_WHEELUP10_PANE: keyc = 38654708224;
pub const KEYC_WHEELUP9_PANE: keyc = 38654707968;
pub const KEYC_WHEELUP8_PANE: keyc = 38654707712;
pub const KEYC_WHEELUP7_PANE: keyc = 38654707456;
pub const KEYC_WHEELUP6_PANE: keyc = 38654707200;
pub const KEYC_WHEELUP3_PANE: keyc = 38654706432;
pub const KEYC_WHEELUP2_PANE: keyc = 38654706176;
pub const KEYC_WHEELUP1_PANE: keyc = 38654705920;
pub const KEYC_WHEELUP_PANE: keyc = 38654705664;
pub const KEYC_WHEELDOWN11_CONTROL9: keyc = 34359741202;
pub const KEYC_WHEELDOWN10_CONTROL9: keyc = 34359740946;
pub const KEYC_WHEELDOWN9_CONTROL9: keyc = 34359740690;
pub const KEYC_WHEELDOWN8_CONTROL9: keyc = 34359740434;
pub const KEYC_WHEELDOWN7_CONTROL9: keyc = 34359740178;
pub const KEYC_WHEELDOWN6_CONTROL9: keyc = 34359739922;
pub const KEYC_WHEELDOWN3_CONTROL9: keyc = 34359739154;
pub const KEYC_WHEELDOWN2_CONTROL9: keyc = 34359738898;
pub const KEYC_WHEELDOWN1_CONTROL9: keyc = 34359738642;
pub const KEYC_WHEELDOWN_CONTROL9: keyc = 34359738386;
pub const KEYC_WHEELDOWN11_CONTROL8: keyc = 34359741201;
pub const KEYC_WHEELDOWN10_CONTROL8: keyc = 34359740945;
pub const KEYC_WHEELDOWN9_CONTROL8: keyc = 34359740689;
pub const KEYC_WHEELDOWN8_CONTROL8: keyc = 34359740433;
pub const KEYC_WHEELDOWN7_CONTROL8: keyc = 34359740177;
pub const KEYC_WHEELDOWN6_CONTROL8: keyc = 34359739921;
pub const KEYC_WHEELDOWN3_CONTROL8: keyc = 34359739153;
pub const KEYC_WHEELDOWN2_CONTROL8: keyc = 34359738897;
pub const KEYC_WHEELDOWN1_CONTROL8: keyc = 34359738641;
pub const KEYC_WHEELDOWN_CONTROL8: keyc = 34359738385;
pub const KEYC_WHEELDOWN11_CONTROL7: keyc = 34359741200;
pub const KEYC_WHEELDOWN10_CONTROL7: keyc = 34359740944;
pub const KEYC_WHEELDOWN9_CONTROL7: keyc = 34359740688;
pub const KEYC_WHEELDOWN8_CONTROL7: keyc = 34359740432;
pub const KEYC_WHEELDOWN7_CONTROL7: keyc = 34359740176;
pub const KEYC_WHEELDOWN6_CONTROL7: keyc = 34359739920;
pub const KEYC_WHEELDOWN3_CONTROL7: keyc = 34359739152;
pub const KEYC_WHEELDOWN2_CONTROL7: keyc = 34359738896;
pub const KEYC_WHEELDOWN1_CONTROL7: keyc = 34359738640;
pub const KEYC_WHEELDOWN_CONTROL7: keyc = 34359738384;
pub const KEYC_WHEELDOWN11_CONTROL6: keyc = 34359741199;
pub const KEYC_WHEELDOWN10_CONTROL6: keyc = 34359740943;
pub const KEYC_WHEELDOWN9_CONTROL6: keyc = 34359740687;
pub const KEYC_WHEELDOWN8_CONTROL6: keyc = 34359740431;
pub const KEYC_WHEELDOWN7_CONTROL6: keyc = 34359740175;
pub const KEYC_WHEELDOWN6_CONTROL6: keyc = 34359739919;
pub const KEYC_WHEELDOWN3_CONTROL6: keyc = 34359739151;
pub const KEYC_WHEELDOWN2_CONTROL6: keyc = 34359738895;
pub const KEYC_WHEELDOWN1_CONTROL6: keyc = 34359738639;
pub const KEYC_WHEELDOWN_CONTROL6: keyc = 34359738383;
pub const KEYC_WHEELDOWN11_CONTROL5: keyc = 34359741198;
pub const KEYC_WHEELDOWN10_CONTROL5: keyc = 34359740942;
pub const KEYC_WHEELDOWN9_CONTROL5: keyc = 34359740686;
pub const KEYC_WHEELDOWN8_CONTROL5: keyc = 34359740430;
pub const KEYC_WHEELDOWN7_CONTROL5: keyc = 34359740174;
pub const KEYC_WHEELDOWN6_CONTROL5: keyc = 34359739918;
pub const KEYC_WHEELDOWN3_CONTROL5: keyc = 34359739150;
pub const KEYC_WHEELDOWN2_CONTROL5: keyc = 34359738894;
pub const KEYC_WHEELDOWN1_CONTROL5: keyc = 34359738638;
pub const KEYC_WHEELDOWN_CONTROL5: keyc = 34359738382;
pub const KEYC_WHEELDOWN11_CONTROL4: keyc = 34359741197;
pub const KEYC_WHEELDOWN10_CONTROL4: keyc = 34359740941;
pub const KEYC_WHEELDOWN9_CONTROL4: keyc = 34359740685;
pub const KEYC_WHEELDOWN8_CONTROL4: keyc = 34359740429;
pub const KEYC_WHEELDOWN7_CONTROL4: keyc = 34359740173;
pub const KEYC_WHEELDOWN6_CONTROL4: keyc = 34359739917;
pub const KEYC_WHEELDOWN3_CONTROL4: keyc = 34359739149;
pub const KEYC_WHEELDOWN2_CONTROL4: keyc = 34359738893;
pub const KEYC_WHEELDOWN1_CONTROL4: keyc = 34359738637;
pub const KEYC_WHEELDOWN_CONTROL4: keyc = 34359738381;
pub const KEYC_WHEELDOWN11_CONTROL3: keyc = 34359741196;
pub const KEYC_WHEELDOWN10_CONTROL3: keyc = 34359740940;
pub const KEYC_WHEELDOWN9_CONTROL3: keyc = 34359740684;
pub const KEYC_WHEELDOWN8_CONTROL3: keyc = 34359740428;
pub const KEYC_WHEELDOWN7_CONTROL3: keyc = 34359740172;
pub const KEYC_WHEELDOWN6_CONTROL3: keyc = 34359739916;
pub const KEYC_WHEELDOWN3_CONTROL3: keyc = 34359739148;
pub const KEYC_WHEELDOWN2_CONTROL3: keyc = 34359738892;
pub const KEYC_WHEELDOWN1_CONTROL3: keyc = 34359738636;
pub const KEYC_WHEELDOWN_CONTROL3: keyc = 34359738380;
pub const KEYC_WHEELDOWN11_CONTROL2: keyc = 34359741195;
pub const KEYC_WHEELDOWN10_CONTROL2: keyc = 34359740939;
pub const KEYC_WHEELDOWN9_CONTROL2: keyc = 34359740683;
pub const KEYC_WHEELDOWN8_CONTROL2: keyc = 34359740427;
pub const KEYC_WHEELDOWN7_CONTROL2: keyc = 34359740171;
pub const KEYC_WHEELDOWN6_CONTROL2: keyc = 34359739915;
pub const KEYC_WHEELDOWN3_CONTROL2: keyc = 34359739147;
pub const KEYC_WHEELDOWN2_CONTROL2: keyc = 34359738891;
pub const KEYC_WHEELDOWN1_CONTROL2: keyc = 34359738635;
pub const KEYC_WHEELDOWN_CONTROL2: keyc = 34359738379;
pub const KEYC_WHEELDOWN11_CONTROL1: keyc = 34359741194;
pub const KEYC_WHEELDOWN10_CONTROL1: keyc = 34359740938;
pub const KEYC_WHEELDOWN9_CONTROL1: keyc = 34359740682;
pub const KEYC_WHEELDOWN8_CONTROL1: keyc = 34359740426;
pub const KEYC_WHEELDOWN7_CONTROL1: keyc = 34359740170;
pub const KEYC_WHEELDOWN6_CONTROL1: keyc = 34359739914;
pub const KEYC_WHEELDOWN3_CONTROL1: keyc = 34359739146;
pub const KEYC_WHEELDOWN2_CONTROL1: keyc = 34359738890;
pub const KEYC_WHEELDOWN1_CONTROL1: keyc = 34359738634;
pub const KEYC_WHEELDOWN_CONTROL1: keyc = 34359738378;
pub const KEYC_WHEELDOWN11_CONTROL0: keyc = 34359741193;
pub const KEYC_WHEELDOWN10_CONTROL0: keyc = 34359740937;
pub const KEYC_WHEELDOWN9_CONTROL0: keyc = 34359740681;
pub const KEYC_WHEELDOWN8_CONTROL0: keyc = 34359740425;
pub const KEYC_WHEELDOWN7_CONTROL0: keyc = 34359740169;
pub const KEYC_WHEELDOWN6_CONTROL0: keyc = 34359739913;
pub const KEYC_WHEELDOWN3_CONTROL0: keyc = 34359739145;
pub const KEYC_WHEELDOWN2_CONTROL0: keyc = 34359738889;
pub const KEYC_WHEELDOWN1_CONTROL0: keyc = 34359738633;
pub const KEYC_WHEELDOWN_CONTROL0: keyc = 34359738377;
pub const KEYC_WHEELDOWN11_SCROLLBAR_DOWN: keyc = 34359741192;
pub const KEYC_WHEELDOWN10_SCROLLBAR_DOWN: keyc = 34359740936;
pub const KEYC_WHEELDOWN9_SCROLLBAR_DOWN: keyc = 34359740680;
pub const KEYC_WHEELDOWN8_SCROLLBAR_DOWN: keyc = 34359740424;
pub const KEYC_WHEELDOWN7_SCROLLBAR_DOWN: keyc = 34359740168;
pub const KEYC_WHEELDOWN6_SCROLLBAR_DOWN: keyc = 34359739912;
pub const KEYC_WHEELDOWN3_SCROLLBAR_DOWN: keyc = 34359739144;
pub const KEYC_WHEELDOWN2_SCROLLBAR_DOWN: keyc = 34359738888;
pub const KEYC_WHEELDOWN1_SCROLLBAR_DOWN: keyc = 34359738632;
pub const KEYC_WHEELDOWN_SCROLLBAR_DOWN: keyc = 34359738376;
pub const KEYC_WHEELDOWN11_SCROLLBAR_SLIDER: keyc = 34359741191;
pub const KEYC_WHEELDOWN10_SCROLLBAR_SLIDER: keyc = 34359740935;
pub const KEYC_WHEELDOWN9_SCROLLBAR_SLIDER: keyc = 34359740679;
pub const KEYC_WHEELDOWN8_SCROLLBAR_SLIDER: keyc = 34359740423;
pub const KEYC_WHEELDOWN7_SCROLLBAR_SLIDER: keyc = 34359740167;
pub const KEYC_WHEELDOWN6_SCROLLBAR_SLIDER: keyc = 34359739911;
pub const KEYC_WHEELDOWN3_SCROLLBAR_SLIDER: keyc = 34359739143;
pub const KEYC_WHEELDOWN2_SCROLLBAR_SLIDER: keyc = 34359738887;
pub const KEYC_WHEELDOWN1_SCROLLBAR_SLIDER: keyc = 34359738631;
pub const KEYC_WHEELDOWN_SCROLLBAR_SLIDER: keyc = 34359738375;
pub const KEYC_WHEELDOWN11_SCROLLBAR_UP: keyc = 34359741190;
pub const KEYC_WHEELDOWN10_SCROLLBAR_UP: keyc = 34359740934;
pub const KEYC_WHEELDOWN9_SCROLLBAR_UP: keyc = 34359740678;
pub const KEYC_WHEELDOWN8_SCROLLBAR_UP: keyc = 34359740422;
pub const KEYC_WHEELDOWN7_SCROLLBAR_UP: keyc = 34359740166;
pub const KEYC_WHEELDOWN6_SCROLLBAR_UP: keyc = 34359739910;
pub const KEYC_WHEELDOWN3_SCROLLBAR_UP: keyc = 34359739142;
pub const KEYC_WHEELDOWN2_SCROLLBAR_UP: keyc = 34359738886;
pub const KEYC_WHEELDOWN1_SCROLLBAR_UP: keyc = 34359738630;
pub const KEYC_WHEELDOWN_SCROLLBAR_UP: keyc = 34359738374;
pub const KEYC_WHEELDOWN11_BORDER: keyc = 34359741189;
pub const KEYC_WHEELDOWN10_BORDER: keyc = 34359740933;
pub const KEYC_WHEELDOWN9_BORDER: keyc = 34359740677;
pub const KEYC_WHEELDOWN8_BORDER: keyc = 34359740421;
pub const KEYC_WHEELDOWN7_BORDER: keyc = 34359740165;
pub const KEYC_WHEELDOWN6_BORDER: keyc = 34359739909;
pub const KEYC_WHEELDOWN3_BORDER: keyc = 34359739141;
pub const KEYC_WHEELDOWN2_BORDER: keyc = 34359738885;
pub const KEYC_WHEELDOWN1_BORDER: keyc = 34359738629;
pub const KEYC_WHEELDOWN_BORDER: keyc = 34359738373;
pub const KEYC_WHEELDOWN11_STATUS_DEFAULT: keyc = 34359741188;
pub const KEYC_WHEELDOWN10_STATUS_DEFAULT: keyc = 34359740932;
pub const KEYC_WHEELDOWN9_STATUS_DEFAULT: keyc = 34359740676;
pub const KEYC_WHEELDOWN8_STATUS_DEFAULT: keyc = 34359740420;
pub const KEYC_WHEELDOWN7_STATUS_DEFAULT: keyc = 34359740164;
pub const KEYC_WHEELDOWN6_STATUS_DEFAULT: keyc = 34359739908;
pub const KEYC_WHEELDOWN3_STATUS_DEFAULT: keyc = 34359739140;
pub const KEYC_WHEELDOWN2_STATUS_DEFAULT: keyc = 34359738884;
pub const KEYC_WHEELDOWN1_STATUS_DEFAULT: keyc = 34359738628;
pub const KEYC_WHEELDOWN_STATUS_DEFAULT: keyc = 34359738372;
pub const KEYC_WHEELDOWN11_STATUS_RIGHT: keyc = 34359741187;
pub const KEYC_WHEELDOWN10_STATUS_RIGHT: keyc = 34359740931;
pub const KEYC_WHEELDOWN9_STATUS_RIGHT: keyc = 34359740675;
pub const KEYC_WHEELDOWN8_STATUS_RIGHT: keyc = 34359740419;
pub const KEYC_WHEELDOWN7_STATUS_RIGHT: keyc = 34359740163;
pub const KEYC_WHEELDOWN6_STATUS_RIGHT: keyc = 34359739907;
pub const KEYC_WHEELDOWN3_STATUS_RIGHT: keyc = 34359739139;
pub const KEYC_WHEELDOWN2_STATUS_RIGHT: keyc = 34359738883;
pub const KEYC_WHEELDOWN1_STATUS_RIGHT: keyc = 34359738627;
pub const KEYC_WHEELDOWN_STATUS_RIGHT: keyc = 34359738371;
pub const KEYC_WHEELDOWN11_STATUS_LEFT: keyc = 34359741186;
pub const KEYC_WHEELDOWN10_STATUS_LEFT: keyc = 34359740930;
pub const KEYC_WHEELDOWN9_STATUS_LEFT: keyc = 34359740674;
pub const KEYC_WHEELDOWN8_STATUS_LEFT: keyc = 34359740418;
pub const KEYC_WHEELDOWN7_STATUS_LEFT: keyc = 34359740162;
pub const KEYC_WHEELDOWN6_STATUS_LEFT: keyc = 34359739906;
pub const KEYC_WHEELDOWN3_STATUS_LEFT: keyc = 34359739138;
pub const KEYC_WHEELDOWN2_STATUS_LEFT: keyc = 34359738882;
pub const KEYC_WHEELDOWN1_STATUS_LEFT: keyc = 34359738626;
pub const KEYC_WHEELDOWN_STATUS_LEFT: keyc = 34359738370;
pub const KEYC_WHEELDOWN11_STATUS: keyc = 34359741185;
pub const KEYC_WHEELDOWN10_STATUS: keyc = 34359740929;
pub const KEYC_WHEELDOWN9_STATUS: keyc = 34359740673;
pub const KEYC_WHEELDOWN8_STATUS: keyc = 34359740417;
pub const KEYC_WHEELDOWN7_STATUS: keyc = 34359740161;
pub const KEYC_WHEELDOWN6_STATUS: keyc = 34359739905;
pub const KEYC_WHEELDOWN3_STATUS: keyc = 34359739137;
pub const KEYC_WHEELDOWN2_STATUS: keyc = 34359738881;
pub const KEYC_WHEELDOWN1_STATUS: keyc = 34359738625;
pub const KEYC_WHEELDOWN_STATUS: keyc = 34359738369;
pub const KEYC_WHEELDOWN11_PANE: keyc = 34359741184;
pub const KEYC_WHEELDOWN10_PANE: keyc = 34359740928;
pub const KEYC_WHEELDOWN9_PANE: keyc = 34359740672;
pub const KEYC_WHEELDOWN8_PANE: keyc = 34359740416;
pub const KEYC_WHEELDOWN7_PANE: keyc = 34359740160;
pub const KEYC_WHEELDOWN6_PANE: keyc = 34359739904;
pub const KEYC_WHEELDOWN3_PANE: keyc = 34359739136;
pub const KEYC_WHEELDOWN2_PANE: keyc = 34359738880;
pub const KEYC_WHEELDOWN1_PANE: keyc = 34359738624;
pub const KEYC_WHEELDOWN_PANE: keyc = 34359738368;
pub const KEYC_MOUSEMOVE11_CONTROL9: keyc = 12884904722;
pub const KEYC_MOUSEMOVE10_CONTROL9: keyc = 12884904466;
pub const KEYC_MOUSEMOVE9_CONTROL9: keyc = 12884904210;
pub const KEYC_MOUSEMOVE8_CONTROL9: keyc = 12884903954;
pub const KEYC_MOUSEMOVE7_CONTROL9: keyc = 12884903698;
pub const KEYC_MOUSEMOVE6_CONTROL9: keyc = 12884903442;
pub const KEYC_MOUSEMOVE3_CONTROL9: keyc = 12884902674;
pub const KEYC_MOUSEMOVE2_CONTROL9: keyc = 12884902418;
pub const KEYC_MOUSEMOVE1_CONTROL9: keyc = 12884902162;
pub const KEYC_MOUSEMOVE_CONTROL9: keyc = 12884901906;
pub const KEYC_MOUSEMOVE11_CONTROL8: keyc = 12884904721;
pub const KEYC_MOUSEMOVE10_CONTROL8: keyc = 12884904465;
pub const KEYC_MOUSEMOVE9_CONTROL8: keyc = 12884904209;
pub const KEYC_MOUSEMOVE8_CONTROL8: keyc = 12884903953;
pub const KEYC_MOUSEMOVE7_CONTROL8: keyc = 12884903697;
pub const KEYC_MOUSEMOVE6_CONTROL8: keyc = 12884903441;
pub const KEYC_MOUSEMOVE3_CONTROL8: keyc = 12884902673;
pub const KEYC_MOUSEMOVE2_CONTROL8: keyc = 12884902417;
pub const KEYC_MOUSEMOVE1_CONTROL8: keyc = 12884902161;
pub const KEYC_MOUSEMOVE_CONTROL8: keyc = 12884901905;
pub const KEYC_MOUSEMOVE11_CONTROL7: keyc = 12884904720;
pub const KEYC_MOUSEMOVE10_CONTROL7: keyc = 12884904464;
pub const KEYC_MOUSEMOVE9_CONTROL7: keyc = 12884904208;
pub const KEYC_MOUSEMOVE8_CONTROL7: keyc = 12884903952;
pub const KEYC_MOUSEMOVE7_CONTROL7: keyc = 12884903696;
pub const KEYC_MOUSEMOVE6_CONTROL7: keyc = 12884903440;
pub const KEYC_MOUSEMOVE3_CONTROL7: keyc = 12884902672;
pub const KEYC_MOUSEMOVE2_CONTROL7: keyc = 12884902416;
pub const KEYC_MOUSEMOVE1_CONTROL7: keyc = 12884902160;
pub const KEYC_MOUSEMOVE_CONTROL7: keyc = 12884901904;
pub const KEYC_MOUSEMOVE11_CONTROL6: keyc = 12884904719;
pub const KEYC_MOUSEMOVE10_CONTROL6: keyc = 12884904463;
pub const KEYC_MOUSEMOVE9_CONTROL6: keyc = 12884904207;
pub const KEYC_MOUSEMOVE8_CONTROL6: keyc = 12884903951;
pub const KEYC_MOUSEMOVE7_CONTROL6: keyc = 12884903695;
pub const KEYC_MOUSEMOVE6_CONTROL6: keyc = 12884903439;
pub const KEYC_MOUSEMOVE3_CONTROL6: keyc = 12884902671;
pub const KEYC_MOUSEMOVE2_CONTROL6: keyc = 12884902415;
pub const KEYC_MOUSEMOVE1_CONTROL6: keyc = 12884902159;
pub const KEYC_MOUSEMOVE_CONTROL6: keyc = 12884901903;
pub const KEYC_MOUSEMOVE11_CONTROL5: keyc = 12884904718;
pub const KEYC_MOUSEMOVE10_CONTROL5: keyc = 12884904462;
pub const KEYC_MOUSEMOVE9_CONTROL5: keyc = 12884904206;
pub const KEYC_MOUSEMOVE8_CONTROL5: keyc = 12884903950;
pub const KEYC_MOUSEMOVE7_CONTROL5: keyc = 12884903694;
pub const KEYC_MOUSEMOVE6_CONTROL5: keyc = 12884903438;
pub const KEYC_MOUSEMOVE3_CONTROL5: keyc = 12884902670;
pub const KEYC_MOUSEMOVE2_CONTROL5: keyc = 12884902414;
pub const KEYC_MOUSEMOVE1_CONTROL5: keyc = 12884902158;
pub const KEYC_MOUSEMOVE_CONTROL5: keyc = 12884901902;
pub const KEYC_MOUSEMOVE11_CONTROL4: keyc = 12884904717;
pub const KEYC_MOUSEMOVE10_CONTROL4: keyc = 12884904461;
pub const KEYC_MOUSEMOVE9_CONTROL4: keyc = 12884904205;
pub const KEYC_MOUSEMOVE8_CONTROL4: keyc = 12884903949;
pub const KEYC_MOUSEMOVE7_CONTROL4: keyc = 12884903693;
pub const KEYC_MOUSEMOVE6_CONTROL4: keyc = 12884903437;
pub const KEYC_MOUSEMOVE3_CONTROL4: keyc = 12884902669;
pub const KEYC_MOUSEMOVE2_CONTROL4: keyc = 12884902413;
pub const KEYC_MOUSEMOVE1_CONTROL4: keyc = 12884902157;
pub const KEYC_MOUSEMOVE_CONTROL4: keyc = 12884901901;
pub const KEYC_MOUSEMOVE11_CONTROL3: keyc = 12884904716;
pub const KEYC_MOUSEMOVE10_CONTROL3: keyc = 12884904460;
pub const KEYC_MOUSEMOVE9_CONTROL3: keyc = 12884904204;
pub const KEYC_MOUSEMOVE8_CONTROL3: keyc = 12884903948;
pub const KEYC_MOUSEMOVE7_CONTROL3: keyc = 12884903692;
pub const KEYC_MOUSEMOVE6_CONTROL3: keyc = 12884903436;
pub const KEYC_MOUSEMOVE3_CONTROL3: keyc = 12884902668;
pub const KEYC_MOUSEMOVE2_CONTROL3: keyc = 12884902412;
pub const KEYC_MOUSEMOVE1_CONTROL3: keyc = 12884902156;
pub const KEYC_MOUSEMOVE_CONTROL3: keyc = 12884901900;
pub const KEYC_MOUSEMOVE11_CONTROL2: keyc = 12884904715;
pub const KEYC_MOUSEMOVE10_CONTROL2: keyc = 12884904459;
pub const KEYC_MOUSEMOVE9_CONTROL2: keyc = 12884904203;
pub const KEYC_MOUSEMOVE8_CONTROL2: keyc = 12884903947;
pub const KEYC_MOUSEMOVE7_CONTROL2: keyc = 12884903691;
pub const KEYC_MOUSEMOVE6_CONTROL2: keyc = 12884903435;
pub const KEYC_MOUSEMOVE3_CONTROL2: keyc = 12884902667;
pub const KEYC_MOUSEMOVE2_CONTROL2: keyc = 12884902411;
pub const KEYC_MOUSEMOVE1_CONTROL2: keyc = 12884902155;
pub const KEYC_MOUSEMOVE_CONTROL2: keyc = 12884901899;
pub const KEYC_MOUSEMOVE11_CONTROL1: keyc = 12884904714;
pub const KEYC_MOUSEMOVE10_CONTROL1: keyc = 12884904458;
pub const KEYC_MOUSEMOVE9_CONTROL1: keyc = 12884904202;
pub const KEYC_MOUSEMOVE8_CONTROL1: keyc = 12884903946;
pub const KEYC_MOUSEMOVE7_CONTROL1: keyc = 12884903690;
pub const KEYC_MOUSEMOVE6_CONTROL1: keyc = 12884903434;
pub const KEYC_MOUSEMOVE3_CONTROL1: keyc = 12884902666;
pub const KEYC_MOUSEMOVE2_CONTROL1: keyc = 12884902410;
pub const KEYC_MOUSEMOVE1_CONTROL1: keyc = 12884902154;
pub const KEYC_MOUSEMOVE_CONTROL1: keyc = 12884901898;
pub const KEYC_MOUSEMOVE11_CONTROL0: keyc = 12884904713;
pub const KEYC_MOUSEMOVE10_CONTROL0: keyc = 12884904457;
pub const KEYC_MOUSEMOVE9_CONTROL0: keyc = 12884904201;
pub const KEYC_MOUSEMOVE8_CONTROL0: keyc = 12884903945;
pub const KEYC_MOUSEMOVE7_CONTROL0: keyc = 12884903689;
pub const KEYC_MOUSEMOVE6_CONTROL0: keyc = 12884903433;
pub const KEYC_MOUSEMOVE3_CONTROL0: keyc = 12884902665;
pub const KEYC_MOUSEMOVE2_CONTROL0: keyc = 12884902409;
pub const KEYC_MOUSEMOVE1_CONTROL0: keyc = 12884902153;
pub const KEYC_MOUSEMOVE_CONTROL0: keyc = 12884901897;
pub const KEYC_MOUSEMOVE11_SCROLLBAR_DOWN: keyc = 12884904712;
pub const KEYC_MOUSEMOVE10_SCROLLBAR_DOWN: keyc = 12884904456;
pub const KEYC_MOUSEMOVE9_SCROLLBAR_DOWN: keyc = 12884904200;
pub const KEYC_MOUSEMOVE8_SCROLLBAR_DOWN: keyc = 12884903944;
pub const KEYC_MOUSEMOVE7_SCROLLBAR_DOWN: keyc = 12884903688;
pub const KEYC_MOUSEMOVE6_SCROLLBAR_DOWN: keyc = 12884903432;
pub const KEYC_MOUSEMOVE3_SCROLLBAR_DOWN: keyc = 12884902664;
pub const KEYC_MOUSEMOVE2_SCROLLBAR_DOWN: keyc = 12884902408;
pub const KEYC_MOUSEMOVE1_SCROLLBAR_DOWN: keyc = 12884902152;
pub const KEYC_MOUSEMOVE_SCROLLBAR_DOWN: keyc = 12884901896;
pub const KEYC_MOUSEMOVE11_SCROLLBAR_SLIDER: keyc = 12884904711;
pub const KEYC_MOUSEMOVE10_SCROLLBAR_SLIDER: keyc = 12884904455;
pub const KEYC_MOUSEMOVE9_SCROLLBAR_SLIDER: keyc = 12884904199;
pub const KEYC_MOUSEMOVE8_SCROLLBAR_SLIDER: keyc = 12884903943;
pub const KEYC_MOUSEMOVE7_SCROLLBAR_SLIDER: keyc = 12884903687;
pub const KEYC_MOUSEMOVE6_SCROLLBAR_SLIDER: keyc = 12884903431;
pub const KEYC_MOUSEMOVE3_SCROLLBAR_SLIDER: keyc = 12884902663;
pub const KEYC_MOUSEMOVE2_SCROLLBAR_SLIDER: keyc = 12884902407;
pub const KEYC_MOUSEMOVE1_SCROLLBAR_SLIDER: keyc = 12884902151;
pub const KEYC_MOUSEMOVE_SCROLLBAR_SLIDER: keyc = 12884901895;
pub const KEYC_MOUSEMOVE11_SCROLLBAR_UP: keyc = 12884904710;
pub const KEYC_MOUSEMOVE10_SCROLLBAR_UP: keyc = 12884904454;
pub const KEYC_MOUSEMOVE9_SCROLLBAR_UP: keyc = 12884904198;
pub const KEYC_MOUSEMOVE8_SCROLLBAR_UP: keyc = 12884903942;
pub const KEYC_MOUSEMOVE7_SCROLLBAR_UP: keyc = 12884903686;
pub const KEYC_MOUSEMOVE6_SCROLLBAR_UP: keyc = 12884903430;
pub const KEYC_MOUSEMOVE3_SCROLLBAR_UP: keyc = 12884902662;
pub const KEYC_MOUSEMOVE2_SCROLLBAR_UP: keyc = 12884902406;
pub const KEYC_MOUSEMOVE1_SCROLLBAR_UP: keyc = 12884902150;
pub const KEYC_MOUSEMOVE_SCROLLBAR_UP: keyc = 12884901894;
pub const KEYC_MOUSEMOVE11_BORDER: keyc = 12884904709;
pub const KEYC_MOUSEMOVE10_BORDER: keyc = 12884904453;
pub const KEYC_MOUSEMOVE9_BORDER: keyc = 12884904197;
pub const KEYC_MOUSEMOVE8_BORDER: keyc = 12884903941;
pub const KEYC_MOUSEMOVE7_BORDER: keyc = 12884903685;
pub const KEYC_MOUSEMOVE6_BORDER: keyc = 12884903429;
pub const KEYC_MOUSEMOVE3_BORDER: keyc = 12884902661;
pub const KEYC_MOUSEMOVE2_BORDER: keyc = 12884902405;
pub const KEYC_MOUSEMOVE1_BORDER: keyc = 12884902149;
pub const KEYC_MOUSEMOVE_BORDER: keyc = 12884901893;
pub const KEYC_MOUSEMOVE11_STATUS_DEFAULT: keyc = 12884904708;
pub const KEYC_MOUSEMOVE10_STATUS_DEFAULT: keyc = 12884904452;
pub const KEYC_MOUSEMOVE9_STATUS_DEFAULT: keyc = 12884904196;
pub const KEYC_MOUSEMOVE8_STATUS_DEFAULT: keyc = 12884903940;
pub const KEYC_MOUSEMOVE7_STATUS_DEFAULT: keyc = 12884903684;
pub const KEYC_MOUSEMOVE6_STATUS_DEFAULT: keyc = 12884903428;
pub const KEYC_MOUSEMOVE3_STATUS_DEFAULT: keyc = 12884902660;
pub const KEYC_MOUSEMOVE2_STATUS_DEFAULT: keyc = 12884902404;
pub const KEYC_MOUSEMOVE1_STATUS_DEFAULT: keyc = 12884902148;
pub const KEYC_MOUSEMOVE_STATUS_DEFAULT: keyc = 12884901892;
pub const KEYC_MOUSEMOVE11_STATUS_RIGHT: keyc = 12884904707;
pub const KEYC_MOUSEMOVE10_STATUS_RIGHT: keyc = 12884904451;
pub const KEYC_MOUSEMOVE9_STATUS_RIGHT: keyc = 12884904195;
pub const KEYC_MOUSEMOVE8_STATUS_RIGHT: keyc = 12884903939;
pub const KEYC_MOUSEMOVE7_STATUS_RIGHT: keyc = 12884903683;
pub const KEYC_MOUSEMOVE6_STATUS_RIGHT: keyc = 12884903427;
pub const KEYC_MOUSEMOVE3_STATUS_RIGHT: keyc = 12884902659;
pub const KEYC_MOUSEMOVE2_STATUS_RIGHT: keyc = 12884902403;
pub const KEYC_MOUSEMOVE1_STATUS_RIGHT: keyc = 12884902147;
pub const KEYC_MOUSEMOVE_STATUS_RIGHT: keyc = 12884901891;
pub const KEYC_MOUSEMOVE11_STATUS_LEFT: keyc = 12884904706;
pub const KEYC_MOUSEMOVE10_STATUS_LEFT: keyc = 12884904450;
pub const KEYC_MOUSEMOVE9_STATUS_LEFT: keyc = 12884904194;
pub const KEYC_MOUSEMOVE8_STATUS_LEFT: keyc = 12884903938;
pub const KEYC_MOUSEMOVE7_STATUS_LEFT: keyc = 12884903682;
pub const KEYC_MOUSEMOVE6_STATUS_LEFT: keyc = 12884903426;
pub const KEYC_MOUSEMOVE3_STATUS_LEFT: keyc = 12884902658;
pub const KEYC_MOUSEMOVE2_STATUS_LEFT: keyc = 12884902402;
pub const KEYC_MOUSEMOVE1_STATUS_LEFT: keyc = 12884902146;
pub const KEYC_MOUSEMOVE_STATUS_LEFT: keyc = 12884901890;
pub const KEYC_MOUSEMOVE11_STATUS: keyc = 12884904705;
pub const KEYC_MOUSEMOVE10_STATUS: keyc = 12884904449;
pub const KEYC_MOUSEMOVE9_STATUS: keyc = 12884904193;
pub const KEYC_MOUSEMOVE8_STATUS: keyc = 12884903937;
pub const KEYC_MOUSEMOVE7_STATUS: keyc = 12884903681;
pub const KEYC_MOUSEMOVE6_STATUS: keyc = 12884903425;
pub const KEYC_MOUSEMOVE3_STATUS: keyc = 12884902657;
pub const KEYC_MOUSEMOVE2_STATUS: keyc = 12884902401;
pub const KEYC_MOUSEMOVE1_STATUS: keyc = 12884902145;
pub const KEYC_MOUSEMOVE_STATUS: keyc = 12884901889;
pub const KEYC_MOUSEMOVE11_PANE: keyc = 12884904704;
pub const KEYC_MOUSEMOVE10_PANE: keyc = 12884904448;
pub const KEYC_MOUSEMOVE9_PANE: keyc = 12884904192;
pub const KEYC_MOUSEMOVE8_PANE: keyc = 12884903936;
pub const KEYC_MOUSEMOVE7_PANE: keyc = 12884903680;
pub const KEYC_MOUSEMOVE6_PANE: keyc = 12884903424;
pub const KEYC_MOUSEMOVE3_PANE: keyc = 12884902656;
pub const KEYC_MOUSEMOVE2_PANE: keyc = 12884902400;
pub const KEYC_MOUSEMOVE1_PANE: keyc = 12884902144;
pub const KEYC_MOUSEMOVE_PANE: keyc = 12884901888;
pub const KEYC_DOUBLECLICK: keyc = 8589934643;
pub const KEYC_DRAGGING: keyc = 8589934642;
pub const KEYC_MOUSE: keyc = 8589934641;
pub const KEYC_REPORT_LIGHT_THEME: keyc = 8589934640;
pub const KEYC_REPORT_DARK_THEME: keyc = 8589934639;
pub const KEYC_KP_PERIOD: keyc = 8589934638;
pub const KEYC_KP_ZERO: keyc = 8589934637;
pub const KEYC_KP_ENTER: keyc = 8589934636;
pub const KEYC_KP_THREE: keyc = 8589934635;
pub const KEYC_KP_TWO: keyc = 8589934634;
pub const KEYC_KP_ONE: keyc = 8589934633;
pub const KEYC_KP_SIX: keyc = 8589934632;
pub const KEYC_KP_FIVE: keyc = 8589934631;
pub const KEYC_KP_FOUR: keyc = 8589934630;
pub const KEYC_KP_PLUS: keyc = 8589934629;
pub const KEYC_KP_NINE: keyc = 8589934628;
pub const KEYC_KP_EIGHT: keyc = 8589934627;
pub const KEYC_KP_SEVEN: keyc = 8589934626;
pub const KEYC_KP_MINUS: keyc = 8589934625;
pub const KEYC_KP_STAR: keyc = 8589934624;
pub const KEYC_KP_SLASH: keyc = 8589934623;
pub const KEYC_RIGHT: keyc = 8589934622;
pub const KEYC_LEFT: keyc = 8589934621;
pub const KEYC_DOWN: keyc = 8589934620;
pub const KEYC_UP: keyc = 8589934619;
pub const KEYC_BTAB: keyc = 8589934618;
pub const KEYC_PPAGE: keyc = 8589934617;
pub const KEYC_NPAGE: keyc = 8589934616;
pub const KEYC_END: keyc = 8589934615;
pub const KEYC_HOME: keyc = 8589934614;
pub const KEYC_DC: keyc = 8589934613;
pub const KEYC_IC: keyc = 8589934612;
pub const KEYC_F12: keyc = 8589934611;
pub const KEYC_F11: keyc = 8589934610;
pub const KEYC_F10: keyc = 8589934609;
pub const KEYC_F9: keyc = 8589934608;
pub const KEYC_F8: keyc = 8589934607;
pub const KEYC_F7: keyc = 8589934606;
pub const KEYC_F6: keyc = 8589934605;
pub const KEYC_F5: keyc = 8589934604;
pub const KEYC_F4: keyc = 8589934603;
pub const KEYC_F3: keyc = 8589934602;
pub const KEYC_F2: keyc = 8589934601;
pub const KEYC_F1: keyc = 8589934600;
pub const KEYC_BSPACE: keyc = 8589934599;
pub const KEYC_PASTE_END: keyc = 8589934598;
pub const KEYC_PASTE_START: keyc = 8589934597;
pub const KEYC_ANY: keyc = 8589934596;
pub const KEYC_FOCUS_OUT: keyc = 8589934595;
pub const KEYC_FOCUS_IN: keyc = 8589934594;
pub const KEYC_UNKNOWN: keyc = 8589934593;
pub const KEYC_NONE: keyc = 8589934592;
pub const KEYC_USER: keyc = 4294967296;
pub const ARGS_PARSE_COMMANDS: args_parse_type = 3;
pub const ARGS_PARSE_COMMANDS_OR_STRING: args_parse_type = 2;
pub const ARGS_PARSE_STRING: args_parse_type = 1;
pub const ARGS_PARSE_INVALID: args_parse_type = 0;
pub const CMD_FIND_SESSION: cmd_find_type = 2;
pub const CMD_FIND_WINDOW: cmd_find_type = 1;
pub const CMD_FIND_PANE: cmd_find_type = 0;
#[inline]
fn toupper(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
            *(*__ctype_toupper_loc()).offset(__c as isize) as ::core::ffi::c_int
        } else {
            __c
        }
    }
}
pub const CMDQ_STATE_CONTROL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CMDQ_STATE_NOHOOKS: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMD_CLIENT_CFLAG: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CMD_CLIENT_TFLAG: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CMD_CLIENT_CANFAIL: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const CLIENT_UTF8: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const FORMAT_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CMDQ_FIRED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMDQ_WAITING: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
/// How a client is named in the queue's log lines, as the caller's own
/// string.
unsafe fn cmdq_name(mut c: *mut client) -> ::std::ffi::CString {
    unsafe {
        if c.is_null() {
            return c"<global>".to_owned();
        }
        if (*c).name.is_some() {
            format_alloc(c"<%s>".as_ptr(), fmt_args![(*c).name.as_deref()])
        } else {
            format_alloc(c"<%p>".as_ptr(), fmt_args![c])
        }
    }
}
/// The queue the items with no client behind them wait on, made when the
/// first one is queued and held by the server for as long as it runs.
static GLOBAL_QUEUE: GlobalQueue<Box<cmdq_list>> = GlobalQueue::new();

unsafe fn cmdq_get(mut c: *mut client) -> *mut cmdq_list {
    unsafe {
        if c.is_null() {
            let held = GLOBAL_QUEUE.queue();
            if held.is_empty() {
                held.push_back(cmdq_new());
            }
            return &raw mut **held.front_mut().expect("the global queue was just made");
        }
        (*c).queue
            .as_mut()
            .map_or(::core::ptr::null_mut::<cmdq_list>(), |queue| {
                &raw mut **queue
            })
    }
}
pub fn cmdq_new() -> Box<cmdq_list> {
    Box::new(cmdq_list {
        running: false,
        list: cmdq_item_list::new(),
    })
}
pub fn cmdq_free(queue: Box<cmdq_list>) {
    unsafe {
        if !queue.list.is_empty() {
            fatalx(c"queue not empty".as_ptr(), fmt_args![]);
        }
    }
}
/// The name the item was queued under, or nothing before it has been given
/// one.
pub fn cmdq_get_name(item: &cmdq_item) -> Option<&::core::ffi::CStr> {
    item.name.as_deref()
}
pub fn cmdq_get_client(item: &cmdq_item) -> *mut client {
    item.client
        .as_ref()
        .map_or(::core::ptr::null_mut::<client>(), ClientRef::as_ptr)
}
/// Makes `tc` the client the item's target was found against.
pub unsafe fn cmdq_set_target_client(item: *mut cmdq_item, tc: *mut client) {
    unsafe {
        (*item).target_client = client_ref_from_ptr(tc).map(|c| c.downgrade());
    }
}

pub fn cmdq_get_target_client(item: &cmdq_item) -> *mut client {
    item.target_client
        .as_ref()
        .and_then(ClientWeak::upgrade)
        .map_or(::core::ptr::null_mut(), |c| c.as_ptr())
}
pub fn cmdq_get_state(item: &cmdq_item) -> *mut cmdq_state {
    (*item).state()
}

/// The item's share of its queue's state, as a handle a new item can be given
/// so that the two run under the same one.
pub(crate) fn cmdq_get_state_ref(item: &CmdqItemRef) -> &CmdqStateRef {
    item.item()
        .state_ref
        .as_ref()
        .expect("a queue item without a state")
}
pub unsafe fn cmdq_get_target(mut item: *mut cmdq_item) -> *mut cmd_find_state {
    unsafe { &raw mut (*item).target }
}
pub unsafe fn cmdq_get_source(mut item: *mut cmdq_item) -> *mut cmd_find_state {
    unsafe { &raw mut (*item).source }
}
pub unsafe fn cmdq_get_event(mut item: *mut cmdq_item) -> *mut key_event {
    unsafe { &raw mut (*(*item).state()).event }
}
pub unsafe fn cmdq_get_current(mut item: *mut cmdq_item) -> *mut cmd_find_state {
    unsafe { &raw mut (*(*item).state()).current }
}
pub unsafe fn cmdq_get_flags(item: &cmdq_item) -> ::core::ffi::c_int {
    unsafe { (*(*item).state()).flags }
}
pub(crate) unsafe fn cmdq_new_state(
    mut current: *mut cmd_find_state,
    mut event: *mut key_event,
    mut flags: ::core::ffi::c_int,
) -> CmdqStateRef {
    unsafe {
        let state = CmdqStateRef::new(cmdq_state {
            flags,
            formats: None,
            event: key_event::default(),
            current: cmd_find_state::default(),
        });
        let state_ptr = state.as_ptr();
        if !event.is_null() {
            (*state_ptr).event = (*event).clone();
        } else {
            (*state_ptr).event.key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
        }
        if !current.is_null() && cmd_find_valid_state(&*current) != 0 {
            cmd_find_copy_state(&mut (*state_ptr).current, &*current);
        } else {
            cmd_find_clear_state(&mut (*state_ptr).current, 0 as ::core::ffi::c_int);
        }
        state
    }
}
pub(crate) unsafe fn cmdq_copy_state(
    state: &CmdqStateRef,
    mut current: *mut cmd_find_state,
) -> CmdqStateRef {
    unsafe {
        let state_ptr = state.as_ptr();
        if !current.is_null() {
            return cmdq_new_state(current, &raw mut (*state_ptr).event, (*state_ptr).flags);
        }
        cmdq_new_state(
            &raw mut (*state_ptr).current,
            &raw mut (*state_ptr).event,
            (*state_ptr).flags,
        )
    }
}
unsafe fn cmdq_state_formats(state: *mut cmdq_state) -> *mut format_tree {
    unsafe {
        (*state)
            .formats
            .as_deref_mut()
            .map_or(::core::ptr::null_mut::<format_tree>(), |ft| &raw mut *ft)
    }
}

/// The formats `state` carries, made when it has none yet.
#[allow(clippy::mut_from_ref)]
unsafe fn cmdq_state_formats_mut(state: &CmdqStateRef) -> &mut format_tree {
    unsafe {
        state.state().formats.get_or_insert_with(|| {
            format_create(
                ::core::ptr::null_mut::<client>(),
                ::core::ptr::null_mut::<cmdq_item>(),
                FORMAT_NONE,
                0 as ::core::ffi::c_int,
            )
        })
    }
}
pub unsafe fn cmdq_add_format(
    state: &CmdqStateRef,
    mut key: *const ::core::ffi::c_char,
    mut fmt: *const ::core::ffi::c_char,
    args: &[FmtArg],
) {
    unsafe {
        let value = format_alloc(fmt, args);
        format_add(
            cmdq_state_formats_mut(state),
            CStr::from_ptr(key),
            c"%s".as_ptr(),
            fmt_args![value.as_ptr()],
        );
    }
}
pub unsafe fn cmdq_add_formats(state: &CmdqStateRef, ft: &mut format_tree) {
    unsafe { format_merge(cmdq_state_formats_mut(state), ft) };
}
pub unsafe fn cmdq_merge_formats(mut item: *mut cmdq_item, ft: &mut format_tree) {
    unsafe {
        let mut entry: *const cmd_entry = ::core::ptr::null::<cmd_entry>();
        let cmd = (*item).cmd();
        if !cmd.is_null() {
            entry = cmd_get_entry(&*cmd);
            format_add(ft, c"command", c"%s".as_ptr(), fmt_args![(*entry).name]);
        }
        if (*(*item).state()).formats.is_some() {
            format_merge(ft, &*cmdq_state_formats((*item).state()));
        }
    }
}
pub unsafe fn cmdq_append(mut c: *mut client, items: cmdq_items) -> *mut cmdq_item {
    unsafe {
        let mut queue: *mut cmdq_list = cmdq_get(c);
        for item in items {
            item.item().client = client_ref_from_ptr(c);
            item.item().queue = queue;
            log_debug(
                c"%s %s: %s".as_ptr(),
                fmt_args![
                    c"cmdq_append".as_ptr(),
                    cmdq_name(c).as_c_str(),
                    item.item().name.as_deref()
                ],
            );
            (*queue).list.push_back(item);
        }
        (*queue)
            .list
            .back()
            .map(CmdqItemRef::as_ptr)
            .unwrap_or(::core::ptr::null_mut::<cmdq_item>())
    }
}
pub unsafe fn cmdq_insert_after(anchor: &CmdqItemRef, items: cmdq_items) -> *mut cmdq_item {
    unsafe {
        let mut after = anchor.as_ptr();
        let mut c: *mut client = cmdq_get_client(anchor.item());
        let mut queue: *mut cmdq_list = anchor.item().queue;
        for item in items {
            item.item().client = client_ref_from_ptr(c);
            item.item().queue = queue;
            log_debug(
                c"%s %s: %s after %s".as_ptr(),
                fmt_args![
                    c"cmdq_insert_after".as_ptr(),
                    cmdq_name(c).as_c_str(),
                    item.item().name.as_deref(),
                    (*after).name.as_deref()
                ],
            );
            let at = cmdq_position(queue, after).expect("the anchor is on this queue");
            after = item.as_ptr();
            (*queue).list.insert(at + 1, item);
        }
        after
    }
}
pub unsafe fn cmdq_insert_hook(
    mut s: *mut session,
    mut item: *mut cmdq_item,
    mut current: *mut cmd_find_state,
    mut fmt: *const ::core::ffi::c_char,
    args: &[FmtArg],
) {
    unsafe {
        let state = (*item).state();
        let cmd: &cmd = &*(*item).cmd();
        let args_0: &args = cmd_get_args(cmd);
        let args_ptr = cmd_get_args_ptr(cmd);
        let mut oo: *mut options = ::core::ptr::null_mut::<options>();
        let mut i: u_int = 0;
        let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let new_state = cmdq_new_state(current, &raw mut (*state).event, CMDQ_STATE_NOHOOKS);
        let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
        let mut a: *mut options_array_item_t = ::core::ptr::null_mut::<options_array_item_t>();
        if (*(*item).state()).flags & CMDQ_STATE_NOHOOKS != 0 {
            return;
        }
        if s.is_null() {
            oo = global_s_options;
        } else {
            oo = session_options(s);
        }
        let name = format_alloc(fmt, args);
        o = options_get_ptr(oo, name.as_ptr());
        if o.is_null() {
            return;
        }
        log_debug(
            c"running hook %s (parent %p)".as_ptr(),
            fmt_args![name.as_ptr(), item],
        );
        cmdq_add_format(
            &new_state,
            c"hook".as_ptr(),
            c"%s".as_ptr(),
            fmt_args![name.as_ptr()],
        );
        let arguments = args_print(args_ptr);
        cmdq_add_format(
            &new_state,
            c"hook_arguments".as_ptr(),
            c"%s".as_ptr(),
            fmt_args![arguments.as_ptr()],
        );
        i = 0 as u_int;
        while i < args_count(args_0) {
            let tmp = xasprintf(c"hook_argument_%d".as_ptr(), fmt_args![i]);
            cmdq_add_format(
                &new_state,
                tmp.as_ptr(),
                c"%s".as_ptr(),
                fmt_args![args_string(args_0, i)],
            );
            i = i.wrapping_add(1);
        }
        for flag in args_flags(args_0).map(|flag| flag as ::core::ffi::c_char) {
            value = args_get(args_0, flag as u_char);
            if value.is_null() {
                let tmp = xasprintf(
                    c"hook_flag_%c".as_ptr(),
                    fmt_args![flag as ::core::ffi::c_int],
                );
                cmdq_add_format(&new_state, tmp.as_ptr(), c"1".as_ptr(), fmt_args![]);
            } else {
                let tmp = xasprintf(
                    c"hook_flag_%c".as_ptr(),
                    fmt_args![flag as ::core::ffi::c_int],
                );
                cmdq_add_format(
                    &new_state,
                    tmp.as_ptr(),
                    c"%s".as_ptr(),
                    fmt_args![value],
                );
            }
            i = 0 as u_int;
            for av in args_value_list(args_0, flag as u_char) {
                let tmp = xasprintf(
                    c"hook_flag_%c_%d".as_ptr(),
                    fmt_args![flag as ::core::ffi::c_int, i],
                );
                cmdq_add_format(
                    &new_state,
                    tmp.as_ptr(),
                    c"%s".as_ptr(),
                    fmt_args![(*av).value.string()],
                );
                i = i.wrapping_add(1);
            }
        }
        a = options_array_first(o);
        while !a.is_null() {
            if let Some(cmdlist) = options_array_item_command(a) {
                let queued = cmdq_get_command(&cmdlist, Some(&new_state));
                item = match cmdq_item_ref_from_ptr(item) {
                    Some(after) => cmdq_insert_after(&after, queued),
                    None => cmdq_append(::core::ptr::null_mut::<client>(), queued),
                };
            }
            a = options_array_next(o, a);
        }
    }
}
/// Lets a parked item run again. Taking the strong handle makes a dead
/// waiter unrepresentable here: whoever answers later holds a
/// [`CmdqItemWeak`] and reaches this only through a successful upgrade.
pub fn cmdq_continue(item: &CmdqItemRef) {
    item.item().flags &= !CMDQ_WAITING;
}
/// Where `item` sits on `queue`, which is wherever it was put.
unsafe fn cmdq_position(queue: *mut cmdq_list, item: *mut cmdq_item) -> Option<usize> {
    unsafe {
        (*queue)
            .list
            .iter()
            .position(|waiting| waiting.as_ptr() == item)
    }
}

unsafe fn cmdq_remove(item: *mut cmdq_item) {
    unsafe {
        let _ = (*item).client.take();
        let _ = (*item).state_ref.take();
        let queue = (*item).queue;
        (*item).name = None;
        if let Some(at) = cmdq_position(queue, item) {
            (*queue).list.remove(at);
        }
    }
}
unsafe fn cmdq_remove_group(item: *mut cmdq_item) {
    unsafe {
        if (*item).group == 0 as u_int {
            return;
        }
        let queue = (*item).queue;
        for this in foreach_safe_after_by(&raw mut (*queue).list, item_at, item) {
            if (*this).group == (*item).group {
                cmdq_remove(this);
            }
        }
    }
}
unsafe fn cmdq_empty_command(_item: *mut cmdq_item, _data: CmdqCallbackData) -> cmd_retval {
    CMD_RETURN_NORMAL
}
pub(crate) unsafe fn cmdq_get_command(
    cmdlist: &CmdListRef,
    state: Option<&CmdqStateRef>,
) -> cmdq_items {
    unsafe {
        let mut item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
        let mut items = cmdq_items::new();
        let mut entry: *const cmd_entry = ::core::ptr::null::<cmd_entry>();
        let state = state
            .cloned()
            .unwrap_or_else(|| cmdq_new_state(::core::ptr::null_mut(), ::core::ptr::null_mut(), 0));
        let commands = cmd_list_all(cmdlist);
        if commands.is_empty() {
            return cmdq_get_callback1(
                c"cmdq_empty_command".as_ptr(),
                Some(cmdq_empty_command),
                CmdqCallbackData::None,
            );
        }
        for (at, cmd) in commands.into_iter().enumerate() {
            entry = cmd_get_entry(&*cmd);
            let new = cmdq_item_new(
                CmdqType::Command {
                    cmdlist: Some(cmdlist.clone()),
                    at,
                },
                state.clone(),
            );
            item = new.as_ptr();
            new.item().name = Some(xasprintf(
                c"[%s/%p]".as_ptr(),
                fmt_args![(*entry).name, item],
            ));
            new.item().group = cmd_get_group(&*cmd);
            log_debug(
                c"%s: %s group %u".as_ptr(),
                fmt_args![
                    c"cmdq_get_command".as_ptr(),
                    new.item().name.as_deref(),
                    new.item().group
                ],
            );
            items.push(new);
        }
        items
    }
}
unsafe fn cmdq_find_flag(
    mut item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut flag: *const cmd_entry_flag,
) -> cmd_retval {
    unsafe {
        let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if (*flag).flag as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            cmd_find_from_client(
                &mut *fs,
                cmdq_get_target_client(&*item),
                0 as ::core::ffi::c_int,
            );
            return CMD_RETURN_NORMAL;
        }
        value = args_get(cmd_get_args(&*(*item).cmd()), (*flag).flag as u_char);
        if cmd_find_target(&mut *fs, item, value, (*flag).type_0, (*flag).flags)
            != 0 as ::core::ffi::c_int
        {
            cmd_find_clear_state(&mut *fs, 0 as ::core::ffi::c_int);
            return CMD_RETURN_ERROR;
        }
        CMD_RETURN_NORMAL
    }
}
unsafe fn cmdq_add_message(mut item: *mut cmdq_item) {
    unsafe {
        let mut c: *mut client = cmdq_get_client(&*item);
        let mut state: *mut cmdq_state = (*item).state();
        let mut key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut uid: uid_t = 0;
        let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
        let mut user: Option<CString> = None;
        let tmp = cmd_print(&*(*item).cmd());
        if !c.is_null() {
            uid = proc_get_peer_uid((*c).peer_ptr());
            if uid != -(1 as ::core::ffi::c_int) as uid_t && uid != getuid() {
                pw = getpwuid(uid as __uid_t);
                if !pw.is_null() {
                    user = Some(xasprintf(c"[%s]".as_ptr(), fmt_args![(*pw).pw_name]));
                } else {
                    user = Some(c"[unknown]".to_owned());
                }
            } else {
                user = Some(c"".to_owned());
            }
            if !(*c).session.is_null()
                && (*state).event.key != KEYC_NONE as ::core::ffi::c_ulong as key_code
            {
                key = key_string_lookup_key((*state).event.key, 0 as ::core::ffi::c_int);
                server_add_message(
                    c"%s%s key %s: %s".as_ptr(),
                    fmt_args![(*c).name.as_deref(), user.as_deref(), key, tmp.as_ptr()],
                );
            } else {
                server_add_message(
                    c"%s%s command: %s".as_ptr(),
                    fmt_args![(*c).name.as_deref(), user.as_deref(), tmp.as_ptr()],
                );
            }
        } else {
            server_add_message(c"command: %s".as_ptr(), fmt_args![tmp.as_ptr()]);
        }
    }
}
/// Fires `fired`'s command. Taking the strong handle by borrow is what keeps
/// the item alive for the whole fire, however the command reaches it again;
/// the raw view below is the compatibility form the rest of the pipeline
/// still speaks.
unsafe fn cmdq_fire_command(fired: &CmdqItemRef) -> cmd_retval {
    unsafe {
        let item: *mut cmdq_item = fired.as_ptr();
        let mut current_block: u64;
        let name = cmdq_name(cmdq_get_client(&*item));
        let name = name.as_c_str();
        let mut state: *mut cmdq_state = (*item).state();
        let mut cmd: *mut cmd = (*item).cmd();
        let args: &args = cmd_get_args(&*cmd);
        let mut entry: *const cmd_entry = cmd_get_entry(&*cmd);
        let mut tc: *mut client = ::core::ptr::null_mut::<client>();
        let saved = (*item).client.clone();
        let mut retval: cmd_retval = CMD_RETURN_NORMAL;
        let mut fsp: *mut cmd_find_state = ::core::ptr::null_mut::<cmd_find_state>();
        let mut fs = cmd_find_state::default();
        let mut flags: ::core::ffi::c_int = 0;
        let mut quiet: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if cfg_finished != 0 {
            cmdq_add_message(item);
        }
        if log_get_level() > 1 as ::core::ffi::c_int {
            let tmp = cmd_print(&*cmd);
            log_debug(
                c"%s %s: (%u) %s".as_ptr(),
                fmt_args![
                    c"cmdq_fire_command".as_ptr(),
                    name,
                    (*item).group,
                    tmp.as_ptr()
                ],
            );
        }
        flags = ((*state).flags & CMDQ_STATE_CONTROL != 0) as ::core::ffi::c_int;
        cmdq_guard(item, c"begin".as_ptr(), flags);
        if cmdq_get_client(&*item).is_null() {
            let c = cmd_find_client(
                item,
                ::core::ptr::null::<::core::ffi::c_char>(),
                1 as ::core::ffi::c_int,
            );
            (*item).client = client_ref_from_ptr(c);
        }
        if (*entry).flags & CMD_CLIENT_CANFAIL != 0 {
            quiet = 1 as ::core::ffi::c_int;
        }
        if (*entry).flags & CMD_CLIENT_CFLAG != 0 {
            tc = cmd_find_client(item, args_get(args, 'c' as i32 as u_char), quiet);
            if tc.is_null() && quiet == 0 {
                retval = CMD_RETURN_ERROR;
                current_block = 14411469634214383879;
            } else {
                current_block = 18317007320854588510;
            }
        } else if (*entry).flags & CMD_CLIENT_TFLAG != 0 {
            tc = cmd_find_client(item, args_get(args, 't' as i32 as u_char), quiet);
            if tc.is_null() && quiet == 0 {
                retval = CMD_RETURN_ERROR;
                current_block = 14411469634214383879;
            } else {
                current_block = 18317007320854588510;
            }
        } else {
            tc = cmd_find_client(
                item,
                ::core::ptr::null::<::core::ffi::c_char>(),
                1 as ::core::ffi::c_int,
            );
            current_block = 18317007320854588510;
        }
        if current_block == 18317007320854588510 {
            cmdq_set_target_client(item, tc);
            retval = cmdq_find_flag(item, &raw mut (*item).source, &raw const (*entry).source);
            if !(retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int) {
                retval = cmdq_find_flag(item, &raw mut (*item).target, &raw const (*entry).target);
                if !(retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int) {
                    retval = ((*entry).exec)(&*cmd, item);
                    if !(retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int)
                        && (*entry).flags & CMD_AFTERHOOK != 0
                    {
                        if cmd_find_valid_state(&(*item).target) != 0 {
                            fsp = &raw mut (*item).target;
                            current_block = 8704759739624374314;
                        } else if cmd_find_valid_state(&(*(*item).state()).current) != 0 {
                            fsp = &raw mut (*(*item).state()).current;
                            current_block = 8704759739624374314;
                        } else if cmd_find_from_client(
                            &mut fs,
                            cmdq_get_client(&*item),
                            0 as ::core::ffi::c_int,
                        ) == 0 as ::core::ffi::c_int
                        {
                            fsp = &raw mut fs;
                            current_block = 8704759739624374314;
                        } else {
                            current_block = 14411469634214383879;
                        }
                        match current_block {
                            14411469634214383879 => {}
                            _ => {
                                cmdq_insert_hook(
                                    (*fsp).session(),
                                    item,
                                    fsp,
                                    c"after-%s".as_ptr(),
                                    fmt_args![(*entry).name],
                                );
                            }
                        }
                    }
                }
            }
        }
        (*item).client = saved;
        if retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int {
            fsp = ::core::ptr::null_mut::<cmd_find_state>();
            if cmd_find_valid_state(&(*item).target) != 0 {
                fsp = &raw mut (*item).target;
            } else if cmd_find_valid_state(&(*(*item).state()).current) != 0 {
                fsp = &raw mut (*(*item).state()).current;
            } else if cmd_find_from_client(
                &mut fs,
                cmdq_get_client(&*item),
                0 as ::core::ffi::c_int,
            ) == 0 as ::core::ffi::c_int
            {
                fsp = &raw mut fs;
            }
            cmdq_insert_hook(
                if !fsp.is_null() {
                    (*fsp).session()
                } else {
                    ::core::ptr::null_mut::<session>()
                },
                item,
                fsp,
                c"command-error".as_ptr(),
                fmt_args![],
            );
            cmdq_guard(item, c"error".as_ptr(), flags);
        } else {
            cmdq_guard(item, c"end".as_ptr(), flags);
        }
        retval
    }
}
pub unsafe fn cmdq_get_callback1(
    mut name: *const ::core::ffi::c_char,
    mut cb: cmdq_cb,
    mut data: CmdqCallbackData,
) -> cmdq_items {
    unsafe {
        let state = cmdq_new_state(
            ::core::ptr::null_mut::<cmd_find_state>(),
            ::core::ptr::null_mut::<key_event>(),
            0 as ::core::ffi::c_int,
        );
        let item = cmdq_item_new(
            CmdqType::Callback {
                cb: cb.expect("non-null function pointer"),
                data,
            },
            state,
        );
        item.item().name = Some(xasprintf(
            c"[%s/%p]".as_ptr(),
            fmt_args![name, item.as_ptr()],
        ));
        item.item().group = 0 as u_int;
        ::std::vec![item]
    }
}
unsafe fn cmdq_error_callback(mut item: *mut cmdq_item, data: CmdqCallbackData) -> cmd_retval {
    unsafe {
        let CmdqCallbackData::String(error) = data else {
            return CMD_RETURN_ERROR;
        };
        cmdq_error(item, c"%s".as_ptr(), fmt_args![error.as_ptr()]);
        CMD_RETURN_NORMAL
    }
}
pub unsafe fn cmdq_get_error(mut error: *const ::core::ffi::c_char) -> cmdq_items {
    unsafe {
        cmdq_get_callback1(
            c"cmdq_error_callback".as_ptr(),
            Some(cmdq_error_callback),
            CmdqCallbackData::String(CStr::from_ptr(error).to_owned()),
        )
    }
}
/// Fires `fired`'s callback, holding the item alive through the borrowed
/// strong handle the same way [`cmdq_fire_command`] does.
unsafe fn cmdq_fire_callback(fired: &CmdqItemRef) -> cmd_retval {
    unsafe {
        let item = fired.as_ptr();
        let (cb, data) = match &mut (*item).type_0 {
            CmdqType::Callback { cb, data } => (*cb, std::mem::take(data)),
            CmdqType::Command { .. } => return CMD_RETURN_ERROR,
        };
        cb(item, data)
    }
}
pub unsafe fn cmdq_next(mut c: *mut client) -> u_int {
    unsafe {
        let mut current_block: u64;
        let mut queue: *mut cmdq_list = cmdq_get(c);
        let name = cmdq_name(c);
        let name = name.as_c_str();
        let mut item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
        let mut retval: cmd_retval = CMD_RETURN_NORMAL;
        let mut items: u_int = 0 as u_int;
        static mut number: u_int = 0;
        if (*queue).list.is_empty() {
            log_debug(
                c"%s %s: empty".as_ptr(),
                fmt_args![c"cmdq_next".as_ptr(), name],
            );
            return 0 as u_int;
        }
        if (*queue).list[0].item().flags & CMDQ_WAITING != 0 {
            log_debug(
                c"%s %s: waiting".as_ptr(),
                fmt_args![c"cmdq_next".as_ptr(), name],
            );
            return 0 as u_int;
        }
        log_debug(
            c"%s %s: enter".as_ptr(),
            fmt_args![c"cmdq_next".as_ptr(), name],
        );
        loop {
            let Some(fired) = (*queue).list.front().cloned() else {
                (*queue).running = false;
                current_block = 7056779235015430508;
                break;
            };
            (*queue).running = true;
            item = fired.as_ptr();
            let is_command = matches!((*item).type_0, CmdqType::Command { .. });
            log_debug(
                c"%s %s: %s (%d), flags %x".as_ptr(),
                fmt_args![
                    c"cmdq_next".as_ptr(),
                    name,
                    (*item).name.as_deref(),
                    if is_command { 0u32 } else { 1u32 },
                    (*item).flags
                ],
            );
            if (*item).flags & CMDQ_WAITING != 0 {
                current_block = 14973237773922011285;
                break;
            }
            if !(*item).flags & CMDQ_FIRED != 0 {
                (*item).time = time(::core::ptr::null_mut::<time_t>());
                number = number.wrapping_add(1);
                (*item).number = number;
                if is_command {
                    retval = cmdq_fire_command(&fired);
                    if retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int {
                        cmdq_remove_group(item);
                    }
                } else {
                    retval = cmdq_fire_callback(&fired);
                }
                (*item).flags |= CMDQ_FIRED;
                if retval as ::core::ffi::c_int == CMD_RETURN_WAIT as ::core::ffi::c_int {
                    (*item).flags |= CMDQ_WAITING;
                    current_block = 14973237773922011285;
                    break;
                } else {
                    items = items.wrapping_add(1);
                }
            }
            cmdq_remove(item);
        }
        match current_block {
            14973237773922011285 => {
                log_debug(
                    c"%s %s: exit (wait)".as_ptr(),
                    fmt_args![c"cmdq_next".as_ptr(), name],
                );
                items
            }
            _ => {
                (*queue).running = false;
                log_debug(
                    c"%s %s: exit (empty)".as_ptr(),
                    fmt_args![c"cmdq_next".as_ptr(), name],
                );
                items
            }
        }
    }
}
pub unsafe fn cmdq_running(mut c: *mut client) -> *mut cmdq_item {
    unsafe {
        let mut queue: *mut cmdq_list = cmdq_get(c);
        if !(*queue).running {
            return ::core::ptr::null_mut::<cmdq_item>();
        }
        let item = (*queue)
            .list
            .front()
            .map(CmdqItemRef::as_ptr)
            .unwrap_or(::core::ptr::null_mut::<cmdq_item>());
        if item.is_null() || (*item).flags & CMDQ_WAITING != 0 {
            return ::core::ptr::null_mut::<cmdq_item>();
        }
        item
    }
}
pub unsafe fn cmdq_guard(
    mut item: *mut cmdq_item,
    mut guard: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) {
    unsafe {
        let mut c: *mut client = cmdq_get_client(&*item);
        let mut t: ::core::ffi::c_long = (*item).time as ::core::ffi::c_long;
        let mut number: u_int = (*item).number;
        if !c.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            control_write(
                c,
                c"%%%s %ld %u %d".as_ptr(),
                fmt_args![guard, t, number, flags],
            );
        }
    }
}
pub unsafe fn cmdq_print_data(mut item: *mut cmdq_item, evb: &mut Buf) {
    unsafe {
        server_client_print(cmdq_get_client(&*item), 1 as ::core::ffi::c_int, evb);
    }
}
pub unsafe fn cmdq_print(
    mut item: *mut cmdq_item,
    mut fmt: *const ::core::ffi::c_char,
    args: &[FmtArg],
) {
    unsafe {
        let mut evb = Buf::new();
        format_buf(&mut evb, fmt, args);
        cmdq_print_data(item, &mut evb);
    }
}
pub unsafe fn cmdq_error(
    mut item: *mut cmdq_item,
    mut fmt: *const ::core::ffi::c_char,
    args: &[FmtArg],
) {
    unsafe {
        let mut c: *mut client = cmdq_get_client(&*item);
        let mut cmd: *mut cmd = (*item).cmd();
        let mut file: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut line: u_int = 0;
        let mut msg = format_alloc(fmt, args);
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"cmdq_error".as_ptr(), msg.as_ptr()],
        );
        if c.is_null() {
            (file, line) = cmd_get_source(&*cmd);
            cfg_add_cause(c"%s:%u: %s".as_ptr(), fmt_args![file, line, msg.as_ptr()]);
        } else if (*c).session.is_null() || (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            server_add_message(
                c"%s message: %s".as_ptr(),
                fmt_args![(*c).name.as_deref(), msg.as_ptr()],
            );
            if !(*c).flags & CLIENT_UTF8 as uint64_t != 0 {
                msg = utf8_sanitize(msg.as_ptr());
            }
            if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                control_write(c, c"%s".as_ptr(), fmt_args![msg.as_ptr()]);
            } else {
                file_error(c, c"%s\n".as_ptr(), fmt_args![msg.as_ptr()]);
            }
            (*c).retval = 1 as ::core::ffi::c_int;
        } else {
            let mut bytes = msg.into_bytes();
            if let Some(first) = bytes.first_mut() {
                *first = toupper(*first as ::core::ffi::c_int) as u8;
            }
            msg = CString::new(bytes).expect("a message holds no NUL");
            status_message_set(
                c,
                -(1 as ::core::ffi::c_int),
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                c"%s".as_ptr(),
                fmt_args![msg.as_ptr()],
            );
        }
    }
}
