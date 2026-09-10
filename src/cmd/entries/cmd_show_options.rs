use crate::args::RustArguments;
use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::consts::{
    CMD_AFTERHOOK, CMD_FIND_CANFAIL, CMD_FIND_PANE, CMD_FIND_WINDOW, CMD_RETURN_ERROR,
    CMD_RETURN_NORMAL, OPTIONS_TABLE_IS_HOOK, OPTIONS_TABLE_NONE,
};
use crate::fmt_args;
use crate::format::format_single_from_target;
use crate::options::{OptionsEngine, RustOptionsEngine};
use crate::types::{OptionsRef, RustOptionsRef, args_parse_t, options_entry, u_char, u_int};
use crate::xmalloc::xasprintf;
use crate::{ArgumentTextCodec, RustArgumentTextCodec};
use ::std::ffi::CString;

pub(crate) static cmd_show_options_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"show-options",
        alias: Some(c"show"),
        args: args_parse_t {
            template: c"AgHpqst:vw",
            lower: 0 as core::ffi::c_int,
            upper: 1 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-AgHpqsvw] [-t target-pane] [option]",
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
        exec: cmd_show_options_exec,
    }
};
pub(crate) static cmd_show_window_options_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"show-window-options",
        alias: Some(c"showw"),
        args: args_parse_t {
            template: c"gvt:",
            lower: 0 as core::ffi::c_int,
            upper: 1 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-gv] [-t target-window] [option]",
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
        exec: cmd_show_options_exec,
    }
};
pub(crate) static cmd_show_hooks_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"show-hooks",
        alias: None,
        args: args_parse_t {
            template: c"gpt:w",
            lower: 0 as core::ffi::c_int,
            upper: 1 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-gpw] [-t target-pane] [hook]",
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
        exec: cmd_show_options_exec,
    }
};
unsafe fn cmd_show_options_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let current_block: u64;
    let args: &RustArguments = cmd_get_args(self_0);
    let target = item.target.clone();
    let mut oo: Option<RustOptionsRef> = None;
    let mut cause: Option<CString> = None;

    let mut idx: core::ffi::c_int = 0;
    let mut ambiguous: core::ffi::c_int = 0;
    let scope: core::ffi::c_int;
    let window: core::ffi::c_int =
        core::ptr::eq(cmd_get_entry(self_0), &cmd_show_window_options_entry) as core::ffi::c_int;
    if args.argument_count() == 0 as u_int {
        unsafe {
            scope = RustOptionsEngine.scope_from_flags(
                args.as_args(),
                window,
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
                return CMD_RETURN_NORMAL;
            }
            let cause = cause.unwrap();
            unsafe { item.error(c"%s", fmt_args![cause.as_c_str()]) };
            return CMD_RETURN_ERROR;
        }
        return unsafe {
            cmd_show_options_all(
                self_0,
                item,
                scope,
                oo.as_ref().expect("option scope was resolved"),
            )
        };
    }
    let argument = unsafe {
        format_single_from_target(
            item,
            args.argument_string(0).expect("argument count checked"),
        )
    };
    let name = RustOptionsEngine.match_name(&argument, &mut idx, &mut ambiguous);
    if name.is_none() {
        if ({
            let flag = 'q' as i32 as u_char;
            args.argument_flag_count(flag)
        }) != 0
        {
            current_block = 14351605340212681318;
        } else {
            if ambiguous != 0 {
                unsafe { item.error(c"ambiguous option: %s", fmt_args![argument.as_c_str()]) };
            } else {
                unsafe { item.error(c"invalid option: %s", fmt_args![argument.as_c_str()]) };
            }
            current_block = 648366277789593999;
        }
    } else {
        let name = match name {
            Some(name) => name,
            None => unreachable!("option name was checked above"),
        };
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
                current_block = 14351605340212681318;
            } else {
                let cause = cause.unwrap();
                unsafe { item.error(c"%s", fmt_args![cause.as_c_str()]) };
                current_block = 648366277789593999;
            }
        } else {
            let oo = oo.as_ref().expect("option scope was resolved");
            let shown = oo.with_entry(&name, args.argument_flag_count(b'A') == 0, |entry| {
                let Some(entry) = entry else {
                    return false;
                };
                let parent =
                    { (!RustOptionsEngine.owner(entry).ptr_eq(oo)) as core::ffi::c_int };
                unsafe { cmd_show_options_print(self_0, item, entry, idx, parent) };
                true
            });
            if shown {
                current_block = 14351605340212681318;
            } else if name.to_bytes().first() == Some(&b'@') {
                if ({
                    let flag = 'q' as i32 as u_char;
                    args.argument_flag_count(flag)
                }) != 0
                {
                    current_block = 14351605340212681318;
                } else {
                    unsafe { item.error(c"invalid option: %s", fmt_args![argument.as_c_str()]) };
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
unsafe fn cmd_show_options_print(
    self_0: &cmd,
    item: &cmdq_item,
    entry: &options_entry,
    index: core::ffi::c_int,
    parent: core::ffi::c_int,
) {
    unsafe {
        let args = cmd_get_args(self_0);
        let name = RustOptionsEngine.name(entry);
        if index == -1 && RustOptionsEngine.is_array(entry) != 0 {
            let indices = RustOptionsEngine.array_indices(entry);
            if indices.is_empty() && args.argument_flag_count(b'v') == 0 {
                item.print(c"%s", fmt_args![name]);
            }
            for index in indices {
                cmd_show_options_print(self_0, item, entry, index as core::ffi::c_int, parent);
            }
            return;
        }
        let indexed_name = (index != -1).then(|| xasprintf(c"%s[%d]", fmt_args![name, index]));
        let name = indexed_name.as_deref().unwrap_or(name);
        let value = RustOptionsEngine.display(entry, index, 0);
        if args.argument_flag_count(b'v') != 0 {
            item.print(c"%s", fmt_args![value.as_c_str()]);
        } else if RustOptionsEngine.is_string(entry) != 0 {
            let escaped = RustArgumentTextCodec.escape(&value);
            if parent != 0 {
                item.print(c"%s* %s", fmt_args![name, escaped.as_c_str()]);
            } else {
                item.print(c"%s %s", fmt_args![name, escaped.as_c_str()]);
            }
        } else if parent != 0 {
            item.print(c"%s* %s", fmt_args![name, value.as_c_str()]);
        } else {
            item.print(c"%s %s", fmt_args![name, value.as_c_str()]);
        }
    }
}
unsafe fn cmd_show_options_all(
    self_0: &cmd,
    item: &cmdq_item,
    scope: core::ffi::c_int,
    oo: &RustOptionsRef,
) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let hooks = core::ptr::eq(cmd_get_entry(self_0), &cmd_show_hooks_entry);
        if !hooks {
            for name in oo.local_names() {
                oo.with_entry(&name, true, |entry| {
                    if let Some(entry) = entry
                        && RustOptionsEngine.definition(Some(entry)).is_none()
                    {
                        cmd_show_options_print(self_0, item, entry, -1, 0);
                    }
                });
            }
        }
        for definition in RustOptionsEngine.table() {
            if definition.scope & scope == 0 {
                continue;
            }
            let is_hook = definition.flags & OPTIONS_TABLE_IS_HOOK != 0;
            if (hooks && !is_hook) || (!hooks && args.argument_flag_count(b'H') == 0 && is_hook) {
                continue;
            }
            oo.with_entry(
                definition.name,
                args.argument_flag_count(b'A') == 0,
                |entry| {
                    let Some(entry) = entry else {
                        return;
                    };
                    let parent = (!RustOptionsEngine.owner(entry).ptr_eq(oo)) as core::ffi::c_int;
                    if RustOptionsEngine.is_array(entry) == 0 {
                        cmd_show_options_print(self_0, item, entry, -1, parent);
                        return;
                    }
                    let indices = RustOptionsEngine.array_indices(entry);
                    if indices.is_empty() && args.argument_flag_count(b'v') == 0 {
                        let name = RustOptionsEngine.name(entry);
                        if parent != 0 {
                            item.print(c"%s*", fmt_args![name]);
                        } else {
                            item.print(c"%s", fmt_args![name]);
                        }
                    }
                    for index in indices {
                        cmd_show_options_print(
                            self_0,
                            item,
                            entry,
                            index as core::ffi::c_int,
                            parent,
                        );
                    }
                },
            );
        }
        CMD_RETURN_NORMAL
    }
}
