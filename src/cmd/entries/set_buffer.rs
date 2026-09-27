use crate::src::arguments::{args_count, args_get, args_has, args_string};
use crate::src::cmd::queue::{cmdq_error, cmdq_get_target_client};
use crate::src::cmd::{cmd_get_args_mut, cmd_get_entry};
use crate::src::paste::{
    paste_buffer_data, paste_free, paste_get_name, paste_get_top, paste_rename, paste_set_owned,
};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args_parse;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmdq_item};
use crate::src::shared::command::{
    CMD_AFTERHOOK, CMD_BUFFER_USAGE, CMD_CLIENT_CANFAIL, CMD_CLIENT_TFLAG,
};
use crate::src::tty::tty_set_selection;
use std::ffi::CStr;
pub static cmd_set_buffer_entry: cmd_entry = {
    cmd_entry {
        name: c"set-buffer",
        alias: Some(c"setb"),
        args: args_parse {
            template: c"ab:t:n:w",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-aw] [-b buffer-name] [-n new-buffer-name] [-t target-client] [data]",
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
        exec: Some(cmd_set_buffer_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
pub static cmd_delete_buffer_entry: cmd_entry = {
    cmd_entry {
        name: c"delete-buffer",
        alias: Some(c"deleteb"),
        args: args_parse {
            template: c"b:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_BUFFER_USAGE,
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
        exec: Some(cmd_set_buffer_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_set_buffer_exec(self_0: *mut cmd, item: *mut cmdq_item) -> cmd_retval {
    let args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let tc = cmdq_get_target_client(item);
    let name = args_get(&*(args), b'b').map_or(std::ptr::null(), |value| value.as_ptr());
    let mut bufname = (!name.is_null()).then(|| CStr::from_ptr(name).to_owned());
    let mut pb = bufname.as_deref().and_then(paste_get_name);
    let mut cause = None;
    let deleting = std::ptr::eq(cmd_get_entry(&*self_0), &cmd_delete_buffer_entry);
    if deleting || args_has(args, b'n') != 0 {
        if pb.is_none() {
            if let Some(name) = bufname.as_ref() {
                cmdq_error(item, |out| {
                    out.write_all(b"unknown buffer: ")?;
                    out.write_all(name.as_bytes())
                });
                return CMD_RETURN_ERROR;
            }
            pb = paste_get_top(Some(&mut bufname));
        }
        let Some(pb) = pb else {
            cmdq_error(item, |out| out.write_all(b"no buffer"));
            return CMD_RETURN_ERROR;
        };
        if deleting {
            paste_free(&pb);
            return CMD_RETURN_NORMAL;
        }
        if paste_rename(
            bufname.as_deref(),
            Some(CStr::from_ptr(args_get(&*(args), b'n').map_or(std::ptr::null(), |value| value.as_ptr()))),
            Some(&mut cause),
        ) != 0
        {
            cmdq_error(item, |out| {
                out.write_all(cause.as_ref().unwrap().as_bytes())
            });
            return CMD_RETURN_ERROR;
        }
        return CMD_RETURN_NORMAL;
    }
    if args_count(args) != 1 {
        cmdq_error(item, |out| out.write_all(b"no data specified"));
        return CMD_RETURN_ERROR;
    }
    let new_data = CStr::from_ptr(args_string(args, 0)).to_bytes();
    if new_data.is_empty() {
        return CMD_RETURN_NORMAL;
    }
    let mut bufdata = Vec::new();
    if args_has(args, b'a') != 0 {
        if let Some(pb) = pb {
            bufdata.extend_from_slice(paste_buffer_data(&pb.borrow()).unwrap_or_default());
        }
    }
    bufdata.extend_from_slice(new_data);
    let selection_data = (args_has(args, b'w') != 0 && !tc.is_null()).then(|| bufdata.clone());
    if paste_set_owned(
        bufdata.into_boxed_slice(),
        bufname.as_deref(),
        Some(&mut cause),
    ) != 0
    {
        cmdq_error(item, |out| {
            out.write_all(cause.as_ref().unwrap().as_bytes())
        });
        return CMD_RETURN_ERROR;
    }
    if let Some(selection_data) = selection_data.as_ref() {
        tty_set_selection(&raw mut (*tc).tty, c"", selection_data);
    }
    CMD_RETURN_NORMAL
}
