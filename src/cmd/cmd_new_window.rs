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
//!
//! Coverage exemptions: every line of the success tail — the current-state
//! re-find, the redraw or status update, the `-P` print, the
//! `after-new-window` hook and the frees behind them — the `SPAWN_KILL` flag
//! `-k` sets, and the two block ends whose only way on is the spawn
//! succeeding or a `-S` search finding nothing. A `new-window` that gets past
//! `spawn_window` has reached `spawn_pane`, which forks a pty child; `-k`
//! only ever unlinks the window in the way and carries on to that same fork,
//! and a `-S` search that matches nothing falls through to it as well. No
//! unit test may go there. Every refusal in front of the spawn is covered.
use crate::arguments::args_get_str;
use crate::arguments::{args_get, args_has, args_to_vector, args_value_list};
use crate::cmd::cmd_get_args;
use crate::cmd::find::cmd_find_from_winlink;
use crate::cmd::queue::cmdq_item_weak_from_ptr;
use crate::cmd::queue::{
    cmdq_error, cmdq_get_client, cmdq_get_current, cmdq_get_target, cmdq_get_target_client,
    cmdq_insert_hook, cmdq_print,
};
use crate::environ::{environ_create_box, environ_put, environ_t};
use crate::fmt_args;
use crate::format::format_single;
use crate::resize::recalculate_sizes;
use crate::server::client_weak_from_ptr;
use crate::server::{
    server_redraw_session, server_redraw_session_group, server_status_session_group,
};
use crate::session::session_get_curw;
use crate::session::session_set_current;
use crate::spawn::spawn_window;
use crate::tmux::{check_name, clean_name};
pub use crate::types::*;
use crate::window::window_get_active;
use crate::window::window_set_latest;
use crate::window::{winlink_shuffle_up, winlinks_after, winlinks_first};
use ::core::ffi::CStr;
use ::core::ptr::null_mut;
use ::std::ffi::CString;
pub const CMD_FIND_WINDOW: cmd_find_type = 1;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const CMD_FIND_WINDOW_INDEX: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const SPAWN_KILL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const SPAWN_DETACHED: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const NEW_WINDOW_TEMPLATE: [::core::ffi::c_char; 46] = unsafe {
    ::core::mem::transmute::<[u8; 46], [::core::ffi::c_char; 46]>(
        *b"#{session_name}:#{window_index}.#{pane_index}\0",
    )
};
pub(crate) static cmd_new_window_entry: cmd_entry = cmd_entry {
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
        flag: b't' as ::core::ffi::c_char,
        type_0: CMD_FIND_WINDOW,
        flags: CMD_FIND_WINDOW_INDEX,
    },
    flags: 0,
    exec: cmd_new_window_exec,
};

/// A spawn context with everything cleared, which is what the C's `= { 0 }`
/// gave the hook.
fn empty_spawn_context<'a>() -> spawn_context<'a> {
    spawn_context::default()
}

/// The winlinks of `s`, in index order.
unsafe fn winlinks_of(s: *mut session) -> impl Iterator<Item = *mut winlink> {
    let mut wl = unsafe { winlinks_first(&raw mut (*s).windows) };
    ::core::iter::from_fn(move || {
        let this = wl;
        if this.is_null() {
            return None;
        }
        wl = unsafe { winlinks_after(this) };
        Some(this)
    })
}

/// The one window of `s` whose name is `name`, as `Ok(null)` when none carries
/// it and as `Err` when more than one does.
unsafe fn only_window_named(s: *mut session, name: &CStr) -> Result<*mut winlink, ()> {
    unsafe {
        let mut found: *mut winlink = null_mut();
        for wl in winlinks_of(s) {
            if (*(*wl).window()).name.as_deref() != Some(name) {
                continue;
            }
            if found.is_null() {
                found = wl;
                continue;
            }
            return Err(());
        }
        Ok(found)
    }
}

/// Selects `new_wl` in `s` the way `-S` without `-d` does, leaving the client
/// that asked as the selected window's latest.
unsafe fn select_found_window(s: *mut session, new_wl: *mut winlink, c: *mut client) {
    unsafe {
        if session_set_current(s, new_wl) == 0 {
            server_redraw_session(s);
        }
        if !c.is_null() && !(*c).session.is_null() {
            window_set_latest((*session_get_curw(s)).window(), c);
        }
        recalculate_sizes();
    }
}

/// The environment the spawn is given, which is a set of its own carrying
/// whatever `-e` asked for, even when nothing did.
unsafe fn spawn_environ(args: &args) -> Box<environ_t> {
    unsafe {
        let mut env = environ_create_box();
        let env_ptr = &raw mut *env;
        for av in args_value_list(args, b'e') {
            environ_put(env_ptr, (*av).value.string().as_ptr(), 0);
        }
        env
    }
}

/// Gives back what the hook allocated for the spawn, whichever way the spawn
/// went.
fn free_spawn_context(sc: &mut spawn_context) {
    drop(sc.environ.take());
}

unsafe fn cmd_new_window_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let c = cmdq_get_client(&*item);
        let current = cmdq_get_current(item);
        let target = cmdq_get_target(item);
        let tc = cmdq_get_target_client(&*item);
        let s = (*target).session();
        let mut wl = (*target).winlink();
        let mut idx = (*target).idx;
        let mut wname: Option<CString> = None;

        let name = args_get(args, b'n');
        if !name.is_null() {
            let expanded = format_single(
                item,
                ::core::ffi::CStr::from_ptr(name),
                c,
                s,
                null_mut(),
                null_mut(),
            );
            if check_name(expanded.as_ptr()) == 0 {
                cmdq_error(
                    item,
                    c"invalid window name: %s".as_ptr(),
                    fmt_args![expanded.as_ptr()],
                );
                return CMD_RETURN_ERROR;
            }
            wname = clean_name(expanded.as_ptr(), 0);
        }
        if args_has(args, b'S') != 0 && wname.is_some() && (*target).idx == -1 {
            let wname_ptr = wname.as_ref().unwrap().as_ptr();
            let expanded = format_single(
                item,
                ::core::ffi::CStr::from_ptr(wname_ptr),
                c,
                s,
                null_mut(),
                null_mut(),
            );
            let found = only_window_named(s, &expanded);
            wl = null_mut();
            let new_wl = match found {
                Ok(new_wl) => new_wl,
                Err(()) => {
                    cmdq_error(
                        item,
                        c"multiple windows named %s".as_ptr(),
                        fmt_args![wname_ptr],
                    );
                    return CMD_RETURN_ERROR;
                }
            };
            if !new_wl.is_null() {
                if args_has(args, b'd') != 0 {
                    return CMD_RETURN_NORMAL;
                }
                select_found_window(s, new_wl, c);
                return CMD_RETURN_NORMAL;
            }
        }

        let before = args_has(args, b'b');
        if args_has(args, b'a') != 0 || before != 0 {
            idx = winlink_shuffle_up(s, wl, before);
            if idx == -1 {
                idx = (*target).idx;
            }
        }

        let mut sc = empty_spawn_context();
        sc.item = cmdq_item_weak_from_ptr(item);
        sc.s = s;
        sc.tc = client_weak_from_ptr(tc);
        sc.name = wname.as_deref();
        sc.argv = args_to_vector(args);
        sc.environ = Some(spawn_environ(args));
        sc.idx = idx;
        sc.cwd = args_get_str(args, b'c');
        sc.flags = 0;
        if args_has(args, b'd') != 0 {
            sc.flags |= SPAWN_DETACHED;
        }
        if args_has(args, b'k') != 0 {
            sc.flags |= SPAWN_KILL;
        }

        let mut cause: Option<CString> = None;
        let new_wl = spawn_window(&mut sc, &mut cause);
        if new_wl.is_null() {
            let cause = cause.unwrap();
            cmdq_error(
                item,
                c"create window failed: %s".as_ptr(),
                fmt_args![cause.as_ptr()],
            );
            free_spawn_context(&mut sc);
            return CMD_RETURN_ERROR;
        }

        if args_has(args, b'd') == 0 || new_wl == session_get_curw(s) {
            cmd_find_from_winlink(&mut *current, new_wl, 0);
            server_redraw_session_group(s);
        } else {
            server_status_session_group(s);
        }

        if args_has(args, b'P') != 0 {
            let mut template = args_get(args, b'F');
            if template.is_null() {
                template = NEW_WINDOW_TEMPLATE.as_ptr();
            }
            let cp = format_single(
                item,
                ::core::ffi::CStr::from_ptr(template),
                tc,
                s,
                new_wl,
                window_get_active((*new_wl).window()),
            );
            cmdq_print(item, c"%s".as_ptr(), fmt_args![cp.as_ptr()]);
        }

        let mut fs = cmd_find_state::default();
        cmd_find_from_winlink(&mut fs, new_wl, 0);
        cmdq_insert_hook(
            s,
            item,
            &raw mut fs,
            c"after-new-window".as_ptr(),
            fmt_args![],
        );

        free_spawn_context(&mut sc);
        CMD_RETURN_NORMAL
    }
}

#[cfg(test)]
#[path = "../tests/test_cmd_new_window.rs"]
mod tests;
