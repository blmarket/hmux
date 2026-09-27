use crate::src::arguments::{args_flag_values, args_get, args_has, args_to_vector};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_target};
use crate::src::environ::{environ_create, environ_put};
use crate::src::format::bytes::write_cstr;
use crate::src::server_fn::{server_redraw_window_borders, server_status_window};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse, args_value};
use crate::src::shared::client::client;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::environment::environ;
use crate::src::shared::layout::layout_cell;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::PANE_REDRAW;
use crate::src::shared::session::session;
use crate::src::shared::spawn::spawn_context;
use crate::src::shared::spawn::{SPAWN_EMPTY, SPAWN_KILL, SPAWN_RESPAWN};
use crate::src::shared::window::{window, winlink};
use crate::src::spawn::spawn_pane;
pub static cmd_respawn_pane_entry: cmd_entry = {
    cmd_entry {
        name: c"respawn-pane",
        alias: Some(c"respawnp"),
        args: args_parse {
            template: c"c:e:Ekt:",
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: c"[-Ek] [-c start-directory] [-e environment] [-t target-pane] [shell-command [argument ...]]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_respawn_pane_exec
                as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe fn cmd_respawn_pane_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
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
        environ: None,
        idx: 0,
        cwd: ::core::ptr::null::<::core::ffi::c_char>(),
        flags: 0,
    };
    let mut argv_owner = Vec::new();
    let mut s: *mut session = (*target).s;
    let mut wl: *mut winlink = (*target).wl;
    let mut wp: *mut window_pane = (*target).wp;
    let mut cause: Option<std::ffi::CString> = None;
    sc.item = item;
    sc.s = s;
    sc.wl = wl;
    sc.wp0 = wp;
    argv_owner = args_to_vector(&*args);
    sc.argv = argv_owner;
    sc.environ = Some(environ_create());
    for av in args_flag_values(&*args, 'e' as i32 as u_char) {
        environ_put(
            sc.environ.as_deref_mut().expect("environment"),
            av.string_ptr(),
            0 as ::core::ffi::c_int,
        );
    }
    sc.idx = -(1 as ::core::ffi::c_int);
    sc.cwd = args_get(args, 'c' as i32 as u_char);
    sc.flags = SPAWN_RESPAWN;
    if args_has(args, 'E' as i32 as u_char) != 0 {
        sc.flags |= SPAWN_EMPTY;
    }
    if args_has(args, 'k' as i32 as u_char) != 0 {
        sc.flags |= SPAWN_KILL;
    }
    if spawn_pane(&raw mut sc, &raw mut cause).is_null() {
        cmdq_error(item, |out| {
            out.write_all(b"respawn pane failed: ")?;
            write_cstr(
                out,
                cause
                    .as_ref()
                    .map_or(::core::ptr::null(), |value| value.as_ptr()),
            )
        });
        drop(sc.environ.take());
        return CMD_RETURN_ERROR;
    }
    (*wp).flags |= PANE_REDRAW;
    server_redraw_window_borders((*wp).window as *mut window);
    server_status_window((*wp).window as *mut window);
    drop(sc.environ.take());
    return CMD_RETURN_NORMAL;
}
