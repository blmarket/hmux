use crate::arguments::args_free_value;
use crate::cmd::cmd_get_alias;
use crate::cmd::find::{cmd_find_from_client, cmd_find_valid_state};
use crate::cmd::queue::{CmdqStateRef, cmdq_append, cmdq_get_command, cmdq_print};
use crate::cmd::{
    cmd_list_append, cmd_list_append_all, cmd_list_move, cmd_list_new, cmd_list_print, cmd_parse,
};
use crate::environ::{environ_entry_value, environ_find, environ_put};
use crate::ffi::{
    __ctype_b_loc, getpwnam, getpwuid, getuid, sscanf, strchr, strcmp, strlen, wctomb,
};
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc};
use crate::format::format_true;
use crate::format::{format_create, format_defaults, format_expand};
use crate::list::foreach_owned;
use crate::log::{fatalx, log_debug};
use crate::tmux::global_environ;
pub use crate::types::*;
use crate::xmalloc::xasprintf;
pub type ctype_mask = ::core::ffi::c_uint;
pub const _ISalnum: ctype_mask = 8;
pub const _ISpunct: ctype_mask = 4;
pub const _IScntrl: ctype_mask = 2;
pub const _ISblank: ctype_mask = 1;
pub const _ISgraph: ctype_mask = 32768;
pub const _ISprint: ctype_mask = 16384;
pub const _ISspace: ctype_mask = 8192;
pub const _ISxdigit: ctype_mask = 4096;
pub const _ISdigit: ctype_mask = 2048;
pub const _ISalpha: ctype_mask = 1024;
pub const _ISlower: ctype_mask = 512;
pub const _ISupper: ctype_mask = 256;
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
pub const CMD_PARSE_SUCCESS: cmd_parse_status = 1;
pub const CMD_PARSE_ERROR: cmd_parse_status = 0;
/// The commands of one parse, in the order they were written. Each command
/// belongs to the list.
pub type cmd_parse_commands = ::std::vec::Vec<::std::boxed::Box<cmd_parse_command>>;

#[repr(C)]
pub struct cmd_parse_command {
    pub line: u_int,
    pub arguments: cmd_parse_arguments,
}

/// The arguments of one command, in the order they were written, and owned by
/// the command the same way.
pub type cmd_parse_arguments = ::std::vec::Vec<::std::boxed::Box<cmd_parse_argument>>;

#[repr(C)]
pub struct cmd_parse_argument {
    pub type_0: cmd_parse_argument_type,
    pub string: Option<::std::ffi::CString>,
    pub commands: Option<::std::boxed::Box<cmd_parse_commands>>,
    pub(crate) cmdlist: Option<CmdListRef>,
}
pub type cmd_parse_argument_type = ::core::ffi::c_uint;
pub const CMD_PARSE_PARSED_COMMANDS: cmd_parse_argument_type = 2;
pub const CMD_PARSE_COMMANDS: cmd_parse_argument_type = 1;
pub const CMD_PARSE_STRING: cmd_parse_argument_type = 0;
/// The state of the parser. The default is a parser reading nothing: no
/// input, no scope stack and no error.
#[derive(Clone, Default)]
#[repr(C)]
pub struct cmd_parse_state {
    /// The bytes of a file being parsed, read as unsigned characters the way
    /// `getc` did. A caller's buffer is held in `buf`/`len` instead.
    pub f: Option<Vec<u8>>,
    pub buf: *const ::core::ffi::c_char,
    pub len: size_t,
    pub off: size_t,
    pub condition: ::core::ffi::c_int,
    pub eol: ::core::ffi::c_int,
    pub eof: ::core::ffi::c_int,
    pub input: *mut cmd_parse_input,
    pub escapes: u_int,
    pub error: Option<::std::ffi::CString>,
}
pub type cmd_parse_token_state = ::core::ffi::c_uint;
pub const SINGLE_QUOTES: cmd_parse_token_state = 3;
pub const DOUBLE_QUOTES: cmd_parse_token_state = 2;
pub const NONE: cmd_parse_token_state = 1;
pub const START: cmd_parse_token_state = 0;
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const EOF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const ENVIRON_HIDDEN: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMD_PARSE_PARSEONLY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CMD_PARSE_NOALIAS: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMD_PARSE_VERBOSE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CMD_PARSE_ONEGROUP: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const FORMAT_NOJOBS: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const FORMAT_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CMD_PARSE_MAX_ENVIRON_LEN: ::core::ffi::c_int = 16384 as ::core::ffi::c_int;
static mut parse_state: cmd_parse_state = cmd_parse_state {
    f: None,
    buf: ::core::ptr::null::<::core::ffi::c_char>(),
    len: 0,
    off: 0,
    condition: 0,
    eol: 0,
    eof: 0,
    input: ::core::ptr::null::<cmd_parse_input>() as *mut cmd_parse_input,
    escapes: 0,
    error: None,
};
unsafe fn cmd_parse_get_error(
    mut file: *const ::core::ffi::c_char,
    mut line: u_int,
    mut error: *const ::core::ffi::c_char,
) -> ::std::ffi::CString {
    unsafe {
        if file.is_null() {
            ::std::ffi::CStr::from_ptr(error).to_owned()
        } else {
            xasprintf(c"%s:%u: %s".as_ptr(), fmt_args![file, line, error])
        }
    }
}
unsafe fn cmd_parse_print_commands(mut pi: *mut cmd_parse_input, mut cmdlist: *mut cmd_list) {
    unsafe {
        if (*pi).item.is_null() || !(*pi).flags & CMD_PARSE_VERBOSE != 0 {
            return;
        }
        let s = cmd_list_print(cmdlist, 0 as ::core::ffi::c_int);
        if (*pi).file.is_some() {
            cmdq_print(
                (*pi).item,
                c"%s:%u: %s".as_ptr(),
                fmt_args![(*pi).file(), (*pi).line, s.as_ptr()],
            );
        } else {
            cmdq_print(
                (*pi).item,
                c"%u: %s".as_ptr(),
                fmt_args![(*pi).line, s.as_ptr()],
            );
        }
    }
}
/// The list an argument carries, as the borrowed view the walks take, or null
/// for an argument that carries none.
fn commands_ptr(commands: &mut Option<Box<cmd_parse_commands>>) -> *mut cmd_parse_commands {
    commands
        .as_mut()
        .map(|cmds| &raw mut **cmds)
        .unwrap_or(::core::ptr::null_mut::<cmd_parse_commands>())
}

/// The commands of `cmds`, in order, as the borrowed pointers the walks over
/// a parse tree take, walked the way the C's `TAILQ_FOREACH` walked them.
unsafe fn command_list(
    cmds: *mut cmd_parse_commands,
) -> impl Iterator<Item = *mut cmd_parse_command> {
    unsafe { foreach_owned(cmds) }
}

/// The arguments of one command, the same way [`command_list`] gives its
/// commands.
unsafe fn argument_list(
    args: *mut cmd_parse_arguments,
) -> impl Iterator<Item = *mut cmd_parse_argument> {
    unsafe { foreach_owned(args) }
}

fn cmd_parse_new_argument() -> Box<cmd_parse_argument> {
    Box::new(cmd_parse_argument {
        type_0: CMD_PARSE_STRING,
        string: None,
        commands: None,
        cmdlist: None,
    })
}

fn cmd_parse_new_command(line: u_int) -> Box<cmd_parse_command> {
    Box::new(cmd_parse_command {
        line,
        arguments: cmd_parse_arguments::new(),
    })
}
lalrpop_util::lalrpop_mod!(parse_grammar, "/cmd/parse_grammar.rs");

/// A NUL-terminated token string owned by the parser.
#[derive(Debug)]
pub struct TokenText(::std::ffi::CString);
impl TokenText {
    fn from_cstring(text: ::std::ffi::CString) -> TokenText {
        TokenText(text)
    }
    pub fn as_ptr(&self) -> *const ::core::ffi::c_char {
        self.0.as_ptr()
    }
    pub fn as_c_str(&self) -> &::core::ffi::CStr {
        self.0.as_c_str()
    }
}
impl Clone for TokenText {
    fn clone(&self) -> TokenText {
        TokenText(self.0.clone())
    }
}

/// A terminal of the command grammar.
#[derive(Clone, Debug)]
pub enum Token {
    Newline,
    Semicolon,
    OpenBrace,
    CloseBrace,
    Hidden,
    If,
    Else,
    Elif,
    Endif,
    Format(TokenText),
    Token(TokenText),
    Equals(TokenText),
}

/// The parse aborted. `cmd_parse_run_parser` turns this into `syntax error`,
/// which `yyerror` keeps only if nothing has reported a better message first.
#[derive(Debug)]
pub struct LexError;

/// One argument of a command, before it is built into a `struct cmd`.
pub enum ParseArgument {
    String(TokenText),
    Commands(Vec<ParseCommand>),
}

/// One command, before it is built into a `struct cmd`.
pub struct ParseCommand {
    pub line: u_int,
    pub arguments: Vec<ParseArgument>,
}
impl ParseCommand {
    /// The `command : assignment` case, which carries no arguments.
    pub fn empty(line: u_int) -> ParseCommand {
        ParseCommand {
            line,
            arguments: Vec::new(),
        }
    }
    pub fn new(line: u_int, name: TokenText, arguments: Vec<ParseArgument>) -> ParseCommand {
        let mut arguments = arguments;
        arguments.insert(0, ParseArgument::String(name));
        ParseCommand { line, arguments }
    }
}

/// The value of an `elif` chain: whether a branch was taken, and its body.
pub struct ElifResult {
    pub flag: bool,
    pub commands: Vec<ParseCommand>,
}
impl ElifResult {
    pub fn taken(commands: Vec<ParseCommand>) -> ElifResult {
        ElifResult {
            flag: true,
            commands,
        }
    }
    pub fn skipped() -> ElifResult {
        ElifResult {
            flag: false,
            commands: Vec::new(),
        }
    }
}

pub fn concat(mut a: Vec<ParseCommand>, b: Vec<ParseCommand>) -> Vec<ParseCommand> {
    a.extend(b);
    a
}

pub fn prepend(a: ParseArgument, mut rest: Vec<ParseArgument>) -> Vec<ParseArgument> {
    rest.insert(0, a);
    rest
}

/// The `%if` scope stack, threaded through the grammar actions.
///
/// `scope` is the innermost `%if`, `stack` the enclosing ones, innermost
/// last.
pub struct ParseState {
    scope: Option<bool>,
    stack: Vec<bool>,
}

impl ParseState {
    pub fn new() -> ParseState {
        ParseState {
            scope: None,
            stack: Vec::new(),
        }
    }

    pub fn line(&self) -> u_int {
        unsafe { (*parse_state.input).line }
    }

    fn scope_active(&self) -> bool {
        self.scope.unwrap_or(true)
    }

    /// `statement : condition | commands` — a body under a false `%if` is
    /// discarded.
    pub fn keep_if_active(&self, commands: Vec<ParseCommand>) -> Vec<ParseCommand> {
        if self.scope_active() {
            commands
        } else {
            Vec::new()
        }
    }

    /// `commands : command`.
    pub fn start_commands(&self, command: ParseCommand) -> Vec<ParseCommand> {
        if !command.arguments.is_empty() && self.scope_active() {
            vec![command]
        } else {
            Vec::new()
        }
    }

    /// `commands : commands ';' command`. An argument-less trailing command
    /// discards the whole accumulated list, matching tmux.
    pub fn push_command(
        &self,
        mut commands: Vec<ParseCommand>,
        command: ParseCommand,
    ) -> Vec<ParseCommand> {
        if !command.arguments.is_empty() && self.scope_active() {
            commands.push(command);
            commands
        } else {
            Vec::new()
        }
    }

    /// `expanded : format`.
    pub fn expand_format(&mut self, token: TokenText) -> TokenText {
        unsafe {
            let pi: *mut cmd_parse_input = parse_state.input;
            let c: *mut client = (*pi).client();
            let mut fs = cmd_find_state::default();
            let fsp: *mut cmd_find_state = if cmd_find_valid_state(&(*pi).fs) != 0 {
                &raw mut (*pi).fs
            } else {
                cmd_find_from_client(&mut fs, c, 0 as ::core::ffi::c_int);
                &raw mut fs
            };
            let mut ft = format_create(
                ::core::ptr::null_mut::<client>(),
                (*pi).item,
                FORMAT_NONE,
                FORMAT_NOJOBS,
            );
            format_defaults(
                &mut ft,
                c,
                (*fsp).session(),
                (*fsp).winlink(),
                (*fsp).pane(),
            );
            let expanded = format_expand(&mut ft, token.as_c_str());
            TokenText::from_cstring(expanded)
        }
    }

    /// `assignment : EQUALS` and `hidden_assignment : HIDDEN EQUALS`.
    pub fn put_environ(&mut self, token: TokenText, hidden: bool) -> Result<(), LexError> {
        unsafe {
            let flags = (*parse_state.input).flags;
            let flag = match self.scope {
                None => true,
                Some(scope) => scope && self.stack.iter().all(|scope| *scope),
            };
            if strlen(token.as_ptr()) > CMD_PARSE_MAX_ENVIRON_LEN as size_t {
                yyerror(c"environment variable is too long".as_ptr(), fmt_args![]);
                return Err(LexError);
            }
            if !flags & CMD_PARSE_PARSEONLY != 0 && flag {
                environ_put(
                    global_environ,
                    token.as_ptr(),
                    if hidden {
                        ENVIRON_HIDDEN
                    } else {
                        0 as ::core::ffi::c_int
                    },
                );
            }
            Ok(())
        }
    }

    /// `if_open : IF expanded`.
    pub fn push_scope(&mut self, expanded: &TokenText) -> bool {
        let flag = unsafe { format_true(Some(expanded.as_c_str())) != 0 };
        if let Some(scope) = self.scope {
            self.stack.push(scope);
        }
        self.scope = Some(flag);
        flag
    }

    /// `if_else : ELSE`.
    pub fn invert_scope(&mut self) {
        self.scope = Some(!self.scope_active());
    }

    /// `if_elif : ELIF expanded`.
    pub fn replace_scope(&mut self, expanded: &TokenText) -> bool {
        let flag = unsafe { format_true(Some(expanded.as_c_str())) != 0 };
        self.scope = Some(flag);
        flag
    }

    /// `if_close : ENDIF`.
    pub fn pop_scope(&mut self) {
        self.scope = self.stack.pop();
    }
}

impl Default for ParseState {
    fn default() -> ParseState {
        ParseState::new()
    }
}

unsafe fn cmd_parse_build_arguments(args: *mut cmd_parse_arguments, arguments: Vec<ParseArgument>) {
    unsafe {
        (*args).clear();
        for argument in arguments {
            let mut arg = cmd_parse_new_argument();
            match argument {
                ParseArgument::String(string) => {
                    arg.type_0 = CMD_PARSE_STRING;
                    arg.string = Some(string.0);
                }
                ParseArgument::Commands(commands) => {
                    arg.type_0 = CMD_PARSE_COMMANDS;
                    arg.commands = Some(cmd_parse_build_list(commands));
                }
            }
            (*args).push(arg);
        }
    }
}

/// Moves an owned parse tree into the lists the command builder reads.
unsafe fn cmd_parse_build_list(commands: Vec<ParseCommand>) -> Box<cmd_parse_commands> {
    unsafe {
        let mut cmds = Box::new(cmd_parse_commands::new());
        for command in commands {
            let mut cmd = cmd_parse_new_command(command.line);
            cmd_parse_build_arguments(&raw mut cmd.arguments, command.arguments);
            cmds.push(cmd);
        }
        cmds
    }
}

unsafe fn cmd_parse_run_parser(
    cause: &mut Option<::std::ffi::CString>,
) -> Option<Box<cmd_parse_commands>> {
    unsafe {
        let mut ps = ParseState::new();
        let result = parse_grammar::LinesParser::new().parse(&mut ps, TokenStream);
        match result {
            Ok(commands) => Some(cmd_parse_build_list(commands)),
            Err(_) => {
                yyerror(c"syntax error".as_ptr(), fmt_args![]);
                let ps: *mut cmd_parse_state = &raw mut parse_state;
                *cause = (*ps).error.take();
                None
            }
        }
    }
}

/// Pulls one token at a time out of `yylex`, the way `yyparse` did.
struct TokenStream;

impl Iterator for TokenStream {
    type Item = Result<(usize, Token, usize), LexError>;

    fn next(&mut self) -> Option<Result<(usize, Token, usize), LexError>> {
        match yylex_next() {
            Ok(None) => None,
            Ok(Some(token)) => Some(Ok((0, token, 0))),
            Err(error) => Some(Err(error)),
        }
    }
}

/// Puts the parser back the way it was once a parse has finished, so that a
/// parse started from inside another one — a command line that parses one of
/// its own arguments — leaves the outer parse where it was.
struct ParserTurn;

impl ParserTurn {
    /// Starts a parse, saving whatever the parser was doing before.
    unsafe fn start(pi: *mut cmd_parse_input) -> (ParserTurn, cmd_parse_state) {
        unsafe {
            let outer = ::core::mem::take(&mut parse_state);
            parse_state.input = pi;
            (ParserTurn, outer)
        }
    }

    /// Gives the parser back to the parse that was under way.
    unsafe fn finish(self, outer: cmd_parse_state) {
        unsafe { parse_state = outer };
    }
}

unsafe fn cmd_parse_do_file(
    f: Vec<u8>,
    mut pi: *mut cmd_parse_input,
    cause: &mut Option<::std::ffi::CString>,
) -> Option<Box<cmd_parse_commands>> {
    unsafe {
        let (turn, outer) = ParserTurn::start(pi);
        parse_state.f = Some(f);
        let parsed = cmd_parse_run_parser(cause);
        turn.finish(outer);
        parsed
    }
}
unsafe fn cmd_parse_do_buffer(
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut pi: *mut cmd_parse_input,
    cause: &mut Option<::std::ffi::CString>,
) -> Option<Box<cmd_parse_commands>> {
    unsafe {
        let (turn, outer) = ParserTurn::start(pi);
        parse_state.buf = buf;
        parse_state.len = len;
        let parsed = cmd_parse_run_parser(cause);
        turn.finish(outer);
        parsed
    }
}
unsafe fn cmd_parse_log_commands(
    mut cmds: *mut cmd_parse_commands,
    mut prefix: *const ::core::ffi::c_char,
) {
    unsafe {
        let mut i: u_int = 0;
        let mut j: u_int = 0;
        i = 0 as u_int;
        for cmd in command_list(cmds) {
            j = 0 as u_int;
            for arg in argument_list(&raw mut (*cmd).arguments) {
                match (*arg).type_0 {
                    CMD_PARSE_STRING => {
                        log_debug(
                            c"%s %u:%u: %s".as_ptr(),
                            fmt_args![prefix, i, j, (*arg).string.as_ref().unwrap().as_ptr()],
                        );
                    }
                    CMD_PARSE_COMMANDS => {
                        let s = xasprintf(c"%s %u:%u".as_ptr(), fmt_args![prefix, i, j]);
                        cmd_parse_log_commands(commands_ptr(&mut (*arg).commands), s.as_ptr());
                    }
                    CMD_PARSE_PARSED_COMMANDS => {
                        let s = cmd_list_print(
                            (*arg)
                                .cmdlist
                                .as_ref()
                                .map_or(::core::ptr::null(), |list| list.as_ptr()),
                            0 as ::core::ffi::c_int,
                        );
                        log_debug(
                            c"%s %u:%u: %s".as_ptr(),
                            fmt_args![prefix, i, j, s.as_ptr()],
                        );
                    }
                    _ => {}
                }
                j = j.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
    }
}
unsafe fn cmd_parse_expand_alias(
    mut cmd: *mut cmd_parse_command,
    mut pi: *mut cmd_parse_input,
    pr: &mut cmd_parse_result,
) -> ::core::ffi::c_int {
    unsafe {
        let mut first: *mut cmd_parse_argument = ::core::ptr::null_mut::<cmd_parse_argument>();
        let mut last: *mut cmd_parse_command = ::core::ptr::null_mut::<cmd_parse_command>();
        let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut cause = None;
        if (*pi).flags & CMD_PARSE_NOALIAS != 0 {
            return 0 as ::core::ffi::c_int;
        }
        *pr = cmd_parse_result::default();
        first = (*cmd)
            .arguments
            .first()
            .map(|arg| &raw const **arg as *mut cmd_parse_argument)
            .unwrap_or(::core::ptr::null_mut::<cmd_parse_argument>());
        if first.is_null()
            || (*first).type_0 as ::core::ffi::c_uint
                != CMD_PARSE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            pr.status = CMD_PARSE_SUCCESS;
            pr.cmdlist = Some(cmd_list_new());
            return 1 as ::core::ffi::c_int;
        }
        name = (*first).string.as_ref().unwrap().as_ptr() as *mut ::core::ffi::c_char;
        let Some(alias) = cmd_get_alias(name) else {
            return 0 as ::core::ffi::c_int;
        };
        log_debug(
            c"%s: %u alias %s = %s".as_ptr(),
            fmt_args![
                c"cmd_parse_expand_alias".as_ptr(),
                (*pi).line,
                name,
                alias.as_ptr()
            ],
        );
        let Some(mut cmds) =
            cmd_parse_do_buffer(alias.as_ptr(), alias.as_bytes().len(), pi, &mut cause)
        else {
            pr.status = CMD_PARSE_ERROR;
            pr.error = cause;
            return 1 as ::core::ffi::c_int;
        };
        last = cmds
            .last()
            .map(|cmd| &raw const **cmd as *mut cmd_parse_command)
            .unwrap_or(::core::ptr::null_mut::<cmd_parse_command>());
        if last.is_null() {
            pr.status = CMD_PARSE_SUCCESS;
            pr.cmdlist = Some(cmd_list_new());
            return 1 as ::core::ffi::c_int;
        }
        drop((*cmd).arguments.remove(0));
        let moved = ::core::mem::take(&mut (*cmd).arguments);
        (*last).arguments.extend(moved);
        cmd_parse_log_commands(&raw mut *cmds, c"cmd_parse_expand_alias".as_ptr());
        (*pi).flags |= CMD_PARSE_NOALIAS;
        cmd_parse_build_commands(&raw mut *cmds, pi, pr);
        (*pi).flags &= !CMD_PARSE_NOALIAS;
        1 as ::core::ffi::c_int
    }
}
unsafe fn cmd_parse_build_command(
    mut cmd: *mut cmd_parse_command,
    mut pi: *mut cmd_parse_input,
    pr: &mut cmd_parse_result,
) {
    unsafe {
        let mut current_block: u64;
        let mut values = Vec::<args_value_t>::new();
        let mut count: u_int = 0 as u_int;
        let mut idx: u_int = 0;
        *pr = cmd_parse_result::default();
        if cmd_parse_expand_alias(cmd, pi, pr) != 0 {
            return;
        }
        current_block = 5143058163439228106;
        for arg in argument_list(&raw mut (*cmd).arguments) {
            values.push(args_value_t::default());
            let value = values.last_mut().unwrap();
            match (*arg).type_0 {
                CMD_PARSE_STRING => {
                    value.value = ArgsValue::String((*arg).string.as_ref().unwrap().clone());
                }
                CMD_PARSE_COMMANDS => {
                    cmd_parse_build_commands(commands_ptr(&mut (*arg).commands), pi, pr);
                    if pr.status as ::core::ffi::c_uint
                        != CMD_PARSE_SUCCESS as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        current_block = 689484554684886290;
                        break;
                    }
                    value.value = ArgsValue::Commands {
                        cmdlist: pr.cmdlist.clone(),
                        cached: None,
                    };
                }
                CMD_PARSE_PARSED_COMMANDS => {
                    value.value = ArgsValue::Commands {
                        cmdlist: (*arg).cmdlist.clone(),
                        cached: None,
                    };
                }
                _ => {}
            }
            count = count.wrapping_add(1);
        }
        if current_block == 5143058163439228106 {
            match cmd_parse(
                values.as_mut_ptr(),
                count,
                (*pi).file(),
                (*pi).line,
                (*pi).flags,
            ) {
                Ok(add) => {
                    pr.status = CMD_PARSE_SUCCESS;
                    pr.cmdlist = Some(cmd_list_new());
                    cmd_list_append(pr.cmdlist.as_ref().unwrap().as_ptr(), add);
                }
                Err(cause) => {
                    pr.status = CMD_PARSE_ERROR;
                    pr.error = Some(cmd_parse_get_error(
                        (*pi).file(),
                        (*pi).line,
                        cause.as_ptr(),
                    ));
                }
            }
        }
        idx = 0 as u_int;
        while idx < count {
            args_free_value(values.as_mut_ptr().offset(idx as isize));
            idx = idx.wrapping_add(1);
        }
    }
}
unsafe fn cmd_parse_build_commands(
    mut cmds: *mut cmd_parse_commands,
    mut pi: *mut cmd_parse_input,
    pr: &mut cmd_parse_result,
) {
    unsafe {
        let mut line: u_int = UINT_MAX;
        let mut current: Option<CmdListRef> = None;
        let mut result: Option<CmdListRef> = None;
        *pr = cmd_parse_result::default();
        if (*cmds).is_empty() {
            pr.status = CMD_PARSE_SUCCESS;
            pr.cmdlist = Some(cmd_list_new());
            return;
        }
        cmd_parse_log_commands(cmds, c"cmd_parse_build_commands".as_ptr());
        result = Some(cmd_list_new());
        for cmd in command_list(cmds) {
            if !(*pi).flags & CMD_PARSE_ONEGROUP != 0 && (*cmd).line != line {
                if let Some(current_list) = current.as_ref() {
                    cmd_parse_print_commands(pi, current_list.as_ptr());
                    cmd_list_move(result.as_ref().unwrap().as_ptr(), current_list.as_ptr());
                }
                current = Some(cmd_list_new());
            }
            if current.is_none() {
                current = Some(cmd_list_new());
            }
            (*pi).line = (*cmd).line;
            line = (*pi).line;
            cmd_parse_build_command(cmd, pi, pr);
            if pr.status as ::core::ffi::c_uint
                != CMD_PARSE_SUCCESS as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return;
            }
            cmd_list_append_all(
                current.as_ref().unwrap().as_ptr(),
                pr.cmdlist.as_ref().unwrap().as_ptr(),
            );
            let _ = pr.cmdlist.take();
        }
        if let Some(current_list) = current.as_ref() {
            cmd_parse_print_commands(pi, current_list.as_ptr());
            cmd_list_move(result.as_ref().unwrap().as_ptr(), current_list.as_ptr());
        }
        let s = cmd_list_print(result.as_ref().unwrap().as_ptr(), 0 as ::core::ffi::c_int);
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"cmd_parse_build_commands".as_ptr(), s.as_ptr()],
        );
        pr.status = CMD_PARSE_SUCCESS;
        pr.cmdlist = result.take();
    }
}
pub unsafe fn cmd_parse_from_file(f: Vec<u8>, mut pi: *mut cmd_parse_input) -> cmd_parse_result {
    unsafe {
        let mut input = cmd_parse_input::default();
        let mut cause = None;
        if pi.is_null() {
            pi = &raw mut input;
        }
        let mut pr = cmd_parse_result::default();
        let Some(mut cmds) = cmd_parse_do_file(f, pi, &mut cause) else {
            pr.status = CMD_PARSE_ERROR;
            pr.error = cause;
            return pr;
        };
        cmd_parse_build_commands(&raw mut *cmds, pi, &mut pr);
        pr
    }
}
pub unsafe fn cmd_parse_from_string(
    mut s: *const ::core::ffi::c_char,
    mut pi: *mut cmd_parse_input,
) -> cmd_parse_result {
    unsafe {
        let mut input = cmd_parse_input::default();
        if pi.is_null() {
            pi = &raw mut input;
        }
        (*pi).flags |= CMD_PARSE_ONEGROUP;
        cmd_parse_from_buffer(s, strlen(s), pi)
    }
}
pub(crate) unsafe fn cmd_parse_and_append(
    mut s: *const ::core::ffi::c_char,
    mut pi: *mut cmd_parse_input,
    mut c: *mut client,
    state: &CmdqStateRef,
    error: &mut Option<::std::ffi::CString>,
) -> cmd_parse_status {
    unsafe {
        let mut pr = cmd_parse_from_string(s, pi);
        match pr.status {
            CMD_PARSE_ERROR => {
                *error = pr.error.take();
            }
            CMD_PARSE_SUCCESS => {
                let cmdlist = pr.cmdlist.take().unwrap();
                cmdq_append(c, cmdq_get_command(&cmdlist, Some(state)));
            }
            _ => {}
        }
        pr.status
    }
}
pub unsafe fn cmd_parse_from_buffer(
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut pi: *mut cmd_parse_input,
) -> cmd_parse_result {
    unsafe {
        let mut input = cmd_parse_input::default();
        let mut cause = None;
        if pi.is_null() {
            pi = &raw mut input;
        }
        let mut pr = cmd_parse_result::default();
        if len == 0 as size_t {
            pr.status = CMD_PARSE_SUCCESS;
            pr.cmdlist = Some(cmd_list_new());
            return pr;
        }
        let Some(mut cmds) = cmd_parse_do_buffer(buf, len, pi, &mut cause) else {
            pr.status = CMD_PARSE_ERROR;
            pr.error = cause;
            return pr;
        };
        cmd_parse_build_commands(&raw mut *cmds, pi, &mut pr);
        pr
    }
}
pub unsafe fn cmd_parse_from_arguments(
    mut values: *mut args_value_t,
    mut count: u_int,
    mut pi: *mut cmd_parse_input,
) -> cmd_parse_result {
    unsafe {
        let mut input = cmd_parse_input::default();
        let mut cmd: Box<cmd_parse_command>;
        let mut i: u_int = 0;
        let mut end: ::core::ffi::c_int = 0;
        if pi.is_null() {
            pi = &raw mut input;
        }
        let mut pr = cmd_parse_result::default();
        let mut cmds = Box::new(cmd_parse_commands::new());
        cmd = cmd_parse_new_command((*pi).line);
        i = 0 as u_int;
        while i < count {
            end = 0 as ::core::ffi::c_int;
            if matches!(&(*values.offset(i as isize)).value, ArgsValue::String(_)) {
                let mut copy =
                    ::std::ffi::CStr::from_ptr((*values.offset(i as isize)).value.string())
                        .to_bytes()
                        .to_vec();
                let mut size = copy.len();
                if size != 0 && copy[size - 1] as ::core::ffi::c_int == ';' as i32 {
                    size -= 1;
                    copy.truncate(size);
                    if size > 0 && copy[size - 1] as ::core::ffi::c_int == '\\' as i32 {
                        copy[size - 1] = b';';
                    } else {
                        end = 1 as ::core::ffi::c_int;
                    }
                }
                if end == 0 || size != 0 {
                    let mut arg = cmd_parse_new_argument();
                    arg.type_0 = CMD_PARSE_STRING;
                    arg.string =
                        Some(::std::ffi::CString::new(copy).expect("command argument has no NUL"));
                    cmd.arguments.push(arg);
                } else {
                    drop(copy);
                }
            } else if let ArgsValue::Commands { cmdlist, .. } = &(*values.offset(i as isize)).value
            {
                let mut arg = cmd_parse_new_argument();
                arg.type_0 = CMD_PARSE_PARSED_COMMANDS;
                arg.cmdlist = cmdlist.clone();
                cmd.arguments.push(arg);
            } else {
                fatalx(c"unknown argument type".as_ptr(), fmt_args![]);
            }
            if end != 0 {
                cmds.push(cmd);
                cmd = cmd_parse_new_command((*pi).line);
            }
            i = i.wrapping_add(1);
        }
        if !cmd.arguments.is_empty() {
            cmds.push(cmd);
        } else {
            drop(cmd);
        }
        cmd_parse_build_commands(&raw mut *cmds, pi, &mut pr);
        pr
    }
}
unsafe fn yyerror(mut fmt: *const ::core::ffi::c_char, args: &[FmtArg]) {
    unsafe {
        let mut ps: *mut cmd_parse_state = &raw mut parse_state;
        let mut pi: *mut cmd_parse_input = (*ps).input;
        if (*ps).error.is_some() {
            return;
        }
        let error = format_alloc(fmt, args);
        (*ps).error = Some(cmd_parse_get_error(
            (*pi).file(),
            (*pi).line,
            error.as_ptr(),
        ));
    }
}
fn yylex_is_var(mut ch: ::core::ffi::c_char, mut first: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
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
        (*(*__ctype_b_loc()).offset(ch as u_char as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
            || ch as ::core::ffi::c_int == '_' as i32) as ::core::ffi::c_int
    }
}
/// The collected bytes as a C string, which ends at the first NUL the same
/// way reading the lexer's NUL-terminated buffer did.
fn yylex_cstring(mut buf: Vec<u8>) -> ::std::ffi::CString {
    if let Some(nul) = buf.iter().position(|&byte| byte == 0) {
        buf.truncate(nul);
    }
    unsafe { ::std::ffi::CString::from_vec_unchecked(buf) }
}
fn yylex_getc1() -> ::core::ffi::c_int {
    unsafe {
        let mut ps: *mut cmd_parse_state = &raw mut parse_state;
        let mut ch: ::core::ffi::c_int = 0;
        if let Some(file) = (*ps).f.as_ref() {
            let off = (*ps).off;
            if off == file.len() {
                ch = EOF;
            } else {
                ch = file[off] as ::core::ffi::c_int;
                (*ps).off = off.wrapping_add(1);
            }
        } else if (*ps).off == (*ps).len {
            ch = EOF;
        } else {
            let fresh27 = (*ps).off;
            (*ps).off = (*ps).off.wrapping_add(1);
            ch = *(*ps).buf.add(fresh27) as ::core::ffi::c_int;
        }
        ch
    }
}
fn yylex_ungetc(mut ch: ::core::ffi::c_int) {
    unsafe {
        let mut ps: *mut cmd_parse_state = &raw mut parse_state;
        if (*ps).off > 0 as size_t && ch != EOF {
            (*ps).off = (*ps).off.wrapping_sub(1);
        }
    }
}
fn yylex_getc() -> ::core::ffi::c_int {
    unsafe {
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
}
fn yylex_get_word(mut ch: ::core::ffi::c_int) -> ::std::ffi::CString {
    unsafe {
        let mut buf: Vec<u8> = Vec::new();
        loop {
            buf.push(ch as u8);
            ch = yylex_getc();
            if !(ch != EOF && strchr(c" \t\n".as_ptr(), ch).is_null()) {
                break;
            }
        }
        yylex_ungetc(ch);
        let word = yylex_cstring(buf);
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"yylex_get_word".as_ptr(), word.as_ptr()],
        );
        word
    }
}
fn yylex_next() -> Result<Option<Token>, LexError> {
    unsafe {
        let ps: *mut cmd_parse_state = &raw mut parse_state;
        let mut ch: ::core::ffi::c_int;
        let mut next: ::core::ffi::c_int;
        if (*ps).eol != 0 {
            (*(*ps).input).line = (*(*ps).input).line.wrapping_add(1);
        }
        (*ps).eol = 0 as ::core::ffi::c_int;
        let condition = (*ps).condition;
        (*ps).condition = 0 as ::core::ffi::c_int;
        loop {
            ch = yylex_getc();
            if ch == EOF {
                if (*ps).eof != 0 {
                    return Ok(None);
                }
                (*ps).eof = 1 as ::core::ffi::c_int;
                return Ok(Some(Token::Newline));
            }
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
                return Ok(Some(Token::Newline));
            }
            if ch == ';' as i32 {
                return Ok(Some(Token::Semicolon));
            }
            if ch == '{' as i32 {
                return Ok(Some(Token::OpenBrace));
            }
            if ch == '}' as i32 {
                return Ok(Some(Token::CloseBrace));
            }
            if ch == '#' as i32 {
                next = yylex_getc();
                if condition != 0 && next == '{' as i32 {
                    let Some(token) = yylex_format() else {
                        return Err(LexError);
                    };
                    return Ok(Some(Token::Format(TokenText::from_cstring(token))));
                }
                while next != '\n' as i32 && next != EOF {
                    next = yylex_getc();
                }
                if next == '\n' as i32 {
                    (*(*ps).input).line = (*(*ps).input).line.wrapping_add(1);
                    return Ok(Some(Token::Newline));
                }
                continue;
            }
            if ch == '%' as i32 {
                let word = TokenText::from_cstring(yylex_get_word('%' as i32));
                let mut cp = word.as_ptr();
                while *cp as ::core::ffi::c_int != '\0' as i32 {
                    if *cp as ::core::ffi::c_int != '%' as i32
                        && *(*__ctype_b_loc()).offset(*cp as u_char as ::core::ffi::c_int as isize)
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
                    return Ok(Some(Token::Token(word)));
                }
                (*ps).condition = 1 as ::core::ffi::c_int;
                if strcmp(word.as_ptr(), c"%hidden".as_ptr()) == 0 as ::core::ffi::c_int {
                    return Ok(Some(Token::Hidden));
                }
                if strcmp(word.as_ptr(), c"%if".as_ptr()) == 0 as ::core::ffi::c_int {
                    return Ok(Some(Token::If));
                }
                if strcmp(word.as_ptr(), c"%else".as_ptr()) == 0 as ::core::ffi::c_int {
                    return Ok(Some(Token::Else));
                }
                if strcmp(word.as_ptr(), c"%elif".as_ptr()) == 0 as ::core::ffi::c_int {
                    return Ok(Some(Token::Elif));
                }
                if strcmp(word.as_ptr(), c"%endif".as_ptr()) == 0 as ::core::ffi::c_int {
                    return Ok(Some(Token::Endif));
                }
                return Err(LexError);
            }
            let Some(token) = yylex_token(ch) else {
                return Err(LexError);
            };
            let token = TokenText::from_cstring(token);
            if !strchr(token.as_ptr(), '=' as i32).is_null()
                && yylex_is_var(*token.as_ptr(), 1 as ::core::ffi::c_int) != 0
            {
                let mut cp = token.as_ptr().offset(1 as ::core::ffi::c_int as isize);
                while *cp as ::core::ffi::c_int != '=' as i32 {
                    if yylex_is_var(*cp, 0 as ::core::ffi::c_int) == 0 {
                        break;
                    }
                    cp = cp.offset(1);
                }
                if *cp as ::core::ffi::c_int == '=' as i32 {
                    return Ok(Some(Token::Equals(token)));
                }
            }
            return Ok(Some(Token::Token(token)));
        }
    }
}
fn yylex_format() -> Option<::std::ffi::CString> {
    unsafe {
        let mut current_block: u64;
        let mut buf: Vec<u8> = Vec::new();
        let mut ch: ::core::ffi::c_int = 0;
        let mut brackets: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        buf.extend_from_slice(b"#{");
        loop {
            ch = yylex_getc();
            if ch == EOF || ch == '\n' as i32 {
                current_block = 13016994178946890092;
                break;
            }
            if ch == '#' as i32 {
                ch = yylex_getc();
                if ch == EOF || ch == '\n' as i32 {
                    current_block = 13016994178946890092;
                    break;
                }
                if ch == '{' as i32 {
                    brackets += 1;
                }
                buf.push(b'#');
            } else if ch == '}' as i32 && brackets != 0 as ::core::ffi::c_int && {
                brackets -= 1;
                brackets == 0 as ::core::ffi::c_int
            } {
                buf.push(ch as u8);
                current_block = 10048703153582371463;
                break;
            }
            buf.push(ch as u8);
        }
        match current_block {
            10048703153582371463 if !(brackets != 0 as ::core::ffi::c_int) => {
                let token = yylex_cstring(buf);
                log_debug(
                    c"%s: %s".as_ptr(),
                    fmt_args![c"yylex_format".as_ptr(), token.as_ptr()],
                );
                return Some(token);
            }
            _ => {}
        }
        None
    }
}
unsafe fn yylex_token_escape(buf: &mut Vec<u8>) -> ::core::ffi::c_int {
    unsafe {
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
            yyerror(c"invalid octal escape".as_ptr(), fmt_args![]);
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
                    buf.push(ch as u8);
                    return 1 as ::core::ffi::c_int;
                }
            }
            yyerror(c"invalid octal escape".as_ptr(), fmt_args![]);
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
                current_block = 17113274278584595704;
            }
            85 => {
                type_0 = 'U' as i32;
                size = 8 as u_int;
                current_block = 17113274278584595704;
            }
            _ => {
                current_block = 17281240262373992796;
            }
        }
        match current_block {
            17113274278584595704 => {
                i = 0 as u_int;
                while i < size {
                    ch = yylex_getc();
                    if ch == EOF || ch == '\n' as i32 {
                        return 0 as ::core::ffi::c_int;
                    }
                    if *(*__ctype_b_loc()).offset(ch as u_char as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        & _ISxdigit as ::core::ffi::c_int as ::core::ffi::c_ushort
                            as ::core::ffi::c_int
                        == 0
                    {
                        yyerror(c"invalid \\%c argument".as_ptr(), fmt_args![type_0]);
                        return 0 as ::core::ffi::c_int;
                    }
                    s[i as usize] = ch as ::core::ffi::c_char;
                    i = i.wrapping_add(1);
                }
                s[i as usize] = '\0' as i32 as ::core::ffi::c_char;
                if size == 4 as u_int
                    && sscanf(
                        &raw mut s as *mut ::core::ffi::c_char,
                        c"%4x".as_ptr(),
                        &raw mut tmp,
                    ) != 1 as ::core::ffi::c_int
                    || size == 8 as u_int
                        && sscanf(
                            &raw mut s as *mut ::core::ffi::c_char,
                            c"%8x".as_ptr(),
                            &raw mut tmp,
                        ) != 1 as ::core::ffi::c_int
                {
                    yyerror(c"invalid \\%c argument".as_ptr(), fmt_args![type_0]);
                    return 0 as ::core::ffi::c_int;
                }
                mlen = wctomb(&raw mut m as *mut ::core::ffi::c_char, tmp as wchar_t);
                if mlen <= 0 as ::core::ffi::c_int
                    || mlen
                        > ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as ::core::ffi::c_int
                {
                    yyerror(c"invalid \\%c argument".as_ptr(), fmt_args![type_0]);
                    return 0 as ::core::ffi::c_int;
                }
                buf.extend(m[..mlen as usize].iter().map(|&byte| byte as u8));
                1 as ::core::ffi::c_int
            }
            _ => {
                buf.push(ch as u8);
                1 as ::core::ffi::c_int
            }
        }
    }
}
unsafe fn yylex_token_variable(buf: &mut Vec<u8>) -> ::core::ffi::c_int {
    unsafe {
        let mut envent: Option<&environ_entry> = None;
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
                buf.push(b'$');
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
                    yyerror(c"invalid environment variable".as_ptr(), fmt_args![]);
                    return 0 as ::core::ffi::c_int;
                }
            } else {
                if namelen
                    == (::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as usize)
                        .wrapping_sub(2_usize)
                {
                    yyerror(c"environment variable is too long".as_ptr(), fmt_args![]);
                    return 0 as ::core::ffi::c_int;
                }
                let fresh29 = namelen;
                namelen = namelen.wrapping_add(1);
                name[fresh29 as usize] = ch as ::core::ffi::c_char;
            }
        }
        name[namelen as usize] = '\0' as i32 as ::core::ffi::c_char;
        envent = environ_find(&*global_environ, &raw mut name as *mut ::core::ffi::c_char);
        if envent.is_some_and(|envent| !environ_entry_value(envent).is_null()) {
            value = environ_entry_value(envent.expect("the entry just looked at"));
            log_debug(
                c"%s: %s -> %s".as_ptr(),
                fmt_args![
                    c"yylex_token_variable".as_ptr(),
                    &raw mut name as *mut ::core::ffi::c_char,
                    value
                ],
            );
            buf.extend_from_slice(::std::ffi::CStr::from_ptr(value).to_bytes());
        }
        1 as ::core::ffi::c_int
    }
}
unsafe fn yylex_token_tilde(buf: &mut Vec<u8>) -> ::core::ffi::c_int {
    unsafe {
        let mut envent: Option<&environ_entry> = None;
        let mut ch: ::core::ffi::c_int = 0;
        let mut name: [::core::ffi::c_char; 1024] = [0; 1024];
        let mut namelen: size_t = 0 as size_t;
        let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
        let mut home: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        loop {
            ch = yylex_getc();
            if ch == EOF || !strchr(c"/ \t\n\"'".as_ptr(), ch).is_null() {
                yylex_ungetc(ch);
                break;
            } else {
                if namelen
                    == (::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as usize)
                        .wrapping_sub(2_usize)
                {
                    yyerror(c"user name is too long".as_ptr(), fmt_args![]);
                    return 0 as ::core::ffi::c_int;
                }
                let fresh30 = namelen;
                namelen = namelen.wrapping_add(1);
                name[fresh30 as usize] = ch as ::core::ffi::c_char;
            }
        }
        name[namelen as usize] = '\0' as i32 as ::core::ffi::c_char;
        if *(&raw mut name as *mut ::core::ffi::c_char) as ::core::ffi::c_int == '\0' as i32 {
            envent = environ_find(&*global_environ, c"HOME".as_ptr());
            if envent.is_some_and(|envent| {
                !environ_entry_value(envent).is_null()
                    && *environ_entry_value(envent) as ::core::ffi::c_int != '\0' as i32
            }) {
                home = environ_entry_value(envent.expect("the entry just looked at"));
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
            c"%s: ~%s -> %s".as_ptr(),
            fmt_args![
                c"yylex_token_tilde".as_ptr(),
                &raw mut name as *mut ::core::ffi::c_char,
                home
            ],
        );
        buf.extend_from_slice(::std::ffi::CStr::from_ptr(home).to_bytes());
        1 as ::core::ffi::c_int
    }
}
fn yylex_token(mut ch: ::core::ffi::c_int) -> Option<::std::ffi::CString> {
    unsafe {
        let mut current_block: u64;
        let mut ps: *mut cmd_parse_state = &raw mut parse_state;
        let mut buf: Vec<u8> = Vec::new();
        let mut state: cmd_parse_token_state = NONE;
        let mut last: cmd_parse_token_state = START;
        loop {
            if ch == EOF {
                log_debug(
                    c"%s: end at EOF".as_ptr(),
                    fmt_args![c"yylex_token".as_ptr()],
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
                    if state as ::core::ffi::c_uint
                        == NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        log_debug(
                            c"%s: end at EOL".as_ptr(),
                            fmt_args![c"yylex_token".as_ptr()],
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
                        c"%s: end at WS".as_ptr(),
                        fmt_args![c"yylex_token".as_ptr()],
                    );
                    current_block = 13321564401369230990;
                    break;
                } else if state as ::core::ffi::c_uint
                    == NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                    && (ch == ';' as i32 || ch == '}' as i32)
                {
                    log_debug(
                        c"%s: end at %c".as_ptr(),
                        fmt_args![c"yylex_token".as_ptr(), ch],
                    );
                    current_block = 13321564401369230990;
                    break;
                } else if ch == '\n' as i32
                    && state as ::core::ffi::c_uint
                        != NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    buf.push(b'\n');
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
                    if !strchr(c",#{}:".as_ptr(), ch).is_null() {
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
                            current_block = 11768010348333939680;
                            break;
                        }
                        current_block = 9512337080773452662;
                    } else if ch == '~' as i32
                        && last as ::core::ffi::c_uint != state as ::core::ffi::c_uint
                        && state as ::core::ffi::c_uint
                            != SINGLE_QUOTES as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        if yylex_token_tilde(&mut buf) == 0 {
                            current_block = 11768010348333939680;
                            break;
                        }
                        current_block = 9512337080773452662;
                    } else if ch == '$' as i32
                        && state as ::core::ffi::c_uint
                            != SINGLE_QUOTES as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        if yylex_token_variable(&mut buf) == 0 {
                            current_block = 11768010348333939680;
                            break;
                        }
                        current_block = 9512337080773452662;
                    } else {
                        if ch == '}' as i32
                            && state as ::core::ffi::c_uint
                                == NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            current_block = 11768010348333939680;
                            break;
                        }
                        if ch == '\'' as i32 {
                            if state as ::core::ffi::c_uint
                                == NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                state = SINGLE_QUOTES;
                                current_block = 12867991516770085914;
                            } else if state as ::core::ffi::c_uint
                                == SINGLE_QUOTES as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                state = NONE;
                                current_block = 12867991516770085914;
                            } else {
                                current_block = 1847472278776910194;
                            }
                        } else {
                            current_block = 1847472278776910194;
                        }
                        match current_block {
                            12867991516770085914 => {}
                            _ => {
                                if ch == '"' as i32 {
                                    if state as ::core::ffi::c_uint
                                        == NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        state = DOUBLE_QUOTES;
                                        current_block = 12867991516770085914;
                                    } else if state as ::core::ffi::c_uint
                                        == DOUBLE_QUOTES as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        state = NONE;
                                        current_block = 12867991516770085914;
                                    } else {
                                        current_block = 14220266465818359136;
                                    }
                                } else {
                                    current_block = 14220266465818359136;
                                }
                                match current_block {
                                    12867991516770085914 => {}
                                    _ => {
                                        buf.push(ch as u8);
                                        current_block = 9512337080773452662;
                                    }
                                }
                            }
                        }
                    }
                    if current_block == 9512337080773452662 {
                        last = state;
                    }
                    ch = yylex_getc();
                }
            }
        }
        match current_block {
            11768010348333939680 => None,
            _ => {
                yylex_ungetc(ch);
                let token = yylex_cstring(buf);
                log_debug(
                    c"%s: %s".as_ptr(),
                    fmt_args![c"yylex_token".as_ptr(), token.as_ptr()],
                );
                Some(token)
            }
        }
    }
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
