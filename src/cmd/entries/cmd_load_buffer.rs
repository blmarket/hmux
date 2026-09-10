use crate::args::args_parse_t;
use crate::args::RustArguments;
use crate::cmd::cmd_get_args;
use crate::cmd::cmdq_item;
use crate::cmd::{CmdqItemWeak, cmdq_item_weak_of};
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::compat::error_message;
use crate::consts::{
    CLIENT_DEAD, CMD_AFTERHOOK, CMD_CLIENT_CANFAIL, CMD_CLIENT_TFLAG, CMD_FIND_PANE,
    CMD_RETURN_WAIT,
};
use crate::file::file_read_for_client;
use crate::fmt_args;
use crate::format::format_single_from_target;
use crate::paste::{PasteBufferStore, paste_buffer_limit, with_paste_buffers_mut};
use crate::types::ClientFileEvent;
use crate::types::{ClientFileData, ClientRef, size_t, u_char, uint64_t};
use ::core::ffi::CStr;

#[derive(Clone, Default)]
#[repr(C)]
pub struct cmd_load_buffer_data {
    pub(crate) client_ref: Option<ClientRef>,
    pub(crate) item: Option<CmdqItemWeak>,
    pub name: Option<std::ffi::CString>,
}

impl cmd_load_buffer_data {
    /// The client the buffer is set for, if one was named.
    pub(crate) fn client(&self) -> Option<ClientRef> {
        self.client_ref.clone()
    }
}

pub(crate) static cmd_load_buffer_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"load-buffer",
        alias: Some(c"loadb"),
        args: args_parse_t {
            template: c"b:t:w",
            lower: 1 as core::ffi::c_int,
            upper: 1 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-b buffer-name] [-t target-client] path",
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
        exec: cmd_load_buffer_exec,
    }
};
pub(crate) fn cmd_load_buffer_done(event: ClientFileEvent<'_>) {
    unsafe {
        let ClientFileEvent::Done {
            path,
            error,
            mut buffer,
            data,
            ..
        } = event
        else {
            return;
        };
        let mut cdata_owner = match data {
            ClientFileData::LoadBuffer(cdata) => cdata,
            _ => panic!("load-buffer callback data is not load-buffer data"),
        };
        let cdata = &mut *cdata_owner;
        let mut tc = cdata.client();
        let item = cdata
            .item
            .as_ref()
            .and_then(CmdqItemWeak::upgrade)
            .expect("the item that asked for the buffer is waiting on it");
        let item_ref = item;
        let item = item_ref.read();
        let bytes = buffer.as_slice().to_vec();
        let bsize = bytes.len();
        if error != 0 as core::ffi::c_int {
            item.error(
                c"%s: %s",
                fmt_args![error_message(error).as_c_str(), path.as_c_str()],
            );
        } else if bsize != 0 as size_t {
            let copy = bytes.clone();
            let result = if let Some(name) = cdata.name.as_deref() {
                with_paste_buffers_mut(|buffers| buffers.set_named(name, copy))
            } else {
                let limit = paste_buffer_limit();
                with_paste_buffers_mut(|buffers| buffers.add_automatic(None, copy, limit));
                Ok(())
            };
            if let Err(cause) = result {
                item.error(c"%s", fmt_args![cause.as_c_str()]);
            } else if let Some(tc) = tc.as_mut()
                && !tc.attached_session().is_none()
                && !tc.flags() & CLIENT_DEAD as uint64_t != 0
            {
                tc.set_clipboard(&bytes);
            }
        }
        let _ = cdata.client_ref.take();
        drop(item);
        item_ref.resume();
    }
}
unsafe fn cmd_load_buffer_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &RustArguments = cmd_get_args(self_0);
    let target_client = item.target_client();
    let mut cdata = Box::<cmd_load_buffer_data>::default();
    cdata.item = cmdq_item_weak_of(item);
    cdata.name = {
        let flag = 'b' as i32 as u_char;
        args.argument_flag_string(flag)
    }
    .map(CStr::to_owned);
    if ({
        let flag = 'w' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        cdata.client_ref = target_client;
    }
    let path = unsafe {
        format_single_from_target(
            item,
            args.argument_string(0).expect("argument count checked"),
        )
    };
    unsafe {
        file_read_for_client(
            item.client().as_mut(),
            path.as_c_str(),
            Some(std::rc::Rc::new(cmd_load_buffer_done)),
            ClientFileData::LoadBuffer(cdata),
        )
    };
    CMD_RETURN_WAIT
}
