use crate::arguments::args_get_str;
use crate::arguments::{args_count, args_get, args_has, args_to_vector, args_value_list};
use crate::cfg::{cfg_finished, cfg_show_causes};
use crate::cmd::cmd_attach_session::cmd_attach_session;
use crate::cmd::find::cmd_find_from_session;
use crate::cmd::queue::cmdq_item_weak_from_ptr;
use crate::cmd::queue::{
    cmdq_error, cmdq_get_client, cmdq_get_current, cmdq_get_flags, cmdq_get_target,
    cmdq_insert_hook, cmdq_print,
};
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::compat::strtonum;
use crate::environ::{environ_create_box, environ_ptr, environ_put, environ_t, environ_update};
use crate::ffi::{sscanf, strcmp, tcgetattr};
use crate::fmt_args;
use crate::format::format_single;
use crate::log::fatal;
use crate::notify::notify_session;
use crate::options::{
    options_create_boxed, options_get_number, options_get_string, options_ptr, options_set_string,
};
use crate::proc::{peer_ptr, proc_send};
use crate::server::client_set_last_session;
use crate::server::client_weak_from_ptr;
use crate::server::{
    server_client_check_nested, server_client_get_cwd, server_client_open, server_client_set_flags,
    server_client_set_key_table, server_client_set_session,
};
use crate::session::{
    session_create, session_destroy, session_find, session_group_add, session_group_contains,
    session_group_find, session_group_name, session_group_new, session_group_synchronize_to,
    session_select,
};
use crate::session::{session_get_curw, session_name};
use crate::spawn::spawn_window;
use crate::tmux::global_s_options;
use crate::tmux::{check_name, clean_name};
pub use crate::types::*;
use crate::window::winlinks_first;
use ::std::ffi::{CStr, CString};
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
pub const USHRT_MAX: ::core::ffi::c_int =
    __SHRT_MAX__ * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const CMD_FIND_CANFAIL: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CMDQ_STATE_REPEAT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMD_STARTSERVER: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CLIENT_ATTACHED: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const CMD_TARGET_SESSION_USAGE: &::core::ffi::CStr = c"[-t target-session]";
pub const NEW_SESSION_TEMPLATE: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"#{session_name}:\0")
};
pub(crate) static cmd_new_session_entry: cmd_entry = {
    cmd_entry {
        name: c"new-session",
        alias: Some(c"new"),
        args: args_parse_t {
            template: c"Ac:dDe:EF:f:n:Ps:t:x:Xy:",
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: c"[-AdDEPX] [-c start-directory] [-e environment] [-F format] [-f flags] [-n window-name] [-s session-name] [-t target-session] [-x width] [-y height] [shell-command [argument ...]]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: CMD_FIND_CANFAIL,
        },
        flags: CMD_STARTSERVER,
        exec: cmd_new_session_exec,
    }
};
pub(crate) static cmd_has_session_entry: cmd_entry = {
    cmd_entry {
        name: c"has-session",
        alias: Some(c"has"),
        args: args_parse_t {
            template: c"t:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_SESSION_USAGE,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: cmd_new_session_exec,
    }
};
unsafe fn cmd_new_session_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let mut current_block: u64;
        let args: &args = cmd_get_args(self_0);
        let mut current: *mut cmd_find_state = cmdq_get_current(item);
        let mut target: *mut cmd_find_state = cmdq_get_target(item);
        let mut c: *mut client = cmdq_get_client(&*item);
        let mut s: *mut session = ::core::ptr::null_mut::<session>();
        let mut as_0: *mut session = ::core::ptr::null_mut::<session>();
        let mut groupwith: *mut session = ::core::ptr::null_mut::<session>();
        let mut env: Option<Box<environ_t>> = None;
        let mut oo: Option<Box<options>> = None;
        let mut tio: termios = ::core::mem::zeroed();
        let mut tiop: *mut termios = ::core::ptr::null_mut::<termios>();
        let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
        let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut group: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut tmp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut cause: Option<CString> = None;
        let mut cwd: Option<CString> = None;
        let mut wname: Option<CString> = None;
        let mut sname: Option<CString> = None;
        let mut prefix: Option<CString> = None;
        let mut detached: ::core::ffi::c_int = 0;
        let mut already_attached: ::core::ffi::c_int = 0;
        let mut is_control: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        let mut dsx: u_int = 0;
        let mut dsy: u_int = 0;
        let mut count: u_int = args_count(args);
        let mut sc = spawn_context::default();
        let mut retval: cmd_retval = CMD_RETURN_NORMAL;
        let mut fs = cmd_find_state::default();
        if ::core::ptr::eq(cmd_get_entry(self_0), &cmd_has_session_entry) {
            return CMD_RETURN_NORMAL;
        }
        if args_has(args, 't' as i32 as u_char) != 0
            && (count != 0 as u_int || args_has(args, 'n' as i32 as u_char) != 0)
        {
            cmdq_error(
                item,
                c"command or window name given with target".as_ptr(),
                fmt_args![],
            );
            return CMD_RETURN_ERROR;
        }
        tmp = args_get(args, 'n' as i32 as u_char);
        if !tmp.is_null() {
            let ename = format_single(
                item,
                CStr::from_ptr(tmp),
                c,
                ::core::ptr::null_mut::<session>(),
                ::core::ptr::null_mut::<winlink>(),
                ::core::ptr::null_mut::<window_pane>(),
            );
            if check_name(ename.as_ptr()) == 0 {
                cmdq_error(
                    item,
                    c"invalid window name: %s".as_ptr(),
                    fmt_args![ename.as_ptr()],
                );
                return CMD_RETURN_ERROR;
            }
            wname = clean_name(ename.as_ptr(), 0 as ::core::ffi::c_int);
        }
        tmp = args_get(args, 's' as i32 as u_char);
        if !tmp.is_null() {
            let ename = format_single(
                item,
                CStr::from_ptr(tmp),
                c,
                ::core::ptr::null_mut::<session>(),
                ::core::ptr::null_mut::<winlink>(),
                ::core::ptr::null_mut::<window_pane>(),
            );
            if check_name(ename.as_ptr()) == 0 {
                cmdq_error(
                    item,
                    c"invalid session name: %s".as_ptr(),
                    fmt_args![ename.as_ptr()],
                );
                current_block = 971598175612140897;
            } else {
                sname = clean_name(ename.as_ptr(), 0 as ::core::ffi::c_int);
                current_block = 10043043949733653460;
            }
        } else {
            current_block = 10043043949733653460;
        }
        if current_block == 10043043949733653460 {
            if args_has(args, 'A' as i32 as u_char) != 0 {
                if let Some(sname) = sname.as_ref() {
                    as_0 = session_find(sname.as_ptr());
                } else {
                    as_0 = (*target).session();
                }
                if !as_0.is_null() {
                    retval = cmd_attach_session(
                        item,
                        session_name(as_0),
                        args_has(args, 'D' as i32 as u_char),
                        args_has(args, 'X' as i32 as u_char),
                        0 as ::core::ffi::c_int,
                        args_get(args, 'c' as i32 as u_char),
                        args_has(args, 'E' as i32 as u_char),
                        args_get(args, 'f' as i32 as u_char),
                    );
                    return retval;
                }
            }
            if sname
                .as_ref()
                .is_some_and(|sname| !session_find(sname.as_ptr()).is_null())
            {
                cmdq_error(
                    item,
                    c"duplicate session: %s".as_ptr(),
                    fmt_args![sname.as_ref().unwrap().as_ptr()],
                );
            } else {
                group = args_get(args, 't' as i32 as u_char);
                if !group.is_null() {
                    groupwith = (*target).session();
                    if groupwith.is_null() {
                        sg = session_group_find(group);
                    } else {
                        sg = session_group_contains(groupwith);
                    }
                    if !sg.is_null() {
                        prefix = Some(CStr::from_ptr(session_group_name(sg)).to_owned());
                        current_block = 6717214610478484138;
                    } else if !groupwith.is_null() {
                        prefix = Some(CStr::from_ptr(session_name(groupwith)).to_owned());
                        current_block = 6717214610478484138;
                    } else if check_name(group) == 0 {
                        cmdq_error(
                            item,
                            c"invalid session group name: %s".as_ptr(),
                            fmt_args![group],
                        );
                        current_block = 971598175612140897;
                    } else {
                        prefix = clean_name(group, 0 as ::core::ffi::c_int);
                        current_block = 6717214610478484138;
                    }
                } else {
                    current_block = 6717214610478484138;
                }
                match current_block {
                    971598175612140897 => {}
                    _ => {
                        detached = args_has(args, 'd' as i32 as u_char);
                        if c.is_null() {
                            detached = 1 as ::core::ffi::c_int;
                        } else if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                            is_control = 1 as ::core::ffi::c_int;
                        }
                        already_attached = 0 as ::core::ffi::c_int;
                        if !c.is_null() && !(*c).session.is_null() {
                            already_attached = 1 as ::core::ffi::c_int;
                        }
                        tmp = args_get(args, 'c' as i32 as u_char);
                        if !tmp.is_null() {
                            cwd = Some(format_single(
                                item,
                                CStr::from_ptr(tmp),
                                c,
                                ::core::ptr::null_mut::<session>(),
                                ::core::ptr::null_mut::<winlink>(),
                                ::core::ptr::null_mut::<window_pane>(),
                            ));
                        } else {
                            cwd = Some(
                                CStr::from_ptr(server_client_get_cwd(
                                    c,
                                    ::core::ptr::null_mut::<session>(),
                                ))
                                .to_owned(),
                            );
                        }
                        if detached == 0
                            && already_attached == 0
                            && (*c).fd != -(1 as ::core::ffi::c_int)
                            && !(*c).flags & CLIENT_CONTROL as uint64_t != 0
                        {
                            if server_client_check_nested(cmdq_get_client(&*item)) != 0 {
                                cmdq_error(
                                    item,
                                    c"sessions should be nested with care, unset $TMUX to force"
                                        .as_ptr(),
                                    fmt_args![],
                                );
                                current_block = 971598175612140897;
                            } else {
                                if tcgetattr((*c).fd, &raw mut tio) != 0 as ::core::ffi::c_int {
                                    fatal(c"tcgetattr failed".as_ptr(), fmt_args![]);
                                }
                                tiop = &raw mut tio;
                                current_block = 6545907279487748450;
                            }
                        } else {
                            tiop = ::core::ptr::null_mut::<termios>();
                            current_block = 6545907279487748450;
                        }
                        match current_block {
                            971598175612140897 => {}
                            _ => {
                                if detached == 0 && already_attached == 0 {
                                    let mut terminal_cause = None;
                                    if server_client_open(c, &mut terminal_cause)
                                        != 0 as ::core::ffi::c_int
                                    {
                                        cmdq_error(
                                            item,
                                            c"open terminal failed: %s".as_ptr(),
                                            fmt_args![
                                                terminal_cause
                                                    .as_ref()
                                                    .map_or(::core::ptr::null(), |cause| cause
                                                        .as_ptr(),)
                                            ],
                                        );
                                        current_block = 971598175612140897;
                                    } else {
                                        current_block = 5181772461570869434;
                                    }
                                } else {
                                    current_block = 5181772461570869434;
                                }
                                match current_block {
                                    971598175612140897 => {}
                                    _ => {
                                        if args_has(args, 'x' as i32 as u_char) != 0 {
                                            tmp = args_get(args, 'x' as i32 as u_char);
                                            if strcmp(tmp, c"-".as_ptr()) == 0 as ::core::ffi::c_int
                                            {
                                                if !c.is_null() {
                                                    dsx = (*c).tty.sx;
                                                } else {
                                                    dsx = 80 as u_int;
                                                }
                                                current_block = 5873035170358615968;
                                            } else {
                                                match strtonum(
                                                    tmp,
                                                    1 as ::core::ffi::c_longlong,
                                                    USHRT_MAX as ::core::ffi::c_longlong,
                                                ) {
                                                    Ok(value) => {
                                                        dsx = value as u_int;
                                                        current_block = 5873035170358615968;
                                                    }
                                                    Err(errstr) => {
                                                        cmdq_error(
                                                            item,
                                                            c"width %s".as_ptr(),
                                                            fmt_args![errstr.as_ptr()],
                                                        );
                                                        current_block = 971598175612140897;
                                                    }
                                                }
                                            }
                                        } else {
                                            dsx = 80 as u_int;
                                            current_block = 5873035170358615968;
                                        }
                                        match current_block {
                                            971598175612140897 => {}
                                            _ => {
                                                if args_has(args, 'y' as i32 as u_char) != 0 {
                                                    tmp = args_get(args, 'y' as i32 as u_char);
                                                    if strcmp(tmp, c"-".as_ptr())
                                                        == 0 as ::core::ffi::c_int
                                                    {
                                                        if !c.is_null() {
                                                            dsy = (*c).tty.sy;
                                                        } else {
                                                            dsy = 24 as u_int;
                                                        }
                                                        current_block = 15855550149339537395;
                                                    } else {
                                                        match strtonum(
                                                            tmp,
                                                            1 as ::core::ffi::c_longlong,
                                                            USHRT_MAX as ::core::ffi::c_longlong,
                                                        ) {
                                                            Ok(value) => {
                                                                dsy = value as u_int;
                                                                current_block =
                                                                    15855550149339537395;
                                                            }
                                                            Err(errstr) => {
                                                                cmdq_error(
                                                                    item,
                                                                    c"height %s".as_ptr(),
                                                                    fmt_args![errstr.as_ptr()],
                                                                );
                                                                current_block = 971598175612140897;
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    dsy = 24 as u_int;
                                                    current_block = 15855550149339537395;
                                                }
                                                match current_block {
                                                    971598175612140897 => {}
                                                    _ => {
                                                        if detached == 0 && is_control == 0 {
                                                            sx = (*c).tty.sx;
                                                            sy = (*c).tty.sy;
                                                            if sy > 0 as u_int
                                                                && options_get_number(
                                                                    global_s_options,
                                                                    c"status".as_ptr(),
                                                                ) != 0
                                                            {
                                                                sy = sy.wrapping_sub(1);
                                                            }
                                                        } else {
                                                            tmp = options_get_string(
                                                                global_s_options,
                                                                c"default-size".as_ptr(),
                                                            );
                                                            if sscanf(
                                                                tmp,
                                                                c"%ux%u".as_ptr(),
                                                                &raw mut sx,
                                                                &raw mut sy,
                                                            ) != 2 as ::core::ffi::c_int
                                                            {
                                                                sx = dsx;
                                                                sy = dsy;
                                                            } else {
                                                                if args_has(
                                                                    args,
                                                                    'x' as i32 as u_char,
                                                                ) != 0
                                                                {
                                                                    sx = dsx;
                                                                }
                                                                if args_has(
                                                                    args,
                                                                    'y' as i32 as u_char,
                                                                ) != 0
                                                                {
                                                                    sy = dsy;
                                                                }
                                                            }
                                                        }
                                                        if sx == 0 as u_int {
                                                            sx = 1 as u_int;
                                                        }
                                                        if sy == 0 as u_int {
                                                            sy = 1 as u_int;
                                                        }
                                                        oo = Some(options_create_boxed(
                                                            global_s_options,
                                                        ));
                                                        if args_has(args, 'x' as i32 as u_char) != 0
                                                            || args_has(args, 'y' as i32 as u_char)
                                                                != 0
                                                        {
                                                            if args_has(args, 'x' as i32 as u_char)
                                                                == 0
                                                            {
                                                                dsx = sx;
                                                            }
                                                            if args_has(args, 'y' as i32 as u_char)
                                                                == 0
                                                            {
                                                                dsy = sy;
                                                            }
                                                            options_set_string(
                                                                options_ptr(&oo),
                                                                c"default-size".as_ptr(),
                                                                0 as ::core::ffi::c_int,
                                                                c"%ux%u".as_ptr(),
                                                                fmt_args![dsx, dsy],
                                                            );
                                                        }
                                                        env = Some(environ_create_box());
                                                        if !c.is_null()
                                                            && args_has(args, 'E' as i32 as u_char)
                                                                == 0
                                                        {
                                                            environ_update(
                                                                global_s_options,
                                                                environ_ptr(&(*c).environ),
                                                                environ_ptr(&env),
                                                            );
                                                        }
                                                        for av in args_value_list(
                                                            args,
                                                            'e' as i32 as u_char,
                                                        ) {
                                                            environ_put(
                                                                environ_ptr(&env),
                                                                (*av).value.string(),
                                                                0 as ::core::ffi::c_int,
                                                            );
                                                        }
                                                        let Some(env) = env.take() else {
                                                            unreachable!();
                                                        };
                                                        let Some(oo) = oo.take() else {
                                                            unreachable!();
                                                        };
                                                        s = session_create(
                                                            prefix.as_ref().map_or(
                                                                ::core::ptr::null(),
                                                                |name| name.as_ptr(),
                                                            ),
                                                            sname.as_ref().map_or(
                                                                ::core::ptr::null(),
                                                                |name| name.as_ptr(),
                                                            ),
                                                            cstr_ptr(&cwd),
                                                            env,
                                                            oo,
                                                            tiop,
                                                        );
                                                        sc.item = cmdq_item_weak_from_ptr(item);
                                                        sc.s = s;
                                                        if detached == 0 {
                                                            sc.tc = client_weak_from_ptr(c);
                                                        }
                                                        sc.name = wname.as_deref();
                                                        sc.argv = args_to_vector(args);
                                                        sc.idx = -(1 as ::core::ffi::c_int);
                                                        sc.cwd = args_get_str(
                                                            args,
                                                            'c' as i32 as u_char,
                                                        );
                                                        sc.flags = 0 as ::core::ffi::c_int;
                                                        if spawn_window(&mut sc, &mut cause)
                                                            .is_null()
                                                        {
                                                            session_destroy(
                                                                s,
                                                                0 as ::core::ffi::c_int,
                                                                c"cmd_new_session_exec".as_ptr(),
                                                            );
                                                            let cause = cause.unwrap();
                                                            cmdq_error(
                                                                item,
                                                                c"create window failed: %s"
                                                                    .as_ptr(),
                                                                fmt_args![cause.as_ptr()],
                                                            );
                                                        } else {
                                                            if !group.is_null() {
                                                                if sg.is_null() {
                                                                    if !groupwith.is_null() {
                                                                        sg = session_group_new(
                                                                            session_name(groupwith),
                                                                        );
                                                                        session_group_add(
                                                                            sg, groupwith,
                                                                        );
                                                                    } else {
                                                                        sg = session_group_new(
                                                                            group,
                                                                        );
                                                                    }
                                                                }
                                                                session_group_add(sg, s);
                                                                session_group_synchronize_to(s);
                                                                session_select(
                                                                    s,
                                                                    (*winlinks_first(
                                                                        &raw mut (*s).windows,
                                                                    ))
                                                                    .idx,
                                                                );
                                                            }
                                                            notify_session(
                                                                c"session-created".as_ptr(),
                                                                s,
                                                            );
                                                            if detached == 0 {
                                                                if args_has(
                                                                    args,
                                                                    'f' as i32 as u_char,
                                                                ) != 0
                                                                {
                                                                    server_client_set_flags(
                                                                        c,
                                                                        args_get(
                                                                            args,
                                                                            'f' as i32 as u_char,
                                                                        ),
                                                                    );
                                                                }
                                                                if already_attached == 0 {
                                                                    if !(*c).flags
                                                                        & CLIENT_CONTROL as uint64_t
                                                                        != 0
                                                                    {
                                                                        proc_send(
                                                                        peer_ptr(&(*c).peer),
                                                                        MSG_READY,
                                                                        -(1 as ::core::ffi::c_int),
                                                                        ::core::ptr::null::<u8>(),
                                                                        0 as size_t,
                                                                    );
                                                                    }
                                                                } else if !(*c).session.is_null() {
                                                                    client_set_last_session(
                                                                        c,
                                                                        (*c).session,
                                                                    );
                                                                }
                                                                server_client_set_session(c, s);
                                                                if !cmdq_get_flags(&*item)
                                                                    & CMDQ_STATE_REPEAT
                                                                    != 0
                                                                {
                                                                    server_client_set_key_table(
                                                                        c,
                                                                        ::core::ptr::null::<
                                                                            ::core::ffi::c_char,
                                                                        >(
                                                                        ),
                                                                    );
                                                                }
                                                            }
                                                            if args_has(args, 'P' as i32 as u_char)
                                                                != 0
                                                            {
                                                                template = args_get(
                                                                    args,
                                                                    'F' as i32 as u_char,
                                                                );
                                                                if template.is_null() {
                                                                    template = NEW_SESSION_TEMPLATE
                                                                        .as_ptr();
                                                                }
                                                                let cp = format_single(
                                                                    item,
                                                                    CStr::from_ptr(template),
                                                                    c,
                                                                    s,
                                                                    session_get_curw(s),
                                                                    ::core::ptr::null_mut::<
                                                                        window_pane,
                                                                    >(
                                                                    ),
                                                                );
                                                                cmdq_print(
                                                                    item,
                                                                    c"%s".as_ptr(),
                                                                    fmt_args![cp.as_ptr()],
                                                                );
                                                            }
                                                            if detached == 0 {
                                                                (*c).flags |=
                                                                    CLIENT_ATTACHED as uint64_t;
                                                            }
                                                            if args_has(args, 'd' as i32 as u_char)
                                                                == 0
                                                            {
                                                                cmd_find_from_session(
                                                                    &mut *current,
                                                                    s,
                                                                    0 as ::core::ffi::c_int,
                                                                );
                                                            }
                                                            cmd_find_from_session(
                                                                &mut fs,
                                                                s,
                                                                0 as ::core::ffi::c_int,
                                                            );
                                                            cmdq_insert_hook(
                                                                s,
                                                                item,
                                                                &raw mut fs,
                                                                c"after-new-session".as_ptr(),
                                                                fmt_args![],
                                                            );
                                                            if cfg_finished != 0 {
                                                                cfg_show_causes(s);
                                                            }
                                                            return CMD_RETURN_NORMAL;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        CMD_RETURN_ERROR
    }
}
pub const __SHRT_MAX__: ::core::ffi::c_int = 32767 as ::core::ffi::c_int;
