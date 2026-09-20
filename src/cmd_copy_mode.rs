pub use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::{menu_data};
pub use crate::src::shared::options::{options};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::process::{tmuxpeer};
pub use crate::src::shared::prompt::{prompt};
pub use crate::src::shared::redraw::{redraw_scene};
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::{spawn_editor_state};
pub use crate::src::shared::status::{status_line};
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::shared::environment::{environ};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::command::{CMD_AFTERHOOK, CMD_READONLY, CMD_TARGET_PANE_USAGE};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::arguments::*;
use crate::src::shared::terminal::*;
use crate::src::shared::event::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;
extern "C" {

    fn tty_window_offset(
        _: *mut tty,
        _: *mut u_int,
        _: *mut u_int,
        _: *mut u_int,
        _: *mut u_int,
    ) -> ::core::ffi::c_int;
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn cmd_get_entry(_: *mut cmd) -> *const cmd_entry;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmd_mouse_pane(
        _: *mut mouse_event,
        _: *mut *mut session,
        _: *mut *mut winlink,
    ) -> *mut window_pane;
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_get_source(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_get_event(_: *mut cmdq_item) -> *mut key_event;
    fn window_pane_set_mode(
        _: *mut window_pane,
        _: *mut window_pane,
        _: *const window_mode,
        _: *mut cmdq_item,
        _: *mut cmd_find_state,
        _: *mut args,
    ) -> ::core::ffi::c_int;
    fn window_pane_reset_mode_all(_: *mut window_pane);
    static window_clock_mode: window_mode;
    static window_copy_mode: window_mode;
    fn window_copy_scroll(
        _: *mut window_pane,
        _: ::core::ffi::c_int,
        _: u_int,
        _: u_int,
        _: ::core::ffi::c_int,
    );
    fn window_copy_pageup(_: *mut window_pane, _: ::core::ffi::c_int);
    fn window_copy_pagedown(_: *mut window_pane, _: ::core::ffi::c_int, _: ::core::ffi::c_int);
    fn window_copy_start_drag(_: *mut client, _: *mut mouse_event);
    fn window_copy_set_line_numbers(_: *mut window_pane, _: ::core::ffi::c_int);
}

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

pub use crate::src::shared::key::key_code_enum as C2RustUnnamed_35;

#[no_mangle]
pub static mut cmd_copy_mode_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"copy-mode\0" as *const u8 as *const ::core::ffi::c_char,
        alias: ::core::ptr::null::<::core::ffi::c_char>(),
        args: args_parse {
            template: b"dekHMqSs:t:u\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-dekHMqSu] [-s src-pane] [-t target-pane]\0" as *const u8
            as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 's' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK | CMD_READONLY,
        exec: Some(
            cmd_copy_mode_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_clock_mode_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"clock-mode\0" as *const u8 as *const ::core::ffi::c_char,
        alias: ::core::ptr::null::<::core::ffi::c_char>(),
        args: args_parse {
            template: b"t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_PANE_USAGE.as_ptr(),
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
        flags: CMD_AFTERHOOK,
        exec: Some(
            cmd_copy_mode_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_copy_mode_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut event: *mut key_event = cmdq_get_event(item);
    let mut source: *mut cmd_find_state = cmdq_get_source(item);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut c: *mut client = cmdq_get_client(item);
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wp: *mut window_pane = (*target).wp;
    let mut swp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut tty_ox: u_int = 0;
    let mut tty_oy: u_int = 0;
    let mut tty_sx: u_int = 0;
    let mut tty_sy: u_int = 0;
    let mut line_numbers: ::core::ffi::c_int = 0;
    if args_has(args, 'q' as i32 as u_char) != 0 {
        window_pane_reset_mode_all(wp);
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'M' as i32 as u_char) != 0 {
        wp = cmd_mouse_pane(
            &raw mut (*event).m,
            &raw mut s,
            ::core::ptr::null_mut::<*mut winlink>(),
        );
        if wp.is_null() {
            return CMD_RETURN_NORMAL;
        }
        if c.is_null() || (*c).session != s {
            return CMD_RETURN_NORMAL;
        }
    }
    if cmd_get_entry(self_0) == &raw const cmd_clock_mode_entry {
        window_pane_set_mode(
            wp,
            ::core::ptr::null_mut::<window_pane>(),
            &raw const window_clock_mode,
            item,
            ::core::ptr::null_mut::<cmd_find_state>(),
            ::core::ptr::null_mut::<args>(),
        );
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 's' as i32 as u_char) != 0 {
        swp = (*source).wp;
    } else {
        swp = wp;
    }
    line_numbers = 1 as ::core::ffi::c_int;
    if !event.is_null()
        && ((*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
            == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
            || (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                    <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                        << 32 as ::core::ffi::c_int)
    {
        line_numbers = 0 as ::core::ffi::c_int;
    }
    if window_pane_set_mode(
        wp,
        swp,
        &raw const window_copy_mode,
        item,
        ::core::ptr::null_mut::<cmd_find_state>(),
        args,
    ) == 0
    {
        window_copy_set_line_numbers(wp, line_numbers);
        if args_has(args, 'M' as i32 as u_char) != 0 {
            window_copy_start_drag(c, &raw mut (*event).m);
        }
    } else {
        window_copy_set_line_numbers(wp, line_numbers);
    }
    if args_has(args, 'u' as i32 as u_char) != 0 {
        window_copy_pageup(wp, 0 as ::core::ffi::c_int);
    }
    if args_has(args, 'd' as i32 as u_char) != 0 {
        window_copy_pagedown(
            wp,
            0 as ::core::ffi::c_int,
            args_has(args, 'e' as i32 as u_char),
        );
    }
    if args_has(args, 'S' as i32 as u_char) != 0 {
        tty_window_offset(
            &raw mut (*c).tty,
            &raw mut tty_ox,
            &raw mut tty_oy,
            &raw mut tty_sx,
            &raw mut tty_sy,
        );
        window_copy_scroll(
            wp,
            (*c).tty.mouse_slider_mpos,
            (*event).m.y,
            tty_oy,
            args_has(args, 'e' as i32 as u_char),
        );
        return CMD_RETURN_NORMAL;
    }
    return CMD_RETURN_NORMAL;
}
