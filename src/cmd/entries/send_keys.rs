use crate::src::arguments::{args_count, args_has, args_string, args_strtonum_and_expand_result};
use crate::src::cmd::queue::{cmdq_error, cmdq_get_event, cmdq_get_target, cmdq_get_target_client};
use crate::src::cmd::{cmd_get_args, cmd_get_entry, cmd_mouse_pane};
use crate::src::ffi::libc::strtol;
use crate::src::format::bytes::write_cstr;
use crate::src::input::input_reset;
use crate::src::key_bindings::{
    key_bindings_dispatch, key_bindings_get, key_bindings_get_table, key_bindings_unref_table,
};
use crate::src::key_string::key_string_parse_cstr;
use crate::src::options::options_get_number;
use crate::src::server_client::{server_client_handle_key, server_client_handle_key_after};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::client::CLIENT_READONLY;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::command::{
    CMD_AFTERHOOK, CMD_CLIENT_CANFAIL, CMD_CLIENT_CFLAG, CMD_READONLY,
};
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::key::{key_binding, key_event, key_table};
use crate::src::shared::limits::UINT_MAX;
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::{PANE_REDRAW, PANE_STYLECHANGED, PANE_THEMECHANGED};
use crate::src::shared::session::session;
use crate::src::shared::utf8::*;
use crate::src::shared::window::{window_mode_entry, winlink};
use crate::src::style::colour::colour_palette_clear;
use crate::src::text::utf8::{utf8_from_data, utf8_fromcstr_vec};
use crate::src::window::window_pane_key;
pub static mut cmd_send_keys_entry: cmd_entry = {
    cmd_entry {
        name: c"send-keys",
        alias: Some(c"send"),
        args: args_parse {
            template: b"c:FHKlMN:Rt:X\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: c"[-FHKlMRX] [-c target-client] [-N repeat-count] [-t target-pane] [key ...]",
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
        exec: Some(cmd_send_keys_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
pub static mut cmd_send_prefix_entry: cmd_entry = {
    cmd_entry {
        name: c"send-prefix",
        alias: None,
        args: args_parse {
            template: b"2t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-2] [-t target-pane]",
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
        exec: Some(cmd_send_keys_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_send_keys_inject_key(
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
    let mut new_after: *mut cmdq_item = after;
    if args_has(args, 'K' as i32 as u_char) != 0 {
        if tc.is_null() {
            return item;
        }
        let event = key_event::new(
            (key as ::core::ffi::c_ulonglong | KEYC_SENT) as key_code,
            Default::default(),
            None,
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
        return item;
    }
    wme = (*wp).modes.active;
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
        crate::src::shared::rc::retain(table);
        after = key_bindings_dispatch(bd, after, tc, ::core::ptr::null_mut::<key_event>(), target);
        key_bindings_unref_table(table);
    }
    return after;
}
unsafe fn cmd_send_keys_inject_string(
    mut item: *mut cmdq_item,
    mut after: *mut cmdq_item,
    mut args: *mut args,
    mut i: ::core::ffi::c_int,
) -> *mut cmdq_item {
    let mut s: *const ::core::ffi::c_char = args_string(args, i as u_int);
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
        key = key_string_parse_cstr(std::ffi::CStr::from_ptr(s)).unwrap_or(KEYC_UNKNOWN);
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
        let cells = utf8_fromcstr_vec(std::ffi::CStr::from_ptr(s));
        let mut current_block_20: u64;
        for cell in cells.iter().take_while(|cell| cell.size != 0) {
            if cell.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                && cell.data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    <= 0x7f as ::core::ffi::c_int
            {
                key = cell.data[0 as ::core::ffi::c_int as usize] as key_code;
                current_block_20 = 12147880666119273379;
            } else if utf8_from_data(cell, &raw mut uc) as ::core::ffi::c_uint
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
        }
    }
    return after;
}
unsafe fn cmd_send_keys_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut s: *mut session = (*target).s;
    let mut wl: *mut winlink = (*target).wl;
    let mut wp: *mut window_pane = (*target).wp;
    let mut event: *mut key_event = cmdq_get_event(item);
    let mut m: *mut mouse_event = &raw mut (*event).m;
    let mut wme: *mut window_mode_entry = (*wp).modes.active;
    let mut after: *mut cmdq_item = item;
    let mut key: key_code = 0;
    let mut i: u_int = 0;
    let mut np: u_int = 1 as u_int;
    let mut count: u_int = args_count(args);
    if !tc.is_null()
        && (*tc).flags & CLIENT_READONLY as uint64_t != 0
        && args_has(args, 'X' as i32 as u_char) == 0
    {
        cmdq_error(item, |out| out.write_all(b"client is read-only"));
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'N' as i32 as u_char) != 0 {
        np = match args_strtonum_and_expand_result(
            args,
            'N' as i32 as u_char,
            1 as ::core::ffi::c_longlong,
            UINT_MAX as ::core::ffi::c_longlong,
            item,
        ) {
            Ok(value) => value as u_int,
            Err(error) => {
                cmdq_error(item, |out| {
                    out.write_all(b"repeat count ")?;
                    write_cstr(out, error.message().as_ptr())
                });
                return CMD_RETURN_ERROR;
            }
        };
        if !wme.is_null() && (args_has(args, 'X' as i32 as u_char) != 0 || count == 0 as u_int) {
            if (*(*wme).mode).command.is_none() {
                cmdq_error(item, |out| out.write_all(b"not in a mode"));
                return CMD_RETURN_ERROR;
            }
            (*wme).prefix = np;
        }
    }
    if args_has(args, 'X' as i32 as u_char) != 0 {
        if wme.is_null() || (*(*wme).mode).command.is_none() {
            cmdq_error(item, |out| out.write_all(b"not in a mode"));
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
            cmdq_error(item, |out| out.write_all(b"no mouse target"));
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
