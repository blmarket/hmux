use crate::src::arguments::{args_create, args_free, args_has, args_set, args_string};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd_queue::cmdq_get_target;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{
    args, args_parse, args_parse_cb, args_value, args_value_c2rust_unnamed, args_value_entry,
};
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
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
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::window::window_pane_set_mode;
use crate::src::window_tree::window_tree_mode;
use crate::src::xmalloc::xasprintf;

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[no_mangle]
pub static mut cmd_find_window_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"find-window\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"findw\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"CiNrt:TZ\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-CiNrTZ] [-t target-pane] match-string\0" as *const u8
            as *const ::core::ffi::c_char,
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
            cmd_find_window_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_find_window_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut new_args: *mut args = ::core::ptr::null_mut::<args>();
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut wp: *mut window_pane = (*target).wp;
    let mut s: *const ::core::ffi::c_char = args_string(args, 0 as u_int);
    let mut suffix: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut star: *const ::core::ffi::c_char = b"*\0" as *const u8 as *const ::core::ffi::c_char;
    let mut filter: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut C: ::core::ffi::c_int = 0;
    let mut N: ::core::ffi::c_int = 0;
    let mut T: ::core::ffi::c_int = 0;
    C = args_has(args, 'C' as i32 as u_char);
    N = args_has(args, 'N' as i32 as u_char);
    T = args_has(args, 'T' as i32 as u_char);
    if args_has(args, 'r' as i32 as u_char) != 0 {
        star = b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if args_has(args, 'r' as i32 as u_char) != 0 && args_has(args, 'i' as i32 as u_char) != 0 {
        suffix = b"/ri\0" as *const u8 as *const ::core::ffi::c_char;
    } else if args_has(args, 'r' as i32 as u_char) != 0 {
        suffix = b"/r\0" as *const u8 as *const ::core::ffi::c_char;
    } else if args_has(args, 'i' as i32 as u_char) != 0 {
        suffix = b"/i\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if C == 0 && N == 0 && T == 0 {
        T = 1 as ::core::ffi::c_int;
        N = T;
        C = N;
    }
    filter = crate::src::arguments::args_new_flag_value();
    (*filter).type_0 = ARGS_STRING;
    if C != 0 && N != 0 && T != 0 {
        xasprintf(
            &raw mut (*filter).c2rust_unnamed.string,
            b"#{||:#{C%s:%s},#{||:#{m%s:%s%s%s,#{window_name}},#{m%s:%s%s%s,#{pane_title}}}}\0"
                as *const u8 as *const ::core::ffi::c_char,
            suffix,
            s,
            suffix,
            star,
            s,
            star,
            suffix,
            star,
            s,
            star,
        );
    } else if C != 0 && N != 0 {
        xasprintf(
            &raw mut (*filter).c2rust_unnamed.string,
            b"#{||:#{C%s:%s},#{m%s:%s%s%s,#{window_name}}}\0" as *const u8
                as *const ::core::ffi::c_char,
            suffix,
            s,
            suffix,
            star,
            s,
            star,
        );
    } else if C != 0 && T != 0 {
        xasprintf(
            &raw mut (*filter).c2rust_unnamed.string,
            b"#{||:#{C%s:%s},#{m%s:%s%s%s,#{pane_title}}}\0" as *const u8
                as *const ::core::ffi::c_char,
            suffix,
            s,
            suffix,
            star,
            s,
            star,
        );
    } else if N != 0 && T != 0 {
        xasprintf(
            &raw mut (*filter).c2rust_unnamed.string,
            b"#{||:#{m%s:%s%s%s,#{window_name}},#{m%s:%s%s%s,#{pane_title}}}\0" as *const u8
                as *const ::core::ffi::c_char,
            suffix,
            star,
            s,
            star,
            suffix,
            star,
            s,
            star,
        );
    } else if C != 0 {
        xasprintf(
            &raw mut (*filter).c2rust_unnamed.string,
            b"#{C%s:%s}\0" as *const u8 as *const ::core::ffi::c_char,
            suffix,
            s,
        );
    } else if N != 0 {
        xasprintf(
            &raw mut (*filter).c2rust_unnamed.string,
            b"#{m%s:%s%s%s,#{window_name}}\0" as *const u8 as *const ::core::ffi::c_char,
            suffix,
            star,
            s,
            star,
        );
    } else {
        xasprintf(
            &raw mut (*filter).c2rust_unnamed.string,
            b"#{m%s:%s%s%s,#{pane_title}}\0" as *const u8 as *const ::core::ffi::c_char,
            suffix,
            star,
            s,
            star,
        );
    }
    new_args = args_create();
    if args_has(args, 'Z' as i32 as u_char) != 0 {
        args_set(
            new_args,
            'Z' as i32 as u_char,
            ::core::ptr::null_mut::<args_value>(),
            0 as ::core::ffi::c_int,
        );
    }
    args_set(
        new_args,
        'f' as i32 as u_char,
        filter,
        0 as ::core::ffi::c_int,
    );
    window_pane_set_mode(
        wp,
        ::core::ptr::null_mut::<window_pane>(),
        &raw const window_tree_mode,
        item,
        target,
        new_args,
    );
    args_free(new_args);
    return CMD_RETURN_NORMAL;
}
