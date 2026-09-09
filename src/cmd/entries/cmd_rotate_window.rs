use crate::arguments::args_has;
use crate::cmd::cmd_get_args;

pub use crate::consts::{CMD_FIND_PANE, CMD_FIND_WINDOW, CMD_RETURN_NORMAL};
pub use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
pub use crate::cmdq::cmdq_item;
pub use crate::types::args_parse_t;
use crate::window::WinlinkRef;

pub(crate) static cmd_rotate_window_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"rotate-window",
        alias: Some(c"rotatew"),
        args: args_parse_t {
            template: c"Dt:UZ",
            lower: 0 as core::ffi::c_int,
            upper: 0 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-DUZ] [-t target-window]",
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
        exec: cmd_rotate_window_exec,
    }
};
unsafe fn cmd_rotate_window_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let current_state_ref = item.state_ref();
    let link = WinlinkRef::new(
        item.target
            .session()
            .expect("a rotation target has a session"),
        item.target
            .wl_idx
            .expect("a rotation target has a window link"),
    )
    .expect("the rotation target is linked");
    let owner = link.window().expect("the rotation window is present");
    unsafe { owner.push_zoom(0, args_has(args, b'Z')) };
    let down = args_has(args, b'D') != 0;
    let panes = unsafe { owner.rotate_pane_geometry(down) };
    let count = panes.len();
    if count == 0 {
        unsafe { owner.pop_zoom() };
        return CMD_RETURN_NORMAL;
    }
    let active = panes
        .iter()
        .position(|pane| Some(pane.id()) == owner.active_pane_id());
    let selected = if down {
        active
            .and_then(|index| index.checked_sub(1))
            .unwrap_or(count - 1)
    } else {
        active.map(|index| (index + 1) % count).unwrap_or(0)
    };
    let selected_id = panes[selected].id();
    unsafe { owner.set_active_pane(&panes[selected], 1) };
    let pane = owner
        .pane_by_id(selected_id)
        .expect("rotation keeps the selected pane");
    unsafe { current_state_ref.update_current_link(&link, Some(&pane), 0) };
    unsafe { owner.pop_zoom() };
    unsafe { owner.redraw() };
    CMD_RETURN_NORMAL
}
