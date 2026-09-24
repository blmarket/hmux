use crate::src::arguments::{
    args_count, args_first_value, args_get, args_has, args_next_value, args_string, args_to_vector,
};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd_find::cmd_find_from_winlink;
use crate::src::cmd_queue::{
    cmdq_error, cmdq_get_client, cmdq_get_current, cmdq_get_target, cmdq_get_target_client,
    cmdq_insert_hook, cmdq_print,
};
use crate::src::environ::{environ_create, environ_free, environ_put};
use crate::src::ffi::libc::strcmp;
use crate::src::format::format_single_cstring;
use crate::src::resize::recalculate_sizes;
use crate::src::server_fn::{
    server_redraw_session, server_redraw_session_group, server_status_session_group,
};
use crate::src::session::session_set_current;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{
    args, args_parse, args_parse_cb, args_value, args_value_entry,
};
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files, overlay_check_cb,
    overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb, overlay_resize_cb,
};
use crate::src::shared::colour::*;
pub use crate::src::shared::command::CMD_FIND_WINDOW_INDEX;
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
    window_pane, window_pane_modes, window_pane_prompt, window_pane_tree_entry, window_panes,
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
pub use crate::src::shared::spawn::{SPAWN_DETACHED, SPAWN_EMPTY, SPAWN_KILL};
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tree::RB_NEGINF;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::spawn::spawn_window;
use crate::src::tmux::{check_name, clean_name_cstring};
use crate::src::window::{
    winlink_find_by_index, winlink_shuffle_up, winlinks_minmax, winlinks_next,
};
use std::ffi::{CStr, CString};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

pub const NEW_WINDOW_TEMPLATE: [::core::ffi::c_char; 46] = unsafe {
    ::core::mem::transmute::<[u8; 46], [::core::ffi::c_char; 46]>(
        *b"#{session_name}:#{window_index}.#{pane_index}\0",
    )
};
#[no_mangle]
pub static mut cmd_new_window_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"new-window\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"neww\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"abc:de:EF:kn:PSt:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: b"[-abdEkPS] [-c start-directory] [-e environment] [-F format] [-n window-name] [-t target-window] [shell-command [argument ...]]\0"
            as *const u8 as *const ::core::ffi::c_char,
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
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_new_window_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
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
        cmdq_error(
            item,
            b"command cannot be given for empty pane\0" as *const u8 as *const ::core::ffi::c_char,
        );
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
            cmdq_error(
                item,
                b"invalid window name: %s\0" as *const u8 as *const ::core::ffi::c_char,
                expanded.as_ptr(),
            );
            return CMD_RETURN_ERROR;
        }
        wname_owned = Some(
            clean_name_cstring(CStr::from_ptr(expanded.as_ptr()), 0)
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
            wl = winlinks_minmax(&raw mut (*s).windows, RB_NEGINF);
            while !wl.is_null() {
                if !(strcmp((*(*wl).window).name, expanded.as_ptr()) != 0 as ::core::ffi::c_int) {
                    if new_wl.is_null() {
                        new_wl = wl;
                    } else {
                        cmdq_error(
                            item,
                            b"multiple windows named %s\0" as *const u8
                                as *const ::core::ffi::c_char,
                            wname,
                        );
                        return CMD_RETURN_ERROR;
                    }
                }
                wl = winlinks_next(wl);
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
    argv_owner = args_to_vector(args);
    sc.argv = argv_owner;
    sc.environ = environ_create();
    av = args_first_value(args, 'e' as i32 as u_char);
    while !av.is_null() {
        environ_put(
            sc.environ,
            (*av).string_ptr(),
            0 as ::core::ffi::c_int,
        );
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
        cmdq_error(
            item,
            b"create window failed: %s\0" as *const u8 as *const ::core::ffi::c_char,
            cause
                .as_ref()
                .map_or(::core::ptr::null(), |value| value.as_ptr()),
        );
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
            cmdq_print(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                cp.as_ptr(),
            );
        }
        cmd_find_from_winlink(&raw mut fs, new_wl, 0 as ::core::ffi::c_int);
        cmdq_insert_hook(
            s,
            item,
            &raw mut fs,
            b"after-new-window\0" as *const u8 as *const ::core::ffi::c_char,
        );
        environ_free(sc.environ);
        return CMD_RETURN_NORMAL;
    };
}
