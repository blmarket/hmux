use crate::arguments::{args_count, args_has, args_string};
use crate::cmd::queue::{cmdq_error, cmdq_get_target};
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::fmt_args;
use crate::format::format_single_from_target;
use crate::notify::notify_hook;
use crate::options::{
    options_array_assign, options_array_clear, options_array_get, options_array_set, options_empty,
    options_from_string, options_is_array, options_remove_or_default,
    options_scope_from_name, options_set_string, options_table_entry,
};
use crate::options::{options_get_only_ptr, options_get_ptr};
use crate::options::{options_match, options_push_changes};
pub use crate::types::*;
use crate::window::{window_panes_first, window_panes_next};
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
pub const OPTIONS_TABLE_COMMAND: options_table_type = 6;
pub const OPTIONS_TABLE_CHOICE: options_table_type = 5;
pub const OPTIONS_TABLE_FLAG: options_table_type = 4;
pub const OPTIONS_TABLE_COLOUR: options_table_type = 3;
pub const OPTIONS_TABLE_KEY: options_table_type = 2;
pub const OPTIONS_TABLE_NUMBER: options_table_type = 1;
pub const OPTIONS_TABLE_STRING: options_table_type = 0;
pub const CMD_FIND_CANFAIL: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_WINDOW: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub(crate) static cmd_set_option_entry: cmd_entry = {
    cmd_entry {
        name: c"set-option",
        alias: Some(c"set"),
        args: args_parse_t {
            template: c"aFgopqst:uUw",
            lower: 1 as ::core::ffi::c_int,
            upper: 2 as ::core::ffi::c_int,
            cb: Some(cmd_set_option_args_parse),
        },
        usage: c"[-aFgopqsuUw] [-t target-pane] option [value]",
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
        flags: CMD_AFTERHOOK,
        exec: cmd_set_option_exec,
    }
};
pub(crate) static cmd_set_window_option_entry: cmd_entry = {
    cmd_entry {
        name: c"set-window-option",
        alias: Some(c"setw"),
        args: args_parse_t {
            template: c"aFgoqt:u",
            lower: 1 as ::core::ffi::c_int,
            upper: 2 as ::core::ffi::c_int,
            cb: Some(cmd_set_option_args_parse),
        },
        usage: c"[-aFgoqu] [-t target-window] option [value]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: CMD_FIND_CANFAIL,
        },
        flags: CMD_AFTERHOOK,
        exec: cmd_set_option_exec,
    }
};
pub(crate) static cmd_set_hook_entry: cmd_entry = {
    cmd_entry {
        name: c"set-hook",
        alias: None,
        args: args_parse_t {
            template: c"agpRt:uw",
            lower: 1 as ::core::ffi::c_int,
            upper: 2 as ::core::ffi::c_int,
            cb: Some(cmd_set_option_args_parse),
        },
        usage: c"[-agpRuw] [-t target-pane] hook [command]",
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
        flags: CMD_AFTERHOOK,
        exec: cmd_set_option_exec,
    }
};
unsafe fn cmd_set_option_args_parse(
    _args: &args,
    mut idx: u_int,
    _cause: &mut Option<CString>,
) -> args_parse_type {
    if idx == 1 as u_int {
        return ARGS_PARSE_COMMANDS_OR_STRING;
    }
    ARGS_PARSE_STRING
}
unsafe fn cmd_set_option_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let mut current_block: u64;
        let args: &args = cmd_get_args(self_0);
        let mut append: ::core::ffi::c_int = args_has(args, 'a' as i32 as u_char);
        let mut target: *mut cmd_find_state = cmdq_get_target(item);
        let mut loop_0: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut oo: *mut options = ::core::ptr::null_mut::<options>();
        let mut parent: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
        let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
        let mut po: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
        let mut expanded: Option<CString> = None;
        let mut cause: Option<CString> = None;
        let mut array_cause: Option<CString> = None;
        let mut option_cause: Option<CString> = None;
        let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut window: ::core::ffi::c_int = 0;
        let mut idx: ::core::ffi::c_int = 0;
        let mut already: ::core::ffi::c_int = 0;
        let mut error: ::core::ffi::c_int = 0;
        let mut ambiguous: ::core::ffi::c_int = 0;
        let mut scope: ::core::ffi::c_int = 0;
        window = (::core::ptr::eq(cmd_get_entry(self_0), &cmd_set_window_option_entry))
            as ::core::ffi::c_int;
        let argument = format_single_from_target(
            item,
            ::core::ffi::CStr::from_ptr(args_string(args, 0 as u_int)),
        );
        if ::core::ptr::eq(cmd_get_entry(self_0), &cmd_set_hook_entry)
            && args_has(args, 'R' as i32 as u_char) != 0
        {
            notify_hook(item, &argument);
            return CMD_RETURN_NORMAL;
        }
        let name = options_match(&argument, &mut idx, &mut ambiguous);
        if name.is_none() {
            if args_has(args, 'q' as i32 as u_char) != 0 {
                current_block = 11153144165560816752;
            } else {
                if ambiguous != 0 {
                    cmdq_error(
                        item,
                        c"ambiguous option: %s".as_ptr(),
                        fmt_args![argument.as_ptr()],
                    );
                } else {
                    cmdq_error(
                        item,
                        c"invalid option: %s".as_ptr(),
                        fmt_args![argument.as_ptr()],
                    );
                }
                current_block = 16446286653754202049;
            }
        } else {
            let name = name.unwrap();
            let name_ptr = name.as_ptr();
            if args_count(args) < 2 as u_int {
                value = ::core::ptr::null::<::core::ffi::c_char>();
            } else {
                value = args_string(args, 1 as u_int);
            }
            if !value.is_null() && args_has(args, 'F' as i32 as u_char) != 0 {
                expanded = Some(format_single_from_target(
                    item,
                    ::core::ffi::CStr::from_ptr(value),
                ));
                value = expanded.as_ref().expect("just expanded").as_ptr();
            }
            scope = options_scope_from_name(args, window, &name, target, &mut oo, &mut cause);
            if scope == OPTIONS_TABLE_NONE {
                if args_has(args, 'q' as i32 as u_char) != 0 {
                    current_block = 11153144165560816752;
                } else {
                    let cause = cause.unwrap();
                    cmdq_error(item, c"%s".as_ptr(), fmt_args![cause.as_ptr()]);
                    current_block = 16446286653754202049;
                }
            } else {
                o = options_get_only_ptr(oo, name_ptr);
                parent = options_get_ptr(oo, name_ptr);
                if idx != -(1 as ::core::ffi::c_int)
                    && (name.to_bytes().first() == Some(&b'@')
                        || options_is_array(parent) == 0)
                {
                    cmdq_error(
                        item,
                        c"not an array: %s".as_ptr(),
                        fmt_args![argument.as_ptr()],
                    );
                    current_block = 16446286653754202049;
                } else {
                    if args_has(args, 'u' as i32 as u_char) == 0
                        && args_has(args, 'o' as i32 as u_char) != 0
                    {
                        if idx == -(1 as ::core::ffi::c_int) {
                            already = (!o.is_null()) as ::core::ffi::c_int;
                        } else if o.is_null() {
                            already = 0 as ::core::ffi::c_int;
                        } else {
                            already = (!options_array_get(o, idx as u_int).is_null())
                                as ::core::ffi::c_int;
                        }
                        if already != 0 {
                            if args_has(args, 'q' as i32 as u_char) != 0 {
                                current_block = 11153144165560816752;
                            } else {
                                cmdq_error(
                                    item,
                                    c"already set: %s".as_ptr(),
                                    fmt_args![argument.as_ptr()],
                                );
                                current_block = 16446286653754202049;
                            }
                        } else {
                            current_block = 10692455896603418738;
                        }
                    } else {
                        current_block = 10692455896603418738;
                    }
                    match current_block {
                        11153144165560816752 => {}
                        16446286653754202049 => {}
                        _ => {
                            if args_has(args, 'U' as i32 as u_char) != 0
                                && scope == OPTIONS_TABLE_WINDOW
                            {
                                let pw: *mut window = (*target).window();
                                loop_0 = window_panes_first(pw);
                                loop {
                                    if loop_0.is_null() {
                                        current_block = 1356832168064818221;
                                        break;
                                    }
                                    po = options_get_only_ptr(
                                        (*loop_0).options_ptr(),
                                        name_ptr,
                                    );
                                    if !po.is_null()
                                        && options_remove_or_default(po, idx, &mut array_cause)
                                            != 0 as ::core::ffi::c_int
                                    {
                                        cmdq_error(
                                            item,
                                            c"%s".as_ptr(),
                                            fmt_args![array_cause.as_ref().unwrap().as_ptr()],
                                        );
                                        current_block = 16446286653754202049;
                                        break;
                                    }
                                    loop_0 = window_panes_next(pw, loop_0);
                                }
                            } else {
                                current_block = 1356832168064818221;
                            }
                            match current_block {
                                16446286653754202049 => {}
                                _ => {
                                    if args_has(args, 'u' as i32 as u_char) != 0
                                        || args_has(args, 'U' as i32 as u_char) != 0
                                    {
                                        if o.is_null() {
                                            current_block = 11153144165560816752;
                                        } else if options_remove_or_default(
                                            o,
                                            idx,
                                            &mut array_cause,
                                        ) != 0 as ::core::ffi::c_int
                                        {
                                            cmdq_error(
                                                item,
                                                c"%s".as_ptr(),
                                                fmt_args![array_cause.as_ref().unwrap().as_ptr()],
                                            );
                                            current_block = 16446286653754202049;
                                        } else {
                                            current_block = 15462640364611497761;
                                        }
                                    } else if name.to_bytes().first() == Some(&b'@') {
                                        if value.is_null() {
                                            cmdq_error(item, c"empty value".as_ptr(), fmt_args![]);
                                            current_block = 16446286653754202049;
                                        } else {
                                            options_set_string(
                                                oo,
                                                name_ptr,
                                                append,
                                                c"%s".as_ptr(),
                                                fmt_args![value],
                                            );
                                            current_block = 15462640364611497761;
                                        }
                                    } else if idx == -(1 as ::core::ffi::c_int)
                                        && options_is_array(parent) == 0
                                    {
                                        let parent_entry = options_table_entry(parent);
                                        error = options_from_string(
                                            oo,
                                            parent_entry,
                                            parent_entry.unwrap().name.as_ptr(),
                                            value,
                                            args_has(args, 'a' as i32 as u_char),
                                            &mut option_cause,
                                        );
                                        if error != 0 as ::core::ffi::c_int {
                                            if let Some(cause) = option_cause.as_ref() {
                                                cmdq_error(
                                                    item,
                                                    c"%s".as_ptr(),
                                                    fmt_args![cause.as_ptr()],
                                                );
                                            }
                                            current_block = 16446286653754202049;
                                        } else {
                                            current_block = 15462640364611497761;
                                        }
                                    } else if value.is_null() {
                                        cmdq_error(item, c"empty value".as_ptr(), fmt_args![]);
                                        current_block = 16446286653754202049;
                                    } else {
                                        if o.is_null() {
                                            o = options_empty(
                                                oo,
                                                options_table_entry(parent).unwrap(),
                                            );
                                        }
                                        if idx == -(1 as ::core::ffi::c_int) {
                                            if append == 0 {
                                                options_array_clear(o);
                                            }
                                            if options_array_assign(
                                                o,
                                                (!value.is_null()).then(|| CStr::from_ptr(value)),
                                                &mut array_cause,
                                            ) != 0 as ::core::ffi::c_int
                                            {
                                                cmdq_error(
                                                    item,
                                                    c"%s".as_ptr(),
                                                    fmt_args![
                                                        array_cause.as_ref().unwrap().as_ptr()
                                                    ],
                                                );
                                                current_block = 16446286653754202049;
                                            } else {
                                                current_block = 15462640364611497761;
                                            }
                                        } else if options_array_set(
                                            o,
                                            idx as u_int,
                                            value,
                                            append,
                                            &mut array_cause,
                                        ) != 0 as ::core::ffi::c_int
                                        {
                                            cmdq_error(
                                                item,
                                                c"%s".as_ptr(),
                                                fmt_args![array_cause.as_ref().unwrap().as_ptr()],
                                            );
                                            current_block = 16446286653754202049;
                                        } else {
                                            current_block = 15462640364611497761;
                                        }
                                    }
                                    match current_block {
                                        16446286653754202049 => {}
                                        11153144165560816752 => {}
                                        _ => {
                                            options_push_changes(&name);
                                            current_block = 11153144165560816752;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        match current_block {
            16446286653754202049 => CMD_RETURN_ERROR,
            _ => CMD_RETURN_NORMAL,
        }
    }
}
