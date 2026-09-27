use crate::src::arguments::{
    args_count, args_first_value, args_get, args_has, args_next_value, args_string, args_to_vector,
};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd::find::cmd_find_from_winlink;
use crate::src::cmd::queue::{
    cmdq_error, cmdq_get_client, cmdq_get_current, cmdq_get_target, cmdq_get_target_client,
    cmdq_insert_hook, cmdq_print,
};
use crate::src::environ::{environ_create, environ_free, environ_put};
use crate::src::ffi::libc::strcmp;
use crate::src::format::bytes::write_cstr;
use crate::src::format::format_single_cstring;
use crate::src::resize::recalculate_sizes;
use crate::src::server_fn::{
    server_redraw_session, server_redraw_session_group, server_status_session_group,
};
use crate::src::session::session_set_current;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse, args_value};
use crate::src::shared::client::client;
use crate::src::shared::command::CMD_FIND_WINDOW_INDEX;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::environment::environ;
use crate::src::shared::layout::layout_cell;
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::spawn::spawn_context;
use crate::src::shared::spawn::{SPAWN_DETACHED, SPAWN_EMPTY, SPAWN_KILL};
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::window::{window, winlink};
use crate::src::spawn::spawn_window;
use crate::src::tmux::{check_name, clean_name_cstring};
use crate::src::window::{
    winlink_find_by_index, winlink_shuffle_up, winlinks_minmax, winlinks_next,
};
use std::ffi::{CStr, CString};

pub const NEW_WINDOW_TEMPLATE: [::core::ffi::c_char; 46] = unsafe {
    ::core::mem::transmute::<[u8; 46], [::core::ffi::c_char; 46]>(
        *b"#{session_name}:#{window_index}.#{pane_index}\0",
    )
};
pub static mut cmd_new_window_entry: cmd_entry = {
    cmd_entry {
        name: c"new-window",
        alias: Some(c"neww"),
        args: args_parse {
            template: b"abc:de:EF:kn:PSt:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: c"[-abdEkPS] [-c start-directory] [-e environment] [-F format] [-n window-name] [-t target-window] [shell-command [argument ...]]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: CMD_FIND_WINDOW_INDEX,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_new_window_exec
                as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe fn cmd_new_window_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut c: *mut client = cmdq_get_client(item);
    let mut current: *mut cmd_find_state = cmdq_get_current(item);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut sc: spawn_context = spawn_context {
        item: ::core::ptr::null_mut::<cmdq_item>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        tc: ::core::ptr::null_mut::<client>(),
        wp0: ::core::ptr::null_mut::<window_pane>(),
        lc: ::core::ptr::null_mut::<layout_cell>(),
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        argv: Vec::new(),
        environ: ::core::ptr::null_mut::<environ>(),
        idx: 0,
        cwd: ::core::ptr::null::<::core::ffi::c_char>(),
        flags: 0,
    };
    let mut argv_owner = Vec::new();
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut s: *mut session = (*target).s;
    let mut wl: *mut winlink = (*target).wl;
    let mut new_wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut idx: ::core::ffi::c_int = (*target).idx;
    let mut before: ::core::ffi::c_int = 0;
    let mut count: ::core::ffi::c_int = args_count(args) as ::core::ffi::c_int;
    let mut cause: Option<std::ffi::CString> = None;
    let mut wname: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut wname_owned: Option<CString> = None;
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut av: *mut args_value = ::core::ptr::null_mut::<args_value>();
    if args_has(args, 'E' as i32 as u_char) != 0
        && count != 0 as ::core::ffi::c_int
        && (count != 1 as ::core::ffi::c_int
            || *args_string(args, 0 as u_int) as ::core::ffi::c_int != '\0' as i32)
    {
        cmdq_error(item, |out| {
            out.write_all(b"command cannot be given for empty pane")
        });
        return CMD_RETURN_ERROR;
    }
    name = args_get(args, 'n' as i32 as u_char);
    if !name.is_null() {
        let expanded = format_single_cstring(
            item,
            name,
            c,
            s,
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
        if check_name(expanded.as_ptr()) == 0 {
            cmdq_error(item, |out| {
                out.write_all(b"invalid window name: ")?;
                out.write_all(expanded.as_bytes())
            });
            return CMD_RETURN_ERROR;
        }
        wname_owned = Some(
            clean_name_cstring(expanded.as_c_str(), 0)
                .expect("check_name validated the window name"),
        );
        wname = wname_owned
            .as_ref()
            .expect("window name was cleaned")
            .as_ptr();
    }
    if args_has(args, 'S' as i32 as u_char) != 0 {
        if idx != -(1 as ::core::ffi::c_int) {
            new_wl = winlink_find_by_index(&raw mut (*s).windows, idx);
        } else if !wname.is_null() {
            let expanded = format_single_cstring(
                item,
                wname,
                c,
                s,
                ::core::ptr::null_mut::<winlink>(),
                ::core::ptr::null_mut::<window_pane>(),
            );
            wl = winlinks_minmax(&(*s).windows, RB_NEGINF);
            while !wl.is_null() {
                if !(strcmp((*(*wl).window).name.as_ptr(), expanded.as_ptr())
                    != 0 as ::core::ffi::c_int)
                {
                    if new_wl.is_null() {
                        new_wl = wl;
                    } else {
                        cmdq_error(item, |out| {
                            out.write_all(b"multiple windows named ")?;
                            write_cstr(out, wname)
                        });
                        return CMD_RETURN_ERROR;
                    }
                }
                wl = winlinks_next(&*wl);
            }
        }
    }
    if !new_wl.is_null() {
        if args_has(args, 'd' as i32 as u_char) != 0 {
            return CMD_RETURN_NORMAL;
        }
        if session_set_current(s, new_wl) == 0 as ::core::ffi::c_int {
            server_redraw_session(s);
        }
        if !c.is_null() && !(*c).session.is_null() {
            (*(*(*s).curw).window).latest = c as *mut ::core::ffi::c_void;
        }
        recalculate_sizes();
        return CMD_RETURN_NORMAL;
    }
    before = args_has(args, 'b' as i32 as u_char);
    if args_has(args, 'a' as i32 as u_char) != 0 || before != 0 {
        idx = winlink_shuffle_up(s, wl, before);
        if idx == -(1 as ::core::ffi::c_int) {
            idx = (*target).idx;
        }
    }
    sc.item = item;
    sc.s = s;
    sc.tc = tc;
    sc.name = wname;
    argv_owner = args_to_vector(&*args);
    sc.argv = argv_owner;
    sc.environ = environ_create();
    av = args_first_value(args, 'e' as i32 as u_char);
    while !av.is_null() {
        environ_put(sc.environ, (*av).string_ptr(), 0 as ::core::ffi::c_int);
        av = args_next_value(av);
    }
    sc.idx = idx;
    sc.cwd = args_get(args, 'c' as i32 as u_char);
    sc.flags = 0 as ::core::ffi::c_int;
    if args_has(args, 'E' as i32 as u_char) != 0
        || count == 1 as ::core::ffi::c_int
            && *args_string(args, 0 as u_int) as ::core::ffi::c_int == '\0' as i32
    {
        sc.flags |= SPAWN_EMPTY;
    }
    if args_has(args, 'd' as i32 as u_char) != 0 {
        sc.flags |= SPAWN_DETACHED;
    }
    if args_has(args, 'k' as i32 as u_char) != 0 {
        sc.flags |= SPAWN_KILL;
    }
    new_wl = spawn_window(&raw mut sc, &raw mut cause);
    if new_wl.is_null() {
        cmdq_error(item, |out| {
            out.write_all(b"create window failed: ")?;
            write_cstr(
                out,
                cause
                    .as_ref()
                    .map_or(::core::ptr::null(), |value| value.as_ptr()),
            )
        });
        environ_free(sc.environ);
        return CMD_RETURN_ERROR;
    } else {
        if args_has(args, 'd' as i32 as u_char) == 0 || new_wl == (*s).curw {
            cmd_find_from_winlink(current, new_wl, 0 as ::core::ffi::c_int);
            server_redraw_session_group(s);
        } else {
            server_status_session_group(s);
        }
        if args_has(args, 'P' as i32 as u_char) != 0 {
            template = args_get(args, 'F' as i32 as u_char);
            if template.is_null() {
                template = NEW_WINDOW_TEMPLATE.as_ptr();
            }
            let cp =
                format_single_cstring(item, template, tc, s, new_wl, (*(*new_wl).window).active);
            cmdq_print(item, |out| out.write_all(cp.as_bytes()));
        }
        cmd_find_from_winlink(&raw mut fs, new_wl, 0 as ::core::ffi::c_int);
        cmdq_insert_hook(s, item, &raw mut fs, |out| {
            out.write_all(b"after-new-window")
        });
        environ_free(sc.environ);
        return CMD_RETURN_NORMAL;
    };
}
