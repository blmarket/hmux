use crate::src::shared::arguments::*;
use crate::src::shared::abi::*;
use crate::src::shared::command::*;
use crate::src::shared::key::*;
extern "C" {
    pub type args;
    pub type cmdq_item;
    pub type cmds;
    pub type cmd;
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn key_bindings_get_table(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> *mut key_table;
    fn key_bindings_remove(_: *const ::core::ffi::c_char, _: key_code);
    fn key_bindings_remove_table(_: *const ::core::ffi::c_char);
    fn key_string_lookup_string(_: *const ::core::ffi::c_char) -> key_code;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct key_table {
    pub name: *const ::core::ffi::c_char,
    pub activity_time: timeval,
    pub key_bindings: key_bindings,
    pub default_key_bindings: key_bindings,
    pub references: u_int,
    pub entry: C2RustUnnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed {
    pub rbe_left: *mut key_table,
    pub rbe_right: *mut key_table,
    pub rbe_parent: *mut key_table,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct key_bindings {
    pub rbh_root: *mut key_binding,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct key_binding {
    pub key: key_code,
    pub cmdlist: *mut cmd_list,
    pub note: *const ::core::ffi::c_char,
    pub tablename: *const ::core::ffi::c_char,
    pub flags: ::core::ffi::c_int,
    pub entry: C2RustUnnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_0 {
    pub rbe_left: *mut key_binding,
    pub rbe_right: *mut key_binding,
    pub rbe_parent: *mut key_binding,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_list {
    pub references: ::core::ffi::c_int,
    pub group: u_int,
    pub list: *mut cmds,
}
pub type C2RustUnnamed_1 = ::core::ffi::c_ulong;
pub type args_parse_cb = Option<
    unsafe extern "C" fn(*mut args, u_int, *mut *mut ::core::ffi::c_char) -> args_parse_type,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_parse {
    pub template: *const ::core::ffi::c_char,
    pub lower: ::core::ffi::c_int,
    pub upper: ::core::ffi::c_int,
    pub cb: args_parse_cb,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_entry_flag {
    pub flag: ::core::ffi::c_char,
    pub type_0: cmd_find_type,
    pub flags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_entry {
    pub name: *const ::core::ffi::c_char,
    pub alias: *const ::core::ffi::c_char,
    pub args: args_parse,
    pub usage: *const ::core::ffi::c_char,
    pub source: cmd_entry_flag,
    pub target: cmd_entry_flag,
    pub flags: ::core::ffi::c_int,
    pub exec: Option<unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval>,
}
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
#[no_mangle]
pub static mut cmd_unbind_key_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"unbind-key\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"unbind\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"anqT:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-anq] [-T key-table] key\0" as *const u8 as *const ::core::ffi::c_char,
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
        flags: CMD_AFTERHOOK,
        exec: Some(
            cmd_unbind_key_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_unbind_key_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut key: key_code = 0;
    let mut tablename: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut keystr: *const ::core::ffi::c_char = args_string(args, 0 as u_int);
    let mut quiet: ::core::ffi::c_int = args_has(args, 'q' as i32 as u_char);
    if args_has(args, 'a' as i32 as u_char) != 0 {
        if !keystr.is_null() {
            if quiet == 0 {
                cmdq_error(
                    item,
                    b"key given with -a\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return CMD_RETURN_ERROR;
        }
        tablename = args_get(args, 'T' as i32 as u_char);
        if tablename.is_null() {
            if args_has(args, 'n' as i32 as u_char) != 0 {
                tablename = b"root\0" as *const u8 as *const ::core::ffi::c_char;
            } else {
                tablename = b"prefix\0" as *const u8 as *const ::core::ffi::c_char;
            }
        }
        if key_bindings_get_table(tablename, 0 as ::core::ffi::c_int).is_null() {
            if quiet == 0 {
                cmdq_error(
                    item,
                    b"table %s doesn't exist\0" as *const u8 as *const ::core::ffi::c_char,
                    tablename,
                );
            }
            return CMD_RETURN_ERROR;
        }
        key_bindings_remove_table(tablename);
        return CMD_RETURN_NORMAL;
    }
    if keystr.is_null() {
        if quiet == 0 {
            cmdq_error(
                item,
                b"missing key\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return CMD_RETURN_ERROR;
    }
    key = key_string_lookup_string(keystr);
    if key == KEYC_NONE as ::core::ffi::c_ulong as key_code
        || key == KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code
    {
        if quiet == 0 {
            cmdq_error(
                item,
                b"unknown key: %s\0" as *const u8 as *const ::core::ffi::c_char,
                keystr,
            );
        }
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'T' as i32 as u_char) != 0 {
        tablename = args_get(args, 'T' as i32 as u_char);
        if key_bindings_get_table(tablename, 0 as ::core::ffi::c_int).is_null() {
            if quiet == 0 {
                cmdq_error(
                    item,
                    b"table %s doesn't exist\0" as *const u8 as *const ::core::ffi::c_char,
                    tablename,
                );
            }
            return CMD_RETURN_ERROR;
        }
    } else if args_has(args, 'n' as i32 as u_char) != 0 {
        tablename = b"root\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        tablename = b"prefix\0" as *const u8 as *const ::core::ffi::c_char;
    }
    key_bindings_remove(tablename, key);
    return CMD_RETURN_NORMAL;
}
