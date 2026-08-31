use crate::arguments::{args_count, args_has, args_string};
use crate::cfg::cfg_print_causes;
use crate::cfg::{cfg_finished, load_cfg_from_buffer};
use crate::cmd::queue::{
    CmdqItemWeak, cmdq_continue, cmdq_error, cmdq_get_callback1, cmdq_get_client, cmdq_get_target,
    cmdq_insert_after, cmdq_item_weak_from_ptr,
};
use crate::cmd::{cmd_get_args, cmd_get_parse_flags};
use crate::ffi::{__ctype_b_loc, glob, globfree, strcmp, strerror};
use crate::file::file_read;
use crate::fmt_args;
use crate::format::format_single_from_target;
use crate::log::log_debug;
use crate::server::server_client_get_cwd;
pub use crate::types::*;
use crate::xmalloc::xasprintf;
use ::core::ffi::CStr;
use ::std::ffi::CString;
pub type ctype_mask = ::core::ffi::c_uint;
pub const _ISalnum: ctype_mask = 8;
pub const _ISpunct: ctype_mask = 4;
pub const _IScntrl: ctype_mask = 2;
pub const _ISblank: ctype_mask = 1;
pub const _ISgraph: ctype_mask = 32768;
pub const _ISprint: ctype_mask = 16384;
pub const _ISspace: ctype_mask = 8192;
pub const _ISxdigit: ctype_mask = 4096;
pub const _ISdigit: ctype_mask = 2048;
pub const _ISalpha: ctype_mask = 1024;
pub const _ISlower: ctype_mask = 512;
pub const _ISupper: ctype_mask = 256;
pub const MSG_READ_CANCEL: msgtype = 307;
pub const MSG_WRITE_CLOSE: msgtype = 306;
pub const MSG_WRITE_READY: msgtype = 305;
pub const MSG_WRITE: msgtype = 304;
pub const MSG_WRITE_OPEN: msgtype = 303;
pub const MSG_READ_DONE: msgtype = 302;
pub const MSG_READ: msgtype = 301;
pub const MSG_READ_OPEN: msgtype = 300;
pub const MSG_FLAGS: msgtype = 218;
pub const MSG_EXEC: msgtype = 217;
pub const MSG_WAKEUP: msgtype = 216;
pub const MSG_UNLOCK: msgtype = 215;
pub const MSG_SUSPEND: msgtype = 214;
pub const MSG_OLDSTDOUT: msgtype = 213;
pub const MSG_OLDSTDIN: msgtype = 212;
pub const MSG_OLDSTDERR: msgtype = 211;
pub const MSG_SHUTDOWN: msgtype = 210;
pub const MSG_SHELL: msgtype = 209;
pub const MSG_RESIZE: msgtype = 208;
pub const MSG_READY: msgtype = 207;
pub const MSG_LOCK: msgtype = 206;
pub const MSG_EXITING: msgtype = 205;
pub const MSG_EXITED: msgtype = 204;
pub const MSG_EXIT: msgtype = 203;
pub const MSG_DETACHKILL: msgtype = 202;
pub const MSG_DETACH: msgtype = 201;
pub const MSG_COMMAND: msgtype = 200;
pub const MSG_IDENTIFY_TERMINFO: msgtype = 112;
pub const MSG_IDENTIFY_LONGFLAGS: msgtype = 111;
pub const MSG_IDENTIFY_STDOUT: msgtype = 110;
pub const MSG_IDENTIFY_FEATURES: msgtype = 109;
pub const MSG_IDENTIFY_CWD: msgtype = 108;
pub const MSG_IDENTIFY_CLIENTPID: msgtype = 107;
pub const MSG_IDENTIFY_DONE: msgtype = 106;
pub const MSG_IDENTIFY_ENVIRON: msgtype = 105;
pub const MSG_IDENTIFY_STDIN: msgtype = 104;
pub const MSG_IDENTIFY_OLDCWD: msgtype = 103;
pub const MSG_IDENTIFY_TTYNAME: msgtype = 102;
pub const MSG_IDENTIFY_TERM: msgtype = 101;
pub const MSG_IDENTIFY_FLAGS: msgtype = 100;
pub const MSG_VERSION: msgtype = 12;
pub const PANE_LINES_SPACES: pane_lines = 5;
pub const PANE_LINES_NUMBER: pane_lines = 4;
pub const PANE_LINES_SIMPLE: pane_lines = 3;
pub const PANE_LINES_HEAVY: pane_lines = 2;
pub const PANE_LINES_DOUBLE: pane_lines = 1;
pub const PANE_LINES_SINGLE: pane_lines = 0;
pub const PROGRESS_BAR_PAUSED: progress_bar_state = 4;
pub const PROGRESS_BAR_INDETERMINATE: progress_bar_state = 3;
pub const PROGRESS_BAR_ERROR: progress_bar_state = 2;
pub const PROGRESS_BAR_NORMAL: progress_bar_state = 1;
pub const PROGRESS_BAR_HIDDEN: progress_bar_state = 0;
pub const SCREEN_CURSOR_BAR: screen_cursor_style = 3;
pub const SCREEN_CURSOR_UNDERLINE: screen_cursor_style = 2;
pub const SCREEN_CURSOR_BLOCK: screen_cursor_style = 1;
pub const SCREEN_CURSOR_DEFAULT: screen_cursor_style = 0;
pub const STYLE_DEFAULT_SET: style_default_type = 3;
pub const STYLE_DEFAULT_POP: style_default_type = 2;
pub const STYLE_DEFAULT_PUSH: style_default_type = 1;
pub const STYLE_DEFAULT_BASE: style_default_type = 0;
pub const STYLE_RANGE_CONTROL: style_range_type = 7;
pub const STYLE_RANGE_USER: style_range_type = 6;
pub const STYLE_RANGE_SESSION: style_range_type = 5;
pub const STYLE_RANGE_WINDOW: style_range_type = 4;
pub const STYLE_RANGE_PANE: style_range_type = 3;
pub const STYLE_RANGE_RIGHT: style_range_type = 2;
pub const STYLE_RANGE_LEFT: style_range_type = 1;
pub const STYLE_RANGE_NONE: style_range_type = 0;
pub const STYLE_LIST_RIGHT_MARKER: style_list = 4;
pub const STYLE_LIST_LEFT_MARKER: style_list = 3;
pub const STYLE_LIST_FOCUS: style_list = 2;
pub const STYLE_LIST_ON: style_list = 1;
pub const STYLE_LIST_OFF: style_list = 0;
pub const STYLE_ALIGN_ABSOLUTE_CENTRE: style_align = 4;
pub const STYLE_ALIGN_RIGHT: style_align = 3;
pub const STYLE_ALIGN_CENTRE: style_align = 2;
pub const STYLE_ALIGN_LEFT: style_align = 1;
pub const STYLE_ALIGN_DEFAULT: style_align = 0;
pub const THEME_DARK: client_theme = 2;
pub const THEME_LIGHT: client_theme = 1;
pub const THEME_UNKNOWN: client_theme = 0;
pub const LAYOUT_WINDOWPANE: layout_type = 2;
pub const LAYOUT_TOPBOTTOM: layout_type = 1;
pub const LAYOUT_LEFTRIGHT: layout_type = 0;
pub const PROMPT_TYPE_INVALID: prompt_type = 255;
pub const PROMPT_TYPE_WINDOW_TARGET: prompt_type = 3;
pub const PROMPT_TYPE_TARGET: prompt_type = 2;
pub const PROMPT_TYPE_SEARCH: prompt_type = 1;
pub const PROMPT_TYPE_COMMAND: prompt_type = 0;
pub const PROMPT_COMMAND: client_prompt_mode = 1;
pub const PROMPT_ENTRY: client_prompt_mode = 0;
pub const CLIENT_EXIT_DETACH: client_exit_type = 2;
pub const CLIENT_EXIT_SHUTDOWN: client_exit_type = 1;
pub const CLIENT_EXIT_RETURN: client_exit_type = 0;
pub const ARGS_PARSE_COMMANDS: args_parse_type = 3;
pub const ARGS_PARSE_COMMANDS_OR_STRING: args_parse_type = 2;
pub const ARGS_PARSE_STRING: args_parse_type = 1;
pub const ARGS_PARSE_INVALID: args_parse_type = 0;
pub const CMD_FIND_SESSION: cmd_find_type = 2;
pub const CMD_FIND_WINDOW: cmd_find_type = 1;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
#[derive(Clone)]
#[repr(C)]
pub struct cmd_source_file_data {
    pub(crate) item: Option<CmdqItemWeak>,
    pub flags: ::core::ffi::c_int,
    pub(crate) after: Option<CmdqItemWeak>,
    pub retval: cmd_retval,
    pub current: u_int,
    pub files: Vec<::std::ffi::CString>,
}
/// The item a stored handle names, or null once its queue has given it up.
fn held_item(held: &Option<CmdqItemWeak>) -> *mut cmdq_item {
    held.as_ref()
        .and_then(CmdqItemWeak::upgrade)
        .map_or(::core::ptr::null_mut(), |item| item.as_ptr())
}

pub const ENOENT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ENOMEM: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const EINVAL: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const GLOB_NOSPACE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const GLOB_NOMATCH: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const CMD_FIND_CANFAIL: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CMD_PARSE_QUIET: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMD_PARSE_PARSEONLY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CMD_PARSE_VERBOSE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const CMD_SOURCE_FILE_DEPTH_LIMIT: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
static mut cmd_source_file_depth: u_int = 0;
pub(crate) static cmd_source_file_entry: cmd_entry = {
    cmd_entry {
        name: c"source-file",
        alias: Some(c"source"),
        args: args_parse_t {
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
        exec: cmd_source_file_exec,
    }
};
unsafe fn cmd_source_file_complete_cb(
    mut item: *mut cmdq_item,
    _data: CmdqCallbackData,
) -> cmd_retval {
    unsafe {
        let mut c: *mut client = cmdq_get_client(&*item);
        if c.is_null() {
            cmd_source_file_depth = cmd_source_file_depth.wrapping_sub(1);
            log_debug(
                c"%s: depth now %u".as_ptr(),
                fmt_args![
                    c"cmd_source_file_complete_cb".as_ptr(),
                    cmd_source_file_depth
                ],
            );
        } else {
            (*c).source_file_depth = (*c).source_file_depth.wrapping_sub(1);
            log_debug(
                c"%s: depth now %u".as_ptr(),
                fmt_args![
                    c"cmd_source_file_complete_cb".as_ptr(),
                    (*c).source_file_depth
                ],
            );
        }
        cfg_print_causes(item);
        CMD_RETURN_NORMAL
    }
}
unsafe fn cmd_source_file_complete(mut c: *mut client, cdata: &SourceFileRef) {
    unsafe {
        let cdata = &*cdata.as_ptr();
        if cfg_finished != 0 {
            if cdata.retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int
                && !c.is_null()
                && (*c).session.is_null()
            {
                (*c).retval = 1 as ::core::ffi::c_int;
            }
            if let Some(after) = cdata.after.as_ref().and_then(CmdqItemWeak::upgrade) {
                cmdq_insert_after(
                    &after,
                    cmdq_get_callback1(
                        c"cmd_source_file_complete_cb".as_ptr(),
                        Some(cmd_source_file_complete_cb),
                        CmdqCallbackData::None,
                    ),
                );
            }
        }
    }
}
unsafe fn cmd_source_file_done(
    mut c: *mut client,
    mut path: *const ::core::ffi::c_char,
    mut error: ::core::ffi::c_int,
    mut closed: ::core::ffi::c_int,
    mut buffer: *mut Buf,
    mut data: ClientFileData,
) {
    unsafe {
        let cdata_ref = match data {
            ClientFileData::SourceFile(cdata) => cdata,
            _ => panic!("source-file callback data is not source-file data"),
        };
        let cdata = &mut *cdata_ref.as_ptr();
        let item_ref = cdata
            .item
            .as_ref()
            .and_then(CmdqItemWeak::upgrade)
            .expect("the item that asked for the file is waiting on it");
        let item = item_ref.as_ptr();
        let mut n: u_int = 0;
        let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
        let mut target: *mut cmd_find_state = cmdq_get_target(item);
        if closed == 0 {
            return;
        }
        let bytes = if buffer.is_null() {
            Vec::new()
        } else {
            (*buffer).as_slice().to_vec()
        };
        let bdata = bytes.as_ptr();
        let bsize = bytes.len();
        if error != 0 as ::core::ffi::c_int {
            cmdq_error(item, c"%s: %s".as_ptr(), fmt_args![strerror(error), path]);
        } else if bsize != 0 as size_t {
            if load_cfg_from_buffer(
                bdata as *const ::core::ffi::c_char,
                bsize,
                path,
                c,
                held_item(&cdata.after),
                target,
                cdata.flags,
                Some(&mut new_item),
            ) < 0 as ::core::ffi::c_int
            {
                cdata.retval = CMD_RETURN_ERROR;
            } else if !new_item.is_null() {
                cdata.after = cmdq_item_weak_from_ptr(new_item);
            }
        }
        cdata.current = cdata.current.wrapping_add(1);
        n = cdata.current;
        if (n as usize) < cdata.files.len() {
            file_read(
                c,
                cdata.files[n as usize].as_ptr(),
                Some(cmd_source_file_done),
                ClientFileData::SourceFile(cdata_ref.clone()),
            );
        } else {
            cmd_source_file_complete(c, &cdata_ref);
            cmdq_continue(&item_ref);
        };
    }
}
unsafe fn cmd_source_file_add(cdata: &SourceFileRef, mut path: *const ::core::ffi::c_char) {
    unsafe {
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"cmd_source_file_add".as_ptr(), path],
        );
        (*cdata.as_ptr()).files.push(CStr::from_ptr(path).to_owned());
    }
}
/// `path` with every byte a glob would take specially backslash-escaped, so
/// that a working directory holding one is still joined onto the pattern as
/// a plain prefix. Only ASCII alphanumerics and `/` are left alone.
fn cmd_source_file_quote_for_glob(path: &CStr) -> CString {
    let mut quoted: Vec<u8> = Vec::new();
    for &c in path.to_bytes() {
        unsafe {
            if c < 128
                && *(*__ctype_b_loc()).offset(c as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    == 0
                && c != b'/'
            {
                quoted.push(b'\\');
            }
        }
        quoted.push(c);
    }
    unsafe { CString::from_vec_unchecked(quoted) }
}
unsafe fn cmd_source_file_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args: &args = cmd_get_args(self_0);
        let mut c: *mut client = cmdq_get_client(&*item);
        let mut retval: cmd_retval = CMD_RETURN_NORMAL;
        let mut pattern = CString::default();
        let mut expanded: Option<CString> = None;
        let mut path: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut g = glob_t::default();
        let mut result: ::core::ffi::c_int = 0;
        let mut parse_flags: ::core::ffi::c_int = 0;
        let mut i: u_int = 0;
        let mut j: u_int = 0;
        if c.is_null() {
            if cmd_source_file_depth >= CMD_SOURCE_FILE_DEPTH_LIMIT as u_int {
                cmdq_error(item, c"too many nested files".as_ptr(), fmt_args![]);
                return CMD_RETURN_ERROR;
            }
            cmd_source_file_depth = cmd_source_file_depth.wrapping_add(1);
            log_debug(
                c"%s: depth now %u".as_ptr(),
                fmt_args![c"cmd_source_file_exec".as_ptr(), cmd_source_file_depth],
            );
        } else {
            if (*c).source_file_depth >= CMD_SOURCE_FILE_DEPTH_LIMIT as u_int {
                cmdq_error(item, c"too many nested files".as_ptr(), fmt_args![]);
                return CMD_RETURN_ERROR;
            }
            (*c).source_file_depth = (*c).source_file_depth.wrapping_add(1);
            log_debug(
                c"%s: depth now %u".as_ptr(),
                fmt_args![c"cmd_source_file_exec".as_ptr(), (*c).source_file_depth],
            );
        }
        let mut flags: ::core::ffi::c_int = 0;
        if args_has(args, 'q' as i32 as u_char) != 0 {
            flags |= CMD_PARSE_QUIET;
        }
        if args_has(args, 'n' as i32 as u_char) != 0 {
            flags |= CMD_PARSE_PARSEONLY;
        }
        if c.is_null() || !(*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            parse_flags = cmd_get_parse_flags(self_0);
            if args_has(args, 'v' as i32 as u_char) != 0 || parse_flags & CMD_PARSE_VERBOSE != 0 {
                flags |= CMD_PARSE_VERBOSE;
            }
        }
        let cdata = SourceFileRef::new(cmd_source_file_data {
            item: cmdq_item_weak_from_ptr(item),
            flags,
            after: cmdq_item_weak_from_ptr(item),
            retval,
            current: 0,
            files: Vec::new(),
        });
        let cwd = cmd_source_file_quote_for_glob(CStr::from_ptr(server_client_get_cwd(
            c,
            ::core::ptr::null_mut::<session>(),
        )));
        i = 0 as u_int;
        while i < args_count(args) {
            path = args_string(args, i);
            if args_has(args, 'F' as i32 as u_char) != 0 {
                expanded = Some(format_single_from_target(item, CStr::from_ptr(path)));
                path = expanded.as_ref().expect("just expanded").as_ptr();
            }
            if strcmp(path, c"-".as_ptr()) == 0 as ::core::ffi::c_int {
                cmd_source_file_add(&cdata, c"-".as_ptr());
            } else {
                if *path as ::core::ffi::c_int == '/' as i32 {
                    pattern = CStr::from_ptr(path).to_owned();
                } else {
                    pattern = xasprintf(c"%s/%s".as_ptr(), fmt_args![cwd.as_ptr(), path])
                }
                log_debug(
                    c"%s: %s".as_ptr(),
                    fmt_args![c"cmd_source_file_exec".as_ptr(), pattern.as_ptr()],
                );
                result = glob(pattern.as_ptr(), 0 as ::core::ffi::c_int, None, &raw mut g);
                if result != 0 as ::core::ffi::c_int {
                    if result != GLOB_NOMATCH || !flags & CMD_PARSE_QUIET != 0 {
                        if result == GLOB_NOMATCH {
                            error = strerror(ENOENT);
                        } else if result == GLOB_NOSPACE {
                            error = strerror(ENOMEM);
                        } else {
                            error = strerror(EINVAL);
                        }
                        cmdq_error(item, c"%s: %s".as_ptr(), fmt_args![error, path]);
                        retval = CMD_RETURN_ERROR;
                    }
                    globfree(&raw mut g);
                } else {
                    j = 0 as u_int;
                    while (j as __size_t) < g.gl_pathc {
                        cmd_source_file_add(&cdata, *g.gl_pathv.offset(j as isize));
                        j = j.wrapping_add(1);
                    }
                    globfree(&raw mut g);
                }
            }
            i = i.wrapping_add(1);
        }
        (*cdata.as_ptr()).after = cmdq_item_weak_from_ptr(item);
        (*cdata.as_ptr()).retval = retval;
        let first = (*cdata.as_ptr()).files.first().map(|path| path.as_ptr());
        if let Some(first) = first {
            file_read(
                c,
                first,
                Some(cmd_source_file_done),
                ClientFileData::SourceFile(cdata.clone()),
            );
            retval = CMD_RETURN_WAIT;
        } else {
            cmd_source_file_complete(c, &cdata);
        }
        retval
    }
}
