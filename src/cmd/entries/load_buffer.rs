use crate::src::server_client::server_client_unref_owned;
use std::cell::UnsafeCell;
use std::rc::Rc;
use crate::src::shared::client::{client_retain, client_rc_ptr};
use crate::src::arguments::{args_get, args_has, args_string};
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::queue::{cmdq_continue, cmdq_error, cmdq_get_client, cmdq_get_target_client};
use crate::src::ffi::libc::strerror;
use crate::src::file::file_read_with_cmdq_wait;
use crate::src::format::bytes::write_cstr;
use crate::src::format::format_single_from_target_cstring;
use crate::src::paste::paste_set_owned;
use crate::src::reactor::{evbuffer_get_length, evbuffer_pullup};
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::client::CLIENT_DEAD;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmdq_item};
use crate::src::shared::command::{CMD_AFTERHOOK, CMD_CLIENT_CANFAIL, CMD_CLIENT_TFLAG};
use crate::src::shared::event::*;
use crate::src::tty::tty_set_selection;
use std::ffi::{CStr, CString};

#[repr(C)]
pub struct cmd_load_buffer_data {
    pub client: Option<Rc<UnsafeCell<client>>>,
    pub item: *mut cmdq_item,
    pub name: Option<CString>,
}

impl cmd_load_buffer_data {
    unsafe fn release_client(&mut self) {
        if let Some(client) = self.client.take() {
            server_client_unref_owned(client);
        }
    }
}

impl Drop for cmd_load_buffer_data {
    fn drop(&mut self) {
        unsafe { self.release_client() }
    }
}
pub static cmd_load_buffer_entry: cmd_entry = {
    cmd_entry {
        name: c"load-buffer",
        alias: Some(c"loadb"),
        args: args_parse {
            template: c"b:t:w",
            lower: 1 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
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
        exec: Some(cmd_load_buffer_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_load_buffer_done(
    cdata: &mut cmd_load_buffer_data,
    path: Option<&CStr>,
    mut error: ::core::ffi::c_int,
    mut closed: ::core::ffi::c_int,
    mut buffer: &mut evbuffer,
) {
    // Progress notifications do not need contiguous storage. Coalesce only
    // once, after the complete file has arrived.
    if closed == 0 {
        return;
    }
    let mut tc: *mut client = client_rc_ptr(&cdata.client);
    let mut item: *mut cmdq_item = cdata.item;
    let mut bdata: *mut ::core::ffi::c_void =
        evbuffer_pullup(buffer, -1)
            .map_or(std::ptr::null_mut(), |bytes| bytes.as_mut_ptr()) as *mut ::core::ffi::c_void;
    let mut bsize: size_t = evbuffer_get_length(&*(buffer));
    let mut cause: Option<CString> = None;
    if error != 0 as ::core::ffi::c_int {
        cmdq_error(item, |out| {
            write_cstr(out, strerror(error))?;
            out.write_all(b": ")?;
            write_cstr(out, path.map_or(::core::ptr::null(), CStr::as_ptr))
        });
    } else if bsize != 0 as size_t {
        let owned: Box<[u8]> = std::slice::from_raw_parts(bdata.cast::<u8>(), bsize).into();
        if paste_set_owned(
            owned,
            cdata.name.as_deref(),
            Some(&mut cause),
        ) != 0 as ::core::ffi::c_int
        {
            cmdq_error(item, |out| {
                write_cstr(out, cause.as_ref().unwrap().as_ptr())
            });
        } else if !tc.is_null()
            && !(*tc).session.is_null()
            && !(*tc).flags & CLIENT_DEAD as uint64_t != 0
        {
            tty_set_selection(
                &raw mut (*tc).tty,
                c"",
                std::slice::from_raw_parts(bdata.cast(), bsize),
            );
        }
    }
    cdata.release_client();
    cmdq_continue(item);
}
unsafe fn cmd_load_buffer_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut cdata = Box::new(cmd_load_buffer_data {
        client: None,
        item,
        name: None,
    });
    let mut bufname: *const ::core::ffi::c_char = args_get(&*(args), 'b' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
    if !bufname.is_null() {
        cdata.name = Some(CStr::from_ptr(bufname).to_owned());
    }
    if args_has(args, 'w' as i32 as u_char) != 0 && !tc.is_null() {
        cdata.client = client_retain(tc);
    }
    let path = format_single_from_target_cstring(item, args_string(&mut *(args), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr()));
    file_read_with_cmdq_wait(
        cmdq_get_client(item),
        path.as_ptr(),
        Some(Box::new(move |event| unsafe {
            cmd_load_buffer_done(
                &mut cdata,
                event.path,
                event.error,
                event.closed as ::core::ffi::c_int,
                event.buffer.expect("read callback buffer"),
            )
        })),
        item,
        None,
    );
    return CMD_RETURN_WAIT;
}

#[cfg(test)]
mod tests {
    use super::*;
    use hmux_buffer::{Buf, BufMut, Buffer, SegmentedBuf};

    #[test]
    fn file_progress_keeps_segments_until_completion() {
        let mut data = cmd_load_buffer_data {
            client: None,
            item: std::ptr::null_mut(),
            name: None,
        };
        let mut buffer = SegmentedBuf::from(vec![1; 4096]);
        let first = buffer.chunk().as_ptr();
        for count in 2..=16 {
            buffer.put(SegmentedBuf::from(vec![2; 4096]));
            unsafe {
                cmd_load_buffer_done(&mut data, Some(c"input"), 0, 0, &mut buffer);
            }
            assert_eq!(buffer.chunks().count(), count);
            assert_eq!(buffer.chunk().as_ptr(), first);
            assert_eq!(buffer.remaining(), count * 4096);
        }
    }
}
