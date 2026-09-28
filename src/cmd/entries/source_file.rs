use crate::src::arguments::{args_count, args_has, args_string};
use crate::src::cfg::{cfg_finished, cfg_print_causes, load_cfg_from_buffer};
use crate::src::cmd::queue::{
    cmdq_continue, cmdq_error, cmdq_get_callback_owned, cmdq_get_client, cmdq_get_target,
    cmdq_insert_after,
};
use crate::src::cmd::{cmd_get_args_mut, cmd_get_parse_flags};
use crate::src::compat::glob::GlobResult;
use crate::src::ffi::libc::{__ctype_b_loc, strcmp, strerror};
use crate::src::file::file_read_with_cmdq_wait;
use crate::src::format::bytes::write_cstr;
use crate::src::format::format_single_from_target_cstring;
use crate::src::log::{log_cstr, log_debug};
use crate::src::reactor::{evbuffer_get_length, evbuffer_pullup};
use crate::src::server_client::{server_client_get_cwd, server_client_unref_owned};
use crate::src::shared::abi::*;
use crate::src::shared::abi::{__size_t, ssize_t};
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::CLIENT_CONTROL;
use crate::src::shared::client::{client, client_file_cb};
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
use std::cell::UnsafeCell;
use std::ffi::{CStr, CString};
use std::rc::{Rc, Weak};

pub struct cmd_source_file_data {
    pub item: Weak<UnsafeCell<cmdq_item>>,
    pub client: Option<Rc<UnsafeCell<client>>>,
    depth_active: bool,
    pub flags: ::core::ffi::c_int,
    pub after: Weak<UnsafeCell<cmdq_item>>,
    pub retval: cmd_retval,
    pub current: u_int,
    pub files: Vec<CString>,
}

pub const GLOB_NOSPACE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const GLOB_NOMATCH: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
static mut cmd_source_file_depth: u_int = 0;
pub static cmd_source_file_entry: cmd_entry = {
    cmd_entry {
        name: c"source-file",
        alias: Some(c"source"),
        args: args_parse {
            template: c"t:Fnqv",
            lower: 1 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: c"[-Fnqv] [-t target-pane] path ...",
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
        exec: Some(cmd_source_file_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
impl cmd_source_file_data {
    fn client_ptr(&self) -> *mut client {
        self.client
            .as_ref()
            .map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr)
    }

    fn decrement_depth(&mut self) {
        if !std::mem::replace(&mut self.depth_active, false) {
            return;
        }
        unsafe {
            let depth = if let Some(client) = &self.client {
                &mut (*client.get()).source_file_depth
            } else {
                &mut *(&raw mut cmd_source_file_depth)
            };
            *depth = depth.wrapping_sub(1);
            log_debug(format_args!(
                "{}: depth now {}",
                "cmd_source_file_complete_cb", *depth
            ));
        }
    }

    fn into_read_callback(self: Box<Self>) -> client_file_cb {
        let mut owner = Some(self);
        Some(Box::new(move |event| {
            // Progress leaves the record in this callback. Only the terminal
            // event transfers it to the next read or the completion command.
            if !event.closed {
                return;
            }
            let owner = owner.take().expect("one terminal source-file callback");
            unsafe {
                cmd_source_file_done(
                    owner,
                    event.path,
                    event.error,
                    event.buffer.expect("read callback buffer"),
                );
            }
        }))
    }
}

impl Drop for cmd_source_file_data {
    fn drop(&mut self) {
        self.decrement_depth();
        // Preserve cleanup order: depth, path copies, then deferred client release.
        drop(std::mem::take(&mut self.files));
        if let Some(client) = self.client.take() {
            server_client_unref_owned(client);
        }
    }
}

unsafe fn cmd_source_file_complete_cb(
    item: *mut cmdq_item,
    mut cdata: Box<cmd_source_file_data>,
) -> cmd_retval {
    cdata.decrement_depth();
    cfg_print_causes(item);
    CMD_RETURN_NORMAL
}

unsafe fn cmd_source_file_complete(mut cdata: Box<cmd_source_file_data>) {
    let c = cdata.client_ptr();
    if cfg_finished == 0 {
        // Startup completion does not decrement nesting depth in tmux.
        // Cancellation during an active read still decrements it.
        cdata.depth_active = false;
        return;
    }
    if cdata.retval == CMD_RETURN_ERROR && !c.is_null() && (*c).session_ptr().is_null() {
        (*c).retval = 1;
    }
    let Some(after_owner) = cdata.after.upgrade().or_else(|| cdata.item.upgrade()) else {
        return;
    };
    let after = after_owner.get();
    let new_item = cmdq_get_callback_owned(
        c"cmd_source_file_complete_cb".as_ptr(),
        Some(Box::new(move |item| unsafe {
            cmd_source_file_complete_cb(item.as_ptr(), cdata)
        })),
    );
    cmdq_insert_after(after, new_item);
}

unsafe fn cmd_source_file_read(cdata: Box<cmd_source_file_data>) {
    let Some(item_owner) = cdata.item.upgrade() else { return };
    let client_owner = cdata.client.clone();
    let item = item_owner.get();
    let path = cdata.files[cdata.current as usize].as_ptr();
    file_read_with_cmdq_wait(client_owner.as_ref(), path, cdata.into_read_callback(), item, None);
}

unsafe fn cmd_source_file_done(
    mut cdata: Box<cmd_source_file_data>,
    path: Option<&CStr>,
    error: ::core::ffi::c_int,
    buffer: &mut evbuffer,
) {
    let Some(item_owner) = cdata.item.upgrade() else { return };
    let item = item_owner.get();
    let path = path.map_or(::core::ptr::null(), CStr::as_ptr);
    // Progress does not coalesce the buffer; only complete files are parsed.
    let bdata = evbuffer_pullup(buffer, -1)
        .map_or(std::ptr::null_mut(), |bytes| bytes.as_mut_ptr())
        .cast();
    let bsize = evbuffer_get_length(buffer);
    let mut new_item = std::ptr::null_mut();
    let target = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    if error != 0 {
        cmdq_error(item, |out| {
            write_cstr(out, strerror(error))?;
            out.write_all(b": ")?;
            write_cstr(out, path)
        });
    } else if bsize != 0 {
        let after_owner = cdata.after.upgrade().unwrap_or_else(|| Rc::clone(&item_owner));
        if load_cfg_from_buffer(
            bdata,
            bsize,
            path,
            cdata.client.as_ref(),
            after_owner.get(),
            target,
            cdata.flags,
            &mut new_item,
        ) < 0
        {
            cdata.retval = CMD_RETURN_ERROR;
        } else if !new_item.is_null() {
            cdata.after = (*new_item).observer.clone();
        }
    }
    cdata.current = cdata.current.wrapping_add(1);
    if (cdata.current as usize) < cdata.files.len() {
        cmd_source_file_read(cdata);
    } else {
        cmd_source_file_complete(cdata);
        cmdq_continue(item);
    }
}
unsafe fn cmd_source_file_add(cdata: &mut cmd_source_file_data, path: &CStr) {
    log_debug(format_args!(
        "{}: {}",
        "cmd_source_file_add",
        log_cstr((path.as_ptr()) as *const _)
    ));
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
unsafe fn cmd_source_file_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let c_owner = cmdq_get_client(item);
    let mut c: *mut client = c_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut retval: cmd_retval = CMD_RETURN_NORMAL;
    let mut path: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut parse_flags: ::core::ffi::c_int = 0;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    if c.is_null() {
        if cmd_source_file_depth >= CMD_SOURCE_FILE_DEPTH_LIMIT as u_int {
            cmdq_error(item, |out| out.write_all(b"too many nested files"));
            return CMD_RETURN_ERROR;
        }
        cmd_source_file_depth = cmd_source_file_depth.wrapping_add(1);
        log_debug(format_args!(
            "{}: depth now {}",
            "cmd_source_file_exec",
            (cmd_source_file_depth) as u32
        ));
    } else {
        if (*c).source_file_depth >= CMD_SOURCE_FILE_DEPTH_LIMIT as u_int {
            cmdq_error(item, |out| out.write_all(b"too many nested files"));
            return CMD_RETURN_ERROR;
        }
        (*c).source_file_depth = (*c).source_file_depth.wrapping_add(1);
        log_debug(format_args!(
            "{}: depth now {}",
            "cmd_source_file_exec",
            ((*c).source_file_depth) as u32
        ));
    }
    let mut cdata = Box::new(cmd_source_file_data {
        item: (*item).observer.clone(),
        client: if c.is_null() {
            None
        } else {
            Some(
                (*c).observer
                    .upgrade()
                    .expect("live source-file client"),
            )
        },
        depth_active: true,
        flags: 0,
        after: Weak::new(),
        retval: CMD_RETURN_NORMAL,
        current: 0,
        files: Vec::new(),
    });
    if args_has(args, 'q' as i32 as u_char) != 0 {
        cdata.flags |= CMD_PARSE_QUIET;
    }
    if args_has(args, 'n' as i32 as u_char) != 0 {
        cdata.flags |= CMD_PARSE_PARSEONLY;
    }
    if c.is_null() || !(*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        parse_flags = cmd_get_parse_flags(self_0);
        if args_has(args, 'v' as i32 as u_char) != 0 || parse_flags & CMD_PARSE_VERBOSE != 0 {
            cdata.flags |= CMD_PARSE_VERBOSE;
        }
    }
    let cwd_owner = server_client_get_cwd(c.as_ref(), None).expect("source working directory");
    let cwd = cmd_source_file_quote_for_glob(&cwd_owner);
    i = 0 as u_int;
    while i < args_count(args) {
        path = args_string(&mut *(args), i).map_or(std::ptr::null(), |value| value.as_ptr());
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
            cmd_source_file_add(&mut cdata, c"-");
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
            log_debug(format_args!(
                "{}: {}",
                "cmd_source_file_exec",
                log_cstr((pattern.as_ptr()) as *const _)
            ));
            let (matches, result) = GlobResult::run(pattern.as_c_str());
            if result != 0 as ::core::ffi::c_int {
                if result != GLOB_NOMATCH || !cdata.flags & CMD_PARSE_QUIET != 0 {
                    if result == GLOB_NOMATCH {
                        error = strerror(ENOENT);
                    } else if result == GLOB_NOSPACE {
                        error = strerror(ENOMEM);
                    } else {
                        error = strerror(EINVAL);
                    }
                    cmdq_error(item, |out| {
                        write_cstr(out, error)?;
                        out.write_all(b": ")?;
                        write_cstr(out, path)
                    });
                    retval = CMD_RETURN_ERROR;
                }
                drop(pattern);
            } else {
                drop(pattern);
                j = 0 as u_int;
                while (j as __size_t) < matches.len() {
                    cmd_source_file_add(&mut cdata, CStr::from_ptr(matches.path(j as usize)));
                    j = j.wrapping_add(1);
                }
            }
        }
        i = i.wrapping_add(1);
    }
    cdata.after = (*item).observer.clone();
    cdata.retval = retval;
    if !cdata.files.is_empty() {
        cmd_source_file_read(cdata);
        retval = CMD_RETURN_WAIT;
    } else {
        cmd_source_file_complete(cdata);
    }
    drop(cwd);
    return retval;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::src::cmd::queue::{cmdq_free_detached, cmdq_get_callback_owned};
    use crate::src::reactor::{evbuffer_new, shutdown_runtime};
    use crate::src::shared::client::client_file_event;
    use std::ptr::NonNull;

    fn data(
        item: *mut cmdq_item,
        client: Option<Rc<UnsafeCell<client>>>,
    ) -> Box<cmd_source_file_data> {
        Box::new(cmd_source_file_data {
            item: unsafe { (*item).observer.clone() },
            client,
            depth_active: true,
            flags: 0,
            after: unsafe { (*item).observer.clone() },
            retval: CMD_RETURN_NORMAL,
            current: 0,
            files: vec![c"owned.conf".to_owned()],
        })
    }

    #[test]
    fn completion_and_dropped_queue_callbacks_decrement_client_depth_once() {
        unsafe {
            for dispatch in [false, true] {
                let client = client::new();
                (*client.get()).source_file_depth = 1;
                let observer = Rc::downgrade(&client);
                let source = cmdq_get_callback_owned(c"source wait test".as_ptr(), None);
                let owner = data(source, Some(client.clone()));
                let item = cmdq_get_callback_owned(
                    c"source-file-test".as_ptr(),
                    Some(Box::new(move |item| {
                        cmd_source_file_complete_cb(item.as_ptr(), owner)
                    })),
                );
                if dispatch {
                    let callback = (*item).cb.take().unwrap();
                    assert_eq!(callback(NonNull::new(item).unwrap()), CMD_RETURN_NORMAL);
                    assert_eq!((*client.get()).source_file_depth, 0);
                }
                cmdq_free_detached(item);
                cmdq_free_detached(source);
                assert_eq!((*client.get()).source_file_depth, 0);
                drop(client);
                assert!(observer.upgrade().is_some());
                shutdown_runtime();
                assert!(observer.upgrade().is_none());
            }
        }
    }

    #[test]
    fn startup_terminal_callback_preserves_depth_and_resumes_source_command() {
        unsafe {
            let saved = (cfg_finished, cmd_source_file_depth);
            cfg_finished = 0;
            cmd_source_file_depth = 7;
            let item = cmdq_get_callback_owned(c"source wait test".as_ptr(), None);
            (*item).flags = CMDQ_WAITING;
            let mut callback = data(item, None).into_read_callback().unwrap();
            let mut buffer = evbuffer_new();
            callback(client_file_event {
                client: None,
                path: Some(c"owned.conf"),
                error: 0,
                closed: true,
                buffer: Some(&mut buffer),
            });
            drop(callback);
            assert_eq!((*item).flags & CMDQ_WAITING, 0);
            assert_eq!(*(&raw const cmd_source_file_depth), 7);
            cmdq_free_detached(item);
            (cfg_finished, cmd_source_file_depth) = saved;
        }
    }

    #[test]
    fn expired_source_wait_releases_depth_without_reading_the_item() {
        unsafe {
            let client = client::new();
            (*client.get()).source_file_depth = 1;
            let item = cmdq_get_callback_owned(c"expired source wait".as_ptr(), None);
            let cdata = data(item, Some(Rc::clone(&client)));
            cmdq_free_detached(item);
            let mut buffer = evbuffer_new();
            cmd_source_file_done(cdata, Some(c"owned.conf"), 0, &mut buffer);
            assert_eq!((*client.get()).source_file_depth, 0);
        }
    }
}
