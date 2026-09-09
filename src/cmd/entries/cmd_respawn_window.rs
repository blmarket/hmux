use crate::args::args_get_str;
use crate::args::{args_has, args_to_vector, args_value_list};
use crate::cmd::cmd_get_args;
use crate::cmd::cmdq_item_weak_of;

use crate::environ::EnvironmentStore;
use crate::environ::new_environment_box;
use crate::fmt_args;

pub use crate::consts::{
    CMD_FIND_PANE, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, SPAWN_KILL, SPAWN_RESPAWN,
};
use crate::spawn::spawn_window;
pub use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
pub use crate::cmd::cmdq_item;
pub use crate::types::{ClientRef, args, args_parse_t, spawn_context, u_char};
use ::std::ffi::CString;

pub(crate) static cmd_respawn_window_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"respawn-window",
        alias: Some(c"respawnw"),
        args: args_parse_t {
            template: c"c:e:kt:",
            lower: 0 as core::ffi::c_int,
            upper: -(1 as core::ffi::c_int),
            cb: None,
        },
        usage: c"[-k] [-c start-directory] [-e environment] [-t target-window] [shell-command [argument ...]]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as core::ffi::c_int,
        },
        flags: 0 as core::ffi::c_int,
        exec: cmd_respawn_window_exec,
    }
};
unsafe fn cmd_respawn_window_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &args = cmd_get_args(self_0);
    let mut sc = spawn_context::default();
    let target_client = item.target_client();
    let mut cause: Option<CString> = None;
    sc.item = cmdq_item_weak_of(item);
    sc.s = item.target.session();
    sc.wl_idx = item.target.wl_idx;
    sc.tc = target_client.as_ref().map(ClientRef::downgrade);
    unsafe { sc.argv = args_to_vector(args) };
    sc.environ = Some(new_environment_box());
    for av in args_value_list(args, 'e' as i32 as u_char) {
        sc.environ
            .as_deref_mut()
            .expect("the respawn environment is initialized")
            .put(av.value.string(), 0);
    }
    sc.idx = -(1 as core::ffi::c_int);
    sc.cwd = args_get_str(args, 'c' as i32 as u_char);
    sc.flags = SPAWN_RESPAWN;
    if args_has(args, 'k' as i32 as u_char) != 0 {
        sc.flags |= SPAWN_KILL;
    }
    let Some(link) = (unsafe { spawn_window(&mut sc, &mut cause) }) else {
        let cause = cause.unwrap();
        unsafe { item.error(c"respawn window failed: %s", fmt_args![cause.as_c_str()]) };
        drop(sc.environ.take());
        return CMD_RETURN_ERROR;
    };
    if let Some(window) = link.window() {
        unsafe { window.redraw() };
    }
    drop(sc.environ.take());
    CMD_RETURN_NORMAL
}
