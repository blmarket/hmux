//! `move-window` and `link-window`: one exec hook, told apart by which entry
//! the running command was parsed as — the link keeps the window where it was,
//! the move unlinks it afterwards.
//!
//! `-t` is special here: the command resolves it itself rather than letting the
//! queue do it, because it wants `CMD_FIND_WINDOW_INDEX`, which answers an
//! index no window holds with the index alone and a null winlink. `-a` and `-b`
//! then make room with [`SessionRef::shuffle_windows`] — around the target window when
//! one was found, and around the destination session's *current* window when
//! `-t` named a free index — and refuse when there is no free index left above
//! the one they were given. `-r` is a different command altogether: it
//! renumbers a session's windows and answers, without moving anything.
//!
//! `server_link_window` is what actually links the window into the destination
//! and reports why it would not; `-k` lets it take an index already in use and
//! `-d` keeps the destination from selecting the newcomer. Afterwards the
//! *source* session is renumbered when its `renumber-windows` is on, unless
//! `-s` named the source, since the destination is already where the caller
//! asked for.

use crate::arguments::{args_get_str, args_has};
use crate::cmd::find::cmd_find_target;

use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::fmt_args;

use crate::resize::recalculate_sizes;
use crate::server::server_link_window;

pub use crate::consts::{
    CMD_FIND_PANE, CMD_FIND_QUIET, CMD_FIND_SESSION, CMD_FIND_WINDOW, CMD_FIND_WINDOW_INDEX,
    CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
};
pub use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_find_type, cmd_retval, cmdq_item};
pub use crate::types::{OptionsRef, SessionRef, args_parse_t, cmd_find_state};
#[cfg(test)]
use crate::types::{WindowRef, u_int};
use crate::window::{WinlinkRef, WinlinkShuffle};
use ::core::ffi::{CStr, c_char, c_int};

pub(crate) static cmd_move_window_entry: RustCommandEntry = RustCommandEntry {
    name: c"move-window",
    alias: Some(c"movew"),
    args: args_parse_t {
        template: c"abdkrs:t:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-abdkr] [-s src-window] [-t dst-window]",
    source: cmd_entry_flag {
        flag: b's' as c_char,
        type_0: CMD_FIND_WINDOW,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: 0,
    exec: cmd_move_window_exec,
};
pub(crate) static cmd_link_window_entry: RustCommandEntry = RustCommandEntry {
    name: c"link-window",
    alias: Some(c"linkw"),
    args: args_parse_t {
        template: c"abdks:t:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-abdk] [-s src-window] [-t dst-window]",
    source: cmd_entry_flag {
        flag: b's' as c_char,
        type_0: CMD_FIND_WINDOW,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: 0,
    exec: cmd_move_window_exec,
};

/// What `-t` resolves to, or nothing when it resolves to nothing at all. The
/// entries leave their own target unset, so this hook does the resolution the
/// command queue would otherwise have done, and picks the flags itself.
unsafe fn resolve(
    item: &cmdq_item,
    tflag: Option<&CStr>,
    type_0: cmd_find_type,
    flags: c_int,
) -> Option<cmd_find_state> {
    unsafe {
        let mut target = cmd_find_state::default();
        (cmd_find_target(&mut target, item, tflag, type_0, flags) == 0).then_some(target)
    }
}

/// The index `-a` or `-b` frees up in `dst`: around the target window when
/// `-t` found one, and around the destination's current window when
/// `CMD_FIND_WINDOW_INDEX` answered a `-t` index no window holds. Nothing when
/// `SessionRef::shuffle_windows` finds no free index left above the one it was given.
///
/// # Safety
/// Exclude conflicting session and linked-window access through the shuffle.
/// Queries finish before mutation; this helper retains no payload borrows.
unsafe fn room_for(
    dst: &mut SessionRef,
    target: &cmd_find_state,
    before: c_int,
) -> Option<WinlinkShuffle> {
    unsafe {
        let around = target
            .wl_idx
            .filter(|index| dst.link(*index).is_some())
            .or_else(|| dst.current_index());
        dst.shuffle_windows(around, before)
    }
}

unsafe fn cmd_move_window_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let tflag = args_get_str(args, b't');
    if args_has(args, b'r') != 0 {
        let Some(target) = (unsafe { resolve(item, tflag, CMD_FIND_SESSION, CMD_FIND_QUIET) })
        else {
            return CMD_RETURN_ERROR;
        };
        if let Some(session) = target.session() {
            unsafe { session.renumber_windows() };
        }
        recalculate_sizes();
        if let Some(session) = target.session() {
            unsafe { session.request_status() };
        }
        return CMD_RETURN_NORMAL;
    }

    let mut source = item.source.session().expect("a move source has a session");
    let mut source_index = item.source.wl_idx.expect("a move source has a window link");
    let Some(target) = (unsafe { resolve(item, tflag, CMD_FIND_WINDOW, CMD_FIND_WINDOW_INDEX) })
    else {
        return CMD_RETURN_ERROR;
    };
    let mut destination = target.session().expect("a move destination has a session");
    let mut index = target.idx;
    let kflag = args_has(args, b'k');
    let dflag = args_has(args, b'd');
    let sflag = args_has(args, b's');
    let before = args_has(args, b'b');
    if args_has(args, b'a') != 0 || before != 0 {
        let Some(shift) = (unsafe { room_for(&mut destination, &target, before) }) else {
            return CMD_RETURN_ERROR;
        };
        index = shift.index;
        if source.ptr_eq(&destination) {
            source_index = shift.remap(source_index);
        }
    }

    let source_link =
        WinlinkRef::new(source.clone(), source_index).expect("the source link is present");
    let mut cause = None;
    if unsafe {
        server_link_window(
            &source_link,
            &mut destination,
            index,
            kflag,
            (dflag == 0) as c_int,
            &mut cause,
        ) != 0
    } {
        unsafe { item.error(c"%s", fmt_args![cause.as_deref()]) };
        return CMD_RETURN_ERROR;
    }
    if cmd_get_entry(self_0).name == cmd_move_window_entry.name {
        unsafe { source.unlink_window(source_index) };
    }
    if unsafe { sflag == 0 && source.options().number(c"renumber-windows") != 0 } {
        unsafe { source.renumber_windows() };
    }
    recalculate_sizes();
    CMD_RETURN_NORMAL
}

#[cfg(test)]
#[path = "../tests/test_cmd_move_window.rs"]
mod tests;
