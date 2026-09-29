use crate::src::arguments::{args_count, args_has, args_string};
use crate::src::cmd::queue::{cmdq_error, cmdq_get_target_client};
use crate::src::cmd::{cmd_get_args_mut, cmd_get_entry};
use crate::src::format::bytes::write_cstr;
use crate::src::shared::arguments::args_parse;
use crate::src::shared::command::*;

pub static cmd_select_layout_entry: cmd_entry = {
    cmd_entry {
        name: c"select-layout",
        alias: Some(c"selectl"),
        args: args_parse {
            template: c"Enopt:",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-Enop] [-t target-pane] [layout-name]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_select_layout_exec),
    }
};
pub static cmd_next_layout_entry: cmd_entry = {
    cmd_entry {
        name: c"next-layout",
        alias: Some(c"nextl"),
        args: args_parse {
            template: c"t:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_WINDOW_USAGE,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_select_layout_exec),
    }
};
pub static cmd_previous_layout_entry: cmd_entry = {
    cmd_entry {
        name: c"previous-layout",
        alias: Some(c"prevl"),
        args: args_parse {
            template: c"t:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_WINDOW_USAGE,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_select_layout_exec),
    }
};
unsafe fn cmd_select_layout_exec(mut command: refbox::Weak<cmd>, item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>) -> cmd_retval {
    use crate::src::server_client::Client;
    use crate::src::window::Window;
    let is_next = std::ptr::eq(cmd_get_entry(command.get_unchecked()), &cmd_next_layout_entry);
    let is_previous = std::ptr::eq(cmd_get_entry(command.get_unchecked()), &cmd_previous_layout_entry);
    let arguments = cmd_get_args_mut(command.get_mut_unchecked()).expect("layout arguments");
    let target = &*crate::src::cmd::queue::cmdq_get_target_mut(&mut *item_handle.get());
    let window = target.winlink_handle().get_unchecked().window_handle().expect("target window").clone();
    let client = cmdq_get_target_client(Some(&*item_handle.get()));
    let next = is_next || args_has(arguments, b'n') != 0;
    let previous = is_previous || args_has(arguments, b'p') != 0;
    let cycle = if next { 1 } else if previous { -1 } else { 0 };
    let spread = if args_has(arguments, b'E') != 0 { target.pane_handle() } else { None };
    let restore = args_has(arguments, b'o') != 0;
    let name = if args_count(arguments) != 0 { args_string(arguments, 0) } else { None };
    let legacy = client.as_ref().is_some_and(|client| client.uses_legacy_layout_format());
    let result = window.select_layout(name, restore, cycle, spread.as_ref(), legacy);
    window.release(c"cmd_select_layout");
    match result {
        Ok(()) => CMD_RETURN_NORMAL,
        Err(cause) => {
            cmdq_error(item_handle, |out| write_cstr(out, cause.as_ptr()));
            CMD_RETURN_ERROR
        }
    }
}
