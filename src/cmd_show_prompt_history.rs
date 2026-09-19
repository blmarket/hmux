extern "C" {
    pub type args;
    pub type cmdq_item;
    pub type cmd;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn cmd_get_entry(_: *mut cmd) -> *const cmd_entry;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_print(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn prompt_type(_: *const ::core::ffi::c_char) -> prompt_type;
    fn prompt_type_string(_: prompt_type) -> *const ::core::ffi::c_char;
    fn prompt_history_size(_: prompt_type) -> u_int;
    fn prompt_history_get(_: prompt_type, _: u_int) -> *const ::core::ffi::c_char;
    fn prompt_history_clear(_: prompt_type);
}
pub type __u_char = ::core::ffi::c_uchar;
pub type __u_int = ::core::ffi::c_uint;
pub type u_char = __u_char;
pub type u_int = __u_int;
pub type args_parse_type = ::core::ffi::c_uint;
pub const ARGS_PARSE_COMMANDS: args_parse_type = 3;
pub const ARGS_PARSE_COMMANDS_OR_STRING: args_parse_type = 2;
pub const ARGS_PARSE_STRING: args_parse_type = 1;
pub const ARGS_PARSE_INVALID: args_parse_type = 0;
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
pub type cmd_find_type = ::core::ffi::c_uint;
pub const CMD_FIND_SESSION: cmd_find_type = 2;
pub const CMD_FIND_WINDOW: cmd_find_type = 1;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub type cmd_retval = ::core::ffi::c_int;
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
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
pub type prompt_type = ::core::ffi::c_uint;
pub const PROMPT_TYPE_INVALID: prompt_type = 255;
pub const PROMPT_TYPE_SEARCH: prompt_type = 1;
pub const PROMPT_TYPE_COMMAND: prompt_type = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const PROMPT_NTYPES: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
#[no_mangle]
pub static mut cmd_show_prompt_history_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"show-prompt-history\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"showphist\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"T:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-T prompt-type]\0" as *const u8 as *const ::core::ffi::c_char,
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
            cmd_show_prompt_history_exec
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_clear_prompt_history_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"clear-prompt-history\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"clearphist\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"T:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-T prompt-type]\0" as *const u8 as *const ::core::ffi::c_char,
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
            cmd_show_prompt_history_exec
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_show_prompt_history_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut typestr: *const ::core::ffi::c_char = args_get(args, 'T' as i32 as u_char);
    let mut type_0: prompt_type = PROMPT_TYPE_COMMAND;
    let mut t: u_int = 0;
    let mut h: u_int = 0;
    let mut v: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if cmd_get_entry(self_0) == &raw const cmd_clear_prompt_history_entry {
        if typestr.is_null() {
            t = 0 as u_int;
            while t < PROMPT_NTYPES as u_int {
                prompt_history_clear(t as prompt_type);
                t = t.wrapping_add(1);
            }
        } else {
            type_0 = prompt_type(typestr);
            if type_0 as ::core::ffi::c_uint
                == PROMPT_TYPE_INVALID as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                cmdq_error(
                    item,
                    b"invalid type: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    typestr,
                );
                return CMD_RETURN_ERROR;
            }
            prompt_history_clear(type_0);
        }
        return CMD_RETURN_NORMAL;
    }
    if typestr.is_null() {
        t = 0 as u_int;
        while t < PROMPT_NTYPES as u_int {
            typestr = prompt_type_string(t as prompt_type);
            cmdq_print(
                item,
                b"History for %s:\n\0" as *const u8 as *const ::core::ffi::c_char,
                typestr,
            );
            h = 0 as u_int;
            while h < prompt_history_size(t as prompt_type) {
                v = prompt_history_get(t as prompt_type, h);
                cmdq_print(
                    item,
                    b"%d: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    h.wrapping_add(1 as u_int),
                    v,
                );
                h = h.wrapping_add(1);
            }
            cmdq_print(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
            );
            t = t.wrapping_add(1);
        }
    } else {
        type_0 = prompt_type(typestr);
        if type_0 as ::core::ffi::c_uint
            == PROMPT_TYPE_INVALID as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            cmdq_error(
                item,
                b"invalid type: %s\0" as *const u8 as *const ::core::ffi::c_char,
                typestr,
            );
            return CMD_RETURN_ERROR;
        }
        cmdq_print(
            item,
            b"History for %s:\n\0" as *const u8 as *const ::core::ffi::c_char,
            prompt_type_string(type_0),
        );
        h = 0 as u_int;
        while h < prompt_history_size(type_0) {
            v = prompt_history_get(type_0, h);
            cmdq_print(
                item,
                b"%d: %s\0" as *const u8 as *const ::core::ffi::c_char,
                h.wrapping_add(1 as u_int),
                v,
            );
            h = h.wrapping_add(1);
        }
        cmdq_print(
            item,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    return CMD_RETURN_NORMAL;
}
