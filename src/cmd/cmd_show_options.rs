use crate::arguments::args_escape;
use crate::arguments::{args_count, args_has, args_string};
use crate::cmd::queue::{cmdq_error, cmdq_get_target, cmdq_print};
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::fmt_args;
use crate::format::format_single_from_target;
use crate::options::options_match;
use crate::options::options_table;
use crate::options::{
    options_array_first, options_array_item_index, options_array_next, options_first,
    options_is_array, options_is_string, options_name, options_next, options_scope_from_flags,
    options_scope_from_name, options_table_entry, options_to_string,
};
use crate::options::{options_get_only_ptr, options_get_ptr};
pub use crate::types::*;
use crate::xmalloc::xasprintf;
use ::std::ffi::CString;
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
pub const OPTIONS_TABLE_IS_HOOK: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub(crate) static cmd_show_options_entry: cmd_entry = {
    cmd_entry {
        name: c"show-options",
        alias: Some(c"show"),
        args: args_parse_t {
            template: c"AgHpqst:vw",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-AgHpqsvw] [-t target-pane] [option]",
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
        exec: cmd_show_options_exec,
    }
};
pub(crate) static cmd_show_window_options_entry: cmd_entry = {
    cmd_entry {
        name: c"show-window-options",
        alias: Some(c"showw"),
        args: args_parse_t {
            template: c"gvt:",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-gv] [-t target-window] [option]",
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
        exec: cmd_show_options_exec,
    }
};
pub(crate) static cmd_show_hooks_entry: cmd_entry = {
    cmd_entry {
        name: c"show-hooks",
        alias: None,
        args: args_parse_t {
            template: c"gpt:w",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-gpw] [-t target-pane] [hook]",
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
        exec: cmd_show_options_exec,
    }
};
unsafe fn cmd_show_options_exec(mut self_0: &cmd, mut item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let mut current_block: u64;
        let args: &args = cmd_get_args(self_0);
        let mut target: *mut cmd_find_state = cmdq_get_target(item);
        let mut oo: *mut options = ::core::ptr::null_mut::<options>();
        let mut cause: Option<CString> = None;
        let mut window: ::core::ffi::c_int = 0;
        let mut idx: ::core::ffi::c_int = 0;
        let mut ambiguous: ::core::ffi::c_int = 0;
        let mut parent: ::core::ffi::c_int = 0;
        let mut scope: ::core::ffi::c_int = 0;
        let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
        window = (::core::ptr::eq(cmd_get_entry(self_0), &cmd_show_window_options_entry))
            as ::core::ffi::c_int;
        if args_count(args) == 0 as u_int {
            scope = options_scope_from_flags(args, window, target, &raw mut oo, &mut cause);
            if scope == OPTIONS_TABLE_NONE {
                if args_has(args, 'q' as i32 as u_char) != 0 {
                    return CMD_RETURN_NORMAL;
                }
                let cause = cause.unwrap();
                cmdq_error(item, c"%s".as_ptr(), fmt_args![cause.as_ptr()]);
                return CMD_RETURN_ERROR;
            }
            return cmd_show_options_all(self_0, item, scope, oo);
        }
        let argument = format_single_from_target(
            item,
            ::core::ffi::CStr::from_ptr(args_string(args, 0 as u_int)),
        );
        let name = options_match(argument.as_ptr(), &raw mut idx, &raw mut ambiguous);
        let name_ptr = name
            .as_ref()
            .map_or(::core::ptr::null(), |name| name.as_ptr());
        if name.is_none() {
            if args_has(args, 'q' as i32 as u_char) != 0 {
                current_block = 14351605340212681318;
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
                current_block = 648366277789593999;
            }
        } else {
            scope =
                options_scope_from_name(args, window, name_ptr, target, &raw mut oo, &mut cause);
            if scope == OPTIONS_TABLE_NONE {
                if args_has(args, 'q' as i32 as u_char) != 0 {
                    current_block = 14351605340212681318;
                } else {
                    let cause = cause.unwrap();
                    cmdq_error(item, c"%s".as_ptr(), fmt_args![cause.as_ptr()]);
                    current_block = 648366277789593999;
                }
            } else {
                o = options_get_only_ptr(oo, name_ptr);
                if args_has(args, 'A' as i32 as u_char) != 0 && o.is_null() {
                    o = options_get_ptr(oo, name_ptr);
                    parent = 1 as ::core::ffi::c_int;
                } else {
                    parent = 0 as ::core::ffi::c_int;
                }
                if !o.is_null() {
                    cmd_show_options_print(self_0, item, o, idx, parent);
                    current_block = 14351605340212681318;
                } else if *name_ptr as ::core::ffi::c_int == '@' as i32 {
                    if args_has(args, 'q' as i32 as u_char) != 0 {
                        current_block = 14351605340212681318;
                    } else {
                        cmdq_error(
                            item,
                            c"invalid option: %s".as_ptr(),
                            fmt_args![argument.as_ptr()],
                        );
                        current_block = 648366277789593999;
                    }
                } else {
                    current_block = 14351605340212681318;
                }
            }
        }
        match current_block {
            648366277789593999 => CMD_RETURN_ERROR,
            _ => CMD_RETURN_NORMAL,
        }
    }
}
unsafe fn cmd_show_options_print(
    mut self_0: &cmd,
    mut item: *mut cmdq_item,
    mut o: *mut options_entry,
    mut idx: ::core::ffi::c_int,
    mut parent: ::core::ffi::c_int,
) {
    unsafe {
        let args: &args = cmd_get_args(self_0);
        let mut a: *mut options_array_item_t = ::core::ptr::null_mut::<options_array_item_t>();
        let mut name: *const ::core::ffi::c_char = options_name(o);
        let _tmp = if idx != -(1 as ::core::ffi::c_int) {
            let tmp = xasprintf(c"%s[%d]".as_ptr(), fmt_args![name, idx]);
            name = tmp.as_ptr();
            Some(tmp)
        } else if options_is_array(o) != 0 {
            a = options_array_first(o);
            if a.is_null() {
                if args_has(args, 'v' as i32 as u_char) == 0 {
                    cmdq_print(item, c"%s".as_ptr(), fmt_args![name]);
                }
                return;
            }
            while !a.is_null() {
                idx = options_array_item_index(a) as ::core::ffi::c_int;
                cmd_show_options_print(self_0, item, o, idx, parent);
                a = options_array_next(o, a);
            }
            return;
        } else {
            None
        };
        let value = options_to_string(o, idx, 0 as ::core::ffi::c_int);
        if args_has(args, 'v' as i32 as u_char) != 0 {
            cmdq_print(item, c"%s".as_ptr(), fmt_args![value.as_ptr()]);
        } else if options_is_string(o) != 0 {
            let escaped = args_escape(value.as_ptr());
            if parent != 0 {
                cmdq_print(item, c"%s* %s".as_ptr(), fmt_args![name, escaped.as_ptr()]);
            } else {
                cmdq_print(item, c"%s %s".as_ptr(), fmt_args![name, escaped.as_ptr()]);
            }
        } else if parent != 0 {
            cmdq_print(item, c"%s* %s".as_ptr(), fmt_args![name, value.as_ptr()]);
        } else {
            cmdq_print(item, c"%s %s".as_ptr(), fmt_args![name, value.as_ptr()]);
        }
    }
}
unsafe fn cmd_show_options_all(
    mut self_0: &cmd,
    mut item: *mut cmdq_item,
    mut scope: ::core::ffi::c_int,
    mut oo: *mut options,
) -> cmd_retval {
    unsafe {
        let args: &args = cmd_get_args(self_0);
        let mut oe: *const options_table_entry_t = ::core::ptr::null::<options_table_entry_t>();
        let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
        let mut a: *mut options_array_item_t = ::core::ptr::null_mut::<options_array_item_t>();
        let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut idx: u_int = 0;
        let mut parent: ::core::ffi::c_int = 0;
        if !::core::ptr::eq(cmd_get_entry(self_0), &cmd_show_hooks_entry) {
            o = options_first(oo);
            while !o.is_null() {
                if options_table_entry(o).is_null() {
                    cmd_show_options_print(
                        self_0,
                        item,
                        o,
                        -(1 as ::core::ffi::c_int),
                        0 as ::core::ffi::c_int,
                    );
                }
                o = options_next(o);
            }
        }
        let mut current_block_28: u64;
        for entry in &options_table {
            oe = entry;
            if !(!(*oe).scope & scope != 0)
                && !(!::core::ptr::eq(cmd_get_entry(self_0), &cmd_show_hooks_entry)
                    && args_has(args, 'H' as i32 as u_char) == 0
                    && (*oe).flags & OPTIONS_TABLE_IS_HOOK != 0
                    || ::core::ptr::eq(cmd_get_entry(self_0), &cmd_show_hooks_entry)
                        && !(*oe).flags & OPTIONS_TABLE_IS_HOOK != 0)
            {
                o = options_get_only_ptr(oo, (*oe).name.as_ptr());
                if o.is_null() {
                    if args_has(args, 'A' as i32 as u_char) == 0 {
                        current_block_28 = 11812396948646013369;
                    } else {
                        o = options_get_ptr(oo, (*oe).name.as_ptr());
                        if o.is_null() {
                            current_block_28 = 11812396948646013369;
                        } else {
                            parent = 1 as ::core::ffi::c_int;
                            current_block_28 = 2370887241019905314;
                        }
                    }
                } else {
                    parent = 0 as ::core::ffi::c_int;
                    current_block_28 = 2370887241019905314;
                }
                match current_block_28 {
                    11812396948646013369 => {}
                    _ => {
                        if options_is_array(o) == 0 {
                            cmd_show_options_print(
                                self_0,
                                item,
                                o,
                                -(1 as ::core::ffi::c_int),
                                parent,
                            );
                        } else {
                            a = options_array_first(o);
                            if a.is_null() {
                                if args_has(args, 'v' as i32 as u_char) == 0 {
                                    name = options_name(o);
                                    if parent != 0 {
                                        cmdq_print(item, c"%s*".as_ptr(), fmt_args![name]);
                                    } else {
                                        cmdq_print(item, c"%s".as_ptr(), fmt_args![name]);
                                    }
                                }
                            } else {
                                while !a.is_null() {
                                    idx = options_array_item_index(a);
                                    cmd_show_options_print(
                                        self_0,
                                        item,
                                        o,
                                        idx as ::core::ffi::c_int,
                                        parent,
                                    );
                                    a = options_array_next(o, a);
                                }
                            }
                        }
                    }
                }
            }
            oe = oe.offset(1);
        }
        CMD_RETURN_NORMAL
    }
}
