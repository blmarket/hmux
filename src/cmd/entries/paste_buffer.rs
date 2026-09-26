use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_target};
use crate::src::ffi::libc::{memchr, strlen};
use crate::src::paste::{paste_buffer_data, paste_free, paste_get_name, paste_get_top};
use crate::src::reactor::bufferevent_write;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::command::CMD_AFTERHOOK;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::PANE_INPUTOFF;
use crate::src::shared::paste::paste_buffer;
use crate::src::shared::screen::MODE_BRACKETPASTE;
use crate::src::shared::vis::{VIS_NOSLASH, VIS_SAFE};
use crate::src::text::utf8::utf8_stravisx_bytes;
use crate::src::window::window_pane_exited;

#[no_mangle]
pub static mut cmd_paste_buffer_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"paste-buffer\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"pasteb\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"db:prSs:t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-dprS] [-s separator] [-b buffer-name] [-t target-pane]\0" as *const u8
            as *const ::core::ffi::c_char,
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
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut wp: *mut window_pane = (*target).wp;
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut sepstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut bufname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut bufdata: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut bufend: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut line: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut seplen: size_t = 0;
    let mut bufsize: size_t = 0;
    let mut len: size_t = 0;
    let mut bracket: ::core::ffi::c_int = args_has(args, 'p' as i32 as u_char);
    if window_pane_exited(wp) != 0 {
        cmdq_error(
            item,
            b"target pane has exited\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    bufname = ::core::ptr::null::<::core::ffi::c_char>();
    if args_has(args, 'b' as i32 as u_char) != 0 {
        bufname = args_get(args, 'b' as i32 as u_char);
    }
    if bufname.is_null() {
        pb = paste_get_top(None);
    } else {
        pb = paste_get_name(bufname);
        if pb.is_null() {
            cmdq_error(
                item,
                b"no buffer %s\0" as *const u8 as *const ::core::ffi::c_char,
                bufname,
            );
            return CMD_RETURN_ERROR;
        }
    }
    if !pb.is_null() && !(*wp).flags & PANE_INPUTOFF != 0 {
        sepstr = args_get(args, 's' as i32 as u_char);
        if sepstr.is_null() {
            if args_has(args, 'r' as i32 as u_char) != 0 {
                sepstr = b"\n\0" as *const u8 as *const ::core::ffi::c_char;
            } else {
                sepstr = b"\r\0" as *const u8 as *const ::core::ffi::c_char;
            }
        }
        seplen = strlen(sepstr);
        if bracket != 0 && (*(*wp).screen).mode & MODE_BRACKETPASTE != 0 {
            bufferevent_write(
                (*wp).event,
                b"\x1B[200~\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                6 as size_t,
            );
        }
        bufdata = paste_buffer_data(pb, &raw mut bufsize);
        bufend = bufdata.offset(bufsize as isize);
        loop {
            line = memchr(
                bufdata as *const ::core::ffi::c_void,
                '\n' as i32,
                bufend.offset_from(bufdata) as ::core::ffi::c_long as size_t,
            ) as *const ::core::ffi::c_char;
            if line.is_null() {
                break;
            }
            len = line.offset_from(bufdata) as ::core::ffi::c_long as size_t;
            if args_has(args, 'S' as i32 as u_char) != 0 {
                bufferevent_write((*wp).event, bufdata as *const ::core::ffi::c_void, len);
            } else {
                cmd_paste_buffer_paste(&*wp, std::slice::from_raw_parts(bufdata.cast::<u8>(), len));
            }
            bufferevent_write((*wp).event, sepstr as *const ::core::ffi::c_void, seplen);
            bufdata = line.offset(1 as ::core::ffi::c_int as isize);
        }
        if bufdata != bufend {
            len = bufend.offset_from(bufdata) as ::core::ffi::c_long as size_t;
            if args_has(args, 'S' as i32 as u_char) != 0 {
                bufferevent_write((*wp).event, bufdata as *const ::core::ffi::c_void, len);
            } else {
                cmd_paste_buffer_paste(&*wp, std::slice::from_raw_parts(bufdata.cast::<u8>(), len));
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
    if !pb.is_null() && args_has(args, 'd' as i32 as u_char) != 0 {
        paste_free(pb);
    }
    return CMD_RETURN_NORMAL;
}
