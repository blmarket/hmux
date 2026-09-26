use crate::src::arguments::{args_count, args_get, args_has, args_string};
use crate::src::cmd::find::cmd_find_copy_state;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_target};
use crate::src::cmd::{cmd_get_args, cmd_get_entry};
use crate::src::events::events_fire;
use crate::src::events_payload::{
    event_payload_create, event_payload_set_client, event_payload_set_int, event_payload_set_pane,
    event_payload_set_session, event_payload_set_target, event_payload_set_window,
};
use crate::src::format::format_single_from_target_cstring;
use crate::src::hooks::{hooks_add_event, hooks_monitor_add, hooks_monitor_remove, hooks_run};
use crate::src::monitor::monitor_parse_owned;
use crate::src::options::options_table_entry;
use crate::src::options::{
    options_array_assign, options_array_clear, options_array_get, options_array_set, options_empty,
    options_from_string, options_get, options_get_only, options_get_string, options_is_array,
    options_match_owned, options_push_changes, options_remove_or_default, options_scope_from_name,
    options_set_string, OptionMatchFailure,
};
use crate::src::session::sessions;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::command::{CMD_AFTERHOOK, CMD_FIND_CANFAIL};
use crate::src::shared::events::event_payload;
use crate::src::shared::monitor::{MONITOR_NOTIFY_TRUE, MONITOR_SESSION};
use crate::src::shared::options::{options, options_entry};
use crate::src::shared::options::{OPTIONS_TABLE_NONE, OPTIONS_TABLE_WINDOW};
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::window::{window, winlink};
use crate::src::tmux::{global_options, global_s_options, global_w_options};
use crate::src::window::{window_pane_first, window_pane_next};
use std::ffi::{CStr, CString};
pub static mut cmd_set_option_entry: cmd_entry = {
    cmd_entry {
        name: c"set-option",
        alias: Some(c"set"),
        args: args_parse {
            template: b"aFgopqst:uUw\0" as *const u8 as *const ::core::ffi::c_char,
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
        exec: Some(cmd_set_option_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
pub static mut cmd_set_window_option_entry: cmd_entry = {
    cmd_entry {
        name: c"set-window-option",
        alias: Some(c"setw"),
        args: args_parse {
            template: b"aFgoqt:u\0" as *const u8 as *const ::core::ffi::c_char,
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
        exec: Some(cmd_set_option_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
pub static mut cmd_set_hook_entry: cmd_entry = {
    cmd_entry {
        name: c"set-hook",
        alias: None,
        args: args_parse {
            template: b"agpERTt:uB:w\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 2 as ::core::ffi::c_int,
            cb: Some(cmd_set_option_args_parse),
        },
        usage: c"[-agpERTuw] [-B name:what:format] [-t target-pane] [hook] [command]",
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
        exec: Some(cmd_set_option_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
fn cmd_set_option_args_parse(
    args: &mut args,
    idx: u_int,
) -> Result<args_parse_type, ArgsParseError> {
    if unsafe { args_has(args as *mut args, 'B' as i32 as u_char) } != 0 {
        return Ok(ARGS_PARSE_COMMANDS_OR_STRING);
    }
    if idx == 1 as u_int {
        return Ok(ARGS_PARSE_COMMANDS_OR_STRING);
    }
    Ok(ARGS_PARSE_STRING)
}
unsafe fn cmd_set_hook_event_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if args_count(args) == 0 as u_int {
        cmdq_error(
            item,
            b"missing argument\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if args_count(args) != 1 as u_int {
        cmdq_error(
            item,
            b"too many arguments\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    let argument = format_single_from_target_cstring(item, args_string(args, 0 as u_int));
    if *argument.as_ptr() as ::core::ffi::c_int != '@' as i32 {
        cmdq_error(
            item,
            b"event name must start with @\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    ep = event_payload_create();
    event_payload_set_target(ep, target);
    c = cmdq_get_client(item);
    if !c.is_null() {
        event_payload_set_client(ep, c);
    }
    if !(*target).s.is_null() {
        event_payload_set_session(
            ep,
            b"session\0" as *const u8 as *const ::core::ffi::c_char,
            (*target).s,
        );
    }
    if !(*target).w.is_null() {
        event_payload_set_window(
            ep,
            b"window\0" as *const u8 as *const ::core::ffi::c_char,
            (*target).w,
        );
    }
    if !(*target).wl.is_null() {
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*target).wl).idx,
        );
    } else if (*target).idx != -(1 as ::core::ffi::c_int) {
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (*target).idx,
        );
    }
    if !(*target).wp.is_null() {
        event_payload_set_pane(
            ep,
            b"pane\0" as *const u8 as *const ::core::ffi::c_char,
            (*target).wp,
        );
    }
    events_fire(argument.as_ptr(), ep);
    return CMD_RETURN_NORMAL;
}
unsafe fn cmd_set_hook_monitor_exec(
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut window: ::core::ffi::c_int,
) -> cmd_retval {
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut cause: Option<CString> = None;
    let mut expanded: Option<CString> = None;
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut scope: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if args_count(args) > 1 as u_int {
        cmdq_error(
            item,
            b"too many arguments\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    value = args_get(args, 'B' as i32 as u_char);
    let unsubscribe = args_has(args, 'u' as i32 as u_char) != 0;
    let parsed = monitor_parse_owned(CStr::from_ptr(value));
    let (name_owned, type_0, id, format_owned) = if unsubscribe {
        match parsed {
            Some(parsed) => (parsed.name, parsed.type_0, parsed.id, None),
            None => (CStr::from_ptr(value).to_owned(), MONITOR_SESSION, 0, None),
        }
    } else {
        let Some(parsed) = parsed else {
            cmdq_error(
                item,
                b"invalid subscription: %s\0" as *const u8 as *const ::core::ffi::c_char,
                value,
            );
            return CMD_RETURN_ERROR;
        };
        (parsed.name, parsed.type_0, parsed.id, Some(parsed.format))
    };
    let name = name_owned.as_ptr();
    let format = format_owned
        .as_ref()
        .map_or(::core::ptr::null(), |format| format.as_ptr());
    if *name as ::core::ffi::c_int != '@' as i32 {
        cmdq_error(
            item,
            b"monitor hook name must start with @\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        scope = options_scope_from_name(args, window, name, target, &raw mut oo, &raw mut cause);
        if scope == OPTIONS_TABLE_NONE {
            cmdq_error(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                cause.as_ref().unwrap().as_ptr(),
            );
        } else {
            cmd_find_copy_state(&raw mut fs, target);
            if args_has(args, 'u' as i32 as u_char) != 0 {
                hooks_monitor_remove(oo, name);
            } else {
                if args_count(args) != 0 as u_int {
                    value = args_string(args, 0 as u_int);
                    if args_has(args, 'F' as i32 as u_char) != 0 {
                        expanded = Some(format_single_from_target_cstring(item, value));
                        value = expanded.as_ref().expect("expanded value was set").as_ptr();
                    }
                    o = options_get_only(oo, name);
                    if args_has(args, 'o' as i32 as u_char) == 0 || o.is_null() {
                        let newvalue = if args_has(args, 'a' as i32 as u_char) != 0 && !o.is_null()
                        {
                            let old = options_get_string(oo, name);
                            // glibc printf renders a null %s argument as "(null)".
                            let old_bytes = if old.is_null() {
                                b"(null)".as_slice()
                            } else {
                                CStr::from_ptr(old).to_bytes()
                            };
                            let value_bytes = CStr::from_ptr(value).to_bytes();
                            let mut bytes = Vec::with_capacity(old_bytes.len() + value_bytes.len());
                            bytes.extend_from_slice(old_bytes);
                            bytes.extend_from_slice(value_bytes);
                            Some(CString::new(bytes).expect("C-string fragments contain no NUL"))
                        } else {
                            None
                        };
                        if let Some(newvalue) = &newvalue {
                            value = newvalue.as_ptr();
                        }
                        options_set_string(
                            oo,
                            name,
                            0 as ::core::ffi::c_int,
                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                            value,
                        );
                        options_push_changes(name);
                    }
                }
                if oo != global_options && oo != global_s_options && oo != global_w_options {
                    s = (*target).s;
                }
                if args_has(args, 'T' as i32 as u_char) != 0 {
                    flags |= MONITOR_NOTIFY_TRUE;
                }
                hooks_monitor_add(oo, name, type_0, id, format, flags, &raw mut fs, s);
            }
            return CMD_RETURN_NORMAL;
        }
    }
    return CMD_RETURN_ERROR;
}
unsafe fn cmd_set_option_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut current_block: u64;
    let mut args: *mut args = cmd_get_args(self_0);
    let mut append: ::core::ffi::c_int = args_has(args, 'a' as i32 as u_char);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut loop_0: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut parent: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut po: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut cause: Option<CString> = None;
    let mut expanded: Option<CString> = None;
    let mut array_key: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut window: ::core::ffi::c_int = 0;
    let mut already: ::core::ffi::c_int = 0;
    let mut error: ::core::ffi::c_int = 0;
    let mut ambiguous: ::core::ffi::c_int = 0;
    let mut scope: ::core::ffi::c_int = 0;
    window =
        (cmd_get_entry(self_0) == &raw const cmd_set_window_option_entry) as ::core::ffi::c_int;
    if cmd_get_entry(self_0) == &raw const cmd_set_hook_entry
        && args_has(args, 'E' as i32 as u_char) != 0
    {
        return cmd_set_hook_event_exec(self_0, item);
    }
    if cmd_get_entry(self_0) == &raw const cmd_set_hook_entry
        && args_has(args, 'B' as i32 as u_char) != 0
    {
        return cmd_set_hook_monitor_exec(item, args, window);
    }
    if args_count(args) == 0 as u_int {
        cmdq_error(
            item,
            b"missing argument\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    let argument = format_single_from_target_cstring(item, args_string(args, 0 as u_int));
    if cmd_get_entry(self_0) == &raw const cmd_set_hook_entry
        && args_has(args, 'R' as i32 as u_char) != 0
    {
        hooks_run(item, argument.as_ptr());
        return CMD_RETURN_NORMAL;
    }
    let matched = options_match_owned(CStr::from_ptr(argument.as_ptr()));
    if let Ok(parsed) = &matched {
        name = parsed.name.as_ptr();
        array_key = parsed
            .array_key
            .as_ref()
            .map_or(::core::ptr::null(), |key| key.as_ptr());
    }
    // tmux leaves its ambiguity output unset on a parse failure. Match the
    // pinned oracle's diagnostic: invalid on an empty server, ambiguous when
    // sessions exist, without reading uninitialized memory.
    ambiguous = match matched {
        Err(OptionMatchFailure::Ambiguous) => true,
        Err(OptionMatchFailure::Parse) => sessions.storage.is_some(),
        _ => false,
    } as ::core::ffi::c_int;
    if name.is_null() {
        if args_has(args, 'q' as i32 as u_char) != 0 {
            current_block = 710513931074292511;
        } else {
            if ambiguous != 0 {
                cmdq_error(
                    item,
                    b"ambiguous option: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    argument.as_ptr(),
                );
            } else {
                cmdq_error(
                    item,
                    b"invalid option: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    argument.as_ptr(),
                );
            }
            current_block = 8517774764635037400;
        }
    } else {
        if args_count(args) < 2 as u_int {
            value = ::core::ptr::null::<::core::ffi::c_char>();
        } else {
            value = args_string(args, 1 as u_int);
        }
        if !value.is_null() && args_has(args, 'F' as i32 as u_char) != 0 {
            expanded = Some(format_single_from_target_cstring(item, value));
            value = expanded.as_ref().expect("expanded value was set").as_ptr();
        }
        scope = options_scope_from_name(args, window, name, target, &raw mut oo, &raw mut cause);
        if scope == OPTIONS_TABLE_NONE {
            if args_has(args, 'q' as i32 as u_char) != 0 {
                current_block = 710513931074292511;
            } else {
                cmdq_error(
                    item,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    cause.as_ref().unwrap().as_ptr(),
                );
                current_block = 8517774764635037400;
            }
        } else {
            o = options_get_only(oo, name);
            parent = options_get(oo, name);
            if !array_key.is_null()
                && (*name as ::core::ffi::c_int == '@' as i32 || options_is_array(parent) == 0)
            {
                cmdq_error(
                    item,
                    b"not an array: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    argument.as_ptr(),
                );
                current_block = 8517774764635037400;
            } else {
                if args_has(args, 'u' as i32 as u_char) == 0
                    && args_has(args, 'o' as i32 as u_char) != 0
                {
                    if array_key.is_null() {
                        already = (o != NULL as *mut options_entry) as ::core::ffi::c_int;
                    } else if o.is_null() {
                        already = 0 as ::core::ffi::c_int;
                    } else if !options_array_get(o, array_key).is_null() {
                        already = 1 as ::core::ffi::c_int;
                    } else {
                        already = 0 as ::core::ffi::c_int;
                    }
                    if already != 0 {
                        if args_has(args, 'q' as i32 as u_char) != 0 {
                            current_block = 710513931074292511;
                        } else {
                            cmdq_error(
                                item,
                                b"already set: %s\0" as *const u8 as *const ::core::ffi::c_char,
                                argument.as_ptr(),
                            );
                            current_block = 8517774764635037400;
                        }
                    } else {
                        current_block = 12997042908615822766;
                    }
                } else {
                    current_block = 12997042908615822766;
                }
                match current_block {
                    710513931074292511 => {}
                    8517774764635037400 => {}
                    _ => {
                        if args_has(args, 'U' as i32 as u_char) != 0
                            && scope == OPTIONS_TABLE_WINDOW
                        {
                            loop_0 = window_pane_first((*target).w);
                            loop {
                                if loop_0.is_null() {
                                    current_block = 10095721787123848864;
                                    break;
                                }
                                po = options_get_only((*loop_0).options, name);
                                if !po.is_null() {
                                    if options_remove_or_default(po, array_key, &raw mut cause)
                                        != 0 as ::core::ffi::c_int
                                    {
                                        cmdq_error(
                                            item,
                                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                            cause.as_ref().unwrap().as_ptr(),
                                        );
                                        current_block = 8517774764635037400;
                                        break;
                                    }
                                }
                                loop_0 = window_pane_next(loop_0);
                            }
                        } else {
                            current_block = 10095721787123848864;
                        }
                        match current_block {
                            8517774764635037400 => {}
                            _ => {
                                if args_has(args, 'u' as i32 as u_char) != 0
                                    || args_has(args, 'U' as i32 as u_char) != 0
                                {
                                    if o.is_null() {
                                        current_block = 710513931074292511;
                                    } else if options_remove_or_default(
                                        o,
                                        array_key,
                                        &raw mut cause,
                                    ) != 0 as ::core::ffi::c_int
                                    {
                                        cmdq_error(
                                            item,
                                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                            cause.as_ref().unwrap().as_ptr(),
                                        );
                                        current_block = 8517774764635037400;
                                    } else {
                                        current_block = 16231175055492490595;
                                    }
                                } else if *name as ::core::ffi::c_int == '@' as i32 {
                                    if value.is_null() {
                                        cmdq_error(
                                            item,
                                            b"empty value\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                        );
                                        current_block = 8517774764635037400;
                                    } else {
                                        options_set_string(
                                            oo,
                                            name,
                                            append,
                                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                            value,
                                        );
                                        if cmd_get_entry(self_0) == &raw const cmd_set_hook_entry {
                                            hooks_add_event(name);
                                        }
                                        current_block = 16231175055492490595;
                                    }
                                } else if array_key.is_null() && options_is_array(parent) == 0 {
                                    error = options_from_string(
                                        oo,
                                        options_table_entry(parent),
                                        (*options_table_entry(parent)).name,
                                        value,
                                        args_has(args, 'a' as i32 as u_char),
                                        &raw mut cause,
                                    );
                                    if error != 0 as ::core::ffi::c_int {
                                        cmdq_error(
                                            item,
                                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                            cause.as_ref().unwrap().as_ptr(),
                                        );
                                        current_block = 8517774764635037400;
                                    } else {
                                        current_block = 16231175055492490595;
                                    }
                                } else if value.is_null() {
                                    cmdq_error(
                                        item,
                                        b"empty value\0" as *const u8 as *const ::core::ffi::c_char,
                                    );
                                    current_block = 8517774764635037400;
                                } else {
                                    if o.is_null() {
                                        o = options_empty(oo, options_table_entry(parent));
                                    }
                                    if array_key.is_null() {
                                        if append == 0 {
                                            options_array_clear(o);
                                        }
                                        if options_array_assign(o, value, &raw mut cause)
                                            != 0 as ::core::ffi::c_int
                                        {
                                            cmdq_error(
                                                item,
                                                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                                cause.as_ref().unwrap().as_ptr(),
                                            );
                                            current_block = 8517774764635037400;
                                        } else {
                                            current_block = 16231175055492490595;
                                        }
                                    } else if options_array_set(
                                        o,
                                        array_key,
                                        value,
                                        append,
                                        &raw mut cause,
                                    ) != 0 as ::core::ffi::c_int
                                    {
                                        cmdq_error(
                                            item,
                                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                            cause.as_ref().unwrap().as_ptr(),
                                        );
                                        current_block = 8517774764635037400;
                                    } else {
                                        current_block = 16231175055492490595;
                                    }
                                }
                                match current_block {
                                    8517774764635037400 => {}
                                    710513931074292511 => {}
                                    _ => {
                                        options_push_changes(name);
                                        current_block = 710513931074292511;
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
        8517774764635037400 => {
            return CMD_RETURN_ERROR;
        }
        _ => {
            return CMD_RETURN_NORMAL;
        }
    };
}
