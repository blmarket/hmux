use crate::src::arguments::{args_count, args_get, args_has, args_string};
use crate::src::cmd::queue::{cmdq_error, cmdq_get_target_client};
use crate::src::cmd::{cmd_get_args, cmd_get_entry};
use crate::src::paste::{
    paste_buffer_data, paste_buffer_name, paste_free, paste_get_name, paste_get_top, paste_rename,
    paste_set_owned,
};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmdq_item};
use crate::src::shared::command::{
    CMD_AFTERHOOK, CMD_BUFFER_USAGE, CMD_CLIENT_CANFAIL, CMD_CLIENT_TFLAG,
};
use crate::src::shared::paste::paste_buffer;
use crate::src::tty::tty_set_selection;
use std::ffi::{CStr, CString};

#[no_mangle]
pub static mut cmd_set_buffer_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"set-buffer\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"setb\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"ab:t:n:w\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-aw] [-b buffer-name] [-n new-buffer-name] [-t target-client] [data]\0"
            as *const u8 as *const ::core::ffi::c_char,
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
        flags: CMD_AFTERHOOK | CMD_CLIENT_TFLAG | CMD_CLIENT_CANFAIL,
        exec: Some(
            cmd_set_buffer_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_delete_buffer_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"delete-buffer\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"deleteb\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"b:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_BUFFER_USAGE.as_ptr(),
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
            cmd_set_buffer_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe fn cmd_set_buffer_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut current_block: u64;
    let mut args: *mut args = cmd_get_args(self_0);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut bufname: Option<CString> = None;
    let mut bufdata = Vec::new();
    let mut cause: Option<CString> = None;
    if !args_get(args, 'b' as i32 as u_char).is_null() {
        bufname = Some(CStr::from_ptr(args_get(args, 'b' as i32 as u_char)).to_owned());
        pb = paste_get_name(bufname.as_ref().unwrap().as_ptr());
    }
    if cmd_get_entry(self_0) == &raw const cmd_delete_buffer_entry {
        if pb.is_null() {
            if let Some(bufname) = bufname.as_ref() {
                cmdq_error(
                    item,
                    b"unknown buffer: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    bufname.as_ptr(),
                );
                current_block = 17843714670734105592;
            } else {
                pb = paste_get_top(None);
                if !pb.is_null() {
                    bufname = Some(CStr::from_ptr(paste_buffer_name(pb)).to_owned());
                }
                current_block = 3640593987805443782;
            }
        } else {
            current_block = 3640593987805443782;
        }
        match current_block {
            17843714670734105592 => {}
            _ => {
                if pb.is_null() {
                    cmdq_error(
                        item,
                        b"no buffer\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                } else {
                    paste_free(pb);
                    return CMD_RETURN_NORMAL;
                }
            }
        }
    } else if args_has(args, 'n' as i32 as u_char) != 0 {
        if pb.is_null() {
            if let Some(bufname) = bufname.as_ref() {
                cmdq_error(
                    item,
                    b"unknown buffer: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    bufname.as_ptr(),
                );
                current_block = 17843714670734105592;
            } else {
                pb = paste_get_top(None);
                if !pb.is_null() {
                    bufname = Some(CStr::from_ptr(paste_buffer_name(pb)).to_owned());
                }
                current_block = 15904375183555213903;
            }
        } else {
            current_block = 15904375183555213903;
        }
        match current_block {
            17843714670734105592 => {}
            _ => {
                if pb.is_null() {
                    cmdq_error(
                        item,
                        b"no buffer\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                } else if paste_rename(
                    bufname.as_ref().unwrap().as_ptr(),
                    args_get(args, 'n' as i32 as u_char),
                    &raw mut cause,
                ) != 0 as ::core::ffi::c_int
                {
                    cmdq_error(
                        item,
                        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                        cause.as_ref().unwrap().as_ptr(),
                    );
                } else {
                    return CMD_RETURN_NORMAL;
                }
            }
        }
    } else if args_count(args) != 1 as u_int {
        cmdq_error(
            item,
            b"no data specified\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        let new_data = CStr::from_ptr(args_string(args, 0 as u_int)).to_bytes();
        if new_data.is_empty() {
            return CMD_RETURN_NORMAL;
        }
        if args_has(args, 'a' as i32 as u_char) != 0 && !pb.is_null() {
            let mut oldsize = 0;
            let olddata = paste_buffer_data(pb, &raw mut oldsize);
            if oldsize != 0 {
                bufdata
                    .extend_from_slice(std::slice::from_raw_parts(olddata.cast::<u8>(), oldsize));
            }
        }
        bufdata.extend_from_slice(new_data);
        let selection_data = if args_has(args, 'w' as i32 as u_char) != 0 && !tc.is_null() {
            Some(bufdata.clone())
        } else {
            None
        };
        let name = bufname
            .as_ref()
            .map_or(::core::ptr::null(), |name| name.as_ptr());
        if paste_set_owned(bufdata.into_boxed_slice(), name, Some(&mut cause))
            != 0 as ::core::ffi::c_int
        {
            cmdq_error(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                cause.as_ref().unwrap().as_ptr(),
            );
        } else {
            if let Some(selection_data) = selection_data.as_ref() {
                tty_set_selection(
                    &raw mut (*tc).tty,
                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                    selection_data.as_ptr().cast(),
                    selection_data.len(),
                );
            }
            return CMD_RETURN_NORMAL;
        }
    }
    return CMD_RETURN_ERROR;
}
