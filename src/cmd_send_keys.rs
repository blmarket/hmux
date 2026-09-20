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
pub use crate::src::shared::limits::{__INT_MAX__, UINT_MAX};
pub use crate::src::shared::pane::{
    PANE_REDRAW, PANE_STYLECHANGED, PANE_THEMECHANGED, window_pane_offset, window_pane_resize,
    window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::command::{
    CMD_AFTERHOOK, CMD_CLIENT_CANFAIL, CMD_CLIENT_CFLAG, CMD_READONLY,
};
pub use crate::src::shared::client::{CLIENT_READONLY};
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
use crate::src::shared::utf8::*;
extern "C" {

    fn strtol(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_count(_: *mut args) -> u_int;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn args_strtonum_and_expand(
        _: *mut args,
        _: u_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut cmdq_item,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn cmd_get_entry(_: *mut cmd) -> *const cmd_entry;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmd_mouse_pane(
        _: *mut mouse_event,
        _: *mut *mut session,
        _: *mut *mut winlink,
    ) -> *mut window_pane;
    fn cmdq_get_target_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_get_event(_: *mut cmdq_item) -> *mut key_event;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn key_bindings_get_table(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> *mut key_table;
    fn key_bindings_unref_table(_: *mut key_table);
    fn key_bindings_get(_: *mut key_table, _: key_code) -> *mut key_binding;
    fn key_bindings_dispatch(
        _: *mut key_binding,
        _: *mut cmdq_item,
        _: *mut client,
        _: *mut key_event,
        _: *mut cmd_find_state,
    ) -> *mut cmdq_item;
    fn key_string_lookup_string(_: *const ::core::ffi::c_char) -> key_code;
    fn server_client_handle_key(_: *mut client, _: *mut key_event) -> ::core::ffi::c_int;
    fn server_client_handle_key_after(
        _: *mut client,
        _: *mut key_event,
        _: *mut cmdq_item,
        _: *mut *mut cmdq_item,
    ) -> ::core::ffi::c_int;
    fn input_reset(_: *mut input_ctx, _: ::core::ffi::c_int);
    fn colour_palette_clear(_: *mut colour_palette);
    fn window_pane_key(
        _: *mut window_pane,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: key_code,
        _: *mut mouse_event,
    ) -> ::core::ffi::c_int;
    fn utf8_from_data(_: *const utf8_data, _: *mut utf8_char) -> utf8_state;
    fn utf8_fromcstr(_: *const ::core::ffi::c_char) -> *mut utf8_data;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_12 {
    pub offset: u_int,
    pub data: C2RustUnnamed_13,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_13 {
    pub attr: u_char,
    pub fg: u_char,
    pub bg: u_char,
    pub data: u_char,
}

pub type C2RustUnnamed_35 = ::core::ffi::c_ulong;

#[no_mangle]
pub static mut cmd_send_keys_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"send-keys\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"send\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"c:FHKlMN:Rt:X\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: b"[-FHKlMRX] [-c target-client] [-N repeat-count] [-t target-pane] [key ...]\0"
            as *const u8 as *const ::core::ffi::c_char,
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
        flags: CMD_AFTERHOOK | CMD_CLIENT_CFLAG | CMD_CLIENT_CANFAIL | CMD_READONLY,
        exec: Some(
            cmd_send_keys_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_send_prefix_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"send-prefix\0" as *const u8 as *const ::core::ffi::c_char,
        alias: ::core::ptr::null::<::core::ffi::c_char>(),
        args: args_parse {
            template: b"2t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-2] [-t target-pane]\0" as *const u8 as *const ::core::ffi::c_char,
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
            cmd_send_keys_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_send_keys_inject_key(
    mut item: *mut cmdq_item,
    mut after: *mut cmdq_item,
    mut args: *mut args,
    mut key: key_code,
) -> *mut cmdq_item {
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut s: *mut session = (*target).s;
    let mut wl: *mut winlink = (*target).wl;
    let mut wp: *mut window_pane = (*target).wp;
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut event: *mut key_event = ::core::ptr::null_mut::<key_event>();
    let mut new_after: *mut cmdq_item = after;
    if args_has(args, 'K' as i32 as u_char) != 0 {
        if tc.is_null() {
            return item;
        }
        event =
            xcalloc(1 as size_t, ::core::mem::size_of::<key_event>() as size_t) as *mut key_event;
        (*event).key = (key as ::core::ffi::c_ulonglong | KEYC_SENT) as key_code;
        memset(
            &raw mut (*event).m as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<mouse_event>() as size_t,
        );
        if after.is_null() {
            if server_client_handle_key(tc, event) != 0 as ::core::ffi::c_int {
                return item;
            }
        } else if server_client_handle_key_after(tc, event, after, &raw mut new_after)
            != 0 as ::core::ffi::c_int
        {
            return new_after;
        }
        free((*event).buf as *mut ::core::ffi::c_void);
        free(event as *mut ::core::ffi::c_void);
        return item;
    }
    wme = (*wp).modes.tqh_first;
    if wme.is_null() || (*(*wme).mode).key_table.is_none() {
        if window_pane_key(wp, tc, s, wl, key, ::core::ptr::null_mut::<mouse_event>())
            != 0 as ::core::ffi::c_int
        {
            return ::core::ptr::null_mut::<cmdq_item>();
        }
        return item;
    }
    table = key_bindings_get_table(
        (*(*wme).mode).key_table.expect("non-null function pointer")(wme),
        1 as ::core::ffi::c_int,
    );
    bd = key_bindings_get(table, key & !KEYC_MASK_FLAGS);
    if !bd.is_null() {
        (*table).references = (*table).references.wrapping_add(1);
        after = key_bindings_dispatch(bd, after, tc, ::core::ptr::null_mut::<key_event>(), target);
        key_bindings_unref_table(table);
    }
    return after;
}
unsafe extern "C" fn cmd_send_keys_inject_string(
    mut item: *mut cmdq_item,
    mut after: *mut cmdq_item,
    mut args: *mut args,
    mut i: ::core::ffi::c_int,
) -> *mut cmdq_item {
    let mut s: *const ::core::ffi::c_char = args_string(args, i as u_int);
    let mut ud: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    let mut loop_0: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    let mut uc: utf8_char = 0;
    let mut key: key_code = 0;
    let mut endptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut n: ::core::ffi::c_long = 0;
    let mut literal: ::core::ffi::c_int = 0;
    if args_has(args, 'H' as i32 as u_char) != 0 {
        n = strtol(s, &raw mut endptr, 16 as ::core::ffi::c_int);
        if *s as ::core::ffi::c_int == '\0' as i32
            || n < 0 as ::core::ffi::c_long
            || n > 0xff as ::core::ffi::c_long
            || *endptr as ::core::ffi::c_int != '\0' as i32
        {
            return item;
        }
        return cmd_send_keys_inject_key(item, after, args, KEYC_LITERAL | n as key_code);
    }
    literal = args_has(args, 'l' as i32 as u_char);
    if literal == 0 {
        key = key_string_lookup_string(s);
        if key != KEYC_NONE as ::core::ffi::c_ulong as key_code
            && key != KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code
        {
            after = cmd_send_keys_inject_key(item, after, args, key);
            if !after.is_null() {
                return after;
            }
        }
        literal = 1 as ::core::ffi::c_int;
    }
    if literal != 0 {
        ud = utf8_fromcstr(s);
        let mut current_block_20: u64;
        loop_0 = ud;
        while (*loop_0).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            if (*loop_0).size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                && (*loop_0).data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    <= 0x7f as ::core::ffi::c_int
            {
                key = (*loop_0).data[0 as ::core::ffi::c_int as usize] as key_code;
                current_block_20 = 12147880666119273379;
            } else if utf8_from_data(loop_0, &raw mut uc) as ::core::ffi::c_uint
                != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                current_block_20 = 1054647088692577877;
            } else {
                key = uc as key_code;
                current_block_20 = 12147880666119273379;
            }
            match current_block_20 {
                12147880666119273379 => {
                    after = cmd_send_keys_inject_key(item, after, args, key);
                }
                _ => {}
            }
            loop_0 = loop_0.offset(1);
        }
        free(ud as *mut ::core::ffi::c_void);
    }
    return after;
}
unsafe extern "C" fn cmd_send_keys_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut s: *mut session = (*target).s;
    let mut wl: *mut winlink = (*target).wl;
    let mut wp: *mut window_pane = (*target).wp;
    let mut event: *mut key_event = cmdq_get_event(item);
    let mut m: *mut mouse_event = &raw mut (*event).m;
    let mut wme: *mut window_mode_entry = (*wp).modes.tqh_first;
    let mut after: *mut cmdq_item = item;
    let mut key: key_code = 0;
    let mut i: u_int = 0;
    let mut np: u_int = 1 as u_int;
    let mut count: u_int = args_count(args);
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !tc.is_null()
        && (*tc).flags & CLIENT_READONLY as uint64_t != 0
        && args_has(args, 'X' as i32 as u_char) == 0
    {
        cmdq_error(
            item,
            b"client is read-only\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'N' as i32 as u_char) != 0 {
        np = args_strtonum_and_expand(
            args,
            'N' as i32 as u_char,
            1 as ::core::ffi::c_longlong,
            UINT_MAX as ::core::ffi::c_longlong,
            item,
            &raw mut cause,
        ) as u_int;
        if !cause.is_null() {
            cmdq_error(
                item,
                b"repeat count %s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
            free(cause as *mut ::core::ffi::c_void);
            return CMD_RETURN_ERROR;
        }
        if !wme.is_null() && (args_has(args, 'X' as i32 as u_char) != 0 || count == 0 as u_int) {
            if (*(*wme).mode).command.is_none() {
                cmdq_error(
                    item,
                    b"not in a mode\0" as *const u8 as *const ::core::ffi::c_char,
                );
                return CMD_RETURN_ERROR;
            }
            (*wme).prefix = np;
        }
    }
    if args_has(args, 'X' as i32 as u_char) != 0 {
        if wme.is_null() || (*(*wme).mode).command.is_none() {
            cmdq_error(
                item,
                b"not in a mode\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
        if (*m).valid == 0 {
            m = ::core::ptr::null_mut::<mouse_event>();
        }
        (*(*wme).mode).command.expect("non-null function pointer")(wme, tc, s, wl, args, m);
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'M' as i32 as u_char) != 0 {
        wp = cmd_mouse_pane(m, &raw mut s, ::core::ptr::null_mut::<*mut winlink>());
        if wp.is_null() {
            cmdq_error(
                item,
                b"no mouse target\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
        window_pane_key(wp, tc, s, wl, (*m).key, m);
        return CMD_RETURN_NORMAL;
    }
    if cmd_get_entry(self_0) == &raw const cmd_send_prefix_entry {
        if args_has(args, '2' as i32 as u_char) != 0 {
            key = options_get_number(
                (*s).options,
                b"prefix2\0" as *const u8 as *const ::core::ffi::c_char,
            ) as key_code;
        } else {
            key = options_get_number(
                (*s).options,
                b"prefix\0" as *const u8 as *const ::core::ffi::c_char,
            ) as key_code;
        }
        cmd_send_keys_inject_key(item, item, args, key);
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'R' as i32 as u_char) != 0 {
        colour_palette_clear(&raw mut (*wp).palette);
        input_reset((*wp).ictx, 1 as ::core::ffi::c_int);
        (*wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED | PANE_REDRAW;
    }
    if count == 0 as u_int {
        if args_has(args, 'N' as i32 as u_char) != 0 || args_has(args, 'R' as i32 as u_char) != 0 {
            return CMD_RETURN_NORMAL;
        }
        after = if args_has(args, 'K' as i32 as u_char) != 0 {
            item
        } else {
            ::core::ptr::null_mut::<cmdq_item>()
        };
        while np != 0 as u_int {
            after = cmd_send_keys_inject_key(item, after, args, (*event).key);
            np = np.wrapping_sub(1);
        }
        return CMD_RETURN_NORMAL;
    }
    while np != 0 as u_int {
        i = 0 as u_int;
        while i < count {
            after = cmd_send_keys_inject_string(item, after, args, i as ::core::ffi::c_int);
            i = i.wrapping_add(1);
        }
        np = np.wrapping_sub(1);
    }
    return CMD_RETURN_NORMAL;
}
