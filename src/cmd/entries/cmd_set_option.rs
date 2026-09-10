use crate::args::args_parse_t;
use crate::args::RustArguments;
use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::consts::{
    ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_STRING, CMD_AFTERHOOK, CMD_FIND_CANFAIL,
    CMD_FIND_PANE, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, OPTIONS_TABLE_NONE,
    OPTIONS_TABLE_WINDOW,
};
use crate::fmt_args;
use crate::format::format_single_from_target;
use crate::notify::notify_hook;
use crate::options::{OptionsEngine, RustOptionsEngine};
use crate::types::{
    OptionsRef, RustOptionsRef, args, args_parse_type, u_char, u_int,
};
use ::std::ffi::CString;

pub(crate) static cmd_set_option_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"set-option",
        alias: Some(c"set"),
        args: args_parse_t {
            template: c"aFgopqst:uUw",
            lower: 1 as core::ffi::c_int,
            upper: 2 as core::ffi::c_int,
            cb: Some(cmd_set_option_args_parse),
        },
        usage: c"[-aFgopqsuUw] [-t target-pane] option [value]",
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
        flags: CMD_AFTERHOOK,
        exec: cmd_set_option_exec,
    }
};
pub(crate) static cmd_set_window_option_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"set-window-option",
        alias: Some(c"setw"),
        args: args_parse_t {
            template: c"aFgoqt:u",
            lower: 1 as core::ffi::c_int,
            upper: 2 as core::ffi::c_int,
            cb: Some(cmd_set_option_args_parse),
        },
        usage: c"[-aFgoqu] [-t target-window] option [value]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: CMD_FIND_CANFAIL,
        },
        flags: CMD_AFTERHOOK,
        exec: cmd_set_option_exec,
    }
};
pub(crate) static cmd_set_hook_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"set-hook",
        alias: None,
        args: args_parse_t {
            template: c"agpRt:uw",
            lower: 1 as core::ffi::c_int,
            upper: 2 as core::ffi::c_int,
            cb: Some(cmd_set_option_args_parse),
        },
        usage: c"[-agpRuw] [-t target-pane] hook [command]",
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
        flags: CMD_AFTERHOOK,
        exec: cmd_set_option_exec,
    }
};
fn cmd_set_option_args_parse(
    _args: &args,
    idx: u_int,
    _cause: &mut Option<CString>,
) -> args_parse_type {
    if idx == 1 as u_int {
        return ARGS_PARSE_COMMANDS_OR_STRING;
    }
    ARGS_PARSE_STRING
}
unsafe fn cmd_set_option_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let mut current_block: u64;
    let args: &RustArguments = cmd_get_args(self_0);
    let append: core::ffi::c_int = {
        let flag = 'a' as i32 as u_char;
        args.argument_flag_count(flag)
    };
    let target = item.target.clone();
    let mut oo: Option<RustOptionsRef> = None;
    let expanded: Option<CString>;
    let mut cause: Option<CString> = None;
    let mut array_cause: Option<CString> = None;
    let mut option_cause: Option<CString> = None;

    let mut idx: core::ffi::c_int = 0;
    let already: core::ffi::c_int;
    let error: core::ffi::c_int;
    let mut ambiguous: core::ffi::c_int = 0;
    let scope: core::ffi::c_int;
    let window: core::ffi::c_int =
        core::ptr::eq(cmd_get_entry(self_0), &cmd_set_window_option_entry) as core::ffi::c_int;
    let argument = unsafe {
        format_single_from_target(
            item,
            args.argument_string(0).expect("argument count checked"),
        )
    };
    if core::ptr::eq(cmd_get_entry(self_0), &cmd_set_hook_entry)
        && ({
            let flag = 'R' as i32 as u_char;
            args.argument_flag_count(flag)
        }) != 0
    {
        unsafe { notify_hook(item, &argument) };
        return CMD_RETURN_NORMAL;
    }
    let name = RustOptionsEngine.match_name(&argument, &mut idx, &mut ambiguous);
    if name.is_none() {
        if ({
            let flag = 'q' as i32 as u_char;
            args.argument_flag_count(flag)
        }) != 0
        {
            current_block = 11153144165560816752;
        } else {
            if ambiguous != 0 {
                unsafe { item.error(c"ambiguous option: %s", fmt_args![argument.as_c_str()]) };
            } else {
                unsafe { item.error(c"invalid option: %s", fmt_args![argument.as_c_str()]) };
            }
            current_block = 16446286653754202049;
        }
    } else {
        let name = match name {
            Some(name) => name,
            None => unreachable!("option name was checked above"),
        };
        let mut value = args.argument_string(1);
        if let Some(raw) = value
            && ({
                let flag = 'F' as i32 as u_char;
                args.argument_flag_count(flag)
            }) != 0
        {
            unsafe { expanded = Some(format_single_from_target(item, raw)) };
            value = expanded.as_deref();
        }
        unsafe {
            scope = RustOptionsEngine.scope_from_name(
                args.as_args(),
                window,
                &name,
                &target,
                &mut oo,
                &mut cause,
            )
        };
        if scope == OPTIONS_TABLE_NONE {
            if ({
                let flag = 'q' as i32 as u_char;
                args.argument_flag_count(flag)
            }) != 0
            {
                current_block = 11153144165560816752;
            } else {
                let cause = cause.unwrap();
                unsafe { item.error(c"%s", fmt_args![cause.as_c_str()]) };
                current_block = 16446286653754202049;
            }
        } else {
            let store = oo.as_ref().expect("option scope was resolved");
            let has_local = store.with_entry(&name, true, |entry| entry.is_some());
            let (parent_entry, is_array) = store.with_entry(&name, false, |entry| {
                (
                    RustOptionsEngine.definition(entry),
                    entry.is_some_and(|entry| { RustOptionsEngine.is_array(entry) != 0 }),
                )
            });
            if idx != -(1 as core::ffi::c_int)
                && (name.to_bytes().first() == Some(&b'@') || !is_array)
            {
                unsafe { item.error(c"not an array: %s", fmt_args![argument.as_c_str()]) };
                current_block = 16446286653754202049;
            } else {
                if ({
                    let flag = 'u' as i32 as u_char;
                    args.argument_flag_count(flag)
                }) == 0
                    && ({
                        let flag = 'o' as i32 as u_char;
                        args.argument_flag_count(flag)
                    }) != 0
                {
                    if idx == -(1 as core::ffi::c_int) {
                        already = has_local as core::ffi::c_int;
                    } else if !has_local {
                        already = 0 as core::ffi::c_int;
                    } else {
                        already = store.with_entry(&name, true, |entry| {
                            entry.is_some_and(|entry| {
                                RustOptionsEngine.array_get(entry, idx as u_int).is_some()
                            })
                        }) as core::ffi::c_int;
                    }
                    if already != 0 {
                        if ({
                            let flag = 'q' as i32 as u_char;
                            args.argument_flag_count(flag)
                        }) != 0
                        {
                            current_block = 11153144165560816752;
                        } else {
                            unsafe {
                                item.error(c"already set: %s", fmt_args![argument.as_c_str()])
                            };
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
                        if ({
                            let flag = 'U' as i32 as u_char;
                            args.argument_flag_count(flag)
                        }) != 0
                            && scope == OPTIONS_TABLE_WINDOW
                        {
                            let window = target.window().expect("window option scope has a window");
                            let pane_options: Vec<_> = unsafe {
                                window
                                    .panes()
                                    .iter()
                                    .filter_map(|pane| pane.options())
                                    .collect()
                            };
                            current_block = 1356832168064818221;
                            for options in pane_options {
                                if unsafe {
                                    RustOptionsEngine.remove_or_default(
                                        &options,
                                        &name,
                                        idx,
                                        &mut array_cause,
                                    ) != 0
                                } {
                                    unsafe {
                                        item.error(
                                            c"%s",
                                            fmt_args![array_cause.as_ref().unwrap().as_c_str()],
                                        )
                                    };
                                    current_block = 16446286653754202049;
                                    break;
                                }
                            }
                        } else {
                            current_block = 1356832168064818221;
                        }
                        match current_block {
                            16446286653754202049 => {}
                            _ => {
                                if ({
                                    let flag = 'u' as i32 as u_char;
                                    args.argument_flag_count(flag)
                                }) != 0
                                    || ({
                                        let flag = 'U' as i32 as u_char;
                                        args.argument_flag_count(flag)
                                    }) != 0
                                {
                                    if !has_local {
                                        current_block = 11153144165560816752;
                                    } else if unsafe {
                                        RustOptionsEngine.remove_or_default(
                                            oo.as_ref().expect("option scope was resolved"),
                                            &name,
                                            idx,
                                            &mut array_cause,
                                        ) != 0 as core::ffi::c_int
                                    } {
                                        unsafe {
                                            item.error(
                                                c"%s",
                                                fmt_args![array_cause.as_ref().unwrap().as_c_str()],
                                            )
                                        };
                                        current_block = 16446286653754202049;
                                    } else {
                                        current_block = 15462640364611497761;
                                    }
                                } else if name.to_bytes().first() == Some(&b'@') {
                                    if value.is_none() {
                                        unsafe { item.error(c"empty value", fmt_args![]) };
                                        current_block = 16446286653754202049;
                                    } else {
                                        unsafe {
                                            (oo.as_ref().expect("option scope was resolved"))
                                                .set_string(&name, append, c"%s", fmt_args![value])
                                        };
                                        current_block = 15462640364611497761;
                                    }
                                } else if idx == -(1 as core::ffi::c_int) && !is_array {
                                    unsafe {
                                        error = (oo.as_ref().expect("option scope was resolved"))
                                            .set_from_string(
                                                parent_entry,
                                                parent_entry.unwrap().name,
                                                value,
                                                {
                                                    let flag = 'a' as i32 as u_char;
                                                    args.argument_flag_count(flag)
                                                },
                                                &mut option_cause,
                                            )
                                    };
                                    if error != 0 as core::ffi::c_int {
                                        if let Some(cause) = option_cause.as_ref() {
                                            unsafe {
                                                item.error(c"%s", fmt_args![cause.as_c_str()])
                                            };
                                        }
                                        current_block = 16446286653754202049;
                                    } else {
                                        current_block = 15462640364611497761;
                                    }
                                } else if value.is_none() {
                                    unsafe { item.error(c"empty value", fmt_args![]) };
                                    current_block = 16446286653754202049;
                                } else {
                                    if !has_local {
                                        {
                                            store.insert_empty(
                                                parent_entry.expect("array has a definition"),
                                            )
                                        };
                                    }
                                    let result = store.with_entry_mut(&name, true, |entry| {
                                        let entry = entry.expect("local array was initialized");
                                        if idx == -1 {
                                            if append == 0 {
                                                RustOptionsEngine.array_clear(entry);
                                            }
                                            unsafe {
                                                RustOptionsEngine.array_assign(
                                                    entry,
                                                    value,
                                                    &mut array_cause,
                                                )
                                            }
                                        } else {
                                            unsafe {
                                                RustOptionsEngine.array_set(
                                                    entry,
                                                    idx as u_int,
                                                    value,
                                                    append,
                                                    &mut array_cause,
                                                )
                                            }
                                        }
                                    });
                                    if result != 0 {
                                        unsafe {
                                            item.error(
                                                c"%s",
                                                fmt_args![array_cause.as_ref().unwrap().as_c_str()],
                                            )
                                        };
                                        current_block = 16446286653754202049;
                                    } else {
                                        current_block = 15462640364611497761;
                                    }
                                }
                                match current_block {
                                    16446286653754202049 => {}
                                    11153144165560816752 => {}
                                    _ => {
                                        unsafe { RustOptionsEngine.push_changes(&name) };
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
