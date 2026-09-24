use crate::src::arguments::{
    args_count, args_get, args_has, args_make_commands, args_make_commands_free,
    args_make_commands_get_command_cstring, args_make_commands_prepare,
};
use crate::src::cmd::{cmd_append_argv, cmd_copy_argv, cmd_free_argv, cmd_get_args, cmd_list_free};
use crate::src::cmd_queue::{
    cmdq_append, cmdq_continue, cmdq_error, cmdq_get_command, cmdq_get_error, cmdq_get_state,
    cmdq_get_target, cmdq_get_target_client, cmdq_insert_after,
};
use crate::src::ffi::libc::free;
use crate::src::prompt::prompt_type;
use crate::src::shared::abi::*;
pub use crate::src::shared::arguments::args_command_state;
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
use crate::src::shared::colour::*;
pub use crate::src::shared::command::CMD_CLIENT_TFLAG;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmdq_state,
    cmds,
};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
pub use crate::src::shared::format::{format_job_tree, format_tree};
use crate::src::shared::grid::*;
pub use crate::src::shared::hyperlinks::hyperlinks;
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
use crate::src::shared::key::*;
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::options;
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
use crate::src::shared::prompt::*;
pub use crate::src::shared::prompt::{
    prompt_free_cb, prompt_result, PROMPT_BSPACE_EXIT, PROMPT_CLOSE, PROMPT_CONTINUE,
    PROMPT_INCREMENTAL, PROMPT_ISPANE, PROMPT_KEY, PROMPT_NOFREEZE, PROMPT_NUMERIC, PROMPT_SINGLE,
};
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::{status_line, status_prompt_input_cb};
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::status::{status_prompt_set, status_prompt_update};
use crate::src::window::{
    window_pane_has_prompt, window_pane_set_prompt, window_pane_update_prompt,
};
use std::ffi::{CStr, CString};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

pub struct cmd_command_prompt_cdata {
    pub item: *mut cmdq_item,
    pub state: *mut args_command_state,
    pub flags: ::core::ffi::c_int,
    pub prompt_type: prompt_type,
    pub wp: *mut window_pane,
    pub prompts: Vec<cmd_command_prompt_prompt>,
    pub current: u_int,
    pub argc: ::core::ffi::c_int,
    pub argv: *mut *mut ::core::ffi::c_char,
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
#[no_mangle]
pub static mut cmd_command_prompt_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"command-prompt\0" as *const u8 as *const ::core::ffi::c_char,
        alias: ::core::ptr::null::<::core::ffi::c_char>(),
        args: args_parse {
            template: b"1CbeFiklI:NPp:t:T:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: Some(
                cmd_command_prompt_args_parse
                    as unsafe extern "C" fn(
                        *mut args,
                        u_int,
                        *mut *mut ::core::ffi::c_char,
                    ) -> args_parse_type,
            ),
        },
        usage: b"[-1CbeFiklNP] [-I inputs] [-p prompts] [-t target-client] [-T prompt-type] [template]\0"
            as *const u8 as *const ::core::ffi::c_char,
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
        exec: Some(
            cmd_command_prompt_exec
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_command_prompt_args_parse(
    mut args: *mut args,
    mut idx: u_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> args_parse_type {
    return ARGS_PARSE_COMMANDS_OR_STRING;
}
unsafe extern "C" fn cmd_command_prompt_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
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
    } else if !(*tc).prompt.is_null() {
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'i' as i32 as u_char) != 0 {
        wait = 0 as ::core::ffi::c_int;
    }
    let cdata = Box::into_raw(Box::new(cmd_command_prompt_cdata {
        item: ::core::ptr::null_mut(),
        state: ::core::ptr::null_mut(),
        flags: 0,
        prompt_type: PROMPT_TYPE_COMMAND,
        wp: ::core::ptr::null_mut(),
        prompts: Vec::new(),
        current: 0,
        argc: 0,
        argv: ::core::ptr::null_mut(),
    }));
    if wait != 0 {
        (*cdata).item = item;
    }
    if pane != 0 {
        (*cdata).wp = wp;
    }
    (*cdata).state = args_make_commands_prepare(
        self_0,
        item,
        0 as u_int,
        b"%1\0" as *const u8 as *const ::core::ffi::c_char,
        wait,
        args_has(args, 'F' as i32 as u_char),
    );
    let literal = args_has(args, 'l' as i32 as u_char) != 0;
    s = args_get(args, 'p' as i32 as u_char);
    if s.is_null() {
        if count != 0 as u_int {
            let command = args_make_commands_get_command_cstring((*cdata).state);
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
    (*cdata).prompts = cmd_command_prompt_rows(&prompt_bytes, input_bytes, literal, space != 0);
    type_0 = args_get(args, 'T' as i32 as u_char);
    if !type_0.is_null() {
        (*cdata).prompt_type = prompt_type(type_0);
        if (*cdata).prompt_type as ::core::ffi::c_uint
            == PROMPT_TYPE_INVALID as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            cmdq_error(
                item,
                b"unknown type: %s\0" as *const u8 as *const ::core::ffi::c_char,
                type_0,
            );
            cmd_command_prompt_free(cdata as *mut ::core::ffi::c_void);
            return CMD_RETURN_ERROR;
        }
    } else {
        (*cdata).prompt_type = PROMPT_TYPE_COMMAND;
    }
    if args_has(args, '1' as i32 as u_char) != 0 {
        (*cdata).flags |= PROMPT_SINGLE;
    } else if args_has(args, 'N' as i32 as u_char) != 0 {
        (*cdata).flags |= PROMPT_NUMERIC;
    } else if args_has(args, 'i' as i32 as u_char) != 0 {
        (*cdata).flags |= PROMPT_INCREMENTAL;
    } else if args_has(args, 'k' as i32 as u_char) != 0 {
        (*cdata).flags |= PROMPT_KEY;
    } else if args_has(args, 'e' as i32 as u_char) != 0 {
        (*cdata).flags |= PROMPT_BSPACE_EXIT;
    }
    if args_has(args, 'C' as i32 as u_char) != 0 {
        (*cdata).flags |= PROMPT_NOFREEZE;
    }
    let (prompt_ptr, input_ptr) = (&(*cdata).prompts)[0].pointers();
    if pane != 0 {
        (*cdata).flags |= PROMPT_ISPANE;
        window_pane_set_prompt(
            wp,
            tc,
            target,
            prompt_ptr,
            input_ptr,
            Some(
                cmd_command_prompt_callback
                    as unsafe extern "C" fn(
                        *mut client,
                        *mut ::core::ffi::c_void,
                        *const ::core::ffi::c_char,
                        prompt_key_result,
                    ) -> prompt_result,
            ),
            Some(cmd_command_prompt_free as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
            cdata as *mut ::core::ffi::c_void,
            (*cdata).flags,
            (*cdata).prompt_type,
        );
    } else {
        status_prompt_set(
            tc,
            target,
            prompt_ptr,
            input_ptr,
            Some(
                cmd_command_prompt_callback
                    as unsafe extern "C" fn(
                        *mut client,
                        *mut ::core::ffi::c_void,
                        *const ::core::ffi::c_char,
                        prompt_key_result,
                    ) -> prompt_result,
            ),
            Some(cmd_command_prompt_free as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
            cdata as *mut ::core::ffi::c_void,
            (*cdata).flags,
            (*cdata).prompt_type,
        );
    }
    if wait == 0 {
        return CMD_RETURN_NORMAL;
    }
    return CMD_RETURN_WAIT;
}
unsafe extern "C" fn cmd_command_prompt_callback(
    mut c: *mut client,
    mut data: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut current_block: u64;
    let mut cdata: *mut cmd_command_prompt_cdata = data as *mut cmd_command_prompt_cdata;
    let mut item: *mut cmdq_item = (*cdata).item;
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut argc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut argv: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    if !(s.is_null()
        || key as ::core::ffi::c_uint
            == PROMPT_KEY_MOVE as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        if key as ::core::ffi::c_uint
            == PROMPT_KEY_CLOSE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*cdata).flags & PROMPT_INCREMENTAL != 0 {
                current_block = 11745758271394821990;
            } else {
                cmd_append_argv(&raw mut (*cdata).argc, &raw mut (*cdata).argv, s);
                (*cdata).current = (*cdata).current.wrapping_add(1);
                if ((*cdata).current as usize) != (*cdata).prompts.len() {
                    let (prompt_ptr, input_ptr) =
                        (&(*cdata).prompts)[(*cdata).current as usize].pointers();
                    if !(*cdata).wp.is_null() {
                        window_pane_update_prompt((*cdata).wp, prompt_ptr, input_ptr);
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
                argc = (*cdata).argc;
                argv = cmd_copy_argv((*cdata).argc, (*cdata).argv);
                if key as ::core::ffi::c_uint
                    != PROMPT_KEY_CLOSE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    cmd_append_argv(&raw mut argc, &raw mut argv, s);
                } else {
                    cmd_free_argv((*cdata).argc, (*cdata).argv);
                    (*cdata).argc = argc;
                    (*cdata).argv = cmd_copy_argv(argc, argv);
                }
                match args_make_commands((*cdata).state, argc, argv) {
                    Err(error) => {
                        cmdq_append(
                            c,
                            cmdq_get_error(error.as_ref().map_or(
                                ::core::ptr::null(),
                                |cause| cause.as_ptr(),
                            )),
                        );
                    }
                    Ok(cmdlist) if item.is_null() => {
                    new_item = cmdq_get_command(cmdlist, ::core::ptr::null_mut::<cmdq_state>());
                    cmdq_append(c, new_item);
                    cmd_list_free(cmdlist);
                    }
                    Ok(cmdlist) => {
                    new_item = cmdq_get_command(cmdlist, cmdq_get_state(item));
                    cmdq_insert_after(item, new_item);
                    cmd_list_free(cmdlist);
                    }
                }
                cmd_free_argv(argc, argv);
                if (*cdata).flags & PROMPT_INCREMENTAL != 0 {
                    return PROMPT_CONTINUE;
                }
            }
        }
    }
    if !item.is_null() {
        (*cdata).item = ::core::ptr::null_mut::<cmdq_item>();
        cmdq_continue(item);
    }
    return PROMPT_CLOSE;
}
unsafe extern "C" fn cmd_command_prompt_free(mut data: *mut ::core::ffi::c_void) {
    let mut cdata = Box::from_raw(data as *mut cmd_command_prompt_cdata);
    if !cdata.item.is_null() {
        cmdq_continue(cdata.item);
        cdata.item = ::core::ptr::null_mut::<cmdq_item>();
    }
    cdata.prompts.clear();
    cmd_free_argv(cdata.argc, cdata.argv);
    args_make_commands_free(cdata.state);
}

#[cfg(test)]
mod tests {
    use super::cmd_command_prompt_rows;

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
