use crate::src::arguments::{
    args_first_value, args_get, args_has, args_next_value, args_to_vector,
};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd_queue::{cmdq_error, cmdq_get_target, cmdq_get_target_client};
use crate::src::environ::{environ_create, environ_free, environ_put};
use crate::src::server_fn::server_redraw_window;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{
    args, args_parse, args_parse_cb, args_value, args_value_c2rust_unnamed, args_value_entry,
};
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files, overlay_check_cb,
    overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb, overlay_resize_cb,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
pub use crate::src::shared::format::{format_job_tree, format_tree};
use crate::src::shared::grid::*;
pub use crate::src::shared::hyperlinks::hyperlinks;
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
use crate::src::shared::key::*;
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::options;
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::spawn_context;
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::spawn::{SPAWN_EMPTY, SPAWN_KILL, SPAWN_RESPAWN};
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::spawn::spawn_window;

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[no_mangle]
pub static mut cmd_respawn_window_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"respawn-window\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"respawnw\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"c:e:Ekt:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: b"[-Ek] [-c start-directory] [-e environment] [-t target-window] [shell-command [argument ...]]\0"
            as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_respawn_window_exec
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_respawn_window_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
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
        environ: ::core::ptr::null_mut::<environ>(),
        idx: 0,
        cwd: ::core::ptr::null::<::core::ffi::c_char>(),
        flags: 0,
    };
    let mut argv_owner = Vec::new();
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut s: *mut session = (*target).s;
    let mut wl: *mut winlink = (*target).wl;
    let mut cause: Option<std::ffi::CString> = None;
    let mut av: *mut args_value = ::core::ptr::null_mut::<args_value>();
    sc.item = item;
    sc.s = s;
    sc.wl = wl;
    sc.tc = tc;
    argv_owner = args_to_vector(args);
    sc.argv = argv_owner;
    sc.environ = environ_create();
    av = args_first_value(args, 'e' as i32 as u_char);
    while !av.is_null() {
        environ_put(
            sc.environ,
            (*av).c2rust_unnamed.string,
            0 as ::core::ffi::c_int,
        );
        av = args_next_value(av);
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
    if spawn_window(&raw mut sc, &raw mut cause).is_null() {
        cmdq_error(
            item,
            b"respawn window failed: %s\0" as *const u8 as *const ::core::ffi::c_char,
            cause
                .as_ref()
                .map_or(::core::ptr::null(), |value| value.as_ptr()),
        );
        environ_free(sc.environ);
        return CMD_RETURN_ERROR;
    }
    server_redraw_window((*wl).window);
    environ_free(sc.environ);
    return CMD_RETURN_NORMAL;
}
