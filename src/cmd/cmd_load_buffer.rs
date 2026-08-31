use crate::arguments::{args_get, args_has, args_string};
use crate::cmd::cmd_get_args;
use crate::cmd::queue::{
    CmdqItemWeak, cmdq_continue, cmdq_error, cmdq_get_client, cmdq_get_target_client,
    cmdq_item_weak_from_ptr,
};
use crate::ffi::strerror;
use crate::file::file_read;
use crate::fmt_args;
use crate::format::format_single_from_target;
use crate::paste::paste_set;
use crate::server::client_ref_from_ptr;
use crate::tty::tty_set_selection;
pub use crate::types::*;
use ::core::ffi::CStr;
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
#[derive(Default)]
#[repr(C)]
pub struct cmd_load_buffer_data {
    pub(crate) client_ref: Option<ClientRef>,
    pub(crate) item: Option<CmdqItemWeak>,
    pub name: Option<::std::ffi::CString>,
}

impl cmd_load_buffer_data {
    /// The client the buffer is set for, or null when none was named.
    pub(crate) fn client(&self) -> *mut client {
        self.client_ref
            .as_ref()
            .map_or(::core::ptr::null_mut(), ClientRef::as_ptr)
    }
}
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMD_CLIENT_TFLAG: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CMD_CLIENT_CANFAIL: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CLIENT_DEAD: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub(crate) static cmd_load_buffer_entry: cmd_entry = {
    cmd_entry {
        name: c"load-buffer",
        alias: Some(c"loadb"),
        args: args_parse_t {
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
        exec: cmd_load_buffer_exec,
    }
};
pub(crate) unsafe fn cmd_load_buffer_done(
    _c: *mut client,
    mut path: *const ::core::ffi::c_char,
    mut error: ::core::ffi::c_int,
    mut closed: ::core::ffi::c_int,
    mut buffer: *mut Buf,
    mut data: ClientFileData,
) {
    unsafe {
        let (cdata, _owner): (*mut cmd_load_buffer_data, Option<Box<cmd_load_buffer_data>>) =
            match data {
                ClientFileData::LoadBuffer(cdata) => (
                    cdata.as_ref() as *const cmd_load_buffer_data as *mut cmd_load_buffer_data,
                    Some(cdata),
                ),
                ClientFileData::LoadBufferView(cdata) => (cdata, None),
                _ => panic!("load-buffer callback data is not load-buffer data"),
            };
        let mut tc: *mut client = (*cdata).client();
        let item = (*cdata)
            .item
            .as_ref()
            .and_then(CmdqItemWeak::upgrade)
            .expect("the item that asked for the buffer is waiting on it");
        let item = item.as_ptr();
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
            let copy = bytes.clone();
            if let Err(cause) = paste_set(copy, cstr_ptr(&(*cdata).name)) {
                cmdq_error(item, c"%s".as_ptr(), fmt_args![cause.as_ptr()]);
            } else if !tc.is_null()
                && !(*tc).session.is_null()
                && !(*tc).flags & CLIENT_DEAD as uint64_t != 0
            {
                tty_set_selection(
                    &raw mut (*tc).tty,
                    c"".as_ptr(),
                    bdata as *const ::core::ffi::c_char,
                    bsize,
                );
            }
        }
        let _ = (*cdata).client_ref.take();
        cmdq_continue(item);
    }
}
unsafe fn cmd_load_buffer_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args: &args = cmd_get_args(self_0);
        let mut tc: *mut client = cmdq_get_target_client(&*item);
        let mut cdata = Box::<cmd_load_buffer_data>::default();
        let cdata_ptr = cdata.as_mut() as *mut cmd_load_buffer_data;
        let mut bufname: *const ::core::ffi::c_char = args_get(args, 'b' as i32 as u_char);
        (*cdata_ptr).item = cmdq_item_weak_from_ptr(item);
        if !bufname.is_null() {
            (*cdata_ptr).name = Some(CStr::from_ptr(bufname).to_owned());
        }
        if args_has(args, 'w' as i32 as u_char) != 0 && !tc.is_null() {
            (*cdata_ptr).client_ref = client_ref_from_ptr(tc);
        }
        let path = format_single_from_target(item, CStr::from_ptr(args_string(args, 0 as u_int)));
        file_read(
            cmdq_get_client(&*item),
            path.as_ptr(),
            Some(cmd_load_buffer_done),
            ClientFileData::LoadBuffer(cdata),
        );
        CMD_RETURN_WAIT
    }
}
