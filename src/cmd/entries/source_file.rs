use crate::src::arguments::{args_count, args_has, args_string};
use crate::src::cfg::{cfg_finished, cfg_print_causes, load_cfg_from_buffer};
use crate::src::cmd::queue::{
    cmdq_continue, cmdq_error, cmdq_get_callback_owned, cmdq_get_client, cmdq_get_target,
    cmdq_insert_after, cmdq_set_cancel_callback,
};
use crate::src::cmd::{cmd_get_args, cmd_get_parse_flags};
use crate::src::compat::glob::GlobResult;
use crate::src::ffi::libc::{__ctype_b_loc, strcmp, strerror};
use crate::src::file::file_read_with_cmdq_wait;
use crate::src::format::format_single_from_target_cstring;
use crate::src::log::log_debug;
use crate::src::reactor::{evbuffer_get_length, evbuffer_pullup};
use crate::src::server_client::{server_client_get_cwd, server_client_unref};
use crate::src::shared::abi::*;
use crate::src::shared::abi::{__size_t, ssize_t};
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::client::CLIENT_CONTROL;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::command::{
    CMD_FIND_CANFAIL, CMD_PARSE_PARSEONLY, CMD_PARSE_QUIET, CMD_PARSE_VERBOSE,
    CMD_SOURCE_FILE_DEPTH_LIMIT,
};
use crate::src::shared::ctype::_ISalnum;
use crate::src::shared::errno::{EINVAL, ENOENT, ENOMEM};
use crate::src::shared::event::*;
use crate::src::shared::session::session;
use std::ffi::{CStr, CString};

pub struct cmd_source_file_data {
    pub item: *mut cmdq_item,
    pub client: *mut client,
    pub flags: ::core::ffi::c_int,
    pub after: *mut cmdq_item,
    pub retval: cmd_retval,
    pub current: u_int,
    pub files: Vec<CString>,
}

pub const GLOB_NOSPACE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const GLOB_NOMATCH: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
static mut cmd_source_file_depth: u_int = 0;
#[no_mangle]
pub static mut cmd_source_file_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"source-file\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"source\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"t:Fnqv\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: b"[-Fnqv] [-t target-pane] path ...\0" as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: CMD_FIND_CANFAIL,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_source_file_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe fn cmd_source_file_free_data(cdata: *mut cmd_source_file_data) {
    let mut cdata = Box::from_raw(cdata);
    // Preserve the old order: release path copies before dropping the client reference.
    drop(std::mem::take(&mut cdata.files));
    if !cdata.client.is_null() {
        server_client_unref(cdata.client);
    }
}
unsafe fn cmd_source_file_decrement_depth(cdata: &cmd_source_file_data) {
    let c = cdata.client;
    if c.is_null() {
        cmd_source_file_depth = cmd_source_file_depth.wrapping_sub(1);
        log_debug(
            b"%s: depth now %u\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_source_file_complete_cb\0" as *const u8 as *const ::core::ffi::c_char,
            cmd_source_file_depth,
        );
    } else {
        (*c).source_file_depth = (*c).source_file_depth.wrapping_sub(1);
        log_debug(
            b"%s: depth now %u\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_source_file_complete_cb\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).source_file_depth,
        );
    }
}
unsafe fn cmd_source_file_cancel_complete_typed(cdata: *mut cmd_source_file_data) {
    cmd_source_file_decrement_depth(&*cdata);
    cmd_source_file_free_data(cdata);
}
unsafe fn cmd_source_file_complete_cb(
    item: *mut cmdq_item,
    cdata: *mut cmd_source_file_data,
) -> cmd_retval {
    cmd_source_file_decrement_depth(&*cdata);
    cfg_print_causes(item);
    cmd_source_file_free_data(cdata);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_source_file_complete(mut cdata: *mut cmd_source_file_data) {
    let mut c: *mut client = (*cdata).client;
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    if cfg_finished == 0 {
        cmd_source_file_free_data(cdata);
        return;
    }
    if (*cdata).retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int
        && !c.is_null()
        && (*c).session.is_null()
    {
        (*c).retval = 1 as ::core::ffi::c_int;
    }
    new_item = cmdq_get_callback_owned(
        b"cmd_source_file_complete_cb\0" as *const u8 as *const ::core::ffi::c_char,
        Some(Box::new(move |item| unsafe {
            cmd_source_file_complete_cb(item, cdata)
        })),
        ::core::ptr::null_mut(),
    );
    cmdq_set_cancel_callback(
        &mut *new_item,
        Box::new(move || unsafe {
            cmd_source_file_cancel_complete_typed(cdata)
        }),
    );
    cmdq_insert_after((*cdata).after, new_item);
}
unsafe fn cmd_source_file_done(
    mut cdata: *mut cmd_source_file_data,
    path: Option<&CStr>,
    mut error: ::core::ffi::c_int,
    mut closed: ::core::ffi::c_int,
    mut buffer: *mut evbuffer,
) {
    // Progress notifications do not need contiguous storage. Coalesce only
    // once, after the complete file has arrived.
    if closed == 0 {
        return;
    }
    let path = path.map_or(::core::ptr::null(), CStr::as_ptr);
    let mut item: *mut cmdq_item = (*cdata).item;
    let mut c: *mut client = (*cdata).client;
    let mut bdata: *mut ::core::ffi::c_void =
        evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t) as *mut ::core::ffi::c_void;
    let mut bsize: size_t = evbuffer_get_length(&*(buffer));
    let mut n: u_int = 0;
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    if error != 0 as ::core::ffi::c_int {
        cmdq_error(
            item,
            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            strerror(error),
            path,
        );
    } else if bsize != 0 as size_t {
        if load_cfg_from_buffer(
            bdata,
            bsize,
            path,
            c,
            (*cdata).after,
            target,
            (*cdata).flags,
            &raw mut new_item,
        ) < 0 as ::core::ffi::c_int
        {
            (*cdata).retval = CMD_RETURN_ERROR;
        } else if !new_item.is_null() {
            (*cdata).after = new_item;
        }
    }
    (*cdata).current = (*cdata).current.wrapping_add(1);
    n = (*cdata).current;
    if (n as usize) < (*cdata).files.len() {
        let next_path = (&(*cdata).files)[n as usize].as_ptr();
        file_read_with_cmdq_wait(
            c,
            next_path,
            Some(Box::new(move |event| unsafe {
                cmd_source_file_done(
                    cdata,
                    event.path,
                    event.error,
                    event.closed as ::core::ffi::c_int,
                    event
                        .buffer
                        .map_or(::core::ptr::null_mut(), |buffer| buffer.as_ptr()),
                )
            })),
            item,
            Some(Box::new(move || unsafe {
                cmd_source_file_cancel_complete_typed(cdata)
            })),
        );
    } else {
        cmd_source_file_complete(cdata);
        cmdq_continue(item);
    };
}
unsafe fn cmd_source_file_add(cdata: &mut cmd_source_file_data, path: &CStr) {
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_source_file_add\0" as *const u8 as *const ::core::ffi::c_char,
        path.as_ptr(),
    );
    cdata.files.push(path.to_owned());
}
unsafe fn cmd_source_file_quote_for_glob(path: &CStr) -> CString {
    let mut quoted = Vec::new();
    for &byte in path.to_bytes() {
        if byte < 128
            && *(*__ctype_b_loc()).offset(byte as isize) as ::core::ffi::c_int
                & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                == 0
            && byte != b'/'
        {
            quoted.push(b'\\');
        }
        quoted.push(byte);
    }
    CString::new(quoted).expect("C string path has no interior NUL")
}
unsafe extern "C" fn cmd_source_file_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut cdata: *mut cmd_source_file_data = ::core::ptr::null_mut::<cmd_source_file_data>();
    let mut c: *mut client = cmdq_get_client(item);
    let mut retval: cmd_retval = CMD_RETURN_NORMAL;
    let mut path: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut parse_flags: ::core::ffi::c_int = 0;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    if c.is_null() {
        if cmd_source_file_depth >= CMD_SOURCE_FILE_DEPTH_LIMIT as u_int {
            cmdq_error(
                item,
                b"too many nested files\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
        cmd_source_file_depth = cmd_source_file_depth.wrapping_add(1);
        log_debug(
            b"%s: depth now %u\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_source_file_exec\0" as *const u8 as *const ::core::ffi::c_char,
            cmd_source_file_depth,
        );
    } else {
        if (*c).source_file_depth >= CMD_SOURCE_FILE_DEPTH_LIMIT as u_int {
            cmdq_error(
                item,
                b"too many nested files\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
        (*c).source_file_depth = (*c).source_file_depth.wrapping_add(1);
        log_debug(
            b"%s: depth now %u\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_source_file_exec\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).source_file_depth,
        );
    }
    cdata = Box::into_raw(Box::new(cmd_source_file_data {
        item,
        client: c,
        flags: 0,
        after: ::core::ptr::null_mut(),
        retval: CMD_RETURN_NORMAL,
        current: 0,
        files: Vec::new(),
    }));
    if !c.is_null() {
        (*c).references += 1;
    }
    if args_has(args, 'q' as i32 as u_char) != 0 {
        (*cdata).flags |= CMD_PARSE_QUIET;
    }
    if args_has(args, 'n' as i32 as u_char) != 0 {
        (*cdata).flags |= CMD_PARSE_PARSEONLY;
    }
    if c.is_null() || !(*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        parse_flags = cmd_get_parse_flags(self_0);
        if args_has(args, 'v' as i32 as u_char) != 0 || parse_flags & CMD_PARSE_VERBOSE != 0 {
            (*cdata).flags |= CMD_PARSE_VERBOSE;
        }
    }
    let cwd = cmd_source_file_quote_for_glob(CStr::from_ptr(server_client_get_cwd(
        c,
        ::core::ptr::null_mut::<session>(),
    )));
    i = 0 as u_int;
    while i < args_count(args) {
        path = args_string(args, i);
        let expanded = if args_has(args, 'F' as i32 as u_char) != 0 {
            Some(format_single_from_target_cstring(item, path))
        } else {
            None
        };
        if let Some(expanded) = expanded.as_ref() {
            path = expanded.as_ptr();
        }
        if strcmp(path, b"-\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            cmd_source_file_add(&mut *cdata, c"-");
        } else {
            let pattern = if *path as ::core::ffi::c_int == '/' as i32 {
                CStr::from_ptr(path).to_owned()
            } else {
                let cwd_bytes = cwd.to_bytes();
                let path_bytes = CStr::from_ptr(path).to_bytes();
                let mut bytes = Vec::with_capacity(cwd_bytes.len() + 1 + path_bytes.len());
                bytes.extend_from_slice(cwd_bytes);
                bytes.push(b'/');
                bytes.extend_from_slice(path_bytes);
                CString::new(bytes).expect("C strings have no interior NUL")
            };
            log_debug(
                b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                b"cmd_source_file_exec\0" as *const u8 as *const ::core::ffi::c_char,
                pattern.as_ptr(),
            );
            let (matches, result) = GlobResult::run(pattern.as_c_str());
            if result != 0 as ::core::ffi::c_int {
                if result != GLOB_NOMATCH || !(*cdata).flags & CMD_PARSE_QUIET != 0 {
                    if result == GLOB_NOMATCH {
                        error = strerror(ENOENT);
                    } else if result == GLOB_NOSPACE {
                        error = strerror(ENOMEM);
                    } else {
                        error = strerror(EINVAL);
                    }
                    cmdq_error(
                        item,
                        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                        error,
                        path,
                    );
                    retval = CMD_RETURN_ERROR;
                }
                drop(pattern);
            } else {
                drop(pattern);
                j = 0 as u_int;
                while (j as __size_t) < matches.len() {
                    cmd_source_file_add(&mut *cdata, CStr::from_ptr(matches.path(j as usize)));
                    j = j.wrapping_add(1);
                }
            }
        }
        i = i.wrapping_add(1);
    }
    (*cdata).after = item;
    (*cdata).retval = retval;
    if !(*cdata).files.is_empty() {
        let first_path = (&(*cdata).files)[0].as_ptr();
        file_read_with_cmdq_wait(
            c,
            first_path,
            Some(Box::new(move |event| unsafe {
                cmd_source_file_done(
                    cdata,
                    event.path,
                    event.error,
                    event.closed as ::core::ffi::c_int,
                    event
                        .buffer
                        .map_or(::core::ptr::null_mut(), |buffer| buffer.as_ptr()),
                )
            })),
            item,
            Some(Box::new(move || unsafe {
                cmd_source_file_cancel_complete_typed(cdata)
            })),
        );
        retval = CMD_RETURN_WAIT;
    } else {
        cmd_source_file_complete(cdata);
    }
    drop(cwd);
    return retval;
}
