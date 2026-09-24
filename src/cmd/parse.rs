use crate::src::arguments::args_free_value;
use crate::src::cmd::{
    cmd_get_alias, cmd_list_append, cmd_list_append_all, cmd_list_free, cmd_list_move,
    cmd_list_new, cmd_list_print_cstring, cmd_parse,
};
use crate::src::cmd_find::{cmd_find_from_client, cmd_find_valid_state};
use crate::src::cmd_queue::{cmdq_append, cmdq_get_command, cmdq_insert_after, cmdq_print};
use crate::src::environ::{environ_find, environ_put};
use crate::src::ffi::libc::{
    __ctype_b_loc, free, getc, getpwnam, getpwuid, getuid, malloc, memset, sscanf, strchr, strcmp,
    strlen, ungetc, wctomb,
};
use crate::src::format::{format_create, format_defaults, format_expand, format_free, format_true};
use crate::src::log::{fatalx, log_debug};
use crate::src::shared::abi::*;
pub use crate::src::shared::abi::{__gid_t, __off64_t, __off_t, __uid_t};
pub use crate::src::shared::account::passwd;
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{
    args, args_value, args_value_c2rust_unnamed, args_value_entry,
};
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files, overlay_check_cb,
    overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb, overlay_resize_cb,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmdq_state, cmds,
};
pub use crate::src::shared::command::{cmd_parse_input, cmd_parse_result};
pub use crate::src::shared::command::{
    CMD_PARSE_MAX_ENVIRON_LEN, CMD_PARSE_NOALIAS, CMD_PARSE_ONEGROUP, CMD_PARSE_PARSEONLY,
    CMD_PARSE_VERBOSE,
};
pub use crate::src::shared::control::control_state;
pub use crate::src::shared::ctype::{
    _ISalnum, _ISalpha, _ISblank, _IScntrl, _ISdigit, _ISgraph, _ISlower, _ISprint, _ISpunct,
    _ISspace, _ISupper, _ISxdigit, ctype_code,
};
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::ENVIRON_HIDDEN;
pub use crate::src::shared::environment::{environ, environ_entry};
use crate::src::shared::event::*;
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::format::{FORMAT_NOJOBS, FORMAT_NONE};
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
pub use crate::src::shared::limits::{__INT_MAX__, SIZE_MAX, UINT_MAX};
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
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
pub use crate::src::shared::stdio::EOF;
pub use crate::src::shared::stdio::{
    _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data, _IO_FILE, FILE,
};
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::utf8::wchar_t;
pub use crate::src::shared::variadic::{__builtin_va_list, __gnuc_va_list, __va_list_tag, va_list};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::tmux::global_environ;
use crate::src::xmalloc::{xasprintf, xcalloc, xmalloc, xstrdup, xvasprintf_cstring};
use libc;
use std::collections::VecDeque;
use std::ffi::{CStr, CString};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_14;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_13;

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
    pub string: *mut ::core::ffi::c_char,
    pub commands: *mut cmd_parse_commands,
    pub cmdlist: *mut cmd_list,
}
pub type cmd_parse_argument_type = ::core::ffi::c_uint;
pub const CMD_PARSE_PARSED_COMMANDS: cmd_parse_argument_type = 2;
pub const CMD_PARSE_COMMANDS: cmd_parse_argument_type = 1;
pub const CMD_PARSE_STRING: cmd_parse_argument_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_parse_state {
    pub f: *mut FILE,
    pub buf: *const ::core::ffi::c_char,
    pub len: size_t,
    pub off: size_t,
    pub condition: ::core::ffi::c_int,
    pub eol: ::core::ffi::c_int,
    pub eof: ::core::ffi::c_int,
    pub input: *mut cmd_parse_input,
    pub escapes: u_int,
    pub error: *mut ::core::ffi::c_char,
}
pub type C2RustUnnamed_44 = ::core::ffi::c_uint;
pub const SINGLE_QUOTES: C2RustUnnamed_44 = 3;
pub const DOUBLE_QUOTES: C2RustUnnamed_44 = 2;
pub const NONE: C2RustUnnamed_44 = 1;
pub const START: C2RustUnnamed_44 = 0;
static mut parse_state: cmd_parse_state = cmd_parse_state {
    f: ::core::ptr::null::<FILE>() as *mut FILE,
    buf: ::core::ptr::null::<::core::ffi::c_char>(),
    len: 0,
    off: 0,
    condition: 0,
    eol: 0,
    eof: 0,
    input: ::core::ptr::null::<cmd_parse_input>() as *mut cmd_parse_input,
    escapes: 0,
    error: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
};
pub const ERROR: ::core::ffi::c_int = 258 as ::core::ffi::c_int;
pub const HIDDEN: ::core::ffi::c_int = 259 as ::core::ffi::c_int;
pub const IF: ::core::ffi::c_int = 260 as ::core::ffi::c_int;
pub const ELSE: ::core::ffi::c_int = 261 as ::core::ffi::c_int;
pub const ELIF: ::core::ffi::c_int = 262 as ::core::ffi::c_int;
pub const ENDIF: ::core::ffi::c_int = 263 as ::core::ffi::c_int;
pub const FORMAT: ::core::ffi::c_int = 264 as ::core::ffi::c_int;
pub const TOKEN: ::core::ffi::c_int = 265 as ::core::ffi::c_int;
pub const EQUALS: ::core::ffi::c_int = 266 as ::core::ffi::c_int;
unsafe extern "C" fn cmd_parse_get_error(
    mut file: *const ::core::ffi::c_char,
    mut line: u_int,
    mut error: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if file.is_null() {
        s = xstrdup(error);
    } else {
        xasprintf(
            &raw mut s,
            b"%s:%u: %s\0" as *const u8 as *const ::core::ffi::c_char,
            file,
            line,
            error,
        );
    }
    return s;
}
unsafe extern "C" fn cmd_parse_print_commands(
    mut pi: *mut cmd_parse_input,
    mut cmdlist: *mut cmd_list,
) {
    if (*pi).item.is_null() || !(*pi).flags & CMD_PARSE_VERBOSE != 0 {
        return;
    }
    let s = cmd_list_print_cstring(cmdlist, 0);
    if !(*pi).file.is_null() {
        cmdq_print(
            (*pi).item,
            b"%s:%u: %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*pi).file,
            (*pi).line,
            s.as_ptr(),
        );
    } else {
        cmdq_print(
            (*pi).item,
            b"%u: %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*pi).line,
            s.as_ptr(),
        );
    }
}
unsafe fn cmd_parse_new_argument() -> *mut cmd_parse_argument {
    Box::into_raw(Box::new(cmd_parse_argument {
        type_0: CMD_PARSE_STRING,
        string: ::core::ptr::null_mut(),
        commands: ::core::ptr::null_mut(),
        cmdlist: ::core::ptr::null_mut(),
    }))
}
unsafe extern "C" fn cmd_parse_free_argument(mut arg: *mut cmd_parse_argument) {
    match (*arg).type_0 as ::core::ffi::c_uint {
        0 => {
            free((*arg).string as *mut ::core::ffi::c_void);
        }
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
unsafe extern "C" fn cmd_parse_free_arguments(mut args: *mut cmd_parse_arguments) {
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
unsafe extern "C" fn cmd_parse_free_command(mut cmd: *mut cmd_parse_command) {
    cmd_parse_free_arguments(&raw mut (*cmd).arguments);
    drop(Box::from_raw(cmd));
}
unsafe extern "C" fn cmd_parse_new_commands() -> *mut cmd_parse_commands {
    Box::into_raw(Box::new(cmd_parse_commands { items: Vec::new() }))
}
unsafe extern "C" fn cmd_parse_free_commands(mut cmds: *mut cmd_parse_commands) {
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
unsafe fn cmd_parse_arguments_append(dst: *mut cmd_parse_arguments, src: *mut cmd_parse_arguments) {
    (*dst).items.append(&mut (*src).items);
}
unsafe fn cmd_parse_commands_push(cmds: *mut cmd_parse_commands, cmd: *mut cmd_parse_command) {
    (*cmds).items.push(Box::from_raw(cmd));
}
unsafe fn cmd_parse_arguments_push(args: *mut cmd_parse_arguments, arg: *mut cmd_parse_argument) {
    (*args).items.push_back(Box::from_raw(arg));
}
// The lexer remains application-owned: variable and tilde expansion depend on
// the live environment. Pull one token at a time so assignments affect later words.
struct ParserContext;

impl hmux_cmdparse::Context for ParserContext {
    fn line(&self) -> u32 {
        unsafe { (*parse_state.input).line }
    }

    fn expand_format(&mut self, token: hmux_cmdparse::TokenText) -> hmux_cmdparse::TokenText {
        unsafe {
            let pi = parse_state.input;
            let mut fs: cmd_find_state = std::mem::zeroed();
            let fsp = if cmd_find_valid_state(&raw mut (*pi).fs) != 0 {
                &raw mut (*pi).fs
            } else {
                cmd_find_from_client(&raw mut fs, (*pi).c, 0);
                &raw mut fs
            };
            let ft = format_create((*pi).c, (*pi).item, FORMAT_NONE, FORMAT_NOJOBS);
            format_defaults(ft, (*pi).c, (*fsp).s, (*fsp).wl, (*fsp).wp);
            let expanded = format_expand(ft, token.as_c_str().as_ptr());
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
                yyerror(b"environment variable is too long\0".as_ptr().cast());
                return Err(hmux_cmdparse::LexError);
            }
            if (*parse_state.input).flags & CMD_PARSE_PARSEONLY == 0 && active {
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

// Lexer strings use malloc; copy before freeing instead of CString::from_raw.
unsafe fn take_parser_token(raw: *mut core::ffi::c_char) -> hmux_cmdparse::TokenText {
    let text = CStr::from_ptr(raw).to_owned();
    free(raw.cast());
    hmux_cmdparse::TokenText::from_cstring(text)
}

fn next_parser_token(
) -> Option<Result<(usize, hmux_cmdparse::Token, usize), hmux_cmdparse::LexError>> {
    use hmux_cmdparse::Token;
    unsafe {
        let mut lexer_token = core::ptr::null_mut();
        let token = match yylex(&mut lexer_token) {
            0 => return None,
            10 => Token::Newline,
            59 => Token::Semicolon,
            123 => Token::OpenBrace,
            125 => Token::CloseBrace,
            HIDDEN => Token::Hidden,
            IF => Token::If,
            ELSE => Token::Else,
            ELIF => Token::Elif,
            ENDIF => Token::Endif,
            kind @ (FORMAT | TOKEN | EQUALS) => {
                let text = take_parser_token(lexer_token);
                match kind {
                    FORMAT => Token::Format(text),
                    TOKEN => Token::Word(text),
                    _ => Token::Equals(text),
                }
            }
            _ => return Some(Err(hmux_cmdparse::LexError)),
        };
        Some(Ok((0, token, 0)))
    }
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
                    (*arg).string = xstrdup(text.as_c_str().as_ptr());
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

unsafe extern "C" fn cmd_parse_run_parser(
    cause: *mut *mut core::ffi::c_char,
) -> *mut cmd_parse_commands {
    match hmux_cmdparse::parse(&mut ParserContext, std::iter::from_fn(next_parser_token)) {
        Ok(commands) => build_parser_commands(commands),
        Err(_) => {
            yyerror(b"syntax error\0".as_ptr().cast());
            *cause = parse_state.error;
            core::ptr::null_mut()
        }
    }
}
unsafe extern "C" fn cmd_parse_do_file(
    mut f: *mut FILE,
    mut pi: *mut cmd_parse_input,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut cmd_parse_commands {
    let mut ps: *mut cmd_parse_state = &raw mut parse_state;
    memset(
        ps as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_parse_state>() as size_t,
    );
    (*ps).input = pi;
    (*ps).f = f;
    return cmd_parse_run_parser(cause);
}
unsafe extern "C" fn cmd_parse_do_buffer(
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut pi: *mut cmd_parse_input,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut cmd_parse_commands {
    let mut ps: *mut cmd_parse_state = &raw mut parse_state;
    memset(
        ps as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_parse_state>() as size_t,
    );
    (*ps).input = pi;
    (*ps).buf = buf;
    (*ps).len = len;
    return cmd_parse_run_parser(cause);
}
unsafe extern "C" fn cmd_parse_log_commands(
    mut cmds: *mut cmd_parse_commands,
    mut prefix: *const ::core::ffi::c_char,
) {
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
                    log_debug(
                        b"%s %u:%u: %s\0" as *const u8 as *const ::core::ffi::c_char,
                        prefix,
                        i,
                        j,
                        (*arg).string,
                    );
                }
                1 => {
                    let mut nested = CStr::from_ptr(prefix).to_bytes().to_vec();
                    nested.extend_from_slice(format!(" {i}:{j}").as_bytes());
                    let nested = CString::new(nested).expect("parser log prefix has no NUL");
                    cmd_parse_log_commands(
                        (*arg).commands as *mut cmd_parse_commands,
                        nested.as_ptr(),
                    );
                }
                2 => {
                    let s = cmd_list_print_cstring((*arg).cmdlist, 0);
                    log_debug(
                        b"%s %u:%u: %s\0" as *const u8 as *const ::core::ffi::c_char,
                        prefix,
                        i,
                        j,
                        s.as_ptr(),
                    );
                }
                _ => {}
            }
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn cmd_parse_expand_alias(
    mut cmd: *mut cmd_parse_command,
    mut pi: *mut cmd_parse_input,
    mut pr: *mut cmd_parse_result,
) -> ::core::ffi::c_int {
    let mut first: *mut cmd_parse_argument = ::core::ptr::null_mut::<cmd_parse_argument>();
    let mut cmds: *mut cmd_parse_commands = ::core::ptr::null_mut::<cmd_parse_commands>();
    let mut last: *mut cmd_parse_command = ::core::ptr::null_mut::<cmd_parse_command>();
    let mut alias: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*pi).flags & CMD_PARSE_NOALIAS != 0 {
        return 0 as ::core::ffi::c_int;
    }
    memset(
        pr as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_parse_result>() as size_t,
    );
    first = cmd_parse_arguments_first(&raw mut (*cmd).arguments);
    if first.is_null()
        || (*first).type_0 as ::core::ffi::c_uint
            != CMD_PARSE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*pr).status = CMD_PARSE_SUCCESS;
        (*pr).cmdlist = cmd_list_new();
        return 1 as ::core::ffi::c_int;
    }
    name = (*first).string;
    alias = cmd_get_alias(name);
    if alias.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"%s: %u alias %s = %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_parse_expand_alias\0" as *const u8 as *const ::core::ffi::c_char,
        (*pi).line,
        name,
        alias,
    );
    cmds = cmd_parse_do_buffer(alias, strlen(alias), pi, &raw mut cause);
    free(alias as *mut ::core::ffi::c_void);
    if cmds.is_null() {
        (*pr).status = CMD_PARSE_ERROR;
        (*pr).error = cause;
        return 1 as ::core::ffi::c_int;
    }
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
    cmd_parse_log_commands(
        cmds,
        b"cmd_parse_expand_alias\0" as *const u8 as *const ::core::ffi::c_char,
    );
    (*pi).flags |= CMD_PARSE_NOALIAS;
    cmd_parse_build_commands(cmds, pi, pr);
    (*pi).flags &= !CMD_PARSE_NOALIAS;
    cmd_parse_free_commands(cmds);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn cmd_parse_build_command(
    mut cmd: *mut cmd_parse_command,
    mut pi: *mut cmd_parse_input,
    mut pr: *mut cmd_parse_result,
) {
    let mut current_block: u64 = 16207960823932980356;
    let mut arg: *mut cmd_parse_argument = ::core::ptr::null_mut::<cmd_parse_argument>();
    let mut add: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut values = Vec::<args_value>::new();
    let mut count: u_int = 0 as u_int;
    let mut idx: u_int = 0;
    let mut argument_index: usize = 0;
    let argument_count = (*cmd).arguments.items.len();
    memset(
        pr as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_parse_result>() as size_t,
    );
    if cmd_parse_expand_alias(cmd, pi, pr) != 0 {
        return;
    }
    while argument_index < argument_count {
        arg = cmd_parse_argument_at(&raw mut (*cmd).arguments, argument_index);
        values.push(::core::mem::zeroed::<args_value>());
        let value = values.as_mut_ptr().add(count as usize);
        match (*arg).type_0 as ::core::ffi::c_uint {
            0 => {
                (*value).type_0 = ARGS_STRING;
                let ref mut fresh0 = (*value).c2rust_unnamed.string;
                *fresh0 = xstrdup((*arg).string);
            }
            1 => {
                cmd_parse_build_commands((*arg).commands as *mut cmd_parse_commands, pi, pr);
                if (*pr).status as ::core::ffi::c_uint
                    != CMD_PARSE_SUCCESS as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    current_block = 16207960823932980356;
                    break;
                }
                (*value).type_0 = ARGS_COMMANDS;
                let ref mut fresh1 = (*value).c2rust_unnamed.cmdlist;
                *fresh1 = (*pr).cmdlist as *mut cmd_list;
            }
            2 => {
                (*value).type_0 = ARGS_COMMANDS;
                let ref mut fresh2 = (*value).c2rust_unnamed.cmdlist;
                *fresh2 = (*arg).cmdlist as *mut cmd_list;
                let ref mut fresh3 = (*(*value).c2rust_unnamed.cmdlist).references;
                *fresh3 += 1;
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
            add = cmd_parse(
                if values.is_empty() {
                    ::core::ptr::null_mut::<args_value>()
                } else {
                    values.as_mut_ptr()
                },
                count,
                (*pi).file,
                (*pi).line,
                (*pi).flags,
                &raw mut cause,
            );
            if add.is_null() {
                (*pr).status = CMD_PARSE_ERROR;
                (*pr).error = cmd_parse_get_error((*pi).file, (*pi).line, cause);
                free(cause as *mut ::core::ffi::c_void);
            } else {
                (*pr).status = CMD_PARSE_SUCCESS;
                (*pr).cmdlist = cmd_list_new();
                cmd_list_append((*pr).cmdlist, add);
            }
        }
        _ => {}
    }
    idx = 0 as u_int;
    while idx < count {
        args_free_value(values.as_mut_ptr().add(idx as usize));
        idx = idx.wrapping_add(1);
    }
}
unsafe extern "C" fn cmd_parse_build_commands(
    mut cmds: *mut cmd_parse_commands,
    mut pi: *mut cmd_parse_input,
    mut pr: *mut cmd_parse_result,
) {
    let mut cmd: *mut cmd_parse_command = ::core::ptr::null_mut::<cmd_parse_command>();
    let mut line: u_int = UINT_MAX;
    let mut current: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    let mut result: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    memset(
        pr as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_parse_result>() as size_t,
    );
    let command_count = (*cmds).items.len();
    if command_count == 0 {
        (*pr).status = CMD_PARSE_SUCCESS;
        (*pr).cmdlist = cmd_list_new();
        return;
    }
    cmd_parse_log_commands(
        cmds,
        b"cmd_parse_build_commands\0" as *const u8 as *const ::core::ffi::c_char,
    );
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
    let s = cmd_list_print_cstring(result, 0);
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_parse_build_commands\0" as *const u8 as *const ::core::ffi::c_char,
        s.as_ptr(),
    );
    (*pr).status = CMD_PARSE_SUCCESS;
    (*pr).cmdlist = result;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_parse_from_file(
    mut f: *mut FILE,
    mut pi: *mut cmd_parse_input,
) -> *mut cmd_parse_result {
    static mut pr: cmd_parse_result = cmd_parse_result {
        status: CMD_PARSE_ERROR,
        cmdlist: ::core::ptr::null::<cmd_list>() as *mut cmd_list,
        error: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
    };
    let mut input: cmd_parse_input = cmd_parse_input {
        flags: 0,
        file: ::core::ptr::null::<::core::ffi::c_char>(),
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
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if pi.is_null() {
        memset(
            &raw mut input as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<cmd_parse_input>() as size_t,
        );
        pi = &raw mut input;
    }
    memset(
        &raw mut pr as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_parse_result>() as size_t,
    );
    cmds = cmd_parse_do_file(f, pi, &raw mut cause);
    if cmds.is_null() {
        pr.status = CMD_PARSE_ERROR;
        pr.error = cause;
        return &raw mut pr;
    }
    cmd_parse_build_commands(cmds, pi, &raw mut pr);
    cmd_parse_free_commands(cmds);
    return &raw mut pr;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_parse_from_string(
    mut s: *const ::core::ffi::c_char,
    mut pi: *mut cmd_parse_input,
) -> *mut cmd_parse_result {
    let mut input: cmd_parse_input = cmd_parse_input {
        flags: 0,
        file: ::core::ptr::null::<::core::ffi::c_char>(),
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
        memset(
            &raw mut input as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<cmd_parse_input>() as size_t,
        );
        pi = &raw mut input;
    }
    (*pi).flags |= CMD_PARSE_ONEGROUP;
    return cmd_parse_from_buffer(s as *const ::core::ffi::c_void, strlen(s), pi);
}
#[no_mangle]
pub unsafe extern "C" fn cmd_parse_and_insert(
    mut s: *const ::core::ffi::c_char,
    mut pi: *mut cmd_parse_input,
    mut after: *mut cmdq_item,
    mut state: *mut cmdq_state,
    mut error: *mut *mut ::core::ffi::c_char,
) -> cmd_parse_status {
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    pr = cmd_parse_from_string(s, pi);
    match (*pr).status as ::core::ffi::c_uint {
        0 => {
            if !error.is_null() {
                *error = (*pr).error;
            } else {
                free((*pr).error as *mut ::core::ffi::c_void);
            }
        }
        1 => {
            item = cmdq_get_command((*pr).cmdlist, state);
            cmdq_insert_after(after, item);
            cmd_list_free((*pr).cmdlist);
        }
        _ => {}
    }
    return (*pr).status;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_parse_and_append(
    mut s: *const ::core::ffi::c_char,
    mut pi: *mut cmd_parse_input,
    mut c: *mut client,
    mut state: *mut cmdq_state,
    mut error: *mut *mut ::core::ffi::c_char,
) -> cmd_parse_status {
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    pr = cmd_parse_from_string(s, pi);
    match (*pr).status as ::core::ffi::c_uint {
        0 => {
            if !error.is_null() {
                *error = (*pr).error;
            } else {
                free((*pr).error as *mut ::core::ffi::c_void);
            }
        }
        1 => {
            item = cmdq_get_command((*pr).cmdlist, state);
            cmdq_append(c, item);
            cmd_list_free((*pr).cmdlist);
        }
        _ => {}
    }
    return (*pr).status;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_parse_from_buffer(
    mut buf: *const ::core::ffi::c_void,
    mut len: size_t,
    mut pi: *mut cmd_parse_input,
) -> *mut cmd_parse_result {
    static mut pr: cmd_parse_result = cmd_parse_result {
        status: CMD_PARSE_ERROR,
        cmdlist: ::core::ptr::null::<cmd_list>() as *mut cmd_list,
        error: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
    };
    let mut input: cmd_parse_input = cmd_parse_input {
        flags: 0,
        file: ::core::ptr::null::<::core::ffi::c_char>(),
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
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if pi.is_null() {
        memset(
            &raw mut input as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<cmd_parse_input>() as size_t,
        );
        pi = &raw mut input;
    }
    memset(
        &raw mut pr as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_parse_result>() as size_t,
    );
    if len == 0 as size_t {
        pr.status = CMD_PARSE_SUCCESS;
        pr.cmdlist = cmd_list_new();
        return &raw mut pr;
    }
    cmds = cmd_parse_do_buffer(buf as *const ::core::ffi::c_char, len, pi, &raw mut cause);
    if cmds.is_null() {
        pr.status = CMD_PARSE_ERROR;
        pr.error = cause;
        return &raw mut pr;
    }
    cmd_parse_build_commands(cmds, pi, &raw mut pr);
    cmd_parse_free_commands(cmds);
    return &raw mut pr;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_parse_from_arguments(
    mut values: *mut args_value,
    mut count: u_int,
    mut pi: *mut cmd_parse_input,
) -> *mut cmd_parse_result {
    static mut pr: cmd_parse_result = cmd_parse_result {
        status: CMD_PARSE_ERROR,
        cmdlist: ::core::ptr::null::<cmd_list>() as *mut cmd_list,
        error: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
    };
    let mut input: cmd_parse_input = cmd_parse_input {
        flags: 0,
        file: ::core::ptr::null::<::core::ffi::c_char>(),
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
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut size: size_t = 0;
    let mut end: ::core::ffi::c_int = 0;
    if pi.is_null() {
        memset(
            &raw mut input as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<cmd_parse_input>() as size_t,
        );
        pi = &raw mut input;
    }
    memset(
        &raw mut pr as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_parse_result>() as size_t,
    );
    cmds = cmd_parse_new_commands();
    cmd = cmd_parse_new_command((*pi).line);
    i = 0 as u_int;
    while i < count {
        end = 0 as ::core::ffi::c_int;
        if (*values.offset(i as isize)).type_0 as ::core::ffi::c_uint
            == ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            copy = xstrdup((*values.offset(i as isize)).c2rust_unnamed.string);
            size = strlen(copy);
            if size != 0 as size_t
                && *copy.offset(size.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    == ';' as i32
            {
                size = size.wrapping_sub(1);
                *copy.offset(size as isize) = '\0' as i32 as ::core::ffi::c_char;
                if size > 0 as size_t
                    && *copy.offset(size.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                        == '\\' as i32
                {
                    *copy.offset(size.wrapping_sub(1 as size_t) as isize) =
                        ';' as i32 as ::core::ffi::c_char;
                } else {
                    end = 1 as ::core::ffi::c_int;
                }
            }
            if end == 0 || size != 0 as size_t {
                arg = cmd_parse_new_argument();
                (*arg).type_0 = CMD_PARSE_STRING;
                (*arg).string = copy;
                cmd_parse_arguments_push(&raw mut (*cmd).arguments, arg);
            } else {
                free(copy as *mut ::core::ffi::c_void);
            }
        } else if (*values.offset(i as isize)).type_0 as ::core::ffi::c_uint
            == ARGS_COMMANDS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            arg = cmd_parse_new_argument();
            (*arg).type_0 = CMD_PARSE_PARSED_COMMANDS;
            (*arg).cmdlist = (*values.offset(i as isize)).c2rust_unnamed.cmdlist as *mut cmd_list;
            (*(*arg).cmdlist).references += 1;
            cmd_parse_arguments_push(&raw mut (*cmd).arguments, arg);
        } else {
            fatalx(b"unknown argument type\0" as *const u8 as *const ::core::ffi::c_char);
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
    return &raw mut pr;
}
unsafe extern "C" fn yyerror(mut fmt: *const ::core::ffi::c_char, mut args: ...) {
    let mut ps: *mut cmd_parse_state = &raw mut parse_state;
    let mut pi: *mut cmd_parse_input = (*ps).input;
    let mut ap: ::core::ffi::VaList;
    if !(*ps).error.is_null() {
        return;
    }
    ap = args.clone();
    let error = xvasprintf_cstring(fmt, ap);
    (*ps).error = cmd_parse_get_error((*pi).file, (*pi).line, error.as_ptr());
}
unsafe extern "C" fn yylex_is_var(
    mut ch: ::core::ffi::c_char,
    mut first: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if ch as ::core::ffi::c_int == '=' as i32 {
        return 0 as ::core::ffi::c_int;
    }
    if first != 0
        && *(*__ctype_b_loc()).offset(ch as u_char as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
    {
        return 0 as ::core::ffi::c_int;
    }
    return (*(*__ctype_b_loc()).offset(ch as u_char as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
        || ch as ::core::ffi::c_int == '_' as i32) as ::core::ffi::c_int;
}
/// A lexer scratch buffer. Only completed tokens need a libc allocation for
/// the generated parser's existing free and transfer paths.
struct LexerBuffer {
    bytes: Vec<u8>,
}

impl LexerBuffer {
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    unsafe fn append(&mut self, add: *const ::core::ffi::c_char, addlen: size_t) {
        if self
            .bytes
            .len()
            .checked_add(addlen)
            .and_then(|len| len.checked_add(1))
            .is_none()
        {
            fatalx(b"buffer is too big\0" as *const u8 as *const ::core::ffi::c_char);
        }
        if addlen == 0 {
            return;
        }
        self.bytes
            .extend_from_slice(std::slice::from_raw_parts(add.cast(), addlen));
    }

    fn push(&mut self, byte: ::core::ffi::c_char) {
        if self.bytes.len() == SIZE_MAX as usize - 1 {
            unsafe { fatalx(b"buffer is too big\0" as *const u8 as *const ::core::ffi::c_char) };
        }
        self.bytes.push(byte as u8);
    }

    unsafe fn into_raw(self) -> *mut ::core::ffi::c_char {
        let size = self.bytes.len() + 1;
        let buf = xmalloc(size) as *mut ::core::ffi::c_char;
        ::core::ptr::copy_nonoverlapping(self.bytes.as_ptr(), buf.cast(), self.bytes.len());
        *buf.add(self.bytes.len()) = 0;
        buf
    }
}
unsafe extern "C" fn yylex_getc1() -> ::core::ffi::c_int {
    let mut ps: *mut cmd_parse_state = &raw mut parse_state;
    let mut ch: ::core::ffi::c_int = 0;
    if !(*ps).f.is_null() {
        ch = getc((*ps).f);
    } else if (*ps).off == (*ps).len {
        ch = EOF;
    } else {
        let fresh27 = (*ps).off;
        (*ps).off = (*ps).off.wrapping_add(1);
        ch = *(*ps).buf.offset(fresh27 as isize) as ::core::ffi::c_int;
    }
    return ch;
}
unsafe extern "C" fn yylex_ungetc(mut ch: ::core::ffi::c_int) {
    let mut ps: *mut cmd_parse_state = &raw mut parse_state;
    if !(*ps).f.is_null() {
        ungetc(ch, (*ps).f);
    } else if (*ps).off > 0 as size_t && ch != EOF {
        (*ps).off = (*ps).off.wrapping_sub(1);
    }
}
unsafe extern "C" fn yylex_getc() -> ::core::ffi::c_int {
    let mut ps: *mut cmd_parse_state = &raw mut parse_state;
    let mut ch: ::core::ffi::c_int = 0;
    if (*ps).escapes != 0 as u_int {
        (*ps).escapes = (*ps).escapes.wrapping_sub(1);
        return '\\' as i32;
    }
    loop {
        ch = yylex_getc1();
        if ch == '\\' as i32 {
            (*ps).escapes = (*ps).escapes.wrapping_add(1);
        } else if ch == '\n' as i32 && (*ps).escapes.wrapping_rem(2 as u_int) == 1 as u_int {
            (*(*ps).input).line = (*(*ps).input).line.wrapping_add(1);
            (*ps).escapes = (*ps).escapes.wrapping_sub(1);
        } else {
            if (*ps).escapes != 0 as u_int {
                yylex_ungetc(ch);
                (*ps).escapes = (*ps).escapes.wrapping_sub(1);
                return '\\' as i32;
            }
            return ch;
        }
    }
}
unsafe extern "C" fn yylex_get_word(mut ch: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    let mut buf = LexerBuffer::new();
    loop {
        buf.push(ch as ::core::ffi::c_char);
        ch = yylex_getc();
        if !(ch != EOF
            && strchr(b" \t\n\0" as *const u8 as *const ::core::ffi::c_char, ch).is_null())
        {
            break;
        }
    }
    yylex_ungetc(ch);
    let buf = buf.into_raw();
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"yylex_get_word\0" as *const u8 as *const ::core::ffi::c_char,
        buf,
    );
    return buf;
}
unsafe fn yylex(lexed: &mut *mut core::ffi::c_char) -> ::core::ffi::c_int {
    let mut ps: *mut cmd_parse_state = &raw mut parse_state;
    let mut token: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ch: ::core::ffi::c_int = 0;
    let mut next: ::core::ffi::c_int = 0;
    let mut condition: ::core::ffi::c_int = 0;
    if (*ps).eol != 0 {
        (*(*ps).input).line = (*(*ps).input).line.wrapping_add(1);
    }
    (*ps).eol = 0 as ::core::ffi::c_int;
    condition = (*ps).condition;
    (*ps).condition = 0 as ::core::ffi::c_int;
    loop {
        ch = yylex_getc();
        if ch == EOF {
            if (*ps).eof != 0 {
                break;
            }
            (*ps).eof = 1 as ::core::ffi::c_int;
            return '\n' as i32;
        } else {
            if ch == ' ' as i32 || ch == '\t' as i32 {
                continue;
            }
            if ch == '\r' as i32 {
                ch = yylex_getc();
                if ch != '\n' as i32 {
                    yylex_ungetc(ch);
                    ch = '\r' as i32;
                }
            }
            if ch == '\n' as i32 {
                (*ps).eol = 1 as ::core::ffi::c_int;
                return '\n' as i32;
            }
            if ch == ';' as i32 || ch == '{' as i32 || ch == '}' as i32 {
                return ch;
            }
            if ch == '#' as i32 {
                next = yylex_getc();
                if condition != 0 && next == '{' as i32 {
                    *lexed = yylex_format();
                    if lexed.is_null() {
                        return 258 as ::core::ffi::c_int;
                    }
                    return 264 as ::core::ffi::c_int;
                }
                while next != '\n' as i32 && next != EOF {
                    next = yylex_getc();
                }
                if next == '\n' as i32 {
                    (*(*ps).input).line = (*(*ps).input).line.wrapping_add(1);
                    return '\n' as i32;
                }
            } else {
                if ch == '%' as i32 {
                    *lexed = yylex_get_word('%' as i32);
                    cp = *lexed;
                    while *cp as ::core::ffi::c_int != '\0' as i32 {
                        if *cp as ::core::ffi::c_int != '%' as i32
                            && *(*__ctype_b_loc())
                                .offset(*cp as u_char as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort
                                    as ::core::ffi::c_int
                                == 0
                        {
                            break;
                        }
                        cp = cp.offset(1);
                    }
                    if *cp as ::core::ffi::c_int == '\0' as i32 {
                        return 265 as ::core::ffi::c_int;
                    }
                    (*ps).condition = 1 as ::core::ffi::c_int;
                    if strcmp(
                        *lexed,
                        b"%hidden\0" as *const u8 as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                    {
                        free(*lexed as *mut ::core::ffi::c_void);
                        return 259 as ::core::ffi::c_int;
                    }
                    if strcmp(*lexed, b"%if\0" as *const u8 as *const ::core::ffi::c_char)
                        == 0 as ::core::ffi::c_int
                    {
                        free(*lexed as *mut ::core::ffi::c_void);
                        return 260 as ::core::ffi::c_int;
                    }
                    if strcmp(
                        *lexed,
                        b"%else\0" as *const u8 as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                    {
                        free(*lexed as *mut ::core::ffi::c_void);
                        return 261 as ::core::ffi::c_int;
                    }
                    if strcmp(
                        *lexed,
                        b"%elif\0" as *const u8 as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                    {
                        free(*lexed as *mut ::core::ffi::c_void);
                        return 262 as ::core::ffi::c_int;
                    }
                    if strcmp(
                        *lexed,
                        b"%endif\0" as *const u8 as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                    {
                        free(*lexed as *mut ::core::ffi::c_void);
                        return 263 as ::core::ffi::c_int;
                    }
                    free(*lexed as *mut ::core::ffi::c_void);
                    return 258 as ::core::ffi::c_int;
                }
                token = yylex_token(ch);
                if token.is_null() {
                    return 258 as ::core::ffi::c_int;
                }
                *lexed = token;
                if !strchr(token, '=' as i32).is_null()
                    && yylex_is_var(*token, 1 as ::core::ffi::c_int) != 0
                {
                    cp = token.offset(1 as ::core::ffi::c_int as isize);
                    while *cp as ::core::ffi::c_int != '=' as i32 {
                        if yylex_is_var(*cp, 0 as ::core::ffi::c_int) == 0 {
                            break;
                        }
                        cp = cp.offset(1);
                    }
                    if *cp as ::core::ffi::c_int == '=' as i32 {
                        return 266 as ::core::ffi::c_int;
                    }
                }
                return 265 as ::core::ffi::c_int;
            }
        }
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn yylex_format() -> *mut ::core::ffi::c_char {
    let mut current_block: u64;
    let mut buf = LexerBuffer::new();
    let mut ch: ::core::ffi::c_int = 0;
    let mut brackets: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    buf.append(b"#{".as_ptr().cast(), 2);
    loop {
        ch = yylex_getc();
        if ch == EOF || ch == '\n' as i32 {
            current_block = 14771530845615209615;
            break;
        }
        if ch == '#' as i32 {
            ch = yylex_getc();
            if ch == EOF || ch == '\n' as i32 {
                current_block = 14771530845615209615;
                break;
            }
            if ch == '{' as i32 {
                brackets += 1;
            }
            buf.push('#' as i32 as ::core::ffi::c_char);
        } else if ch == '}' as i32 {
            if brackets != 0 as ::core::ffi::c_int && {
                brackets -= 1;
                brackets == 0 as ::core::ffi::c_int
            } {
                buf.push(ch as ::core::ffi::c_char);
                current_block = 10048703153582371463;
                break;
            }
        }
        buf.push(ch as ::core::ffi::c_char);
    }
    match current_block {
        10048703153582371463 => {
            if !(brackets != 0 as ::core::ffi::c_int) {
                let buf = buf.into_raw();
                log_debug(
                    b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    b"yylex_format\0" as *const u8 as *const ::core::ffi::c_char,
                    buf,
                );
                return buf;
            }
        }
        _ => {}
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe fn yylex_token_escape(buf: &mut LexerBuffer) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut ch: ::core::ffi::c_int = 0;
    let mut type_0: ::core::ffi::c_int = 0;
    let mut o2: ::core::ffi::c_int = 0;
    let mut o3: ::core::ffi::c_int = 0;
    let mut mlen: ::core::ffi::c_int = 0;
    let mut size: u_int = 0;
    let mut i: u_int = 0;
    let mut tmp: u_int = 0;
    let mut s: [::core::ffi::c_char; 9] = [0; 9];
    let mut m: [::core::ffi::c_char; 16] = [0; 16];
    ch = yylex_getc();
    if ch >= '4' as i32 && ch <= '7' as i32 {
        yyerror(b"invalid octal escape\0" as *const u8 as *const ::core::ffi::c_char);
        return 0 as ::core::ffi::c_int;
    }
    if ch >= '0' as i32 && ch <= '3' as i32 {
        o2 = yylex_getc();
        if o2 >= '0' as i32 && o2 <= '7' as i32 {
            o3 = yylex_getc();
            if o3 >= '0' as i32 && o3 <= '7' as i32 {
                ch = 64 as ::core::ffi::c_int * (ch - '0' as i32)
                    + 8 as ::core::ffi::c_int * (o2 - '0' as i32)
                    + (o3 - '0' as i32);
                buf.push(ch as ::core::ffi::c_char);
                return 1 as ::core::ffi::c_int;
            }
        }
        yyerror(b"invalid octal escape\0" as *const u8 as *const ::core::ffi::c_char);
        return 0 as ::core::ffi::c_int;
    }
    match ch {
        EOF => return 0 as ::core::ffi::c_int,
        97 => {
            ch = '\u{7}' as i32;
            current_block = 17281240262373992796;
        }
        98 => {
            ch = '\u{8}' as i32;
            current_block = 17281240262373992796;
        }
        101 => {
            ch = '\u{1b}' as i32;
            current_block = 17281240262373992796;
        }
        102 => {
            ch = '\u{c}' as i32;
            current_block = 17281240262373992796;
        }
        115 => {
            ch = ' ' as i32;
            current_block = 17281240262373992796;
        }
        118 => {
            ch = '\u{b}' as i32;
            current_block = 17281240262373992796;
        }
        114 => {
            ch = '\r' as i32;
            current_block = 17281240262373992796;
        }
        110 => {
            ch = '\n' as i32;
            current_block = 17281240262373992796;
        }
        116 => {
            ch = '\t' as i32;
            current_block = 17281240262373992796;
        }
        117 => {
            type_0 = 'u' as i32;
            size = 4 as u_int;
            current_block = 16115080005730334406;
        }
        85 => {
            type_0 = 'U' as i32;
            size = 8 as u_int;
            current_block = 16115080005730334406;
        }
        _ => {
            current_block = 17281240262373992796;
        }
    }
    match current_block {
        16115080005730334406 => {
            i = 0 as u_int;
            while i < size {
                ch = yylex_getc();
                if ch == EOF || ch == '\n' as i32 {
                    return 0 as ::core::ffi::c_int;
                }
                if *(*__ctype_b_loc()).offset(ch as u_char as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & _ISxdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    == 0
                {
                    yyerror(
                        b"invalid \\%c argument\0" as *const u8 as *const ::core::ffi::c_char,
                        type_0,
                    );
                    return 0 as ::core::ffi::c_int;
                }
                s[i as usize] = ch as ::core::ffi::c_char;
                i = i.wrapping_add(1);
            }
            s[i as usize] = '\0' as i32 as ::core::ffi::c_char;
            if size == 4 as u_int
                && sscanf(
                    &raw mut s as *mut ::core::ffi::c_char,
                    b"%4x\0" as *const u8 as *const ::core::ffi::c_char,
                    &raw mut tmp,
                ) != 1 as ::core::ffi::c_int
                || size == 8 as u_int
                    && sscanf(
                        &raw mut s as *mut ::core::ffi::c_char,
                        b"%8x\0" as *const u8 as *const ::core::ffi::c_char,
                        &raw mut tmp,
                    ) != 1 as ::core::ffi::c_int
            {
                yyerror(
                    b"invalid \\%c argument\0" as *const u8 as *const ::core::ffi::c_char,
                    type_0,
                );
                return 0 as ::core::ffi::c_int;
            }
            mlen = wctomb(&raw mut m as *mut ::core::ffi::c_char, tmp as wchar_t);
            if mlen <= 0 as ::core::ffi::c_int
                || mlen > ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as ::core::ffi::c_int
            {
                yyerror(
                    b"invalid \\%c argument\0" as *const u8 as *const ::core::ffi::c_char,
                    type_0,
                );
                return 0 as ::core::ffi::c_int;
            }
            buf.append(&raw mut m as *mut ::core::ffi::c_char, mlen as size_t);
            return 1 as ::core::ffi::c_int;
        }
        _ => {
            buf.push(ch as ::core::ffi::c_char);
            return 1 as ::core::ffi::c_int;
        }
    };
}
unsafe fn yylex_token_variable(buf: &mut LexerBuffer) -> ::core::ffi::c_int {
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut ch: ::core::ffi::c_int = 0;
    let mut brackets: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut name: [::core::ffi::c_char; 1024] = [0; 1024];
    let mut namelen: size_t = 0 as size_t;
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    ch = yylex_getc();
    if ch == EOF {
        return 0 as ::core::ffi::c_int;
    }
    if ch == '{' as i32 {
        brackets = 1 as ::core::ffi::c_int;
    } else {
        if yylex_is_var(ch as ::core::ffi::c_char, 1 as ::core::ffi::c_int) == 0 {
            buf.push('$' as i32 as ::core::ffi::c_char);
            yylex_ungetc(ch);
            return 1 as ::core::ffi::c_int;
        }
        let fresh28 = namelen;
        namelen = namelen.wrapping_add(1);
        name[fresh28 as usize] = ch as ::core::ffi::c_char;
    }
    loop {
        ch = yylex_getc();
        if brackets != 0 && ch == '}' as i32 {
            break;
        }
        if ch == EOF || yylex_is_var(ch as ::core::ffi::c_char, 0 as ::core::ffi::c_int) == 0 {
            if brackets == 0 {
                yylex_ungetc(ch);
                break;
            } else {
                yyerror(
                    b"invalid environment variable\0" as *const u8 as *const ::core::ffi::c_char,
                );
                return 0 as ::core::ffi::c_int;
            }
        } else {
            if namelen
                == (::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as usize)
                    .wrapping_sub(2 as usize)
            {
                yyerror(
                    b"environment variable is too long\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
                return 0 as ::core::ffi::c_int;
            }
            let fresh29 = namelen;
            namelen = namelen.wrapping_add(1);
            name[fresh29 as usize] = ch as ::core::ffi::c_char;
        }
    }
    name[namelen as usize] = '\0' as i32 as ::core::ffi::c_char;
    envent = environ_find(global_environ, &raw mut name as *mut ::core::ffi::c_char);
    if !envent.is_null() && !(*envent).value.is_none() {
        value = ((*envent).value)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
        log_debug(
            b"%s: %s -> %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"yylex_token_variable\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut name as *mut ::core::ffi::c_char,
            value,
        );
        buf.append(value, strlen(value));
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn yylex_token_tilde(buf: &mut LexerBuffer) -> ::core::ffi::c_int {
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut ch: ::core::ffi::c_int = 0;
    let mut name: [::core::ffi::c_char; 1024] = [0; 1024];
    let mut namelen: size_t = 0 as size_t;
    let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
    let mut home: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    loop {
        ch = yylex_getc();
        if ch == EOF
            || !strchr(
                b"/ \t\n\"'\0" as *const u8 as *const ::core::ffi::c_char,
                ch,
            )
            .is_null()
        {
            yylex_ungetc(ch);
            break;
        } else {
            if namelen
                == (::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as usize)
                    .wrapping_sub(2 as usize)
            {
                yyerror(b"user name is too long\0" as *const u8 as *const ::core::ffi::c_char);
                return 0 as ::core::ffi::c_int;
            }
            let fresh30 = namelen;
            namelen = namelen.wrapping_add(1);
            name[fresh30 as usize] = ch as ::core::ffi::c_char;
        }
    }
    name[namelen as usize] = '\0' as i32 as ::core::ffi::c_char;
    if *(&raw mut name as *mut ::core::ffi::c_char) as ::core::ffi::c_int == '\0' as i32 {
        envent = environ_find(
            global_environ,
            b"HOME\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if !envent.is_null()
            && !(*envent).value.is_none()
            && *(*envent)
                .value
                .as_ref()
                .expect("environment value is present")
                .as_ptr() as ::core::ffi::c_int
                != '\0' as i32
        {
            home = ((*envent).value)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
        } else {
            pw = getpwuid(getuid());
            if !pw.is_null() {
                home = (*pw).pw_dir;
            }
        }
    } else {
        pw = getpwnam(&raw mut name as *mut ::core::ffi::c_char);
        if !pw.is_null() {
            home = (*pw).pw_dir;
        }
    }
    if home.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"%s: ~%s -> %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"yylex_token_tilde\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut name as *mut ::core::ffi::c_char,
        home,
    );
    buf.append(home, strlen(home));
    return 1 as ::core::ffi::c_int;
}

#[cfg(test)]
mod parser_collection_tests {
    use super::*;

    unsafe fn parse_commands(input: &[u8]) -> Vec<Vec<Vec<u8>>> {
        let mut pi: cmd_parse_input = ::core::mem::zeroed();
        pi.line = 1;
        let mut cause = ::core::ptr::null_mut();
        let cmds = cmd_parse_do_buffer(input.as_ptr().cast(), input.len(), &mut pi, &mut cause);
        assert!(
            !cmds.is_null(),
            "parser failed: {:?}",
            if cause.is_null() {
                None
            } else {
                Some(CStr::from_ptr(cause).to_string_lossy().into_owned())
            }
        );
        let result = (*cmds)
            .items
            .iter()
            .map(|cmd| {
                cmd.arguments
                    .items
                    .iter()
                    .map(|arg| CStr::from_ptr(arg.string).to_bytes().to_vec())
                    .collect()
            })
            .collect();
        cmd_parse_free_commands(cmds);
        result
    }

    #[test]
    fn parser_collections_preserve_order_scope_and_element_addresses() {
        let _guard = crate::src::cfg::CFG_TEST_LOCK.lock().unwrap();
        unsafe {
            let mut pi: cmd_parse_input = ::core::mem::zeroed();
            pi.line = 1;
            let malformed = b"%if 1\ndisplay-message unfinished\n";
            let mut cause = ::core::ptr::null_mut();
            let failed = cmd_parse_do_buffer(
                malformed.as_ptr().cast(),
                malformed.len(),
                &mut pi,
                &mut cause,
            );
            assert!(failed.is_null());
            if !cause.is_null() {
                free(cause.cast());
            }

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
unsafe extern "C" fn yylex_token(mut ch: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    let mut current_block: u64;
    let mut ps: *mut cmd_parse_state = &raw mut parse_state;
    let mut buf = LexerBuffer::new();
    let mut state: C2RustUnnamed_44 = NONE;
    let mut last: C2RustUnnamed_44 = START;
    loop {
        if ch == EOF {
            log_debug(
                b"%s: end at EOF\0" as *const u8 as *const ::core::ffi::c_char,
                b"yylex_token\0" as *const u8 as *const ::core::ffi::c_char,
            );
            current_block = 13321564401369230990;
            break;
        } else {
            if state as ::core::ffi::c_uint == NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                && ch == '\r' as i32
            {
                ch = yylex_getc();
                if ch != '\n' as i32 {
                    yylex_ungetc(ch);
                    ch = '\r' as i32;
                }
            }
            if ch == '\n' as i32 {
                if state as ::core::ffi::c_uint == NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    log_debug(
                        b"%s: end at EOL\0" as *const u8 as *const ::core::ffi::c_char,
                        b"yylex_token\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                    current_block = 13321564401369230990;
                    break;
                } else {
                    (*(*ps).input).line = (*(*ps).input).line.wrapping_add(1);
                }
            }
            if state as ::core::ffi::c_uint == NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                && (ch == ' ' as i32 || ch == '\t' as i32)
            {
                log_debug(
                    b"%s: end at WS\0" as *const u8 as *const ::core::ffi::c_char,
                    b"yylex_token\0" as *const u8 as *const ::core::ffi::c_char,
                );
                current_block = 13321564401369230990;
                break;
            } else if state as ::core::ffi::c_uint
                == NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                && (ch == ';' as i32 || ch == '}' as i32)
            {
                log_debug(
                    b"%s: end at %c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"yylex_token\0" as *const u8 as *const ::core::ffi::c_char,
                    ch,
                );
                current_block = 13321564401369230990;
                break;
            } else if ch == '\n' as i32
                && state as ::core::ffi::c_uint != NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                buf.push('\n' as i32 as ::core::ffi::c_char);
                loop {
                    ch = yylex_getc();
                    if !(ch == ' ' as i32 || ch == '\t' as i32) {
                        break;
                    }
                }
                if ch != '#' as i32 {
                    continue;
                }
                ch = yylex_getc();
                if !strchr(b",#{}:\0" as *const u8 as *const ::core::ffi::c_char, ch).is_null() {
                    yylex_ungetc(ch);
                    ch = '#' as i32;
                } else {
                    loop {
                        ch = yylex_getc();
                        if !(ch != '\n' as i32 && ch != EOF) {
                            break;
                        }
                    }
                }
            } else {
                if ch == '\\' as i32
                    && state as ::core::ffi::c_uint
                        != SINGLE_QUOTES as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if yylex_token_escape(&mut buf) == 0 {
                        current_block = 9856007333916341158;
                        break;
                    }
                    current_block = 2515528795579319319;
                } else if ch == '~' as i32
                    && last as ::core::ffi::c_uint != state as ::core::ffi::c_uint
                    && state as ::core::ffi::c_uint
                        != SINGLE_QUOTES as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if yylex_token_tilde(&mut buf) == 0 {
                        current_block = 9856007333916341158;
                        break;
                    }
                    current_block = 2515528795579319319;
                } else if ch == '$' as i32
                    && state as ::core::ffi::c_uint
                        != SINGLE_QUOTES as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if yylex_token_variable(&mut buf) == 0 {
                        current_block = 9856007333916341158;
                        break;
                    }
                    current_block = 2515528795579319319;
                } else {
                    if ch == '}' as i32
                        && state as ::core::ffi::c_uint
                            == NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        current_block = 9856007333916341158;
                        break;
                    }
                    if ch == '\'' as i32 {
                        if state as ::core::ffi::c_uint
                            == NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            state = SINGLE_QUOTES;
                            current_block = 13302697912176427116;
                        } else if state as ::core::ffi::c_uint
                            == SINGLE_QUOTES as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            state = NONE;
                            current_block = 13302697912176427116;
                        } else {
                            current_block = 1847472278776910194;
                        }
                    } else {
                        current_block = 1847472278776910194;
                    }
                    match current_block {
                        13302697912176427116 => {}
                        _ => {
                            if ch == '"' as i32 {
                                if state as ::core::ffi::c_uint
                                    == NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    state = DOUBLE_QUOTES;
                                    current_block = 13302697912176427116;
                                } else if state as ::core::ffi::c_uint
                                    == DOUBLE_QUOTES as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    state = NONE;
                                    current_block = 13302697912176427116;
                                } else {
                                    current_block = 14220266465818359136;
                                }
                            } else {
                                current_block = 14220266465818359136;
                            }
                            match current_block {
                                13302697912176427116 => {}
                                _ => {
                                    buf.push(ch as ::core::ffi::c_char);
                                    current_block = 2515528795579319319;
                                }
                            }
                        }
                    }
                }
                match current_block {
                    2515528795579319319 => {
                        last = state;
                    }
                    _ => {}
                }
                ch = yylex_getc();
            }
        }
    }
    match current_block {
        9856007333916341158 => {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        _ => {
            yylex_ungetc(ch);
            let buf = buf.into_raw();
            log_debug(
                b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                b"yylex_token\0" as *const u8 as *const ::core::ffi::c_char,
                buf,
            );
            return buf;
        }
    };
}
