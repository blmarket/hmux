use crate::src::cmd::find::{cmd_find_from_client, cmd_find_valid_state};
use crate::src::cmd::queue::{cmdq_append, cmdq_get_command, cmdq_insert_after, cmdq_print};
use crate::src::cmd::{
    cmd_get_alias, cmd_list_append, cmd_list_append_all, cmd_list_free, cmd_list_move,
    cmd_list_new, cmd_list_print_cstring, cmd_parse,
};
use crate::src::environ::environ_put;
use crate::src::format::bytes::write_cstr;
use crate::src::format::{
    format_create, format_defaults, format_expand_cstring, format_free, format_true,
};
use crate::src::log::{fatalx, log_bytes, log_cstr, log_debug};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args_value;
use crate::src::shared::arguments::*;
use crate::src::shared::client::client;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_find_state, cmd_list, cmdq_item, cmdq_state};
use crate::src::shared::command::{cmd_parse_input, cmd_parse_result};
use crate::src::shared::command::{
    CMD_PARSE_MAX_ENVIRON_LEN, CMD_PARSE_NOALIAS, CMD_PARSE_ONEGROUP, CMD_PARSE_PARSEONLY,
    CMD_PARSE_VERBOSE,
};
use crate::src::shared::environment::ENVIRON_HIDDEN;
use crate::src::shared::format::{FORMAT_NOJOBS, FORMAT_NONE};
use crate::src::shared::limits::UINT_MAX;
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::stdio::FILE;
use crate::src::shared::window::{window, winlink};
use crate::src::tmux::global_environ;
use libc;
use std::collections::VecDeque;
use std::ffi::{CStr, CString};

/// Box-owned collection of parser commands. Each command is separately boxed
/// so pointers held by the parser remain stable when the collection grows.
pub struct cmd_parse_commands {
    pub items: Vec<Box<cmd_parse_command>>,
}
/// Box-owned parser command; its argument collection is inline and stable.
pub struct cmd_parse_command {
    pub line: u_int,
    pub arguments: cmd_parse_arguments,
}
/// Inline in commands, or Box-owned until a grammar action transfers its items.
pub struct cmd_parse_arguments {
    pub items: VecDeque<Box<cmd_parse_argument>>,
}
/// Box-owned parser argument; payload ownership depends on `type_0`.
pub struct cmd_parse_argument {
    pub type_0: cmd_parse_argument_type,
    pub string: Option<CString>,
    pub commands: *mut cmd_parse_commands,
    pub cmdlist: *mut cmd_list,
}
pub type cmd_parse_argument_type = ::core::ffi::c_uint;
pub const CMD_PARSE_PARSED_COMMANDS: cmd_parse_argument_type = 2;
pub const CMD_PARSE_COMMANDS: cmd_parse_argument_type = 1;
pub const CMD_PARSE_STRING: cmd_parse_argument_type = 0;
mod lexer;
use lexer::Lexer;
use std::cell::RefCell;

/// Shared only for the duration of a parse. Grammar actions and the lazy lexer
/// borrow it in turn, so assignments remain visible to subsequent words.
struct ParseSession<'a> {
    input: &'a mut cmd_parse_input,
    error: Option<CString>,
}

impl ParseSession<'_> {
    fn record_error(&mut self, error: &CStr) {
        if self.error.is_none() {
            self.error = Some(cmd_parse_get_error(
                self.input.file.as_deref(),
                self.input.line,
                error,
            ));
        }
    }
}

fn cmd_parse_get_error(file: Option<&CStr>, line: u_int, error: &CStr) -> CString {
    let Some(file) = file else {
        return error.to_owned();
    };
    let mut bytes = file.to_bytes().to_vec();
    bytes.push(b':');
    bytes.extend_from_slice(line.to_string().as_bytes());
    bytes.extend_from_slice(b": ");
    bytes.extend_from_slice(error.to_bytes());
    CString::new(bytes).expect("parser diagnostic contains no NUL")
}

pub fn cmd_parse_error_uppercase_first(error: &mut Option<CString>) {
    let Some(cause) = error.as_mut() else {
        return;
    };
    let mut bytes = cause.as_bytes_with_nul().to_vec();
    if let Some(first) = bytes.first_mut() {
        *first = unsafe { libc::toupper(*first as u8 as libc::c_int) as u8 };
    }
    *cause = CString::from_vec_with_nul(bytes).expect("cause remains NUL terminated");
}
unsafe fn cmd_parse_print_commands(mut pi: *mut cmd_parse_input, mut cmdlist: *mut cmd_list) {
    if (*pi).item.is_null() || !(*pi).flags & CMD_PARSE_VERBOSE != 0 {
        return;
    }
    let s = cmd_list_print_cstring(&*cmdlist, 0);
    if (*pi).file.is_some() {
        cmdq_print((*pi).item, |out| {
            write_cstr(out, (*pi).file_ptr())?;
            write!(out, ":{}: ", ((*pi).line) as u32)?;
            out.write_all(s.as_bytes())
        });
    } else {
        cmdq_print((*pi).item, |out| {
            write!(out, "{}: ", ((*pi).line) as u32)?;
            out.write_all(s.as_bytes())
        });
    }
}
unsafe fn cmd_parse_new_argument() -> *mut cmd_parse_argument {
    Box::into_raw(Box::new(cmd_parse_argument {
        type_0: CMD_PARSE_STRING,
        string: None,
        commands: ::core::ptr::null_mut(),
        cmdlist: ::core::ptr::null_mut(),
    }))
}
unsafe fn cmd_parse_free_argument(mut arg: *mut cmd_parse_argument) {
    match (*arg).type_0 as ::core::ffi::c_uint {
        0 => {}
        1 => {
            cmd_parse_free_commands((*arg).commands as *mut cmd_parse_commands);
        }
        2 => {
            cmd_list_free((*arg).cmdlist);
        }
        _ => {}
    }
    drop(Box::from_raw(arg));
}
unsafe fn cmd_parse_free_arguments(mut args: *mut cmd_parse_arguments) {
    while let Some(arg) = (*args).items.pop_front() {
        cmd_parse_free_argument(Box::into_raw(arg));
    }
}
unsafe fn cmd_parse_new_command(line: u_int) -> *mut cmd_parse_command {
    Box::into_raw(Box::new(cmd_parse_command {
        line,
        arguments: cmd_parse_arguments {
            items: VecDeque::new(),
        },
    }))
}
unsafe fn cmd_parse_free_command(mut cmd: *mut cmd_parse_command) {
    cmd_parse_free_arguments(&raw mut (*cmd).arguments);
    drop(Box::from_raw(cmd));
}
unsafe fn cmd_parse_new_commands() -> *mut cmd_parse_commands {
    Box::into_raw(Box::new(cmd_parse_commands { items: Vec::new() }))
}
unsafe fn cmd_parse_free_commands(mut cmds: *mut cmd_parse_commands) {
    while let Some(cmd) = (*cmds).items.pop() {
        cmd_parse_free_command(Box::into_raw(cmd));
    }
    drop(Box::from_raw(cmds));
}
unsafe fn cmd_parse_command_at(
    cmds: *mut cmd_parse_commands,
    index: usize,
) -> *mut cmd_parse_command {
    (&mut (*cmds).items)
        .get_mut(index)
        .map_or(::core::ptr::null_mut(), |cmd| &mut **cmd)
}
unsafe fn cmd_parse_arguments_first(args: *mut cmd_parse_arguments) -> *mut cmd_parse_argument {
    (&mut (*args).items)
        .front_mut()
        .map_or(::core::ptr::null_mut(), |arg| &mut **arg)
}
unsafe fn cmd_parse_argument_at(
    args: *mut cmd_parse_arguments,
    index: usize,
) -> *mut cmd_parse_argument {
    (&mut (*args).items)
        .get_mut(index)
        .map_or(::core::ptr::null_mut(), |arg| &mut **arg)
}
unsafe fn cmd_parse_commands_push(cmds: *mut cmd_parse_commands, cmd: *mut cmd_parse_command) {
    (*cmds).items.push(Box::from_raw(cmd));
}
unsafe fn cmd_parse_arguments_push(args: *mut cmd_parse_arguments, arg: *mut cmd_parse_argument) {
    (*args).items.push_back(Box::from_raw(arg));
}
// The lexer remains application-owned: variable and tilde expansion depend on
// the live environment. Pull one token at a time so assignments affect later words.
struct ParserContext<'a, 'input>(&'a RefCell<ParseSession<'input>>);

impl hmux_cmdparse::Context for ParserContext<'_, '_> {
    fn line(&self) -> u32 {
        self.0.borrow().input.line
    }

    fn expand_format(&mut self, token: hmux_cmdparse::TokenText) -> hmux_cmdparse::TokenText {
        unsafe {
            let mut session = self.0.borrow_mut();
            let pi = &mut *session.input;
            let mut fs: cmd_find_state = Default::default();
            let fsp = if cmd_find_valid_state(&raw mut (*pi).fs) != 0 {
                &raw mut (*pi).fs
            } else {
                cmd_find_from_client(&raw mut fs, (*pi).c, 0);
                &raw mut fs
            };
            let ft = format_create((*pi).c, (*pi).item, FORMAT_NONE, FORMAT_NOJOBS);
            format_defaults(ft, (*pi).c, (*fsp).s, (*fsp).wl, (*fsp).wp);
            let expanded = format_expand_cstring(ft, token.as_c_str().as_ptr());
            format_free(ft);
            take_parser_token(expanded)
        }
    }

    fn put_environ(
        &mut self,
        token: hmux_cmdparse::TokenText,
        hidden: bool,
        active: bool,
    ) -> Result<(), hmux_cmdparse::LexError> {
        unsafe {
            if token.as_c_str().to_bytes().len() > CMD_PARSE_MAX_ENVIRON_LEN as usize {
                self.0
                    .borrow_mut()
                    .record_error(c"environment variable is too long");
                return Err(hmux_cmdparse::LexError);
            }
            if self.0.borrow().input.flags & CMD_PARSE_PARSEONLY == 0 && active {
                environ_put(
                    global_environ,
                    token.as_c_str().as_ptr(),
                    if hidden { ENVIRON_HIDDEN } else { 0 },
                );
            }
        }
        Ok(())
    }

    fn is_true(&self, token: &hmux_cmdparse::TokenText) -> bool {
        unsafe { format_true(token.as_c_str().as_ptr()) != 0 }
    }
}

fn take_parser_token(text: CString) -> hmux_cmdparse::TokenText {
    hmux_cmdparse::TokenText::from_cstring(text)
}

unsafe fn build_parser_commands(
    commands: Vec<hmux_cmdparse::ParseCommand>,
) -> *mut cmd_parse_commands {
    let output = cmd_parse_new_commands();
    for command in commands {
        let cmd = cmd_parse_new_command(command.line);
        for argument in command.arguments {
            let arg = cmd_parse_new_argument();
            match argument {
                hmux_cmdparse::ParseArgument::String(text) => {
                    (*arg).type_0 = CMD_PARSE_STRING;
                    (*arg).string = Some(text.as_c_str().to_owned());
                }
                hmux_cmdparse::ParseArgument::Commands(commands) => {
                    (*arg).type_0 = CMD_PARSE_COMMANDS;
                    (*arg).commands = build_parser_commands(commands);
                }
            }
            cmd_parse_arguments_push(&raw mut (*cmd).arguments, arg);
        }
        cmd_parse_commands_push(output, cmd);
    }
    output
}

unsafe fn cmd_parse_run_parser(
    session: &RefCell<ParseSession<'_>>,
    lexer: Lexer<'_, '_>,
) -> Result<*mut cmd_parse_commands, CString> {
    match hmux_cmdparse::parse(&mut ParserContext(session), lexer) {
        Ok(commands) => Ok(build_parser_commands(commands)),
        Err(_) => {
            let mut session = session.borrow_mut();
            session.record_error(c"syntax error");
            Err(session.error.take().expect("parser error was recorded"))
        }
    }
}
unsafe fn cmd_parse_do_file(
    f: *mut FILE,
    pi: *mut cmd_parse_input,
) -> Result<*mut cmd_parse_commands, CString> {
    let session = RefCell::new(ParseSession {
        input: &mut *pi,
        error: None,
    });
    cmd_parse_run_parser(&session, Lexer::from_file(f, &session))
}
unsafe fn cmd_parse_do_buffer(
    buf: *const ::core::ffi::c_char,
    len: size_t,
    pi: *mut cmd_parse_input,
) -> Result<*mut cmd_parse_commands, CString> {
    let bytes = if len == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(buf.cast(), len)
    };
    let session = RefCell::new(ParseSession {
        input: &mut *pi,
        error: None,
    });
    cmd_parse_run_parser(&session, Lexer::from_bytes(bytes, &session))
}
unsafe fn cmd_parse_log_commands(mut cmds: *mut cmd_parse_commands, prefix: &CStr) {
    let mut cmd: *mut cmd_parse_command = ::core::ptr::null_mut::<cmd_parse_command>();
    let mut arg: *mut cmd_parse_argument = ::core::ptr::null_mut::<cmd_parse_argument>();
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let command_count = (*cmds).items.len();
    i = 0 as u_int;
    while (i as usize) < command_count {
        cmd = cmd_parse_command_at(cmds, i as usize);
        j = 0 as u_int;
        let argument_count = (*cmd).arguments.items.len();
        while (j as usize) < argument_count {
            arg = cmd_parse_argument_at(&raw mut (*cmd).arguments, j as usize);
            match (*arg).type_0 as ::core::ffi::c_uint {
                0 => {
                    log_debug(format_args!(
                        "{} {}:{}: {}",
                        log_bytes(prefix.to_bytes()),
                        (i) as u32,
                        (j) as u32,
                        log_bytes(
                            (*arg)
                                .string
                                .as_ref()
                                .expect("parser string argument owns its text")
                                .as_bytes()
                        )
                    ));
                }
                1 => {
                    let mut nested = prefix.to_bytes().to_vec();
                    nested.extend_from_slice(format!(" {i}:{j}").as_bytes());
                    let nested = CString::new(nested).expect("parser log prefix has no NUL");
                    cmd_parse_log_commands((*arg).commands as *mut cmd_parse_commands, &nested);
                }
                2 => {
                    let s = cmd_list_print_cstring(&*(*arg).cmdlist, 0);
                    log_debug(format_args!(
                        "{} {}:{}: {}",
                        log_bytes(prefix.to_bytes()),
                        (i) as u32,
                        (j) as u32,
                        log_bytes(s.as_bytes())
                    ));
                }
                _ => {}
            }
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
}
unsafe fn cmd_parse_expand_alias(
    mut cmd: *mut cmd_parse_command,
    mut pi: *mut cmd_parse_input,
    mut pr: *mut cmd_parse_result,
) -> ::core::ffi::c_int {
    let mut first: *mut cmd_parse_argument = ::core::ptr::null_mut::<cmd_parse_argument>();
    let mut cmds: *mut cmd_parse_commands = ::core::ptr::null_mut::<cmd_parse_commands>();
    let mut last: *mut cmd_parse_command = ::core::ptr::null_mut::<cmd_parse_command>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*pi).flags & CMD_PARSE_NOALIAS != 0 {
        return 0 as ::core::ffi::c_int;
    }
    *pr = cmd_parse_result::empty();
    first = cmd_parse_arguments_first(&raw mut (*cmd).arguments);
    if first.is_null()
        || (*first).type_0 as ::core::ffi::c_uint
            != CMD_PARSE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*pr).status = CMD_PARSE_SUCCESS;
        (*pr).cmdlist = cmd_list_new();
        return 1 as ::core::ffi::c_int;
    }
    name = (*first)
        .string
        .as_ref()
        .expect("alias command owns its name")
        .as_ptr()
        .cast_mut();
    let Some(alias) = cmd_get_alias(CStr::from_ptr(name)) else {
        return 0 as ::core::ffi::c_int;
    };
    log_debug(format_args!(
        "{}: {} alias {} = {}",
        "cmd_parse_expand_alias",
        ((*pi).line) as u32,
        log_cstr((name) as *const _),
        log_bytes(alias.as_bytes())
    ));
    let parsed = cmd_parse_do_buffer(alias.as_ptr(), alias.as_bytes().len(), pi);
    cmds = match parsed {
        Ok(cmds) => cmds,
        Err(cause) => {
            (*pr).status = CMD_PARSE_ERROR;
            (*pr).error = Some(cause);
            return 1 as ::core::ffi::c_int;
        }
    };
    let command_count = (*cmds).items.len();
    if command_count == 0 {
        (*pr).status = CMD_PARSE_SUCCESS;
        (*pr).cmdlist = cmd_list_new();
        cmd_parse_free_commands(cmds);
        return 1 as ::core::ffi::c_int;
    }
    last = cmd_parse_command_at(cmds, command_count - 1);
    let first_argument = (*cmd)
        .arguments
        .items
        .pop_front()
        .expect("alias command has a first argument");
    cmd_parse_free_argument(Box::into_raw(first_argument));
    (*last).arguments.items.append(&mut (*cmd).arguments.items);
    cmd_parse_log_commands(cmds, c"cmd_parse_expand_alias");
    (*pi).flags |= CMD_PARSE_NOALIAS;
    cmd_parse_build_commands(cmds, pi, pr);
    (*pi).flags &= !CMD_PARSE_NOALIAS;
    cmd_parse_free_commands(cmds);
    return 1 as ::core::ffi::c_int;
}
unsafe fn cmd_parse_build_command(
    mut cmd: *mut cmd_parse_command,
    mut pi: *mut cmd_parse_input,
    mut pr: *mut cmd_parse_result,
) {
    let mut current_block: u64 = 16207960823932980356;
    let mut arg: *mut cmd_parse_argument = ::core::ptr::null_mut::<cmd_parse_argument>();
    let mut add: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut values = Vec::<args_value>::new();
    let mut count: u_int = 0 as u_int;
    let _idx: u_int = 0;
    let mut argument_index: usize = 0;
    let argument_count = (*cmd).arguments.items.len();
    *pr = cmd_parse_result::empty();
    if cmd_parse_expand_alias(cmd, pi, pr) != 0 {
        return;
    }
    while argument_index < argument_count {
        arg = cmd_parse_argument_at(&raw mut (*cmd).arguments, argument_index);
        values.push(args_value::empty());
        let value = values.as_mut_ptr().add(count as usize);
        match (*arg).type_0 as ::core::ffi::c_uint {
            0 => {
                *value = args_value::borrowed_string(
                    (*arg)
                        .string
                        .as_ref()
                        .expect("parser string argument owns its text")
                        .as_ptr()
                        .cast_mut(),
                );
            }
            1 => {
                cmd_parse_build_commands((*arg).commands as *mut cmd_parse_commands, pi, pr);
                if (*pr).status as ::core::ffi::c_uint
                    != CMD_PARSE_SUCCESS as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    current_block = 16207960823932980356;
                    break;
                }
                *value = args_value::commands((*pr).cmdlist as *mut cmd_list);
            }
            2 => {
                crate::src::shared::rc::retain((*arg).cmdlist);
                *value = args_value::commands((*arg).cmdlist as *mut cmd_list);
            }
            _ => {}
        }
        count = count.wrapping_add(1);
        argument_index += 1;
    }
    if argument_index == argument_count {
        current_block = 5143058163439228106;
    }
    match current_block {
        5143058163439228106 => {
            match cmd_parse(
                if values.is_empty() {
                    ::core::ptr::null_mut::<args_value>()
                } else {
                    values.as_mut_ptr()
                },
                count,
                (*pi).file.as_deref(),
                (*pi).line,
                (*pi).flags,
            ) {
                Ok(command) => {
                    add = command;
                    (*pr).status = CMD_PARSE_SUCCESS;
                    (*pr).cmdlist = cmd_list_new();
                    cmd_list_append((*pr).cmdlist, add);
                }
                Err(cause) => {
                    (*pr).status = CMD_PARSE_ERROR;
                    (*pr).error = Some(cmd_parse_get_error(
                        (*pi).file.as_deref(),
                        (*pi).line,
                        cause.as_c_str(),
                    ));
                }
            }
        }
        _ => {}
    }
}
unsafe fn cmd_parse_build_commands(
    mut cmds: *mut cmd_parse_commands,
    mut pi: *mut cmd_parse_input,
    mut pr: *mut cmd_parse_result,
) {
    let mut cmd: *mut cmd_parse_command = ::core::ptr::null_mut::<cmd_parse_command>();
    let mut line: u_int = UINT_MAX;
    let mut current: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    let mut result: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    *pr = cmd_parse_result::empty();
    let command_count = (*cmds).items.len();
    if command_count == 0 {
        (*pr).status = CMD_PARSE_SUCCESS;
        (*pr).cmdlist = cmd_list_new();
        return;
    }
    cmd_parse_log_commands(cmds, c"cmd_parse_build_commands");
    result = cmd_list_new();
    let mut command_index = 0usize;
    while command_index < command_count {
        cmd = cmd_parse_command_at(cmds, command_index);
        if !(*pi).flags & CMD_PARSE_ONEGROUP != 0 && (*cmd).line != line {
            if !current.is_null() {
                cmd_parse_print_commands(pi, current);
                cmd_list_move(result, current);
                cmd_list_free(current);
            }
            current = cmd_list_new();
        }
        if current.is_null() {
            current = cmd_list_new();
        }
        (*pi).line = (*cmd).line;
        line = (*pi).line;
        cmd_parse_build_command(cmd, pi, pr);
        if (*pr).status as ::core::ffi::c_uint
            != CMD_PARSE_SUCCESS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            cmd_list_free(result);
            cmd_list_free(current);
            return;
        }
        cmd_list_append_all(current, (*pr).cmdlist);
        cmd_list_free((*pr).cmdlist);
        command_index += 1;
    }
    if !current.is_null() {
        cmd_parse_print_commands(pi, current);
        cmd_list_move(result, current);
        cmd_list_free(current);
    }
    let s = cmd_list_print_cstring(&*result, 0);
    log_debug(format_args!(
        "{}: {}",
        "cmd_parse_build_commands",
        log_bytes(s.as_bytes())
    ));
    (*pr).status = CMD_PARSE_SUCCESS;
    (*pr).cmdlist = result;
}
pub unsafe fn cmd_parse_from_file(
    mut f: *mut FILE,
    mut pi: *mut cmd_parse_input,
) -> cmd_parse_result {
    let mut input: cmd_parse_input = cmd_parse_input {
        flags: 0,
        file: None,
        line: 0,
        item: ::core::ptr::null_mut::<cmdq_item>(),
        c: ::core::ptr::null_mut::<client>(),
        fs: cmd_find_state {
            flags: 0,
            current: ::core::ptr::null_mut::<cmd_find_state>(),
            s: ::core::ptr::null_mut::<session>(),
            wl: ::core::ptr::null_mut::<winlink>(),
            w: ::core::ptr::null_mut::<window>(),
            wp: ::core::ptr::null_mut::<window_pane>(),
            idx: 0,
        },
    };
    let mut pr = cmd_parse_result::empty();
    if pi.is_null() {
        pi = &raw mut input;
    }
    let cmds = match cmd_parse_do_file(f, pi) {
        Ok(cmds) => cmds,
        Err(cause) => {
            pr.status = CMD_PARSE_ERROR;
            pr.error = Some(cause);
            return pr;
        }
    };
    cmd_parse_build_commands(cmds, pi, &raw mut pr);
    cmd_parse_free_commands(cmds);
    return pr;
}
pub unsafe fn cmd_parse_from_string(s: &CStr, mut pi: *mut cmd_parse_input) -> cmd_parse_result {
    let mut input: cmd_parse_input = cmd_parse_input {
        flags: 0,
        file: None,
        line: 0,
        item: ::core::ptr::null_mut::<cmdq_item>(),
        c: ::core::ptr::null_mut::<client>(),
        fs: cmd_find_state {
            flags: 0,
            current: ::core::ptr::null_mut::<cmd_find_state>(),
            s: ::core::ptr::null_mut::<session>(),
            wl: ::core::ptr::null_mut::<winlink>(),
            w: ::core::ptr::null_mut::<window>(),
            wp: ::core::ptr::null_mut::<window_pane>(),
            idx: 0,
        },
    };
    if pi.is_null() {
        pi = &raw mut input;
    }
    (*pi).flags |= CMD_PARSE_ONEGROUP;
    return cmd_parse_from_buffer(
        s.as_ptr() as *const ::core::ffi::c_void,
        s.to_bytes().len() as size_t,
        pi,
    );
}
pub unsafe fn cmd_parse_and_append(
    s: &CStr,
    mut c: *mut client,
    mut state: *mut cmdq_state,
) -> Result<cmd_parse_status, Option<CString>> {
    let mut pi: *mut cmd_parse_input = ::core::ptr::null_mut::<cmd_parse_input>();
    let mut item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut pr = cmd_parse_from_string(s, pi);
    if pr.status == CMD_PARSE_ERROR {
        return Err(pr.error.take());
    }
    item = cmdq_get_command(pr.cmdlist, state);
    cmdq_append(c, item);
    cmd_list_free(pr.cmdlist);
    Ok(pr.status)
}
pub unsafe fn cmd_parse_from_buffer(
    mut buf: *const ::core::ffi::c_void,
    mut len: size_t,
    mut pi: *mut cmd_parse_input,
) -> cmd_parse_result {
    let mut input: cmd_parse_input = cmd_parse_input {
        flags: 0,
        file: None,
        line: 0,
        item: ::core::ptr::null_mut::<cmdq_item>(),
        c: ::core::ptr::null_mut::<client>(),
        fs: cmd_find_state {
            flags: 0,
            current: ::core::ptr::null_mut::<cmd_find_state>(),
            s: ::core::ptr::null_mut::<session>(),
            wl: ::core::ptr::null_mut::<winlink>(),
            w: ::core::ptr::null_mut::<window>(),
            wp: ::core::ptr::null_mut::<window_pane>(),
            idx: 0,
        },
    };
    let mut pr = cmd_parse_result::empty();
    if pi.is_null() {
        pi = &raw mut input;
    }
    if len == 0 as size_t {
        pr.status = CMD_PARSE_SUCCESS;
        pr.cmdlist = cmd_list_new();
        return pr;
    }
    let cmds = match cmd_parse_do_buffer(buf as *const ::core::ffi::c_char, len, pi) {
        Ok(cmds) => cmds,
        Err(cause) => {
            pr.status = CMD_PARSE_ERROR;
            pr.error = Some(cause);
            return pr;
        }
    };
    cmd_parse_build_commands(cmds, pi, &raw mut pr);
    cmd_parse_free_commands(cmds);
    return pr;
}
/// Parse argv while borrowing its strings for the duration of the parser call.
pub unsafe fn cmd_parse_from_argv(argv: &[CString]) -> cmd_parse_result {
    let pi: *mut cmd_parse_input = ::core::ptr::null_mut::<cmd_parse_input>();
    let mut values: Vec<args_value> = argv
        .iter()
        .map(|string| args_value::borrowed_string(string.as_ptr()))
        .collect();
    cmd_parse_from_arguments(
        values.as_mut_ptr(),
        u_int::try_from(values.len()).expect("argv length exceeds u_int"),
        pi,
    )
}

pub unsafe fn cmd_parse_from_arguments(
    mut values: *mut args_value,
    mut count: u_int,
    mut pi: *mut cmd_parse_input,
) -> cmd_parse_result {
    let mut input: cmd_parse_input = cmd_parse_input {
        flags: 0,
        file: None,
        line: 0,
        item: ::core::ptr::null_mut::<cmdq_item>(),
        c: ::core::ptr::null_mut::<client>(),
        fs: cmd_find_state {
            flags: 0,
            current: ::core::ptr::null_mut::<cmd_find_state>(),
            s: ::core::ptr::null_mut::<session>(),
            wl: ::core::ptr::null_mut::<winlink>(),
            w: ::core::ptr::null_mut::<window>(),
            wp: ::core::ptr::null_mut::<window_pane>(),
            idx: 0,
        },
    };
    let mut cmds: *mut cmd_parse_commands = ::core::ptr::null_mut::<cmd_parse_commands>();
    let mut cmd: *mut cmd_parse_command = ::core::ptr::null_mut::<cmd_parse_command>();
    let mut arg: *mut cmd_parse_argument = ::core::ptr::null_mut::<cmd_parse_argument>();
    let mut i: u_int = 0;
    let mut size: size_t = 0;
    let mut end: ::core::ffi::c_int = 0;
    let mut pr = cmd_parse_result::empty();
    if pi.is_null() {
        pi = &raw mut input;
    }
    cmds = cmd_parse_new_commands();
    cmd = cmd_parse_new_command((*pi).line);
    i = 0 as u_int;
    while i < count {
        end = 0 as ::core::ffi::c_int;
        if (*values.offset(i as isize)).type_0() as ::core::ffi::c_uint
            == ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut bytes = CStr::from_ptr((*values.add(i as usize)).string_ptr())
                .to_bytes()
                .to_vec();
            size = bytes.len() as size_t;
            if size != 0 && bytes[size as usize - 1] == b';' {
                size -= 1;
                if size > 0 && bytes[size as usize - 1] == b'\\' {
                    bytes[size as usize - 1] = b';';
                } else {
                    end = 1;
                }
            }
            if end == 0 || size != 0 as size_t {
                bytes.truncate(size as usize);
                arg = cmd_parse_new_argument();
                (*arg).type_0 = CMD_PARSE_STRING;
                (*arg).string = Some(CString::new(bytes).expect("argument contains no NUL"));
                cmd_parse_arguments_push(&raw mut (*cmd).arguments, arg);
            }
        } else if (*values.offset(i as isize)).type_0() as ::core::ffi::c_uint
            == ARGS_COMMANDS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            arg = cmd_parse_new_argument();
            (*arg).type_0 = CMD_PARSE_PARSED_COMMANDS;
            (*arg).cmdlist = (*values.offset(i as isize)).cmdlist();
            crate::src::shared::rc::retain((*arg).cmdlist);
            cmd_parse_arguments_push(&raw mut (*cmd).arguments, arg);
        } else {
            fatalx(|out| out.write_all(b"unknown argument type"));
        }
        if end != 0 {
            cmd_parse_commands_push(cmds, cmd);
            cmd = cmd_parse_new_command((*pi).line);
        }
        i = i.wrapping_add(1);
    }
    if !(*cmd).arguments.items.is_empty() {
        cmd_parse_commands_push(cmds, cmd);
    } else {
        cmd_parse_free_command(cmd);
    }
    cmd_parse_build_commands(cmds, pi, &raw mut pr);
    cmd_parse_free_commands(cmds);
    return pr;
}
#[cfg(test)]
mod parser_collection_tests {
    use super::*;

    unsafe fn parse_commands(input: &[u8]) -> Vec<Vec<Vec<u8>>> {
        let mut pi: cmd_parse_input = Default::default();
        pi.line = 1;
        let cmds = cmd_parse_do_buffer(input.as_ptr().cast(), input.len(), &mut pi)
            .expect("valid parser test input");
        assert!(
            !cmds.is_null(),
            "parser returned a null command list for valid input"
        );
        let result = (*cmds)
            .items
            .iter()
            .map(|cmd| {
                cmd.arguments
                    .items
                    .iter()
                    .map(|arg| {
                        arg.string
                            .as_ref()
                            .expect("parser string argument owns its text")
                            .as_bytes()
                            .to_vec()
                    })
                    .collect()
            })
            .collect();
        cmd_parse_free_commands(cmds);
        result
    }

    #[test]
    fn lazy_expansion_respects_assignments_scopes_and_parseonly() {
        let _guard = crate::src::cfg::CFG_TEST_LOCK.lock().unwrap();
        unsafe {
            struct RestoreEnvironment(*mut crate::src::shared::environment::environ);
            impl Drop for RestoreEnvironment {
                fn drop(&mut self) {
                    unsafe {
                        global_environ = self.0;
                    }
                }
            }
            let owner = crate::src::environ::EnvironOwner::new();
            let _restore = RestoreEnvironment(global_environ);
            global_environ = owner.as_ptr();
            environ_put(global_environ, c"HOME=/test/home".as_ptr(), 0);
            let commands = parse_commands(
                b"LEX_VALUE=first\ndisplay-message $LEX_VALUE\nLEX_VALUE=second\n%if 0\nLEX_VALUE=ignored\n%endif\ndisplay-message \"${LEX_VALUE}\" '$LEX_VALUE' ~/dir\n%hidden LEX_VALUE=hidden\ndisplay-message $LEX_VALUE\n"
            );
            assert_eq!(
                commands,
                vec![
                    vec![b"display-message".to_vec(), b"first".to_vec()],
                    vec![
                        b"display-message".to_vec(),
                        b"second".to_vec(),
                        b"$LEX_VALUE".to_vec(),
                        b"/test/home/dir".to_vec()
                    ],
                    vec![b"display-message".to_vec(), b"hidden".to_vec()],
                ]
            );
            let value = crate::src::environ::environ_find(global_environ, c"LEX_VALUE".as_ptr());
            assert_ne!((*value).flags & ENVIRON_HIDDEN, 0);

            let source = b"LEX_VALUE=skipped\ndisplay-message $LEX_VALUE\n";
            let mut input = cmd_parse_input {
                line: 1,
                flags: CMD_PARSE_PARSEONLY,
                ..Default::default()
            };
            let session = RefCell::new(ParseSession {
                input: &mut input,
                error: None,
            });
            let commands = hmux_cmdparse::parse(
                &mut ParserContext(&session),
                Lexer::from_bytes(source, &session),
            )
            .unwrap();
            let hmux_cmdparse::ParseArgument::String(value) = &commands[0].arguments[1] else {
                panic!("expected string")
            };
            assert_eq!(value.as_c_str().to_bytes(), b"hidden");
        }
    }

    #[test]
    fn parser_preserves_specific_errors_and_recovers_for_next_parse() {
        unsafe {
            for (source, expected) in [
                (
                    b"display-message \\400\n".as_slice(),
                    b"input.conf:3: invalid octal escape".as_slice(),
                ),
                (
                    b"%if 1\ndisplay-message missing-endif\n",
                    b"input.conf:5: syntax error",
                ),
            ] {
                let mut input = cmd_parse_input {
                    line: 3,
                    file: Some(c"input.conf".to_owned()),
                    ..Default::default()
                };
                let error = cmd_parse_do_buffer(source.as_ptr().cast(), source.len(), &mut input)
                    .unwrap_err();
                assert_eq!(error.as_bytes(), expected);
            }
            assert_eq!(
                parse_commands(b"display-message recovered"),
                vec![vec![b"display-message".to_vec(), b"recovered".to_vec()]]
            );
        }
    }

    #[test]
    fn parser_collections_preserve_order_scope_and_element_addresses() {
        let _guard = crate::src::cfg::CFG_TEST_LOCK.lock().unwrap();
        unsafe {
            let mut pi: cmd_parse_input = Default::default();
            pi.line = 1;
            let malformed = b"%if 1\ndisplay-message unfinished\n";
            let failed = cmd_parse_do_buffer(malformed.as_ptr().cast(), malformed.len(), &mut pi);
            assert!(failed.is_err());

            assert_eq!(
                parse_commands(b"display-message alpha beta; display-message gamma\n"),
                vec![
                    vec![
                        b"display-message".to_vec(),
                        b"alpha".to_vec(),
                        b"beta".to_vec()
                    ],
                    vec![b"display-message".to_vec(), b"gamma".to_vec()],
                ],
            );
            assert_eq!(
                parse_commands(b"%if 1\ndisplay-message yes\n%else\ndisplay-message no\n%endif\n"),
                vec![vec![b"display-message".to_vec(), b"yes".to_vec()]],
            );
            assert_eq!(
                parse_commands(b"%if 0\ndisplay-message yes\n%else\ndisplay-message no\n%endif\n"),
                vec![vec![b"display-message".to_vec(), b"no".to_vec()]],
            );
            assert_eq!(
                parse_commands(
                    b"%if 1\n%if 0\ndisplay-message inner-no\n%else\ndisplay-message inner-yes\n%endif\n%else\ndisplay-message outer-no\n%endif\n"
                ),
                vec![vec![b"display-message".to_vec(), b"inner-yes".to_vec()]],
            );

            let cmds = cmd_parse_new_commands();
            let first_cmd = cmd_parse_new_command(1);
            let first_arg = cmd_parse_new_argument();
            cmd_parse_arguments_push(&raw mut (*first_cmd).arguments, first_arg);
            cmd_parse_commands_push(cmds, first_cmd);
            let command_ptr = cmd_parse_command_at(cmds, 0);
            let argument_ptr = cmd_parse_argument_at(&raw mut (*command_ptr).arguments, 0);
            for _ in 0..128 {
                let cmd = cmd_parse_new_command(1);
                let arg = cmd_parse_new_argument();
                cmd_parse_arguments_push(&raw mut (*cmd).arguments, arg);
                cmd_parse_commands_push(cmds, cmd);
            }
            assert_eq!(cmd_parse_command_at(cmds, 0), command_ptr);
            assert_eq!(
                cmd_parse_argument_at(&raw mut (*command_ptr).arguments, 0),
                argument_ptr
            );
            cmd_parse_free_commands(cmds);
        }
    }
}
