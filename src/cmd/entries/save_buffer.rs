use crate::src::arguments::{args_get, args_has, args_string};
use crate::src::cmd::queue::{cmdq_continue, cmdq_error, cmdq_get_client, cmdq_print_data};
use crate::src::cmd::{cmd_get_args, cmd_get_entry};
use crate::src::ffi::libc::strerror;
use crate::src::file::file_write_with_cmdq_wait;
use crate::src::format::bytes::write_cstr;
use crate::src::format::format_single_from_target_cstring;
use crate::src::paste::{paste_buffer_data, paste_get_name, paste_get_top};
use crate::src::reactor::{evbuffer_add, evbuffer_new};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::client::CLIENT_CONTROL;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmdq_item};
use crate::src::shared::command::{CMD_AFTERHOOK, CMD_BUFFER_USAGE};
use crate::src::shared::event::*;
use crate::src::shared::posix_io::{O_APPEND, O_TRUNC};
use std::ffi::CStr;
pub static cmd_save_buffer_entry: cmd_entry = {
    cmd_entry {
        name: c"save-buffer",
        alias: Some(c"saveb"),
        args: args_parse {
            template: c"ab:",
            lower: 1 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-a] [-b buffer-name] path",
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
        exec: Some(cmd_save_buffer_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
pub static cmd_show_buffer_entry: cmd_entry = {
    cmd_entry {
        name: c"show-buffer",
        alias: Some(c"showb"),
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
        exec: Some(cmd_save_buffer_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_save_buffer_done(
    mut item: *mut cmdq_item,
    path: Option<&CStr>,
    mut error: ::core::ffi::c_int,
    mut closed: ::core::ffi::c_int,
) {
    if closed == 0 {
        return;
    }
    if error != 0 as ::core::ffi::c_int {
        cmdq_error(item, |out| {
            write_cstr(out, strerror(error))?;
            out.write_all(b": ")?;
            write_cstr(out, path.map_or(::core::ptr::null(), CStr::as_ptr))
        });
    }
    cmdq_continue(item);
}
unsafe fn cmd_save_buffer_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut c: *mut client = cmdq_get_client(item);
    let pb;
    let mut flags: ::core::ffi::c_int = 0;
    let mut bufname: *const ::core::ffi::c_char = args_get(args, 'b' as i32 as u_char);
    if bufname.is_null() {
        pb = paste_get_top(None);
        if pb.is_none() {
            cmdq_error(item, |out| out.write_all(b"no buffers"));
            return CMD_RETURN_ERROR;
        }
    } else {
        pb = paste_get_name(CStr::from_ptr(bufname));
        if pb.is_none() {
            cmdq_error(item, |out| {
                out.write_all(b"no buffer ")?;
                write_cstr(out, bufname)
            });
            return CMD_RETURN_ERROR;
        }
    }
    let pb = pb.expect("buffer lookup checked above");
    let show_buffer = std::ptr::eq(cmd_get_entry(&*self_0), &cmd_show_buffer_entry);
    if show_buffer {
        if !(*c).session.is_null() || (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            let mut evb = evbuffer_new();
            {
                let buffer = pb.borrow();
                let bufdata = paste_buffer_data(&buffer).unwrap_or_default();
                evbuffer_add(&mut *evb, bufdata.as_ptr().cast(), bufdata.len());
            }
            cmdq_print_data(item, &mut *evb);
            return CMD_RETURN_NORMAL;
        }
    }
    let expanded_path = (!show_buffer)
        .then(|| format_single_from_target_cstring(item, args_string(args, 0 as u_int)));
    let dash_path = CStr::from_bytes_with_nul(b"-\0").unwrap();
    let path: *const ::core::ffi::c_char = expanded_path
        .as_ref()
        .map_or(dash_path.as_ptr(), |path| path.as_ptr());
    if args_has(args, 'a' as i32 as u_char) != 0 {
        flags = O_APPEND;
    } else {
        flags = O_TRUNC;
    }
    let buffer = pb.borrow();
    let bufdata = paste_buffer_data(&buffer).unwrap_or_default();
    file_write_with_cmdq_wait(
        cmdq_get_client(item),
        path,
        flags,
        bufdata.as_ptr().cast(),
        bufdata.len(),
        Some(Box::new(move |event| unsafe {
            cmd_save_buffer_done(
                item,
                event.path,
                event.error,
                event.closed as ::core::ffi::c_int,
            )
        })),
        item,
    );
    return CMD_RETURN_WAIT;
}
