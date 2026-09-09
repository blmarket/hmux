//! `display-panes`: puts each pane's number over the panes of a client's
//! current window and runs a command against whichever one is chosen.
//!
//! Exec prepares the command template — `select-pane -t "%%%"` unless the
//! command line gave another — parks it with the item that is waiting, if any,
//! in a [`cmd_display_panes_data`] owned by the overlay, and hands that to the
//! client as an overlay with a delay, a draw callback, a free callback and,
//! unless `-N` was given, a key callback. `-b` gives up the wait, which is what
//! decides whether the chosen pane's command is spliced in behind the asking
//! item or appended to the client's own queue.
//!
//! Rendering is owned by the overlay subsystem; this command chooses delay,
//! key handling and whether the queue waits for selection.

use crate::args::RustArguments;
use crate::args::{args_has, args_make_commands, args_make_commands_prepare, args_strtonum};
use crate::cmd::CmdqItemRef;
use crate::cmd::cmd_get_args;
use crate::cmd::{CmdqItemWeak, cmdq_append, cmdq_item_weak_of};
use crate::fmt_args;

use crate::server::client_set_overlay;

use crate::consts::{
    ARGS_PARSE_COMMANDS_OR_STRING, CMD_AFTERHOOK, CMD_CLIENT_TFLAG, CMD_FIND_PANE,
    CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_WAIT, KEYC_MASK_KEY, KEYC_MASK_MODIFIERS,
    UINT_MAX,
};
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::cmd::cmdq_item;
use crate::types::{
    ClientRef, OptionsRef, Overlay, OverlayState, SessionRef, args, args_command_state,
    args_parse_t, args_parse_type, key_code, key_event, u_int,
};
#[cfg(test)]
use crate::types::{CmdqListRef, client, screen_redraw_ctx, window_pane};
use crate::xmalloc::xasprintf;
use ::core::ffi::{c_int, c_longlong, c_ulonglong};
use ::std::ffi::CString;

/// What the command leaves on the client while the numbers are up: the item
/// waiting for a pane to be chosen, if any, and the prepared template the
/// chosen pane's id is substituted into.
#[derive(Default)]
#[repr(C)]
pub struct cmd_display_panes_data {
    pub(crate) item: Option<CmdqItemWeak>,
    pub state: Option<Box<args_command_state>>,
}
#[derive(Clone, Default)]
pub struct DisplayPanesRef(std::rc::Rc<std::cell::RefCell<cmd_display_panes_data>>);

impl DisplayPanesRef {
    pub(crate) fn new(value: cmd_display_panes_data) -> Self {
        Self(std::rc::Rc::new(std::cell::RefCell::new(value)))
    }

    pub(crate) fn borrow(&self) -> std::cell::Ref<'_, cmd_display_panes_data> {
        self.0.borrow()
    }

    pub(crate) fn borrow_mut(&self) -> std::cell::RefMut<'_, cmd_display_panes_data> {
        self.0.borrow_mut()
    }
}

impl PartialEq for DisplayPanesRef {
    fn eq(&self, other: &Self) -> bool {
        std::rc::Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for DisplayPanesRef {}

impl std::fmt::Debug for DisplayPanesRef {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("DisplayPanesRef")
            .field(&std::rc::Rc::as_ptr(&self.0))
            .finish()
    }
}

pub(crate) static cmd_display_panes_entry: RustCommandEntry = RustCommandEntry {
    name: c"display-panes",
    alias: Some(c"displayp"),
    args: args_parse_t {
        template: c"bd:Nt:",
        lower: 0,
        upper: 1,
        cb: Some(cmd_display_panes_args_parse),
    },
    usage: c"[-bN] [-d duration] [-t target-client] [template]",
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
    flags: CMD_AFTERHOOK | CMD_CLIENT_TFLAG,
    exec: cmd_display_panes_exec,
};

/// How the parser is told to read the optional template: as a command list if
/// it parses as one, and as a plain string otherwise.
fn cmd_display_panes_args_parse(
    _args: &args,
    _idx: u_int,
    _cause: &mut Option<CString>,
) -> args_parse_type {
    ARGS_PARSE_COMMANDS_OR_STRING
}

/// The pane index a key press names, if it names one: a digit is that index,
/// and an unmodified letter continues from ten. The digits are tested against
/// the whole key before the modifiers are masked off, so a modified digit is
/// not one.
fn cmd_display_panes_index(key: key_code) -> Option<u_int> {
    if (b'0' as key_code..=b'9' as key_code).contains(&key) {
        return Some(key.wrapping_sub(b'0' as key_code) as u_int);
    }
    if key as c_ulonglong & KEYC_MASK_MODIFIERS != 0 {
        return None;
    }
    let key = (key as c_ulonglong & KEYC_MASK_KEY) as key_code;
    match (b'a' as key_code..=b'z' as key_code).contains(&key) {
        true => Some((10 as key_code).wrapping_add(key.wrapping_sub(b'a' as key_code)) as u_int),
        false => None,
    }
}

/// How long the numbers stay up: what `-d` says, or `display-panes-time`. An
/// unusable `-d` is the command's own error.
unsafe fn cmd_display_panes_delay(
    args: &RustArguments,
    item: &cmdq_item,
    s: Option<&SessionRef>,
) -> Option<u_int> {
    unsafe {
        if args_has(args, b'd') == 0 {
            let s = s?;
            return Some(s.options().number(c"display-panes-time") as u_int);
        }
        let mut cause = None;
        let delay = args_strtonum(args, b'd', 0, UINT_MAX as c_longlong, &mut cause) as u_int;
        if let Some(cause) = cause.as_ref() {
            item.error(c"delay %s", fmt_args![cause.as_c_str()]);
            return None;
        }
        Some(delay)
    }
}

unsafe fn cmd_display_panes_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let mut tc = item
        .target_client()
        .expect("the command has a target client");

    let wait = args_has(args, b'b') == 0;

    if tc.overlay().is_some() {
        return CMD_RETURN_NORMAL;
    }
    let session = { tc.attached_session() };
    let delay = match unsafe { cmd_display_panes_delay(args, item, session.as_ref()) } {
        Some(delay) => delay,
        None => return CMD_RETURN_ERROR,
    };

    let mut cdata = cmd_display_panes_data::default();
    if wait {
        cdata.item = cmdq_item_weak_of(item);
    }
    cdata.state = Some(args_make_commands_prepare(
        self_0,
        item,
        0,
        Some(c"select-pane -t \"%%%\""),
        wait as c_int,
        ::core::ffi::CStr::to_owned,
    ));

    let overlay = Overlay::DisplayPanes {
        keys: args_has(args, b'N') == 0,
    };
    unsafe {
        client_set_overlay(
            &mut tc,
            delay,
            overlay,
            OverlayState::DisplayPanes(DisplayPanesRef::new(cdata)),
        )
    };

    match wait {
        true => CMD_RETURN_WAIT,
        false => CMD_RETURN_NORMAL,
    }
}

#[cfg(test)]
#[path = "../../tests/test_cmd_display_panes.rs"]
mod tests;

impl DisplayPanesRef {
    /// Gives the private state back once the numbers are down, however they went,
    /// and lets whatever was waiting on them carry on.
    #[allow(clippy::boxed_local)]
    pub(crate) fn close(self) {
        let data = self;

        let item = data.borrow().item.as_ref().and_then(CmdqItemWeak::upgrade);
        if let Some(item) = item {
            item.resume();
        }
    }
    /// The overlay's key callback. A key naming a pane the window has runs the
    /// template against that pane's id — spliced in behind the waiting item, or
    /// appended to the client's own queue when nothing is waiting — and a template
    /// that does not parse is reported instead. Anything else is refused, which is
    /// what takes the numbers down.
    pub(crate) unsafe fn key(&self, c: &ClientRef, event: &key_event) -> c_int {
        let owner = self;

        unsafe {
            let item = owner.borrow().item.as_ref().and_then(CmdqItemWeak::upgrade);

            let index = match cmd_display_panes_index(event.key) {
                Some(index) => index,
                None => return -1,
            };
            let Some(session) = c.attached_session() else {
                return -1;
            };
            let Some(window) = session.current_window() else {
                return -1;
            };
            let Some(pane) = window.pane_at_index(index) else {
                return 1;
            };
            window.unzoom(1);

            let expanded = xasprintf(c"%%%u", fmt_args![pane.id()]);
            let mut error = None;
            let cmdlist = args_make_commands(
                owner.borrow_mut().state.as_deref_mut().unwrap(),
                &[expanded],
                &mut error,
            );
            if let Some(error) = error.as_ref() {
                cmdq_append(Some(c), CmdqItemRef::error_items(error));
            } else if let Some(item) = &item {
                let state = item.state_ref();
                item.insert_after((cmdlist.as_ref().unwrap()).queue_items(Some(&state)));
            } else {
                cmdq_append(Some(c), (cmdlist.as_ref().unwrap()).queue_items(None));
            }
            1
        }
    }
}

#[cfg(test)]
use crate::overlay::draw_pane_numbers as cmd_display_panes_draw;
