use crate::src::arguments::{args_get, args_has, args_make_commands_now};
use crate::src::cmd::queue::{
    cmdq_append, cmdq_continue, cmdq_error, cmdq_get_client, cmdq_get_command, cmdq_get_state,
    cmdq_get_target, cmdq_get_target_client, cmdq_insert_after,
};
use crate::src::cmd::{cmd_get_args, cmd_get_entry, cmd_list_first, cmd_list_free};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::client::CLIENT_DEAD;
use crate::src::shared::command::CMD_CLIENT_TFLAG;
use crate::src::shared::command::*;
use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_state,
};
use crate::src::shared::prompt::*;
use crate::src::shared::prompt::{prompt_result, PROMPT_CLOSE, PROMPT_SINGLE};
use crate::src::status::status_prompt_set;
use std::ffi::{CStr, CString};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_confirm_before_data {
    pub item: *mut cmdq_item,
    pub cmdlist: *mut cmd_list,
    pub confirm_key: u_char,
    pub default_yes: ::core::ffi::c_int,
}
pub static mut cmd_confirm_before_entry: cmd_entry =  {
    cmd_entry {
        name: c"confirm-before",
        alias: Some(c"confirm"),
        args: args_parse {
            template: b"bc:p:t:y\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: Some(cmd_confirm_before_args_parse),
        },
        usage: c"[-by] [-c confirm-key] [-p prompt] [-t target-client] command",
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
        flags: CMD_CLIENT_TFLAG,
        exec: Some(cmd_confirm_before_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
fn cmd_confirm_before_args_parse(
    _args: &mut args,
    _idx: u_int,
) -> Result<args_parse_type, ArgsParseError> {
    Ok(ARGS_PARSE_COMMANDS_OR_STRING)
}
unsafe fn cmd_confirm_before_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut confirm_key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut prompt: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut wait: ::core::ffi::c_int =
        (args_has(args, 'b' as i32 as u_char) == 0) as ::core::ffi::c_int;
    let mut cdata = Box::new(cmd_confirm_before_data {
        item: ::core::ptr::null_mut(),
        cmdlist: ::core::ptr::null_mut(),
        confirm_key: 0,
        default_yes: 0,
    });
    cdata.cmdlist = args_make_commands_now(self_0, item, 0 as u_int, 1 as ::core::ffi::c_int);
    if cdata.cmdlist.is_null() {
        return CMD_RETURN_ERROR;
    }
    if wait != 0 {
        cdata.item = item;
    }
    cdata.default_yes = args_has(args, 'y' as i32 as u_char);
    confirm_key = args_get(args, 'c' as i32 as u_char);
    if !confirm_key.is_null() {
        if *confirm_key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '\0' as i32
            && *confirm_key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 31 as ::core::ffi::c_int
            && (*confirm_key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 127 as ::core::ffi::c_int
        {
            cdata.confirm_key = *confirm_key.offset(0 as ::core::ffi::c_int as isize) as u_char;
        } else {
            cmdq_error(
                item,
                b"invalid confirm key\0" as *const u8 as *const ::core::ffi::c_char,
            );
            cmd_list_free(cdata.cmdlist);
            return CMD_RETURN_ERROR;
        }
    } else {
        cdata.confirm_key = 'y' as i32 as u_char;
    }
    prompt = args_get(args, 'p' as i32 as u_char);
    let new_prompt = if !prompt.is_null() {
        let mut bytes = CStr::from_ptr(prompt).to_bytes().to_vec();
        bytes.push(b' ');
        CString::new(bytes).expect("C string prompt contains no interior NUL")
    } else {
        cmd = (*cmd_get_entry(cmd_list_first(cdata.cmdlist))).name.as_ptr();
        let mut bytes = b"Confirm '".to_vec();
        bytes.extend_from_slice(CStr::from_ptr(cmd).to_bytes());
        bytes.extend_from_slice(b"'? (");
        bytes.push(cdata.confirm_key);
        bytes.extend_from_slice(b"/n) ");
        CString::new(bytes).expect("C string command and validated key contain no interior NUL")
    };
    let cdata = Box::into_raw(cdata);
    let inputcb: crate::src::shared::status::status_prompt_input_cb =
        Some(Box::new(move |c, s, _key| unsafe {
            cmd_confirm_before_callback(
                c.map_or(::core::ptr::null_mut(), std::ptr::NonNull::as_ptr),
                cdata,
                s,
            )
        }));
    let freecb: prompt_free_cb = Some(Box::new(move || unsafe { cmd_confirm_before_free(cdata) }));
    status_prompt_set(
        tc,
        target,
        new_prompt.as_ptr(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        inputcb,
        freecb,
        PROMPT_SINGLE,
        PROMPT_TYPE_COMMAND,
    );
    drop(new_prompt);
    if wait == 0 {
        return CMD_RETURN_NORMAL;
    }
    return CMD_RETURN_WAIT;
}
unsafe fn cmd_confirm_before_callback(
    mut c: *mut client,
    mut cdata: *mut cmd_confirm_before_data,
    s: Option<&CStr>,
) -> prompt_result {
    let mut item: *mut cmdq_item = (*cdata).item;
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut retcode: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    if !((*c).flags & CLIENT_DEAD as uint64_t != 0) {
        if let Some(s) = s {
            let bytes = s.to_bytes();
            let confirmed = bytes.len() == 1
                && (bytes[0] == (*cdata).confirm_key
                    || (bytes[0] == b'\r' && (*cdata).default_yes != 0));
            if confirmed {
                retcode = 0 as ::core::ffi::c_int;
                if item.is_null() {
                    new_item =
                        cmdq_get_command((*cdata).cmdlist, ::core::ptr::null_mut::<cmdq_state>());
                    cmdq_append(c, new_item);
                } else {
                    new_item = cmdq_get_command((*cdata).cmdlist, cmdq_get_state(item));
                    cmdq_insert_after(item, new_item);
                }
            }
        }
    }
    if !item.is_null() {
        if !cmdq_get_client(item).is_null() && (*cmdq_get_client(item)).session.is_null() {
            (*cmdq_get_client(item)).retval = retcode;
        }
        cmdq_continue(item);
    }
    return PROMPT_CLOSE;
}
unsafe fn cmd_confirm_before_free(mut data: *mut cmd_confirm_before_data) {
    let cdata = Box::from_raw(data);
    cmd_list_free(cdata.cmdlist);
}
