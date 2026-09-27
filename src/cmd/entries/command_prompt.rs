use crate::src::arguments::{
    args_count, args_get, args_has, args_make_commands, args_make_commands_get_command_cstring,
    args_make_commands_prepare,
};
use crate::src::cmd::queue::{
    cmdq_append, cmdq_continue, cmdq_error, cmdq_get_command, cmdq_get_error, cmdq_get_state,
    cmdq_get_target, cmdq_get_target_client, cmdq_insert_after,
};
use crate::src::cmd::{cmd_append_argv, cmd_get_args_mut};
use crate::src::format::bytes::write_cstr;
use crate::src::prompt::prompt_type;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args_command_state;
use crate::src::shared::arguments::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::command::CMD_CLIENT_TFLAG;
use crate::src::shared::command::*;
use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item, cmdq_state,
};
use crate::src::shared::pane::window_pane;
use crate::src::shared::prompt::*;
use crate::src::shared::prompt::{
    prompt_result, PROMPT_BSPACE_EXIT, PROMPT_CLOSE, PROMPT_CONTINUE, PROMPT_INCREMENTAL,
    PROMPT_ISPANE, PROMPT_KEY, PROMPT_NOFREEZE, PROMPT_NUMERIC, PROMPT_SINGLE,
};
use crate::src::shared::rc;
use crate::src::status::{status_prompt_set, status_prompt_update};
use crate::src::window::{
    window_pane_has_prompt, window_pane_set_prompt, window_pane_update_prompt,
    window_pane_upgrade, window_pane_weak,
};
use std::cell::UnsafeCell;
use std::ffi::{CStr, CString};
use std::rc::Weak;

pub struct cmd_command_prompt_cdata {
    pub item: *mut cmdq_item,
    pub state: Option<Box<args_command_state>>,
    pub flags: ::core::ffi::c_int,
    pub prompt_type: prompt_type,
    pub wp: Option<Weak<UnsafeCell<window_pane>>>,
    pub prompts: Vec<cmd_command_prompt_prompt>,
    pub current: u_int,
    pub argv: Vec<CString>,
}
pub struct cmd_command_prompt_prompt {
    pub input: Option<CString>,
    pub prompt: CString,
}

impl cmd_command_prompt_prompt {
    fn input_ptr(&self) -> *const ::core::ffi::c_char {
        self.input
            .as_ref()
            .map_or(::core::ptr::null(), |s| s.as_ptr())
    }

    fn pointers(&self) -> (*const ::core::ffi::c_char, *const ::core::ffi::c_char) {
        (self.prompt.as_ptr(), self.input_ptr())
    }
}

fn cmd_command_prompt_rows(
    prompts: &[u8],
    inputs: Option<&[u8]>,
    literal: bool,
    space: bool,
) -> Vec<cmd_command_prompt_prompt> {
    if literal {
        return vec![cmd_command_prompt_prompt {
            prompt: CString::new(prompts).expect("prompt has no embedded NUL"),
            input: inputs.map(|s| CString::new(s).expect("input has no embedded NUL")),
        }];
    }

    let mut input_parts = inputs.map(|s| s.split(|&byte| byte == b','));
    prompts
        .split(|&byte| byte == b',')
        .map(|part| {
            let mut prompt = part.to_vec();
            if space {
                prompt.push(b' ');
            }
            let input = input_parts.as_mut().and_then(Iterator::next).unwrap_or(b"");
            cmd_command_prompt_prompt {
                prompt: CString::new(prompt).expect("prompt has no embedded NUL"),
                input: Some(CString::new(input).expect("input has no embedded NUL")),
            }
        })
        .collect()
}
pub static cmd_command_prompt_entry: cmd_entry = {
    cmd_entry {
        name: c"command-prompt",
        alias: None,
        args: args_parse {
            template: c"1CbeFiklI:NPp:t:T:",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: Some(cmd_command_prompt_args_parse),
        },
        usage:
            c"[-1CbeFiklNP] [-I inputs] [-p prompts] [-t target-client] [-T prompt-type] [template]",
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
        exec: Some(cmd_command_prompt_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
fn cmd_command_prompt_args_parse(
    _args: &mut args,
    _idx: u_int,
) -> Result<args_parse_type, ArgsParseError> {
    Ok(ARGS_PARSE_COMMANDS_OR_STRING)
}
unsafe fn cmd_command_prompt_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut type_0: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut prompt_bytes = Vec::<u8>::new();
    let mut wp: *mut window_pane = (*target).wp;
    let mut count: u_int = args_count(args);
    let mut wait: ::core::ffi::c_int =
        (args_has(args, 'b' as i32 as u_char) == 0) as ::core::ffi::c_int;
    let mut space: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut pane: ::core::ffi::c_int = args_has(args, 'P' as i32 as u_char);
    if pane != 0 {
        if wp.is_null() || window_pane_has_prompt(wp) != 0 {
            return CMD_RETURN_NORMAL;
        }
    } else if (*tc).prompt.is_some() {
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'i' as i32 as u_char) != 0 {
        wait = 0 as ::core::ffi::c_int;
    }
    let mut cdata = Box::new(cmd_command_prompt_cdata {
        item: ::core::ptr::null_mut(),
        state: None,
        flags: 0,
        prompt_type: PROMPT_TYPE_COMMAND,
        wp: None,
        prompts: Vec::new(),
        current: 0,
        argv: Vec::new(),
    });
    if wait != 0 {
        cdata.item = item;
    }
    if pane != 0 {
        cdata.wp = Some(window_pane_weak(wp));
    }
    cdata.state = Some(args_make_commands_prepare(
        self_0,
        item,
        0 as u_int,
        b"%1\0" as *const u8 as *const ::core::ffi::c_char,
        wait,
        args_has(args, 'F' as i32 as u_char),
    ));
    let literal = args_has(args, 'l' as i32 as u_char) != 0;
    s = args_get(args, 'p' as i32 as u_char);
    if s.is_null() {
        if count != 0 as u_int {
            let command = args_make_commands_get_command_cstring(
                cdata.state.as_deref().expect("prepared command state"),
            );
            prompt_bytes.push(b'(');
            prompt_bytes.extend_from_slice(command.as_bytes());
            prompt_bytes.push(b')');
        } else {
            prompt_bytes.push(b':');
            space = 0;
        }
    } else {
        prompt_bytes.extend_from_slice(CStr::from_ptr(s).to_bytes());
    }
    s = args_get(args, 'I' as i32 as u_char);
    let input_bytes = if s.is_null() {
        None
    } else {
        Some(CStr::from_ptr(s).to_bytes())
    };
    cdata.prompts = cmd_command_prompt_rows(&prompt_bytes, input_bytes, literal, space != 0);
    type_0 = args_get(args, 'T' as i32 as u_char);
    if !type_0.is_null() {
        cdata.prompt_type = prompt_type(CStr::from_ptr(type_0));
        if cdata.prompt_type as ::core::ffi::c_uint
            == PROMPT_TYPE_INVALID as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            cmdq_error(item, |out| {
                out.write_all(b"unknown type: ")?;
                write_cstr(out, type_0)
            });
            return CMD_RETURN_ERROR;
        }
    } else {
        cdata.prompt_type = PROMPT_TYPE_COMMAND;
    }
    if args_has(args, '1' as i32 as u_char) != 0 {
        cdata.flags |= PROMPT_SINGLE;
    } else if args_has(args, 'N' as i32 as u_char) != 0 {
        cdata.flags |= PROMPT_NUMERIC;
    } else if args_has(args, 'i' as i32 as u_char) != 0 {
        cdata.flags |= PROMPT_INCREMENTAL;
    } else if args_has(args, 'k' as i32 as u_char) != 0 {
        cdata.flags |= PROMPT_KEY;
    } else if args_has(args, 'e' as i32 as u_char) != 0 {
        cdata.flags |= PROMPT_BSPACE_EXIT;
    }
    if args_has(args, 'C' as i32 as u_char) != 0 {
        cdata.flags |= PROMPT_NOFREEZE;
    }
    if pane != 0 {
        cdata.flags |= PROMPT_ISPANE;
    }
    let (prompt_ptr, input_ptr) = cdata.prompts[0].pointers();
    let flags = cdata.flags;
    let prompt_type = cdata.prompt_type;
    let inputcb = cdata.into_callback();
    if pane != 0 {
        window_pane_set_prompt(
            wp,
            tc,
            target,
            prompt_ptr,
            input_ptr,
            inputcb,
            None,
            flags,
            prompt_type,
        );
    } else {
        status_prompt_set(
            tc,
            target,
            prompt_ptr,
            input_ptr,
            inputcb,
            None,
            flags,
            prompt_type,
        );
    }
    if wait == 0 {
        return CMD_RETURN_NORMAL;
    }
    return CMD_RETURN_WAIT;
}
unsafe fn cmd_command_prompt_callback(
    mut c: *mut client,
    cdata: &mut cmd_command_prompt_cdata,
    mut s: Option<&CStr>,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut current_block: u64;
    let mut item: *mut cmdq_item = cdata.item;
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    if !(s.is_none()
        || key as ::core::ffi::c_uint
            == PROMPT_KEY_MOVE as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        if key as ::core::ffi::c_uint
            == PROMPT_KEY_CLOSE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if cdata.flags & PROMPT_INCREMENTAL != 0 {
                current_block = 11745758271394821990;
            } else {
                cmd_append_argv(&mut cdata.argv, s.expect("prompt text is present"));
                cdata.current = cdata.current.wrapping_add(1);
                if (cdata.current as usize) != cdata.prompts.len() {
                    let (prompt_ptr, input_ptr) =
                        (&cdata.prompts)[cdata.current as usize].pointers();
                    if let Some(pane) = &cdata.wp {
                        let Some(pane) = window_pane_upgrade(pane) else {
                            return PROMPT_CLOSE;
                        };
                        window_pane_update_prompt(rc::as_ptr(&pane), prompt_ptr, input_ptr);
                    } else {
                        status_prompt_update(c, prompt_ptr, input_ptr);
                    }
                    return PROMPT_CONTINUE;
                }
                current_block = 1841672684692190573;
            }
        } else {
            current_block = 1841672684692190573;
        }
        match current_block {
            11745758271394821990 => {}
            _ => {
                let mut argv_owner = cdata.argv.clone();
                if key as ::core::ffi::c_uint
                    != PROMPT_KEY_CLOSE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    cmd_append_argv(&mut argv_owner, s.expect("prompt text is present"));
                } else {
                    cdata.argv = argv_owner.clone();
                }
                match args_make_commands(
                    cdata.state.as_deref_mut().expect("prepared command state"),
                    &argv_owner,
                ) {
                    Err(error) => {
                        cmdq_append(
                            c,
                            cmdq_get_error(
                                error
                                    .as_ref()
                                    .map_or(::core::ptr::null(), |cause| cause.as_ptr()),
                            ),
                        );
                    }
                    Ok(cmdlist) if item.is_null() => {
                        new_item = cmdq_get_command(
                            &cmdlist, None,
                        );
                        cmdq_append(c, new_item);
                        drop(cmdlist);
                    }
                    Ok(cmdlist) => {
                        new_item = cmdq_get_command(&cmdlist, (*item).state.as_ref());
                        cmdq_insert_after(item, new_item);
                        drop(cmdlist);
                    }
                }
                if cdata.flags & PROMPT_INCREMENTAL != 0 {
                    return PROMPT_CONTINUE;
                }
            }
        }
    }
    if !item.is_null() {
        cdata.item = ::core::ptr::null_mut::<cmdq_item>();
        cmdq_continue(item);
    }
    return PROMPT_CLOSE;
}
impl cmd_command_prompt_cdata {
    fn into_callback(mut self: Box<Self>) -> crate::src::shared::status::status_prompt_input_cb {
        Some(Box::new(move |client, text, key| unsafe {
            cmd_command_prompt_callback(
                client.map_or(std::ptr::null_mut(), std::ptr::NonNull::as_ptr),
                &mut self,
                text,
                key,
            )
        }))
    }
}

impl Drop for cmd_command_prompt_cdata {
    fn drop(&mut self) {
        unsafe {
            if !self.item.is_null() {
                cmdq_continue(self.item);
            }
            self.prompts.clear();
            drop(self.state.take());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::src::cmd::cmd_list_new;
    use crate::src::prompt::{prompt_free, prompt_key};
    use crate::src::shared::rc;
    use crate::src::text::utf8::utf8_fromcstr_vec;
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn owned_callback_releases_waiting_command_and_state_on_every_close_path() {
        unsafe {
            for close_path in 0..3 {
                let mut item = cmdq_item::empty();
                item.flags = CMDQ_WAITING;
                let cmdlist = cmd_list_new();
                let commands = std::rc::Rc::downgrade(&cmdlist);
                let mut state = Box::new(args_command_state::empty());
                state.cmdlist = Some(cmdlist);
                let data = Box::new(cmd_command_prompt_cdata {
                    item: &mut item,
                    state: Some(state),
                    flags: 0,
                    prompt_type: PROMPT_TYPE_COMMAND,
                    wp: None,
                    prompts: cmd_command_prompt_rows(b"owner:", None, false, true),
                    current: 0,
                    argv: Vec::new(),
                });
                let owner = refbox::RefBox::new(prompt {
                    flags: 0,
                    buffer: utf8_fromcstr_vec(c""),
                    ..Default::default()
                });
                let active = owner.downgrade();
                let observed = commands.clone();
                let mut callback = data.into_callback().unwrap();
                owner.try_borrow_mut().unwrap().inputcb = Some(Box::new(move |text, key| {
                    let result = callback(None, text, key);
                    if close_path == 2 {
                        prompt_free(&active);
                        assert!(observed.upgrade().is_some());
                    }
                    result
                }));
                if close_path != 0 {
                    assert_eq!(prompt_key(&owner.downgrade(), 27, &mut 0), PROMPT_KEY_CLOSE);
                    assert_eq!(item.flags & CMDQ_WAITING, 0);
                }
                prompt_free(&owner.downgrade());
                assert_eq!(item.flags & CMDQ_WAITING, 0);
                assert!(commands.upgrade().is_none());
                assert!(owner.try_borrow_mut().unwrap().inputcb.is_none());
                // Retaining the closed prompt does not retain its callback record.
                prompt_free(&owner.downgrade());
            }
        }
    }

    #[test]
    fn split_prompts_keep_empty_fields_and_fill_missing_inputs() {
        let rows = cmd_command_prompt_rows(b"first,,last,", Some(b"one,,three"), false, true);
        let prompts: Vec<_> = rows.iter().map(|row| row.prompt.to_bytes()).collect();
        let inputs: Vec<_> = rows
            .iter()
            .map(|row| row.input.as_ref().unwrap().to_bytes())
            .collect();
        assert_eq!(prompts, [b"first ".as_slice(), b" ", b"last ", b" "]);
        assert_eq!(inputs, [b"one".as_slice(), b"", b"three", b""]);
    }

    #[test]
    fn literal_prompt_keeps_commas_and_null_input() {
        let rows = cmd_command_prompt_rows(b"first,second", None, true, true);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].prompt.to_bytes(), b"first,second");
        assert!(rows[0].input_ptr().is_null());

        let rows = cmd_command_prompt_rows(b":", Some(b""), true, false);
        assert_eq!(rows[0].prompt.to_bytes(), b":");
        assert!(!rows[0].input_ptr().is_null());
        assert_eq!(rows[0].input.as_ref().unwrap().to_bytes(), b"");
    }
}
