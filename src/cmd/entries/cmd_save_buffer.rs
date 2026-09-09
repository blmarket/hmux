use crate::args::RustArguments;
use crate::args::{args_get_str, args_has, args_string_str};
use crate::cmd::cmdq_item_weak_of;
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::compat::error_message;
use crate::consts::{
    CLIENT_CONTROL, CMD_AFTERHOOK, CMD_BUFFER_USAGE, CMD_FIND_PANE, CMD_RETURN_ERROR,
    CMD_RETURN_NORMAL, CMD_RETURN_WAIT, O_APPEND, O_TRUNC,
};
use crate::file::file_write_for_client;
use crate::fmt_args;
use crate::format::format_single_from_target;
use crate::paste::{PasteBufferStore, with_paste_buffers};
use crate::types::ClientFileEvent;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::cmd::cmdq_item;
use crate::types::{ByteBuffer, ClientFileData, args_parse_t, u_char, uint64_t};
use ::std::ffi::CString;

pub(crate) static cmd_save_buffer_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"save-buffer",
        alias: Some(c"saveb"),
        args: args_parse_t {
            template: c"ab:",
            lower: 1 as core::ffi::c_int,
            upper: 1 as core::ffi::c_int,
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
        exec: cmd_save_buffer_exec,
    }
};
pub(crate) static cmd_show_buffer_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"show-buffer",
        alias: Some(c"showb"),
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
        exec: cmd_save_buffer_exec,
    }
};
fn cmd_save_buffer_done(event: ClientFileEvent<'_>) {
    unsafe {
        let ClientFileEvent::Done {
            path, error, data, ..
        } = event
        else {
            return;
        };
        let observed = match data {
            ClientFileData::SaveBuffer(item) => item,
            _ => panic!("save-buffer callback data is not save-buffer data"),
        };
        let Some(item) = observed.upgrade() else {
            return;
        };
        if error != 0 as core::ffi::c_int {
            (item.read()).error(
                c"%s: %s",
                fmt_args![error_message(error).as_c_str(), path.as_c_str()],
            );
        }
        item.resume();
    }
}
unsafe fn cmd_save_buffer_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &RustArguments = cmd_get_args(self_0);
    let c = item.client();
    let bufdata = if let Some(bufname) = args_get_str(args, 'b' as i32 as u_char) {
        let data =
            with_paste_buffers(|buffers| buffers.get(bufname).map(|buffer| buffer.data.to_vec()));
        if data.is_none() {
            unsafe { item.error(c"no buffer %s", fmt_args![bufname]) };
            return CMD_RETURN_ERROR;
        }
        data
    } else {
        let data = with_paste_buffers(|buffers| buffers.top().map(|buffer| buffer.data.to_vec()));
        if data.is_none() {
            unsafe { item.error(c"no buffers", fmt_args![]) };
            return CMD_RETURN_ERROR;
        }
        data
    }
    .expect("the selected paste buffer was checked");
    let path: CString = if core::ptr::eq(cmd_get_entry(self_0), &cmd_show_buffer_entry) {
        let c = c.as_ref().expect("the command has a client");
        if unsafe { !c.attached_session().is_none() || c.flags() & CLIENT_CONTROL as uint64_t != 0 }
        {
            let mut evb = ByteBuffer::new();
            evb.append(&bufdata);
            unsafe { item.print_data(&mut evb) };
            return CMD_RETURN_NORMAL;
        }
        c"-".to_owned()
    } else {
        unsafe {
            format_single_from_target(
                item,
                args_string_str(args, 0).expect("argument count checked"),
            )
        }
    };
    let flags = if args_has(args, 'a' as i32 as u_char) != 0 {
        O_APPEND
    } else {
        O_TRUNC
    };
    unsafe {
        file_write_for_client(
            item.client().as_mut(),
            path.as_c_str(),
            flags,
            &bufdata,
            Some(std::rc::Rc::new(cmd_save_buffer_done)),
            ClientFileData::SaveBuffer(cmdq_item_weak_of(item).expect("the saving item is live")),
        )
    };
    CMD_RETURN_WAIT
}
