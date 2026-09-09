use crate::GlobPaths;
use crate::args::RustArguments;
use crate::args::{args_has, args_string_str};
use crate::cfg::cfg_print_causes;
use crate::cfg::{configuration_finished, load_cfg_buffer_for_client};
use crate::cmd::cmdq_item;
use crate::cmd::{CmdqItemRef, CmdqItemWeak, cmdq_item_weak_of};
use crate::cmd::{RustCommandEntry, SourceFileRef, cmd, cmd_entry_flag, cmd_retval};
use crate::cmd::{cmd_get_args, cmd_get_parse_flags};
use crate::compat::error_message;
use crate::consts::{
    CLIENT_CONTROL, CMD_FIND_CANFAIL, CMD_FIND_PANE, CMD_PARSE_PARSEONLY, CMD_PARSE_QUIET,
    CMD_PARSE_VERBOSE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_WAIT, EINVAL, ENOENT,
    ENOMEM,
};
use crate::file::file_read_for_client;
use crate::fmt_args;
use crate::format::format_single_from_target;
use crate::log::log_debug;
use crate::server::client_working_directory;
use crate::types::ClientFileEvent;
use crate::types::{ClientFileData, ClientRef, args_parse_t, size_t, u_char, u_int, uint64_t};
use crate::xmalloc::xasprintf;
use ::core::ffi::CStr;
use ::std::ffi::CString;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone)]
#[repr(C)]
pub struct cmd_source_file_data {
    pub(crate) item: Option<CmdqItemWeak>,
    pub flags: core::ffi::c_int,
    pub(crate) after: Option<CmdqItemWeak>,
    pub retval: cmd_retval,
    pub current: u_int,
    pub files: Vec<Rc<CStr>>,
}
/// The live item a stored handle names, if its queue still holds it.
fn held_item(held: &Option<CmdqItemWeak>) -> Option<CmdqItemRef> {
    held.as_ref().and_then(CmdqItemWeak::upgrade)
}

pub const GLOB_NOSPACE: core::ffi::c_int = 1 as core::ffi::c_int;
pub const GLOB_NOMATCH: core::ffi::c_int = 3 as core::ffi::c_int;

pub const CMD_SOURCE_FILE_DEPTH_LIMIT: core::ffi::c_int = 50 as core::ffi::c_int;
static CMD_SOURCE_FILE_DEPTH: AtomicU32 = AtomicU32::new(0);
pub(crate) static cmd_source_file_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"source-file",
        alias: Some(c"source"),
        args: args_parse_t {
            template: c"t:Fnqv",
            lower: 1 as core::ffi::c_int,
            upper: -(1 as core::ffi::c_int),
            cb: None,
        },
        usage: c"[-Fnqv] [-t target-pane] path ...",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: CMD_FIND_CANFAIL,
        },
        flags: 0 as core::ffi::c_int,
        exec: cmd_source_file_exec,
    }
};
fn cmd_source_file_complete_cb(item: &CmdqItemRef) -> cmd_retval {
    unsafe {
        let item = item.read();
        let mut c = item.client();
        if let Some(c) = c.as_mut() {
            c.leave_source_file();
            log_debug(
                c"%s: depth now %u",
                fmt_args![c"cmd_source_file_complete_cb", c.source_file_depth()],
            );
        } else {
            let depth = CMD_SOURCE_FILE_DEPTH
                .fetch_sub(1, Ordering::Relaxed)
                .wrapping_sub(1);
            log_debug(
                c"%s: depth now %u",
                fmt_args![c"cmd_source_file_complete_cb", depth],
            );
        }
        cfg_print_causes(&item);
        CMD_RETURN_NORMAL
    }
}

fn cmd_source_file_done(event: ClientFileEvent<'_>) {
    unsafe {
        let ClientFileEvent::Done {
            mut client,
            path,
            error,
            mut buffer,
            data,
        } = event
        else {
            return;
        };
        let cdata_ref = match data {
            ClientFileData::SourceFile(cdata) => cdata,
            _ => panic!("source-file callback data is not source-file data"),
        };
        let item_ref = cdata_ref.with(|cdata| {
            cdata
                .item
                .as_ref()
                .and_then(CmdqItemWeak::upgrade)
                .expect("the item that asked for the file is waiting on it")
        });
        let mut new_item = None;
        let bytes = buffer.as_slice().to_vec();
        let bsize = bytes.len();
        if error != 0 as core::ffi::c_int {
            item_ref.with_item(|item| {
                item.error(
                    c"%s: %s",
                    fmt_args![error_message(error).as_c_str(), path.as_c_str()],
                );
            });
        } else if bsize != 0 as size_t {
            let target = item_ref.with_item(|item| item.target.clone());
            let (after, flags) = cdata_ref.with(|cdata| (held_item(&cdata.after), cdata.flags));
            if load_cfg_buffer_for_client(
                &bytes,
                &path,
                client.as_ref(),
                after.as_ref(),
                Some(&target),
                flags,
                Some(&mut new_item),
            ) < 0 as core::ffi::c_int
            {
                cdata_ref.with_mut(|cdata| cdata.retval = CMD_RETURN_ERROR);
            } else if let Some(new_item) = new_item {
                cdata_ref.with_mut(|cdata| cdata.after = Some(new_item.downgrade()));
            }
        }
        let next = cdata_ref.with_mut(|cdata| {
            cdata.current = cdata.current.wrapping_add(1);
            cdata.files.get(cdata.current as usize).cloned()
        });
        if let Some(next) = next {
            file_read_for_client(
                client.as_mut(),
                &next,
                Some(std::rc::Rc::new(cmd_source_file_done)),
                ClientFileData::SourceFile(cdata_ref.clone()),
            );
        } else {
            cdata_ref.complete(client.as_mut());
            item_ref.resume();
        };
    }
}

/// `path` with every byte a glob would take specially backslash-escaped, so
/// that a working directory holding one is still joined onto the pattern as
/// a plain prefix. Only ASCII alphanumerics and `/` are left alone.
fn cmd_source_file_quote_for_glob(path: &CStr) -> CString {
    let mut quoted: Vec<u8> = Vec::new();
    for &c in path.to_bytes() {
        unsafe {
            if c < 128 && libc::isalnum(c.into()) == 0 && c != b'/' {
                quoted.push(b'\\');
            }
        }
        quoted.push(c);
    }
    CString::new(quoted).expect("glob quoting does not introduce NUL")
}
unsafe fn cmd_source_file_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &RustArguments = cmd_get_args(self_0);
    let mut c = item.client();
    let mut retval: cmd_retval = CMD_RETURN_NORMAL;
    let parse_flags: core::ffi::c_int;
    if let Some(c) = c.as_mut() {
        if c.source_file_depth() >= CMD_SOURCE_FILE_DEPTH_LIMIT as u_int {
            unsafe { item.error(c"too many nested files", fmt_args![]) };
            return CMD_RETURN_ERROR;
        }
        c.enter_source_file();
        unsafe {
            log_debug(
                c"%s: depth now %u",
                fmt_args![c"cmd_source_file_exec", c.source_file_depth()],
            )
        };
    } else {
        let Ok(previous) =
            CMD_SOURCE_FILE_DEPTH.try_update(Ordering::Relaxed, Ordering::Relaxed, |depth| {
                (depth < CMD_SOURCE_FILE_DEPTH_LIMIT as u_int).then(|| depth + 1)
            })
        else {
            unsafe { item.error(c"too many nested files", fmt_args![]) };
            return CMD_RETURN_ERROR;
        };
        unsafe {
            log_debug(
                c"%s: depth now %u",
                fmt_args![c"cmd_source_file_exec", previous + 1],
            )
        };
    }
    let mut flags: core::ffi::c_int = 0;
    if args_has(args, 'q' as i32 as u_char) != 0 {
        flags |= CMD_PARSE_QUIET;
    }
    if args_has(args, 'n' as i32 as u_char) != 0 {
        flags |= CMD_PARSE_PARSEONLY;
    }
    if unsafe {
        c.as_ref()
            .is_none_or(|c| !c.flags() & CLIENT_CONTROL as uint64_t != 0)
    } {
        parse_flags = cmd_get_parse_flags(self_0);
        if args_has(args, 'v' as i32 as u_char) != 0 || parse_flags & CMD_PARSE_VERBOSE != 0 {
            flags |= CMD_PARSE_VERBOSE;
        }
    }
    let cdata = SourceFileRef::new(cmd_source_file_data {
        item: cmdq_item_weak_of(item),
        flags,
        after: cmdq_item_weak_of(item),
        retval,
        current: 0,
        files: Vec::new(),
    });
    let cwd = unsafe {
        cmd_source_file_quote_for_glob(client_working_directory(c.as_ref(), None).as_c_str())
    };
    for i in 0..args.argument_count() {
        let argument = unsafe { args_string_str(args, i).expect("argument index checked") };
        let expanded = (args_has(args, b'F') != 0)
            .then(|| unsafe { format_single_from_target(item, argument) });
        let path = expanded.as_deref().unwrap_or(argument);
        if path == c"-" {
            unsafe { cdata.add(Rc::from(path)) };
            continue;
        }
        let pattern = if path.to_bytes().starts_with(b"/") {
            path.to_owned()
        } else {
            xasprintf(c"%s/%s", fmt_args![cwd.as_c_str(), path])
        };
        unsafe {
            log_debug(
                c"%s: %s",
                fmt_args![c"cmd_source_file_exec", pattern.as_c_str()],
            )
        };
        match GlobPaths::expand(&pattern) {
            Ok(paths) => {
                for path in paths.into_paths() {
                    unsafe { cdata.add(path) };
                }
            }
            Err(result) if result != GLOB_NOMATCH || flags & CMD_PARSE_QUIET == 0 => {
                let error = match result {
                    GLOB_NOMATCH => ENOENT,
                    GLOB_NOSPACE => ENOMEM,
                    _ => EINVAL,
                };
                unsafe { item.error(c"%s: %s", fmt_args![error_message(error).as_c_str(), path]) };
                retval = CMD_RETURN_ERROR;
            }
            Err(_) => {}
        }
    }
    let first = cdata.with_mut(|cdata| {
        cdata.after = cmdq_item_weak_of(&*item);
        cdata.retval = retval;
        cdata.files.first().cloned()
    });
    if let Some(first) = first {
        unsafe {
            file_read_for_client(
                c.as_mut(),
                &first,
                Some(std::rc::Rc::new(cmd_source_file_done)),
                ClientFileData::SourceFile(cdata.clone()),
            )
        };
        retval = CMD_RETURN_WAIT;
    } else {
        unsafe { cdata.complete(c.as_mut()) };
    }
    retval
}

impl SourceFileRef {
    unsafe fn complete(&self, c: Option<&mut ClientRef>) {
        let cdata = self;

        unsafe {
            let (retval, after) = cdata.with(|cdata| {
                (
                    cdata.retval,
                    cdata.after.as_ref().and_then(CmdqItemWeak::upgrade),
                )
            });
            if configuration_finished() {
                if retval as core::ffi::c_int == CMD_RETURN_ERROR as core::ffi::c_int
                    && c.as_ref().is_some_and(|c| c.attached_session().is_none())
                    && let Some(c) = c
                {
                    c.set_return_code(1);
                }
                if let Some(after) = after {
                    after.insert_after(CmdqItemRef::callback_items(
                        c"cmd_source_file_complete_cb",
                        cmd_source_file_complete_cb,
                    ));
                }
            }
        }
    }
    unsafe fn add(&self, path: Rc<CStr>) {
        let cdata = self;

        unsafe {
            log_debug(c"%s: %s", fmt_args![c"cmd_source_file_add", path.as_ref()]);
            cdata.with_mut(|cdata| cdata.files.push(path));
        }
    }
}
