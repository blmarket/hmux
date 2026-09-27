use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_target};
use crate::src::format::bytes::write_cstr;
use crate::src::paste::{paste_buffer_data, paste_free, paste_get_name, paste_get_top};
use crate::src::reactor::bufferevent_write;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::command::CMD_AFTERHOOK;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::PANE_INPUTOFF;
use crate::src::shared::screen::MODE_BRACKETPASTE;
use crate::src::shared::vis::{VIS_NOSLASH, VIS_SAFE};
use crate::src::text::utf8::utf8_stravisx_bytes;
use crate::src::window::window_pane_exited;
use std::ffi::CStr;
pub static cmd_paste_buffer_entry: cmd_entry = {
    cmd_entry {
        name: c"paste-buffer",
        alias: Some(c"pasteb"),
        args: args_parse {
            template: c"db:prSs:t:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-dprS] [-s separator] [-b buffer-name] [-t target-pane]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_paste_buffer_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_paste_buffer_paste(wp: &window_pane, buf: &[u8]) {
    let escaped = utf8_stravisx_bytes(buf, VIS_SAFE | VIS_NOSLASH);
    bufferevent_write(wp.event, escaped.as_ptr().cast(), escaped.len());
}
unsafe fn cmd_paste_buffer_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut wp: *mut window_pane = (*target).wp;
    let pb;
    let mut sepstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut bufname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut bracket: ::core::ffi::c_int = args_has(args, 'p' as i32 as u_char);
    if window_pane_exited(wp) != 0 {
        cmdq_error(item, |out| out.write_all(b"target pane has exited"));
        return CMD_RETURN_ERROR;
    }
    bufname = ::core::ptr::null::<::core::ffi::c_char>();
    if args_has(args, 'b' as i32 as u_char) != 0 {
        bufname = args_get(args, 'b' as i32 as u_char);
    }
    if bufname.is_null() {
        pb = paste_get_top(None);
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
    let Some(pb) = pb else {
        return CMD_RETURN_NORMAL;
    };
    if !(*wp).flags & PANE_INPUTOFF != 0 {
        sepstr = args_get(args, 's' as i32 as u_char);
        if sepstr.is_null() {
            if args_has(args, 'r' as i32 as u_char) != 0 {
                sepstr = b"\n\0" as *const u8 as *const ::core::ffi::c_char;
            } else {
                sepstr = b"\r\0" as *const u8 as *const ::core::ffi::c_char;
            }
        }
        let separator = CStr::from_ptr(sepstr).to_bytes();
        if bracket != 0 && (*(*wp).screen).mode & MODE_BRACKETPASTE != 0 {
            bufferevent_write(
                (*wp).event,
                b"\x1B[200~\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                6 as size_t,
            );
        }
        let buffer = pb.borrow();
        let bufdata = paste_buffer_data(&buffer).unwrap_or_default();
        for chunk in bufdata.split_inclusive(|&byte| byte == b'\n') {
            let line = chunk.strip_suffix(b"\n").unwrap_or(chunk);
            if args_has(args, 'S' as i32 as u_char) != 0 {
                bufferevent_write((*wp).event, line.as_ptr().cast(), line.len());
            } else {
                cmd_paste_buffer_paste(&*wp, line);
            }
            if line.len() != chunk.len() {
                bufferevent_write((*wp).event, separator.as_ptr().cast(), separator.len());
            }
        }
        if bracket != 0 && (*(*wp).screen).mode & MODE_BRACKETPASTE != 0 {
            bufferevent_write(
                (*wp).event,
                b"\x1B[201~\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                6 as size_t,
            );
        }
    }
    if args_has(args, 'd' as i32 as u_char) != 0 {
        paste_free(&pb);
    }
    return CMD_RETURN_NORMAL;
}
