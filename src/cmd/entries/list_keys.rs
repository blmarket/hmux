use crate::src::arguments::{args_get, args_has, args_string};
use crate::src::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_target_client, cmdq_print};
use crate::src::cmd::{cmd_get_args, cmd_list_print_cstring};
use crate::src::format::bytes::write_cstr;
use crate::src::format::{
    format_add, format_create, format_defaults, format_expand_cstring, format_free,
};
use crate::src::key_bindings::{key_bindings_get_table, key_bindings_has_repeat, key_bindings_tables};
use crate::src::key_string::{key_string_format, key_string_parse_cstr};
use crate::src::options::options_get_number;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::client::CLIENT_CONTROL;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmdq_item};
use crate::src::shared::command::{
    CMD_AFTERHOOK, CMD_LIST_PRINT_ESCAPED, CMD_LIST_PRINT_NO_GROUPS, CMD_STARTSERVER,
};
use crate::src::shared::format::format_tree;
use crate::src::shared::format::FORMAT_NONE;
use crate::src::shared::key::KEY_BINDING_REPEAT;
use crate::src::shared::key::*;
use crate::src::shared::key::{key_binding, key_table};
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::sort::*;
use crate::src::shared::window::winlink;
use crate::src::sort::{
    sort_get_key_bindings, sort_get_key_bindings_table, sort_order_from_string,
};
use crate::src::status::status_message_set;
use crate::src::text::utf8::utf8_cstrwidth;
use crate::src::tmux::global_s_options;
use std::ffi::{CStr, CString};

pub const LIST_KEYS_TEMPLATE: [::core::ffi::c_char; 250] = unsafe {
    ::core::mem::transmute::<
        [u8; 250],
        [::core::ffi::c_char; 250],
    >(
        *b"#{?notes_only,#{key_prefix} #{p|#{key_string_width}:key_string} #{?key_note,#{key_note},#{key_command}},bind-key #{?key_has_repeat,#{?key_repeat,-r,  },} -T #{p|#{key_table_width}:key_table} #{p|#{key_string_width}:#{q|a:key_string}} #{key_command}}\0",
    )
};
pub static mut cmd_list_keys_entry: cmd_entry = {
    cmd_entry {
        name: c"list-keys",
        alias: Some(c"lsk"),
        args: args_parse {
            template: c"1aF:NO:P:rT:",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-1aNr] [-F format] [-O order] [-P prefix-string][-T key-table] [key]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        flags: CMD_STARTSERVER | CMD_AFTERHOOK,
        exec: Some(cmd_list_keys_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_list_keys_get_prefix(args: *mut args) -> CString {
    let mut prefix: key_code = 0;
    if args_has(args, 'P' as i32 as u_char) != 0 {
        return CStr::from_ptr(args_get(args, 'P' as i32 as u_char)).to_owned();
    }
    prefix = options_get_number(
        global_s_options,
        b"prefix\0" as *const u8 as *const ::core::ffi::c_char,
    ) as key_code;
    if prefix == KEYC_NONE as ::core::ffi::c_ulong as key_code {
        return CString::default();
    }
    key_string_format(prefix, false)
}
unsafe fn cmd_list_keys_get_width(bindings: &[&key_binding]) -> u_int {
    bindings
        .iter()
        .map(|&bd| {
            let key_string = key_string_format(bd.key, false);
            utf8_cstrwidth(&key_string)
        })
        .max()
        .unwrap_or(0)
}
unsafe fn cmd_list_keys_get_table_width(bindings: &[&key_binding]) -> u_int {
    bindings
        .iter()
        .map(|&bd| bd.tablename.as_ref().map_or(0, |s| utf8_cstrwidth(s)))
        .max()
        .unwrap_or(0)
}
unsafe fn cmd_list_keys_get_root_and_prefix<'a>(
    tables: &'a [std::rc::Rc<std::cell::UnsafeCell<key_table>>],
    sort_crit: &sort_criteria,
) -> Vec<&'a key_binding> {
    let mut bindings = Vec::new();
    for name in [c"prefix", c"root"] {
        let table = tables
            .iter()
            .map(|table| &*crate::src::shared::rc::as_ptr(table))
            .find(|table| table.name.as_c_str() == name);
        bindings.extend(sort_get_key_bindings_table(table, sort_crit));
    }
    bindings
}
unsafe fn cmd_list_keys_filter_key_list(
    filter_notes: ::core::ffi::c_int,
    filter_key: ::core::ffi::c_int,
    only: key_code,
    bindings: &mut Vec<&key_binding>,
) {
    bindings.retain(|&bd| {
        let key = (bd.key as ::core::ffi::c_ulonglong & (KEYC_MASK_KEY | KEYC_MASK_MODIFIERS))
            as key_code;
        (filter_key == 0 || only == key) && (filter_notes == 0 || !bd.note.is_none())
    });
}
unsafe fn cmd_list_keys_format_add_key_binding(
    mut ft: *mut format_tree,
    bd: &key_binding,
    prefix: &CStr,
) {
    if bd.flags & KEY_BINDING_REPEAT != 0 {
        format_add(
            ft,
            b"key_repeat\0" as *const u8 as *const ::core::ffi::c_char,
            |out| out.write_all(b"1"),
        );
    } else {
        format_add(
            ft,
            b"key_repeat\0" as *const u8 as *const ::core::ffi::c_char,
            |out| out.write_all(b"0"),
        );
    }
    if bd.note.is_some() {
        format_add(
            ft,
            b"key_note\0" as *const u8 as *const ::core::ffi::c_char,
            |out| {
                write_cstr(
                    out,
                    bd.note
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                )
            },
        );
    } else {
        format_add(
            ft,
            b"key_note\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write_cstr(out, b"\0" as *const u8 as *const ::core::ffi::c_char),
        );
    }
    let key_string = key_string_format(bd.key, false);
    format_add(
        ft,
        b"key_prefix\0" as *const u8 as *const ::core::ffi::c_char,
        |out| {
            write_cstr(
                out, // format_add copies the bytes synchronously; this pointer cannot escape.
                prefix.as_ptr(),
            )
        },
    );
    format_add(
        ft,
        b"key_table\0" as *const u8 as *const ::core::ffi::c_char,
        |out| {
            write_cstr(
                out,
                bd.tablename
                    .as_ref()
                    .map_or(::core::ptr::null(), |s| s.as_ptr()),
            )
        },
    );
    format_add(
        ft,
        b"key_string\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write_cstr(out, key_string.as_ptr()),
    );
    let command = cmd_list_print_cstring(
        &*bd.cmdlist(),
        CMD_LIST_PRINT_ESCAPED | CMD_LIST_PRINT_NO_GROUPS,
    );
    format_add(
        ft,
        b"key_command\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write_cstr(out, command.as_ptr()),
    );
}
unsafe fn cmd_list_keys_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut only: key_code = KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut tablename: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut keystr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut single: ::core::ffi::c_int = 0;
    let mut notes_only: ::core::ffi::c_int = 0;
    let mut filter_notes: ::core::ffi::c_int = 0;
    let mut filter_key: ::core::ffi::c_int = 0;
    let mut sort_crit: sort_criteria = sort_criteria {
        order: SORT_ACTIVITY,
        reversed: 0,
        order_seq: &[],
    };
    keystr = args_string(args, 0 as u_int);
    if !keystr.is_null() {
        only = key_string_parse_cstr(std::ffi::CStr::from_ptr(keystr)).unwrap_or(KEYC_UNKNOWN);
        if only == KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code {
            cmdq_error(item, |out| {
                out.write_all(b"invalid key: ")?;
                write_cstr(out, keystr)
            });
            return CMD_RETURN_ERROR;
        }
        only &= KEYC_MASK_KEY | KEYC_MASK_MODIFIERS;
    }
    sort_crit.order = sort_order_from_string(args_get(args, 'O' as i32 as u_char));
    if sort_crit.order as ::core::ffi::c_uint
        == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
        && args_has(args, 'O' as i32 as u_char) != 0
    {
        cmdq_error(item, |out| out.write_all(b"invalid sort order"));
        return CMD_RETURN_ERROR;
    }
    sort_crit.reversed = args_has(args, 'r' as i32 as u_char);
    tablename = args_get(args, 'T' as i32 as u_char);
    if !tablename.is_null() {
        table = key_bindings_get_table(tablename, 0 as ::core::ffi::c_int);
        if table.is_null() {
            cmdq_error(item, |out| {
                out.write_all(b"table ")?;
                write_cstr(out, tablename)?;
                out.write_all(b" doesn't exist")
            });
            return CMD_RETURN_ERROR;
        }
    }
    let prefix = cmd_list_keys_get_prefix(args);
    single = args_has(args, '1' as i32 as u_char);
    notes_only = args_has(args, 'N' as i32 as u_char);
    template = args_get(args, 'F' as i32 as u_char);
    if template.is_null() {
        template = LIST_KEYS_TEMPLATE.as_ptr();
    }
    let tables = key_bindings_tables();
    let mut bindings = if !table.is_null() {
        sort_get_key_bindings_table(table.as_ref(), &sort_crit)
    } else if notes_only != 0 {
        cmd_list_keys_get_root_and_prefix(&tables, &sort_crit)
    } else {
        sort_get_key_bindings(&tables, &sort_crit)
    };
    filter_notes =
        (notes_only != 0 && args_has(args, 'a' as i32 as u_char) == 0) as ::core::ffi::c_int;
    filter_key = (only != KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code) as ::core::ffi::c_int;
    if filter_notes != 0 || filter_key != 0 {
        cmd_list_keys_filter_key_list(filter_notes, filter_key, only, &mut bindings);
    }
    if filter_key != 0 && bindings.is_empty() {
        cmdq_error(item, |out| {
            out.write_all(b"unknown key: ")?;
            write_cstr(out, keystr)
        });
        return CMD_RETURN_ERROR;
    }
    if single != 0 {
        bindings.truncate(1);
    }
    ft = format_create(
        cmdq_get_client(item),
        item,
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    format_defaults(
        ft,
        tc,
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    format_add(
        ft,
        b"notes_only\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (notes_only) as i32),
    );
    format_add(
        ft,
        b"key_has_repeat\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (key_bindings_has_repeat(&bindings)) as i32),
    );
    format_add(
        ft,
        b"key_string_width\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (cmd_list_keys_get_width(&bindings)) as u32),
    );
    format_add(
        ft,
        b"key_table_width\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (cmd_list_keys_get_table_width(&bindings)) as u32),
    );
    for &bd in &bindings {
        cmd_list_keys_format_add_key_binding(ft, &*bd, &prefix);
        let line = format_expand_cstring(ft, template);
        if single != 0 && !tc.is_null() && !(*tc).flags & CLIENT_CONTROL as uint64_t != 0 {
            status_message_set(
                tc,
                -(1 as ::core::ffi::c_int),
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                |out| write_cstr(out, line.as_ptr()),
            );
        } else if !line.is_empty() {
            cmdq_print(item, |out| write_cstr(out, line.as_ptr()));
        }
        if single != 0 {
            break;
        }
    }
    format_free(ft);
    return CMD_RETURN_NORMAL;
}
