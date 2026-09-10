use crate::args::RustArguments;
use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::consts::{
    CMD_AFTERHOOK, CMD_BUFFER_USAGE, CMD_CLIENT_CANFAIL, CMD_CLIENT_TFLAG, CMD_FIND_PANE,
    CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
};
use crate::fmt_args;
use crate::paste::{
    PasteBufferStore, paste_buffer_limit, with_paste_buffers, with_paste_buffers_mut,
};
use crate::types::{args_parse_t, u_int};
use ::std::ffi::CStr;

pub(crate) static cmd_set_buffer_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"set-buffer",
        alias: Some(c"setb"),
        args: args_parse_t {
            template: c"ab:t:n:w",
            lower: 0 as core::ffi::c_int,
            upper: 1 as core::ffi::c_int,
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
        exec: cmd_set_buffer_exec,
    }
};
pub(crate) static cmd_delete_buffer_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"delete-buffer",
        alias: Some(c"deleteb"),
        args: args_parse_t {
            template: c"b:",
            lower: 0 as core::ffi::c_int,
            upper: 0 as core::ffi::c_int,
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
        exec: cmd_set_buffer_exec,
    }
};
unsafe fn cmd_set_buffer_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &RustArguments = cmd_get_args(self_0);
    let mut tc = item.target_client();
    let mut bufname = args.argument_flag_string(b'b').map(CStr::to_owned);
    let mut bufdata: Vec<u8> = Vec::new();
    let mut existing = bufname.as_deref().and_then(|name| {
        with_paste_buffers(|buffers| {
            buffers
                .get(name)
                .map(|buffer| (buffer.name.to_owned(), buffer.data.to_vec()))
        })
    });
    if core::ptr::eq(cmd_get_entry(self_0), &cmd_delete_buffer_entry) {
        if existing.is_none() && bufname.is_none() {
            existing = with_paste_buffers(|buffers| {
                buffers
                    .top()
                    .map(|buffer| (buffer.name.to_owned(), buffer.data.to_vec()))
            });
        }
        let Some((name, _)) = existing else {
            if let Some(name) = bufname {
                unsafe { item.error(c"unknown buffer: %s", fmt_args![name.as_c_str()]) };
            } else {
                unsafe { item.error(c"no buffer", fmt_args![]) };
            }
            return CMD_RETURN_ERROR;
        };
        with_paste_buffers_mut(|buffers| buffers.remove(name.as_c_str()));
        return CMD_RETURN_NORMAL;
    }
    if args.argument_flag_count(b'n') != 0 {
        if existing.is_none() && bufname.is_none() {
            existing = with_paste_buffers(|buffers| {
                buffers
                    .top()
                    .map(|buffer| (buffer.name.to_owned(), buffer.data.to_vec()))
            });
        }
        let Some((old_name, _)) = existing else {
            if let Some(name) = bufname {
                unsafe { item.error(c"unknown buffer: %s", fmt_args![name.as_c_str()]) };
            } else {
                unsafe { item.error(c"no buffer", fmt_args![]) };
            }
            return CMD_RETURN_ERROR;
        };
        let new_name = args.argument_flag_string(b'n').expect("-n has a value");
        if let Err(error) =
            with_paste_buffers_mut(|buffers| buffers.rename(old_name.as_c_str(), new_name))
        {
            unsafe { item.error(c"%s", fmt_args![error.as_c_str()]) };
            return CMD_RETURN_ERROR;
        } else {
            return CMD_RETURN_NORMAL;
        }
    }
    if args.argument_count() != 1 as u_int {
        unsafe { item.error(c"no data specified", fmt_args![]) };
        return CMD_RETURN_ERROR;
    }
    let value = args.argument_string(0).expect("argument count checked");
    if value.is_empty() {
        return CMD_RETURN_NORMAL;
    }
    if args.argument_flag_count(b'a') != 0
        && let Some((_, data)) = existing
    {
        bufdata.extend_from_slice(&data);
    }
    bufdata.extend_from_slice(value.to_bytes());
    let selection = match args.argument_flag_count(b'w') != 0 && tc.is_some() {
        true => Some(bufdata.clone()),
        false => None,
    };
    let result = if let Some(name) = bufname.take() {
        with_paste_buffers_mut(|buffers| buffers.set_named(name.as_c_str(), bufdata))
    } else {
        let limit = unsafe { paste_buffer_limit() };
        with_paste_buffers_mut(|buffers| buffers.add_automatic(None, bufdata, limit));
        Ok(())
    };
    match result {
        Err(error) => {
            unsafe { item.error(c"%s", fmt_args![error.as_c_str()]) };
            CMD_RETURN_ERROR
        }
        Ok(()) => {
            if let Some(selection) = selection {
                unsafe {
                    tc.as_mut()
                        .expect("the command has a client")
                        .set_clipboard(&selection)
                };
            }
            CMD_RETURN_NORMAL
        }
    }
}
