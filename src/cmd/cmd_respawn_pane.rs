use crate::arguments::args_get_str;
use crate::arguments::{args_has, args_to_vector, args_value_list};
use crate::cmd::cmd_get_args;

use crate::cmd::queue::cmdq_item_weak_of;
use crate::environ::EnvironmentStore;
use crate::environ::new_environment_box;
use crate::fmt_args;

pub use crate::consts::{
    CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, SPAWN_KILL, SPAWN_RESPAWN,
};
use crate::spawn::spawn_pane;
pub use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval, cmdq_item};
pub use crate::types::{args, args_parse_t, spawn_context, u_char};
use ::std::ffi::CString;

pub(crate) static cmd_respawn_pane_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"respawn-pane",
        alias: Some(c"respawnp"),
        args: args_parse_t {
            template: c"c:e:kt:",
            lower: 0 as core::ffi::c_int,
            upper: -(1 as core::ffi::c_int),
            cb: None,
        },
        usage: c"[-k] [-c start-directory] [-e environment] [-t target-pane] [shell-command [argument ...]]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as core::ffi::c_int,
        },
        flags: 0 as core::ffi::c_int,
        exec: cmd_respawn_pane_exec,
    }
};
unsafe fn cmd_respawn_pane_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &args = cmd_get_args(self_0);
    let mut sc = spawn_context::default();
    let mut cause: Option<CString> = None;
    sc.item = cmdq_item_weak_of(item);
    sc.s = item.target.session();
    sc.wl_idx = item.target.wl_idx;
    sc.wp0 = item.target.pane_ref();
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
    let Some(pane) = (unsafe { spawn_pane(&mut sc, None, &mut cause) }) else {
        let cause = cause.unwrap();
        unsafe { item.error(c"respawn pane failed: %s", fmt_args![cause.as_c_str()]) };
        drop(sc.environ.take());
        return CMD_RETURN_ERROR;
    };
    unsafe { pane.request_redraw() };
    let window = pane.window().expect("the respawned pane has a window");
    unsafe { window.redraw_borders() };
    unsafe { window.redraw_status() };
    drop(sc.environ.take());
    CMD_RETURN_NORMAL
}
