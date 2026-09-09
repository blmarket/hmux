//! `new-window`: creates a window in a session and links it at an index.
//!
//! The hook decides four things before the spawn: the window's name, whether
//! a window of that name is to be selected instead of a new one made, which
//! index the new window is linked at, and what the spawn is told to do. `-n`
//! names it, through the format engine and the same validity check the rename
//! commands use; `-S` with `-n` and no `-t` index searches the session for a
//! window already carrying that name and selects it — refusing when two
//! windows share it, and with `-d` selecting nothing at all; `-a` and `-b`
//! shuffle the windows above the target up to open an index beside it; and
//! `-d` and `-k` become the spawn's detached and kill-what-is-there flags.
//!
//! Everything after that is `spawn_window`'s: the index it refuses when it is
//! already linked, the window and its first pane, and the process behind that
//! pane. What comes back is either a failure this hook reports and cleans up
//! after, or a new winlink it re-finds the current state from, redraws or
//! restatuses the session group for, prints under `-P`'s template, and fires
//! the `after-new-window` hook against.
//!
//! Upstream quirk kept: the `-S` search walks the session's windows with the
//! very variable it took the target's winlink from, so a search that finds
//! nothing leaves the hook with no winlink at all — and the `-a`/`-b` shuffle
//! below it, which needs one, then does nothing and hands back the target's
//! own index. The conversion nulls that variable where the C's loop left it.

use crate::args::RustArguments;
use crate::args::args_value_list;
use crate::cmd::cmd_get_args;
use crate::cmd::cmdq_item_weak_of;

use crate::environ::EnvironmentStore;
use crate::environ::{RustEnvironment, new_environment_box};
use crate::fmt_args;
use crate::format::{format_create_for_client, format_defaults_for_handles, format_expand};
use crate::resize::recalculate_sizes;

use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::consts::{
    CMD_FIND_PANE, CMD_FIND_WINDOW, CMD_FIND_WINDOW_INDEX, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
    SPAWN_DETACHED, SPAWN_KILL,
};
use crate::spawn::spawn_window;
use crate::tmux::{check_name, clean_name};
use crate::types::{ClientRef, SessionRef, args_parse_t, cmd_find_state, spawn_context};
#[cfg(test)]
use crate::types::{tmuxpeer, winlink};
use ::core::ffi::{CStr, c_int};
use ::std::ffi::CString;

pub const NEW_WINDOW_TEMPLATE: &CStr = c"#{session_name}:#{window_index}.#{pane_index}";
pub(crate) static cmd_new_window_entry: RustCommandEntry = RustCommandEntry {
    name: c"new-window",
    alias: Some(c"neww"),
    args: args_parse_t {
        template: c"abc:de:F:kn:PSt:",
        lower: 0,
        upper: -1,
        cb: None,
    },
    usage: c"[-abdkPS] [-c start-directory] [-e environment] [-F format] [-n window-name] [-t target-window] [shell-command [argument ...]]"
        ,
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as core::ffi::c_char,
        type_0: CMD_FIND_WINDOW,
        flags: CMD_FIND_WINDOW_INDEX,
    },
    flags: 0,
    exec: cmd_new_window_exec,
};

/// Selects the window at `idx` in `s` the way `-S` without `-d` does, leaving
/// the client that asked as the selected window's latest.
unsafe fn select_found_window(session: &SessionRef, idx: c_int, c: Option<&ClientRef>) {
    unsafe {
        if session.set_current(Some(idx)) == 0 {
            session.request_redraw();
        }
        if let Some(c) = c
            && !c.attached_session().is_none()
        {
            let owner = session
                .current_link()
                .and_then(|link| link.window())
                .expect("the selected window has an owner");
            owner.set_latest_client(Some(c));
        }
        recalculate_sizes();
    }
}

/// The environment the spawn is given, which is a set of its own carrying
/// whatever `-e` asked for, even when nothing did.
fn spawn_environ(args: &RustArguments) -> Box<RustEnvironment> {
    let mut env = new_environment_box();
    for av in args_value_list(args, b'e') {
        env.put(av.string(), 0);
    }
    env
}

/// Gives back what the hook allocated for the spawn, whichever way the spawn
/// went.
fn free_spawn_context(sc: &mut spawn_context) {
    drop(sc.environ.take());
}

unsafe fn cmd_new_window_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let c = item.client();
    let current_state_ref = item.state_ref();
    let target_client = item.target_client();
    let tc = target_client.clone();
    let mut session = item
        .target
        .session()
        .expect("a new-window target has a session");
    let mut around = item
        .target
        .wl_idx
        .filter(|index| unsafe { session.link(*index).is_some() });
    let target_idx = item.target.idx;
    let mut idx = target_idx;
    let mut wname: Option<CString> = None;

    if let Some(name) = args.argument_flag_string(b'n') {
        let expanded = unsafe {
            let mut ft = format_create_for_client(item.client().as_ref(), Some(item), 0, 0);
            format_defaults_for_handles(&mut ft, c.as_ref(), Some(&session), None, None);
            format_expand(&mut ft, name)
        };
        if unsafe { check_name(Some(&expanded)) == 0 } {
            unsafe { item.error(c"invalid window name: %s", fmt_args![expanded.as_c_str()]) };
            return CMD_RETURN_ERROR;
        }
        unsafe { wname = clean_name(&expanded, 0) };
    }
    if args.argument_flag_count(b'S') != 0
        && let Some(wname) = wname.as_ref()
        && target_idx == -1
    {
        let expanded = unsafe {
            let mut ft = format_create_for_client(item.client().as_ref(), Some(item), 0, 0);
            format_defaults_for_handles(&mut ft, c.as_ref(), Some(&session), None, None);
            format_expand(&mut ft, wname.as_c_str())
        };
        let found = unsafe { session.unique_window_index_named(&expanded) };
        around = None;
        let found_idx = match found {
            Ok(found_idx) => found_idx,
            Err(()) => {
                unsafe { item.error(c"multiple windows named %s", fmt_args![wname.as_c_str()]) };
                return CMD_RETURN_ERROR;
            }
        };
        if let Some(found_idx) = found_idx {
            if args.argument_flag_count(b'd') != 0 {
                return CMD_RETURN_NORMAL;
            }
            unsafe { select_found_window(&session, found_idx, c.as_ref()) };
            return CMD_RETURN_NORMAL;
        }
    }

    let before = args.argument_flag_count(b'b');
    if args.argument_flag_count(b'a') != 0 || before != 0 {
        unsafe {
            idx = session
                .shuffle_windows(around, before)
                .map_or(target_idx, |shift| shift.index)
        };
    }

    let mut sc = spawn_context {
        item: cmdq_item_weak_of(item),
        s: Some(session.clone()),
        tc: target_client.as_ref().map(ClientRef::downgrade),
        name: wname.as_deref(),
        ..Default::default()
    };
    unsafe { sc.argv = args.to_vector() };
    sc.environ = Some(spawn_environ(args));
    sc.idx = idx;
    sc.cwd = args.argument_flag_string(b'c');
    sc.flags = 0;
    if args.argument_flag_count(b'd') != 0 {
        sc.flags |= SPAWN_DETACHED;
    }
    if args.argument_flag_count(b'k') != 0 {
        sc.flags |= SPAWN_KILL;
    }

    let mut cause: Option<CString> = None;
    let Some(new_wl) = (unsafe { spawn_window(&mut sc, &mut cause) }) else {
        let cause = cause.unwrap();
        unsafe { item.error(c"create window failed: %s", fmt_args![cause.as_c_str()]) };
        free_spawn_context(&mut sc);
        return CMD_RETURN_ERROR;
    };

    if unsafe { args.argument_flag_count(b'd') == 0 || session.current_index() == Some(new_wl.index()) } {
        unsafe { current_state_ref.update_current_link(&new_wl, None, 0) };
        unsafe { session.redraw_group() };
    } else {
        unsafe { session.status_group() };
    }

    if args.argument_flag_count(b'P') != 0 {
        let template = args
            .argument_flag_string(b'F')
            .unwrap_or(NEW_WINDOW_TEMPLATE);
        let owner = new_wl.window().expect("the spawned window has an owner");
        let pane = owner.active_pane_id().and_then(|id| owner.pane_by_id(id));
        let cp = unsafe {
            let mut ft = format_create_for_client(item.client().as_ref(), Some(item), 0, 0);
            format_defaults_for_handles(
                &mut ft,
                tc.as_ref(),
                Some(&session),
                Some(&new_wl),
                pane.as_ref(),
            );
            format_expand(&mut ft, template)
        };
        unsafe { item.print(c"%s", fmt_args![cp.as_c_str()]) };
    }

    let mut fs = cmd_find_state::default();
    unsafe { crate::cmd::cmd_find_from_link_ref(&mut fs, &new_wl, None, 0) };
    unsafe {
        (crate::cmd::cmdq_item_ref_of(item).expect("the command has an owner")).insert_session_hook(
            Some(&session),
            Some(&fs),
            c"after-new-window",
            fmt_args![],
        )
    };

    free_spawn_context(&mut sc);
    CMD_RETURN_NORMAL
}

#[cfg(test)]
#[path = "../../tests/test_cmd_new_window.rs"]
mod tests;
